# Sound system (QSound Z80 driver) - SPEC

Status: finished by task M4-00 (2026-10-13). The driver, the tables and the pan motion are read and checked; what is
still unknown is listed in section 17.

Tags: **[O]** observed in code/data, **[I]** inferred, **[U]** unknown / not checked yet, **[M]** measured in MAME.
Z80 addresses are 16-bit CPU addresses. "Linear" addresses are 24-bit offsets into `cd_q.5k`.
Offsets inside a track (`+36`) are hex, as in the Z80 listing (`ix+54` is `+36`).

Tools in this folder (run with `python3 -I`, from the repo root; `tables.py`, `pan.py` and `mame_irq.py` read `dino.zip`
at run time, the older three use the extracted `re/rom/cd_q.5k` and `re/out/z80_*.bin`):

| tool | what it does |
| --- | --- |
| `kabuki.py` | decrypts `cd_q.5k` (writes `re/out/z80_op.bin`, `re/out/z80_data.bin`) |
| `z80dis.py` | dual-image Z80 disassembler (`--range a b` for linear listings) |
| `walkall.py` | recursive listing with all known jump tables (output `re/out/z80_full.txt`) |
| `z80mini.py` | small Z80 interpreter that runs routines of the real program (used by the two checks below) |
| `tables.py` | checks every table of section 12, the multiply routines (13), samples and instruments (14), the pitch clamp (15) |
| `pan.py` | `PanModel`: the pan motion of section 9 in Python, compared with the ROM routines on 4000 setups and 1.3 million steps |
| `mame_irq.py` | one headless MAME run: Z80 interrupt rate and the attract mute of the 68000 flush (section 16) |

The decrypted program and its disassembly are also committed in `re/out/assets/sound/`
(`z80_opcode_image.bin`, `z80_data_image.bin`, `z80_listing.txt`).

## 1. Kabuki decryption [O]

- Only bytes 0x0000-0x7FFF of `cd_q.5k` are encrypted. 0x8000-0x1FFFF (music/sfx data) is plain.
- Algorithm = MAME `kabuki_decode`. Keys for dino: swap_key1 `0x76543210`, swap_key2 `0x24601357`,
  addr_key `0x4343`, xor_key `0x43`.
- Opcode image: select = `addr + 0x4343`. Data image: select = `(addr ^ 0x1FC0) + 0x4343 + 1`.
- Opcode fetches (M1: first opcode byte, and the second byte after CB/DD/ED/FD) use the opcode
  image. Operands, displacements, the last byte of `DD CB d op`, and all data reads use the data image.
- Verified: reset code decodes to `di; im 1; ld sp,$ffff; ld hl,$f000; ld de,$f001; ld bc,$0fff;
  ld (hl),0; ldir; ...`, and the data image shows the text " version 1.00   1992 / JUNE" at 0x1A.
- In the listing `z80_listing.txt` the hex column shows the data image bytes, the text is decoded from the opcode
  image. So `dd 56 37  ld h,(ix+55)` is correct: the real second byte is 0x66.

## 2. Hardware map seen by the Z80 [O unless tagged]

- 0x0000-0x7FFF fixed ROM, 0x8000-0xBFFF banked window, 0xC000-0xCFFF shared RAM with the 68000 (68000 address
  `0xF18000 + 2*n + 1` = Z80 `0xC000 + n`), 0xF000-0xFFFF Z80 work RAM.
- 0xD000/0xD001 = QSound data high/low, 0xD002 = register number (write commits), 0xD003 = bank
  register (driver always writes `0x80 | bank`), 0xD007 read: bit 7 = DSP ready.
- Write helper 0x0A13: wait D007 bit7, write E to D001, D to D000, A to D002 (reg A = value DE).
- Linear address -> bank: the driver's read routine (0x09BC, also inlined at 0x0A8B and 0x0FC7)
  takes a 24-bit pointer B:HL. If `addr < 0xC000` it reads directly (fixed ROM or bank 0); else
  bank = `(addr >> 14) - 2`, CPU address `0x8000 | (addr & 0x3FFF)`. So **linear address = file
  offset in cd_q.5k**. The pointer is incremented with carry into B.
- IRQ (IM 1, vector 0x0038) **[M]**: 250 Hz. MAME 0.289 (`mame_irq.py`): the counter F004 advanced 66564 times in
  266.26 emulated seconds = 249.997 Hz (the rest is the quantisation of one read per video frame). The Z80 clock is
  8 MHz [M: `mame -listxml dino`]. The 68000 flush runs once per video frame (59.6374 Hz, [M]), so 4.19 IRQ ticks
  happen per video frame and at most one 68000 request is handed over per frame.

## 3. 68000 side [O]

- Enqueue 0xA63C: entry = 16 bytes at `$3A(a5) + $34(a5)`: word d0 (code), long d1, long d2
  (6 bytes unused). Write index `$34`, read index `$36`, both step 0x10 and wrap at 0x400
  (64 entries). If the ring is full the entry is overwritten but not committed.
- Flush in vblank (0x610), one entry per frame, only when `$36 != $34` and `0xF1801F == 0xFF`:
  - Normal code (high byte of d0 != 0xFF): C000 = d0 high, C001 = d0 low, C006 = d1 bits 23-16,
    C007 = d1 bits 7-0, C008 = d1 bits 15-8, C009..C00C = d2 bytes (high to low), C00F = 0.
  - Special code (d0 high byte 0xFF): C000 = 0xFF, C001 = sub-command, C003 = d1 bits 31-24,
    C004 = d1 bits 23-16, C00F = 0.
  - Attract mute: if `$4CC(a5) == 0` and code != 0x0030 and `($777C(a5) & 0x20) == 0`, the entry is dropped.
    **[M]** confirmed in MAME (section 16): `G.$4CC` (0xFF84CC) is the in-game flag (1 after the start press) and
    bit 5 of `G.$777C` (0xFFF77C) is the "demo sound" setting of the cabinet menu (factory: off). With both clear,
    100 seconds of attract let through only 0xFF00 and 0xFF0B. With bit 5 set, or with `G.$4CC = 1`, all the
    attract and game codes pass.
