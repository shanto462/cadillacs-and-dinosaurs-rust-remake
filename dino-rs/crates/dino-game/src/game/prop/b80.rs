//! Pool B class 80 (`+$20 = 0x140`, handler 0x8A582): the jungle sound of Episode 2, made by the stage
//! start. Mode 0 (0x8A596) sets mode 2 and the word `+$A0` to 0x78, then falls into the countdown.
//! Mode 2 (0x8A5A2) counts that word down. At 0 it reloads 0x78 (0x8A5AA) and draws a number: when
//! `rand() & 0xF` is below 4, a second `rand()` decides. Bit 0 set plays sound 0x78 through 0xA63C
//! (d1 = 0, d2 = 0). Bit 0 clear (0x8A5D0) loads 0x61 into d0 and returns without the request, so the
//! second random is not drawn when the first nibble is 4 or more. Mode 4 and mode 6 free it (0x8A5F4).

use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x8A582.
    pub fn prop_b80(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode = 2; // 0x8A596
                p.m.sw(0xA0, 0x78);
            }
            2 => {}
            _ => return self.prop_free(p, i), // 0x8A5F4
        }
        // 0x8A5A2, also the fall-through of mode 0.
        let left = p.m.w(0xA0).wrapping_sub(1);
        p.m.sw(0xA0, left);
        if left != 0 {
            return;
        }
        p.m.sw(0xA0, 0x78); // 0x8A5AA
        if self.rand() & 0xF < 4 && self.rand() & 1 != 0 {
            self.snd_request(0x78, 0, 0); // 0x8A5E0, jsr 0xA63C
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::snd::SndRequest;
    use dino_assets::AssetPack;

    fn game() -> Game {
        Game::new(Box::leak(Box::new(AssetPack::default())))
    }

    fn live(g: &Game) -> Option<&Prop> {
        g.props.slots.iter().find(|s| s.b.live != 0 && s.b.k20 == 80 * 4)
    }

    fn arm(g: &mut Game, rng: u16) {
        let i = g.props.slots.iter().position(|s| s.b.k20 == 80 * 4).unwrap();
        g.props.slots[i].m.sw(0xA0, 1);
        g.w.rng = rng;
        g.w.sounds.clear();
        g.spawn.g.snd_ring.clear();
    }

    /// Mode 0 falls into the countdown, so one update leaves the word at 0x77 and plays nothing.
    #[test]
    fn the_first_update_only_counts() {
        let mut g = game();
        g.spawn_prop(80, 0, None).unwrap();
        g.pool_b_update();
        let p = live(&g).unwrap();
        assert_eq!((p.b.mode, p.m.w(0xA0)), (2, 0x77));
        assert!(g.w.sounds.is_empty());
    }

    /// Seed 1: both draws return 1, and the odd one requests 0x78. Seed 2: the even branch loads 0x61
    /// and requests nothing. Seed 0: the first nibble is 8, so the second draw does not run.
    #[test]
    fn only_the_odd_second_draw_requests_0x78() {
        let mut g = game();
        g.spawn_prop(80, 0, None).unwrap();
        g.pool_b_update();

        arm(&mut g, 1);
        g.pool_b_update();
        assert_eq!(g.w.sounds, vec![0x78]);
        assert_eq!(g.spawn.g.snd_ring, vec![SndRequest { code: 0x78, d1: 0, d2: 0 }]);
        assert_eq!(live(&g).unwrap().m.w(0xA0), 0x78);
        assert_eq!(g.w.rng, 1);

        arm(&mut g, 2);
        g.pool_b_update();
        assert!(g.w.sounds.is_empty(), "the even branch does not request 0x61");
        assert_eq!(g.w.rng, 2, "the second draw did run");

        arm(&mut g, 0);
        g.pool_b_update();
        assert!(g.w.sounds.is_empty());
        // First draw from seed 0 is 0x5C8 (low nibble 8). The next draw would be 0x11D9.
        assert_eq!(g.w.rng, 0x05C8, "the second draw is skipped when the nibble is 4 or more");
    }

    #[test]
    fn mode_4_frees_it() {
        let mut g = game();
        g.spawn_prop(80, 0, None).unwrap();
        g.pool_b_update();
        let i = g.props.slots.iter().position(|s| s.b.k20 == 80 * 4).unwrap();
        g.props.slots[i].b.mode = 4;
        g.pool_b_update();
        assert!(live(&g).is_none());
    }
}
