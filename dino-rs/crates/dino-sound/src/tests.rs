//! Tests of the mailbox, the command rules, the priorities, the special commands and the pan
//! setup. Most tests use a small synthetic sound image (made here, no ROM bytes). The last two
//! read the real `dino.zip` through `dino-extract` and skip when it is not there.

use std::path::{Path, PathBuf};

use dino_assets::sound::SoundData;

use crate::driver::Driver;
use crate::map::*;

/// Command count N (the big-endian word at 0x8000) of the real ROM.
const N: u16 = 0x106;

/// Builds a sound image with `N` command slots. Each entry in `headers` is
/// `(code, header byte, voices)`: the header holds the byte and a BE offset per voice
/// (0x40 + 0x10 * v for a voice in the list, 0 otherwise).
fn synth(headers: &[(u16, u8, &[u8])]) -> SoundData {
    let mut data = vec![0u8; 0x2_0000];
    data[0x8000..0x8002].copy_from_slice(&N.to_be_bytes());
    for code in 0..N {
        let hdr = header_addr(code);
        let e = 0x8006 + 4 * code as usize;
        data[e] = (hdr >> 16) as u8;
        data[e + 1] = (hdr >> 8) as u8;
        data[e + 2] = hdr as u8;
    }
    for &(code, h, voices) in headers {
        let hdr = header_addr(code) as usize;
        data[hdr] = h;
        for v in 0..16usize {
            let off: u16 = if voices.contains(&(v as u8)) { 0x40 + 0x10 * v as u16 } else { 0 };
            data[hdr + 1 + 2 * v..hdr + 3 + 2 * v].copy_from_slice(&off.to_be_bytes());
        }
    }
    SoundData {
        data,
        ..Default::default()
    }
}

fn header_addr(code: u16) -> u32 {
    0x1_0000 + 0x40 * code as u32
}

/// Sends one request the way the game does: queue, vblank flush, one tick (the IRQ copy and
/// the main loop take it at once).
fn send(d: &mut Driver, code: u16, d1: u32, d2: u32) {
    d.command(code, d1, d2);
    d.vblank();
    d.tick();
}

/// Lets the boot requests (0xFF0B, 0xFF00) go through.
fn settle(d: &mut Driver) {
    for _ in 0..4 {
        d.vblank();
        d.tick();
    }
}

/// Request with a pan and speed for the mode tests: d1 = mode<<16 | spread<<8 | pan,
/// d2 = speed<<16 | speed2.
fn req(mode: u8, pan: u8, spread: u8, speed: u16, speed2: u16) -> (u32, u32) {
    let d1 = (mode as u32) << 16 | (spread as u32) << 8 | pan as u32;
    let d2 = (speed as u32) << 16 | speed2 as u32;
    (d1, d2)
}

fn rom_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dino.zip")
}

#[test]
fn mailbox_holds_63_requests_and_keeps_the_64th_uncommitted() {
    let snd = synth(&[]);
    let mut d = Driver::new(&snd, true);
    settle(&mut d);
    assert_eq!(d.pending_requests(), 0);
    for i in 0..63u16 {
        d.command(0x0100 + i, 0, 0);
    }
    assert_eq!(d.pending_requests(), 63);
    d.command(0x0200, 0, 0);
    assert_eq!(d.pending_requests(), 63, "a full ring does not move the write index");
}

#[test]
fn flush_sends_one_request_per_free_status() {
    let snd = synth(&[]);
    let mut d = Driver::new(&snd, true);
    settle(&mut d);
    d.command(0x0030, 0, 0);
    d.command(0x0031, 0, 0);
    d.vblank();
    assert_eq!(d.pending_requests(), 1);
    // Status is 0 (full) until the Z80 copies it, so a second flush sends nothing.
    d.vblank();
    assert_eq!(d.pending_requests(), 1);
    assert_eq!(d.peek(SHARED_STATUS), 0);
}

