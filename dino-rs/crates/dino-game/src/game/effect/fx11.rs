//! Pool C class 11 (+$20 = 0x2C), handler 0x934F6: the debris of a broken container or weapon. The spawn sites are in the item code (0x259E8 the bat,
//! 0x292A6 and 0x294AA the containers); they set the variant (+$26, the picture), the speeds and, for the pieces that may turn into a stone, +$5C = 1.
//!
//! The handler has a mode 0 init by variant and four run states by +$05:
//! * variants 0 (a bat piece, the frame list 0x937D0), 2 and 4 (the two pieces of a thrown container, the lists 0x93872 / 0x9387E): state 0, 60 updates,
//!   a full bounce, drawn in every second update by the timer;
//! * every other variant (6 .. 0x22: an animation of the table 0x937AA by variant / 2, a life of 30 to 93 updates from the random number): state 2
//!   below 0xC (a half bounce, the animation ends it, drawn by a counting blink), state 4 from 0xC (a half bounce, drawn on odd frames), state 6
//!   when +$5C is set (like state 4, but the piece ends as a stone weapon, 0x93770, after its life when it lies on the floor).
//!
//! The floor snap and the bounce run only in the updates where `G.$0 + d7` is odd (d7 is the loop counter of the pool update 0x237B2: 0x2F - slot).
//! +$58 is the life (a word, bit 0 of its low byte, +$59, decides the drawing of state 0), +$5A / +$5C the blink counters of state 2.

use super::*;

/// 0x937AA: the debris animations by variant / 2 (the entries 0, 1 and 2 are the lists of variants 0, 2 and 4).
const DEBRIS_TABLE: u32 = 0x937AA;
/// The frame lists of the first three variants (0x9353A, 0x93564, 0x9356C).
const DEBRIS_LISTS: [u32; 3] = [0x937D0, 0x93872, 0x9387E];

