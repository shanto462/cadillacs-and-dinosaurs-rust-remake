//! Pool B class 7 (0x76CD2): tile animation. The variant word picks a mode table at 0x76CE6
//! (var 0 → 0x76CEE, var 4 → 0x76D8C). Any other variant is freed: the table has only those two.
//!
//! Var 0 (0x76CF6 / 0x76D06) is the near-layer pair of Episode 6. Mode 0 sets the byte `+$A0` to
//! 15 and `+$A2` to 0, then returns. Mode 2 stops (mode + 2) when the scroll2 camera X is above
//! 0xB80 (`bgt`, so 0xB80 still runs). Otherwise `+$A0` counts down and, every 24 frames after the
//! first 15, writes one 2×2 picture of scroll2 codes at 0x90ED4C. Five pictures (index 0, 2, 4, 6,
//! 8); the fifth sets the pause byte to 0x60 and clears the index.
//!
//! Var 4 (0x76D94 / 0x76DA4) is the far layer of Episode 2 area 1. Mode 0 sets the byte `+$A0` to
//! 0x40 (64 frames) and `+$A2` to 0, then returns. Mode 2 stops when object X + 0x40 is below the
//! scroll3 camera X (`blt` on `cmp.w G.$7E4`, so equal still runs). Otherwise the same countdown
//! writes two scroll3 codes, at 0x913D04 and one column to the right (0x913D24). Six pictures
//! (index steps of 4); the sixth sets the pause byte to 0x60 and clears the index.
//!
//! Mode 4 and mode 6 of both variants are 0x76D86 (`jmp 0x4938`).

use super::tile_block::TilePos;
use super::Prop;
use crate::game::Game;

/// Var 0 (0x76D3E): five offset words, then the 2×2 pictures they point at. `+$A2` is the byte offset.
const NEAR: u32 = 0x76D64;
/// Var 4 (0x76DDC): six pictures of two code words. `+$A2` is the byte offset, steps of 4.
const FAR: u32 = 0x76DF6;

/// Scroll2 2×2 of var 0 (0x76D4C): column 0 at 0x90ED4C, column 1 is `+$3C` from the second row.
const NEAR_AT: u32 = 0x90ED4C;
/// Scroll3 pair of var 4 (0x76DDC). The second cell is `adda.w #$20` (one scroll3 column).
const FAR_AT: u32 = 0x913D04;

impl Game {
    /// 0x76CD2.
    pub fn prop_b7(&mut self, p: &mut Prop, i: usize) {
        match p.b.var26 {
            0 => self.prop_b7_var0(p, i),
            4 => self.prop_b7_var4(p, i),
            _ => self.prop_free(p, i),
        }
    }

