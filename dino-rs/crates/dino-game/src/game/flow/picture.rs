//! Picture loaders and block printers of the front end scenes.
//!
//! The pictures of the title, select, continue, story and ending scenes are plain data: a page is
//! a list of block ids and every block is a small block of cells (`dino_assets::frontend`). The
//! loaders copy a page into the tilemap at an absolute VRAM address the way the original does
//! (scroll1 base 0x908000, scroll2 0x90C000, scroll3 0x910000; a cell is a code word and an
//! attribute word; the offsets are listed at `hud.rs` and `vram.rs`).
//!
//! | routine | what | here |
//! | --- | --- | --- |
//! | 0x20454 | scroll1 page: 8 x 8 blocks of 4 x 4 cells (32 x 32 cells) | [`Game::flow_page1`] |
//! | 0x204CA | scroll2 page: 8 x 8 blocks of 2 x 2 cells (16 x 16 cells) | [`Game::flow_page2`] |
//! | 0x20592 | scroll3 page: 8 x 8 cells | [`Game::flow_page3`] |
//! | 0x98B86 / 0x98BAE | scroll2 / scroll3 block from a code list and an attribute list | [`Game::flow_block2`], [`Game::flow_block3`] |
//! | 0x89E6 | the palettes of a scene set | [`Game::flow_scene_palettes`] |

// The services are used by the scenes of the next tasks (T1.2 to T1.8); until they are, parts of
// them are unused.
#![allow(dead_code)]

use crate::game::Game;
use crate::game::vram::PaletteRam;

/// Scroll2 and scroll3 VRAM base addresses (scroll1 is `hud.rs`' 0x908000).
const S2: u32 = 0x90C000;
const S3: u32 = 0x910000;

/// Big-endian word `i` of a list in the ROM tables.
fn be(s: &[u8], i: usize) -> u16 {
    u16::from_be_bytes([s[2 * i], s[2 * i + 1]])
}

impl Game {
    /// Cell index of a scroll2 / scroll3 VRAM address (the address wraps inside the 0x4000 bytes).
    fn pic_cell2(a: u32) -> usize {
        ((a.wrapping_sub(S2) / 4) & 0xFFF) as usize
    }
    fn pic_cell3(a: u32) -> usize {
        ((a.wrapping_sub(S3) / 4) & 0xFFF) as usize
    }

    /// 0x20454: copies scroll1 page `page` (the d5 of the original) to `a0`: 8 columns of 8
    /// blocks, each block 4 columns of 4 cells (the block list is column by column, top block
    /// first). The page covers 32 columns and 32 rows.
    pub fn flow_page1(&mut self, a0: u32, page: usize) {
        let f = &self.assets.frontend;
        let ids = f.scroll1_page(page);
        let (mut a4, mut a) = (a0, a0);
        let mut k = 0;
        for _col in 0..8 {
            for _row in 0..8 {
                let block = &f.scroll1_blocks[ids[k] as usize];
                k += 1;
                // 0x2047C: four runs of 4 cells down, the next run one column right
                for (i, c) in block.iter().enumerate() {
                    let at = a + (i as u32 / 4) * 0x80 + (i as u32 % 4) * 4;
                    self.hud_put(at, c.code, c.attr);
                }
                a += 0x10; // 0x190 after the block, -0x180 at 0x20472: four rows down
            }
            a4 += 0x200; // four columns right
            a = a4;
        }
    }

    /// 0x204CA: copies scroll2 scene page `page` to `a0`: 8 columns of 8 blocks of 2 x 2 cells
    /// (16 columns, 16 rows).
    pub fn flow_page2(&mut self, a0: u32, page: usize) {
        let f = &self.assets.frontend;
        let ids = f.scroll2_page(page);
        let (mut a4, mut a) = (a0, a0);
        let mut k = 0;
        for _col in 0..8 {
            for _row in 0..8 {
                let block = &f.scroll2_blocks[ids[k] as usize];
                k += 1;
                // 0x204F2: two cells down, then the next column (+0x40) with two more
                for (i, c) in block.iter().enumerate() {
                    let at = a + (i as u32 / 2) * 0x40 + (i as u32 % 2) * 4;
                    let cell = &mut self.vram.scroll2[Self::pic_cell2(at)];
                    cell.code = c.code;
                    cell.attr = c.attr;
                }
                a += 8; // two rows down
            }
            a4 += 0x80; // two columns right
            a = a4;
        }
    }

    /// 0x20592: copies scroll3 page `page` (a screen of `StageData::scroll3_screen_cells`, shared
    /// with the stages) to `a0`: 8 columns of 8 cells.
    pub fn flow_page3(&mut self, a0: u32, page: usize) {
        for col in 0..8usize {
            for row in 0..8usize {
                let c = self.assets.stages.scroll3_screen_cells[page * 64 + col * 8 + row];
                let at = a0 + col as u32 * 0x20 + row as u32 * 4;
                let cell = &mut self.vram.scroll3[Self::pic_cell3(at)];
                cell.code = c.code;
                cell.attr = c.attr;
            }
        }
    }

