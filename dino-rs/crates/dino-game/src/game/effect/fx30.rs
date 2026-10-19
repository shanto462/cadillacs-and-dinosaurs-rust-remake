//! Pool C class 30 (+$20 = 0x78), handler 0x966B4: the "Z" mark of a sleeping dinosaur (Shivat, and the Mack sleeper). Made by 0x9667E
//! (`enemy_monster_spawn_z`) at the dinosaur's position words, with the variant byte in +$26 (Shivat 2, Mack 0), the dinosaur's handle as a sign extended
//! long in +$68 and its state long +$04 saved in +$6C. The variant indexes the word table at 0x96794; its block gives dx and dy (added once to the integer
//! words), four longs (+$58 the X speed, +$5C the Y speed, +$60 the X acceleration, +$64 the X speed where the acceleration turns) and a life byte (+$70).
//! Animation 0 of table 0x96770 (3 frames), no shadow (+$25 = 0x10).
//!
//! Every update (the init falls into the run): the life byte loses one and a negative life frees it; the object frees itself when the dinosaur's state
//! long is no longer the saved one (he woke up, or was freed); the animation ticks; when the speed +$58 has reached +$64 the acceleration is negated and
//! +$64 becomes 0x8000 - +$64 (the mark sways); +$58 += +$60, X += +$58, Y += +$5C; the draw check 0x49CA. No sound, no random number.

use super::*;

/// 0x96770: the animation table (the init 0x966C8 starts animation 0).
const Z_ANIMS: u32 = 0x96770;
/// 0x96794: word offsets by the variant byte, then the blocks.
const Z_PARAMS: u32 = 0x96794;

impl Effect {
    /// The long +$60 (the X acceleration): +$60 / +$62 as one big-endian long.
    fn p60(&self) -> u32 {
        ((self.owner60 as u32) << 16) | self.off_x as u16 as u32
    }

    fn set_p60(&mut self, a: u32) {
        self.owner60 = (a >> 16) as u16;
        self.off_x = a as u16 as i16;
    }

    /// The long +$64: +$64 / +$66 as one big-endian long.
    fn p64(&self) -> u32 {
        ((self.off_y as u16 as u32) << 16) | self.w66 as u32
    }

    fn set_p64(&mut self, a: u32) {
        self.off_y = (a >> 16) as u16 as i16;
        self.w66 = a as u16;
    }

    /// The life byte +$70 (the high byte of the word `b.partner`).
    fn z_life(&self) -> u8 {
        (self.b.partner >> 8) as u8
    }

    fn set_z_life(&mut self, v: u8) {
        self.b.partner = (self.b.partner & 0x00FF) | ((v as u16) << 8);
    }
}

impl Game {
    /// 0x966B4: mode 0 (init, falls into the run) and 2; 4 and 6 free the slot (0x9676A).
    pub(super) fn fx_z_mark(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => self.fx_z_mark_init(i),
            2 => {}
            _ => return self.fx_free(i),
        }
        // 0x96712
        let n = self.effects.slots[i].z_life().wrapping_sub(1);
        self.effects.slots[i].set_z_life(n);
        if n & 0x80 != 0 {
            return self.fx_free(i); // bmi 0x9676A
        }
        let b = &self.effects.slots[i].b;
        let owner = b.hp_bar; // the low word of the sign extended long +$68
        let saved = ((b.hp as u32) << 16) | b.hp_prev as u32; // the long +$6C
        if self.body(owner).state() != saved {
            return self.fx_free(i); // bne 0x9676A: he is not asleep any more
        }
        let fx = self.fx_data();
        fx_tick(&mut self.effects.slots[i].b, fx); // jsr 0x15F4, the carry is ignored
        let e = &mut self.effects.slots[i];
        if e.p64() == e.p58() {
            let acc = e.p60().wrapping_neg();
            e.set_p60(acc);
            let t = e.p64().wrapping_sub(0x8000).wrapping_neg();
            e.set_p64(t);
        }
        let vx = e.p58().wrapping_add(e.p60());
        e.set_p58(vx);
        e.b.x = e.b.x.wrapping_add(vx as i32);
        e.b.y = e.b.y.wrapping_add(e.p5c() as i32);
        self.draw_list_add_check(self.effects.handle(i)); // jmp 0x49CA
    }

    /// 0x966C8: the block of the variant byte, animation 0, mode + 2.
    fn fx_z_mark_init(&mut self, i: usize) {
        let var = (self.effects.slots[i].b.var26 >> 8) as u8; // move.b $26, d0: the byte at +$26, the high byte of the word
        let t = &self.assets.enemy.t;
        let off = t.u16(Z_PARAMS, var as u32);
        let a = Z_PARAMS.wrapping_add(off as i16 as i32 as u32);
        let (dx, dy) = (t.i16(a, 0), t.i16(a, 2));
        let (v58, v5c, v60, v64) = (t.u32(a, 4), t.u32(a, 8), t.u32(a, 12), t.u32(a, 16));
        let life = t.u8(a, 20);
        let fx = self.fx_data();
        let e = &mut self.effects.slots[i];
        e.b.anim_table = Z_ANIMS;
        e.b.draw_flags = 0x10;
        let (x, y) = (e.b.xi().wrapping_add(dx), e.b.yi().wrapping_add(dy));
        set_word(&mut e.b.x, x); // add.w, the fraction stays
        set_word(&mut e.b.y, y);
        e.set_p58(v58);
        e.set_p5c(v5c);
        e.set_p60(v60);
        e.set_p64(v64);
        e.set_z_life(life); // move.b (a0), $70: the low byte of the word stays
        fx_start(&mut e.b, fx, 0);
        e.b.mode = e.b.mode.wrapping_add(2);
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

    /// The Z mark lives 120 updates (life byte 0x78, the init counts as one) while the dinosaur keeps his state, and frees itself at once when it changes.
    #[test]
    fn the_z_mark_lives_while_the_dinosaur_sleeps() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        let i = g.enemies.alloc().unwrap();
        let h = g.enemies.handle(i);
        g.enemies.slots[i].b.live = 1;
        g.enemies.slots[i].b.set_state(0x0200_0200);
        g.enemies.slots[i].b.set_xi(0x300);
        let e = g.enemies.slots[i].clone();
        g.enemy_monster_spawn_z(&e, h, 2);
        let j = g.effects.slots.iter().position(|e| e.b.live != 0).unwrap();
        let mut n = 0;
        while g.effects.slots[j].b.live != 0 && n < 200 {
            g.pool_c_update();
            n += 1;
        }
        assert_eq!(n, 121, "120 drawn updates, then the life byte goes negative");

        let mut g = Game::new(a);
        let i = g.enemies.alloc().unwrap();
        let h = g.enemies.handle(i);
        g.enemies.slots[i].b.live = 1;
        g.enemies.slots[i].b.set_state(0x0200_0200);
        let e = g.enemies.slots[i].clone();
        g.enemy_monster_spawn_z(&e, h, 2);
        let j = g.effects.slots.iter().position(|e| e.b.live != 0).unwrap();
        g.pool_c_update();
        g.pool_c_update();
        assert_ne!(g.effects.slots[j].b.live, 0);
        g.enemies.slots[i].b.set_state(0x0200_0400); // he wakes
        g.pool_c_update();
        assert_eq!(g.effects.slots[j].b.live, 0);
    }
}
