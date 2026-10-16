//! Pool B class 87 (`+$20 = 0x15C`, handler 0x8B60E): the staff roll of the ending (M3-24 / M3-25). Made by the master 82 in the still
//! picture scene (record 771); it prints a line of the staff text (12 letters, ROM 0x8B7EC) on scroll1 every 32 frames, moves scroll1 up by
//! 0x6C00 / 0x10000 pixels a frame (G.$6C8 as a long) and lives 0xC00 frames, after a delay of 0x78. The red header is the attribute 2 of
//! the table 0x8BC84, the other lines attribute 1.
//!
//! | mode | what |
//! | --- | --- |
//! | 0 | 0x8B622: mode 2, `+$A0` = 0xC00, `+$B8` = 0x78, the pointers (`+$AA` the scroll1 cell, `+$AE` the line, `+$B2` the attribute, `+$B6` the row 0x1E), the first line (STAFF, attribute 1), then the frame of mode 2 |
//! | 2 | 0x8B6A0: the delay `+$B8`, then `+$A0` counts down; at 0 G.$6C8 = 0 and mode 4; else Y -= 0x6C00 and every 32 frames the next line two rows lower |
//! | 4, 6 | 0x8B7BA: free |

use super::Prop;
use crate::game::Game;

mod f {
    pub const TIMER: usize = 0xA0;
    pub const CELL: usize = 0xAA;
    pub const LINE: usize = 0xAE;
    pub const ATTR: usize = 0xB2;
    pub const ROW: usize = 0xB6;
    pub const DELAY: usize = 0xB8;
}

/// The letter tiles (words, A = 0x41), the lines and the attributes of the roll.
const TILES: u32 = 0x8B786;
const LINES: u32 = 0x8B7EC;
const ATTRS: u32 = 0x8BC84;

