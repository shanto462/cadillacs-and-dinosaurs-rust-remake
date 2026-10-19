//! Pool A class 12 (`+$20 = 0x30`, handler 0x3DC5C): a golden raptor of the opening movie's jungle (M1-10). The movie
//! handlers of the counters 900 (two, variant `+$26` high byte 1: they run) and 930 (three, variant 0: they stand
//! about) create them to the right of the camera.
//!
//! Mode 0 (0x3DC70) sets up the object (live 2, the table 0x3DD50, no census, `+$63` negative, facing 1, `+$A6` = a random
//! number 0 to 3, depth + 0x10, Y = -0x20, the terrain, the animation 2, or 1 for the variant 1, `+$A0` = 0x104) and goes
//! on in mode 2 (0x3DCD2) in the same frame: Y = 0x18, `+$A0` counts down (at 0 the object is freed), then
//!
//! * variant 0: the animation 0 when `+$A0` is 0xE6; from there on X + 10 a frame;
//! * variant 1: X + 4 a frame and the sound 0x73 every 8 frames;
//!
//! and in both X + `+$A6` every 4th frame, the animation step 0x121E and the draw list 0x49CA. Mode 4 and 6 free it.

use super::lib32::put_hi;
use super::Enemy;
use crate::game::{Game, Handle};

/// Offsets of this class.
mod f {
    /// word: the frames to live.
    pub const TIMER: usize = 0xA0;
    /// word: the extra pixels every 4th frame (0 to 3).
    pub const DRIFT: usize = 0xA6;
}

const TABLE: u32 = 0x3DD50;

impl Game {
    /// 0x3DC5C class 12 (jump table 0x3DC68: mode 0 -> 0x3DC70, 2 -> 0x3DCD2, 4 and 6 -> 0x3DD4A).
    pub fn enemy_class12(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                e.b.live = 2;
                e.b.anim_table = TABLE;
                e.m.sb(super::off::ATTACKING, 0xFF);
                e.b.b63 = 0xFF;
                e.b.facing = 1;
                let r = self.rand() & 3;
                e.m.sw(f::DRIFT, r);
                let z = e.b.zi().wrapping_add(0x10);
                put_hi(&mut e.b.z, z);
                put_hi(&mut e.b.y, -0x20);
                if let Some(o) = self.terrain_collide(&mut e.b, h, false) {
                    e.m.sw(0x94, o); // 0x12CB4 leaves the obstacle in +$94
                }
                self.enemy_anim(&mut e.b, 2);
                if e.b.var26 >> 8 != 0 {
                    self.enemy_anim(&mut e.b, 1);
                }
                e.m.sw(f::TIMER, 0x104);
                e.b.mode += 2;
                self.enemy_c12_frame(e, h);
            }
            2 => self.enemy_c12_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x3DD4A: jmp 0x48FC
            }
        }
    }

    /// 0x3DCD2.
    fn enemy_c12_frame(&mut self, e: &mut Enemy, h: Handle) {
        put_hi(&mut e.b.y, 0x18);
        let t = e.m.w(f::TIMER).wrapping_sub(1);
        e.m.sw(f::TIMER, t);
        if t == 0 {
            let i = self.enemy_index(h);
            return self.enemy_free_pool_a(e, i); // 0x3DD4A
        }
        let add_x = |e: &mut Enemy, d: u16| {
            let w = e.b.xi().wrapping_add(d as i16);
            put_hi(&mut e.b.x, w);
        };
        if e.b.var26 >> 8 == 0 {
            if t == 0xE6 {
                self.enemy_anim(&mut e.b, 0);
            }
            if (t as i16) <= 0xE6 {
                add_x(e, 0x0A);
            }
        } else {
            add_x(e, 4);
            if t & 7 == 0 {
                self.sound(0x73);
            }
        }
        if t & 3 == 0 {
            let d = e.m.w(f::DRIFT);
            add_x(e, d);
        }
        self.enemy_step(&mut e.b); // jsr 0x121E
        self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
    }
}
