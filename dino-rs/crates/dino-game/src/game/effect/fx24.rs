//! Pool C class 24 (+$20 = 0x60), handler 0x959E6: the maker of the pieces of a broken wall panel (pool B class 0, 0x751FC: variant 0x0E, facing of the
//! attacker). It is the maker of class 21 (`effect/fx21.rs`) with shorter records: it reads the block of its variant in the table 0x95B80 (a word offset
//! table by variant, then `ay`, a word for +$60 and 8 byte records `delay, animation, dx, dy, vx (word), vy (word)`, ended by a byte of 0x80 or more) and
//! makes one **class 21 piece** per record (`+$20 = 0x54`, mode 0, sub 4, the maker's position plus (dx, dy), `ay` and +$60 of the maker, vx negated for
//! a maker that faces left, vy; no vz and no dz). A delay of 0 makes the next piece in the same frame; the maker frees itself after the last record. The
//! maker variant 2 colours the pieces (0x8B). The code after the maker (0x95AD6 .. 0x95B7A, a piece with the table 0x95E28) is never reached: the
//! pieces are class 21 objects and use the animation table 0x951AC.

use super::*;

/// 0x95B80: the lists by variant.
const BLAST_LISTS: u32 = 0x95B80;

impl Game {
    /// 0x959E6: mode 0, sub 0 (the block of the variant), sub 2 (the delays and the pieces); the modes 2, 4 and 6 free the slot (0x95B7A).
    pub(super) fn fx_blast(&mut self, i: usize) {
        let (mode, sub) = (self.effects.slots[i].b.mode, self.effects.slots[i].b.sub);
        match (mode, sub) {
            (0, 0) => {
                self.fx_blast_init(i);
                self.fx_blast_run(i); // 0x95A2A falls into 0x95A2E
            }
            (0, 2) => self.fx_blast_run(i),
            (0, _) => {} // past the table 0x959FA
            _ => self.fx_free(i),
        }
    }

    /// 0x95A0C.
    fn fx_blast_init(&mut self, i: usize) {
        let t = &self.assets.enemy.t;
        let e = &mut self.effects.slots[i];
        let off = t.u16(BLAST_LISTS, e.b.var26 as u32);
        let a = BLAST_LISTS.wrapping_add(off as i16 as i32 as u32);
        e.b.ay = t.i16(a, 0);
        e.owner60 = t.u16(a, 2);
        e.set_b5c(t.u8(a, 4));
        e.set_p58(a + 4);
        e.b.sub = e.b.sub.wrapping_add(2);
    }

    /// 0x95A2E.
    fn fx_blast_run(&mut self, i: usize) {
        let e = &mut self.effects.slots[i];
        let (d, borrow) = e.b5c().overflowing_sub(1);
        e.set_b5c(d);
        if !borrow {
            return;
        }
        loop {
            let a1 = self.effects.slots[i].p58();
            self.fx_blast_make_piece(i, a1);
            let next = a1 + 8; // addq.l #8, a1
            let delay = self.assets.enemy.t.u8(next, 0);
            let e = &mut self.effects.slots[i];
            e.set_p58(next);
            e.set_b5c(delay);
            if delay == 0 {
                continue;
            }
            if delay >= 0x80 {
                e.b.mode = 6;
                e.b.sub = 0;
            }
            return;
        }
    }

    /// 0x95A3C..0x95AB6: one class 21 piece from the record at `a1`.
    fn fx_blast_make_piece(&mut self, i: usize, a1: u32) {
        let Some(j) = self.effects.alloc() else { return };
        let t = &self.assets.enemy.t;
        let m = self.effects.slots[i].clone();
        let c = &mut self.effects.slots[j];
        c.b.live = 1;
        c.b.k20 = FX_PUFF;
        c.b.mode = 0;
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
        let x = c.b.xi().wrapping_add(t.i8(a1, 2) as i16);
        let y = c.b.yi().wrapping_add(t.i8(a1, 3) as i16);
        set_word(&mut c.b.x, x);
        set_word(&mut c.b.y, y);
        let vx = t.i16(a1, 4);
        c.b.vx = if m.b.facing != 0 { vx } else { vx.wrapping_neg() };
        c.b.vy = t.i16(a1, 6);
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
    fn the_wall_panel_maker_makes_five_class_21_pieces_for_variant_0xe() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        g.prop_c24_spawn(0x300, 0x20, 0x40, 0x0E, 1);
        let maker = g.effects.slots.iter().position(|e| e.b.live != 0).unwrap();
        let mut made = 0;
        let mut seen = std::collections::HashSet::new();
        for _ in 0..200 {
            g.pool_c_update();
            for (k, e) in g.effects.slots.iter().enumerate() {
                if k != maker && e.b.live != 0 && e.b.k20 == FX_PUFF && e.b.mode == 0 && e.b.sub == 4 && seen.insert((k, e.b.x, e.b.var26)) {
                    made += 1;
                    assert_eq!(e.b.ay, -56, "ay of the block");
                }
            }
            if g.effects.slots[maker].b.live == 0 {
                break;
            }
        }
        assert_eq!(g.effects.slots[maker].b.live, 0);
        assert_eq!(made, 5, "the block of the variant 0xE has five records");
    }
}
