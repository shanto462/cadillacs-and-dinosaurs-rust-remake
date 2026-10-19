//! Players: per-frame update (0x18834 / 0x18952) and the whole state machine
//! (0x18A82 .. 0x1D940). Owner: player port. Spec: `re/specs/player/SPEC.md`.
//!
//! Reading guide
//! * Every handler names its original routine. `*_entry` routines set a state and usually
//!   **fall through** into the handler in the same frame, exactly like the 68000 code (the
//!   handler code follows the entry in ROM). Where the original does not fall through, the
//!   entry returns.
//! * A few checks (special move 0x63AA, jump attack 0x1A704, wall jump 0x1A744) pop the caller's
//!   return address in the original. Here they return `true` and the caller returns at once.
//! * Character ids (+$03): 0 Jack, 1 Mustapha, 2 Hannah, 3 Mess.
//! * Held weapons, pick-up and the split-body layer are in `player_weapon.rs` (a child module).
//! * Unported neighbours (effects, scripted scenes) are marked `TODO(...)`.

use super::body::Body;
use super::{Game, Handle, Slot, slot_of};
use dino_assets::player::PlayerData;

#[path = "player_weapon.rs"]
mod weapon;
// The scripted control of the heroes (M2-08, M2-09): a child module, it runs the scripted player states (0x2A, 0x2C, 0x18, 0x34 ...).
#[path = "script.rs"]
pub mod script;

/// Input bits (G.$16..$18, active high).
pub const RIGHT: u8 = 1;
pub const LEFT: u8 = 2;
pub const DOWN: u8 = 4;
pub const UP: u8 = 8;
pub const ATK: u8 = 0x10;
pub const JMP: u8 = 0x20;

/// Player object (0x180 bytes in the original). `b` holds the shared header; the fields below
/// are player-only. Offsets in the doc comments.
#[derive(Clone, Debug, Default)]
pub struct Player {
    pub b: Body,
    /// +$94 the obstacle (container) the terrain test last ran into (0x12E9A).
    pub w94: Handle,
    /// +$73 frames until the held enemy escapes (refreshed by attack presses).
    pub escape: u8,
    /// +$75 last enemy frame byte used to place a held player.
    pub b75: u8,
    /// +$76 enemy that claimed this downed player (0x6606, enemy AI).
    pub w76: Handle,
    /// +$7A cleared by hits.
    pub b7a: u8,
    /// +$7E / +$7F claim flags (0x6606), cleared after a dizzy.
    pub b7e: u8,
    pub b7f: u8,
    /// +$80 (byte) / +$81 (byte); the dizzy code reads both as one word.
    pub w80: u16,
    /// +$83 water splash done.
    pub b83: u8,
    /// +$88 distance scratch (0x6606); the target X of the scripted jump (state 0x34, 0x1DD4C).
    pub w88: u16,
    /// +$8A the target Z of the scripted jump.
    pub w8a: u16,
    /// +$90 state-local step.
    pub b90: u8,
    /// +$A0 input held, +$A1 previous, +$A8 newly pressed.
    pub held: u8,
    pub prev: u8,
    pub pressed: u8,
    /// +$A3 extra lives.
    pub lives: u8,
    /// +$A7 facing last frame, +$AB frames since the facing changed.
    pub last_facing: u8,
    pub facing_age: u8,
    /// +$A9 grab push-away timer / run release grace.
    pub ba9: u8,
    /// +$AA frames attack held (stops counting when bit 7 is set).
    pub atk_hold: u8,
    /// +$AC previous run direction.
    pub run_dir: u8,
    /// +$AD buffered attack press (combo), +$AE combo window timer.
    pub atk_buf: u8,
    pub combo_t: u8,
    /// +$AF wall contact latch (wall jump).
    pub wall_latch: u8,
    /// +$118 voice bank (0xA502): selects the sound codes of the voice routines.
    pub voice: u8,
    /// +$B2 combo hit count, +$B3 post-whiff movement lock.
    pub combo: u8,
    pub lock: u8,
    /// +$B4 held item object, +$B6 held item type (0x2xx).
    pub item: Handle,
    pub item_type: u16,
    /// +$B8 pickup candidate object, +$BA its type (0x8000 = none).
    pub cand: Handle,
    pub cand_type: u16,
    /// +$BC double-tap timer, +$BD stick nibble at the last walk start.
    pub dtap: u8,
    pub walk_in: u8,
    /// +$BE mash meter, +$BF mash cooldown.
    pub mash: u8,
    pub mash_cd: u8,
    /// +$C0, +$C1 (set by stage code) [U].
    pub bc0: u8,
    pub bc1: u8,
    /// +$C4 weapon attack active, +$C5 drop weapon request.
    pub weapon_atk: u8,
    pub drop: u8,
    /// +$C6 attack power factor (x/32).
    pub power: u8,
    /// +$C7 [U] (held-by-enemy variant).
    pub bc7: u8,
    /// +$C8 action class read by other code (0x10 attack, 0x20 hurt, ...).
    pub action: u8,
    /// +$C9 buffered special request (10 frames).
    pub special_buf: u8,
    /// +$CA hits taken recently, +$CB its 60-frame window.
    pub hits: u8,
    pub hits_t: u8,
    /// +$CC tech roll flag, +$CD 1 = dizzy after getup / 2 = dying (slow fall).
    pub tech: u8,
    pub slow: u8,
    /// +$CE dizzy immunity timer (word; dizzy blocked while its high byte is nonzero).
    pub dizzy_cd: u16,
    /// +$D0 copy of G.$4DD when scripted control started.
    pub bd0: u8,
    /// +$D1 .. +$DE the scripted control of this hero (`script.rs`).
    pub sc: script::HeroScript,
    /// +$E8 lie-down timer, +$E9 serial of the last attack that hit me.
    pub lie_t: u8,
    pub hit_serial: u8,
    /// +$EA aim target (guns).
    pub aim: Handle,
    /// +$EC respawn invulnerability timer, +$EE special HP-cost budget.
    pub respawn_t: u16,
    pub special_budget: u16,
    /// +$F0 re-grab cooldown, +$F1 command move detected, +$F4 / +$F5 command FSM.
    pub regrab_cd: u8,
    pub cmd_flag: u8,
    pub cmd_state: u8,
    pub cmd_t: u8,
    /// +$F6 stun refill timer, +$F8 weapon pickup cooldown, +$F9 invulnerability timer.
    pub stun_refill: u16,
    pub pickup_cd: u8,
    pub invuln: u8,
    /// +$FA run time left, +$FC run cooldown.
    pub run_t: u16,
    pub run_cd: u16,
    /// +$100 score (BCD).
    pub score: u32,
    /// +$109 mash scratch, +$10A (0x33 at init) [U], +$10C, +$10E/+$11C/+$11D timed status [U].
    pub b109: u8,
    pub b10a: u8,
    pub w10c: u16,
    pub b10e: u8,
    pub b10f: u8,
    /// +$115 knee counter while grabbing, +$116, +$117 turn after pushback.
    pub knees: u8,
    pub b116: u8,
    pub turn: u8,
    /// +$119 the car steering table timer (0x8C346: 0x3D at a new press of a direction, counts down while the stick is held).
    pub b119: u8,
    /// +$11A input lock, +$11B, +$11C / +$11D timers.
    pub input_lock: u8,
    pub b11b: u8,
    pub b11c: u8,
    pub b11d: u8,
}

impl Player {
    pub fn b80(&self) -> u8 {
        (self.w80 >> 8) as u8
    }
    pub fn set_b80(&mut self, v: u8) {
        self.w80 = (self.w80 & 0x00FF) | (v as u16) << 8;
    }
    pub fn b81(&self) -> u8 {
        self.w80 as u8
    }
    pub fn set_b81(&mut self, v: u8) {
        self.w80 = (self.w80 & 0xFF00) | v as u16;
    }
    /// Writes only the integer word of a 16.16 coordinate (`move.w d0,$c(a6)`).
    fn set_hi(v: &mut i32, w: i16) {
        *v = ((w as i32) << 16) | (*v & 0xFFFF);
    }
}

/// Facing helper for "negate when facing left / right" vector loads.
fn signed(v: i16, neg: bool) -> i16 {
    if neg { v.wrapping_neg() } else { v }
}

