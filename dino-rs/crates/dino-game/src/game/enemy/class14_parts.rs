//! The objects class 14 (BLADE / RAZOR) makes: the two knife parts (class 60, 0x580C6), the knife that
//! drops when a hit Blade is in the air (class 34, 0x5129E) and the thrown blade (class 41, 0x55866).
//! Phase 1 task 4a2. See `class14.rs`.

use super::class14::f;
use super::lib32::put_hi;
use super::{Enemy, off::*};
use crate::game::{Game, Handle, Slot, slot_of};

impl Game {
    // ------------------------------------------------------------------ class 60: the knife parts

    /// 0x580C6 class 60: a part follows its master (`+$A0`, a long) and is drawn while the master holds
    /// knives (the first part for one or two, the second only for two). It is placed by 0x41D6E.
    pub fn enemy_class60(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => self.c60_init(e),
            2 => self.c60_alive(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x58172
            }
        }
    }

    /// 0x580DA.
    fn c60_init(&mut self, e: &mut Enemy) {
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_begin_attack(&mut e.b, 0x1E);
        e.b.live = 2;
        e.m.sb(ATTACKING, 0xFF);
        e.b.b63 = 0xFF;
        e.b.anim_table = 0x6FB6A;
        e.b.draw_flags = 0x10;
        e.m.sb(0xA5, 0);
        e.b.hitstop = 0;
        self.enemy_anim(&mut e.b, 0x22);
        self.enemy_floor_collision(e); // 0x12CB4
        e.b.mode += 2;
    }

    /// 0x5813C.
    fn c60_alive(&mut self, e: &mut Enemy, h: Handle) {
        let m = e.m.l(0xA0) as u16;
        let Some(Slot::Enemy(mi)) = slot_of(m) else {
            let i = self.enemy_index(h);
            return self.enemy_free_pool_a(e, i);
        };
        let (mode, knives) = {
            let master = &self.enemies.slots[mi];
            (master.b.mode, master.m.b(f::KNIVES))
        };
        if (mode as i8) >= 4 {
            let i = self.enemy_index(h);
            return self.enemy_free_pool_a(e, i); // 0x58172
        }
        if knives == 0 {
            return;
        }
        if (e.b.var26 >> 8) as u8 != 0 && knives == 1 {
            return;
        }
        self.enemy_step(&mut e.b);
        self.enemy_draw_check(&mut e.b, h); // 0x49CA
    }

    // ------------------------------------------------------------------ class 34: the dropped knife

    /// 0x5129E class 34 (KNIFE): thrown up and away by a hit Blade, falls, and where it lands it
    /// becomes a pick up knife (pool E weapon, variant 4) unless two guns lie about.
    pub fn enemy_class34(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => self.c34_init(e, h),
            2 => self.c34_alive(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x513D4
            }
        }
    }

    /// 0x512B2.
    fn c34_init(&mut self, e: &mut Enemy, h: Handle) {
        e.b.id = 0x17;
        e.m.sw(W84, 0x5C);
        e.m.sb(B87, 0);
        e.m.sb(0xAF, 0);
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        e.b.live = 2;
        e.m.sb(ATTACKING, 0xFF);
        e.b.anim_table = 0x6FB6A;
        e.b.draw_flags = 0x10;
        e.m.sb(0xA5, 0);
        self.enemy_floor_collision(e);
        let id = if e.b.var26 != 0 { 0x15 } else { 0x16 };
        self.enemy_anim(&mut e.b, id);
        e.b.mode += 2;
        e.b.act = 4;
        e.b.airborne = 1;
        e.b.vx = 0x100;
        e.b.vy = 0x680;
        e.b.ax = 0;
        e.b.ay = -0x45; // 0xFFBB
        if e.b.facing != 0 {
            e.b.vx = e.b.vx.wrapping_neg(); // 0x5133E
        }
        self.enemy_anim(&mut e.b, 0x20);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x51350.
    fn c34_alive(&mut self, e: &mut Enemy, h: Handle) {
        e.b.motion(); // 0x189C
        self.enemy_floor_collision(e);
        if e.b.airborne != 0 {
            self.enemy_step(&mut e.b);
            return self.enemy_draw_check(&mut e.b, h);
        }
        let off = (self.spawn.g.b4fa as i8) >= 2
            || (self.w.stage == 7 && self.cam.s2.area == 1 && e.b.xi() >= 0xB80);
        if !off {
            self.enemy_anim(&mut e.b, 0);
            self.c34_make_knife_item(e);
        }
        e.b.mode = 4;
    }

    /// 0x4818 + 0x5139C / 0x55A4C: a pick up knife (pool E class 0, variant 4) at the object.
    fn c34_make_knife_item(&mut self, e: &Enemy) {
        let Some(j) = self.items.alloc() else { return };
        let n = &mut self.items.slots[j];
        n.b.live = 1;
        n.b.k20 = 0;
        n.b.var26 = 4;
        put_hi(&mut n.b.x, e.b.xi());
        put_hi(&mut n.b.y, e.b.yi());
        put_hi(&mut n.b.z, e.b.zi());
    }

    // ------------------------------------------------------------------ class 41: the thrown blade

    /// 0x55866 class 41: a blade that Razor throws; it flies straight at 6 px per frame and hurts a
    /// player (attack id 0x19). A hit blade falls and becomes a pick up knife.
    pub fn enemy_class41(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode & 6 {
            0 => self.c41_init(e, h),
            2 => self.c41_alive(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x55A78
            }
        }
    }

    /// 0x5587E and 0x5594A.
    fn c41_init(&mut self, e: &mut Enemy, h: Handle) {
        e.b.id = 0x17;
        e.m.sw(W84, 0x5C);
        e.b.anim_table = 0x6FB6A;
        e.m.sb(B87, 1);
        e.m.sb(0xAF, 0);
        e.b.hitstop = 0;
        e.b.b63 = 0xFF;
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        e.b.live = 1;
        e.m.sb(ATTACKING, 0xFF);
        e.b.mode = 2;
        // 0x5594A: placed in front of the master.
        let master = e.m.l(0xA0) as u16;
        let (mf, my, mz) = {
            let b = self.body(master);
            (b.facing, b.yi(), b.zi())
        };
        e.b.airborne = 1;
        e.b.facing = mf;
        e.b.dir = 8;
        e.b.x = e.b.x.wrapping_add(0x40 << 16);
        if e.b.facing == 0 {
            e.b.dir = 0x18;
            e.b.x = e.b.x.wrapping_sub(0x80 << 16);
        }
        put_hi(&mut e.b.y, my);
        e.b.y = e.b.y.wrapping_add(0x40 << 16);
        put_hi(&mut e.b.z, mz);
        e.b.z = e.b.z.wrapping_add(1 << 16);
        e.m.sl(0xAA, 0x60000);
        if e.b.facing == 0 {
            e.m.sl(0xAA, 0x60000u32.wrapping_neg());
        }
        self.enemy_begin_attack(&mut e.b, 0x19);
        self.enemy_anim(&mut e.b, 0x21);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x558CC: the state `+$05` through the long table 0x558E0.
    fn c41_alive(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => self.c41_fly(e, h),
            2 => self.c41_hit(e),
            6 => self.enemy_grabbed(e, h), // 0x2B35A
            8 => self.c41_fall(e, h),
            s => panic!("class 41 sub {s:#x}"),
        }
        let i = self.enemy_index(h);
        self.enemy_despawn_if_far(e, i); // jmp 0x173C
    }

    /// 0x558F4: fly. A set `+$AF` or a hit stop ends the flight (the blade was hit: state 2 / gone).
    fn c41_fly(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.b(0xAF) != 0 {
            e.b.sub = 2;
        } else if e.b.hitstop != 0 {
            e.b.mode = 4; // 0x55942
            return;
        }
        // 0x5590E: wall bit 7 (a wall in the way) ends it; the terrain test is never reached.
        if e.b.wall & 0x80 != 0 {
            e.b.mode = 4;
            return;
        }
        let i = self.enemy_index(h);
        self.enemy_despawn_if_far(e, i);
        e.b.x = e.b.x.wrapping_add(e.m.l(0xAA) as i32);
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h); // 0x4BEA
    }

    /// 0x559CC: the blade was hit: it drops (state 8).
    fn c41_hit(&mut self, e: &mut Enemy) {
        e.b.vx = 0;
        e.b.vy = 0x400;
        e.b.ax = 0;
        e.b.ay = -0x40;
        e.b.az = 0;
        e.b.vz = 0;
        e.b.airborne = 1;
        if e.b.facing != 0 {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        self.enemy_anim(&mut e.b, 0x20);
        self.snd_96ce();
        e.b.sub = 8;
    }

    /// 0x55A24: the fall; it ends as a pick up knife.
    fn c41_fall(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_step(&mut e.b);
        e.b.motion();
        self.enemy_floor_collision(e);
        if e.b.airborne != 0 {
            return self.enemy_draw_check(&mut e.b, h);
        }
        self.c34_make_knife_item(e);
        e.b.mode = 4;
    }
}
