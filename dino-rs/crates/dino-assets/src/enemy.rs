//! Enemy, stage-script and item data (owner: enemy port). Formats: `re/specs/enemy/SPEC.md`.
//!
//! Structured data (spawn lists, wave scripts, attack records, speed tables, HP rows) has its own
//! types. The many small AI tables (choice rows, timers, masks) are kept as byte blobs named by
//! their ROM address (`ByteTables`): the ported code indexes them exactly like the 68000 code
//! does (`t.u8(0x353B6, row + aggr)`), which keeps out-of-range quirks visible.

use crate::sprites::AttackRecord;
use std::collections::HashMap;

/// One spawn record. List 1 records are 14 bytes (`b5b`/`b96` are 0), list 2 and wave records 16.
/// In wave scripts and boss reinforcement lists `trig` is the delay in frames.
/// Command records (flags bit 7) use `x` (+6) and `wc` (+C) as arguments.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpawnRec {
    /// +0 camera X (or Y) trigger, or delay.
    pub trig: u16,
    /// +2 flags: bit 7 command; bits 5-6 player count; bits 3-4 level test against the high byte;
    /// bits 0-2 pool (0 A, 1 B, 3 E).
    pub flags: u16,
    /// +4 class * 4 (pool dispatch offset), or the command number.
    pub class: u16,
    /// +6 X.
    pub x: u16,
    /// +8 Z.
    pub z: u16,
    /// +A var word (obj+$26).
    pub var: u16,
    /// +C list 1: Y (obj+$0C); list 2: obj+$5C.
    pub wc: u16,
    /// +E obj+$5B, +F obj+$96 (item drop row).
    pub b5b: u8,
    pub b96: u8,
}

/// Spawn lists of one area (0xFC2E list 1, 0xFC4E list 2). Each list keeps its terminating
/// sentinel record (a trigger the camera never reaches), as the original does.
#[derive(Clone, Debug, Default)]
pub struct AreaScript {
    pub list1_addr: u32,
    pub list2_addr: u32,
    pub list1: Vec<SpawnRec>,
    pub list2: Vec<SpawnRec>,
}

/// One group of a wave script: wait until the alive count is at most `thr[players-1]`, then
/// spawn the records (each after its delay). The 16-byte terminator is not stored.
#[derive(Clone, Debug, Default)]
pub struct WaveGroup {
    pub thr: [u16; 3],
    pub spawns: Vec<SpawnRec>,
}

/// A wave script (table 0x7B37C): groups, then the final threshold and the end word
/// (negative; `end & 0xE` = 0 GO, 2 walk to the next area, 4 stage exit).
#[derive(Clone, Debug, Default)]
pub struct Wave {
    pub addr: u32,
    pub groups: Vec<WaveGroup>,
    pub final_thr: [u16; 3],
    pub end: u16,
}

#[derive(Clone, Debug, Default)]
pub struct StageScript {
    /// Areas 0..3 (unused areas repeat area 0).
    pub areas: Vec<AreaScript>,
    pub waves: Vec<Wave>,
    /// Boss reinforcement controller (pool B class 22): timed list (0x7BF36) and threshold
    /// groups (0x7BF56). Same record format as waves.
    pub boss_timed: Vec<SpawnRec>,
    pub boss_groups: Wave,
}

/// Byte blobs copied from the ROM, named by start address.
#[derive(Clone, Debug, Default)]
pub struct ByteTables {
    pub blobs: Vec<(u32, Vec<u8>)>,
}

impl ByteTables {
    fn find(&self, addr: u32) -> (&[u8], usize) {
        for (start, data) in &self.blobs {
            if addr >= *start && ((addr - start) as usize) < data.len() {
                return (data, (addr - start) as usize);
            }
        }
        panic!("ROM table byte {addr:#x} was not extracted (add it to dino-extract/src/enemy.rs TABLES)")
    }
    /// Byte at `table + off` (off is a byte offset, as the 68000 index register).
    pub fn u8(&self, table: u32, off: u32) -> u8 {
        let (d, i) = self.find(table.wrapping_add(off));
        d[i]
    }
    pub fn i8(&self, table: u32, off: u32) -> i8 {
        self.u8(table, off) as i8
    }
    /// Big-endian word at `table + off`.
    pub fn u16(&self, table: u32, off: u32) -> u16 {
        let a = table.wrapping_add(off);
        u16::from_be_bytes([self.u8(a, 0), self.u8(a, 1)])
    }
    pub fn i16(&self, table: u32, off: u32) -> i16 {
        self.u16(table, off) as i16
    }
    pub fn u32(&self, table: u32, off: u32) -> u32 {
        let a = table.wrapping_add(off);
        ((self.u16(a, 0) as u32) << 16) | self.u16(a, 2) as u32
    }
}

#[derive(Clone, Debug, Default)]
pub struct EnemyData {
    /// Stage scripts by stage index (G.$4D9).
    pub stages: Vec<StageScript>,
    /// 0x31454: HP by name id and level (0x31424).
    pub hp: Vec<[u16; 32]>,
    /// 0x31E52: boss HP by boss index, players-1 and level / 4 (0x31E14).
    pub boss_hp: Vec<[[u16; 8]; 3]>,
    /// 0xA2EA4: enemy attack records by attack id (obj+$58).
    pub attacks: Vec<AttackRecord>,
    /// Speed tables (0x17A4): `speed_ptrs[id]` = table address; `speeds[addr][dir]` = (vx, vz).
    pub speed_ptrs: Vec<u32>,
    pub speeds: HashMap<u32, [(i16, i16); 32]>,
    /// Backward animation step (0x1380): frame address -> (previous frame address, carry).
    pub reverse_frames: HashMap<u32, (u32, bool)>,
    /// Small tables by ROM address.
    pub t: ByteTables,
}

impl EnemyData {
    /// Speed vector of table `table` for direction `dir` (0..31).
    pub fn speed(&self, table: u32, dir: u8) -> (i16, i16) {
        self.speeds.get(&table).unwrap_or_else(|| panic!("speed table {table:#x} was not extracted"))[(dir & 31) as usize]
    }
}
