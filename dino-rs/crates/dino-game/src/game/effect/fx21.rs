//! Pool C class 21 (+$20 = 0x54), handler 0x94672: a puff or a shower of pieces. Made by 0x94640 (`prop_c21_spawn`, `fx_spawn_class21`, the container
//! and the barrel breaks of the item code) with a variant (+$26, an even number below 0x2C) and a facing.
//!
//! One handler runs two kinds of object, told by +$04 / +$05 (the dispatch 0x9467E by mode, then 0x94692 by sub):
//! * the maker (mode 0, sub 0 then 2): reads its list of the table 0x9484A by variant (a word offset table; a block of `ay` word, `+$60` word and 12 byte
//!   records `delay, animation, dx, dy, dz, -, vx, vy, vz` ended by a byte of 0x80 or more), keeps the record pointer in the long +$58 and the delay in +$5C
//!   (a byte), and makes one piece per record: a new object of the same class in mode 0, sub 4, with the maker's position plus the record's offsets, `ay` and
//!   +$60 of the maker, the record's speeds (vx negated for a maker that faces left) and its animation as the variant. A delay of 0 makes the next piece in
//!   the same frame. The maker frees itself (mode 6) after the last record;
//! * a piece (sub 4 first, then mode 2): the animation of the table 0x951AC, a life of `rand() & 0x1F` + the maker's +$60 updates (+$5E), moves by 0x189C,
//!   one floor snap 0x12DAC, and bounces on the floor at half the speed. It is drawn every update at first; the counters +$62 and +$63 (8 or 9, from the
//!   parity of d7) make the pause between drawings grow, then it blinks (every second update by `G.$0 + d7`).
//!
//! d7 is the loop counter of the pool update 0x237B2 (0x2F - slot).

use super::*;

/// 0x9484A: the puff lists by variant.
const PUFF_LISTS: u32 = 0x9484A;
/// 0x951AC: the animation table of the pieces (0x94776).
const PUFF_ANIMS: u32 = 0x951AC;

impl Effect {
    /// The long +$58 (the record pointer of a maker): +$58 / +$5A as one big-endian long.
    fn p58(&self) -> u32 {
        ((self.owner58 as u32) << 16) | self.owner_class as u32
    }

    fn set_p58(&mut self, a: u32) {
        self.owner58 = (a >> 16) as u16;
        self.owner_class = a as u16;
    }

    /// The byte +$5C (the high byte of the word).
    fn b5c(&self) -> u8 {
        (self.w5c >> 8) as u8
    }

    fn set_b5c(&mut self, v: u8) {
        self.w5c = (self.w5c & 0x00FF) | ((v as u16) << 8);
    }
}

impl Game {
    /// 0x94672.
    pub(super) fn fx_puff(&mut self, i: usize) {
        let (mode, sub) = (self.effects.slots[i].b.mode, self.effects.slots[i].b.sub);
        match (mode, sub) {
            (0, 0) => {
                self.fx_puff_maker_init(i);
                self.fx_puff_maker_run(i); // 0x946B6 falls into 0x946BA
            }
            (0, 2) => self.fx_puff_maker_run(i),
            (0, 4) => {
                self.fx_puff_piece_init(i);
                self.fx_puff_piece_run(i); // 0x947BA falls into 0x947C0
            }
            (2, _) => self.fx_puff_piece_run(i),
            (0, _) => {} // past the table 0x94692
            _ => self.fx_free(i), // 0x94844
        }
    }

    /// 0x94698: the block of the variant (`ay`, +$60, the first record).
    fn fx_puff_maker_init(&mut self, i: usize) {
        let t = &self.assets.enemy.t;
        let e = &mut self.effects.slots[i];
        let off = t.u16(PUFF_LISTS, e.b.var26 as u32); // move.w (a0, d0.w), d0 ; adda.w d0, a0
        let a = PUFF_LISTS.wrapping_add(off as i16 as i32 as u32);
        e.b.ay = t.i16(a, 0);
        e.owner60 = t.u16(a, 2);
        e.set_b5c(t.u8(a, 4));
        e.set_p58(a + 4);
        e.b.sub = e.b.sub.wrapping_add(2);
    }

