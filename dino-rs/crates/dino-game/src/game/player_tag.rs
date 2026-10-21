//! The tag team move ("two player moves": Let's Go, Let's Roll, Jump, Let's Fly; M9-01, M9-02). A child module of `player` (it uses the private helpers of the state
//! machine). D2a section 9, D2c section 15.
//!
//! Two heroes stand still on top of each other for 90 frames: the one that is deeper on the screen (the initiator A) swings the other (the partner B).
//! * The trigger 0x670C is called once a frame at the end of the object update (0xE9C8, after the held players; `Game::tag_trigger`). Per initiator two counters
//!   (`+$C2`, `+$C3`: one for each of the other two players, table 0x66F4) count down from 0x5A; see `tag_check` for the rules. At 0 the start 0x686E (`tag_start`)
//!   makes A swing (state 0x0200001A, 0x19A7C, `player_tag_swing`) and B the partner (frame class 0xE, so `held_players_update` runs it after the enemies, state by the
//!   byte pair table 0x6902).
//! * The partner states: Jack's partner runs (the run group step 4, 0x1A330, `player_tag_run`), the others fly (the air steps 0x22 Hannah 0x1B0F6, 0x24 Mustapha 0x1B226,
//!   0x26 Mess 0x1B356, `player_tag_air`); until the release frame of the swing (`+$2C` of A) the follow routine 0x6922 (`tag_follow`) puts B at the place of the table
//!   0x69D4.. by A's phase `+$2D`.
//! * Both heroes blink (`+$EC` = 180) for 180 frames: the partner from its first update, the initiator from the end of its swing.
//!
//! The tables are read from the ROM (dino-extract): 0x6902 (the partner state by initiator and partner), 0x6994 (the follow table pointers by initiator and partner),
//! 0x69D4 (the follow offsets), 0x6244 (the bubble variant by hero, used by `fx_spawn_bubbles_now`).

use super::{ATK, JMP};
use crate::game::{slot_of, Game, Slot};

/// 0x66F4: the two other players of each player, in the order of the counters `+$C2` and `+$C3`.
const OTHERS: [[usize; 2]; 3] = [[1, 2], [0, 2], [0, 1]];
/// 0x6902: the partner state (act, step) by `initiator k20 * 2 + partner k20 / 2`, 0x6994: the follow table by `initiator k20 * 4 + partner k20`.
const PARTNER_STATE: u32 = 0x6902;
const FOLLOW_PTRS: u32 = 0x6994;
/// 0x1A164: the split body offset tables of the run pose (by hero), 0x1A228: the run swing pose.
const RUN_OFFSETS: u32 = 0x1A164;
const RUN_SWING_POSE: u32 = 0x1A228;

impl Game {
    /// 0x670C: the trigger for all three players. Not while a scripted scene (G.$4DD) runs.
    pub fn tag_trigger(&mut self) {
        if self.w.scripted != 0 {
            return;
        }
        for a in 0..3 {
            self.tag_check(a);
        }
    }

    /// 0x6728: one initiator A. Nothing for a hero that is tied up already (`+$C1`), invulnerable (`+$00` bit 0 clear: the counters stay as they are), holds a heavy
    /// object (0x20A) or has no input. Any input of A resets both counters to 0x5A. For each of the other players B (the first one in the table, then the second):
    /// B must be live and not invulnerable (`+$00` bit 0), in the normal mode (`+$05` 0), without a heavy object, frame class 0 (`+$2E`), action 0 (`+$C8`), without
    /// input, `A.x - B.x` from -32 to 31 and `A.z - B.z` from 0 to 15 (unsigned words: A is level with B or deeper). When it holds the counter of B counts down and at 0
    /// the tie up starts (the second B is not looked at then); else the counter goes back to 0x5A.
    fn tag_check(&mut self, a: usize) {
        {
            let p = &self.players[a];
            if p.bc1 != 0 || p.b.live & 1 == 0 || p.item_type == 0x20A || p.input_lock != 0 {
                return;
            }
        }
        if self.players[a].held != 0 {
            let p = &mut self.players[a];
            p.c2 = 0x5A;
            p.c3 = 0x5A;
            return;
        }
        let who = self.players[a].b.who as usize % 3;
        let [b1, b2] = OTHERS[who];
        if self.tag_partner_ok(a, b1) {
            let p = &mut self.players[a];
            p.c2 = p.c2.wrapping_sub(1);
            if p.c2 == 0 {
                return self.tag_start(a, b1);
            }
        } else {
            self.players[a].c2 = 0x5A;
        }
        if self.tag_partner_ok(a, b2) {
            let p = &mut self.players[a];
            p.c3 = p.c3.wrapping_sub(1);
            if p.c3 == 0 {
                self.tag_start(a, b2);
            }
        } else {
            self.players[a].c3 = 0x5A;
        }
    }

