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
    pub const AC: usize = 0xAC;
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
                // 0x86FBA: the text of the variant (the pointer table by variant * 4); `clr.b $AC(a6)` first (the compares after it
                // change nothing: every branch goes to 0x87008)
                p.m.sb(f::AC, 0);
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

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use std::path::Path;
    use std::sync::OnceLock;

    /// The user's ROM set (`dino.zip` in the repository root); without it the tests of this module pass without checking anything.
    fn assets() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let rom = Path::new("../../../dino.zip");
            rom.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(rom).expect("the ROM set loads"))))
        })
    }

    fn cells(g: &Game) -> Vec<(u16, u16)> {
        g.vram.scroll1.iter().map(|c| (c.code, c.attr)).collect()
    }

    /// M2-10: every caption variant that the game uses (0 to 0x67: the places of the Episodes 1 to 8 and the boss speech lines; the flag
    /// and chain tables are 0x60 entries long and the variants 0x68 to 0x6F read other data, they are never made) can be shown with the
    /// real strings. The object waits, prints its text, waits, erases it, makes the next banner of its chain when the table says so and frees
    /// itself, whatever the variant: no string runs past the extracted blob, and no banner stays for ever.
    #[test]
    fn every_caption_variant_runs_to_its_end() {
        let Some(a) = assets() else { return };
        for var in 0..0x68u16 {
            let mut g = Game::new(a);
            g.spawn_prop(66, var, None).unwrap();
            let blank = cells(&g);
            let mut printed = false;
            let mut frames = 0;
            while g.props.slots.iter().any(|p| p.b.live != 0) {
                g.pool_b_update();
                printed |= cells(&g) != blank;
                frames += 1;
                assert!(frames < 3000, "variant {var:#x} does not end");
            }
            // The only variant that shows nothing is the empty string of Episode 8 area 3 (var 7).
            assert!(printed || var == 7, "variant {var:#x} printed nothing");
        }
    }

    /// The banner object follows the tables: it waits `+$A0` frames, prints for `+$A2` frames and frees itself, so its life is the sum
    /// plus the init frame and the print step (the tables are read from the ROM, the sum is checked against the object's own fields).
    #[test]
    fn a_banner_lives_as_long_as_its_tables_say() {
        let Some(a) = assets() else { return };
        for var in [9u16, 0x12, 0x27, 0x50, 0x61, 0x66] {
            let mut g = Game::new(a);
            let h = g.spawn_prop(66, var, None).unwrap();
            let i = (0..24).find(|&k| g.props.handle(k) == h).unwrap();
            g.pool_b_update(); // mode 0: the four fields from the tables
            let (before, shown, next) = {
                let p = &g.props.slots[i];
                (p.m.w(f::A0) as u32, p.m.w(f::A2) as u32, p.m.b(f::A8))
            };
            let mut frames = 1;
            while g.props.slots[i].b.live != 0 {
                g.pool_b_update();
                frames += 1;
                assert!(frames < 3000);
            }
            // init (1) + wait (before + 1) + print (1) + show (shown) + erase (1) + free (1)
            assert_eq!(frames, 1 + before + 1 + 1 + shown + 1 + 1, "variant {var:#x} (next banner flag {next})");
        }
    }
}
