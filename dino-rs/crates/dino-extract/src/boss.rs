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
    (0x442E0, 0x36, "Vice initial target weights"),
    (0x44B84, 0x40, "Vice attack chance"),
    (0x44C8C, 0x40, "Vice combo choice"),
    (0x43004, 13 * 8, "Vice combo scripts"),
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
