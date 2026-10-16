//! QSound tables, sequences and samples (SPEC sections 1, 5, 12, 13, 14).
//!
//! `cd_q.5k` is Kabuki-encrypted in 0x0000..0x7FFF. This module ports MAME `kabuki_decode`
//! (keys in SPEC section 1) and copies the decrypted data image into [`dino_assets::sound::SoundData`].
//! Sequence bytes stay in that image and are addressed by [`dino_assets::sound::SoundCommand::track`].
//! The sample ROM is `cd-q1.1k` .. `cd-q4.4k` in MAME load order. Nothing is stored in the source.

use crate::rom::Rom;
use dino_assets::sound::{
    Instrument, LevelTables, MultiplyTables, PitchTables, Sample, SoundCommand, SoundData,
    SoundKind, SoundTables,
};

/// Kabuki keys for this set (SPEC section 1).
const SWAP1: u32 = 0x7654_3210;
const SWAP2: u32 = 0x2460_1357;
const ADDR_KEY: u32 = 0x4343;
const XOR_KEY: u8 = 0x43;

/// Only the low 32 KB of `cd_q.5k` is encrypted. The rest is plain music and effect data.
const ENCRYPTED: usize = 0x8000;

/// Command count, big-endian word (0x017D reads N, then `code = code mod N`).
const COMMAND_COUNT: usize = 0x8000;
/// Music master volume index. Bytes 0x8003..0x8005 are unused.
const MASTER_VOLUME: usize = 0x8002;
/// First command-table entry: bank, hi, lo, unused.
const COMMAND_ENTRIES: usize = 0x8006;

/// Sample table. 8 bytes each, up to the instrument table.
const SAMPLE_TABLE: usize = 0x6A82;
/// Instrument table. 256 entries of 8 bytes (0..129 real, 130..255 filler).
const INSTR_TABLE: usize = 0x6E62;
const INSTR_LEN: usize = 256;

/// Kabuki decrypt of `cd_q.5k` (MAME `kabuki_decode`, SPEC section 1).
///
/// Opcode select = `addr + 0x4343`. Data select = `(addr ^ 0x1FC0) + 0x4343 + 1`.
/// Opcode fetches (the first byte, and the second byte after CB/DD/ED/FD) use the opcode
/// image. Operands and every other read use the data image. Returns `(opcode, data)`.
fn kabuki_decrypt(raw: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut op = raw.to_vec();
    let mut data = raw.to_vec();
    let n = raw.len().min(ENCRYPTED);
    for addr in 0..n {
        let a = addr as u32;
        op[addr] = byte_decode(raw[addr], (a + ADDR_KEY) & 0xFFFF);
        data[addr] = byte_decode(raw[addr], ((a ^ 0x1FC0) + ADDR_KEY + 1) & 0xFFFF);
    }
    (op, data)
}

/// Swap bits `2 * pair` and `2 * pair + 1` of one byte.
fn swap_pair(src: u8, pair: u32) -> u8 {
    let lo = 1u8 << (2 * pair);
    let hi = lo << 1;
    let keep = src & !(lo | hi);
    keep | ((src & lo) << 1) | ((src & hi) >> 1)
}

fn bitswap1(mut src: u8, key: u32, select: u32) -> u8 {
    for (pair, shift) in [(0u32, 0u32), (1, 4), (2, 8), (3, 12)] {
        if select & (1 << ((key >> shift) & 7)) != 0 {
            src = swap_pair(src, pair);
        }
    }
    src
}

fn bitswap2(mut src: u8, key: u32, select: u32) -> u8 {
    for (pair, shift) in [(0u32, 12u32), (1, 8), (2, 4), (3, 0)] {
        if select & (1 << ((key >> shift) & 7)) != 0 {
            src = swap_pair(src, pair);
        }
    }
    src
}