impl Game {
    fn player_pd(&self) -> &'static PlayerData {
        &self.assets.player
    }

    // ---------------------------------------------------------------------------------------
    // Small shared helpers
    // ---------------------------------------------------------------------------------------

    /// 0x13A0 .. 0x13CC: start animation `id` of player set `set` (set 2 = 0x13B0 also clears $25).
    fn player_panim(&mut self, i: usize, set: usize, id: u16) {
        let s = &self.assets.sprites;
        let p = &mut self.players[i];
        if set == 2 {
            p.b.draw_flags = 0;
        }
        p.b.start_player_anim(s, set, id);
    }

    /// 0x1426 animation tick; true = finished (carry).
    fn player_pstep(&mut self, i: usize) -> bool {
        let s = &self.assets.sprites;
        self.players[i].b.tick_anim(s)
    }

    /// 0x1426 plus the low byte of the 68000's d0 at its return: the composition word of the
    /// frame that was loaded when the timer ran out (`w1` unless it is negative, else `w0`), else
    /// 0 (the low byte of the Z motion left in d0 by 0x189C). 0x1BD94 ends with it, and a caller
    /// stores it (0x1A220: `move.b d0,$b2(a6)`).
    fn player_pstep_d0(&mut self, i: usize) -> (bool, u8) {
        let loaded = self.players[i].b.timer.wrapping_sub(1) == 0;
        let carry = self.player_pstep(i);
        let d0 = if loaded {
            let f = self.assets.sprites.frame(self.players[i].b.frame);
            (if (f.w1 as i16) >= 0 { f.w1 } else { f.w0 }) as u8
        } else {
            0
        };
        (carry, d0)
    }

    /// 0x189C.
    fn player_pmove(&mut self, i: usize) {
        self.players[i].b.motion();
    }

    /// Attack start: `$58 = id; $59 = 0; $5E = G.$67BE++`.
    fn player_new_attack(&mut self, i: usize, id: u8) {
        let serial = self.w.attack_serial;
        self.w.attack_serial = serial.wrapping_add(1);
        let p = &mut self.players[i];
        p.b.atk = id;
        p.b.hitstop = 0;
        p.b.serial = serial;
    }

    /// Player voice / effect routines 0x95CE..0x9C0C (`snd.rs`): `i` is the player (a6).
    fn player_sfx(&mut self, i: usize, routine: u32) {
        self.snd_player(i, routine);
    }

    /// 0x192A2: face the held left/right direction (right wins when both are held).
    fn player_face_input(&mut self, i: usize) {
        let p = &mut self.players[i];
        let d = p.held & 3;
        if d != 0 {
            p.b.facing = d & 1;
        }
    }

    /// 0x5036: select the walk / run speed table (class 0, 4, 8, 12). The table is kept as the
    /// address of its pointer slot (0x5064 + 4 * index, 0x50A4.. while carrying 0x20A).
    /// 0x5036: +$64 = long at 0x5064 (0x50A4 while carrying object 0x20A) + class + $20, i.e.
    /// the ROM pointer of the speed table, as the original stores it.
    fn player_set_speed_class(&mut self, i: usize, class: u16) {
        let pd = &self.assets.player;
        let p = &mut self.players[i];
        // ext.w d0; add d0,d0; add d0,d0; add $20,d0: entry = class + character.
        let idx = (class.wrapping_mul(4).wrapping_add(p.b.k20) / 4) as usize & 15;
        p.b.speed_table = if p.item_type == 0x20A { pd.speed_heavy_ptr[idx] } else { pd.speed_ptr[idx] };
    }

    /// Entry `off` (byte offset / 4) of the speed table at +$64.
    fn player_speed_entry(&self, i: usize, off: u8) -> [i16; 2] {
        let t = self.players[i].b.speed_table;
        let pd = self.player_pd();
        let tab = match pd.speed_ptr.iter().position(|&p| p == t) {
            Some(k) => &pd.speed[k],
            None => {
                let k = pd.speed_heavy_ptr.iter().position(|&p| p == t).unwrap_or_else(|| panic!("player speed table {t:#x} unknown"));
                &pd.speed_heavy[k]
            }
        };
        tab[(off as usize / 4) & 7]
    }

    /// 0x1BD94 walk_move: face, take the direction's speed, move, animate. Returns the low byte
    /// of d0 the 68000 routine leaves behind (`player_pstep_d0`).
    fn player_walk_move(&mut self, i: usize, nib: u8) -> u8 {
        self.player_face_input_nib(i, nib);
        let [off, dir] = self.player_pd().walk_dir[nib as usize & 15];
        self.players[i].b.dir = dir;
        let [vx, vz] = self.player_speed_entry(i, off);
        let p = &mut self.players[i];
        p.b.vx = vx;
        p.b.vz = vz;
        self.player_pmove(i);
        self.player_pstep_d0(i).1
    }

    fn player_face_input_nib(&mut self, i: usize, nib: u8) {
        let d = nib & 3;
        if d != 0 {
            self.players[i].b.facing = d & 1;
        }
    }

    /// 0x1BDEC run_move: with no direction held keep running in $22.
    fn player_run_move(&mut self, i: usize) {
        let nib = self.players[i].held & 0xF;
        let off = if nib != 0 {
            self.player_face_input_nib(i, nib);
            let [off, dir] = self.player_pd().walk_dir[nib as usize];
            self.players[i].b.dir = dir;
            off
        } else {
            let d = self.players[i].b.dir;
            if (d as i8) < 0 {
                return;
            }
            d
        };
        let [vx, vz] = self.player_speed_entry(i, off);
        let p = &mut self.players[i];
        p.b.vx = vx;
        p.b.vz = vz;
        self.player_pmove(i);
        self.player_pstep(i);
    }

    /// 0x1BE3C air control (overwrites $22).
    fn player_air_control(&mut self, i: usize) {
        let nib = self.players[i].held & 0xF;
        let [off, dir] = self.player_pd().walk_dir[nib as usize];
        let p = &mut self.players[i];
        p.b.dir = dir;
        if (dir as i8) >= 0 {
            let [dx, dz] = self.assets.player.air_control[(off as usize / 4) & 7];
            p.b.vx = p.b.vx.wrapping_add(dx);
            p.b.vz = p.b.vz.wrapping_add(dz);
        }
    }

    /// 0x5364: jump vector by class (0 standing, 4 running, 8 wall, 12 heavy) and direction $22.
    fn player_set_jump_velocity(&mut self, i: usize, class: u16) {
        let p = &mut self.players[i];
        // add d0,d0; add d0,d0; add $20,d0 (0x5364): entry = class + character.
        let k = (class.wrapping_mul(4).wrapping_add(p.b.k20) / 4) as usize & 15;
        let d = p.b.dir;
        let e = if (d as i8) < 0 { 0 } else { (d as usize / 4) + 1 };
        let v = self.assets.player.jump[k][e.min(8)];
        p.b.vx = v.vx;
        p.b.ax = v.ax;
        p.b.vy = v.vy;
        p.b.ay = v.ay;
        p.b.vz = v.vz;
        p.b.az = v.az;
    }

    /// Height above the floor: `Y - $54 - $56`.
    fn player_height(&self, i: usize) -> i16 {
        let b = &self.players[i].b;
        b.yi().wrapping_sub(b.floor).wrapping_sub(b.floor2)
    }

    /// 0x606E: water splash when falling into water (effect TODO). Only sets the done flag.
    fn player_splash_check(&mut self, i: usize) {
        let p = &mut self.players[i];
        if p.b83 != 0 || p.b.draw_flags & 0x20 != 0 || (p.b.wading as i8) <= 0 || p.b.airborne == 0 {
            return;
        }
        if p.b.vy >= 0 || p.b.yi() >= 0x20 {
            return;
        }
        if self.fx_spawn_water_splash(Game::player_handle(i)) {
            self.players[i].b83 = 1;
            self.player_sfx(i, 0x9AF4);
        }
    }

    /// 0x11738: attack power factor from own HP and rand().
    pub(super) fn player_power_factor(&mut self, i: usize) {
        let hp = self.players[i].b.hp;
        let mut row = 0u16;
        if hp < 0x4B {
            row = 0x20;
            if hp < 0x32 {
                row = 0x40;
                if hp < 0x19 {
                    row = 0x60;
                }
            }
        }
        let r = self.rand() & 0x1F;
        self.players[i].power = self.player_pd().power[(row + r) as usize];
    }

    /// 0x63AA special check; true when it jumped to the special (caller must return).
    fn player_special_check(&mut self, i: usize) -> bool {
        let p = &self.players[i];
        if p.b.hp > 1
            && ((p.held & ATK != 0 && p.pressed & JMP != 0) || (p.held & JMP != 0 && p.pressed & ATK != 0))
        {
            self.player_special_entry(i);
            return true;
        }
        false
    }

    /// 0x63E6: same test, only buffers the request for 10 frames.
    fn player_special_buffer(&mut self, i: usize) {
        let p = &mut self.players[i];
        if p.b.hp > 1
            && ((p.held & ATK != 0 && p.pressed & JMP != 0) || (p.held & JMP != 0 && p.pressed & ATK != 0))
        {
            p.special_buf = 10;
        }
    }

    /// 0x6012 mash: stick taps and attack presses lower $BE.
    fn player_mash(&mut self, i: usize) {
        let w = self.player_pd().mash_weight;
        let p = &mut self.players[i];
        let d = p.pressed & 0xF;
        if d != 0 {
            let (v, borrow) = p.mash.overflowing_sub(w[d as usize]);
            if borrow {
                p.mash = 0;
                return;
            }
            // 0x6024: `bcc 0x6032`: without a borrow the code goes on with the attack press and
            // the cooldown (a stick tap together with A counts twice).
            p.mash = v;
        }
        if p.mash_cd != 0 {
            p.mash_cd -= 1;
            return;
        }
        if p.pressed & ATK != 0 {
            p.mash_cd = 6;
            p.mash = p.mash.saturating_sub(1);
        }
    }

    /// 0x6C2A: invulnerability timer `$F9 = INV[d0 + character]`.
    fn player_set_invuln(&mut self, i: usize, d0: usize) {
        let ch = self.players[i].b.id as usize & 3;
        self.players[i].invuln = self.player_pd().invuln[(d0 + ch) & 15];
    }

    // ---------------------------------------------------------------------------------------
    // Join, frame loop
    // ---------------------------------------------------------------------------------------

    /// Player `i` joins with character `chr` (0 Jack, 1 Mustapha, 2 Hannah, 3 Mess).
    /// 0x4D46 / 0x4D5E / 0x4D76 (+ 0x4D8E, 0x4DD6), what the select screen sets ($03, $20,
    /// $00 = 1), then the stage start placement 0x4E0A. The spawn itself runs in mode 0.
    pub fn player_join(&mut self, i: usize, chr: u8) {
        let mut p = Player::default(); // 0x4D8E clears 0x180 bytes
        p.b.hp_prev = 100;
        p.b.hp = 100;
        p.b.hp_bar = 100;
        p.b.facing = 1;
        p.b10a = 0x33;
        p.b.who = i as u8;
        p.b.serial = (i as u8) << 4; // +$5E = index * 0x10
        p.b.id = chr & 3;
        p.b.k20 = (chr as u16 & 3) * 4;
        p.b.live = 1;
        self.players[i] = p;
        self.player_power_factor(i); // 0x11738 (jsr at 0x4DBA)
        // 0x4DD6: lives from the settings (G.$777C & 3).
        self.flow_lives_init(i);
        self.player_start_position(i);
    }

    /// 0x4E0A: place a joined player at the stage / area start position, state 0.
    pub fn player_start_position(&mut self, i: usize) {
        if self.players[i].b.live == 0 {
            return;
        }
        let stage = self.w.stage as usize;
        let area = self.cam.s2.area as usize;
        let pos = self.player_pd().start_pos.get(stage).map(|t| t[area.min(2)][i]).unwrap_or([0, 0, 0]);
        let p = &mut self.players[i];
        p.b.set_state(0);
        p.w10c = 0;
        Player::set_hi(&mut p.b.x, pos[0]);
        Player::set_hi(&mut p.b.y, pos[1]);
        Player::set_hi(&mut p.b.z, pos[2]);
    }

    /// 0x18834: all players (P1, P2, P3).
    pub fn players_update(&mut self) {
        // The inputs are copied with interrupts masked (trap #12) into $77A4..$77A6.
        let snap = self.w.input;
        // 0x9A146 demo input record / playback (when G.$4CC == 0): not ported.
        self.snd_update_voice_banks(); // 0xA502
        for i in 0..3 {
            let p = &self.players[i];
            if p.b.live != 0 && p.b.kind != 0xE {
                self.player_frame(i, snap[i]);
            }
        }
    }

    /// 0x188DA: players currently held by an enemy run after the enemies.
    pub fn held_players_update(&mut self) {
        let snap = self.w.input;
        for i in 0..3 {
            let p = &self.players[i];
            if p.b.live != 0 && p.b.kind == 0xE {
                // P1 only: 1/32 chance of a ROM checksum test (0x104F6); skipped, but rand()
                // is still drawn in the original order.
                if i == 0 {
                    let _ = self.rand();
                }
                self.player_frame(i, snap[i]);
            }
        }
    }

    /// 0x18952: one player for one frame.
    fn player_frame(&mut self, i: usize, cur: u8) {
        {
            let p = &mut self.players[i];
            p.prev = p.held;
            p.held = cur;
            p.pressed = cur & (cur ^ p.prev);
        }
        self.script_player_hook(i); // 0x189CE: scripted control (0x1DB0C once, 0x1DB5E every frame) while G.$4DD is set
        if self.players[i].input_lock != 0 {
            let p = &mut self.players[i];
            p.held = 0;
            p.prev = 0;
            p.pressed = 0;
        }
        // 0x18A02: the special modes. In the car section (G.$518) the respawn timer `+$EC` runs here (the hero is in the car and invulnerable
        // until it ends) and the hero is neither in the collision list nor drawn (below). While the Episode 8 clear sequence (G.$516, script action
        // 0x42) shows the stand ins, the hero state machine is skipped: with the car off at once, in the car only a hero out of mode 0.
        let car = self.spawn.g.car_mode != 0;
        if car {
            let p = &mut self.players[i];
            if p.respawn_t != 0 {
                p.respawn_t -= 1;
                if p.respawn_t == 0 {
                    p.b.live = 1; // 0x18A1A
                }
            }
            if self.players[i].b.mode != 0 && self.flow.script.b516 != 0 {
                return;
            }
        } else if self.flow.script.b516 != 0 {
            return; // 0x18A28
        }
        match self.players[i].b.mode {
            0 => self.player_spawn(i),
            2 => self.player_alive(i),
            4 => self.player_dead(i),
            6 => self.player_remove(i),
            _ => {}
        }
        if self.spawn.g.car_mode != 0 {
            return; // 0x18A3E: the car hides the heroes
        }
        let h = Game::player_handle(i);
        // 0x4BC0: collision list of players (max 3).
        if self.combat.get(0x6718).len() < 3 {
            self.combat.push(0x6718, h);
        }
        let p = &self.players[i];
        if (p.action == 0x12 || p.b.kind != 0xE) && p.b.draw_flags & 8 == 0 {
            self.draw_list_add_check(h); // 0x49CA
        }
    }

    /// 0x18EB0 mode 6: free the slot.
    fn player_remove(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.b.live = 0;
        p.b.visible = 0;
        p.b.set_state(0);
        // G.$4CD / G.$4E4 bits of this player are cleared (flow port).
    }

    // ---------------------------------------------------------------------------------------
    // Mode 0: (re)spawn 0x18A82
    // ---------------------------------------------------------------------------------------

    fn player_spawn(&mut self, i: usize) {
        match self.players[i].b.sub {
            0 => self.player_spawn_stage_start(i),
            2 => self.player_respawn(i),
            _ => {}
        }
    }

    /// 0x18C82 / 0x18C88: physics reset.
    fn player_physics_reset(&mut self, i: usize) {
        let boxes = self.assets.sprites.player_box_tables[self.players[i].b.id as usize & 3];
        {
            let p = &mut self.players[i];
            p.b.box_table = boxes; // $44 = long[0x18CC4 + $20]
            p.b.multi_box = false;
        }
        self.player_set_speed_class(i, 0);
        let p = &mut self.players[i];
        p.b.vx = 0;
        p.b.vy = 0;
        p.b.vz = 0;
        p.b.ax = 0;
        p.b.ay = 0;
        p.b.az = 0;
        p.b116 = 0;
        p.b11b = 0;
        p.b.stun = 10;
        p.slow = 0;
        self.player_idle_anim(i);
    }

    /// 0x18A94: first frame after the stage start placement.
    fn player_spawn_stage_start(&mut self, i: usize) {
        {
            let p = &mut self.players[i];
            p.b.palette_override = 0;
            p.b.live = 1;
        }
        self.player_physics_reset(i); // 0x18C88
        self.players[i].b.mode = 2;
        self.player_count_players(); // 0x655E
        let stage = self.w.stage;
        if self.cam.s2.area == 0 && (stage == 1 || stage == 3) {
            // Vehicle entrance (state 0x432): wait for G.$505, then jump out (0x1BBA6).
            let who = self.players[i].b.who;
            let behind = match who {
                0 => false,
                1 => self.players[0].b.live != 0,
                _ => self.players[0].b.live != 0 || self.players[1].b.live != 0,
            };
            if behind {
                self.player_panim(i, 0, 6);
                self.players[i].b.facing = 1;
            } else {
                // G.$504 = 0 (flow port).
                self.player_panim(i, 0, 0x1D);
                self.players[i].b.facing = 0;
            }
            let p = &mut self.players[i];
            p.b.airborne = 1;
            p.b11b = 0;
            p.b.draw_flags = 0x10;
            p.b.set_state(0x0200_0432);
        }
    }

    /// 0x655E: G.$4EF = number of active players - 1.
    fn player_count_players(&mut self) {
        let n = self.players.iter().filter(|p| p.b.live != 0 && p.b.mode != 0).count();
        self.w.players_minus_one = n.saturating_sub(1) as u8;
    }

    /// 0x18B38 respawn (mode 0, sub 2): [0] place above the camera, [2] reset, [4] wait, drop in.
    fn player_respawn(&mut self, i: usize) {
        match self.players[i].b.act {
            0 => {
                // 0x18B4C
                let cx = self.cam.s2.cam_x;
                let cy = self.cam.s2.cam_y;
                self.players[i].b.palette_override = 0;
                self.players[i].b.airborne = 1;
                self.player_drop_place(i); // 0x62D6
                {
                    let p = &mut self.players[i];
                    p.b.live = 2;
                    Player::set_hi(&mut p.b.x, cx.wrapping_add(0xC0) as i16);
                    Player::set_hi(&mut p.b.z, if self.w.vertical_scene != 0 { 0x30 } else { cy as i16 });
                }
                self.player_floor_only(i); // 0x12D4E
                let p = &mut self.players[i];
                let y = p.b.floor.wrapping_add(p.b.floor2).wrapping_add(0x180);
                Player::set_hi(&mut p.b.y, y);
                p.b.act = 2;
                self.player_respawn_reset(i);
            }
            2 => self.player_respawn_reset(i),
            4 => self.player_respawn_wait(i),
            _ => {}
        }
    }

    /// 0x18B9A
    fn player_respawn_reset(&mut self, i: usize) {
        self.player_physics_reset(i); // 0x18C82 (also 0x162AE, palette: not ported)
        {
            let p = &mut self.players[i];
            p.respawn_t = 0x168;
            p.b.act = 4;
            p.b.facing = 1;
            p.set_b80(0); // 0x18BAE clr.b $80
        }
        if self.spawn.g.car_mode != 0 {
            return; // 0x18BB2: Cadillac section, no wait, no bazooka, no strike
        }
        self.players[i].set_b80(0x3C);
        // 0x18BC0: a player that joins in the middle of a stage (+$07 == 0) gets a bazooka (0x65A4).
        if self.players[i].b.step == 0 {
            self.item_give_bazooka(i);
        }
        self.player_spawn_air_strike(i); // 0x18BCE bra 0x18C5E
    }

    /// 0x18C5E: pool B class 18, the respawn air strike. A full pool skips it (carry of 0x4860).
    fn player_spawn_air_strike(&mut self, i: usize) {
        let who = self.players[i].b.who;
        let id = self.players[i].b.id;
        let Some(h) = self.spawn_prop(18, 0, None) else {
            return;
        };
        let b = self.body_mut(h);
        b.who = who;
        b.id = id;
    }

    /// 0x62D6: place a player that drops in near the camera. The X offset depends on the number
    /// of players and the player's index (table 0x638C); the floor lookup 0x12FE4 and the terrain
    /// test 0x12CAC run at that spot. In the vertical scene (G.$4DC) fixed spots are used.
    fn player_drop_place(&mut self, i: usize) {
        let t = &self.assets.terrain;
        let idx = self.players[i].b.who as usize & 3;
        let vertical = self.w.vertical_scene != 0;
        let (cx, cy) = (self.cam.s2.cam_x as i16, self.cam.s2.cam_y as i16);
        if vertical {
            // 0x6370
            let p = &mut self.players[i];
            Player::set_hi(&mut p.b.x, t.drop_x[idx]);
            Player::set_hi(&mut p.b.z, t.drop_z[idx]);
            Player::set_hi(&mut p.b.y, 0x100);
            return;
        }
        // 0x62DE: d0 = (players - 1) * 8, d1 = 0 / 2 / 4 by who is present.
        let n = self.w.players_minus_one as usize;
        let mut d1 = 0usize;
        if n != 0 && idx != 0 {
            if self.players[0].b.live != 0 {
                d1 = 2;
                if idx != 1 && self.players[1].b.live != 0 {
                    d1 = 4;
                }
            } else {
                // 0x631A: `cmpi.w #1,$2(a6)` compares the word (index, character), never 1 here.
                d1 = 4;
            }
        }
        let x = cx.wrapping_add(t.drop_x[n * 4 + d1 / 2]);
        let me = Game::player_handle(i);
        let mut b = std::mem::take(&mut self.players[i].b);
        Player::set_hi(&mut b.x, x);
        self.terrain_floor_lookup(&mut b); // 0x12FE4
        let (y, z) = (b.floor.wrapping_add(0x100), cy.wrapping_sub(b.floor));
        Player::set_hi(&mut b.y, y);
        Player::set_hi(&mut b.z, z);
        if let Some(o) = self.terrain_player(&mut b, me) {
            self.players[i].w94 = o;
        }
        let z = b.zi().wrapping_add(t.drop_z[idx]);
        Player::set_hi(&mut b.z, z);
        self.players[i].b = b;
    }

    /// 0x18BD4
    fn player_respawn_wait(&mut self, i: usize) {
        let p = &mut self.players[i];
        if p.b80() != 0 {
            p.set_b80(p.b80() - 1);
            return;
        }
        p.b.airborne = 1;
        self.player_drop_place(i); // 0x62D6
        self.players[i].b.set_state(0x0200_0000);
        // In the car section (G.$518) the hero does not fall in and gets no marker bubble (0x18BF6, 0x18C0A): he is aboard at once.
        let car = self.spawn.g.car_mode != 0;
        if !car {
            self.player_drop_entry(i); // 0x1B67C: fall in, state 0x42E
        }
        self.player_count_players(); // 0x655E
        if !car {
            self.fx_spawn_player_marker(i, 0); // 0x18C18
        }
    }

    // ---------------------------------------------------------------------------------------
    // Mode 2: alive 0x18CD4
    // ---------------------------------------------------------------------------------------

    fn player_alive(&mut self, i: usize) {
        if self.players[i].respawn_t != 0 && self.players[i].respawn_t & 3 == 0 {
            self.fx_spawn_respawn_sparkle(i); // 0x18CD4 -> 0x958E4
        }
        {
            let p = &mut self.players[i];
            if p.respawn_t != 0 {
                p.respawn_t -= 1;
                if p.respawn_t == 0 && p.invuln == 0 {
                    p.b.live = 1;
                }
            }
            if p.invuln != 0 {
                p.invuln -= 1;
                if p.invuln == 0 && p.respawn_t == 0 {
                    p.b.live = 1;
                    p.b.palette_override = 0;
                }
            }
            let mut skip_ae = false;
            if p.held & ATK != 0 {
                if (p.atk_hold as i8) < 0 {
                    skip_ae = true;
                } else {
                    p.atk_hold = p.atk_hold.wrapping_add(1);
                }
            } else {
                p.atk_hold = 0;
            }
            if !skip_ae && p.combo_t != 0 {
                p.combo_t -= 1;
            }
            if p.b.facing != p.last_facing {
                p.last_facing = p.b.facing;
                p.facing_age = 0;
            }
            p.facing_age = p.facing_age.wrapping_add(1);
            if (p.facing_age as i8) < 0 {
                p.facing_age = 0x70;
            }
            if p.dtap != 0 {
                p.dtap -= 1;
            }
            if p.lock != 0 {
                p.lock -= 1;
            }
            if (p.b.hp as i16) <= 0 && p.b.mode < 4 && !(p.b.mode == 2 && p.b.sub == 2) {
                p.b.hitstop = 0x10;
                p.b.set_state(0x0202_0000);
                p.b.attacker = Game::player_handle(i);
                p.item_type = 0;
                p.b.palette_override = 0;
                p.b11b = 1;
                return;
            }
            if p.stun_refill != 0 {
                p.stun_refill -= 1;
                if p.stun_refill == 0 {
                    p.b.stun = 10;
                }
            }
            if p.hits_t != 0 {
                p.hits_t -= 1;
                if p.hits_t == 0 {
                    p.hits = 0;
                }
            }
            if p.dizzy_cd != 0 {
                p.dizzy_cd -= 1;
            }
            if p.special_buf != 0 {
                p.special_buf -= 1;
            }
            if p.regrab_cd != 0 {
                p.regrab_cd -= 1;
            }
            if p.pickup_cd != 0 {
                p.pickup_cd -= 1;
            }
            if p.run_t != 0 {
                p.run_t -= 1;
            }
            if !(p.b.mode == 2 && p.b.sub == 0 && p.b.act == 2) && p.run_cd != 0 {
                p.run_cd -= 1;
            }
            p.cmd_flag = 0;
        }
        self.player_command_fsm(i); // 0x6AC8
        {
            let p = &mut self.players[i];
            if p.b11c != 0 {
                p.b11c -= 1;
                if p.b11c == 0 {
                    p.b10e = 0;
                }
            }
        }
        match self.players[i].b.sub {
            0 => self.player_normal(i),
            2 => self.player_hurt(i),
            4 => self.player_grabbing(i),
            6 => self.player_grabbed(i),
            _ => {}
        }
        if self.w.floor_only == 0 {
            self.player_floor_and_walls(i); // 0x12CAC
        } else {
            self.player_floor_only(i); // 0x12D4E (G.$4F0)
        }
        if self.w.no_clamp == 0 {
            self.player_clamp_to_screen(i); // 0x4FCE (skipped when G.$506)
        }
    }

    /// 0x6AC8 command FSM: down, up, attack while holding up.
    fn player_command_fsm(&mut self, i: usize) {
        let ch = self.players[i].b.id as usize & 3;
        let [t1, t2] = self.player_pd().command_time;
        let p = &mut self.players[i];
        match p.cmd_state {
            0 => {
                // 0x6ADC: the input changed and the PREVIOUS input was exactly down.
                if p.prev != p.held && p.prev & 0xF == DOWN {
                    p.cmd_t = t1[ch];
                    p.cmd_state = 2;
                }
            }
            2 => {
                p.cmd_t = p.cmd_t.wrapping_sub(1);
                if p.cmd_t == 0xFF {
                    p.cmd_state = 0;
                } else if p.held & 0xF == UP {
                    p.cmd_t = t2[ch];
                    p.cmd_state = 4;
                    // falls into state 4 (0x6B44) in the same frame
                    p.cmd_t = p.cmd_t.wrapping_sub(1);
                    if p.cmd_t == 0xFF {
                        p.cmd_state = 0;
                    } else if p.pressed & ATK != 0 {
                        if p.held & 0xF == UP {
                            p.cmd_flag = 2;
                        }
                        p.cmd_state = 0;
                    }
                } else if p.held & ATK != 0 {
                    p.cmd_state = 0;
                }
            }
            4 => {
                p.cmd_t = p.cmd_t.wrapping_sub(1);
                if p.cmd_t == 0xFF {
                    p.cmd_state = 0;
                } else if p.pressed & ATK != 0 {
                    if p.held & 0xF == UP {
                        p.cmd_flag = 2;
                    }
                    p.cmd_state = 0;
                }
            }
            _ => {}
        }
    }

    /// 0x12CAC: terrain collision of a player: floor, walls, obstacles, floor snap (0x12CB4) and
    /// the ceiling test (0x1361E). See `terrain.rs`.
    fn player_floor_and_walls(&mut self, i: usize) {
        let me = Game::player_handle(i);
        let mut b = std::mem::take(&mut self.players[i].b);
        if let Some(o) = self.terrain_player(&mut b, me) {
            self.players[i].w94 = o; // move.w a1,$94(a6)
        }
        self.players[i].b = b;
    }

    /// 0x12D4E: floor only (no walls, no obstacles).
    fn player_floor_only(&mut self, i: usize) {
        let mut b = std::mem::take(&mut self.players[i].b);
        self.terrain_floor_only(&mut b);
        self.players[i].b = b;
    }

    /// 0x4FCE: keep the player on screen (integer words only).
    fn player_clamp_to_screen(&mut self, i: usize) {
        let (cx, cy) = if self.w.vertical_scene != 0 { (0i16, 0i16) } else { (self.cam.s2.cam_x as i16, self.cam.s2.cam_y as i16) };
        let p = &mut self.players[i];
        let rel = p.b.xi().wrapping_sub(cx).wrapping_sub(0x10);
        let d = if rel < 0 { rel } else { rel.wrapping_sub(0x160) };
        if rel < 0 || d >= 0 {
            let x = p.b.xi().wrapping_sub(d);
            Player::set_hi(&mut p.b.x, x);
        }
        let zf = if self.w.vertical_scene != 0 { 0 } else { p.b.floor };
        let rel = p.b.zi().wrapping_add(zf).wrapping_sub(cy).wrapping_sub(0x10);
        let d = if rel < 0 { rel } else { rel.wrapping_sub(0x90) };
        if rel < 0 || d >= 0 {
            let z = p.b.zi().wrapping_sub(d);
            Player::set_hi(&mut p.b.z, z);
        }
    }

    // ---------------------------------------------------------------------------------------
    // Normal (mode 2, sub 0): dispatch 0x19090
    // ---------------------------------------------------------------------------------------

    fn player_normal(&mut self, i: usize) {
        // 0x19090: a scripted scene (G.$4DD) drops a carried object: any object while a stage clear runs (G.$4D1), a heavy object
        // (a barrel, 0x20A) unless the area end table says it may stay (G.$514 = 1).
        if self.w.scripted != 0
            && (self.flow.sub != 0 || (self.players[i].item_type == 0x20A && self.flow.script.b514 == 0))
        {
            self.players[i].item_type = 0;
        }
        match self.players[i].b.act {
            0 => self.player_ground(i),
            2 => self.player_run_group(i),
            4 => self.player_air(i),
            _ => {}
        }
    }

    /// 0x190CA ground states (table 0x190F4).
    fn player_ground(&mut self, i: usize) {
        let step = self.players[i].b.step;
        if step != 0x22 && step != 0x0E && self.players[i].b.airborne != 0 {
            return self.player_fall_entry(i);
        }
        match step {
            0x00 => self.player_idle(i),
            0x02 => self.player_walk(i),
            0x04 => self.player_jab(i),
            0x06 => self.player_hit_confirm(i),
            0x08 => self.player_punch2(i),
            0x0A => self.player_punch3(i),
            0x0C => self.player_finisher(i),
            0x0E => self.player_special(i),
            0x10 => self.player_dash_attack(i),
            0x12 => self.player_recovery(i),
            0x24 => self.player_cmd_dash(i),
            0x26 | 0x28 => {
                if self.player_pstep(i) {
                    self.player_idle_entry(i);
                }
            }
            0x14 => self.player_pickup(i),
            0x16 => self.player_weapon_attack(i),
            0x1C => self.player_gun_stand(i),
            0x1E => self.player_gun_walk(i),
            0x20 => self.player_charge(i),
            0x22 => self.player_recoil(i),
            0x18 => self.player_pose(i), // 0x19A0A the pose (scripted scenes, extra life)
            0x2A => self.player_hops(i), // 0x19EB2 two small hops (script action 0x3A)
            0x2C => self.player_chest_pose(i), // 0x19F6A the pose with an effect (script action 0x3E)
            // 0x1A (the second extra life state, 0x19A7C) is M2-17.
            _ => self.player_idle_entry(i),
        }
    }

    /// 0x19122 idle_entry (falls through into idle).
    fn player_idle_entry(&mut self, i: usize) {
        {
            let p = &mut self.players[i];
            p.b.set_state(0x0200_0000);
            p.b.vx = 0;
            p.b.vy = 0;
            p.b.vz = 0;
            p.b.ax = 0;
            p.b.ay = 0;
            p.b.az = 0;
            p.b.draw_flags = 0;
            p.weapon_atk = 0;
            p.bc1 = 0;
            p.action = 0;
            p.b.b72 = 0x3C;
            p.b.palette_override = 0;
        }
        self.player_set_speed_class(i, 0);
        self.player_idle_anim(i);
        self.player_idle(i);
    }

    /// 0x19186 -> 0x2A4FC: the life pose's one-frame attack box (variant 1).
    fn player_oneup_attack_box(&mut self, i: usize) {
        let Some(j) = self.items.alloc() else { return };
        let p = &self.players[i].b;
        let b = &mut self.items.slots[j].b;
        b.live = 1;
        b.k20 = 11 * 4;
        b.who = p.who;
        b.id = p.id;
        b.x = (b.x & 0xFFFF) | ((p.xi() as i32) << 16);
        b.y = (b.y & 0xFFFF) | ((p.yi() as i32) << 16);
        b.z = (b.z & 0xFFFF) | ((p.zi() as i32) << 16);
        b.var26 = 1;
        b.partner = Game::player_handle(i);
    }

    /// 0x9814: pose voice, indexed by character and the current voice bank.
    pub(super) fn player_oneup_voice(&mut self, i: usize) {
        let p = &self.players[i];
        let code = self.assets.text.u8(0x985C, p.b.k20 as u32 + p.voice as u32);
        let pan = self.snd_pan_x(p.b.xi());
        self.snd_request(code as u16, 0x0007_0000 | pan as u32, 0x0040_0000);
    }

    /// 0x1916E idle.
    fn player_idle(&mut self, i: usize) {
        if self.w.scripted == 0 && self.players[i].b10e != 0 {
            // 0x1916E -> 0x19996: life pose takes priority over idle input.
            self.players[i].b.live = 2;
            self.player_oneup_attack_box(i); // 0x19186 -> 0x2A4FC
            return self.player_pose_entry(i); // bra 0x19996
        }
        if self.players[i].special_buf != 0 {
            return self.player_special_entry(i);
        }
        if self.player_special_check(i) {
            return;
        }
        let p = &self.players[i];
        let new = p.pressed;
        if p.atk_buf != 0 {
            return self.player_chain_continue(i);
        }
        if new & ATK != 0 {
            if p.combo != 0 && p.combo_t != 0 {
                return self.player_chain_continue(i);
            }
            // 0x191C4: G.$506 (set by scripted scenes): no pick-up and no weapon, a jab.
            if self.w.no_clamp != 0 {
                return self.player_jab_entry(i, 0);
            }
            if p.item_type != 0x20A {
                self.player_find_pickup(i);
                if (self.players[i].cand_type as i16) >= 0 {
                    return self.player_pickup_entry(i);
                }
            }
            if self.players[i].item_type != 0 {
                return self.player_weapon_attack_entry(i);
            }
            return self.player_jab_entry(i, 0);
        }
        if new & JMP != 0 && !(p.item_type == 0x20A && p.b.id == 2) {
            return self.player_jump_entry(i);
        }
        if p.lock != 0 || p.combo_t >= 0x24 {
            self.player_face_input(i);
        } else {
            let d = p.held & 0xF;
            if d != 0 {
                if p.run_cd != 0 {
                    return self.player_walk_entry(i);
                }
                if p.item_type != 0x20A {
                    let t = self.player_pd().dir8;
                    let diff = t[d as usize].wrapping_sub(t[p.walk_in as usize & 15]).wrapping_add(1) & 0x1F;
                    if diff < 3 && p.dtap >= 3 {
                        return self.player_run_entry(i);
                    }
                }
                self.players[i].dtap = 0x0F;
                return self.player_walk_entry(i);
            }
        }
        let p = &self.players[i];
        if p.item_type == 0 && self.w.scripted == 0 {
            self.player_pstep(i);
        }
    }

    /// 0x192B8 walk_entry.
    fn player_walk_entry(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.walk_in = p.held & 0xF;
        self.player_face_input(i);
        self.player_walk_reenter(i);
    }

    /// 0x192C6
    fn player_walk_reenter(&mut self, i: usize) {
        {
            let p = &mut self.players[i];
            p.b.set_state(0x0200_0002);
            p.set_b80(0);
            p.b.draw_flags = 0;
        }
        self.player_walk_anim(i);
        self.player_walk(i);
    }

    /// 0x192DE walk.
    fn player_walk(&mut self, i: usize) {
        {
            let p = &mut self.players[i];
            if p.b.wading != 0 && (p.b.yi() as u16) < 0x20 {
                // stage 1 area 1 hook 0x60E2 (water ripples): effects TODO.
                let t = p.b80().wrapping_sub(1);
                p.set_b80(t);
                if t == 0xFF {
                    p.set_b80(0x18);
                    self.player_sfx(i, 0x9B60);
                }
            }
        }
        if self.player_special_check(i) {
            return;
        }
        let p = &self.players[i];
        if p.item_type != 0x20A && p.atk_buf == 0 && p.cmd_flag == 2 {
            return self.player_command_move_entry(i);
        }
        let new = p.pressed;
        if new & ATK != 0 {
            if p.combo_t != 0 {
                return self.player_chain_continue(i);
            }
            if self.w.no_clamp != 0 {
                return self.player_jab_entry(i, 0); // 0x19356
            }
            if p.item_type != 0x20A {
                self.player_find_pickup(i);
                if (self.players[i].cand_type as i16) >= 0 {
                    return self.player_pickup_entry(i);
                }
            }
            if self.players[i].item_type != 0 {
                return self.player_weapon_attack_entry(i);
            }
            return self.player_jab_entry(i, 0);
        }
        if new & JMP != 0 && !(p.item_type == 0x20A && p.b.id == 2) {
            return self.player_jump_entry(i);
        }
        let d = p.held & 0xF;
        if d == 0 {
            return self.player_idle_entry(i);
        }
        // 0x193AE: with the split body the legs layer steps; its end restarts the walk (0x192C6).
        if self.players[i].b.draw_flags & 1 != 0 && self.player_layer_tick(i) {
            return self.player_walk_reenter(i);
        }
        self.player_walk_move(i, d);
    }

    // ---------------------------------------------------------------------------------------
    // Punch chain
    // ---------------------------------------------------------------------------------------

    /// 0x193CE (alt 0) / 0x193DE (alt 1): does NOT fall through.
    fn player_jab_entry(&mut self, i: usize, alt: u16) {
        self.players[i].b.draw_flags = 0;
        self.player_panim(i, 1, alt);
        self.players[i].b.set_state(0x0200_0004);
        self.player_new_attack(i, 0);
        let p = &mut self.players[i];
        p.combo = 0;
        p.dtap = 0;
        self.player_sfx(i, 0x95CE);
    }

    /// 0x19416 jab.
    fn player_jab(&mut self, i: usize) {
        if self.players[i].b.hitstop != 0 {
            self.player_panim(i, 1, 2);
            return self.player_hit_confirm_entry(i);
        }
        if self.player_pstep(i) {
            let p = &mut self.players[i];
            p.lock = 5;
            p.combo_t = 0;
            p.combo = 0;
            return self.player_idle_entry(i);
        }
        let p = &self.players[i];
        if p.b.param_a != 0 && p.pressed & ATK != 0 {
            self.player_face_input(i);
            self.player_jab_entry(i, 1);
        }
    }

    /// 0x19464 (falls through).
    fn player_hit_confirm_entry(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.b.set_state(0x0200_0006);
        p.atk_buf = 0;
        p.combo = p.combo.wrapping_add(1);
        p.b.partner = 0;
        p.action = 0x10;
        self.player_hit_confirm(i);
    }

    /// 0x1947E hit-stop and combo buffer.
    fn player_hit_confirm(&mut self, i: usize) {
        let hs = self.players[i].b.hitstop;
        if hs != 0 {
            let hs = hs - 1;
            self.players[i].b.hitstop = hs;
            if hs == 0 {
                self.player_pstep(i);
            }
            let p = &mut self.players[i];
            if p.pressed & ATK != 0 {
                p.atk_buf = p.pressed;
            }
            return;
        }
        if self.player_pstep(i) {
            let p = &mut self.players[i];
            p.combo_t = 0x28;
            if p.combo == 4 {
                p.atk_buf = 0;
                p.combo = 0;
                p.combo_t = 0;
            }
            return self.player_idle_entry(i);
        }
        self.player_chain_continue(i);
    }

    /// 0x194D4
    fn player_chain_continue(&mut self, i: usize) {
        let p = &self.players[i];
        if p.atk_buf == 0 && p.pressed & ATK == 0 {
            return;
        }
        let d3 = p.combo;
        if d3 >= 4 {
            self.players[i].atk_buf = 0;
            return;
        }
        self.player_face_input(i);
        match d3 {
            1 => self.player_punch2_entry(i),
            2 => self.player_punch3_entry(i),
            3 => self.player_finisher_entry(i),
            _ => self.players[i].atk_buf = 0,
        }
    }

    /// 0x1950E (falls through).
    fn player_punch2_entry(&mut self, i: usize) {
        self.player_panim(i, 1, 0);
        self.players[i].b.set_state(0x0200_0008);
        self.player_new_attack(i, 0);
        self.players[i].atk_buf = 0;
        self.player_sfx(i, 0x95CE);
        self.player_punch2(i);
    }

    /// 0x1953E
    fn player_punch2(&mut self, i: usize) {
        if self.players[i].b.hitstop != 0 {
            self.player_panim(i, 1, 2);
            return self.player_hit_confirm_entry(i);
        }
        self.player_punch_end(i);
    }

    /// 0x19552
    fn player_punch_end(&mut self, i: usize) {
        if self.player_pstep(i) {
            let p = &mut self.players[i];
            p.lock = 5;
            p.combo = 0;
            self.player_idle_entry(i);
        }
    }

    /// 0x1956C (falls through).
    fn player_punch3_entry(&mut self, i: usize) {
        self.player_panim(i, 1, 3);
        self.players[i].b.set_state(0x0200_000A);
        self.player_new_attack(i, 2);
        self.players[i].atk_buf = 0;
        self.player_sfx(i, 0x99E8);
        self.player_punch3(i);
    }

    /// 0x1959C
    fn player_punch3(&mut self, i: usize) {
        if self.players[i].b.hitstop == 0 {
            return self.player_punch_end(i);
        }
        let p = &mut self.players[i];
        p.b.hitstop = p.b.hitstop.wrapping_add(1);
        self.player_panim(i, 1, 5);
        self.player_hit_confirm_entry(i);
    }

    /// 0x195B2 finisher entry with the combo throw.
    fn player_finisher_entry(&mut self, i: usize) {
        let e = self.players[i].b.partner;
        if e != 0 && slot_of(e).is_some() {
            let eb = self.body(e);
            let ok = eb.live != 0 && eb.mode == 2 && eb.sub == 0;
            let d1 = self.players[i].held & 0xF;
            if ok && d1 & 0xC != 0 && d1 & 3 == 0 {
                // 0x195F4: up keeps the facing, down turns around.
                let up = d1 & 8 != 0;
                let f = self.players[i].b.facing;
                let face = if up { (f != 0) as u8 } else { (f == 0) as u8 };
                self.players[i].b.facing = face;
                let me = Game::player_handle(i);
                let eb = self.body_mut(e);
                eb.partner = me;
                eb.set_state(0x0206_0000);
                return self.player_throw_from_combo(i); // 0x1D888
            }
        }
        let p = &self.players[i];
        let anim = if p.b.id == 1 && p.b.param_a == 0 { 7 } else { 4 };
        self.player_panim(i, 1, anim);
        self.players[i].b.set_state(0x0200_000C);
        self.player_new_attack(i, 4);
        self.players[i].atk_buf = 0;
        self.player_sfx(i, 0x9A36);
        self.player_finisher(i);
    }

    /// 0x1965A
    fn player_finisher(&mut self, i: usize) {
        if self.players[i].b.hitstop == 0 {
            return self.player_punch_end(i);
        }
        self.player_panim(i, 1, 6);
        self.player_hit_confirm_entry(i);
    }

    // ---------------------------------------------------------------------------------------
    // Special, dash attack, command move
    // ---------------------------------------------------------------------------------------

    /// 0x1968E special_entry.
    fn player_special_entry(&mut self, i: usize) {
        {
            let p = &mut self.players[i];
            p.special_budget = 0x14;
            // G.$4D5 == 0: 0x16446 HP bar redraw (HUD port reads +$6C).
        }
        self.player_sfx(i, 0x992C);
        self.players[i].action = 0x11;
        if self.players[i].b.id == 1 {
            return self.player_air_special_entry(i); // 0x1AAE6 (Mustapha)
        }
        self.players[i].b.set_state(0x0200_000E);
        self.player_new_attack(i, 0x12);
        let p = &mut self.players[i];
        p.combo = 0;
        p.b.draw_flags = 0;
        p.item = 0;
        p.item_type = 0;
        p.drop = 0;
        self.player_panim(i, 1, 8);
        self.player_special(i);
    }

    /// 0x196F2
    fn player_special(&mut self, i: usize) {
        let p = &mut self.players[i];
        if p.b.hitstop != 0 {
            p.b.hitstop -= 1;
            return;
        }
        if self.player_pstep(i) {
            self.players[i].b.live = 2;
            self.player_set_invuln(i, 12);
            self.player_idle_entry(i);
        }
    }

    /// 0x19720 dash attack entry (run + attack).
    fn player_dash_attack_entry(&mut self, i: usize) {
        self.player_sfx(i, 0x9984);
        {
            let p = &mut self.players[i];
            p.action = 0x13;
            p.cmd_flag = 0;
            p.dtap = 0;
        }
        if self.players[i].b.id != 0 {
            return self.player_lunge_entry(i);
        }
        self.players[i].b.set_state(0x0200_0010);
        self.player_new_attack(i, 0x14);
        let p = &mut self.players[i];
        p.combo = 0;
        p.b.vy = 0;
        p.b.ay = 0;
        p.b.vz = 0;
        p.b.az = 0;
        p.b.draw_flags = 0;
        p.set_b80(p.b.param_a);
        p.set_b81(0);
        let right = p.b.facing != 0;
        p.b.vx = signed(0x500, !right);
        p.b.ax = signed(-0x20, !right);
        self.player_panim(i, 1, 0xC);
        self.player_dash_attack(i);
    }

    /// 0x197A2
    fn player_dash_attack(&mut self, i: usize) {
        let p = &mut self.players[i];
        if p.b.hitstop != 0 {
            p.b.hitstop -= 1;
            return;
        }
        self.player_pmove(i);
        if self.player_pstep(i) {
            self.players[i].b.set_state(0x0200_0012);
            self.player_panim(i, 0, 0xD);
            self.players[i].dtap = 0;
            return self.player_recovery(i);
        }
        self.player_multi_hit_tick(i, 0x5A, 0x5C);
    }

    /// New attack id whenever the frame byte +$2C changes: first `a`, then `b` (dash attacks).
    fn player_multi_hit_tick(&mut self, i: usize, a: u8, b: u8) {
        let p = &mut self.players[i];
        let old = p.b80();
        p.set_b80(p.b.param_a);
        if old == p.b.param_a {
            return;
        }
        let id = if p.b81() == 0 { a } else { b };
        self.player_new_attack(i, id);
        let p = &mut self.players[i];
        p.set_b81(p.b81().wrapping_add(1));
    }

    /// 0x19826
    fn player_recovery(&mut self, i: usize) {
        if self.player_pstep(i) {
            self.player_idle_entry(i);
        }
    }

    /// 0x19DDA command-move dash (state 0x24, Hannah's second landing).
    fn player_cmd_dash_entry(&mut self, i: usize) {
        self.players[i].b.set_state(0x0200_0024);
        self.player_panim(i, 1, 0xF);
        self.player_new_attack(i, 0x76);
        let v = self.player_pd().vectors_1bf06;
        let p = &mut self.players[i];
        p.b.vy = 0;
        p.b.ay = 0;
        let left = p.b.facing == 0;
        p.b.vx = signed(v[12], left);
        p.b.ax = signed(v[13], left);
        self.player_cmd_dash(i);
    }

    /// 0x19E24
    fn player_cmd_dash(&mut self, i: usize) {
        let p = &mut self.players[i];
        if p.b.hitstop != 0 {
            p.b.hitstop -= 1;
            return;
        }
        self.player_pstep(i);
        if self.players[i].b.param_a == 0 {
            self.player_pmove(i);
        }
        let p = &self.players[i];
        let hi = (p.b.vx >> 8) as i8; // tst.b $14: sign of the high byte
        if (p.b.facing == 0 && hi >= 0) || (p.b.facing != 0 && hi < 0) {
            self.player_idle_entry(i);
        }
    }

    /// 0x1B6EC command move entry (down, up + attack), then the per-character code (0x1B72E).
    fn player_command_move_entry(&mut self, i: usize) {
        {
            let p = &mut self.players[i];
            p.b.set_state(0x0200_0430);
            p.b90 = 0;
            p.atk_buf = 0;
            p.combo = 0;
            p.action = 0x15;
            p.b.draw_flags = 0;
        }
        self.player_panim(i, 1, 0xE);
        self.player_face_input(i);
        self.player_sfx(i, 0x9A36);
        self.player_command_move(i);
    }

    /// 0x1B71E per-character command moves.
    fn player_command_move(&mut self, i: usize) {
        let v = self.player_pd().vectors_1bf06;
        let ch = self.players[i].b.id & 3;
        let step = self.players[i].b90;
        let load = |p: &mut Player, k: usize| {
            let left = p.b.facing == 0;
            p.b.vx = signed(v[k], left);
            p.b.ax = signed(v[k + 1], left);
            p.b.vy = v[k + 2];
            p.b.ay = v[k + 3];
        };
        match (ch, step) {
            // Jack 0x1B736
            (0, 0) => {
                if !self.player_pstep(i) {
                    return;
                }
                self.player_new_attack(i, 0x72);
                let p = &mut self.players[i];
                p.set_b81(0);
                p.b.vz = 0;
                p.b.az = 0;
                load(p, 0);
                p.b.airborne = 1;
                self.player_panim(i, 1, 0xF);
                let p = &mut self.players[i];
                p.set_b80(p.b.param_a);
                p.b90 = 2;
            }
            (0, 2) => {
                if self.player_hitstop_wait(i) {
                    return;
                }
                self.player_pstep(i);
                self.player_multi_hit_tick(i, 0x74, 0x76);
                if self.players[i].b.vy < 0 {
                    self.player_splash_check(i);
                    if self.players[i].b.airborne == 0 {
                        let p = &mut self.players[i];
                        p.b.vy = 0;
                        p.b.ay = 0;
                        self.player_panim(i, 0, 0x17);
                        self.players[i].b90 = 4;
                        return;
                    }
                }
                self.player_pmove(i);
            }
            (0, _) => {
                if self.player_pstep(i) {
                    self.player_idle_entry(i);
                }
            }
            // Mustapha 0x1B85C
            (1, 0) => {
                self.player_new_attack(i, 0x74);
                self.players[i].b90 = 2;
                self.player_command_move(i);
            }
            (1, 2) => {
                if self.player_hitstop_wait(i) {
                    return;
                }
                self.player_pstep(i);
                if self.players[i].b.param_a != 0 {
                    self.player_new_attack(i, 0x76);
                    let p = &mut self.players[i];
                    p.set_b80(p.b.param_a);
                    p.set_b81(0);
                    p.b.vz = 0;
                    p.b.az = 0;
                    load(p, 4);
                    p.b.airborne = 1;
                    let y = p.b.yi().wrapping_add(0x30);
                    Player::set_hi(&mut p.b.y, y);
                    p.b90 = 4;
                }
            }
            (1, _) => {
                if self.player_hitstop_wait(i) {
                    return;
                }
                self.player_pstep(i);
                self.player_multi_hit_tick(i, 0x78, 0x7A);
                if self.players[i].b.vy < 0 {
                    self.player_splash_check(i);
                    if self.players[i].b.airborne == 0 {
                        // 0x19E8A state 0x28
                        self.players[i].b.set_state(0x0200_0028);
                        self.player_panim(i, 1, 0xF);
                        if self.player_pstep(i) {
                            self.player_idle_entry(i);
                        }
                        return;
                    }
                }
                self.player_pmove(i);
            }
            // Hannah 0x1B988
            (2, 0) => {
                self.player_pstep(i);
                if self.players[i].b.param_a != 0 {
                    return;
                }
                self.player_new_attack(i, 0x70);
                let p = &mut self.players[i];
                p.w80 = 0;
                p.b.vz = 0;
                p.b.az = 0;
                load(p, 8);
                p.b.airborne = 1;
                p.set_b80(p.b.param_a);
                p.b90 = 2;
                self.player_command_move(i);
            }
            (2, _) => {
                if self.player_hitstop_wait(i) {
                    return;
                }
                self.player_pstep(i);
                let p = &mut self.players[i];
                if p.b.vy < 0 && p.b81() == 0 {
                    p.set_b81(1);
                    if p.b80() == 0 {
                        self.player_new_attack(i, 0x72);
                    }
                }
                if self.players[i].b.vy < 0 {
                    self.player_splash_check(i);
                    if self.players[i].b.airborne == 0 {
                        if self.players[i].b80() != 0 {
                            let p = &mut self.players[i];
                            p.b.vy = 0;
                            p.b.ay = 0;
                            return self.player_cmd_dash_entry(i);
                        }
                        let p = &mut self.players[i];
                        p.b.airborne = 1;
                        p.b.vy = p.b.vy.wrapping_neg() >> 1;
                        p.set_b80(1);
                        self.player_new_attack(i, 0x74);
                    }
                }
                self.player_pmove(i);
            }
            // Mess 0x1BAA6
            (3, 0) => {
                self.player_new_attack(i, 0x72);
                let p = &mut self.players[i];
                p.set_b81(0);
                p.b.vz = 0;
                p.b.az = 0;
                load(p, 14);
                p.b.airborne = 1;
                p.set_b80(p.b.param_a);
                p.b90 = 2;
                self.player_command_move(i);
            }
            (3, _) => {
                if self.player_hitstop_wait(i) {
                    return;
                }
                self.player_pstep(i);
                let p = &mut self.players[i];
                let old = p.b80();
                p.set_b80(p.b.param_a);
                if old != p.b.param_a {
                    let id = match p.b81() {
                        0 => 0x74,
                        1 => 0x76,
                        _ => 0x78,
                    };
                    self.player_new_attack(i, id);
                    let p = &mut self.players[i];
                    p.set_b81(p.b81().wrapping_add(1));
                }
                if self.players[i].b.vy < 0 {
                    self.player_splash_check(i);
                    if self.players[i].b.airborne == 0 {
                        // 0x19E70: hard landing requests strength 3 (G.$755C).
                        self.cam_shake(3);
                        self.players[i].b.set_state(0x0200_0026);
                        self.player_sfx(i, 0x9B9E);
                        self.player_panim(i, 1, 0xF);
                        if self.player_pstep(i) {
                            self.player_idle_entry(i);
                        }
                        return;
                    }
                }
                self.player_pmove(i);
            }
            _ => {}
        }
    }

    /// `if $59 { $59 -= 1; return }` used by most attack states.
    fn player_hitstop_wait(&mut self, i: usize) -> bool {
        let p = &mut self.players[i];
        if p.b.hitstop != 0 {
            p.b.hitstop -= 1;
            return true;
        }
        false
    }
}

