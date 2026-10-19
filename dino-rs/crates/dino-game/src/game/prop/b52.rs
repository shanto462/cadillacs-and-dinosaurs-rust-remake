//! Pool B class 52 (0x85178): Episode 7 bookshelf burst (wave 12).
//! Spec: D3e part 3 sections 2.1, 4.3, A17, W302.
//!
//! Spawned at (1840, 0, 112) in wave 12 of Episode 7 area 2 (library).
//! Swaps a 10 x 7 tile block (70 cells) at scroll2 0x90FBC4 (cols 47..56, rows 49..55),
//! spawns flying books (pool C class 21, var 0x20, facing 0) via 0x94640,
//! plays the break sound 0x9E60, and frees itself on mode 6.

use super::tile_block::TilePos;
use super::Prop;
use crate::game::Game;

/// Destination base for the bookshelf gap on scroll2: cols 47..56, rows 49..55.
const DEST: TilePos = TilePos::at(0x90FBC4);

impl Game {
    /// 0x85178: pool B class 52 update (bookshelf burst).
    pub fn prop_b52(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x8518C: mode 0 arms with timer 1 and falls through to mode 2 and mode 4
                p.b.mode = 2;
                p.m.sw(0xA0, 1);
                // 0x85196: subq.w #1, $a0(a6) ; beq 0x8519E
                let t = p.m.w(0xA0).wrapping_sub(1);
                p.m.sw(0xA0, t);
                if t == 0 {
                    p.b.mode = 4;
                    self.prop_b52_swap(p);
                    p.b.mode = 6; // 0x851B0: addq.b #2, $4(a6)
                }
            }
            2 => {
                // 0x85196: mode 2 wait
                let t = p.m.w(0xA0).wrapping_sub(1);
                p.m.sw(0xA0, t);
                if t == 0 {
                    p.b.mode = 4;
                    self.prop_b52_swap(p);
                    p.b.mode = 6;
                }
            }
            4 => {
                // 0x851A2: mode 4 swap
                self.prop_b52_swap(p);
                p.b.mode = 6;
            }
            _ => {
                // 0x85300: jmp 0x4938
                self.prop_free(p, i);
            }
        }
    }

    /// 0x851A2..0x851B4: execute the bookshelf burst swap.
    fn prop_b52_swap(&mut self, p: &Prop) {
        // 0x851B6: write 10 columns of 7 cells
        let mut idx = 0;
        for c in 0..10 {
            for r in 0..7 {
                let cell = DEST.right(c).down(r);
                let code = self.assets.enemy.t.u16(0x851DE, 2 * idx);
                let attr = self.assets.enemy.t.u16(0x8526A, 2 * idx);
                self.prop_tile_put(cell, code, attr);
                idx += 1;
            }
        }
        // 0x851A6 -> 0x852F6: spawn pool C class 21 (flying books), var 0x20, facing 0
        self.prop_c21_spawn(p.b.xi(), p.b.yi(), p.b.zi(), 0x20, 0);
        // 0x851AA -> 0x9E60: break sound with pan
        self.snd_break(p.b.xi());
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
    fn b52_bursts_and_frees() {
        let mut codes = Vec::new();
        let mut attrs = Vec::new();
        for i in 0..70 {
            codes.extend_from_slice(&(0x6100u16 + i as u16).to_be_bytes());
            attrs.extend_from_slice(&(0x0002u16 + (i % 8) as u16).to_be_bytes());
        }
        let blobs = vec![
            (0xA66A, (0..0x30).collect()),
            (0x851DE, codes),
            (0x8526A, attrs),
        ];

        let mut g = test_game(blobs);

        // the test hands out a prop that is not from the pool: take every slot so index 0 is not free
        for _ in 0..24 {
            g.props.alloc();
        }
        let mut p = Prop::default();
        p.b.live = 1;
        p.b.k20 = 52 * 4;
        p.b.set_xi(1840);
        p.b.set_yi(0);
        p.b.set_zi(112);

        // First call: mode 0 falls through mode 2 and mode 4, sets mode 6
        g.prop_b52(&mut p, 0);
        assert_eq!(p.b.mode, 6);

        // Effect spawned: pool C class 21 (k20 = 0x54), var 0x20
        let eff = g.effects.slots.iter().find(|e| e.b.live != 0).expect("effect spawned");
        assert_eq!(eff.b.k20, 0x54);
        assert_eq!(eff.b.var26, 0x20);
        assert_eq!(eff.b.facing, 0);
        assert_eq!(eff.b.xi(), 1840);

        // Sound queued
        let reqs = g.snd_take_requests();
        assert_eq!(reqs.len(), 1);

        // Next call frees prop
        g.prop_b52(&mut p, 0);
        assert_eq!(p.b.live, 0);
    }
}
