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
    (0x44672, 9 * 32, "Vice arena cell transitions"),
    (0x44868, 9 * 4, "Vice arena destinations"),
    (0x44ADE, 16, "Vice approach choice"),
    (0x44F4E, 32 * 2, "Vice walk speed ids"),
    (0x442E0, 0x36, "Vice initial target weights"),
    (0x44B84, 0x40, "Vice attack chance"),
    (0x44C8C, 0x40, "Vice combo choice"),
    (0x43004, 13 * 8, "Vice combo scripts"),
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
