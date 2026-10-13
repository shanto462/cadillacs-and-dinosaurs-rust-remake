//! Pool B class 72 (`+$20 = 0x120`, handler 0x89984): the spawner of a flock of birds (pool A class 46). The hunters
//! of the opening movie's jungle create one with the variant 0x600 (0x3DA96, M1-10).
//!
//! Mode 0 (0x89998) sets `+$A0` = 8 (frames between two birds) and `+$A2` = 6 (birds + 1), for a variant other than
//! 0 `+$A2` = 8 (0x400: 0xC, nothing uses it) and goes on in mode 2 (0x899DC) in the same frame. Mode 2: `+$A0`
//! counts down; at 0 `+$A2` counts down, at 0 the object ends (mode 4), else a bird is made (0x899FA) and `+$A0` is
//! 8 again. The bird (pool A class 46, palette override 0x98): X = the scroll2 camera + 0xB0 + (`rand()` & 0x3C)
//! (0x40 less for the variant 0x600), Y = camera Y + 0x40, depth = camera depth, the spawner's `+$5C` (0) copied, its
//! variant 0 (0x600 and 0x400 also), and for the variant 0x400 every other bird goes the other way (variant 0x200). A
//! full pool makes no bird (the original skips the rest of the code, the timer is not reloaded).

use super::Prop;
use crate::game::Game;

/// Offsets of this class.
mod f {
    pub const A0: usize = 0xA0;
    pub const A2: usize = 0xA2;
}

impl Game {
    /// 0x89DB4: the flock spawner of the variant 0x600.
    pub fn prop_flock_spawn(&mut self) {
        let Some(i) = self.props.alloc() else { return };
        let p = &mut self.props.slots[i];
        p.b.live = 1;
        p.b.k20 = 0x120;
        p.b.var26 = 0x600;
    }

    /// 0x89984 pool B class 72 (jump table 0x89990: mode 0 -> 0x89998, 2 -> 0x899DC, 4 and 6 -> 0x89AB4).
    pub fn prop_class72(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode = 2; // 0x89998: move.b #2,$4(a6)
                p.b.live = 2;
                p.b.facing = 0;
                p.m.sw(f::A0, 8);
                p.m.sw(f::A2, 6);
                if p.b.var26 != 0 {
                    p.m.sw(f::A0, 8);
                    p.m.sw(f::A2, 8);
                    if p.b.var26 == 0x400 {
                        p.m.sw(f::A2, 0xC);
                    }
                }
                self.prop_c72_frame(p);
            }
            2 => self.prop_c72_frame(p),
            _ => self.prop_free(p, i), // 0x89AB4: jmp 0x4938
        }
    }

    /// 0x899DC.
    fn prop_c72_frame(&mut self, p: &mut Prop) {
        let a0 = p.m.w(f::A0).wrapping_sub(1);
        p.m.sw(f::A0, a0);
        if a0 != 0 {
            return;
        }
        let a2 = p.m.w(f::A2).wrapping_sub(1);
        p.m.sw(f::A2, a2);
        if a2 == 0 {
            p.b.mode = 4; // move.b #4,$4(a6)
            return;
        }
        // 0x899FA: a bird
        if let Some(j) = self.enemies.alloc() {
            let var = p.b.var26;
            let cx = self.cam.s2.cam_x as i16;
            let r = self.rand();
            let e = &mut self.enemies.slots[j];
            e.b.live = 1;
            e.b.k20 = 0xB8;
            e.b.palette_override = 0x98;
            let mut x = cx.wrapping_add(0xB0).wrapping_add((r & 0x3C) as i16);
            if var == 0x600 {
                x = x.wrapping_sub(0x40);
            }
            e.b.set_xi(x);
            e.b.set_yi((self.cam.s2.cam_y as i16).wrapping_add(0x40));
            e.b.set_zi(0); // G.$74C
            e.b.dir = 0;
            e.m.sw(0x5C, p.m.w(0x5C));
            e.b.var26 = var;
            if var == 0x600 {
                e.b.var26 = 0;
            }
            if var == 0x400 {
                e.b.var26 = 0;
                if p.m.w(f::A2) & 1 == 0 {
                    e.b.var26 = 0x200;
                }
            }
        }
        p.m.sw(f::A0, 8);
    }
}
