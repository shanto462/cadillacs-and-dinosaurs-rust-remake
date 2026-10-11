//! Extracts stage maps, palettes, camera events and per-area constants.
//! Formats: `re/specs/stage/SPEC.md` sections 2 and 4.

use crate::rom::Rom;
use dino_assets::stage::*;

/// Areas per stage (stage table in SPEC 4.6).
const AREAS: [u8; 8] = [3, 3, 1, 3, 2, 3, 4, 3];

/// Scroll2 screen tables per stage, plus the table that follows the last one (end marker).
const SCREEN_TABLES: [u32; 9] = [0x138FC8, 0x139AC8, 0x13BAC8, 0x13BDC8, 0x13DDC8, 0x13FDC8, 0x141DC8, 0x143DC8, 0x145DC8];
/// Scroll2 block tables per stage, plus the end marker.
const BLOCK_TABLES: [u32; 9] = [0x162068, 0x164198, 0x166758, 0x166C98, 0x1683E8, 0x16B3D8, 0x16E188, 0x172158, 0x175378];
const SCROLL3_SCREENS: u32 = 0x177288;
const SCROLL3_SCREEN_COUNT: u32 = 128;

fn area_map(rom: &Rom, hdr: u32) -> AreaMap {
    let (w, h) = (rom.u8(hdr), rom.u8(hdr + 1));
    let ptr = rom.u32(hdr + 2);
    let screens = if w == 0 { Vec::new() } else { (0..w as u32 * h as u32).map(|i| rom.u16(ptr + 2 * i)).collect() };
    AreaMap { rom_addr: hdr, w, h, screens, init_x: rom.u16(hdr + 6), init_y: rom.u16(hdr + 8) }
}

fn palette_bank(rom: &Rom, addr: u32) -> Vec<u16> {
    (0..512).map(|i| rom.u16(addr + 2 * i)).collect()
}

fn cell(rom: &Rom, a: u32) -> Cell {
    Cell { code: rom.u16(a), attr: rom.u16(a + 2) }
}

pub fn extract(rom: &Rom) -> StageData {
    let mut data = StageData {
        scroll3_screen_cells: (0..SCROLL3_SCREEN_COUNT * 64).map(|i| cell(rom, SCROLL3_SCREENS + 4 * i)).collect(),
        scroll1_palettes: palette_bank(rom, 0xBBA52),
        ..Default::default()
    };
    for stage in 0..8u32 {
        let screens = (SCREEN_TABLES[stage as usize]..SCREEN_TABLES[stage as usize + 1]).step_by(2).map(|a| rom.u16(a)).collect();
        let blocks = (BLOCK_TABLES[stage as usize]..BLOCK_TABLES[stage as usize + 1])
            .step_by(16)
            .map(|b| [cell(rom, b), cell(rom, b + 4), cell(rom, b + 8), cell(rom, b + 12)])
            .collect();
        // Camera events (0x2099A): stage 2 uses its own table at 0x22B9E.
        let mut ev = if stage == 2 { 0x22B9E } else { 0x21B2A + rom.u16(0x21B2A + stage * 2) as u32 };
        let mut camera_events = Vec::new();
        loop {
            let x = rom.u16(ev);
            camera_events.push(CameraEvent {
                x,
                y: rom.u16(ev + 2),
                push: rom.u16(ev + 4),
                up: rom.u16(ev + 6),
                down: rom.u16(ev + 8),
                area: rom.u8(ev + 10),
                mode: rom.u8(ev + 11),
                vfollow: rom.u8(ev + 12),
                flag70: rom.u8(ev + 13),
                next_mode: rom.u8(ev + 14),
                by_y: rom.u8(ev + 15),
            });
            if x == 0xFFFF {
                break;
            }
            ev += 16;
        }
        let areas = (0..AREAS[stage as usize] as u32)
            .map(|area| {
                let shift_tab = 0x22DE4 + rom.u16(0x22DE4 + stage * 2) as u32;
                let prio = if stage == 6 { 0x8DCC + area * 8 } else { 0x8DEC + stage * 8 };
                Area {
                    scroll2: area_map(rom, rom.u32(0x21580 + stage * 4) + area * 10),
                    scroll3: area_map(rom, rom.u32(0x2311E + stage * 4) + area * 10),
                    scroll2_palettes: palette_bank(rom, rom.u32(0x8AE0 + stage * 16 + area * 4)),
                    scroll3_palettes: palette_bank(rom, rom.u32(0x8C52 + stage * 16 + area * 4)),
                    scroll3_shift: rom.u8(shift_tab + area),
                    scroll3_yfollow: rom.u8(0x22E54 + stage * 4 + area) != 0,
                    slope_shift: rom.u8(0x20F58 + stage * 4 + area),
                    priority_masks: [rom.u16(prio), rom.u16(prio + 2), rom.u16(prio + 4), rom.u16(prio + 6)],
                }
            })
            .collect();
        data.stages.push(Stage { scroll2_screen_words: screens, scroll2_blocks: blocks, camera_events, areas });
    }
    data
}
