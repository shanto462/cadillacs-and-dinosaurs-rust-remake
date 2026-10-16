//! The sequencer (SPEC sections 4, 6, 7): the 62.5 Hz step over the 16 voices (0x00DA), the
//! track step (0x0A28), the event reader (0x0A8B), notes with portamento and key-on (0x0AD4 ..
//! 0x0C4F), the commands 0x00..0x1F (table 0x0C84), the sample load (0x0D5E), the track end
//! (0x0EA4) and the tied-note resume (0x0EF0).
//!
//! Not here yet (M4-05 and M4-06): the no-event step 0x1007 (pan motion, LFO, envelope and the
//! writes of volume and pitch), the volume and pitch writes 0x12C2 and 0x12CC, the ADPCM slots
//! 0x212C (no ADPCM header exists in this ROM, SPEC 11). The sequence reads follow the Z80 bank
//! window (0x0A8B), and the sample, instrument and table reads go through the data image.

use crate::driver::Driver;
use crate::map::*;

/// Word at a linear address of the data image (little-endian, as the Z80 reads a table).
fn le_at(d: &Driver, addr: u32) -> u16 {
    u16::from_le_bytes([d.rom(addr), d.rom(addr.wrapping_add(1))])
}

/// Linear address of a 24-bit sequence pointer, as the Z80 read routine (0x0A8B, 0x0FC7)
/// maps it. Let x = (ptr >> 14) - 2 (8 bits). When x is 1..=0x7F the bank window gives the
/// linear address back unchanged. Otherwise the Z80 reads its CPU address (mid:lo) with
/// bank 0. The ROM's pointers always take the first case.
fn seq_linear(ptr: u32) -> u32 {
    let x = (((ptr >> 14) & 0xFF) as u8).wrapping_sub(2);
    if (1..=0x7F).contains(&x) {
        ptr
    } else {
        let cpu = ptr & 0xFFFF;
        if cpu < 0x8000 { cpu } else { 0x8000 | (cpu & 0x3FFF) }
    }
}

impl<'a> Driver<'a> {
    // ----- 0x00DA: the sequencer step (every 4th tick) ---------------------------------------

    /// 0x00DA .. 0x0141 (the loop over the voices), then the fades 0x0143 .. 0x017A. Called from
    /// the main loop on every 4th tick.
    pub(crate) fn sequencer_step(&mut self) {
        for v in 0..VOICES {
            self.set_voice(v);
            self.w8(SEQ_FLAG, 0);
            self.w8(VOICE_OWNED, 0);
            // SFX track of voice v.
            let s = sfx_track(v);
            if self.r8(s.wrapping_add(T_ACTIVE)) != 0 {
                let mut owned = false;
                let delay = self.r8(s.wrapping_add(T_DELAY));
                if delay != 0 {
                    // 0x0101: the start delay counts down; at zero the voice is reset and panned.
                    let left = delay - 1;
                    self.w8(s.wrapping_add(T_DELAY), left);
                    if left == 0 {
                        self.voice_reset(v);
                        let e = self.r8(s.wrapping_add(T_PAN));
                        self.pan_write_dbg(v, e);
                        owned = true;
                    }
                } else {
                    // 0x010D: tempo = the track's own word (+30/+31), then the step.
                    let tempo = self.r16(s.wrapping_add(T_TEMPO));
                    self.w16(TEMPO_WORD, tempo);
                    self.track_step(s);
                    owned = true;
                }
                if owned {
                    self.w8(VOICE_OWNED, 0xFF);
                }
            }
            // Music track of voice v (0x011E): F006 = its active byte, tempo = F012 + F014.
            let m = music_track(v);
            let act = self.r8(m.wrapping_add(T_ACTIVE));
            if act != 0 {
                self.w8(SEQ_FLAG, act);
                let tempo = self.r16(MUSIC_TEMPO).wrapping_add(self.r16(TEMPO_OFFSET));
                self.w16(TEMPO_WORD, tempo);
                self.track_step(m);
            }
        }
        // 0x0143 .. 0x017A: music fade (F030) and second fade (F035), both clamp at 0.
        if self.r8(FADE1_ON) != 0 {
            let l = self.r16(FADE1_LEVEL).saturating_sub(self.r16(FADE1_SPEED));
            self.w16(FADE1_LEVEL, l);
        }
        if self.r8(FADE2_ON) != 0 {
            let l = self.r16(FADE2_LEVEL).saturating_sub(self.r16(FADE2_SPEED));
            self.w16(FADE2_LEVEL, l);
        }
    }

