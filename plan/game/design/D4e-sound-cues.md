# D4e: sound cues

Written 2026-10-12 by design task D4e. Input to task T4.5 in `plan/game/NEW-PLAN.md` (cue wiring: music per area and boss, front end, clock beep, voices). Depends on D4e and milestone M3. The driver that actually plays audio is milestone M4, not this document. This file only describes. No code was changed. MAME was not run.

**Source tags.** `R:` a repo file, a 68000 address, or a decoded table read from `re/out/dino_68k.bin` (file offset equals the 68000 address). `G.$xxxx` is RAM `0xFF8000 + xxxx`. `G.-$xxxx` is RAM `0xFF8000 - xxxx`. `W:` a web note. This task did not run MAME, so it adds no `M:` tag. Where an older design doc measured a frame in MAME, this file cites that doc. ROM code wins over the web. `re/tools/d` prints data tables as if they were instructions, so every table below was read as bytes, not from the disassembly text.

Port words: REC means a Rust function already calls `sound()` with that code. MISS means it does not. FLOW means only `flow_area_music` or the episode 1 start can send it. Hearing any of them is still MISS: `Game::sound` only stores the id (`dino-rs/crates/dino-game/src/game/snd.rs`).

**Do not trust a catalog label "code at" for music 0x00 inside 0x95CE to 0xA57E.** Those helpers do `moveq #0, d0` and then replace d0 with a table byte. The dump's caller line for code 0 lists 0x99DA, 0x9A22, 0x9A70, 0x9AD2, 0x9B42, 0x9C58, 0x9CC8, 0x9D2C, 0x9DF0, 0x9E54, 0x9FA6, 0xA490 and 0xA57E for that reason. 0xA57E is the `bra` at the end of the area music routine itself. 0xA5C4 is the same kind of `bra` on the lock path, not a use of music 0x08. R: `re/out/assets/sound/sound_codes.txt`, disassembly of 0xA55A.

## 0. How a request is sent

Enqueue `0xA63C`. One ring entry is 16 bytes at `G.$3A` plus the write index `G.$34` (RAM `0xFF803A`, index `0xFF8034`): code word d0, long d1, long d2. The write and read indexes step by 0x10 and wrap at 0x400 (64 entries). A full ring overwrites the slot and does not commit it. R: `re/specs/sound/SPEC.md` section 3.

The vblank flush `0x610` sends one entry per frame, and only when the read index differs from the write index and the Z80 status byte `0xF1801F` is 0xFF. A normal code (high byte of d0 is not 0xFF) is copied as C000 = high byte, C001 = low byte, C006 = d1 bits 23 to 16, C007 = d1 bits 7 to 0, C008 = d1 bits 15 to 8, C009 to C00C = the four bytes of d2, C00F = 0. R: SPEC section 3.

d1 is not "pan, then world X". After the flush, C006 is the pan-motion mode, C007 is the pan, and C008 is a fixed middle byte that the helper wrote (for example `move.w #$2000, d1` leaves 0x20, `move.w #$1200, d1` leaves 0x12). The pan byte then replaces the low byte of d1. R: SPEC section 3, and the helpers disassembled in section 4.

**Pan curve.** Table `0xA66A`, 48 bytes. The index is `(object X - camera X) / 8`, with the difference clamped to 0..0x17F and the low 3 bits cleared before the divide. Index 0 and 1 are 0x00. Index 24 and 25 are 0x10 (center). Index 46 and 47 are 0x20. The curve steps up and skips the value 0x1A. The Z80 writes pan 0..0x20 to the DSP. Mono (stereo flag off) forces pan 0x10. R: bytes at 0xA66A, SPEC sections 3 and 4.

**Attract filter**, in the flush, before a normal code is copied. If `G.$4CC` is 0 (not in a game) and the code is not 0x0030 and `(G.$777C & 0x20) == 0`, the entry is dropped. 0x0030 is the coin sound and always passes. An 0xFFxx command takes the special path and also passes. Factory demo sound is off, so title, profiles, the demo and the movie queue their codes and the flush throws them away. Only the coin sound is heard. R: SPEC section 3, `plan/game/design/D4d-front-end-and-story.md` section 3.11. The port does not filter.

**Special commands** (high byte 0xFF). C000 = 0xFF, C001 = the subcommand, C003 and C004 = the high bytes of d1. The Z80 handles subcommands 0 to 11. R: SPEC section 5.

| Sub | Helper | What it does | Who calls the helper |
| --- | --- | --- | --- |
| 0xFF00 | `0xA5D2` | Stop everything (music, effects, ADPCM) | 19 absolute calls, listed in section 6 |
| 0xFF05 | `0xA5DE` | Stop music only | No `jsr` and no `jmp` anywhere in the ROM |
| 0xFF07 | `0xA5EA` | Music fade. d1 = 0x04440000, so the fade speed is 0x0444. Speed 0 would cancel a fade | 13 calls, section 1.3 |
| 0xFF0B | init `0xA5FA` | Commands enabled. The init writes 0x88 to `0xF19FFB`, clears the ring, then sends 0xFF0B with d1 = 0 and d2 = 0 (`0xA626`) | Boot |

R: disassembly of 0xA5FA to 0xA63A, SPEC section 5. Other subcommands (stop one effect, tempo, pitch, debug pan) are in SPEC section 5. This task found `jsr 0xA63C` at `0xA73CE` in that boot block and did not re-read its immediate. Stereo versus mono is `0xA716`: Z80 flag from `G.$777A` bit 7. R: SPEC section 3.

**Priority.** An effect starts on a voice only if its header priority is greater than or equal to the effect already on that voice. The same priority replaces the current effect. While an effect owns a voice, music still advances on that voice but does not write the DSP. A new music code clears the music tempo offset, the pitch offset and the second fade. R: SPEC sections 4 and 5. There are no ADPCM headers in this ROM. The command table has 0x106 entries: 39 music (0x00 to 0x26), 204 effects, 19 empty (0x27 to 0x2B and 0x92 to 0x9F). R: `plan/game/design/asset-catalog.md` row N8, `sound_codes.txt`.

**Toggle bytes.** Many helpers flip one bit and pick one of two codes. `bchg` tests the old bit, so the first request after a zero byte takes the second code. The bits are global, not per object.

| RAM | Used by |
| --- | --- |
| `G.$43A` (`0xFF843A`) | Player voices and a few shared helpers. Bit 0 jab, bit 1 routine `0x96CE`, bit 2 blocked hit `0x9732`, bit 3 thud `0x9796`, bit 4 non-player splash, bit 5 wade and footstep (they share it), bit 6 non-player landing, bit 7 thrown blade `0x9D44` |
| `G.$43B` (`0xFF843B`) | Items. Bit 0 break `0x9E60`, bit 1 container thud `0x9FB6`, bit 2 `0x9FF8`, bit 5 boomerang `0xA21E`, bit 6 barrel kind `0xA3C6`, bit 7 `0xA404` |
| `G.$43E` (`0xFF843E`) | Biker only, in the helpers found here. Bit 0 fall `0xA164`, bit 1 ride `0xA128` |
| `G.$43C` (`0xFF843C`) | Hit sounds. Read at `0x9638` into a register, flipped there, never written back |

R: `snd.rs`, `item/shared.rs`, disassembly of 0x962A, 0x96CE, 0x9732, 0xA128, 0xA164, 0xA21E, 0xA3C6. The only `(a5)` use of `0x43C` found in the program is that read. If nothing else sets the word, every hit plays the first code of its pair. The port's `snd_hit` also never sets the word, so it always sends the first code. R: search of the ROM image for the displacement.

## 1. Music

Music codes are 0x00 to 0x26. A music start replaces the music state. It does not stop effects. R: SPEC section 5. Lengths below are the sequence lengths in `sound_codes.txt` (the sequencer step is 62.5 Hz, SPEC section 4). They are not the album timings.

### 1.1 Area table `0xA582` (32 bytes, stage * 4 + area)

Routine `0xA55A`. If `G.$7B4` is not 2: index = stage byte `G.$4D9` times 4, plus area byte `G.$786`. d1 = 0, d2 = 0. R: disassembly of 0xA55A. Bytes:

| Episode (stage byte) | Area 0 | Area 1 | Area 2 | Area 3 |
| --- | --- | --- | --- | --- |
| 1 (0) | 0x02 | 0x03 | 0x04 | 0x00 |
| 2 (1) | 0x05 | 0x06 | 0x03 | 0x00 |
| 3 (2) | 0x07 | 0x08 | 0x00 | 0x00 |
| 4 (3) | 0x09 | 0x0A | 0x0B | 0x00 |
| 5 (4) | 0x17 | 0x0A | 0x00 | 0x00 |
| 6 (5) | 0x03 | 0x05 | 0x06 | 0x00 |
| 7 (6) | 0x04 | 0x08 | 0x0B | 0x06 |
| 8 (7) | 0x0C | 0x09 | 0x0D | 0x22 |

A 0x00 byte is music code 0x00 (the same code as the select tune), not silence. The stage-start path in section 1.3 does not index those slots. Episode 1 does not use this table at the start of the stage: it sends code 0x01 instead.

### 1.2 Lock table `0xA5C8` (10 bytes)

`0xA55A` reads this when `G.$7B4 == 2` (stage end / boss lock). Index is the stage byte. If the stage byte is 7 and the object pointed to by `G.-$70BC` (RAM `0xFF0F44`) has `+$7D == 0xFE`, the index is forced to 8. d1 = 0, d2 = 0. Byte 9 is 0xFF and is not used as an index. R: disassembly of 0xA5A2.

| Index | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Episode | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | stage 7 and `+$7D == 0xFE` | unused |
| Code | 0x0E | 0x0F | 0x10 | 0x11 | 0x0E | 0x0F | 0x11 | 0x12 | 0x0F | 0xFF |

This is not the only way a boss theme starts. Each boss also queues its theme from its own init (section 1.5). `G.$7B4` is cleared at stage start (`0xACAC`) before the stop and the music call. R: disassembly cited by `dino-rs/crates/dino-game/src/game/flow.rs` `flow_stage_start`.

### 1.3 Who starts, stops and fades music

**Stage start** `0xAC4E` (port `flow_stage_start`). Clears `G.$7B4`, calls `0xA5D2` (stop all), then: if the stage byte is 0, `moveq #1` and jump to `0xA63C` with d1 = 0 and d2 = 0 (`0xACCA`). Otherwise jump to `0xA55A` (`0xACBE`). R: bytes at 0xACCA (`70 01 72 00 74 00` then `jmp 0xA63C`). So episode 1 opens on code 0x01 (54.46 s, loops, samples 3 and 16), and the area table takes over when the area changes.

**Area load.** `0xACD0` calls `0xAA0E`, then `0xACD4` calls `0xA55A`. R: absolute call list. The port's area-end hook sends 0xFF07 and then relies on a later `flow_area_music`. R: `flow.rs` around the 0xFF07 call.

**Class 96** (`0x9239A`) does not call `0xA55A` and does not look at `G.$7B4`. State 0 (`0x92396`) advances the state, fades (`jsr 0xA5EA`), stores 0x3C in `+$80`, then falls into the wait. The wait (`0x923A6`, state 2) decrements `+$80` and returns while it is not 0. On the frame it hits 0 the routine reads `0xA582` (stage * 4 + area, no lock check) and queues that byte with d1 = 0 and d2 = 0 (`0x923D2`). The fade frame already consumes one count (0x3C becomes 0x3B), so the tune is queued 59 frames after the fade frame. R: disassembly of 0x92382 to 0x923D8, state table at 0x9238E. The port starts the area tune at once.

**Other callers of `0xA55A`.** `jsr` at `0xEF62` after `0xEF56` has set mode byte `G.$4D0` to 0xC (continue return, and the demo, which stays `G.$4CC == 0` so the filter drops it). `jmp` at `0x9A0C8` after a mode setup. R: absolute call list. Four callers total.

**Fade `0xA5EA` (0xFF07), 13 `jsr` sites.** `0x1DC38`, `0x20C48`, `0x20DE2`, `0x227AE`, `0x2294E` (area-end camera, D4c mode 1), `0x38D44`, `0x43884` (Vice), `0x4C8D2` (Slice), `0x5036E`, `0x55826`, `0x5A38A`, `0x819FC` (Slisaurs), `0x9239A` (class 96). R: absolute call list, boss names from `D3c-bosses-1.md` and `D3d-bosses-2.md`. `flow_boss_defeated` does not send music. The bosses call the fade themselves.

**Stop all `0xA5D2` (0xFF00), 18 `jsr` and one `jmp`.** `0xA72A`, `0xAC2C` (game over), `0xACB0` (stage start), `0xEA00` (continue setup), `0xEEE0`, `0x15194` and `0x15204` (ranking), `0x8A60E` and `0x8B55C` (ending), `0x97CA4`, `0x97CC0`, `0x97CE6`, `0x97D02`, `0x97D42`, `0x97D84` (each attract state), `0x9B36C`, `0x9D584`, `0x9D640`, and `jmp 0xA7B50`. R: absolute call list.

