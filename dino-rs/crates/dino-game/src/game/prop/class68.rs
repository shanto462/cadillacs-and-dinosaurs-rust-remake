//! Pool B class 68 (`+$20 = 0x110`, handler 0x888B2): the tag of a player on the group picture of the character select ("1P", "2P",
//! "3P", variants 0, 4 and 8 = the player number * 4) (M2-04). Made by the first step of the player's select record (0xB0A6).
//!
//! Mode 0 (0x888C6) puts the tag over the cursor position of the record (X from the table 0x88948 by position, 0x88922), picks the frame
//! from the table 0x88A54 by the variant (a looping frame, the sizes of the three tags are the same), sets the on-screen flag and the draw
//! flags 0x10, and makes ready the palette cycle: the list of palettes of this tag (0x8890A: the lists at 0x889C4, 0x889EE, 0x88A18 end
//! with a negative long) and the OBJ palette they go into (0x914280, 0x9142A0, 0x9142C0). It does not draw in that frame. Mode 2
//! (0x88950): every 3 frames the next palette of the list goes into the OBJ palette (keeping the brightness nibble; at the end of the list
//! it starts again and `+$B0` says so); when it started again and the record is picked (state 4 or more) the tag goes to mode 4 and keeps
//! its place (0x88A42: draws each frame forever), else it follows the cursor and draws. Mode 6 frees it.

use super::Prop;
use crate::game::Game;

/// The tables of this class.
mod rom {
    pub const LISTS: u32 = 0x8890A;
    pub const RECORD_OFFSETS: u32 = 0x88942;
    pub const X: u32 = 0x88948;
    pub const FRAMES: u32 = 0x88A54;
}

/// Offsets of this class.
mod f {
    /// long: the next palette of the list, +$A8 the start of the list, +$AC the OBJ palette address.
    pub const LIST: usize = 0xA0;
    pub const LIST_START: usize = 0xA8;
    pub const TARGET: usize = 0xAC;
    /// byte: frames to the next palette (3 frame cycle), +$B0 the list ran out this frame.
    pub const WAIT: usize = 0xA4;
    pub const WRAPPED: usize = 0xB0;
}

impl Game {
    /// 0x888B2 pool B class 68 (jump table 0x888BE: mode 0 -> 0x888C6, 2 -> 0x88950, 4 -> 0x88A42, 6 -> 0x88A4E).
    pub fn prop_class68(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode += 2; // 0x888C6
                self.prop_c68_place(p); // bsr 0x88922
                let var = p.b.var26 as u32;
                let frame = self.assets.frontend.text.u32(rom::FRAMES, var);
                let id = self.assets.sprites.frame_at(frame);
                p.b.load_frame(&self.assets.sprites, id); // jsr 0x1236
                p.b.visible = 1;
                p.b.draw_flags = 0x10;
                let t = &self.assets.frontend.text;
                let list = t.u32(rom::LISTS, 2 * var); // 0x888EC
                let target = t.u32(rom::LISTS, 2 * var + 4);
                p.m.sl(f::LIST, list);
                p.m.sl(f::LIST_START, list);
                p.m.sl(f::TARGET, target);
                p.m.sb(f::WAIT, 0);
            }
            2 => {
                p.m.sb(f::WRAPPED, 0); // 0x88950
                self.prop_c68_cycle(p); // bsr 0x88984
                if p.m.b(f::WRAPPED) != 0 {
                    let var = (p.b.var26 / 4) as usize;
                    if self.flow.front.select.rec[var.min(2)].state >= 4 {
                        p.b.mode = 4; // 0x88A42
                        self.prop_draw_list(p, i);
                        return;
                    }
                }
                self.prop_c68_place(p); // 0x8897C
                self.prop_draw_list(p, i);
            }
            4 => {
                p.b.mode = 4; // 0x88A42
                self.prop_draw_list(p, i);
            }
            _ => self.prop_free(p, i), // 0x88A4E
        }
    }

    /// 0x88922: the X of the tag from the cursor position of the player's record.
    fn prop_c68_place(&mut self, p: &mut Prop) {
        let t = &self.assets.frontend.text;
        let who = (p.b.var26 / 4) as usize;
        let pos = self.flow.front.select.rec[who.min(2)].pos;
        debug_assert!(t.u16(rom::RECORD_OFFSETS, p.b.var26 as u32 >> 1) as usize == 0xE + 0xC * who);
        p.b.set_xi(t.u16(rom::X, 2 * pos as u32) as i16);
    }

    /// 0x88984: every third frame the next palette of the list goes into the OBJ palette of the tag, keeping the brightness nibble.
    fn prop_c68_cycle(&mut self, p: &mut Prop) {
        let w = p.m.b(f::WAIT).wrapping_sub(1);
        p.m.sb(f::WAIT, w);
        if w != 0xFF {
            return; // the subtraction borrowed (carry set) only when the counter was 0
        }
        p.m.sb(f::WAIT, 2);
        let t = &self.assets.frontend.text;
        let mut list = p.m.l(f::LIST);
        let mut palette = t.u32(list, 0);
        list += 4;
        if (palette as i32) < 0 {
            // the end of the list: start again, the first palette
            p.m.sb(f::WRAPPED, 1);
            list = p.m.l(f::LIST_START);
            palette = t.u32(list, 0);
            list += 4;
        }
        p.m.sl(f::LIST, list);
        let first = ((p.m.l(f::TARGET) - 0x914000) / 2) as usize;
        for k in 0..16 {
            let src = self.assets.frontend.text.u16(palette, 2 * k as u32);
            let w = &mut self.pal.words[first + k];
            *w = (*w & 0xF000) | src;
        }
        self.pal.dirty = true;
    }
}
