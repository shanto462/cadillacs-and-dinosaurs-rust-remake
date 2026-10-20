//! Pool B class 30 (handler 0x811DE..0x817CE): the lab monitor of Episode 7 area 3 (script record before the Slisaur fight). It plays sound 0x23 and
//! runs a script of 9 steps (0x81268: the step routine `+$05` and its time `+$A0`): the black screen (scroll2 tiles at 0x90F2C8), a wait, the
//! doctor's face with the talk (three caption boxes B34, the mouth tiles at 0x90F350 by the sequences 0x8159C / 0x815E8 / 0x81616), the end of
//! the talk clearing G.$4FC (the Slisaurs start), static frames (two tile sets in turns), the black screen and the idle end. While the script is
//! past its first step the palette 0x914BA0 / 0x914940 cycles through the sources of the current set (0x81390, 0x813FA). D3e B30, D5 M8-16.

use super::Prop;
use crate::game::Game;

/// The tables of the handler (read at run time through `enemy.t`; dino-extract/src/boss.rs).
mod t {
    pub const SCRIPT: u32 = 0x81268;
    pub const BLACK: u32 = 0x81302;
    pub const FACE: u32 = 0x81488;
    pub const STATIC: u32 = 0x81724;
    pub const PAL_PERIOD: u32 = 0x813AE;
    pub const PAL_SETS: u32 = 0x813B4;
    pub const TALK_PICK: u32 = 0x8153A;
    pub const TALK_SEQ: u32 = 0x8153E;
    pub const MOUTH: u32 = 0x81636;
    pub const STATIC_STEPS: u32 = 0x816E8;
}

/// The monitor's corner in scroll2 (6 rows of 4 cells, the rows 0x40 apart).
const SCREEN: u32 = 0x90F2C8;

