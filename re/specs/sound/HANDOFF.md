# Sound work - handoff

## Done
- Kabuki keys found and verified (`kabuki.py`), decrypted images in `re/out/z80_op.bin` / `z80_data.bin`
  (also committed in `re/out/assets/sound/`).
- Own dual-image Z80 disassembler (`z80dis.py`) and full recursive listing (`walkall.py` ->
  `re/out/z80_full.txt`, all 7 jump tables included).
- SPEC.md documents: 68000 queue and mailbox, Z80 IRQ/main loop, command dispatch, special commands, sound table
  layout, track struct, sequence events, envelope, LFO, volume and pitch formulas, DSP register writes.
- **M4-00 (2026-10-13) finished the spec**: pan motion (section 9, with `pan.py`), voice ownership and fades (10), the
  ADPCM player (11, unused by this ROM), every table with length, index and formula (12, `tables.py`), the multiply
  tables and routines (13), samples, instruments and the command table (14), the pitch clamp (15), the MAME checks
  (16, `mame_irq.py`), and what is still unknown (17).
- Checks that can be re-run at any time (from the repo root, `python3 -I re/specs/sound/NAME.py`):
  `tables.py` (about 3 s), `pan.py` (about 15 s), `mame_irq.py` (about 10 s, one headless MAME process).
- No file in `dino-rs/` was changed by this work. The workspace is not touched.

## What the driver port (M4-02 onward) needs to know
1. **Time**: IRQ 250 Hz [M], one 68000 request handed to the Z80 per IRQ at most, one request per 68000 frame; the
   sequencer runs on every 4th tick (62.5 Hz). One video frame is 4.19 ticks. The Z80 clock is 8 MHz.
2. **Per step writes**: a track step that reads no event does the pan step, LFO, envelope and writes volume and pitch
   (and pan while a pan motion runs), even when the values did not change. A step that reads events does none of this.
   Register trace equality (M4-05, M4-09) depends on it.
3. **Pan motion** is `PanModel` of `pan.py` (modes 0 to 7, the mode 7 twin on the next voice with a 4 step delay). It
   is checked against the ROM on 4000 setups and 1.3 million steps. Pan 0..0x20, register `0x110 + pan`, mono forces 0x10.
   Request bytes: d1 = mode<<16 | spread<<8 | pan, d2 = speed<<16 | speed2.
4. **Masking**: an SFX on voice v mutes the music track of v (it keeps running, writes nothing); on SFX end a held
   (tied) note resumes at once, other notes at the next key-on. See SPEC 10 for the exact rule.
5. **Fades**: the music fade F033 multiplies all tracks, music and SFX. F030 stays set after the fade, so everything is
   silent until 0xFF00 or a cancel. A new music code does not clear it.
6. **Multiply**: `(a as u32 * b as u32) >> 16` is exact for the routine 0x29CB; 0x29AA is an 8 by 8 product.
7. **Tables**: five tables have no closed form and need exact copies (0x23A9, 0x25A9, 0x2629, 0x26A9, 0x27A9); 0x1453 and
   0x152B reduce to 12 base words and 12 slopes (SPEC 12, exact copies are simpler); 0x13DE, 0x144A, 0x0C50, 0x0C57, 0x0C5E,
   0x0C65 and 0x28A9 have exact formulas, 0x24A9 an approximate one. Table 0x2629 is read at index 0x40 once (instrument
   16): treat it as 65 entries, the last = 0.
8. **Loops**: command 0E..11 with operand n plays the body n + 1 times in total.
9. **Samples**: 124, ROM order cd-q1..q4 (banks 0..31). Instruments 0..129 are real. The ADPCM code is dead in this ROM and
   can be left out.
10. **Pitch clamp** at 0x139D is inverted but no note of the ROM reaches it; port it as is.
11. **Twin headers** (`0x65/0xE4`, hero voice banks, `0x2C/0xA0/0xA1` ...) use different voices, so they do not cut each
    other off.

## Next steps (in order, see plan/game/design/D5-final-plan.md M4-01 to M4-11)
1. M4-01 MAME register trace recorder, M4-09 comparer. Use the trace to settle the open item "stage clear jingle after
   a fade" (SPEC 17) and to check the per step writes of point 2.
2. M4-02 `dino-assets/src/sound.rs` + `dino-extract/src/sound.rs` (add `cd_q.5k` and `cd-q1.1k..cd-q4.4k` with CRCs
   to `rom.rs`; Kabuki decrypt in the extractor; tables as typed data).
3. M4-03 to M4-06 `crates/dino-sound` (Driver::command(code,d1,d2), tick at 250 Hz, sequencer every 4th tick); M4-07/08
   the DSP register model and mixer at 24038 Hz; then `dino-game/src/audio.rs` with cpal.
4. The 68000 code table (who sends which code, with d1/d2) is done in `plan/game/design/D4e-sound-cues.md`.
5. The game API needs d1/d2 for positional SFX: propose `Game::sound_ex(code, d1, d2)` to the main session.

## Known issues / open questions (all answered or marked unknown with the reason)
- **Pan law, FIR filters and echo taps**: unknown, they live in the QSound DSP internal ROM (`dl-1425.bin`), which is not
  in dino.zip (the file in `re/mame/roms/qsound/` is a zero placeholder). Use a simple equal power pan over the 33 pan
  steps and a plain echo (feedback reg 0x93, length 0xD9). The register writes can still be matched exactly. SPEC 8, 17.
- **Sample ROM order for cd-q1..q4**: answered. MAME loads q1 at 0, q2 at 0x80000, q3 at 0x100000, q4 at 0x180000, the
  CRCs of the zip are equal, and every sample address is inside that 2 MB (highest end 0x1FF9AF). SPEC 14, 16.
- **Pitch clamp at 0x139D**: answered. It is inverted for both sides (too high gives pitch 0, too low gives the highest),
  and no note, vibrato or music transpose of the 243 codes reaches it. Keep it as in the ROM. SPEC 15.
- **Meaning of `$4CC(a5)` and `$777C(a5)` bit 5**: answered by a MAME poke test. `$4CC` is the in-game flag, bit 5 of
  `$777C` is the "demo sound" cabinet setting (factory off). SPEC 3, 16.
- **Where the 68000 writes 0xFF to 0xF19FFF** (was [U] in SPEC 3): answered, the boot RAM test at 0xAAA9C fills all odd
  bytes of 0xF18000-0xF1A000 with 0xFF. SPEC 3.
- **IRQ rate**: answered, 249.997 Hz in MAME. SPEC 2, 16.
- **Sample count**: answered, 124 (the table ends where the instrument table begins; 121 are used). SPEC 14.
- **0x2A81 and 0x4A81 exact products**: confirmed, and 0x29CB is exactly `(a*b)>>16`. SPEC 13.
- Unknown on purpose: which side pan 0 is (listening test), the closed form of the floating point like tables, and the
  stage clear jingle after a fade (needs the MAME trace of M4-01). SPEC 17.
- Corrections for the design documents (the main session decides about D5-errata): D4e s0 calls the middle byte of d1
  "fixed", it is the pan spread; D4e s0 lists 0xFF0B as "commands enabled", it clears the Z80 debug flag (the enable is the
  0x88 byte at 0xF19FFB).
