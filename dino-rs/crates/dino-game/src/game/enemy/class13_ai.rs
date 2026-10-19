//! Class 13: the state handlers of `+$06` (table 0x3E1AA, 40 words, states 0..0x4E), the two decisions (0x4005C when a state ends,
//! 0x40118 / 0x40156 the far decision) and the state entries they call (0x3F15E .. 0x3F5E2, 0x3F670, 0x3FC60). The attack states
//! (rifle, dash, melee, kick) and their entries are in `class13_attack.rs`; the entrances and the alive frame in `class13.rs`.
//! `bra 0x4005C` of the original is `enemy_c13_4005c` (the decision).

use super::class13::{f::*, t};
use super::{Enemy, off::*};
use crate::game::{Game, Handle};

impl Game {
    // ------------------------------------------------------------------ the state table

    /// 0x3E14E: the handler of the state `+$06` (word table 0x3E1AA). The states that nothing enters (0, 0x1A, 0x20, 0x22, 0x2E, 0x30, 0x32,
    /// 0x3A, 0x40) point into code that is data (state 0, 0x22 and 0x2E) or have no entry in the live code.
    pub(super) fn enemy_c13_state(&mut self, e: &mut Enemy, h: Handle) {
        let _ = h;
        match e.b.act {
            0x02 => self.enemy_c13_3e54c(e),
            0x04 => self.enemy_c13_3e558(e),
            0x06 => self.enemy_c13_3e78a(e),
            0x08 => self.enemy_c13_3e828(e),
            0x0A => self.enemy_c13_3e6b4(e),
            0x0C => self.enemy_c13_3e8c6(e),
            0x0E => self.enemy_c13_3e576(e),
            0x10 => self.enemy_c13_3ea2a(e),
            0x12 => self.enemy_c13_3eafa(e),
            0x14 | 0x18 | 0x3C | 0x46 => self.enemy_c13_3eb8c(e),
            0x16 => self.enemy_c13_3eba4(e),
            0x1C => self.enemy_c13_3ec22(e),
            0x1E => self.enemy_c13_3eca8(e),
            0x24 => self.enemy_c13_3eb4c(e),
            0x26 => self.enemy_c13_3ee04(e),
            0x28 => self.enemy_c13_3ee2c(e),
            0x2A => self.enemy_c13_3e98a(e),
            0x2C => self.enemy_c13_3eb7e(e),
            0x34 => self.enemy_c13_3f038(e),
            0x36 => self.enemy_c13_3ef56(e),
            0x38 => self.enemy_c13_3f058(e),
            0x3E => self.enemy_c13_3f0a4(e),
            0x42 => self.enemy_c13_3eb3e(e),
            0x44 => self.enemy_c13_3eb30(e),
            0x48 => self.enemy_c13_3eb66(e),
            0x4A => self.enemy_c13_3e602(e),
            0x4C => self.enemy_c13_3e64c(e),
            0x4E => self.enemy_c13_3f076(e),
            s => panic!("class 13 state {s:#x} (no live handler in the table 0x3E1AA)"),
        }
    }

