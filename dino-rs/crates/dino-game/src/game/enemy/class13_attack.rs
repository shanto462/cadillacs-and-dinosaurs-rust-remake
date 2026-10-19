//! Class 13: the attack states and their entries: the dash tackle (state 0x44 wind up, 0x1C, 0x1E), the rifle volley (state 0x10) with the
//! bullet (class 59) and the muzzle flash (class 57), the melee chain (states 0x24 and 0x2C), the kick (states 0x42 and 0x26) and the hop back
//! after a block (state 0x28). See `class13.rs`.
//!
//! Every attack sets `+$58` (attack id), clears `+$5A` / `+$59` and takes the next attack serial G.$67BE into `+$5E` (`enemy_begin_attack`).

use super::class13::{f::*, t};
use super::Enemy;
use crate::game::Game;

impl Game {
    // ------------------------------------------------------------------ the dash tackle

    /// 0x3F82A (far decision value 6): the dash tackle is dropped (decide again) unless the target is more than 100 px away in X; then the wind
    /// up (0x3F856: attack 0x2C armed, state 0x44, 30 frames, anim 0x1A).
    pub(super) fn enemy_c13_3f82a(&mut self, e: &mut Enemy) {
        let tx = self.lib_target(e).xi();
        let x = e.b.xi();
        let near = if tx >= x { tx.wrapping_sub(0x64) < x } else { tx.wrapping_add(0x64) > x };
        if near {
            return self.enemy_c13_4005c(e);
        }
        self.enemy_begin_attack(&mut e.b, 0x2C); // 0x3F856
        e.m.sb(BUSY, 1);
        e.m.sb(COMMIT, 1);
        e.b.act = 0x44;
        e.m.sw(TIMER, 0x1E);
        self.enemy_anim(&mut e.b, 0x1A);
    }

    /// 0x3F88E (the end of the wind up, and the entrance 8): the dash (state 0x1C, 55 frames, speed id 0xD, anim 0x15) at the target; attack 0x2C.
    pub(super) fn enemy_c13_3f88e(&mut self, e: &mut Enemy) {
        self.enemy_begin_attack(&mut e.b, 0x2C);
        e.m.sb(BUSY, 1);
        e.m.sb(COMMIT, 1);
        e.b.act = 0x1C;
        e.m.sw(TIMER, 0x37);
        self.enemy_set_speed(&mut e.b, 0xD);
        self.lib_aim_at_target(e); // 0x32A58
        self.lib_face_target(e);
        e.b.dir = 0x18;
        if e.b.facing != 0 {
            e.b.dir = 8;
        }
        self.enemy_anim(&mut e.b, 0x15);
    }

