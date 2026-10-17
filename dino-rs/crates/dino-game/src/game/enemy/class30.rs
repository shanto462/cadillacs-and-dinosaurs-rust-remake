//! Pool A class 30 (`+$20 = 0x78`, handler 0x4FB10): the big brown dinosaur that walks into the opening movie's jungle
//! (M1-11; the movie handler of the counter 1100 creates it, 0x9C426). It stands a little in front of the hunters
//! (depth 0x45, Y = -0x20), shakes the ground three times (a screen shake request, `G.$755C` = 4, with the sound 0x75
//! at its frames 0, 0x40 and 0x80), walks right for 0xA0 frames (1 pixel a frame while its counter `+$84` is below 0xAE),
//! roars (the sound 0x36 at frame 0xC2), changes its palette override from 0x91 to 0x88 at 0xFA and is gone at 0x160.
//!
//! Mode 0 (0x4FB24) sets up the object (live 2, the table 0x4FC22, override 0x91, draw flags 0x20, `+$63` negative, `+$80`
//! = 0, `+$82` = 0x64, the terrain, Y = -0x20, depth 0x45, facing 1, `+$88` = the scroll2 camera, the animation 0, `+$86`
//! = 0xA0, `+$84` = 0) and goes on in mode 2 (0x4FB84) in the same frame; mode 4 and 6 free it (0x4FC1C).
//!
//! The end test of the original compares a register that holds the counter before its increment while `+$86` is 0
//! (0x4FBC8..0x4FBFE): the object is freed one frame later then.

use super::lib32::put_hi;
use super::Enemy;
use crate::game::{Game, Handle};

/// Offsets of this class.
mod f {
    /// word: the frame counter.
    pub const COUNT: usize = 0x84;
    /// word: frames the walk lasts.
    pub const WALK: usize = 0x86;
    /// word: the scroll2 camera X at the start.
    pub const CAM: usize = 0x88;
}

const TABLE: u32 = 0x4FC22;

impl Game {
    /// 0x4FB10 class 30 (jump table 0x4FB1C: mode 0 -> 0x4FB24, 2 -> 0x4FB84, 4 and 6 -> 0x4FC1C).
    pub fn enemy_class30(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                e.b.live = 2;
                e.b.anim_table = TABLE;
                e.b.palette_override = 0x91;
                e.b.draw_flags = 0x20;
                e.b.b63 = 0xFF;
                e.m.sw(0x80, 0);
                e.m.sw(0x82, 0x64);
                let _ = self.terrain_collide(&mut e.b, h, false);
                put_hi(&mut e.b.y, -0x20);
                put_hi(&mut e.b.z, 0x45);
                e.b.facing = 1;
                e.m.sw(f::CAM, self.cam.s2.cam_x);
                self.enemy_anim(&mut e.b, 0);
                e.m.sw(f::WALK, 0xA0);
                e.m.sw(f::COUNT, 0);
                e.b.mode += 2;
                self.enemy_c30_frame(e, h);
            }
            2 => self.enemy_c30_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x4FC1C: jmp 0x48FC
            }
        }
    }

    /// 0x4FB84.
    fn enemy_c30_frame(&mut self, e: &mut Enemy, h: Handle) {
        let mut d0 = e.m.w(f::COUNT) as i16;
        if d0 < 0xBE && d0 & 0x3F == 0 {
            // jsr 0xA31C: d1 = 0x71000 with the pan byte of the object's X (table 0xA66A), d2 = 0x3000200
            let pan = self.snd_pan_x(e.b.xi());
            self.snd_request(0x75, 0x71000 | pan as u32, 0x0300_0200);
            self.cam_shake(4); // move.b #4,$755C(a5)
        }
        d0 = e.m.w(f::COUNT) as i16;
        if d0 == 0xC2 {
            self.sound(0x36); // jsr 0xA63C
        }
        let n = e.m.w(f::COUNT).wrapping_add(1);
        e.m.sw(f::COUNT, n);
        if e.m.w(f::WALK) != 0 {
            e.m.sw(f::WALK, e.m.w(f::WALK).wrapping_sub(1));
            d0 = n as i16;
            if d0 < 0xAE {
                let x = e.b.xi().wrapping_add(1);
                put_hi(&mut e.b.x, x);
            }
        }
        if n == 0xFA {
            e.b.palette_override = 0x88;
        }
        if d0 == 0x160 {
            let i = self.enemy_index(h);
            return self.enemy_free_pool_a(e, i); // 0x4FC1C
        }
        if (n as i16) <= 0xFA {
            self.enemy_step(&mut e.b); // jsr 0x121E
        }
        self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
    }
}