fn byte_decode(mut src: u8, select: u32) -> u8 {
    src = bitswap1(src, SWAP1 & 0xFFFF, select & 0xFF);
    src = src.rotate_left(1);
    src = bitswap2(src, SWAP1 >> 16, select & 0xFF);
    src ^= XOR_KEY;
    src = src.rotate_left(1);
    src = bitswap2(src, SWAP2 & 0xFFFF, select >> 8);
    src = src.rotate_left(1);
    bitswap1(src, SWAP2 >> 16, select >> 8)
}

fn le_words<const N: usize>(data: &[u8], addr: usize) -> [u16; N] {
    std::array::from_fn(|i| u16::from_le_bytes([data[addr + 2 * i], data[addr + 2 * i + 1]]))
}

fn le_vec(data: &[u8], addr: usize, n: usize) -> Vec<u16> {
    (0..n)
        .map(|i| u16::from_le_bytes([data[addr + 2 * i], data[addr + 2 * i + 1]]))
        .collect()
}

pub fn extract(rom: &Rom) -> SoundData {
    let (_op, data) = kabuki_decrypt(&rom.audiocpu);
    let n = u16::from_be_bytes([data[COMMAND_COUNT], data[COMMAND_COUNT + 1]]) as usize;
    let mut commands = Vec::with_capacity(n);
    for code in 0..n {
        let e = COMMAND_ENTRIES + 4 * code;
        let header = ((data[e] as u32) << 16) | ((data[e + 1] as u32) << 8) | data[e + 2] as u32;
        if header == 0 {
            commands.push(SoundCommand {
                code: code as u16,
                header: 0,
                kind: None,
                voice_off: [0; 16],
            });
            continue;
        }
        let at = header as usize;
        if at + 33 > data.len() {
            panic!("sound command {code:#x} header {header:#x} runs off the data image");
        }
        let h = data[at];
        let voice_off = std::array::from_fn(|v| {
            let a = at + 1 + 2 * v;
            u16::from_be_bytes([data[a], data[a + 1]])
        });
        // 0x017D: header 0 is music, 1..=0x7F is an effect priority, >= 0x80 is ADPCM.
        let (kind, voice_off) = if h == 0 {
            (SoundKind::Music, voice_off)
        } else if h < 0x80 {
            (SoundKind::Effect(h), voice_off)
        } else {
            (SoundKind::Adpcm(h), [0; 16])
        };
        commands.push(SoundCommand {
            code: code as u16,
            header,
            kind: Some(kind),
            voice_off,
        });
    }

    let n_samples = (INSTR_TABLE - SAMPLE_TABLE) / 8;
    let samples = (0..n_samples)
        .map(|i| {
            let a = SAMPLE_TABLE + 8 * i;
            Sample {
                bank: data[a],
                start: u16::from_le_bytes([data[a + 1], data[a + 2]]),
                loop_start: u16::from_le_bytes([data[a + 3], data[a + 4]]),
                end: u16::from_le_bytes([data[a + 5], data[a + 6]]),
                root_key: data[a + 7],
            }
        })
        .collect();
    let instruments = (0..INSTR_LEN)
        .map(|i| {
            let a = INSTR_TABLE + 8 * i;
            Instrument {
                sample: data[a],
                fine: data[a + 1],
                attack: data[a + 2],
                decay: data[a + 3],
                sustain: data[a + 4],
                slow_decay: data[a + 5],
                release: data[a + 6],
                unused: data[a + 7],
            }
        })
        .collect();

    let mut frac = [[0u8; 256]; 12];
    for (row, slot) in frac.iter_mut().enumerate() {
        let at = 0x152B + row * 256;
        slot.copy_from_slice(&data[at..at + 256]);
    }
    let tables = SoundTables {
        pitch: PitchTables {
            len_triplet: data[0x0C50..0x0C57].try_into().unwrap(),
            len_normal: data[0x0C57..0x0C5E].try_into().unwrap(),
            len_dotted: data[0x0C5E..0x0C65].try_into().unwrap(),
            note_base: data[0x0C65..0x0C75].try_into().unwrap(),
            octave: data[0x13DE..0x144A].try_into().unwrap(),
            octave_first: data[0x144A..0x1453].try_into().unwrap(),
            base: le_words(&data, 0x1453),
            frac,
        },
        level: LevelTables {
            volume: le_words(&data, 0x23A9),
            gain: le_words(&data, 0x24A9),
            attack: le_words(&data, 0x25A9),
            // 65 words: index 0x40 reads the first word of 0x26A9 (SPEC section 12).
            decay: le_words(&data, 0x2629),
            lfo_speed: le_words(&data, 0x26A9),
            vibrato: le_words(&data, 0x27A9),
            tremolo: le_words(&data, 0x28A9),
        },
        multiply: MultiplyTables {
            lo: le_vec(&data, 0x2A81, 0x1000),
            hi: le_vec(&data, 0x4A81, 0x1000),
        },
    };

    let master_volume = data[MASTER_VOLUME];
    SoundData {
        data,
        pcm: rom.qsound.clone(),
        master_volume,
        commands,
        samples,
        instruments,
        tables,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::sound::SoundKind;
    use std::path::PathBuf;

    /// FNV-1a, 32 bit. Used so the reset check compares a number, not a list of ROM bytes.
    fn fnv1a(bytes: &[u8]) -> u32 {
        let mut h = 0x811c_9dc5u32;
        for &b in bytes {
            h ^= b as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
        h
    }

    /// The reset stub at Z80 0x0000, fetched the way the CPU does (SPEC section 1):
    /// opcode bytes from the opcode image, operand bytes from the data image.
    /// Returns the mnemonic text and the bytes that were fetched, in order.
    fn decode_reset(op: &[u8], data: &[u8]) -> (String, Vec<u8>) {
        let mut pc = 0usize;
        let mut text = Vec::new();
        let mut fetched = Vec::new();
        for _ in 0..8 {
            let b = op[pc];
            match b {
                0xF3 => {
                    text.push("di".to_string());
                    fetched.push(b);
                    pc += 1;
                }
                0xED => {
                    let b2 = op[pc + 1];
                    let name = match b2 {
                        0x56 => "im 1",
                        0xB0 => "ldir",
                        _ => return (format!("ed ${b2:02x}"), fetched),
                    };
                    text.push(name.to_string());
                    fetched.push(b);
                    fetched.push(b2);
                    pc += 2;
                }
                0x31 | 0x21 | 0x11 | 0x01 => {
                    let reg = match b {
                        0x31 => "sp",
                        0x21 => "hl",
                        0x11 => "de",
                        _ => "bc",
                    };
                    let nn = u16::from_le_bytes([data[pc + 1], data[pc + 2]]);
                    text.push(format!("ld {reg},${nn:04x}"));
                    fetched.push(b);
                    fetched.extend_from_slice(&data[pc + 1..pc + 3]);
                    pc += 3;
                }
                0x36 => {
                    let n = data[pc + 1];
                    text.push(format!("ld (hl),{n}"));
                    fetched.push(b);
                    fetched.push(n);
                    pc += 2;
                }
                _ => {
                    text.push(format!("db ${b:02x}"));
                    fetched.push(b);
                    break;
                }
            }
        }
        (text.join("; "), fetched)
    }

    fn load() -> Option<(Rom, SoundData, Vec<u8>)> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../dino.zip");
        if !path.exists() {
            return None;
        }
        let rom = Rom::load(&path).unwrap();
        let (op, _) = kabuki_decrypt(&rom.audiocpu);
        let sound = extract(&rom);
        Some((rom, sound, op))
    }

    #[test]
    fn kabuki_bit_steps() {
        // Pair 0 swaps bits 0 and 1. 0b01 becomes 0b10.
        assert_eq!(swap_pair(0b0000_0001, 0), 0b0000_0010);
        assert_eq!(0x81u8.rotate_left(1), 0x03);
    }

    /// Needs `dino.zip` (skipped otherwise). Counts are the asset catalog (N8, section 8).
    /// The reset check is a checksum of the fetched stub, not a copy of its bytes.
    #[test]
    fn catalog_counts_and_reset_checksum() {
        let Some((rom, sound, op)) = load() else {
            return;
        };

        // Plain region is not encrypted. The two images differ only below 0x8000.
        let raw = &rom.audiocpu;
        assert_eq!(&sound.data[ENCRYPTED..], &raw[ENCRYPTED..]);
        assert_eq!(&op[ENCRYPTED..], &raw[ENCRYPTED..]);
        assert_ne!(&op[..ENCRYPTED], &raw[..ENCRYPTED]);
        assert_ne!(&sound.data[..ENCRYPTED], &raw[..ENCRYPTED]);
        assert_ne!(&op[..ENCRYPTED], &sound.data[..ENCRYPTED]);

        // SPEC section 1. Operands come from the data image, so this is not a raw slice compare.
        // FNV-1a of the 19 bytes the stub fetches (M1 from the opcode image, operands from the data image).
        // Only a checksum: the expected program text is not written into the source (ROM content stays in dino.zip).
        let (text, fetched) = decode_reset(&op, &sound.data);
        assert!(!text.contains("db "), "every stub instruction decodes");
        assert_eq!(fetched.len(), 19);
        assert_eq!(fnv1a(&fetched), 0xb0c5_0be9);

        assert_eq!(sound.commands.len(), 0x106);
        assert_eq!(sound.sequences().count(), 243);
        let music: Vec<u16> = sound
            .commands
            .iter()
            .filter(|c| c.kind == Some(SoundKind::Music))
            .map(|c| c.code)
            .collect();
        let effects: Vec<u16> = sound
            .commands
            .iter()
            .filter(|c| matches!(c.kind, Some(SoundKind::Effect(_))))
            .map(|c| c.code)
            .collect();
        let empty: Vec<u16> = sound
            .commands
            .iter()
            .filter(|c| c.header == 0)
            .map(|c| c.code)
            .collect();
        assert_eq!(music, (0x00..=0x26).collect::<Vec<_>>());
        assert_eq!(
            effects,
            (0x2C..=0x91).chain(0xA0..=0x105).collect::<Vec<_>>()
        );
        assert_eq!(empty, (0x27..=0x2B).chain(0x92..=0x9F).collect::<Vec<_>>());
        assert_eq!(music.len(), 39);
        assert_eq!(effects.len(), 204);
        assert_eq!(empty.len(), 19);
        assert!(
            sound
                .commands
                .iter()
                .all(|c| !matches!(c.kind, Some(SoundKind::Adpcm(_))))
        );
        assert!(sound.master_volume < 128);
        assert!(sound.data[0x8003..0x8006].iter().all(|&b| b == 0));
        for code in 0..sound.commands.len() {
            assert_eq!(sound.data[COMMAND_ENTRIES + 4 * code + 3], 0);
        }

        // Priority counts from the catalog (N8): 3, 4, 5, 6, 7 and 10.
        let mut pri = [0u32; 16];
        for c in &sound.commands {
            if let Some(SoundKind::Effect(p)) = c.kind {
                pri[p as usize] += 1;
            }
        }
        assert_eq!(
            (pri[3], pri[4], pri[5], pri[6], pri[7], pri[10]),
            (32, 4, 58, 1, 108, 1)
        );
        assert_eq!(pri.iter().sum::<u32>(), 204);

        for c in sound.sequences() {
            match c.kind {
                Some(SoundKind::Music) => {
                    let n = c.voice_off.iter().filter(|&&o| o != 0).count();
                    assert!((8..=16).contains(&n), "music {:#x} uses {n} voices", c.code);
                }
                Some(SoundKind::Effect(_)) => {
                    assert!(
                        c.voice_off[..8].iter().all(|&o| o == 0),
                        "effect {:#x} uses a low voice",
                        c.code
                    );
                    let n = c.voice_off[8..].iter().filter(|&&o| o != 0).count();
                    assert!((1..=5).contains(&n), "effect {:#x} uses {n} voices", c.code);
                }
                _ => panic!("live command {:#x} has no kind", c.code),
            }
            for v in 0..16 {
                if let Some(addr) = c.track(v) {
                    assert!(
                        (addr as usize) < sound.data.len(),
                        "track {addr:#x} off the image"
                    );
                    let _ = sound.byte(addr);
                }
            }
        }

        assert_eq!(sound.samples.len(), 124);
        assert_eq!(sound.pcm.len(), 0x20_0000);
        assert_eq!(sound.data.len(), 0x2_0000);
        let mut banks = [false; 32];
        let mut sum = 0u32;
        for (i, s) in sound.samples.iter().enumerate() {
            assert!(
                s.start <= s.loop_start && s.loop_start <= s.end,
                "sample {i}"
            );
            assert!(s.bank < 32, "sample {i} bank {}", s.bank);
            banks[s.bank as usize] = true;
            let pcm = sound.sample_pcm(i);
            assert_eq!(pcm.len(), (s.end - s.start) as usize);
            sum += pcm.len() as u32;
        }
        assert_eq!(sum, 1_963_673);
        assert!(banks.iter().all(|&b| b));

        assert_eq!(sound.instruments.len(), 256);
        assert!(
            sound.instruments[..130]
                .iter()
                .all(|i| (i.sample as usize) < sound.samples.len())
        );
        assert!(
            sound.instruments[130..]
                .iter()
                .enumerate()
                .all(|(n, i)| i.sample == (n as u8 + 130))
        );

        let p = &sound.tables.pitch;
        assert_eq!(p.len_normal, std::array::from_fn(|k| (3u8) << k));
        assert_eq!(p.len_triplet, std::array::from_fn(|k| (2u8) << k));
        let dotted = std::array::from_fn(|k| {
            let n = 3u16 << k;
            let d = n * 3 / 2;
            if n % 2 == 0 && d < 256 { d as u8 } else { 0 }
        });
        assert_eq!(p.len_dotted, dotted);
        assert_eq!(
            p.note_base,
            std::array::from_fn(|f| (12 * ((f & 7) + 2 * (f >> 3))) as u8)
        );
        assert_eq!(p.octave, std::array::from_fn(|n| (n / 12) as u8));
        assert_eq!(p.octave_first, std::array::from_fn(|k| (12 * k) as u8));
        assert!(
            p.base
                .iter()
                .enumerate()
                .all(|(n, w)| *w == p.base[n % 12] << (n / 12))
        );
        assert_eq!(p.base[0x3C], 0x1000);
        assert_eq!(sound.word_le(0x1453), p.base[0]);
        for row in &p.frac {
            assert!(row.windows(2).all(|w| w[0] <= w[1]));
        }

        let level = &sound.tables.level;
        for (name, table) in [
            ("volume", &level.volume[..]),
            ("gain", &level.gain[..]),
            ("attack", &level.attack[..]),
            ("decay", &level.decay[..64]),
            ("lfo", &level.lfo_speed[..]),
            ("vibrato", &level.vibrato[..]),
            ("tremolo", &level.tremolo[..]),
        ] {
            assert!(
                table.windows(2).all(|w| w[0] <= w[1]),
                "{name} is not monotone"
            );
        }
        // Index 0x40 is the word past the table (the first word of 0x26A9). It is 0.
        assert_eq!(level.decay.len(), 65);
        assert_eq!(level.decay[0x40], 0);

        let m = &sound.tables.multiply;
        assert_eq!(m.lo.len(), 0x1000);
        assert_eq!(m.hi.len(), 0x1000);
        for nib in 0..16u16 {
            for byte in 0..256u16 {
                let i = (nib * 256 + byte) as usize;
                assert_eq!(m.lo[i], nib * byte);
                assert_eq!(m.hi[i], (nib * byte) << 4);
            }
        }
    }
}
