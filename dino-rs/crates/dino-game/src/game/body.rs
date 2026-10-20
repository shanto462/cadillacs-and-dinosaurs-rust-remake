//! The object header every object shares (players, enemies, items, props, effects).
//!
//! In the original, all objects live in RAM blocks and the first ~0xA0 bytes have the same
//! meaning for every kind: position, state bytes, animation, boxes, hit state. Shared routines
//! (animation, physics, drawing, collision, damage) only touch these. Fields from about +$80 up
//! differ per kind and live in the kind's own struct (`Player`, `Enemy`, ...).
//!
//! Offsets are in the comments so the port can be read next to `re/ghidra/decomp.c`.

use super::Handle;
use dino_assets::{FrameId, SpriteData};

/// 16.16 fixed point, as the original's long words at +$08 / +$0C / +$10.
pub type Fixed = i32;

#[inline]
pub fn px(v: Fixed) -> i16 {
    (v >> 16) as i16
}

#[derive(Clone, Debug, Default)]
pub struct Body {
    /// +$00 presence. 0 = free slot. Players: 1 active, 2 active but invulnerable.
    /// Enemies: 1 alive, 2 in hit reaction.
    pub live: u8,
    /// +$01 on-screen flag, set by the draw-list check (0x16FA).
    pub visible: u8,
    /// +$02 players: index 0..2. Enemies: credited player for score (negative = nobody).
    pub who: u8,
    /// +$03 players: character (0 Jack, 1 Mustapha, 2 Hannah, 3 Mess). Enemies: name id.
    pub id: u8,
    /// +$04..+$07 state bytes: routine, sub-routine, action, step (often written as one long).
    pub mode: u8,
    pub sub: u8,
    pub act: u8,
    pub step: u8,
    /// +$08 world X, +$0C height above the floor plane (up = +), +$10 depth (+ = away).
    pub x: Fixed,
    pub y: Fixed,
    pub z: Fixed,
    /// +$14 / +$16 / +$18 velocities and +$1A / +$1C / +$1E accelerations, 8.8 px per frame.
    pub vx: i16,
    pub vy: i16,
    pub vz: i16,
    pub ax: i16,
    pub ay: i16,
    pub az: i16,
    /// +$20 class * 4 (pool dispatch) or character * 4 (players).
    pub k20: u16,
    /// +$22 move direction (players: 0..0x1C step 4, 0xFF none; enemies: 0..31).
    pub dir: u8,
    /// +$23 palette override: bit 7 set => low 5 bits replace every sprite's palette.
    pub palette_override: u8,
    /// +$24 facing: 0 = art as stored (faces left), nonzero = mirrored (faces right).
    pub facing: u8,
    /// +$25 draw flags: bit0 split body, bit3 hidden (blink), bit4 no shadow, bit5 no splash,
    /// bit6 raw list, bit7 screen space.
    pub draw_flags: u8,
    /// +$26 spawn "var" word from the stage script (enemies, items).
    pub var26: u16,
    /// +$28 current frame.
    pub frame: FrameId,
    /// Not an original field: the second word of the last frame record read by `load_frame` (0x1236 leaves it in d1). The Butcher's roll 0x5A1AC compares
    /// d1 as a word whose high byte is still this (the animation step of the same frame), so the word decides when a frame was loaded.
    pub frame_w1: u16,
    /// Not an original field: how often a frame record was loaded into +$28 / +$30 (the trace writer shows a null frame pointer of the original, which the port
    /// cannot keep as a frame id, until the first load).
    pub frame_loads: u32,
    pub frame2_loads: u32,
    /// +$2C / +$2D / +$2E: frame bytes +4 / +5 / +6 (+$2E = frame class for collision).
    pub param_a: u8,
    pub param_b: u8,
    pub kind: u8,
    /// +$2F frame timer.
    pub timer: u8,
    /// +$30 split body: secondary ("legs") frame; +$34/+$36 its words; +$37 its timer.
    pub frame2: FrameId,
    pub w34: u16,
    pub w36: u16,
    /// +$38 split body offset table (ROM address), +$3C/+$3E current offset of the upper part.
    pub offs_table: u32,
    pub dx3c: i16,
    pub dy3e: i16,
    /// +$40 animation table (non-player objects), as its ROM address.
    pub anim_table: u32,
    /// +$44 hit box table (ROM address) and +$4A multi-box flag.
    pub box_table: u32,
    pub multi_box: bool,
    /// +$48 body box index, +$49 attack / probe box index.
    pub body_box: u8,
    pub attack_box: u8,
    /// +$4C..+$4F per-composition extra bytes (wading clip counts, shadow type).
    pub extra: [u8; 4],
    /// +$50 wall / terrain contact bits from the floor collision.
    pub wall: u8,
    /// +$51 airborne flag.
    pub airborne: u8,
    /// +$52 floor type byte (low byte of the flat floor segment's base, 0x12FE4).
    pub b52: u8,
    /// +$53 wading flag.
    pub wading: u8,
    /// +$54 floor height and +$56 floor offset; the floor is +$54 + +$56.
    pub floor: i16,
    pub floor2: i16,
    /// +$58 current attack id (selects the attack record).
    pub atk: u8,
    /// +$59 hit-stop counter (set on both sides by a hit).
    pub hitstop: u8,
    /// +$5A "force knockdown" / last hit of a combo flag.
    pub knock: u8,
    /// +$5E attack serial of the current swing (one swing hits each target once).
    pub serial: u8,
    /// +$60 attach point (12-byte frames).
    pub attach: u16,
    /// +$62 draw order switch for a grab pair.
    pub b62: u8,
    /// +$63 "big" flag / grab immunity when negative.
    pub b63: u8,
    /// +$64 speed table (ROM address).
    pub speed_table: u32,
    /// +$68 last attacker.
    pub attacker: Handle,
    /// +$6A HP shown by the bar (max 100), +$6C HP, +$6E HP before the last hit.
    pub hp_bar: u16,
    pub hp: u16,
    pub hp_prev: u16,
    /// +$70 grab partner (holder or held object).
    pub partner: Handle,
    /// +$72 hold time this object allows when grabbed.
    pub b72: u8,
    /// +$78 stun / poise meter (10 = full; 0 forces a knockdown).
    pub stun: u8,
    /// +$7B last hurt reaction code.
    pub reaction: u8,
    /// +$9A hit spark / dizzy row byte from the attack record.
    pub b9a: u8,
}

