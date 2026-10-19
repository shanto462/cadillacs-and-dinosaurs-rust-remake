//! Pool B class 39, the swamp water surface of Episode 6 area 2 (0x830C6).
//!
//! Mode 0 (0x830DA) sets mode 2, `+$A0 = 15` and `+$A2 = 0`, then returns.
//! Mode 2 (0x830EA) checks if camera Y >= 0x170 (0x830EE). If so, it sets mode 4
//! which frees on the next update. Otherwise `+$A0` counts down. At 0 it reloads 24,
//! steps the phase word `+$A2` through 4, 8, 0xC, 0, then writes two blocks:
//! 1. Staircase block cols 48..63, rows 43..47 (0x83118).
//! 2. Rectangular block cols 0..15, rows 43..47 (0x8318C).
//! Mode 4 and mode 6 are 0x8369E (`jmp 0x4938`).

use super::tile_block::TilePos;
use super::Prop;
use crate::game::Game;

/// Pointer table for part 1 (0x831BE): 4 pointers to 70-word code lists.
const PTRS1: u32 = 0x831BE;
/// Pointer table for part 2 (0x8340E): 4 pointers to 80-word code lists.
const PTRS2: u32 = 0x8340E;

impl Game {
    /// 0x830C6.
    pub fn prop_b39(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x830DA. Does not fall into mode 2.
                p.b.mode = 2;
                p.m.sb(0xA0, 0x0F);
                p.m.sw(0xA2, 0);
            }
            2 => self.prop_b39_run(p),
            _ => self.prop_free(p, i), // 0x8369E
        }
    }

    /// 0x830EA. Runs while camera Y < 0x170.
    fn prop_b39_run(&mut self, p: &mut Prop) {
        // 0x830EE: cmpi.w #$170, $748(a5); bge.b 0x83112 (signed)
        if (self.cam.s2.cam_y as i16) >= 0x0170 {
            p.b.mode = p.b.mode.wrapping_add(2); // 0x83112
            return;
        }
        let n = p.m.b(0xA0).wrapping_sub(1);
        p.m.sb(0xA0, n);
        if n != 0 {
            return; // 0x83110
        }
        p.m.sb(0xA0, 0x18);
        let phase = p.m.w(0xA2).wrapping_add(4) & 0x0C;
        p.m.sw(0xA2, phase);
        self.prop_b39_part1(phase);
        self.prop_b39_part2(phase);
    }

    /// 0x83118. Staircase block: rows 43 to 47 in columns 48 to 63.
    /// Row 43: cols 52..63 (12 tiles).
    /// Row 44: cols 51..63 (13 tiles).
    /// Row 45: cols 50..63 (14 tiles).
    /// Row 46: cols 49..63 (15 tiles).
    /// Row 47: cols 48..63 (16 tiles).
    fn prop_b39_part1(&mut self, phase: u16) {
        let mut src = self.assets.enemy.t.u32(PTRS1, phase as u32) & 0xFF_FFFF;
        let mut a2: u32 = 0x90EC2C;
        for p in 0..5u32 {
            let mut a0 = a2.wrapping_add((4 - p) * 0x40);
            for _ in 0..(12 + p) {
                let code = self.assets.enemy.t.u16(src, 0);
                self.prop_tile_code(TilePos::at(a0), code);
                src = src.wrapping_add(2);
                a0 = a0.wrapping_add(0x40);
            }
            a2 = a2.wrapping_add(4);
        }
    }

    /// 0x8318C. 16 columns (cols 0 to 15), 5 rows (rows 43 to 47), starting at 0x90E02C.
    fn prop_b39_part2(&mut self, phase: u16) {
        let mut src = self.assets.enemy.t.u32(PTRS2, phase as u32) & 0xFF_FFFF;
        let mut a0: u32 = 0x90E02C;
        for _ in 0..16 {
            for r in 0..5u32 {
                let code = self.assets.enemy.t.u16(src, 0);
                self.prop_tile_code(TilePos::at(a0.wrapping_add(r * 4)), code);
                src = src.wrapping_add(2);
            }
            a0 = a0.wrapping_add(0x40);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::enemy::ByteTables;
    use dino_assets::stage::Cell;
    use dino_assets::AssetPack;

    fn words(start: u16, n: usize) -> Vec<u8> {
        (0..n as u16).flat_map(|i| (start + i).to_be_bytes()).collect()
    }

    fn synthetic_blob() -> Vec<u8> {
        let mut b = Vec::new();
        // PTRS1 at 0x831BE: 4 pointers (each pointing to 70 words = 140 bytes)
        for i in 0..4u32 {
            b.extend_from_slice(&(0x831CE + i * 140).to_be_bytes());
        }
        // 4 pictures of 70 words
        for p in 0..4u16 {
            b.extend(words(0x1000 + p * 0x100, 70));
        }
        assert_eq!(b.len(), 16 + 4 * 140);
        // Resize to 0x250 so PTRS2 starts exactly at 0x8340E
        b.resize(0x250, 0);
        // PTRS2 at 0x8340E: 4 pointers (each pointing to 80 words = 160 bytes)
        for i in 0..4u32 {
            b.extend_from_slice(&(0x8341E + i * 160).to_be_bytes());
        }
        // 4 pictures of 80 words
        for p in 0..4u16 {
            b.extend(words(0x2000 + p * 0x100, 80));
        }
        b
    }

    fn game() -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs: vec![(PTRS1, synthetic_blob())] };
        Game::new(Box::leak(Box::new(a)))
    }

    fn cell(g: &Game, addr: u32) -> (u16, u16) {
        let c = g.vram.scroll2[((addr - 0x90C000) / 4) as usize];
        (c.code, c.attr)
    }

    fn paint(g: &mut Game, addr: u32) {
        g.vram.scroll2[((addr - 0x90C000) / 4) as usize] = Cell { code: 0x1111, attr: 0xABCD };
    }

    fn live(g: &Game) -> Option<&Prop> {
        g.props.slots.iter().find(|s| s.b.live != 0 && s.b.k20 == 39 * 4)
    }

    /// Mode 0 does not write. The 16th update writes phase 4 (first index = 4).
    /// Then every 24th update steps 8, 0xC, 0, 4...
    #[test]
    fn b39_first_picture_is_on_the_16th_update_then_every_24() {
        let mut g = game();
        // Part 1 destination: row 43, col 52 is 0x90EC2C + 4 * 0x40 = 0x90ED2C
        paint(&mut g, 0x90ED2C);
        // Part 2 destination: row 43, col 0 is 0x90E02C
        paint(&mut g, 0x90E02C);

        g.spawn_prop(39, 0, None).unwrap();
        g.pool_b_update();
        let p = live(&g).unwrap();
        assert_eq!((p.b.mode, p.m.b(0xA0), p.m.w(0xA2)), (2, 0x0F, 0));
        assert_eq!(cell(&g, 0x90ED2C).0, 0x1111, "mode 0 does not write");

        for _ in 0..14 {
            g.pool_b_update();
        }
        assert_eq!(live(&g).unwrap().m.b(0xA0), 1);
        assert_eq!(cell(&g, 0x90ED2C).0, 0x1111);

        g.pool_b_update();
        assert_eq!(live(&g).unwrap().m.w(0xA2), 4);
        assert_eq!(live(&g).unwrap().m.b(0xA0), 0x18);
        // Phase 4 points to picture 1 (start 0x1100 for part 1, 0x2100 for part 2)
        assert_eq!(cell(&g, 0x90ED2C), (0x1100, 0xABCD));
        assert_eq!(cell(&g, 0x90E02C), (0x2100, 0xABCD));

        // Next phase is 8 (picture 2)
        for _ in 0..24 {
            g.pool_b_update();
        }
        assert_eq!(live(&g).unwrap().m.w(0xA2), 8);
        assert_eq!(cell(&g, 0x90ED2C).0, 0x1200);
        assert_eq!(cell(&g, 0x90E02C).0, 0x2200);

        // Next phase is 0xC (picture 3)
        for _ in 0..24 {
            g.pool_b_update();
        }
        assert_eq!(live(&g).unwrap().m.w(0xA2), 0x0C);
        assert_eq!(cell(&g, 0x90ED2C).0, 0x1300);
        assert_eq!(cell(&g, 0x90E02C).0, 0x2300);

        // Next phase is 0 (picture 0)
        for _ in 0..24 {
            g.pool_b_update();
        }
        assert_eq!(live(&g).unwrap().m.w(0xA2), 0);
        assert_eq!(cell(&g, 0x90ED2C).0, 0x1000);
        assert_eq!(cell(&g, 0x90E02C).0, 0x2000);
    }

    /// Camera Y >= 0x170 stops and frees the prop.
    #[test]
    fn b39_stops_when_camera_y_ge_170() {
        let mut g = game();
        g.spawn_prop(39, 0, None).unwrap();
        g.pool_b_update();
        assert_eq!(live(&g).unwrap().b.mode, 2);

        g.cam.s2.cam_y = 0x016F;
        g.pool_b_update();
        assert_eq!(live(&g).unwrap().b.mode, 2, "0x16F still runs");

        g.cam.s2.cam_y = 0x0170;
        g.pool_b_update();
        assert_eq!(live(&g).unwrap().b.mode, 4, "0x170 triggers mode 4");

        g.pool_b_update();
        assert!(live(&g).is_none(), "mode 4 frees on next update");
    }
}
