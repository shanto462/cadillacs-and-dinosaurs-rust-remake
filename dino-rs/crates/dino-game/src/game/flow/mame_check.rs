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
    /// The attract state byte G.-$7030 (MAME, port).
    state: Option<(u8, u8)>,
    /// The six scroll shadows G.$6C4, $6C8, $744, $748, $7E4, $7E8 (MAME, port). Not part of `none`: the
    /// stub screens do not scroll as the original's do.
    cams: Option<([u16; 6], [u16; 6])>,
    /// The variables of the system task and the game start that differ (name, MAME, port), see
    /// `system_fields`. Not part of `none`: the demos and other screens of MAME have their own values.
    sys: Vec<(&'static str, u16, u16)>,
}

impl Differences {
    /// The system variables that matter in every screen of the attract loop: coins, credits, the press
    /// start flag, in game, started and the wait counter (not the fade mask, the mode and G.$4D5, which
    /// the stubs and the real screens set differently).
    fn core_sys(&self) -> Vec<&(&'static str, u16, u16)> {
        self.sys.iter().filter(|(n, _, _)| !n.starts_with("fade") && !n.starts_with("mode") && !n.starts_with("G.$4D5")).collect()
    }

    /// Nothing differs in the screens, the layer control, the title counter, the sounds and the state byte.
    fn none(&self) -> bool {
        Differences { sys: Vec::new(), cams: None, ..self.clone() } == Differences::default()
    }
}

/// The system task's and the game start's variables: (name, address, MAME's value of the byte or
/// word at it, the port's). Compared by the coin and start tests.
fn system_fields(g: &Game, ram: &[u8]) -> Vec<(&'static str, u16, u16)> {
    let b = |a: u32| ram[(a - 0xFF0000) as usize] as u16;
    let s = &g.flow.front.sys;
    [
        ("coins P1 (G.$47B)", b(0xFF847B), s.slots[0].coins as u16),
        ("credits P1 (G.$47D)", b(0xFF847D), s.slots[0].credits as u16),
        ("credits pool (G.$46A)", b(0xFF846A), s.credits as u16),
        ("press start up (G.$46D)", b(0xFF846D), s.press_up as u16),
        ("in game (G.$4CC)", b(0xFF84CC), g.flow.in_game as u16),
        ("started (G.$4CD)", b(0xFF84CD), g.flow.started as u16),
        ("G.$4D5", b(0xFF84D5), g.spawn.g.b4d5 as u16),
        ("mode (G.$4D0)", b(0xFF84D0), g.flow.mode as u16),
        ("fade mask (G.$44A)", b(0xFF844A), g.fade.mask as u16),
        ("wait counter (G.$4F6)", ram16(ram, 0xFF84F6), g.rank.w4f6),
    ]
    .into_iter()
    .filter(|&(_, m, p)| m != p)
    .map(|(n, m, p)| (n, m, p))
    .collect()
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
    compare_power_on_with(a, trace, last, input, |k, g, _| before(k, g), |_, _, _, _| {})
}

/// `compare_power_on` with a look at the game and the recording's RAM and gfx after each tick, for what the
/// `Differences` do not hold.
fn compare_power_on_with(
    a: &'static AssetPack,
    trace: &str,
    last: i32,
    input: impl Fn(i32) -> [u8; 3],
    mut before: impl FnMut(i32, &mut Game, &[u8]),
    mut after: impl FnMut(i32, &Game, &[u8], &[u8]),
) -> (Game, Vec<Differences>) {
    let mut rd = TraceReader::open(trace).expect("the recording opens");
    rd.next(); // record -1, the sync state
    let mut g = Game::new(a);
    g.flow_boot();
    let mut prev_ram = rd.region("ram").unwrap().data.clone();
    let mut out = Vec::new();
    for k in 0..=last {
        g.w.input = input(k);
        before(k, &mut g, &prev_ram);
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
        d.sys = system_fields(&g, ram);
        let (ms, ps) = (ram[0xFF0FD0 - 0xFF0000], g.flow.front.attract_state);
        if ms != ps {
            d.state = Some((ms, ps));
        }
        let c = &g.cam;
        let (mc, pc) = (
            [0xFF86C4, 0xFF86C8, 0xFF8744, 0xFF8748, 0xFF87E4, 0xFF87E8].map(|x| ram16(ram, x)),
            [c.cam1_x, c.cam1_y, c.s2.cam_x, c.s2.cam_y, c.s3.cam_x, c.s3.cam_y],
        );
        if mc != pc {
            d.cams = Some((mc, pc));
        }
        after(k, &g, ram, gfx);
        let (msn, psn) = (mame_sounds(&prev_ram, ram), g.w.sounds.clone());
        if msn != psn {
            d.sounds = Some((msn, psn));
        }
        prev_ram.clone_from(ram);
        out.push(d);
    }
    (g, out)
}

