//! Pool B class 9, lightning of Episode 6 area 1 (0x77E76).
//!
//! Three variants at far layer X 0xA0 (var 0), 0x260 (var 4), 0x340 (var 8).
//! An object runs while its X + 0x20 is not left of the far layer camera X (0x77EB8).
//! While active, it cycles through:
//! 1. 4 bolt tiles drawn in 5 steps of 3 frames (0x77ED8..0x77F22, 0x77F48).
//! 2. At bolt completion, restore step 0 is drawn, pause timer is loaded from 0x77F38,
//!    thunder timer 0x28 is armed, and flashes are activated.
//! 3. Sequence 1 (8 scroll3 sky palettes, 0x77FD2) runs 6 steps of [3, 1, 2, 1, 2, 3] frames.
//! 4. Sequence 2 (32 scroll2 foreground palettes, 0x78146) starts 8 frames later and runs 6 steps.
//! 5. Thunder sound 0x77 plays 40 frames after the bolt finishes (0x77EC4, 0xA25A).

use super::tile_block::{TileMode, TilePos};
use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x77E76: pool B class 9, lightning.
    pub fn prop_b9(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => self.prop_b9_init(p),
            2 => self.prop_b9_run(p),
            _ => self.prop_free(p, i), // 0x77FC4: jmp 0x4938
        }
    }

    /// 0x77E8A: mode 0 initialization.
    fn prop_b9_init(&mut self, p: &mut Prop) {
        p.b.mode = 2; // 0x77E8A addq.b #2, $4(a6)
        p.m.sw(0xA0, 1); // 0x77E8E move.w #1, $a0(a6)
        p.m.sw(0xA2, 0); // 0x77E94 clr.w $a2(a6)
        p.m.sw(0xA6, 0); // 0x77E98 clr.w $a6(a6)
        p.m.sb(0xBC, 0); // 0x77E9C clr.b $bc(a6)
        p.m.sw(0xB8, 0); // 0x77EA0 clr.w $b8(a6)
        p.m.sw(0xBA, 0); // 0x77EA4 clr.w $ba(a6)
        p.m.sw(0xBE, 0xFFFF); // 0x77EA8 move.w #$ffff, $be(a6)
    }

    /// 0x77EB0: mode 2 run.
    fn prop_b9_run(&mut self, p: &mut Prop) {
        let limit = p.b.xi().wrapping_add(0x20); // 0x77EB0 move.w $8(a6),d0 ; addi.w #$20,d0
        if limit < self.cam.s3.cam_x as i16 { // 0x77EB8 cmp.w $7e4(a5),d0 ; bge.b 0x77ec4
            if p.m.b(0xBC) == 0 { // 0x77EBE tst.b $bc(a6) ; beq.b 0x77f32
                p.b.mode = p.b.mode.wrapping_add(2); // 0x77F32 addq.b #2, $4(a6)
                return;
            }
        }

        // 0x77EC4: thunder timer countdown
        let be = p.m.w(0xBE).wrapping_sub(1); // subq.w #1, $be(a6)
        p.m.sw(0xBE, be);
        if be == 0 { // 0x77EC8 bne.b 0x77ed0
            self.prop_b9_thunder(p); // 0x77ECA jsr 0xA25A
        }

        // 0x77ED0: bolt step timer countdown
        let a0 = p.m.w(0xA0).wrapping_sub(1); // subq.w #1, $a0(a6)
        p.m.sw(0xA0, a0);
        if a0 == 0 { // 0x77ED4 bne.w 0x77f26
            p.m.sw(0xA0, 3); // 0x77ED8 move.w #3, $a0(a6)
            let a2 = p.m.w(0xA2).wrapping_add(4); // 0x77EDE addq.w #4, $a2(a6)
            p.m.sw(0xA2, a2);
            if a2 == 0x14 { // 0x77EE2 cmpi.w #$14, $a2(a6) ; bne.b 0x77f22
                let a6_idx = p.m.w(0xA6);
                let pause = self.assets.enemy.t.u16(0x77F38, a6_idx as u32); // 0x77EEE move.w 0x77f38(pc, d0.w), $a0(a6)
                p.m.sw(0xA0, pause);
                p.m.sw(0xBE, 0x28); // 0x77EF4 move.w #$28, $be(a6)
                p.m.sw(0xA6, a6_idx.wrapping_add(2) & 0x0E); // 0x77EFA addq.w #2, $a6(a6) ; andi.w #$e, $a6(a6)
                p.m.sw(0xA2, 0); // 0x77F04 clr.w $a2(a6)
                p.m.sb(0xBC, 1); // 0x77F08 move.b #1, $bc(a6)
                p.m.sw(0xA8, 9); // 0x77F0E move.w #9, $a8(a6)
                p.m.sw(0xAA, 1); // 0x77F14 move.w #1, $aa(a6)
                p.m.sw(0xB8, 0); // 0x77F1A clr.w $b8(a6)
                p.m.sw(0xBA, 0); // 0x77F1E clr.w $ba(a6)
            }
            self.prop_b9_paint(p.b.var26, p.m.w(0xA2)); // 0x77F22 bsr.w 0x77f48
        }

        // 0x77F26: palette flashes while active
        if p.m.b(0xBC) != 0 { // tst.b $bc(a6) ; beq.b 0x77f30
            self.prop_b9_flashes(p); // 0x77F2C bra.w 0x77fca
        }
    }

    /// 0x77F48: paint 4 bolt tiles (code and attribute pairs) at scroll3 destination.
    fn prop_b9_paint(&mut self, var26: u16, a2: u16) {
        let t = &self.assets.enemy.t;
        let dest = t.u32(0x77F68, var26 as u32) & 0xFFFFFF; // 0x77F5A movea.l $77f68(pc, d0.w), a0
        let src = 0x77F74 + (a2 as u32) * 4; // 0x77F4E add.w d0, d0 ; add.w d0, d0 ; lea $77f74(pc, d0.w), a1
        self.prop_tile_block(TilePos::at(dest), 1, 4, src, TileMode::Pairs);
    }

    /// 0x77FCA: run palette flash sequence 1 and sequence 2.
    fn prop_b9_flashes(&mut self, p: &mut Prop) {
        self.prop_b9_seq1(p); // 0x77FCA bsr.w 0x77fd2
        self.prop_b9_seq2(p); // 0x77FCE bra.w 0x78146
    }

    /// 0x77FD2: flash sequence 1 (8 scroll3 sky palettes).
    fn prop_b9_seq1(&mut self, p: &mut Prop) {
        let aa = p.m.w(0xAA).wrapping_sub(1); // subq.w #1, $aa(a6)
        p.m.sw(0xAA, aa);
        if aa != 0 {
            return; // 0x77FD6 bne.b 0x77ff4
        }
        let ba = p.m.w(0xBA);
        let next_timer = self.assets.enemy.t.u16(0x77FF6, (ba >> 1) as u32); // 0x77FDE move.w $77ff6(pc, d0.w), $aa(a6)
        p.m.sw(0xAA, next_timer);
        if ba == 0x18 {
            return; // 0x77FEA beq.b 0x77ff4
        }
        // 0x78002: apply 8 palettes
        for lane in 0..8u32 {
            let src_tbl = self.assets.enemy.t.u32(0x78036, lane * 8);
            let dst_pal = self.assets.enemy.t.u32(0x78036, lane * 8 + 4) & 0xFFFFFF;
            let pal_ptr = self.assets.enemy.t.u32(src_tbl, ba as u32);
            self.prop_palette_copy(dst_pal, pal_ptr, 16);
        }
        p.m.sw(0xBA, ba.wrapping_add(4)); // 0x77FF0 addq.w #4, $ba(a6)
    }

    /// 0x78146: flash sequence 2 (32 scroll2 foreground palettes).
    fn prop_b9_seq2(&mut self, p: &mut Prop) {
        let a8 = p.m.w(0xA8).wrapping_sub(1); // subq.w #1, $a8(a6)
        p.m.sw(0xA8, a8);
        if a8 != 0 {
            return; // 0x7814A bne.b 0x78168
        }
        let b8 = p.m.w(0xB8);
        let next_timer = self.assets.enemy.t.u16(0x78170, (b8 >> 1) as u32); // 0x78152 move.w $78170(pc, d0.w), $a8(a6)
        p.m.sw(0xA8, next_timer);
        if b8 == 0x18 {
            p.m.sw(0xBC, 0); // 0x7816A clr.w $bc(a6)
            return;
        }
        // 0x7817C: apply 32 palettes
        for lane in 0..32u32 {
            let src_tbl = self.assets.enemy.t.u32(0x781B0, lane * 8);
            let dst_pal = self.assets.enemy.t.u32(0x781B0, lane * 8 + 4) & 0xFFFFFF;
            let pal_ptr = self.assets.enemy.t.u32(src_tbl, b8 as u32);
            self.prop_palette_copy(dst_pal, pal_ptr, 16);
        }
        p.m.sw(0xB8, b8.wrapping_add(4)); // 0x78164 addq.w #4, $b8(a6)
    }

    /// 0xA25A: thunder sound with pan from object X.
    fn prop_b9_thunder(&mut self, p: &Prop) {
        let pan = self.snd_pan_x(p.b.xi()); // 0xA25C..0xA27C
        self.snd_request(0x77, pan as u32, 0); // 0xA280..0xA284
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::enemy::ByteTables;
    use dino_assets::AssetPack;
    use std::path::Path;

    fn test_game(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs };
        Game::new(Box::leak(Box::new(a)))
    }

    fn synthetic_tables() -> Vec<(u32, Vec<u8>)> {
        let mut blobs = Vec::new();

        // 1. Pause table: 0x77F38 (16 bytes = 8 u16s)
        blobs.push((
            0x77F38,
            vec![0, 100, 0, 200, 0, 60, 0, 120, 0, 60, 1, 0xA4, 0, 240, 0, 120],
        ));

        // 2. VRAM dest table: 0x77F68 (12 bytes = 3 u32s: var 0, 4, 8)
        let mut vram_dest = Vec::new();
        vram_dest.extend_from_slice(&0x009138A0u32.to_be_bytes());
        vram_dest.extend_from_slice(&0x00913A60u32.to_be_bytes());
        vram_dest.extend_from_slice(&0x00913B40u32.to_be_bytes());
        blobs.push((0x77F68, vram_dest));

        // 3. Bolt pictures: 0x77F74 (80 bytes = 5 steps * 16 bytes)
        let mut bolt_pics = Vec::new();
        for step in 0..5u16 {
            for row in 0..4u16 {
                let code = 0x1000 + step * 0x10 + row;
                let attr: u16 = 0x0004;
                bolt_pics.extend_from_slice(&code.to_be_bytes());
                bolt_pics.extend_from_slice(&attr.to_be_bytes());
            }
        }
        blobs.push((0x77F74, bolt_pics));

        // 4. Seq 1 frame counts: 0x77FF6 (16 bytes)
        blobs.push((
            0x77FF6,
            vec![0, 3, 0, 1, 0, 2, 0, 1, 0, 2, 0, 3, 0x7C, 0, 0x7C, 0],
        ));

        // 5. Seq 1 palette table: 0x78036 (64 bytes = 8 entries * 8 bytes)
        let mut seq1_table = Vec::new();
        for lane in 0..8u32 {
            let src_tbl = 0x78076 + lane * 24;
            let dst_pal = 0x00914C80 + lane * 32;
            seq1_table.extend_from_slice(&src_tbl.to_be_bytes());
            seq1_table.extend_from_slice(&dst_pal.to_be_bytes());
        }
        blobs.push((0x78036, seq1_table));

        // 6. Seq 1 source pointers: 0x78076 (208 bytes)
        let mut seq1_ptrs = Vec::new();
        for _ in 0..52 {
            seq1_ptrs.extend_from_slice(&0x000C36D2u32.to_be_bytes());
        }
        blobs.push((0x78076, seq1_ptrs));

        // 7. Seq 2 frame counts: 0x78170 (16 bytes)
        blobs.push((
            0x78170,
            vec![0, 3, 0, 1, 0, 2, 0, 1, 0, 2, 0, 3, 0x7C, 0, 0x7C, 0],
        ));

        // 8. Seq 2 palette table: 0x781B0 (256 bytes = 32 entries * 8 bytes)
        let mut seq2_table = Vec::new();
        for lane in 0..32u32 {
            let src_tbl = 0x782B0 + lane * 24;
            let dst_pal = 0x00914800 + lane * 32;
            seq2_table.extend_from_slice(&src_tbl.to_be_bytes());
            seq2_table.extend_from_slice(&dst_pal.to_be_bytes());
        }
        blobs.push((0x781B0, seq2_table));

        // 9. Seq 2 source pointers: 0x782B0 (832 bytes)
        let mut seq2_ptrs = Vec::new();
        for _ in 0..208 {
            seq2_ptrs.extend_from_slice(&0x000BEE52u32.to_be_bytes());
        }
        blobs.push((0x782B0, seq2_ptrs));

        // 10. Dummy palette blobs at 0xC36D2 and 0xBEE52 (32 bytes each: 16 colors)
        blobs.push((0x0C36D2, vec![0x22; 32]));
        blobs.push((0x0BEE52, vec![0x33; 32]));

        // 11. Pan table: 0xA66A (48 bytes)
        blobs.push((0xA66A, vec![0x10; 48]));

        blobs
    }

    fn rom_game() -> Option<Game> {
        let path = Path::new("../../../dino.zip");
        let alt = Path::new("../dino.zip");
        let p = if path.exists() {
            path
        } else if alt.exists() {
            alt
        } else {
            return None;
        };
        let assets: &'static AssetPack =
            Box::leak(Box::new(dino_extract::extract(p).expect("dino.zip loads")));
        Some(Game::new(assets))
    }

    #[test]
    fn b9_init_and_camera_limit() {
        let mut g = test_game(vec![]);
        let mut p = Prop::default();
        p.b.live = 1;
        p.b.k20 = 9 * 4;
        p.b.set_xi(0x260); // far layer X

        // Mode 0: init
        g.prop_b9(&mut p, 0);
        assert_eq!(p.b.mode, 2);
        assert_eq!(p.m.w(0xA0), 1);
        assert_eq!(p.m.w(0xA2), 0);
        assert_eq!(p.m.w(0xA6), 0);
        assert_eq!(p.m.b(0xBC), 0);
        assert_eq!(p.m.w(0xB8), 0);
        assert_eq!(p.m.w(0xBA), 0);
        assert_eq!(p.m.w(0xBE), 0xFFFF);

        // Camera is behind object (cam_x = 0x200 <= 0x260 + 0x20 = 0x280)
        g.cam.s3.cam_x = 0x200;
        // Mode 2: run with no tables will fail if bolt fires, but timer is 1, so it counts down
        // to 0 and needs bolt table. Tested further in b9_bolt_cycle_and_thunder.
    }

    #[test]
    fn b9_bolt_cycle_and_thunder() {
        let blobs = synthetic_tables();
        let mut g = test_game(blobs);
        let mut p = Prop::default();
        p.b.live = 1;
        p.b.k20 = 9 * 4;
        p.b.var26 = 4; // var 4 -> dest 0x913A60
        p.b.set_xi(0x260);

        // Init
        g.prop_b9(&mut p, 0);
        assert_eq!(p.b.mode, 2);

        // Frame 1: a0 (1 -> 0) triggers step 1 paint
        g.prop_b9(&mut p, 0);
        assert_eq!(p.m.w(0xA0), 3);
        assert_eq!(p.m.w(0xA2), 4);
        // Cell at 0x913A60 row 0 should have step 1 code 0x1010
        let c = g.vram.scroll3[(0x3A60 / 4) as usize];
        assert_eq!(c.code, 0x1010);

        // Frames 2..3: counting down a0
        g.prop_b9(&mut p, 0); // a0 -> 2
        assert_eq!(p.m.w(0xA0), 2);
        g.prop_b9(&mut p, 0); // a0 -> 1
        assert_eq!(p.m.w(0xA0), 1);

        // Frame 4: a0 -> 0 triggers step 2 paint
        g.prop_b9(&mut p, 0);
        assert_eq!(p.m.w(0xA0), 3);
        assert_eq!(p.m.w(0xA2), 8);
        assert_eq!(g.vram.scroll3[(0x3A60 / 4) as usize].code, 0x1020);

        // Step 3 after 3 frames
        g.prop_b9(&mut p, 0);
        g.prop_b9(&mut p, 0);
        g.prop_b9(&mut p, 0);
        assert_eq!(p.m.w(0xA2), 12);
        assert_eq!(g.vram.scroll3[(0x3A60 / 4) as usize].code, 0x1030);

        // Step 4 after 3 frames
        g.prop_b9(&mut p, 0);
        g.prop_b9(&mut p, 0);
        g.prop_b9(&mut p, 0);
        assert_eq!(p.m.w(0xA2), 16);
        assert_eq!(g.vram.scroll3[(0x3A60 / 4) as usize].code, 0x1040);

        // End of bolt: step 0 (plain sky restore) painted, pause timer loaded, flashes armed!
        g.prop_b9(&mut p, 0); // a0: 3 -> 2
        g.prop_b9(&mut p, 0); // a0: 2 -> 1
        g.prop_b9(&mut p, 0); // a0: 1 -> 0, a2 hits 20 -> reset to 0, pause timer loaded
        assert_eq!(p.m.w(0xA2), 0);
        assert_eq!(p.m.w(0xA0), 100);
        assert_eq!(p.m.w(0xA6), 2);
        assert_eq!(p.m.w(0xBE), 40);
        assert_eq!(p.m.b(0xBC), 1);
        // Plain sky restore code
        assert_eq!(g.vram.scroll3[(0x3A60 / 4) as usize].code, 0x1000);

        // Advance 40 frames to thunder
        let reqs_before = g.snd_take_requests();
        assert!(reqs_before.is_empty());
        for _ in 0..40 {
            g.prop_b9(&mut p, 0);
        }
        let reqs = g.snd_take_requests();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].code, 0x77);
        assert_eq!(reqs[0].d1, 0x10); // pan byte from table
    }

    #[test]
    fn b9_camera_cleanup_respects_flashes() {
        let mut g = test_game(vec![]);
        let mut p = Prop::default();
        p.b.live = 1;
        p.b.k20 = 9 * 4;
        p.b.set_xi(0x100);
        p.b.mode = 2;
        p.m.sb(0xBC, 1); // flashes active

        // Camera moves past object: limit = 0x100 + 0x20 = 0x120. cam_x = 0x130.
        g.cam.s3.cam_x = 0x130;
        // While bc == 1, prop does not free
        p.m.sw(0xBE, 100);
        p.m.sw(0xA0, 100);
        p.m.sw(0xAA, 100);
        p.m.sw(0xA8, 100);
        g.prop_b9(&mut p, 0);
        assert_eq!(p.b.mode, 2);

        // When bc == 0, prop frees on next tick
        p.m.sb(0xBC, 0);
        g.prop_b9(&mut p, 0);
        assert_eq!(p.b.mode, 4);
    }

    #[test]
    fn b9_palette_flashes_timing() {
        let blobs = synthetic_tables();
        let mut g = test_game(blobs);
        let mut p = Prop::default();
        p.b.live = 1;
        p.b.k20 = 9 * 4;
        p.b.var26 = 0;
        p.b.set_xi(0x260);

        p.b.mode = 2;
        p.m.sw(0xBE, 0xFFFF);
        p.m.sw(0xA0, 100); // bolt waiting
        p.m.sb(0xBC, 1); // flashes active
        p.m.sw(0xAA, 1); // seq 1 ready
        p.m.sw(0xA8, 9); // seq 2 starts 8 frames later
        p.m.sw(0xBA, 0);
        p.m.sw(0xB8, 0);

        // Frame 1: seq 1 step 0 fires (ba: 0 -> 4, aa: 1 -> 3), seq 2 counts down (a8: 9 -> 8)
        g.prop_b9(&mut p, 0);
        assert_eq!(p.m.w(0xBA), 4);
        assert_eq!(p.m.w(0xAA), 3);
        assert_eq!(p.m.w(0xB8), 0);
        assert_eq!(p.m.w(0xA8), 8);
        assert!(g.pal.dirty);

        // Run until frame 8 (a8 counts 8 down to 1)
        for _ in 0..7 {
            g.prop_b9(&mut p, 0);
        }
        assert_eq!(p.m.w(0xA8), 1);
        assert_eq!(p.m.w(0xB8), 0);

        // Frame 9 (8 frames after frame 1): seq 2 step 0 fires (b8: 0 -> 4, a8: 1 -> 3)
        g.prop_b9(&mut p, 0);
        assert_eq!(p.m.w(0xB8), 4);
        assert_eq!(p.m.w(0xA8), 3);

        // Run remaining sequence until sequence 2 completes (b8 reaches 24 and resets bc)
        for _ in 0..100 {
            g.prop_b9(&mut p, 0);
            if p.m.b(0xBC) == 0 {
                break;
            }
        }
        assert_eq!(p.m.b(0xBC), 0);
        assert_eq!(p.m.w(0xB8), 24);
        assert_eq!(p.m.w(0xBA), 24);
    }

    #[test]
    fn b9_rom_tables_integration() {
        let Some(mut g) = rom_game() else {
            return;
        };
        let mut p = Prop::default();
        p.b.live = 1;
        p.b.k20 = 9 * 4;
        p.b.var26 = 4; // var 4
        p.b.set_xi(0x260);

        // Mode 0 init
        g.prop_b9(&mut p, 0);
        assert_eq!(p.b.mode, 2);

        // Run 200 frames: bolt fires, paints VRAM, restore happens, flashes run, thunder sounds
        for _ in 0..200 {
            g.prop_b9(&mut p, 0);
        }
        // thunder sound 0x77 should have been queued
        let sounds = g.snd_take_requests();
        assert!(sounds.iter().any(|r| r.code == 0x77));
    }
}
