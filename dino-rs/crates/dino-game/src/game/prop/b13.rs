//! Pool B class 13, the burning huts of Episode 5 area 1 (0x79218).
//!
//! Nine variant writers, dispatched by the word at +$26 through the table at 0x7925E. Eight are
//! placed. Variant 2 is a real entry and is not placed. Variants 6 and 0 share the anchor
//! (0xE20, 0x200) at trigger 0xC80 (records 0xAD31E then 0xAD32C). They write different cells.
//! Both have to run.
//!
//! Mode 0 (0x7922C) only adds 2 to the mode. Mode 2 (0x79232) frees on the next update once
//! `(X + 0x100) <` the scroll2 camera (G.$744, signed `blt`). Otherwise, on a frame G.$7AE
//! differs from G.$7AF, the variant writes code words. The attribute stays. Mode 4 (0x796E0)
//! is `jmp 0x4938`. No sprite, no random number, no sound, no object byte besides the mode.
//!
//! The patches are not rectangles. Each [`Run`] is one straight line of the unrolled writer
//! (a `+4` down a column, or a `+0x40` across columns). A later variant replaces cells an
//! earlier one left behind: the tiles stay after the object is freed. Those pairs are never
//! live together (the earlier object has already passed its camera test):
//! variant 4 with 0xC (23 cells), variant 6 with 0xE (35), variant 8 with 0x10 (17).

use super::Prop;
use super::tile_block::TilePos;
use crate::game::Game;

/// One straight run of code words. `n` words from `at`, each `step` after the previous.
/// The step is not applied after the last word (the next run names its own address).
#[derive(Clone, Copy)]
struct Run {
    at: u32,
    n: u16,
    step: u32,
}

/// 0x79270. Placed at (0xE20, 0x200), trigger 0xC80, the second of the two records there. 88 words.
const VAR_0: &[Run] = &[
    Run { at: 0x90D884, n: 2, step: 0x40 },
    Run { at: 0x90D8C8, n: 2, step: 0x4 },
    Run { at: 0x90D904, n: 3, step: 0x4 },
    Run { at: 0x90D944, n: 5, step: 0x4 },
    Run { at: 0x90D984, n: 5, step: 0x4 },
    Run { at: 0x90D9C4, n: 7, step: 0x4 },
    Run { at: 0x90DA04, n: 7, step: 0x4 },
    Run { at: 0x90DA44, n: 9, step: 0x4 },
    Run { at: 0x90DA8C, n: 7, step: 0x4 },
    Run { at: 0x90DACC, n: 9, step: 0x4 },
    Run { at: 0x90DB14, n: 7, step: 0x4 },
    Run { at: 0x90DB54, n: 9, step: 0x4 },
    Run { at: 0x90DB9C, n: 7, step: 0x4 },
    Run { at: 0x90DBDC, n: 9, step: 0x4 },
];

/// 0x796E6. Not placed in Episode 5. The `moveq #$5` loop is 6 writes, and the write after it
/// lands on the same `+0x40` step, so the first run is 7. 68 words.
const VAR_2: &[Run] = &[
    Run { at: 0x90F000, n: 7, step: 0x40 },
    Run { at: 0x90F184, n: 2, step: 0x4 },
    Run { at: 0x90F1C0, n: 3, step: 0x4 },
    Run { at: 0x90F200, n: 5, step: 0x4 },
    Run { at: 0x90F240, n: 5, step: 0x4 },
    Run { at: 0x90F280, n: 7, step: 0x4 },
    Run { at: 0x90F2C0, n: 7, step: 0x4 },
    Run { at: 0x90F300, n: 9, step: 0x4 },
    Run { at: 0x90F348, n: 7, step: 0x4 },
    Run { at: 0x90F388, n: 9, step: 0x4 },
    Run { at: 0x90F3D0, n: 7, step: 0x4 },
];