    // ----- 0x0A28: one track step ------------------------------------------------------------

    /// 0x0A28: gate counter, duration counter, then either the next event (0x0A8B) or the
    /// no-event step (0x1007).
    fn track_step(&mut self, t: u16) {
        if self.r16(t.wrapping_add(T_DUR)) == 0 {
            self.read_events(t);
            return;
        }
        let tempo = self.r16(TEMPO_WORD);
        let gate = self.r16(t.wrapping_add(T_GATE));
        if gate != 0 {
            // Gate above the tempo: subtract. Equal or below: release (0x0A47), counter 0.
            let g = if gate > tempo {
                gate - tempo
            } else {
                self.release(t);
                0
            };
            self.w16(t.wrapping_add(T_GATE), g);
        }
        let dur = self.r16(t.wrapping_add(T_DUR));
        let borrow = dur < tempo;
        let dur = dur.wrapping_sub(tempo);
        self.w16(t.wrapping_add(T_DUR), dur);
        // 0x0A7F .. 0x0A88: zero or borrow reads events; a high byte of 0xC1 or more too.
        if dur == 0 || borrow || (dur >> 8) as u8 >= 0xC1 {
            self.read_events(t);
        } else {
            self.no_event_step(t);
        }
    }

    /// 0x0A47: enter the release state, the release rate is 0x2629 at the instrument's byte 6.
    fn release(&mut self, t: u16) {
        self.w8(t.wrapping_add(T_ENV_STATE), 4);
        let p = self.r16(t.wrapping_add(T_ENV_PTR)).wrapping_add(4) as u32;
        let idx = self.rom(p) as u32;
        let rate = le_at(self, TAB_ENV_DECAY + 2 * idx);
        self.w16(t.wrapping_add(T_ENV_RATE), rate);
    }

    /// 0x1007: pan motion, LFO, envelope and the volume and pitch writes. Not ported yet
    /// (M4-05 and M4-06). Until then a step with no event writes nothing.
    fn no_event_step(&mut self, _t: u16) {}

    /// 0x12C2 and 0x12CC: the volume and pitch writes of the voice. Not ported yet (M4-05).
    fn level_writes(&mut self) {}

    // ----- 0x0A8B: events -------------------------------------------------------------------

    /// Reads the byte at the track pointer and moves the pointer on (0x0A8B, 0x0FC7).
    fn seq_byte(&mut self, t: u16) -> u8 {
        let p = self.seq_ptr(t);
        let b = self.rom(seq_linear(p));
        self.set_seq_ptr(t, p.wrapping_add(1) & 0xFF_FFFF);
        b
    }

    fn seq_ptr(&self, t: u16) -> u32 {
        u32::from_le_bytes([
            self.r8(t.wrapping_add(T_PTR)),
            self.r8(t.wrapping_add(T_PTR + 1)),
            self.r8(t.wrapping_add(T_PTR + 2)),
            0,
        ])
    }

    fn set_seq_ptr(&mut self, t: u16, p: u32) {
        self.set_track_ptr(t, p);
    }

