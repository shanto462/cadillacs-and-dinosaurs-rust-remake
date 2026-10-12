//! Terrain collision tables (`re/specs/stage/SPEC.md`, phase 1 task 1). Filled by
//! `dino-extract/src/terrain.rs`; read by `dino-game/src/game/terrain.rs`.
//!
//! The collision routines of the original (0x12CB4 and its variants) read the floor segment list
//! of the stage and area, the terrain class stored in bits 10..15 of every scroll2 tile attribute,
//! a per stage and area table that maps a class to a flag byte (0x13B0C) and one shape descriptor
//! per class (0x1370C). All of it is data and lives here.

/// One floor segment (12 bytes, lists reached through the table 0x12C30, read by 0x12FE4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FloorSegment {
    /// +0: 0 = flat floor, else a slope.
    pub slope: i16,
    /// +2: first X of the segment (also read by the pointer advance 0x12C90).
    pub x0: i16,
    /// +4: last X; the lookup takes the first segment with `x1 >= X`.
    pub x1: i16,
    /// +6 word: slope base height (its low byte +7 goes to obj+$52 for flat floors).
    pub base: i16,
    /// +8: flat floor height, or slope direction (0 = rising).
    pub height: i16,
    /// +$A: wading flag copied to obj+$53.
    pub wading: u8,
}

/// The floor segments of one stage and area.
#[derive(Clone, Debug, Default)]
pub struct FloorList {
    /// ROM address of the first segment (the original keeps G.$6AC as such a pointer).
    pub addr: u32,
    pub segs: Vec<FloorSegment>,
}

#[derive(Clone, Debug, Default)]
pub struct TerrainData {
    /// 0x12C30: floor segment lists [stage][area] (an empty list for areas that do not exist).
    /// A list ends at the first segment that reaches X 0x7FFF; the lookup (0x12FE4) scans at most
    /// 5 entries from the current pointer, and the pointer only advances while the next segment
    /// starts left of the camera.
    pub floor: Vec<Vec<FloorList>>,
    /// 0x12C40: word per stage, copied to G.$6B0 by 0x12BE4 (the height of the lower level that
    /// the street edge shapes use).
    pub floor_base: [i16; 8],
    /// 0x1370C: 64 shape descriptors of 8 words, one per terrain class (bits 10..15 of a scroll2
    /// attribute). Word 0 selects the handler (0, 4, .. 0x1C, or 0x20 = ceiling), word 1 its
    /// variant, the other words are the handler's parameters (the low byte of word 7 is the wall
    /// bit mask it stores in obj+$50).
    pub shapes: Vec<[u16; 8]>,
    /// 0x13B0C: flag byte per terrain class, by [stage][area] (stored in G.$6B2, OR-ed into obj+$50).
    pub flags: Vec<Vec<[u8; 64]>>,
    /// 0x136A6: height of a character's head above its feet for the ceiling test (0x1361E).
    pub head: [i16; 4],
    /// 0x638C: X offsets from the camera for a player that drops in (0x62D6), by
    /// (players - 1) * 4 + 0 / 1 / 2 (the original adds the byte offset (players - 1) * 8 + 0 / 2 / 4);
    /// in the vertical scene the first three words are the X by player.
    pub drop_x: [i16; 12],
    /// 0x63A4: Z offset of a dropping player by player index (also the Z in the vertical scene).
    pub drop_z: [i16; 3],
    /// 0x12C50: the map that 0x12BE4 draws at 0x91C000 / 0x91C400 in the vertical (elevator)
    /// scene: 32 columns x 16 rows of (code, attribute) cells, cell index = column * 16 + row,
    /// the original's byte offset / 4. Read by 0x136E4.
    pub scene_map: Vec<[u16; 2]>,
}
