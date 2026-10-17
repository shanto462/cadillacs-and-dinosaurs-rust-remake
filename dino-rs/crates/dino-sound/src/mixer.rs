//! QSound DSP model: the register file, sample playback of 16 voices at 24038 Hz, the pan law,
//! the plain echo, and mixing to interleaved f32 stereo (M4-07 and M4-08).
//!
//! The DSP internal ROM (pan gains, FIR taps) is not in `dino.zip`, so this is a model built from
//! the register meaning in `re/specs/sound/SPEC.md` section 8, not a copy of any emulator:
//!
//! - Voice `v` (0..16) has eight registers at `v*8`: +1 start address, +2 pitch (4.12, 0x1000 =
//!   one PCM byte per output frame), +3 key on (bit 15 set = on, cleared = off), +4 loop length
//!   (`end - loop start`), +5 end address, +6 volume (0x4000 = unity), +7 unused.
//! - The bank of voice `v` is register `((v - 1) & 15) * 8` (voice 0 uses 0x78), value `0x8000 | bank`.
//! - Pan of voice `v` is register `0x80 + v`, value `0x110 + p`, p = 0..0x20 (0x10 = centre).
//! - A PCM byte is `pcm[bank * 0x10000 + address]`, signed 8 bit.
//!
//! Key on restarts the voice at its start address. When the address reaches the end, the loop
//! length is subtracted once (SPEC 8). A voice with no loop length stops at its end.
//! Echo (M4-08, assumptions marked in `re/specs/sound/HANDOFF.md`): the send of voice `v` is
//! register `0xBA+v` (unity at 0x4000), the feedback is register 0x93 (unity at 0x8000), the delay
//! in samples is register 0xD9 (low 11 bits). The echo return goes to both channels equally.
//! ADPCM is not used by this ROM and is not modelled.

use crate::driver::RegWrite;
use crate::map::VOICES;

/// Output rate of the QSound DSP in Hz (60 MHz / 2 / 1248, SPEC 8).
pub const RATE: u32 = 24038;

/// Pan register base (`0x80 + v`) and the centre value of the pan index.
const PAN_BASE: usize = 0x80;
const PAN_OFFSET: u16 = 0x110;
const PAN_MAX: u16 = 0x20;

/// Output level applied to the sum of the voices. M4-12: raised from 0.25 so the loudest music code
/// peaks near 0.5 (codes 0x00 to 0x26, 20 s renders: the peak was 0.109 at 0.25, so 0.25 * 0.5 / 0.109
/// = 1.15). At this level the loudest sound effect peaks near 0.57, and a sum above 1.0 is clipped by
/// the clamp. Tuned by the peaks of the renders, not by ear (the main session listens).
pub const OUTPUT_GAIN: f32 = 1.15;

/// Echo registers (SPEC 8): feedback, delay length, and the send base (`0xBA + v`).
const REG_ECHO_FB: usize = 0x93;
const REG_ECHO_LEN: usize = 0xD9;
const REG_ECHO_SEND: usize = 0xBA;
/// Unity values of the echo feedback and the echo send [A: assumed, not checked on hardware].
const ECHO_FB_FULL: f32 = 0x8000 as f32;
const ECHO_SEND_FULL: f32 = 0x4000 as f32;
/// Size of the echo delay line in frames. The delay is at most 0x7FF frames (about 85 ms).
const ECHO_BUF: usize = 0x800;

/// Pan register value of the centre (0x110 + 0x10). Mono mode forces every voice here.
const MONO_PAN_REG: u16 = PAN_OFFSET + 0x10;

/// Playback state of one voice that the register file does not hold: the position in 4.12
/// fixed point (address in the high part) and whether the voice is keyed on.
#[derive(Clone, Copy, Debug, Default)]
struct Playback {
    on: bool,
    pos: u32,
}

/// The QSound DSP model. Borrows the sample ROM (`SoundData::pcm`), so it never copies it.
pub struct Mixer<'a> {
    pcm: &'a [u8],
    regs: [u16; 256],
    voices: [Playback; VOICES as usize],
    /// Mono output: every voice is panned to the centre (the cabinet setting, SPEC 8).
    mono: bool,
    /// Echo delay line (mono) and its write index.
    echo: Vec<f32>,
    echo_pos: usize,
}

