//! Class 14: the state handlers of `+$06` (table 0x407F8, 30 entries 0..0x3A, handlers 0x40940..0x41154
//! and 0x40EBC). Each handler is one frame of its state; the entry of a state sets `+$06` and its
//! timers (`class14_entry.rs`). `bra 0x41BE8` of the original is `c14_41be8` (the decision).

use super::class14::f;
use super::{Enemy, off::*};
use crate::game::{Game, Handle};

impl Game {
    /// 0x407A2: the handler of the state `+$06` (word table 0x407F8).
    pub(super) fn c14_state(&mut self, e: &mut Enemy, h: Handle) {
        let _ = h;
        match e.b.act {
            0x00 | 0x16 => self.c14_40ebc(e),
            0x02 => self.c14_40940(e),
            0x04 => self.c14_40950(e),
            0x06 => self.c14_40c34(e),
            0x08 => self.c14_409ca(e),
            0x0A => self.c14_40f72(e, h),
            0x0C => self.c14_4096e(e),
            0x0E => self.c14_40c18(e),
            0x10 => self.c14_40d98(e),
            0x12 => self.c14_40da6(e),
            0x14 => self.c14_40bbe(e),
            0x18 => self.c14_40a9a(e),
            0x1A => self.c14_40b2c(e),
            0x1C | 0x1E | 0x20 => self.c14_40f66(e),
            0x22 => self.c14_40bd6(e),
            0x24 => self.c14_40fce(e),
            0x26 => self.c14_410f2(e),
            0x28 | 0x2A => self.c14_41100(e),
            0x2C => self.c14_40d56(e),
            0x2E => self.c14_41120(e),
            0x30 => self.c14_41138(e),
            0x32 => self.c14_41076(e),
            0x34 => self.c14_40bca(e),
            0x36 => self.c14_40c46(e),
            0x38 => self.c14_40c90(e),
            0x3A => self.c14_40c26(e),
            s => panic!("class 14 state {s:#x}"),
        }
    }

    /// `subq.w #1,$A6(a6)`: true when the result is zero.
    fn c14_tick_timer(e: &mut Enemy) -> bool {
        let t = e.m.w(f::TIMER).wrapping_sub(1);
        e.m.sw(f::TIMER, t);
        t == 0
    }

