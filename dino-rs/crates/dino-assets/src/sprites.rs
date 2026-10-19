//! Sprite data: compositions (how 16x16 tiles form one picture), frame records (what to show,
//! for how long, with which hit boxes), animation tables, hitbox and attack banks.
//! Formats are documented in `re/specs/anim/SPEC.md`.

use std::collections::HashMap;

/// One hardware-style sprite of a composition: a tile (or a block of tiles) at an offset
/// from the object's feet point. `attr` keeps the original bits:
/// 0-4 palette, 5 X flip, 6 Y flip, 8-11 block width-1, 12-15 block height-1.
#[derive(Clone, Copy, Debug)]
pub struct SpriteEntry {
    pub dx: i16,
    pub dy: i16,
    pub code: u16,
    pub attr: u16,
}

/// A picture made of sprite entries, listed head to feet (the wading clip drops the last ones).
/// The first entry has the highest priority (drawn on top).
#[derive(Clone, Debug, Default)]
pub struct Composition {
    pub rom_addr: u32,
    /// "Alt" format: every entry has its own attr (block sprites possible).
    /// Normal format: one shared attr, always 1x1 tiles.
    pub alt: bool,
    pub entries: Vec<SpriteEntry>,
}

/// Part of a compound frame: a composition drawn at an extra offset.
#[derive(Clone, Copy, Debug)]
pub struct CompoundPart {
    pub dx: i16,
    /// Added to the object's height (+$0C), so positive moves the part up.
    pub dy: i16,
    pub comp: u16,
    pub clip_flag: u16,
}

/// What a frame draws.
#[derive(Clone, Debug)]
pub enum FrameDraw {
    /// Main composition, then an optional overlay (drawn after, so below).
    Comp { main: u16, overlay: Option<u16> },
    /// Several compositions with per-part offsets (w0 bit 15 set).
    Compound(Vec<CompoundPart>),
}

/// Identifies a frame inside [`SpriteData::frames`].
pub type FrameId = u32;

/// One frame record. Field names follow the object fields they are copied into.
#[derive(Clone, Debug)]
pub struct Frame {
    pub rom_addr: u32,
    /// Record size in bytes: 10 (enemies, most objects) or 12 (players).
    pub stride: u8,
    pub w0: u16,
    pub w1: u16,
    pub draw: FrameDraw,
    /// Byte +4 -> obj+$2C (per-state parameter, e.g. combo cancel window).
    pub param_a: u8,
    /// Byte +5 -> obj+$2D (phase index).
    pub param_b: u8,
    /// Byte +6 -> obj+$2E (frame type / collision class).
    pub kind: u8,
    /// Byte +7 bits 0-6: duration in ticks (0 means 256).
    pub duration: u8,
    /// Byte +7 bit 7: last frame of the animation; `next` then is the link target.
    pub last: bool,
    /// Byte +8 -> obj+$48 body (hurt) box index, 0 = none.
    pub body_box: u8,
    /// Byte +9 -> obj+$49 attack / probe box index, 0 = none.
    pub attack_box: u8,
    /// Word +10 -> obj+$60 attach point (12-byte records only).
    pub attach: u16,
    /// 4 bytes from the per-composition extra table (0x11D220), selected by the loader rule.
    pub extra: [u8; 4],
    /// Frame that follows: the next record, or the link target when `last` is set.
    pub next: FrameId,
}

/// An animation table: offsets to the first frame of each animation id.
#[derive(Clone, Debug)]
pub struct AnimTable {
    pub rom_addr: u32,
    pub stride: u8,
    /// First frame per animation id; `None` for unused ids.
    pub anims: Vec<Option<FrameId>>,
}

/// One hit box (6 signed words, pixels). See `re/specs/anim/SPEC.md` section 4.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HitBox {
    pub zoff: i16,
    pub zsize: i16,
    pub xoff: i16,
    pub xsize: i16,
    pub yoff: i16,
    pub ysize: i16,
}

/// The ROM region that holds every hit box table, kept as words so the game can index any
/// table the original code names (box tables are addressed by base pointer + index).
#[derive(Clone, Debug, Default)]
pub struct HitBoxBank {
    pub base: u32,
    pub words: Vec<i16>,
}

