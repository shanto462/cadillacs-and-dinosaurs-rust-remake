//! Pool B class 40 (0x836A4): steam pipes in wall.
//! Spec: D3e part 1 section 3.1 (A1 to A3), D3e part 3 section 2.6 (A23, W315).
//!
//! Variants:
//! - Episode 1 area 0: var 0 at X 0x620, Z 0x130 (304), kind 0.
//! - Episode 8 area 0: var 4 at X 0x690, Z 0x40 (64), kind 0.
//! - Episode 8 area 1: var 8 at X 0x0B0, Z 0x340 (832), kind 1.
//! - Episode 8 area 1: var 0xC at X 0x580, Z 0x340 (832), kind 1.
//! - Episode 8 area 1: var 0x10 at X 0x6E0, Z 0x340 (832), kind 0.

use super::tile_block::TilePos;
use super::Prop;
use crate::game::{Game, Handle};

impl Game {
    /// 0x836A4: pool B class 40 update.
    pub fn prop_b40(&mut self, p: &mut Prop, h: Handle, i: usize) {
        match p.b.mode {
            0 => {
                // 0x836B8: init
                p.b.visible = 1; // 0x836BC: move.b #1,$1(a6)
                p.b.body_box = 1; // 0x836C2: move.b #1,$48(a6)
                p.b.attack_box = 0; // 0x836C8: move.b #0,$49(a6)
                p.m.sb(0xA7, 0); // 0x836CE: clr.b $A7(a6)
                let d1 = u32::from(p.b.var26 >> 2);
                let kind = self.assets.enemy.t.u8(0x83786, d1); // 0x836D2
                p.m.sb(0xA8, kind); // 0x836DC: move.b (a0,d1.w),$A8(a6)
                p.b.box_table = if kind == 0 { 0x8380A } else { 0x83822 }; // 0x836EA / 0x836F6
                p.b.hp = 8; // 0x836FE: move.w #8,$6C(a6)
                p.b.hp_prev = 8; // 0x83704: move.w #8,$6E(a6)
                p.b.mode += 2; // 0x836B8: addq.b #2,$4(a6)
                self.prop_b40_mode2(p, h, i); // falls into 0x8370A
            }
            2 => self.prop_b40_mode2(p, h, i),
            4 => self.prop_b40_mode4(p, i),
            _ => self.prop_free(p, i), // 0x83804: jmp 0x4938
        }
    }

    /// 0x8370A: mode 2 idle check and hit list registration.
    fn prop_b40_mode2(&mut self, p: &mut Prop, h: Handle, i: usize) {
        // 0x8370A: move.w $744(a5),d0 ; addi.w #$20,d0 ; cmp.w $8(a6),d0 ; bcc 0x83804
        let cam_x = self.cam.s2.cam_x;
        if cam_x.wrapping_add(0x20) >= p.b.xi() as u16 {
            return self.prop_free(p, i); // 0x83804 -> 0x4938
        }
        self.prop_register_hit(h); // 0x8371A: jmp 0x4BFA
    }

    /// 0x83720: mode 4 broken states.
    fn prop_b40_mode4(&mut self, p: &mut Prop, i: usize) {
        match p.b.sub {
            0 => {
                // 0x83732: sub 0 tile rewrite and burst effect
                self.prop_b40_tiles(p.b.var26); // bsr 0x837AA
                let kind = p.m.b(0xA8); // 0x83736: tst.b $A8(a6)
                let var = if kind == 0 { 8 } else { 0x28 }; // 0x8373E / 0x83744
                self.prop_c21_spawn(p.b.xi(), p.b.yi(), p.b.zi(), var, kind); // 0x8374E: jsr 0x94640
                p.b.sub += 2; // 0x83754: addq.b #2,$5(a6)
            }
            2 => {
                // 0x8375A: sub 2 steam puff piece spawner
                let t = p.m.b(0xA7).wrapping_sub(1); // 0x8375A: subq.b #1,$A7(a6)
                p.m.sb(0xA7, t);
                if t == 0xFF {
                    // 0x8375E: bcc.b 0x83784 not taken (borrow from 0)
                    p.m.sb(0xA7, 5); // 0x83760: move.b #5,$A7(a6)
                    let kind = p.m.b(0xA8); // 0x83770: kind byte from table 0x83786
                    let pos = (p.b.xi(), p.b.yi(), p.b.zi());
                    self.enemy_spawn_piece(pos, 0, kind); // 0x83776: jsr 0x41EEA with d0=0
                    self.enemy_spawn_piece(pos, 2, kind); // 0x8377E: jmp 0x41EEA with d0=2
                }
            }
            _ => self.prop_free(p, i),
        }
    }

