//! Port of the QSound Z80 sound driver (`re/specs/sound/SPEC.md`, sections 2 to 5).
//!
//! The driver is modelled the way the original runs: a 64 KB Z80 RAM image with the same
//! addresses, the 68000 request ring and the vblank flush, the 250 Hz IRQ, and the main loop.
//! Every DSP register write goes to a log ([`RegWrite`]) so a later mixer, or a test, can read it.
//!
//! Part 1 (task M4-03): mailbox, command dispatch, priorities, track setup and the pan setup
//! of the eight modes. Not yet here: the sequencer (M4-04), envelope, LFO and the volume and
//! pitch writes (M4-05), the pan motion steps, fades, masking of music by effects and twins of
//! mode 7 in the step (M4-06), ADPCM (not used by this ROM).
//!
//! The crate does not depend on `dino-game`. Sound data comes from `dino_assets::sound::SoundData`,
//! which `dino-extract` reads from the user's `dino.zip` at run time. No ROM bytes live here.

mod driver;
mod map;
mod pan;

#[cfg(test)]
mod tests;

pub use driver::{Driver, MailEntry, RegWrite, TrackInfo};
pub use map::{SFX_TRACKS, MUSIC_TRACKS, VOICES};