    /// 0x946BA: count the delay down; at 0 make the piece of the record and the next ones whose delay is 0.
    fn fx_puff_maker_run(&mut self, i: usize) {
        let e = &mut self.effects.slots[i];
        let (d, borrow) = e.b5c().overflowing_sub(1); // subq.b #1, $5C ; bcs 0x946C4
        e.set_b5c(d);
        if !borrow {
            return;
        }
        loop {
            let a1 = self.effects.slots[i].p58();
            self.fx_puff_make_piece(i, a1); // 0x946C4 (a full pool skips it: bcs 0x94758)
            let t = &self.assets.enemy.t;
            let next = a1 + 0xC; // adda.l #$C, a1
            let delay = t.u8(next, 0);
            let e = &mut self.effects.slots[i];
            e.set_p58(next);
            e.set_b5c(delay);
            if delay == 0 {
                continue; // beq 0x946C4
            }
            if delay >= 0x80 {
                e.b.mode = 6; // move.w #$600, $4(a6)
                e.b.sub = 0;
            }
            return; // bpl rts
        }
    }

    /// 0x946C8..0x94752: one piece from the record at `a1`.
    fn fx_puff_make_piece(&mut self, i: usize, a1: u32) {
        let Some(j) = self.effects.alloc() else { return };
        let t = &self.assets.enemy.t;
        let m = self.effects.slots[i].clone();
        let c = &mut self.effects.slots[j];
        c.b.live = 1;
        c.b.k20 = FX_PUFF;
        c.b.mode = 0; // move.w #$4, $4(a0)
        c.b.sub = 4;
        set_word(&mut c.b.x, m.b.xi());
        set_word(&mut c.b.y, m.b.yi());
        set_word(&mut c.b.z, m.b.zi());
        c.b.ay = m.b.ay;
        c.owner60 = m.owner60;
        if m.b.var26 == 2 {
            c.b.palette_override = 0x8B;
        }
        c.b.var26 = t.u8(a1, 1) as u16;
        let x = c.b.xi().wrapping_add(t.i8(a1, 2) as i16); // ext.w ; add.w d0, $8(a0)
        let y = c.b.yi().wrapping_add(t.i8(a1, 3) as i16);
        let z = c.b.zi().wrapping_add(t.i8(a1, 4) as i16);
        set_word(&mut c.b.x, x);
        set_word(&mut c.b.y, y);
        set_word(&mut c.b.z, z);
        let vx = t.i16(a1, 6);
        c.b.vx = if m.b.facing != 0 { vx } else { vx.wrapping_neg() };
        c.b.vy = t.i16(a1, 8);
        c.b.vz = t.i16(a1, 0xA);
    }

    /// 0x94776: the piece starts: its animation, the life and the two blink counters.
    fn fx_puff_piece_init(&mut self, i: usize) {
        let fx = self.fx_data();
        let e = &mut self.effects.slots[i];
        e.b.anim_table = PUFF_ANIMS;
        let var = e.b.var26;
        fx_start(&mut e.b, fx, var);
        e.b.draw_flags = 0x30;
        e.b.airborne = 1;
        let r = self.rand() & 0x1F;
        let d7 = 47 - i as u16;
        let e = &mut self.effects.slots[i];
        e.b5e = r.wrapping_add(e.owner60) as u8; // add.w $60(a6), d0 ; move.b d0, $5E
        let d = ((d7 & 1) + 8) as u8;
        e.off_x = ((d as u16) << 8 | d as u16) as i16; // +$62 and +$63
        e.off_y &= 0x00FF; // clr.b $64
        e.b.mode = 2; // move.w #$200, $4(a6)
        e.b.sub = 0;
    }

