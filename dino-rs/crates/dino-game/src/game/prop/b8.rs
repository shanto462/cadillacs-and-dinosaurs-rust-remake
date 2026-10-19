//! Pool B class 8, the swamp water surface of Episode 6 area 1 (0x76E0E).
//!
//! Mode 0 (0x76E22) sets mode 2, `+$A0 = 15`, `+$A2 = 0` and `+$A4 = 0`, then returns.
//! Mode 2 (0x76E36) counts `+$A0` down. At 0 it reloads 24, steps the phase `+$A4`
//! through 0, 2, 4, 6, stores that phase in G.-$703C (0x76E4E), then writes a window
//! of the 12-byte records at 0x76EDE. Mode 4 and mode 6 are 0x770F0 (`jmp 0x4938`).

use super::tile_block::TilePos;
use super::Prop;
use crate::game::Game;

/// Word offsets added to the source for phase 0, 2, 4, 6 (0x76ECE). Indexed by the phase word.
const PHASE: u32 = 0x76ECE;
/// Extra source offset, subtracted only when the record flag is 4 (0x76ED6). Same index as `PHASE`.
const EXTRA: u32 = 0x76ED6;
/// 12-byte records (0x76EDE): xlim word, flag word, dest long, source long. xlim 0xFFFF ends the list.
const RECORDS: u32 = 0x76EDE;

