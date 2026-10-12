//! Checks of the front end services against the original (MAME), on real ROM data.
//!
//! The printers and loaders are run with the real strings and pages and the result is compared,
//! cell by cell, with the tilemaps of the MAME power-on recording `re/out/traces/attract.mame.trc`
//! (made as described in `re/harness/README.md`; frame k of that file is video frame k + 2). A cell
//! is a code word and an attribute word. The tests need the ROM (`dino.zip` in the repo root) and
//! the recording; they pass without doing anything when either is missing (both are git-ignored).
//!
//! The record numbers are the frames where the original draws each screen (D4d section 3).

use crate::game::Game;
use crate::harness::TraceReader;
use dino_assets::AssetPack;
use std::path::Path;
use std::sync::OnceLock;

const ROM: &str = "../../../dino.zip";
const TRACE: &str = "../../../re/out/traces/attract.mame.trc";
/// Power on, a coin at video frame 600, start at 700 and the character select (re/harness/README.md).
const TRACE_COINSTART: &str = "../../../re/out/traces/coinstart.mame.trc";

/// Offsets of the layers in the trace's gfx region (base 0x900000).
const SCROLL1: usize = 0x8000;
const SCROLL2: usize = 0xC000;
const SCROLL3: usize = 0x10000;
const PALETTES: usize = 0x14000;

fn assets() -> Option<&'static AssetPack> {
    static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
    *A.get_or_init(|| {
        if !Path::new(ROM).exists() || !Path::new(TRACE).exists() || !Path::new(TRACE_COINSTART).exists() {
            eprintln!("mame_check: needs {ROM}, {TRACE} and {TRACE_COINSTART}, skipped");
            return None;
        }
        Some(Box::leak(Box::new(dino_extract::extract(Path::new(ROM)).expect("the ROM set loads"))))
    })
}

/// The gfx region (tilemaps and palettes) of the MAME recording after each of the given records.
fn mame(records: &[i32]) -> Vec<Vec<u8>> {
    mame_from(TRACE, records)
}

fn mame_from(trace: &str, records: &[i32]) -> Vec<Vec<u8>> {
    let mut rd = TraceReader::open(trace).expect("the recording opens");
    let last = *records.iter().max().unwrap();
    let mut out = Vec::new();
    while let Some(f) = rd.next() {
        if records.contains(&f) {
            out.push((f, rd.region("gfx").unwrap().data.clone()));
        }
        if f >= last {
            break;
        }
    }
    records.iter().map(|r| out.iter().find(|(f, _)| f == r).expect("record in the recording").1.clone()).collect()
}

fn cell_at(gfx: &[u8], base: usize, i: usize) -> (u16, u16) {
    let a = base + 4 * i;
    (u16::from_be_bytes([gfx[a], gfx[a + 1]]), u16::from_be_bytes([gfx[a + 2], gfx[a + 3]]))
}

/// The cells of layer `layer` (1, 2, 3) of the game that differ from the MAME image.
fn diff(g: &Game, gfx: &[u8], layer: u8) -> Vec<(usize, (u16, u16), (u16, u16))> {
    let (cells, base) = match layer {
        1 => (&g.vram.scroll1, SCROLL1),
        2 => (&g.vram.scroll2, SCROLL2),
        _ => (&g.vram.scroll3, SCROLL3),
    };
    (0..0x1000)
        .filter_map(|i| {
            let (m, p) = (cell_at(gfx, base, i), (cells[i].code, cells[i].attr));
            (m != p).then_some((i, p, m))
        })
        .collect()
}

fn assert_layer(g: &Game, gfx: &[u8], layer: u8, what: &str) {
    let d = diff(g, gfx, layer);
    assert!(d.is_empty(), "{what}: scroll{layer} differs in {} cells, (cell, port, MAME): {:?}", d.len(), &d[..d.len().min(40)]);
}

/// The palette words (of the four banks) that differ; `mask` drops the brightness nibble when a
/// fade may differ.
fn palette_diff(g: &Game, gfx: &[u8], mask: u16) -> Vec<usize> {
    (0..2048)
        .filter(|&i| {
            let m = u16::from_be_bytes([gfx[PALETTES + 2 * i], gfx[PALETTES + 2 * i + 1]]);
            (m & mask) != (g.pal.words[i] & mask)
        })
        .collect()
}

