//! Class 14: the state entries (0x41278..0x41A1C, called by the decisions and by the entrance
//! dispatch), the two decisions (0x41BE8, 0x41CD0) with the ROM tables 0x73214 / 0x73414 / 0x73494 /
//! 0x73514 / 0x73714 / 0x73914, and the placing of the knife parts (0x41D6E). See `class14.rs`.

use super::class14::f;
use super::lib32::put_hi;
use super::{Enemy, off::*};
use crate::game::{Game, Slot, slot_of};

impl Game {
    /// Variant (high byte of `+$26`).
    fn c14_variant(e: &Enemy) -> u32 {
        (e.b.var26 >> 8) as u32 & 0xFF
    }

    // ------------------------------------------------------------------ decisions

    /// 0x41BE8: decision 1: the distance class of the target and a random row choose the next state.
    pub(super) fn c14_41be8(&mut self, e: &mut Enemy) {
        self.enemy_set_speed(&mut e.b, 5);
        e.m.sb(f::BUSY, 0);
        e.m.sb(f::BLOCK, 0);
        let (dx, _) = self.lib_distance_class(e); // 0x3293C
        e.m.sb(f::DIST, dx as u8);
        let r = (self.rand() & 0x1F) as u32;
        let idx = (r << 3) + e.m.b(f::DIST) as u32 + (Self::c14_variant(e) << 7);
        match self.assets.enemy.t.u8(0x73214, idx) {
            1 => self.c14_412f6(e),
            2 => self.c14_41278(e),
            3 => self.c14_4135a(e),
            4 => self.c14_413f8(e),
            5 => self.c14_4124a(e),
            v => panic!("class 14 decision 1 value {v:#x}"),
        }
    }

    /// 0x41CD0: decision 2: first the chance of the table 0x73494 (same facing), else the table of
    /// the number of knives (0x73514 none, 0x73714 one, 0x73914 two).
    pub(super) fn c14_41cd0(&mut self, e: &mut Enemy) {
        self.enemy_set_speed(&mut e.b, 5);
        e.m.sb(f::BUSY, 0);
        e.m.sb(f::BLOCK, 0);
        let (dx, _) = self.lib_distance_class(e);
        e.m.sb(f::DIST, dx as u8);
        if self.c14_41c46(e) {
            return self.c14_41be8(e); // 0x41D52
        }
        let r = (self.rand() & 0xE8) as u32;
        let idx = r + e.m.b(f::DIST) as u32 + (Self::c14_variant(e) << 7);
        let table = match e.m.b(f::KNIVES) {
            0 => 0x73514,
            1 => 0x73714,
            _ => 0x73914,
        };
        match self.assets.enemy.t.u8(table, idx) {
            1 => self.c14_4169c(e),
            2 => self.c14_416c0(e),
            3 => self.c14_416e4(e),
            4 => self.c14_attack_start(e, 0),
            5 => self.c14_attack_start(e, 1),
            6 => self.c14_attack_start(e, 2),
            7 => self.c14_attack_start(e, 3),
            8 => self.c14_attack_start(e, 4),
            9 => self.c14_41708(e),
            10 => self.c14_41722(e),
            11 => self.c14_41470(e),
            v => panic!("class 14 decision 2 value {v:#x}"),
        }
    }

    /// 0x41C46: true when the target faces the same way and a random draw reaches the table 0x73494.
    fn c14_41c46(&mut self, e: &mut Enemy) -> bool {
        if self.lib_target(e).facing != e.b.facing {
            return false;
        }
        let d0 = (((self.rank.aggression << 3) as u8).wrapping_add(e.m.b(f::DIST))) as u32 + (Self::c14_variant(e) << 5);
        let d1 = self.assets.enemy.t.u8(0x73494, d0) as i8;
        let r = (self.rand() & 0x1F) as i8;
        r >= d1
    }

