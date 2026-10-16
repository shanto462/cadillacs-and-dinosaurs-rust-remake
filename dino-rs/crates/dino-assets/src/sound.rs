//! QSound driver data (`re/specs/sound/SPEC.md` sections 1, 5, 12 and 14). Filled by
//! `dino-extract/src/sound.rs`. The game never reads the ROM: every table here is an exact copy
//! taken at run time from the user's `dino.zip`.
//!
//! `data` is the decrypted data image of `cd_q.5k` (linear address = index, SPEC section 2).
//! Sequence bytes live there and are read by address. The structs below are the same bytes
//! grouped the way the Z80 program reads them. `pcm` is `cd-q1.1k` .. `cd-q4.4k` in that order.

/// Header byte of a command-table entry (0x017D).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundKind {
    /// Header byte 0.
    Music,
    /// Header byte 1..=0x7F: the effect priority. A start replaces a voice only when this is
    /// greater than or equal to the effect already on that voice.
    Effect(u8),
    /// Header byte >= 0x80. This ROM has none (SPEC section 11).
    Adpcm(u8),
}

/// One slot of the command table at linear 0x8006 (4 bytes: bank, hi, lo, unused).
/// A non-empty slot is one sequence: the header byte, then 16 voice tracks.
#[derive(Clone, Debug)]
pub struct SoundCommand {
    /// Sound code. The table has the big-endian count at linear 0x8000 (0x106 in this ROM).
    pub code: u16,
    /// 24-bit header pointer (`bank << 16 | hi << 8 | lo`). 0 = empty slot.
    pub header: u32,
    pub kind: Option<SoundKind>,
    /// Big-endian offset of each voice's track from `header`. 0 = that voice is unused.
    /// The track bytes start at [`Self::track`] and run in `SoundData::data` (0x0A28).
    pub voice_off: [u16; 16],
}

impl SoundCommand {
    /// Linear address of voice `v`'s sequence, or `None` when the voice is unused.
    pub fn track(&self, voice: usize) -> Option<u32> {
        let off = self.voice_off[voice];
        if self.header == 0 || off == 0 {
            None
        } else {
            Some(self.header + off as u32)
        }
    }
}

/// One row of the sample table at 0x6A82 (8 bytes, SPEC section 14).
#[derive(Clone, Copy, Debug, Default)]
pub struct Sample {
    /// QSound bank 0..31. The PCM address is `bank * 0x10000 + offset` in [`SoundData::pcm`].
    pub bank: u8,
    pub start: u16,
    pub loop_start: u16,
    pub end: u16,
    /// Root key. Pitch register 0x1000 plays the sample at the QSound rate.
    pub root_key: u8,
}

/// One row of the instrument table at 0x6E62 (8 bytes, command 08, SPEC section 14).
/// Entries 0..129 are real. Entries 130..255 are filler and nothing refers to them.
#[derive(Clone, Copy, Debug, Default)]
pub struct Instrument {
    pub sample: u8,
    /// Fine tune. The sequencer adds it as a signed byte (SPEC section 7).
    pub fine: u8,
    /// Attack rate index into [`LevelTables::attack`] (0x25A9).
    pub attack: u8,
    /// Decay rate index into [`LevelTables::decay`] (0x2629).
    pub decay: u8,
    /// Sustain level index into [`LevelTables::gain`] (0x24A9).
    pub sustain: u8,
    /// Slow decay rate index into [`LevelTables::decay`].
    pub slow_decay: u8,
    /// Release rate index into [`LevelTables::decay`].
    pub release: u8,
    /// Byte 7. The driver does not read it.
    pub unused: u8,
}

/// Note length, octave and pitch-register tables (SPEC section 12).
#[derive(Clone, Debug)]
pub struct PitchTables {
    /// 0x0C50, 7 bytes. Triplet length units, index `(note byte >> 5) - 1`.
    pub len_triplet: [u8; 7],
    /// 0x0C57, 7 bytes. Normal length units.
    pub len_normal: [u8; 7],
    /// 0x0C5E, 7 bytes. Dotted length units.
    pub len_dotted: [u8; 7],
    /// 0x0C65, 16 bytes. Note base, index `track flags & 0x0F`.
    pub note_base: [u8; 16],
    /// 0x13DE, 108 bytes. Octave of n.
    pub octave: [u8; 0x6C],
    /// 0x144A, 9 bytes. First n of each octave.
    pub octave_first: [u8; 9],
    /// 0x1453, 108 little-endian words. Base pitch register.
    pub base: [u16; 0x6C],
    /// 0x152B, 12 rows of 256 bytes. Fraction added into the pitch register.
    pub frac: [[u8; 256]; 12],
}

