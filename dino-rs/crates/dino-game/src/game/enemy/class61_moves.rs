//! The Butcher (class 61): the blocks that start a move (0x5988C .. 0x5A186). Each one sets the state byte `+$06` (act), its timer
//! `+$AE`, the attack record `+$58` and the speeds, then starts the animation. D3c section 3.3, BS09. Owner: enemy port (M6-10).
//!
//! The original is one routine with branches into the middle of other states, so the blocks keep their addresses as names.

use super::class61::{c61_count, o};
use super::{off::*, Enemy};
use crate::game::Game;

/// The integer words and long positions of the hero the Butcher hunts (`a0` after `movea.w $76(a6),a0`).
#[derive(Clone, Copy)]
pub(super) struct Tgt {
    pub x: i16,
    pub z: i16,
    pub xl: i32,
    pub zl: i32,
}

impl Game {
    pub(super) fn c61_tgt(&self, e: &Enemy) -> Tgt {
        let b = self.lib_target(e);
        Tgt { x: b.xi(), z: b.zi(), xl: b.x, zl: b.z }
    }

    /// G.$744 as a long (the camera X with its fraction).
    fn c61_cam_long(&self) -> i32 {
        (((self.cam.s2.cam_x as u32) << 16) | self.cam.s2.cam_x_frac as u32) as i32
    }

    /// 0x5988C: walk toward the slot point (`+$88`, `+$8A`): the step vectors `+$C0` / `+$C4`, 60 frames, act 0x18.
    pub(super) fn c61_5988c(&mut self, e: &mut Enemy) {
        e.m.sw(o::TIMER, 0x3C);
        let (dx, dz) = (e.m.w(DEST_X) as i16, e.m.w(DEST_Z) as i16);
        e.m.sl(0xC0, if dx > e.b.xi() { 0x15000 } else { 0xFFFE_B000 });
        e.m.sl(0xC4, if dz > e.b.zi() { 0x10000 } else { 0xFFFF_0000 });
        if e.b.act != 0x18 {
            self.enemy_anim(&mut e.b, 1);
        }
        e.b.act = 0x18;
    }

