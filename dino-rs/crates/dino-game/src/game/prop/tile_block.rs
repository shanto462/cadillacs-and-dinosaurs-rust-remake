//! The tile block writer of the scenery props (T-B0, M2-05 / M2-06): one small service for the ~60 pool B classes that
//! rewrite tiles of the three scroll layers (door smashes, breaking walls, water and lamp animations, one shot tile
//! changes). Owner: enemy port. Spec: D3e part 1 section 1.4 (tile geometry), D3e part 3 section 2.
//!
//! # The model
//!
//! The original writes tiles by video address (`movea.l #$90F288,a0`, then `move.w (a1)+,(a0)+` loops). A tile is 4
//! bytes: the code word, then the attribute word (bits 0 to 4 palette, 5 and 6 flips, 7 and 8 priority group). The
//! three maps are one after the other (scroll1 0x908000, scroll2 0x90C000, scroll3 0x910000, 0x4000 bytes each), each
//! a ring of 64 x 64 cells, and the address arithmetic of the original (`+4` is the next row, `+col step` the next
//! column, the address simply runs on) is kept: a block that is taller than the rest of its 16 row page spills into
//! the next column, as the 68000 code does.
//!
//! | layer | base | next row | next column | rows per page |
//! | --- | --- | --- | --- | --- |
//! | scroll1 (8 x 8) | 0x908000 | +4 | +0x80 | 32 |
//! | scroll2 (16 x 16) | 0x90C000 | +4 | +0x40 | 16 |
//! | scroll3 (32 x 32) | 0x910000 | +4 | +0x20 | 8 |
//!
//! - [`TilePos`] is the address of a cell. Build it from an address of the original code (`TilePos::at(0x90F288)`: the
//!   props are ported next to the instruction, so keep the address and say where it came from), from the ring
//!   coordinates (`TilePos::new(Layer::Scroll2, col, row)`, column and row wrap at 64), or from a world position
//!   (`TilePos::scroll2_at_world(x, y)` = the helper 0x75208 of the breakable wall class 0).
//! - The writers are `Game::prop_tile_put` (code and attribute), `prop_tile_code`, `prop_tile_attr` (`and` then `or` on
//!   the attribute word, the loops of the original that only change bits) and the rectangles `prop_tile_block` and
//!   `prop_tile_attr_block` (column by column, each column top to bottom, as the original loops).
//! - Source data (tile code lists) are ROM tables. They come through `dino-extract`: add the table to `TABLES` of
//!   `dino-extract/src/enemy.rs` (an exact copy of the bytes) and pass its ROM address. The writer reads
//!   `assets.enemy.t` and panics when a byte was not extracted, so a wrong address shows up in the first test run.
//! - [`BlockTable`] plays a table of block pointers one block every `period` frames (the tile animations).
//!
//! # How to build a scenery prop with tile_block
//!
//! A prop is one file `game/prop/bNN.rs` (a child module of `prop.rs` with `#[path]`, see `class93.rs`), one dispatch
//! line in `Game::pool_b_update` and one number in `PORTED_CLASSES` ("prop") of `trace.rs`. The three shapes:
//!
//! 1. **One shot patch** (door smash, class 11 at 0x78A38). Port the routine as it is, replace the address loop by one
//!    call and keep the original address in a comment:
//!
//!    ```ignore
//!    // routine 2 (0x78A38): the sound, the codes, the attributes, the dust effect, mode 4.
//!    self.snd_break(p.b.xi());                                           // jsr 0x9E60: break sound with pan
//!    // 0x78A5A: 3 columns of 7 tile codes (the list at 0x78A8E, extracted) from 0x90E9E4
//!    self.prop_tile_block(TilePos::at(0x90E9E4), 3, 7, 0x78A8E, TileMode::Codes);
//!    // 0x78AF0: 8 cells with attribute bit 7 set (and.w #$FE7F, or.w #$80) from 0x90EA66, then 7 columns of 8 with bit 8
//!    self.prop_tile_attr_block(TilePos::at(0x90EA66), 1, 8, 0xFE7F, 0x80);
//!    self.prop_tile_attr_block(TilePos::at(0x90EA66).right(1), 7, 8, 0xFE7F, 0x100);
//!    ```
//!
//! 2. **Animation** (hall door, class 12 var 0 at 0x78B86): keep the table in a `const`, start it in mode 0, step it
//!    every frame in mode 2, go to the end mode when it reports done:
//!
//!    ```ignore
//!    const HALL_DOOR: BlockTable = BlockTable {
//!        table: 0x78BDA, count: 3,                       // the pointer table (3 longs) of 9 x 9 cells
//!        at: TilePos::at(0x90F288), cols: 9, rows: 9,
//!        mode: TileMode::PairsMerge(0xFFC0),             // code, attribute = (old & 0xFFC0) | source
//!        period: 4, looped: false,
//!    };
//!    // mode 0:  HALL_DOOR.start(p, 1);                  // first block on the next update (+$A0 = 1, +$A2 = 0)
//!    // mode 2:  if self.prop_block_table_step(p, &HALL_DOOR) { p.b.mode += 2; }
//!    ```
//!
//! 3. **Breakable** (steam pipes, class 40 at 0x836A4; wall, class 0): set the prop contract in mode 0, register in
//!    the hit list every frame, and write the broken picture in mode 4 (the hit pass of 0x10CD0 sets the state long to
//!    0x04000000 and puts the player's attack data in +$59, +$68, +$6C, +$6E; the prop never checks its HP):
//!
//!    ```ignore
//!    // mode 0 (0x836B8): +$48 = 1, +$49 = 0, +$44 = box table, HP 8.
//!    p.hit_init(0x8380A, 8);
//!    // mode 2: every frame (0x8371A, jmp 0x4BFA)
//!    self.prop_register_hit(self.props.handle(i));
//!    // mode 4: the break
//!    self.snd_break(p.b.xi());                           // only when the original calls 0x9E60
//!    self.prop_tile_block(pos, 3, 4, 0x837E8, TileMode::Codes);
//!    ```
//!
//! A script action that makes a prop (door smash, GO hint, tile change) calls `Game::spawn_prop` or
//! `Game::spawn_script_prop(action)` (`spawn.rs`). Tests of a prop build a game with synthetic tables
//! (`AssetPack::default()`, `enemy.t.blobs`), as the tests of this file do: no ROM bytes in source.
// A service for the scenery prop tasks of M3: nothing calls it before the first of them is ported.
#![allow(dead_code)]

