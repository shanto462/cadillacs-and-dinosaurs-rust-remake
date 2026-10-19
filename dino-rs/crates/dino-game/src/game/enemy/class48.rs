//! Pool A class 48 (`+$20 = 0xC0`, handler 0x56924): a feather that a bird of the opening movie's jungle drops (M1-10,
//! made by class 46, 0x5670A). It floats down and sways: for `+$A8` frames (8 to 11, then 16 to 23) it moves one pixel
//! sideways (to the right while `+$AE` is 0, else to the left; one more pixel to the right now and then, one random
//! number a frame decides: 1 in 4), falls `+$AA` pixels (1 or 2) a frame, then turns and steps its animation.
//!
//! Mode 0 (0x56938) sets up the object (live 2, the table 0x56A56, the palette override 0x83, no shadow, the terrain,
//! the animation 0) and goes on in mode 2 (0x56980): sub 0 (0x569A0) draws the numbers and falls into sub 2 (0x569DA),
//! the far test, the draw list. Mode 4 and 6 free it (0x56A50).

use super::Enemy;
use crate::game::{Game, Handle};

/// Offsets of this class.
mod f {
    /// word: frames of the present swing and the frames it lasts.
    pub const COUNT: usize = 0xAC;
    pub const LENGTH: usize = 0xA8;
    /// word: the fall in pixels a frame.
    pub const FALL: usize = 0xAA;
    /// word: the direction of the swing.
    pub const DIR: usize = 0xAE;
}

const TABLE: u32 = 0x56A56;

impl Game {
    /// 0x56924 class 48 (jump table 0x56930: mode 0 -> 0x56938, 2 -> 0x56980, 4 and 6 -> 0x56A50).
    pub fn enemy_class48(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                e.b.live = 2;
                e.b.sub = 0;
                e.m.sb(super::off::ATTACKING, 0xFF);
                e.b.anim_table = TABLE;
                e.b.palette_override = 0x83;
                e.b.draw_flags = 0x10;
                e.m.sw(0xA0, 0);
                e.m.sw(0xA2, 0x64);
                e.m.sw(0xA6, 0x10);
                if let Some(o) = self.terrain_collide(&mut e.b, h, false) {
                    e.m.sw(0x94, o); // 0x12CB4 leaves the obstacle in +$94
                }
                self.enemy_anim(&mut e.b, 0);
                e.b.mode += 2;
                self.enemy_c48_frame(e, h);
            }
            2 => self.enemy_c48_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x56A50: jmp 0x48FC
            }
        }
    }

    /// 0x56980.
    fn enemy_c48_frame(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => {
                // 0x569A0
                e.b.sub += 2;
                e.m.sw(f::COUNT, 0);
                e.m.sw(f::LENGTH, (self.rand() & 3) + 8);
                e.m.sw(f::FALL, (self.rand() & 1) + 1);
                e.m.sw(f::DIR, 0);
                self.enemy_c48_sway(e);
            }
            2 => self.enemy_c48_sway(e),
            _ => {} // 0x56A4E: rts
        }
        let i = self.enemy_index(h);
        self.enemy_despawn_if_far(e, i); // jsr 0x173C
        self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
    }

    /// 0x569DA.
    fn enemy_c48_sway(&mut self, e: &mut Enemy) {
        let dx: i16 = if e.m.w(f::DIR) == 0 { 1 } else { -1 };
        e.b.set_xi_keep(e.b.xi().wrapping_add(dx));
        if self.rand() & 3 == 0 {
            e.b.set_xi_keep(e.b.xi().wrapping_add(1));
        }
        let c = e.m.w(f::COUNT).wrapping_add(1);
        e.m.sw(f::COUNT, c);
        e.b.set_yi_keep(e.b.yi().wrapping_sub(e.m.w(f::FALL) as i16));
        if c != e.m.w(f::LENGTH) {
            return;
        }
        e.m.sw(f::DIR, (e.m.w(f::DIR) + 1) & 1);
        e.m.sw(f::COUNT, 0);
        e.m.sw(f::LENGTH, (self.rand() & 7) + 0x10);
        self.enemy_step(&mut e.b); // jsr 0x121E
    }
}
