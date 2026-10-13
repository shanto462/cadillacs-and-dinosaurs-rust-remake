//! Pool A class 46 (`+$20 = 0xB8`, handler 0x5656C): a bird of the flock in the opening movie's jungle (M1-10); the
//! spawner is pool B class 72 (`prop/class72.rs`). It sits where it is made (Y and depth are 0: the jungle's birds fly
//! from above) until `+$5C` frames have passed (0 here), then takes off: a horizontal speed of 4 to 7 pixels a frame (a
//! 16.16 number at `+$B0`; the variant 0x200 goes the other way), a rise of 1 or 2 pixels a frame (`+$AA`), the
//! sound 0x61 (0xA63C with d1 = 0x5011F, d2 = 0x600030) and the five random numbers of 0x5661A. Every frame of the
//! flight it moves; when `+$AC` and `+$AE` (24 to 39 and 32 to 47 frames) run out it drops a feather (pool A class 48,
//! 0x5670A); when `+$A4` (32 to 47) is negative the speed falls by 0x400 (`+$B4`) a frame.
//!
//! | mode | what |
//! | --- | --- |
//! | 0 | 0x56580: live 2, no census, no shadow, the table 0x5674E, Y and depth 0, the terrain, the animation 0 (1 when a random number is even), mode 2 and the frame at once |
//! | 2 | 0x565E4: the sub state, the animation step 0x121E, the far test 0x173C, the draw list 0x49CA |
//! | 4, 6 | 0x56748: free |
//!
//! `neg.w $B4(a6)` of the variant 0x200 negates the high word of the long 0x400, which is 0: the speed still falls.

use super::Enemy;
use crate::game::{Game, Handle};

/// Offsets of this class.
mod f {
    pub const FRAMES: usize = 0x5C;
    pub const A4: usize = 0xA4;
    pub const RISE: usize = 0xAA;
    pub const A_C: usize = 0xAC;
    pub const A_E: usize = 0xAE;
    /// long: the X speed, long: its deceleration.
    pub const VX: usize = 0xB0;
    pub const DECEL: usize = 0xB4;
}

const TABLE: u32 = 0x5674E;

impl Game {
    /// 0x5656C class 46 (jump table 0x56578: mode 0 -> 0x56580, 2 -> 0x565E4, 4 and 6 -> 0x56748).
    pub fn enemy_class46(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                e.b.live = 2;
                e.m.sb(super::off::ATTACKING, 0xFF);
                e.b.sub = 0;
                e.b.draw_flags = 0x10;
                e.b.anim_table = TABLE;
                e.m.sw(0xA0, 0);
                e.m.sw(0xA2, 0x64);
                e.m.sw(0xA6, 0x10);
                e.b.set_yi(0); // move.w #0,$C(a6)
                e.b.set_zi(0);
                let _ = self.terrain_collide(&mut e.b, h, false);
                self.enemy_anim(&mut e.b, 0);
                if self.rand() & 1 == 0 {
                    self.enemy_anim(&mut e.b, 1);
                }
                e.b.mode += 2;
                self.enemy_c46_frame(e, h);
            }
            2 => self.enemy_c46_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x56748: jmp 0x48FC
            }
        }
    }

    /// 0x565E4.
    fn enemy_c46_frame(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => self.enemy_c46_wait(e),
            2 => self.enemy_c46_fly(e),
            _ => {} // 0x56746: rts
        }
        self.enemy_step(&mut e.b); // jsr 0x121E
        let i = self.enemy_index(h);
        self.enemy_despawn_if_far(e, i); // jsr 0x173C
        self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
    }

    /// 0x5660A: wait `+$5C` frames, then the take off (0x5661A).
    fn enemy_c46_wait(&mut self, e: &mut Enemy) {
        let w = e.m.w(f::FRAMES);
        if w != 0 {
            e.m.sw(f::FRAMES, w.wrapping_sub(1));
            return;
        }
        e.b.sub += 2;
        e.m.sl(f::VX, 0);
        let hi = (self.rand() & 3) + 4; // move.w d0,$B0(a6) then addi.w #4: the high word of the long
        e.m.sw(f::VX, hi);
        e.m.sw(f::RISE, (self.rand() & 1) + 1);
        e.m.sw(f::A_C, (self.rand() & 0xF) + 0x18);
        e.m.sw(f::A_E, (self.rand() & 0xF) + 0x20);
        e.m.sw(f::A4, (self.rand() & 0xF) + 0x20);
        e.m.sl(f::DECEL, 0x400);
        if e.b.var26 == 0x200 {
            e.m.sw(f::VX, e.m.w(f::VX).wrapping_neg()); // neg.w $B0(a6): the high word
            e.m.sw(f::DECEL, e.m.w(f::DECEL).wrapping_neg()); // neg.w $B4(a6): the high word of 0x400, 0
            e.b.facing = 1;
        }
        self.sound(0x61); // jsr 0xA63C
    }

    /// 0x566CA: the flight.
    fn enemy_c46_fly(&mut self, e: &mut Enemy) {
        let c = e.m.w(f::A_C).wrapping_sub(1);
        e.m.sw(f::A_C, c);
        if c == 0 {
            self.enemy_c46_feather(e);
        }
        let c = e.m.w(f::A_E).wrapping_sub(1);
        e.m.sw(f::A_E, c);
        if c == 0 {
            self.enemy_c46_feather(e);
        }
        e.b.x = e.b.x.wrapping_add(e.m.l(f::VX) as i32); // add.l $B0(a6),$8(a6)
        let y = e.b.yi().wrapping_add(e.m.w(f::RISE) as i16); // add.w $AA(a6),$C(a6)
        e.b.set_yi_keep(y);
        let a4 = e.m.w(f::A4).wrapping_sub(1);
        e.m.sw(f::A4, a4);
        if (a4 as i16) < 0 {
            let v = e.m.l(f::VX).wrapping_sub(e.m.l(f::DECEL)); // sub.l $B4(a6),$B0(a6)
            e.m.sl(f::VX, v);
        }
    }

    /// 0x5670A: a feather (pool A class 48, variant 0) at the bird's position.
    fn enemy_c46_feather(&mut self, e: &Enemy) {
        if let Some(i) = self.enemies.alloc() {
            let b = &mut self.enemies.slots[i].b;
            b.live = 1;
            b.k20 = 0xC0;
            b.set_xi(e.b.xi());
            b.set_yi(e.b.yi());
            b.set_zi(e.b.zi());
            b.var26 = 0;
            b.dir = 0;
        }
    }
}
