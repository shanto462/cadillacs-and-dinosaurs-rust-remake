//! Writes the sound of one or more sound codes to WAV files, for listening by hand. Nothing is
//! played on the machine.
//!
//! Usage (from `dino-rs/`):
//!   cargo run -p dino-sound --example play -- --codes 0x01,0x30 [--rom ../dino.zip]
//!        [--out ../re/out/sound] [--seconds S] [--tail S] [--d1 N] [--d2 N] [--mono]
//!
//! Without `--seconds` a sound effect is cut at its last DSP write (plus `--tail`, default 0),
//! so the length is the audible span of the code. A music code is rendered for 20 seconds: its
//! loop length is not measured by this tool (the loop is a jump, not the track end). Pass
//! `--seconds` for music. The lengths are printed so they can be checked against
//! `plan/game/design/D4e-sound-cues.md` section 0 (the table of codes) and section 1.6.
//!
//! The sample ROM and the sequences are read from the zip at run time (`dino-extract`); no
//! sound bytes are written into the source. The output of the DSP model is an approximation
//! (see `re/specs/sound/HANDOFF.md`), so the sound is for ear checks only.

use std::fs;
use std::path::{Path, PathBuf};

use dino_sound::{Driver, Mixer, RATE};

/// Sequencer ticks per second (the 250 Hz IRQ, SPEC 2).
const TICKS_PER_SEC: f64 = 250.0;
/// Video frames per second (SPEC 2); one video frame is 250 / 59.6374 ticks.
const VIDEO_HZ: f64 = 59.6374;
/// Longest render when no `--seconds` is given.
const MAX_SECONDS: f64 = 120.0;

struct Args {
    codes: Vec<u16>,
    rom: PathBuf,
    out: PathBuf,
    seconds: Option<f64>,
    tail: f64,
    /// Request words (default 0 and 0, the MAME recordings). D4e lists d1 = 0x70010 and
    /// d2 = 0x3800000 for the coin, not verified: pass them with `--d1` and `--d2`.
    d1: Option<u32>,
    d2: Option<u32>,
    mono: bool,
}

fn parse_num(s: &str) -> Result<u64, String> {
    let t = s.trim();
    let r = match t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        Some(h) => u64::from_str_radix(h, 16),
        None => t.parse::<u64>(),
    };
    r.map_err(|e| format!("bad number {t:?}: {e}"))
}

fn parse_args() -> Result<Args, String> {
    let root = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .unwrap_or_else(|_| PathBuf::from("."));
    let mut a = Args {
        codes: vec![0x01, 0x30],
        rom: root.join("dino.zip"),
        out: root.join("re/out/sound"),
        seconds: None,
        tail: 0.0,
        d1: None,
        d2: None,
        mono: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut val = || it.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--codes" => {
                a.codes = val()?.split(',').map(|c| parse_num(c).map(|n| n as u16)).collect::<Result<_, _>>()?
            }
            "--rom" => a.rom = PathBuf::from(val()?),
            "--out" => a.out = PathBuf::from(val()?),
            "--seconds" => a.seconds = Some(val()?.parse().map_err(|e| format!("--seconds: {e}"))?),
            "--tail" => a.tail = val()?.parse().map_err(|e| format!("--tail: {e}"))?,
            "--d1" => a.d1 = Some(parse_num(&val()?)? as u32),
            "--d2" => a.d2 = Some(parse_num(&val()?)? as u32),
            "--mono" => a.mono = true,
            other => return Err(format!("unknown argument {other}")),
        }
    }
    Ok(a)
}

