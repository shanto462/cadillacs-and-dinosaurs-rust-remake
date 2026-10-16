//! Text, HUD and game-flow tables: strings, tile code tables, palettes and layout tables that the
//! HUD (0x16BEA and friends) and the flow code read. Owner: flow / HUD port.
//! See `re/specs/flow/SPEC.md`.
//!
//! The tables are kept as byte blobs named by their ROM address, so the ported code can index them
//! exactly as the 68000 code does (`t.u16(0x163D4, 2 * chr)`), and strings are read with the same
//! offset tables the original uses (`string_at`, `table_string`).

/// Byte blobs copied from the ROM, named by start address.
#[derive(Clone, Debug, Default)]
pub struct TextData {
    pub blobs: Vec<(u32, Vec<u8>)>,
}

impl TextData {
    fn find(&self, addr: u32) -> (&[u8], usize) {
        for (start, data) in &self.blobs {
            if addr >= *start && ((addr - start) as usize) < data.len() {
                return (data, (addr - start) as usize);
            }
        }
        panic!("ROM text table byte {addr:#x} was not extracted (add it to dino-extract/src/text.rs TABLES)")
    }

    /// Byte at `table + off` (byte offset, as the 68000 index register).
    pub fn u8(&self, table: u32, off: u32) -> u8 {
        let (d, i) = self.find(table.wrapping_add(off));
        d[i]
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

    /// The rest of the blob that holds `addr`, from `addr` on (for the text printers, which read
    /// a string through the end marker they know: '@', 0x4040 ...).
    pub fn slice_from(&self, addr: u32) -> &[u8] {
        let (d, i) = self.find(addr);
        &d[i..]
    }

    /// Bytes from `addr` up to and including the '@' (0x40) terminator.
    pub fn string_at(&self, addr: u32) -> Vec<u8> {
        let mut v = Vec::new();
        let mut a = addr;
        loop {
            let c = self.u8(a, 0);
            v.push(c);
            if c == b'@' || v.len() > 256 {
                return v;
            }
            a += 1;
        }
    }

    /// Entry `index` of a table of word offsets relative to the table start
    /// (`move.w table(pc,d0.w),d0; lea table(pc,d0.w),a0`): the string bytes up to '@'.
    pub fn table_string(&self, table: u32, index: u32) -> Vec<u8> {
        let off = self.i16(table, 2 * index) as i32;
        self.string_at((table as i32 + off) as u32)
    }
}

/// Table regions read by the HUD and flow code: (ROM address, length in bytes, what it is).
/// The extractor copies exactly these; the game panics if it reads outside them.
pub const TABLES: &[(u32, u32, &str)] = &[
    (0x4E56, 0x178, "player start positions by stage, area and player (0x4E0A)"),
    (0x6BFE, 0x2C, "extra life score thresholds by DIP (0x6B98)"),
    (0x10292, 0x90, "BCD score values (0x10036), indexed by byte offset"),
    (0x150A4, 0xC8, "default high score initials, 4 bytes x 50 (0x1504E)"),
    (0x159F8, 0x08, "rank suffixes ST ND RD TH (0x15974)"),
    (0x15ACE, 0x14, "rank text VRAM addresses, 3 players then 2 players (0x15AB0)"),
    (0x1632A, 0x0C, "player block VRAM base, 3-player layout (0x1631A)"),
    (0x1638E, 0x08, "player block VRAM base, 2-player layout (0x1637E)"),
    (0x163D4, 0x10, "face first tile and attribute by character (0x16396)"),
    (0x1641E, 0x28, "player HUD names by character (0x163F4)"),
    (0x16520, 0x28, "life bar partial tiles: middle, left end, right end (0x16446)"),
    (0x1666A, 0x14, "HUD block clear rectangles, 3 then 2 players (0x1664E)"),
    (0x1677C, 0x118, "low vitality blink attribute tables (0x16704)"),
    (0x17426, 0x20, "enemy portrait blink tiles (0x173BC)"),
    (0x175EE, 0x2E, "enemy bar partial tiles, 3-player layout (0x174F2)"),
    (0x177B2, 0x44, "enemy bar partial tiles, 2-player layout (0x17676)"),
    (0x17828, 0x0C, "enemy bar attribute and palette offset by player, 3 players (0x17834)"),
    (0x17892, 0x4C, "enemy bar colours by extra bars, 3 players; then the 2-player attribute table"),
    (0x1793C, 0x50, "enemy bar colours by extra bars, 2 players (0x178DE)"),
    (0x17A1E, 0x310, "portrait palettes and 3x2 portrait tiles by name id (0x179B6)"),
    (0x17D70, 0x220, "player and enemy name strings (0x17D2E)"),
    (0x96AA, 0x24, "hit sound code pairs by attack record sound byte (0x962A)"),
    (0x98C6, 0x06, "player voice codes by +$118, ids 0/1 then 2 (0x986C)"),
    (0x9926, 0x06, "player voice codes by +$118, ids 0/1 then 2 (0x98CC)"),
    (0x9974, 0x10, "player voice codes by character and +$118 (0x992C)"),
    (0x99DE, 0x09, "player voice codes by id (0, 1, 2) and +$118 (0x9984)"),
    (0x9A26, 0x10, "player voice codes by character and +$118 (0x99E8)"),
    (0x9A74, 0x10, "player voice codes by character and +$118 (0x9A36)"),
    (0x9B5C, 0x03, "player voice codes by +$118 (0x9AF4)"),
    (0x9C72, 0x03, "player voice codes by +$118 (0x9C0C)"),
    (0xA53A, 0x20, "voice bank +$118 of players 1..3 by player mix (0xA502)"),
    (0xA582, 0x20, "area music id by stage and area (0xA55A)"),
    (0xA5C8, 0x0A, "alternate music id by stage (0xA55A)"),
    (0x23AC6, 0x60, "HUD gun icon tiles (0x23A90)"),
    (0x23B52, 0x0C, "HUD ARM icon tiles (0x23B26)"),
    (0x23BD0, 0x0C, "HUD gun red ammo thresholds (0x23B7A)"),
    (0x23BDC, 0x50, "HUD ammo BCD lookup table (0x23B92)"),
    (0x23C58, 0x54, "HUD gun name string table (0x23C44)"),
    (0x23CDE, 0xFE, "HUD other weapon name string table (0x23CCA)"),
    (0x23EB4, 0x0C, "HUD cartridge bar step table (0x23E92)"),
    (0x24016, 0x0C, "food icon tiles (0x23FEA)"),
    (0x240A0, 0x23C, "food strings: GOOD, names, points (0x24026)"),
    (0x243FE, 0x0C, "treasure icon tiles (0x243D0)"),
    (0x24436, 0x1D0, "treasure strings (0x2440E)"),
];
