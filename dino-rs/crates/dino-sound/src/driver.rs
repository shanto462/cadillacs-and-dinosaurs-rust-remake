//! The driver: 68000 mailbox, IRQ, main loop, command dispatch, SFX and music start, stops,
//! fades and tempo or pitch offsets. Z80 addresses are kept as names (see `map.rs`).

use dino_assets::sound::SoundData;

use crate::map::*;

/// One 68000 request (the ring entry of 0xA63C: d0 word, d1 long, d2 long).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MailEntry {
    pub code: u16,
    pub d1: u32,
    pub d2: u32,
}

/// One write to the QSound DSP: register number and value (the helper 0x0A13 writes the
/// value in DE and the register in A).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegWrite {
    pub reg: u8,
    pub value: u16,
}

/// What a track holds right now (for tests and the game).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrackInfo {
    pub active: bool,
    /// 24-bit position of the next sequence byte (+00..+02).
    pub ptr: u32,
    pub priority: u8,
    pub pan: u8,
    pub muted: bool,
}

/// The sound driver. Holds the Z80 RAM image, the 68000 request ring and the DSP write log.
/// Sound data is borrowed from the asset pack and read by address, as the Z80 reads the ROM.
pub struct Driver<'a> {
    pub(crate) snd: &'a SoundData,
    pub(crate) ram: Box<[u8; 0x10000]>,
    dsp: Vec<RegWrite>,
    /// 68000 request ring at 0xFF803A (64 entries). Indexes are byte offsets 0..0x3F0 (SPEC 3).
    mail: [MailEntry; 64],
    /// 68000 write index (RAM 0xFF8034).
    mail_wr: u16,
    /// 68000 read index (RAM 0xFF8036).
    mail_rd: u16,
}

impl<'a> Driver<'a> {
    /// Power on: the Z80 reset (SPEC 4, the DSP init and the reset of every voice), the 68000
    /// boot (SPEC 3: enable with 0x88, empty ring, 0xFF0B, then stop everything).
    /// `stereo` is the cabinet setting (`G.$777A` bit 7).
    pub fn new(snd: &'a SoundData, stereo: bool) -> Self {
        let mut d = Driver {
            snd,
            ram: Box::new([0; 0x10000]),
            dsp: Vec::new(),
            mail: [MailEntry::default(); 64],
            mail_wr: 0,
            mail_rd: 0,
        };
        // 0xA716: stereo flag on the shared RAM (0xFF stereo, 0 mono).
        d.w8(SHARED_STEREO, if stereo { 0xFF } else { 0 });
        d.z80_boot();
        d.m68k_init();
        d
    }

    /// 0xA63C: queue one request from the 68000 side. A full ring keeps the old write index, so
    /// at most 63 requests wait (the 64th overwrites the slot without committing it).
    pub fn command(&mut self, code: u16, d1: u32, d2: u32) {
        let slot = (self.mail_wr / 0x10) as usize;
        self.mail[slot] = MailEntry { code, d1, d2 };
        let next = self.mail_wr.wrapping_add(0x10) & 0x3F0;
        if next != self.mail_rd {
            self.mail_wr = next;
        }
    }

    /// 0x610: the vblank flush. Sends one queued request when the Z80 has taken the previous one
    /// (shared status 0xFF). Called once per video frame (59.6374 Hz).
    pub fn vblank(&mut self) {
        if self.mail_rd == self.mail_wr || self.r8(SHARED_STATUS) != 0xFF {
            return;
        }
        let e = self.mail[(self.mail_rd / 0x10) as usize];
        self.mail_rd = self.mail_rd.wrapping_add(0x10) & 0x3F0;
        let [code_hi, code_lo] = e.code.to_be_bytes();
        let d1 = e.d1.to_be_bytes();
        let d2 = e.d2.to_be_bytes();
        if code_hi == 0xFF {
            // Special command: sub-command in C001, the high two bytes of d1 in C003 and C004.
            self.w8(SHARED_REQ, 0xFF);
            self.w8(SHARED_REQ + 1, code_lo);
            self.w8(SHARED_REQ + 3, d1[0]);
            self.w8(SHARED_REQ + 4, d1[1]);
        } else {
            // Normal request: C000 and C001 = code, C006..C008 = d1, C009..C00C = d2.
            self.w8(SHARED_REQ, code_hi);
            self.w8(SHARED_REQ + 1, code_lo);
            self.w8(SHARED_REQ + 6, d1[1]);
            self.w8(SHARED_REQ + 7, d1[3]);
            self.w8(SHARED_REQ + 8, d1[2]);
            self.w8(SHARED_REQ + 9, d2[0]);
            self.w8(SHARED_REQ + 10, d2[1]);
            self.w8(SHARED_REQ + 11, d2[2]);
            self.w8(SHARED_REQ + 12, d2[3]);
        }
        self.w8(SHARED_STATUS, 0);
    }

