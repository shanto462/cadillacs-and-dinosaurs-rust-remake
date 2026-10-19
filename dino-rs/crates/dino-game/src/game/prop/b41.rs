//! Pool B class 41 (0x8383A): Episode 7 computer room console blinking lights.
//! Spec: D3e part 3 sections 2.1, 4.4, A18, W313.
//!
//! Three variants at X 0x430:
//! - var 0 (0x8389C): 6 tile codes at scroll2 cols 0..2 row 50..51 and 10, 12, 13, 15 row 49; period 19 frames.
//! - var 4 (0x838D8): 5 tile codes at scroll2 cols 0..2 row 50..51 and 11, 14, 16 row 49; period 7 frames.
//! - var 8 (0x8390E): 3 tile codes at scroll2 cols 1..2 row 50..51 and 17 row 49; period 13 frames.
//!
//! The lights cycle through 4 pictures while the camera has not scrolled past X + 0x100.

use super::tile_block::TilePos;
use super::Prop;
use crate::game::Game;

/// Scroll2 destination tile positions for each variant.
const DEST_VAR0: [TilePos; 6] = [
    TilePos::at(0x90F008), // col 0, row 50
    TilePos::at(0x90F08C), // col 2, row 51
    TilePos::at(0x90F284), // col 10, row 49
    TilePos::at(0x90F304), // col 12, row 49
    TilePos::at(0x90F344), // col 13, row 49
    TilePos::at(0x90F3C4), // col 15, row 49
];

const DEST_VAR4: [TilePos; 5] = [
    TilePos::at(0x90F00C), // col 0, row 51
    TilePos::at(0x90F048), // col 1, row 50
    TilePos::at(0x90F2C4), // col 11, row 49
    TilePos::at(0x90F384), // col 14, row 49
    TilePos::at(0x90F404), // col 16, row 49
];

const DEST_VAR8: [TilePos; 3] = [
    TilePos::at(0x90F04C), // col 1, row 51
    TilePos::at(0x90F088), // col 2, row 50
    TilePos::at(0x90F444), // col 17, row 49
];

