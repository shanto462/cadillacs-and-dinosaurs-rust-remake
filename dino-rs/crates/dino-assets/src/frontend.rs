//! Front end data: the scene picture pages (scroll1, scroll2), the scene palette sets and the
//! ROM tables of the attract, select, continue and story scenes. Owner: flow / HUD port.
//! Spec: `plan/game/design/D4d-front-end-and-story.md` (appendix B lists the tables).
//!
//! Pictures are plain data in the original: a "page" is a list of block ids and each block is a
//! small block of cells (scroll1 4x4, scroll2 2x2). The scroll3 pages are the 64 cell screens of
//! `StageData::scroll3_screen_cells` (table 0x177288, the same table the stages use).

use crate::stage::Cell;
use crate::text::TextData;

/// Scroll1 pages: block id lists at 0x138C48 + 0x80 * page (64 words), pages 0..7.
pub const SCROLL1_PAGE_COUNT: u32 = 7;
/// Scroll1 blocks: 0x15CFE8 + 0x40 * id (16 cells), ids 0..98 (the highest id the pages use is 97).
pub const SCROLL1_BLOCK_COUNT: u32 = 98;
/// Scroll2 scene pages: block id lists at 0x145DC8 + 0x80 * page (64 words), pages 0..33.
pub const SCROLL2_PAGE_COUNT: u32 = 33;
/// Scroll2 blocks: 0x175378 + 0x10 * id (4 cells), ids 0..496 (the highest id the pages use is 495).
pub const SCROLL2_BLOCK_COUNT: u32 = 496;
/// Scene palette sets of 0x89E6 (table 0x8A5E, 8 bytes each).
pub const SCENE_PALETTE_SETS: u32 = 6;

/// The palettes one scene set loads (routine 0x89E6): the OBJ bank 0x8E4C, the scroll2 bank
/// 0x8AAE, the scroll3 bank 0x8C20 and the priority masks 0x8D92. The scroll1 bank is the fixed
/// one (`StageData::scroll1_palettes`). All words have brightness 0 as in the ROM.
#[derive(Clone, Debug, Default)]
pub struct ScenePalettes {
    pub obj: Vec<u16>,
    pub scroll2: Vec<u16>,
    pub scroll3: Vec<u16>,
    pub priority_masks: [u16; 4],
}

#[derive(Clone, Debug, Default)]
pub struct FrontendData {
    /// Byte blobs of the front end tables, same format and helpers as `AssetPack::text`
    /// (`FRONT_TABLES` lists what is copied).
    pub text: TextData,
    /// Scroll1 page block ids (64 per page, order of the loader 0x20454: 8 columns of 8 blocks).
    pub scroll1_pages: Vec<[u16; 64]>,
    /// Scroll1 blocks: 4x4 cells in the order the loader copies them (4 cells per column, left
    /// column first).
    pub scroll1_blocks: Vec<[Cell; 16]>,
    /// Scroll2 page block ids (64 per page, order of the loader 0x204CA).
    pub scroll2_pages: Vec<[u16; 64]>,
    /// Scroll2 blocks: 2x2 cells, 2 cells per column, left column first.
    pub scroll2_blocks: Vec<[Cell; 4]>,
    /// The 6 scene palette sets.
    pub scene_palettes: Vec<ScenePalettes>,
}