impl HitBoxBank {
    fn word(&self, addr: u32) -> i16 {
        self.words[((addr - self.base) / 2) as usize]
    }

    fn read(&self, addr: u32) -> HitBox {
        let w = |k: u32| self.word(addr + 2 * k);
        HitBox { zoff: w(0), zsize: w(1), xoff: w(2), xsize: w(3), yoff: w(4), ysize: w(5) }
    }

    /// Single-box table (obj+$4A == 0): box = table + 12 * index.
    pub fn single(&self, table: u32, index: u8) -> HitBox {
        self.read(table + 12 * index as u32)
    }

    /// Like [`single`](Self::single), `None` when the record is outside the bank (a stale or
    /// cleared object pointer).
    pub fn try_single(&self, table: u32, index: u8) -> Option<HitBox> {
        let a = table.checked_add(12 * index as u32)?;
        let end = self.base as u64 + 2 * self.words.len() as u64;
        (a >= self.base && (a as u64 + 12) <= end && a % 2 == 0).then(|| self.read(a))
    }

    /// The 12 byte record at any address (a box offset that is not a multiple of 12, as a stale register can give it); `None` outside the bank.
    pub fn try_at(&self, a: u32) -> Option<HitBox> {
        let end = self.base as u64 + 2 * self.words.len() as u64;
        (a >= self.base && (a as u64 + 12) <= end && a % 2 == 0).then(|| self.read(a))
    }

    /// Multi-box table (obj+$4A == 1): table + i16(table + 2*index) -> (count-1, boxes...).
    pub fn multi(&self, table: u32, index: u8) -> Vec<HitBox> {
        let p = table.wrapping_add(self.word(table + 2 * index as u32) as i32 as u32);
        let n = self.word(p) as u16 as u32 + 1;
        (0..n).map(|k| self.read(p + 2 + 12 * k)).collect()
    }
}

/// 8-byte attack property record (`re/specs/anim/SPEC.md` 4.5).
#[derive(Clone, Copy, Debug, Default)]
pub struct AttackRecord {
    pub damage: i8,
    pub stun: u8,
    pub reaction: u8,
    pub hit_stop: u8,
    pub sound: u8,
    pub score: u8,
    pub spark: u8,
    pub b7: u8,
}

#[derive(Default)]
pub struct SpriteData {
    /// Indexed by composition index / 4 (frame words hold byte offsets).
    pub compositions: Vec<Composition>,
    pub frames: Vec<Frame>,
    pub frame_by_addr: HashMap<u32, FrameId>,
    pub anim_tables: HashMap<u32, AnimTable>,
    /// Player animation sets: `player_sets[set][character]` = animation table ROM address.
    pub player_sets: [[u32; 4]; 6],
    /// Player hit box tables by character (ROM address, from 0x18CC4).
    pub player_box_tables: [u32; 4],
    pub hitboxes: HitBoxBank,
    /// Attack records per character (from the table at 0xA26FC), indexed by obj+$58 / 2.
    pub player_attacks: [Vec<AttackRecord>; 4],
}

impl SpriteData {
    /// Composition for a frame word (a byte offset into the original pointer table).
    pub fn comp(&self, index: u16) -> &Composition {
        &self.compositions[(index / 4) as usize]
    }

    pub fn frame(&self, id: FrameId) -> &Frame {
        &self.frames[id as usize]
    }

    pub fn table(&self, rom_addr: u32) -> &AnimTable {
        self.anim_tables
            .get(&rom_addr)
            .unwrap_or_else(|| panic!("animation table {rom_addr:#x} was not extracted; add it to the extractor registry"))
    }

    /// First frame of animation `id` in table `rom_addr`.
    pub fn anim(&self, rom_addr: u32, id: u16) -> FrameId {
        self.table(rom_addr).anims[id as usize].unwrap_or_else(|| panic!("animation {id} of table {rom_addr:#x} is unused"))
    }

    pub fn frame_at(&self, rom_addr: u32) -> FrameId {
        *self.frame_by_addr.get(&rom_addr).unwrap_or_else(|| panic!("no frame record at {rom_addr:#x}"))
    }
}