fn assert_palettes(g: &Game, gfx: &[u8], mask: u16, what: &str) {
    let bad = palette_diff(g, gfx, mask);
    assert!(bad.is_empty(), "{what}: {} palette words differ, first at word {:?}", bad.len(), &bad[..bad.len().min(4)]);
}

fn blank_game(a: &'static AssetPack) -> Game {
    let mut g = Game::new(a);
    g.flow_clear_screen(); // 0xAE0
    g
}

fn rom(g: &Game, addr: u32) -> &'static [u8] {
    g.assets.frontend.text.slice_from(addr)
}

/// What the comparison with a MAME record looks at, as differences of the port after its tick.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct Differences {
    /// Cells of scroll1, 2, 3 and palette words that differ.
    s1: usize,
    s2: usize,
    s3: usize,
    pal: usize,
    /// The sprite list differs (entry count of MAME, of the port).
    sprites: Option<(usize, usize)>,
    /// Layer control G.$28 (MAME, port), the title counter G.-$7022, the sound codes queued.
    layer: Option<(u16, u16)>,
    counter: Option<(u16, u16)>,
    sounds: Option<(Vec<u16>, Vec<u16>)>,
}

impl Differences {
    fn none(&self) -> bool {
        *self == Differences::default()
    }
}

fn ram16(ram: &[u8], a: u32) -> u16 {
    let o = (a - 0xFF0000) as usize;
    u16::from_be_bytes([ram[o], ram[o + 1]])
}

/// The sprite list MAME's builder (0x13FA6) wrote: the buffer that G.$69B4 (0xFFE9B4) selects. A
/// buffer without the end marker is the blank one the clear 0xAE0 leaves: no sprite (the port has
/// no list then either).
fn mame_sprites(ram: &[u8], gfx: &[u8]) -> Vec<(u16, u16, u16, u16)> {
    let base = if ram16(ram, 0xFFE9B4) != 0 { 0x4000 } else { 0 };
    let mut v = Vec::new();
    for i in 0..256 {
        let o = base + 8 * i;
        let w = |k: usize| u16::from_be_bytes([gfx[o + 2 * k], gfx[o + 2 * k + 1]]);
        if w(3) == 0xFF00 {
            return v;
        }
        v.push((w(0), w(1), w(2), w(3)));
    }
    Vec::new()
}

/// The sound codes queued between two records: the ring G.$3A (16-byte entries, write index G.$34).
fn mame_sounds(prev: &[u8], ram: &[u8]) -> Vec<u16> {
    let (mut a, b) = (ram16(prev, 0xFF8034) & 0x3F0, ram16(ram, 0xFF8034) & 0x3F0);
    let mut v = Vec::new();
    while a != b {
        v.push(ram16(ram, 0xFF803A + a as u32));
        a = (a + 0x10) & 0x3F0;
    }
    v
}

/// Runs the port from power on (`Game::flow_boot`) with the scripted input and compares it with the
/// MAME recording after every tick, records `0..=last`. `input(tick)` gives the player 1 and 2
/// buttons of that tick. Returns the differences of each record.
fn compare_power_on(
    a: &'static AssetPack,
    trace: &str,
    last: i32,
    input: impl Fn(i32) -> [u8; 3],
    mut before: impl FnMut(i32, &mut Game),
) -> (Game, Vec<Differences>) {
    let mut rd = TraceReader::open(trace).expect("the recording opens");
    rd.next(); // record -1, the sync state
    let mut g = Game::new(a);
    g.flow_boot();
    let mut prev_ram = rd.region("ram").unwrap().data.clone();
    let mut out = Vec::new();
    for k in 0..=last {
        g.w.input = input(k);
        before(k, &mut g);
        g.tick();
        assert_eq!(rd.next(), Some(k), "the recording has record {k}");
        let (gfx, ram) = (&rd.region("gfx").unwrap().data, &rd.region("ram").unwrap().data);
        let mut d = Differences {
            s1: diff(&g, gfx, 1).len(),
            s2: diff(&g, gfx, 2).len(),
            s3: diff(&g, gfx, 3).len(),
            pal: palette_diff(&g, gfx, 0xFFFF).len(),
            ..Default::default()
        };
        let (ms, ps): (Vec<_>, Vec<_>) = (
            mame_sprites(ram, gfx),
            g.sprites.entries.iter().map(|e| (e.x as u16, e.y as u16, e.code, e.attr)).collect(),
        );
        if ms != ps {
            d.sprites = Some((ms.len(), ps.len()));
        }
        let (ml, pl) = (ram16(ram, 0xFF8028), g.w.layer_ctrl);
        if ml != pl {
            d.layer = Some((ml, pl));
        }
        let (mc, pc) = (ram16(ram, 0xFF0FDE), g.flow.front.title);
        if mc != pc {
            d.counter = Some((mc, pc));
        }
        let (msn, psn) = (mame_sounds(&prev_ram, ram), g.w.sounds.clone());
        if msn != psn {
            d.sounds = Some((msn, psn));
        }
        prev_ram.clone_from(ram);
        out.push(d);
    }
    (g, out)
}

