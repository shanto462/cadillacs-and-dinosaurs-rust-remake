//! Pool B class 1 (0x75238): chain-link fence / garage gate (Episode 4 and Episode 7).
//! Spec: D3e part 2 section 5, A211, W214; D3e part 3 section 2.6, A16, A17, W312.

use super::tile_block::TilePos;
use super::Prop;
use crate::game::{Game, Handle};

impl Game {
    /// 0x75238: pool B class 1 update.
    pub fn prop_b1(&mut self, p: &mut Prop, h: Handle, i: usize) {
        match p.b.mode {
            0 => {
                // 0x7524C: init
                p.b.draw_flags = 0x10; // 0x7524C
                p.b.visible = 1; // 0x75252: move.b #1,$1(a6)
                p.m.sw(0xA0, p.b.xi() as u16); // 0x75258: store initial X
                p.b.box_table = 0x75414; // 0x7525E
                p.b.hp_bar = 1; // 0x7526E
                p.b.hp = 1; // 0x75274
                p.b.hp_prev = 1; // 0x7527A
                let frame = self.assets.sprites.frame_at(0x75400); // 0x75294: sprite 326
                p.b.load_frame(&self.assets.sprites, frame); // 0x75298: jsr 0x1236
                p.b.mode = 2; // 0x7529E
                // Falls through to 0x752A2
                self.prop_b1_camera_check(p); // 0x752A2: bsr 0x752B2 (mode 6 when passed; the frame goes on)
                self.prop_draw_list(p, i); // 0x752A6: jsr 0x49C0
                self.prop_register_hit(h); // 0x752AC: jmp 0x4BFA
            }
            2 => {
                // 0x752A2: idle / solid
                self.prop_b1_camera_check(p); // 0x752A2
                self.prop_draw_list(p, i); // 0x752A6
                self.prop_register_hit(h); // 0x752AC
            }
            4 => {
                // 0x752C8: hit reaction / broken fence
                if p.b.sub != 0 {
                    // 0x75318: lingering broken fence (sub state 2)
                    self.prop_b1_camera_check(p); // 0x75318: bsr 0x752B2
                    self.prop_draw_list(p, i); // 0x7531A: jmp 0x49C0
                    return;
                }

                // Sub state 0
                if p.b.hp != 0 {
                    // 0x752D4: hit stop / flash
                    p.m.sb(0x5F, 1);
                    let stop = p.b.hitstop.wrapping_sub(1);
                    p.b.hitstop = stop;
                    if stop == 0xFF {
                        p.m.sb(0x5F, 0); // 0x752E0
                        p.b.mode = 2; // 0x752E4
                    }
                    self.prop_b1_camera_check(p); // 0x75318: bsr 0x752B2
                    self.prop_draw_list(p, i); // 0x7531A: jmp 0x49C0
                    return;
                }

                // 0x752EC: break
                self.prop_b1_rewrite_attr(p.b.var26); // 0x75320
                // 0x752F0 - 0x75300: effect C21 variant 0xC, facing 1, Z reduced by 0x30
                self.prop_c21_spawn(p.b.xi(), p.b.yi(), p.b.zi().wrapping_sub(0x30), 0x0C, 1);
                p.b.set_yi(p.b.yi().wrapping_add(0x20)); // 0x75306: gate Y rises 0x20
                let frame = self.assets.sprites.frame_at(0x7540A); // 0x7530C: sprite 327
                p.b.load_frame(&self.assets.sprites, frame);
                p.b.sub = 2; // 0x75312: move.b #2, $5(a6)

                // 0x75318
                self.prop_b1_camera_check(p); // 0x75318: bsr 0x752B2
                self.prop_draw_list(p, i); // 0x7531A: jmp 0x49C0
            }
            _ => self.prop_free(p, i), // 0x753FA: jmp 0x4938
        }
    }

    /// 0x752B2: once the camera X - 0x80 is at or past the fence X (signed), mode 6; the caller goes on with its frame.
    fn prop_b1_camera_check(&self, p: &mut Prop) {
        if self.prop_b1_camera_off(p) {
            p.b.mode = 6; // 0x752C0
        }
    }