impl Game {
    // ---------------------------------------------------------------------------------------
    // Run (mode 2, sub 0, act 2)
    // ---------------------------------------------------------------------------------------

    /// 0x19FA8
    fn player_run_group(&mut self, i: usize) {
        if self.players[i].b.airborne != 0 {
            return self.player_fall_entry(i);
        }
        match self.players[i].b.step {
            0 => self.player_run(i),
            2 => self.player_run_swing(i),
            // 0x04 (the two-player tie-up, entered from 0x6866) and 0x06 (0x1E1EE): not ported.
            _ => self.player_idle_entry(i),
        }
    }

    /// 0x81FB0: spawn pool B class 35 at the player's position, holding the player's handle in +$76.
    pub fn player_b35_spawn(&mut self, i: usize) -> Option<Handle> {
        let (x, y, z) = (self.players[i].b.xi(), self.players[i].b.yi(), self.players[i].b.zi());
        let h = self.spawn_prop(35, 0, Some((x, y, z)))?;
        if let Some(crate::game::Slot::Prop(k)) = crate::game::slot_of(h) {
            self.props.slots[k].m.sw(0x76, Game::player_handle(i));
        }
        Some(h)
    }

    /// 0x19F40 state 0x2C entry (action 0x3E, leader only): the pose with an effect at the chest.
    /// The spark 0xD (0x92422) at the hero's X, height + 0x58 and depth, then 41 frames of waiting in the handler.
    pub fn player_chest_pose_entry(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.b.set_state(0x0200_002C);
        p.b90 = 0;
        p.set_b80(0x28);
        let (x, y, z) = (p.b.xi(), p.b.yi(), p.b.zi());
        self.fx_spawn_spark_at(Game::player_handle(i), x, y.wrapping_add(0x58), z, 0xD);
    }

