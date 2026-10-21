//! Pan setup of an effect (0x0251 and the eight mode routines, SPEC section 9.2) and the
//! twin of mode 7 (SPEC 9.4). The pan motion steps (9.3) come with M4-06.
//!
//! Request bytes (ring entry `iy`, SPEC 9.2): `iy+6` low 3 bits = mode, `iy+7` = pan `b`,
//! `iy+8` = spread `S`, `iy+9..10` = speed `v` (BE), `iy+11..12` = second speed (BE).

use crate::driver::Driver;
use crate::map::*;

impl<'a> Driver<'a> {
    /// 0x0251: set the pan motion of the SFX track `t` (voice F00D, priority F00C) from the
    /// request at `iy`. Sets +36 = mode + 1 first, then the mode routine (jump table 0x0269).
    pub(crate) fn pan_setup(&mut self, t: u16, iy: u16) {
        let mode = self.r8(iy + 6) & 7;
        self.w8(t + T_PAN_MODE, mode + 1);
        let b = self.r8(iy + 7);
        let s = self.r8(iy + 8);
        let v = self.r16be(iy + 9);
        let v2 = self.r16be(iy + 11);
        match mode {
            0 => self.pan_mode0(t, b),
            1 => self.pan_mode1(t, b, v),
            2 => self.pan_mode2(t, b, s, v),
            3 => self.pan_mode3(t, b, s, v),
            4 => self.pan_mode4(t, b, s, v),
            5 => {
                self.pan_mode3(t, b, s, v);
                self.set_speed2(t, v2);
            }
            6 => {
                self.pan_mode4(t, b, s, v);
                self.set_speed2(t, 0u16.wrapping_sub(v2));
            }
            _ => {
                self.pan_mode3(t, b, s, v);
                self.pan_twin(t, b, s, v);
            }
        }
    }

    /// 0x0279: no motion, pan = min(b, 0x20).
    fn pan_mode0(&mut self, t: u16, b: u8) {
        self.w8(t + T_PAN_MODE, 0);
        self.w8(t + T_PAN, b.min(0x20));
    }

    /// 0x028A: bounces between 0 and 0x20 for ever. Below 0x11 the speed is kept, from 0x11 it
    /// is negated and the pan is clamped to 0x20.
    fn pan_mode1(&mut self, t: u16, b: u8, v: u16) {
        let (pan, spd) = if b < 0x11 {
            (b, v)
        } else {
            (b.min(0x20), 0u16.wrapping_sub(v))
        };
        self.set_speed(t, spd);
        self.w8(t + T_PAN, pan);
        self.w8(t + T_PAN_FRAC, 0);
    }

    /// 0x02B4: swings between the pan and the pan +- S for ever, toward the centre first.
    /// The spread S is stored in +39.
    fn pan_mode2(&mut self, t: u16, b: u8, s: u8, v: u16) {
        let (pan, limit, spd) = if b < 0x11 {
            // 8-bit add, then clamp to 0x20.
            let a = b.wrapping_add(s);
            (b, if a < 0x21 { a } else { 0x20 }, v)
        } else {
            let pan = b.min(0x20);
            let limit = pan.saturating_sub(s);
            (pan, limit, 0u16.wrapping_sub(v))
        };
        self.w8(t + T_PAN_LIMIT, limit);
        self.w8(t + T_PAN, pan);
        self.w8(t + T_PAN_FRAC, 0);
        self.set_speed(t, spd);
        self.w8(t + T_SPREAD, s);
    }

    /// 0x02FD: sweeps up to the limit min(b + S, 0x20), then stops. The pan is not clamped.
    fn pan_mode3(&mut self, t: u16, b: u8, s: u8, v: u16) {
        let a = b.wrapping_add(s);
        let limit = if a < 0x21 { a } else { 0x20 };
        self.w8(t + T_PAN_LIMIT, limit);
        self.w8(t + T_PAN, b);
        self.w8(t + T_PAN_FRAC, 0);
        self.set_speed(t, v);
    }

    /// 0x0322: sweeps down to the limit max(pan - S, 0), then stops. Speed negated.
    fn pan_mode4(&mut self, t: u16, b: u8, s: u8, v: u16) {
        let pan = b.min(0x20);
        let limit = pan.saturating_sub(s);
        self.w8(t + T_PAN_LIMIT, limit);
        self.w8(t + T_PAN, pan);
        self.w8(t + T_PAN_FRAC, 0);
        self.set_speed(t, 0u16.wrapping_sub(v));
    }

    /// The twin of mode 7 (end of 0x03C7, SPEC 9.4): the whole track is copied to the next
    /// voice when the priority allows it (F00C >= its priority). The copy sweeps down from the
    /// pan to max(pan - S, 0) at -v, with a start delay of 4 steps.
    fn pan_twin(&mut self, t: u16, b: u8, s: u8, v: u16) {
        let voice = self.r8(VOICE);
        let next = if voice.wrapping_add(1) & 0x0F != 0 {
            t.wrapping_add(TRACK_SIZE)
        } else {
            t.wrapping_add(0xFC40)
        };
        let prio = self.r8(PRIO);
        if prio < self.r8(next + T_PRIO) {
            return;
        }
        for i in 0..TRACK_SIZE {
            let byte = self.r8(t + i);
            self.w8(next + i, byte);
        }
        let pan = b.min(0x20);
        let limit = pan.saturating_sub(s);
        self.w8(next + T_PAN_LIMIT, limit);
        self.w8(next + T_PAN, pan);
        self.w8(next + T_PAN_FRAC, 0);
        self.set_speed_at(next, 0u16.wrapping_sub(v));
        self.w8(next + T_DELAY, 4);
    }

    /// Pan speed (+3B lo, +3C hi) of track `t`.
    fn set_speed(&mut self, t: u16, v: u16) {
        self.set_speed_at(t, v);
    }

    fn set_speed_at(&mut self, t: u16, v: u16) {
        let [lo, hi] = v.to_le_bytes();
        self.w8(t + T_PAN_SPEED, lo);
        self.w8(t + T_PAN_SPEED + 1, hi);
    }

    /// Second pan speed (+3D lo, +3E hi) of track `t`.
    fn set_speed2(&mut self, t: u16, v: u16) {
        let [lo, hi] = v.to_le_bytes();
        self.w8(t + T_PAN_SPEED2, lo);
        self.w8(t + T_PAN_SPEED2 + 1, hi);
    }
}