/// 0x79A62. Placed at (0xD20, 0x200), trigger 0xB90. 51 words.
const VAR_4: &[Run] = &[
    Run { at: 0x90E480, n: 2, step: 0x4 },
    Run { at: 0x90E4C0, n: 2, step: 0x4 },
    Run { at: 0x90E500, n: 2, step: 0x4 },
    Run { at: 0x90E540, n: 2, step: 0x4 },
    Run { at: 0x90E580, n: 2, step: 0x4 },
    Run { at: 0x90E5C0, n: 2, step: 0x4 },
    Run { at: 0x90E600, n: 2, step: 0x4 },
    Run { at: 0x90E640, n: 2, step: 0x4 },
    Run { at: 0x90E680, n: 4, step: 0x4 },
    Run { at: 0x90E6C0, n: 4, step: 0x4 },
    Run { at: 0x90E700, n: 6, step: 0x4 },
    Run { at: 0x90E740, n: 6, step: 0x4 },
    Run { at: 0x90E780, n: 8, step: 0x4 },
    Run { at: 0x90E7C4, n: 7, step: 0x4 },
];

/// 0x79D1E. Placed at (0xE20, 0x200), trigger 0xC80, the first of the two records there. 87 words.
const VAR_6: &[Run] = &[
    Run { at: 0x90E804, n: 9, step: 0x4 },
    Run { at: 0x90E84C, n: 7, step: 0x4 },
    Run { at: 0x90E88C, n: 9, step: 0x4 },
    Run { at: 0x90E8D4, n: 7, step: 0x4 },
    Run { at: 0x90E914, n: 9, step: 0x4 },
    Run { at: 0x90E95C, n: 7, step: 0x4 },
    Run { at: 0x90E99C, n: 8, step: 0x4 },
    Run { at: 0x90E9E4, n: 6, step: 0x4 },
    Run { at: 0x90EA24, n: 6, step: 0x4 },
    Run { at: 0x90EA6C, n: 4, step: 0x4 },
    Run { at: 0x90EAAC, n: 4, step: 0x4 },
    Run { at: 0x90EAF4, n: 2, step: 0x4 },
    Run { at: 0x90EB34, n: 2, step: 0x4 },
    Run { at: 0x90EB78, n: 2, step: 0x38 },
    Run { at: 0x90EBB4, n: 2, step: 0x4 },
    Run { at: 0x90EBF0, n: 3, step: 0x4 },
];

/// 0x7A18E. Placed at (0xF20, 0x200), trigger 0xD80. Column runs (`adda.w #$40` inside `dbra`),
/// and the row base moves through a2. 96 words.
const VAR_8: &[Run] = &[
    Run { at: 0x90EC40, n: 7, step: 0x40 },
    Run { at: 0x90EC44, n: 15, step: 0x40 },
    Run { at: 0x90ECC8, n: 13, step: 0x40 },
    Run { at: 0x90ECCC, n: 13, step: 0x40 },
    Run { at: 0x90EC30, n: 16, step: 0x40 },
    Run { at: 0x90EC34, n: 16, step: 0x40 },
    Run { at: 0x90EC38, n: 16, step: 0x40 },
];

/// 0x7A52A. Placed at (0x1020, 0x100), trigger 0xE80. 91 words.
const VAR_A: &[Run] = &[
    Run { at: 0x90E004, n: 15, step: 0x40 },
    Run { at: 0x90E008, n: 14, step: 0x40 },
    Run { at: 0x90E00C, n: 14, step: 0x40 },
    Run { at: 0x90E030, n: 16, step: 0x40 },
    Run { at: 0x90E034, n: 16, step: 0x40 },
    Run { at: 0x90E038, n: 16, step: 0x40 },
];

/// 0x7A8B4. Placed at (0x1120, 0x100), trigger 0xF80. The first rows start at `a0 + 0x140`
/// and `a2 + 0x180` (0x7A8CC, 0x7A8E0, 0x7A8F4). 79 words.
const VAR_C: &[Run] = &[
    Run { at: 0x90E544, n: 11, step: 0x40 },
    Run { at: 0x90E588, n: 10, step: 0x40 },
    Run { at: 0x90E58C, n: 10, step: 0x40 },
    Run { at: 0x90E430, n: 16, step: 0x40 },
    Run { at: 0x90E434, n: 16, step: 0x40 },
    Run { at: 0x90E438, n: 16, step: 0x40 },
];

/// 0x7ABEA. Placed at (0x1220, 0x100), trigger 0x1080. 96 words.
const VAR_E: &[Run] = &[
    Run { at: 0x90E804, n: 16, step: 0x40 },
    Run { at: 0x90E808, n: 16, step: 0x40 },
    Run { at: 0x90E80C, n: 16, step: 0x40 },
    Run { at: 0x90E830, n: 16, step: 0x40 },
    Run { at: 0x90E834, n: 16, step: 0x40 },
    Run { at: 0x90E838, n: 16, step: 0x40 },
];