/// ROM tables of the front end scenes (appendix B of D4d), copied as byte blobs by their ROM
/// address into `FrontendData::text`: (address, length in bytes, what it is). Lengths come from
/// the next label; the game panics with the address when it reads outside a blob, so a table
/// that is too short is found at once when a scene uses it.
pub const FRONT_TABLES: &[(u32, u32, &str)] = &[
    (0x113C, 0x60, "Capcom sign tile codes, 16 columns of 3 cells (0x110C)"),
    (0x83EB, 0x415, "HUD credit line strings and the glyph blocks of the join prompts: INSERT COIN, JOIN-IN, blanks (0x82E8, 0x6E04, 0x80A2 .. 0x8196)"),
    (0x8461A, 0x2F6, "portrait palette words, portrait list pointers, text id pointers (0x84736)"),
    (0xA33C2, 0x292A, "portrait code lists, attribute lists and the tall font texts (0x84736, 0x84F72)"),
    (0x8B7EA, 0x31A, "staff roll lines (ending)"),
    (0x97A20, 0x200, "press start background block lists (0x978EE)"),
    (0x97F4B, 0x165, "title strings (0x9A3CE)"),
    (0x988A4, 0x1C, "INSERT COIN and FREE PLAY strings (0x98874)"),
    (0x98BD6, 0x41A, "story slide block lists and texts (0x988C0)"),
    (0x9A13A, 0x234, "demo input addresses, rows and stream pointer tables (0x99F10, 0x9A146)"),
    (0x9A5E6, 0x200, "title background block lists (0x9A3CE)"),
    (0x9A8E4, 0x4A, "profile timeline and handler offsets (0x9A7E6)"),
    (0x9ABF6, 0x54, "profile labels (0x9A950)"),
    (0x9B0F8, 0x24A, "profile card texts (0x9A950)"),
    (0x92158, 0x28, "profile card board tile codes, 4 words per card and variant 4 (0x91FC8)"),
    (0xC5952, 0x20, "profile card board palette (0x921AE)"),
    (0x6A724, 0x10, "profile card portrait palette pointers (0x6A6F6)"),
    (0xBB252, 0x20, "profile card portrait palette 0 (0x6A6F6)"),
    (0xBB292, 0x20, "profile card portrait palette 2 (0x6A6F6)"),
    (0xBB3B2, 0x20, "profile card portrait palette 1 (0x6A6F6)"),
    (0xBB3F2, 0x20, "profile card portrait palette 3 (0x6A6F6)"),
    (0x9C002, 0x48, "movie timeline tables (0x9BF50)"),
    (0x9C280, 0x1C, "movie tile block lists (0x9C21C)"),
    (0x9C69C, 0x2C, "movie logo palette offsets (0x9C5D8)"),
    (0x9C6C8, 0xC0, "movie tables (0x9C5D8)"),
    (0xBFFD2, 0x122, "movie logo palettes for scroll2 (0xBCA52 + 0x3400, 0x9C61A)"),
    (0xC5452, 0x42, "movie logo palettes for scroll3 (0xC2252 + 0x3000, 0x9C644)"),
    (0x86DD8, 0x1830, "caption banner tables and texts (pool B class 66, 0x86D76 and the movie's 0x86D2E)"),
    (0x9CAA0, 0x526, "movie texts (0x9C04A)"),
    (0x9D038, 0x10, "coin mode ratios (0x9CFC6)"),
    (0x9D666, 0x04, "rectangle the credit text clears: column, row, width, height (0x9D102, 0xF94)"),
    (0x9D984, 0x374, "credit text and prompt strings, to the end of the last one (0x9D66A, 0x9D752, 0x9D8EE)"),
    (0x9DDAC, 0x5E0, "boot text tables, region texts and palettes (0x9DCF6)"),
    (0xAEFC, 0x80, "select object records (0xAD5E) and the timer digit table (0xAF0E)"),
    (0xB07E, 0x28, "select tables (0xB06A)"),
    (0xB10A, 0x18, "select tables (0xB122)"),
    (0xB24A, 0x120, "select position map, pans, player objects, panel origins, face lists and their attributes (0xB25E)"),
    (0xB396, 0xBA, "select hero names, labels, stat diamonds (0xB25E)"),
    (0xB47C, 0x200, "select rating word lists (0xB25E)"),
    (0xB70E, 0x20, "select pick palette targets and sources by cursor position (0xB6A2)"),
    (0xB78E, 0x80, "select pick flash palette lists (0xB6A2)"),
    (0xB8DE, 0x3C, "select end sequence palette targets (0xB810)"),
    (0xBB2D2, 0x380, "select player tag palettes (pool B class 68, 0x888B2) and hero palettes (0xB6A2, 0xB810)"),
    (0xBBA52, 0x20, "select frame palette (pool B class 17, 0x7B554)"),
    (0xBC052, 0x200, "select pick flash palettes (0xB6A2)"),
    (0xBFE52, 0x80, "select end sequence hero palettes (0xB810)"),
    (0xBFF92, 0x40, "select hero palettes (0xB6A2, 0xB810)"),
    (0x8885E, 0x8, "select golden man X positions (pool B class 67, 0x8881A)"),
    (0x88872, 0x10, "select golden man frame pointers (pool B class 67)"),
    (0x8890A, 0x18, "select tag palette lists and targets (pool B class 68)"),
    (0x88942, 0xE, "select tag record offsets and X by cursor position (pool B class 68)"),
    (0x889C4, 0x80, "select tag palette cycles (pool B class 68)"),
    (0x88A54, 0xC, "select tag frame pointers (pool B class 68)"),
    (0x7B85E, 0x10, "select hero picture animation tables by position (pool B class 19, 2 player layout)"),
    (0xBAAE, 0x114, "tally tables: the panel origins by player count, the panel positions by step (0xBA76, 0xBAD2)"),
    (0xBD72, 0x0B, "tally string VITALITY with its attribute word (0xBC28)"),
    (0xBEA0, 0x0A, "tally string ENEMIES with its attribute word (0xBD7E)"),
    (0xBF98, 0x0D, "tally string FINAL BLOW with its attribute word (0xBEAA)"),
    (0xC0E6, 0x08, "tally string M.V.P. with its attribute word (0xBFA6)"),
    (0xC2C2, 0x1C, "tally WINNER! label tile pairs, 7 columns (0xC2A6)"),
    (0xC2FA, 0x28, "tally TOTAL label tile pairs, 10 columns (0xC2DE)"),
    (0x84612, 0x08, "panel portrait by hero id and the winner flag (0x84604)"),
    (0xF60, 0x38, "glyph codes of the letters A to Z for the string printer 0xF1C"),
    (0x7EB18, 0x1C, "tally number label tile pairs for full vitality (pool B class 25, 0x7EACE)"),
    (0x7EB56, 0x52, "tally number sprite list templates and digit tiles (pool B class 25, 0x7E97C, 0x7EA56, 0x7EACE)"),
    (0xB8612, 0x40, "tally number flash palettes (pool C class 22, 0x95840)"),
    (0xBC252, 0x400, "map panel palettes, 32 of 16 words (0xCD1A, 0xCB7A)"),
    (0xCACE, 0xAC, "map panel marker lists: pointers by stage and the (palette slot, palette, palette) lists (0xCAA8)"),
    (0xCD6C, 0xF0, "map panel palette lists by stage (0xCD1A)"),
    (0xCEC4, 0x5F0, "map panel tile codes and attributes of the frame and the picture (0xCE5C)"),
    (0xD4DC, 0x1C8, "map panel extra tile codes and attributes for the stages 6 and 7 (0xD4B4)"),
    (0xD6CE, 0x90, "map panel extra tile codes and attributes for the stage 7 (0xD6A4)"),
    (0xD780, 0xE0, "map panel name plate pointers by stage and the 12 tile codes of each (0xD75E)"),
    (0xC6D2, 0x34E, "intro title pointer table and episode titles (0xC6A8)"),
    (0xD94E, 0x654, "story panel picture lists (0xD898)"),
    (0xE42A, 0x4D6, "story panel texts (0xD898)"),
    (0x153CA, 0x0A, "ranking row offsets (0x15594)"),
    (0x8CA2E, 0x90, "ranking cloud start list and cloud heights (0x8C982)"),
    (0x96B34, 0x0C, "ranking cloud speeds (0x96A7E)"),
    (0x15508, 0x51, "ranking label block (0x1516C)"),
    (0x156A8, 0x48, "ranking face tiles (0x15594)"),
    (0x15768, 0x100, "ranking font: glyph codes by character and glyph widths (0x15748)"),
    (0x15F34, 0x20, "name entry glyph map (0x15E3A)"),
    (0x15FF4, 0xF0, "name entry glyph tiles (0x15FB4)"),
    (0x161B2, 0x28, "name entry profanity lists (0x1616C)"),
    (0xC869A, 0x184C, "demo input streams (0x9A146)"),
    (0x222B6, 0x20, "demo start event index per stage and area, 4 bytes per stage (0x2228A)"),
];

impl FrontendData {
    /// Block ids of scroll1 page `page` (the `d5` of the loader 0x20454).
    pub fn scroll1_page(&self, page: usize) -> &[u16; 64] {
        &self.scroll1_pages[page]
    }
    pub fn scroll2_page(&self, page: usize) -> &[u16; 64] {
        &self.scroll2_pages[page]
    }
}