impl<'a> Mixer<'a> {
    /// A silent DSP. `pcm` is the sample ROM, banks of 0x10000 bytes in order.
    pub fn new(pcm: &'a [u8]) -> Self {
        Self {
            pcm,
            regs: [0; 256],
            voices: [Playback::default(); VOICES as usize],
            mono: false,
            echo: vec![0.0; ECHO_BUF],
            echo_pos: 0,
        }
    }

    /// Mono mode (cabinet setting, SPEC 8): the pan of every voice is forced to the centre
    /// (register value 0x110 + 0x10), so the left and right output are equal.
    pub fn set_mono(&mut self, mono: bool) {
        self.mono = mono;
    }

    /// Applies the DSP writes of the driver (`Driver::take_dsp_writes`), in order.
    pub fn apply(&mut self, writes: &[RegWrite]) {
        for w in writes {
            self.write(w.reg, w.value);
        }
    }

    /// One register write. A key-on (bit 15 of register `v*8+3`) restarts voice `v` at the start
    /// address held in `v*8+1` at that moment; a key-off (bit 15 clear) stops it.
    pub fn write(&mut self, reg: u8, value: u16) {
        self.regs[reg as usize] = value;
        let r = reg as usize;
        if r < 0x80 && r & 7 == 3 {
            let v = r >> 3;
            if value & 0x8000 != 0 {
                self.voices[v] = Playback {
                    on: true,
                    pos: (self.regs[v * 8 + 1] as u32) << 12,
                };
            } else {
                self.voices[v].on = false;
            }
        }
    }

    /// The register value last written (for tests and tools).
    pub fn reg(&self, reg: u8) -> u16 {
        self.regs[reg as usize]
    }

    /// Whether voice `v` is keyed on and has not reached a stop.
    pub fn voice_on(&self, v: usize) -> bool {
        self.voices[v].on
    }

    /// Appends `frames` stereo frames (L, R interleaved, f32 in -1..1) to `out`.
    pub fn render(&mut self, frames: usize, out: &mut Vec<f32>) {
        out.reserve(frames * 2);
        for _ in 0..frames {
            let [l, r] = self.next_frame();
            out.push(l);
            out.push(r);
        }
    }

    /// One output frame at [`RATE`]: all 16 voices summed, panned, sent to the echo, and scaled by
    /// [`OUTPUT_GAIN`]. The echo return is added to both channels.
    pub fn next_frame(&mut self) -> [f32; 2] {
        let (mut l, mut r) = (0.0f32, 0.0f32);
        let mut send = 0.0f32;
        for v in 0..VOICES as usize {
            let s = self.voice_sample(v);
            if s == 0.0 {
                continue;
            }
            let pan = if self.mono { MONO_PAN_REG } else { self.regs[PAN_BASE + v] };
            let [gl, gr] = pan_gains(pan);
            l += s * gl;
            r += s * gr;
            send += s * self.regs[REG_ECHO_SEND + v] as f32 / ECHO_SEND_FULL;
        }
        let e = self.echo_step(send);
        [
            ((l + e) * OUTPUT_GAIN).clamp(-1.0, 1.0),
            ((r + e) * OUTPUT_GAIN).clamp(-1.0, 1.0),
        ]
    }

    /// One frame of the echo: returns the delayed signal and stores `send + feedback * delayed`.
    /// The delay is register 0xD9 (low 11 bits, at least one frame). Runs every frame, so the
    /// tail rings on after the voices stop.
    fn echo_step(&mut self, send: f32) -> f32 {
        let len = ((self.regs[REG_ECHO_LEN] & 0x7FF) as usize).max(1);
        let rd = (self.echo_pos + ECHO_BUF - len) % ECHO_BUF;
        let delayed = self.echo[rd];
        let fb = (self.regs[REG_ECHO_FB] as f32 / ECHO_FB_FULL).min(0.999);
        self.echo[self.echo_pos] = send + fb * delayed;
        self.echo_pos = (self.echo_pos + 1) % ECHO_BUF;
        delayed
    }