/// The records in `rows` (index = record) that differ in `what`, as ranges "a..=b".
fn ranges(rows: &[Differences], what: impl Fn(&Differences) -> bool) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut k = 0;
    while k < rows.len() {
        if what(&rows[k]) {
            let s = k;
            while k + 1 < rows.len() && what(&rows[k + 1]) {
                k += 1;
            }
            out.push(if s == k { format!("{s}") } else { format!("{s}..={k}") });
        }
        k += 1;
    }
    out.join(" ")
}

/// The boot screens (0x9DCF6) from power on: both texts, the clear and the palette set 0 that the
/// boot task loads after them, at the records where the original shows them.
#[test]
fn boot_screens_match_mame() {
    let Some(a) = assets() else { return };
    let records = [70, 100, 191, 250, 311, 312];
    let images = mame(&records);
    let mut g = Game::new(a);
    g.flow_boot();
    let mut next = 0;
    for tick in 0..=312 {
        g.tick();
        if next < records.len() && tick == records[next] {
            let gfx = &images[next];
            let what = format!("boot record {tick}");
            assert_layer(&g, gfx, 1, &what);
            if tick <= 312 {
                assert_layer(&g, gfx, 2, &what);
                assert_layer(&g, gfx, 3, &what);
                assert_palettes(&g, gfx, 0xFFFF, &what);
            }
            next += 1;
        }
    }
}

/// The title pieces one by one: the green background blocks (0x98B86 with the lists 0x9A5E6 and
/// 0x9A6E6), INSERT COIN (0xBB6), the two small lines and the Capcom sign (0x110C) at title frame 30
/// and the licence block at frame 360 (the whole title run is `power_on_boot_and_title_match_mame`).
#[test]
fn title_screen_texts_and_blocks_match_mame() {
    let Some(a) = assets() else { return };
    let images = mame(&[319, 320, 349, 679]);
    // (record 319 is mid-way through the fourth block: the frame ended while the original was
    // still writing it. The cells it had not written yet are the 27 last ones in the draw order.)
    let mut g = blank_game(a);
    // 0x9A468: four blocks of 16 x 8 cells
    for at in [0x90F000, 0x90F020, 0x90F400, 0x90F420] {
        g.flow_block2(at, rom(&g, 0x9A5E6), rom(&g, 0x9A6E6), 16, 8);
    }
    let late = diff(&g, &images[0], 2);
    assert_eq!(late.len(), 27, "record 319: the cells the original had not drawn yet");
    // row 14 from column 21 and all of row 15 of the block at 0x90F420 (cell = column * 16 + row + 3072)
    assert!(late.iter().all(|&(i, _, _)| i >= 3072 + 16 * 16 && i % 16 >= 14));
    assert_layer(&g, &images[1], 2, "title blocks, record 320");
    // the blink task 0x98874 prints INSERT COIN (0xBB6)
    g.text_bytes(rom(&g, 0x988A4), true);
    assert_layer(&g, &images[1], 1, "INSERT COIN, record 320");
    // title frame 30: 0x97F64, then the Capcom sign 0x110C (3 x 16 cells, attribute 0x0A)
    g.text_bytes(rom(&g, 0x97F64), true);
    g.flow_capcom_sign(0x10, 0x1A);
    assert_layer(&g, &images[2], 1, "title lines and Capcom sign, record 349");
    // title frame 360: clear, the licence block 0x97F9E, two cells at 0x90A6E8 (0x9A582); the third
    // copyright line only for regions other than ETC (G.$32 = 2 here)
    g.flow_clear_scroll1();
    g.text_bytes(rom(&g, 0x97F9E), true);
    g.hud_put(0x90A6E8, 0x40, 0);
    g.hud_put(0x90A6EC, 0x40, 0);
    assert_layer(&g, &images[3], 1, "licence block, record 679");
}

