//! Exactness harness support (`plan/phase-00-exactness-harness.md`, `re/harness/README.md`).
//!
//! Every field the port models is listed here once, by its original address (work RAM at
//! 0xFF0000, `G.$x` = 0xFF8000 + x, objects at their handle). One list per struct drives both
//! directions:
//! * [`Collect`] reads the values into a 64 KB RAM image plus a mask of the bytes the port
//!   models; `--trace` writes that image every frame and `re/harness/diff.py` compares it with
//!   MAME's RAM. Bytes outside the mask are "not modeled".
//! * [`Load`] writes values from a MAME RAM dump into the port (`--load-state`), so both sides
//!   start from the same state.
//!
//! Fields the port keeps in another form (indexes instead of ROM pointers, frame ids instead of
//! frame record addresses) are converted here in both directions.

use super::body::Body;
use super::effect::{Effect, Special};
use super::enemy::{Enemy, Ram};
use super::item::{Item, class};
use super::player::Player;
use super::prop::Prop;
use super::draw::{is_slot_frame, slot_frame};
use super::{Game, POOL_A, POOL_B, POOL_C, POOL_D, POOL_E, PLAYERS, Pool, Slot, slot_of};
use dino_assets::effect::EffectData;
use dino_assets::{AssetPack, FrameId};

/// Work RAM base (the trace's "ram" region).
pub const RAM: u32 = 0xFF0000;
/// `G.$x` of the original (`x(a5)` with A5 = 0xFF8000).
pub const fn g(x: u32) -> u32 {
    0xFF8000 + x
}
/// `-x` as an offset from A5 (for `F` based at `g(0)`; addresses wrap at 24 bits).
pub const fn n(x: u32) -> u32 {
    0u32.wrapping_sub(x)
}
/// An object handle as a full address.
pub const fn obj(h: u16) -> u32 {
    RAM + h as u32
}

/// A field visitor. `addr` is the 24-bit address, `size` 1, 2 or 4 bytes, `v` the value as the
/// 68000 stores it (zero-extended).
pub trait Visit {
    fn field(&mut self, addr: u32, size: u8, name: &'static str, v: &mut u32);
    /// True when values flow from the dump into the port.
    fn loading(&self) -> bool {
        false
    }
    /// True when only the field names are wanted: visit conditional fields too.
    fn all(&self) -> bool {
        false
    }
    /// Load side: the raw bytes of an object that the port keeps as a byte array (the part of
    /// an enemy / prop / item beyond the modeled header), so stale bytes of the original are
    /// kept too. The other visitors ignore it.
    fn raw_bytes(&mut self, _addr: u32, _out: &mut [u8]) {}
    fn error(&mut self, _msg: String) {}
    /// Trace side: the original value of a frame pointer the port could not map at load time,
    /// while the object still shows the frame id it got then.
    fn frame_override(&self, _field: u32, _id: FrameId) -> Option<u32> {
        None
    }
    /// Load side: remember a frame pointer that has no extracted frame record.
    fn frame_unmapped(&mut self, _field: u32, _id: FrameId, _raw: u32) {}
}

/// Frame pointers that had no extracted frame record at load time: field address ->
/// (frame id the port kept, original pointer).
pub type Unmapped = std::collections::HashMap<u32, (FrameId, u32)>;

/// A pre-initialization snapshot can have null pointers although the native port keeps
/// only indexes/counts. Retain those nulls while their native state and initialization
/// phase stay unchanged. Discard each entry as soon as either changes, so subsequent
/// trace records expose the newly computed pointer normally.
#[derive(Default)]
pub struct DormantPointers {
    phase: [u8; 6],
    fields: Vec<(usize, usize, Vec<u8>)>,
}

/// Typed access for one object or struct at `base`.
pub struct F<'a> {
    pub v: &'a mut dyn Visit,
    pub base: u32,
    pub assets: &'static AssetPack,
}

impl<'a> F<'a> {
    pub fn new(v: &'a mut dyn Visit, base: u32, assets: &'static AssetPack) -> F<'a> {
        F { v, base, assets }
    }
    fn raw(&mut self, off: u32, size: u8, name: &'static str, x: u32) -> u32 {
        let mut t = x;
        self.v.field(self.base.wrapping_add(off) & 0xFF_FFFF, size, name, &mut t);
        t
    }
    pub fn u8(&mut self, off: u32, name: &'static str, x: &mut u8) {
        *x = self.raw(off, 1, name, *x as u32) as u8;
    }
    pub fn flag(&mut self, off: u32, name: &'static str, x: &mut bool) {
        *x = self.raw(off, 1, name, *x as u32) as u8 != 0;
    }
    pub fn u16(&mut self, off: u32, name: &'static str, x: &mut u16) {
        *x = self.raw(off, 2, name, *x as u32) as u16;
    }
    pub fn i16(&mut self, off: u32, name: &'static str, x: &mut i16) {
        *x = self.raw(off, 2, name, *x as u16 as u32) as u16 as i16;
    }
    pub fn u32(&mut self, off: u32, name: &'static str, x: &mut u32) {
        *x = self.raw(off, 4, name, *x);
    }
    pub fn i32(&mut self, off: u32, name: &'static str, x: &mut i32) {
        *x = self.raw(off, 4, name, *x as u32) as i32;
    }
    /// Load side only: copies `out.len()` bytes at `off` from the dump into `out`.
    pub fn raw_bytes(&mut self, off: u32, out: &mut [u8]) {
        self.v.raw_bytes(self.base.wrapping_add(off) & 0xFF_FFFF, out);
    }
    pub fn loading(&self) -> bool {
        self.v.loading()
    }
    pub fn all(&self) -> bool {
        self.v.all()
    }

    /// A frame record pointer (+$28 / +$30). The port keeps frame ids; the original the ROM
    /// address of the record. `effects`: the object is a pool C effect (effect / raw frames).
    /// `live`: report a pointer without a frame record (only for objects in use).
    pub fn frame(&mut self, off: u32, name: &'static str, id: &mut FrameId, effects: bool, raw: bool, live: bool) {
        let field = self.base.wrapping_add(off) & 0xFF_FFFF;
        let addr = self.v.frame_override(field, *id).unwrap_or_else(|| frame_addr(self.assets, *id));
        let a = self.raw(off, 4, name, addr);
        if self.loading() {
            match frame_id(self.assets, a, effects, raw) {
                Some(n) => *id = n,
                None => {
                    self.v.frame_unmapped(field, *id, a);
                    // 0 (no frame) and RAM pointers (lists an object builds itself) are not ROM
                    // records: they are kept verbatim, with no warning.
                    if live && (0x1000..0x18_0000).contains(&a) {
                        self.v.error(format!("{field:#08x} {name}: no extracted frame record at {a:#x}"));
                    }
                }
            }
        }
    }
}

/// ROM address of a frame id (0 if unknown).
pub fn frame_addr(a: &AssetPack, id: FrameId) -> u32 {
    let fx = &a.effects;
    if is_slot_frame(id) {
        // a pool D slot: `movea.w` made its RAM address 0xFFFFxxxx
        0xFFFF_0000 | POOL_D.0.wrapping_add(POOL_D.1 * (id & 0xFF) as u16) as u32
    } else if EffectData::is_effect_frame(id) {
        fx.frames.get((id & 0x3FFF_FFFF) as usize).map(|f| f.rom_addr).unwrap_or(0)
    } else if EffectData::is_raw_frame(id) {
        fx.raw_frames.get((id & 0x3FFF_FFFF) as usize).map(|f| f.rom_addr).unwrap_or(0)
    } else {
        a.sprites.frames.get(id as usize).map(|f| f.rom_addr).unwrap_or(0)
    }
}

/// Frame id of a ROM frame record address.
pub fn frame_id(a: &AssetPack, addr: u32, effects: bool, raw: bool) -> Option<FrameId> {
    if addr >> 16 == 0xFFFF {
        if let Some(Slot::Special(i)) = slot_of(addr as u16) {
            return Some(slot_frame(i));
        }
    }
    let fx = &a.effects;
    let s = a.sprites.frame_by_addr.get(&addr).copied();
    let e = fx.frame_by_addr.get(&addr).copied();
    let r = fx.raw_by_addr.get(&addr).copied();
    if raw {
        r.or(e).or(s)
    } else if effects {
        e.or(s).or(r)
    } else {
        s.or(e).or(r)
    }
}

// ------------------------------------------------------------------------------------------
// Visitors
// ------------------------------------------------------------------------------------------

/// Collects values into a RAM image and a mask (trace side).
pub struct Collect {
    pub ram: Vec<u8>,
    pub mask: Vec<u8>,
    pub unmapped: Unmapped,
}

impl Default for Collect {
    fn default() -> Self {
        Collect { ram: vec![0; 0x10000], mask: vec![0; 0x10000], unmapped: Unmapped::new() }
    }
}

impl Visit for Collect {
    fn field(&mut self, addr: u32, size: u8, _name: &'static str, v: &mut u32) {
        let Some(o) = addr.checked_sub(RAM).map(|o| o as usize) else { return };
        let bytes = v.to_be_bytes();
        for k in 0..size as usize {
            if o + k < self.ram.len() {
                self.ram[o + k] = bytes[4 - size as usize + k];
                self.mask[o + k] = 1;
            }
        }
    }
    fn frame_override(&self, field: u32, id: FrameId) -> Option<u32> {
        self.unmapped.get(&field).filter(|(i, _)| *i == id).map(|(_, a)| *a)
    }
}

/// Collects (address, size, name) of every field (header of the port trace).
#[derive(Default)]
pub struct Names {
    pub list: Vec<(u32, u8, &'static str)>,
}

impl Visit for Names {
    fn field(&mut self, addr: u32, size: u8, name: &'static str, _v: &mut u32) {
        self.list.push((addr, size, name));
    }
    fn all(&self) -> bool {
        true
    }
}

/// Fills the port from a MAME RAM image (64 KB at 0xFF0000).
pub struct Load<'a> {
    pub ram: &'a [u8],
    pub errors: Vec<String>,
    pub unmapped: Unmapped,
}

impl Load<'_> {
    pub fn read(&self, addr: u32, size: u8) -> u32 {
        let o = (addr - RAM) as usize;
        (0..size as usize).fold(0u32, |acc, k| (acc << 8) | self.ram[o + k] as u32)
    }
}

impl Visit for Load<'_> {
    fn field(&mut self, addr: u32, size: u8, _name: &'static str, v: &mut u32) {
        if addr >= RAM && addr + size as u32 <= RAM + 0x10000 {
            *v = self.read(addr, size);
        }
    }
    fn loading(&self) -> bool {
        true
    }
    fn raw_bytes(&mut self, addr: u32, out: &mut [u8]) {
        if addr >= RAM && addr as usize + out.len() <= RAM as usize + 0x10000 {
            let o = (addr - RAM) as usize;
            out.copy_from_slice(&self.ram[o..o + out.len()]);
        }
    }
    fn error(&mut self, msg: String) {
        self.errors.push(msg);
    }
    fn frame_unmapped(&mut self, field: u32, id: FrameId, raw: u32) {
        self.unmapped.insert(field, (id, raw));
    }
}

// ------------------------------------------------------------------------------------------
// Objects
// ------------------------------------------------------------------------------------------

impl Body {
    /// +$00..+$2F: the header every object uses.
    pub fn trace_header(&mut self, f: &mut F, effects: bool) {
        f.u8(0x00, "live", &mut self.live);
        f.u8(0x01, "visible", &mut self.visible);
        f.u8(0x02, "who", &mut self.who);
        f.u8(0x03, "id", &mut self.id);
        f.u8(0x04, "mode", &mut self.mode);
        f.u8(0x05, "sub", &mut self.sub);
        f.u8(0x06, "act", &mut self.act);
        f.u8(0x07, "step", &mut self.step);
        f.i32(0x08, "x", &mut self.x);
        f.i32(0x0C, "y", &mut self.y);
        f.i32(0x10, "z", &mut self.z);
        f.i16(0x14, "vx", &mut self.vx);
        f.i16(0x16, "vy", &mut self.vy);
        f.i16(0x18, "vz", &mut self.vz);
        f.i16(0x1A, "ax", &mut self.ax);
        f.i16(0x1C, "ay", &mut self.ay);
        f.i16(0x1E, "az", &mut self.az);
        f.u16(0x20, "class", &mut self.k20);
        f.u8(0x22, "dir", &mut self.dir);
        f.u8(0x23, "palette_override", &mut self.palette_override);
        f.u8(0x24, "facing", &mut self.facing);
        f.u8(0x25, "draw_flags", &mut self.draw_flags);
        f.u16(0x26, "var26", &mut self.var26);
        let raw = self.draw_flags & 0x40 != 0;
        f.frame(0x28, "frame", &mut self.frame, effects, raw, self.live != 0);
        f.u8(0x2C, "param_a", &mut self.param_a);
        f.u8(0x2D, "param_b", &mut self.param_b);
        f.u8(0x2E, "kind", &mut self.kind);
        f.u8(0x2F, "timer", &mut self.timer);
    }

