# Sound system (QSound Z80 driver) - SPEC (work in progress)

Tags: **[O]** observed in code/data, **[I]** inferred, **[U]** unknown / not checked yet.
Z80 addresses are 16-bit CPU addresses. "Linear" addresses are 24-bit offsets into `cd_q.5k`.

Tools in this folder: `kabuki.py` (writes `re/out/z80_op.bin`, `re/out/z80_data.bin`),
`z80dis.py` (dual-image Z80 disassembler; `--range a b` for linear listings),
`walkall.py` (recursive listing with all known jump tables; output `re/out/z80_full.txt`).

## 1. Kabuki decryption [O]

- Only bytes 0x0000-0x7FFF of `cd_q.5k` are encrypted. 0x8000-0x1FFFF (music/sfx data) is plain.
- Algorithm = MAME `kabuki_decode`. Keys for dino: swap_key1 `0x76543210`, swap_key2 `0x24601357`,
  addr_key `0x4343`, xor_key `0x43`.
- Opcode image: select = `addr + 0x4343`. Data image: select = `(addr ^ 0x1FC0) + 0x4343 + 1`.
- Opcode fetches (M1: first opcode byte, and the second byte after CB/DD/ED/FD) use the opcode
  image. Operands, displacements, the last byte of `DD CB d op`, and all data reads use the data image.
- Verified: reset code decodes to `di; im 1; ld sp,$ffff; ld hl,$f000; ld de,$f001; ld bc,$0fff;
  ld (hl),0; ldir; ...`, and the data image shows the text " version 1.00   1992 / JUNE" at 0x1A.

## 2. Hardware map seen by the Z80 [O unless tagged]

- 0x0000-0x7FFF fixed ROM, 0x8000-0xBFFF banked window, 0xC000-0xCFFF shared RAM with the 68000
  (68000 address `0xF18000 + 2*n + 1` = Z80 `0xC000 + n`), 0xF000-0xFFFF Z80 work RAM.
- 0xD000/0xD001 = QSound data high/low, 0xD002 = register number (write commits), 0xD003 = bank
  register (driver always writes `0x80 | bank`), 0xD007 read: bit 7 = DSP ready.
- Write helper 0x0A13: wait D007 bit7, write E to D001, D to D000, A to D002 (reg A = value DE).
- Linear address -> bank: the driver's read routine (0x09BC, also inlined at 0x0A8B and 0x0FC7)
  takes a 24-bit pointer B:HL. If `addr < 0xC000` it reads directly (fixed ROM or bank 0); else
  bank = `(addr >> 14) - 2`, CPU address `0x8000 | (addr & 0x3FFF)`. So **linear address = file
  offset in cd_q.5k**. The pointer is incremented with carry into B.
- IRQ (IM 1, vector 0x0038): MAME uses 250 Hz [I: from MAME driver; not visible in the ROM].

## 3. 68000 side [O]

- Enqueue 0xA63C: entry = 16 bytes at `$3A(a5) + $34(a5)`: word d0 (code), long d1, long d2
  (6 bytes unused). Write index `$34`, read index `$36`, both step 0x10 and wrap at 0x400
  (64 entries). If the ring is full the entry is overwritten but not committed.
- Flush in vblank (0x610), one entry per frame, only when `$36 != $34` and `0xF1801F == 0xFF`:
  - Normal code (high byte of d0 != 0xFF): C000 = d0 high, C001 = d0 low, C006 = d1 bits 23-16,
    C007 = d1 bits 7-0, C008 = d1 bits 15-8, C009..C00C = d2 bytes (high to low), C00F = 0.
  - Attract mode mute: if `$4CC(a5) == 0` and code != 0x0030 and `($777C(a5) & 0x20) == 0`, the
    entry is dropped (demo sound DIP) [I: meaning of $4CC/$777C].
  - Special code (d0 high byte 0xFF): C000 = 0xFF, C001 = sub-command, C003 = d1 bits 31-24,
    C004 = d1 bits 23-16, C00F = 0.
- Init 0xA5FA: write 0x88 to 0xF19FFB (Z80 CFFD: "commands enabled"), clear the ring, send 0xFF0B.
- 0xA716: 0xF19FFD (Z80 CFFE) = 0xFF if `$777A(a5)` bit 7 set (stereo) else 0 (mono).
- 0xAAA7A checks 0xF19FFF == 0x77 (Z80 boot signature). Z80 main waits for CFFF == 0xFF before
  starting [U: where the 68000 writes 0xFF there].
