//! Dr. Fessenden, form 1: pool A class 94 (handler 0x67D2C..0x6995E), name 0x25, the Morgue animation table 0x6EA48. A 90 percent copy of Morgue
//! (class 40) with its own gate and action tables (0x74A42, 0x74A82, `boss_common.rs`): a 276 frame human intro (state 0xE), then the green dino
//! man who headbutts up close, runs in and shoulder rushes in the middle and roars and charges from far away. No helpers, no 0xACEE; at its
//! death the growth scene (`+$04` 4, 0x6967E) and then class 84 in its place (0x698E8). D3d 6.1, 6.2, BT14, BT15. Owner: enemy port.
//!
//! Not reached by any code and left out: the states 2, 4, 6, 0xA, 0xC, 0x10, 0x16, 0x18, 0x1A, 0x24 (their set-up routines 0x6822C, 0x68B72,
//! 0x68CA0, 0x68D08, 0x68D92, 0x68F7E, 0x6903E have no caller) and the escape of the phase timer (0x6800C, states 0x28 and 0x2A: it needs
//! G.-$703E, which the init clears and nothing sets). The palette animation 0x67F92 has no caller either.

use super::boss_common::{BossAi, BossRecover};
use super::{off::*, Enemy};
use crate::game::{Game, Handle};

/// Own bytes (the original offsets).
mod o {
    /// `+$A0` byte: 1 aggressive, 0 defensive. `+$A1` byte: busy. `+$A2` / `+$A3`: the step delay of the walk and its reload flag. `+$A5`: the
    /// target's X class (0x329B8). `+$A8` byte: a short counter; `+$A9` byte: the direction before a turn (0x68F38). `+$AA`, `+$AE` bytes.
    pub const PHASE: usize = 0xA0;
    pub const BUSY: usize = 0xA1;
    pub const DELAY: usize = 0xA2;
    pub const RELOAD: usize = 0xA3;
    pub const XCLASS: usize = 0xA5;
    pub const A8: usize = 0xA8;
    pub const OLD_DIR: usize = 0xA9;
    pub const AA: usize = 0xAA;
    pub const TIMER: usize = 0xAC;
    pub const WAKE: usize = 0xAE;
    /// `+$B0` / `+$B4` longs: the X / Z steps of the walk (16.16). `+$B9` byte: a headbutt is due. `+$BE` byte: the planned direction.
    pub const VX: usize = 0xB0;
    pub const VZ: usize = 0xB4;
    pub const B9: usize = 0xB9;
    pub const BE: usize = 0xBE;
    /// `+$C0` word: the swings left of a flurry; `+$C2` word: the flurry kind (0..3).
    pub const SWINGS: usize = 0xC0;
    pub const KIND: usize = 0xC2;
}

/// The tables of the handler (read at run time through `enemy.t`; dino-extract/src/boss.rs).
mod t {
    pub const ROAR_LIMIT: u32 = 0x68222;
    pub const REMAP: u32 = 0x68714;
    pub const STEP_TIME: u32 = 0x68E96;
    pub const GRID_ROW: u32 = 0x68EB6;
    pub const FLURRY_ANIM: u32 = 0x691AC;
    pub const HEADBUTT_WAIT: u32 = 0x6923C;
    pub const FLURRY_WAIT_A: u32 = 0x692C8;
    pub const FLURRY_WAIT_B: u32 = 0x69354;
    pub const LEVEL_DELAY: u32 = 0x74BC2;
    pub const SWING_WAIT: u32 = 0x74DA2;
    pub const SWING_LEVEL: u32 = 0x74F62;
    /// Morgue's direction table (0x53034, bytes by the grid cells of the target and the object).
    pub const GRID_DIR: u32 = 0x53034;
}