impl Game {
    /// 0x8B60E pool B class 87 (jump table 0x8B61A: mode 0 -> 0x8B622, 2 -> 0x8B6A0, 4 and 6 -> 0x8B7BA).
    pub fn prop_b87(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode = 2;
                p.m.sw(f::TIMER, 0xC00);
                p.m.sw(f::DELAY, 0x78);
                for o in [0xA2, 0xA4, f::CELL, f::LINE, f::ATTR] {
                    p.m.sl(o, 0);
                }
                p.m.sw(0xA8, 0);
                p.m.sw(f::ROW, 0x1E);
                p.m.sl(f::ATTR, ATTRS);
                let a0 = Self::b87_cell(0x24, 0x1E);
                p.m.sl(f::CELL, a0);
                p.m.sl(f::LINE, LINES);
                self.b87_print(a0, LINES, 1);
                self.b87_frame(p, i);
            }
            2 => self.b87_frame(p, i),
            _ => self.prop_free(p, i), // 0x8B7BA: jmp 0x4938
        }
    }

    /// 0x8B7C0: the address of the scroll1 cell of column `col` and row `row` (the rows 0 to 31 are the second half of the column).
    fn b87_cell(col: u16, row: u16) -> u32 {
        let c = (col & 0x3F) << 7;
        let mut r = row & 0x3F;
        r = if r & 0x20 != 0 { (r - 0x20) << 2 } else { (r << 2) + 0x2000 };
        0x908000 + c.wrapping_add(r) as u32
    }

    /// 0x8B6A0.
    fn b87_frame(&mut self, p: &mut Prop, i: usize) {
        if p.m.w(f::DELAY) != 0 {
            let d = p.m.w(f::DELAY) - 1;
            p.m.sw(f::DELAY, d);
            if d != 0 {
                return; // 0x8B71C
            }
        }
        let t = p.m.w(f::TIMER).wrapping_sub(1);
        p.m.sw(f::TIMER, t);
        if t == 0 {
            // 0x8B71E: G.$6C8 = 0 (the long), mode 4 and the free at once
            self.cam.cam1_y = 0;
            self.cam.cam1_y_frac = 0;
            p.b.mode += 2;
            return self.prop_free(p, i);
        }
        let y = (((self.cam.cam1_y as u32) << 16) | self.cam.cam1_y_frac as u32).wrapping_sub(0x6C00);
        (self.cam.cam1_y, self.cam.cam1_y_frac) = ((y >> 16) as u16, y as u16);
        if t & 0x1F != 0 {
            return;
        }
        let mut cell = p.m.l(f::CELL).wrapping_add(8);
        if cell & 0x7F == 0 {
            cell = cell.wrapping_add(0x4000);
        }
        p.m.sl(f::CELL, cell);
        let line = p.m.l(f::LINE).wrapping_add(0xC);
        p.m.sl(f::LINE, line);
        let attr_at = p.m.l(f::ATTR).wrapping_add(2);
        p.m.sl(f::ATTR, attr_at);
        let row = p.m.w(f::ROW).wrapping_add(2);
        p.m.sw(f::ROW, row);
        let a0 = Self::b87_cell(0x24, row);
        let attr = self.assets.frontend.text.u16(attr_at, 0);
        self.b87_print(a0, line, attr);
    }

    /// 0x8B730: a line of letters along the columns from the cell `a0`: a letter is the tile of the table 0x8B786, other bytes (the space)
    /// are tile codes of their own, '%' prints an A with the attribute bit 0x40, '@' ends the line. The attribute is `attr & 0x1F`.
    fn b87_print(&mut self, mut a0: u32, line: u32, attr: u16) {
        let mut a1 = line;
        loop {
            let c = self.assets.frontend.text.u8(a1, 0) as u16;
            a1 += 1;
            let mut d2 = attr & 0x1F;
            let mut d0 = c;
            if d0 == 0x25 {
                d2 |= 0x40;
                d0 = 0x41;
            }
            if d0 == 0x40 {
                return;
            }
            if d0 == 0x2F {
                continue; // 0x8B75A: a jump through VRAM that no line of the ROM uses
            }
            if d0 > 0x40 {
                d0 = self.assets.frontend.text.u16(TILES, 2 * (d0 - 0x41) as u32);
            }
            self.hud_put(a0, d0, d2);
            a0 = a0.wrapping_add(0x80);
        }
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

    /// 0x8B7C0: column 0x24, row 0x1E is the cell 0x90B278 (the first cell of the staff roll in the recording); rows 32 and up are in the first half of the column.
    #[test]
    fn the_cell_of_a_row_in_the_second_half_of_the_column() {
        assert_eq!(Game::b87_cell(0x24, 0x1E), 0x90B278);
        assert_eq!(Game::b87_cell(0x24, 0x20), 0x909200);
        assert_eq!(Game::b87_cell(0x24, 0x22), 0x909208);
    }

    /// The first line is printed at once (STAFF, the header attribute 1 of the first call), then the roll waits 120 frames; scroll1 Y moves 0x6C00 a frame
    /// after that, and a line (two rows down, 12 letters along the columns) is printed every 32 frames; after 0xC00 frames Y is 0 again and the object ends.
    #[test]
    fn the_staff_roll_prints_a_line_every_32_frames_and_scrolls() {
        let Some(a) = assets() else { return };
        let mut g = Game::new(a);
        let i = g.prop_spawn(87, 0).unwrap();
        g.pool_b_update();
        let t = &g.assets.frontend.text;
        let s_tile = t.u16(0x8B786, 2 * (b'S' - b'A') as u32);
        let cell = |g: &Game, a: u32| {
            let c = &g.vram.scroll1[Game::hud_cell(a)];
            (c.code, c.attr)
        };
        assert_eq!(cell(&g, 0x90B278), (s_tile, 1));
        assert_eq!((g.cam.cam1_y, g.cam.cam1_y_frac), (0, 0));
        for _ in 0..0x77 {
            g.pool_b_update(); // the delay of 0x78 frames: +$B8 counts down from the first pass, scrolling starts in the 0x78th
        }
        assert_eq!((g.cam.cam1_y, g.cam.cam1_y_frac), (0xFFFF, 0x9400), "0 - 0x6C00");
        let _ = (); // (the 0x78th pass is the one above: the loop ran 0x77 more after the first)
        // every 32 frames (`+$A0` & 0x1F = 0) the next line, two rows lower
        let row2 = Game::b87_cell(0x24, 0x20);
        let first_code = |g: &Game| g.vram.scroll1[Game::hud_cell(row2)].code;
        let before = first_code(&g);
        let line0 = g.props.slots[i].m.l(0xAE);
        for _ in 0..40 {
            g.pool_b_update();
        }
        assert_eq!(g.props.slots[i].m.l(0xAE), line0 + 0xC, "the pointer to the line moved once");
        assert_ne!(first_code(&g), before, "the second line (a blank one) is printed two rows down");
    }
}
