//! Headless runs for the exactness harness (`re/harness/README.md`, plan phase 0).
//!
//! `--trace FILE` runs the game without a window for `--frames N` game frames with the scripted
//! input (`--input-file` / `--input`) and writes the port trace. `--load-state FILE` starts from
//! a MAME snapshot written by `re/harness/record.lua` instead of `--stage` / `--chars`.
//!
//! `--poke F:ADDR:SIZE:VAL` and `--poke-file FILE` write RAM values into the running game at a given
//! frame (`PokeSet`, same lines as MAME's `DINO_POKES`); `port.sh` passes the sidecar file
//! `NAME.pokes.txt` that `probe_record.sh` writes (the holds and timed writes a probe scenario keeps
//! after its snapshot), so the port repeats what the probe did to MAME.
//!
//! Trace format: see `re/harness/trace.py` (same container as the MAME trace). Port regions:
//! `ram` / `ram_mask` (every modeled field at its original address, and which bytes those are),
//! `gfx` / `gfx_mask` (tilemaps and palettes), `obj` (the sprite list built this frame). Each
//! record also carries `{"snd": [...], "snd_d": [[d1, d2], ...]}`, the sound requests of the frame (codes, and d1 and d2 of each).

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
    /// The flags word of the record `next()` applied (bit 0: a lag record, MAME's game frame overran a vblank).
    pub flags: u32,
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
        Ok(TraceReader { pos: 12 + n, d, block, regions, flags: 0 })
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
        self.flags = self.u32le(self.pos + 4);
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

// ------------------------------------------------------------------------------------------
// Pokes: timed writes into the running game
// ------------------------------------------------------------------------------------------

/// What the frame numbers of a poke line count.
/// * `Power`: video frames from power on, as MAME's `DINO_POKES` (`re/harness/record.lua`): record -1
///   is power frame 1 and record k is power frame k + 2. MAME writes at the end of the video frame,
///   before the trace sample, so the port writes after the tick of record k and before that record
///   is taken: the record shows the poked value, on both sides.
/// * `Game`: game frames of the trace, the clock of the key lines of an input script. A write at
///   frame k lands before the tick of record k, after record k - 1 was taken. This is where the
///   probe applies its holds (`probe.lua`, `apply_holds`), so the sidecar `NAME.pokes.txt` of
///   `probe_record.sh` uses it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PokeClock {
    Power,
    Game,
}

#[derive(Clone, Debug)]
struct PokeLine {
    /// The frame as written, for messages.
    frame: u32,
    addr: u32,
    size: u8,
    value: u32,
    /// A write the port cannot take is reported and skipped (generated sidecars), not an error.
    lenient: bool,
    /// Where the line came from (`--poke` or `FILE:LINE`), for messages.
    src: String,
}

/// The pokes of a run: lines `frame hexaddr size value` (the format of MAME's `DINO_POKES`), from
/// `--poke F:ADDR:SIZE:VAL` and `--poke-file FILE`.
///
/// The address is the full RAM address in hex (`FF84D0`, also with `0x`), or `G.$4D0` (`G.$` =
/// 0xFF8000, `G.-$7098` below it). The size is 1, 2 or 4 (a word or long at an even address); the
/// value is decimal or `0x` hex. A file may carry the directives `#!clock game|power` (the clock of
/// the lines after it; the default is the clock of `--poke-clock`) and `#!unknown warn|error`
/// (what to do with a write the port cannot take; the default is error). Other `#` text is a
/// comment.
///
/// A poke goes through the port's trace field list (`Game::trace_poke`): every field the trace
/// models can be poked by its original address, plus G.$46B (free play) and G.$757A (demo
/// counter). An address that no modeled field covers is an error, never a silent skip; the whole
/// set is checked before the run starts (`check`).
#[derive(Default)]
pub struct PokeSet {
    lines: Vec<PokeLine>,
    /// Line indexes by (record number, after the tick): `false` = before the tick of that record.
    slots: std::collections::BTreeMap<(i32, bool), Vec<usize>>,
    applied: usize,
    skipped: usize,
}