impl Game {
    /// 0x76E0E.
    pub fn prop_b8(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x76E22. Does not fall into mode 2.
                p.b.mode = 2;
                p.m.sb(0xA0, 0x0F);
                p.m.sw(0xA2, 0);
                p.m.sw(0xA4, 0);
            }
            2 => self.prop_b8_run(p),
            _ => self.prop_free(p, i), // 0x770F0
        }
    }

    /// 0x76E36. The phase and G.-$703C are stored before the search, so a search that hits the
    /// end marker still keeps the new phase. The marker during the search (0x76EC8) bumps the
    /// mode and returns; the marker peeked after a write (0x76EBC) only returns.
    fn prop_b8_run(&mut self, p: &mut Prop) {
        let n = p.m.b(0xA0).wrapping_sub(1); // subq.b #1,+$A0
        p.m.sb(0xA0, n);
        if n != 0 {
            return; // 0x76E3A bne 0x76EC6
        }
        p.m.sb(0xA0, 0x18); // move.b #$18,+$A0
        let phase = p.m.w(0xA4).wrapping_add(2) & 6;
        p.m.sw(0xA4, phase);
        self.w.water_phase = phase; // 0x76E4E move.w +$A4,-$703C(a5)
        self.prop_b8_window(p, phase);
    }

    /// 0x76E54. Records whose X limit is left of camera X - 0x30 are skipped (`+$A2` advances by 12
    /// and is not wound back). From the first record still ahead, up to 13 records are written
    /// (`moveq #$C,d6` then `dbra`). Only the first record supplies dest and source. Later records
    /// keep the address the writer left and add 0xC0 to the source.
    fn prop_b8_window(&mut self, p: &mut Prop, phase: u16) {
        let cam = (self.cam.s2.cam_x as i16).wrapping_sub(0x30); // 0x76E64 subi.w #$30
        let start = loop {
            let index = p.m.w(0xA2);
            let xlim = self.assets.enemy.t.i16(RECORDS, index as u32);
            if xlim < 0 {
                p.b.mode = p.b.mode.wrapping_add(2); // 0x76EC8
                return;
            }
            // cmp.w xlim, cam; ble uses the record. Both words are signed.
            if cam <= xlim {
                break index;
            }
            p.m.sw(0xA2, index.wrapping_add(12)); // 0x76E70
        };
        let mut dest = self.assets.enemy.t.u32(RECORDS, start as u32 + 4) & 0xFF_FFFF;
        let mut src = (self.assets.enemy.t.u32(RECORDS, start as u32 + 8) & 0xFF_FFFF)
            .wrapping_add(self.assets.enemy.t.u16(PHASE, phase as u32) as u32);
        let mut rec = start;
        for _ in 0..13 {
            let flag = self.assets.enemy.t.u16(RECORDS, rec as u32 + 2);
            if flag == 4 {
                // 0x76E9C. Subtract extra phase offset.
                src = src.wrapping_sub(self.assets.enemy.t.u16(EXTRA, phase as u32) as u32);
            }
            dest = self.prop_b8_blit(dest, src);
            src = src.wrapping_add(0xC0); // 0x76EB2
            rec = rec.wrapping_add(12);
            if self.assets.enemy.t.i16(RECORDS, rec as u32) < 0 {
                return; // 0x76EBC
            }
        }
    }

    /// Writer 0x76FB8. Writes 3 columns of 8 rows (rows 42 to 49) into scroll2.
    /// Each column writes 6 code words on page 2 (rows 42..47), steps +$FC4 to row 48 on page 3,
    /// then writes 2 code words (rows 48..49).
    /// Next column is +$40 from previous column start.
    /// After 3 columns (+0xC0), 0x76EA8 applies `andi.l #$FFEFFF`, wrapping column 64 back to 0.
    fn prop_b8_blit(&mut self, dest: u32, mut src: u32) -> u32 {
        for col in 0..3 {
            let col_base = dest.wrapping_add(col * 0x40);
            for r in 0..6 {
                let code = self.assets.enemy.t.u16(src, 0);
                self.prop_tile_code(TilePos::at(col_base.wrapping_add(r * 4)), code);
                src = src.wrapping_add(2);
            }
            let page3_base = col_base.wrapping_add(0xFD8);
            for r in 0..2 {
                let code = self.assets.enemy.t.u16(src, 0);
                self.prop_tile_code(TilePos::at(page3_base.wrapping_add(r * 4)), code);
                src = src.wrapping_add(2);
            }
        }
        dest.wrapping_add(0xC0) & 0xFFEFFF
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
        a.enemy.t = ByteTables { blobs: vec![(PHASE, table(recs)), (0x770F6, words(tiles))], ..Default::default() };
        Game::new(Box::leak(Box::new(a)))
    }

    fn cell(g: &Game, addr: u32) -> (u16, u16) {
        let c = g.vram.scroll2[((addr - 0x90C000) / 4) as usize];
        (c.code, c.attr)
    }

    fn paint(g: &mut Game, addr: u32) {
        g.vram.scroll2[((addr - 0x90C000) / 4) as usize] = Cell { code: 0x1111, attr: 0xABCD };
    }

    fn paint_col(g: &mut Game, col_base: u32) {
        for r in 0..6 {
            paint(g, col_base + r * 4);
        }
        for r in 0..2 {
            paint(g, col_base + 0xFD8 + r * 4);
        }
    }

    fn live(g: &Game) -> Option<&Prop> {
        g.props.slots.iter().find(|s| s.b.live != 0 && s.b.k20 == 8 * 4)
    }

    /// Mode 0 does not write. The 16th update writes phase 2. Then every 24th update steps 4, 6, 0.
    #[test]
    fn b8_first_picture_is_on_the_16th_update_then_every_24() {
        let dest = 0x90E968;
        let mut g = game(&[rec(0x100, 0, dest, 0x770F6)], 100);
        for col in 0..3 {
            paint_col(&mut g, dest + col * 0x40);
        }
        g.spawn_prop(8, 0, None).unwrap();
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
        assert_eq!(cell(&g, dest), (9, 0xABCD));
        // Rows 42..47 on page 2:
        assert_eq!(cell(&g, dest + 5 * 4), (9 + 5, 0xABCD));
        // Row 48 on page 3:
        assert_eq!(cell(&g, dest + 0xFD8), (9 + 6, 0xABCD));
        assert_eq!(cell(&g, dest + 0xFD8 + 4), (9 + 7, 0xABCD));

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

    /// Records left of the camera (cam_x - 0x30 > xlim) are skipped.
    #[test]
    fn b8_records_left_of_camera_are_skipped() {
        let (a, b) = (0x90E968, 0x90EA28);
        let mut g = game(&[rec(0x100, 0, a, 0x770F6), rec(0x200, 0, b, 0x770F6)], 160);
        for col in 0..3 {
            paint_col(&mut g, a + col * 0x40);
            paint_col(&mut g, b + col * 0x40);
        }
        g.cam.s2.cam_x = 0x131; // 0x131 - 0x30 = 0x101 > 0x100
        g.spawn_prop(8, 0, None).unwrap();
        for _ in 0..16 {
            g.pool_b_update();
        }
        assert_eq!(live(&g).unwrap().m.w(0xA2), 12);
        assert_eq!(cell(&g, a).0, 0x1111, "skipped record is not written");
        assert_eq!(cell(&g, b).0, 9);
    }

    /// Ring wrap wraps column 64 back to column 0.
    #[test]
    fn b8_ring_wrap_masks_dest() {
        let dest = 0x90EF68; // column 61
        let mut g = game(&[rec(0x100, 0, dest, 0x770F6)], 100);
        g.spawn_prop(8, 0, None).unwrap();
        for _ in 0..16 {
            g.pool_b_update();
        }
        // dest + 0xC0 = 0x90F028 & 0xFFEFFF = 0x90E028 (column 0).
        let next_dest = dest.wrapping_add(0xC0) & 0xFFEFFF;
        assert_eq!(next_dest, 0x90E028);
    }

    /// End marker frees prop on following update.
    #[test]
    fn b8_end_marker_sets_mode_4_and_frees() {
        let mut g = game(&[], 4);
        g.spawn_prop(8, 0, None).unwrap();
        for _ in 0..16 {
            g.pool_b_update();
        }
        assert_eq!(live(&g).unwrap().b.mode, 4);
        g.pool_b_update();
        assert!(live(&g).is_none());
    }
}
