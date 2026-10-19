//! The boomerang, pool A class 19 (`+$20` = 0x4C, handler 0x45082..0x45706). Slice throws it (`class25_ai.rs`); the Slisaurs of Episode 7 call
//! the same spawn helper 0x4504E. A monster (list 0x6B64, `+$7D` 0xFF): heroes can hit it, then it falls and is freed. D3c section 6.4, BS25,
//! BS26.
//!
//! Variants (the high byte of `+$26`): 0, 2 and 4 a single boomerang (attack 0x3C, 0x40 px in front of the thrower), 6 a member of the fan
//! (attack 0x3D, 0x50 px, the speeds of row `+$A9` of the table 0x453FE). A single one flies out at 10 px per frame (clamped to 5) and turns
//! back 200 px past the target (variant 4 also homes in on the target in Y and Z); a fan member swings between its start X / Z and back by
//! its own accelerations. After 10 frames it is freed when it touches the thrower's body box. Owner: enemy port.

use super::{off::*, Enemy};
use crate::game::{slot_of, Game, Handle, Slot};

/// Own bytes (the original offsets).
mod o {
    /// `+$A0` word: the thrower. `+$A4` / `+$A6` words: the thrower's X and Z at the throw.
    pub const THROWER: usize = 0xA0;
    pub const START_X: usize = 0xA4;
    pub const START_Z: usize = 0xA6;
    /// `+$A2` byte: on the way back (singles). `+$A3` / `+$A8` bytes: the X / Z swing side of a fan member. `+$A9` byte: the fan row.
    pub const BACK: usize = 0xA2;
    pub const SWING_X: usize = 0xA3;
    pub const SWING_Z: usize = 0xA8;
    pub const ROW: usize = 0xA9;
    /// `+$AA` / `+$AC` words: the X / Z swing accelerations of a fan member.
    pub const ACC_X: usize = 0xAA;
    pub const ACC_Z: usize = 0xAC;
    /// `+$AE` word: the frames before the catch test. `+$B0` word: the Slisaur's helper (0x4558C).
    pub const CATCH_T: usize = 0xAE;
    pub const B0: usize = 0xB0;
    /// `+$B2` / `+$B3` bytes: the spin sound timer and its period.
    pub const SPIN_T: usize = 0xB2;
    pub const SPIN: usize = 0xB3;
    /// The fan speeds (12 rows of 6 words: vx, ax, the X swing, vz, az, the Z swing) and this object's catch box (6 words).
    pub const FAN: u32 = 0x453FE;
    pub const BOX: u32 = 0x45694;
}

impl Game {
    /// 0x4504E: a boomerang of variant `d1` (the high byte of `+$26`) with the fan row `d2` (`+$A9`); `+$A0` = the thrower, `+$A4` / `+$A6`
    /// = his X and Z, `+$76` = his target. Nothing when pool A is full.
    pub(super) fn c19_spawn(&mut self, thrower: &Enemy, h: Handle, d1: u8, d2: u8) {
        let Some(j) = self.enemies.alloc() else { return };
        let n = &mut self.enemies.slots[j];
        n.b.live = 1;
        n.b.k20 = 19 * 4;
        n.m.sw(o::THROWER, h);
        n.m.sw(o::START_X, thrower.b.xi() as u16);
        n.m.sw(o::START_Z, thrower.b.zi() as u16);
        n.m.sw(TARGET, thrower.m.w(TARGET));
        n.b.var26 = ((d1 as u16) << 8) | (n.b.var26 & 0xFF);
        n.m.sb(o::ROW, d2);
    }

    /// 0x45082 (mode table 0x45090): 0 the init 0x45098 (falls into the run), 2 the run 0x45188, 4 and 6 the free 0x4535C.
    pub(super) fn enemy_class19(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.c19_init(e);
                self.c19_run(e, h, i);
            }
            2 => self.c19_run(e, h, i),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// The thrower's body (a cleared body for a word that is no object).
    fn c19_thrower(&self, e: &Enemy) -> crate::game::body::Body {
        let t = e.m.w(o::THROWER);
        if slot_of(t).is_none() {
            return crate::game::body::Body::default();
        }
        self.body(t).clone()
    }