fn poke_addr(s: &str) -> Result<u32, String> {
    let t = s.trim();
    if let Some(r) = t.strip_prefix("G.").or_else(|| t.strip_prefix("g.")) {
        let (neg, r) = match r.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, r),
        };
        let off = u32::from_str_radix(r.strip_prefix('$').unwrap_or(r), 16).map_err(|_| format!("bad address `{s}`"))?;
        return Ok(if neg { 0xFF8000u32.wrapping_sub(off) } else { 0xFF8000 + off });
    }
    let h = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")).unwrap_or(t);
    u32::from_str_radix(h, 16).map_err(|_| format!("bad address `{s}`"))
}

fn poke_number(s: &str) -> Result<u32, String> {
    let t = s.trim();
    match t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        Some(h) => u32::from_str_radix(h, 16),
        None => t.parse(),
    }
    .map_err(|_| format!("bad number `{s}`"))
}

impl PokeSet {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    fn push(&mut self, f: [&str; 4], clock: PokeClock, lenient: bool, src: String) -> Result<(), String> {
        let frame = f[0].trim().parse::<u32>().map_err(|_| format!("{src}: bad frame `{}`", f[0]))?;
        let addr = poke_addr(f[1]).map_err(|e| format!("{src}: {e}"))?;
        let size = poke_number(f[2]).map_err(|e| format!("{src}: {e}"))?;
        let value = poke_number(f[3]).map_err(|e| format!("{src}: {e}"))?;
        if !matches!(size, 1 | 2 | 4) {
            return Err(format!("{src}: size must be 1, 2 or 4, not {size}"));
        }
        let size = size as u8;
        if addr < 0xFF0000 || addr + size as u32 > 0x100_0000 {
            return Err(format!("{src}: address {addr:06X} is not in work RAM (FF0000..FFFFFF)"));
        }
        if size > 1 && addr % 2 != 0 {
            return Err(format!("{src}: a word or long needs an even address, not {addr:06X}"));
        }
        if size < 4 && value >= 1 << (8 * size as u32) {
            return Err(format!("{src}: value {value:#X} does not fit in {size} byte(s)"));
        }
        // The record the write belongs to: power frame F is record F - 2 (frames 0 and 1 are
        // "before record -1"); a game frame is its own record number.
        let key = match clock {
            PokeClock::Power => ((frame as i32 - 2).max(-1), true),
            PokeClock::Game => (frame as i32, false),
        };
        self.slots.entry(key).or_default().push(self.lines.len());
        self.lines.push(PokeLine { frame, addr, size, value, lenient, src });
        Ok(())
    }

    /// `--poke F:ADDR:SIZE:VAL`.
    pub fn add_spec(&mut self, spec: &str, clock: PokeClock) -> Result<(), String> {
        let parts: Vec<&str> = spec.split(':').collect();
        let src = format!("--poke {spec}");
        match parts[..] {
            [f, a, s, v] => self.push([f, a, s, v], clock, false, src),
            _ => Err(format!("{src}: expected FRAME:ADDR:SIZE:VALUE")),
        }
    }

