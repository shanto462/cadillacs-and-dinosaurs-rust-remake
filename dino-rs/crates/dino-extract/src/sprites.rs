//! Extracts sprite data. Formats: `re/specs/anim/SPEC.md`.

use crate::rom::Rom;
use dino_assets::*;
use std::collections::{HashMap, VecDeque};

/// Pointer to the composition pointer table (u32) and its entry count (u16).
const COMP_TABLE_PTR: u32 = 0x0FFFF4;
const COMP_COUNT_PTR: u32 = 0x0FFFF8;
/// Pointer to the layout pointer table (u32).
const LAYOUT_TABLE_PTR: u32 = 0x0FFFFA;
/// Per-composition extra bytes (wading clip counts, shadow type).
const COMP_EXTRA: u32 = 0x11D220;
/// Player animation sets 0..5: each holds 4 table pointers (one per character).
const PLAYER_SETS: [u32; 6] = [0x163E, 0x164E, 0x165E, 0x166E, 0x167E, 0x168E];
const PLAYER_BOX_TABLES: u32 = 0x18CC4;
const PLAYER_ATTACKS: u32 = 0xA26FC;
/// Records copied per character. The original indexes the table by the attack id / 2 with no bound (an id is a byte), so all 128 are
/// copied: an id past the character's own table reads the bytes that follow, as the original does (Mustapha's 0x7A in the attract demo).
const ATTACKS_PER_CHARACTER: u32 = 128;
/// Region that holds every hit box table.
/// It starts 0x8000 bytes before the first table (0x100000): a box test that adds a stale register word to a table (0x128F8 with the d0
/// that the car's container pass 0x122D8 keeps between objects, M6-20) reaches back that far (`adda.w`, sign extended).
const HITBOX_REGION: (u32, u32) = (0xF8000, 0x11D220);

