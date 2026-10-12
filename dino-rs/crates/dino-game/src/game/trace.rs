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
use super::item::Item;
use super::player::Player;
use super::prop::Prop;
use super::{Game, POOL_A, POOL_B, POOL_C, POOL_D, POOL_E, PLAYERS, Pool};
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
                    if live {
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
    if EffectData::is_effect_frame(id) {
        fx.frames.get((id & 0x3FFF_FFFF) as usize).map(|f| f.rom_addr).unwrap_or(0)
    } else if EffectData::is_raw_frame(id) {
        fx.raw_frames.get((id & 0x3FFF_FFFF) as usize).map(|f| f.rom_addr).unwrap_or(0)
    } else {
        a.sprites.frames.get(id as usize).map(|f| f.rom_addr).unwrap_or(0)
    }
}

/// Frame id of a ROM frame record address.
pub fn frame_id(a: &AssetPack, addr: u32, effects: bool, raw: bool) -> Option<FrameId> {
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

impl Enemy {
    pub fn trace_visit(&mut self, f: &mut F) {
        self.b.trace_visit(f);
        self.m.trace_list(f, ENEMY_RAW);
    }
}

impl Item {
    /// Pool E is not ported yet: only the header.
    pub fn trace_visit(&mut self, f: &mut F) {
        self.b.trace_header(f, false);
    }
}

impl Prop {
    /// Header, and the wave controller (class 15) fields. The port keeps the wave position as
    /// group / record indexes (+$A0 / +$A2); the original keeps a ROM pointer at +$A0.
    pub fn trace_visit(&mut self, f: &mut F, stage: u8) {
        self.b.trace_header(f, false);
        if f.all() || self.b.k20 == 15 * 4 {
            let mut t = self.m.w(0x80);
            f.u16(0x80, "wave_timer", &mut t);
            self.m.sw(0x80, t);
            let mut n = self.m.w(0xA6);
            f.u16(0xA6, "wave_spawned", &mut n);
            self.m.sw(0xA6, n);
            let waves = &f.assets.enemy.stages[stage as usize].waves;
            if let Some(w) = waves.get(self.b.var26 as usize) {
                let (g, r) = (self.m.w(0xA0) as usize, self.m.w(0xA2) as usize);
                let mut p = wave_ptr(w, g, r, self.b.sub);
                let old = p;
                f.u32(0xA0, "wave_ptr", &mut p);
                if f.loading() && p != old {
                    match wave_pos(w, p) {
                        Some((g, r)) => {
                            self.m.sw(0xA0, g as u16);
                            self.m.sw(0xA2, r as u16);
                        }
                        None => f.v.error(format!("wave controller: pointer {p:#x} not in wave {:#x}", w.addr)),
                    }
                }
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

impl Special {
    /// Pool D is not ported yet: only the header.
    pub fn trace_visit(&mut self, f: &mut F) {
        self.b.trace_header(f, true);
    }
}

impl Player {
    /// Player-only fields. The HUD / flow / enemy ports keep some player fields elsewhere; those
    /// are visited by `Game::trace_player_extras`.
    pub fn trace_visit(&mut self, f: &mut F) {
        self.b.trace_visit(f);
        f.u8(0x52, "b52", &mut self.b52);
        f.u8(0x73, "escape", &mut self.escape);
        f.u8(0x75, "b75", &mut self.b75);
        f.u16(0x76, "w76", &mut self.w76);
        f.u8(0x7A, "b7a", &mut self.b7a);
        f.u8(0x7E, "b7e", &mut self.b7e);
        f.u8(0x7F, "b7f", &mut self.b7f);
        f.u16(0x80, "w80", &mut self.w80);
        f.u8(0x83, "b83", &mut self.b83);
        f.u16(0x88, "w88", &mut self.w88);
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

/// Free list stacks: (count G.x, pointer G.x, stack end G.x) for pools E, B, A, C, D.
const FREE_LISTS: [(u32, u32, u32); 5] =
    [(0x65F4, 0x65FE, 0x6638), (0x65F6, 0x6600, 0x6668), (0x65F8, 0x6602, 0x6698), (0x65FA, 0x6604, 0x66F8), (0x65FC, 0x6606, 0x6718)];

impl Game {
    /// Visits every modeled object field and global.
    pub fn trace_visit(&mut self, v: &mut dyn Visit) {
        let a = self.assets;
        let names_only = v.all();
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
                    if $pool.slots[i].b.live == 0 && !names_only {
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
                    } else {
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
        f.u8(0x07B4, "b7b4", &mut r.b7b4);
        f.u8(0x04E8, "time_min", &mut r.time_min);
        f.u8(0x04E9, "time_sec", &mut r.time_sec);
        f.u8(0x04EA, "time_div", &mut r.time_div);
        f.u8(0x04EB, "time_lock", &mut r.time_lock);
        if f.loading() {
            r.started = true;
        }
        // Enemy-side globals (spawn.rs).
        let s = &mut self.spawn.g;
        f.u8(0x04F3, "alive", &mut s.alive);
        f.u8(0x04FB, "alive_run", &mut s.alive_run);
        f.u8(0x050E, "kill_all", &mut s.kill_all);
        f.u16(0x04E0, "killer", &mut s.killer);
        f.u16(0x050C, "wave_timer", &mut s.wave_timer);
        f.u8(0x0518, "car_mode", &mut s.car_mode);
        f.u8(0x0502, "b502", &mut s.b502);
        f.u8(0x04D5, "b4d5", &mut s.b4d5);
        f.u8(0x04F1, "b4f1", &mut s.b4f1);
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
        f.u8(0x755C, "shake_req", &mut self.cam.shake_req);
    }

    fn trace_flow(&mut self, f: &mut F) {
        let fl = &mut self.flow;
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
        f.u8(0x04E6, "b4e6", &mut fl.b4e6);
        f.u8(0x04E5, "b4e5", &mut fl.b4e5);
        f.u16(0x0508, "timer508", &mut fl.timer508);
        for k in 0..4 {
            f.u8(0x0510 + k as u32, ["saved_mode0", "saved_mode1", "saved_mode2", "saved_mode3"][k], &mut fl.saved_mode[k]);
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
        // Stage intro task struct G.$735E (the task itself: task slot 0x30, see trace_load_tasks).
        let it = &mut fl.intro;
        f.u8(0x735E, "intro_state", &mut it.state);
        f.u8(0x735F, "intro_sub", &mut it.sub);
        f.u16(0x7364, "intro_w6", &mut it.w6);
        f.u16(0x7366, "intro_w8", &mut it.w8);
        f.u32(0x7368, "intro_src", &mut it.src);
        f.u32(0x736C, "intro_dst", &mut it.dst);
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
        self.trace_visit(&mut l);
        self.trace_unmapped = l.unmapped;
        let mut warn = l.errors;
        self.trace_load_tasks(ram, &mut warn);
        self.trace_load_gfx(gfx);
        warn
    }

    /// Abstract state the port keeps instead of the task table (0xFF0052, 16 bytes per slot:
    /// +0 state, +1 sleep counter).
    fn trace_load_tasks(&mut self, ram: &[u8], warn: &mut Vec<String>) {
        let task = |slot: usize| (ram[0x52 + 16 * slot], ram[0x53 + 16 * slot]);
        // Slot 2 (0x20): the game task sleeps G.$4D6 & 15 frames between runs.
        let (st, cnt) = task(2);
        self.flow.wait = if st == 1 { cnt } else { 0 };
        // Slot 3 (0x30): stage intro or stage clear task.
        let (st, cnt) = task(3);
        self.flow.intro.live = false;
        self.flow.clear.live = false;
        if st != 0 {
            warn.push(format!("task slot 0x30 (intro / clear) is active (state {st}, sleep {cnt}): not taken over"));
        }
        // Slot 4 (0x40): fade task.
        let (st, _) = task(4);
        self.fade.task = None;
        if st != 0 {
            warn.push(format!("task slot 0x40 (fade) is active (state {st}): not taken over"));
        }
        self.flow.front.prev_system = ram[0x801F];
        for i in 0..3 {
            // Join / continue task 0x6C4A is a stub: nothing to load beyond +$A2.
            let _ = i;
        }
    }

    /// Tilemaps and palettes from GFX RAM. Bases: scroll1 0x908000, scroll2 0x90C000,
    /// scroll3 0x910000, palettes 0x914000 (the in-game CPS-A settings).
    fn trace_load_gfx(&mut self, gfx: &[u8]) {
        let w = |o: usize| u16::from_be_bytes([gfx[o], gfx[o + 1]]);
        for (layer, base) in [(0, 0x8000usize), (1, 0xC000), (2, 0x10000)] {
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
    }

    // --------------------------------------------------------------------------------------
    // Trace images
    // --------------------------------------------------------------------------------------

    /// RAM image and mask of every modeled field.
    pub fn trace_ram(&mut self) -> Collect {
        let mut c = Collect { unmapped: std::mem::take(&mut self.trace_unmapped), ..Default::default() };
        self.trace_visit(&mut c);
        self.trace_unmapped = std::mem::take(&mut c.unmapped);
        c
    }

    /// GFX RAM image (0x900000, 192 KB) and mask: the three tilemaps and the palettes.
    pub fn trace_gfx(&self) -> (Vec<u8>, Vec<u8>) {
        let mut img = vec![0u8; 0x30000];
        let mut mask = vec![0u8; 0x30000];
        for (cells, base) in [(&self.vram.scroll1, 0x8000usize), (&self.vram.scroll2, 0xC000), (&self.vram.scroll3, 0x10000)] {
            for (i, c) in cells.iter().enumerate() {
                let o = base + 4 * i;
                img[o..o + 2].copy_from_slice(&c.code.to_be_bytes());
                img[o + 2..o + 4].copy_from_slice(&c.attr.to_be_bytes());
            }
            mask[base..base + 0x4000].fill(1);
        }
        for (i, p) in self.pal.words.iter().enumerate() {
            img[0x14000 + 2 * i..0x14000 + 2 * i + 2].copy_from_slice(&p.to_be_bytes());
        }
        mask[0x14000..0x15000].fill(1);
        (img, mask)
    }

    /// The sprite list built this frame, as the hardware entries (8 bytes each, up to 255) and
    /// the end marker the original writes (0x13FA6: X 0, Y 0x100, code 0, attr 0xFF00).
    pub fn trace_obj(&self) -> Vec<u8> {
        let mut v = vec![0u8; 0x800];
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