/// The demo (0x99E88) clears the RAM it uses, the state byte among it, and restores it 3 frames later;
/// the stub of the demo does not (M1-12). Takes these runs of 4 records or fewer, where MAME has
/// 0 and the port the real state, out of the differences.
fn drop_demo_blips(rows: &mut [Differences]) {
    let mut k = 0;
    while k < rows.len() {
        if matches!(rows[k].state, Some((0, p)) if p != 0) {
            let s = k;
            while k + 1 < rows.len() && rows[k + 1].state == rows[s].state {
                k += 1;
            }
            if k - s < 4 {
                rows[s..=k].iter_mut().for_each(|d| d.state = None);
            }
        }
        k += 1;
    }
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

/// A5 (M1-05 and M1-06), the four profile cards: the whole routine 0x9A7E6 against `attract3.mame.trc`, in
/// the five runs it has there (called at the records 919, 7477, 14014, 20582 and 27119). In every record from
/// the call to the one after the clear that ends it, the three tilemaps, the four palette banks, the sprite
/// list (the portrait and the small hero of the cards), the layer control, the sound codes queued (the music
/// 0x13, the hits 0x38, the voices 0x4B to 0x4E and the poses 0x40 to 0x43 of the small hero), the scroll
/// shadows G.$6C4 .. G.$7E8, the counter G.-$7018 and its table index G.-$7016 equal the recording. The first
/// run is 1399 frames long, the later ones 1407 (the routine returns in the pass of the counter 0x4B0: the
/// records 2318, 8884, ...). The card starts (counters 10, 310, 610, 910: records 930, 1276, 1637, 1998) draw
/// the text at once and the objects' first sprites come two frames later (records 932, 1278, 1639, 2000: 12,
/// 30, 15 and 20 sprites). The screen shows the scroll registers 2 frames after the shadows (cpsa of the
/// recording): that is the renderer's, not compared here.
#[test]
fn profile_cards_run_like_the_original() {
    let Some(a) = assets() else { return };
    let trace = "../../../re/out/traces/attract3.mame.trc";
    if !Path::new(trace).exists() {
        eprintln!("mame_check: {trace} is missing, skipped");
        return;
    }
    // (record of the call, the last record that equals the recording: two after the routine returned)
    let runs: [(usize, usize); 5] = [(919, 2319), (7477, 8885), (14014, 15422), (20582, 21990), (27119, 28527)];
    let mut problems: Vec<String> = Vec::new();
    let mut sprites = std::collections::BTreeMap::new();
    let mut ends = Vec::new();
    let mut was_up = false;
    let (_, rows) = compare_power_on_with(a, trace, 28527, |_| [0; 3], |_, _, _| {}, |k, g, ram, _gfx| {
        let up = g.flow.front.stub == super::attract::Stub::Profiles;
        if up {
            let (m, p) = ((ram16(ram, 0xFF0FE8), ram16(ram, 0xFF0FEA)), (g.flow.front.task.count, g.flow.front.task.sub));
            if m != p && problems.len() < 6 {
                problems.push(format!("record {k}: counter and index (MAME) {m:?} (port) {p:?}"));
            }
        }
        if was_up && !up {
            ends.push(k);
        }
        was_up = up;
        if [932, 1278, 1639, 2000].contains(&k) {
            sprites.insert(k, g.sprites.entries.len());
        }
    });
    assert!(problems.is_empty(), "{problems:?}");
    for (from, to) in runs {
        let bad: Vec<(usize, &Differences)> = rows
            .iter()
            .enumerate()
            .take(to + 1)
            .skip(from)
            .filter(|(_, d)| d.s1 + d.s2 + d.s3 + d.pal > 0 || d.sprites.is_some() || d.layer.is_some() || d.sounds.is_some() || d.cams.is_some() || d.state.is_some())
            .collect();
        assert!(bad.is_empty(), "profile run {from}: {} records differ, first {:?}", bad.len(), &bad[..bad.len().min(3)]);
    }
    assert_eq!(ends, [2318, 8884, 15421, 21989, 28526], "the routine returns after 1399 and then 1407 frames");
    assert_eq!(sprites.values().copied().collect::<Vec<_>>(), [12, 30, 15, 20], "the sprites of the cards at their starts: {sprites:?}");
}

/// A6 (M1-07), the ranking screen "BRAVES": the whole routine 0x1516C (through 0x983D8) in the nine runs it has
/// in `attract3.mame.trc` (called at the records 2320, 6449, 8886, 11241, 15423, 19554, 21991, 24346 and 28528,
/// after the profile cards, the story slides, the profile cards, the demo and so on), from the call to the frame
/// it returns (422 frames the first time, 423 later; the records before the frame it returns, which is the first
/// one of the next screen). In every record the tilemaps (the label block and the ten
/// rows of the default table, the desert pages, the cut of the drawing by the video frame boundary), the palettes
/// (set 4 and the fade of the title after it), the sprite list (the Cadillac with its two heroes, the clouds),
/// the layer control, the sound codes (the stop code, the music 0x80 at the second frame, the ambient 0x70 when the
/// car has arrived), the six scroll shadows (the desert goes by, 8 and 4 pixels a frame), the wait counter G.$7018
/// and the random number state G.$14 (one per hero a frame, the clouds) equal the recording. The demo (4 runs) is a
/// stub that plays none of its random numbers, so the state G.$14 is set to the recording's value in the frame it
/// returns (M1-12 removes this). The story slides, the profile cards and the opening movie draw their own (the slides
/// none, see `story_slides_run_like_the_original`), so no ranking needs the state set in its own frame.
#[test]
fn ranking_runs_like_the_original() {
    let Some(a) = assets() else { return };
    let trace = "../../../re/out/traces/attract3.mame.trc";
    if !Path::new(trace).exists() {
        eprintln!("mame_check: {trace} is missing, skipped");
        return;
    }
    // (the frame it is called, the frames it lasts)
    let runs: [(usize, usize); 9] =
        [(2320, 422), (6449, 423), (8886, 423), (11241, 423), (15423, 423), (19554, 423), (21991, 423), (24346, 423), (28528, 423)];
    let mut problems: Vec<String> = Vec::new();
    let mut returned = Vec::new();
    let mut was_up = false;
    let (_, rows) = compare_power_on_with(
        a,
        trace,
        28528 + 423,
        |_| [0; 3],
        |k, g, prev| {
            // the demo (M1-12) is a stub that plays none of its random numbers: the state it leaves is set from the
            // recording in the frame the stub returns (the frame the story slides start or the clear before the
            // ranking of state 6). The slides, the profile cards and the movie play theirs.
            if [4668, 11239, 17773, 24344].contains(&k) {
                g.w.rng = ram16(prev, 0xFF8014);
            }
        },
        |k, g, ram, _| {
            let up = g.flow.front.stub == super::attract::Stub::Ranking;
            if up {
                if g.w.rng != ram16(ram, 0xFF8014) && problems.len() < 6 {
                    problems.push(format!("record {k}: random number state (MAME) {:04X} (port) {:04X}", ram16(ram, 0xFF8014), g.w.rng));
                }
                if g.flow.front.task.state == super::ranking::rst::PASS && g.flow.front.task.count != ram16(ram, 0xFFF018) && problems.len() < 6 {
                    problems.push(format!("record {k}: wait counter (MAME) {:04X} (port) {:04X}", ram16(ram, 0xFFF018), g.flow.front.task.count));
                }
            }
            if was_up && !up {
                returned.push(k as usize);
            }
            was_up = up;
        },
    );
    assert!(problems.is_empty(), "{problems:?}");
    for (from, len) in runs {
        let bad: Vec<(usize, &Differences)> = rows
            .iter()
            .enumerate()
            .take(from + len)
            .skip(from)
            .filter(|(_, d)| d.s1 + d.s2 + d.s3 + d.pal > 0 || d.sprites.is_some() || d.layer.is_some() || d.sounds.is_some() || d.cams.is_some() || d.state.is_some())
            .collect();
        assert!(bad.is_empty(), "ranking run {from}: {} records differ, first {:?}", bad.len(), &bad[..bad.len().min(3)]);
    }
    assert_eq!(returned, runs.map(|(s, l)| s + l), "the routine returns after 422 and then 423 frames");
}

/// A8 (M1-08), the five story slides: the whole routine 0x988C0 in the two runs it has in `attract3.mame.trc`
/// (called at the records 4668 and 17773, the frame the demo's fade ends; the demo is a stub, so the screen before
/// the first full picture, the two records of the clear, is not compared). From the record two frames after
/// the call (the palettes and the first pictures) to the one before the ranking starts, every record equals the
/// recording: the three tilemaps (the pictures, the hard cut, the two fades, the blank, the captions of the tall
/// font), the four palette banks (the fades step 16 times), the sprite list (none: 0xAE0 blanks it and the routine
/// builds none), the layer control (0x12D6, 0x1396 in slide 4, 0x12D6), the sound codes (the music 0x24 and 0x25,
/// the stop code of the state routine), the six scroll shadows (scroll3 X 0x180 and the pan of 49 frames), the attract
/// state byte. The routine is 1779 frames long (the stub it replaces had the same length) and returns in the record
/// the stop code is queued (6447 and 19552).
#[test]
fn story_slides_run_like_the_original() {
    let Some(a) = assets() else { return };
    let trace = "../../../re/out/traces/attract3.mame.trc";
    if !Path::new(trace).exists() {
        eprintln!("mame_check: {trace} is missing, skipped");
        return;
    }
    let runs = [4668usize, 17773];
    let mut returned = Vec::new();
    let mut was_up = false;
    // the random number state G.$14 (port, MAME) in the first and the last record that is compared
    let mut rng = std::collections::BTreeMap::new();
    let (_, rows) = compare_power_on_with(a, trace, 19554, |_| [0; 3], |_, _, _| {}, |k, g, ram, _| {
        let up = g.flow.front.stub == super::attract::Stub::Slides;
        if was_up && !up {
            returned.push(k as usize);
        }
        was_up = up;
        if runs.iter().any(|&s| k as usize == s + 2 || k as usize == s + 1780) {
            rng.insert(k as usize, (g.w.rng, ram16(ram, 0xFF8014)));
        }
    });
    assert_eq!(returned, runs.map(|s| s + 1779), "the routine is 1779 frames long");
    for s in runs {
        let bad: Vec<(usize, &Differences)> = rows
            .iter()
            .enumerate()
            .take(s + 1781)
            .skip(s + 2)
            .filter(|(_, d)| d.s1 + d.s2 + d.s3 + d.pal > 0 || d.sprites.is_some() || d.layer.is_some() || d.sounds.is_some() || d.cams.is_some() || d.state.is_some())
            .collect();
        assert!(bad.is_empty(), "slides run {s}: {} records differ, first {:?}", bad.len(), &bad[..bad.len().min(3)]);
        // the slides draw no random number (neither does the recording): G.$14 is the same in both ends of the run
        let (first, last) = (rng[&(s + 2)], rng[&(s + 1780)]);
        assert_eq!((first.0, first.1), (last.0, last.1), "slides run {s}: the random number state changed: {first:04X?} {last:04X?}");
    }
}

/// The wait counter G.$4F6 through the story slides with free play on (`m108_free`: G.$46B poked to 1 at power frame
/// 100 as in `m104_free`, 6700 records: `DINO_CLOCK=video DINO_BOOT=re/harness/scripts/attract.txt DINO_SYNC=vblank:1
/// DINO_POKES=re/harness/pokes/freeplay_power.txt re/harness/record.sh re/harness/scripts/attract.txt m108_free 6700`):
/// the counter counts a frame at a time while the system task runs and misses the frames in
/// which the attract task is busy past the end of the video frame (`front.sys_skip`). The demo before the slides is
/// a stub that does not reset the counter as the original's RAM clear does, so the counts since the start of the
/// slides are compared: they are equal in every record from the call of the routine (4668) to the start of
/// the ranking (6450) and its first frames (6458). The last clear of the routine misses two counts (it is followed by
/// the clear of the state routine without a break), the clears before it one each.
#[test]
fn story_slides_wait_counter_matches_mame() {
    let Some(a) = assets() else { return };
    let trace = "../../../re/out/traces/m108_free.mame.trc";
    if !Path::new(trace).exists() {
        eprintln!("mame_check: {trace} is missing (see the doc of this test), skipped");
        return;
    }
    let s = 4668usize;
    let mut series = Vec::new();
    compare_power_on_with(a, trace, 6458, |_| [0; 3], |k, g, _| {
        if k == 99 {
            g.flow.front.sys.free_play = true; // the poke
        }
    }, |k, g, ram, _| {
        if k as usize >= s {
            series.push((k as usize, ram16(ram, 0xFF84F6), g.rank.w4f6));
        }
    });
    let (m0, p0) = (series[0].1, series[0].2);
    let bad: Vec<_> = series.iter().filter(|&&(_, m, p)| m.wrapping_sub(m0) != p.wrapping_sub(p0)).take(4).collect();
    assert!(bad.is_empty(), "the counts since record {s} differ (record, MAME, port): {bad:?}");
    // 1790 frames, 10 passes missed: one at each of the first five clears, two at the last one, three round the
    // start of the ranking
    let counted = series.last().unwrap().1.wrapping_sub(m0);
    assert_eq!(counted as usize, 6458 - s - 10, "the counts of 1790 frames");
}

/// A9 (M1-09 to M1-11), the opening movie: the routine 0x9BF50 with its timeline engine, the helpers and all the handlers
/// (the captions, the Capcom sign, the jungle with the two hunters, the flock, the golden raptors and the brown
/// dinosaur, the logo, the fade out) in the two runs it has in `attract3.mame.trc` (called at the records 11664 and
/// 24769, the frame the ranking of state 6 returns). In every record from the call to the one the routine returns in (1745
/// frames later: the records 13409 and 26514, where the state byte becomes 0) the port equals the recording: the three
/// tilemaps (the clear, the captions of the tall font, the 16 step fades, the Capcom sign, the black bars and the jungle
/// pictures), the four palette banks (the logo palettes at the counter 1350), the sprite list, the layer control, the
/// sound codes (the stop code of the end of the ranking, 0x61, 0xEA, 0x73, 0x75, 0x36, the title tune 0x88, the stop code
/// of the end), the six scroll shadows (scroll2 X -12 in the Capcom part, the scroll of the jungle and its shake), the
/// attract state byte, the counter G.-$7004, the index G.-$7002 and the random number state G.$14 (the birds, the
/// feathers, the raptors and the shake draw it; the first number is at record 12413). The records after it (the title)
/// are the business of `attract_loop_state_and_titles_match_mame`.
const MOVIE_REAL: usize = 1746;

#[test]
fn opening_movie_runs_like_the_original() {
    let Some(a) = assets() else { return };
    let trace = "../../../re/out/traces/attract3.mame.trc";
    if !Path::new(trace).exists() {
        eprintln!("mame_check: {trace} is missing, skipped");
        return;
    }
    let runs = [11664usize, 24769];
    let mut returned = Vec::new();
    let mut was_up = false;
    let mut problems: Vec<String> = Vec::new();
    let (_, rows) = compare_power_on_with(
        a,
        trace,
        26514,
        |_| [0; 3],
        |k, g, prev| {
            // the demo stub (4 runs) plays none of its random numbers (see `ranking_runs_like_the_original`)
            if [4668, 11239, 17773, 24344].contains(&k) {
                g.w.rng = ram16(prev, 0xFF8014);
            }
        },
        |k, g, ram, _| {
            let up = g.flow.front.stub == super::attract::Stub::Movie;
            if was_up && !up {
                returned.push(k as usize);
            }
            was_up = up;
            for &s in &runs {
                let k = k as usize;
                if k > s + 2 && k < s + MOVIE_REAL && problems.len() < 6 {
                    let (m, p) = ((ram16(ram, 0xFF0FFC), ram16(ram, 0xFF0FFE)), (g.flow.front.task.count, g.flow.front.task.sub));
                    if m != p {
                        problems.push(format!("record {k}: counter and index (MAME) {m:?} (port) {p:?}"));
                    }
                }
                if k >= s && k < s + MOVIE_REAL && g.w.rng != ram16(ram, 0xFF8014) && problems.len() < 6 {
                    problems.push(format!("record {k}: random number state (MAME) {:04X} (port) {:04X}", ram16(ram, 0xFF8014), g.w.rng));
                }
            }
        },
    );
    assert!(problems.is_empty(), "{problems:?}");
    for s in runs {
        let bad: Vec<(usize, &Differences)> = rows
            .iter()
            .enumerate()
            .take(s + MOVIE_REAL)
            .skip(s)
            .filter(|(_, d)| d.s1 + d.s2 + d.s3 + d.pal > 0 || d.sprites.is_some() || d.layer.is_some() || d.sounds.is_some() || d.cams.is_some() || d.state.is_some())
            .collect();
        assert!(bad.is_empty(), "movie run {s}: {} records differ, first {:?}", bad.len(), &bad[..bad.len().min(3)]);
    }
    assert_eq!(returned, runs.map(|s| s + 1745), "the routine returns 1745 frames after its call");
}

/// Diagnostic for M1-10 and M1-11: the first records of the first movie run (called at 11664) that differ from
/// `attract3.mame.trc`, with both sprite lists, the sounds and the random number state. Run with
/// `DINO_FROM=12362 DINO_COUNT=6 cargo test -p dino-game movie_report -- --ignored --nocapture`.
#[test]
#[ignore]
fn movie_report() {
    let Some(a) = assets() else { return };
    let trace = "../../../re/out/traces/attract3.mame.trc";
    let from: usize = std::env::var("DINO_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(11664);
    let count: usize = std::env::var("DINO_COUNT").ok().and_then(|v| v.parse().ok()).unwrap_or(5);
    let last: usize = std::env::var("DINO_LAST").ok().and_then(|v| v.parse().ok()).unwrap_or(13409);
    let mut shown = 0;
    let mut first_bad = None;
    compare_power_on_with(
        a,
        trace,
        last as i32,
        |_| [0; 3],
        |k, g, prev| {
            if [4668, 11239].contains(&k) {
                g.w.rng = ram16(prev, 0xFF8014);
            }
        },
        |k, g, ram, gfx| {
            let k = k as usize;
            if k < from || shown >= count {
                return;
            }
            let (ms, ps): (Vec<_>, Vec<_>) = (
                mame_sprites(ram, gfx),
                g.sprites.entries.iter().map(|e| (e.x as u16, e.y as u16, e.code, e.attr)).collect(),
            );
            let (s1, s2, s3) = (diff(g, gfx, 1).len(), diff(g, gfx, 2).len(), diff(g, gfx, 3).len());
            let pal = palette_diff(g, gfx, 0xFFFF).len();
            let cams = [0xFF86C4, 0xFF86C8, 0xFF8744, 0xFF8748, 0xFF87E4, 0xFF87E8].map(|x| ram16(ram, x));
            let pc = &g.cam;
            let pcams = [pc.cam1_x, pc.cam1_y, pc.s2.cam_x, pc.s2.cam_y, pc.s3.cam_x, pc.s3.cam_y];
            let rng = (ram16(ram, 0xFF8014), g.w.rng);
            let bad = ms != ps || s1 + s2 + s3 + pal > 0 || cams != pcams || g.w.layer_ctrl != ram16(ram, 0xFF8028);
            if bad && first_bad.is_none() {
                first_bad = Some(k);
            }
            if first_bad.is_some() {
                shown += 1;
                println!("record {k}: s1 {s1} s2 {s2} s3 {s3} pal {pal} cams MAME {cams:X?} port {pcams:X?} rng MAME {:04X} port {:04X}", rng.0, rng.1);
                if pal > 0 {
                    let bad = palette_diff(g, gfx, 0xFFFF);
                    let w = |i: usize| u16::from_be_bytes([gfx[PALETTES + 2 * i], gfx[PALETTES + 2 * i + 1]]);
                    println!("  palette words differ at {:?} (MAME, port): {:?}", &bad[..bad.len().min(8)], bad.iter().take(8).map(|&i| (w(i), g.pal.words[i])).collect::<Vec<_>>());
                }
                if ms != ps {
                    println!("  MAME sprites {}: {:?}", ms.len(), &ms[..ms.len().min(12)]);
                    println!("  port sprites {}: {:?}", ps.len(), &ps[..ps.len().min(12)]);
                    let diff_at = ms.iter().zip(ps.iter()).position(|(m, p)| m != p);
                    println!("  first different entry {diff_at:?}");
                }
            }
        },
    );
}

/// The wait counter G.$4F6 through the opening movie with free play on (`m110_free`: the same recording as `m108_free`, 13600
/// records): the counts since the call of the routine (11664) are equal in every record to the one it returns in (13409). The movie
/// loses one count at each of its five clears (the first, the three of the first handlers and the last handler's), always in the
/// second of the two frames, nothing else (the sleep of 120 frames in the handler of the counter 1350 does not stop the system task).
#[test]
fn opening_movie_wait_counter_matches_mame() {
    let Some(a) = assets() else { return };
    let trace = "../../../re/out/traces/m110_free.mame.trc";
    if !Path::new(trace).exists() {
        eprintln!("mame_check: {trace} is missing (record it like `m108_free` with 13600 records), skipped");
        return;
    }
    let s = 11664usize;
    let mut series = Vec::new();
    compare_power_on_with(a, trace, (s + 1745) as i32, |_| [0; 3], |k, g, prev| {
        if k == 99 {
            g.flow.front.sys.free_play = true; // the poke
        }
        if [4668, 11239].contains(&k) {
            g.w.rng = ram16(prev, 0xFF8014); // the demo stub
        }
    }, |k, g, ram, _| {
        if k as usize >= s {
            series.push((k as usize, ram16(ram, 0xFF84F6), g.rank.w4f6));
        }
    });
    let (m0, p0) = (series[0].1, series[0].2);
    let bad: Vec<_> = series.iter().filter(|&&(_, m, p)| m.wrapping_sub(m0) != p.wrapping_sub(p0)).take(4).collect();
    assert!(bad.is_empty(), "the counts since record {s} differ (record, MAME, port): {bad:?}");
    let counted = series.last().unwrap().1.wrapping_sub(m0);
    assert_eq!(counted as usize, 1745 - 5, "1745 frames, 5 passes missed (one at each clear)");
}

/// Diagnostic for M1-07: for each ranking run of `attract3.mame.trc` the number of writes of the draw
/// (`Game::ranking_ops`) the original has done when the first frame after the call ends (the record S + 1: the
/// smallest and the largest count that give the recording's tilemaps and scroll2 Y). These are the `CUTS` of
/// `ranking.rs`. Run with `cargo test -p dino-game ranking_cut_report -- --ignored --nocapture`.
#[test]
#[ignore]
fn ranking_cut_report() {
    use super::ranking::Op;
    let Some(a) = assets() else { return };
    let trace = "../../../re/out/traces/attract3.mame.trc";
    // the frames the ranking routine is called (the stop code 0xFF00 and the palette set 4)
    let starts = [2320, 6449, 8886, 11241, 15423, 19554, 21991, 24346, 28528];
    let mut found = Vec::new();
    compare_power_on_with(a, trace, 28530, |_| [0; 3], |_, _, _| {}, |k, g, _ram, gfx| {
        if !starts.contains(&(k - 1)) {
            return;
        }
        let ops = g.ranking_ops();
        let mut cells = [g.vram.scroll1.clone(), g.vram.scroll2.clone(), g.vram.scroll3.clone()];
        let mut cam_y = g.cam.s2.cam_y;
        let base = [SCROLL1, SCROLL2, SCROLL3];
        let want = |layer: usize, i: usize| cell_at(gfx, base[layer], i);
        let mut bad: i64 = (0..3).map(|l| (0..0x1000).filter(|&i| (cells[l][i].code, cells[l][i].attr) != want(l, i)).count() as i64).sum();
        let cam_want = ram16(_ram, 0xFF8748);
        let mut zero: Vec<usize> = Vec::new(); // in half writes
        let mut best = (bad, 0usize);
        if bad == 0 && cam_y == cam_want {
            zero.push(0);
        }
        for (n, op) in ops.iter().enumerate() {
            match *op {
                Op::Cell(l, i, code, attr) => {
                    let (l, i) = (l as usize - 1, i as usize);
                    // the code word first: the frame can end between it and the attribute word
                    let was = (cells[l][i].code, cells[l][i].attr) != want(l, i);
                    cells[l][i].code = code;
                    let half = (cells[l][i].code, cells[l][i].attr) != want(l, i);
                    bad += half as i64 - was as i64;
                    if bad == 0 && cam_y == cam_want && (code, attr) != (cells[l][i].code, cells[l][i].attr) {
                        zero.push(n * 2 + 1); // n writes and the code of the next
                    }
                    cells[l][i].attr = attr;
                    let is = (code, attr) != want(l, i);
                    bad += is as i64 - half as i64;
                }
                Op::CamY(y) => cam_y = y,
            }
            if bad == 0 && cam_y == cam_want {
                zero.push(n * 2 + 2);
            }
            if bad < best.0 {
                best = (bad, n + 1);
            }
        }
        if zero.is_empty() {
            let l_bad: Vec<(usize, usize, (u16, u16), (u16, u16))> = (0..3)
                .flat_map(|l| (0..0x1000).map(move |i| (l, i)))
                .filter(|&(l, i)| (cells[l][i].code, cells[l][i].attr) != want(l, i))
                .map(|(l, i)| (l, i, (cells[l][i].code, cells[l][i].attr), want(l, i)))
                .take(8)
                .collect();
            println!("  record {k}: no count fits, fewest wrong {} at {}; after all writes (layer, cell, port, MAME): {l_bad:?} cam y {cam_y} (MAME {cam_want})", best.0, best.1);
        }
        found.push((k - 1, ops.len(), zero.first().copied(), zero.last().copied(), zero.len()));
    });
    for f in &found {
        println!("ranking called at {}: {} writes, the first frame has done {:?} to {:?} of them ({} counts fit)", f.0, f.1, f.2, f.3, f.4);
    }
}

/// Diagnostic report of the power-on comparison: the records that differ, by kind. Run with
/// `cargo test -p dino-game power_on_report -- --ignored --nocapture`; `DINO_LAST` (default 940) is
/// the last record, `DINO_FROM` the first one whose differences are listed in full.
#[test]
#[ignore]
fn power_on_report() {
    let Some(a) = assets() else { return };
    let last = std::env::var("DINO_LAST").ok().and_then(|v| v.parse().ok()).unwrap_or(940);
    let trace = std::env::var("DINO_TRACE").unwrap_or_else(|_| "../../../re/out/traces/attract3.mame.trc".into());
    let (_, mut rows) = compare_power_on(a, &trace, last, |_| [0; 3], |_, _| {});
    drop_demo_blips(&mut rows);
    println!("s1      {}", ranges(&rows, |d| d.s1 > 0));
    println!("s2      {}", ranges(&rows, |d| d.s2 > 0));
    println!("s3      {}", ranges(&rows, |d| d.s3 > 0));
    println!("palette {}", ranges(&rows, |d| d.pal > 0));
    println!("sprites {}", ranges(&rows, |d| d.sprites.is_some()));
    println!("layer   {}", ranges(&rows, |d| d.layer.is_some()));
    println!("counter {}", ranges(&rows, |d| d.counter.is_some()));
    println!("sounds  {}", ranges(&rows, |d| d.sounds.is_some()));
    println!("state   {}", ranges(&rows, |d| d.state.is_some()));
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

/// The attract loop from power on without input, 28960 records of `attract3.mame.trc` (made like the
/// others, `re/harness/record.sh re/harness/scripts/attract.txt attract3 29500` with the video
/// clock; see `re/harness/README.md`). A3: the attract state byte G.-$7030 changes at the video frames
/// 2744, 6874, 9311, 13411, 15848, 19979, 22416, 26516 and 28953 (records 2 less), equal in
/// every record (the demo's own clear of the byte for 3 frames aside, see `drop_demo_blips`). The
/// title is real in every pass: from the frame of its setup (layer control 0x1396) to the one before
/// the next screen starts, tilemaps, palettes, sprite list, layer control, title counter and sound
/// codes equal the recording. The other screens are stubs of the measured length (`attract.rs`) and
/// are not compared.
#[test]
fn attract_loop_state_and_titles_match_mame() {
    let Some(a) = assets() else { return };
    let trace = "../../../re/out/traces/attract3.mame.trc";
    if !Path::new(trace).exists() {
        eprintln!("mame_check: {trace} is missing, skipped");
        return;
    }
    let (_, mut rows) = compare_power_on(a, trace, 28960, |_| [0; 3], |_, _| {});
    drop_demo_blips(&mut rows);
    let state = ranges(&rows, |d| d.state.is_some());
    assert!(state.is_empty(), "the attract state byte differs at records {state}");
    // (setup frame, last frame) of the ten titles: the layer control 0x1396 to the frame before the title returns
    let titles = [
        (319, 918),
        (2744, 3342),
        (6876, 7476),
        (9313, 9913),
        (13413, 14013),
        (15848, 16447),
        (19981, 20581),
        (22418, 23018),
        (26518, 27118),
        (28953, 28960),
    ];
    for (from, to) in titles {
        let bad: Vec<(usize, &Differences)> = rows.iter().enumerate().take(to + 1).skip(from).filter(|(_, d)| !d.none()).collect();
        assert!(bad.is_empty(), "title {from}..={to}: {} records differ, first {:?}", bad.len(), &bad[..bad.len().min(3)]);
    }
}

/// A hash of what the player sees and hears of the game after a tick, to tell two runs apart.
fn fingerprint(g: &Game) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for layer in [&g.vram.scroll1, &g.vram.scroll2, &g.vram.scroll3] {
        layer.iter().for_each(|c| (c.code, c.attr).hash(&mut h));
    }
    g.pal.words.hash(&mut h);
    g.sprites.entries.iter().for_each(|e| (e.x, e.y, e.code, e.attr).hash(&mut h));
    (g.w.sounds.clone(), g.w.layer_ctrl, g.flow.in_game, g.flow.front.attract_state, g.flow.mode).hash(&mut h);
    h.finish()
}

/// The recordings of the flow scenarios (`re/harness/scripts/flow_system.txt`, recorded in one batch
/// with `re/harness/record.sh --batch`): the same inputs reach a task one record earlier than in the
/// plain recordings, so the port gets the script line `f` at tick `f - 1`.
fn flow_trace(name: &str) -> Option<String> {
    let p = format!("../../../re/out/traces/flow_{name}.mame.trc");
    if Path::new(&p).exists() {
        return Some(p);
    }
    eprintln!("mame_check: {p} is missing, skipped");
    None
}

/// A10, a coin at video frame 450 to 457 and a start at 600 to 605 (`flow_c-title`, script lines
/// `450-457:C` and `600-605:S`): the coin is accepted when the signal has been high for two frames and low
/// for two (record 458), the credit, the sounds 0xFF00 and 0x30, the press start screen (G.$46D =
/// 0xFF) and its picture (scroll2, scroll3, palettes, sprite list, layer control: scroll1 holds the
/// prompts of the system task, M2-02) equal the recording from the frame its task first runs; the
/// start (record 599) takes the credit, sets G.$4CC and G.$4CD, queues 0x31, fades out for 16 frames
/// (the fade mask clears at 614), the game task is created at 617 (G.$4D5 = 1, G.$46D = 0) and its mode
/// becomes 4 (the select) at 621. Every watched variable of `system_fields` equals up to then.
#[test]
fn coin_and_start_match_mame() {
    let Some(a) = assets() else { return };
    let Some(trace) = flow_trace("c-title") else { return };
    let (_, rows) = compare_power_on(a, &trace, 621, |_| [0; 3], |k, g| {
        g.w.coins = (449..=456).contains(&k) as u8;
        g.w.system = 8 | (599..=604).contains(&k) as u8;
    });
    let mut bad = Vec::new();
    for (k, d) in rows.iter().enumerate().skip(25) {
        if !d.sys.is_empty() || d.sounds.is_some() || d.state.is_some() || d.counter.is_some() {
            bad.push(format!("record {k}: {:?} sounds {:?} state {:?} counter {:?}", d.sys, d.sounds, d.state, d.counter));
        }
        let picture = if k < 458 {
            d.s1 + d.s2 + d.s3 + d.pal > 0 || d.sprites.is_some() || d.layer.is_some()
        } else if k >= 461 && k < 599 {
            d.s2 + d.s3 + d.pal > 0 || d.sprites.is_some() || d.layer.is_some()
        } else {
            false
        };
        if picture {
            bad.push(format!("record {k}: picture {d:?}"));
        }
    }
    assert!(bad.is_empty(), "{} problems, first: {:?}", bad.len(), &bad[..bad.len().min(4)]);
}

/// A11, a start without a credit does nothing: `flow_s-nocredit` (start at 450 to 455 and 1000 to 1005)
/// and `flow_s-nocredit-demo` (4000 to 4005) are identical to the plain attract recording in the
/// pictures, the layer control, the state byte and the sound ring. The port gives the same output
/// with and without the presses in every tick, and equals the recording where its screens are real
/// (the title, records 25 to 918).
#[test]
fn start_without_credit_does_nothing() {
    let Some(a) = assets() else { return };
    let Some(trace) = flow_trace("s-nocredit") else { return };
    let press = |k: i32| (449..=454).contains(&k) || (999..=1004).contains(&k) || (3999..=4004).contains(&k);
    let (mut with, mut without) = (Game::new(a), Game::new(a));
    with.flow_boot();
    without.flow_boot();
    for k in 0..4100 {
        with.w.system = 8 | press(k) as u8;
        without.w.system = 8;
        with.tick();
        without.tick();
        assert_eq!(fingerprint(&with), fingerprint(&without), "a start without credit changed something at tick {k}");
        assert_eq!(with.flow.in_game, 0);
    }
    let (_, rows) = compare_power_on(a, &trace, 1400, |_| [0; 3], |k, g| g.w.system = 8 | press(k) as u8);
    let bad: Vec<(usize, &Differences)> = rows.iter().enumerate().skip(25).take(894).filter(|(_, d)| !d.none() || !d.core_sys().is_empty()).collect();
    assert!(bad.is_empty(), "differs at {:?}", &bad[..bad.len().min(3)]);
    let state = ranges(&rows[25..], |d| d.state.is_some() || !d.core_sys().is_empty());
    assert!(state.is_empty(), "the state byte or a system variable differs at records {state}");
}

/// A12, free play (`flow_freeplay`: G.$46B poked to 1 at power frame 100, start at 500 to 505): the
/// title shows FREE PLAY (the blink task reads G.$46B), there is no press start screen, a start takes
/// no credit, queues 0x31, fades out, creates the game task and reaches the select (mode 4) at
/// record 521, 22 frames after the start at 499. The title records 25 to 498 equal the recording
/// (FREE PLAY text included) and every watched variable equals up to the select.
#[test]
fn free_play_start_matches_mame() {
    let Some(a) = assets() else { return };
    let Some(trace) = flow_trace("freeplay") else { return };
    let (_, rows) = compare_power_on(a, &trace, 521, |_| [0; 3], |k, g| {
        if k == 99 {
            g.flow.front.sys.free_play = true; // the poke
        }
        g.w.system = 8 | (499..=504).contains(&k) as u8;
    });
    let bad: Vec<(usize, &Differences)> = rows.iter().enumerate().skip(25).filter(|(k, d)| (*k < 499 && !d.none()) || !d.sys.is_empty() || d.sounds.is_some()).collect();
    assert!(bad.is_empty(), "{} records differ, first {:?}", bad.len(), &bad[..bad.len().min(3)]);
}

/// F35, the wait counter G.$4F6 (0x1827A): it counts every frame while nobody plays and the common pool
/// has credits or free play is on, and a lost pass of the system task (the attract task is busy with
/// a clear) skips a count. `m104_free` is the attract loop with G.$46B poked to 1 at power frame 100
/// (`DINO_POKES`) for 3100 records; the counter equals it in every record up to 3000 (it has lost 12 counts
/// by then: at records 316 to 319, 920, 1245, 1606, 1967, 2319 to 2321 and 2743). With the factory
/// settings (separate chutes) a coin gives a credit to the player and not to the common pool, so the
/// counter stays 0: `m104_coin` (a coin at 448 to 455, 3100 records) shows the press start screen the
/// whole time with G.$4F6 = 0 and the same picture as the port.
#[test]
fn wait_counter_matches_mame() {
    let Some(a) = assets() else { return };
    let free = "../../../re/out/traces/m104_free.mame.trc";
    let coin = "../../../re/out/traces/m104_coin.mame.trc";
    if !Path::new(free).exists() || !Path::new(coin).exists() {
        eprintln!("mame_check: {free} or {coin} is missing (see the doc of this test), skipped");
        return;
    }
    let (g, rows) = compare_power_on(a, free, 3000, |_| [0; 3], |k, g| {
        if k == 99 {
            g.flow.front.sys.free_play = true;
        }
    });
    let bad: Vec<(usize, Vec<&(&str, u16, u16)>)> = rows.iter().enumerate().skip(25).filter(|(_, d)| !d.core_sys().is_empty()).map(|(k, d)| (k, d.core_sys())).collect();
    assert!(bad.is_empty(), "{} records differ, first {:?}", bad.len(), &bad[..bad.len().min(3)]);
    assert_eq!(g.rank.w4f6, 2890, "the counter after 3000 records (2902 frames of counting, 12 lost)");
    // the title (FREE PLAY text included) and the state byte equal the recording through the title
    let title: Vec<&Differences> = rows.iter().skip(25).take(894).filter(|d| !d.none()).collect();
    assert!(title.is_empty(), "the free play title differs: {:?}", title.first());

    let (g, rows) = compare_power_on(a, coin, 3000, |_| [0; 3], |k, g| g.w.coins = (448..=455).contains(&k) as u8);
    assert_eq!(g.rank.w4f6, 0, "factory settings: the counter does not run with a credit of a player");
    assert_eq!(g.flow.front.sys.slots[0].credits, 1);
    let bad: Vec<(usize, &Differences)> =
        rows.iter().enumerate().skip(25).filter(|(k, d)| !d.core_sys().is_empty() || (*k >= 461 && (d.s2 + d.s3 + d.pal > 0 || d.sprites.is_some() || d.layer.is_some()))).collect();
    assert!(bad.is_empty(), "{} records differ, first {:?}", bad.len(), &bad[..bad.len().min(3)]);
}

/// F25, the attract state is kept across games (D4d s3.1, probe `d4d-b16/go-attract`, no recording
/// here): a game over in state 0 (the profile cards are up) restarts the attract task, which shows the
/// ranking once (0x97C94), then runs the routine of the same state 0 again (title, profile cards,
/// ranking) and only then steps to state 2. Port only.
#[test]
fn game_over_resumes_the_attract_state() {
    let Some(a) = assets() else { return };
    let mut g = Game::new(a);
    g.flow_boot();
    for _ in 0..1000 {
        g.tick(); // in the profile cards of state 0
    }
    assert_eq!((g.flow.front.attract_state, g.flow.front.stub), (0, super::attract::Stub::Profiles));
    g.flow.in_game = 1; // a game that ends at once
    g.flow_game_over();
    let mut events: Vec<String> = Vec::new();
    let mut last_stub = super::attract::Stub::None;
    let mut last_state = 0;
    for _ in 0..5000 {
        g.tick();
        if g.flow.front.stub != last_stub {
            last_stub = g.flow.front.stub;
            if last_stub != super::attract::Stub::None {
                events.push(format!("{last_stub:?}"));
            }
        }
        if g.flow.front.title == 1 && events.last().map(String::as_str) != Some("Title") {
            events.push("Title".into());
        }
        if g.flow.front.attract_state != last_state {
            last_state = g.flow.front.attract_state;
            events.push(format!("state {last_state}"));
        }
        if last_state == 2 {
            break;
        }
    }
    assert_eq!(events, ["Ranking", "Title", "Profiles", "Ranking", "state 2"]);
    assert_eq!(g.flow.front.game_over_flag, 0, "the flag was cleared after the ranking");
}