- Helpers: 0xA5D2 sends 0xFF00 (stop all), 0xA5DE 0xFF05 (stop music), 0xA5EA 0xFF07 with
  d1 = 0x04440000 (fade speed 0x0444).
- Positional SFX wrappers 0x96xx-0x9F60 (198 calls to 0xA63C in total): d1 = `mode << 16 | x << 8 | pan`,
  where pan = table at 0xA66A (48 bytes) indexed by `(objX - camX) / 8` clamped to 0..0x17F, and
  mode = pan-motion mode (5/6 chosen by facing, 7, 3/4) and d2 = pan speed words. Some wrappers
  alternate between two codes with a toggle bit in `$43A(a5)` (e.g. 0x65/0xE4, 0x7B/0xFC, 0x72/0xF8).
  [U: full code table per caller; not done yet.]

## 4. Z80 main program [O]

- Reset: clear F000-FFFF, write 0x77 to CFFF, toggle bank register 4 IRQ ticks each, wait CFFF == 0xFF,
  init DSP (0x0912), F016 = table 0x24A9[byte at linear 0x8002] (music master volume).
- IRQ 0x0038: F004++. If CFFD == 0x88 and C00F != 0xFF: copy C000-C00F to `F100 + F007` (16-entry
  ring, F007 += 0x10), set C00F = 0xFF (ready). Always F040 = CFFE (stereo flag).
- Main loop 0x00BE: wait until F004 != F005, F005++; call 0x017D (handle at most one queued command
  per IRQ tick); every 4th tick (`F005 & 3 == 0`, so 62.5 Hz with a 250 Hz IRQ) run the sequencer:
  - For voice v = 0..15: F00D = v, F00E = v*8, F006 = 0, F020 = 0.
    SFX track at `F200 + 0x40*v`: if active (+0x20 != 0): if delay +0x3F != 0, count it down and
    when it hits 0 reset the voice (0x0981) and set pan; else tempo F010 = word +0x30/+0x31 and
    step the track (0x0A28). Then F020 = 0xFF (voice is owned by the SFX).
    Music track at `F600 + 0x40*v`: if active: F006 = its +0x20, tempo F010 = F012 + F014, step it.
    While F020 = 0xFF the music track runs but never writes the DSP (it is masked).
  - Then 0x212C (ADPCM slots), then music fade F033 -= F031 (if F030), fade 2 F038 -= F036 (if F035),
    both clamp at 0.
- DSP init 0x0912: reg 0xDE=0, 0xDF=0x2E, 0xE0=0, 0xE1=0x30, 0xE4..0xE7=0x3FFF, 0xE2=0xFFFF,
  0xDA=0x0DB2, 0xDC=0x0E11, 0xE3=0x0039; reset all 16 voices (0x0981: reg 0xBA+v=0, v*8+6=0, v*8+2=0)
  and the 3 ADPCM voices (0x09A8: reg 0xCD+4v=0, 0xD6+v=1).
- Pan write 0x09E7/0x09FE: value e (0..0x20) -> reg `0x80+v` = `0x110 + e`; mono (F040 == 0)
  forces e = 0x10. If F041 != 0 the pan is also copied to C005 (debug).

## 5. Command handling 0x017D [O]

- Code 0xFFxx -> special command table 0x0796 (sub < 12):
  0 stop everything (clear F200-FA5F, F030; reset voices + ADPCM), 1 stop all SFX tracks (0x0EBB each),
  2 stop SFX track `C004` (voice), 3 clear ADPCM slots, 4 stop ADPCM slot `C004`, 5 stop music,
  6 music fade: speed = C003:C004 (0 cancels; else F030=FF, F031=speed, F033=0xFFFF),
  7 second fade (F035/F036/F038, same rules), 8 tempo offset F014 = (C004*0x369)>>8 (negated if C003==0xFF),
  9 pitch offset F01B = C004<<4 (negated if C003==0xFF), 10 F041=FF, 11 F041=0.
