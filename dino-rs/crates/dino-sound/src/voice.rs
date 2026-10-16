//! The no-event step of a track (SPEC sections 7, 9.3, 15): the pan motion (0x1007, table 0x101D,
//! handlers 0x1030 to 0x1157), the LFO and portamento (0x1196), the envelope (table 0x11FF,
//! handlers 0x120B to 0x128D) and the volume and pitch writes (0x12C2 to 0x13DB, with the pitch
//! clamp at 0x138F).
//!
//! Track fields are little endian (the Z80 `ld e,(ix+n); ld d,(ix+n+1)` pairs), except the
//! pitch, the portamento slide and the portamento target, which are big endian (note 0x0B81).
//! Every table is read from the sound data image with [`Driver::rom`]; no table bytes live here.

use crate::driver::Driver;
use crate::map::*;

/// 0x1453: base pitch register of note n (108 words, little endian).
const TAB_PITCH_BASE: u32 = 0x1453;
/// 0x152B: fraction rows, 12 rows of 256 bytes (row = note minus the first note of the octave).
const TAB_FRACTION: u32 = 0x152B;
/// 0x13DE: octave of note n (108 bytes).
const TAB_OCTAVE: u32 = 0x13DE;
/// 0x144A: first note of each octave (9 bytes).
const TAB_OCTAVE_FIRST: u32 = 0x144A;

/// Little-endian word of the data image (the Z80 `ld a,(hl); inc hl; ld h,(hl)` read).
fn rom16(d: &Driver, addr: u32) -> u16 {
    u16::from_le_bytes([d.rom(addr), d.rom(addr.wrapping_add(1))])
}

/// 0x29CB: HL = (HL * DE) >> 16, floor. It also leaves HL at F000 and DE at F002 (SPEC 13).
fn mul16(d: &mut Driver, a: u16, b: u16) -> u16 {
    d.w16(0xF000, a);
    d.w16(0xF002, b);
    ((a as u32 * b as u32) >> 16) as u16
}

/// 0x138F .. 0x13A1: the note index from `a = 0x3C - root + (H - 1)`. Too high (0x6C..0xEB)
/// gives 0, too low (0xEC..0xFF) gives 0x6B. This is the ROM behaviour, kept as it is (SPEC 15).
fn pitch_clamp(a: u8) -> u8 {
    if a < 0x6C {
        a
    } else if a.wrapping_sub(0x6C) & 0x80 != 0 {
        0x6B
    } else {
        0
    }
}

impl<'a> Driver<'a> {
    /// 0x1007: the no-event step of track `t` (SPEC 7). The pan motion runs first while the
    /// pan mode is set, then the LFO and portamento (0x1196), the envelope (0x11EE), and the
    /// volume and pitch writes (0x12C2) when the envelope handler reaches them.
    pub(crate) fn no_event_step(&mut self, t: u16) {
        let mode = self.r8(t.wrapping_add(T_PAN_MODE));
        if mode != 0 {
            // Table 0x101D index = +36 - 1; index 0 (pan mode 0) jumps straight to 0x1196.
            self.pan_step(t, mode - 1);
        }
        self.lfo_step(t);
        self.envelope_step(t);
    }

