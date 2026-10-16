//! Front end data: scene picture pages and blocks, scene palette sets and the ROM tables of the
//! attract and story scenes (owner: flow / HUD port). Formats: `dino_assets::frontend`.
//!
//! The loaders that read these tables are 0x20454 (scroll1 page), 0x204CA (scroll2 page) and
//! 0x20592 (scroll3 page, already in `StageData::scroll3_screen_cells`); the palette sets are
//! loaded by 0x89E6.

use crate::rom::Rom;
use dino_assets::frontend::*;
use dino_assets::stage::Cell;
use dino_assets::text::TextData;

/// Scroll1 page block id lists (0x20454 reads `0x138C48 + 0x80 * page`).
const SCROLL1_PAGES: u32 = 0x138C48;
/// Scroll1 blocks (0x2047C: `0x15CFE8 + 0x40 * id`).
const SCROLL1_BLOCKS: u32 = 0x15CFE8;
/// Scroll2 scene page block id lists (0x204CA reads `0x145DC8 + 0x80 * page`).
const SCROLL2_PAGES: u32 = 0x145DC8;
/// Scroll2 scene blocks (0x204F2: `0x175378 + 0x10 * id`).
const SCROLL2_BLOCKS: u32 = 0x175378;
/// 0x89E6: the (stage, area) pairs of each scene set, 8 bytes per set: OBJ, scroll2, scroll3, priority.
const SCENE_SETS: u32 = 0x8A5E;
/// 0x8E4C: long pointer to 32 OBJ palette numbers per stage * 16 + area * 4; the palettes are at 0xB7A52.
const OBJ_LISTS: u32 = 0x8E8E;
const OBJ_BANK: u32 = 0xB7A52;
/// 0x8AAE and 0x8C20: long pointers to 512 palette words per stage * 16 + area * 4.
const SCROLL2_PALETTES: u32 = 0x8AE0;
const SCROLL3_PALETTES: u32 = 0x8C52;
/// 0x8D92: priority masks, 8 bytes per stage (0x8DEC) or, for stage 6, per area (0x8DCC).
const PRIORITY_BY_STAGE: u32 = 0x8DEC;
const PRIORITY_STAGE6_BY_AREA: u32 = 0x8DCC;

fn cell(rom: &Rom, a: u32) -> Cell {
    Cell { code: rom.u16(a), attr: rom.u16(a + 2) }
}

fn page(rom: &Rom, base: u32, n: u32) -> [u16; 64] {
    let mut p = [0u16; 64];
    for (i, v) in p.iter_mut().enumerate() {
        *v = rom.u16(base + n * 0x80 + 2 * i as u32);
    }
    p
}

fn words(rom: &Rom, a: u32, n: u32) -> Vec<u16> {
    (0..n).map(|i| rom.u16(a + 2 * i)).collect()
}

/// One scene palette set: the same table lookups as 0x8E4C, 0x8AAE, 0x8C20 and 0x8D92 with the
/// (stage, area) pairs of the set (the code sets the stage and area bytes before each call).
fn scene_palettes(rom: &Rom, set: u32) -> ScenePalettes {
    let pair = |k: u32| (rom.u8(SCENE_SETS + set * 8 + 2 * k) as u32, rom.u8(SCENE_SETS + set * 8 + 2 * k + 1) as u32);
    let (os, oa) = pair(0);
    let list = rom.u32(OBJ_LISTS + os * 16 + oa * 4);
    let mut obj = Vec::with_capacity(512);
    for slot in 0..32 {
        let n = rom.u16(list + 2 * slot) as u32;
        obj.extend(words(rom, OBJ_BANK + n * 32, 16));
    }
    let (s2s, s2a) = pair(1);
    let (s3s, s3a) = pair(2);
    let (ps, pa) = pair(3);
    let prio = if ps == 6 { PRIORITY_STAGE6_BY_AREA + pa * 8 } else { PRIORITY_BY_STAGE + ps * 8 };
    ScenePalettes {
        obj,
        scroll2: words(rom, rom.u32(SCROLL2_PALETTES + s2s * 16 + s2a * 4), 512),
        scroll3: words(rom, rom.u32(SCROLL3_PALETTES + s3s * 16 + s3a * 4), 512),
        priority_masks: [rom.u16(prio), rom.u16(prio + 2), rom.u16(prio + 4), rom.u16(prio + 6)],
    }
}