    /// +$30..+$3F: split body (only when +$25 bit 0 is set).
    pub fn trace_split(&mut self, f: &mut F) {
        if f.all() || self.draw_flags & 1 != 0 {
            f.frame(0x30, "frame2", &mut self.frame2, false, false, self.live != 0);
            f.u16(0x34, "w34", &mut self.w34);
            f.u16(0x36, "w36", &mut self.w36);
            f.u32(0x38, "offs_table", &mut self.offs_table);
            f.i16(0x3C, "dx3c", &mut self.dx3c);
            f.i16(0x3E, "dy3e", &mut self.dy3e);
        }
    }

    /// +$40..+$9A: animation, boxes, floor, combat and HP fields.
    pub fn trace_rest(&mut self, f: &mut F) {
        f.u32(0x40, "anim_table", &mut self.anim_table);
        f.u32(0x44, "box_table", &mut self.box_table);
        f.u8(0x48, "body_box", &mut self.body_box);
        f.u8(0x49, "attack_box", &mut self.attack_box);
        f.flag(0x4A, "multi_box", &mut self.multi_box);
        for (k, name) in ["extra0", "extra1", "extra2", "extra3"].into_iter().enumerate() {
            f.u8(0x4C + k as u32, name, &mut self.extra[k]);
        }
        f.u8(0x50, "wall", &mut self.wall);
        f.u8(0x51, "airborne", &mut self.airborne);
        f.u8(0x52, "b52", &mut self.b52);
        f.u8(0x53, "wading", &mut self.wading);
        f.i16(0x54, "floor", &mut self.floor);
        f.i16(0x56, "floor2", &mut self.floor2);
        f.u8(0x58, "atk", &mut self.atk);
        f.u8(0x59, "hitstop", &mut self.hitstop);
        f.u8(0x5A, "knock", &mut self.knock);
        f.u8(0x5E, "serial", &mut self.serial);
        f.u16(0x60, "attach", &mut self.attach);
        f.u8(0x62, "b62", &mut self.b62);
        f.u8(0x63, "b63", &mut self.b63);
        f.u32(0x64, "speed_table", &mut self.speed_table);
        f.u16(0x68, "attacker", &mut self.attacker);
        f.u16(0x6A, "hp_bar", &mut self.hp_bar);
        f.u16(0x6C, "hp", &mut self.hp);
        f.u16(0x6E, "hp_prev", &mut self.hp_prev);
        f.u16(0x70, "partner", &mut self.partner);
        f.u8(0x72, "b72", &mut self.b72);
        f.u8(0x78, "stun", &mut self.stun);
        f.u8(0x7B, "reaction", &mut self.reaction);
        f.u8(0x9A, "b9a", &mut self.b9a);
    }

    /// All shared fields (players and pool A).
    pub fn trace_visit(&mut self, f: &mut F) {
        self.trace_header(f, false);
        self.trace_split(f);
        self.trace_rest(f);
    }
}

/// (offset, size, name) lists for the raw `Ram` bytes of the pool objects.
const ENEMY_RAW: &[(u32, u8, &str)] = &[
    (0x5B, 1, "spawn_b5b"),
    (0x5C, 2, "spawn_w5c"),
    (0x76, 2, "target"),
    (0x79, 1, "poise_t"),
    (0x7A, 1, "wait"),
    (0x7C, 1, "slot"),
    (0x7D, 1, "attacking"),
    (0x80, 2, "t80"),
    (0x83, 1, "b83"),
    (0x84, 2, "w84"),
    (0x86, 1, "stuck_t"),
    (0x87, 1, "b87"),
    (0x88, 2, "dest_x"),
    (0x8A, 2, "dest_z"),
    (0x8C, 2, "last_x"),
    (0x8E, 2, "last_z"),
    (0x90, 1, "b90"),
    (0x94, 2, "obstacle"),
    (0x96, 1, "drop"),
    (0xA0, 2, "wa0"),
    (0xA2, 2, "wa2"),
    (0xA4, 1, "taunted"),
    (0xA5, 1, "repos"),
    (0xA6, 1, "detour_t"),
    (0xA7, 1, "detour"),
    (0xA8, 1, "combo"),
    (0xA9, 1, "dash_t"),
    (0xAA, 2, "cursor"),
    (0xAC, 1, "dash_chk"),
    (0xAD, 1, "dash_kind"),
    (0xAE, 2, "dash_x"),
    (0xB0, 1, "turned"),
    (0xB2, 2, "wb2"),
    (0xB4, 2, "start_x"),
    (0xB6, 2, "idle_t"),
    (0xB8, 1, "pause"),
    (0xB9, 1, "no_dash_atk"),
    (0xBA, 1, "fade_t"),
    (0xBB, 1, "fade_step"),
    (0xBC, 1, "obst_t"),
    // Bytes that no shared field and no class 2 name covers (classes 49, 50, 54, 55 and later ones
    // use them); a byte is a byte, whatever the class calls it.
    (0x4B, 1, "r4b"),
    (0x5F, 1, "r5f"),
    (0x73, 1, "r73"),
    (0x74, 1, "r74"),
    (0x75, 1, "r75"),
    (0x7E, 1, "r7e"),
    (0x7F, 1, "r7f"),
    (0x82, 1, "r82"),
    (0x91, 1, "r91"),
    (0x92, 1, "r92"),
    (0x93, 1, "r93"),
    (0x97, 1, "r97"),
    (0x98, 1, "r98"),
    (0x99, 1, "r99"),
    (0x9B, 1, "r9b"),
    (0x9C, 1, "r9c"),
    (0x9D, 1, "r9d"),
    (0x9E, 1, "r9e"),
    (0x9F, 1, "r9f"),
    (0xB1, 1, "rb1"),
    (0xBD, 1, "rbd"),
    (0xBE, 1, "rbe"),
    (0xBF, 1, "rbf"),
    (0xC0, 1, "rc0"),
    (0xC1, 1, "rc1"),
    (0xC2, 1, "rc2"),
    (0xC3, 1, "rc3"),
    (0xC4, 1, "rc4"),
    (0xC5, 1, "rc5"),
    (0xC6, 1, "rc6"),
    (0xC7, 1, "rc7"),
    (0xC8, 1, "rc8"),
    (0xC9, 1, "rc9"),
    (0xCA, 1, "rca"),
    (0xCB, 1, "rcb"),
    (0xCC, 1, "rcc"),
    (0xCD, 1, "rcd"),
    (0xCE, 1, "rce"),
    (0xCF, 1, "rcf"),
    (0xD0, 1, "rd0"),
    (0xD1, 1, "rd1"),
    (0xD2, 1, "rd2"),
    (0xD3, 1, "rd3"),
    (0xD4, 1, "rd4"),
    (0xD5, 1, "rd5"),
    (0xD6, 1, "rd6"),
    (0xD7, 1, "rd7"),
    (0xD8, 1, "rd8"),
    (0xD9, 1, "rd9"),
    (0xDA, 1, "rda"),
    (0xDB, 1, "rdb"),
    (0xDC, 1, "rdc"),
    (0xDD, 1, "rdd"),
    (0xDE, 1, "rde"),
    (0xDF, 1, "rdf"),
];

/// Raw bytes of a pool E object that the item code reads or writes (everything the shared
/// header `Body` does not cover). One byte each: the classes use the same offset as a byte or as
/// half of a word.
const ITEM_RAW: &[(u32, u8, &str)] = &[
    (0x5B, 1, "r5b"),
    (0x5C, 1, "r5c"),
    (0x5D, 1, "r5d"),
    (0x73, 1, "r73"),
    (0x74, 1, "r74"),
    (0x75, 1, "r75"),
    (0x76, 1, "r76"),
    (0x77, 1, "r77"),
    (0x79, 1, "r79"),
    (0x7A, 1, "r7a"),
    (0x7C, 1, "r7c"),
    (0x7D, 1, "r7d"),
    (0x7E, 1, "r7e"),
    (0x7F, 1, "r7f"),
    (0x80, 1, "r80"),
    (0x81, 1, "r81"),
    (0x82, 1, "r82"),
    (0x83, 1, "r83"),
    (0x84, 1, "r84"),
    (0x85, 1, "r85"),
    (0x86, 1, "r86"),
    (0x87, 1, "r87"),
    (0x88, 1, "r88"),
    (0x89, 1, "r89"),
    (0x8A, 1, "r8a"),
    (0x8B, 1, "r8b"),
    (0x8C, 1, "r8c"),
    (0x8D, 1, "r8d"),
    (0x8E, 1, "r8e"),
    (0x8F, 1, "r8f"),
    (0x90, 1, "r90"),
    (0x91, 1, "r91"),
    (0x92, 1, "r92"),
    (0x93, 1, "r93"),
    (0x94, 1, "r94"),
    (0x95, 1, "r95"),
    (0x96, 1, "r96"),
    (0x97, 1, "r97"),
    (0x98, 1, "r98"),
    (0x99, 1, "r99"),
    (0x9B, 1, "r9b"),
    (0x9C, 1, "r9c"),
    (0x9D, 1, "r9d"),
    (0x9E, 1, "r9e"),
    (0x9F, 1, "r9f"),
    (0xA0, 1, "ra0"),
    (0xA1, 1, "ra1"),
    (0xA2, 1, "ra2"),
    (0xA3, 1, "ra3"),
    (0xA4, 1, "ra4"),
    (0xA5, 1, "ra5"),
    (0xA6, 1, "ra6"),
    (0xA7, 1, "ra7"),
    (0xA8, 1, "ra8"),
    (0xA9, 1, "ra9"),
    (0xAA, 1, "raa"),
    (0xAB, 1, "rab"),
    (0xAC, 1, "rac"),
    (0xAD, 1, "rad"),
    (0xAE, 1, "rae"),
    (0xAF, 1, "raf"),
    (0xB0, 1, "rb0"),
    (0xB1, 1, "rb1"),
    (0xB2, 1, "rb2"),
    (0xB3, 1, "rb3"),
    (0xB4, 1, "rb4"),
    (0xB5, 1, "rb5"),
    (0xB6, 1, "rb6"),
    (0xB7, 1, "rb7"),
    (0xB8, 1, "rb8"),
    (0xB9, 1, "rb9"),
    (0xBA, 1, "rba"),
    (0xBB, 1, "rbb"),
    (0xBC, 1, "rbc"),
    (0xBD, 1, "rbd"),
    (0xBE, 1, "rbe"),
    (0xBF, 1, "rbf"),
];

