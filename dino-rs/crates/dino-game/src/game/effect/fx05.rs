//! Pool C class 5 (+$20 = 0x14), handler 0x92B7A: bubbles and a wake. Two spawners, both at the caller's position with Y + 0x60:
//! 0x6192 (`fx_spawn_bubbles`, the leader's pose, variant from one `rand()` and the caller's class word, +$5C = 0 so the picture waits
//! until the caller's frame byte +$2C is set) and 0x61F6 (`fx_spawn_bubbles_now`, variant from the character id, +$5C = 1 so it starts
//! at once). The picture does not move. It frees when the animation ends or when G.$518 (the car) is set.

use super::*;

/// 0x92BF0: the animation table (0x92B8E).
const BUBBLE_ANIMS: u32 = 0x92BF0;
/// 0x61E6: one variant byte per (rand() & 1) + the owner's class word +$20 (0x61D8).
const BUBBLE_BY_CLASS: u32 = 0x61E6;
/// 0x6244: one variant byte per character id (0x6234).
const BUBBLE_BY_ID: u32 = 0x6244;

impl Game {
    /// 0x92B7A: mode 0 (init, falls into the run) and 2; 4 and 6 free the slot (0x92BEA).
    pub(super) fn fx_bubbles(&mut self, i: usize) {
        let fx = self.fx_data();
        match self.effects.slots[i].b.mode {
            0 => {
                let e = &mut self.effects.slots[i];
                e.b.anim_table = BUBBLE_ANIMS;
                let id = e.b.var26;
                fx_start(&mut e.b, fx, id); // 0x15E4
                e.b.facing = 0;
                e.b.visible = 1;
                e.b.draw_flags = 0x10;
                e.b.mode = e.b.mode.wrapping_add(2);
            }
            2 => {}
            _ => return self.fx_free(i),
        }
        // 0x92BB4
        if self.spawn.g.car_mode != 0 {
            return self.fx_free(i);
        }
        if self.effects.slots[i].b.sub == 0 {
            // +$5C set (the 0x61F6 spawn) starts at once; otherwise wait until the owner's +$2C is set.
            if self.effects.slots[i].w5c == 0 {
                let owner = self.effects.slots[i].owner58;
                if self.body(owner).param_a == 0 {
                    return;
                }
            }
            self.effects.slots[i].b.sub = self.effects.slots[i].b.sub.wrapping_add(2);
        }
        if fx_tick(&mut self.effects.slots[i].b, fx) {
            self.fx_free(i); // bcs 0x92BEA
        } else {
            self.draw_list_add_if_visible(self.effects.handle(i)); // jmp 0x49C0
        }
    }

    /// 0x6192: bubbles for `owner` (a6). The variant byte is table 0x61E6 indexed by `(rand() & 1) + owner.k20`. +$5C stays 0.
    /// `false` when pool C is full (carry).
    pub fn fx_spawn_bubbles(&mut self, owner: Handle) -> bool {
        let Some(j) = self.fx_alloc() else { return false };
        let (who, x, y, z, k20) = {
            let o = self.body(owner);
            (o.who, o.xi(), o.yi(), o.zi(), o.k20)
        };
        {
            let e = &mut self.effects.slots[j];
            e.b.live = 1;
            e.b.k20 = FX_BUBBLES;
            e.b.who = who;
            set_word(&mut e.b.x, x);
            set_word(&mut e.b.y, y.wrapping_add(0x60)); // addi.w #$60, $C
            set_word(&mut e.b.z, z);
            e.owner58 = owner; // move.w a6, $58 (the word, not the long)
        }
        let bit = self.rand() & 1; // jsr 0x119C ; andi.w #1
        let var = self.assets.enemy.t.u8(BUBBLE_BY_CLASS, k20.wrapping_add(bit) as u32);
        let e = &mut self.effects.slots[j];
        e.b.var26 = var as u16; // move.b into d0, then move.w d0, $26 (the class word fits in a byte)
        e.w5c = 0;
        true
    }