use super::Prop;
use crate::game::vram::Vram;
use crate::game::Game;
use dino_assets::stage::Cell;

/// The three tile layers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Scroll1,
    Scroll2,
    Scroll3,
}

impl Layer {
    /// First video address of the layer's map.
    pub const fn base(self) -> u32 {
        match self {
            Layer::Scroll1 => 0x908000,
            Layer::Scroll2 => 0x90C000,
            Layer::Scroll3 => 0x910000,
        }
    }

    /// Address distance of two columns (the rows are 4 bytes apart in every layer).
    pub const fn col_step(self) -> u32 {
        match self {
            Layer::Scroll1 => 0x80,
            Layer::Scroll2 => 0x40,
            Layer::Scroll3 => 0x20,
        }
    }

    fn of(addr: u32) -> Option<Layer> {
        match addr {
            0x908000..=0x90BFFF => Some(Layer::Scroll1),
            0x90C000..=0x90FFFF => Some(Layer::Scroll2),
            0x910000..=0x913FFF => Some(Layer::Scroll3),
            _ => None,
        }
    }
}

/// The video address of a tile cell (see the module doc). Plain data: copy it, add to it with `right` and `down`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TilePos(pub u32);

impl TilePos {
    /// A cell by the address the original code uses. The address of the attribute word (cell + 2) is the same
    /// cell, so it is rounded down to the cell.
    pub const fn at(addr: u32) -> TilePos {
        TilePos(addr & !3)
    }

    /// A cell by ring coordinates: column and row wrap at 64 (D3e 1.4). The map column of a world X is `x / tile size`
    /// modulo 64.
    pub fn new(layer: Layer, col: u16, row: u16) -> TilePos {
        let (col, row) = ((col & 63) as usize, (row & 63) as usize);
        let i = match layer {
            Layer::Scroll1 => Vram::index1(col, row),
            Layer::Scroll2 => Vram::index2(col, row),
            Layer::Scroll3 => Vram::index3(col, row),
        };
        TilePos(layer.base() + 4 * i as u32)
    }

