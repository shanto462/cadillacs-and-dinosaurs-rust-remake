//! Pool B class 2, the swamp water of Episode 2 area 1 (0x7542C).
//!
//! Mode 0 (0x75440) sets mode 2, `+$A0 = 15`, `+$A2 = 0` and `+$A4 = 0`, then returns.
//! Mode 2 (0x75454) counts `+$A0` down. At 0 it reloads 24, steps the phase `+$A4`
//! through 0, 2, 4, 6, stores that phase in G.-$703C (0x7546C), then writes a window
//! of the 12-byte records at 0x7550E. Mode 4 and mode 6 are 0x75740 (`jmp 0x4938`).

use super::tile_block::{TileMode, TilePos};
use super::Prop;
use crate::game::Game;

/// 12-byte records (0x7550E): xlim word, flag word, dest long, source long. xlim 0xFFFF ends the list.
const RECORDS: u32 = 0x7550E;
/// Word offsets added to the source for phase 0, 2, 4, 6 (0x754FE). Indexed by the phase word.
const PHASE: u32 = 0x754FE;
/// Extra source offset, added only when the record flag is 8 (0x75506). Same index as `PHASE`.
const EXTRA: u32 = 0x75506;

impl Game {
    /// 0x7542C.
    pub fn prop_b2(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x75440. Does not fall into mode 2.
                p.b.mode = 2;
                p.m.sb(0xA0, 0x0F);
                p.m.sw(0xA2, 0);
                p.m.sw(0xA4, 0);
            }
            2 => self.prop_b2_run(p),
            _ => self.prop_free(p, i), // 0x75740
        }
    }

    /// 0x75454. The phase and G.-$703C are stored before the search, so a search that hits the
    /// end marker still keeps the new phase. The marker during the search (0x754EC) bumps the
    /// mode and returns; the marker peeked after a write (0x754DE) only returns.
    fn prop_b2_run(&mut self, p: &mut Prop) {
        let n = p.m.b(0xA0).wrapping_sub(1); // subq.b #1,+$A0
        p.m.sb(0xA0, n);
        if n != 0 {
            return; // 0x75458 bne 0x754EA
        }
        p.m.sb(0xA0, 0x18);
        let phase = p.m.w(0xA4).wrapping_add(2) & 6;
        p.m.sw(0xA4, phase);
        self.w.water_phase = phase; // 0x7546C move.w +$A4,-$703C(a5)
        self.prop_b2_window(p, phase);
    }

    /// 0x75472. Records whose X limit is left of camera X - 0x30 are skipped (`+$A2` advances by 12
    /// and is not wound back). From the first record still ahead, up to 13 records are written
    /// (`moveq #$C,d6` then `dbra`). Only the first record supplies dest and source. Later records
    /// keep the address the writer left and add 0xA8 to the source. `+$A6` at 0x754B0 is overwritten
    /// at 0x754B4 and has no effect.
    fn prop_b2_window(&mut self, p: &mut Prop, phase: u16) {
        let cam = (self.cam.s2.cam_x as i16).wrapping_sub(0x30); // 0x75480 subi.w #$30
        let start = loop {
            let index = p.m.w(0xA2);
            let xlim = self.assets.enemy.t.i16(RECORDS, index as u32);
            if xlim < 0 {
                p.b.mode = p.b.mode.wrapping_add(2); // 0x754EC
                return;
            }
            // cmp.w xlim, cam; ble uses the record. Both words are signed.
            if cam <= xlim {
                break index;
            }
            p.m.sw(0xA2, index.wrapping_add(12)); // 0x75490
        };
        let mut dest = self.assets.enemy.t.u32(RECORDS, start as u32 + 4) & 0xFF_FFFF;
        let mut src = (self.assets.enemy.t.u32(RECORDS, start as u32 + 8) & 0xFF_FFFF)
            .wrapping_add(self.assets.enemy.t.u16(PHASE, phase as u32) as u32);
        let mut rec = start;
        for _ in 0..13 {
            let flag = self.assets.enemy.t.u16(RECORDS, rec as u32 + 2);
            if flag == 8 {
                // 0x754BA. The add stays on the source for the records after this one.
                src = src.wrapping_add(self.assets.enemy.t.u16(EXTRA, phase as u32) as u32);
            }
            dest = self.prop_b2_blit(flag, dest, src);
            src = src.wrapping_add(0xA8); // 0x754D0
            rec = rec.wrapping_add(12);
            if self.assets.enemy.t.i16(RECORDS, rec as u32) < 0 {
                return; // 0x754DE
            }
        }
    }

    /// Writers 0x755F4 (flag 0), 0x75656 (flag 4) and 0x756BE (flag 8), selected by the long table
    /// 0x754F2. Each writes tile codes only. Flag 4 is the scroll2 ring: two columns, then the
    /// column at 0x90F020, and the running dest becomes 0x90F060.
    fn prop_b2_blit(&mut self, flag: u16, dest: u32, src: u32) -> u32 {
        let dest = dest & 0xFF_FFFF;
        let src = src & 0xFF_FFFF;
        match flag {
            0 => {
                self.prop_tile_block(TilePos::at(dest), 3, 7, src, TileMode::Codes);
                dest.wrapping_add(0xC0)
            }
            4 => {
                self.prop_tile_block(TilePos::at(dest), 2, 7, src, TileMode::Codes);
                self.prop_tile_block(TilePos::at(0x90F020), 1, 7, src.wrapping_add(28), TileMode::Codes);
                0x90F060
            }
            8 => {
                self.prop_tile_block(TilePos::at(dest), 4, 7, src, TileMode::Codes);
                dest.wrapping_add(0x100)
            }
            _ => dest,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::enemy::ByteTables;
    use dino_assets::stage::Cell;
    use dino_assets::AssetPack;

    fn words(n: usize) -> Vec<u8> {
        (0..n as u16).flat_map(|i| (i + 1).to_be_bytes()).collect()
    }

    fn rec(xlim: u16, flag: u16, dest: u32, src: u32) -> [u8; 12] {
        let mut r = [0u8; 12];
        r[0..2].copy_from_slice(&xlim.to_be_bytes());
        r[2..4].copy_from_slice(&flag.to_be_bytes());
        r[4..8].copy_from_slice(&dest.to_be_bytes());
        r[8..12].copy_from_slice(&src.to_be_bytes());
        r
    }

    /// Phase words 0, 0x10, 0x20, 0x30 and extra words 0, 4, 8, 12, then the records and 0xFFFF.
    /// Not the ROM table: the offsets are whatever the blob says.
    fn table(recs: &[[u8; 12]]) -> Vec<u8> {
        let mut b = Vec::new();
        for w in [0u16, 0x10, 0x20, 0x30, 0, 4, 8, 12] {
            b.extend_from_slice(&w.to_be_bytes());
        }
        for r in recs {
            b.extend_from_slice(r);
        }
        b.extend_from_slice(&0xFFFFu16.to_be_bytes());
        b
    }

    fn game(recs: &[[u8; 12]], tiles: usize) -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs: vec![(PHASE, table(recs)), (0x75746, words(tiles))] };
        Game::new(Box::leak(Box::new(a)))
    }

    fn cell(g: &Game, addr: u32) -> (u16, u16) {
        let c = g.vram.scroll2[((addr - 0x90C000) / 4) as usize];
        (c.code, c.attr)
    }

    fn paint(g: &mut Game, addr: u32) {
        g.vram.scroll2[((addr - 0x90C000) / 4) as usize] = Cell { code: 0x1111, attr: 0xABCD };
    }

    fn paint_block(g: &mut Game, addr: u32, cols: i32, rows: i32) {
        let at = TilePos::at(addr);
        for c in 0..cols {
            for r in 0..rows {
                paint(g, at.right(c).down(r).0);
            }
        }
    }

    fn live(g: &Game) -> Option<&Prop> {
        g.props.slots.iter().find(|s| s.b.live != 0 && s.b.k20 == 2 * 4)
    }

    /// Mode 0 does not write. The 16th update writes phase 2. Then every 24th update steps 4, 6, 0.
    #[test]
    fn the_first_picture_is_on_the_16th_update_then_every_24() {
        let dest = 0x90C400;
        let mut g = game(&[rec(0x100, 0, dest, 0x75746)], 80);
        paint_block(&mut g, dest, 3, 8);
        g.spawn_prop(2, 0, None).unwrap();
        g.pool_b_update();
        let p = live(&g).unwrap();
        assert_eq!((p.b.mode, p.m.b(0xA0), p.m.w(0xA2), p.m.w(0xA4)), (2, 0x0F, 0, 0));
        assert_eq!(cell(&g, dest).0, 0x1111, "mode 0 does not write");

        for _ in 0..14 {
            g.pool_b_update();
        }
        assert_eq!(live(&g).unwrap().m.b(0xA0), 1);
        assert_eq!(cell(&g, dest).0, 0x1111);

        g.pool_b_update();
        assert_eq!(g.w.water_phase, 2);
        assert_eq!(live(&g).unwrap().m.b(0xA0), 0x18);
        // Phase word 2 selects offset 0x10, so the first code is word 8 (value 9).
        let at = TilePos::at(dest);
        assert_eq!(cell(&g, at.0), (9, 0xABCD));
        assert_eq!(cell(&g, at.right(2).down(6).0), (9 + 20, 0xABCD));
        assert_eq!(cell(&g, at.down(7).0), (0x1111, 0xABCD), "row 7 is outside the 7 rows");

        let expect = [4u16, 6, 0];
        let off = [0x20u16, 0x30, 0];
        for (phase, byte_off) in expect.into_iter().zip(off) {
            for _ in 0..24 {
                g.pool_b_update();
            }
            assert_eq!(g.w.water_phase, phase);
            assert_eq!(cell(&g, dest).0, byte_off / 2 + 1);
        }
    }

    /// A record whose limit is left of camera X - 0x30 is skipped. Moving the camera back does not revisit it.
    #[test]
    fn records_left_of_the_camera_are_skipped_and_stay_skipped() {
        let (a, b) = (0x90C800, 0x90CC00);
        let mut g = game(&[rec(0x100, 0, a, 0x75746), rec(0x200, 0, b, 0x75746)], 160);
        paint_block(&mut g, a, 3, 7);
        paint_block(&mut g, b, 3, 7);
        g.cam.s2.cam_x = 0x131; // 0x131 - 0x30 = 0x101, past the first limit and not past the second
        g.spawn_prop(2, 0, None).unwrap();
        for _ in 0..16 {
            g.pool_b_update();
        }
        assert_eq!(live(&g).unwrap().m.w(0xA2), 12);
        assert_eq!(cell(&g, a).0, 0x1111, "the skipped record is not written");
        assert_eq!(cell(&g, b).0, 9);
        g.cam.s2.cam_x = 0;
        for _ in 0..24 {
            g.pool_b_update();
        }
        assert_eq!(live(&g).unwrap().m.w(0xA2), 12);
        assert_eq!(cell(&g, a).0, 0x1111, "a skipped record stays skipped");
        assert_ne!(cell(&g, b).0, 0x1111);
    }

    /// Flag 4 writes two columns at its dest, then the column at 0x90F020. The next record continues at 0x90F060.
    #[test]
    fn the_ring_wrap_record_writes_the_hardcoded_column() {
        let dest = 0x90FFA0;
        let decoy = 0x90D000;
        let mut g = game(&[rec(0x100, 4, dest, 0x75746), rec(0x200, 0, decoy, 0x75746)], 200);
        paint_block(&mut g, dest, 2, 7);
        paint_block(&mut g, 0x90F020, 1, 7);
        paint_block(&mut g, 0x90F060, 3, 7);
        paint_block(&mut g, decoy, 3, 7);
        g.spawn_prop(2, 0, None).unwrap();
        for _ in 0..16 {
            g.pool_b_update();
        }
        let at = TilePos::at(dest);
        assert_eq!(cell(&g, at.0).0, 9);
        assert_eq!(cell(&g, at.right(1).down(6).0).0, 9 + 13);
        // The 15th word of the picture (word index 8 + 14) is the first cell of 0x90F020.
        assert_eq!(cell(&g, 0x90F020).0, 9 + 14);
        assert_eq!(cell(&g, TilePos::at(0x90F020).down(6).0).0, 9 + 20);
        assert_eq!(cell(&g, decoy).0, 0x1111, "the next record's own dest is not used");
        // Next source is 0xA8 bytes on, plus the same phase offset 0x10: word index (0x10 + 0xA8) / 2 = 92.
        assert_eq!(cell(&g, 0x90F060).0, 93);
        assert_eq!(cell(&g, 0x90F020).1, 0xABCD);
    }

    /// Flag 8 adds the extra offset and writes 4 columns. Phase 2 uses 0x10 + 4.
    #[test]
    fn a_flag_8_record_adds_the_extra_offset_and_writes_four_columns() {
        let dest = 0x90E000;
        let mut g = game(&[rec(0x100, 8, dest, 0x75746)], 80);
        paint_block(&mut g, dest, 4, 8);
        g.spawn_prop(2, 0, None).unwrap();
        for _ in 0..16 {
            g.pool_b_update();
        }
        let at = TilePos::at(dest);
        // Word index of byte offset 0x14 is 10, value 11.
        assert_eq!(cell(&g, at.0), (11, 0xABCD));
        assert_eq!(cell(&g, at.right(3).down(6).0).0, 11 + 27);
        assert_eq!(cell(&g, at.down(7).0).0, 0x1111);
        assert_eq!(g.w.water_phase, 2);
    }

    /// `moveq #$C` writes records 0..12 and leaves 13 and 14 for a later camera position.
    #[test]
    fn at_most_13_records_are_written() {
        let recs: Vec<[u8; 12]> = (0..15u16)
            .map(|i| rec(0x10, 0, 0x90C000 + (i as u32) * 0xC0, 0x75746))
            .collect();
        let mut g = game(&recs, 15 * 0xA8);
        for i in 0..15i32 {
            paint_block(&mut g, 0x90C000 + (i as u32) * 0xC0, 3, 7);
        }
        g.spawn_prop(2, 0, None).unwrap();
        for _ in 0..16 {
            g.pool_b_update();
        }
        assert_ne!(cell(&g, 0x90C000).0, 0x1111);
        assert_ne!(cell(&g, 0x90C000 + 12 * 0xC0).0, 0x1111, "record 12 is the 13th write");
        assert_eq!(cell(&g, 0x90C000 + 13 * 0xC0).0, 0x1111, "record 13 waits");
        assert_eq!(cell(&g, 0x90C000 + 14 * 0xC0).0, 0x1111);
        assert_eq!(live(&g).unwrap().b.mode, 2);
    }

    /// The end marker during the search stores the phase, sets mode 4, and frees on the next update.
    #[test]
    fn the_end_marker_during_the_search_frees_on_the_next_update() {
        let mut g = game(&[], 4);
        paint(&mut g, 0x90C000);
        g.spawn_prop(2, 0, None).unwrap();
        for _ in 0..16 {
            g.pool_b_update();
        }
        let p = live(&g).unwrap();
        assert_eq!((p.b.mode, p.m.w(0xA4), g.w.water_phase), (4, 2, 2));
        assert_eq!(cell(&g, 0x90C000).0, 0x1111);
        g.pool_b_update();
        assert!(live(&g).is_none());
    }
}