    /// 0x45098: not counted, `+$63` 0xFF, the table 0x45706 (animation 1 for the fan, else 0), the box table 0x103000; the thrower's facing;
    /// X = his X +- 0x40 (single, the spin sound every 8 frames, `+$A9` 0, attack 0x3C) or +- 0x50 (fan, every 4 frames, attack 0x3D), Y + 0x38,
    /// his Z; 10 frames before the catch test; the speeds (0x45362).
    fn c19_init(&mut self, e: &mut Enemy) {
        e.b.mode = e.b.mode.wrapping_add(2);
        e.m.sb(ATTACKING, 0xFF);
        e.b.b63 = 0xFF;
        e.m.sb(o::SPIN_T, 0);
        e.m.sb(o::SPIN, 0);
        e.m.sb(B87, 0);
        e.b.anim_table = 0x45706;
        let fan = (e.b.var26 >> 8) as u8 == 6;
        self.enemy_anim(&mut e.b, fan as u16);
        e.b.box_table = 0x103000;
        e.b.multi_box = false;
        let t = self.c19_thrower(e);
        e.b.facing = t.facing;
        let (spin, dx, atk) = if fan { (4, 0x50, 0x3D) } else { (8, 0x40, 0x3C) };
        e.m.sb(o::SPIN, spin);
        if !fan {
            e.m.sb(o::ROW, 0);
        }
        let dx: i16 = if t.facing != 0 { dx } else { -dx };
        let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
        put(&mut e.b.x, t.xi().wrapping_add(dx));
        put(&mut e.b.y, t.yi().wrapping_add(0x38));
        put(&mut e.b.z, t.zi());
        self.enemy_begin_attack(&mut e.b, atk);
        e.m.sb(o::BACK, 0);
        e.m.sw(o::CATCH_T, 0x0A);
        self.c19_launch(e);
    }

    /// 0x45362: the launch. Single: vx +-0xA00, ax +-0x80 (minus when facing left), vz and az 0; variant 0 vy -0x200, the others 0; no gravity.
    /// Fan (0x453A8): row `+$A9` of 0x453FE: vx, ax (negated when facing left, which also clears `+$A3`), the X swing `+$AA`, vy 0xC0, vz,
    /// az, the Z swing `+$AC`, `+$A8` = 1.
    fn c19_launch(&mut self, e: &mut Enemy) {
        if (e.b.var26 >> 8) as u8 != 6 {
            let (mut d0, mut d1): (i16, i16) = (0xA00, 0x80);
            if e.b.facing == 0 {
                d0 = -d0;
                d1 = -d1;
            }
            e.b.vx = d0;
            e.b.ax = d1;
            e.b.vz = 0;
            e.b.az = 0;
            e.b.vy = if e.b.var26 >> 8 != 0 { 0 } else { -0x200 };
            e.b.ay = 0;
            return;
        }
        let tb = &self.assets.enemy.t;
        let row = e.m.b(o::ROW) as u32 * 12;
        let w = |k: u32| tb.i16(o::FAN, row + 2 * k);
        let (mut d0, mut d1, swing) = (w(0), w(1), w(2) as u16);
        let (vz, az, swing_z) = (w(3), w(4), w(5) as u16);
        let mut d2 = 1;
        if e.b.facing == 0 {
            d0 = d0.wrapping_neg();
            d1 = d1.wrapping_neg();
            d2 = 0;
        }
        e.m.sw(o::ACC_X, swing);
        e.b.vx = d0;
        e.b.ax = d1;
        e.m.sb(o::SWING_X, d2);
        e.b.vy = 0xC0;
        e.b.ay = 0;
        e.b.vz = vz;
        e.b.az = az;
        e.m.sw(o::ACC_Z, swing_z);
        e.m.sb(o::SWING_Z, 1);
    }

