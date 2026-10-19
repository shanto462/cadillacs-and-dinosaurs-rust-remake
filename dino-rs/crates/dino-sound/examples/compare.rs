//! Compare a whole sound.sh batch, including optional `at=N` requests.
//! Run from dino-rs: cargo run -p dino-sound --example compare -- REQUESTS TRACE_DIR FRAMES
//! ROM data is loaded by dino-extract. Reports and port traces belong under re/out/.
//! Register order and values are checked independently of video-frame timing: the native
//! driver has IRQ ticks, not Z80 instruction or DSP handshake timing (SPEC 2, 4).
use dino_assets::sound::SoundKind;
use dino_sound::Driver;
use std::{env, fs, path::PathBuf};

#[derive(Clone)]
struct Request {
    code: u16,
    d1: u32,
    d2: u32,
    name: String,
    extra: Option<(usize, u16, u32, u32)>,
}
fn number(s: &str) -> u32 {
    s.strip_prefix("0x").map_or_else(
        || s.parse().unwrap(),
        |s| u32::from_str_radix(s, 16).unwrap(),
    )
}

type Write = (usize, u8, u16);

/// Compare all expected writes. Only the final boundary's port tail is permitted, since
/// the recorder and the native driver sample different phases of the 250 Hz IRQ.
fn differences(expected: &[Write], actual: &[Write], frames: usize) -> (usize, Option<usize>) {
    let mut count = 0;
    let mut first = None;
    for (i, want) in expected.iter().enumerate() {
        if actual.get(i).map(|w| (w.1, w.2)) != Some((want.1, want.2)) {
            count += 1;
            first.get_or_insert(i);
        }
    }
    for (i, w) in actual.iter().enumerate().skip(expected.len()) {
        if w.0 + 1 < frames {
            count += 1;
            first.get_or_insert(i);
        }
    }
    (count, first)
}

#[test]
fn comparison_detects_values_order_missing_and_extra_writes() {
    let a = [(0, 1, 100), (1, 2, 200)];
    assert_eq!(differences(&a, &a, 10), (0, None));
    assert_eq!(differences(&a, &[(1, 1, 100), (2, 2, 200)], 10), (0, None));
    assert_eq!(differences(&a, &[(0, 1, 101), a[1]], 10), (1, Some(0)));
    assert_eq!(differences(&a, &[a[1], a[0]], 10), (2, Some(0)));
    assert_eq!(differences(&a, &a[..1], 10), (1, Some(1)));
    assert_eq!(differences(&[], &a, 10), (2, Some(0)));
    assert_eq!(differences(&a[..1], &a, 10), (1, Some(1)));
    assert_eq!(differences(&a, &[a[0], a[1], (9, 3, 300)], 10), (0, None));
}