There is no helper call that stops music without stopping effects. `0xA5DE` (0xFF05) has no `jsr` and no `jmp`. The port does not send 0xFF05 or 0xFF0B.

### 1.4 Front end, clear, time over

Frame timings and screen order are D4d. This list is the code and what the sequence is. "Loops" means the sequence header loops.

| Code | Length | Where it is queued | Port |
| --- | --- | --- | --- |
| 0x00 | 14.98 s, loops, 14 voices, samples 4, 9, 10, 12, 13, 15, 18 to 20, 23, 26, 27, 30 | Select music. D4d: queued at select setup with d1 = 0xFFFF. Real `moveq #0` sites that are not the false labels: `0xAE12` and `0x1DC2E` (d1 = 0, d2 = 0). Also hit id 0, whose two codes are both 0x00 | MISS (select). FLOW if a 0x00 area slot is indexed |
| 0x01 | 54.46 s, loops, samples 3, 16 | Episode 1 stage start, `0xACCA`, d1 = 0, d2 = 0 | REC `flow_stage_start` |
| 0x13 | 14.70 s, loops | Profile card (`sound_codes.txt` enqueue `0x9A964`) and story panels of episodes 2 and 6 (`0xD8EA` is `moveq #$13`, `0xDDCE`) | MISS |
| 0x14 | 7.18 s, no loop | Story panels of episodes 3 and 7 (`0xD980`, `0xDE6E`) | MISS |
| 0x15 | 19.48 s, loops | Story panels of episodes 4 and 8 (`0xDAB6`, `0xDF20`) | MISS |
| 0x16 | 14.15 s, loops | Story panel of episode 5 only (`0xDBB4`) | MISS |
| 0x18 | 6.44 s, no loop, 12 voices | Victory pose. Script action 0x04, `moveq #$18` at `0x1E24C`. D4c: player state 0x18 | MISS |
| 0x19 | 107.55 s, no loop, samples include 35 and 120 | Ending scene, desert. `0x8A618` is `move.w #$19` right after a call to `0xA5D2`. D4d: t = 975 | MISS |
| 0x1A | 240.65 s, loops, same samples and length as 0x02 | Ending, when the car arrives. `0x8A7A4` is `move.w #$1A`. D4d: t = 3956 | MISS |
| 0x1B | 31.13 s, loops | Continue music. `0xEB20` is `moveq #$1B`, after `0xEA00` has stopped all sound | MISS |
| 0x1C | 25.63 s, loops | Continue, when the number is 9. `0xF47C`: `cmpi.b #9, d5`, then `moveq #$1C` | MISS |
| 0x1D | 6.12 s, no loop | Game over jingle. `0xEDB4` is `moveq #$1D`. D4d: about 191 frames after the countdown hits 0, unless name entry runs | MISS |
| 0x1E | 3.89 s, no loop | No sender found. See section 7 | MISS |
| 0x1F | 29.40 s, loops | Name entry. `0xEDFC` is `moveq #$1F`. D4d: queued instead of 0x1D when the score qualifies, and again after the ending | MISS |
| 0x24 | 29.96 s, loops | Attract story slides, start. `0x988E2` is `moveq #$24` | MISS |
| 0x25 | 6.06 s, no loop | Fifth story slide. `0x98AF0` is `moveq #$25`. Not the stage clear | MISS |
| 0x26 | 5.55 s, sample 5 only, no loop | No sound sender. Class 42's `moveq #$26` at `0x83ABA` jumps to `0x94640`, which stores d0 as the variant of a new object (class word 0x54). It is not `0xA63C` | MISS |

R: the immediates above (this task), `sound_codes.txt` for length and samples, D4d sections 3, 5, 6, 7, 8 and 9 for the screen, D4c for action 0x04. Story panel map, episodes 2 to 8: 0x13, 0x14, 0x15, 0x16, 0x13, 0x14, 0x15. Episode 1 has no story panel. R: D4d section 9.

Title "music" is not music. It is effect 0x88 (0.91 s, sample 114, priority 3). Ranking "music" is effect 0x80 then the loop 0x70. Section 4.

**Time over.** `0x96450` allocates two objects of class 0x70 (variants 0 and 1). D2b calls this the TIME OVER words. It does not call `0xA63C`. HP is cleared later by `0x1046A` (D2b). No music code was found on that path. 0x1E is the only short music code with no sender, and 3.89 s is close to the rip's "Time over 0:04", but that is not a reason to assign it.

**Stage clear.** Boss death chains call `0xA5EA` (fade). The victory pose then starts 0x18, which replaces the music state. The port fades on the area-end hook and does not send 0x18.

### 1.5 Boss themes

The lock table and the boss init agree, with one split on the last boss.

| Code | Length | Lock table | Direct send | Port |
| --- | --- | --- | --- | --- |
| 0x0E | 82.80 s, loops | Episodes 1 and 5 | Vice (class A18) at the end of the intro, about `0x42856`. Morgan at `0x517D6`. Vice also sends 0x20 (9.23 s, loops) at talk start | FLOW if `G.$7B4 == 2`. Boss init MISS |
| 0x0F | 88.64 s, loops | Episodes 2 and 6, and stage 7 index 8 | Tyrog (class 4), Butcher (class 61) at init, Fessenden class 94 at `0x687A8`. Butcher also sends 0x21 (7.24 s) from the starter and 0x5F every 32 frames while he chops | FLOW, except the port never selects index 8 (it keeps stage 7, code 0x12). Boss init MISS |
| 0x10 | 112.74 s, loops | Episode 3 | Hogg, when the player passes X 0x2D00. Also bomb code 0x66 | FLOW. Boss init MISS |
| 0x11 | 166.39 s, loops | Episodes 4 and 7 | Slice after 0x23 (27.98 s, loops). Slisaur starter `0x8199E` (`move` site in the `0xA63C` list) after flag `G.$4FC` clears. Fade at `0x819FC` | FLOW. Boss init MISS |
| 0x12 | 94.82 s, loops, samples include 99 and 105 | Episode 8 | Fessenden class 84 at `0x61882` | FLOW. Boss init MISS |
| 0x22 | 35.19 s, no loop, samples 23, 27, 34 | Episode 8 area 3 of the area table, not the lock table | Also class A94, `moveq` at `0x696C2` | FLOW when that area is current |

R: tables in 1.1 and 1.2, `D3c-bosses-1.md`, `D3d-bosses-2.md`, `sound_codes.txt`. Morgue has no music code of its own in D3d. Morgan's 0x0E is the episode 5 boss theme, which is what the album calls Boss 1.

0x20 and 0x23 are short looping beds for the talk, not a second copy of the boss theme. They are not long enough, and not wired as a prefix, to prove the album's boss intros (section 1.6).

### 1.6 Compare with the soundtrack map (N02)

N02 is `plan/game/mechanics-seen.md`: the album and rip map in `plan/game/sources/claude-doc-deep-dive-supplement-2.md`. W: Game Music Online, KHInsider, VGMdb. Not verified by listening. ROM placement wins. Lengths differ because the CD edits loops and the rip's "full loop" is not the sequence length.

Track names below are the album's names (W). The code and the episode are R.

| Album name (W) | Album place (W) | ROM code and place (R) | Verdict |
| --- | --- | --- | --- |
| Overture | Stage 1-1, and Ending 2. CD 3:18, rip 5:40 | 0x02 in episode 1 area 0 (240.65 s). Ending 2 is the copy 0x1A, same length | AGREE on place. Length differs |
| Trap of Silence | 1-2, 2-3, 6-1. CD 2:06, rip 1:59 | 0x03: ep1 area 1, ep2 area 2, ep6 area 0 (109.33 s) | AGREE. ROM shorter than both |
| Pursuit | 1-3, 7-1. CD 1:50, rip 1:55 | 0x04: ep1 area 2, ep7 area 0 (97.58 s) | AGREE |
| Forest Poachers | 2-1, 6-2. CD 3:12, rip 3:19 | 0x05: ep2 area 0, ep6 area 1 (177.06 s) | AGREE |
| Ancient Earth | 2-2, 6-3. CD 1:33, rip 1:23 | 0x06: ep2 area 1, ep6 area 2 (76.13 s), and also ep7 area 3 | AGREE on 2-2 and 6-3. DIFFER: ep7 area 3 is extra. The album has no 7-4 |
| Roaring Sound | 3-A. CD 2:19, rip 2:13 | 0x07: ep3 area 0 (124.75 s) | AGREE |
| Like a Squall | 3-B, 7-2. CD 1:35, rip 1:25 | 0x08: ep3 area 1, ep7 area 1 (79.44 s) | AGREE |
| Held Captive in the Fortress | 4-1, 8-2. CD 2:27, rip 2:16 | 0x09: ep4 area 0, ep8 area 1 (131.47 s) | AGREE. Rip 2:16 is close |
| Shooting Blindly / Brawl | 4-2, 5-2 | 0x0A: ep4 area 1, ep5 area 1 (138.60 s) | AGREE |
| Ambush | 4-3, 7-3. CD 1:40, rip 1:44 | 0x0B: ep4 area 2, ep7 area 2 (82.30 s) | AGREE. ROM shorter |
| Dinosaur Wasteland | 5-1 | 0x17: ep5 area 0 (130.19 s) | AGREE |
| True Colors of Evil | 8-1. CD 3:27, rip 3:40 | 0x0C: ep8 area 0 (190.75 s) | AGREE |
| Madness in the Depths | 8-3. Rip 1:51 | 0x0D: ep8 area 2 (105.51 s) | AGREE. Area 3 of episode 8 is 0x22, which the album does not have |
| Boss 1 | Stages 1 and 5. CD 1:46, rip 0:15 intro + 1:29 | 0x0E: ep1 and ep5, Vice and Morgan (82.80 s) | AGREE on who. Intro not a separate code |
| Boss 2 | Stages 2, 6 and 8 | 0x0F: ep2, ep6, Tyrog, Butcher, and Fessenden's second form (index 8 / class 94). The first form uses 0x12 | AGREE on ep2 and ep6. DIFFER on which episode 8 form gets Boss 2 |
| Boss 3 | Stage 3 | 0x10: Hogg (112.74 s) | AGREE |
| Boss 4 | Stages 4 and 7. CD 3:31, rip 0:47 intro + 1:30 | 0x11: Slice and the episode 7 Slisaurs (166.39 s). Slice's talk bed is 0x23 (27.98 s) | AGREE on who. 0x23 is not proven to be the 0:47 intro |
| Final boss | Stage 8. CD 2:24, rip 0:35 + 1:42 | 0x12: class 84 only (94.82 s). The second form switches to 0x0F | AGREE for the first form. DIFFER for the second form |
| (no album row) | | 0x22, episode 8 area 3, 35.19 s, does not loop | DIFFER. Extra scene |
| Player Select | 0:25 CD, 0:21 rip | Select screen plays 0x00 (14.98 s). 0x1C is 25.63 s and plays when the continue count is 9 | DIFFER if the album name is taken as the select screen. The 0:25 length matches 0x1C, not 0x00 |
| Opening Demo | 1:07 CD, 1:12 rip | 0x01 is 54.46 s and is the episode 1 stage start, not the attract movie. The movie uses effects (0x61, 0xEA, 0x73, 0x75, 0x36, 0x88), and the filter mutes them | DIFFER |
| Demo (attract) | 0:43 CD, 0:35 rip | No music code plays under factory demo sound. The title sting is effect 0x88 (0.91 s) | DIFFER |
| Stage Clear | 0:07 CD, 0:06 rip | 0x18 is 6.44 s | AGREE |
| Return (Ending 1) | 1:49 CD, 1:47 rip | 0x19 is 107.55 s. 1:47 is 107 s | AGREE |
| Ending 2 | the stage 1-1 theme | 0x1A, copy of 0x02 | AGREE |
| Continue, two versions | rip 0:36 and 0:31 | 0x1B (31.13 s) then 0x1C (25.63 s) at number 9 | AGREE on two versions. Lengths are close, not equal |
| Game over | rip 0:05 | 0x1D is 6.12 s | AGREE |
| Name entry | 0:47 CD, 0:35 rip | 0x1F is 29.40 s | AGREE on the role. Length is nearer the rip than the CD |
| Time over | rip 0:04 | 0x1E is 3.89 s and has no sender | Do not assign. Open |
| (not on the album map) | | Story and profile codes 0x13 to 0x16, slide codes 0x24 and 0x25, talk beds 0x20, 0x21, 0x23 | Extra ROM cues |