/// The four profile cards: label block 0x9AC16 (0xBB6), the card text (tall font 0x84F72, one of
/// 0x9B0F8, 0x9B192, 0x9B224, 0x9B2C2) and the labels 0x9ABF6 and 0x9AC02 (0xBB6).
#[test]
fn profile_cards_match_mame() {
    let Some(a) = assets() else { return };
    let records = [931, 1277, 1638, 1999];
    let images = mame(&records);
    for (k, text) in [0x9B0F8u32, 0x9B192, 0x9B224, 0x9B2C2].into_iter().enumerate() {
        let mut g = blank_game(a);
        g.text_bytes(rom(&g, 0x9AC16), true);
        g.text_tall(rom(&g, text));
        g.text_bytes(rom(&g, 0x9ABF6), true);
        g.text_bytes(rom(&g, 0x9AC02), true);
        assert_layer(&g, &images[k], 1, &format!("profile card {}, record {}", k + 1, records[k]));
    }
}

/// The ranking screen's pictures: palette set 4 (0x89E6), scroll2 page 0x16 four times (0x204CA)
/// and scroll3 page 0x7C eight times (0x20592), as 0x1516C draws them.
#[test]
fn ranking_pictures_match_mame() {
    let Some(a) = assets() else { return };
    let images = mame(&[2330]);
    let mut g = blank_game(a);
    g.flow_scene_palettes(4);
    for at in [0x90F000, 0x90F400, 0x90F800, 0x90FC00] {
        g.flow_page2(at, 0x16);
    }
    for k in 0..8 {
        g.flow_page3(0x913800 + 0x100 * k, 0x7C);
    }
    assert_layer(&g, &images[0], 2, "ranking, record 2330");
    assert_layer(&g, &images[0], 3, "ranking, record 2330");
    // the colours (the fade may still change the brightness nibble)
    assert_palettes(&g, &images[0], 0x0FFF, "ranking palette set 4");
}

/// The pictures of the character select screen (0xAD64): palette set 1, the two scroll1 panel
/// frames (0x20454, pages 5 and 6), scroll2 pages 1 and 2 (0x204CA) and scroll3 pages 6 and 9
/// (0x20592). The scroll1 loader is only used by this screen.
#[test]
fn select_pictures_match_mame() {
    let Some(a) = assets() else { return };
    let images = mame_from(TRACE_COINSTART, &[722]);
    let mut g = blank_game(a);
    g.flow_scene_palettes(1);
    g.flow_page1(0x90A000, 5);
    g.flow_page1(0x90B000, 6);
    g.flow_page2(0x90F000, 1);
    g.flow_page2(0x90F400, 2);
    g.flow_page3(0x913800, 6);
    g.flow_page3(0x913900, 9);
    // The second row page of scroll1 (cells 2048..4095) holds the two pages. In the same frame the
    // select code draws the panel of player 1 (face and name) over blank cells of the frame, 91
    // cells in all (the faces and names are T2.2); every other cell must be equal.
    let d: Vec<_> = diff(&g, &images[0], 1).into_iter().filter(|&(i, _, _)| i >= 2048).collect();
    assert_eq!(d.len(), 91, "select scroll1 pages: {:?}", &d[..d.len().min(10)]);
    assert!(d.iter().all(|&(_, p, _)| p == (0x20, 0)), "only blank cells of the frame are drawn over");
    assert_layer(&g, &images[0], 2, "select, record 722");
    assert_layer(&g, &images[0], 3, "select, record 722");
    // The select code changes one OBJ palette (slot 10, words 160..175) of the set.
    assert_eq!(palette_diff(&g, &images[0], 0x0FFF), (160..176).collect::<Vec<_>>(), "select palette set 1");
}