    /// The tests of 0x6728 for the other player B (0x677A to 0x67D8).
    fn tag_partner_ok(&self, a: usize, b: usize) -> bool {
        let (pa, pb) = (&self.players[a], &self.players[b]);
        pb.b.live & 1 != 0
            && pb.b.sub == 0
            && pb.item_type != 0x20A
            && pb.b.kind == 0
            && pb.action == 0
            && pb.held == 0
            && (pa.b.xi().wrapping_sub(pb.b.xi()).wrapping_add(0x20) as u16) < 0x40
            && (pa.b.zi() as u16) >= (pb.b.zi() as u16)
            && (pa.b.zi() as u16).wrapping_sub(pb.b.zi() as u16) < 0x10
    }

    /// 0x686E: the tie up. A: invulnerable (`+$00` 2), tied (`+$C1`), state 0x0200001A, action 0x12, frame class 0xC, the partner link; both face right when A is
    /// less than 0xC0 from the left edge of the camera, else left. B: placed on A (one deeper), tied, action 0x12, frame class 0xE, its state (act, step) from the
    /// table 0x6902; then the bubbles of 0x61F6 (`fx_spawn_bubbles_now`, owner A).
    fn tag_start(&mut self, a: usize, b: usize) {
        let (ha, hb) = (Game::player_handle(a), Game::player_handle(b));
        let facing = (self.players[a].b.xi().wrapping_sub(self.cam.s2.cam_x as i16) as u16) < 0xC0;
        {
            let p = &mut self.players[a];
            p.b.live = 2;
            p.bc1 = 1;
            p.b.set_state(0x0200_001A);
            p.b90 = 0;
            p.b.partner = hb;
            p.action = 0x12;
            p.b.kind = 0xC;
            p.b.facing = facing as u8;
        }
        let (x, y, z, ka) = {
            let p = &self.players[a].b;
            (p.xi(), p.yi(), p.zi(), p.k20)
        };
        let idx = (ka as u32).wrapping_mul(2).wrapping_add(self.players[b].b.k20 as u32 / 2);
        let t = &self.assets.player.t;
        let (act, step) = (t.u8(PARTNER_STATE, idx), t.u8(PARTNER_STATE, idx + 1));
        {
            let p = &mut self.players[b];
            p.b.facing = facing as u8;
            Self::tag_set_hi(&mut p.b.x, x);
            Self::tag_set_hi(&mut p.b.y, y);
            Self::tag_set_hi(&mut p.b.z, z.wrapping_sub(1));
            p.b.partner = ha;
            p.action = 0x12;
            p.bc1 = 1;
            p.b.kind = 0xE;
            p.b.act = act;
            p.b.step = step;
            p.b90 = 0;
        }
        self.fx_spawn_bubbles_now(ha); // bra 0x61F6
    }

    /// `move.w d0,$8(a6)`: the integer word of a 16.16 coordinate.
    fn tag_set_hi(v: &mut i32, w: i16) {
        *v = ((w as i32) << 16) | (*v & 0xFFFF);
    }

    /// The initiator that holds partner `i` (`+$70`).
    fn tag_initiator(&self, i: usize) -> Option<usize> {
        match slot_of(self.players[i].b.partner) {
            Some(Slot::Player(k)) => Some(k),
            _ => None,
        }
    }