The 22 area uses of the 13 shared area themes match the album's stage-area labels, plus two extra slots (episode 7 area 3, episode 8 area 3). Boss reuse matches, except the final boss's second form uses Boss 2's code. Demo-sound factory default: D4d and the ROM filter say off. The supplement's bootleg DIP list says the default is on. R wins. W: supplement 2.

## 2. Voices

### 2.1 Which bank

Routine `0xA502`, table `0xA53A` (32 bytes), called once from `0x18884` at the start of the player update. The port's `snd_update_voice_banks` does this. R: `snd.rs`, `dino-rs/crates/dino-assets/src/text.rs`.

The mix is not "how many players". It is a byte: player 1 adds 4, player 2 adds 8, player 3 adds 0x10. That byte plus the slot (0, 1 or 2) indexes the table. One player, any slot: bank 0. Two players: the higher slot gets bank 1 and the other gets 0 (slots 0+1 give 0, 1, 0; slots 0+2 give 0, 0, 1; slots 1+2 give 0, 0, 1). All three: banks 0, 1, 2. R: bytes at 0xA53A. Join order is not stored. Only which slots are live.

The bank picks the column of that hero's row, so a second player can use a twin code of the same sample. Whether the twin header names a different hardware voice was not checked. SPEC says an equal priority replaces the effect on that voice, so two players do not overlap unless the headers differ. Open.

Character id is `+$20 / 4`: 0 Jack, 1 Mustapha, 2 Hannah, 3 Mess. R: `Claude.md`.

### 2.2 Hero routines

The port's `snd_player` (`snd.rs`) covers every row in this table except `0x9814`. It passes no d1 and no d2. `tag` is hardcoded to 0, so the non-player branch of `0x9AF4` and of `0x9C0C` never runs. Class 20 sends the landing pair itself (`class20_lib.rs` `snd_9c20`).

Pan for the routines that were re-read is in section 4. The shared shape is: mode 3, or 4 if facing `+$24` is set, then middle byte 0x20, then the pan byte, d2 often `0x15000000` (that d2 is what `0x962A` writes).

| Routine | Callers | Rule | Codes by id, then by bank 0, 1, 2 | Port |
| --- | --- | --- | --- | --- |
| `0x95CE` jab | 4: `0x1D34`, `0x1E5C`, `0x19410`, `0x19538`. Port: punch and punch2 | Toggle `G.$43A` bit 0. First request after clear is the second code | 0x38 / 0xA8. Both sample 73. 0x38 is 0.13 s, priority 3 | REC |
| `0x986C` death shout | 10. Port: knockdown when HP is 0 | Id 2 (Hannah) uses the second row. Mess (id 3) uses the first row | Others: 0x44, 0xB8, 0xB9. Hannah: 0x45, 0xBA, 0xBB. Table `0x98C6` | REC |
| `0x98CC` hurt | 12. Port: light hurt, hurt behind, get-up. D2b: the hurt state | Same row rule as death | Others: 0x3C, 0xAC, 0xAD (sample 61, 0.26 s). Hannah: 0x3D, 0xAE, 0xAF (sample 66, 0.26 s). Table `0x9926` | REC |
| `0x992C` ground special | 2: `0x1FC6`, `0x196A2` (`player_special_entry`). Not the team-move start | Row = character id * 4, column = bank, pad 0x00. Table `0x9974` | Jack 0x4B, 0xC2, 0xC3. Mustapha 0x4D, 0xC6, 0xC7. Hannah 0x4C, 0xC4, 0xC5. Mess 0x4E, 0xC8, 0xC9 | REC |
| `0x9984` dash attack | 2: `0x2044`, `0x19720` | Rows of 3 for id 0, 1, 2. Id 3 falls through to id 0. Table `0x99DE` | Id 0: 0x46, 0xBC, 0xBD. Id 1: 0x47, 0xBE, 0xBF. Id 2: 0x48, 0xC0, 0xC1. 0x46 and 0x47 are also hit ids 7 and 8 | REC |
| `0x99E8` punch 3 | 2: `0x1EBA`, `0x19596` | Row = id * 4. Table `0x9A26` | Jack 0x4F, 0xCA, 0xCB. Mustapha 0x53, 0xD2, 0xD3. Hannah 0x51, 0xCE, 0xCF. Mess 0x55, 0xD6, 0xD7 | REC |
| `0x9A36` punch 4 and rising attack | 4: `0x1F78`, `0x3FF2`, `0x19654`, `0x1B718` (action 0x15) | Table `0x9A74` | Jack 0x50, 0xCC, 0xCD. Mustapha 0x54, 0xD4, 0xD5. Hannah 0x52, 0xD0, 0xD1 (also hit id 9). Mess 0x56, 0xD8, 0xD9 | REC |
| `0x9814` pose / 1UP voice | 2: `0x22DE`, `0x199BA` | Mode 7, d2 = 0x400000, pan from `0xA66A`. Table `0x985C`, row = id * 4 | Jack 0x40, 0xB0, 0xB1 (0x40 is 0.83 s, sample 56). Mustapha 0x42, 0xB4, 0xB5 (0x42 is 1.28 s, samples 89 and 92). Hannah 0x41, 0xB2, 0xB3 (0x41 is 1.21 s, sample 38). Mess 0x43, 0xB6, 0xB7 | MISS. The port only plays jingle 0x4A (`hud.rs` `snd_extra_life`, routine `0xA4CA`) |
| `0x9AF4` water splash | 1: `0x60DA` | If `+$82 == 0`, table `0x9B5C`: 0x6A, 0xED, 0xEE. Else toggle bit 4: 0xEF / 0xF0 | Same sample family on both branches | REC for the player row only |
| `0x9B60` wading steps | 4 | Toggle bit 5 (shared with footsteps): 0x6C / 0xF2 | | REC |
| `0x9B9E` hard landing | 2: `0x2744`, `0x19E6A`. D2a: Mess flying-lariat landing, screen shake | Fixed 0x75 | | REC |
| `0x9BCE` footstep | 4 | Toggle bit 5: 0x71 / 0xF7 | | REC |
| `0x9C0C` landing | 8 | No sound if wading and Y < 0x20. Players: table `0x9C72` = 0x62, 0xDE, 0xDF. Others: toggle bit 6, 0xE0 / 0xE1 | | REC for the player row. 0xE0 / 0xE1 also REC from `snd_9c20` |
| `0x9796` thud | 44, players and enemies | Toggle bit 3: 0x72 / 0xF8. Not a voice | | REC `snd_9796` |