    /// 0x98B86: a scroll2 block at `a0`: `width` cells (d1 + 1) per row, `height` rows (d2 + 1);
    /// the code list (words, 0x5800 is added) and the attribute list (words) are read row by row.
    pub fn flow_block2(&mut self, a0: u32, codes: &[u8], attrs: &[u8], width: u16, height: u16) {
        let mut k = 0;
        for row in 0..height as u32 {
            for col in 0..width as u32 {
                let at = a0 + col * 0x40 + row * 4;
                let cell = &mut self.vram.scroll2[Self::pic_cell2(at)];
                cell.code = be(codes, k).wrapping_add(0x5800);
                cell.attr = be(attrs, k);
                k += 1;
            }
        }
    }

    /// 0x98BAE: the same for scroll3 (0x1C00 is added to the codes, a column is +0x20).
    pub fn flow_block3(&mut self, a0: u32, codes: &[u8], attrs: &[u8], width: u16, height: u16) {
        let mut k = 0;
        for row in 0..height as u32 {
            for col in 0..width as u32 {
                let at = a0 + col * 0x20 + row * 4;
                let cell = &mut self.vram.scroll3[Self::pic_cell3(at)];
                cell.code = be(codes, k).wrapping_add(0x1C00);
                cell.attr = be(attrs, k);
                k += 1;
            }
        }
    }

