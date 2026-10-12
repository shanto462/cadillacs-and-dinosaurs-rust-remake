//! Player movement and combat tables (`re/specs/player/SPEC.md`). Filled by
//! `dino-extract/src/player.rs`; read by `dino-game/src/game/player*.rs`.
//!
//! Every field names the ROM address of its table. Per-character arrays are indexed by the
//! character id in obj+$03: 0 Jack, 1 Mustapha, 2 Hannah, 3 Mess.

use crate::AttackRecord;

/// One jump start vector (0x5364): 8.8 velocities and accelerations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JumpVector {
    pub vx: i16,
    pub ax: i16,
    pub vy: i16,
    pub ay: i16,
    pub vz: i16,
    pub az: i16,
}

/// One floor segment (12 bytes, table 0x12C30, read by 0x12FE4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FloorSegment {
    /// +0: 0 = flat floor, else a slope.
    pub slope: i16,
    /// +2: first X of the segment (also read by the pointer advance 0x12C90).
    pub x0: i16,
    /// +4: last X; the lookup takes the first segment with `x1 >= X`.
    pub x1: i16,
    /// +6 word: slope base height (its low byte +7 goes to obj+$52 for flat floors).
    pub base: i16,
    /// +8: flat floor height, or slope direction (0 = rising).
    pub height: i16,
    /// +$A: wading flag copied to obj+$53.
    pub wading: u8,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerData {
    /// 0x5064: walk / run speed tables, index `class/4*4 + character` (class 0, 4, 8, 12).
    /// Each table: (vx, vz) for directions U, UR, R, DR, D, DL, L, UL (8.8).
    pub speed: [[[i16; 2]; 8]; 16],
    /// 0x50A4: the same while carrying object 0x20A.
    pub speed_heavy: [[[i16; 2]; 8]; 16],
    /// The ROM pointers in those two tables (what 0x5036 stores in obj+$64).
    pub speed_ptr: [u32; 16],
    pub speed_heavy_ptr: [u32; 16],
    /// 0x53A2: jump vectors, index `class/4*4 + character`, then entry 0 = no direction,
    /// 1..8 = U, UR, R, DR, D, DL, L, UL.
    pub jump: [[JumpVector; 9]; 16],
    /// 0x1BE66: air control (dvx, dvz) per direction.
    pub air_control: [[i16; 2]; 8],
    /// 0x19292: stick nibble -> direction 0..7 (0xFF = impossible combination).
    pub dir8: [u8; 16],
    /// 0x1BDCC: stick nibble -> (byte offset into a speed table, obj+$22 direction value).
    pub walk_dir: [[u8; 2]; 16],
    /// 0x1A6E8: stick nibble -> obj+$22 direction for jumps.
    pub jump_dir: [u8; 16],
    /// 0x6C3A: invulnerability frames, index `d0 + character` (d0 0 getup, 4 throw, 8 pickup, 12 special).
    pub invuln: [u8; 16],
    /// 0x6B7C / 0x6B84: command input step timers per character.
    pub command_time: [[u8; 4]; 2],
    /// 0x4E56: stage start positions [stage][area][player] = (x, y, z).
    pub start_pos: Vec<[[[i16; 3]; 3]; 3]>,
    /// 0x12C30: floor segments [stage][area] (empty for areas that do not exist).
    pub floor: Vec<Vec<Vec<FloorSegment>>>,
    /// 0xA26FC: attack records per character, indexed by obj+$58 / 2 (real table lengths).
    pub attacks: [Vec<AttackRecord>; 4],
    /// 0xA2EA4: enemy attack records, indexed by the enemy's obj+$58 / 2.
    pub enemy_attacks: Vec<AttackRecord>,
    /// 0x2C9B6: attack power factor rows (x/32), 4 rows of 32 by own HP.
    pub power: Vec<u8>,
    /// 0x2C88E: damage taken factor per character (x/32).
    pub defense: [u16; 4],
    /// 0x2C896: random damage reduction words, 4 rows of 32 by own HP.
    pub enemy_hp_reduce: Vec<u16>,
    /// 0x2C996: damage factor by own HP (x/32), index (HP >> 2) & 0x1E as words.
    pub enemy_hp_scale: [u16; 16],
    /// 0x2C3AE: enemy damage factor bytes (x/32) at `enemy+$84 * 8 + (level & 0x1F)`.
    pub enemy_level_scale: Vec<u8>,
    /// 0xA320C: counter-hit factor words (x/32) by the enemy's obj+$58 / 2 (0 = none).
    pub counter_scale: Vec<u16>,
    /// 0xA32FA: weaknesses at `enemy+$84`: (attack id / 2, factor, attack id / 2, factor).
    pub weakness: Vec<u8>,
    /// 0x2BECE: damage factor bytes (x/32) at `enemy+$84 * 8 + level`.
    pub level_type_scale: Vec<u8>,
    /// 0x2BC4E: factor words by hundreds of enemy HP above 100 (rows of 32 words).
    pub hp_hundreds_scale: Vec<u16>,
    /// 0x2BB4E: factor words by enemy HP band (4 rows of 32 words).
    pub hp_band_scale: Vec<u16>,
    /// 0x1D27C: pushback reactions (frames, vx, ax) for 0x0E/0x14, 0x10/0x16, 0x12/0x18.
    pub pushback: [[i16; 3]; 3],
    /// 0x1D28E: knockdown vectors (vx, ax, vy, ay): reaction 4 alive / dead, 6 alive / dead,
    /// burn alive / dead.
    pub knockdown: [[i16; 4]; 6],
    /// 0x1D1C4: lie-down frames, rows of 16 by (level & 0x18) >> 3.
    pub lie_time: Vec<u8>,
    /// 0x1D204: mash meter after a knockdown, same layout.
    pub lie_mash: Vec<u8>,
    /// 0x1CF5C: dizzy mash meter, same layout.
    pub dizzy_stars: Vec<u8>,
    /// 0x1CEDC: dizzy maximum frames (words), same layout.
    pub dizzy_time: Vec<u16>,
    /// 0x1D166: dizzy chance masks by recent hit count (0..5).
    pub dizzy_mask: [u16; 6],
    /// 0x605E: mash meter decrease per newly pressed stick nibble.
    pub mash_weight: [u8; 16],
    /// 0x2CB32: thrown enemy offset (dx for facing left, dy), index obj+$20 / 4 (+1 for Mess' back throw).
    pub throw_offset: [[i16; 2]; 5],
    /// 0x1D93E: knee spark offsets (dx, dy), index character * 2 + (knee 2 ? 1 : 0).
    pub knee_spark: [[i16; 2]; 8],
    /// 0x1A734: jump attack (minimum height, high/low split) per character.
    pub jump_attack_height: [[i16; 2]; 4],
    /// 0x1A994: dive kick (vx, vy) per character.
    pub dive: [[i16; 2]; 4],
    /// 0x1BF06..0x1BF41 (word index = (address - 0x1BF06) / 2), (vx, ax, vy, ay) blocks:
    /// [0] Jack command move 0x1BF06, [4] Mustapha 0x1BF0E, [8] Hannah 0x1BF16,
    /// [12] state 0x24 (vx, ax) 0x1BF1E, [14] Mess 0x1BF22, [18] / [22] / [26] airborne lunge of
    /// characters 1 / 2 / 3 (0x1BF2A / 0x1BF32 / 0x1BF3A).
    pub vectors_1bf06: [i16; 30],
    /// 0x19CD8: recoil vx per character (weapon type 0x209).
    pub recoil: [i16; 4],
    /// 0x1A902: fall animation (set 5) while carrying object 0x20A, per character.
    pub heavy_fall_anim: [u8; 4],
    /// 0x1B0C6 / 0x1B0F2: jump and landing animations (set 5) while carrying 0x20A.
    pub heavy_jump_anim: [u8; 4],
    pub heavy_land_anim: [u8; 4],
    /// 0x2CBE0..0x2D0E0 as bytes: where a player held by an enemy is placed (0x1D4EA):
    /// long[0x2CBE0 + player+$20] -> word offset table by enemy+$84 / 2 -> rows of
    /// (dx, dy, facing, anim) words by the enemy's frame byte +$2D & 0x3F.
    pub held_offsets: Vec<u8>,
    /// 0x10292: BCD score values (longs); 0xFFC8 adds the long at byte offset `record byte 5`.
    pub score_bcd: Vec<u32>,
    /// 0x1C8A6: the two flames of a burn knockdown: (dx, dy, var).
    pub burn_flames: [[i16; 3]; 2],
}

/// ROM address of `PlayerData::held_offsets[0]`.
pub const HELD_OFFSETS_BASE: u32 = 0x2CBE0;
