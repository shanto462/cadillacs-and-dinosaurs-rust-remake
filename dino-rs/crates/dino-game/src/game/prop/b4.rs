//! Pool B class 4 (0x767D8): mine door burst open at the start of a wave group.
//! Spec: D3e part 3 sections 2.1, 2.9 (A05, W302).
//!
//! Spawns in Episode 6 Area 0 (stage 5 area 0):
//! - Wave 0: var 4 at X 0x220, Z 0x60. Dest: scroll2 0x90F88C.
//! - Wave 0: var 8 at X 0x2A0, Z 0x60. Dest: scroll2 0x90FA8C.
//! - Wave 2: var 0xC at X 0x7B0, Z 0x60. Dest: scroll2 0x90FED0.
//!
//! On update 0 (mode 0), it sets Y to 0x60 (96), sets timer at 0xA0 to 1, advances to mode 2,
//! and falls through to mode 2. When the timer expires, it transitions to mode 4, executes
//! the tile block swap (4 columns x 6 rows) and attribute update (4 cells in col 0 with bit 5 set),
//! spawns pool C class 21 var 0x16 (flying planks), plays break sound 0x9E60, advances to mode 6,
//! and frees itself on the next update (0x769DC -> 0x4938).
//! The wave controller spawns the emerging enemies 0x50 frames later.

use super::tile_block::{TileMode, TilePos};
use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x767D8: pool B class 4 update (door burst).
    pub fn prop_b4(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x767EC: mode 0
                p.b.set_yi(0x60); // 0x767F6: move.w #$60, $c(a6)
                p.b.mode = 2; // 0x767EC: addq.b #2, $4(a6)
                p.m.sw(0xA0, 1); // 0x767F0: move.w #1, $a0(a6)
                // 0x767FC: subq.w #1, $a0(a6) ; beq 0x76804
                let t = p.m.w(0xA0).wrapping_sub(1);
                p.m.sw(0xA0, t);
                if t == 0 {
                    p.b.mode = 4;
                    self.prop_b4_fire(p);
                    p.b.mode = 6;
                }
            }
            2 => {
                // 0x767FC: mode 2 wait
                let t = p.m.w(0xA0).wrapping_sub(1);
                p.m.sw(0xA0, t);
                if t == 0 {
                    p.b.mode = 4;
                    self.prop_b4_fire(p);
                    p.b.mode = 6;
                }
            }
            4 => {
                // 0x76808: mode 4 swap
                self.prop_b4_fire(p);
                p.b.mode = 6;
            }
            _ => {
                // 0x769DC: jmp 0x4938
                self.prop_free(p, i);
            }
        }
    }

    /// 0x76808: execute the door burst tile swap, effect spawn, and break sound.
    fn prop_b4_fire(&mut self, p: &Prop) {
        let (dest, codes, rows, attr_row) = if p.b.var26 == 0xC {
            // 0x76910: var 0xC uses destination 0x90FED0 and codes from 0x7699A (4 cols x 6 rows)
            (0x0090FED0, 0x7699A, 6, 1)
        } else {
            // 0x7682A: var 4 and var 8 use destination table at 0x76904 and codes from 0x768CC (4 cols x 7 rows)
            let dest = self.assets.enemy.t.u32(0x76904, u32::from(p.b.var26));
            (dest, 0x768CC, 7, 2)
        };
        // 0x7683E / 0x7691C: 4 columns of tile codes
        self.prop_tile_block(TilePos::at(dest), 4, rows, codes, TileMode::Codes);
        // 0x76848 / 0x76922: 4 attribute cells in col 0 with bit 5 set (X-flip)
        self.prop_tile_attr_block(TilePos::at(dest).down(attr_row), 1, 4, 0xFFFF, 0x0020);
        // 0x769D2: spawn pool C class 21 (0x54) var 0x16, facing 0
        self.prop_c21_spawn(p.b.xi(), p.b.yi(), p.b.zi(), 0x16, 0);
        // 0x7681E: break sound 0x9E60 with pan from X
        self.snd_break(p.b.xi());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::enemy::ByteTables;
    use dino_assets::AssetPack;

    fn test_game(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs };
        Game::new(Box::leak(Box::new(a)))
    }

    #[test]
    fn b4_var4_burst_and_free() {
        let mut dests = Vec::new();
        dests.extend_from_slice(&0x0090F50Cu32.to_be_bytes()); // var 0
        dests.extend_from_slice(&0x0090F88Cu32.to_be_bytes()); // var 4
        dests.extend_from_slice(&0x0090FA8Cu32.to_be_bytes()); // var 8
        let codes: Vec<u8> = (0..28u16).flat_map(|w| (0x7100 + w).to_be_bytes()).collect();
        let blobs = vec![
            (0xA66A, vec![0x10; 48]), // pan table for 0x9E60
            (0x76904, dests),
            (0x768CC, codes),
        ];
        let mut g = test_game(blobs);

        let h = g.spawn_prop(4, 4, Some((0x220, 0, 0x60))).unwrap();
        let i = match crate::game::slot_of(h).unwrap() {
            crate::game::Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };

        // First update: mode 0 sets Y=0x60, falls into mode 2, timer expires, fires burst, mode becomes 6
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 6);
        assert_eq!(g.props.slots[i].b.yi(), 0x60);
        assert_eq!(g.w.sounds, vec![0xF6]);

        // Second update: mode 6 frees slot
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.live, 0);
    }

    #[test]
    fn b4_var_c_burst_and_free() {
        let codes: Vec<u8> = (0..24u16).flat_map(|w| (0x7200 + w).to_be_bytes()).collect();
        let blobs = vec![
            (0xA66A, vec![0x10; 48]),
            (0x7699A, codes),
        ];
        let mut g = test_game(blobs);

        let h = g.spawn_prop(4, 0xC, Some((0x7B0, 0, 0x60))).unwrap();
        let i = match crate::game::slot_of(h).unwrap() {
            crate::game::Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };

        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 6);
        assert_eq!(g.props.slots[i].b.yi(), 0x60);

        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.live, 0);
    }
}
