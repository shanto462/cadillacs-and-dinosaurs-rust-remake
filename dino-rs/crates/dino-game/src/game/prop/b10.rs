//! Pool B class 10, the foreground fire strips of Episode 5 area 1 (0x785F0).
//!
//! Two objects. Variant 0 at (0xD20, 0x200, 0), trigger 0xB90, writes 13 columns of 3 code words
//! at scroll2 0x90D4B4 (0x7863E). Variant 4 at (0xF20, 0x200, 0), trigger 0xD80, writes the
//! irregular patch at 0x90DC24 (0x787F4). Both share mode 0 (0x78616), which only advances to
//! mode 2, and the free at 0x787C4 (`jmp 0x4938`). Mode 2 frees on the next update once
//! `(X + 0x100) <` the scroll2 camera (G.$744, signed `blt`). Otherwise, on a frame G.$7AE
//! differs from G.$7AF, it rewrites code words from the picture G.$7AE selects. The attribute
//! stays. Any other variant is freed: the pointer table at 0x78606 has only those two entries.
//! No sprite, no random number, no sound, no object timer.

use super::Prop;
use super::tile_block::{TileMode, TilePos};
use crate::game::Game;

/// 0x787F4. `(writes, tail)`: `writes - 1` rows of `addq.w #4`, then the last write, then
/// `adda.w tail` from that write (there is no extra `+4` before the tail). 32 words.
const VAR4: &[(u16, u32)] = &[
    (7, 0x28),
    (7, 0x30),
    (5, 0x30),
    (5, 0x38),
    (3, 0x38),
    (3, 0x40),
    (1, 0x40),
    (1, 0),
];

impl Game {
    /// 0x785F0.
    pub fn prop_b10(&mut self, p: &mut Prop, i: usize) {
        // 0x785F4 indexes 0x78606 by the variant word. Only 0 and 4 are real entries.
        if p.b.var26 != 0 && p.b.var26 != 4 {
            self.prop_free(p, i);
            return;
        }
        match p.b.mode {
            0 => p.b.mode = p.b.mode.wrapping_add(2), // 0x78616, shared by both variants
            2 => {
                if self.prop_b10_due(p) {
                    if p.b.var26 == 0 {
                        self.prop_b10_var0(); // 0x7863E
                    } else {
                        self.prop_b10_var4(); // 0x787F4
                    }
                }
            }
            _ => self.prop_free(p, i), // 0x787C4
        }
    }

    /// 0x7861C and 0x787D2. The clock bytes are the ones `tile_anim_tick` left (it runs after the props).
    fn prop_b10_due(&self, p: &mut Prop) -> bool {
        let x = p.b.xi().wrapping_add(0x100);
        if x < self.cam.s2.cam_x as i16 {
            p.b.mode = p.b.mode.wrapping_add(2); // 0x78638
            return false;
        }
        let (now, prev) = self.prop_tile_phase();
        now != prev // 0x7862A
    }

    /// Picture long for this frame's G.$7AE. The phase byte is the offset (0, 4, 8, 12). Not clamped.
    fn prop_b10_pic(&self, table: u32) -> u32 {
        let (phase, _) = self.prop_tile_phase();
        self.assets.enemy.t.u32(table, u32::from(phase)) & 0x00FF_FFFF
    }

    /// 0x7863E. `moveq #$c` runs 13 times: three rows, then `adda.w #$38` from the third, which
    /// lands on the next scroll2 column (step 0x40).
    fn prop_b10_var0(&mut self) {
        let pic = self.prop_b10_pic(0x78664);
        self.prop_tile_block(TilePos::at(0x90D4B4), 13, 3, pic, TileMode::Codes);
    }

    /// 0x787F4. The runs are not a rectangle.
    fn prop_b10_var4(&mut self) {
        let mut src = self.prop_b10_pic(0x78898);
        let mut addr = 0x90DC24u32;
        for &(n, tail) in VAR4 {
            self.prop_b10_span(&mut addr, &mut src, n, tail);
        }
    }