    /// 0x89E6: loads the palettes of scene set `set` (0..5): the OBJ palettes, the fixed scroll1
    /// palettes, the scroll2 and scroll3 palettes and the priority masks. Brightness stays 0 as
    /// in the ROM; the caller brings it up (`brightness_full`, the fade tasks).
    pub fn flow_scene_palettes(&mut self, set: usize) {
        let f = &self.assets.frontend;
        let p = &f.scene_palettes[set];
        self.pal.copy(PaletteRam::OBJ, &p.obj);
        self.pal.copy(PaletteRam::SCROLL1, &self.assets.stages.scroll1_palettes);
        self.pal.copy(PaletteRam::SCROLL2, &p.scroll2);
        self.pal.copy(PaletteRam::SCROLL3, &p.scroll3);
        self.w.priority_masks = p.priority_masks;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use dino_assets::frontend::{FrontendData, ScenePalettes};
    use dino_assets::stage::Cell;

    fn cell(code: u16, attr: u16) -> Cell {
        Cell { code, attr }
    }

    fn game(frontend: FrontendData, scroll3: Vec<Cell>) -> Game {
        let mut a = AssetPack::default();
        a.frontend = frontend;
        a.stages.scroll3_screen_cells = scroll3;
        Game::new(Box::leak(Box::new(a)))
    }

    fn s1(g: &Game, a: u32) -> (u16, u16) {
        let c = g.vram.scroll1[((a - 0x908000) / 4) as usize & 0xFFF];
        (c.code, c.attr)
    }
    fn s2(g: &Game, a: u32) -> (u16, u16) {
        let c = g.vram.scroll2[((a - 0x90C000) / 4) as usize & 0xFFF];
        (c.code, c.attr)
    }
    fn s3(g: &Game, a: u32) -> (u16, u16) {
        let c = g.vram.scroll3[((a - 0x910000) / 4) as usize & 0xFFF];
        (c.code, c.attr)
    }

    /// Scroll1 page: block list order is column of blocks by row of blocks; a block is 4 runs of
    /// 4 cells down, left run first. Block 1 (cells 0x100 + i) is the second block of the page.
    #[test]
    fn scroll1_page_layout() {
        let blocks: Vec<[Cell; 16]> = (0..3u16)
            .map(|b| std::array::from_fn(|i| cell(0x100 * b + i as u16, 0x20 + b)))
            .collect();
        let mut ids = [0u16; 64];
        ids[1] = 1; // block column 0, block row 1
        ids[8] = 2; // block column 1, block row 0
        let f = FrontendData { scroll1_pages: vec![ids], scroll1_blocks: blocks, ..Default::default() };
        let mut g = game(f, vec![]);
        let a0 = 0x90A000;
        g.flow_page1(a0, 0);
        // block 1 starts four rows down in the first block column
        let b1 = a0 + 4 * 4;
        assert_eq!(s1(&g, b1), (0x100, 0x21));
        assert_eq!(s1(&g, b1 + 4), (0x101, 0x21)); // 4 cells go down
        assert_eq!(s1(&g, b1 + 0x80), (0x104, 0x21)); // the next run is the next column
        assert_eq!(s1(&g, b1 + 3 * 0x80 + 3 * 4), (0x10F, 0x21));
        // block 2 starts four columns right
        assert_eq!(s1(&g, a0 + 4 * 0x80), (0x200, 0x22));
        assert_eq!(s1(&g, a0 + 7 * 0x80 + 3 * 4), (0x20F, 0x22));
        // the last block (id 0) fills the corner: 32 columns, 32 rows
        assert_eq!(s1(&g, a0 + 31 * 0x80 + 31 * 4), (0x00F, 0x20));
    }

    /// Scroll2 page: blocks of 2 x 2 cells, two rows down for the next block of the column, two
    /// columns right for the next block column; the page is 16 x 16 cells.
    #[test]
    fn scroll2_page_layout() {
        let blocks: Vec<[Cell; 4]> = (0..3u16).map(|b| std::array::from_fn(|i| cell(0x10 * b + i as u16, b))).collect();
        let mut ids = [0u16; 64];
        ids[1] = 1;
        ids[8] = 2;
        let f = FrontendData { scroll2_pages: vec![ids], scroll2_blocks: blocks, ..Default::default() };
        let mut g = game(f, vec![]);
        let a0 = 0x90F000;
        g.flow_page2(a0, 0);
        assert_eq!(s2(&g, a0 + 2 * 4), (0x10, 1)); // second block of column 0: two rows down
        assert_eq!(s2(&g, a0 + 2 * 4 + 4), (0x11, 1));
        assert_eq!(s2(&g, a0 + 2 * 4 + 0x40), (0x12, 1)); // the next column (+0x40)
        assert_eq!(s2(&g, a0 + 0x80), (0x20, 2)); // block column 1: two columns right
        assert_eq!(s2(&g, a0 + 15 * 0x40 + 15 * 4), (0x03, 0));
    }

    /// Scroll3 page: 8 columns of 8 cells in the order of the screen table.
    #[test]
    fn scroll3_page_layout() {
        let cells: Vec<Cell> = (0..128u16).map(|i| cell(0x1C00 + i, i & 0x1F)).collect();
        let mut g = game(FrontendData::default(), cells);
        g.flow_page3(0x913800, 1); // page 1 = cells 64..127
        assert_eq!(s3(&g, 0x913800), (0x1C00 + 64, 0));
        assert_eq!(s3(&g, 0x913800 + 4), (0x1C00 + 65, 1));
        assert_eq!(s3(&g, 0x913800 + 0x20), (0x1C00 + 72, 8)); // next column
        assert_eq!(s3(&g, 0x913800 + 7 * 0x20 + 7 * 4), (0x1C00 + 127, 31));
    }

    /// 0x98B86 and 0x98BAE: the codes get the layer's base added, the lists are read row by row.
    #[test]
    fn block_printers() {
        let list = |v: &[u16]| v.iter().flat_map(|w| w.to_be_bytes()).collect::<Vec<u8>>();
        let mut g = game(FrontendData::default(), vec![]);
        g.flow_block2(0x90F000, &list(&[1, 2, 3, 4, 5, 6]), &list(&[0x10, 0x11, 0x12, 0x13, 0x14, 0x15]), 3, 2);
        assert_eq!(s2(&g, 0x90F000), (0x5801, 0x10));
        assert_eq!(s2(&g, 0x90F000 + 0x40), (0x5802, 0x11)); // along the row: next column
        assert_eq!(s2(&g, 0x90F000 + 2 * 0x40), (0x5803, 0x12));
        assert_eq!(s2(&g, 0x90F000 + 4), (0x5804, 0x13)); // next row
        g.flow_block3(0x913828, &list(&[1, 2, 3, 4]), &list(&[9, 8, 7, 6]), 2, 2);
        assert_eq!(s3(&g, 0x913828), (0x1C01, 9));
        assert_eq!(s3(&g, 0x913828 + 0x20), (0x1C02, 8));
        assert_eq!(s3(&g, 0x913828 + 4), (0x1C03, 7));
        assert_eq!(s3(&g, 0x913828 + 0x24), (0x1C04, 6));
    }

    #[test]
    fn scene_palette_set_fills_the_banks() {
        let p = ScenePalettes { obj: vec![1; 512], scroll2: vec![2; 512], scroll3: vec![3; 512], priority_masks: [4, 5, 6, 7] };
        let f = FrontendData { scene_palettes: vec![p], ..Default::default() };
        let mut g = game(f, vec![]);
        g.flow_scene_palettes(0);
        assert_eq!([g.pal.words[0], g.pal.words[512], g.pal.words[1024], g.pal.words[1536]], [1, 0, 2, 3]);
        assert_eq!(g.w.priority_masks, [4, 5, 6, 7]);
    }
}
