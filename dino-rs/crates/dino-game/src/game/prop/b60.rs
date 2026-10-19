//! Pool B class 60 (0x863AC): Episode 6 Area 0 mine lamps palette cycling.
//! Spec: D3e part 3 sections 2.1, 2.2, A06, W304.
//!
//! Two scroll2 palettes cycle in 4 steps of 9 frames (36 frame cycle):
//! - Palette 80 (0x914A00): 16 colours from list A (0x86476)
//! - Palette 81, colours 7..14 (0x914A2E): 8 colours from list B (0x86488)
//!
//! Active only in stage 5 area 0 (cmpi.b #5, $4D9; tst.b $786). When outside, frees itself.

use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x863AC: pool B class 60 update (mine lamps).
    pub fn prop_b60(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x863C0: init
                p.m.sb(0xA5, 8); // 0x863C0: move.b #$8, $a5(a6)
                p.m.sl(0xA0, 0x86476); // 0x863C6: move.l #$86476, $a0(a6)
                p.m.sl(0xB0, 0x86488); // 0x863CE: move.l #$86488, $b0(a6)
                p.m.sl(0xA8, 0x86476); // 0x863D6: move.l $a0(a6), $a8(a6)
                p.m.sl(0xB4, 0x86488); // 0x863DC: move.l $b0(a6), $b4(a6)
                p.m.sl(0xAC, 0x914A00); // 0x863E2: move.l #$914a00, $ac(a6)
                p.m.sb(0xA4, 0); // 0x863EA: clr.b $a4(a6)
                p.b.mode = 2; // 0x863EE: addq.b #2, $4(a6)
            }
            2 => {
                // 0x863F4: stage/area guard (cmpi.b #5, $4D9; tst.b $786)
                if self.w.stage != 5 || self.cam.s2.area != 0 {
                    p.b.mode = 4;
                    return;
                }
                // 0x8640C: subq.b #1, $a4(a6) ; bcs 0x86414
                let n = p.m.b(0xA4).wrapping_sub(1);
                p.m.sb(0xA4, n);
                if n != 0xFF {
                    return;
                }
                // 0x86414: move.b $a5(a6), $a4(a6)
                p.m.sb(0xA4, p.m.b(0xA5));

                // 0x8641A: Lane A (palette 80)
                let mut ptr_a = p.m.l(0xA0);
                if self.assets.enemy.t.i16(ptr_a, 0) < 0 {
                    ptr_a = p.m.l(0xA8);
                }
                let set_a = self.assets.enemy.t.u32(ptr_a, 0);
                ptr_a += 4;
                p.m.sl(0xA0, ptr_a);
                let dest_a = p.m.l(0xAC);
                self.prop_palette_copy(dest_a, set_a, 16);

                // 0x86448: Lane B (palette 81, colours 7..14)
                let mut ptr_b = p.m.l(0xB0);
                if self.assets.enemy.t.i16(ptr_b, 0) < 0 {
                    ptr_b = p.m.l(0xB4);
                }
                let set_b = self.assets.enemy.t.u32(ptr_b, 0);
                ptr_b += 4;
                p.m.sl(0xB0, ptr_b);
                self.prop_palette_copy(0x914A2E, set_b, 8);
            }
            _ => {
                // 0x86470: jmp 0x4938
                self.prop_free(p, i);
            }
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
        let mut g = Game::new(Box::leak(Box::new(a)));
        g.w.stage = 5;
        g.cam.s2.area = 0;
        g
    }

    fn longs(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|v| v.to_be_bytes()).collect()
    }

    fn words(w: &[u16]) -> Vec<u8> {
        w.iter().flat_map(|v| v.to_be_bytes()).collect()
    }

    #[test]
    fn b60_cycles_every_9_frames() {
        let mut list_a = longs(&[0xBE052, 0xBEA52, 0xBEA72, 0xBEA92]);
        list_a.extend_from_slice(&0xFFFFu16.to_be_bytes());
        let mut list_b = longs(&[0x8649A, 0x864AA, 0x864BA, 0x864CA]);
        list_b.extend_from_slice(&0xFFFFu16.to_be_bytes());
        let mut set_0 = vec![0x0321u16; 16];
        set_0[5] = 0x0CBA;
        let mut set_1 = vec![0x0661u16; 16];
        set_1[5] = 0x0FFF;
        let mut set_2 = vec![0x0541u16; 16];
        set_2[5] = 0x0EDC;
        let mut set_3 = vec![0x0431u16; 16];
        set_3[5] = 0x0DCB;

        let b_sets = vec![0x0111u16; 32]; // 4 steps of 8 colours

        let blobs = vec![
            (0x86476, list_a),
            (0x86488, list_b),
            (0xBE052, words(&set_0)),
            (0xBEA52, words(&[set_1, set_2, set_3].concat())),
            (0x8649A, words(&b_sets)),
        ];
        let mut g = test_game(blobs);

        // Pre-fill palette 80 with initial map brightness 0xF000
        for k in 0..16 {
            g.pal.words[((0x914A00 - 0x914000) / 2 + k) as usize] = 0xF000;
        }

        let h = g.spawn_prop(60, 0, None).unwrap();
        let i = match crate::game::slot_of(h).unwrap() {
            crate::game::Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };

        // Frame 0: mode 0 inits, mode becomes 2
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 2);

        // Frame 1: mode 2 executes first cycle step (set 0 written at t=0 of running)
        g.pool_b_update();
        let c0 = g.pal.words[((0x914A00 - 0x914000) / 2) as usize];
        assert_eq!(c0, 0xF321);

        // Frames 2..9: countdown runs (8 frames without change)
        for _ in 0..8 {
            g.pool_b_update();
            let c = g.pal.words[((0x914A00 - 0x914000) / 2) as usize];
            assert_eq!(c, 0xF321);
        }

        // Frame 10 (9 frames after first step): steps to set 1 (F661)
        g.pool_b_update();
        let c1 = g.pal.words[((0x914A00 - 0x914000) / 2) as usize];
        assert_eq!(c1, 0xF661);
    }

    #[test]
    fn b60_frees_when_leaving_stage5_area0() {
        let blobs = vec![
            (0x86476, vec![0; 16]),
            (0x86488, vec![0; 16]),
        ];
        let mut g = test_game(blobs);

        let h = g.spawn_prop(60, 0, None).unwrap();
        let i = match crate::game::slot_of(h).unwrap() {
            crate::game::Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };

        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 2);

        // Change area to 1
        g.cam.s2.area = 1;
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 4);

        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.live, 0);
    }
}
