//! Extracts enemy, stage-script and item data. Formats: `re/specs/enemy/SPEC.md`.

use crate::rom::Rom;
use crate::sprites::OBJECT_ANIM_TABLES;
use dino_assets::enemy::*;
use dino_assets::AttackRecord;
use std::collections::HashMap;

/// Spawn list pointer tables (by stage, then 4 areas): list 1 (14-byte records), list 2 (16-byte).
const LIST1_TABLE: u32 = 0xFC2E;
const LIST2_TABLE: u32 = 0xFC4E;
/// Wave script tables by stage (word offsets per wave), used by pool B class 15 (0x7B336).
const WAVE_TABLE: u32 = 0x7B37C;
/// Boss reinforcement lists by stage (pool B class 22, init 0x7BF0A).
const BOSS_TIMED_TABLE: u32 = 0x7BF36;
const BOSS_GROUP_TABLE: u32 = 0x7BF56;
/// Trigger values at or above this never fire (list sentinels).
const SENTINEL: u16 = 0x9FFF;

/// Small tables copied as byte blobs: (ROM address, length, user).
const TABLES: &[(u32, u32, &str)] = &[
    // score
    (0x10292, 0x90, "BCD score values (0x10036), indexed by byte offset"),
    (0x325F8, 0x28, "kill score offset by name id (0x325E6)"),
    // damage enemy -> player (0x6422)
    (0x2C3AE, 0x27 * 32, "damage scale by name id and level"),
    (0x2C896, 0x100, "random damage reduction by HP band"),
    (0x2C88E, 8, "character factor"),
    (0x2C996, 0x20, "low-HP factor"),
    // damage player -> enemy (0x2BA2A)
    (0xA320C, 0x100, "counter-hit factor by enemy attack id"),
    (0xA32FA, 0xA0, "weak-point attack ids and factors by +$84"),
    (0x2BECE, 0x27 * 8 + 0x20, "damage scale by +$84*8 + level"),
    (0x2BC4E, 0x240, "HP bar factor"),
    (0x2BB4E, 0x100, "in-bar HP factor"),
    // class 2 (FERRIS / GNEISS / DRIVER)
    (0x33B28, 0x40, "entrance 4 idle timers"),
    (0x33CDA, 0x30, "entrance 6 fade palette pointers"),
    (0x33FA4, 8, "ambush threshold by aggression"),
    (0x34186, 0x20, "pause at slot by level"),
    (0x34210, 8, "state E pause by aggression"),
    (0x34338, 0x60, "combo choice by variant"),
    (0x343E8, 0x30, "combo scripts"),
    (0x345D0, 0x20, "recovery index"),
    (0x345F0, 0x20, "recovery frames by level band"),
    (0x3524C, 0x404, "approach slots (+ the bytes after them, read by the slot-index quirk)"),
    (0x352F8, 0x20, "attack positions"),
    (0x3533C, 0x20, "attack side offset"),
    (0x353B6, 0x18, "attack chance at slot"),
    (0x3546A, 0x10, "reposition duration"),
    (0x354B2, 0x60, "reposition choice"),
    (0x35796, 0x60, "dash attack chance"),
    (0x35832, 0x30, "walk speed ids"),
    (0x35882, 0x30, "run speed ids"),
    (0x1033E, 0x40, "stage time by stage*8 + area*2 (0x1031A)"),
    // HP
    (0x31454, 0x27 * 64, "HP by name id and level"),
    (0x31E52, 10 * 48, "boss HP"),
    // rank / level / aggression
    (0x18094, 0x20, "initial aggression + 1 by stage*4 + area"),
    (0x180B4, 0x46, "initial rank: thresholds and row offsets"),
    (0x180FA, 0x140, "initial rank rows"),
    (0x1823A, 0x40, "per-stage rank delta / max / min"),
    (0x182E4, 0x100, "aggression Markov tables (0x182E4, 0x18364)"),
    (0x184C4, 0x1E, "extra-life rank table"),
    (0x1860C, 0xC0, "level by rank and players"),
    (0x187B8, 0x7C, "rank-up periods and rank constants"),
    // hit reaction (0x2A606)
    (0x2B158, 0x1B4, "dizzy masks (0x2B158, 0x2B168, 0x2B180, 0x2B200, 0x2B28C)"),
    (0x2B604, 0x290, "lie-down time: random add and rows by +$84"),
    (0x2B8C4, 0x40, "counter-attack delay"),
    (0x2CB46, 0x9A, "throw / slide / knockback records"),
    (0x2CB80, 0x20, "weapon break masks"),
    (0x2E68C, 0xA0, "hit animation tables by +$84"),
    (0x2E724, 0xA0, "grabbed animation tables by +$84"),
    // drops and items
    (0xB64D2, 0xE40, "drop rows: fallback 0xB64D2, containers 0xB64F2, enemies 0xB6D12"),
    (0x28B10, 0x48, "food heal and score offset"),
    (0x2A164, 0x10, "treasure score offsets"),
    // pool E: containers, hand positions, guns
    (0x29860, 0x0E, "container thrown attack id by variant"),
    (0x2986E, 0xF0, "container throw velocities: table of 4 pointers, then 4 x 7 records (vx, ax, vy, ay)"),
    (0x28F24, 8, "container fuse: blink count, blink period by stage"),
    (0x29552, 0x15A, "container debris: chance masks, then the piece lists 0x295F4 / 0x29650"),
    (0x24CC6, 0x20, "gun flag by weapon variant (0x24C3A)"),
    (0x2494A, 0x10, "gun-kind flag by item kind low nibble (0x248F6)"),
    (0x2CBA0, 0x20, "weapon attack tables (0x259C8)"),
    (0x2855A, 0x80, "ammo box refill by weapon variant: max ammo, kind, +$A2"),
    (0x122230, 0x600, "hand position of held items: 4 pointers, then (dx, dy) by character and attach index"),
    (0x120A20, 0x1810, "hand animation of held items: 4 pointers, then pointer rows by +$A2, then ids by attach index"),
    (0x26408, 0x200, "weapon tables: explosion chain offsets, throw vectors 0x26534.., hand offsets 0x265B4, depth speeds 0x265D4"),
    (0x28814, 0xE0, "bullet records (vx, animation, attack id, aim), pointer rows, vertical speeds, aim tables (class 3)"),
    (0x2A484, 0x18, "pellet records (vx, vy) by variant (class 10)"),
    (0x62AE, 0x28, "weapon reach box by character: 5 words (0x6248)"),
    (0x9E5C, 4, "bazooka sound by voice bank (0x9E08)"),
    (0xA498, 6, "shotgun sound by voice bank (0xA442)"),
    (0x9FAA, 12, "eating sound by voice bank, normal and Hannah rows (0x9F54)"),
];

