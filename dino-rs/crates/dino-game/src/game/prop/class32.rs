//! Pool B class 32 (`+$20 = 0x80`, handler 0x817CE): the black bars (letterbox) of the opening movie's jungle (M1-10).
//! The movie handler of the counter 600 fills the cells of scroll1 with the opaque tile 0x18 (0x81888) and creates
//! this object. Its first update fills them again (0x817E6: the same cells, no change), then it opens a window in
//! the middle of the screen: two rows of 49 transparent cells (tile 0x1F, 0x8186C) at the rows 15 and 16 of the
//! page 0x90A000, and every 4 frames the two rows one step further away from the middle, 9 steps in all.
//!
//! | mode | sub | what |
//! | --- | --- | --- |
//! | 0 | | 0x817E2: mode 2, `+$80` = 0x3C (a word, so the byte `+$80` is 0), the fill 0x81888 |
//! | 2 | 0 | 0x81804: `+$80` = `+$82` = 10 (bytes), the two row pointers `+$84` = 0x90A03C and `+$88` = 0x90A040 |
//! | 2 | 2 | 0x8182A: `+$80` counts down; at 0 it is 4 again, `+$82` - 1, then (not at 0) `+$84` + 4 and `+$88` - 4, both rows |
//! | 4 | | 0x818DC: free (0x4938) |
//!
//! The 10th event (`+$82` at 0) ends it: sub and mode + 2 (0x81862, 0x81866).

use super::Prop;
use crate::game::Game;

/// Offsets of this class.
mod f {
    /// word (its high byte is the byte `+$80`): the wait counter.
    pub const T80: usize = 0x80;
    /// byte: the number of steps still to do.
    pub const STEPS: usize = 0x82;
    /// longs: the addresses of the two rows (down and up from the middle).
    pub const ROW_A: usize = 0x84;
    pub const ROW_B: usize = 0x88;
}

impl Game {
    /// 0x81888: all the cells of the rows 1 to 34 of the page 0x90A000 (49 columns each) get the code 0x18 and
    /// the attribute 0 (the movie calls it itself first and the bars again in their first update).
    pub fn prop_c32_fill(&mut self) {
        for r in 0..34u32 {
            let a1 = 0x90A000 + 4 * (r + 1);
            for k in 0..49u32 {
                self.hud_put(a1 + 0x80 * k, 0x18, 0);
            }
        }
    }

    /// 0x8186C: a row of 49 cells (the address steps by 0x80) with the code 0x1F and the attribute 0.
    fn prop_c32_row(&mut self, a0: u32) {
        for k in 0..49u32 {
            self.hud_put(a0 + 0x80 * k, 0x1F, 0);
        }
    }

    /// 0x817CE pool B class 32 (jump table 0x817DA: mode 0 -> 0x817E2, 2 -> 0x817F0, 4 and 6 -> 0x818DC).
    pub fn prop_class32(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode += 2; // 0x817E2
                p.m.sw(f::T80, 0x3C);
                self.prop_c32_fill(); // bra 0x81888
            }
            2 => match p.b.sub {
                0 => {
                    p.m.sb(f::T80, 10); // 0x81804
                    p.m.sb(f::STEPS, 10);
                    p.m.sl(f::ROW_A, 0x90A03C);
                    p.m.sl(f::ROW_B, 0x90A040);
                    p.b.sub += 2;
                }
                _ => {
                    let t = p.m.b(f::T80).wrapping_sub(1); // 0x8182A
                    p.m.sb(f::T80, t);
                    if t != 0 {
                        return;
                    }
                    p.m.sb(f::T80, 4);
                    let steps = p.m.b(f::STEPS).wrapping_sub(1);
                    p.m.sb(f::STEPS, steps);
                    if steps == 0 {
                        p.b.sub += 2; // 0x81862
                        p.b.mode += 2; // 0x81866
                        return;
                    }
                    let a = p.m.l(f::ROW_A).wrapping_add(4);
                    p.m.sl(f::ROW_A, a);
                    self.prop_c32_row(a);
                    let b = p.m.l(f::ROW_B).wrapping_sub(4);
                    p.m.sl(f::ROW_B, b);
                    self.prop_c32_row(b);
                }
            },
            _ => self.prop_free(p, i), // 0x818DC: jmp 0x4938
        }
    }
}