    /// Reads events until a note or a rest (0x0AD4 ends the step) or the track end. A command
    /// (byte below 0x20) runs and the next event is read (0x0A8B).
    fn read_events(&mut self, t: u16) {
        // The ROM always reaches a note or the track end. The cap only stops a track of bad
        // data from looping forever (4096 commands with no note in one step).
        for _ in 0..4096 {
            let b = self.seq_byte(t);
            if b >= 0x20 {
                self.note(t, b);
                return;
            }
            if self.seq_command(t, b) {
                return;
            }
        }
    }

    // ----- 0x0AD4: notes --------------------------------------------------------------------

    /// One note or rest with event byte `b` (0x20 or more). Duration, pitch and portamento,
    /// key-on (unless tied), then the gate counter (0x0C23).
    fn note(&mut self, t: u16, b: u8) {
        // Length: the table depends on the flags (dotted beats triplet). A dotted note clears
        // its dot flag (0x0AF7).
        let idx = ((b >> 5) - 1) as u32;
        let mut flags = self.r8(t.wrapping_add(T_FLAGS));
        let table = if flags & 0x30 != 0 {
            if flags & 0x10 != 0 {
                flags &= !0x10;
                self.w8(t.wrapping_add(T_FLAGS), flags);
                TAB_LEN_DOTTED
            } else {
                TAB_LEN_TRIPLET
            }
        } else {
            TAB_LEN_NORMAL
        };
        let len = self.rom(table + idx) as u16;
        let dur = self.r16(t.wrapping_add(T_DUR)).wrapping_add(len << 8);
        self.w16(t.wrapping_add(T_DUR), dur);

        let note = b & 0x1F;
        if note == 0 {
            return; // rest
        }
        // Semitone = note + table 0x0C65[flags & 0x0F] + music transpose (F006 set) + transpose.
        let base = self.rom(TAB_NOTE_BASE + (flags & 0x0F) as u32);
        let music = if self.r8(SEQ_FLAG) != 0 { self.r8(MUSIC_TRANSPOSE) } else { 0 };
        let semi = note
            .wrapping_add(base)
            .wrapping_add(music)
            .wrapping_add(self.r8(t.wrapping_add(T_TRANSPOSE)));
        // 8.8 pitch: detune + fine (signed byte) added, then the high byte clamp.
        let fine = self.r8(t.wrapping_add(T_DETUNE)).wrapping_add(self.r8(t.wrapping_add(T_FINE)));
        let mut pitch = ((semi as u16) << 8).wrapping_add(fine as i8 as i16 as u16);
        if (pitch >> 8) as u8 >= 0x62 {
            pitch = (0x61u16 << 8) | (pitch & 0xFF);
        }
        let new_hi = (pitch >> 8) as u8;

        // Portamento (0x0B4E .. 0x0B81): the current pitch starts at the old target and slides.
        let mut cur = pitch;
        let porta = self.r8(t.wrapping_add(T_PORTA)) & 0x7F;
        self.w8(t.wrapping_add(T_PORTA), porta);
        if porta != 0 {
            let th = self.r8(t.wrapping_add(T_TARGET));
            if th != 0 && th != new_hi {
                let mut step = (porta as u16) << 1;
                if th >= new_hi {
                    step = 0u16.wrapping_sub(step);
                }
                self.w8(t.wrapping_add(T_SLIDE), (step >> 8) as u8);
                self.w8(t.wrapping_add(T_SLIDE + 1), step as u8);
                cur = u16::from_be_bytes([th, self.r8(t.wrapping_add(T_TARGET + 1))]);
                self.w8(t.wrapping_add(T_PORTA), porta | 0x80);
            }
        }
        // 0x0B81: target = the new pitch, current = cur.
        let [nh, nl] = pitch.to_be_bytes();
        self.w8(t.wrapping_add(T_TARGET), nh);
        self.w8(t.wrapping_add(T_TARGET + 1), nl);
        let [ch, cl] = cur.to_be_bytes();
        self.w8(t.wrapping_add(T_PITCH), ch);
        self.w8(t.wrapping_add(T_PITCH + 1), cl);

        if self.r8(t.wrapping_add(T_FLAGS)) & 0x80 != 0 {
            // Tied note (0x0C1C): no key-on, the mute is cleared.
            self.note_end_levels(t);
        } else {
            // Key-on (0x0B94 .. 0x0C19).
            let lfo = self.r8(t.wrapping_add(T_LFO_FLAGS));
            if lfo & 1 != 0 {
                self.w8(t.wrapping_add(T_LFO_FLAGS), lfo & 0x3F);
                self.w16(t.wrapping_add(T_LFO_PHASE), 0);
            }
            self.w8(t.wrapping_add(T_ENV_STATE), 0);
            let p = self.r16(t.wrapping_add(T_ENV_PTR)) as u32;
            let idx = self.rom(p) as u32;
            let w = le_at(self, TAB_ENV_ATTACK + 2 * idx);
            self.w16(t.wrapping_add(T_ENV_RATE), w);
            self.w16(t.wrapping_add(T_ENV_LEVEL), w);
            if self.r8(VOICE_OWNED) == 0 {
                self.key_on(t);
                self.note_end_levels(t);
            }
        }
        self.gate_counter(t);
    }

