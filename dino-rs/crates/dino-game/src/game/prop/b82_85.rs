//! Pool B classes of the ending (M3-23): the master of the ending scenes (82, handler 0x8AA0A) and the showers of debris
//! and quakes (83: 0x8B212, 84: 0x8B2CA, 85: 0x8B34C). The master is created by the ending routine 0xA69B8 (sub 2,
//! `flow/ending.rs`) and runs the scenes of the lab corridor (the explosions chase the heroes), the sunset outside and the
//! still picture; its last state makes pool B 81 (the desert, `prop/b81.rs`) and ends.
//!
//! # The master (class 82, `+$20 = 0x148`)
//!
//! Mode 0 (0x8AA1E): full brightness, mode 2, sub 0, `+$A0` = 0x50. Mode 2 runs the state `+$05` of the table 0x8AA46:
//!
//! | state | routine | what |
//! | --- | --- | --- |
//! | 0 | 0x8AB8E | palette set 3, full brightness, clear all (0xAE0), the four scroll2 pages 6, 7, 8, 6, `+$A0` = 0x1E |
//! | 2 | 0x8AA9A | the four explosion clouds (pool A 90, variants 0 to 0x300), the showers B83 and B84, the caption 0x69 (HURRY UP!!), `+$A0` = 0x1CC, the camera speed `+$A4` = 4 |
//! | 4 | 0x8ABF0 | 460 frames of the lab corridor: more showers, the captions 0x6A, the camera runs at `+$A4` pixels a frame and stops, the quake shakes every 16 frames at the end |
//! | 6, 0xA, 0xE, 0x12, 0x18 | 0x8AA62 | wait `+$A0` frames |
//! | 8 | 0x8ACE8 | outside: the pages 9 and 10 (scroll2), 0x65 and 0x66 (scroll3), two flying pieces (pool A 96) |
//! | 0xC | 0x8AD8C | the wall breaks (pool B 3, variant 4), the sound 0x56 |
//! | 0x10 | 0x8ADCE | clears scroll1, a puff of smoke (pool C 21, variant 0xE), the quakes B85 and a shower |
//! | 0x14 | 0x8AF50 | all pool A objects are ended, the pages 0xB, 0xC, 0x68, 0x69, the shower B84 (variant 0x10), the caption 0x6B |
//! | 0x16 | 0x8B000 | the still picture: the palettes of scroll2 cycle (0x8B058), at 0xC0 the staff roll text (pool B 87), then the layers are cleared |
//! | 0x1A | 0x8AA74 | layer control 0x12D6, the desert master (pool B 81) and the end of this object |
//!
//! `+$A0` the timer, `+$A4` the camera speed, `+$A8` the count of the palette cycle and `+$AA` its pointer into the table 0x8B112.

use super::Prop;
use crate::game::flow::ending::Cont;
use crate::game::vram::PaletteRam;
use crate::game::Game;

/// `+$20 / 4` of the classes this file makes.
pub const B3: u16 = 3;
pub const B81: u16 = 81;
pub const B83: u16 = 83;
pub const B84: u16 = 84;
pub const B85: u16 = 85;
pub const B87: u16 = 87;

mod f {
    pub const TIMER: usize = 0xA0;
    pub const SPEED: usize = 0xA4;
    pub const COUNT: usize = 0xA8;
    pub const PTR: usize = 0xAA;
    pub const SPAWNED: usize = 0xA6;
}

/// The offsets of the palette cycle (6 rows of 6 words, read column by column) and the palettes they point into.
const CYCLE: u32 = 0x8B112;
const PALETTES: u32 = 0xBCA52;
/// The positions of the pieces of the quake (X, then Y, 8 words each).
const QUAKE_X: u32 = 0x8B402;
const QUAKE_Y: u32 = 0x8B412;

impl Game {
    /// A pool B object of the class `class` (0x4860): live, class word and the variant. None when the pool is full.
    pub fn prop_spawn(&mut self, class: u16, var: u16) -> Option<usize> {
        let i = self.props.alloc()?;
        let p = &mut self.props.slots[i];
        p.b.live = 1;
        p.b.k20 = class * 4;
        p.b.var26 = var;
        Some(i)
    }

    /// A pool A object (0x483C) of the class `class` with the variant and the position words X and Z (Y 0). None when the pool is full.
    fn enemy_spawn_at(&mut self, class: u16, var: u16, x: i16, z: i16) -> Option<usize> {
        let i = self.enemies.alloc()?;
        let e = &mut self.enemies.slots[i];
        e.b.live = 1;
        e.b.k20 = class * 4;
        e.b.var26 = var;
        e.b.set_zi(z);
        e.b.set_xi(x);
        Some(i)
    }

