//! Pool B class 93 (`+$20 = 0x174`, handler 0x91FB4): the checker board behind a profile card
//! (M1-06). The card handlers 0x9A950 .. 0x9AB32 create it with the card number as variant.
//!
//! Mode 0 (0x91FC8) clears the scroll3 map (0xB88), loads the colours of the board into scroll3
//! palette 0 (0x921AE keeps the brightness nibble), puts the scroll3 camera at 0 / 0, clears the map
//! again and draws the board: a checker of two tile codes (the table 0x92158, four words per card: the
//! codes of the two tiles in checker order) in four strips of 3 x 8 blocks of 2 x 2 cells
//! (0x920F6). The strips start at 0x90FFE0, 0x9107E0, 0x910FE0 and 0x9137E0: one column left of
//! the map (the first cells fall into the end of scroll2, where the original's address arithmetic
//! puts them) in four of its eight row pages. Variant 4 draws one wider strip (12 x 8 blocks at
//! 0x9137C0, table 0x92178) and, in mode 2, starts a fade in (0x8824, not ported: no object of the
//! cards creates it). Mode 2 does nothing (the object never frees itself; the cards' pool B slots
//! are used up one by one). Mode 4 frees (0x921D4).
//!
//! The first update takes two clears of 4096 cells and so crosses the video frame boundary in the
//! original; the profile cards write that as two halves (`CUT_FIELD`, `flow/profiles.rs`): the
//! first one (clear, colours) in the frame of the pool B update, the second (the rest) when the
//! card's pass ends, `Game::prop_card_board_finish`.

use super::Prop;
use crate::game::flow::profiles::CUT_FIELD;
use crate::game::Game;

/// Offsets of this class.
mod f {
    /// word: 0x12C (not used).
    pub const TIMER: usize = 0xA0;
    /// long: cleared (not used).
    pub const L_AA: usize = 0xAA;
    /// word: 0x1E (not used).
    pub const W_B6: usize = 0xB6;
    /// long: the table of the card (a1 of the original).
    pub const TABLE: usize = 0xBA;
}

/// `CUT_FIELD` values: 1 = the first update is to be split, 2 = its first half is done.
const SPLIT: u8 = 1;
const HALF_DONE: u8 = 2;

/// The palette words of the board (scroll3 palette 0, 0x914C00).
const BOARD_PALETTE: u32 = 0xC5952;
/// The tile code table of the cards, 8 bytes each, and the one of variant 4.
const BOARD_CODES: u32 = 0x92158;
const BOARD_CODES_V4: u32 = 0x92178;