    /// 0x75208 (class 0, the breakable wall): the scroll2 cell of the world position `x`, `y` (column from X, the row
    /// counts from the top of the 1024 pixel map: it is `63 - y / 16`).
    pub fn scroll2_at_world(x: u16, y: u16) -> TilePos {
        let col = (x & 0x3F0) << 2;
        let up = 0x3F0 - (y & 0x3F0);
        let row = ((up & 0x300) << 4) + ((up & 0xF0) >> 2);
        TilePos(0x90C000 + (col + row) as u32)
    }

    /// `n` columns to the right (the step of the layer of this address).
    pub fn right(self, n: i32) -> TilePos {
        let step = Layer::of(self.0).map_or(0x40, Layer::col_step) as i32;
        TilePos((self.0 as i32).wrapping_add(n * step) as u32)
    }

    /// `n` rows down: `+4` a row, the address runs on over the end of a page as the original's loops do.
    pub fn down(self, n: i32) -> TilePos {
        TilePos((self.0 as i32).wrapping_add(4 * n) as u32)
    }

    /// The layer and the ring coordinates of the cell (the inverse of `new`), `None` outside the three maps.
    pub fn layer_col_row(self) -> Option<(Layer, u16, u16)> {
        let layer = Layer::of(self.0)?;
        let i = (self.0 - layer.base()) / 4;
        let (col, row) = match layer {
            Layer::Scroll1 => ((i >> 5) & 0x3F, (i & 0x1F) | (((i >> 11) & 1) << 5)),
            Layer::Scroll2 => ((i >> 4) & 0x3F, (i & 0xF) | (((i >> 10) & 3) << 4)),
            Layer::Scroll3 => ((i >> 3) & 0x3F, (i & 7) | (((i >> 9) & 7) << 3)),
        };
        Some((layer, col as u16, row as u16))
    }
}

/// What a block write does with the source words (the three loops the original uses).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileMode {
    /// Two words a cell, code then attribute: both replace the cell (`move.w (a1)+,(a0)+` twice).
    Pairs,
    /// Two words a cell: the code replaces, the attribute becomes `(old & keep) | source` (0x78BC4: `keep` 0xFFC0
    /// clears the palette and flip bits and keeps the priority bits of the map).
    PairsMerge(u16),
    /// One word a cell: the code only, the attribute stays (0x78A68, 0x837BC).
    Codes,
}

impl Game {
    /// The cell at `at`, `None` when the address is outside the three maps (the original would write palette or
    /// other video RAM there: not modeled).
    fn prop_tile_cell(&mut self, at: TilePos) -> Option<&mut Cell> {
        let layer = Layer::of(at.0)?;
        let map = match layer {
            Layer::Scroll1 => &mut self.vram.scroll1,
            Layer::Scroll2 => &mut self.vram.scroll2,
            Layer::Scroll3 => &mut self.vram.scroll3,
        };
        map.get_mut(((at.0 - layer.base()) / 4) as usize)
    }

    /// `move.w code,(a0)+ ; move.w attr,(a0)+`: both words of the cell.
    pub fn prop_tile_put(&mut self, at: TilePos, code: u16, attr: u16) {
        if let Some(c) = self.prop_tile_cell(at) {
            *c = Cell { code, attr };
        }
    }

    /// `move.w code,(a0)`: the code word only.
    pub fn prop_tile_code(&mut self, at: TilePos, code: u16) {
        if let Some(c) = self.prop_tile_cell(at) {
            c.code = code;
        }
    }

    /// `and.w keep,attr ; or.w or,attr` on the attribute word (`keep` 0xFFFF and `or` 0 do nothing).
    pub fn prop_tile_attr(&mut self, at: TilePos, keep: u16, or: u16) {
        if let Some(c) = self.prop_tile_cell(at) {
            c.attr = (c.attr & keep) | or;
        }
    }

    /// A rectangle of `cols` columns of `rows` cells, from the cell `at`, column by column and each column top to
    /// bottom, taking the words of the ROM table at `src` in that order (`mode`).
    pub fn prop_tile_block(&mut self, at: TilePos, cols: u16, rows: u16, src: u32, mode: TileMode) {
        let t = &self.assets.enemy.t;
        let mut s = src;
        for c in 0..cols as i32 {
            for r in 0..rows as i32 {
                let cell = at.right(c).down(r);
                match mode {
                    TileMode::Pairs => {
                        self.prop_tile_put(cell, t.u16(s, 0), t.u16(s, 2));
                        s += 4;
                    }
                    TileMode::PairsMerge(keep) => {
                        self.prop_tile_code(cell, t.u16(s, 0));
                        self.prop_tile_attr(cell, keep, t.u16(s, 2));
                        s += 4;
                    }
                    TileMode::Codes => {
                        self.prop_tile_code(cell, t.u16(s, 0));
                        s += 2;
                    }
                }
            }
        }
    }