impl Default for PitchTables {
    fn default() -> Self {
        Self {
            len_triplet: [0; 7],
            len_normal: [0; 7],
            len_dotted: [0; 7],
            note_base: [0; 16],
            octave: [0; 0x6C],
            octave_first: [0; 9],
            base: [0; 0x6C],
            frac: [[0; 256]; 12],
        }
    }
}

/// Volume, envelope and LFO tables, little-endian words (SPEC section 12).
#[derive(Clone, Debug)]
pub struct LevelTables {
    /// 0x23A9, 128 words. Channel volume (command 07) and echo send (command 19).
    pub volume: [u16; 128],
    /// 0x24A9, 128 words. Music master volume, command 1A, and the sustain level.
    pub gain: [u16; 128],
    /// 0x25A9, 64 words. Attack rate.
    pub attack: [u16; 64],
    /// 0x2629, 65 words. Decay, slow decay and release.
    /// Index 0x40 is the first word of the next table (0x26A9). Instrument 16 reads it once,
    /// and that word is 0, so the note never releases (SPEC section 12).
    pub decay: [u16; 65],
    /// 0x26A9, 128 words. LFO speed (command 1D).
    pub lfo_speed: [u16; 128],
    /// 0x27A9, 128 words. Vibrato depth (command 1B), 8.8 semitones.
    pub vibrato: [u16; 128],
    /// 0x28A9, 128 words. Tremolo depth (command 1C).
    pub tremolo: [u16; 128],
}

impl Default for LevelTables {
    fn default() -> Self {
        Self {
            volume: [0; 128],
            gain: [0; 128],
            attack: [0; 64],
            decay: [0; 65],
            lfo_speed: [0; 128],
            vibrato: [0; 128],
            tremolo: [0; 128],
        }
    }
}

/// The two product tables used by 0x29AA and 0x29CB (SPEC section 13).
#[derive(Clone, Debug, Default)]
pub struct MultiplyTables {
    /// 0x2A81, 4096 little-endian words. `word[nibble * 256 + byte] = nibble * byte`.
    pub lo: Vec<u16>,
    /// 0x4A81, 4096 little-endian words. `word[nibble * 256 + byte] = (nibble * byte) << 4`.
    pub hi: Vec<u16>,
}

/// Pitch, level and multiply tables.
#[derive(Clone, Debug, Default)]
pub struct SoundTables {
    pub pitch: PitchTables,
    pub level: LevelTables,
    pub multiply: MultiplyTables,
}

/// Everything the sound driver reads.
#[derive(Clone, Debug, Default)]
pub struct SoundData {
    /// Decrypted data image of `cd_q.5k`. Index = linear address.
    pub data: Vec<u8>,
    /// Sample ROM: `cd-q1.1k` .. `cd-q4.4k`, 512 KB each, banks 0..31.
    pub pcm: Vec<u8>,
    /// Byte at linear 0x8002. Indexes [`LevelTables::gain`] for the music master volume (F016).
    pub master_volume: u8,
    /// Command table, one entry per code, including the empty slots.
    pub commands: Vec<SoundCommand>,
    /// Sample table 0x6A82. 124 entries in this ROM. The instrument table starts where this ends.
    pub samples: Vec<Sample>,
    /// Instrument table 0x6E62. 256 entries.
    pub instruments: Vec<Instrument>,
    pub tables: SoundTables,
}

impl SoundData {
    /// Data-image byte at a linear address.
    pub fn byte(&self, addr: u32) -> u8 {
        self.data[addr as usize]
    }

    /// Little-endian word, the way the Z80 reads a table (`ld a,(hl); inc hl; ld h,(hl)`).
    pub fn word_le(&self, addr: u32) -> u16 {
        let a = addr as usize;
        u16::from_le_bytes([self.data[a], self.data[a + 1]])
    }

    pub fn command(&self, code: u16) -> Option<&SoundCommand> {
        self.commands.get(code as usize)
    }

    /// Live command-table entries. Each one is a sequence (39 music and 204 effects in this ROM).
    pub fn sequences(&self) -> impl Iterator<Item = &SoundCommand> {
        self.commands.iter().filter(|c| c.header != 0)
    }

    /// PCM bytes of sample `index`: `bank * 0x10000 + start .. end`.
    /// `end` is the address the driver writes to the DSP (SPEC section 7), and the length
    /// used by the catalog is `end - start`.
    pub fn sample_pcm(&self, index: usize) -> &[u8] {
        let s = &self.samples[index];
        let base = (s.bank as usize) << 16;
        let a = base + s.start as usize;
        let b = base + s.end as usize;
        &self.pcm[a..b]
    }
}
