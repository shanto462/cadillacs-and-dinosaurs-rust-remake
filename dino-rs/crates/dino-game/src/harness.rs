//! Headless runs for the exactness harness (`re/harness/README.md`, plan phase 0).
//!
//! `--trace FILE` runs the game without a window for `--frames N` game frames with the scripted
//! input (`--input-file` / `--input`) and writes the port trace. `--load-state FILE` starts from
//! a MAME snapshot written by `re/harness/record.lua` instead of `--stage` / `--chars`.
//!
//! Trace format: see `re/harness/trace.py` (same container as the MAME trace). Port regions:
//! `ram` / `ram_mask` (every modeled field at its original address, and which bytes those are),
//! `gfx` / `gfx_mask` (tilemaps and palettes), `obj` (the sprite list built this frame). Each
//! record also carries `{"snd": [...]}`, the sound requests of the frame.

use crate::game::Game;
use crate::game::trace::{POOLS, PORTED_CLASSES};
use crate::input::Script;
use std::fs::File;
use std::io::{BufWriter, Write};

const BLOCK: usize = 64;

/// One region of a snapshot file.
pub struct Region {
    pub name: String,
    pub base: u32,
    pub data: Vec<u8>,
}

impl Region {
    fn check(&self, base: u32) -> Result<&Vec<u8>, String> {
        if self.base == base { Ok(&self.data) } else { Err(format!("region {} at {:#x}, expected {base:#x}", self.name, self.base)) }
    }
}

/// Reads a trace or state file record by record (format: `re/harness/trace.py`). Each `next()`
/// applies the blocks of one record to the region images, so `regions` always holds the full state
/// after the record last read.
pub struct TraceReader {
    d: Vec<u8>,
    pos: usize,
    block: usize,
    pub regions: Vec<Region>,
}

impl TraceReader {
    pub fn open(path: &str) -> Result<TraceReader, String> {
        let d = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        if d.len() < 12 || &d[..8] != b"DINOTRC1" {
            return Err(format!("{path}: not a harness trace / state file"));
        }
        let n = u32::from_le_bytes([d[8], d[9], d[10], d[11]]) as usize;
        let header = std::str::from_utf8(&d[12..12 + n]).map_err(|e| format!("{path}: {e}"))?;
        let block = json_number(header, "\"block\":").unwrap_or(BLOCK as u64) as usize;
        let mut regions = Vec::new();
        let regs = header.split("\"regions\":[").nth(1).ok_or("state header has no regions")?;
        for item in regs.split('}').take_while(|s| s.contains("\"name\"")) {
            let name = item.split("\"name\":\"").nth(1).and_then(|s| s.split('"').next()).unwrap_or("").to_string();
            let base = json_number(item, "\"base\":").ok_or("region without base")? as u32;
            let size = json_number(item, "\"size\":").ok_or("region without size")? as usize;
            regions.push(Region { name, base, data: vec![0; size] });
        }
        Ok(TraceReader { pos: 12 + n, d, block, regions })
    }

    fn u32le(&self, o: usize) -> u32 {
        u32::from_le_bytes([self.d[o], self.d[o + 1], self.d[o + 2], self.d[o + 3]])
    }

    /// Applies the next record. Returns its frame number (-1 = the sync state), or `None` at the
    /// end of the file.
    pub fn next(&mut self) -> Option<i32> {
        if self.pos + 16 > self.d.len() {
            return None;
        }
        let frame = self.u32le(self.pos) as i32;
        let mut pos = self.pos + 16; // frame, flags, vblank, lag
        for r in 0..self.regions.len() {
            let count = self.u32le(pos) as usize;
            pos += 4;
            for _ in 0..count {
                let idx = self.u32le(pos) as usize;
                pos += 4;
                let b = self.block;
                self.regions[r].data[idx * b..idx * b + b].copy_from_slice(&self.d[pos..pos + b]);
                pos += b;
            }
        }
        let extra = self.u32le(pos) as usize;
        self.pos = pos + 4 + extra;
        Some(frame)
    }