#[test]
fn normal_request_layout_in_shared_ram() {
    let snd = synth(&[]);
    let mut d = Driver::new(&snd, true);
    settle(&mut d);
    d.command(0x0030, 0x0012_3456, 0x1122_3344);
    d.vblank();
    assert_eq!(d.peek(0xC000), 0x00);
    assert_eq!(d.peek(0xC001), 0x30);
    assert_eq!(d.peek(0xC006), 0x12, "d1 bits 23-16");
    assert_eq!(d.peek(0xC007), 0x56, "d1 bits 7-0");
    assert_eq!(d.peek(0xC008), 0x34, "d1 bits 15-8");
    assert_eq!([d.peek(0xC009), d.peek(0xC00A), d.peek(0xC00B), d.peek(0xC00C)], [0x11, 0x22, 0x33, 0x44]);
    assert_eq!(d.peek(SHARED_STATUS), 0);
}

#[test]
fn special_request_layout_in_shared_ram() {
    let snd = synth(&[]);
    let mut d = Driver::new(&snd, true);
    settle(&mut d);
    d.command(0xFF07, 0x0444_0000, 0);
    d.vblank();
    assert_eq!(d.peek(0xC000), 0xFF);
    assert_eq!(d.peek(0xC001), 0x07);
    assert_eq!(d.peek(0xC003), 0x04);
    assert_eq!(d.peek(0xC004), 0x44);
    assert_eq!(d.peek(SHARED_STATUS), 0);
}

#[test]
fn priority_rule_replaces_on_equal_or_higher_only() {
    // A: prio 5 on voices 0 and 1. B: prio 4 on 0. C: prio 5 on 0. D: prio 3 on 1. E: prio 6 on 0.
    let snd = synth(&[
        (0x10, 5, &[0, 1]),
        (0x11, 4, &[0]),
        (0x12, 5, &[0]),
        (0x13, 3, &[1]),
        (0x14, 6, &[0]),
    ]);
    let mut d = Driver::new(&snd, true);
    settle(&mut d);
    let (d1, d2) = (0, 0);
    send(&mut d, 0x10, d1, d2);
    let a_ptr = d.track(0, false).ptr;
    assert_eq!((d.track(0, false).priority, d.track(1, false).priority), (5, 5));
    assert_eq!(a_ptr, header_addr(0x10) + 0x40);

    send(&mut d, 0x11, d1, d2);
    assert_eq!(d.track(0, false).ptr, a_ptr, "prio 4 does not replace prio 5");

    send(&mut d, 0x12, d1, d2);
    assert_eq!(d.track(0, false).ptr, header_addr(0x12) + 0x40, "prio 5 replaces prio 5");
    assert_eq!(d.track(0, false).priority, 5);

    send(&mut d, 0x13, d1, d2);
    assert_eq!(d.track(1, false).priority, 5, "prio 3 does not replace prio 5 on voice 1");

    send(&mut d, 0x14, d1, d2);
    assert_eq!(d.track(0, false).priority, 6, "prio 6 replaces on voice 0");
    assert_eq!(d.track(1, false).ptr, header_addr(0x10) + 0x40 + 0x10, "voice 1 keeps A");
}

#[test]
fn stop_all_clears_tracks_fades_and_resets_voices() {
    let snd = synth(&[(0x10, 5, &[0, 1]), (0x01, 0, &[0, 2])]);
    let mut d = Driver::new(&snd, true);
    settle(&mut d);
    send(&mut d, 0x10, 0, 0);
    send(&mut d, 0x01, 0, 0);
    assert!(d.track(0, false).active && d.track(1, false).active);
    assert!(d.track(0, true).active && d.track(2, true).active);
    send(&mut d, 0xFF07, 0x0444_0000, 0);
    d.take_dsp_writes();
    send(&mut d, 0xFF00, 0, 0);
    for v in 0..16 {
        assert!(!d.track(v, false).active && !d.track(v, true).active, "voice {v}");
    }
    assert_eq!(d.peek(FADE1_ON), 0);
    let writes = d.take_dsp_writes();
    // Every voice is reset (echo, volume, pitch): 16 * 3 writes, then 3 ADPCM voice resets.
    let echo_resets = writes.iter().filter(|w| (0xBA..0xCA).contains(&w.reg) && w.value == 0).count();
    assert_eq!(echo_resets, 16);
}