    /// 0x19F6A / 0x19F8A (a second copy of the pose routine is at 0x2864) state 0x2C: step 0 waits (`+$80` counts 0x28 to below 0),
    /// then makes a pool B class 35 object at the hero's place that holds his handle in `+$76` (0x81FB0) and starts animation 0x1F (0x19F8A / 0x2864);
    /// step 2 plays it and goes idle (0x19122) when finished.
    pub fn player_chest_pose(&mut self, i: usize) {
        match self.players[i].b90 {
            0 => {
                let p = &mut self.players[i];
                let old = p.b80();
                p.set_b80(old.wrapping_sub(1));
                if old != 0 {
                    return; // bcs only on the borrow
                }
                p.b90 += 2;
                self.player_b35_spawn(i);
                self.player_panim(i, 0, 0x1F);
            }
            2 => {
                if self.player_pstep(i) {
                    self.player_idle_entry(i); // bra 0x19122
                }
            }
            _ => {}
        }
    }

    /// 0x19FC6 run_entry (falls through).
    fn player_run_entry(&mut self, i: usize) {
        self.player_panim(i, 0, 2);
        {
            let p = &mut self.players[i];
            p.run_t = 0xF0;
            p.run_cd = 5;
        }
        self.player_run_entry_tail(i);
    }