R: bytes of each table (section 0's read), caller counts from an absolute-call scan, `snd.rs`, D2a, D2b, D4d section 3 (profile: name voices 0x4B to 0x4E and pose voices 0x40 to 0x43, which are bank 0 of `0x992C` and of `0x9814`). Lengths and samples for every code are in section 5.

D4d measured no hero voice on the select screen. The codes there are 0x31, 0x00, 0x32, 0x33, 0xFF00, then the level music.

**Team move.** Start `0x686E`, states 0x1A / 0x424 / 0x422 / 0x426. None of the voice routines lists `0x686E` as a caller. D2a section 9 records no sound call. W: mechanics-seen row M25 (Jack "Let's Go!", Mustapha "Let's Roll!", Mess "Let's Fly!", Hannah "Jump!"). Those words are not in the ROM notes. Do not assign `0x992C` to the team move.

**Explosive kill line.** D2b: the explosion helper is `0x9F12`. It always queues 0x66 (mode 5 or 6 by facing, middle byte 0x12, d2 = 0x01561080). No voice table. `item/blast.rs` sends 0x66. W: M25 (Jack "Take that", Hannah and Mustapha "Gotcha", Mess "Blammo"), plus eyeballs in the remains. Owned with D3f, which is not written yet. Not assigned.

### 2.3 Enemy and boss voices

| Code or routine | Event | Port |
| --- | --- | --- |
| `0xA09E` queues 0x60, pan from X. 10 call sites, including `0x5361A` and `0x538CC` | Mack roar (and the calm), Hopper roar, Shivat roar. R: `D3b-dinosaurs.md` | MISS |
| `0xA0CC` queues 0x5F | Zeke grab-release slam, Hopper. The same code is hit id 11 and the Butcher's chop | MISS on the dinosaur path. Hit path REC if the attack record uses id 11 |
| `0xA288` queues 0x76, d2 = 0, pan from X. Callers `0x47EF8`, `0x59264` | Brontosaurus foot, with screen shake 4. Same code as hit id 10 | MISS on the foot path |
| `0x97D2` / `0xA354` | Enemy death. If `+$63` is positive, 0x58, else 0x79. Port `snd_enemy_death`. What `+$63` means for each class is open. Scenery calls it with 0, so 0x79 | REC |
| `0x9732` | Blocked hit. Toggle `G.$43A` bit 2: 0x7B / 0xFC. d2 = 0, pan from X. Caller `0x10B42`. Port `player_combat.rs` | REC |
| `0x96CE` | Toggle bit 1: 0x65 / 0xE4. d2 = 0. Callers `0x111AE`, `0x11CB6`, `0x55A0A`. Port: class 41 blade when it is hit (`class14_parts.rs`) | REC |
| 0x20, 0x21, 0x23 | Vice talk, Butcher starter, Slice talk. Section 1.5 | MISS |
| `0x9D9C` | Morgan's Uzi burst (D3d). Item branch is 0xDC. Player branch is the table in section 4 | REC as 0xDC only |
| 0x79 | Villager shot, D3d, and the non-positive `+$63` death | REC via `snd_enemy_death` |
| 0x37 | Lab tank glass (pool B 44) and the hall window (pool B 12). D3d also: Hopper and Tyrog tanks, d1 = 0x72010, d2 = 0x2002200 at `0x83C62`. Bludge tanks are silent | MISS |
| 0x78 | Jungle maker, pool B 80, every 120 frames with a 1 in 4 chance and then a coin flip. R: D3e, mechanics-seen W32 | MISS |

Dinosaur classes in D3b do not have a separate voice table beyond these helpers. Do not invent growls for the rest.

## 3. Hit sounds

Routine `0x962A`. The attack record stores a byte offset into table `0x96AA` (36 bytes, 12 entries of 3). The port's `snd_hit` reads the same table (`player_combat.rs`, `enemy_combat.rs`). Entry: bit number in `G.$43C`, code if that bit is clear, code if it is set. The routine flips a copy of the word and does not store it (section 0).

Pan, re-read this task: mode 3, or 4 if `+$24` is set, then `move.w #$2000` (middle byte 0x20), pan from `0xA66A` using the X of **a2** (not a6), d2 = `0x15000000`. The port stores the code only.

| Id | Bit | Clear | Set | Sample, from `sound_codes.txt` | Also |
| --- | --- | --- | --- | --- | --- |
| 0 | 0 | 0x00 | 0x00 | music 0x00 | If any attack uses offset 0, the select tune starts. Not checked whether a record does |
| 1 | 1 | 0x39 | 0xA9 | 110, 0.15 s | |
| 2 | 2 | 0x3A | 0xAA | 98, 0.18 s | |
| 3 | 3 | 0x3B | 0xAB | 90, 0.27 s | 0x3B also Vice's gun and `scenery.rs`. 0xAB also the continue-accept jingle (D4d) and `0xA1D8` |
| 4 | 4 | 0x63 | 0xE2 | 104, 0.34 s | |
| 5 | 0 | 0x66 | 0x66 | 99, 0.60 s | No alternate. Explosions (`0x9F12`), item blasts, Hogg's bomb, ending lab bursts |
| 6 | 5 | 0x5E | 0xDD | 93, 0.42 s | |
| 7 | 6 | 0x46 | 0xBC | 49, 0.34 s | Same codes as dash voice id 0, bank 0 and 1 |
| 8 | 7 | 0x47 | 0xBE | 113, 0.24 s | Dash voice id 1 |
| 9 | 8 | 0x52 | 0xD0 | 52, 0.38 s | Hannah's punch-4 sample |
| 10 | 0 | 0x76 | 0x76 | 97 | No alternate. Also the foot at `0xA288`, and direct sites `0xA2B2`, `0xA2EA` |
| 11 | 0 | 0x5F | 0x5F | 104, 0.34 s | No alternate. Butcher chop, `0xA0CC` |

R: bytes at 0x96AA, disassembly of 0x962A, `sound_codes.txt`.

## 4. Named effects

Lengths are in section 5. Parameters are the ones re-read in this task, or the ones D4d already printed.

### 4.1 Items and weapons

Port: `item/shared.rs`, `item/small.rs`, `item/container.rs`, `item/blast.rs`, `item/weapon` callers. Pan is not recorded.

| Routine | Codes | Who | Port |
| --- | --- | --- | --- |
| `0x9FB6` | Toggle `G.$43B` bit 1: 0x6E / 0xF5, sample 108 | Container break and land. 9 callers. Barrel hop uses this when variant low byte is 0 (`0x5B9FC`) | REC |
| `0x9E60` | Bit 0: 0x6F / 0xF6, sample 81 | 15 callers. Walls, doors (D3e classes 3, 4, 11), weapons | REC |
| `0x9FF8` | Bit 2: 0x74 / 0xFA, sample 106 | 1 caller, `0x294AA` | REC |
| `0xA404` | Bit 7: 0x8C / 0x105 | 1 caller, `0x29832` | REC |
| `0xA396` | 0x89, 1.21 s, sample 116 | Port `container.rs` | REC |
| `0x9C76` | If `+$82 != 0`: 0xEA (sample 107, priority 4). If `+$82 == 0`: table 0x68, 0xE8, 0xE9 by the owner's bank. d2 on the player path is `0x15000000`, mode 3 or 4 | 4 callers: `0x25200`, `0x3DA3A`, `0x3EA84`, `0x6AC02`. D3a: the shot sound | REC as 0xEA only. Player row MISS |
| `0x9CD8` | Item: 0xE7 (sample 78). Player: table `0x9D40` = 0x67, 0xE5, 0xE6. d2 = `0x15000000` | 5 callers. 0xE6 is also the game-over blood splash (D4d, seven times) | REC as 0xE7 only |
| `0x9D44` | Toggle `G.$43A` bit 7: 0x64 / 0xE3, sample 73 (same sample as the jab) | 18 callers. Thrown knife. Port `item_snd_9d44` and `lib32.rs` | REC |
| `0x9D9C` | Item: 0xDC (sample 82). Player: table `0x9E04` = 0x5D, 0xDA, 0xDB. d2 = `0x02400000`, mode 3 or 4 | 7 callers, including Morgan `0x51486` and `0x514FC` | REC as 0xDC only |
| `0x9E08` bazooka | Table `0x9E5C`: 0x69, 0xEB, 0xEC (sample 105). Mode 5 or 6 by facing, middle byte 0x12, d2 = `0x00C41240` | Callers `0x258F0`, `0x6B2F2` | REC |
| `0xA442` shotgun | Words at `0xA498`: 0x0081, 0x0102, 0x0103 (sample 103) | Callers `0x257CC`, `0x6B1CE` | REC |
| `0x9F54` food | Not Hannah: words 0x007C, 0x00FD, 0x00FE (sample 67). Hannah (id 2): 0x007D, 0x00FF, 0x0100 (sample 64). Bank 0, 1, 2 only | 1 caller, `0x28A96`. Port `item_snd_food` | REC |
| blast `0x9F12` | Always 0x66. Mode 5, or 6 if facing is set, then middle byte 0x12, pan from a6's X, d2 = 0x01561080 | Item blast, Hogg's bomb, ending debris | REC from `item/blast.rs`. Other callers MISS |

R: disassembly of 0x9C76 through 0x9E60 and of 0xA3C6, bytes of the tables, D3a section 10, D3d.

### 4.2 Clock, GO, lift, tally

**Clock beep 0x34.** Class 31, `0x810E2`. When clock word `G.$4E8` changes and object `+$A2` is already non-zero: `moveq #$34`, `moveq #$10` into d1, `moveq #0` into d2, `jsr 0xA63C`. d1 = 0x10 means no position pan. If `+$A2` is 0 on a change, it arms a window instead of beeping: 0x12C frames (5 s) when the low byte of the clock is 0x00 or 0x30 (the :00 and :30 marks), otherwise 0, and 0x320 frames (800) when the clock word is 0x15 or less. `+$A2` counts down every frame, so the beep repeats each second while the window is open. 0x34 is an effect, priority 5, 0.34 s, 3 voices, sample 10 (sample 10 is also a music instrument). The "show the clock for 5 s at 3:00" display rule is separate (mechanics F02, D4b). Port: MISS. `rank.rs` can zero HP. It does not beep.

**GO hint 0x3E.** `moveq #$3E` at `0x7BC58` and `0x7BD7C`. Both are in class 20 (D4c, D3e class 20: blink the GO text). Effect, priority 6, 0.40 s, sample 121. Port: MISS (D4c task T-C3).

**Lift.** D4c: start plays 0x86 (10.92 s loop, sample 85, priority 5, `0x21106`, d1 = 0x10). Stop at Y plays 0x87 (1.33 s, sample 79, callers `0x20C2C`, `0x210AC`, `0x2119C`, `0x22A24`). The ride restarts, so 0x86 repeats. Port `camera.rs`: REC.

**Tally 0x49.** 0.80 s, 5 voices, sample 18. Every 11 frames while a bonus counts, and on the 4 pops. Direct `moveq` sites `0xBC2E`, `0xBD84`, `0xBEB0`, `0xBFAC`, `0xC334`, `0x7C6A2`, `0x7C7F4`, `0x7C920`, `0x7CA1C`, `0x7CDA4`. R: D4d section 9.1, and those addresses are in the `jsr 0xA63C` list. Port: MISS.

### 4.3 Front end effects

All of these are muted in attract unless demo sound is on, except 0x30 and the 0xFFxx commands. R: section 0, D4d section 11.

| Code | What | Parameters D4d recorded | Port |
| --- | --- | --- | --- |
| 0x30 | Coin and service credit. Priority 10, the only one. 1.81 s, 2 voices, sample 96. `jsr` `0x9D348`, `0x9D598` | d1 = 0x70010, d2 = 0x3800000 | MISS |
| 0x31 | Start press with a credit. 0.48 s, priority 5, 4 voices, sample 24. `0x9D168`. Sets `G.$4CC = 1`, so later audio is not filtered | d1 = 0x10 | MISS |
| 0x32 | Select cursor. 0.32 s, sample 7. `0xB222` | pan d1 = 0x00, 0x0B, 0x15, 0x20 for the four positions | MISS |
| 0x33 | Select pick. 0.57 s, sample 5. `0xB19C` | same pan | MISS |
| 0x88 | Title sting, also the movie at counter 1350, also pool B 14 (a door). 0.91 s, sample 114. Extra direct send at `0x6A3E2` with d1 = 0, d2 = 0 | | MISS |
| 0x80 | Ranking, two frames after the stop. 1.45 s, sample 95, priority 7 | D4d: d1 = 0x72000, d2 = 0x4000000 | MISS |
| 0x70 | Ranking wind, about 78 frames later. 10.28 s loop, sample 77, priority 5 | same d1, d2 shape in D4d | MISS |
| 0x35 | Continue tick. 0.40 s, sample 24. `0x7EF6` and `0xF460` | d1 = 0x10 | MISS |
| 0xAB | Continue accepted. 0.27 s, sample 90. Also hit id 3's other code | D4d: one site d1 = 0x72020, d2 = 0x1C000000 | MISS as a continue cue. REC only if a hit id 3 runs with the bit set, which the port never does |
| 0xE6 | Game over blood, seven times, every 18 frames. Sample 78, 0.53 s. Also bank 2 of `0x9CD8` | | MISS |
| 0xE1, 0xE0 | D4d measured these in the join strip (0xE1 about 16 frames after the number 2, 0xE0 about 56 frames after the hero is picked). Sample 101, 0.26 s. The same pair is the non-player landing toggle of `0x9C0C` (bit 6) and `snd_9c20`. The strip's own caller was not disassembled | | 0xE0 / 0xE1 REC from class 20, not from the join strip |
| 0x4A | Extra life jingle. 1.28 s, 5 voices, sample 21. Routine `0xA4CA`, 1 caller `0x6BE2` | | REC `hud.rs` |
| 0x61, 0xEA, 0x73, 0x75, 0x36 | Movie: steps, a throw-like code, a rustle, dinosaur steps, a roar. 0x36 is 1.61 s, sample 96, priority 7, catalog owner A22. 0x73 is also the class 10 toggle. 0x75 is the hard landing | | 0xEA, 0x73, 0x75 can be sent by item and player code. The movie itself is MISS |
| 0x66, 0xBB, 0x10, 0xF6, 0x56 | Ending lab, D4d section 8: 0x66 every 16 frames, 0xBB at t = 261, 0x10 and 0xF6 at t = 469, 0x56 at t = 530. 0x56 is also queued at `0x8ADC6` with d1 = 0x50020 and d2 = 0x00800000 (this task). Pool B 82 plays 0x56 (D3e) | | 0x66 and 0xF6 can come from item code. The ending scene is MISS |

### 4.4 Biker, barrel, boomerang, and two unnamed sends

**Class 70 biker** (`0x5C356`, D3a). `0xA128` toggles `G.$43E` bit 1: 0x7F / 0x90, d2 = 0, pan from X. Call sites: ride attack setup `0x5C43E` (attack id 0x47), the frame the 30 frame wait ends `0x5C5D0`, and the kick `0x5C65A` (sets `+$6 = 2`). The fall is a different helper: `0xA164` at `0x5C6B4`, toggle `G.$43E` bit 0, 0x7A / 0xFB. D3a already named `0xA164` as the rider falling. Port: MISS. The port has no `G.$43E`.

**Barrel hop, the non-wood kind.** At `0x5BA06`, if `+$27` is 0 the landing calls `0x9FB6`, else `0xA3C6`. `0xA3C6` toggles `G.$43B` bit 6: 0x8B / 0x104, d2 = 0, pan from X. Port: MISS.

**Boomerang, class 19** (handler `0x45082`, shared with Slice). `0x451C6` calls `0xA21E` when `+$B2` expires and `+$A9` is 0. `0xA21E` toggles `G.$43B` bit 5: 0x7E / 0x101, d2 = 0, pan from X. Port: MISS.

**0x82 and 0x83** do have senders. They were missing from the catalog's "who" column because the code is a `move.l`, not a `moveq`.

- 0x82 (1.56 s, priority 5, sample 83). `0x37C7E` and `0x6669A` were checked: d0 = 0x82, d2 = `0x00540080`, d1 = `0x0005011F`, or `0x0006201F` when `(X - 0x1000)` is not below 0xC0. The same trailing test sits in front of `0x37CF8`, `0x37D68`, `0x37DEE`, `0x66714`, `0x66784` and `0x6680A`. Also `0x5ED4A`, with d1 = `0x00062020` and the same d2.
- 0x83 (0.77 s, priority 5, sample 86). `0x5ED7C` uses the same d1 and d2 as `0x5ED4A`. `0x697CE` sends 0x83 with d1 = 0 and d2 = 0.

The class that owns the 0x37Cxx and 0x666xx blocks was not named. Port: MISS.

**0x60 besides the roar helper.** `0x54CBC` and `0x68B64` send `move.l #$60` with d1 = `0x00071010` and d2 = `0x08000000`. `0x5ED94` sends 0x60 with d1 = 0 and d2 = 0. `0x4FBC2` sends 0x36 with d1 = 0 and d2 = 0.

### 4.5 Ambient loops that do have a caller

| Code | Length | Where | Port |
| --- | --- | --- | --- |
| 0x6D | | Pool B 61, every 120 frames (stage 7 palette cycle). R: D3e | MISS |
| 0x8D | | Pool B 56, falling dust, every 240 frames. R: D3e | MISS |
| 0x8E | | Pool B 62, stage 5 palette cycle. R: D3e | MISS |
| 0x8A | | Pool B 92 portrait box. Also `0x91E1C` with d1 = 0, d2 = 0. R: D3e | MISS |
| 0x5A | | Class 10, port `class10_ai.rs`, comment says pan by X was not compared | REC |

## 5. Every live code

243 live codes (39 music, 204 effects) and 19 empty slots. R: asset-catalog N8. Empty slots, no header: 0x27 to 0x2B and 0x92 to 0x9F. Codes 0x100 to 0x105 are real entries (the table count is 0x106). They are the high twins of shotgun, food, container, boomerang and barrel codes.

The path column is the event when this task named it. Otherwise it is the sender line from `sound_codes.txt`, shortened. "Table" means there is no `move.w #code` and the code is reached only from a table or a toggle in sections 2 to 4. A catalog "code at" on music 0x00 inside the helper range is omitted (section 0).

Port column: REC, FLOW, or MISS, as in the header. FLOW+ means the area or lock table can send it and the boss or front end send is still MISS.

REC means the port already calls `sound()` with this code. FLOW means `flow_stage_start` or `flow_area_music` can send it. Neither means the speakers play it.

| Code | Kind | Prio | Voices | Length | Loop | Samples | Path | Port |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0x00 | music | - | 14 | 14.98 s | yes | 4, 9, 10, 12, 13, 15, 18, 19, 20, 23, 26, 27, 30 | Select tune (D4d, d1=0xFFFF). Also hit id 0. Not the false helper labels | FLOW if a 0x00 slot is read. Select MISS. Hit id 0 would send it |
| 0x01 | music | - | 13 | 54.46 s | yes | 3, 16 | Episode 1 stage start, 0xACCA, d1=0, d2=0 | FLOW |
| 0x02 | music | - | 13 | 240.65 s | yes | 2, 7, 9, 17, 19, 23, 25, 26, 27, 30 | Episode 1 area 0. Copy of 0x1A | FLOW |
| 0x03 | music | - | 16 | 109.33 s | yes | 9, 10, 13, 15, 17, 19, 21, 23, 24, 26, 27, 30 | Ep1 area 1, ep2 area 2, ep6 area 0 | FLOW |
| 0x04 | music | - | 12 | 97.58 s | yes | 2, 9, 10, 17, 18, 19, 23, 24, 25, 27, 35 | Ep1 area 2, ep7 area 0 | FLOW |
| 0x05 | music | - | 16 | 177.06 s | yes | 0, 1, 2, 4, 6, 7, 9, 12, 13, 15, 17, 19, 20, 23, 24, 26, 27, 30, 32, 33 | Ep2 area 0, ep6 area 1 | FLOW |
| 0x06 | music | - | 14 | 76.13 s | yes | 0, 1, 3, 6, 9, 10, 11, 13, 19, 23, 25, 26, 30, 33 | Ep2 area 1, ep6 area 2, ep7 area 3 | FLOW |
| 0x07 | music | - | 14 | 124.75 s | yes | 2, 4, 7, 8, 9, 13, 17, 20, 23, 24, 25, 26, 27, 30 | Episode 3 area 0 | FLOW |
| 0x08 | music | - | 12 | 79.44 s | yes | 2, 7, 9, 13, 17, 19, 23, 25, 26, 27, 30 | Ep3 area 1, ep7 area 1. Ignore catalog 'code at 0xA5C4' | FLOW |
| 0x09 | music | - | 14 | 131.47 s | yes | 2, 7, 8, 9, 13, 14, 15, 17, 19, 21, 23, 24, 26, 27, 30 | Ep4 area 0, ep8 area 1 | FLOW |
| 0x0A | music | - | 15 | 138.60 s | yes | 2, 7, 9, 13, 15, 17, 20, 21, 22, 23, 24, 26, 27, 30 | Ep4 area 1, ep5 area 1 | FLOW |
| 0x0B | music | - | 15 | 82.30 s | yes | 9, 10, 15, 17, 18, 19, 21, 23, 24, 26, 27, 30 | Ep4 area 2, ep7 area 2 | FLOW |
| 0x0C | music | - | 12 | 190.75 s | yes | 2, 5, 9, 13, 18, 19, 22, 23, 24, 25, 26, 27, 30, 33 | Episode 8 area 0 | FLOW |
| 0x0D | music | - | 13 | 105.51 s | yes | 2, 3, 5, 9, 13, 19, 22, 23, 25, 26, 27, 30, 122 | Episode 8 area 2 | FLOW |
| 0x0E | music | - | 10 | 82.80 s | yes | 2, 3, 7, 9, 10, 13, 15, 17, 20, 23, 24, 26, 27, 30 | Boss ep1 and ep5. Vice, Morgan | FLOW |
| 0x0F | music | - | 13 | 88.64 s | yes | 3, 9, 13, 15, 17, 18, 19, 20, 21, 23, 24, 26, 27, 30 | Boss ep2 and ep6, Fessenden second form. Tyrog, Butcher, class 94 | FLOW |
| 0x10 | music | - | 10 | 112.74 s | yes | 2, 3, 7, 9, 10, 15, 17, 19, 20, 21, 23, 24, 27, 30 | Boss ep3. Hogg | FLOW |
| 0x11 | music | - | 14 | 166.39 s | yes | 2, 7, 9, 10, 12, 15, 17, 19, 20, 23, 27, 30 | Boss ep4 and ep7. Slice, Slisaurs | FLOW |
| 0x12 | music | - | 15 | 94.82 s | yes | 3, 9, 13, 15, 17, 18, 19, 20, 21, 23, 24, 26, 27, 30, 99, 105 | Boss ep8 first form. Fessenden class 84 | FLOW |
| 0x13 | music | - | 14 | 14.70 s | yes | 3, 9, 15, 17, 18, 20, 23, 27 | Profile card and story ep2, ep6 | MISS |
| 0x14 | music | - | 13 | 7.18 s | no | 5, 9, 13, 15, 17, 19, 23, 26, 27, 30 | Story ep3, ep7 | MISS |
| 0x15 | music | - | 11 | 19.48 s | yes | 3, 9, 10, 15, 17, 18, 19, 23, 27 | Story ep4, ep8 | MISS |
| 0x16 | music | - | 12 | 14.15 s | yes | 3, 9, 15, 17, 18, 19, 21, 23, 27 | Story ep5 | MISS |
| 0x17 | music | - | 10 | 130.19 s | yes | 2, 7, 9, 13, 16, 22, 23, 25, 26, 27, 30 | Episode 5 area 0 | FLOW |
| 0x18 | music | - | 12 | 6.44 s | no | 9, 10, 13, 15, 17, 20, 23, 25, 26, 27, 30 | Victory pose, 0x1E24C | MISS |
| 0x19 | music | - | 15 | 107.55 s | no | 0, 3, 7, 9, 13, 15, 16, 17, 18, 19, 20, 23, 25, 26, 27, 30, 35, 120 | Ending desert, 0x8A618, after a stop | MISS |
| 0x1A | music | - | 13 | 240.65 s | yes | 2, 7, 9, 17, 19, 23, 25, 26, 27, 30 | Ending car, 0x8A7A4. Same sequence as 0x02 | MISS |
| 0x1B | music | - | 13 | 31.13 s | yes | 9, 10, 15, 17, 18, 20, 23, 24, 27, 36 | Continue music, 0xEB20 | MISS |
| 0x1C | music | - | 13 | 25.63 s | yes | 9, 10, 15, 17, 18, 20, 23, 24, 27 | Continue when the number is 9, 0xF47C | MISS |
| 0x1D | music | - | 12 | 6.12 s | no | 2, 9, 10, 15, 20, 23, 24, 27 | Game over jingle, 0xEDB4 | MISS |
| 0x1E | music | - | 14 | 3.89 s | no | 9, 13, 15, 17, 20, 23, 24, 26, 27, 30 | No path (section 7) | MISS |
| 0x1F | music | - | 13 | 29.40 s | yes | 9, 10, 15, 19, 20, 23, 24, 31 | Name entry, 0xEDFC | MISS |
| 0x20 | music | - | 9 | 9.23 s | yes | 2, 3, 6, 15 | Vice talk bed. Also catalog A45 | MISS |
| 0x21 | music | - | 10 | 7.24 s | yes | 3, 18, 19, 24, 27, 32 | Butcher starter. Also pool B 79 | MISS |
| 0x22 | music | - | 12 | 35.19 s | no | 23, 27, 34 | Episode 8 area 3. Also class A94 at 0x696C2 | FLOW |
| 0x23 | music | - | 8 | 27.98 s | yes | 3, 9, 15, 17, 19 | Slice talk, episode 8 clear action 0x42, pool B 30 | MISS |
| 0x24 | music | - | 13 | 29.96 s | yes | 3, 9, 13, 15, 18, 19, 20, 23, 26, 27, 30 | Attract slides start, 0x988E2 | MISS |
| 0x25 | music | - | 16 | 6.06 s | no | 9, 10, 15, 17, 20, 23, 24, 26, 27, 30 | Fifth slide, 0x98AF0 | MISS |
| 0x26 | music | - | 9 | 5.55 s | no | 5 | No path (section 7). Class 42's 0x26 is an object variant | MISS |
| 0x2C | sfx | 7 | 1 | 0.48 s | no | 59 | No path (section 7) | MISS |
| 0x2D | sfx | 7 | 1 | 0.48 s | no | 57 | No path (section 7) | MISS |
| 0x2E | sfx | 7 | 1 | 0.48 s | no | 63 | No path (section 7) | MISS |
| 0x2F | sfx | 7 | 1 | 0.72 s | no | 65 | No path (section 7) | MISS |
| 0x30 | sfx | 10 | 2 | 1.81 s | no | 96 | Coin. Always passes the attract filter | MISS |
| 0x31 | sfx | 5 | 4 | 0.48 s | no | 24 | Start press | MISS |
| 0x32 | sfx | 5 | 4 | 0.32 s | no | 7 | Select cursor | MISS |
| 0x33 | sfx | 5 | 4 | 0.57 s | no | 5 | Select pick | MISS |
| 0x34 | sfx | 5 | 3 | 0.34 s | no | 10 | Clock beep, class 31, d1=0x10, d2=0 | MISS |
| 0x35 | sfx | 5 | 3 | 0.40 s | no | 24 | Continue tick | MISS |
| 0x36 | sfx | 7 | 1 | 1.61 s | no | 96 | Movie roar. Also 0x4FBC2 with d1=0, d2=0. Catalog A22 | MISS |
| 0x37 | sfx | 7 | 1 | 0.72 s | no | 84 | Tank glass and hall window. D3d d1=0x72010, d2=0x2002200 | MISS |
| 0x38 | sfx | 3 | 1 | 0.13 s | no | 73 | Jab, toggle bit 0, with 0xA8 | REC |
| 0x39 | sfx | 5 | 1 | 0.15 s | no | 110 | hit sound id 1 (routine 0x962A, alternates with the other code of the pair) | REC |
| 0x3A | sfx | 5 | 1 | 0.18 s | no | 98 | hit sound id 2 (routine 0x962A, alternates with the other code of the pair) | REC |
| 0x3B | sfx | 5 | 1 | 0.27 s | no | 90 | Hit id 3, and Vice gun, scenery.rs | REC |
| 0x3C | sfx | 7 | 1 | 0.26 s | no | 61 | Table or toggle only (sections 2 to 4) | REC |
| 0x3D | sfx | 7 | 1 | 0.26 s | no | 66 | Table or toggle only (sections 2 to 4) | REC |
| 0x3E | sfx | 6 | 1 | 0.40 s | no | 121 | GO hint, class 20, 0x7BC58 and 0x7BD7C | MISS |
| 0x3F | sfx | 5 | 4 | 0.30 s | no | 24 | No path. 0xA614 is the ring-clear counter | MISS |
| 0x40 | sfx | 7 | 1 | 0.83 s | no | 56 | Pose / 1UP voice, Jack bank 0. Table 0x985C | MISS |
| 0x41 | sfx | 7 | 1 | 1.21 s | no | 38 | Table or toggle only (sections 2 to 4) | MISS |
| 0x42 | sfx | 7 | 1 | 1.28 s | no | 89, 92 | Table or toggle only (sections 2 to 4) | MISS |
| 0x43 | sfx | 7 | 1 | 0.90 s | no | 53 | Table or toggle only (sections 2 to 4) | MISS |
| 0x44 | sfx | 7 | 1 | 0.67 s | no | 100 | Table or toggle only (sections 2 to 4) | REC |
| 0x45 | sfx | 7 | 1 | 1.12 s | no | 41 | Table or toggle only (sections 2 to 4) | REC |
| 0x46 | sfx | 7 | 1 | 0.34 s | no | 49 | hit sound id 7 (routine 0x962A, alternates with the other code of the pair) | REC |
| 0x47 | sfx | 7 | 1 | 0.24 s | no | 113 | hit sound id 8 (routine 0x962A, alternates with the other code of the pair) | REC |
| 0x48 | sfx | 7 | 1 | 0.83 s | no | 50 | Table or toggle only (sections 2 to 4) | REC |
| 0x49 | sfx | 5 | 5 | 0.80 s | no | 18 | Tally count and the 4 pops (D4d 9.1). Queued at 0xBC2E, 0xBD84, 0xBEB0, 0xBFAC, 0xC334 and the 0x7C6A2 group | MISS |
| 0x4A | sfx | 5 | 5 | 1.28 s | no | 21 | Extra life jingle, 0xA4CA | REC |
| 0x4B | sfx | 7 | 1 | 1.25 s | no | 37 | Jack special bank 0, and the profile name voice | REC |
| 0x4C | sfx | 7 | 1 | 1.16 s | no | 40 | Table or toggle only (sections 2 to 4) | REC |
| 0x4D | sfx | 7 | 1 | 1.07 s | no | 46 | Table or toggle only (sections 2 to 4) | REC |
| 0x4E | sfx | 7 | 1 | 1.32 s | no | 48, 51 | Table or toggle only (sections 2 to 4) | REC |
| 0x4F | sfx | 7 | 1 | 0.24 s | no | 45 | Table or toggle only (sections 2 to 4) | REC |
| 0x50 | sfx | 7 | 1 | 0.61 s | no | 68 | Table or toggle only (sections 2 to 4) | REC |
| 0x51 | sfx | 7 | 1 | 0.22 s | no | 60 | Table or toggle only (sections 2 to 4) | REC |
| 0x52 | sfx | 7 | 1 | 0.38 s | no | 52 | hit sound id 9 (routine 0x962A, alternates with the other code of the pair) | REC |
| 0x53 | sfx | 7 | 1 | 0.24 s | no | 47 | Table or toggle only (sections 2 to 4) | REC |
| 0x54 | sfx | 7 | 1 | 0.58 s | no | 87 | Table or toggle only (sections 2 to 4) | REC |
| 0x55 | sfx | 7 | 1 | 0.35 s | no | 55 | Table or toggle only (sections 2 to 4) | REC |
| 0x56 | sfx | 7 | 1 | 1.04 s | no | 39 | Mess punch-4 bank 0 (table 0x9A74). Also the ending at 0x8ADC6 (d1=0x50020, d2=0x00800000) and pool B 82 | REC as the voice. Ending MISS |
| 0x57 | sfx | 3 | 1 | 0.26 s | no | 43 | No path (section 7) | MISS |
| 0x58 | sfx | 7 | 2 | 4.42 s | no | 44 | Enemy death when +$63 > 0, routine 0x97D2 | REC |
| 0x59 | sfx | 4 | 1 | 0.77 s | no | 62 | 0xa318 | MISS |
| 0x5A | sfx | 4 | 1 | 0.79 s | no | 58 | Class 10, port class10_ai.rs | REC |
| 0x5B | sfx | 3 | 1 | 0.75 s | no | 54 | No path (section 7) | MISS |
| 0x5C | sfx | 5 | 1 | 0.24 s | no | 76 | 0xa124 | MISS |
| 0x5D | sfx | 7 | 1 | 0.54 s | no | 82 | Table or toggle only (sections 2 to 4) | MISS |
| 0x5E | sfx | 5 | 1 | 0.42 s | no | 93 | hit sound id 6 (routine 0x962A, alternates with the other code of the pair) | REC |
| 0x5F | sfx | 5 | 1 | 0.34 s | no | 104 | Hit id 11, Butcher chop, Zeke slam 0xA0CC | REC |
| 0x60 | sfx | 5 | 1 | 0.88 s | no | 96 | Roar helper 0xA09E. Also 0x54CBC, 0x68B64, 0x5ED94 | MISS |
| 0x61 | sfx | 5 | 1 | 3.77 s | no | 80 | A11, A46, 0x9946a, 0x9c3ce | MISS |
| 0x62 | sfx | 7 | 1 | 0.26 s | no | 101 | Table or toggle only (sections 2 to 4) | REC |
| 0x63 | sfx | 5 | 1 | 0.34 s | no | 104 | hit sound id 4 (routine 0x962A, alternates with the other code of the pair) | REC |
| 0x64 | sfx | 5 | 1 | 0.13 s | no | 73 | Table or toggle only (sections 2 to 4) | REC |
| 0x65 | sfx | 5 | 1 | 0.29 s | no | 91 | Table or toggle only (sections 2 to 4) | REC |
| 0x66 | sfx | 5 | 1 | 0.60 s | no | 99 | Hit id 5, explosion 0x9F12, item blast, Hogg bomb | REC |
| 0x67 | sfx | 7 | 1 | 0.53 s | no | 78 | Table or toggle only (sections 2 to 4) | MISS |
| 0x68 | sfx | 7 | 1 | 0.58 s | no | 107 | Table or toggle only (sections 2 to 4) | MISS |
| 0x69 | sfx | 7 | 1 | 0.54 s | no | 105 | 0xa21a | REC |
| 0x6A | sfx | 3 | 1 | 0.74 s | no | 102 | Table or toggle only (sections 2 to 4) | REC |
| 0x6B | sfx | 5 | 1 | 0.75 s | no | 88 | No path (section 7) | MISS |
| 0x6C | sfx | 3 | 1 | 0.53 s | no | 111 | Table or toggle only (sections 2 to 4) | REC |
| 0x6D | sfx | 5 | 1 | 3.77 s | no | 74 | Pool B 61, every 120 frames | MISS |
| 0x6E | sfx | 3 | 1 | 0.64 s | no | 108 | Table or toggle only (sections 2 to 4) | REC |
| 0x6F | sfx | 3 | 1 | 0.53 s | no | 81 | Table or toggle only (sections 2 to 4) | REC |
| 0x70 | sfx | 5 | 1 | 10.28 s | yes | 77 | Ranking wind loop | MISS |
| 0x71 | sfx | 3 | 1 | 0.19 s | no | 112 | Table or toggle only (sections 2 to 4) | REC |
| 0x72 | sfx | 3 | 1 | 0.43 s | no | 94 | Thud 0x9796, with 0xF8 | REC |
| 0x73 | sfx | 5 | 1 | 0.42 s | no | 94 | Class 10 toggle with 0xF9. Also the movie | REC |
| 0x74 | sfx | 3 | 1 | 0.67 s | no | 106 | Table or toggle only (sections 2 to 4) | REC |
| 0x75 | sfx | 5 | 1 | 0.56 s | no | 97 | Hard landing 0x9B9E. Also the movie | REC |
| 0x76 | sfx | 5 | 1 | 0.54 s | no | 97 | Hit id 10 and the brontosaurus foot 0xA288 | REC |
| 0x77 | sfx | 3 | 1 | 1.25 s | no | 69 | 0xa284 | MISS |
| 0x78 | sfx | 5 | 1 | 3.77 s | no | 71 | Jungle maker, pool B 80 | MISS |
| 0x79 | sfx | 4 | 1 | 1.04 s | no | 42 | Enemy death when +$63 is not positive. Villager shot | REC |
| 0x7A | sfx | 3 | 1 | 0.96 s | no | 118 | Biker fall 0xA164, with 0xFB. G.$43E bit 0 | MISS |
| 0x7B | sfx | 5 | 1 | 0.21 s | no | 109 | Blocked hit 0x9732, with 0xFC | REC |
| 0x7C | sfx | 7 | 1 | 0.72 s | no | 67 | Table or toggle only (sections 2 to 4) | REC |
| 0x7D | sfx | 7 | 1 | 0.74 s | no | 64 | Table or toggle only (sections 2 to 4) | REC |
| 0x7E | sfx | 3 | 1 | 0.15 s | no | 72 | Boomerang 0xA21E, with 0x101. G.$43B bit 5 | MISS |
| 0x7F | sfx | 5 | 1 | 1.45 s | no | 95 | Biker ride and kick 0xA128, with 0x90. G.$43E bit 1 | MISS |
| 0x80 | sfx | 7 | 1 | 1.45 s | no | 95 | Ranking sting | MISS |
| 0x81 | sfx | 5 | 1 | 0.67 s | no | 103 | Table or toggle only (sections 2 to 4) | REC |
| 0x82 | sfx | 5 | 1 | 1.56 s | no | 83 | move.l sites 0x37C7E family, 0x6669A family, 0x5ED4A | MISS |
| 0x83 | sfx | 5 | 1 | 0.77 s | no | 86 | 0x5ED7C and 0x697CE | MISS |
| 0x84 | sfx | 5 | 1 | 5.14 s | yes | 70 | No path (section 7) | MISS |
| 0x85 | sfx | 5 | 1 | 10.28 s | yes | 75 | No path (section 7) | MISS |
| 0x86 | sfx | 5 | 1 | 10.92 s | yes | 85 | Lift start | REC |
| 0x87 | sfx | 5 | 1 | 1.33 s | no | 79 | Lift end | REC |
| 0x88 | sfx | 3 | 1 | 0.91 s | no | 114 | Title sting, movie, pool B 14 door. Also 0x6A3E2 | MISS |
| 0x89 | sfx | 5 | 1 | 1.21 s | no | 116 | Container helper 0xA396 | REC |
| 0x8A | sfx | 3 | 1 | 0.46 s | no | 115 | Portrait box, pool B 92. Also 0x91E1C | MISS |
| 0x8B | sfx | 3 | 1 | 0.37 s | no | 117 | Non-wood barrel landing 0xA3C6, with 0x104 | MISS |
| 0x8C | sfx | 3 | 1 | 0.43 s | no | 119 | Table or toggle only (sections 2 to 4) | REC |
| 0x8D | sfx | 5 | 1 | 4.66 s | no | 70 | Pool B 56 dust | MISS |
| 0x8E | sfx | 5 | 1 | 4.10 s | no | 75 | Pool B 62, stage 5 | MISS |
| 0x8F | sfx | 5 | 1 | 1.45 s | no | 95 | 0xa4c6 | MISS |
| 0x90 | sfx | 3 | 1 | 1.45 s | no | 95 | 0xa160 | MISS |
| 0x91 | sfx | 5 | 1 | 2.32 s | no | 77 | No path (section 7) | MISS |
| 0xA0 | sfx | 7 | 1 | 0.48 s | no | 59 | No path (section 7) | MISS |
| 0xA1 | sfx | 7 | 1 | 0.48 s | no | 59 | No path (section 7) | MISS |
| 0xA2 | sfx | 7 | 1 | 0.48 s | no | 57 | No path (section 7) | MISS |
| 0xA3 | sfx | 7 | 1 | 0.48 s | no | 57 | No path (section 7) | MISS |
| 0xA4 | sfx | 7 | 1 | 0.48 s | no | 63 | No path (section 7) | MISS |
| 0xA5 | sfx | 7 | 1 | 0.48 s | no | 63 | No path (section 7) | MISS |
| 0xA6 | sfx | 7 | 1 | 0.72 s | no | 65 | No path (section 7) | MISS |
| 0xA7 | sfx | 7 | 1 | 0.72 s | no | 65 | No path (section 7) | MISS |
| 0xA8 | sfx | 3 | 1 | 0.13 s | no | 73 | Table or toggle only (sections 2 to 4) | REC |
| 0xA9 | sfx | 5 | 1 | 0.15 s | no | 110 | hit sound id 1 (routine 0x962A, alternates with the other code of the pair) | REC |
| 0xAA | sfx | 5 | 1 | 0.18 s | no | 98 | hit sound id 2 (routine 0x962A, alternates with the other code of the pair) | REC |
| 0xAB | sfx | 5 | 1 | 0.27 s | no | 90 | Continue accepted, and hit id 3's other code | REC |
| 0xAC | sfx | 7 | 1 | 0.26 s | no | 61 | Table or toggle only (sections 2 to 4) | REC |
| 0xAD | sfx | 7 | 1 | 0.26 s | no | 61 | Table or toggle only (sections 2 to 4) | REC |
| 0xAE | sfx | 7 | 1 | 0.26 s | no | 66 | Table or toggle only (sections 2 to 4) | REC |
| 0xAF | sfx | 7 | 1 | 0.26 s | no | 66 | Table or toggle only (sections 2 to 4) | REC |
| 0xB0 | sfx | 7 | 1 | 0.83 s | no | 56 | Table or toggle only (sections 2 to 4) | MISS |
| 0xB1 | sfx | 7 | 1 | 0.83 s | no | 56 | Table or toggle only (sections 2 to 4) | MISS |
| 0xB2 | sfx | 7 | 1 | 1.21 s | no | 38 | Table or toggle only (sections 2 to 4) | MISS |
| 0xB3 | sfx | 7 | 1 | 1.21 s | no | 38 | Table or toggle only (sections 2 to 4) | MISS |
| 0xB4 | sfx | 7 | 1 | 1.28 s | no | 89, 92 | Table or toggle only (sections 2 to 4) | MISS |
| 0xB5 | sfx | 7 | 1 | 1.28 s | no | 89, 92 | Table or toggle only (sections 2 to 4) | MISS |
| 0xB6 | sfx | 7 | 1 | 0.90 s | no | 53 | Table or toggle only (sections 2 to 4) | MISS |
| 0xB7 | sfx | 7 | 1 | 0.90 s | no | 53 | Table or toggle only (sections 2 to 4) | MISS |
| 0xB8 | sfx | 7 | 1 | 0.67 s | no | 100 | Table or toggle only (sections 2 to 4) | REC |
| 0xB9 | sfx | 7 | 1 | 0.67 s | no | 100 | Table or toggle only (sections 2 to 4) | REC |
| 0xBA | sfx | 7 | 1 | 1.12 s | no | 41 | Table or toggle only (sections 2 to 4) | REC |
| 0xBB | sfx | 7 | 1 | 1.12 s | no | 41 | A90 | REC |
| 0xBC | sfx | 7 | 1 | 0.34 s | no | 49 | hit sound id 7 (routine 0x962A, alternates with the other code of the pair) | REC |
| 0xBD | sfx | 7 | 1 | 0.34 s | no | 49 | Table or toggle only (sections 2 to 4) | REC |
| 0xBE | sfx | 7 | 1 | 0.24 s | no | 113 | hit sound id 8 (routine 0x962A, alternates with the other code of the pair) | REC |
| 0xBF | sfx | 7 | 1 | 0.24 s | no | 113 | Table or toggle only (sections 2 to 4) | REC |
| 0xC0 | sfx | 7 | 1 | 0.83 s | no | 50 | Table or toggle only (sections 2 to 4) | REC |
| 0xC1 | sfx | 7 | 1 | 0.83 s | no | 50 | Table or toggle only (sections 2 to 4) | REC |
| 0xC2 | sfx | 7 | 1 | 1.25 s | no | 37 | Table or toggle only (sections 2 to 4) | REC |
| 0xC3 | sfx | 7 | 1 | 1.25 s | no | 37 | Table or toggle only (sections 2 to 4) | REC |
| 0xC4 | sfx | 7 | 1 | 1.16 s | no | 40 | Table or toggle only (sections 2 to 4) | REC |
| 0xC5 | sfx | 7 | 1 | 1.16 s | no | 40 | Table or toggle only (sections 2 to 4) | REC |
| 0xC6 | sfx | 7 | 1 | 1.07 s | no | 46 | Table or toggle only (sections 2 to 4) | REC |
| 0xC7 | sfx | 7 | 1 | 1.07 s | no | 46 | Table or toggle only (sections 2 to 4) | REC |
| 0xC8 | sfx | 7 | 1 | 1.32 s | no | 48, 51 | Table or toggle only (sections 2 to 4) | REC |
| 0xC9 | sfx | 7 | 1 | 1.32 s | no | 48, 51 | Table or toggle only (sections 2 to 4) | REC |
| 0xCA | sfx | 7 | 1 | 0.24 s | no | 45 | Table or toggle only (sections 2 to 4) | REC |
| 0xCB | sfx | 7 | 1 | 0.24 s | no | 45 | Table or toggle only (sections 2 to 4) | REC |
| 0xCC | sfx | 7 | 1 | 0.61 s | no | 68 | Table or toggle only (sections 2 to 4) | REC |
| 0xCD | sfx | 7 | 1 | 0.61 s | no | 68 | Table or toggle only (sections 2 to 4) | REC |
| 0xCE | sfx | 7 | 1 | 0.22 s | no | 60 | Table or toggle only (sections 2 to 4) | REC |
| 0xCF | sfx | 7 | 1 | 0.22 s | no | 60 | Table or toggle only (sections 2 to 4) | REC |
| 0xD0 | sfx | 7 | 1 | 0.38 s | no | 52 | hit sound id 9 (routine 0x962A, alternates with the other code of the pair) | REC |
| 0xD1 | sfx | 7 | 1 | 0.38 s | no | 52 | Table or toggle only (sections 2 to 4) | REC |
| 0xD2 | sfx | 7 | 1 | 0.24 s | no | 47 | Table or toggle only (sections 2 to 4) | REC |
| 0xD3 | sfx | 7 | 1 | 0.24 s | no | 47 | Table or toggle only (sections 2 to 4) | REC |
| 0xD4 | sfx | 7 | 1 | 0.58 s | no | 87 | Table or toggle only (sections 2 to 4) | REC |
| 0xD5 | sfx | 7 | 1 | 0.58 s | no | 87 | Table or toggle only (sections 2 to 4) | REC |
| 0xD6 | sfx | 7 | 1 | 0.35 s | no | 55 | Table or toggle only (sections 2 to 4) | REC |
| 0xD7 | sfx | 7 | 1 | 0.35 s | no | 55 | Table or toggle only (sections 2 to 4) | REC |
| 0xD8 | sfx | 7 | 1 | 1.04 s | no | 39 | Table or toggle only (sections 2 to 4) | REC |
| 0xD9 | sfx | 7 | 1 | 1.04 s | no | 39 | Table or toggle only (sections 2 to 4) | REC |
| 0xDA | sfx | 7 | 1 | 0.54 s | no | 82 | Table or toggle only (sections 2 to 4) | MISS |
| 0xDB | sfx | 7 | 1 | 0.54 s | no | 82 | Table or toggle only (sections 2 to 4) | MISS |
| 0xDC | sfx | 5 | 1 | 0.54 s | no | 82 | 0x9dfc | REC |
| 0xDD | sfx | 3 | 1 | 0.42 s | no | 93 | hit sound id 6 (routine 0x962A, alternates with the other code of the pair) | REC |
| 0xDE | sfx | 7 | 1 | 0.26 s | no | 101 | Table or toggle only (sections 2 to 4) | REC |
| 0xDF | sfx | 7 | 1 | 0.26 s | no | 101 | Table or toggle only (sections 2 to 4) | REC |
| 0xE0 | sfx | 5 | 1 | 0.26 s | no | 101 | Non-player landing, and class 20. D4d also heard it in the join strip | REC |
| 0xE1 | sfx | 5 | 1 | 0.26 s | no | 101 | Pair of 0xE0. Direct sender 0x9C6E | REC |
| 0xE2 | sfx | 5 | 1 | 0.34 s | no | 104 | hit sound id 4 (routine 0x962A, alternates with the other code of the pair) | REC |
| 0xE3 | sfx | 5 | 1 | 0.13 s | no | 73 | 0x9d94 | REC |
| 0xE4 | sfx | 5 | 1 | 0.29 s | no | 91 | Table or toggle only (sections 2 to 4) | REC |
| 0xE5 | sfx | 7 | 1 | 0.53 s | no | 78 | Table or toggle only (sections 2 to 4) | MISS |
| 0xE6 | sfx | 7 | 1 | 0.53 s | no | 78 | Game-over blood, and bank 2 of 0x9CD8 | MISS |
| 0xE7 | sfx | 5 | 1 | 0.53 s | no | 78 | Item branch of 0x9CD8 | REC |
| 0xE8 | sfx | 7 | 1 | 0.58 s | no | 107 | Table or toggle only (sections 2 to 4) | MISS |
| 0xE9 | sfx | 7 | 1 | 0.58 s | no | 107 | Table or toggle only (sections 2 to 4) | MISS |
| 0xEA | sfx | 4 | 1 | 0.58 s | no | 107 | Item branch of 0x9C76. Also the movie | REC |
| 0xEB | sfx | 7 | 1 | 0.54 s | no | 105 | Table or toggle only (sections 2 to 4) | REC |
| 0xEC | sfx | 7 | 1 | 0.54 s | no | 105 | Table or toggle only (sections 2 to 4) | REC |
| 0xED | sfx | 3 | 1 | 0.74 s | no | 102 | Table or toggle only (sections 2 to 4) | REC |
| 0xEE | sfx | 3 | 1 | 0.74 s | no | 102 | Table or toggle only (sections 2 to 4) | REC |
| 0xEF | sfx | 5 | 1 | 0.74 s | no | 102 | Table or toggle only (sections 2 to 4) | MISS |
| 0xF0 | sfx | 5 | 1 | 0.74 s | no | 102 | 0x9b58 | MISS |
| 0xF1 | sfx | 5 | 1 | 0.75 s | no | 88 | No path (section 7) | MISS |
| 0xF2 | sfx | 3 | 1 | 0.53 s | no | 111 | 0x9b9a | REC |
| 0xF3 | sfx | 3 | 1 | 0.53 s | no | 111 | No path (section 7) | MISS |
| 0xF4 | sfx | 5 | 1 | 0.53 s | no | 111 | No path (section 7) | MISS |
| 0xF5 | sfx | 3 | 1 | 0.64 s | no | 108 | 0x9ff0 | REC |
| 0xF6 | sfx | 3 | 1 | 0.53 s | no | 81 | 0x9e9a | REC |
| 0xF7 | sfx | 3 | 1 | 0.19 s | no | 112 | 0x9c08 | REC |
| 0xF8 | sfx | 3 | 1 | 0.43 s | no | 94 | 0x97ce | REC |
| 0xF9 | sfx | 5 | 1 | 0.42 s | no | 94 | 0xa09a | REC |
| 0xFA | sfx | 3 | 1 | 0.67 s | no | 106 | 0xa030 | REC |
| 0xFB | sfx | 5 | 1 | 0.96 s | no | 118 | 0xa19c | MISS |
| 0xFC | sfx | 5 | 1 | 0.21 s | no | 109 | Table or toggle only (sections 2 to 4) | REC |
| 0xFD | sfx | 7 | 1 | 0.72 s | no | 67 | Table or toggle only (sections 2 to 4) | REC |
| 0xFE | sfx | 7 | 1 | 0.72 s | no | 67 | Table or toggle only (sections 2 to 4) | REC |
| 0xFF | sfx | 7 | 1 | 0.74 s | no | 64 | Table or toggle only (sections 2 to 4) | REC |
| 0x100 | sfx | 7 | 1 | 0.74 s | no | 64 | Table or toggle only (sections 2 to 4) | REC |
| 0x101 | sfx | 3 | 1 | 0.15 s | no | 72 | 0xa256 | MISS |
| 0x102 | sfx | 7 | 1 | 0.67 s | no | 103 | Table or toggle only (sections 2 to 4) | REC |
| 0x103 | sfx | 7 | 1 | 0.67 s | no | 103 | Table or toggle only (sections 2 to 4) | REC |
| 0x104 | sfx | 3 | 1 | 0.37 s | no | 117 | 0xa400 | MISS |
| 0x105 | sfx | 3 | 1 | 0.43 s | no | 119 | 0xa43e | REC |

## 6. Rules

**Attract and demo filter.** Section 0. Implement it in one place, on the flush, not by skipping the `sound()` calls. The game code queues the attract codes either way (D4d 3.11). Coin 0x30 and every 0xFFxx pass. Factory `G.$777C` bit 5 is clear.

**In a game.** Start press sets `G.$4CC = 1` before the select music (D4d). From there the filter is off until game over clears `G.$4CC` again (`flow_game_over` does this).

**Priority and stealing a voice.** Section 0. Practical result: a priority 7 voice shout replaces a priority 5 one on the same voice, and a priority 3 footstep does not replace a shout. Coin is the only priority 10, so it replaces anything on its two voices. Music is masked, not stopped, while an effect holds the voice. Two heroes using the same bank-0 code will cut each other off unless the twin headers use other voices. Open.

**What silences sound.**

| Event | What is queued | What keeps playing |
| --- | --- | --- |
| Each attract state | 0xFF00 via `0xA5D2` (six call sites in `0x97C20`'s range) | Nothing. Stop all clears music and effects |
| Coin during attract | 0x30, and 0xFF00 when not in a game (D4d) | The coin |
| Stage start | 0xFF00, then 0x01 or the area byte | The new music. Effects are cleared by the stop |
| Continue setup | 0xFF00 at `0xEA00`, then 0x1B | Continue music. The in-level continue (the other player is alive) uses the same countdown numbers. D4d did not record a music change for that case |
| Game over modes 0x14 and 0x18 | 0xFF00 at `0xAC2C`. Port sends 0xFF00. Then the ranking path plays 0x80 and 0x70 because the game-over flag is set (D4d) | Ranking audio |
| Stage clear | Fade 0xFF07 from the boss, then 0x18, which replaces music. Effects are not stopped by the fade | Effects already running, until they end or a later 0xFF00 |
| Ending scene changes | 0xFF00 at `0x8A60E` before 0x19, and `0x8B55C` in the ending block | The tune that follows |
| Pause / freeze | No stop command. At `0x5B4`, if `G.$4CC` is not negative and DIP byte `0x80001E` bit 3 is clear, the vblank loops and never reaches the flush at `0x5CC` | The Z80 is not halted, so music that has already started keeps going. D4d says not to port this debug freeze |

R: disassembly of 0x5B4 to 0x5CC, the call lists in section 1.3, D4d.

**Demo.** `0x99E88` / the mode 0xC path calls `0xA55A`, but `G.$4CC` stays 0, so the area music and the demo's effects are dropped unless demo sound is on. R: D4d section 3.11, `jsr 0xA55A` at `0xEF62`.

## 7. Codes with no path

`sound_codes.txt` has an empty sender column for 133 codes. Asset-catalog N8 asked D4e to trace them. They are not 133 unused sounds.

- 91 of the 133 are bytes in the voice, hit, food, landing, bazooka, shotgun or pose tables (sections 2 to 4), including the twins 0xA8 and up.
- 16 more are the first code of a toggle, or a player-weapon row, whose `move.w #twin` is what the catalog listed: 0x65/0xE4 (`0x96CE`), 0x67/0xE5 and 0x68/0xE8/0xE9 (`0x9CD8` and `0x9C76` player rows; 0xE6 and 0xEA are the item codes and already had senders), 0x5D/0xDA/0xDB (`0x9D9C` player row), 0x7B/0xFC (`0x9732`), 0x7A (`0xA164`), 0x7E (`0xA21E`), 0x7F (`0xA128`), 0x8B (`0xA3C6`).
- 0x82 and 0x83 are `move.l` immediates (section 4.4).

That leaves 24 codes with no path into `0xA63C`.

| Code | Kind | Prio | Length | Sample | Note |
| --- | --- | --- | --- | --- | --- |
| 0x1E | music | | 3.89 s, no loop | 9, 13, 15, 17, 20, 23, 24, 26, 27, 30 | Length is close to the rip's time-over sting. `0x96450` does not send it |
| 0x26 | music | | 5.55 s, no loop | 5 | The class 42 constant 0x26 is an object variant, not this tune |
| 0x2C | effect | 7 | 0.48 s | 59 | Same sample as 0xA0 and 0xA1. No table |
| 0x2D | effect | 7 | 0.48 s | 57 | Same sample as 0xA2 and 0xA3 |
| 0x2E | effect | 7 | 0.48 s | 63 | Same sample as 0xA4 and 0xA5 |
| 0x2F | effect | 7 | 0.72 s | 65 | Same sample as 0xA6 and 0xA7 |
| 0x3F | effect | 5 | 0.30 s, 4 voices | 24 | Same sample as start 0x31 and continue tick 0x35. The `move.w #$3F` at `0xA614` is the ring-clear counter (`dbra`), not a send |
| 0x57 | effect | 3 | 0.26 s | 43 | |
| 0x5B | effect | 3 | 0.75 s | 54 | |
| 0x6B | effect | 5 | 0.75 s | 88 | Same sample as 0xF1. A `moveq #$6B` hit at `0x6A6C5` is the middle of a branch, not an instruction |
| 0x84 | effect | 5 | 5.14 s, loops | 70 | Same sample family as used loops (0x8D uses sample 70). No enqueue |
| 0x85 | effect | 5 | 10.28 s, loops | 75 | Same sample as 0x8E, which pool B 62 does send |
| 0x91 | effect | 5 | 2.32 s | 77 | Same sample as the ranking loop 0x70. No enqueue |
| 0xA0, 0xA1 | effect | 7 | 0.48 s | 59 | Twins of 0x2C |
| 0xA2, 0xA3 | effect | 7 | 0.48 s | 57 | Twins of 0x2D |
| 0xA4, 0xA5 | effect | 7 | 0.48 s | 63 | Twins of 0x2E |
| 0xA6, 0xA7 | effect | 7 | 0.72 s | 65 | Twins of 0x2F |
| 0xF1 | effect | 5 | 0.75 s | 88 | Twin sample of 0x6B |
| 0xF3 | effect | 3 | 0.53 s | 111 | Same sample as the wade pair 0x6C / 0xF2, but not in that toggle |
| 0xF4 | effect | 5 | 0.53 s | 111 | Same |

R: `sound_codes.txt`, the table bytes, the disassemblies in sections 1 and 4. Do not wire these 24 until a caller is found. 0x2C to 0x2F and 0xA0 to 0xA7 look like a voice bank that nothing indexes.

## 8. Port tasks

T4.5 is the bundle. Each line is one agent task, about the size NEW-PLAN uses (1 to 2 hours, or a fraction of one). The sound driver (play the sequence, honor priority, pan, fade) is M4 and is not in these hours. "REC" lines need no new call. They need the driver to honor the id, and later the d1/d2 they do not store yet.

**Already calling `sound()`.** Do not add a second call.

| What | Where |
| --- | --- |
| 0xFF00, episode 1 code 0x01, area and lock bytes, 0xFF07 | `flow.rs` `flow_stage_start`, `flow_area_music`, `flow_game_over`, the area-end hook |
| Voice banks, jab, hurt, death shout, special, dash, punch 3, punch 4, splash player row, wade, foot, hard landing, landing player row, thud | `snd.rs` `snd_player`, `snd_9796`, called from `player.rs` |
| Hit ids | `snd_hit` from `player_combat.rs` and `enemy_combat.rs` |
| Enemy death 0x58 / 0x79, extra life 0x4A | `snd_enemy_death`, `hud.rs` |
| Container, break, food, bazooka, shotgun, blast 0x66, 0x89 | `item/shared.rs`, `small.rs`, `container.rs`, `blast.rs` |
| Lift 0x86 / 0x87 | `camera.rs` |
| Class 10: 0x5A, 0x73 / 0xF9 | `class10_ai.rs`, `class10_lib.rs` |
| 0xE0 / 0xE1 | `class20_lib.rs` `snd_9c20` |
| Blocked hit, blade hit | `lib32.rs`, `class14_parts.rs`, `player_combat.rs` |

**New hooks.**

| Id | Work | File and routine | Hours |
| --- | --- | --- | --- |
| S1 | Attract filter on the flush. Drop a normal code when not in a game, unless it is 0x30 or demo sound is on. Still record the call | The future flush (M4). Rule is `0x610`. `snd.rs` is where a temporary filter can live | 0.5 |
| S2 | Keep d1 and d2 on every request. Pan mode, pan byte, middle byte, d2. The hit helper uses a2's X | `Game::sound` and every caller. Driver work is M4. Threading the two longs through the game calls | 2 |
| S3 | Front end codes once the screens exist. The screen work is D4d's. This is only the `sound()` lines: title 0x88, ranking 0x80 then 0x70, slides 0x24 and 0x25, select 0x00 / 0x32 / 0x33, coin 0x30, start 0x31, continue 0x1B / 0x35 / 0x1C / 0xAB, game over 0xE6 / 0x1D / 0x1F, story 0x13 to 0x16, tally 0x49, ending 0x19 / 0x1A and the lab codes | `flow/front.rs`, `flow/join.rs`, `flow.rs` modes 8, 0x10, 0x14, 0x1C | 2.5 across those screens, not one sitting |
| S4 | Boss init themes: 0x20 then 0x0E, 0x21 then 0x0F and the 0x5F chop, 0x10, 0x23 then 0x11, 0x12, class 94's 0x0F, fade `0xA5EA` on the death each doc already names | Each boss file when that boss is ported. Not `flow_boss_defeated` | 0.5 once the inits exist |
| S5 | Clock 0x34 with the window in section 4.2 | Class 31, with D4b. Not `rank.rs`'s HP clear | 1 |
| S6 | GO 0x3E at the two class 20 sites | Inside D4c T-C3 | 0.25 |
| S7 | Class 96: fade, then 59 frames later the `0xA582` byte (timer starts at 0x3C and the fade frame uses one count). Do not use the lock table on this path | The area-change object. Today `flow_area_music` runs at once | 0.5 |
| S8 | Victory pose queues 0x18 | Script action 0x04, D4c. Player state 0x18 is still MISS (D2a) | 0.5 |
| S9 | Pose voice `0x9814` (table `0x985C`) on the 1UP pose, beside the 0x4A jingle that already plays | `player.rs` when states 0x18 / 0x418 / 0x41A exist | 0.5 |
| S10 | Stage 7, lock 2, boss `+$7D == 0xFE`: index 8, code 0x0F. The port comment in `flow_area_music` says this is not tracked, and the code leaves the index at 7 (0x12) | `flow.rs` `flow_area_music` | 0.25 |
| S11 | Biker `0xA128` / `0xA164` (needs a `G.$43E` toggle), barrel `0xA3C6`, boomerang `0xA21E`, roars `0xA09E`, foot `0xA288` | When classes 70, 65, 19 and the dinosaurs are ported | 0.5 for the four helpers. The classes themselves are other docs' hours |
| S12 | Player rows of `0x9C76`, `0x9CD8`, `0x9D9C` if a hero-owned shot calls them. Items must keep 0xEA, 0xE7, 0xDC | `item/shared.rs` | 0.5, only after a caller with `+$82 == 0` is confirmed |
| S13 | Stop hardcoding `tag = 0` in `snd_player` if a non-player must use `0x9AF4` or `0x9C0C`. Class 20 already covers 0xE0 / 0xE1 | `snd.rs` | 0.5 if a caller shows up |

Do not add 0xFF05. Nothing calls it. 0xFF0B belongs to boot init, with the rest of boot (D4d), not to a level hook.

`flow_area_music` already indexes `0xA582` and `0xA5C8` through the asset text (`text.rs` lists both). NEW-PLAN's "episodes 2 to 8 panic on the music table" describes the port when that plan was written. This task did not boot a stage to re-check. Heard audio is still missing either way.

## 9. Acceptance scenarios

Not run. Another job was using MAME. Watch the sound ring at RAM `0xFF803A`, write index `0xFF8034`, read index `0xFF8036`. Each entry is the code word, then d1, then d2.

1. Stage start, stage byte 0: the ring gets 0xFF00, then 0x01 with d1 = 0 and d2 = 0 (`0xACCA`). It does not get 0x02 on that frame. After an area change, episode 1 area 1 queues 0x03.
2. Stage start, stage byte 1 to 7: 0xFF00, then the area 0 byte of `0xA582` (0x05, 0x07, 0x09, 0x17, 0x03, 0x04, 0x0C).
3. Class 96: 0xFF07 with d1 = 0x04440000, then 59 frames after that fade frame the `0xA582` byte, d1 = 0, d2 = 0. The object word `+$80` is set to 0x3C and counted down, including once on the fade frame. No read of `0xA5C8` on this path.
4. Set `G.$7B4` to 2 and call `0xA55A`. The code is the `0xA5C8` byte for that stage. Stage byte 7 and the object at `0xFF0F44` with `+$7D = 0xFE`: code 0x0F, not 0x12.
5. Attract, `G.$4CC` = 0, demo-sound bit clear: a queued 0x88 is dropped in the flush. A queued 0x30 is copied to the Z80. A queued 0xFF00 is copied.
6. Continue setup: 0xFF00, then 0x1B. Each tick 0x35 with d1 = 0x10. When the countdown byte is 9, 0x1C (`0xF47C`).
7. Ending: `0x8A618` is 0x19 after a stop. `0x8A7A4` is 0x1A.
8. A punch chain on Jack, banks clear: first jab is 0xA8 (toggle bit was 0), then 0x38. Punch 3 queues 0x4F. Punch 4 queues 0x50. With a second player so Jack's bank is 1, those become 0xCA and 0xCC.
9. Two container breaks: 0xF5 then 0x6E, or the other order if `G.$43B` bit 1 was already set. Bit flips each call.
10. Clock word changes while `+$A2` of the class 31 object is non-zero: code 0x34, d1 = 0x10, d2 = 0. On a change at :30 with `+$A2` clear, `+$A2` becomes 0x12C and that frame does not beep.
11. Victory script action 0x04: code 0x18.
12. Hit offset 0: the ring gets music 0x00. Confirm whether any attack record stores offset 0 before treating that as a real event.

## 10. Open items

1. Time over has no music send. 0x1E matches the rip's length and has no caller. Left unassigned on purpose.
2. Music 0x26 has no sound caller. Do not take class 42's constant as this tune.
3. Team-move lines and explosive-kill lines (mechanics-seen M25) are web only. The team start does not call a voice routine. `0x9F12` sends 0x66 and no voice.
4. The freeze at `0x5B4` does not silence the Z80. A port pause has to decide whether to stop music. That choice is not in the ROM.
5. Twin voice headers: whether bank 1 uses a different hardware voice is unchecked. Equal priority will cut the bank 0 code off if they share a voice.
6. Hit id 0 is music 0x00. No store to `G.$43C` was found, so the second code of every hit pair may be dead. A write that uses another addressing mode was not ruled out.
7. Which enemies have `+$63 > 0` (death code 0x58 instead of 0x79) is not listed.
8. `0xA5DE` (0xFF05, stop music only) has no caller. Do not invent a use.
9. Demo-sound factory default: ROM and D4d say off. The supplement's bootleg DIP list says on.
10. Album boss intros (Boss 1 0:15, Boss 4 0:47, final 0:35) are not pinned to 0x20 or 0x23. Those two are talk beds.
11. The 24 codes in section 7. The 0x2C to 0x2F family may be an unused voice row.
12. Join-strip 0xE0 / 0xE1: D4d heard them. The strip routine was not disassembled, so they may just be `0x9C0C`.
13. Morgue has no music line in D3d. Morgan sends 0x0E.
14. The class that sends 0x82 and 0x83 (the 0x37Cxx and 0x666xx blocks, and `0x5ED4A`) was not named.
15. A few `jsr 0xA63C` sites take d0 from an object byte rather than an immediate. `0x6A6E4` does `moveq #0` and then `move.b $2D(a6), d0` before the call (the same shape as `0x6A6D2`). Those computed codes were not listed one by one.
16. Sequence lengths are shorter than the CD on most area themes. The comparison in 1.6 uses the sequence length, not a recording.
17. Whether episodes 2 to 8 still panic inside `flow_area_music` was not booted. The source reads the extracted tables and does not show that panic.