    /// One 250 Hz tick: the IRQ (0x0038) and one pass of the main loop (0x00BE). Call it 4.19
    /// times per video frame, with [`Driver::vblank`] once per frame.
    pub fn tick(&mut self) {
        self.irq();
        self.main_step();
    }

    /// DSP writes made since the last [`Driver::take_dsp_writes`], in order.
    pub fn dsp_writes(&self) -> &[RegWrite] {
        &self.dsp
    }

    /// Takes the DSP write log.
    pub fn take_dsp_writes(&mut self) -> Vec<RegWrite> {
        std::mem::take(&mut self.dsp)
    }

    /// The state of the SFX (`music == false`) or music track of voice `v`.
    pub fn track(&self, v: u8, music: bool) -> TrackInfo {
        let t = if music { music_track(v) } else { sfx_track(v) };
        TrackInfo {
            active: self.r8(t.wrapping_add(T_ACTIVE)) != 0,
            ptr: u32::from_le_bytes([
                self.r8(t.wrapping_add(T_PTR)),
                self.r8(t.wrapping_add(T_PTR + 1)),
                self.r8(t.wrapping_add(T_PTR + 2)),
                0,
            ]),
            priority: self.r8(t.wrapping_add(T_PRIO)),
            pan: self.r8(t.wrapping_add(T_PAN)),
            muted: self.r8(t.wrapping_add(T_MUTE)) == 0xFF,
        }
    }

    /// Requests waiting in the 68000 ring (not yet sent to the Z80).
    pub fn pending_requests(&self) -> usize {
        ((self.mail_wr.wrapping_sub(self.mail_rd) & 0x3F0) / 0x10) as usize
    }

    /// Z80 RAM byte (for tests and for the game's debug views).
    pub fn peek(&self, addr: u16) -> u8 {
        self.r8(addr)
    }

    /// Z80 RAM little-endian word.
    pub fn peek_word(&self, addr: u16) -> u16 {
        self.r16(addr)
    }

    /// Writes one Z80 RAM byte (tests only: the game never pokes the driver).
    pub fn poke(&mut self, addr: u16, v: u8) {
        self.w8(addr, v);
    }

    /// Pan setup of a track, for tests of the mode routines.
    pub fn pan_setup_pub(&mut self, t: u16, iy: u16) {
        self.pan_setup(t, iy);
    }

    // ----- memory helpers -------------------------------------------------------------------

    pub(crate) fn r8(&self, a: u16) -> u8 {
        self.ram[a as usize]
    }

    pub(crate) fn w8(&mut self, a: u16, v: u8) {
        self.ram[a as usize] = v;
    }

    /// Little-endian word, the Z80 way for every variable in work RAM.
    pub(crate) fn r16(&self, a: u16) -> u16 {
        u16::from_le_bytes([self.r8(a), self.r8(a.wrapping_add(1))])
    }

    pub(crate) fn w16(&mut self, a: u16, v: u16) {
        let [lo, hi] = v.to_le_bytes();
        self.w8(a, lo);
        self.w8(a.wrapping_add(1), hi);
    }

    /// Big-endian word, the way the request bytes and the sequence offsets are stored.
    pub(crate) fn r16be(&self, a: u16) -> u16 {
        u16::from_be_bytes([self.r8(a), self.r8(a.wrapping_add(1))])
    }

    /// Byte of the sound data image at a linear address (0 past the end).
    pub(crate) fn rom(&self, addr: u32) -> u8 {
        self.snd.data.get(addr as usize).copied().unwrap_or(0)
    }