impl Game {
    /// 0x8383A: pool B class 41 update (console blinking lights).
    pub fn prop_b41(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x8384E: mode 0 init
                p.b.mode = 2; // addq.b #2, $4(a6)
                p.m.sw(0xA0, 1); // move.w #1, $a0(a6): first step on the next frame
                p.m.sw(0xA2, 0); // clr.w $a2(a6): phase index
            }
            2 => {
                // 0x8385E: check if camera has scrolled past object X + 0x100
                let bound = p.b.xi().wrapping_add(0x100);
                if bound < self.cam.s2.cam_x as i16 {
                    p.b.mode = 4; // 0x83880: addq.b #2, $4(a6)
                    return;
                }
                // 0x8386C: subq.w #1, $a0(a6)
                let t = p.m.w(0xA0).wrapping_sub(1);
                p.m.sw(0xA0, t);
                if t != 0 {
                    return; // 0x8387E: rts
                }
                // 0x83872: advance phase index (addq.w #4, andi.w #$C)
                let phase = p.m.w(0xA2).wrapping_add(4) & 0xC;
                p.m.sw(0xA2, phase);
                // 0x83886: draw
                self.prop_b41_draw(p, phase);
            }
            _ => {
                // 0x83A04: jmp 0x4938
                self.prop_free(p, i);
            }
        }
    }

    /// 0x83886: write blinking light tiles by variant.
    fn prop_b41_draw(&mut self, p: &mut Prop, phase: u16) {
        match p.b.var26 {
            0 => {
                // 0x8389C: var 0, reload 18 (period 19)
                p.m.sw(0xA0, 18);
                let pic_ptr = self.assets.enemy.t.u32(0x83944, phase as u32);
                for (k, &dest) in DEST_VAR0.iter().enumerate() {
                    let code = self.assets.enemy.t.u16(pic_ptr, (k * 2) as u32);
                    self.prop_tile_code(dest, code);
                }
            }
            4 => {
                // 0x838D8: var 4, reload 6 (period 7)
                p.m.sw(0xA0, 6);
                let pic_ptr = self.assets.enemy.t.u32(0x83954, phase as u32);
                for (k, &dest) in DEST_VAR4.iter().enumerate() {
                    let code = self.assets.enemy.t.u16(pic_ptr, (k * 2) as u32);
                    self.prop_tile_code(dest, code);
                }
            }
            8 => {
                // 0x8390E: var 8, reload 12 (period 13)
                p.m.sw(0xA0, 12);
                let pic_ptr = self.assets.enemy.t.u32(0x83964, phase as u32);
                for (k, &dest) in DEST_VAR8.iter().enumerate() {
                    let code = self.assets.enemy.t.u16(pic_ptr, (k * 2) as u32);
                    self.prop_tile_code(dest, code);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use dino_assets::enemy::ByteTables;

    fn test_game(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs, ..Default::default() };
        Game::new(Box::leak(Box::new(a)))
    }

    #[test]
    fn b41_blinks_at_correct_periods_and_frees_when_offscreen() {
        let mut blobs = Vec::new();
        // Setup pointer tables at 0x83944, 0x83954, 0x83964
        let mut v0_ptrs = Vec::new();
        let mut v4_ptrs = Vec::new();
        let mut v8_ptrs = Vec::new();
        for step in 0..4 {
            let p0 = 0x83974 + step * 12;
            let p4 = 0x839A4 + step * 10;
            let p8 = 0x839D4 + step * 6;
            v0_ptrs.extend_from_slice(&(p0 as u32).to_be_bytes());
            v4_ptrs.extend_from_slice(&(p4 as u32).to_be_bytes());
            v8_ptrs.extend_from_slice(&(p8 as u32).to_be_bytes());

            blobs.push((p0, [0x60, 0xD5 + step as u8].repeat(6)));
            blobs.push((p4, [0x60, 0xD5 + step as u8].repeat(5)));
            blobs.push((p8, [0x60, 0xD5 + step as u8].repeat(3)));
        }
        blobs.push((0x83944, v0_ptrs));
        blobs.push((0x83954, v4_ptrs));
        blobs.push((0x83964, v8_ptrs));

        let mut g = test_game(blobs);
        g.cam.s2.cam_x = 0x300;

        // Test var 0 (period 19)
        let mut p0 = Prop::default();
        p0.b.live = 1;
        p0.b.k20 = 41 * 4;
        p0.b.var26 = 0;
        p0.b.set_xi(0x430);

        g.prop_b41(&mut p0, 0);
        assert_eq!(p0.b.mode, 2);

        // Frame 1: first step fires
        g.prop_b41(&mut p0, 0);
        assert_eq!(p0.m.w(0xA2), 4);
        assert_eq!(p0.m.w(0xA0), 18);

        // 17 ticks of countdown
        for _ in 0..17 {
            g.prop_b41(&mut p0, 0);
            assert_eq!(p0.m.w(0xA2), 4);
        }

        // Frame 19 after first step: second step fires
        g.prop_b41(&mut p0, 0);
        assert_eq!(p0.m.w(0xA2), 8);
        assert_eq!(p0.m.w(0xA0), 18);

        // Camera past 0x430 + 0x100 = 0x530: frees
        g.cam.s2.cam_x = 0x531;
        g.prop_b41(&mut p0, 0);
        assert_eq!(p0.b.mode, 4);

        // Test var 4 (period 7)
        let mut p4 = Prop::default();
        p4.b.live = 1;
        p4.b.k20 = 41 * 4;
        p4.b.var26 = 4;
        p4.b.set_xi(0x430);
        g.cam.s2.cam_x = 0x300;

        g.prop_b41(&mut p4, 0);
        g.prop_b41(&mut p4, 0);
        assert_eq!(p4.m.w(0xA0), 6);
        for _ in 0..5 {
            g.prop_b41(&mut p4, 0);
            assert_eq!(p4.m.w(0xA2), 4);
        }
        g.prop_b41(&mut p4, 0);
        assert_eq!(p4.m.w(0xA2), 8);
        assert_eq!(p4.m.w(0xA0), 6);

        // Test var 8 (period 13)
        let mut p8 = Prop::default();
        p8.b.live = 1;
        p8.b.k20 = 41 * 4;
        p8.b.var26 = 8;
        p8.b.set_xi(0x430);

        g.prop_b41(&mut p8, 0);
        g.prop_b41(&mut p8, 0);
        assert_eq!(p8.m.w(0xA0), 12);
        for _ in 0..11 {
            g.prop_b41(&mut p8, 0);
            assert_eq!(p8.m.w(0xA2), 4);
        }
        g.prop_b41(&mut p8, 0);
        assert_eq!(p8.m.w(0xA2), 8);
        assert_eq!(p8.m.w(0xA0), 12);
    }
}