    /// The same rectangle for the loops that only change the attribute word of every cell (no source table).
    pub fn prop_tile_attr_block(&mut self, at: TilePos, cols: u16, rows: u16, keep: u16, or: u16) {
        for c in 0..cols as i32 {
            for r in 0..rows as i32 {
                self.prop_tile_attr(at.right(c).down(r), keep, or);
            }
        }
    }
}

/// Offsets of the block table driver (the original's tile animations keep them in +$A0 and +$A2).
mod f {
    /// byte: frames until the next block.
    pub const TIMER: usize = 0xA0;
    /// word: the byte offset of the next entry of the pointer table (4 a block, as the original counts).
    pub const INDEX: usize = 0xA2;
}

/// A table of blocks played one after the other into one place: the tile animations (water, doors opening, lamps) are
/// this and a condition. `table` is the ROM address of `count` longs, each the ROM address of one block (cells in
/// the order of [`Game::prop_tile_block`]). The blocks must be in the extracted tables (`dino-extract/src/enemy.rs`).
#[derive(Clone, Copy, Debug)]
pub struct BlockTable {
    pub table: u32,
    pub count: u16,
    /// Where the blocks go, and their size.
    pub at: TilePos,
    pub cols: u16,
    pub rows: u16,
    pub mode: TileMode,
    /// Frames from one block to the next.
    pub period: u8,
    /// After the last block start again with the first one (`step` then never reports done).
    pub looped: bool,
}

impl BlockTable {
    /// Mode 0 of the prop: the first block is written when the counter runs out, `delay` updates from now (1 = in
    /// the next update, as the original sets +$A0 = 1).
    pub fn start(&self, p: &mut Prop, delay: u8) {
        p.m.sb(f::TIMER, delay);
        p.m.sw(f::INDEX, 0);
    }
}

impl Game {
    /// One update of a block table (the shape of 0x78B86: `subq.b #1,$A0 ; bne rts`, write the block of the index,
    /// `+$A0 = period`, index + 4, done at the end of the table). Returns true in the update that wrote the last
    /// block (a `looped` table wraps to the first block and never reports done).
    pub fn prop_block_table_step(&mut self, p: &mut Prop, t: &BlockTable) -> bool {
        let timer = p.m.b(f::TIMER).wrapping_sub(1);
        p.m.sb(f::TIMER, timer);
        if timer != 0 {
            return false;
        }
        let index = p.m.w(f::INDEX);
        let block = self.assets.enemy.t.u32(t.table, index as u32) & 0xFF_FFFF;
        self.prop_tile_block(t.at, t.cols, t.rows, block, t.mode);
        p.m.sb(f::TIMER, t.period);
        let next = index.wrapping_add(4);
        let done = next == 4 * t.count;
        p.m.sw(f::INDEX, if done && t.looped { 0 } else { next });
        done && !t.looped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::enemy::ByteTables;
    use dino_assets::AssetPack;

    /// A game whose ROM byte tables are the given blobs (synthetic data: no ROM bytes).
    fn game(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs };
        Game::new(Box::leak(Box::new(a)))
    }

    fn words(w: &[u16]) -> Vec<u8> {
        w.iter().flat_map(|v| v.to_be_bytes()).collect()
    }

    fn cell(g: &Game, a: u32) -> (u16, u16) {
        let c = match Layer::of(a).unwrap() {
            Layer::Scroll1 => g.vram.scroll1[((a - 0x908000) / 4) as usize],
            Layer::Scroll2 => g.vram.scroll2[((a - 0x90C000) / 4) as usize],
            Layer::Scroll3 => g.vram.scroll3[((a - 0x910000) / 4) as usize],
        };
        (c.code, c.attr)
    }