    /// Reads one byte at `p` and advances the 24-bit pointer (routine 0x09BC).
    pub(crate) fn rom_next(&self, p: &mut u32) -> u8 {
        let b = self.rom(*p);
        *p = p.wrapping_add(1) & 0xFF_FFFF;
        b
    }

    /// Reads a big-endian word at `p` and advances the pointer by two.
    pub(crate) fn rom_next_be16(&self, p: &mut u32) -> u16 {
        let hi = self.rom_next(p);
        let lo = self.rom_next(p);
        u16::from_be_bytes([hi, lo])
    }

    /// Writes a 24-bit track pointer into +00..+02 (lo, mid, bank) of track `t`.
    pub(crate) fn set_track_ptr(&mut self, t: u16, p: u32) {
        let b = p.to_le_bytes();
        self.w8(t.wrapping_add(T_PTR), b[0]);
        self.w8(t.wrapping_add(T_PTR + 1), b[1]);
        self.w8(t.wrapping_add(T_PTR + 2), b[2]);
    }

    pub(crate) fn clear(&mut self, base: u16, n: u16) {
        for i in 0..n {
            self.w8(base.wrapping_add(i), 0);
        }
    }

    // ----- DSP writes -----------------------------------------------------------------------

    /// Helper 0x0A13: one register write.
    pub(crate) fn dsp(&mut self, reg: u8, value: u16) {
        self.dsp.push(RegWrite { reg, value });
    }

    /// 0x0981 for voice `v`: echo send 0xBA+v, volume and pitch of v*8 cleared.
    pub(crate) fn voice_reset(&mut self, v: u8) {
        self.dsp(0xBAu8.wrapping_add(v), 0);
        let base = v.rotate_left(3);
        self.dsp(base.wrapping_add(6), 0);
        self.dsp(base.wrapping_add(2), 0);
    }

    /// 0x0972 then 0x099D: every voice, then the three ADPCM voices.
    pub(crate) fn reset_all_voices(&mut self) {
        for v in 0..VOICES {
            self.voice_reset(v);
        }
        self.adpcm_reset();
    }

    /// 0x099D: reset the three ADPCM voices (unused by this ROM, SPEC 11).
    pub(crate) fn adpcm_reset(&mut self) {
        for n in 0..3u8 {
            self.dsp(0xCDu8.wrapping_add(n.wrapping_mul(4)), 0);
            self.dsp(0xD6u8.wrapping_add(n), 1);
        }
    }

    /// 0x09FE: pan of voice `v`. Mono (F040 == 0) forces the centre 0x10. The register is
    /// 0x80+v and the value 0x110 + pan.
    pub(crate) fn pan_write(&mut self, v: u8, e: u8) {
        let e = self.mono_pan(e);
        self.dsp(0x80u8.wrapping_add(v), 0x110 + e as u16);
    }

    /// 0x09E7: the same pan write, plus the debug copy to C005 when F041 is set.
    pub(crate) fn pan_write_dbg(&mut self, v: u8, e: u8) {
        let e = self.mono_pan(e);
        if self.r8(DEBUG) != 0 {
            self.w8(SHARED_PAN_DEBUG, e);
        }
        self.dsp(0x80u8.wrapping_add(v), 0x110 + e as u16);
    }

    fn mono_pan(&self, e: u8) -> u8 {
        if self.r8(STEREO) == 0 { 0x10 } else { e }
    }

    /// 0x0912 (DSP init), the list of SPEC section 4.
    fn dsp_init(&mut self) {
        self.dsp(0xDE, 0);
        self.dsp(0xDF, 0x2E);
        self.dsp(0xE0, 0);
        self.dsp(0xE1, 0x30);
        for r in [0xE4u8, 0xE5, 0xE6, 0xE7] {
            self.dsp(r, 0x3FFF);
        }
        self.dsp(0xE2, 0xFFFF);
        self.dsp(0xDA, 0x0DB2);
        self.dsp(0xDC, 0x0E11);
        self.dsp(0xE3, 0x0039);
    }

    // ----- boot and 68000 side --------------------------------------------------------------

