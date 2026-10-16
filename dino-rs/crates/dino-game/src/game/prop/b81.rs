//! Pool B class 81 (`+$20 = 0x144`, handler 0x8A5FA): the master of the desert scene of the ending (M3-25 / M3-26). It is made by the master 82 at the
//! end of the still picture (record 969) and lives about 4000 frames: the desert (scroll2 page 0x16, scroll3 page 0x6C), the two walkers (pool A 81 variants
//! 1 and 2) and the staff roll (pool B 87), the Cadillac (pool A 81 variant 0) with the music 0x1A, the talk of the heroes (the portraits and texts of the
//! ending with 0x84736) and at the end THANK YOU FOR PLAYING, PRODUCED BY and the CAPCOM sign.
//!
//! Mode 0 (0x8A60E): the music stops, the music 0x19 starts, mode 2, scroll3 X + 0x180, the pages, the walkers (`+$A4`, `+$A8` their handles),
//! `+$AC` = 1 (the camera speed), `+$A0` = 0xBA4 and `+$AE` = 0x1036. Mode 2 (0x8A726): while `+$AE` has not run out (4150 frames) and the sub state
//! is below 0x32 the camera X goes back `+$AC` pixels and scroll3 X 1/16 pixel a frame; then the state of the table 0x8A758:
//!
//! | sub | what |
//! | --- | --- |
//! | 0, 4, 8, 0xC, 0x10, 0x14, 0x18, 0x1C, 0x20, 0x24, 0x28, 0x2E, 0x32 | wait `+$A0` frames (0x8A78E) |
//! | 2 | 0x8A7A0: the music 0x1A, the car (pool A 81 variant 0), `+$A0` = 0x14 |
//! | 6, 0xE, 0x16, 0x1E, 0x26 | 0x8A81C ... 0x8A88C: a portrait and its text (Mustapha 0x35, Jack 0x36, 0x39, 0x3A, Hannah 0x38) for 0x78 / 0x78 / 0xB4 / 0xB4 / 0x78 frames |
//! | 0xA, 0x12, 0x1A, 0x22, 0x2A | 0x8A8CA: the text block is blanked, `+$A0` = 0x1E (the last one 1) |
//! | 0x2C | 0x8A8A8: the camera speed 2, `+$A0` = 0x64 |
//! | 0x30 | 0x8A8F0: THANK YOU FOR PLAYING and PRODUCED BY (the big font, 0xCE4), the CAPCOM sign (0x110C at 0x11, 0x1A), `+$A0` = 0x3C |
//! | 0x34 | 0x8A7D4: G.$507 = 0 (the name entry may start) and the end of the object |

use super::Prop;
use crate::game::Game;

mod f {
    pub const TIMER: usize = 0xA0;
    pub const SPEED: usize = 0xAC;
    pub const RUN: usize = 0xAE;
}

/// The text block of the portraits (the cell the portrait is drawn at) and the two end texts.
const TEXT_AT: u32 = 0x90A29C;
const END_TEXTS: [u32; 2] = [0x8A956, 0x8A972];

