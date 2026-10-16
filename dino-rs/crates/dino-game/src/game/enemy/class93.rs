//! Pool A class 93 (`+$20 = 0x174`, handler 0x67B32): the falling pieces of the explosions of the ending (M3-23). The shower
//! of B84 makes them (variant 0); a piece of the variant 0 throws smaller ones (variant 0x100) on its way down. They rise from
//! the height 0xE0 downward with a speed in `+$A0`, with the palette override 0x88 and the animation table 0x67C3A.
//!
//! | mode | what |
//! | --- | --- |
//! | 0 | 0x67B46: the table, draw flags 0x10, `+$63 = 0xFF`, palette 0x88; the variant 0x100 has the speed 0x4000 and the animation 8, the others take X = camera X + `rand() & 0x1F0`, Y 0xE0, Z 0, the speed 0x28000 + `rand() & 3` (high word) and the animation `rand() & 7`; then mode 2 and its frame at once |
//! | 2 | 0x67BC6: ends (mode 4 and free) when Y is negative; else Y -= `+$A0`, one animation step (a piece of the variant 0 may make a small one: `rand() & 0x1F` = 0), draw check |
//! | 4, 6 | 0x67C34: free (0x48FC) |

use super::Enemy;
use crate::game::{Game, Handle};

const TABLE: u32 = 0x67C3A;
/// `+$A0` the falling speed (a long, 16.16).
const SPEED: usize = 0xA0;

impl Game {
    /// 0x67B32 class 93 (jump table 0x67B3E: mode 0 -> 0x67B46, 2 -> 0x67BC6, 4 and 6 -> 0x67C34).
    pub fn enemy_class93(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                self.c93_init(e);
                self.c93_frame(e, h);
            }
            2 => self.c93_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // jmp 0x48FC
            }
        }
    }

    /// 0x67B46.
    fn c93_init(&mut self, e: &mut Enemy) {
        e.b.anim_table = TABLE;
        e.b.draw_flags = 0x10;
        e.b.b63 = 0xFF;
        e.b.palette_override = 0x88;
        if (e.b.var26 >> 8) != 0 {
            e.m.sl(SPEED, 0x4000);
            self.enemy_anim(&mut e.b, 8);
        } else {
            e.b.set_xi(self.cam.s2.cam_x as i16);
            e.m.sl(SPEED, 0x28000);
            let r = self.rand() & 3;
            e.m.sw(SPEED, e.m.w(SPEED).wrapping_add(r));
            e.b.set_yi(0xE0);
            e.b.set_zi(0);
            let r = (self.rand() & 0x1F0) as i16;
            e.b.set_xi(e.b.xi().wrapping_add(r));
            let a = self.rand() & 7;
            self.enemy_anim(&mut e.b, a);
        }
        e.b.mode += 2; // 0x67BC2
    }

    /// 0x67BC6.
    fn c93_frame(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.yi() < 0 {
            return self.c93_end(e, h); // bmi 0x67C30
        }
        e.b.y = e.b.y.wrapping_sub(e.m.l(SPEED) as i32);
        if (e.b.var26 >> 8) != 0 {
            // 0x67C22
            if self.enemy_step(&mut e.b) {
                return self.c93_end(e, h);
            }
            return self.enemy_draw_check(&mut e.b, h);
        }
        self.enemy_step(&mut e.b); // the carry is not used
        if self.rand() & 0x1F == 0 {
            if let Some(j) = self.enemies.alloc() {
                let (x, y, z) = (e.b.xi(), e.b.yi(), e.b.zi());
                let n = &mut self.enemies.slots[j];
                n.b.live = 1;
                n.b.k20 = 0x174;
                n.b.var26 = 0x0100; // move.b #1,$26(a0)
                n.b.set_xi(x);
                n.b.set_yi(y);
                n.b.set_zi(z);
            }
        }
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x67C30: mode 4, then 0x67C34 frees it.
    fn c93_end(&mut self, e: &mut Enemy, h: Handle) {
        e.b.mode += 2;
        let i = self.enemy_index(h);
        self.enemy_free_pool_a(e, i);
    }
}