#[test]
fn stop_music_keeps_sfx_and_resets_only_free_voices() {
    let snd = synth(&[(0x01, 0, &[0, 1]), (0x10, 5, &[1])]);
    let mut d = Driver::new(&snd, true);
    settle(&mut d);
    send(&mut d, 0x01, 0, 0);
    send(&mut d, 0x10, 0, 0);
    send(&mut d, 0xFF05, 0, 0);
    assert!(!d.track(0, true).active && !d.track(1, true).active);
    assert!(d.track(1, false).active, "the effect is not stopped by 0xFF05");
}

#[test]
fn sfx_stop_unmutes_music_on_its_voice() {
    let snd = synth(&[(0x01, 0, &[0]), (0x10, 3, &[0])]);
    let mut d = Driver::new(&snd, true);
    settle(&mut d);
    send(&mut d, 0x01, 0, 0);
    send(&mut d, 0x10, 0, 0);
    assert!(d.track(0, true).active);
    // 0xFF02 with C004 = voice 0 stops the effect on that voice.
    send(&mut d, 0xFF02, 0, 0);
    assert!(!d.track(0, false).active);
    assert!(d.track(0, true).active && d.track(0, true).muted, "music is muted until its next key-on");
}

#[test]
fn fade_starts_once_and_zero_speed_cancels() {
    let snd = synth(&[]);
    let mut d = Driver::new(&snd, true);
    settle(&mut d);
    // 0xFF07 is sub-command 7: the second fade (F035 / F036 / F038, music volume).
    send(&mut d, 0xFF07, 0x0444_0000, 0);
    assert_eq!(d.peek(FADE2_ON), 0xFF);
    assert_eq!(d.r16_for_test(FADE2_SPEED), 0x0444);
    assert_eq!(d.r16_for_test(FADE2_LEVEL), 0xFFFF);
    send(&mut d, 0xFF07, 0x0100_0000, 0);
    assert_eq!(d.r16_for_test(FADE2_SPEED), 0x0444, "a running fade is not restarted");
    send(&mut d, 0xFF07, 0, 0);
    assert_eq!(d.peek(FADE2_ON), 0, "speed 0 cancels");
    // 0xFF06 is the first fade (F030 / F031 / F033, all tracks).
    send(&mut d, 0xFF06, 0x0200_0000, 0);
    assert_eq!(d.peek(FADE1_ON), 0xFF);
    assert_eq!(d.r16_for_test(FADE1_SPEED), 0x0200);
    assert_eq!(d.r16_for_test(FADE1_LEVEL), 0xFFFF);
}

#[test]
fn tempo_pitch_and_debug_commands() {
    let snd = synth(&[]);
    let mut d = Driver::new(&snd, true);
    settle(&mut d);
    // Command 8: (0x40 * 0x369) >> 8 = 218, and -218 with C003 = 0xFF.
    send(&mut d, 0xFF08, 0x0040_0000, 0);
    assert_eq!(d.r16_for_test(TEMPO_OFFSET), 218);
    send(&mut d, 0xFF08, 0xFF40_0000, 0);
    assert_eq!(d.r16_for_test(TEMPO_OFFSET), 0u16.wrapping_sub(218));
    // Command 9: C004 << 4, negated with C003 = 0xFF.
    send(&mut d, 0xFF09, 0x0005_0000, 0);
    assert_eq!(d.r16_for_test(PITCH_OFFSET), 0x50);
    send(&mut d, 0xFF09, 0xFF05_0000, 0);
    assert_eq!(d.r16_for_test(PITCH_OFFSET), 0u16.wrapping_sub(0x50));
    send(&mut d, 0xFF0A, 0, 0);
    assert_eq!(d.peek(DEBUG), 0xFF);
    send(&mut d, 0xFF0B, 0, 0);
    assert_eq!(d.peek(DEBUG), 0);
}

#[test]
fn special_commands_with_unknown_sub_do_nothing() {
    let snd = synth(&[]);
    let mut d = Driver::new(&snd, true);
    settle(&mut d);
    send(&mut d, 0xFF0C, 0, 0);
    send(&mut d, 0xFF20, 0, 0);
    assert_eq!(d.peek(FADE1_ON), 0);
    assert_eq!(d.peek(DEBUG), 0);
}

