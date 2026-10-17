//! Pool B class 27, the Butcher yard tile animation of Episode 2 area 2 (0x7EEBA).
//!
//! Mode 0 (0x7EECE) sets mode 2, `+$A0 = 15` and `+$A2 = 0`, then returns.
//! Mode 2 (0x7EEDE) frees once the camera X is past object X + 0x1A0 (`blt`, so the edge still
//! runs). Otherwise it counts `+$A0` down and, every 24 frames after the first 15, writes the
//! next of 4 pictures. Mode 4 and mode 6 are 0x7F724 (`jmp 0x4938`).

use super::tile_block::{TileMode, TilePos};
use super::Prop;
use crate::game::Game;

/// Long pointer table of the 4 pictures (0x7F1D4). `+$A2` is the byte offset 0, 4, 8 or 0xC.
const PICTURES: u32 = 0x7F1D4;

/// 0x7EF2C, one column top to bottom. The counts are the unrolled `move.w (a1)+,(a0)` runs.
const PIT: &[(u32, u16)] = &[
    (0x90F000, 2),
    (0x90F040, 2),
    (0x90F080, 4),
    (0x90F0C0, 4),
    (0x90F100, 6),
    (0x90F140, 6),
    (0x90F180, 8),
    (0x90F1C0, 8),
    (0x90F200, 10),
    (0x90F240, 10),
    (0x90F284, 11),
    (0x90F2C8, 10),
    (0x90F30C, 11),
    (0x90F350, 10),
    (0x90F394, 10),
    (0x90F3D8, 9),
    (0x90F41C, 8),
    (0x90F460, 7),
    (0x90F4A4, 6),
    (0x90F4E8, 5),
    (0x90F52C, 4),
    (0x90F570, 3),
    (0x90F5B4, 2),
    (0x90F5F8, 1),
];

