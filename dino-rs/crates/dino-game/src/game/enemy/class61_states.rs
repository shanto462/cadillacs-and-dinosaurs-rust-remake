//! The Butcher (class 61): the state bodies, one per value of `+$06` (act). The jump table is at 0x588E0; D3c section 3.3 names the moves.
//! A state returns after it ran once for the frame; most end by calling a move block (`class61_moves.rs`). Owner: enemy port (M6-10).

use super::class61::{above, c61_count, o};
use super::{off::*, Enemy};
use crate::game::{slot_of, Game, Handle, Slot};

impl Game {
    /// 0x588E0: the state table.
    pub(super) fn c61_state(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0x00 => {
                // 0x591B8: the slash animation; at its end the pause between two slashes.
                if self.enemy_step(&mut e.b) {
                    self.c61_59796(e);
                }
            }
            0x02 | 0x1A => self.c61_s02(e),
            0x04 | 0x34 => self.c61_wait_then_decide(e),
            0x06 => self.c61_s06(e),
            0x08 => self.c61_s08(e),
            0x0A => self.c61_s0a(e),
            0x0C => self.c61_s0c(e),
            0x0E => {
                // 0x591EE
                if self.enemy_step(&mut e.b) {
                    self.c61_5a186(e);
                }
            }
            0x10 => self.c61_s10(e, h),
            0x12 => self.c61_s12(e),
            0x14 | 0x16 => {
                // 0x5954C / 0x59558
                if self.enemy_step(&mut e.b) {
                    self.c61_598f2(e);
                }
            }
            0x18 => self.c61_s18(e),
            0x1C => {
                // 0x591C4
                let t = e.m.w(o::TIMER).wrapping_sub(1);
                e.m.sw(o::TIMER, t);
                if t == 0 {
                    self.c61_5946a(e);
                }
            }
            0x1E => {
                // 0x591CE
                if self.enemy_step(&mut e.b) {
                    self.c61_59d86(e);
                }
            }
            0x20 => {
                // 0x591DA
                let t = e.m.w(o::TIMER).wrapping_sub(1);
                e.m.sw(o::TIMER, t);
                if t == 0 {
                    return self.c61_5a04e(e); // 0x59474
                }
                e.b.motion();
                self.enemy_step(&mut e.b);
            }
            0x22 => self.c61_s22(e),
            0x24 => {
                // 0x590DE
                let t = e.m.w(o::TIMER).wrapping_sub(1);
                e.m.sw(o::TIMER, t);
                if t == 0 {
                    self.c61_59e56(e);
                }
            }
            0x26 => self.c61_s26(e),
            0x28 => self.c61_sub4(e, h),
            0x2A => {
                // 0x591FA
                let t = e.m.w(o::TIMER).wrapping_sub(1);
                e.m.sw(o::TIMER, t);
                if t == 0 {
                    return self.c61_59b44(e);
                }
                self.enemy_step(&mut e.b);
            }
            0x2C => self.c61_s2c(e),
            0x2E => {
                // 0x5912E
                let t = e.m.w(o::TIMER).wrapping_sub(1);
                e.m.sw(o::TIMER, t);
                if t == 0 {
                    return self.c61_59ec0(e, h);
                }
                self.enemy_step(&mut e.b);
            }
            // 0x30 and 0x32 point at the table itself in the original (no code sets them: 0x5A090 and 0x5A11C are reached only through
            // `+$CA`, which is never set).
            _ => {}
        }
    }

    /// 0x5953E: wait `+$AE` frames, animate, then the decision.
    fn c61_wait_then_decide(&mut self, e: &mut Enemy) {
        let t = e.m.w(o::TIMER).wrapping_sub(1);
        e.m.sw(o::TIMER, t);
        if t == 0 {
            return self.c61_5a186(e);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x59796: the pause after a slash (act 0x1C): `+$AE` from the table 0x74DA2 by a random number plus the table 0x74DE2 by the level
    /// (at least 1).
    pub(super) fn c61_59796(&mut self, e: &mut Enemy) {
        e.m.sb(0xA0, 1);
        e.b.act = 0x1C;
        let r = (self.rand() & 0x3E) as u32;
        let t = &self.assets.enemy.t;
        let mut v = t.u16(0x74DA2, r);
        let add = t.u16(0x74DE2, 2 * self.w.level as u32);
        v = v.wrapping_add(add);
        e.m.sw(o::TIMER, if (v as i16) <= 0 { 1 } else { v });
    }

    /// 0x5946A: a pause is over: one step of the combo less; with steps left the next slash (0x5948C), else the dash (0x5A04E).
    fn c61_5946a(&mut self, e: &mut Enemy) {
        let bc = e.m.b(o::BC).wrapping_sub(1);
        e.m.sb(o::BC, bc);
        if bc == 0 {
            return self.c61_5a04e(e);
        }
        self.enemy_begin_attack(&mut e.b, 0x2D);
        if e.m.b(o::KNIVES) == 0 {
            self.enemy_begin_attack(&mut e.b, 0x1B);
        }
        e.m.sb(0xA0, 1);
        e.b.act = 0;
        e.m.sw(o::TIMER, 0x3C);
        self.lib_face_target(e);
        self.lib_aim_beside_target(e, 0x50);
        let idx = 4 * e.m.b(o::BB) as u32 + e.m.b(o::BC) as u32;
        let mut anim = self.assets.enemy.t.u8(0x5951E, idx) as u16;
        if (anim as u8 as i8) < 0 {
            return self.c61_5a04e(e); // 0x5A04E
        }
        if anim == 8 && e.m.b(o::KNIVES) != 0 {
            anim = 0x11;
        }
        if anim == 0x5A {
            return self.c61_59d66(e);
        }
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x58E04: act 6, stand and wait: every fourth frame the step vectors `+$C0` / `+$C4` toward the slot point, then the steps.
    fn c61_s06(&mut self, e: &mut Enemy) {
        let t = e.m.w(o::TIMER).wrapping_sub(1);
        e.m.sw(o::TIMER, t);
        if t == 0 {
            return self.c61_5a186(e);
        }
        if t & 3 == 0 {
            let (dx, dz) = (e.m.w(DEST_X) as i16, e.m.w(DEST_Z) as i16);
            e.m.sl(0xC0, if dx > e.b.xi() { 0x18000 } else { 0xFFFE_8000 });
            e.m.sl(0xC4, if dz > e.b.zi() { 0x10000 } else { 0xFFFF_0000 });
        }
        let a2 = e.m.b(o::A2);
        if a2 != 0 {
            e.m.sb(o::A2, a2 - 1);
            return;
        }
        let (dx, dz) = (e.m.w(DEST_X) as i16, e.m.w(DEST_Z) as i16);
        let mut moved = false;
        if (dx.wrapping_add(8).wrapping_sub(e.b.xi()) as u16) >= 0x10 {
            e.b.x = e.b.x.wrapping_add(e.m.l(0xC0) as i32);
            moved = true;
        }
        if (dz.wrapping_add(0xA).wrapping_sub(e.b.zi()) as u16) >= 0xB {
            e.b.z = e.b.z.wrapping_add(e.m.l(0xC4) as i32);
            moved = true;
        }
        if moved {
            self.c61_walk_adjust(e); // 0x589FA
            if e.m.b(o::A2) != 0 {
                return;
            }
            self.enemy_step(&mut e.b);
            return;
        }
        // 0x58EC0: nothing to move: the first time only the flag `+$AD`, then the animation 0; every time a roll to mirror the point.
        let before = e.m.b(0xAD);
        e.m.sb(0xAD, 1);
        if before == 1 {
            e.m.sb(o::A2, 0);
            return self.enemy_anim(&mut e.b, 0);
        }
        if self.rand() & 0xF < 0xC {
            e.m.sw(DEST_X, (e.m.w(DEST_X) as i16).wrapping_neg() as u16); // neg.w $88(a6): kept as the original has it
        }
    }

    /// 0x589FA: after a step in the walk states: a pause `+$A2` (steps skipped) chosen by the HP in quarters of the bar and a
    /// random number, from the tables 0x744DA / 0x744E2 (with `+$BA`) and 0x744DE / 0x744F2; then the animation 1.
    pub(super) fn c61_walk_adjust(&mut self, e: &mut Enemy) {
        if e.m.b(0xAD) == 0 {
            return;
        }
        let quarter = (e.b.hp_bar as i16) >> 2;
        let mut left = e.b.hp as i16;
        let mut d2: u16 = 0;
        if quarter > 0 {
            loop {
                left = left.wrapping_sub(quarter);
                if left < 0 {
                    break;
                }
                d2 += 1;
            }
        }
        let d0 = (self.rand() & 0xF) as u32;
        let t = &self.assets.enemy.t;
        let low = |d2: u16, v: u8| (d2 & 0xFF00) | v as u16;
        let mut skip = false;
        if e.m.b(o::BA) != 0 {
            d2 = low(d2, t.u8(0x744DA, d2 as u32));
            let sum = (d2 as u8).wrapping_add(t.u8(0x744E2, d0));
            d2 = low(d2, sum);
            skip = sum == 0; // beq 0x58A56
        }
        if !skip {
            d2 = low(d2, t.u8(0x744DE, d2 as u32));
            let sum = (d2 as u8).wrapping_add(t.u8(0x744F2, d0));
            d2 = low(d2, sum);
        }
        let v = d2 as u8;
        e.m.sb(o::A2, if (v as i8) <= 0 { 1 } else { v });
        e.m.sb(0xAD, 0);
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x58EFA: act 0x18, walk to the hero: every fourth frame the vectors toward a point 94 pixels beside him, the steps like act 6.
    fn c61_s18(&mut self, e: &mut Enemy) {
        let t = e.m.w(o::TIMER).wrapping_sub(1);
        e.m.sw(o::TIMER, t);
        if t == 0 {
            return self.c61_5a186(e);
        }
        let tg = self.c61_tgt(e);
        if t & 3 == 0 {
            let d1: i16 = if tg.x > e.b.xi() { -0x5E } else { 0x5E };
            e.b.frame_w1 = d1 as u16; // moveq #$5E / #$A2,d1: the d1 the decision roll 0x5A1AC keeps in its high byte
            e.m.sl(0xC0, if tg.x.wrapping_add(d1) > e.b.xi() { 0x20000 } else { 0xFFFE_0000 });
            e.m.sl(0xC4, if tg.z > e.b.zi() { 0x18000 } else { 0xFFFE_8000 });
        }
        let a2 = e.m.b(o::A2);
        if a2 != 0 {
            e.m.sb(o::A2, a2 - 1);
            return;
        }
        e.b.frame_w1 = 0; // 0x58F78: moveq #0,d1 (1 when a step was made)
        let d2: i16 = if tg.x > e.b.xi() { -0x68 } else { 0x68 };
        let mut moved = false;
        if (tg.x.wrapping_add(d2).wrapping_add(8).wrapping_sub(e.b.xi()) as u16) >= 0x10 {
            e.b.x = e.b.x.wrapping_add(e.m.l(0xC0) as i32);
            moved = true;
        }
        if (tg.z.wrapping_add(0xA).wrapping_sub(e.b.zi()) as u16) >= 0xB {
            e.b.z = e.b.z.wrapping_add(e.m.l(0xC4) as i32);
            moved = true;
        }
        if moved {
            e.b.frame_w1 = 1; // moveq #1,d1
            self.c61_walk_adjust(e);
            if e.m.b(o::A2) != 0 {
                return;
            }
            self.enemy_step(&mut e.b);
        } else {
            e.m.sb(0xAD, 1); // 0x58FE6
            e.m.sb(o::A2, 0);
            self.enemy_anim(&mut e.b, 0);
        }
    }

    /// 0x58FFC: act 0x22, the short approach for the leap (a point 16 pixels beside the hero): then act 0x24 (0x59E22).
    fn c61_s22(&mut self, e: &mut Enemy) {
        let t = e.m.w(o::TIMER).wrapping_sub(1);
        e.m.sw(o::TIMER, t);
        if t == 0 {
            return self.c61_5a186(e);
        }
        let tg = self.c61_tgt(e);
        if t & 3 == 0 {
            let d1: i16 = if tg.x > e.b.xi() { -0x10 } else { 0x10 };
            e.b.frame_w1 = d1 as u16; // moveq #$10 / #$F0,d1 (see c61_s18)
            e.m.sl(0xC0, if tg.x.wrapping_add(d1) > e.b.xi() { 0x20000 } else { 0xFFFE_0000 });
            e.m.sl(0xC4, if tg.z > e.b.zi() { 0x18000 } else { 0xFFFE_8000 });
        }
        let a2 = e.m.b(o::A2);
        if a2 != 0 {
            e.m.sb(o::A2, a2 - 1);
            return;
        }
        e.b.frame_w1 = 0; // 0x5907A: moveq #0,d1 (1 when a step was made)
        let d2: i16 = if tg.x > e.b.xi() { -0x10 } else { 0x10 };
        let mut moved = false;
        if (tg.x.wrapping_add(d2).wrapping_add(8).wrapping_sub(e.b.xi()) as u16) >= 0x10 {
            e.b.x = e.b.x.wrapping_add(e.m.l(0xC0) as i32);
            moved = true;
        }
        if (tg.z.wrapping_add(8).wrapping_sub(e.b.zi()) as u16) >= 0x10 {
            e.b.z = e.b.z.wrapping_add(e.m.l(0xC4) as i32);
            moved = true;
        }
        if moved {
            e.b.frame_w1 = 1; // moveq #1,d1
            self.enemy_step(&mut e.b);
        } else {
            self.c61_59e22(e);
        }
    }

    /// 0x590E8: act 0x26 (after the heavy slash): the count down, a hero in reach starts the grab reach (0x59E3C), else the
    /// dash (0x59474); otherwise the body moves on with its speeds.
    fn c61_s26(&mut self, e: &mut Enemy) {
        let t = e.m.w(o::TIMER).wrapping_sub(1);
        e.m.sw(o::TIMER, t);
        if t == 0 {
            return self.c61_5a04e(e);
        }
        let tg = self.c61_tgt(e);
        if !above(tg.x, 0x58, e.b.xi(), 0xB0) && !above(tg.z, 0x20, e.b.zi(), 0x40) {
            return self.c61_59e3c(e);
        }
        e.b.motion();
        self.enemy_step(&mut e.b);
    }

    /// 0x5913C: sub 4 and act 0x28, holding the hero: the count down; the hero must stay in the held action 0x40 (`+$C8`), else the
    /// state is over (back to sub 0 and the decision); when the time is up the leap onto him (0x59A9E).
    pub(super) fn c61_sub4(&mut self, e: &mut Enemy, _h: Handle) {
        let t = e.m.w(o::TIMER).wrapping_sub(1);
        e.m.sw(o::TIMER, t);
        if t == 0 {
            e.b.sub = 0;
            return self.c61_59a9e(e);
        }
        let tp = self.c10_target(e);
        if self.players[tp].action == 0x40 {
            self.enemy_step(&mut e.b);
            return;
        }
        e.b.sub = 0;
        self.c61_5a186(e);
    }

    /// 0x5916C: act 8: when `+$AE` is out the Butcher stands up (act 0x18, 180 frames, the brake `+$1A` = 16), animation 1; the
    /// animation steps in every frame.
    fn c61_s08(&mut self, e: &mut Enemy) {
        let t = e.m.w(o::TIMER).wrapping_sub(1);
        e.m.sw(o::TIMER, t);
        if t == 0 {
            e.m.sb(0xA0, 1);
            e.b.act = 0x18;
            e.m.sw(o::TIMER, 0xB4);
            e.b.vz = 0;
            e.b.az = 0;
            e.b.vy = 0;
            e.b.ay = 0;
            e.b.vx = 0;
            e.b.ax = 0x10;
            self.enemy_anim(&mut e.b, 1);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x59208: act 2 (the proximity leap) and act 0x1A (the leap): the body slam attack starts when `+$AE` reaches 1; at 0 the
    /// flight: the apex aims at the hero (0x592EE), the landing shakes the screen (G.$755C = 4, sound 0x76) and bounces or ends in act 0xA.
    fn c61_s02(&mut self, e: &mut Enemy) {
        let ae = e.m.w(o::TIMER);
        if ae != 0 {
            if ae == 1 {
                self.enemy_begin_attack(&mut e.b, 0x2E);
            }
            e.m.sw(o::TIMER, ae.wrapping_sub(1));
            self.enemy_step(&mut e.b); // 0x592E8
            return;
        }
        if e.m.b(o::CC) == 0 && e.b.vy < 0 {
            return self.c61_592ee(e); // the apex
        }
        e.b.motion();
        if e.b.airborne == 0 {
            // 0x59258: landed
            e.b.set_yi_keep(0);
            self.cam_shake(4);
            let pan = self.snd_pan_x(e.b.xi()) as u32;
            self.snd_request(0x76, pan, 0); // 0xA288
            let bf = e.m.b(o::BF);
            if bf != 0 {
                e.m.sb(o::BF, bf - 1);
                return self.c61_59b62(e);
            }
            e.m.sw(o::TIMER, 0);
            e.b.act = 0xA;
            e.m.sb(0xA0, 1);
            e.b.airborne = 1;
            e.b.vx = 0x40;
            e.b.vy = 0x400;
            e.b.vz = 0;
            e.b.ax = 0;
            e.b.ay = 0xFFC0u16 as i16;
            e.b.az = 0;
            if e.b.facing == 0 {
                e.b.vx = e.b.vx.wrapping_neg();
            }
            return self.enemy_anim(&mut e.b, 0x18);
        }
        // 0x592CA: in the air
        if e.b.vy < 0 {
            self.enemy_anim(&mut e.b, 7);
        }
        if e.b.vy == 0 {
            e.b.ay = 0xFFBBu16 as i16;
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x592EE: the apex of a leap: `+$CC` = 1; the X and Z speeds from the distances kept at the start (G.-$704E, G.-$7052:
    /// a quarter of the way is already walked), the vertical speed 0, gravity 0xFF80.
    fn c61_592ee(&mut self, e: &mut Enemy) {
        e.m.sb(o::CC, 1);
        let d1 = c61_count((self.w.leap_dx as i32).wrapping_abs(), 0xD00) as i16;
        e.b.vx = if d1 >= 0x80 { d1 } else { 0x80 };
        let d1 = c61_count((self.w.leap_dz as i32).wrapping_abs(), 0xD00) as i16;
        e.b.vz = d1;
        e.b.vy = 0;
        e.b.ax = 0;
        e.b.ay = 0xFF80u16 as i16;
        e.b.az = 0;
        if e.b.facing == 0 {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        let tz = self.c61_tgt(e).zl;
        if e.b.z >= tz {
            e.b.vz = e.b.vz.wrapping_neg();
        }
    }

    /// 0x5937E: act 0x2C (the leap that carries the hero he holds): the hero's height and depth follow his; on the landing the hero
    /// is let go (`+$C8` and `+$2E` cleared), the screen shakes and the bounce (act 0xA) starts.
    fn c61_s2c(&mut self, e: &mut Enemy) {
        e.b.motion();
        let tp = self.c10_target(e);
        {
            let (y, z) = (e.b.yi(), e.b.zi());
            let p = &mut self.players[tp].b;
            p.set_yi_keep(y);
            p.set_zi_keep(z);
        }
        if e.b.airborne == 0 {
            self.enemy_begin_attack(&mut e.b, 0x2E);
            let p = &mut self.players[tp];
            p.action = 0;
            p.b.kind = 0;
            self.cam_shake(4);
            e.m.sw(o::TIMER, 0);
            e.b.act = 0xA;
            e.m.sb(0xA0, 1);
            e.b.airborne = 1;
            e.b.vx = 0x40;
            e.b.vy = 0x400;
            e.b.vz = 0;
            e.b.ax = 0;
            e.b.ay = 0xFFC0u16 as i16;
            e.b.az = 0;
            if e.b.facing == 0 {
                e.b.vx = e.b.vx.wrapping_neg();
            }
            e.b.set_yi_keep(0);
            return;
        }
        if e.b.vy < 0 {
            self.enemy_anim(&mut e.b, 7);
        }
        if e.b.vy == 0 {
            e.b.ay = 0xFFBBu16 as i16;
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x5943A: act 0xA, the body slam bounce: the motion; on the floor the decision (0x5A04E after a first leap, else a wait of 20 frames, act 4).
    fn c61_s0a(&mut self, e: &mut Enemy) {
        e.b.motion();
        if e.b.airborne == 0 {
            if e.m.b(o::AB) != 0 {
                return self.c61_5a04e(e);
            }
            e.m.sw(o::TIMER, 0x14);
            e.b.act = 4;
            self.enemy_anim(&mut e.b, 0);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x59564: act 0xC (the spin): the animation, at its end 0x5A008; the attack 0x33 repeats every fourth frame.
    fn c61_s0c(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            // 0x5A008: a hero who is not in the air ends it; in the air he is hit again from 0x5A014.
            let tg = self.lib_target(e).airborne;
            if tg == 0 {
                return self.c61_5a186(e);
            }
            return self.c61_5a014(e);
        }
        let t = e.m.w(o::TIMER).wrapping_add(1);
        e.m.sw(o::TIMER, t);
        if t & 3 == 0 {
            e.m.sb(ATTACKING, 0xFF);
            self.enemy_begin_attack(&mut e.b, 0x33);
        }
    }

    /// 0x5959E: act 0x10, the knife grab: when the animation ends, a new knife object A52 for the first and then the second knife
    /// (the dropped knife that he picked up is taken off the floor: `+$04` = 4, `+$26` = 0).
    fn c61_s10(&mut self, e: &mut Enemy, h: Handle) {
        if !self.enemy_step(&mut e.b) {
            return;
        }
        let kn = e.m.b(o::KNIVES);
        if kn < 2 {
            let second = kn != 0;
            let Some(a) = self.enemies.alloc() else { return self.c61_5a186(e) };
            let (x, y, z) = (e.b.xi(), e.b.yi(), e.b.zi());
            let handle = self.enemies.handle(a);
            let owner = super::class5::ptr(h);
            {
                let k = &mut self.enemies.slots[a];
                k.b.live = 1;
                k.b.k20 = 0xD0;
                k.b.var26 = if second { 0x100 } else { 0 };
                super::lib32::put_hi(&mut k.b.x, x);
                super::lib32::put_hi(&mut k.b.y, y);
                super::lib32::put_hi(&mut k.b.z, z);
                k.m.sl(0xA0, owner);
                k.b.anim_table = 0x6F8A6;
            }
            let (copy, knife) = if second { (o::KNIFE_B2, o::KNIFE_B) } else { (o::KNIFE_A2, o::KNIFE_A) };
            let old = e.m.w(copy);
            if let Some(Slot::Item(i)) = slot_of(old) {
                self.items.slots[i].b.var26 = 0;
                self.items.slots[i].b.mode = 4;
            } else if let Some(Slot::Enemy(i)) = slot_of(old) {
                self.enemies.slots[i].b.var26 = 0;
                self.enemies.slots[i].b.mode = 4;
            }
            e.m.sw(knife, handle);
            e.m.sw(copy, handle);
            e.m.sb(o::KNIVES, kn + 1);
        }
        self.c61_tail_knives(e); // 0x59670
        self.c61_59e08(e);
    }

    /// 0x59678: act 0x12, the dash (speed 13): every 32 frames a new direction toward the dropped knife he wants (`+$B4` / `+$B6`),
    /// else toward the hero; he faces the hero, moves, and picks a knife up on the way (0x58752).
    fn c61_s12(&mut self, e: &mut Enemy) {
        let t = e.m.w(o::TIMER).wrapping_sub(1);
        e.m.sw(o::TIMER, t);
        if t == 0 {
            return self.c61_5a186(e);
        }
        let a3 = e.m.b(o::A3).wrapping_sub(1);
        e.m.sb(o::A3, a3);
        if a3 == 0 {
            e.m.sb(o::A3, 0x20);
            e.m.sb(o::OLD_DIR, e.b.dir);
            if e.m.b(o::KNIVES) < 2 {
                let who = if e.m.b(o::KNIVES) == 1 { o::KNIFE_B2 } else { o::KNIFE_A2 };
                let knife = e.m.w(who);
                let (sub, kx, kz) = match slot_of(knife) {
                    Some(Slot::Item(i)) => {
                        let b = &self.items.slots[i].b;
                        (b.sub, b.xi(), b.zi())
                    }
                    Some(Slot::Enemy(i)) => {
                        let b = &self.enemies.slots[i].b;
                        (b.sub, b.xi(), b.zi())
                    }
                    _ => (0, 0, 0),
                };
                if sub == 8 {
                    return self.c61_5a186(e);
                }
                e.b.dir = Game::enemy_angle_to(&e.b, kx as u16, kz as u16);
            }
        }
        self.lib_face_target(e);
        self.enemy_move_dir(&mut e.b);
        self.c61_knife_pickup(e);
        self.enemy_step(&mut e.b);
    }
}