    /// `--poke-file FILE` (see the type's doc for the lines and the directives).
    pub fn add_file(&mut self, path: &str, clock: PokeClock) -> Result<(), String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        self.add_text(&text, path, clock)
    }

    fn add_text(&mut self, text: &str, path: &str, clock: PokeClock) -> Result<(), String> {
        let (mut clock, mut lenient) = (clock, false);
        for (n, raw) in text.lines().enumerate() {
            let src = format!("{path}:{}", n + 1);
            let line = raw.trim();
            if let Some(d) = line.strip_prefix("#!") {
                match d.split_whitespace().collect::<Vec<_>>()[..] {
                    ["clock", "game"] => clock = PokeClock::Game,
                    ["clock", "power"] => clock = PokeClock::Power,
                    ["unknown", "warn"] => lenient = true,
                    ["unknown", "error"] => lenient = false,
                    _ => return Err(format!("{src}: unknown directive `{line}`")),
                }
                continue;
            }
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            match line.split_whitespace().collect::<Vec<_>>()[..] {
                [f, a, s, v] => self.push([f, a, s, v], clock, lenient, src)?,
                _ => return Err(format!("{src}: expected `frame hexaddr size value`, got `{line}`")),
            }
        }
        Ok(())
    }

    /// Checks every line against the fields of the port before a run starts: an address that no
    /// modeled field covers (or that only a derived field covers) is an error, for a line that
    /// is not lenient. Lenient lines are counted and the first few are listed on stderr.
    pub fn check(&self, game: &mut Game) -> Result<(), String> {
        if self.lines.is_empty() {
            return Ok(());
        }
        let map = game.trace_poke_map();
        let mut unknown = 0;
        for l in &self.lines {
            let o = (l.addr - 0xFF0000) as usize;
            if let Some(k) = (0..l.size as usize).find(|k| map[o + k] != 1) {
                let why = if map[o + k] == 2 {
                    "a field the port keeps in another form (a pointer or list), a poke cannot set it"
                } else {
                    "not in the port's field list (a poke needs a field the trace models)"
                };
                let msg = format!("{} (frame {}): {:06X} size {}: byte {:06X} is {why}", l.src, l.frame, l.addr, l.size, l.addr + k as u32);
                if !l.lenient {
                    return Err(msg);
                }
                unknown += 1;
                if unknown <= 5 {
                    eprintln!("poke: {msg}: will be skipped");
                }
            }
        }
        if unknown > 5 {
            eprintln!("poke: {unknown} writes in all are not in the port's field list and will be skipped");
        }
        Ok(())
    }

    /// Applies the pokes of record `k`: `after` false = before the tick of record `k` (game clock),
    /// true = after the tick, before the record is taken (power clock).
    pub fn apply(&mut self, game: &mut Game, k: i32, after: bool) -> Result<(), String> {
        let Some(ids) = self.slots.get(&(k, after)).cloned() else { return Ok(()) };
        for i in ids {
            let l = self.lines[i].clone();
            match game.trace_poke(l.addr, l.size, l.value) {
                Ok(()) => self.applied += 1,
                Err(e) if l.lenient => {
                    self.skipped += 1;
                    if self.skipped <= 5 {
                        eprintln!("poke: {} (frame {}): not applied: {e}", l.src, l.frame);
                    }
                }
                Err(e) => return Err(format!("{} (frame {}): {e}", l.src, l.frame)),
            }
        }
        Ok(())
    }

    /// One line for stderr at the end of a run.
    pub fn summary(&self) -> String {
        format!("pokes: {} lines, {} writes applied, {} not applied", self.lines.len(), self.applied, self.skipped)
    }
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

/// The number of resyncs of a lag-sync run over the records 0 .. frames - 1 of a MAME trace: the lag records, and the
/// frames k with (k + 1) % every == 0 when `every` > 0 (the diagnostic period).
fn count_lag_records(path: &str, frames: u32, every: u32, in_range: impl Fn(u32) -> bool) -> Result<u32, String> {
    let mut r = TraceReader::open(path)?;
    r.next(); // record -1
    let mut prev_g0 = record_g0(&r)?;
    let mut n = 0;
    for k in 0..frames {
        if r.next().is_none() {
            break;
        }
        let g0 = record_g0(&r)?;
        if r.flags & 1 != 0 || catch_up_record(prev_g0, g0) || (every > 0 && (k + 1) % every == 0 && in_range(k)) {
            n += 1;
        }
        prev_g0 = g0;
    }
    Ok(n)
}

/// G.$0 (the game frame counter) of the record last read.
fn record_g0(r: &TraceReader) -> Result<u16, String> {
    let ram = &r.region("ram")?.data;
    Ok(u16::from_be_bytes([ram[0x8000], ram[0x8001]]))
}

/// A catch-up record: not flagged as a lag record, but it holds 2 to 0x100 game frames (G.$0 jumps by more than 1: the record after an
/// area start, where MAME runs the walk in inside one record). One port tick cannot replay it, so a lagsync run resyncs after it too
/// (`diff.py` leaves it out in lagsync mode).
fn catch_up_record(prev_g0: u16, g0: u16) -> bool {
    (2..=0x100).contains(&g0.wrapping_sub(prev_g0))
}