pub fn extract(rom: &Rom) -> FrontendData {
    // Layout pointer tables used by join prompts (0x8050..0x83D4).
    const TABLES: &[(u32, u32, &str)] = &[
        (0x7F48, 0x3E, "continue HUD origins and digit tiles"),
        (0xF026, 0x22, "continue mouth timing"),
        (0xF072, 12, "continue mouth tile pointers"),
        (0xF0A0, 8, "continue eye tile pointers"),
        (0xF194, 0x28A, "continue taunt strings and offsets"),
        (0xF544, 22, "continue background flash periods"),
        (0xF57C, 0x57C, "continue digit tiles, speech frame and face tiles"),
        (0x7232, 12, "join cursor inverse and hero/class pairs"),
        (0x7640, 0x1CC, "join three-player rating strings and origins"),
        (0x785C, 0x288, "join two-player rating strings and origins"),
        (0x7B2C, 0x9C, "join face attributes, origins and tiles"),
        (0x7BF4, 0x168, "join name strings and origins"),
        (0x7E20, 0x50, "join countdown origins and big digits"),
        (0x7FB6, 0x14, "join strip clear rectangles"),
        (0x803C, 0x14, "join countdown clear rectangles"),
        (0x82D4, 0x14, "join arrow origins"),
        (0x80F4, 8, "two-player big prompt origins"),
        (0x8242, 12, "three-player HUD prompt origins"),
        (0x8292, 8, "two-player HUD prompt origins"),
        (0x83D6, 0x24, "HUD credit line origins and first byte string (attribute at 0x83EA)"),
    ];
    FrontendData {
        text: TextData { blobs: FRONT_TABLES.iter().chain(TABLES).map(|&(a, n, _)| (a, (a..a + n).map(|x| rom.u8(x)).collect())).collect() },
        scroll1_pages: (0..SCROLL1_PAGE_COUNT).map(|n| page(rom, SCROLL1_PAGES, n)).collect(),
        scroll1_blocks: (0..SCROLL1_BLOCK_COUNT)
            .map(|id| {
                let mut b = [Cell::default(); 16];
                for (i, c) in b.iter_mut().enumerate() {
                    *c = cell(rom, SCROLL1_BLOCKS + id * 0x40 + 4 * i as u32);
                }
                b
            })
            .collect(),
        scroll2_pages: (0..SCROLL2_PAGE_COUNT).map(|n| page(rom, SCROLL2_PAGES, n)).collect(),
        scroll2_blocks: (0..SCROLL2_BLOCK_COUNT)
            .map(|id| {
                let mut b = [Cell::default(); 4];
                for (i, c) in b.iter_mut().enumerate() {
                    *c = cell(rom, SCROLL2_BLOCKS + id * 0x10 + 4 * i as u32);
                }
                b
            })
            .collect(),
        scene_palettes: (0..SCENE_PALETTE_SETS).map(|s| scene_palettes(rom, s)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn rom() -> Option<Rom> {
        let path = Path::new("../../../dino.zip");
        path.exists().then(|| Rom::load(path).unwrap())
    }

    /// Needs the user's ROM at `../dino.zip` (skipped otherwise). The page tables have the sizes of
    /// the picture data, every block id a page uses exists, and the last block of each table is the
    /// last one a page uses (so the extents are not too long by accident).
    #[test]
    fn page_tables_cover_every_block_a_page_uses() {
        let Some(rom) = rom() else { return };
        let f = extract(&rom);
        assert_eq!(f.scroll1_pages.len(), 7);
        assert_eq!(f.scroll2_pages.len(), 33);
        let max1 = f.scroll1_pages.iter().flatten().copied().max().unwrap();
        let max2 = f.scroll2_pages.iter().flatten().copied().max().unwrap();
        assert_eq!(max1 as usize + 1, f.scroll1_blocks.len());
        assert_eq!(max2 as usize + 1, f.scroll2_blocks.len());
        // Page 0 of both is the blank picture (block 0 everywhere).
        assert!(f.scroll1_pages[0].iter().all(|&b| b == 0));
        assert!(f.scroll2_pages[0].iter().all(|&b| b == 0));
    }

    /// The scene palette sets: 512 words per bank, brightness nibble 0 as in the ROM, and set 5
    /// (all pairs (0, 0)) equals the palettes of stage 0 area 0.
    #[test]
    fn scene_palette_sets_have_full_banks() {
        let Some(rom) = rom() else { return };
        let f = extract(&rom);
        assert_eq!(f.scene_palettes.len(), 6);
        for s in &f.scene_palettes {
            assert_eq!((s.obj.len(), s.scroll2.len(), s.scroll3.len()), (512, 512, 512));
            assert!(s.scroll2.iter().chain(&s.scroll3).chain(&s.obj).all(|&w| w & 0xF000 == 0));
        }
        let st = crate::stage::extract(&rom);
        assert_eq!(f.scene_palettes[5].scroll2, st.area(0, 0).scroll2_palettes);
        assert_eq!(f.scene_palettes[5].scroll3, st.area(0, 0).scroll3_palettes);
        assert_eq!(f.scene_palettes[5].priority_masks, st.area(0, 0).priority_masks);
    }

    /// The boot, title and coin strings end with their marker inside the copied table, so a scene
    /// that prints them never reads past the blob.
    #[test]
    fn string_tables_end_inside_their_blobs() {
        let Some(rom) = rom() else { return };
        let t = extract(&rom).text;
        // Boot screens (3 region texts of each kind; the offset tables are word offsets).
        for table in [0x9DDACu32, 0x9DDB2] {
            for region in 0..3 {
                let s = t.slice_from((table as i32 + t.i16(table, 2 * region) as i32) as u32);
                assert!(s.contains(&b'@'), "boot text {table:#x} region {region}");
            }
        }
        // INSERT COIN and FREE PLAY (0x98874): '@' ended byte strings.
        assert!(t.slice_from(0x988A4).contains(&b'@'));
        assert!(t.slice_from(0x988B3).contains(&b'@'));
        // The tall font texts (0x84736 text ids): each ends with 0x4040 before the blob ends.
        for id in 0..68 {
            let mut a = t.u32(0x84800, 4 * id);
            let mut ended = false;
            while !ended {
                a += 4; // column, row, attribute
                loop {
                    let w = t.u16(a, 0);
                    a += 2;
                    if w == 0x4040 {
                        ended = true;
                        break;
                    }
                    if w == 0x2F2F {
                        break;
                    }
                }
            }
        }
    }
}
