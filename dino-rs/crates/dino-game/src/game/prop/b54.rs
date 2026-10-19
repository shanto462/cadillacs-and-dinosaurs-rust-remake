//! Pool B class 54 (0x855A8): Episode 7 cage wall smash (lift, wave 4).
//! Spec: D3e part 3 sections 2.1, 4.2, A15, W310.
//!
//! Smashes the lower left and lower right cage walls on the lift:
//! - Scroll1 cols 5..12 and 35..42, rows 47..54: two 8 x 8 blocks become the broken corners.
//! - If `cage_smashed` (G.$7B6) is 0: spawns pool C class 21 (flying planks), var 0x1E, facing 1
//!   via 0x94640, and plays the break sound 0x9E60.
//! - Mode 6 sets `cage_smashed` (G.$7B6) to 1 and frees the prop.

use super::tile_block::{TileMode, TilePos};
use super::Prop;
use crate::game::Game;

/// Destination base addresses for the two 8 x 8 scroll1 blocks:
/// - Left corner: 0x90A2BC (cols 5..12, rows 47..54)
/// - Right corner: 0x90B1BC (cols 35..42, rows 47..54)
const DEST_LEFT: TilePos = TilePos::at(0x90A2BC);
const DEST_RIGHT: TilePos = TilePos::at(0x90B1BC);

impl Game {
    /// 0x855A8: pool B class 54 update (cage wall smash).
    pub fn prop_b54(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x855BC: mode 0 arms with timer 1 and falls through to mode 2 and mode 4
                p.b.mode = 2;
                p.m.sw(0xA0, 1);
                // 0x855C6: subq.w #1, $a0(a6) ; beq 0x855CE
                let t = p.m.w(0xA0).wrapping_sub(1);
                p.m.sw(0xA0, t);
                if t == 0 {
                    p.b.mode = 4;
                    self.prop_b54_swap(p);
                    p.b.mode = 6; // 0x855E6: addq.b #2, $4(a6)
                }
            }
            2 => {
                // 0x855C6: mode 2 wait
                let t = p.m.w(0xA0).wrapping_sub(1);
                p.m.sw(0xA0, t);
                if t == 0 {
                    p.b.mode = 4;
                    self.prop_b54_swap(p);
                    p.b.mode = 6;
                }
            }
            4 => {
                // 0x855D2: mode 4 swap
                self.prop_b54_swap(p);
                p.b.mode = 6;
            }
            6 => {
                // 0x85740: move.b #1, $7B6(a5) ; jmp 0x4938
                self.spawn.g.cage_smashed = 1;
                self.prop_free(p, i);
            }
            _ => {
                self.prop_free(p, i);
            }
        }
    }

    /// 0x855D2..0x855EA: execute the cage smash swap.
    fn prop_b54_swap(&mut self, p: &Prop) {
        // 0x855EC: write two 8 x 8 blocks of code words on scroll1
        self.prop_tile_block(DEST_LEFT, 8, 8, 0x8562E, TileMode::Codes);
        self.prop_tile_block(DEST_RIGHT, 8, 8, 0x856B2, TileMode::Codes);

        // 0x855D6: tst.b $7B6(a5)
        if self.spawn.g.cage_smashed == 0 {
            // 0x855DC -> 0x85736: spawn pool C class 21 (flying planks), var 0x1E, facing 1
            self.prop_c21_spawn(p.b.xi(), p.b.yi(), p.b.zi(), 0x1E, 1);
            // 0x855E0 -> 0x9E60: break sound with pan
            self.snd_break(p.b.xi());
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
        a.enemy.t = ByteTables { blobs };
        Game::new(Box::leak(Box::new(a)))
    }

    #[test]
    fn b54_smashes_cage_spawns_effect_and_sets_cage_smashed() {
        let mut b1 = Vec::new();
        let mut b2 = Vec::new();
        for i in 0..64 {
            b1.extend_from_slice(&(0x0F00u16 + i as u16).to_be_bytes());
            b2.extend_from_slice(&(0x0F80u16 + i as u16).to_be_bytes());
        }
        let blobs = vec![
            (0xA66A, (0..0x30).collect()),
            (0x8562E, b1),
            (0x856B2, b2),
        ];

        let mut g = test_game(blobs);

        // the test hands out a prop that is not from the pool: take every slot so index 0 is not free
        for _ in 0..24 {
            g.props.alloc();
        }
        assert_eq!(g.spawn.g.cage_smashed, 0);

        let mut p = Prop::default();
        p.b.live = 1;
        p.b.k20 = 54 * 4;
        p.b.set_xi(192);
        p.b.set_yi(0);
        p.b.set_zi(48);

        // First call: mode 0 falls through mode 2 and mode 4, sets mode 6
        g.prop_b54(&mut p, 0);
        assert_eq!(p.b.mode, 6);

        // Effect spawned: pool C class 21, var 0x1E, facing 1
        let eff = g.effects.slots.iter().find(|e| e.b.live != 0).expect("effect spawned");
        assert_eq!(eff.b.k20, 0x54);
        assert_eq!(eff.b.var26, 0x1E);
        assert_eq!(eff.b.facing, 1);

        // Sound queued
        let reqs = g.snd_take_requests();
        assert_eq!(reqs.len(), 1);

        // Next call (mode 6): sets cage_smashed = 1 and frees prop
        g.prop_b54(&mut p, 0);
        assert_eq!(g.spawn.g.cage_smashed, 1);
        assert_eq!(p.b.live, 0);
    }

    #[test]
    fn b54_skips_effect_if_already_smashed() {
        let mut b1 = Vec::new();
        let mut b2 = Vec::new();
        for i in 0..64 {
            b1.extend_from_slice(&(0x0F00u16 + i as u16).to_be_bytes());
            b2.extend_from_slice(&(0x0F80u16 + i as u16).to_be_bytes());
        }
        let blobs = vec![
            (0xA66A, (0..0x30).collect()),
            (0x8562E, b1),
            (0x856B2, b2),
        ];

        let mut g = test_game(blobs);

        // the test hands out a prop that is not from the pool: take every slot so index 0 is not free
        for _ in 0..24 {
            g.props.alloc();
        }
        g.spawn.g.cage_smashed = 1;

        let mut p = Prop::default();
        p.b.live = 1;
        p.b.k20 = 54 * 4;

        g.prop_b54(&mut p, 0);
        assert_eq!(p.b.mode, 6);
        assert_eq!(g.effects.slots.iter().filter(|e| e.b.live != 0).count(), 0);
        assert!(g.snd_take_requests().is_empty());

        g.prop_b54(&mut p, 0);
        assert_eq!(g.spawn.g.cage_smashed, 1);
        assert_eq!(p.b.live, 0);
    }
}
