//! Pool B class 72 (`+$20 = 0x120`, handler 0x89984): the spawner of a flock of birds (pool A class 46).
//! Episode 2 area 0 places variant 0 at t = 80 and variant 0x200 at t = 620. The hunters of the opening
//! movie's jungle create one with the variant 0x600 (0x3DA96, M1-10).
//!
//! Mode 0 (0x89998) sets `+$A0` = 8 (frames between two birds) and `+$A2` = 6 (birds + 1). A variant
//! other than 0 sets `+$A2` = 8 (0x400: 0xC, nothing uses it) and goes on in mode 2 (0x899DC) in the
//! same frame. Mode 2: `+$A0` counts down; at 0 `+$A2` counts down; at 0 the object ends (mode 4), else
//! a bird is made (0x899FA) and `+$A0` is 8 again (0x89AAC, also when the pool was full: the `bcs` at
//! 0x89A04 returns to the reload). The bird (pool A class 46, palette override 0x98): X = the scroll2
//! camera + 0xB0 + (`rand()` & 0x3C) (0x40 less for the variant 0x600), Y = camera Y + 0x40, depth =
//! camera depth G.$74C, the spawner's `+$5C` copied, its variant copied (0x600 becomes 0, and for the
//! variant 0x400 every other bird goes the other way, variant 0x200).

use super::Prop;
use crate::game::Game;

/// Offsets of this class.
mod f {
    pub const A0: usize = 0xA0;
    pub const A2: usize = 0xA2;
}

impl Game {
    /// 0x89DB4: the flock spawner of the variant 0x600.
    pub fn prop_flock_spawn(&mut self) {
        let Some(i) = self.props.alloc() else { return };
        let p = &mut self.props.slots[i];
        p.b.live = 1;
        p.b.k20 = 0x120;
        p.b.var26 = 0x600;
    }

