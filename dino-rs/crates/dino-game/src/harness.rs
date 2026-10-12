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
use crate::game::trace::POOLS;
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

/// Reads the first record (the sync state) of a trace or state file into full region images.
pub fn read_state(path: &str) -> Result<Vec<Region>, String> {
    let d = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    if d.len() < 12 || &d[..8] != b"DINOTRC1" {
        return Err(format!("{path}: not a harness trace / state file"));
    }
    let u32le = |o: usize| u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]]);
    let n = u32le(8) as usize;
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
    let mut pos = 12 + n + 16; // frame, flags, vblank, lag
    for r in regions.iter_mut() {
        let count = u32le(pos) as usize;
        pos += 4;
        for _ in 0..count {
            let idx = u32le(pos) as usize;
            pos += 4;
            r.data[idx * block..idx * block + block].copy_from_slice(&d[pos..pos + block]);
            pos += block;
        }
    }
    Ok(regions)
}

fn json_number(s: &str, key: &str) -> Option<u64> {
    let rest = s.split(key).nth(1)?;
    let digits: String = rest.trim_start().chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn json_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Loads a MAME snapshot into the game. Returns the warnings of the loader.
pub fn load_state(game: &mut Game, path: &str) -> Result<Vec<String>, String> {
    let regions = read_state(path)?;
    let find = |name: &str| regions.iter().find(|r| r.name == name).ok_or(format!("state file has no {name} region"));
    let ram = find("ram")?.check(0xFF0000)?;
    let gfx = find("gfx")?.check(0x900000)?;
    Ok(game.trace_load(ram, gfx))
}

pub struct TraceWriter {
    out: BufWriter<File>,
    prev: Vec<Vec<u8>>,
}

const REGIONS: [(&str, u32, usize); 5] = [
    ("ram", 0xFF0000, 0x10000),
    ("ram_mask", 0xFF0000, 0x10000),
    ("gfx", 0x900000, 0x30000),
    ("gfx_mask", 0x900000, 0x30000),
    ("obj", 0, 0x800),
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
        let (kinds, globals) = game.trace_names();
        let list = |v: &[(u32, u8, &str)]| -> String {
            let items: Vec<String> = v.iter().map(|(a, s, n)| format!("[{a},{s},{}]", json_str(n))).collect();
            format!("[{}]", items.join(","))
        };
        let fields: Vec<String> = kinds.iter().map(|(k, v)| format!("\"{k}\":{}", list(v))).collect();
        let header = format!(
            "{{\"source\":\"port\",\"format\":1,\"block\":{BLOCK},\"regions\":[{}],\"pools\":[{}],\"fields\":{{{}}},\"globals\":{}{meta}}}",
            regions.join(","),
            pools.join(","),
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
        let images = [c.ram, c.mask, gfx, gfx_mask, obj];
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

/// The headless run: load or start, then `frames` ticks with scripted input, tracing each.
pub fn run(
    assets: &'static dino_assets::AssetPack,
    trace: &str,
    load: Option<&str>,
    stage: u8,
    chars: &[u8],
    script: Option<&Script>,
    frames: u32,
) -> Result<(), String> {
    let mut game = Game::new(assets);
    let mut warnings = Vec::new();
    match load {
        Some(p) => {
            warnings = load_state(&mut game, p)?;
            for w in &warnings {
                eprintln!("load-state: {w}");
            }
        }
        None => game.start_stage(stage, chars),
    }
    let warn: Vec<String> = warnings.iter().map(|w| json_str(w)).collect();
    let meta = format!(
        ",\"kind\":\"trace\",\"frames\":{frames},\"load_state\":{},\"warnings\":[{}]",
        json_str(load.unwrap_or("")),
        warn.join(",")
    );
    let mut tw = TraceWriter::create(trace, &mut game, &meta)?;
    tw.record(&mut game, -1)?;
    for k in 0..frames {
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
