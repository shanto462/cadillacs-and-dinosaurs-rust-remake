//! Pool B class 70 (0x896E6): decorative rifle in the Episode 5 chief scene.
//! Spec: D3e part 2 s7, A223, W239, P212.
//!
//! One looping frame of table 0x89736 (composition 2324), never drawn: nothing sets +$01. Floor snap 0x12CB4, far despawn
//! 0x173C, draw 0x49C0. No hit box, no pickup, no animation step.

use super::Prop;
use crate::game::{Game, Handle};

impl Game {
    /// 0x896E6: pool B class 70 update (jump table 0x896F2).
    pub fn prop_b70(&mut self, p: &mut Prop, h: Handle, i: usize) {
        match p.b.mode {
            0 => {
                // 0x896FA
                p.b.live = 2;
                p.b.anim_table = 0x89736;
                p.m.sw(0x80, 0);
                p.m.sw(0x82, 0x64);
                let _ = self.terrain_collide(&mut p.b, h, false); // jsr 0x12CB4
                p.b.start_anim(&self.assets.sprites, 0); // jsr 0x120E, d0 = 0
                p.b.mode = 2; // addq.b #2,+$04
                self.prop_b70_run(p, i); // falls into 0x89724
            }
            2 => self.prop_b70_run(p, i), // 0x89724
            _ => self.prop_free(p, i),    // 0x89730: jmp 0x4938
        }
    }

    /// 0x89724: jsr 0x173C, jmp 0x49C0. Nothing sets +$01 (MAME m700_a223_e5_chief_gun: 0 in every record), so 0x49C0 never draws it.
    fn prop_b70_run(&mut self, p: &mut Prop, i: usize) {
        // 0x173C: free next frame (mode 6) when far off screen
        if self.w.vertical_scene != 0 {
            if (p.b.xi().wrapping_add(0x80) as u16) >= 0x280 {
                p.b.mode = 6;
            }
        } else {
            let d = p.b.xi().wrapping_sub(self.cam.s2.cam_x as i16).wrapping_add(0xC0) as u16;
            if d >= 0x300 {
                p.b.mode = 6;
            }
        }
        self.prop_draw_list(p, i); // jmp 0x49C0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{slot_of, Slot};
    use dino_assets::AssetPack;
    use std::path::Path;

    fn rom_game() -> Option<Game> {
        let path = Path::new("../../../dino.zip");
        let alt = Path::new("../dino.zip");
        let p = if path.exists() {
            path
        } else if alt.exists() {
            alt
        } else {
            return None;
        };
        let assets: &'static AssetPack = Box::leak(Box::new(dino_extract::extract(p).expect("the ROM set loads")));
        Some(Game::new(assets))
    }

    #[test]
    fn b70_is_decorative_and_despawns_off_screen() {
        let Some(mut g) = rom_game() else { return };
        let h = g.spawn_prop(70, 0, Some((0x1270, 0, 0x40))).unwrap();
        let i = match slot_of(h).unwrap() {
            Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };

        g.cam.s2.cam_x = 0x1120;
        g.pool_b_update();
        let p = &g.props.slots[i];
        assert_eq!(p.b.live, 2);
        assert_eq!(p.b.mode, 2);
        assert_eq!(p.b.anim_table, 0x89736);
        assert_eq!(p.b.body_box, 0);
        assert_eq!(p.b.attack_box, 0);
        assert_eq!(p.b.k20, 70 * 4);
        assert_eq!(p.b.visible, 0, "nothing sets +$01 (MAME: 0 in every record)");
        assert!(g.draw_list.is_empty(), "0x49C0 draws only with +$01 set");

        // 0x173C: X - cam + 0xC0 >= 0x300
        g.cam.s2.cam_x = 0;
        g.props.slots[i].b.set_xi(0x400);
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 6);
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.live, 0);
    }
}