- Other codes: bank 0; count N = BE word at linear 0x8000 (0x106); code = code mod N; entry at
  `0x8006 + 4*code` = [bank, hi, lo, unused] -> 24-bit header pointer (0 = no sound).
  Header byte h: `h == 0` music, `1..0x7F` SFX with priority h, `>= 0x80` ADPCM one-shot.
  - SFX/music: then 16 BE words, one per voice: offset of that voice's track from the header (0 = unused).
  - SFX: for each used voice, start only if `h >= priority of the current SFX on that voice (+0x32)`.
    Clear 0x40 bytes, ptr = header + offset, +0x32 = h, +0x20 = 0xFF, +0x17 = 1, +0x25 = 5, pan setup
    from C006 (0x0251, table 0x0269), reset voice, write pan.
  - Music: F012/F014/F01A/F01B/F035 = 0, F018 = (F016*0xFFFE)>>16; each voice: offset 0 stops that
    music track; else clear 0x30 bytes, set ptr, +0x20 = 0xFF, +0x17 = 1, +0x25 = 5, pan 0x37 = 0x10;
    if no SFX owns the voice, reset it and set pan.
  - ADPCM (0x04F2): slot = byte & 3 at `FA00 + 0x20*slot`, sample from table 0x6A82, volume from 0x23A9.
- Code table in this ROM [O]: 0x00-0x26 music (header 0), 0x2C-0x91 and 0xA0-0x105 SFX
  (priorities 3..0x0A), no ADPCM headers. Exact meaning of each song (stage) [U].

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

## 7. Sequence format (0x0A28 / 0x0A8B) [O]

Per 62.5 Hz tick: gate counter -= tempo (on <= 0: release, envelope state 4); duration -= tempo; if
it reaches <= 0 (or high byte >= 0xC1) read events, else run pan motion (table 0x101D), LFO and
envelope (table 0x11FF). Events:
- `0x20..0xFF` note: length index = (b >> 5) - 1 into table 0x0C57 (normal), 0x0C50 (triplet),
  0x0C5E (dotted, one note); duration += len << 8. Low 5 bits: 0 = rest, else note =
  (b & 0x1F) + table 0x0C65[flags & 0x0F]. Pitch = note + transpose (+ F01A music) in 8.8,
  plus detune+fine, clamp < 0x62. Then portamento, key-on (unless tie): reg v*8+2=0, 0x93=0x27FF,
  0xD9=0x07FF, 0xBA+v = table23A9[+0F]<<2, v*8+1 = sample start, v*8+3 = 0x8000; gate = dur*gate/256.
- Commands 0x00..0x1F (table 0x0C84): 00 toggle triplet, 01 toggle tie, 02 dot next, 03 toggle
  octave bank, 04 n flags=(flags&0x97)|n, 05 hh ll tempo (SFX: own, music: F012), 06 n gate,
  07 n volume, 08 n instrument (table 0x6E62, 8 bytes: sample, fine, env[5], ...), 09 n octave,
  0A n music transpose, 0B n transpose, 0C n detune, 0D n portamento, 0E..11 n hh ll loop k
  (repeat n times, relative jump), 12..15 f hh ll break on last loop pass, 16 hh ll jump
  (signed BE offset from after the operand), 17 end of track, 18 n pan (music), 19 n echo send,
  1A n music volume (F018 = F016*table24A9[n]>>16), 1B n vibrato depth (0x27A9), 1C n tremolo
  depth (0x28A9), 1D n LFO speed (0x26A9), 1E n LFO flags, 1F unused (points into data).
- Sample table 0x6A82 (8 bytes, data image): bank, start(LE), loop start(LE), end(LE), root key.
  Load: v*8+2=0, v*8+1=start, v*8+5=end, v*8+4=end-loop, reg ((v-1)*8)&0x78 = 0x8000|bank.
- Envelope: state 0 attack (+rate until overflow -> 0xFFFF), 1 decay (to sustain), 3 slow decay,
  4 release, 5 off; rates from table 0x2629[env byte], attack rate from 0x25A9[env[0]].
- Volume reg v*8+6 = tremolo * env * (music: F018, F038) * F033 fade * channel volume (each step is
  `(a*b)>>16`, routine 0x29CB). Pitch reg v*8+2 = table 0x1453[n] + (table 0x152B[semi*256+frac]
  << octave 0x13DE[n]), n = pitch - 1 + 0x3C - root key, clamped (see 0x138F; odd clamp kept).

## 8. QSound DSP model [I: from MAME qsound_hle docs, not the ROM]

Rate = 60 MHz/2/1248 = 24038 Hz. PCM: 8-bit signed samples at `(bank<<16)|addr`, rate 4.12,
loop subtracts loop length once at end, volume 0x4000 = unity (`out = vol*s>>14`). Pan and FIR
taps come from the DSP internal ROM (not in dino.zip), so the port needs its own pan law and filter.
ADPCM: 3 voices, 4-bit, ~8 kHz. Echo: feedback reg 0x93, length via 0xD9.