    /// Z80 reset (0x0000 .. 0x00BE): RAM clear, the boot handshake the 68000 RAM test answers
    /// (CFFF, C00F, CFFD and CFFE become 0xFF, SPEC 3), DSP init, master volume F016.
    fn z80_boot(&mut self) {
        for a in 0xF000u32..=0xFFFF {
            self.ram[a as usize] = 0;
        }
        self.w8(SHARED_BOOT, 0xFF);
        self.w8(SHARED_STATUS, 0xFF);
        self.w8(SHARED_ENABLE, 0xFF);
        self.dsp_init();
        self.reset_all_voices();
        // 0x00A9: F016 = table 0x24A9 at index byte 0x8002 (words).
        let idx = self.rom(0x8002) as u32;
        let master = self.snd.word_le(0x24A9 + 2 * idx);
        self.w16(MASTER_VOLUME, master);
    }

    /// 0xA5FA: enable the request copy (0x88 in CFFD), empty the ring, send 0xFF0B, then the
    /// stop of everything (boot order of SPEC 3).
    fn m68k_init(&mut self) {
        self.w8(SHARED_ENABLE, ENABLE_VALUE);
        self.mail_wr = 0;
        self.mail_rd = 0;
        self.command(0xFF0B, 0, 0);
        self.command(0xFF00, 0, 0);
    }

    // ----- Z80 interrupt and main loop ------------------------------------------------------

    /// 0x0038: F004++, then copy the request (C000..C00F) into the Z80 ring when the 68000 has
    /// left one and the request copy is enabled. F040 follows CFFE while enabled.
    fn irq(&mut self) {
        let n = self.r8(IRQ_COUNT).wrapping_add(1);
        self.w8(IRQ_COUNT, n);
        if self.r8(SHARED_ENABLE) == ENABLE_VALUE {
            if self.r8(SHARED_STATUS) != 0xFF {
                let wr = self.r8(RING_WR);
                for i in 0..0x10u16 {
                    let b = self.r8(SHARED_REQ + i);
                    self.w8(RING + wr as u16 + i, b);
                }
                self.w8(RING_WR, wr.wrapping_add(0x10));
                self.w8(SHARED_STATUS, 0xFF);
            }
            let stereo = self.r8(SHARED_STEREO);
            self.w8(STEREO, stereo);
        }
    }

    /// One pass of the main loop: F005++, one request (0x017D), and the sequencer every 4th
    /// tick (62.5 Hz).
    fn main_step(&mut self) {
        let n = self.r8(MAIN_COUNT).wrapping_add(1);
        self.w8(MAIN_COUNT, n);
        self.dispatch();
        if self.r8(MAIN_COUNT) & 3 == 0 {
            self.sequencer_step();
        }
    }


    /// 0x017D: take one request from the Z80 ring and start it.
    fn dispatch(&mut self) {
        let wr = self.r8(RING_WR);
        let rd = self.r8(RING_RD);
        if rd == wr {
            return;
        }
        self.w8(RING_RD, rd.wrapping_add(0x10));
        let iy = RING + rd as u16;
        if self.r8(iy) == 0xFF {
            self.special(iy);
            return;
        }
        // Count N at linear 0x8000 (big-endian), code = code mod N, entry at 0x8006 + 4*index.
        let n = u16::from_be_bytes([self.rom(0x8000), self.rom(0x8001)]);
        let code = u16::from_be_bytes([self.r8(iy), self.r8(iy + 1)]);
        if n == 0 {
            return;
        }
        let index = (code % n) as u32;
        let e = 0x8006 + 4 * index;
        let bank = self.rom(e);
        let hi = self.rom(e + 1);
        let lo = self.rom(e + 2);
        let hdr = (bank as u32) << 16 | (hi as u32) << 8 | lo as u32;
        if hdr == 0 {
            return;
        }
        self.w8(HDR_BANK, bank);
        self.w8(HDR_PTR, lo);
        self.w8(HDR_PTR + 1, hi);
        let mut p = hdr;
        let h = self.rom_next(&mut p);
        match h {
            0 => self.music_start(hdr, p),
            1..=0x7F => self.sfx_start(hdr, p, h, iy),
            // 0x80 and up: ADPCM one-shot. This ROM has no such header (SPEC 5, 11).
            _ => {}
        }
    }