impl Game {
    /// 0x7EEBA.
    pub fn prop_b27(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x7EECE. Does not fall into mode 2, and does not clear +$A4.
                p.b.mode = 2;
                p.m.sb(0xA0, 0x0F);
                p.m.sw(0xA2, 0);
            }
            2 => self.prop_b27_run(p),
            _ => self.prop_free(p, i), // 0x7F724
        }
    }

    /// 0x7EEDE. The camera test is first, so a camera that has passed does not count the timer.
    fn prop_b27_run(&mut self, p: &mut Prop) {
        // 0x7EEDE: cmp.w G.$744, X+0x1A0; blt (signed) bumps the mode. Equal still runs.
        let limit = p.b.xi().wrapping_add(0x1A0);
        if limit < self.cam.s2.cam_x as i16 {
            p.b.mode = p.b.mode.wrapping_add(2); // 0x7EF06
            return;
        }
        let n = p.m.b(0xA0).wrapping_sub(1);
        p.m.sb(0xA0, n);
        if n != 0 {
            return; // 0x7EF04
        }
        p.m.sb(0xA0, 0x18);
        let index = p.m.w(0xA2).wrapping_add(4) & 0xC;
        p.m.sw(0xA2, index);
        self.prop_b27_paint(index); // 0x7EF02 bra 0x7EF0C, same update
    }

    /// 0x7EF0C. Nine columns of one row at 0x90E03C (`moveq #8` / `dbra` runs 9 times), then the
    /// pit runs at 0x90F000. 166 words. Each picture is 0x150 bytes, so the last two words are padding.
    fn prop_b27_paint(&mut self, index: u16) {
        let mut src = self.assets.enemy.t.u32(PICTURES, index as u32) & 0xFF_FFFF;
        self.prop_tile_block(TilePos::at(0x90E03C), 9, 1, src, TileMode::Codes);
        src = src.wrapping_add(18);
        for &(at, rows) in PIT {
            self.prop_tile_block(TilePos::at(at), 1, rows, src, TileMode::Codes);
            src = src.wrapping_add(rows as u32 * 2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::enemy::ByteTables;
    use dino_assets::stage::Cell;
    use dino_assets::AssetPack;

    /// Four pictures of 168 words. Word `i` of picture `p` is `0x1000 * (p + 1) + i`.
    fn pictures() -> Vec<u8> {
        let mut b = Vec::new();
        for addr in [0x7F1E4u32, 0x7F334, 0x7F484, 0x7F5D4] {
            b.extend_from_slice(&addr.to_be_bytes());
        }
        for p in 0..4u16 {
            for i in 0..168u16 {
                b.extend_from_slice(&(0x1000 * (p + 1) + i).to_be_bytes());
            }
        }
        b
    }

    fn game() -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs: vec![(PICTURES, pictures())] };
        Game::new(Box::leak(Box::new(a)))
    }

    fn cell(g: &Game, addr: u32) -> (u16, u16) {
        let c = g.vram.scroll2[((addr - 0x90C000) / 4) as usize];
        (c.code, c.attr)
    }

    fn paint_all(g: &mut Game) {
        for c in &mut g.vram.scroll2 {
            *c = Cell { code: 0x1111, attr: 0xABCD };
        }
    }

    fn live(g: &Game) -> Option<&Prop> {
        g.props.slots.iter().find(|s| s.b.live != 0 && s.b.k20 == 27 * 4)
    }

    fn changed(g: &Game) -> usize {
        g.vram.scroll2.iter().filter(|c| c.code != 0x1111).count()
    }

    /// The 16th update writes picture 1 (index 4). 166 cells change. The index then walks 8, 0xC, 0.
    #[test]
    fn the_first_picture_is_index_4_on_the_16th_update() {
        let mut g = game();
        paint_all(&mut g);
        g.spawn_prop(27, 0, Some((0, 0x100, 0))).unwrap();
        g.pool_b_update();
        assert_eq!((live(&g).unwrap().b.mode, live(&g).unwrap().m.b(0xA0), live(&g).unwrap().m.w(0xA2)), (2, 0x0F, 0));
        assert_eq!(changed(&g), 0, "mode 0 does not write");

        for _ in 0..14 {
            g.pool_b_update();
        }
        assert_eq!(changed(&g), 0);
        g.pool_b_update();
        assert_eq!(live(&g).unwrap().m.w(0xA2), 4);
        assert_eq!(changed(&g), 166);
        assert_eq!(cell(&g, 0x90E03C), (0x2000, 0xABCD));
        assert_eq!(cell(&g, TilePos::at(0x90E03C).right(8).0).0, 0x2000 + 8);
        assert_eq!(cell(&g, 0x90E040).0, 0x1111, "the row under the lip is not written");
        assert_eq!(cell(&g, 0x90F000).0, 0x2000 + 9);
        assert_eq!(cell(&g, 0x90F004).0, 0x2000 + 10);
        assert_eq!(cell(&g, 0x90F008).0, 0x1111, "the third row of that column is outside the run");
        assert_eq!(cell(&g, 0x90F5F8).0, 0x2000 + 165);
        assert_eq!(cell(&g, 0x90F5FC).0, 0x1111);

        let index = [8u16, 0xC, 0];
        let base = [0x3000u16, 0x4000, 0x1000];
        for (ix, code) in index.into_iter().zip(base) {
            for _ in 0..24 {
                g.pool_b_update();
            }
            assert_eq!(live(&g).unwrap().m.w(0xA2), ix);
            assert_eq!(cell(&g, 0x90E03C).0, code);
            assert_eq!(cell(&g, 0x90F5F8).0, code + 165);
        }
    }

    /// Camera X == X + 0x1A0 still writes. One pixel past that sets mode 4 with no write, and the next update frees.
    #[test]
    fn the_camera_window_is_inclusive_and_then_frees() {
        let mut g = game();
        paint_all(&mut g);
        g.cam.s2.cam_x = 0x1A0;
        g.spawn_prop(27, 0, Some((0, 0, 0))).unwrap();
        for _ in 0..16 {
            g.pool_b_update();
        }
        assert_eq!(changed(&g), 166, "X + 0x1A0 is still inside");
        assert_eq!(live(&g).unwrap().b.mode, 2);

        g.cam.s2.cam_x = 0x1A1;
        g.pool_b_update();
        let p = live(&g).unwrap();
        assert_eq!(p.b.mode, 4);
        assert_eq!(p.m.b(0xA0), 0x18, "the timer is not counted once the camera has passed");
        assert_eq!(cell(&g, 0x90E03C).0, 0x2000, "that update does not write");
        g.pool_b_update();
        assert!(live(&g).is_none());
    }
}