#[test]
fn mono_forces_centre_pan() {
    let snd = synth(&[(0x10, 5, &[0])]);
    let mut d = Driver::new(&snd, false);
    settle(&mut d);
    d.take_dsp_writes();
    send(&mut d, 0x10, 0x0000_0018, 0);
    assert_eq!(d.track(0, false).pan, 0x18, "the track keeps its pan");
    assert!(d.dsp_writes().iter().any(|w| w.reg == 0x80 && w.value == 0x120));
}

#[test]
fn pan_write_uses_register_80_plus_voice() {
    let snd = synth(&[(0x10, 5, &[3])]);
    let mut d = Driver::new(&snd, true);
    settle(&mut d);
    d.take_dsp_writes();
    // Pan 0x18 on voice 3 (the SFX pan write of 0x0251 / 0x09E7).
    send(&mut d, 0x10, 0x0000_0018, 0);
    let writes = d.take_dsp_writes();
    assert!(writes.contains(&crate::RegWrite { reg: 0x83, value: 0x128 }), "{writes:?}");
}

/// Test-only access to the private helpers (this module is inside the crate).
impl<'a> Driver<'a> {
    fn r16_for_test(&self, a: u16) -> u16 {
        self.peek_word(a)
    }
    fn poke_for_test(&mut self, a: u16, v: u8) {
        self.poke(a, v)
    }
    fn pan_setup_for_test(&mut self, t: u16, iy: u16) {
        self.pan_setup_pub(t, iy)
    }
}

/// Pan setup of each mode, checked against the table of SPEC 9.2. The request is written
/// into ring slot F100 and the track is SFX voice 0.
fn setup(mode: u8, pan: u8, spread: u8, speed: u16, speed2: u16) -> Driver<'static> {
    // The synthetic image is leaked: the driver borrows it for the test.
    let snd: &'static SoundData = Box::leak(Box::new(synth(&[])));
    let mut d = Driver::new(snd, true);
    settle(&mut d);
    // sfx_start has set the priority and the active flag before the pan setup runs.
    d.poke_for_test(sfx_track(0) + T_PRIO, 5);
    d.poke_for_test(sfx_track(0) + T_ACTIVE, 0xFF);
    let (d1, d2) = req(mode, pan, spread, speed, speed2);
    let iy = RING_SLOT;
    d.poke_for_test(iy + 6, (d1 >> 16) as u8);
    d.poke_for_test(iy + 7, pan);
    d.poke_for_test(iy + 8, spread);
    d.poke_for_test(iy + 9, (d2 >> 24) as u8);
    d.poke_for_test(iy + 10, (d2 >> 16) as u8);
    d.poke_for_test(iy + 11, (d2 >> 8) as u8);
    d.poke_for_test(iy + 12, d2 as u8);
    d.poke_for_test(PRIO, 5);
    d.poke_for_test(VOICE, 0);
    d.pan_setup_for_test(sfx_track(0), iy);
    d
}

const RING_SLOT: u16 = 0xF100;

#[test]
fn pan_setup_mode_0_is_fixed_and_clamped() {
    let d = setup(0, 0x25, 0, 0x1500, 0);
    let t = sfx_track(0);
    assert_eq!(d.peek(t + T_PAN_MODE), 0);
    assert_eq!(d.peek(t + T_PAN), 0x20);
}

#[test]
fn pan_setup_mode_1_bounces_and_negates_from_0x11() {
    let d = setup(1, 0x10, 0, 0x0100, 0);
    let t = sfx_track(0);
    assert_eq!((d.peek(t + T_PAN), d.r16_for_test(t + T_PAN_SPEED)), (0x10, 0x0100));
    let d = setup(1, 0x18, 0, 0x0100, 0);
    assert_eq!((d.peek(t + T_PAN), d.r16_for_test(t + T_PAN_SPEED)), (0x18, 0xFF00));
    assert_eq!(d.peek(t + T_PAN_MODE), 2);
}

