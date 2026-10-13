//! Pool B class 17 (`+$20 = 0x44`, handler 0x7B554): the frame of the character select (M2-04). Made by the select set-up (0xAE9C).
//!
//! Mode 0 (0x7B56A) loads the raw sprite list frame at 0x7B5C8 (17 tiles of the border of the screen, loader 0x16C6), puts the
//! object at X 0x180, height 0x58, depth 8 with the draw flags 0x50 (raw list), copies the palette at 0xBBA52 to the OBJ palette 10
//! (0x914140, the routine 0x7B5A6) and goes into the draw list; mode 2 (0x7B59A) only does the latter; modes 4 and 6 free it.

use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x7B554 pool B class 17 (jump table 0x7B562: mode 0 -> 0x7B56A, 2 -> 0x7B59A, 4 and 6 -> 0x7B5A0).
    pub fn prop_class17(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode += 2; // 0x7B56A
                let fx = &self.assets.effects;
                crate::game::effect::raw_load(&mut p.b, fx, fx.raw_at(0x7B5C8)); // movea.l 0x7B5C4(pc),a0; jsr 0x16C6
                p.b.visible = 1;
                p.b.draw_flags = 0x50;
                p.b.set_xi(0x180);
                p.b.set_zi(8);
                p.b.set_yi(0x58);
                // 0x7B5A6: the 16 words of 0xBBA52 into the OBJ palette at 0x914140 (or.w #0: the words as they are)
                for k in 0..16u32 {
                    self.pal.words[0xA0 + k as usize] = self.assets.frontend.text.u16(0xBBA52, 2 * k);
                }
                self.pal.dirty = true;
                self.prop_draw_list(p, i); // jmp 0x49C0
            }
            2 => self.prop_draw_list(p, i), // 0x7B59A
            _ => self.prop_free(p, i), // 0x7B5A0: jmp 0x4938
        }
    }
}