    /// 0x45188: the hit stop (at its end the animation steps in this frame), then by `+$05` (the long table 0x451AC): 0 the flight 0x451B4,
    /// 2 the fall after a hero's hit 0x45308.
    fn c19_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        let hs = e.b.hitstop;
        if hs != 0 {
            e.b.hitstop = hs - 1;
            if hs != 1 {
                return;
            }
            e.b.timer = 1;
        }
        if e.b.sub == 0 {
            self.c19_flight(e, h, i);
        } else {
            self.c19_fall(e, h, i);
        }
    }

    /// 0x451B4: the spin sound (0xA21E) every `+$B3` frames (not from fan members other than row 0); the move by variant (table 0x45218:
    /// 0 and 2 0x45220, 4 0x4528E, 6 0x452F4); the shadow (0x456E0); after 10 frames the catch test (0x455E8, or 0x4558C in stage 6), a catch
    /// frees it; the draw check, the monster list (0x4C0A), the fall below the screen (0x456C4) and the far test (0x173C).
    fn c19_flight(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        let t = e.m.b(o::SPIN_T);
        e.m.sb(o::SPIN_T, t.wrapping_sub(1));
        if t == 0 {
            e.m.sb(o::SPIN_T, e.m.b(o::SPIN));
            if e.m.b(o::ROW) == 0 {
                self.c19_spin_sound(e.b.xi());
            }
        }
        match (e.b.var26 >> 8) as u8 {
            4 => self.c19_homing(e),
            6 => {
                // 0x452F4
                self.c19_swing_x(e);
                self.c19_swing_z(e);
                e.b.motion();
                self.enemy_step(&mut e.b);
            }
            _ => self.c19_single(e),
        }
        self.c19_shadow(e);
        let c = e.m.w(o::CATCH_T);
        e.m.sw(o::CATCH_T, c.wrapping_sub(1));
        if c == 0 {
            e.m.sw(o::CATCH_T, 0);
            let caught = if self.w.stage == 6 { self.c19_caught_by_helpers(e) } else { self.c19_touches(e, e.m.w(o::THROWER)) };
            if caught {
                return self.enemy_free_pool_a(e, i); // 0x4535C: jmp 0x48FC
            }
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_monster_register(h);
        // 0x456C4: below the screen (camera Y + 0xD8 at or below Y + Z, unsigned) mode 6.
        let d0 = (self.cam.s2.cam_y as u16).wrapping_add(0xD8);
        let d1 = (e.b.yi() as u16).wrapping_add(e.b.zi() as u16);
        if d0 <= d1 {
            e.b.mode = 6;
        }
        self.enemy_despawn_if_far(e, i); // jmp 0x173C
    }

    /// 0xA21E: the spin sound with the pan of X: 0x7E, or 0x101 when bit 5 of G.$43B was clear (`bchg`), d2 = 0.
    fn c19_spin_sound(&mut self, x: i16) {
        let pan = self.snd_pan_x(x);
        let old = self.snd_toggle_b(5);
        self.snd_request(if old { 0x7E } else { 0x101 }, pan as u32, 0);
    }

    /// 0x45220: variants 0 and 2. On the way out, 200 px past the target (0x4548E) it turns: ax +-0x400 and vy negated. Then the flight
    /// (0x189C), |vx| at most 0x500, Y at least 0x10 on the way out and at most 0x48 on the way back; the animation.
    fn c19_single(&mut self, e: &mut Enemy) {
        if e.m.b(o::BACK) == 0 && self.c19_past_target(e) {
            e.b.ax = if e.b.facing != 0 { -0x400 } else { 0x400 };
            e.b.vy = e.b.vy.wrapping_neg();
        }
        e.b.motion();
        Self::c19_clamp_vx(e);
        let y = e.b.yi();
        let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
        if e.m.b(o::BACK) == 0 {
            if y < 0x10 {
                put(&mut e.b.y, 0x10);
            }
        } else if y > 0x48 {
            put(&mut e.b.y, 0x48);
        }
        self.enemy_step(&mut e.b);
    }

    /// The clamp of vx to +-0x500 (`bpl` / `neg` / `ble`, signed).
    fn c19_clamp_vx(e: &mut Enemy) {
        let v = e.b.vx;
        let a = if v < 0 { v.wrapping_neg() } else { v };
        if a > 0x500 {
            e.b.vx = if v < 0 { -0x500 } else { 0x500 };
        }
    }

    /// 0x4528E: variant 4: the turn as variant 0, the flight, the clamp; on the way back, or 0x40 px or more from the start X, it homes in on
    /// the target (0x454E4 Y, 0x45506 Z); the animation.
    fn c19_homing(&mut self, e: &mut Enemy) {
        if e.m.b(o::BACK) == 0 && self.c19_past_target(e) {
            e.b.ax = if e.b.facing != 0 { -0x400 } else { 0x400 };
        }
        e.b.motion();
        Self::c19_clamp_vx(e);
        let far = {
            let d = (e.m.w(o::START_X) as i16).wrapping_sub(e.b.xi());
            let d = if d < 0 { d.wrapping_neg() } else { d };
            d >= 0x40
        };
        if e.m.b(o::BACK) != 0 || far {
            let tgt = self.c19_target(e);
            // 0x454E4: vy +-0x100 toward the target's Y + 0x30 (the borrow of the unsigned difference), no gravity.
            let dy = (tgt.yi() as u16).wrapping_add(0x30);
            e.b.vy = if dy < e.b.yi() as u16 { -0x100 } else { 0x100 };
            e.b.ay = 0;
            // 0x45506: vz +-0x100 toward the target's Z, no Z acceleration.
            e.b.vz = if (tgt.zi() as u16) < e.b.zi() as u16 { -0x100 } else { 0x100 };
            e.b.az = 0;
        }
        self.enemy_step(&mut e.b);
    }

    /// The target's body (`+$76`).
    fn c19_target(&self, e: &Enemy) -> crate::game::body::Body {
        let t = e.m.w(TARGET);
        if slot_of(t).is_none() {
            return crate::game::body::Body::default();
        }
        self.body(t).clone()
    }

    /// 0x4548E: on the way out, 200 px past the target (facing right: target X + 0xC8 - X below 0; facing left: target X - 0xC8 - X above
    /// 0, signed): on the way back (`+$A2` = 1) with a new attack 0x3C. The carry.
    fn c19_past_target(&mut self, e: &mut Enemy) -> bool {
        let tx = self.c19_target(e).xi();
        let d1: i16 = if e.b.facing != 0 { 0xC8 } else { -0xC8 };
        let d0 = tx.wrapping_add(d1).wrapping_sub(e.b.xi());
        let past = if e.b.facing != 0 { d0 < 0 } else { d0 > 0 };
        if !past {
            return false;
        }
        e.m.sb(o::BACK, 1);
        self.enemy_begin_attack(&mut e.b, 0x3C);
        true
    }

    /// 0x45524: the X swing of a fan member: past its start X on the swing side, the side flips and ax is negated, then pulled back by `+$AA`
    /// (subtracted, negated when ax became negative).
    fn c19_swing_x(&self, e: &mut Enemy) {
        let (a, b) = if e.m.b(o::SWING_X) == 0 { (e.m.w(o::START_X), e.b.xi() as u16) } else { (e.b.xi() as u16, e.m.w(o::START_X)) };
        if a >= b {
            return;
        }
        let mut d0 = e.m.w(o::ACC_X) as i16;
        e.m.sb(o::SWING_X, e.m.b(o::SWING_X) ^ 1);
        e.b.ax = e.b.ax.wrapping_neg();
        if e.b.ax < 0 {
            d0 = d0.wrapping_neg();
        }
        e.b.ax = e.b.ax.wrapping_sub(d0);
    }

    /// 0x45558: the Z swing (start Z `+$A6`, side `+$A8`, `+$AC`, az).
    fn c19_swing_z(&self, e: &mut Enemy) {
        let (a, b) = if e.m.b(o::SWING_Z) == 0 { (e.m.w(o::START_Z), e.b.zi() as u16) } else { (e.b.zi() as u16, e.m.w(o::START_Z)) };
        if a >= b {
            return;
        }
        let mut d0 = e.m.w(o::ACC_Z) as i16;
        e.m.sb(o::SWING_Z, e.m.b(o::SWING_Z) ^ 1);
        e.b.az = e.b.az.wrapping_neg();
        if e.b.az < 0 {
            d0 = d0.wrapping_neg();
        }
        e.b.az = e.b.az.wrapping_sub(d0);
    }

    /// 0x456E0: no shadow (bit 4 of `+$25`) while Z is 0x70 or more (0x80 in stage 3), signed.
    fn c19_shadow(&self, e: &mut Enemy) {
        let d2: i16 = if self.w.stage == 3 { 0x80 } else { 0x70 };
        if d2 > e.b.zi() {
            e.b.draw_flags &= !0x10;
        } else {
            e.b.draw_flags |= 0x10;
        }
    }

    /// 0x455EC: this object's catch box (0x45694) overlaps the body box of `other` (`+$44` table, `+$48` index): Z, then X (the box mirrored
    /// for a facing right), then Y, each as an unsigned window test.
    fn c19_touches(&self, e: &Enemy, other: Handle) -> bool {
        if slot_of(other).is_none() {
            return false;
        }
        let a = self.body(other);
        let bank = &self.assets.sprites.hitboxes;
        let Some(ab) = bank.try_at((a.box_table & 0xFF_FFFF).wrapping_add(a.body_box as u32 * 12)) else { return false };
        let tb = &self.assets.enemy.t;
        let m = |k: u32| tb.i16(o::BOX, 2 * k);
        let (zo, zs, xo, xs, yo, ys) = (m(0), m(1), m(2), m(3), m(4), m(5));
        // Z
        let d0 = a.zi().wrapping_add(ab.zoff).wrapping_sub(e.b.zi().wrapping_add(zo)).wrapping_add(ab.zsize);
        if d0 as u16 > ab.zsize.wrapping_add(zs) as u16 {
            return false;
        }
        // X
        let (mut d1, d2) = (ab.xoff, ab.xsize);
        if a.facing != 0 {
            d1 = d1.wrapping_neg().wrapping_sub(d2);
        }
        let ax = a.xi().wrapping_add(d1);
        let (mut d3, d4) = (xo, xs);
        if e.b.facing != 0 {
            d3 = d3.wrapping_neg().wrapping_sub(d4);
        }
        let bx = e.b.xi().wrapping_add(d3);
        let d0 = ax.wrapping_sub(bx).wrapping_add(d2);
        if d0 as u16 > d2.wrapping_add(d4) as u16 {
            return false;
        }
        // Y
        let d0 = a.yi().wrapping_add(ab.yoff).wrapping_sub(e.b.yi().wrapping_add(yo)).wrapping_add(ab.ysize);
        d0 as u16 <= ab.ysize.wrapping_add(ys) as u16
    }

    /// 0x4558C (stage 6, the Slisaurs): the thrower's `+$C6` object (kept in `+$B0`); with its `+$AA` byte set, the live ones of its
    /// `+$A0`, `+$A2`, `+$A4` objects are tested in order.
    fn c19_caught_by_helpers(&self, e: &mut Enemy) -> bool {
        let t = e.m.w(o::THROWER);
        let helper = self.c19_word(t, 0xC6);
        e.m.sw(o::B0, helper);
        if self.c19_byte(helper, 0xAA) == 0 {
            return false;
        }
        for off in [0xA0, 0xA2, 0xA4] {
            let o2 = self.c19_word(helper, off);
            if slot_of(o2).is_some() && self.body(o2).live != 0 && self.c19_touches(e, o2) {
                return true;
            }
        }
        false
    }

    /// The word at `off` of a pool A or B object (0 for others).
    fn c19_word(&self, h: Handle, off: usize) -> u16 {
        match slot_of(h) {
            Some(Slot::Enemy(i)) => self.enemies.slots[i].m.w(off),
            Some(Slot::Prop(i)) => self.props.slots[i].m.w(off),
            _ => 0,
        }
    }

    /// The byte at `off` of a pool A or B object (0 for others).
    fn c19_byte(&self, h: Handle, off: usize) -> u8 {
        match slot_of(h) {
            Some(Slot::Enemy(i)) => self.enemies.slots[i].m.b(off),
            Some(Slot::Prop(i)) => self.props.slots[i].m.b(off),
            _ => 0,
        }
    }

    /// 0x45308: hit by a hero (table 0x4531C by `+$07`): step 0 vx 0x100, gravity -0x40, in the air (falls into 2); step 2 the fall (0x189C)
    /// and the terrain (0x12CB4) until the landing, then the free. The draw check follows in both (0x45316, also after the free).
    fn c19_fall(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.step == 0 {
            e.b.step = e.b.step.wrapping_add(2);
            e.b.vx = 0x100;
            e.b.ax = 0;
            e.b.vy = 0;
            e.b.ay = -0x40;
            e.b.vz = 0;
            e.b.az = 0;
            e.b.airborne = 1;
        }
        if e.b.airborne != 0 {
            e.b.motion();
            if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
                e.m.sw(OBSTACLE, ob);
            }
        } else {
            self.enemy_free_pool_a(e, i);
        }
        self.enemy_draw_check(&mut e.b, h);
    }
}