    /// 0x1030 .. 0x1157: one pan motion step. `index` = pan mode (1..7) from table 0x101D.
    /// The new pan goes to the track and to the pan register 0x80+v (0x118C), also when the
    /// pan did not change. A stop clears +36.
    fn pan_step(&mut self, t: u16, index: u8) {
        if index == 0 || index > 7 {
            return;
        }
        let pan = self.r8(t.wrapping_add(T_PAN));
        let frac = self.r8(t.wrapping_add(T_PAN_FRAC));
        let spread = self.r8(t.wrapping_add(T_SPREAD));
        let mut limit = self.r8(t.wrapping_add(T_PAN_LIMIT));
        let mut speed = self.r16(t.wrapping_add(T_PAN_SPEED));
        let speed2 = self.r16(t.wrapping_add(T_PAN_SPEED2));
        // hl = (pan << 8 | fraction) + speed; h is the new pan.
        let sum = u16::from_be_bytes([pan, frac]).wrapping_add(speed);
        let mut h = (sum >> 8) as u8;
        let neg = speed & 0x8000 != 0;
        let mut stop = false;
        match index {
            // 0x1030: outside 0..0x20 the pan bounces back and the speed is negated.
            1 => {
                if h >= 0x21 {
                    h = if neg { 0 } else { 0x20 };
                    speed = speed.wrapping_neg();
                }
            }
            // 0x105C: swing between the limit and the centre, the limit moves by the spread.
            2 => {
                let reset = if !neg { h >= limit } else { h <= limit || h >= 0x21 };
                if reset {
                    h = limit;
                    limit = if neg {
                        let a = limit.wrapping_add(spread);
                        if a < 0x21 { a } else { 0x20 }
                    } else {
                        limit.saturating_sub(spread)
                    };
                    speed = speed.wrapping_neg();
                }
            }
            // 0x10B4: sweep up to the limit, then stop.
            3 => {
                if h >= limit {
                    h = limit;
                    stop = true;
                }
            }
            // 0x10CE: sweep down to the limit, then stop.
            4 => {
                if h <= limit || h >= 0x21 {
                    h = limit;
                    stop = true;
                }
            }
            // 0x10F0: up to the limit at the speed, then up to 0x20 at the second speed.
            5 => {
                if h >= limit {
                    h = limit;
                    if limit == 0x20 {
                        stop = true;
                    } else {
                        speed = speed2;
                        limit = 0x20;
                    }
                }
            }
            // 0x1120: down to the limit, then down to 0 at the second speed.
            6 => {
                if h <= limit || h >= 0x21 {
                    h = limit;
                    if limit == 0 {
                        stop = true;
                    } else {
                        speed = speed2;
                        limit = 0;
                    }
                }
            }
            // 0x1157: sweep to the limit in the direction of the speed, then stop.
            _ => {
                if !neg {
                    if h >= limit {
                        h = limit;
                        stop = true;
                    }
                } else if h <= limit || h >= 0x21 {
                    h = limit;
                    stop = true;
                }
            }
        }
        if stop {
            self.w8(t.wrapping_add(T_PAN_MODE), 0);
        }
        self.w8(t.wrapping_add(T_PAN_LIMIT), limit);
        self.w16(t.wrapping_add(T_PAN_SPEED), speed);
        self.w8(t.wrapping_add(T_PAN), h);
        self.w8(t.wrapping_add(T_PAN_FRAC), sum as u8);
        let v = self.r8(VOICE);
        self.pan_write_dbg(v, h);
    }

    /// 0x1196 .. 0x11B4: the LFO phase (+1A) advances by the speed (+18). A carry out of the
    /// phase adds 0x40 to the LFO flags (+17), which toggles the triangle direction.
    fn lfo_step(&mut self, t: u16) {
        let speed = self.r16(t.wrapping_add(T_LFO_SPEED));
        let phase = self.r16(t.wrapping_add(T_LFO_PHASE));
        let (sum, carry) = phase.overflowing_add(speed);
        self.w16(t.wrapping_add(T_LFO_PHASE), sum);
        if carry {
            let f = self.r8(t.wrapping_add(T_LFO_FLAGS));
            self.w8(t.wrapping_add(T_LFO_FLAGS), f.wrapping_add(0x40));
        }
        self.portamento_step(t);
    }