    /// 0x598F2: the hop away (after a hop animation ended, acts 0x14 / 0x16): act 2 in the air, a slot of the ring round the
    /// target (0x32AE4), speeds by the distance class from the tables 0x59988 / 0x59998 / 0x599A8.
    pub(super) fn c61_598f2(&mut self, e: &mut Enemy) {
        e.m.sb(0xA0, 1);
        e.m.sw(o::TIMER, 0x3C);
        e.b.act = 2;
        e.b.airborne = 1;
        let slot = self.lib_slot_ring(e, e.m.b(o::B0) as u16); // 0x32AE4
        e.b.facing = (e.b.dir < 0x10) as u8;
        e.m.sb(o::B0, slot as u8);
        let (dx, dz) = self.lib_distance_class(e); // 0x3293C
        let t = &self.assets.enemy.t;
        e.b.vx = t.i16(0x59988, 2 * dx as u32);
        e.b.vy = t.i16(0x59998, 2 * dx as u32);
        e.b.vz = t.i16(0x599A8, 2 * dz as u32);
        e.b.ax = 0;
        e.b.ay = 0xFFBBu16 as i16;
        e.b.az = 0;
        if e.b.facing == 0 {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        if self.rand() & 1 == 0 {
            e.b.vz = e.b.vz.wrapping_neg();
        }
        self.enemy_anim(&mut e.b, 7);
    }

    /// 0x599C8: the proximity leap (act 2): the body slam attack 0x2E, the target point 0x80 pixels to the side of the hero,
    /// the X speed from the absolute position (the original divides the point, not the distance: `vx` is almost always the cap 0x300).
    pub(super) fn c61_599c8(&mut self, e: &mut Enemy) {
        e.m.sb(0xA0, 1);
        e.m.sw(o::TIMER, 0xA);
        e.b.act = 2;
        e.b.airborne = 1;
        e.m.sb(o::BF, 0);
        self.enemy_begin_attack(&mut e.b, 0x2E);
        self.lib_aim_at_target(e); // 0x32A70
        let t = self.c61_tgt(e);
        let aim = if t.xl.wrapping_sub(self.c61_cam_long()) <= 0xC0_0000 { t.xl.wrapping_add(0x80_0000) } else { t.xl.wrapping_sub(0x80_0000) };
        let vx = c61_count(aim, 0x3800) as i16;
        e.b.vx = if vx > 0x300 { 0x300 } else { vx };
        e.b.vy = 0x700;
        e.b.ax = 0;
        e.b.ay = 0xFFC0u16 as i16;
        e.b.az = 0;
        e.b.vz = 0;
        if aim < e.b.x {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        self.enemy_begin_attack(&mut e.b, 0x2E);
        self.enemy_anim(&mut e.b, 0x10);
    }

    /// 0x59A9E: the leap onto the hero he holds (from the grab state 0x28, sub 4 to 0): act 0x2C straight up, the attack 0x2E.
    pub(super) fn c61_59a9e(&mut self, e: &mut Enemy) {
        e.b.sub = 0;
        e.m.sb(0xA0, 1);
        e.m.sw(o::TIMER, 0xA);
        e.b.act = 0x2C;
        e.b.airborne = 1;
        e.m.sb(o::BF, 0);
        self.enemy_begin_attack(&mut e.b, 0x2E);
        e.b.vx = 0;
        e.b.vy = 0x700;
        e.b.ax = 0;
        e.b.ay = 0xFFC0u16 as i16;
        e.b.az = 0;
        e.b.vz = 0;
        self.enemy_begin_attack(&mut e.b, 0x2E);
        self.enemy_anim(&mut e.b, 0x17);
    }

    /// 0x59B1E: the leap with the "first" flag `+$AB` (decision 8), then 0x59B24.
    pub(super) fn c61_59b1e(&mut self, e: &mut Enemy) {
        e.m.sb(o::AB, 1);
        self.c61_59b24(e);
    }

    /// 0x59B24: the leap crouch (act 0x2A, 10 frames, animation 0x10).
    pub(super) fn c61_59b24(&mut self, e: &mut Enemy) {
        e.m.sb(0xA0, 1);
        e.b.act = 0x2A;
        e.b.airborne = 1;
        e.m.sw(o::TIMER, 0xA);
        self.enemy_anim(&mut e.b, 0x10);
    }

    /// 0x59B44: the end of the crouch: animation 0x10 again, then the launch (0x59B6A).
    pub(super) fn c61_59b44(&mut self, e: &mut Enemy) {
        self.enemy_anim(&mut e.b, 0x10);
        self.c61_59b6a(e);
    }

    /// 0x59B50: the bounce on the hero (animation 0x16, `+$AE` = 0), then the launch (0x59B70).
    pub(super) fn c61_59b50(&mut self, e: &mut Enemy) {
        self.enemy_anim(&mut e.b, 0x16);
        e.m.sw(o::TIMER, 0);
        self.c61_59b70(e);
    }

    /// 0x59B62: a repeat bounce (animation 0x15), then the launch (0x59B6A).
    pub(super) fn c61_59b62(&mut self, e: &mut Enemy) {
        self.enemy_anim(&mut e.b, 0x15);
        self.c61_59b6a(e);
    }

    /// 0x59B6A: `+$AE` = 10, then the launch.
    fn c61_59b6a(&mut self, e: &mut Enemy) {
        e.m.sw(o::TIMER, 0xA);
        self.c61_59b70(e);
    }

    /// 0x59B70: the leap (act 0x1A). With no bounce count (`+$BF` = 0) he faces the hero and draws one from the table 0x59CFC
    /// (always 0 after the first decision with `+$AB`). The speeds: X and Z from a quarter off the way (the long distances
    /// are kept in G.-$704E and G.-$7052 for the apex, 0x592EE), `vy` 0x900, gravity 0xFFC0.
    fn c61_59b70(&mut self, e: &mut Enemy) {
        e.m.sb(0xA0, 1);
        e.b.act = 0x1A;
        e.b.airborne = 1;
        if e.m.b(o::BF) == 0 {
            self.lib_face_target(e);
            let r = (self.rand() & 0xF) as u32;
            let bf = self.assets.enemy.t.u8(0x59CFC, r);
            e.m.sb(o::BF, bf);
            if e.m.b(o::AB) != 0 {
                e.m.sb(o::BF, 0);
            }
        }
        self.lib_aim_at_target(e); // 0x32A70
        e.m.sb(o::CC, 0);
        let t = self.c61_tgt(e);
        let d0 = e.b.x.wrapping_sub(t.xl);
        let q = d0 >> 2;
        self.w.leap_dx = q as u32;
        let d0 = d0.wrapping_sub(q);
        let d1 = c61_count(d0.wrapping_abs(), 0x3800) as i16;
        e.b.vx = if d1 >= 0x80 { d1 } else { 0x80 };
        let d0 = e.b.z.wrapping_sub(t.zl);
        let q = d0 >> 2;
        self.w.leap_dz = q as u32;
        let d0 = d0.wrapping_sub(q);
        e.b.vz = c61_count(d0.wrapping_abs(), 0x3800) as i16;
        e.b.vy = 0x900;
        e.b.ax = 0;
        e.b.ay = 0xFFC0u16 as i16;
        e.b.az = 0;
        if e.b.facing == 0 {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        if e.b.z >= t.zl {
            e.b.vz = e.b.vz.wrapping_neg();
        }
    }

    /// 0x59D66: the far slash wind up (act 0x1E, 75 frames, animation 0x12) with the sound 0x59.
    pub(super) fn c61_59d66(&mut self, e: &mut Enemy) {
        let pan = self.snd_pan_x(e.b.xi()) as u32;
        self.snd_request(0x59, pan, 0); // 0xA2EE
        e.m.sb(0xA0, 1);
        e.b.act = 0x1E;
        e.m.sw(o::TIMER, 0x4B);
        self.enemy_anim(&mut e.b, 0x12);
    }

    /// 0x59D86: the far slash (act 0x20, 20 frames): the attack 0x2F (0x30 with no knife left), a step of 0x800 with the
    /// brake 0xFFC0 toward the facing side.
    pub(super) fn c61_59d86(&mut self, e: &mut Enemy) {
        e.m.sb(0xA0, 1);
        e.b.act = 0x20;
        e.m.sw(o::TIMER, 0x14);
        self.enemy_begin_attack(&mut e.b, 0x2F);
        e.b.vz = 0;
        e.b.az = 0;
        e.b.vy = 0;
        e.b.ay = 0;
        e.b.vx = 0x800;
        e.b.ax = 0xFFC0u16 as i16;
        if e.b.facing == 0 {
            e.b.vx = e.b.vx.wrapping_neg();
            e.b.ax = e.b.ax.wrapping_neg();
        }
        let mut anim = 0x13;
        if e.m.b(o::KNIVES) == 0 {
            self.enemy_begin_attack(&mut e.b, 0x30);
            anim = 0x14;
        }
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x59E08: act 0x34, 2 frames, animation 0 (after the knives were made again).
    pub(super) fn c61_59e08(&mut self, e: &mut Enemy) {
        e.m.sb(0xA0, 1);
        e.b.act = 0x34;
        e.m.sw(o::TIMER, 2);
        self.enemy_anim(&mut e.b, 0);
    }

    /// 0x59E22: act 0x24, 60 frames, animation 0x12.
    pub(super) fn c61_59e22(&mut self, e: &mut Enemy) {
        e.m.sb(0xA0, 1);
        e.b.act = 0x24;
        e.m.sw(o::TIMER, 0x3C);
        self.enemy_anim(&mut e.b, 0x12);
    }

    /// 0x59E3C: act 0x2E (the grab reach), 20 frames, animation 0x19.
    pub(super) fn c61_59e3c(&mut self, e: &mut Enemy) {
        e.m.sb(0xA0, 1);
        e.b.act = 0x2E;
        e.m.sw(o::TIMER, 0x14);
        self.enemy_anim(&mut e.b, 0x19);
    }

    /// 0x59E56: the heavy slash (act 0x26, 32 frames): the attack 0x31, the same step as 0x59D86, animation 0x13 (0x14 with no knife).
    pub(super) fn c61_59e56(&mut self, e: &mut Enemy) {
        e.m.sb(0xA0, 1);
        e.b.act = 0x26;
        e.m.sw(o::TIMER, 0x20);
        self.enemy_begin_attack(&mut e.b, 0x31);
        e.b.vz = 0;
        e.b.az = 0;
        e.b.vy = 0;
        e.b.ay = 0;
        e.b.vx = 0x800;
        e.b.ax = 0xFFC0u16 as i16;
        if e.b.facing == 0 {
            e.b.vx = e.b.vx.wrapping_neg();
            e.b.ax = e.b.ax.wrapping_neg();
        }
        let anim = if e.m.b(o::KNIVES) != 0 { 0x13 } else { 0x14 };
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x59EC0: the grab (from act 0x2E, `+$AE` out): the hero goes into the held state 0x02060000 (action 0x40, frame class 0xE),
    /// the Butcher is the partner, sub 4 (act 0x28), 120 frames.
    pub(super) fn c61_59ec0(&mut self, e: &mut Enemy, h: crate::game::Handle) {
        e.m.sb(0xA0, 1);
        let tp = self.c10_target(e);
        e.b.sub = 4;
        e.m.sw(o::TIMER, 0x78);
        e.b.partner = Game::player_handle(tp);
        e.b.kind = 0xC;
        e.b.act = 0x28;
        let p = &mut self.players[tp];
        p.b.partner = h;
        p.b.set_state(0x0206_0000);
        p.action = 0x40;
        p.b.kind = 0xE;
        p.bc7 = 0;
        p.b.hp_prev = p.b.hp;
    }

    /// 0x59F0C: a decision 1 to 4 (a combo, `d0` = 2 * decision): the attack 0x2D (0x1B with no knife is set before it; with
    /// a knife and the first animation 8 the attack 0x1B and then 0x2D again), the combo counters `+$BB` / `+$BC` and the first
    /// animation from the table 0x59FAA, a turn toward the hero.
    pub(super) fn c61_59f0c(&mut self, e: &mut Enemy, decision: u16) {
        let row = 4 * (decision as u32 - 1);
        self.enemy_begin_attack(&mut e.b, 0x2D);
        e.m.sb(0xA0, 1);
        e.b.act = 0;
        e.m.sw(o::TIMER, 0x3C);
        let t = &self.assets.enemy.t;
        e.m.sb(o::BB, t.u8(0x59FAA, row));
        e.m.sb(o::BC, t.u8(0x59FAA, row + 1));
        let mut anim = t.u8(0x59FAA, row + 2) as u16;
        if anim == 8 {
            self.enemy_begin_attack(&mut e.b, 0x1B);
            if e.m.b(o::KNIVES) != 0 {
                self.enemy_begin_attack(&mut e.b, 0x2D);
                anim = 0x11;
            }
        }
        self.c61_anim_or_windup(e, anim);
    }

    /// 0x59F8E: animation `anim`, or for 0x5A the far slash wind up (0x59D66); then the turn (0x32A88 with 0x50) and the face (0x32B68).
    pub(super) fn c61_anim_or_windup(&mut self, e: &mut Enemy, anim: u16) {
        if anim as u8 == 0x5A {
            return self.c61_59d66(e);
        }
        self.enemy_anim(&mut e.b, anim);
        self.lib_aim_beside_target(e, 0x50);
        self.lib_face_target(e);
    }

    /// 0x5A014: the counter (decision 5): act 0xC, the attack 0x33, animation 0xB. The spin is never chosen in play (D3c).
    pub(super) fn c61_5a014(&mut self, e: &mut Enemy) {
        e.m.sb(0xA0, 1);
        e.b.act = 0xC;
        self.enemy_begin_attack(&mut e.b, 0x33);
        self.enemy_anim(&mut e.b, 0xB);
    }

    /// 0x5A04E: the dash (act 0x12, 240 frames, speed 13, animation 0x1A); with both knives he goes on with the decision.
    pub(super) fn c61_5a04e(&mut self, e: &mut Enemy) {
        if e.m.b(o::KNIVES) == 2 {
            return self.c61_5a186(e);
        }
        self.enemy_set_speed(&mut e.b, 0xD);
        e.m.sw(o::TIMER, 0xF0);
        e.b.act = 0x12;
        self.enemy_anim(&mut e.b, 0x1A);
    }

    /// 0x5A186: the decision: speed 5, the distance class of the hero, then the walk (0x5988C, `+$BA` set) or the stand (0x59822).
    pub(super) fn c61_5a186(&mut self, e: &mut Enemy) {
        self.enemy_set_speed(&mut e.b, 5);
        e.m.sb(0xA0, 0);
        let (dx, _dz) = self.lib_distance_class(e); // 0x3293C
        e.m.sb(o::DIST_X, dx as u8);
        if e.m.b(o::BA) != 0 {
            self.c61_5988c(e);
        } else {
            self.c61_state_wait(e);
        }
    }
}
