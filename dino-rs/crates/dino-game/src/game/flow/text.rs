//! Text printers of the front end: the scroll1 string routines of the original and the typewriter.
//!
//! All of them write cells (tile code word, attribute word) into the scroll1 tilemap with the
//! original's VRAM addresses: base 0x90A000 (scroll1 row 32), a column is +0x80 and a row is +4
//! (`hud.rs` documents the layout; `hud_put` does the writes and wraps the address). A string
//! is a byte or word list read from the ROM tables (`assets.frontend.text.slice_from(addr)`).
//!
//! | routine | what | here |
//! | --- | --- | --- |
//! | 0xBB6 / 0xBAE | byte string with its own position, print / erase | [`Game::text_bytes`] |
//! | 0xC0A | byte string at a1, '/' = next row | `hud_print` (`hud.rs`) |
//! | 0xC4C / 0xC44 | word string with its own position, print / erase | [`Game::text_words_at`] |
//! | 0xC9A | word string at a1, '//' = next row | [`Game::text_words`] |
//! | 0xCD2 / 0xCE4 / 0xCDA | big font (2x2 cells per letter) at a1, own position, erase | [`Game::text_big`] |
//! | 0x84F72 | tall font (1 column, 2 rows per letter), own position | [`Game::text_tall`] |
//! | 0xE12 / 0xE0A | tall font word string with its own position, print / erase | [`Game::text_tall_at`] |
//! | 0xE68 | tall font word string at a1, '//' = two rows down | [`Game::text_tall_a1`] |
//! | 0x84736 | portrait block and a tall text by id | [`Game::text_portrait`] |
//! | 0x7EC20 / 0x7EC48 | the typewriter (a pool B object in the original) | [`Typer`] |
//!
//! Byte strings: `col, row, attribute, characters ... '@'` (0x40 ends, '/' = 0x2F starts a new
//! position block); the attribute byte is ORed with 0x180. Word strings end with 0x4040 and a
//! new position block starts with 0x2F2F. Characters are tile codes (no letter decode).

// The services are used by the scenes of the next tasks (T1.2 to T1.8); until they are, parts of
// them are unused.
#![allow(dead_code)]

use crate::game::Game;
use std::mem;

/// Scroll1 row 32, column 0: where the position bytes of the strings count from.
const BASE: u32 = 0x90A000;

/// The string reader: the 68000 code walks a pointer, this walks an index.
struct Rd<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> Rd<'a> {
    fn new(s: &'a [u8]) -> Self {
        Rd { s, i: 0 }
    }
    fn u8(&mut self) -> u8 {
        let v = *self.s.get(self.i).unwrap_or_else(|| panic!("text string runs past the end of its table (byte {})", self.i));
        self.i += 1;
        v
    }
    fn u16(&mut self) -> u16 {
        let hi = self.u8() as u16;
        (hi << 8) | self.u8() as u16
    }
    /// The position block: column byte, row byte (`lsl.w #7` and `lsl.w #2`, added to a1).
    fn position(&mut self) -> u32 {
        let col = self.u8() as u32;
        let row = self.u8() as u32;
        BASE + (col << 7) + (row << 2)
    }
}

/// Which glyph printer: the big font writes 2x2 cells per letter (0xCD2 family and the
/// typewriter), the tall font 1x2 cells (0x84F72).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Glyphs {
    Big,
    Tall,
}