/// 0x7AF74. Placed at (0x1320, 0x100), trigger 0x1180. `moveq #$3` is 4 columns. 24 words.
const VAR_10: &[Run] = &[
    Run { at: 0x90EC04, n: 4, step: 0x40 },
    Run { at: 0x90EC08, n: 4, step: 0x40 },
    Run { at: 0x90EC0C, n: 4, step: 0x40 },
    Run { at: 0x90EC30, n: 4, step: 0x40 },
    Run { at: 0x90EC34, n: 4, step: 0x40 },
    Run { at: 0x90EC38, n: 4, step: 0x40 },
];

#[derive(Clone, Copy)]
struct Patch {
    var: u16,
    /// Four picture pointers, indexed by G.$7AE (0, 4, 8, 12).
    table: u32,
    runs: &'static [Run],
}

const PATCHES: &[Patch] = &[
    Patch { var: 0x00, table: 0x79400, runs: VAR_0 },
    Patch { var: 0x02, table: 0x79812, runs: VAR_2 },
    Patch { var: 0x04, table: 0x79B5E, runs: VAR_4 },
    Patch { var: 0x06, table: 0x79EAE, runs: VAR_6 },
    Patch { var: 0x08, table: 0x7A21A, runs: VAR_8 },
    Patch { var: 0x0A, table: 0x7A5A4, runs: VAR_A },
    Patch { var: 0x0C, table: 0x7A93A, runs: VAR_C },
    Patch { var: 0x0E, table: 0x7AC64, runs: VAR_E },
    Patch { var: 0x10, table: 0x7AFEE, runs: VAR_10 },
];

fn patch_of(var: u16) -> Option<Patch> {
    PATCHES.iter().copied().find(|p| p.var == var)
}