    pub fn region(&self, name: &str) -> Result<&Region, String> {
        self.regions.iter().find(|r| r.name == name).ok_or(format!("trace has no {name} region"))
    }
}

fn json_number(s: &str, key: &str) -> Option<u64> {
    let rest = s.split(key).nth(1)?;
    let digits: String = rest.trim_start().chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn json_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Frame record pointers of the live objects of a snapshot (+$28, and +$30 for split bodies),
/// sorted by the kind of record they probably are, for `dino_extract::extract_with`. The
/// extractor decodes any that its registered animation tables did not reach, so unported classes
/// still show their draw state. Pointers outside the ROM (0 = none, RAM lists) are skipped.
/// `all`: scan every record of the file (step runs), not only the first one.
pub fn scan_frames(path: &str, all: bool) -> Result<dino_extract::ExtraFrames, String> {
    let mut rd = TraceReader::open(path)?;
    let mut out = dino_extract::ExtraFrames::default();
    while rd.next().is_some() {
        let ram = &rd.region("ram")?.data;
        let long = |o: usize| u32::from_be_bytes([ram[o], ram[o + 1], ram[o + 2], ram[o + 3]]);
        for (kind, (base, stride, count)) in POOLS.iter().skip(1) {
            let effects = matches!(*kind, "effect" | "special");
            for i in 0..*count {
                let o = (*base + *stride * i as u16) as usize;
                if ram[o] == 0 {
                    continue;
                }
                let flags = ram[o + 0x25];
                let rom = |p: u32| (0x1000..0x18_0000).contains(&p);
                let p = long(o + 0x28);
                if rom(p) {
                    if flags & 0x40 != 0 {
                        out.raw.push(p);
                    } else if effects {
                        out.effect.push(p);
                    } else {
                        out.sprite.push(p);
                    }
                }
                let p2 = long(o + 0x30);
                if flags & 1 != 0 && rom(p2) {
                    out.sprite.push(p2);
                }
            }
        }
        for v in [&mut out.sprite, &mut out.effect, &mut out.raw] {
            v.sort_unstable();
            v.dedup();
        }
        if !all {
            break;
        }
    }
    Ok(out)
}

/// Loads a MAME snapshot into the game. Returns the warnings of the loader.
pub fn load_state(game: &mut Game, path: &str) -> Result<Vec<String>, String> {
    let mut rd = TraceReader::open(path)?;
    rd.next();
    load_regions(game, &rd)
}

/// Loads the state a [`TraceReader`] currently holds.
fn load_regions(game: &mut Game, rd: &TraceReader) -> Result<Vec<String>, String> {
    let ram = rd.region("ram")?.check(0xFF0000)?;
    let gfx = rd.region("gfx")?.check(0x900000)?;
    game.hold_unported = true; // keep objects of unported classes (`Game::trace_hold`)
    Ok(game.trace_load(ram, gfx))
}

pub struct TraceWriter {
    out: BufWriter<File>,
    prev: Vec<Vec<u8>>,
}

const REGIONS: [(&str, u32, usize); 6] = [
    ("ram", 0xFF0000, 0x10000),
    ("ram_mask", 0xFF0000, 0x10000),
    ("gfx", 0x900000, 0x30000),
    ("gfx_mask", 0x900000, 0x30000),
    ("obj", 0, 0x800),
    ("obj_src", 0, 0x200),
];

impl TraceWriter {
    pub fn create(path: &str, game: &mut Game, meta: &str) -> Result<TraceWriter, String> {
        let file = File::create(path).map_err(|e| format!("{path}: {e}"))?;
        let mut out = BufWriter::new(file);
        let regions: Vec<String> =
            REGIONS.iter().map(|(n, b, s)| format!("{{\"name\":\"{n}\",\"base\":{b},\"size\":{s}}}")).collect();
        let pools: Vec<String> = POOLS
            .iter()
            .map(|(k, (b, s, n))| format!("{{\"kind\":\"{k}\",\"base\":{},\"stride\":{s},\"count\":{n}}}", 0xFF0000 + *b as u32))
            .collect();
        let ported: Vec<String> = PORTED_CLASSES
            .iter()
            .map(|(k, v)| format!("\"{k}\":[{}]", v.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(",")))
            .collect();
        let (kinds, globals) = game.trace_names();
        let list = |v: &[(u32, u8, &str)]| -> String {
            let items: Vec<String> = v.iter().map(|(a, s, n)| format!("[{a},{s},{}]", json_str(n))).collect();
            format!("[{}]", items.join(","))
        };
        let fields: Vec<String> = kinds.iter().map(|(k, v)| format!("\"{k}\":{}", list(v))).collect();
        let header = format!(
            "{{\"source\":\"port\",\"format\":1,\"block\":{BLOCK},\"regions\":[{}],\"pools\":[{}],\"ported\":{{{}}},\"fields\":{{{}}},\"globals\":{}{meta}}}",
            regions.join(","),
            pools.join(","),
            ported.join(","),
            fields.join(","),
            list(&globals)
        );
        out.write_all(b"DINOTRC1").map_err(|e| e.to_string())?;
        out.write_all(&(header.len() as u32).to_le_bytes()).map_err(|e| e.to_string())?;
        out.write_all(header.as_bytes()).map_err(|e| e.to_string())?;
        Ok(TraceWriter { out, prev: Vec::new() })
    }

    /// Writes the record of game frame `frame` (-1 = the state before the first frame).
    pub fn record(&mut self, game: &mut Game, frame: i32) -> Result<(), String> {
        let c = game.trace_ram();
        let (gfx, gfx_mask) = game.trace_gfx();
        let obj = game.trace_obj();
        let obj_src = game.trace_obj_src();
        let images = [c.ram, c.mask, gfx, gfx_mask, obj, obj_src];
        let mut buf = Vec::with_capacity(4096);
        buf.extend_from_slice(&frame.to_le_bytes());
        buf.extend_from_slice(&[0u8; 12]); // flags, vblank, lag
        for (k, img) in images.iter().enumerate() {
            let prev = self.prev.get(k);
            let mut blocks = Vec::new();
            for (i, b) in img.chunks(BLOCK).enumerate() {
                if prev.is_none_or(|p| p[i * BLOCK..i * BLOCK + BLOCK] != *b) {
                    blocks.push(i);
                }
            }
            buf.extend_from_slice(&(blocks.len() as u32).to_le_bytes());
            for i in blocks {
                buf.extend_from_slice(&(i as u32).to_le_bytes());
                buf.extend_from_slice(&img[i * BLOCK..i * BLOCK + BLOCK]);
            }
        }
        let snd: Vec<String> = game.w.sounds.iter().map(|s| s.to_string()).collect();
        let extra = format!("{{\"snd\":[{}]}}", snd.join(","));
        buf.extend_from_slice(&(extra.len() as u32).to_le_bytes());
        buf.extend_from_slice(extra.as_bytes());
        self.out.write_all(&buf).map_err(|e| e.to_string())?;
        self.prev = images.into();
        Ok(())
    }

    pub fn finish(mut self) -> Result<(), String> {
        self.out.flush().map_err(|e| e.to_string())
    }
}

/// Where a headless run starts when it neither loads a snapshot nor steps from a trace: from
/// power on (`boot`, the boot screens and the attract mode), or directly in a stage.
pub struct Start<'a> {
    pub boot: bool,
    pub stage: u8,
    pub chars: &'a [u8],
}