impl Game {
    /// 0xBB6 (`print`) and 0xBAE (erase): byte string with its own position blocks. The erase
    /// version writes a space (code 0x20, attribute 0) where each character would be.
    pub fn text_bytes(&mut self, s: &[u8], print: bool) {
        let mut r = Rd::new(s);
        'block: loop {
            let mut a1 = r.position(); // 0xBBE
            let mut attr = r.u8() as u16 | 0x180;
            loop {
                let c = r.u8();
                match c {
                    0x40 => return,
                    0x2F => continue 'block,
                    _ => {
                        let code = if print {
                            c as u16
                        } else {
                            attr = 0;
                            0x20
                        };
                        self.hud_put(a1, code, attr);
                        a1 += 0x80;
                    }
                }
            }
        }
    }

    /// 0xC4C (`print`) and 0xC44 (erase): word string with its own position blocks (column and row
    /// bytes, then the attribute WORD, which is not ORed with anything), then tile code words;
    /// 0x4040 ends, 0x2F2F starts the next position block.
    pub fn text_words_at(&mut self, s: &[u8], print: bool) {
        let mut r = Rd::new(s);
        'block: loop {
            let mut a1 = r.position(); // 0xC54
            let mut attr = r.u16();
            loop {
                let w = r.u16();
                match w {
                    0x4040 => return,
                    0x2F2F => continue 'block,
                    _ => {
                        let code = if print {
                            w
                        } else {
                            attr = 0;
                            0x20
                        };
                        self.hud_put(a1, code, attr);
                        a1 += 0x80;
                    }
                }
            }
        }
    }

    /// 0xC9A: word string at `a1`: the attribute word (ORed with 0x180) then tile code words;
    /// 0x4040 ends, 0x2F2F goes to the next row of the first column.
    pub fn text_words(&mut self, s: &[u8], a1: u32) {
        let mut r = Rd::new(s);
        let attr = r.u16() | 0x180;
        let (mut a1, mut a2) = (a1, a1);
        loop {
            let w = r.u16();
            match w {
                0x4040 => return,
                0x2F2F => {
                    a2 += 4;
                    a1 = a2;
                }
                _ => {
                    self.hud_put(a1, w, attr);
                    a1 += 0x80;
                }
            }
        }
    }

    /// The big font printers 0xCD2 (`at` = Some(a1): print at a1, the string starts with the
    /// attribute word), 0xCE4 (`at` = None, print) and 0xCDA (`at` = None, `print` false = erase):
    /// letters are 2x2 cells (code, code + 0x10 below, code + 1 and code + 0x11 to the right).
    /// The strings also hold marks (see [`Game::text_glyph_mark`]) and 0x2F2F starts a position block.
    pub fn text_big(&mut self, s: &[u8], at: Option<u32>, print: bool) {
        self.text_glyphs(s, at, Glyphs::Big, print)
    }

    /// 0x84F72: the tall font, always printed, with its own position blocks.
    pub fn text_tall(&mut self, s: &[u8]) {
        self.text_glyphs(s, None, Glyphs::Tall, true)
    }

    /// 0xE12 (`print`) and 0xE0A (erase): a tall letter string (code and code + 0x10 below, one
    /// column per letter) with its own position blocks and the attribute word as it is. The credit
    /// text uses it. No marks: every word is a letter.
    pub fn text_tall_at(&mut self, s: &[u8], print: bool) {
        let mut r = Rd::new(s);
        'block: loop {
            let mut a1 = r.position(); // 0xE1A
            let mut attr = r.u16();
            loop {
                let w = r.u16();
                match w {
                    0x4040 => return,
                    0x2F2F => continue 'block,
                    _ => {
                        let code = if print {
                            w
                        } else {
                            attr = 0;
                            0x20
                        };
                        self.hud_put(a1, code, attr);
                        self.hud_put(a1 + 4, code.wrapping_add(0x10), attr);
                        a1 += 0x80;
                    }
                }
            }
        }
    }

    /// 0xE68: tall letters at `a1`; the string starts with the attribute word (used as it is);
    /// 0x2F2F starts the next line two rows down in the first column (letters are two rows high).
    pub fn text_tall_a1(&mut self, s: &[u8], a1: u32) {
        let mut r = Rd::new(s);
        let attr = r.u16();
        let (mut a1, mut a2) = (a1, a1);
        loop {
            let w = r.u16();
            match w {
                0x4040 => return,
                0x2F2F => {
                    a2 += 8;
                    a1 = a2;
                }
                _ => {
                    self.hud_put(a1, w, attr);
                    self.hud_put(a1 + 4, w.wrapping_add(0x10), attr);
                    a1 += 0x80;
                }
            }
        }
    }

    fn text_glyphs(&mut self, s: &[u8], at: Option<u32>, font: Glyphs, print: bool) {
        let mut r = Rd::new(s);
        let mut a1 = at;
        'block: loop {
            let mut p = match a1.take() {
                Some(a) => a,
                None => r.position(), // 0xCEA / 0x84F78
            };
            let mut attr = r.u16();
            loop {
                let d0 = r.u16();
                match d0 {
                    0x4040 => return,
                    0x2F2F => continue 'block,
                    _ => {}
                }
                if let Some(adv) = self.text_glyph_mark(font, d0, p) {
                    p = p.wrapping_add(adv);
                    continue;
                }
                // A letter. The erase version of the big font writes spaces with attribute 0.
                let code = if print {
                    d0
                } else {
                    attr = 0;
                    0x20
                };
                self.hud_put(p, code, attr);
                self.hud_put(p.wrapping_add(4), code.wrapping_add(0x10), attr);
                match font {
                    Glyphs::Tall => p = p.wrapping_add(0x80),
                    Glyphs::Big => {
                        self.hud_put(p.wrapping_add(0x80), code.wrapping_add(1), attr);
                        self.hud_put(p.wrapping_add(0x84), code.wrapping_add(0x11), attr);
                        p = p.wrapping_add(0x100);
                    }
                }
            }
        }
    }

    /// The special words of the glyph strings (marks and punctuation drawn with their own tile at
    /// attribute 0, even by the erase version). Returns how far the write address moves, or `None`
    /// for an ordinary letter. Addresses as the `lea` instructions of 0xCD2 / 0x84F72 / 0x7EC48.
    fn text_glyph_mark(&mut self, font: Glyphs, w: u16, a1: u32) -> Option<u32> {
        // (cell offset, tile code, advance); the offsets are from `lea d(a1),a1` before the write.
        let big = font == Glyphs::Big;
        let (off, code, adv): (i32, u16, u32) = match (font, w) {
            (Glyphs::Big, 0x7E7E) => (-0x84, 0x1B1, 0),
            (Glyphs::Big, 0x2829) => (-0x84, 0x1B0, 0),
            (Glyphs::Tall, 0x2727) => (0, 0x2C, 0x80),
            (Glyphs::Tall, 0x5E5E) => (0, 0x22, 0x80),
            (_, 0x2C2C) => (4, if big { 0x1B2 } else { 0x2C }, if big { 0x100 } else { 0x80 }),
            (_, 0x2E2E) => (4, if big { 0x1B0 } else { 0x2E }, if big { 0x100 } else { 0x80 }),
            (_, 0x5B5B) => (-4, 0x18F, 0x80),
            (_, 0x5D5D) => (8, 0x1BF, 0x84),
            _ => return None,
        };
        self.hud_put(a1.wrapping_add(off as u32), code, 0);
        Some(adv)
    }

    /// 0x84736: a 9x9 cell portrait at `a0` (`portrait` = d0, 0..17, 0x63 = none) and then the tall
    /// font text number `text` (d1, a pointer in the table 0x84800; negative = no text, which also
    /// picks the second attribute list for the portrait). The text box (35 columns x 9 rows from
    /// 0x90A898) is cleared first except for the text ids 0x2A..0x2D.
    pub fn text_portrait(&mut self, a0: u32, portrait: u16, text: i16) {
        let t = &self.assets.frontend.text;
        if portrait != 0x63 {
            let d0 = portrait as u32;
            let pal = t.u16(0x8461A, 2 * d0); // kept in +$A2 of the caller's object
            let mut codes = t.u32(0x8463E, 4 * d0);
            let mut attrs = if text < 0 { t.u32(0x846DE, 4 * d0) } else { t.u32(0x84696, 4 * d0) };
            let mut a0 = a0;
            for _row in 0..9 {
                let mut a2 = a0;
                for _col in 0..9 {
                    let code = t.u16(codes, 0);
                    codes += 2;
                    // `add.b (a3)+,d0`: the byte add keeps the high byte of the palette word.
                    let attr = (pal & 0xFF00) | (pal as u8).wrapping_add(t.u8(attrs, 0)) as u16;
                    attrs += 1;
                    self.hud_put(a2, code, attr);
                    a2 += 0x80;
                }
                a0 += 4;
            }
        }
        if text < 0 {
            return;
        }
        if !(0x2A..=0x2D).contains(&text) {
            self.text_clear_box(0x90A898); // 0x84E0C
        }
        let t = &self.assets.frontend.text;
        let s = t.slice_from(t.u32(0x84800, 4 * text as u32));
        self.text_tall(s);
    }

    /// 0x84E0C: 35 columns of 9 cells filled with spaces (code 0x20, attribute 0) from `a0`.
    pub fn text_clear_box(&mut self, a0: u32) {
        for col in 0..35u32 {
            for row in 0..9u32 {
                self.hud_put(a0 + col * 0x80 + row * 4, 0x20, 0);
            }
        }
    }

    // ---------------------------------------------------------------- the typewriter

    /// 0x7EC20: starts a typewriter on `text` (a big font string, see [`Typer`]) with `period`
    /// frames per letter (the callers use 1). The original takes a pool B slot (class 0x68,
    /// handler 0x7EC48); here the typer lives in `flow.typers`.
    pub fn flow_typer_spawn(&mut self, text: &[u8], period: u16) {
        self.flow.typers.push(Typer::new(text, period));
    }

    /// 0x7EC20 in a level: the typewriter object takes a pool B slot (`jsr 0x4860`), live byte 1, class +$20 = 0x68, +$26 = 0, period +$86, the text +$80; it is
    /// stepped by the pool B update (`prop_class26`). A full pool makes none.
    pub fn flow_typer_spawn_pool_b(&mut self, text: &[u8], period: u16) {
        let Some(i) = self.props.alloc() else { return };
        let p = &mut self.props.slots[i];
        *p = Default::default();
        p.b.live = 1;
        p.b.k20 = 0x68;
        p.b.var26 = 0;
        let mut t = Typer::new(text, period);
        t.slot = Some(i);
        self.flow.typers.push(t);
    }

    /// True while a typewriter object exists (the scenes poll the pool B slot).
    pub fn flow_typer_busy(&self) -> bool {
        self.flow.typers.iter().any(|t| t.live)
    }

    /// One frame of every typewriter, in slot order: what the pool B update 0x23604 does with the
    /// objects of class 0x68. Scenes call it where the original loop runs the pool B update.
    pub fn flow_typers_step(&mut self) {
        let mut typers = mem::take(&mut self.flow.typers);
        for t in typers.iter_mut() {
            self.typer_step(t);
        }
        typers.retain(|t| t.live);
        typers.append(&mut self.flow.typers); // spawned by a step (none today)
        self.flow.typers = typers;
    }

    /// The object routine 0x7EC48 for one typewriter, one frame.
    pub fn typer_step(&mut self, t: &mut Typer) {
        if !t.live {
            return;
        }
        match t.state {
            0 => t.state = 2, // 0x7EC5C, then straight on to the sub state
            2 => {}
            _ => {
                // 0x7EEB4: state 4 frees the object
                t.live = false;
                return;
            }
        }
        match t.sub {
            0 => {
                // 0x7EC7C: counter, skip flag, the first position block. Row and column are masked.
                t.delay = t.period;
                t.fast = false;
                let mut r = Rd::new(&t.text);
                let col = (r.u8() & 0x3F) as u32;
                let row = (r.u8() & 0x3F) as u32;
                let _attr = r.u16(); // read and dropped: the typewriter writes attribute 0
                t.cursor = Typer::wrap(BASE + (col << 7) + (row << 2));
                t.pos = r.i;
                t.sub = 2;
                self.typer_run(t); // falls into 0x7ECCC in the same frame
            }
            2 => self.typer_run(t),
            _ => t.state = 4, // 0x7EEAE: state 4, freed next frame
        }
    }

    /// 0x7ECCC to 0x7EEA8: count down, type one item, test the skip key, repeat while skipping.
    fn typer_run(&mut self, t: &mut Typer) {
        loop {
            t.delay = t.delay.wrapping_sub(1);
            if t.delay == 0 {
                t.delay = t.period;
                if self.typer_item(t) {
                    return; // 0x4040: the string is done
                }
            }
            // 0x7EE8A: attack pressed on its own (G.$16 == 0x10, changed since G.$1B) skips the wait
            let (now, before) = (self.w.input[0], self.flow.front.prev_input[0]);
            if now != before && now == 0x10 {
                t.fast = true;
            }
            if !t.fast {
                return;
            }
        }
    }

    /// 0x7ED12: the next word of the string. Returns true at the end marker.
    fn typer_item(&mut self, t: &mut Typer) -> bool {
        loop {
            let mut r = Rd { s: &t.text, i: t.pos };
            let w = r.u16();
            t.pos = r.i;
            match w {
                0x4040 => {
                    t.fast = false;
                    t.sub = 4; // 0x7EE84
                    return true;
                }
                0x2F2F => {
                    // 0x7ECE0: a new position block (not masked here)
                    let a = r.position();
                    let _attr = r.u16();
                    t.cursor = Typer::wrap(a);
                    t.pos = r.i;
                    continue;
                }
                _ => {}
            }
            let a1 = t.cursor;
            let adv = match w {
                0x7E7E => Some((-0x84i32, 0x1B1u16, 0u32)),
                0x2829 => Some((-0x84, 0x1B0, 0)),
                0x2727 => Some((0, 0x27, 0x80)),
                0x2C2C => Some((4, 0x1B2, 0x100)),
                0x2E2E => Some((4, 0x1B0, 0x100)),
                0x5B5B => Some((-4, 0x18F, 0x80)),
                0x5D5D => Some((8, 0x1BF, 0x84)),
                _ => None,
            };
            if let Some((off, code, adv)) = adv {
                self.hud_put(a1.wrapping_add(off as u32), code, 0);
                t.cursor = Typer::wrap(a1.wrapping_add(adv));
                return false;
            }
            // A letter: the first pair of cells, and (Japan, or a code below 0x100) the second pair.
            self.hud_put(a1, w, 0);
            self.hud_put(a1.wrapping_add(4), w.wrapping_add(0x10), 0);
            let mut p = a1.wrapping_add(0x80);
            let next = w.wrapping_add(1);
            // `tst.w $32(a5)`: the region word is 0 for Japan; the World set writes 2 (G.$32).
            if self.flow.region.0 == 0 || next & 0xFF00 == 0 {
                p = Typer::wrap(p);
                self.hud_put(p, next, 0);
                self.hud_put(p.wrapping_add(4), next.wrapping_add(0x10), 0);
                p = p.wrapping_add(0x80);
            }
            t.cursor = Typer::wrap(p);
            return false;
        }
    }
}

