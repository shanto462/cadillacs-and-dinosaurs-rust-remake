# Phase 3: sound

**Goal:** music and sound effects that match the original: same notes, timing, instruments,
volumes, panning and priorities, played in real time.

## Tasks (one agent each, in order)

1. **Driver data.** Extract the decrypted Z80 driver tables, sequences, instrument table and the
   2 MB sample ROM into `dino-assets` (Kabuki keys and formats in `re/specs/sound/SPEC.md`).
2. **Driver port (`crates/dino-sound`).** Command handling, sequencer, channel allocation and
   priority, envelopes, all in the driver's own tick rate. Unit tests on sequence decoding.
3. **QSound mixer.** 16 PCM voices (rate, volume, pan, loop), the QSound panning and echo, at the
   original output rate, then resampled for the device.
4. **Output and wiring.** `dino-game/src/audio.rs` with `cpal`; `Game::sound` extended with the d1/d2
   parameters that positional effects use; the game's sound requests go to the driver every frame.
5. **Exactness pass.** Compare the sound request stream with the phase 0 traces (exact). For the
   audio itself, compare rendered WAVs against MAME's WAV output (`-wavwrite`) for a list of codes:
   same note events and timing, close waveforms.

## Done when

The request stream matches exactly, every music track and effect plays, and the WAV comparison
shows the same events and timing.

## Status

Pending. Reverse engineering done: `re/specs/sound/SPEC.md`, `HANDOFF.md`.