    /// 0x89984 pool B class 72 (jump table 0x89990: mode 0 -> 0x89998, 2 -> 0x899DC, 4 and 6 -> 0x89AB4).
    pub fn prop_class72(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode = 2; // 0x89998: move.b #2,$4(a6)
                p.b.live = 2;
                p.b.facing = 0;
                p.m.sw(f::A0, 8);
                p.m.sw(f::A2, 6);
                if p.b.var26 != 0 {
                    p.m.sw(f::A0, 8);
                    p.m.sw(f::A2, 8);
                    if p.b.var26 == 0x400 {
                        p.m.sw(f::A2, 0xC);
                    }
                }
                self.prop_c72_frame(p);
            }
            2 => self.prop_c72_frame(p),
            _ => self.prop_free(p, i), // 0x89AB4: jmp 0x4938
        }
    }

    /// 0x899DC.
    fn prop_c72_frame(&mut self, p: &mut Prop) {
        let a0 = p.m.w(f::A0).wrapping_sub(1);
        p.m.sw(f::A0, a0);
        if a0 != 0 {
            return;
        }
        let a2 = p.m.w(f::A2).wrapping_sub(1);
        p.m.sw(f::A2, a2);
        if a2 == 0 {
            p.b.mode = 4; // move.b #4,$4(a6)
            return;
        }
        // 0x899FA: a bird. A full pool (0x89A04 bcs) skips the bird and still reloads +$A0.
        if let Some(j) = self.enemies.alloc() {
            let var = p.b.var26;
            let cx = self.cam.s2.cam_x as i16;
            let r = self.rand();
            let e = &mut self.enemies.slots[j];
            e.b.live = 1;
            e.b.k20 = 0xB8;
            e.b.palette_override = 0x98;
            let mut x = cx.wrapping_add(0xB0).wrapping_add((r & 0x3C) as i16);
            if var == 0x600 {
                x = x.wrapping_sub(0x40);
            }
            e.b.set_xi(x);
            e.b.set_yi((self.cam.s2.cam_y as i16).wrapping_add(0x40));
            e.b.set_zi(self.cam.s2.cam_z as i16); // move.w G.$74C, $10(a0)
            e.b.dir = 0;
            e.m.sw(0x5C, p.m.w(0x5C));
            e.b.var26 = var;
            if var == 0x600 {
                e.b.var26 = 0;
            }
            if var == 0x400 {
                e.b.var26 = 0;
                if p.m.w(f::A2) & 1 == 0 {
                    e.b.var26 = 0x200;
                }
            }
        }
        p.m.sw(f::A0, 8); // 0x89AAC
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game() -> Game {
        Game::new(Box::leak(Box::new(AssetPack::default())))
    }

    fn flock(g: &Game) -> Option<&Prop> {
        g.props.slots.iter().find(|s| s.b.live != 0 && s.b.k20 == 72 * 4)
    }

    fn birds(g: &Game) -> Vec<(i16, i16, i16, u16)> {
        g.enemies
            .slots
            .iter()
            .filter(|e| e.b.live != 0 && e.b.k20 == 0xB8)
            .map(|e| (e.b.xi(), e.b.yi(), e.b.zi(), e.b.var26))
            .collect()
    }

    /// Variant 0 makes 5 birds, on updates 8, 16, 24, 32 and 40. The first uses one `rand()`.
    #[test]
    fn var0_makes_five_birds_eight_frames_apart() {
        let mut g = game();
        g.cam.s2.cam_x = 0x100;
        g.cam.s2.cam_y = 0x20;
        g.cam.s2.cam_z = 0x55;
        g.w.rng = 0;
        g.spawn_prop(72, 0, None).unwrap();
        for _ in 0..7 {
            g.pool_b_update();
        }
        assert!(birds(&g).is_empty());
        g.pool_b_update();
        assert_eq!(birds(&g), vec![(0x1B8, 0x60, 0x55, 0)]);
        assert_eq!(g.enemies.slots.iter().find(|e| e.b.k20 == 0xB8).unwrap().b.palette_override, 0x98);
        for _ in 0..32 {
            g.pool_b_update();
        }
        assert_eq!(birds(&g).len(), 5, "update 40");
        assert_eq!(flock(&g).unwrap().b.mode, 2);
        g.pool_b_update(); // update 41..47 keep the gap
        for _ in 0..7 {
            g.pool_b_update();
        }
        assert_eq!(birds(&g).len(), 5);
        assert_eq!(flock(&g).unwrap().b.mode, 4, "update 48 ends the flock");
        g.pool_b_update();
        assert!(flock(&g).is_none());
    }

    /// Variant 0x200 starts `+$A2` at 8, so it makes 7 birds and each one keeps the variant.
    #[test]
    fn var_200_makes_seven_birds() {
        let mut g = game();
        g.cam.s2.cam_x = 0x100;
        g.w.rng = 0;
        g.spawn_prop(72, 0x200, None).unwrap();
        for _ in 0..56 {
            g.pool_b_update();
        }
        let b = birds(&g);
        assert_eq!(b.len(), 7);
        assert!(b.iter().all(|(_, _, _, v)| *v == 0x200));
        assert_eq!(flock(&g).unwrap().b.mode, 2);
        for _ in 0..8 {
            g.pool_b_update();
        }
        assert_eq!(flock(&g).unwrap().b.mode, 4);
    }

    /// A full pool A makes no bird and does not call `rand()`, and `+$A0` is still reloaded (0x89AAC).
    #[test]
    fn a_full_pool_reloads_the_timer_without_a_rand() {
        let mut g = game();
        while g.enemies.alloc().is_some() {}
        g.spawn_prop(72, 0, None).unwrap();
        g.pool_b_update();
        let i = g.props.slots.iter().position(|s| s.b.k20 == 72 * 4).unwrap();
        g.props.slots[i].b.mode = 2;
        g.props.slots[i].m.sw(0xA0, 1);
        g.props.slots[i].m.sw(0xA2, 3);
        g.w.rng = 0x1111;
        g.pool_b_update();
        assert!(birds(&g).is_empty());
        assert_eq!(g.w.rng, 0x1111);
        assert_eq!((g.props.slots[i].m.w(0xA0), g.props.slots[i].m.w(0xA2)), (8, 2));
    }
}
