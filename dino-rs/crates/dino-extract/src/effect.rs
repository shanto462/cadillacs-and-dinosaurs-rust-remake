//! Extracts the data used by the draw paths (shadow and splash lists) and by the effect objects
//! of pool C (effect animation tables, direct frame lists, raw-list frames). Formats:
//! `re/specs/anim/SPEC.md` 3.1 / 3.4. Every address below is a constant in the 68000 routine named
//! next to it.

use crate::rom::Rom;
use dino_assets::effect::{EFX_FRAME, EffectData, RAW_FRAME, RawFrame, RawSprite};
use dino_assets::{CompoundPart, Frame, FrameDraw, FrameId};
use std::collections::VecDeque;

/// Effect animation tables (10-byte records), the constant each effect stores in obj+$40.
const EFFECT_TABLES: &[u32] = &[
    0x93C82, // respawn bubble and player markers (0x93B44)
    0x924F6, // hit spark, effect $20 = 4 (0x924BE)
    0x92AC0, // water splash, effect $20 = 0x10 (0x92A72)
    0x937AA, // debris, effect $20 = 0x2C (0x93590)
    0x9770A, // Fessenden's death debris, effect $20 = 0xA0 (0x97688)
    0x951AC, // puff pieces, effect $20 = 0x54 (0x94776)
    0x944DC, // dizzy stars, effect $20 = 0x4C (0x94476)
    0x96884, // gunshot muzzle flash, effect $20 = 0x7C (0x96800), 4 animations by the gun
    0x96D06, // bullet ricochet spark, effect $20 = 0x84 (0x96B78)
    0x28904, // fuse spark of a TNT barrel, effect $20 = 0x64 (0x962AA)
    0x92BF0, // water bubbles and wake, effect $20 = 0x14 (0x92B8E), 12 animations
    0x945CC, // sweat and shock beads, effect $20 = 0x50 (0x9456E), 9 frames
    0x9664E, // ground impact smoke ring, effect $20 = 0x74 (0x965E2), 1 animation
    0x97150, // the Cadillac's seat drawers, effect $20 = 0x90 (0x97008), 8 animations (character * 2, + 1 for the third seat)
    0x9786E, // the Cadillac's dust and smoke, effect $20 = 0xA4 (0x97814)
    0x972B2, // the teammate who delivers the car, effect $20 = 0x94 (0x9721E), 8 animations (jump by character, then stand)
    0x96770, // the Z mark of a sleeping dinosaur, effect $20 = 0x78 (0x966C8), 1 animation
    0x93426, // the shards of the broken hall window, effect $20 = 0x28 (0x932D2), 7 animations
];

/// Frame lists loaded directly with 0x160A (10-byte records, no table).
const EFFECT_LISTS: &[u32] = &[
    0x92E1E, // game-over flash (0x92CF6)
    0x92E34, // class 6 variant 8 frame (0x92D2A)
    0x9422A, // countdown digit frames (0x94120)
    0x94236,
    0x94242,
    0x9424E,
    0x92E12, // continue return flash, class 6 variant 0 (0x92CD6)
    0x9430E, // Bludge acid splash, effect $20 = 0x48 (0x942C6, loaded with 0x1236)
    0x95980, // invulnerability glitter (0x9592E)
    0x975AC, // mutation aura, effect $20 = 0x9C (0x97546)
    0x93096, // burning flames, effect $20 = 0x20, airborne loop, var 0 (0x92FD4)
    0x930F4, // same, var != 0 (0x92FE0)
    0x930D4, // flames dying out after landing, var 0 (0x93054)
    0x9313C, // same, var != 0 (0x93060)
    0x29CB6, // torch glow, idle flame, item class 6 (0x29B1A)
    0x29CF4, // torch glow, swing flame (0x29C10)
    0x92A30, // hammer shockwave, effect $20 = 0xC, var 0 (0x929CC)
    0x92A3C, // same, var != 0 (0x929D6)
    0x937D0, // debris piece of a broken bat, var 0 (0x9353A)
    0x93872, // debris piece (0x93564)
    0x9387E, // debris piece (0x9356C)
    0x96B10, // cloud of the ranking screen, effect $20 = 0x80, var 0 (0x96AB6)
    0x96B1C, // same, var 1 (0x96AC8)
    0x96B28, // same, var 2 (0x96AD4)
    0x975AC, // the flare of the Episode 8 clear scene, effect $20 = 0x9C (0x97546, M9-04)
];

