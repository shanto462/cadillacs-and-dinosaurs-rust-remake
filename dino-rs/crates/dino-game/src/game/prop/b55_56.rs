//! Pool B class 55 (0x8574C) and class 56 (0x85916): Episode 6 Area 1 rain effects.
//! Spec: D3e part 3 sections 2.1, 2.3, A07, W305.
//!
//! - B55 (0x8574C): 75 rain streaks of 4 scroll1 tiles redrawn every frame (8 patterns,
//!   phase advancing by 8 each frame). When camera X reaches 0x500 or more, clears the
//!   scroll1 text block [0, 7, 0x30, 0x19] via 0xF94 (hud_clear_rect) and frees itself.
//! - B56 (0x85916): 24 slots 20 pixels apart across the top (camera X + 0x10 + slot * 0x14).
//!   Every slot tries every 5 frames. On hit (density 2/16, 4/16, 8/16 by +$A2 timer),
//!   reloads slot timer with 30 and spawns pool C class 2 drop (FX_CAVE_DROP) at Y = 0xD0
//!   with depth from table 0x85AE4 and variant 0, 4, 8 from variant table. Plays sound 0x8D
//!   every 241 frames. Transitions to mode 4 when camera X > 0x500, and frees when camera
//!   X > 0x580 or when countdown +$A2 finishes.

use super::tile_block::TilePos;
use super::Prop;
use crate::game::effect::FX_CAVE_DROP;
use crate::game::Game;