    /// 0x19FDE: the rest of the run entry (also where the run swing 0x1A2F2 starts the run again).
    fn player_run_entry_tail(&mut self, i: usize) {
        {
            let p = &mut self.players[i];
            p.b.draw_flags = 0;
            p.action = 0;
            p.set_b80(0);
        }
        self.player_run_overlay(i); // 0x1A0F0: the pose of a held weapon
        {
            let p = &mut self.players[i];
            p.b.set_state(0x0200_0200);
            p.ba9 = 3;
        }
        let d = self.players[i].held & 0xF;
        self.player_walk_move(i, d); // first frame also moves at the previous speed class
        let p = &mut self.players[i];
        p.run_dir = p.b.dir;
        self.player_run(i);
    }

    /// 0x1A012 run.
    fn player_run(&mut self, i: usize) {
        if self.player_special_check(i) {
            return;
        }
        if self.players[i].run_t == 0 {
            return self.player_idle_entry(i);
        }
        {
            let p = &mut self.players[i];
            if p.b.wading != 0 {
                if (p.b.yi() as u16) < 0x20 {
                    let t = p.b80().wrapping_sub(1);
                    p.set_b80(t);
                    if t == 0xFF {
                        p.set_b80(0x14);
                        self.player_sfx(i, 0x9B60);
                    }
                    // stage 1 area 1: 0x60E2 water ripples (effects TODO)
                }
            } else {
                let t = p.b80().wrapping_sub(1);
                p.set_b80(t);
                if t == 0xFF {
                    p.set_b80(0x0C);
                    self.player_sfx(i, 0x9BCE);
                }
            }
        }
        let class = if self.players[i].b.param_a != 0 { 8 } else { 4 };
        self.player_set_speed_class(i, class);
        let p = &self.players[i];
        if p.pressed & ATK != 0 {
            return self.player_run_attack_entry(i); // 0x1A174
        }
        if p.pressed & JMP != 0 {
            return self.player_running_jump_entry(i);
        }
        if self.w.scripted != 0 && p.held & 0xF == 0 {
            return self.player_idle_entry(i);
        }
        let p = &mut self.players[i];
        p.run_dir = p.b.dir;
        self.player_run_move(i);
        let p = &self.players[i];
        if (p.b.dir as i8) >= 0 && (p.b.dir.wrapping_sub(p.run_dir).wrapping_add(4) & 0x1C) > 8 {
            self.players[i].atk_hold = 0;
            return self.player_idle_entry(i);
        }
        // 0x1A0DE: the legs layer of the held weapon steps.
        self.player_layer_tick(i);
    }

    // ---------------------------------------------------------------------------------------
    // Air (mode 2, sub 0, act 4), table 0x1A598
    // ---------------------------------------------------------------------------------------

    fn player_air(&mut self, i: usize) {
        match self.players[i].b.step {
            0x00 => self.player_jump(i),
            0x02 => self.player_wall_jump(i),
            0x04 => self.player_landing(i),
            0x06 => self.player_fall(i),
            0x08 => self.player_running_jump(i),
            0x0A => self.player_jump_attack_high(i),
            0x0C => self.player_jump_attack_low(i),
            0x0E => self.player_air_special(i),
            0x10 => self.player_air_special_fall(i),
            0x12 | 0x14 => self.player_lunge(i),
            0x16 => self.player_run_jump_attack_jack(i),
            0x28 => self.player_run_jump_attack_1(i),
            0x2A => self.player_run_jump_attack_2(i),
            0x2C => self.player_run_jump_attack_3(i),
            0x1E => self.player_heavy_jump(i),
            0x20 => self.player_heavy_land(i),
            0x2E => self.player_drop(i),
            0x30 => self.player_command_move(i),
            0x18 => self.player_pose_air_mustapha(i), // 0x1ADCC the pose in the air (Mustapha)
            0x1A => self.player_pose_air_hannah(i), // 0x1AE88 the pose in the air (Hannah)
            0x1C => self.player_scene_fall(i), // 0x1AFD0 the fall into Episode 1 area 2 (script action 0x1A)
            0x32 => self.player_vehicle_wait(i), // 0x1BBA6 wait in the vehicle until G.$505, then jump out
            0x34 => self.player_scripted_jump(i), // 0x1BC8A the scripted jump to a point (script action 0x32)
            0x36 => {
                if self.player_pstep(i) {
                    self.player_idle_entry(i);
                }
            }
            // 0x22..0x26 partner moves: TODO.
            _ => self.player_fall_entry(i),
        }
    }

    /// 0x1A5D0 jump_entry (falls through).
    fn player_jump_entry(&mut self, i: usize) {
        if self.players[i].item_type == 0x20A {
            return self.player_heavy_jump_entry(i); // 0x1A5D0 -> 0x1B038
        }
        {
            let p = &mut self.players[i];
            p.b.set_state(0x0200_0400);
            p.b.airborne = 1;
        }
        self.player_face_input(i);
        let nib = self.players[i].held & 0xF;
        self.players[i].b.dir = self.player_pd().jump_dir[nib as usize];
        self.player_set_jump_velocity(i, 0);
        let p = &mut self.players[i];
        let d = p.b.dir;
        let anim = if (d as i8) < 0 || d & 4 != 0 {
            3
        } else if p.facing_age >= 3 {
            4
        } else {
            p.b.facing ^= 1;
            5
        };
        p.b.draw_flags = 0;
        self.player_panim(i, 0, anim);
        let p = &mut self.players[i];
        p.wall_latch = 0;
        p.b83 = 0;
        p.set_b80(0);
        p.action = 1;
        self.player_jump(i);
    }

    /// 0x1A64A standing jump.
    fn player_jump(&mut self, i: usize) {
        if self.players[i].b.vy < 0 {
            self.player_splash_check(i);
            if self.players[i].b.airborne == 0 {
                let p = &mut self.players[i];
                p.b.vy = 0;
                p.b.ay = 0;
                return self.player_landing_entry(i);
            }
        } else if self.players[i].b80() == 0 && (self.player_height(i) as u16) >= 0x30 {
            let p = &self.players[i];
            let mut d1 = p.held & 3;
            if d1 != 0 {
                let d2 = p.prev & 3;
                if d2 != 0 && d2 != d1 {
                    d1 ^= 3;
                }
                if (p.b.dir as i8) < 0 {
                    let anim = if d1 == p.b.facing { 7 } else { 0xC };
                    self.players[i].set_b80(1);
                    self.player_panim(i, 0, anim);
                }
            }
        }
        self.player_pstep(i);
        self.player_pmove(i);
        self.player_air_control(i);
        if self.player_jump_attack_check(i) {
            return;
        }
        self.player_wall_jump_check(i);
    }

    /// 0x1A704; true when a jump attack started.
    fn player_jump_attack_check(&mut self, i: usize) -> bool {
        if self.players[i].held & ATK == 0 {
            return false;
        }
        let ch = self.players[i].b.id as usize & 3;
        let [min, split] = self.player_pd().jump_attack_height[ch];
        let h = self.player_height(i);
        if h < min {
            return false;
        }
        if h >= split {
            self.player_jump_attack_high_entry(i);
        } else {
            self.player_jump_attack_low_entry(i);
        }
        true
    }

    /// 0x1A744 wall jump check (characters 1 and 2); true when it started.
    fn player_wall_jump_check(&mut self, i: usize) -> bool {
        let p = &mut self.players[i];
        if p.b.id == 0 || p.b.id == 3 {
            return false;
        }
        let mut w = p.b.wall;
        if w == 0 {
            w = p.wall_latch;
        }
        p.wall_latch = w;
        if !(w == 4 || w == 0x18 || w == 0x1C) || p.pressed & JMP == 0 || p.b.vx == 0 {
            return false;
        }
        if self.player_height(i) < 0x20 {
            return false;
        }
        // 0x1A7A4
        self.players[i].b.set_state(0x0200_0402);
        self.player_panim(i, 0, 0x12);
        let p = &mut self.players[i];
        if p.b.vx < 0 {
            p.b.facing = 1;
            p.b.dir = 8;
        } else {
            p.b.facing = 0;
            p.b.dir = 0x18;
        }
        self.player_set_jump_velocity(i, 8);
        self.players[i].b83 = 0;
        self.player_wall_jump(i);
        true
    }

    /// 0x1A7D8
    fn player_wall_jump(&mut self, i: usize) {
        if self.players[i].b.vy < 0 {
            self.player_splash_check(i);
            if self.players[i].b.airborne == 0 {
                let p = &mut self.players[i];
                p.b.vy = 0;
                p.b.ay = 0;
                return self.player_landing_entry(i);
            }
        }
        self.player_pstep(i);
        if self.players[i].b.param_a != 0 {
            return;
        }
        self.player_pmove(i);
        self.player_air_control(i);
        self.player_jump_attack_check(i);
    }

    /// 0x1A81A landing_entry (falls through).
    fn player_landing_entry(&mut self, i: usize) {
        self.player_panim(i, 0, 6);
        self.players[i].b.set_state(0x0200_0404);
        self.player_sfx(i, 0x9C0C);
        self.player_landing(i);
    }

    /// 0x1A830: any input cancels the landing.
    fn player_landing(&mut self, i: usize) {
        if self.player_pstep(i) {
            return self.player_idle_entry(i);
        }
        if self.players[i].held & 0x3F != 0 {
            self.player_idle_entry(i);
        }
    }

    /// 0x1A848 fall_entry (walked off a ledge, or airborne in a ground state).
    fn player_fall_entry(&mut self, i: usize) {
        if self.players[i].b.act == 2 {
            self.player_set_speed_class(i, 4);
            let d = self.players[i].held & 0xF;
            self.player_walk_move(i, d);
        }
        let p = &mut self.players[i];
        p.b.set_state(0x0200_0406);
        p.b.airborne = 1;
        if p.b.vy >= 0 {
            p.b.vy = 0;
        }
        p.b.ay = -0x20;
        p.b.vz >>= 1;
        p.b.vx >>= 1;
        p.b.draw_flags = 0;
        p.b83 = 0;
        p.action = 1;
        if self.players[i].item_type == 0x20A {
            let anim = self.player_pd().heavy_fall_anim[self.players[i].b.id as usize & 3];
            self.player_panim(i, 5, anim as u16); // 0x1A8AA
        } else {
            self.player_panim(i, 0, 4);
        }
        self.player_fall(i);
    }

    /// 0x1A8D4: no animation step while falling.
    fn player_fall(&mut self, i: usize) {
        if self.players[i].b.vy < 0 {
            self.player_splash_check(i);
            if self.players[i].b.airborne == 0 {
                let p = &mut self.players[i];
                p.b.vy = 0;
                p.b.ay = 0;
                self.player_sfx(i, 0x9C0C);
                return self.player_idle_entry(i);
            }
        }
        self.player_pmove(i);
    }

    /// 0x1A906 running jump entry (falls through).
    fn player_running_jump_entry(&mut self, i: usize) {
        let (vx, vz) = {
            let p = &mut self.players[i];
            p.b.set_state(0x0200_0408);
            p.b.airborne = 1;
            (p.b.vx, p.b.vz)
        };
        self.player_set_jump_velocity(i, 4);
        let p = &mut self.players[i];
        if p.bc1 != 0 {
            p.b.vx = vx;
            p.b.vz = vz;
        }
        p.b.draw_flags = 0;
        p.b83 = 0;
        p.wall_latch = 0;
        self.player_panim(i, 0, 8);
        self.players[i].action = 2;
        self.player_running_jump(i);
    }

    /// 0x1A950
    fn player_running_jump(&mut self, i: usize) {
        if self.players[i].pressed & ATK != 0 {
            return self.player_running_jump_attack_entry(i);
        }
        if self.players[i].b.vy < 0 {
            self.player_splash_check(i);
            if self.players[i].b.airborne == 0 {
                let p = &mut self.players[i];
                p.b.vy = 0;
                p.b.ay = 0;
                p.atk_hold = 0;
                p.b.draw_flags = 0;
                return self.player_landing_entry(i);
            }
        }
        self.player_pstep(i);
        self.player_pmove(i);
        self.player_wall_jump_check(i);
    }

    /// 0x1A9A4 dive kick (high jump attack).
    fn player_jump_attack_high_entry(&mut self, i: usize) {
        self.players[i].b.set_state(0x0200_040A);
        self.players[i].b.draw_flags = 0;
        self.player_panim(i, 1, 0xA);
        self.player_face_input(i);
        let ch = self.players[i].b.id as usize & 3;
        let [vx, vy] = self.player_pd().dive[ch];
        let p = &mut self.players[i];
        p.b.vy = vy;
        p.b.ay = -0x20;
        let left = p.b.facing == 0;
        p.b.vx = signed(vx, left);
        p.b.ax = signed(0x20, left);
        p.b.vz = 0;
        p.b.az = 0;
        self.player_new_attack(i, 0x0E);
        let p = &mut self.players[i];
        p.combo = 0;
        p.b.wall = 0;
        p.action = 0x14;
        self.player_jump_attack_high(i);
    }

    /// 0x1AA16
    fn player_jump_attack_high(&mut self, i: usize) {
        if self.player_hitstop_wait(i) {
            return;
        }
        if self.players[i].b.vy < 0 {
            self.player_splash_check(i);
            if self.players[i].b.airborne == 0 {
                let p = &mut self.players[i];
                p.b.vy = 0;
                p.b.ay = 0;
                // 0x1BD56 dive landing
                self.player_panim(i, 0, 0x1E);
                self.players[i].b.set_state(0x0200_0436);
                self.player_sfx(i, 0x9C0C);
                if self.player_pstep(i) {
                    self.player_idle_entry(i);
                }
                return;
            }
        }
        self.player_pstep(i);
        self.player_pmove(i);
    }

    /// 0x1AA54 low jump attack.
    fn player_jump_attack_low_entry(&mut self, i: usize) {
        self.players[i].b.set_state(0x0200_040C);
        self.players[i].b.draw_flags = 0;
        self.player_panim(i, 1, 0xB);
        let p = &mut self.players[i];
        if p.b.vx != 0 {
            p.b.facing = (p.b.vx > 0) as u8;
        }
        self.player_new_attack(i, 0x0C);
        let p = &mut self.players[i];
        p.combo = 0;
        p.b.wall = 0;
        p.b.vz = 0;
        p.b.az = 0;
        p.action = 0x14;
        self.player_jump_attack_low(i);
    }

    /// 0x1AAA8
    fn player_jump_attack_low(&mut self, i: usize) {
        if self.player_hitstop_wait(i) {
            return;
        }
        if self.players[i].b.vy < 0 {
            self.player_splash_check(i);
            if self.players[i].b.airborne == 0 {
                let p = &mut self.players[i];
                p.b.vy = 0;
                p.b.ay = 0;
                return self.player_landing_entry(i);
            }
        }
        self.player_pstep(i);
        self.player_pmove(i);
    }

    /// 0x1AAE6 Mustapha's airborne special (falls through).
    fn player_air_special_entry(&mut self, i: usize) {
        self.players[i].b.set_state(0x0200_040E);
        self.players[i].b.draw_flags = 0;
        self.player_panim(i, 1, 8);
        self.player_new_attack(i, 0x12);
        let p = &mut self.players[i];
        p.combo = 0;
        p.item = 0;
        p.item_type = 0;
        p.drop = 0;
        p.b.vy = 0x4C0;
        p.b.ay = -0x40;
        p.b.vx = 0;
        p.b.ax = 0;
        p.b.vz = 0;
        p.b.az = 0;
        p.b.airborne = 1;
        self.player_air_special(i);
    }

    /// 0x1AB44 rising.
    fn player_air_special(&mut self, i: usize) {
        if self.players[i].b.param_a != 0 {
            self.player_pstep(i);
            return;
        }
        if self.player_hitstop_wait(i) {
            return;
        }
        self.player_pmove(i);
        if self.players[i].b.vy < 0 {
            // 0x1AB6E: 15-frame hover, then fall (falls through into 0x1AB7C)
            let p = &mut self.players[i];
            p.set_b80(0x0F);
            p.b.set_state(0x0200_0410);
            return self.player_air_special_fall(i);
        }
        self.player_pstep(i);
    }

    /// 0x1AB7C
    fn player_air_special_fall(&mut self, i: usize) {
        if self.player_hitstop_wait(i) {
            return;
        }
        if self.players[i].b.vy < 0 {
            self.player_splash_check(i);
            if self.players[i].b.airborne == 0 {
                let p = &mut self.players[i];
                p.b.vy = 0;
                p.b.ay = 0;
                p.b.live = 2;
                self.player_set_invuln(i, 12);
                return self.player_landing_entry(i);
            }
        }
        let p = &mut self.players[i];
        let t = p.b80().wrapping_sub(1);
        p.set_b80(t);
        if t == 0xFF {
            p.set_b80(0);
            self.player_pmove(i);
        }
        self.player_pstep(i);
    }

