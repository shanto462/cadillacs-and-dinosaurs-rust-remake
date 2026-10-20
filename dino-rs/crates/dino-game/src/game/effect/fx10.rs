//! Pool C class 10 (+$20 = 0x28), handler 0x932D2: a shard of a broken pane. The window of the Episode 1 hall (pool B 12 variant 4, 0x79146) makes ten of them
//! with the loop 0x93362 (`prop_b12_shards`): the speeds and accelerations of the record, the variant (+$26, 0 to 0xC) is the byte offset of the picture in the
//! table 0x93426 (7 animations; the first frame of the animation is the picture).
//!
//! Mode 0 sets the blink counters (+$58 = +$5A = 10) and the draw flags 0x30, loads the picture (0x160A) and falls into mode 2. Mode 2 shows the shard, counts
//! +$5A down; at 0 the shard is hidden for that update, +$58 goes down by one (to 0 at the lowest) and refills +$5A: the shard blinks faster and faster, and
//! from +$58 = 0 it hides in every update (a count of 1). It moves (0x189C) and ends (mode 4, 0x4964) when its height is negative or its depth is below the
//! camera Z (G.$74C, a signed compare). The picture's animation is ticked (0x15F4, the carry is not read) and the shard is drawn if visible.

use super::*;

/// 0x93426: the animation table of the shards.
const SHARD_TABLE: u32 = 0x93426;

impl Game {
    /// 0x932D2: modes 0 (init, falls into the run) and 2; 4 and 6 free the slot (0x9335C).
    pub(super) fn fx_shard(&mut self, i: usize) {
        let fx = self.fx_data();
        match self.effects.slots[i].b.mode {
            0 => {
                let e = &mut self.effects.slots[i];
                e.owner58 = 10; // move.w #$A, $58(a6)
                e.owner_class = 10; // move.w #$A, $5A(a6)
                e.b.draw_flags = 0x30;
                let first = fx.anim(SHARD_TABLE, e.b.var26 >> 1); // the word table by +$26 (a byte offset), a0 = the record it points to
                fx_load(&mut e.b, fx, first); // 0x160A
                e.b.mode = e.b.mode.wrapping_add(2);
            }
            2 => {}
            _ => return self.fx_free(i),
        }
        let cam_z = self.cam.s2.cam_z as i16;
        let e = &mut self.effects.slots[i];
        e.b.visible = 1; // 0x93314
        e.owner_class = e.owner_class.wrapping_sub(1); // subq.w #1, $5A ; bne 0x93334
        if e.owner_class == 0 {
            e.b.visible = 0;
            let p = (e.owner58 as i16).wrapping_sub(1); // subq.w #1, $58 ; bpl
            e.owner58 = if p < 0 { 0 } else { p as u16 };
            e.owner_class = e.owner58;
        }
        e.b.motion(); // 0x189C
        if e.b.yi() < 0 || e.b.zi() < cam_z {
            // 0x93358: mode 4, then the free
            e.b.mode = e.b.mode.wrapping_add(2);
            return self.fx_free(i);
        }
        fx_tick(&mut self.effects.slots[i].b, fx); // 0x15F4
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

    /// A shard that flies up and falls: it blinks with +$5A (10 updates, then 9, 8 ...), is drawn while visible, and ends when its height goes
    /// negative (0x93358 frees it, mode 4 first).
    #[test]
    fn a_shard_blinks_faster_and_ends_below_the_floor() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        let i = g.effects.alloc().unwrap();
        {
            let e = &mut g.effects.slots[i];
            e.b.live = 1;
            e.b.k20 = FX_SHARD;
            e.b.var26 = 4; // the third animation of the table
            e.b.set_xi(0x100);
            e.b.set_yi(0x20);
            e.b.set_zi(0x100);
            e.b.vy = 0x200;
            e.b.ay = -0x40;
        }
        g.cam.s2.cam_z = 0;
        g.pool_c_update();
        let e = &g.effects.slots[i];
        assert_eq!((e.b.mode, e.owner58, e.owner_class, e.b.draw_flags, e.b.visible), (2, 10, 9, 0x30, 1), "init and one update");
        let mut hidden = 0;
        let mut n = 1;
        while g.effects.slots[i].b.live != 0 && n < 400 {
            g.pool_c_update();
            n += 1;
            if g.effects.slots[i].b.live != 0 && g.effects.slots[i].b.visible == 0 {
                hidden += 1;
            }
        }
        assert_eq!(g.effects.slots[i].b.live, 0, "freed when the height is negative");
        assert!(hidden >= 1, "the blink hid the shard at least once");
        assert_eq!(g.effects.free_count(), POOL_C.2);
    }
}