    /// 0x11B4 .. 0x11E8: while bit 7 of +10 is set the pitch moves by the slide (+11). It stops
    /// on the target when the sum reaches it: equal, or crossing it in the slide direction
    /// (the sign byte plus the borrow is 0). Then bit 7 is cleared.
    fn portamento_step(&mut self, t: u16) {
        if self.r8(t.wrapping_add(T_PORTA)) & 0x80 == 0 {
            return;
        }
        let step = self.r16be(t.wrapping_add(T_SLIDE));
        let cur = self.r16be(t.wrapping_add(T_PITCH));
        let target = self.r16be(t.wrapping_add(T_TARGET));
        let new = cur.wrapping_add(step);
        let (diff, borrow) = new.overflowing_sub(target);
        let sign: u8 = if step & 0x8000 != 0 { 0xFF } else { 0 };
        let reached = diff == 0 || sign.wrapping_add(borrow as u8) == 0;
        let value = if reached {
            let p = self.r8(t.wrapping_add(T_PORTA));
            self.w8(t.wrapping_add(T_PORTA), p & 0x7F);
            target
        } else {
            new
        };
        self.r16be_set(t.wrapping_add(T_PITCH), value);
    }

    /// 0x11EE: the envelope handler by state (+25). States 2 and 5 are `ret` (0x120B): nothing
    /// is written for them. States 6 and 7 never occur.
    fn envelope_step(&mut self, t: u16) {
        match self.r8(t.wrapping_add(T_ENV_STATE)) {
            0 => self.env_attack(t),
            1 => self.env_decay(t),
            3 | 4 => self.env_fall(t),
            _ => {}
        }
    }

    /// 0x120C: attack. The level adds the rate; a carry out ends the attack (level 0xFFFF).
    fn env_attack(&mut self, t: u16) {
        let level = self.r16(t.wrapping_add(T_ENV_LEVEL));
        let rate = self.r16(t.wrapping_add(T_ENV_RATE));
        let (sum, carry) = level.overflowing_add(rate);
        if carry {
            self.env_change(t, 0xFFFF);
        } else {
            self.w16(t.wrapping_add(T_ENV_LEVEL), sum);
            self.level_writes(t);
        }
    }

    /// 0x1227: decay. The level drops by the rate until it reaches the sustain level (+26).
    fn env_decay(&mut self, t: u16) {
        let sustain = self.r16(t.wrapping_add(T_SUSTAIN));
        let level = self.r16(t.wrapping_add(T_ENV_LEVEL));
        let rate = self.r16(t.wrapping_add(T_ENV_RATE));
        let (d, borrow) = level.overflowing_sub(rate);
        if borrow || d <= sustain {
            self.env_change(t, sustain);
        } else {
            self.w16(t.wrapping_add(T_ENV_LEVEL), d);
            self.level_writes(t);
        }
    }

    /// 0x124F (slow decay, state 3) and 0x126E (release, state 4): the level drops by the rate
    /// to 0, then the state changes to off.
    fn env_fall(&mut self, t: u16) {
        let level = self.r16(t.wrapping_add(T_ENV_LEVEL));
        let rate = self.r16(t.wrapping_add(T_ENV_RATE));
        let (d, borrow) = level.overflowing_sub(rate);
        if borrow || d == 0 {
            self.env_change(t, 0);
        } else {
            self.w16(t.wrapping_add(T_ENV_LEVEL), d);
            self.level_writes(t);
        }
    }

    /// 0x128D: state change. A zero target turns the envelope off (state 5). Otherwise the
    /// state goes up by one, skipping 2, and its rate is the 0x2629 entry of the instrument
    /// byte for the new state (+1 decay, +3 slow decay, +4 release). The level becomes the target.
    fn env_change(&mut self, t: u16, target: u16) {
        if target == 0 {
            self.w8(t.wrapping_add(T_ENV_STATE), 5);
        } else {
            let mut ns = self.r8(t.wrapping_add(T_ENV_STATE)).wrapping_add(1);
            if ns == 2 {
                ns = 3;
            }
            let p = self.r16(t.wrapping_add(T_ENV_PTR)) as u32;
            let idx = self.rom(p.wrapping_add(ns as u32)) as u32;
            let rate = rom16(self, TAB_ENV_DECAY + 2 * idx);
            self.w16(t.wrapping_add(T_ENV_RATE), rate);
            self.w8(t.wrapping_add(T_ENV_STATE), ns);
        }
        self.w16(t.wrapping_add(T_ENV_LEVEL), target);
        self.level_writes(t);
    }

