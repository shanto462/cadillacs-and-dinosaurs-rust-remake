//! Pool B class 46 (handler 0x8419E, maker 0x84188): the wall break at the right end of the Episode 8 clear scene (M9-04). The flare of the stand in (pool C 39, 0x97530)
//! makes it when it hits X 0x12D0 (the only caller of 0x84188, found by the static search of M9-04). Its first update does all: the three columns of tile codes (9, 10 and
//! 10 cells, the list 0x84204) at 0x90EB08, 0x90EB48 and 0x90EB8C on scroll 2 and the break sound (0x9E60, the pan of its own X, which is 0); the next update frees it.

use super::tile_block::{TileMode, TilePos};
use super::Prop;
use crate::game::Game;

/// 0x84204: the tile codes, column by column (9 + 10 + 10 words); 0x841D0 writes them at 0x90EB08 with the next column 0x40 on.
const CODES: u32 = 0x84204;
const AT: u32 = 0x90EB08;

impl Game {
    /// 0x84188: a pool B object of the class 46 (`+$20` 0xB8); a full pool makes none.
    pub(crate) fn prop_spawn_b46(&mut self) {
        let Some(i) = self.props.alloc() else { return };
        let p = &mut self.props.slots[i];
        p.b.live = 1;
        p.b.k20 = 46 * 4;
    }

    /// 0x8419E (mode table 0x841AC): 0 and 2 go on to mode 4 at once (0x841B4, 0x841B8), 4 the break (0x841BC), 6 the free (0x8424C, jmp 0x4938).
    pub fn prop_b46(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 | 2 | 4 => {
                // 0x841D0: the three columns of codes
                let at = TilePos::at(AT);
                self.prop_tile_block(at, 1, 9, CODES, TileMode::Codes);
                self.prop_tile_block(at.right(1), 1, 10, CODES + 18, TileMode::Codes);
                // the address runs on: 10 cells of the second column + 0x1C leave the third column one cell lower (0x90EB8C, not 0x90EB88)
                self.prop_tile_block(TilePos::at(AT + 0x84), 1, 10, CODES + 38, TileMode::Codes);
                self.snd_break(p.b.xi()); // jsr 0x9E60
                p.b.mode = 6;
            }
            _ => self.prop_free(p, i),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn rom() -> Option<&'static AssetPack> {
        static A: std::sync::OnceLock<Option<&'static AssetPack>> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let path = std::path::Path::new("../../dino.zip");
            let path = if path.exists() { path } else { std::path::Path::new("../../../dino.zip") };
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    fn code(g: &Game, addr: u32) -> u16 {
        g.vram.scroll2[((addr - 0x90C000) / 4) as usize].code
    }

    /// The first update writes the three columns (9, 10 and 10 codes of the list 0x84204) at 0x90EB08, 0x90EB48 and 0x90EB8C (the third one cell lower: MAME ep8_jack record
    /// 64623) and queues the break sound; the next update frees the object.
    #[test]
    fn the_wall_break_writes_its_three_columns_at_once_and_goes() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let mut g = Game::new(a);
        g.prop_spawn_b46();
        g.pool_b_update();
        let list = |k: u32| a.enemy.t.u16(CODES, 2 * k);
        for k in 0..9 {
            assert_eq!(code(&g, 0x90EB08 + 4 * k), list(k), "column 0 cell {k}");
        }
        for k in 0..10 {
            assert_eq!(code(&g, 0x90EB48 + 4 * k), list(9 + k), "column 1 cell {k}");
            assert_eq!(code(&g, 0x90EB8C + 4 * k), list(19 + k), "column 2 cell {k}");
        }
        assert!(g.w.sounds.iter().any(|&s| s == 0x6F || s == 0xF6), "the break sound");
        let i = g.props.slots.iter().position(|p| p.b.k20 == 46 * 4).unwrap();
        assert_eq!(g.props.slots[i].b.mode, 6);
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.live, 0, "freed by the next update");
    }
}
