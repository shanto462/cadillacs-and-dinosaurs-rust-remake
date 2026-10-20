//! Pool C class 29 (+$20 = 0x74), handler 0x965CE: a ground impact smoke ring. Made by 0x965A2 (`fx_spawn_smoke_ring`) with the caller's
//! position and a variant byte in the high byte of +$26 (the callers pass 0 or 2). The variant byte indexes the word table at 0x9665E; the block is dx, dy
//! (added once to the integer words), a vx long at +$58, a vy long at +$5C and a life byte at +$60. Animation 0 of table 0x9664E. Every
//! update, including the init's fall-through, loses one life, ticks the animation (the result is ignored), adds the two longs to X and Y
//! and draws with 0x49CA. A negative life frees it. No sound, no random number.

use super::*;

/// 0x9664E: the animation table (0x965E2). The handler always starts animation 0.
const SMOKE_ANIMS: u32 = 0x9664E;
/// 0x9665E: word offsets by the variant byte, then the blocks (0x965F2).
const SMOKE_PARAMS: u32 = 0x9665E;

impl Effect {
    /// The long +$5C (vy of the smoke ring): +$5C / +$5E / +$5F as one big-endian long.
    pub(super) fn p5c(&self) -> u32 {
        ((self.w5c as u32) << 16) | ((self.b5e as u32) << 8) | self.b5f as u32
    }

    pub(super) fn set_p5c(&mut self, a: u32) {
        self.w5c = (a >> 16) as u16;
        self.b5e = (a >> 8) as u8;
        self.b5f = a as u8;
    }

    /// The byte +$60 (the high byte of the word). `subq.b #1, $60` / `bmi` read this byte.
    fn b60(&self) -> u8 {
        (self.owner60 >> 8) as u8
    }

    fn set_b60(&mut self, v: u8) {
        self.owner60 = (self.owner60 & 0x00FF) | ((v as u16) << 8);
    }
}

impl Game {
    /// 0x965CE: mode 0 (init, falls into the run) and 2; 4 and 6 free the slot (0x96648).
    pub(super) fn fx_smoke_ring(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => self.fx_smoke_ring_init(i),
            2 => {}
            _ => return self.fx_free(i),
        }
        // 0x96624, also the fall-through of the init.
        let n = self.effects.slots[i].b60().wrapping_sub(1);
        self.effects.slots[i].set_b60(n);
        if n & 0x80 != 0 {
            return self.fx_free(i); // bmi 0x96648
        }
        let fx = self.fx_data();
        fx_tick(&mut self.effects.slots[i].b, fx); // jsr 0x15F4, the carry is ignored
        let e = &mut self.effects.slots[i];
        let vx = e.p58() as i32;
        let vy = e.p5c() as i32;
        e.b.x = e.b.x.wrapping_add(vx); // add.l $58, $8
        e.b.y = e.b.y.wrapping_add(vy);
        self.draw_list_add_check(self.effects.handle(i)); // jmp 0x49CA
    }

    /// 0x965E2: the block of the variant byte, animation 0, mode + 2.
    fn fx_smoke_ring_init(&mut self, i: usize) {
        let var = (self.effects.slots[i].b.var26 >> 8) as u8; // move.b $26(a6), d0: the high byte of the word, a byte index
        let t = &self.assets.enemy.t;
        let off = t.u16(SMOKE_PARAMS, var as u32);
        let a = SMOKE_PARAMS.wrapping_add(off as i16 as i32 as u32);
        let (dx, dy) = (t.i16(a, 0), t.i16(a, 2));
        let (vx, vy, life) = (t.u32(a, 4), t.u32(a, 8), t.u8(a, 12));
        let fx = self.fx_data();
        let e = &mut self.effects.slots[i];
        e.b.anim_table = SMOKE_ANIMS;
        e.b.draw_flags = 0x10;
        let (x, y) = (e.b.xi().wrapping_add(dx), e.b.yi().wrapping_add(dy));
        set_word(&mut e.b.x, x); // add.w, the fraction stays
        set_word(&mut e.b.y, y);
        e.set_p58(vx);
        e.set_p5c(vy);
        e.set_b60(life); // move.b (a0), $60: the low byte of the word stays
        fx_start(&mut e.b, fx, 0);
        e.b.mode = e.b.mode.wrapping_add(2);
    }

    /// 0x965A2: a smoke ring at `owner`'s position words. `var` goes to the high byte of +$26 (`move.b d0,$26(a0)`, an even
    /// address); the low byte is not written.
    /// `false` when pool C is full (carry). The callers are jumps, so the spawn's rts leaves the caller's update.
    pub fn fx_spawn_smoke_ring(&mut self, owner: Handle, var: u8) -> bool {
        let Some(j) = self.fx_alloc() else { return false };
        let (x, y, z) = {
            let o = self.body(owner);
            (o.xi(), o.yi(), o.zi())
        };
        let e = &mut self.effects.slots[j];
        e.b.live = 1;
        e.b.k20 = FX_SMOKE_RING;
        set_word(&mut e.b.x, x);
        set_word(&mut e.b.y, y);
        set_word(&mut e.b.z, z);
        e.b.var26 = (e.b.var26 & 0x00FF) | ((var as u16) << 8);
        true
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

    fn place(g: &mut Game) {
        g.players[0].b.set_xi(0x200);
        g.players[0].b.set_yi(0x10);
        g.players[0].b.set_zi(0x40);
    }

    #[test]
    fn the_smoke_ring_moves_for_25_updates_and_the_two_variants_leave_opposite_ways() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        let h = Game::player_handle(0);
        place(&mut g);
        let rng = g.w.rng;
        assert!(g.fx_spawn_smoke_ring(h, 0));
        assert_eq!(g.w.rng, rng, "the spawn draws no random number");
        let i = g.effects.slots.iter().position(|e| e.b.live != 0).unwrap();
        assert_eq!(g.effects.slots[i].b.xi(), 0x200, "the shift is applied on the first update");
        g.pool_c_update();
        assert_eq!(g.effects.slots[i].b.mode, 2);
        assert_eq!(g.effects.slots[i].b.visible, 1, "0x49CA sets visible when the ring is on screen");
        assert!(g.effects.slots[i].b.xi() < 0x200, "variant 0 steps left");
        assert!(g.effects.slots[i].b.yi() > 0x10, "both variants step down");
        let mut n = 1;
        while g.effects.slots[i].b.live != 0 && n < 40 {
            g.pool_c_update();
            n += 1;
        }
        assert_eq!(n, 26, "25 drawn updates, then the life byte goes negative and frees");

        let mut g = Game::new(a);
        place(&mut g);
        g.fx_spawn_smoke_ring(h, 2);
        g.pool_c_update();
        let e = g.effects.slots.iter().find(|e| e.b.live != 0).unwrap();
        assert!(e.b.xi() > 0x200, "variant 2 steps right");
        assert!(e.b.yi() > 0x10);
    }
}
