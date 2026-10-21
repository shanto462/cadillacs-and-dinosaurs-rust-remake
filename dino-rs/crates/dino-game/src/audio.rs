//! Audio output (M4-12): the sound side of the game. Each video frame the game's sound requests (`Game::snd_feed_requests`)
//! go to the sound driver (`dino-sound`), the driver runs its 250 Hz ticks for the frame, the QSound model renders the
//! samples at `dino_sound::RATE`, a linear resampler turns them into the device rate, and a ring buffer hands them to
//! the cpal output stream.
//!
//! The sound side only reads the game. It never writes into the game state, so the game is the same with or without
//! sound. The device is opened only by `Audio::start`, which only the windowed run calls (`main.rs`). A missing device,
//! `--mute` or `sound = off` in `dino.cfg` means no `Audio`: the game runs silent. Headless runs, harness runs and the
//! tests never open a device; the tests drive `Feed` directly and check the samples.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dino_assets::sound::SoundData;
use dino_sound::{Driver, Mixer, RATE};

use crate::game::Game;

/// CPS1 master clock and raster dimensions (MAME `mame -listxml dino`, `re/specs/sound/SPEC.md` section 2).
/// Pixel clock: 8 MHz. Htotal: 512 dots. Vtotal: 262 lines.
pub const CPS1_PIXCLOCK: f64 = 8_000_000.0;
pub const CPS1_HTOTAL: f64 = 512.0;
pub const CPS1_VTOTAL: f64 = 262.0;
/// Video refresh rate: 8,000,000 / (512 * 262) = 59.63740458015267 Hz (~59.64 Hz).
pub const VIDEO_HZ: f64 = CPS1_PIXCLOCK / (CPS1_HTOTAL * CPS1_VTOTAL);
/// Per-frame video duration: (512 * 262) / 8,000,000 = 0.016768 s = 16.768 ms = 16,768,000 ns.
pub const FRAME_DURATION: std::time::Duration = std::time::Duration::from_nanos(16_768_000);

/// The 250 Hz sequencer tick (SPEC 2).
/// Sequencer ticks per video frame: 250.0 / VIDEO_HZ = 4.192 ticks/frame.
pub const TICKS_PER_SEC: f64 = 250.0;
/// Target audio cushion in the ring buffer (~50 ms, about 3 video frames) to prevent underruns.
const TARGET_BUFFER_SECS: f64 = 0.05;
/// How much rendered audio the ring buffer holds before the oldest samples are dropped (keeps the latency bounded).
const RING_SECONDS: f64 = 0.5;

/// The DSP side: the driver (68000 mailbox, Z80 model) and the QSound mixer. One video frame at a time: `request` for each
/// queued code, then `run_frame`, which appends the frame's samples at `RATE` (interleaved stereo, f32).
pub struct Feed<'a> {
    driver: Driver<'a>,
    mixer: Mixer<'a>,
    /// Sequencer ticks run and samples rendered so far (the sample clock is `ticks * RATE / 250`).
    ticks: u64,
    rendered: usize,
    /// Fraction of a tick carried from one video frame to the next (4.19 ticks per frame).
    tick_acc: f64,
}

impl<'a> Feed<'a> {
    /// Power on of the sound side. `qsound` is the cabinet setting (`stereo` in `dino.cfg`, `G.$777A` bit 7): off is mono.
    pub fn new(snd: &'a SoundData, qsound: bool) -> Self {
        let mut driver = Driver::new(snd, qsound);
        let mut mixer = Mixer::new(&snd.pcm);
        mixer.set_mono(!qsound);
        // The four settle ticks of the MAME comparison (the `play` example): their writes are dropped.
        for _ in 0..4 {
            driver.vblank();
            driver.tick();
        }
        driver.take_dsp_writes();
        Feed { driver, mixer, ticks: 0, rendered: 0, tick_acc: 0.0 }
    }

    /// One request of the 68000 side (code, d1, d2), queued in the driver's mailbox. The driver sends one per frame.
    pub fn request(&mut self, code: u16, d1: u32, d2: u32) {
        self.driver.command(code, d1, d2);
    }

