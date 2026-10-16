//! Pool B class 47 (0x84282), the manhole hole patch.
//!
//! Spawned by B35 mode 0 (0x81FFC / 0x8426A) during Script 21. Mode 0 advances through mode 2 to mode 4
//! in the first frame, writing the 6x2 tile code patch at 0x90F86C from ROM table 0x842F6.
//! It stays in mode 4, writing the patch every frame, until G.$4F0 (floor-only / script flag) is set.
//! When G.$4F0 is set, it modifies the 6x3 tile attributes at 0x90F872 (0x842CC: AND #$FEFF, OR #$0100),
//! advances to mode 6, and frees itself (0x84310 -> 0x4938).

use super::tile_block::{TileMode, TilePos};
use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x84282 pool B class 47 update.
    pub fn prop_b47(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 | 2 => {
                // 0x84296: addq.b #2, $4(a6); 0x8429a: addq.b #2, $4(a6)
                p.b.mode = 4;
                self.prop_b47_step4(p, i);
            }
            4 => self.prop_b47_step4(p, i),
            _ => self.prop_free(p, i), // 0x84310 -> 0x4938
        }
    }

    /// 0x8429E: mode 4 handler.
    fn prop_b47_step4(&mut self, p: &mut Prop, _i: usize) {
        // 0x842AE: 6 columns x 2 rows of tile code words at 0x90F86C
        self.prop_tile_block(TilePos::at(0x90F86C), 6, 2, 0x842F6, TileMode::Codes);
        // 0x842A0: tst.b $4f0(a5)
        if self.w.floor_only != 0 {
            // 0x842CC: 6 columns x 3 rows of attributes at 0x90F872
            self.prop_tile_attr_block(TilePos::at(0x90F872), 6, 3, 0xFEFF, 0x0100);
            p.b.mode = 6; // 0x842A8: addq.b #2, $4(a6)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game(blobs: &[(u32, Vec<u8>)]) -> Game {
        let mut pack = AssetPack::default();
        pack.enemy.t.blobs = blobs.to_vec();
        Game::new(Box::leak(Box::new(pack)))
    }

    fn cell(g: &Game, addr: u32) -> (u16, u16) {
        let c = g.vram.scroll2[((addr - 0x90C000) / 4) as usize];
        (c.code, c.attr)
    }

    #[test]
    fn b47_writes_codes_in_mode_4_and_attributes_when_floor_only_set() {
        let codes: Vec<u8> = (0..24u8).collect();
        let blobs = vec![(0x842F6, codes)];
        let mut g = game(&blobs);

        // Pre-fill target cells with known codes and attributes
        for col in 0..6 {
            for row in 0..3 {
                let pos = TilePos::at(0x90F86C).right(col).down(row);
                g.prop_tile_put(pos, 0xAAAA, 0x00FF);
            }
        }

        let h = g.spawn_prop(47, 0, None).unwrap();
        let i = match crate::game::slot_of(h).unwrap() {
            crate::game::Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };

        // First update: mode 0 -> 4, writes codes at 0x90F86C
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 4);
        assert_eq!(cell(&g, 0x90F86C).0, 0x0001); // first word of table
        assert_eq!(cell(&g, 0x90F86C).1, 0x00FF, "attributes unchanged yet");

        // Stays in mode 4 while floor_only == 0
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 4);

        // Set G.$4F0 (floor_only = 1) -> writes attributes and advances to mode 6
        g.w.floor_only = 1;
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 6);
        // Attribute at 0x90F872: (0x00FF & 0xFEFF) | 0x0100 = 0x01FF
        assert_eq!(cell(&g, 0x90F872).1, 0x01FF);

        // Next frame in mode 6: freed
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.live, 0, "freed after mode 6");
    }
}