/// Raw-list animations (8-byte records), loaded with 0x16C6.
const RAW_LISTS: &[u32] = &[
    0x940A4, // GAME (0x94026)
    0x940AE, // OVER (0x94048)
    0x93D1A, // 1UP marker (0x93BF8)
    0x96360, // water ripple, effect $20 = 0x68 (0x96330)
    0x7B5C8, // the frame of the select screen, pool B class 17 (0x7B554)
    0x96546, // the word TIME, effect $20 = 0x70 variant 0 (0x964B8)
    0x96550, // the word OVER, effect $20 = 0x70 variant 1 (0x964DA)
    0x92F3A, // ground dust, effect $20 = 0x1C (0x92EE0)
    0x928C0, // cave dust/drops falling, effect $20 = 0x8 var 0 (0x928B0)
    0x928DA, // same, var 4
    0x928F4, // same, var 8
    0x9290E, // cave dust/drops floor splash (0x92892)
];

/// Shadow sprite list table used by the shadow pass 0x14EC0 (8 longs).
const SHADOW_TABLE: u32 = 0x14F98;
/// Splash sprite list table used by 0x14BC6: 8 types x 4 phases (longs).
const SPLASH_TABLE: u32 = 0x14C9C;
/// Jitter pairs of the spark spawner 0x92450 (indexed by frame counter & 0xE).
const SPARK_JITTER: u32 = 0x9248A;
/// Per-composition extra bytes (same as the sprite extractor).
const COMP_EXTRA: u32 = 0x11D220;
const COMP_COUNT_PTR: u32 = 0x0FFFF8;

pub fn extract(rom: &Rom, extra: &crate::ExtraFrames) -> EffectData {
    let mut d = EffectData::default();
    let n_comps = rom.u16(COMP_COUNT_PTR) as u32;
    let mut w = Walker { rom, d: &mut d, n_comps };
    for &t in EFFECT_TABLES {
        w.table(t);
    }
    for &a in EFFECT_LISTS {
        w.walk(a);
    }
    for &a in RAW_LISTS {
        w.raw(a);
    }
    // On demand (harness): records that only a snapshot's objects point at.
    for &a in &extra.effect {
        if w.chain_valid(a, 10) {
            w.walk(a);
        }
    }
    for &a in &extra.raw {
        if w.chain_valid(a, 8) {
            w.raw(a);
        }
    }

    d.shadows = (0..8).map(|i| shadow_list(rom, rom.u32(SHADOW_TABLE + 4 * i))).collect();
    d.splashes = (0..8)
        .map(|t| std::array::from_fn(|p| splash_list(rom, rom.u32(SPLASH_TABLE + 16 * t + 4 * p as u32))))
        .collect();
    for (k, pair) in d.spark_jitter.iter_mut().enumerate() {
        *pair = (rom.i16(SPARK_JITTER + 4 * k as u32), rom.i16(SPARK_JITTER + 4 * k as u32 + 2));
    }
    d
}

/// Shadow list (0x14F48): u16 n-1, u16 count, n x (attr, dx, code).
fn shadow_list(rom: &Rom, a: u32) -> Vec<RawSprite> {
    let n = rom.u16(a) as u32 + 1;
    (0..n)
        .map(|k| {
            let e = a + 4 + 6 * k;
            RawSprite { attr: rom.u16(e), dx: rom.i16(e + 2), dy: 0, code: rom.u16(e + 4) }
        })
        .collect()
}

/// Splash list (0x14BDE): u16 n-1, u16 count, n x (attr, dx, dy, code).
fn splash_list(rom: &Rom, a: u32) -> Vec<RawSprite> {
    let n = rom.u16(a) as u32 + 1;
    (0..n)
        .map(|k| {
            let e = a + 4 + 8 * k;
            RawSprite { attr: rom.u16(e), dx: rom.i16(e + 2), dy: rom.i16(e + 4), code: rom.u16(e + 6) }
        })
        .collect()
}

/// Raw sprite list (0x147FE): u16 count, u16 n, n x (dx, dy, attr, code).
fn raw_list(rom: &Rom, a: u32) -> Vec<RawSprite> {
    let n = rom.u16(a + 2) as u32;
    (0..n)
        .map(|k| {
            let e = a + 4 + 8 * k;
            RawSprite { dx: rom.i16(e), dy: rom.i16(e + 2), attr: rom.u16(e + 4), code: rom.u16(e + 6) }
        })
        .collect()
}