pub fn extract(rom: &Rom) -> EnemyData {
    let mut d = EnemyData::default();
    for st in 0..8u32 {
        d.stages.push(stage_script(rom, st));
    }
    d.hp = (0..0x27u32).map(|i| std::array::from_fn(|l| rom.u16(0x31454 + 64 * i + 2 * l as u32))).collect();
    d.boss_hp = (0..10u32)
        .map(|i| std::array::from_fn(|p| std::array::from_fn(|l| rom.u16(0x31E52 + 48 * i + 16 * p as u32 + 2 * l as u32))))
        .collect();
    d.attacks = (0..0x80u32)
        .map(|k| {
            let a = 0xA2EA4 + 8 * k;
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
    // 0x17A4: speed table pointers at 0x17CE, each table 32 x (vx, vz).
    for i in 0..64u32 {
        let p = rom.u32(0x17CE + 4 * i);
        if !(0x120000..0x130000).contains(&p) {
            break;
        }
        d.speed_ptrs.push(p);
        d.speeds.entry(p).or_insert_with(|| std::array::from_fn(|k| (rom.i16(p + 4 * k as u32), rom.i16(p + 4 * k as u32 + 2))));
    }
    d.reverse_frames = reverse_frames(rom);
    d.t.blobs = TABLES
        .iter()
        .map(|&(a, n, _)| (a, (a..a + n).map(|x| rom.u8(x)).collect()))
        .collect();
    // Palette blocks pointed to by the entrance 6 fade table (16 colours each).
    let mut extra = Vec::new();
    for k in 0..12u32 {
        let p = rom.u32(0x33CDA + 4 * k);
        extra.push((p, (p..p + 32).map(|x| rom.u8(x)).collect()));
    }
    d.t.blobs.extend(extra);
    d
}

fn rec(rom: &Rom, a: u32, long: bool) -> SpawnRec {
    SpawnRec {
        trig: rom.u16(a),
        flags: rom.u16(a + 2),
        class: rom.u16(a + 4),
        x: rom.u16(a + 6),
        z: rom.u16(a + 8),
        var: rom.u16(a + 10),
        wc: rom.u16(a + 12),
        b5b: if long { rom.u8(a + 14) } else { 0 },
        b96: if long { rom.u8(a + 15) } else { 0 },
    }
}

/// A trigger list: records until (and including) the first sentinel.
fn list(rom: &Rom, start: u32, long: bool) -> Vec<SpawnRec> {
    let size = if long { 16 } else { 14 };
    let mut v = Vec::new();
    for k in 0..400 {
        let r = rec(rom, start + size * k, long);
        v.push(r);
        if r.trig >= SENTINEL {
            break;
        }
    }
    v
}

/// Wave format: { thr[3], (16-byte record)+, 16-byte terminator }* thr[3] end_word.
fn wave(rom: &Rom, start: u32) -> Wave {
    let mut w = Wave { addr: start, ..Default::default() };
    let mut a = start;
    for _ in 0..64 {
        let thr = [rom.u16(a), rom.u16(a + 2), rom.u16(a + 4)];
        a += 6;
        if (rom.u16(a) as i16) < 0 {
            w.final_thr = thr;
            w.end = rom.u16(a);
            return w;
        }
        let mut g = WaveGroup { thr, spawns: Vec::new() };
        while (rom.u16(a) as i16) >= 0 {
            g.spawns.push(rec(rom, a, true));
            a += 16;
        }
        a += 16;
        w.groups.push(g);
    }
    w
}

/// Records with a delay word until a negative delay (boss timed list).
fn delay_list(rom: &Rom, start: u32) -> Vec<SpawnRec> {
    let mut v = Vec::new();
    let mut a = start;
    for _ in 0..64 {
        let r = rec(rom, a, true);
        v.push(r);
        if (r.trig as i16) < 0 {
            break;
        }
        a += 16;
    }
    v
}

fn stage_script(rom: &Rom, st: u32) -> StageScript {
    let t1 = rom.u32(LIST1_TABLE + 4 * st);
    let t2 = rom.u32(LIST2_TABLE + 4 * st);
    let areas = (0..4)
        .map(|ar| {
            let (p1, p2) = (rom.u32(t1 + 4 * ar), rom.u32(t2 + 4 * ar));
            AreaScript { list1_addr: p1, list2_addr: p2, list1: list(rom, p1, false), list2: list(rom, p2, true) }
        })
        .collect();
    let base = rom.u32(WAVE_TABLE + 4 * st);
    let count = rom.u16(base) / 2;
    let waves = (0..count as u32).map(|k| wave(rom, base.wrapping_add(rom.i16(base + 2 * k) as i32 as u32))).collect();
    StageScript {
        areas,
        waves,
        boss_timed: delay_list(rom, rom.u32(BOSS_TIMED_TABLE + 4 * st)),
        boss_groups: wave(rom, rom.u32(BOSS_GROUP_TABLE + 4 * st)),
    }
}

/// 0x1380 steps an animation backwards: p = frame - 10; if the word at p is negative, the
/// target is p + 2 + i16(p + 2) and the carry is set. Precomputed for every frame of the
/// 10-byte object tables.
fn reverse_frames(rom: &Rom) -> HashMap<u32, (u32, bool)> {
    let mut m = HashMap::new();
    for &table in OBJECT_ANIM_TABLES {
        let mut first = u32::MAX;
        let mut i = 0;
        while 2 * i < first && i < 256 {
            let off = rom.u16(table + 2 * i);
            if off != 0 {
                first = first.min(off as u32);
                let mut a = table.wrapping_add(off as i16 as i32 as u32);
                for _ in 0..256 {
                    let p = a.wrapping_sub(10);
                    let prev = if (rom.u16(p) as i16) < 0 { (p + 2).wrapping_add(rom.i16(p + 2) as i32 as u32) } else { p };
                    m.insert(a, (prev, (rom.u16(p) as i16) < 0));
                    if rom.u8(a + 7) & 0x80 != 0 {
                        break;
                    }
                    a += 10;
                }
            }
            i += 1;
        }
    }
    m
}
