//! Hogg's motorbike, class 32 (handler 0x50FAC..0x51076): made by the rider's init (`class31.rs`, 0x4FCEE), it keeps the rider in `+$A0`.
//! The rider moves it; the bike only animates, collides with the terrain and draws itself, and breaks up when the rider's HP is 0.
//! D3c section 4.1. Owner: enemy port.

use super::{off::*, Enemy};
use crate::game::{Game, Handle};

impl Game {
    /// 0x50FAC: class 32 (mode table 0x50FB8): 0 the init (0x50FD0, the only entry of the table 0x50FCE), 2 the ride (0x5101E), 4 the
    /// break (0x5103A), 6 the free (0x5104C).
    pub(super) fn enemy_class32(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => self.c32_init(e),
            2 => {
                if self.body(e.m.w(0xA0)).hp != 0 {
                    self.enemy_step(&mut e.b); // 0x121E
                    if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
                        e.m.sw(OBSTACLE, ob); // 0x12CB4
                    }
                    self.enemy_draw_check(&mut e.b, h); // 0x49CA
                } else {
                    self.c32_break(e);
                }
            }
            4 => self.c32_break(e),
            _ => self.enemy_free_pool_a(e, i), // 0x48FC
        }
    }

    /// 0x50FD0: the table 0x51076, animation 0, HP by its name (0x31424), the poise 0x1E, the floor (0x3140C), not counted (`+$7D`).
    fn c32_init(&mut self, e: &mut Enemy) {
        e.b.anim_table = 0x51076;
        self.enemy_anim(&mut e.b, 0);
        self.enemy_set_hp(&mut e.b); // 0x31424
        e.b.stun = 0x1E;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.m.sb(ATTACKING, 0);
        e.m.sb(B87, 0);
        e.b.b63 = 0;
        e.b.b72 = 0;
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sb(ATTACKING, 0xFF);
        e.b.mode = e.b.mode.wrapping_add(2);
    }

    /// 0x5103A: the rider is down: mode 6 (freed in the next frame) and the debris shower 0x94640 (pool C class 21, variant 0x1C) with the
    /// bike's facing. The pieces and the explosion where they land are M6-26.
    fn c32_break(&mut self, e: &mut Enemy) {
        e.b.mode = 6;
        let b = e.b.clone();
        self.fx_spawn_class21(&b, 0x1C, b.facing);
    }
}