struct Walker<'a> {
    rom: &'a Rom,
    d: &'a mut EffectData,
    n_comps: u32,
}

impl Walker<'_> {
    /// True when every record reachable from `start` (`stride` 10 or 8 bytes) looks like frame
    /// data. Only used for on-demand addresses; the registered tables are known to be good.
    fn chain_valid(&self, start: u32, stride: u32) -> bool {
        let len = self.rom.program.len() as u32;
        let mut seen = Vec::new();
        let mut p = start;
        while !seen.contains(&p) && !self.d.frame_by_addr.contains_key(&p) && !self.d.raw_by_addr.contains_key(&p) {
            if p % 2 != 0 || p + stride + 2 > len {
                return false;
            }
            let ok = if stride == 8 {
                let list = self.rom.u32(p);
                list % 2 == 0 && list + 4 <= len && list + 4 + 8 * self.rom.u16(list + 2) as u32 <= len
            } else {
                let (w0, w1) = (self.rom.u16(p), self.rom.u16(p + 2));
                let comp = |w: u16| w % 4 == 0 && ((w / 4) as u32) < self.n_comps;
                if w0 & 0x8000 != 0 {
                    let q = (p + 2).wrapping_add(self.rom.i16(p + 2) as i32 as u32);
                    q + 2 < len && self.rom.u16(q) < 64
                } else {
                    comp(w0) && (w1 & 0x8000 != 0 || comp(w1))
                }
            };
            if !ok {
                return false;
            }
            seen.push(p);
            p = self.next_addr(p, stride);
        }
        true
    }

    /// One 10 byte record looks like frame data: a compound frame with a short part list, or one or two composition numbers.
    fn record_ok(&self, p: u32) -> bool {
        let (w0, w1) = (self.rom.u16(p), self.rom.u16(p + 2));
        let comp = |w: u16| w % 4 == 0 && ((w / 4) as u32) < self.n_comps;
        if w0 & 0x8000 != 0 {
            let q = (p + 2).wrapping_add(self.rom.i16(p + 2) as i32 as u32);
            (q as usize) + 2 < self.rom.program.len() && self.rom.u16(q) < 64
        } else {
            comp(w0) && (w1 & 0x8000 != 0 || comp(w1))
        }
    }

    fn table(&mut self, table: u32) {
        let mut first = u32::MAX;
        let mut offs = Vec::new();
        let mut i = 0;
        while 2 * i < first {
            let off = self.rom.u16(table + 2 * i);
            if off != 0 {
                first = first.min(off as u32);
            }
            offs.push(off);
            i += 1;
        }
        let anims = offs
            .iter()
            .map(|&o| if o == 0 { None } else { Some(self.walk(table.wrapping_add(o as i16 as i32 as u32))) })
            .collect();
        self.d.tables.insert(table, anims);
    }

    fn next_addr(&self, p: u32, stride: u32) -> u32 {
        let q = p + stride;
        if self.rom.u8(p + 7) & 0x80 != 0 { q.wrapping_add(self.rom.i16(q) as i32 as u32) } else { q }
    }

    /// 10-byte records reachable from `start`.
    fn walk(&mut self, start: u32) -> FrameId {
        let mut order = Vec::new();
        let mut q = VecDeque::from([start]);
        while let Some(p) = q.pop_front() {
            if self.d.frame_by_addr.contains_key(&p) || order.contains(&p) {
                continue;
            }
            // A chain that runs on into data that is not a frame record ends there (a snapshot object whose animation has no
            // closing record the walk knows: the harness registers such addresses on demand).
            if p != start && !self.record_ok(p) {
                continue;
            }
            order.push(p);
            q.push_back(self.next_addr(p, 10));
        }
        let base = self.d.frames.len() as FrameId;
        for (k, &p) in order.iter().enumerate() {
            self.d.frame_by_addr.insert(p, EFX_FRAME | (base + k as FrameId));
        }
        for &p in &order {
            let f = self.decode(p);
            self.d.frames.push(f);
        }
        self.d.frame_by_addr[&start]
    }

    fn decode(&self, p: u32) -> Frame {
        let rom = self.rom;
        let (w0, w1) = (rom.u16(p), rom.u16(p + 2));
        assert!(w0 & 0x8000 != 0 || ((w0 / 4) as u32) < self.n_comps, "bad effect frame at {p:#x}");
        let draw = if w0 & 0x8000 != 0 {
            let mut q = (p + 2).wrapping_add(rom.i16(p + 2) as i32 as u32);
            let n = rom.u16(q) as u32 + 1;
            q += 2;
            FrameDraw::Compound(
                (0..n)
                    .map(|k| {
                        let e = q + 8 * k;
                        CompoundPart { dx: rom.i16(e), dy: rom.i16(e + 2), comp: rom.u16(e + 4), clip_flag: rom.u16(e + 6) }
                    })
                    .collect(),
            )
        } else {
            FrameDraw::Comp { main: w0, overlay: (w1 & 0x8000 == 0).then_some(w1) }
        };
        // 0x1236 rule (10-byte loader); effects load with 0x160A, which does not copy it.
        let ci = if w0 & 0x8000 != 0 {
            w0 & 0x7FFF
        } else if w1 & 0x8000 == 0 {
            w1
        } else {
            w0
        };
        let b7 = rom.u8(p + 7);
        Frame {
            rom_addr: p,
            stride: 10,
            w0,
            w1,
            draw,
            param_a: rom.u8(p + 4),
            param_b: rom.u8(p + 5),
            kind: rom.u8(p + 6),
            duration: b7 & 0x7F,
            last: b7 & 0x80 != 0,
            body_box: rom.u8(p + 8),
            attack_box: rom.u8(p + 9),
            attach: 0,
            extra: rom.u32(COMP_EXTRA.wrapping_add(ci as i16 as i32 as u32)).to_be_bytes(),
            // a link into data that is not a frame record (the chain was cut in `walk`) points back at the frame itself
            next: self.d.frame_by_addr.get(&self.next_addr(p, 10)).copied().unwrap_or(self.d.frame_by_addr[&p]),
        }
    }

    /// 8-byte raw-list records reachable from `start`.
    fn raw(&mut self, start: u32) -> FrameId {
        let mut order = Vec::new();
        let mut p = start;
        while !order.contains(&p) && !self.d.raw_by_addr.contains_key(&p) {
            order.push(p);
            p = self.next_addr(p, 8);
        }
        let base = self.d.raw_frames.len() as FrameId;
        for (k, &p) in order.iter().enumerate() {
            self.d.raw_by_addr.insert(p, RAW_FRAME | (base + k as FrameId));
        }
        for &p in &order {
            let rom = self.rom;
            let list_addr = rom.u32(p);
            let b7 = rom.u8(p + 7);
            let f = RawFrame {
                rom_addr: p,
                list_addr,
                sprites: raw_list(rom, list_addr),
                param_a: rom.u8(p + 4),
                param_b: rom.u8(p + 5),
                kind: rom.u8(p + 6),
                duration: b7 & 0x7F,
                last: b7 & 0x80 != 0,
                next: self.d.raw_by_addr[&self.next_addr(p, 8)],
            };
            self.d.raw_frames.push(f);
        }
        self.d.raw_by_addr[&start]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// Needs the user's ROM set at `../dino.zip` (skipped otherwise).
    #[test]
    fn effect_data_shapes() {
        let path = Path::new("../../../dino.zip");
        if !path.exists() {
            return;
        }
        let rom = Rom::load(path).unwrap();
        let d = extract(&rom, &crate::ExtraFrames::default());
        // Hit spark table: 27 ids, id 0 = 3 frames 0x15F4 / 0x15F8 / 0x15FC.
        assert_eq!(d.tables[&0x924F6].len(), 27);
        let f = d.frame(d.anim(0x924F6, 0));
        assert_eq!((f.rom_addr, f.w0, f.duration), (0x9252C, 0x15F4, 2));
        // Ripple: 3 raw frames, first list 0x9637A = one 16x16 tile 0x4534 at (-8, -16).
        let r = d.raw(d.raw_at(0x96360));
        assert_eq!((r.list_addr, r.sprites[0].code, r.sprites[0].dx, r.sprites[0].dy), (0x9637A, 0x4534, -8, -16));
        assert_eq!(d.raw(d.raw(d.raw(r.next).next).next).rom_addr, 0x96360);
        // Shadow type 0: one 3-wide sprite 0x1FFD at dx -24. Splash type 0 phase 0: 0x500D.
        assert_eq!((d.shadows[0][0].code, d.shadows[0][0].dx, d.shadows[0][0].attr), (0x1FFD, -24, 0x200));
        assert_eq!((d.splashes[0][0][0].code, d.splashes[0][0][0].dy), (0x500D, -40));
        assert_eq!(d.spark_jitter[0], (-2, 0));
    }
}