    /// 0x1ABD4 airborne lunge of characters 1, 2, 3 (falls through).
    fn player_lunge_entry(&mut self, i: usize) {
        self.players[i].b.set_state(0x0200_0412);
        self.player_new_attack(i, 0x14);
        let v = self.player_pd().vectors_1bf06;
        let p = &mut self.players[i];
        p.combo = 0;
        p.b.draw_flags = 0;
        p.set_b81(0);
        p.b.vz = 0;
        p.b.az = 0;
        let k = match p.b.id {
            1 => 18,
            2 => 22,
            _ => 26,
        };
        let left = p.b.facing == 0;
        p.b.vx = signed(v[k], left);
        p.b.ax = signed(v[k + 1], left);
        p.b.vy = v[k + 2];
        p.b.ay = v[k + 3];
        p.b.airborne = 1;
        self.player_panim(i, 1, 0xC);
        let p = &mut self.players[i];
        p.set_b80(p.b.param_a);
        self.player_lunge(i);
    }

    /// 0x1AC5A
    fn player_lunge(&mut self, i: usize) {
        if self.player_hitstop_wait(i) {
            return;
        }
        self.player_pstep(i);
        self.player_multi_hit_tick(i, 0x5A, 0x5C);
        if self.players[i].b.vy < 0 {
            self.player_splash_check(i);
            if self.players[i].b.airborne == 0 {
                let p = &mut self.players[i];
                p.b.vy = 0;
                p.b.ay = 0;
                p.b.set_state(0x0200_0012);
                self.player_panim(i, 0, 0xD);
                self.players[i].dtap = 0;
                return self.player_recovery(i);
            }
        }
        self.player_pmove(i);
    }

    /// 0x1ACE8 running-jump attack by character.
    fn player_running_jump_attack_entry(&mut self, i: usize) {
        let ch = self.players[i].b.id;
        let state = [0x0200_0416, 0x0200_0428, 0x0200_042A, 0x0200_042C][ch as usize & 3];
        self.players[i].b.set_state(state);
        self.player_panim(i, 1, 0xD);
        self.player_new_attack(i, 0x10);
        let p = &mut self.players[i];
        match ch {
            0 => {
                p.set_b80(p.b.param_a);
                self.player_run_jump_attack_jack(i);
            }
            1 => {
                p.set_b81(2);
                p.set_b80(p.b.param_a);
                self.player_run_jump_attack_1(i);
            }
            2 => {
                p.combo = 0;
                self.player_run_jump_attack_2(i);
            }
            _ => {
                p.set_b80(p.b.param_a);
                self.player_run_jump_attack_3(i);
            }
        }
    }

    /// Landing test of the running-jump attacks: true when landed (state changed).
    fn player_air_attack_land(&mut self, i: usize) -> bool {
        if self.players[i].b.vy < 0 {
            self.player_splash_check(i);
            if self.players[i].b.airborne == 0 {
                let p = &mut self.players[i];
                p.b.vy = 0;
                p.b.ay = 0;
                p.atk_hold = 0;
                self.player_landing_entry(i);
                return true;
            }
        }
        false
    }

    /// 0x1AD32 Jack (state 0x416).
    fn player_run_jump_attack_jack(&mut self, i: usize) {
        let p = &mut self.players[i];
        if p.b.hitstop != 0 {
            p.b.hitstop -= 1;
            if p.b.hitstop != 0 {
                return;
            }
            p.b.timer = 1;
        }
        let p = &mut self.players[i];
        let old = p.b80();
        p.set_b80(p.b.param_a);
        if old != p.b.param_a {
            self.player_new_attack(i, 0x70);
        }
        if self.player_air_attack_land(i) {
            return;
        }
        self.player_pstep(i);
        self.player_pmove(i);
    }

    /// 0x1B4C8 character 1 (state 0x428).
    fn player_run_jump_attack_1(&mut self, i: usize) {
        let p = &mut self.players[i];
        if p.b.hitstop != 0 {
            p.b.hitstop -= 1;
            if p.b.hitstop != 0 {
                return;
            }
            if p.b81() != 0 {
                p.set_b81(p.b81() - 1);
                p.b.timer = 1;
            }
        }
        let p = &mut self.players[i];
        let old = p.b80();
        p.set_b80(p.b.param_a);
        if old != p.b.param_a {
            self.player_new_attack(i, 0x70);
            if self.players[i].b.param_a != 1 {
                self.player_new_attack(i, 0x72);
            }
        }
        if self.player_air_attack_land(i) {
            return;
        }
        self.player_pstep(i);
        self.player_pmove(i);
    }

    /// 0x1B592 character 2 (state 0x42A).
    fn player_run_jump_attack_2(&mut self, i: usize) {
        if self.player_hitstop_wait(i) {
            return;
        }
        if self.player_air_attack_land(i) {
            return;
        }
        self.player_pstep(i);
        self.player_pmove(i);
    }

    /// 0x1B600 character 3 (state 0x42C).
    fn player_run_jump_attack_3(&mut self, i: usize) {
        let p = &mut self.players[i];
        if p.b.hitstop != 0 {
            p.b.hitstop -= 1;
            if p.b.hitstop != 0 {
                return;
            }
            if p.b.param_b != 0 {
                p.b.timer = 1;
            }
        }
        let p = &mut self.players[i];
        let old = p.b80();
        p.set_b80(p.b.param_a);
        if old != p.b.param_a {
            self.player_new_attack(i, 0x70);
        }
        if self.player_air_attack_land(i) {
            return;
        }
        self.player_pstep(i);
        self.player_pmove(i);
    }

    /// 0x1B67C drop from above (state 0x42E).
    pub fn player_drop_entry(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.b.set_state(0x0200_042E);
        p.b.airborne = 1;
        p.b.vy = -0x100;
        p.b.ay = -0x80;
        p.b.vx = 0;
        p.b.ax = 0;
        p.b.vz = 0;
        p.b.az = 0;
        p.b.draw_flags = 0;
        p.b83 = 0;
        p.action = 1;
        self.player_panim(i, 0, 4);
    }

    /// 0x1B6BE
    fn player_drop(&mut self, i: usize) {
        if self.players[i].b.vy < 0 {
            self.player_splash_check(i);
            if self.players[i].b.airborne == 0 {
                let p = &mut self.players[i];
                p.b.vy = 0;
                p.b.ay = 0;
                self.player_sfx(i, 0x9C0C);
                return self.player_idle_entry(i);
            }
        }
        self.player_pmove(i);
    }
}

impl Game {
    // ---------------------------------------------------------------------------------------
    // Hurt (mode 2, sub 2), reaction code in $06, table 0x1C03E
    // ---------------------------------------------------------------------------------------

    fn player_hurt(&mut self, i: usize) {
        self.player_special_buffer(i); // 0x63E6
        match self.players[i].b.act {
            0x00 => self.player_light(i, 0),
            0x02 => self.player_light(i, 1),
            0x04 => self.player_knockdown(i, 4),
            0x06 => self.player_knockdown(i, 6),
            0x08 => self.player_thrown(i),
            0x0A | 0x0C => self.player_burn(i),
            0x0E | 0x10 | 0x12 => self.player_pushback(i, false),
            0x14 | 0x16 | 0x18 => self.player_pushback(i, true),
            0x1A => self.player_light_behind(i),
            0x1C => self.player_knockdown_behind(i),
            0x1E => self.player_getup(i),
            0x20 => self.player_wall_hit(i),
            0x22 => self.player_stagger(i),
            _ => self.player_recover(i),
        }
    }

    /// Attacker on my right while facing left, or on my left while facing right.
    fn player_behind(&self, i: usize) -> bool {
        let p = &self.players[i];
        let ax = if p.b.attacker != 0 && slot_of(p.b.attacker).is_some() { self.body(p.b.attacker).xi() } else { p.b.xi() };
        let d1 = ((ax as u16) < (p.b.xi() as u16)) as u8; // bcc: unsigned
        d1 == p.b.facing
    }

    /// The knockdown override of light reactions.
    fn player_must_fall(&self, i: usize) -> bool {
        let p = &self.players[i];
        p.b.hp == 0 || p.b.stun == 0 || p.b.knock != 0 || p.b.airborne != 0
    }

    /// 0x1C062 / 0x1C124 light hit (anim 0 / 1).
    fn player_light(&mut self, i: usize, anim: u16) {
        if self.players[i].b.step == 0 {
            self.players[i].b.palette_override = 0;
            if self.player_behind(i) {
                let p = &mut self.players[i];
                p.b.act = 0x1A;
                p.b.step = 0;
                return self.player_light_behind(i);
            }
            if self.player_must_fall(i) {
                return self.player_knockdown(i, if anim == 0 { 4 } else { 6 });
            }
            self.player_panim(i, 2, anim);
            self.player_sfx(i, 0x98CC);
            // 0x16B18(2) face icon, 0x16446 HP bar: HUD port reads +$6C / +$6E.
            let p = &mut self.players[i];
            p.action = 0x20;
            p.b.step = 2;
        }
        let p = &mut self.players[i];
        p.b.hitstop = p.b.hitstop.wrapping_sub(1);
        if p.b.hitstop == 0 {
            self.player_recover(i);
        }
    }

    /// 0x1CB84 light hit from behind: flips at the end.
    fn player_light_behind(&mut self, i: usize) {
        if self.players[i].b.step == 0 {
            if self.player_must_fall(i) {
                return self.player_knockdown_behind_entry(i);
            }
            self.player_panim(i, 2, 6);
            self.player_sfx(i, 0x98CC);
            let p = &mut self.players[i];
            p.action = 0x20;
            p.b.step = 2;
        }
        let p = &mut self.players[i];
        p.b.hitstop = p.b.hitstop.wrapping_sub(1);
        if p.b.hitstop == 0 {
            p.b.facing ^= 1;
            self.player_recover(i);
        }
    }

    /// 0x1C0DA recover.
    fn player_recover(&mut self, i: usize) {
        self.players[i].b.palette_override = 0;
        if self.players[i].b.hp == 0 {
            self.players[i].b.set_state(0x0400_0000);
            return self.player_dead(i);
        }
        self.rank_event_183e4(); // 0x1C0F4 / 0x1F276
        if self.players[i].b.reaction == 4 {
            self.players[i].b.live = 2;
            self.player_set_invuln(i, 0);
        } else {
            self.players[i].b.live = 1;
        }
        self.player_idle_entry(i);
    }

    /// Knockdown vector load with the facing rule "negate when facing right" (pushed back).
    fn player_kd_vec(&mut self, i: usize, k: usize, neg_when_right: bool) {
        let [vx, ax, vy, ay] = self.player_pd().knockdown[k];
        let p = &mut self.players[i];
        let n = if neg_when_right { p.b.facing != 0 } else { p.b.facing == 0 };
        p.b.vx = signed(vx, n);
        p.b.ax = signed(ax, n);
        p.b.vy = vy;
        p.b.ay = ay;
    }

    /// Common tail of a knockdown start.
    fn player_kd_common(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.b.vz = 0;
        p.b.az = 0;
        p.item_type = 0;
        p.drop = 1;
        p.b.airborne = 1;
        p.b83 = 0;
        p.b.reaction = 4;
        p.b.stun = 10;
        p.b.step = p.b.step.wrapping_add(2);
    }

    /// 0x1D0C0 knockdown motion (slow motion for dizzy / death).
    fn player_kd_motion(&mut self, i: usize) {
        let f = self.w.frame;
        match self.players[i].slow {
            0 => self.player_pmove(i),
            1 => {
                if f & 3 != 0 {
                    self.player_pmove(i);
                }
            }
            _ => {
                if f & 2 != 0 {
                    self.player_pmove(i);
                }
            }
        }
    }

    /// 0x1D0FA dizzy roll at knockdown start.
    fn player_dizzy_roll(&mut self, i: usize) {
        let p = &self.players[i];
        if p.tech != 0 || p.b.hp < 0x1E || (p.dizzy_cd >> 8) != 0 || p.slow != 0 || p.hits == 0 {
            let p = &mut self.players[i];
            p.hits = 0;
            p.slow = 0;
            return;
        }
        let n = p.hits.min(5) as usize;
        let mask = self.player_pd().dizzy_mask[n];
        let r = self.rand() & 0xF;
        let p = &mut self.players[i];
        p.hits = 0;
        if (mask >> r) & 1 != 0 {
            p.slow = 1;
            p.b7e = 0;
            p.b7f = 0;
        } else {
            p.slow = 0;
        }
    }

    /// `move.b $5ee(a5),d1 ; andi.w #$18,d1` (0x1CE28, 0x1CE4C, 0x1D188, 0x1D1A8): the 68000 is big
    /// endian, so the byte at the address of the level word is its HIGH byte. The level is 0..0x1F,
    /// so the band is always 0 (the rows 8, 16 and 24 of the tables are never used).
    fn player_level_band(&self) -> usize {
        ((self.w.level >> 8) as u8 & 0x18) as usize
    }

    /// 0x1D17E lie-down setup (two rand() calls).
    fn player_lie_setup(&mut self, i: usize) {
        let row = self.player_level_band() * 2;
        let r = (self.rand() & 0xF) as usize;
        let t = self.player_pd().lie_time[(row + r) & 63];
        let r = (self.rand() & 0xF) as usize;
        let m = self.player_pd().lie_mash[(row + r) & 63];
        let p = &mut self.players[i];
        p.lie_t = t;
        p.mash = m;
        p.mash_cd = 0;
    }

    /// 0x6606: lets an enemy claim this downed player (enemy AI, `enemy/claim.rs`).
    fn player_downed_claim(&mut self, i: usize) {
        self.enemy_claim_downed_player(i);
    }

    /// 0x1C19C / 0x1C37C knockdown (type 4 or 6).
    fn player_knockdown(&mut self, i: usize, kind: u8) {
        if self.players[i].b.act != kind {
            let p = &mut self.players[i];
            p.b.act = kind;
            p.b.step = 0;
        }
        self.combat.push(0x6AC2, Game::player_handle(i)); // 0x4B58
        let base = if kind == 4 { 0 } else { 2 };
        match self.players[i].b.step {
            0 => {
                // 0x1C1C0 / 0x1C3A0
                let p = &mut self.players[i];
                p.b.palette_override = 0;
                p.action = 0x23;
                p.tech = 0;
                if self.player_behind(i) {
                    return self.player_knockdown_behind_entry(i);
                }
                self.player_panim(i, 2, if kind == 4 { 2 } else { 8 });
                self.player_kd_vec(i, base, true);
                self.player_dizzy_roll(i);
                if self.players[i].b.hp == 0 {
                    self.player_kd_vec(i, base + 1, true);
                    self.players[i].slow = 2;
                    self.player_sfx(i, 0x986C);
                }
                self.player_kd_common(i);
                self.player_kd_air1(i, kind);
            }
            2 => self.player_kd_air1(i, kind),
            4 => self.player_kd_air2(i, kind),
            _ => self.player_kd_lie(i),
        }
    }

    /// 0x1C4F0: stop the horizontal slowdown once |vx| < 0x40.
    fn player_ax_clamp(&mut self, i: usize) {
        let p = &mut self.players[i];
        let v = p.b.vx;
        if v >= 0 {
            if (v as u16) < 0x40 {
                p.b.ax = 0;
            }
        } else if (v as u16) >= 0xFFC0 {
            p.b.ax = 0;
        }
    }

    /// 0x1C290 / 0x1C470 first flight, bounce on landing.
    fn player_kd_air1(&mut self, i: usize, kind: u8) {
        self.player_downed_claim(i);
        self.player_splash_check(i);
        self.player_kd_motion(i);
        if kind == 6 {
            self.player_ax_clamp(i);
        }
        if self.players[i].b.param_a == 0 {
            self.player_pstep(i);
        }
        self.player_floor_and_walls(i);
        let w = self.players[i].b.wall;
        if w != 0 && w & 1 == 0 {
            return self.player_wall_hit_entry(i);
        }
        if self.players[i].b.airborne != 0 {
            return;
        }
        let alive = self.players[i].b.hp != 0;
        let k = if kind == 4 { 0 } else { 2 } + if alive { 0 } else { 1 };
        let [_, _, vy, ay] = self.player_pd().knockdown[k];
        let p = &mut self.players[i];
        p.b.airborne = 1;
        p.b.vy = vy;
        p.b.ay = ay.wrapping_mul(2);
        self.player_pmove(i);
        self.players[i].b.step += 2;
        self.player_pstep(i);
        self.player_sfx(i, 0x9796);
    }

    /// 0x1C302 / 0x1C514 second flight, then lie.
    fn player_kd_air2(&mut self, i: usize, kind: u8) {
        self.player_downed_claim(i);
        self.player_kd_motion(i);
        if kind == 6 {
            self.player_ax_clamp(i);
        }
        if self.players[i].b.param_a == 0 {
            self.player_pstep(i);
        }
        self.player_floor_and_walls(i);
        if self.players[i].b.airborne != 0 {
            return;
        }
        self.player_lie_setup(i);
        if self.players[i].tech != 0 && self.players[i].b.hp != 0 {
            self.player_panim(i, 2, 9);
            let p = &mut self.players[i];
            p.b.facing ^= 1;
            p.b.live = 1;
        }
        self.player_sfx(i, 0x9796);
        self.players[i].b.step += 2;
    }