    /// 0x837AA: tile rewriting for break.
    fn prop_b40_tiles(&mut self, var: u16) {
        let t = &self.assets.enemy.t;
        let mut a0 = t.u32(0x8378C, u32::from(var));
        let step = (t.i16(0x837A0, u32::from(var / 2)) as i32) as u32;
        let mut src = 0x837E8;

        for count in [3, 4, 4] {
            for j in 0..count {
                self.prop_tile_code(TilePos::at(a0), t.u16(src, 0));
                src += 2;
                if j + 1 < count {
                    a0 = a0.wrapping_add(4);
                }
            }
            a0 = a0.wrapping_add(step);
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
    fn b40_init_kind0_and_kind1() {
        let Some(mut g) = rom_game() else { return };

        // Test var 0 (kind 0)
        let h0 = g.spawn_prop(40, 0, Some((1568, 0, 304))).expect("spawn b40 var 0");
        let i0 = (0..24).find(|&k| g.props.handle(k) == h0).unwrap();
        let mut p0 = std::mem::take(&mut g.props.slots[i0]);
        g.cam.s2.cam_x = 1000;
        g.prop_b40(&mut p0, h0, i0);
        assert_eq!(p0.b.visible, 1);
        assert_eq!(p0.b.body_box, 1);
        assert_eq!(p0.b.attack_box, 0);
        assert_eq!(p0.m.b(0xA7), 0);
        assert_eq!(p0.m.b(0xA8), 0);
        assert_eq!(p0.b.box_table, 0x8380A);
        assert_eq!(p0.b.hp, 8);
        assert_eq!(p0.b.hp_prev, 8);
        assert_eq!(p0.b.mode, 2);

        // Test var 8 (kind 1)
        let h8 = g.spawn_prop(40, 8, Some((176, 48, 832))).expect("spawn b40 var 8");
        let i8 = (0..24).find(|&k| g.props.handle(k) == h8).unwrap();
        let mut p8 = std::mem::take(&mut g.props.slots[i8]);
        g.cam.s2.cam_x = 0;
        g.prop_b40(&mut p8, h8, i8);
        assert_eq!(p8.m.b(0xA8), 1);
        assert_eq!(p8.b.box_table, 0x83822);
        assert_eq!(p8.b.hp, 8);
        assert_eq!(p8.b.mode, 2);
    }

    #[test]
    fn b40_camera_despawn() {
        let Some(mut g) = rom_game() else { return };
        let h = g.spawn_prop(40, 0, Some((1000, 0, 304))).expect("spawn b40");
        let i = (0..24).find(|&k| g.props.handle(k) == h).unwrap();
        let mut p = std::mem::take(&mut g.props.slots[i]);

        // When cam_x + 0x20 >= 1000 (e.g. cam_x = 980), frees itself
        g.cam.s2.cam_x = 980;
        g.prop_b40(&mut p, h, i);
        assert_eq!(p.b.live, 0);
    }

    #[test]
    fn b40_tile_patch_a2() {
        let Some(mut g) = rom_game() else { return };

        // Check A2: exactly 11 tile code words change for var 0
        // (3 at 0x90E8A0, 4 at 0x90E8DC, 4 at 0x90E91C); attribute words stay.
        let target_addrs = [
            0x90E8A0, 0x90E8A4, 0x90E8A8,
            0x90E8DC, 0x90E8E0, 0x90E8E4, 0x90E8E8,
            0x90E91C, 0x90E920, 0x90E924, 0x90E928,
        ];

        // Seed initial cells with known code and attribute
        for (idx, &addr) in target_addrs.iter().enumerate() {
            g.prop_tile_put(TilePos::at(addr), 0x1111, 0x4000 | (idx as u16));
        }

        g.prop_b40_tiles(0);

        // Verify the 11 tile codes changed, attributes stayed intact
        for (idx, &addr) in target_addrs.iter().enumerate() {
            let cell = g.vram.scroll2[((addr - 0x90C000) / 4) as usize];
            assert_ne!(cell.code, 0x1111, "code at {addr:#x} must change");
            assert_eq!(cell.attr, 0x4000 | (idx as u16), "attr at {addr:#x} must be preserved");
        }

        // Verify other vars also rewrite 11 words without panic
        for &var in &[4, 8, 0xC, 0x10] {
            g.prop_b40_tiles(var);
        }
    }

    #[test]
    fn b40_break_spawn_c21_and_steam_a3() {
        let Some(mut g) = rom_game() else { return };
        let h = g.spawn_prop(40, 0, Some((1568, 0, 304))).expect("spawn b40");
        let i = (0..24).find(|&k| g.props.handle(k) == h).unwrap();

        // Mode 0 init -> mode 2
        let mut p = std::mem::take(&mut g.props.slots[i]);
        g.cam.s2.cam_x = 1000;
        g.prop_b40(&mut p, h, i);
        assert_eq!(p.b.mode, 2);
        assert_eq!(p.b.state(), 0x0200_0000);

        // Hit sets mode = 4, sub = 0
        p.b.set_state(0x0400_0000);
        g.prop_b40(&mut p, h, i);
        assert_eq!(p.b.state(), 0x0402_0000);
        // Verify C21 effect was spawned
        let live_effects = g.effects.slots.iter().filter(|s| s.b.live != 0).count();
        assert_eq!(live_effects, 1);

        // Sub 2 spawns class 15 steam pieces every 6 frames
        // Frame 1 of sub 2: counter was 0, borrows to 0xFF -> reloads with 5 and spawns pair (2 pieces)
        g.prop_b40(&mut p, h, i);
        assert_eq!(p.m.b(0xA7), 5);
        let live_enemies = g.enemies.slots.iter().filter(|s| s.b.live != 0).count();
        assert_eq!(live_enemies, 2);

        // Frames 2..6: counter decrements 4, 3, 2, 1, 0 without spawning
        for c in (0..5).rev() {
            g.prop_b40(&mut p, h, i);
            assert_eq!(p.m.b(0xA7), c);
            let live = g.enemies.slots.iter().filter(|s| s.b.live != 0).count();
            assert_eq!(live, 2);
        }

        // Frame 7: borrows from 0, reloads with 5 and spawns another pair (4 pieces total)
        g.prop_b40(&mut p, h, i);
        assert_eq!(p.m.b(0xA7), 5);
        let live = g.enemies.slots.iter().filter(|s| s.b.live != 0).count();
        assert_eq!(live, 4);
    }

    #[test]
    fn b40_reach_a1() {
        let Some(mut g) = rom_game() else { return };

        // Test D3e A1: punch at X 0x5F0 against prop at X 0x620, Z 304.
        // Hits for Z in 296, 304, 312, 320; misses for 272..288 and 328..344.
        for z in (272..=344).step_by(8) {
            let h = g.spawn_prop(40, 0, Some((0x620, 0, 304))).expect("spawn b40");
            let pi = (0..24).find(|&k| g.props.handle(k) == h).unwrap();
            let mut p = std::mem::take(&mut g.props.slots[pi]);
            g.cam.s2.cam_x = 0x4A0;
            g.prop_b40(&mut p, h, pi);
            g.props.slots[pi] = p;

            // Setup Jack with jab attack box at X 0x5F0, Z z, facing right
            let pl = &mut g.players[0];
            pl.b.live = 1;
            pl.b.k20 = 0; // Jack
            pl.b.kind = 4; // attacking frame class
            pl.b.set_xi(0x5F0);
            pl.b.set_yi(0);
            pl.b.set_zi(z);
            pl.b.dir = 0;
            pl.b.facing = 1; // facing right mirrors box
            pl.b.serial = 1;
            pl.b.box_table = g.assets.sprites.player_box_tables[0];
            pl.b.attack_box = 2; // jab attack box
            pl.b.atk = 0; // jab attack record

            g.combat_lists_reset();
            g.prop_register_hit(h);
            g.combat.push(0x6718, Game::player_handle(0));
            g.pass_10cd0();

            let prop_state = g.props.slots[pi].b.state();
            if (296..=320).contains(&z) {
                assert_eq!(prop_state, 0x0400_0000, "Z={z} should hit and set mode 4");
            } else {
                assert_eq!(prop_state, 0x0200_0000, "Z={z} should miss and remain mode 2");
            }

            let mut p_clean = std::mem::take(&mut g.props.slots[pi]);
            g.prop_free(&mut p_clean, pi);
        }
    }

    #[test]
    fn b40_reach_a23_jump_kick() {
        let Some(mut g) = rom_game() else { return };

        // Test D3e part 3 2.6 and A23: var 0x10 at X 0x6E0 (1760), Y 0x20 (32), Z 0x340 (832).
        // Hero at X 1728, Z 832. Standing jab (Y=0) misses vertically; jump kick (in air, Y=30) hits!
        let h = g.spawn_prop(40, 0x10, Some((1760, 32, 832))).expect("spawn b40 var 0x10");
        let pi = (0..24).find(|&k| g.props.handle(k) == h).unwrap();
        let mut p = std::mem::take(&mut g.props.slots[pi]);
        g.cam.s2.cam_x = 1500;
        g.prop_b40(&mut p, h, pi);
        g.props.slots[pi] = p;

        // 1. Standing jab at Y=0 misses vertically (height 29..71 vs pipe height 88..128)
        {
            let pl = &mut g.players[0];
            pl.b.live = 1;
            pl.b.k20 = 0;
            pl.b.kind = 4;
            pl.b.set_xi(1728);
            pl.b.set_yi(0);
            pl.b.set_zi(832);
            pl.b.facing = 1;
            pl.b.serial = 1;
            pl.b.box_table = g.assets.sprites.player_box_tables[0];
            pl.b.attack_box = 2; // jab box
            pl.b.atk = 0;

            g.combat_lists_reset();
            g.prop_register_hit(h);
            g.combat.push(0x6718, Game::player_handle(0));
            g.pass_10cd0();

            assert_eq!(g.props.slots[pi].b.state(), 0x0200_0000, "jab must miss high pipe");
        }

        // 2. Jump kick in air at Y=30 hits! (Jack jump attack box 10: yoff 12, ysize 31 -> 42..73 + Y=30 -> 72..103 overlaps 88..128)
        {
            let pl = &mut g.players[0];
            pl.b.live = 1;
            pl.b.k20 = 0;
            pl.b.kind = 6; // airborne attack class
            pl.b.set_xi(1728);
            pl.b.set_yi(30);
            pl.b.set_zi(832);
            pl.b.facing = 1;
            pl.b.serial = 2; // new swing
            pl.b.box_table = g.assets.sprites.player_box_tables[0];
            pl.b.attack_box = 16; // jump kick box
            pl.b.atk = 6; // jump attack record

            g.combat_lists_reset();
            g.prop_register_hit(h);
            g.combat.push(0x6718, Game::player_handle(0));
            g.pass_10cd0();

            assert_eq!(g.props.slots[pi].b.state(), 0x0400_0000, "jump kick must hit high pipe");
        }
    }
}