impl Body {
    pub fn xi(&self) -> i16 {
        px(self.x)
    }
    pub fn yi(&self) -> i16 {
        px(self.y)
    }
    pub fn zi(&self) -> i16 {
        px(self.z)
    }
    pub fn set_xi(&mut self, v: i16) {
        self.x = (v as i32) << 16;
    }
    pub fn set_yi(&mut self, v: i16) {
        self.y = (v as i32) << 16;
    }
    pub fn set_zi(&mut self, v: i16) {
        self.z = (v as i32) << 16;
    }

    /// +$04..+$07 as one long (e.g. 0x02000004).
    pub fn state(&self) -> u32 {
        u32::from_be_bytes([self.mode, self.sub, self.act, self.step])
    }
    pub fn set_state(&mut self, v: u32) {
        [self.mode, self.sub, self.act, self.step] = v.to_be_bytes();
    }

    /// 0x189C: v += a, pos += v << 8, in the order X, Y, Z (semi-implicit Euler).
    pub fn motion(&mut self) {
        self.vx = self.vx.wrapping_add(self.ax);
        self.x = self.x.wrapping_add((self.vx as i32) << 8);
        self.vy = self.vy.wrapping_add(self.ay);
        self.y = self.y.wrapping_add((self.vy as i32) << 8);
        self.vz = self.vz.wrapping_add(self.az);
        self.z = self.z.wrapping_add((self.vz as i32) << 8);
    }

    /// 0x18E0: the Y axis only.
    pub fn motion_y(&mut self) {
        self.vy = self.vy.wrapping_add(self.ay);
        self.y = self.y.wrapping_add((self.vy as i32) << 8);
    }

    /// A write of the frame pointer +$28 / +$30.
    pub fn set_frame(&mut self, id: FrameId) {
        self.frame = id;
        self.frame_loads = self.frame_loads.wrapping_add(1);
    }
    pub fn set_frame2(&mut self, id: FrameId) {
        self.frame2 = id;
        self.frame2_loads = self.frame2_loads.wrapping_add(1);
    }

    /// Frame loaders 0x1236 (10-byte records) and 0x143E (12-byte records). The extractor
    /// already resolved which composition selects the extra bytes.
    pub fn load_frame(&mut self, s: &SpriteData, id: FrameId) {
        let f = s.frame(id);
        self.set_frame(id);
        self.frame_w1 = f.w1;
        self.param_a = f.param_a;
        self.param_b = f.param_b;
        self.kind = f.kind;
        self.timer = f.duration;
        self.body_box = f.body_box;
        self.attack_box = f.attack_box;
        if f.stride == 12 {
            self.attach = f.attach;
        }
        if self.draw_flags & 1 == 0 {
            self.extra = f.extra;
        }
    }

    /// Animation tick 0x121E / 0x1426. Returns true when the animation followed its link
    /// (the 68000 carry flag), which the state code reads as "animation finished".
    pub fn tick_anim(&mut self, s: &SpriteData) -> bool {
        self.timer = self.timer.wrapping_sub(1);
        if self.timer != 0 {
            return false;
        }
        let f = s.frame(self.frame);
        let (next, finished) = (f.next, f.last);
        self.load_frame(s, next);
        finished
    }

    /// 0x120E: start animation `id` of this object's own table (+$40).
    pub fn start_anim(&mut self, s: &SpriteData, id: u16) {
        let first = s.anim(self.anim_table, id);
        self.load_frame(s, first);
    }

    /// 0x1416 + the player set entries 0x13A0..0x13CC: start animation `id` of player set `set`.
    pub fn start_player_anim(&mut self, s: &SpriteData, set: usize, id: u16) {
        let table = s.player_sets[set][self.id as usize & 3];
        let first = s.anim(table, id);
        self.load_frame(s, first);
    }
}