/// The block printers of the story slides (0x988C0): a scroll2 block of 8 x 6 cells at 0x90F1D4
/// (0x98B86, lists 0x98BD6 and 0x98C4E) in slide 2 and a scroll3 block of 4 x 3 cells at 0x913828
/// (0x98BAE, lists 0x98C36 and 0x98CAE) in slide 4. Only the cells of the blocks are compared.
#[test]
fn slide_blocks_match_mame() {
    let Some(a) = assets() else { return };
    let images = mame(&[5003, 5667]);
    let mut g = blank_game(a);
    g.flow_block2(0x90F1D4, rom(&g, 0x98BD6), rom(&g, 0x98C4E), 8, 6);
    let cells2: Vec<usize> = (0..8).flat_map(|c| (0..6).map(move |r| (0x90F1D4 - 0x90C000) / 4 + c * 16 + r)).collect();
    for &i in &cells2 {
        assert_eq!((g.vram.scroll2[i].code, g.vram.scroll2[i].attr), cell_at(&images[0], SCROLL2, i), "slide 2 block, cell {i}");
    }
    assert!(cells2.iter().any(|&i| g.vram.scroll2[i].code != 0x5800), "the block is not blank");
    g.flow_block3(0x913828, rom(&g, 0x98C36), rom(&g, 0x98CAE), 4, 3);
    let cells3: Vec<usize> = (0..4).flat_map(|c| (0..3).map(move |r| (0x913828 - 0x910000) / 4 + c * 8 + r)).collect();
    for &i in &cells3 {
        assert_eq!((g.vram.scroll3[i].code, g.vram.scroll3[i].attr), cell_at(&images[1], SCROLL3, i), "slide 4 block, cell {i}");
    }
    assert!(cells3.iter().any(|&i| g.vram.scroll3[i].code != 0x1C00), "the block is not blank");
}

/// The tall font texts of the first story slide (0x84F72 on 0x98CC6, then 0x98D14).
#[test]
fn slide_texts_match_mame() {
    let Some(a) = assets() else { return };
    let images = mame(&[4701, 4761]);
    let mut g = blank_game(a);
    g.text_tall(rom(&g, 0x98CC6));
    assert_layer(&g, &images[0], 1, "slide 1 first text, record 4701");
    g.text_tall(rom(&g, 0x98D14));
    assert_layer(&g, &images[1], 1, "slide 1 second text, record 4761");
}

