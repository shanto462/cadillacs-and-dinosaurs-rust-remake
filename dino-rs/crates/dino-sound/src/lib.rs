//! Port of the QSound Z80 sound driver (`re/specs/sound/SPEC.md`, sections 2 to 5).
//!
//! The driver is modelled the way the original runs: a 64 KB Z80 RAM image with the same
//! addresses, the 68000 request ring and the vblank flush, the 250 Hz IRQ, and the main loop.
//! Every DSP register write goes to a log ([`RegWrite`]) so a later mixer, or a test, can read it.
//!
//! Part 1 (task M4-03): mailbox, command dispatch, priorities, track setup and the pan setup
//! of the eight modes. Part 2 (task M4-04, `seq.rs`): the sequencer step, events, notes, key-on,
//! the commands and track end. Part 3 (task M4-05, `voice.rs`): the no-event step (pan motion
//! steps, LFO, portamento, envelope) and the volume and pitch writes. Not yet here: ADPCM (not
//! used by this ROM).
//!
//! The crate does not depend on `dino-game`. Sound data comes from `dino_assets::sound::SoundData`,
//! which `dino-extract` reads from the user's `dino.zip` at run time. No ROM bytes live here.

mod driver;
mod map;
mod pan;
mod seq;
mod voice;

#[cfg(test)]
mod tests;

pub use driver::{Driver, MailEntry, RegWrite, TrackInfo};
pub use map::{SFX_TRACKS, MUSIC_TRACKS, VOICES};