impl Game {
    /// 0x811DE (mode table 0x811EA): 0 the init 0x811F2 (falls into 2), 2 the run 0x8121C, 4 and 6 the free 0x817C8.
    pub(super) fn prop_b30(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode = p.b.mode.wrapping_add(2);
                p.m.sw(0xBA, 0);
                p.m.sw(0xA0, 0);
                p.m.sw(0xB2, 0);
                p.m.sw(0xBC, 0);
                p.m.sw(0xA2, 1);
                self.sound(0x23);
                self.prop_b30_palette_set(p, 0);
                self.prop_b30_run(p);
            }
            2 => self.prop_b30_run(p),
            _ => self.prop_free(p, i),
        }
    }

    /// 0x8121C: the next script step when `+$A0` is 0 (0x81244); the step routine of `+$05` (table 0x8123A); the palette cycle (0x813FA) unless
    /// the script is at its first step.
    fn prop_b30_run(&mut self, p: &mut Prop) {
        if p.m.w(0xA0) == 0 {
            let k = p.m.w(0xBA) as u32 * 8;
            let tb = &self.assets.enemy.t;
            p.b.sub = tb.u16(t::SCRIPT, k) as u8;
            p.m.sw(0xA0, tb.u16(t::SCRIPT, k + 2));
            p.b.act = 0;
            p.m.sw(0xBA, p.m.w(0xBA).wrapping_add(1));
        }
        match p.b.sub {
            0 => {
                // 0x812B0: wait `+$A0`, then the black screen and the palette set 0.
                if p.b.act == 0 {
                    p.b.act = 2;
                }
                let t = p.m.w(0xA0).wrapping_sub(1);
                p.m.sw(0xA0, t);
                if t == 0 {
                    self.prop_b30_tiles(t::BLACK);
                    self.prop_b30_attr(0x1D);
                    self.prop_b30_flip_on();
                    self.prop_b30_palette_set(p, 0);
                }
            }
            2 => p.m.sw(0xA0, p.m.w(0xA0).wrapping_sub(1)), // 0x813F4
            4 => self.prop_b30_talk(p),
            6 => self.prop_b30_static(p),
            _ => {} // 0x817C6: idle
        }
        if p.m.w(0xBA) != 1 {
            self.prop_b30_palette_cycle(p);
        }
    }

    /// 0x81432 by `+$06` (table 0x81440): 0 the face (tiles 0x81488, the flip bits off, palette field 0xA, palette set 2) and the first caption;
    /// 2 the mouth sequence of each caption (0x81552), after the last sequence state 4; 4 the end of the talk: `+$A0` 0 and G.$4FC 0.
    fn prop_b30_talk(&mut self, p: &mut Prop) {
        match p.b.act {
            0 => {
                self.prop_b30_tiles(t::FACE);
                self.prop_b30_flip_face();
                self.prop_b30_attr(0xA);
                self.prop_b30_palette_set(p, 2);
                self.prop_b30_caption(p);
                p.b.act = p.b.act.wrapping_add(2);
            }
            2 => loop {
                // 0x81552
                let t = p.m.w(0xB8).wrapping_sub(1);
                p.m.sw(0xB8, t);
                if t != 0 {
                    return;
                }
                let a1 = p.m.l(0xB4);
                let tb = &self.assets.enemy.t;
                let f = tb.u8(a1, 0);
                if (f as i8) >= 0 {
                    let src = tb.u32(t::MOUTH, (f.wrapping_sub(1) as u32) << 2);
                    let (c0, c1) = (tb.u16(src, 0), tb.u16(src, 2));
                    self.prop_b30_put(0x90F350, c0);
                    self.prop_b30_put(0x90F390, c1);
                    p.m.sw(0xB8, tb.u8(a1, 1) as u16);
                    p.m.sl(0xB4, a1.wrapping_add(2));
                    return;
                }
                // 0x8158E: the next caption and sequence; none left: state 4.
                self.prop_b30_caption(p);
                if (p.m.l(0xB4) as i32) < 0 {
                    p.b.act = p.b.act.wrapping_add(2);
                    return;
                }
            },
            _ => {
                p.m.sw(0xA0, 0);
                self.spawn.g.b4fc = 0;
            }
        }
    }

    /// 0x81516: the caption box B34 with the variant `+$B2` (0x81B24), its mouth sequence (0x8153A / 0x8153E) in `+$B4`, `+$B8` 0xA, `+$B2` + 1.
    fn prop_b30_caption(&mut self, p: &mut Prop) {
        let n = p.m.w(0xB2);
        self.prop_b34_spawn(n);
        let tb = &self.assets.enemy.t;
        let d1 = tb.u8(t::TALK_PICK, n as u32);
        p.m.sl(0xB4, tb.u32(t::TALK_SEQ, d1 as u32));
        p.m.sw(0xB8, 0xA);
        p.m.sw(0xB2, n.wrapping_add(1));
    }

    /// 0x81660 by `+$06` (table 0x8166E): 0 the static: palette set 4, the steps of 0x816E8 (`+$B4` frames per frame, `+$B2` frames); 2 the
    /// static frames: the two tile sets of 0x81724 in turns (0x816FC, the flip bits 0x81792, palette field 0xA); after the last step state 4
    /// (0x8178C: `+$A0` 0, the next script step).
    fn prop_b30_static(&mut self, p: &mut Prop) {
        if p.b.act == 0 {
            p.b.act = 2;
            p.m.sw(0xB8, 1);
            for k in [0xB2, 0xBE, 0xBC, 0xB4] {
                p.m.sw(k, 0);
            }
            self.prop_b30_palette_set(p, 4);
            self.prop_b30_static_step(p);
        }
        if p.b.act != 2 {
            p.m.sw(0xA0, 0); // 0x8178C: the next script step
            return;
        }
        loop {
            let t = p.m.w(0xB8).wrapping_sub(1);
            p.m.sw(0xB8, t);
            if t != 0 {
                return;
            }
            let set = self.assets.enemy.t.u32(t::STATIC, p.m.w(0xBC) as u32);
            self.prop_b30_tiles(set);
            self.prop_b30_flip_static();
            self.prop_b30_attr(0xA);
            p.m.sw(0xB8, p.m.w(0xB4));
            p.m.sw(0xBC, p.m.w(0xBC).wrapping_add(4) & 4);
            let n = p.m.w(0xB2).wrapping_sub(1);
            p.m.sw(0xB2, n);
            if n != 0 {
                return;
            }
            self.prop_b30_static_step(p);
            if (p.m.w(0xB2) as i16) < 0 {
                p.b.act = p.b.act.wrapping_add(2);
                return;
            }
        }
    }

    /// 0x816D2: the next (`+$B4`, `+$B2`) pair of 0x816E8 by `+$BE`.
    fn prop_b30_static_step(&self, p: &mut Prop) {
        let k = p.m.w(0xBE) as u32;
        let tb = &self.assets.enemy.t;
        p.m.sw(0xB4, tb.u16(t::STATIC_STEPS, k));
        p.m.sw(0xB2, tb.u16(t::STATIC_STEPS, k + 2));
        p.m.sw(0xBE, p.m.w(0xBE).wrapping_add(4));
    }

    /// A scroll2 tile code word at `addr` (a cell is 4 bytes from 0x90C000: code, attribute).
    fn prop_b30_put(&mut self, addr: u32, code: u16) {
        self.vram.scroll2[((addr - 0x90C000) / 4) as usize].code = code;
    }

    /// The attribute word of the cell at `addr` through `f`.
    fn prop_b30_attr_at(&mut self, addr: u32, f: impl Fn(u16) -> u16) {
        let c = &mut self.vram.scroll2[((addr - 0x90C000) / 4) as usize];
        c.attr = f(c.attr);
    }

    /// 0x812DE / 0x81464 / 0x816FC: 6 rows of 4 tile codes from `src` at the monitor.
    fn prop_b30_tiles(&mut self, src: u32) {
        let mut k = 0u32;
        for row in 0..6u32 {
            for col in 0..4u32 {
                let code = self.assets.enemy.t.u16(src, k);
                k += 2;
                self.prop_b30_put(SCREEN + row * 0x40 + col * 4, code);
            }
        }
    }

    /// 0x81332 / 0x814B8: the palette field (bits 0..4) of the monitor's 24 cells.
    fn prop_b30_attr(&mut self, pal: u16) {
        for row in 0..6u32 {
            for col in 0..4u32 {
                self.prop_b30_attr_at(SCREEN + row * 0x40 + col * 4, |a| (a & 0xFFE0) | pal);
            }
        }
    }

    /// 0x81362: the flip bits of the black screen (the first right cell 0x60, the five below 0x40).
    fn prop_b30_flip_on(&mut self) {
        self.prop_b30_attr_at(0x90F2D4, |a| a | 0x60);
        for k in 1..6u32 {
            self.prop_b30_attr_at(0x90F2D4 + k * 0x40, |a| a | 0x40);
        }
    }

    /// 0x814E8: the flip bits of the face off (the first right cell & 0xFF9F, the five below & 0xFFBF).
    fn prop_b30_flip_face(&mut self) {
        self.prop_b30_attr_at(0x90F2D4, |a| a & 0xFF9F);
        for k in 1..6u32 {
            self.prop_b30_attr_at(0x90F2D4 + k * 0x40, |a| a & 0xFFBF);
        }
    }

    /// 0x81792: the flip bits of the static (0x90F2D0 | 0x20, then 0x90F2D4 and the five below & 0xFFBF).
    fn prop_b30_flip_static(&mut self) {
        self.prop_b30_attr_at(0x90F2D0, |a| a | 0x20);
        self.prop_b30_attr_at(0x90F2D4, |a| a & 0xFFBF);
        for k in 1..6u32 {
            self.prop_b30_attr_at(0x90F2D4 + k * 0x40, |a| a & 0xFFBF);
        }
    }

    /// 0x81390: the palette set `d0` (0, 2, 4): its period `+$A4` (0x813AE), its source list `+$A6` / `+$AE` and its destination `+$AA`
    /// (0x813B4).
    fn prop_b30_palette_set(&self, p: &mut Prop, d0: u32) {
        let tb = &self.assets.enemy.t;
        p.m.sw(0xA4, tb.u16(t::PAL_PERIOD, d0));
        let list = tb.u32(t::PAL_SETS, d0 * 4);
        p.m.sl(0xA6, list);
        p.m.sl(0xAE, list);
        p.m.sl(0xAA, tb.u32(t::PAL_SETS, d0 * 4 + 4));
    }

    /// 0x813FA: every `+$A4` frames the next source of the list (back to its start at the end mark) into the 16 colours at `+$AA`, the
    /// brightness nibbles kept.
    fn prop_b30_palette_cycle(&mut self, p: &mut Prop) {
        let t = p.m.w(0xA2).wrapping_sub(1);
        p.m.sw(0xA2, t);
        if t != 0 {
            return;
        }
        p.m.sw(0xA2, p.m.w(0xA4));
        let tb = &self.assets.enemy.t;
        let mut a0 = p.m.l(0xA6);
        let mut src = tb.u32(a0, 0);
        a0 = a0.wrapping_add(4);
        if (src as i32) < 0 {
            a0 = p.m.l(0xAE);
            src = tb.u32(a0, 0);
            a0 = a0.wrapping_add(4);
        }
        p.m.sl(0xA6, a0);
        let dst = ((p.m.l(0xAA) - 0x914000) / 2) as usize;
        for n in 0..16usize {
            let w = self.assets.enemy.t.u16(src, 2 * n as u32);
            let k = dst + n;
            self.pal.words[k] = (self.pal.words[k] & 0xF000) | w;
        }
        self.pal.dirty = true;
    }
}
