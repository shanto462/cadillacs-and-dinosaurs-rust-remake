# D4d: front end and story (boot, attract, coin, select, join, continue, game over, ranking, ending, link scenes)

Written 2026-10-12 by design agent D4d. Input to milestones M1 (boot to attract loop), M2 (coin, select, join, continue,
game over, tally) and M3 (link scenes, ending, ranking) of `plan/game/NEW-PLAN.md`. This file only describes. No code was
changed. No ROM bytes or ROM text are copied: strings are named by what they show, tables by address and size.

**Source tags.** `R:` repo file or ROM address (68000 address, `G.$xxxx` = RAM `0xFF8000 + xxxx`, `G.-$xxxx` = RAM
`0xFF8000 - xxxx`; I always give the absolute RAM address too). `M:` MAME probe run (batch and segment names are listed in
appendix A; frames are video frames from power on, or `t` frames after a `jump` line). `W:` web. MAME wins over the web.
Every number below that has an `M:` tag was measured by me in this task, not copied from older notes.

Port state words: DONE, PARTIAL, STUB, MISSING (as in `plan/game/README.md`).

## 0. What changed compared to the older notes (read first)

These facts corrected `attract-and-front-end.md` and `README.md`. Details are in the sections named in the last column.

| # | Old note | Correct fact | Section |
| --- | --- | --- | --- |
| 1 | "0x98330 draws the ranking" | 0x98330 draws only the region warning picture, and the World set skips it (region word G.$32 = 2). The BRAVES ranking screen is routine 0x1516C, called through 0x983D8. | 3.5 |
| 2 | "No name entry exists" | A name entry exists (routines 0x15AE2, 0x15BE4). It runs after the continue countdown ends (game over) and after the ending, if the player's score is in the top 10. The old test used a poked score of 500000 which gave an extra life, so the player never reached game over. | 7 |
| 3 | "Coin during the countdown speeds it up" | A coin only adds a credit. Pressing attack or jump speeds the continue countdown up (one number per press). | 6.3 |
| 4 | "62 frames per number" | The first number (20) lasts 100 frames, every other number 60 frames. | 6.3 |
| 5 | "The ending may depend on heroes or players" | The scenes, texts and timing are fixed. With 2 or 3 players only the random explosion details of the first scene differ (the random number generator is used by the extra players). | 8 |
| 6 | "MVP bonus only with 2 or 3 players" | The M.V.P. bonus (10000) shows with 1 player too. It goes to the player with the most damage dealt (ties share it). | 9.1 |
| 7 | "The 1 in 128 task at the title does something" | It is an anti-tamper check (scans the vblank code for patched words). On the real ROM it never fires. Same for 0xD860 (tally) and 0x1F18A (first story panel): ignore. | 3.3 |
| 8 | "The SELECT PLAYER strip is a big strip over the level" | Join and continue use a small strip inside the player's own HUD block (face, name, red seconds number, rating word). | 6.2 |
| 9 | Story panels have a random 1 in 16 branch | Same anti-tamper pattern (0x1F18A). Ignore. | 9.3 |
| 10 | "Join costs 1 credit and gives only a bazooka" | Join and continue also add stage time (+3:30 / +4:00, capped at 4:00), add 600 frames to the wave timer G.$50C, and add 1 point to the score of all three players. | 6.2 |
| 11 | "The story panels are fixed pictures with fixed text" | The panels of Episodes 4, 5, 6 and 7 change their portrait or the hero's name with the heroes in play, and every panel can be skipped with the attack button (a present player). | 9.3 |

## 1. How the front end is built in the original

### 1.1 Tasks and modes

The game runs cooperative tasks, one run per frame, in slot order (`re/specs/CONTEXT.md`). Slot numbers below are the `d0`
value of the task calls (slot k at RAM `0xFF0052 + 16k`).

| Slot | Task | Entry | Alive when |
| --- | --- | --- | --- |
| 0x00 | System task: coin and start logic, credit text, TEST button | 0x9D048 | always, from boot |
| 0x10 | Boot task, then the attract task; in a game the join/continue task; in test mode the test menu | 0xA69A (boot, only once), 0x97C20 (attract), 0x6C4A (join), 0xA6CE8 (test) | one at a time |
| 0x20 | INSERT COIN blink task during attract (0x98874), the press-start screen (0x978EE), or the game task | 0x98874 / 0x978EE / 0xA870 | |
| 0x30 | Stage intro (0xB9BE) or stage clear tally (0xB97E) | | in a game |
| 0x40 | Fade task (0x8824 fade in, 0x8840 fade out) | `palette.rs` ports it | |
| 0xF0 | Skip-wait task: while scripted control is on (G.$4D8 bit 0) and a present player holds attack or jump, every sleeping task wakes one frame early (scenes run about twice as fast) | 0x9E38C | always |

R: `a69a`, `9d048`, `a870`, `9e38c`. The boot task `0xA69A` loads the hi-score table (0x1504E), reads the EEPROM into
G.$777A/B/C (0xAC794), sets the sound output flag, clears the screens, writes the region word G.$32 = 2 (0xA736), creates
the system task and the skip-wait task, initialises the three player objects, shows the two boot screens (0x9DCF6) and jumps
to the attract task. If DIP C bit 7 (0x80001E) is clear it jumps to the test mode instead.

Game task modes (G.$4D0 at 0xFF84D0, steps of 4, jump table at 0xA958; R: `a958`):
0 start (0xA978), 4 character select (0xAD46), 8 story panel (0xD898), 0xC in level (0xAAF6), 0x10 continue (0xE9E6),
0x14 game over (0xAC06), 0x18 game over 2 (0xAC26), 0x1C ending (0xAC48 then 0xA69B8). Sub modes: G.$4D1 (0xFF84D1),
G.$4D2 and G.$4D3 (bytes at 0xFF84D2, 0xFF84D3) are the nested steps of the screens below.

The attract and demo are **not** a game mode: G.$4CC (0xFF84CC, "in game") is 0 and mode stays 0 or 0xC while a demo runs.
G.$4CC becomes 1 when a player presses start (system task, 0x9D15C), before the select screen. M: d4d-b2/c-title (ingame
0 to 1 at frame 601 for a start at 600).

### 1.2 RAM map of the front end (absolute addresses)

| Address | Name | Meaning | R/M |
| --- | --- | --- | --- |
| 0xFF8016, 17, 18 | G.$16..18 | cooked input of player 1..3: bit0 right, 1 left, 2 down, 3 up, 4 attack, 5 jump | R: 0x738 |
| 0xFF801B, 1C, 1D | | same, previous frame | |
| 0xFF801A / 0xFF801F | G.$1A / G.$1F | start buttons bit0..2 (P1..P3), bit6 TEST, bit7 SERVICE / previous | R: 0x9D326 |
| 0xFF8020 / 0xFF8021 | | coin inputs bit0..2 / coin lockout outputs | R: 0x9D4BE, 0x9D40E |
| 0xFF8028 | G.$28 | layer control shadow: 0x12D6 (level), 0x1396 (title and cards), 0x18D6 (attract task top) | M: d4d-b1 |
| 0xFF8032 | G.$32 | region word: 0 Japan, 1 USA, 2 ETC (the World set writes 2) | R: 0xA736 |
| 0xFF8034 / 36, ring at 0xFF803A | | sound request ring: write offset, read offset, 64 entries of 16 bytes (code word, d1 long, d2 long) | R: 0xA63C |
| 0xFF844A | G.$44A | fade counter (15 down to 0), fade tasks wait for 0 | R: 0x879A |
| 0xFF846A | G.$46A | credits (common credit pool modes) | R: 0x9D386 |
| 0xFF846B / 6C / 6D | | free play flag / "2 coins start" flag / press-start screen is up | R: 0x9CFC6 |
| 0xFF8470 / 71 | | coins needed per credit / credits per coin batch | R: 0x9CFC6 |
| 0xFF8472 / 74 | | common chute flag / 2 player common chute flag | |
| 0xFF8473 | | "redraw the credit text" request | |
| 0xFF8475 | | coin count in common chute mode | |
| 0xFF8477 | | prompt hold timer (2 or 3 after a coin or start) | |
| 0xFF847A, 0xFF848A, 0xFF849A | | per player coin block, 16 bytes: +0 debounce timer (0x32), +1 coin count, +2 shift register, +3 credits, +4 previous count | R: 0x9D4BE |
| 0xFF84AA | G.$4AA | debug flag, set by holding P1 up, right, attack, jump and coin 1 at power on | R: 0xA6C0 |
| 0xFF84CD | G.$4CD | players that pressed start (bits) | |
| 0xFF84CF | G.$4CF | 2 player cabinet layout | |
| 0xFF84D5 | G.$4D5 | HUD hold (join prompts off) during intros | |
| 0xFF84D7 / DB | G.$4D7 / G.$4DB | no continue possible / credits are ignored | |
| 0xFF84E4 | G.$4E4 | players that chose a hero (bits) | |
| 0xFF84E8 | G.$4E8 | stage clock, BCD m:ss word | R: 0x10490 |
| 0xFF850C | G.$50C | wave timer (7200 frames at area start) | R: 0x10490 |
| 0xFF8507 | G.$507 | ending flag: name entry is parked while 1 | R: 0xA6A16, 0x8A7D4 |
| 0xFF8510 | G.$510 | game mode saved while the continue mode runs | R: 0x6C9C |
| 0xFF85E4 | G.$5E4 | name entry running (1 or 2) | R: 0x73A6 |
| 0xFF862C | | character select struct (48 bytes) | R: 0xAD36 |
| 0xFF86C4 / 86C8 | | scroll1 camera X / Y shadow | |
| 0xFF8744 / 87E4 / 87E8 | | scroll2 camera X / scroll3 X / scroll3 Y shadows | |
| 0xFF0FD0 | G.-$7030 | attract state byte (0, 2, 4, 6) | R: 0x97C20 |
| 0xFF0FD2 | G.-$702E | game over flag: the attract shows the ranking once | R: 0xAC20 |
| 0xFF0FDE | G.-$7022 | title frame counter (0 to 600) | R: 0x9A3CE |
| 0xFF0FE8 / EA / EC / EE / F0 | G.-$7018.. | profile counter (to 0x4B0), timeline index, scroll flag, shake flag, shake base | R: 0x9A7E6 |
| 0xFF0FFC / FE | G.-$7004 / -$7002 | movie counter (to 0x5C8), timeline index | R: 0x9BF50 |
| 0xFF0EE4, F04, F24 | G.-$711C.. | name entry blocks of player 1, 2, 3 (32 bytes each) | R: 0x15AE2 |
| 0xFFF012 / 0xFFF018 | G.$7012 / G.$7018 | ranking screen state / wait counter | R: 0x1516C |
| 0xFFF03E | G.$703E | hi-score table, 50 entries of 16 bytes | R: 0x1504E |
| 0xFFF35E | G.$735E | intro task and tally task struct; player tally blocks at 0xFFF37E, 39E, 3BE (32 bytes) | R: 0xB9BE |
| 0xFF84F6 | G.$4F6 | frames waited with a credit outside a game | R: 0x1827A |
| 0xFFF57A / 57C | G.$757A / G.$757C | demo counter / frames of play left | R: 0x99F10 |
| 0xFFF58A + 0x10 * who | | per player demo playback state (+0 stream pointer long, +4 frames left in the record) | R: 0x9A146 |
| 0xFFF65A (128 bytes) | G.$765A | EEPROM image; G.$777A/B/C (0xFFF77A, 7B, 7C) are the cooked settings | R: 0xAC794 |
| 0xFFF7A4, A5, A6 | G.$77A4.. | input bytes the players read (copied from G.$16..18 each frame, overwritten by the demo playback) | R: 0x18834 |

## 2. Boot (screens A1 and A2)

**What happens.** R: `0x9DCF6` (called by the boot task `0xA69A`). Two text screens in blue small font on black, region
dependent. M: d4d-b1/attract, d4d-b14.

| Step | Frames (M: d4d-b1) | What | R |
| --- | --- | --- | --- |
| Power on | 0 to 72 | black, the program clears RAM and sets up the video registers | 0x41A |
| Screen 1 | text drawn at about frame 73 (fade counter set at 72), shown 121 frames at most | three centred lines: game name, date code 930201, region word (ETC for the World set) | 0x9DDB2 table, texts 0x9DE18 (JAPAN), 0x9DE58 (USA), 0x9DEDA (ETC) |
| Screen 2 | about 196 to 317 | the legal WARNING text, region version (11 lines for ETC: sales, export and operation "inside these countries" are not allowed) | 0x9DDAC table, texts 0x9DF5C, 0x9E098, 0x9E204 |
| Boot ends | layer control changes at frame 317, title counter starts at 322 | | |

Rules.
- Each screen waits 30 frames (not skippable), then up to 91 more frames in 1 frame steps. Any attack or jump press
  (mask 0x30) of player 1 or player 2 ends the wait at once. Player 3 cannot skip. R: `0x9DD80`. M: d4d-b14/boot-skip-a
  (press at 100: both screens end at 226), boot-skip-b (press of player 2 at 230: ends at 234), boot-skip-p3 (no effect).
- The colour of the text depends on the region word: three scroll1 palettes of 32 bytes at 0x9DDB8 (white to grey
  for Japan, red for USA, blue for ETC). The World set always uses the blue one. R: `0x9DCF6` copies palette
  `G.$32 * 0x20` to 0x914400.
- A coin or a start press during the boot screens works (the system task runs from boot): the boot task is killed and the
  press-start screen (section 3.9) appears at once. M: d4d-b2/c-boot (coin at 100, credit at 110, start at 260 gives the
  select screen at 283).
- Debug flag G.$4AA (hold player 1 up, right, attack, jump and coin 1 at power on) turns the DIP switches into cheats
  (starting stage from G.$22, invincibility, speed, boxes) and enables a pause. R: `0xA6C0` (sets it), read at `0x594`, `0x974`, `0xA4C`, `0x6422`, `0xA978`, `0x1037E`, `0x1055C`, `0x105CA` and by the
  task `0x9E38C`. Not needed for the port (section 10).

Port: MISSING (`flow_boot` only loads the hi-score table). Size: tiny (three text blocks and a wait).

## 3. The attract loop (nobody plays)

### 3.1 State machine

R: attract task `0x97C20`, state byte G.-$7030 at 0xFF0FD0 (values 0, 2, 4, 6), table `0x97C88`. Each pass of the task:
sleep 2 frames, G.$4CC = 0, scroll1 palette set 0 (0x89E6), brightness (0x892C), cameras to 0, sleep 1, G.$28 = 0x18D6,
then `0x97C94` (if the game over flag 0xFF0FD2 is set: palettes, stop sound, clear all, `0x983D8` = show the ranking once,
clear the flag), then jump to the routine of the state:

| State | Routine | Parts in order | Then |
| --- | --- | --- | --- |
| 0 and 4 | 0x97D84 | stop sound (0xFF00), palettes, clear all, fade, **title** (0x9A3CE), **profiles** (0x9A7E6), warning picture (0x98330, skipped), clear all, **ranking** (0x983D8) | state += 2 |
| 2 | 0x97CC0 | stop sound, **title**, 0x98330, demo counter +1 and **demo** (0x99E88), kill task slot 0x20 (the blink task), **story slides** (0x988C0), stop sound, clear all, fade, **ranking** | state += 2 |
| 6 | 0x97D02 | stop sound, palettes, clear all, fade, **title**, 0x98330, demo counter +1 and **demo**, clear all, **ranking**, **opening movie** (0x9BF50) | state = 0 |
| 8 and 0xA | 0x97D42, 0x97DBE | ranking only, then state 0 | not reachable |

So the cycle is 0, 2, 4, 6, 0, 2, ... Demos 1, 2, 3 ... alternate between state 2 and state 6. The story slides follow every
demo made in state 2 and the opening movie follows every demo made in state 6. R: `97c20`..`97dc2`.
The state byte is **not reset** when a game starts and ends: after game over the attract continues in the state it was in
(it restarts that state at its title). M: d4d-b16/go-attract (game started in state 0, after game over the loop shows the
ranking once, then title, profiles, ranking, state 2 at the next demo).

### 3.2 Frame timeline, measured

All numbers are video frames from power on, no input, first pass. M: d4d-b1/attract, d4d-b17/att-counters.

