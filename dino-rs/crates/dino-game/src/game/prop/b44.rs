//! Pool B class 44, the lab tank breaker of Episode 8 area 0 (handler 0x83BB0, jump table 0x83BBC): part 1 of M8-19.
//!
//! The script makes four of them (D3d section 5.1): variant 0x0000 (cylinder 01, a Bludge), 0x0400 (cylinder 03, a Rock Hopper), 0x0600
//! (cylinder 04, a Bludge) and 0x1400 (cylinder 11, Tyrog 2). The high byte of the variant `H` chooses the cylinder (`H / 2 + 1`), the creature
//! row of the table 0x83C36 (`H / 2`) and the video address of the glass block (the long table 0x83DD8). The event is three frames:
//! * mode 0 (0x83BC4): a low byte of the variant makes a random gate (`rand() & 31 < 0x16` frees the object: none of the four records has one), Y = 0x18, mode 2,
//!   and the same frame goes on in
//! * mode 2 (0x83BEA): mode 4, the glass effect (pool C class 21 variant 0x24 at the object: `prop_c21_spawn`, which makes the 10 pieces in the next
//!   updates of the pool C), then a pool A slot with the position, the class word and the variant of the row and the drop byte +$96 of the breaker (a full pool
//!   frees the breaker);
//! * mode 4 (0x83C62): the cylinder is rewritten (6 columns of 7 tile codes from 0x83D84, the attribute bits of the rim 0x83D46) and, for the Hopper and
//!   Tyrog 2 tanks only (the flag table 0x83C96), the palette bits of 18 cells (0x83CE2, words 0x83D22) and the sound 0x37; mode 6;
//! * mode 6 (0x83E06): freed (0x4938).
//!
//! The pieces of the glass (class 21 variant 0x24 and 0x0E to 0x14) and the creatures' entrance are part 2 (M8-20); the class 21 handler of M5-04 already
//! makes them from its variant table.

use super::tile_block::{TileMode, TilePos};
use super::Prop;
use crate::game::Game;

/// 0x83C36: the creature rows (class word, variant), 4 bytes a row, row = H / 2.
const CREATURES: u32 = 0x83C36;
/// 0x83C96: 1 for the tanks that get the palette change and the sound (by H / 2).
const RECOLOUR_FLAGS: u32 = 0x83C96;
/// 0x83D22: the 18 palette words of the recolour (6 columns of 3 cells).
const RECOLOUR_WORDS: u32 = 0x83D22;
/// 0x83D84: the 42 tile codes of the broken cylinder (6 columns of 7).
const BROKEN_TILES: u32 = 0x83D84;
/// 0x83DD8: the video address of the block of each cylinder (a long by H / 2).
const CYLINDERS: u32 = 0x83DD8;

