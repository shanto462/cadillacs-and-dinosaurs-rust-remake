//! Z80 RAM map and track field offsets (SPEC sections 2, 4 and 6).
//!
//! Some names are used only by the parts still to come (M4-04 to M4-06), hence the allow.
#![allow(dead_code)]

//! Offsets written in the SPEC as `+NN` are hex. The Z80 listing shows the same values in
//! decimal (`ix+50` is `+32`), so every constant here is the hex value.

/// Number of voices (QSound channels).
pub const VOICES: u8 = 16;
/// Bytes per track (SFX and music).
pub const TRACK_SIZE: u16 = 0x40;

// Shared RAM with the 68000 (68000 address 0xF18000 + 2*n + 1 = Z80 0xC000 + n).
/// Request bytes: code hi, code lo, (unused), (unused), C003, C004, mode/pan/spread, speed, speed2.
pub const SHARED_REQ: u16 = 0xC000;
/// C005: copy of the pan when the debug flag is on.
pub const SHARED_PAN_DEBUG: u16 = 0xC005;
/// CFFD: 0x88 turns the request copy on (commands enabled).
pub const SHARED_ENABLE: u16 = 0xCFFD;
/// CFFE: stereo flag (0xFF stereo, 0 mono), copied to F040 while enabled.
pub const SHARED_STEREO: u16 = 0xCFFE;
/// CFFF: boot handshake byte.
pub const SHARED_BOOT: u16 = 0xCFFF;
/// C00F: mailbox state. 0xFF = free for the 68000 to write, 0 = full (the Z80 copies it).
pub const SHARED_STATUS: u16 = 0xC00F;
/// Value of [`SHARED_ENABLE`] that turns the request copy on.
pub const ENABLE_VALUE: u8 = 0x88;

// Z80 work RAM (0xF000 .. 0xFFFF).
/// F004: IRQ counter (250 Hz).
pub const IRQ_COUNT: u16 = 0xF004;
/// F005: main loop counter (catches up with F004).
pub const MAIN_COUNT: u16 = 0xF005;
/// F006: sequencer flag used by the track step (set by the sequencer, M4-04).
pub const SEQ_FLAG: u16 = 0xF006;
/// F020: 0xFF while the SFX on the current voice owns it (music is masked, M4-06).
pub const VOICE_OWNED: u16 = 0xF020;
/// F007: ring write index (IRQ side, 0x10 steps).
pub const RING_WR: u16 = 0xF007;
/// F008: ring read index (main side, 0x10 steps).
pub const RING_RD: u16 = 0xF008;
/// F009: bank of the header of the current request.
pub const HDR_BANK: u16 = 0xF009;
/// F00A: low 16 bits of the header pointer of the current request.
pub const HDR_PTR: u16 = 0xF00A;
/// F00C: priority of the current request (the header byte).
pub const PRIO: u16 = 0xF00C;
/// F00D: voice of the track being handled.
pub const VOICE: u16 = 0xF00D;
/// F00E: voice times 8 (the register base of the voice).
pub const VOICE8: u16 = 0xF00E;
/// F016: music master volume (word, from table 0x24A9).
pub const MASTER_VOLUME: u16 = 0xF016;
/// F018: music volume after the master volume and the fixed 0xFFFE factor (word).
pub const MUSIC_VOLUME: u16 = 0xF018;
/// F014: global tempo offset (word, command 08).
pub const TEMPO_OFFSET: u16 = 0xF014;
/// F01B: global pitch offset (word, command 09).
pub const PITCH_OFFSET: u16 = 0xF01B;
/// F030: first fade active (byte, 0xFF while a fade runs).
pub const FADE1_ON: u16 = 0xF030;
/// F031: first fade speed (word).
pub const FADE1_SPEED: u16 = 0xF031;
/// F033: first fade level (word, 0xFFFF at the start, counts down).
pub const FADE1_LEVEL: u16 = 0xF033;
/// F035: second fade active (byte).
pub const FADE2_ON: u16 = 0xF035;
/// F036: second fade speed (word).
pub const FADE2_SPEED: u16 = 0xF036;
/// F038: second fade level (word).
pub const FADE2_LEVEL: u16 = 0xF038;
/// F040: stereo flag copied from CFFE (0 = mono, pan forced to 0x10).
pub const STEREO: u16 = 0xF040;
/// F041: debug flag (command 0xFF0A sets it, 0xFF0B clears it).
pub const DEBUG: u16 = 0xF041;
/// F100..F1FF: 16-entry request ring, 16 bytes each (SPEC section 4).
pub const RING: u16 = 0xF100;
/// F200: SFX tracks, 16 x 0x40 bytes.
pub const SFX_TRACKS: u16 = 0xF200;
/// F600: music tracks, 16 x 0x40 bytes.
pub const MUSIC_TRACKS: u16 = 0xF600;

/// Address of SFX track of voice `v`.
pub fn sfx_track(v: u8) -> u16 {
    SFX_TRACKS.wrapping_add(TRACK_SIZE.wrapping_mul(v as u16))
}

/// Address of music track of voice `v`.
pub fn music_track(v: u8) -> u16 {
    MUSIC_TRACKS.wrapping_add(TRACK_SIZE.wrapping_mul(v as u16))
}

