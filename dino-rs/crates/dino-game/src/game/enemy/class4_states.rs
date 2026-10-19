//! Tyrog's states (`+$06`, table 0x36CA0) and actions (the decision 0x38B90 and the jump table 0x38CA6). D3d 3.4, 3.5. `class4.rs` has the
//! frame, the init, the recover entry, the hit and the defeat. Owner: enemy port.
//!
//! The states 0x20 and 0x28 (0x3752E, 0x37760 with the routines 0x3883E and 0x38098) and 0x42 (0x382DC) are set only by code that nothing
//! reaches; they are kept as the original has them where it is cheap (the state 0x20 bullet, class 59, is left out).

use super::class4::o;
use super::{off::*, Enemy};
use super::boss_common::BossAi;
use crate::game::{Game, Handle};

/// The tables of the states (read at run time through `enemy.t`).
mod t {
    pub const REST: u32 = 0x37C20;
    pub const CLAW_WAIT: u32 = 0x38214;
    pub const CHAIN_WAIT: u32 = 0x38290;
    pub const LAND_WAIT: u32 = 0x3868C;
    pub const LEAP_VY: u32 = 0x38754;
    pub const SIDE_X: u32 = 0x74BE2;
    pub const SIDE_Z: u32 = 0x74C62;
    pub const SIDE_VY: u32 = 0x74CA2;
    pub const CHAIN_A: u32 = 0x74EE2;
    pub const CHAIN_B: u32 = 0x74F22;
    pub const LEVEL_DELAY: u32 = 0x74B82;
    pub const LEVEL_DELAY_T2: u32 = 0x74BA2;
    pub const ROCKS: u32 = 0x37ECA;
}