    /// Special commands 0xFFxx (0x0782 and the table at 0x0796). `iy` is the request.
    fn special(&mut self, iy: u16) {
        let sub = self.r8(iy + 1);
        let c003 = self.r8(iy + 3);
        let c004 = self.r8(iy + 4);
        match sub {
            // 0x07AE: stop everything. Clears all tracks and the first fade.
            0 => {
                self.clear(SFX_TRACKS, 0x860);
                self.w8(FADE1_ON, 0);
                self.reset_all_voices();
            }
            // 0x07C2: stop every SFX track.
            1 => {
                self.w8(VOICE_OWNED, 0);
                self.w8(SEQ_FLAG, 0);
                for v in 0..VOICES {
                    self.set_voice(v);
                    let t = sfx_track(v);
                    if self.r8(t.wrapping_add(T_ACTIVE)) != 0 {
                        self.stop_sfx(t, v);
                    }
                }
            }
            // 0x07EF: stop the SFX track of voice C004.
            2 => {
                let v = c004;
                self.set_voice(v);
                let t = SFX_TRACKS.wrapping_add(0x40u16.wrapping_mul(v as u16));
                if self.r8(t.wrapping_add(T_ACTIVE)) != 0 {
                    self.stop_sfx(t, v);
                }
            }
            // 0x0819: clear the ADPCM slots, then reset the ADPCM voices.
            3 => {
                self.clear(0xFA00, 0x60);
                self.adpcm_reset();
            }
            // 0x0829: clear the ADPCM slot C004, then reset voice C004 (SPEC 5 as written).
            4 => {
                let base = 0x20u16.wrapping_mul(c004 as u16);
                self.clear(0xFA00u16.wrapping_add(base), 0x20);
                self.voice_reset(c004);
            }
            // 0x0845: stop the music.
            5 => self.stop_music(),
            // 0x087F and 0x08A1: music fades, speed C003:C004.
            6 => self.fade(FADE1_ON, FADE1_SPEED, FADE1_LEVEL, c003, c004),
            7 => self.fade(FADE2_ON, FADE2_SPEED, FADE2_LEVEL, c003, c004),
            // 0x08C3: tempo offset = (C004 * 0x369) >> 8, negated when C003 is 0xFF.
            8 => {
                let mut v = ((c004 as u32 * 0x369) >> 8) as u16;
                if c003 == 0xFF {
                    v = 0u16.wrapping_sub(v);
                }
                self.w16(TEMPO_OFFSET, v);
            }
            // 0x08E1: pitch offset = C004 << 4, negated when C003 is 0xFF.
            9 => {
                let mut v = (c004 as u16) << 4;
                if c003 == 0xFF {
                    v = 0u16.wrapping_sub(v);
                }
                self.w16(PITCH_OFFSET, v);
            }
            // 0x0907 and 0x090D: debug flag (0xFF0A sets it, 0xFF0B clears it).
            10 => self.w8(DEBUG, 0xFF),
            11 => self.w8(DEBUG, 0),
            _ => {}
        }
    }

    /// Sets F00D and F00E for voice `v` (the handler reads them).
    pub(crate) fn set_voice(&mut self, v: u8) {
        self.w8(VOICE, v);
        self.w8(VOICE8, v.rotate_left(3));
    }

    /// 0x07AE .. 0x087E fade start: speed 0 cancels; a running fade is not restarted.
    fn fade(&mut self, on: u16, speed_addr: u16, level: u16, hi: u8, lo: u8) {
        let speed = u16::from_be_bytes([hi, lo]);
        if speed == 0 {
            self.w8(on, 0);
            return;
        }
        if self.r8(on) != 0 {
            return;
        }
        self.w8(on, 0xFF);
        self.w16(speed_addr, speed);
        self.w16(level, 0xFFFF);
    }

    /// 0x0845: stop every music track. A voice with no SFX on it is reset.
    fn stop_music(&mut self) {
        for v in 0..VOICES {
            self.set_voice(v);
            let m = music_track(v);
            if self.r8(m.wrapping_add(T_ACTIVE)) != 0 {
                self.w8(m.wrapping_add(T_ACTIVE), 0);
                let s = sfx_track(v);
                if self.r8(s.wrapping_add(T_ACTIVE)) == 0 {
                    self.voice_reset(v);
                }
            }
        }
    }