/// Loads the state a [`TraceReader`] currently holds.
fn load_regions(game: &mut Game, rd: &TraceReader) -> Result<Vec<String>, String> {
    let ram = rd.region("ram")?.check(0xFF0000)?;
    let gfx = rd.region("gfx")?.check(0x900000)?;
    game.hold_unported = true; // keep objects of unported classes (`Game::trace_hold`)
    let warn = game.trace_load(ram, gfx);
    // The shown sprite buffer of the snapshot is all zero (the clear 0xAE0, no build since): the list the port built before the load is gone too, as after
    // its own clear (`SpriteList::wipe`). Without this a load inside a clear that the port had not reached itself (the continue strip's game over clear in
    // the lag record m523_ep1_2p 30612) kept the old list on the screen until the port's own clear four records later.
    if game.sprites.blank {
        game.sprites.entries.clear();
    }
    // A lag record holds the whole clear 0xAE0 of the pass that overran: the part the port still had to do in the next frames
    // (`flow_attract_clear_step`, begun by its own tick before the load) is in the snapshot already. Without this the story panel's
    // exit (m800_bta9_ty_full_c lag record 8348: the exit clear, the area setup and its scroll2, a treasure put down by the area
    // script) cleared the loaded scroll2 once more in the next tick, and the treasure's first terrain test (0x3140C) found no wall.
    if rd.flags & 1 != 0 {
        game.flow.front.clear_stage = 0;
        game.flow.front.clear3_cut = 0;
    }
    Ok(warn)
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
        let snd_d: Vec<String> = game.w.snd_reqs.iter().map(|r| format!("[{},{}]", r.d1, r.d2)).collect();
        let extra = format!("{{\"snd\":[{}],\"snd_d\":[{}]}}", snd.join(","), snd_d.join(","));
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
    /// Area of the start (`--area`, or the point of `--jump`); later areas are walked to before the trace starts (`goto_area`).
    pub area: u8,
    /// Camera X of the start (`--camx`, or the point of `--jump`); 0 = the area start.
    pub camx: u16,
    pub chars: &'a [u8],
}