impl Game {
    /// 0x8574C: pool B class 55 update (rain streaks).
    pub fn prop_b55(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x85760: mode 0
                p.b.mode = 2; // 0x85760: addq.b #2, $4(a6)
                p.m.sw(0xA0, 1); // 0x85764: move.w #1, $a0(a6)
                p.m.sb(0xA2, 0); // 0x8576A: clr.b $a2(a6)
                // 0x8576A falls through into 0x8576E
                self.prop_b55_step(p, i);
            }
            2 => {
                self.prop_b55_step(p, i);
            }
            _ => {
                // 0x85910: jmp 0x4938
                self.prop_free(p, i);
            }
        }
    }

    /// 0x8576E: rain streaks frame step.
    fn prop_b55_step(&mut self, p: &mut Prop, i: usize) {
        // 0x8576E: cmpi.w #$500, $744(a5)
        if self.cam.s2.cam_x >= 0x500 {
            // 0x85796: move.b #$4, $4(a6) ; lea $857A6, a0 ; jmp $F94
            p.b.mode = 4;
            self.hud_clear_rect([0x00, 0x07, 0x30, 0x19]);
            // The original's jmp $F94 returns to the object loop, so the slot lives on: mode 4 (0x85910) frees it on the next frame.
            return;
        }
        // 0x85778: subq.w #1, $a0(a6) ; bne 0x85794
        let t = p.m.w(0xA0).wrapping_sub(1);
        p.m.sw(0xA0, t);
        if t != 0 {
            return;
        }
        // 0x85780: move.w #1, $a0(a6)
        p.m.sw(0xA0, 1);
        // 0x85786: addq.b #8, $a2(a6) ; andi.b #$3F, $a2(a6)
        let phase = p.m.b(0xA2).wrapping_add(8) & 0x3F;
        p.m.sb(0xA2, phase);

        // 0x857AA: redraw 75 streaks
        for streak in 0..75 {
            let word_off = self.assets.enemy.t.u16(0x857D8, streak * 2);
            let byte_off = self.assets.enemy.t.u8(0x8586E, streak);
            let d5 = (u32::from(byte_off) + u32::from(phase)) & 0x3F;
            let mut a0 = 0x90A000u32.wrapping_add(u32::from(word_off));
            // 0x858BA: 4 tile codes from 0x858D0 + d5, stepping by -0x7C
            for step in 0..4 {
                let code = self.assets.enemy.t.u16(0x858D0 + d5, step * 2);
                self.prop_tile_code(TilePos::at(a0), code);
                a0 = a0.wrapping_sub(0x7C);
            }
        }
    }

    /// 0x85916: pool B class 56 update (rain drops spawner).
    pub fn prop_b56(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x8592C: mode 0
                p.m.sw(0xBC, 0); // clr.w $bc(a6): sound countdown
                p.b.mode = 2; // addq.b #2, $4(a6)
                p.m.sb(0xA0, 0); // clr.b $a0(a6)
                p.m.sw(0xA2, 1); // move.w #1, $a2(a6)
                // 0x8593E: initialize 24 slot timers to 1
                for s in 0..24 {
                    p.m.sb(0xA4 + s, 1);
                }
                // 0x8594A falls into 0x8594E (mode 2)
                self.prop_b56_mode2(p, i);
            }
            2 => {
                self.prop_b56_mode2(p, i);
            }
            4 => {
                self.prop_b56_mode4(p, i);
            }
            _ => {
                // 0x85B5E: jmp 0x4938
                self.prop_free(p, i);
            }
        }
    }

    /// 0x8594E: pool B class 56 mode 2 update.
    fn prop_b56_mode2(&mut self, p: &mut Prop, _i: usize) {
        // 0x8594E: subq.w #1, $bc(a6) ; bcc 0x85970
        let snd_t = p.m.w(0xBC).wrapping_sub(1);
        p.m.sw(0xBC, snd_t);
        if (snd_t as i16) < 0 {
            // 0x85954: reload 0xF0 (240), queue sound 0x8D
            p.m.sw(0xBC, 0xF0);
            self.snd_request(0x8D, 0x10, 0);
        }

        // 0x85970: cmpi.w #$500, $744(a5) ; bgt 0x859C8
        if self.cam.s2.cam_x > 0x500 {
            // 0x859C8: switch to mode 4
            p.b.mode = 4;
            p.m.sw(0xA2, 0x200);
            for s in 0..24 {
                p.m.sb(0xA4 + s, 1);
            }
            return;
        }

        // Run slot loop
        self.prop_b56_slots(p);

        // 0x859BA: cmpi.w #$1FF, $a2(a6) ; bge 0x859C6 ; addq.w #1, $a2(a6)
        let a2 = p.m.w(0xA2);
        if a2 < 0x1FF {
            p.m.sw(0xA2, a2.wrapping_add(1));
        }
    }

    /// 0x85B04: pool B class 56 mode 4 update (exit phase).
    fn prop_b56_mode4(&mut self, p: &mut Prop, i: usize) {
        // 0x85B04: cmpi.w #$580, $744(a5) ; bgt 0x85B56
        if self.cam.s2.cam_x > 0x580 {
            p.b.mode = 6;
            self.prop_free(p, i);
            return;
        }

        // Run slot loop
        self.prop_b56_slots(p);

        // 0x85B4E: subq.w #1, $a2(a6) ; bcs 0x85B56
        let a2 = p.m.w(0xA2).wrapping_sub(1);
        p.m.sw(0xA2, a2);
        if (a2 as i16) < 0 {
            p.b.mode = 6;
            self.prop_free(p, i);
        }
    }

    /// 0x85988 / 0x85B1C: loop through 24 slots across the screen width.
    fn prop_b56_slots(&mut self, p: &mut Prop) {
        // 0x8597A: d1 = cam_x + 0x10
        let mut slot_x = self.cam.s2.cam_x.wrapping_add(0x10);
        let a2_val = p.m.w(0xA2);
        // 0x859E6: select tables by +$A2
        let (prob_tbl, var_tbl) = if a2_val > 0x140 {
            (0x85A74, 0x85A84)
        } else if a2_val > 0x78 {
            (0x85A44, 0x85A54)
        } else {
            (0x85A14, 0x85A24)
        };

        for s in 0..24 {
            let t = p.m.b(0xA4 + s).wrapping_sub(1);
            p.m.sb(0xA4 + s, t);
            if t == 0 {
                let r = (self.rand() & 0xF) as u32;
                let hit = self.assets.enemy.t.u8(prob_tbl, r) != 0;
                if hit {
                    p.m.sb(0xA4 + s, 0x1E);
                    // 0x85AA4: spawn drop
                    if let Some(j) = self.effects.alloc() {
                        let r_depth = (self.rand() & 0x1E) as u32;
                        let z = self.assets.enemy.t.u16(0x85AE4, r_depth) as i16;
                        let r_var = (self.rand() & 0x1E) as u32;
                        let var = self.assets.enemy.t.u16(var_tbl, r_var);
                        let e = &mut self.effects.slots[j];
                        e.b.live = 1;
                        e.b.k20 = FX_CAVE_DROP;
                        e.b.set_xi(slot_x as i16);
                        e.b.set_yi(0xD0);
                        e.b.set_zi(z);
                        e.b.var26 = var;
                    }
                } else {
                    p.m.sb(0xA4 + s, 5);
                }
            }
            slot_x = slot_x.wrapping_add(0x14);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::enemy::ByteTables;
    use dino_assets::AssetPack;

    fn test_game(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs, ..Default::default() };
        let mut g = Game::new(Box::leak(Box::new(a)));
        g.w.stage = 5;
        g.cam.s2.area = 1;
        g
    }

    #[test]
    fn b55_advances_phase_and_frees_at_cam500() {
        let offsets = vec![0u8; 150]; // 75 words
        let phases = vec![0u8; 75];
        let glyphs = vec![0u8; 64];

        let blobs = vec![
            (0x857D8, offsets),
            (0x8586E, phases),
            (0x858D0, glyphs),
        ];
        let mut g = test_game(blobs);
        g.cam.s2.cam_x = 0x200;

        let h = g.spawn_prop(55, 0, None).unwrap();
        let i = match crate::game::slot_of(h).unwrap() {
            crate::game::Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };

        // First update: mode 0 inits, advances phase by 8
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 2);
        assert_eq!(g.props.slots[i].m.b(0xA2), 8);

        // Next update: phase becomes 16
        g.pool_b_update();
        assert_eq!(g.props.slots[i].m.b(0xA2), 16);

        // Advance cam_x to 0x500: mode 4 (the slot lives on this frame), freed by the next update
        g.cam.s2.cam_x = 0x500;
        g.pool_b_update();
        assert_eq!((g.props.slots[i].b.live, g.props.slots[i].b.mode), (1, 4));
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.live, 0);
    }

    #[test]
    fn b56_spawns_drops_and_sound() {
        let prob = vec![1u8; 16]; // always hit
        let var_tbl: Vec<u8> = (0..16u16).flat_map(|_| 4u16.to_be_bytes()).collect();
        let depth_tbl: Vec<u8> = (0..16u16).flat_map(|_| 0x120u16.to_be_bytes()).collect();

        let blobs = vec![
            (0x85A14, prob.clone()),
            (0x85A24, var_tbl.clone()),
            (0x85A44, prob.clone()),
            (0x85A54, var_tbl.clone()),
            (0x85A74, prob),
            (0x85A84, var_tbl),
            (0x85AE4, depth_tbl),
        ];
        let mut g = test_game(blobs);
        g.cam.s2.cam_x = 0x200;

        let h = g.spawn_prop(56, 0, None).unwrap();
        let i = match crate::game::slot_of(h).unwrap() {
            crate::game::Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };

        // Update 0: mode 0 queues sound 0x8D, spawns drops for initial slots (timer 1 decrements to 0)
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 2);
        assert_eq!(g.w.sounds, vec![0x8D]);

        // At least one effect drop should have been spawned in pool C
        let spawned = g.effects.slots.iter().any(|e| e.b.live != 0 && e.b.k20 == FX_CAVE_DROP);
        assert!(spawned);
    }
}