    /// 0x947C0.
    fn fx_puff_piece_run(&mut self, i: usize) {
        let e = &mut self.effects.slots[i];
        let (life, borrow) = e.b5e.overflowing_sub(1); // subq.b #1, $5E ; bcs 0x94844
        e.b5e = life;
        if borrow {
            return self.fx_free(i);
        }
        let mut b = std::mem::take(&mut self.effects.slots[i].b);
        b.motion(); // jsr 0x189C
        self.terrain_effect_floor(&mut b); // jsr 0x12DAC
        fx_tick(&mut b, self.fx_data()); // jsr 0x15F4
        if b.airborne == 0 {
            b.vy = b.vy.wrapping_neg() >> 1; // neg.w ; asr.w #1
            let floor = b.floor;
            set_word(&mut b.y, floor); // move.w $54(a6), $c(a6)
            b.airborne = 1;
        }
        let e = &mut self.effects.slots[i];
        e.b = b;
        let h = self.effects.handle(i);
        if self.effects.slots[i].off_y >> 8 == 0 {
            // +$64 = 0: the growing pause.
            let e = &mut self.effects.slots[i];
            let (b62, b63) = ((e.off_x >> 8) as u8, e.off_x as u8);
            let (n62, borrow) = b62.overflowing_sub(1);
            e.off_x = ((n62 as u16) << 8 | b63 as u16) as i16;
            if !borrow {
                return self.draw_list_add_check(h); // jmp 0x49CA
            }
            if b63 != 1 {
                let n63 = b63 - 1;
                e.off_x = ((n63 as u16) << 8 | n63 as u16) as i16; // move.b $63(a6), $62(a6)
            } else {
                e.off_y = (e.off_y & 0x00FF) | 0x0100; // move.b #1, $64(a6)
            }
        } else if (self.w.frame.wrapping_add(47 - i as u16)) & 1 != 0 {
            self.draw_list_add_check(h); // 0x9482E: blink
        }
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

    #[test]
    fn a_maker_makes_its_pieces_by_the_delays_of_its_list_and_frees_itself() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        g.prop_c21_spawn(0x200, 0x10, 0x60, 0x0C, 1); // the variant 0xC has 14 pieces
        let maker = g.effects.slots.iter().position(|e| e.b.live != 0).unwrap();
        let mut seen = 0;
        for _ in 0..200 {
            g.pool_c_update();
            seen = seen.max(g.effects.slots.iter().filter(|e| e.b.live != 0).count());
            if g.effects.slots[maker].b.live == 0 {
                break;
            }
        }
        assert_eq!(g.effects.slots[maker].b.live, 0, "the maker ends after its last record");
        assert!(seen >= 3, "pieces were made: {seen}");
    }

    #[test]
    fn a_piece_lives_by_its_roll_and_the_maker_variant_2_colours_it() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        let mut expected = Game::new(a);
        g.prop_c21_spawn(0x200, 0x10, 0x60, 2, 0);
        g.pool_c_update(); // the maker makes its first pieces (mode 0, sub 4); they have lower slots, so they run in the next update
        let made: Vec<_> = g.effects.slots.iter().filter(|e| e.b.live != 0 && e.b.mode == 0 && e.b.sub == 4).collect();
        let made_n = made.len();
        assert!(made_n > 0);
        assert!(made.iter().all(|e| e.b.palette_override == 0x8B));
        assert_eq!(g.w.rng, expected.w.rng, "a maker draws no random number");
        g.pool_c_update();
        let n = g.effects.slots.iter().filter(|e| e.b.live != 0 && e.b.mode == 2).count();
        assert_eq!(n, made_n);
        for _ in 0..n {
            expected.rand(); // one roll for the life of each piece
        }
        assert_eq!(g.w.rng, expected.w.rng);
    }
}
