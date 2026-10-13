//! Pool A class 69 (`+$20 = 0x114`, handler 0x5C2C6): a piece of the big dinosaur's head that is drawn on the screen
//! while the movie's second caption shows (M1-11; the movie handler of the counter 1250 creates two, variants 0 and 1,
//! 0x9C3E4). It never moves on its own: every frame it is put at the scroll2 camera X + 0xC0 and the camera Y + `+$A2`
//! (0x88, 0x60 for the variant 1), with the palette override 0x8D, and drawn; after 0x50 frames it frees itself.
//!
//! Mode 0 (0x5C2DA) sets up the object (live 2, no census, `+$63` negative, the table 0x718C2, the animation 0 or 1 for
//! the variant 1, `+$A2`, `+$A0` = 0x50) and ends the frame there; mode 2 is 0x5C324; mode 4 and 6 free it (0x5C350).

use super::lib32::put_hi;
use super::Enemy;
use crate::game::{Game, Handle};

/// Offsets of this class.
mod f {
    /// word: frames to live.
    pub const LIFE: usize = 0xA0;
    /// word: the offset from the camera Y.
    pub const DY: usize = 0xA2;
}

const TABLE: u32 = 0x718C2;

impl Game {
    /// 0x5C2C6 class 69 (jump table 0x5C2D2: mode 0 -> 0x5C2DA, 2 -> 0x5C324, 4 and 6 -> 0x5C350).
    pub fn enemy_class69(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                e.b.live = 2;
                e.m.sb(super::off::ATTACKING, 0xFF);
                e.b.b63 = 0xFF;
                e.b.anim_table = TABLE;
                self.enemy_anim(&mut e.b, 0);
                e.m.sw(f::DY, 0x88);
                if e.b.var26 != 0 {
                    self.enemy_anim(&mut e.b, 1);
                    e.m.sw(f::DY, 0x60);
                }
                e.m.sw(f::LIFE, 0x50);
                e.b.mode += 2;
            }
            2 => {
                e.b.palette_override = 0x8D;
                put_hi(&mut e.b.x, (self.cam.s2.cam_x as i16).wrapping_add(0xC0));
                put_hi(&mut e.b.y, (self.cam.s2.cam_y as i16).wrapping_add(e.m.w(f::DY) as i16));
                let t = e.m.w(f::LIFE).wrapping_sub(1);
                e.m.sw(f::LIFE, t);
                if t == 0 {
                    let i = self.enemy_index(h);
                    return self.enemy_free_pool_a(e, i); // 0x5C350
                }
                self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
            }
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x5C350: jmp 0x48FC
            }
        }
    }
}