    /// 0x61F6: the same placement as 0x6192, but the variant comes from the character id (table 0x6244) and +$5C = 1, so the
    /// handler does not wait for the owner's +$2C. No random number. The only caller is the tag team start (0x686E, `bra` at 0x68FC: `player_tag.rs`).
    pub fn fx_spawn_bubbles_now(&mut self, owner: Handle) -> bool {
        let Some(j) = self.fx_alloc() else { return false };
        let (who, id, x, y, z) = {
            let o = self.body(owner);
            (o.who, o.id, o.xi(), o.yi(), o.zi())
        };
        let var = self.assets.enemy.t.u8(BUBBLE_BY_ID, id as u32);
        let e = &mut self.effects.slots[j];
        e.b.live = 1;
        e.b.k20 = FX_BUBBLES;
        e.b.who = who;
        set_word(&mut e.b.x, x);
        set_word(&mut e.b.y, y.wrapping_add(0x60));
        set_word(&mut e.b.z, z);
        e.owner58 = owner;
        e.b.var26 = var as u16;
        e.w5c = 1;
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

    #[test]
    fn bubbles_wait_for_the_owner_then_play_once_and_the_car_frees_them() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        let h = Game::player_handle(0);
        g.players[0].b.who = 1;
        g.players[0].b.k20 = 0;
        g.players[0].b.set_xi(0x180);
        g.players[0].b.set_yi(0x20);
        g.players[0].b.set_zi(0x30);
        g.w.rng = 0x01C3;
        assert!(g.fx_spawn_bubbles(h));
        let i = g.effects.slots.iter().position(|e| e.b.live != 0).unwrap();
        let e = &g.effects.slots[i];
        let got = e.b.var26;
        assert_eq!((e.b.xi(), e.b.yi(), e.b.zi(), e.b.who, e.owner58, e.w5c), (0x180, 0x80, 0x30, 1, h, 0));
        g.w.rng = 0x01C3;
        let bit = g.rand() & 1; // the same one call 0x6192 made
        assert_eq!(got, a.enemy.t.u8(BUBBLE_BY_CLASS, bit as u32) as u16);
        g.pool_c_update(); // mode 0 loads the frame, then waits
        assert_eq!(g.effects.slots[i].b.sub, 0, "the owner's +$2C is still 0");
        let timer = g.effects.slots[i].b.timer;
        assert!(timer > 0, "the frame is loaded while the object waits");
        for _ in 0..4 {
            g.pool_c_update();
            assert_eq!(g.effects.slots[i].b.sub, 0);
            assert_eq!(g.effects.slots[i].b.timer, timer, "the animation does not tick while it waits");
        }
        g.players[0].b.param_a = 1;
        let mut n = 0;
        while g.effects.slots[i].b.live != 0 && n < 400 {
            g.pool_c_update();
            n += 1;
        }
        assert_eq!(n, timer as usize, "one play of the frame, then free");

        g.w.rng = 0x01C3;
        g.fx_spawn_bubbles(h);
        g.spawn.g.car_mode = 1;
        g.pool_c_update();
        assert!(g.effects.slots.iter().all(|e| e.b.live == 0), "G.$518 set: freed on the first update");
    }

    #[test]
    fn bubbles_now_uses_the_character_and_no_random_number() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        let h = Game::player_handle(0);
        g.players[0].b.id = 2;
        g.players[0].b.set_yi(0);
        let rng = g.w.rng;
        assert!(g.fx_spawn_bubbles_now(h));
        assert_eq!(g.w.rng, rng, "0x61F6 draws no random number");
        let i = g.effects.slots.iter().position(|e| e.b.live != 0).unwrap();
        assert_eq!(g.effects.slots[i].b.var26, a.enemy.t.u8(BUBBLE_BY_ID, 2) as u16);
        assert_eq!(g.effects.slots[i].w5c, 1);
        g.pool_c_update();
        assert_eq!(g.effects.slots[i].b.sub, 2, "starts without the owner's +$2C");
        assert_eq!(g.effects.slots[i].b.yi(), 0x60);
    }

    /// Probe `jump 2 1 0x7C0` (Episode 2 area 1, hold right): MAME logs no pool C class 5. The wading loop spawns ripples (0x60E2), not these bubbles.
    #[test]
    fn a_wade_in_the_episode_2_swamp_makes_no_bubbles() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        g.start_stage_at(1, 1, 0x7C0, &[0]);
        let mut bubbles = 0usize;
        let mut wading = false;
        for _ in 0..400 {
            g.w.input = [1, 0, 0]; // right
            g.tick();
            if g.players[0].b.wading != 0 {
                wading = true;
            }
            bubbles += g.effects.slots.iter().filter(|e| e.b.live != 0 && e.b.k20 == FX_BUBBLES).count();
        }
        assert!(wading, "the hero walks into the water");
        assert_eq!(bubbles, 0, "class 5 is not the wading loop");
    }
}