    /// 0x0C1C: mute cleared, then the volume and pitch writes.
    fn note_end_levels(&mut self, t: u16) {
        self.w8(t.wrapping_add(T_MUTE), 0);
        self.level_writes();
    }

    /// The key-on writes (0x0BD1 .. 0x0C13): pitch 0, 0x93, 0xD9, echo send, sample start and
    /// the 0x8000 loop/mode word.
    fn key_on(&mut self, t: u16) {
        let v = self.r8(VOICE);
        let b8 = self.r8(VOICE8);
        self.dsp(b8.wrapping_add(2), 0);
        self.dsp(0x93, 0x27FF);
        self.dsp(0xD9, 0x07FF);
        self.echo_write(t, v);
        self.dsp(b8.wrapping_add(1), self.r16(t.wrapping_add(T_SAMPLE_START)));
        self.dsp(b8.wrapping_add(3), 0x8000);
    }

    /// Echo send register 0xBA+v = table 0x23A9[echo index] * 4.
    fn echo_write(&mut self, t: u16, v: u8) {
        let idx = self.r8(t.wrapping_add(T_ECHO)) as u32;
        let w = le_at(self, TAB_VOLUME + 2 * idx).wrapping_mul(4);
        self.dsp(0xBAu8.wrapping_add(v), w);
    }

    /// 0x0C23: the tie bit moves from bit 6 to bit 7 of the flags. The gate counter is
    /// duration x gate ratio / 256 (0x29CB), or 0 when the note is tied.
    fn gate_counter(&mut self, t: u16) {
        let f = self.r8(t.wrapping_add(T_FLAGS));
        let nf = (f & 0x7F) | ((f & 0x40) << 1);
        self.w8(t.wrapping_add(T_FLAGS), nf);
        let gate = if nf & 0x80 != 0 {
            0
        } else {
            let dur = self.r16(t.wrapping_add(T_DUR));
            let ratio = self.r8(t.wrapping_add(T_GATE_RATIO)) as u16;
            mul_hi(self, dur, ratio << 8)
        };
        self.w16(t.wrapping_add(T_GATE), gate);
    }

    // ----- 0x0EA4: track end ----------------------------------------------------------------

    /// 0x0EA4: the track has ended. An SFX stops (0x0EBB). A music track becomes inactive and
    /// its voice is reset unless an SFX owns the voice. Returns true (the event loop stops).
    fn track_end(&mut self, t: u16) -> bool {
        if self.r8(SEQ_FLAG) == 0 {
            let v = self.r8(VOICE);
            self.stop_sfx(t, v);
        } else {
            self.w8(t.wrapping_add(T_ACTIVE), 0);
            if self.r8(VOICE_OWNED) == 0 {
                let v = self.r8(VOICE);
                self.voice_reset(v);
            }
        }
        true
    }