- Meaning of the d1/d2 bytes for the pan motion (the Z80 reads them as `iy+6 .. iy+12` of the ring entry, section 9):
  **d1 = mode << 16 | spread << 8 | pan**, **d2 = speed << 16 | speed2** (speed and speed2 are 16-bit big endian
  words, signed 8.8 pan units per sequencer step). The "middle byte" of d1 that D4e s0 calls fixed is the spread.
- Init 0xA5FA: write 0x88 to 0xF19FFB (Z80 CFFD: the IRQ copies requests only while CFFD == 0x88), clear the ring,
  send 0xFF0B.
- 0xA716: 0xF19FFD (Z80 CFFE) = 0xFF if `$777A(a5)` bit 7 set (stereo) else 0 (mono).
- Boot handshake **[O, answers an old [U]]**: the Z80 writes 0x77 to CFFF (68000 0xF19FFF) and then waits for CFFF ==
  0xFF. The 68000 RAM test at 0xAAA7A waits for 0x77 at 0xF19FFF, tests 0xF18000-0xF1A000, and at 0xAAA9C fills every
  odd byte 0xF18001, 0xF18003 ... 0xF19FFF with 0xFF (`moveq #-1,d0; move.b d0,(a0); addq.w #2,a0`). That fill sets
  CFFF to 0xFF, which releases the Z80. It also sets C00F = 0xFF (mailbox free), CFFD = 0xFF (commands off until the
  0x88 write above) and CFFE = 0xFF (stereo until 0xA716 writes it). No other 68000 code touches 0xF19FFF.
- Helpers: 0xA5D2 sends 0xFF00 (stop all), 0xA5DE 0xFF05 (stop music, never called), 0xA5EA 0xFF07 with
  d1 = 0x04440000 (fade speed 0x0444). Every 0xFFxx constant in the 68000 code (scan of `move.w #$FFxx,d0`): 0xFF00 at
  0xA5D2 (the helper), 0x29D5A, 0xA7464 and 0xA7494; 0xFF05 at 0xA5DE; 0xFF07 at 0xA5EA; 0xFF0B at 0xA626 (init
  0xA5FA), 0xA71C (boot, right after the stereo flag, followed by the stop all of 0xA5D2) and 0xA746E; 0xFF0A at
  0xA73CA. The last four sit in the service (test) screens (the strings just before 0xA73B4 are the I/O test labels)
  [I], so in play only 0xFF00, 0xFF07 and 0xFF0B are sent and F041 stays 0. 0xFF0B sets the Z80 debug flag F041 to 0
  (it is not "commands enabled", that is the 0x88 byte); 0xFF0A sets it to 0xFF (pan values are then also copied
  to C005).
- Positional SFX wrappers 0x96xx-0x9F60 (198 calls to 0xA63C in total): d1 = `mode << 16 | spread << 8 | pan`,
  pan = table at 0xA66A (48 bytes) indexed by `(objX - camX) / 8` clamped to 0..0x17F, d2 = the speed words.
  Some wrappers alternate between two codes with a toggle bit in `$43A(a5)` (e.g. 0x65/0xE4, 0x7B/0xFC,
  0x72/0xF8). The full code table per caller is in `plan/game/design/D4e-sound-cues.md`.

## 4. Z80 main program [O]

- Reset: clear F000-FFFF, write 0x77 to CFFF, toggle bank register 4 IRQ ticks each, wait CFFF == 0xFF,
  init DSP (0x0912), F016 = table 0x24A9[byte at linear 0x8002] (music master volume).
- IRQ 0x0038: F004++. If CFFD == 0x88 and C00F != 0xFF: copy C000-C00F to `F100 + F007` (16-entry
  ring, F007 += 0x10), set C00F = 0xFF (free). F040 = CFFE (stereo flag) is refreshed only while CFFD == 0x88.
- Main loop 0x00BE: wait until F004 != F005, F005++; call 0x017D (handle at most one queued command
  per IRQ tick); every 4th tick (`F005 & 3 == 0`, so 62.5 Hz with a 250 Hz IRQ) run the sequencer:
  - For voice v = 0..15: F00D = v, F00E = v*8, F006 = 0, F020 = 0.
    SFX track at `F200 + 0x40*v`: if active (+20 != 0): if delay +3F != 0, count it down and
    when it hits 0 reset the voice (0x0981) and set pan (the twin of pan mode 7, section 9.4; while it counts down
    F020 stays 0, so music on that voice is not masked yet); else tempo F010 = word +30/+31 and
    step the track (0x0A28). Then F020 = 0xFF (voice is owned by the SFX).
    Music track at `F600 + 0x40*v`: if active: F006 = its +20, tempo F010 = F012 + F014, step it.
    While F020 = 0xFF the music track runs but never writes the DSP (it is masked, section 10).
  - Then 0x212C (ADPCM slots, section 11), then music fade F033 -= F031 (if F030), fade 2 F038 -= F036 (if F035),
    both clamp at 0.
