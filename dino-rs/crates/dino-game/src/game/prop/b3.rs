//! Pool B class 3 (0x763F6): mine entrance solid wall (var 0) and exit wall (var 8).
//! Spec: D3e part 3 section 2.6, A01 to A04, W301.

use super::tile_block::TilePos;
use super::Prop;
use crate::game::{Game, Handle};

impl Game {
    /// 0x763F6: pool B class 3 update. The table by variant (0x7640C): var 0 -> 0x76418, var 4 and 8 -> 0x7667E.
    pub fn prop_b3(&mut self, p: &mut Prop, h: Handle, i: usize) {
        match p.b.var26 {
            4 | 8 => self.prop_b3_var8(p, i),
            _ => self.prop_b3_var0(p, h, i),
        }
    }

    /// 0x76418: var 0 (the mine entrance wall, broken by hits).
    fn prop_b3_var0(&mut self, p: &mut Prop, h: Handle, i: usize) {
        match p.b.mode {
            0 => {
                // 0x76420: init
                p.b.visible = 1; // move.b #1,$1(a6)
                p.b.body_box = 1;
                p.b.attack_box = 0;
                p.b.box_table = 0x76666;
                p.b.hp = 8;
                p.b.hp_prev = 8;
                p.b.mode += 2; // 0x76446, falls into 0x7644A
                self.prop_register_hit(h); // 0x76450: var 0, jmp 0x4BFA
            }
            2 => self.prop_register_hit(h), // 0x7644A / 0x76450
            4 => {
                // 0x7645A: the break (the hit pass set mode 4)
                self.snd_break(p.b.xi()); // jsr 0x9E60
                self.prop_b3_v0_tiles(p.b.var26); // 0x76460, 0x76464, 0x76468, 0x7646C
                self.prop_c21_spawn(p.b.xi(), p.b.yi(), p.b.zi(), 0x14, 1); // 0x76470 -> 0x76656
                p.b.mode += 2; // 0x76474
            }
            _ => self.prop_free(p, i), // 0x76660: jmp 0x4938
        }
    }