    // ----- 0x0C84: commands 0x00 .. 0x1F ----------------------------------------------------

    /// Runs command `code` (below 0x20). Returns true when the track ended.
    fn seq_command(&mut self, t: u16, code: u8) -> bool {
        let fl = t.wrapping_add(T_FLAGS);
        match code {
            // 0x0CC2: toggle triplet.
            0x00 => {
                let f = self.r8(fl);
                self.w8(fl, f ^ 0x20);
            }
            // 0x0CCB: toggle tie.
            0x01 => {
                let f = self.r8(fl);
                self.w8(fl, f ^ 0x40);
            }
            // 0x0CD4: dot the next note.
            0x02 => {
                let f = self.r8(fl);
                self.w8(fl, f | 0x10);
            }
            // 0x0CD9: toggle the octave bank.
            0x03 => {
                let f = self.r8(fl);
                self.w8(fl, f ^ 0x08);
            }
            // 0x0E72: flags = (flags & 0x97) | n.
            0x04 => {
                let n = self.seq_byte(t);
                self.set_flags_masked(t, n);
            }
            // 0x0CE2: tempo hh ll. Music sets F012, an effect sets its own word.
            0x05 => {
                let hi = self.seq_byte(t);
                let lo = self.seq_byte(t);
                if self.r8(SEQ_FLAG) != 0 {
                    self.w8(MUSIC_TEMPO, lo);
                    self.w8(MUSIC_TEMPO + 1, hi);
                } else {
                    self.w8(t.wrapping_add(T_TEMPO), lo);
                    self.w8(t.wrapping_add(T_TEMPO + 1), hi);
                }
            }
            // 0x0CFD: gate ratio.
            0x06 => {
                let n = self.seq_byte(t);
                self.w8(t.wrapping_add(T_GATE_RATIO), n);
            }
            // 0x0D04: channel volume from table 0x23A9.
            0x07 => {
                let n = self.seq_byte(t) as u32;
                let w = le_at(self, TAB_VOLUME + 2 * n);
                self.w16(t.wrapping_add(T_VOLUME), w);
            }
            // 0x0D19: instrument.
            0x08 => self.instrument(t),
            // 0x0DC1: octave (flags low 3 bits).
            0x09 => {
                let n = self.seq_byte(t);
                let f = self.r8(fl);
                self.w8(fl, (f & 0xF8) | n);
            }
            // 0x0DCF: music transpose (music tracks only).
            0x0A => {
                let n = self.seq_byte(t);
                if self.r8(SEQ_FLAG) != 0 {
                    self.w8(MUSIC_TRANSPOSE, n);
                }
            }
            // 0x0DDD: transpose.
            0x0B => {
                let n = self.seq_byte(t);
                self.w8(t.wrapping_add(T_TRANSPOSE), n);
            }
            // 0x0DE4: detune.
            0x0C => {
                let n = self.seq_byte(t);
                self.w8(t.wrapping_add(T_DETUNE), n);
            }
            // 0x0DEB: portamento speed (bit 7 cleared).
            0x0D => {
                let n = self.seq_byte(t) & 0x7F;
                self.w8(t.wrapping_add(T_PORTA), n);
            }
            // 0x0E00 .. 0x0E37 and 0x0E09: loop k with the count operand.
            0x0E..=0x11 => self.loop_cmd(t, (code - 0x0E) as u16),
            // 0x0E38 .. 0x0E4D: break k.
            0x12..=0x15 => self.break_cmd(t, (code - 0x12) as u16),
            // 0x0E80: jump hh ll.
            0x16 => self.jump(t),
            // 0x0EA4: track end.
            0x17 => return self.track_end(t),
            // 0x0F41: pan (music tracks only).
            0x18 => {
                let n = self.seq_byte(t);
                if self.r8(SEQ_FLAG) != 0 {
                    self.w8(t.wrapping_add(T_PAN), n);
                    if self.r8(VOICE_OWNED) == 0 {
                        let v = self.r8(VOICE);
                        self.pan_write(v, n);
                    }
                }
            }
            // 0x0F56: echo send index.
            0x19 => {
                let n = self.seq_byte(t);
                self.w8(t.wrapping_add(T_ECHO), n);
            }
            // 0x0F5D: music volume = F016 * table 0x24A9[n] >> 16 (music tracks only).
            0x1A => {
                let n = self.seq_byte(t) as u32;
                if self.r8(SEQ_FLAG) != 0 {
                    let gain = le_at(self, TAB_GAIN + 2 * n);
                    let master = self.r16(MASTER_VOLUME);
                    let v = mul_hi(self, master, gain);
                    self.w16(MUSIC_VOLUME, v);
                }
            }
            // 0x0F7A: vibrato depth (table 0x27A9).
            0x1B => {
                let n = self.seq_byte(t) as u32;
                let w = le_at(self, TAB_VIBRATO + 2 * n);
                self.w16(t.wrapping_add(T_VIB), w);
            }
            // 0x0F8F: tremolo depth (table 0x28A9).
            0x1C => {
                let n = self.seq_byte(t) as u32;
                let w = le_at(self, TAB_TREMOLO + 2 * n);
                self.w16(t.wrapping_add(T_TREM), w);
            }
            // 0x0FA4: LFO speed (table 0x26A9).
            0x1D => {
                let n = self.seq_byte(t) as u32;
                let w = le_at(self, TAB_LFO_SPEED + 2 * n);
                self.w16(t.wrapping_add(T_LFO_SPEED), w);
            }
            // 0x0FB9: LFO flags: bit 0 from the operand, the other bits kept.
            0x1E => {
                let n = self.seq_byte(t);
                let lfo = self.r8(t.wrapping_add(T_LFO_FLAGS));
                self.w8(t.wrapping_add(T_LFO_FLAGS), n | (lfo & 0xFE));
            }
            // 0x1F points into the pitch fraction table (0x208A). No track uses it (SPEC 4).
            _ => {}
        }
        false
    }

