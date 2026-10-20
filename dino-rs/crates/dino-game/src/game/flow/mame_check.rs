//! Checks of the front end services against the original (MAME), on real ROM data.
//!
//! The printers and loaders are run with the real strings and pages and the result is compared,
//! cell by cell, with the tilemaps of the MAME power-on recording `re/out/traces/attract.mame.trc`
//! (made as described in `re/harness/README.md`; frame k of that file is video frame k + 2). A cell
//! is a code word and an attribute word. The tests need the ROM (`dino.zip` in the repo root) and
//! the recording; they pass without doing anything when either is missing (both are git-ignored).
//!
//! The record numbers are the frames where the original draws each screen (D4d section 3).
//!
//! The acceptance of the attract loop (M1-15) is `attract_loop_two_loops_match_mame`: two whole loops from power on against
//! `attract3.mame.trc`, record by record, with no allowed difference outside the demos. Its diagnostics (ignored tests) are
//! `loop_report` (the records outside the demos that differ), `press_report` (the records of a flow recording with a coin or
//! a start, cell ranges per layer) and `demo_report` (the demos). The field by field comparison of the same loop with
//! `re/harness/diff.py` has its approved differences, with causes, in `re/harness/allow/m1.txt`.

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
    /// The sound requests of the frame with their d1 and d2 (MAME, port), when the codes agree and an argument differs (M4-15).
    /// Not part of `none`: the tests that check the arguments read it with `sound_arg_diffs`.
    sound_args: Option<(Vec<(u16, u32, u32)>, Vec<(u16, u32, u32)>)>,
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
        Differences { sys: Vec::new(), cams: None, sound_args: None, ..self.clone() } == Differences::default()
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
        ("coins P2 (G.$48B)", b(0xFF848B), s.slots[1].coins as u16),
        ("credits P2 (G.$48D)", b(0xFF848D), s.slots[1].credits as u16),
        ("coins P3 (G.$49B)", b(0xFF849B), s.slots[2].coins as u16),
        ("credits P3 (G.$49D)", b(0xFF849D), s.slots[2].credits as u16),
        ("coins before P1 (G.$47E)", b(0xFF847E), s.slots[0].prev as u16),
        ("credits pool (G.$46A)", b(0xFF846A), s.credits as u16),
        ("coins pool (G.$475)", b(0xFF8475), s.common_coins as u16),
        ("coins pool before (G.$476)", b(0xFF8476), s.common_prev as u16),
        ("coin counter pulses (G.$46E)", b(0xFF846E), s.pulse as u16),
        ("coin counter timer (G.$46F)", b(0xFF846F), s.pulse_timer as u16),
        ("coin outputs (G.$21)", b(0xFF8021), s.outputs as u16),
        ("redraw credit text (G.$473)", b(0xFF8473), s.redraw as u16),
        // the join task of a game (0x6D4A, not ported before M2-04) counts it, so it is compared outside a game only
        ("prompt hold (G.$477)", if b(0xFF84CC) == 0 { b(0xFF8477) } else { s.hold as u16 }, s.hold as u16),
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
    // a RAM clear (the demo setup 0x99F10) resets the index and zeroes the ring: the empty entries it passes are not requests
    v.retain(|&c| c != 0);
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
    mut after: impl FnMut(i32, &mut Game, &[u8], &[u8]),
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
        after(k, &mut g, ram, gfx);
        let (msn, mut psn) = (mame_sounds(&prev_ram, ram), g.w.sounds.clone());
        psn.retain(|&c| c != 0); // the code 0 (the select music) is not told from an empty entry of the ring
        if msn != psn {
            d.sounds = Some((msn, psn));
        }
        let ma = mame_sound_args(&prev_ram, ram);
        let pa: Vec<(u16, u32, u32)> = g.w.snd_reqs.iter().filter(|r| r.code != 0).map(|r| (r.code, r.d1, r.d2)).collect();
        if ma != pa {
            d.sound_args = Some((ma, pa));
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
/// and the random number state G.$14 (one per hero a frame, the clouds) equal the recording. The demos (4 runs) are real since
/// M1-12 and draw their numbers, but the levels they play are not complete: the objects of the classes the port does not run draw
/// numbers the port does not (the generator first differs 12 to 239 frames into each demo, see `demo_report`). So the state G.$14 is
/// still set to the recording's value in the frame the demo returns, until the levels are real (M10-03 to M10-05 remove this). The
/// story slides, the profile cards and the opening movie draw their own (the slides none, see
/// `story_slides_run_like_the_original`), so no ranking needs the state set in its own frame.
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
            // the demos (M1-12) play their random numbers, but not those of the objects of classes the port does not run
            // yet (M10-03 to M10-05): the state they leave is set from the recording in the frame the demo returns (the frame
            // the story slides start or the clear before the ranking of state 6). The slides, the profile cards and the movie
            // play theirs.
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
        if k == 608 {
            // the system task ends the attract task and calls the screen inside frame 607 (the staged call of M1-15 does its
            // second step at the start of the next frame's system task, so a call before tick 608 stands for one in frame 607)
            g.flow.front.task.kill();
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
/// 0xFF) and its whole picture (scroll1 with the credit lines and the prompts of the system task from record 461, scroll2,
/// scroll3, palettes, sprite list, layer control) equal the recording in every record from the call (458 to 460 are the three
/// frames of the call 0x983EE) to the start; the
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
        // 458 to 460 are the three frames of the call 0x983EE (the clear, the palette load, the brightness loop cut by the frame
        // boundary): equal in everything; from 461 scroll1 holds the credit text of the system task (M2-02): equal too
        let picture = if k < 599 { d.s1 + d.s2 + d.s3 + d.pal > 0 || d.sprites.is_some() || d.layer.is_some() } else { false };
        if picture {
            bad.push(format!("record {k}: picture {d:?}"));
        }
    }
    assert!(bad.is_empty(), "{} problems, first: {:?}", bad.len(), &bad[..bad.len().min(4)]);
}

/// The coin and start inputs of a flow recording: `ranges` are (player 1 to 3, first, last) in script lines, which reach the
/// port one tick earlier (`flow_trace`); the bits of the players whose range holds the tick.
fn script_bits(ranges: &[(u8, i32, i32)], tick: i32) -> u8 {
    ranges.iter().filter(|(_, a, b)| (a - 1..=b - 1).contains(&tick)).fold(0, |m, (p, _, _)| m | 1 << (p - 1))
}

/// The records of a flow with coins and starts that differ from the recording: every variable of the system task, the sound codes,
/// the state byte and the title counter in the records `25..=vars_to`, the whole picture (the three layers, the palettes, the
/// sprite list, the layer control) in the records `25..picture_to` except `skip_palette` (record numbers).
fn coin_flow_problems(
    a: &'static AssetPack,
    name: &str,
    coins: &[(u8, i32, i32)],
    starts: &[(u8, i32, i32)],
    vars_to: i32,
    picture_to: usize,
    skip_palette: &[usize],
) -> Option<Vec<String>> {
    let trace = flow_trace(name)?;
    let (_, rows) = compare_power_on(a, &trace, vars_to, |_| [0; 3], |k, g| {
        g.w.coins = script_bits(coins, k);
        g.w.system = 8 | script_bits(starts, k);
    });
    let mut bad = Vec::new();
    for (k, d) in rows.iter().enumerate().skip(25) {
        if !d.sys.is_empty() || d.sounds.is_some() || d.state.is_some() || d.counter.is_some() {
            bad.push(format!("record {k}: {:?} sounds {:?} state {:?} counter {:?}", d.sys, d.sounds, d.state, d.counter));
        }
        let pal = if skip_palette.contains(&k) { 0 } else { d.pal };
        if k < picture_to && (d.s1 + d.s2 + d.s3 + pal > 0 || d.sprites.is_some() || d.layer.is_some()) {
            bad.push(format!("record {k}: picture {d:?}"));
        }
    }
    Some(bad)
}

/// M2-02, the credit text and the prompts of the press start screen for several players (the factory settings: three chutes, one
/// coin one credit): `flow_c-two` (a coin of player 1 at 450 to 457 and of player 2 at 500 to 507: each credit line and each prompt
/// changes in the column of its player), `flow_c-three-start` (a third coin at 550 to 557, then a start of player 1 at 600:
/// the credits of players 2 and 3 stay in their columns until the start) and `flow_c-title-coinhold` (the coin held for 31 frames:
/// it counts when it is let go, record 483). Every record equals the recording in the three layers, the palettes, the sprite
/// list and all the variables of the system task, up to the start (the select that follows is a stub until M2-03).
/// Known difference: the first call frame of `c-title-coinhold` (record 483, a coin 25 frames later in the title than A10): the
/// brightness loop of the call is cut by the video frame boundary three words earlier or later than in the measured cases
/// (`PRESS_CALL_CUT`, 154 words, for a coin at 458), 3 palette words of one record.
#[test]
fn credits_of_all_players_match_mame() {
    let Some(a) = assets() else { return };
    let c = |p, a, b| (p, a, b);
    if let Some(bad) = coin_flow_problems(a, "c-two", &[c(1, 450, 457), c(2, 500, 507)], &[], 900, 900, &[]) {
        assert!(bad.is_empty(), "c-two: {} problems, first: {:?}", bad.len(), &bad[..bad.len().min(3)]);
    }
    if let Some(bad) = coin_flow_problems(a, "c-three-start", &[c(1, 450, 457), c(2, 500, 507), c(3, 550, 557)], &[c(1, 600, 605)], 621, 599, &[]) {
        assert!(bad.is_empty(), "c-three-start: {} problems, first: {:?}", bad.len(), &bad[..bad.len().min(3)]);
    }
    if let Some(bad) = coin_flow_problems(a, "c-title-coinhold", &[c(1, 450, 480)], &[], 1400, 1400, &[483]) {
        assert!(bad.is_empty(), "c-title-coinhold: {} problems, first: {:?}", bad.len(), &bad[..bad.len().min(3)]);
    }
}

/// The segments of the scenario file `re/harness/scripts/flow_system_modes.txt` (M2-02) with their pokes
/// (`re/harness/pokes/flow_system_modes.txt`): (name, frames, coin ranges, start ranges, pokes (address, size, value)). The ranges are
/// (player 1 to 3, first, last) of the script lines.
type ModeSegment = (String, i32, Vec<(u8, i32, i32)>, Vec<(u8, i32, i32)>, Vec<(u32, u8, u32)>);

fn mode_segments() -> Option<Vec<ModeSegment>> {
    let script = std::fs::read_to_string("../../../re/harness/scripts/flow_system_modes.txt").ok()?;
    let pokes = std::fs::read_to_string("../../../re/harness/pokes/flow_system_modes.txt").ok()?;
    let mut out: Vec<ModeSegment> = Vec::new();
    for line in script.lines().map(|l| l.split('#').next().unwrap().trim()).filter(|l| !l.is_empty()) {
        if let Some(name) = line.strip_prefix("segment ") {
            out.push((name.trim().to_string(), 0, vec![], vec![], vec![]));
        } else if let Some(n) = line.strip_prefix("frames ") {
            out.last_mut()?.1 = n.trim().parse().ok()?;
        } else {
            let (range, keys) = line.split_once(':')?;
            let (a, b) = range.split_once('-')?;
            let (a, b): (i32, i32) = (a.parse().ok()?, b.parse().ok()?);
            let (player, key) = match keys.len() {
                1 => (1u8, keys),
                2 => (keys[..1].parse().ok()?, &keys[1..]),
                _ => return None,
            };
            let seg = out.last_mut()?;
            if key == "C" { seg.2.push((player, a, b)) } else { seg.3.push((player, a, b)) }
        }
    }
    let mut current = String::new();
    for line in pokes.lines().map(|l| l.split('#').next().unwrap().trim()).filter(|l| !l.is_empty()) {
        if let Some(name) = line.strip_prefix("segment ") {
            current = name.trim().to_string();
        } else {
            let f: Vec<&str> = line.split_whitespace().collect();
            let poke = (u32::from_str_radix(f[1], 16).ok()?, f[2].parse().ok()?, f[3].parse().ok()?);
            out.iter_mut().find(|s| s.0 == current)?.4.push(poke);
        }
    }
    Some(out)
}

/// The writes of a poke into the system task variables (G.$46B free play, G.$46C 2 coin start, G.$470 / G.$471 the ratio as a word,
/// G.$472 one chute, G.$474 two players on one chute, G.$4CF the 2 player layout).
fn apply_mode_poke(g: &mut Game, (addr, _size, value): (u32, u8, u32)) {
    let s = &mut g.flow.front.sys;
    match addr {
        0xFF846B => s.free_play = value != 0,
        0xFF846C => s.two_coin_start = value != 0,
        0xFF8470 => (s.coins_per_credit, s.credits_per_coin) = ((value >> 8) as u8, value as u8),
        0xFF8472 => s.common = value != 0,
        0xFF8474 => s.two_player = value != 0,
        0xFF84CF => g.flow.b4cf = value as u8,
        _ => panic!("no field for the poke at {addr:X}"),
    }
}

