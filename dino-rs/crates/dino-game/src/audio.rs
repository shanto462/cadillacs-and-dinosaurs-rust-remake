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
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dino_assets::sound::SoundData;
use dino_sound::{Driver, Mixer, RATE};

use crate::game::Game;

/// The 250 Hz sequencer tick (SPEC 2) and the video frame rate of the cabinet (SPEC 2).
const TICKS_PER_SEC: f64 = 250.0;
const VIDEO_HZ: f64 = 59.6374;
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
    resampler: Resampler,
    scratch: Vec<f32>,
}

impl Output {
    /// Opens the default output device. An error names the reason (no device, unsupported format, ...).
    pub fn open() -> Result<Output, String> {
        let device = cpal::default_host().default_output_device().ok_or("no audio output device")?;
        let supported = device.default_output_config().map_err(|e| format!("output config: {e}"))?;
        let format = supported.sample_format();
        let config: cpal::StreamConfig = supported.config();
        let ring: Ring = Arc::new(Mutex::new(VecDeque::new()));
        let stream = match format {
            cpal::SampleFormat::F32 => build::<f32>(&device, &config, ring.clone())?,
            cpal::SampleFormat::I16 => build::<i16>(&device, &config, ring.clone())?,
            cpal::SampleFormat::U16 => build::<u16>(&device, &config, ring.clone())?,
            other => return Err(format!("unsupported sample format {other:?}")),
        };
        stream.play().map_err(|e| format!("start stream: {e}"))?;
        let channels = config.channels as usize;
        let rate = config.sample_rate.0;
        Ok(Output {
            _stream: stream,
            ring,
            channels,
            capacity: (rate as f64 * channels as f64 * RING_SECONDS) as usize,
            resampler: Resampler::new(rate),
            scratch: Vec::new(),
        })
    }

    /// Queues stereo samples at `RATE`: they are resampled, spread over the device channels (mono devices get the mean
    /// of left and right, extra channels get silence) and appended to the ring.
    pub fn push(&mut self, samples: &[f32]) {
        self.scratch.clear();
        self.resampler.process(samples, &mut self.scratch);
        let mut q = match self.ring.lock() {
            Ok(q) => q,
            Err(poisoned) => poisoned.into_inner(),
        };
        for pair in self.scratch.chunks_exact(2) {
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
}

/// The stream for one sample format: the callback copies samples from the ring.
fn build<T>(device: &cpal::Device, config: &cpal::StreamConfig, ring: Ring) -> Result<cpal::Stream, String>
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
                    *s = T::from_sample(q.pop_front().unwrap_or(0.0));
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
}
