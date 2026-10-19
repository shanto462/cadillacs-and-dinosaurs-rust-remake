//! Extracts the boss reinforcement lists (pool B class 22, 0x7BEF6). Formats: D3c section 1.7.

use crate::enemy::{delay_list, wave};
use crate::rom::Rom;
use dino_assets::boss::*;

/// Timed lists by stage (long pointers, read by the init 0x7BF0A).
const BOSS_TIMED_TABLE: u32 = 0x7BF36;
/// Threshold lists by stage.
const BOSS_GROUP_TABLE: u32 = 0x7BF56;
/// The three timed list limits by players - 1 (0x7BFEE, bytes).
const ALIVE_LIMIT: u32 = 0x7BFEE;

/// Vice tables used by 0x425B6 (target selection) and the later AI tasks.
/// Copied only from the user's ROM at extraction time into EnemyData::t.
pub const TABLES: &[(u32, u32, &str)] = &[
    (0x44370, 0x36, "Vice retarget weights"),
    (0x444C0, 32, "Walther initial cell"),
    (0x44572, 8 * 32, "Walther cell transitions"),
    (0x44812, 16 * 4, "Walther target offsets"),
    (0x44672, 9 * 32, "Vice arena cell transitions"),
    (0x44868, 9 * 4, "Vice arena destinations"),
    (0x44ADE, 16, "Vice approach choice"),
    (0x44F4E, 3 * 32 * 2, "Vice walk speed ids"),
    (0x442E0, 0x36, "Vice initial target weights"),
    (0x44B84, 0x40, "Vice attack chance"),
    (0x44C8C, 0x40, "Vice combo choice"),
    (0x43004, 13 * 8, "Vice combo scripts"),
    (0x429AE, 12, "Vice counterattack chance"),
    (0x42A24, 96, "Vice counterattack choices on screen"),
    (0x42A84, 96, "Vice counterattack choices off screen"),
    (0x436B4, 32, "Vice pause distribution"),
    (0x436D4, 32, "Vice pause lengths"),
    (0x44C06, 13 * 2, "Vice combo approach distances"),
    (0x44D8A, 4, "Vice run brake windows"),
    (0x44DB8, 4, "Vice run arrival windows"),
    // The Butcher (M6-09): the knife holder offsets (0x57EA2, both tables), the turn table of the state "stand and wait"
    // (0x59822, read at byte offsets up to 0x1E and a word more) and the slot offsets of the library 0x32E0A (X and Z, 7 words each).
    (0x57F0E, 0x30, "Butcher knife holder offsets"),
    (0x5986C, 0x24, "Butcher stand and wait turn table"),
    (0x32EA4, 0x1C, "Butcher slot offsets (0x32E0A)"),
    // The Butcher's alive code (M6-10): the leap roll limits (0x58AB8), the aggression toggle timers (0x58916), the decision chances (0x5A1AC)
    // and the decision rows by knives held (0x5A1EA: three tables of 256), the combo rows (0x59F0C, 0x5948C), the hop speeds (0x598F2), the
    // bounce counts (0x59B70), the ring of hop points (0x32AE4), the pause tables of the walk (0x589FA) and of the combo (0x59796) and the
    // knife offsets of the hand (0x5A2A4: two tables of 27 rows of 8 bytes).
    (0x58B0C, 4, "Butcher leap roll limits"),
    (0x74502, 0x30, "Butcher aggression toggle timers"),
    (0x73E94, 0x40, "Butcher decision chances"),
    (0x73ED4, 0x300, "Butcher decision rows by knives held"),
    (0x59FAA, 0x10, "Butcher combo start rows"),
    (0x5951E, 0x14, "Butcher combo animations"),
    (0x59988, 0x28, "Butcher hop speeds"),
    (0x59CFC, 0x10, "Butcher bounce counts"),
    (0x57480, 0x1B0, "Butcher knife offsets by frame"),
    (0x32B48, 0x18, "ring of hop points (0x32AE4)"),
    (0x744DA, 0x30, "Butcher walk pause tables"),
    (0x74DA2, 0x40, "Butcher combo pause by random"),
    (0x74DE2, 0x40, "Butcher combo pause by level"),
    // Hogg (M6-23 / M6-24, classes 31 and 32): the first target cycle (0x4FE06), the rider offsets on the bike (0x5078E), the entry points
    // by side (0x507DC) and their weights (0x5083E), the throw limits by level (0x50AD8), the target rows (0x50C08), the death speeds
    // (0x5040E / 0x5062E and the bounce 0x5049A / 0x506BA), the knock back of a car ram (0x5059A); the car mode zone maps: the 256 cell
    // grid of aim offsets (0xC821A, dx and dz words), the cell transition rows (0xC6252, 256 rows of 32), the cruise times (0xC861A) and
    // the unused re-aim times (0xC865A, read and overwritten at 0x5014E).
    (0x4FE46, 0x10, "Hogg first target cycle"),
    (0x507D0, 0x0C, "Hogg rider offsets on the bike"),
    (0x50816, 0x28, "Hogg entry points by side"),
    (0x5083E, 0x20, "Hogg entry point weights"),
    (0x50B0A, 0x20, "Hogg throw limits by level"),
    (0x50C62, 0x36, "Hogg target rows by player count"),
    (0x504E0, 0x0C, "Hogg death speeds"),
    (0x50616, 0x04, "Hogg car ram knock back"),
    (0xC821A, 0x400, "Hogg car mode aim grid"),
    (0xC6252, 0x2000, "Hogg car mode cell transitions"),
    (0xC861A, 0x40, "Hogg car mode cruise times"),
    (0xC865A, 0x40, "Hogg car mode re-aim times"),
    // Hogg's bomb, class 42 (M6-25): the landing cell list by rand() & 15 (0x55EB0) and the nine landing offsets around the car (0x55EB6).
    (0x55ED4, 0x20, "Hogg bomb landing cells"),
    (0x55EF4, 0x24, "Hogg bomb landing offsets"),
    // The car, pool B class 88 (M6-18 / M6-19): the stick to pair offsets (0x8C364), the two equal pair tables (0x8C36C / 0x8C378), the body
    // bump steps (0x8C42C, read up to +$AE = 0x18) and the front wheel frames (0x8C5BA).
    (0x8C886, 0x10, "car stick to pair offsets"),
    (0x8C896, 0x20, "car steering pairs (the copy while +$119 runs)"),
    (0x8C8B6, 0x20, "car steering pairs"),
    (0x8C404, 0x19, "car body bump steps"),
    (0x8C5CC, 0x06, "car front wheel frames"),
    // The seat drawers of the car (M6-21, 0x9704E): the offset of each seat from the car (dx, dy, dz words).
    (0x970F6, 0x12, "car seat offsets"),
    // The seat drawers' animation offsets (0x97150, 8 words) and the records' +$A2..+$A4 bytes read after a start (0x970BE, 0x9720C..0x97276).
    (0x97150, 0x128, "seat drawer animation offsets and record bytes (0x97008, 0x970BE)"),
    // The car's dust and smoke (M6-22, 0x97834): four drifts of two 16.16 longs (X, Y).
    (0x978CE, 0x20, "car puff drifts"),
    // The bomb carriers (M6-27): the byte code lists of class 64 (0x5B318 by the variant high byte, 0x5B4D8 by the low byte) and the hand
    // offsets of the dynamite class 77 (0x5EEE0: two tables of 16 rows of dx, dy words).
    (0x5B326, 0x08, "bomb carrier first lists"),
    (0x5B4E6, 0x08, "bomb carrier run lists"),
    (0x5EF2A, 0x80, "dynamite hand offsets"),
    // Slice (M7-01 .. M7-04, class 25, 0x4BE70..0x4D48A): the whole handler, whose tables sit between its routines (the wake-up roll 0x4C0FC,
    // the scripts 0x4C4AE, the boomerang and fan tables 0x4C61A / 0x4C79A / 0x4C7BA, the kick pauses 0x4C870 / 0x4C890, the hero tables
    // 0x4CBC6 / 0x4CC56, the points 0x4CDBE / 0x4CEF2 / 0x4CF52 / 0x4CF72 / 0x4CFD2, the run kinds 0x4D194, the gate 0x4D268, the scripts by HP
    // and distance 0x4D2E8, the speeds 0x4D448 / 0x4D476). The boomerang, class 19 (0x45082..0x45706): the fan speeds 0x453FE and the catch
    // box 0x45694. The afterimage palettes (0x9644A).
    (0x4BE70, 0x161A, "Slice handler and tables"),
    (0x45082, 0x684, "boomerang handler and tables"),
    (0x9644A, 0x06, "afterimage palettes"),
    // D3d shared bosses (M7-13): timers, aggression gates (8 x 8),
    // and action choices (32 x 8). Only copied from the user ROM.
    (0x5196A, 0x40, "Morgan phase timers"),
    (0x5418A, 0x40, "Morgue phase timers"),
    (0x36D36, 0x40, "Tyrog phase timers"),
    (0x6575C, 0x40, "Tyrog 2 phase timers"),
    (0x68080, 0x40, "Fessenden phase timers"),
    (0x73B14, 0x40, "Morgan aggression gate"),
    (0x73B54, 0x200, "Morgan action choices by AB sign"),
    (0x73D54, 0x40, "Morgue aggression gate"),
    (0x73D94, 0x100, "Morgue action choices"),
    (0x74542, 0xC0, "Tyrog aggression gates by form"),
    (0x74602, 0x300, "Tyrog action choices by form"),
    (0x74A42, 0x40, "Fessenden aggression gate"),
    (0x74A82, 0x100, "Fessenden action choices"),
    (0x32D7E, 0x0E, "boss four-slot offsets including six-slot carryover"),
    // The Episode 3 sky blend, pool B 95 (M6-31, 0x92220): the periods by variant / 2 (0x922C8) and the target / palette address pairs (0x922D0).
    (0x922C8, 0x40, "sky blend periods and targets"),
    // Tyrog (M8-07 / M8-08, class 4, 0x36A64..0x38DDE): the handler with its tables (the phase times 0x36D36 / 0x36D56, the waits 0x37C20,
    // 0x38214, 0x38290, 0x3868C, the leap speeds 0x38754, the rocks 0x37ECA), the boss data block 0x74542..0x74F42 (gates, actions, sidesteps,
    // the claw chain waits, the level delays) and the windows of 0x329B8 (0x32A28).
    (0x36A64, 0x237A, "Tyrog handler and tables"),
    (0x74542, 0xA00, "Tyrog gates, actions and moves"),
    (0x32A28, 0x30, "distance class windows of 0x329B8"),
    // The bosses' fire, class 26 (M8-09, 0x4D48A): the landing spot offsets 0x4D49E / 0x4D51E; the chest blades, class 74 (0x5E4F4): the spark
    // offsets 0x5E6D4.
    (0x4D49E, 0x100, "boss fire landing spots"),
    (0x5E6D4, 0x20, "chest blade spark offsets"),
];

