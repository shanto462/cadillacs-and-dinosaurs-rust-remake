//! The dynamite of a bomb carrier, pool A class 77 (handler 0x5EDF4..0x5F0AC): made with its carrier (class 64, `class64.rs`), `+$A0` = the carrier.
//! It shows the carrier's frame byte `+$2C` as its own animation and sits in the hand (offset tables 0x5EF2A / 0x5EF6A by that byte); a negative
//! frame byte throws it. Where it lands the Punk's stick explodes (pool E blast kind 3, 0x27DD8); the Wrench's stick lies as a lit weapon item
//! (pool E class 0, variant 0x18). A knock down of the carrier, or the carrier's death, drops it as a weapon item (0x24B86). D3a 8.2, HU26.
//! Owner: enemy port.

use super::Enemy;
use crate::game::item::{class as item_class, Item};
use crate::game::{slot_of, Game, Handle, Slot};

/// The hand offsets by variant (0x5EF22: two longs pointing at 0x5EF2A and 0x5EF6A; rows of dx, dy words by the carrier's frame byte).
const HAND: [u32; 2] = [0x5EF2A, 0x5EF6A];

impl Game {
    /// 0x5EDF4 (mode table 0x5EE02): 0 the init 0x5EE0A (falls into the run), 2 the run 0x5EE58, 4 the drop 0x5F088, 6 the free 0x5F0A6.
    pub(super) fn enemy_class77(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.c77_init(e);
                self.c77_run(e, h, i);
            }
            2 => self.c77_run(e, h, i),
            4 => self.c77_drop(e, i),
            _ => self.enemy_free_pool_a(e, i), // 0x5F0A6: jmp 0x48FC
        }
    }

    /// 0x5EE0A: the variant becomes the carrier's high byte (0 or 2); the table 0x5F0AC (Punk) or 0x5F128 (Wrench); animation 0; not counted;
    /// `+$A2` = 1; no shadow (`+$25` 0x10); `+$63` 0xFF; the box table 0x103000.
    fn c77_init(&mut self, e: &mut Enemy) {
        e.b.var26 >>= 8;
        e.b.anim_table = if e.b.var26 == 0 { 0x5F0AC } else { 0x5F128 };
        self.enemy_anim(&mut e.b, 0);
        e.m.sb(0x7D, 0xFF);
        e.m.sb(0xA2, 1);
        e.b.draw_flags = 0x10;
        e.b.b63 = 0xFF;
        e.b.mode = e.b.mode.wrapping_add(2);
        e.b.box_table = 0x103000;
        e.b.multi_box = false;
    }

    /// The carrier's body (a cleared body when the handle is not a pool A slot).
    fn c77_carrier(&self, e: &Enemy) -> crate::game::body::Body {
        match slot_of(e.m.w(0xA0)) {
            Some(Slot::Enemy(c)) => self.enemies.slots[c].b.clone(),
            _ => crate::game::body::Body::default(),
        }
    }

    /// 0x5EE58: by `+$05` (table 0x5EEA8: 0 in the hand 0x5EEAC, 2 the flight 0x5F020); then, with the carrier gone (not live, or mode 4 and up,
    /// signed) the drop 0x5F088; else the terrain (0x12CB4), the draw check unless the carrier is in its hit reaction or held (`+$05` 2 or 6), the
    /// monster list (0x4C0A) and the far test (0x173C).
    fn c77_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.sub == 0 {
            self.c77_in_hand(e, h);
        } else {
            self.c77_flight(e, h);
        }
        let c = self.c77_carrier(e);
        if c.live == 0 || (c.mode as i8) >= 4 {
            return self.c77_drop(e, i);
        }
        if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
            e.m.sw(super::off::OBSTACLE, ob);
        }
        let c = self.c77_carrier(e);
        if c.sub != 2 && c.sub != 6 {
            self.enemy_draw_check(&mut e.b, h);
        }
        self.enemy_monster_register(h);
        self.enemy_despawn_if_far(e, i); // jmp 0x173C
    }

    /// 0x5EEAC: in the hand. A knock down of the carrier drops the stick (0x5EFAA); a negative frame byte of the carrier throws it (0x5EFE8); else the
    /// animation of that frame byte (0x120E) and the place in the hand (0x5EED0).
    fn c77_in_hand(&mut self, e: &mut Enemy, h: Handle) {
        if self.c77_knock_drop(e) {
            return;
        }
        let c = self.c77_carrier(e);
        if (c.param_a as i8) < 0 {
            return self.c77_throw(e, h);
        }
        self.enemy_anim(&mut e.b, c.param_a as u16);
        self.c77_follow(e, &c);
    }

    /// 0x5EED0: the carrier's facing; X = carrier X + dx (negated when it faces right), Y = carrier Y + dy, Z = carrier Z (the Wrench's stick 1
    /// nearer and 1 higher), by the row of the carrier's frame byte.
    fn c77_follow(&self, e: &mut Enemy, c: &crate::game::body::Body) {
        e.b.facing = c.facing;
        let table = HAND[(e.b.var26 != 0) as usize];
        let row = (c.param_a as u32) * 4;
        let t = &self.assets.enemy.t;
        let mut dx = t.i16(table, row);
        if c.facing != 0 {
            dx = dx.wrapping_neg();
        }
        let dy = t.i16(table, row + 2);
        let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
        put(&mut e.b.x, c.xi().wrapping_add(dx));
        put(&mut e.b.y, c.yi().wrapping_add(dy));
        let mut z = c.zi();
        if e.b.var26 != 0 {
            z = z.wrapping_sub(1);
            let y = e.b.yi().wrapping_add(1);
            put(&mut e.b.y, y);
        }
        put(&mut e.b.z, z);
    }

    /// 0x5EFAA: the carrier in its hit reaction (`+$04` word 0x0202) with the knock down reaction 4: mode 6 and the stick dropped as a weapon item
    /// (0x24B86: code 0xE for the Punk's, 0xD for the Wrench's; after the second the original also writes this object's `+$07` = 2). The carry.
    fn c77_knock_drop(&mut self, e: &mut Enemy) -> bool {
        let c = self.c77_carrier(e);
        if (c.mode, c.sub) != (2, 2) || c.reaction != 4 {
            return false;
        }
        e.b.mode = 6;
        self.c77_weapon_item(e);
        true
    }

    /// 0x24B86 with d1 = 0xE (variant 0) or 0xD: a weapon item (pool E class 0, variant (d1 - 1) * 2, act 2) at this object; the Wrench's case then
    /// writes `+$07` = 2 of this object (0x5EFD8 / 0x5F0A0, the item's register is not used).
    fn c77_weapon_item(&mut self, e: &mut Enemy) {
        let d1: u16 = if e.b.var26 == 0 { 0xE } else { 0xD };
        let pos = (e.b.xi(), e.b.yi(), e.b.zi());
        self.item_spawn(item_class::WEAPON * 4, pos, Some((d1 - 1) * 2));
        if e.b.var26 != 0 {
            e.b.step = 2;
        }
    }

    /// 0x5EFE8: the throw: `+$05` = 2, `+$A2` = 0, the draw flags 0, in the air, vx +-0x400 (- when facing left), vy 0, gravity -0x30; falls into the
    /// flight.
    fn c77_throw(&mut self, e: &mut Enemy, h: Handle) {
        e.b.sub = e.b.sub.wrapping_add(2);
        e.m.sb(0xA2, 0);
        e.b.draw_flags = 0;
        e.b.airborne = 1;
        e.b.vx = if e.b.facing != 0 { 0x400 } else { -0x400 };
        e.b.ax = 0;
        e.b.vy = 0;
        e.b.ay = -0x30;
        e.b.vz = 0;
        e.b.az = 0;
        self.c77_flight(e, h);
    }

    /// 0x5F020: the flight (0x189C, 0x121E) until the landing; then the Punk's stick explodes (nobody credited: `+$02` = 0xFF; the pool E blast of
    /// kind 3 by 0x27DD8; mode 6) and the Wrench's lies as a lit weapon item (pool E class 0, variant 0x18, `+$07` = 2, at this object; mode 6).
    fn c77_flight(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.airborne != 0 {
            e.b.motion();
            self.enemy_step(&mut e.b);
            return;
        }
        if e.b.var26 == 0 {
            e.b.who = 0xFF;
            let src = Item { b: e.b.clone(), ..Default::default() };
            self.item_explosion(&src, h, 3); // 0x27DD8 with d0 = 3
            e.b.mode = 6;
            return;
        }
        let pos = (e.b.xi(), e.b.yi(), e.b.zi());
        if let Some(j) = self.item_spawn(item_class::WEAPON * 4, pos, Some(0x18)) {
            let it = &mut self.items.slots[j];
            it.b.act = 0; // 0x5F04E builds the item by hand: no act 2 (item_spawn sets it)
            it.b.step = 2;
        }
        e.b.mode = 6;
    }

    /// 0x5F088: mode 4 and the carrier gone: the weapon item (0x24B86, as 0x5EFAA) and the free (0x48FC).
    fn c77_drop(&mut self, e: &mut Enemy, i: usize) {
        self.c77_weapon_item(e);
        self.enemy_free_pool_a(e, i);
    }
}