    /// 0x12C2 .. 0x13DB: the volume and pitch writes of track `t` on the voice in F00E. Nothing
    /// is written while the track is muted (+33) or an SFX owns the voice (F020).
    pub(crate) fn level_writes(&mut self, t: u16) {
        if self.r8(t.wrapping_add(T_MUTE)) != 0 || self.r8(VOICE_OWNED) != 0 {
            return;
        }
        let speed = self.r16(t.wrapping_add(T_LFO_SPEED));
        let flags = self.r8(t.wrapping_add(T_LFO_FLAGS));
        // Volume (0x12CC): tremolo factor, times the envelope level, times the music master
        // volume and the second fade (music only), times the first fade, times the channel volume.
        let mut vol = if speed == 0 {
            0xFFFF
        } else {
            let mut ph = self.r16(t.wrapping_add(T_LFO_PHASE));
            if flags & 0x40 != 0 {
                ph = 0xFFFF - ph;
            }
            let depth = self.r16(t.wrapping_add(T_TREM));
            let tri = mul16(self, ph, depth);
            0xFFFF - tri
        };
        let level = self.r16(t.wrapping_add(T_ENV_LEVEL));
        vol = mul16(self, vol, level);
        if self.r8(SEQ_FLAG) != 0 {
            let mv = self.r16(MUSIC_VOLUME);
            vol = mul16(self, vol, mv);
            if self.r8(FADE2_ON) != 0 {
                let f2 = self.r16(FADE2_LEVEL);
                vol = mul16(self, vol, f2);
            }
        }
        if self.r8(FADE1_ON) != 0 {
            let f1 = self.r16(FADE1_LEVEL);
            vol = mul16(self, vol, f1);
        }
        let chan = self.r16(t.wrapping_add(T_VOLUME));
        vol = mul16(self, vol, chan);
        let v8 = self.r8(VOICE8);
        self.dsp(v8.wrapping_add(6), vol);

        // Pitch (0x1343): the vibrato (LFO) is added to the track pitch, plus the music pitch
        // offset (F01B) for music, then the clamp and the table lookup.
        let mut p = if speed == 0 {
            0
        } else {
            let mut ph = self.r16(t.wrapping_add(T_LFO_PHASE));
            if flags & 0x40 != 0 {
                ph = 0xFFFF - ph;
            }
            let depth = self.r16(t.wrapping_add(T_VIB));
            let mut v = mul16(self, ph, depth);
            if flags & 0x80 != 0 {
                v = 0u16.wrapping_sub(v);
            }
            v
        };
        p = p.wrapping_add(self.r16be(t.wrapping_add(T_PITCH)));
        if self.r8(SEQ_FLAG) != 0 {
            p = p.wrapping_add(self.r16(PITCH_OFFSET));
        }
        let hi = (p >> 8) as u8;
        let lo = p as u8;
        // 0x138F: the note index, with H - 1.
        let a = 0x3Cu8
            .wrapping_sub(self.r8(t.wrapping_add(T_ROOT)))
            .wrapping_add(hi.wrapping_sub(1));
        let n = pitch_clamp(a);
        let octave = self.rom(TAB_OCTAVE + n as u32);
        let first = self.rom(TAB_OCTAVE_FIRST + octave as u32);
        let row = n.wrapping_sub(first) as u32;
        let frac = self.rom(TAB_FRACTION + row * 256 + lo as u32) as u16;
        let shifted = frac << octave;
        let base = rom16(self, TAB_PITCH_BASE + 2 * n as u32);
        self.dsp(v8.wrapping_add(2), base.wrapping_add(shifted));
    }

    /// Writes a big-endian word of the track (pitch, slide, target).
    fn r16be_set(&mut self, a: u16, v: u16) {
        let [hi, lo] = v.to_be_bytes();
        self.w8(a, hi);
        self.w8(a.wrapping_add(1), lo);
    }
}