#[test]
fn pan_setup_mode_2_limits_and_spread() {
    let t = sfx_track(0);
    let d = setup(2, 0x08, 0x04, 0x0200, 0);
    assert_eq!(d.peek(t + T_PAN_LIMIT), 0x0C);
    assert_eq!(d.r16_for_test(t + T_PAN_SPEED), 0x0200);
    assert_eq!(d.peek(t + T_SPREAD), 4);
    let d = setup(2, 0x18, 0x04, 0x0200, 0);
    assert_eq!(d.peek(t + T_PAN_LIMIT), 0x14);
    assert_eq!(d.r16_for_test(t + T_PAN_SPEED), 0xFE00);
}

#[test]
fn pan_setup_mode_3_sweeps_to_min_of_pan_plus_spread() {
    let t = sfx_track(0);
    let d = setup(3, 0x10, 0x20, 0x1500, 0);
    assert_eq!(d.peek(t + T_PAN_LIMIT), 0x20);
    assert_eq!((d.peek(t + T_PAN), d.r16_for_test(t + T_PAN_SPEED)), (0x10, 0x1500));
    let d = setup(3, 0x10, 0x04, 0x1500, 0);
    assert_eq!(d.peek(t + T_PAN_LIMIT), 0x14);
}

#[test]
fn pan_setup_mode_4_sweeps_down_with_negated_speed() {
    let t = sfx_track(0);
    let d = setup(4, 0x10, 0x04, 0x1500, 0);
    assert_eq!(d.peek(t + T_PAN_LIMIT), 0x0C);
    assert_eq!(d.r16_for_test(t + T_PAN_SPEED), 0xEB00);
    let d = setup(4, 0x02, 0x04, 0x1500, 0);
    assert_eq!(d.peek(t + T_PAN_LIMIT), 0);
}

#[test]
fn pan_setup_modes_5_and_6_use_the_second_speed() {
    let t = sfx_track(0);
    let d = setup(5, 0x10, 0x04, 0x0156, 0x0100);
    assert_eq!(d.r16_for_test(t + T_PAN_SPEED), 0x0156);
    assert_eq!(d.r16_for_test(t + T_PAN_SPEED2), 0x0100);
    let d = setup(6, 0x10, 0x04, 0x0156, 0x0100);
    assert_eq!(d.r16_for_test(t + T_PAN_SPEED), 0xFEAA);
    assert_eq!(d.r16_for_test(t + T_PAN_SPEED2), 0xFF00);
}

#[test]
fn pan_setup_mode_7_copies_a_twin_to_the_next_voice() {
    let t = sfx_track(0);
    let next = sfx_track(1);
    // Voice 1 has a priority 2 effect: the twin replaces it (5 >= 2) with delay 4 and -v.
    let mut d = setup(7, 0x10, 0x04, 0x0380, 0);
    let twin = next;
    assert_eq!(d.peek(twin + T_PAN_MODE), 8);
    assert_eq!(d.peek(twin + T_PRIO), 5);
    assert_eq!(d.peek(twin + T_DELAY), 4);
    assert_eq!(d.peek(twin + T_PAN_LIMIT), 0x0C);
    assert_eq!(d.r16_for_test(twin + T_PAN_SPEED), 0u16.wrapping_sub(0x0380));
    assert_eq!(d.peek(t + T_DELAY), 0, "the first voice keeps its own motion");
    // A higher priority effect on voice 1 is not replaced.
    d.poke_for_test(next + T_PRIO, 6);
    d.poke_for_test(next + T_PAN_LIMIT, 0x01);
    d.pan_setup_for_test(t, RING_SLOT);
    assert_eq!(d.peek(next + T_PAN_LIMIT), 0x01);
}

#[test]
fn boot_sends_stop_all_and_enables_requests() {
    let snd = synth(&[]);
    let mut d = Driver::new(&snd, true);
    assert_eq!(d.peek(SHARED_ENABLE), 0x88);
    settle(&mut d);
    assert_eq!(d.peek(DEBUG), 0);
    assert_eq!(d.peek(STEREO), 0xFF);
    // The first DSP write of the boot is the DSP init (register 0xDE = 0).
    assert_eq!(d.dsp_writes().first().copied(), Some(crate::RegWrite { reg: 0xDE, value: 0 }));
}

