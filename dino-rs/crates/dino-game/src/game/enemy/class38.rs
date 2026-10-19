//! Mack, the triceratops, class 38 (handler 0x5341C..0x53BE4, 2,218 bytes): Episode 2 and Episode 5. D3b section 3, DS10.
//!
//! +$04 mode: 0 init (falls into mode 2 in the same frame), 2 alive, 4 and 6 free (0x53BDE). In mode 2 the sub state (+$05, table 0x53528)
//! is 0 the AI (0x53538), 2 the hit reaction (0x53A80), 4 and 6 nothing. The AI mode word +$A0 (table 0x5357C) is 0 the sleeper (never
//! scripted, variant 4), 2 the charge, 4 the calm flash, 6 the walk off. The charge states are +$06 = 0 .. 0xC (table 0x535E4).
//! Own bytes: +$80 timer, +$A4 "already angry", +$A5 / +$A6 / +$A7 colour flash (shared flash service), +$A8 / +$A9 stride timers,
//! +$AA passes left, +$AB sleeper Z timer, +$C0 hunt flag. The shared monster services (hit passes, flash, register) live in
//! `enemy.rs` and `enemy_combat.rs`.
//!
//! Owner: enemy port (M6-03).

use super::{off::*, Enemy};
use crate::game::{Game, Handle};

/// Tables read from the ROM at run time (`dino-extract` TABLES).
mod t {
    /// 0x53CBA: the bit mask of the jump roll (variant 2 only).
    pub const JUMP_MASK: u32 = 0x53CBA;
    /// 0x53CBC: slide speed and slide acceleration (words).
    pub const SLIDE: u32 = 0x53CBC;
    /// 0x53CC0: depth nudge speeds by the depth distance class (three words).
    pub const NUDGE: u32 = 0x53CC0;
}

const MODE_A0: usize = 0xA0;
const ANGRY: usize = 0xA4;
const STRIDE_T: usize = 0xA8;
const STILL_T: usize = 0xA9;
const PASSES: usize = 0xAA;
const Z_TIMER: usize = 0xAB;
const DUST_T: usize = 0xAC;
const HUNT: usize = 0xC0;