impl Game {
    /// 0x79218.
    pub fn prop_b13(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => p.b.mode = p.b.mode.wrapping_add(2), // 0x7922C
            2 => {
                // 0x79232. The variant jump is only reached when the clock differs.
                if self.prop_b13_due(p) {
                    match patch_of(p.b.var26) {
                        Some(patch) => self.prop_b13_paint(patch),
                        None => self.prop_free(p, i),
                    }
                }
            }
            _ => self.prop_free(p, i), // 0x796E0
        }
    }

    /// 0x79232. The clock bytes are the ones `tile_anim_tick` left (it runs after the props).
    fn prop_b13_due(&self, p: &mut Prop) -> bool {
        let x = p.b.xi().wrapping_add(0x100);
        if x < self.cam.s2.cam_x as i16 {
            p.b.mode = p.b.mode.wrapping_add(2); // 0x79258
            return false;
        }
        let (now, prev) = self.prop_tile_phase();
        now != prev // 0x79244
    }

    /// The variant routine. Code words only (`move.w (a1)+,(a0)`). The attribute is left alone.
    fn prop_b13_paint(&mut self, patch: Patch) {
        let (phase, _) = self.prop_tile_phase();
        let mut src = self.assets.enemy.t.u32(patch.table, u32::from(phase)) & 0x00FF_FFFF;
        for run in patch.runs {
            let mut addr = run.at;
            for k in 0..run.n {
                let code = self.assets.enemy.t.u16(src, 0);
                src = src.wrapping_add(2);
                self.prop_tile_code(TilePos::at(addr), code);
                if k + 1 != run.n {
                    addr = addr.wrapping_add(run.step);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::stage::Cell;
    use dino_assets::AssetPack;

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

    /// Pointer table of 4 longs at `table`, then pictures of `words_n` words.
    /// Word `k` of picture `p` is `base + p * words_n + k`.
    fn pics(table: u32, words_n: usize, base: u16) -> (u32, Vec<u8>) {
        let mut data = Vec::new();
        let pic0 = table + 16;
        for p in 0..4u32 {
            data.extend_from_slice(&be32(pic0 + p * (words_n as u32) * 2));
        }
        for p in 0..4u16 {
            let row: Vec<u16> = (0..words_n as u16).map(|k| base + p * words_n as u16 + k).collect();
            data.extend_from_slice(&words(&row));
        }
        (table, data)
    }

    fn game() -> Game {
        let mut pack = AssetPack::default();
        // Distinct bases, so a shared cell shows which variant wrote it last.
        pack.enemy.t.blobs = vec![
            pics(0x79400, 88, 0x1000),
            pics(0x79812, 68, 0x1800),
            pics(0x79B5E, 51, 0x2000),
            pics(0x79EAE, 87, 0x2800),
            pics(0x7A21A, 96, 0x3000),
            pics(0x7A5A4, 91, 0x3800),
            pics(0x7A93A, 79, 0x4000),
            pics(0x7AC64, 96, 0x4800),
            pics(0x7AFEE, 24, 0x5000),
        ];
        Game::new(Box::leak(Box::new(pack)))
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

    fn changed(g: &Game) -> usize {
        g.vram.scroll2.iter().filter(|c| c.code != 0x1111).count()
    }

    fn arm(g: &mut Game) {
        g.cam.s2.phase72 = 4;
        g.cam.s2.prev73 = 0;
        g.cam.s2.cam_x = 0;
    }

    /// Picture 1 (phase 4) starts at `base + words`.
    fn pic1(base: u16, words_n: u16, index: u16) -> u16 {
        base + words_n + index
    }

    #[test]
    fn mode_0_does_not_write_and_the_next_update_paints_picture_1() {
        let mut g = game();
        paint_all(&mut g);
        arm(&mut g);
        g.spawn_prop(13, 0, Some((0xE20, 0x200, 0))).unwrap();
        g.pool_b_update();
        assert_eq!(g.props.slots.iter().find(|s| s.b.live != 0).unwrap().b.mode, 2);
        assert_eq!(changed(&g), 0, "mode 0 does not write");

        g.pool_b_update();
        // 0x79286, then +0x40, then the +4 run at 0x90D8C8. The cell between them stays.
        assert_eq!(cell(&g, 0x90D884), (pic1(0x1000, 88, 0), 0xABCD));
        assert_eq!(cell(&g, 0x90D8C4).0, pic1(0x1000, 88, 1));
        assert_eq!(cell(&g, 0x90D8C8).0, pic1(0x1000, 88, 2));
        assert_eq!(cell(&g, 0x90D888).0, 0x1111, "the +0x40 step skips this row");
        assert_eq!(changed(&g), 88);
        assert_eq!(g.props.slots.iter().find(|s| s.b.live != 0).unwrap().b.live, 1);

        g.cam.s2.prev73 = 4;
        g.vram.scroll2[((0x90D884 - 0x90C000) / 4) as usize].code = 0x1111;
        g.pool_b_update();
        assert_eq!(cell(&g, 0x90D884).0, 0x1111, "equal phases do not write");

        g.cam.s2.phase72 = 8;
        g.pool_b_update();
        assert_eq!(cell(&g, 0x90D884).0, 0x1000 + 2 * 88, "phase 8 is picture 2");
    }

    #[test]
    fn every_variant_writes_its_own_count_and_the_anchors() {
        // (var, words, base, samples of (address, word index), one address that must stay blank)
        let cases: &[(u16, u16, u16, &[(u32, u16)], u32)] = &[
            (0x00, 88, 0x1000, &[(0x90D884, 0), (0x90D8C4, 1), (0x90D8C8, 2)], 0x90D888),
            (0x02, 68, 0x1800, &[(0x90F000, 0), (0x90F140, 5), (0x90F180, 6)], 0x90F004),
            (0x04, 51, 0x2000, &[(0x90E480, 0), (0x90E484, 1)], 0x90E488),
            (0x06, 87, 0x2800, &[(0x90E804, 0), (0x90E808, 1)], 0x90E828),
            (0x08, 96, 0x3000, &[(0x90EC40, 0), (0x90EDC0, 6), (0x90EC44, 7)], 0x90EC48),
            (0x0A, 91, 0x3800, &[(0x90E004, 0)], 0x90E010),
            (0x0C, 79, 0x4000, &[(0x90E544, 0)], 0x90E404),
            (0x0E, 96, 0x4800, &[(0x90E804, 0)], 0x90E810),
            (0x10, 24, 0x5000, &[(0x90EC04, 0), (0x90ECC4, 3), (0x90EC08, 4)], 0x90EC10),
        ];
        for &(var, n, base, samples, gap) in cases {
            let mut g = game();
            paint_all(&mut g);
            arm(&mut g);
            g.spawn_prop(13, var, Some((0, 0, 0))).unwrap();
            g.pool_b_update();
            g.pool_b_update();
            assert_eq!(changed(&g), n as usize, "var {var:#x} word count");
            for &(addr, index) in samples {
                assert_eq!(cell(&g, addr).0, pic1(base, n, index), "var {var:#x} at {addr:#x}");
                assert_eq!(cell(&g, addr).1, 0xABCD, "the attribute stays");
            }
            assert_eq!(cell(&g, gap).0, 0x1111, "var {var:#x} leaves {gap:#x}");
        }
    }

    /// The two records at X 0xE20 are different patches. Spawn order is the script order
    /// (variant 6, then variant 0). Neither cell set is dropped.
    #[test]
    fn the_two_records_at_the_same_anchor_both_paint() {
        let mut g = game();
        paint_all(&mut g);
        arm(&mut g);
        g.spawn_prop(13, 6, Some((0xE20, 0x200, 0))).unwrap();
        g.spawn_prop(13, 0, Some((0xE20, 0x200, 0))).unwrap();
        g.pool_b_update();
        assert_eq!(changed(&g), 0, "the first update is mode 0 for both");
        g.pool_b_update();
        assert_eq!(changed(&g), 87 + 88);
        assert_eq!(cell(&g, 0x90E804).0, pic1(0x2800, 87, 0), "variant 6");
        assert_eq!(cell(&g, 0x90D884).0, pic1(0x1000, 88, 0), "variant 0");
        assert_eq!(g.props.slots.iter().filter(|s| s.b.live != 0).count(), 2);
    }

    /// A later hut replaces cells the earlier one already wrote, after that object has freed.
    /// The shared cells are 0x90E544 (4 then 0xC), 0x90E804 (6 then 0xE), 0x90EC30 (8 then 0x10).
    #[test]
    fn a_later_variant_replaces_tiles_the_freed_one_left() {
        // (early var, early base, early words, early X, cull cam, late var, late base, shared addr)
        let pairs = [
            (4u16, 0x2000u16, 51u16, 0xD20i16, 0xE21u16, 0x0Cu16, 0x4000u16, 0x90E544u32),
            (6, 0x2800, 87, 0xE20, 0xF21, 0x0E, 0x4800, 0x90E804),
            (8, 0x3000, 96, 0xF20, 0x1021, 0x10, 0x5000, 0x90EC30),
        ];
        for (early, ebase, ewords, ex, cull, late, lbase, shared) in pairs {
            let mut g = game();
            paint_all(&mut g);
            arm(&mut g);
            g.spawn_prop(13, early, Some((ex, 0, 0))).unwrap();
            g.pool_b_update();
            g.pool_b_update();
            let left = cell(&g, shared).0;
            assert_ne!(left, 0x1111, "var {early:#x} writes {shared:#x}");
            assert_eq!(left, pic1(ebase, ewords, word_index(early, shared)));

            g.cam.s2.cam_x = cull;
            g.cam.s2.prev73 = 4;
            g.pool_b_update();
            g.pool_b_update();
            assert!(g.props.slots.iter().all(|s| s.b.live == 0), "var {early:#x} freed");
            assert_eq!(cell(&g, shared).0, left, "the tiles stay after the free");

            // The later object is still inside at this camera. Put the camera back so its own
            // X + 0x100 test does not free it before the paint.
            g.cam.s2.cam_x = 0;
            g.spawn_prop(13, late, Some((0, 0, 0))).unwrap();
            g.pool_b_update();
            assert_eq!(cell(&g, shared).0, left, "mode 0 of the later variant does not write");
            g.cam.s2.phase72 = 8;
            g.pool_b_update();
            let now = cell(&g, shared).0;
            assert_eq!(now, lbase + 2 * words_of(late) + word_index(late, shared), "var {late:#x} replaces it");
            assert_ne!(now, left);
        }
    }

    fn words_of(var: u16) -> u16 {
        match var {
            0x00 => 88,
            0x04 => 51,
            0x06 => 87,
            0x08 => 96,
            0x0C => 79,
            0x0E => 96,
            0x10 => 24,
            _ => panic!("{var:#x}"),
        }
    }

    fn word_index(var: u16, addr: u32) -> u16 {
        let runs = patch_of(var).unwrap().runs;
        let mut i = 0u16;
        for run in runs {
            for k in 0..run.n {
                let at = run.at.wrapping_add(run.step.wrapping_mul(u32::from(k)));
                if at == addr {
                    return i;
                }
                i += 1;
            }
        }
        panic!("var {var:#x} does not write {addr:#x}");
    }

    #[test]
    fn the_camera_window_is_inclusive_and_then_frees() {
        let mut g = game();
        paint_all(&mut g);
        arm(&mut g);
        g.cam.s2.cam_x = 0x200;
        g.spawn_prop(13, 4, Some((0x100, 0, 0))).unwrap();
        g.pool_b_update();
        g.pool_b_update();
        assert_eq!(changed(&g), 51, "X + 0x100 is still inside");

        g.cam.s2.cam_x = 0x201;
        g.cam.s2.prev73 = 0;
        g.vram.scroll2[((0x90E480 - 0x90C000) / 4) as usize].code = 0x1111;
        g.pool_b_update();
        let p = g.props.slots.iter().find(|s| s.b.live != 0).unwrap();
        assert_eq!(p.b.mode, 4);
        assert_eq!(cell(&g, 0x90E480).0, 0x1111, "the cull frame does not write");
        g.pool_b_update();
        assert!(g.props.slots.iter().all(|s| s.b.live == 0));
    }

    /// The jump at 0x7924A is only taken when the phases differ. An equal clock leaves the object.
    #[test]
    fn an_unknown_variant_frees_only_when_the_clock_would_jump() {
        let mut g = game();
        arm(&mut g);
        g.cam.s2.prev73 = 4;
        g.spawn_prop(13, 0x12, Some((0, 0, 0))).unwrap();
        g.pool_b_update();
        g.pool_b_update();
        assert_eq!(g.props.slots.iter().filter(|s| s.b.live != 0).count(), 1);

        g.cam.s2.prev73 = 0;
        g.pool_b_update();
        assert!(g.props.slots.iter().all(|s| s.b.live == 0));
    }

    fn rom_pack() -> Option<&'static AssetPack> {
        use std::path::Path;
        use std::sync::OnceLock;
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// The first paint reads the extracted picture. A short table panics here. This covers the
    /// init (mode 0, then the write), not a snapshot taken after the tiles were already there.
    #[test]
    fn the_rom_pictures_match_the_anchors() {
        let Some(pack) = rom_pack() else {
            eprintln!("skipped: no dino.zip");
            return;
        };
        // One variant at a time. Two of these anchors are also written by a later variant, so a
        // combined paint would hide the earlier word.
        let cases: &[(u16, u32, &[(u32, u16)])] = &[
            (0x00, 0x79400, &[(0x90D884, 0), (0x90D8C4, 1)]),
            (0x02, 0x79812, &[(0x90F000, 0), (0x90F180, 6)]),
            (0x04, 0x79B5E, &[(0x90E480, 0)]),
            (0x06, 0x79EAE, &[(0x90E804, 0)]),
            (0x08, 0x7A21A, &[(0x90EC40, 0), (0x90EC44, 7)]),
            (0x0A, 0x7A5A4, &[(0x90E004, 0)]),
            (0x0C, 0x7A93A, &[(0x90E544, 0)]),
            (0x0E, 0x7AC64, &[(0x90E804, 0)]),
            (0x10, 0x7AFEE, &[(0x90EC04, 0), (0x90EC08, 4)]),
        ];
        let mut g = Game::new(pack);
        for &(var, table, samples) in cases {
            // Read the picture before the updates. A live borrow of the tables cannot cross them.
            let pic = g.assets.enemy.t.u32(table, 4) & 0x00FF_FFFF;
            let want: Vec<(u32, u16)> = samples
                .iter()
                .map(|&(addr, index)| (addr, g.assets.enemy.t.u16(pic, u32::from(index) * 2)))
                .collect();
            paint_all(&mut g);
            arm(&mut g);
            g.spawn_prop(13, var, Some((0, 0, 0))).unwrap();
            g.pool_b_update();
            assert_eq!(cell(&g, samples[0].0).0, 0x1111, "var {var:#x} mode 0 does not write");
            g.pool_b_update();
            for &(addr, code) in &want {
                assert_eq!(cell(&g, addr).0, code, "var {var:#x} {addr:#x}");
                assert_eq!(cell(&g, addr).1, 0xABCD);
            }
            g.cam.s2.cam_x = 0x4000;
            g.pool_b_update();
            g.pool_b_update();
        }
    }
}
