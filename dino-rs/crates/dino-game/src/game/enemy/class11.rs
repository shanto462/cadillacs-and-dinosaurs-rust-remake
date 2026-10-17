//! Pool A class 11 (`+$20 = 0x2C`, handler 0x3D960): the two hunters of the opening movie's jungle (M1-10).
//! The movie handler of the counter 600 (0x9C340, 0x9C380) creates two of them; the byte `+$26` (the high byte of
//! the variant word) is 0 for the first and 1 for the second, who use the animation tables 0x3DAF8 and 0x3DB00.
//!
//! Mode 0 (0x3D974) sets up the object (live 2, the table, no census, facing 1, Y = -0x20, the terrain, the
//! animation 1, `+$A0` = 0x14) and runs the first frame at once (0x3D9C2, which sets Y = 0x20 each frame). The
//! frame has three sub states by `+$05`:
//!
//! | sub | frames | what |
//! | --- | --- | --- |
//! | 0 | `+$A0` = 0x14 | X + 2 each frame; at the end `+$A0` = 0x8C, animation 0, the caption banner 0x12 (0x86D2E) |
//! | 2 | `+$A0` = 0x8C | X stays; at `+$A0` = 0x4A the flock of birds (pool B class 72, variant 0x600) and the sound 0x61, at 0x46 the sound 0xEA (0x9C76), a puff (pool A class 57) in every frame the animation frame has the flag `+$2D`; at the end `+$A0` = 0xC8, animation 2, the caption banner 0x13 |
//! | 4 | `+$A0` = 0xC8 | X + 2 (+ 3 for the first hunter), at 0 the object is freed (0x3DAF2: jmp 0x48FC) |
//!
//! Mode 4 and 6 free it too.

use super::lib32::put_hi;
use super::Enemy;
use crate::game::{Game, Handle};

/// Offsets of this class.
mod f {
    /// word: the frames of the sub state.
    pub const TIMER: usize = 0xA0;
}

/// The animation tables of the two hunters (the first, the second: `+$26` high byte).
const TABLE_A: u32 = 0x3DAF8;
const TABLE_B: u32 = 0x3DB00;

impl Game {
    /// 0x3D960 class 11 (jump table 0x3D96C: mode 0 -> 0x3D974, 2 -> 0x3D9C2, 4 and 6 -> 0x3DAF2).
    pub fn enemy_class11(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                self.enemy_c11_init(e, h);
                self.enemy_c11_frame(e, h); // 0x3D9BE: addq.b #2,4(a6), then 0x3D9C2
            }
            2 => self.enemy_c11_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x3DAF2: jmp 0x48FC
            }
        }
    }

    /// 0x3D974..0x3D9BE.
    fn enemy_c11_init(&mut self, e: &mut Enemy, h: Handle) {
        e.b.live = 2;
        e.b.anim_table = if e.b.var26 >> 8 != 0 { TABLE_B } else { TABLE_A };
        e.m.sb(super::off::ATTACKING, 0xFF); // +$7D
        e.b.b63 = 0xFF;
        e.b.facing = 1;
        put_hi(&mut e.b.y, -0x20); // move.w #$FFE0,$C(a6)
        let _ = self.terrain_collide(&mut e.b, h, false); // jsr 0x12CB4
        self.enemy_anim(&mut e.b, 1);
        e.m.sw(f::TIMER, 0x14);
        e.b.mode += 2;
    }

    /// 0x3D9C2: Y = 0x20, then the sub state.
    fn enemy_c11_frame(&mut self, e: &mut Enemy, h: Handle) {
        put_hi(&mut e.b.y, 0x20);
        let second = e.b.var26 >> 8 != 0; // tst.b $26(a6)
        let t = e.m.w(f::TIMER).wrapping_sub(1);
        e.m.sw(f::TIMER, t);
        match e.b.sub {
            0 => {
                if t == 0 {
                    // 0x3D9E4
                    e.m.sw(f::TIMER, 0x8C);
                    e.b.sub += 2;
                    self.enemy_anim(&mut e.b, 0);
                    self.prop_caption_spawn(0x12); // jsr 0x86D2E
                }
                self.enemy_c11_walk(e, h, 2);
            }
            2 => {
                if t == 0 {
                    // 0x3DA16
                    e.m.sw(f::TIMER, 0xC8);
                    e.b.sub += 2;
                    self.enemy_anim(&mut e.b, 2);
                    self.prop_caption_spawn(0x13);
                }
                let t = e.m.w(f::TIMER);
                if t == 0x46 {
                    // jsr 0x9C76: d1 = (mode << 16) | 0x2000 | pan (mode 3, or 4 when +$24 (facing) is set), d2 = 0x15000000;
                    // the pool tag +$82 is 4, so the code is the fixed 0xEA
                    let mode: u32 = if e.b.facing == 0 { 3 } else { 4 };
                    let pan = self.snd_pan_x(e.b.xi());
                    self.snd_request(0xEA, (mode << 16) | 0x2000 | pan as u32, 0x1500_0000);
                }
                if e.b.param_b != 0 {
                    self.enemy_c11_puff(e); // 0x3DA48
                }
                if t == 0x4A {
                    // 0x3DA96: pool B class 72 (the flock of birds), variant 0x600, the sound 0x61
                    self.prop_flock_spawn();
                    self.sound(0x61);
                }
                self.enemy_step(&mut e.b); // 0x3DAC2: jsr 0x121E
                self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
            }
            _ => {
                if t == 0 {
                    let i = self.enemy_index(h);
                    return self.enemy_free_pool_a(e, i); // 0x3DAF2
                }
                if !second {
                    self.enemy_c11_add_x(e, 1); // 0x3DADE: addq.w #1,$8(a6)
                }
                self.enemy_c11_walk(e, h, 2);
            }
        }
    }

    /// `addq.w #n,$8(a6)`, the animation step 0x121E and the draw list 0x49CA (0x3D9FE, 0x3DAE2).
    fn enemy_c11_walk(&mut self, e: &mut Enemy, h: Handle, dx: i16) {
        self.enemy_c11_add_x(e, dx);
        self.enemy_step(&mut e.b);
        self.enemy_draw_check(&mut e.b, h);
    }

    fn enemy_c11_add_x(&mut self, e: &mut Enemy, dx: i16) {
        let w = e.b.xi().wrapping_add(dx);
        put_hi(&mut e.b.x, w);
    }

    /// 0x3DA48: a puff (pool A class 57, variant 1) at the hunter's position + 0x2B / 0x44, facing like him.
    fn enemy_c11_puff(&mut self, e: &Enemy) {
        if let Some(i) = self.enemies.alloc() {
            let b = &mut self.enemies.slots[i].b;
            b.live = 1;
            b.k20 = 0xE4;
            b.var26 = 1;
            b.set_xi(e.b.xi().wrapping_add(0x2B));
            b.set_yi(e.b.yi().wrapping_add(0x44));
            b.set_zi(e.b.zi());
            b.facing = e.b.facing;
        }
    }
}