    /// 0x0EBB: stop the SFX track `t` on voice `v`. A music track on the voice gets its sample
    /// back (0x0D5E), its pan, and then either the tied-note resume (0x0EF0, bit 7 of its flags)
    /// or the unmute at the next key-on.
    pub(crate) fn stop_sfx(&mut self, t: u16, v: u8) {
        self.w8(t.wrapping_add(T_ACTIVE), 0);
        self.w8(t.wrapping_add(T_PRIO), 0);
        self.set_voice(v);
        self.voice_reset(v);
        let m = t.wrapping_add(0x400);
        if self.r8(m.wrapping_add(T_ACTIVE)) != 0 {
            let idx = self.r8(m.wrapping_add(T_ACTIVE)).wrapping_sub(1);
            self.sample_load(m, idx);
            let pan = self.r8(m.wrapping_add(T_PAN));
            self.pan_write(v, pan);
            if self.r8(m.wrapping_add(T_FLAGS)) & 0x80 != 0 {
                self.tie_resume(m, v);
            } else {
                self.w8(m.wrapping_add(T_MUTE), 0xFF);
            }
        }
    }

    /// 0x0251 .. 0x0245: start an effect. `hdr` is the header, `p` the position after the
    /// header byte `prio`, `iy` the request in the Z80 ring.
    fn sfx_start(&mut self, hdr: u32, mut p: u32, prio: u8, iy: u16) {
        self.w8(PRIO, prio);
        for v in 0..VOICES {
            self.w8(VOICE, v);
            let off = self.rom_next_be16(&mut p);
            let t = sfx_track(v);
            if off == 0 {
                continue;
            }
            // A new effect replaces a voice only when its priority is at least the current one.
            if prio < self.r8(t.wrapping_add(T_PRIO)) {
                continue;
            }
            self.clear(t, TRACK_SIZE);
            self.set_track_ptr(t, hdr.wrapping_add(off as u32) & 0xFF_FFFF);
            self.w8(t.wrapping_add(T_PRIO), prio);
            self.w8(t.wrapping_add(T_ACTIVE), 0xFF);
            self.w8(t.wrapping_add(T_LFO_FLAGS), 1);
            self.w8(t.wrapping_add(T_ENV_STATE), 5);
            self.pan_setup(t, iy);
            self.voice_reset(v);
            let pan = self.r8(t.wrapping_add(T_PAN));
            self.pan_write_dbg(v, pan);
        }
    }

    /// 0x0449 .. 0x04F1: start a music piece. The global tempo, transpose, fade 2 and pitch
    /// offsets are cleared; F018 = F016 * 0xFFFE >> 16. A new music code does not clear F030.
    fn music_start(&mut self, hdr: u32, mut p: u32) {
        self.w16(0xF012, 0);
        self.w8(0xF01A, 0);
        self.w8(FADE2_ON, 0);
        self.w16(TEMPO_OFFSET, 0);
        self.w16(PITCH_OFFSET, 0);
        let master = self.r16(MASTER_VOLUME) as u32;
        let vol = ((master * 0xFFFE) >> 16) as u16;
        self.w16(MUSIC_VOLUME, vol);
        for v in 0..VOICES {
            self.w8(VOICE, v);
            let off = self.rom_next_be16(&mut p);
            let m = music_track(v);
            if off != 0 {
                self.clear(m, 0x30);
                self.set_track_ptr(m, hdr.wrapping_add(off as u32) & 0xFF_FFFF);
                self.w8(m.wrapping_add(T_ACTIVE), 0xFF);
                self.w8(m.wrapping_add(T_LFO_FLAGS), 1);
                self.w8(m.wrapping_add(T_ENV_STATE), 5);
                self.w8(m.wrapping_add(T_PAN), 0x10);
            } else {
                let was = self.r8(m.wrapping_add(T_ACTIVE));
                self.w8(m.wrapping_add(T_ACTIVE), 0);
                if was == 0 {
                    continue;
                }
            }
            // 0x04CA: a voice with no effect on it is reset and centred.
            let s = sfx_track(v);
            if self.r8(s.wrapping_add(T_ACTIVE)) == 0 {
                self.voice_reset(v);
                self.pan_write(v, 0x10);
            }
        }
    }
}