    /// 0x6922: the partner follows the initiator. True (carry) when the tie is over: the initiator is gone, hurt or not in the swing action 0x12 (the partner goes
    /// idle). Else the partner takes the facing and the place of the follow table (the offset by the initiator's phase `+$2D`, X mirrored for a facing left, Y added,
    /// one deeper than the initiator).
    pub(super) fn tag_follow(&mut self, i: usize) -> bool {
        let Some(a) = self.tag_initiator(i) else { return true };
        {
            let pa = &self.players[a];
            if pa.b.live == 0 || pa.b.sub != 0 || pa.action != 0x12 {
                return true;
            }
        }
        let (ka, kb) = (self.players[a].b.k20 as u32, self.players[i].b.k20 as u32);
        let t = &self.assets.player.t;
        let ptr = t.u32(FOLLOW_PTRS, ka.wrapping_mul(4).wrapping_add(kb));
        let phase = self.players[a].b.param_b as u32;
        let row = ptr.wrapping_add(phase * 4);
        let (mut dx, dy) = (t.u16(row, 0) as i16, t.u16(row, 2) as i16);
        let (ax, ay, az, fac) = {
            let pa = &self.players[a].b;
            (pa.xi(), pa.yi(), pa.zi(), pa.facing)
        };
        if fac != 0 {
            dx = dx.wrapping_neg();
        }
        let p = &mut self.players[i];
        p.b.facing = fac;
        Self::tag_set_hi(&mut p.b.x, ax.wrapping_add(dx));
        Self::tag_set_hi(&mut p.b.y, ay.wrapping_add(dy));
        Self::tag_set_hi(&mut p.b.z, az.wrapping_sub(1));
        false
    }

    /// 0x13EA: animation `anim` of the player's set 0 at the frame `idx` (12 bytes a frame after the first one).
    fn player_anim_at(&mut self, i: usize, anim: u16, idx: u8) {
        if let Some(f) = self.player_set0_frame(i, anim, idx) {
            let s = &self.assets.sprites;
            self.players[i].b.load_frame(s, f);
        }
    }

    /// 0x19A7C state 0x0200001A, the initiator's swing: animation 0:0xF once (the pose has no hurt box, the release frame is the frame byte `+$2C`); at its end 180
    /// frames of blinking invulnerability (`+$EC`, `+$00` 2) and the idle (0x19122, which also frees `+$C1`).
    pub(super) fn player_tag_swing(&mut self, i: usize) {
        if self.players[i].b90 == 0 {
            self.players[i].b.draw_flags = 0;
            self.player_panim(i, 0, 0xF);
            self.players[i].b90 += 2;
        }
        if self.player_pstep(i) {
            let p = &mut self.players[i];
            p.respawn_t = 0xB4;
            p.b.live = 2;
            self.player_idle_entry(i);
        }
    }

    /// 0x1A330 the run group step 4: Jack's partner. `+$90`: 0 the blink starts, 2 follows the swing (animation 0:0x17 at the initiator's phase) until the release
    /// frame, 4 the run, 6 the run attack (animation 1:9, attack id 0x1A with a direction held).
    pub(super) fn player_tag_run(&mut self, i: usize) {
        let mut st = self.players[i].b90;
        if st == 0 {
            let p = &mut self.players[i];
            p.b.draw_flags = 0;
            p.respawn_t = 0xB4;
            p.b.live = 2;
            p.b90 += 2;
            st = 2;
        }
        if st == 2 {
            if self.tag_follow(i) {
                return self.player_idle_entry(i);
            }
            let Some(a) = self.tag_initiator(i) else { return self.player_idle_entry(i) };
            let idx = self.players[a].b.param_b;
            self.player_anim_at(i, 0x17, idx);
            if self.players[a].b.param_a == 0 {
                return;
            }
            // 0x1A37C: the release: the run starts (+$80 0, +$81 0x28 frames of straight running)
            {
                let p = &mut self.players[i];
                p.b90 += 2;
                p.b.draw_flags = 0;
                p.action = 0x12;
                p.set_b80(0);
                p.set_b81(0x28);
            }
            self.player_run_overlay(i); // bsr 0x1A0F0
            self.players[i].ba9 = 3;
            self.player_set_speed_class(i, 12);
            let nib = if self.players[i].b.facing != 0 { 1 } else { 2 };
            self.player_walk_move(i, nib); // bsr 0x1BD94
            let p = &mut self.players[i];
            p.run_dir = p.b.dir;
            st = 4;
        }
        if st == 4 {
            return self.player_tag_run_step(i);
        }
        self.player_tag_run_attack(i);
    }