    /// The ring coordinates and the video addresses agree in all three layers, with the addresses the original
    /// uses: the door smash (0x90E9E4) and the hall door (0x90F288) of scroll2, the profile board strip (0x9137E0).
    #[test]
    fn coordinates_and_addresses_agree() {
        assert_eq!(TilePos::new(Layer::Scroll2, 39, 41), TilePos(0x90E9E4));
        assert_eq!(TilePos::new(Layer::Scroll2, 10, 50), TilePos(0x90F288));
        assert_eq!(TilePos::new(Layer::Scroll3, 63, 48), TilePos(0x9137E0));
        assert_eq!(TilePos::new(Layer::Scroll1, 5, 40), TilePos(0x908000 + 0x80 * 5 + 4 * 8 + 0x2000));
        for layer in [Layer::Scroll1, Layer::Scroll2, Layer::Scroll3] {
            for (col, row) in [(0, 0), (1, 31), (17, 33), (63, 63), (40, 8), (2, 17)] {
                assert_eq!(TilePos::new(layer, col, row).layer_col_row(), Some((layer, col, row)));
            }
        }
        // The ring: column and row wrap at 64.
        assert_eq!(TilePos::new(Layer::Scroll2, 64 + 39, 64 + 41), TilePos(0x90E9E4));
        assert_eq!(TilePos::at(0x90EA66), TilePos(0x90EA64), "the attribute word belongs to its cell");
        assert_eq!(TilePos(0x914000).layer_col_row(), None);
    }

    /// 0x75208: the door smash X 0x670 is column 39 of scroll2; the row counts down from the top of the map.
    #[test]
    fn the_world_position_helper_of_class_0() {
        assert_eq!(TilePos::scroll2_at_world(0x670, 0x3F0), TilePos::new(Layer::Scroll2, 39, 0));
        assert_eq!(TilePos::scroll2_at_world(0x670 + 0x400, 0x3F0 - 41 * 16), TilePos::new(Layer::Scroll2, 39, 41));
        assert_eq!(TilePos::scroll2_at_world(0x10, 0), TilePos::new(Layer::Scroll2, 1, 63));
    }

    /// The three word writers: both words, the code only, and the attribute bits (`and` then `or`).
    #[test]
    fn single_cell_writers() {
        let mut g = game(vec![]);
        let at = TilePos::at(0x90E9E4);
        g.prop_tile_put(at, 0x1234, 0x00A5);
        assert_eq!(cell(&g, 0x90E9E4), (0x1234, 0x00A5));
        g.prop_tile_code(at, 0x4321);
        assert_eq!(cell(&g, 0x90E9E4), (0x4321, 0x00A5));
        g.prop_tile_attr(at, 0xFFC0, 0x0012); // keep the high bits, replace the palette and flips
        assert_eq!(cell(&g, 0x90E9E4), (0x4321, 0x0092));
        g.prop_tile_attr(at, 0xFE7F, 0x0100); // class 11 (0x78AF0): bits 7 and 8 off, bit 8 on
        assert_eq!(cell(&g, 0x90E9E4), (0x4321, 0x0112));
        g.prop_tile_put(TilePos(0x914000), 1, 1); // outside the maps: ignored
    }

    /// A block is written column by column, each column top to bottom, in the three modes. Class 11 shape: 3 columns
    /// of 7 codes; class 12 shape: code and attribute pairs with the attribute merged.
    #[test]
    fn blocks_follow_the_original_loops() {
        let codes: Vec<u16> = (0..21).map(|k| 0x100 + k).collect();
        let pairs: Vec<u16> = (0..8).flat_map(|k| [0x200 + k, 0x0003 + 0x10 * k]).collect();
        let mut g = game(vec![(0x78A8E, words(&codes)), (0x78BE6, words(&pairs))]);
        let at = TilePos::at(0x90E9E4);
        g.prop_tile_put(at, 0xFFFF, 0x0777);
        g.prop_tile_block(at, 3, 7, 0x78A8E, TileMode::Codes);
        assert_eq!(cell(&g, 0x90E9E4), (0x100, 0x0777), "the attribute stays");
        assert_eq!(cell(&g, 0x90E9E4 + 4 * 6), (0x106, 0));
        assert_eq!(cell(&g, 0x90E9E4 + 0x40), (0x107, 0), "the next column starts at the top");
        assert_eq!(cell(&g, 0x90E9E4 + 0x80 + 24), (0x114, 0));
        // 2 x 4 pairs, replace both words.
        g.prop_tile_block(TilePos::at(0x90F288), 2, 4, 0x78BE6, TileMode::Pairs);
        assert_eq!(cell(&g, 0x90F288), (0x200, 0x0003));
        assert_eq!(cell(&g, 0x90F288 + 0x40 + 12), (0x207, 0x0003 + 0x70));
        // Merge: the old high bits stay, the low six bits are the source (0x78BC4: keep 0xFFC0).
        g.prop_tile_put(TilePos::at(0x90F288), 0, 0x01C5);
        g.prop_tile_block(TilePos::at(0x90F288), 2, 4, 0x78BE6, TileMode::PairsMerge(0xFFC0));
        assert_eq!(cell(&g, 0x90F288), (0x200, 0x01C0 | 0x0003));
        // Attribute only, over a rectangle (0x78AB8: 7 cells across, 0x40 apart; an attribute address rounds to its cell).
        for k in 0..7 {
            g.prop_tile_put(TilePos::at(0x90E9BC).right(k), 0, 0xFFFF);
        }
        g.prop_tile_attr_block(TilePos::at(0x90E9BE), 7, 1, 0x03FF, 0);
        assert!((0..7).all(|k| cell(&g, 0x90E9BC + 0x40 * k) == (0, 0x03FF)));
        assert_eq!(cell(&g, 0x90E9BC + 0x40 * 7).1, 0, "the 8th cell is not touched");
    }