    /// `n` code words down, then `tail` added to the address of the last write (0x787F4).
    fn prop_b10_span(&mut self, addr: &mut u32, src: &mut u32, n: u16, tail: u32) {
        for k in 0..n {
            let code = self.assets.enemy.t.u16(*src, 0);
            *src = src.wrapping_add(2);
            self.prop_tile_code(TilePos::at(*addr), code);
            if k + 1 != n {
                *addr = addr.wrapping_add(4);
            }
        }
        *addr = addr.wrapping_add(tail);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use dino_assets::stage::Cell;

    fn be32(v: u32) -> [u8; 4] {
        v.to_be_bytes()
    }

    fn words(ws: &[u16]) -> Vec<u8> {
        let mut v = Vec::with_capacity(ws.len() * 2);
        for w in ws {
            v.extend_from_slice(&w.to_be_bytes());
        }
        v
    }

    /// Pointer table of 4 longs at `table`, pictures of `words_n` words packed just after it.
    /// Word `k` of picture `p` is `base + p * words_n + k`.
    fn pics(table: u32, words_n: usize, base: u16) -> (u32, Vec<u8>) {
        let mut data = Vec::new();
        let pic0 = table + 16;
        for p in 0..4u32 {
            data.extend_from_slice(&be32(pic0 + p * (words_n as u32) * 2));
        }
        for p in 0..4u16 {
            let row: Vec<u16> = (0..words_n as u16)
                .map(|k| base + p * words_n as u16 + k)
                .collect();
            data.extend_from_slice(&words(&row));
        }
        (table, data)
    }

    fn game() -> Game {
        let mut pack = AssetPack::default();
        pack.enemy.t.blobs = vec![pics(0x78664, 39, 0x1000), pics(0x78898, 32, 0x2000)];
        Game::new(Box::leak(Box::new(pack)))
    }

    fn cell(g: &Game, addr: u32) -> (u16, u16) {
        let c = g.vram.scroll2[((addr - 0x90C000) / 4) as usize];
        (c.code, c.attr)
    }

    fn paint_all(g: &mut Game) {
        for c in &mut g.vram.scroll2 {
            *c = Cell {
                code: 0x1111,
                attr: 0xABCD,
            };
        }
    }

    fn changed(g: &Game) -> usize {
        g.vram.scroll2.iter().filter(|c| c.code != 0x1111).count()
    }

    fn slot(g: &Game, class_var: u16) -> &Prop {
        g.props
            .slots
            .iter()
            .find(|s| s.b.live != 0 && s.b.var26 == class_var && s.b.k20 == 10 * 4)
            .unwrap()
    }

    /// Mode 0 does not write. The next update, with the phases different, writes picture 1
    /// (phase 4): 13 columns by 3 rows. Equal phases write nothing.
    #[test]
    fn variant_0_writes_thirteen_by_three_on_the_clock_edge() {
        let mut g = game();
        paint_all(&mut g);
        g.cam.s2.phase72 = 4;
        g.cam.s2.prev73 = 0;
        g.spawn_prop(10, 0, Some((0xD20, 0x200, 0))).unwrap();
        g.pool_b_update();
        assert_eq!(slot(&g, 0).b.mode, 2);
        assert_eq!(changed(&g), 0, "mode 0 does not write");

        g.pool_b_update();
        assert_eq!(cell(&g, 0x90D4B4), (0x1027, 0xABCD));
        assert_eq!(cell(&g, 0x90D4B4 + 8).0, 0x1029, "the third row");
        assert_eq!(
            cell(&g, 0x90D4C0).0,
            0x1111,
            "the fourth row of the column is not written"
        );
        assert_eq!(cell(&g, 0x90D4B4 + 12 * 0x40 + 8).0, 0x104D);
        assert_eq!(
            cell(&g, 0x90D4B4 + 13 * 0x40).0,
            0x1111,
            "column 13 is outside the strip"
        );
        assert_eq!(changed(&g), 39);

        g.cam.s2.prev73 = 4;
        cell_code(&mut g, 0x90D4B4, 0x1111);
        g.pool_b_update();
        assert_eq!(cell(&g, 0x90D4B4).0, 0x1111, "equal phases do not write");

        g.cam.s2.phase72 = 8;
        g.pool_b_update();
        assert_eq!(
            cell(&g, 0x90D4B4).0,
            0x1000 + 2 * 39,
            "phase 8 is picture 2"
        );
    }

    /// Variant 4 writes the 32-word patch. The gap after the first seven words stays blank.
    #[test]
    fn variant_4_writes_the_irregular_patch() {
        let mut g = game();
        paint_all(&mut g);
        g.cam.s2.phase72 = 4;
        g.cam.s2.prev73 = 12;
        g.spawn_prop(10, 4, Some((0xF20, 0x200, 0))).unwrap();
        g.pool_b_update();
        g.pool_b_update();
        // Picture 1 starts at 0x2000 + 32.
        assert_eq!(cell(&g, 0x90DC24).0, 0x2020);
        assert_eq!(cell(&g, 0x90DC3C).0, 0x2026);
        assert_eq!(
            cell(&g, 0x90DC40).0,
            0x1111,
            "the tail skips to the next run"
        );
        assert_eq!(cell(&g, 0x90DC64).0, 0x2027);
        assert_eq!(cell(&g, 0x90DDBC).0, 0x203E);
        assert_eq!(cell(&g, 0x90DDFC), (0x203F, 0xABCD));
        assert_eq!(changed(&g), 32);
    }

    /// X + 0x100 == camX still runs. One pixel past sets mode 4 with no write, then the next update frees.
    #[test]
    fn the_camera_window_is_inclusive_and_then_frees() {
        let mut g = game();
        paint_all(&mut g);
        g.cam.s2.phase72 = 0;
        g.cam.s2.prev73 = 4;
        g.cam.s2.cam_x = 0x200;
        g.spawn_prop(10, 0, Some((0x100, 0, 0))).unwrap();
        g.pool_b_update();
        g.pool_b_update();
        assert_eq!(changed(&g), 39, "X + 0x100 is still inside");
        assert_eq!(slot(&g, 0).b.mode, 2);

        g.cam.s2.cam_x = 0x201;
        g.cam.s2.prev73 = 0;
        cell_code(&mut g, 0x90D4B4, 0x1111);
        g.pool_b_update();
        assert_eq!(slot(&g, 0).b.mode, 4);
        assert_eq!(
            cell(&g, 0x90D4B4).0,
            0x1111,
            "the cull frame does not write"
        );
        g.pool_b_update();
        assert!(g.props.slots.iter().all(|s| s.b.live == 0));
    }

    /// A variant other than 0 or 4 is freed. The pointer table is not followed.
    #[test]
    fn an_unknown_variant_frees() {
        let mut g = game();
        g.spawn_prop(10, 8, Some((0, 0, 0))).unwrap();
        g.pool_b_update();
        assert!(g.props.slots.iter().all(|s| s.b.live == 0));
    }

    fn cell_code(g: &mut Game, addr: u32, code: u16) {
        g.vram.scroll2[((addr - 0x90C000) / 4) as usize].code = code;
    }
}
