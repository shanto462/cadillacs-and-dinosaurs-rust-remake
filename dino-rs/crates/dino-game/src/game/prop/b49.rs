//! Pool B class 49 (0x8438E): Episode 5 village wall burst at wave 6.
//! Spec: D3e part 2 s7, A222, W237, W238, P211.
//!
//! Init raises Y by 0x50, plays the break sound, writes 20 scroll2 tile codes (4 columns of 5
//! rows) and four attribute bits 0x40, then makes pool C class 21 variant 0x16 and frees.
//! The dialogue makers at 0x8446C / 0x84490 are not called.

use super::tile_block::{TileMode, TilePos};
use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x8438E: pool B class 49 update (jump table 0x8439A).
    pub fn prop_b49(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x843A2: addq.b #2 ; addi.w #$50,+$0C ; fall through
                p.b.mode = 2;
                p.b.set_yi(p.b.yi().wrapping_add(0x50));
                self.prop_b49_sound_and_swap(p);
            }
            2 => self.prop_b49_sound_and_swap(p), // 0x843AC
            4 => self.prop_b49_swap(p),           // 0x843B6
            _ => self.prop_free(p, i),            // 0x84466: jmp 0x4938
        }
    }

    /// 0x843AC: break sound 0x9E60, then the tile swap.
    fn prop_b49_sound_and_swap(&mut self, p: &mut Prop) {
        self.snd_break(p.b.xi());
        self.prop_b49_swap(p);
    }

    /// 0x843B6: 0x843C8 tile codes, 0x843FA attributes, 0x8445C class 21, mode 6.
    fn prop_b49_swap(&mut self, p: &mut Prop) {
        let dest = self.assets.enemy.t.u32(0x84454, u32::from(p.b.var26));
        let at = TilePos::at(dest);
        // 0x843C8: 4 columns of 5 code words from 0x84424, column by column, top to bottom
        self.prop_tile_block(at, 4, 5, 0x84424, TileMode::Codes);
        // 0x843FA: four `ori.w #$40` on the attribute words
        self.prop_tile_attr(at, 0xFFFF, 0x40);
        self.prop_tile_attr(at.down(1), 0xFFFF, 0x40);
        self.prop_tile_attr(at.right(3).down(2), 0xFFFF, 0x40);
        self.prop_tile_attr(at.right(3).down(3), 0xFFFF, 0x40);
        // 0x8445C: moveq #$16,d0 ; moveq #0,d1 ; jmp 0x94640
        self.prop_c21_spawn(p.b.xi(), p.b.yi(), p.b.zi(), 0x16, 0);
        p.b.mode = 6;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{slot_of, Slot};
    use dino_assets::enemy::ByteTables;
    use dino_assets::AssetPack;

    fn test_game(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs, ..Default::default() };
        Game::new(Box::leak(Box::new(a)))
    }

    fn cell(g: &Game, addr: u32) -> (u16, u16) {
        let c = g.vram.scroll2[((addr - 0x90C000) / 4) as usize];
        (c.code, c.attr)
    }

    #[test]
    fn b49_writes_twenty_codes_four_attrs_and_c21_then_frees() {
        let codes: Vec<u8> = (0..20u16).flat_map(|w| (0x6400 + w).to_be_bytes()).collect();
        let mut dests = Vec::new();
        dests.extend_from_slice(&0x0090E010u32.to_be_bytes());
        dests.extend_from_slice(&0x0090F40Cu32.to_be_bytes());
        let blobs = vec![
            (0xA66A, vec![0x10; 48]),
            (0x84424, codes),
            (0x84454, dests),
        ];
        let mut g = test_game(blobs);
        for c in &mut g.vram.scroll2 {
            c.code = 0x1111;
            c.attr = 0x0001;
        }

        let h = g.spawn_prop(49, 0, Some((0x1000, 0, 0x170))).unwrap();
        let i = match slot_of(h).unwrap() {
            Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };

        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 6);
        assert_eq!(g.props.slots[i].b.yi(), 0x50);
        assert_eq!(g.w.sounds, vec![0xF6]);

        let dest = 0x90E010u32;
        let mut wrote = 0u16;
        for col in 0..4 {
            for row in 0..5 {
                let addr = dest + col * 0x40 + row * 4;
                let (code, attr) = cell(&g, addr);
                assert_eq!(code, 0x6400 + wrote, "code at {addr:#x}");
                wrote += 1;
                let expect_bit = matches!((col, row), (0, 0) | (0, 1) | (3, 2) | (3, 3));
                if expect_bit {
                    assert_eq!(attr, 0x0041, "attr bit 0x40 at {addr:#x}");
                } else {
                    assert_eq!(attr, 0x0001, "attr unchanged at {addr:#x}");
                }
            }
        }

        let fx: Vec<_> = g
            .effects
            .slots
            .iter()
            .filter(|e| e.b.live != 0 && e.b.k20 == 0x54)
            .collect();
        assert_eq!(fx.len(), 1);
        assert_eq!(fx[0].b.var26, 0x16);
        assert_eq!(fx[0].b.yi(), 0x50);
        assert_eq!(fx[0].b.xi(), 0x1000);
        assert_eq!(fx[0].b.zi(), 0x170);

        assert!(!g.props.slots.iter().any(|p| p.b.live != 0 && p.b.k20 == 50 * 4));

        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.live, 0);
    }
}