    /// Var 4 and 8 (table 0x7667E): var 4 breaks in its first frame, var 8 after 48 frames (the mine exit wall). The break runs
    /// in the frame the wait ends (0x766A0); mode 4 (0x766A4) is the entry for a break set from outside.
    fn prop_b3_var8(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode += 2; // 0x76686
                if p.b.var26 == 4 {
                    return self.prop_b3_v8_break(p); // 0x7668A: beq 0x766A0
                }
                p.m.sw(0xA0, 0x30); // 0x76692
                self.prop_b3_v8_wait(p); // falls into 0x76698
            }
            2 => self.prop_b3_v8_wait(p),
            4 => self.prop_b3_v8_actions(p),
            _ => self.prop_free(p, i), // 0x76660: jmp 0x4938
        }
    }

    /// 0x76698: `subq.w #1,$A0(a6)`, the break when it reaches 0.
    fn prop_b3_v8_wait(&mut self, p: &mut Prop) {
        let t = p.m.w(0xA0).wrapping_sub(1);
        p.m.sw(0xA0, t);
        if t == 0 {
            self.prop_b3_v8_break(p);
        }
    }

    /// 0x766A0: mode 4, then the break (0x766A4).
    fn prop_b3_v8_break(&mut self, p: &mut Prop) {
        p.b.mode += 2;
        self.prop_b3_v8_actions(p);
    }

    /// 0x766A4: tiles, attributes, the effect (pool C class 21 var 0x14, facing 1), mode 6.
    fn prop_b3_v8_actions(&mut self, p: &mut Prop) {
        self.prop_b3_v8_tiles(p.b.var26); // 0x766B6, 0x76732
        self.prop_c21_spawn(p.b.xi(), p.b.yi(), p.b.zi(), 0x14, 1); // 0x767CE
        p.b.mode += 2; // 0x766B0
    }

    /// 0x7647A to 0x765D0: tile codes and attributes for var 0 break.
    fn prop_b3_v0_tiles(&mut self, var: u16) {
        // 0x7647A: tile codes
        let base_addr = if var == 4 { 0x90F350 } else { 0x90FD50 };
        let t = &self.assets.enemy.t;
        let mut s = 0x76530;
        for c in 0..3 {
            let col_pos = TilePos::at(base_addr).right(c);
            for r in 0..9 {
                self.prop_tile_code(col_pos.down(r), t.u16(s, 0));
                s += 2;
            }
        }
        self.prop_tile_code(TilePos::at(0x90FD4C), t.u16(s, 0));
        s += 2;
        self.prop_tile_code(TilePos::at(0x90FD84), t.u16(s, 0));
        s += 2;
        self.prop_tile_code(TilePos::at(0x90FD88), t.u16(s, 0));
        s += 2;
        self.prop_tile_code(TilePos::at(0x90FD8C), t.u16(s, 0));
        s += 2;
        self.prop_tile_code(TilePos::at(0x90FDCC), t.u16(s, 0));

        // 0x764DA: attribute tweaks
        self.prop_tile_attr(TilePos::at(0x90FD6A), 0xFFE0, 0x1C);
        self.prop_tile_attr(TilePos::at(0x90FD6E), 0xFFE0, 0x1C);
        self.prop_tile_attr(TilePos::at(0x90FDAA), 0xFFE0, 0x1D);
        self.prop_tile_attr(TilePos::at(0x90FDAE), 0xFFE0, 0x1D);
        self.prop_tile_attr(TilePos::at(0x90FDB2), 0xFFE0, 0x1C);
        self.prop_tile_attr(TilePos::at(0x90FDB6), 0xFFE0, 0x1C);
        self.prop_tile_attr(TilePos::at(0x90FD4A), 0xFFFF, 0x09);
        self.prop_tile_attr(TilePos::at(0x90FD8A), 0xFFFF, 0x09);

        // 0x76576: 7 columns of 5 rows clearing wall bit / setting attribute class
        let attr_base = if var == 4 { 0x90F30A } else { 0x90FD2A };
        for c in 0..7 {
            let col_addr = 0x90F000 | (((attr_base + c * 0x40) & 0xFFFF) as u32);
            let col_pos = TilePos::at(col_addr);
            self.prop_tile_attr(col_pos, 0x03FF, 0x0800); // 0x7658E: row 0
            for r in 1..5 {
                self.prop_tile_attr(col_pos.down(r), 0x03FF, 0); // 0x76598, 0x765A0, 0x765A8, 0x765B0: rows 1..4
            }
        }

        // 0x765D0: priority bit 7 tweaks
        for pos in [
            0x90FD86, 0x90FD8A, 0x90FD8E,
            0x90FDC6, 0x90FDCA, 0x90FDCE, 0x90FDD2, 0x90FDD6, 0x90FDDA, 0x90FDDE,
            0x90FDE2, 0x90FDE6, 0x90FDEA, 0x90FDEE, 0x90FDF2, 0x90FDF6,
            0x90FDFA, 0x90FDBA,
        ] {
            self.prop_tile_attr(TilePos::at(pos), 0xFF7F, 0x0080);
        }
    }

    /// 0x766B6 to 0x76732: tile codes and attributes for var 8 break.
    fn prop_b3_v8_tiles(&mut self, var: u16) {
        // 0x766B6: tile codes
        let base_addr = if var == 4 { 0x90F010 } else { 0x90E010 };
        let t = &self.assets.enemy.t;
        let mut s = 0x766FA;
        for c in 0..3 {
            let col_pos = TilePos::at(base_addr).right(c);
            for r in 0..9 {
                self.prop_tile_code(col_pos.down(r), t.u16(s, 0));
                s += 2;
            }
        }

        // 0x76732: attributes
        if var == 4 {
            self.prop_tile_attr(TilePos::at(0x90F06A), 0xFFE0, 0x32);
            self.prop_tile_attr(TilePos::at(0x90F06E), 0xFFE0, 0x32);
            self.prop_tile_attr(TilePos::at(0x90F072), 0xFFE0, 0x26);
            self.prop_tile_attr(TilePos::at(0x90F076), 0xFFE0, 0x06);
            self.prop_tile_attr(TilePos::at(0x90F0AA), 0xFFE0, 0x26);
            self.prop_tile_attr(TilePos::at(0x90F0AE), 0xFFE0, 0x29);
            self.prop_tile_put(TilePos::at(0x90F400), 0x6CA3, 0x000C);
        } else {
            self.prop_tile_attr(TilePos::at(0x90E06A), 0xFFE0, 0x1D);
            self.prop_tile_attr(TilePos::at(0x90E06E), 0xFFE0, 0x1D);
            self.prop_tile_attr(TilePos::at(0x90E072), 0xFFE0, 0x1F);
            self.prop_tile_attr(TilePos::at(0x90E076), 0xFFE0, 0x1F);
            self.prop_tile_attr(TilePos::at(0x90E0AA), 0xFFE0, 0x1F);
            self.prop_tile_attr(TilePos::at(0x90E0AE), 0xFFE0, 0x1F);
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
    fn b3_var0_init_and_break() {
        let Some(mut g) = rom_game() else { return };
        let h = g.spawn_prop(3, 0, None).expect("spawn b3");
        let i = (0..24).find(|&k| g.props.handle(k) == h).unwrap();
        
        // Mode 0 initializes wall
        let mut p = std::mem::take(&mut g.props.slots[i]);
        g.prop_b3(&mut p, h, i);
        assert_eq!(p.b.live, 1);
        assert_eq!(p.b.body_box, 1);
        assert_eq!(p.b.attack_box, 0);
        assert_eq!(p.b.box_table, 0x76666);
        assert_eq!(p.b.hp, 8);
        assert_eq!(p.b.mode, 2);

        // Mode 4 breaks wall
        p.b.mode = 4;
        g.prop_b3(&mut p, h, i);
        assert_eq!(p.b.mode, 6);

        // Mode 6 frees wall
        g.prop_b3(&mut p, h, i);
        assert_eq!(p.b.live, 0);
    }

    #[test]
    fn b3_var8_countdown() {
        let Some(mut g) = rom_game() else { return };
        let h = g.spawn_prop(3, 8, None).expect("spawn b3 var 8");
        let i = (0..24).find(|&k| g.props.handle(k) == h).unwrap();

        let mut p = std::mem::take(&mut g.props.slots[i]);
        // Mode 0 initializes countdown
        g.prop_b3(&mut p, h, i);
        assert_eq!(p.b.mode, 2);
        assert_eq!(p.m.w(0xA0), 47); // 48 - 1 on frame 0 fall-through

        // Countdown 46 more frames
        for _ in 0..46 {
            g.prop_b3(&mut p, h, i);
            assert_eq!(p.b.mode, 2);
        }

        // Frame 48: the timer reaches 0 and the wall breaks in the same frame (0x7669C beq 0x766A0): mode 6
        g.prop_b3(&mut p, h, i);
        assert_eq!(p.b.mode, 6);

        // Mode 6 frees object
        g.prop_b3(&mut p, h, i);
        assert_eq!(p.b.live, 0);
    }
}
