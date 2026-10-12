//! 0x6606: a downed player lets enemies of the lists A (0x6A5A) and B (0x6A8E) claim him.
//!
//! The thugs of class 10 register themselves in both lists every frame they walk (0x4AF4 /
//! 0x4B26, see `class10_ai.rs`); the lists are rebuilt in every frame (the original clears them
//! between pool E and pool A, `enemies_update` here). The player code calls this routine from
//! the states of a knocked down player (the callers 0x1C290..0x1CD2E), so it reads the lists of the
//! previous frame. A claim sets `+$7E` (list A) or `+$7F` (list B) on both objects and makes the
//! player the target (`+$76`) of the chosen thug; `+$76` of the player takes the thug.

use super::off::*;
use crate::game::{Game, Handle, Slot, slot_of};

impl Game {
    /// Pool A byte `off` of the object `h` (0 for a handle that is no pool A object).
    fn claim_ram_b(&self, h: Handle, off: usize) -> u8 {
        match slot_of(h) {
            Some(Slot::Enemy(i)) => self.enemies.slots[i].m.b(off),
            _ => 0,
        }
    }

    fn claim_ram_set(&mut self, h: Handle, off: usize, v: u8) {
        if let Some(Slot::Enemy(i)) = slot_of(h) {
            self.enemies.slots[i].m.sb(off, v);
        }
    }

    /// 0x6606 for player `pi` (the original's a6).
    pub fn enemy_claim_downed_player(&mut self, pi: usize) {
        let me = Game::player_handle(pi);
        // List A: flag +$7E, the candidates must not carry +$7F.
        if self.players[pi].b7e == 0 {
            let list = self.combat.get(0x6A5A).to_vec();
            if !list.is_empty() {
                if self.claim_pick(pi, &list, 0x7F) && (self.players[pi].w88 as i16) >= 0 {
                    self.players[pi].b7e = 1;
                    let a0 = self.players[pi].w76;
                    self.claim_ram_set(a0, 0x7E, 1);
                    self.claim_ram_set(a0, TARGET, (me >> 8) as u8);
                    self.claim_ram_set(a0, TARGET + 1, me as u8);
                }
            }
        }
        // List B: flag +$7F, the candidates must not carry +$7E.
        if self.players[pi].b7f != 0 {
            return;
        }
        let list = self.combat.get(0x6A8E).to_vec();
        if list.is_empty() {
            return;
        }
        if self.claim_pick(pi, &list, 0x7E) && (self.players[pi].w88 as i16) >= 0 {
            self.players[pi].b7f = 1;
            let a0 = self.players[pi].w76;
            self.claim_ram_set(a0, 0x7F, 1);
            self.claim_ram_set(a0, TARGET, (me >> 8) as u8);
            self.claim_ram_set(a0, TARGET + 1, me as u8);
        }
    }

    /// The loop of 0x6606 (0x6622 / 0x6698): the nearest candidate in X goes to `+$76` of the
    /// player, its distance to `+$88`. A candidate that carries the other flag (`bad_flag`) ends the
    /// whole attempt (returns false).
    fn claim_pick(&mut self, pi: usize, list: &[Handle], bad_flag: usize) -> bool {
        self.players[pi].w88 = 0xFFFF;
        let px = self.players[pi].b.xi();
        for &a1 in list {
            if self.claim_ram_b(a1, bad_flag) != 0 {
                return false;
            }
            let ax = self.body(a1).xi();
            let diff = px.wrapping_sub(ax);
            let d0 = if (diff.wrapping_add(0xC0) as u16) >= 0x180 {
                diff.wrapping_add(0xC0) as u16
            } else {
                diff.unsigned_abs()
            };
            // cmp.w $88(a6),d0 ; bhi: farther than the best so far -> next
            if d0 > self.players[pi].w88 {
                continue;
            }
            self.players[pi].w88 = d0;
            self.players[pi].w76 = a1;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game() -> Game {
        Game::new(Box::leak(Box::new(AssetPack::default())))
    }

    /// Puts a pool A object at handle `h` with X `x` (for the list tests).
    fn place(g: &mut Game, i: usize, x: i16) -> Handle {
        g.enemies.slots[i].b.live = 1;
        g.enemies.slots[i].b.set_xi(x);
        g.enemies.handle(i)
    }

    #[test]
    fn the_nearest_candidate_in_x_is_claimed() {
        let mut g = game();
        g.players[0].b.set_xi(0x100);
        let a = place(&mut g, 0, 0x180); // |dx| 0x80
        let b = place(&mut g, 1, 0x120); // |dx| 0x20
        assert!(g.claim_pick(0, &[a, b], 0x7F));
        assert_eq!((g.players[0].w76, g.players[0].w88), (b, 0x20));
    }

    #[test]
    fn a_far_candidate_keeps_its_biased_signed_distance() {
        // 0x606: beyond 0xC0 the distance is not made positive (+0xC0 stays added): the player at 0x400,
        // the thug at 0x100 gives 0x300 + 0xC0 = 0x3C0 (far), the one at 0x500 gives -0x100 + 0xC0 = 0xFFC0.
        let mut g = game();
        g.players[0].b.set_xi(0x400);
        let a = place(&mut g, 0, 0x100);
        let b = place(&mut g, 1, 0x500);
        assert!(g.claim_pick(0, &[a, b], 0x7F));
        // the first candidate always wins against 0xFFFF; the second is farther (0xFFC0 > 0x3C0)
        assert_eq!((g.players[0].w76, g.players[0].w88), (a, 0x3C0));
    }

    #[test]
    fn a_candidate_with_the_other_flag_ends_the_attempt() {
        let mut g = game();
        g.players[0].b.set_xi(0x100);
        let a = place(&mut g, 0, 0x110);
        g.enemies.slots[0].m.sb(0x7F, 1);
        assert!(!g.claim_pick(0, &[a], 0x7F));
    }
}