    /// 0x0E72 (and the tail of command 0x12..0x15): flags = (flags & 0x97) | n.
    fn set_flags_masked(&mut self, t: u16, n: u8) {
        let fl = t.wrapping_add(T_FLAGS);
        let f = self.r8(fl);
        self.w8(fl, (f & 0x97) | n);
    }

    /// 0x0D19: instrument n (8 bytes at table 0x6E62). Loads the sample when it changes and
    /// sets the fine tune, the envelope pointer (instrument + 2) and the sustain level.
    fn instrument(&mut self, t: u16) {
        let n = self.seq_byte(t) as u32;
        let base = TAB_INSTRUMENT + 8 * n;
        let sample_plus1 = self.rom(base).wrapping_add(1);
        if sample_plus1 != self.r8(t.wrapping_add(T_ACTIVE)) {
            self.w8(t.wrapping_add(T_ACTIVE), sample_plus1);
            if self.r8(VOICE_OWNED) == 0 {
                self.sample_load(t, sample_plus1.wrapping_sub(1));
            }
        }
        let fine = self.rom(base + 1);
        self.w8(t.wrapping_add(T_FINE), fine);
        self.w16(t.wrapping_add(T_ENV_PTR), (base + 2) as u16);
        let sus = le_at(self, TAB_GAIN + 2 * self.rom(base + 4) as u32);
        self.w16(t.wrapping_add(T_SUSTAIN), sus);
    }

