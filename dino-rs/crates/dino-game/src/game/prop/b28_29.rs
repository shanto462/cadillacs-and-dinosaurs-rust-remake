//! Pool B classes 28 and 29: Episode 8 sewer water bands (0x7F72A and 0x8048A).
//!
//! Class 28 updates rows 12 to 14 of scroll2 in 5 variants of differing widths.
//! Class 29 updates a 16-column by 3-row block (rows 12 to 14) in 7 variants across the screen.
//! Both are driven by the tile animation clock G.$7AE / G.$7AF.

use super::tile_block::TilePos;
use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x7F72A pool B class 28: sewer water bands (rows 12 to 14).
    pub fn prop_b28(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x7F73E: mode 0 -> mode 2
                p.b.mode = 2;
            }
            2 => {
                // 0x7F744: camera cull test
                if p.b.xi().wrapping_add(0x100) < self.cam.s2.cam_x as i16 {
                    // 0x7F76A: behind camera, advance to mode 4 to free next frame
                    p.b.mode += 2;
                    return;
                }
                // 0x7F752: check tile animation clock
                let (phase72, prev73) = self.prop_tile_phase();
                if phase72 == prev73 {
                    return; // 0x7F768
                }
                // 0x7F75C: dispatch variant
                match p.b.var26 {
                    0 => self.prop_b28_var0(phase72),
                    2 => self.prop_b28_var2(phase72),
                    4 => self.prop_b28_var4(phase72),
                    6 => self.prop_b28_var6(phase72),
                    8 => self.prop_b28_var8(phase72),
                    _ => self.prop_free(p, i),
                }
            }
            _ => self.prop_free(p, i), // 0x80484: jmp 0x4938
        }
    }

    /// 0x7F77A: B28 variant 0 (42 tile codes).
    fn prop_b28_var0(&mut self, phase: u8) {
        let p = self.assets.enemy.t.u32(0x7F7C8, (phase & 0xC) as u32);
        let mut a1 = p;
        // 0x7F796: 12 words from 0x90C130
        let mut a0 = 0x90C130;
        for _ in 0..12 {
            self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
            a1 += 2;
            a0 += 0x40;
        }
        // 0x7F7AA: 14 words from 0x90C0B4
        let mut a0 = 0x90C0B4;
        for _ in 0..14 {
            self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
            a1 += 2;
            a0 += 0x40;
        }
        // 0x7F7BA: 16 words from 0x90C038
        let mut a0 = 0x90C038;
        for _ in 0..16 {
            self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
            a1 += 2;
            a0 += 0x40;
        }
    }

    /// 0x7F928: B28 variant 2 (80 tile codes across 7 rows).
    fn prop_b28_var2(&mut self, phase: u8) {
        let p = self.assets.enemy.t.u32(0x7F9BE, (phase & 0xC) as u32);
        let mut a1 = p;
        // 0x7F944: 5 words from 0x90C6E0
        let mut a0 = 0x90C6E0;
        for _ in 0..5 {
            self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
            a1 += 2;
            a0 += 0x40;
        }
        // 0x7F958: 7 words from 0x90C664
        let mut a0 = 0x90C664;
        for _ in 0..7 {
            self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
            a1 += 2;
            a0 += 0x40;
        }
        // 0x7F96C: 9 words from 0x90C5E8
        let mut a0 = 0x90C5E8;
        for _ in 0..9 {
            self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
            a1 += 2;
            a0 += 0x40;
        }
        // 0x7F980: 11 words from 0x90C56C
        let mut a0 = 0x90C56C;
        for _ in 0..11 {
            self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
            a1 += 2;
            a0 += 0x40;
        }
        // 0x7F990, 0x7F9A0, 0x7F9B0: 3 rows of 16 words from 0x90C430, 0x90C434, 0x90C438
        let mut a2 = 0x90C430;
        for _ in 0..3 {
            let mut a0 = a2;
            for _ in 0..16 {
                self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
                a1 += 2;
                a0 += 0x40;
            }
            a2 += 4;
        }
    }

    /// 0x7FC6E: B28 variant 4 (103 tile codes across 7 rows).
    fn prop_b28_var4(&mut self, phase: u8) {
        let p = self.assets.enemy.t.u32(0x7FCF4, (phase & 0xC) as u32);
        let mut a1 = p;
        // 0x7FC86: 11 words from 0x90C820
        let mut a0 = 0x90C820;
        for _ in 0..11 {
            self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
            a1 += 2;
            a0 += 0x40;
        }
        // 0x7FC96: 13 words from 0x90C824
        let mut a0 = 0x90C824;
        for _ in 0..13 {
            self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
            a1 += 2;
            a0 += 0x40;
        }
        // 0x7FCA6: 15 words from 0x90C828
        let mut a0 = 0x90C828;
        for _ in 0..15 {
            self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
            a1 += 2;
            a0 += 0x40;
        }
        // 0x7FCB6, 0x7FCC6, 0x7FCD6, 0x7FCE6: 4 rows of 16 words from 0x90C82C..0x90C838
        let mut a2 = 0x90C82C;
        for _ in 0..4 {
            let mut a0 = a2;
            for _ in 0..16 {
                self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
                a1 += 2;
                a0 += 0x40;
            }
            a2 += 4;
        }
    }

    /// 0x80064: B28 variant 6 (49 tile codes).
    fn prop_b28_var6(&mut self, phase: u8) {
        let p = self.assets.enemy.t.u32(0x800AE, (phase & 0xC) as u32);
        let mut a1 = p;
        // 0x8007A: 1 word at 0x90CC2C
        self.prop_tile_code(TilePos::at(0x90CC2C), self.assets.enemy.t.u16(a1, 0));
        a1 += 2;
        // 0x80080, 0x80090, 0x800A0: 3 rows of 16 words from 0x90CC30, 0x90CC34, 0x90CC38
        let mut a2 = 0x90CC30;
        for _ in 0..3 {
            let mut a0 = a2;
            for _ in 0..16 {
                self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
                a1 += 2;
                a0 += 0x40;
            }
            a2 += 4;
        }
    }

    /// 0x8026E: B28 variant 8 (50 tile codes).
    fn prop_b28_var8(&mut self, phase: u8) {
        let p = self.assets.enemy.t.u32(0x802C4, (phase & 0xC) as u32);
        let mut a1 = p;
        // 0x80286: 13 words from 0x90CC30
        let mut a0 = 0x90CC30;
        for _ in 0..13 {
            self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
            a1 += 2;
            a0 += 0x40;
        }
        // 0x80296: 15 words from 0x90CC34
        let mut a0 = 0x90CC34;
        for _ in 0..15 {
            self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
            a1 += 2;
            a0 += 0x40;
        }
        // 0x802A6: 16 words from 0x90CC38
        let mut a0 = 0x90CC38;
        for _ in 0..16 {
            self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
            a1 += 2;
            a0 += 0x40;
        }
        // 0x802B2: 6 words from 0x90CEBC (0x90CC38 + 0x284)
        let mut a0 = 0x90CEBC;
        for _ in 0..6 {
            self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
            a1 += 2;
            a0 += 0x40;
        }
    }

    /// 0x8048A pool B class 29: sewer water bands (3 rows by 16 columns).
    pub fn prop_b29(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x8049E: mode 0 -> mode 2
                p.b.mode = 2;
            }
            2 => {
                // 0x804A4: camera cull test
                if p.b.xi().wrapping_add(0x100) < self.cam.s2.cam_x as i16 {
                    // 0x804C0: behind camera, advance to mode 4 to free next frame
                    p.b.mode += 2;
                    return;
                }
                // 0x804B2: check tile animation clock
                let (phase72, prev73) = self.prop_tile_phase();
                if phase72 == prev73 {
                    return; // 0x804BE
                }
                // 0x804C6: draw block
                let var = p.b.var26 as u32;
                if var > 0x18 || (var & 3) != 0 {
                    return self.prop_free(p, i);
                }
                let dest = self.assets.enemy.t.u32(0x80518, var);
                let phase_tbl = self.assets.enemy.t.u32(0x80534, var);
                let src = self.assets.enemy.t.u32(phase_tbl, (phase72 & 0xC) as u32);
                let mut a1 = src;
                let mut a2 = dest;
                for _ in 0..3 {
                    let mut a0 = a2;
                    for _ in 0..16 {
                        self.prop_tile_code(TilePos::at(a0), self.assets.enemy.t.u16(a1, 0));
                        a1 += 2;
                        a0 += 0x40;
                    }
                    a2 += 4;
                }
            }
            _ => self.prop_free(p, i), // 0x81040: jmp 0x4938
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn cell(g: &Game, a: u32) -> u16 {
        g.vram.scroll2[((a - 0x90C000) / 4) as usize].code
    }

    fn test_game() -> Game {
        let mut a = AssetPack::default();
        // Synthetic blobs for B28 (var 0) and B29 (var 0)
        // 0x7F7C8: 4 pointers (16 bytes)
        let ptr_bytes: Vec<u8> = [0x7F7D8u32, 0x7F7D8, 0x7F7D8, 0x7F7D8]
            .iter()
            .flat_map(|p| p.to_be_bytes())
            .collect();
        // 42 tile code words (84 bytes)
        let code_bytes: Vec<u8> = (0..42u16).flat_map(|w| (0x1200 + w).to_be_bytes()).collect();
        a.enemy.t.blobs.push((0x7F7C8, ptr_bytes));
        a.enemy.t.blobs.push((0x7F7D8, code_bytes));

        // B29 synthetic data
        // 0x80518: 7 dest addresses
        let dests: Vec<u8> = (0..7u32)
            .flat_map(|_| 0x90C430u32.to_be_bytes())
            .collect();
        a.enemy.t.blobs.push((0x80518, dests));
        // 0x80534: 7 phase table pointers
        let ptbls: Vec<u8> = (0..7u32)
            .flat_map(|_| 0x80550u32.to_be_bytes())
            .collect();
        a.enemy.t.blobs.push((0x80534, ptbls));
        // 0x80550: 4 phase pointers
        let phases: Vec<u8> = (0..4u32)
            .flat_map(|_| 0x805C0u32.to_be_bytes())
            .collect();
        a.enemy.t.blobs.push((0x80550, phases));
        // 0x805C0: 48 tile codes (96 bytes)
        let b29_codes: Vec<u8> = (0..48u16).flat_map(|w| (0x2400 + w).to_be_bytes()).collect();
        a.enemy.t.blobs.push((0x805C0, b29_codes));

        Game::new(Box::leak(Box::new(a)))
    }

    #[test]
    fn b28_advances_mode_and_writes_tiles_on_clock_tick() {
        let mut g = test_game();
        let i = g.props.alloc().unwrap();
        {
            let p = &mut g.props.slots[i];
            p.b.live = 1;
            p.b.k20 = 28 * 4;
            p.b.var26 = 0;
            p.b.set_xi(0x200);
        }
        g.cam.s2.cam_x = 0x100;
        // Mode 0 -> mode 2
        let mut p = g.props.slots[i].clone();
        g.prop_b28(&mut p, i);
        g.props.slots[i] = p.clone();
        assert_eq!(p.b.mode, 2);

        // Same phase: no write
        g.cam.s2.phase72 = 4;
        g.cam.s2.prev73 = 4;
        g.prop_b28(&mut p, i);
        assert_eq!(cell(&g, 0x90C130), 0);

        // Differing phase: writes tiles
        g.cam.s2.phase72 = 0;
        g.cam.s2.prev73 = 4;
        g.prop_b28(&mut p, i);
        g.props.slots[i] = p.clone();
        assert_eq!(cell(&g, 0x90C130), 0x1200);

        // Behind camera: mode advances to 4
        g.cam.s2.cam_x = 0x400; // xi (0x200) + 0x100 = 0x300 < 0x400
        g.prop_b28(&mut p, i);
        assert_eq!(p.b.mode, 4);
    }

    #[test]
    fn b29_writes_3_rows_by_16_cols() {
        let mut g = test_game();
        let i = g.props.alloc().unwrap();
        {
            let p = &mut g.props.slots[i];
            p.b.live = 1;
            p.b.k20 = 29 * 4;
            p.b.var26 = 0;
            p.b.set_xi(0x200);
        }
        g.cam.s2.cam_x = 0x100;
        let mut p = g.props.slots[i].clone();
        g.prop_b29(&mut p, i);
        g.props.slots[i] = p.clone();
        assert_eq!(p.b.mode, 2);

        // Trigger draw
        g.cam.s2.phase72 = 0;
        g.cam.s2.prev73 = 4;
        g.prop_b29(&mut p, i);
        g.props.slots[i] = p;

        // Verify row 0, col 0 and row 2, col 15
        assert_eq!(cell(&g, 0x90C430), 0x2400);
        assert_eq!(cell(&g, 0x90C430 + 15 * 0x40 + 8), 0x2400 + 47);
    }
}