/// Runs a segment of `flow_system_modes` against its recording: the records `25..` equal in the whole picture and in all the variables of
/// the system task up to the start (the picture) and the select that follows it (the variables), or to the end without a start.
/// Returns the problems and the number of records compared.
fn mode_problems(a: &'static AssetPack, seg: &ModeSegment, detail: bool) -> Option<(Vec<String>, usize)> {
    let (name, frames, coins, starts, pokes) = seg;
    let trace = flow_trace(name)?;
    let first_start = starts.iter().map(|s| s.1).min();
    // a start at script line f reaches the game at tick f - 1; the select (mode 4) comes 22 records later
    let (vars_to, picture_to) = match first_start {
        Some(f) => (f - 1 + 22, (f - 1) as usize),
        None => (frames - 3, (frames - 3) as usize),
    };
    let (_, rows) = compare_power_on_with(
        a,
        &trace,
        vars_to,
        |_| [0; 3],
        |k, g, _| {
            if k == 99 {
                pokes.iter().for_each(|&p| apply_mode_poke(g, p)); // the poke at power frame 100
            }
            g.w.coins = script_bits(coins, k);
            g.w.system = 8 | script_bits(starts, k);
        },
        |k, g, _, gfx| {
            if detail && ((k as usize) >= 440 || k == 319) && (k as usize) < picture_to {
                for layer in 1..=3u8 {
                    let full = diff(g, gfx, layer);
                    let d: Vec<usize> = full.iter().map(|x| x.0).collect();
                    if !d.is_empty() {
                        println!("record {k}: scroll{layer} {} cells: {}", d.len(), index_ranges(&d));
                        if k == 319 {
                            println!("    (cell, port, MAME) {:04X?}", &full[..full.len().min(8)]);
                        }
                    }
                }
            }
        },
    );
    let mut bad = Vec::new();
    for (k, d) in rows.iter().enumerate().skip(25) {
        // The three known differences (see `coin_and_chute_modes_match_mame`), each a few cells, words or sprites of one record
        let mut d = d.clone();
        match k {
            319 if d.s2 <= 6 => d.s2 = 0,
            460 if d.pal <= 44 => d.pal = 0,
            462 if d.sprites == Some((19, 0)) => d.sprites = None,
            _ => {}
        }
        if !d.sys.is_empty() || d.sounds.is_some() || d.state.is_some() || d.counter.is_some() {
            bad.push(format!("record {k}: {:?} sounds {:?} state {:?} counter {:?}", d.sys, d.sounds, d.state, d.counter));
        }
        if k < picture_to && (d.s1 + d.s2 + d.s3 + d.pal > 0 || d.sprites.is_some() || d.layer.is_some()) {
            bad.push(format!("record {k}: picture {d:?}"));
        }
    }
    Some((bad, rows.len()))
}

/// M2-02, the coin modes, the chute modes, the "2 coins start, 1 coin continue" mode and free play coins, from power on, against
/// the recordings of `flow_system_modes` (the settings are written into the system task variables at power frame 100, the same
/// writes in the port; MAME has no switch for them): 1 coin 2 credits, 1 coin 6 credits (the cap of 9), 2 and 3 coins per credit
/// (the coin counts `CREDIT 0 (1/2)` and the "2 COIN 1 CREDIT" line), the common pool with its three prompts (one, two, three
/// players), two players on one chute (the third credit still says two players, a start of player 2 takes 2 credits), the 2 coin
/// start text of the pool up to 7 credits (PUSH START BUTTON, how many players, the lines about additional players), a coin in free
/// play (the screen does not change), and 21 coins (the cap of 9 with a pending coin shown as 0). Every record equals the
/// recording in the three layers, the palettes, the sprite list and every variable of the system task (all the credits, coins
/// and the coin lockout and counter outputs G.$21 included) up to the start, except three differences of a few cells, words or
/// sprites in one record each that come from the CPU time the settings change, not from the game rules: a pass of the system
/// task is shorter with one chute (the pool conversion and the lockout of one chute instead of three) and a little different
/// with other coin ratios, so the video frame boundary falls elsewhere in the code that runs after the system task or at its
/// end: record 319 (the title's block drawing of its first frame has 6 more cells in MAME with one chute), record 460 (the cut of
/// the brightness loop in the first frame of the press start call: `PRESS_CALL_CUT` is 154 words for the factory settings, MAME
/// differs by 38 / 44 words with one chute and 4 with a coin ratio) and record 462 (one chute: the press start task's first object
/// pass comes a frame earlier). The factory settings equal the recordings exactly (`coin_and_start_match_mame`).
#[test]
fn coin_and_chute_modes_match_mame() {
    let Some(a) = assets() else { return };
    let Some(segments) = mode_segments() else {
        eprintln!("mame_check: the scenario files of flow_system_modes are missing, skipped");
        return;
    };
    let detail = std::env::var("DINO_DETAIL").is_ok();
    let only = std::env::var("DINO_SEGMENT").ok();
    for seg in segments.iter().filter(|s| only.as_ref().is_none_or(|o| *o == s.0)) {
        if let Some((bad, n)) = mode_problems(a, seg, detail) {
            assert!(n > 600);
            if detail {
                println!("== {}: {} problems", seg.0, bad.len());
                bad.iter().take(12).for_each(|b| println!("   {}", &b[..b.len().min(230)]));
                continue;
            }
            assert!(bad.is_empty(), "{}: {} problems, first: {:?}", seg.0, bad.len(), &bad[..bad.len().min(3)]);
        }
    }
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
        rows.iter().enumerate().skip(25).filter(|(k, d)| !d.core_sys().is_empty() || (*k >= 461 && (d.s1 + d.s2 + d.s3 + d.pal > 0 || d.sprites.is_some() || d.layer.is_some()))).collect();
    assert!(bad.is_empty(), "{} records differ, first {:?}", bad.len(), &bad[..bad.len().min(3)]);
}