fn main() {
    let args: Vec<_> = env::args().collect();
    assert_eq!(args.len(), 4, "REQUESTS TRACE_DIR FRAMES");
    let rom = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../dino.zip");
    let pack = dino_extract::extract(&rom).expect("read user ROM");
    // Generate requests from the runtime command table, never a copied ROM table.
    // `--batch music|effects FILE` writes an input for sound.sh.
    if args[1] == "--batch" {
        let music = match args[2].as_str() {
            "music" => true,
            "effects" => false,
            _ => panic!("music|effects"),
        };
        let mut batch = String::new();
        let commands: Vec<_> = pack
            .sound
            .commands
            .iter()
            .filter(|c| {
                matches!(
                    (music, c.kind),
                    (true, Some(SoundKind::Music)) | (false, Some(SoundKind::Effect(_)))
                )
            })
            .collect();
        assert_eq!(commands.len(), if music { 39 } else { 204 });
        for c in &commands {
            batch.push_str(&format!("sound {:#x} 0 0\n", c.code));
        }
        if !music {
            // Mode 7 exercises delayed twins and priority checks on every effect header.
            for c in &commands {
                batch.push_str(&format!("sound {:#x} 0x00072010 0x03800000\n", c.code));
            }
            // The position and sweep modes used by the game, including opposite facing.
            for mode in 1..=6 {
                batch.push_str(&format!(
                    "sound 0x66 {:#x} 0x01561080\n",
                    (mode << 16) | 0x1210
                ));
            }
            // Find five overlapping voice pairs with different priorities, from runtime
            // headers. Try both orders: a higher priority steals, a lower one is rejected.
            let mut pairs = Vec::new();
            let mut priorities = Vec::new();
            for low in &commands {
                for high in &commands {
                    let (Some(SoundKind::Effect(a)), Some(SoundKind::Effect(b))) =
                        (low.kind, high.kind)
                    else {
                        unreachable!()
                    };
                    if a < b
                        && low
                            .voice_off
                            .iter()
                            .zip(high.voice_off)
                            .any(|(&a, b)| a != 0 && b != 0)
                        && !priorities.contains(&a)
                    {
                        pairs.push((low.code, high.code));
                        priorities.push(a);
                        break;
                    }
                }
                if pairs.len() == 5 {
                    break;
                }
            }
            assert_eq!(pairs.len(), 5);
            for (a, b) in pairs {
                for (a, b) in [(a, b), (b, a)] {
                    batch.push_str(&format!(
                        "sound {a:#x} 0 0 name=priority_{a:04x}_{b:04x}\nsound {b:#x} 0 0 at=3\n"
                    ));
                }
            }
            // Toggle and player bank twins keep the first code alive on its own voice.
            for (a, b) in [
                (0x65, 0xe4),
                (0x7b, 0xfc),
                (0x38, 0xa8),
                (0x6b, 0xf1),
                (0x4b, 0xc2),
                (0xc2, 0xc3),
            ] {
                batch.push_str(&format!("sound {a:#x} 0x00032010 0x15000000 name=toggle_{a:04x}_{b:04x}\nsound {b:#x} 0x00042010 0x15000000 at=3\n"));
            }
        }
        fs::write(&args[3], batch).unwrap();
        return;
    }
    let dir = PathBuf::from(&args[2]);
    let frames: usize = args[3].parse().unwrap();
    // Diagnostic clock seed only. Defaults retain the existing tests' common phase;
    // the recorder does not currently save the Z80 IRQ phase at request injection.
    let irq_phase: u8 = env::var("DINO_SOUND_IRQ_PHASE").map_or(0, |s| s.parse().unwrap());
    let frame_phase: f64 = env::var("DINO_SOUND_FRAME_PHASE").map_or(0.0, |s| s.parse().unwrap());
    assert!(irq_phase < 4 && (0.0..1.0).contains(&frame_phase));
    let mut requests: Vec<Request> = Vec::new();
    for line in fs::read_to_string(&args[1]).unwrap().lines() {
        let words: Vec<_> = line.split('#').next().unwrap().split_whitespace().collect();
        if words.is_empty() {
            continue;
        }
        let words = if words[0] == "sound" {
            &words[1..]
        } else {
            &words[..]
        };
        let code = number(words[0]) as u16;
        let d1 = words.get(1).map_or(0, |s| number(s));
        let d2 = words.get(2).map_or(0, |s| number(s));
        if let Some(at) = words.iter().find_map(|s| s.strip_prefix("at=")) {
            let req = requests.last_mut().unwrap();
            assert!(req.extra.is_none(), "only one at=N request per trace");
            req.extra = Some((at.parse().unwrap(), code, d1, d2));
            continue;
        }
        let name = words
            .iter()
            .find_map(|s| s.strip_prefix("name="))
            .map(str::to_owned)
            .unwrap_or_else(|| {
                if d1 == 0 && d2 == 0 {
                    format!("{code:04x}")
                } else {
                    format!("{code:04x}_{d1:08x}_{d2:08x}")
                }
            });
        requests.push(Request {
            code,
            d1,
            d2,
            name,
            extra: None,
        });
    }
    let port = dir.join("port");
    fs::create_dir_all(&port).unwrap();
    let status_path = env::var_os("DINO_SOUND_STATUS")
        .map(PathBuf::from)
        .unwrap_or_else(|| dir.join("qs-status.txt"));
    let status = fs::read_to_string(status_path).expect("completed MAME batch status");
    assert!(
        !status.lines().any(|l| l.starts_with("error ")),
        "MAME batch failed"
    );
    println!("code,register_differences,first_write,mame_frame,port_frame,mame_register,port_register,mame_value,port_value,timing_differences,max_frame_delta,mame_writes,port_tail_writes,bank_writes,status,first_timing_write,first_timing_mame_frame,first_timing_port_frame");
    let mut failures = 0;
    for req in requests {
        let text = fs::read_to_string(dir.join(format!("{}.qs.trc", req.name))).unwrap();
        let window: usize = text
            .lines()
            .next()
            .unwrap()
            .split_whitespace()
            .find_map(|w| w.strip_prefix("frames="))
            .expect("trace window")
            .parse()
            .unwrap();
        assert!(
            window >= frames,
            "{}: only {window} recorded frames, need {frames}",
            req.name
        );
        assert!(
            status.lines().any(|line| {
                let w: Vec<_> = line.split_whitespace().collect();
                w.len() == 6
                    && w[0] == "code"
                    && w[1] == req.name
                    && w[5] == "1"
                    && w[3].parse::<usize>().ok()
                        == Some(text.lines().filter(|l| l.starts_with("f ")).count())
            }),
            "{}: missing, incomplete or unflushed MAME trace",
            req.name
        );
        let mut expected = Vec::new();
        let mut banks = 0;
        for line in text.lines() {
            let w: Vec<_> = line.split_whitespace().collect();
            if w.len() == 4 && w[2] == "b" {
                banks += 1;
            }
            if w.len() == 6 && w[0] == "f" && w[2] == "r" {
                let f: usize = w[1].parse().unwrap();
                if f < frames {
                    expected.push((
                        f,
                        u8::from_str_radix(w[3], 16).unwrap(),
                        u16::from_str_radix(w[5], 16).unwrap(),
                    ));
                }
            }
        }
        let mut d = Driver::new(&pack.sound, true);
        for _ in 0..4 {
            d.vblank();
            d.tick();
        }
        d.take_dsp_writes();
        // IRQ 0x0038 and main loop 0x00BE: counters F004 and F005 have the same value.
        let counter = (d.peek(0xF004) & !3) | irq_phase;
        d.poke(0xF004, counter);
        d.poke(0xF005, counter);
        d.command(req.code, req.d1, req.d2);
        let mut actual = Vec::new();
        let mut ticks = 0;
        for f in 0..frames + 2 {
            d.vblank();
            if let Some((at, code, d1, d2)) = req.extra {
                if f + 1 == at {
                    d.command(code, d1, d2);
                }
            }
            let target = (((f + 1) as f64) * 250.0 / 59.6374 + frame_phase).floor() as usize;
            while ticks < target {
                d.tick();
                ticks += 1;
            }
            actual.extend(d.take_dsp_writes().into_iter().map(|w| (f, w.reg, w.value)));
        }
        let trace: String = std::iter::once(format!(
            "# sound code={:04x} d1={:08x} d2={:08x} frames={frames}\n",
            req.code, req.d1, req.d2
        ))
        .chain(
            actual
                .iter()
                .map(|(f, r, v)| format!("f {f} r {r:02x} v {v:04x}\n")),
        )
        .collect();
        fs::write(port.join(format!("{}.qs.trc", req.name)), trace).unwrap();
        let (differences, first) = differences(&expected, &actual, frames);
        let mut timing = 0;
        let mut first_timing = None;
        let mut delta = 0;
        for (i, want) in expected.iter().enumerate() {
            let got = actual.get(i);
            if let Some(got) = got {
                if got.0 != want.0 {
                    timing += 1;
                    first_timing.get_or_insert((i + 1, want.0, got.0));
                }
                delta = delta.max(got.0.abs_diff(want.0));
            }
        }
        // The two extra frames supply the final MAME IRQ. Extra writes before the final
        // boundary frame are an error too, including a silent MAME trace versus an active port.
        let tail = &actual[expected.len().min(actual.len())..];
        let detail = first
            .map(|i| {
                let a = expected.get(i).copied().unwrap_or((usize::MAX, 0, 0));
                let b = actual.get(i).copied().unwrap_or((usize::MAX, 0, 0));
                format!(
                    "{},{},{},{:02x},{:02x},{:04x},{:04x}",
                    i + 1,
                    a.0,
                    b.0,
                    a.1,
                    b.1,
                    a.2,
                    b.2
                )
            })
            .unwrap_or_else(|| ",,,,,,".into());
        println!(
            "{},{},{},{},{},{},{},{},{},{}",
            req.name,
            differences,
            detail,
            timing,
            delta,
            expected.len(),
            tail.len(),
            banks,
            if differences == 0 {
                "registers_equal"
            } else {
                "DIFFER"
            },
            first_timing
                .map(|(i, a, b)| format!("{i},{a},{b}"))
                .unwrap_or_else(|| ",,".into())
        );
        if differences != 0 {
            failures += 1;
        }
    }
    if failures != 0 {
        std::process::exit(1);
    }
}
