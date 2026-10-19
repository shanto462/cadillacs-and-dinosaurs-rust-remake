//! Pool B class 53 (0x85306): fire glow palette set.
//! Spec: D3e part 3 section 2.1, section 5.3 (A29, W319).
//!
//! Spawned in Episode 8 Area 2 (List 1, trigger 0) and Area 3 (List 1, trigger 0xD60).
//! Whenever G.$7B2 changes (set to 2 while a hero lies burning, 0 otherwise), it updates
//! 27 palettes (370 colours, palettes 65 to 122 in scroll2 and scroll3 palette banks):
//! warm red/orange fire glow tones while burning, restored to normal when the hero stands.

use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x85306: pool B class 53 update.
    pub fn prop_b53(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x8531A: mode 0 init
                p.b.mode = 2; // 0x8531A: addq.b #2, $4(a6)
                self.spawn.g.b7b2 = 0; // 0x8531E: clr.b $7b2(a5)
                self.spawn.g.b7b3 = 0; // 0x85322: clr.b $7b3(a5)
            }
            2 => {
                // 0x85328: mode 2
                let b7b2 = self.spawn.g.b7b2;
                let b7b3 = self.spawn.g.b7b3;
                if b7b2 != b7b3 {
                    // 0x85334..0x8533C: offset from table 0x8534C
                    let off = self.assets.enemy.t.u16(0x8534C, u32::from(b7b2.wrapping_mul(2)));
                    self.prop_b53_apply(u32::from(off));
                    self.spawn.g.b7b3 = self.spawn.g.b7b2; // 0x85342..0x85346: move.b d0,$7b3(a5)
                }
            }
            _ => self.prop_free(p, i), // 0x85452: jmp 0x4938
        }
    }

    /// 0x85354: apply 27 palettes (16 colours each) to palette RAM.
    fn prop_b53_apply(&mut self, off: u32) {
        for k in 0..27u32 {
            let tbl_ptr = self.assets.enemy.t.u32(0x8537A, 8 * k);
            let dest = self.assets.enemy.t.u32(0x8537A, 8 * k + 4);
            let pal_ptr = self.assets.enemy.t.u32(tbl_ptr, off);
            self.prop_palette_copy(dest, pal_ptr, 16);
        }
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

    #[test]
    fn b53_palette_switch_and_restore() {
        // Build test tables for B53:
        // Table 0x8534C: [0x0000, 0x0004, 0x0008, 0x0008]
        let off_table: Vec<u8> = vec![0x00, 0x00, 0x00, 0x04, 0x00, 0x08, 0x00, 0x08];

        // 1 destination palette: 0x914820 (pal 65).
        // For testing we will populate 27 entries all pointing to tbl_0 (0x85458) and dest 0x914820..
        let mut tbl_pointers = Vec::new();
        for k in 0..27u32 {
            let tbl = 0x85458u32;
            let dest = 0x914820u32 + 32 * k;
            tbl_pointers.extend_from_slice(&tbl.to_be_bytes());
            tbl_pointers.extend_from_slice(&dest.to_be_bytes());
        }

        // Table 0x85458: 3 pointers for off=0, 4, 8:
        // [pal_normal (0xBF672), pal_mid (0xC0272), pal_fire (0xC0352)]
        let pal_table: Vec<u8> = vec![
            0x00, 0x0B, 0xF6, 0x72, // 0: normal
            0x00, 0x0C, 0x02, 0x72, // 4: mid
            0x00, 0x0C, 0x03, 0x52, // 8: fire
        ];

        // 16 colours each:
        let normal_colors: Vec<u8> = (0..16u16).flat_map(|c| (0x0100 + c).to_be_bytes()).collect();
        let mid_colors: Vec<u8> = (0..16u16).flat_map(|c| (0x0200 + c).to_be_bytes()).collect();
        let fire_colors: Vec<u8> = (0..16u16).flat_map(|c| (0x0300 + c).to_be_bytes()).collect();

        let blobs = vec![
            (0x8534C, off_table),
            (0x8537A, tbl_pointers),
            (0x85458, pal_table),
            (0xBF672, normal_colors),
            (0xC0272, mid_colors),
            (0xC0352, fire_colors),
        ];

        let mut g = test_game(blobs);

        let h = g.spawn_prop(53, 0, None).unwrap();
        let i = match crate::game::slot_of(h).unwrap() {
            crate::game::Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };

        // Mode 0: clears b7b2, b7b3, mode becomes 2
        g.spawn.g.b7b2 = 5;
        g.spawn.g.b7b3 = 5;
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 2);
        assert_eq!(g.spawn.g.b7b2, 0);
        assert_eq!(g.spawn.g.b7b3, 0);

        // Pre-fill palette RAM with brightness nibble F and color 0
        for w in &mut g.pal.words {
            *w = 0xF000;
        }

        // Frame with b7b2 unchanged (0 == 0): no write
        g.pal.dirty = false;
        g.pool_b_update();
        assert!(!g.pal.dirty);
        assert_eq!(g.pal.words[1040], 0xF000);

        // Set b7b2 = 2 (hero burning): applies fire palette (off = 8)
        g.spawn.g.b7b2 = 2;
        g.pool_b_update();
        assert!(g.pal.dirty);
        assert_eq!(g.spawn.g.b7b3, 2);
        // Palette 65 (word index 1040..1055): preserves top nibble 0xF000 and has fire color 0x0300
        assert_eq!(g.pal.words[1040], 0xF300);
        assert_eq!(g.pal.words[1041], 0xF301);

        // Set b7b2 = 0 (hero recovers): applies normal palette (off = 0)
        g.spawn.g.b7b2 = 0;
        g.pool_b_update();
        assert_eq!(g.spawn.g.b7b3, 0);
        assert_eq!(g.pal.words[1040], 0xF100);
        assert_eq!(g.pal.words[1041], 0xF101);
    }
}