    /// Var 0, mode table 0x76CEE.
    fn prop_b7_var0(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x76CF6. Does not fall into mode 2.
                p.b.mode = p.b.mode.wrapping_add(2);
                p.m.sb(0xA0, 0x0F);
                p.m.sw(0xA2, 0);
            }
            2 => self.prop_b7_near(p),
            _ => self.prop_free(p, i), // 0x76D86
        }
    }

    /// 0x76D06. The camera test is first, so a camera past 0xB80 does not count the timer.
    fn prop_b7_near(&mut self, p: &mut Prop) {
        // cmpi.w #$B80, G.$744; bgt (signed). Equal still runs.
        if (self.cam.s2.cam_x as i16) > 0x0B80 {
            p.b.mode = p.b.mode.wrapping_add(2); // 0x76D38
            return;
        }
        let n = p.m.b(0xA0).wrapping_sub(1);
        p.m.sb(0xA0, n);
        if n != 0 {
            return; // 0x76D36
        }
        self.prop_b7_near_paint(p.m.w(0xA2)); // 0x76D3E, before the index moves
        p.m.sb(0xA0, 0x18);
        let index = p.m.w(0xA2).wrapping_add(2);
        p.m.sw(0xA2, index);
        if index == 0x0A {
            p.m.sb(0xA0, 0x60);
            p.m.sw(0xA2, 0);
        }
    }

    /// 0x76D3E. The word at `NEAR + index` is the byte offset of four code words, written as two
    /// columns of two rows (code only, the attribute stays).
    fn prop_b7_near_paint(&mut self, index: u16) {
        let off = self.assets.enemy.t.u16(NEAR, index as u32);
        let src = NEAR.wrapping_add(off as u32);
        let at = TilePos::at(NEAR_AT);
        for col in 0..2 {
            for row in 0..2 {
                let code = self.assets.enemy.t.u16(src, (col * 2 + row) as u32 * 2);
                self.prop_tile_code(at.right(col).down(row), code);
            }
        }
    }

    /// Var 4, mode table 0x76D8C.
    fn prop_b7_var4(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x76D94. Does not fall into mode 2. The first wait is 64 frames, not 15.
                p.b.mode = p.b.mode.wrapping_add(2);
                p.m.sb(0xA0, 0x40);
                p.m.sw(0xA2, 0);
            }
            2 => self.prop_b7_far(p),
            _ => self.prop_free(p, i), // 0x76D86
        }
    }

    /// 0x76DA4. The camera test is first.
    fn prop_b7_far(&mut self, p: &mut Prop) {
        // 0x76DA4: d0 = X + 0x40; cmp.w G.$7E4, d0; blt. Equal still runs.
        let limit = p.b.xi().wrapping_add(0x40);
        if limit < self.cam.s3.cam_x as i16 {
            p.b.mode = p.b.mode.wrapping_add(2); // 0x76D38
            return;
        }
        let n = p.m.b(0xA0).wrapping_sub(1);
        p.m.sb(0xA0, n);
        if n != 0 {
            return; // 0x76DDA
        }
        self.prop_b7_far_paint(p.m.w(0xA2)); // 0x76DDC, before the index moves
        p.m.sb(0xA0, 0x18);
        let index = p.m.w(0xA2).wrapping_add(4);
        p.m.sw(0xA2, index);
        if index == 0x18 {
            p.m.sb(0xA0, 0x60);
            p.m.sw(0xA2, 0);
        }
    }

    /// 0x76DDC. Two code words at `FAR + index`, written at 0x913D04 and 0x913D24.
    fn prop_b7_far_paint(&mut self, index: u16) {
        let src = FAR.wrapping_add(index as u32);
        let code0 = self.assets.enemy.t.u16(src, 0);
        let code1 = self.assets.enemy.t.u16(src, 2);
        let at = TilePos::at(FAR_AT);
        self.prop_tile_code(at, code0);
        self.prop_tile_code(at.right(1), code1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::enemy::ByteTables;
    use dino_assets::stage::Cell;
    use dino_assets::AssetPack;

    /// Offset words 0x0A, 0x12, 0x1A, 0x12, 0x1A and three pictures of four words.
    fn near_blob() -> Vec<u8> {
        let mut b = Vec::new();
        for w in [0x000Au16, 0x0012, 0x001A, 0x0012, 0x001A] {
            b.extend_from_slice(&w.to_be_bytes());
        }
        for p in 0..3u16 {
            for i in 0..4u16 {
                b.extend_from_slice(&(0x1100 + p * 0x10 + i).to_be_bytes());
            }
        }
        assert_eq!(b.len(), 0x22);
        b
    }

    /// Six pictures of two words. Picture `p`, word `i` is `0xA000 + p * 2 + i`.
    fn far_blob() -> Vec<u8> {
        let mut b = Vec::new();
        for p in 0..6u16 {
            for i in 0..2u16 {
                b.extend_from_slice(&(0xA000 + p * 2 + i).to_be_bytes());
            }
        }
        b
    }

    fn game() -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs: vec![(NEAR, near_blob()), (FAR, far_blob())] };
        Game::new(Box::leak(Box::new(a)))
    }

    fn cell2(g: &Game, addr: u32) -> (u16, u16) {
        let c = g.vram.scroll2[((addr - 0x90C000) / 4) as usize];
        (c.code, c.attr)
    }

    fn cell3(g: &Game, addr: u32) -> (u16, u16) {
        let c = g.vram.scroll3[((addr - 0x910000) / 4) as usize];
        (c.code, c.attr)
    }

    fn paint(g: &mut Game) {
        for c in &mut g.vram.scroll2 {
            *c = Cell { code: 0x1111, attr: 0xABCD };
        }
        for c in &mut g.vram.scroll3 {
            *c = Cell { code: 0x1111, attr: 0xABCD };
        }
    }

    fn live(g: &Game) -> Option<&Prop> {
        g.props.slots.iter().find(|s| s.b.live != 0 && s.b.k20 == 7 * 4)
    }

    /// The 65th update writes picture 0. Six pictures, then a pause of 0x60, then picture 0 again.
    #[test]
    fn var4_first_picture_is_on_the_65th_update() {
        let mut g = game();
        paint(&mut g);
        g.spawn_prop(7, 4, Some((0x100, 0, 0))).unwrap();
        g.pool_b_update();
        let p = live(&g).unwrap();
        assert_eq!((p.b.mode, p.m.b(0xA0), p.m.w(0xA2)), (2, 0x40, 0));
        assert_eq!(cell3(&g, FAR_AT).0, 0x1111, "mode 0 does not write");

        for _ in 0..63 {
            g.pool_b_update();
        }
        assert_eq!(cell3(&g, FAR_AT).0, 0x1111);
        g.pool_b_update();
        let p = live(&g).unwrap();
        assert_eq!((p.m.b(0xA0), p.m.w(0xA2)), (0x18, 4));
        assert_eq!(cell3(&g, FAR_AT), (0xA000, 0xABCD));
        assert_eq!(cell3(&g, FAR_AT + 0x20), (0xA001, 0xABCD));
        assert_eq!(cell3(&g, FAR_AT + 4).0, 0x1111, "the next row is not written");

        // Pictures 1..5 (indexes 4, 8, 0xC, 0x10, 0x14), one every 24 frames.
        for n in 1..6u16 {
            for _ in 0..24 {
                g.pool_b_update();
            }
            assert_eq!(cell3(&g, FAR_AT).0, 0xA000 + n * 2);
            assert_eq!(cell3(&g, FAR_AT + 0x20).0, 0xA001 + n * 2);
        }
        let p = live(&g).unwrap();
        assert_eq!((p.m.b(0xA0), p.m.w(0xA2)), (0x60, 0), "the sixth picture starts the long pause");

        for _ in 0..95 {
            g.pool_b_update();
        }
        assert_eq!(cell3(&g, FAR_AT).0, 0xA000 + 5 * 2, "the pause holds the last picture");
        g.pool_b_update();
        assert_eq!(cell3(&g, FAR_AT).0, 0xA000, "the cycle starts again at picture 0");
        assert_eq!(cell3(&g, FAR_AT).1, 0xABCD);
    }

    /// Scroll3 camera X == object X + 0x40 still runs. One pixel past that sets mode 4, and the next update frees.
    #[test]
    fn var4_camera_edge_is_signed_and_then_frees() {
        let mut g = game();
        paint(&mut g);
        g.spawn_prop(7, 4, Some((0x500, 0, 0))).unwrap();
        g.pool_b_update();
        g.cam.s3.cam_x = 0x540;
        for _ in 0..64 {
            g.pool_b_update();
        }
        assert_eq!(cell3(&g, FAR_AT).0, 0xA000, "X + 0x40 is still inside");
        assert_eq!(live(&g).unwrap().b.mode, 2);

        g.cam.s3.cam_x = 0x541;
        let timer = live(&g).unwrap().m.b(0xA0);
        g.pool_b_update();
        let p = live(&g).unwrap();
        assert_eq!(p.b.mode, 4);
        assert_eq!(p.m.b(0xA0), timer, "the timer is not counted once the camera has passed");
        g.pool_b_update();
        assert!(live(&g).is_none());
    }

    /// Var 0 writes its first 2×2 on the 16th update. Scroll2 camera 0xB80 still runs, 0xB81 sets mode 4.
    #[test]
    fn var0_writes_a_2x2_after_15_frames() {
        let mut g = game();
        paint(&mut g);
        g.spawn_prop(7, 0, None).unwrap();
        g.pool_b_update();
        assert_eq!((live(&g).unwrap().b.mode, live(&g).unwrap().m.b(0xA0)), (2, 0x0F));
        for _ in 0..14 {
            g.pool_b_update();
        }
        assert_eq!(cell2(&g, NEAR_AT).0, 0x1111);
        g.pool_b_update();
        assert_eq!(cell2(&g, NEAR_AT), (0x1100, 0xABCD));
        assert_eq!(cell2(&g, NEAR_AT + 4), (0x1101, 0xABCD));
        assert_eq!(cell2(&g, NEAR_AT + 0x40), (0x1102, 0xABCD));
        assert_eq!(cell2(&g, NEAR_AT + 0x44), (0x1103, 0xABCD));
        assert_eq!(live(&g).unwrap().m.w(0xA2), 2);

        g.cam.s2.cam_x = 0x0B80;
        g.pool_b_update();
        assert_eq!(live(&g).unwrap().b.mode, 2, "0xB80 still runs");
        g.cam.s2.cam_x = 0x0B81;
        g.pool_b_update();
        assert_eq!(live(&g).unwrap().b.mode, 4);
    }

    /// Var 0: 5 pictures every 24 frames, then 96 frame pause, then picture 0 again.
    #[test]
    fn var0_five_pictures_then_pause_of_96_frames() {
        let mut g = game();
        paint(&mut g);
        g.spawn_prop(7, 0, None).unwrap();
        // Update 1 (mode 0) + 15 updates: 16th update writes picture 0 (offset 0x0A -> 0x1100)
        for _ in 0..16 {
            g.pool_b_update();
        }
        assert_eq!(cell2(&g, NEAR_AT).0, 0x1100);

        // Pictures 1..4 (indexes 2, 4, 6, 8), one every 24 frames
        let expected = [0x1110u16, 0x1120, 0x1110, 0x1120];
        for exp in expected {
            for _ in 0..24 {
                g.pool_b_update();
            }
            assert_eq!(cell2(&g, NEAR_AT).0, exp);
        }
        let p = live(&g).unwrap();
        assert_eq!((p.m.b(0xA0), p.m.w(0xA2)), (0x60, 0), "fifth picture starts 96-frame pause");

        for _ in 0..95 {
            g.pool_b_update();
        }
        assert_eq!(cell2(&g, NEAR_AT).0, 0x1120, "pause holds last picture");
        g.pool_b_update();
        assert_eq!(cell2(&g, NEAR_AT).0, 0x1100, "cycle restarts at picture 0");
    }
}