/// 10-byte animation tables used by non-player objects. Each address is a constant in the
/// object's init code (stored into obj+$40); the comment names that code.
pub const OBJECT_ANIM_TABLES: &[u32] = &[
    0x6162E, // A83 flame pieces, 0x61570
    0x93F42, // burn death pieces, pool C class 15, 0x93E68 (M5-07)
    // The Butcher (M6-09): the starter class 5 (0x38E20), the carcass class 51 (0x5719E), the machete helper class 53 (0x57650), the knife
    // holder class 58 (0x57E78), the fighter class 61 (0x584C4), the knives class 52 (0x572C0) and his hit (0x2E68C, +$84 = 0x3C) / held
    // (0x2E724) tables.
    0x3906A, 0x5727A, 0x57682, 0x57F44, 0x6FDEC, 0x6F8A6, 0x30750, 0x311C4,
    0x4F4E4, // Rock Hopper init 0x4DD2A (M5-10)
    0x53BE4, // Mack init 0x53430 (M6-03)
    0x49574, // Shivat init 0x483B2 (M6-04)
    0x5F4F8, 0x600E0, // ground critters A78 (0x5F1AC) and A80 (0x5FF42) (M6-03)
    0x6FB1A, // enemy bullet class 59, init 0x57FC8
    0x2FDC6, 0x30FFC, // Walther hit / held tables, 0x2E68C / 0x2E724 +$34
    0x43E54, // Walther, 0x425F8
    0x438F2, // Vice T. intro, 0x425B6 (M5-09)
    0x50DFA, 0x51076, // Hogg: the rider class 31 (0x4FD6A) and the bike class 32 (0x50FD0) (M6-23)
    0x55BF6, // Hogg's bomb class 42 (0x55B30) (M6-25)
    0x8C8D6, // the Cadillac, pool B class 88 (0x8BE24) (M6-18)
    0x5B606, 0x5B77C, // the bomb carriers class 64: Punk alt and Wrench T. alt (0x5B068, 0x5B092) (M6-27)
    0x5F0AC, 0x5F128, // their dynamite class 77 (0x5EE16, 0x5EE20) (M6-27)
    0x5BAE0, // barrel class 65 (0x5B96A) (M6-29)
    0x4C90E, 0x45706, // Slice class 25 (0x4BEA8) and his boomerang class 19 (0x450B4) (M7-01, M7-03)
    0x302BA, 0x310E0, // Slice's hit reactions and held poses (by name 5 in the reaction tables) (M7-01)
    0x5BDA6, // rock class 66 (0x5BC2E) (M6-30)
    0x5C6F0, // biker class 70 (0x5C412) (M6-29)
    0x2FBE2, // Vice T. hit reactions (table 0x2E68C, +$84 = 0x30)
    0x30FB0, // Vice T. held by a player (table 0x2E724, +$84 = 0x30)
    0x34892, // enemy family init 0x3388A, variant 0 (palette 13)
    0x34B90, // same family, palette 14
    0x34E7C, // same family, palette 26
    0x2E7BC, // hit reactions (table 0x2E68C by +$84) of the class 2 variants: name 0
    0x2E928, // name 1
    0x2EA94, // name 0x1A
    // Held by a player (0x2B35A, table 0x2E724 by +$84): the class 2 variants.
    0x30B64, // name 0
    0x30BB0, // name 1
    0x30BFC, // name 0x1A
    // Pool A scenery and opening scene (phase 1 task 4a): classes 49, 50, 54, 55.
    0x56C38, // class 49 background birds
    0x5708C, // class 50 flying scenery
    0x577D2, // class 54 opening scene bystander
    0x57BA4, // class 55 scripted thug, Ferris
    0x57C20, // class 55, Gneiss
    0x57D84, // class 56 the flying cast member of the opening scene (M2-11)
    // Class 10 (phase 1 task 4a): animations, then the hit tables (0x2E68C by +$84) and the grabbed
    // tables (0x2E724) of the names HAMMER T. (0xC), BLK ELMER (0x28), WRENCH T. (0x2C).
    0x3D150, 0x3D400, 0x3D6B0, 0x2F480, 0x2F5E2, 0x2F744, 0x30E34, 0x30E80, 0x30ECC,
    // Class 20 (PUNK 0x47244, THUG 0x475FE) and their hit (0x2E68C) / grabbed (0x2E724) tables.
    0x47244, 0x475FE, 0x2FFAA, 0x30132, 0x31048, 0x31094,
    // Class 14 (BLADE 0x6DA40, RAZOR 0x6DF2A), the parts and thrown knives (classes 60 / 34 / 41, 0x6FB6A) and the hit / grabbed tables.
    0x6DA40, 0x6DF2A, 0x6FB6A, 0x2F8A6, 0x2FA44, 0x30F18, 0x30F64,
    // Class 13 (POACHER J 0x6C57A, SKINNER 0x6CA24, GUTTER 0x6CF3E; init 0x3DDFE) and their hit (0x2E68C, +$84 = 8 / 0x48 / 0x4C) / grabbed (0x2E724) tables.
    0x6C57A, 0x6CA24, 0x6CF3E, 0x2EFEC, 0x2F176, 0x2F2F6, 0x30D2C, 0x30D84, 0x30DDC,
    0x4200E, // class 15 flying pieces (0x41F2E)
    0x71F72, // class 98 title logo (0x6A21E): the "Cadillacs" / "Dinosaurs" letters
    0x71F9E, // class 100 profile card portraits and moves (0x6A608)
    // The jungle of the opening movie (M1-10): the two hunters (class 11, variants 0 and 1, 0x3D974), the golden raptor
    // (class 12, 0x3DC70), the puff of the hunter (class 57, 0x57DAA), the flock bird (class 46, 0x56598) and its feather (class 48, 0x5694A).
    0x3DAF8, 0x3DB00, 0x3DD50, 0x57E0E, 0x5674E, 0x56A56,
    // The brown dinosaur (class 30, 0x4FB24) and the two pieces of the big one (class 69, 0x5C2EC).
    0x4FC22, 0x718C2,
    0x8B5E6, // pool B class 86, the Cadillac of the ranking screen (0x8B43C)
    0x89736, // pool B class 70, decorative chief-scene rifle (0x896E6, M7-25)
    // Pool B class 19, the hero pictures of the select screen on the 2 player layout (0x7B7C6): the tables of the four cursor positions
    // (pointers at 0x7B85E; animation 0 = standing, 1 = picked).
    0x7B86E, 0x7B872, 0x7B876, 0x7B87A,
    0x42516, // pool A class 17, the two heroes in the Cadillac (0x423B8)
    0x42386, // pool A class 16, the blue Cadillac (0x420EA)
    // The ending (M3-23): the explosion clouds (class 90), the pieces (92, 93) and the flying pieces of the sunset (96).
    0x6531C, 0x678A0, 0x67C3A, 0x69E00,
    0x6A020, 0x6A08E, 0x6A0FC, 0x6A16A, // pool A class 97, the stand ins of the heroes (0x69EA2)
    0x604EA, // pool A class 81, the walkers and the Cadillac of the desert (0x60158)
    0x4DA2A, 0x4DA44, 0x4DA9A, // pool A class 27, the crouching men, variants 0, 2, 4 (0x4D900)
    0x56106, // pool A class 43, the orange leader of the Episode 4 opening (0x55F2A)
    // Pool E (items, phase 1 task 2). Containers 0x28C8A..0x28D1C, one table per variant.
    0x2995E, // container var 0
    0x2998C, // var 2
    0x299FE, // var 4
    0x29A66, // var 6
    0x29A86, // var 8
    0x29AA6, // var 0xA
    0x29AC8, // var 0xC
    0x2A174, // treasure (0x2A044)
    0x28B54, // food (0x289A8)
    // Weapons (0x24D8A..0x25020, by type), explosions, bullets, shells, projectiles, pellets.
    0x265F4, // type 0
    0x268BC, // 1
    0x26A3C, // 2
    0x26BBC, // 3
    0x26D3C, // 4
    0x26F7E, // 5
    0x270FE, // 6
    0x2727E, // 7
    0x273FE, // 8
    0x2757E, // 9
    0x276FE, // 10 / 11
    0x2787E, // 12 (time bomb)
    0x27B12, // 13 (fire bomb)
    0x27CD2, // 14 (explosive stick, +$5C = 0)
    0x27D1A, // 14 (+$5C = 1)
    0x27D62, // 14 (+$5C >= 2)
    0x28102, // explosions (class 1)
    0x28904, // bullets (class 3)
    0x29EB4, // shells (class 7)
    0x2A00A, // projectiles (class 8)
    0x2A49C, // pellets (class 10)
    0x611D2, // Zeke class 82, init 0x606E8 (M7-10)
    0x3A234, // Bludge class 6, init 0x392FE (M7-07)
    0x2EE70, // Bludge hit reaction table (0x2A606)
    0x30CE0, // Bludge grab animation table (0x2B35A)
];

