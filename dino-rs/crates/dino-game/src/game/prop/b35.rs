//! Pool B class 35 (0x81FDE), the manhole cover grate.
//!
//! Spawned by 0x81FB0 from the hero pose routine at 0x19F8A / 0x2864 (hero anim 0x1F) during Script 21.
//! Mode 0 loads frame 0x82172 (0x81FF6), spawns B47 (0x81FFC / 0x8426A), advances to mode 2 (0x82002),
//! and falls through into mode 2 in the same frame.
//!
//! Mode 2 (0x82006):
//! - sub 0: follows the owner hero (+$76) at offsets from table 0x820FE by hero class (+0x20) and anim frame (+0x2C).
//!   When the hero throws the grate (player +$2D != 0), sets vx = 0x240, ay = -0x20, airborne = 1, sub = 2.
//! - sub 2: terrain collision (0x12CB4). When landed (airborne == 0), bounces: airborne = 1, vy = -vy, vx >>= 1, sub = 4.
//! - sub 4: terrain collision (0x12CB4). When landed (airborne == 0), enters mode 4 with timer +$80 = 0x1E (30 frames).
//!
//! Mode 4 (0x820DE): decrements timer +$80; blinks every other frame (drawn when timer & 1 != 0);
//! when timer counts below 0, frees itself (0x820F8 -> 0x4938).
//! Mode 6 frees itself (0x820F8 -> 0x4938).

use super::Prop;
use crate::game::body::Body;
use crate::game::{slot_of, Game, Handle, Slot};

impl Game {
    /// 0x81FDE pool B class 35 update.
    pub fn prop_b35(&mut self, p: &mut Prop, i: usize) {
        let h = self.props.handle(i);
        match p.b.mode {
            0 => {
                // 0x81FF2: load frame 0x82172
                let frame = self.assets.sprites.frame_at(0x82172);
                p.b.load_frame(&self.assets.sprites, frame); // 0x81FF6
                // 0x81FFC: jsr $8426a.l -> spawn B47
                self.spawn_prop(47, 0, None);
                // 0x82002: addq.b #2, $4(a6)
                p.b.mode = 2;
                // Falls through into mode 2 in the same frame!
                self.prop_b35_mode2(p, h);
            }
            2 => self.prop_b35_mode2(p, h),
            4 => {
                // 0x820DE
                let timer = p.m.b(0x80);
                let next = timer.wrapping_sub(1);
                p.m.sb(0x80, next);
                if timer == 0 {
                    // 0x820E2: bcs.w $820F8 -> 0x4938
                    self.prop_free(p, i);
                    return;
                }
                if next & 1 != 0 {
                    self.prop_b35_draw(&mut p.b, h); // 0x820F0: jmp $49CA
                }
            }
            _ => self.prop_free(p, i), // 0x820F8: jmp $4938
        }
    }

    /// 0x82006: mode 2 handler.
    fn prop_b35_mode2(&mut self, p: &mut Prop, h: Handle) {
        match p.b.sub {
            0 => {
                // 0x82018: held / following player
                let player_h = p.m.w(0x76);
                if let Some(Slot::Player(pi)) = slot_of(player_h) {
                    let player = &self.players[pi];
                    let ch = (player.b.k20 as usize) & 0xC;
                    let base_ptr = self.assets.enemy.t.u32(0x820FE, ch as u32);
                    let frame_step = player.b.param_a as usize;
                    let off_ptr = base_ptr + (frame_step * 4) as u32;
                    let mut dx = self.assets.enemy.t.i16(off_ptr, 0);
                    let dy = self.assets.enemy.t.i16(off_ptr, 2);
                    if player.b.facing != 0 {
                        dx = dx.wrapping_neg();
                    }
                    p.b.set_xi(player.b.xi().wrapping_add(dx));
                    p.b.set_yi(player.b.yi().wrapping_add(dy));
                    p.b.set_zi(player.b.zi().wrapping_sub(1));
                    if player.b.param_b != 0 {
                        // 0x82064: throw release!
                        p.b.vx = 0x0240;
                        p.b.vy = 0;
                        p.b.ay = -0x0020;
                        p.b.airborne = 1;
                        p.b.sub += 2;
                    }
                }
                self.prop_b35_draw(&mut p.b, h); // 0x82080: jmp $49CA
            }
            2 => {
                // 0x82086: first flight arc
                if let Some(o) = self.terrain_collide(&mut p.b, h, false) { // 0x12CB4
                    p.m.sw(0x94, o); // 0x12CB4 leaves the obstacle in +$94
                }
                if p.b.airborne == 0 {
                    // 0x82094: bounce!
                    p.b.airborne = 1;
                    p.b.vy = p.b.vy.wrapping_neg();
                    p.b.vx = ((p.b.vx as u16) >> 1) as i16;
                    p.b.sub += 2;
                }
                p.b.motion(); // 0x189C
                self.prop_b35_draw(&mut p.b, h); // 0x820B2: jmp $49CA
            }
            _ => {
                // 0x820B8: second flight arc (sub >= 4)
                if let Some(o) = self.terrain_collide(&mut p.b, h, false) { // 0x12CB4
                    p.m.sw(0x94, o); // 0x12CB4 leaves the obstacle in +$94
                }
                if p.b.airborne == 0 {
                    // 0x820C6: landed
                    p.b.mode = 4;
                    p.m.sb(0x80, 0x1E);
                }
                p.b.motion(); // 0x189C
                self.prop_b35_draw(&mut p.b, h); // 0x820D8: jmp $49CA
            }
        }
    }