    /// 0x1A3BE: one frame of the partner's run. The footstep counter `+$80` (reload 10, the sound 0x9BCE) unless wading deeper than 0x20, attack starts the run
    /// attack, jump the running jump; with no direction held and `+$81` at 0 the run ends 3 frames later (`+$A9`); the first 28 frames (`+$81` from 0x28 down to 0x14)
    /// run straight ahead, then the stick steers; a turn of more than a quarter ends the run.
    fn player_tag_run_step(&mut self, i: usize) {
        if self.players[i].b.wading == 0 || (self.players[i].b.yi() as u16) < 0x20 {
            let old = self.players[i].b80();
            self.players[i].set_b80(old.wrapping_sub(1));
            if old == 0 {
                self.players[i].set_b80(0xA);
                self.player_sfx(i, 0x9BCE);
            }
        }
        if self.players[i].pressed & ATK != 0 {
            return self.player_tag_run_attack_entry(i);
        }
        if self.players[i].pressed & JMP != 0 {
            return self.player_running_jump_entry(i);
        }
        let nib = self.players[i].held & 0xF;
        {
            let p = &mut self.players[i];
            if nib == 0 && p.b81() == 0 {
                let old = p.ba9;
                p.ba9 = old.wrapping_sub(1);
                if old == 0 {
                    p.atk_hold = 0;
                    return self.player_idle_entry(i);
                }
            } else {
                p.ba9 = 3;
                if (p.b.dir as i8) >= 0 {
                    p.run_dir = p.b.dir;
                }
            }
        }
        let b81 = self.players[i].b81();
        let mut straight = false;
        if b81 != 0 {
            self.players[i].set_b81(b81 - 1);
            straight = b81 > 0x14;
        }
        if !straight && nib != 0 {
            self.player_walk_move(i, nib); // 0x1BD94
        } else {
            self.player_pstep(i);
            self.player_pmove(i);
        }
        let p = &self.players[i];
        if (p.b.dir as i8) >= 0 && (p.b.dir.wrapping_sub(p.run_dir).wrapping_add(4) & 0x1C) > 8 {
            self.players[i].atk_hold = 0;
            return self.player_idle_entry(i);
        }
        self.player_layer_tick(i); // 0x1A486: jmp 0x1542
    }

    /// 0x1A498: the attack button in the partner's run: the legs layer (set 0 animation 0x18 at `+$2D`, its timer from `+$2F`), the pose of the held item, then the
    /// attack 0x1A (hit stop 0, a new swing serial, `+$B2` 0) and the state step 6.
    fn player_tag_run_attack_entry(&mut self, i: usize) {
        let k20 = self.players[i].b.k20 as u32;
        let offs = self.assets.player.t.u32(RUN_OFFSETS, k20);
        {
            let b = &mut self.players[i].b;
            b.offs_table = offs;
            b.draw_flags = 1;
        }
        self.players[i].b90 = 6;
        let (d1, d3) = (self.players[i].b.param_b, self.players[i].b.timer);
        self.player_layer_start(i, 0x18, d1);
        {
            let b = &mut self.players[i].b;
            b.w36 = (b.w36 & 0xFF00) | d3 as u16;
        }
        match self.players[i].item_type {
            0 => self.player_panim(i, 1, 9),
            0x201 | 0x202 => {
                let id = self.assets.player.t.u16(RUN_SWING_POSE, self.players[i].b.id as u32 * 2);
                self.player_panim(i, 5, id);
                self.players[i].weapon_atk = 1;
                self.player_set_item_a8(i, 1);
            }
            0x206 | 0x207 => {
                self.players[i].b.draw_flags = 0;
                self.player_walk_anim(i); // jsr 0x5CA6
                return self.player_weapon_attack_entry(i); // bra 0x198DC
            }
            _ => return self.player_weapon_attack_entry(i),
        }
        self.player_new_attack(i, 0x1A);
        self.players[i].combo = 0;
        self.player_tag_run_attack(i);
    }

    /// 0x1A548 (`+$90` 6): the run attack. With no direction the run ends (idle); the body moves on with its speed, the animation steps; at its end the run goes on
    /// (animation 0:0x19 at the layer's frame, the layer's timer, `+$90` 4).
    fn player_tag_run_attack(&mut self, i: usize) {
        if self.players[i].held & 0xF == 0 {
            return self.player_idle_entry(i);
        }
        self.player_pmove(i);
        if !self.player_pstep(i) {
            self.player_layer_tick(i);
            return;
        }
        let (d3, d1) = {
            let b = &mut self.players[i].b;
            b.draw_flags = 0;
            (b.w36 as u8, b.w34 as u8)
        };
        self.player_anim_at(i, 0x19, d1);
        self.players[i].b.timer = d3;
        self.players[i].b90 = 4;
        self.player_tag_run_step(i);
    }

