//! Pool B class 21, the one-shot tile attribute change (0x7BDC4).
//!
//! Script action 0x24 spawns it with no position (0x1DECC then 0x7BDAE, `spawn_prop`). The first
//! update (0x7BDD8) clears bits 7 and 8 of 48 attribute words (0x7BDEA, `and #$FE7F`) and then
//! marks 16 cells (0x7BE00). Mode 2 enters at the first of those two writers, so it does the same
//! pair. Mode 4 and mode 6 free the slot (0x7BED6 jumps to 0x4938). There is no ROM tile table:
//! the masks are the immediates of the instructions.

use super::tile_block::TilePos;
use super::Prop;
use crate::game::Game;

/// The 16 marks of 0x7BE00, as cell addresses. The first fifteen `ori` #$2000. The last, at
/// 0x7BECE, `ori` #$2400. A mark is `(attr & #$3FF) | mask`.
const MARKS: &[(u32, u16)] = &[
    (0x90F02C, 0x2000),
    (0x90F030, 0x2000),
    (0x90F034, 0x2000),
    (0x90F06C, 0x2000),
    (0x90F070, 0x2000),
    (0x90F074, 0x2000),
    (0x90F0AC, 0x2000),
    (0x90F0B0, 0x2000),
    (0x90F0B4, 0x2000),
    (0x90F0EC, 0x2000),
    (0x90F0F0, 0x2000),
    (0x90F0F4, 0x2000),
    (0x90F12C, 0x2000),
    (0x90F130, 0x2000),
    (0x90F16C, 0x2000),
    (0x90F170, 0x2400),
];

impl Game {
    /// 0x7BDC4.
    pub fn prop_b21(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 | 2 => {
                // 0x7BDD8 does `addq.b #2` and then both writers. Mode 2 is the jump-table entry
                // at the first `bsr` (0x7BDDC), so it writes too. The clear runs first: a marked
                // cell inside the 48 is cleared and then marked.
                self.prop_tile_attr_block(TilePos::at(0x90F000), 1, 48, 0xFE7F, 0); // 0x7BDEA
                for &(cell, mask) in MARKS {
                    self.prop_tile_attr(TilePos::at(cell), 0x3FF, mask); // 0x7BE00 / 0x7BECE
                }
                p.b.mode = 4;
            }
            _ => self.prop_free(p, i), // 0x7BED6
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game() -> Game {
        Game::new(Box::leak(Box::new(AssetPack::default())))
    }

    fn attr(g: &Game, addr: u32) -> u16 {
        g.vram.scroll2[((addr - 0x90C000) / 4) as usize].attr
    }

    fn code(g: &Game, addr: u32) -> u16 {
        g.vram.scroll2[((addr - 0x90C000) / 4) as usize].code
    }

    fn fill(g: &mut Game) {
        // 0x90F000 through the last mark at 0x90F170 (92 cells) and one cell past it.
        for n in 0..94 {
            let i = ((0x90F000 - 0x90C000) / 4) as usize + n;
            g.vram.scroll2[i] = dino_assets::stage::Cell { code: 0x1111, attr: 0xFFFF };
        }
    }

    /// From attribute 0xFFFF: a cleared cell is 0xFE7F. A marked cell inside the 48 becomes
    /// 0x227F (the clear removes bits 7 and 8, then the mark keeps the low 10 bits). A marked
    /// cell past the 48 becomes 0x23FF, and the last mark is 0x27FF.
    #[test]
    fn the_first_update_clears_48_and_marks_16() {
        let mut g = game();
        fill(&mut g);
        g.spawn_prop(21, 0, None).unwrap();
        g.pool_b_update();
        assert_eq!(attr(&g, 0x90F000), 0xFE7F, "cleared, not marked");
        assert_eq!(attr(&g, 0x90F0BC), 0xFE7F, "the 48th cell");
        assert_eq!(attr(&g, 0x90F02C), 0x227F, "marked inside the clear");
        assert_eq!(attr(&g, 0x90F0B4), 0x227F);
        assert_eq!(attr(&g, 0x90F0C0), 0xFFFF, "just past the clear, not a mark");
        assert_eq!(attr(&g, 0x90F0EC), 0x23FF, "marked past the clear");
        assert_eq!(attr(&g, 0x90F16C), 0x23FF);
        assert_eq!(attr(&g, 0x90F170), 0x27FF, "the #$2400 mark");
        assert_eq!(code(&g, 0x90F000), 0x1111, "codes stay");
        assert_eq!(code(&g, 0x90F170), 0x1111);
        let p = g.props.slots.iter().find(|s| s.b.k20 == 21 * 4).unwrap();
        assert_eq!((p.b.live, p.b.mode), (1, 4));
        g.pool_b_update();
        assert!(g.props.slots.iter().all(|s| s.b.live == 0 || s.b.k20 != 21 * 4));
    }

    /// Mode 2 is the same two writers (0x7BDDC). A second pass does not change the result.
    #[test]
    fn mode_2_writes_the_same_attributes() {
        let mut g = game();
        fill(&mut g);
        g.spawn_prop(21, 0, None).unwrap();
        {
            let p = g.props.slots.iter_mut().find(|s| s.b.k20 == 21 * 4).unwrap();
            p.b.mode = 2;
        }
        g.pool_b_update();
        assert_eq!(attr(&g, 0x90F000), 0xFE7F);
        assert_eq!(attr(&g, 0x90F02C), 0x227F);
        assert_eq!(attr(&g, 0x90F170), 0x27FF);
        g.pool_b_update();
        g.spawn_prop(21, 0, None).unwrap();
        {
            let p = g.props.slots.iter_mut().find(|s| s.b.live != 0 && s.b.k20 == 21 * 4).unwrap();
            p.b.mode = 2;
        }
        g.pool_b_update();
        assert_eq!(attr(&g, 0x90F02C), 0x227F, "a second pass stays");
        assert_eq!(attr(&g, 0x90F170), 0x27FF);
    }
}
