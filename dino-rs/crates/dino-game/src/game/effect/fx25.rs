//! Pool C class 25 (+$20 = 0x64), handler 0x96296: the spark on the fuse of a TNT barrel (made by 0x9625C, `item_spawn_fuse_spark`: +$60 = the barrel, X + 10,
//! Y + 0x39). It shows the animation 1 of the table 0x28904 at the barrel's X + 10, Y + 0x39, Z - 1 for as long as the barrel is alive, still a container
//! (class word 0x14) and in its idle state (mode 2, sub 0); in another state of the barrel it is not drawn (and not updated); when the barrel is gone it ends.

use super::*;

const SPARK_ANIMS: u32 = 0x28904;

impl Game {
    /// 0x96296: mode 0 (init, falls into the run) and 2; 4 and 6 free the slot (0x96316).
    pub(super) fn fx_fuse_spark(&mut self, i: usize) {
        let fx = self.fx_data();
        match self.effects.slots[i].b.mode {
            0 => {
                let e = &mut self.effects.slots[i];
                e.b.anim_table = SPARK_ANIMS;
                fx_start(&mut e.b, fx, 1);
                e.b.draw_flags = 0x10;
                e.b.mode = 2; // move.w #$200, $4(a6)
                e.b.sub = 0;
            }
            2 => {}
            _ => return self.fx_free(i),
        }
        let owner = self.effects.slots[i].owner60;
        let o = self.body(owner);
        if o.live == 0 || o.k20 != 0x14 {
            return self.fx_free(i); // 0x96316
        }
        if (o.mode, o.sub) != (2, 0) {
            return; // bne 0x96314: rts
        }
        let (x, y, z) = (o.xi().wrapping_add(0xA), o.yi().wrapping_add(0x39), o.zi().wrapping_sub(1));
        let e = &mut self.effects.slots[i];
        set_word(&mut e.b.x, x);
        set_word(&mut e.b.y, y);
        set_word(&mut e.b.z, z);
        fx_tick(&mut e.b, fx);
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

    #[test]
    fn the_spark_follows_an_idle_barrel_and_ends_with_it() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        let h = g.items.handle(5);
        {
            let o = &mut g.items.slots[5].b;
            o.live = 1;
            o.k20 = 0x14;
            o.set_state(0x0200_0000);
            o.set_xi(0x100);
            o.set_yi(0);
            o.set_zi(0x50);
        }
        let i = g.effects.alloc().unwrap();
        let e = &mut g.effects.slots[i];
        e.b.live = 1;
        e.b.k20 = FX_FUSE_SPARK;
        e.owner60 = h;
        g.pool_c_update();
        let e = &g.effects.slots[i];
        assert_eq!((e.b.live, e.b.mode, e.b.xi(), e.b.yi(), e.b.zi()), (1, 2, 0x10A, 0x39, 0x4F));
        g.items.slots[5].b.set_state(0x0204_0000); // another state: no update, no free
        g.items.slots[5].b.set_xi(0x200);
        g.pool_c_update();
        assert_eq!((g.effects.slots[i].b.live, g.effects.slots[i].b.xi()), (1, 0x10A));
        g.items.slots[5].b.k20 = 0x10; // not a container any more
        g.pool_c_update();
        assert_eq!(g.effects.slots[i].b.live, 0);
    }
}