    /// 0x1B0F6 (Hannah, animation 0x1B), 0x1B226 (Mustapha, 0x1A), 0x1B356 (Mess, 0x1C): the flying partner. `+$90`: 0 the blink starts, 2 follows the swing (the
    /// animation at the initiator's phase; Mess flips her facing on a frame with `+$2C` set) until the release frame, then the launch (vx 3 pixels a frame in the
    /// facing, vy 6.5, gravity -69, action 0x12, airborne); 4 the flight (the jump motion and the air control; attack starts the air attack, animation 1:0xD, attack
    /// 0x10); 6 the air attack (hit stop first). The landing is the normal one (0x1A81A, state 0x404).
    pub(super) fn player_tag_air(&mut self, i: usize, anim: u16) {
        let mut st = self.players[i].b90;
        if st == 0 {
            let p = &mut self.players[i];
            p.b.draw_flags = 0;
            p.respawn_t = 0xB4;
            p.b.live = 2;
            p.b90 += 2;
            st = 2;
        }
        if st == 2 {
            if self.tag_follow(i) {
                return self.player_idle_entry(i);
            }
            let Some(a) = self.tag_initiator(i) else { return self.player_idle_entry(i) };
            if self.players[a].b.param_a == 0 {
                let idx = self.players[a].b.param_b;
                self.player_anim_at(i, anim, idx);
                if anim == 0x1C && self.players[i].b.param_a != 0 {
                    self.players[i].b.facing ^= 1;
                }
                return;
            }
            let p = &mut self.players[i];
            p.b90 += 2;
            p.b.vx = if p.b.facing != 0 { 0x300 } else { -0x300 };
            p.b.ax = 0;
            p.b.vz = 0;
            p.b.az = 0;
            p.b.vy = 0x680;
            p.b.ay = -69;
            p.b83 = 0;
            p.action = 0x12;
            p.b.airborne = 1;
            st = 4;
        }
        if st == 4 {
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
            self.player_air_control(i);
            if self.players[i].held & ATK == 0 {
                return;
            }
            // 0x1B1BC: the air attack
            self.players[i].b90 += 2;
            self.players[i].b.draw_flags = 0;
            self.player_panim(i, 1, 0xD);
            self.player_new_attack(i, 0x10);
            self.players[i].combo = 0;
        }
        // 0x1B1E8
        if self.players[i].b.hitstop != 0 {
            self.players[i].b.hitstop -= 1;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn rom() -> Option<&'static AssetPack> {
        static A: std::sync::OnceLock<Option<&'static AssetPack>> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let path = std::path::Path::new("../../dino.zip");
            let path = if path.exists() { path } else { std::path::Path::new("../../../dino.zip") };
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// Two heroes in the level, standing: P1 Jack at (x1, z1), P2 Hannah at (x2, z2).
    fn pair(a: &'static AssetPack, x1: i16, z1: i16, x2: i16, z2: i16) -> Game {
        let mut g = Game::new(a);
        for (i, id, x, z) in [(0usize, 0u8, x1, z1), (1, 2, x2, z2)] {
            let p = &mut g.players[i];
            p.b.live = 1;
            p.b.who = i as u8;
            p.b.id = id;
            p.b.k20 = id as u16 * 4;
            p.b.mode = 2;
            p.b.set_xi(x);
            p.b.set_zi(z);
        }
        g.cam.s2.cam_x = 500; // Jack is more than 0xC0 right of the camera: he faces left
        g
    }

    /// The tie starts exactly when the counter has run down: 90 frames of stillness (the counters start at 0x5A once a check has reset them).
    fn frames_to_tie(g: &mut Game) -> Option<u32> {
        for p in &mut g.players[..2] {
            p.c2 = 0x5A;
            p.c3 = 0x5A;
        }
        for t in 1..=200 {
            g.tag_trigger();
            if g.players[0].bc1 != 0 || g.players[1].bc1 != 0 {
                return Some(t);
            }
        }
        None
    }

    /// D2c 15.1, 15.2: A.x - B.x from -32 to 31 and A.z - B.z from 0 to 15 start the tie up after 90 frames, else never; the one with the larger Z is the initiator.
    #[test]
    fn the_tie_up_needs_the_overlap_of_the_rules_and_90_frames() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        // (dx, dz of P1 to P2, a tie, the initiator): at +32 or Z -1 the offsets seen from P2 are -32 and +1, so P2 starts (D2c 15.2)
        for (dx, dz, tie, init) in [
            (0, 0, true, 0),
            (31, 0, true, 0),
            (-32, 0, true, 0),
            (32, 0, true, 1),
            (-33, 0, false, 0),
            (0, 15, true, 0),
            (0, 16, false, 0),
            (0, -1, true, 1),
            (0, -15, true, 1),
            (0, -16, false, 0),
        ] {
            let mut g = pair(a, 1000 + dx, 296 + dz, 1000, 296);
            let got = frames_to_tie(&mut g);
            assert_eq!(got.is_some(), tie, "dx {dx} dz {dz}");
            if tie {
                assert_eq!(got, Some(90), "dx {dx} dz {dz}");
                assert_eq!(g.players[init].b.state(), 0x0200_001A, "P{} is the initiator (dx {dx} dz {dz})", init + 1);
                assert_eq!((g.players[init].b.live, g.players[init].bc1, g.players[init].action, g.players[init].b.kind), (2, 1, 0x12, 0xC));
                let other = 1 - init;
                assert_eq!((g.players[other].bc1, g.players[other].action, g.players[other].b.kind), (1, 0x12, 0xE));
                // Jack as the initiator throws Hannah up (0x422); Hannah as the initiator makes Jack run (0x204)
                let want = if init == 0 { (2, 4) } else { (4, 0x22) };
                assert_eq!((g.players[other].b.act, g.players[other].b.step), want);
            }
        }
        // the deeper hero of the two starts: P2 two pixels deeper than P1
        let mut g = pair(a, 1000, 296, 1000, 298);
        assert_eq!(frames_to_tie(&mut g), Some(90));
        assert_eq!(g.players[1].b.state(), 0x0200_001A, "P2 is the initiator");
        assert_eq!((g.players[0].b.act, g.players[0].b.step), (4, 0x22), "Hannah's partner is thrown up (state 0x422)");
    }

    /// Input of the initiator resets both counters, an invulnerable initiator pauses them, an invulnerable or busy partner resets its counter.
    #[test]
    fn input_resets_the_counters_and_invulnerability_pauses_them() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let mut g = pair(a, 1000, 296, 1000, 296);
        g.players[0].c2 = 0x5A;
        g.players[0].c3 = 0x10;
        for _ in 0..30 {
            g.tag_trigger();
        }
        assert_eq!(g.players[0].c2, 0x5A - 30);
        g.players[0].held = 1;
        g.tag_trigger();
        assert_eq!((g.players[0].c2, g.players[0].c3), (0x5A, 0x5A), "any input of the initiator resets both");
        g.players[0].held = 0;
        g.tag_trigger();
        assert_eq!(g.players[0].c2, 0x59);
        g.players[0].b.live = 2; // invulnerable: the counters stay as they are
        g.tag_trigger();
        assert_eq!(g.players[0].c2, 0x59);
        g.players[0].b.live = 1;
        g.players[1].b.live = 2; // an invulnerable partner resets its counter (the initiator P2 of the pair is paused too)
        g.tag_trigger();
        assert_eq!(g.players[0].c2, 0x5A);
        g.players[1].b.live = 1;
        g.players[1].action = 3; // a partner in a move
        g.tag_trigger();
        assert_eq!(g.players[0].c2, 0x5A);
    }