/// Single frame records items load directly (0x1236): the ammo box picture.
pub const OBJECT_FRAMES: &[u32] = &[
    0x285DC,
    0x2AC1C, // the charred body of a thug that burned to death, 0x2AC4C (M5-07)
    // Pool B class 67 (the golden man and the others of the select screen, 0x8881A, pointers at 0x88872) and class 68 (the player
    // tags 1P, 2P, 3P, 0x888B2, pointers at 0x88A54).
    0x88882, 0x8888E, 0x8889A, 0x888A6, 0x88A60, 0x88A6C, 0x88A78,
    // Pool B class 1 (chain-link gate / fence, 0x75294).
    0x75400,
    // Pool B class 35 (the manhole cover grate, 0x81FDE, loaded by 0x81FF6).
    0x82172,
];

/// Single frame records the 12-byte loader 0x143E reads: pool B class 64, the four cage posts of the lift (pointer table 0x86BD4).
pub const OBJECT_FRAMES_12: &[u32] = &[0x86BE4, 0x86BF0, 0x86BFC, 0x86C08];

pub fn extract(rom: &Rom, extra: &crate::ExtraFrames) -> SpriteData {
    let mut data = SpriteData { compositions: compositions(rom), ..Default::default() };
    let mut walker = FrameWalker { rom, data: &mut data };

    for (set, &ptr) in PLAYER_SETS.iter().enumerate() {
        for ch in 0..4 {
            let table = rom.u32(ptr + 4 * ch);
            walker.data.player_sets[set][ch as usize] = table;
            walker.table(table, 12);
        }
    }
    for &table in OBJECT_ANIM_TABLES {
        walker.table(table, 10);
    }
    for &frame in OBJECT_FRAMES {
        walker.walk(frame, 10);
    }
    for &frame in OBJECT_FRAMES_12 {
        walker.walk(frame, 12);
    }
    // On demand (harness): frame records of objects whose animation table is not registered yet.
    for &addr in &extra.sprite {
        if addr % 2 == 0 && (addr as usize) + 16 < rom.program.len() {
            walker.walk(addr, 10);
        }
    }

    for ch in 0..4 {
        data.player_box_tables[ch] = rom.u32(PLAYER_BOX_TABLES + 4 * ch as u32);
        let base = rom.u32(PLAYER_ATTACKS + 4 * ch as u32);
        data.player_attacks[ch] = (0..ATTACKS_PER_CHARACTER)
            .map(|k| {
                let a = base + 8 * k;
                AttackRecord {
                    damage: rom.u8(a) as i8,
                    stun: rom.u8(a + 1),
                    reaction: rom.u8(a + 2),
                    hit_stop: rom.u8(a + 3),
                    sound: rom.u8(a + 4),
                    score: rom.u8(a + 5),
                    spark: rom.u8(a + 6),
                    b7: rom.u8(a + 7),
                }
            })
            .collect();
    }
    let (start, end) = HITBOX_REGION;
    data.hitboxes = HitBoxBank { base: start, words: (start..end).step_by(2).map(|a| rom.i16(a)).collect() };
    data
}