| Part | Start | End | Length | Remarks |
| --- | --- | --- | --- | --- |
| Boot screens | 73 | 317 | 244 | section 2 |
| State 0: title | 322 (counter 1) | 921 (counter 600) | 600 | music code 0x88 queued at 331 (title + 9) |
| profiles | 923 (counter 1) | 2320 (counter 0x4B0) | 1398 | the counter ticks 1200 times, the handlers add waits (3.4) |
| ranking | 2322 | 2744 | 423 | stop sound at 2322, music 0x80 at 2324, ambient 0x70 at 2400 |
| State 2: title | 2746 | about 3345 | 600 | |
| demo 1 (Episode 2, area 0) | 3350 (first game frame) | 4555 (last game frame) | 1205 video frames, 1201 game frames | RAM clear and setup 3345 to 3352 (3.6) |
| frozen last picture, then fade out | 4555 | 4670 | 100 + 15 | |
| story slides | 4672 (music 0x24) | 6449 | 1778 | 3.7 |
| ranking | 6451 | 6874 | 424 | |
| State 4 (as state 0) | 6874 | 9311 | 2437 | title, profiles (counter 1 at 7482), ranking (8888 to 9311) |
| State 6: title | 9311 | about 9916 | 600 | |
| demo 2 (Episode 3, area 0) | 9922 | 11126 | 1205 | |
| frozen picture and fade | 11126 | 11242 | 116 | |
| ranking | 11243 | 11666 | 424 | |
| opening movie | 11669 (counter 1) | 13411 (counter 0x5C8) | 1743 | 3.8 |
| State 0 again | 13411 | 15848 | 2437 | |

Loop length: state 0 = 2437, state 2 = 4130 (4131 the second time), state 4 = 2437, state 6 = 4100. One pass of the four states is
**13104 frames** (about 218 seconds at 60 Hz). Because demos rotate through six rows, the whole sequence repeats after three
passes (39312 frames). Jitter between passes is 1 frame. M: d4d-b1 (state changes at 2744, 6874, 9311, 13411, 15848, 19979, 22416,
26516, 28953).

### 3.3 Title screen (routine 0x9A3CE, 600 frames)

R: `0x9A3CE`. Used by states 0, 2, 4 and 6 (parameter d0 is not used).

1. Setup: pools reset (0x49A8, 0x4A18, 0x4A46, 0x4C48, 0x4C76), floor table (0x12BE4), spawn reset (0xFBE8), counter G.-$701C = 0,
   cameras 0, clear all, palettes set 0, fade in (0x892C).
2. The anti-tamper roll: one chance in 128 calls 0x6A73A. That routine scans 102 words of the vblank code (0x542) for a branch or
   nop word. The ROM has none, so it never fires. **Do not port** (it would start a copy of the title task in slot 0x30).
3. Create the blink task `0x98874` in slot 0x20: it draws INSERT COIN (or FREE PLAY when G.$46B is set) at column 18, row 56
   (scroll1 base 0x90A000, text strings 0x988A4 and 0x988B3) for 30 frames, erases it for 30 frames, forever.
4. Layer control G.$28 = 0x1396. Four 16 by 8 tile blocks (routine 0x98B86: code list 0x9A5E6 plus 0x5800, attribute list 0x9A6E6)
   at scroll2 addresses 0x90F000, 0x90F020, 0x90F400, 0x90F420 make the green background; routine 0x980B0 adds more.
5. Two pool A objects of class word 0x188 (class 98, handler 0x6A20A), variants 0 and 1, make the big logo (the "Cadillacs"
   letters and the "Dinosaurs" letters) as sprites. The Capcom sign appears with the first text (counter 30), probably drawn by the 0x110C call [I].
6. Frame loop (600 passes): sleep 1, screen shake task (0x222D6), pool B (0x23604), pool A (0x23420), sprite list (0x13FA6),
   counter +1.
   - Counter 30 (frame 30 of the title): print the two small lines (string block 0x97F64, printer 0xBB6): the "tales based upon
     the comic" line (col 11, row 20) and the quoted title line (col 12, row 22); then 0x110C (d0 = 0x10, d1 = 0x1A).
   - Counter 360 (0x168): kill the blink task, clear scroll1, print the licence block 0x97F9E (5 lines of the car maker's
     licence text at col 4 rows 20 to 24, plus 2 copyright lines at col 15 rows 26 and 27). For regions other than ETC (G.$32 != 2)
     a third copyright line (0x98094, rows 28) is printed too. After frame 360 INSERT COIN is gone.
7. Return after counter 600.
Sound: code 0x88 is queued 9 frames after the title starts (title tune). Without the demo sound setting it is not heard (3.11).
Port: MISSING.

### 3.4 Character profiles (routine 0x9A7E6, counter 1200 ticks, 1398 frames)

R: `0x9A7E6`, tables `0x9A8E4` (frames) and `0x9A908` (handlers). A small timeline machine: per frame call the scroll helper
(0x9AC86, moves the checker board: scroll2 X += 4 or 8 per frame, scroll3 X += 4, wrap at 0x100) and the shake helper (0x9ACBA),
object loops, counter +1 (word at 0xFF0FE8); when the counter reaches the next table entry, call the handler. The handler index
is the table index plus 1 (the code increments the index before it reads the handler).

| Counter | Handler | Action |
| --- | --- | --- |
| 9 | 0x9A94E | none |
| 10 | 0x9A950 | **card 1 (Jack)**: clear scroll1, sound 0x13 (music), print the label block 0x9AC16, then the card text 0x9B0F8 (tall font, 6 segments), create 2 pool B objects (class 93 var 0, class 94 var 0) and 2 pool A objects (class word 0x190 = class 100, var 0 and var 4), print the labels 0x9ABF6 (CM, KG) and 0x9AC02 (HEIGHT, WEIGHT) |
| 11 | 0x9A94E | none |
| 308 | 0x9A92E | fade out (0x87BA, d0 = 1) and wait |
| 309 | 0x9ABDC | clear all, sleep 30 |
| 310 | 0x9A9FE | **card 2 (Hannah)**: same with text 0x9B192, variants 1 and 5 |
| 311 | 0x9A93E | fade in (0x879A) and wait |
| 608, 609, 610, 611 | 0x9A92E, 0x9ABDC, 0x9AA98, 0x9A93E | fade out, clear, **card 3 (Mustapha)** text 0x9B224 (variants 2 and 6), fade in |
| 908, 909, 910, 911 | 0x9A92E, 0x9ABDC, 0x9AB32, 0x9A93E | fade out, clear, **card 4 (Mess)** text 0x9B2C2 (variants 3 and 7), fade in |
| 1190 | 0x9A92E | fade out |
| 1200 (0x4B0) | | the loop ends (the table entries 2200 and 10000 are never reached) |

Card order on screen is Jack, Hannah, Mustapha, Mess (the variant numbers follow the screen order, not the hero ids).
Each card shows: a big portrait on a checker board (blue, red, green, orange), the hero's full name, the labels HEIGHT and WEIGHT with
two numbers (cm and kg), two job lines and one red skill word (GOOD ABILITY, ITEM SKILLED, FLYING KICK, HIGH POWERED), and on the
right a small animated hero that performs a few moves. All text is stored as tile codes (no letter decode is needed): the numbers
use their own tile codes. M: d4d-b10/att-profiles pictures, d4d-b1.

Real frame times (the waits inside the handlers are why 1200 ticks last 1398 frames): M: d4d-b1
- Card 1 draws at frame 932 (counter 10). Sounds queued at 953, 962, 984, 1002 (code 0x38, four hit sounds of the small hero),
  1036 (voice 0x4B), 1092 (pose sound 0x40). Fade out ends 1245, fade in ends 1294.
- Card 2: 0x38 at 1318, 1327, 1345, 1361; voice 0x4C at 1393; pose 0x41 at 1457. Fades end 1606 and 1655.
- Card 3: 0x38 at 1679, 1693, 1708, 1731; voice 0x4D at 1755; pose 0x42 at 1807. Fades end 1967 and 2016.
- Card 4: 0x38 at 2040, 2052, 2070, 2092; voice 0x4E at 2106; eight times 0x43 at 2170 to 2177 (the muscle pose). Fade out ends 2310.
These offsets come from the animation scripts of the pool A object (class 100), they are not in a table.

