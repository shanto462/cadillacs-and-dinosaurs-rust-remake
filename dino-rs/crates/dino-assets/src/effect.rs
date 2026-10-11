//! Data for the draw paths and the effect objects (pool C). Formats: `re/specs/anim/SPEC.md`
//! sections 2.6, 3.1 (8-byte records), 3.4 (raw lists, shadow pass) and the effect notes in
//! `dino-game/src/game/effect.rs`.
//!
//! Effect frames are ordinary 10-byte frame records, but they are stored here (not in
//! `SpriteData::frames`) and their ids carry the [`EFX_FRAME`] tag, so the draw code can tell them
//! apart. Raw-list frames (8-byte records, drawn by 0x147FE when obj+$25 bit 6 is set) carry the
//! [`RAW_FRAME`] tag.

use crate::sprites::{Frame, FrameId};
use std::collections::HashMap;

/// Tag bit of a frame id that lives in [`EffectData::frames`].
pub const EFX_FRAME: FrameId = 0x8000_0000;
/// Tag bit of a frame id that lives in [`EffectData::raw_frames`].
pub const RAW_FRAME: FrameId = 0x4000_0000;
const INDEX_MASK: FrameId = 0x3FFF_FFFF;

/// One sprite of a raw sprite list (0x147FE: dx, dy, attr, code) or of a ground list
/// (shadow 0x14F98: attr, dx, code with dy = 0; splash 0x14C9C: attr, dx, dy, code).
#[derive(Clone, Copy, Debug, Default)]
pub struct RawSprite {
    pub dx: i16,
    pub dy: i16,
    pub attr: u16,
    pub code: u16,
}

/// An 8-byte frame record: `u32 list pointer, u8 $2C, u8 $2D, u8 $2E, u8 duration | 0x80 last`.
/// The list is `u16 count, u16 n, n x (dx, dy, attr, code)`.
#[derive(Clone, Debug)]
pub struct RawFrame {
    pub rom_addr: u32,
    pub list_addr: u32,
    pub sprites: Vec<RawSprite>,
    pub param_a: u8,
    pub param_b: u8,
    pub kind: u8,
    pub duration: u8,
    pub last: bool,
    /// Tagged id ([`RAW_FRAME`]) of the following record (link target when `last`).
    pub next: FrameId,
}

#[derive(Default)]
pub struct EffectData {
    /// 10-byte frames used by effect objects; id = `EFX_FRAME | index`.
    pub frames: Vec<Frame>,
    pub frame_by_addr: HashMap<u32, FrameId>,
    /// Effect animation tables (the value an effect stores in obj+$40): first frame per id.
    pub tables: HashMap<u32, Vec<Option<FrameId>>>,
    /// 8-byte raw-list frames; id = `RAW_FRAME | index`.
    pub raw_frames: Vec<RawFrame>,
    pub raw_by_addr: HashMap<u32, FrameId>,
    /// Dynamic shadow lists from the table at 0x14F98, by shadow type (obj+$4E >> 4), 8 entries.
    pub shadows: Vec<Vec<RawSprite>>,
    /// Water splash lists from the table at 0x14C9C: `[type][phase]`, 8 types x 4 phases.
    pub splashes: Vec<[Vec<RawSprite>; 4]>,
    /// Position jitter of the spark spawner 0x92450 (table 0x9248A), 8 (dx, dy) pairs.
    pub spark_jitter: [(i16, i16); 8],
}

impl EffectData {
    pub fn is_effect_frame(id: FrameId) -> bool {
        id & EFX_FRAME != 0
    }

    pub fn is_raw_frame(id: FrameId) -> bool {
        id & EFX_FRAME == 0 && id & RAW_FRAME != 0
    }

    pub fn frame(&self, id: FrameId) -> &Frame {
        &self.frames[(id & INDEX_MASK) as usize]
    }

    pub fn raw(&self, id: FrameId) -> &RawFrame {
        &self.raw_frames[(id & INDEX_MASK) as usize]
    }

    /// First frame of animation `id` of effect table `table` (what 0x15E4 starts).
    pub fn anim(&self, table: u32, id: u16) -> FrameId {
        let t = self.tables.get(&table).unwrap_or_else(|| panic!("effect table {table:#x} was not extracted"));
        t.get(id as usize).copied().flatten().unwrap_or_else(|| panic!("effect anim {id} of table {table:#x} is unused"))
    }

    /// Frame record at a ROM address (what 0x160A loads directly).
    pub fn frame_at(&self, addr: u32) -> FrameId {
        *self.frame_by_addr.get(&addr).unwrap_or_else(|| panic!("no effect frame at {addr:#x}"))
    }

    /// Raw-list frame record at a ROM address (what 0x16C6 loads).
    pub fn raw_at(&self, addr: u32) -> FrameId {
        *self.raw_by_addr.get(&addr).unwrap_or_else(|| panic!("no raw frame at {addr:#x}"))
    }
}