    /// 0x49CA: set visible flag (0x16FA) and add to draw list if on screen.
    fn prop_b35_draw(&mut self, b: &mut Body, h: Handle) {
        let (cx, cy) = if self.w.vertical_scene != 0 { (0, 0) } else { (self.cam.s2.cam_x, self.cam.s2.cam_y) };
        let dx = b.xi().wrapping_sub(cx as i16).wrapping_add(0x100) as u16;
        let dy = b.yi().wrapping_add(b.zi()).wrapping_sub(cy as i16).wrapping_add(0x40) as u16;
        b.visible = (dx <= 0x380 && dy <= 0x1C0) as u8;
        if b.visible != 0 {
            let key = ((b.zi() as u16 as u32) << 16) | (h as u32);
            self.draw_list.push(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn test_game() -> Game {
        let mut pack = AssetPack::default();
        let mut bytes = Vec::new();
        // Setup synthetic offset table 0x820FE:
        // 4 pointers pointing to 0x8210e, 0x82126, 0x8213e, 0x82156
        for ptr in [0x8210e_u32, 0x82126, 0x8213e, 0x82156] {
            bytes.extend_from_slice(&ptr.to_be_bytes());
        }
        // Synthetic offsets: for Jack frame 0: dx=10, dy=-5
        bytes.extend_from_slice(&10_i16.to_be_bytes());
        bytes.extend_from_slice(&(-5_i16).to_be_bytes());
        bytes.resize(0x70, 0);
        pack.enemy.t.blobs = vec![(0x820FE, bytes)];

        // Dummy frame at 0x82172
        let dummy_frame = dino_assets::sprites::Frame {
            rom_addr: 0x82172,
            stride: 10,
            w0: 0,
            w1: 0,
            draw: dino_assets::sprites::FrameDraw::Comp { main: 0, overlay: None },
            param_a: 0,
            param_b: 0,
            kind: 0,
            duration: 1,
            last: false,
            body_box: 0,
            attack_box: 0,
            attach: 0,
            extra: [0; 4],
            next: 0,
        };
        pack.sprites.frames.push(dummy_frame);
        pack.sprites.frame_by_addr.insert(0x82172, 0);

        Game::new(Box::leak(Box::new(pack)))
    }

    #[test]
    fn b35_follows_hero_position() {
        let mut g = test_game();
        g.players[0].b.live = 1;
        g.players[0].b.k20 = 0; // Jack
        g.players[0].b.facing = 1; // facing right -> negates dx (10 -> -10)
        g.players[0].b.set_xi(1544);
        g.players[0].b.set_yi(0);
        g.players[0].b.set_zi(56);
        g.players[0].b.param_a = 0; // frame 0: dx=10, dy=-5
        g.players[0].b.param_b = 0;

        let h = g.spawn_prop(35, 0, Some((1544, 0, 56))).unwrap();
        let i = match slot_of(h).unwrap() {
            Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };
        g.props.slots[i].m.sw(0x76, Game::player_handle(0));

        // First update: mode 0 -> loads frame 0x82172, spawns B47, falls into mode 2 sub 0
        g.pool_b_update();
        let p = &g.props.slots[i];
        assert_eq!(p.b.mode, 2);
        assert_eq!(p.b.sub, 0);
        // Position at frame 0: 1544 + (-10) = 1534, 0 + (-5) = -5, 56 - 1 = 55
        assert_eq!((p.b.xi(), p.b.yi(), p.b.zi()), (1534, -5, 55));

        // B47 should have been spawned
        let b47 = g.props.slots.iter().find(|pr| pr.b.live != 0 && pr.b.k20 / 4 == 47);
        assert!(b47.is_some(), "B47 was spawned by B35");
    }
}
