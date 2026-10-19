//! Pool B class 48 (0x84316): Episode 5 cliff no-op.
//! Spec: D3e part 2 s7, A216, W222, P210.
//!
//! Waits 128 updates, walks the end states, then frees. The nearby tile writer at 0x84346 is never
//! called. This is not the falling-rock spawner.

use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x84316: pool B class 48 update (jump table 0x84322).
    pub fn prop_b48(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x8432A: addq.b #2,+$04 ; move.w #$80,+$A0 ; fall into the wait
                p.b.mode = 2;
                p.m.sw(0xA0, 0x80);
                self.prop_b48_wait(p);
            }
            2 => self.prop_b48_wait(p), // 0x84334
            4 => p.b.mode = 6,          // 0x84340: one addq.b #2
            _ => self.prop_free(p, i),  // 0x84388: jmp 0x4938
        }
    }

    /// 0x84334: `subq.w #1,+$A0 ; beq` into two `addq.b #2` (mode 2 becomes 6).
    fn prop_b48_wait(&mut self, p: &mut Prop) {
        let t = p.m.w(0xA0).wrapping_sub(1);
        p.m.sw(0xA0, t);
        if t == 0 {
            p.b.mode = 6;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{slot_of, Slot};
    use dino_assets::AssetPack;

    fn test_game() -> Game {
        Game::new(Box::leak(Box::new(AssetPack::default())))
    }

    #[test]
    fn b48_waits_128_calls_then_frees() {
        let mut g = test_game();
        let h = g.spawn_prop(48, 0, Some((0x810, 0, 0x295))).unwrap();
        let i = match slot_of(h).unwrap() {
            Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };

        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 2);
        assert_eq!(g.props.slots[i].m.w(0xA0), 0x7F);
        assert_eq!(g.props.slots[i].b.live, 1);

        for n in 0..126 {
            g.pool_b_update();
            assert_eq!(g.props.slots[i].b.mode, 2, "still waiting after {} more calls", n + 1);
            assert_eq!(g.props.slots[i].b.live, 1);
        }

        // 128th call: timer hits 0, two addq fall-throughs leave mode 6
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 6);
        assert_eq!(g.props.slots[i].b.live, 1);

        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.live, 0);
    }
}