// Track fields (`ix+N`, N in hex).
/// +00 .. +02: pointer to the next byte of the sequence (lo, mid, bank).
pub const T_PTR: u16 = 0x00;
/// +03: flags (bit 7 = tie active).
pub const T_FLAGS: u16 = 0x03;
/// +0F: echo send index.
pub const T_ECHO: u16 = 0x0F;
/// +17: LFO flags (bit 0 = reset on key-on).
pub const T_LFO_FLAGS: u16 = 0x17;
/// +20: active (0 = free, 0xFF = running).
pub const T_ACTIVE: u16 = 0x20;
/// +21 .. +22: sample start (lo, hi).
pub const T_SAMPLE_START: u16 = 0x21;
/// +25: envelope state (5 = off).
pub const T_ENV_STATE: u16 = 0x25;
/// +30 .. +31: SFX tempo (word).
pub const T_TEMPO: u16 = 0x30;
/// +32: priority.
pub const T_PRIO: u16 = 0x32;
/// +33: mute of volume and pitch writes until the next key-on (0xFF = muted).
pub const T_MUTE: u16 = 0x33;
/// +36: pan motion mode + 1 (0 = no motion).
pub const T_PAN_MODE: u16 = 0x36;
/// +37: pan (0..0x20, centre 0x10).
pub const T_PAN: u16 = 0x37;
/// +38: pan fraction (low byte of the 16-bit pan position).
pub const T_PAN_FRAC: u16 = 0x38;
/// +39: spread.
pub const T_SPREAD: u16 = 0x39;
/// +3A: pan limit (the sweep target).
pub const T_PAN_LIMIT: u16 = 0x3A;
/// +3B .. +3C: pan speed (word, lo then hi).
pub const T_PAN_SPEED: u16 = 0x3B;
/// +3D .. +3E: second pan speed (word, lo then hi).
pub const T_PAN_SPEED2: u16 = 0x3D;
/// +3F: start delay of the pan motion (steps).
pub const T_DELAY: u16 = 0x3F;

// Sequencer (M4-04). Work RAM.
/// F010: tempo of the track being stepped (word, LE). Set before each track step.
pub const TEMPO_WORD: u16 = 0xF010;
/// F012: music tempo (word, LE, command 05 of a music track).
pub const MUSIC_TEMPO: u16 = 0xF012;
/// F01A: music transpose (command 0A of a music track).
pub const MUSIC_TRANSPOSE: u16 = 0xF01A;

// Track fields used by the sequencer (`ix+N` in the listing is decimal, the SPEC is hex).
/// +04 .. +05: duration counter (8.8, LE).
pub const T_DUR: u16 = 0x04;
/// +06: gate ratio (command 06).
pub const T_GATE_RATIO: u16 = 0x06;
/// +07 .. +08: gate counter (LE).
pub const T_GATE: u16 = 0x07;
/// +09: transpose (command 0B).
pub const T_TRANSPOSE: u16 = 0x09;
/// +0A: detune (command 0C).
pub const T_DETUNE: u16 = 0x0A;
/// +0B: sample root key.
pub const T_ROOT: u16 = 0x0B;
/// +0C: instrument fine tune.
pub const T_FINE: u16 = 0x0C;
/// +0D .. +0E: channel volume word (command 07).
pub const T_VOLUME: u16 = 0x0D;
/// +10: portamento speed (bit 7 = sliding).
pub const T_PORTA: u16 = 0x10;
/// +11 .. +12: slide step (BE in this port, as the Z80 stores d then e).
pub const T_SLIDE: u16 = 0x11;
/// +13 .. +14: current pitch (hi, lo).
pub const T_PITCH: u16 = 0x13;
/// +15 .. +16: target pitch (hi, lo).
pub const T_TARGET: u16 = 0x15;
/// +18 .. +19: LFO speed word.
pub const T_LFO_SPEED: u16 = 0x18;
/// +1A .. +1B: LFO phase word.
pub const T_LFO_PHASE: u16 = 0x1A;
/// +1C .. +1D: vibrato depth word.
pub const T_VIB: u16 = 0x1C;
/// +1E .. +1F: tremolo depth word.
pub const T_TREM: u16 = 0x1E;
/// +23 .. +24: envelope pointer (instrument byte 2).
pub const T_ENV_PTR: u16 = 0x23;
/// +26 .. +27: sustain level word.
pub const T_SUSTAIN: u16 = 0x26;
/// +28 .. +29: envelope rate word.
pub const T_ENV_RATE: u16 = 0x28;
/// +2A .. +2B: envelope level word.
pub const T_ENV_LEVEL: u16 = 0x2A;
/// +2C .. +2F: loop counters of commands 0E..15.
pub const T_LOOP: u16 = 0x2C;

// Constant tables of the Z80 program (linear addresses, read through the data image).
pub const TAB_LEN_TRIPLET: u32 = 0x0C50;
pub const TAB_LEN_NORMAL: u32 = 0x0C57;
pub const TAB_LEN_DOTTED: u32 = 0x0C5E;
pub const TAB_NOTE_BASE: u32 = 0x0C65;
pub const TAB_VOLUME: u32 = 0x23A9;
pub const TAB_ENV_ATTACK: u32 = 0x25A9;
pub const TAB_ENV_DECAY: u32 = 0x2629;
pub const TAB_GAIN: u32 = 0x24A9;
pub const TAB_LFO_SPEED: u32 = 0x26A9;
pub const TAB_VIBRATO: u32 = 0x27A9;
pub const TAB_TREMOLO: u32 = 0x28A9;
pub const TAB_SAMPLE: u32 = 0x6A82;
pub const TAB_INSTRUMENT: u32 = 0x6E62;