    /// One video frame: the vblank flush (one queued request reaches the Z80), then the ticks of this frame. The writes
    /// of each tick reach the mixer before the samples of that tick are rendered. Appends the samples to `out`.
    pub fn run_frame(&mut self, out: &mut Vec<f32>) {
        self.driver.vblank();
        self.tick_acc += TICKS_PER_SEC / VIDEO_HZ;
        let n = self.tick_acc.floor();
        self.tick_acc -= n;
        for _ in 0..n as u32 {
            self.driver.tick();
            self.mixer.apply(&self.driver.take_dsp_writes());
            self.ticks += 1;
            let target = self.ticks as usize * RATE as usize / TICKS_PER_SEC as usize;
            self.mixer.render(target - self.rendered, out);
            self.rendered = target;
        }
    }
}

/// Linear sample rate conversion from `RATE` (stereo, interleaved) to the device rate. The state carries from one call
/// to the next, so the conversion does not click at the frame edges.
pub struct Resampler {
    /// Input frames per output frame.
    step: f64,
    /// Position of the next output sample, in input frames, relative to the start of the next call's buffer.
    pos: f64,
    /// The last input frame of the previous call (position 0 of the next buffer).
    prev: [f32; 2],
}

impl Resampler {
    pub fn new(device_rate: u32) -> Self {
        Resampler { step: RATE as f64 / device_rate as f64, pos: 0.0, prev: [0.0; 2] }
    }

    /// `input` is stereo at `RATE`; the converted stereo samples are appended to `out`.
    pub fn process(&mut self, input: &[f32], out: &mut Vec<f32>) {
        let n = input.len() / 2;
        if n == 0 {
            return;
        }
        // Buffer position 0 is the previous call's last frame, position k is input frame k - 1.
        let prev = self.prev;
        let at = |i: usize| -> [f32; 2] {
            if i == 0 { prev } else { [input[2 * i - 2], input[2 * i - 1]] }
        };
        let len = n + 1;
        while self.pos + 1.0 < len as f64 {
            let idx = self.pos.floor() as usize;
            let f = (self.pos - idx as f64) as f32;
            let (a, b) = (at(idx), at(idx + 1));
            out.push(a[0] + (b[0] - a[0]) * f);
            out.push(a[1] + (b[1] - a[1]) * f);
            self.pos += self.step;
        }
        self.prev = at(n);
        self.pos -= n as f64;
    }
}

type Ring = Arc<Mutex<VecDeque<f32>>>;

/// The output stream. The callback pops interleaved samples (device channel count) from the ring buffer and plays
/// silence when the ring is empty. Dropping it stops the sound.
pub struct Output {
    _stream: cpal::Stream,
    ring: Ring,
    channels: usize,
    capacity: usize,
    target_len: usize,
    device_rate: u32,
    resampler: Resampler,
    scratch: Vec<f32>,
    underruns: Arc<AtomicUsize>,
}

impl Output {
    /// Opens the default output device. An error names the reason (no device, unsupported format, ...).
    pub fn open() -> Result<Output, String> {
        let device = cpal::default_host().default_output_device().ok_or("no audio output device")?;
        let supported = device.default_output_config().map_err(|e| format!("output config: {e}"))?;
        let format = supported.sample_format();
        let config: cpal::StreamConfig = supported.config();
        let ring: Ring = Arc::new(Mutex::new(VecDeque::new()));
        let underruns = Arc::new(AtomicUsize::new(0));
        let stream = match format {
            cpal::SampleFormat::F32 => build::<f32>(&device, &config, ring.clone(), underruns.clone())?,
            cpal::SampleFormat::I16 => build::<i16>(&device, &config, ring.clone(), underruns.clone())?,
            cpal::SampleFormat::U16 => build::<u16>(&device, &config, ring.clone(), underruns.clone())?,
            other => return Err(format!("unsupported sample format {other:?}")),
        };
        let channels = config.channels as usize;
        let rate = config.sample_rate.0;
        let target_len = (rate as f64 * channels as f64 * TARGET_BUFFER_SECS) as usize;
        // Pre-fill cushion with silence so the callback never starves on startup
        if let Ok(mut q) = ring.lock() {
            q.extend(std::iter::repeat_n(0.0f32, target_len));
        }
        stream.play().map_err(|e| format!("start stream: {e}"))?;
        Ok(Output {
            _stream: stream,
            ring,
            channels,
            capacity: (rate as f64 * channels as f64 * RING_SECONDS) as usize,
            target_len,
            device_rate: rate,
            resampler: Resampler::new(rate),
            scratch: Vec::new(),
            underruns,
        })
    }

