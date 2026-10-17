//! Pool B class 0 (0x74FA2): breakable wall panels and garage fixtures (Episode 4).
//! Spec: D3e part 2 section 5, A210, W212, W213, W215.

use super::tile_block::{TileMode, TilePos};
use super::Prop;
use crate::game::{slot_of, Game, Handle};

impl Game {
    /// 0x74FA2: pool B class 0 update.
    pub fn prop_b0(&mut self, p: &mut Prop, h: Handle, i: usize) {
        match p.b.mode {
            0 => {
                // 0x74FB6: init
                p.b.mode = 2; // 0x74FB6: addq.b #2, $4(a6)
                p.b.visible = 1; // 0x74FBA: move.b #1, $1(a6)
                p.b.body_box = 1; // 0x74FC0: move.b #1, $48(a6)
                p.b.attack_box = 0; // 0x74FC6: move.b #0, $49(a6)
                let var = p.b.var26 as u32;
                let box_table = self.assets.enemy.t.u32(0x750B4, var * 4); // 0x74FD4
                p.b.box_table = box_table;
                p.b.hp = 8; // 0x74FE0
                p.b.hp_prev = 8; // 0x74FE6
                let dest = TilePos::scroll2_at_world(p.b.xi() as u16, p.b.yi() as u16); // 0x74FF4: bsr 0x75208
                p.m.sl(0xA0, dest.0); // 0x74FF8: move.l a0, $a0(a6)
                let s_base = 0x75018 + var * 3; // 0x75006
                let cols = self.assets.enemy.t.u8(s_base, 0); // 0x7500A: $a4(a6)
                let rows = self.assets.enemy.t.u8(s_base, 1); // 0x7500E: $a5(a6)
                let mode = self.assets.enemy.t.u8(s_base, 2); // 0x75012: $a6(a6)
                p.m.sb(0xA4, cols);
                p.m.sb(0xA5, rows);
                p.m.sb(0xA6, mode);
                // 0x75016: rts
            }
            2 => {
                // 0x75022: jmp 0x4BFA
                self.prop_register_hit(h);
            }
            4 => {
                // 0x75028: hit reaction / break
                let at = TilePos::at(p.m.l(0xA0));
                let cols = (p.m.b(0xA5) as u16) + 1; // outer loop: cols
                let rows = (p.m.b(0xA4) as u16) + 1; // inner loop: rows
                let patch = self.assets.enemy.t.u32(0x750F0, (p.b.var26 as u32) * 4);
                let mode = if p.m.b(0xA6) != 0 { TileMode::Pairs } else { TileMode::Codes };
                self.prop_tile_block(at, cols, rows, patch, mode); // 0x75046 / 0x7507A

                p.b.set_yi(0); // 0x75038: clr.w $c(a6)

                // 0x751E0: effects
                let facing = if p.b.attacker != 0 && slot_of(p.b.attacker).is_some() {
                    self.body(p.b.attacker).facing
                } else {
                    0
                };
                let fx_var = if p.b.var26 == 0 { 0x0E } else { 0x10 };
                if p.b.var26 == 0 {
                    self.prop_c24_spawn(p.b.xi(), p.b.yi(), p.b.zi(), fx_var, facing); // 0x751FC: jsr 0x959B4
                }
                self.prop_c21_spawn(p.b.xi(), p.b.yi(), p.b.zi(), fx_var, facing); // 0x75202: jmp 0x94640

                p.b.mode = 6; // 0x75040: addq.b #2, $4(a6)
            }
            _ => self.prop_free(p, i), // 0x750AE: jmp 0x4938
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
        let alt = Path::new("../dino.zip");
        let p = if path.exists() { path } else if alt.exists() { alt } else { return None };
        let assets: &'static AssetPack = Box::leak(Box::new(dino_extract::extract(p).expect("the ROM set loads")));
        Some(Game::new(assets))
    }

    #[test]
    fn b0_init_and_break_variants() {
        let Some(mut g) = rom_game() else { return };

        // Variant 0: column at (0x900, 0xE0, 0x60)
        let h0 = g.spawn_prop(0, 0, None).expect("spawn b0 var 0");
        let i0 = (0..24).find(|&k| g.props.handle(k) == h0).unwrap();
        let mut p0 = std::mem::take(&mut g.props.slots[i0]);
        p0.b.set_xi(0x900);
        p0.b.set_yi(0xE0);
        p0.b.set_zi(0x60);

        g.prop_b0(&mut p0, h0, i0);
        assert_eq!(p0.b.mode, 2);
        assert_eq!(p0.b.visible, 1);
        assert_eq!(p0.b.body_box, 1);
        assert_eq!(p0.b.attack_box, 0);
        assert_eq!(p0.b.hp, 8);
        assert_eq!(p0.b.box_table, 0x750C0);
        assert_eq!(p0.m.b(0xA4), 8); // 8 + 1 = 9 rows
        assert_eq!(p0.m.b(0xA5), 5); // 5 + 1 = 6 cols
        assert_eq!(p0.m.b(0xA6), 1); // TileMode::Pairs

        // Break variant 0
        p0.b.mode = 4;
        g.prop_b0(&mut p0, h0, i0);
        assert_eq!(p0.b.mode, 6);
        assert_eq!(p0.b.yi(), 0);

        // Variant 1: lamp at (0x980, 0xB0, 0x78)
        let h1 = g.spawn_prop(0, 1, None).expect("spawn b0 var 1");
        let i1 = (0..24).find(|&k| g.props.handle(k) == h1).unwrap();
        let mut p1 = std::mem::take(&mut g.props.slots[i1]);
        p1.b.set_xi(0x980);
        p1.b.set_yi(0xB0);
        p1.b.set_zi(0x78);

        g.prop_b0(&mut p1, h1, i1);
        assert_eq!(p1.b.mode, 2);
        assert_eq!(p1.b.box_table, 0x750D8);
        assert_eq!(p1.m.b(0xA4), 2); // 2 + 1 = 3 rows
        assert_eq!(p1.m.b(0xA5), 1); // 1 + 1 = 2 cols
        assert_eq!(p1.m.b(0xA6), 0); // TileMode::Codes

        // Break variant 1
        p1.b.mode = 4;
        g.prop_b0(&mut p1, h1, i1);
        assert_eq!(p1.b.mode, 6);
        assert_eq!(p1.b.yi(), 0);

        // Free in mode 6
        g.prop_b0(&mut p1, h1, i1);
        assert_eq!(p1.b.live, 0);
    }
}