- DSP init 0x0912: reg 0xDE=0, 0xDF=0x2E, 0xE0=0, 0xE1=0x30, 0xE4..0xE7=0x3FFF, 0xE2=0xFFFF,
  0xDA=0x0DB2, 0xDC=0x0E11, 0xE3=0x0039; reset all 16 voices (0x0981: reg 0xBA+v=0, v*8+6=0, v*8+2=0)
  and the 3 ADPCM voices (0x09A8: reg 0xCD+4v=0, 0xD6+v=1).
- Pan write 0x09E7/0x09FE: value e (0..0x20) -> reg `0x80+v` = `0x110 + e`; mono (F040 == 0)
  forces e = 0x10. If F041 != 0 the pan is also copied to C005 (debug, 0x09E7 only).
- Code map **[O]**: code is 0x0000-0x13DD (with small tables inside: the jump tables 0x0269, 0x05A8, 0x0796, 0x0C84,
  0x101D, 0x11FF and the tables 0x0C50-0x0C74), 0x212C-0x23A7 (ADPCM slots and ADPCM pan, jump table 0x2211) and
  0x29AA-0x2A7F (multiply). Everything else up to 0x7FFF is data (section 12). Command 0x1F of the sequence format
  points to 0x208A, inside the pitch fraction table: it is never used by a track.

## 5. Command handling 0x017D [O]

- Code 0xFFxx -> special command table 0x0796 (sub < 12):
  0 stop everything (clear F200-FA5F, F030; reset voices + ADPCM), 1 stop all SFX tracks (0x0EBB each),
  2 stop SFX track `C004` (voice), 3 clear ADPCM slots, 4 stop ADPCM slot `C004`, 5 stop music,
  6 music fade: speed = C003:C004 (0 cancels; else if F030 is already set the request is ignored, else
  F030=FF, F031=speed, F033=0xFFFF), 7 second fade (F035/F036/F038, same rules),
  8 tempo offset F014 = (C004*0x369)>>8 (negated if C003==0xFF),
  9 pitch offset F01B = C004<<4 (negated if C003==0xFF), 10 F041=FF, 11 F041=0.
- Other codes: bank 0; count N = BE word at linear 0x8000 (0x106); code = code mod N; entry at
  `0x8006 + 4*code` = [bank, hi, lo, unused] -> 24-bit header pointer (0 = no sound).
  Header byte h: `h == 0` music, `1..0x7F` SFX with priority h, `>= 0x80` ADPCM one-shot (none exist, section 11).
  - SFX/music: then 16 BE words, one per voice: offset of that voice's track from the header (0 = unused).
  - SFX: for each used voice, start only if `h >= priority of the current SFX on that voice (+32)`.
    Clear 0x40 bytes, ptr = header + offset, +32 = h, +20 = 0xFF, +17 = 1, +25 = 5, pan setup
    from C006 (0x0251, table 0x0269, section 9), reset voice, write pan.
  - Music: F012/F014/F01A/F01B/F035 = 0, F018 = (F016*0xFFFE)>>16; each voice: offset 0 stops that
    music track; else clear 0x30 bytes, set ptr, +20 = 0xFF, +17 = 1, +25 = 5, pan +37 = 0x10;
    if no SFX owns the voice, reset it and set pan 0x10. A new music code does not clear F030 (the fade state).
  - ADPCM (0x04F2): section 11.
- Code table in this ROM [O]: 0x00-0x26 music (header 0), 0x2C-0x91 and 0xA0-0x105 SFX
  (priorities 3..0x0A), no ADPCM headers. The meaning of each song is in D4e s1.

## 6. Track (channel) state, 0x40 bytes [O]

+00..02 ptr (lo, mid, bank) | +03 flags: bits0-2 octave, bit3 octave bank, bit4 dot next note,
bit5 triplet, bit6 tie, bit7 tie active | +04/05 duration counter (8.8, LE) | +06 gate ratio |
+07/08 gate counter | +09 transpose | +0A detune | +0B sample root key | +0C instrument fine tune |
+0D/0E volume (table 0x23A9) | +0F echo send index | +10 portamento speed (bit7 sliding) |
+11/12 slide step | +13/14 current pitch (hi=semitone, lo=fraction; big-endian here) |
+15/16 target pitch | +17 LFO flags (bit0 reset on key-on, bits6-7 quadrant) | +18/19 LFO speed |
+1A/1B LFO phase | +1C/1D vibrato depth | +1E/1F tremolo depth | +20 active = sample+1 |
+21/22 sample start | +23/24 envelope pointer | +25 envelope state 0..5 | +26/27 sustain level |
+28/29 envelope rate | +2A/2B envelope level | +2C..2F loop counters | +30/31 SFX tempo |
+32 priority | +33 mute volume/pitch writes until next key-on | +36 pan mode+1 | +37 pan |
+38 pan fraction | +39 spread | +3A pan limit | +3B/3C pan speed | +3D/3E second pan speed | +3F start delay.
A new track starts with every field 0 except those set in section 5, so a track that never sets a volume (command 07)
would be silent and the echo send index +0F is 0 (no echo) unless the track sets it (command 19). All 727 tracks of the
ROM set the volume and an instrument (08) before their first note.

## 7. Sequence format (0x0A28 / 0x0A8B) [O]

