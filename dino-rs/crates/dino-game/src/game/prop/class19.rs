//! Pool B class 19 (`+$20 = 0x4C`, handler 0x7B7C6): the animated picture of the hero under a player's cursor on the 2 player layout of the
//! character select (G.$4CF set; M2-04). Made by the first step of the player's select record (0xB0A6) from the table 0xB096 (X 0xA0 or
//! 0x160, height 0x10, the variant is the player number * 4). The 2 player layout was never measured (MAME has no switch for it): this is
//! the routine as the code reads, checked by unit tests only.
//!
//! Mode 0 (0x7B7DA) keeps the pointer to the select record of the player (G.$62C + 0xC + 3 * variant) and starts the animation 0 of the
//! table of the hero position (0x7B85E, 4 tables). Mode 2 (0x7B7FA): when the record is picked (state 4) the object goes to mode 4 and plays
//! the animation 1; else when the cursor moved the animation 0 of the new hero's table starts; the animation ticks (0x121E) and the object
//! goes into the draw list with the on-screen test (0x49CA). Mode 4 (0x7B82C) only ticks and draws, mode 6 frees it.

use super::Prop;
use crate::game::Game;

/// The tables of this class.
mod rom {
    /// 0x7B85E: the animation table of each hero position (4 longs).
    pub const TABLES: u32 = 0x7B85E;
}

/// Offsets of this class.
mod f {
    /// the select record this picture follows (the index of the player), +$A4 the cursor position the animation was started for.
    pub const PLAYER: usize = 0xA0;
    pub const POS: usize = 0xA4;
}

impl Game {
    /// 0x7B7C6 pool B class 19 (jump table 0x7B7D2: mode 0 -> 0x7B7DA, 2 -> 0x7B7FA, 4 -> 0x7B82C, 6 -> 0x7B838).
    pub fn prop_class19(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode += 2; // 0x7B7DA
                p.m.sb(f::PLAYER, (p.b.var26 / 4).min(2) as u8);
                self.prop_c19_start(p); // bra 0x7B83E
            }
            2 => {
                let rec = self.flow.front.select.rec[p.m.b(f::PLAYER) as usize];
                if rec.state >= 4 {
                    p.b.mode += 2; // 0x7B820: picked, the picked animation
                    p.b.start_anim(&self.assets.sprites, 1);
                } else if rec.pos != p.m.b(f::POS) {
                    self.prop_c19_start(p); // 0x7B810
                }
                p.b.tick_anim(&self.assets.sprites); // jsr 0x121E
                self.prop_c19_draw(p, i); // jmp 0x49CA
            }
            4 => {
                p.b.tick_anim(&self.assets.sprites); // 0x7B82C
                self.prop_c19_draw(p, i);
            }
            _ => self.prop_free(p, i), // 0x7B838
        }
    }

    /// 0x7B83E: the animation 0 of the table of the hero position of the record.
    fn prop_c19_start(&mut self, p: &mut Prop) {
        let pos = self.flow.front.select.rec[p.m.b(f::PLAYER) as usize].pos;
        p.m.sb(f::POS, pos);
        p.b.anim_table = self.assets.frontend.text.u32(rom::TABLES, 4 * (pos & 3) as u32);
        p.b.start_anim(&self.assets.sprites, 0); // jmp 0x120E
    }

    /// 0x49CA: the on-screen flag (0x16FA) and the draw list (the object is out of its slot while it runs).
    fn prop_c19_draw(&mut self, p: &mut Prop, i: usize) {
        let (cx, cy) = if self.w.vertical_scene != 0 { (0, 0) } else { (self.cam.s2.cam_x, self.cam.s2.cam_y) };
        let dx = p.b.xi().wrapping_sub(cx as i16).wrapping_add(0x100) as u16;
        let dy = p.b.yi().wrapping_add(p.b.zi()).wrapping_sub(cy as i16).wrapping_add(0x40) as u16;
        p.b.visible = (dx <= 0x380 && dy <= 0x1C0) as u8;
        self.prop_draw_list(p, i);
    }
}