fn compositions(rom: &Rom) -> Vec<Composition> {
    let table = rom.u32(COMP_TABLE_PTR);
    let count = rom.u16(COMP_COUNT_PTR) as u32;
    let layouts = rom.u32(LAYOUT_TABLE_PTR);
    (0..count)
        .map(|i| {
            let a = rom.u32(table + 4 * i);
            let n = rom.u16(a + 2) as u32;
            let hattr = rom.u16(a + 4);
            let layout = rom.u32(layouts + 4 * rom.u16(a + 6) as u32);
            let alt = hattr & 0x8000 != 0;
            let entries = (0..n)
                .map(|k| {
                    let (code, attr) =
                        if alt { (rom.u16(a + 8 + 4 * k), rom.u16(a + 10 + 4 * k)) } else { (rom.u16(a + 8 + 2 * k), hattr) };
                    SpriteEntry { dx: rom.i16(layout + 4 * k), dy: rom.i16(layout + 4 * k + 2), code, attr }
                })
                .collect();
            Composition { rom_addr: a, alt, entries }
        })
        .collect()
}

struct FrameWalker<'a> {
    rom: &'a Rom,
    data: &'a mut SpriteData,
}

impl FrameWalker<'_> {
    fn comp_valid(&self, index: u16) -> bool {
        index % 4 == 0 && ((index / 4) as usize) < self.data.compositions.len()
    }

    /// Decodes one animation table and every frame reachable from it.
    fn table(&mut self, table: u32, stride: u8) {
        if self.data.anim_tables.contains_key(&table) {
            return;
        }
        // The length is not stored: the first animation starts right after the offsets.
        let offsets: Vec<u16> = {
            let mut first = u32::MAX;
            let mut v = Vec::new();
            let mut i = 0;
            while 2 * i < first {
                let off = self.rom.u16(table + 2 * i);
                if off != 0 {
                    first = first.min(off as u32);
                }
                v.push(off);
                i += 1;
            }
            v
        };
        let anims =
            offsets.iter().map(|&off| if off == 0 { None } else { self.walk(table.wrapping_add(off as i16 as i32 as u32), stride) }).collect();
        self.data.anim_tables.insert(table, AnimTable { rom_addr: table, stride, anims });
    }

    /// Decodes the frame at `start` and everything it links to. Returns `None` when the
    /// record is not valid frame data (an unused id that points at garbage).
    fn walk(&mut self, start: u32, stride: u8) -> Option<FrameId> {
        if let Some(&id) = self.data.frame_by_addr.get(&start) {
            return Some(id);
        }
        // Pass 1: discover every record reachable from `start`.
        let mut order = Vec::new();
        let mut queue = VecDeque::from([start]);
        let mut seen: HashMap<u32, ()> = HashMap::new();
        while let Some(p) = queue.pop_front() {
            if self.data.frame_by_addr.contains_key(&p) || seen.contains_key(&p) {
                continue;
            }
            if !self.record_valid(p, stride) {
                if p == start {
                    return None;
                }
                continue;
            }
            seen.insert(p, ());
            order.push(p);
            queue.push_back(self.next_addr(p, stride));
        }
        // Pass 2: assign ids, then decode with resolved links.
        let first_id = self.data.frames.len() as FrameId;
        for (k, &p) in order.iter().enumerate() {
            self.data.frame_by_addr.insert(p, first_id + k as FrameId);
        }
        for &p in &order {
            let frame = self.decode(p, stride);
            self.data.frames.push(frame);
        }
        Some(self.data.frame_by_addr[&start])
    }

    fn next_addr(&self, p: u32, stride: u8) -> u32 {
        let q = p + stride as u32;
        if self.rom.u8(p + 7) & 0x80 != 0 { q.wrapping_add(self.rom.i16(q) as i32 as u32) } else { q }
    }

    fn record_valid(&self, p: u32, stride: u8) -> bool {
        if p as usize + stride as usize + 2 > self.rom.program.len() {
            return false;
        }
        let (w0, w1) = (self.rom.u16(p), self.rom.u16(p + 2));
        if w0 & 0x8000 != 0 {
            let q = (p + 2).wrapping_add(self.rom.i16(p + 2) as i32 as u32);
            return (q as usize) + 2 < self.rom.program.len() && self.rom.u16(q) < 64;
        }
        self.comp_valid(w0) && (w1 & 0x8000 != 0 || self.comp_valid(w1))
    }

    fn decode(&self, p: u32, stride: u8) -> Frame {
        let rom = self.rom;
        let (w0, w1) = (rom.u16(p), rom.u16(p + 2));
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
        // Composition used for the extra table, by loader: 0x1236 (10-byte) / 0x143E (12-byte).
        let ci = match stride {
            12 => {
                if w1 & 0x8000 == 0 {
                    w1
                } else {
                    w0
                }
            }
            _ => {
                if w0 & 0x8000 != 0 {
                    w0 & 0x7FFF
                } else if w1 & 0x8000 == 0 {
                    w1
                } else {
                    w0
                }
            }
        };
        let extra = rom.u32(COMP_EXTRA.wrapping_add(ci as i16 as i32 as u32)).to_be_bytes();
        let b7 = rom.u8(p + 7);
        Frame {
            rom_addr: p,
            stride,
            w0,
            w1,
            draw,
            param_a: rom.u8(p + 4),
            param_b: rom.u8(p + 5),
            kind: rom.u8(p + 6),
            duration: b7 & 0x7F,
            last: b7 & 0x80 != 0,
            body_box: if stride >= 10 { rom.u8(p + 8) } else { 0 },
            attack_box: if stride >= 10 { rom.u8(p + 9) } else { 0 },
            attach: if stride == 12 { rom.u16(p + 10) } else { 0 },
            extra,
            next: self.data.frame_by_addr.get(&self.next_addr(p, stride)).copied().unwrap_or(u32::MAX),
        }
    }
}