Per 62.5 Hz tick: gate counter -= tempo (on <= 0: release, envelope state 4); duration -= tempo; if
it reaches <= 0 (or high byte >= 0xC1) read events, else run pan motion (table 0x101D), LFO and
envelope (table 0x11FF). **The two cases exclude each other**: the event path ends with `ret` after a note or a
rest (0x0C4F), so in a step that reads events there is no pan step, no LFO step and no envelope step, and no volume
and pitch write. Events:
- `0x20..0xFF` note: length index = (b >> 5) - 1 into table 0x0C57 (normal), 0x0C50 (triplet),
  0x0C5E (dotted, one note); duration += len << 8. Low 5 bits: 0 = rest, else note =
  (b & 0x1F) + table 0x0C65[flags & 0x0F]. Pitch = note + transpose (+ F01A music) in 8.8,
  plus detune+fine (signed), upper clamp: if the high byte is >= 0x62 it becomes 0x61 (low byte kept; there is no
  lower clamp, so a negative sum also ends as 0x61). Then portamento, key-on (unless tie): reg v*8+2=0, 0x93=0x27FF,
  0xD9=0x07FF, 0xBA+v = table23A9[+0F]<<2, v*8+1 = sample start, v*8+3 = 0x8000; gate = dur*gate/256.
  At key-on the envelope starts in state 0 with both the level (+2A/2B) and the rate (+28/29) = table 0x25A9[env[0]]
  (the first step adds the rate to a level that already equals it), and the LFO phase is cleared if +17 bit 0 is set.
- Commands 0x00..0x1F (table 0x0C84): 00 toggle triplet, 01 toggle tie, 02 dot next, 03 toggle
  octave bank, 04 n flags=(flags&0x97)|n, 05 hh ll tempo (SFX: own, music: F012), 06 n gate,
  07 n volume, 08 n instrument (table 0x6E62, 8 bytes, section 14), 09 n octave,
  0A n music transpose (music tracks only, global F01A), 0B n transpose, 0C n detune, 0D n portamento,
  0E..11 n hh ll loop k (the counter starts at n on the first arrival and the offset is taken each time it is not
  yet 0 after a decrement: the loop body runs **n + 1 times** in total), 12..15 f hh ll break (on the last pass, when
  the counter is 1: counter = 0, flags = (flags&0x97)|f, jump), 16 hh ll jump
  (signed BE offset from after the operand), 17 end of track, 18 n pan (music tracks only: SFX tracks ignore it),
  19 n echo send, 1A n music volume (F018 = F016*table24A9[n]>>16), 1B n vibrato depth (0x27A9), 1C n tremolo
  depth (0x28A9), 1D n LFO speed (0x26A9), 1E n LFO flags, 1F unused (points into data).
- Sample table 0x6A82 (8 bytes, data image): bank, start(LE), loop start(LE), end(LE), root key.
  Load: v*8+2=0, v*8+1=start, v*8+5=end, v*8+4=end-loop, reg ((v-1)*8)&0x78 = 0x8000|bank.
- Envelope: state 0 attack (+rate until overflow -> 0xFFFF), 1 decay (to sustain), 3 slow decay, 4 release, 5 off.
  State change (0x128D): new state = state + 1, skipping 2; if the level it ends with is 0 the new state is 5.
  Rates: table 0x25A9[instrument byte 2] for the attack, table 0x2629[...] for the others: decay = instrument byte 3,
  slow decay = byte 5, release = byte 6 (loaded at key-off, 0x0A47). Instrument byte 4 = sustain level index into
  table 0x24A9. See section 12.
- Volume reg v*8+6 = tremolo * env * (music only: F018, and F038 while F035) * F033 (fade, SFX too, section 10) *
  channel volume (each step is `(a*b)>>16`, routine 0x29CB, section 13). The tremolo factor is `0xFFFF -
  (triangle(phase) * depth >> 16)`, and 0xFFFF when the LFO speed is 0. Pitch reg v*8+2 = table 0x1453[n] +
  (table 0x152B[semi*256+frac] << octave 0x13DE[n]), n = pitch - 1 + 0x3C - root key, clamped (section 15).
- Every step in which an unmasked, unmuted track reads no event writes two DSP registers (volume and pitch, 0x12C2),
  plus the pan register while a pan motion runs. These writes are made even when the value did not change. The
  exception is envelope state 5 (off) and state 2 (never used): their handler (0x120B) is a `ret`, so such a step
  writes nothing, but the LFO and portamento part before it (0x1196) still runs.

## 8. QSound DSP model [I: from MAME qsound_hle docs, not the ROM]

Rate = 60 MHz/2/1248 = 24038 Hz. PCM: 8-bit signed samples at `(bank<<16)|addr`, rate 4.12,
loop subtracts loop length once at end, volume 0x4000 = unity (`out = vol*s>>14`). Pan and FIR
taps come from the DSP internal ROM (not in dino.zip), so the port needs its own pan law and filter.
ADPCM: 3 voices, 4-bit, ~8 kHz (the slot timer of section 11 consumes 0x40 bytes per 62.5 Hz step = 4000 bytes/s =
8000 samples/s [O]). Echo: feedback reg 0x93, length via 0xD9.

Registers the driver writes **[O]** (value DE, register number A of the helper 0x0A13):

| register | written with | where |
| --- | --- | --- |
| `((v-1)&15)*8` | the bank register of voice v (voice 0 uses 0x78): `0x8000 \| bank` | sample load 0x0D5E |
| `v*8+1` | sample start | key-on, sample load |
| `v*8+2` | pitch (0 at key-on and at voice reset) | every step |
| `v*8+3` | 0x8000 at key-on | key-on |
| `v*8+4` | end - loop start | sample load |
| `v*8+5` | end | sample load |
| `v*8+6` | volume (0 at reset) | every step |
| `0x80+v` | pan: 0x110 + (0..0x20) | pan write |
| `0xBA+v` | echo send [I: meaning from the MAME docs] = table 0x23A9[+0F] * 4 (0 at reset) | key-on |
| `0x90+n` | ADPCM pan: 0x110 + (0..0x20) | section 11 |
| `0xCA+4n .. 0xCD+4n` | ADPCM start, end, bank (0x80 \| bank), volume * 4 | section 11 |
| `0xD6+n` | 1 (ADPCM start and reset) | section 11 |
| `0x93`, `0xD9` | 0x27FF and 0x07FF at every key-on | key-on |
| `0xDA..0xE7` | constants at init (section 4) | 0x0912 |