/// The typewriter object of class 0x68 (pool B, routine 0x7EC48, spawned by 0x7EC20): prints a
/// big font string one item per `period` frames, starting in the frame it is first run. Letters
/// are written with attribute 0 at the position of the string's first block, wrapped into the
/// first scroll1 page (`& 0x909FFF | 0x908000`: rows 0..31). A press of attack alone on player 1
/// (G.$16 = 0x10 and changed since the last frame) prints the rest at once. The object lives one
/// more frame after the end marker.
#[derive(Clone, Debug, Default)]
pub struct Typer {
    pub live: bool,
    /// +4 state (0 new, 2 running, 4 done) and +5 sub state (0 init, 2 typing, 4 end).
    state: u8,
    sub: u8,
    text: Vec<u8>,
    /// +$80 write address, +$88 read position in the string.
    cursor: u32,
    pos: usize,
    /// +$84 frames to the next item, +$86 frames per item, +$8C skip flag.
    delay: u16,
    period: u16,
    fast: bool,
    /// The pool B slot that the object takes in the original (the object in the level: `prop/class26.rs`); `None` for the scenes that step it themselves.
    pub slot: Option<usize>,
}

impl Typer {
    pub fn new(text: &[u8], period: u16) -> Typer {
        Typer { live: true, text: text.to_vec(), period, ..Default::default() }
    }

