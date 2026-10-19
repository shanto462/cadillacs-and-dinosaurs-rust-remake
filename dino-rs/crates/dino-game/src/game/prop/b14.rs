//! Pool B class 14, the building hole and the wooden door of Episode 1 area 2 (0x7B0BE).
//!
//! Variant 0 (0x7B0DA) is the hole: hold 8 (0x7B0E2), then four columns of seven codes at
//! scroll2 0x90C10C (0x7B108) and pool C class 21 variant 4 (0x7B182). Variant 4 (0x7B258) is
//! the wooden door: the first update moves to (0x40, 0x10, 0x120) and swaps (0x7B260 falls into
//! 0x7B27A). Any other variant frees the slot (the pointer table at 0x7B0D2 has only these two).
//! 0x7B18C (the glass shards) has no caller.

use super::drivers::{BurstAction, BurstBlock, BurstEffect, BurstSound, BurstSpec, BurstWait};
use super::tile_block::{TileMode, TilePos};
use super::Prop;
use crate::game::Game;

/// 0x7B108: four columns of seven code words, table 0x7B14A.
const HOLE_TILES: &[BurstBlock] = &[BurstBlock {
    at: TilePos::at(0x90C10C),
    cols: 4,
    rows: 7,
    src: 0x7B14A,
    mode: TileMode::Codes,
}];

/// 0x7B2A4: four columns of nine code words, table 0x7B2E6.
const DOOR_TILES: &[BurstBlock] = &[BurstBlock {
    at: TilePos::at(0x90E110),
    cols: 4,
    rows: 9,
    src: 0x7B2E6,
    mode: TileMode::Codes,
}];

const HOLE: BurstSpec<'static> = BurstSpec {
    wait: BurstWait::Hold(8),
    action: BurstAction {
        tiles: HOLE_TILES,
        attrs: &[],
        effect: Some(BurstEffect { var: 4, facing: 1 }),
        sound: BurstSound::Quiet,
    },
};

/// Mode 0 falls through the empty mode 2 into the swap (0x7B260, 0x7B276, 0x7B27A), so the
/// first update fires. `Now` is that fall-through. A loaded mode 2 is advanced to mode 4 before
/// the call, because `Now` on mode 2 would only set mode 4.
const DOOR: BurstSpec<'static> = BurstSpec {
    wait: BurstWait::Now,
    action: BurstAction {
        tiles: DOOR_TILES,
        attrs: &[],
        effect: Some(BurstEffect { var: 6, facing: 1 }),
        sound: BurstSound::Request { code: 0x88, d1: 0x32000, d2: 0x1C00_0000 },
    },
};