Pan law **[U, approximation needed]**: the register holds `0x110 + p`, p = 0..0x20 (0x10 = centre, 33 steps). The
gains per p live in the DSP internal ROM (`dl-1425.bin`, 24576 bytes, CRC d6cf5ef5, the MAME device ROM `qsound`),
which is not in dino.zip; the file in `re/mame/roms/qsound/` is a zero placeholder. An older MAME core used an
equal power law over the 33 steps (gain left = sqrt((32-p)/32), gain right = sqrt(p/32)) [I: from memory of that
source, not checked here]. Which side p = 0 is: [I] the left, because the 68000 table 0xA66A gives 0 for an object
at the left screen edge.

## 9. Pan motion [O, checked by pan.py against the ROM routines]

`PanModel` in `pan.py` is the executable form of this section (setup, twin, steps and the register writes agree with
the ROM on 4000 random setups, 1.32 million steps, and 451,500 ADPCM slot cases).

### 9.1 Fields and the register

Track fields (section 6): +36 mode+1 (0 = no motion), +37 pan (0..0x20, centre 0x10), +38 fraction, +39 spread, +3A limit
(the sweep target), +3B/+3C speed (lo, hi; signed, 8.8 pan units added per step), +3D/+3E second speed, +3F start
delay. An ADPCM slot has the same fields 0x20 lower (+16 mode, +17 pan, ... +1F delay).
Every pan write is `reg 0x80+v = 0x110 + e` (ADPCM: `0x90+n`), where e = pan, or 0x10 in mono (F040 == 0).

### 9.2 Request fields

The request comes from the ring entry (`iy+n`, 68000 side in section 3): `iy+6` low 3 bits = mode, `iy+7` = pan (the
start value, 0..0x20), `iy+8` = spread, `iy+9:iy+10` = speed (BE), `iy+11:iy+12` = second speed (BE). In 68000
terms d1 = mode<<16 | spread<<8 | pan and d2 = speed<<16 | speed2. Only SFX use this. Music tracks have mode 0 and
set their pan with the track command 0x18 (default 0x10); an SFX ignores command 0x18.
Setup runs once, at SFX start (0x0251, jump table 0x0269 by mode), then the pan register is written with the start
pan. `b` below is the start pan, `S` the spread, `v` the speed.

| mode | routine | setup (the track then has: pan = b, fraction 0) | after setup |
| --- | --- | --- | --- |
| 0 | 0x0279 | pan = min(b, 0x20), +36 = 0 | fixed |
| 1 | 0x028A | speed = v; if b >= 0x11: pan = min(b, 0x20) and speed = -v | bounces between 0 and 0x20 for ever |
| 2 | 0x02B4 | if b < 0x11: limit = min(b + S, 0x20), speed = v; else: pan = min(b, 0x20), limit = max(pan - S, 0), speed = -v; spread kept | swings between pan and pan +- S for ever, toward the centre first |
| 3 | 0x02FD | limit = min(b + S, 0x20), speed = v | sweeps up to the limit, then stops |
| 4 | 0x0322 | pan = min(b, 0x20), limit = max(pan - S, 0), speed = -v | sweeps down to the limit, then stops |
| 5 | 0x0353 | as mode 3, second speed = speed2 | up to the limit at v, then up to 0x20 at speed2, stops |
| 6 | 0x0384 | as mode 4, second speed = -speed2 | down to the limit at -v, then down to 0 at the second speed, stops |
| 7 | 0x03C7 | as mode 3 (the speed keeps its sign) | sweeps to the limit in the direction of the speed, stops; and a twin track (9.4) |

The `b + S` sums are 8-bit adds; the compare is `< 0x21` (unsigned). The game never sends a pan above 0x20.

### 9.3 One step (0x1007 -> table 0x101D, handlers 0x1030 to 0x1157, ends at 0x118C)

Run once per sequencer step in which the track reads no event (section 7), while +36 != 0. Let `hl = (pan<<8 | fraction)
+ speed` (16-bit, wraps), `h` its high byte, `neg` = the speed is negative, `out` = `h >= 0x21` (the sum left
0..0x20, also when it went below 0).

| mode | handler | rule |
| --- | --- | --- |
| 0 | 0x102D | nothing (never reached: setup of mode 0 clears +36) |
| 1 | 0x1030 | if out: h = (neg ? 0 : 0x20), speed = -speed |
| 2 | 0x105C | not neg: if h >= limit: h = limit, limit = max(limit - S, 0), speed = -speed. neg: if h <= limit or out: h = limit, limit = min(limit + S, 0x20), speed = -speed |
| 3 | 0x10B4 | if h >= limit: h = limit, stop |
| 4 | 0x10CE | if h <= limit or out: h = limit, stop |
| 5 | 0x10F0 | if h >= limit: h = limit; if limit == 0x20: stop, else speed = speed2, limit = 0x20 |
| 6 | 0x1120 | if h <= limit or out: h = limit; if limit == 0: stop, else speed = speed2, limit = 0 |
| 7 | 0x1157 | not neg: if h >= limit: h = limit, stop. neg: if h <= limit or out: h = limit, stop |

Then (0x118C) pan = h, fraction = low byte of hl, and the pan register is written with e = h (**every step, also when
the pan did not change**, and also in the step that stops). "Stop" means +36 = 0; later steps write nothing. The ADPCM
handlers (table 0x2211, routines 0x2222 to 0x2349) are the same logic on the slot fields, ending at 0x237E and the
pan write 0x2385.

