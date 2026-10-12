//! Pool C class `+$20 = 0x80` (handler 0x96A7E): a cloud of the ranking screen. The cloud manager (pool B
//! class 89, `prop/class89.rs`) creates them with a start position and a size as variant (0 to 2).
//!
//! Mode 0 (0x96A92) sets the draw flags 0x10 and the on-screen flag, takes the speed of the variant
//! (a 16.16 value per frame, table 0x96B34) and starts its picture (one record of 0x96B10, 0x96B1C or
//! 0x96B28 by variant: a picture that lasts and links to itself), then goes on into mode 2 at once.
//! Mode 2 (0x96AE2): the animation tick, X plus the speed, and when the cloud is 0x10 or more left of
//! the scroll2 camera it frees itself (0x96B06, mode 4 and 6 free too), else it goes into the draw list.

use super::{Effect, fx_load, fx_tick};
use crate::game::Game;

/// The pictures of the variants 0, 1 and any other (instruction constants).
const PICTURES: [u32; 3] = [0x96B10, 0x96B1C, 0x96B28];
/// The speed table of the variants.
const SPEEDS: u32 = 0x96B34;

impl Game {
    /// 0x96A7E pool C class 0x80 (jump table 0x96A8A: mode 0 -> 0x96A92, 2 -> 0x96AE2, 4 and 6 -> 0x96B0A).
    pub(super) fn fx_cloud(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => {
                let fx = self.fx_data();
                let var = self.effects.slots[i].b.var26;
                let speed = self.assets.frontend.text.u32(SPEEDS, 4 * var as u32) as i32;
                let picture = PICTURES[(var as usize).min(2)];
                let e: &mut Effect = &mut self.effects.slots[i];
                e.b.draw_flags = 0x10;
                e.b.visible = 1;
                e.speed60 = speed;
                fx_load(&mut e.b, fx, fx.frame_at(picture));
                e.b.mode += 2;
                self.fx_cloud_move(i);
            }
            2 => self.fx_cloud_move(i),
            _ => self.fx_free(i),
        }
    }

    /// 0x96AE2.
    fn fx_cloud_move(&mut self, i: usize) {
        let fx = self.fx_data();
        let h = self.effects.handle(i);
        let cam = self.cam.s2.cam_x;
        let e = &mut self.effects.slots[i];
        fx_tick(&mut e.b, fx); // jsr 0x15F4
        e.b.x = e.b.x.wrapping_add(e.speed60); // add.l d0,$8(a6)
        if (cam.wrapping_sub(0x10) as i16) >= e.b.xi() {
            e.b.mode += 2; // 0x96B06
            self.fx_free(i); // jmp 0x4964
        } else {
            self.draw_list_add_check(h); // jmp 0x49CA
        }
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

    /// A cloud moves its variant's speed (3.5, 2.5, 1.5 pixels) in every update, the first included, and frees
    /// itself when it is 0x10 or more left of the camera.
    #[test]
    fn a_cloud_drifts_by_its_speed_and_leaves_on_the_left() {
        if assets().is_none() {
            return;
        }
        for (var, per_frame) in [(0u16, 3.5f32), (1, 2.5), (2, 1.5)] {
            let mut g = Game::new(assets().unwrap());
            let i = g.effects.alloc().unwrap();
            let e = &mut g.effects.slots[i];
            e.b.live = 1;
            e.b.k20 = 0x80;
            e.b.var26 = var;
            e.b.set_xi(0x100);
            e.b.set_yi(0xC8);
            for n in 1..=4 {
                g.pool_c_update();
                let x = g.effects.slots[i].b.x as f32 / 65536.0;
                assert!((x - (256.0 + per_frame * n as f32)).abs() < 0.001, "variant {var} after {n} updates: {x}");
            }
        }
        let mut g = Game::new(assets().unwrap());
        let i = g.effects.alloc().unwrap();
        g.effects.slots[i].b.live = 1;
        g.effects.slots[i].b.k20 = 0x80;
        g.effects.slots[i].b.set_xi(0x30);
        g.cam.s2.cam_x = 0x40;
        g.pool_c_update();
        assert_ne!(g.effects.slots[i].b.live, 0, "0x33 is not 0x10 left of 0x40");
        g.cam.s2.cam_x = 0x50; // 0x40 is left of the 0x37 the cloud has reached
        g.pool_c_update();
        assert_eq!(g.effects.slots[i].b.live, 0);
    }
}