    /// 0x41C8C: a second chance of the decision 2 (table 0x73414) while the target is not in a reaction.
    pub(super) fn c14_41c8c(&mut self, e: &mut Enemy) {
        let t = self.lib_target(e);
        if t.mode == 2 && t.sub == 2 {
            return;
        }
        self.lib_face_target(e);
        let d0 = (((self.rank.aggression << 3) as u8).wrapping_add(e.m.b(f::DIST))) as u32 + (Self::c14_variant(e) << 5);
        let d1 = self.assets.enemy.t.u8(0x73414, d0) as i8;
        let r = (self.rand() & 0x1F) as i8;
        if r < d1 {
            self.c14_41cd0(e);
        }
    }

    // ------------------------------------------------------------------ state entries of decision 1

    /// 0x4124A (decision 1, value 5): walk round (state 0xC).
    pub(super) fn c14_4124a(&mut self, e: &mut Enemy) {
        e.b.act = 0xC;
        e.m.sw(f::TIMER, 0xB4);
        e.m.sb(f::TURN_T, 0x1E);
        self.lib_aim_at_target(e);
        self.enemy_set_speed(&mut e.b, 0xD);
        self.lib_face_target(e);
        self.enemy_anim(&mut e.b, 2);
    }

    /// 0x41278 (value 2, and the end of states 2 and 0x2E): approach (state 0x16).
    pub(super) fn c14_41278(&mut self, e: &mut Enemy) {
        e.m.sb(f::OLD_DIR, e.b.dir);
        self.lib_aim_beside_target(e, 0x50);
        self.c14_412b0(e);
        e.m.sb(f::TURN_T, 6);
        e.m.sw(f::TIMER, 0x3C);
        e.b.act = 0x16;
        self.enemy_set_speed(&mut e.b, 0x15);
        self.enemy_anim(&mut e.b, 1);
        self.c14_41d6e(e);
    }

    /// 0x412B0 (and its copy 0x41BA2): the new direction `+$22` may differ from the old one `+$A4` by at
    /// most 2 steps.
    pub(super) fn c14_412b0(&mut self, e: &mut Enemy) {
        let old = e.m.b(f::OLD_DIR);
        let mut d0 = e.b.dir.wrapping_sub(old) & 0x1F;
        d0 = d0.wrapping_add(2) & 0x1F;
        if (d0 as i8) <= 4 {
            return;
        }
        d0 = d0.wrapping_sub(2);
        if (d0 as i8) > 0x10 {
            e.b.dir = old.wrapping_sub(2) & 0x1F; // 0x412E6
        } else {
            e.b.dir = old.wrapping_add(2) & 0x1F;
        }
    }

    /// 0x412F6 (value 1, and the "target is hit" check of the frame): side step (state 8).
    pub(super) fn c14_412f6(&mut self, e: &mut Enemy) {
        e.m.sb(f::BUSY, 0);
        e.m.sw(f::TIMER, 0x3C);
        e.b.act = 8;
        self.c14_41308(e);
    }