Port: MISSING. Data to extract: the 4 card text blocks (0x9B0F8 to 0x9B342, 586 bytes), labels 0x9ABF6 to 0x9AC4A (84 bytes), the two
timeline tables (0x24 bytes each), and the object classes B93, B94, A100 (pool dispatch entries 93, 94, 100; asset catalog "A99,
character portraits" and the class 100 animation tables).

### 3.5 Ranking screen "BRAVES" (routine 0x1516C, about 423 frames)

R: `0x1516C` (called by `0x983D8`, which first kills the blink task in slot 0x20, and sets G.$6C8 = 0 after). A task-like loop
with a state byte G.$7012 (0xFFF012): 0 setup, 2 draw, 4 wait, 6 end.

- State 0: pools reset (0x49A8, 0x4A46, 0x4A18, 0x4C76, 0x4C48, 0x4CA4, 0x8C96C).
- State 2: scroll1 palette set 4, fade in, stop sound (0xA5D2), clear scroll1, print the label block 0x15508 (BRAVES at col 18,
  row 3; 1ST to 10TH at col 2, rows 6, 10, 12, 14, 16, 19, 21, 23, 25, 27, the 10TH at col 0), scroll2 page 0x16 four times at
  0x90F000, 0x90F400, 0x90F800, 0x90FC00 (loader 0x204CA), scroll3 page 0x7C eight times at 0x913800 to 0x913F00 (loader 0x20592),
  scroll3 Y = 0x40. Then 10 rows are drawn by 0x15594: row y offsets from the table 0x153CA (4, 2, 2, 2, 3, 2, 2, 2, 2, 2), the name
  (3 letters from the 4 byte tag), the score (8 digits, leading zeros blank), a face icon (2 by 3 tiles by the character id, tables
  0x156A8 and 0x156E8) and the stage digit followed by two small marks (tiles for "-" and "R", two columns to the right, rows 1 and 2 below). One more object is spawned: pool B class 0x158 (class 86), the
  Cadillac that drives across, started at the camera X minus 0x80.
- State 4: wait 0x1A4 = 420 frames (counter G.$7018 at 0xFFF018). Each frame the scroll2 camera X += 8 and the scroll3 camera
  X += 4 (the desert scrolls past), objects are updated (0x23420, 0x23604, 0x237B2, 0x13FA6).
- State 6: fade out; stop sound.
Sounds: stop sound (0xFF00) on entry, music 0x80 two frames later, ambient 0x70 about 78 frames after that. M: d4d-b1 (2322, 2324, 2400).
The table shown is G.$703E (section 7): at power on it holds the default list (scores 100000 down to 10000, then more default
entries below the first ten, names from 0x150A4: C P, SYS, TEM, DA., YKI, TEZ, CKY, DNA, JRA, SOM). New entries from the name entry
(section 7) show here. M: d4d-b5/end-hiscore (new 1ST row, old rows move down).
Port: PARTIAL (the table and rank text exist in `hud.rs`: `hud_hiscore_init`, `hud_rank_*`; the screen is MISSING).

### 3.6 The demo (routines 0x99E88 and 0x99F10)

**Selection.** R: `0x97D6C` and `0x97D78` add 1 to the demo counter word G.$757A (0xFFF57A) and call `0x99E88`. `0x99F10` uses
`counter & 7` as the row if the counter is below 6, else sets it to 0. The first demo uses counter 1, the second 2, and so on; after
counter 5 the next demo wraps to row 0. M: d4d-b1 (counter 1 at frame 3345, 2 at 9916, 3 at 16450, 4 at 23021, 5 at 29555).

| Counter | Demo | Episode and area (stage G.$4D9, area G.$786) | Team in the 3 player layout (P1, P2, P3) | Team in the 2 player layout (P1, P2) |
| --- | --- | --- | --- | --- |
| 1 | state 2 (first loop) | Episode 2 area 0 (stage 1, area 0) | Jack, Mustapha, Mess | Jack, Mustapha |
| 2 | state 6 | Episode 3 area 0 | Hannah, Mustapha, Mess | Jack, Mess |
| 3 | state 2 (second loop) | Episode 8 area 1 | Jack, Hannah, Mess | Hannah, Mustapha |
| 4 | state 6 | Episode 5 area 0 | Jack, Mustapha, Mess | Hannah, Mess |
| 5 | state 2 (third loop) | Episode 6 area 1 | Hannah, Mustapha, Mess | Mustapha, Mess |
| 0 (after 5) | state 6 | Episode 1 area 1 | Jack, Mustapha, Hannah | Jack, Hannah |

Rows are at 0x9A1E2 (3 player, 6 offsets then rows of: stage byte, area byte, frames word 1200, then 3 words per player: stream
index in bytes, who and hero id word, class word = hero id times 4) and 0x9A1EE (2 player, shares the area up to 0x9A2DE). Hero ids
0 Jack, 1 Mustapha, 2 Hannah, 3 Mess. The demo plays with the 3 player table unless the DIP selects the 2 player layout (G.$4CF).

**Setup `0x99F10`.** R: it saves the attract state word, clears the game RAM (0xFF0E66 to 0xFF8000, the small blocks 0xFF8000 to 0xFF8016,
0xFF8034 to 0xFF844A, 0xFF84AC to 0xFFEEDE, 0xFFEF1E to 0xFFF00E, 0xFFF35E to 0xFFF57A including the sound ring), restores the attract
state, clears all, clears 31 words from G.$757C, decodes the settings (0xA804), 0xFF2E, 0xA836, rank start (0x17F8A, G.$4F6 is 0 so rank
band 0), extra life selector (0x6B88), sets G.$4D7 = 1 (no continue), initialises the three player objects (0x4D46, 0x4D5E, 0x4D76),
marks player 1 and 2 (and 3 in the 3 player layout) present (+0 = 1, G.$4CD bits, G.$4E4 = G.$4CD) and calls `0x18C88` for each (sets the
hero's attack table pointer, clears speeds, state 0x5BDC). Then G.$4D6 = 1, **the random generator is reset**: G.$4 to G.$E = 0 and
G.$14 = 0x1C3 (so the demo replays exactly the same every time), the row is read, G.$757C = 1200 (the frames of play), per player
`0x9A0CE` stores the stream pointer and first count (+4), the who and hero word (+2) and the class (+$20), position fields cleared,
frame counters G.$0, G.$2, G.$4DA = 0, then `0xAC6A` (the end of the stage start routine: 0xA9BE stage setup, mode = 0xC, 0x655E, HP 100,
rank speed, music), `0x2228A` (scroll controller init), G.$5EC and G.$5EE = 0, G.$602 = 1 (rank frozen), HUD redraw `0x162AE`, area
music code (0xA55A, muted in attract). **The mode byte becomes 0xC here**, which is why the log shows mode 0 to 0xC at frame 3352.
M: d4d-b1 (mode 0 to 0xC at 3352, 9924, 16458).

**Frame loop `0x99E88`.** Each pass: sleep (G.$4D6 and 15) frames, G.$0 and G.$4DA +1, object update (0xE90E), collisions (0x10546),
HUD messages (0x16BEA), sprite list (0x13FA6), list reset (0x105E8), spawn script (0xFBF4), G.$757C -1; it repeats while G.$757C is
not negative (1201 passes). After the loop: sleep 100 frames (the last picture stays), fade out (0x87BA, d0 = 1), clear bit 0 of G.$26.
For Episode 6 (stage 5) the blink task in slot 0x20 is killed at the start of the demo, so there is no INSERT COIN text in that demo.
All other demos show the blink text at the bottom centre. The HUD shows the three hero blocks with names and bars (all full, score 0
at the start). The demo has no continue and no join prompts (G.$4D7 = 1).

**Input playback `0x9A146`.** R: called by the player update (0x18834) for each player when G.$4CC is 0. The pointers 0x9A13A..0x9A144
give the input byte addresses 0xFFF7A4, A5, A6 (G.$77A4..6); the player code reads these bytes. Each player has a playback state at
0xFFF58A + 0x10 * who: +0 pointer to the current record, +4 frames left. A record is two bytes: the input byte (bit0 right, bit1 left,
bit2 down, bit3 up, bit4 attack, bit5 jump, same as G.$16) and a frame count. Each frame: count -1; when it reaches 0 the pointer moves to
the next record, the new count is loaded, and the record's byte is written to the input byte. A stream ends with the record (0, 0) (never
reached: the streams are 1458 or 1758 frames long, the demo plays 1201). Stream index words in the rows give the byte offset into the
pointer tables 0x9A2DE (3 player layout, 18 pointers) and 0x9A326 (2 player layout, 18 pointers, 12 distinct). All 30 distinct streams
lie in 0xC869A to 0xC9EE6 (6220 bytes, 136 records on average). The first record is "no input" for 60 to 255 frames (the opening scene
runs), the buttons used are only bits 0 to 5. R: `9a146`, tables decoded by `re/out/d4d/tools/rom.py`.
There is also a recording mode (bit 0 of G.$24, stream tables at 0x9A36E in VRAM space) used by the developers: ignore.
Port: MISSING. The level code already has the same per-frame order (`flow.rs game_frame`), so the demo needs: the row and stream
extraction, the setup (including the RNG reset and the exact RAM that the setup clears, because many objects start from zeroed state),
the playback hook in the player input, and the end (freeze 100 frames, fade). The levels of the demo rows must exist (M0, M5 to M8):
until they do, the demo is checked only for its first frames (acceptance 13.1).

### 3.7 Story slides (routine 0x988C0, 1778 frames)

R: `0x988C0`. A straight script with sleeps (no counters). Used only in state 2. Music code 0x24 is queued at the start (frame 4672),
code 0x25 at the start of the fifth slide (6077, 60 frames before the blank). M: d4d-b1, d4d-b10/att-slides, d4d-b12/slides-fine.

| # | Frames (first pass) | Picture (pages) | Text | Sleeps in the script |
| --- | --- | --- | --- | --- |
| 1 | 4672 to 5010 | scroll2 pages 0x0D, 0x0E at 0x90F000, 0x90F400; scroll3 pages 0x6A, 0x6B at 0x913800, 0x913900; G.$7E8 = 0x10 | two blocks 0x98CC6 and 0x98D14: the villagers cry for help, the dinosaurs rarely attack people | 30, text, 60, text, 60, 180 (about 330 frames) |
| 2 | 5010 to 5326 | block 8 by 6 at 0x90F1D4 (lists 0x98BD6 and 0x98C4E), no fade: a hard cut | blocks 0x98D7E and 0x98DE2: the heroes decide the black marketeer is behind it | 60, 60, 180 then fade out (16) |
| 3 | 5326 to 5640 | scroll2 pages 0x10, 0x11; scroll3 pages 0x64, 0x71 (jungle) | blocks 0x98DE8 and 0x98E4A: a poacher tells a helper to hurry with the skinning | fade in 16, 30, 60, 180, fade out 16 |
| 4 | 5640 to 6077 | block at 0x90F1D4 again (lists 0x98BD6 and 0x98C4E) plus a 4 by 3 block at 0x913828 (lists 0x98C36 and 0x98CAE), cameras slide left (scroll3 X minus 8 and scroll2 X minus 2 per frame for 49 frames) | blocks 0x98E50 and 0x98EDA: a heroine at a window does not understand why they do it | 30, 49 slide frames, 60, 60, 180 |
| 5 | blank 6080 to 6200, then 6200 to 6449 | scroll3 pages 0x6D, 0x6E (the villain's face) | blocks 0x98F2A and 0x98F8A: the villain says they keep messing with the balance and he will crush them | blank, 0x98EE0 text, 60, 60, pages, 127 frames of sleep 1, 60, 60 |

(The blocks are tall font strings printed by `0x84F72`; picture loaders `0x204CA`, `0x20592`, block printers `0x98B86`, `0x98BAE`.)
Fades are 16 frames (fade counter 15 to 0). The slide boundaries in the table come from picture differences every 10 frames
(M: d4d-b12/slides-fine: cuts at 5010, fades at 5320 to 5340 and 5610 to 5630, camera slide change at 5730 to 5750, blank 6080, new
picture 6200, ranking at 6450). The exact frame of each text appearance follows from the sleeps.
Port: MISSING. Size: one script plus 6 picture loads and 10 text blocks (about 2 agent hours with the services of T1.1).

### 3.8 Opening movie (routine 0x9BF50, 1743 frames, 1480 timeline ticks)

R: `0x9BF50`, same timeline engine as the profiles: counter G.-$7004 (0xFF0FFC) to 0x5C8, table `0x9C002` (frames) and `0x9C026`
(handler offsets, index plus 1). Two helper routines run every frame (`0x9C7C4`, `0x9C7F8`: they scroll the jungle layers when the
flags G.-$7000 and G.-$6FFE at 0xFF1000 and 0xFF1002 are set). The waits inside the handlers add 262 frames to the 1480 ticks.
M: d4d-b17 (counter 1 at 11669, reaches 0x5C8 at 13411), d4d-b10/att-movie, d4d-b12/mov-fine, d4d-b1 (fades and sounds).

| Counter | Handler | Action (R) and what the screen shows (M, paraphrased) | Frames seen (M) |
| --- | --- | --- | --- |
| 10 | 0x9C04A | cameras 0, fade out, clear, print text block 0x9CBB2 (first caption: about 400 years ago science nearly made new life), fade in | fade 11693 to 11710, text until 11900 |
| 200 | 0x9C088 | fade out, clear, print block 0x9CDEC (the long typed crawl: they went too far, nature takes care of itself, key words "THE GREAT CATACLYSM" in red) | 11900 to 11932, text until 12232 |
| 500 | 0x9C21C | fade out, clear, scroll2 X = -12, two tile blocks 0x9C280 and 0x9C28C (the CAPCOM sign and a small one) and a 0x110C print ("presents"), fade in | 12232 to 12264, shown until 12364 |
| 600 | 0x9C29C | call 0x81888, fade out, spawn pool B class 32 (0x80), scroll2 pages 3, 4, 3, 3 at 0x90F000..0x90FC00 and scroll3 page 0x72 three times (the jungle), fade in, spawn pool A class 11 (0x2C) variants 0 and 1 (the man) | fade 12364 to 12394, sound 0x61 (footsteps) from 12394, then every 8 frames from 12480 to about 12530 |
| 850 | 0x9C3D4 | flag G.-$7000 = 1 (layer scroll starts) | strong scroll from 12650 |
| 890 | 0x9C5CE | print block 0x9CBE6 (his caption: "wow, that one is huge") | |
| 895 | 0x9C3DC | flag G.-$6FFE = 1 | |
| 900 | 0x9C52A | spawn pool A class 12 (0x30) variant 1, two objects | sound 0x73 at 12698 and 12706 |
| 930 | 0x9C46C | spawn pool A class 12 variant 0, three objects (the raptors, pool A class 12 is the golden velociraptor of the asset catalog) | |
| 1100 | 0x9C426 | spawn pool A class 30 (0x78) variant 1 | sound 0x75 at 12895, 12959, 13023 |
| 1120 | 0x9C5C6 | flag G.-$6FFE = 0 | |
| 1140 | 0x9C5BE | flag G.-$7000 = 2 | |
| 1150 | 0x9C5AC | print block 0x9CCD4 (second caption: "this is the wild ...") and flag G.-$7000 = 0 | caption 13080 to 13220, sound 0x36 (roar) at 13089 |
| 1250 | 0x9C3E4 | spawn pool A class 69 (0x114) variants 0 and 1 and print block 0x9CC64 | brown dinosaur 13150 to 13340 |
| 1350 | 0x9C5D8 | print block 0x9CC9C, spawn pool A class word 0x188 variants 0x10 and 0x11 (the game logo pieces), change the logo palettes (helper 0x9C818 from palette ROM 0xBCA52 and 0xC2252), then sleep 120 | logo 13320 to 13380, sound 0x88 (title tune) at 13274 |
| 1450 | 0x9C788 | fade out, clear | black 13364 to 13394 |
| 1500 | 0x9C79C | never reached (the loop ends at 0x5C8 = 1480) | |

After the movie the attract task sets the state to 0 (13411) and the next title starts: stop sound (13411) and the title tune again at 13426.
So **the movie does use sound codes**: 0x61, 0xEA, 0x73, 0x75, 0x36 and the title tune 0x88. They are muted without the demo sound setting.
M cuts (10 frame steps): crawl texts 11840 to 12260, CAPCOM 12260 to 12380, jungle 12400 to 12950 (the picture changes 50 percent every
10 frames from 12650 because the layers scroll), green raptor about 12960 to 13140, brown dinosaur 13150 to 13340, logo 13320 to 13380,
black 13390, title at 13420.
Port: MISSING. 17 handlers, 7 object classes (pool A 11, 12, 30, 69, 98 (0x188), pool B 32). Size: 2 tasks (text and CAPCOM part,
jungle part).

### 3.9 Press-start screen (routine 0x983EE, task 0x978EE)

R: `0x983EE`: clear all, palette set 0, create the screen task `0x978EE` in slot 0x20, layer control 0x12D6 then 0x1396,
clear scroll1, scroll2/3 base shadows 0x90C0 and 0x9100, brightness `0x892C` (d1 = 0x38). The task `0x978EE`: pool resets, G.-$7034 = 0,
cameras 0, palette 0, fade in, G.-$703A/-$7038/-$7036 = 0, four 16 by 8 blocks at 0x90F000, 0x90F020, 0x90F400, 0x90F420 (code list
0x97A20, attribute list 0x97B20: the same green as the title), two pool A objects class word 0x188 variants 2 and 3 (the logo), and
the usual object loop. It has no timer and no end: only the system task ends it (start) or the TEST button. M: d4d-b2/c-nostart
(9000 frames, nothing changes).
The prompts are printed by the system task (section 4.4): per player "PUSH START BUTTON" when that player has a credit, else
"INSERT COIN", and "CREDIT n" below. Look: `fe-title-credit` (capture recipes).

### 3.10 What a coin or a start press does at each moment

M: d4d-b2 (14 segments). Frames are from power on.

| Moment | Coin | Start with a credit | Start without credit |
| --- | --- | --- | --- |
| Boot screens (73 to 317) | credit at +8 or +9 frames after the coin signal begins (the coin must be seen for at least 2 frames and be released); sound 0x30 (coin) and 0xFF00; the boot task is killed; press-start screen appears | select screen about 22 frames later | nothing |
| Title, profiles, ranking, demo, slides, movie | same (attract tasks in slots 0x10 to 0x40 are killed, `0x983EE`, `G.$46D = 0xFF`); the press-start screen is visible after a fade of 15 frames. The demo is not finished, the RAM it set is simply not used again. M: d4d-b2/c-demo (coin 4000, screen at 4030) | sound 0x31 (d1 = 0x10) at +1 frame, G.$4CC = 1, fade out 15 frames, the game task starts at about +19 (player +0 = 1, G.$4D5 = 1, G.$46D cleared), mode 4 (select) at +23 | nothing, no sound |
| Press-start screen | more credits; sound 0x30 each | same | nothing |
| Free play (G.$46B = 1) | not needed | the game starts at once from any attract moment (no press-start screen), 22 frames to the select screen. The screens show FREE PLAY instead of INSERT COIN. M: d4d-b15 | same |

Credit cap: 9 (`0x9D3C8`). With separate chutes each player has own credits; start of a player uses that player's credit.
Port: MISSING (system task is a stub).

### 3.11 Sound rules in attract and demo

R: the sound flush in the vblank routine (`0x610` to `0x69E`): one entry of the ring is sent per frame, when the Z80 is ready. A code whose high
byte is 0xFF (0xFF00 stop, 0xFF05, 0xFF07 fade, 0xFF0B reset) is always sent. Any other code is sent only if G.$4CC is not 0 (in a game,
including the press-start-to-select moment), **or** the code is exactly 0x0030 (the coin sound), **or** the DIP setting "demo sound" is on
(G.$777C bit 5). The factory value has demo sound **off**, so the title, the profiles, the demo and the movie are silent and only the
coin sound is heard. The codes are still queued by the game code, so the port should keep calling `sound()` and filter in one place.
M: d4d-b1 (the ring holds the codes of every part, see tables above).
Codes seen in the attract: 0xFF0B at 43 and 70, 0xFF00 at 70 and 317, 0x88 title tune (331, 2755), 0x13 and 0x38/0x4B..0x4E/0x40..0x43
(profiles), 0x80 ranking music (2324) and 0x70 (2400), 0x24 slides music (4672), 0x25 (6077), the movie codes 0x61, 0xEA, 0x73, 0x75,
0x36, 0x88 (3.8), the demo level music (the same code as the level) and its sound effects, 0x30 coin, 0x31 start.

## 4. Coins, credits and settings

### 4.1 Coin input (R: `0x9D4BE`, called from the vblank routine every frame)

Per player block at 0xFF847A (P1), 0xFF848A (P2), 0xFF849A (P3), coin input bit 0, 1, 2 of G.$20 (0xFF8020):

1. `shift = ((shift << 1) | coin_bit) & 0xF` (byte +2).
2. If the debounce timer (+0) is 0: when `shift == 3` set the timer to 0x32 (50) and stop. Otherwise stop.
3. Timer running: decrement; at 0 stop (the coin took too long); if `shift == 0` clear the timer and stop; if `shift != 0xC` stop.
4. `shift == 0xC` (two frames high, two frames low): **a coin is accepted**: G.$46E +1 (display pulse), the coin count (+1) +1 (common
   chute modes: G.$475 +1), previous count (+4) = old count, G.$473 = 1 (redraw credit text), G.$477 = 2, if not in a game stop the sound
   (0xFF00), queue sound 0x30 (d1 = 0x70010, d2 = 0x3800000), timer cleared.
M: d4d-b2/c-title: coin key held at 450 to 457, shift 1, 3, 7, 15 at 451 to 454, 14 at 459, 12 at 460, accept at **460**. A key tap shorter
than 2 frames is not counted. The probe press needs at least about 3 frames.

Conversion (R: `0x9D3DA` for per player credits, `0x9D386` for the common pool, run every frame by the system task): if the coin count is at
least G.$470 (coins per credit) subtract G.$470 and add G.$471 credits, cap 9. The coin enable outputs in G.$21 (0xFF8021: bits 2, 3, 5 in the common pool modes, one bit per player otherwise) are kept set while the chute may take coins
and cleared (lockout) when credits plus one batch would pass 9, or in free play (`0x9D40E`). The coin counter pulse (G.$46E, G.$46F, G.$21 bit 0 on for 0x20 frames of a 0x3F frame step) comes from `0x9D478`.
SERVICE (G.$1A bit 7 edge): queue the coin sound and add one credit to every player (or the common pool). TEST (bit 6): kill all tasks and start the
test menu (section 10).

### 4.2 Modes (from the EEPROM, decoded at boot by `0xAC794` and `0x9CFC6`)

Coin mode (EEPROM +8, G.$777A bits 0 to 2, table `0x9D038` of (coins, credits) pairs): 0 = 1 coin 1 credit, 1 = 1 coin 2 credits, 2 = 1 coin
3 credits, 3 = 1 coin 4 credits, 4 = 1 coin 6 credits, 5 = 2 coins 1 credit, 6 = 3 coins 1 credit, 7 = 4 coins 1 credit, 8 = "2 coins start, 1 coin
continue" (flag G.$46C, G.$470 = 1, G.$471 = 1; a start or an in-level join takes **2 credits**, a continue takes 1 credit).
Chute mode (EEPROM +$D, G.$777A bits 3 and 4): 0 = three players, three chutes (separate credits per player), 1 = three players, one chute (common
pool G.$46A, flag G.$472), 2 = two players, one chute (common pool, 2 player layout G.$4CF = 1).
Free play (EEPROM +$11, G.$777C bit 2, flag G.$46B): no coins; the screens show FREE PLAY; start works at once.

### 4.3 Settings (EEPROM image G.$765A, 128 bytes; factory image at 0xACBF4)

The EEPROM is checked against an 8 byte header; if it differs the factory image is written (`0xAC9D2`, `0xACABA`) so a fresh MAME run (empty
EEPROM) always starts with the factory values. Cooked bytes (R: `0xAC794`):

| Cooked byte | Bits | Setting | Factory value | Used by |
| --- | --- | --- | --- | --- |
| G.$777A (0xFFF77A) | 7 | sound output: 1 = QSound, 0 = monaural (flag byte at 0xF19FFD) | 1 | boot task |
| | 6 | coin mode is 8 | 0 | |
| | 3 and 4 | chute mode (0, 1, 2) | 0 | system task, join, HUD layout |
| | 0 to 2 | coin mode 0 to 7 | 0 | system task |
| G.$777B (0xFFF77B) | 5 and 6 | extra life option: 0 none, 1 first 300000 next 700000, 2 first 500000 next 1000000, 3 only 1000000 | 2 | score code (0x6B88, table 0x6BFE) |
| | 3 and 4 | "timer level" 0 to 3 (EASY, NORMAL, HARD, VERY HARD in the test menu): rank speed = level + 1 | 1 | rank code |
| | 0 to 2 | difficulty "RANK" 0 to 7 (EXTRA EASY to HARDEST): index of the rank tables | 3 | rank code (0x17F8A) |
| G.$777C (0xFFF77C) | 6 | continue allowed (0 sets G.$4D7 = 1) | 1 | 0xA804 |
| | 5 | demo sound | 0 | sound flush 0x610 |
| | 4 | unused in the front end [U] | 0 | |
| | 2 | free play | 0 | 0x9CFC6 |
| | 0 and 1 | extra lives at start (lives byte +$A3 = this value) | 1 | 0x4DD6 |

So the factory game is: 1 coin 1 credit, three chutes, continue on, no free play, no attract sound, 2 lives in all, extra life at 500000 (and
at 1000000 more), difficulty 3, timer level 1. MAME has no DIP for these, so MAME shows the factory values. R: `re/specs/flow/SPEC.md` section 6
(0x80, 0x4B, 0x41). Port: `Settings` in `flow.rs` DONE for the three cooked bytes.
**Recommendation:** a small config file (TOML or JSON) with the factory values and these named options (coin mode, chute mode, free play, continue,
demo sound, difficulty, timer level, extra life option, extra lives, stereo). No EEPROM emulation and no test menu.

### 4.4 Credit text and prompts (system task, G.$473 request)

R: `0x9D66A` (credit lines), `0x9D752` (prompts), strings at `0x9D984` to `0x9DBC0`, `0x83EB`, `0x841B`, `0x842B`. In the press-start screen
and in the attract the bottom row (scroll1 row 60) shows per player (three columns at 0x90A0F0, 0x90A8F0, 0x90B0F0):
- a prompt in the large 2-tile font: "PUSH START BUTTON" if that player has a credit (or free play), else "INSERT COIN",
- a small line "CREDIT n" (n = credits of that player, one digit),
- in coin modes with more than one coin per credit also "COIN n" (coins waiting) or the line "n COIN(S) m CREDIT(S)".
In the one chute modes there is one "CREDIT n" line in the middle column and the prompts follow the common pool.
For the "2 coins start, 1 coin continue" mode two extra lines explain it.
M: d4d-b2/c-title, c-two (credit n in the column of the player who put the coin), c-three-start. The pictures: `fe-title-credit`.

### 4.5 Per frame order of the system task (R: `0x9D048`)

`0x9D326` (SERVICE, credit text flag), `0x9D40E` (coin lockout), `0x9D478` (coin counter pulse), `0x9D074` (attract: press-start screen and start
detection, section 3.10), `0x1827A` (G.$4F6 counter, see below), `0x9D5A4` (TEST edge), `0x9D066` (statistics: `0xA6B8A` play time counters), sleep 1.
G.$4F6 (0xFF84F6): while not in a game and credits (or free play) exist the counter adds 1 per frame (when it wraps past 0xFFFF it is set to 0x8000); the rank start
code (0x17F8A) reads it as the time the machine waited with a credit: bands below 720, 3600, 10800 frames pick the rank table row
(longer wait gives an easier start) and clears it. `rank.rs` already has the bands and the field `w4f6`; the system task must feed it.
M: not measured (the demo clears nothing of it). Port: STUB (`flow_system_task`).

## 5. Character select (game mode 4, routine 0xAD46)

R: `0xAD36` (clear the struct 0xFF862C), `0xAD46` (jump on G.$4D1: 0 setup `0xAD64`, 2 main `0xAF7C`, 4 end `0xB810`), player step `0xB06A`,
cursor `0xB122`, panels `0xB25E`. M: d4d-b4/sel-1p, sel-1p-moves, sel-2p-late, sel-same, sel-3p-all-left, sel-3p-timeout, sel-picks-sim;
d4d-b2/c-title (start to select).

**Entry.** Start press at frame F: sound 0x31 at F+1; fade out 15 frames; the game task starts at about F+19 (mode 0, `0xA978`: create the join task 0x6C4A in slot 0x10,
clear layers, rank start, then `0xA9BE`); mode 4 at F+23. Sub mode 0 runs `0xAD64`: palette set 1, a fade task (0x8824 in slot 0x40),
scroll1 panel frames (page ids 5 and 6 at 0x90A000 and 0x90B000 for the 3 player layout, pages 3 and 4 for the 2 player layout, loader 0x20454),
scroll2 pages 1 and 2 at 0x90F000 and 0x90F400 (the group picture of the four heroes, loader 0x204CA), scroll3 pages 6 and 9 at 0x913800 and 0x913900,
sound code 0 (select music, d1 = 0xFFFF), the struct is filled (records of 12 bytes at +0xC, +0x18, +0x24 for P1, P2, P3; pending counter +5, shared
timer seconds +9, frames +$A), pool B objects (class 67 variants 4 and 12 at y 80, class 68 variants 0, 4, 8 at y 136, 120, 104 (the cursor tags "1P", "2P", "3P"
[I]); class 19 at x 160 and 352 for the 2 player layout; class 17), then sub mode 2 at F+25 (`sub 0 to 2` at 725 for a start at 700).

**Screen.** The group picture on top, left to right Jack, Hannah, Mustapha, Mess (cursor positions 0 to 3). Top centre "TIME" with a two tile
digit number (digits from the table 0xAF50). At the bottom a panel per player, framed blue, red and orange: face (from the lists at 0xB2D2..), the hero
name (strings 0xB39D..: JACK.T, HANNAH.D, MUSTAPHA.C, MESS.O), three labels POWER, SPEED, SKILL (strings at 0xB3E4) with filled diamonds (tile 0x693) counted
from the table 0xB444 (3 bytes per hero in position order: Jack 4 3 3, Hannah 2 4 5, Mustapha 3 5 3, Mess 5 2 4) and a rating word (from the pointer list 0xB57C for
all regions except Japan, 0xB47C for Japan): GOOD ABILITY (Jack), ITEM SKILLED (Hannah), FLYING KICK (Mustapha), HIGH POWERED (Mess) as in the profile cards.
Empty panels show JOIN-IN (if that player has a credit; with no credit the panel alternates between INSERT COIN and JOIN-IN) in the large font, blinking like
the in-level prompts (6.1), or FREE PLAY. Pictures: `fe-select-1p`, `fe-select-3p`.

**Rules (all M).**
- Start cursors: P1 on Jack (position 0), P2 on Hannah (1), P3 on Mustapha (2). Mess is free. (Hero ids 0 Jack, 1 Mustapha, 2 Hannah, 3 Mess; the table
  at 0xB24A maps cursor position to id: 0, 2, 1, 3.)
- Left or right moves a cursor one step; positions held by other cursors are skipped, with wrap at the ends (3 players all pressing left in the same frame
  rotate without a clash). **Two players can never hold the same hero** (no duplicates are possible at all, not even by pressing at the same frame).
  Sound 0x32 on each move with d1 = the pan of the new position: 0x00, 0x0B, 0x15, 0x20 for positions 0 to 3.
- Attack or jump (mask 0x30 of the player's input) picks the hero under the cursor: sound 0x33 with the same pan (no hero voice is queued at the select, M: all sel segments show only 0x31, 0x00, 0x32, 0x33, 0xFF00 and the level music); the player's object gets the hero class
  (id times 4) and state; the cursor tag stays.
- A player who is not present can press start (with a credit; the credit is used) **during the select**: the player joins with its own panel, and the shared timer
  is reset to 10 with a 10 frame pause. M: sel-2p-late (P2 start at 841: timer 8 to 10, panel appears, both picked at timer 0).
- Timer: starts at 10. The number 10 is shown for 10 frames (725 to 735 in the test), every other number lasts 60 frames (frame counter at 0xFF8636 counts 60 to 1). At 0 the code writes
  0x10 (attack) into the three input bytes G.$16..18, so every player who has not chosen takes the hero under the cursor on the next frame
  (sounds 0x33 for each). M: sel-1p (TIME 10 at 725, 9 at 735, ..., 0 at 1275, auto pick at 1335 = 610 frames after the screen starts), sel-3p-timeout.
- After all present players picked (state 4 of every present record): +60 frames hold, then sub mode 4 (`0xB810`): the picture darkens in 16 steps (palette
  steps every 2 frames), then a fade out task (0x8840) and `jmp 0xAC4E` (stage start). Offsets from the last pick: sub 4 at +61, sub2 2 at +62, sub2 4 at +79, fade out
  +139 to +158, mode 0xC at +158 or +159. M: sel-1p (pick 1335, level 1494), sel-1p-moves (pick 1001, level 1160).
- The level music (code 1 for Episode 1) is queued at the first level frame.

Port: STUB (`flow_select_mode` starts the stage at once). Needs: scroll page loaders, panel drawing, the record machine, the join of new players, 5 object
classes (B19, B17, B67, B68, the hero objects), sounds 0x31, 0x32, 0x33, music 0. 2 player layout panels: [U] not measured (DIP not available in MAME).

## 6. Join in, continue, game over, and every moment a hero can be chosen

The join task `0x6C4A` (slot 0x10, created by mode 0, `0xA978`) runs one state machine per player (routine `0x6D60`, state byte at the player's +$A2, jump
table `0x6D6E`). Player objects: P1 0xFFB274, P2 0xFFB3F4, P3 0xFFB574 (0x180 bytes each). It also switches the game to the continue mode when nobody is
active (`0x6C8A` to `0x6CB0`).

| State | Meaning | Routine |
| --- | --- | --- |
| 0 | absent, no credit: prompt INSERT COIN | 0x6D7C |
| 2 | absent, has a credit (or free play): prompt JOIN-IN, waits for start | 0x6E2A |
| 4 | hero strip (new join or continue), timer B0 and B1 | 0x70F0 |
| 6 | playing; when the player's +0 byte is 0 (dead, no life left): to 8 (continue allowed) or to 0xA or 0xC (no continue) | 0x72EE |
| 8 | continue countdown | 0x732E |
| 0xA | high score name entry | 0x74A4 |
| 0xC | "GAME OVER" label in the HUD block | 0x74E2 |

### 6.1 Prompts in the HUD block of an absent player

R: `0x6E04` (state 0, no credit) uses `G.$4DA and 0x7F` (the game frame counter, byte at 0xFF84DA, 128 frame cycle): value 0 draws the INSERT COIN glyph block
(0x84D6 for the 3 player layout, 0x8514 for the 2 player layout), 0x30 erases it with a blank block (0x843A), 0x3F draws the JOIN-IN block (0x852E, 0x856C in the
2 player layout) and 0x70 erases it. **So with no credit the prompt alternates between INSERT COIN and JOIN-IN.** State 2 (a credit exists) uses `G.$4DA and 0x3F`
(64 frame cycle): 0 draws JOIN-IN (0x8640, or the "n COIN(S) m CREDIT(S)" line 0x8602 in the coin ratio modes, or the one chute blocks 0x8586 and 0x85C4) and
0x30 erases it. State 0xC (game over label) uses the `and 0x3F` rhythm too (blocks 0x867E and 0x8196). The blocks are two row glyph strings (word format, printer
`0xC9A`, attribute 0x18B). The origin is the HUD block of the player (3 player layout: scroll1 columns 0, 16, 32 from row 34). Nothing is drawn while G.$4D5 (HUD hold)
or G.$4DD (scripted control) is set.
M: d4d-b13/blink-nocredit: INSERT COIN visible from the cycle start for 48 frames, blank 16 frames, JOIN-IN for 49 frames, blank 16 frames (measured on the pictures);
blink-credit: JOIN-IN 48 frames on, 16 off (period 64). `mechanics-seen.md` F18 ("48 on, 80 off") described only the first half.

### 6.2 Joining during play (state 2 to 4 to 6)

M: d4d-b3/j-p2 (coin 2 at t = 10, start 2 at t = 60, attack 2 at t = 140), j-p2-wait, j-p23, j-nocredit, d4d-b20.

1. Coin: credit +1 at t = 20 (the key was held 10 to 17), P2 state 0 to 2 at once, sound 0x30. The prompt changes from INSERT COIN to JOIN-IN.
2. **Join request**: the player's own start edge (G.$1A and not G.$1F), **or** the player's own attack edge in the three chute mode (R: `0x6FD4`; in the Japan
   set the attack alternative is off). M: d4d-b20/join-by-attack (attack alone joined), join-by-jump (jump does not). Without a credit nothing happens (M: j-nocredit,
   join-by-attack-nocredit). Free play: always allowed. Cost: 1 credit, or 2 credits in the "2 coins start" mode. The request is ignored while G.$4DB is 1.
3. On the request (R: `0x7072` to `0x70F0`): the player object is initialised (`0x4D46`, `0x4D5E`, `0x4D76`), the HUD block prompt is erased (`0x80A2`), then
   `0x7476` (adds **1 to the score of all three players**, BCD, last byte, cap 99), `0x4DE8` (state long = 0x00020000 "entering", HP 100, facing byte +$24 = 1), `0x1049E`
   (stage clock **+3:30 with a cap of 4:00**, wave timer G.$50C +600 frames), `0x18410` (rank event for a join), statistics `0xA6B74`. Strip timer B0 = 0x11
   (0x06 in the 2 player layout), B1 = 1, G.$4CD bit set, state 4, `0x7F86` draws the strip.
   M: j-p2 (t = 61: P2 hero = Hannah, state 4, B0 = 17; the clock went from 1:00 to 4:00; G.$50C 7199 to 7799; scores of P1, P2, P3 all 1). d4d-b19/time-join,
   time-join-cap (3:50 also becomes 4:00).
4. **The strip** (state 4, R: `0x70F0` and `0x75B2`, `0x7AE4`, `0x7BC8`, `0x7D8E`): only runs when the game is in mode 0xC and the HUD is free (G.$4D5 and G.$4DD both 0
   if anybody is present). It occupies the player's own HUD block: the face of the hero under the cursor (left), his name, a red number (the seconds), the rating word,
   and (3 player layout) two small arrows. M: sheet of j-p2 (t = 80 to 140).
   - Start cursor: the player's previous hero (the byte +3, which is Jack for a player who never played) or the next free one: routine `0x723E` is called twice
     (direction −1, then +1) so that the cursor never stays on a hero held by another player. A hero is held while its player is in state 4 or 6 (strip or playing). A player
     in state 8 (continue countdown) does **not** hold his hero. M: P2 got Hannah (Jack was held by P1).
   - Left and right (`0x71FE` reads the edges) move through Jack, Hannah, Mustapha, Mess in the order of the select screen, skipping held heroes (`0x723E`).
   - Timer: B0 goes 0x11, 0x10, 9, 8 ... 0 (the numbers shown are 10 to 0), B1 = 40 frames per step (`move.b #$28`), the first number 0x10 starts at t + 1.
     Total 1 + 40 + 10 x 40 = 441 frames from the join request to the automatic pick. M: j-p2-wait (request 61, B0 steps at 62, 102, 142, ..., 462, pick at 502).
   - Confirm: the player's attack edge or own start edge; ignored while B0 = 0x10 and B1 >= 0x14 (the first 20 frames after the strip opens, so the opening press
     does not confirm). M: j-p2 (attack at 140 gives the pick at 141).
   - Sound: none for the strip itself (no cursor sound in the level). Sound 0xE1 is queued 16 frames after the number 2 starts (M: j-p2-wait t = 398).
5. **Pick** (R: `0x71DA`): +0 = 1 (active), G.$4E4 bit, state 6, `0x7F86` (HUD block redraw). The hero drops in: P2 position x = camera X + 0xC0 (192), height 0x180
   (384, falls), later z = 64 and the usual landing state 0x200; invulnerability counter +$EC = 360 frames (the player blinks, state byte +0 = 2 until it ends, then 1);
   a weapon in hand: the **bazooka** (`item_give_bazooka` 0x65A4, ported); lives = the extra lives setting (1); HP 100. A speech bubble appears over the hero. M: j-p2 (P2 at
   t = 141: x 192, y 384, held 38964 (0x9834, the bazooka object), rinv 360; at t = 600 +0 = 1).
6. Two or three players can join the same frame; each gets own strip. M: j-p23.
Port: STUB (`flow_join_task`, `flow_join_prompts`). Ported already: `player_join`, the HUD block code, `item_give_bazooka`.

### 6.3 Dying and the continue countdown

When a player's HP is 0 and lives are 0 the player object's +0 becomes 0 (state 6 sees it, R: `0x72EE`): the player is removed from G.$4CD and G.$4E4. If continue is allowed
(G.$4D7 = 0): state 8, B0 = 0x14 (20), B1 = 0x64 (100). Otherwise the player goes to the name entry check (7.2) at once.

**(a) In the level while another player is still active.** M: d4d-b3/k2-timeout (P1 dies at 214, state 8 at 215).
- The HUD block of the dead player shows "CONTINUE?" and the number (R: `0x7EE8`, `0x7E70`). The number is B0, 20 down to 0: the first number 100 frames, then 60 frames each, so
  the countdown ends 1240 frames after it starts (215 to 1455 in the test). The level goes on for the other players.
- Pressing attack or jump (edge of the player's `$A8` bit 4 or 5) with B0 at 18 or less (R: `0x734E`) sets the frame counter back to 60 and subtracts 1 from B0 at once: **button
  mashing skips the countdown**. The first two numbers (20 and 19) cannot be skipped.
- At 0 (60 frames after the 0 is shown): state 0xC, the label GAME OVER in the block for 256 frames (it ends sooner when the player presses attack or start), then state 0
  (INSERT COIN or JOIN-IN again). M: k2-timeout (state 12 at 1515, state 0 at 1771).
- A start press (or, quirk, the player's own attack press while any start edge exists in the same frame) with a credit (R: `0x73BE`, `0x7426`): credit −1, G.$4CD bit,
  player init `0x4DE8`, lives `0x4DD6` (= extra lives setting), **stage clock +4:00 capped at 4:00 and G.$50C +600** (`0x10490`), statistics, `0x7476` (+1 score for all
  three), rank event `0x18566` with d0 = 0xFE, then the strip (state 4, section 6.2 from step 4). The player's score is kept. M: d4d-b3/k2-coin, d4d-b19/time-cont (1:00 became 4:00).

**(b) Everybody is dead: the continue mode (game mode 0x10, routine 0xE9E6).** The join task switches the mode when the last player has left G.$4CD (`0x6C8A`: G.$4CD = 0 and mode below 0x10),
or when no player is in state 4 or 6 and at least one is in state 8 (`0x6CB0` to `0x6D46`, which also sets G.$4D5 = 1 and G.$5E4 = 0). It saves the mode and sub mode in G.$510 and sets mode 0x10, sub 0.
With continue disabled (G.$4D7 = 1) the countdown word is set to −1 at once (`0xEB18`), so the game over part runs immediately.
M: d4d-b3/k-1p (death at t = 202, mode 0x10 at 203). R: `0xE9E6` (jump on G.$4D1: 0 setup `0xEA00`, 2 countdown `0xEB60`, 4 game over `0xEBA6`).
- Setup (sub 0, one frame): stop sound, scroll1 palette set 1, brightness, clear scroll1 and sprites, **the scrolls of the level are saved** (G.$744, 748, 7E4, 7E8 into
  the struct at 0xFF85CC), scroll2 pages 0x18 and 0x19 at 0x91F000 and 0x91F400 (the villain), scroll3 pages 0x75 and 0x76 at 0x923800 and 0x923900, red background
  (palette flashing), the countdown number 20 (word 0xFF85D4 = 0x14, frame counter +9 = 100), sound 0x1B (continue music). The HUD blocks of absent players keep blinking at the top.
- Picture: a villain with a pistol on a red background, a speech bubble on the left that changes its sentence at 20 (luck ran out), 15 (how dare you), 10 (I will feed you to
  the Shivat) and 5 (eat lead), and a big two tile number on the right. The strings are tile lists (table 0xF57C for the digits). M: sheet of k-1p.
- Countdown (sub 2, `0xEB60`, `0xF41E`, `0xF4BA`): the number 20 lasts 100 frames, every other number 60 frames. Each tick queues sound 0x35 (d1 = 0x10). When the number
  becomes 9 the music switches (sound 0x1C). **Attack or jump of any player skips one number per press once the number is 17 or less** (R: `0xF4BA`; M: d4d-b3/k-mash: with a press
  every 4 frames from t = 300 the beeps come every 4 frames from t = 425 and the countdown ends at 493). The first three numbers (20, 19, 18) cannot be skipped. Coins during the
  countdown only add credits (M: k-coin2). Start without a credit does nothing (M: k-nocoin-start).
  Timeline for no input (M: k-1p): 203 start, 302 (19), 362, 422, ..., 902 (number 9, sound 0x1C), ..., 1502 (0).
- **Continue** (start press with a credit while the number is above 0, R: join task state 8, `0x73BE`): sub 2 to 6 at once, the saved scrolls are restored [I], mode 0xC
  again about 41 frames after the press (the level is exactly where it was, enemies and all), the player is in state 4 (strip) with a credit −1, lives 1, HP 100.
  Sounds: stop (0xFF00) +2, jingle 0xAB +18, level music (the area's code) at the switch. M: d4d-b3/k-coin-late (start at 1341, strip at 1341, sound 0xFF00 1343, 0xAB 1359, music 3
  and mode 0xC at 1382, strip numbers 40 frames each, pick by timeout at 1823, P1 active). The strip is the same as in 6.2, the stage clock is topped up the same way (+4:00).
- **Game over** (number reaches 0 at t = 1502, sub 2 to 4, `0xEBA6`, G.$4DB = 1, G.$4D7 = 1: coins are ignored from now): sub2 0, 2 and sub3 0, 2 at +1, +2: the red flash, the blood splashes
  (sound 0xE6 seven times, every 18 frames from +3), sub3 4 at +110, sub3 6 at +191 (sound 0x1D, the name entry runs here when the player qualifies, with music 0x1F instead),
  sub3 8, sub2 4 at +392, **mode 0x14 at +483** after the 0. The word GAME OVER appears (blinks) from about +200. M: k-1p (1502, 1612, 1693/1694, 1894, 1985), sheet at t = 1600 to 1900.

### 6.4 The game over modes 0x14 and 0x18 and the way back

R: `0xAC06` (kill the join task and the tally task, mode 0x18, G.-$702E = 1) and `0xAC26` (clear all, stop sound, create the attract task `0x97C20` in slot 0x10, the game task ends).
M: d4d-b3/k-1p: mode 0x14 at 1985, 0x18 at 1989, G.$4CC = 0 at 1993, sounds 0xFF00 at 1991, 1994, 1996, ranking music 0x80 at 1998, 0x70 at 2075. The attract task then shows the ranking
(3.5, because the flag is set) and carries on in the state it was in (3.1). M: d4d-b16/go-attract. Port: PARTIAL (`flow_game_over` sets the flag and calls `flow_attract_start`, which is empty).

### 6.5 Every moment a hero can be chosen

1. At the start: the select screen, mode 4 (section 5), with the 10 second timer.
2. Joining during play: the in-HUD strip (6.2), 10 numbers of 40 frames.
3. Continuing (in the level or after the continue mode): the same strip (6.3), starting on the player's previous hero.
There is no other moment: the hero does not change when a life is lost, between levels, or at the ending. Two players can never hold the same hero (sections 5, 6.2). The hero of a dead
player in the countdown is free for others. A continuing player who finds his hero taken starts on the next free one.

## 7. The ranking table and the name entry

### 7.1 The table (G.$703E, 0xFFF03E, 50 entries of 16 bytes)

R: `0x1504E` builds the default table at boot (`hud_hiscore_init` is the port of it). Entry layout: +0 score (BCD long), +4 name (3 letters and a space), +8 hero id (0 to 3),
+9 stage number plus 1 (1 to 8), +$A a byte (100, 98, 96 ... in the default table, not set by new entries, use unknown [U]).
Default: score of entry n (from 1) is 100000 minus 10000 times (n − 1) in BCD (entries 11 and up wrap to 99990000 and below, negative as signed longs), names from the 4 byte table 0x150A4 (50
entries; the first 10 are initials of the development team), hero id 0, 1, 2, 3, 0, 1, ... and stage digit 8, 7, 6, 5, 4, 3, 2, 1, 1, 1, ... The ranking screen shows the first 10 (3.5).

### 7.2 Who qualifies

R: `0x154EE`: the player's rank byte +$10A (see `re/specs/flow/SPEC.md` section 2, maintained by `0x15A2E` whenever a score changes) must be 10 or better **and** the player's score must be **greater**
than the score of the 10th entry (a tie does not qualify). It is checked: when the continue countdown of a player ends (join task state 8, `0x7396`), when a player dies with continue disabled
(`0x7392`), and for every present player at the start of the ending (`0xA6A2C`). A qualifying player gets state 0xA, G.$5E4 = 1, and the entry is set up by `0x15AE2` (block of 32 bytes at 0xFF0EE4, 0xFF0F04, 0xFF0F24).
Not qualifying: state 0xC (game over label in the HUD, or nothing in mode 0x10).

### 7.3 The entry (routines 0x15AE2 setup, 0x15BE4 per frame, helpers 0x15C7A to 0x16270)

Data per block: +0 word seconds left (20), +2 word idle frames (420), +4 byte rank (1 to 10), +5 byte letters entered, +6 to +9 name, +$A word frames per second (60), +$C and +$D byte current and previous input of the player
(read from G.$16, 17, 18 and G.$1B to 1D), +$E byte repeat counter, +$F byte glyph under the cursor (0 to 29), +$10 to +$13 display copy of the name, +$16 key lock, +$17 done flag, +$18 stage of the finish blink, +$19 blink count, +$14 blink timer.
Controls (R: `0x15E3A`, `0x15EE8`, `0x15F86`): left and right (the stick) move the cursor through 30 glyphs: the letters A to Z, a comma, a point, **RUB** (glyph 28, only after one letter exists)
and **END** (glyph 29). The first press moves at once, the key repeats every 4 frames after a 24 frame delay. Attack (edge, with a 4 frame lock) enters the glyph: a letter is stored, the count rises and the cursor goes back to A; after the 3rd letter the cursor
jumps to END. RUB erases the last letter and returns the cursor to it. END ends the entry. Every key press sets the idle timer back to 420 frames; the 20 second timer is never reset. When either timer runs out the entry ends with the letters typed so far (blank
letters stay as spaces).
Finish (R: `0x15C32`): the profanity filter `0x1616C` replaces the names AAA, SEX, FUK, FUC (table 0x161B2) by blank, CPS, CAP, COM (table 0x161C6) (a name of only blanks is left as is); `0x161DA` moves the entries from the rank down by one and writes the new entry (score, the name,
the hero id of the player, the current stage plus 1; the old note "who calls 0x16230": it is the store part inside `0x161DA`, called by the entry code at `0x15C3C` to `0x15C40`), then `0x16260` sorts the 50 entries by score; the line "nTH NAME" blinks 4 times for 30 frames; the state becomes 0xC and G.$5E4 = 0.
Display (R: `0x15CEA`, `0x15D3C`, `0x15FB4`, `0x160FC`): the HUD block of the player: the rank word 1ST to 10TH in small letters at the top left of the block (scroll1 0x90A010 + 0x800 * player), the three letters in the big 2 tile glyphs behind it,
the cursor glyph (blinking) one position after the last letter. M: d4d-b18/k-name-entry (sheet). 
M timeline, one player (d4d-b4/k-hiscore with no keys, d4d-b18/k-name-entry with keys; score 450000, died at t = 202). The countdown ends at 1502 and the join task goes to state 0xA at 1503: from then on the entry runs
(`0x15BE4` is called every frame by the join task) and takes keys, while the game over scene (blood and flash, 6.3) plays behind it; the label "1ST" and the blinking letter are visible from about t = 1520. The idle timer (420 frames) starts at 1503, so with no key the entry
ends at 1923 (the table is written at 1925, blank name) and the entry blinks until 2079. At 1693 to 1694 the game over scene reaches its wait step (sub3 6): sound 0x1F (name entry tune) is queued instead of 0x1D, G.$5E4 becomes 2, and the scene waits
until every entry is finished; then sound 0x1D, sub3 8, state 0xC to 0 (2079 and 2080 in the no key run). With keys (R, R, attack, R, attack, L, attack at 1751 to 1991) the letters C and B were entered at 1831 and 1911, the cursor jumped to END with the left
key (wrap) at 1951, END at 1991, the table was written at 1992 (name CB), the entry blinked 2000 to 2140 and the state went 0xA to 0xC to 0 at 2146 and 2147. Mode 0x14 follows 291 frames after the wait step ends (as without entry).
Several players can enter their names at the same time.
Where it appears: (a) after a game over, over the villain and blood picture of the continue mode (6.3), at the player's HUD block position; (b) after the ending, over the "THANK YOU FOR PLAYING" picture (it is parked while G.$507 = 1: state 0xA does nothing, `0x74AE`; it starts when
the ending scenes clear the flag at `0x8A7D4`, t = 5019 in the ending run; with no key the idle timer ends it at 5439, the table is written at 5441, the state is 0 at 5596; the ending waits `0xA6AE8`, sleeps 180 and sets mode 0x14 at 5776). M: d4d-b6/end-hiscore.
Port: MISSING. Needs: state 0xA in `join.rs`, the entry (about 1 agent hour), table insert and sort in `hud.rs`, the ranking screen (3.5).

## 8. The ending (game mode 0x1C, routines 0xAC48, 0xA69B8)

R: the stage clear switch (`flow.rs flow_ingame_switch`, DONE) enters mode 0x1C after the Episode 8 tally with G.$4D7 = G.$4DB = G.$507 = 1 (0xABA8, timer 0xE10 = 3600 frames for stage 7). `0xA69B8` runs the object loop
(shake 0x222D6, pool B 0x23604, pool A 0x23420, sprite list 0x13FA6) once G.$4D1 is not 0 and then a jump table on G.$4D1:

| Sub | Routine | Action |
| --- | --- | --- |
| 0 | 0xA69F2 | clear all, brightness, clear the three player +0 bytes (players are out), G.$4D7 = G.$4DB = G.$507 = G.$5E4 = 1, for every player in G.$4CD run the name entry setup `0x15AE2` (so the join task sees them in state 6 with no continue and runs the qualification check, 7.2), pools reset, palette set 0, scrolls 0, G.-$6FFA = 0x14F0 (5360), sub 2 |
| 2 | 0xA6ABA | spawn the master object, pool B class 0x148 (class 82, handler 0x8AA0A), sub 4 |
| 4 | 0xA6ADA | wait: G.-$6FFA -1 per frame until 0 (5360 frames) |
| 6 | 0xA6AE8 | wait while any player is in name entry (`0xEFC2`) |
| 8 | 0xA6AF6 | sleep 180, mode 0x14, clear all (the game over path, 6.4) |
| 0xA | 0xA6B10 | a loop that shortens the timer when a player presses jump: never entered by the code above [I, not reached in 5700 frames] |

M: d4d-b5/end-jack: mode 0x1C at t = 0 (poke), sub 4 at 3, sub 6 at 5364, sub 8 at 5365, mode 0x14 at 5546, 0x18 at 5548, ingame 0 at 5552, ranking music 0x80 at 5557.

**Players and heroes: nothing changes.** M: d4d-b5 (pixel comparison of every 100th frame): Jack, Hannah and Mustapha as the only player give identical pictures for the whole ending except the first frame (HUD); two or three
players change only the first 700 frames (the chain of explosions of the lab scene differs in position, because the extra players use the random number generator), and later frames by 0.3 percent
(HUD). The cast is fixed: Mustapha (green cap) and Mess walk, Jack and Hannah arrive in the Cadillac, whatever heroes were played.

**Scenes** (t = frames after the mode starts, +/- 10; M: d4d-b5/end-jack, d4d-b12/end-fine, d4d-b11/end-objs):

| t | Master | Scene | Sound (queued) |
| --- | --- | --- | --- |
| 3 to about 480 | pool B 82 | The lab corridor: two heroes run to the right with the caption HURRY UP!! (t = 10 to 130); explosions chain behind them (explosion and debris objects, pool A classes 90 to 93 and 96, from t = 7); the caption HANNAH! appears about t = 330; a body lies in the fire | 0x66 explosions every 16 frames, 0xBB at 261, 0x10 and 0xF6 at 469 |
| 480 to 660 | 82 | Outside at sunset: a tree, the sea, a far skyline; explosions flash at the left, smoke; effect class 21 at 531 and 551, pool B 3, 83, 85, 66, 84 captions | 0x56 at 530 |
| 660 to 975 | 82 (class 87 at 776) | Still picture (black border): Mess and Mustapha look at the city; the caption JACK! HANNAH! | none |
| 975 | 82 ends, **81 starts** | Black frame at 970, then the desert at sunset | stop sound, music 0x19 (975) |
| 980 to 3955 | 81 | The two walk to the right; the camera scrolls; the staff roll: vertical blocks of lines (a red header and lines in blue, 53 string lines from 0x8B7EA, header "STAFF", PLANNING, OBJECT, SCROLL, PROGRAM, COMPOSE, SOUND DES., ASSIST, AD-DESIGN, USA SUPPORT, MANY THANKS ...) move left across the screen with the scroll | |
| 3956 | 81 (pool A 17 twice from 3956; pool A 81 are the two walkers, from 976) | music 0x1A; the Cadillac with Hannah at the wheel and Jack beside her rolls in from the right and stops by the two walkers | 0x1A |
| 3980 to 4220 | 81 | Mustapha's portrait (green frame) and the caption "Jack, Hannah, you are alright!" | |
| 4290 to 4470 | 81 | Jack's portrait: "Yeah, we barely made it!" | |
| 4400 to 4680 | 81 | Jack: Fessenden was a madman, his insane plans almost became our reality (caption typed in 3 lines) | |
| 4680 to 4830 | 81 | Jack: Fessenden tried to mess with the balance, nature always balances itself | |
| 4830 to 4930 | 81 | Hannah's portrait (red frame): "Let's go home!" | |
| 4930 to 5364 | 81 ends at 4989 | "THANK YOU FOR PLAYING" appears (typed), then "PRODUCED BY" and the CAPCOM sign; the car drives on to the right | |
| 5364 | | timer ends, sub 6, name entry if a player qualifies | |
| 5365 to 5546 | | sleep 180 | |
| 5546 | | mode 0x14, then the ranking and the attract loop (3.5, 6.4) | stop sound, music 0x80 at +11 |

Captions are tall font strings (printer 0x84F72) with portrait blocks; the staff roll lines are not reproduced here (the names are people's names, not copied).
Pictures used: scroll3 page 0x6C and pages 0x78 to 0x7C (the desert and sky), scroll2 page 0x16 (the ground), the lab corridor is the Episode 8 map (R: picture_calls at 0x8A63C, 0x15222; the lab and tree
backgrounds belong to the level data).
Port: MISSING (`flow_game_over` is called instead). The objects seen: pool B 82 (scene 1), 81 (scene 2), 87, 85, 84, 83, 66 (captions and helpers), pool A 17, 81, 90 to 93 and 96, effect class 21 (the object list of d4d-b11 was taken in a run that started from the Episode 1 level, so a few leftover objects of that level appear at t = 0). The explosion chain uses the random generator, so with the same state the same explosions appear.
Size: 3 to 4 tasks (scene 1 and 2, scene 3 and staff roll, car and dialogue, name entry and end).

## 9. Link scenes between episodes

Order (M: d4d-b5/t-1p-fb-lowhp, d4d-b9/link-ep2 to ep8): boss dies (`0xACEE`, `flow.rs flow_boss_defeated`, DONE) then G.$4D1 sub 2, 4 (walk off or 1800 frames) and 6, the **tally** (task 0xB97E, slot 0x30) with the **map panel** at its end, then
mode 8 (**story panel**), then the level start with the **intro task** (0xB9BE, slot 0x30: map panel, EPISODE card, HUD slide-in, opening scene).

### 9.1 Stage clear tally (task 0xB97E, state tables 0xB9A4)

R: states 0 (`0xBA28` setup, fade, task 0x8854), 2 (`0xBBFC`, nine sub steps at G.$735F, table 0xBC16: 0xBC28 VITALITY, 0xBD7E ENEMIES, 0xBEAA FINAL BLOW, 0xBFA6 M.V.P., 0xC0EE clear panels, 0xC14C TOTAL, 0xC322 count down into the score,
0xC444 clean up, 0xC4BE), 4 (`0xC4CE` prepare the map panel, clear scroll1; stage 7 jumps to state 0xA), 6 (`0xC576`: the map panel slides in, 26 frames, then the destination marker blinks for 120 frames, same code as the intro card), 8 and 0xA (`0xCBA4`, `0xCCFE`: end, clear G.$4E2). Player blocks of 32 bytes at 0xFFF37E, 0xFFF39E, 0xFFF3BE (flag +6 present). Strings VITALITY, ENEMIES, FINAL BLOW and M.V.P are at 0xBD74, 0xBEA2, 0xBF9A, 0xC0E8 (glyph strings).
Rules, measured in MAME with poked HP, kill counts, damage and final blow flags (one MAME run per value, d4d-b7, d4d-b8):

| Panel | Formula per player | Notes |
| --- | --- | --- |
| VITALITY | (HP − 1) * 100, but **20000 when HP is 100** | HP 99 gives 9800, 41 gives 4000, 1 gives 0. Shown as the number counting up ("5.9 x 100" first, then 5900) |
| ENEMIES | kills * 100 | kills = byte +$10B of the player (the killing player gets +1 per enemy; in the car section every player gets it; enemies killed by a kill-all, also the boss kill-all at the end, count too: in every test the 6 enemies of the Episode 1 opening scene that were still on screen were counted, +600 for one player, split between the players by last attacker). 150 kills gives 15000; the value 255 counts as −1 |
| FINAL BLOW | 20000 | only for the player who killed the boss (+$C0 flag) |
| M.V.P. | 10000 | for every player whose damage dealt (+$10C, sum of the HP taken from enemies) equals the highest among the present players (ties share it; with one player always) |
| TOTAL | sum | counts down 100 per frame into the score; the player with the highest total gets the red WINNER! label |

Examples (M): one player HP 100, 0 kills, no final blow: 20000 + 0 + 10000 = 30600 with 6 enemies still alive (600), 750 frames. Three players HP 100 / 60 / 30 etc. shown in d4d-b5/t-3p (Jack 31200, Hannah 38100 with final blow and WINNER!, Mess 16000).
Timeline (M: t-1p-fb-lowhp, t-3p; t = frames after the tally task starts, sound 0x49 at each pop): VITALITY panel t = 8, ENEMIES 39, FINAL BLOW 70, M.V.P. 101 (31 frames apart), the panels clear about 140, TOTAL shown at 160, the count down starts at 177
(one step of 100 per frame, sound 0x49 every 11 frames) and takes max_total / 100 frames, then 268 frames of pause during which the map panel slides in from the right (columns copied, scroll1 X + 12 per frame to 0x130) and the task clears G.$4E2: the end is at
177 + max_total / 100 + 268 (801 for 35700, 826 for 38100, 750 for 30600). The G.$4D1 sub 6 timer (0x708 = 1800 frames, 0xE10 for the last stage) is only a limit. Then mode 8.
The stage clear tally for Episode 8 skips the map and goes to the ending.
Port: STUB (`flow_clear_task` waits 120 frames). Needs: 4 panels with counters, the player blocks, the score feed (`hud_add_score_to`), the map panel and its slide, the WINNER label. Sizes: 2 tasks.

### 9.2 Map panel and EPISODE card (intro task 0xB9BE, table 0xB9EE)

R: `0xB9BE` (struct G.$735E, states 0 `0xC526`, 2 `0xC588`, 4 `0xCBAA`, 6 `0xCCFE`). For stage 0 (Episode 1) it starts at state 4 (`0xB9F6`): no card, scroll1 held, HUD slide-in after the walk-in.
For stages 1 to 7:
- State 0 (1 frame): sleep, state 2, prepare the copy of the map panel (source 0x90A028, target 0x909808, 12 columns per step).
- State 2 sub 0 (`0xC642`): the **map panel** slides in from the right: columns are copied 8 pixels at a time while scroll1 X += 12 per frame until 0x130 (26 frames). The picture is an illustrated map (a framed panel at the top centre) with the place.
- State 2 sub 2 (`0xCA20`, after `0xC6A8` picks the title block from the table 0xC6D2): the title card: "EPISODE n" and the title of the level (7 titles: the swamp forest, hell road, Jack's garage, the village of flame, jungle and mine, the vault, the under world; strings at
  0xC8AC to 0xCA20, table 0xC6D2 of 16 pointers: 8 for the Japan set, 8 for the other sets) typed letter by letter (`0x7EC20`), 141 frames in all, then removed.
- State 4 (`0xCBAA`: sub 0 `0xCBBE` clears the panel columns while scroll1 X + 12 per frame to 0x260, sub 2 `0xCC58`, sub 4 `0xCC84` the HUD slide-in): the map panel slides out and the HUD slides in (from the right, 12 px per frame to 0x3F0, `flow.rs flow_intro_task` already has this part for stage 0), G.$4D5 = 1 during it, then the scripted opening scene runs until the players' script sets G.$4E3, then state 6 (G.$4E2 = 0, task ends).
Times (M: d4d-b9, t after the poke of the panel; the panel ends at P): state 2 at P + 2 or 3, sub 2 at P + 26 to 28, state 4 at P + 169 or 170, G.$4D5 = 1 at P + 229 or 230, state 4 sub 2 at P + 253 or 254, G.$4E3 = 1 at the end of the opening scene,
state 6 two frames later:

| Episode | Panel length P (mode 8) | State 4 sub 2 starts | G.$4E3 set | Scripted part |
| --- | --- | --- | --- | --- |
| 1 (select to level, frames from power on) | none (no card) | level starts at 1494 (mode 0xC), state 4 at once | 2109 | 613 frames from G.$4D8 = 1 at 1496; G.$4D5 released 2111; G.$4DD and G.$4D8 cleared 2141; state 6 at 2143 (M: d4d-b22/ep1-start) |
| 2 | 306 | 560 | 876 | 316 frames (the car arrives, the heroes jump out) |
| 3 | 250 | 505 | 569 | 64 |
| 4 | 308 | 563 | 1200 | 637 frames (the garage scene, sounds from 887) |
| 5 | 439 (mode 8 goes straight to 0xC) | 694 | 757 | 63 |
| 6 | 368 | 622 | 686 | 64 |
| 7 | 368 | 622 | 686 | 64 |
| 8 | 313 | 567 | 631 | 64 |

(64 frames is the HUD slide-in itself: no walk-in script. For Episode 1 the same hold lasts 613 frames: the walk-in and the fight of the opening scene, see D4c.) Scripted control sets G.$4D8 (skip-wait task) and G.$4DD at the card change; both end 3 or 30 frames after G.$4E3.
Sound: the level music is queued when the level starts (mode 4 to 0xC): codes 5, 7, 9, 0x17, 3, 4, 0x0C for Episodes 2 to 8 (table 0xA582 area 0; Episode 1 uses code 1), preceded by the stop code 0xFF00.
Port: PARTIAL (the HUD slide-in for stage 0 is ported). The map card and typed title are MISSING. Needs `0x7EC20` (typed text), the map panel data, the title strings.

### 9.3 Story panels (game mode 8, routine 0xD898)

R: `0xD898` (palettes 0x8A44, 0x8932, clear all, `0xD8B4`: palette set 2, brightness, then a jump on the stage through the table 0xD8D8 (stage 0 does nothing; 1 and the others to the entries below), then `jmp 0xAC4E` (stage start)).
Each entry sets a music code (the panel tune), draws a small picture (a face, tile block printer 0x98BAE, lists next to the entry), prints the tall text with the typing helper `0x84736`, waits with a counter in G.-$719A (word at 0xFF0E66: 0x12C or 0x168 frames), and clears all.
M: d4d-b9 (sound at t = 3, durations): the panel is a black screen with a small picture and 2 or 3 lines of white text (typed).

| Stage (Episode) | Entry | Music code | Text blocks | Length (poke to mode 4) | Content (paraphrased) |
| --- | --- | --- | --- | --- | --- |
| 1 (Ep 2) | 0xD8E8 | 0x13 | 0xE42A | 306 | the Butcher hunts in the northern woods, do not mess with him (a rare random branch at 0x1F18A is the anti-tamper check, ignore) |
| 2 (Ep 3) | 0xD97E | 0x14 | 0xE500 | 250 | a blond friend says the heroes look busy, a good time to take Jack's garage (scroll2 page 0x1B drawn three times) |
| 3 (Ep 4) | 0xDAB4 | 0x15 | 0xE5AE | 308 | Mustapha shows a place on a map and says somebody will pay for the garage |
| 4 (Ep 5) | 0xDBB2 | 0x16 | 0xE606 then 0xE8B0 | 439 (ends with mode 0xC at once) | a radio call from a village under dinosaur attack, then Jack says something happened to him (second text and a 64 frame pause) |
| 5 (Ep 6) | 0xDDCC | 0x13 | 0xE6B0 | 368 | a hero says the doctor is mad, he is creating new life forms |
| 6 (Ep 7) | 0xDE6C | 0x14 | 0xE74E | 368 | a hero recognises the genius Dr. Fessenden and the equipment |
| 7 (Ep 8) | 0xDF1E | 0x15 | 0xE7DC | 313 | the villain says the heroes have only seen a taste of his power and invites them to come |

**The panels depend on the heroes in play (Episodes 4 to 7) and can be skipped.** M: d4d-b21 (32 segments: Episodes 4 to 8 with Jack, Mess, Mustapha, Hannah alone, Jack and Hannah, Jack and Mustapha and Mess).
- R: `0xE0AA` returns the hero id of the first present player (P1, else P2, else P3), the "playing hero". `0xE034` returns the lowest hero id (ids 0 Jack, 1 Mustapha, 2 Hannah, 3 Mess) that no present player uses, the "caller", then swaps 1 and 2 to match the portrait table.
  The portrait and text printer is `0x84736` (d0 = portrait 0 to 3, 0x63 = none; d1 = text id; it is also used by in-level dialogues at 0x696B6 and 0xA607C to 0xA62D0 and the pointer table 0x84800).
- Episode 4 panel (0xDAB4): the portrait of the caller (green cap Mustapha for Jack alone, Jack for Mess, Mustapha or Hannah alone, Mustapha for Jack and Hannah, Hannah for Jack, Mustapha and Mess) and the line "<playing hero>, look..." with the name
  of the playing hero (JACK, MUSTPHA (sic), HANNAH, MESS; strings 0xA54DE, 0xA54FC, 0xA5520, 0xA5542); the second text and the picture (a desert garage) are fixed.
- Episode 5 panel (0xDBB2): the text "HELLO THIS IS <playing hero>" (strings 0xA5566, 0xA5598, 0xA55D2, 0xA5608) beside a group picture; the rest is fixed.
- Episode 6 panel (0xDDCC): the portrait of the playing hero says the line; **when Hannah plays alone no portrait is drawn** (a blank space; probably a bug of the original).
- Episode 7 panel (0xDE6C): the portrait of the caller (same rule as Episode 4); fixed text.
- Episodes 2, 3 and 8 are fixed.
- **Skip:** a present player pressing attack alone (the input byte is exactly 0x10, no direction, and changed since last frame) sets the wait counter to 1 (R: `0xDFCC`); jump does nothing. M: pan-skip-ep2: attack at 60 gives mode 4 at 64 instead of 306.
  The same wait loop serves the other panels (the probe showed the skip for Episode 2; the other episodes [I]).
- The text and portrait ids that do not appear in the table above (the 0xA4F16 to 0xA58F6 block) are used by these panels and by in-level dialogue.

Port: MISSING (`flow.rs flow_game_task` mode 8 starts the stage at once). Needs: the entry scripts (7), the picture block lists, the typing helper. Size: 1.5 agent hours.

## 10. Test mode and debug mode

**Test mode** (service menu). R: task `0xA6CE8` in slot 0x10. It starts (a) at boot when DIP C bit 7 (port 0x80001E) is clear, and (b) at any time when the TEST input bit (G.$1A bit 6) changes: `0x9D5A4` kills the tasks 0x10 to 0x60, 0xE0 and
0xF0, resets the video registers, loads palettes and creates the test task. Menu "TEST MENU": 1 INPUT & OUTPUT, 2 SOUND & VOICE, 3 COLOR, 4 DOT CROSS HATCH, 5 SYSTEM CONFIGURATION, 6 OBJECT, 7 SCROLL1, 8 SCROLL2, 9 SCROLL3, 10 OBJECT TEST,
11 SCROLL TEST, 12 MEMORY CHECK, 13 EXIT (a shorter 11 item version without OBJECT TEST and SCROLL TEST exists). SYSTEM CONFIGURATION edits the EEPROM: coin menu (8 ratios, "2 coins start / 1 coin continue", free play), monaural or
Q Sound, chute modes (3 players 3 chutes, 3 players 1 chute, 2 players 1 chute), difficulty "RANK" (8 levels, EXTRA EASY to HARDEST), "TIMER LEVEL" (4 levels), extra life (4 options), demo sound, continue, lives. MEMORY CHECK tests EEPROM, QSound RAM, CPS buffer RAM, work
RAM, scroll RAMs, object RAM. There are also developer pages (hit box editor strings BODY HIT and ATACK HIT, "PARAMETER CHANGE" with the second stick). 82 strings in all (`asset-catalog.md` N6).
**Debug mode** (G.$4AA): P1 up, right, attack, jump and coin 1 held at power on turns DIP bits into cheats (stage select, invincible, boxes, pause) (section 2).
**Recommendation:** do not port either. The port gets the config file of 4.3 and keeps the existing F1 box overlay as its developer aid. If the user wants a pause key, add it outside the original logic.

## 11. Sound codes of the front end (queued by the game code, filtered as in 3.11)

R: queue routine `0xA63C` (d0 code, d1, d2 parameters); music or effect numbers are the Z80 command table of `asset-catalog.md` section 8. D4e owns the meaning, this list says **where each code is sent** (M: the ring of the probe).

| Code | Where | Parameters seen |
| --- | --- | --- |
| 0xFF0B | boot (twice) | |
| 0xFF00 | stop sound: attract state starts, coin (when not in a game), boot end, ranking start, level start, game over | |
| 0x88 | title tune, 9 frames after each title starts; again in the movie at counter 1350 | |
| 0x13, 0x38, 0x4B to 0x4E, 0x40 to 0x43 | profile music, hit sounds of the small hero, name voices (Jack, Hannah, Mustapha, Mess), pose sounds | |
| 0x80, 0x70 | ranking music and ambient | |
| 0x24, 0x25 | story slides music (start and the fifth slide) | |
| 0x61, 0xEA, 0x73, 0x75, 0x36 | movie: steps, other, rustle, dinosaur steps, roar | |
| 0x30 | coin or service credit | d1 = 0x70010, d2 = 0x3800000 (plays even in attract) |
| 0x31 | game start (start press with credit) | d1 = 0x10 |
| 0x00, 0x32, 0x33 | select: music, cursor move (d1 = pan 0x00, 0x0B, 0x15 or 0x20 by position), pick (same pan) | d1 = 0xFFFF for the music |
| 0x01, 0x05, 0x07, 0x09, 0x17, 0x03, 0x04, 0x0C | level music at the first level frame: Episode 1 (code 1), then Episodes 2 to 8 | table 0xA582 |
| 0x13, 0x14, 0x15, 0x16 | story panel tunes: Episodes 2 to 8 use 0x13, 0x14, 0x15, 0x16, 0x13, 0x14, 0x15 | |
| 0x49 | tally pops (4) and count down ticks (every 11 frames) | |
| 0xE1, 0xE0 | in the join strip: 0xE1 about 16 frames after number 2 starts; 0xE0 about 56 frames after the hero is picked (the hero's line) [U meaning] | |
| 0x1B, 0x35, 0x1C, 0xAB | continue: music, tick (d1 = 0x10), music at number 9, jingle when a continue is accepted | |
| 0xE6, 0x1D, 0x1F | game over: seven blood splashes, game over jingle, name entry tune | |
| 0x19, 0x1A, 0x66, 0xBB, 0x10, 0xF6, 0x56 | ending music (desert, car arrives) and the explosion sounds of the lab scene | |

## 12. Port state per screen (checked in the repo today)

| Screen | Original routines | State | Where in the port |
| --- | --- | --- | --- |
| Power on, tasks, EEPROM settings | 0xA69A, 0xAC794 | PARTIAL: `Settings` (G.$777A/B/C, factory values) DONE; boot sequence MISSING; `flow_boot` only loads the hi-score table | `flow.rs`, `main.rs` always calls `start_stage` |
| Boot screens | 0x9DCF6 | MISSING | |
| Attract task and loop | 0x97C20, 0x97D84 .. | STUB (`flow_attract_start` sets in_game 0, `flow_attract_task` empty) | `flow/front.rs` |
| Title, press-start screen | 0x9A3CE, 0x983EE, 0x978EE | MISSING | |
| Profiles, ranking screen, slides, movie | 0x9A7E6, 0x1516C, 0x988C0, 0x9BF50 | MISSING | |
| Ranking table and rank text | 0x1504E, 0x15A2E, 0x16230 | PARTIAL: `hud_hiscore_init`, `hud_rank_*` in `hud.rs`; insert and sort MISSING | `hud.rs` |
| Name entry | 0x15AE2, 0x15BE4 | MISSING | |
| Demo | 0x99E88, 0x99F10, 0x9A146 | MISSING (the per frame order exists in `game_frame`; `rank.rs` has `w4f6` and the bands) | |
| Sound filter for attract | 0x610 | MISSING (`snd.rs` only records requests) | |
| System task, coins, credits | 0x9D048 .. | STUB (`flow_system_task` stores the previous input) | `flow/front.rs` |
| Character select | 0xAD46 | STUB (`flow_select_mode` starts the stage) | `flow/front.rs` |
| Join, continue, game over label | 0x6C4A .. | STUB (`flow_join_task`, `flow_join_prompts` empty; `player_join` and `item_give_bazooka` ported) | `flow/join.rs` |
| Continue mode | 0xE9E6 | STUB (`flow_continue_mode` goes to mode 0x14) | `flow/join.rs` |
| Game over modes | 0xAC06, 0xAC26 | PARTIAL (`flow_game_over`) | `flow.rs` |
| Tally | 0xB97E | STUB (120 frame wait) | `flow.rs flow_clear_task` |
| Intro (map card, EPISODE card, HUD slide, walk-in wait) | 0xB9BE | PARTIAL (stage 0 state 4 only; `flow_player_scripts_ported` is false, TEMP) | `flow.rs flow_intro_task` |
| Story panels | 0xD898 | MISSING (`flow_game_task` mode 8 starts the stage) | `flow.rs` |
| Ending | 0xA69B8 | MISSING (mode 0x1C goes to game over) | `flow.rs` |
| Stage clear switch, boss defeat | 0xAB2C, 0xACEE | DONE | `flow.rs` |
| Test mode | 0xA6CE8 | not needed | |

### 12.1 What changes with the number of players and with the cabinet layout

| Place | 1 player | 2 or 3 players | 2 player cabinet layout (G.$4CF = 1, DIP) |
| --- | --- | --- | --- |
| Demo | three heroes play (the three player table) | same | two heroes, other rows (3.6), P3 absent |
| Select | one panel, cursors as 5 | cursors skip each other, joins reset the timer | panel frames from pages 3 and 4, two panels [U] |
| Join strip | | each joiner has an own strip; the held heroes are only those of players in state 4 or 6 | strip timer starts at 6 (seconds 5 to 0, lock check at 5), shorter |
| Continue | countdown in the HUD block when another player is alive, else the continue screen | same | same |
| Tally | no WINNER! label (`0xC276` needs more than one player) | panels per player, M.V.P. for the most damage (ties share), WINNER! for the highest total | two columns [U] |
| Ending | fixed | fixed (explosions differ by random numbers) | fixed |
| Name entry | one entry | every qualifying player enters his name at once | same |
| Story panels | speaker and names follow the party (9.3) | the first player's hero, and the first unused hero as the caller | same |
| Credits | | per player credits (three chutes) or a common pool (one chute modes) | common pool |
| Join and continue effects | clock, wave timer, score +1 for all three players | same | same |

## 13. (a) Acceptance scenarios

How to check: run the original with the probe (`re/harness/probe/probe.sh`, manual in `capture-recipes.md`), run the port with the same input script
(`dino-game --input-file ... --screenshot ... --frames N` and the harness trace), and compare. "Frames" are from power on unless the line starts with `jump`.
All probe inputs below exist as files in `re/out/d4d/tools/bN.txt` (git-ignored, appendix A). **Exact** = equal to the frame. "Picture" = the 384 by 224
screenshot equals the MAME screenshot at that frame (tilemaps, sprite list and palettes equal; the harness diff gives the detail).

| ID | Probe lines (shortened) | What must match |
| --- | --- | --- |
| A1 boot | `frames 330`, `snaps 60,100,130,170,250,310` (recipe `fe-boot`) | text 1 visible from 73 to 194, text 2 from about 196 to 317, layer control change at 317, title counter 1 at 322 (exact); pictures at 100, 250, 330 |
| A2 boot skip | `100-103:A` (b14 boot-skip-a), `230-233:2J`, `100-103:3A` | screens end at 226, 234, and not at all for player 3 (layer change frames 226, 234, 317; exact) |
| A3 attract loop | `frames 32000`, no input, watch 0xFF0FD0, mode, G.$757A and the counters of 3.2 (b1, b17) | the state change frames 2744, 6874, 9311, 13411, 15848, 19979, 22416, 26516, 28953, the part frames of the table in 3.2 (exact), demo counters 1 at 3345, 2 at 9916, 3 at 16450, 4 at 23021, 5 at 29555, and sound codes at the frames of 3.11 (exact) |
| A4 title | `snaps 340,420,560,700,860` (recipe `fe-title`) | picture; INSERT COIN visible 30 on and 30 off; small lines at title frame 30; licence text at frame 360 |
| A5 profiles | `snaps` every 20 from 900 to 2800 (b10 att-profiles) | picture at the card starts (932 exact, the others about 1297, 1658, 2019), the fades and the sound frames of 3.4 |
| A6 ranking | `snaps` 2330, 2500, 2740 (recipe `fe-ranking`) | picture; the default table rows; the desert scrolls 8 px (scroll 2) and 4 px (scroll 3) per frame, the car drives across |
| A7 demo | rows 1 to 5 and 0: poke G.$757A before the title (recipes `fe-demo-row-0` to `-5`), first game frame and frames 60, 300, 600, 900, 1200, then 1205 and the fade | RAM: player objects, rank, RNG state, sprite list at those frames (exact once the level exists); the HUD |
| A8 slides | `snaps` every 10 from 4600 to 6500 (b12 slides-fine) | picture at each of 5 slides, cuts at 5010, 5320 to 5340, 5610 to 5630, 5730 to 5750, 6080, 6200, 6450 |
| A9 movie | `snaps` every 10 from 11900 to 13450 (b12 mov-fine) | the cut list of 3.8 and the sound frames |
| A10 coin and start | `450-457:C`, `600-605:S` (b2 c-title) | credit at 460, G.$46D = 0xFF, sounds 0xFF00 and 0x30 at 460, 0x31 at 601, ingame at 601, mode 0 about 619, mode 4 at 623, sub 2 at 625 (exact) |
| A11 no credit | `450-455:S`, `1000-1005:S` (b2 s-nocredit) | nothing changes, no sound |
| A12 free play | poke 0xFF846B = 1 at 100 then `500-505:S` (b15) | no press-start screen; FREE PLAY text; select at 523 |
| A13 select | b4 sel-1p (no keys), sel-1p-moves, sel-2p-late, sel-same, sel-3p-all-left, sel-3p-timeout | TIME 10 at 725, steps of 60 from 735, auto pick 610 frames after the screen starts, sounds 0x32 and 0x33 with the pans, no duplicate heroes, timer reset 8 to 10 when P2 joins at 841, the end offsets +61, +62, +79, +139 to +158, mode 0xC at +159 |
| A14 join | `jump 1 1`, `0 god on 1`, `10-17:2C`, `60-65:2S`, `140-145:2A` (b3 j-p2), j-p2-wait, j-p23, d4d-b20 | the P2 timeline of 6.2: state 2 at 20, 4 at 61, strip steps every 40 frames, pick, x 192, y 384, rinv 360, held bazooka; clock to 4:00 (b19); scores +1; attack alone joins, jump does not |
| A15 continue | `jump 1 1`, `0 lives 0`, `0 hp 0` (b3 k-1p), k-mash, k-coin-late, k-coin2, k2-timeout, k2-coin | mode 0x10 at 203, numbers at 302, 362 ..., 1502; the mash result (beeps from 425, end 493); continue start at 1341 gives mode 0xC at 1382; in-level countdown 215 to 1455 then GAME OVER label 256 frames; credits and lives |
| A16 game over | k-1p to 2300 | mode 0x14 at 1985, 0x18 at 1989, ingame 0 at 1993, sounds at 1998 and 2075, ranking screen after |
| A17 name entry | `score 450000`, `lives 0`, `hp 0`, keys of b18 | state 0xA at 1503, music 0x1F at 1694, the letters, table write at 1992 with name CB, state 0 at 2147, and the final ranking picture (the new first row) |
| A18 tally | b7 tv-hp1 to tv-hp100, tv-k0 to tv-k255, tv-fb, tv-2p-*, tv-3p-a, b8 td-* | the score gains listed in 9.1 (30600, 20400, 10600 ...), MVP by damage, the end frame formula, pictures at t = 60, 100, 120, 160 (b5 t-1p-fb-lowhp) |
| A19 link scenes | b9 link-ep2 to link-ep8 | panel lengths 306, 250, 308, 439, 368, 368, 313, the intro frames of 9.2, sounds |
| A20 ending | b5 end-jack (and end-hannah, end-3p for the independence), b11 end-objs, b12 end-fine | sub 4 at 3, sub 6 at 5364, mode 0x14 at 5546; cuts at 10, 660, 970 to 980, 5550; the sound codes; pictures at 100 steps; the independence of the hero |
| A21 ending name entry | b6 end-hiscore | entry parked until 5019, table at 5441, mode 0x14 at 5776 |
| A22 story panels | b21: `jump 1 0 char=...`, poke stage, sub 0, mode 8 at t = 0, snaps 100, 250, 320; pan-skip-ep2 (`60-63:A`) | the portrait and name of 9.3 per party (Episodes 4 to 7), the fixed panels, skip at attack (mode 4 at 64), pictures |

## 14. (b) Task breakdown for the port (M1 and M2, plus what M3 takes from here)

Rules: one task is one agent run of 1 to 2 hours, ends green with evidence, each ported routine names its original address, data goes through `dino-extract` as exact table copies
(`dino-assets/src/text.rs TABLES` pattern, see appendix B). Sizes are agent hours including the MAME comparison.

**M1: boot to attract loop** (sum about 17.5 hours; NEW-PLAN had 10, the profile cards, the movie and the demo are bigger than assumed)

| Task | Scope | Routines | Data to extract | Files | Hours |
| --- | --- | --- | --- | --- | --- |
| T1.1 Services | a front end scene runner (a struct with a step function and sleep counters, so scripts like 0x988C0 read like the original); text printers 0xBAE, 0xBB6 (bytes with positions), 0xC0A, 0xC44 and 0xC4C, 0xC9A (words), 0xCD2, 0xCE4, 0x84F72 (tall font), the typing helper 0x84736 and 0x7EC20; picture loaders 0x20454, 0x204CA, 0x20592; block printers 0x98B86 and 0x98BAE; the sound filter of 3.11 (setting `demo_sound`); the config file; `main.rs` boots into the attract by default | | page tables (scroll1 base 0x15CFE8 and the page index lists, scroll2 base 0x145DC8 with cells 0x175378, scroll3 base 0x177288) as needed by the scenes; text tables of appendix B | `game/flow/scene.rs`, `game/flow/text.rs`, `dino-extract/src/frontend.rs`, `main.rs` | 2 |
| T1.2 Boot, title, press-start | boot screens (section 2), title (3.3), the blink task, the logo object (pool A class 98, variants 0 to 3, 0x10, 0x11), the green background blocks | 0x9DCF6, 0x9DD80, 0x9A3CE, 0x98874, 0x983EE, 0x978EE, 0x980B0, 0x6A20A | boot text and palettes, title strings, block lists | `flow/attract.rs` | 2 |
| T1.3 Profiles | the timeline machine, 4 cards, objects pool B 93, 94, pool A 100 with the pose sounds | 0x9A7E6, 0x9A92E to 0x9AB32, 0x9AC86, 0x9ACBA, 0x91FB4, 0x921DA, 0x6A5F4 | card texts, labels, timeline tables | `flow/attract.rs` | 2 |
| T1.4 Ranking screen | draw 10 rows from the table, car object, scroll, fades, sounds | 0x1516C, 0x15594, 0x15748, 0x15888, 0x1591C, 0x8B428 (pool B 86) | label block, row tables, face tiles | `flow/attract.rs`, `hud.rs` | 1.5 |
| T1.5 Demo | rows, stream extraction, setup with the exact RAM clear and the RNG reset, player input playback, end sequence, INSERT COIN task kill for stage 5 | 0x99E88, 0x99F10, 0x9A0CE, 0x9A146, 0x18C88, 0xAC6A | rows (0x9A1E2..), pointer tables, 30 streams (0xC869A..0xC9EE6) | `flow/attract.rs`, `input.rs` | 3 (needs M0 and, to look right, M5 to M8) |
| T1.6 Story slides | the script of 3.7 | 0x988C0 | blocks, texts | `flow/attract.rs` | 1.5 |
| T1.7a Opening movie, part 1 | timeline engine, handlers 10, 200, 500 (texts and CAPCOM), helpers | 0x9BF50 .. 0x9C29C | texts, block lists | `flow/attract.rs` | 1.5 |
| T1.7b Opening movie, part 2 | handlers 600 to 1450: jungle, objects 11, 12, 30, 69, 32, logo, helpers 0x9C7C4 and 0x9C7F8, sounds | 0x9C29C .. 0x9C79C, 0x9C818 | palettes, pages 3, 4, 0x72, lists | `flow/attract.rs`, pool A and B classes | 2.5 |
| T1.8 Attract state machine, minimal system task | state byte and order of 3.1, the game over flag, resume after a game over, coin edge and credit (4.1), press-start screen switch (3.10), start, sound 0x30 and 0x31, G.$4F6 counter | 0x97C20, 0x97C94, 0x97D84, 0x97CC0, 0x97D02, 0x983D8, 0x9D048, 0x9D4BE, 0x9D326 | | `flow/front.rs` | 1.5 |

**M2: coin to select to play to continue** (sum about 18.5 hours; NEW-PLAN had 14, because the continue mode, the name entry and the intro cards were not counted)

| Task | Scope | Routines | Hours |
| --- | --- | --- | --- |
| T2.1 System task complete | all coin and chute modes of 4.2 (or only the three factory ones plus free play, decide with the user), credit text 4.4, SERVICE, lockout, the config file | 0x9D048 .. 0x9D8EE | 2 |
| T2.2 Character select | section 5: records, cursors, panels, stat diamonds, timer, auto pick, joins during the select, end sequence, 5 object classes, sounds | 0xAD46, 0xAF7C, 0xB06A, 0xB122, 0xB25E, 0xB444, 0xB810 | 3 |
| T2.3 Join task | states 0 to 6 of the join task, prompts 6.1, the strip, drop-in values, clock and score effects | 0x6C4A, 0x6D60, 0x6ED8 to 0x7580, 0x75B2, 0x7AE4, 0x7BC8, 0x7D5C, 0x7D8E, 0x7E70, 0x7EE8, 0x7F86, 0x80A2 to 0x82E8, 0x1049E | 3 |
| T2.4 Continue | in-level countdown (state 8), the continue mode 0xE9E6 (sub 0, 2, 4, 6), game over modes 0xAC06 and 0xAC26 and the way back to the attract | 0x732E, 0xE9E6 .. 0xF57C, 0x10490, 0xAC06 | 3 |
| T2.5 Name entry | state 0xA, the entry, insert and sort, the ranking update, the parking while G.$507 | 0x15AE2 .. 0x16270, 0x74A4 | 1.5 |
| T2.6 Tally | task 0xB97E complete with the rules of 9.1 and the map panel | 0xB97E .. 0xCCFE (clear part) | 3 |
| T2.7 Intro cards and story panels (NEW-PLAN had them in M3) | intro states 0 to 2 (map, typed title), the 7 mode 8 entries | 0xB9BE .. 0xC9xx, 0xD898 .. 0xE020, 0x84736, 0x7EC20 | 3 |

**M3 items fed by this document:** T3.7 ending as 4 tasks (scene 1 and 2 with pool B 82, 87, 85, 66, 83, 84, the desert and staff roll with pool B 81, the car and the dialogue, the end sequence and the name entry hook), T3.8 ranking and name entry are
already in T1.4 and T2.5. Order inside M2: T2.1 then T2.2, T2.3, T2.4, T2.5, then T2.6, T2.7.

## 15. (c) Open items

1. The profile card objects (pool A class 100 poses and sounds, pool B 93 and 94): read from the code when porting T1.3; not documented beyond the sound frames.
2. The movie helpers (0x9C7C4, 0x9C7F8, 0x9C818) and the object classes 11, 12, 30, 69 are described by what they spawn, not by their animation scripts.
3. The 2 player cabinet layout (G.$4CF = 1): demo teams and block addresses are in the tables, the select panels and HUD variants were not measured because MAME has no DIP for it.
4. The strings at 0xA4F16 to 0xA58F6 (per hero lines): the story panels of Episodes 4 to 7 use some (9.3); the others (0xA4F16 to 0xA5030, 0xA53C2 to 0xA54B0) are probably in-level scenes (D4c). Which strings go with the 0x84736 text ids 0x28 to 0x2D and the portrait tables 0x8461A, 0x8463E, 0x84696, 0x846DE was not decoded.
5. The staff roll: block order and scroll speed (it moves with the camera of the scene); strings are in 0x8B7EA to 0x8BB04 (53 lines).
6. The opening scene of Episodes 2 and 4 (length 316 and more than 700 frames): D4c.
7. Why +$A of the hi-score entries is 100, 98, 96 ...: unknown [U]. EEPROM bit G.$777C bit 4: unused in the front end [U].
8. The quirk "attack also continues, but only when some start edge is present in the same frame" (R: `0x73E4` to `0x7412`) and the Japan region rule (no attack join) are from the code, not measured with a second start edge.
9. Exact tile columns of the ranking rows (name, score, face, stage digit): the code places them at column offsets 10, 20, 40 and 43 (G.$700E), checked on pictures only.
10. The continue mode sub 6 (restore of the scrolls, 0xEE6E) was read only partly; the pictures and the timing match what is written.
11. Sounds 0xE0 and 0xE1 in the join strip: meaning unknown (D4e).
12. The panel pictures: the small pictures (a face, a desert, a group, a machine, the villain) are tile blocks next to each entry (lists at 0xD94E.. 0xDF78..); the Episode 5 panel shows a brown static block in its second part (a radio picture): checked on pictures only.
13. The 1 in 16 branch of the first panel and the 1 in 128 tasks: confirmed anti-tamper by reading, never fire in MAME. [R]
14. Region sets (Japan, USA): their differences are listed (G.$32: boot text, warning picture, copyright line, select ratings, attack join) but were not run; the World set only is the target.

## 16. (d) New rows for `mechanics-seen.md`

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| F25 | The attract state is kept across games: after a game over the loop resumes in the state it was in, after one ranking screen | M: d4d-b16 | MISSING | 3.1 |
| F26 | Boot screens end early when player 1 or 2 presses attack or jump (after 30 frames) | M: d4d-b14 | MISSING | section 2 |
| F27 | Debug mode at power on (P1 up, right, attack, jump, coin 1): DIP bits become cheats, a pause task; the skip-wait task 0x9E38C doubles sleep speed in scripted scenes while a button is held | R: 0xA6C0, 0x9E38C | n/a | section 1 and 10 |
| F28 | Continue countdown: 20 lasts 100 frames, others 60; attack or jump skips one number per press from 17 (mode 0x10) or 18 (in level); coins only add credits | M: d4d-b3 | MISSING | 6.3 (replaces P33 notes) |
| F29 | Join and continue also add stage time (+3:30 join, +4:00 continue, cap 4:00), +600 frames to the wave timer, +1 point to the score of all three players, and cost 1 credit (2 in the "2 coins start" mode, continue 1) | M: d4d-b19, d4d-b3 | MISSING | 6.2 |
| F30 | Tally rules: (HP − 1) * 100 or 20000 at HP 100; kills * 100; final blow 20000; M.V.P. 10000 for the most damage (ties share); count down 100 per frame; end frame = 177 + max_total / 100 + 268 | M: d4d-b7, b8, b5 | STUB | 9.1 (replaces F10 notes) |
| F31 | Name entry exists (game over and ending): 30 glyphs (A to Z, comma, point, RUB, END), 3 letters, 420 frame idle and 20 second limits, profanity filter, parked during the ending | M: d4d-b4, b6, b18 | MISSING | 7 (replaces F20 notes) |
| F32 | The ending is independent of hero and player count (only the explosion randomness differs); fixed cast Mustapha and Mess walk, Jack and Hannah drive | M: d4d-b5 | MISSING | 8 |
| F33 | The demo resets the random generator (G.$4 to G.$E = 0, G.$14 = 0x1C3) and replays recorded input streams (30 streams, 0xC869A) | R: 0x99F10, 0x9A146 | MISSING | 3.6 |
| F34 | Anti-tamper checks (0x6A73A at the title, 0xD860 in the tally, 0x1F18A in the first panel, 0x86B3C in the join task, 0x104F6) never fire on a real ROM: do not port | R | n/a | replaces F23 |
| F35 | The rank at the start of a game depends on the frames the machine waited with a credit (G.$4F6 counted by the system task, bands 720, 3600, 10800) | R: 0x1827A, 0x17F8A | PARTIAL (`rank.rs` has the bands, no feeder) | resolves F12 |
| F36 | Free play: no press-start screen, start works at once from any attract moment, FREE PLAY texts; credits are capped at 9; SERVICE adds one credit to every player | M: d4d-b15, R | MISSING | 3.10, 4.1 |
| F37 | In the select, no two players can hold the same hero, cursors skip taken heroes, joining resets the timer to 10, auto pick presses attack for all at 0, pans 0x00, 0x0B, 0x15, 0x20 for sounds 0x32 and 0x33 | M: d4d-b4 | STUB | 5 |
| F38 | The map panel appears twice: at the end of the tally (slides in, marker blinks 120 frames) and in the intro card of the next episode (slides in, typed EPISODE title 141 frames, slides out, HUD slides in) | M: d4d-b5, b9 | MISSING | 9 |
| F39 | The sound filter: in attract and demo only codes 0xFFxx and 0x0030 reach the sound chip unless the demo sound setting is on (factory: off) | R: 0x610 | MISSING | 3.11 |
| F40 | In-level prompts of absent players alternate INSERT COIN and JOIN-IN when there is no credit (128 frame cycle), JOIN-IN only with a credit (64 frame cycle) | M: d4d-b13 | STUB | 6.1 (extends F18) |
| P43 | Join: attack alone also joins a player with a credit (three chute mode, not in the Japan set); the strip starts on the player's previous hero; a hero is held only by players in strip or play state | M: d4d-b20, b3 | MISSING | 6.2 |
| P44 | A joining or continuing player gets HP 100, extra lives (1), a bazooka, 360 frames of blinking invulnerability, drops in at camera X + 0xC0 | M: d4d-b3 | PARTIAL (`item_give_bazooka`) | 6.2 |
| P45 | The hero can be chosen at three moments only: select, join strip, continue strip | M, R | STUB | 6.5 (answers P41) |
| F41 | Story panels: Episodes 4 and 7 show the first unused hero as the speaker, Episodes 4 and 5 use the first player's hero name, Episode 6 shows the first player's portrait (blank for Hannah alone); attack skips any panel | M: d4d-b21, R: 0xE034, 0xE0AA, 0xDFCC | MISSING | 9.3 |
| P46 | The continue screen keeps the level frozen and restores it after a continue (about 41 frames after the press) | M: d4d-b3/k-coin-late | MISSING | 6.3 |

## Appendix A. Probes used (all headless, one MAME run per batch)

Batch files are in `re/out/d4d/` (b1.txt, b2.txt, b3.txt, b4.txt, b5.txt) and `re/out/d4d/tools/` (b6 to b20 and the generators `mk*.py`); outputs are in `re/out/probe/d4d-bN/SEGMENT/` (git-ignored). Analysis scripts (no ROM data):
`re/out/d4d/tools/an.py` (readable change log with names and sound codes from the ring), `an1.py` (attract log), `objs.py` (object spawns), `cuts.py` (hard cuts from snapshots), `diffsnap.py`, `rom.py` (string and table decoders), `summ.py` (call summary of a code range).

| Batch | Segments | Purpose |
| --- | --- | --- |
| b1 | attract | 32000 frames from power on, G fields and the sound ring each frame |
| b2 | c-title, c-prof, c-rank, c-demo, c-slides, c-movie, c-boot, c-warn, c-nostart, s-nocredit, s-nocredit-demo, c-two, c-three-start, c-title-coinhold | coin and start at every attract moment |
| b3 | k-1p, k-mash, k-coin-late, k-nocoin-start, k-coin2, k2-timeout, k2-coin, k2-both, j-p2, j-p2-wait, j-p2-moves, j-p23, j-nocredit, j-coin-only, j-coin-start-3p-char, k-hiscore (invalid: extra life) | continue and join |
| b4 | k-hiscore, k-hiscore-mid, k-hiscore-5th, sel-1p, sel-1p-moves, sel-2p-late, sel-same, sel-3p-all-left, sel-3p-timeout, sel-picks-sim | name entry, select |
| b5 | t-1p-base, t-1p-kills, t-1p-fb-lowhp, t-3p, end-jack, end-hannah, end-mess-jack, end-3p, end-mustapha, end-hiscore (parked entry, invalid pokes) | tally pictures and endings |
| b6 | end-hiscore | ending name entry (score poked 30 frames before) |
| b7, b8 | tv-hp*, tv-k*, tv-fb, tv-2p-*, tv-3p-a, td-* | tally formulas, M.V.P. |
| b9 | link-ep2 to link-ep8 | story panel and intro timing |
| b10, b12 | att-slides, att-movie, att-profiles, end-fine, mov-fine, slides-fine | pictures every 10 or 20 frames, cut detection |
| b11 | end-objs, mov-objs | object spawns by frame |
| b13 | blink-nocredit, blink-credit | HUD prompt rhythm |
| b14, b15 | boot-skip-a/b/p3, freeplay | boot skip, free play |
| b16, b17 | go-attract, att-counters | resume after game over, part frames |
| b18 | k-name-entry | typed name |
| b19 | time-join, time-join-cap, time-cont | stage clock |
| b20 | join-by-attack, join-by-jump, cont-by-attack, join-by-attack-nocredit | join buttons |
| b21 | pan-ep4..8 with 6 parties each, pan-skip-ep2, pan-skip-ep2-jump | story panels by party, skip |

Known invalid early tests (fixed in the final batches): a poked score of 500000 gives an extra life (b3 k-hiscore, old `fe-highscore`); poking the ending flags and the score in the same frame leaves the rank byte unset (b5 end-hiscore).
The probe did not need changes.

## Appendix B. Data to extract (ROM address, approximate length, use)

Lengths are read to the next label and must be checked by the task; every item is an exact table copy for `dino-assets/src/text.rs TABLES`.

| Address | Bytes | Use |
| --- | --- | --- |
| 0x9DDAC to 0x9E38C | 0x5E0 | boot text pointer tables, 3 region texts, 3 palettes |
| 0x97A20 to 0x97C20 | 0x200 | press-start background block lists |
| 0x97F4B to 0x980B0 | 0x165 | title strings |
| 0x988A4 to 0x988C0 | 0x1C | INSERT COIN and FREE PLAY strings |
| 0x9A5E6 to 0x9A7E6 | 0x200 | title background block lists |
| 0x9A8E4 to 0x9A92E | 0x4A | profile timeline and handler offsets |
| 0x9ABF6 to 0x9AC4A | 0x54 | profile labels |
| 0x9B0F8 to 0x9B342 | 0x24A | profile card texts (tile codes) |
| 0x153CA, 0x15508 to 0x15559, 0x156A8 to 0x156F0 | 0xA, 0x51, 0x48 | ranking row table, label block, face tiles |
| 0x15F34 to 0x15F54, 0x15FF4 to 0x160E4, 0x161B2 to 0x161DA | 0x20, 0xF0, 0x28 | name entry glyph map, glyph tiles, profanity lists |
| 0x9A13A to 0x9A36E | 0x234 | demo input addresses, rows, stream pointer tables |
| 0xC869A to 0xC9EE6 | 0x184C | 30 demo input streams |
| 0x98BD6 to 0x98FF0 | 0x41A | slide block lists and texts |
| 0x9C002 to 0x9C04A, 0x9C280 to 0x9C29C, 0x9C6C8 to 0x9C788, 0x9CAA0 to 0x9CFC6 | 0x48, 0x1C, 0xC0, about 0x526 | movie tables, lists, texts |
| 0xAEFC to 0xAF50, 0xB07E to 0xB0A6, 0xB10A to 0xB122, 0xB24A to 0xB362, 0xB396 to 0xB450, 0xB47C to 0xB67C | about 0x500 | select tables, names, stats, rating lists |
| 0x83EB to 0x8440, 0x84D6 to 0x86BC, 0x9D038 to 0x9D048, 0x9D666, 0x9D984 to 0x9DBC0 | about 0x500 | credit text and prompt glyph blocks, coin ratios |
| 0xBAAE to 0xBBC2, 0xBD74 and the glyph strings of the tally (0xBEA2, 0xBF9A, 0xC0E8), 0xC6D2 to 0xCA20 | about 0x600 | tally tables and strings, intro title pointer table and the 7 titles |
| 0xD94E to 0xDFA2 (lists), 0xE42A to 0xE900 (texts) | about 0x1000 | story panel lists and texts |
| 0x8B7EA to 0x8BB04 | 0x31A | staff roll lines |
| 0x145DC8 + 0x80 * page (pages 1 to 0x1B), 0x177288 + 0x100 * page (pages 0x60 to 0x7F), 0x15CFE8 + 0x40 * index and the page index lists of the scroll1 pages 1 to 6, cells 0x175378 | per page | the scene pictures of the picture_calls list (title, select, continue, slides, movie, ranking, ending) |
| 0xBCA52 + 0x3400 and 0xC2252 + 0x3000 (palette ROM for the movie logo), scene palettes 0x89E6 sets 0 to 5 | | palettes for the scenes (already in the gfx extraction for stages, check the scene sets) |

