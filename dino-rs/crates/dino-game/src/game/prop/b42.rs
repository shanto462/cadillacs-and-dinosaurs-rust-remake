//! Pool B class 42 (0x83A0A): red sewer door burst open.
//! Spec: D3e part 3 section 2.1, section 2.9 (A24, W302).
//!
//! Spawns in Episode 8 Area 1 at the start of wave groups:
//! - Wave 3: var 0 at X 0x1C8, Z 0x370. Dest: scroll2 0x90C708.
//! - Wave 4: var 4 at X 0x383, Z 0x370. Dest: scroll2 0x90CE08.
//! - Wave 6: var 0xC at X 0x980, Z 0x370. Dest: scroll2 0x90C608.
//! - Wave 7: var 0x10 at X 0xB47, Z 0x370. Dest: scroll2 0x90CD08.
//!
//! On its first update (mode 0), it sets Y to 0x60 (96), falls through to play the break
//! sound (0x9E60), rewrites 4 columns x 6 rows of scroll2 tile codes from 0x83A76 at the
//! destination given by table 0x83AA6, spawns pool C class 21 var 0x26, advances to mode 6,
//! and frees itself on the next update (0x83AC4 -> 0x4938).

use super::tile_block::{TileMode, TilePos};
use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x83A0A: pool B class 42 update.
    pub fn prop_b42(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x83A1E: mode 0
                // 0x83A1E: addq.b #2, $4(a6) -> mode = 2
                // 0x83A22: move.w #$60, $c(a6)
                p.b.set_yi(0x60);
                // 0x83A1E falls into 0x83A28 / 0x83A2C on same frame
                self.prop_b42_fire(p);
                p.b.mode = 6; // 0x83A3A: addq.b #2, $4(a6)
            }
            2 | 4 => {
                // 0x83A28: mode 2 / 0x83A2C: mode 4
                self.prop_b42_fire(p);
                p.b.mode = 6; // 0x83A3A: addq.b #2, $4(a6)
            }
            _ => self.prop_free(p, i), // 0x83AC4: jmp 0x4938
        }
    }

    /// 0x83A2C: sound, tile block swap and effect spawn.
    fn prop_b42_fire(&mut self, p: &Prop) {
        self.snd_break(p.b.xi()); // 0x83A2C: jsr $9e60.l
        // 0x83A40: 4 columns x 6 rows of tile codes from 0x83A76
        let dest = self.assets.enemy.t.u32(0x83AA6, u32::from(p.b.var26));
        self.prop_tile_block(TilePos::at(dest), 4, 6, 0x83A76, TileMode::Codes);
        // 0x83ABA: spawn pool C class 21 (0x54) var 0x26, facing 0
        self.prop_c21_spawn(p.b.xi(), p.b.yi(), p.b.zi(), 0x26, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::enemy::ByteTables;
    use dino_assets::AssetPack;

    fn test_game(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs, ..Default::default() };
        Game::new(Box::leak(Box::new(a)))
    }

    fn cell(g: &Game, addr: u32) -> u16 {
        g.vram.scroll2[((addr - 0x90C000) / 4) as usize].code
    }

    #[test]
    fn b42_burst_and_free() {
        let mut dests = Vec::new();
        // Destination 0x90C708 for var 0
        dests.extend_from_slice(&0x0090C708u32.to_be_bytes());
        let codes: Vec<u8> = (0..24u16).flat_map(|w| (0x6500 + w).to_be_bytes()).collect();
        let blobs = vec![
            (0xA66A, vec![0x10; 48]), // pan table for 0x9E60
            (0x83AA6, dests),
            (0x83A76, codes),
        ];
        let mut g = test_game(blobs);

        let h = g.spawn_prop(42, 0, Some((0x1C8, 0, 0x370))).unwrap();
        let i = match crate::game::slot_of(h).unwrap() {
            crate::game::Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };

        // Pre-fill target cells with known codes
        for col in 0..4 {
            for row in 0..6 {
                let pos = TilePos::at(0x90C708).right(col).down(row);
                g.prop_tile_put(pos, 0x1111, 0x1806);
            }
        }

        // First update: mode 0 sets Y=0x60, fires, advances to mode 6
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 6);
        assert_eq!(g.props.slots[i].b.yi(), 0x60);
        // Break sound played
        assert_eq!(g.w.sounds, vec![0xF6]);
        // C21 var 0x26 spawned
        let fx = g.effects.slots.iter().find(|e| e.b.live != 0).unwrap();
        assert_eq!((fx.b.k20, fx.b.var26, fx.b.facing), (0x54, 0x26, 0));
        assert_eq!((fx.b.xi(), fx.b.yi(), fx.b.zi()), (0x1C8, 0x60, 0x370));

        // 24 tile codes updated, attributes intact
        let base = TilePos::at(0x90C708);
        for col in 0..4 {
            for row in 0..6 {
                let pos = base.right(col).down(row);
                let cell_val = g.vram.scroll2[((pos.0 - 0x90C000) / 4) as usize];
                assert_eq!(cell_val.code, 0x6500 + (col * 6 + row) as u16);
                assert_eq!(cell_val.attr, 0x1806);
            }
        }

        // Second update: freed
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.live, 0);
    }
}
