//! Stage data: background maps, palettes, camera events and per-area constants.
//! Formats are documented in `re/specs/stage/SPEC.md`.
//!
//! Screen and block data are kept as flat word arrays per stage, because the original streams
//! them with pointer arithmetic (it can read past one screen into the next); flat arrays let the
//! ported code use exactly the same index math.

/// One tilemap cell: tile code and attribute (bits 0-4 palette, 5 X flip, 6 Y flip,
/// 7-8 priority group, 10-15 game data in scroll2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cell {
    pub code: u16,
    pub attr: u16,
}

/// Area header (10 bytes in ROM), same layout for scroll2 and scroll3.
#[derive(Clone, Debug, Default)]
pub struct AreaMap {
    pub rom_addr: u32,
    /// ROM address of the screen array (the long at +2 of the header; the controller's +$42).
    pub screens_addr: u32,
    /// Width and height in screens of 256x256 px (0 = no map / keep the previous one).
    pub w: u8,
    pub h: u8,
    /// Screen ids, column-major, bottom row first: index = sx * h + sy. Followed by a few more ROM
    /// words: the streaming code reads past the end of the array (rows above the map).
    pub screens: Vec<u16>,
    pub init_x: u16,
    pub init_y: u16,
}

/// 16-byte camera event record (table 0x21B2A).
#[derive(Clone, Copy, Debug, Default)]
pub struct CameraEvent {
    pub x: u16,
    pub y: u16,
    pub push: u16,
    pub up: u16,
    pub down: u16,
    pub area: u8,
    pub mode: u8,
    pub vfollow: u8,
    pub flag70: u8,
    pub next_mode: u8,
    pub by_y: u8,
}

#[derive(Clone, Debug, Default)]
pub struct Area {
    pub scroll2: AreaMap,
    pub scroll3: AreaMap,
    /// 32 palettes x 16 colours (raw ROM words, brightness nibble 0).
    pub scroll2_palettes: Vec<u16>,
    pub scroll3_palettes: Vec<u16>,
    /// Parallax: scroll3 X = camera X >> shift (table 0x22DE4).
    pub scroll3_shift: u8,
    /// Scroll3 follows the camera vertically (table 0x22E54).
    pub scroll3_yfollow: bool,
    /// Slope shift for camera mode 0x0B (table 0x20F58).
    pub slope_shift: u8,
    /// CPS-B priority masks for tile groups 0..3 (tables 0x8DEC / 0x8DCC).
    pub priority_masks: [u16; 4],
}

#[derive(Clone, Debug, Default)]
pub struct Stage {
    /// Scroll2 screen data: 64 block numbers per screen id, column-major, top row first.
    pub scroll2_screen_words: Vec<u16>,
    /// Scroll2 blocks: 4 cells each, column-major [(c0,r0), (c0,r1), (c1,r0), (c1,r1)].
    pub scroll2_blocks: Vec<[Cell; 4]>,
    pub camera_events: Vec<CameraEvent>,
    /// ROM address of `camera_events[0]` (the scroll2 controller's +$56 points into this table).
    pub camera_events_addr: u32,
    /// Stage 2 only: the stage's list of the standard event table 0x21B2A (the one 0x2099A reads; `camera_events` of stage 2 is the
    /// controller's own list 0x22B9E, read by 0x2251C). The demo's start of the controller (0x2228A) loads its first event with 0x2099A,
    /// so +$56 points into this list and the stage 2 controller (0x224A4) reads its next events here too. Empty for the other stages
    /// (their `camera_events` is this list).
    pub std_events: Vec<CameraEvent>,
    /// ROM address of `std_events[0]`.
    pub std_events_addr: u32,
    pub areas: Vec<Area>,
    /// Stage 2 only: the scroll2 map header 0x22AFA that its own controller (0x22398, init 0x22404) loads instead of the one of the
    /// table 0x21580 that the standard controller (and the demo's area setup) uses. The screen arrays are the same 60 screens and
    /// the same words up to index 76; the word 77 (the screen above the last column the map wraps at, 0x4C00) differs.
    pub controller_map: Option<AreaMap>,
}

#[derive(Clone, Debug, Default)]
pub struct StageData {
    pub stages: Vec<Stage>,
    /// Scroll3 screens (all stages share one table at 0x177288): 64 cells per screen id,
    /// column-major, top row first.
    pub scroll3_screen_cells: Vec<Cell>,
    /// The fixed scroll1 (text/HUD) palette bank at 0xBBA52.
    pub scroll1_palettes: Vec<u16>,
}

impl StageData {
    pub fn area(&self, stage: u8, area: u8) -> &Area {
        &self.stages[stage as usize].areas[area as usize]
    }
}
