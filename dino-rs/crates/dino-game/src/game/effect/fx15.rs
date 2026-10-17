//! Pool C class 15 (+$20 = 0x3C), handler 0x93E54: a piece of a body that burned to death (the shower of 16 pieces, 15 in the Japanese region, that the charred
//! death of a thug makes: `enemy_burn`, 0x2AC42 -> 0x93D48 `fx_spawn_burn_pieces`). Each piece has a picture by its variant (+$26, 0 to 5: an animation of the
//! table 0x93F42 with the sprite frame loader 0x120E, like an object), a life of 30 to 93 updates (+$58, from the random number), flies up with the speeds of its
//! record (table 0x93DB4: `dy, vy, vz, vx, variant`; `ay` is -0x60; vx is negated for an owner that faces left) and bounces on the floor: sub 0 is the first
//! flight (a half bounce, then sub 2), sub 2 the rest (a full bounce, drawn on every second frame). The floor snap 0x12DAC runs in every second update
//! (`G.$0 + d7` odd; +$51 is set to 1 before it in every update). In Episode 8 (stage 7) the draw flags are 0x30.

use super::*;

/// 0x93F42: the animation table of the pieces.
const BURN_ANIMS: u32 = 0x93F42;
/// 0x93DB4: 16 records of 5 words.
const BURN_PIECES: u32 = 0x93DB4;

impl Game {
    /// 0x93D48: the pieces of the burned body `o` (a6): 16 of them (G.$32, the region word, is not 0) or 15, from the table 0x93DB4. Stops when pool C is full.
    pub fn fx_spawn_burn_pieces(&mut self, o: &Body) {
        let count = if self.flow.region.0 != 0 { 16 } else { 15 };
        for k in 0..count {
            let Some(j) = self.effects.alloc() else { return };
            let t = &self.assets.enemy.t;
            let a = BURN_PIECES + 10 * k;
            let e = &mut self.effects.slots[j];
            e.b.live = 1;
            e.b.k20 = FX_BURN_PIECE;
            set_word(&mut e.b.x, o.xi());
            set_word(&mut e.b.y, o.yi());
            set_word(&mut e.b.z, o.zi());
            let y = e.b.yi().wrapping_add(t.i16(a, 0));
            set_word(&mut e.b.y, y);
            e.b.vy = t.i16(a, 2);
            e.b.ay = -0x60;
            e.b.vz = t.i16(a, 4);
            let vx = t.i16(a, 6);
            e.b.vx = if o.facing != 0 { vx.wrapping_neg() } else { vx };
            e.b.var26 = t.u16(a, 8);
        }
    }

    /// 0x93E54: mode 0 (init, falls into the run) and 2; 4 and 6 free the slot (0x93F3C).
    pub(super) fn fx_burn_piece(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => {
                let life = self.fx_burn_init(i);
                self.effects.slots[i].owner58 = life;
            }
            2 => {}
            _ => return self.fx_free(i),
        }
        self.fx_burn_run(i);
    }

    /// 0x93E68: the picture, the draw flags, the life. Returns the life (the random number draw comes after the picture).
    fn fx_burn_init(&mut self, i: usize) -> u16 {
        let stage7 = self.w.stage == 7;
        let s = &self.assets.sprites;
        let e = &mut self.effects.slots[i];
        e.b.anim_table = BURN_ANIMS;
        let var = e.b.var26;
        e.b.start_anim(s, var); // jsr 0x120E
        e.b.draw_flags = 0x10;
        if stage7 {
            e.b.draw_flags += 0x20;
        }
        e.b.airborne = 1;
        let life = (self.rand() & 0x3F) + 0x1E;
        let e = &mut self.effects.slots[i];
        e.b.mode = e.b.mode.wrapping_add(2);
        life
    }

    /// 0x93EAE.
    fn fx_burn_run(&mut self, i: usize) {
        let e = &mut self.effects.slots[i];
        let (life, borrow) = e.owner58.overflowing_sub(1); // subq.w #1, $58 ; bcs 0x93F3C
        e.owner58 = life;
        if borrow {
            return self.fx_free(i);
        }
        e.b.airborne = 1;
        let odd = (self.w.frame.wrapping_add(47 - i as u16)) & 1 != 0;
        if odd {
            let mut b = std::mem::take(&mut self.effects.slots[i].b);
            self.terrain_effect_floor(&mut b); // jsr 0x12DAC
            self.effects.slots[i].b = b;
        }
        let e = &mut self.effects.slots[i];
        if e.b.sub == 0 {
            if e.b.airborne == 0 {
                e.b.vy = e.b.vy.wrapping_neg() >> 1; // neg.w ; asr.w #1
                e.b.airborne = 1;
                e.b.sub = e.b.sub.wrapping_add(2);
            }
            let s = &self.assets.sprites;
            e.b.tick_anim(s); // jsr 0x121E
            e.b.motion(); // jsr 0x189C
            self.draw_list_add_check(self.effects.handle(i)); // jmp 0x49CA
        } else {
            if e.b.airborne == 0 {
                e.b.vy = e.b.vy.wrapping_neg();
            }
            let s = &self.assets.sprites;
            e.b.tick_anim(s);
            e.b.motion();
            if odd {
                self.draw_list_add_check(self.effects.handle(i));
            }
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
    fn a_burned_body_makes_sixteen_pieces_that_all_end_after_their_lives() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        g.flow.region.0 = 2;
        let mut o = Body::default();
        o.set_xi(0x300);
        o.set_yi(0);
        o.set_zi(0x40);
        o.facing = 1;
        g.fx_spawn_burn_pieces(&o);
        let live: Vec<_> = g.effects.slots.iter().filter(|e| e.b.live != 0).collect();
        assert_eq!(live.len(), 16);
        let first = live.iter().find(|e| e.b.var26 == 5 && e.b.yi() == 64).unwrap();
        assert_eq!((first.b.vy, first.b.vz, first.b.vx, first.b.ay), (0x880, -21, -0x100, -0x60), "the first record, vx negated for an owner facing left");
        g.flow.region.0 = 0;
        let mut h = Game::new(a);
        h.flow.region.0 = 0;
        h.fx_spawn_burn_pieces(&o);
        assert_eq!(h.effects.slots.iter().filter(|e| e.b.live != 0).count(), 15, "15 pieces in the Japanese region");
        let mut expected = Game::new(a);
        for n in 0..100 {
            g.w.frame = g.w.frame.wrapping_add(1);
            g.pool_c_update();
            if n == 0 {
                for _ in 0..16 {
                    expected.rand(); // one roll for the life of each piece
                }
                assert_eq!(g.w.rng, expected.w.rng);
            }
        }
        assert_eq!(g.effects.slots.iter().filter(|e| e.b.live != 0).count(), 0, "a life is 30 to 93 updates");
    }

    #[test]
    fn a_burning_enemy_gets_the_flames_of_its_name_id() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        let h = g.enemies.handle(2);
        let mut e = crate::game::enemy::Enemy::default();
        e.b.live = 1;
        e.m.sw(crate::game::enemy::off::W84, 0); // name id 0: the list at 0x2CA96, two flames
        g.enemy_burn_flames(h, &e);
        let flames: Vec<_> = g.effects.slots.iter().filter(|e| e.b.live != 0).collect();
        assert_eq!(flames.len(), 2);
        let mut got: Vec<_> = flames.iter().map(|f| (f.owner60, f.off_x, f.off_y, f.b.var26, f.w5c)).collect();
        got.sort_by_key(|x| x.4);
        assert_eq!(got, vec![(h, -30, 8, 0, 0), (h, 16, 10, 0, 1)]);
    }
}