Examples (values from D4e s2 to s4): hero shots and hit sounds use mode 3, or mode 4 when the facing flag +24 of the
object is set, with S = 0x20 and a speed of 0x1500 (21 units per step): the limit is the far edge (0x20 or 0), which
the pan reaches in one or two steps. So the pan is the start pan (from the table 0xA66A) for the first step only and
the far edge afterwards. Explosion 0x66 (mode 5, or 6 by facing, S = 0x12, speeds 0x0156 and 0x1080): a slow drift of
18 units over about 14 steps, then one or two steps to the edge. Coin 0x30 (mode 7, S = 0, speed 0x0380): the limit
equals the start pan, so the pan stops at the first step; the effect of the mode is the twin track.

### 9.4 Mode 7 twin (the end of 0x03C7)

After the primary track is set up, the routine copies the whole 0x40-byte track to the **next voice** `(v+1) & 15`
(voice 15 wraps to 0) if `h >= priority (+32) of the SFX track already on that voice` (so an equal priority replaces
it). The copy plays the **same sequence** from its start (same pointer, active, priority, state, +36 = 8) with its own
pan fields: pan = start pan (clamped to 0x20), limit = max(pan - S, 0), speed = -v (the other direction), fraction 0,
and **start delay +3F = 4**. In the sequencer loop the counter drops by one per step. On the step where it reaches 0
(the 4th) the voice is reset (0x0981), the pan is written, F020 becomes 0xFF and the track is not stepped; so the twin
reads its first events 4 steps (64 ms) after the primary does. While it waits, F020 stays 0, so music on that voice
keeps sounding. If the header also uses the next voice, that voice's own track (set up later
in the voice loop, which counts 0 to 15) overwrites the twin. ADPCM mode 7 does the same to the next slot (slot 2
wraps to 0, delay at +1F).

## 10. Voice ownership, effects over music, fades [O]

- **Mask F020.** In the sequencer loop F020 is 0 at the start of voice v and becomes 0xFF when the SFX track of v is
  active and not delaying. Then the music track of v still advances (events, loops, tempo, duration) but writes
  nothing to the DSP: no key-on (0x0BCB), no volume/pitch (0x12C7), no sample load (0x0D32), no pan (0x0F4D); its
  fields are still updated.
- **SFX start on a voice** (0x0235..0x0241): the voice is reset (volume 0, pitch 0, echo send 0) and the pan is written.
  Music on that voice is silent from that moment.
- **Music start** (0x04CA..0x04E0): a voice that no SFX owns is reset and its pan set to 0x10; an owned voice is left
  alone. A new music code also clears F012, F014, F01A, F01B, F035 (not F030 or F033).
- **SFX end** (`end` command, or special 1 or 2; routine 0x0EBB): +20 = 0, +32 = 0, voice reset. Then the music track of
  the same voice (0x400 bytes up) is looked at: if it is inactive, nothing. Otherwise its sample is loaded again
  (0x0D5E, from +20-1), its pan written (+37), and: if its tie flag (+03 bit 7) is set (a held note) the echo
  registers, the echo send, the sample start and the phase register are rewritten and the volume and pitch are written
  at once (0x12CC), so the held note continues; if not, +33 = 0xFF and the music voice stays silent until its next key-on.
- **Music end** (command 0x17 on a music track): +20 = 0; if the voice is not masked, the voice is reset.
- **Fades.** F033 (0xFFFF at the start) loses F031 per sequencer step and stops at 0; it multiplies the volume of
  **every** track, music and SFX (0x1323 is outside the music only part). F035/F036/F038 is a second fade that
  multiplies music only. F030 stays 0xFF after the fade has ended, so after a fade everything stays at volume 0 until
  0xFF00 (clears F030) or a fade request with speed 0 (cancels at once: the volume jumps back to full). A fade request
  while F030 is set is ignored. D4e s6 says effects are not stopped by the boss fade: they are not stopped, but
  they are silenced by the factor. Whether the clear jingle 0x18 (sent after the fade) is heard needs the MAME trace
  [U, M4-01]: either a 0xFF00 comes in between or the driver port must keep this behaviour.

## 11. ADPCM player (0x04F2, 0x212C..0x2385) [O by reading; not used by this ROM]

No header of the command table is >= 0x80 (the ADPCM type) and the 68000 never sends 0xFF03 or 0xFF04, so this
code never runs in dino. `tables.py` checks the first part. It is documented for completeness; the port can skip it.

- Header (after the type byte, which is 0x80 | priority): byte 1: bit 7 = repeat, bits 0-1 = slot; byte 2 = volume index
  (table 0x23A9); byte 3 = sample number (the table at 0x6A82, read as bank, +2/+3 start, +4/+5 end: a layout that
  differs from the PCM entries, [I] it was meant for a separate table).
- Slot = 0x20 bytes at `FA00 + 0x20*slot` (3 slots): +00 start request (0xFF), +01 repeat flag, +02/03 volume word,
  +04 bank, +05/06 start, +07/08 end, +09/0A length, +10/11 remaining, +12 priority, +16..1E pan fields (section 9.1),
  +1F start delay. A start needs `priority >= +12` of the slot.
- Every sequencer step (0x212C): for each slot with remaining != 0: if +1F != 0 count it down and stop; if +00 != 0 (start):
  clear it, write regs `0xCA+4n` start, `0xCB+4n` end, `0xCC+4n` = 0x8000 | bank, `0xCD+4n` = volume (times F033 if
  F030; then times 4), `0xD6+n` = 1, then the pan (`0x90+n`). Else remaining -= 0x40; if it is still above 0 run the
  pan step (table 0x2211); at 0 or below: repeat = restart (rewrite the registers) else clear the slot and reset the
  voice (0x09A8).