impl Ram {
    fn trace_list(&mut self, f: &mut F, list: &[(u32, u8, &'static str)]) {
        for &(off, size, name) in list {
            let o = off as usize;
            match size {
                1 => {
                    let mut v = self.b(o);
                    f.u8(off, name, &mut v);
                    self.sb(o, v);
                }
                2 => {
                    let mut v = self.w(o);
                    f.u16(off, name, &mut v);
                    self.sw(o, v);
                }
                _ => {
                    let mut v = self.l(o);
                    f.u32(off, name, &mut v);
                    self.sl(o, v);
                }
            }
        }
    }
}

impl Ram {
    /// Load side: the whole byte array of an object of `len` bytes (stale bytes included).
    pub fn trace_load_all(&mut self, f: &mut F, len: usize) {
        if f.loading() {
            f.raw_bytes(0, &mut self.0[..len]);
        }
    }
}

impl Enemy {
    pub fn trace_visit(&mut self, f: &mut F) {
        self.m.trace_load_all(f, 0xE0);
        self.b.trace_visit(f);
        self.m.trace_list(f, ENEMY_RAW);
    }
}

impl Item {
    /// Header, the copy of the frame record in +$30 (items copy +$28 there every frame), the
    /// shared fields +$40..+$9A and the raw bytes `ITEM_RAW`. When loading, all raw bytes first.
    pub fn trace_visit(&mut self, f: &mut F) {
        self.m.trace_load_all(f, 0xC0);
        self.b.trace_header(f, false);
        let live = self.b.live != 0;
        f.frame(0x30, "frame2", &mut self.b.frame2, false, false, live);
        self.b.trace_rest(f);
        self.m.trace_list(f, ITEM_RAW);
    }
}

impl Prop {
    /// Header, and the wave controller (class 15) fields. The port keeps the wave position as
    /// group / record indexes (+$A0 / +$A2); the original keeps a ROM pointer at +$A0.
    pub fn trace_visit(&mut self, f: &mut F, stage: u8) {
        self.b.trace_header(f, false);
        self.m.trace_load_all(f, 0xC0);
        if f.all() || self.b.k20 == 11 * 4 {
            // The metal door's armed wait (class 11, 0x78A20) is the byte at +$80.
            let mut w = self.m.b(0x80);
            f.u8(0x80, "door_wait", &mut w);
            self.m.sb(0x80, w);
        }
        if f.all() || self.b.k20 == 14 * 4 {
            // The hole's hold (class 14 var 0, 0x7B0E2) is the word at +$A0. Var 4 does not use it.
            let mut w = self.m.w(0xA0);
            f.u16(0xA0, "hole_wait", &mut w);
            self.m.sw(0xA0, w);
        }
        if f.all() || self.b.k20 == 73 * 4 || self.b.k20 == 96 * 4 {
            // The bird spawner's delay (class 73, 0x89AE0) and class 96 music timer sit in the same word.
            let mut t = self.m.w(0x80);
            f.u16(0x80, "wave_timer", &mut t);
            self.m.sw(0x80, t);
        }
        if f.all() || self.b.k20 == 20 * 4 {
            for (off, name) in [(0x80, "go_ticks"), (0x81, "go_blinks")] {
                let mut v = self.m.b(off);
                f.u8(off as u32, name, &mut v);
                self.m.sb(off, v);
            }
            let mut x = self.m.w(0xA0);
            f.u16(0xA0, "go_reference_x", &mut x);
            self.m.sw(0xA0, x);
        }
        if f.all() || self.b.k20 == 31 * 4 {
            // The stage clock (0x81046): its pool D slot, the clock word shown last and the display window.
            for (off, name) in [(0x98, "clock_slot"), (0xA0, "clock_time"), (0xA2, "clock_window")] {
                let mut v = self.m.w(off);
                f.u16(off as u32, name, &mut v);
                self.m.sw(off, v);
            }
        }
        if f.all() || self.b.k20 == 22 * 4 {
            self.trace_boss_controller(f, stage);
        }
        if f.all() || self.b.k20 == 15 * 4 {
            let mut t = self.m.w(0x80);
            f.u16(0x80, "wave_timer", &mut t);
            self.m.sw(0x80, t);
            let mut n = self.m.w(0xA6);
            f.u16(0xA6, "wave_spawned", &mut n);
            self.m.sw(0xA6, n);
            let waves = &f.assets.enemy.stages[stage as usize].waves;
            if let Some(w) = waves.get(self.b.var26 as usize) {
                if self.b.mode == 0 && !f.all() {
                    // A new controller: 0x7B34A (mode 0) has not run yet, so +$A0 still holds
                    // stale bytes. Keep them as they are.
                    let mut p = self.m.l(0xA0);
                    f.u32(0xA0, "wave_ptr", &mut p);
                    self.m.sl(0xA0, p);
                } else {
                    let (g, r) = (self.m.w(0xA0) as usize, self.m.w(0xA2) as usize);
                    // A controller that has ended (mode 6, 0x7B4DE / 0x7B4F2 / 0x7B506 write the word 0x0600 over mode and sub) keeps
                    // the pointer of the end word (+6) in RAM; the port keeps the group index, which is past the last group then.
                    let sub = if self.b.mode == 6 && g >= w.groups.len() { 4 } else { self.b.sub };
                    let mut p = wave_ptr(w, g, r, sub);
                    f.u32(0xA0, "wave_ptr", &mut p);
                    if f.loading() {
                        match wave_pos(w, p) {
                            Some((g, r)) => {
                                self.m.sw(0xA0, g as u16);
                                self.m.sw(0xA2, r as u16);
                            }
                            // (a free slot keeps stale bytes: no warning for those)
                            None if self.b.live != 0 => f.v.error(format!("wave controller: pointer {p:#x} not in wave {:#x}", w.addr)),
                            None => {}
                        }
                    }
                }
            }
        }
    }
}

impl Prop {
    /// The boss reinforcement controller (class 22, 0x7BEF6, `boss.rs`): the port keeps list indexes (`+$A0` timed
    /// record, `+$A4` / `+$A6` group and record), the original ROM pointers at `+$A0` and `+$A4`.
    fn trace_boss_controller(&mut self, f: &mut F, stage: u8) {
        for (off, name) in [(0x80, "boss_group_timer"), (0xA8, "boss_timed_timer")] {
            let mut v = self.m.w(off);
            f.u16(off as u32, name, &mut v);
            self.m.sw(off, v);
        }
        let mut done = self.m.b(0xAE);
        f.u8(0xAE, "boss_done", &mut done);
        self.m.sb(0xAE, done);
        let mut boss = self.m.l(0xAA);
        f.u32(0xAA, "boss_obj", &mut boss);
        self.m.sl(0xAA, boss);
        let Some(l) = f.assets.boss.stages.get(stage as usize) else { return };
        if self.b.mode == 0 && !f.all() {
            // A new controller: the init (0x7BF0A) has not run, the pointers are stale bytes. Keep them.
            for (off, name) in [(0xA0, "boss_timed_ptr"), (0xA4, "boss_group_ptr")] {
                let mut p = self.m.l(off);
                f.u32(off as u32, name, &mut p);
                self.m.sl(off, p);
            }
            return;
        }
        let mut t = l.timed_addr + 16 * self.m.w(0xA0) as u32;
        f.u32(0xA0, "boss_timed_ptr", &mut t);
        let (g, r) = (self.m.w(0xA4) as usize, self.m.w(0xA6) as usize);
        // after the final thresholds (0x7C060) the pointer stays 6 bytes past them
        let mut p = if done != 0 { wave_ptr(&l.groups, g, 0, 0) + 6 } else { wave_ptr(&l.groups, g, r, self.b.sub) };
        f.u32(0xA4, "boss_group_ptr", &mut p);
        if f.loading() {
            let k = t.wrapping_sub(l.timed_addr);
            if k % 16 == 0 && ((k / 16) as usize) < l.timed.len() {
                self.m.sw(0xA0, (k / 16) as u16);
            } else if self.b.live != 0 {
                f.v.error(format!("boss controller: timed pointer {t:#x} not in the list {:#x}", l.timed_addr));
            }
            match wave_pos(&l.groups, p) {
                Some((g, r)) => {
                    self.m.sw(0xA4, g as u16);
                    self.m.sw(0xA6, r as u16);
                }
                None if self.b.live != 0 => f.v.error(format!("boss controller: group pointer {p:#x} not in the list {:#x}", l.groups.addr)),
                None => {}
            }
        }
    }
}

/// Start address of each group's threshold words, and of the final threshold.
fn wave_groups(w: &dino_assets::enemy::Wave) -> Vec<u32> {
    let mut v = Vec::new();
    let mut a = w.addr;
    for g in &w.groups {
        v.push(a);
        a += 6 + 16 * g.spawns.len() as u32 + 16;
    }
    v.push(a);
    v
}

/// The original's +$A0 for (group, record, sub-state): the group's thresholds while waiting
/// (sub 0), the current record while spawning (sub 2), the end word at the end (sub 4).
fn wave_ptr(w: &dino_assets::enemy::Wave, g: usize, r: usize, sub: u8) -> u32 {
    let gs = wave_groups(w);
    let base = gs[g.min(gs.len() - 1)];
    match sub {
        0 => base,
        _ => base + 6 + 16 * r as u32,
    }
}

fn wave_pos(w: &dino_assets::enemy::Wave, p: u32) -> Option<(usize, usize)> {
    let gs = wave_groups(w);
    for (g, &a) in gs.iter().enumerate() {
        if p == a {
            return Some((g, 0));
        }
        let n = w.groups.get(g).map(|x| x.spawns.len()).unwrap_or(0);
        if p >= a + 6 && p <= a + 6 + 16 * n as u32 && (p - a - 6) % 16 == 0 {
            return Some((g, ((p - a - 6) / 16) as usize));
        }
    }
    None
}

impl Effect {
    pub fn trace_visit(&mut self, f: &mut F) {
        self.b.trace_header(f, true);
        f.u32(0x40, "anim_table", &mut self.b.anim_table);
        f.u16(0x58, "owner58", &mut self.owner58);
        f.u16(0x5A, "owner_class", &mut self.owner_class);
        f.u16(0x5C, "w5c", &mut self.w5c);
        f.u16(0x60, "owner60", &mut self.owner60);
        f.i16(0x62, "off_x", &mut self.off_x);
        f.i16(0x64, "off_y", &mut self.off_y);
    }
}

/// The words of a pool D slot the stage clock fills (0x81088 copies 18 longs): the draw record at +$00 (a long pointer and
/// four parameter bytes) and the sprite list from +$0A. Names by offset.
const SPECIAL_WORDS: [&str; 32] = [
    "w08", "w0a", "w0c", "w0e", "w10", "w12", "w14", "w16", "w18", "w1a", "w1c", "w1e", "w20", "w22", "w24", "w26", "w28", "w2a",
    "w2c", "w2e", "w30", "w32", "w34", "w36", "w38", "w3a", "w3c", "w3e", "w40", "w42", "w44", "w46",
];

impl Special {
    /// A pool D slot (0x48A8) is a byte image (`m`): the first 0x48 bytes are modeled, as the words of `SPECIAL_WORDS`
    /// after the two longs of the draw record. `b.live` mirrors the byte +$00 (the high byte of the list pointer, 0xFF for the
    /// clock's slot, which the free routine 0x4994 does not clear); the pool visit looks at it to tell a free slot (only +$00
    /// is compared) from a used one. When loading, all the bytes come first (stale ones too).
    pub fn trace_visit(&mut self, f: &mut F) {
        self.m.trace_load_all(f, super::effect::SPECIAL_SIZE);
        self.b.live = self.m.b(0);
        let mut v = self.m.l(0);
        f.u32(0, "list_ptr", &mut v);
        self.m.sl(0, v);
        let mut v = self.m.l(4);
        f.u32(4, "record", &mut v);
        self.m.sl(4, v);
        for (k, name) in SPECIAL_WORDS.into_iter().enumerate() {
            let off = 8 + 2 * k;
            let mut v = self.m.w(off);
            f.u16(off as u32, name, &mut v);
            self.m.sw(off, v);
        }
        self.b.live = self.m.b(0);
    }
}

impl Player {
    /// Player-only fields. The HUD / flow / enemy ports keep some player fields elsewhere; those
    /// are visited by `Game::trace_player_extras`.
    pub fn trace_visit(&mut self, f: &mut F) {
        self.b.trace_visit(f);
        f.u16(0x94, "obstacle", &mut self.w94);
        f.u8(0x73, "escape", &mut self.escape);
        f.u8(0x75, "b75", &mut self.b75);
        f.u16(0x76, "w76", &mut self.w76);
        f.u8(0x7A, "b7a", &mut self.b7a);
        f.u8(0x7E, "b7e", &mut self.b7e);
        f.u8(0x7F, "b7f", &mut self.b7f);
        f.u16(0x80, "w80", &mut self.w80);
        f.u8(0x83, "b83", &mut self.b83);
        f.u16(0x88, "w88", &mut self.w88);
        f.u16(0x8A, "w8a", &mut self.w8a);
        f.u8(0x90, "b90", &mut self.b90);
        f.u8(0xA0, "held", &mut self.held);
        f.u8(0xA1, "prev", &mut self.prev);
        f.u8(0xA3, "lives", &mut self.lives);
        f.u8(0xA7, "last_facing", &mut self.last_facing);
        f.u8(0xA8, "pressed", &mut self.pressed);
        f.u8(0xA9, "ba9", &mut self.ba9);
        f.u8(0xAA, "atk_hold", &mut self.atk_hold);
        f.u8(0xAB, "facing_age", &mut self.facing_age);
        f.u8(0xAC, "run_dir", &mut self.run_dir);
        f.u8(0xAD, "atk_buf", &mut self.atk_buf);
        f.u8(0xAE, "combo_t", &mut self.combo_t);
        f.u8(0xAF, "wall_latch", &mut self.wall_latch);
        f.u8(0xB2, "combo", &mut self.combo);
        f.u8(0xB3, "lock", &mut self.lock);
        f.u16(0xB4, "item", &mut self.item);
        f.u16(0xB6, "item_type", &mut self.item_type);
        f.u16(0xB8, "cand", &mut self.cand);
        f.u16(0xBA, "cand_type", &mut self.cand_type);
        f.u8(0xBC, "dtap", &mut self.dtap);
        f.u8(0xBD, "walk_in", &mut self.walk_in);
        f.u8(0xBE, "mash", &mut self.mash);
        f.u8(0xBF, "mash_cd", &mut self.mash_cd);
        f.u8(0xC0, "bc0", &mut self.bc0);
        f.u8(0xC1, "bc1", &mut self.bc1);
        f.u8(0xC4, "weapon_atk", &mut self.weapon_atk);
        f.u8(0xC5, "drop", &mut self.drop);
        f.u8(0xC6, "power", &mut self.power);
        f.u8(0xC7, "bc7", &mut self.bc7);
        f.u8(0xC8, "action", &mut self.action);
        f.u8(0xC9, "special_buf", &mut self.special_buf);
        f.u8(0xCA, "hits", &mut self.hits);
        f.u8(0xCB, "hits_t", &mut self.hits_t);
        f.u8(0xCC, "tech", &mut self.tech);
        f.u8(0xCD, "slow", &mut self.slow);
        f.u16(0xCE, "dizzy_cd", &mut self.dizzy_cd);
        f.u8(0xD0, "bd0", &mut self.bd0);
        // The scripted control (`script.rs`): the state, the action, the step, the pointer, the timer, the walk rank and counter, the target.
        f.u8(0xD1, "sc_state", &mut self.sc.state);
        f.u8(0xD2, "sc_act", &mut self.sc.act);
        f.u8(0xD3, "sc_step", &mut self.sc.step);
        f.u32(0xD4, "sc_ptr", &mut self.sc.ptr);
        f.u16(0xD8, "sc_timer", &mut self.sc.timer);
        f.u8(0xDA, "sc_rank", &mut self.sc.rank);
        f.u8(0xDB, "sc_count", &mut self.sc.count);
        f.u16(0xDC, "sc_tx", &mut self.sc.tx);
        f.u16(0xDE, "sc_tz", &mut self.sc.tz);
        f.u8(0xE8, "lie_t", &mut self.lie_t);
        f.u8(0xE9, "hit_serial", &mut self.hit_serial);
        f.u16(0xEA, "aim", &mut self.aim);
        f.u16(0xEC, "respawn_t", &mut self.respawn_t);
        f.u16(0xEE, "special_budget", &mut self.special_budget);
        f.u8(0xF0, "regrab_cd", &mut self.regrab_cd);
        f.u8(0xF1, "cmd_flag", &mut self.cmd_flag);
        f.u8(0xF4, "cmd_state", &mut self.cmd_state);
        f.u8(0xF5, "cmd_t", &mut self.cmd_t);
        f.u16(0xF6, "stun_refill", &mut self.stun_refill);
        f.u8(0xF8, "pickup_cd", &mut self.pickup_cd);
        f.u8(0xF9, "invuln", &mut self.invuln);
        f.u16(0xFA, "run_t", &mut self.run_t);
        f.u16(0xFC, "run_cd", &mut self.run_cd);
        f.u32(0x100, "score", &mut self.score);
        f.u8(0x109, "b109", &mut self.b109);
        f.u8(0x10A, "b10a", &mut self.b10a);
        f.u16(0x10C, "w10c", &mut self.w10c);
        f.u8(0x10E, "b10e", &mut self.b10e);
        f.u8(0x10F, "b10f", &mut self.b10f);
        f.u8(0x115, "knees", &mut self.knees);
        f.u8(0x116, "b116", &mut self.b116);
        f.u8(0x117, "turn", &mut self.turn);
        f.u8(0x11A, "input_lock", &mut self.input_lock);
        f.u8(0x11B, "b11b", &mut self.b11b);
        f.u8(0x11C, "b11c", &mut self.b11c);
        f.u8(0x11D, "b11d", &mut self.b11d);
        f.u8(0x118, "voice", &mut self.voice);
    }
}

// ------------------------------------------------------------------------------------------
// The whole game
// ------------------------------------------------------------------------------------------

/// Pool table for the diff tool: (kind, base handle, stride, count).
pub const POOLS: [(&str, (u16, u16, usize)); 6] = [
    ("player", PLAYERS),
    ("enemy", POOL_A),
    ("prop", POOL_B),
    ("item", POOL_E),
    ("effect", POOL_C),
    ("special", POOL_D),
];

/// Object classes (`+$20 / 4`) the port runs, per pool kind; every other live class is held as the
/// snapshot had it (`Game::trace_hold`). The diff tool marks differences of other classes as
/// "unported". Keep in sync with the dispatch in `enemies_update`, `pool_b_update`,
/// `pool_c_update`, `pool_e_update`.
pub const PORTED_CLASSES: [(&str, &[u16]); 6] = [
    ("player", &[0]),
    ("enemy", &[2, 10, 14, 15, 16, 17, 20, 27, 34, 41, 43, 49, 50, 54, 55, 56, 57, 60]),
    ("prop", &[11, 14, 15, 18, 20, 21, 22, 25, 31, 35, 47, 66, 73, 74, 96]),
    (
        "item",
        &[
            class::WEAPON,
            class::BLAST,
            class::AMMO,
            class::BULLET,
            class::FOOD,
            class::CONTAINER,
            class::SHELL,
            class::PROJECTILE,
            class::TREASURE,
            class::GUN_SHOT,
            class::ATTACK_BOX,
        ],
    ),
    ("effect", &[0x04 / 4, 0x10 / 4, 0x18 / 4, 0x20 / 4, 0x38 / 4, 0x58 / 4, 0x5C / 4, 0x68 / 4, 0x70 / 4, 0x88 / 4]),
    // a pool D slot has no class word: the diff tool reads +$20 of the slot, which is 0 for the clock's slot (a sprite word)
    ("special", &[0]),
];

/// Free list stacks: (count G.x, pointer G.x, stack end G.x) for pools E, B, A, C, D.
const FREE_LISTS: [(u32, u32, u32); 5] =
    [(0x65F4, 0x65FE, 0x6638), (0x65F6, 0x6600, 0x6668), (0x65F8, 0x6602, 0x6698), (0x65FA, 0x6604, 0x66F8), (0x65FC, 0x6606, 0x6718)];

impl Game {
    /// Visits every modeled object field and global.
    pub fn trace_visit(&mut self, v: &mut dyn Visit) {
        let a = self.assets;
        let names_only = v.all();
        let loading = v.loading();
        for i in 0..3 {
            let base = obj(Game::player_handle(i));
            let mut f = F::new(v, base, a);
            self.players[i].trace_visit(&mut f);
            self.trace_player_extras(i, &mut f);
        }
        // Free slots: only +$00 (the rest is stale memory in the original).
        macro_rules! pool {
            ($pool:expr, $visit:ident $(, $arg:expr)*) => {
                for i in 0..$pool.slots.len() {
                    let base = obj($pool.handle(i));
                    let mut f = F::new(v, base, a);
                    // Free slots: only +$00 when tracing (the rest is stale memory that the port does
                    // not keep identical). When loading, the stale bytes of the dump are taken too.
                    if $pool.slots[i].b.live == 0 && !names_only && !loading {
                        f.u8(0, "live", &mut $pool.slots[i].b.live);
                    } else {
                        $pool.slots[i].$visit(&mut f $(, $arg)*);
                    }
                }
            };
        }
        let stage = self.w.stage;
        pool!(self.enemies, trace_visit);
        pool!(self.props, trace_visit, stage);
        pool!(self.items, trace_visit);
        pool!(self.effects, trace_visit);
        pool!(self.specials, trace_visit);
        self.trace_free_lists(v);
        self.trace_globals(v);
    }

    /// Player fields kept by other ports (census and kills: enemy port; join: flow port; HUD).
    fn trace_player_extras(&mut self, i: usize, f: &mut F) {
        let g = &mut self.spawn.g;
        for k in 0..4 {
            f.u8(0xE0 + k as u32, ["census_prev0", "census_prev1", "census_prev2", "census_prev3"][k], &mut g.census_prev[i][k]);
            f.u8(0xE4 + k as u32, ["census0", "census1", "census2", "census3"][k], &mut g.census[i][k]);
        }
        f.u8(0x10B, "kills", &mut g.kills[i]);
        let j = &mut self.flow.join[i];
        f.u8(0xA2, "join_state", &mut j.state);
        f.u8(0xB0, "join_seconds", &mut j.seconds);
        f.u8(0xB1, "join_frames", &mut j.frames);
        let s = &mut self.hud.stats[i];
        f.u32(0x110, "score_prev", &mut s.score_prev);
        f.u8(0x114, "low_hp", &mut s.low_hp);
    }

    /// Pool free lists: count, stack pointer and the stack (the next allocation first). The
    /// pointer word holds the low word of the absolute address (0x8000 + G offset).
    fn trace_free_lists(&mut self, v: &mut dyn Visit) {
        let a = self.assets;
        let names_only = v.all();
        let loading = v.loading();
        macro_rules! fl {
            ($pool:expr, $k:expr) => {{
                let (cnt, ptr, end) = FREE_LISTS[$k];
                let mut f = F::new(v, g(0), a);
                let len = $pool.free.len() as u32;
                let mut count = len as u16;
                f.u16(cnt, "free_count", &mut count);
                let mut p = (0x8000 + end - 2 * len) as u16;
                f.u16(ptr, "free_ptr", &mut p);
                if loading {
                    let top = (p as u32).wrapping_sub(0x8000);
                    let mut free = Vec::new();
                    if top <= end && end - top <= 2 * $pool.slots.len() as u32 {
                        let mut q = end;
                        while q > top {
                            q -= 2;
                            let mut h = 0u16;
                            f.u16(q, "free_slot", &mut h);
                            match $pool.index_of(h) {
                                Some(i) => free.push(i),
                                None => f.v.error(format!("free list G.${:X}: {h:#06x} is not a slot of this pool", q)),
                            }
                        }
                    } else if p != 0 || count != 0 {
                        f.v.error(format!("free list pointer G.${ptr:X} = {p:#06x} out of range"));
                    }
                    if free.len() != count as usize {
                        f.v.error(format!("free list G.${cnt:X}: count {count} but {} entries", free.len()));
                    }
                    $pool.free = free;
                } else if names_only {
                    for k in 0..$pool.slots.len() as u32 {
                        let mut h = 0u16;
                        f.u16(end - 2 - 2 * k, "free_slot", &mut h);
                    }
                } else {
                    for (k, &i) in $pool.free.iter().enumerate() {
                        let mut h = $pool.handle(i);
                        f.u16(end - 2 - 2 * k as u32, "free_slot", &mut h);
                    }
                }
            }};
        }
        fl!(self.items, 0);
        fl!(self.props, 1);
        fl!(self.enemies, 2);
        fl!(self.effects, 3);
        fl!(self.specials, 4);
    }

    /// Globals (`G.$x`), by owner.
    fn trace_globals(&mut self, v: &mut dyn Visit) {
        let a = self.assets;
        let mut f = F::new(v, g(0), a);
        let f = &mut f;
        // World (main session).
        let w = &mut self.w;
        f.u16(0x0000, "frame", &mut w.frame);
        f.u16(0x0014, "rng", &mut w.rng);
        f.u8(0x0016, "input_p1", &mut w.input[0]);
        f.u8(0x0017, "input_p2", &mut w.input[1]);
        f.u8(0x0018, "input_p3", &mut w.input[2]);
        if f.loading() {
            // Cooked by the vblank IRQ (0x738); the harness feeds them like the input bytes.
            f.u8(0x001A, "system", &mut w.system);
            f.u8(0x0020, "coins", &mut w.coins);
        }
        f.u16(0x0026, "video_ctrl", &mut w.video_ctrl);
        f.u16(0x0028, "layer_ctrl", &mut w.layer_ctrl);
        f.u8(0x04D4, "stage_clear", &mut w.stage_clear);
        f.u8(0x04D9, "stage", &mut w.stage);
        f.u8(0x04DC, "vertical_scene", &mut w.vertical_scene);
        f.u8(0x04DD, "scripted", &mut w.scripted);
        f.u8(0x04F0, "floor_only", &mut w.floor_only);
        f.u8(0x0506, "no_clamp", &mut w.no_clamp);
        f.u8(0x04EF, "players_minus_one", &mut w.players_minus_one);
        f.u16(0x05EE, "level", &mut w.level);
        f.u16(0x05F0, "rank", &mut w.rank);
        f.u8(0x67BE, "attack_serial", &mut w.attack_serial);
        // Rank struct G.$5EC and the time limit (enemy port).
        let r = &mut self.rank;
        f.u16(0x05EC, "aggression", &mut r.aggression);
        f.u16(0x05F2, "force_x", &mut r.force_x);
        f.u16(0x05F4, "force_val", &mut r.force_val);
        f.u16(0x05F6, "rank_speed", &mut r.speed);
        f.u16(0x05F8, "rank_timer", &mut r.timer);
        f.u16(0x05FA, "rank_ups", &mut r.ups);
        f.u16(0x05FC, "rank_downs", &mut r.downs);
        f.u16(0x05FE, "rank_ups_t", &mut r.ups_t);
        f.u16(0x0600, "rank_downs_t", &mut r.downs_t);
        f.u16(0x0602, "rank_freeze", &mut r.freeze);
        f.u16(0x0604, "w604", &mut r.w604);
        f.u16(0x0606, "w606", &mut r.w606);
        f.u16(0x0608, "calm", &mut r.calm);
        f.u16(0x04F6, "w4f6", &mut r.w4f6);
        f.u8(0x04E8, "time_min", &mut r.time_min);
        f.u8(0x04E9, "time_sec", &mut r.time_sec);
        f.u8(0x04EA, "time_div", &mut r.time_div);
        f.u8(0x04EB, "time_lock", &mut r.time_lock);
        if f.loading() {
            r.started = true;
        }
        // Enemy-side globals (spawn.rs).
        let s = &mut self.spawn.g;
        f.u8(0x043A, "snd_toggle", &mut self.w.snd_toggle);
        f.u8(0x043B, "snd_toggle_b", &mut s.snd_toggle_b);
        f.u16(0x043C, "snd_hit_toggle", &mut self.w.snd_hit_toggle);
        f.u8(0x04F3, "alive", &mut s.alive);
        f.u8(0x04FB, "alive_run", &mut s.alive_run);
        f.u8(0x050E, "kill_all", &mut s.kill_all);
        f.u16(0x04E0, "killer", &mut s.killer);
        f.u16(0x050C, "wave_timer", &mut s.wave_timer);
        f.u16(0x04FE, "go_timer", &mut s.go_timer);
        f.u16(0x0500, "go_camera_x", &mut s.go_camera_x);
        f.u8(0x0518, "car_mode", &mut s.car_mode);
        f.u8(0x0502, "b502", &mut s.b502);
        f.u8(0x04D5, "b4d5", &mut s.b4d5);
        f.u8(0x04F1, "b4f1", &mut s.b4f1);
        f.u8(0x04FA, "b4fa", &mut s.b4fa);
        f.u8(0x04FC, "b4fc", &mut s.b4fc);
        f.u8(0x0504, "b504", &mut s.b504);
        f.u8(0x0505, "b505", &mut s.b505);
        f.u8(0x04F9, "dizzy_prev", &mut s.dizzy_prev);
        f.u8(0x04F8, "dizzy", &mut s.dizzy);
        f.u8(0x04DE, "b4de", &mut s.b4de);
        f.u8(0x07B2, "b7b2", &mut s.b7b2);
        // Spawn interpreter G.$66C: +$04 init, +$08 / +$0C list pointers (indexes in the port).
        f.u8(0x0670, "spawn_init", &mut self.spawn.init);
        let st = self.w.stage as usize;
        let area = self.cam.s2.area as usize & 3;
        if let Some(sc) = a.enemy.stages.get(st).and_then(|s| s.areas.get(area)) {
            let mut p1 = sc.list1_addr + 14 * self.spawn.l1 as u32;
            let mut p2 = sc.list2_addr + 16 * self.spawn.l2 as u32;
            f.u32(0x0674, "spawn_list1", &mut p1);
            f.u32(0x0678, "spawn_list2", &mut p2);
            if f.loading() {
                self.spawn.l1 = (p1.wrapping_sub(sc.list1_addr) / 14) as usize;
                self.spawn.l2 = (p2.wrapping_sub(sc.list2_addr) / 16) as usize;
                self.spawn.area = self.cam.s2.area;
            }
        }
        // Camera (stage port): scroll2 controller G.$73C, scroll3 G.$7DC, scroll1 G.$6C4.
        self.trace_camera(f);
        self.trace_terrain(f);
        self.trace_item_list(f);
        // Sprite list builder state.
        f.u16(n(0x7126), "splash_timer", &mut self.sprites.splash_timer);
        f.u16(n(0x7124), "splash_phase", &mut self.sprites.splash_phase);
        // Fade control block G.$44A.
        f.u8(0x044A, "fade_mask", &mut self.fade.mask);
        f.u8(0x044B, "fade_steps", &mut self.fade.steps);
        f.u16(0x044C, "fade_delay", &mut self.fade.delay);
        self.trace_flow(f);
        self.trace_hud(f);
    }

    /// Terrain collision state (`terrain.rs`): floor list pointer G.$6AC (a ROM address; the port
    /// keeps the segment index), G.$6B0, and the obstacle lists G.$6A22 / $6A26 with their storage.
    fn trace_terrain(&mut self, f: &mut F) {
        let (list, len) = (self.terrain_list_addr(), self.terrain_list_len());
        let (vertical, scene) = (self.w.vertical_scene != 0, &self.assets.terrain.scene_map);
        let t = &mut self.terrain;
        let mut p = list + 12 * t.seg as u32;
        f.u32(0x06AC, "floor_ptr", &mut p);
        if f.loading() {
            t.seg = if list == 0 { 0 } else { (p.wrapping_sub(list) / 12) as usize };
            t.seg = t.seg.min(len.saturating_sub(1));
        }
        f.i16(0x06B0, "floor_base", &mut t.base);
        if f.loading() && t.scene.is_empty() && vertical {
            t.scene = scene.clone(); // 0x12C50 drew it when the area started
        }
        // The list pointers hold the low word of the address of the stack top.
        f.u16(0x6A22, "obst_count", &mut t.count);
        let mut p = 0xEA5Au16.wrapping_sub(2 * t.count);
        f.u16(0x6A24, "obst_ptr", &mut p);
        f.u16(0x6A26, "obst_prev_count", &mut t.prev);
        let mut p = 0xEA5Au16.wrapping_sub(2 * t.prev);
        f.u16(0x6A28, "obst_prev_ptr", &mut p);
        for k in 0..super::terrain::OBSTACLE_SLOTS {
            f.u16(0x6A58 - 2 * k as u32, "obst_slot", &mut t.buf[k]);
        }
    }

    /// The pickup list G.$69EE (count), $69F0 (pointer), the stack below $6A22 (pool E update).
    /// Only the entries in use are traced (the stack keeps stale words). Items registered by the
    /// previous frame are what `find_pickup` reads; loading it makes `--step-from` runs see them.
    fn trace_item_list(&mut self, f: &mut F) {
        let list = self.combat.get(0x69EE).to_vec();
        let n = list.len();
        let mut count = n as u16;
        f.u16(0x69EE, "item_count", &mut count);
        let mut ptr = 0xEA22u16.wrapping_sub(2 * count);
        f.u16(0x69F0, "item_ptr", &mut ptr);
        let used = if f.all() { 24 } else { (count as usize).min(24) };
        let mut loaded = Vec::new();
        for k in 0..used {
            // The first registration is stored at the top (address $6A20), the next below it.
            let mut h = if k < n { list[n - 1 - k] } else { 0 };
            f.u16(0x6A20 - 2 * k as u32, "item_slot", &mut h);
            loaded.push(h);
        }
        if f.loading() {
            self.combat.clear(0x69EE);
            for &h in loaded.iter().take((count as usize).min(24)) {
                self.combat.push(0x69EE, h);
            }
        }
    }

    fn trace_camera(&mut self, f: &mut F) {
        let s = &mut self.cam.s2;
        let b = 0x073C;
        f.u8(b + 0x04, "cam2_state", &mut s.state);
        f.u8(b + 0x05, "cam2_mode", &mut s.mode);
        f.u8(b + 0x06, "cam2_sub", &mut s.sub);
        f.u16(b + 0x08, "cam_x", &mut s.cam_x);
        f.u16(b + 0x0A, "cam_x_frac", &mut s.cam_x_frac);
        f.u16(b + 0x0C, "cam_y", &mut s.cam_y);
        f.u16(b + 0x0E, "cam_y_frac", &mut s.cam_y_frac);
        f.u16(b + 0x10, "cam_z", &mut s.cam_z);
        f.u16(b + 0x12, "cam_z_frac", &mut s.cam_z_frac);
        f.u16(b + 0x14, "cam2_w14", &mut s.w14);
        f.u16(b + 0x16, "cam2_w16", &mut s.w16);
        f.u16(b + 0x18, "cam2_w18", &mut s.w18);
        f.i16(b + 0x1A, "cam2_a1a", &mut s.a1a);
        f.i16(b + 0x1C, "cam2_a1c", &mut s.a1c);
        f.i16(b + 0x1E, "cam2_a1e", &mut s.a1e);
        f.u16(b + 0x46, "cam2_sx46", &mut s.sx46);
        f.u16(b + 0x48, "cam2_sy48", &mut s.sy48);
        f.u8(b + 0x4A, "area", &mut s.area);
        f.u8(b + 0x4B, "vfollow", &mut s.vfollow);
        f.u16(b + 0x4C, "event_x", &mut s.x4c);
        f.u16(b + 0x4E, "event_y", &mut s.y4e);
        f.u16(b + 0x50, "push50", &mut s.push50);
        f.u16(b + 0x52, "up52", &mut s.up52);
        f.u16(b + 0x54, "down54", &mut s.down54);
        let ev_base = self.assets.stages.stages.get(self.w.stage as usize).map(|x| x.camera_events_addr).unwrap_or(0);
        let mut p = ev_base + 16 * s.next_event as u32;
        f.u32(b + 0x56, "next_event", &mut p);
        if f.loading() {
            s.next_event = (p.wrapping_sub(ev_base) / 16) as usize;
            s.map_area = s.area;
        }
        // +$42: the screen array of the scroll2 map. Stage 2 has two copies of the map header (the controller's own, 0x22AFA, and the
        // table's, used by the demo): the pointer tells which one was loaded last (`Scroll2::own_map`).
        if let Some(own) = self.assets.stages.stages.get(2).and_then(|x| x.controller_map.as_ref()) {
            if f.all() || (self.w.stage == 2 && s.state != 0) {
                let table = self.assets.stages.stages[2].areas.get(s.map_area as usize).map_or(0, |a| a.scroll2.screens_addr);
                let mut p = if s.own_map { own.screens_addr } else { table };
                f.u32(b + 0x42, "map_ptr", &mut p);
                if f.loading() {
                    s.own_map = p == own.screens_addr;
                }
            }
        }
        f.u16(b + 0x5A, "cam2_x5a", &mut s.x5a);
        f.u16(b + 0x5C, "cam2_y5c", &mut s.y5c);
        f.u16(b + 0x60, "max_x60", &mut s.max_x60);
        f.u16(b + 0x62, "max_y62", &mut s.max_y62);
        f.u16(b + 0x64, "min_x64", &mut s.min_x64);
        f.u16(b + 0x66, "min_y66", &mut s.min_y66);
        f.u8(b + 0x6C, "event_idx", &mut s.event_idx);
        f.u8(b + 0x6D, "go", &mut s.go);
        f.u8(b + 0x70, "flag70", &mut s.flag70);
        f.u8(b + 0x71, "t71", &mut s.t71);
        f.u8(b + 0x72, "phase72", &mut s.phase72);
        f.u8(b + 0x73, "prev73", &mut s.prev73);
        f.u8(b + 0x75, "t75", &mut s.t75);
        f.u8(b + 0x78, "lock78", &mut s.lock78);
        f.u8(b + 0x79, "by_y79", &mut s.by_y79);
        f.u8(b + 0x7B, "next_mode7b", &mut s.next_mode7b);
        // Stage 2: the row scroll task G.$7780 (state +$04, base word +$08) and the car object of the drive, G.$51A.
        f.u8(0x7784, "rowscroll_state", &mut self.cam.row.state);
        f.u16(0x7788, "rowscroll_base", &mut self.cam.row.base8);
        f.u16(0x051A, "car_handle", &mut self.cam.car);
        let t = &mut self.cam.s3;
        let b = 0x07DC;
        f.u8(b + 0x04, "cam3_state", &mut t.state);
        f.u8(b + 0x05, "cam3_b5", &mut t.b5);
        f.u16(b + 0x08, "cam3_x", &mut t.cam_x);
        f.u16(b + 0x0C, "cam3_y", &mut t.cam_y);
        f.u16(b + 0x46, "cam3_sx46", &mut t.sx46);
        f.u16(b + 0x48, "cam3_sy48", &mut t.sy48);
        f.u16(b + 0x5A, "cam3_x5a", &mut t.x5a);
        f.u16(b + 0x5C, "cam3_y5c", &mut t.y5c);
        f.u8(b + 0x5F, "cam3_shift5f", &mut t.shift5f);
        if f.loading() {
            t.map_area = self.cam.s2.area;
        }
        f.u16(0x06C4, "cam1_x", &mut self.cam.cam1_x);
        f.u16(0x06C8, "cam1_y", &mut self.cam.cam1_y);
        f.u8(0x755A, "shake_state", &mut self.cam.shake_state);
        f.u8(0x755C, "shake_req", &mut self.cam.shake_req);
        f.u8(0x755D, "shake_strength", &mut self.cam.shake_strength);
        f.u8(0x755E, "shake_timer", &mut self.cam.shake_timer);
        let mut sign = self.cam.shake_sign as u8;
        f.u8(0x755F, "shake_sign", &mut sign);
        self.cam.shake_sign = sign as i8;
        f.i16(0x7560, "shake_amp", &mut self.cam.shake_amp);
        f.u16(0x07B8, "shake_scroll1_y", &mut self.cam.shake_scroll1_y);
        let mut y1 = self.cam.cam1_y.wrapping_add(self.cam.shake_dy1 as u16);
        let mut y2 = self.cam.s2.cam_y.wrapping_add(self.cam.shake_dy2 as u16);
        f.u16(0x0860, "display_y1", &mut y1);
        f.u16(0x0868, "display_y2", &mut y2);
        if f.loading() {
            self.cam.shake_dy1 = y1.wrapping_sub(self.cam.cam1_y) as i16;
            self.cam.shake_dy2 = y2.wrapping_sub(self.cam.s2.cam_y) as i16;
        }
    }

    fn trace_flow(&mut self, f: &mut F) {
        let fl = &mut self.flow;
        // 0x6C4A reads the system task's credit/debounce state between snapshots.
        let s = &mut fl.front.sys;
        f.u8(0x046A, "credits", &mut s.credits);
        f.flag(0x046B, "free_play", &mut s.free_play);
        f.flag(0x046C, "two_coin_start", &mut s.two_coin_start);
        f.u8(0x046D, "press_up", &mut s.press_up);
        f.u8(0x046E, "coin_pulse", &mut s.pulse);
        f.u8(0x046F, "coin_pulse_timer", &mut s.pulse_timer);
        f.u8(0x0470, "coins_per_credit", &mut s.coins_per_credit);
        f.u8(0x0471, "credits_per_coin", &mut s.credits_per_coin);
        f.flag(0x0472, "common_chute", &mut s.common);
        f.flag(0x0473, "credit_redraw", &mut s.redraw);
        f.flag(0x0474, "two_player_chute", &mut s.two_player);
        f.u8(0x0475, "common_coins", &mut s.common_coins);
        f.u8(0x0476, "common_prev", &mut s.common_prev);
        f.u8(0x0477, "prompt_hold", &mut s.hold);
        for (i, slot) in s.slots.iter_mut().enumerate() {
            let at = 0x047A + 16 * i as u32;
            f.u8(at, "coin_timer", &mut slot.timer);
            f.u8(at + 1, "slot_coins", &mut slot.coins);
            f.u8(at + 2, "coin_shift", &mut slot.shift);
            f.u8(at + 3, "slot_credits", &mut slot.credits);
            f.u8(at + 4, "slot_prev", &mut slot.prev);
        }
        f.u8(0x04CC, "in_game", &mut fl.in_game);
        f.u8(0x04CD, "started", &mut fl.started);
        f.u8(0x04E4, "chosen", &mut fl.chosen);
        f.u8(0x04CF, "b4cf", &mut fl.b4cf);
        f.u8(0x04D0, "game_mode", &mut fl.mode);
        f.u8(0x04D1, "game_sub", &mut fl.sub);
        f.u16(0x04D2, "game_sub2", &mut fl.sub2);
        f.u8(0x04D6, "game_speed", &mut fl.speed);
        f.u8(0x04D7, "b4d7", &mut fl.b4d7);
        f.u8(0x04DB, "b4db", &mut fl.b4db);
        f.u8(0x0507, "b507", &mut fl.b507);
        f.u8(0x04DA, "game_frame", &mut fl.frame);
        f.u8(0x04E2, "b4e2", &mut fl.b4e2);
        f.u8(0x04E3, "b4e3", &mut fl.b4e3);
        // The globals of the scripted control (`script.rs`).
        let sg = &mut fl.script;
        f.u8(0x04D8, "script_b4d8", &mut sg.b4d8);
        f.u8(0x04DF, "script_b4df", &mut sg.b4df);
        for k in 0..3 {
            f.u8(0x04EC + k as u32, ["script_taken0", "script_taken1", "script_taken2"][k], &mut sg.taken[k]);
        }
        f.u8(0x04FD, "script_b4fd", &mut sg.b4fd);
        f.u8(0x0503, "script_b503", &mut sg.b503);
        f.u8(0x0514, "script_b514", &mut sg.b514);
        f.u8(0x0516, "script_b516", &mut sg.b516);
        f.u8(n(0x70AA), "script_b70aa", &mut sg.b70aa);
        f.u8(n(0x70A9), "script_b70a9", &mut sg.b70a9);
        f.u8(0x04E6, "b4e6", &mut fl.b4e6);
        f.u8(0x04E5, "b4e5", &mut fl.b4e5);
        f.u16(0x0508, "timer508", &mut fl.timer508);
        for k in 0..4 {
            f.u8(0x0510 + k as u32, ["saved_mode0", "saved_mode1", "saved_mode2", "saved_mode3"][k], &mut fl.saved_mode[k]);
        }
        for k in 0..0x1E {
            f.u8(0x05CC + k as u32, "continue", &mut fl.cont.raw.0[k]);
        }
        f.u8(0x777A, "dip_a", &mut fl.dip.dip_a);
        f.u8(0x777B, "dip_b", &mut fl.dip.dip_b);
        f.u8(0x777C, "dip_c", &mut fl.dip.dip_c);
        if f.loading() {
            self.rank.dip = fl.dip.dip_b;
        }
        let sel = &mut fl.front.select;
        f.u8(0x062F, "select_taken", &mut sel.taken);
        f.u8(0x0635, "select_seconds", &mut sel.seconds);
        f.u8(0x0636, "select_frames", &mut sel.frames);
        f.u8(n(0x702E), "game_over_flag", &mut fl.front.game_over_flag);
        // The struct G.$735E (the task itself: task slot 0x30, see trace_load_tasks) belongs to the stage clear task (the tally, a byte image)
        // while the game is in its tally steps (G.$4D1 = 6) and until the stage intro task of the next stage starts, else to the stage intro task.
        if f.loading() {
            fl.tally.owner = fl.sub >= 6 && fl.mode == 0xC;
        }
        if fl.tally.owner {
            for k in 0..super::flow::tally::SIZE {
                let mut v = fl.tally.raw.b(k);
                f.u8(0x735E + k as u32, "tally", &mut v);
                fl.tally.raw.sb(k, v);
            }
        } else {
            let it = &mut fl.intro;
            f.u8(0x735E, "intro_state", &mut it.state);
            f.u8(0x735F, "intro_sub", &mut it.sub);
            f.u8(0x7360, "intro_b2", &mut it.b2);
            f.u8(0x7361, "intro_b3", &mut it.b3);
            f.u16(0x7362, "intro_w4", &mut it.w4);
            f.u16(0x7364, "intro_w6", &mut it.w6);
            f.u16(0x7366, "intro_w8", &mut it.w8);
            f.u32(0x7368, "intro_src", &mut it.src);
            f.u32(0x736C, "intro_dst", &mut it.dst);
        }
    }

    fn trace_hud(&mut self, f: &mut F) {
        let h = &mut self.hud;
        for (i, s) in h.slots.iter_mut().enumerate() {
            let b = 0x6BA8 + 0x10 * i as u32;
            f.u8(b, "hud_idx", &mut s.idx);
            f.u8(b + 1, "hud_delay", &mut s.delay);
            f.u8(b + 2, "hud_step", &mut s.step);
            f.u8(b + 3, "hud_timer", &mut s.timer);
            f.u8(b + 4, "hud_kind", &mut s.kind);
            f.u8(b + 5, "hud_blink", &mut s.blink);
            f.u16(b + 8, "hud_tracked", &mut s.tracked);
            f.u32(b + 0xA, "hud_base", &mut s.base);
            f.u8(b + 0xE, "hud_saved_kind", &mut s.saved_kind);
            f.u8(b + 0xF, "hud_saved_id", &mut s.saved_id);
        }
        let c = &mut h.cur;
        f.u8(0x6B9C, "hudcur_kind", &mut c.kind);
        f.u8(0x6B9D, "hudcur_id", &mut c.id);
        f.i16(0x6B9E, "hudcur_segs", &mut c.segs);
        f.u16(0x6BA0, "hudcur_attr", &mut c.attr);
        f.u16(0x6BA2, "hudcur_extra", &mut c.extra);
        f.i16(0x6BA6, "hudcur_segs_total", &mut c.segs_total);
        for i in 0..3 {
            f.u8(0x6BD8 + 2 * i as u32, "hud_ring_w", &mut h.ring_w[i]);
            f.u8(0x6BD9 + 2 * i as u32, "hud_ring_r", &mut h.ring_r[i]);
            for m in 0..16 {
                for w in 0..8 {
                    f.u16(0x6BDE + 0x100 * i as u32 + 16 * m as u32 + 2 * w as u32, "hud_ring_msg", &mut h.rings[i][m][w]);
                }
            }
        }
        f.u16(n(0x70BC), "hud_boss", &mut h.boss);
        for w in 0..8 {
            f.u16(n(0x70BA) + 2 * w as u32, "hud_boss_msg", &mut h.boss_msg[w]);
        }
        f.u32(n(0x70AE), "hud_bar_start", &mut h.bar_start);
        f.u16(n(0x7098), "weapon_dirty", &mut h.weapon_dirty);
        f.flag(n(0x7134), "score_dirty", &mut h.score_dirty);
        for i in 0..3 {
            f.flag(n(0x7133) + i as u32, "score_dirty_p", &mut h.score_dirty_p[i]);
        }
        f.u8(0x050A, "blink_phase", &mut h.blink_phase);
        f.u8(0x050B, "blink_timer", &mut h.blink_timer);
        for (k, e) in h.hiscores.iter_mut().enumerate() {
            let b = 0x703E + 16 * k as u32;
            f.u32(b, "hiscore", &mut e.score);
            for n in 0..4 {
                f.u8(b + 4 + n as u32, "hiscore_name", &mut e.name[n]);
            }
            f.u8(b + 8, "hiscore_chr", &mut e.chr);
            f.u8(b + 9, "hiscore_stage", &mut e.stage);
            f.u8(b + 0xA, "hiscore_b0a", &mut e.b0a);
        }
    }

    // --------------------------------------------------------------------------------------
    // Loading a snapshot
    // --------------------------------------------------------------------------------------

    /// Fills the port from a MAME snapshot: `ram` = 64 KB at 0xFF0000, `gfx` = 192 KB at
    /// 0x900000. Returns warnings for state the port could not take over.
    pub fn trace_load(&mut self, ram: &[u8], gfx: &[u8]) -> Vec<String> {
        let mut l = Load { ram, errors: Vec::new(), unmapped: Unmapped::new() };
        // Live bytes first: object structs are visited fully only when live.
        for (pool_i, (_, (base, stride, n))) in POOLS.iter().enumerate().skip(1) {
            for i in 0..*n {
                let h = base + stride * i as u16;
                let live = l.read(obj(h), 1) as u8;
                match pool_i {
                    1 => self.enemies.slots[i].b.live = live,
                    2 => self.props.slots[i].b.live = live,
                    3 => self.items.slots[i].b.live = live,
                    4 => self.effects.slots[i].b.live = live,
                    _ => self.specials.slots[i].b.live = live,
                }
            }
        }
        // Stage and area first. The visit reaches the objects before the globals, and it turns ROM
        // pointers into indexes with the tables of the stage and area: the wave controllers' +$A0
        // (`Prop::trace_visit`), the spawn lists G.$674 / $678 and the camera event pointer. With
        // the stale values of a new game these were read in the tables of stage 0 area 0, so any
        // other episode or area differed at frame -1.
        self.w.stage = l.read(g(0x04D9), 1) as u8;
        self.cam.s2.area = l.read(g(0x073C + 0x4A), 1) as u8;
        self.trace_visit(&mut l);
        self.trace_unmapped = l.unmapped;
        let mut warn = l.errors;
        self.trace_load_tasks(ram, &mut warn);
        self.trace_load_gfx(gfx);
        // Index/count conversions cannot encode null pointers before the corresponding
        // subsystem has initialized. Remember only these nulls, never live ROM pointers.
        let mut canonical = Collect::default();
        self.trace_visit(&mut canonical);
        let mut dormant = DormantPointers { phase: self.trace_pointer_phase(), fields: Vec::new() };
        for (off, size) in [
            (0x8674, 4), (0x8678, 4), (0x86AC, 4), (0x8792, 4),
            (0xE5FE, 2), (0xE600, 2), (0xE602, 2), (0xE604, 2), (0xE606, 2),
            (0xE9F0, 2), (0xEA24, 2), (0xEA28, 2),
        ] {
            if ram.get(off..off + size).is_some_and(|v| v.iter().all(|b| *b == 0))
                && canonical.ram[off..off + size].iter().any(|b| *b != 0)
            {
                dormant.fields.push((off, size, canonical.ram[off..off + size].to_vec()));
            }
        }
        self.spawn.g.trace_dormant = dormant;
        warn
    }

    /// Harness (step runs with `DINO_STEP_PLAYERS`): load the three player objects (and the
    /// per-player fields kept by other ports) from the MAME RAM image of the frame being run, so
    /// that the objects updated after the players (pool E, A, ...) see the players exactly as
    /// the original had them in that frame, whatever the player port computed itself.
    pub fn trace_load_players(&mut self, ram: &[u8]) {
        let a = self.assets;
        let mut l = Load { ram, errors: Vec::new(), unmapped: self.trace_unmapped.clone() };
        for i in 0..3 {
            let mut f = F::new(&mut l, obj(Game::player_handle(i)), a);
            self.players[i].trace_visit(&mut f);
            self.trace_player_extras(i, &mut f);
        }
        self.trace_unmapped = l.unmapped;
    }

    /// Abstract state the port keeps instead of the task table (0xFF0052, 16 bytes per slot:
    /// +0 state, +1 sleep counter).
    fn trace_load_tasks(&mut self, ram: &[u8], warn: &mut Vec<String>) {
        let task = |slot: usize| (ram[0x52 + 16 * slot], ram[0x53 + 16 * slot]);
        // Slot 2 (0x20): the game task sleeps G.$4D6 & 15 frames between runs.
        let (st, cnt) = task(2);
        self.flow.cont.return_pending = self.flow.mode == 0x10 && self.flow.sub == 6 && self.flow.sub2 == 0 && st == 8;
        self.flow.wait = if st == 1 { cnt } else { 0 };
        // Slot 3 (0x30): stage intro or stage clear task. Both keep their state in the struct G.$735E (loaded with the globals). The
        // intro task (0xB9BE) is the one while the level runs and the stage is not cleared (game mode 0xC, G.$4D1 = 0); it is taken
        // over with the sleep counter of the snapshot. The clear task (0xB97E) is not (the tally is M2-28).
        let (st, cnt) = task(3);
        self.flow.intro.live = false;
        self.flow.tally.live = false;
        if st != 0 {
            if self.flow.tally.owner && self.flow.mode == 0xC {
                // The clear task: the position inside its routine follows from the sleep counter (a counter above 1 is a sleep inside a
                // routine of the tally, 0xA7E or 0xA82), the state and the sub step (a task is made and runs for the first time in the same frame, so a snapshot never holds one that has not run).
                use super::flow::tally::After;
                let (state, sub) = (self.flow.tally.raw.b(0), self.flow.tally.raw.b(1));
                let t = &mut self.flow.tally;
                t.live = true;
                t.sleep = if st == 1 { cnt as u16 } else { 1 };
                t.after = if cnt > 1 {
                    match (state, sub) {
                        (2, 0xA) => After::AdvanceTotal,
                        (2, _) => After::Advance,
                        (6, _) => After::Resume,
                        _ => After::Loop,
                    }
                } else {
                    After::Loop
                };
            } else if self.flow.mode == 0xC && self.flow.sub == 0 && self.flow.intro.state <= 6 {
                let it = &mut self.flow.intro;
                it.live = true;
                it.sleep = if st == 1 { cnt as u16 } else { 1 };
                it.init = false;
                // The task sleeps once inside 0xCC58 (`jsr 0xA76`, then the HUD redraw 0x162AE): in the first frame after it saw G.$4E3 the
                // sleep counter of the snapshot is the same as in the next one. The redraw is still to come while the rank of the hero
                // (+$10A, 0x33 at the start, set by the redraw) is untouched.
                it.after = if it.state == 4
                    && it.sub == 4
                    && it.w6 == 0
                    && self.players.iter().any(|p| p.b.live != 0 && p.b10a == 0x33)
                {
                    super::flow::IntroAfter::Redraw
                } else {
                    super::flow::IntroAfter::Loop
                };
            } else {
                warn.push(format!("task slot 0x30 (intro / clear) is active (state {st}, sleep {cnt}): not taken over"));
            }
        }
        // Slot 4 (0x40): fade task.
        let (st, cnt) = task(4);
        self.fade.task = None;
        if st != 0 {
            if self.flow.tally.live {
                // The fade of the clear tally: down (0x8854) in its first sub steps, up (0x8838) from the clean up (sub 0xE). The kernel counter is
                // the sleep between two steps; the step count, the delay and the mask are the globals G.$44B, G.$44C and G.$44A.
                let fade_in = self.flow.tally.raw.b(1) >= 0xE;
                self.fade.task = Some(crate::game::palette::FadeTask { fade_in, sleep: cnt as u16, started: true });
            } else {
                warn.push(format!("task slot 0x40 (fade) is active (state {st}): not taken over"));
            }
        }
        // The next vblank copies the snapshot's current inputs to the previous
        // bytes before sampling the new frame (0x738).
        self.flow.front.prev_system = ram[0x801A];
        self.flow.front.prev_input.copy_from_slice(&ram[0x8016..0x8019]);
        self.flow.region.0 = u16::from_be_bytes([ram[0x8032], ram[0x8033]]);
        // Slot 0 runs the system task, slot 1 the in-game join task (0x6C4A).
        for (slot, scene) in [(0, &mut self.flow.front.sys_task), (1, &mut self.flow.join_task)] {
            let (st, cnt) = task(slot);
            *scene = super::flow::scene::Scene::default();
            if self.flow.in_game != 0 && st != 0 {
                *scene = super::flow::scene::Scene::start(0);
                if st == 1 && cnt != 0 {
                    scene.sleep(cnt as u16);
                } else if st == 2 {
                    scene.sleep(0);
                }
            }
        }
    }

    /// Tilemaps and palettes from GFX RAM. Bases: scroll1 0x908000, scroll2 0x90C000,
    /// scroll3 0x910000, palettes 0x914000 (the in-game CPS-A settings).
    fn trace_load_gfx(&mut self, gfx: &[u8]) {
        let w = |o: usize| u16::from_be_bytes([gfx[o], gfx[o + 1]]);
        let scene = self.flow.mode == 0x10 && self.flow.sub != 0;
        if scene {
            for (cells, base) in [(&mut self.flow.cont.level2, 0xC000usize), (&mut self.flow.cont.level3, 0x10000)] {
                *cells = (0..0x1000).map(|i| dino_assets::stage::Cell { code: w(base + 4*i), attr: w(base + 4*i + 2) }).collect();
            }
        } else {
            self.flow.cont.level2.clear();
            self.flow.cont.level3.clear();
        }
        for (layer, base) in [(0, 0x8000usize), (1, if scene {0x1C000} else {0xC000}), (2, if scene {0x20000} else {0x10000})] {
            let cells = match layer {
                0 => &mut self.vram.scroll1,
                1 => &mut self.vram.scroll2,
                _ => &mut self.vram.scroll3,
            };
            for (i, c) in cells.iter_mut().enumerate() {
                c.code = w(base + 4 * i);
                c.attr = w(base + 4 * i + 2);
            }
        }
        for (i, p) in self.pal.words.iter_mut().enumerate() {
            *p = w(0x14000 + 2 * i);
        }
        self.pal.dirty = true;
        // the row scroll RAM 0x915800 .. 0x918000
        for (i, r) in self.vram.rowscroll.iter_mut().enumerate() {
            *r = w(0x15800 + 2 * i);
        }
    }

    // --------------------------------------------------------------------------------------
    // Trace images
    // --------------------------------------------------------------------------------------

    fn trace_pointer_phase(&self) -> [u8; 6] {
        [self.flow.mode, self.flow.sub, self.flow.in_game, self.spawn.init, self.w.stage, self.cam.s2.area]
    }

    /// RAM image and mask of every modeled field.
    pub fn trace_ram(&mut self) -> Collect {
        let mut c = Collect { unmapped: std::mem::take(&mut self.trace_unmapped), ..Default::default() };
        self.trace_visit(&mut c);
        self.trace_unmapped = std::mem::take(&mut c.unmapped);
        let phase = self.trace_pointer_phase();
        let d = &mut self.spawn.g.trace_dormant;
        if phase != d.phase {
            d.fields.clear();
        }
        d.fields.retain(|(off, size, native)| {
            if c.ram[*off..*off + *size] != **native {
                return false;
            }
            c.ram[*off..*off + *size].fill(0);
            true
        });
        c
    }

    /// GFX RAM image (0x900000, 192 KB) and mask: the three tilemaps and the palettes.
    pub fn trace_gfx(&self) -> (Vec<u8>, Vec<u8>) {
        let mut img = vec![0u8; 0x30000];
        let mut mask = vec![0u8; 0x30000];
        let scene = self.flow.mode == 0x10 && self.flow.sub != 0;
        for (cells, base) in [(&self.vram.scroll1, 0x8000usize), (&self.vram.scroll2, if scene {0x1C000} else {0xC000}), (&self.vram.scroll3, if scene {0x20000} else {0x10000}),
            (&self.flow.cont.level2, 0xC000), (&self.flow.cont.level3, 0x10000)] {
            for (i, c) in cells.iter().enumerate() {
                let o = base + 4 * i;
                img[o..o + 2].copy_from_slice(&c.code.to_be_bytes());
                img[o + 2..o + 4].copy_from_slice(&c.attr.to_be_bytes());
            }
            if !cells.is_empty() { mask[base..base + 0x4000].fill(1); }
        }
        for (i, p) in self.pal.words.iter().enumerate() {
            img[0x14000 + 2 * i..0x14000 + 2 * i + 2].copy_from_slice(&p.to_be_bytes());
        }
        mask[0x14000..0x15000].fill(1);
        // the row scroll RAM 0x915800 .. 0x918000 (all zero outside stage 2 in the recordings)
        for (i, r) in self.vram.rowscroll.iter().enumerate() {
            img[0x15800 + 2 * i..0x15800 + 2 * i + 2].copy_from_slice(&r.to_be_bytes());
        }
        mask[0x15800..0x15800 + 2 * self.vram.rowscroll.len()].fill(1);
        (img, mask)
    }

    /// The sprite list built this frame, as the hardware entries (8 bytes each, up to 255) and
    /// the end marker the original writes (0x13FA6: X 0, Y 0x100, code 0, attr 0xFF00).
    pub fn trace_obj(&self) -> Vec<u8> {
        let mut v = vec![0u8; 0x800];
        if self.sprites.blank && self.sprites.entries.is_empty() {
            return v;
        }
        let n = self.sprites.entries.len().min(255);
        for (k, e) in self.sprites.entries.iter().take(n).enumerate() {
            let o = 8 * k;
            v[o..o + 2].copy_from_slice(&(e.x as u16).to_be_bytes());
            v[o + 2..o + 4].copy_from_slice(&(e.y as u16).to_be_bytes());
            v[o + 4..o + 6].copy_from_slice(&e.code.to_be_bytes());
            v[o + 6..o + 8].copy_from_slice(&e.attr.to_be_bytes());
        }
        let o = 8 * n;
        v[o..o + 8].copy_from_slice(&[0, 0, 1, 0, 0, 0, 0xFF, 0]);
        v
    }

    /// For each entry of the sprite list: the handle of the object that drew it (u16, big endian,
    /// 0 = none), so the diff tool can tell entries of unported classes from the rest.
    pub fn trace_obj_src(&self) -> Vec<u8> {
        let mut v = vec![0u8; 0x200];
        for (k, e) in self.sprites.entries.iter().take(255).enumerate() {
            v[2 * k..2 * k + 2].copy_from_slice(&e.src.to_be_bytes());
        }
        v
    }

    /// Field names for the trace header: per object kind (offset, size, name) and globals.
    pub fn trace_names(&mut self) -> (Vec<(&'static str, Vec<(u32, u8, &'static str)>)>, Vec<(u32, u8, &'static str)>) {
        let a = self.assets;
        let mut kinds = Vec::new();
        let mut one = |kind: &'static str, visit: &mut dyn FnMut(&mut F)| {
            let mut n = Names::default();
            let mut f = F::new(&mut n, 0, a);
            visit(&mut f);
            kinds.push((kind, n.list));
        };
        let mut p = Player::default();
        one("player", &mut |f| p.trace_visit(f));
        let mut e = Enemy::default();
        one("enemy", &mut |f| e.trace_visit(f));
        let mut pr = Prop::default();
        one("prop", &mut |f| pr.trace_visit(f, 0));
        let mut it = Item::default();
        one("item", &mut |f| it.trace_visit(f));
        let mut ef = Effect::default();
        one("effect", &mut |f| ef.trace_visit(f));
        let mut sp = Special::default();
        one("special", &mut |f| sp.trace_visit(f));
        // Player extras live in other structs.
        {
            let mut n = Names::default();
            let mut f = F::new(&mut n, 0, a);
            self.trace_player_extras(0, &mut f);
            kinds[0].1.extend(n.list);
        }
        let mut n = Names::default();
        self.trace_free_lists(&mut n);
        self.trace_globals(&mut n);
        (kinds, n.list)
    }
}

impl Game {
    /// Harness only. An object of a class the port does not run yet (an "unported" class) is not
    /// freed when the game was loaded from a snapshot: it stays as MAME had it, so its position
    /// and draw state show up in the trace and the free lists stay equal. It does not move or
    /// animate (that is the missing port). It still goes into the draw list if the snapshot had
    /// it on screen and its frame record is known. Returns false when the game was not loaded
    /// from a snapshot (the caller then frees the object, as the normal game does).
    /// `b` is the object's header (callers that took the object out of its slot pass their copy).
    pub fn trace_hold(&mut self, h: u16, b: &Body) -> bool {
        if !self.hold_unported {
            return false;
        }
        if b.visible != 0 && !self.trace_unmapped.contains_key(&(obj(h) + 0x28)) {
            self.draw_list.push(((b.zi() as u16 as u32) << 16) | h as u32); // 0x49C0
        }
        true
    }
}

// ------------------------------------------------------------------------------------------
// Pokes (`--poke`, probe sidecar files; `harness.rs` `PokeSet`)
// ------------------------------------------------------------------------------------------

/// Fields the trace keeps in another form than the port does (a pointer for an index or a frame
/// id, a list for a count). They are converted only when loading a snapshot, so a poke into them
/// would be dropped without a word. A poke into one of these is an error.
const POKE_DERIVED: &[&str] = &[
    "frame",
    "frame2",
    "wave_ptr",
    "free_count",
    "free_ptr",
    "free_slot",
    "spawn_list1",
    "spawn_list2",
    "next_event",
    "floor_ptr",
    "obst_ptr",
    "obst_prev_ptr",
    "item_count",
    "item_ptr",
    "item_slot",
];

/// Visits the fields once to write a RAM store (`size` bytes at `addr`, as the 68000 stores it)
/// into every modeled field it overlaps. With `apply` false nothing is written: the visit only
/// finds out whether the poke can be taken (every byte covered, no derived field).
struct PokeVisit {
    addr: u32,
    size: u8,
    value: u32,
    apply: bool,
    covered: [bool; 4],
    derived: Option<&'static str>,
}

impl Visit for PokeVisit {
    fn field(&mut self, addr: u32, size: u8, name: &'static str, v: &mut u32) {
        let (lo, hi) = (addr.max(self.addr), (addr + size as u32).min(self.addr + self.size as u32));
        if lo >= hi {
            return;
        }
        if POKE_DERIVED.contains(&name) {
            self.derived = Some(name);
            return;
        }
        for b in lo..hi {
            self.covered[(b - self.addr) as usize] = true;
            if self.apply {
                let at_field = 8 * (addr + size as u32 - 1 - b);
                let at_poke = 8 * (self.addr + self.size as u32 - 1 - b);
                let byte = (self.value >> at_poke) & 0xFF;
                *v = (*v & !(0xFF << at_field)) | (byte << at_field);
            }
        }
    }
}

impl Game {
    /// Fields the trace does not list but a recipe pokes: G.$46B free play (a flag), G.$757A the
    /// demo counter (a word, the demo row of the attract loop). Returns whether `addr` is one.
    fn trace_poke_extra(&mut self, addr: u32, size: u8, value: u32, apply: bool) -> bool {
        if addr == g(0x046B) && size == 1 {
            if apply {
                self.flow.front.sys.free_play = value & 0xFF != 0;
            }
            return true;
        }
        if addr == g(0x757A) && size == 2 {
            if apply {
                self.flow.front.demo_counter = value as u16;
            }
            return true;
        }
        if addr == g(0x757A) && size == 1 {
            if apply {
                self.flow.front.demo_counter = (self.flow.front.demo_counter & 0x00FF) | ((value as u16 & 0xFF) << 8);
            }
            return true;
        }
        if addr == g(0x757B) && size == 1 {
            if apply {
                self.flow.front.demo_counter = (self.flow.front.demo_counter & 0xFF00) | (value as u16 & 0xFF);
            }
            return true;
        }
        false
    }