    /// 0x3EB30 (state 0x44): wait out the wind up, then the dash.
    pub(super) fn enemy_c13_3eb30(&mut self, e: &mut Enemy) {
        let t = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, t);
        if t == 0 {
            self.enemy_c13_3f88e(e);
        }
    }

    /// 0x3EC22 (state 0x1C): the dash. While the target is farther than 0x80 px (X window) it runs at the speed table; when the time is out
    /// or the target is near, the tackle ends in a slide (state 0x1E, vx 0x300 slowed by 0x20 a frame, anim 0x16) with a new attack serial.
    pub(super) fn enemy_c13_3ec22(&mut self, e: &mut Enemy) {
        let t = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, t);
        let mut slide = t == 0;
        if !slide {
            let tx = self.lib_target(e).xi();
            let d0 = tx.wrapping_add(0x40).wrapping_sub(e.b.xi()) as u16;
            slide = d0 <= 0x80;
        }
        if slide {
            e.b.vz = 0;
            e.b.az = 0;
            e.b.vy = 0;
            e.b.ay = 0;
            e.b.vx = 0x300;
            e.b.ax = 0xFFE0u16 as i16;
            if e.b.facing == 0 {
                e.b.vx = e.b.vx.wrapping_neg();
                e.b.ax = e.b.ax.wrapping_neg();
            }
            e.m.sw(TIMER, 0x10);
            e.b.act = 0x1E;
            self.enemy_begin_attack(&mut e.b, 0x2C);
            self.enemy_anim(&mut e.b, 0x16);
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x3ECA8 (state 0x1E): the slide. When it ends: a pause (state 0x18, the time from the table 0x3ECE2 by `rand & 0x1F`), anim 0.
    pub(super) fn enemy_c13_3eca8(&mut self, e: &mut Enemy) {
        let t = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, t);
        if t == 0 {
            let r = (self.rand() & 0x1F) as u32;
            let v = self.assets.enemy.t.u8(0x3ECE2, r) as u16;
            e.m.sw(TIMER, v);
            e.m.sb(BUSY, 1);
            e.b.act = 0x18;
            self.enemy_anim(&mut e.b, 0);
        }
        e.b.motion(); // 0x189C
        self.enemy_step(&mut e.b);
    }

    // ------------------------------------------------------------------ the rifle volley

    /// 0x3F6B2 (far decision value 7): the number of shots (the table 0x3F6E8 by `rand & 0x1E`: 1, 2 or 3) and the first shot.
    pub(super) fn enemy_c13_3f6b2(&mut self, e: &mut Enemy) {
        let r = (self.rand() & 0x1E) as u32;
        let n = self.assets.enemy.t.u16(0x3F6E8, r);
        e.m.sw(SHOTS, n);
        self.enemy_c13_3f6c2(e);
    }

    /// 0x3F6C2: one shot: state 0x10, 60 frames, aim and face the target, anim 0xF.
    fn enemy_c13_3f6c2(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 1);
        e.m.sw(TIMER, 0x3C);
        e.b.act = 0x10;
        self.lib_aim_at_target(e); // 0x32A58
        self.lib_face_target(e);
        self.enemy_anim(&mut e.b, 0xF);
    }

    /// 0x3EA2A (state 0x10): the shot. The frame with the flag `+$2D` fires. When the animation ends (69 ticks): a pause (state 0x18, the time
    /// of the table 0x3EADA), `+$C4` -= 1; with shots left the next one starts at once, else the flee timer `+$B8` = -90.
    pub(super) fn enemy_c13_3ea2a(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            e.m.sb(BUSY, 1); // 0x3EA3E
            e.b.act = 0x18;
            self.enemy_anim(&mut e.b, 0);
            let r = (self.rand() & 0x1F) as u32;
            let v = self.assets.enemy.t.u8(t::PAUSE_AFTER_SHOT, r) as u16;
            e.m.sw(TIMER, v);
            let n = e.m.w(SHOTS).wrapping_sub(1);
            e.m.sw(SHOTS, n);
            if (n as i16) > 0 {
                return self.enemy_c13_3f6c2(e);
            }
            e.m.sw(FLEE_T, 0xFFA6);
            return;
        }
        if e.b.param_b != 0 {
            self.enemy_c13_3ea78(e);
        }
    }

    /// 0x3EA78: fire. The bullet (class 59, variant 0) 0x30 px in front and 0x40 up (0x57F62), the shot sound (0x9C76) and the muzzle flash
    /// (class 57, variant 1) at +0x2B / +0x44, on the other side for a left facing object; its `+$24` is the negated facing.
    fn enemy_c13_3ea78(&mut self, e: &mut Enemy) {
        self.enemy_spawn_bullet(e, 0, 0x30, 0x40); // 0x57F62
        self.enemy_c13_snd_9c76(e);
        let Some(j) = self.enemies.alloc() else { return }; // 0x483C
        let n = &mut self.enemies.slots[j].b;
        n.live = 1;
        n.k20 = 0xE4;
        n.var26 = 1;
        n.set_xi_keep(e.b.xi());
        n.set_yi_keep(e.b.yi());
        n.set_zi_keep(e.b.zi());
        n.set_xi_keep(n.xi().wrapping_add(0x2B));
        n.set_yi_keep(n.yi().wrapping_add(0x44));
        n.facing = e.b.facing.wrapping_neg();
        if n.facing == 0 {
            n.set_xi_keep(n.xi().wrapping_sub(0x56));
        }
    }

    /// 0x9C76 for a pool A object (`+$82` is not 0): the code 0xEA with d1 = (3, or 4 for a right facing object) << 16 | 0x2000 | the pan
    /// byte of its X, d2 = 0x15000000.
    fn enemy_c13_snd_9c76(&mut self, e: &Enemy) {
        let mode: u32 = if e.b.facing == 0 { 3 } else { 4 };
        let pan = self.snd_pan_x(e.b.xi()) as u32;
        self.snd_request(0xEA, (mode << 16) | 0x2000 | pan, 0x1500_0000);
    }

    // ------------------------------------------------------------------ the melee

    /// 0x3F998 (far decision values 1, 3, 4, 5): the melee chain: attack 8, state 0x24 (60 frames); the row of the table 0x3FA3E (by `rand & 0xF`)
    /// gives the kind `+$BA`, the hits `+$BB` and the animation (0xC: attack 9, 0xD: attack 0x2B, new serials).
    pub(super) fn enemy_c13_3f998(&mut self, e: &mut Enemy) {
        self.enemy_begin_attack(&mut e.b, 8);
        e.m.sb(BUSY, 1);
        e.m.sb(COMMIT, 1);
        e.b.act = 0x24;
        e.m.sw(TIMER, 0x3C);
        self.lib_aim_at_target(e); // 0x32A58
        self.lib_face_target(e);
        let r = ((self.rand() & 0xF) * 4) as u32;
        let tb = &self.assets.enemy.t;
        let (a, n, anim) = (tb.u8(0x3FA3E, r), tb.u8(0x3FA3E, r + 1), tb.u8(0x3FA3E, r + 2) as u16);
        e.m.sb(MELEE_A, a);
        e.m.sb(MELEE_N, n);
        if anim == 0xC {
            self.enemy_begin_attack(&mut e.b, 9);
        }
        if anim == 0xD {
            self.enemy_begin_attack(&mut e.b, 0x2B);
        }
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x3EB4C (state 0x24): a melee hit; when its animation ends the chain of kind 6 is over (decide), else the step between hits (0x3F0F6).
    pub(super) fn enemy_c13_3eb4c(&mut self, e: &mut Enemy) {
        if !self.enemy_step(&mut e.b) {
            return;
        }
        if e.m.b(MELEE_A) == 6 {
            return self.enemy_c13_4005c(e);
        }
        self.enemy_c13_3f0f6(e);
    }

    /// 0x3F0F6: after a hit: a target within 0x30 px in X gets the next hit at once (0x3EE82), else the step state 0x2C for `+$AC` frames (the
    /// table 0x74DA2 by a random draw plus the table 0x74E22 by level, at least 1). The first draw is thrown away.
    fn enemy_c13_3f0f6(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 1);
        let tx = self.lib_target(e).xi();
        let d0 = tx.wrapping_add(0x30).wrapping_sub(e.b.xi()) as u16;
        if d0 < 0x60 {
            return self.enemy_c13_3ee82(e);
        }
        e.b.act = 0x2C;
        let _ = self.rand() & 0x3E;
        let r = (self.rand() & 0x3E) as u32;
        let tb = &self.assets.enemy.t;
        let base = tb.u16(0x74DA2, r);
        let add = tb.u16(0x74E22, (self.w.level as u32) * 2);
        let mut v = base.wrapping_add(add);
        if v == 0 || (v as i16) < 0 {
            v = 1;
        }
        e.m.sw(TIMER, v);
    }

    /// 0x3EB7E (state 0x2C): the step between two hits; the next hit when it ends.
    pub(super) fn enemy_c13_3eb7e(&mut self, e: &mut Enemy) {
        let t = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, t);
        if t == 0 {
            return self.enemy_c13_3ee82(e);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x3EE82: the next hit of the chain (the animation by `+$BA * 8 + +$BB` of the table 0x3EF1E; 0xB attack 8, 0xD attack 0x2B, else 9; 0x63
    /// the kick approach); when the hits are used up the pause 0x3F5AE.
    fn enemy_c13_3ee82(&mut self, e: &mut Enemy) {
        let n = e.m.b(MELEE_N).wrapping_sub(1);
        e.m.sb(MELEE_N, n);
        if n == 0 {
            return self.enemy_c13_3f5ae(e);
        }
        self.enemy_begin_attack(&mut e.b, 9);
        e.m.sb(BUSY, 1);
        e.b.act = 0x24;
        e.m.sw(TIMER, 0x3C);
        self.lib_aim_at_target(e); // 0x32A58
        let idx = ((e.m.b(MELEE_A) as u32) << 3) + e.m.b(MELEE_N) as u32;
        let d0 = self.assets.enemy.t.u8(0x3EF1E, idx) as u16;
        if d0 == 0xB {
            self.enemy_begin_attack(&mut e.b, 8);
        }
        if d0 == 0x63 {
            return self.enemy_c13_3fb20(e);
        }
        if d0 == 0xD {
            self.enemy_begin_attack(&mut e.b, 0x2B);
        }
        self.enemy_anim(&mut e.b, d0);
    }

    // ------------------------------------------------------------------ the kick

    /// 0x3FB20 (far decision value 2, and the animation 0x63 of a melee chain): the kick approach (state 0x42, 20 frames, anim 0x18).
    pub(super) fn enemy_c13_3fb20(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 1);
        e.m.sb(COMMIT, 1);
        e.m.sw(TIMER, 0x14);
        e.b.act = 0x42;
        self.enemy_anim(&mut e.b, 0x18);
    }

    /// 0x3EB3E (state 0x42): wait out the approach, then the kick.
    pub(super) fn enemy_c13_3eb3e(&mut self, e: &mut Enemy) {
        let t = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, t);
        if t == 0 {
            self.enemy_c13_3fb40(e);
        }
    }

    /// 0x3FB40: the kick (state 0x26, 30 frames, attack 0xA, anim 0x19): a jump forward with the speeds of the tables 0x3FBC4 / 0x3FBD4 by the X
    /// class (a left facing object goes left).
    fn enemy_c13_3fb40(&mut self, e: &mut Enemy) {
        self.enemy_set_speed(&mut e.b, 6);
        self.enemy_begin_attack(&mut e.b, 0xA);
        e.m.sb(BUSY, 1);
        e.m.sb(COMMIT, 1);
        e.m.sw(TIMER, 0x1E);
        e.b.act = 0x26;
        e.b.airborne = 1;
        let (dx, _) = self.lib_distance_class(e);
        let tb = &self.assets.enemy.t;
        e.b.vx = tb.u16(0x3FBC4, dx as u32 * 2) as i16;
        e.b.vy = tb.u16(0x3FBD4, dx as u32 * 2) as i16;
        e.b.ax = 0;
        e.b.ay = 0xFFD0u16 as i16;
        e.b.az = 0;
        e.b.vz = 0;
        if e.b.facing == 0 {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        self.enemy_anim(&mut e.b, 0x19);
    }

    /// 0x3EE04 (state 0x26): the kick in the air; on landing 20 frames of state 0x14.
    pub(super) fn enemy_c13_3ee04(&mut self, e: &mut Enemy) {
        e.b.motion(); // 0x189C
        if e.b.airborne == 0 {
            e.m.sw(TIMER, 0x14);
            e.b.act = 0x14;
            self.enemy_anim(&mut e.b, 0);
        }
        self.enemy_step(&mut e.b);
    }

    // ------------------------------------------------------------------ the hop back after a block

    /// 0x3FBE4 (the guard blocked a hit, `+$BC`): hop back (state 0x28, 60 frames, anim 0x11): vx 0x300 * 4 away from the facing, slowed by 0x100.
    pub(super) fn enemy_c13_3fbe4(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 1);
        e.b.act = 0x28;
        e.m.sw(TIMER, 0x3C);
        self.lib_aim_at_target(e); // 0x32A58
        e.b.vz = 0;
        e.b.az = 0;
        e.b.vy = 0;
        e.b.ay = 0;
        e.b.vx = 0xC00;
        e.b.ax = 0xFF00u16 as i16;
        if e.b.facing != 0 {
            e.b.vx = e.b.vx.wrapping_neg();
            e.b.ax = e.b.ax.wrapping_neg();
        }
        self.enemy_anim(&mut e.b, 0x11);
    }

    /// 0x3EE2C (state 0x28): the hop back; when it ends the guard counter becomes negative by the aggression (the table 0x3EE5E: a pause before
    /// the next block) and a decision follows. While vx is not 0 the object faces the target and moves (0x189C).
    pub(super) fn enemy_c13_3ee2c(&mut self, e: &mut Enemy) {
        let t = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, t);
        if t == 0 {
            let v = self.assets.enemy.t.u8(0x3EE5E, self.rank.aggression as u32);
            e.m.sb(COUNT, v);
            return self.enemy_c13_4005c(e);
        }
        if e.b.vx != 0 {
            self.lib_face_target(e);
            e.b.motion(); // 0x189C
        }
        self.enemy_step(&mut e.b);
    }
}
