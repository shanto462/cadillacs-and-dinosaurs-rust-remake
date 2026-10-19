//! Pool A class 74 (handler 0x5E4F4..0x5E702): Tyrog's chest blades (table 0x71936). The owner (class 4, `+$A0`) makes one in the blade states
//! (variant 0 blade A with attack 0x50, variant 0x100 blade B with attack 0x6B). It has 20 HP, is on the hurtable list (a hit gives its hit stop
//! to the owner) and lives while the owner's frame byte `+$2D` is set; each time that byte changes it drops a spark (class 75 variant 3) along
//! the blade. D3d 3.5, BT10. Owner: enemy port.

use super::{off::*, Enemy};
use crate::game::{slot_of, Game, Handle, Slot};

/// 0x5E6D4: the X offsets of the sparks by the owner's frame byte (words, the byte is the offset).
const SPARK_DX: u32 = 0x5E6D4;

impl Game {
    /// 0x5E4F4 (mode table 0x5E500): 0 the init 0x5E508 (falls into 2), 2 the run 0x5E5DE, 4 and 6 the free 0x5E6FC.
    pub(super) fn enemy_class74(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.c74_init(e);
                self.c74_run(e, h, i);
            }
            2 => self.c74_run(e, h, i),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// The owner's index in pool A (`+$A0`).
    fn c74_owner(e: &Enemy) -> Option<usize> {
        match slot_of(e.m.w(0xA0)) {
            Some(Slot::Enemy(k)) => Some(k),
            _ => None,
        }
    }

    /// 0x5E508: `+$00` 2, not counted, `+$63` 0xFF, the table 0x71936, the box table 0x103C00, attack 0x50, no shadow; the palette and `+$7D` of
    /// the owner's form; HP 20; `+$A2` 0x64, `+$A6` 0x10; variant 0x100 attack 0x6B and animation 1, else animation 0.
    fn c74_init(&mut self, e: &mut Enemy) {
        e.b.live = 2;
        e.b.sub = 0;
        e.m.sb(ATTACKING, 0xFF);
        e.b.b63 = 0xFF;
        e.b.anim_table = 0x71936;
        e.b.hitstop = 0;
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_begin_attack(&mut e.b, 0x50);
        e.b.draw_flags = 0x10;
        e.b.palette_override = 0x9F;
        let form = Self::c74_owner(e).map(|k| (self.enemies.slots[k].b.var26 >> 8) as u8).unwrap_or(0);
        if form == 1 {
            e.b.palette_override = 0x9E;
            e.m.sb(ATTACKING, 0xFE);
        }
        if form == 2 {
            e.b.palette_override = 0x9D;
            e.m.sb(ATTACKING, 0xFE);
        }
        e.m.sw(0xB2, 0);
        e.m.sw(0xA2, 0x64);
        e.b.hp_bar = 0x14;
        e.b.hp = 0x14;
        e.b.hp_prev = 0x14;
        e.m.sw(0xA6, 0x10);
        let mut anim = 0;
        if (e.b.var26 >> 8) as u8 != 0 {
            anim = 1;
            self.enemy_begin_attack(&mut e.b, 0x6B);
        }
        self.enemy_anim(&mut e.b, anim);
        e.b.mode = e.b.mode.wrapping_add(2);
    }

    /// 0x5E5DE: when the owner's `+$2D` is 0 mode 4 (freed next frame); else by `+$05` (table 0x5E612: 0 and 2 the blade, 4 mode 4); then the
    /// far test (0x173C), the hurtable list (0x4BEA) and the draw check (0x49CA).
    fn c74_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        let owner = Self::c74_owner(e);
        let pb = owner.map(|k| self.enemies.slots[k].b.param_b).unwrap_or(0);
        if pb == 0 {
            e.b.mode = 4;
            return;
        }
        match e.b.sub {
            0 | 2 => {
                e.b.sub = 2;
                self.c74_blade(e, owner);
            }
            _ => e.b.mode = e.b.mode.wrapping_add(2),
        }
        self.enemy_despawn_if_far(e, i);
        self.enemy_push_hurtable(h);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x5E61E: the animation (at its end mode 4); its hit stop goes to the owner; with the owner hit (`+$05` 2 or more) mode 4; while the owner
    /// is in a hit stop the animation waits (`+$2F` 1); else a spark when its `+$2D` changed to a value that is not negative.
    fn c74_blade(&mut self, e: &mut Enemy, owner: Option<usize>) {
        if self.enemy_step(&mut e.b) {
            e.b.mode = e.b.mode.wrapping_add(2);
            return;
        }
        let Some(k) = owner else { return };
        if e.b.hitstop != 0 {
            self.enemies.slots[k].b.hitstop = e.b.hitstop;
            e.b.hitstop = 0;
        }
        let ob = &self.enemies.slots[k].b;
        if (ob.sub as i8) >= 2 {
            e.b.mode = e.b.mode.wrapping_add(2);
            return;
        }
        if ob.hitstop != 0 {
            e.b.timer = 1;
            return;
        }
        let pb = e.b.param_b;
        if e.m.b(0xB0) == pb {
            return;
        }
        e.m.sb(0xB0, pb);
        if (pb as i8) < 0 {
            return;
        }
        let Some(j) = self.enemies.alloc() else { return };
        let dx = self.assets.enemy.t.i16(SPARK_DX, pb as u32);
        let x = if e.b.facing != 0 { e.b.xi().wrapping_add(dx) } else { e.b.xi().wrapping_sub(dx) };
        let n = &mut self.enemies.slots[j];
        n.b.live = 1;
        n.b.k20 = 75 * 4;
        n.b.var26 = 3;
        n.b.facing = e.b.facing;
        let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
        put(&mut n.b.x, x);
        put(&mut n.b.z, e.b.zi().wrapping_sub(1));
        put(&mut n.b.y, e.b.yi().wrapping_sub(0x30));
    }
}
