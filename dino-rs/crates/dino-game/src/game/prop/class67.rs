//! Pool B class 67 (`+$20 = 0x10C`, handler 0x8881A): the golden man and the other pictures that the character select puts over the
//! group picture (M2-04). Two are made by the select set-up (0xAE70) with the variants 4 and 0xC (Y 0x50, depth 0x10).
//!
//! Mode 0 (0x8882E) takes its X from the table 0x8885E (`var / 2`: 0x30, 0x90, 0xF0, 0x150) and its picture from the frame pointer
//! table 0x88872 (`var`, one looping frame each), sets the on-screen flag and the draw flags 0x10 and goes into the draw list; mode 2
//! (0x88866) only goes into the draw list; modes 4 and 6 free it (0x4938).

use super::Prop;
use crate::game::Game;

/// The tables of this class.
mod rom {
    pub const X: u32 = 0x8885E;
    pub const FRAMES: u32 = 0x88872;
}

impl Game {
    /// 0x8881A pool B class 67 (jump table 0x88826: mode 0 -> 0x8882E, 2 -> 0x88866, 4 and 6 -> 0x8886C).
    pub fn prop_class67(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode += 2; // 0x8882E
                let var = p.b.var26 as u32;
                p.b.set_xi(self.assets.frontend.text.u16(rom::X, var >> 1) as i16);
                let frame = self.assets.frontend.text.u32(rom::FRAMES, var); // movea.l (a0,d1.w),a0
                let id = self.assets.sprites.frame_at(frame);
                p.b.load_frame(&self.assets.sprites, id); // jsr 0x1236
                p.b.visible = 1;
                p.b.draw_flags = 0x10;
                self.prop_draw_list(p, i); // 0x88866: jmp 0x49C0
            }
            2 => self.prop_draw_list(p, i),
            _ => self.prop_free(p, i), // 0x8886C: jmp 0x4938
        }
    }

    /// 0x49C0: into the draw list when the on-screen flag is set (the object is out of its slot while it runs).
    pub(super) fn prop_draw_list(&mut self, p: &Prop, i: usize) {
        if p.b.visible != 0 {
            let h = self.props.handle(i);
            self.draw_list.push(((p.b.zi() as u16 as u32) << 16) | h as u32);
        }
    }
}