/// Diagnostic report of the power-on comparison: the records that differ, by kind. Run with
/// `cargo test -p dino-game power_on_report -- --ignored --nocapture`; `DINO_LAST` (default 940) is
/// the last record, `DINO_FROM` the first one whose differences are listed in full.
#[test]
#[ignore]
fn power_on_report() {
    let Some(a) = assets() else { return };
    let last = std::env::var("DINO_LAST").ok().and_then(|v| v.parse().ok()).unwrap_or(940);
    let (_, rows) = compare_power_on(a, TRACE, last, |_| [0; 3], |_, _| {});
    println!("s1      {}", ranges(&rows, |d| d.s1 > 0));
    println!("s2      {}", ranges(&rows, |d| d.s2 > 0));
    println!("s3      {}", ranges(&rows, |d| d.s3 > 0));
    println!("palette {}", ranges(&rows, |d| d.pal > 0));
    println!("sprites {}", ranges(&rows, |d| d.sprites.is_some()));
    println!("layer   {}", ranges(&rows, |d| d.layer.is_some()));
    println!("counter {}", ranges(&rows, |d| d.counter.is_some()));
    println!("sounds  {}", ranges(&rows, |d| d.sounds.is_some()));
    let from: usize = std::env::var("DINO_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(25);
    for (k, d) in rows.iter().enumerate().skip(from) {
        if !d.none() {
            println!("record {k}: {d:?}");
        }
    }
}

/// A1 and A4, the whole boot and the title from power on: after every tick of records 25 to 918
/// the port equals the recording in all three tilemaps, all four palette banks, the sprite list (the
/// logo objects of pool A class 98), the layer control G.$28, the title counter G.-$7022 and the
/// sound codes queued (0xFF0B at 41 and 68, 0xFF00 at 68 and 315, the title tune 0x88 at 329).
/// Records 0 to 24 are the reset code clearing and testing the video RAM (the port starts from the
/// state it leaves at record 25; the first records differ in a few cells), 919 on the profile
/// cards (M1-05). The pictures of D4d A1 and A4 (records 100, 250, 330, 340, 420, 560, 700), the
/// text 1 and 2 frames, the layer change and the title counter are all inside this run.
#[test]
fn power_on_boot_and_title_match_mame() {
    let Some(a) = assets() else { return };
    let (_, rows) = compare_power_on(a, TRACE, 918, |_| [0; 3], |_, _| {});
    let bad: Vec<(usize, &Differences)> = rows.iter().enumerate().skip(25).filter(|(_, d)| !d.none()).collect();
    assert!(bad.is_empty(), "differs at records {} of {}: {:?}", bad.len(), rows.len() - 25, &bad[..bad.len().min(4)]);
}

/// The numbers of D4d A1 and A4 on the port itself (record k is video frame k + 2): text 1 from
/// video frame 72 to 192, text 2 from 193, the layer control 0x18D6 at video frame 317 and 0x1396 at
/// 321, the title counter 1 at video frame 322, INSERT COIN 30 frames on and 30 off, the small
/// lines at title frame 30, the licence block at title frame 360, the title tune queued at title
/// frame 10. The equality with the recording is `power_on_boot_and_title_match_mame`.
#[test]
fn title_timeline_numbers() {
    let Some(a) = assets() else { return };
    let mut g = Game::new(a);
    g.flow_boot();
    let at = |g: &Game, a: u32| {
        let c = g.vram.scroll1[((a - 0x908000) / 4) as usize & 0xFFF];
        (c.code, c.attr)
    };
    let insert_coin = 0x90A960u32; // column 18, row 56
    let text1 = |g: &Game| g.vram.scroll1.iter().filter(|c| c.code != 0x20).count();
    let mut seen = std::collections::BTreeMap::new();
    for k in 0..=700 {
        g.tick();
        if k == 69 || k == 70 || k == 190 || k == 191 || k == 311 || k == 312 {
            seen.insert(k, text1(&g));
        }
        if g.w.layer_ctrl == 0x18D6 {
            seen.entry(1000).or_insert(k as usize);
        }
        if g.w.layer_ctrl == 0x1396 {
            seen.entry(1001).or_insert(k as usize);
        }
        if g.flow.front.title == 1 {
            seen.entry(1002).or_insert(k as usize);
        }
        let on = at(&g, insert_coin).0 == 0x49; // the I of INSERT
        if k == 320 || k == 349 {
            assert!(on, "INSERT COIN on at {k}");
        }
        if k == 350 || k == 379 {
            assert!(!on, "INSERT COIN off at {k}");
        }
        if k == 380 {
            assert!(on, "INSERT COIN on again at {k}");
        }
        if g.w.sounds.contains(&0x88) {
            seen.entry(1003).or_insert(k as usize);
        }
        if k == 348 {
            assert_eq!(g.vram.scroll1[((0x90A868u32 - 0x908000) / 4) as usize].code, 0x20, "no Capcom sign before title frame 30");
        }
        if k == 349 {
            assert_eq!(at(&g, 0x90A868 + 0x80 * 0).1, 0x0A, "the Capcom sign (attribute 0x0A) at title frame 30");
        }
    }
    assert_eq!(seen[&69], 0, "black before video frame 72");
    // (text 1 is 28 letters, the warning of text 2 is 288)
    assert_eq!((seen[&70], seen[&190]), (28, 28), "text 1 from video frame 72 to 192");
    assert_eq!((seen[&191], seen[&311]), (288, 288), "text 2 from video frame 193 to 313");
    assert_eq!(seen[&312], 0, "the screen is blank when the boot ends");
    assert_eq!((seen[&1000], seen[&1001], seen[&1002], seen[&1003]), (315, 319, 320, 329));
}

/// The press start screen (0x983EE, task 0x978EE): called in the frame the system task (M1-04)
/// would call it, here frame 607 of the coin start recording (the coin is at video frame 600).
/// Scroll2 (the green blocks), scroll3, all palettes, the sprite list (the two fixed logo pieces)
/// and the layer control equal the recording from the first frame of the task (610) to the start of
/// the fade out (697). Scroll1 holds the prompts of the system task (INSERT COIN / PUSH START
/// BUTTON and CREDIT, M1-04) and is not compared.
#[test]
fn press_start_screen_matches_mame() {
    let Some(a) = assets() else { return };
    let (g, rows) = compare_power_on(a, TRACE_COINSTART, 697, |_| [0; 3], |k, g| {
        if k == 607 {
            g.flow.front.task.kill(); // the system task ends the attract task
            g.flow_press_start_screen();
        }
    });
    if std::env::var("DINO_SHOW").is_ok() {
        for k in 604..=616 {
            println!("record {k}: {:?}", rows[k]);
        }
    }
    // the screen is really there: the logo pieces are in the sprite list and the blocks are green
    assert_eq!(g.sprites.entries.len(), 19, "the two fixed logo pieces");
    assert_eq!(g.vram.scroll2[3072].code, 0x5801);
    assert_eq!(g.w.layer_ctrl, 0x12D6);
    let bad: Vec<(usize, &Differences)> = rows.iter().enumerate().skip(610).filter(|(_, d)| d.s2 + d.s3 + d.pal > 0 || d.sprites.is_some() || d.layer.is_some()).collect();
    assert!(bad.is_empty(), "press start screen differs at records {:?}", &bad[..bad.len().min(6)]);
}

/// The ticks at which the layer control becomes 0x18D6 (the attract task's first pass) and 0x1396
/// (the title) when the port boots with `input(tick)` held.
fn layer_change_ticks(a: &'static AssetPack, last: i32, input: impl Fn(i32) -> [u8; 3]) -> (Option<i32>, Option<i32>) {
    let mut g = Game::new(a);
    g.flow_boot();
    let (mut first, mut title) = (None, None);
    for k in 0..=last {
        g.w.input = input(k);
        g.tick();
        if g.w.layer_ctrl == 0x18D6 && first.is_none() {
            first = Some(k);
        }
        if g.w.layer_ctrl == 0x1396 && title.is_none() {
            title = Some(k);
        }
    }
    (first, title)
}

/// A2, the boot screens end early when player 1 or 2 presses attack or jump (mask 0x30) after the
/// first 30 frames of a screen; player 3 cannot skip. Three recordings from power on with the
/// harness (`DINO_CLOCK=video DINO_BOOT=re/harness/scripts/attract.txt DINO_SYNC=vblank:1
/// re/harness/record.sh SCRIPT NAME 340`; scripts with one line, `98-101:A` for NAME `boot_skip_a`,
/// `228-231:2J` for `boot_skip_b`; the third is the plain recording with the same press of player 3
/// that changes nothing) are compared after every tick, records 25 to 335: the screens, palettes,
/// sprite list, layer control and sounds are equal, so the layer control changes at the same
/// frames (224 and 231, where the plain boot has 315). The skip tests of D4d A2 (video frames 226,
/// 234 and none) are the same runs with the probe's own input timing (its frame f is the record
/// f - 2, a press at its frame 230 is seen one frame earlier by a task than the script line `230`).
#[test]
fn boot_screens_skip_like_the_original() {
    let Some(a) = assets() else { return };
    let cases: [(&str, &str, usize, u8, i32, i32); 3] = [
        ("../../../re/out/traces/boot_skip_a.mame.trc", "player 1 attack", 0, 0x10, 98, 224),
        ("../../../re/out/traces/boot_skip_b.mame.trc", "player 2 jump", 1, 0x20, 228, 231),
        (TRACE, "player 3 attack", 2, 0x10, 98, 315),
    ];
    for (trace, what, who, bit, from, layer_at) in cases {
        if !Path::new(trace).exists() {
            eprintln!("mame_check: {trace} is missing, {what} skipped");
            continue;
        }
        let press = |k: i32| -> [u8; 3] {
            let mut v = [0; 3];
            if (from..from + 4).contains(&k) {
                v[who] = bit;
            }
            v
        };
        let (_, rows) = compare_power_on(a, trace, 335, press, |_, _| {});
        let bad: Vec<(usize, &Differences)> = rows.iter().enumerate().skip(25).filter(|(_, d)| !d.none()).collect();
        assert!(bad.is_empty(), "{what}: differs at records {:?}", &bad[..bad.len().min(4)]);
        assert_eq!(layer_change_ticks(a, 335, press), (Some(layer_at), Some(layer_at + 4)), "{what}");
    }
}