    /// 0x0D5E: load sample `idx` of voice F00D. Writes the start, the end, the loop length,
    /// the bank word, and copies the root key into the track (+0B).
    pub(crate) fn sample_load(&mut self, t: u16, idx: u8) {
        let p = TAB_SAMPLE + 8 * idx as u32;
        let bank = self.rom(p);
        let start = le_at(self, p + 1);
        let loop_start = le_at(self, p + 3);
        let end = le_at(self, p + 5);
        let root = self.rom(p + 7);
        let v = self.r8(VOICE);
        let b8 = self.r8(VOICE8);
        self.dsp(b8.wrapping_add(2), 0);
        self.dsp(b8.wrapping_add(1), start);
        self.w16(t.wrapping_add(T_SAMPLE_START), start);
        self.dsp(b8.wrapping_add(5), end);
        self.dsp(b8.wrapping_add(4), end.wrapping_sub(loop_start));
        // Bank register of voice v - 1 (voice 0 uses 0x78): ((v-1)*8) & 0x78.
        let bank_reg = v.wrapping_sub(1).rotate_left(3) & 0x78;
        self.dsp(bank_reg, 0x8000 | bank as u16);
        self.w8(t.wrapping_add(T_ROOT), root);
    }

    /// 0x0EF0: resume a tied note after an SFX ends on its voice: the echo, the sample start
    /// and the loop word, without the pitch reset. F006 is set around the level writes.
    pub(crate) fn tie_resume(&mut self, m: u16, v: u8) {
        self.dsp(0x93, 0x27FF);
        self.dsp(0xD9, 0x07FF);
        self.echo_write(m, v);
        let b8 = self.r8(VOICE8);
        self.dsp(b8.wrapping_add(1), self.r16(m.wrapping_add(T_SAMPLE_START)));
        self.dsp(b8.wrapping_add(3), 0x8000);
        self.w8(SEQ_FLAG, 0xFF);
        self.level_writes();
        self.w8(SEQ_FLAG, 0);
    }

    /// 0x0E09 .. 0x0E37 (commands 0E..11): the loop counter starts at n on the first arrival
    /// and the jump is taken while it is not yet 0 after a decrement. The body runs n + 1 times.
    fn loop_cmd(&mut self, t: u16, k: u16) {
        let off = T_LOOP + k;
        let n = self.seq_byte(t);
        let c = self.r8(t.wrapping_add(off));
        if c == 0 {
            self.w8(t.wrapping_add(off), n);
            self.jump(t);
        } else {
            let c = c - 1;
            self.w8(t.wrapping_add(off), c);
            if c != 0 {
                self.jump(t);
            } else {
                self.skip(t, 2);
            }
        }
    }

    /// 0x0E38 .. 0x0E6D (commands 12..15): on the last pass (counter 1) the counter is cleared,
    /// the flags are set from the operand and the jump is taken. Otherwise the 3 bytes are skipped.
    fn break_cmd(&mut self, t: u16, k: u16) {
        let off = T_LOOP + k;
        let c = self.r8(t.wrapping_add(off)).wrapping_sub(1);
        if c == 0 {
            self.w8(t.wrapping_add(off), 0);
            let f = self.seq_byte(t);
            self.set_flags_masked(t, f);
            self.jump(t);
        } else {
            self.skip(t, 3);
        }
    }

    /// 0x0E80: jump by the signed big-endian offset that follows the command, counted from
    /// the byte after it.
    fn jump(&mut self, t: u16) {
        let hi = self.seq_byte(t);
        let lo = self.seq_byte(t);
        let off = u16::from_be_bytes([hi, lo]) as i16 as i32 as u32;
        let p = self.seq_ptr(t).wrapping_add(off) & 0xFF_FFFF;
        self.set_seq_ptr(t, p);
    }

    /// Moves the track pointer forward by `n` bytes without reading them.
    fn skip(&mut self, t: u16, n: u32) {
        let p = self.seq_ptr(t).wrapping_add(n) & 0xFF_FFFF;
        self.set_seq_ptr(t, p);
    }
}

/// 0x29CB: HL = (HL * DE) >> 16, floor. Also leaves HL at F000 and DE at F002 (SPEC 13).
fn mul_hi(d: &mut Driver, a: u16, b: u16) -> u16 {
    d.w16(0xF000, a);
    d.w16(0xF002, b);
    ((a as u32 * b as u32) >> 16) as u16
}
