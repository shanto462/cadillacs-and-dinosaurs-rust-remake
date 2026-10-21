//! The Butcher's knife, class 52 (handler 0x57298, 920 bytes plus the offset tables 0x57480 / 0x57558, read by `class61.rs`). D3c section 3.4.
//!
//! A knife is a drawn part of his body: while he holds it (sub 0) it copies his facing and shows the animation the Butcher chose
//! (`+$A6` = 0x5A hides it); its position is set by the Butcher every frame (`c61_knife_follow`). When a knockdown makes him drop it, the
//! Butcher sets `+$05` = 8: the knife flies off (sub 8 then sub 4), lands, turns into a pool E item (class 0, variant 0x12, the Butcher
//! knife a hero can pick up) and frees itself. It frees itself at once when the Butcher is dead (his mode 4). Owner: enemy port (M6-10,
//! started here because the Butcher's tail writes into the knives; M6-11 checks the rest).

use super::lib32::put_hi;
use super::Enemy;
use crate::game::{slot_of, Game, Handle, Slot};

impl Game {
    /// 0x57298: class 52. Modes (0x572A4): 0 init, 2 run, 4 and 6 free.
    pub(super) fn enemy_class52(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.c52_init(e);
                self.c52_run(e, h, i);
            }
            2 => self.c52_run(e, h, i),
            _ => self.enemy_free_pool_a(e, i), // 0x5747A
        }
    }

    /// 0x572AC.
    fn c52_init(&mut self, e: &mut Enemy) {
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        e.b.live = 2;
        e.b.anim_table = 0x6F8A6;
        e.b.draw_flags = 0x10;
        e.m.sb(0xA5, 0);
        e.m.sb(0x7D, 0xFF);
        self.enemy_snap_to_floor(e); // 0x3140C
        self.enemy_floor_collision(e); // 0x12CB4
        self.enemy_anim(&mut e.b, 0x1B);
        e.b.mode = e.b.mode.wrapping_add(2);
    }

    /// 0x572F0: the owner dead (mode 4 or more): free. Else the sub state (0x57312: 0 held, 2 sliding, 4 falling, 6 and 8 start to fall)
    /// and the playfield clamp 0x193E.
    fn c52_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        let owner = e.m.w(0xA2);
        if let Some(Slot::Enemy(o)) = slot_of(owner)
            && self.enemies.slots[o].b.mode >= 4 {
                return self.enemy_free_pool_a(e, i);
            }
        match e.b.sub {
            0 => self.c52_held(e, h),
            2 => {
                // 0x5735A: slides with the long speed `+$AA`.
                e.b.x = e.b.x.wrapping_add(e.m.l(0xAA) as i32);
                self.enemy_draw_check(&mut e.b, h);
            }
            4 => self.c52_falling(e, h),
            6 => self.c52_6(e, h),
            8 => self.c52_8(e, h),
            _ => {}
        }
        self.enemy_clamp_to_playfield(e, i); // jmp 0x193E
    }

    /// 0x5731C: held: his facing, the animation (`+$A5` is cleared at the entry and set at the end of the animation, so it never holds
    /// it back), drawn unless the Butcher hid it (`+$A6` = 0x5A).
    fn c52_held(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sb(0xA5, 0);
        if let Some(Slot::Enemy(o)) = slot_of(e.m.w(0xA2)) {
            e.b.facing = self.enemies.slots[o].b.facing;
        }
        if self.enemy_step(&mut e.b) {
            e.m.sb(0xA5, 1);
        }
        if e.m.w(0xA6) != 0x5A {
            self.enemy_draw_check(&mut e.b, h);
        }
    }

    /// 0x57368: falling: the motion; on the floor (height below 0, or no longer airborne with a height) it becomes the item and frees
    /// itself next frame (mode 4). Else the animation and the draw check.
    fn c52_falling(&mut self, e: &mut Enemy, h: Handle) {
        e.b.motion();
        if e.b.yi() < 0 || e.b.airborne == 0 {
            self.enemy_anim(&mut e.b, 0);
            if let Some(a) = self.items.alloc() {
                let (x, y, z) = (e.b.xi(), e.b.yi(), e.b.zi());
                let it = &mut self.items.slots[a];
                it.b.live = 1;
                it.b.k20 = 0;
                it.b.var26 = 0x12;
                put_hi(&mut it.b.x, x);
                put_hi(&mut it.b.y, y);
                put_hi(&mut it.b.z, z);
                let item = self.items.handle(a);
                if let Some(Slot::Enemy(o)) = slot_of(e.m.w(0xA2)) {
                    let field = if e.b.var26 >> 8 == 0 { 0xB4 } else { 0xB6 };
                    self.enemies.slots[o].m.sw(field, item);
                }
            }
            e.b.mode = 4;
            return;
        }
        self.enemy_step(&mut e.b);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x573DE: starts to slide away at his side (X + 0x40, or - 0x40 when he faces the other way), height 0x40 above his, speed 0x58000.
    fn c52_6(&mut self, e: &mut Enemy, h: Handle) {
        let Some(Slot::Enemy(o)) = slot_of(e.m.w(0xA2)) else { return };
        let (facing, oy) = (self.enemies.slots[o].b.facing, self.enemies.slots[o].b.yi());
        e.b.sub = 2;
        e.b.facing = facing;
        e.b.dir = 8;
        e.b.set_xi_keep(e.b.xi().wrapping_add(0x40));
        if e.b.facing == 0 {
            e.b.dir = 0x18;
            e.b.set_xi_keep(e.b.xi().wrapping_sub(0x80));
        }
        e.b.set_yi_keep(oy.wrapping_add(0x40));
        e.m.sl(0xAA, if e.b.facing == 0 { 0x58000u32.wrapping_neg() } else { 0x58000 });
        self.enemy_anim(&mut e.b, 8);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x5743C: the drop (the Butcher set `+$05` = 8): sub 4, airborne, `vx` 0x100 toward the facing side, `vy` 0x680, gravity 0xFFBB, animation 0x20.
    fn c52_8(&mut self, e: &mut Enemy, h: Handle) {
        e.b.sub = 4;
        e.b.airborne = 1;
        e.b.vx = 0x100;
        e.b.vy = 0x680;
        e.b.ax = 0;
        e.b.ay = 0xFFBBu16 as i16;
        if e.b.facing == 0 {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        self.enemy_anim(&mut e.b, 0x20);
        self.enemy_draw_check(&mut e.b, h);
    }
}
