//! Tilemap buffers and palette memory owned by the game logic.
//!
//! These are plain data, not hardware emulation: the original game code computes byte offsets
//! into 64x64-cell maps and writes palette words, and keeping the same layout lets the ported code
//! do exactly the same writes. The GPU renderer reads them when it draws a frame.

use dino_assets::stage::Cell;

pub const CELLS: usize = 64 * 64;
/// Words of the "other" RAM 0x915800 .. 0x918000 (the row scroll task clears all of it).
pub const ROWSCROLL_WORDS: usize = 0x1400;

/// The three tile layers. Cell index = original VRAM byte offset / 4.
pub struct Vram {
    /// 8x8 text / HUD layer (also a background in stage 6 area 1).
    pub scroll1: Vec<Cell>,
    /// 16x16 playfield.
    pub scroll2: Vec<Cell>,
    /// 32x32 far background.
    pub scroll3: Vec<Cell>,
    /// The "other" RAM 0x915800 .. 0x918000 (CPS-A register 0x108 = 0x9158): the row scroll table, 1024 words used. Screen line
    /// s (0 to 223) adds `rowscroll[(s + 16) & 0x3FF]` to the scroll2 X when the video control bit 0 is set (stage 2 only).
    pub rowscroll: Vec<u16>,
}

impl Default for Vram {
    fn default() -> Self {
        Vram {
            scroll1: vec![Cell::default(); CELLS],
            scroll2: vec![Cell::default(); CELLS],
            scroll3: vec![Cell::default(); CELLS],
            rowscroll: vec![0; ROWSCROLL_WORDS],
        }
    }
}

impl Vram {
    /// Cell index of (col, row) in each layer (address routines 0x20650 / 0x214C0 / 0x23090).
    pub fn index1(col: usize, row: usize) -> usize {
        (col * 0x80 + (row & 31) * 4 + (row >> 5) * 0x2000) / 4
    }
    pub fn index2(col: usize, row: usize) -> usize {
        (col * 0x40 + (row & 15) * 4 + (row >> 4) * 0x1000) / 4
    }
    pub fn index3(col: usize, row: usize) -> usize {
        (col * 0x20 + (row & 7) * 4 + (row >> 3) * 0x800) / 4
    }

    /// 0xB6C / 0xB88: clear scroll2 / scroll3 with their blank tiles.
    pub fn clear2(&mut self) {
        self.scroll2.fill(Cell { code: 0x5800, attr: 0 });
    }
    pub fn clear3(&mut self) {
        self.scroll3.fill(Cell { code: 0x1C00, attr: 0 });
    }
}

/// Palette memory: 4 banks (sprites, scroll1, scroll2, scroll3) x 32 palettes x 16 colours.
pub struct PaletteRam {
    pub words: Vec<u16>,
    pub dirty: bool,
    /// The colours the screen shows: the palette of the frame before (the video hardware copies the
    /// palette memory in the vertical blank, one frame late, see `latch`). Empty until the first frame ends.
    shown: Vec<u16>,
    prev: Vec<u16>,
}

impl Default for PaletteRam {
    fn default() -> Self {
        PaletteRam { words: vec![0; 2048], dirty: true, shown: Vec::new(), prev: Vec::new() }
    }
}

impl PaletteRam {
    pub const OBJ: usize = 0;
    pub const SCROLL1: usize = 512;
    pub const SCROLL2: usize = 1024;
    pub const SCROLL3: usize = 1536;

    pub fn bank_mut(&mut self, bank: usize) -> &mut [u16] {
        self.dirty = true;
        &mut self.words[bank * 512..bank * 512 + 512]
    }

    /// The end of a frame: the screen shows the palette as it was at the end of the frame before (the
    /// recordings of the fades: a MAME picture of a fade step shows the brightness of the step before).
    pub fn latch(&mut self) {
        if self.shown.is_empty() {
            self.shown.clone_from(&self.words);
            self.prev.clone_from(&self.words);
            self.dirty = true;
            return;
        }
        if self.prev != self.shown {
            self.shown.clone_from(&self.prev);
            self.dirty = true;
        }
        self.prev.clone_from(&self.words);
    }

    /// The colours the screen shows now (`words` before the first frame ends).
    pub fn shown_words(&self) -> &[u16] {
        if self.shown.is_empty() { &self.words } else { &self.shown }
    }

    pub fn copy(&mut self, start: usize, words: &[u16]) {
        self.words[start..start + words.len()].copy_from_slice(words);
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The screen shows the palette of the frame before.
    #[test]
    fn the_screen_shows_the_palette_one_frame_late() {
        let mut p = PaletteRam::default();
        assert_eq!(p.shown_words()[0], 0);
        let mut shown = Vec::new();
        for v in 1..=4u16 {
            p.words[0] = v;
            p.latch();
            shown.push(p.shown_words()[0]);
        }
        assert_eq!(shown, [1, 1, 2, 3]);
    }
}