    /// The test of 0x752B2 (`cmp.w $8(a6)` with the camera X - 0x80, `blt` keeps the fence).
    fn prop_b1_camera_off(&self, p: &Prop) -> bool {
        let d0 = (self.cam.s2.cam_x as i16).wrapping_sub(0x80);
        d0 >= p.b.xi()
    }

    /// 0x75320: rewrite tile attributes (13 columns by 6 rows).
    fn prop_b1_rewrite_attr(&mut self, var: u16) {
        let base_addr = match var {
            0 => 0x90F8A6,
            4 => 0x90F3E6,
            8 => 0x90FF66,
            _ => 0x90F8A6,
        };
        let t = &self.assets.enemy.t;
        let mut s = 0x7535E;
        for c in 0..13 {
            let col_pos = TilePos::at(base_addr).right(c);
            for r in 0..6 {
                let val = t.u16(s, 0);
                s += 2;
                self.prop_tile_attr(col_pos.down(r), 0x03FF, val);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use std::path::Path;

    fn rom_game() -> Option<Game> {
        let path = Path::new("../../../dino.zip");
        if !path.exists() {
            return None;
        }
        let assets: &'static AssetPack = Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads")));
        Some(Game::new(assets))
    }

    #[test]
    fn b1_init_and_idle() {
        let Some(mut g) = rom_game() else { return };
        let h = g.spawn_prop(1, 4, None).expect("spawn b1 var 4");
        let i = (0..24).find(|&k| g.props.handle(k) == h).unwrap();

        let mut p = std::mem::take(&mut g.props.slots[i]);
        p.b.set_xi(500);
        g.cam.s2.cam_x = 100;
        g.prop_b1(&mut p, h, i);
        assert_eq!(p.b.live, 1);
        assert_eq!(p.b.draw_flags, 0x10);
        assert_eq!(p.b.box_table, 0x75414);
        assert_eq!(p.b.hp, 1);
        assert_eq!(p.b.mode, 2);
    }

    #[test]
    fn b1_break_on_hit() {
        let Some(mut g) = rom_game() else { return };
        let h = g.spawn_prop(1, 4, None).expect("spawn b1 var 4");
        let i = (0..24).find(|&k| g.props.handle(k) == h).unwrap();

        let mut p = std::mem::take(&mut g.props.slots[i]);
        p.b.set_xi(500);
        p.b.set_yi(100);
        g.cam.s2.cam_x = 100;
        g.prop_b1(&mut p, h, i);

        // Hit sets hp = 0, mode = 4
        p.b.hp = 0;
        p.b.mode = 4;
        p.b.sub = 0;
        g.prop_b1(&mut p, h, i);

        assert_eq!(p.b.sub, 2);
        assert_eq!(p.b.yi(), 100 + 0x20); // Y raised by 0x20
        assert_eq!(p.b.frame, g.assets.sprites.frame_at(0x7540A)); // Broken fence sprite composition 327
    }

    #[test]
    fn b1_camera_off_frees() {
        let Some(mut g) = rom_game() else { return };
        let h = g.spawn_prop(1, 4, None).expect("spawn b1 var 4");
        let i = (0..24).find(|&k| g.props.handle(k) == h).unwrap();

        let mut p = std::mem::take(&mut g.props.slots[i]);
        p.b.set_xi(200);
        // Camera moves past fence (cam_x - 0x80 >= fence X)
        // 280 - 0x80 = 0x98 (152) < 200 -> not off
        g.cam.s2.cam_x = 280;
        assert!(!g.prop_b1_camera_off(&p));

        // 328 - 0x80 = 200 >= 200 -> off!
        g.cam.s2.cam_x = 200 + 0x80;
        assert!(g.prop_b1_camera_off(&p));

        g.prop_b1(&mut p, h, i);
        assert_eq!(p.b.mode, 6);

        // Next frame mode 6 frees
        g.prop_b1(&mut p, h, i);
        assert_eq!(p.b.live, 0);
    }
}
