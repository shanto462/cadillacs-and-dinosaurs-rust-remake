//! Pool C class 7 (+$20 = 0x1C), handler 0x92ECC: a puff of ground dust (a hero or a thug that slides, a spent shell). Two spawn entries:
//! 0x92E5E (`fx_spawn_dust`, at the position of the calling object) and 0x92EA4 (`fx_spawn_dust_at`, at a given position). The picture is the raw sprite
//! list animation 0x92F3A (6 records of 2 updates). A variant 0 object is always drawn; every other variant (the slide dust has 2) blinks: its visible byte is
//! `(G.$0 + d7) & 1` before each update (d7 = 0x2F - slot, the loop counter of 0x237B2). +$58 (the high byte) is cleared at the start.

use super::*;

/// 0x92F3A: the dust animation.
const DUST_FRAMES: u32 = 0x92F3A;

impl Game {
    /// 0x92ECC: mode 0 (init, falls into the run) and 2; 4 and 6 free the slot (0x92F34).
    pub(super) fn fx_dust(&mut self, i: usize) {
        let fx = self.fx_data();
        match self.effects.slots[i].b.mode {
            0 => {
                let e = &mut self.effects.slots[i];
                raw_load(&mut e.b, fx, fx.raw_at(DUST_FRAMES)); // 0x16C6
                e.b.draw_flags = 0x50;
                e.b.visible = 1;
                e.owner58 &= 0x00FF; // clr.b $58(a6)
                e.b.mode = e.b.mode.wrapping_add(2);
            }
            2 => {}
            _ => return self.fx_free(i),
        }
        if self.effects.slots[i].b.var26 != 0 {
            self.effects.slots[i].b.visible = (self.w.frame.wrapping_add(47 - i as u16) & 1) as u8; // move.b d0, $1(a6)
        }
        if raw_tick(&mut self.effects.slots[i].b, fx) {
            return self.fx_free(i); // bcs 0x92F34
        }
        self.draw_list_add_if_visible(self.effects.handle(i)); // jmp 0x49C0
    }

    /// 0x92E5E: dust of the variant `var` (d0) at the position of the object `b` (a6). A variant other than 0 is not made for an object that stands in water
    /// (+$53 != 0) with its height at or below 0x18 (`bls`). Nothing when pool C is full.
    pub fn fx_spawn_dust(&mut self, b: &Body, var: u16) {
        if var != 0 && b.wading != 0 && (b.yi() as u16) <= 0x18 {
            return;
        }
        self.fx_spawn_dust_at(b.xi(), b.yi(), b.zi(), var);
    }

    /// 0x92E76 / 0x92EA4: the allocation and the fields (X, Y, Z words, variant).
    pub fn fx_spawn_dust_at(&mut self, x: i16, y: i16, z: i16, var: u16) {
        let Some(j) = self.effects.alloc() else { return };
        let e = &mut self.effects.slots[j];
        e.b.live = 1;
        e.b.k20 = FX_DUST;
        set_word(&mut e.b.x, x);
        set_word(&mut e.b.y, y);
        set_word(&mut e.b.z, z);
        e.b.var26 = var;
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
    fn dust_lives_12_updates_and_a_water_walker_makes_none() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        let mut b = Body::default();
        b.set_xi(0x100);
        b.set_yi(0x10);
        b.wading = 1;
        g.fx_spawn_dust(&b, 2);
        assert_eq!(g.effects.free_count(), POOL_C.2, "in the water: no dust");
        g.fx_spawn_dust(&b, 0);
        assert_eq!(g.effects.free_count(), POOL_C.2 - 1, "variant 0 has no water test");
        b.wading = 0;
        g.fx_spawn_dust(&b, 2);
        assert_eq!(g.effects.free_count(), POOL_C.2 - 2);
        let i = g.effects.slots.iter().position(|e| e.b.live != 0 && e.b.var26 == 2).unwrap();
        let mut n = 0;
        while g.effects.slots[i].b.live != 0 && n < 50 {
            g.pool_c_update();
            n += 1;
        }
        assert_eq!(n, 12, "six records of two updates");
    }
}