impl Game {
    /// 0x83BB0.
    pub fn prop_b44(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x83BC4: the random gate of a variant with a low byte (the script has none).
                if p.b.var26 & 0xFF != 0 && ((self.rand() & 0x1F) as i16) < 0x16 {
                    return self.prop_free(p, i); // 0x83E06
                }
                p.b.mode += 2; // 0x83BE0
                p.b.set_yi(0x18);
                self.b44_break(p, i); // falls into 0x83BEA
            }
            2 => self.b44_break(p, i),
            4 => self.b44_tiles(p),
            _ => self.prop_free(p, i), // 0x83E06: jmp 0x4938
        }
    }

    /// 0x83BEA: the glass and the creature.
    fn b44_break(&mut self, p: &mut Prop, i: usize) {
        p.b.mode += 2;
        self.prop_c21_spawn(p.b.xi(), p.b.yi(), p.b.zi(), 0x24, 0); // jsr 0x94640 with d0 = 0x24, d1 = 0
        let Some(slot) = self.enemies.alloc() else {
            return self.prop_free(p, i); // 0x483C carry: bcs 0x83E06
        };
        let row = ((p.b.var26 >> 8) & 0xFF) * 2; // lsr.w #8 ; add.w d0,d0: a byte offset of the 4 byte rows
        let (class_word, variant) = {
            let t = &self.assets.enemy.t;
            (t.u16(CREATURES, row as u32), t.u16(CREATURES, row as u32 + 2))
        };
        let e = &mut self.enemies.slots[slot];
        e.b.live = 1;
        e.b.set_xi(p.b.xi());
        e.b.set_yi(p.b.yi());
        e.b.set_zi(p.b.zi());
        e.b.k20 = class_word;
        e.b.var26 = variant;
        e.m.sb(super::super::enemy::off::DROP, p.m.b(0x96));
    }

    /// 0x83C62: the empty cylinder, the recolour and the sound.
    fn b44_tiles(&mut self, p: &mut Prop) {
        let h = (p.b.var26 >> 8) & 0xFF;
        let base = self.assets.enemy.t.u32(CYLINDERS, h as u32 * 2) & 0xFF_FFFF; // movea.l (a0, d0.w), a0
        // 0x83CA6: 6 columns of 7 codes (the cells 4 bytes apart, the columns 0x40 apart).
        self.prop_tile_block(TilePos::at(base), 6, 7, BROKEN_TILES, TileMode::Codes);
        // 0x83D46: the attribute bits of the rim: 0x20 on six cells, the palette 0x17 on one of them.
        for (offset, palette) in [(0xD6, false), (0xDA, false), (0x116, true), (0x11A, false), (0x156, false), (0x15A, false)] {
            let at = TilePos::at(base + offset);
            if palette {
                self.prop_tile_attr(at, 0xFFE0, 0x17);
            }
            self.prop_tile_attr(at, 0xFFFF, 0x20);
        }
        if self.assets.enemy.t.u8(RECOLOUR_FLAGS, (h >> 1) as u32) != 0 {
            self.b44_recolour(base);
            // 0x83C7C: sound 0x37 (d1 = 0x72010, d2 = 0x2002200 through 0xA63C).
            self.snd_request(0x37, 0x72010, 0x0200_2200);
        }
        p.b.mode += 2;
    }

    /// 0x83CE2: the palette of 6 columns of 3 cells (the rows 5 to 7 of the block) from the words 0x83D22.
    fn b44_recolour(&mut self, base: u32) {
        for column in 0..6u32 {
            for row in 0..3u32 {
                let word = self.assets.enemy.t.u16(RECOLOUR_WORDS, (column * 3 + row) * 2);
                self.prop_tile_attr(TilePos::at(base + 0x16 + 0x40 * column + 4 * row), 0xFFE0, word);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::enemy::ByteTables;
    use dino_assets::AssetPack;

    /// Synthetic tables (no ROM bytes): the cylinder 0 block at the cell 0x90C100, the cylinder 2 block at 0x90C500.
    fn game(var: u16) -> (Game, usize) {
        let be16 = |v: &[u16]| v.iter().flat_map(|w| w.to_be_bytes()).collect::<Vec<u8>>();
        let mut rows = vec![0u16; 22];
        rows[0..2].copy_from_slice(&[0x18, 0]); // row 0: class 6, variant 0
        rows[4..6].copy_from_slice(&[0x74, 0x600]); // row 2: class 29, variant 0x600
        let mut flags = vec![0u8; 12];
        flags[2] = 1;
        let mut cylinders = vec![0u8; 44];
        cylinders[0..4].copy_from_slice(&0x90C100u32.to_be_bytes());
        cylinders[8..12].copy_from_slice(&0x90C500u32.to_be_bytes());
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables {
            blobs: vec![
                (CREATURES, be16(&rows)),
                (RECOLOUR_FLAGS, flags),
                (RECOLOUR_WORDS, be16(&(0x10..0x22).collect::<Vec<u16>>())),
                (BROKEN_TILES, be16(&(0x100..0x12A).collect::<Vec<u16>>())),
                (CYLINDERS, cylinders),
            ],
            ..Default::default()
        };
        let mut g = Game::new(Box::leak(Box::new(a)));
        let i = g.props.alloc().unwrap();
        let p = &mut g.props.slots[i];
        p.b.live = 1;
        p.b.k20 = 44 * 4;
        p.b.var26 = var;
        p.b.set_xi(0x150);
        p.b.set_zi(0x6C);
        p.m.sb(0x96, 3);
        (g, i)
    }

    fn cell(g: &Game, addr: u32) -> (u16, u16) {
        let c = g.vram.scroll2[((addr - 0x90C000) / 4) as usize];
        (c.code, c.attr)
    }

    #[test]
    fn three_frames_glass_creature_tiles_recolour_and_sound() {
        let (mut g, i) = game(0x0400);
        g.pool_b_update(); // t = 90: state 0 -> 4 in the same frame as the break
        assert_eq!((g.props.slots[i].b.mode, g.props.slots[i].b.yi()), (4, 0x18));
        let fx: Vec<_> = g.effects.slots.iter().filter(|e| e.b.live != 0).collect();
        assert_eq!((fx.len(), fx[0].b.k20, fx[0].b.var26, fx[0].b.xi(), fx[0].b.zi()), (1, 0x54, 0x24, 0x150, 0x6C));
        let e = g.enemies.slots.iter().find(|e| e.b.live != 0).expect("the creature");
        assert_eq!((e.b.k20, e.b.var26, e.b.xi(), e.b.yi(), e.b.zi(), e.m.b(0x96)), (0x74, 0x600, 0x150, 0x18, 0x6C, 3));
        assert_eq!(cell(&g, 0x90C500), (0, 0), "the tiles come one frame later");
        g.pool_b_update(); // t = 91: the cylinder
        assert_eq!(g.props.slots[i].b.mode, 6);
        // 6 columns of 7 codes in the order of the table, the columns 0x40 apart
        assert_eq!(cell(&g, 0x90C500).0, 0x100);
        assert_eq!(cell(&g, 0x90C500 + 4 * 6).0, 0x106);
        assert_eq!(cell(&g, 0x90C500 + 0x40).0, 0x107);
        assert_eq!(cell(&g, 0x90C500 + 5 * 0x40 + 4 * 6).0, 0x129);
        // the rim: 0x20 on six cells, the palette 0x17 on (col 4, row 5); the recolour words in 3 rows of 6 columns
        assert_eq!(cell(&g, 0x90C500 + 0xD6).1 & 0x20, 0x20);
        assert_eq!(cell(&g, 0x90C500 + 0x116).1 & 0x3F, 0x3C, "the recolour (word 12) comes after the rim: 0x20 | 0x1C");
        assert_eq!(cell(&g, 0x90C500 + 0x16).1, 0x10);
        assert_eq!(cell(&g, 0x90C500 + 0x16 + 0x40 * 5 + 8).1, 0x21);
        let requests = g.snd_take_requests();
        assert!(requests.iter().any(|r| r.code == 0x37 && r.d1 == 0x72010 && r.d2 == 0x0200_2200));
        g.pool_b_update(); // t = 92: freed
        assert_eq!(g.props.slots[i].b.live, 0);
    }

    #[test]
    fn a_bludge_tank_makes_no_sound_and_no_recolour() {
        let (mut g, _) = game(0);
        g.pool_b_update();
        g.pool_b_update();
        assert_eq!(cell(&g, 0x90C100).0, 0x100);
        assert_eq!(cell(&g, 0x90C100 + 0x16).1, 0, "no recolour");
        assert_eq!(cell(&g, 0x90C100 + 0x116).1 & 0x3F, 0x37, "the rim keeps the palette 0x17");
        assert!(g.snd_take_requests().iter().all(|r| r.code != 0x37));
    }

    #[test]
    fn a_low_byte_makes_the_random_gate() {
        let (mut g, i) = game(0x0001);
        g.w.rng = 1; // rand() & 31 = 1 < 0x16: freed at once, no glass, no creature
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.live, 0);
        assert!(g.effects.slots.iter().all(|e| e.b.live == 0));
        assert!(g.enemies.slots.iter().all(|e| e.b.live == 0));
    }
}