    /// 0x1C35A lying.
    fn player_kd_lie(&mut self, i: usize) {
        self.player_downed_claim(i);
        self.player_mash(i);
        if self.players[i].mash == 0 {
            return self.player_getup_entry(i);
        }
        let p = &mut self.players[i];
        p.lie_t = p.lie_t.wrapping_sub(1);
        if p.lie_t == 0xFF {
            return self.player_getup_entry(i);
        }
        self.player_pstep(i);
    }

    /// 0x1CBE6 knockdown from behind (reaction 0x1C).
    fn player_knockdown_behind_entry(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.b.act = 0x1C;
        p.b.step = 0;
        self.player_knockdown_behind(i);
    }

    /// 0x1CBF2
    fn player_knockdown_behind(&mut self, i: usize) {
        match self.players[i].b.step {
            0 => {
                // 0x1CC06
                let p = &mut self.players[i];
                p.action = 0x23;
                p.tech = 0; // 0x1D244 with d0 = 1 also only clears $CC
                self.player_panim(i, 2, 7);
                let p = &mut self.players[i];
                let left = p.b.facing == 0;
                p.b.vx = signed(0x340, left);
                p.b.ax = 0;
                p.b.vy = 0x500;
                p.b.ay = -0x4A;
                self.player_dizzy_roll(i);
                if self.players[i].b.hp == 0 {
                    let p = &mut self.players[i];
                    p.b.vx = signed(0x340, left);
                    p.b.ax = 0;
                    p.b.vy = 0x540;
                    p.b.ay = -0x4A;
                    p.slow = 2;
                    self.player_sfx(i, 0x986C);
                }
                self.player_kd_common(i);
                self.player_kb_air1(i);
            }
            2 => self.player_kb_air1(i),
            4 => {
                // 0x1CD2E
                self.player_downed_claim(i);
                self.player_kd_motion(i);
                if self.players[i].b.param_a == 0 {
                    self.player_pstep(i);
                }
                self.player_floor_and_walls(i);
                if self.players[i].b.airborne == 0 {
                    self.player_lie_setup(i);
                    self.player_sfx(i, 0x9796);
                    self.players[i].b.step += 2;
                    self.player_panim(i, 2, 9);
                }
            }
            _ => {
                // 0x1CD6A
                self.player_downed_claim(i);
                let p = &mut self.players[i];
                p.lie_t = p.lie_t.wrapping_sub(1);
                let timeout = p.lie_t == 0xFF;
                if !timeout {
                    self.player_mash(i);
                    if self.players[i].mash != 0 {
                        return;
                    }
                }
                if self.players[i].b.hp != 0 {
                    self.players[i].b.facing ^= 1;
                }
                self.player_getup_entry(i);
            }
        }
    }

    /// 0x1CCBA: first flight from behind; bounce keeps the landing speed (vy = -vy).
    fn player_kb_air1(&mut self, i: usize) {
        self.player_downed_claim(i);
        self.player_splash_check(i);
        self.player_kd_motion(i);
        if self.players[i].b.param_a == 0 {
            self.player_pstep(i);
        }
        self.player_floor_and_walls(i);
        let w = self.players[i].b.wall;
        if w != 0 && w & 1 == 0 {
            self.players[i].b.facing ^= 1;
            return self.player_wall_hit_entry(i);
        }
        if self.players[i].b.airborne != 0 {
            return;
        }
        let p = &mut self.players[i];
        p.b.airborne = 1;
        p.b.vy = p.b.vy.wrapping_neg();
        p.b.ay = p.b.ay.wrapping_mul(2);
        self.player_pmove(i);
        self.player_pstep(i);
        self.players[i].b.step += 2;
        self.player_sfx(i, 0x9796);
    }

    /// 0x1C56E thrown (reaction 8).
    fn player_thrown(&mut self, i: usize) {
        self.combat.push(0x6AC2, Game::player_handle(i));
        match self.players[i].b.step {
            0 => {
                let p = &mut self.players[i];
                p.b.palette_override = 0;
                p.action = 0x23;
                p.tech = 0;
                self.player_panim(i, 2, 8);
                let p = &mut self.players[i];
                let right = p.b.facing != 0;
                p.b.vx = signed(0x600, right);
                p.b.ax = signed(-0x18, right);
                p.b.vy = 0x500;
                p.b.ay = -0x40;
                self.player_dizzy_roll(i);
                if self.players[i].b.hp == 0 {
                    let p = &mut self.players[i];
                    p.b.vx = signed(0x600, right);
                    p.b.ax = signed(-0x18, right);
                    p.b.vy = 0x500;
                    p.b.ay = -0x40;
                    p.slow = 2;
                    self.player_sfx(i, 0x986C);
                }
                self.player_kd_common(i);
                self.player_thrown_air1(i);
            }
            2 => self.player_thrown_air1(i),
            4 => {
                // 0x1C6AC
                self.player_downed_claim(i);
                self.player_kd_motion(i);
                self.player_ax_clamp(i);
                if self.players[i].b.param_a == 0 {
                    self.player_pstep(i);
                }
                self.player_floor_and_walls(i);
                if self.players[i].b.airborne == 0 {
                    self.player_lie_setup(i);
                    self.player_sfx(i, 0x9796);
                    self.players[i].b.step += 2;
                }
            }
            _ => self.player_kd_lie(i),
        }
    }

    /// 0x1C636
    fn player_thrown_air1(&mut self, i: usize) {
        self.player_splash_check(i);
        self.player_kd_motion(i);
        self.player_ax_clamp(i);
        if self.players[i].b.param_a == 0 {
            self.player_pstep(i);
        }
        self.player_floor_and_walls(i);
        let w = self.players[i].b.wall;
        if w != 0 && w & 1 == 0 {
            return self.player_wall_hit_entry(i);
        }
        if self.players[i].b.airborne != 0 {
            return;
        }
        let p = &mut self.players[i];
        p.b.airborne = 1;
        p.b.vy = p.b.vy.wrapping_neg();
        p.b.ay = p.b.ay.wrapping_mul(2);
        self.player_pmove(i);
        self.players[i].b.step += 2;
        self.player_pstep(i);
        self.player_sfx(i, 0x9796);
    }

    /// 0x1C6E4 burn knockdown (reactions 0x0A / 0x0C).
    fn player_burn(&mut self, i: usize) {
        self.combat.push(0x6AC2, Game::player_handle(i));
        match self.players[i].b.step {
            0 => {
                let ax = {
                    let p = &self.players[i];
                    if p.b.attacker != 0 && slot_of(p.b.attacker).is_some() { self.body(p.b.attacker).xi() } else { p.b.xi() }
                };
                let p = &mut self.players[i];
                p.b.palette_override = 0;
                p.b.facing = ((ax as u16) >= (p.b.xi() as u16)) as u8; // face the attacker
                p.tech = 0;
                self.player_panim(i, 2, 2);
                self.player_kd_vec(i, 4, true);
                self.player_dizzy_roll(i);
                if self.players[i].b.hp == 0 {
                    self.player_kd_vec(i, 5, true);
                    self.players[i].slow = 2;
                    self.player_sfx(i, 0x986C);
                }
                let p = &mut self.players[i];
                p.b.vz = 0;
                p.b.az = 0;
                self.hud_life_bar(i); // 0x1C794
                let me = Game::player_handle(i);
                let flames = self.player_pd().burn_flames;
                for (k, [dx, dy, var]) in flames.iter().enumerate() {
                    self.fx_spawn_flame(me, *dx, *dy, *var as u16, k as u16); // 0x1C86E
                }
                let p = &mut self.players[i];
                p.b.palette_override = 0x84;
                self.spawn.g.b7b2 = 2; // 0x1C7A4: burn palette flag.
                p.item_type = 0;
                p.drop = 1;
                p.b.airborne = 1;
                p.b83 = 0;
                p.b.reaction = 4;
                p.b.stun = 10;
                self.player_sfx(i, 0x9796);
                self.players[i].b.step += 2;
                self.player_kd_air1(i, 4); // bra 0x1C290 (this frame only)
            }
            2 => {
                // 0x1C7D8
                self.player_downed_claim(i);
                self.player_splash_check(i);
                self.player_kd_motion(i);
                if self.players[i].b.param_a == 0 {
                    self.player_pstep(i);
                }
                self.player_floor_and_walls(i);
                if self.players[i].b.airborne != 0 {
                    return;
                }
                self.player_lie_setup(i);
                let k = if self.players[i].b.hp != 0 { 4 } else { 5 };
                let [_, _, vy, ay] = self.player_pd().knockdown[k];
                let p = &mut self.players[i];
                p.b.airborne = 1;
                p.b.vy = vy;
                p.b.ay = ay.wrapping_mul(2);
                self.player_pmove(i);
                let p = &mut self.players[i];
                p.b.step += 2;
                p.b.palette_override = 0;
                self.player_pstep(i);
                self.player_sfx(i, 0x9796);
            }
            4 => self.player_kd_air2(i, 4),
            _ => {
                // 0x1C844
                self.player_downed_claim(i);
                self.player_mash(i);
                if self.players[i].mash == 0 {
                    self.spawn.g.b7b2 = 0; // 0x1C860
                    return self.player_getup_entry(i);
                }
                let p = &mut self.players[i];
                p.lie_t = p.lie_t.wrapping_sub(1);
                if p.lie_t == 0xFF {
                    self.spawn.g.b7b2 = 0; // 0x1C860
                    return self.player_getup_entry(i);
                }
                self.player_pstep(i);
            }
        }
    }

    /// 0x1C8B2 / 0x1CA26 pushback reactions.
    fn player_pushback(&mut self, i: usize, heavy: bool) {
        match self.players[i].b.step {
            0 => {
                self.players[i].b.palette_override = 0;
                let code = self.players[i].b.act;
                let k = if heavy { (code.wrapping_sub(0x14) / 2) as usize } else { (code.wrapping_sub(0x0E) / 2) as usize };
                let [frames, vx, ax] = self.player_pd().pushback[k.min(2)];
                self.players[i].set_b80(frames as u8);
                if self.player_behind(i) {
                    if self.player_must_fall(i) {
                        return self.player_knockdown_behind_entry(i);
                    }
                    let p = &mut self.players[i];
                    let left = p.b.facing == 0;
                    p.b.vx = signed(vx, left);
                    p.b.ax = signed(ax, left);
                    self.player_panim(i, 2, 6);
                    self.players[i].turn = 1;
                } else {
                    if self.player_must_fall(i) {
                        return self.player_knockdown(i, if heavy { 6 } else { 4 });
                    }
                    let p = &mut self.players[i];
                    let right = p.b.facing != 0;
                    p.b.vx = signed(vx, right);
                    p.b.ax = signed(ax, right);
                    self.player_panim(i, 2, 0);
                    self.players[i].turn = 0;
                }
                let p = &mut self.players[i];
                p.b.vy = 0;
                p.b.ay = 0;
                p.b.vz = 0;
                p.b.az = 0;
                self.player_sfx(i, 0x98CC);
                let p = &mut self.players[i];
                p.action = 0x20;
                p.b.step = 2;
                // 1/64: 0x5A454 (enemy-side helper; carry clear for players in practice).
                let _ = self.rand();
                self.player_pushback_stop(i);
            }
            2 => self.player_pushback_stop(i),
            _ => self.player_pushback_slide(i),
        }
    }

    fn player_pushback_stop(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.b.hitstop = p.b.hitstop.wrapping_sub(1);
        if p.b.hitstop == 0 {
            p.b.step = 4;
            self.player_pushback_slide(i);
        }
    }

    /// 0x1C9F4
    fn player_pushback_slide(&mut self, i: usize) {
        let p = &mut self.players[i];
        let t = p.b80().wrapping_sub(1);
        p.set_b80(t);
        if t == 0 {
            if p.turn != 0 {
                p.b.facing ^= 1;
            }
            return self.player_recover(i);
        }
        // 0x1CA0E / 0x1CB60 (and the copies 0x1FB56 / 0x1FCA4): every other frame (bit 0 of G.$1) the dust 0x92E5E with d0 = 2, then 0x189C.
        if self.w.frame & 1 != 0 {
            let b = self.players[i].b.clone();
            self.fx_spawn_dust(&b, 2);
        }
        self.player_pmove(i);
    }

    /// 0x1CF9C wall hit entry (reaction 0x20).
    fn player_wall_hit_entry(&mut self, i: usize) {
        {
            let p = &mut self.players[i];
            p.b.act = 0x20;
            p.b.step = 0;
            p.set_b80(0x1E);
        }
        self.player_panim(i, 2, 5);
        let p = &mut self.players[i];
        p.b.palette_override = 0;
        p.b.facing = (p.b.vx < 0) as u8;
        self.player_sfx(i, 0x9796);
        self.player_wall_hit(i);
    }

    /// 0x1CFCE: hang 30 frames, slide down 1.5 px/frame, lie.
    fn player_wall_hit(&mut self, i: usize) {
        if self.players[i].b.step == 0 {
            let p = &mut self.players[i];
            if p.b80() != 0 {
                p.set_b80(p.b80() - 1);
                return;
            }
            p.b.y = p.b.y.wrapping_sub(0x18000);
            let floor = p.b.floor.wrapping_add(p.b.floor2);
            if floor < p.b.yi() {
                return;
            }
            Player::set_hi(&mut p.b.y, floor);
            p.b.airborne = 0;
            self.player_lie_setup(i);
            self.players[i].b.step += 2;
            self.player_pstep(i);
        } else {
            let p = &mut self.players[i];
            p.lie_t = p.lie_t.wrapping_sub(1);
            if p.lie_t == 0xFF {
                p.b.stun = 10;
                return self.player_getup_entry(i);
            }
            self.player_pstep(i);
        }
    }

    /// 0x1D032 stagger (reaction 0x22).
    fn player_stagger(&mut self, i: usize) {
        match self.players[i].b.step {
            0 => {
                let p = &self.players[i];
                if p.b.hp == 0 || p.b.stun == 0 || p.b.knock != 0 || p.b.reaction == 4 || p.b.reaction == 6 {
                    return self.player_knockdown(i, 4);
                }
                self.player_panim(i, 2, 0xA);
                self.player_sfx(i, 0x98CC);
                let p = &mut self.players[i];
                p.action = 0x22;
                p.b.step = 2;
                self.player_stagger(i);
            }
            2 => {
                let p = &mut self.players[i];
                p.b.hitstop = p.b.hitstop.wrapping_sub(1);
                if p.b.hitstop == 0 {
                    p.mash_cd = 0;
                    p.b.live = 1;
                    p.b.step = 4;
                    self.player_panim(i, 2, 9);
                }
            }
            _ => {
                let p = &mut self.players[i];
                p.lie_t = p.lie_t.wrapping_sub(1);
                if p.lie_t == 0xFF {
                    return self.player_recover(i);
                }
                self.player_mash(i);
                if self.players[i].mash == 0 {
                    self.player_recover(i);
                }
            }
        }
    }

    /// 0x1CD9A getup entry.
    fn player_getup_entry(&mut self, i: usize) {
        if self.players[i].b.hp == 0 {
            let p = &mut self.players[i];
            p.b.set_state(0x0400_0000);
            p.b.palette_override = 0;
            return;
        }
        {
            let p = &mut self.players[i];
            p.b.set_state(0x0202_1E00);
            p.stun_refill = 0x78;
            p.b.knock = 0;
            p.b.stun = 10;
        }
        self.player_panim(i, 2, 3);
        self.players[i].b.palette_override = 0;
        self.player_getup(i);
    }

    /// 0x1CDDA getup / dizzy.
    fn player_getup(&mut self, i: usize) {
        if self.players[i].b.step == 0 {
            if !self.player_pstep(i) {
                return;
            }
            if self.players[i].slow == 0 {
                return self.player_recover(i);
            }
            // 0x1CE02 dizzy entry
            self.players[i].b.step = 2;
            self.player_panim(i, 2, 4);
            let row = self.player_level_band();
            let r = (self.rand() & 0xF) as usize;
            let stars = self.player_pd().dizzy_stars[(row * 2 + r) & 63];
            let r = (self.rand() & 0x1E) as usize;
            let time = self.player_pd().dizzy_time[((row * 4 + r) / 2) & 63];
            let p = &mut self.players[i];
            p.hits = 0;
            p.hits_t = 0;
            p.action = 0x21;
            p.mash = stars;
            p.mash_cd = 0;
            p.w80 = time;
            p.dizzy_cd = time.wrapping_add(0x708);
            p.b.live = 1; // a dizzy player can be hit
            // 0x1CE7E: d0 = +$80, then 0x94338: the stars over the hero.
            let (b, w80) = (self.players[i].b.clone(), self.players[i].w80);
            self.fx_spawn_dizzy(Game::player_handle(i), &b, w80);
        }
        // 0x1CE88
        let before = self.players[i].mash;
        self.players[i].b109 = before;
        self.player_mash(i);
        let p = &mut self.players[i];
        if p.b109.wrapping_sub(p.mash) >= 1 && p.b.timer != 1 {
            p.b.timer = 1;
        }
        if p.mash != 0 {
            p.w80 = p.w80.wrapping_sub(1);
            if p.w80 != 0xFFFF {
                self.player_pstep(i);
                return;
            }
        }
        let p = &mut self.players[i];
        p.slow = 0;
        p.b7e = 0;
        p.b7f = 0;
        self.player_recover(i);
    }