fn wav_bytes(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut b = Vec::with_capacity(44 + data_len as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes()); // PCM
    b.extend_from_slice(&2u16.to_le_bytes()); // stereo
    b.extend_from_slice(&RATE.to_le_bytes());
    b.extend_from_slice(&(RATE * 4).to_le_bytes());
    b.extend_from_slice(&4u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for &x in samples {
        let v = (x.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        b.extend_from_slice(&v.to_le_bytes());
    }
    b
}

/// Renders one code. Without `--seconds`, a sound effect (a code that starts an SFX track) is
/// cut at its last DSP write (plus `--tail`); a music code is rendered for 20 seconds, because its
/// loop length is not measured here (SPEC 17, section 0 of D4e has the lengths). Returns the
/// stereo samples, the length in seconds and the kind of length.
fn render(pack: &dino_assets::AssetPack, a: &Args, code: u16, mono: bool) -> (Vec<f32>, f64, &'static str) {
    let mut d = Driver::new(&pack.sound, !mono);
    let mut m = Mixer::new(&pack.sound.pcm);
    m.set_mono(mono);
    let mut out = Vec::new();
    let mut rendered = 0usize;

    // The same four settle ticks as the MAME comparison in `dino-sound/src/tests.rs`.
    for _ in 0..4 {
        d.vblank();
        d.tick();
    }
    d.take_dsp_writes();
    // d1 = d2 = 0 by default: the MAME recordings of the tests are made with these requests.
    d.command(code, a.d1.unwrap_or(0), a.d2.unwrap_or(0));

    let limit = a.seconds.unwrap_or(if is_effect_code(code) { MAX_SECONDS } else { 20.0 });
    let max_ticks = (limit * TICKS_PER_SEC) as u64;
    let per_frame = TICKS_PER_SEC / VIDEO_HZ;
    let mut tick = 0u64;
    let mut last_write = 0u64;
    let mut is_sfx = false;
    let mut f = 0u64;
    'frames: loop {
        d.vblank();
        let want = ((f + 1) as f64 * per_frame).floor() as u64;
        while tick < want && tick < max_ticks {
            d.tick();
            tick += 1;
            let w = d.take_dsp_writes();
            if !w.is_empty() {
                last_write = tick;
            }
            if !is_sfx {
                is_sfx = (0..dino_sound::VOICES).any(|v| d.track(v, false).active);
            }
            m.apply(&w);
            let target = (tick as usize) * RATE as usize / TICKS_PER_SEC as usize;
            m.render(target - rendered, &mut out);
            rendered = target;
        }
        // A sound effect that has stopped writing for one second is over.
        if a.seconds.is_none() && is_sfx && tick > last_write + TICKS_PER_SEC as u64 {
            break 'frames;
        }
        if tick >= max_ticks {
            break 'frames;
        }
        f += 1;
    }
    let (frames, kind) = if a.seconds.is_some() {
        (out.len() / 2, "--seconds")
    } else if is_sfx {
        let secs = last_write as f64 / TICKS_PER_SEC + a.tail;
        ((secs * RATE as f64).round() as usize, "sound effect, last DSP write")
    } else {
        (out.len() / 2, "music, rendered 20 s, loop not measured")
    };
    let frames = frames.min(out.len() / 2);
    out.truncate(frames * 2);
    (out, frames as f64 / RATE as f64, kind)
}

/// Codes 0x00 to 0x26 are music, 0x27 and up are sound effects (D4e section 0). Only used to
/// choose the run limit; the cut uses what the run shows (`is_sfx`).
fn is_effect_code(code: u16) -> bool {
    code >= 0x27
}

fn main() {
    let a = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("play: {e}");
            std::process::exit(2);
        }
    };
    let pack = match dino_extract::extract(&a.rom) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("play: cannot read {}: {e}", a.rom.display());
            std::process::exit(1);
        }
    };
    if let Err(e) = fs::create_dir_all(&a.out) {
        eprintln!("play: cannot create {}: {e}", a.out.display());
        std::process::exit(1);
    }
    let out_dir = fs::canonicalize(&a.out).unwrap_or(a.out.clone());
    for &code in &a.codes {
        let (samples, secs, kind) = render(&pack, &a, code, a.mono);
        let name = format!("code_{code:04x}{}.wav", if a.mono { "_mono" } else { "" });
        let path = out_dir.join(name);
        if let Err(e) = fs::write(&path, wav_bytes(&samples)) {
            eprintln!("play: cannot write {}: {e}", path.display());
            std::process::exit(1);
        }
        println!("code {code:#06x}: {secs:.3} s ({kind}) -> {}", path.display());
    }
}
