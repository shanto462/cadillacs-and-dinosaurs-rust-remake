//! Loads the user's own `dino.zip` (MAME set "dino", World 930201) and rebuilds the
//! memory images the original hardware sees:
//!
//! * `program`: 1.5 MB of 68000 code and data. The three EPROMs store each 16-bit word
//!   byte-swapped (MAME `ROM_LOAD16_WORD_SWAP`), so we swap them back to big-endian.
//! * `gfx`: 4 MB of tile graphics. Eight mask ROMs are interleaved two bytes at a time
//!   into 8-byte groups (MAME `ROM_LOAD64_WORD`), which is the layout the video chip reads.
//! * `audiocpu`: the Z80 program `cd_q.5k` (still Kabuki-encrypted in the low 32 KB).
//! * `qsound`: the sample ROMs `cd-q1.1k` .. `cd-q4.4k`, 512 KB each, in MAME load order.
//!
//! Nothing here is copied into the project: the data lives only in memory at runtime.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// (file name, expected CRC32). The CRCs were observed in the user's set and match MAME's "dino".
const EXPECTED: &[(&str, u32)] = &[
    ("cde_23a.8f", 0x8f4e585e),
    ("cde_22a.7f", 0x9278aa12),
    ("cde_21a.6f", 0x66d23de2),
    ("cd-1m.3a", 0x8da4f917),
    ("cd-3m.5a", 0x6c40f603),
    ("cd-2m.4a", 0x09c8fc2d),
    ("cd-4m.6a", 0x637ff38f),
    ("cd-5m.7a", 0x470befee),
    ("cd-7m.9a", 0x22bfb7a3),
    ("cd-6m.8a", 0xe7599ac4),
    ("cd-8m.10a", 0x211b4b15),
    // QSound program and samples (MAME region audiocpu / qsound, SPEC sound section 16).
    ("cd_q.5k", 0x605fdb0b),
    ("cd-q1.1k", 0x60927775),
    ("cd-q2.2k", 0x770f4c47),
    ("cd-q3.3k", 0x2f273ffc),
    ("cd-q4.4k", 0x2c67821d),
];

/// Sample ROMs in MAME load order: bank 0 at the start of q1, bank 31 at the end of q4.
const QSOUND: [&str; 4] = ["cd-q1.1k", "cd-q2.2k", "cd-q3.3k", "cd-q4.4k"];

/// Program EPROMs in address order.
const PROGRAM: [&str; 3] = ["cde_23a.8f", "cde_22a.7f", "cde_21a.6f"];

/// Graphics mask ROMs: (name, base address, byte offset inside each 8-byte group).
const GFX: [(&str, usize, usize); 8] = [
    ("cd-1m.3a", 0x000000, 0),
    ("cd-3m.5a", 0x000000, 2),
    ("cd-2m.4a", 0x000000, 4),
    ("cd-4m.6a", 0x000000, 6),
    ("cd-5m.7a", 0x200000, 0),
    ("cd-7m.9a", 0x200000, 2),
    ("cd-6m.8a", 0x200000, 4),
    ("cd-8m.10a", 0x200000, 6),
];

pub struct Rom {
    pub program: Vec<u8>,
    pub gfx: Vec<u8>,
    /// Raw `cd_q.5k` (128 KB). The low 32 KB is Kabuki-encrypted; `sound::extract` decrypts it.
    pub audiocpu: Vec<u8>,
    /// `cd-q1.1k` .. `cd-q4.4k` concatenated (2 MB).
    pub qsound: Vec<u8>,
}

impl Rom {
    /// Finds `dino.zip`: `--rom <path>`, then `$DINO_ROM`, then `./dino.zip`, then `../dino.zip`.
    pub fn locate() -> Result<PathBuf, String> {
        let args: Vec<String> = std::env::args().collect();
        if let Some(i) = args.iter().position(|a| a == "--rom") {
            return args
                .get(i + 1)
                .map(PathBuf::from)
                .ok_or("--rom needs a path".into());
        }
        if let Ok(p) = std::env::var("DINO_ROM") {
            return Ok(PathBuf::from(p));
        }
        for p in ["dino.zip", "../dino.zip"] {
            if Path::new(p).exists() {
                return Ok(PathBuf::from(p));
            }
        }
        Err("dino.zip not found. Pass --rom <path> or set DINO_ROM.".into())
    }

    pub fn load(path: &Path) -> Result<Rom, String> {
        let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut files: HashMap<&str, Vec<u8>> = HashMap::new();
        for &(name, crc) in EXPECTED {
            let mut entry = zip
                .by_name(name)
                .map_err(|_| format!("{name} missing from {}", path.display()))?;
            if entry.crc32() != crc {
                return Err(format!(
                    "{name}: CRC {:08x}, expected {crc:08x} (wrong ROM set?)",
                    entry.crc32()
                ));
            }
            let mut data = Vec::with_capacity(entry.size() as usize);
            entry
                .read_to_end(&mut data)
                .map_err(|e| format!("{name}: {e}"))?;
            files.insert(name, data);
        }

        let mut program = Vec::with_capacity(0x180000);
        for name in PROGRAM {
            let chunk = &files[name];
            for pair in chunk.chunks_exact(2) {
                program.extend_from_slice(&[pair[1], pair[0]]);
            }
        }

        let mut gfx = vec![0u8; 0x400000];
        for (name, base, offset) in GFX {
            for (i, pair) in files[name].chunks_exact(2).enumerate() {
                let at = base + offset + i * 8;
                gfx[at] = pair[0];
                gfx[at + 1] = pair[1];
            }
        }

        let audiocpu = files.remove("cd_q.5k").expect("cd_q.5k");
        let mut qsound = Vec::with_capacity(0x20_0000);
        for name in QSOUND {
            qsound.extend(files.remove(name).expect(name));
        }
        if audiocpu.len() != 0x20000 || qsound.len() != 0x20_0000 {
            return Err(format!(
                "sound ROM size: cd_q.5k {:#x} (expected 0x20000), samples {:#x} (expected 0x200000)",
                audiocpu.len(),
                qsound.len()
            ));
        }
        Ok(Rom {
            program,
            gfx,
            audiocpu,
            qsound,
        })
    }

    pub fn u8(&self, addr: u32) -> u8 {
        self.program[addr as usize]
    }

    pub fn u16(&self, addr: u32) -> u16 {
        let a = addr as usize;
        u16::from_be_bytes([self.program[a], self.program[a + 1]])
    }

    pub fn i16(&self, addr: u32) -> i16 {
        self.u16(addr) as i16
    }

    pub fn u32(&self, addr: u32) -> u32 {
        let a = addr as usize;
        u32::from_be_bytes(self.program[a..a + 4].try_into().unwrap())
    }

    /// Reads an '@'-terminated string (the game's text format).
    pub fn text(&self, addr: u32, max: usize) -> String {
        let start = addr as usize;
        self.program[start..]
            .iter()
            .take(max)
            .take_while(|&&b| b != b'@')
            .map(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '?'
                }
            })
            .collect()
    }
}
