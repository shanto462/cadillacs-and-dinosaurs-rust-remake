//! Pool B class 25 (`+$20 = 0x64`, handler 0x7E7B4): the number of a panel of the stage clear tally (`flow/tally.rs`, M2-24). The tally makes
//! one for each panel of each hero (0x7E72C, the variant is the sub step 0, 2, 4 or 6) and one for each total (0x7E772, the variant 8). The
//! object draws a sprite list that lives in a pool D slot (digit sprites 16 x 16, palette 6, the object's palette override 0x8B for the best hero
//! and 0x8C for the others) at the position the tally gives it (screen space, draw flags 0xD0).
//!
//! Variants 0, 2, 4 and 6 (`+$58` the block of the hero in the tally struct, `+$5C` his player object, `+$60` the value, `+$64` "leading zeros are
//! blank", `+$66` the first Y, `+$98` the pool D slot):
//!
//! | mode | what |
//! | --- | --- |
//! | 0 | 0x7E7D2: takes a slot (none: 0x7E93E), clears it and builds the list (below), visible, draw flags 0xD0, mode 2 |
//! | 2 | 0x7E84E: waits (drawing the list) until the tally's sub step is the variant + 2, then mode 4 and the same frame goes on |
//! | 4 | 0x7E864: sub 0 (once) the second form of the number (below); sub 2 waits for the sub step 8 and sets the speed to the target Y of the block (`+$14` of the block, 0x50) in 16 frames; sub 4 moves and, past the target, mode 6 |
//! | 6 | 0x7E93E: when the tally counts down (G.$7370) the slot is given back and the object freed |
//!
//! The first form: VITALITY the figure HP - 1 (three digits and the four sprites of the suffix, the list of 7 sprites) or, with full HP, one big sprite and
//! a label of 7 columns of tall tiles at the panel (+0x1AC); ENEMIES the kills the same way; FINAL BLOW and M.V.P. the whole number 20000 or 10000 of
//! six digits, which is added to the total of the block at once. The second form (mode 4 sub 0) is the whole number for VITALITY ((HP - 1) * 100, or 20000
//! with the label cleared) and ENEMIES (kills * 100), added to the total of the block; the other two keep their list.
//!
//! The variant 8 (0x7EBA8): mode 0 takes a slot and shows the total of the block (six digits); mode 2 draws until the sub step 0xC, mode 4 rebuilds the
//! list from the total in every frame (the count down) until the sub step 0xE, mode 6 frees it as above.

use super::Prop;
use crate::game::draw::slot_frame;
use crate::game::effect::Special;
use crate::game::flow::tally::{blk, block};
use crate::game::hud::{bcd_add32, to_bcd};
use crate::game::{slot_of, Game, Slot};

/// Offsets of this class.
mod f {
    /// long: the block of the hero in the tally struct (a RAM address).
    pub const BLOCK: usize = 0x58;
    /// long: the player object.
    pub const PLAYER: usize = 0x5C;
    /// long: the value shown (BCD).
    pub const VALUE: usize = 0x60;
    /// byte: leading zeros are blank.
    pub const LEADING: usize = 0x64;
    /// word: the Y the object started at.
    pub const START_Y: usize = 0x66;
    /// word: the pool D slot.
    pub const SLOT: usize = 0x98;
}

/// ROM tables (`FRONT_TABLES`).
mod rom {
    /// The record of the digit list: pointer, parameters, X; then four suffix sprites.
    pub const TEMPLATE: u32 = 0x7EB56;
    /// The tile codes of the digits 0 to 9 and the blank (index 10).
    pub const DIGITS: u32 = 0x7EB80;
    /// The record of the big sprite (full vitality).
    pub const BIG: u32 = 0x7EB96;
    /// The 7 columns of the label.
    pub const LABEL: u32 = 0x7EB18;
}

/// The address of block `k` (G.$737E + 0x20 * k).
const fn block_addr(k: usize) -> u16 {
    0xF37E + 0x20 * k as u16
}

impl Game {
    /// 0x7E72C (the variant is G.$735F) and 0x7E772 (`var` = 8): the number object of hero `k`; `colour` is 0xB for the best panel, else 0xC.
    pub(crate) fn prop_tally_number(&mut self, k: usize, colour: u8, var: Option<u16>) {
        let Some(i) = self.props.alloc() else { return };
        let sub = self.flow.tally.raw.b(1) as u16;
        let (x, y) = (self.flow.tally.raw.w(block(k) + blk::X), self.flow.tally.raw.w(block(k) + blk::Y));
        let player = Game::player_handle(k);
        let p = &mut self.props.slots[i];
        p.b.live = 1;
        p.b.k20 = 0x64;
        p.b.palette_override = colour.wrapping_add(0x80);
        p.m.sl(f::BLOCK, Game::tally_ptr(block_addr(k)));
        p.m.sl(f::PLAYER, Game::tally_ptr(player));
        p.b.var26 = var.unwrap_or(sub);
        p.b.set_xi_keep(x as i16);
        p.m.sw(f::START_Y, y);
        p.b.set_yi_keep(y as i16);
        p.b.set_zi_keep(0);
    }