pub fn extract(rom: &Rom) -> BossData {
    BossData {
        stages: (0..8u32)
            .map(|st| {
                let timed_addr = rom.u32(BOSS_TIMED_TABLE + 4 * st);
                BossLists { timed: delay_list(rom, timed_addr), timed_addr, groups: wave(rom, rom.u32(BOSS_GROUP_TABLE + 4 * st)) }
            })
            .collect(),
        alive_limit: std::array::from_fn(|k| rom.u8(ALIVE_LIMIT + k as u32)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// 0x5196A..0x74B82: verify the registered boss tables against the
    /// user's ROM. Expected bytes stay in the zip, never in the test source.
    #[test]
    fn shared_boss_tables_are_exact_runtime_copies() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dino.zip");
        if !path.exists() { return; }
        let rom = Rom::load(&path).unwrap();
        let data = crate::enemy::extract(&rom);
        for &(addr, size, _) in TABLES {
            let blob = data.t.blobs.iter().find(|(a, b)| *a == addr && b.len() == size as usize)
                .unwrap_or_else(|| panic!("boss table {addr:#x} not extracted"));
            for offset in 0..size {
                assert_eq!(blob.1[offset as usize], rom.u8(addr + offset), "table {addr:#x}+{offset:#x}");
            }
        }
    }

    /// Needs the user's ROM set at `../../../dino.zip` (skipped otherwise). The checks follow the group
    /// order and the thresholds of D3c section 1.7 (Vice, stage 0).
    #[test]
    fn lists_decode_in_the_order_of_d3c_1_7() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dino.zip");
        if !path.exists() {
            eprintln!("boss list test skipped: user's dino.zip is absent");
            return;
        }
        let rom = Rom::load(&path).unwrap();
        let b = extract(&rom);
        assert_eq!(b.stages.len(), 8);
        assert_eq!(b.alive_limit, [4, 5, 6]);

        // Episode 1 (Vice): five timed records, the first after 480 frames and a Ferris / Gneiss / Driver
        // (name id 2), then the end record with a negative delay.
        let vice = &b.stages[0];
        assert_eq!(vice.timed.len(), 6);
        assert_eq!(vice.timed[0].trig, 480);
        assert_eq!(vice.timed[0].class / 4, 2);
        assert!((vice.timed[5].trig as i16) < 0);

        // Seven threshold groups, in order: group 0 at 99 alive or fewer, group 1 at 1 for one player,
        // groups 3, 4 and 6 at 5, 3 and 4 (the one player column of D3c section 1.7).
        let g = &vice.groups;
        assert_eq!(g.groups.len(), 7);
        assert_eq!(g.groups[0].thr, [99, 99, 99]);
        assert_eq!(g.groups[1].thr[0], 1);
        assert_eq!(g.groups[3].thr[0], 5);
        assert_eq!(g.groups[4].thr[0], 3);
        assert_eq!(g.groups[6].thr[0], 4);
        assert_eq!(g.final_thr, [0, 0, 0]);
        // Group 0 drop rows: 7 on the first record, 4 on the third (its delay is 5 frames).
        assert_eq!(g.groups[0].spawns[0].b96, 7);
        assert_eq!(g.groups[0].spawns[2].b96, 4);
        assert_eq!(g.groups[0].spawns[2].trig, 5);

        // Every stage has threshold groups with records, and its threshold list ends with the end word.
        for (st, lists) in b.stages.iter().enumerate() {
            assert!(!lists.groups.groups.is_empty(), "stage {st} has no threshold group");
            assert!(lists.groups.groups.iter().all(|grp| !grp.spawns.is_empty()));
            assert!((lists.groups.end as i16) < 0, "stage {st} threshold list has no end word");
        }
    }
}