impl Game {
    /// 0x5341C: class 38. Mode 0 init, then (the same frame) mode 2; modes 4 and 6 free the object (0x53BDE).
    pub(super) fn enemy_class38(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.c38_init(e);
                self.c38_sub(e, h);
            }
            2 => self.c38_sub(e, h),
            _ => self.enemy_free_pool_a(e, i), // 0x53BDE: jmp 0x48FC
        }
    }

    /// 0x53430: the init. The side is chosen by the record X against the camera X + 0xC0: a record on the right makes Mack enter at
    /// camera + 0x1E0 facing left, else at camera - 0x60 facing right. Variant 4 is the sleeper (+$A0 = 0, +$C0 = 0xFF, palette 0x91 / 0x9B).
    fn c38_init(&mut self, e: &mut Enemy) {
        e.b.visible = 1; // 0x53430: move.b #1,$1(a6), so the first frame is drawn
        e.b.id = 0x11;
        e.m.sw(W84, 0x44);
        e.b.anim_table = 0x53BE4;
        self.enemy_set_hp(&mut e.b); // 0x31424
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        e.b.stun = 10;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.b.b72 = 0xC8;
        e.m.sb(B87, 0);
        e.m.sb(ANGRY, 0);
        e.m.sb(HUNT, 0);
        e.m.sb(0xC1, 0);
        self.enemy_set_speed(&mut e.b, 0x22); // 0x17A4: the gallop table
        e.m.sb(PASSES, e.m.b(B5B));
        e.m.sw(TARGET, self.enemy_pick_target(&e.b)); // 0x32032
        if e.b.var26 >> 8 == 4 {
            e.m.sb(HUNT, 0xFF);
            e.m.sw(MODE_A0, 0);
            e.b.palette_override = if self.w.stage == 4 { 0x9B } else { 0x91 };
        } else {
            let right = (self.cam.s2.cam_x.wrapping_add(0xC0)) < e.b.xi() as u16;
            let (facing, dx) = if right { (0, 0x1E0u16) } else { (1, 0xFFA0) };
            e.b.facing = facing;
            e.b.set_xi_keep(dx.wrapping_add(self.cam.s2.cam_x) as i16);
            e.m.sw(MODE_A0, 2);
            e.m.sb(ANGRY, 1);
        }
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sb(ATTACKING, 0xFF);
        self.enemy_anim(&mut e.b, 0);
        e.b.mode = e.b.mode.wrapping_add(2);
    }

    /// 0x5351C: the sub state dispatch of mode 2 (table 0x53528).
    fn c38_sub(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => self.c38_alive(e, h),
            2 => self.c38_hit(e, h),
            _ => {}
        }
    }

    /// 0x49C0: the draw list entry by the visible byte of the last 0x49CA.
    fn c38_draw(&mut self, e: &Enemy, h: Handle) {
        if e.b.visible != 0 {
            self.draw_list.push(((e.b.zi() as u16 as u32) << 16) | h as u32);
        }
    }

    /// 0x53538: the poise refill timer, the skip timer +$7A, the mode word, the floor (0x12E20), the hostile list (0x4C0A), the draw (0x49C0).
    fn c38_alive(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(POISE_T);
        if t != 0 {
            e.m.sb(POISE_T, t.wrapping_sub(1));
            if t == 1 {
                e.b.stun = 10;
            }
        }
        if e.m.b(WAIT) != 0 {
            e.m.sb(WAIT, e.m.b(WAIT).wrapping_sub(1));
        } else {
            match e.m.w(MODE_A0) {
                0 => self.c38_sleeper(e, h),
                2 => self.c38_charge(e, h),
                4 => self.c38_calm(e),
                6 => self.c38_walk_off(e),
                _ => {}
            }
            self.terrain_collide_free(&mut e.b); // 0x12E20
        }
        self.enemy_monster_register(h); // 0x4C0A
        self.c38_draw(e, h); // jmp 0x49C0
    }

    /// 0x53584: the sleeper (variant 4). State 0 lies down (animation 6); state 2 makes a Z mark (pool C class 30) every 0x32 + (rand & 0x3F)
    /// frames. Then the far test.
    fn c38_sleeper(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => {
                // 0x5359A
                e.m.sb(Z_TIMER, 0);
                e.b.act = e.b.act.wrapping_add(2);
                self.enemy_anim(&mut e.b, 6);
            }
            2 => {
                // 0x535AA
                let t = e.m.b(Z_TIMER);
                e.m.sb(Z_TIMER, t.wrapping_sub(1));
                if t == 0 {
                    self.enemy_monster_spawn_z(e, h, 0); // 0x9667E
                    let r = (self.rand() & 0x3F) as u8;
                    e.m.sb(Z_TIMER, r.wrapping_add(0x32));
                }
            }
            _ => {}
        }
        self.enemy_despawn_if_far(e, 0); // jmp 0x173C
    }

    /// 0xA09E: the roar 0x60 with the pan of the world X.
    fn c38_roar(&mut self, e: &Enemy) {
        self.snd_request(0x60, self.snd_pan_x(e.b.xi()) as u32, 0);
    }

    /// 0x53A66: the leave test. `+$5C` is the camera X at which Mack leaves (0 = never); true (carry) when the camera has reached it.
    fn c38_leave_test(&self, e: &Enemy) -> bool {
        let w = e.m.w(W5C);
        w != 0 && w <= self.cam.s2.cam_x
    }

    /// 0x539C4: the charge target. Facing left it is the player with the largest X (player 1 counts whenever he is in the game: d0 starts
    /// as Mack's own X, only the LOW BYTE of d0 is tested at the end, as the original does); facing right the smallest X. Nobody: Mack himself.
    fn c38_pick_target(&self, e: &Enemy, h: Handle) -> Handle {
        let live = |p: usize| self.players[p].b.live != 0;
        let px = |p: usize| self.players[p].b.xi() as u16;
        let mut a0 = h;
        if e.b.facing == 0 {
            let mut d0 = 0u16;
            if live(0) {
                d0 = e.b.xi() as u16;
                a0 = Game::player_handle(0);
            }
            for p in 1..3 {
                if live(p) && d0 < px(p) {
                    d0 = px(p);
                    a0 = Game::player_handle(p);
                }
            }
            if d0 as u8 == 0 {
                a0 = h;
            }
        } else {
            let mut d0 = 0xFFF0u16;
            if live(0) {
                d0 = px(0);
                a0 = Game::player_handle(0);
            }
            for p in 1..3 {
                if live(p) && d0 >= px(p) {
                    d0 = px(p);
                    a0 = Game::player_handle(p);
                }
            }
            if (d0 as u8) & 0x80 != 0 {
                a0 = h;
            }
        }
        a0
    }

    /// X and Z words of the target object (Mack himself is not in his slot while he runs).
    fn c38_target_xz(&self, e: &Enemy, h: Handle, target: Handle) -> (i16, i16) {
        if target == h {
            (e.b.xi(), e.b.zi())
        } else {
            let b = self.body(target);
            (b.xi(), b.zi())
        }
    }

    /// 0x538AE / 0x538B2: calm (`clear_hp` is the leave entry 0x538AE, which also sets HP 0): mode word 4, the colour flag flips, +$00 = 2
    /// (cannot be hit), the roar, the flash start, animation 1.
    fn c38_calm_enter(&mut self, e: &mut Enemy, clear_hp: bool) {
        if clear_hp {
            e.b.hp = 0;
        }
        e.m.sw(MODE_A0, 4);
        e.b.act = 0;
        e.b.step = 0;
        e.m.sb(ANGRY, e.m.b(ANGRY) ^ 1);
        e.b.live = 2;
        e.m.sb(HUNT, 0);
        self.c38_roar(e); // jsr 0xA09E
        self.enemy_monster_flash_start(e); // 0x53900
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x538DE: the calm flash (16 toggles of the palette). At its end: HP 0 walks off (0x5396E), else the charge again.
    fn c38_calm(&mut self, e: &mut Enemy) {
        if !self.enemy_monster_flash_tick(e) {
            self.enemy_step(&mut e.b); // jmp 0x121E
            return;
        }
        if e.b.hp == 0 {
            // 0x5396E: walk off to the left at 3 px a frame (animation 7).
            e.m.sw(MODE_A0, 6);
            e.b.live = 2;
            e.b.act = 0;
            e.b.step = 0;
            e.m.sb(HUNT, 0);
            e.b.facing = 0;
            e.b.vx = 0xFD00u16 as i16;
            e.b.ax = 0;
            e.b.vz = 0;
            e.b.az = 0;
            e.b.vy = 0;
            e.b.ay = 0;
            e.b.dir = 0;
            e.m.sb(STRIDE_T, 0);
            self.enemy_anim(&mut e.b, 7);
        } else {
            e.m.sw(MODE_A0, 2);
            e.b.act = 0;
            e.b.step = 0;
        }
    }

    /// 0x539B2: walking off: the animation, the motion, the far test.
    fn c38_walk_off(&mut self, e: &mut Enemy) {
        self.enemy_step(&mut e.b); // jsr 0x121E
        e.b.motion(); // jsr 0x189C
        self.enemy_despawn_if_far(e, 0); // jmp 0x173C
    }

    /// 0x535CE: the charge. The leave test first, then the state (+$06, table 0x535E4).
    fn c38_charge(&mut self, e: &mut Enemy, h: Handle) {
        if self.c38_leave_test(e) {
            return self.c38_calm_enter(e, true); // bcs 0x538AE
        }
        match e.b.act {
            0 => {
                // 0x535F2: wait 30 frames (animation 0).
                e.m.sb(HUNT, 0);
                e.m.sb(T80, 0x1E);
                e.b.act = e.b.act.wrapping_add(2);
                self.enemy_anim(&mut e.b, 0);
            }
            2 => {
                // 0x53608: after the wait: roar, head down (animation 1), wait 30 frames.
                if !self.c38_timer(e) {
                    return;
                }
                e.b.act = e.b.act.wrapping_add(2);
                e.m.sb(T80, 0x1E);
                self.c38_roar(e);
                self.enemy_anim(&mut e.b, 1);
            }
            4 => {
                // 0x5362A: pick the target, the charge attack (id 0x59, new serial), the gallop starts (animation 2).
                if !self.c38_timer(e) {
                    return;
                }
                let target = self.c38_pick_target(e, h);
                e.m.sw(TARGET, target);
                e.b.atk = 0x59;
                e.b.knock = 0;
                e.b.hitstop = 0;
                e.b.serial = self.w.attack_serial;
                self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
                e.b.vz = 0;
                e.b.az = 0;
                e.b.dir = 0;
                e.m.sb(STRIDE_T, 0);
                e.m.sb(STILL_T, 0);
                e.b.act = e.b.act.wrapping_add(2);
                e.m.sb(0x81, 4);
                self.enemy_anim(&mut e.b, 2);
            }
            6 => self.c38_gallop(e, h),
            8 => {
                // 0x5377E: the wait at the turn, then the charge target again.
                let old = e.m.b(T80);
                e.m.sb(T80, old.wrapping_sub(1));
                if old == 0 {
                    e.m.sb(T80, 0);
                    e.b.act = 4;
                }
            }
            0xA => {
                // 0x53792: the jump (variant 2) until he lands, then the screen shakes (G.$755C = 3).
                if e.b.airborne != 0 {
                    e.b.motion();
                    return;
                }
                self.cam_shake(3);
                e.b.vx = 0;
                e.b.ax = 0;
                e.b.vy = 0;
                e.b.ay = 0;
                e.m.sb(T80, 0x3C);
                e.b.act = e.b.act.wrapping_add(2);
                self.enemy_anim(&mut e.b, 4);
            }
            0xC => {
                // 0x537CA: the rest after the landing, then the charge again.
                if !self.c38_timer(e) {
                    return;
                }
                e.b.act = 6;
                self.enemy_anim(&mut e.b, 2);
            }
            _ => {}
        }
    }

    /// `subq.b #1,$80(a6); bcc`: false while the byte timer +$80 has not run out.
    fn c38_timer(&self, e: &mut Enemy) -> bool {
        let old = e.m.b(T80);
        e.m.sb(T80, old.wrapping_sub(1));
        old == 0
    }

    /// 0x5367A: state 6, the gallop. Variant 2 may jump (0x536A4); then the end of the pass (0x53724), the animation, the stride
    /// (0x537E2), the horn gate (0x53890) and the motion (0x189C).
    fn c38_gallop(&mut self, e: &mut Enemy, h: Handle) {
        let target = e.m.w(TARGET);
        if target != h {
            let (tx, _) = self.c38_target_xz(e, h, target);
            let mx = e.b.xi() as u16;
            // facing left tests "target left of Mack" (bcs), facing right "target not left of him".
            let ahead = if e.b.facing == 0 { (tx as u16) < mx } else { (tx as u16) >= mx };
            if ahead && e.b.var26 >> 8 == 2 && self.c38_try_jump(e, tx) {
                return;
            }
        }
        // 0x53724: the end of the pass.
        let cam = self.cam.s2.cam_x as i16;
        let outside = if e.b.facing == 0 {
            !(cam.wrapping_sub(0x70) <= e.b.xi())
        } else {
            !(cam.wrapping_add(0x1F0) >= e.b.xi())
        };
        if outside {
            e.b.facing ^= 1;
            e.b.act = e.b.act.wrapping_add(2);
            e.m.sb(T80, 0x78);
            let left = (e.m.b(PASSES).wrapping_sub(1)) as i8;
            e.m.sb(PASSES, left as u8);
            if left <= 0 {
                e.b.mode = 6; // move.w #$600,$4(a6): mode 6, sub 0 (the state bytes stay)
                e.b.sub = 0;
            }
        }
        self.enemy_step(&mut e.b); // 0x121E
        self.c38_stride(e, h, target);
        self.c38_horns(e);
        e.b.motion(); // jmp 0x189C
    }

    /// 0x536A4: the jump of variant 2 (unused in the game): when the target is within 0x50 px behind or 0x4F ahead a roll of 4 bits
    /// against the mask word 0x53CBA starts the leap (attack 0x5A).
    fn c38_try_jump(&mut self, e: &mut Enemy, tx: i16) -> bool {
        let d = tx.wrapping_sub(e.b.xi()).wrapping_add(0x50) as u16;
        if d >= 0xA0 {
            return false;
        }
        let r = (self.rand() & 0xF) as u32;
        let mask = self.assets.enemy.t.u16(t::JUMP_MASK, 0) as u32;
        if mask >> r & 1 == 0 {
            return false;
        }
        e.b.vx = if e.b.facing != 0 { 0x300 } else { -0x300 };
        e.b.vy = 0x720;
        e.b.ay = 0xFFBBu16 as i16;
        e.b.atk = 0x5A;
        e.b.knock = 0;
        e.b.hitstop = 0;
        e.b.serial = self.w.attack_serial;
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
        e.b.act = 0xA;
        e.b.airborne = 1;
        self.enemy_anim(&mut e.b, 3);
        true
    }

    /// 0x537E2: one stride. While +$A9 counts the stand still (20 frames after each stride), at its end the depth nudge: vz 0x20 / 0x40 /
    /// 0x80 by the depth distance (none within -8 .. +23), toward the target. Else the direction +$22 goes up every 2 frames (0x1F wraps) and
    /// vx is the absolute x of the speed table 0x22 entry, negative facing left. At `dir & 0xF == 0` the stand still starts.
    fn c38_stride(&mut self, e: &mut Enemy, h: Handle, target: Handle) {
        if e.m.b(STILL_T) != 0 {
            let mut vz = 0i16;
            let n = e.m.b(STILL_T).wrapping_sub(1);
            e.m.sb(STILL_T, n);
            if n == 0 {
                let (_, tz) = self.c38_target_xz(e, h, target);
                let dz = tz.wrapping_sub(e.b.zi());
                if (dz.wrapping_add(8) as u16) >= 0x20 {
                    let class = if dz.unsigned_abs() < 8 { 0 } else if dz.unsigned_abs() < 0x10 { 2 } else { 4 };
                    vz = self.assets.enemy.t.i16(t::NUDGE, class);
                    if (tz as u16) < e.b.zi() as u16 {
                        vz = vz.wrapping_neg();
                    }
                }
            }
            e.b.vz = vz;
            return;
        }
        let old = e.m.b(STRIDE_T);
        e.m.sb(STRIDE_T, old.wrapping_sub(1));
        if old != 0 {
            return;
        }
        e.m.sb(STRIDE_T, 1);
        e.b.dir = e.b.dir.wrapping_add(1) & 0x1F;
        if e.b.dir & 0xF == 0 {
            e.m.sb(STILL_T, 0x14);
        }
        let mut vx = self.assets.enemy.speed(e.b.speed_table, e.b.dir).0;
        if vx < 0 {
            vx = vx.wrapping_neg();
        }
        if e.b.facing == 0 {
            vx = vx.wrapping_neg();
        }
        e.b.vx = vx;
    }

    /// 0x53890: the horn box (+$49) stays only in the middle of a stride, `dir & 0xF` = 6 .. 10.
    fn c38_horns(&self, e: &mut Enemy) {
        let d = e.b.dir & 0xF;
        if !(6..=10).contains(&d) {
            e.b.attack_box = 0;
        }
    }

    /// 0x53A80: the hit reaction. The state (+$06): 0 the first hit frames, 2 and 4 the slide back (0x53AD8). The draw (0x49C0) is
    /// called once, twice when the slide runs (its own jmp returns into this one).
    fn c38_hit(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.act == 0 {
            // 0x53A9A
            if e.b.step == 0 {
                if e.b.hp != 0 && e.b.stun != 0 && e.b.airborne == 0 {
                    // The flinch: animation 5 for the hit stop frames, the HUD bar, poise already taken by the pass.
                    self.enemy_anim(&mut e.b, 5);
                    self.enemy_hud_hit(h, &e.b, 0); // 0x16B36
                    e.b.step = e.b.step.wrapping_add(2);
                    return self.c38_flinch_tick(e, h);
                }
                self.c38_slide(e, h);
                return self.c38_draw(e, h);
            }
            return self.c38_flinch_tick(e, h);
        }
        self.c38_slide(e, h);
        self.c38_draw(e, h); // 0x53A8E: the slide's own 0x49C0 returned here
    }

    /// 0x53ACE: the hit stop counter +$59 runs down, at 0 the recovery.
    fn c38_flinch_tick(&mut self, e: &mut Enemy, h: Handle) {
        e.b.hitstop = e.b.hitstop.wrapping_sub(1);
        if e.b.hitstop == 0 {
            self.c38_recover(e);
        }
        self.c38_draw(e, h);
    }

    /// 0x53AD8: the slide back (step 0 starts it, step 2 goes on) and its own draw (0x53AE4).
    fn c38_slide(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.step == 0 {
            // 0x53AEE: act 2, animation 3, speed 1280 and acceleration -32 away from the hero.
            e.b.act = 2;
            self.enemy_anim(&mut e.b, 3);
            let mut vx = self.assets.enemy.t.i16(t::SLIDE, 0);
            let mut ax = self.assets.enemy.t.i16(t::SLIDE, 2);
            if e.b.facing != 0 {
                vx = vx.wrapping_neg();
                ax = ax.wrapping_neg();
            }
            e.b.vx = vx;
            e.b.ax = ax;
            e.b.vz = 0;
            e.b.az = 0;
            e.m.sb(DUST_T, 0);
            self.enemy_hud_hit(h, &e.b, 0); // 0x16B36
            e.b.step = e.b.step.wrapping_add(2);
        }
        self.c38_dust(e); // 0x53B96
        self.combat.push(0x6AC2, h); // 0x4B58
        e.b.motion(); // 0x189C
        self.terrain_collide_free(&mut e.b); // 0x12E20
        // `tst.w $24(a6)`: facing and the draw flags together. Zero: the slide is over when vx has turned negative;
        // else when it has turned non-negative.
        let word = ((e.b.facing as u16) << 8) | e.b.draw_flags as u16;
        if (word == 0 && e.b.vx < 0) || (word != 0 && e.b.vx >= 0) {
            self.c38_recover(e);
        }
        self.c38_draw(e, h); // 0x53AE4: jmp 0x49C0
    }

    /// 0x53B96: a dust puff (pool C class 7, variant 1, 0x92EA4) every 6 frames behind the sliding body.
    fn c38_dust(&mut self, e: &mut Enemy) {
        let old = e.m.b(DUST_T);
        e.m.sb(DUST_T, old.wrapping_sub(1));
        if old != 0 {
            return;
        }
        e.m.sb(DUST_T, 5);
        let r = (self.rand() & 0x1F) as i16 - 0x10;
        let back = if e.b.facing != 0 { 0x10 } else { -0x10 };
        self.fx_spawn_dust_at(r.wrapping_add(back).wrapping_add(e.b.xi()), e.b.yi(), e.b.zi(), 1);
    }

    /// 0x53B64: the recovery: poise 10 after 60 frames (+$79), the sub state back to the AI, the charge restarts (state 0). With HP 0 or a
    /// Mack who was not angry (the sleeper) the calm flash follows (0x538B2).
    fn c38_recover(&mut self, e: &mut Enemy) {
        e.b.stun = 10;
        e.m.sb(POISE_T, 0x3C);
        e.b.sub = 0;
        e.b.act = 0;
        e.b.step = 0;
        e.b.ax = 0;
        if e.b.hp == 0 {
            return self.c38_calm_enter(e, false);
        }
        e.b.live = 1;
        if e.m.b(ANGRY) == 0 {
            self.c38_calm_enter(e, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game() -> Game {
        Game::new(Box::leak(Box::new(AssetPack::default())))
    }

    /// 0x539C4: facing left player 1 counts whenever he is in the game and d0 starts as MACK's own X, so only later players with a larger X replace him;
    /// the end test reads the low byte of d0 only: zero gives Mack himself (a quirk of the original).
    #[test]
    fn the_charge_target_follows_the_low_byte_quirk() {
        let mut g = game();
        let h = 0xD7D4;
        let mut e = Enemy::default();
        e.b.set_xi(0x321);
        e.b.facing = 0;
        g.players[0].b.live = 1;
        g.players[0].b.set_xi(0x100);
        assert_eq!(g.c38_pick_target(&e, h), Game::player_handle(0));
        g.players[1].b.live = 1;
        g.players[1].b.set_xi(0x401);
        assert_eq!(g.c38_pick_target(&e, h), Game::player_handle(1));
        e.b.set_xi(0x300); // d0 = 0x300 with player 1 only: its low byte is 0
        g.players[1].b.live = 0;
        assert_eq!(g.c38_pick_target(&e, h), h);
        // nobody in the game: Mack himself, facing either way
        g.players[0].b.live = 0;
        assert_eq!(g.c38_pick_target(&e, h), h);
        e.b.facing = 1;
        assert_eq!(g.c38_pick_target(&e, h), h);
    }

    /// 0x53A66: a record X of 0 never leaves, else the camera X reaching it does.
    #[test]
    fn the_leave_test_needs_a_record_x_and_the_camera() {
        let mut g = game();
        let mut e = Enemy::default();
        g.cam.s2.cam_x = 0x480;
        assert!(!g.c38_leave_test(&e));
        e.m.sw(W5C, 0x480);
        assert!(g.c38_leave_test(&e));
        g.cam.s2.cam_x = 0x47F;
        assert!(!g.c38_leave_test(&e));
    }
}