    /// 0x94640: a pool C object of the class 21 (`+$20 = 0x54`) at the position of `b` with the variant `var` and the facing `facing`.
    /// The effect is made but this routine of the ending never updates pool C, so it stays as it is (state 0).
    pub fn fx_spawn_class21(&mut self, b: &crate::game::body::Body, var: u16, facing: u8) {
        let Some(j) = self.effects.alloc() else { return };
        let e = &mut self.effects.slots[j];
        e.b.live = 1;
        e.b.k20 = 0x54;
        e.b.set_xi(b.xi());
        e.b.set_yi(b.yi());
        e.b.set_zi(b.zi());
        e.b.var26 = var;
        e.b.facing = facing;
    }

    /// 0x9F12: the rumble of the showers and quakes: sound 0x66 with the pan of the object (`x`; B objects have 0) and the side
    /// bit of its facing (+$24), the second argument fixed.
    pub fn snd_rumble(&mut self, x: i16, facing: u8) {
        let side: u32 = if facing != 0 { 6 } else { 5 };
        let pan = self.snd_pan_x(x) as u32;
        self.snd_request(0x66, (side << 16) | 0x1200 | pan, 0x0156_1080);
    }

    /// 0x8AA0A pool B class 82 (jump table 0x8AA16: mode 0 -> 0x8AA1E, 2 -> 0x8AA38, 4 and 6 -> 0x8B16A free).
    pub fn prop_b82(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                self.brightness_full(); // 0x892C
                p.b.mode = 2;
                p.b.sub = 0;
                p.m.sw(f::TIMER, 0x50);
            }
            2 => self.b82_state(p, i),
            _ => self.prop_free(p, i),
        }
    }

    /// 0x8AA38: the state table 0x8AA46 (one entry per even `+$05`).
    fn b82_state(&mut self, p: &mut Prop, i: usize) {
        match p.b.sub {
            0x00 => self.b82_pictures(p, i),
            0x02 => self.b82_cast(p),
            0x04 => self.b82_corridor(p),
            0x08 => self.b82_outside(p),
            0x0C => self.b82_wall(p),
            0x10 => self.b82_smoke(p),
            0x14 => self.b82_end_of_fire(p),
            0x16 => self.b82_still(p, i),
            0x1A => self.b82_desert(p, i),
            0x06 | 0x0A | 0x0E | 0x12 | 0x18 => {
                // 0x8AA62: wait `+$A0` frames
                let t = p.m.w(f::TIMER).wrapping_sub(1);
                p.m.sw(f::TIMER, t);
                if t == 0 {
                    p.b.sub = p.b.sub.wrapping_add(2); // 0x8AA6C
                }
            }
            _ => {}
        }
    }

    /// 0x8AB8E: the palettes of the scene set 3, the clear and the corridor's scroll2 pages. The clear keeps the CPU through the video
    /// frame it starts in and the next, so the four pages (and the end of the state) come in the third frame (the recording, records 1729 to
    /// 1731 of `flow_end-jack`: the palettes and the clear first, the pages at the beginning of the frame of the next pass).
    fn b82_pictures(&mut self, p: &mut Prop, i: usize) {
        self.flow_scene_palettes(3); // moveq #3,d0; jsr 0x89E6
        self.brightness_full(); // 0x892C
        self.flow_clear_screen(); // 0xAE0
        self.flow.ending.cont = Some(Cont::Pages(self.props.handle(i)));
        let _ = p;
        self.flow_lag(1);
    }

    /// 0x8ABA6 on: the pages, the sub state and the timer of the state 0 (see `b82_pictures`).
    pub fn b82_pages(&mut self, h: crate::game::Handle) {
        self.flow_page2(0x90F000, 6);
        self.flow_page2(0x90F400, 7);
        self.flow_page2(0x90F800, 8);
        self.flow_page2(0x90FC00, 6);
        let b = self.body_mut(h);
        b.sub += 2;
        if let Some(crate::game::Slot::Prop(i)) = crate::game::slot_of(h) {
            self.props.slots[i].m.sw(f::TIMER, 0x1E);
        }
    }

    /// 0x8AA9A: the four clouds of the explosions (pool A 90), the showers and the first caption.
    fn b82_cast(&mut self, p: &mut Prop) {
        let _ = self.enemy_spawn_at(90, 0x0000, 0x20, 0x10);
        let _ = self.enemy_spawn_at(90, 0x0100, 0x10, 0x50);
        let _ = self.enemy_spawn_at(90, 0x0200, 0x00, 0x30);
        let _ = self.enemy_spawn_at(90, 0x0300, 0x00, 0x20);
        let _ = self.prop_spawn(B83, 0);
        let _ = self.prop_spawn(B84, 0);
        self.prop_caption_spawn(0x69); // moveq #$69,d0; jsr 0x86D2E
        p.b.sub += 2;
        p.m.sw(f::TIMER, 0x1CC);
        p.m.sw(f::SPEED, 4);
    }

    /// 0x8ABF0: the corridor scene. `+$A0` counts down from 0x1CC.
    fn b82_corridor(&mut self, p: &mut Prop) {
        let t = p.m.w(f::TIMER).wrapping_sub(1);
        p.m.sw(f::TIMER, t);
        if t == 0 {
            p.b.sub += 2; // 0x8ACDA
            p.m.sw(f::TIMER, 1);
            return;
        }
        if t == 0x12C {
            let _ = self.prop_spawn(B83, 0);
        }
        if t == 0xD2 {
            p.m.sw(f::SPEED, 2);
        }
        if t == 0xCD {
            p.m.sw(f::SPEED, 0);
            self.prop_caption_spawn(0x6A);
        }
        if t == 0xC8 {
            p.m.sw(f::SPEED, 0);
            // the second allocation is tried only when the first one worked (bcs 0x8AC8E)
            if self.prop_spawn(B83, 0x600).is_some() {
                let _ = self.prop_spawn(B83, 0x700);
            }
        }
        self.cam.s2.cam_x = self.cam.s2.cam_x.wrapping_add(p.m.w(f::SPEED)); // add.w $A4(a6),$744(a5)
        if t == 0xDC {
            let _ = self.prop_spawn(B83, 0x100);
        }
        if (t as i16) <= 0xC8 && t & 0xF == 0 {
            self.cam_shake(4); // move.b #4,$755C(a5)
        }
    }

    /// 0x8ACE8: outside at sunset: the pages and the two pieces of the explosion.
    fn b82_outside(&mut self, p: &mut Prop) {
        self.cam.s2.cam_x = 0;
        self.flow_page2(0x90F000, 9);
        self.flow_page2(0x90F400, 0xA);
        self.flow_page3(0x913800, 0x65);
        self.flow_page3(0x913900, 0x66);
        let _ = self.enemy_spawn_at(96, 0x0000, 0x20, 0x10);
        let _ = self.enemy_spawn_at(96, 0x0100, 0x20, 0x10);
        p.b.sub += 2;
        p.m.sw(f::TIMER, 0x3C);
    }

    /// 0x8AD8C: the wall breaks (pool B 3, variant 4) and the sound 0x56. A full pool goes to 0x8AE56.
    fn b82_wall(&mut self, p: &mut Prop) {
        if self.prop_spawn(B3, 4).is_none() {
            p.b.sub += 2; // 0x8AE56
            p.m.sw(f::TIMER, 0x64);
            return;
        }
        p.b.sub += 2;
        p.m.sw(f::TIMER, 0x14);
        self.snd_request(0x56, 0x5_0020, 0x80_0000);
    }

    /// 0x8ADCE: the smoke and the quakes.
    fn b82_smoke(&mut self, p: &mut Prop) {
        p.b.set_xi(self.cam.s2.cam_x as i16);
        p.b.set_yi(self.cam.s2.cam_y as i16);
        p.b.set_zi(self.cam.s2.cam_z as i16);
        p.b.set_xi(p.b.xi().wrapping_add(0x40));
        p.b.set_zi(p.b.zi().wrapping_add(0x50));
        self.flow_clear_scroll1(); // 0xB52
        self.fx_spawn_class21(&p.b, 0xE, 1); // moveq #$E,d0; moveq #1,d1; jsr 0x94640
        let ok = self.prop_spawn(85, 0x100).is_some() && self.prop_spawn(85, 0x100).is_some() && self.prop_spawn(B83, 0x100).is_some();
        let _ = ok; // every `bcs` goes to 0x8AE56, the same tail
        p.b.sub += 2;
        p.m.sw(f::TIMER, 0x64);
    }

    /// 0x8AF50: everything of pool A ends, the next pictures, the shower and the caption.
    fn b82_end_of_fire(&mut self, p: &mut Prop) {
        self.w.layer_ctrl = 0x18D6; // move.w #$18D6,$28(a5)
        for e in self.enemies.slots.iter_mut() {
            if e.b.live != 0 {
                e.b.hp = 0; // move.w #0,$6C(a0)
                e.b.mode = 6;
            }
        }
        self.cam.s2.cam_x = 0;
        self.flow_page2(0x90F000, 0xB);
        self.flow_page2(0x90F400, 0xC);
        self.flow_page3(0x913800, 0x68);
        self.flow_page3(0x913900, 0x69);
        let _ = self.prop_spawn(B84, 0x10);
        self.prop_caption_spawn(0x6B);
        p.b.sub += 2;
        p.m.sw(f::TIMER, 0x13C);
        p.m.sw(f::COUNT, 5);
        p.m.sl(f::PTR, CYCLE);
    }

    /// 0x8B000: the still picture. The palettes of the sky cycle until the timer ends.
    fn b82_still(&mut self, p: &mut Prop, i: usize) {
        let t = p.m.w(f::TIMER).wrapping_sub(1);
        p.m.sw(f::TIMER, t);
        if t == 0 {
            // 0x8B032: the two clears (0xB6C, 0xB88) keep the 68000 past the end of the video frame: scroll2 and the first 3584 cells of scroll3
            // are cleared when it ends, the rest of scroll3 and the end of the state come at the start of the next one (the recording)
            self.vram.clear2();
            for c in self.vram.scroll3.iter_mut().take(3584) {
                (c.code, c.attr) = (0x1C00, 0);
            }
            self.flow.ending.cont = Some(Cont::StillEnd(self.props.handle(i)));
            return;
        }
        if t == 0xC0 {
            let _ = self.prop_spawn(B87, 0);
        }
        self.b82_cycle_palettes(p);
    }

    /// 0x8B038 on: the rest of the clear, the cameras, the sub state and the timer (see `b82_still`).
    pub fn b82_still_end_b(&mut self, h: crate::game::Handle) {
        self.vram.clear3();
        self.cam.s2.cam_x = 0;
        self.cam.s3.cam_x = 0;
        if let Some(crate::game::Slot::Prop(i)) = crate::game::slot_of(h) {
            self.props.slots[i].b.sub += 2;
            self.props.slots[i].m.sw(f::TIMER, 6);
        }
    }

    /// 0x8B058: six palettes of the scroll2 bank (20 to 25) take the colours of the table row of the step, keeping the brightness nibble.
    fn b82_cycle_palettes(&mut self, p: &mut Prop) {
        let a3 = p.m.l(f::PTR);
        for r in 0..6u32 {
            let off = self.assets.frontend.text.u16(a3 + 12 * r, 0) as u32;
            let dst = PaletteRam::SCROLL2 + 16 * (20 + r as usize);
            for k in 0..16u32 {
                let c = self.assets.frontend.text.u16(PALETTES + off + 2 * k, 0);
                let w = &mut self.pal.words[dst + k as usize];
                *w = (*w & 0xF000) | c;
            }
        }
        self.pal.dirty = true;
        p.m.sl(f::PTR, a3 + 2);
        let n = p.m.w(f::COUNT).wrapping_sub(1);
        p.m.sw(f::COUNT, n);
        if n == 0 {
            p.m.sw(f::COUNT, 5);
            p.m.sl(f::PTR, CYCLE);
        }
    }

    /// 0x8AA74: the layer control, the desert master and the end of this object.
    fn b82_desert(&mut self, p: &mut Prop, i: usize) {
        self.w.layer_ctrl = 0x12D6;
        let _ = self.prop_spawn(B81, 0);
        self.prop_free(p, i); // 0x8B16A: jmp 0x4938
    }

    // ------------------------------------------------------------------ the showers and the quake

    /// 0x8B212 pool B class 83 (jump table 0x8B21E: mode 0 -> 0x8B226, 2 -> 0x8B260, 4 and 6 -> 0x8B2C4): a shower of debris, one
    /// piece (pool A 92, with the variant of the shower) every 4 frames and the rumble every 16. The variant gives the time: 200 frames
    /// (variant 0, 0x700), 50 (0x100), 180 (0x600).
    pub fn prop_b83(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode = 2;
                let var = p.b.var26;
                p.m.sw(f::TIMER, 0xC8);
                if var != 0 {
                    p.m.sw(f::TIMER, 0x32);
                }
                if var == 0x600 {
                    p.m.sw(f::TIMER, 0xB4);
                }
                if var == 0x700 {
                    p.m.sw(f::TIMER, 0xC8);
                }
                self.b83_frame(p);
            }
            2 => self.b83_frame(p),
            _ => self.prop_free(p, i),
        }
    }

    /// 0x8B260.
    fn b83_frame(&mut self, p: &mut Prop) {
        let t = p.m.w(f::TIMER).wrapping_sub(1);
        p.m.sw(f::TIMER, t);
        if t == 0 {
            p.b.mode += 2; // 0x8B2B8
            return;
        }
        if t & 3 == 0 {
            if let Some(j) = self.enemies.alloc() {
                let e = &mut self.enemies.slots[j];
                e.b.live = 1;
                e.b.k20 = 92 * 4;
                e.b.var26 = p.b.var26;
            }
        }
        if t & 0xF == 0 {
            self.snd_rumble(p.b.xi(), p.b.facing); // jsr 0x9F12
        }
    }

    /// 0x8B2CA pool B class 84 (jump table 0x8B2D6: mode 0 -> 0x8B2DE, 2 -> 0x8B2FA, 4 and 6 -> 0x8B346): a shower of falling pieces
    /// (pool A 93) every 32 frames for 400 (the variant 0x10: every 8 frames for 100).
    pub fn prop_b84(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode = 2;
                p.m.sw(f::TIMER, 0x190);
                if p.b.var26 == 0x10 {
                    p.m.sw(f::TIMER, 0x64);
                }
                self.b84_frame(p);
            }
            2 => self.b84_frame(p),
            _ => self.prop_free(p, i),
        }
    }

    /// 0x8B2FA.
    fn b84_frame(&mut self, p: &mut Prop) {
        let t = p.m.w(f::TIMER).wrapping_sub(1);
        p.m.sw(f::TIMER, t);
        if t == 0 {
            p.b.mode += 2; // 0x8B33A
            return;
        }
        let spawn = if p.b.var26 == 0x10 && t & 7 == 0 { true } else { t & 0x1F == 0 };
        if spawn {
            if let Some(j) = self.enemies.alloc() {
                let e = &mut self.enemies.slots[j];
                e.b.live = 1;
                e.b.k20 = 93 * 4; // move.w #$174,$20(a0); the variant stays 0
            }
        }
    }

    /// 0x8B34C pool B class 85 (jump table 0x8B358: mode 0 -> 0x8B360, 2 -> 0x8B37A, 4 and 6 -> 0x8B422): the quake: for 100 frames
    /// the camera shakes and the rumble sounds every 32 frames, and every 4 frames a piece (pool A 92, variant 0x500) is made at the
    /// next of 8 positions of the table 0x8B402.
    pub fn prop_b85(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode = 2;
                p.m.sw(f::TIMER, 0x64);
                p.m.sw(f::SPAWNED, 0);
                p.m.sl(0xA2, 0);
                self.b85_frame(p, i);
            }
            2 => self.b85_frame(p, i),
            _ => self.prop_free(p, i),
        }
    }

    /// 0x8B37A.
    fn b85_frame(&mut self, p: &mut Prop, i: usize) {
        let t = p.m.w(f::TIMER).wrapping_sub(1);
        p.m.sw(f::TIMER, t);
        if t == 0 {
            p.b.mode += 2; // 0x8B3F8
            return self.prop_free(p, i); // bra 0x8B422
        }
        if t & 0x1F == 0 {
            self.cam_shake(4);
            self.snd_rumble(p.b.xi(), p.b.facing);
        }
        if t & 3 != 0 {
            return;
        }
        let n = p.m.w(f::SPAWNED).wrapping_add(1);
        p.m.sw(f::SPAWNED, n);
        let Some(j) = self.enemies.alloc() else { return };
        let k = ((n & 7) * 2) as u32;
        let tx = self.assets.frontend.text.u16(QUAKE_X + k, 0);
        let far = p.m.l(0xA2);
        // d0 is the long +$A2 (0 here): the word at 0x8B412 + d0.w is the first one
        let ty = self.assets.frontend.text.u16(QUAKE_Y + (far as u16 as u32), 0);
        let r = (self.rand() & 0xF0) as i16;
        let e = &mut self.enemies.slots[j];
        e.b.live = 1;
        e.b.k20 = 92 * 4;
        e.b.var26 = 0x500;
        e.b.set_xi(tx as i16);
        e.b.x = e.b.x.wrapping_add(far as i32); // add.l d0,$8(a0)
        e.b.set_yi(ty as i16);
        e.b.set_yi(e.b.yi().wrapping_add(r));
        e.b.set_zi(0);
    }
}
