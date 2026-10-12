//! Extracts the player movement and combat tables (`re/specs/player/SPEC.md`).

use crate::rom::Rom;
use dino_assets::player::{JumpVector, PlayerData};
use dino_assets::AttackRecord;

/// 0xA26FC per-character attack table lengths (entries of 8 bytes): the tables are packed one
/// after the other (0xA270C, 0xA28EC, 0xA2ADC, 0xA2CBC), so each length is the gap to the next.
const ATTACK_COUNTS: [u32; 4] = [60, 62, 60, 61];
/// 0xA2EA4 enemy attack records, up to the next table (0xA320C).
const ENEMY_ATTACKS: (u32, u32) = (0xA2EA4, 0xA320C);

fn attack(rom: &Rom, a: u32) -> AttackRecord {
    AttackRecord {
        damage: rom.u8(a) as i8,
        stun: rom.u8(a + 1),
        reaction: rom.u8(a + 2),
        hit_stop: rom.u8(a + 3),
        sound: rom.u8(a + 4),
        score: rom.u8(a + 5),
        spark: rom.u8(a + 6),
        b7: rom.u8(a + 7),
    }
}

fn bytes<const N: usize>(rom: &Rom, a: u32) -> [u8; N] {
    std::array::from_fn(|i| rom.u8(a + i as u32))
}

fn words<const N: usize>(rom: &Rom, a: u32) -> [u16; N] {
    std::array::from_fn(|i| rom.u16(a + 2 * i as u32))
}

fn swords<const N: usize>(rom: &Rom, a: u32) -> [i16; N] {
    std::array::from_fn(|i| rom.i16(a + 2 * i as u32))
}

fn speed_table(rom: &Rom, p: u32) -> [[i16; 2]; 8] {
    std::array::from_fn(|d| [rom.i16(p + 4 * d as u32), rom.i16(p + 4 * d as u32 + 2)])
}

pub fn extract(rom: &Rom) -> PlayerData {
    let mut d = PlayerData {
        air_control: std::array::from_fn(|i| [rom.i16(0x1BE66 + 4 * i as u32), rom.i16(0x1BE68 + 4 * i as u32)]),
        dir8: bytes(rom, 0x19292),
        walk_dir: std::array::from_fn(|n| [rom.u8(0x1BDCC + 2 * n as u32), rom.u8(0x1BDCD + 2 * n as u32)]),
        jump_dir: bytes(rom, 0x1A6E8),
        invuln: bytes(rom, 0x6C3A),
        command_time: [bytes(rom, 0x6B7C), bytes(rom, 0x6B84)],
        power: bytes::<128>(rom, 0x2C9B6).to_vec(),
        defense: words(rom, 0x2C88E),
        enemy_hp_reduce: words::<128>(rom, 0x2C896).to_vec(),
        enemy_hp_scale: words(rom, 0x2C996),
        enemy_level_scale: (0x2C3AE..0x2C88E).map(|a| rom.u8(a)).collect(),
        counter_scale: (0xA320Cu32..0xA32FA).step_by(2).map(|a| rom.u16(a)).collect(),
        weakness: (0xA32FA..0xA3400).map(|a| rom.u8(a)).collect(),
        level_type_scale: (0x2BECE..0x2C3AE).map(|a| rom.u8(a)).collect(),
        hp_hundreds_scale: (0x2BC4Eu32..0x2BECE).step_by(2).map(|a| rom.u16(a)).collect(),
        hp_band_scale: words::<128>(rom, 0x2BB4E).to_vec(),
        pushback: std::array::from_fn(|i| swords(rom, 0x1D27C + 6 * i as u32)),
        knockdown: std::array::from_fn(|i| swords(rom, 0x1D28E + 8 * i as u32)),
        lie_time: bytes::<64>(rom, 0x1D1C4).to_vec(),
        lie_mash: bytes::<64>(rom, 0x1D204).to_vec(),
        dizzy_stars: bytes::<64>(rom, 0x1CF5C).to_vec(),
        dizzy_time: words::<64>(rom, 0x1CEDC).to_vec(),
        dizzy_mask: words(rom, 0x1D166),
        mash_weight: bytes(rom, 0x605E),
        throw_offset: std::array::from_fn(|i| swords(rom, 0x2CB32 + 4 * i as u32)),
        knee_spark: std::array::from_fn(|i| swords(rom, 0x1D93E + 4 * i as u32)),
        jump_attack_height: std::array::from_fn(|i| swords(rom, 0x1A734 + 4 * i as u32)),
        dive: std::array::from_fn(|i| swords(rom, 0x1A994 + 4 * i as u32)),
        vectors_1bf06: swords(rom, 0x1BF06),
        recoil: swords(rom, 0x19CD8),
        heavy_fall_anim: bytes(rom, 0x1A902),
        heavy_jump_anim: bytes(rom, 0x1B0C6),
        heavy_land_anim: bytes(rom, 0x1B0F2),
        burn_flames: [swords(rom, 0x1C8A6), swords(rom, 0x1C8AC)],
        score_bcd: (0..64u32).map(|k| rom.u32(0x10292 + 4 * k)).collect(),
        held_offsets: (dino_assets::player::HELD_OFFSETS_BASE..0x2D0E0).map(|a| rom.u8(a)).collect(),
        ..Default::default()
    };
    for k in 0..16u32 {
        d.speed_ptr[k as usize] = rom.u32(0x5064 + 4 * k);
        d.speed_heavy_ptr[k as usize] = rom.u32(0x50A4 + 4 * k);
        d.speed[k as usize] = speed_table(rom, d.speed_ptr[k as usize]);
        d.speed_heavy[k as usize] = speed_table(rom, d.speed_heavy_ptr[k as usize]);
        let t = rom.u32(0x53A2 + 4 * k);
        d.jump[k as usize] = std::array::from_fn(|i| {
            let [vx, ax, vy, ay, vz, az] = swords(rom, t + 12 * i as u32);
            JumpVector { vx, ax, vy, ay, vz, az }
        });
    }
    for (ch, &n) in ATTACK_COUNTS.iter().enumerate() {
        let base = rom.u32(0xA26FC + 4 * ch as u32);
        d.attacks[ch] = (0..n).map(|k| attack(rom, base + 8 * k)).collect();
    }
    d.enemy_attacks = (ENEMY_ATTACKS.0..ENEMY_ATTACKS.1).step_by(8).map(|a| attack(rom, a)).collect();
    // 0x4E56: word offsets per stage, then 3 areas x 3 players x (x, y, z).
    d.start_pos = (0..8u32)
        .map(|stage| {
            let t = 0x4E56 + rom.u16(0x4E56 + 2 * stage) as u32;
            std::array::from_fn(|area| {
                std::array::from_fn(|p| swords(rom, t + 18 * area as u32 + 6 * p as u32))
            })
        })
        .collect();
    d
}

#[cfg(test)]
mod tests {
    #[test]
    fn attack_tables_are_packed() {
        // Bases 0xA270C, 0xA28EC, 0xA2ADC, 0xA2CBC and the enemy table at 0xA2EA4.
        let bases = [0xA270Cu32, 0xA28EC, 0xA2ADC, 0xA2CBC, 0xA2EA4];
        for (k, n) in super::ATTACK_COUNTS.iter().enumerate() {
            assert_eq!(bases[k] + 8 * n, bases[k + 1]);
        }
    }
}
