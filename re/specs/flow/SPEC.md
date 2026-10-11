# Flow and HUD: Cadillacs and Dinosaurs (CPS1, dino World 930201)

Tags: [O] observed in the disassembly, [I] inferred, [U] unknown. Addresses are 68000 ROM
addresses; `G.$x` = RAM 0xFF8000 + x. Helpers: `dc` (Ghidra C by address), `strtab.py`
(word-offset string tables). Scratch notes of the two research passes: `scratch_a/`, `scratch_b/`.

## 1. Scroll1 coordinates [O]

- Scroll1 VRAM 0x908000..0x90BFFF. Cell = code word + attribute word. +4 = next row, +0x80 = next
  column, +0x2000 = row 32. col = (off & 0x1FFF) >> 7, row = (off & 0x7F)/4 + 32*bit13.
- Registers (vblank 0x6C0): X = G.$6C4 - 0x40, Y = 0x100 - G.$6C8 (last frame's values).
- In game, title and press start: G.$6C8 = 0, so rows 34..61 are visible (screen y = (row-34)*8).
  The stage intro sets G.$6C8 = 0x100 (rows 2..29) while the HUD slides in. So `cam1_y` must be
  0 in game (camera.rs default 0x100 is only right during the intro).
- 0xB52 clears scroll1 with code 0x20, attribute 6. 0xAE0 clears all layers and cameras.
- print_string 0xC0A: string = attribute byte, text, '@'; attribute |= 0x180; '/' = next row at
  the first column; the character byte is the tile code.
- 0xF94: clear rectangle (col, row-32, width, height) with code 0x20 attr 0x180.
- Other formats (from scratch_a): 0xBB6 / 0xBAE positioned byte strings (col, row-32, attr, text,
  '/' = new positioned segment); 0xC44 / 0xC4C word strings; 0xC9A word strings with attr word.

## 2. HUD layout (3-player layout; G.$4CF = 0) [O]

Per player i, block base = 0x90A008 + 0x800*i (col 16*i, row 34) (table 0x1632A).

| item | cells | routine |
|---|---|---|
| face 3x3 | cols 0-2, rows 34-36; tiles T[0x163D4+2*chr] (+1 per column, +0x10 row 35, +0x3A0 row 36), attr T[0x163DC+2*chr] | 0x16396 |
| "=" (tile 0x3D) | col 3, row 34, attr 0x180 | 0x16396 |
| extra lives (tile = count) | col 4, row 34, attr 0x180 | 0x163E4 |
| name (0x1641E: JACK.T, MUSTAPHA, HANNAH.D, MESS.O) | cols 3.., row 35, attr 0x180 | 0x163F4 |
| vitality bar | cols 3-15, row 36, attr 0x18D | 0x16446 |
| score, 8 BCD digits, no leading zeros | right digit col 15, row 34, tile 0x124+digit, attr 0x181 | 0x101B6 |
| rank "1ST".."10TH" | cols 12-15, row 35 (table 0x15ACE) | 0x15974 |
| message area (enemy / item) | base 0x90A014+0x800*i: portrait cols 0-2 rows 37-38, name col 3 row 37, bar col 3 row 38 | 0x16BEA |
| weapon area | message base +0x5C (rows 60-61) | 0x2389C |

2-player layout (G.$4CF = 1): blocks at col 0 and col 26 (0x1638E), 24 columns wide, score right
digit at 0x90AA88 / 0x90B788, bars scaled by 1.5 (0x16548, 0x17676).

### Player vitality bar 0x16446 [O]
hp' = hp, minus 1 unless hp is 0 or 100. Right end (col 15): 100 -> 0x983, hp' <= 0x5E -> 0x989,
else T[0x1653C + 2*((hp'+2)&7)]. Segments cols 14..4 (d5 = 11..1): level = (hp'+2)>>3;
level == d5 -> T[0x16520+2*((hp'+2)&7)], level > d5 -> 0x903, else 0x90B. Left end (col 3):
hp' >= 6 -> 0x90C else T[0x16530+2*(hp'&7)]. Called by the damage / heal code right after HP
changes (0x11CAA, food 0x28ADC, ...). There is no drain animation on the player bar.

### Low vitality blink 0x1667E (object phase, after 0x100DA) [O]
Timer G.$50B counts 5..0; on borrow the phase G.$50A toggles (every 6 frames). A present player
with HP <= 15 (signed) sets +$114 and rewrites the attribute palettes of rows 34-35, cols 3-15
from table 0x1677C[phase] (phase 0: palettes 0/1 = normal, phase 1: palette 2). When HP goes
above 15 again: phase = 0 (global quirk) and the normal palettes are restored.

### Score, extra life, rank [O]
- add_score 0x10036: value = BCD long at 0x10292 + d0 (byte offset), credited to player +$02 of
  the object (negative = nobody; 0 P1, 1 P2, else P3); `abcd` chain; carry -> 99999999; sets
  dirty flags G.-$7134 (any) and G.-$7133.. (per player).
- 0x100DA (object phase): if G.$4D5 = 0 and dirty: rank texts of all players (0x15948), then per
  dirty player: draw score, extra-life check 0x6B98.
- 0x6B98: once per player (+$10E): thresholds list 0x6BFE[G.$4E5]; factory setting -> 500000.
  On pass: +$10E = 1, +$11C = 0xF0, lives + 1, rank 0x1843C, sound 0xA4CA, redraw block.
- Rank 0x15A2E: +$10A starts at 0x33; moves up past every high score entry (table G.$703E,
  16-byte entries) whose score it reaches (signed compare); +1 for each present player with a
  higher score. Shown as "nST/ND/RD/TH" (suffixes 0x159F8) when <= 10, else 4 spaces.
- Default high score table 0x1504E (boot): 50 entries, score 100000 down by 10000 in BCD; entries
  11..49 wrap to 99990000.. which are negative as signed longs, so a new player starts at rank 11.

## 3. Message area (enemy name and bar, items) [O]

Rings: 3 x 16 entries x 16 bytes at G.$6BDE / $6CDE / $6DDE, write offsets G.$6BD8/$6BDA/$6BDC,
read offsets G.$6BD9/$6BDB/$6BDD (+0x10, wrap 0x100). Free entry = word 0 negative. Init 0x4CEE.

Producers (queue into the ring of a player):
- 0x16B18 `hud_hit(e, kind)`: player = e's last attacker (+$68) if it is a player and not e.
- 0x16B36 `hud_hit_credit(e, kind)`: player = e's +$02.
  Entry: kind, name id (+$03), +$6A (bar length, max 100), +$6E (HP before hit), +$6C (HP), ptr.
  Kinds: 0 enemy hit (25 callers), 1 follow the object while grabbed (2), 2 hit on a player
  (portrait and name from the player tables 0x17A82 / 0x17D70).
- 0x1693C kind 3 food (item +$6C = 100 healed / 99 points, +$26 food id).
- 0x16A5A kind 4 treasure (+$26 treasure id).

Consumer 0x16CE6 per frame per present player (slot struct G.$6BA8 + 16*i: +1 delay, +2 step,
+3 timer, +4 kind, +5 blink, +6 object, +$A VRAM base, +$E/$F saved kind/id):
- Empty ring: kind 1 keeps the display while the object's frame class is not 0xA and its state
  is 0x206/0x208; otherwise the 60-frame timer counts down and clears the area at 1. With
  timer 0 and a boss set (G.-$70BC, set by boss inits), the boss bar is shown (0x17190).
- Message: delay counts first. Kind 3/4 -> food / treasure text (0x23F24 / 0x24312: icon 3x2
  attr 0xC, name at +0x180, " GOOD !!" or points / treasure points at +0x184).
- Enemy: step 0 draws the bar with the HP before the hit, portrait (0x179B6) and name
  (0x17D2E), delay 3; step 1 redraws with the current HP, delay 10, timer 60, advance. HP 0
  starts a 60-frame portrait blink (skull tiles 0x17436 attr 0x18B, every 2 frames).
- Bar 0x174F2: length from +$6A (left cap 0x91C, full 0x913, empty 0x91B, right end 0x999,
  partial tables 0x175EE / 0x17600 / 0x1760E; 1 tile = 8 HP). HP above 100 (150 in 2P) changes
  colours 3-5 of scroll1 palette 13/14/15 (by player) from table 0x17892 per extra bar.

## 4. Game task 0xA870 and modes (G.$4D0) [O]

Loop: sleep (G.$4D6 & 15) frames (1; 4 after a boss kill = slow motion), G.$0000++, G.$4DA++.
| mode | routine | what |
|---|---|---|
| 0 | 0xA978 | task 0x6C4A (join/continue) slot 0x10; clear layers; rank 0x17F8A; 0xA9BE -> mode 4 |
| 4 | 0xAD46 | character select (section 6) -> 0xAC4E |
| 8 | 0xD898 | intermission cut-scene per stage (blocking) -> 0xAC4E |
| 0xC | 0xAAF6 | game frame, then the G.$4D1 switch 0xAB2C |
| 0x10 | 0xE9E6 | continue screen when nobody plays |
| 0x14/0x18 | 0xAC06/0xAC26 | game over -> attract task 0x97C20 |
| 0x1C | 0xA69B8 | ending |

0xA9BE (every stage): palettes 0x8A44, brightness 0x892C, scroll reset 0x20402, clear G.$62C,
rank 0x1800E, layer ctrl 0x12D6, G.$4D4 = 0, HUD init 0x16894, rings 0x4CEE, mode 4, pool E
init, 0xAA0E (player start positions 0x4E0A from table 0x4E56, scripts 0x1DA10, lists, pools A-D,
pool E loop, clear G.$4F1.., rank 0x18048, time 0x1031A, floor 0x12BE4, spawn 0xFBE8).

0xAC4E stage start: 0xAE0, G.$6C8 = 0x100, intro task 0xB9BE, 0xA9BE, mode 0xC, 0x655E, HP 100,
G.$7B4 = 0, sound 0xFF00, music (stage 0: 1, else 0xA55A table 0xA582).

Intro 0xB9BE: stage 0 starts at state 4 sub 2: hold cam1 at 0 until G.$4E3 (players' walk-in
script), then HUD redraw 0x162AE and slide-in: each frame copy 32-cell columns from the lower
half to col 48+ of the upper half, G.$6C4 += 12, until > 0x180 (0x3F0 other stages); then cam1
= 0, G.$4D5 = 0. Other stages first show the map panel and "EPISODE n" title (scratch_b).

G.$4D1 switch (0xAB3A): 2 wait boss (G.$4E0) state >= 4 -> speed 1, timer 1800, 4; 4 timer or
G.$4E3 -> clear task 0xB97E, G.$4E2 = 1, timer 1800 (3600 stage 7), 6; 6 timer or G.$4E2 = 0 ->
stage+1, area 0, mode 8 (stage 7: mode 0x1C, G.$4D7 = G.$4DB = G.$507 = 1).
Boss defeated 0xACEE: G.$4D4 = 1, G.$4D6 = 4, G.$4D5 = 1, G.$4E0 = boss, G.$50E = 1, G.$4D1 = 2,
rank speed +1, clear absent players' HUD blocks 0x165FC.
Clear task 0xB97E: tally (VITALITY, ENEMIES, FINAL BLOW +20000, M.V.P. +10000), bonus count-up,
next-stage map; details in scratch_b.

## 5. Camera hooks [O]

| hook | does |
|---|---|
| 0xA5EA | sound command 0xFF07 (music fade out) |
| 0xACD0 | 0xAA0E area start, area music 0xA55A, HUD redraw 0x162AE, 0x655E, rank speed +2 |
| 0x12BE4 | floor table G.$6AC/$6B0 by stage/area (read by 0x12C90, 0x133E0); vertical scene draws at 0x91C000 |
| 0x162AE | HUD redraw: blocks of present players (0x162F8), scores and ranks (0x101E6) |
| 0xAAE2 | 0x1031A stage time, 0x12BE4, 0xFBE8 spawn reset |
| 0x1031A | stage time from table 0x1033E |
| 0xACE0 | 0x655E (G.$4EF = players in state != 0, minus 1), rank speed +2 |
| 0x9236C | pool B object class 0x60 (+$20 = 0x180) |

## 6. Front end, select, join / continue

Summarised from the research passes (full notes with addresses in `scratch_a/` and the fork
reports quoted in `HANDOFF.md`):
- Inputs [O]: G.$1A bit0-2 START1-3, bit6 TEST, bit7 SERVICE; G.$20 bit0-2 COIN1-3; previous
  G.$1A in G.$1F (edge = cur & ~prev).
- Settings [O]: G.$777A/B/C come from EEPROM; factory defaults (0xACBF4) give 0x80 / 0x4B / 0x41:
  1 coin 1 credit, 3-player layout, difficulty 3, rank speed 1(+1), extra life 500000, 1 extra
  life, continue on.
- System task 0x9D048, attract 0x97C20 (title 0x9A3CE 600 frames, INSERT COIN blink 30/30
  frames at (18,56)), press start 0x983EE, character select 0xAD46 (10-second timer, cursor
  order Jack, Hannah, Mustapha, Mess, no shared cursor), join / continue task 0x6C4A (player
  +$A2 states 0..0xC, CONTINUE countdown 20 -> 0 at 60 frames per number).
