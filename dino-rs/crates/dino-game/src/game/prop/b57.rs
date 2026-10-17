//! Pool B class 57 (`+$20 = 0xE4`, handler 0x85B64): the floor labels of the lift of Episode 7 (area 1), and class 64
//! (`+$20 = 0x100`, handler 0x86B64): the four posts of the cage (M3-14).
//!
//! # B57 (0x85B64)
//!
//! Made at the start of the area by the spawn list (trigger Y 0x4000). Mode 0 (0x85B7A) sets the ring address `+$A0` to
//! 0x90C268 (scroll2 column 9, row 10), the height `+$C` to 0x3760 and the label counter `+$A4` to 0 and runs the per frame
//! part (0x85B8E) in the same frame. Every frame it compares its height (a signed word) with the camera Y G.$748: when the
//! camera has risen to it or above (`blt` returns while `+$C` < camera Y) it paints the next label (0x85BA8) and takes 0x100
//! from `+$C`, so **label k is painted when the camera Y is 0x3760 - 0x100 k or less** (one label a frame at most).
//!
//! A label is three glyph words from the table 0x85BF6 (`+$A4` counts by 2, the label is at 3 * `+$A4` bytes): the letter (0xA = L
//! or 0xB = B), then the dash and the two digits. The first word is -1 for the blank floors: nothing is painted. A glyph is a block of
//! 2 x 2 scroll2 cells (4 long words, 16 bytes at 0x85D6C + 16 * number, written by 0x85D46: two cells down, the next column,
//! two cells down; the address moves two columns on), a word of -1 moves two columns without painting. The dash (0x85BC2) is
//! one column of two cells (0x618F0013, 0x61800013). After the label the ring address moves 0x1000 on (16 rows: the four pages
//! of the map), back to 0x90C268 at 0x910000. The labels stay in the map until the stream of scroll2 writes over them.
//!
//! # B64 (0x86B64)
//!
//! Four objects (`+$26` = 0, 4, 8, 0xC), each a single frame record of the 12 byte loader 0x143E (compositions 1897 to
//! 1900, pointers 0x86BD4, records 0x86BE4 ..), drawn raw at the scroll1 scroll plus an offset: X = G.$6C4 + dx, Y = G.$7B8 + dy
//! (0x86BBE: (24, 184), (96, 56), (288, 56), (360, 184): the two front posts and the two back posts of the wooden frame).
//! Mode 0 (0x86B7A) loads the frame, sets the visible flag and the draw flags 0x10 and falls into mode 2 (0x86B9C), which
//! puts the position in and calls 0x49C0. Mode 4 frees it (0x86BCE, jmp 0x4938).

