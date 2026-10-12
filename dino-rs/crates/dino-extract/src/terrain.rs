//! Extracts the terrain collision tables (`dino-assets/src/terrain.rs`).

use crate::rom::Rom;
use dino_assets::terrain::{FloorList, FloorSegment, TerrainData};

/// 0x12C30: floor segment lists per stage and area. A list ends at the first segment that
/// reaches X 0x7FFF (the lists are followed by flat dummy segments that start at X 0x7FFF, so the
/// pointer advance 0x12C90 never leaves them).
fn floor_lists(rom: &Rom) -> Vec<Vec<FloorList>> {
    (0..8u32)
        .map(|stage| {
            let t = 0x12C30 + rom.u16(0x12C30 + 2 * stage) as u32;
            (0..4u32)
                .map(|area| {
                    let a = t.wrapping_add(rom.u16(t + 2 * area) as u32);
                    if a & 1 != 0 || !(0x13D00..0x14000).contains(&a) {
                        // Offsets of areas that do not exist point at garbage.
                        return FloorList::default();
                    }
                    let mut segs = Vec::new();
                    for k in 0..64 {
                        let e = a + 12 * k;
                        let s = FloorSegment {
                            slope: rom.i16(e),
                            x0: rom.i16(e + 2),
                            x1: rom.i16(e + 4),
                            base: rom.i16(e + 6),
                            height: rom.i16(e + 8),
                            wading: rom.u8(e + 10),
                        };
                        segs.push(s);
                        if s.x1 == 0x7FFF {
                            break;
                        }
                    }
                    FloorList { addr: a, segs }
                })
                .collect()
        })
        .collect()
}

/// 0x12C50 (called by 0x12BE4 in the vertical scene): builds the collision map at 0x91C000 and
/// 0x91C400 from the screens 0xC and 0xD of the table 0x141DC8 and the 2x2 cell blocks of
/// 0x16E188. Each block entry (16 bytes) fills two cells of column c and two of column c + 1.
fn scene_map(rom: &Rom) -> Vec<[u16; 2]> {
    let mut cells = vec![[0u16; 2]; 512];
    for (base, screen) in [(0usize, 0xCu32), (256, 0xD)] {
        let src = 0x141DC8 + (screen << 7);
        for outer in 0..8u32 {
            for inner in 0..8u32 {
                let idx = rom.u16(src + 2 * (outer * 8 + inner));
                // lsl.w #4 keeps 16 bits and `lea (a3,d0.w)` sign-extends them.
                let off = ((idx << 4) as i16) as i32;
                let t = 0x16E188u32.wrapping_add(off as u32);
                let cell = |k: u32| [rom.u16(t + 4 * k), rom.u16(t + 4 * k + 2)];
                let c = base + (outer * 32 + inner * 2) as usize;
                cells[c] = cell(0);
                cells[c + 1] = cell(1);
                cells[c + 16] = cell(2);
                cells[c + 17] = cell(3);
            }
        }
    }
    cells
}

pub fn extract(rom: &Rom) -> TerrainData {
    TerrainData {
        floor: floor_lists(rom),
        floor_base: std::array::from_fn(|i| rom.i16(0x12C40 + 2 * i as u32)),
        shapes: (0..64u32).map(|c| std::array::from_fn(|k| rom.u16(0x1370C + 16 * c + 2 * k as u32))).collect(),
        flags: (0..8u32)
            .map(|stage| {
                (0..4u32)
                    .map(|area| {
                        let t = 0x13B0C + rom.u16(0x13B0C + 8 * stage + 2 * area) as u32;
                        std::array::from_fn(|c| rom.u8(t + c as u32))
                    })
                    .collect()
            })
            .collect(),
        head: std::array::from_fn(|i| rom.i16(0x136A6 + 2 * i as u32)),
        drop_x: std::array::from_fn(|i| rom.i16(0x638C + 2 * i as u32)),
        drop_z: std::array::from_fn(|i| rom.i16(0x63A4 + 2 * i as u32)),
        scene_map: scene_map(rom),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// Needs the user's ROM set at `../dino.zip` (skipped otherwise).
    #[test]
    fn tables_match_the_rom() {
        let path = Path::new("../../../dino.zip");
        if !path.exists() {
            return;
        }
        let rom = Rom::load(path).unwrap();
        let t = extract(&rom);
        assert_eq!(t.shapes.len(), 64);
        assert_eq!(t.head, [0x50; 4]);
        // Class 0x15 is the ceiling shape (word 0 = 0x20), class 0x3F is a no-op.
        assert_eq!(t.shapes[0x15][0], 0x20);
        assert_eq!(t.shapes[0x3F][0], 0x14);
        assert_eq!(t.floor_base[1], 0x40);
        // Stage 1 (internal 0) area 0 has one flat segment.
        assert_eq!(t.floor[0][0].segs.len(), 1);
        assert_eq!(t.floor[0][0].segs[0].x1, 0x7FFF);
        assert_eq!(t.scene_map.len(), 512);
        assert_eq!(t.drop_x[0], 0x60);
        assert_eq!(t.drop_z, [0x20, 0x30, 0x40]);
    }
}