/// The headless run: load or start, then `frames` ticks with scripted input, tracing each.
///
/// `follow_rng`: a MAME trace whose random number state replaces the port's before every frame
/// (G.$14 after MAME's previous frame). Objects of unported classes call `rand()` in the original,
/// which the port cannot repeat, so a free run would drift apart at the first such call; with
/// this the drift is cut at every frame boundary and the ported code keeps being compared.
///
/// `step`: a MAME trace for a step run. Before every frame the port is loaded with MAME's state
/// after the previous frame, so a difference at frame k comes from frame k's logic alone and
/// cannot be a consequence of an earlier difference (see `re/harness/README.md`).
pub fn run(
    assets: &'static dino_assets::AssetPack,
    trace: &str,
    load: Option<&str>,
    step: Option<&str>,
    follow_rng: Option<&str>,
    start: Start,
    config: crate::game::flow::Config,
    script: Option<&Script>,
    frames: u32,
) -> Result<(), String> {
    let mut game = Game::new(assets);
    game.flow_apply_config(config);
    let mut warnings = Vec::new();
    let mut reader = match step {
        Some(p) => Some(TraceReader::open(p)?),
        None => None,
    };
    let mut rng_reader = match follow_rng {
        Some(p) => {
            let mut r = TraceReader::open(p)?;
            r.next(); // record -1: the state the run starts from
            Some(r)
        }
        None => None,
    };
    match (&mut reader, load) {
        (Some(rd), _) => {
            rd.next(); // record -1, the sync state
            warnings = load_regions(&mut game, rd)?;
        }
        (None, Some(p)) => warnings = load_state(&mut game, p)?,
        (None, None) if start.boot => game.flow_boot(),
        (None, None) => game.start_stage(start.stage, start.chars),
    }
    for w in &warnings {
        eprintln!("load-state: {w}");
    }
    let warn: Vec<String> = warnings.iter().map(|w| json_str(w)).collect();
    let meta = format!(
        ",\"kind\":\"trace\",\"mode\":\"{}\",\"follow_rng\":{},\"frames\":{frames},\"load_state\":{},\"warnings\":[{}]",
        if step.is_some() { "step" } else { "run" },
        follow_rng.is_some(),
        json_str(step.or(load).unwrap_or("")),
        warn.join(",")
    );
    // `DINO_STEP_PLAYERS`: a second reader, one record ahead, supplies the players' state after the
    // frame being run (see `Game::trace_load_players`).
    let mut ahead = match (step, std::env::var_os("DINO_STEP_PLAYERS")) {
        (Some(p), Some(_)) => {
            let mut r = TraceReader::open(p)?;
            r.next(); // record -1
            Some(r)
        }
        _ => None,
    };
    let mut tw = TraceWriter::create(trace, &mut game, &meta)?;
    tw.record(&mut game, -1)?;
    for k in 0..frames {
        if let Some(rd) = &mut reader
            && k > 0
        {
            // The state after MAME's frame k-1 is the start of frame k.
            if rd.next().is_none() {
                return Err(format!("the MAME trace has no record for frame {}", k - 1));
            }
            load_regions(&mut game, rd)?; // the warnings repeat every frame: shown once above
        }
        if let Some(rd) = &mut rng_reader {
            if k > 0 && rd.next().is_none() {
                return Err(format!("the MAME trace has no record for frame {}", k - 1));
            }
            let ram = &rd.region("ram")?.data;
            game.w.rng = u16::from_be_bytes([ram[0x8014], ram[0x8015]]); // G.$14
        }
        if let Some(a) = &mut ahead {
            if a.next().is_none() {
                return Err(format!("the MAME trace has no record for frame {k}"));
            }
            game.spawn.g.step_players = Some(a.region("ram")?.check(0xFF0000)?.to_vec());
        }
        let f = script.map(|s| s.at(k)).unwrap_or_default();
        game.w.input = f.players;
        // 0x738 quirk: bit 3 of G.$1A is always set (lsr.w shifts bit 8 of d1 = 0xFFC0, left
        // over from 0x6C0, into the start bits).
        game.w.system = f.system | 0x08;
        game.w.coins = f.coins;
        game.tick();
        tw.record(&mut game, k as i32)?;
    }
    tw.finish()?;
    eprintln!("trace: {frames} frames written to {trace}");
    Ok(())
}
