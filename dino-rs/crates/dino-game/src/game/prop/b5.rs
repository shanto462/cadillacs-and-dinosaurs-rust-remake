//! Pool B class 5, the far chimney smoke of Episode 5 area 1 (0x769E2).
//!
//! Trigger 0xB00 spawns variant 0 at (0x700, 0xE0, 0). Mode 0 (0x769F6) sets mode 2, `+$A0 = 1`
//! and `+$A2 = 0`, and does not paint. Mode 2 (0x76A06) frees on the next update once
//! `(X + 0x20) <` the scroll3 camera (G.$7E4, signed `blt`, so the edge still runs). Otherwise it
//! counts `+$A0` and, every 16 frames after the first, writes the next of 4 pictures: two scroll3
//! code words, one column, at 0x913F00. `+$26` is loaded at 0x76A48 and never used. Mode 4 and
//! mode 6 are 0x76A6A (`jmp 0x4938`). No sprite, no random number, no sound.

use super::Prop;
use super::tile_block::{TileMode, TilePos};
use crate::game::Game;

/// Four pictures of two code words (0x76A5A). `+$A2` is the byte offset 0, 4, 8 or 0xC.
const PICTURES: u32 = 0x76A5A;

impl Game {
    /// 0x769E2.
    pub fn prop_b5(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x769F6. Does not fall into mode 2.
                p.b.mode = 2;
                p.m.sb(0xA0, 1);
                p.m.sw(0xA2, 0);
            }
            2 => self.prop_b5_run(p),
            _ => self.prop_free(p, i), // 0x76A6A
        }
    }

    /// 0x76A06. The camera test is first, so a camera that has passed does not count the timer.
    fn prop_b5_run(&mut self, p: &mut Prop) {
        let limit = p.b.xi().wrapping_add(0x20);
        if limit < self.cam.s3.cam_x as i16 {
            p.b.mode = p.b.mode.wrapping_add(2); // 0x76A38
            return;
        }
        // 0x76A14: subq.b #1. A stored 0 becomes 0xFF and this frame does not paint.
        let n = p.m.b(0xA0).wrapping_sub(1);
        p.m.sb(0xA0, n);
        if n != 0 {
            return; // 0x76A18
        }
        p.m.sb(0xA0, 0x10); // 0x76A1C
        let mut index = p.m.w(0xA2).wrapping_add(4);
        if index == 0x10 {
            index = 0; // 0x76A26, word compare, then clr.w
        }
        p.m.sw(0xA2, index);
        self.prop_b5_paint(index); // 0x76A32
    }

    /// 0x76A3E. Two code words down one scroll3 column. The variant word is loaded and unused.
    fn prop_b5_paint(&mut self, index: u16) {
        let src = PICTURES.wrapping_add(u32::from(index));
        self.prop_tile_block(TilePos::at(0x913F00), 1, 2, src, TileMode::Codes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use dino_assets::enemy::ByteTables;
    use dino_assets::stage::Cell;

    /// Four pictures of two words. Word `k` of picture `p` is `0x1000 * (p + 1) + k`.
    fn pictures() -> Vec<u8> {
        let mut b = Vec::new();
        for p in 0..4u16 {
            for k in 0..2u16 {
                b.extend_from_slice(&(0x1000 * (p + 1) + k).to_be_bytes());
            }
        }
        b
    }

    fn game() -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables {
            blobs: vec![(PICTURES, pictures())],
        };
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

    fn live(g: &Game) -> Option<&Prop> {
        g.props
            .slots
            .iter()
            .find(|s| s.b.live != 0 && s.b.k20 == 5 * 4)
    }

    /// The second update writes picture 1 (index 4). The index then walks 8, 0xC, 0. Variant 4
    /// still writes 0x913F00: the variant load at 0x76A48 has no effect.
    #[test]
    fn the_first_picture_is_index_4_and_the_period_is_16() {
        let mut g = game();
        paint_all(&mut g);
        g.spawn_prop(5, 4, Some((0x700, 0xE0, 0))).unwrap();
        g.pool_b_update();
        let p = live(&g).unwrap();
        assert_eq!((p.b.mode, p.m.b(0xA0), p.m.w(0xA2)), (2, 1, 0));
        assert_eq!(cell(&g, 0x913F00).0, 0x1111, "mode 0 does not write");

        g.pool_b_update();
        assert_eq!(live(&g).unwrap().m.w(0xA2), 4);
        assert_eq!(live(&g).unwrap().m.b(0xA0), 0x10);
        assert_eq!(cell(&g, 0x913F00), (0x2000, 0xABCD));
        assert_eq!(cell(&g, 0x913F04), (0x2001, 0xABCD));
        assert_eq!(
            cell(&g, 0x913F20).0,
            0x1111,
            "the next column is not written"
        );

        let mut held = 4u16;
        for (ix, code) in [(8u16, 0x3000u16), (0xC, 0x4000), (0, 0x1000)] {
            for _ in 0..15 {
                g.pool_b_update();
            }
            assert_eq!(
                live(&g).unwrap().m.w(0xA2),
                held,
                "fifteen frames do not advance"
            );
            g.pool_b_update();
            assert_eq!(live(&g).unwrap().m.w(0xA2), ix);
            assert_eq!(cell(&g, 0x913F00).0, code);
            assert_eq!(cell(&g, 0x913F04).0, code + 1);
            held = ix;
        }
    }

    /// A stored timer of 0 wraps to 0xFF (`subq.b`) and that frame does not paint.
    #[test]
    fn a_zero_timer_wraps_and_does_not_paint() {
        let mut g = game();
        paint_all(&mut g);
        g.spawn_prop(5, 0, Some((0, 0, 0))).unwrap();
        g.pool_b_update();
        live_mut(&mut g).m.sb(0xA0, 0);
        g.pool_b_update();
        let p = live(&g).unwrap();
        assert_eq!(p.m.b(0xA0), 0xFF);
        assert_eq!(p.m.w(0xA2), 0);
        assert_eq!(cell(&g, 0x913F00).0, 0x1111);
    }

    /// Camera X == X + 0x20 still runs. One pixel past sets mode 4 and does not count the timer.
    #[test]
    fn the_camera_window_is_the_scroll3_edge() {
        let mut g = game();
        paint_all(&mut g);
        g.cam.s3.cam_x = 0x120;
        g.spawn_prop(5, 0, Some((0x100, 0, 0))).unwrap();
        g.pool_b_update();
        g.pool_b_update();
        assert_eq!(cell(&g, 0x913F00).0, 0x2000, "X + 0x20 is still inside");
        assert_eq!(live(&g).unwrap().b.mode, 2);

        g.cam.s3.cam_x = 0x121;
        g.pool_b_update();
        let p = live(&g).unwrap();
        assert_eq!(p.b.mode, 4);
        assert_eq!(
            p.m.b(0xA0),
            0x10,
            "the timer is not counted once the camera has passed"
        );
        assert_eq!(cell(&g, 0x913F00).0, 0x2000);
        g.pool_b_update();
        assert!(live(&g).is_none());
    }

    fn live_mut(g: &mut Game) -> &mut Prop {
        g.props
            .slots
            .iter_mut()
            .find(|s| s.b.live != 0 && s.b.k20 == 5 * 4)
            .unwrap()
    }

    fn scroll2(g: &Game, addr: u32) -> (u16, u16) {
        let c = g.vram.scroll2[((addr - 0x90C000) / 4) as usize];
        (c.code, c.attr)
    }

    /// The user's ROM, when `dino.zip` is next to the repo. The first paint must be the extracted
    /// picture, so a short table fails here.
    fn rom_pack() -> Option<&'static AssetPack> {
        use std::path::Path;
        use std::sync::OnceLock;
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = Path::new("../../../dino.zip");
            path.exists().then(|| {
                &*Box::leak(Box::new(
                    dino_extract::extract(path).expect("the ROM set loads"),
                ))
            })
        })
    }

    #[test]
    fn the_rom_pictures_are_what_the_first_paint_writes() {
        let Some(pack) = rom_pack() else {
            eprintln!("skipped: no dino.zip");
            return;
        };
        let mut g = Game::new(pack);
        for c in &mut g.vram.scroll3 {
            *c = Cell {
                code: 0x1111,
                attr: 0xABCD,
            };
        }
        for c in &mut g.vram.scroll2 {
            *c = Cell {
                code: 0x2222,
                attr: 0xABCD,
            };
        }
        g.cam.s2.phase72 = 4;
        g.cam.s2.prev73 = 0;
        g.spawn_prop(5, 0, Some((0x700, 0xE0, 0))).unwrap();
        g.spawn_prop(6, 0, Some((0x800, 0xA0, 0))).unwrap();
        g.spawn_prop(6, 4, Some((0x900, 0xA0, 0))).unwrap();
        g.spawn_prop(6, 8, Some((0xA00, 0xA0, 0))).unwrap();
        g.spawn_prop(10, 0, Some((0xD20, 0x200, 0))).unwrap();
        g.spawn_prop(10, 4, Some((0xF20, 0x200, 0))).unwrap();
        g.pool_b_update();
        assert_eq!(cell(&g, 0x913F00).0, 0x1111, "mode 0 does not write");
        g.pool_b_update();

        let t = &g.assets.enemy.t;
        assert_eq!(cell(&g, 0x913F00), (t.u16(0x76A5A, 4), 0xABCD));
        assert_eq!(cell(&g, 0x913F04).0, t.u16(0x76A5A, 6));

        for var in [0u32, 4, 8] {
            let dest = t.u32(0x76AE6, var) & 0x00FF_FFFF;
            let src = (t.u32(0x76AF2, var) & 0x00FF_FFFF).wrapping_add(32);
            assert_eq!(cell(&g, dest).0, t.u16(src, 0), "var {var}");
            if var == 4 {
                assert_eq!(
                    cell(&g, dest + 0x7C).0,
                    t.u16(src, 16),
                    "the split starts at word 8"
                );
                assert_eq!(cell(&g, dest + 0x7C + 3 * 0x20 + 4).0, t.u16(src, 30));
            } else {
                assert_eq!(cell(&g, dest + 7 * 0x20 + 4).0, t.u16(src, 30));
                assert_eq!(cell(&g, dest + 0x7C).0, 0x1111, "var {var} does not split");
            }
        }

        let pic0 = t.u32(0x78664, 4) & 0x00FF_FFFF;
        assert_eq!(scroll2(&g, 0x90D4B4), (t.u16(pic0, 0), 0xABCD));
        assert_eq!(scroll2(&g, 0x90D4B4 + 12 * 0x40 + 8).0, t.u16(pic0, 76));
        let pic4 = t.u32(0x78898, 4) & 0x00FF_FFFF;
        assert_eq!(scroll2(&g, 0x90DC24).0, t.u16(pic4, 0));
        assert_eq!(scroll2(&g, 0x90DDFC).0, t.u16(pic4, 62));
        assert_eq!(
            scroll2(&g, 0x90DC40).0,
            0x2222,
            "the gap in the irregular patch stays"
        );
    }
}
