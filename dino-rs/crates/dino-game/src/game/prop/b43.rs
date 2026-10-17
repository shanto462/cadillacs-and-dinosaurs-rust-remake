//! Pool B class 43: Episode 8 wall torch flame (0x83ACA).
//!
//! A 2-by-2 tile flame in scroll2, animated with 4 pictures every 8 frames after an
//! initial wait of 15 frames.

use super::tile_block::TilePos;
use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x83ACA pool B class 43: wall torch flame (2 by 2 tiles).
    pub fn prop_b43(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x83ADE: mode 0 -> mode 2, wait = 15, frame = 0
                p.b.mode = 2;
                p.m.sb(0xA0, 15);
                p.m.sw(0xA2, 0);
            }
            2 => {
                // 0x83AEE: camera cull test
                if p.b.xi().wrapping_add(0x20) < self.cam.s2.cam_x as i16 {
                    // 0x83B16: behind camera, advance to mode 4 to free next frame
                    p.b.mode += 2;
                    return;
                }
                // 0x83AFC: tick timer
                let t = p.m.b(0xA0).wrapping_sub(1);
                p.m.sb(0xA0, t);
                if t != 0 {
                    return; // 0x83B14
                }
                // 0x83B02: reload period 8, advance frame index in steps of 4 (mod 16)
                p.m.sb(0xA0, 8);
                let frame = (p.m.w(0xA2).wrapping_add(4)) & 0x0C;
                p.m.sw(0xA2, frame);

                // 0x83B1C: draw 2x2 tiles
                let var = p.b.var26 as u32;
                if var > 0x18 || (var & 3) != 0 {
                    return self.prop_free(p, i);
                }
                let dest = self.assets.enemy.t.u32(0x83B4E, var);
                let frame_ptr = self.assets.enemy.t.u32(0x83B6A, frame as u32);
                let t = &self.assets.enemy.t;

                // col 0, row 0 (0x83B32)
                self.prop_tile_code(TilePos::at(dest), t.u16(frame_ptr, 0));
                // col 0, row 1 (0x83B36)
                self.prop_tile_code(TilePos::at(dest.wrapping_add(4)), t.u16(frame_ptr, 2));
                // col 1, row 0 (0x83B38..0x83B46: wrap col 63 -> 0 within scroll2 page)
                let col1 = dest.wrapping_add(0x40) & 0xFFFF_CFFF;
                self.prop_tile_code(TilePos::at(col1), t.u16(frame_ptr, 4));
                // col 1, row 1 (0x83B4A)
                self.prop_tile_code(TilePos::at(col1.wrapping_add(4)), t.u16(frame_ptr, 6));
            }
            _ => self.prop_free(p, i), // 0x83BAA: jmp 0x4938
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn cell(g: &Game, a: u32) -> u16 {
        g.vram.scroll2[((a - 0x90C000) / 4) as usize].code
    }

    fn test_game() -> Game {
        let mut a = AssetPack::default();
        // 0x83B4E: 7 dest addresses
        let dests: Vec<u8> = (0..7u32)
            .flat_map(|_| 0x90C4C4u32.to_be_bytes())
            .collect();
        a.enemy.t.blobs.push((0x83B4E, dests));
        // 0x83B6A: 4 frame pointers
        let ptrs: Vec<u8> = [0x83B7Au32, 0x83B86, 0x83B92, 0x83B9E]
            .iter()
            .flat_map(|p| p.to_be_bytes())
            .collect();
        a.enemy.t.blobs.push((0x83B6A, ptrs));
        // 4 frames of 4 words each (+2 pad words)
        for (i, addr) in [0x83B7Au32, 0x83B86, 0x83B92, 0x83B9E].iter().enumerate() {
            let base_code = 0x6500u16 + (i as u16) * 0x10;
            let words = [base_code, base_code + 1, base_code + 2, base_code + 3, 0x5800, 0x5800];
            let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_be_bytes()).collect();
            a.enemy.t.blobs.push((*addr, bytes));
        }

        Game::new(Box::leak(Box::new(a)))
    }

    #[test]
    fn b43_waits_15_frames_then_steps_every_8_frames() {
        let mut g = test_game();
        let i = g.props.alloc().unwrap();
        {
            let p = &mut g.props.slots[i];
            p.b.live = 1;
            p.b.k20 = 43 * 4;
            p.b.var26 = 0;
            p.b.set_xi(0x200);
        }
        g.cam.s2.cam_x = 0x100;
        // Mode 0 init
        let mut p = g.props.slots[i].clone();
        g.prop_b43(&mut p, i);
        g.props.slots[i] = p.clone();
        assert_eq!(p.b.mode, 2);
        assert_eq!(p.m.b(0xA0), 15);
        assert_eq!(p.m.w(0xA2), 0);

        // First 14 ticks in mode 2: timer decrements, no draw
        for expected_t in (1..15).rev() {
            g.prop_b43(&mut p, i);
            assert_eq!(p.m.b(0xA0), expected_t);
        }

        // 15th tick: timer reaches 0, reloads 8, frame index becomes 4, draws frame 1
        g.prop_b43(&mut p, i);
        assert_eq!(p.m.b(0xA0), 8);
        assert_eq!(p.m.w(0xA2), 4);
        assert_eq!(cell(&g, 0x90C4C4), 0x6510);
        assert_eq!(cell(&g, 0x90C4C8), 0x6511);

        // Next 7 ticks: no draw
        for expected_t in (1..8).rev() {
            g.prop_b43(&mut p, i);
            assert_eq!(p.m.b(0xA0), expected_t);
        }

        // 8th tick: draws frame 2 (index 8)
        g.prop_b43(&mut p, i);
        assert_eq!(p.m.b(0xA0), 8);
        assert_eq!(p.m.w(0xA2), 8);
        assert_eq!(cell(&g, 0x90C4C4), 0x6520);

        // Behind camera test
        g.cam.s2.cam_x = 0x300; // xi (0x200) + 0x20 = 0x220 < 0x300
        g.prop_b43(&mut p, i);
        assert_eq!(p.b.mode, 4);
    }
}