use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x85B64 pool B class 57 (jump table 0x85B70: mode 0 -> 0x85B7A, mode 2 -> 0x85B8E).
    pub fn prop_b57(&mut self, p: &mut Prop, _i: usize) {
        if p.b.mode == 0 {
            // 0x85B7A
            p.b.mode += 2;
            p.m.sl(0xA0, 0x90C268);
            p.b.set_yi(0x3760);
            p.m.sw(0xA4, 0);
        } else if p.b.mode != 2 {
            return; // the jump table has no other entry
        }
        // 0x85B8E
        if p.b.yi() < self.cam.s2.cam_y as i16 {
            return; // blt
        }
        self.prop_b57_label(p);
        p.b.set_yi(p.b.yi().wrapping_sub(0x100)); // 0x85B9C
        p.m.sw(0xA4, p.m.w(0xA4).wrapping_add(2));
    }

    /// 0x85BA8: paints the label `+$A4 / 2` at the ring address `+$A0` and moves the ring on.
    fn prop_b57_label(&mut self, p: &mut Prop) {
        let mut a0 = p.m.l(0xA0);
        let k = p.m.w(0xA4) as u32;
        let a2 = 0x85BF6u32.wrapping_add(k * 3); // d0 + 2 * d0 + the table: 3 words a label (+$A4 counts by 2)
        let first = self.assets.enemy.t.i16(a2, 0);
        a0 = self.prop_b57_glyph(a0, first);
        if first >= 0 {
            // 0x85BC2: the dash, one column of two cells, then the next column
            self.prop_b57_cell(a0, 0x618F, 0x0013);
            self.prop_b57_cell(a0 + 4, 0x6180, 0x0013);
            a0 += 0x40;
            a0 = self.prop_b57_glyph(a0, self.assets.enemy.t.i16(a2, 2)); // 0x85BD2
            self.prop_b57_glyph(a0, self.assets.enemy.t.i16(a2, 4)); // 0x85BD6
        }
        // 0x85BDA: the ring address of the next label
        let next = p.m.l(0xA0).wrapping_add(0x1000);
        p.m.sl(0xA0, if (next as i32) < 0x910000 { next } else { 0x90C268 });
    }

    /// 0x85D46: a glyph (number `g`, a negative number is blank) at `a0`: 2 columns of 2 cells from the table 0x85D6C. Returns the
    /// address two columns on.
    fn prop_b57_glyph(&mut self, a0: u32, g: i16) -> u32 {
        if g >= 0 {
            let src = 0x85D6Cu32 + 16 * g as u32; // lsl.w #4
            for k in 0..4u32 {
                let (code, attr) = (self.assets.enemy.t.u16(src, 4 * k), self.assets.enemy.t.u16(src, 4 * k + 2));
                // two longs, then +0x38 after the post increment: the next column
                self.prop_b57_cell(a0 + (k / 2) * 0x40 + (k % 2) * 4, code, attr);
            }
        }
        a0 + 0x80
    }

    fn prop_b57_cell(&mut self, a: u32, code: u16, attr: u16) {
        use super::tile_block::TilePos;
        self.prop_tile_put(TilePos::at(a), code, attr);
    }

    /// 0x86B64 pool B class 64 (jump table 0x86B72: mode 0 -> 0x86B7A, mode 2 -> 0x86B9C, mode 4 -> 0x86BCE).
    pub fn prop_b64(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode += 2; // 0x86B7A
                let v = p.b.var26 as u32; // 0, 4, 8, 0xC
                let rec = self.assets.enemy.t.u32(0x86BD4, v); // movea.l (a0, d0.w), a0
                let frame = self.assets.sprites.frame_at(rec);
                p.b.load_frame(&self.assets.sprites, frame); // jsr 0x143E
                p.b.visible = 1;
                p.b.draw_flags = 0x10;
                self.prop_b64_place(p, i);
            }
            2 => self.prop_b64_place(p, i),
            _ => self.prop_free(p, i), // 0x86BCE: jmp 0x4938
        }
    }

    /// 0x86B9C: the position from the scroll1 scroll and the offsets of the variant, then 0x49C0.
    fn prop_b64_place(&mut self, p: &mut Prop, i: usize) {
        let v = p.b.var26 as u32;
        let (dx, dy) = (self.assets.enemy.t.u16(0x86BBE, v), self.assets.enemy.t.u16(0x86BBE, v + 2));
        p.b.set_xi(self.cam.cam1_x.wrapping_add(dx) as i16);
        p.b.set_yi(self.cam.shake_scroll1_y.wrapping_add(dy) as i16);
        self.prop_draw_list(p, i); // jmp 0x49C0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    /// A game with synthetic tables: label k of the table 0x85BF6 is chosen by `labels` (3 words each) and glyph g has the cells
    /// (0x1000 + 16 g + n, 0x2000 + 16 g + n) for n = 0 .. 3.
    fn game(labels: &[[i16; 3]]) -> Game {
        let mut pack = AssetPack::default();
        let mut table = Vec::new();
        for l in labels {
            for w in l {
                table.extend_from_slice(&w.to_be_bytes());
            }
        }
        table.resize(0x150, 0xFF);
        let mut glyphs = Vec::new();
        for g in 0..12u16 {
            for n in 0..4u16 {
                glyphs.extend_from_slice(&(0x1000 + 16 * g + n).to_be_bytes());
                glyphs.extend_from_slice(&(0x2000 + 16 * g + n).to_be_bytes());
            }
        }
        pack.enemy.t.blobs = vec![(0x85BF6, table), (0x85D6C, glyphs)];
        Game::new(Box::leak(Box::new(pack)))
    }

    fn cell(g: &Game, col: usize, row: usize) -> (u16, u16) {
        let c = g.vram.scroll2[crate::game::vram::Vram::index2(col, row)];
        (c.code, c.attr)
    }

    fn make(g: &mut Game) -> usize {
        let h = g.spawn_prop(57, 0, None).unwrap();
        match crate::game::slot_of(h).unwrap() {
            crate::game::Slot::Prop(i) => i,
            _ => unreachable!(),
        }
    }

    /// The first label (L-15: the glyphs 0xA, 1, 5 with the dash between) is painted in the first update at the ring address
    /// 0x90C268 (column 9, row 10): the letter in the columns 9 and 10, the dash in 11, the digits in 12, 13 and 14, 15. Y falls by
    /// 0x100 and the ring moves 16 rows on.
    #[test]
    fn the_first_label_is_painted_in_the_first_update() {
        let mut g = game(&[[0xA, 1, 5]]);
        g.cam.s2.cam_y = 0x3700;
        let i = make(&mut g);
        g.pool_b_update();
        // the letter: 2 x 2 cells of the glyph 0xA from (9, 10)
        assert_eq!(cell(&g, 9, 10), (0x1000 + 16 * 0xA, 0x2000 + 16 * 0xA));
        assert_eq!(cell(&g, 9, 11).0, 0x1000 + 16 * 0xA + 1);
        assert_eq!(cell(&g, 10, 10).0, 0x1000 + 16 * 0xA + 2);
        assert_eq!(cell(&g, 10, 11).0, 0x1000 + 16 * 0xA + 3);
        // the dash: one column of two cells
        assert_eq!(cell(&g, 11, 10), (0x618F, 0x0013));
        assert_eq!(cell(&g, 11, 11), (0x6180, 0x0013));
        // the digits 1 and 5
        assert_eq!(cell(&g, 12, 10).0, 0x1000 + 16);
        assert_eq!(cell(&g, 14, 10).0, 0x1000 + 16 * 5);
        assert_eq!(cell(&g, 15, 11).0, 0x1000 + 16 * 5 + 3);
        let p = &g.props.slots[i];
        assert_eq!(p.b.yi(), 0x3660);
        assert_eq!(p.m.w(0xA4), 2);
        assert_eq!(p.m.l(0xA0), 0x90D268);
    }

    /// A label is painted when the camera Y has fallen to the height of the object (a signed compare), one label a frame at most,
    /// and a blank label (first word -1) paints nothing but moves the ring on; after the fourth page the ring starts over.
    #[test]
    fn labels_follow_the_camera_and_blank_ones_only_move_the_ring() {
        let mut g = game(&[[0xA, 1, 5], [-1, -1, -1], [0xB, 2, -1], [0xB, 3, 0], [0xA, 9, -1]]);
        g.cam.s2.cam_y = 0x3700;
        let i = make(&mut g);
        g.pool_b_update(); // label 0
        g.pool_b_update(); // the camera is above 0x3660: nothing
        assert_eq!(g.props.slots[i].m.w(0xA4), 2);
        g.cam.s2.cam_y = 0x3660;
        g.pool_b_update(); // label 1 (blank): no cell, ring and counter move
        assert_eq!(cell(&g, 9, 26), (0, 0), "the blank label paints nothing");
        assert_eq!(g.props.slots[i].m.w(0xA4), 4);
        assert_eq!(g.props.slots[i].m.l(0xA0), 0x90E268);
        g.cam.s2.cam_y = 0x3000; // far below: still one label a frame
        g.pool_b_update(); // label 2 at the ring address 0x90E268 = column 9, row 42: B-2 and a blank third glyph
        assert_eq!(cell(&g, 9, 42).0, 0x1000 + 16 * 0xB);
        assert_eq!(cell(&g, 12, 42).0, 0x1000 + 16 * 2);
        assert_eq!(g.props.slots[i].m.w(0xA4), 6);
        g.pool_b_update(); // label 3 at 0x90F268
        g.pool_b_update(); // label 4: the ring is back at 0x90C268
        assert_eq!(g.props.slots[i].m.w(0xA4), 10);
        assert_eq!(cell(&g, 9, 10).0, 0x1000 + 16 * 0xA, "the ring wrapped to the first page");
        assert_eq!(g.props.slots[i].m.l(0xA0), 0x90D268);
    }

    /// The cage posts (B64): four objects, one frame record each, placed from the scroll1 scroll plus the offsets of the variant,
    /// visible at once, in the draw list from the first update, and following the scroll.
    #[test]
    fn the_cage_posts_follow_the_scroll1_position() {
        let mut pack = AssetPack::default();
        let mut offsets = Vec::new();
        for (dx, dy) in [(24u16, 184u16), (96, 56), (288, 56), (360, 184)] {
            offsets.extend_from_slice(&dx.to_be_bytes());
            offsets.extend_from_slice(&dy.to_be_bytes());
        }
        let mut ptrs = Vec::new();
        for k in 0..4u32 {
            ptrs.extend_from_slice(&(0x86BE4 + 12 * k).to_be_bytes());
            let frame = dino_assets::sprites::Frame {
                rom_addr: 0x86BE4 + 12 * k,
                stride: 12,
                w0: 0,
                w1: 0,
                draw: dino_assets::sprites::FrameDraw::Comp { main: 0, overlay: None },
                param_a: 0,
                param_b: 0,
                kind: 0,
                duration: 1,
                last: false,
                body_box: 0,
                attack_box: 0,
                attach: 0,
                extra: [0; 4],
                next: 0,
            };
            pack.sprites.frames.push(frame);
            pack.sprites.frame_by_addr.insert(0x86BE4 + 12 * k, k);
        }
        pack.enemy.t.blobs = vec![(0x86BBE, offsets), (0x86BD4, ptrs)];
        let mut g = Game::new(Box::leak(Box::new(pack)));
        g.cam.cam1_x = 3;
        g.cam.shake_scroll1_y = 5;
        let hs: Vec<_> = [0u16, 4, 8, 0xC].iter().map(|&v| g.spawn_prop(64, v, None).unwrap()).collect();
        g.pool_b_update();
        let want = [(24, 184), (96, 56), (288, 56), (360, 184)];
        for (h, (dx, dy)) in hs.iter().zip(want) {
            let b = g.body(*h);
            assert_eq!((b.mode, b.visible, b.draw_flags), (2, 1, 0x10));
            assert_eq!((b.xi(), b.yi()), (3 + dx, 5 + dy));
        }
        assert_eq!(g.draw_list.len(), 4, "all four posts are in the draw list");
        g.cam.shake_scroll1_y = 9;
        g.draw_list.clear();
        g.pool_b_update();
        assert_eq!(g.body(hs[0]).yi(), 9 + 184);
        assert_eq!(g.draw_list.len(), 4);
    }
}