- Special 3 clears all slots, special 4 stops slot `C004` (0x0829).

## 12. Tables [O, all checked by tables.py]

"word" = 16 bit little endian (the Z80 reads them as `ld a,(hl); inc hl; ld h,(hl)`). Exact copies go through
`dino-extract` (M4-02); this section gives what the port needs to know. Formulas marked "about" are descriptions, not
the exact values.

| address | length | read with | content |
| --- | --- | --- | --- |
| 0x0C50 | 7 bytes | (note byte >> 5) - 1 | triplet length units: 2 * 2^k |
| 0x0C57 | 7 bytes | same | normal length units: 3 * 2^k (3 = 1/64 note ... 192 = whole note) |
| 0x0C5E | 7 bytes | same | dotted: 1.5 * normal, but 0 for k = 0 (4.5 is not an integer) and k = 6 (288 > 255) |
| 0x0C65 | 16 bytes | track flags & 0x0F | note base: 12 * ((f & 7) + 2 * (f >> 3)) (octave bank adds 2 octaves) |
| 0x13DE | 108 bytes | n = 0..0x6B | octave of n = n div 12 |
| 0x144A | 9 bytes | octave 0..8 | first n of the octave = 12 * octave |
| 0x1453 | 108 words (0xD8 bytes) | n | base pitch register: entry n = entry (n mod 12) << (n div 12); octave 0 is about 128 * 2^(s/12) (every entry within 1); entry 0x3C = 0x1000 = natural speed |
| 0x152B | 0xC00 bytes = 12 rows of 256 | row s = n - 12*octave, column f = fraction byte | row s = floor(k_s * f / 256) with a small slope k_s (7 to 13, never decreasing with s); the result is shifted left by the octave and added to the base pitch |
| 0x23A9 | 128 words (0x100 bytes) | volume operand (cmd 07), echo send operand (cmd 19), ADPCM volume | monotone, about exponential, 0 to 0x1FFD; the echo send register gets the word * 4 |
| 0x24A9 | 128 words | master volume byte at 0x8002, cmd 1A operand, instrument byte 4 (sustain level) | about linear: entry i = 512 * (i + 1) - 2 within 5 for i >= 1, entry 0 = 0, entry 127 = 0xFFFE |
| 0x25A9 | 64 words (0x80 bytes) | instrument byte 2 (attack index 0..0x3F) | attack rate (0 to 0xFFFF), monotone, floating point like: small steps first, step doubling every few entries |
| 0x2629 | 64 words (0x80 bytes) | instrument bytes 3, 5, 6 (decay, slow decay, release index) | rates (0 to 0xFFFF), same shape as 0x25A9. Index 0x40 occurs once (instrument 16, release) and reads word 0 of 0x26A9, which is 0: that note never releases |
| 0x26A9 | 128 words | cmd 1D operand (LFO speed) | monotone, 0 to 0xFBB0 |
| 0x27A9 | 128 words | cmd 1B operand (vibrato depth) | 8.8 semitones, monotone, up to 0x0BE8 (11.9 semitones); the used operands stay below 0x2E |
| 0x28A9 | 128 words | cmd 1C operand (tremolo depth) | 16 bit fraction, piecewise linear: step 252 for entries 2..63, 508 for 65..95, 1020 for 96..126, 0 for entry 0 |
| 0x2A81 | 4096 words (0x2000 bytes) | `nibble * 256 + byte`, nibble 0..15 | `nibble * byte` (section 13) |
| 0x4A81 | 4096 words (0x2000 bytes) | same | `(nibble * byte) << 4` (section 13) |
| 0x6A81 | 1 byte | | padding (0) |
| 0x6A82 | 124 * 8 bytes | sample number | sample table (section 14) |
| 0x6E62 | 256 * 8 bytes | instrument number | instrument table (section 14) |

The sound 16-bit tables 0x23A9 to 0x28A9 run back to back (0x600 bytes) and end at 0x29A8. Every operand of the
sequences stays inside the domain above (checked over all 243 codes: volume, echo send and LFO speed up to 0x7F, music
volume up to 0x78, vibrato up to 0x2D, tremolo up to 0x78, music pan up to 0x20).

## 13. Multiply routines [O, exactness checked by running the ROM code]

- 0x2A81 and 0x4A81 are **exact products**: word at `2A81 + 2*(nibble*256 + byte)` = nibble * byte, and word at
  `4A81 + 2*(nibble*256 + byte)` = (nibble * byte) * 16, for every nibble 0..15 and byte 0..255.
- 0x29AA: HL = B * C (8 by 8 bit, two table lookups: the low nibble of B from 0x2A81, the high nibble from 0x4A81).
  Used by special command 2 (0x07EF, B = 0x40) and special command 4 (0x0829, B = 0x20) to turn a voice or slot number
  into a byte offset of the track or slot.
- 0x29CB: HL = (HL * DE) >> 16 (the high word of the unsigned 32-bit product, **floor**, no rounding). It also stores HL
  at F000/F001 and DE at F002/F003 (low byte first). Checked on the real routine for 65,266 inputs (all powers of two
  pairs, edge values, random) with no difference. The port uses `((a as u32 * b as u32) >> 16) as u16`.
- The tempo offset and the music master volume use the same routine: tempo offset = (C004 * 0x369) >> 8, music
  volume F018 = F016 * 0xFFFE >> 16 = F016 minus at most 1.

## 14. Command table, samples, instruments [O]

