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

/// The title: the green background blocks (0x98B86 with the lists 0x9A5E6 and 0x9A6E6), INSERT COIN
/// (0xBB6), the two small lines at title frame 30 and the licence block at frame 360.
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
    // title frame 30: 0x97F64
    g.text_bytes(rom(&g, 0x97F64), true);
    // The only other thing the original draws at that frame is the Capcom sign, 3 x 16 cells of
    // attribute 0x0A written by 0x110C (T1.2); everything else is the text.
    let sign = diff(&g, &images[2], 1);
    assert_eq!(sign.len(), 48, "title lines, record 349");
    assert!(sign.iter().all(|&(_, p, m)| p == (0x20, 6) && m.1 == 0x0A));
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
