//! The map panel of the link scenes (M2-25): the framed picture of the map with the name plate of the place, drawn into scroll1 and slid in from the
//! right, and the blinking marker of the next place. The same routines are used by the stage clear tally (`tally.rs`, states 4 and 6) and by the intro card
//! of the next episode (`flow.rs`, M2-26). The panel is a picture of tile cells in the first page of scroll1 (0x908010) that the slide copies, column by
//! column, to the columns behind the right edge while scroll1 moves left; the colours are the palettes 1 to 13 of scroll1 (0x914400 + 0x20 * n), loaded by
//! stage from the ROM palettes 0xBC252 + 0x20 * n; the marker is a change of some of these palettes.
//!
//! All tables are by the stage G.$4D9 (the stage that has just been cleared): `FRONT_TABLES`.

use crate::game::vram::PaletteRam;
use crate::game::Game;

/// The ROM tables.
mod rom {
    /// The 32 palettes of the map (16 words each).
    pub const PALETTES: u32 = 0xBC252;
    /// 8 pointers to the marker lists: (palette slot, palette, palette) words, ended by a negative word.
    pub const MARKERS: u32 = 0xCACE;
    /// 8 pointers to the 13 palette numbers of the stage.
    pub const PALETTE_LISTS: u32 = 0xCD6C;
    /// The tile codes (380 words) and the attributes (380 words) of the frame and the picture.
    pub const CODES: u32 = 0xCEC4;
    pub const ATTRS: u32 = 0xD1BC;
    /// The 19 columns of 6 cells (codes, then attributes) of the stages 6 and 7.
    pub const EXTRA_CODES: u32 = 0xD4DC;
    pub const EXTRA_ATTRS: u32 = 0xD5C0;
    /// The 12 columns of 3 cells of the stage 7.
    pub const LAST_CODES: u32 = 0xD6CE;
    pub const LAST_ATTRS: u32 = 0xD716;
    /// 8 pointers to the 12 tile codes of the name plate.
    pub const PLATES: u32 = 0xD780;
}

impl Game {
    /// 0xCD1A: the palettes 1 to 13 of scroll1 from the ROM palettes the stage's list names; the whole bank then gets the full brightness (mask 2, 0x8932),
    /// and the mask stays 2.
    pub(crate) fn flow_card_palettes(&mut self) {
        let stage = self.w.stage as u32;
        let t = &self.assets.frontend.text;
        let list = t.u32(rom::PALETTE_LISTS, 4 * stage);
        for n in 1..=13u32 {
            let index = t.u16(list, 2 * (n - 1)) as u32;
            let words: Vec<u16> = (0..16).map(|k| t.u16(rom::PALETTES + 32 * index, 2 * k)).collect();
            self.pal.copy(PaletteRam::SCROLL1 + 16 * n as usize, &words);
        }
        self.fade.mask = 2;
        self.brightness_full_current(); // jsr 0x8932
    }

    /// 0xCB7A: the palette `slot` of scroll1 is the ROM palette `index` with the brightness F.
    fn flow_card_palette(&mut self, slot: u16, index: u16) {
        let t = &self.assets.frontend.text;
        let words: Vec<u16> = (0..16).map(|k| t.u16(rom::PALETTES + 32 * index as u32, 2 * k) | 0xF000).collect();
        self.pal.copy(PaletteRam::SCROLL1 + 16 * slot as usize, &words);
    }

    /// 0xCAA8: the marker of the next place: every (slot, palette, palette) of the stage's list gets the first palette (`blink` 0) or the second (1).
    pub(crate) fn flow_card_marker(&mut self, blink: bool) {
        let stage = self.w.stage as u32;
        let t = &self.assets.frontend.text;
        let mut a = t.u32(rom::MARKERS, 4 * stage);
        let mut set = Vec::new();
        loop {
            let d0 = t.u16(a, 0);
            if d0 & 0x8000 != 0 {
                break;
            }
            let d1 = if blink { t.u16(a, 4) } else { t.u16(a, 2) };
            set.push((d0, d1));
            a += 6;
        }
        for (slot, index) in set {
            self.flow_card_palette(slot, index);
        }
    }

    /// One tile cell of a panel: code word, attribute word at `a`, from the two tables at the offsets `k` words.
    fn flow_card_cell(&mut self, a: u32, codes: u32, attrs: u32, k: u32) {
        let t = &self.assets.frontend.text;
        let (code, attr) = (t.u16(codes, 2 * k), t.u16(attrs, 2 * k));
        self.hud_put(a, code, attr);
    }

    /// A block of `columns` columns of `rows` cells at `a0` (column by column, a cell is a code and an attribute from the two tables, `k` words in);
    /// returns the next word of the tables and the address of the next column.
    fn flow_card_block(&mut self, mut a0: u32, tables: (u32, u32), mut k: u32, columns: u32, rows: u32) -> (u32, u32) {
        for _ in 0..columns {
            let mut a = a0;
            for _ in 0..rows {
                self.flow_card_cell(a, tables.0, tables.1, k);
                k += 1;
                a = a.wrapping_add(4);
            }
            a0 = a0.wrapping_add(0x80);
        }
        (k, a0)
    }

    /// 0xCE5C: the panel at `a0` (the frame and the picture in three blocks, 10 x 13, 8 x 15 and 10 x 13 cells: the second starts two cells above the
    /// first block's next column, the third where the second ended), and for the stages 6 and 7 the extra blocks (0xD4B4, 0xD6A4).
    pub(crate) fn flow_card_panel(&mut self, a0: u32) {
        let t = (rom::CODES, rom::ATTRS);
        let (k, a) = self.flow_card_block(a0, t, 0, 10, 13);
        let (k, a) = self.flow_card_block(a.wrapping_sub(8), t, k, 8, 15);
        self.flow_card_block(a.wrapping_add(8), t, k, 10, 13);
        let stage = self.w.stage;
        if stage >= 6 {
            self.flow_card_block(a0.wrapping_add(0x498), (rom::EXTRA_CODES, rom::EXTRA_ATTRS), 0, 19, 6); // 0xD4B4
        }
        if stage == 7 {
            self.flow_card_block(a0.wrapping_add(0x4A4), (rom::LAST_CODES, rom::LAST_ATTRS), 0, 12, 3); // 0xD6A4
        }
    }

    /// 0xD75E: the name plate of the place, 12 tile codes in a row (the codes only) at the panel + 0x42C.
    pub(crate) fn flow_card_plate(&mut self, panel: u32) {
        let t = &self.assets.frontend.text;
        let list = t.u32(rom::PLATES, 4 * self.w.stage as u32);
        let mut a = panel.wrapping_add(0x42C);
        for k in 0..12 {
            let code = self.assets.frontend.text.u16(list, 2 * k);
            self.hud_word(a, code);
            a = a.wrapping_add(0x80);
        }
    }
}