    /// A tall block runs on over the end of a page as the 68000 address does: row 15 of a scroll2 page is followed
    /// by row 0 of the next column, not by row 16.
    #[test]
    fn a_block_runs_on_over_the_page_end() {
        let mut g = game(vec![(0x1000, words(&[1, 2, 3]))]);
        let at = TilePos::new(Layer::Scroll2, 5, 14);
        g.prop_tile_block(at, 1, 3, 0x1000, TileMode::Codes);
        assert_eq!(cell(&g, at.0), (1, 0));
        assert_eq!(cell(&g, TilePos::new(Layer::Scroll2, 5, 15).0), (2, 0));
        assert_eq!(cell(&g, TilePos::new(Layer::Scroll2, 6, 0).0), (3, 0), "the third cell fell into the next column");
        assert_eq!(cell(&g, TilePos::new(Layer::Scroll2, 5, 16).0), (0, 0));
    }

    /// The block table driver (class 12 var 0): three blocks, one every 4 frames, the first on the next update, done
    /// in the update of the last block (0x78B86: +$A0 = 4, +$A2 + 4, done at 0xC).
    #[test]
    fn the_block_table_plays_one_block_every_period() {
        let table: Vec<u8> = [0x2000u32, 0x2010, 0x2020].iter().flat_map(|v| v.to_be_bytes()).collect();
        let mut blobs = vec![(0x1000, table)];
        for (k, a) in [0x2000u32, 0x2010, 0x2020].into_iter().enumerate() {
            blobs.push((a, words(&[0x300 + k as u16, 1, 0x310 + k as u16, 2])));
        }
        let mut g = game(blobs);
        let t = BlockTable {
            table: 0x1000,
            count: 3,
            at: TilePos::at(0x90F288),
            cols: 1,
            rows: 2,
            mode: TileMode::Pairs,
            period: 4,
            looped: false,
        };
        let mut p = Prop::default();
        t.start(&mut p, 1);
        let mut log = Vec::new();
        for frame in 1..=9 {
            let done = g.prop_block_table_step(&mut p, &t);
            log.push((frame, cell(&g, 0x90F288).0, done));
        }
        let changes: Vec<(i32, u16, bool)> = log.windows(2).filter(|w| w[0].1 != w[1].1).map(|w| w[1]).collect();
        assert_eq!(log[0], (1, 0x300, false), "the first block at once");
        assert_eq!(changes, vec![(5, 0x301, false), (9, 0x302, true)], "one block every 4 updates, done with the last");
        assert_eq!(cell(&g, 0x90F288 + 4), (0x310 + 2, 2));
        assert_eq!(p.m.w(f::INDEX), 12);
        // A looped table wraps and never reports done.
        let mut again = BlockTable { looped: true, ..t };
        again.period = 1;
        let mut p = Prop::default();
        again.start(&mut p, 1);
        let seen: Vec<u16> = (0..7)
            .map(|_| {
                assert!(!g.prop_block_table_step(&mut p, &again));
                cell(&g, 0x90F288).0
            })
            .collect();
        assert_eq!(seen, vec![0x300, 0x301, 0x302, 0x300, 0x301, 0x302, 0x300]);
    }
}