    /// D2a 9.4: the follow table 0x69D4: Jack's partner hangs 32 pixels behind (the initiator facing left), 16 in front, 48 in front; one deeper; mirrored when the
    /// initiator faces right; and the tie is over (carry) when the initiator is gone.
    #[test]
    fn the_partner_follows_the_table_of_the_initiator() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let mut g = pair(a, 1000, 296, 1000, 296);
        assert!(frames_to_tie(&mut g).is_some());
        // Jack faces left here (his X is more than 0xC0 right of the camera): the partner is behind him, 32 pixels right
        assert_eq!(g.players[0].b.facing, 0);
        for (phase, dx) in [(0u8, 32i16), (1, -16), (2, -48)] {
            g.players[0].b.param_b = phase;
            assert!(!g.tag_follow(1));
            assert_eq!((g.players[1].b.xi(), g.players[1].b.zi()), (1000 + dx, 295), "phase {phase}");
        }
        g.players[0].b.facing = 1;
        g.players[0].b.param_b = 0;
        assert!(!g.tag_follow(1));
        assert_eq!(g.players[1].b.xi(), 1000 - 32, "mirrored for a facing right");
        g.players[0].b.live = 0;
        assert!(g.tag_follow(1), "the initiator is gone: the tie is over");
    }
}