    // ---------------------------------------------------------------------------------------
    // Grabbing an enemy (mode 2, sub 4)
    // ---------------------------------------------------------------------------------------

    /// 0x1D652
    fn player_grabbing(&mut self, i: usize) {
        if self.players[i].cmd_flag == 2 {
            return self.player_command_move_entry(i);
        }
        match self.players[i].b.act {
            0 => self.player_hold(i),
            2 | 4 => self.player_knee(i),
            _ => self.player_throw(i),
        }
    }

    /// 0x1D676 hold.
    fn player_hold(&mut self, i: usize) {
        if self.players[i].b.step == 0 {
            self.player_hold_entry(i);
        }
        let e = self.players[i].b.partner;
        let ok = e != 0 && slot_of(e).is_some() && {
            let eb = self.body(e);
            eb.kind == 0xE && eb.sub == 6
        };
        if !ok {
            return self.player_release(i);
        }
        let p = &mut self.players[i];
        p.escape = p.escape.wrapping_sub(1);
        if p.escape == 0xFF {
            return self.player_release(i);
        }
        let d = p.held & 0xF;
        let toward = if p.b.facing != 0 { d & 1 } else { d & 2 };
        if d == 0 || toward != 0 {
            p.ba9 = 0x78;
        } else {
            p.ba9 = p.ba9.wrapping_sub(1);
            if p.ba9 == 0xFF {
                return self.player_release(i);
            }
        }
        if self.player_special_check(i) {
            return;
        }
        let p = &self.players[i];
        if p.pressed & ATK != 0 {
            let b72 = self.body(e).b72;
            let p = &mut self.players[i];
            p.escape = b72;
            if p.held & 3 != 0 {
                return self.player_throw_entry(i);
            }
            return self.player_knee_entry(i);
        }
        if p.pressed & JMP != 0 {
            return self.player_release(i);
        }
        self.player_pstep(i);
    }

    /// 0x1D67C
    fn player_hold_entry(&mut self, i: usize) {
        self.player_panim(i, 3, 0);
        let p = &mut self.players[i];
        p.b.draw_flags = 0;
        p.action = 0x30;
        p.ba9 = 0x78;
        p.b.set_state(0x0204_0002);
    }

    /// 0x1D72A
    fn player_release(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.atk_buf = 0;
        p.combo_t = 0;
        p.regrab_cd = 10;
        self.player_idle_entry(i);
    }

    /// 0x1D746 knee entry (falls through).
    fn player_knee_entry(&mut self, i: usize) {
        if self.players[i].knees == 0 {
            self.players[i].b.set_state(0x0204_0200);
            self.player_panim(i, 3, 1);
            self.player_new_attack(i, 6);
        } else {
            self.players[i].b.set_state(0x0204_0400);
            self.player_panim(i, 3, 2);
            self.player_new_attack(i, 8);
        }
        let p = &mut self.players[i];
        p.knees = p.knees.wrapping_add(1);
        self.player_knee(i);
    }

    /// 0x1D778 / 0x1D810
    fn player_knee(&mut self, i: usize) {
        let second = self.players[i].b.act == 4;
        let old = self.players[i].b.param_a;
        if self.player_pstep(i) {
            self.player_hold_entry(i);
            return self.player_hold(i);
        }
        if self.players[i].b.param_a <= old {
            return;
        }
        let e = self.players[i].b.partner;
        if e == 0 || slot_of(e).is_none() {
            return;
        }
        self.player_grab_hit(i); // 0x12526
        // 0x1D792: unless the hit threw the enemy into a reaction (sub 2), its HP bar goes to the
        // HUD (0x16B36): kind 2 when the enemy's +$82 is 0, else kind 0.
        let reacting = self.body(e).sub == 2;
        if !reacting {
            let kind = if self.enemy_holder_tag(e) == 0 { 2 } else { 0 };
            self.enemy_hud_request(e, kind);
        }
        // 0x1D7BA: spark at offset 0x1D93E[char * 2 + second] (0x128C8). The second knee jumps to
        // the first knee's call (0x1D7B2, offset 0) when the enemy is in a reaction (it was killed).
        let second = second && !reacting;
        let p = &self.players[i];
        let [dx, dy] = self.player_pd().knee_spark[(p.b.id as usize & 3) * 2 + second as usize];
        let x = p.b.xi().wrapping_add(if p.b.facing != 0 { dx.wrapping_neg() } else { dx });
        let y = p.b.yi().wrapping_add(dy);
        self.fx_spawn_spark_simple(Game::player_handle(i), x, y);
    }

    /// 0x1D850 throw entry (falls through).
    fn player_throw_entry(&mut self, i: usize) {
        let p = &mut self.players[i];
        let anim = if p.b.id == 3 {
            let toward = p.held & if p.b.facing != 0 { 1 } else { 2 };
            if toward != 0 { 3 } else { 4 }
        } else {
            let d = p.held & 3;
            if d != 0 {
                p.b.facing = d & 1;
            }
            3
        };
        self.player_panim(i, 3, anim);
        self.player_throw_start(i);
    }

    /// 0x1D888: combo throw from the finisher.
    fn player_throw_from_combo(&mut self, i: usize) {
        self.player_panim(i, 3, 3);
        self.player_throw_start(i);
    }

    /// 0x1D890
    fn player_throw_start(&mut self, i: usize) {
        self.players[i].b.set_state(0x0204_0600);
        self.player_new_attack(i, 0x0A);
        self.player_throw(i);
    }

    /// 0x1D8AE
    fn player_throw(&mut self, i: usize) {
        if self.player_pstep(i) {
            self.players[i].b.live = 2;
            self.player_set_invuln(i, 4);
            return self.player_release(i);
        }
        let p = &self.players[i];
        let e = p.b.partner;
        if p.b.param_a == 0 || e == 0 || slot_of(e).is_none() {
            return;
        }
        self.player_grab_hit(i);
        let (facing, k20, pa, px, py) = {
            let p = &self.players[i];
            (p.b.facing, p.b.k20, p.b.param_a, p.b.xi(), p.b.yi())
        };
        let mut k = k20;
        let mut efacing = facing;
        if k == 0xC && pa != 2 {
            k += 4;
            efacing = facing ^ 1;
        }
        let [dx, dy] = self.player_pd().throw_offset[((k / 4) as usize).min(4)];
        let eb = self.body_mut(e);
        eb.set_state(0x0202_0000);
        eb.act = 8;
        eb.facing = efacing;
        let x = px.wrapping_add(if facing != 0 { dx.wrapping_neg() } else { dx });
        Player::set_hi(&mut eb.x, x);
        Player::set_hi(&mut eb.y, py.wrapping_add(dy));
        self.players[i].b.partner = 0;
    }

    // ---------------------------------------------------------------------------------------
    // Held by an enemy (mode 2, sub 6) 0x1D2BE
    // ---------------------------------------------------------------------------------------

    fn player_grabbed(&mut self, i: usize) {
        match self.players[i].b.act {
            0 => {
                // 0x1D2D4
                let p = &mut self.players[i];
                p.action = 0x40;
                p.b75 = 0xFF;
                p.mash = 0x28;
                p.mash_cd = 0;
                p.b.reaction = 0xFF;
                p.b.hitstop = 0;
                p.set_b80(0xB4);
                p.item_type = 0;
                p.b.act = 2;
                p.b.step = 0;
                p.b.b62 = 0;
                if p.bc7 != 0 {
                    p.b.step = 2;
                    p.b.b62 = 1;
                }
                self.player_grabbed_hold(i);
            }
            2 => self.player_grabbed_hold(i),
            4 => self.player_grabbed_escape(i),
            _ => {
                // 0x1D4A4
                if self.player_pstep(i) {
                    self.players[i].slow = 0;
                    self.player_getup_entry(i);
                }
            }
        }
    }

    /// 0x1D31A
    fn player_grabbed_hold(&mut self, i: usize) {
        let p = &self.players[i];
        let broke = p.b.stun == 0 || p.b.knock != 0 || p.b.reaction == 4 || p.b.reaction == 6;
        if p.b.step == 0 {
            if broke || self.player_held_follow(i) {
                return self.player_idle_entry(i);
            }
            let e = self.players[i].b.partner;
            if self.body(e).param_a != 0 {
                self.player_pstep(i);
                return;
            }
            if self.players[i].mash == 0 {
                return self.player_idle_entry(i);
            }
            self.player_grabbed_mash(i);
            return;
        }
        // $07 != 0
        let p = &mut self.players[i];
        let t = p.b80().wrapping_sub(1);
        p.set_b80(t);
        if t == 0xFF || broke {
            return self.player_grabbed_stumble(i);
        }
        if self.player_held_follow(i) {
            return self.player_idle_entry(i);
        }
        let p = &mut self.players[i];
        if p.b.hitstop != 0 {
            p.b.hitstop -= 1;
            if p.b.hitstop != 0 {
                self.player_pstep(i);
                return;
            }
            p.b.reaction = 0xFF;
        }
        if self.players[i].mash == 0 {
            return self.player_grabbed_escape_entry(i);
        }
        self.player_grabbed_mash(i);
    }

    fn player_grabbed_mash(&mut self, i: usize) {
        let before = self.players[i].mash;
        self.player_mash(i);
        let p = &mut self.players[i];
        if p.mash != before {
            let t = (p.b.timer as i8).wrapping_sub(10);
            p.b.timer = if t <= 0 { 1 } else { t as u8 };
        }
        self.player_pstep(i);
    }

    /// 0x1D48E
    fn player_grabbed_stumble(&mut self, i: usize) {
        self.players[i].action = 0x20;
        self.players[i].b.set_state(0x0206_0600);
        self.player_panim(i, 4, 2);
        if self.player_pstep(i) {
            self.players[i].slow = 0;
            self.player_getup_entry(i);
        }
    }

    /// 0x1D3F8 escape attack.
    fn player_grabbed_escape_entry(&mut self, i: usize) {
        self.players[i].b.set_state(0x0206_0400);
        self.player_panim(i, 4, 3);
        self.player_new_attack(i, 0x18);
        self.player_grabbed_escape(i);
    }

    /// 0x1D41E
    fn player_grabbed_escape(&mut self, i: usize) {
        if self.player_pstep(i) {
            return self.player_idle_entry(i);
        }
        let e = self.players[i].b.partner;
        if e == 0 || slot_of(e).is_none() {
            return;
        }
        if self.players[i].b.id == 1 {
            // Mustapha reverses the hold.
            if self.players[i].b.kind == 0xC {
                let me = Game::player_handle(i);
                let who = self.players[i].b.who;
                let eb = self.body_mut(e);
                eb.partner = me;
                eb.set_state(0x0206_0000);
                eb.kind = 0xE;
                eb.who = who;
                eb.hp_prev = eb.hp;
                self.players[i].b.set_state(0x0204_0600);
            }
            return;
        }
        if self.players[i].b.param_a == 1 {
            self.players[i].b.param_a = 0;
            self.player_grab_hit(i);
            let f = self.players[i].b.facing;
            self.body_mut(e).facing = f;
        }
    }

    /// 0x1D4B8: follow the holder; true (carry) when the hold is over.
    fn player_held_follow(&mut self, i: usize) -> bool {
        let e = self.players[i].b.partner;
        let Some(Slot::Enemy(_)) = slot_of(e) else {
            // Held by a player (0x1D582, uses +$60): not ported.
            return true;
        };
        let eb = self.body(e).clone();
        if eb.live == 0 || eb.mode != 2 || eb.sub != 4 || eb.kind != 0xC {
            return true;
        }
        let blob = &self.assets.player.held_offsets;
        let base = dino_assets::player::HELD_OFFSETS_BASE;
        let rd16 = |a: u32| -> u16 {
            let k = (a.wrapping_sub(base)) as usize;
            if k + 1 < blob.len() { u16::from_be_bytes([blob[k], blob[k + 1]]) } else { 0 }
        };
        let rd32 = |a: u32| -> u32 { (rd16(a) as u32) << 16 | rd16(a + 2) as u32 };
        let k20 = self.players[i].b.k20 as u32;
        let t = rd32(base + k20);
        let w84 = self.player_enemy_w84(e) as u32;
        let off = rd16(t + (w84 >> 1)) as i16 as i32;
        let row = (t as i32 + off) as u32 + ((eb.param_b as u32 & 0x3F) << 3);
        let dx = rd16(row) as i16;
        let dy = rd16(row + 2) as i16;
        let fac = rd16(row + 4) as u8;
        let anim = rd16(row + 6);
        let p = &mut self.players[i];
        let x = eb.xi().wrapping_add(if eb.facing != 0 { dx.wrapping_neg() } else { dx });
        // 0x1D512: `move.w $8(a0),$8(a6)` copies the integer words only, the hero keeps his own fractions.
        Player::set_hi(&mut p.b.x, eb.xi());
        Player::set_hi(&mut p.b.y, eb.yi());
        Player::set_hi(&mut p.b.z, eb.zi());
        Player::set_hi(&mut p.b.x, x);
        let y = eb.yi().wrapping_add(dy);
        Player::set_hi(&mut p.b.y, y);
        Player::set_hi(&mut p.b.z, eb.zi());
        p.b.facing = if eb.facing != 0 { fac ^ 1 } else { fac };
        let r = p.b.reaction;
        if (r as i8) < 0 {
            if eb.param_b == p.b75 {
                return false;
            }
            p.b75 = eb.param_b;
            self.player_panim(i, 4, anim);
        } else {
            self.player_panim(i, 4, if r == 0 { 4 } else { 5 });
        }
        false
    }

    // ---------------------------------------------------------------------------------------
    // Mode 4: dead 0x1BF42
    // ---------------------------------------------------------------------------------------

    fn player_dead(&mut self, i: usize) {
        match self.players[i].b.sub {
            0 => {
                let p = &mut self.players[i];
                p.b.live = 2;
                p.set_b80(0x3C);
                p.b.sub = 2;
                // every 32 frames 0x5DD30 (voice). Not ported.
            }
            2 => {
                if self.w.stage_clear != 0 {
                    let p = &mut self.players[i];
                    p.b.hp = 1;
                    p.b.hp_prev = 1;
                    p.b.live = 1;
                    return self.player_idle_entry(i);
                }
                self.player_pstep(i);
                let p = &mut self.players[i];
                let t = p.b80().wrapping_sub(1);
                p.set_b80(t);
                if t != 0xFF {
                    p.b.draw_flags = if t & 1 != 0 { 0 } else { 8 };
                    return;
                }
                p.b.draw_flags = 0;
                p.b.sub = 4;
                self.player_dead_continue(i);
            }
            _ => self.player_dead_continue(i),
        }
    }

    /// 0x1BFD0 lives check: respawn or game over.
    fn player_dead_continue(&mut self, i: usize) {
        if self.players[i].lives != 0 {
            let p = &mut self.players[i];
            p.lives -= 1;
            p.b.live = 1;
            p.b.set_state(0);
            // 0x1BFEC: 0x184E2 reads +$A3, so the spent life is put back for that call.
            p.lives = p.lives.wrapping_add(1);
            let player_a3 = p.lives;
            let lives = self.hud_lives_sum();
            let (b4cf, b4e6) = (self.flow.b4cf(), self.flow.b4e6);
            self.rank_event_continue(player_a3, lives, b4cf, b4e6); // 0x1BFF0
            self.players[i].lives = self.players[i].lives.wrapping_sub(1); // 0x1BFF6
            let p = &mut self.players[i];
            let facing = p.b.facing;
            p.b.set_state(0x0002_0000); // 0x4DE8
            p.b.hp = 100;
            p.b.hp_prev = 100;
            p.b.hp_bar = 100;
            p.b.facing = facing;
            p.b.step = 2;
            self.rank_respawn_time(); // 0x1C010: jmp 0x10482, +3:30 on the stage clock and 600 frames of wave time
        } else {
            let p = &mut self.players[i];
            p.b.live = 0;
            self.player_count_players();
            let player_a3 = self.players[i].lives;
            let lives = self.hud_lives_sum();
            let (b4cf, b4e6) = (self.flow.b4cf(), self.flow.b4e6);
            self.rank_event_continue(player_a3, lives, b4cf, b4e6); // 0x1C020
            self.rank_event_1842c(); // 0x1C026
        }
    }

    /// Enemy type +$84 (name id * 4), read from the attacker (a pool A enemy; a pool E item keeps
    /// whatever its +$84 holds). Other objects read 0.
    pub fn player_enemy_w84(&self, h: Handle) -> u16 {
        match slot_of(h) {
            Some(Slot::Enemy(i)) => self.enemies.slots[i].m.w(super::enemy::off::W84),
            Some(Slot::Item(i)) => self.items.slots[i].m.w(0x84),
            _ => 0,
        }
    }
}
