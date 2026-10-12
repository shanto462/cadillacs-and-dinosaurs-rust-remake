//! Pool B class 89 (`+$20 = 0x164`, handler 0x8C982): the cloud manager of the ranking screen (M1-07). The
//! ranking routine creates it first (0x8C96C), before the Cadillac.
//!
//! Mode 0 (0x8C996) makes the 15 clouds of the start (pool C class 0x80, spawn routine 0x96A56: variant,
//! X plus the scroll2 camera, Y from the list 0x8CA2E, depth 0) and sets the timer `+$A0` (a byte) to 5.
//! Mode 2 (0x8C9BE) counts the timer down; when it is below 0 it draws a random number: its low four bits
//! are the next timer, its low two bits the size of a new cloud (0, 1 or 2; 3 makes none); the new cloud
//! starts 0x280 to the right of the camera at a Y taken from the size's list (0x8CA8E, 0x8CA9E, 0x8CAAE)
//! by a second random number (low bit clear, 0 to 14). Mode 4 frees (0x8CA28). The random numbers are
//! the only ones the ranking screen's objects draw besides those of the two heroes in the car.

use super::Prop;
use crate::game::Game;

/// Offsets of this class.
mod f {
    /// byte: frames until the next cloud (counts down; the action is at -1).
    pub const TIMER: usize = 0xA0;
    /// word: the size of the new cloud.
    pub const SIZE: usize = 0xA2;
}

/// The start list (15 x variant, X offset, Y) and the three Y lists of the new clouds.
const START_LIST: u32 = 0x8CA2E;
const HEIGHTS: [u32; 3] = [0x8CA8E, 0x8CA9E, 0x8CAAE];

impl Game {
    /// 0x8C982 pool B class 89 (jump table 0x8C98E: mode 0 -> 0x8C996, 2 -> 0x8C9BE, 4 -> 0x8CA28).
    pub fn prop_class89(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode += 2; // 0x8C996
                let cam = self.cam.s2.cam_x;
                for k in 0..15u32 {
                    let t = &self.assets.frontend.text;
                    let (var, x, y) = (t.u16(START_LIST, 6 * k), t.u16(START_LIST, 6 * k + 2), t.u16(START_LIST, 6 * k + 4));
                    self.prop_cloud_spawn(var, x.wrapping_add(cam), y); // jsr 0x96A56
                }
                p.m.sb(f::TIMER, 5);
            }
            2 => self.prop_c89_run(p),
            _ => self.prop_free(p, i), // 0x8CA28: jmp 0x4938
        }
    }

    /// 0x8C9BE.
    fn prop_c89_run(&mut self, p: &mut Prop) {
        let t = p.m.b(f::TIMER).wrapping_sub(1);
        p.m.sb(f::TIMER, t);
        if (t as i8) >= 0 {
            return; // bpl
        }
        let r = self.rand();
        p.m.sb(f::TIMER, (r & 0xF) as u8);
        let size = (r & 3) as usize;
        if size > 2 {
            return;
        }
        p.m.sw(f::SIZE, size as u16);
        let k = (self.rand() & 0xE) as u32;
        let y = self.assets.frontend.text.u16(HEIGHTS[size], k);
        let x = self.cam.s2.cam_x.wrapping_add(0x280);
        self.prop_cloud_spawn(size as u16, x, y);
    }

    /// 0x96A56: a cloud (pool C class 0x80) with variant `var` at X `x`, Y `y`, depth 0; a full pool makes none.
    fn prop_cloud_spawn(&mut self, var: u16, x: u16, y: u16) {
        let Some(i) = self.effects.alloc() else { return };
        let b = &mut self.effects.slots[i].b;
        b.live = 1;
        b.k20 = 0x80;
        b.var26 = var;
        b.set_xi(x as i16);
        b.set_yi(y as i16);
        b.set_zi(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use std::path::Path;
    use std::sync::OnceLock;

    fn assets() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// The first update makes the 15 clouds (3 sizes of 5, pool C slot 47 first) at the list's X plus the
    /// camera; the timer is then 5, so the first new cloud comes in the 6th update after it, at the right
    /// of the camera (the random number decides whether there is one: size 3 makes none).
    #[test]
    fn fifteen_clouds_at_the_start_then_new_ones_on_the_right() {
        if assets().is_none() {
            return;
        }
        let mut g = Game::new(assets().unwrap());
        g.cam.s2.cam_x = 8;
        let i = g.props.alloc().unwrap();
        g.props.slots[i].b.live = 1;
        g.props.slots[i].b.k20 = 0x164;
        g.pool_b_update();
        let live: Vec<(usize, u16)> = (0..48).filter(|&k| g.effects.slots[k].b.live != 0).map(|k| (k, g.effects.slots[k].b.var26)).collect();
        assert_eq!(live.len(), 15);
        assert_eq!((live[14], live[0]), ((47, 0), (33, 2)));
        assert_eq!(g.effects.slots[47].b.xi() as u16, g.assets.frontend.text.u16(START_LIST, 2).wrapping_add(8));
        let rng = g.w.rng;
        for _ in 0..5 {
            g.pool_b_update();
        }
        assert_eq!(g.w.rng, rng, "the timer counts down without random numbers");
        g.pool_b_update();
        assert_ne!(g.w.rng, rng, "the 6th update draws one");
    }
}