/// The real sound data (`dino.zip` at run time). Every effect and music code starts the voices
/// its header lists, with the header byte as priority.
#[test]
fn rom_codes_allocate_the_voices_their_header_lists() {
    let path = rom_path();
    if !path.exists() {
        eprintln!("skipped: {} not found", path.display());
        return;
    }
    let pack = dino_extract::extract(&path).expect("dino.zip reads");
    let snd = &pack.sound;
    let mut d = Driver::new(snd, true);
    settle(&mut d);
    for code in [0x00u16, 0x01, 0x30, 0x2C, 0x65, 0xE4, 0xCA, 0xCC] {
        let cmd = snd.command(code).expect("code in table");
        let Some(_) = cmd.kind else { continue };
        // Fresh start for every code: stop everything first.
        send(&mut d, 0xFF00, 0, 0);
        send(&mut d, code, 0, 0);
        let header = cmd.header;
        let h = snd.byte(header);
        for v in 0..16u8 {
            let listed = cmd.voice_off[v as usize] != 0;
            let t = if h == 0 { d.track(v, true) } else { d.track(v, false) };
            assert_eq!(t.active, listed, "code {code:#04x} voice {v}");
            if listed && h != 0 {
                assert_eq!(t.priority, h, "code {code:#04x} voice {v}");
            }
            if listed {
                assert_eq!(t.ptr, cmd.track(v as usize).expect("listed voice has a track"), "code {code:#04x} voice {v}");
            }
        }
    }
}

/// DSP register writes of frame 0 in a MAME QSound trace (`re/out/sound/CODE.qs.trc`, made by
/// the M4-01 probe). The directory comes from `DINO_SOUND_TRACE_DIR`; without the trace the
/// test is skipped. Lines look like `f 0 r c4 v 0000`; `b` lines are bank writes and are not DSP
/// register writes of the driver.
fn mame_start_writes(code: &str) -> Option<Vec<(u8, u16)>> {
    let dir = match std::env::var("DINO_SOUND_TRACE_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) => Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../re/out/sound"),
    };
    let text = std::fs::read_to_string(dir.join(format!("{code}.qs.trc"))).ok()?;
    let mut out = Vec::new();
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() == 6 && f[0] == "f" && f[1] == "0" && f[2] == "r" && f[4] == "v" {
            let reg = u8::from_str_radix(f[3], 16).ok()?;
            let val = u16::from_str_radix(f[5], 16).ok()?;
            out.push((reg, val));
        }
    }
    Some(out)
}

/// The writes that start a coin (0x30): the voice resets and the centre pans of its two
/// voices, before the first key-on. MAME's frame 0 starts with the same eight writes.
#[test]
fn rom_coin_start_writes_match_mame_trace() {
    let path = rom_path();
    let Some(expected) = mame_start_writes("0030") else {
        eprintln!("skipped: no MAME trace of code 0x30 (set DINO_SOUND_TRACE_DIR)");
        return;
    };
    let pack = dino_extract::extract(&path).expect("dino.zip reads");
    let mut d = Driver::new(&pack.sound, true);
    settle(&mut d);
    d.take_dsp_writes();
    send(&mut d, 0x0030, 0, 0);
    let mine: Vec<(u8, u16)> = d.take_dsp_writes().iter().map(|w| (w.reg, w.value)).collect();
    assert_eq!(mine.len(), 8);
    assert_eq!(&expected[..mine.len()], &mine[..]);
}

/// The writes that start music 0x00: a voice reset and a centre pan for each music voice with
/// no effect on it, in voice order. MAME's frame 0 starts with the same writes.
#[test]
fn rom_music_start_writes_match_mame_trace() {
    let path = rom_path();
    let Some(expected) = mame_start_writes("0000") else {
        eprintln!("skipped: no MAME trace of code 0x00 (set DINO_SOUND_TRACE_DIR)");
        return;
    };
    let pack = dino_extract::extract(&path).expect("dino.zip reads");
    let mut d = Driver::new(&pack.sound, true);
    settle(&mut d);
    d.take_dsp_writes();
    send(&mut d, 0x0000, 0, 0);
    let mine: Vec<(u8, u16)> = d.take_dsp_writes().iter().map(|w| (w.reg, w.value)).collect();
    assert!(mine.len() >= 4 * 14, "music 0x00 uses 14 voices, got {} writes", mine.len());
    assert_eq!(&expected[..mine.len()], &mine[..]);
}
