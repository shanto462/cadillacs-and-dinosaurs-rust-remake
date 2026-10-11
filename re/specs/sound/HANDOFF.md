# Sound work - handoff

## Done
- Kabuki keys found and verified (`kabuki.py`), decrypted images in `re/out/z80_op.bin` / `z80_data.bin`.
- Own dual-image Z80 disassembler (`z80dis.py`) and full recursive listing (`walkall.py` ->
  `re/out/z80_full.txt`, all 7 jump tables included). Code is 0x0000-0x13DD and 0x208A-0x2A80;
  0x13DE-0x2129 and 0x23A9-0x29A9 are tables; 0x2A81-0x6A81 multiply tables; 0x6A82 samples;
  0x6E62 instruments.
- Read and documented in SPEC.md: 68000 queue and mailbox, Z80 IRQ/main loop, command dispatch,
  special commands, sound table layout, track struct, sequence events, envelope, LFO, volume and
  pitch formulas, DSP register writes.
- No file in `dino-rs/` was changed yet. The workspace is not touched.

## Next steps (in order)
1. Read the pan-motion handlers 0x1030-0x1196 and ADPCM pan 0x2222-0x2384 in detail (only skimmed).
2. Dump tables from the data image: 0x0C50/0x0C57/0x0C5E (lengths), 0x0C65 (octaves), 0x13DE,
   0x144A, 0x1453, 0x152B (pitch), 0x23A9/0x24A9/0x25A9/0x2629/0x26A9/0x27A9/0x28A9, 0x6A82 samples
   (count unknown: find max sample index used by 0x6E62), 0x6E62 instruments. Check that 0x2A81 and
   0x4A81 are exact nibble*byte products (then 0x29CB = `(a*b)>>16`).
3. Build the 68000 code table: list every caller of 0xA63C (198 sites in `re/out/sweep.txt`) with d0,
   d1, d2; map music codes 0x00-0x26 to stages (find the stage-start caller).
4. Find the IRQ rate (MAME: 250 Hz) and confirm the 68000 write of 0xFF to 0xF19FFF.
5. Implement: `dino-assets/src/sound.rs` (tables + sample ROM as typed data), `dino-extract/src/sound.rs`
   (+ add `cd_q.5k` and `cd-q1.1k..cd-q4.4k` with CRCs to `rom.rs`; Kabuki decrypt in the extractor),
   new crate `crates/dino-sound` (Driver::command(code,d1,d2), tick at 250 Hz, sequencer every 4th
   tick, DSP register model + mixer at 24038 Hz, render(&mut [f32])), example `play` writing WAV to
   `re/out/`, then `dino-game/src/audio.rs` with cpal.
6. The game API needs d1/d2 for positional SFX: propose `Game::sound_ex(code, d1, d2)` to the main session.

## Known issues / open questions
- Pan law, FIR filters and echo exact taps live in the QSound DSP internal ROM (not in dino.zip):
  use a simple equal-power pan and a plain echo (feedback reg 0x93, length 0xD9) as approximation.
- Sample ROM order for cd-q1..q4 (MAME loads them in sequence, 512 KB each) [check CRCs in MAME].
- Pitch clamp at 0x139D looks inverted (too-high notes become 0); keep it as in the ROM.
- Meaning of `$4CC(a5)` and `$777C(a5)` bit 5 in the vblank flush (attract-mode mute) is inferred.