impl Game {
    /// 0x934F6: modes 0 (init, falls into the run) and 2; the modes 4 and 6 free the slot (0x937A4).
    pub(super) fn fx_debris(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => self.fx_debris_init(i),
            2 => self.fx_debris_run(i),
            _ => self.fx_free(i),
        }
    }

    /// 0x9350A: the init by variant (the jump table 0x93516), then 0x935DE.
    fn fx_debris_init(&mut self, i: usize) {
        let fx = self.fx_data();
        let var = self.effects.slots[i].b.var26;
        if var <= 4 {
            // 0x9353A (var 0): speeds of its own; 0x93564 / 0x9356C (var 2, 4): the spawn site made them. 0x93570: the list, up in the air, 60 updates.
            let e = &mut self.effects.slots[i];
            fx_load(&mut e.b, fx, fx.frame_at(DEBRIS_LISTS[(var / 2) as usize]));
            if var == 0 {
                e.b.vy = 0x100;
                e.b.ay = -0x10;
            }
            e.b.airborne = 1;
            e.owner58 = 0x3C;
            e.b.mode = e.b.mode.wrapping_add(2);
        } else {
            // 0x9358A
            let e = &mut self.effects.slots[i];
            e.b.anim_table = DEBRIS_TABLE;
            fx_start(&mut e.b, fx, var >> 1);
            e.b.airborne = 1;
            let life = (self.rand() & 0x3F) + 0x1E; // andi.w #$3F ; addi.w #$1E
            let e = &mut self.effects.slots[i];
            e.owner58 = life;
            e.b.mode = e.b.mode.wrapping_add(2);
            e.b.sub = 4;
            if var < 0xC {
                e.b.sub = 2;
            }
            if e.w5c != 0 {
                e.b.sub = 6;
            }
        }
        // 0x935DE
        let e = &mut self.effects.slots[i];
        e.owner_class = 1;
        e.w5c = 1;
        e.b.draw_flags = 0x10;
        self.fx_debris_run(i);
    }

    /// 0x935F2: the run state by +$05 (the jump table 0x935FE).
    fn fx_debris_run(&mut self, i: usize) {
        match self.effects.slots[i].b.sub {
            0 => self.fx_debris_s0(i),
            2 => self.fx_debris_s2(i),
            4 => self.fx_debris_s4(i),
            _ => self.fx_debris_s6(i),
        }
    }

    /// The bounce shared by the states (0x93618, 0x93658, 0x936B6, 0x93706): in every second update (`G.$0 + d7` odd) the floor snap 0x12DAC, and a
    /// piece that has landed (+$51 = 0) turns its vertical speed round, halved when `halve` (`neg.w` and `asr.w #1`; the state 0 only negates).
    /// Returns true when the piece is on the floor after the snap in an update that ran the check.
    fn fx_debris_bounce(&mut self, i: usize, halve: bool) -> bool {
        let parity = (self.w.frame.wrapping_add(47 - i as u16)) & 1; // move.w G.$0, d0 ; add.w d7, d0 ; andi.w #1, d0
        if parity == 0 {
            return false;
        }
        let mut b = std::mem::take(&mut self.effects.slots[i].b);
        self.terrain_effect_floor(&mut b); // jsr 0x12DAC
        let landed = b.airborne == 0;
        if landed {
            b.vy = b.vy.wrapping_neg();
            if halve {
                b.vy >>= 1;
            }
        }
        self.effects.slots[i].b = b;
        landed
    }

    /// 0x93606 state 0.
    fn fx_debris_s0(&mut self, i: usize) {
        let e = &mut self.effects.slots[i];
        let (life, borrow) = e.owner58.overflowing_sub(1); // subq.w #1, $58 ; bcs 0x937A4
        e.owner58 = life;
        if borrow {
            return self.fx_free(i);
        }
        self.fx_debris_bounce(i, false);
        let fx = self.fx_data();
        fx_tick(&mut self.effects.slots[i].b, fx); // jsr 0x15F4
        self.effects.slots[i].b.motion(); // jsr 0x189C
        if self.effects.slots[i].owner58 & 1 != 0 {
            return; // btst #0, $59(a6) ; bne rts
        }
        self.draw_list_add_check(self.effects.handle(i)); // jmp 0x49CA
    }

    /// 0x93652 state 2: the animation ends the piece; it is drawn when the blink counter +$5A runs out (the pause grows: +$5C counts up).
    fn fx_debris_s2(&mut self, i: usize) {
        self.fx_debris_bounce(i, true);
        let fx = self.fx_data();
        if fx_tick(&mut self.effects.slots[i].b, fx) {
            return self.fx_free(i); // bcs 0x937A4
        }
        let e = &mut self.effects.slots[i];
        e.b.motion();
        e.owner_class = e.owner_class.wrapping_sub(1); // subq.w #1, $5A
        if e.owner_class != 0 {
            return; // bne rts
        }
        e.w5c = e.w5c.wrapping_add(1);
        e.owner_class = e.w5c >> 1; // lsr.w #1
        self.draw_list_add_check(self.effects.handle(i));
    }

    /// 0x936A8 state 4: a life of +$58 updates, drawn on odd frames.
    fn fx_debris_s4(&mut self, i: usize) {
        let e = &mut self.effects.slots[i];
        let (life, borrow) = e.owner58.overflowing_sub(1);
        e.owner58 = life;
        if borrow {
            return self.fx_free(i);
        }
        self.fx_debris_bounce(i, true);
        let fx = self.fx_data();
        fx_tick(&mut self.effects.slots[i].b, fx);
        self.effects.slots[i].b.motion();
        if self.w.frame & 1 == 0 {
            return; // btst #0, $1(a5) ; beq rts
        }
        self.draw_list_add_check(self.effects.handle(i));
    }

    /// 0x936F4 state 6: like state 4, but the life stops at 0 and the piece that is on the floor then becomes a stone weapon (0x93770) and ends.
    fn fx_debris_s6(&mut self, i: usize) {
        if self.effects.slots[i].owner58 != 0 {
            self.effects.slots[i].owner58 -= 1;
        }
        let landed = self.fx_debris_bounce(i, true);
        if landed && self.effects.slots[i].owner58 == 0 {
            // 0x93730: mode 6 (the next update frees the slot), the size of the stone by the variant, then 0x93770.
            let e = &mut self.effects.slots[i];
            e.b.mode = 6;
            e.b.sub = 0;
            let size = match e.b.var26 {
                0x20 => 0,
                0x22 => 1,
                v if v > 0x1A => 1, // bhi
                _ => 2,
            };
            let pos = (e.b.xi(), e.b.yi(), e.b.zi());
            self.item_spawn_stone(pos, size);
            return;
        }
        let fx = self.fx_data();
        fx_tick(&mut self.effects.slots[i].b, fx);
        self.effects.slots[i].b.motion();
        self.draw_list_add_check(self.effects.handle(i));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rom() -> Option<&'static dino_assets::AssetPack> {
        static A: std::sync::OnceLock<Option<&'static dino_assets::AssetPack>> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let path = std::path::Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("ROM loads"))))
        })
    }

    fn spawn(g: &mut Game, var: u16, w5c: u16) -> usize {
        let i = g.effects.alloc().unwrap();
        let e = &mut g.effects.slots[i];
        e.b.live = 1;
        e.b.k20 = FX_DEBRIS;
        e.b.var26 = var;
        e.w5c = w5c;
        e.b.set_yi(0x40);
        i
    }

    #[test]
    fn a_bat_piece_lives_60_updates_and_the_init_draws_no_random_number() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        let seed = g.w.rng;
        let i = spawn(&mut g, 0, 0);
        for n in 0..60 {
            g.pool_c_update();
            assert_eq!(g.effects.slots[i].b.live, 1, "update {n}");
        }
        g.pool_c_update();
        assert_eq!(g.effects.slots[i].b.live, 0);
        assert_eq!(g.w.rng, seed);
    }

    #[test]
    fn a_numbered_piece_rolls_once_for_its_life_and_a_bonus_piece_ends_as_a_stone() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        let mut expected = Game::new(a);
        let life = (expected.rand() & 0x3F) + 0x1E;
        let i = spawn(&mut g, 0x20, 1);
        g.pool_c_update();
        assert_eq!(g.w.rng, expected.w.rng);
        let e = &g.effects.slots[i];
        assert_eq!((e.b.sub, e.b.mode), (6, 2));
        assert_eq!(e.owner58, life - 1, "the life is counted down in the first update");
        let before = g.items.free_count();
        g.effects.slots[i].b.ay = -0x30; // it falls to the floor
        for _ in 0..200 {
            g.w.frame = g.w.frame.wrapping_add(1);
            g.pool_c_update();
            if g.effects.slots[i].b.mode == 6 {
                break;
            }
        }
        assert_eq!(g.effects.slots[i].b.mode, 6, "the piece ends");
        assert_eq!(g.items.free_count(), before - 1, "and leaves a stone");
        let stone = g.items.slots.iter().find(|s| s.b.live != 0).unwrap();
        assert_eq!((stone.b.k20, stone.b.var26, stone.m.w(crate::game::item::o::W5C)), (0, 0x1C, 0), "variant 0x20 is the small stone");
        g.pool_c_update();
        assert_eq!(g.effects.slots[i].b.live, 0, "the next update frees the slot");
    }
}