    /// Advances voice `v` by one output frame and returns its level before pan, in units of
    /// full scale at unity volume (`0x4000`). Silent when the voice is off.
    fn voice_sample(&mut self, v: usize) -> f32 {
        if !self.voices[v].on {
            return 0.0;
        }
        let b = v * 8;
        let end = self.regs[b + 5] as u32;
        let loop_len = self.regs[b + 4] as u32;
        if (self.voices[v].pos >> 12) >= end {
            if loop_len == 0 {
                self.voices[v].on = false;
                return 0.0;
            }
            self.voices[v].pos = self.voices[v].pos.wrapping_sub(loop_len << 12);
            if (self.voices[v].pos >> 12) >= end {
                self.voices[v].on = false;
                return 0.0;
            }
        }
        let addr = self.voices[v].pos >> 12;
        let bank = (self.regs[((v + 15) & 15) * 8] & 0x7F) as usize;
        let idx = (bank << 16) | addr as usize;
        let s = self.pcm.get(idx).map_or(0, |&x| x as i8 as i32);
        let vol = self.regs[b + 6] as i32;
        self.voices[v].pos = self.voices[v].pos.wrapping_add(self.regs[b + 2] as u32);
        (s * vol) as f32 / (1 << 14) as f32 / 128.0
    }
}

