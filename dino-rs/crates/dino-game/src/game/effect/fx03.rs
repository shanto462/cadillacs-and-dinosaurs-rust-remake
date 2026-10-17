//! Pool C class 3 (+$20 = 0xC), handler 0x929B8: the shockwave picture of a thug (class 10: Hammer T., Blk Elmer, Wrench T.) who holds a hero (0x3C9FC,
//! `c10_3c9fc`: +$58 the thug, +$5A his class word). It shows one frame (the list 0x92A30, or 0x92A3C for a variant other than 0) at the thug's position
//! and facing for as long as the thug is alive, still the same class and in the frame class 0xC (+$2E: the hold); it never changes its picture.

use super::*;

/// 0x92A30 (variant 0) and 0x92A3C (the others).
const SHOCK_FRAMES: [u32; 2] = [0x92A30, 0x92A3C];

impl Game {
    /// 0x929B8: mode 0 (init, falls into the run) and 2; 4 and 6 free the slot (0x92A2A).
    pub(super) fn fx_shockwave(&mut self, i: usize) {
        let fx = self.fx_data();
        match self.effects.slots[i].b.mode {
            0 => {
                let e = &mut self.effects.slots[i];
                let list = SHOCK_FRAMES[(e.b.var26 != 0) as usize];
                fx_load(&mut e.b, fx, fx.frame_at(list));
                e.b.visible = 1;
                e.b.draw_flags = 0x10;
                e.b.mode = e.b.mode.wrapping_add(2);
            }
            2 => {}
            _ => return self.fx_free(i),
        }
        let (owner, class) = (self.effects.slots[i].owner58, self.effects.slots[i].owner_class);
        let o = self.body(owner);
        if o.live == 0 || o.k20 != class || o.kind != 0xC {
            return self.fx_free(i); // 0x92A2A
        }
        let (x, y, z, facing) = (o.xi(), o.yi(), o.zi(), o.facing);
        let e = &mut self.effects.slots[i];
        set_word(&mut e.b.x, x);
        set_word(&mut e.b.y, y);
        set_word(&mut e.b.z, z);
        e.b.facing = facing;
        self.draw_list_add_if_visible(self.effects.handle(i)); // jmp 0x49C0
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
    fn the_shockwave_follows_a_holding_thug_and_ends_with_the_hold() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        let h = g.enemies.handle(3);
        {
            let o = &mut g.enemies.slots[3].b;
            o.live = 1;
            o.k20 = 0x28;
            o.kind = 0xC;
            o.set_xi(0x120);
            o.set_yi(0);
            o.set_zi(0x60);
            o.facing = 1;
        }
        let i = g.effects.alloc().unwrap();
        let e = &mut g.effects.slots[i];
        e.b.live = 1;
        e.b.k20 = FX_SHOCKWAVE;
        e.owner58 = h;
        e.owner_class = 0x28;
        for _ in 0..30 {
            g.pool_c_update();
            let e = &g.effects.slots[i];
            assert_eq!((e.b.live, e.b.mode, e.b.xi(), e.b.zi(), e.b.facing), (1, 2, 0x120, 0x60, 1));
        }
        g.enemies.slots[3].b.set_xi(0x130);
        g.pool_c_update();
        assert_eq!(g.effects.slots[i].b.xi(), 0x130, "it follows the thug");
        g.enemies.slots[3].b.kind = 4; // the hold is over
        g.pool_c_update();
        assert_eq!(g.effects.slots[i].b.live, 0);
        let j = g.effects.alloc().unwrap();
        let e = &mut g.effects.slots[j];
        e.b.live = 1;
        e.b.k20 = FX_SHOCKWAVE;
        e.owner58 = h;
        e.owner_class = 0x2C; // another class in the slot: not the thug that made it
        g.enemies.slots[3].b.kind = 0xC;
        g.pool_c_update();
        assert_eq!(g.effects.slots[j].b.live, 0);
    }
}