    /// 0x41308: the speeds `+$B0` / `+$B4` towards the point, then the walk animation.
    fn c14_41308(&mut self, e: &mut Enemy) {
        let right = (e.m.w(DEST_X) as i16).wrapping_add(0x400) > e.b.xi().wrapping_add(0x400);
        e.m.sl(f::VX, if right { 0x15000 } else { 0xFFFE_B000 });
        let more = (e.m.w(DEST_Z) as i16) > e.b.zi();
        e.m.sl(f::VZ, if more { 0x10000 } else { 0xFFFF_0000 });
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x4135A (value 3): side step in X at a speed from the table 0x413B8 (state 0x18).
    pub(super) fn c14_4135a(&mut self, e: &mut Enemy) {
        e.m.sb(f::BUSY, 0);
        e.m.sw(f::TIMER, 0x78);
        e.b.act = 0x18;
        let r = (self.rand() & 0x3C) as u32;
        let v = self.assets.enemy.t.u32(0x413B8, r);
        e.m.sl(f::SPEED_C4, v);
        e.m.sl(f::VX, v);
        if !((e.m.w(DEST_X) as i16).wrapping_add(0x400) > e.b.xi().wrapping_add(0x400)) {
            e.m.sl(f::VX, (v as i32).wrapping_neg() as u32);
        }
        self.lib_face_target(e);
        let id = if (e.m.l(f::VX) as i32) < 0x14000 { 1 } else { 0x1D };
        self.enemy_anim(&mut e.b, id);
    }

    /// 0x413F8 (value 4): side step in Z (state 0x1A).
    pub(super) fn c14_413f8(&mut self, e: &mut Enemy) {
        e.m.sb(f::BUSY, 0);
        e.m.sw(f::TIMER, 0x78);
        e.b.act = 0x1A;
        self.c14_41308(e);
    }

    // ------------------------------------------------------------------ state entries of decision 2

    /// 0x4169C: take one knife (state 0x1C).
    fn c14_4169c(&mut self, e: &mut Enemy) {
        e.m.sb(f::BUSY, 1);
        e.b.act = 0x1C;
        e.m.sw(f::TIMER, 0x1E);
        e.m.sb(f::KNIVES, 1);
        self.enemy_anim(&mut e.b, 4);
        self.c14_41d6e(e);
    }

    /// 0x416C0: take two knives (state 0x1E).
    fn c14_416c0(&mut self, e: &mut Enemy) {
        e.m.sb(f::BUSY, 1);
        e.b.act = 0x1E;
        e.m.sw(f::TIMER, 0x1E);
        e.m.sb(f::KNIVES, 2);
        self.enemy_anim(&mut e.b, 7);
        self.c14_41d6e(e);
    }

    /// 0x416E4: take two knives (state 0x20; the entrances call it too).
    pub(super) fn c14_416e4(&mut self, e: &mut Enemy) {
        e.m.sb(f::BUSY, 1);
        e.b.act = 0x20;
        e.m.sw(f::TIMER, 0x1E);
        e.m.sb(f::KNIVES, 2);
        self.enemy_anim(&mut e.b, 0x14);
        self.c14_41d6e(e);
    }

    /// 0x41708: throw a knife (state 0xA).
    fn c14_41708(&mut self, e: &mut Enemy) {
        e.m.sb(f::BUSY, 1);
        e.b.act = 0xA;
        e.m.sw(f::TIMER, 0x1E);
        self.enemy_anim(&mut e.b, 0xC);
    }

    /// 0x41722: run at the target with throws (state 0x24).
    fn c14_41722(&mut self, e: &mut Enemy) {
        e.b.act = 0x24;
        e.m.sw(f::TIMER, 0xF0);
        e.m.sb(f::TURN_T, 0x1E);
        e.m.sb(f::BUSY, 1);
        let r = (self.rand() & 0xF) as u32;
        let v = self.assets.enemy.t.u8(0x4177E, r);
        e.m.sb(f::KNIVES_TO_THROW, v);
        self.lib_aim_at_target(e);
        self.lib_face_target(e);
        self.enemy_set_speed(&mut e.b, 0xD);
        self.enemy_begin_attack(&mut e.b, 0x1C);
        self.enemy_anim(&mut e.b, 3);
    }

    /// 0x41590..0x415A8 -> 0x415AA: start attack kind `kind` (0..4: rows of the table 0x4162E) in state
    /// 0x14; attack id 0x17, and 0x1A as well when no knives are held (the second call takes the
    /// next serial too).
    fn c14_attack_start(&mut self, e: &mut Enemy, kind: u32) {
        self.enemy_begin_attack(&mut e.b, 0x17);
        if e.m.b(f::KNIVES) == 0 {
            self.enemy_begin_attack(&mut e.b, 0x1A);
        }
        e.m.sb(f::BUSY, 1);
        e.b.act = 0x14;
        e.m.sw(f::TIMER, 0x3C);
        let t = &self.assets.enemy.t;
        let (bb, bc, mut id) = (t.u8(0x4162E, kind * 4), t.u8(0x4162E, kind * 4 + 1), t.u8(0x4162E, kind * 4 + 2) as u16);
        e.m.sb(f::ATK_NO, bb);
        e.m.sb(f::ATK_STEP, bc);
        if e.m.b(f::KNIVES) == 0 && id == 0xB {
            id = 0xA;
        }
        self.enemy_anim(&mut e.b, id);
        self.lib_face_target(e);
        self.lib_aim_beside_target(e, 0x50);
    }

    /// 0x41646: the back step after a block (state 0x2C).
    pub(super) fn c14_41646(&mut self, e: &mut Enemy) {
        e.m.sb(f::BUSY, 1);
        e.b.act = 0x2C;
        e.m.sw(f::TIMER, 0x3C);
        self.lib_aim_beside_target(e, 0x50);
        e.b.vz = 0;
        e.b.az = 0;
        e.b.vy = 0;
        e.b.ay = 0;
        e.b.vx = 0xC00;
        e.b.ax = -0x100;
        if e.b.facing != 0 {
            e.b.vx = e.b.vx.wrapping_neg();
            e.b.ax = e.b.ax.wrapping_neg();
        }
        self.enemy_anim(&mut e.b, 0x10);
    }

    /// 0x4146A: a ledge hop; 0x41470: the jump kick wind up (state 0x26).
    pub(super) fn c14_4146a(&mut self, e: &mut Enemy) {
        e.m.sb(f::HOP_C8, 1);
        self.c14_41470(e);
    }

    pub(super) fn c14_41470(&mut self, e: &mut Enemy) {
        e.m.sb(f::BUSY, 1);
        e.m.sw(f::TIMER, 0x1E);
        e.b.act = 0x26;
        self.enemy_begin_attack(&mut e.b, 0x34);
        self.lib_face_target(e);
        self.enemy_anim(&mut e.b, 0x13);
        self.enemy_set_speed(&mut e.b, 5);
    }

    /// 0x414B0: the jump kick (state 0x22): a leap at the target. The horizontal speed is the number of
    /// 0x3800 steps in the distance (less 0x30 px in front), at most 0x280.
    pub(super) fn c14_414b0(&mut self, e: &mut Enemy) {
        e.m.sb(f::BUSY, 1);
        e.m.sw(f::TIMER, 0x3C);
        e.b.act = 0x22;
        e.b.airborne = 1;
        self.enemy_begin_attack(&mut e.b, 0x34);
        let tx = self.lib_target(e).x;
        let mut d0 = e.b.x.wrapping_sub(tx);
        d0 = if e.b.facing == 0 { d0.wrapping_sub(0x30_0000) } else { d0.wrapping_add(0x30_0000) };
        if d0 < 0 {
            d0 = d0.wrapping_neg();
        }
        e.b.vx = Self::c14_leap_steps(d0);
        if e.b.vx > 0x280 {
            e.b.vx = 0x280;
        }
        e.b.vy = 0x700;
        e.b.ax = 0;
        e.b.ay = -0x40;
        if e.m.b(f::HOP_C8) != 0 {
            e.b.vz = 0x20;
            e.b.az = 0;
        } else {
            e.b.az = 0;
            e.b.vz = 0;
        }
        e.m.sb(f::HOP_C8, 0);
        if e.b.facing == 0 {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        self.enemy_begin_attack(&mut e.b, 0x34);
        self.enemy_anim(&mut e.b, 0x17);
    }

    /// 0x4150A: the number of 0x3800 steps in `d0` (at least 1): `d1 = 0; do { d1 += 1; d0 -= 0x3800 } while d0 >= 0`.
    pub(super) fn c14_leap_steps(mut d0: i32) -> i16 {
        let mut d1: i16 = 0;
        loop {
            d1 = d1.wrapping_add(1);
            d0 = d0.wrapping_sub(0x3800);
            if d0 < 0 {
                return d1;
            }
        }
    }

    /// 0x41430: an obstacle in the way (wall bit 7): face it and swing (state 0x34).
    pub(super) fn c14_41430(&mut self, e: &mut Enemy) {
        e.b.facing = if (e.b.dir as i8) > 0x10 { 0 } else { 1 };
        e.m.sb(f::BUSY, 1);
        e.m.sw(f::TIMER, 0x1E);
        e.b.act = 0x34;
        self.enemy_anim(&mut e.b, 0x19);
    }

    /// 0x4178E: stand and wait (state 6): the wait is from the tables 0x7445A (level) and 0x417E2 (rand),
    /// at least 1.
    pub(super) fn c14_4178e(&mut self, e: &mut Enemy) {
        let d2 = ((e.b.var26 >> 8) as u8 as i8 >> 5) as u8;
        let idx = (self.w.level as u8).wrapping_add(d2) as u32;
        let mut d3 = self.assets.enemy.t.u8(0x7445A, idx) as i16;
        let r = ((self.rand() & 0xF) * 2) as u32;
        d3 = d3.wrapping_add(self.assets.enemy.t.i16(0x417E2, r));
        if d3 <= 0 {
            d3 = 1;
        }
        e.m.sw(f::TIMER, d3 as u16);
        e.m.sb(f::BUSY, 1);
        e.b.act = 6;
        self.enemy_set_speed(&mut e.b, 6);
        self.enemy_anim(&mut e.b, 0);
    }

    /// 0x411F0: the end of an attack animation: the next attack of the chain (0x40DFA), else a reload
    /// wait (state 0x3A).
    pub(super) fn c14_411f0(&mut self, e: &mut Enemy) {
        if e.m.b(f::ATK_STEP) == 1 {
            return self.c14_40dfa(e);
        }
        e.m.sb(f::BUSY, 1);
        e.b.act = 0x3A;
        let _ = self.rand() & 0x3E;
        let r = (self.rand() & 0x3E) as u32;
        let a = self.assets.enemy.t.u16(0x74DA2, r);
        let b = self.assets.enemy.t.u16(0x74EA2, self.w.level as u32 * 2);
        let n = a.wrapping_add(b) as i16;
        e.m.sw(f::TIMER, if n <= 0 { 1 } else { n as u16 });
    }

    /// 0x41A96: run off (state 0x36) when off screen; on screen the decision.
    pub(super) fn c14_41a96(&mut self, e: &mut Enemy) {
        if !self.lib_off_screen(&e.b) {
            return self.c14_41be8(e);
        }
        e.b.act = 0x36;
        e.m.sw(f::TIMER, 0x64);
        e.m.sb(f::TURN_T, 0x1E);
        e.m.sw(f::FLEE_T, 0x3C);
        e.m.sb(f::BUSY, 0);
        let _ = self.lib_slot6_new(e); // 0x32C5E
        e.m.sb(f::OLD_DIR, e.b.dir);
        self.lib_aim_at_target(e);
        let d0 = e.b.dir.wrapping_add(8) & 0x10;
        if (d0 as i8) < 0x10 {
            e.b.facing = 1;
            e.b.dir = 0;
        } else {
            e.b.facing = 0;
            e.b.dir = 0x10;
        }
        if (self.rand() & 0xF) > 0xC {
            e.b.dir = e.b.dir.wrapping_add(0x10) & 0x10;
        }
        self.enemy_set_speed(&mut e.b, 0xB);
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x41B2E: chase (state 0x38) when off screen; on screen the decision.
    pub(super) fn c14_41b2e(&mut self, e: &mut Enemy) {
        if !self.lib_off_screen(&e.b) {
            return self.c14_41be8(e);
        }
        e.m.sb(f::BUSY, 0);
        e.m.sw(f::TIMER, 0x64);
        e.m.sw(f::FLEE_T, 0x3C);
        e.b.act = 0x38;
        let right = (e.m.w(DEST_X) as i16).wrapping_add(0x400) > e.b.xi().wrapping_add(0x400);
        e.m.sl(f::VX, if right { 0x28000 } else { 0xFFFD_8000 });
        let more = (e.m.w(DEST_Z) as i16) > e.b.zi();
        e.m.sl(f::VZ, if more { 0x20000 } else { 0xFFFE_0000 });
        self.enemy_anim(&mut e.b, 1);
    }

    // ------------------------------------------------------------------ entrances (0x403E8)

    /// 0x41954 (entrance 0): walk in (state 0x30).
    pub(super) fn c14_41954(&mut self, e: &mut Enemy) {
        e.b.act = 0x30;
        e.m.sw(f::TIMER, 0x48);
        e.m.sb(f::TURN_T, 0x1E);
        self.lib_slot3_new(e);
        self.lib_face_target(e);
        e.b.dir = 8;
        if e.b.facing == 0 {
            e.b.dir = 0x18;
        }
        self.enemy_set_speed(&mut e.b, 5);
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x41802 (entrance 2): stand and wait 0x4B0 frames (state 0x28).
    pub(super) fn c14_41802(&mut self, e: &mut Enemy) {
        e.m.sb(f::BUSY, 1);
        e.m.sw(f::TIMER, 0x4B0);
        e.b.act = 0x28;
        self.enemy_anim(&mut e.b, 0x11);
        self.enemy_set_speed(&mut e.b, 5);
    }

    /// 0x41824 (entrance 4): stand and wait 0x1E0 frames (state 0x2A).
    pub(super) fn c14_41824(&mut self, e: &mut Enemy) {
        e.m.sb(f::BUSY, 1);
        e.m.sw(f::TIMER, 0x1E0);
        e.b.act = 0x2A;
        self.enemy_anim(&mut e.b, 0x12);
        self.enemy_set_speed(&mut e.b, 5);
    }

    /// 0x41846 (entrance 6): jump in from above, two knives (state 0x2E).
    pub(super) fn c14_41846(&mut self, e: &mut Enemy) {
        self.c14_416e4(e);
        e.m.sb(f::BUSY, 1);
        e.m.sw(f::TIMER, 0x3C);
        e.b.act = 0x2E;
        self.enemy_anim(&mut e.b, 0x16);
        self.enemy_begin_attack(&mut e.b, 0x18);
        e.b.set_yi_keep(e.b.yi().wrapping_add(0x120));
        e.b.vy = 0;
        e.b.ay = -0x20;
        e.b.vx = 0;
        e.b.ax = 0;
        e.b.vz = 0;
        e.b.az = 0;
        self.lib_face_target(e);
        self.enemy_set_speed(&mut e.b, 5);
    }

    /// 0x418B4 (entrance 8): jump in beside the target (state 0x2E).
    pub(super) fn c14_418b4(&mut self, e: &mut Enemy) {
        self.c14_416e4(e);
        e.m.sb(f::BUSY, 1);
        e.m.sw(f::TIMER, 0x3C);
        e.b.act = 0x2E;
        self.enemy_anim(&mut e.b, 0x16);
        let t = self.lib_target(e);
        let (tx, tz) = (t.xi(), t.zi());
        put_hi(&mut e.b.z, tz);
        put_hi(&mut e.b.x, tx);
        e.b.x = e.b.x.wrapping_sub(0x40 << 16);
        e.b.dir = 8;
        e.b.facing = 1;
        let d0 = tx.wrapping_sub(self.cam.s2.cam_x as i16);
        if d0 > 0xC0 {
            e.b.x = e.b.x.wrapping_add(0x40 << 16);
            e.b.dir = 0xC;
            e.b.facing = 0;
        }
        self.enemy_begin_attack(&mut e.b, 0x18);
        e.b.set_yi_keep(e.b.yi().wrapping_add(0x120));
        e.b.vy = 0;
        e.b.ay = -0x20;
        e.b.vz = 0;
        e.b.az = 0;
        self.enemy_set_speed(&mut e.b, 5);
    }

    /// 0x419AC (entrance 0xA): walk in with two knives (state 0x30).
    pub(super) fn c14_419ac(&mut self, e: &mut Enemy) {
        self.c14_416e4(e);
        e.b.act = 0x30;
        e.m.sw(f::TIMER, 0x48);
        e.m.sb(f::TURN_T, 0x1E);
        self.lib_slot3_new(e);
        self.lib_face_target(e);
        e.b.dir = 8;
        if e.b.facing == 0 {
            e.b.dir = 0x18;
        }
        self.enemy_set_speed(&mut e.b, 0xD);
        self.enemy_begin_attack(&mut e.b, 0x1C);
        self.enemy_anim(&mut e.b, 2);
    }

    /// 0x41A1C (entrance 0xC): like 0xA, level with the target in Z.
    pub(super) fn c14_41a1c(&mut self, e: &mut Enemy) {
        self.c14_416e4(e);
        e.b.act = 0x30;
        e.m.sw(f::TIMER, 0x48);
        e.m.sb(f::TURN_T, 0x1E);
        let tz = self.lib_target(e).zi();
        put_hi(&mut e.b.z, tz);
        self.lib_slot3_new(e);
        self.lib_face_target(e);
        e.b.dir = 8;
        if e.b.facing == 0 {
            e.b.dir = 0x18;
        }
        self.enemy_set_speed(&mut e.b, 0xD);
        self.enemy_begin_attack(&mut e.b, 0x1C);
        self.enemy_anim(&mut e.b, 3);
    }

    // ------------------------------------------------------------------ the knife parts

    /// 0x41D6E: place the knife parts: one when `+$BE` is 1, two otherwise; a negative count is reset.
    pub(super) fn c14_41d6e(&mut self, e: &mut Enemy) {
        let be = e.m.b(f::KNIVES) as i8;
        if be == 0 {
            return;
        }
        if be < 0 {
            e.m.sb(f::KNIVES, 0);
            return;
        }
        self.c14_41d9e(e, e.m.w(f::PART_A));
        if be == 1 {
            return;
        }
        self.c14_41d9e(e, e.m.w(f::PART_B));
    }

    /// 0x41D9E: one part (class 60): its animation and offset follow the master's frame byte `+$2D`
    /// (tables 0x58178 BLADE / 0x58308 RAZOR, 8 bytes per frame: dx, dy, dz, animation).
    fn c14_41d9e(&mut self, e: &Enemy, a1: u16) {
        let Some(Slot::Enemy(j)) = slot_of(a1) else { return };
        let (live, k20, var) = {
            let p = &self.enemies.slots[j];
            (p.b.live, p.b.k20, p.b.var26)
        };
        if live == 0 || k20 != 0xF0 {
            return;
        }
        let idx = e.b.param_b;
        let row = if (var >> 8) as u8 == 0 { 0x58178 } else { 0x58308 } + idx as u32 * 8;
        let t = &self.assets.enemy.t;
        let mut dx = t.i16(row, 0);
        let dy = t.i16(row, 2);
        let mut anim = t.u16(row, 6);
        if e.b.facing != 0 {
            dx = dx.wrapping_neg();
        }
        if (anim as i16) > 0x2F {
            anim = 0x28;
        }
        let assets = self.assets;
        let p = &mut self.enemies.slots[j];
        p.m.sw(f::TIMER, idx as u16);
        p.b.facing = e.b.facing;
        put_hi(&mut p.b.x, dx.wrapping_add(e.b.xi()));
        put_hi(&mut p.b.y, dy.wrapping_add(e.b.yi()));
        put_hi(&mut p.b.z, e.b.zi());
        p.m.sw(f::TIMER, anim);
        if idx == p.m.b(f::OLD_DIR) {
            return;
        }
        p.m.sb(f::OLD_DIR, idx);
        p.b.start_anim(&assets.sprites, anim);
    }
}