    /// 0x7E7B4 pool B class 25 (jump table 0x7E7CA: mode 0 -> 0x7E7D2, 2 -> 0x7E84E, 4 -> 0x7E864, 6 -> 0x7E93E).
    pub fn prop_class25(&mut self, p: &mut Prop, i: usize) {
        if p.b.var26 == 8 {
            return self.prop_c25_total(p, i); // beq 0x7EBA8
        }
        match p.b.mode {
            0 => self.prop_c25_init(p, i),
            2 => self.prop_c25_wait(p, i),
            4 => self.prop_c25_run(p, i),
            _ => self.prop_c25_free(p, i),
        }
    }

    /// The block index and the player of the object (`+$58` and `+$5C`).
    fn prop_c25_hero(p: &Prop) -> usize {
        ((p.m.w(f::BLOCK + 2).wrapping_sub(block_addr(0))) / 0x20) as usize
    }

    /// 0x7E93E: when the tally counts down (G.$7370) the slot goes back (0x498C) and the object is freed (0x4938); else nothing.
    fn prop_c25_free(&mut self, p: &mut Prop, i: usize) {
        if self.flow.tally.raw.b(0x12) != 0 {
            self.draw_special_free(&mut p.m);
            self.prop_free(p, i);
        }
    }

    /// 0x7E7D2.
    fn prop_c25_init(&mut self, p: &mut Prop, i: usize) {
        p.b.mode += 2;
        let Some(slot) = self.draw_special_alloc(&mut p.m) else { return self.prop_c25_free(p, i) };
        for k in 0..0x80 {
            self.specials.slots[slot].m.sb(k, 0); // 32 longs
        }
        let k = Self::prop_c25_hero(p);
        match p.b.var26 {
            0 => {
                let hp = self.players[k].b.hp;
                if hp == 0x64 {
                    self.prop_c25_full_vitality(p, slot, k); // 0x7EACE
                } else {
                    let v = (hp & 0xFF00) | (hp as u8).wrapping_sub(1) as u16; // subq.b #1,d0
                    self.prop_c25_small(p, slot, v);
                }
            }
            2 => {
                let kills = self.spawn.g.kills[k] as u16;
                self.prop_c25_small(p, slot, kills);
            }
            4 => self.prop_c25_value(p, slot, k, 0xC8), // 0x7E83E: 20000
            _ => self.prop_c25_value(p, slot, k, 0x64), // 0x7E848: 10000
        }
        // 0x7E802: the slot is the draw record (0x16C6)
        let sp = &self.specials.slots[slot];
        let (a, b, c, d) = sp.record_params();
        p.b.frame = slot_frame(slot);
        (p.b.param_a, p.b.param_b, p.b.kind, p.b.timer) = (a, b, c, d);
        p.b.visible = 1;
        p.b.draw_flags = 0xD0;
    }

    /// 0x16C6 on the slot again: the record parameters of the object are read once more.
    fn prop_c25_reload(&mut self, p: &mut Prop) {
        let Some(Slot::Special(slot)) = slot_of(p.m.w(f::SLOT)) else { return };
        let (a, b, c, d) = self.specials.slots[slot].record_params();
        p.b.frame = slot_frame(slot);
        (p.b.param_a, p.b.param_b, p.b.kind, p.b.timer) = (a, b, c, d);
    }

    /// 0x7E84E: draw until the tally's sub step is the variant + 2, then mode 4 and on in the same frame.
    fn prop_c25_wait(&mut self, p: &mut Prop, i: usize) {
        let sub = self.flow.tally.raw.b(1);
        if (p.b.var26 as u8).wrapping_add(2) != sub {
            return self.prop_draw_list(p, i); // jmp 0x49C0
        }
        p.b.mode += 2;
        self.prop_c25_run(p, i);
    }

    /// 0x7E864.
    fn prop_c25_run(&mut self, p: &mut Prop, i: usize) {
        let k = Self::prop_c25_hero(p);
        match p.b.sub {
            0 => {
                // 0x7E886: sub 2, the second form of the number
                p.b.sub += 2;
                if self.prop_c25_second(p, k) {
                    self.prop_c25_reload(p); // bcc: jmp 0x16C6
                } else {
                    self.prop_c25_fall(p, k); // bra 0x7E8A4
                }
            }
            2 => self.prop_c25_fall(p, k),
            _ => self.prop_c25_move(p, k),
        }
        self.prop_draw_list(p, i); // jmp 0x49C0
    }

