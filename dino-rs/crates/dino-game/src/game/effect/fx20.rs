//! Pool C class 20 (+$20 = 0x50), handler 0x9454E: sweat and shock beads over an owner. Made by 0x94522 (`fx_spawn_sweat`, the only caller is
//! the Rock Hopper's init at 0x4DDFE) with the owner long in +$58 and the owner's position. Each update shows the frame of table 0x945CC
//! indexed by the owner's frame byte +$2D (loaded with 0x160A, not ticked). When the loaded frame's own +$2D is set, the next update takes
//! the end path (0x945B8): mode + 2 and a class 21 puff, variant 0xA, facing 0. Mode 4 is that same end path, so the puff is made twice and
//! the update after that frees the beads (mode 6, 0x945C6). No movement, no sound, no random number.

use super::*;

/// 0x945CC: the frame table (0x9456E stores it, 0x945A4 indexes it by the owner's +$2D).
const SWEAT_ANIMS: u32 = 0x945CC;

impl Game {
    /// 0x9454E. Mode 0 falls into the run. Mode 4 is the end path again (the table word at 0x9455C + 4 is 0x5C, the same 0x945B8).
    pub(super) fn fx_sweat(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => {
                self.fx_sweat_init(i);
                self.fx_sweat_run(i);
            }
            2 => self.fx_sweat_run(i),
            4 => self.fx_sweat_end(i),
            _ => self.fx_free(i),
        }
    }

    /// 0x94564: mode + 2, visible, the table, clear the byte +$5C, load the first frame, then fall into the run.
    fn fx_sweat_init(&mut self, i: usize) {
        let fx = self.fx_data();
        let e = &mut self.effects.slots[i];
        e.b.mode = e.b.mode.wrapping_add(2);
        e.b.visible = 1;
        e.b.anim_table = SWEAT_ANIMS;
        e.set_b5c(0); // clr.b $5C: the high byte only
        let first = fx.anim(SWEAT_ANIMS, 0);
        fx_load(&mut e.b, fx, first); // 0x160A of the frame the first table word points at
    }

    /// 0x94588: end when this object's +$2D is set; otherwise follow the owner's +$2D and draw (0x49C0).
    fn fx_sweat_run(&mut self, i: usize) {
        if self.effects.slots[i].b.param_b != 0 {
            return self.fx_sweat_end(i);
        }
        let owner = self.effects.slots[i].owner_class; // the low word of the long +$58 (movea.l $58)
        let idx = self.body(owner).param_b;
        if idx != self.effects.slots[i].b5c() {
            let fx = self.fx_data();
            let e = &mut self.effects.slots[i];
            e.set_b5c(idx); // move.b d0, $5C
            let frame = fx.anim(SWEAT_ANIMS, idx as u16);
            fx_load(&mut e.b, fx, frame);
        }
        self.draw_list_add_if_visible(self.effects.handle(i));
    }

    /// 0x945B8: mode + 2, then jmp 0x94640 with d0 = 0xA and d1 = 0 (a class 21 puff at this object's position). Does not return to the draw.
    fn fx_sweat_end(&mut self, i: usize) {
        let e = &mut self.effects.slots[i];
        e.b.mode = e.b.mode.wrapping_add(2);
        let b = e.b.clone();
        self.fx_spawn_class21(&b, 0xA, 0);
    }

    /// 0x94522: the beads over `owner` (a6). The owner long is `move.l a6, $58` (high word 0xFFFF, low word the handle). Nothing when pool C is full.
    pub fn fx_spawn_sweat(&mut self, owner: Handle) -> bool {
        let Some(j) = self.fx_alloc() else { return false };
        let (x, y, z) = {
            let o = self.body(owner);
            (o.xi(), o.yi(), o.zi())
        };
        let e = &mut self.effects.slots[j];
        e.b.live = 1;
        e.b.k20 = FX_SWEAT;
        e.owner58 = 0xFFFF;
        e.owner_class = owner;
        set_word(&mut e.b.x, x);
        set_word(&mut e.b.y, y);
        set_word(&mut e.b.z, z);
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
    fn sweat_follows_the_owner_frame_then_makes_two_puffs() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        let h = Game::player_handle(0);
        g.players[0].b.set_xi(0x100);
        g.players[0].b.set_yi(0x40);
        g.players[0].b.set_zi(0x20);
        g.players[0].b.param_b = 0;
        assert!(g.fx_spawn_sweat(h));
        let i = g.effects.slots.iter().position(|e| e.b.k20 == FX_SWEAT).unwrap();
        assert_eq!((g.effects.slots[i].owner58, g.effects.slots[i].owner_class), (0xFFFF, h));
        g.pool_c_update();
        assert_eq!(g.effects.slots[i].b.mode, 2);
        assert_eq!((g.effects.slots[i].b.xi(), g.effects.slots[i].b.yi()), (0x100, 0x40), "the beads do not move");
        let mut saw = false;
        for k in 0..9u8 {
            g.players[0].b.param_b = k;
            g.pool_c_update();
            if g.effects.slots[i].b.param_b != 0 {
                saw = true;
                assert_eq!(g.effects.slots[i].b5c(), k, "the stored index is the owner's +$2D");
                break;
            }
            assert_eq!(g.effects.slots[i].b.mode, 2);
        }
        assert!(saw, "one frame of the table has +$2D set");
        // Update only the beads. A pool update would also run the class 21 maker, which makes its own pieces.
        g.fx_sweat(i);
        assert_eq!(g.effects.slots[i].b.mode, 4, "the end path bumps the mode and does not free yet");
        let puffs: Vec<_> = g.effects.slots.iter().filter(|e| e.b.live != 0 && e.b.k20 == FX_PUFF).map(|e| (e.b.var26, e.b.facing, e.b.xi())).collect();
        assert_eq!(puffs, vec![(0xA, 0, 0x100)]);
        g.fx_sweat(i);
        assert_eq!(g.effects.slots[i].b.mode, 6, "mode 4 is the end path again");
        assert_eq!(g.effects.slots.iter().filter(|e| e.b.live != 0 && e.b.k20 == FX_PUFF).count(), 2);
        g.fx_sweat(i);
        assert_eq!(g.effects.slots[i].b.live, 0);
    }
}