/// `--area` above 0 (or a jump point in a later area): the same walk as `tick_game` in `main.rs`, with no input, before the trace
/// starts. The game's own area change (the debug key F6 logic, `debug::next_area`) once the hero has control, then the camera X.
fn goto_area(game: &mut crate::game::Game, area: u8, camx: u16) -> Result<(), String> {
    for _ in 0..40_000 {
        if game.cam.s2.area >= area {
            if camx != 0 {
                game.flow_jump_camera(camx);
            }
            return Ok(());
        }
        if game.flow.mode == 0xC && game.w.scripted == 0 && game.cam.s2.mode == 0 {
            crate::debug::next_area(game);
        }
        game.tick();
    }
    Err(format!("the game did not reach area {area} (the walk to it did not end)"))
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
///
/// `lagsync`: a MAME trace for a lag-sync run: a free run like `run`, but after a frame whose MAME record is
/// a lag record (flags & 1: the original's game frame overran a vblank, one port tick cannot replay it) the
/// port is loaded with MAME's state of that record, which is the state the next frame starts from in a step
/// run. Every other frame runs free, so the drift is cut only at the lag records. The port trace has one
/// record per MAME frame; the lag records themselves are left out of diff.py. The header counts the resyncs. A catch-up record (not
/// flagged, but G.$0 jumps by 2 to 0x100: the record after an area start) is treated like a lag record (`catch_up_record`).
pub fn run(
    assets: &'static dino_assets::AssetPack,
    trace: &str,
    load: Option<&str>,
    step: Option<&str>,
    follow_rng: Option<&str>,
    lagsync: Option<&str>,
    start: Start,
    config: crate::game::flow::Config,
    script: Option<&Script>,
    mut pokes: PokeSet,
    frames: u32,
) -> Result<(), String> {
    let mut game = Game::new(assets);
    game.flow_apply_config(config);
    game.flow.game_clock = std::env::var_os("DINO_GAME_CLOCK").is_some();
    let mut warnings = Vec::new();
    if lagsync.is_some() && (step.is_some() || follow_rng.is_some()) {
        return Err("--lagsync is a mode of its own (not with --step-from or --follow-rng)".into());
    }
    // `DINO_LAGSYNC_EVERY=N` (diagnostic): resync every N frames too (N = 1 is a step run). Found by the lag-sync study, not a mode of its own.
    let every: u32 = std::env::var("DINO_LAGSYNC_EVERY").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    // `DINO_LAGSYNC_RANGE=A-B` (diagnostic, with DINO_LAGSYNC_EVERY): the periodic resyncs only after the records A ..= B (a power-on
    // run that resyncs inside the demo loop of the attract mode only, where the loader keeps the front end tasks of the port).
    let range: Option<(u32, u32)> = std::env::var("DINO_LAGSYNC_RANGE").ok().and_then(|v| {
        let (a, b) = v.split_once('-')?;
        Some((a.parse().ok()?, b.parse().ok()?))
    });
    let in_range = |k: u32| range.is_none_or(|(a, b)| (a..=b).contains(&k));
    let resyncs = match lagsync {
        Some(p) => count_lag_records(p, frames, every, in_range)?,
        None => 0,
    };
    let mut lag_reader = match lagsync {
        Some(p) => {
            let mut r = TraceReader::open(p)?;
            r.next(); // record -1: the state the run starts from (the --load-state file)
            Some(r)
        }
        None => None,
    };
    let mut prev_g0 = match &lag_reader {
        Some(r) => record_g0(r)?,
        None => 0,
    };
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
        (None, None) => {
            game.start_stage_at(start.stage, 0, if start.area > 0 { 0 } else { start.camx }, start.chars);
            if start.area > 0 {
                goto_area(&mut game, start.area, start.camx)?;
            }
        }
    }
    if load.is_some() || step.is_some() {
        game.flow_title_runs_late();
    }
    for w in &warnings {
        eprintln!("load-state: {w}");
    }
    pokes.check(&mut game)?;
    let warn: Vec<String> = warnings.iter().map(|w| json_str(w)).collect();
    let meta = format!(
        ",\"kind\":\"trace\",\"mode\":\"{}\",\"follow_rng\":{},\"frames\":{frames},\"load_state\":{},\"pokes\":{},\"warnings\":[{}]{}",
        if step.is_some() {
            "step"
        } else if lagsync.is_some() {
            "lagsync"
        } else {
            "run"
        },
        follow_rng.is_some(),
        json_str(step.or(load).unwrap_or("")),
        pokes.len(),
        warn.join(","),
        if lagsync.is_some() { format!(",\"lagsync_resyncs\":{resyncs},\"lagsync_trace\":{}", json_str(lagsync.unwrap_or(""))) } else { String::new() }
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
    let ticks_resync = std::env::var("DINO_LAGSYNC_TICKS").map_or(true, |v| v != "0");
    let mut mame_clock_before = match &mut lag_reader {
        Some(r) => {
            let ram = &r.region("ram")?.data;
            u16::from_be_bytes([ram[0x8000], ram[0x8001]])
        }
        None => 0,
    };
    let mut tw = TraceWriter::create(trace, &mut game, &meta)?;
    pokes.apply(&mut game, -1, true)?; // power frame 1 (and 0): after the load, before record -1
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
        let port_clock = game.w.frame; // G.$0 before the tick (the lag-sync run compares its step with MAME's)
        pokes.apply(&mut game, k as i32, false)?; // game clock: after record k - 1, before the tick
        let f = script.map(|s| s.at(k)).unwrap_or_default();
        game.w.input = f.players;
        // 0x738 quirk: bit 3 of G.$1A is always set (lsr.w shifts bit 8 of d1 = 0xFFC0, left
        // over from 0x6C0, into the start bits).
        game.w.system = f.system | 0x08;
        game.w.coins = f.coins;
        game.tick();
        // a game clock recording leaves out the video frames a game frame overran: one record spans the pass that overran, the frames of
        // the lag and the pass after them
        // A skip (hold attack or jump in a scripted scene, `flow/skip.rs`) keeps the CPU busy: its area change is one record (at most 64 ticks).
        let mut joined = 0;
        // `tick` clears the sound requests of the tick before: a joined record keeps the requests of all its ticks (the original queues
        // them in the one game frame that overran, `0xA63C`; M4-16: the fade and the area music of an area change are in the joined ticks)
        let (mut joined_sounds, mut joined_reqs) = (Vec::new(), Vec::new());
        while game.flow.game_clock
            && (game.flow.lag > 0 || game.flow.lag_tick || game.flow.join_next || game.flow.ending.cont.is_some() || (game.skip_join_more() && joined < 64))
        {
            joined_sounds.append(&mut game.w.sounds);
            joined_reqs.append(&mut game.w.snd_reqs);
            game.tick();
            joined += 1;
        }
        if joined > 0 {
            joined_sounds.append(&mut game.w.sounds);
            joined_reqs.append(&mut game.w.snd_reqs);
            game.w.sounds = joined_sounds;
            game.w.snd_reqs = joined_reqs;
        }
        pokes.apply(&mut game, k as i32, true)?; // power clock: after the tick, before the record
        tw.record(&mut game, k as i32)?;
        if let Some(rd) = &mut lag_reader {
            // MAME's record k: a lag record means the next frame starts from its state (as a step run loads it).
            if rd.next() != Some(k as i32) {
                return Err(format!("the MAME trace has no record for frame {k}"));
            }
            let g0 = record_g0(rd)?;
            let catch_up = rd.flags & 1 == 0 && catch_up_record(prev_g0, g0);
            prev_g0 = g0;
            // A record in which MAME ran another number of game frames than the port (G.$0 counts them) is a resync point too: the original runs as many game
            // frames in a video frame as the CPU time allows (the skip wait of `flow/skip.rs`, a story panel that starts inside a lag record), the port a fixed number.
            // `DINO_LAGSYNC_TICKS=0` leaves these out.
            let mame_clock = {
                let ram = &rd.region("ram")?.data;
                u16::from_be_bytes([ram[0x8000], ram[0x8001]])
            };
            let clocks_differ = ticks_resync && mame_clock.wrapping_sub(mame_clock_before) != game.w.frame.wrapping_sub(port_clock);
            mame_clock_before = mame_clock;
            if rd.flags & 1 != 0 || catch_up || clocks_differ || (every > 0 && (k + 1) % every == 0 && in_range(k)) {
                load_regions(&mut game, rd)?; // the warnings repeat at every resync: not shown
            }
        }
    }
    tw.finish()?;
    if !pokes.is_empty() {
        eprintln!("{}", pokes.summary());
    }
    eprintln!("trace: {frames} frames written to {trace}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> Game {
        let mut a = dino_assets::AssetPack::default();
        a.enemy.stages = (0..8).map(|_| Default::default()).collect(); // the field list visit reads the wave tables
        Game::new(Box::leak(Box::new(a)))
    }

    /// The frame counter G.$0 steps by 1 in a normal record; a jump of 2 to 0x100 is the record after an area start (a catch-up record). A 0 step
    /// (the tally stops the counter), a wrap and a jump past 0x100 are not.
    #[test]
    fn a_catch_up_record_is_a_small_unflagged_jump_of_the_frame_counter() {
        assert!(!catch_up_record(0x100, 0x101));
        assert!(!catch_up_record(0x100, 0x100));
        assert!(catch_up_record(0x23, 0x29));
        assert!(catch_up_record(0xFFFE, 2));
        assert!(!catch_up_record(0x100, 0x400));
        assert!(!catch_up_record(0x400, 0x100));
    }

    #[test]
    fn poke_lines_parse_and_slot() {
        let mut p = PokeSet::default();
        // Power frame F is record F - 2 and lands after the tick; frames 0 and 1 are before record -1.
        p.add_text("# c\n1300 FF84D9 1 1\n0 0xFF84D1 1 0x0\n1800 G.$4E8 2 0x0013 # the clock\n", "t", PokeClock::Power).unwrap();
        assert_eq!(p.len(), 3);
        assert_eq!(p.slots.keys().copied().collect::<Vec<_>>(), vec![(-1, true), (1298, true), (1798, true)]);
        assert_eq!(p.lines[2].addr, 0xFF84E8);
        // The game clock of a sidecar: frame k is before the tick of record k.
        let mut p = PokeSet::default();
        p.add_text("#!clock game\n#!unknown warn\n0 FFB274 1 2\n5 FFB36D 1 0x7F\n", "t", PokeClock::Power).unwrap();
        assert_eq!(p.slots.keys().copied().collect::<Vec<_>>(), vec![(0, false), (5, false)]);
        assert!(p.lines.iter().all(|l| l.lenient));
        // The command line form, with G.-$ below the base.
        let mut p = PokeSet::default();
        p.add_spec("10:G.-$7098:2:1", PokeClock::Game).unwrap();
        assert_eq!(p.lines[0].addr, 0xFF8000 - 0x7098);
    }

    #[test]
    fn bad_poke_lines_are_errors() {
        for bad in ["1 FF84D9 3 1", "1 FF84D9 2 1", "1 FF8000 2 0x10000", "1 FE0000 1 1", "1 FF84D9 1", "x FF84D9 1 1", "1 ZZ 1 1", "#!clock moon"] {
            let mut p = PokeSet::default();
            let text = if bad == "1 FF84D9 2 1" { "1 FF84D9 2 1\n" } else { bad };
            // FF84D9 is odd: a word there is an error.
            assert!(p.add_text(text, "t", PokeClock::Power).is_err(), "{bad}");
        }
    }

    #[test]
    fn pokes_reach_the_fields_and_unknown_addresses_are_errors() {
        let mut gm = game();
        let mut p = PokeSet::default();
        for spec in [
            "1:FF84D0:1:8",       // mode
            "1:FF84D1:1:0",       // sub mode
            "1:FF84D9:1:1",       // stage
            "1:G.$786:1:2",       // area
            "1:FF84E8:2:0x0013",  // time word: minutes and seconds
            "1:G.$508:2:0x1234",
            "1:G.$50E:1:2",       // kill all
            "1:G.$46B:1:1",       // free play
            "1:G.$757A:2:3",      // demo counter
            "1:FFB274:1:2",       // player 1 live flag
            "1:FFB317:1:3",       // lives (+$A3)
            "1:FFB2E0:2:0x0064",  // hp word (+$6C)
            "1:FFB374:4:0x00012345", // score (+$100)
        ] {
            p.add_spec(spec, PokeClock::Game).unwrap();
        }
        p.check(&mut gm).unwrap();
        // (1) the player base is 0xB274: +$6C = 0xB2E0, +$A3 = 0xB317, +$100 = 0xB374.
        p.apply(&mut gm, 1, false).unwrap();
        assert_eq!((gm.flow.mode, gm.flow.sub, gm.w.stage, gm.cam.s2.area), (8, 0, 1, 2));
        assert_eq!((gm.rank.time_min, gm.rank.time_sec), (0x00, 0x13));
        assert_eq!(gm.flow.timer508, 0x1234);
        assert_eq!(gm.spawn.g.kill_all, 2);
        assert!(gm.flow.front.sys.free_play);
        assert_eq!(gm.flow.front.demo_counter, 3);
        assert_eq!((gm.players[0].b.live, gm.players[0].lives, gm.players[0].b.hp, gm.players[0].score), (2, 3, 0x64, 0x12345));
        assert_eq!(p.summary(), "pokes: 13 lines, 13 writes applied, 0 not applied");
        // A byte that no modeled field covers is an error, before and during the run.
        let mut bad = PokeSet::default();
        bad.add_spec("1:FF0300:1:1", PokeClock::Game).unwrap();
        assert!(bad.check(&mut gm).is_err());
        assert!(bad.apply(&mut gm, 1, false).is_err());
        // The same line in a generated sidecar is skipped and counted.
        let mut lenient = PokeSet::default();
        lenient.add_text("#!clock game\n#!unknown warn\n1 FF0300 1 1\n", "t", PokeClock::Game).unwrap();
        lenient.check(&mut gm).unwrap();
        lenient.apply(&mut gm, 1, false).unwrap();
        assert_eq!(lenient.summary(), "pokes: 1 lines, 0 writes applied, 1 not applied");
    }
}