    /// Queues stereo samples at `RATE`: they are resampled, spread over the device channels (mono devices get the mean
    /// of left and right, extra channels get silence) and appended to the ring.
    pub fn push(&mut self, samples: &[f32]) {
        let q_len = match self.ring.lock() {
            Ok(q) => q.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        };
        // Dynamic Rate Control: subtle resampler step adjustment (±0.5% max) to maintain
        // the target ~50 ms cushion without drift or clicks over long play sessions.
        let base_step = RATE as f64 / self.device_rate as f64;
        let diff = (q_len as f64 - self.target_len as f64) / self.target_len as f64;
        let adjust = (diff * 0.05).clamp(-0.005, 0.005);
        self.resampler.step = base_step * (1.0 + adjust);

        self.scratch.clear();
        self.resampler.process(samples, &mut self.scratch);
        let mut q = match self.ring.lock() {
            Ok(q) => q,
            Err(poisoned) => poisoned.into_inner(),
        };
        for pair in self.scratch.as_chunks::<2>().0 {
            let (l, r) = (pair[0], pair[1]);
            if self.channels == 1 {
                q.push_back((l + r) * 0.5);
            } else {
                q.push_back(l);
                q.push_back(r);
                for _ in 2..self.channels {
                    q.push_back(0.0);
                }
            }
        }
        while q.len() > self.capacity {
            q.pop_front();
        }
    }

    /// Number of sample underruns experienced by the audio callback since startup.
    pub fn underruns(&self) -> usize {
        self.underruns.load(Ordering::Relaxed)
    }
}

/// The stream for one sample format: the callback copies samples from the ring.
fn build<T>(device: &cpal::Device, config: &cpal::StreamConfig, ring: Ring, underruns: Arc<AtomicUsize>) -> Result<cpal::Stream, String>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                let mut q = match ring.lock() {
                    Ok(q) => q,
                    Err(poisoned) => poisoned.into_inner(),
                };
                for s in data.iter_mut() {
                    match q.pop_front() {
                        Some(sample) => *s = T::from_sample(sample),
                        None => {
                            *s = T::from_sample(0.0);
                            underruns.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            },
            |err| eprintln!("audio: {err}"),
            None,
        )
        .map_err(|e| format!("build stream: {e}"))
}

/// One video frame of the sound side: the game's requests into the feed, the samples to the output.
fn feed_frame(feed: &mut Feed, game: &mut Game, samples: &mut Vec<f32>) {
    for r in game.snd_feed_requests() {
        feed.request(r.code, r.d1, r.d2);
    }
    samples.clear();
    feed.run_frame(samples);
}

/// The game's sound: the feed and the open output. Only the windowed run has one.
pub struct Audio<'a> {
    feed: Feed<'a>,
    out: Output,
    samples: Vec<f32>,
}

impl<'a> Audio<'a> {
    /// `None` when muted (`--mute` or `sound = off`) or when no output opens; the reason goes to stderr and the game runs
    /// silent. The device is opened before the feed is built, so a muted or device-less run never runs the driver.
    pub fn start(snd: &'a SoundData, qsound: bool, muted: bool) -> Option<Self> {
        if muted {
            return None;
        }
        match Output::open() {
            Ok(out) => Some(Audio { feed: Feed::new(snd, qsound), out, samples: Vec::new() }),
            Err(e) => {
                eprintln!("audio: {e}: the game runs without sound");
                None
            }
        }
    }