/// OBJ palettes: the ROM palette bank (32 bytes per palette at 0xB7A52) and, per stage and area,
/// the 32 bank numbers that 0x8E4C copies into OBJ palette slots 0..31 (table 0x8E8E).
pub fn obj_palettes(rom: &Rom) -> (Vec<[u16; 16]>, HashMap<(u8, u8), [u16; 32]>) {
    const BANK: u32 = 0x0B7A52;
    const SETS: u32 = 0x008E8E;
    let mut sets = HashMap::new();
    let mut max = 0u16;
    for stage in 0..8u8 {
        for area in 0..4u8 {
            let list = rom.u32(SETS + stage as u32 * 16 + area as u32 * 4);
            if list == 0 || list as usize + 64 > rom.program.len() {
                continue;
            }
            let limit = ((rom.program.len() as u32 - BANK) / 32) as u16;
            let mut slots = [0u16; 32];
            for (i, slot) in slots.iter_mut().enumerate() {
                *slot = rom.u16(list + 2 * i as u32);
            }
            if slots.iter().any(|&n| n >= limit) {
                continue;
            }
            max = max.max(*slots.iter().max().unwrap());
            sets.insert((stage, area), slots);
        }
    }
    let bank = (0..=max as u32)
        .map(|n| {
            let mut pal = [0u16; 16];
            for (i, w) in pal.iter_mut().enumerate() {
                *w = rom.u16(BANK + n * 32 + 2 * i as u32);
            }
            pal
        })
        .collect();
    (bank, sets)
}