impl Game {
    /// 0x67D2C (mode table 0x67D3A): 0 the init 0x67D42, 2 the run 0x67E08, 4 the death scene 0x6967E, 6 form 2 (0x698E8).
    pub(super) fn enemy_class94(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => self.c94_init(e, h),
            2 => self.c94_run(e, h),
            4 => self.c94_dying(e, h),
            _ => self.c94_form2(e, h, i),
        }
    }

    /// 0x67D42: mode 2, name 0x25, `+$84` 0x94, the table 0x6EA48, `+$63` 2, animation 0xB, the box table 0x103C00, speed id 7, direction 0x18;
    /// HP row 8 of 0x31E14; the target; poise 11; `+$72` 0xC8; `+$AC` 0x14; aggressive with the phase timer 1; G.-$703E 0; the floor; `+$7D`
    /// 0xFE; a slot (0x32C5E); the HUD bar; G.$4FC 0; G.-$7048 = HP; then the human intro (0x6901C): busy, `+$AC` 0x1E, state 0xE, animation
    /// 0x14, speed id 5.
    fn c94_init(&mut self, e: &mut Enemy, h: Handle) {
        e.b.mode = 2;
        e.m.sb(B87, 0);
        e.b.id = 0x25;
        e.m.sw(W84, 0x94);
        e.b.anim_table = 0x6EA48;
        e.b.b63 = 2;
        self.enemy_anim(&mut e.b, 0xB);
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_set_speed(&mut e.b, 7);
        e.b.dir = 0x18;
        self.enemy_set_boss_hp(&mut e.b, 8); // 0x31E14 row 8
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        e.b.stun = 0x0B;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.m.sb(ATTACKING, 0);
        e.b.b72 = 0xC8;
        e.m.sw(o::TIMER, 0x14);
        e.m.sb(0xBC, 0);
        e.m.sb(WAIT, 0);
        e.m.sb(o::B9, 0);
        e.m.sw(0xA6, 1);
        e.m.sb(o::PHASE, 1);
        e.m.sb(o::AA, 0);
        e.m.sb(o::WAKE, 0);
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sb(ATTACKING, 0xFE);
        self.lib_slot6_new(e); // 0x32C5E
        self.hud.boss = h; // G.-$70BC
        self.spawn.g.b4fc = 0;
        self.spawn.g.fess_hp = e.b.hp; // G.-$7048
        // 0x6901C
        e.m.sb(o::BUSY, 1);
        e.m.sw(o::TIMER, 0x1E);
        e.b.act = 0x0E;
        self.enemy_anim(&mut e.b, 0x14);
        self.enemy_set_speed(&mut e.b, 5);
    }

    /// 0x67E08: an act past 0xE in the death sub is reset; a state past 0x02020200 (a hit in progress) sets `+$AE`; X is kept in camera
    /// X - 0x50 .. + 0x1D0 (with HP 0 in - 0x10 .. + 0x190); by `+$05` (the long table 0x67EC2): 0 the AI 0x67ED2, 2 the hit 0x6965C, 4 the
    /// death 0x6967E, 6 held (0x2B35A).
    fn c94_run(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.sub == 4 && (e.b.act as i8) > 0xE {
            e.b.act = 0;
        }
        let state = u32::from_be_bytes([e.b.mode, e.b.sub, e.b.act, e.b.step]) as i32;
        if state > 0x0202_0200 {
            e.m.sb(o::WAKE, 1);
        }
        let cam = self.cam.s2.cam_x as i16;
        let put = |b: &mut crate::game::body::Body, x: i16| b.x = (b.x & 0xFFFF) | ((x as i32) << 16);
        let (lo, hi) = (cam.wrapping_sub(0x50), cam.wrapping_add(0x1D0));
        if e.b.xi().wrapping_sub(lo) < 0 {
            put(&mut e.b, lo);
        }
        if hi.wrapping_sub(e.b.xi()) < 0 {
            put(&mut e.b, hi);
        }
        if e.b.hp == 0 {
            let (lo, hi) = (cam.wrapping_sub(0x10), cam.wrapping_add(0x190));
            if e.b.xi().wrapping_sub(lo) < 0 {
                put(&mut e.b, lo);
            }
            if hi.wrapping_sub(e.b.xi()) < 0 {
                put(&mut e.b, hi);
            }
        }
        match e.b.sub {
            0 => self.c94_alive(e, h),
            2 => self.c94_hit(e, h),
            4 => self.c94_dying(e, h),
            _ => self.enemy_grabbed(e, h),
        }
    }

    /// 0x67ED2: the poise refill (11), the terrain (0x12CB4); the hit stop (`+$2F` = 1) or the wait `+$7A`; else the four-slot point (0x32D0C),
    /// face the target when not busy (0x32B68), the state of `+$06` (table 0x67F5E), the gate test (0x68164), the push flag (0x681A6), the roar
    /// test (0x681C8) and the phase timer (0x67FF0); then the target check, the draw check and the hurtable list.
    fn c94_alive(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(POISE_T);
        if t != 0 {
            e.m.sb(POISE_T, t - 1);
            if t == 1 {
                e.b.stun = 0x0B;
            }
        }
        if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
            e.m.sw(OBSTACLE, ob);
        }
        if e.b.hitstop != 0 {
            e.b.hitstop -= 1;
            e.b.timer = 1;
        } else if e.m.b(WAIT) != 0 {
            e.m.sb(WAIT, e.m.b(WAIT) - 1);
        } else {
            self.boss_slot4_keep(e); // 0x32D0C
            if e.m.b(o::BUSY) == 0 {
                self.lib_face_target(e);
            }
            self.c94_state(e, h);
            self.c94_gate_test(e, h); // 0x68164
            if e.m.b(o::BUSY) == 0 && e.m.b(o::PHASE) != 0 && e.m.w(o::TIMER) == 0x10 {
                self.spawn.g.b502 = 1; // 0x681A6
            }
            self.c94_roar_test(e, h); // 0x681C8
            self.boss_phase_tick(e, BossAi::Fessenden); // 0x67FF0 (its escape needs G.-$703E, never set)
        }
        self.lib_check_target(e); // 0x32F56
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// `+$AC` down by one; true at 0.
    fn c94_count(e: &mut Enemy) -> bool {
        let t = e.m.w(o::TIMER).wrapping_sub(1);
        e.m.sw(o::TIMER, t);
        t == 0
    }

    /// 0x67F5E: the state routine of `+$06` (only the states that the code sets).
    fn c94_state(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0x08 => self.c94_walk_to_point(e, h),
            0x0E => {
                // 0x68794: the human intro; at its end (0x687A0) G.$7B4 = 2, the music 0x0F, G.$4FC = 0 and the decision.
                if self.enemy_step(&mut e.b) {
                    self.cam.s2.lock78 = 2; // G.$7B4
                    self.sound(0x0F); // 0xA63C, d1 / d2 left from the animation step
                    self.spawn.g.b4fc = 0;
                    self.c94_decide(e, h);
                }
            }
            0x12 => {
                // 0x687D2
                if self.enemy_step(&mut e.b) {
                    self.c94_decide(e, h);
                }
            }
            0x14 => self.c94_wander(e, h),
            0x1C => {
                // 0x688AC: the run in, then the shoulder rush.
                if self.enemy_step(&mut e.b) {
                    self.c94_rush(e, 0x1A, 0x2C, 0x900, -0x30);
                }
            }
            0x1E => self.c94_slide(e, h, 0x12, false),
            0x20 => {
                // 0x689CC: the charge: at the end or within 1 px of the target's X the stop (0x68C48).
                let tx = self.lib_target(e).xi();
                if Self::c94_count(e) || (tx.wrapping_add(1).wrapping_sub(e.b.xi()) as u16) <= 2 {
                    return self.c94_charge_stop(e);
                }
                self.enemy_move_dir(&mut e.b); // 0x1862
                self.enemy_step(&mut e.b);
            }
            0x22 => {
                // 0x68866
                if Self::c94_count(e) {
                    self.c94_headbutt_now(e);
                }
            }
            0x26 => self.c94_roar(e, h),
            0x2C => self.c94_slide(e, h, 0x1A, true),
            0x2E => {
                // 0x68870
                if Self::c94_count(e) {
                    self.c94_flurry_first(e);
                }
            }
            0x30 => {
                // 0x6887A: the swing; at its end state 0x32 (0x68AC6).
                if self.enemy_step(&mut e.b) {
                    self.c94_swing_end(e);
                }
            }
            0x32 => {
                // 0x68886
                if Self::c94_count(e) {
                    return self.c94_flurry_next(e, h);
                }
                self.enemy_step(&mut e.b);
            }
            _ => {}
        }
    }

    /// 0x68510 (state 8): toward the point (`+$88`, `+$8A`) at 1.3 / 1 px per frame (the steps set every 4 frames), with the step delay `+$A2`
    /// (0x680C0); at the end of `+$AC` a due headbutt (`+$B9`, 0x691E6) or face and the decision; at the point (8 / 7 round it) animation 0,
    /// `+$A3` 1, `+$A2` 0.
    fn c94_walk_to_point(&mut self, e: &mut Enemy, h: Handle) {
        if Self::c94_count(e) {
            if e.m.b(o::B9) != 0 {
                return self.c94_headbutt(e);
            }
            self.lib_face_target(e);
            return self.c94_decide(e, h);
        }
        if e.m.w(o::TIMER) & 3 == 0 {
            self.c94_point_steps(e);
        }
        let d = e.m.b(o::DELAY);
        if d != 0 {
            e.m.sb(o::DELAY, d - 1);
            return;
        }
        let mut moved = false;
        if ((e.m.w(DEST_X) as i16).wrapping_add(8).wrapping_sub(e.b.xi()) as u16) >= 0x10 {
            e.b.x = e.b.x.wrapping_add(e.m.l(o::VX) as i32);
            moved = true;
        }
        if ((e.m.w(DEST_Z) as i16).wrapping_add(7).wrapping_sub(e.b.zi()) as u16) >= 0xA {
            e.b.z = e.b.z.wrapping_add(e.m.l(o::VZ) as i32);
            moved = true;
        }
        if moved {
            // 0x680C0: the delay from 0x74BC2 by level (+ the variant's top bits) while `+$A3` is set.
            if e.m.b(o::RELOAD) != 0 {
                let i = self.w.level.wrapping_add((((e.b.var26 >> 8) as u8 as i8) >> 5) as i16 as u16);
                e.m.sb(o::DELAY, self.assets.enemy.t.u8(t::LEVEL_DELAY, i as u32));
                e.m.sb(o::RELOAD, 0);
            }
            if e.m.b(o::DELAY) == 0 {
                self.enemy_step(&mut e.b);
            }
            return;
        }
        if (e.m.b(o::DELAY) as i8) < 5 {
            self.enemy_anim(&mut e.b, 0);
        }
        e.m.sb(o::RELOAD, 1);
        e.m.sb(o::DELAY, 0);
    }

    /// 0x68536 / 0x68EF6: the walk steps toward the point: X +-0x15000, Z +-0x10000.
    fn c94_point_steps(&self, e: &mut Enemy) {
        let vx: u32 = if e.m.w(DEST_X) as i16 > e.b.xi() { 0x15000 } else { 0xFFFE_B000 };
        e.m.sl(o::VX, vx);
        let vz: u32 = if e.m.w(DEST_Z) as i16 > e.b.zi() { 0x10000 } else { 0xFFFF_0000 };
        e.m.sl(o::VZ, vz);
    }

    /// 0x68466 (state 0x14): the wander round the arena: at the end of `+$AC` the decision; every 0x1E frames a new direction from the grid
    /// (0x68E12 part, with the turn limit 0x68F38); face, move along the direction (0x1862), animate.
    fn c94_wander(&mut self, e: &mut Enemy, h: Handle) {
        if Self::c94_count(e) {
            return self.c94_decide(e, h);
        }
        let n = e.m.b(o::A8).wrapping_sub(1);
        e.m.sb(o::A8, n);
        if n == 0 {
            e.m.sb(o::A8, 0x1E);
            e.m.sb(o::OLD_DIR, e.b.dir);
            self.c94_grid_dir(e, true);
            self.c94_turn_limit(e);
        }
        self.lib_face_target(e);
        self.enemy_move_dir(&mut e.b); // 0x1862
        self.enemy_step(&mut e.b);
    }

    /// 0x68486 / 0x68E18: the planned direction `+$BE` from Morgue's grid table 0x53034 (the row by the target's cell, 0x68EB6, plus the
    /// object's cell, 0x686AE), times 4; it becomes `+$22` (the old one in `+$A9`); left of the camera direction 8, right of camera X + 0x180
    /// direction 0x18. The remap 0x686F6 (values 8 and up at random) works on d1: in state 0x14 (`remap`) that is the table value; the wander's
    /// start (0x68E3E) reads the value into d2, so there 0x686F6 only draws its random number when the object's cell - 1 is above 7.
    fn c94_grid_dir(&mut self, e: &mut Enemy, remap: bool) {
        let tgt = self.lib_target(e).clone();
        let row = Self::c94_cell(tgt.xi(), tgt.zi()).wrapping_sub(1);
        let mut d3 = self.assets.enemy.t.u16(t::GRID_ROW, (row as u32).wrapping_mul(2));
        let own = Self::c94_cell(e.b.xi(), e.b.zi()).wrapping_sub(1);
        d3 = d3.wrapping_add(own);
        let mut d = self.assets.enemy.t.u8(t::GRID_DIR, d3 as i16 as i32 as u32);
        let d1 = if remap { d } else { own as u8 };
        if (d1 as i8) > 7 {
            // 0x686F6
            let r = (self.rand() & 0xF) as u8;
            let k = (d1.wrapping_sub(8) << 4).wrapping_add(r);
            let v = self.assets.enemy.t.u8(t::REMAP, k as u32);
            if remap {
                d = v;
            }
        }
        let dir = (d.wrapping_mul(4)) & 0x1F;
        e.m.sb(o::BE, dir);
        e.m.sb(o::OLD_DIR, e.b.dir);
        e.b.dir = dir;
        let cam = self.cam.s2.cam_x as i16;
        if e.b.xi().wrapping_sub(cam) < 0 {
            e.b.dir = 8;
        }
        if cam.wrapping_add(0x180).wrapping_sub(e.b.xi()) < 0 {
            e.b.dir = 0x18;
        }
    }

    /// 0x686AE: the arena cell (1..18) of a point: the Z band (Z - 0x100 up to 0x1A: 13, up to 0x34: 7, else 1) plus one per 0x40 px of X past
    /// 0x1160 (at most 5).
    fn c94_cell(x: i16, z: i16) -> u16 {
        let dz = z.wrapping_sub(0x100);
        let mut d1: u16 = if dz <= 0x1A {
            0xD
        } else if dz <= 0x34 {
            7
        } else {
            1
        };
        let mut d0 = x.wrapping_sub(0x1120);
        let mut d2 = 0;
        loop {
            d0 = d0.wrapping_sub(0x40);
            if d0 < 0 {
                break;
            }
            d1 += 1;
            d2 += 1;
            if d2 == 5 {
                break;
            }
        }
        d1
    }

    /// 0x68F38: the turn limit: from the old direction `+$A9` the new one may turn at most 2 steps (of 32) each way.
    fn c94_turn_limit(&self, e: &mut Enemy) {
        let old = e.m.b(o::OLD_DIR);
        let d0 = (e.b.dir.wrapping_sub(old) & 0x1F).wrapping_add(2) & 0x1F;
        if (d0 as i8) <= 4 {
            return;
        }
        if (d0.wrapping_sub(2) as i8) <= 0x10 {
            e.b.dir = old.wrapping_add(2) & 0x1F;
        } else {
            e.b.dir = old.wrapping_sub(2) & 0x1F;
        }
    }

    /// 0x688B8 / 0x68922 (states 0x1E and 0x2C): the slide after a charge or a shoulder rush: at the end of `+$AC` `+$AA` 0 and the decision;
    /// in the last 0x10 frames attack 0x42 again (each frame); in the last 5 animation 0; above 0xE frames left the slide (0x189C), else the
    /// animation `anim` (0x12 or 0x1A); animate.
    fn c94_slide(&mut self, e: &mut Enemy, h: Handle, anim: u16, _rush: bool) {
        if Self::c94_count(e) {
            e.m.sb(o::AA, 0);
            return self.c94_decide(e, h);
        }
        let t = e.m.w(o::TIMER) as i16;
        if t < 0x10 {
            self.enemy_begin_attack(&mut e.b, 0x42);
        }
        if t < 5 {
            self.enemy_anim(&mut e.b, 0);
        }
        if t > 0xE {
            e.b.motion(); // 0x189C
        } else {
            self.enemy_anim(&mut e.b, anim);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x68BE6 (the shoulder rush, after the run in) and 0x68B8C: animation, busy, state, `+$AC` 0x30, no Y or Z speed, vx and the slow down
    /// ax (negated when facing left), attack 0x42.
    fn c94_rush(&mut self, e: &mut Enemy, anim: u16, state: u8, vx: i16, ax: i16) {
        self.enemy_anim(&mut e.b, anim);
        e.m.sb(o::BUSY, 1);
        e.b.act = state;
        e.m.sw(o::TIMER, 0x30);
        e.b.vz = 0;
        e.b.az = 0;
        e.b.vy = 0;
        e.b.ay = 0;
        e.b.vx = vx;
        e.b.ax = ax;
        if e.b.facing == 0 {
            e.b.vx = e.b.vx.wrapping_neg();
            e.b.ax = e.b.ax.wrapping_neg();
        }
        self.enemy_begin_attack(&mut e.b, 0x42);
    }

    /// 0x68C48: the end of the charge: animation 0xD, busy, state 0x1E, `+$AC` 0x30, no Y or Z speed, the X speed kept, ax -0x20 (0x20 when
    /// facing left), attack 0x42.
    fn c94_charge_stop(&mut self, e: &mut Enemy) {
        self.enemy_anim(&mut e.b, 0xD);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x1E;
        e.m.sw(o::TIMER, 0x30);
        e.b.vz = 0;
        e.b.az = 0;
        e.b.vy = 0;
        e.b.ay = 0;
        e.b.ax = -0x20;
        if e.b.facing == 0 {
            e.b.ax = 0x20;
        }
        self.enemy_begin_attack(&mut e.b, 0x42);
    }

    /// 0x687F2 (state 0x26): the roar: every 8 frames a spark (class 75 variant 0x13) 0x30 higher, in front by 0x28 - `+$AC`; at the end the
    /// charge (0x69428).
    fn c94_roar(&mut self, e: &mut Enemy, _h: Handle) {
        if Self::c94_count(e) {
            return self.c94_charge(e);
        }
        let t = e.m.w(o::TIMER);
        if t & 7 == 0 {
            let d = 0x28i16.wrapping_sub(t as i16);
            let x = if e.b.facing == 0 { e.b.xi().wrapping_add(d) } else { e.b.xi().wrapping_sub(d) };
            let (y, z) = (e.b.yi().wrapping_sub(0x30), e.b.zi().wrapping_sub(1));
            self.c75_spawn(75 * 4, 0x13, x, y, z, Some(e.b.facing));
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x69428: the charge: face, speed id 0x24, attack 0x41, busy, state 0x20, `+$AC` 0x1E, animation 7.
    fn c94_charge(&mut self, e: &mut Enemy) {
        self.lib_face_target(e);
        self.enemy_set_speed(&mut e.b, 0x24);
        self.enemy_begin_attack(&mut e.b, 0x41);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x20;
        e.m.sw(o::TIMER, 0x1E);
        self.enemy_anim(&mut e.b, 7);
    }

    /// 0x68AC6: the end of a swing: busy, state 0x32; `+$AC` from 0x74DA2 (`rand() & 0x3E`) plus 0x74F62 by level, then 1 in any case.
    fn c94_swing_end(&mut self, e: &mut Enemy) {
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x32;
        let r = (self.rand() & 0x3E) as u32;
        let tb = &self.assets.enemy.t;
        let v = tb.u16(t::SWING_WAIT, r).wrapping_add(tb.u16(t::SWING_LEVEL, (self.w.level as u32).wrapping_mul(2)));
        e.m.sw(o::TIMER, v);
        e.m.sw(o::TIMER, 1); // 0x68B0A: overwritten in both branches
    }

    /// 0x69128: the next swing of a flurry: `+$C0` down (none left: the decision); face, attack 0x4B (0x6C but the last), busy, state 0x30,
    /// `+$AC` 0x3C, `+$B9` 0, aim (0x32A58); the animation of 0x691AC by `+$C2` and `+$C0` (0x5A: the run in, 0x691CC).
    fn c94_flurry_next(&mut self, e: &mut Enemy, h: Handle) {
        let n = e.m.w(o::SWINGS).wrapping_sub(1);
        e.m.sw(o::SWINGS, n);
        if n == 0 {
            return self.c94_decide(e, h);
        }
        self.lib_face_target(e);
        self.enemy_begin_attack(&mut e.b, 0x4B);
        if n != 1 {
            self.enemy_begin_attack(&mut e.b, 0x6C);
        }
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x30;
        e.m.sw(o::TIMER, 0x3C);
        e.m.sb(o::B9, 0);
        self.lib_aim_at_target(e); // 0x32A58
        let i = (e.m.w(o::KIND) << 2).wrapping_add(n).wrapping_mul(2);
        let anim = self.assets.enemy.t.u16(t::FLURRY_ANIM, i as u32);
        if anim == 0x5A {
            // 0x691CC
            e.m.sb(o::BUSY, 1);
            e.b.act = 0x1C;
            e.m.sw(o::TIMER, 0x1E);
            return self.enemy_anim(&mut e.b, 0x1C);
        }
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x690E0: the first swing of a flurry: attack 0x6C, busy, state 0x30, `+$AC` 0x3C, `+$B9` 0, aim, animation 4 (0x1B when `+$C2` is set).
    fn c94_flurry_first(&mut self, e: &mut Enemy) {
        self.enemy_begin_attack(&mut e.b, 0x6C);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x30;
        e.m.sw(o::TIMER, 0x3C);
        e.m.sb(o::B9, 0);
        self.lib_aim_at_target(e);
        let anim = if e.m.w(o::KIND) != 0 { 0x1B } else { 4 };
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x690A2: the headbutt strikes: attack 0x40, busy, state 0x12, `+$AC` 0x3C, `+$B9` 0, aim, animation 4.
    fn c94_headbutt_now(&mut self, e: &mut Enemy) {
        self.enemy_begin_attack(&mut e.b, 0x40);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x12;
        e.m.sw(o::TIMER, 0x3C);
        e.m.sb(o::B9, 0);
        self.lib_aim_at_target(e);
        self.enemy_anim(&mut e.b, 4);
    }

    /// 0x691E6 (actions 0 and 1): the headbutt's wind-up: face, attack 0x40, busy, state 0x22, `+$AC` from 0x6923C; a due headbutt (`+$B9`)
    /// strikes at once (0x690A2); else `+$B9` 0, aim, animation 4.
    fn c94_headbutt(&mut self, e: &mut Enemy) {
        self.lib_face_target(e);
        self.enemy_begin_attack(&mut e.b, 0x40);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x22;
        let r = (self.rand() & 0x1E) as u32;
        e.m.sw(o::TIMER, self.assets.enemy.t.u16(t::HEADBUTT_WAIT, r));
        if e.m.b(o::B9) != 0 {
            return self.c94_headbutt_now(e);
        }
        e.m.sb(o::B9, 0);
        self.lib_aim_at_target(e);
        self.enemy_anim(&mut e.b, 4);
    }

    /// 0x6926C (actions 7 and 10, `+$C2` 0 / 3: attack 0x6C, animation 4) and 0x692F8 (actions 8 and 9, `+$C2` 1 / 2: attack 0x40, animation
    /// 0x1B): a flurry's wind-up: face, busy, state 0x2E, `+$C0` 3, `+$AC` from 0x692C8 / 0x69354; a due headbutt strikes at once.
    fn c94_flurry(&mut self, e: &mut Enemy, kind: u16) {
        e.m.sw(o::KIND, kind);
        let (atk, table, anim) = if kind == 0 || kind == 3 {
            (0x6C, t::FLURRY_WAIT_A, 4)
        } else {
            (0x40, t::FLURRY_WAIT_B, 0x1B)
        };
        self.lib_face_target(e);
        self.enemy_begin_attack(&mut e.b, atk);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x2E;
        e.m.sw(o::SWINGS, 3);
        let r = (self.rand() & 0x1E) as u32;
        e.m.sw(o::TIMER, self.assets.enemy.t.u16(table, r));
        if e.m.b(o::B9) != 0 {
            return self.c94_headbutt_now(e);
        }
        e.m.sb(o::B9, 0);
        self.lib_aim_at_target(e);
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x69374: the shoulder from the recover entry: face, attack 0x40, busy, `+$AC` 1, `+$B9` 0, state 0x12, aim, animation 0x19.
    fn c94_shoulder(&mut self, e: &mut Enemy) {
        self.lib_face_target(e);
        self.enemy_begin_attack(&mut e.b, 0x40);
        e.m.sb(o::BUSY, 1);
        e.m.sw(o::TIMER, 1);
        e.m.sb(o::B9, 0);
        e.b.act = 0x12;
        self.lib_aim_at_target(e);
        self.enemy_anim(&mut e.b, 0x19);
    }

    /// 0x693B8 (action 3, the swing: attack 0x2B, animation 8) and 0x693F0 (action 4, the flurry of slaps: attack 0x31, animation 0xC): busy,
    /// state 0x12, `+$AC` 0x3C, aim.
    fn c94_strike(&mut self, e: &mut Enemy, atk: u8, anim: u16) {
        self.enemy_begin_attack(&mut e.b, atk);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x12;
        e.m.sw(o::TIMER, 0x3C);
        self.lib_aim_at_target(e);
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x68B12 (action 5): the run in: busy, state 0x1C, `+$AC` 0x1E; the direction of 0x32AAC (the target's X and its Y as Z) with the turn
    /// limit; facing by the direction; face; sound 0x60; animation 0xF.
    fn c94_run_in(&mut self, e: &mut Enemy) {
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x1C;
        e.m.sw(o::TIMER, 0x1E);
        e.m.sb(o::OLD_DIR, e.b.dir);
        let tgt = self.lib_target(e);
        let (tx, ty) = (tgt.xi() as u16, tgt.yi() as u16);
        e.b.dir = Self::enemy_angle_to(&e.b, tx, ty); // 0x32AAC
        self.c94_turn_limit(e);
        e.b.facing = if e.b.dir > 0x10 { 0 } else { 1 };
        self.lib_face_target(e);
        self.sound_ex(0x60, 0x71010, 0x0800_0000);
        self.enemy_anim(&mut e.b, 0xF);
    }

    /// 0x68FC2 (action 6): the roar, only with the target more than 0x80 px away in X (else the decision): busy, `+$AC` 0x28, state 0x26,
    /// animation 0x12, face, aim (0x32A58), speed id 5.
    fn c94_roar_start(&mut self, e: &mut Enemy, h: Handle) {
        let tx = self.lib_target(e).xi();
        let x = e.b.xi();
        let near = if tx >= x { tx.wrapping_sub(0x80) < x } else { tx.wrapping_add(0x80) > x };
        if near {
            return self.c94_decide(e, h);
        }
        e.m.sb(o::BUSY, 1);
        e.m.sw(o::TIMER, 0x28);
        e.b.act = 0x26;
        self.enemy_anim(&mut e.b, 0x12);
        self.lib_face_target(e);
        self.lib_aim_at_target(e);
        self.enemy_set_speed(&mut e.b, 5);
    }

    /// 0x69468 (action 2): a headbutt becomes due (`+$B9` 1, animation 5) unless one already is; then the walk to the point (0x68EE4).
    fn c94_due(&mut self, e: &mut Enemy) {
        if e.m.b(o::B9) == 0 {
            e.m.sb(o::B9, 1);
            self.enemy_anim(&mut e.b, 5);
        }
        self.c94_walk_start(e);
    }

    /// 0x68EE4: the walk to the point: not busy, `+$AC` 0x3C, state 8, the steps (0x68EF6).
    fn c94_walk_start(&mut self, e: &mut Enemy) {
        e.m.sb(o::BUSY, 0);
        e.m.sw(o::TIMER, 0x3C);
        e.b.act = 8;
        self.c94_point_steps(e);
    }

    /// 0x6956C: the decision (nothing in state 0xA): `+$AC` 0x3C, speed id 5, not busy, the X class (0x329B8); by the phase flag (table
    /// 0x695AA) the wander (defensive, 0x68DD0) or the walk to the point (aggressive, 0x68EDC: animation 1 first).
    pub(super) fn c94_decide(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.act == 0x0A {
            return;
        }
        e.m.sw(o::TIMER, 0x3C);
        self.enemy_set_speed(&mut e.b, 5);
        e.m.sb(o::BUSY, 0);
        let (xc, _, _, _) = self.c4_classes(e);
        e.m.sb(o::XCLASS, xc as u8);
        if e.m.b(o::PHASE) == 0 {
            return self.c94_wander_start(e, h);
        }
        self.enemy_anim(&mut e.b, 1);
        self.c94_walk_start(e);
    }

    /// 0x68DD0: the wander's start: aim 0x30 beside the target (0x32A88), face, speed id 6, not busy, `+$A8` 6, `+$AC` from 0x68E96, state
    /// 0x14, the grid direction (without the turn limit), animation 1.
    fn c94_wander_start(&mut self, e: &mut Enemy, _h: Handle) {
        e.m.sb(o::OLD_DIR, e.b.dir);
        self.lib_aim_beside_target(e, 0x30);
        self.lib_face_target(e);
        self.enemy_set_speed(&mut e.b, 6);
        e.m.sb(o::BUSY, 0);
        e.m.sb(o::A8, 6);
        let r = (self.rand() & 0x1E) as u32;
        e.m.sw(o::TIMER, self.assets.enemy.t.u16(t::STEP_TIME, r));
        e.b.act = 0x14;
        e.m.sb(o::OLD_DIR, e.b.dir);
        self.c94_grid_dir(e, false);
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x68164: not busy and aggressive, with the target within 0xA of Z (window +7): the gate (0x695B0).
    fn c94_gate_test(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.b(o::BUSY) != 0 || e.m.b(o::PHASE) == 0 {
            return;
        }
        let tz = self.lib_target(e).zi();
        if (tz.wrapping_add(7).wrapping_sub(e.b.zi()) as u16) <= 0xA {
            self.c94_gate(e, h);
        }
    }

    /// 0x695B0: the gate `rand & 31 < 0x74A42[aggression][+$A5]` (0x695B4); passed: face, speed id 5, not busy, the X class (0x329B8), the
    /// action of 0x74A82 (a due headbutt allows only action 1) by the jump table 0x69638.
    fn c94_gate(&mut self, e: &mut Enemy, h: Handle) {
        if !self.boss_aggression_gate(e, BossAi::Fessenden) {
            return;
        }
        self.lib_face_target(e);
        self.enemy_set_speed(&mut e.b, 5);
        e.m.sb(o::BUSY, 0);
        let (xc, _, _, _) = self.c4_classes(e);
        e.m.sb(o::XCLASS, xc as u8);
        let action = self.boss_action_pick(e, BossAi::Fessenden);
        if e.m.b(o::B9) != 0 && action != 1 {
            return;
        }
        match action {
            0 | 1 => self.c94_headbutt(e),
            2 => self.c94_due(e),
            3 => self.c94_strike(e, 0x2B, 8),
            4 => self.c94_strike(e, 0x31, 0xC),
            5 => self.c94_run_in(e),
            6 => self.c94_roar_start(e, h),
            7 => self.c94_flurry(e, 0),
            8 => self.c94_flurry(e, 1),
            9 => self.c94_flurry(e, 2),
            _ => self.c94_flurry(e, 3),
        }
    }

    /// 0x681C8: aggressive, not busy and no step delay: `rand & 31` at most 2 (1 below a quarter of the start HP G.-$7048) with the target more
    /// than 0x100 px away in X: the roar (0x68FC2).
    fn c94_roar_test(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.b(o::PHASE) == 0 || e.m.b(o::BUSY) != 0 || e.m.b(o::DELAY) != 0 {
            return;
        }
        let quarter = (self.spawn.g.fess_hp as i16) >> 2;
        let d2 = if (e.b.hp as i16).wrapping_sub(quarter) < 0 { 0 } else { 2 };
        let limit = self.assets.enemy.t.u16(t::ROAR_LIMIT, d2);
        let r = self.rand() & 0x1F;
        if r > limit {
            return;
        }
        let tx = self.lib_target(e).xi();
        if (tx.wrapping_add(0x100).wrapping_sub(e.b.xi()) as u16) > 0x200 {
            self.c94_roar_start(e, h);
        }
    }

    /// 0x682A2: the recover entry of `+$84` 0x94 (table 0x2B56C): the shared part (`boss_recover`), then the shoulder (0x69374) or the
    /// decision.
    pub(super) fn c94_recover(&mut self, e: &mut Enemy, h: Handle) {
        match self.boss_recover(e, BossAi::Fessenden) {
            BossRecover::Shoulder => self.c94_shoulder(e),
            _ => self.c94_decide(e, h),
        }
    }

    /// 0x6965C: the hit: the credit (0x3262A); at HP 0 the slow motion (G.$4D6 = 3) and `+$6E` 0 (no 0xACEE); then 0x2A606.
    fn c94_hit(&mut self, e: &mut Enemy, h: Handle) {
        self.boss_credit(e);
        if e.b.hp == 0 {
            self.flow.speed = 3;
            if e.b.hp_prev != 0 {
                e.b.hp_prev = 0;
            }
        }
        self.enemy_hit_reaction(e, h);
    }

    // ------------------------------------------------------------------ the death scene (M8-28)

    /// 0x6967E by `+$05` (table 0x6968A): the growth scene. 0: normal speed, the portrait 4 with the line 0x3D, sound 0x22, 240 frames (0x6969E);
    /// 2: the wait, then the line 0x3E and the walk to the arena centre (0x696E4); 4: the walk (0x69748); 6: the original never counts here
    /// (0x6978C); 8: the score, animation 0x15 and sound 0x83 (0x697A4); 0xA: animation 0x18 (0x697D4); 0xC, 0xE, 0x10: the lines 0x40, 0x41,
    /// 0x42 with the animations 0x1D, 0x18, 0x18; 0x12: the box cleared and mode 6 (form 2).
    fn c94_dying(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 | 2 => {
                if e.b.sub == 0 {
                    // 0x6969E
                    self.flow.speed = 1; // G.$4D6
                    self.c94_caption(e, 0x3D);
                    self.sound(0x22);
                    e.m.sw(T80, 0xF0);
                    e.b.sub = 2;
                }
                // 0x696D2
                let t = e.m.w(T80);
                if t != 0 {
                    e.m.sw(T80, t - 1);
                    return self.enemy_draw_check(&mut e.b, h);
                }
                self.c94_caption_clear();
                self.c94_caption(e, 0x3E);
                e.b.facing = 1;
                e.b.dir = 8;
                if e.b.xi().wrapping_sub(0x1120) >= 0xC0 {
                    e.b.facing = 0;
                    e.b.dir = 0x18;
                }
                e.b.sub = 4;
                self.enemy_set_speed(&mut e.b, 5);
                e.m.sw(T80, 0x3C);
                self.enemy_anim(&mut e.b, 0x16);
                self.c94_walk_centre(e, h);
            }
            4 => self.c94_walk_centre(e, h),
            6 => {
                // 0x6978C: `+$80` is read but never stored back, so this step only ends when it is 0.
                if e.m.w(T80) == 0 {
                    e.b.sub = 8;
                    return self.c94_stagger(e, h);
                }
                self.enemy_step(&mut e.b);
                self.enemy_draw_check(&mut e.b, h);
            }
            8 => self.c94_stagger(e, h),
            0x0A => self.c94_grow(e, h, 0x18, None),
            0x0C => self.c94_grow(e, h, 0x1D, Some(0x40)),
            0x0E => self.c94_grow(e, h, 0x18, Some(0x41)),
            0x10 => self.c94_grow(e, h, 0x18, Some(0x42)),
            _ => {
                // 0x698C2
                if self.enemy_step(&mut e.b) {
                    self.c94_caption_clear();
                    e.b.mode = 6;
                }
                self.enemy_draw_check(&mut e.b, h);
            }
        }
    }

    /// 0x69748: the walk to the arena centre: X - 0x1120 inside 0x40 .. 0x140 the stagger (sub 6 skipped, 0x697A4); else `+$80` down (at its
    /// end 180 frames and sub 6), animate, move along the direction (0x1862).
    fn c94_walk_centre(&mut self, e: &mut Enemy, h: Handle) {
        let d = e.b.xi().wrapping_sub(0x1120);
        if d > 0x40 && d < 0x140 {
            e.b.sub = 6;
            return self.c94_stagger(e, h);
        }
        let t = e.m.w(T80);
        e.m.sw(T80, t.wrapping_sub(1));
        if t == 0 {
            // 0x69782
            e.m.sw(T80, 0xB4);
            e.b.sub = 6;
            self.enemy_step(&mut e.b);
            return self.enemy_draw_check(&mut e.b, h);
        }
        self.enemy_step(&mut e.b);
        self.enemy_move_dir(&mut e.b); // 0x1862
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x697A0 / 0x697A4: `+$80` 0x78, sub + 2, the kill score (0x325E6, 8,000), animation 0x15, sound 0x83; then 0x697D4.
    fn c94_stagger(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sw(T80, 0x78);
        e.b.sub = e.b.sub.wrapping_add(2);
        let b = e.b.clone();
        self.enemy_award_kill_score(&b);
        self.enemy_anim(&mut e.b, 0x15);
        self.sound(0x83);
        self.c94_grow(e, h, 0x18, None);
    }

    /// 0x697D4 / 0x697F6 / 0x6983A / 0x6987E: animate; at the end the next animation, sub + 2 and (with a line) the box cleared and the
    /// portrait 4 with that line.
    fn c94_grow(&mut self, e: &mut Enemy, h: Handle, anim: u16, line: Option<i16>) {
        if self.enemy_step(&mut e.b) {
            if line.is_some() {
                self.c94_caption_clear();
            }
            self.enemy_anim(&mut e.b, anim);
            e.b.sub = e.b.sub.wrapping_add(2);
            if let Some(l) = line {
                self.c94_caption(e, l);
            }
        }
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x84E2E (row 0) and 0x84736: the doctor's portrait (4) and the tall font line `line` at scroll1 0x90A29C; 0x84736 keeps the portrait's
    /// palette word (0x8461A) in `+$A2` of the caller.
    fn c94_caption(&mut self, e: &mut Enemy, line: i16) {
        self.prop_b50_palette(0);
        self.text_portrait(0x90A29C, 4, line);
        e.m.sw(0xA2, self.assets.frontend.text.u16(0x8461A, 2 * 4));
    }

    /// 0x687B6: 45 columns of 13 cells from 0x90A29C filled with spaces (code 0x20, attribute 0).
    fn c94_caption_clear(&mut self) {
        for col in 0..45u32 {
            for row in 0..13u32 {
                self.hud_put(0x90A29C + col * 0x80 + row * 4, 0x20, 0);
            }
        }
    }

    /// 0x698E8 (mode 6): form 2, class 84 variant 0 at this place with this facing and direction (0x483C; the HUD bar follows it); then the draw
    /// check and the free (0x48FC).
    fn c94_form2(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if let Some(j) = self.enemies.alloc() {
            let nh = self.enemies.handle(j);
            let n = &mut self.enemies.slots[j];
            n.b.live = 1;
            n.b.k20 = 84 * 4;
            n.b.var26 = 0;
            n.b.facing = e.b.facing;
            n.b.dir = e.b.dir;
            let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
            put(&mut n.b.x, e.b.xi());
            put(&mut n.b.y, e.b.yi());
            put(&mut n.b.z, e.b.zi());
            self.hud.boss = nh;
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_free_pool_a(e, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game() -> Game {
        static A: std::sync::OnceLock<AssetPack> = std::sync::OnceLock::new();
        Game::new(A.get_or_init(|| {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dino.zip");
            dino_extract::extract(&path).expect("Fessenden checks require the user's dino.zip")
        }))
    }

    /// D3d 6.2, BT-A13: at level 10 form 1 has 450 / 550 / 650 HP by player count; the init leaves the human intro (state 0xE) running and
    /// keeps the start HP in G.-$7048.
    #[test]
    fn hp_by_player_count_and_intro() {
        for (players, hp) in [(0u8, 450u16), (1, 550), (2, 650)] {
            let mut g = game();
            g.w.level = 10;
            g.w.players_minus_one = players;
            let mut e = Enemy::default();
            e.b.k20 = 94 * 4;
            g.enemy_class94(&mut e, 0, 0);
            assert_eq!(e.b.hp, hp, "{} players", players + 1);
            assert_eq!(g.spawn.g.fess_hp, hp);
            assert_eq!((e.b.mode, e.b.act, e.m.b(ATTACKING)), (2, 0x0E, 0xFE));
        }
    }

    /// The kill score of 0x325E6 for the name 0x25 is 8,000 (the death scene awards it twice, M f94-trans).
    #[test]
    fn kill_score() {
        let g = game();
        let t = &g.assets.enemy.t;
        assert_eq!(t.u32(0x10292, t.u8(0x325F8, 0x25) as u32), 0x8000);
    }

    /// 0x686AE: the arena cells (Z bands 13 / 7 / 1, one more per 0x40 px past 0x1160, at most 5).
    #[test]
    fn arena_cells() {
        assert_eq!(Game::c94_cell(0x1100, 0x110), 0xD);
        assert_eq!(Game::c94_cell(0x1160, 0x130), 8);
        assert_eq!(Game::c94_cell(0x1400, 0x160), 6);
    }
}