    /// `subq.w #1,$AC(a6)`: true when the result is zero (the `beq` / `bne` of the original).
    fn enemy_c13_tick_timer(e: &mut Enemy) -> bool {
        let t = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, t);
        t == 0
    }

    /// `subq.b #1,$A8(a6)`: true when the result is zero.
    fn enemy_c13_tick_aim(e: &mut Enemy) -> bool {
        let t = e.m.b(AIM_T).wrapping_sub(1);
        e.m.sb(AIM_T, t);
        t == 0
    }

    /// 0x3E54C (state 2): wait out `+$7A`.
    fn enemy_c13_3e54c(&mut self, e: &mut Enemy) {
        let w = e.m.b(WAIT);
        if w != 0 {
            e.m.sb(WAIT, w - 1);
        }
    }

    /// 0x3E558 (state 4): walk `+$AC` frames facing the target, then decide.
    fn enemy_c13_3e558(&mut self, e: &mut Enemy) {
        if Self::enemy_c13_tick_timer(e) {
            return self.enemy_c13_4005c(e);
        }
        self.lib_face_target(e);
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x3E576 (state 0xE): approach the slot at speed id 0xD, re-aim every 30 frames, ends early within 0xC0 px of the target in X.
    fn enemy_c13_3e576(&mut self, e: &mut Enemy) {
        if Self::enemy_c13_tick_timer(e) {
            return self.enemy_c13_4005c(e);
        }
        if Self::enemy_c13_tick_aim(e) {
            e.m.sb(AIM_T, 0x1E);
            let tx = self.lib_target(e).xi();
            let d0 = tx.wrapping_add(0x60).wrapping_sub(e.b.xi()) as u16;
            if d0 <= 0xC0 {
                return self.enemy_c13_4005c(e);
            }
            e.m.sb(OLD_DIR, e.b.dir);
            self.lib_aim_at_target(e); // 0x32A70
            self.enemy_c13_3f32e(e);
        }
        e.b.facing = 1;
        if (e.b.dir as i8) > 0x10 {
            e.b.facing = 0;
        }
        let d0 = (e.m.w(DEST_X) as i16).wrapping_add(4).wrapping_sub(e.b.xi()) as u16;
        if d0 <= 8 {
            let d0 = (e.m.w(DEST_Z) as i16).wrapping_add(4).wrapping_sub(e.b.zi()) as u16;
            if d0 < 8 {
                return self.enemy_c13_4005c(e);
            }
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x3E602 (state 0x4A): run back onto the screen at speed id 0x1C toward the slot point (re-aimed every 12 frames); after `+$AC`
    /// frames 0x3F464.
    fn enemy_c13_3e602(&mut self, e: &mut Enemy) {
        if Self::enemy_c13_tick_timer(e) {
            return self.enemy_c13_3f464(e);
        }
        if Self::enemy_c13_tick_aim(e) {
            e.m.sb(AIM_T, 0xC);
            e.m.sb(OLD_DIR, e.b.dir);
            self.lib_aim_at_target(e);
            self.enemy_c13_3f32e(e);
        }
        e.b.facing = 1;
        if (e.b.dir as i8) > 0x10 {
            e.b.facing = 0;
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x3E64C (state 0x4C): slide toward the target by 2 px in X and 1.5 px in Z a frame (no speed table); after `+$AC` frames 0x3F3D8.
    fn enemy_c13_3e64c(&mut self, e: &mut Enemy) {
        if Self::enemy_c13_tick_timer(e) {
            return self.enemy_c13_3f3d8(e);
        }
        let t = self.lib_target(e);
        let (tx, tz) = (t.xi(), t.zi());
        if tx > e.b.xi() {
            e.b.x = e.b.x.wrapping_add(0x2_0000);
        } else {
            e.b.x = e.b.x.wrapping_add(0xFFFE_0000u32 as i32);
        }
        if tz > e.b.zi() {
            e.b.z = e.b.z.wrapping_add(0x1_8000);
        } else {
            e.b.z = e.b.z.wrapping_add(0xFFFE_8000u32 as i32);
        }
        e.b.facing = 1;
        if (e.b.dir as i8) > 0x10 {
            e.b.facing = 0;
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x3E6B4 (state 0xA): shuffle toward the point (+$88 / +$8A) at the steps +$B0 / +$B4 (chosen every 4 frames), pausing (`+$A2`)
    /// after a step; X and Z move together.
    fn enemy_c13_3e6b4(&mut self, e: &mut Enemy) {
        if Self::enemy_c13_tick_timer(e) {
            return self.enemy_c13_4005c(e);
        }
        if e.m.w(TIMER) & 3 == 0 {
            self.enemy_c13_shuffle_steps(e, 0x1_2000, 0xFFFE_E000);
        }
        let p = e.m.b(STILL_T);
        if p != 0 {
            e.m.sb(STILL_T, p - 1);
            return;
        }
        let mut moved = false;
        let d0 = (e.m.w(DEST_X) as i16).wrapping_add(8).wrapping_sub(e.b.xi()) as u16;
        if d0 >= 0x10 {
            e.b.x = e.b.x.wrapping_add(e.m.l(VX) as i32);
            moved = true;
        }
        let d0 = (e.m.w(DEST_Z) as i16).wrapping_add(0xA).wrapping_sub(e.b.zi()) as u16;
        if d0 >= 0xB {
            e.b.z = e.b.z.wrapping_add(e.m.l(VZ) as i32);
            moved = true;
        }
        if !moved {
            e.m.sb(ARRIVED, 1); // 0x3E77A
            e.m.sb(STILL_T, 0);
            return;
        }
        self.enemy_c13_3e4dc(e);
        if e.m.b(STILL_T) != 0 {
            return;
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x3E6C8..0x3E716: the X step `+$B0` (`right` when the point is to the right, biased signed compare) and the Z step `+$B4`
    /// (+0x10000 when the point is deeper than the object, else -0x10000).
    fn enemy_c13_shuffle_steps(&mut self, e: &mut Enemy, right: u32, left: u32) {
        let to_right = (e.m.w(DEST_X) as i16).wrapping_add(0x400) > e.b.xi().wrapping_add(0x400);
        e.m.sl(VX, if to_right { right } else { left });
        let deeper = (e.m.w(DEST_Z) as i16) > e.b.zi();
        e.m.sl(VZ, if deeper { 0x1_0000 } else { 0xFFFF_0000 });
    }

    /// 0x3E78A (state 6): drift in X toward the point at the speed `+$C0` (sign by the side), re-signed every 4 frames, pausing.
    fn enemy_c13_3e78a(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 0);
        if Self::enemy_c13_tick_timer(e) {
            return self.enemy_c13_4005c(e);
        }
        if e.m.w(TIMER) & 3 == 0 {
            let mut v = e.m.l(SPEED);
            e.m.sl(VX, v);
            let to_right = (e.m.w(DEST_X) as i16).wrapping_add(0x400) > e.b.xi().wrapping_add(0x400);
            if !to_right {
                v = v.wrapping_neg();
                e.m.sl(VX, v);
            }
        }
        let p = e.m.b(STILL_T);
        if p != 0 {
            e.m.sb(STILL_T, p - 1);
            return;
        }
        let d0 = (e.m.w(DEST_X) as i16).wrapping_add(8).wrapping_sub(e.b.xi()) as u16;
        if d0 >= 0x10 {
            self.enemy_c13_3e4dc(e);
            if e.m.b(STILL_T) != 0 {
                return;
            }
            e.b.x = e.b.x.wrapping_add(e.m.l(VX) as i32);
            self.enemy_step(&mut e.b);
            return self.lib_face_target(e);
        }
        e.m.sb(ARRIVED, 1); // 0x3E814
        e.m.sb(STILL_T, 0);
        self.lib_face_target(e);
    }

    /// 0x3E828 (state 8): the same in Z (the step +$B4 is chosen every 4 frames: +0x12000 deeper, -0x12000 nearer).
    fn enemy_c13_3e828(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 0);
        if Self::enemy_c13_tick_timer(e) {
            return self.enemy_c13_4005c(e);
        }
        if e.m.w(TIMER) & 3 == 0 {
            let deeper = (e.m.w(DEST_Z) as i16) > e.b.zi();
            e.m.sl(VZ, if deeper { 0x1_2000 } else { 0xFFFE_E000 });
        }
        let p = e.m.b(STILL_T);
        if p != 0 {
            e.m.sb(STILL_T, p - 1);
            return;
        }
        let d0 = (e.m.w(DEST_Z) as i16).wrapping_add(8).wrapping_sub(e.b.zi()) as u16;
        if d0 >= 0x10 {
            self.enemy_c13_3e4dc(e);
            if e.m.b(STILL_T) != 0 {
                return;
            }
            e.b.z = e.b.z.wrapping_add(e.m.l(VZ) as i32);
            self.enemy_step(&mut e.b);
            return self.lib_face_target(e);
        }
        e.m.sb(ARRIVED, 1); // 0x3E8B2
        e.m.sb(STILL_T, 0);
        self.lib_face_target(e);
    }

    /// 0x3E8C6 (state 0xC): walk at speed id 0xF toward the target, re-aimed every 15 frames (turn limit 1 step).
    fn enemy_c13_3e8c6(&mut self, e: &mut Enemy) {
        if Self::enemy_c13_tick_timer(e) {
            return self.enemy_c13_4005c(e);
        }
        if Self::enemy_c13_tick_aim(e) {
            e.m.sb(AIM_T, 0xF);
            e.m.sb(OLD_DIR, e.b.dir);
            self.lib_aim_at_target(e);
            self.enemy_c13_3f2e8(e);
        }
        let p = e.m.b(STILL_T);
        if p != 0 {
            e.m.sb(STILL_T, p - 1);
            return;
        }
        let d0 = (e.m.w(DEST_X) as i16).wrapping_add(4).wrapping_sub(e.b.xi()) as u16;
        let arrived = d0 <= 8 && {
            let d0 = (e.m.w(DEST_Z) as i16).wrapping_add(4).wrapping_sub(e.b.zi()) as u16;
            d0 < 8
        };
        if !arrived {
            self.enemy_c13_3e4dc(e); // 0x3E926
            self.enemy_move_dir(&mut e.b);
            self.enemy_step(&mut e.b);
            return self.lib_face_target(e);
        }
        e.m.sb(ARRIVED, 1); // 0x3E93C
        e.m.sb(STILL_T, 0);
        self.lib_face_target(e);
    }

    /// 0x3E98A (state 0x2A): flank: walk to a point 0x40 px beside the target on the side of the object (aim 0x32A88 every 6 frames).
    fn enemy_c13_3e98a(&mut self, e: &mut Enemy) {
        if Self::enemy_c13_tick_timer(e) {
            return self.enemy_c13_4005c(e);
        }
        if (e.m.w(TIMER) as i16) <= 0x3C {
            e.m.sb(BUSY, 0);
        }
        if Self::enemy_c13_tick_aim(e) {
            e.m.sb(AIM_T, 6);
            e.m.sb(OLD_DIR, e.b.dir);
            self.lib_aim_beside_target(e, 0x40); // 0x32A88
        }
        let p = e.m.b(STILL_T);
        if p != 0 {
            e.m.sb(STILL_T, p - 1);
            return;
        }
        let t = self.lib_target(e);
        let (tx, tz) = (t.xi(), t.zi());
        let d1: i16 = if tx > e.b.xi() { -0x40 } else { 0x40 };
        let d0 = tx.wrapping_add(d1).wrapping_add(8).wrapping_sub(e.b.xi()) as u16;
        let arrived = d0 <= 0x10 && (tz.wrapping_add(7).wrapping_sub(e.b.zi()) as u16) < 0xA;
        if !arrived {
            self.enemy_c13_3e4dc(e); // 0x3EA0C
            self.enemy_move_dir(&mut e.b);
            return {
                self.enemy_step(&mut e.b);
            };
        }
        e.m.sb(ARRIVED, 1); // 0x3EA1C
        e.m.sb(STILL_T, 0);
    }

    /// 0x3EAFA (state 0x12): the jump (airborne): physics; on landing 8 frames of state 0x14 (anim 0x14).
    fn enemy_c13_3eafa(&mut self, e: &mut Enemy) {
        e.b.motion(); // 0x189C
        if e.b.airborne == 0 {
            e.m.sw(TIMER, 8);
            e.b.act = 0x14;
            self.enemy_anim(&mut e.b, 0x14);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x3EB8C (states 0x14, 0x18, 0x3C, 0x46): pause `+$AC` frames, then free again (`+$A1` = 0) and decide.
    fn enemy_c13_3eb8c(&mut self, e: &mut Enemy) {
        if Self::enemy_c13_tick_timer(e) {
            e.m.sb(BUSY, 0);
            return self.enemy_c13_4005c(e);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x3EBA4 (state 0x16): close in at speed id 6 (the walk of the decision `0x3F5E2`), then the far decision proper (0x40156).
    fn enemy_c13_3eba4(&mut self, e: &mut Enemy) {
        if Self::enemy_c13_tick_timer(e) {
            return self.enemy_c13_40156(e);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x3F038 (state 0x34): stand (anim 5) for `+$AC` frames or until the target is within X class 2 (0x3293C).
    fn enemy_c13_3f038(&mut self, e: &mut Enemy) {
        if Self::enemy_c13_tick_timer(e) {
            return self.enemy_c13_4005c(e);
        }
        let (dx, _) = self.lib_distance_class(e);
        if (dx as i16) < 3 {
            return self.enemy_c13_4005c(e);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x3EF56 (state 0x36): rise with the fade: the palette steps 4, 8 and 0xC (0x3EFD0) at 40, 20 and 1 frames left, Z += +$B4 a frame;
    /// at the end the final palette override (0x8F / 0x90 / 0x91 by the variant) and a decision.
    fn enemy_c13_3ef56(&mut self, e: &mut Enemy) {
        if Self::enemy_c13_tick_timer(e) {
            self.lib_face_target(e);
            let v = (e.b.var26 >> 8) as u8;
            e.b.palette_override = 0x8F;
            if v == 2 {
                e.b.palette_override = 0x90;
            }
            if v == 4 {
                e.b.palette_override = 0x91;
            }
            return self.enemy_c13_4005c(e);
        }
        let t = e.m.w(TIMER) as i16;
        if t <= 0x28 {
            self.enemy_c13_3efd0(e, 4);
        }
        if t <= 0x14 {
            self.enemy_c13_3efd0(e, 8);
        }
        if t == 1 {
            self.enemy_c13_3efd0(e, 0xC);
        }
        e.b.z = e.b.z.wrapping_add(e.m.l(VZ) as i32);
        self.enemy_step(&mut e.b);
    }

    /// 0x3F058 (state 0x38): fall from above (60 frames); on landing face the target and decide.
    fn enemy_c13_3f058(&mut self, e: &mut Enemy) {
        e.b.motion();
        if e.b.airborne == 0 {
            self.lib_face_target(e);
            return self.enemy_c13_4005c(e);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x3F076 (state 0x4E): the jump on (anim 9; anim 0x1C when falling); on landing face the target and decide.
    fn enemy_c13_3f076(&mut self, e: &mut Enemy) {
        e.b.motion();
        if e.b.airborne == 0 {
            self.lib_face_target(e);
            return self.enemy_c13_4005c(e);
        }
        if e.b.vy < 0 {
            self.enemy_anim(&mut e.b, 0x1C);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x3F0A4 (state 0x3E): run in at speed id 0xD toward the slot point (36 frames); stops within 0x10 px of the target in X when
    /// less than 0x24 frames are left, or when the time is out.
    fn enemy_c13_3f0a4(&mut self, e: &mut Enemy) {
        let t = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, t);
        if (t as i16) <= 0x24 {
            let tx = self.lib_target(e).xi();
            let d0 = tx.wrapping_add(0x10).wrapping_sub(e.b.xi()) as u16;
            if d0 <= 0x20 || t == 0 {
                self.lib_face_target(e);
                return self.enemy_c13_4005c(e);
            }
        }
        if Self::enemy_c13_tick_aim(e) {
            e.m.sb(AIM_T, 0x1E);
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x3EB66 (state 0x48): the swing at an obstacle (anim 0x1B): decide when it ends.
    fn enemy_c13_3eb66(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            self.enemy_c13_4005c(e);
        }
    }

    // ------------------------------------------------------------------ turn limits

    /// 0x3F2E8: the new direction `+$22` may differ from the old one `+$A9` by at most 1 step.
    pub(super) fn enemy_c13_3f2e8(&mut self, e: &mut Enemy) {
        let old = e.m.b(OLD_DIR);
        let mut d0 = e.b.dir.wrapping_sub(old) & 0x1F;
        d0 = d0.wrapping_add(1) & 0x1F;
        if (d0 as i8) <= 2 {
            return;
        }
        d0 = d0.wrapping_sub(1);
        if (d0 as i8) > 0x10 {
            e.b.dir = old.wrapping_sub(1) & 0x1F; // 0x3F31E
        } else {
            e.b.dir = old.wrapping_add(1) & 0x1F;
        }
    }

    /// 0x3F32E: the same with 2 steps.
    pub(super) fn enemy_c13_3f32e(&mut self, e: &mut Enemy) {
        let old = e.m.b(OLD_DIR);
        let mut d0 = e.b.dir.wrapping_sub(old) & 0x1F;
        d0 = d0.wrapping_add(2) & 0x1F;
        if (d0 as i8) <= 4 {
            return;
        }
        d0 = d0.wrapping_sub(2);
        if (d0 as i8) > 0x10 {
            e.b.dir = old.wrapping_sub(2) & 0x1F; // 0x3F364
        } else {
            e.b.dir = old.wrapping_add(2) & 0x1F;
        }
    }

    // ------------------------------------------------------------------ the decisions

    /// 0x4005C: the decision when a state ends. Speed id 6, the flags cleared, the X distance class stored in `+$A5` and a roll of 32 rows of
    /// the table 0x72A94 (by variant, `rand & 0x1F` and the class) chooses the next state: 1 shuffle, 2 flank, 3 aimed walk, 4 drift, 5
    /// approach, 8 drift in Z first (the values 6, 7, 9 and 10 do not occur in the ROM table).
    pub(super) fn enemy_c13_4005c(&mut self, e: &mut Enemy) {
        self.enemy_set_speed(&mut e.b, 6);
        e.m.sb(ARRIVED, 0);
        e.m.sb(BUSY, 0);
        e.m.sb(LUNGE, 0);
        e.m.sb(COMMIT, 0);
        let (dx, _) = self.lib_distance_class(e); // 0x3293C
        e.m.sb(DIST, dx as u8);
        e.m.sb(PREV_ACT, e.b.act);
        let r = (self.rand() & 0x1F) as u32;
        let idx = (r << 3) + e.m.b(DIST) as u32 + (Self::enemy_c13_variant(e) << 7);
        match self.assets.enemy.t.u8(t::DECISION1, idx) {
            1 => self.enemy_c13_3f374(e),
            2 => self.enemy_c13_3f206(e),
            3 => self.enemy_c13_3f1c8(e),
            4 => self.enemy_c13_3f4d2(e),
            5 => self.enemy_c13_3f15e(e),
            8 => self.enemy_c13_3f570(e),
            v => panic!("class 13 decision 1 value {v:#x} (not in the ROM table)"),
        }
    }

    /// 0x40118: the far decision: a gate (the roll `rand & 0x1F` below the byte of the table 0x72D94 by aggression, variant row and the X class
    /// of the LAST decision, `+$A5`), not while the target is in a hit reaction; then 0x40156.
    pub(super) fn enemy_c13_40118(&mut self, e: &mut Enemy) {
        let t = self.lib_target(e);
        if t.mode == 2 && t.sub == 2 {
            return; // cmpi.w #$202,$4(a0)
        }
        let d0 = (self.rank.aggression << 3).wrapping_add((Self::enemy_c13_variant(e) << 5) as u16);
        // add.b $A5(a6),d0: only the low byte takes the sum
        let d0 = (d0 & 0xFF00) | (d0 as u8).wrapping_add(e.m.b(DIST)) as u16;
        let d1 = self.assets.enemy.t.u8(t::FAR_GATE, d0 as u32) as i8;
        let r = (self.rand() & 0x1F) as i8;
        if d1 > r {
            self.enemy_c13_40156(e);
        }
    }

    /// 0x40156: the far decision proper: face the target, speed id 6, the X class again, then a roll of the table 0x72F14 (by variant,
    /// `rand & 0xF8` and the class): 1, 3, 4, 5 melee (0x3F998), 2 kick approach (0x3FB20), 6 the dash (0x3F82A, dropped when the target is
    /// closer than 100 px), 7 the rifle volley (0x3F6B2), 8 wait (0x3FC60).
    pub(super) fn enemy_c13_40156(&mut self, e: &mut Enemy) {
        self.lib_face_target(e);
        self.enemy_set_speed(&mut e.b, 6);
        e.m.sb(BUSY, 0);
        e.m.sb(LUNGE, 0);
        let (dx, _) = self.lib_distance_class(e);
        e.m.sb(DIST, dx as u8);
        let r = (self.rand() & 0xF8) as u32;
        let idx = r + e.m.b(DIST) as u32 + (Self::enemy_c13_variant(e) << 7);
        match self.assets.enemy.t.u8(t::FAR_DECISION, idx) {
            1 | 3 | 4 | 5 => self.enemy_c13_3f998(e),
            2 => self.enemy_c13_3fb20(e),
            6 => self.enemy_c13_3f82a(e),
            7 => self.enemy_c13_3f6b2(e),
            8 => self.enemy_c13_3fc60(e),
            v => panic!("class 13 far decision value {v:#x} (not in the ROM table)"),
        }
    }

    // ------------------------------------------------------------------ state entries of the decision 1

    /// 0x3F374 (value 1): shuffle (state 0xA, 60 frames, anim 1): +$B0 = 0x15000 right or 0xFFFEB000 left, +$B4 = +-0x10000.
    fn enemy_c13_3f374(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 0);
        e.m.sw(TIMER, 0x3C);
        e.b.act = 0xA;
        self.enemy_c13_3f386(e);
    }

    /// 0x3F386: the steps of the shuffle and the animation 1 (also the tail of the decision 8).
    fn enemy_c13_3f386(&mut self, e: &mut Enemy) {
        let to_right = (e.m.w(DEST_X) as i16).wrapping_add(0x400) > e.b.xi().wrapping_add(0x400);
        e.m.sl(VX, if to_right { 0x1_5000 } else { 0xFFFE_B000 });
        let deeper = (e.m.w(DEST_Z) as i16) > e.b.zi();
        e.m.sl(VZ, if deeper { 0x1_0000 } else { 0xFFFF_0000 });
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x3F206 (value 2): flank (state 0x2A, 120 frames, anim 1, speed id 0xF) to a point 0x30 px beside the target.
    fn enemy_c13_3f206(&mut self, e: &mut Enemy) {
        e.m.sb(OLD_DIR, e.b.dir);
        self.lib_aim_beside_target(e, 0x30); // 0x32A88
        self.lib_face_target(e);
        self.enemy_set_speed(&mut e.b, 0xF);
        e.m.sb(BUSY, 1);
        e.m.sb(AIM_T, 6);
        e.m.sw(TIMER, 0x78);
        e.b.act = 0x2A;
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x3F1C8 (value 3): aimed walk (state 0xC, 120 frames, anim 1, speed id 0xF).
    fn enemy_c13_3f1c8(&mut self, e: &mut Enemy) {
        e.m.sb(OLD_DIR, e.b.dir);
        self.lib_aim_at_target(e); // 0x32A70
        self.enemy_c13_3f2e8(e);
        self.lib_face_target(e);
        self.enemy_set_speed(&mut e.b, 0xF);
        e.m.sb(BUSY, 0);
        e.m.sb(AIM_T, 6);
        e.m.sw(TIMER, 0x78);
        e.b.act = 0xC;
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x3F4D2 (value 4): drift (state 6, 120 frames): the speed `+$C0` / `+$B0` from the table 0x3F530 (`rand & 0x3C`), signed by the side
    /// of the point; anim 1, or 0x1E at 0x14000 and more.
    fn enemy_c13_3f4d2(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 0);
        e.m.sw(TIMER, 0x78);
        e.b.act = 6;
        let r = (self.rand() & 0x3C) as u32;
        let d0 = self.assets.enemy.t.u32(0x3F530, r);
        e.m.sl(SPEED, d0);
        e.m.sl(VX, d0);
        let to_right = (e.m.w(DEST_X) as i16).wrapping_add(0x400) > e.b.xi().wrapping_add(0x400);
        if !to_right {
            e.m.sl(VX, d0.wrapping_neg());
        }
        self.lib_face_target(e);
        let anim = if (e.m.l(VX) as i32) < 0x1_4000 { 1 } else { 0x1E };
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x3F15E (value 5): approach the slot (state 0xE, 180 frames) at speed id 0xD, anim 3 unless the state was 0xE before.
    fn enemy_c13_3f15e(&mut self, e: &mut Enemy) {
        e.b.act = 0xE;
        e.m.sw(TIMER, 0xB4);
        e.m.sb(AIM_T, 0x1E);
        let _ = self.lib_slot6_new(e); // 0x32C5E
        e.m.sb(OLD_DIR, e.b.dir);
        self.lib_aim_at_target(e);
        if (e.b.dir as i8) >= 0x10 {
            e.b.facing = 0; // 0x3F19E
            e.b.dir = 0x18;
        } else {
            e.b.facing = 1;
            e.b.dir = 8;
        }
        self.enemy_set_speed(&mut e.b, 0xD);
        if e.b.act != e.m.b(PREV_ACT) {
            self.enemy_anim(&mut e.b, 3);
        }
    }

    /// 0x3F570 (value 8): shuffle in Z first (state 8, 120 frames), then the steps of 0x3F386.
    fn enemy_c13_3f570(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 0);
        e.m.sw(TIMER, 0x78);
        e.b.act = 8;
        self.lib_face_target(e);
        self.enemy_c13_3f386(e);
    }

    // ------------------------------------------------------------------ small entries

    /// 0x3F3D8: off screen: run back onto it (state 0x4A, 100 frames, speed id 0x1C) toward a new slot point; on screen: decide.
    pub(super) fn enemy_c13_3f3d8(&mut self, e: &mut Enemy) {
        if !self.lib_off_screen(&e.b) {
            return self.enemy_c13_4005c(e);
        }
        e.b.act = 0x4A;
        e.m.sw(TIMER, 0x64);
        e.m.sb(AIM_T, 0x1E);
        let _ = self.lib_slot6_new(e);
        e.m.sb(OLD_DIR, e.b.dir);
        self.lib_aim_at_target(e);
        let d0 = e.b.dir.wrapping_add(8) & 0x10;
        if (d0 as i8) >= 0x10 {
            e.b.facing = 0; // 0x3F42A
            e.b.dir = 0x10;
        } else {
            e.b.facing = 1;
            e.b.dir = 0;
        }
        let r = self.rand() & 0xF;
        if r > 0xC {
            e.b.dir = e.b.dir.wrapping_add(0x10) & 0x10;
        }
        self.enemy_set_speed(&mut e.b, 0x1C);
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x3F464: the end of the run back: still off screen, slide (state 0x4C, 100 frames, anim 1) with the steps +$B0 / +$B4 of the table of
    /// the side; on screen: decide.
    pub(super) fn enemy_c13_3f464(&mut self, e: &mut Enemy) {
        if !self.lib_off_screen(&e.b) {
            return self.enemy_c13_4005c(e);
        }
        e.m.sb(BUSY, 0);
        e.m.sw(TIMER, 0x64);
        e.b.act = 0x4C;
        let to_right = (e.m.w(DEST_X) as i16).wrapping_add(0x400) > e.b.xi().wrapping_add(0x400);
        e.m.sl(VX, if to_right { 0x2_4000 } else { 0xFFFD_C000 });
        let deeper = (e.m.w(DEST_Z) as i16) > e.b.zi();
        e.m.sl(VZ, if deeper { 0x1_C000 } else { 0xFFFE_4000 });
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x3F242 (an obstacle in the way, called by 0x3E2F8): swing at it (state 0x48, 120 frames, anim 0x1B). The 1 in 32 draw of the call
    /// of the anti tamper check 0x5124E (no effect on a genuine ROM, no random number of its own) is kept.
    pub(super) fn enemy_c13_3f242(&mut self, e: &mut Enemy) {
        let r = self.rand() & 0x1F;
        if r == 0 {
            // jsr 0x5124E: the checksum of the code in RAM; carry set only on a changed ROM (then G.$4D9 = 9)
        }
        e.b.facing = 0;
        if (e.b.dir as i8) <= 0x10 {
            e.b.facing = 1;
        }
        e.m.sb(BUSY, 1);
        e.m.sw(TIMER, 0x78);
        e.b.act = 0x48;
        self.enemy_anim(&mut e.b, 0x1B);
    }

    /// 0x3F5E2 (the attack start tests, target close): close in (state 0x16) for `+$AC` frames at speed id 6, anim 0; the timer is the table
    /// 0x743BA by level and variant plus a random add of the table 0x3F636, at least 1.
    pub(super) fn enemy_c13_3f5e2(&mut self, e: &mut Enemy) {
        let d2 = ((e.b.var26 >> 8) as u8 as i8 >> 5) as u8;
        let level = self.w.level;
        let idx = (level & 0xFF00) | (level as u8).wrapping_add(d2) as u16;
        let d3 = self.assets.enemy.t.u8(t::CLOSE_IN, idx as u32) as u16;
        let r = ((self.rand() & 0xF) * 2) as u32;
        let add = self.assets.enemy.t.u16(t::CLOSE_IN_ADD, r);
        let mut d3 = d3.wrapping_add(add);
        if d3 == 0 || (d3 as i16) < 0 {
            d3 = 1;
        }
        e.m.sw(TIMER, d3);
        e.m.sb(BUSY, 1);
        e.b.act = 0x16;
        self.enemy_set_speed(&mut e.b, 6);
        self.enemy_anim(&mut e.b, 0);
    }

    /// 0x3F670 (the end of the entrance 0): state 0x3C (30 frames, anim 0, speed id 6), the flee timer 0xB4, one time in 8 a new target. The
    /// decision that follows (0x4005C) replaces the state at once.
    pub(super) fn enemy_c13_3f670(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 1);
        e.m.sw(TIMER, 0x1E);
        e.b.act = 0x3C;
        e.m.sw(FLEE_T, 0xB4);
        self.enemy_anim(&mut e.b, 0);
        self.enemy_set_speed(&mut e.b, 6);
        if self.rand() & 7 == 0 {
            let tg = self.enemy_pick_target(&e.b);
            e.m.sw(TARGET, tg);
        }
    }

    /// 0x3FC60 (the target is in a hit reaction, or the far decision value 8): wait in state 0x46 for the time of the table 0x3FC84 (anim 0).
    pub(super) fn enemy_c13_3fc60(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 1);
        let r = (self.rand() & 0x1E) as u32;
        let w = self.assets.enemy.t.u16(0x3FC84, r);
        e.m.sw(TIMER, w);
        e.b.act = 0x46;
        self.enemy_anim(&mut e.b, 0);
    }

    /// 0x3F5AE (the end of a melee chain): state 0x14 for 30 frames at speed id 6; one time in 8 a new target.
    pub(super) fn enemy_c13_3f5ae(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 1);
        e.m.sw(TIMER, 0x1E);
        e.b.act = 0x14;
        self.enemy_set_speed(&mut e.b, 6);
        if self.rand() & 7 == 0 {
            let tg = self.enemy_pick_target(&e.b);
            e.m.sw(TARGET, tg);
        }
    }
}