    /// 0x7EC48: restore a pool B typewriter from its saved state. `text` starts
    /// at the saved read pointer (+$88), or at the first position block (+$80)
    /// before sub-state 0 has run. The bytes come from the runtime asset pack.
    pub(crate) fn from_pool_b_snapshot(text: &[u8], slot: usize, state_sub: (u8, u8), cursor: u32, timing: (u16, u16, u16)) -> Self {
        Self {
            live: true, state: state_sub.0, sub: state_sub.1, text: text.to_vec(),
            cursor, pos: 0, delay: timing.0, period: timing.1, fast: timing.2 != 0,
            slot: Some(slot),
        }
    }

    /// `move.l a1,d4; andi.l #$909fff,d4; ori.l #$908000,d4`.
    fn wrap(a: u32) -> u32 {
        (a & 0x909FFF) | 0x908000
    }

    /// The write address (+$80), for tests and for scenes that react to the cursor.
    pub fn cursor(&self) -> u32 {
        self.cursor
    }

    /// The state (+4) and sub state (+5) bytes of the object.
    pub fn state_sub(&self) -> (u8, u8) {
        (self.state, self.sub)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::flow::Region;
    use dino_assets::AssetPack;
    use dino_assets::frontend::FrontendData;
    use dino_assets::text::TextData;

    fn game() -> Game {
        Game::new(Box::leak(Box::new(AssetPack::default())))
    }

    fn game_with(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut a = AssetPack::default();
        a.frontend = FrontendData { text: TextData { blobs }, ..Default::default() };
        Game::new(Box::leak(Box::new(a)))
    }

    /// The cell at the VRAM address of the original.
    fn cell(g: &Game, a: u32) -> (u16, u16) {
        let c = g.vram.scroll1[((a - 0x908000) / 4) as usize & 0xFFF];
        (c.code, c.attr)
    }

    fn words(w: &[u16]) -> Vec<u8> {
        w.iter().flat_map(|v| v.to_be_bytes()).collect()
    }

    /// 0xBB6: column 5, row 3, attribute 2, then the letters down a row each +0x80 (one column to
    /// the right); the attribute is ORed with 0x180. '/' starts another block.
    #[test]
    fn byte_string_with_position() {
        let mut g = game();
        g.text_bytes(b"\x05\x03\x02AB/\x07\x04\x00C@", true);
        let a = 0x90A000 + 5 * 0x80 + 3 * 4;
        assert_eq!(cell(&g, a), (b'A' as u16, 0x182));
        assert_eq!(cell(&g, a + 0x80), (b'B' as u16, 0x182));
        assert_eq!(cell(&g, a + 0x100), (0, 0)); // nothing after the string
        assert_eq!(cell(&g, 0x90A000 + 7 * 0x80 + 4 * 4), (b'C' as u16, 0x180));
    }

    /// 0xBAE: the same string erases: spaces with attribute 0, also for the second block.
    #[test]
    fn byte_string_erase() {
        let mut g = game();
        let s = b"\x05\x03\x02AB/\x07\x04\x00C@";
        g.text_bytes(s, true);
        g.text_bytes(s, false);
        let a = 0x90A000 + 5 * 0x80 + 3 * 4;
        assert_eq!(cell(&g, a), (0x20, 0));
        assert_eq!(cell(&g, a + 0x80), (0x20, 0));
        assert_eq!(cell(&g, 0x90A000 + 7 * 0x80 + 4 * 4), (0x20, 0));
    }

    /// 0xC4C and 0xC44: the attribute is a word and is used as it is (no 0x180); codes are words.
    #[test]
    fn word_string_with_position() {
        let mut g = game();
        let mut s = vec![2, 1];
        s.extend(words(&[0x0185, 0x0141, 0x0142, 0x2F2F]));
        s.extend([3, 0]);
        s.extend(words(&[0x0007, 0x0143, 0x4040]));
        g.text_words_at(&s, true);
        let a = 0x90A000 + 2 * 0x80 + 4;
        assert_eq!(cell(&g, a), (0x141, 0x185));
        assert_eq!(cell(&g, a + 0x80), (0x142, 0x185));
        assert_eq!(cell(&g, 0x90A000 + 3 * 0x80), (0x143, 7));
        g.text_words_at(&s, false);
        assert_eq!(cell(&g, a), (0x20, 0));
        assert_eq!(cell(&g, 0x90A000 + 3 * 0x80), (0x20, 0));
    }

    /// 0xC9A: attribute word ORed with 0x180, then codes at a1 going right; 0x2F2F is the next
    /// row of the first column.
    #[test]
    fn word_string_at_a1() {
        let mut g = game();
        let a1 = 0x90A000 + 4 * 0x80 + 5 * 4;
        g.text_words(&words(&[0x0003, 0x10, 0x11, 0x2F2F, 0x20, 0x4040]), a1);
        assert_eq!(cell(&g, a1), (0x10, 0x183));
        assert_eq!(cell(&g, a1 + 0x80), (0x11, 0x183));
        assert_eq!(cell(&g, a1 + 4), (0x20, 0x183));
        assert_eq!(cell(&g, a1 + 0x100), (0, 0));
    }

    /// 0xC0A is `hud_print`: the attribute byte first, '/' for the next row.
    #[test]
    fn char_string_is_the_hud_print() {
        let mut g = game();
        g.hud_print(b"\x05AB/C@", 0x90A000);
        assert_eq!(cell(&g, 0x90A000), (b'A' as u16, 0x185));
        assert_eq!(cell(&g, 0x90A080), (b'B' as u16, 0x185));
        assert_eq!(cell(&g, 0x90A004), (b'C' as u16, 0x185));
    }

    /// 0xCE4: a letter is a 2x2 block: code and code + 0x10 below, code + 1 and code + 0x11 to the
    /// right, and the next letter starts two columns on. The attribute word is used as it is.
    #[test]
    fn big_font_letters_are_two_by_two() {
        let mut g = game();
        let mut s = vec![4, 2];
        s.extend(words(&[0x0006, 0x0100, 0x0120, 0x4040]));
        g.text_big(&s, None, true);
        let a = 0x90A000 + 4 * 0x80 + 2 * 4;
        assert_eq!(cell(&g, a), (0x100, 6));
        assert_eq!(cell(&g, a + 4), (0x110, 6));
        assert_eq!(cell(&g, a + 0x80), (0x101, 6));
        assert_eq!(cell(&g, a + 0x84), (0x111, 6));
        assert_eq!(cell(&g, a + 0x100), (0x120, 6));
        assert_eq!(cell(&g, a + 0x184), (0x131, 6));
        // 0xCDA (erase): every letter becomes d0 = 0x20 with attribute 0, but the code of the
        // other three cells still comes from d0 + 0x10 and d0 + 1 (a quirk: 0x20, 0x30, 0x21, 0x31)
        g.text_big(&s, None, false);
        assert_eq!(cell(&g, a), (0x20, 0));
        assert_eq!(cell(&g, a + 4), (0x30, 0));
        assert_eq!(cell(&g, a + 0x80), (0x21, 0));
        assert_eq!(cell(&g, a + 0x184), (0x31, 0));
    }

    /// 0xCD2: the position is given (a1) and the string starts with the attribute word. The marks
    /// have their own tile and attribute 0: the comma goes one row down and moves two columns,
    /// "~" sits up-left of the letter place and does not move, "[" is one row above and moves one
    /// column, "]" is two rows below and moves one column and one row.
    #[test]
    fn big_font_marks() {
        let mut g = game();
        let a1 = 0x90A000 + 10 * 0x80 + 10 * 4;
        g.text_big(&words(&[0x0005, 0x2C2C, 0x7E7E, 0x0100, 0x5B5B, 0x5D5D, 0x4040]), Some(a1), true);
        assert_eq!(cell(&g, a1 + 4), (0x1B2, 0)); // the comma, one row down
        let p = a1 + 0x100; // two columns on
        assert_eq!(cell(&g, p - 0x84), (0x1B1, 0)); // the "~" mark, up-left, no move
        assert_eq!(cell(&g, p), (0x100, 5)); // then the letter at the same place
        let p = p + 0x100; // the letter took two columns
        assert_eq!(cell(&g, p - 4), (0x18F, 0)); // "[" one row above
        let p = p + 0x80; // and moved one column
        assert_eq!(cell(&g, p + 8), (0x1BF, 0)); // "]" two rows below
    }

    /// 0xE12 / 0xE0A: tall letters at their own position, the attribute word as it is; the erase
    /// version writes code 0x20 (and 0x30 below) with attribute 0.
    #[test]
    fn tall_letters_with_position_and_at_a1() {
        let mut g = game();
        let mut s = vec![3, 2];
        s.extend(words(&[0x0004, 0x0160, 0x0161, 0x2F2F]));
        s.extend([9, 9]);
        s.extend(words(&[0x0005, 0x0162, 0x4040]));
        g.text_tall_at(&s, true);
        let a = 0x90A000 + 3 * 0x80 + 2 * 4;
        assert_eq!(cell(&g, a), (0x160, 4));
        assert_eq!(cell(&g, a + 4), (0x170, 4));
        assert_eq!(cell(&g, a + 0x80), (0x161, 4));
        assert_eq!(cell(&g, 0x90A000 + 9 * 0x80 + 9 * 4), (0x162, 5));
        g.text_tall_at(&s, false);
        assert_eq!(cell(&g, a), (0x20, 0));
        assert_eq!(cell(&g, a + 4), (0x30, 0));
        assert_eq!(cell(&g, 0x90A000 + 9 * 0x80 + 9 * 4 + 4), (0x30, 0));
        // 0xE68: a1 given, the next line is two rows down
        let a1 = 0x90A000 + 20 * 0x80;
        g.text_tall_a1(&words(&[0x0006, 0x0100, 0x2F2F, 0x0101, 0x4040]), a1);
        assert_eq!(cell(&g, a1), (0x100, 6));
        assert_eq!(cell(&g, a1 + 4), (0x110, 6));
        assert_eq!(cell(&g, a1 + 8), (0x101, 6));
        assert_eq!(cell(&g, a1 + 12), (0x111, 6));
    }

    /// 0x84F72: the tall font is one column wide: code, then code + 0x10 below; the cursor moves
    /// one column; the apostrophe and the comma use their own tiles at attribute 0.
    #[test]
    fn tall_font_letters_are_one_by_two() {
        let mut g = game();
        let mut s = vec![6, 1];
        s.extend(words(&[0x0002, 0x0160, 0x0161, 0x2727, 0x2C2C, 0x4040]));
        g.text_tall(&s);
        let a = 0x90A000 + 6 * 0x80 + 4;
        assert_eq!(cell(&g, a), (0x160, 2));
        assert_eq!(cell(&g, a + 4), (0x170, 2));
        assert_eq!(cell(&g, a + 0x80), (0x161, 2));
        assert_eq!(cell(&g, a + 0x84), (0x171, 2));
        assert_eq!(cell(&g, a + 0x100), (0x2C, 0)); // the apostrophe (top row)
        assert_eq!(cell(&g, a + 0x184), (0x2C, 0)); // the comma (row below)
    }

    /// Front end tables for the portrait routine: portrait 1 has the palette word 0x0111, its
    /// code list is 0x300 + k, the attribute list k and the second attribute list 0x80 + k, and
    /// text id 1 is the tall string "column 9, row 0, attribute 7: 0x170".
    fn portrait_tables() -> Vec<(u32, Vec<u8>)> {
        let mut t = vec![0u8; 0x300];
        t[0..4].copy_from_slice(&[0x00, 0x10, 0x01, 0x11]); // 0x8461A: palette words
        t[0x24 + 4..0x24 + 8].copy_from_slice(&0xA0000u32.to_be_bytes()); // 0x8463E + 4: code list
        t[0x7C + 4..0x7C + 8].copy_from_slice(&0xA0200u32.to_be_bytes()); // 0x84696 + 4: attributes
        t[0xC4 + 4..0xC4 + 8].copy_from_slice(&0xA0300u32.to_be_bytes()); // 0x846DE + 4: second list
        t[0x1E6 + 4..0x1E6 + 8].copy_from_slice(&0xA0400u32.to_be_bytes()); // 0x84800 + 4: text 1
        let mut l = vec![0u8; 0x500];
        for k in 0..81usize {
            l[2 * k..2 * k + 2].copy_from_slice(&(0x300 + k as u16).to_be_bytes());
            l[0x200 + k] = k as u8;
            l[0x300 + k] = 0x80 + k as u8;
        }
        let mut s = vec![9, 0];
        s.extend(words(&[0x0007, 0x0170, 0x4040]));
        l[0x400..0x400 + s.len()].copy_from_slice(&s);
        vec![(0x8461A, t), (0xA0000, l)]
    }

    /// 0x84736: the portrait is 9 rows of 9 cells from a0 (the code list is row by row), the
    /// attribute is the palette word with the list byte added to its low byte only. A text id
    /// outside 0x2A..0x2D clears the 35 x 9 box (from 0x90A898) first, then the tall text is
    /// printed.
    #[test]
    fn portrait_and_text() {
        let mut g = game_with(portrait_tables());
        g.hud_put(0x90A898 + 5 * 0x80 + 2 * 4, 0x41, 3); // inside the box: must be cleared
        let a0 = 0x90A000;
        g.text_portrait(a0, 1, 1);
        assert_eq!(cell(&g, a0), (0x300, 0x0111)); // palette 0x0111 + list byte 0
        assert_eq!(cell(&g, a0 + 0x80), (0x301, 0x0112)); // next column
        assert_eq!(cell(&g, a0 + 4), (0x309, 0x011A)); // item 9 is the second row
        assert_eq!(cell(&g, a0 + 8 * 0x80 + 8 * 4), (0x350, 0x0161)); // item 80, byte add 0x11 + 0x50
        assert_eq!(cell(&g, 0x90A898 + 5 * 0x80 + 2 * 4), (0x20, 0));
        assert_eq!(cell(&g, 0x90A000 + 9 * 0x80), (0x170, 7)); // the text, tall font
    }

    /// A negative text id prints only the portrait and takes the second attribute list; the
    /// portrait id 0x63 prints no portrait; the text ids 0x2A..0x2D do not clear the box.
    #[test]
    fn portrait_without_text_and_text_without_portrait() {
        let mut g = game_with(portrait_tables());
        g.text_portrait(0x90A000, 1, -1);
        assert_eq!(cell(&g, 0x90A000), (0x300, 0x0111 + 0x80)); // second list: 0x80 + k
        assert_eq!(cell(&g, 0x90A000 + 9 * 0x80), (0, 0)); // no text

        let mut t = portrait_tables();
        t[0].1[0x1E6 + 4 * 0x2A..0x1E6 + 4 * 0x2A + 4].copy_from_slice(&0xA0400u32.to_be_bytes());
        t[0].1.resize(0x300, 0);
        let mut g = game_with(t);
        g.hud_put(0x90A898, 0x41, 3);
        g.text_portrait(0x90A000, 0x63, 0x2A);
        assert_eq!(cell(&g, 0x90A898), (0x41, 3)); // 0x2A..0x2D: box not cleared
        assert_eq!(cell(&g, 0x90A000), (0, 0)); // no portrait
        assert_eq!(cell(&g, 0x90A000 + 9 * 0x80), (0x170, 7));
    }

    /// 0x7EC20 / 0x7EC48: the first item is typed in the frame the object first runs, with
    /// attribute 0, in the first page (row 24 of the string is row 24, not 56).
    #[test]
    fn typewriter_writes_attribute_zero_into_the_first_page() {
        let mut g = game();
        let mut s = vec![3, 24];
        s.extend(words(&[0x0006, 0x0040, 0x0060, 0x4040]));
        g.flow_typer_spawn(&s, 1);
        g.flow_typers_step();
        let a = 0x908000 + 3 * 0x80 + 24 * 4;
        assert_eq!(cell(&g, a), (0x40, 0));
        assert_eq!(cell(&g, a + 4), (0x50, 0));
        assert_eq!(cell(&g, a + 0x80), (0x41, 0)); // the second pair (World set, code below 0x100)
        assert_eq!(cell(&g, a + 0x84), (0x51, 0));
        assert_eq!(cell(&g, a + 0x100), (0, 0)); // the next letter comes next frame
        g.flow_typers_step();
        assert_eq!(cell(&g, a + 0x100), (0x60, 0)); // two columns on
    }

    /// For the World set (region word 2) the second pair of cells is written only for codes below
    /// 0x100 (the test is on code + 1); for Japan (region 0) always.
    #[test]
    fn typewriter_second_pair_depends_on_the_region() {
        for (region, code, second) in [(2u16, 0x00A0u16, true), (2, 0x0100, false), (0, 0x0100, true)] {
            let mut g = game();
            g.flow.region = Region(region);
            let mut s = vec![0, 0];
            s.extend(words(&[0, code, 0x4040]));
            g.flow_typer_spawn(&s, 1);
            g.flow_typers_step();
            let a = 0x908000;
            assert_eq!(cell(&g, a), (code, 0));
            assert_eq!(cell(&g, a + 0x80) == (code + 1, 0), second, "region {region} code {code:#x}");
        }
    }

    /// One letter per `period` frames, the first one after `period - 1` more frames of the init
    /// frame, the end marker is read at the next item time, and the object is freed in the
    /// second frame after it (the sub state 4 sets state 4, which frees).
    #[test]
    fn typewriter_timing_and_end() {
        let mut g = game();
        let mut s = vec![0, 0];
        s.extend(words(&[0, 0x0010, 0x0020, 0x0030, 0x4040]));
        g.flow_typer_spawn(&s, 2); // one item every 2 frames
        let (mut letters, mut busy) = (Vec::new(), Vec::new());
        for _ in 0..11 {
            g.flow_typers_step();
            letters.push((0..3).filter(|&i| cell(&g, 0x908000 + i * 0x100).0 != 0).count());
            busy.push(g.flow_typer_busy());
        }
        assert_eq!(letters, vec![0, 1, 1, 2, 2, 3, 3, 3, 3, 3, 3]);
        // end marker read in frame 7, state 4 set in frame 8, freed in frame 9
        assert_eq!(busy, vec![true, true, true, true, true, true, true, true, true, false, false]);
    }

    /// The marks of the typewriter strings: the apostrophe has its own tile (0x27) and moves one
    /// column; the comma sits one row down and moves two.
    #[test]
    fn typewriter_marks() {
        let mut g = game();
        let mut s = vec![0, 0];
        s.extend(words(&[0, 0x2727, 0x2C2C, 0x4040]));
        g.flow_typer_spawn(&s, 1);
        for _ in 0..3 {
            g.flow_typers_step();
        }
        assert_eq!(cell(&g, 0x908000), (0x27, 0));
        assert_eq!(cell(&g, 0x908000 + 0x80 + 4), (0x1B2, 0));
    }

    /// Attack pressed alone (G.$16 = 0x10, changed since the last frame) prints everything now.
    #[test]
    fn typewriter_skip_with_attack() {
        let mut s = vec![0, 0];
        s.extend(words(&[0, 0x0010, 0x0020, 0x0030, 0x4040]));
        let mut g = game();
        g.flow_typer_spawn(&s, 3);
        g.flow_typers_step();
        g.w.input[0] = 0x10; // pressed now, was 0 last frame
        g.flow_typers_step();
        assert!((0..3).all(|i| cell(&g, 0x908000 + i * 0x100).0 != 0));
        // a held button, or attack with a direction, does not skip
        let mut g = game();
        g.flow_typer_spawn(&s, 3);
        g.flow_typers_step();
        g.flow.front.prev_input[0] = 0x10;
        g.w.input[0] = 0x10;
        g.flow_typers_step();
        g.flow.front.prev_input[0] = 0;
        g.w.input[0] = 0x11;
        g.flow_typers_step();
        assert_eq!(cell(&g, 0x908000 + 0x100).0, 0);
    }
}