/// Equal power pan over the 33 pan steps of register `0x110 + p` (approximation, SPEC 8 [I]).
/// Gain left = sqrt((32 - p) / 32), gain right = sqrt(p / 32): the power of each voice is
/// `gl^2 + gr^2 = 1` at every step. p = 0 is full left, p = 0x20 full right, p = 0x10 both at
/// 0.707. Values outside the range are clamped (a value below 0x110 counts as p = 0).
/// The true gains live in the DSP internal ROM (not in `dino.zip`), so this law is not checked.
pub fn pan_gains(reg: u16) -> [f32; 2] {
    let p = reg.saturating_sub(PAN_OFFSET).min(PAN_MAX) as f32;
    let max = PAN_MAX as f32;
    [((max - p) / max).sqrt(), (p / max).sqrt()]
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-6;

    /// A voice on at `start` with `pitch`, `end`, `loop_len`, volume `vol`, voice `v`.
    fn key_on(m: &mut Mixer, v: u8, start: u16, pitch: u16, end: u16, loop_len: u16, vol: u16) {
        let b = v * 8;
        m.write(b + 2, 0);
        m.write(b + 1, start);
        m.write(b + 5, end);
        m.write(b + 4, loop_len);
        m.write(b + 6, vol);
        m.write(b + 2, pitch);
        m.write(b + 3, 0x8000);
    }

    fn ramp(n: usize) -> Vec<u8> {
        (0..n).map(|i| i as u8).collect()
    }

    fn expect(byte: u8, vol: f32) -> f32 {
        byte as i8 as f32 * vol / 128.0 * OUTPUT_GAIN
    }

    #[test]
    fn pitch_0x1000_steps_one_byte_per_frame() {
        let pcm = ramp(0x100);
        let mut m = Mixer::new(&pcm);
        key_on(&mut m, 0, 0, 0x1000, 100, 0, 0x4000);
        for i in 0..100 {
            let [l, r] = m.next_frame();
            assert!((l - expect(pcm[i], 1.0)).abs() < EPS, "frame {i}");
            assert_eq!(r, 0.0, "pan 0 is full left");
        }
        assert!(m.voice_on(0), "a voice without loop stops at its end, not before");
    }

    #[test]
    fn half_pitch_reads_each_byte_twice() {
        let pcm = ramp(0x100);
        let mut m = Mixer::new(&pcm);
        key_on(&mut m, 0, 4, 0x0800, 20, 0, 0x4000);
        let got: Vec<f32> = (0..6).map(|_| m.next_frame()[0]).collect();
        let want: Vec<f32> = [4, 4, 5, 5, 6, 6].iter().map(|&i| expect(pcm[i], 1.0)).collect();
        for (g, w) in got.iter().zip(&want) {
            assert!((g - w).abs() < EPS);
        }
    }

    #[test]
    fn loop_subtracts_the_loop_length_once_at_the_end() {
        // end 10, loop length 4: the loop start is 6, so after address 9 the voice reads 6, 7, ...
        let pcm = ramp(0x100);
        let mut m = Mixer::new(&pcm);
        key_on(&mut m, 0, 0, 0x1000, 10, 4, 0x4000);
        let got: Vec<f32> = (0..14).map(|_| m.next_frame()[0]).collect();
        let want_addr = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 6, 7, 8, 9];
        for (i, a) in want_addr.iter().enumerate() {
            assert!((got[i] - expect(pcm[*a], 1.0)).abs() < EPS, "frame {i} reads {a}");
        }
    }

    #[test]
    fn voice_without_loop_stops_at_its_end() {
        let pcm = ramp(0x100);
        let mut m = Mixer::new(&pcm);
        key_on(&mut m, 0, 0, 0x1000, 5, 0, 0x4000);
        for _ in 0..5 {
            m.next_frame();
        }
        assert_eq!(m.next_frame(), [0.0, 0.0]);
        assert!(!m.voice_on(0));
    }

    #[test]
    fn volume_scales_linearly_with_unity_at_0x4000() {
        let pcm = vec![100u8; 16];
        let mut full = Mixer::new(&pcm);
        key_on(&mut full, 0, 0, 0x1000, 8, 0, 0x4000);
        let mut half = Mixer::new(&pcm);
        key_on(&mut half, 0, 0, 0x1000, 8, 0, 0x2000);
        let a = full.next_frame()[0];
        let b = half.next_frame()[0];
        assert!((a - 2.0 * b).abs() < EPS);
        assert!((a - expect(100, 1.0)).abs() < EPS);
    }

    #[test]
    fn bank_register_of_voice_v_is_block_v_minus_one() {
        // Voice 1 uses bank 1: the bank register is block 0, field 0 (SPEC 8, sample_load).
        let mut pcm = vec![0u8; 0x2_0000];
        pcm[0x1_0000] = 50;
        let mut m = Mixer::new(&pcm);
        m.write(0x00, 0x8001);
        key_on(&mut m, 1, 0, 0x1000, 4, 0, 0x4000);
        assert!((m.next_frame()[0] - expect(50, 1.0)).abs() < EPS);
    }

    #[test]
    fn pan_law_is_equal_power_with_full_left_at_0_and_full_right_at_0x20() {
        assert_eq!(pan_gains(0x110), [1.0, 0.0]);
        let [l, r] = pan_gains(0x130);
        assert!(l.abs() < EPS && (r - 1.0).abs() < EPS);
        let [l, r] = pan_gains(0x120);
        assert!((l - r).abs() < EPS && (l - std::f32::consts::FRAC_1_SQRT_2).abs() < EPS);
        for p in 0..=0x20u16 {
            let [l, r] = pan_gains(0x110 + p);
            assert!((l * l + r * r - 1.0).abs() < 1e-5, "power at p={p}");
        }
    }

    #[test]
    fn sixteen_voices_sum_to_one_voice_at_the_same_total_level() {
        let pcm: Vec<u8> = (0..0x40).map(|i| (i * 3) as u8).collect();
        let mut one = Mixer::new(&pcm);
        key_on(&mut one, 0, 0, 0x1000, 0x30, 0, 0x4000);
        let mut many = Mixer::new(&pcm);
        for v in 0..16u8 {
            key_on(&mut many, v, 0, 0x1000, 0x30, 0, 0x0400);
        }
        for _ in 0..0x30 {
            let a = one.next_frame();
            let b = many.next_frame();
            assert!((a[0] - b[0]).abs() < 1e-5 && (a[1] - b[1]).abs() < 1e-5);
        }
    }

    #[test]
    fn key_off_silences_a_looping_voice() {
        let pcm = ramp(0x100);
        let mut m = Mixer::new(&pcm);
        key_on(&mut m, 2, 0, 0x1000, 0x40, 0x10, 0x4000);
        m.next_frame();
        assert!(m.voice_on(2));
        m.write(2 * 8 + 3, 0);
        assert_eq!(m.next_frame(), [0.0, 0.0]);
    }

    #[test]
    fn output_is_clamped_to_full_scale() {
        let pcm = vec![127u8; 64];
        let mut m = Mixer::new(&pcm);
        for v in 0..16u8 {
            key_on(&mut m, v, 0, 0x1000, 60, 0, 0x7FFF);
        }
        let [l, r] = m.next_frame();
        assert!(l <= 1.0 && r <= 1.0 && l >= -1.0 && r >= -1.0);
    }

    #[test]
    fn render_appends_interleaved_stereo() {
        let pcm = ramp(0x100);
        let mut m = Mixer::new(&pcm);
        let mut out = Vec::new();
        m.render(10, &mut out);
        assert_eq!(out.len(), 20);
    }

    #[test]
    fn echo_repeats_the_send_after_the_delay() {
        // Send 1.0 on voice 0, feedback 0, delay 3 frames: the first sample comes back at frame 3.
        let pcm = vec![100u8, 0, 0, 0, 0, 0, 0, 0];
        let mut m = Mixer::new(&pcm);
        m.write(0xBA, 0x4000);
        m.write(0x93, 0);
        m.write(0xD9, 3);
        key_on(&mut m, 0, 0, 0x1000, 8, 0, 0x4000);
        let f0 = m.next_frame();
        assert!((f0[0] - expect(100, 1.0)).abs() < EPS, "dry sample at frame 0");
        assert_eq!(f0[1], 0.0, "pan 0 is full left, the echo is not there yet");
        for _ in 0..2 {
            m.next_frame();
        }
        let f3 = m.next_frame();
        assert!((f3[0] - expect(100, 1.0)).abs() < EPS, "echo on the left at frame 3");
        assert!((f3[1] - expect(100, 1.0)).abs() < EPS, "echo return is on both channels");
    }

    #[test]
    fn echo_feedback_makes_a_second_repeat() {
        let pcm = vec![100u8, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut m = Mixer::new(&pcm);
        m.write(0xBA, 0x4000);
        m.write(0x93, 0x4000); // feedback 0.5
        m.write(0xD9, 2);
        key_on(&mut m, 0, 0, 0x1000, 10, 0, 0x4000);
        let out: Vec<f32> = (0..7).map(|_| m.next_frame()[0]).collect();
        assert!((out[2] - expect(100, 1.0)).abs() < EPS, "first repeat at frame 2");
        assert!((out[4] - 0.5 * expect(100, 1.0)).abs() < EPS, "second repeat at frame 4, half level");
    }

    #[test]
    fn mono_mode_centres_every_voice() {
        let pcm = vec![100u8; 8];
        let mut m = Mixer::new(&pcm);
        m.set_mono(true);
        key_on(&mut m, 0, 0, 0x1000, 4, 0, 0x4000); // pan register 0x110 would be full left
        let [l, r] = m.next_frame();
        assert_eq!(l, r, "mono output has equal channels");
        assert!((l - expect(100, 1.0) * std::f32::consts::FRAC_1_SQRT_2).abs() < EPS);
    }

    /// Renders the real code through the driver, the way the game runs it: one 250 Hz tick at a
    /// time, the DSP writes applied before the audio of that tick. Needs `dino.zip`; skipped
    /// when it is not there.
    fn render_code(snd: &dino_assets::sound::SoundData, pcm: &[u8], code: u16, ticks: usize) -> Vec<f32> {
        use crate::driver::Driver;
        let mut d = Driver::new(snd, true);
        let mut m = Mixer::new(pcm);
        let mut out = Vec::new();
        let mut rendered = 0usize;
        d.command(code, 0, 0);
        let per_frame = 250.0 / 59.6374;
        let mut tick = 0usize;
        for f in 0.. {
            d.vblank();
            let want = ((f + 1) as f64 * per_frame).floor() as usize;
            while tick < want && tick < ticks {
                d.tick();
                tick += 1;
                m.apply(&d.take_dsp_writes());
                let target = tick * RATE as usize / 250;
                m.render(target - rendered, &mut out);
                rendered = target;
            }
            if tick >= ticks {
                break;
            }
        }
        out
    }

    #[test]
    fn real_codes_render_to_the_right_length_and_are_not_silent() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dino.zip");
        if !path.exists() {
            eprintln!("skipped: {} not found", path.display());
            return;
        }
        let pack = dino_extract::extract(&path).expect("dino.zip reads");
        let ticks = 250; // one second of game sound
        for code in [0x30u16, 0x01] {
            let a = render_code(&pack.sound, &pack.sound.pcm, code, ticks);
            assert_eq!(a.len(), 2 * RATE as usize, "code {code:#04x}: one second of stereo");
            let peak = a.iter().fold(0.0f32, |p, &x| p.max(x.abs()));
            assert!(peak > 0.01, "code {code:#04x} is silent (peak {peak})");
            assert!(a.iter().all(|x| x.is_finite() && x.abs() <= 1.0));
            let b = render_code(&pack.sound, &pack.sound.pcm, code, ticks);
            assert_eq!(a, b, "code {code:#04x} renders the same twice");
            eprintln!("code {code:#04x}: peak {peak:.3}");
        }
    }
}