/// M2-23, the wait counter G.$4F6 and the starting rank (0x17F8A, D4b s2): `m223_free_wait` is free play (G.$46B poked to 1 at power frame 100) with a start
/// pressed at record 2600, 2490 counts into the wait (the second band of table 0x180B4, 720 to 3599: a rank two lower). The system task clears the counter
/// (0x1827A with G.$4CC set) in the pass that creates the game task, 18 records before the game task runs 0x17F8A, so the band is always the first one: MAME
/// shows G.$4F6 = 0 at record 2618 and the rank 13 (a roll of the first band) at 2620. The port equals it in every record from the start to the select: the
/// counter, the rank, the rank speed, the random number and the mode (the three layers and the sprites are those of the press start in the profile cards).
/// Recording: `DINO_CLOCK=video DINO_BOOT=re/harness/scripts/attract.txt DINO_SYNC=vblank:1 DINO_POKES=re/harness/pokes/freeplay_power.txt
/// re/harness/record.sh re/harness/scripts/m223_free_wait.txt m223_free_wait 3000`.
#[test]
fn a_long_wait_does_not_change_the_starting_rank() {
    let Some(a) = assets() else { return };
    let trace = "../../../re/out/traces/m223_free_wait.mame.trc";
    if !Path::new(trace).exists() {
        eprintln!("mame_check: {trace} is missing (see the doc of this test), skipped");
        return;
    }
    let mut bad: Vec<(i32, &'static str, u16, u16)> = Vec::new();
    let mut seen = Vec::new();
    let (g, _) = compare_power_on_with(
        a,
        trace,
        2640,
        |_| [0; 3],
        |k, g, _| {
            if k == 99 {
                g.flow.front.sys.free_play = true; // the poke
            }
            g.w.system = 8 | (2600..=2605).contains(&k) as u8;
        },
        |k, g, ram, _| {
            let w = |a: u32| ram16(ram, a);
            for (name, m, p) in [
                ("G.$4F6 wait", w(0xFF84F6), g.rank.w4f6),
                ("G.$5F0 rank", w(0xFF85F0), g.w.rank),
                ("G.$5F6 speed", w(0xFF85F6), g.rank.speed),
                ("G.$14 rng", w(0xFF8014), g.w.rng),
                ("G.$5EC aggression", w(0xFF85EC), g.rank.aggression),
            ] {
                if k >= 2590 && m != p {
                    bad.push((k, name, m, p));
                }
            }
            if k == 2599 {
                seen.push(g.rank.w4f6);
            }
        },
    );
    assert_eq!(seen, [2490], "the counter was 2490 when the start was pressed");
    assert!(bad.is_empty(), "{} differences, first {:?}", bad.len(), &bad[..bad.len().min(4)]);
    assert_eq!((g.w.rank, g.rank.speed, g.rank.w4f6), (13, 2, 0));
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

// ---------------------------------------------------------------------------------------------------------------------
// The demo (M1-12, M1-13)
// ---------------------------------------------------------------------------------------------------------------------

/// The game variables of a level frame that the demo test compares: (name, MAME, port).
fn demo_globals(g: &Game, ram: &[u8]) -> Vec<(&'static str, u16, u16)> {
    let b = |a: u32| ram[(a - 0xFF0000) as usize] as u16;
    [
        ("G.$14 random", ram16(ram, 0xFF8014), g.w.rng),
        ("G.$0 frame", ram16(ram, 0xFF8000), g.w.frame),
        ("G.$757C left", ram16(ram, 0xFFF57C), g.flow.front.demo.left),
        ("G.$757A counter", ram16(ram, 0xFFF57A), g.flow.front.demo_counter),
        ("G.$4D0 mode", b(0xFF84D0), g.flow.mode as u16),
        ("G.$4D9 stage", b(0xFF84D9), g.w.stage as u16),
        ("G.$786 area", b(0xFF8786), g.cam.s2.area as u16),
        ("G.$4DA frame", b(0xFF84DA), g.flow.frame as u16),
    ]
    .into_iter()
    .filter(|&(_, m, p)| m != p)
    .collect()
}

/// The modeled bytes of the three player objects (0xFFB274 + 0x180 * i) that differ: (player, offset, MAME, port).
fn player_diffs(g: &mut Game, ram: &[u8]) -> Vec<(usize, usize, u8, u8)> {
    let c = g.trace_ram();
    let mut out = Vec::new();
    for i in 0..3 {
        for o in 0..0x180 {
            let a = 0xB274 + 0x180 * i + o;
            if c.mask[a] != 0 && c.ram[a] != ram[a] {
                out.push((i, o, ram[a], c.ram[a]));
            }
        }
    }
    out
}

/// What one record of a demo shows.
#[derive(Clone, Debug, Default)]
struct DemoRecord {
    /// The record number (video frame - 2) and the frames since the demo call (A).
    k: usize,
    rel: usize,
    d: Differences,
    globals: Vec<(&'static str, u16, u16)>,
    /// Modeled bytes of the player objects that differ, only collected at the checkpoints.
    players: Vec<(usize, usize, u8, u8)>,
    /// Palette words that have the colour of MAME's word but another brightness nibble (a fade that is not in step).
    bright: usize,
}

/// Runs the port from power on against `trace` up to `last` and collects the records of the demos: for each demo the
/// frame A the title returned (the demo counter G.$757A of the recording steps) and the records A ..= A + 1326. Returns
/// (A, records) per demo. `check` lists the demo frames (G.$0) at which the player objects are compared too.
fn run_demos(
    a: &'static AssetPack,
    trace: &str,
    last: i32,
    before: impl FnMut(i32, &mut Game, &[u8]),
    checks: &[u16],
) -> (Game, Vec<(usize, Vec<DemoRecord>)>) {
    let mut demos: Vec<(usize, Vec<DemoRecord>)> = Vec::new();
    let mut counter = 0u16;
    let mut prev_state = 0u8;
    let mut extra: Vec<Option<DemoRecord>> = Vec::new();
    let (g, rows) = compare_power_on_with(a, trace, last, |_| [0; 3], before, |k, g, ram, gfx| {
        // the call of the demo: the counter G.$757A steps in the record where the attract state byte goes to 0 (the
        // sweeps of the RAM clear take it out); a poke of the counter alone is not a demo
        let c = ram16(ram, 0xFFF57A);
        let state = ram[0xFF0FD0 - 0xFF0000];
        if c != counter {
            if state == 0 && prev_state != 0 {
                demos.push((k as usize, Vec::new()));
            }
            counter = c;
        }
        prev_state = state;
        let rel = demos.last().map(|(s, _)| k as usize - s).unwrap_or(usize::MAX);
        let mut r = None;
        if rel <= 1326 {
            let mut rec = DemoRecord { k: k as usize, rel, globals: demo_globals(g, ram), ..Default::default() };
            rec.bright = (0..2048)
                .filter(|&i| {
                    let m = u16::from_be_bytes([gfx[PALETTES + 2 * i], gfx[PALETTES + 2 * i + 1]]);
                    m & 0x0FFF == g.pal.words[i] & 0x0FFF && m & 0xF000 != g.pal.words[i] & 0xF000
                })
                .count();
            if checks.contains(&ram16(ram, 0xFF8000)) && ram16(ram, 0xFF8000) == g.w.frame && rel > 5 {
                rec.players = player_diffs(g, ram);
            }
            r = Some(rec);
        }
        extra.push(r);
    });
    for (k, e) in extra.into_iter().enumerate() {
        if let Some(mut rec) = e {
            rec.d = rows[k].clone();
            if let Some((_, v)) = demos.iter_mut().rev().find(|(s, _)| *s <= k) {
                v.push(rec);
            }
        }
    }
    (g, demos)
}

/// Diagnostic report of the demos of a recording: for each demo the first record (relative to the call A) where each kind
/// of difference shows, and the number of records. `DINO_TRACE` (default attract3), `DINO_LAST` (default 24400, the end
/// of the fourth demo), `DINO_DEMO` (the demo to list in full, by position).
#[test]
#[ignore]
fn demo_report() {
    let Some(a) = assets() else { return };
    let trace = std::env::var("DINO_TRACE").unwrap_or_else(|_| "../../../re/out/traces/attract3.mame.trc".into());
    let last = std::env::var("DINO_LAST").ok().and_then(|v| v.parse().ok()).unwrap_or(24400);
    let full: Option<usize> = std::env::var("DINO_DEMO").ok().and_then(|v| v.parse().ok());
    // `DINO_POKE_COUNTER`: the demo counter G.$757A is set at record 1000 (the recordings `flow_m112-row5` and `-row0`)
    let poke: Option<u16> = std::env::var("DINO_POKE_COUNTER").ok().and_then(|v| v.parse().ok());
    let (_, demos) = run_demos(
        a,
        &trace,
        last,
        |k, g, _| {
            if let (1000, Some(c)) = (k, poke) {
                g.flow.front.demo_counter = c;
            }
        },
        &[60, 300, 600],
    );
    for (n, (start, recs)) in demos.iter().enumerate() {
        println!("demo {n}: call at record {start}, {} records", recs.len());
        let kinds: [(&str, Box<dyn Fn(&DemoRecord) -> bool>); 10] = [
            ("scroll1", Box::new(|r| r.d.s1 > 0)),
            ("scroll2", Box::new(|r| r.d.s2 > 0)),
            ("scroll3", Box::new(|r| r.d.s3 > 0)),
            ("palette", Box::new(|r| r.d.pal > 0)),
            ("sprites", Box::new(|r| r.d.sprites.is_some())),
            ("layer", Box::new(|r| r.d.layer.is_some())),
            ("counter", Box::new(|r| r.d.counter.is_some())),
            ("state", Box::new(|r| r.d.state.is_some())),
            ("sounds", Box::new(|r| r.d.sounds.is_some())),
            ("globals", Box::new(|r| !r.globals.is_empty())),
        ];
        for (name, f) in &kinds {
            let hits: Vec<usize> = recs.iter().filter(|r| f(r)).map(|r| r.rel).collect();
            if let Some(first) = hits.first() {
                // the runs of consecutive records, the first six
                let mut runs: Vec<(usize, usize)> = Vec::new();
                for &h in &hits {
                    match runs.last_mut() {
                        Some(r) if r.1 + 1 == h => r.1 = h,
                        _ => runs.push((h, h)),
                    }
                }
                let shown: Vec<String> = runs.iter().take(6).map(|&(a, b)| if a == b { format!("{a}") } else { format!("{a}..{b}") }).collect();
                println!("  {name:8} first at A+{first}, {} records in {} runs: {}{}", hits.len(), runs.len(), shown.join(" "), if runs.len() > 6 { " ..." } else { "" });
            }
        }
        for r in recs {
            if r.rel <= 12 && (!r.d.none() || !r.globals.is_empty()) {
                println!("  A+{}: {:?} {:?}", r.rel, r.d, r.globals);
            }
            if !r.players.is_empty() {
                println!("  A+{} (G.$0 checkpoint): {} player bytes differ, first {:?}", r.rel, r.players.len(), &r.players[..r.players.len().min(8)]);
            }
        }
        if let Some(r) = recs.iter().find(|r| r.rel == 1326) {
            println!("  A+1326: {:?} {:?}", r.d, r.globals);
        }
        if let Some(first) = recs.iter().find(|r| r.rel >= 10 && r.globals.iter().any(|g| g.0.starts_with("G.$14"))) {
            println!("  the generator first differs at A+{}: {:?}", first.rel, first.globals);
        }
        if full == Some(n) {
            for r in recs {
                println!("  A+{}: {:?} {:?}", r.rel, r.d, r.globals);
            }
        }
    }
}

/// The recording of 32000 frames from power on with no input (`re/harness/scripts/m113_long.txt`, `record.sh --batch`).
const TRACE_LONG: &str = "../../../re/out/traces/flow_m113-long.mame.trc";
/// The first demo with the counter G.$757A poked at power frame 1000 (`re/harness/scripts/m112_demo.txt`): `-row5` (counter 4:
/// Episode 6 area 1) and `-row0` (counter 5: Episode 1 area 1).
const TRACE_ROW5: &str = "../../../re/out/traces/flow_m112-row5.mame.trc";
const TRACE_ROW0: &str = "../../../re/out/traces/flow_m112-row0.mame.trc";

/// What has to be equal in every demo: the setup, the frame counters and the end. `known` names the globals that are allowed to
/// differ (the generator draws numbers for objects the port does not run yet, G.$757C is read in the middle of a pass in the
/// first records).
fn assert_demo(n: usize, start: usize, recs: &[DemoRecord]) {
    assert_eq!(recs.len(), 1327, "demo {n}: the records from the call to the first record of the next screen");
    for r in recs.iter().filter(|r| r.rel <= 1325) {
        let what = format!("demo {n} (call at record {start}) record A+{}", r.rel);
        // the setup is equal in every layer, palette, sprite list, sound code and variable (A + 9 and A + 10: the frame boundary
        // falls inside the draw of the map and of the HUD, which the port draws whole; the first pass of Episode 1 starts in A + 9)
        if (2..=8).contains(&r.rel) {
            assert!(r.d.none() && r.globals.is_empty(), "{what}: {:?} {:?}", r.d, r.globals);
        }
        // the layer control, the title counter G.-$7022, the attract state byte, the frame counters G.$0 and G.$4DA, the demo
        // counter, the mode, the stage and the area are equal in every record, from the call to the fade
        assert!(r.d.layer.is_none() && r.d.counter.is_none() && r.d.state.is_none(), "{what}: {:?}", r.d);
        let bad: Vec<_> = r
            .globals
            .iter()
            .filter(|g| !g.0.starts_with("G.$14") && !(g.0.starts_with("G.$757C") && r.rel <= 12))
            .collect();
        assert!(bad.is_empty(), "{what}: {bad:?}");
        // the fade out of the last picture is in step (the colours may differ: objects of the level that the port does not run
        // have not loaded their palettes)
        if r.rel >= 1200 {
            assert_eq!(r.bright, 0, "{what}: palette words with the colour of MAME's but another brightness");
        }
    }
}

/// M1-12, the six demo rows: the setup of the demo from the call to the first pass (the RAM clear, the staged clear of the
/// screen, the palettes, the stage setup, the map, the HUD, the sound codes: D4d 3.6, A7), then in every record the layer control,
/// the attract state byte, the frame counters, the mode, the stage and the area, and the end (100 frozen frames and the fade).
/// The 5 demos of `flow_m113-long` (counters 1 to 5: Episodes 2, 3, 8, 5 and 6; the first runs and the later ones are timed
/// differently, see `demo.rs`).
#[test]
fn demos_set_up_and_run_in_step_with_mame() {
    let Some(a) = assets() else { return };
    if !Path::new(TRACE_LONG).exists() {
        eprintln!("mame_check: {TRACE_LONG} is missing, skipped");
        return;
    }
    let (_, demos) = run_demos(a, TRACE_LONG, 31000, |_, _, _| {}, &[]);
    assert_eq!(demos.iter().map(|d| d.0).collect::<Vec<_>>(), [3343, 9914, 16448, 23019, 29553], "the frames the title returned");
    for (n, (start, recs)) in demos.iter().enumerate() {
        assert_demo(n, *start, recs);
    }
    // the INSERT COIN blink task (the text at the bottom, on for 30 frames and off for 30) runs in the demos of Episode 2 (the first demo:
    // scroll1 is equal to the recording for 500 frames, ten blinks) and is killed in Episode 6 (the fifth: no text, scroll1 equal for 300 frames)
    let quiet = |n: usize, to: usize| demos[n].1.iter().filter(|r| (10..=to).contains(&r.rel)).all(|r| r.d.s1 == 0);
    assert!(quiet(0, 500), "the blink text of the first demo");
    assert!(quiet(4, 300), "no blink text in Episode 6 (stage 5)");
}

/// M1-12, the rows that the loop of the recording does not reach early: the counter G.$757A is set at record 1000 so that the
/// first demo is Episode 6 (row 5, counter 4) or Episode 1 (row 0, counter 5; the sixth demo, the counter wraps to 0).
#[test]
fn demo_rows_5_and_0_set_up_and_run_in_step_with_mame() {
    let Some(a) = assets() else { return };
    for (trace, counter) in [(TRACE_ROW5, 4u16), (TRACE_ROW0, 5)] {
        if !Path::new(trace).exists() {
            eprintln!("mame_check: {trace} is missing, skipped");
            continue;
        }
        let (_, demos) = run_demos(
            a,
            trace,
            4700,
            |k, g, _| {
                if k == 1000 {
                    g.flow.front.demo_counter = counter;
                }
            },
            &[],
        );
        assert_eq!(demos.iter().map(|d| d.0).collect::<Vec<_>>(), [3343], "{trace}");
        assert_demo(counter as usize, 3343, &demos[0].1);
    }
}

/// The whole attract loop from power on with no input, 40000 records (`re/harness/scripts/m1003_loop.txt`, `record.sh --batch`): the
/// six demos (counters 1 to 5, then the wrap to row 0), three passes of the four attract states.
const TRACE_LOOP: &str = "../../../re/out/traces/flow_m1003-loop.mame.trc";
/// The records of the demo calls in `flow_m1003-loop` (A: the title returned, G.$757A stepped) and the rows they play: Episode 2 area 0,
/// Episode 3 area 0, Episode 8 area 1, Episode 5 area 0, Episode 6 area 1, Episode 1 area 1.
const LOOP_CALLS: [usize; 6] = [3343, 9914, 16448, 23019, 29553, 36124];

/// M10-03 to M10-05: the six demos of the attract loop with their real levels, from power on (a free run of 37460 records, every demo
/// set up by the port's own RAM clear and played from the streams). In every demo from its first full pass (A + 11) to the first record
/// of the next screen (A + 1326) everything the test compares is equal to the recording: the three tilemaps, the four palette banks,
/// the sprite list, the layer control, the title counter, the attract state byte, the six scroll shadows, the sound codes, the system
/// variables, the random number state G.$14, the frame counters, G.$757C, the mode, stage and area; at the demo frames G.$0 = 60, 300,
/// 600, 900 and 1200 every modeled byte of the three player objects; the 100 frozen frames, the fade (A + 1210 on) and the first frame
/// of the clear that follows. The setup records are checked as in `demos_set_up_and_run_in_step_with_mame`. No exception: the tag team
/// move of Episode 3 (A + 928, Hannah and Mustapha, the trigger 0x6728) is equal too.
#[test]
fn the_six_demos_of_the_loop_match_mame() {
    let Some(a) = assets() else { return };
    if !Path::new(TRACE_LOOP).exists() {
        eprintln!("mame_check: {TRACE_LOOP} is missing, skipped");
        return;
    }
    let (_, demos) = run_demos(a, TRACE_LOOP, 37460, |_, _, _| {}, &[60, 300, 600, 900, 1200]);
    assert_eq!(demos.iter().map(|d| d.0).collect::<Vec<_>>(), LOOP_CALLS, "the frames the title returned");
    for (n, (start, recs)) in demos.iter().enumerate() {
        assert_demo(n, *start, recs);
        let bad: Vec<String> = recs
            .iter()
            .filter(|r| r.rel >= 11)
            .filter(|r| !r.d.none() || r.d.cams.is_some() || !r.d.core_sys().is_empty() || !r.globals.is_empty() || !r.players.is_empty())
            .map(|r| format!("A+{}: {:?} {:?} {:?}", r.rel, r.d, r.globals, &r.players[..r.players.len().min(4)]))
            .collect();
        // (G.$0 is equal in every record, so the player objects are compared at the five checkpoints of every demo)
        assert!(bad.is_empty(), "demo {n} (call at record {start}): {} records differ, first {:?}", bad.len(), &bad[..bad.len().min(2)]);
    }
}

/// Diagnostic: a coin (and, with `DINO_START`, a start) during the demo against the recordings of M1-13. `DINO_CASE` is `coin`
/// (`flow_m113-coin`: coin at 4000 to 4007) or `free` (`flow_m113-freestart`: free play, start at 4000 to 4005).
#[test]
#[ignore]
fn demo_input_report() {
    let Some(a) = assets() else { return };
    let case = std::env::var("DINO_CASE").unwrap_or_else(|_| "coin".into());
    let last = std::env::var("DINO_LAST").ok().and_then(|v| v.parse().ok()).unwrap_or(4590);
    let (name, free) = if case == "coin" { ("m113-coin", false) } else { ("m113-freestart", true) };
    let Some(trace) = flow_trace(name) else { return };
    let (_, rows) = compare_power_on(a, &trace, last, |_| [0; 3], |k, g| {
        if free && k == 99 {
            g.flow.front.sys.free_play = true;
        }
        g.w.coins = (case == "coin" && (3999..=4006).contains(&k)) as u8;
        g.w.system = 8 | (free && (3999..=4004).contains(&k)) as u8;
    });
    println!("s1      {}", ranges(&rows, |d| d.s1 > 0));
    println!("s2      {}", ranges(&rows, |d| d.s2 > 0));
    println!("s3      {}", ranges(&rows, |d| d.s3 > 0));
    println!("palette {}", ranges(&rows, |d| d.pal > 0));
    println!("sprites {}", ranges(&rows, |d| d.sprites.is_some()));
    println!("layer   {}", ranges(&rows, |d| d.layer.is_some()));
    println!("counter {}", ranges(&rows, |d| d.counter.is_some()));
    println!("sounds  {}", ranges(&rows, |d| d.sounds.is_some()));
    println!("state   {}", ranges(&rows, |d| d.state.is_some()));
    println!("sys     {}", ranges(&rows, |d| !d.sys.is_empty()));
    let from: usize = std::env::var("DINO_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(3990);
    for (k, d) in rows.iter().enumerate().skip(from) {
        if !d.none() || !d.sys.is_empty() {
            println!("record {k}: {d:?}");
        }
    }
}

/// M1-13, a coin during the demo (`flow_m113-coin`: the coin at 4000 to 4007, the demo of Episode 2 is half way): the system
/// task kills the attract tasks and calls the press start screen 0x983EE, which takes it 3 frames (records 4008 to 4010: sprites and
/// scroll1, then scroll2 and the first part of scroll3, then the rest of scroll3, the palette load and the new task, M1-15); the demo
/// does not finish. From the record 4008 the palettes, the sprite list, the layer control and the sound codes equal the recording
/// and from 4010 the layers too, scroll1 with the credit line and the prompts of the system task included (until then the level cells
/// of the demo that are not cleared yet differ: the level is not complete); all the variables of the system task equal it.
/// G.$4CD, which the demo set, is cleared in the
/// tail of the pass of the system task, 3 frames after the call (record 4011).
#[test]
fn coin_during_the_demo_matches_mame() {
    let Some(a) = assets() else { return };
    let Some(trace) = flow_trace("m113-coin") else { return };
    let (_, rows) = compare_power_on(a, &trace, 4590, |_| [0; 3], |k, g| g.w.coins = (3999..=4006).contains(&k) as u8);
    let bad: Vec<String> = rows
        .iter()
        .enumerate()
        .skip(3343)
        .filter(|(k, d)| {
            !d.core_sys().is_empty()
                || d.state.is_some()
                || d.counter.is_some()
                // 4008 to 4010 are the three frames of the call: the level cells of the demo that are not cleared yet differ (the
                // level is not complete, M10), the palette load, the sprite list, the layer control and the sounds equal
                || (*k >= 4008 && (d.pal > 0 || d.sprites.is_some() || d.layer.is_some() || d.sounds.is_some()))
                || (*k >= 4010 && d.s1 + d.s2 + d.s3 > 0)
        })
        .map(|(k, d)| format!("record {k}: {d:?}"))
        .collect();
    assert!(bad.is_empty(), "{} records differ, first {:?}", bad.len(), &bad[..bad.len().min(3)]);
}

/// M1-13, a start during the demo (`flow_m113-freestart`: free play from power frame 100, the start at 4000 to 4005): the game starts at
/// once from any moment of the loop, the demo ends where it is: sound 0x31, the fade out of the demo picture in 16 steps (records
/// 3999 to 4014, the palette words equal), the clear of all layers (the sprite buffers and scroll1 in 4015, scroll2 and the first part of
/// scroll3 in 4016, the rest of scroll3 in 4017), the game task (G.$4CC, G.$4CD, G.$4D5, G.$4D0 = 0 in 4017) and the select (mode 4) in 4021.
/// Every watched system variable equals up to the select (the select itself is a stub, M2-03); the layers equal from the end of the clear.
#[test]
fn start_during_the_demo_goes_to_select() {
    let Some(a) = assets() else { return };
    let Some(trace) = flow_trace("m113-freestart") else { return };
    let (_, rows) = compare_power_on(a, &trace, 4021, |_| [0; 3], |k, g| {
        if k == 99 {
            g.flow.front.sys.free_play = true; // the poke
        }
        g.w.system = 8 | (3999..=4004).contains(&k) as u8;
    });
    let bad: Vec<String> = rows
        .iter()
        .enumerate()
        .skip(3343)
        .filter(|(k, d)| {
            !d.sys.is_empty()
                || d.state.is_some()
                || d.counter.is_some()
                || (*k >= 3999 && *k <= 4014 && d.pal > 0)
                || (*k == 4017 || *k == 4018) && !d.none()
                || (*k >= 4015 && d.sprites.is_some())
        })
        .map(|(k, d)| format!("record {k}: {d:?}"))
        .collect();
    assert!(bad.is_empty(), "{} records differ, first {:?}", bad.len(), &bad[..bad.len().min(3)]);
}

/// A3 (M1-13), the whole loop for 32000 frames from power on with no input (`flow_m113-long`): the attract state byte equals the recording
/// in every record, so the state change frames equal it (records 2742, 6872, 9309, 13409, 15846, 19977, 22414, 26514, 28951 = the video
/// frames 2744 ... 28953 of D4d s3.2), and so does the demo counter G.$757A, which steps in the records 3343, 9914, 16448, 23019 and
/// 29553 (the video frames 3345, 9916, 16450, 23021, 29555: counters 1 to 5). The title, the layers of the screens that are real
/// and the sounds of the real screens are compared by the tests of those screens; the demos by `demos_set_up_and_run_in_step_with_mame`.
#[test]
fn attract_loop_32000_frames_match_mame() {
    let Some(a) = assets() else { return };
    if !Path::new(TRACE_LONG).exists() {
        eprintln!("mame_check: {TRACE_LONG} is missing, skipped");
        return;
    }
    let (mut mame_state, mut port_state) = (Vec::new(), Vec::new());
    let (mut mame_counter, mut port_counter) = (Vec::new(), Vec::new());
    let (mut last_m, mut last_p) = ((0u8, 0u16), (0u8, 0u16));
    let (_, rows) = compare_power_on_with(a, TRACE_LONG, 31990, |_| [0; 3], |_, _, _| {}, |k, g, ram, _| {
        let m = (ram[0xFF0FD0 - 0xFF0000], ram16(ram, 0xFFF57A));
        let p = (g.flow.front.attract_state, g.flow.front.demo_counter);
        if m.0 != last_m.0 {
            mame_state.push((k, m.0));
        }
        if p.0 != last_p.0 {
            port_state.push((k, p.0));
        }
        if m.1 != last_m.1 {
            mame_counter.push((k, m.1));
        }
        if p.1 != last_p.1 {
            port_counter.push((k, p.1));
        }
        (last_m, last_p) = (m, p);
    });
    let state = ranges(&rows[25..], |d| d.state.is_some());
    assert!(state.is_empty(), "the attract state byte differs at records {state}");
    assert_eq!(port_state, mame_state, "the state changes (the RAM clear of the demo takes the byte out for 3 frames, hence the 0s)");
    assert_eq!(port_counter, mame_counter, "the demo counter steps");
    // the counter steps at the call of each demo and goes back to 0 for the sixth (not in these 32000 frames)
    let counter_steps: Vec<(i32, u16)> = port_counter.iter().copied().filter(|&(_, c)| c != 0).collect();
    assert_eq!(counter_steps, [(3343, 1), (9914, 2), (16448, 3), (23019, 4), (29553, 5)]);
    // D4d s3.2: the state byte changes at the video frames 2744, 6874, 9311, 13411, 15848, 19979, 22416, 26516 and 28953
    let steps: Vec<i32> = port_state.iter().map(|&(k, _)| k).collect();
    for expected in [2742, 6872, 9309, 13409, 15846, 19977, 22414, 26514, 28951] {
        assert!(steps.contains(&expected), "state change at record {expected} (video frame {}) is missing in {steps:?}", expected + 2);
    }
}

/// The demo calls of `attract3.mame.trc` (the records where G.$757A steps) and the records the demo routine returns in (A + 1325).
const LOOP_DEMOS: [(usize, usize); 4] = [(3343, 4668), (9914, 11239), (16448, 17773), (23019, 24344)];

/// What the whole loop of `attract3.mame.trc` is compared with: the records `from ..= last` of the loop from power on, per record all
/// differences of `Differences` (tilemaps, palettes, sprite list, layer control, title counter, sound codes, attract state byte, the six
/// scroll shadows) and the system variables, and G.$14 itself. Until M10-03 the random number state was set from the recording in the
/// frame each demo returned (the demos drew numbers in classes the port did not run); the demos play their real levels now and leave
/// the original's G.$14, so nothing is set.
fn run_loop(a: &'static AssetPack, trace: &str, last: i32) -> (Vec<Differences>, Vec<(i32, u16, u16)>) {
    let mut rng_bad: Vec<(i32, u16, u16)> = Vec::new();
    let (_, rows) = compare_power_on_with(
        a,
        trace,
        last,
        |_| [0; 3],
        |_, _, _| {},
        |k, g, ram, _| {
            let in_demo = LOOP_DEMOS.iter().any(|&(s, e)| (s..e).contains(&(k as usize)));
            if k >= 25 && !in_demo && g.w.rng != ram16(ram, 0xFF8014) {
                rng_bad.push((k, ram16(ram, 0xFF8014), g.w.rng));
            }
        },
    );
    (rows, rng_bad)
}

/// A1 to A9 (M1-15), the whole attract loop from power on, no input: the port is run for 26600 records (two loops of the four states
/// and the title of the third, 13104 records a loop) and compared with `attract3.mame.trc` record by record: the three tilemaps,
/// the four palette banks, the sprite list, the layer control, the title counter, the sound codes, the attract state byte, the six
/// scroll shadows, the variables of the system task, and G.$14. Records 25 on (0 to 24 are the reset code clearing the video RAM).
///
/// The boot, the title, the profile cards, the ranking, the story slides and the opening movie (all four runs of it) are equal in
/// EVERY record, without exception, and so are the records of the clear after each demo (M10-03: the demos leave the original's
/// picture and G.$14, nothing is set from the recording). The records of the four demos (A .. A + 1324) are not compared here: they
/// are the test `the_six_demos_of_the_loop_match_mame` (`flow_m1003-loop`), which lists what differs there.
#[test]
fn attract_loop_two_loops_match_mame() {
    let Some(a) = assets() else { return };
    let trace = "../../../re/out/traces/attract3.mame.trc";
    if !Path::new(trace).exists() {
        eprintln!("mame_check: {trace} is missing, skipped");
        return;
    }
    let last = 26600;
    let (rows, rng_bad) = run_loop(a, trace, last);
    let mut compared = 0;
    let mut bad: Vec<String> = Vec::new();
    for (k, d) in rows.iter().enumerate().skip(25) {
        if LOOP_DEMOS.iter().any(|&(s, e)| (s..e).contains(&k)) {
            continue;
        }
        compared += 1;
        if !d.none() || d.cams.is_some() || !d.core_sys().is_empty() {
            bad.push(format!("record {k}: {d:?}"));
        }
    }
    assert!(bad.is_empty(), "{} of {compared} records differ outside the demos, first: {:?}", bad.len(), &bad[..bad.len().min(3)]);
    assert!(rng_bad.is_empty(), "the random number state (record, MAME, port) differs outside the demos: {:?}", &rng_bad[..rng_bad.len().min(4)]);
    assert_eq!(compared, 26601 - 25 - 4 * 1325, "records compared (the loop minus the reset and the four demos)");
    // the demos are in step in everything that is not level content (`demos_set_up_and_run_in_step_with_mame` has the details)
    for &(s, e) in &LOOP_DEMOS {
        let off: Vec<usize> = (s..e).filter(|&k| rows[k].layer.is_some() || rows[k].counter.is_some() || rows[k].state.is_some() || !rows[k].core_sys().is_empty()).collect();
        assert!(off.is_empty(), "demo {s}: the layer control, the title counter, the state byte or the system variables differ at {:?}", &off[..off.len().min(4)]);
    }
}

/// M1-15 diagnostic: the loop of `attract3.mame.trc` (see `run_loop`), the records OUTSIDE the demos that differ, by kind. With
/// `DINO_FROM=n` the records from n on are listed in full. `cargo test -p dino-game loop_report -- --ignored --nocapture`.
#[test]
#[ignore]
fn loop_report() {
    let Some(a) = assets() else { return };
    let trace = std::env::var("DINO_TRACE").unwrap_or_else(|_| "../../../re/out/traces/attract3.mame.trc".into());
    let last: i32 = std::env::var("DINO_LAST").ok().and_then(|v| v.parse().ok()).unwrap_or(29490);
    let (mut rows, rng_bad) = run_loop(a, &trace, last);
    drop_demo_blips(&mut rows);
    for (s, e) in LOOP_DEMOS {
        for d in rows.iter_mut().take(e).skip(s) {
            *d = Differences::default();
        }
    }
    for d in rows.iter_mut().take(25) {
        *d = Differences::default();
    }
    println!("s1      {}", ranges(&rows, |d| d.s1 > 0));
    println!("s2      {}", ranges(&rows, |d| d.s2 > 0));
    println!("s3      {}", ranges(&rows, |d| d.s3 > 0));
    println!("palette {}", ranges(&rows, |d| d.pal > 0));
    println!("sprites {}", ranges(&rows, |d| d.sprites.is_some()));
    println!("layer   {}", ranges(&rows, |d| d.layer.is_some()));
    println!("counter {}", ranges(&rows, |d| d.counter.is_some()));
    println!("sounds  {}", ranges(&rows, |d| d.sounds.is_some()));
    println!("state   {}", ranges(&rows, |d| d.state.is_some()));
    println!("cams    {}", ranges(&rows, |d| d.cams.is_some()));
    println!("sys     {}", ranges(&rows, |d| !d.core_sys().is_empty()));
    println!("rng     {:?}", &rng_bad[..rng_bad.len().min(8)]);
    let from: usize = std::env::var("DINO_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(999999);
    for (k, d) in rows.iter().enumerate().skip(from) {
        if !d.none() || d.cams.is_some() || !d.core_sys().is_empty() {
            println!("record {k}: {d:?}");
        }
    }
}

/// Index ranges "a..=b" of a list of cell numbers (sorted).
fn index_ranges(v: &[usize]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut k = 0;
    while k < v.len() {
        let s = k;
        while k + 1 < v.len() && v[k + 1] == v[k] + 1 {
            k += 1;
        }
        out.push(if s == k { format!("{}", v[s]) } else { format!("{}..={}", v[s], v[k]) });
        k += 1;
    }
    out.join(" ")
}

/// The values of the character select that a record holds (name, MAME, port): the random number state, G.$4DA and G.$0000, the mode bytes,
/// the join states of the three players (+$A2), their object present flags, G.$4CD and G.$4E4 and the select struct G.$62C (0x38 bytes).
fn select_fields(g: &Game, ram: &[u8]) -> Vec<(String, String, String)> {
    let b = |a: u32| ram[(a - 0xFF0000) as usize];
    let w = |a: u32| ram16(ram, a);
    let mut v: Vec<(String, String, String)> = Vec::new();
    let mut add = |n: &str, m: String, p: String| {
        if m != p {
            v.push((n.to_string(), m, p));
        }
    };
    add("rng G.$14", format!("{:04X}", w(0xFF8014)), format!("{:04X}", g.w.rng));
    add("G.$4DA", format!("{:02X}", b(0xFF84DA)), format!("{:02X}", g.flow.frame));
    add("G.$0000", format!("{:04X}", w(0xFF8000)), format!("{:04X}", g.w.frame));
    add("sub", format!("{:02X}", b(0xFF84D1)), format!("{:02X}", g.flow.sub));
    add("sub2", format!("{:02X}", b(0xFF84D2)), format!("{:02X}", g.flow.sub2 >> 8));
    for (i, a) in [0xFFB316u32, 0xFFB496, 0xFFB616].into_iter().enumerate() {
        add(&format!("join state P{}", i + 1), format!("{:02X}", b(a)), format!("{:02X}", g.flow.join[i].state));
        add(&format!("object live P{}", i + 1), format!("{:02X}", b(0xFFB274 + 0x180 * i as u32)), format!("{:02X}", g.players[i].b.live));
    }
    add("G.$4CD", format!("{:02X}", b(0xFF84CD)), format!("{:02X}", g.flow.started));
    add("G.$4E4", format!("{:02X}", b(0xFF84E4)), format!("{:02X}", g.flow.chosen));
    let mine = g.flow.front.select.bytes();
    let theirs: Vec<u8> = (0..0x38).map(|k| b(0xFF862C + k)).collect();
    if theirs[..] != mine[..] {
        add("select G.$62C", theirs.iter().map(|x| format!("{x:02X}")).collect::<Vec<_>>().join(""), mine.iter().map(|x| format!("{x:02X}")).collect::<Vec<_>>().join(""));
    }
    v
}

/// Diagnostic for the character select: for the records `DINO_FROM ..= DINO_LAST` of a flow recording (`DINO_FLOW`, a start at `DINO_START`
/// as in `press_report`, `DINO_KEYS` = "tick:player:bits" list for the inputs of the select, `DINO_COIN`) prints what differs from the
/// recording: the picture (layers, palettes, sprites) and `select_fields`.
#[test]
#[ignore]
fn select_report() {
    let Some(a) = assets() else { return };
    let name = std::env::var("DINO_FLOW").unwrap_or_else(|_| "sel-1p".into());
    let Some(trace) = flow_trace(&name) else { return };
    let get = |k: &str| std::env::var(k).ok();
    let from: usize = get("DINO_FROM").and_then(|v| v.parse().ok()).unwrap_or(700);
    let last: i32 = get("DINO_LAST").and_then(|v| v.parse().ok()).unwrap_or(800);
    let parse = |s: Option<String>| -> Vec<(u8, i32, i32)> {
        s.unwrap_or_default()
            .split(',')
            .filter_map(|r| {
                let (p, r) = r.split_once(':').unwrap_or(("1", r));
                let (x, y) = r.split_once('-')?;
                Some((p.parse::<u8>().ok()?, x.parse::<i32>().ok()?, y.parse::<i32>().ok()?))
            })
            .collect()
    };
    // the segments of the scenario file (script lines of the recording): "a-b:keys" per player as in `re/harness/scripts/flow_select_name.txt`
    let script = select_script(&name);
    let _ = parse;
    let (_, rows) = compare_power_on_with(
        a,
        &trace,
        last,
        |_| [0; 3],
        |k, g, _| {
            g.w.coins = script_bits(&script.coins, k);
            g.w.system = 8 | script_bits(&script.starts, k);
            g.w.input = script.input(k);
        },
        |k, g, ram, _| {
            if (k as usize) >= from {
                for (n, m, p) in select_fields(g, ram) {
                    println!("record {k}: {n}: MAME {m} port {p}");
                }
            }
        },
    );
    for (k, d) in rows.iter().enumerate().skip(from) {
        if d.s1 + d.s2 + d.s3 + d.pal > 0 || d.sprites.is_some() || d.layer.is_some() || d.sounds.is_some() || !d.sys.is_empty() {
            println!("record {k}: s1 {} s2 {} s3 {} pal {} sprites {:?} layer {:?} sounds {:?} sys {:?}", d.s1, d.s2, d.s3, d.pal, d.sprites, d.layer, d.sounds, d.sys);
        }
    }
}

/// The input script of a flow recording of the select (coins, starts and the buttons of the players), read from the scenario file.
struct SelectScript {
    coins: Vec<(u8, i32, i32)>,
    starts: Vec<(u8, i32, i32)>,
    /// (player 1 to 3, first, last, bits of G.$16: 1 right, 2 left, 4 down, 8 up, 0x10 attack, 0x20 jump)
    keys: Vec<(u8, i32, i32, u8)>,
}

impl SelectScript {
    fn input(&self, tick: i32) -> [u8; 3] {
        let mut v = [0u8; 3];
        for &(p, a, b, bits) in &self.keys {
            if (a - 1..=b - 1).contains(&tick) {
                v[p as usize - 1] |= bits;
            }
        }
        v
    }
}

/// Reads the segment `name` of `flow_select_name.txt`: lines `a-b:KEYS` (R L U D A J S C and the digits 2, 3 that choose the player).
fn select_script(name: &str) -> SelectScript {
    let text = std::fs::read_to_string("../../../re/harness/scripts/flow_select_name.txt").unwrap_or_default();
    let mut out = SelectScript { coins: vec![], starts: vec![], keys: vec![] };
    let mut inside = false;
    for line in text.lines().map(str::trim) {
        if let Some(n) = line.strip_prefix("segment ") {
            inside = n == name;
        } else if inside {
            let Some((range, keys)) = line.split_once(':') else { continue };
            let Some((a, b)) = range.split_once('-') else { continue };
            let (Ok(a), Ok(b)) = (a.parse::<i32>(), b.parse::<i32>()) else { continue };
            let mut player = 1u8;
            for c in keys.chars() {
                match c {
                    '1'..='3' => player = c as u8 - b'0',
                    'C' => out.coins.push((player, a, b)),
                    'S' => out.starts.push((player, a, b)),
                    'R' => out.keys.push((player, a, b, 1)),
                    'L' => out.keys.push((player, a, b, 2)),
                    'D' => out.keys.push((player, a, b, 4)),
                    'U' => out.keys.push((player, a, b, 8)),
                    'A' => out.keys.push((player, a, b, 0x10)),
                    'J' => out.keys.push((player, a, b, 0x20)),
                    _ => {}
                }
            }
        }
    }
    out
}

/// The first record of a recording in which the game mode G.$4D0 is 0xC after the select (the level), or the last record when the
/// recording ends in the select (the second value tells which).
fn first_level_record(trace: &str) -> Option<(i32, bool)> {
    let mut rd = TraceReader::open(trace).ok()?;
    rd.next();
    let mut last = 0;
    while let Some(f) = rd.next() {
        let ram = &rd.region("ram").unwrap().data;
        if f > 600 && ram[0xFF84D0 - 0xFF0000] == 0x0C {
            return Some((f, true));
        }
        last = f;
    }
    Some((last, false))
}

/// The sound requests with their d1 / d2 longs that were queued between two records (the ring G.$3A, entries of 16 bytes: the code word,
/// d1, d2).
fn mame_sound_args(prev: &[u8], ram: &[u8]) -> Vec<(u16, u32, u32)> {
    let (mut a, b) = (ram16(prev, 0xFF8034) & 0x3F0, ram16(ram, 0xFF8034) & 0x3F0);
    let mut v = Vec::new();
    while a != b {
        let e = 0xFF803A + a as u32;
        v.push((ram16(ram, e), (ram16(ram, e + 2) as u32) << 16 | ram16(ram, e + 4) as u32, (ram16(ram, e + 6) as u32) << 16 | ram16(ram, e + 8) as u32));
        a = (a + 0x10) & 0x3F0;
    }
    v.retain(|e| e.0 != 0);
    v
}

/// The front end codes of M4-15 (select, continue, game over, story, tally, ending, lab, name entry) whose d1 and d2 are compared
/// with the recordings. The select music 0x00 is not in the list: `mame_sound_args` drops code 0 (it is also the empty ring entry).
pub(super) const FRONT_END_CODES: [u16; 21] = [0x32, 0x33, 0x1B, 0x35, 0x1C, 0xAB, 0xE6, 0x1D, 0x1F, 0x13, 0x14, 0x15, 0x16, 0x49, 0x19, 0x1A, 0x66, 0xBB, 0x10, 0xF6, 0x56];

/// M4-15: the requests of the listed codes whose d1 or d2 differs from the recording, one text per record (MAME, port).
fn arg_diffs_of(rows: &[Differences], codes: &[u16]) -> Vec<String> {
    let listed = |v: &[(u16, u32, u32)]| v.iter().filter(|e| codes.contains(&e.0)).copied().collect::<Vec<_>>();
    rows.iter()
        .enumerate()
        .filter_map(|(k, d)| {
            let (m, p) = d.sound_args.as_ref()?;
            let (mf, pf) = (listed(m), listed(p));
            (mf != pf).then(|| format!("record {k}: MAME {mf:X?} port {pf:X?}"))
        })
        .collect()
}

/// M4-15: the requests of the listed codes (`FRONT_END_CODES`) with d1 and d2. Three codes take d2 from the registers the frame left
/// (M10, register leftovers, not modelled), so only their d1 is compared: 0x1A (`0x8A7A0`, `move.w #$1A,d0; jsr 0xA63C`: 0x1388 in the
/// recordings, 0xE69 in `flow_end-3p` at record 3955), 0x1D (`0xEDB4`, after `bsr.w 0xEFC2`: MAME 0x45C3 in `flow_k-1p` at 3506, the
/// port sends 0) and 0x1F (`0xEDFC`, after the allocations `jsr 0x4884`: MAME 0x45C3 in `flow_k-name-entry`).
pub(super) fn listed_args(v: &[(u16, u32, u32)]) -> Vec<(u16, u32, u32)> {
    v.iter().filter(|e| FRONT_END_CODES.contains(&e.0)).map(|&(c, d1, d2)| (c, d1, if matches!(c, 0x1A | 0x1D | 0x1F) { 0 } else { d2 })).collect()
}

/// M4-15: the listed requests of this frame (`FRONT_END_CODES`) with d1 and d2, MAME against the port, when they differ.
fn listed_arg_diff(prev: &[u8], ram: &[u8], g: &Game) -> Option<String> {
    let m = listed_args(&mame_sound_args(prev, ram));
    let p = listed_args(&g.w.snd_reqs.iter().filter(|r| r.code != 0).map(|r| (r.code, r.d1, r.d2)).collect::<Vec<_>>());
    (m != p).then(|| format!("MAME {m:X?} port {p:X?}"))
}

/// M2-03 and M2-04, the character select from power on to the level, record by record against the recordings of `flow_select_name` (the
/// scripts are read from the scenario file): `sel-1p` (the 10 second timer from 725, the auto pick 610 frames after the screen starts, the
/// palette flash, the 16 end steps, the fade out, mode 0xC 159 records after the pick), `sel-1p-moves` (left and right presses, attack
/// pick), `sel-same` (player 2 starts the game, player 1 presses start in the fade: it never joins; player 2 moves left and right while
/// player 1 is an absent player with a credit: the prompt blinks; both pick, no hero twice), `sel-2p-late` (player 2 starts at 841: the
/// timer goes back to 10, the lock counter of the new record, its panel), `sel-3p-all-left` (three cursors rotate without a clash),
/// `sel-3p-timeout` (nobody presses: all three pick at 0), `sel-picks-sim` (both pick in the same frame).
/// Every record from the boot equals the recording in the three layers, the palettes, the sprite list, the layer control, the sound
/// codes, G.$14, G.$4DA, G.$0000, the mode bytes, the join states, G.$4CD, G.$4E4 and the 0x38 bytes of the struct G.$62C, up to 3 records
/// before the level (the intro HUD of Episode 1 is the temporary one); the level mode comes in the same record. Known and tolerated: the
/// first game task iteration counts one frame earlier (G.$0000 and G.$4DA, record 718: mode 0 keeps the CPU), the palette cut of the press
/// start call in record 610 (2 words, CPU time) and the counters of the first level record. The pans of the sounds 0x32 and 0x33 are the
/// table 0xB24E by the cursor position (checked in the ring).
#[test]
fn character_select_matches_mame() {
    let Some(a) = assets() else { return };
    for name in ["sel-1p", "sel-1p-moves", "sel-same", "sel-2p-late", "sel-3p-all-left", "sel-3p-timeout", "sel-picks-sim"] {
        let Some(trace) = flow_trace(name) else { continue };
        let (level, has_level) = first_level_record(&trace).expect("the recording opens");
        // without a level the whole recording is compared (the stage start tail is not in it)
        let tail = if has_level { 3 } else { -1 };
        let script = select_script(name);
        let mut bad: Vec<String> = Vec::new();
        let mut first = -1;
        let mut pan_sounds = 0;
        let mut last_ram: Vec<u8> = Vec::new();
        let (_, rows) = compare_power_on_with(
            a,
            &trace,
            level,
            |_| [0; 3],
            |k, g, _| {
                g.w.coins = script_bits(&script.coins, k);
                g.w.system = 8 | script_bits(&script.starts, k);
                g.w.input = script.input(k);
            },
            |k, g, ram, _| {
                if first < 0 && ram16(ram, 0xFF8000) == 1 {
                    first = k;
                }
                for (n, m, p) in select_fields(g, ram) {
                    if (n == "G.$4DA" || n == "G.$0000") && (k == first || k == level) {
                        continue; // the first iteration of the game task and the first level record
                    }
                    bad.push(format!("record {k}: {n}: MAME {m} port {p}"));
                }
                // the sounds of the cursor: d2 is 0 and d1 the pan of the position (table 0xB24E) of one of the players with a cursor
                if !last_ram.is_empty() {
                    let pos: Vec<u8> = (0..3).map(|i| last_ram[0xFF862C + 0xC + 0xC * i + 2 - 0xFF0000]).filter(|&p| p < 4).collect();
                    for (code, d1, d2) in mame_sound_args(&last_ram, ram) {
                        pan_sounds += (code == 0x32 || code == 0x33) as usize;
                        if (code == 0x32 || code == 0x33) && !(d2 == 0 && pos.iter().any(|&p| d1 == g.select_pan(p) as u32)) {
                            bad.push(format!("record {k}: sound {code:#X} d1 {d1:#X} d2 {d2:#X}, positions {pos:?}"));
                        }
                    }
                }
                last_ram = ram.to_vec();
            },
        );
        bad.extend(arg_diffs_of(&rows, &FRONT_END_CODES).into_iter().map(|s| format!("{name}: {s}")));
        for (k, d) in rows.iter().enumerate().skip(25) {
            let mut d = d.clone();
            if k == 610 && d.pal <= 4 {
                d.pal = 0; // the cut of the brightness loop of the press start call
            }
            let picture = (k as i32) < level - tail && (d.s1 + d.s2 + d.s3 + d.pal > 0 || d.sprites.is_some() || d.layer.is_some());
            if picture || d.sounds.is_some() || !d.sys.is_empty() && (k as i32) < level - tail {
                bad.push(format!("record {k}: {d:?}"));
            }
            if !d.sys.is_empty() && (k as i32) >= level - tail && d.sys.iter().any(|s| s.0.starts_with("mode")) {
                bad.push(format!("record {k}: mode {:?}", d.sys));
            }
        }
        assert!(bad.is_empty(), "{name}: {} problems, first: {:?}", bad.len(), &bad[..bad.len().min(4)]);
        assert!(pan_sounds >= 1 && rows.len() > 1000, "{name}: {pan_sounds} cursor sounds in {} records", rows.len());
    }
}

/// M1-15 diagnostic: the records `DINO_FROM ..= DINO_LAST` of a flow recording (`DINO_FLOW`, for example `c-title`) with the
/// inputs `DINO_COIN` / `DINO_START` (a list of ranges of the port's tick, "a-b" for player 1 or "2:a-b" for player 2, for
/// example `449-456,2:499-506`) and `DINO_FREE=1` (free play poked at 99): for each
/// record the cell ranges of each layer and the palette words that differ from the recording.
#[test]
#[ignore]
fn press_report() {
    let Some(a) = assets() else { return };
    let name = std::env::var("DINO_FLOW").unwrap_or_else(|_| "c-title".into());
    let Some(trace) = flow_trace(&name) else { return };
    let get = |k: &str| std::env::var(k).ok();
    let ranges = |s: Option<String>| -> Vec<(u8, i32, i32)> {
        s.unwrap_or_default()
            .split(',')
            .filter_map(|r| {
                let (p, r) = r.split_once(':').unwrap_or(("1", r));
                let (x, y) = r.split_once('-')?;
                Some((p.parse::<u8>().ok()? - 1, x.parse::<i32>().ok()?, y.parse::<i32>().ok()?))
            })
            .collect()
    };
    let (coin, start) = (ranges(get("DINO_COIN")), ranges(get("DINO_START")));
    let bits = |v: &[(u8, i32, i32)], k: i32| v.iter().filter(|(_, x, y)| (*x..=*y).contains(&k)).fold(0u8, |a, (p, _, _)| a | 1 << p);
    let free = get("DINO_FREE").is_some();
    // `DINO_POKE_COUNTER`: the demo counter G.$757A is set at record 1000 (the recordings `flow_m112-row5` and `-row0`)
    let poke: Option<u16> = get("DINO_POKE_COUNTER").and_then(|v| v.parse().ok());
    let from: usize = get("DINO_FROM").and_then(|v| v.parse().ok()).unwrap_or(450);
    let last: i32 = get("DINO_LAST").and_then(|v| v.parse().ok()).unwrap_or(470);
    let (_, rows) = compare_power_on_with(a, &trace, last, |_| [0; 3], |k, g, _| {
        if k == 99 && free {
            g.flow.front.sys.free_play = true;
        }
        if let (1000, Some(c)) = (k, poke) {
            g.flow.front.demo_counter = c;
        }
        g.w.coins = bits(&coin, k);
        g.w.system = 8 | bits(&start, k);
    }, |k, g, _, gfx| {
        if (k as usize) < from {
            return;
        }
        for layer in 1..=3u8 {
            let full = diff(g, gfx, layer);
            let d: Vec<usize> = full.iter().map(|x| x.0).collect();
            if !d.is_empty() {
                println!("record {k}: scroll{layer} {} cells: {}", d.len(), index_ranges(&d));
                if get("DINO_CELLS").is_some() {
                    println!("    (cell, port code/attr, MAME code/attr) {:04X?}", &full[..full.len().min(6)]);
                }
            }
        }
        let p = palette_diff(g, gfx, 0xFFFF);
        if !p.is_empty() {
            println!("record {k}: palette {} words: {}", p.len(), index_ranges(&p));
        }
    });
    for (k, d) in rows.iter().enumerate().skip(from) {
        println!("record {k}: sprites {:?} layer {:?} counter {:?} sounds {:?} state {:?} sys {:?}", d.sprites, d.layer, d.counter, d.sounds, d.state, d.sys);
    }
}

/// The live objects of pool A (base 0xFFC8F4, 24 slots of 0xE0 bytes) and pool B (0xFFB6F4, 24 of 0xC0) of a MAME RAM image that have one of the
/// given classes (`+$20 / 4`), as (pool, class, variant, X, height, depth, mode, sub, act), sorted: the slot numbers are not compared.
fn mame_objects(ram: &[u8], classes_a: &[u16], classes_b: &[u16]) -> Vec<(char, u16, u16, u16, u16, u16, u8, u8, u8)> {
    let mut v = Vec::new();
    for (pool, base, stride, classes) in [('A', 0xFFC8F4u32, 0xE0u32, classes_a), ('B', 0xFFB6F4, 0xC0, classes_b)] {
        for i in 0..24 {
            let o = base + stride * i;
            let class = ram16(ram, o + 0x20) / 4;
            if ram[(o - 0xFF0000) as usize] != 0 && classes.contains(&class) {
                let b = |k: u32| ram[(o + k - 0xFF0000) as usize];
                v.push((pool, class, ram16(ram, o + 0x26), ram16(ram, o + 8), ram16(ram, o + 0xC), ram16(ram, o + 0x10), b(4), b(5), b(6)));
            }
        }
    }
    v.sort();
    v
}

/// The same list from the port.
fn port_objects(g: &Game, classes_a: &[u16], classes_b: &[u16]) -> Vec<(char, u16, u16, u16, u16, u16, u8, u8, u8)> {
    let mut v = Vec::new();
    for e in g.enemies.slots.iter().filter(|e| e.b.live != 0 && classes_a.contains(&(e.b.k20 / 4))) {
        let b = &e.b;
        v.push(('A', b.k20 / 4, b.var26, b.xi() as u16, b.yi() as u16, b.zi() as u16, b.mode, b.sub, b.act));
    }
    for p in g.props.slots.iter().filter(|p| p.b.live != 0 && classes_b.contains(&(p.b.k20 / 4))) {
        let b = &p.b;
        v.push(('B', b.k20 / 4, b.var26, b.xi() as u16, b.yi() as u16, b.zi() as u16, b.mode, b.sub, b.act));
    }
    v.sort();
    v
}

/// M2-10 to M2-12: the Episode 1 opening from power on (coin 598, start 698, Jack at 898: `coinstart.mame.trc`), record by record from the first
/// frame after the level start (1059) to the end of the special attack (1760): the layers scroll2 and scroll3 and the palettes equal MAME in every
/// record, scroll1 to 1677 (after it the join prompts of the absent heroes are drawn by the original's 0x7562, M2-18), the sprite list and the sound
/// codes in every record, and the state: the game mode, G.$0000 and G.$4DA, G.$4DD, G.$4E2, G.$4E3, G.$4D5, G.$4D8, G.$4F0, G.$506, the six scroll
/// shadows, the random number, hero 1 (position, state, action) and the objects of the opening (the bystander, the thugs and the flying member
/// with the master, the three banners, the birds and their spawner), by class, variant, position and state (not by slot: the hint object B20
/// is unported until M2-15 and is freed at once, so the later props take other slots). Not compared: records 1057 and 1058, where the original's
/// first level frame (the area setup, the map and the start of the scripts) takes about 2.5 video frames and the port runs it whole in 1057.
/// The timeline it checks (D4c A1): the camera pans 3 pixels a frame (mode 0x11) to 0x381 and stops (mode 0) at 1358, G.$506 is 1 from 1059 to
/// 1669, G.$4E3 is 1 in 1672, G.$4D5 is 0 in 1674 and G.$4DD is 0 in 1704.
#[test]
fn episode_one_opening_matches_mame() {
    let Some(a) = assets() else { return };
    let mut bad: Vec<String> = Vec::new();
    let mut seen = [0i32; 5]; // the records of 4E3, 4D5 = 0, 4DD = 0 and the camera mode 0 that the port shows, and 506 = 0
    let classes_a = [49u16, 50, 54, 55, 56, 57];
    let classes_b = [66u16, 73, 74];
    let (_, rows) = compare_power_on_with(
        a,
        TRACE_COINSTART,
        1760,
        |k| [if (898..=903).contains(&k) { 0x10 } else { 0 }, 0, 0],
        |k, g, _| {
            g.w.coins = (598..=605).contains(&k) as u8;
            g.w.system = 8 | (698..=703).contains(&k) as u8;
        },
        |k, g, ram, _| {
            if k < 1059 {
                return;
            }
            let b = |a: u32| ram[(a - 0xFF0000) as usize];
            let w = |a: u32| ram16(ram, a);
            let mut diff = |what: &str, m: u32, p: u32| {
                if m != p {
                    bad.push(format!("record {k}: {what} MAME {m:#x} port {p:#x}"));
                }
            };
            diff("G.$0000", w(0xFF8000) as u32, g.w.frame as u32);
            diff("G.$4DA", b(0xFF84DA) as u32, g.flow.frame as u32);
            diff("mode G.$4D0", b(0xFF84D0) as u32, g.flow.mode as u32);
            diff("G.$4DD", b(0xFF84DD) as u32, g.w.scripted as u32);
            diff("G.$4E2", b(0xFF84E2) as u32, g.flow.b4e2 as u32);
            diff("G.$4E3", b(0xFF84E3) as u32, g.flow.b4e3 as u32);
            diff("G.$4D5", b(0xFF84D5) as u32, g.spawn.g.b4d5 as u32);
            diff("G.$4D8", b(0xFF84D8) as u32, g.flow.script.b4d8 as u32);
            diff("G.$4F0", b(0xFF84F0) as u32, g.w.floor_only as u32);
            diff("G.$506", b(0xFF8506) as u32, g.w.no_clamp as u32);
            diff("camera mode G.$741", b(0xFF8741) as u32, g.cam.s2.mode as u32);
            diff("random number G.$14", w(0xFF8014) as u32, g.w.rng as u32);
            let p1 = &g.players[0];
            diff("P1 X", w(0xFFB274 + 8) as u32, p1.b.xi() as u16 as u32);
            diff("P1 height", w(0xFFB274 + 0xC) as u32, p1.b.yi() as u16 as u32);
            diff("P1 depth", w(0xFFB274 + 0x10) as u32, p1.b.zi() as u16 as u32);
            diff("P1 state", (w(0xFFB274 + 4) as u32) << 16 | w(0xFFB274 + 6) as u32, p1.b.state());
            if mame_objects(ram, &classes_a, &classes_b) != port_objects(g, &classes_a, &classes_b) {
                bad.push(format!(
                    "record {k}: objects MAME {:?} port {:?}",
                    mame_objects(ram, &classes_a, &classes_b),
                    port_objects(g, &classes_a, &classes_b)
                ));
            }
            if b(0xFF84E3) != 0 && seen[0] == 0 {
                seen[0] = k;
            }
        },
    );
    for (k, d) in rows.iter().enumerate().skip(1059) {
        let k = k as i32;
        if d.s2 + d.s3 + d.pal > 0 || (k < 1678 && d.s1 > 0) || d.sprites.is_some() || d.sounds.is_some() || d.cams.is_some() || d.layer.is_some() {
            bad.push(format!("record {k}: picture {d:?}"));
        }
    }
    assert!(seen[0] != 0, "the heroes were released: G.$4E3 was set");
    assert_eq!(seen[0], 1672, "G.$4E3 is set in record 1672 (D4c A1: frame 615 of the level, which starts in record 1057)");
    let first: Vec<String> = bad.iter().take(4).map(|m| m.chars().take(700).collect()).collect();
    assert!(bad.is_empty(), "{} problems, first: {:#?}", bad.len(), first);
}

/// M2-24, the stage clear tally against the recordings of `flow_tally` (re/harness/scripts/flow_tally.txt, one MAME process): the port is loaded
/// with the state of the first record where the stage clear sequence G.$4D1 is 6 (the tally task has run once) and runs without input to the record
/// before the story panel (mode 8, M2-28: the port's stub starts the next stage there). In every record the three layers, the
/// palettes, the sprite list, the sound codes, the random number, the fade, the scores and the whole struct of the task G.$735E (0x80 bytes) equal MAME:
/// one hero with every HP boundary (100 gives 20000), kills (150), the final blow, two and three heroes (the best of each panel, M.V.P. by damage with a tie,
/// WINNER! and its sparkle) and the damage ties. The end of the count is the frame 177 + the largest total / 100 + the map panel (M2-25: the slide in and
/// the blinking marker, 140 frames of it and 60 of sleep); the number of records checked per scenario is printed with `--nocapture`. The one record of
/// the panel draw where the 68000 overruns the frame (3 heroes, a tie of 2) shows the last cells of the panel undrawn in MAME: its scroll1 is not compared.
#[test]
fn clear_tally_matches_mame() {
    let Some(a) = assets() else { return };
    let mut checked = Vec::new();
    for name in ["tv-hp100", "tv-hp99", "tv-hp1", "tv-k150", "tv-k255", "tv-fb", "tv-2p-a", "tv-2p-eq", "tv-3p-a", "td-2p-12", "td-3p-tie"] {
        let Some(trace) = flow_trace_named(name) else { continue };
        let mut rd = TraceReader::open(&trace).expect("the recording opens");
        // the first record with G.$4D1 = 6, the stage clear tally
        let mut start = None;
        while let Some(f) = rd.next() {
            if f >= 0 && rd.region("ram").unwrap().data[0x84D1] == 6 {
                start = Some(f);
                break;
            }
        }
        let start = start.expect("the recording reaches the tally");
        let mut g = Game::new(a);
        g.hold_unported = true;
        let warn = g.trace_load(&rd.region("ram").unwrap().data.clone(), &rd.region("gfx").unwrap().data.clone());
        assert!(warn.iter().all(|w| w.contains("fade") || w.contains("intro")), "{name}: {warn:?}");
        let mut bad: Vec<String> = Vec::new();
        let mut n = 0;
        let mut prev_ram = rd.region("ram").unwrap().data.clone();
        loop {
            g.w.input = [0; 3];
            g.w.system = 8;
            g.w.coins = 0;
            g.tick();
            match rd.next() {
                Some(f) => assert_eq!(f - start, n + 1, "{name}: the next record"),
                None => break, // the recording ends before the story panel (k150, fb)
            }
            let (ram, gfx) = (rd.region("ram").unwrap().data.clone(), rd.region("gfx").unwrap().data.clone());
            if ram[0x84D0] == 8 {
                break; // the story panel (M2-28)
            }
            n += 1;
            let k = n;
            let d = (diff(&g, &gfx, 1).len(), diff(&g, &gfx, 2).len(), diff(&g, &gfx, 3).len(), palette_diff(&g, &gfx, 0xFFFF).len());
            // the frame overrun of the panel draw (`tally_state4`): the recording is cut in the middle of it
            let overrun = ram[0xF35E] == 6 && prev_ram[0xF35E] == 4 && g.flow.tally.sleep == 2;
            if d != (0, 0, 0, 0) && !(overrun && d == (d.0, 0, 0, 0)) {
                bad.push(format!("{name} +{k}: cells {d:?}"));
            }
            let ms: Vec<_> = mame_sprites(&ram, &gfx);
            let ps: Vec<_> = g.sprites.entries.iter().map(|e| (e.x as u16, e.y as u16, e.code, e.attr)).collect();
            if ms != ps {
                bad.push(format!("{name} +{k}: sprite list {} / {}", ms.len(), ps.len()));
            }
            if ram[0xF35E..0xF3DE] != g.flow.tally.raw.0[..0x80] {
                let at = (0..0x80).find(|&i| ram[0xF35E + i] != g.flow.tally.raw.0[i]).unwrap();
                bad.push(format!("{name} +{k}: the struct G.$735E differs at +{at:#x}: MAME {:#x} port {:#x}", ram[0xF35E + at], g.flow.tally.raw.0[at]));
            }
            for p in 0..3 {
                let o = 0xB374 + 0x180 * p;
                if ram[o..o + 4] != g.players[p].score.to_be_bytes() {
                    bad.push(format!("{name} +{k}: score of player {p}"));
                }
            }
            if ram16(&ram, 0xFF8014) != g.w.rng {
                bad.push(format!("{name} +{k}: random number"));
            }
            if ram[0x84E2] != g.flow.b4e2 {
                bad.push(format!("{name} +{k}: G.$4E2"));
            }
            if ram[0x844A] != g.fade.mask {
                bad.push(format!("{name} +{k}: fade mask"));
            }
            let (msn, psn) = (mame_sounds(&prev_ram, &ram), g.w.sounds.clone());
            if msn != psn {
                bad.push(format!("{name} +{k}: sounds MAME {msn:?} port {psn:?}"));
            }
            if let Some(t) = listed_arg_diff(&prev_ram, &ram, &g) {
                bad.push(format!("{name} +{k}: sound arguments {t}"));
            }
            prev_ram = ram;
            if bad.len() > 5 || n > 900 {
                break;
            }
        }
        assert!(bad.is_empty(), "{} problems, first: {:#?}", bad.len(), &bad[..bad.len().min(5)]);
        checked.push((name, n));
    }
    println!("clear tally records equal to MAME before the story panel: {checked:?}");
}

/// M2-26 to M2-28, the story panel (mode 8), the map card and the walk in of the next episode against the recordings of `flow_links` (re/harness/scripts/flow_links.txt):
/// the port is loaded with the record before the poke of the mode 8 (the level of Episode 1), gets the poke (the stage, mode 8) and runs without input through the
/// panel (300 to 440 frames), the stage start (0xAC4E), the card (the typed title, the blinking marker, the slide out), the walk in and the HUD slide in. In every
/// record the mode, the intro task, the handshake G.$4E2 / G.$4E3 / G.$4D5, the camera X, the three layers, the palettes and the sound codes equal MAME. Not
/// compared (the allowed differences of re/harness/allow/m228.txt): the two records of the first level frame (the original takes 2.5 video frames for it), the
/// join prompts of the absent heroes (scroll1 rows 4 and 5 of the columns 81 to 111, M2-18) and, in the long run, the rest (the whole episode 6 and 8 openings
/// are 64 frames of HUD slide: no actors). Episodes 6 and 8 run equal to the end of the recording (`re/harness/link_check.sh` lists all of them).
#[test]
fn story_panel_and_card_match_mame() {
    let Some(a) = assets() else { return };
    let mut checked = Vec::new();
    let mut failed: Vec<String> = Vec::new();
    // M2-32 part 3: episodes 3, 5 and 7 differ (see the report); `DINO_LINKS=link-ep3,link-ep5,link-ep7` runs them without failing the test.
    let names: Vec<String> = match std::env::var("DINO_LINKS") {
        Ok(v) => v.split(',').map(String::from).collect(),
        Err(_) => ["link-ep2", "link-ep4", "link-ep6", "link-ep8"].map(String::from).to_vec(),
    };
    for name in names.iter().map(String::as_str) {
        let Some(trace) = flow_trace_named(name) else { continue };
        let mut rd = TraceReader::open(&trace).expect("the recording opens");
        let (mut prev, mut start): (Option<(Vec<u8>, Vec<u8>)>, Option<(i32, Vec<u8>, Vec<u8>)>) = (None, None);
        let mut records: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        while let Some(f) = rd.next() {
            if f < 0 {
                continue;
            }
            let (ram, gfx) = (rd.region("ram").unwrap().data.clone(), rd.region("gfx").unwrap().data.clone());
            if start.is_none() && ram[0x84D0] == 8 {
                let (pr, pg) = prev.clone().expect("a record before the poke");
                start = Some((f, pr, pg));
            }
            if start.is_some() {
                records.push((ram.clone(), gfx.clone()));
            }
            prev = Some((ram, gfx));
        }
        let (_, pram, pgfx) = start.expect("the recording has a story panel");
        let level = records.iter().position(|(r, _)| r[0x84D0] != 8).expect("the level starts");
        let (fl, fe) = (level + 1, level + 2);
        let mut g = Game::new(a);
        g.hold_unported = true;
        let warn = g.trace_load(&pram, &pgfx);
        assert!(warn.iter().all(|w| w.contains("fade") || w.contains("intro")), "{name}: {warn:?}");
        // the pokes of the recording: G.$4D9, G.$4D1 = 0, G.$4D0 = 8 and the live bytes of the heroes (the recorder's jump leaves 2, the record has 1)
        g.w.stage = records[0].0[0x84D9];
        g.flow.mode = 8;
        g.flow.sub = 0;
        for (p, o) in [0xB274usize, 0xB3F4, 0xB574].into_iter().enumerate() {
            if records[0].0[o] != pram[o] {
                g.players[p].b.live = records[0].0[o];
            }
        }
        let mut bad: Vec<String> = Vec::new();
        let mut prev_ram = pram.clone();
        for (k, (ram, gfx)) in records.iter().enumerate() {
            g.w.input = [0; 3];
            g.w.system = 8;
            g.w.coins = 0;
            g.tick();
            let partial = k >= fl && k <= fe;
            let mut check = |what: &str, ok: bool| {
                if !ok {
                    bad.push(format!("{name} +{k}: {what}"));
                }
            };
            check("mode", g.flow.mode == ram[0x84D0]);
            check("sub", g.flow.sub == ram[0x84D1]);
            check("G.$4E2", g.flow.b4e2 == ram[0x84E2]);
            check("G.$4E3", g.flow.b4e3 == ram[0x84E3]);
            check("G.$4D5", g.spawn.g.b4d5 == ram[0x84D5]);
            check("camera X", g.cam.cam1_x == ram16(ram, 0xFF86C4));
            if !partial {
                check("intro state, sub", (g.flow.intro.state, g.flow.intro.sub) == (ram[0xF35E], ram[0xF35F]));
                check("layer control", g.w.layer_ctrl == ram16(ram, 0xFF8028));
                let skip = |i: usize| (81..=111).contains(&(i / 32)) && matches!(i % 32, 4 | 5);
                let d1: Vec<_> = diff(&g, gfx, 1).into_iter().filter(|(i, ..)| !skip(*i)).collect();
                check(&format!("scroll1: {} cells, first {:?}", d1.len(), d1.first()), d1.is_empty());
                check("scroll2", diff(&g, gfx, 2).is_empty());
                check("scroll3", diff(&g, gfx, 3).is_empty());
                check("palettes", palette_diff(&g, gfx, 0xFFFF).is_empty());
            }
            let (ms, ps) = (mame_sounds(&prev_ram, ram), g.w.sounds.clone());
            check(&format!("sounds MAME {ms:?} port {ps:?}"), ms == ps);
            prev_ram = ram.clone();
            if bad.len() > 5 {
                break;
            }
        }
        if !bad.is_empty() {
            eprintln!("{name}: {} problems, first: {:#?}", bad.len(), &bad[..bad.len().min(6)]);
            failed.push(name.to_string());
        }
        checked.push((name, records.len()));
    }
    println!("story panel and card records equal to MAME: {checked:?}");
    assert!(failed.is_empty(), "episodes with problems: {failed:?}");
}

/// The recording `flow_NAME.mame.trc` of a batch segment (`re/out/traces`), or None (the test skips).
fn flow_trace_named(name: &str) -> Option<String> {
    flow_trace(name)
}

/// M3-23 to M3-27, the ending (mode 0x1C) against the video clock recordings of `flow_ending` (re/harness/scripts/flow_ending.txt, `end-jack` and the party
/// recordings): the port is loaded with the first record that has the mode 0x1C (the probe's poke of the ending, the state of the Episode 1 opening
/// level) and runs without input. In every record the three layers, the palettes, the sprite list, the sound codes and the random number equal MAME, for
/// `ENDING_RECORDS` records after it (the lab corridor, the sunset and the still picture are the first 975). `ending_report` (ignored) prints the first
/// difference of each kind for any length (`DINO_ENDING` names the recording, `DINO_LAST` the last record after the start).
const ENDING_RECORDS: i32 = 5700; // the whole ending and the first records of the ranking after it

fn ending_run(a: &'static AssetPack, name: &str, last: i32, report: bool) -> Vec<String> {
    let Some(trace) = flow_trace_named(name) else { return vec![] };
    let mut rd = TraceReader::open(&trace).expect("the recording opens");
    let mut start = None;
    while let Some(f) = rd.next() {
        if f >= 0 && rd.region("ram").unwrap().data[0x84D0] == 0x1C {
            start = Some(f);
            break;
        }
    }
    let start = start.expect("the recording reaches the ending");
    // the record before the poke: load it, give the port the poke (stage 7, G.$4D7 = G.$4DB = G.$507 = 1, sub 0, mode 0x1C) and run
    let mut rd = TraceReader::open(&trace).expect("the recording opens");
    let (mut ram0, mut gfx0) = (Vec::new(), Vec::new());
    while let Some(f) = rd.next() {
        if f == start - 1 {
            (ram0, gfx0) = (rd.region("ram").unwrap().data.clone(), rd.region("gfx").unwrap().data.clone());
            break;
        }
    }
    let mut g = Game::new(a);
    g.hold_unported = true;
    let _ = g.trace_load(&ram0, &gfx0);
    g.w.stage = 7;
    (g.flow.b4d7, g.flow.b4db, g.flow.b507, g.flow.sub, g.flow.mode) = (1, 1, 1, 0, 0x1C);
    let mut bad: Vec<String> = Vec::new();
    let mut prev_ram = ram0;
    let mut last_join = (0xFFu8, 0u16, 0u8, 0u8);
    for n in 0..=last {
        g.w.input = [0; 3];
        g.w.system = 8;
        g.w.coins = 0;
        g.tick();
        let Some(f) = rd.next() else { break };
        assert_eq!(f - start, n, "{name}: the next record");
        let (ram, gfx) = (rd.region("ram").unwrap().data.clone(), rd.region("gfx").unwrap().data.clone());
        let mut d = (diff(&g, &gfx, 1).len(), diff(&g, &gfx, 2).len(), diff(&g, &gfx, 3).len(), palette_diff(&g, &gfx, 0xFFFF).len());
        if n == 0 {
            d.1 = 0; // how far the clear of scroll2 has got when the first frame ends depends on the work before it (`CLEAR2_FIRST`)
        }
        if g.flow.ending.ranking_follows && d.2 <= 4 {
            d.2 = 0; // the cut of the first frame of the ranking after the ending is 2662 to 2666 words in the six recordings (`CUT_AFTER_ENDING`)
        }
        if d != (0, 0, 0, 0) {
            bad.push(format!("{name} +{n}: cells {d:?}"));
            if report && (n < 8 || std::env::var("DINO_AT").ok().and_then(|v| v.parse::<i32>().ok()) == Some(n)) {
                for l in 1..=3u8 {
                    let v = diff(&g, &gfx, l);
                    if let (Some(first), Some(last)) = (v.first(), v.last()) {
                        println!("  layer {l}: first {first:?} last {last:?}");
                        if std::env::var_os("DINO_AT").is_some() {
                            println!("    cells {:?}", v.iter().take(12).collect::<Vec<_>>());
                        }
                    }
                }
            }
        }
        if report && n < 9 {
            println!("  +{n}: G.$0 MAME {:#x} port {:#x}, B82 MAME {:#x} port {:?}", ram16(&ram, 0xFF8000), g.w.frame, ram16(&ram, 0xFFC834 + 4), g.props.slots.iter().find(|p| p.b.k20 == 0x148).map(|p| (p.b.mode, p.b.sub)));
        }
        if report && std::env::var_os("DINO_JOIN").is_some() {
            let cur = (g.flow.join[0].state, g.players[0].w80, g.flow.b5e4, g.flow.b507);
            if cur != last_join {
                println!("  +{n}: port join state {:02X} w80 {:04X} 5E4={} 507={}", cur.0, cur.1, cur.2, cur.3);
                last_join = cur;
            }
        }
        let ms: Vec<_> = mame_sprites(&ram, &gfx);
        let ps: Vec<_> = g.sprites.entries.iter().map(|e| (e.x as u16, e.y as u16, e.code, e.attr)).collect();
        if ms != ps {
            bad.push(format!("{name} +{n}: sprite list {} / {}", ms.len(), ps.len()));
        }
        if ram16(&ram, 0xFF8014) != g.w.rng {
            bad.push(format!("{name} +{n}: random number MAME {:#x} port {:#x}", ram16(&ram, 0xFF8014), g.w.rng));
        }
        let (msn, psn) = (mame_sounds(&prev_ram, &ram), g.w.sounds.clone());
        if msn != psn {
            bad.push(format!("{name} +{n}: sounds MAME {msn:?} port {psn:?}"));
        }
        if let Some(t) = listed_arg_diff(&prev_ram, &ram, &g) {
            bad.push(format!("{name} +{n}: sound arguments {t}"));
        }
        if (ram[0x84D0], ram[0x84D1]) != (g.flow.mode, g.flow.sub) {
            bad.push(format!("{name} +{n}: mode {:#x}/{:#x} port {:#x}/{:#x}", ram[0x84D0], ram[0x84D1], g.flow.mode, g.flow.sub));
        }
        let (m, c) = (|a: u32| ram16(&ram, a), &g.cam);
        let (mc, pc) = (
            [m(0xFF86C4), m(0xFF86C8), m(0xFF86CA), m(0xFF8744), m(0xFF8748), m(0xFF87E4), m(0xFF87E8)],
            [c.cam1_x, c.cam1_y, c.cam1_y_frac, c.s2.cam_x, c.s2.cam_y, c.s3.cam_x, c.s3.cam_y],
        );
        if mc != pc {
            bad.push(format!("{name} +{n}: cameras (scroll1 X, Y, Y fraction, scroll2 X, Y, scroll3 X, Y) MAME {mc:x?} port {pc:x?}"));
        }
        prev_ram = ram;
        if !report && bad.len() > 5 {
            break;
        }
    }
    bad
}

#[test]
fn the_ending_matches_mame() {
    let Some(a) = assets() else { return };
    for name in ["end-jack", "end-hannah", "end-3p", "end-mess-jack", "end-mustapha", "end-hiscore"] {
        let bad = ending_run(a, name, ENDING_RECORDS, false);
        assert!(bad.is_empty(), "{} problems, first: {:#?}", bad.len(), &bad[..bad.len().min(5)]);
    }
}

#[test]
#[ignore]
fn ending_report() {
    let Some(a) = assets() else { return };
    let name = std::env::var("DINO_ENDING").unwrap_or("end-jack".into());
    let last: i32 = std::env::var("DINO_LAST").ok().and_then(|v| v.parse().ok()).unwrap_or(ENDING_RECORDS);
    let bad = ending_run(a, &name, last, true);
    let mut kinds: Vec<(String, usize)> = Vec::new();
    for b in &bad {
        let kind = b.split(": ").nth(1).map(|s| s.split(' ').next().unwrap_or("").to_string()).unwrap_or_default();
        match kinds.iter_mut().find(|(k, _)| *k == kind) {
            Some(e) => e.1 += 1,
            None => kinds.push((kind.clone(), 1)),
        }
        if kinds.iter().find(|(k, _)| *k == kind).unwrap().1 <= 2 || std::env::var_os("DINO_ALL").is_some() {
            println!("{b}");
        }
    }
    println!("{} differences: {kinds:?}", bad.len());
}

/// Review run (not a MAME check): Episode 8 from a direct start with the placeholder bosses, one hero who cannot die, the right key and kill all every 400
/// frames, to the ending and back to the attract loop (`cargo test -p dino-game episode_eight_reaches_the_ending -- --ignored --nocapture`).
#[test]
#[ignore]
fn episode_eight_reaches_the_ending() {
    let Some(a) = assets() else { return };
    let mut g = Game::new(a);
    g.start_stage(7, &[0]);
    let mut last = (0u8, 0u8, 0u8);
    let mut ending_at = None;
    for t in 0..200_000u32 {
        g.w.input = [crate::input::RIGHT | if t % 12 < 4 { crate::input::ATTACK } else { 0 } | if t % 180 < 8 { crate::input::JUMP } else { 0 }, 0, 0];
        g.w.system = 8;
        g.w.coins = 0;
        if t % 600 == 0 {
            g.spawn.g.kill_all = 2;
        }
        // the debug key F6 (`debug::next_area`) takes the hero to the last area, where the placeholder Fessenden is
        if g.cam.s2.area < 3 && g.flow.mode == 0xC && g.w.scripted == 0 && g.cam.s2.mode == 0 && t > 300 {
            crate::debug::next_area(&mut g);
        }
        if g.flow.mode == 0xC {
            g.players[0].b.hp = 100;
            g.players[0].b.hp_bar = 100;
        }
        g.tick();
        if g.spawn.g.kill_all == 2 {
            g.spawn.g.kill_all = 0;
        }
        if t % 3000 == 0 {
            println!("  t {t}: cam {:#x} x {:#x} mode {:#x} lock {} hp {} scripted {:#x}", g.cam.s2.cam_x, g.players[0].b.xi(), g.cam.s2.mode, g.cam.s2.lock78, g.players[0].b.hp, g.w.scripted);
        }
        let now = (g.flow.mode, g.flow.sub, g.cam.s2.area);
        if now != last && (now.0 != last.0 || t % 1000 == 0 || now.1 != last.1 && now.0 == 0x1C) {
            println!("tick {t}: mode {:#04x} sub {:#x} area {}", now.0, now.1, now.2);
        }
        last = now;
        if g.flow.mode == 0x1C && ending_at.is_none() {
            ending_at = Some(t);
        }
        if let Some(e) = ending_at {
            if g.flow.in_game == 0 {
                println!("the attract loop is back at tick {t} ({} ticks after the ending started)", t - e);
                return;
            }
        }
    }
    panic!("the ending was not reached or did not end: mode {:#x} sub {:#x}", g.flow.mode, g.flow.sub);
}

/// M4-14: every sound request of the power on run (code, d1, d2) against the ring of `trace`, per record from power on to
/// `last`. Returns the records where the requests differ, as text. `extra` sets the inputs of the run (coins, system bits).
fn sound_args_diffs(trace: &str, last: i32, input: impl Fn(i32) -> [u8; 3], mut extra: impl FnMut(i32, &mut Game)) -> Vec<(usize, Vec<(u16, u32, u32)>, Vec<(u16, u32, u32)>)> {
    let Some(a) = assets() else { return Vec::new() };
    let prev_cell: std::cell::RefCell<Vec<u8>> = std::cell::RefCell::new(Vec::new());
    let mut out = Vec::new();
    let _ = compare_power_on_with(
        a,
        trace,
        last,
        input,
        |k, g, prev| {
            // the demo stub plays none of its random numbers (as in `opening_movie_runs_like_the_original`)
            if [4668, 11239, 17773, 24344].contains(&k) {
                g.w.rng = ram16(prev, 0xFF8014);
            }
            extra(k, g);
            let mut b = prev_cell.borrow_mut();
            b.clear();
            b.extend_from_slice(prev);
        },
        |k, g, ram, _| {
            let prev = prev_cell.borrow();
            let m: Vec<(u16, u32, u32)> = mame_sound_args(&prev, ram).into_iter().filter(|e| e.0 != 0).collect();
            let p: Vec<(u16, u32, u32)> = g.w.snd_reqs.iter().filter(|r| r.code != 0).map(|r| (r.code, r.d1, r.d2)).collect();
            if m != p {
                out.push((k as usize, m, p));
            }
        },
    );
    out
}

/// The demo runs of `attract3.mame.trc` (the records of `LOOP_DEMOS`, with their demo returns): the in-level sounds there are
/// not complete yet (M10).
fn in_demo(k: usize) -> bool {
    LOOP_DEMOS.iter().any(|&(a, b)| (a..=b).contains(&k))
}

/// M4-14 (the sound lines of the front end, from power on): outside the demos every sound request of the attract loop (the boot,
/// the title 0x88, the profiles, the ranking 0x80 and 0x70, the slides 0x24 and 0x25, the movie) has the code of the recording in
/// the same frame. The d1 and d2 longs equal the recording for the title, the profiles, the ranking, the dinosaur steps (0x75)
/// and the join hunter's 0xEA. Known differences, all from register leftovers of the original: the slide music 0x24 and 0x25 are
/// queued by a script line whose d1 and d2 are what the palette routine left in the registers, and the movie handler's 0x61 (0x9C3C6,
/// `jmp 0xA63C`) takes d1 and d2 from the caller (M10).
#[test]
fn front_end_sound_args_match_mame() {
    let Some(trace) = Some("../../../re/out/traces/attract3.mame.trc").filter(|p| Path::new(p).exists()) else { return };
    let diffs = sound_args_diffs(trace, 13500, |_| [0; 3], |_, _| {});
    let outside: Vec<_> = diffs.iter().filter(|(k, _, _)| !in_demo(*k)).collect();
    let codes: std::collections::BTreeSet<u16> = outside.iter().flat_map(|(_, m, _)| m.iter().map(|e| e.0)).collect();
    assert_eq!(codes.into_iter().collect::<Vec<_>>(), vec![0x24, 0x25, 0x61], "only the known leftover codes differ");
    let records: Vec<usize> = outside.iter().map(|(k, _, _)| *k).collect();
    // (the flock's 0x61 at 12486 to 12526 equals MAME since M6-16 gave class 46 its d1 / d2)
    assert_eq!(records, [4670, 6075, 12392, 12478]);
    // the codes of the same records are equal: only d1 or d2 differ
    for (k, m, p) in &outside {
        let (mc, pc): (Vec<u16>, Vec<u16>) = (m.iter().map(|e| e.0).collect(), p.iter().map(|e| e.0).collect());
        assert_eq!(mc, pc, "record {k}: the codes differ");
    }
}

/// M4-14: the coin (0x30 with d1 = 0x70010, d2 = 0x3800000) and the start (0x31 with d1 = 0x10) of `flow_c-title` (the coin at 450
/// to 457, the start at 600 to 605) equal the recording in every request, with d1 and d2, from power on to record 621.
#[test]
fn coin_and_start_sound_args_match_mame() {
    let Some(trace) = flow_trace("c-title") else { return };
    let diffs = sound_args_diffs(&trace, 621, |_| [0; 3], |k, g| {
        g.w.coins = (449..=456).contains(&k) as u8;
        g.w.system = 8 | (599..=604).contains(&k) as u8;
    });
    assert!(diffs.is_empty(), "{diffs:?}");
}
