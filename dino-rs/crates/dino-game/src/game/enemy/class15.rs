//! Pool A class 15 (`+$20 = 0x3C`, handler 0x41F1A): a flying piece (debris) that pool B props throw
//! when they break, made by the spawner 0x41EEA. Phase 1 task 4a2. Stage 1 uses it through the prop
//! B#40 (0x836A4: the unported pool B class 40 calls the spawner twice every 6 frames with the
//! pieces 0 and 2). It sits in the address range of class 14 but is not one of its parts.
//!
//! `+$26` is the row of the table 0x42050 (a word offset: 0, 2, 4, 6, 8): the offset of the start
//! position (dx, dy, dz as words, dx mirrored by the facing), the three speeds (longs, 16.16, the X one
//! mirrored too) and an attack flag. A piece with the flag set is an attacker (attack id 0x23, box
//! table 0x100000) and is registered as hurtable. It lives until its animation ends.

use super::lib32::put_hi;
use super::{Enemy, off::*};
use crate::game::{Game, Handle};

/// Offsets (this class' meaning).
mod f {
    pub const VX: usize = 0xA0; // long
    pub const VY: usize = 0xA4; // long
    pub const VZ: usize = 0xA8; // long
    pub const ATTACK: usize = 0xAC; // byte
}

const ROWS: u32 = 0x42050;

impl Game {
    /// 0x41EEA: a new piece at the position of the caller (`pos` = its X, Y, Z words), row `kind` (`d0`, the
    /// byte offset into the table of offsets) and facing `facing` (`d1`). The pool being empty loses it.
    pub fn enemy_spawn_piece(&mut self, pos: (i16, i16, i16), kind: u16, facing: u8) {
        let Some(j) = self.enemies.alloc() else { return };
        let n = &mut self.enemies.slots[j];
        n.b.live = 1;
        n.b.k20 = 0x3C;
        put_hi(&mut n.b.x, pos.0);
        put_hi(&mut n.b.y, pos.1);
        put_hi(&mut n.b.z, pos.2);
        n.b.var26 = kind;
        n.b.facing = facing;
    }

    /// 0x41F1A class 15.
    pub fn enemy_class15(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                self.c15_init(e);
                self.c15_alive(e, h); // 0x41FC8 falls into 0x41FCC: the first frame already moves the piece
            }
            2 => self.c15_alive(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x42008
            }
        }
    }

    /// 0x41F2E.
    fn c15_init(&mut self, e: &mut Enemy) {
        e.b.anim_table = 0x4200E;
        e.b.draw_flags = 0x10;
        e.m.sb(ATTACKING, 0xFF);
        e.b.hitstop = 0;
        e.b.b63 = 0xFF;
        let t = &self.assets.enemy.t;
        let row = ROWS.wrapping_add(t.u16(ROWS, e.b.var26 as u32) as i16 as i32 as u32);
        let left = e.b.facing == 0;
        let dx = t.i16(row, 0);
        let (dy, dz) = (t.i16(row, 2), t.i16(row, 4));
        let (vx, vy, vz) = (t.u32(row, 6), t.u32(row, 10), t.u32(row, 14));
        let attack = t.u8(row, 18);
        let (x, y, z) = (e.b.xi(), e.b.yi(), e.b.zi());
        put_hi(&mut e.b.x, x.wrapping_add(if left { dx } else { dx.wrapping_neg() }));
        put_hi(&mut e.b.y, y.wrapping_add(dy));
        put_hi(&mut e.b.z, z.wrapping_add(dz));
        e.m.sl(f::VX, if left { vx } else { vx.wrapping_neg() });
        e.m.sl(f::VY, vy);
        e.m.sl(f::VZ, vz);
        e.m.sb(f::ATTACK, attack);
        if attack != 0 {
            e.b.box_table = 0x100000;
            e.b.multi_box = false;
            self.enemy_begin_attack(&mut e.b, 0x23);
        }
        self.enemy_anim(&mut e.b, 0);
        e.b.mode += 2;
    }

    /// 0x41FCC.
    fn c15_alive(&mut self, e: &mut Enemy, h: Handle) {
        if self.enemy_step(&mut e.b) {
            let i = self.enemy_index(h);
            return self.enemy_free_pool_a(e, i); // 0x42008
        }
        e.b.x = e.b.x.wrapping_add(e.m.l(f::VX) as i32);
        e.b.y = e.b.y.wrapping_add(e.m.l(f::VY) as i32);
        e.b.z = e.b.z.wrapping_add(e.m.l(f::VZ) as i32);
        if e.m.b(f::ATTACK) != 0 {
            self.enemy_push_hurtable(h); // 0x4BEA
        }
        self.enemy_draw_check(&mut e.b, h); // 0x49CA
        let i = self.enemy_index(h);
        self.enemy_despawn_if_far(e, i); // jmp 0x173C
    }
}
