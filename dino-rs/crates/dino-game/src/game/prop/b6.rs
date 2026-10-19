//! Pool B class 6, the far interior flames of Episode 5 area 1 (0x76A70).
//!
//! Trigger 0xF00 spawns three objects: (0x800, 0xA0, 0) variant 0, (0x900, 0xA0, 0) variant 4,
//! (0xA00, 0xA0, 0) variant 8. Mode 0 (0x76A84) sets mode 2, `+$A0 = 1` and `+$A2 = 0`. Mode 2
//! (0x76A94) frees on the next update once `(X + 0x100) <` the scroll3 camera (G.$7E4, signed
//! `blt`). Otherwise it counts `+$A0` and, every 12 frames after the first, writes the next of 4
//! pictures. Variant 4 uses the split writer (0x76B1C): four columns, then the next four start at
//! the destination plus 0x7C. Any other variant uses the 8 by 2 writer (0x76AFE). The destination
//! long is at 0x76AE6 and the picture base at 0x76AF2, both indexed by the variant word. Mode 4
//! and mode 6 are 0x76CCC (`jmp 0x4938`). No sprite, no random number, no sound.

use super::Prop;
use super::tile_block::{TileMode, TilePos};
use crate::game::Game;

impl Game {
    /// 0x76A70.
    pub fn prop_b6(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x76A84. Does not fall into mode 2.
                p.b.mode = 2;
                p.m.sb(0xA0, 1);
                p.m.sw(0xA2, 0);
            }
            2 => self.prop_b6_run(p),
            _ => self.prop_free(p, i), // 0x76CCC
        }
    }

    /// 0x76A94. The camera test is first, so a camera that has passed does not count the timer.
    fn prop_b6_run(&mut self, p: &mut Prop) {
        let limit = p.b.xi().wrapping_add(0x100);
        if limit < self.cam.s3.cam_x as i16 {
            p.b.mode = p.b.mode.wrapping_add(2); // 0x76AE0
            return;
        }
        let n = p.m.b(0xA0).wrapping_sub(1);
        p.m.sb(0xA0, n);
        if n != 0 {
            return; // 0x76AA6
        }
        p.m.sb(0xA0, 0x0C); // 0x76AAA
        let mut index = p.m.w(0xA2).wrapping_add(4);
        if index == 0x10 {
            index = 0; // 0x76AB4
        }
        p.m.sw(0xA2, index);
        self.prop_b6_paint(p); // 0x76AC0
    }

    /// 0x76AC0. `+$A2 << 3` is the byte offset of the picture (16 code words). Variant 4 only
    /// takes the split writer.
    fn prop_b6_paint(&mut self, p: &Prop) {
        let var = u32::from(p.b.var26);
        let dest = self.assets.enemy.t.u32(0x76AE6, var) & 0x00FF_FFFF;
        let base = self.assets.enemy.t.u32(0x76AF2, var) & 0x00FF_FFFF;
        let src = base.wrapping_add(u32::from(p.m.w(0xA2)) << 3);
        if p.b.var26 == 4 {
            // 0x76B1C. After four columns a0 is dest+0x80, and `subq.w #4,a0` starts the next
            // four at dest+0x7C. a1 has already consumed 8 words.
            self.prop_tile_block(TilePos::at(dest), 4, 2, src, TileMode::Codes);
            self.prop_tile_block(
                TilePos::at(dest.wrapping_add(0x7C)),
                4,
                2,
                src.wrapping_add(16),
                TileMode::Codes,
            );
        } else {
            self.prop_tile_block(TilePos::at(dest), 8, 2, src, TileMode::Codes); // 0x76AFE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use dino_assets::enemy::ByteTables;
    use dino_assets::stage::Cell;

    fn be32(v: u32) -> [u8; 4] {
        v.to_be_bytes()
    }

    /// Destinations, picture bases and three variants of four 16-word pictures.
    /// Word `k` of picture `p` of variant slot `v` is `0x1000 * (v + 1) + 0x100 * p + k`.
    fn blobs() -> Vec<(u32, Vec<u8>)> {
        let mut dests = Vec::new();
        for a in [0x913808u32, 0x913908, 0x913A04] {
            dests.extend_from_slice(&be32(a));
        }
        let mut bases = Vec::new();
        for a in [0x76B4Cu32, 0x76BCC, 0x76C4C] {
            bases.extend_from_slice(&be32(a));
        }
        let mut pics = Vec::new();
        for v in 0..3u16 {
            for p in 0..4u16 {
                for k in 0..16u16 {
                    pics.extend_from_slice(&(0x1000 * (v + 1) + 0x100 * p + k).to_be_bytes());
                }
            }
        }
        vec![(0x76AE6, dests), (0x76AF2, bases), (0x76B4C, pics)]
    }

    fn game() -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs: blobs(), ..Default::default() };
        Game::new(Box::leak(Box::new(a)))
    }

    fn cell(g: &Game, addr: u32) -> (u16, u16) {
        let c = g.vram.scroll3[((addr - 0x910000) / 4) as usize];
        (c.code, c.attr)
    }

    fn paint_all(g: &mut Game) {
        for c in &mut g.vram.scroll3 {
            *c = Cell {
                code: 0x1111,
                attr: 0xABCD,
            };
        }
    }

    fn changed(g: &Game) -> usize {
        g.vram.scroll3.iter().filter(|c| c.code != 0x1111).count()
    }

    /// Variant 0 writes an 8 by 2 block of picture 1 on the second update. Twelve frames later
    /// the index is 8.
    #[test]
    fn variant_0_writes_eight_by_two_on_a_12_frame_period() {
        let mut g = game();
        paint_all(&mut g);
        g.spawn_prop(6, 0, Some((0x800, 0xA0, 0))).unwrap();
        g.pool_b_update();
        assert_eq!(changed(&g), 0, "mode 0 does not write");
        g.pool_b_update();
        let p = g.props.slots.iter().find(|s| s.b.live != 0).unwrap();
        assert_eq!((p.m.b(0xA0), p.m.w(0xA2)), (0x0C, 4));
        assert_eq!(cell(&g, 0x913808), (0x1100, 0xABCD));
        assert_eq!(cell(&g, 0x91380C).0, 0x1101);
        assert_eq!(cell(&g, 0x913808 + 7 * 0x20).0, 0x110E);
        assert_eq!(cell(&g, 0x913808 + 7 * 0x20 + 4).0, 0x110F);
        assert_eq!(
            cell(&g, 0x913808 + 8 * 0x20).0,
            0x1111,
            "column 8 is outside the block"
        );
        assert_eq!(changed(&g), 16);

        for _ in 0..11 {
            g.pool_b_update();
        }
        assert_eq!(p_index(&g), 4);
        g.pool_b_update();
        assert_eq!(p_index(&g), 8);
        assert_eq!(cell(&g, 0x913808).0, 0x1200);
    }

    /// Variant 4 writes four columns, then four more starting at dest+0x7C (one row above the
    /// next column). Variant 8 stays an 8 by 2 block at its own destination.
    #[test]
    fn variant_4_splits_and_variant_8_does_not() {
        let mut g = game();
        paint_all(&mut g);
        g.spawn_prop(6, 4, Some((0x900, 0xA0, 0))).unwrap();
        g.spawn_prop(6, 8, Some((0xA00, 0xA0, 0))).unwrap();
        g.pool_b_update();
        g.pool_b_update();
        assert_eq!(cell(&g, 0x913908).0, 0x2100);
        assert_eq!(cell(&g, 0x913908 + 3 * 0x20 + 4).0, 0x2107);
        assert_eq!(
            cell(&g, 0x913908 + 0x7C).0,
            0x2108,
            "the second group starts at dest+0x7C"
        );
        assert_eq!(
            cell(&g, 0x913908 + 0x80).0,
            0x2109,
            "dest+0x80 is the second row, not word 8"
        );
        assert_eq!(cell(&g, 0x913908 + 0x7C + 3 * 0x20 + 4).0, 0x210F);
        assert_eq!(
            cell(&g, 0x913980).0,
            0x1111,
            "the cell before the split is not written"
        );

        assert_eq!(cell(&g, 0x913A04).0, 0x3100);
        assert_eq!(cell(&g, 0x913A04 + 7 * 0x20 + 4).0, 0x310F);
        assert_eq!(
            cell(&g, 0x913A04 + 0x7C).0,
            0x1111,
            "variant 8 does not split"
        );
        assert_eq!(changed(&g), 32);
    }

    /// The far camera, not the scroll2 camera, ends the object. Equal still runs.
    #[test]
    fn the_camera_window_is_the_scroll3_edge() {
        let mut g = game();
        paint_all(&mut g);
        g.cam.s2.cam_x = 0x7FFF;
        g.cam.s3.cam_x = 0x900;
        g.spawn_prop(6, 0, Some((0x800, 0, 0))).unwrap();
        g.pool_b_update();
        g.pool_b_update();
        assert_eq!(
            cell(&g, 0x913808).0,
            0x1100,
            "X + 0x100 equals the far camera"
        );
        g.cam.s3.cam_x = 0x901;
        g.pool_b_update();
        let p = g.props.slots.iter().find(|s| s.b.k20 == 6 * 4).unwrap();
        assert_eq!(p.b.mode, 4);
        assert_eq!(
            p.m.b(0xA0),
            0x0C,
            "the timer is not counted once the camera has passed"
        );
        g.pool_b_update();
        assert!(g.props.slots.iter().all(|s| s.b.live == 0));
    }

    fn p_index(g: &Game) -> u16 {
        g.props
            .slots
            .iter()
            .find(|s| s.b.live != 0)
            .unwrap()
            .m
            .w(0xA2)
    }
}