    /// Call once per video frame, after `game.tick()`. Reads the game's requests and never changes the game.
    pub fn frame(&mut self, game: &mut Game) {
        feed_frame(&mut self.feed, game, &mut self.samples);
        self.out.push(&self.samples);
    }

    /// Number of sample underruns recorded by the output stream.
    pub fn underruns(&self) -> usize {
        self.out.underruns()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::flow::Config;
    use dino_assets::AssetPack;
    use std::path::Path;
    use std::sync::OnceLock;

    /// The asset pack of the user's ROM, `None` when it is not there (the tests that need it skip).
    fn rom() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    #[test]
    fn resampler_keeps_level_and_length() {
        let mut r = Resampler::new(48_000);
        let input = vec![0.5f32; 2 * RATE as usize];
        let mut out = Vec::new();
        r.process(&input, &mut out);
        // one second in gives one second out (up to the one frame held back for the next call)
        assert!((out.len() / 2) as i64 - 48_000 <= 1 && (out.len() / 2) >= 47_999, "frames {}", out.len() / 2);
        // the first output frame is the zero the resampler starts from; the rest is the constant input
        assert!(out[4..].iter().all(|&x| (x - 0.5).abs() < 1e-4), "a constant input stays constant");
    }

    #[test]
    fn resampler_split_calls_match_one_call() {
        let input: Vec<f32> = (0..2000).map(|i| ((i as f32) * 0.01).sin()).collect();
        let mut whole = Vec::new();
        Resampler::new(44_100).process(&input, &mut whole);
        let mut parts = Vec::new();
        let mut r = Resampler::new(44_100);
        for chunk in input.chunks(2 * 137) {
            r.process(chunk, &mut parts);
        }
        assert_eq!(whole.len(), parts.len());
        assert!(whole.iter().zip(&parts).all(|(a, b)| (a - b).abs() < 1e-6));
    }

    /// Runs `frames` game frames of the power-on boot with the given config and returns the peak of each frame's samples
    /// (the requests of each frame are rendered in the frame they are queued, as the windowed run does).
    fn peaks_of_boot(assets: &'static AssetPack, config: Config, frames: u32) -> Vec<f32> {
        let mut game = crate::game::Game::new(assets);
        game.flow_apply_config(config);
        game.flow_boot();
        let mut feed = Feed::new(&assets.sound, true);
        let mut samples = Vec::new();
        let mut peaks = Vec::new();
        for _ in 0..frames {
            game.tick();
            feed_frame(&mut feed, &mut game, &mut samples);
            peaks.push(samples.iter().fold(0.0f32, |p, &x| p.max(x.abs())));
        }
        peaks
    }

    /// The title tune plays only with the demo sound setting (the attract filter of 0x610, `snd.rs`). With the factory
    /// setting the title is silent; with `demo_sound = on` it is heard from the first frame of its request.
    #[test]
    fn title_is_silent_at_factory_and_heard_with_demo_sound() {
        let Some(assets) = rom() else {
            eprintln!("skipped: no dino.zip");
            return;
        };
        let silent = peaks_of_boot(assets, Config::default(), 400);
        let loud = peaks_of_boot(assets, Config { demo_sound: true, ..Config::default() }, 400);
        let first = loud.iter().position(|&p| p > 1e-4);
        eprintln!("demo sound on: first audible frame {first:?}; peaks at the first 5 audible frames {:?}",
            first.map(|f| &loud[f..(f + 5).min(loud.len())]));
        // the request log: MAME and the port both queue the title tune (code 0x88) at frame 329 (attract.mame.trc)
        assert_eq!(first, Some(329), "the title tune starts in the frame of its request");
        assert!(silent.iter().all(|&p| p < 1e-6), "the factory setting is silent in the title");
    }

    /// The samples of `frames` game frames of a scripted run, rendered the way the windowed run feeds the driver
    /// (`feed_frame`: the requests that pass the attract filter of 0x610, one video frame of ticks), and the peak of every frame.
    fn render_run(assets: &'static AssetPack, boot: bool, script: &str, frames: u32) -> (Vec<f32>, Vec<f32>) {
        let script = crate::input::Script::load(script).expect("the input script loads");
        let mut game = crate::game::Game::new(assets);
        if boot {
            game.flow_apply_config(Config::default());
            game.flow_boot();
        } else {
            game.start_stage(0, &[0]);
        }
        let mut feed = Feed::new(&assets.sound, true);
        let (mut all, mut samples, mut peaks) = (Vec::new(), Vec::new(), Vec::new());
        for k in 0..frames {
            let f = script.at(k);
            game.w.input = f.players;
            game.w.system = f.system | 0x08;
            game.w.coins = f.coins;
            game.tick();
            feed_frame(&mut feed, &mut game, &mut samples);
            peaks.push(samples.iter().fold(0.0f32, |p, &x| p.max(x.abs())));
            all.extend_from_slice(&samples);
        }
        (all, peaks)
    }

    /// 16 bit stereo WAV of the mixer output (`RATE`), for the ear check: `DINO_WAV_OUT=dir cargo test ... listening`.
    fn write_wav(path: &Path, samples: &[f32]) {
        let data: Vec<u8> = samples.iter().flat_map(|&x| ((x.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes()).collect();
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes()); // PCM
        out.extend_from_slice(&2u16.to_le_bytes()); // stereo
        out.extend_from_slice(&RATE.to_le_bytes());
        out.extend_from_slice(&(RATE * 4).to_le_bytes());
        out.extend_from_slice(&4u16.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&data);
        std::fs::write(path, out).expect("the WAV is written");
    }

    /// M4-19 listening check that does not need ears: the real mix of a game run is not silent where the game makes sounds, is silent
    /// in the attract filter (title, factory setting), has no NaN and never reaches full scale (no clipping). Two runs: from power on
    /// (coin, start, hero select, the Episode 1 opening: `coinstart.txt`) and a fight in stage 1 (`stage1_fight.txt`, hits, voices,
    /// containers, many sounds at once). With `DINO_WAV_OUT=dir` the two mixes are also written as WAV files for the ear check.
    #[test]
    fn listening_check_game_mix_is_not_silent_and_does_not_clip() {
        let Some(assets) = rom() else {
            eprintln!("skipped: no dino.zip");
            return;
        };
        let scripts = ["../../../re/harness/scripts/coinstart.txt", "../../../re/harness/scripts/stage1_fight.txt"];
        if !scripts.iter().all(|s| Path::new(s).exists()) {
            eprintln!("skipped: no input scripts");
            return;
        }
        for (name, boot, script, frames) in [("power_on_to_episode1", true, scripts[0], 2300u32), ("stage1_fight", false, scripts[1], 1400u32)] {
            let (all, peaks) = render_run(assets, boot, script, frames);
            assert!(all.iter().all(|x| x.is_finite()), "{name}: NaN or infinity in the mix");
            let peak = all.iter().fold(0.0f32, |p, &x| p.max(x.abs()));
            let loud = peaks.iter().filter(|&&p| p > 1e-3).count();
            let over = peaks.iter().filter(|&&p| p > 0.9).count();
            let first = peaks.iter().position(|&p| p > 1e-3);
            eprintln!("{name}: {frames} frames, peak {peak:.3}, {loud} audible frames, {over} frames above 0.9, first audible frame {first:?}");
            assert!(peak < 1.0, "{name}: the mix reaches full scale ({peak}): clipping");
            assert!(loud > 20, "{name}: the mix is silent (only {loud} audible frames)");
            if boot {
                // the attract filter: nothing is heard before the coin (frame ~600); the coin sound is the first sound
                assert!(first.is_some_and(|f| (590..640).contains(&f)), "{name}: the first sound is the coin, got {first:?}");
            }
            if let Some(dir) = std::env::var_os("DINO_WAV_OUT") {
                std::fs::create_dir_all(&dir).expect("the output directory exists");
                write_wav(&Path::new(&dir).join(format!("{name}.wav")), &all);
            }
        }
    }

    /// M10-23, the whole game once through the sound side (the ear check without ears): the flow bot (`--autoplay`, god mode and the
    /// clock top up, as `the_bot_plays_from_power_on_through_all_eight_episodes_to_the_ending`) plays from power on through the eight
    /// episodes and the ending back to the attract loop, every frame's requests go to the driver and the mixer as in the windowed run.
    /// Per part of the game (the attract loop before the coin, each episode, the ending): no NaN, no clipping (the peak stays below full
    /// scale), sound in most of the part. With `DINO_WAV_OUT=dir` the first 60 seconds of each part are written as WAV files for a person
    /// to listen to. Slow (the mixer renders about 50 minutes of audio): `cargo test --release -p dino-game whole_game_mix -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn whole_game_mix_is_finite_audible_and_does_not_clip() {
        let Some(assets) = rom() else {
            eprintln!("skipped: no dino.zip");
            return;
        };
        let mut g = crate::game::Game::new(assets);
        g.flow_apply_config(Config::default());
        g.flow_boot();
        let (mut debug, mut bot) = (crate::debug::Debug::new(false), crate::autoplay::Bot::default());
        let mut feed = Feed::new(&assets.sound, true);
        let mut samples = Vec::new();
        // (name, frames, audible frames, frames above 0.9, peak, the first 60 seconds of samples)
        let mut parts: Vec<(String, u32, u32, u32, f32, Vec<f32>)> = Vec::new();
        let keep = 60 * RATE as usize * 2;
        for tick in 0..600_000u32 {
            let f = bot.frame(&mut g, tick, &mut debug);
            g.w.input = f.players;
            g.w.system = f.system | 0x08;
            g.w.coins = f.coins;
            debug.before_tick(&mut g);
            g.tick();
            debug.after_tick(&mut g);
            bot.report(&g, tick);
            feed_frame(&mut feed, &mut g, &mut samples);
            let name = if g.flow.mode == 0x1C {
                "ending".to_string()
            } else if g.flow.in_game != 0 {
                format!("episode{}", g.w.stage + 1)
            } else if parts.is_empty() {
                "attract".to_string()
            } else {
                parts.last().unwrap().0.clone()
            };
            if parts.last().is_none_or(|p| p.0 != name) {
                parts.push((name, 0, 0, 0, 0.0, Vec::new()));
            }
            let p = parts.last_mut().unwrap();
            let peak = samples.iter().fold(0.0f32, |m, &x| m.max(x.abs()));
            assert!(samples.iter().all(|x| x.is_finite()), "{}: NaN or infinity in the mix at tick {tick}", p.0);
            p.1 += 1;
            p.2 += (peak > 1e-3) as u32;
            p.3 += (peak > 0.9) as u32;
            p.4 = p.4.max(peak);
            if p.5.len() < keep {
                p.5.extend_from_slice(&samples);
            }
            if bot.done() {
                break;
            }
        }
        assert!(bot.done(), "the bot did not reach the ending and the attract loop");
        let mut written = std::collections::HashSet::new();
        for (name, frames, loud, over, peak, first) in &parts {
            eprintln!("{name}: {frames} frames, {loud} audible ({:.0} %), {over} above 0.9, peak {peak:.3}", 100.0 * *loud as f64 / *frames as f64);
            // A name can come back (after the ending the game is in Episode 8 for a few frames): the first, full part keeps its file.
            if !written.insert(name.clone()) {
                continue;
            }
            if let Some(dir) = std::env::var_os("DINO_WAV_OUT") {
                std::fs::create_dir_all(&dir).expect("the output directory exists");
                write_wav(&Path::new(&dir).join(format!("{name}.wav")), first);
            }
        }
        for (name, frames, loud, _, peak, _) in &parts {
            assert!(*peak < 1.0, "{name}: the mix reaches full scale ({peak}): clipping");
            if name.starts_with("episode") || name == "ending" {
                assert!(*loud * 2 > *frames, "{name}: sound in only {loud} of {frames} frames");
            }
        }
    }
}