    /// Which RAM bytes a poke can reach, indexed from 0xFF0000: 0 no modeled field, 1 a field a poke
    /// can set (also the extra fields of `trace_poke_extra`), 2 a derived field (`POKE_DERIVED`). Used
    /// to check a whole poke file before a run starts.
    pub fn trace_poke_map(&mut self) -> Vec<u8> {
        let mut names = Names::default();
        self.trace_visit(&mut names);
        let mut map = vec![0u8; 0x10000];
        for (addr, size, name) in names.list {
            let kind = if POKE_DERIVED.contains(&name) { 2 } else { 1 };
            for k in 0..size as u32 {
                if let Some(o) = (addr + k).checked_sub(RAM).filter(|o| *o < 0x10000) {
                    if map[o as usize] != 1 {
                        map[o as usize] = kind;
                    }
                }
            }
        }
        for a in [g(0x046B), g(0x757A), g(0x757B)] {
            map[(a - RAM) as usize] = 1;
        }
        map
    }

    /// Harness: stores `value` (`size` 1, 2 or 4 bytes, as the 68000 writes it) at the original RAM
    /// address `addr`, into the port's own field or fields there. The error says why a poke
    /// cannot be taken: no modeled field at that address, a derived field (`POKE_DERIVED`), or
    /// a field of a free pool slot (a free slot keeps only +$00 in the port).
    pub fn trace_poke(&mut self, addr: u32, size: u8, value: u32) -> Result<(), String> {
        if self.trace_poke_extra(addr, size, value, true) {
            return Ok(());
        }
        let mut p = PokeVisit { addr, size, value, apply: false, covered: [false; 4], derived: None };
        self.trace_visit(&mut p);
        if let Some(name) = p.derived {
            return Err(format!("{addr:06X}: `{name}` is a field the port keeps in another form (pointer or list), a poke cannot set it"));
        }
        if let Some(k) = (0..size as usize).find(|k| !p.covered[*k]) {
            let a = addr + k as u32;
            let why = match slot_of_ram(a) {
                Some(h) => format!("it is inside the object {h:04X}, which is a free slot or has no such field in the port"),
                None => "the port has no field there".to_string(),
            };
            return Err(format!("{addr:06X} size {size}: byte {a:06X} is not modeled: {why}"));
        }
        p.apply = true;
        self.trace_visit(&mut p);
        Ok(())
    }
}

/// The object (pool slot) of a RAM address, as its handle.
fn slot_of_ram(a: u32) -> Option<u16> {
    let h = a.checked_sub(RAM).filter(|o| *o < 0x10000)? as u16;
    for (base, stride, n) in [PLAYERS, POOL_A, POOL_B, POOL_E, POOL_C, POOL_D] {
        if h >= base && h < base + stride * n as u16 {
            return Some(base + (h - base) / stride * stride);
        }
    }
    None
}

impl<T> Pool<T> {
    /// Slot index of a handle of this pool.
    pub fn index_of(&self, h: u16) -> Option<usize> {
        let d = h.wrapping_sub(self.base);
        (h >= self.base && d % self.stride == 0 && ((d / self.stride) as usize) < self.slots.len()).then(|| (d / self.stride) as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::enemy::{SpawnRec, Wave, WaveGroup};

    fn assets() -> &'static AssetPack {
        Box::leak(Box::new(AssetPack::default()))
    }

    /// What `trace_ram` writes, `trace_load` reads back: fields and pool free lists.
    #[test]
    fn trace_then_load_round_trip() {
        let mut a = Game::new(assets());
        a.players[0].b.live = 1;
        a.players[0].b.x = 0x0421_9E00;
        a.players[0].b.vx = -0x19E;
        a.players[0].score = 0x0001_2345;
        a.players[0].held = 0x11;
        a.w.rng = 0xA753;
        a.cam.s2.cam_x = 0x381;
        a.rank.freeze = 1;
        let e = a.enemies.alloc().unwrap();
        a.enemies.slots[e].b.live = 1;
        a.enemies.slots[e].b.k20 = 8;
        a.enemies.slots[e].m.sw(0x76, 0xB274);
        let _ = a.effects.alloc();
        let c = a.trace_ram();
        let mut b = Game::new(a.assets);
        let warnings = b.trace_load(&c.ram, &vec![0u8; 0x30000]);
        assert!(warnings.iter().all(|w| w.contains("frame")), "{warnings:?}");
        assert_eq!(b.players[0].b.x, 0x0421_9E00);
        assert_eq!(b.players[0].b.vx, -0x19E);
        assert_eq!(b.players[0].score, 0x0001_2345);
        assert_eq!(b.players[0].held, 0x11);
        assert_eq!((b.w.rng, b.cam.s2.cam_x, b.rank.freeze), (0xA753, 0x381, 1));
        assert_eq!(b.enemies.free, a.enemies.free);
        assert_eq!(b.effects.free, a.effects.free);
        assert_eq!(b.enemies.slots[e].b.k20, 8);
        assert_eq!(b.enemies.slots[e].m.w(0x76), 0xB274);
        // The next allocation is the same slot on both sides.
        assert_eq!(b.enemies.alloc(), a.enemies.alloc());
    }

    #[test]
    fn null_preinitialization_pointers_round_trip_then_expire() {
        let mut game = Game::new(assets());
        let ram = vec![0; 0x10000];
        let warnings = game.trace_load(&ram, &vec![0; 0x30000]);
        assert!(warnings.iter().all(|w| !w.contains("free list pointer")), "{warnings:?}");
        let c = game.trace_ram();
        for off in [0xE5FE, 0xE600, 0xE602, 0xE604, 0xE606, 0xE9F0, 0xEA24, 0xEA28] {
            assert_eq!(&c.ram[off..off + 2], &[0, 0]);
        }
        // A new phase initializes lists even when an empty list still has count zero.
        game.flow.mode = 0xC;
        let c = game.trace_ram();
        assert_eq!(&c.ram[0xE9F0..0xE9F2], &0xEA22u16.to_be_bytes());
        assert_eq!(&c.ram[0xEA24..0xEA26], &0xEA5Au16.to_be_bytes());
    }

    /// A snapshot of a later episode and area: the spawn list pointers and the wave controller's
    /// +$A0 are turned into indexes with the tables of the snapshot's own stage and area, not of the
    /// game that loads it (which starts in stage 0, area 0). The loader reads those two first.
    #[test]
    fn load_reads_stage_and_area_before_the_pointers() {
        use dino_assets::enemy::{AreaScript, StageScript};
        let mut pack = AssetPack::default();
        pack.enemy.stages = (0..8).map(|_| StageScript { areas: vec![AreaScript::default(); 4], ..Default::default() }).collect();
        // Episode 5 (stage 4), area 2: lists of its own, at addresses that differ from stage 0 area 0.
        pack.enemy.stages[4].areas[2] = AreaScript { list1_addr: 0xAD000, list2_addr: 0xAF000, ..Default::default() };
        pack.enemy.stages[4].waves = vec![Wave {
            addr: 0x7C000,
            groups: vec![WaveGroup { thr: [0; 3], spawns: vec![SpawnRec::default(); 2] }, WaveGroup { thr: [0; 3], spawns: vec![SpawnRec::default(); 3] }],
            ..Default::default()
        }];
        let a: &'static AssetPack = Box::leak(Box::new(pack));
        let mut ram = vec![0u8; 0x10000];
        let mut put = |addr: u32, size: usize, v: u32| {
            let o = (addr - RAM) as usize;
            ram[o..o + size].copy_from_slice(&v.to_be_bytes()[4 - size..]);
        };
        put(g(0x04D9), 1, 4); // stage
        put(g(0x073C + 0x4A), 1, 2); // area
        put(g(0x0674), 4, 0xAD000 + 14 * 5); // spawn list 1: record 5
        put(g(0x0678), 4, 0xAF000 + 16 * 7); // spawn list 2: record 7
        // A wave controller (class 15, variant 0) in pool B slot 0, waiting in group 1.
        let (base, ..) = POOL_B;
        put(obj(base), 1, 1);
        put(obj(base) + 0x20, 2, 15 * 4);
        put(obj(base) + 0x05, 1, 0); // sub 0
        put(obj(base) + 0x04, 1, 2); // mode 2: running
        put(obj(base) + 0xA0, 4, 0x7C000 + 54);
        let mut game = Game::new(a);
        let warnings = game.trace_load(&ram, &vec![0u8; 0x30000]);
        assert_eq!((game.w.stage, game.cam.s2.area), (4, 2));
        assert_eq!((game.spawn.l1, game.spawn.l2), (5, 7));
        assert_eq!((game.props.slots[0].m.w(0xA0), game.props.slots[0].m.w(0xA2)), (1, 0));
        assert!(warnings.iter().all(|w| !w.contains("wave controller")), "{warnings:?}");
    }

    /// A poke into an object's field needs the object: a free pool slot keeps only +$00.
    #[test]
    fn poke_needs_a_live_object() {
        let mut a = AssetPack::default();
        a.enemy.stages = (0..8).map(|_| Default::default()).collect();
        let mut game = Game::new(Box::leak(Box::new(a)));
        let x = obj(POOL_A.0 + 0xE0 * 3) + 8; // pool A slot 3, +$08 x
        assert!(game.trace_poke(x, 2, 0x0123).is_err());
        let i = game.enemies.alloc().unwrap(); // slot 23 (the first allocation)
        game.enemies.slots[i].b.live = 1;
        let x = obj(game.enemies.handle(i)) + 8;
        game.trace_poke(x, 2, 0x0123).unwrap();
        assert_eq!(game.enemies.slots[i].b.x >> 16, 0x0123);
        // A derived field (a pointer kept as an index) is an error, never silently dropped.
        assert!(game.trace_poke(g(0x0674), 4, 1).is_err());
        game.trace_poke(g(0x04D9), 1, 3).unwrap();
        assert_eq!(game.w.stage, 3);
    }

    #[test]
    fn wave_pointer_round_trip() {
        let rec = SpawnRec::default();
        let w = Wave {
            addr: 0x7C000,
            groups: vec![WaveGroup { thr: [0; 3], spawns: vec![rec; 2] }, WaveGroup { thr: [0; 3], spawns: vec![rec; 3] }],
            ..Default::default()
        };
        // Group 1 starts after group 0: 6 + 2 * 16 + 16 bytes.
        assert_eq!(wave_ptr(&w, 1, 0, 0), 0x7C000 + 54);
        assert_eq!(wave_ptr(&w, 1, 2, 2), 0x7C000 + 54 + 6 + 32);
        for (g, r, sub) in [(0, 0, 0), (0, 1, 2), (1, 0, 0), (1, 2, 2)] {
            let p = wave_ptr(&w, g, r, sub);
            assert_eq!(wave_pos(&w, p), Some((g, if sub == 0 { 0 } else { r })));
        }
    }
}