impl Game {
    /// 0x91FB4 pool B class 93 (jump table 0x91FC0: mode 0 -> 0x91FC8, 2 -> 0x92180, 4 -> 0x921D4).
    pub fn prop_class93(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => self.prop_c93_init(p),
            2 => {} // 0x92180: variant 4 starts a fade in (not ported), then the mode stays 2
            _ => self.prop_free(p, i), // 0x921D4: jmp 0x4938
        }
    }

    /// 0x91FC8: the clear and the colours, then (`prop_c93_finish`) the camera, the second clear
    /// and the board.
    fn prop_c93_init(&mut self, p: &mut Prop) {
        self.vram.clear3(); // jsr 0xB88
        self.prop_c93_palette(); // bsr 0x921AE
        if p.m.b(CUT_FIELD) == SPLIT {
            p.m.sb(CUT_FIELD, HALF_DONE); // the frame ends here
            return;
        }
        self.prop_c93_finish(p);
    }

    /// 0x921AE: scroll3 palette 0 gets the 16 words of 0xC5952 under its brightness nibble.
    fn prop_c93_palette(&mut self) {
        let words: Vec<u16> = (0..16).map(|k| self.assets.frontend.text.u16(BOARD_PALETTE, 2 * k)).collect();
        let bank = self.pal.bank_mut(3);
        for (w, new) in bank.iter_mut().zip(words) {
            *w = (*w & 0xF000) | new;
        }
    }

    /// 0x91FD2 on: G.$7E4 and G.$7E8 to 0, the second clear, mode 2, the object's words, the board.
    fn prop_c93_finish(&mut self, p: &mut Prop) {
        self.cam.s3.cam_x = 0; // move.w #0,$7E4(a5)
        self.cam.s3.cam_y = 0; // move.w #0,$7E8(a5)
        self.vram.clear3(); // jsr 0xB88
        p.b.mode = 2;
        p.m.sw(f::TIMER, 0x12C);
        p.m.sl(f::L_AA, 0);
        p.m.sw(f::W_B6, 0x1E);
        let var = p.b.var26;
        if var == 4 {
            // 0x92048: the fade mask 0xC and brightness 0 for those banks (0x8938), one wide strip
            self.fade.mask = 0x0C;
            self.brightness_low();
            self.prop_c93_blocks(BOARD_CODES_V4, 0x9137C0, 0xB, 7);
        } else if var < 4 {
            // 0x92070: the four strips; a1 is kept in +$BA between them
            let table = BOARD_CODES + 8 * var as u32;
            p.m.sl(f::TABLE, table);
            for at in [0x90FFE0, 0x9107E0, 0x910FE0, 0x9137E0] {
                self.prop_c93_blocks(p.m.l(f::TABLE), at, 2, 7);
            }
        }
        p.m.sb(CUT_FIELD, 0);
    }

    /// The model of the frame boundary: finishes the first update of every board object whose first
    /// half is done (called by the profile pass after the pool B update of the frame before).
    pub(crate) fn prop_card_board_finish(&mut self) {
        for i in 0..self.props.slots.len() {
            let s = &self.props.slots[i];
            if s.b.live == 0 || s.b.k20 / 4 != 93 || s.b.mode != 0 || s.m.b(CUT_FIELD) != HALF_DONE {
                continue;
            }
            let mut p = std::mem::take(&mut self.props.slots[i]);
            self.prop_c93_finish(&mut p);
            self.props.slots[i] = p;
        }
    }

    /// 0x920F6: `d1 + 1` blocks of 2 x 2 cells side by side from `a0`, `d2 + 1` rows of them (each row 8
    /// bytes on: two cells down); the four words of the table are the cells (top left, below it, right
    /// of it, below that), each plus 0x1C00 with attribute 0. The cells are written by address: a
    /// column of a map is 0x20 bytes for scroll3 and the address runs on into the next page.
    fn prop_c93_blocks(&mut self, table: u32, a0: u32, d1: u32, d2: u32) {
        let words: Vec<u16> = (0..4).map(|k| self.assets.frontend.text.u16(table, 2 * k).wrapping_add(0x1C00)).collect();
        let mut row = a0;
        for _ in 0..=d2 {
            let mut a4 = row;
            for _ in 0..=d1 {
                for (off, w) in [(0, words[0]), (4, words[1]), (0x20, words[2]), (0x24, words[3])] {
                    self.prop_c93_put(a4 + off, w);
                }
                a4 += 0x40;
            }
            row += 8;
        }
    }

    /// One cell (code word, attribute 0) by video memory address: the maps are one after the other
    /// at 0x908000 (scroll1), 0x90C000 (scroll2) and 0x910000 (scroll3), 0x4000 bytes each.
    fn prop_c93_put(&mut self, a: u32, code: u16) {
        let (layer, base) = match a {
            0x908000..=0x90BFFF => (&mut self.vram.scroll1, 0x908000),
            0x90C000..=0x90FFFF => (&mut self.vram.scroll2, 0x90C000),
            0x910000..=0x913FFF => (&mut self.vram.scroll3, 0x910000),
            _ => return,
        };
        let cell = &mut layer[((a - base) / 4) as usize];
        cell.code = code;
        cell.attr = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use std::path::Path;
    use std::sync::OnceLock;

    fn assets() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    fn lit(cells: &[dino_assets::stage::Cell], blank: u16) -> usize {
        cells.iter().filter(|c| c.code != blank).count()
    }

    /// One board per card: 216 cells of scroll3 and the 8 last cells of scroll2 (the strip starts one
    /// column left of the map) hold the checker tiles, the tile codes are 0x1C00 + the two codes of the
    /// card's table words, and the scroll3 camera is 0 / 0. The split first update (as the profile
    /// cards run it) gives the same board once it is finished, and has done nothing visible before.
    #[test]
    fn the_board_of_each_card() {
        if assets().is_none() {
            return;
        }
        for card in 0..4u16 {
            let mut whole = Game::new(assets().unwrap());
            whole.cam.s3.cam_y = 0x1234;
            whole.flow_profile_spawn_b(93, card);
            whole.props.slots[23].m.sb(CUT_FIELD, 0); // not split
            whole.pool_b_update();
            let mut split = Game::new(assets().unwrap());
            split.cam.s3.cam_y = 0x1234;
            split.flow_profile_spawn_b(93, card);
            split.pool_b_update();
            assert_eq!((lit(&split.vram.scroll3, 0x1C00), split.cam.s3.cam_y), (0, 0x1234), "the first half draws nothing");
            assert_eq!(split.pal.words[1536..1552], whole.pal.words[1536..1552], "but the colours are there");
            split.prop_card_board_finish();
            for g in [&whole, &split] {
                assert_eq!((lit(&g.vram.scroll3, 0x1C00), lit(&g.vram.scroll2, 0), g.cam.s3.cam_y, g.props.slots[23].b.mode), (216, 8, 0, 2), "card {card}");
                let codes: std::collections::BTreeSet<u16> = g.vram.scroll3.iter().map(|c| c.code).filter(|&c| c != 0x1C00).collect();
                let t = &g.assets.frontend.text;
                let want: std::collections::BTreeSet<u16> = (0..4).map(|k| 0x1C00 + t.u16(BOARD_CODES, 8 * card as u32 + 2 * k)).collect();
                assert_eq!(codes, want, "card {card}");
            }
            assert!(whole.vram.scroll3.iter().zip(&split.vram.scroll3).all(|(a, b)| (a.code, a.attr) == (b.code, b.attr)));
        }
    }
}