impl Game {
    /// 0x7B0BE: `+$26` picks the routine, then the mode jump of that routine.
    pub fn prop_b14(&mut self, p: &mut Prop, i: usize) {
        match p.b.var26 {
            0 => {
                self.prop_burst_step(p, i, &HOLE);
            }
            4 => {
                if p.b.mode == 0 {
                    // 0x7B264, before the fall into the swap. The effect copies this position.
                    p.b.set_xi(0x40);
                    p.b.set_yi(0x10);
                    p.b.set_zi(0x120);
                } else if p.b.mode == 2 {
                    // 0x7B276 is `addq.b #2` and falls into 0x7B27A.
                    p.b.mode = 4;
                }
                self.prop_burst_step(p, i, &DOOR);
            }
            _ => self.prop_free(p, i),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::snd::SndRequest;
    use dino_assets::enemy::ByteTables;
    use dino_assets::AssetPack;

    fn words(w: &[u16]) -> Vec<u8> {
        w.iter().flat_map(|v| v.to_be_bytes()).collect()
    }

    fn game() -> Game {
        let hole: Vec<u16> = (0..28).map(|k| 0x4100 + k).collect();
        let door: Vec<u16> = (0..36).map(|k| 0x5100 + k).collect();
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs: vec![(0x7B14A, words(&hole)), (0x7B2E6, words(&door))], ..Default::default() };
        Game::new(Box::leak(Box::new(a)))
    }

    fn code(g: &Game, addr: u32) -> u16 {
        g.vram.scroll2[((addr - 0x90C000) / 4) as usize].code
    }

    fn live(g: &Game, var: u16) -> Option<&Prop> {
        g.props.slots.iter().find(|s| s.b.live != 0 && s.b.k20 == 14 * 4 && s.b.var26 == var)
    }

    /// Hold(8): the hole is written on the 9th update (0x7B0E2, 0x7B0F8). The 10th frees.
    #[test]
    fn the_hole_opens_on_the_ninth_update() {
        let mut g = game();
        g.spawn_prop(14, 0, Some((0x60, 0x290, 0x120))).unwrap();
        for _ in 0..8 {
            g.pool_b_update();
        }
        assert_eq!(code(&g, 0x90C10C), 0, "still the wall after eight updates");
        assert_eq!(live(&g, 0).unwrap().b.mode, 4);
        assert_eq!(live(&g, 0).unwrap().m.w(0xA0), 0);
        g.pool_b_update();
        let at = TilePos::at(0x90C10C);
        for col in 0..4 {
            for row in 0..7 {
                assert_eq!(code(&g, at.right(col).down(row).0), 0x4100 + (col * 7 + row) as u16);
            }
        }
        let p = live(&g, 0).unwrap();
        assert_eq!((p.b.xi(), p.b.yi(), p.b.zi(), p.b.mode), (0x60, 0x290, 0x120, 6));
        let fx = g.effects.slots.iter().find(|e| e.b.live != 0).unwrap();
        assert_eq!((fx.b.k20, fx.b.var26, fx.b.facing), (0x54, 4, 1));
        assert_eq!((fx.b.xi(), fx.b.yi(), fx.b.zi()), (0x60, 0x290, 0x120));
        assert!(g.w.sounds.is_empty(), "the hole plays no sound");
        g.pool_b_update();
        assert!(live(&g, 0).is_none());
    }

    /// Variant 4 fires on its first update, at the position 0x7B264 writes, with sound 0x88.
    #[test]
    fn the_wooden_door_bursts_on_the_first_update() {
        let mut g = game();
        g.spawn_prop(14, 4, Some((0x100, 0, 0x40))).unwrap();
        g.pool_b_update();
        let at = TilePos::at(0x90E110);
        for col in 0..4 {
            for row in 0..9 {
                assert_eq!(code(&g, at.right(col).down(row).0), 0x5100 + (col * 9 + row) as u16);
            }
        }
        let p = live(&g, 4).unwrap();
        assert_eq!((p.b.xi(), p.b.yi(), p.b.zi(), p.b.mode), (0x40, 0x10, 0x120, 6));
        let fx = g.effects.slots.iter().find(|e| e.b.k20 == 0x54).unwrap();
        assert_eq!((fx.b.var26, fx.b.facing, fx.b.xi(), fx.b.yi(), fx.b.zi()), (6, 1, 0x40, 0x10, 0x120));
        assert_eq!(
            g.snd_take_requests(),
            vec![SndRequest { code: 0x88, d1: 0x32000, d2: 0x1C00_0000 }]
        );
        g.pool_b_update();
        assert!(live(&g, 4).is_none());
    }

    /// A loaded mode 2 falls into the swap and does not write the position again (0x7B276).
    #[test]
    fn a_wooden_door_already_in_mode_2_fires_where_it_stands() {
        let mut g = game();
        g.spawn_prop(14, 4, Some((9, 8, 7))).unwrap();
        {
            let p = g.props.slots.iter_mut().find(|s| s.b.k20 == 14 * 4).unwrap();
            p.b.mode = 2;
        }
        g.pool_b_update();
        let p = live(&g, 4).unwrap();
        assert_eq!((p.b.xi(), p.b.yi(), p.b.zi(), p.b.mode), (9, 8, 7, 6));
        assert_eq!(code(&g, 0x90E110), 0x5100);
    }

    #[test]
    fn an_unknown_variant_frees_the_slot() {
        let mut g = game();
        g.spawn_prop(14, 1, None).unwrap();
        g.pool_b_update();
        assert!(g.props.slots.iter().all(|s| s.b.live == 0 || s.b.k20 != 14 * 4));
    }
}
