//! Pool B class 66 (`+$20 = 0x108`, handler 0x86D76): a caption banner, a line of the tall font that appears on
//! scroll1 for a while (the movie's hunters call it with 0x86D2E and the variants 0x12 and 0x13, M1-10; the other
//! users, the scripted scenes of the game, come with M2-10). The variant (`+$26`) selects the entries of the tables
//! at 0x86DD8 (frames to show it, `+$A2`), 0x86EB4 (frames before it, `+$A0`), 0x875F2 (a flag byte, `+$A8`),
//! 0x87652 (the variant of the next banner, `+$AA`) and the text (the pointer table 0x87710).
//!
//! | mode | sub | what |
//! | --- | --- | --- |
//! | 0 | | 0x86D8A: one `rand()` (a chance of 1 in 128 would call 0x86B3C, the scan of the vblank code, which never finds a patch, so nothing happens), mode 2, the four fields from the tables; the frame ends here |
//! | 2 | 0 | 0x86FA6: wait `+$A0` frames |
//! | 2 | 2 | 0x86FBA: print the text of the variant (0x84F72) |
//! | 2 | 4 | 0x87186: wait `+$A2` frames |
//! | 2 | 6 | 0x87194: print the text that erases it (0x8757C for the variants 0x12 and 0x13, 0x872A2 below 0x14, 0x874BE from 0x14), the next banner `+$AA` when `+$A8` is not 0, mode 4 |
//! | 4 | | 0x87220: free (0x4938) |

use super::Prop;
use crate::game::Game;

/// Offsets of this class.
mod f {
    pub const A0: usize = 0xA0;
    pub const A2: usize = 0xA2;
    pub const A8: usize = 0xA8;
    pub const AA: usize = 0xAA;
}

/// The tables of the original (in `FRONT_TABLES`).
const TIME_SHOWN: u32 = 0x86DD8;
const TIME_BEFORE: u32 = 0x86EB4;
const FLAGS: u32 = 0x875F2;
const NEXT: u32 = 0x87652;
const TEXTS: u32 = 0x87710;
/// The texts that erase the banner.
const ERASE_LOW: u32 = 0x872A2;
const ERASE_HIGH: u32 = 0x874BE;
const ERASE_MOVIE: u32 = 0x8757C;

impl Game {
    /// 0x86D2E: a caption banner of the variant `var` (pool B class 66); a full pool makes none.
    pub fn prop_caption_spawn(&mut self, var: u16) {
        let Some(i) = self.props.alloc() else { return };
        let p = &mut self.props.slots[i];
        p.b.live = 1;
        p.b.k20 = 0x108;
        p.b.var26 = var;
    }

    /// 0x86D76 pool B class 66 (jump table 0x86D82: mode 0 -> 0x86D8A, 2 -> 0x86F90, 4 and 6 -> 0x87220).
    pub fn prop_class66(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                let _ = self.rand() & 0x7F; // 0x86D8A: the 1 in 128 chance calls 0x86B3C, which never finds a patch
                p.b.mode += 2;
                let var = p.b.var26 as u32;
                let t = &self.assets.frontend.text;
                p.m.sb(f::A8, t.u8(FLAGS, var));
                p.m.sw(f::A2, t.u16(TIME_SHOWN, 2 * var));
                p.m.sw(f::A0, t.u16(TIME_BEFORE, 2 * var));
                p.m.sw(f::AA, t.u16(NEXT, 2 * var));
            }
            2 => self.prop_c66_frame(p),
            _ => self.prop_free(p, i), // 0x87220: jmp 0x4938
        }
    }

    /// 0x86F90.
    fn prop_c66_frame(&mut self, p: &mut Prop) {
        match p.b.sub {
            0 => {
                // 0x86FA6
                if p.m.w(f::A0) == 0 {
                    p.b.sub += 2;
                } else {
                    p.m.sw(f::A0, p.m.w(f::A0).wrapping_sub(1));
                }
            }
            2 => {
                // 0x86FBA: the text of the variant (the pointer table by variant * 4)
                let t = &self.assets.frontend.text;
                let s = t.slice_from(t.u32(TEXTS, 4 * p.b.var26 as u32));
                self.text_tall(s);
                p.b.sub += 2;
            }
            4 => {
                // 0x87186
                let t = p.m.w(f::A2).wrapping_sub(1);
                p.m.sw(f::A2, t);
                if t == 0 {
                    p.b.sub += 2;
                }
            }
            _ => {
                // 0x87194
                let var = p.b.var26 as i16;
                let mut text = ERASE_LOW;
                if var >= 0x14 {
                    self.prop_c66_blank_cells(); // bsr 0x871FC
                    text = ERASE_HIGH;
                }
                if var == 0x12 || var == 0x13 {
                    text = ERASE_MOVIE;
                }
                let s = self.assets.frontend.text.slice_from(text);
                self.text_tall(s);
                if p.m.b(f::A8) != 0 {
                    self.prop_caption_spawn(p.m.w(f::AA)); // bsr 0x86D2E
                }
                p.b.mode += 2;
            }
        }
    }

    /// 0x871FC: two cells at 0x90A068 (the second one a column on) get the code 0x1F, attribute 0.
    fn prop_c66_blank_cells(&mut self) {
        for k in 0..2u32 {
            self.hud_put(0x90A068 + 0x80 * k, 0x1F, 0);
        }
    }
}