- **Command table** at linear 0x8000: BE word N = 0x106 (262 entries at 0x8006, 4 bytes each: bank, hi, lo, 0).
  243 live: 39 music (header 0, codes 0x00-0x26), 204 SFX, 19 empty (0x27-0x2B and 0x92-0x9F). Priorities in use: 3, 4, 5, 6,
  7 and 0x0A (coin). The byte at 0x8002 is the music master volume index (a single byte, 0x2C in this ROM); bytes
  0x8003-0x8005 are 0.
- **Voices**: SFX use only voices 8 to 15 (194 of the 204 use one voice, the others up to 5). Music uses voices 0 to 15
  (8 to 16 voices per song). So an effect only ever masks the high voices of a song.
- **Twin headers use other voices** (answers D4e open item 5 and the "two players cut each other off" worry): the bank
  twins of one sound use different voices: the hero voice codes use voice 10 (bank 0), 9 (bank 1) and 8 (bank 2); the
  toggle pairs 0x65/0xE4, 0x7B/0xFC and 0x38/0xA8 use voices 12 and 13, 0x6B/0xF1 use 14 and 15. Equal priority never makes
  two of these cut each other off, only a repeat of the same code does.
- **Sample table** at 0x6A82: **124 entries** of 8 bytes (0x6A82 + 0x3E0 = 0x6E62, where the instrument table starts):
  bank, start (LE), loop start (LE), end (LE), root key. PCM address = bank * 0x10000 + offset in the concatenation of
  `cd-q1.1k`, `cd-q2.2k`, `cd-q3.3k`, `cd-q4.4k` (512 KB each, in that order, banks 0..31, 2 MB; the order is the
  load order of MAME, section 16). Every entry has start <= loop start <= end; the highest end is 0x1FF9AF; the sum of
  (end - start) is 1,963,673 bytes (equal to the asset catalog). 121 samples are used by a sequence; the others (28, 29,
  123) are not.
- **Instrument table** at 0x6E62: 256 entries of 8 bytes, but only entries 0..129 are real (130 entries); entries
  130..255 are a filler pattern (sample number = entry number, fixed envelope bytes) and nothing refers to them. After
  0x7662 the area up to 0x7FFF is filler. The highest instrument used by a sequence is 129; 126 distinct instruments
  are used. Instrument bytes: 0 sample number, 1 fine tune, 2 attack rate index (0x25A9), 3 decay rate index (0x2629),
  4 sustain level index (0x24A9), 5 slow decay rate index (0x2629), 6 release rate index (0x2629), 7 not used.
  Command 08 stores `byte0 + 1` as the track's +20 (so +20 != 0 means active).

## 15. The pitch clamp at 0x138F..0x13A1 [O]

The pitch register routine computes `a = (0x3C - root key + (H - 1)) & 0xFF`, where H is the high byte of pitch +
vibrato (+ F01B). Then `cp 0x6C; jp c,ok; ld a,0x6B; jp m,ok; xor a`. The `jp m` uses the flags of the compare, so for
a >= 0x6C: if `(a - 0x6C)` has bit 7 set (a = 0xEC..0xFF, a negative n) the result is 0x6B, otherwise (a = 0x6C..0xEB,
too high) the result is **0**. So too high notes play at the lowest pitch (table entry 0, 0x0080) and too low notes at
the highest. It looks inverted, and it is the ROM behaviour: keep it. **No note in this ROM reaches it**: `tables.py`
walks all tracks (both ways of every loop, every music transpose the code sets, the vibrato depth the track sets) and
finds `a` between 0x02 and 0x61 for all 29,543 notes, with 0 notes over an edge. The port may implement it as is
(three lines) at no risk.

## 16. MAME checks (mame_irq.py, 2026-10-13) [M]

One headless process (`-video none -sound none`, SDL dummy drivers), a Lua frame hook reading the Z80 RAM:

- **IRQ rate**: 249.997 Hz (66564 interrupts in 266.26 emulated seconds), main loop ticks equal the interrupts, the sequencer
  therefore runs at 62.499 Hz. The `-sound none` option does not stop the Z80 or the DSP handshake.
- **Attract mute**: phase 1 (frames 0 to 5999, both flags clear): 8 requests reached the Z80, all 0xFF00 or 0xFF0B, no
  other code. Phase 2 (bit 5 of `G.$777C` set at frame 6000): 215 requests, 207 of them normal codes. Phase 3 (bit 5
  cleared and `G.$4CC` = 1 at frame 11000): 148 requests, 142 normal codes. So both flags work as section 3 says.
- **ROM order and CRCs**: `mame -listxml dino` lists `cd-q1.1k` (CRC 60927775) at offset 0, `cd-q2.2k` (770f4c47) at
  0x80000, `cd-q3.3k` (2f273ffc) at 0x100000, `cd-q4.4k` (2c67821d) at 0x180000, region `qsound`, and `cd_q.5k`
  (605fdb0b) as `audiocpu`. The CRCs of the files in dino.zip are equal.

## 17. Still unknown, and why

- The exact pan law, the FIR filters and the echo taps: inside the DSP internal ROM, which is not in dino.zip (section 8).
  The port uses an equal power pan and a plain echo. The register writes can still be matched exactly (M4-09).
- Which side pan 0 is: [I] the left (section 8). A listening test settles it.
- Whether the stage clear jingle is silenced by the persisting fade (section 10): needs a MAME trace of the 68000 flow
  (M4-01).
- The exact formulas of the floating point like tables 0x23A9, 0x25A9, 0x2629, 0x26A9, 0x27A9: they are monotone and
  smooth but no closed form was found; exact copies are needed (M4-02). The linear tables 0x1453, 0x152B, 0x24A9 and
  0x28A9 are described above.
- The meaning of the 19 empty command slots and of the 4th byte of the command entries (always 0): not used by the driver.