impl Game {
    /// 0x8A5FA pool B class 81 (jump table 0x8A606: mode 0 -> 0x8A60E, 2 -> 0x8A726, 4 and 6 -> 0x8A740).
    pub fn prop_b81(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                self.snd_request(0xFF00, 0, 0); // jsr 0xA5D2: the sound stop (d1 and d2 stay 0)
                self.snd_request(0x19, 0, 0); // move.w #$19,d0; jsr 0xA63C
                p.b.mode = 2;
                p.b.sub = 0;
                self.cam.s3.cam_x = self.cam.s3.cam_x.wrapping_add(0x180);
                for a in [0x90F000, 0x90F400, 0x90F800, 0x90FC00] {
                    self.flow_page2(a, 0x16);
                }
                for a in [0x913800, 0x913900, 0x913A00, 0x913B00, 0x913C00, 0x913D00] {
                    self.flow_page3(a, 0x6C);
                }
                for (v, off) in [(1u16, 0xA4usize), (2, 0xA8)] {
                    if let Some(j) = self.enemies.alloc() {
                        let e = &mut self.enemies.slots[j];
                        e.b.live = 1;
                        e.b.k20 = 0x144;
                        e.b.var26 = v;
                        let h = self.enemies.handle(j);
                        p.m.sl(off, h as i16 as i32 as u32); // move.l a0,$A4(a6): the handle sign extended
                    }
                }
                p.m.sw(f::SPEED, 1);
                p.m.sw(f::TIMER, 0xBA4);
                p.m.sw(f::RUN, 0xDE8);
                p.m.sw(f::RUN, p.m.w(f::RUN).wrapping_add(0x122));
                p.m.sw(f::RUN, p.m.w(f::RUN).wrapping_add(0x12C));
                self.b81_frame(p, i);
            }
            2 => self.b81_frame(p, i),
            _ => self.prop_free(p, i), // 0x8A740
        }
    }

    /// 0x8A726.
    fn b81_frame(&mut self, p: &mut Prop, i: usize) {
        let run = p.m.w(f::RUN).wrapping_sub(1);
        p.m.sw(f::RUN, run);
        if (run as i16) >= 0 && (p.b.sub as i8) < 0x32 {
            self.cam.s2.cam_x = self.cam.s2.cam_x.wrapping_sub(p.m.w(f::SPEED)); // sub.w d0,$744(a5)
            self.cam.s3.cam_x_long_sub(0x1000); // subi.l #$1000,$7E4(a5)
        }
        match p.b.sub {
            0x00 | 0x04 | 0x08 | 0x0C | 0x10 | 0x14 | 0x18 | 0x1C | 0x20 | 0x24 | 0x28 | 0x2E | 0x32 => {
                // 0x8A78E: wait `+$A0` frames
                let t = p.m.w(f::TIMER).wrapping_sub(1);
                p.m.sw(f::TIMER, t);
                if t == 0 {
                    p.b.sub += 2; // 0x8A798
                }
            }
            0x02 => {
                // 0x8A7A0: the music 0x1A (d1 = the offset 0x48 of the jump table entry, d2 as the register held it), the car
                self.snd_request(0x1A, 0x48, 0x1388);
                let _ = self.enemy_spawn_var(0x144, 0);
                p.b.sub += 2;
                p.m.sw(f::TIMER, 0x14);
            }
            0x06 => self.b81_talk(p, 0xC, 0x35, 0x78),
            0x0E => self.b81_talk(p, 0xA, 0x36, 0x78),
            0x16 => self.b81_talk(p, 0xA, 0x39, 0xB4),
            0x1E => self.b81_talk(p, 0xA, 0x3A, 0xB4),
            0x26 => self.b81_talk(p, 0xB, 0x38, 0x78),
            0x0A | 0x12 | 0x1A | 0x22 => {
                // 0x8A8CA: blank the text block
                self.b81_blank();
                p.b.sub += 2;
                p.m.sw(f::TIMER, 0x1E);
            }
            0x2A => {
                self.b81_blank(); // 0x8A91E
                p.b.sub += 2;
                p.m.sw(f::TIMER, 1);
            }
            0x2C => {
                p.m.sw(f::SPEED, 2); // 0x8A8A8
                p.b.sub += 2;
                p.m.sw(f::TIMER, 0x64);
            }
            0x30 => {
                p.b.sub += 2; // 0x8A8F0
                p.m.sw(f::TIMER, 0x3C);
                for a in END_TEXTS {
                    let s = self.assets.frontend.text.slice_from(a);
                    self.text_big(s, None, true); // jsr 0xCE4
                }
                self.flow_capcom_sign(0x11, 0x1A); // moveq #$11,d0; moveq #$1A,d1; jmp 0x110C
            }
            0x34 => {
                self.flow.b507 = 0; // 0x8A7D4
                self.prop_free(p, i);
            }
            _ => {}
        }
    }

    /// 0x8A81C and its kin: the state goes on, the timer is `time`, the portrait `portrait` and the text `text` are drawn at the text block.
    fn b81_talk(&mut self, p: &mut Prop, portrait: u16, text: i16, time: u16) {
        p.b.sub += 2;
        p.m.sw(f::TIMER, time);
        let pal = self.assets.frontend.text.u16(0x8461A, 2 * portrait as u32);
        p.m.sw(0xA2, pal); // 0x84736 keeps the palette word of the portrait in +$A2 of the caller
        self.text_portrait(TEXT_AT, portrait, text);
    }

    /// 0x8A934: 45 columns of 13 cells from the text block are set to the space (0x20, attribute 0).
    fn b81_blank(&mut self) {
        for col in 0..45u32 {
            for row in 0..13u32 {
                self.hud_put(TEXT_AT + col * 0x80 + row * 4, 0x20, 0);
            }
        }
    }

    /// A pool A object (0x483C) with the class word `k20 / 4 * 4` and the variant; None when the pool is full.
    pub fn enemy_spawn_var(&mut self, k20: u16, var: u16) -> Option<usize> {
        let j = self.enemies.alloc()?;
        let e = &mut self.enemies.slots[j];
        e.b.live = 1;
        e.b.k20 = k20;
        e.b.var26 = var;
        Some(j)
    }
}