impl Game {
    /// 0x36C7A: the state routine of `+$06`.
    pub(super) fn c4_state(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0x06 if Self::c4_is_t2(e) => self.c91_wait(e, h),
            0x02 => {
                // 0x36F6A
                let w = e.m.b(WAIT);
                if w != 0 {
                    e.m.sb(WAIT, w - 1);
                }
            }
            0x04 => self.c4_sidestep_air(e),
            0x08 => self.c4_walk_to_point(e, h),
            0x0A | 0x0C | 0x12 | 0x3E => {
                // 0x37670 / 0x37630 / 0x3764C: a timed animation, then the decision.
                if Self::c4_count(e) {
                    return self.c4_decide(e, h);
                }
                self.enemy_step(&mut e.b);
            }
            0x0E => {
                // 0x373AA
                if self.enemy_step(&mut e.b) {
                    self.c4_decide_clear(e, h);
                }
            }
            0x10 => self.c4_leap_air(e, h),
            0x14 => self.c4_drift(e, h),
            0x16 => {
                // 0x3774C
                if Self::c4_count(e) {
                    return self.c4_rest(e);
                }
                self.enemy_move_dir(&mut e.b);
                self.enemy_step(&mut e.b);
            }
            0x18 => {
                // 0x37666: the claw's wait (no animation), then its strike.
                if Self::c4_count(e) {
                    self.c4_crouch(e, 0x4F, 0x30, 0x14); // 0x385BA
                }
            }
            0x1A => {
                // 0x372AE
                if e.b.airborne == 0 {
                    return self.c4_decide(e, h);
                }
                e.b.motion();
                self.enemy_step(&mut e.b);
            }
            0x1C => {
                // 0x3738A
                if e.b.airborne == 0 {
                    // 0x37DF4
                    self.enemy_anim(&mut e.b, 0x21);
                    e.m.sb(o::BUSY, 1);
                    e.b.act = 0x52;
                    return;
                }
                e.b.motion();
                self.enemy_step(&mut e.b);
            }
            0x1E => self.c4_breath(e, h),
            0x22 => {
                // 0x37436
                if Self::c4_count(e) {
                    return self.c4_blade_a_start(e, h);
                }
                self.enemy_step(&mut e.b);
            }
            0x24 => self.c4_blade(e, h, 0),
            0x26 => self.c4_close_in(e, h),
            0x2A | 0x3C | 0x44 => self.c4_hop_air(e, h),
            0x2C => {
                // 0x373C2
                if self.enemy_step(&mut e.b) {
                    self.c4_leap_start(e);
                }
            }
            0x2E => {
                // 0x373CE
                if self.enemy_step(&mut e.b) {
                    self.c4_sidestep_start(e);
                }
            }
            0x30 => {
                // 0x373DA
                if self.enemy_step(&mut e.b) {
                    self.c4_jump_attack(e, h);
                }
            }
            0x32 => {
                // 0x373E6
                if self.enemy_step(&mut e.b) {
                    self.c4_pounce(e, h);
                }
            }
            0x34 => self.c4_pounce_air(e),
            0x36 => self.c4_lava(e, h),
            0x38 => self.c4_run_off(e, h),
            0x3A => self.c4_run_back(e, h),
            0x40 => {
                // 0x373F2: the wake-up claw strikes: attack 0x4F, state 0xE, animation 0x18.
                self.lib_face_target(e);
                self.enemy_begin_attack(&mut e.b, 0x4F);
                e.m.sb(o::BUSY, 1);
                e.b.act = 0x0E;
                e.m.sw(o::TIMER, 0x3C);
                e.m.sb(o::B9, 0);
                self.lib_aim_at_target(e); // 0x32A58
                self.enemy_anim(&mut e.b, 0x18);
            }
            0x42 => {
                // 0x382DC (nothing reaches it)
                self.lib_face_target(e);
                e.m.sb(o::BUSY, 1);
                e.m.sw(o::TIMER, 0x3C);
                e.m.sb(o::B9, 0);
            }
            0x46 => self.c4_blade(e, h, 0x100),
            0x48 => {
                // 0x373B6
                if self.enemy_step(&mut e.b) {
                    self.c4_chain_wait(e);
                }
            }
            0x4A => {
                // 0x3763E
                if Self::c4_count(e) {
                    return self.c4_chain_next(e, h);
                }
                self.enemy_step(&mut e.b);
            }
            0x4C => self.c4_leap_start(e),
            0x4E => {
                // 0x3765C
                if Self::c4_count(e) {
                    self.c4_chain_last(e, h);
                }
            }
            0x50 => {
                // 0x3783E
                if self.enemy_step(&mut e.b) {
                    self.c4_blade_b_cycle(e, h);
                }
            }
            0x52 => {
                // 0x3739E
                if self.enemy_step(&mut e.b) {
                    self.c4_decide(e, h);
                }
            }
            _ => {} // 0, 6 (table entries that point at the table itself), 0x20, 0x28: not reached
        }
    }

    /// The timer `+$AC` counted down: true when it reaches 0.
    fn c4_count(e: &mut Enemy) -> bool {
        let t = e.m.w(o::TIMER).wrapping_sub(1);
        e.m.sw(o::TIMER, t);
        t == 0
    }

    // ------------------------------------------------------------------ the decision

    /// 0x38B8A: no pending attack, then the decision.
    fn c4_decide_clear(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sb(o::PENDING, 0);
        self.c4_decide(e, h);
    }

    /// 0x38B90 / 0x38C36: the decision. A pending attack (`+$A4`): speed id 5, the X class (0x3293C), the action of `table[form][rand & 31][class]`
    /// (jump table 0x38CA6). None: `+$AC` 0x3C, speed id 5, the X class of 0x329B8, and by the phase flag the crouch of a recoil leap (aggressive,
    /// 0x3854A) or of a sidestep (defensive, 0x38582).
    pub(super) fn c4_decide(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sb(o::BUSY, 0);
        if e.m.b(o::PENDING) == 0 {
            // 0x38B9A
            e.m.sw(o::TIMER, 0x3C);
            self.enemy_set_speed(&mut e.b, 5);
            e.m.sb(o::BUSY, 0);
            // Tyrog 2 (0x67586) takes the class of 0x3293C.
            let xc = if Self::c4_is_t2(e) { self.lib_distance_class(e).0 } else { self.c4_classes(e).0 };
            e.m.sb(o::XCLASS, xc as u8);
            let st = if e.m.b(o::PHASE) & 1 != 0 { 0x2C } else { 0x2E };
            return self.c4_crouch(e, 0x4D, st, 0x14);
        }
        e.m.sb(o::PENDING, 0);
        self.enemy_set_speed(&mut e.b, 5);
        let (xc, _) = self.lib_distance_class(e); // 0x3293C
        e.m.sb(o::XCLASS, xc as u8);
        let action = self.boss_action_pick(e, Self::c4_ai(e));
        match action & 7 {
            0 | 1 => self.c4_claw(e),
            2 => self.c4_crouch(e, 0x4D, 0x32, 0x14), // 0x385F2
            3 => self.c4_hop(e, h, 0x3C, 0x800000, 2),  // 0x389D0
            4 => self.c4_lava_start(e, h),
            5 => self.c4_hop(e, h, 0x2A, 0x900000, 2),  // 0x388F8
            6 => self.c4_hop(e, h, 0x44, 0x800000, 3),  // 0x38AA8
            _ => self.c4_chain(e, h),
        }
    }

    /// The crouches 0x3854A / 0x38582 / 0x385BA / 0x385F2 / 0x3862A: face the target, an attack, busy, the state, `+$AC` 0x3C, the animation.
    fn c4_crouch(&mut self, e: &mut Enemy, atk: u8, state: u8, anim: u16) {
        self.lib_face_target(e);
        self.enemy_begin_attack(&mut e.b, atk);
        e.m.sb(o::BUSY, 1);
        e.b.act = state;
        e.m.sw(o::TIMER, 0x3C);
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x3862A: the off screen pounce's crouch (animation 0x26), also the recover entry's.
    pub(super) fn c4_pounce_crouch(&mut self, e: &mut Enemy, anim: u16) {
        self.c4_crouch(e, 0x4D, 0x32, anim);
    }

    /// 0x382B0: the wake-up claw: face, busy, state 0x40, `+$AC` 0x3C, `+$B9` 0, aim (0x32A58), animation 0x18.
    pub(super) fn c4_wake_claw(&mut self, e: &mut Enemy) {
        self.lib_face_target(e);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x40;
        e.m.sw(o::TIMER, 0x3C);
        e.m.sb(o::B9, 0);
        self.lib_aim_at_target(e);
        self.enemy_anim(&mut e.b, 0x18);
    }

    // ------------------------------------------------------------------ leaps and sidesteps

    /// 0x386F6: the recoil leap (state 0x10): in the air, aim and face, vy from 0x38754 by `rand() & 0x1E`, gravity -0x48, attack 0x4D,
    /// animation 0x15 (X and Z speeds are kept).
    fn c4_leap_start(&mut self, e: &mut Enemy) {
        e.m.sb(o::BUSY, 1);
        e.m.sw(o::TIMER, 0x3C);
        e.b.act = 0x10;
        e.b.airborne = 1;
        self.lib_aim_at_target(e); // 0x32A70
        self.lib_face_target(e);
        let r = (self.rand() & 0x1E) as u32;
        e.b.vy = self.assets.enemy.t.i16(t::LEAP_VY, r);
        e.b.ay = -0x48;
        self.enemy_begin_attack(&mut e.b, 0x4D);
        self.enemy_anim(&mut e.b, 0x15);
    }

    /// 0x36FDA: the recoil leap in the air (state 0x10): at the landing the wait (0x38662); else the flight, and the direction turned back into
    /// camera X .. + 0x180 (left of it 8, right of it 0x18 with the turn limit 0x37B86).
    fn c4_leap_air(&mut self, e: &mut Enemy, _h: Handle) {
        if e.b.airborne == 0 {
            return self.c4_land_wait(e);
        }
        e.b.motion();
        let cam = self.cam.s2.cam_x as i16;
        if e.b.xi().wrapping_sub(cam) < 0 {
            e.b.dir = 8;
        }
        if cam.wrapping_add(0x180).wrapping_sub(e.b.xi()) < 0 {
            e.b.dir = 0x18;
            self.c4_turn_limit(e);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x38662: the wait after a landing (state 0x3E): face, busy, `+$AC` from 0x3868C by `rand() & 0x1E`, animation 0x14.
    fn c4_land_wait(&mut self, e: &mut Enemy) {
        self.lib_face_target(e);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x3E;
        let r = (self.rand() & 0x1E) as u32;
        e.m.sw(o::TIMER, self.assets.enemy.t.u16(t::LAND_WAIT, r));
        self.enemy_anim(&mut e.b, 0x14);
    }

    /// 0x37BEE: the rest (state 0xA) after the third sidestep: `+$AB` 0, busy, `+$AC` from 0x37C20, animation 0, speed id 5.
    fn c4_rest(&mut self, e: &mut Enemy) {
        e.m.sb(o::STEPS, 0);
        e.m.sb(o::BUSY, 1);
        let r = (self.rand() & 0x1E) as u32;
        e.m.sw(o::TIMER, self.assets.enemy.t.u16(t::REST, r));
        e.b.act = 0x0A;
        self.enemy_anim(&mut e.b, 0);
        self.enemy_set_speed(&mut e.b, 5);
    }

    /// 0x37A10: the sidestep (state 4): aim 0x30 beside the target (0x32A88), face, in the air; the sideways and depth steps from the tables
    /// 0x74BE2 and 0x74C62 by the classes of 0x329B8 (with the registers it leaves), vy 0x400, gravity -0x60, animation 0x15.
    pub(super) fn c4_sidestep_start(&mut self, e: &mut Enemy) {
        if Self::c4_is_t2(e) {
            return self.c91_sidestep_start(e);
        }
        e.m.sb(o::OLD_DIR, e.b.dir);
        self.lib_aim_beside_target(e, 0x30);
        self.lib_face_target(e);
        e.m.sb(o::BUSY, 0);
        e.b.airborne = 1;
        let _ = self.rand(); // `rand() & 0xF` into d2 and d3, both lost in the call that follows
        let (xc, zc, d2, d3) = self.c4_classes(e);
        let tb = &self.assets.enemy.t;
        let bx = tb.u8(t::SIDE_X, (xc << 4).wrapping_add(d2) as u32);
        let _ = tb.u16(t::SIDE_VY, ((zc << 4).wrapping_add(d3)) as u32); // overwritten at once
        e.b.vy = 0x400;
        e.b.ay = -0x60;
        // `tst.w d2`: the byte under the high byte that 0x329B8 left.
        let w = (d2 & 0xFF00) | bx as u16;
        e.b.vx = if w == 0 {
            0
        } else if (w as i16) < 0 {
            -0x300
        } else {
            0x300
        };
        e.b.ax = 0;
        let tgt = self.lib_target(e);
        let (tx, tz) = (tgt.xi(), tgt.zi());
        if tx >= e.b.xi() {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        let r = (self.rand() & 0xF) as u16;
        let bz = self.assets.enemy.t.u8(t::SIDE_Z, ((zc << 4).wrapping_add(r)) as u32);
        e.b.vz = if bz == 0 {
            0
        } else if (bz as i8) < 0 {
            -0x300
        } else {
            0x300
        };
        e.b.az = 0;
        if tz >= e.b.zi() {
            e.b.vz = e.b.vz.wrapping_neg();
        }
        e.b.act = 4;
        e.m.sb(o::OLD_DIR, e.b.dir);
        self.enemy_anim(&mut e.b, 0x15);
    }

    /// 0x36F76: the sidestep in the air; at the landing the third one in a row rests (0x37BEE), the others wait (0x38662).
    fn c4_sidestep_air(&mut self, e: &mut Enemy) {
        if e.b.airborne != 0 {
            e.b.motion();
            self.enemy_step(&mut e.b);
            return;
        }
        let n = e.m.b(o::STEPS).wrapping_add(1);
        e.m.sb(o::STEPS, n);
        if n == 3 {
            self.c4_rest(e);
        } else {
            self.c4_land_wait(e);
        }
    }

    /// 0x37B86: the turn limit: the direction may move at most 2 steps away from `+$A9`.
    fn c4_turn_limit(&self, e: &mut Enemy) {
        let a9 = e.m.b(o::OLD_DIR);
        let d0 = (e.b.dir.wrapping_sub(a9) & 0x1F).wrapping_add(2) & 0x1F;
        if (d0 as i8) <= 4 {
            return;
        }
        let d0 = d0.wrapping_sub(2);
        e.b.dir = if (d0 as i8) > 0x10 { a9.wrapping_sub(2) & 0x1F } else { a9.wrapping_add(2) & 0x1F };
    }

    // ------------------------------------------------------------------ the claws

    /// 0x381C6 (actions 0 and 1): one claw: face, attack 0x4F, busy, state 0x18, `+$CC` 1, the wait from 0x38214, aim (0x32A58), animation 3.
    fn c4_claw(&mut self, e: &mut Enemy) {
        self.lib_face_target(e);
        self.enemy_begin_attack(&mut e.b, 0x4F);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x18;
        e.m.sw(o::REPEAT, 1);
        let r = (self.rand() & 0x1E) as u32;
        e.m.sw(o::TIMER, self.assets.enemy.t.u16(t::CLAW_WAIT, r));
        self.lib_aim_at_target(e);
        self.enemy_anim(&mut e.b, 3);
    }

    /// 0x38234 (action 7): the claw chain: face, attack 0x68, busy, state 0x18, `+$CC` 4, the wait from 0x38290; with `+$B9` set the chain
    /// goes on at once (0x37FDE); else aim, animation 3.
    fn c4_chain(&mut self, e: &mut Enemy, h: Handle) {
        self.lib_face_target(e);
        self.enemy_begin_attack(&mut e.b, 0x68);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x18;
        e.m.sw(o::REPEAT, 4);
        let r = (self.rand() & 0x1E) as u32;
        e.m.sw(o::TIMER, self.assets.enemy.t.u16(t::CHAIN_WAIT, r));
        if e.m.b(o::B9) != 0 {
            return self.c4_chain_last(e, h);
        }
        e.m.sb(o::B9, 0);
        self.lib_aim_at_target(e);
        self.enemy_anim(&mut e.b, 3);
    }

    /// 0x37FDE: with swipes left the next one (0x3802A); else the last claw: attack 0x4F, state 0xE, animation 3.
    fn c4_chain_last(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.w(o::REPEAT) != 0 {
            return self.c4_chain_next(e, h);
        }
        self.lib_face_target(e);
        self.enemy_begin_attack(&mut e.b, 0x4F);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x0E;
        e.m.sw(o::TIMER, 0x3C);
        e.m.sb(o::B9, 0);
        self.lib_aim_at_target(e);
        self.enemy_anim(&mut e.b, 3);
    }

    /// 0x3802A: one swipe less (none left: the decision); face, attack 0x4F, and 0x68 over it unless this is the last; state 0x48, animation 3.
    fn c4_chain_next(&mut self, e: &mut Enemy, h: Handle) {
        let n = e.m.w(o::REPEAT).wrapping_sub(1);
        e.m.sw(o::REPEAT, n);
        if n == 0 {
            return self.c4_decide(e, h);
        }
        self.lib_face_target(e);
        self.enemy_begin_attack(&mut e.b, 0x4F);
        if n != 1 {
            self.enemy_begin_attack(&mut e.b, 0x68);
        }
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x48;
        e.m.sw(o::TIMER, 0x3C);
        e.m.sb(o::B9, 0);
        self.lib_aim_at_target(e);
        self.enemy_anim(&mut e.b, 3);
    }

    /// 0x37986: the short wait between swipes (state 0x4A). The time from 0x74EE2 and 0x74F22 by level is computed and then set to 1 (the
    /// original writes 1 over it in both branches).
    fn c4_chain_wait(&mut self, e: &mut Enemy) {
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x4A;
        let r = (self.rand() & 0x3E) as u32;
        let tb = &self.assets.enemy.t;
        let a = tb.u16(t::CHAIN_A, r).wrapping_add(tb.u16(t::CHAIN_B, 2 * self.w.level as u32));
        let _ = a;
        e.m.sw(o::TIMER, 1);
    }

    // ------------------------------------------------------------------ hops, breath, blades

    /// 0x389D0 / 0x388F8 / 0x38AA8 (actions 3, 5, 6): a hop to a spot `side` (16.16) beside the target, toward the screen centre: busy,
    /// `+$AC` 0xA, the state, in the air, `+$CC`; the X and Z speeds are the distances / 0x2800 (counted, +1 when the count has bit 7), vy 0x380,
    /// gravity -0x40, animation 0x15. Action 6 jumps into action 3's loops after its first step (`bpl.w 0x38A2A`), so its own face call at
    /// 0x38B7C is never reached.
    pub(super) fn c4_hop(&mut self, e: &mut Enemy, _h: Handle, state: u8, side: i32, repeat: u16) {
        e.m.sb(o::BUSY, 1);
        e.m.sw(o::TIMER, 0xA);
        e.b.act = state;
        e.b.airborne = 1;
        e.m.sw(o::REPEAT, repeat);
        let tgt = self.lib_target(e).clone();
        let cam_l = ((self.cam.s2.cam_x as u32) << 16 | self.cam.s2.cam_x_frac as u32) as i32; // the long G.$744
        let d2 = if tgt.x.wrapping_sub(cam_l) > 0xC0_0000 { tgt.x.wrapping_sub(side) } else { tgt.x.wrapping_add(side) };
        let count = |d: i32, bump: bool| -> i16 {
            let mut d0 = d;
            if d0 < 0 {
                d0 = d0.wrapping_neg();
            }
            let mut n: u16 = 0;
            loop {
                n = n.wrapping_add(1);
                d0 = d0.wrapping_sub(0x2800);
                if d0 < 0 {
                    break;
                }
            }
            if bump && n & 0x80 != 0 {
                n = n.wrapping_add(1);
            }
            n as i16
        };
        let bump = true;
        e.b.vx = count(d2.wrapping_sub(e.b.x), bump);
        let d3 = tgt.z;
        e.b.vz = count(d3.wrapping_sub(e.b.z), bump);
        e.b.vy = 0x380;
        e.b.ax = 0;
        e.b.ay = -0x40;
        e.b.az = 0;
        if d2 < e.b.x {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        if d3 < e.b.z {
            e.b.vz = e.b.vz.wrapping_neg();
        }
        self.enemy_anim(&mut e.b, 0x15);
    }

    /// 0x3779C / 0x377D2 / 0x37808: a hop in the air: `+$AC` frames of flight first; then at the landing the action's next step (state 0x2A:
    /// 0x37C50, 0x3C: the breath 0x37E6E, 0x44: 0x37D78); at the top (vy 0) the gravity -0x45.
    fn c4_hop_air(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.w(o::TIMER) != 0 {
            e.m.sw(o::TIMER, e.m.w(o::TIMER) - 1);
        } else if e.b.airborne == 0 {
            return match e.b.act {
                0x2A => self.c4_blades_twice(e),
                0x3C => self.c4_breath_start(e, h, 7),
                _ => {
                    // 0x37D78
                    self.lib_face_target(e);
                    self.enemy_anim(&mut e.b, 0x21);
                    e.m.sb(o::BUSY, 1);
                    e.b.act = 0x50;
                }
            };
        } else if e.b.vy == 0 {
            e.b.ay = -0x45;
        }
        e.b.motion();
        self.enemy_step(&mut e.b);
    }

    /// The sound 0x82 of an attack start with the pan by X: d1 0x6201F when X - 0x1000 is 0xC0 or more (signed), else 0x5011F.
    fn c4_attack_sound(&mut self, e: &Enemy) {
        let d1 = if e.b.xi().wrapping_sub(0x1000) < 0xC0 { 0x5011F } else { 0x6201F };
        self.snd_request(0x82, d1, 0x540080);
    }

    /// 0x37C50 (action 5 at its landing): face, the sound 0x82, busy, `+$AC` 0xF, state 0x22, animation 6.
    fn c4_blades_twice(&mut self, e: &mut Enemy) {
        self.lib_face_target(e);
        self.c4_attack_sound(e);
        e.m.sb(o::BUSY, 1);
        e.m.sw(o::TIMER, 0xF);
        e.b.act = 0x22;
        self.enemy_anim(&mut e.b, 6);
    }

    /// 0x37C9E: one blade A more (none left: the decision): attack 0x4F, busy, state 0x24, `+$AC` 0x3C, the sound, animation 6.
    fn c4_blade_a_start(&mut self, e: &mut Enemy, h: Handle) {
        let n = e.m.w(o::REPEAT).wrapping_sub(1);
        e.m.sw(o::REPEAT, n);
        if n == 0 {
            return self.c4_decide(e, h);
        }
        self.enemy_begin_attack(&mut e.b, 0x4F);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x24;
        e.m.sw(o::TIMER, 0x3C);
        self.c4_attack_sound(e);
        self.enemy_anim(&mut e.b, 6);
    }

    /// 0x37D06 (state 0x50 at its end): animation 0x1B, then 0x37D0E: one more blade B (none left: the decision): attack 0x4F, busy, state 0x46,
    /// `+$AC` 0x3C, the sound.
    fn c4_blade_b_cycle(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_anim(&mut e.b, 0x1B);
        self.c4_blade_b_next(e, h);
    }

    /// 0x37D0E.
    fn c4_blade_b_next(&mut self, e: &mut Enemy, h: Handle) {
        let n = e.m.w(o::REPEAT).wrapping_sub(1);
        e.m.sw(o::REPEAT, n);
        if n == 0 {
            return self.c4_decide(e, h);
        }
        self.enemy_begin_attack(&mut e.b, 0x4F);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x46;
        e.m.sw(o::TIMER, 0x3C);
        self.c4_attack_sound(e);
    }

    /// 0x37444 (state 0x24, variant 0, blade A) / 0x374B4 (state 0x46, variant 0x100, blade B): the animation; on the frame byte `+$2D` = 1 a
    /// chest blade (class 74) 0x48 in front, 0x40 higher, 1 deeper, `+$A0` = this object. At the end: blade A the decision; blade B 0x37D6E
    /// (`+$CC` 2: animation 0x1C, then 0x37D0E) or 0x37D94 (animation 0x1D, attack 0x4F, state 0x24, the sound).
    fn c4_blade(&mut self, e: &mut Enemy, h: Handle, var: u16) {
        if self.enemy_step(&mut e.b) {
            if var == 0 {
                return self.c4_decide(e, h);
            }
            if e.m.w(o::REPEAT) == 2 {
                self.enemy_anim(&mut e.b, 0x1C);
                return self.c4_blade_b_next(e, h);
            }
            self.enemy_anim(&mut e.b, 0x1D);
            self.enemy_begin_attack(&mut e.b, 0x4F);
            e.m.sb(o::BUSY, 1);
            e.b.act = 0x24;
            e.m.sw(o::TIMER, 0x3C);
            self.c4_attack_sound(e);
            return;
        }
        if e.b.param_b != 1 {
            return;
        }
        let Some(j) = self.enemies.alloc() else { return };
        let facing = e.b.facing;
        let mut x = e.b.xi().wrapping_add(0x48);
        if facing == 0 {
            x = x.wrapping_sub(0x90);
        }
        let n = &mut self.enemies.slots[j];
        n.b.live = 1;
        n.b.k20 = 74 * 4;
        n.b.var26 = var;
        n.b.facing = facing;
        let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
        put(&mut n.b.x, x);
        put(&mut n.b.y, e.b.yi().wrapping_add(0x40));
        put(&mut n.b.z, e.b.zi().wrapping_add(1));
        n.m.sw(0xA0, h);
    }

    /// 0x37E6E (action 3 at its landing): with a live target: the rocks count from 0x37ECA (unused here), attack 0x4F, busy, state 0x1E,
    /// `+$AC` 0x3C, face, aim (0x32A58), animation 7.
    pub(super) fn c4_breath_start(&mut self, e: &mut Enemy, h: Handle, anim: u16) {
        if self.lib_target(e).live != 1 {
            return self.c4_decide(e, h);
        }
        let r = (self.rand() & 0xF) as u32;
        e.m.sb(o::ROCKS, self.assets.enemy.t.u8(t::ROCKS, r));
        self.enemy_begin_attack(&mut e.b, 0x4F);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x1E;
        e.m.sw(o::TIMER, 0x3C);
        self.lib_face_target(e);
        self.lib_aim_at_target(e);
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x3754E: the breath (state 0x1E): at the end the decision (0x38B8A); on frames with `+$2D` a fireball (class 26 variant 3) 0x24 behind
    /// (0x24 in front when facing right), 0x30 higher, `+$A0` = this object.
    fn c4_breath(&mut self, e: &mut Enemy, h: Handle) {
        if self.enemy_step(&mut e.b) {
            return self.c4_decide_clear(e, h);
        }
        if e.b.param_b == 0 {
            return;
        }
        let Some(j) = self.enemies.alloc() else { return };
        let facing = e.b.facing;
        let mut x = e.b.xi().wrapping_sub(0x24);
        if facing != 0 {
            x = x.wrapping_add(0x48);
        }
        let n = &mut self.enemies.slots[j];
        n.b.live = 1;
        n.b.k20 = 26 * 4;
        n.b.var26 = 3;
        n.b.facing = facing;
        let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
        put(&mut n.b.x, x);
        put(&mut n.b.y, e.b.yi().wrapping_add(0x30));
        put(&mut n.b.z, e.b.zi());
        n.m.sl(0xA0, 0xFFFF_0000 | h as u32);
    }

    /// 0x37F46 (action 4): with a live target eight rocks (`+$AA`), animation 0x17, then 0x37F62: attack 0x4F, busy, state 0x36, `+$AC` 0x3C,
    /// aim (0x32A58).
    fn c4_lava_start(&mut self, e: &mut Enemy, h: Handle) {
        if self.lib_target(e).live != 1 {
            return self.c4_decide(e, h);
        }
        e.m.sb(o::ROCKS, 8);
        self.enemy_anim(&mut e.b, 0x17);
        self.c4_lava_arm(e);
    }

    /// 0x37F62.
    fn c4_lava_arm(&mut self, e: &mut Enemy) {
        self.enemy_begin_attack(&mut e.b, 0x4F);
        e.m.sb(o::BUSY, 1);
        e.b.act = 0x36;
        e.m.sw(o::TIMER, 0x3C);
        self.lib_aim_at_target(e);
    }

    /// 0x375B8: the spit (state 0x36): at the end of the animation a lava rock (class 26 variant 4) 0x3C behind (0x3C in front when facing right),
    /// 0x60 higher, direction (rocks left - 1) * 4, `+$A0` = this object; one rock less: none left the decision (0x38B8A), else 0x37F62.
    fn c4_lava(&mut self, e: &mut Enemy, h: Handle) {
        if !self.enemy_step(&mut e.b) {
            return;
        }
        if let Some(j) = self.enemies.alloc() {
            let facing = e.b.facing;
            let mut x = e.b.xi().wrapping_sub(0x3C);
            if facing != 0 {
                x = x.wrapping_add(0x78);
            }
            let dir = e.m.b(o::ROCKS).wrapping_sub(1).wrapping_mul(4);
            let n = &mut self.enemies.slots[j];
            n.b.live = 1;
            n.b.k20 = 26 * 4;
            n.b.var26 = 4;
            n.b.facing = facing;
            let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
            put(&mut n.b.x, x);
            put(&mut n.b.y, e.b.yi().wrapping_add(0x60));
            put(&mut n.b.z, e.b.zi());
            n.b.dir = dir;
            n.m.sl(0xA0, 0xFFFF_0000 | h as u32);
        }
        let n = e.m.b(o::ROCKS).wrapping_sub(1);
        e.m.sb(o::ROCKS, n);
        if n == 0 {
            return self.c4_decide_clear(e, h);
        }
        self.c4_lava_arm(e);
    }

    // ------------------------------------------------------------------ jump attack and pounce

    /// 0x382FC (state 0x30 at its end): the jump attack (state 0x1A): busy, in the air, attack 0x4D, aim; with a live target the speeds: X the
    /// distance to 0x30 px in front of the target / 0x3800 (counted), Z the distance / 0x3800, vy 0x700, gravity -0x40; signs toward the target;
    /// attack 0x4D again, animation 0x15.
    fn c4_jump_attack(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sb(o::BUSY, 1);
        e.m.sw(o::TIMER, 0x3C);
        e.b.act = 0x1A;
        e.b.airborne = 1;
        self.enemy_begin_attack(&mut e.b, 0x4D);
        self.lib_aim_at_target(e); // 0x32A70
        if (e.m.b(o::XCLASS) as i8) <= 5 {
            e.b.dir &= 0x10;
        }
        let tgt = self.lib_target(e).clone();
        if tgt.live != 1 {
            return self.c4_decide(e, h);
        }
        let mut d0 = e.b.x.wrapping_sub(tgt.x);
        d0 = if e.b.facing == 0 { d0.wrapping_sub(0x30_0000) } else { d0.wrapping_add(0x30_0000) };
        e.b.vx = Self::c4_steps(d0, 0x3800);
        e.b.vz = Self::c4_steps(e.b.z.wrapping_sub(tgt.z), 0x3800);
        e.b.vy = 0x700;
        e.b.ax = 0;
        e.b.ay = -0x40;
        e.b.az = 0;
        if e.b.facing == 0 {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        if e.b.z >= tgt.z {
            e.b.vz = e.b.vz.wrapping_neg();
        }
        self.enemy_begin_attack(&mut e.b, 0x4D);
        self.enemy_anim(&mut e.b, 0x15);
    }

    /// The counted division of the original: n + 1 where n steps of `step` fit in |d| (a 32 bit count, kept as a word).
    fn c4_steps(d: i32, step: i32) -> i16 {
        let mut d0 = d;
        if d0 < 0 {
            d0 = d0.wrapping_neg();
        }
        let mut n: u16 = 0;
        loop {
            n = n.wrapping_add(1);
            d0 = d0.wrapping_sub(step);
            if d0 < 0 {
                return n as i16;
            }
        }
    }

    /// 0x3841C (state 0x32 at its end): the pounce (state 0x34): busy, in the air, `+$C2` 0, attack 0x4D, aim; with a live target a quarter of
    /// the distance in `+$C4` / `+$C8`, the rest / 0x1E00 counted as the speeds (X at least 0x80), vy 0x800, gravity -0x44, signs toward the
    /// target; attack 0x4D again, animation 2.
    fn c4_pounce(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sb(o::BUSY, 1);
        e.m.sw(o::TIMER, 0x3C);
        e.b.act = 0x34;
        e.b.airborne = 1;
        e.m.sb(o::APEX, 0);
        self.enemy_begin_attack(&mut e.b, 0x4D);
        self.lib_aim_at_target(e);
        if (e.m.b(o::XCLASS) as i8) <= 5 {
            e.b.dir &= 0x10;
        }
        let tgt = self.lib_target(e).clone();
        if tgt.live != 1 {
            return self.c4_decide(e, h);
        }
        let dx = e.b.x.wrapping_sub(tgt.x);
        let qx = dx >> 2;
        e.m.sl(o::QX, qx as u32);
        let mut vx = Self::c4_steps(dx.wrapping_sub(qx), 0x1E00);
        if vx < 0x80 && !Self::c4_is_t2(e) {
            vx = 0x80; // (not in Tyrog 2: 0x3731A, 0x384B6)
        }
        e.b.vx = vx;
        let dz = e.b.z.wrapping_sub(tgt.z);
        let qz = dz >> 2;
        e.m.sl(o::QZ, qz as u32);
        e.b.vz = Self::c4_steps(dz.wrapping_sub(qz), 0x1E00);
        e.b.vy = 0x800;
        e.b.ax = 0;
        e.b.ay = -0x44;
        e.b.az = 0;
        if e.b.facing == 0 {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        if e.b.z >= tgt.z {
            e.b.vz = e.b.vz.wrapping_neg();
        }
        self.enemy_begin_attack(&mut e.b, 0x4D);
        self.enemy_anim(&mut e.b, 2);
    }

    /// 0x372C2: the pounce in the air (state 0x34): at the landing animation 0x21 and the rest (0x37BEE); at the top (vy below 0, once) animation
    /// 0x19 and the speeds of the last quarter (0x372F2: `+$C4` / `+$C8` / 0xF00 counted, X at least 0x80, vy 0, gravity -0x88); the flight.
    fn c4_pounce_air(&mut self, e: &mut Enemy) {
        if e.b.airborne == 0 {
            self.enemy_anim(&mut e.b, 0x21);
            return self.c4_rest(e);
        }
        if e.m.b(o::APEX) == 0 && e.b.vy < 0 {
            self.enemy_anim(&mut e.b, 0x19);
            e.m.sb(o::APEX, 1);
            let mut vx = Self::c4_steps(e.m.l(o::QX) as i32, 0xF00);
            if vx < 0x80 && !Self::c4_is_t2(e) {
                vx = 0x80; // (not in Tyrog 2: 0x3731A, 0x384B6)
            }
            e.b.vx = vx;
            e.b.vz = Self::c4_steps(e.m.l(o::QZ) as i32, 0xF00);
            e.b.vy = 0;
            e.b.ax = 0;
            e.b.ay = -0x88;
            e.b.az = 0;
            if e.b.facing == 0 {
                e.b.vx = e.b.vx.wrapping_neg();
            }
            let tz = self.lib_target(e).z;
            if e.b.z >= tz {
                e.b.vz = e.b.vz.wrapping_neg();
            }
            return;
        }
        e.b.motion();
        self.enemy_step(&mut e.b);
    }

    // ------------------------------------------------------------------ walks (states 8, 0x14, 0x26, 0x38, 0x3A)

    /// 0x3701E (state 8): toward the point (`+$88`, `+$8A`) at 2.5 / 2 px per frame (the speeds set every 4 frames), with the step delay `+$A2`
    /// (0x36D76); at the end of `+$AC` the decision; at the point (8 round it) no move (`+$A3` = 1).
    fn c4_walk_to_point(&mut self, e: &mut Enemy, h: Handle) {
        if Self::c4_count(e) {
            self.lib_face_target(e);
            return self.c4_decide(e, h);
        }
        if e.m.w(o::TIMER) & 3 == 0 {
            let vx: u32 = if e.m.w(DEST_X) as i16 > e.b.xi() { 0x28000 } else { 0xFFFD_8000 };
            e.m.sl(o::VX, vx);
            let vz: u32 = if e.m.w(DEST_Z) as i16 > e.b.zi() { 0x20000 } else { 0xFFFE_0000 };
            e.m.sl(o::VZ, vz);
        }
        self.c4_walk_step(e, e.m.w(DEST_X) as i16, 8, 0x10, e.m.w(DEST_Z) as i16, false);
    }

    /// The step of the walks: the delay `+$A2`; X moves while |point + `ax` - X| is `bx` or more, Z while |point Z + 8 - Z| is 0x10 or more
    /// (unsigned windows); a move reloads the delay (0x36D76) and animates when it is 0; no move: `+$A3` 1 and `+$A2` 0 (state 8) or the claw
    /// (state 0x26). Returns true when it did not move.
    fn c4_walk_step(&mut self, e: &mut Enemy, px: i16, ax: i16, bx: u16, pz: i16, _claw: bool) -> bool {
        let d = e.m.b(o::DELAY);
        if d != 0 {
            e.m.sb(o::DELAY, d - 1);
            return false;
        }
        let mut moved = false;
        if (px.wrapping_add(ax).wrapping_sub(e.b.xi()) as u16) >= bx {
            e.b.x = e.b.x.wrapping_add(e.m.l(o::VX) as i32);
            moved = true;
        }
        if (pz.wrapping_add(8).wrapping_sub(e.b.zi()) as u16) >= 0x10 {
            e.b.z = e.b.z.wrapping_add(e.m.l(o::VZ) as i32);
            moved = true;
        }
        if moved {
            // 0x36D76: the delay from 0x74B82 by level while `+$A3` is set.
            if e.m.b(o::RELOAD) != 0 {
                let table = if Self::c4_is_t2(e) { t::LEVEL_DELAY_T2 } else { t::LEVEL_DELAY }; // 0x657A8
                let v = self.assets.enemy.t.u8(table, self.w.level as u32);
                e.m.sb(o::DELAY, v);
                e.m.sb(o::RELOAD, 0);
            }
            if e.m.b(o::DELAY) == 0 {
                self.enemy_step(&mut e.b);
            }
            return false;
        }
        true
    }

    /// 0x370EC (state 0x26): toward 0x20 px in front of the target; close (0x40 / 0x10 windows) the claw (0x381C6).
    fn c4_close_in(&mut self, e: &mut Enemy, h: Handle) {
        if Self::c4_count(e) {
            self.lib_face_target(e);
            return self.c4_decide(e, h);
        }
        let tgt = self.lib_target(e).clone();
        if e.m.w(o::TIMER) & 3 == 0 {
            let mut d0 = tgt.xi().wrapping_add(0x20);
            if e.b.facing != 0 {
                d0 = d0.wrapping_sub(0x20);
            }
            let vx: u32 = if d0 > e.b.xi() { 0x28000 } else { 0xFFFD_8000 };
            e.m.sl(o::VX, vx);
            let vz: u32 = if tgt.zi() > e.b.zi() { 0x20000 } else { 0xFFFE_0000 };
            e.m.sl(o::VZ, vz);
        }
        if self.c4_walk_step(e, tgt.xi(), 0x20, 0x40, tgt.zi(), true) {
            self.c4_claw(e);
        }
    }

    /// 0x36F9E (state 0x14): Z drifts by `+$B4`; every 8 frames the stuck test (0x324EE): not stuck the decision, stuck `+$AC` down and at 0 the
    /// drift turns; face, animate.
    fn c4_drift(&mut self, e: &mut Enemy, h: Handle) {
        let n = e.m.b(o::A8).wrapping_sub(1);
        e.m.sb(o::A8, n);
        if n == 0 {
            e.m.sb(o::A8, 8);
            if !Self::enemy_stuck_check(e) {
                return self.c4_decide(e, h);
            }
            if Self::c4_count(e) {
                let v = (e.m.l(o::VZ) as i32).wrapping_neg();
                e.m.sl(o::VZ, v as u32);
            }
        }
        e.b.z = e.b.z.wrapping_add(e.m.l(o::VZ) as i32);
        self.lib_face_target(e);
        self.enemy_step(&mut e.b);
    }

    /// 0x3784A (state 0x38): the run off the screen: every 12 frames aim and the turn limit; facing by the direction; move and animate; at the end
    /// of `+$AC` 0x38162.
    fn c4_run_off(&mut self, e: &mut Enemy, h: Handle) {
        if Self::c4_count(e) {
            return self.c4_run_back_start(e, h);
        }
        let n = e.m.b(o::A8).wrapping_sub(1);
        e.m.sb(o::A8, n);
        if n == 0 {
            e.m.sb(o::A8, 0xC);
            e.m.sb(o::OLD_DIR, e.b.dir);
            self.lib_aim_at_target(e);
            self.c4_turn_limit(e);
        }
        e.b.facing = ((e.b.dir as i8) <= 0x10) as u8;
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x38162: on screen the decision; else state 0x3A (busy off, `+$AC` 0x64) with the speeds toward the point, animation 1.
    fn c4_run_back_start(&mut self, e: &mut Enemy, h: Handle) {
        if !self.lib_off_screen(&e.b) {
            return self.c4_decide(e, h);
        }
        e.m.sb(o::BUSY, 0);
        e.m.sw(o::TIMER, 0x64);
        e.b.act = 0x3A;
        let vx: u32 = if e.m.w(DEST_X) as i16 > e.b.xi() { 0x24000 } else { 0xFFFD_C000 };
        e.m.sl(o::VX, vx);
        let vz: u32 = if e.m.w(DEST_Z) as i16 > e.b.zi() { 0x1C000 } else { 0xFFFE_4000 };
        e.m.sl(o::VZ, vz);
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x37894 (state 0x3A): toward the target at 2.25 / 1.75 px per frame; facing by the direction; at the end of `+$AC` 0x380D6.
    fn c4_run_back(&mut self, e: &mut Enemy, h: Handle) {
        if Self::c4_count(e) {
            return self.c4_run_off_start(e, h);
        }
        let tgt = self.lib_target(e).clone();
        e.b.x = e.b.x.wrapping_add(if tgt.xi() > e.b.xi() { 0x24000 } else { 0xFFFD_C000u32 as i32 });
        e.b.z = e.b.z.wrapping_add(if tgt.zi() > e.b.zi() { 0x1C000 } else { 0xFFFE_4000u32 as i32 });
        e.b.facing = ((e.b.dir as i8) <= 0x10) as u8;
        self.enemy_step(&mut e.b);
    }

    /// 0x380D6: on screen the decision; else the run off (state 0x38, `+$AC` 0x64, `+$A8` 0x1E): a slot, aim, the direction straight left or
    /// right by the aim (and 4 in 16 turned round), speed id 0xB, animation 1.
    fn c4_run_off_start(&mut self, e: &mut Enemy, h: Handle) {
        if !self.lib_off_screen(&e.b) {
            return self.c4_decide(e, h);
        }
        e.b.act = 0x38;
        e.m.sw(o::TIMER, 0x64);
        e.m.sb(o::A8, 0x1E);
        self.lib_slot6_new(e);
        e.m.sb(o::OLD_DIR, e.b.dir);
        self.lib_aim_at_target(e);
        let d0 = e.b.dir.wrapping_add(8) & 0x10;
        if (d0 as i8) < 0x10 {
            e.b.facing = 1;
            e.b.dir = 0;
        } else {
            e.b.facing = 0;
            e.b.dir = 0x10;
        }
        let r = self.rand() & 0xF;
        if r > 0xC {
            e.b.dir = e.b.dir.wrapping_add(0x10) & 0x10;
        }
        self.enemy_set_speed(&mut e.b, 0xB);
        self.enemy_anim(&mut e.b, 1);
    }
}
