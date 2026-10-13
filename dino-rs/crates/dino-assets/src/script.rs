//! Scripted player control data, read from the user's ROM by `dino-extract`.

use crate::text::TextData;

/// One six-byte record consumed by 0x1DB5E. Flags stay as bytes, not booleans.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScriptRecord {
    /// Action byte, the byte offset into the dispatch table 0x1DBD6.
    pub action: u8,
    /// G.$4F0: floor-only terrain flag.
    pub floor_only: u8,
    /// G.$4FD: caption or sound argument.
    pub argument: u8,
    /// G.$506: screen clamp, pickup and weapon control flag.
    pub control: u8,
    /// Big-endian timer word. 0xFFFF remains an unsigned word.
    pub timer: u16,
}

/// Records through the first END (0x0A or 0x26), including that END.
#[derive(Clone, Debug, Default)]
pub struct Script {
    pub rom_addr: u32,
    pub records: Vec<ScriptRecord>,
}

#[derive(Clone, Debug, Default)]
pub struct ScriptData {
    /// 0x1E6FC: all 27 pointers, including entry 26's alias of script 0.
    pub pointers: [u32; 27],
    /// The 26 active scripts. Addresses identify their six-byte records.
    pub scripts: [Script; 26],
    /// Exact table regions, including padding, unused rows and trailing script data.
    /// Use `tables.u8/u16/i16/u32(table, byte_offset)` for original indexing.
    pub tables: TextData,
}

impl ScriptData {
    /// Resolve a script id as 0x1DB0C does, including the final alias.
    pub fn script(&self, id: u8) -> &Script {
        let addr = self.pointers[id as usize];
        self.scripts
            .iter()
            .find(|s| s.rom_addr == addr)
            .expect("script pointer not extracted")
    }

    /// Decode a record at an original script address (0x1DB5E).
    pub fn record_at(&self, addr: u32) -> ScriptRecord {
        ScriptRecord {
            action: self.tables.u8(addr, 0),
            floor_only: self.tables.u8(addr, 1),
            argument: self.tables.u8(addr, 2),
            control: self.tables.u8(addr, 3),
            timer: self.tables.u16(addr, 4),
        }
    }
}

/// (Address, byte length, layout). Lengths describe storage, including unused entries.
pub const TABLES: &[(u32, u32, &str)] = &[
    (
        0x1D9B0,
        0x60,
        "end ids: 8 long pointers, 8 stages x 4 areas x (id, G.$514)",
    ),
    (
        0x1DA64,
        0x60,
        "start ids: 8 long pointers, 8 stages x 4 areas x 2 bytes",
    ),
    (0x1DAFC, 0x10, "boss area ids: 8 stages x (id, G.$514)"),
    (0x1DD88, 0x40, "jump offsets: 8 stages x 4 signed words"),
    (
        0x1E3F4,
        0x80,
        "single points: 8 stages x 4 areas x (X, Z) signed words",
    ),
    (
        0x1E532,
        0x18,
        "fixed spots: 2 banks x 3 (X, Z) pairs; second bank for stage 7",
    ),
    (
        0x1E6FC,
        0x1EE6A - 0x1E6FC,
        "27 long pointers, then six-byte scripts and unused trailing records",
    ),
    (
        0x1EE6A,
        0x80,
        "end targets: 8 stages x 4 (X, Z) pairs, retaining the cabinet offset row",
    ),
    (
        0x1EEEA,
        0x220,
        "start targets: 8 long pointers, 8 stages x 4 areas x 4 (X, Z) pairs",
    ),
    (
        0x1F10A,
        0x80,
        "lineup spots: 8 stages x 4 (X, Z) pairs, including unused entries",
    ),
];
