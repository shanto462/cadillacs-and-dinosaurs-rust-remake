//! Pool A class 92 (`+$20 = 0x170`, handler 0x676FE): the pieces of the explosions and showers of the ending (M3-23).
//! The debris showers (pool B 83 and 85) make them; the variant (`+$26`) is the kind: 0 (a piece that flies up and out,
//! drawn with a shadow), 0x100 (high, no shadow), 0x500 (the quake pieces of B85, fast), 0x600 and 0x700 (the pieces of
//! the shower of the last explosion, B83), 0x800 (straight to the animation). The animation table is 0x678A0.
//!
//! | mode | what |
//! | --- | --- |
//! | 0 | 0x67712: `+$63 = 0xFF`, the table, draw flags 0, the speed `+$A0` (a 16.16 long, set from `rand()`), the position and the animation by the variant, then the frame of mode 2 at once (0x67864: `+$A4 = 0`, mode 2) |
//! | 2 | 0x6786E: one animation step; at its end mode 4 and free (0x6789A); else X += `+$A0` and the object is drawn on every second frame only (`+$A4` counts) |
//! | 4, 6 | 0x6789A: free (0x48FC) |

use super::Enemy;
use crate::game::{Game, Handle};

const TABLE: u32 = 0x678A0;
/// `+$A0` the X speed (a long, 16.16), `+$A4` the frame counter (word).
const SPEED: usize = 0xA0;
const COUNT: usize = 0xA4;

impl Game {
    /// 0x676FE class 92 (jump table 0x6770A: mode 0 -> 0x67712, 2 -> 0x6786E, 4 and 6 -> 0x6789A).
    pub fn enemy_class92(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                if self.c92_init(e) {
                    return self.enemy_class92_free(e, h); // 0x6789A
                }
                e.m.sw(COUNT, 0); // 0x67864
                e.b.mode += 2;
                self.c92_frame(e, h);
            }
            2 => self.c92_frame(e, h),
            _ => self.enemy_class92_free(e, h),
        }
    }

    fn enemy_class92_free(&mut self, e: &mut Enemy, h: Handle) {
        let i = self.enemy_index(h);
        self.enemy_free_pool_a(e, i); // jmp 0x48FC
    }

    /// 0x67712. True when the object ends at once (the variant 0x700 draws a 1 in 16 chance, 0x67828).
    fn c92_init(&mut self, e: &mut Enemy) -> bool {
        e.b.b63 = 0xFF;
        e.b.anim_table = TABLE;
        e.b.draw_flags = 0;
        e.m.sw(SPEED, 0);
        self.enemy_anim(&mut e.b, 6);
        let var = e.b.var26;
        if var == 0x800 {
            return false; // beq 0x67864
        }
        if var != 0x500 {
            e.b.set_xi(self.cam.s2.cam_x as i16); // move.w $744(a5),$8(a6)
        }
        e.m.sl(SPEED, 0x8000);
        let r = self.rand() & 3;
        e.m.sw(SPEED, e.m.w(SPEED).wrapping_add(r)); // add.w d0,$A0(a6): the high word of the long
        e.m.sl(SPEED, e.m.l(SPEED).wrapping_add(0x30000));
        if var == 0x500 {
            e.m.sl(SPEED, 0xC0000);
            let r = self.rand() & 7;
            e.m.sw(SPEED, e.m.w(SPEED).wrapping_add(r));
            self.enemy_anim(&mut e.b, 6);
        } else {
            e.b.set_yi(0);
            e.b.set_zi(0);
            let r = (self.rand() & 0xF0) as i16;
            e.b.set_yi(e.b.yi().wrapping_add(r));
            let r = (self.rand() & 0x30) as i16;
            e.b.set_zi(e.b.zi().wrapping_add(r));
            let a = self.rand() & 3;
            self.enemy_anim(&mut e.b, a);
            if var != 0 {
                e.m.sw(SPEED, 0); // 0x677DC: the high word only
                e.b.set_zi(0x80);
                e.b.draw_flags = 0x10;
                let r = (self.rand() & 0x1F0) as i16;
                e.b.set_xi(e.b.xi().wrapping_add(r));
            }
        }
        if var == 0x600 {
            self.enemy_anim(&mut e.b, 2); // 0x677FC
        }
        if var == 0x700 {
            if self.rand() & 0xF == 0 {
                return true; // bra 0x6789A: free
            }
            self.enemy_anim(&mut e.b, 5);
            e.b.set_yi(0);
            e.b.set_zi(self.cam.s2.cam_z as i16); // move.w $74C(a5),$10(a6)
            let r = (self.rand() & 0x70) as i16;
            e.b.set_zi(e.b.zi().wrapping_add(r));
            e.b.set_xi(self.cam.s2.cam_x as i16);
            let r = (self.rand() & 0x1F0) as i16;
            e.b.set_xi(e.b.xi().wrapping_add(r));
        }
        false
    }

    /// 0x6786E: the animation step; at its end the object is freed, else it moves and is drawn on every second frame.
    fn c92_frame(&mut self, e: &mut Enemy, h: Handle) {
        if self.enemy_step(&mut e.b) {
            e.b.mode += 2; // 0x67896
            return self.enemy_class92_free(e, h);
        }
        e.b.x = e.b.x.wrapping_add(e.m.l(SPEED) as i32);
        let n = e.m.w(COUNT).wrapping_add(1);
        e.m.sw(COUNT, n);
        if n & 1 == 0 {
            self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
        }
    }
}
