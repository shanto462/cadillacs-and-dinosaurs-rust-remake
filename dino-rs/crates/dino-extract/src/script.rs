//! Extract scripted control tables without executing the original actions.

use crate::rom::Rom;
use dino_assets::script::{Script, ScriptData, TABLES};

/// 0x1E6FC script pointers and the data tables used by 0x1D95E..0x1E650.
pub fn extract(rom: &Rom) -> ScriptData {
    let mut data = ScriptData::default();
    data.tables.blobs = TABLES
        .iter()
        .map(|&(addr, len, _)| (addr, (addr..addr + len).map(|a| rom.u8(a)).collect()))
        .collect();
    data.pointers = std::array::from_fn(|i| rom.u32(0x1E6FC + 4 * i as u32));
    data.scripts = std::array::from_fn(|i| {
        let start = data.pointers[i];
        let mut script = Script {
            rom_addr: start,
            records: Vec::new(),
        };
        let mut addr = start;
        loop {
            assert!(
                addr >= 0x1E6FC + 27 * 4 && addr + 6 <= 0x1EE6A,
                "script {i} has no END inside the script region"
            );
            let record = data.record_at(addr);
            script.records.push(record);
            if matches!(record.action, 0x0A | 0x26) {
                break;
            }
            addr += 6;
        }
        script
    });
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn rom_scripts_and_table_lengths() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dino.zip");
        if !path.exists() {
            eprintln!("script extraction test skipped: user's dino.zip is absent");
            return;
        }
        let rom = Rom::load(&path).unwrap();
        let data = extract(&rom);
        assert_eq!(data.pointers.len(), 27);
        assert_eq!(data.scripts.len(), 26);
        assert_eq!(data.pointers[26], data.pointers[0]);
        assert!(std::ptr::eq(data.script(26), data.script(0)));
        // D4c 5.5 describes 26 END-terminated scripts, with script 25 being only END.
        // Derive the counts from ROM pointer spans, never a copied count table.
        for (id, script) in data.scripts.iter().enumerate() {
            assert_eq!(script.rom_addr, rom.u32(0x1E6FC + 4 * id as u32));
            assert!(!script.records.is_empty());
            assert!(matches!(script.records.last().unwrap().action, 0x0A | 0x26));
            if id < 25 {
                let span = data.pointers[id + 1] - data.pointers[id];
                assert_eq!(span % 6, 0, "script {id} alignment");
                assert_eq!(script.records.len(), span as usize / 6, "script {id} count");
            } else {
                assert_eq!(script.records.len(), 1);
            }
            for (index, record) in script.records.iter().enumerate() {
                let addr = script.rom_addr + 6 * index as u32;
                assert_eq!(record.action, rom.u8(addr));
                assert_eq!(record.floor_only, rom.u8(addr + 1));
                assert_eq!(record.argument, rom.u8(addr + 2));
                assert_eq!(record.control, rom.u8(addr + 3));
                assert_eq!(record.timer, rom.u16(addr + 4));
            }
        }
        for ((addr, bytes), &(expected_addr, len, _)) in data.tables.blobs.iter().zip(TABLES) {
            assert_eq!(*addr, expected_addr);
            assert_eq!(bytes.len(), len as usize);
            assert_eq!(
                bytes.as_slice(),
                &rom.program[*addr as usize..(*addr + len) as usize]
            );
        }
        // Pointer-indexed rows fit their documented stage/area strides.
        for stage in 0..8 {
            for (table, stride) in [(0x1D9B0, 8), (0x1DA64, 8), (0x1EEEA, 64)] {
                let ptr = data.tables.u32(table, 4 * stage);
                let (_, len, _) = TABLES.iter().find(|&&(a, _, _)| a == table).unwrap();
                assert!(ptr >= table + 32 && ptr + stride <= table + len);
            }
        }
    }
}