    /// 0x7E8E4: the second form of the number of the variants 0 and 2 (the whole number, added to the total); true when the list changed (carry clear), false
    /// for the variants 4 and 6 (carry set).
    fn prop_c25_second(&mut self, p: &mut Prop, k: usize) -> bool {
        let Some(Slot::Special(slot)) = slot_of(p.m.w(f::SLOT)) else { return false };
        match p.b.var26 {
            0 => {
                let hp = self.players[k].b.hp;
                let v = if hp == 0x64 {
                    self.prop_tally_clear_label(self.flow.tally.raw.l(block(k) + blk::PTR)); // bsr 0x7EB34
                    0xC8
                } else {
                    (hp & 0xFF00) | (hp as u8).wrapping_sub(1) as u16
                };
                self.prop_c25_value(p, slot, k, v);
                true
            }
            2 => {
                let kills = self.spawn.g.kills[k] as u16;
                self.prop_c25_value(p, slot, k, kills);
                true
            }
            _ => false,
        }
    }

    /// 0x7E8A4 (sub 2): from the sub step 8 on the number starts to move to the Y of the block: the speed is the distance times 16 (8.8: 16 frames).
    fn prop_c25_fall(&mut self, p: &mut Prop, k: usize) {
        if self.flow.tally.raw.b(1) < 8 {
            return;
        }
        p.b.sub += 2;
        let target = self.flow.tally.raw.w(block(k) + blk::TARGET);
        let d = target.wrapping_sub(p.b.yi() as u16) << 4; // asl.w #4
        p.b.vy = d as i16;
        p.b.vx = 0;
        p.b.ax = 0;
        p.b.ay = 0;
        p.b.vz = 0;
        p.b.az = 0;
    }

    /// 0x7E8B8 (sub 4): the motion (0x189C); the object is done when it has passed the target Y.
    fn prop_c25_move(&mut self, p: &mut Prop, k: usize) {
        p.b.motion();
        let target = self.flow.tally.raw.w(block(k) + blk::TARGET);
        let start = p.m.w(f::START_Y);
        let y = p.b.yi() as u16;
        let done = if start < target {
            // bcs: the number started above the target and moves down: done when y is not below it
            (y as i16) >= (target as i16)
        } else {
            (y as i16) <= (target as i16)
        };
        if done {
            p.b.mode += 2;
        }
    }

    // ------------------------------------------------------------------------------------------------ the lists

    /// The record of a list in the slot: the list pointer (the slot + 0xA as a RAM address), the record parameters and the word of the template
    /// `tpl`, then the two words 1 and the number of sprites.
    fn prop_c25_head(&mut self, slot: usize, tpl: u32, sprites: u16) {
        let handle = self.specials.handle(slot);
        let t = &self.assets.frontend.text;
        let sp = &mut self.specials.slots[slot];
        sp.m.sl(0, 0xFFFF_0000 | handle.wrapping_add(0xA) as u32);
        sp.m.sl(4, t.u32(tpl, 4));
        sp.m.sw(8, t.u16(tpl, 8));
        sp.m.sw(0xA, 1);
        sp.m.sw(0xC, sprites);
        sp.b.live = sp.m.b(0);
    }

    /// The digits of a number: from the top nibble `top` of `d5` down, a sprite each (X from `x0` in steps of 16, Y 0xFFF0, attribute 6, the tile of the
    /// digit, a blank for a leading zero) at the offset `off` of the slot (0x7E9C0 and 0x7EA80). `+$64` = 1 means leading zeros are blank until a
    /// digit is found.
    fn prop_c25_digits(&mut self, p: &mut Prop, slot: usize, mut off: usize, d5: u32, top: u32, x0: u16) {
        let mut x = x0;
        for d4 in (0..=top).rev() {
            let mut d0 = (d5 >> (4 * d4)) & 0xF;
            if d0 != 0 {
                p.m.sb(f::LEADING, 0);
            } else if p.m.b(f::LEADING) != 0 && d4 != 0 {
                d0 = 0xA;
            }
            let tile = self.assets.frontend.text.u16(rom::DIGITS, 2 * d0);
            let sp: &mut Special = &mut self.specials.slots[slot];
            sp.m.sw(off, x);
            sp.m.sw(off + 2, 0xFFF0);
            sp.m.sw(off + 4, 6);
            sp.m.sw(off + 6, tile);
            off += 8;
            x = x.wrapping_add(0x10);
        }
    }

