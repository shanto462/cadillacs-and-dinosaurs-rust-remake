//! Pool B class 11, the metal end door of Episode 1 area 0 (0x789EE).
//!
//! Script action 0x40 spawns it at (0x670, 0, 0x118) (0x1DD30 then 0x789C8, `spawn_prop`). The
//! handler is the burst driver: armed wait 3 (0x78A20), then the swap of 0x78A38. Mode 6 frees
//! (0x78B34 jumps to 0x4938). 0x78AB8 and 0x78B2A have no caller.

use super::drivers::{BurstAction, BurstAttr, BurstBlock, BurstEffect, BurstSound, BurstSpec, BurstWait};
use super::tile_block::{TileMode, TilePos};
use super::Prop;
use crate::game::Game;

/// 0x78A5A: three columns of seven code words at scroll2 0x90E9E4, table 0x78A8E.
const TILES: &[BurstBlock] = &[BurstBlock {
    at: TilePos::at(0x90E9E4),
    cols: 3,
    rows: 7,
    src: 0x78A8E,
    mode: TileMode::Codes,
}];

/// 0x78AF0: eight rows at the attribute of 0x90EA66 (`and #$FE7F ; or #$80`), then seven columns
/// of eight rows with `or #$100`. The second cell is the next column (attribute 0x90EAA6).
const ATTRS: &[BurstAttr] = &[
    BurstAttr { at: TilePos::at(0x90EA66), cols: 1, rows: 8, keep: 0xFE7F, or_bits: 0x80 },
    BurstAttr { at: TilePos::at(0x90EAA6), cols: 7, rows: 8, keep: 0xFE7F, or_bits: 0x100 },
];

const SPEC: BurstSpec<'static> = BurstSpec {
    wait: BurstWait::Armed(3),
    action: BurstAction {
        tiles: TILES,
        attrs: ATTRS,
        effect: Some(BurstEffect { var: 2, facing: 0 }),
        sound: BurstSound::Break,
    },
};

impl Game {
    /// 0x789EE. Mode 0 is 0x78A02 (visible, body box, HP, then the armed wait stores 3 at +$80 and
    /// returns). Mode 2 is 0x78A2C (`subq.b #1,+$80`, and a zero count only advances to mode 4).
    /// Mode 4 is 0x78A38: break sound 0x9E60, the tile block, the attribute block, pool C class 21
    /// with d0 = 2 and d1 = 0 (0x94640). The driver plays the sound after the tiles; neither call
    /// uses `rand`, so the request is the same.
    pub fn prop_b11(&mut self, p: &mut Prop, i: usize) {
        if p.b.mode == 0 {
            // 0x78A04..0x78A1A. No box table at +$44, so this is not `hit_init`.
            p.b.visible = 1;
            p.b.body_box = 1;
            p.b.attack_box = 0;
            p.b.hp = 1;
            p.b.hp_prev = 1;
        }
        self.prop_burst_step(p, i, &SPEC);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::enemy::ByteTables;
    use dino_assets::AssetPack;

    fn words(w: &[u16]) -> Vec<u8> {
        w.iter().flat_map(|v| v.to_be_bytes()).collect()
    }

    fn game(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs };
        Game::new(Box::leak(Box::new(a)))
    }

    fn cell(g: &Game, addr: u32) -> (u16, u16) {
        let c = g.vram.scroll2[((addr - 0x90C000) / 4) as usize];
        (c.code, c.attr)
    }

    fn set_attr(g: &mut Game, addr: u32, attr: u16) {
        g.vram.scroll2[((addr - 0x90C000) / 4) as usize].attr = attr;
    }

    /// Twenty-one door codes and the pan byte of 0x9E60. X 0 reads pan byte 0.
    fn door_game() -> Game {
        let codes: Vec<u16> = (0..21).map(|k| 0x3100 + k).collect();
        game(vec![(0xA66A, vec![0x10; 48]), (0x78A8E, words(&codes))])
    }

    fn paint_attrs(g: &mut Game) {
        let base = TilePos::at(0x90EA66);
        for col in 0..8 {
            for row in 0..8 {
                set_attr(g, base.right(col).down(row).0, 0xFFFF);
            }
        }
    }

    /// Armed(3): the swap is the 5th update (0x78A20, 0x78A2C, 0x78A38). The 6th frees.
    #[test]
    fn the_metal_door_opens_on_the_fifth_update() {
        let mut g = door_game();
        paint_attrs(&mut g);
        g.spawn_prop(11, 0, Some((0x670, 0, 0x118))).unwrap();
        g.pool_b_update();
        let p = g.props.slots.iter().find(|s| s.b.k20 == 11 * 4).unwrap();
        assert_eq!(p.b.mode, 2, "mode 0 stores the wait and returns");
        assert_eq!(
            (p.b.visible, p.b.body_box, p.b.attack_box, p.b.hp, p.b.hp_prev, p.m.b(0x80)),
            (1, 1, 0, 1, 1, 3)
        );
        assert_eq!((p.b.xi(), p.b.yi(), p.b.zi()), (0x670, 0, 0x118));
        for _ in 0..3 {
            g.pool_b_update();
        }
        assert_eq!(cell(&g, 0x90E9E4).0, 0, "still closed after four updates");
        let p = g.props.slots.iter().find(|s| s.b.k20 == 11 * 4).unwrap();
        assert_eq!(p.b.mode, 4, "the zero count only advances");
        g.pool_b_update();
        let at = TilePos::at(0x90E9E4);
        for col in 0..3 {
            for row in 0..7 {
                let (code, _) = cell(&g, at.right(col).down(row).0);
                assert_eq!(code, 0x3100 + (col * 7 + row) as u16, "col {col} row {row}");
            }
        }
        let attr_at = TilePos::at(0x90EA66);
        for row in 0..8 {
            assert_eq!(cell(&g, attr_at.down(row).0).1, 0xFEFF, "first column row {row}");
        }
        for col in 1..8 {
            for row in 0..8 {
                assert_eq!(cell(&g, attr_at.right(col).down(row).0).1, 0xFF7F, "col {col} row {row}");
            }
        }
        let fx = g.effects.slots.iter().find(|e| e.b.live != 0).unwrap();
        assert_eq!((fx.b.k20, fx.b.var26, fx.b.facing), (0x54, 2, 0));
        assert_eq!((fx.b.xi(), fx.b.yi(), fx.b.zi()), (0x670, 0, 0x118));
        assert_eq!(g.w.sounds, vec![0xF6]);
        let p = g.props.slots.iter().find(|s| s.b.k20 == 11 * 4).unwrap();
        assert_eq!(p.b.mode, 6);
        g.pool_b_update();
        assert!(g.props.slots.iter().all(|s| s.b.k20 != 11 * 4 || s.b.live == 0));
    }
}
