//! Pool A class 90 (`+$20 = 0x168`, handler 0x650FA): a cloud of the explosions of the ending (M3-23). The master of the
//! ending (pool B 82, `prop/b82_85.rs`) makes four of them in the lab corridor scene, the variants 0, 0x100, 0x200 and
//! 0x300; the high byte of the variant (`+$26`) is the animation and the movement. They run for 460 frames next to the
//! two heroes (the camera moves 4 pixels a frame at first), drawn with the animation table 0x6531C (8 animations).
//!
//! | mode | what |
//! | --- | --- |
//! | 0 | 0x6510E: live 2, sub 0, census mark `+$7D = 0xFF`, `+$63 = 0xFF`, no shadow, the table, `+$AC` = the X word, X += camera X, Y 0, the speed `+$A8` (1.0, 1.125 for the variant 0), the animation = the high byte of the variant, direction 8, facing 1, mode 2 and the frame of mode 2 at once |
//! | 2 | 0x6517E: the sub state routine, then the draw check 0x49CA |
//! | 4, 6 | 0x65316: free (0x48FC) |
//!
//! Sub states of mode 2: 0 (0x65198) the timer `+$B2` = 0x1CC and on into sub 2; sub 2 (0x651A4) one of three scripts by the
//! high byte of the variant (2: 0x651F4, 3: 0x65278, else the ordinary one 0x651B8); sub 4 (0x65308) mode 4. Every script moves
//! the object with `+$AC += +$A8` (a 16.16 long) and sets X to the camera X (G.$744, a long) plus `+$AC`, then steps the animation (0x121E).

use super::Enemy;
use crate::game::{Game, Handle};

/// The animation table of the class (8 animations).
const TABLE: u32 = 0x6531C;
/// `+$AC` the X offset (long, 16.16), `+$A8` its speed (long), `+$B2` the frame timer (word).
const OFF_X: usize = 0xAC;
const SPEED: usize = 0xA8;
const TIMER: usize = 0xB2;

impl Game {
    /// 0x650FA class 90 (jump table 0x65106: mode 0 -> 0x6510E, 2 -> 0x6517E, 4 and 6 -> 0x65316).
    pub fn enemy_class90(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                self.c90_init(e);
                self.c90_frame(e, h); // 0x6517A: addq.b #2,$4(a6), then the code of mode 2
            }
            2 => self.c90_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x65316: jmp 0x48FC
            }
        }
    }

    /// 0x6510E: the init.
    fn c90_init(&mut self, e: &mut Enemy) {
        e.b.live = 2;
        e.b.sub = 0;
        e.m.sb(super::off::ATTACKING, 0xFF); // +$7D: not counted by the census
        e.b.b63 = 0xFF;
        e.b.draw_flags = 0x10;
        e.b.anim_table = TABLE;
        e.m.sw(OFF_X, e.b.xi() as u16); // move.w $8(a6),$AC(a6): the high word of the long
        e.b.set_xi(e.b.xi().wrapping_add(self.cam.s2.cam_x as i16)); // add.w $744(a5),$8(a6)
        e.b.set_yi(0);
        e.m.sl(SPEED, 0x10000);
        if (e.b.var26 >> 8) == 0 {
            e.m.sl(SPEED, 0x12000);
        }
        let a = e.b.var26 >> 8; // d0 = the byte +$26
        self.enemy_anim(&mut e.b, a);
        e.b.dir = 8;
        e.b.facing = 1;
        e.b.mode += 2;
    }

    /// 0x6517E: the sub state, then the draw check (0x49CA).
    fn c90_frame(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => {
                e.m.sw(TIMER, 0x1CC); // 0x65198
                e.b.sub += 2;
                self.c90_run(e);
            }
            2 => self.c90_run(e),
            4 => {
                e.b.mode += 2; // 0x65308: the timer word is set too, mode 4 frees it in the next frame
                e.m.sw(TIMER, 0x3C);
            }
            _ => {}
        }
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x651A4: the script of the variant (the high byte of +$26: 2 = 0x651F4, 3 = 0x65278, else 0x651B8).
    fn c90_run(&mut self, e: &mut Enemy) {
        let t = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, t);
        if t == 0 {
            e.b.sub = 4; // 0x651EC / 0x65270 / 0x65300
            return;
        }
        match e.b.var26 >> 8 {
            2 => {
                if t == 0xBE {
                    e.m.sl(SPEED, 0x10000);
                    self.enemy_anim(&mut e.b, 4);
                }
                if t == 0xAA {
                    e.m.sl(SPEED, 0xFFFE_C000);
                    e.b.facing = 0;
                    self.enemy_anim(&mut e.b, 2);
                }
                if t == 0x96 {
                    e.m.sl(SPEED, 0);
                    e.b.facing = 0;
                    self.enemy_anim(&mut e.b, 5);
                }
            }
            3 => {
                if t == 0xD2 {
                    e.m.sl(SPEED, 0x4000);
                    self.enemy_anim(&mut e.b, 6);
                }
                if t == 0xCD {
                    e.m.sl(SPEED, 0);
                    self.enemy_anim(&mut e.b, 6);
                    self.snd_request(0xBB, 0x10, 0); // 0x652B4: move.w #$BB,d0; move.l #$10,d1; moveq #0,d2
                }
                if t == 0x96 {
                    e.m.sl(SPEED, 0);
                    e.b.facing = 0;
                    self.enemy_anim(&mut e.b, 7);
                }
            }
            _ => {
                if t == 0xD2 {
                    e.m.sl(SPEED, e.m.l(SPEED).wrapping_add(0x10000)); // 0x651CA
                }
            }
        }
        // 0x651D2 / 0x65256 / 0x652E6: +$AC += +$A8; X = the camera X (long, with its fraction) + +$AC
        let off = e.m.l(OFF_X).wrapping_add(e.m.l(SPEED));
        e.m.sl(OFF_X, off);
        let cam = ((self.cam.s2.cam_x as u32) << 16) | self.cam.s2.cam_x_frac as u32; // move.l $744(a5),d0
        e.b.x = cam.wrapping_add(off) as i32;
        self.enemy_step(&mut e.b); // jmp 0x121E: the carry is not used
    }
}