    /// 0x7E97C: a small number (`hex` = HP - 1 or the kills): three digits and the four sprites of the suffix, seven sprites.
    fn prop_c25_small(&mut self, p: &mut Prop, slot: usize, hex: u16) {
        p.m.sb(f::LEADING, 1);
        let d5 = to_bcd(hex) as u32; // jsr 0x11C4
        p.m.sl(f::VALUE, d5);
        self.prop_c25_head(slot, rom::TEMPLATE, 7);
        for w in 0..16 {
            // the 8 longs of the template from 0x7EB60: four sprites
            let v = self.assets.frontend.text.u16(rom::TEMPLATE, 10 + 2 * w);
            self.specials.slots[slot].m.sw(0xE + 2 * w as usize, v);
        }
        self.prop_c25_digits(p, slot, 0x2E, d5, 2, 0xFFB0);
    }

    /// 0x7E958: the whole number: `hex` as BCD times 100 (shifted one byte), added to the total of the block (the carry is lost), and the six digits
    /// (0x7EA56).
    fn prop_c25_value(&mut self, p: &mut Prop, slot: usize, k: usize, hex: u16) {
        let value = (to_bcd(hex) as u32) << 8; // jsr 0x11C4 / lsl.l #8,d0
        p.m.sl(f::VALUE, value);
        let b = block(k) + blk::TOTAL;
        let (total, _) = bcd_add32(self.flow.tally.raw.l(b), value);
        self.flow.tally.raw.sl(b, total);
        self.prop_c25_six(p, slot, value);
    }

    /// 0x7EA56: the list of six digits.
    fn prop_c25_six(&mut self, p: &mut Prop, slot: usize, d5: u32) {
        p.m.sb(f::LEADING, 1);
        self.prop_c25_head(slot, rom::TEMPLATE, 6);
        self.prop_c25_digits(p, slot, 0xE, d5, 5, 0xFFA0);
    }

    /// 0x7EACE: full vitality: a list of one big sprite and the label of seven columns at the panel.
    fn prop_c25_full_vitality(&mut self, p: &mut Prop, slot: usize, k: usize) {
        self.prop_c25_head(slot, rom::BIG, 1);
        for w in 0..4 {
            let v = self.assets.frontend.text.u16(rom::BIG, 10 + 2 * w);
            self.specials.slots[slot].m.sw(0xE + 2 * w as usize, v);
        }
        let a = self.flow.tally.raw.l(block(k) + blk::PTR).wrapping_add(0x1AC);
        self.tally_label(a, rom::LABEL, 7);
        let _ = p;
    }

    /// 0x7EB34: the label of full vitality is cleared (code 0x20, attribute 0).
    fn prop_tally_clear_label(&mut self, ptr: u32) {
        let mut a = ptr.wrapping_add(0x1AC);
        for _ in 0..7 {
            self.hud_put(a, 0x20, 0);
            self.hud_put(a.wrapping_add(4), 0x20, 0);
            a = a.wrapping_add(0x80);
        }
    }

    // ------------------------------------------------------------------------------------------------ the total (variant 8)

    /// 0x7EBA8 (jump table 0x7EBB4: mode 0 -> 0x7EBBC, 2 -> 0x7EBF0, 4 -> 0x7EC02, 6 -> 0x7E93E).
    fn prop_c25_total(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode += 2;
                let Some(slot) = self.draw_special_alloc(&mut p.m) else {
                    return self.prop_c25_total_run(p, i); // bcs 0x7EC02
                };
                let k = Self::prop_c25_hero(p);
                let total = self.flow.tally.raw.l(block(k) + blk::TOTAL);
                self.prop_c25_six(p, slot, total);
                let (a, b, c, d) = self.specials.slots[slot].record_params();
                p.b.frame = slot_frame(slot);
                (p.b.param_a, p.b.param_b, p.b.kind, p.b.timer) = (a, b, c, d);
                p.b.visible = 1;
                p.b.draw_flags = 0xD0;
            }
            2 => {
                if self.flow.tally.raw.b(1) >= 0xC {
                    p.b.mode += 2; // 0x7EBF8
                }
                self.prop_draw_list(p, i);
            }
            4 => self.prop_c25_total_run(p, i),
            _ => self.prop_c25_free(p, i),
        }
    }

    /// 0x7EC02: after the sub step 0xE the mode goes on; every frame the list is built again from the total.
    fn prop_c25_total_run(&mut self, p: &mut Prop, i: usize) {
        if self.flow.tally.raw.b(1) >= 0xE {
            p.b.mode += 2;
        }
        if let Some(Slot::Special(slot)) = slot_of(p.m.w(f::SLOT)) {
            let k = Self::prop_c25_hero(p);
            let total = self.flow.tally.raw.l(block(k) + blk::TOTAL);
            self.prop_c25_six(p, slot, total);
        }
        self.prop_draw_list(p, i);
    }
}