    /// 0x40EBC (states 0 and 0x16): walk to a point beside the target (0x38 away), pausing at it.
    fn c14_40ebc(&mut self, e: &mut Enemy) {
        if Self::c14_tick_timer(e) {
            return self.c14_41be8(e);
        }
        if (e.m.w(f::TIMER) as i16) <= 0x3C {
            e.m.sb(f::BUSY, 0);
        }
        let t = e.m.b(f::TURN_T).wrapping_sub(1);
        e.m.sb(f::TURN_T, t);
        if t == 0 {
            e.m.sb(f::TURN_T, 6);
            e.m.sb(f::OLD_DIR, e.b.dir);
            self.lib_aim_beside_target(e, 0x38);
        }
        let p = e.m.b(f::PAUSE);
        if p != 0 {
            e.m.sb(f::PAUSE, p - 1);
            return;
        }
        let t = self.lib_target(e);
        let (tx, tz) = (t.xi(), t.zi());
        let mut d1 = 0x38i16;
        if tx.wrapping_add(0x400) > e.b.xi().wrapping_add(0x400) {
            d1 = -0x38; // moveq #$C8,d1
        }
        let dx = tx.wrapping_add(d1).wrapping_add(8).wrapping_sub(e.b.xi()) as u16;
        let dz = tz.wrapping_add(9).wrapping_sub(e.b.zi()) as u16;
        if dx <= 0x10 && dz < 0xA {
            e.m.sb(f::MOVED, 1); // 0x40F58
            e.m.sb(f::PAUSE, 0);
            return;
        }
        self.c14_40604(e);
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x40940 (state 2): wait `+$7A`, then the first step of the approach (0x41278).
    fn c14_40940(&mut self, e: &mut Enemy) {
        let w = e.m.b(WAIT);
        if w != 0 {
            e.m.sb(WAIT, w - 1);
            return;
        }
        self.c14_41278(e);
    }

    /// 0x40950 (state 4): walk towards the aim for `+$A6` frames.
    fn c14_40950(&mut self, e: &mut Enemy) {
        if Self::c14_tick_timer(e) {
            return self.c14_41be8(e);
        }
        self.lib_face_target(e);
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x4096E (state 0xC): walk to a point 0x50 beside the target, new point every 0x1E frames.
    fn c14_4096e(&mut self, e: &mut Enemy) {
        let t = e.m.w(f::TIMER).wrapping_sub(1);
        e.m.sw(f::TIMER, t);
        if (t as i16) <= 0x96 {
            let d0 = (e.m.w(DEST_X) as i16).wrapping_add(0x60).wrapping_sub(e.b.xi()) as u16;
            if d0 <= 0xC0 || e.m.w(f::TIMER) == 0 {
                return self.c14_41be8(e);
            }
        }
        let t = e.m.b(f::TURN_T).wrapping_sub(1);
        e.m.sb(f::TURN_T, t);
        if t == 0 {
            e.m.sb(f::TURN_T, 0x1E);
            self.lib_slot3_new(e); // 0x32D90
            e.m.sb(f::OLD_DIR, e.b.dir);
            self.lib_aim_beside_target(e, 0x50);
            self.c14_412b0(e);
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x409CA (state 8): side step to the point (+$88, +$8A) at the speeds +$B0 / +$B4 (16.16 per
    /// frame), re-aimed every 4 frames; pauses (`+$AC`) are drawn by 0x40604.
    fn c14_409ca(&mut self, e: &mut Enemy) {
        if Self::c14_tick_timer(e) {
            return self.c14_41be8(e);
        }
        if e.m.w(f::TIMER) & 3 == 0 {
            let right = (e.m.w(DEST_X) as i16).wrapping_add(0x400) > e.b.xi().wrapping_add(0x400);
            e.m.sl(f::VX, if right { 0x1D000 } else { 0xFFFE_3000 });
            let up = (e.m.w(DEST_Z) as i16) > e.b.zi();
            e.m.sl(f::VZ, if up { 0x18000 } else { 0xFFFE_8000 });
        }
        let p = e.m.b(f::PAUSE);
        if p != 0 {
            e.m.sb(f::PAUSE, p - 1);
            return;
        }
        let mut moved = false;
        let dx = (e.m.w(DEST_X) as i16).wrapping_add(8).wrapping_sub(e.b.xi()) as u16;
        if dx >= 0x10 {
            e.b.x = e.b.x.wrapping_add(e.m.l(f::VX) as i32);
            moved = true;
        }
        let dz = (e.m.w(DEST_Z) as i16).wrapping_add(0xA).wrapping_sub(e.b.zi()) as u16;
        if dz >= 0xB {
            e.b.z = e.b.z.wrapping_add(e.m.l(f::VZ) as i32);
            moved = true;
        }
        if moved {
            self.c14_40604(e);
            if e.m.b(f::PAUSE) == 0 {
                self.enemy_step(&mut e.b);
            }
        } else {
            e.m.sb(f::MOVED, 1);
        }
    }

    /// 0x40A9A (state 0x18): side step in X at the speed chosen by the entry (`+$C4`, towards the point).
    fn c14_40a9a(&mut self, e: &mut Enemy) {
        if Self::c14_tick_timer(e) {
            return self.c14_41be8(e);
        }
        if e.m.w(f::TIMER) & 3 == 0 {
            e.m.sl(f::VX, e.m.l(f::SPEED_C4));
            if (e.m.w(DEST_X) as i16).wrapping_add(0x400) <= e.b.xi().wrapping_add(0x400) {
                e.m.sl(f::VX, (e.m.l(f::VX) as i32).wrapping_neg() as u32);
            }
        }
        let p = e.m.b(f::PAUSE);
        if p != 0 {
            e.m.sb(f::PAUSE, p - 1);
            return;
        }
        let dx = (e.m.w(DEST_X) as i16).wrapping_add(4).wrapping_sub(e.b.xi()) as u16;
        if dx < 8 {
            e.m.sb(f::MOVED, 1);
            return self.lib_face_target(e);
        }
        self.c14_40604(e);
        if e.m.b(f::PAUSE) != 0 {
            return;
        }
        e.b.x = e.b.x.wrapping_add(e.m.l(f::VX) as i32);
        self.enemy_step(&mut e.b);
        self.lib_face_target(e);
    }

    /// 0x40B2C (state 0x1A): side step in Z.
    fn c14_40b2c(&mut self, e: &mut Enemy) {
        if Self::c14_tick_timer(e) {
            return self.c14_41be8(e);
        }
        if e.m.w(f::TIMER) & 3 == 0 {
            let more = (e.m.w(DEST_Z) as i16) > e.b.zi();
            e.m.sl(f::VZ, if more { 0x18000 } else { 0xFFFE_8000 });
        }
        let p = e.m.b(f::PAUSE);
        if p != 0 {
            e.m.sb(f::PAUSE, p - 1);
            return;
        }
        let dz = (e.m.w(DEST_Z) as i16).wrapping_add(0x10).wrapping_sub(e.b.zi()) as u16;
        if dz < 0x20 {
            e.m.sb(f::MOVED, 1);
            return self.lib_face_target(e);
        }
        self.c14_40604(e);
        if e.m.b(f::PAUSE) != 0 {
            return;
        }
        e.b.z = e.b.z.wrapping_add(e.m.l(f::VZ) as i32);
        self.enemy_step(&mut e.b);
        self.lib_face_target(e);
    }

    /// 0x40BBE (state 0x14): an attack animation; at its end 0x411F0.
    fn c14_40bbe(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            self.c14_411f0(e);
        }
    }

    /// 0x40BCA (state 0x34) and 0x40F66 (states 0x1C, 0x1E, 0x20): the animation, then the decision.
    fn c14_40bca(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            self.c14_41be8(e);
        }
    }

    fn c14_40f66(&mut self, e: &mut Enemy) {
        self.c14_40bca(e);
    }

    /// 0x40BD6 (state 0x22): the jump back (flight), then state 0xE on landing.
    fn c14_40bd6(&mut self, e: &mut Enemy) {
        e.b.motion(); // 0x189C
        if (e.b.vy) < 0 {
            let id = if e.m.b(f::KNIVES) != 0 { 0x16 } else { 0x1A };
            self.enemy_anim(&mut e.b, id);
        }
        if e.b.airborne == 0 {
            e.m.sw(f::TIMER, 0x1E);
            e.b.act = 0xE;
            self.enemy_anim(&mut e.b, 0x15);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x40C18 (state 0xE): wait `+$A6` frames, then the decision.
    fn c14_40c18(&mut self, e: &mut Enemy) {
        if Self::c14_tick_timer(e) {
            return self.c14_41be8(e);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x40C26 (state 0x3A): wait, then the next attack of the chain (0x40DFA).
    fn c14_40c26(&mut self, e: &mut Enemy) {
        if Self::c14_tick_timer(e) {
            return self.c14_40dfa(e);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x40C34 (state 6): wait, then the second decision (0x41CD0).
    fn c14_40c34(&mut self, e: &mut Enemy) {
        if Self::c14_tick_timer(e) {
            return self.c14_41cd0(e);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x40C46 (state 0x36): run away from the target (off screen), turn to the aim every 12 frames.
    fn c14_40c46(&mut self, e: &mut Enemy) {
        if Self::c14_tick_timer(e) {
            return self.c14_41b2e(e);
        }
        let t = e.m.b(f::TURN_T).wrapping_sub(1);
        e.m.sb(f::TURN_T, t);
        if t == 0 {
            e.m.sb(f::TURN_T, 0xC);
            e.m.sb(f::OLD_DIR, e.b.dir);
            self.lib_aim_at_target(e); // 0x32A70
            self.c14_412b0(e); // 0x41BA2 is a copy of 0x412B0
        }
        e.b.facing = if (e.b.dir as i8) <= 0x10 { 1 } else { 0 };
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x40C90 (state 0x38): walk at the target at 0x24000 / 0x1C000 per frame (16.16) until the timer is out.
    fn c14_40c90(&mut self, e: &mut Enemy) {
        if Self::c14_tick_timer(e) {
            return self.c14_41a96(e);
        }
        let t = self.lib_target(e);
        let (tx, tz) = (t.xi(), t.zi());
        let right = tx.wrapping_add(0x400) > e.b.xi().wrapping_add(0x400);
        e.b.x = e.b.x.wrapping_add(if right { 0x24000 } else { 0xFFFD_C000u32 as i32 });
        let more = tz > e.b.zi();
        e.b.z = e.b.z.wrapping_add(if more { 0x1C000 } else { 0xFFFE_4000u32 as i32 });
        self.enemy_step(&mut e.b);
    }

    /// 0x40D56 (state 0x2C, the back step of a block): slide, then pick the pause after it.
    fn c14_40d56(&mut self, e: &mut Enemy) {
        if Self::c14_tick_timer(e) {
            let b = self.assets.enemy.t.u8(0x40D88, self.rank.aggression as u32);
            e.m.sb(f::COOL, b);
            return self.c14_41be8(e);
        }
        if e.b.vx != 0 {
            self.lib_face_target(e);
            e.b.motion(); // 0x189C
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x40D98 (state 0x10): wait, then the decision.
    fn c14_40d98(&mut self, e: &mut Enemy) {
        if Self::c14_tick_timer(e) {
            return self.c14_41be8(e);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x40DA6 (state 0x12): a stand wait (0x1E frames of the entrance), then busy for 0xB4 frames.
    fn c14_40da6(&mut self, e: &mut Enemy) {
        if Self::c14_tick_timer(e) {
            e.m.sb(f::BUSY, 1);
            e.m.sw(f::TIMER, 0xB4);
            self.enemy_anim(&mut e.b, 1);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x40DFA: the next attack of a chain: when `+$BC` reaches 0 go on to state 0xE, else start the
    /// next attack (ids 0x17, and 0x1A when no knives are left) in state 0x14.
    pub(super) fn c14_40dfa(&mut self, e: &mut Enemy) {
        let n = e.m.b(f::ATK_STEP).wrapping_sub(1);
        e.m.sb(f::ATK_STEP, n);
        if n == 0 {
            e.m.sb(f::BUSY, 1);
            e.m.sw(f::TIMER, 0x1E);
            e.b.act = 0xE;
            return self.enemy_anim(&mut e.b, 0);
        }
        if (e.m.b(f::DIST) as i8) > 3 {
            return self.c14_41be8(e);
        }
        self.enemy_begin_attack(&mut e.b, 0x17);
        if e.m.b(f::KNIVES) == 0 {
            self.enemy_begin_attack(&mut e.b, 0x1A);
        }
        e.m.sb(f::BUSY, 1);
        e.b.act = 0x14;
        e.m.sw(f::TIMER, 0x3C);
        self.lib_aim_beside_target(e, 0x50);
        let idx = e.m.b(f::ATK_NO) as u32 * 4;
        let mut id = self.assets.enemy.t.u8(0x40EA8, idx) as u16;
        if e.m.b(f::KNIVES) == 0 && id == 0xB {
            id = 0xA;
        }
        self.enemy_anim(&mut e.b, id);
    }

    /// 0x40F72 (state 0xA): the throw. The animation ends in the decision; on the frame with the frame
    /// byte 0x2F the blade (class 41) leaves the hand (sound 0x9D44).
    fn c14_40f72(&mut self, e: &mut Enemy, h: Handle) {
        if self.enemy_step(&mut e.b) {
            return self.c14_41be8(e);
        }
        if e.b.param_b != 0x2F {
            return;
        }
        self.snd_9d44();
        let Some(j) = self.enemies.alloc() else { return };
        let n = &mut self.enemies.slots[j];
        n.b.set_state(0);
        n.b.live = 1;
        n.b.k20 = 0xA4; // class 41
        n.b.var26 = 0;
        n.b.facing = e.b.facing;
        super::lib32::put_hi(&mut n.b.x, e.b.xi());
        super::lib32::put_hi(&mut n.b.y, e.b.yi());
        super::lib32::put_hi(&mut n.b.z, e.b.zi());
        n.m.sl(0xA0, 0xFFFF_0000 | h as u32);
        e.m.sb(f::KNIVES, e.m.b(f::KNIVES).wrapping_sub(1));
    }

    /// 0x40FCE (state 0x24): run at the target at a throwing distance; the run (state 0x32) starts when
    /// the target is within reach or the timer is out; every 0xF frames a knife aim (`+$A1`).
    fn c14_40fce(&mut self, e: &mut Enemy) {
        let t = e.m.w(f::TIMER).wrapping_sub(1);
        e.m.sw(f::TIMER, t);
        let mut go = t == 0;
        if !go {
            let tx = self.lib_target(e).xi();
            let d0 = tx.wrapping_add(0x70).wrapping_sub(e.b.xi()) as u16;
            go = d0 <= 0xC0;
        }
        if go {
            // 0x40FF8: the dash.
            e.b.vz = 0;
            e.b.az = 0;
            e.b.vy = 0;
            e.b.ay = 0;
            e.b.vx = 0x800;
            e.b.ax = -0x10;
            if e.b.facing == 0 {
                e.b.vx = e.b.vx.wrapping_neg();
                e.b.ax = e.b.ax.wrapping_neg();
            }
            e.m.sb(f::BUSY, 1);
            e.m.sw(f::TIMER, 0x64);
            e.b.act = 0x32;
            self.enemy_anim(&mut e.b, 0x1B);
        }
        // 0x41040
        let t = e.m.b(f::TURN_T).wrapping_sub(1);
        e.m.sb(f::TURN_T, t);
        if t == 0 {
            e.m.sb(f::TURN_T, 0xF);
            let a1 = e.m.b(f::KNIVES_TO_THROW);
            if a1 != 0 {
                e.m.sb(f::KNIVES_TO_THROW, a1 - 1);
                e.m.sb(f::OLD_DIR, e.b.dir);
                self.lib_aim_at_target(e);
                self.c14_412b0(e);
            }
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x41076 (state 0x32): the dash ends when the speed has turned round or the timer is out; then
    /// a pause from the table 0x410CC and state 0xE.
    fn c14_41076(&mut self, e: &mut Enemy) {
        let turned = if e.b.facing == 0 { e.b.vx >= 0 } else { e.b.vx < 0 };
        let end = turned || Self::c14_tick_timer(e);
        if end {
            let r = (self.rand() & 0x1F) as u32;
            let v = self.assets.enemy.t.u8(0x410CC, r) as u16;
            e.m.sw(f::TIMER, v);
            e.m.sb(f::BUSY, 1);
            e.b.act = 0xE;
            self.enemy_anim(&mut e.b, 0);
        }
        e.b.motion(); // 0x189C
        self.enemy_step(&mut e.b);
    }

    /// 0x410F2 (state 0x26, wind up of the jump kick): at the end the kick (0x414B0).
    fn c14_410f2(&mut self, e: &mut Enemy) {
        if Self::c14_tick_timer(e) {
            return self.c14_414b0(e);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x41100 (states 0x28, 0x2A: the stand waits of the entrances): until the timer is out or the target
    /// is within the distance class 3.
    fn c14_41100(&mut self, e: &mut Enemy) {
        if Self::c14_tick_timer(e) {
            return self.c14_41be8(e);
        }
        let (dx, _) = self.lib_distance_class(e);
        if (dx as i16) < 3 {
            return self.c14_41be8(e);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x41120 (state 0x2E, the jump in): flight, then the first approach step on landing.
    fn c14_41120(&mut self, e: &mut Enemy) {
        e.b.motion();
        if e.b.airborne == 0 {
            return self.c14_41278(e);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x41138 (state 0x30, the walk in): until the timer is out or the target is in reach.
    fn c14_41138(&mut self, e: &mut Enemy) {
        let t = e.m.w(f::TIMER).wrapping_sub(1);
        e.m.sw(f::TIMER, t);
        let go = if (t as i16) <= 0x24 {
            let tx = self.lib_target(e).xi();
            let mut d0 = tx.wrapping_add(0x10).wrapping_sub(e.b.xi()).wrapping_add(0x30);
            if e.b.facing == 0 {
                d0 = d0.wrapping_sub(0x60);
            }
            d0 as u16 <= 0x20 || e.m.w(f::TIMER) == 0
        } else {
            false
        };
        if go {
            return self.c14_41be8(e);
        }
        let t = e.m.b(f::TURN_T).wrapping_sub(1);
        e.m.sb(f::TURN_T, t);
        if t == 0 {
            e.m.sb(f::TURN_T, 0x1E);
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }
}
