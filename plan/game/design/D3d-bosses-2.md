# D3d: Bosses part 2 (Morgan, Morgue, Tyrog, Slisaurs, tank creatures, Dr. Fessenden, ending trigger)

**STATUS: COMPLETE (2026-10-13).** All sections are written. MAME budget: 4 of 5 probe runs used (no GUI, `re/harness/probe/probe.sh` only).

Scope: design numbers and ROM addresses only. No code was changed. Nothing here is derived from art; no raw disassembly or byte dumps are kept in this file.

Source tags: **R:** ROM code or table (address given), **M:** MAME probe segment (name given; the segments are under `re/out/probe/d3d-r1` to `d3d-r4`, scripts in `re/out/d3d/`), **W:** web source (and which file), **U:** unknown or not verified.

Sections: 1 shared facts, 2 Morgan and Morgue, 3 Tyrog, 4 Slisaurs, 5 lab tanks, 6 Dr. Fessenden, 7 the ending trigger, 8 checks of the leads, 9 (a) acceptance scenarios, (b) port status and task list, (c) open items, (d) new rows for `mechanics-seen.md` (BT01 to BT22), 10 files used.

## Summary of the findings

- **Slisaurs (Episode 7):** the starter is pool B class **33** (not 41). It makes **2 Slisaurs (3 when `G.$4EF != 0`, that is 2 or 3 players)**, class 71, at X 0x580. HP rows are the same as Slice's (450 / 600 / 700 at level 10 by player count), score 70,000 each, no wake-up super. They throw blades (class 19, single or a fan of 5, 7, 9) and dash up to 7 px per frame with an afterimage trail (pool C class 27). No Slisaur exists outside Episode 7, so the user note "Episode 3 has one Slisaur" is false. [section 4]
- **Tanks (Episode 8):** the hall has 11 painted glass cylinders, but only **four** open (01, 03, 04, 11 by script), not 1, 3, 5, 7. They release a Bludge, a Rock Hopper, a Bludge and **Tyrog 2** (class 91). The web swaps jars 3 and 4. [section 5]
- **Morgan and Morgue (Episode 5):** the villager scene (four shots at t = 406 to 418), Morgan (class 36, 256 HP, **0 points**, Uzi, grenades, knife / rock / stone throws, hop; one Uzi shot slides him 108 px), his change into Morgue (class 40, 30,000 points) who roars and charges at 7 px per frame. [section 2]
- **Tyrog (Episode 6):** a steel crate with four hunters breaks after 320 frames and class 76 jumps out; then **one slot is rewritten again and again**: class 4 var 2 (green: claws and chest blades) → class 72 var 1 (the host, 250 frames) → class 4 var 1 (orange: fireballs) → class 72 var 0 → class 4 var 0 (grey and blue: everything, with a lava volley on a cross of five spots). Each form has 256 HP (halved index 6) at level 10, only the last form gives 60,000 and calls `0xACEE`. The host (450 HP, poise 1) would give 100,000 and end the stage if killed. Tyrog 2 is a 300 HP copy with the fight 3 tables. [section 3]
- **Fessenden (Episode 8):** the script spawns **class 94 first** (a clone of Morgue: human for 276 frames, then a dino man with a much lower attack gate, 8,000 points, no helpers), and after a 1,224 frame death talk with five captions it makes **class 84** (the pink two-headed dinosaur with five parts, 640 HP, a counter reaction to jump attacks, aimed fireballs and a jaw grab). This reverses the order of the older documents. `0xACEE` is called only by class 84 at HP 0; the ending starts 3,600 frames after the tally begins. [sections 6 and 7]
- **Rejected or corrected leads:** bosses' HP is not fixed by rank (M06); Slisaurs have no wake-up super (M10); tanks are 1, 3, 4, 11; "six hits" is four jabs in the test (M13); "one hit chance in the small-head phase" is not a rule (M12). [section 8]

## 1. Shared facts (the boss framework of the four episodes)

### 1.1 Class and address map (corrected)

| Boss part | Class and handler | Notes |
| --- | --- | --- |
| Morgan scene actors | A **35** 0x513DA (696 B), A **45** 0x5639A (466 B), A **67** 0x5BE66 (the rifle prop) | 35 turns into 36 by rewriting its class word [R 0x51658] |
| Morgan | A **36** 0x51692 (6,886 B), name 0x15, HP index 4 | gives **no kill score** (no call of 0x325E6) |
| Thrown things | A **68** 0x5C0B6 (thrown object, art table 0x71408), A **26** 0x4D48A (grenade, same art), A **59** (bullet 0x57FB4, creator 0x57F62) | [R] |
| Morgue | A **40** 0x53E08 (6,750 B), name 0x16, HP index 5, kill score 30,000 | made from Morgan by rewriting the class word [R 0x52EBE] |
| Tyrog scene | pool B **79** 0x8A3D0 (starter), pool A **75** 0x5E702 (actors and small things), **76** 0x5EC26 (the creature maker) | [R] |
| Tyrog | A **4** 0x36A64 (9,338 B), name 0x19, boss index 6, three forms by the high byte of `+26` (2, 1, 0) | the three fights |
| Tyrog host | A **72** 0x5DD58 (1,904 B), name 0x1B, HP index 0 | the human body that is taken over |
| Tyrog 2 | A **91** 0x654E2 (8,732 B), name 0x23, normal enemy HP | a 94 percent copy of class 4 for Episode 8 |
| Tyrog projectile | A **74** 0x5E4F4 (530 B) | the chest spike child [R] |
| Slisaur | pool B **33** + A **71** + A **19** + C **27** | section 4 |
| Tank breaker | pool B **44** 0x83BB0 | section 5 |
| Fessenden first form | A **94** 0x67D2C (7,218 B), name 0x25, HP index 8, kill score 8,000 | **a clone of Morgue (class 40)**; spawned by the script record at trigger 0x10A0 |
| Fessenden final form | A **84** 0x6173E (8,250 B) main body, parts A **86** 0x63806, **87** 0x63BC4, **88** and **89** 0x63E30 (one handler for both), **85** 0x63778; name 0x22, HP index 9, kill score 1,000 | made by class 94's death at 0x698E8 |
| Fessenden fire | A **95** 0x6995E (fireball and flame), creator routine 0x69932 | [R] |
| Lair debris | A **92** 0x676FE, **93** 0x67B32 (pool B 83 to 85 make them), A **90** 0x650FA | ending and quake pieces |

**Corrections to earlier documents.** `D2c` table 16.3 and `episode-8.md` call class 84 the first form and class 94 the final form. The truth (script, MAME `f94-intro`, `f94-trans`, `f84-trace`): the script spawns **class 94 first** (Morgue-like fighter, human in a white coat for its first 276 frames, then a pale green dino man), and when it dies it creates **class 84** with its five parts, the big pink two-headed dinosaur (the final fight). HP indices follow: class 94 uses index 8 (400 / 520 / 650 at rank 0), class 84 uses index 9 (600 / 680 / 800 at rank 0) [R 0x67D8A, 0x6189E]. The score list reads the same way: the first form gives 8,000, the last 1,000.

### 1.2 The shared AI skeleton of Morgan, Morgue, Fessenden's first form and Tyrog (class 4, 72, 91)

All of these bosses are written on the same frame, which the port should build once and fill with tables [R, listings compared with `re/out/d3d/cmpcls.py`: class 94 and class 40 are 90 percent identical, class 91 and class 4 94 percent]:

1. **Frame order** (class 36 at 0x517F8, class 40 at 0x53F10, class 4 at 0x36B9A): clamp X into the arena while alive (`camX - 0x10 .. camX + 0x190` for Morgan; Morgue and Fessenden `camX - 0x50 .. camX + 0x1D0`), the `+5` sub-routine (0 AI, 2 hit reaction, 4 dying, 6 grabbed 0x2B35A), then `0x193E` (the standard clamp).
2. **Phase flag `+A0`** (0x51928, 0x540FA, 0x36CF4): a countdown `+A6` flips the boss between **0 = defensive** and **1 = aggressive**. After a flip into aggressive the next flip comes after `T1[rand & 0x1E]` frames, after a flip into defensive after `T2[...]`: Morgan and Tyrog use `T1 = {200, 220, 220, 240, 240, 260, 260, 260, 280, 280, 280, 300, 300, 320, 320, 360}` frames and `T2 = {90, 90, 90, 90, 120 (x8), 150 (x4)}`; Morgue's tables are at 0x5418A and 0x541AA. At each flip the target is re-picked (0x32032) [R, M m5-a: `+A0` 0 to 1 at t = 816 and 1 to 0 at t = 1096].
3. **State byte `+6`** (even values) is the AI state; `+AC` a word timer; `+A1` "locked in a move" flag (the pick routine is not run while it is 1); `+A5` the **X distance class** of the target (function 0x3293C for Morgan, Morgue and class 91, 0x329B8 for class 4); `+A9` the last direction (turn smoothing 0x5246A); `+B0/+B4` long step vectors.
4. **Decision** (0x52BFC for Morgan, 0x55704 for Morgue, 0x38B90 / 0x38C36 for Tyrog): in the aggressive phase it rolls an attack gate `(rand & 31) < G[level][class]` (Morgan table 0x73B14, Morgue 0x73D54, Tyrog 0x74542 / 0x74582 / 0x745C2 by form), and on success picks a move from a table of 32 rows by 8 distance classes. Defensive phase: move to a point (Morgan 0x522A8 kite, 0x523B2 approach).
5. **Registration**: every frame `0x49CA` (draw), `0x4BEA` (hurtable list), and class 74 / 95 and the thrown objects `0x4C0A` (monster list).
6. **Death and boss end**: the hit handler (0x52D7A, 0x557F4, 0x38CC6) tallies damage (0x3262A) and at HP 0 raises `G.$4D6` and `G.$50E`; the boss defeat routine `0xACEE` is called by Vice, Butcher, Slice, Hogg (D3c), **Morgue (0x55806), class 4 form 0 (0x38CFE), class 72 (0x5E460), class 84 (0x63492), the Slisaur starter (0x819F6)** and not by Morgan, class 94 or class 91.

### 1.3 Boss HP, score, music, flags (all by the ROM tables, levels as measured in MAME at rank 12, one player = level 10)

| Boss | HP index | HP at rank 0 (1P / 2P / 3P) | HP measured | Kill score | Music code (table 0xA5C8) | `+7D` | `+63` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Morgan | 4 | 256 / 320 / 512 | 256 [M m5-a] | 0 | 0x0E (written at init, 0x517D4) | 0xFE | 2 |
| Morgue | 5 | 512 / 576 / 768 | 512 [M m5-b] | 30,000 | no new request | 0xFF after init | 2 |
| Tyrog fight 1 (class 4, var 2) | 6, then halved | 256 / 288 / 352 | 256 [M ty-chain] | none (the death routine skips 0x325E6 for forms 2 and 1) | 0x0F (0x36B6C, only var 2) | 0xFE | 2 |
| Tyrog host (class 72) | 0 | 350 / 550 / 650 | 450 [M ty-full-c] | 100,000 (0x1B row, call at 0x5E48C) if the host dies (this is the web's "speedrun finish gives 100,000"); its death also calls `0xACEE` (0x5E460) and ends the stage; **never reached in the natural chain** (it is only hurt for 14 HP at most) | none | 0xFE | 2 |
| Tyrog fight 3 (class 4, var 0) | 6, halved | 256 / 288 / 352 | 256 [M ty-full-c] | 60,000 (0x38D56) | none | 0xFF | 2 |
| Tyrog 2 (class 91) | normal enemy table (0x31424, name 0x23) | | 300 at level 10 [M] | 1,000 | none | 0 | 0 |
| Slisaur | 0 | 350 / 550 / 650 each | 450, 600, 700 each [M] | 70,000 each | 0x11 | 0xFF | 2 |
| Fessenden 94 | 8 | 400 / 520 / 650 | 450 (spawned at level 10), 550 (script spawn at the jump rank) [M] | 8,000 | 0x0F (0x687A8) | 0xFE | 2 |
| Fessenden 84 | 9 | 600 / 680 / 800 | 640 [M f84-dmg] | 1,000 (0x63552) | 0x12 (0x61882) | 0xFF | 4 |

Notes [R D2c 16.3, M]: boss HP depends on the level (8 bands, `(level & 0x1C) >> 1`), so the supplement claim "bosses have a fixed HP regardless of rank" (M06) is **false**; helpers per count are `{4, 5, 6}` alive (0x7BFEE) as in Vice's fight.

### 1.4 Damage a boss does (the formula is already ported; only the attack ids are new)

`damage = floor(floor(base * S[row][level] / 32) * hero[char] / 32 * HPfactor / 32)` with `base` from the attack table 0xA2EA4, `S` the type table 0x2C3AE (row = the boss's `+84`, name id * 32 bytes), the random cut only below 76 HP, `hero` 32 / 34 / 35 / 30, `HPfactor` 25 to 33 by the hero's HP, and the one HP mercy rule (SPEC 6.6, D2b). All six rows used here (MORGAN, MORGUE 38 at level 10; SLISAUR, TYROG, FESSENDEN, TYROG 2 42 at level 10; 79 at level 31) are in 0x2C3AE [R]. Check against MAME at level 10, hero HP 100, Jack: Slisaur 0x36 = 13 and 0x37 = 21 (formula 13 and 21), Morgue 0x6C = 20, 0x4B = 17, 0x42 = 23 (formula 20, 17, 23), Fessenden 94 0x42 = 26, 0x40 = 22 (formula 26, 22) [M sl-dmg-1p, m5-morgue-dmg, f94-dmg]. Two exceptions: **objects that never set `+84` read a stale value** (HU08: blades, bullets, and class 74 spikes show 7 to 18 instead of the scaled value) and **Morgan's own hits measured 1 point above the formula** (0x3F 15, 0x34 18; reason not found: open item 3).

### 1.5 The end of a boss fight, step by step (all bosses of this document, M)

HP 0 in the hit handler starts the reaction (slow motion period `G.$4D6 = 4` is set by `0xACEE`; Tyrog forms 2 and 1 and Morgan set 3 and 2 instead, see their sections). `G.sub` goes 2 (boss defeat) at the first frame, 4 when the boss body reaches `+4 = 4` (368 video frames later for a normal fall, 320 for Tyrog 4), then 6 after another **1,801 video frames** (1,800 frame timers of the death routines), then the tally and the next episode (`mode 8` 958 video frames later): Slisaur t = 2027 / 2395 / 4196 / 5144, Morgue t = 3927 / 4295 / 6096 / 7054, Tyrog t = 4909 / 5229 / 7030 / 7988, Fessenden t = 1527 / 1895 / 3696 / mode 0x1C at 7297 [M sl-kill, m5-morgue-kill, ty-full-c, f84-kill]. D4c T19 has the Vice numbers (94 + 278 + 951 game frames) for comparison.

### 1.6 Helper threads: the reinforcement controller (pool B class 22, handler 0x7BEF6)

Created by the boss (Morgan init 0x517EE, Morgue 0x53EDA, class 4 form 2 at 0x36B90, the Slisaur starter 0x819A4, Fessenden 84 at 0x6194A; **not** by class 94, so Fessenden's first form has no helpers, and not by Tyrog forms 1 and 0). It holds a timed list (table 0x7BF36 by stage) and a threshold list (0x7BF56); both use the 16 byte spawn records of the scripts with the first word as delay. A threshold group runs only while `G.$502 > 0`; the boss sets `G.$502 = 1` itself (Morgan when `+AC` is 0x10 in the aggressive phase 0x51A46, the Slisaur starter every 300 frames, Morgue 0x542CA), and the controller clears it after each group. Records spawn only while the live count `G.$4F3` is below `{4, 5, 6}[players - 1]`. Lists (decoded with `re/out/d3d/ctl.py`):

| Fight | Timed list | Threshold groups (alive at most 1P / 2P / 3P, one group each) |
| --- | --- | --- |
| Morgan, Morgue (stage index 4) | none | 12 groups of one thug at X 0x11C0 or 0x1380, Z 0x110 or 0x150: Ferris (alive <= 0), Ferris, Punk x2, Poacher x3, Blade x2, Hammer x3 (alive <= 1, 2, 2; the last 2, 3, 3) |
| Tyrog fight 1 (stage index 5) | none | 12 groups at X 0xFE0 or 0x11A0, Z 0x170 to 0x1B0: Poacher x3, Punk x2 (2P, 3P), Blade x2, Ferris x3, Hammer x2 (alive <= 2, 2, 3) |
| Slisaurs (stage index 6) | none | first group alive <= 99: Ferris at X 0x430 after 300 frames (level >= 20), a second one with 3 players; then groups alive <= 0, 1, 1 of Poacher, Blade, Ferris, Hammer at X 0x430 and 0x5F0 (levels 20 and more and 3 player variants), last groups alive <= 3 |
| Fessenden final form (stage index 7) | Ferris (level 31 only, after 300 frames), then groups of 4 alternate **Blades (class 63)** at X 0x1120 or 0x12C0, Z 0x110 to 0x140, delays 2400, 1800, 70, 1800, 2400, 1800, 70, 1800 frames | groups alive <= 99 (Ferris x2 with 2 or 3 players), then alive <= 1, 1, 2: Poacher, Blade, Ferris, Hammer, Poacher, Ferris, Ferris, Hammer, Poacher at X 0x1120 / 0x12C0; one group alive <= 3 |

The Fessenden helpers therefore come from the **timed list** (Blade groups of four, throwing knives) and the threshold list, and only after the final form exists [R].

## 2. Episode 5: Morgan and Morgue (area 1, trigger 0x1120)

### 2.1 How the fight starts (the village scene)

Script (Episode 5 area 1 list 2): `A#35` at X 0x1380 Z 0x150 and `A#45` at X 0x1360 Z 0x100, both at trigger 0x1120, and the class 67 rifle prop is made by class 35 itself [R stage_scripts 0xAFA94, 0xAFAA4]. Wave 6 (the last wave before it) ends with `END type 4` (boss area), wave 7 has no record (the Morgan objects run it, D3a). The scene objects stand off the right edge of the screen until the camera reaches the place; the hero has to walk right (in a probe run with an idle hero nothing is seen: M m5-a) [M m5-scene].

Timeline, frames counted from the first update of class 35 (M m5-scene, m5-a; code in brackets):

| t | What happens |
| --- | --- |
| 0 | `A#35` (old looking man with a green scarf and a gun, the future Morgan), `A#45` (the villager, an old man with a white beard in a purple robe), A#67 (rifle prop, follows Morgan's frame marker) and thugs (Ferris x2, a poacher) stand there; the place name "VILLAGE" fades on the floor (the area card) [R 0x51412] |
| about 80 | a caption window with the old man's portrait: "Thank you for coming. The Black Marketeers did this." (paraphrase) [M m5-scene] |
| about 160 | the second caption: "Because I would not help with their nasty evil plan. His name is ..." (the sentence is cut by the shot) [M] |
| 290 | Morgan (class 35 state `+5 = 4`) starts to walk left: 60 frames at 1 px per frame (X 1380 to 1320), anim 1 (walk) [R 0x5149C] |
| 352 | he stops and points the gun: 40 frames in anim 12 (0xC) [R 0x51558] |
| 392 | a voice sound (0x9D9C) and 30 frames of firing pose; **four bullets** (pool A class 59 variant 5, flying at 8 px per frame left, height 36, life 30 frames, attack 0x0B = 15 knock down) at t = 406, 410, 414, 418 [R 0x514E6; M m5-a: A#17, 16, 15, 14 created at those frames] |
| 422 | the villager is down (class 45 lies, `sound 0x79` the shot, D3a); Morgan stands for 240 frames while the caption window now shows Morgan's own portrait (a green scarfed old man): "That guy always was a big mouth!" (from about t = 520) [M m5-scene] |
| 662 to 664 | `+5 = 0x10`: Morgan rewrites its class word to 0x90 (class 36), clears `+4..+7` and calls its own init: **the boss fight starts** (HP 256 at t = 666) [R 0x5156C, 0x51658; M m5-a] |

Class 36's init (0x516A8) first runs a "two body" step: it needs two free pool A slots (`G.$65F8 >= 2`), allocates a second slot, and the slot with the **lower address becomes Morgan** (class 0x90, init phase 2) while the other becomes the **rifle prop (class 67 var 0)** linked in `+A0` (Morgan's `+AE` points back) [R 0x516B0..0x5172C]. Phase 2 (0x5172E) is the data of 2.2. The place of the fight is the right half of area 1 (X about 0x1120 to 0x1300; the arena centre used by the death walk is 0x11C0 + 0x69).

### 2.2 Morgan (class 36): data

| Item | Value |
| --- | --- |
| Name, `+84`, anim table | 0x15 MORGAN, 0x54, 0x70F02 (18 animations, shared with the scene), boxes 0x103C00 [R] |
| HP | 256 / 320 / 512 at rank 0 (boss index 4); 256 at level 10 [R, M] |
| Speed | walk speed id 7 at init (2.0 px per frame), speed id 5 (1.5) while kiting, `+B0/+B4` steps of 1 to 1.3 px for the point moves (0x51CDC) [R] |
| Arena clamp | `camX - 0x10 .. camX + 0x190` while alive [R 0x5183E..0x51876] |
| Poise | 10 (`+78 = 0xA`), `+79` refills; common reaction [R] |
| Music | sound 0x0E at init [R 0x517D6] |
| Helpers | the reinforcement controller (Ep5 lists in 1.6), requested through `G.$502` [R] |
| Score | **0**: the dying routine never calls 0x325E6 (so the supplement is right) [R 0x52DA2..0x52EEA] |
| Weapon | the Uzi prop (class 67); **drops as a pool E weapon (class 0, var 6 = Uzi)** when Morgan turns into Morgue [R 0x5BF98..0x5BFD4; M m5-trans: a blue gun lies on the floor from t = 2520] |

### 2.3 Morgan's moves

States (`+6`, `+4 = 2`, `+5 = 0`) and what they do, with the duration measured when the hero stands still (M m5-dmg, one player, level 10):

| `+6` | Handler | Role | Measured |
| --- | --- | --- | --- |
| 2 | 0x51B96 | wait (`+7A`) | |
| 4, 6 | 0x51BA2, 0x51BC0 | walk toward the target (4) or with re-aim every 6 frames (6) for `+AC` frames | |
| 8 | 0x51CDC | **walk to the point** (`+88/+8A`): steps of `+B0/+B4` every 4th frame, 2 to 3 px | 44 to 104 frames, mean 62 |
| 0xA, 0xC | 0x52078 | wait `+AC` frames in the idle pose (`+AC` = 30) | |
| 0xE | 0x51F98 | anim to its end, then decide (after an attack) | 37 |
| 0x10 | 0x51C36 | **kite**: heads for a new grid direction (tables 0x5238C and 0x52EF0, the arena is a 14 x 6 cell grid seen from X 0x11C0, Z 0x110 to 0x144) for `+AC` frames (20 to 60, table 0x5236C), speed id 0xB | 60 to 143, mean 91 |
| 0x12 | 0x52060 | wait | |
| 0x14 | 0x51BFA | strafe in depth (`+B4`), turns every `+AC` | |
| 0x16, 0x18 | 0x52154, 0x5206E | wait `+AC` then start the punch 0x526B0 | 10 |
| 0x1A, 0x1C | 0x51F70, 0x51F84 | physics until landing (the jump kick), then decide | |
| 0x1E | 0x51FF8 | **grenade**: anim end, creates class 26 var 0 beside him (X -0x24 or +0x24 by facing, Y +0x30), repeats `+AA` times, `+AA` = 4 (4 of 16), 5 (8) or 6 (4) from table 0x526A0 | 80 for one cycle in the log |
| 0x20 | 0x51FCA | **Uzi burst**: anim 7, at the anim end one class 59 bullet (var 0, offset 0x10 forward, 0x24 up, sound 0x9D9C); `+A4 = 5` shots | 40 frames, 5 shots 8 frames apart |
| 0x22 | 0x51FA4 | aim wait `+AC` = 15 frames, then the long punch 0x525B4 | 15 |
| 0x24 | 0x51FB2 | **throw an object**: anim end; creates class 68 with variant from table 0x52594 `{2, 2, 3, 4, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4}` (var 2 six of 16 = **knife**, 3 five of 16 = **rock**, 4 five of 16 = **small stone**), at X -0x10 or +0x20 by facing, Y +0x30; repeats `+BC` times (3 six of 16, 4 eight of 16, 5 two of 16, table 0x524F4) | 30 per throw |
| 0x26 | 0x51DAA | walk to the target (X +-0x20, same depth) | 37 |
| 0x28, 0x2A | 0x52168, 0x521A4 | **hop** to the far side of the hero (hero X -+ 0xB0 depending on the screen side): vy 0x380, ay -0x40 per frame, vx and vz by distance / 0x2800 steps; at landing the throw preparation 0x52504 (0x28) or 0x526EE (0x2A) | 38 |

The starts (code that sets a state and an attack id) are the following (a start routine = "move"):

| Move | Start | Attack | Anim | Next state | Notes |
| --- | --- | --- | --- | --- | --- |
| long punch | 0x525B4 / 0x52668 / 0x526B0 / 0x52738 | **0x3F** (base 12, poise 11, reaction 4 knock down, stop 16) | 6 (5 frames, 30 ticks; box 106 active ticks 15 to 26, reach 30 to 65 px in front, y 31 to 55) | 0x24 / 0x1E / 0xE / 0x18 | the arm is 65 px long |
| quick jab | 0x527AE | 0x3F | 13 (14 ticks, box active 0 to 10) | 0xE | |
| plain punch | 0x526FA | **0x08** (base 10, poise 4, reaction 0, stop 16) | 7 | 0x20 | set at the start of the Uzi state: the burst frames also carry the 10 dmg punch box |
| jump kick | 0x527EC to 0x52834 (0x1A), 0x528EC (0x1C) | **0x34** (base 15, poise 11, reaction 4, stop 12) | 5 (21 ticks) | 0x1A / 0x1C | vy 0x700 up, ay -0x40, vx = distance / 0x3800 steps capped at 0x280 (2.5 px per frame), facing the target |
| grenade | 0x52004 | class 26 var 0 (attack 0x44 base 15 reaction 0xA burn; art: spinning grenade) | 10 | 0x1E | see 2.4 |
| throw | 0x52504..0x52590 | class 68 (attack **0x45**: base 18, reaction 4, stop 12) | 11 | 0x22 | |
| Uzi burst | 0x526EE..0x5272C | class 59 var 0 bullets (attack 0x0B base 15, knock down, 7 px per frame) | 7 | 0x20 | |

**Decision** (0x52BFC): after any move `+AC = 60`, speed id 5, `+A1 = 0`, `+A5 = X class` (0x3293C), then by the phase flag `+A0`: `0` defensive: **kite** (0x522A8: `+6 = 0x10`, `+AC` from table 0x5236C, direction from the grid tables, speed id 0xB); `1` aggressive: **approach** (0x523B2: `+6 = 8` with `+AC = 60`), and every frame of the aggressive phase 0x52C50 rolls the **attack gate**: `rand & 31 < G[aggression][A5]`, table 0x73B14 (8 rows by aggression 0 to 7, 8 columns by X class; row 0 = `12 14 16 18 20 18 16 14`, each row 2 higher, row 7 = `26 28 30 32 32 32 30 28`). On success the **action** is read from 32 rows of 8 columns (table 0x73B54 when `+AB` is negative, else 0x73C54):

| X class | table 0x73B54 (counts of 32) | table 0x73C54 |
| --- | --- | --- |
| 0 to 2 | action 1: 16, action 4: 16 | actions 1, 2, 3, 4: 8 each |
| 3 | 1: 8, 2: 12, 4: 12 | 1, 2, 3, 4: 8 each |
| 4 to 7 | 2: 20, 4: 12 | 2: 8, 3: 16, 4: 8 |

Actions (jump table 0x52CD4): 0 melee (0x52732: long punch, `+6 = 0x18`), 1 go beside the target (0x5240E, state 0x26), 2 **grenades** (0x5264A: state 0x1E, `+AA` = 4 to 6), 3 **hop and throw** (0x529B6, state 0x28), 4 hop (0x52B2A, state 0x2A, throws). `+AB` starts at 0x30 and is decremented by the grenade state (0x51FE8); once it is negative the table 0x73B54 (fewer grenades) is used. [R 0x52BFC..0x52CDC; the distribution in M m5-dmg: state entries 0x8: 15, 0x10: 7, 0x20: 6, 0x22 and 0x24: 11 each, 0x28: 6, 0x2A: 3 in 2,700 frames]

**Reaction**: common code; the one special rule is that **a hit that starts a knock down while HP is above 0 pushes Morgan back**: one Uzi bullet from the hero (attack record 11 damage at level 10) knocked him down and slid him **108 px to the right in 40 frames** (vx 4 px per frame), down for 100 frames; the next bullets only flinch him for 10 damage each [M m5-uzi: HP 256 to 245 at t = 859, then 234, 224, 214, 204, 194]. So the supplement and FAQ ("Uzi shots knock Morgan off screen when timed") are **confirmed** [W supplement 2].

### 2.4 The thrown things (classes 68, 26) and the bullet

- **Class 68** (0x5C0B6): variants 0 grenade (anim 5), 1 dynamite (anim 6), 2 knife (anim 7), 3 rock (anim 8), 4 stone (anim 9) of the art table 0x71408 [M `re/out/assets/anims/actor_a26` strips]. Init: attack 0x45, `+7D = 0xFF`, takes the target `+76` and the point `+88/+8A` of its owner; flies in an arc (`vy = 0x3A0`, `ay = -0x40` per frame; the horizontal speed comes from the distance to the target, the exact step rule is not decoded, open item); at landing variant 1 creates a **pool E weapon item 0x18 = dynamite** (so a dynamite lies on the floor), the others vanish; variant 0 makes an explosion (0x27DD8) [R 0x5C0CA..0x5C2C0]. Morgan only throws 2, 3, 4. Measured path: rises 24 px, peaks at 72 px, then flies **6.5 px per frame** toward the hero while falling [M m5-a: A#19 x 4640 to 4198 over 100 frames, y 48, 72, ..., 35].
- **Class 26** (0x4D48A): the grenade (anim 0 spins, 16 frames); attacks 0x44 (base 15, reaction 0xA = burn knock down), 0x4E and 0x67 (burn); made by Morgan (state 0x1E), by Tyrog class 4 and 91 (0x37574, 0x375D2, 0x65FA2, 0x66000) and others; at the end it makes the explosion effects (pool E class 0 and C objects). Details belong to D3f.
- **Class 59** bullet var 0 (Uzi): 7 px per frame, attack 0x0B (base 15, knock down) [R, M m5-a]. Whether a hero hit removes a bullet is not tested (open item).

### 2.5 Morgan dies and becomes Morgue

[M m5-trans, m5-b; code 0x52D7A, 0x52DA2..0x52EEA]

1. HP 0 in the hit handler: `G.$4D6 = 3` (slow motion step 3) and `G.$50E = 2` (kill all helpers, no score). The knock down reaction (state `+6 = 6`, `+7` = 2, 4, 6) runs for about 200 frames.
2. `+4 = 4`, `+5 = 0` (t = 2380): `G.$50E = 2`, `G.$4D6 = 1` (normal speed), `G.$502 = 0`; draws the **caption window** (portrait of Morgan, 0x84E2E, text line 0x3B at 0x90A29C): "Aaargh! Now I'm really gonna get bent out of shape!" (paraphrase); `+80 = 0x78`.
3. `+5 = 2` (t = 2382): 120 frames of talk; `G.$50E = 0`.
4. `+5 = 4` (t = 2502): he **walks to the centre** of the screen (speed id 5, anim 0xE, 60 frames, direction toward camX + 0xC0: x 4561 to 4649) [M]. (If he already stands within 0x40 to 0x140 of 0x11C0 this step is skipped.)
5. `+5 = 0xA` (t = 2562): pose anim 0x10 (4 frames, 40 ticks) for 120 frames, and the **Uzi item appears on the floor** (class 67 is freed and creates the item).
6. t = 2682: the caption window is cleared (45 x 13 words of 0x20 written at 0x90A29C), the class word becomes 0xA0 (class 40) and `+4..+7` are cleared: **Morgue's init** runs in the same slot. No score for Morgan.

### 2.6 Morgue (class 40)

| Item | Value |
| --- | --- |
| Name, `+84`, anim table | 0x16 MORGUE, 0x58, table 0x6EA48 (30 animations), boxes 0x103C00; the art is a green raptor like dino man, shown first in a flashing palette cycle [R, M m5-trans snapshots] |
| HP | 512 / 576 / 768 at rank 0 (boss index 5): the supplement's "same HP as the Butcher" is true for one and two players only (Butcher 512 / 576 / 640) [R D2c 16.3] |
| Palette | `+23 = 0x9D` after init; init rewrites **sprite palette 0x1D** (16 words at 0x9143A0 from 0xB7D32) with Morgue's colours [R 0x53EE0..0x53F0C] |
| Intro | state `+6 = 0xE`: anim 11 (44 frames, 136 ticks) with `+23` cycling through **0x9E, 0x9D, 0x9F, 0x6E, 0xAC, 0x00** every 2 frames for 136 frames (t = 2684 to 2820) [M m5-b; R 0x5489E] |
| Arena clamp | `camX - 0x50 .. camX + 0x1D0`; leaves via the same boundaries; arena centre X 0x11C0 [R 0x53F36..0x53F74] |
| Phase timers | aggressive 120 x4, 150 x4, 180 x4, 210 x4 frames (table 0x5418A), defensive 60 x4, 70 x4, 80 x4, 90 x4 (table 0x541AA) [R] |
| Roar and charge trigger | on top of the action table: every frame while aggressive and free (`+A1 = 0`, `+A2 = 0`), a roll `(rand & 31) <= 2` (3 in 32) when HP is at least a quarter of the start HP, `<= 1` (2 in 32) below a quarter, and the hero more than 0x100 px away in X: the **roar** 0x5511A (the same as action 6 below) [R 0x542D2..0x5432A] |
| Helpers | controller created at init (0x53EDA); Ep5 list in 1.6 |

Moves (start routine, attack id, damage base / reaction, anim and reach from `anim2.py 6ea48 103c00`; boxes are x from the body, mirrored when facing right):

| Move | Start | Attack | Anim and box | Notes |
| --- | --- | --- | --- | --- |
| **headbutt** | 0x5521C (anim 4, state 0x12), 0x55360, 0x55478 (anim 0x1B), 0x55512 (anim 0x19) | **0x40** (base 17, reaction 4, stop 16) | anim 4 / 25: box 114 active ticks 4 to 8 / 2 to 6, reach 22 to 107 px, height 42 to 74 | the long reach headbutt |
| **slap** | 0x5525A (anim 0x1B, state 0x30), 0x553EC, 0x552D2 | **0x6C** (base 17, poise 4, reaction 2, stop 16) | anim 27: box 116 ticks 8 to 20, reach 46 to 88, height 49 to 74 | the "close slaps", 102 in the idle test, 12 to 33 frames each, state 0x30 then 0x32 |
| back strike | 0x54250 (from 0x541F4) | **0x4B** (base 15, reaction 4) | | fires when the target stands behind him in a down state, same depth (+-10) |
| **charge** | 0x555CE (anim 7, state 0x20, `+AC` = 30, speed id 0x24 = **7 px per frame**) | **0x41** (base 20, reaction 4, stop 5) | anim 7: 6 frames, 42 ticks, box 115 on every 7th tick, reach 43 to 78, height 37 to 65 | the "roaring headbutt rush": runs 210 px in 30 frames |
| shoulder rush | 0x54A10 (state 0x1E), 0x54A7A (0x2C), 0x54AE4 (0x24) | **0x42** (base 20, reaction 4, stop 5) | anim 0 or 0x12 / 0x1A while moving by physics, attack set in the last 16 frames | |
| swing | 0x55550 (anim 8) | **0x2B** (base 12, poise 8, reaction 2) | | |
| flurry | 0x55588 (anim 12) | **0x31** (base 20, reaction 4) | anim 12: 10 frames 50 ticks, box 116 at ticks 5, 15, 25, 35, 45 | five swings |
| lunge | 0x5442A, 0x54494 | **0x4A** (base 17, reaction 4) | | after the first decision at spawn (`+AE` flag) |
| **roar, then charge** | 0x5511A | no attack in the roar: state 0x26 for 40 frames (`+AC = 0x28`), anim 0x12, a class 75 var 0x13 spark every 8 frames (5 sparks); at the end 0x555C0 starts the **charge 0x41** (speed id 0x24, 30 frames) | | only when the hero is more than 0x80 px away (nearer: the decision runs again) [R 0x5511A..0x55146] |

Decision (0x55704): same frame as Morgan: `+A0 = 1` rolls the gate `G[aggression][X class]` from table 0x73D54 (row 0 = `16 18 16 18 10 2 4 10`, plus 2 per row) and then the **action table 0x73D94** (32 rows of 8 classes): X class 0 and 2: action 7 (24 of 32) or 5 (8), class 1: 7 or 8 (16 each), class 3: 1 or 5, class 4: 5 or 9, class 5: **6** (32 of 32), class 6: 5 or 6, class 7: 5 (24) or 6 (8). Jump table 0x557D0: 0 and 1 = 0x55360 (headbutt), 2 = 0x55600, 3 = 0x55550 (swing), 4 = 0x55588 (flurry), 5 = 0x54C6A (run in: state 0x1C, then attack), 6 = 0x5511A (roar, then charge), 7 = 0x553E0 (slap), 8 = 0x5546C, 9 = 0x55462, 10 = 0x553D6. Defensive phase: approach / kite as Morgan [R 0x55704..0x557E4].

Measured damage (hero HP 100, level 10, Jack) [M m5-morgue-dmg]: slap 0x6C = 20, back strike 0x4B = 17, shoulder rush 0x42 = 23; all three equal the formula (1.4). The charge 0x41 (base 20), headbutt 0x40 (base 17) and flurry 0x31 were not measured; use the formula [R].

### 2.7 Morgue dies; what follows

[M m5-morgue-kill; R 0x557F4..0x55866]

- Hit handler: tally, HP 0: `G.$4D6 = 3`, `G.$50E = 2`.
- Reaction (knock down in an arc, 240 frames), `G.sub` 2 at t = 3927, `+4 = 4` at t = 4295 (**`0xACEE` is called at the first dying frame 0x55806**, then `0xA5EA` music fade, `+80 = 0x78`, kill score 30,000 (`0x325E6`), `0x6148` marks the last-blow player), freed 120 frames later (t = 4416) [M].
- Then the normal boss end (1.5): `G.sub` 4 at t = 4295, 6 at t = 6096 (tally), the next scene (Episode 6 story panel) at t = 7054 [M m5-morgue-kill]. The panel text belongs to D4d.

### 2.8 Checks of what the user and the web said

| Claim | Verdict | Evidence |
| --- | --- | --- |
| G10: the old villager starts to explain, Morgan shoots him | **True** | 2.1 (four bullets at t = 406 to 418, villager down) |
| "Morgan hops, throws knives, rocks and grenades, fires an Uzi" (supplement, blueprint) | **True**: hop (states 0x28, 0x2A), knife / rock / stone throws (class 68 var 2, 3, 4), close grenades (class 26), Uzi burst (5 shots) | 2.3, 2.4 |
| "gives 0 points" | **True** | 2.2 |
| "drops the Uzi when beaten" | **True** (pool E Uzi, var 6) | 2.5 |
| "Morgue has the Butcher's HP" | true for 1P and 2P, not 3P (768 against 640) | 2.6 |
| "roaring headbutt rush and close slaps" | **True**: charge 0x41 at 7 px per frame, slap 0x6C | 2.6 |
| "Jack's slide passes under Morgue's charge" | **True by the numbers**: every Morgue attack box starts at height 37 or more (114: 42, 115: 37, 116: 49) and Jack's slide body box is 35 px high (D2a section 7); not run in MAME | R |
| "beats even Mustapha's drop kick in a trade" | not checked (trade rules of the hit passes, D2b) | |
| "Uzi shots knock Morgan off screen" | **True** (108 px slide, down 100 frames) | M m5-uzi |
| Fessenden's first form "uses Morgue's pattern with more headbutt dashes and no helpers" | true for the code (class 94 is a clone) and no helpers; "more headbutt dashes" is the different action table 0x74A82 | section 6 |

## 3. Episode 6: Tyrog (starter pool B 79, scene class 75 and 76, boss class 4, host class 72, Tyrog 2 class 91)

### 3.1 What exists

| Part | Class and address | Role |
| --- | --- | --- |
| Starter | pool B **79** 0x8A3D0, script record at trigger 0x0F80, X 0x1120, Z 0x190 [R stage_scripts 0xB03D6] | makes the crate scene, sound 0x21 |
| Scene actor | pool A **75** 0x5E702 (art table 0x71A4E, 20 variants) | crate (var 0), guards (var 1, 4), sparks (var 3, 6, 7, 9), flash puffs (var 5, 0x12) and landing marks (var 8, 0x10, 0x11) |
| Creature | pool A **76** 0x5EC26 (art table 0x71D66) | the Tyrog body that jumps out of the crate, then makes class 4 var 0x200 |
| Boss | pool A **4** 0x36A64, name 0x19 TYROG, art table 0x702C4 (39 animations), boxes 0x103C00 | three forms by the high byte of `+26`: var 2 green, var 1 orange, var 0 grey and blue |
| Host | pool A **72** 0x5DD58, name 0x1B (HUD name also TYROG), HP index 0 | the man whose body is taken over between two forms; same art table 0x702C4 |
| Chest blades | pool A **74** 0x5E4F4 (art table 0x71936), attached to the Tyrog that makes it | the "chest blades" |
| Fire | pool A **26** var 3 (fireball) and var 4 (lava rock), 0x4D48A | shared with Morgan's grenades (var 0), details of the class in D3f |
| Tyrog 2 | pool A **91** 0x654E2, name 0x23 TYROG 2 | Episode 8 enemy (section 5 and 3.8) |

The chain is made by rewriting the class word of one slot in place (the object keeps X, Y, Z, facing and slot): **class 4 var 2 → 72 var 1 → 4 var 1 → 72 var 0 → 4 var 0 → end** [R 0x38DAC, 0x5E076; M ty-full-c, ty-chain].

### 3.2 The scene before the fight (M ty-full-a, t counted from the hero walking in at camera 0xF00)

| t | What happens |
| --- | --- |
| about 96 | the starter makes 4 class 75 actors: **var 0** at X 0x1120 (a steel crate), **var 1** at X -0x30, **var 4** at X +0x30 (Z -0x20) and **var 4** at X -0x10 (Z +0x20), plus the effect 0x86D2E with code 0x50 = caption window "YOU CAN'T GO ANY FURTHER!" for 121 frames, then codes 0x51 (221 frames), 0x52 and 0x53 (121 frames each; not seen as captions in M, U) [R 0x8A412..0x8A57C; M snapshot t = 200] |
| 96 to 416 | four hunters (blue caps) stand round the crate; the crate counts `+B2 = 0x140` = **320 frames** [R 0x5E7A0] |
| from 316 | the crate shakes (X +-4 every 4 frames) and throws sparks (class 75 var 7) every 4 frames, at `+B2 = 99` one flash (var 9) [R 0x5E9B0..0x5EA3A; M] |
| 416 | the crate breaks: class 75 var 0 dies (st 0404) and makes **class 76** at the same place [R 0x5EA40; M] |
| 416 to 486 | class 76 jumps out (vy 0x600 up, gravity 0x40, so about 48 frames in the air, vx 1 px per frame to the left), makes a class 75 var 6 puff every 8 frames; at landing anim 1 and sound 0x82 (params 0x62020, 0x540080), then anim 2 with sounds 0x83 and 0x60 [R 0x5EC94..0x5ED9A] |
| 596 | the hunters (var 4, `+B2 = 0x1F4` = 500 frames after creation) are replaced by live thugs: a class 2 var 0x200 each (the var 1 hunter just disappears after 370 frames) [R 0x5EAC6..0x5EB0C; M sheet t = 600: three thugs fight the hero] |
| **648** | class 76 makes **class 4 var 0x200** at its X, Z (fight 1 starts; HP 256 at level 10) [R 0x5EDC2; M] |

Class 4 init for every form (0x36A7A): `+4 = 2`, `+84 = 0x64` (name 0x19), anim table 0x702C4, `+63 = 2`, speed id 7 (2.0 px per frame), `+22 = 0x18`, HP = index 6 of 0x31E14 **halved** (so every form starts with half of the table value), `+78 = 0xB` poise, `+72 = 0x1E`, `+A0 = 1` (aggressive), `+A6 = 1`, palette `+23` = 0x9D (var 2), 0x9E (var 1), 0x9F (var 0), `+7D` = 0xFE for var 2 and var 1, 0xFF for var 0. **Only var 2** plays music **0x0F** (0x36B6C) and makes the reinforcement controller (0x7BEDC, lists in 1.6). The HUD pointer `-$70BC(a5)` is set by every form [R 0x36A7A..0x36B96].

The arena clamp while alive is `camX - 0x10 .. camX + 0x190` (class 72: `camX - 0x20 .. camX + 0x1B0`) [R 0x36BD4, 0x5DE8C].

### 3.3 HP, score, flags

| Form | Class and var | HP (1P / 2P / 3P at level 8 to 11, band 2) | Kill score | `+7D` | Music | Colour |
| --- | --- | --- | --- | --- | --- | --- |
| Fight 1 | 4 var 2 | **256 / 288 / 352** | none | 0xFE | 0x0F | green (0x9D) |
| Host 1 | 72 var 1 | 450 / 600 / 650 | 100,000 and `0xACEE` if it is killed | 0xFE | none | |
| Fight 2 | 4 var 1 | 256 / 288 / 352 | none | 0xFE | none | orange (0x9E) |
| Host 2 | 72 var 0 | 450 / 600 / 650 | as above | 0xFE | none | |
| Fight 3 | 4 var 0 | 256 / 288 / 352 | **60,000** (call 0x325E6 at 0x38D56) and `0xACEE` | 0xFF | none | grey and blue (0x9F) |

Table by level band (`level >> 2`, bands 0 to 7), HP after the halving [R 0x31E14 index 6]: 1P 256, 256, 256, 288, 288, 288, 320, 320; 2P 288, 288, 288, 320, 320, 320, 352, 352; 3P 352, 352, 352, 384, 384, 384, 400, 400. The host uses index 0: 1P 350, 400, 450, 500, 500, 550, 550, 600; 2P 500, 550, 600, 600, 650, 650, 700, 700; 3P 600, 600, 650, 700, 700, 750, 750, 750 (same row as Slice and the Slisaurs). Measured at level 10: 256, 450, 256 [M ty-full-c]. The **HP is refilled at each change** (the supplement is right): the total to remove is 3 x 256 = 768 at 1P, 3 x 288 at 2P, 3 x 352 at 3P; the host's 450 is optional.

### 3.4 The AI of class 4 (0x36A64)

**Frame** (0x36C34 and 0x36CA0): poise refill `+79`; hit stop `+59`; then the state routine of `+6`; then 0x36DF0 (aggressive request and gate) and 0x36CF4 (phase flag); `0x32F56`, `0x49CA` (draw), `0x4BEA` (hurtable). Sub-routine `+5`: 0 AI, 2 hit (0x38CC6), 4 dying (0x38D0E), 6 grabbed (0x2B35A) [R].

**Phase flag `+A0`** (0x36CF4): `+A6` counts down; at 0 the flag flips and the target is re-picked (0x32032). After a flip to aggressive the next flip comes after `T1[rand & 0x1E]` and after a flip to defensive after `T2[...]`: **T1 = 200, 220, 220, 240, 240, 260, 260, 260, 280, 280, 280, 300, 300, 320, 320, 360** frames and **T2 = 90 x4, 120 x8, 150 x4** (the same tables as Morgan, 1.2). Fight starts aggressive [R 0x36D36, 0x36D56; M ty-full-c `+0xA0.b`].

**Gate** (0x36DF0 then 0x38BD6): every frame of the aggressive phase `G.$502 = 1` (asks the controller for a thug group) and the gate rolls `rand & 31 < G[aggression][X class]`. The table is the same for all three forms: 8 rows by aggression `G.$5EC` (0 to 7) and 8 columns by the X class of 0x329B8: row 0 = 16 14 16 18 20 18 16 14, each row 2 higher, row 6 = 28 26 28 30 32 30 28 26, row 7 = 30 28 30 32 32 32 30 28 [R 0x74542, 0x74582, 0x745C2]. `G.$5EC` is set by the script: it is forced to 7 from camera 0xAE0 and **re-rolled when the camera reaches 0xF00** (records `FORCE-AGGRESSION` and `REROLL-AGGRESSION` of area 2 [R stage_scripts 0xAD542, 0xAD55E]). A success sets `+A4 = 1` (an attack is pending); the pending attack is taken at the next decision.

**Decision** (0x38B90 then 0x38C36), run at the end of every move:
- If `+A4 = 1`: the **action** is read from `table[form][rand & 31][X class]` (32 rows by 8 columns, X class from 0x3293C, range classes of 24 px as in D3a 3.4) and the action routine of the jump table 0x38CA6 starts.
- If `+A4 = 0`: speed id 5, a short **crouch** (anim 20, 6 frames) and then, **aggressive**: a **recoil leap** (state 0x2C then 0x10: vy 0x380, 0x300 or 0x280, attack 0x4D, anim 0x15, 18 to 22 frames), **defensive**: a **sidestep hop** (state 0x2E then 0x04: vy 0x400, 22 frames, sideways 3 px per frame or none, depth 3 px per frame or none, tables 0x74BE2, 0x74C62, 0x74CA2). The third sidestep in a row (`+AB = 3`) is followed by a **rest** of 40 (4 of 16), 60 (8 of 16) or 80 (4 of 16) frames (state 0x0A, table 0x37C20). After each landing a wait of 5, 10 or 15 frames (state 0x3E, table 0x3868C) comes before the next decision [R 0x38B9A..0x38BD6, 0x37A10, 0x37BEE; M ty4-v0-dmg, ty-full-c state chains].

**Actions** (jump table 0x38CA6 gives 8 entries; the ROM tables use only 2, 3, 4, 6 and 7):

| Action | Start | What it does | Used by |
| --- | --- | --- | --- |
| 0, 1 | 0x381C6 | one claw swipe: anim 3 with attack 0x4F, then a wait of 5 (6 of 16), 10 (8) or 15 (2) frames | never picked by the tables (U) |
| **2** | 0x385F2 | **pounce**: crouch (anim 20, state 0x32, 6 frames), then a **high leap** at the hero (state 0x34: vy 0x800, vx at least 0x80, depth step, anim 2, attack 0x4D), 54 to 59 frames | fight 3 far classes (6, 7: 8 of 32) |
| **3** | 0x389D0 | **hop and fireball**: a hop (vy 0x380, state 0x3C) to a spot 0x80 px beside the hero, repeated `+CC = 2`; on landing the **fire breath**: anim 7 (38 ticks, state 0x1E), one **class 26 var 3** at X -0x24 or +0x24, Y +0x30 | fight 2 (far), fight 3 (mid) |
| **4** | 0x37F46 | **lava spit**: crouch (anim 0x17, state 0x36), then at the anim end one **class 26 var 4** (a rock thrown in an arc at a random spot round the hero), **8 times** in a row (`+AA = 8`, the facing direction `+22` of each rock = (count - 1) x 4) | fight 3 (mid and far) |
| 5 | 0x388F8 | hop (0x90 px), then the chest blades twice (state 0x2A, 0x37C50) | never picked (U) |
| **6** | 0x38AA8 | **hop and chest blades**: a hop (vy 0x380, state 0x44) 0x80 px beside the hero, `+CC = 3`; on landing the blades: crouch (anim 0x1C, state 0x50), blade B (anim 27, state 0x46, class 74 var 0x100), blade A (anim 6, state 0x24, class 74 var 0), and again until `+CC` is used up | fight 2 (close, fight 1 far), fight 3 |
| **7** | 0x38234 | **claw chain**: `+CC = 4` swipes, each anim 3 (19 ticks, box active ticks 6 to 15) with attack **0x68** (light), the last one with **0x4F** (heavy), waits of 5 to 15 frames and a random short wait of 1 to 8 frames between swipes (table 0x74EE2 plus `0x74F22[level]`: 8 at level 0 to 3 down to 1 at level 28 to 31); then the crouch of the "no pending attack" case (a leap away or a sidestep) | close range in all forms |

**Action table histogram** (counts of 32 rows by X class) [R 0x74602, 0x74702, 0x74802]:

| X class | Fight 1 (var 2, table 0x74802) | Fight 2 (var 1, 0x74702) | Fight 3 (var 0, 0x74602) |
| --- | --- | --- | --- |
| 0 | 7 x32 | 7 x32 | 6 x8, 7 x24 |
| 1 | 6 x8, 7 x24 | 3 x8, 7 x24 | 6 x16, 7 x16 |
| 2 | 6 x16, 7 x16 | 3 x16, 7 x16 | 6 x24, 7 x8 |
| 3, 4, 5, 6, 7 | 6 x32 | 3 x32 | class 3: 3 x24, 4 x8; class 4: 3 x16, 4 x16; class 5: 3 x8, 4 x24; classes 6 and 7: 2 x8, 3 x8, 4 x16 |

So **fight 1 = claws and chest blades**, **fight 2 = claws and fireballs**, **fight 3 = everything: claws, blades (close), fireballs (mid), lava spit (far), the pounce (rare)**. The web notes "chest blades, fireballs, lava spit" are confirmed and the order in which they appear is the one of the table.

**After a hit** (recover entry 0x36E7C, from the table 0x2B56C): palette back to normal, `+4 = 2`; then
1. if the hero stands within 0x50 px in X and 8 px in Z and the last reaction was a knock down (`+7B = 4`): the **wake-up claw** (0x382B0: anim 0x18 = 24, attack 0x4F, state 0x40 then 0x0E);
2. else if the boss is on screen (0 <= X - camX < 0x180) go on to 4;
3. else with 10 in 16 (and `+7B = 4`) a **pounce** (0x3862A: attack 0x4D, anim 0x26);
4. else a recoil leap start (state 0x4C, then state 0x10).
This recover path is also what the **first frames of each form** do (`4C(1) 10(22) 3E`) [R 0x36E7C..0x36F68; M ty-full-c].

Measured durations (frames, one player, level 10) [M ty4-v0-dmg, ty4-v1-dmg, ty4-v2-dmg, ty-full-c]: leap 0x10 18 to 22; sidestep 0x04 22; crouch 0x2C, 0x2E, 0x30, 0x32 6; wait 0x3E 4 to 16; hop wait 0x3C 28 to 84, 0x44 22 to 28; fire breath 0x1E 38; blade cycle 0x44 (28), 0x50 (16), 0x46 (32 to 53), 0x24 (31 to 37), repeated 3 to 5 times in a row; rest 0x0A 40, 60 or 80; lava state 0x36 64; pounce 0x34 54 to 59. The sequences are the same on every replay of the same input.

### 3.5 The attacks, boxes and damage

Attack records [R 0xA2EA4, 8 bytes per id: base, poise, reaction, hit stop, sound]:

| Id | Base | Poise | Reaction | Stop | Used by | Dmg at level 10 (TYROG row 42, Jack, hero HP 100) |
| --- | --- | --- | --- | --- | --- | --- |
| 0x4F | 18 | 11 | 4 knock down | 16 | claw swipe finisher, wake-up claw, the body during blade states | formula 23 |
| 0x68 | 10 | 1 | 0 | 12 | light claw swipes | formula 13 |
| 0x4D | 20 | 11 | 4 | 5 | leaps, recoil leap, pounces | **26** (formula 26) [M ty4-v0-dmg, state 0x34, t = 1403] |
| 0x50 | 24 | 11 | 4 | 16 | class 74 var 0 (long blade A) | 27 measured in ty4-v2-dmg (formula 31; see HU08) |
| 0x6B | 15 | 4 | 2 flinch | 16 | class 74 var 1 (short blade B) | 15, 16 and 18 measured (formula 19; HU08) |
| 0x67 | 20 | 11 | 10 burn knock down | 5 | class 26 var 3 (fireball), moves **8 px per frame to the left or 6.5 px per frame to the right** (by the owner's facing), anim 0xC | 22 measured [M ty4-v1-dmg] |
| 0x4E | 15 | 11 | 4 | 5 | class 26 var 4 (lava rock), palette 0x81, anim 0xA, vy 0x400 | not measured (formula 19) |

The blade and fire objects never set `+84`, so their damage reads a stale row (the HU08 effect); the numbers above are what the probe saw on a fresh slot, not a design value [R, M]. The port should use the row of the owner (one deliberate departure to record in D4: the original value is history dependent).

Hit boxes of the body (box table 0x103C00, entries x offset, width, y offset, height in front of the body, mirrored when facing right) [R anim2 listing]: anim 3 and anim 24 (claw): box 143 = X 27 to 122 px in front, Y 51 to 78, active ticks 6 to 15 of 19 (21 for anim 24); anim 25 (a 1-frame claw after a fall): box 142 = X -11 to 59, Y 27 to 46, ticks 0 to 10. **The leaps, pounces and the crouch have no attack box in the art** (mode 0): their damage comes from the object hit test while the hit stop `+59` is 0, with the body box (127 to 138: 50 to 67 px wide, 63 to 94 px high) [R; the exact hit rule of leaps is the common one in D3a, U for Tyrog only].

**Chest blades (class 74, 0x5E4F4)**: made by the owner in state 0x24 (var 0, anim 6) and 0x46 (var 0x100, anim 27) at the frame where the owner's `+2D` is 1 (tick 33 of anim 6, tick 13 of anim 27) [R 0x37444, 0x374B4]. The child follows its owner (`+A0` = owner slot), keeps `+63 = 0xFF`, `+7D = 0xFF` and **20 HP** (`+6A, +6C, +6E = 0x14`), is registered as hurtable (0x4BEA) so a hit on it gives hit stop `+59` to the owner, and ends when the owner leaves the blade frames (`+2D = 0`). Its anim 0 (var 0) has 19 ticks and a **single hit tick (tick 9)** with box 163 = X -116 to +64 (a 180 px wide sweep through the owner), Y -16 to +5 (a **low** box); anim 1 (var 1) has 8 ticks, hit tick 2, box 158 = X -40 to +64, same height. While it runs it drops a class 75 var 3 spark every time the owner's `+2D` changes, at X offsets -0x30, -0x20, -0x10, 0, 0x10, 0x30, 0x50, 0x70, 0x90, 0xA0, 0xC0 (table 0x5E6D4) [R 0x5E65A..0x5E6D4]. The low box suggests a jump clears the blade (U, not run).

**Fireball (class 26 var 3)**: made at X -0x24 (facing right: +0x24) and Y +0x30 of the owner, starts with no vertical speed and falls with `+1C = -0x38` (about 20 frames to the ground) while it moves 8 px per frame left or 6.5 px per frame right; on the ground it burns for the length of its anim 0xD (box table 0x103000) and then is freed [R 0x4D728..0x4D770, 0x4D7E0..0x4D844]. **Lava rock (var 4)**: its own palette 0x81, vy 0x400, `+1C = -0x40`, vx = distance / 0x3800 steps (each rock picks its own landing spot at its creation: the hero's position at that moment plus an offset from the tables 0x4D49E (X) and 0x4D51E (Z) indexed by `rand & 31`: **centre 16 in 32, X +48 4 in 32, X -48 4 in 32, Z +32 4 in 32, Z -32 4 in 32**, a cross of five spots, so the volley of 8 rocks covers a plus shape round the hero) [R 0x4D59E..0x4D794].

### 3.6 Hit reaction, grab, immunities

- Poise `+78 = 11`, refilled by `+79` frames after a hit (as Vice, D3c 3.x). Hit reaction is the common routine 0x2A606; recover entry 0x36E7C. No block, no armour: the harness hit it with every hero move that was tried (M ty-full).
- **Grab and throw** work on all three forms and on Tyrog 2 at any time it is not in the air or dying: the hero grabs at t = 34 and throws at t = 35 (sub 6 = 0x2B35A), the boss flies 48 frames, loses **27 HP** (256 to 229), lies 46 frames and recovers through 0x36E7C (st 0x4C at t = 161) [M grab-ty4v0, grab-ty4v1, grab-ty4v2: identical]. The host (class 72) was **not** grabbed in one test (it ran its punch and jump, U) [M grab-ty72]. The supplement's "grabbable between phases" is half checked (the forms yes, the host no).
- Poise: forms 11 (as Vice), host 1 (every hit knocks the host down; M `ty-full-c`: one hit of 14 delayed its change by 124 frames). Whether a long chain locks a Tyrog form as it does Vice was **not** tested (U). The supplement's "not combo-able" is therefore open.
- Immunities: none seen. The host is **not hittable for its first 30 frames** (state 0, flag `+C8 = 1`, the frame does not call 0x4BEA) [R 0x5DE52..0x5DF94].

### 3.7 Death, host and chain, step by step

All times are frames after the class-4 HP reached 0, measured in `ty-full-c` (the same in a to e) [M]:

1. **Hit handler 0x38CC6**: tally (0x3262A). At HP 0: for var 2 and 1 `G.$4D6 = 3` and `G.$50E = 2` (slow motion step 3, helpers die); 0x32740 starts the fall; for var 0 `0xACEE` is called here (once, `+6E` is cleared). The fall is the common knock down.
2. **Dying 0x38D0E** (`+4 = 4`) for var 2 and 1: `G.$4D6 = 1`, `G.$50E = 0`, `+80 = 0x78`, `+5 = 4`; the corpse lies **120 frames** (drawn every frame, no blink); then the slot is rewritten: `+4..+7 = 0`, class word **0x120 (class 72)**, var - 1 (2 to 1, 1 to 0). On **1 in 16** form changes the routine 0x5A454 is called; it sums the ROM bytes 0x30 to 0x419 against 0x137E8 and, only if the sum is wrong, adds 1 to `G.$757A` and calls 0x97D78. It is a ROM checksum test and does nothing on a good ROM: **skip it in the port** [R 0x38DB8..0x38DD0, 0x5A454].
3. **Host (class 72)**, 250 frames when it is left alone (M: t = 1630 to 1878 for host 1): init 0x5DD6E: name 0x1B, `+84 = 0x6C`, HP index 0, `+78 = 1` (poise 1: **every hit knocks it down**), `+C8 = 1`, anim 0x25 (var 1; var 0 uses 0x24), 30 frames of intro with palette flicker 0x9E / 0x9D (var 1) or 0x9F (var 0) toggling with the saved colour. At the end of the intro it makes a class 75 var 5 (var 0x12 for var 0) puff and starts: **punch** (state 4, 22 frames, anim 9 or 0x20, attack **0x3F**, base 12), **jump** (state 6, 58 frames, anim 10, attack **0x34**, base 15, vy 0x700, to the screen edge on its own side: X camX + 0x30 when it stands in the left half of the screen, camX + 0x150 otherwise, Z camZ + 0x50; it makes a class 75 landing mark (var 8, 0x10 or 0x11) and takes the mark's X and Y at the landing), **landing** (0x5E2F8: sound 0x79, effect 0x86D2E code 0x5A = caption "SOMEONE HELP MMMPHH.. GURGLE!"), and the **transformation** (state 8: anim 0x22 for var 1 and 0x23 for var 0, 41 frames, 140 ticks, palette flicker) [R 0x5DD6E..0x5E3B8; M ty-full-a snapshots t = 1700 to 1860 and 3400]. At the end of the anim the slot becomes class 4 again (class word 0x10, same var) and runs init: HP is **256 again**. In the snapshots the host is a fat man in a green vest and an orange jellyfish creature hovers over him; the picture at the end is the orange Tyrog (host 1) or the grey and blue one (host 2).
4. The host has **450 HP** and its hit handler 0x5E448 and dying routine 0x5E470 are those of a boss: if it is killed (450 damage inside the 250 frames; every hit knocks it down and the fall adds about 120 frames, so this needs very strong hits, U) it calls `0xACEE` (stage end), gives the **100,000** points (0x5E48C) and `0x6148` marks the player who killed it. This is the web's "finish with 100,000" and it is the only way to skip fights 2 and 3 [R 0x5E448..0x5E4BA; U not run in MAME, the host only took 14 damage in `ty-full-c`].
5. **Fight 3 dying** (var 0): `0xA5EA` (music fade), `G.$4D6 = 1`, `+80 = 0x78`, 60,000 points, 120 frames, `0x6148`, `+4 = 6` (free). Boss end timeline (M ty-full-c): sub 2 at t = 4909, sub 4 at t = 5229 (**320 frames** later, not 368), sub 6 at t = 7030, next mode at t = 7988; then the tally and the Episode 7 story panel (D4d).

Between the death of a form and the start of the next one, the thugs of the controller keep coming (1.6). `G.$50E = 2` at the first hit kills the thugs already on the screen (no score, as for every boss).

### 3.8 Tyrog 2 (class 91, 0x654E2)

A 94 percent copy of class 4 (diff in `re/out/d3d/diff_4_91.txt`): name 0x23 TYROG 2 (`+84 = 0x8C`), palette 0x9D (green), HP from the **normal enemy table** (0x31424, name 0x23: **300 at every level and every player count**), `+7D = 0` (it is **counted** by the wave census, unlike the forms of class 4), `+63 = 0`, no music, no controller, hit handler 0x2A606 (the common one) and a plain death, no `0xACEE`, no forced aggression. Its gates and actions are the **fight 3 tables** (var 0: it is created with var 0 by the tank) and the move set is the same (claws, blades, fireballs, lava rocks), but with different sidestep tables (0x74CC2, 0x74D82). The byte `+A2` is loaded from a level table (0x74B82 for class 4: 31 down to 0 by level; 0x74BA2 for class 91: 40, 35, 30, 25, 20, 15, 10, 5 in steps of 3 levels) and no use of it is visible in the listings (U). Kill score **1,000** [R, M ty91-dmg]. It never absorbs a host and has no form change (the supplement is right). Where it comes from: the Episode 8 tank 11 (section 5). The grab test was not conclusive (two objects were in the log, U).

### 3.9 Sounds and palettes (for D4e)

Sound ids written by the code [R]: starter 0x21; class 76 landing 0x82 then 0x83 and 0x60; class 4 attack starts 0x82 (params 0x5011F, or 0x6201F when the boss is at least 0xC0 px right of 0x1000, and 0x540080: a position dependent pan); music 0x0F (fight 1); music fade 0xA5EA at the end of fight 3; host landing 0x79. Palettes: forms 0x9D (green), 0x9E (orange), 0x9F (grey and blue); blades use the owner's palette; lava rocks 0x81. The class 4 hit flash uses `+23` back to the form palette in the recover entry.

### 3.10 Checks of the web and user notes for Tyrog

| Note | Verdict | Evidence |
| --- | --- | --- |
| "Tyrog small head enters a fat enemy, then Walther" (supplement) | **Half true**: a jellyfish creature takes a fat man (host 1) and a second host (host 2) before the next form; the second host is not Walther in M (not identified, U) | 3.7 |
| "HP refills at each change" | **True** (256 each form) | 3.3 |
| "one hit chance in the small-head phase" | **No such rule**: the host has 450 HP and can be hit after 30 frames; it only lasts 250 frames | 3.7 |
| "co-op multiplies jellyfish HP" | **True in a form**: the host HP is 450 / 600 / 650 by player count at level 10 and the forms 256 / 288 / 352 | 3.3 |
| "Tyrog 2 never absorbs" | **True** | 3.8 |
| "chest blades, fireballs, lava spit" | **True** (class 74, class 26 var 3 and 4) | 3.4, 3.5 |
| "grabbable between phases but not combo-able" | **Half**: the forms are grabbable (27 damage), the host not in one test; combo lock not tested | 3.6 |
| "60,000 for Tyrog, 1,000 for Tyrog 2" | **True** (fight 3 only; forms 2 and 1 give nothing; the host 100,000) | 3.3 |

## 4. Episode 7: the Slisaurs (pool B starter 33, pool A class 71, blade class 19, afterimage class C27)

### 4.1 What really exists (the starter is class 33, not 41)

| Part | Class and address | What it is |
| --- | --- | --- |
| Starter | pool B **33**, handler 0x818E4 (109 instructions), script record `L2 0xB0DE2` trigger 0x3C0 of Episode 7 area 3 | creates the Slisaurs, sums their HP, ends the fight [R] |
| Slisaur | pool A **71**, handler 0x5C818 (5,440 bytes), name id 0x18 "SLISAUR", kill score 70,000 each | the boss body [R: classmap, 0x325E6 call at 0x5D19E] |
| Hit handler | the `+5 = 2` slot of class 71 points into the starter's file: **0x81A7A** | damage tally, death bookkeeping, then the common reaction 0x2A606 [R] |
| Blade | pool A **19**, handler 0x45082 (1,838 bytes), created by the helper **0x4504E** (called from 0x5CEF4, 0x5D042 and, for Slice, 0x4C608, 0x4C784) | the boomerang blade, shared with Slice (D3c) [R] |
| Afterimage | pool C **27** (0x6C), handler 0x963D6, created by **0x9639E** (called from 0x5CC68, 0x5CF6A and, for Slice, 0x4C37C, 0x4C67E) | the trail copies [R] |
| Lab monitor talk | pool B 30 (machine event, sound 0x23) and 34 (caption box, three lines of the tall font) | Fessenden's face on the monitor [R D3e, M s7-1p-a] |
| Class 41 x3 (var 0, 4, 8, X 0x430) | pool B **41** (0x8383A) | **not a starter**: a lab tile animation family (D3e). The task text and `episode-7.md` call 41 the starter: wrong |

Nothing else creates class 71: a scan of the whole ROM for writes of the class word value 0x11C into the `+20` field of a new object finds only the three writes at 0x8191E, 0x81948 and 0x81982, all inside class 33 [R: `re/out/d3d/creators.py`]. The scripts have no `A#71` record. So **no Slisaur exists in Episode 3, and none in any episode but 7**. The single fast boss with afterimages and boomerangs is Slice (Episode 4, D3c): his code calls the same two helpers 0x4504E and 0x9639E [R]. The user note is answered in 4.8.

### 4.2 The starter (class 33)

- Waits while `G.$4FC` is not 0 (opening scene flag); then creates the reinforcement controller (pool B class 22, 0x7BEDC), sets `+7D = 0xFF`, plays **sound 0x11** (the boss tune of stage index 6: table 0xA5C8, code 0x11) and goes to its run state [R 0x8199E].
- Creation (state 0 sub 0, 0x8190A): two Slisaurs at **X 0x580 (world), Z 0x30 and 0x40**; if `G.$4EF != 0` (2 or 3 players) a **third at Z 0x20**. It stores their handles in `+A0`, `+A2`, `+A4`, the count in `+AA` (2 or 3) and puts its own address in each Slisaur's `+C6` [R 0x818E4..0x8199C; M s7-1p-a: `+AA = 2`; s7-2p-a, s7-3p-a: `+AA = 3`].
- **Count by players: 2 for one player, 3 for two or three players** (the web claim is right) [M s7-1p-a, s7-2p-a, s7-3p-a; W supplement, blueprint].
- Every frame (state 2, 0x819BE): `G.$7B4 = 2` (boss fight flag), sums the HP of the live Slisaurs into its own `+6C` / `+6E` (not drawn: the bar draws the Slisaur in `-$70BC(a5)`), every 300 frames sets `G.$502 = 1` (turns on the threshold list of the reinforcement controller), and keeps `-$70BC(a5)` on the first live Slisaur (0x81AF4) so the HUD name bar always shows a Slisaur [R].
- **End of the fight**: when the three handles are all 0 it calls `0xACEE` with `a6` = the last Slisaur (so the killer credit and `G.$4E0` are that body), then `0xA5EA` (music fade), then state 4 calls `0x6148` (marks the player of the last blow with `+C0 = 1`, the "FINAL BLOW" bonus of the tally) [R 0x819EC..0x81A70].
- The handle bookkeeping is done by the **Slisaur hit handler 0x81A7A**: it adds the damage to the hitter's tally (0x3262A), and at HP 0 it clears the body's handle in the starter, decrements `+AA`, and for the last one stores the body in `+A8`. So the fight ends **on the frame the last Slisaur reaches 0 HP**, not when its body is freed [R; M sl-kill: handles cleared at t = 1940 and 2027, `G.sub` 2 at t = 2027, the last body starts dying at t = 2396].

### 4.3 Data of a Slisaur (class 71)

| Item | Value |
| --- | --- |
| HP | boss table index 0: **450 each at one player (level 10), 600 at two (level 12), 700 at three (level 14)**; at rank 0 it is 350 / 550 / 650 (D2c). The class keeps the maximum in `+BE` for its phase tests. [M s7-1p-a hp 450, sl-dmg-2p hp 600, sl-dmg-3p hp 700] |
| Poise | 11 at init (`+78`), refilled by `+79` as for all enemies; reaction by the common code 0x2A606 [R] |
| `+7D`, `+63` | 0xFF (not counted by waves, immune to kill-all), 2 (big: bounces off screen edges when dead) [R] |
| Name, `+84` | 0x18, 0x60 (damage scale row SLISAUR: 32 at level 0, 42 at level 10, 79 at level 31) [R 0x2C3AE] |
| Anim table, boxes | 0x5D1C0 (13 animations), box table 0x103000 [R] |
| Speeds | walk speed id by level band: table 0x5DCEE `{0x1B, 0x1C, 0x1D, 0x0B, 0x0B, 0x0C, 0x0C, 0x0D}` indexed by `(level & 0x1C) >> 2` = 2.7, 2.8, 2.9, then 3.0, 3.0, 3.25, 3.25, 3.5 px per frame; level 10 gives 2.9 [R 0x5DCDA]. Run (state 8): speed id 0x1F = 4.0 px per frame. **Dash ramp**: table 0x5DD1C `{0x0B, 0x1F, 0x20, 0x1E, 0x21, 0x22, 0x23, 0x24, 0x24, 0x24}` = 3, 4, 4.5, 5, 5.5, 6, 6.5, 7, 7, 7 px per frame, one step every 3 frames from the start of the dash [R 0x5DCFE; M sl-fine: 8, 9, 10, 11, 12, 13 px per 2 frames at t = 260 to 282] |
| Hop (mode 2 of the walk) | `vy = 0x800` (8 px per frame up), `ay = -0x60` per frame squared, flat air time 42 frames, horizontal speed chosen so that it lands at the destination (division by 42) [R 0x5CAE8..0x5CB78] |
| Kill score | 70,000 each, awarded at the start of dying (0x5D19E) [R] |
| Palette | no override (OBJ palette of the area); trail palettes 0x9D, 0x9C, 0x9B, 0x9A, 0x8C, 0x8B (see 4.6) |

### 4.4 The AI (what a Slisaur does)

State bytes: `+6` 0 = pick, 2 = walk and wait, 4 = attack script, 6 = wait (+C1 timer, 0x78 frames); in state 4 the byte `+7` is the sub-state. Entry order after spawn: the first action is **combo 5 (dash then kick)** or combo 4 (fan) when the Slisaur starts off screen (0x5C9EE: `d1 = 4`, `bsr 0x5DCC4` on screen test, `d1 = 5` when on screen) [R; M s7-1p-a: `+7` = 0xC (dash) at t = 260, then 2, then 0xE (kick)].

**Walk state (+6 = 2)** [R 0x5CA08..0x5CC10, M]:

1. Every 600 frames (`+AA`) the target is re-picked (0x5D4A4) among the players present; each Slisaur keeps **its own target** in `+76` (and its index in `+A8`, list `+A2..+A6` built by 0x5D5A4). A target that is down (`+C8 = 0x22` or 0x23 on the hero) ends the action at once [R; M sl-dmg-2p, sl-dmg-3p: three `+76` values 0xB274, 0xB3F4, 0xB574 on three bodies].
2. When the cool-down `+B3` is 0 it rolls the **attack gate** (0x5DAE8): attack when `(rand & 31) < G[level]`, table 0x5DB0E (indexed by level 0..31): `6 10 11 11 12 12 13 14 15 15 16 16 17 17 18 18 19 19 20 20 21 21 22 23 24 24 25 25 26 27 28 30` (level 10 gives 16 of 32 = 50 percent per roll). A success starts state 4 and sets `+B3 = 0xB4` (**180 frame cool-down**, counted down every frame). Rolled every frame while walking and with no cool-down, so a gap of about 180 frames plus a few between attack sequences [R; M sl-dmg-1p: attack sequences about every 200 to 320 frames].
3. Otherwise it walks. The walk mode `+AC` (0, 2, 4, 6) is chosen from the previous mode (0x5DA14): `idx = ((prev & 6) << 4) + (rand & 31)` in the 128 byte table 0x5DA3A (rows of 32 entries, row = previous mode / 2):

   | previous | row of 32 (counts of mode 0 / 2 / 4 / 6) |
   | --- | --- |
   | 0 | 4 / 10 / 5 / 13 |
   | 2 | 8 / 4 / 5 / 15 (row starts at byte 32) |
   | 4 | 9 / 15 / 6 / 2 (row at byte 64) |
   | 6 | 6 / 15 / 0 / 11 (row at byte 96) |

   Mode 0 and 4 = walk to a point beside the target (offsets by the side `+7C`, 0x5D7C4); mode 2 = **hop** (anim 2, 7 frames 49 ticks); mode 6 = **run with afterimages** (anim 4, speed ramp, an afterimage every 13 frames, ends when the destination is reached or `+AE` = 300 frames ran out). The side of the target to stand on (`+7C`) comes from 0x5D606 (mode 0: the side the body is already on, mode 1: the screen side, mode 2: random from the 32 byte rows at 0x5D666, mode 3: none) [R].

**Attack scripts (+6 = 4)** [R 0x5CCF6..0x5D12C, tables 0x5CD9A, 0x5CDBE]: the combo id `+BA` (0..5) is chosen from **HP phase and distance** by 0x5DB2E: phase 0 = HP at least half of `+BE`, phase 1 = HP between a quarter and a half, phase 2 = below a quarter; distance band `min(dx, 0x80) >> 6` = 0, 1, 2 (under 0x40, 0x40 to 0x7F, 0x80 and more; see the quirk below); the row is `32 * (3 * phase + band)` bytes into table 0x5DB8E and `rand & 31` picks the entry:

| phase | band | combo 0 | combo 1 | combo 2 | combo 3 (out of 32) |
| --- | --- | --- | --- | --- | --- |
| 0 (HP >= 1/2) | near | 13 | 13 | 6 | 0 |
| 0 | middle | 7 | 18 | 7 | 0 |
| 0 | far | 5 | 14 | 13 | 0 |
| 1 (1/4 to 1/2) | near | 6 | 10 | 13 | 3 |
| 1 | middle | 8 | 11 | 9 | 4 |
| 1 | far | 4 | 13 | 10 | 5 |
| 2 (< 1/4) | near | 6 | 7 | 15 | 4 |
| 2 | middle | 4 | 9 | 14 | 5 |
| 2 | far | 2 | 10 | 14 | 6 |

**Quirk:** `dx = target X - own X` is compared with 0x80 as an unsigned word, so a hero to the left of the Slisaur always gives band 2 (far) [R 0x5DB60..0x5DB76]. The scale of the table is thus for a hero to the right.

The scripts (6 bytes per combo, `0xFF` ends the combo and the next combo is chosen at once; actions are indexes into 0x5CDBE):

| Combo | Script | Meaning |
| --- | --- | --- |
| 0 | 0, 2, 2, 2, 2 | approach to 101 px from the target (anim 1), then **four slashes** (action 2) |
| 1 | 4, 6 | decide: if the target is within 0x40 px switch to combo 0, else **run** (4.0 px per frame, up to 240 frames) until the depth lines match (|dz| < 7), then **one boomerang** (action 6) |
| 2 | 8, 0xA | **dash** to 145 px from the target, then **kick** (action 0xA) |
| 3 | 0xC | if the Slisaur is on screen: **fan throw** (anim 8); else come back with combo 2 |
| 4 | 0xE | **fan throw** (anim 0xB) at once (only the spawn rule above picks it) |
| 5 | 8, 0x10 | dash to 118 px, then kick |

Action details [R, anim data from `anim2.py 5d1c0 103000`]:

| Action | State `+7` | Anim | Attack | Notes |
| --- | --- | --- | --- | --- |
| 2 slash | 6 | 5 (4 frames, 8 ticks) | id **0x36** (base 10, poise 3, reaction 0, stop 15) | box 198 active ticks 2 to 7: reach 50 to 95 px in front, height 61 to 73, depth 8; chain of 4 with a recovery wait after each (state 0x12, wait `+80` from table 0x5D12E index by `rand & 31`, then table 0x5D14E by `(level & 0x18)`: 4, 4, 4, 6, 8, 10, 12, 14 frames at level band 0 down to 1, 1, 1, 1, 2, 3, 4, 4 at band 24) |
| 0xA, 0x10 kick | 0xE | 7 (6 frames, 41 ticks) | id **0x37** (base 16, poise 9, reaction 0x12, stop 12) | boxes 199 (ticks 6 to 8, reach 86 to 138 px) and 200 (ticks 8 to 11, reach 68 to 122 px): the longest reach of the fight; ends the combo |
| 8 dash | 0xC | 4 | none | afterimages, ramp; ends when within 4 px in X and 7 in Z of the point or after 240 frames |
| 6 single blade | 0xA | 6 (4 frames, 35 ticks) | blade | at the frame marker `+2D` the blade is created (d1 = type from 0x5CF06: 0 four times, 2 fourteen times, 4 fourteen times of 32) |
| 0xC, 0xE fan | 0x10 | 8 or 0xB | blades | at the frame marker: `n = T[rand & 31]` of table 0x5D058: **5 blades 16/32, 7 blades 10/32, 9 blades 6/32**; the blades take the first n entries of the direction list `5, 11, 3, 9, 2, 8, 1, 7, 0, 6, 4, 10` (12 directions around the body, all rows of table 0x5D078 are the same), type 6 |
| walk hop | `+6 = 2`, `+AC = 2` | 2 then 3 | none | 42 frame arc described above |

### 4.5 The blade (class 19) and what it does

- Type (the high byte of `+26`): **0 and 2** = straight boomerang from 0x40 px in front of the thrower, height +0x38, attack **0x3C** (base 7, poise 6, reaction 0x14, stop 6); **4** = same record but a different path (code 0x4528E); **6** = a fan blade, attack **0x3D** (base 13, poise 11, reaction 4 knock down, stop 18), spawned 0x50 px away, with its own path (code 0x452F4, directions from the 12 entry list) [R 0x45098..0x45184].
- Path of type 2 seen in MAME: flies at **5.5 px per frame** (11 px per 2 frames) at height 56 for 50 frames (x 1310 down to 1040), turns, and returns at **9 px per frame**; it passes the thrower and goes on [M s7-1p-a, A#22: x 1310, 1299, ..., 1040 at t = 580 to 630, then 1050 to 1302 at t = 632 to 660]. The blade registers in the monster hit list (0x4C0A every frame) and its `+5 = 2` handler (0x45308) makes it fall (vx 0x100, gravity -0x40) and be freed on landing, so **a hero's hit knocks a blade out of the air** (the supplement says the same of Slice's single boomerang; not run in MAME). It is also freed by the off screen test 0x173C or when its `+AE` life counter ends (10 frames per step, 0x451DE) [R].
- Damage to a hero: the blade reads a stale `+84` for the scale (HU08), so its damage is **7 or 8** at level 10 instead of the type-scaled value [M sl-dmg-1p: 7 and 8].

### 4.6 The afterimage trail (how it is drawn)

- Created by **0x9639E**: one pool C object of class 27 at the Slisaur's own X, Y, Z, **copying the current frame record pointer `+28`** and the facing `+24`; so a ghost is a still picture of the pose at that frame [R 0x9639E..0x963D4; M sl-fine: `anim=0x5D298` fixed for the life of a ghost].
- Life: **6 steps of 6 frames = 36 frames**. The palette override `+23` is `0x80 + {0x1D, 0x1C, 0x1B, 0x1A, 0x0C, 0x0B}` per step (table 0x9644A), a blue to green fade, and the sprite is drawn **only every second frame** (bit 0 of the counter `+5B`), which makes the ghost flicker [R 0x963D6..0x9643C; M sl-fine: pal 9D, 9C, 9B... seen on C#45 to C#47].
- Rate: in the dash (action 8) one ghost every **7 frames** (`+C0` counts 6 down to -1, 0x5CF54 to 0x5CF6A); in walk mode 6 one every **13 frames** (`+C0` from 0xC, 0x5CC5C to 0x5CC68). So at most 5 ghosts of one Slisaur exist at once [R; M sl-fine: new ghost at t = 265, 272, 279].
- The Slice code calls the same routine (0x4C37C, 0x4C67E): D3c reuses this object. The port needs pool C class 27 once for both bosses.

### 4.7 Hit reaction, death and what follows

- Reaction: the common table 0x2A606 via 0x81A7A: poise 11, the knock down chain `+6 = 6, +7 = 2, 4, 6`, then the body returns to state 0 (pick) with `+B3` unchanged (so there is **no wake up special** and no wind up: a freshly risen Slisaur rolls the normal gate) [R; M s7-1p-b: `st` 0206.. then 0200 0000].
- Jab damage from Jack to a Slisaur at level 10: 8 to 11 per jab (HP 450 to 410 to 25 by the poke) [M s7-1p-b].
- Death: HP 0 in the reaction; the Slisaur lies (state `+4 = 4`), `0x5D19E` awards 70,000 (`0x325E6`), waits 60 frames (`+80 = 0x3C`), then the body is freed (`+4 = 6`) [R 0x5D18C..0x5D1BA; M sl-kill].
- After the last one: kill-all, boss defeat flow (slow motion 368 frames, tally, walk, story panel of Episode 8): `G.sub` 2 at t = 2027, 4 at t = 2395 (368 frames later), 6 at t = 4196, mode 8 (story panel) at t = 5144 [M sl-kill; D4c T19].
- Reinforcements: the controller (class 22) adds thugs only while fewer than `{4, 5, 6}[players - 1]` enemies live (0x7BFEE); its threshold list needs `G.$502 = 1`, which the starter sets every 300 frames [R].

### 4.8 The user's and the web's notes checked

| Note | Verdict | Evidence |
| --- | --- | --- |
| "The Slisaur throws blades" | **True**: single boomerang (combo 1) and a fan of 5, 7 or 9 blades (combos 3, 4, only below half HP and rare) | 4.4, 4.5 |
| "moves really fast" | **True**: dash ramp up to 7 px per frame, run 4 px, afterimage every 7 frames in a dash | 4.3, 4.6 |
| "leaves an afterimage trail" | **True**: pool C 27, still copy of the pose, blue to green fade, flicker, 36 frames | 4.6 |
| "Episode 3 has only one Slisaur" | **False**: no code or script makes a Slisaur outside Episode 7; the single boss with this style is Slice (Episode 4) | 4.1 |
| "the second last episode has two or three by player count" | **True**: 2 for one player, 3 for two or three | 4.2 |
| Supplement: "repeat Slice's patterns with more health" | **Half true**: the HP table index is the same row as Slice (index 0), so equal HP for equal level; the starter shows the sum only internally | R D2c table 16.3 |
| Supplement: "no wind-up before the wake-up super" | **No such super**: after a knock down the AI returns to the normal pick state | 4.7 |
| Supplement: "each tracks its own target" | **True** (`+76` per body, re-picked every 600 frames) | 4.4 |

## 5. Episode 8 area 0: the lab tanks (pool B class 44)

### 5.1 The background and the four breakers

- The hall is a row of **11 glass cylinders painted 01 to 11** in scroll 2, each 144 px (0x90) wide, 1,792 px in all; creature art repeats in threes: 01, 04, 07, 10 a green crested lizard man, 02, 05, 08, 11 a pink muscular creature, 03, 06, 09 an orange raptor [M k8-start, k8-walk snapshots; `re/out/assets/stages/ep8_a0_scroll2.png`]. Each cylinder is a block of **6 rows by 7 tiles** (16x16) in the scroll 2 map; the 11 block addresses are in the long table 0x83DD8 (0x90F004, 0x90F244, 0x90F484, 0x90F6C4, 0x90F904, 0x90FB44, 0x90FD84, 0x90FFC4, 0x90F204, 0x90F444, 0x90F684) [R].
- The script creates only **four** pool B class 44 objects (the "tank breakers"); the tank is chosen by the high byte of the variant `+26` (`H`), the cylinder number is `H / 2 + 1`:

| Script record (area 0) | Variant | Cylinder | X (px) | Creature made | When | Seen in MAME |
| --- | --- | --- | --- | --- | --- | --- |
| L2 0xB1712, trigger 0x0001 | 0x0000 | **01** | 0x30 | **Bludge** (pool A class 6, var 0) | the first frame the camera X is 1 or more (the first step of the hero to the right) | t = 90 after control, Bludge at x 48, y 24, z 108 |
| L2 0xB1722, trigger 0x0001 | 0x0400 | **03** | 0x150 | **Rock Hopper** (pool A class 29, var 0x600) | same frame | t = 90, Hopper at x 336 |
| wave 0 of area 0, second group (alive at most 1, 2, 2 then the second group) | 0x0600 | **04** | 0x1E0 | **Bludge** (class 6, var 2) | with the second group of Blade or Razor (wave 0 of area 0) | t = 275 |
| L2 0xB1782, trigger 0x047F | 0x1400 | **11** | 0x5D0 | **Tyrog 2** (pool A class 91, var 0) | the camera reaching X 0x47F, just before the second lock (0x480) | t = 1085, x 1488 |

  All four sit at Z 0x6C. **Tanks 02, 05 to 10 never open**: no record creates them, and `B#44` objects were seen only four times in a 2,600 frame walk [M k8-walk: B#19, 20, 23 var 0000, 0400, 0600, 1400]. The user note "tanks 1, 3, 5 and 7 break" does **not** match the ROM: it is 1, 3, 4 and 11 (1 and 3 agree). The web (jars 3, 4 and 11 break) agrees on 3, 4, 11 and on Tyrog 2 from 11 [W blueprint], and "a Bludge and a dinosaur break out of the glass cylinders, then a second Bludge with knife guys, then Tyrog 2 with hunters" [W supplement 2] is exactly the order of the table.
- The creature table at 0x83C36 (rows of a class word and a variant, row = `H / 2`): row 0 Bludge var 0, 1 Tyrog 2 (class 91), 2 Rock Hopper var 0x600, 3 Bludge var 2, 4 Tyrog 2, 5 Hopper, 6 Bludge, 7 Tyrog 2, 8 Hopper, 9 Bludge, 10 Tyrog 2. The art in the cylinders (green, pink, orange) matches the creature of the row only by accident of the repeat: the green cylinders (01, 04, 07, 10) hold the Bludge look, the pink ones (02, 05, 08, 11) the Tyrog look, the orange ones (03, 06, 09) the Hopper look; the four live cylinders 01, 03, 04, 11 are exactly one of each, with the Bludge twice [R, M].

### 5.2 What a tank breaker does (handler 0x83BB0, 112 instructions)

Frame 0 (+4 = 0, init 0x83BC4): if the low byte of `+26` is not 0 it rolls `rand & 31 < 0x16` and frees itself 22 times in 32 (a random gate: **none of the four records has a low byte, so all four always break**); then `+C = 0x18` and the same frame continues into the next step [R].

Frame 0 step 2 (0x83BEA): `+4 = 4`; calls the glass effect **0x94640** (`d0 = 0x24, d1 = 0`): one pool C **class 21** (0x54) object with variant 0x24 at the tank position, which in turn makes **10 shard pieces of the same class** (variants 0x0E to 0x14, spread over x -16 to +24 and y 0 to 72, z 108) all in the same frame (22 pool C objects for the two tanks of the first step, 11 each) [R 0x94640..0x94758; M k8-fine: C#26 to C#47 created at t = 91]; then it takes a free pool A slot (0x483C) and makes the creature: copies X, Y, Z, class word and variant from the row of table 0x83C36 and the drop byte `+96` of its own record [R 0x83BF8..0x83C34].

Frame 1 (`+4 = 4`, 0x83C62): rewrites the cylinder in the background: `bsr 0x83CA6` copies **42 tile codes (table 0x83D84)** into the 6 x 7 tile block (empty cylinder with broken glass); `bsr 0x83D46` sets attribute bits (0x20 on the rim tiles) in the same block; then, **only for the Hopper and Tyrog 2 tanks** (byte table 0x83C96 indexed by `H / 2`: 0, 1, 1, 0, 1, 1, ... is 1 for variants 0x0400 and 0x1400 and 0 for 0x0000 and 0x0600), `bsr 0x83CE2` rewrites the palette bits of the block (table 0x83D22) and **sound 0x37** is played (parameters `d1 = 0x72010`, `d2 = 0x2002200`). The two Bludge tanks make no sound 0x37 and no recolour [R 0x83C62..0x83C90]. Frame 2 (`+4 = 6`, 0x83E06): freed. So the whole tank event is **3 frames** (t = 90, 91, 92 in k8-walk: B state 0, 4, 6) [M].

What the player sees (k8-fine snapshots): at t = 90 the cylinder is intact; at t = 92 it is empty with the glass shards flying and falling for about 40 frames (they have gravity, `+1C`); the creature appears at the same place: the Bludge drops from y 24 and lands at t = 94, then walks right at about 2.2 px per frame (x 48 at t = 94 to 144 at t = 138); the Hopper jumps out in an arc, x 336 to 385 over 44 frames, y up to 50, and lands at t = 136 [M k8-fine, k8-walk A#22, A#23].

### 5.3 The creatures after the break

- They are ordinary enemies of their classes: the Bludge (class 6) and the Hopper (29) with the entrance by variant (Bludge var 0 and 2, Hopper var 0x600, D3a, D3b). The Bludge is counted by waves and gives 8,000; the Hopper is a monster (not counted, no score, D3b).
- Tyrog 2 (class 91) is a normal enemy with the "mid boss" stats (5.4 below and section 3.6).
- The reinforcement of area 0: wave 0 starts at 0x120 (`B#15` var 0), the three Punk and Wrench "midgets" with dynamite (`A#64`, X 0x3B8, trigger 0x188), wave 1 at 0x2D0 (hammers), wave 2 at 0x480 (8 poachers). [R stage_scripts].

## 6. Episode 8: Dr. Fessenden (area 3, class 94 first, class 84 and parts second, class 95 fire)

### 6.1 Order and classes (corrects D2c table 16.3 and `episode-8.md`)

| Part | Class and address | Role |
| --- | --- | --- |
| **Form 1** | pool A **94** 0x67D2C (7,218 B), name 0x25 FESSENDEN, HP index 8, anim table 0x6EA48 (the Morgue table, 30 animations), boxes 0x103C00 | a 90 percent copy of Morgue (class 40, 2.6): human first, then a pale green dino man, then the growth scene |
| **Form 2 body** | pool A **84** 0x6173E (8,250 B), name 0x22 FESSENDEN, HP index 9, anim table 0x64452, boxes 0x102400 | the big pink two-headed dinosaur. made at the death of form 1 (0x698E8) |
| Arm | pool A **86** 0x63806, anim table 0x647D8, offsets 0x648EA | the arm or tail, three strikes (attack ids 0x5F, 0x60, 0x61) |
| Second limb | pool A **87** 0x63BC4, anim table 0x648AA, offsets 0x6496E | follows `+A3` (pose 8) only, no attack |
| Heads | pool A **88** (left, `+5C = 0`) and **89** (right, `+5C = -1`), one handler 0x63E30, anim table 0x646A4, offsets 0x649F2 and 0x64A76 | bite and fire |
| Body layer | pool A **85** 0x63778 | copies the body's position, palette and animation words and only draws (a second sprite layer) |
| Fire | pool A **95** 0x6995E, made by 0x69932 (called by the heads at 0x642AE) | the fireball |

All five parts have `+63 = 4` (multi part), `+7D = 0xFF`, are made at the body's init and **copy HP (`+6A, +6C, +6E`), the hit id `+5E` and the hit flag from the body** every frame (0x638CE, 0x63F08): a hit on any part is a hit on the body [R 0x61752..0x61872, 0x63806..0x63B9C].

The script (stage index 7, area 3 list 2 record 0xB1AB2) makes only **class 94** at trigger 0x10A0, X 0x1220, Z 0x130, var 0; list 1 of area 3 also has the lair props B#71, B#83 (var 0x100), B#84 at trigger 0x1140, X 0x12C0, Z 0x160 and B#66 var 7 at 0x10A8, B#50 var 4 at 0x1120 (D4c). The last wave (wave 11: 4 Poachers and 4 Blades at X 0xC90 to 0xE60, `END type 4`) comes before it [R stage_scripts 0xB1AB2, 0xB2090].

### 6.2 Form 1 (class 94)

**Start.** Init (0x67D42): name 0x25, `+84 = 0x94`, anim table 0x6EA48, `+63 = 2`, speed id 7, HP = index 8 of 0x31E14, poise 11, `+72 = 0xC8`, `+A0 = 1`, `+7D = 0xFE`, HUD pointer set, `+6C` saved in `G.$7048`. It then plays the **human intro**: anim 20 of the table (60 frames, **270 ticks**), state 0x0E, **276 frames**. The human (a man in a white coat, shown at the right edge of the screen at X 0x1220) does nothing and the hero can walk and fight the wave thugs. At t = 276 state 0x0E ends in 0x687A0: **music 0x0F** starts, `G.$7B4 = 2`, and the AI starts at state 8 (M f94-trans, f94-intro: state 0x0E for 276 frames then 0x08; HP 550 at the jump rank, 450 when spawned at level 10) [R 0x67D42..0x67E04, 0x6901C, 0x687A0; M]. The creature that fights is the pale green dino man of the Morgue table.

**HP** (index 8, 1P / 2P / 3P by band `level >> 2`): 1P 400, 420, 450, 470, 500, 520, 550, 600; 2P 500, 520, 550, 570, 600, 620, 650, 550 (the last value is in the ROM table); 3P 600, 620, 650, 670, 700, 720, 750, 750. Level 10 (band 2): **450 / 550 / 650**. Bar scale 100: 4.5 gauges at 450 [R 0x31E14 via hit_tables; D3c line 28 for the scale]. Kill score **8,000** (0x325E6 at 0x697AE). **No helpers**: class 94 does not make the controller and has no `G.$502` write; the thugs seen in the probe are the script waves (M f94-trans). `+7D = 0xFE`, `+78 = 11`, no `0xACEE` here (the hit handler 0x6965C only sets `G.$4D6 = 3` and clears `+6E`).

**AI = Morgue's skeleton (1.2)**: states 0x02 to 0x52 are the same table (41 entries), the moves are the **same 8 attacks as class 40** (ids 0x2B, 0x31, 0x40, 0x41, 0x42, 0x4A, 0x4B, 0x6C: base 12, 20, 17, 20, 20, 17, 15, 17) with the same boxes (headbutt box 114 reach 22 to 107 px, slap and swing box 116, charge box 115 reach 43 to 78 px, flurry five swings, see 2.6). Differences from Morgue, all in tables [R diff `diff_40_94.txt`]:

| Item | Morgue (class 40) | Fessenden form 1 (class 94) |
| --- | --- | --- |
| Phase timers | aggressive 120 x4, 150 x4, 180 x4, 210 x4 frames (0x5418A), defensive 60 x4, 70 x4, 80 x4, 90 x4 (0x541AA) | the same values at 0x68080 and 0x680A0 |
| Gate `rand & 31 < G[aggression][X class]` | table 0x73D54: row 0 = 16 18 16 18 10 2 4 10, plus 2 per row | table **0x74A42**: row 0 = **4 6 8 10 12 10 8 6**, rows 1 to 7 add 0 or 1 and 2 alternately (row 7 = 10 13 15 17 19 17 15 13): **a much lower gate** |
| Action table (32 rows, 8 X classes) | 0x73D94 | **0x74A82**: X classes 0, 1, 2: action 1 (headbutt) 24, action 3 (swing) 4, action 4 (flurry) 4; class 3: headbutt 8, swing 16, action 5 (run in, shoulder rush 0x42) 8; class 4: run in 32; class 5: run in 24, action 6 (**roar then charge**) 8; class 6: run in 8, roar and charge 24; class 7: roar and charge 32 |
| `+A2` level table | 0x744BA | 0x74BC2 (40, 40, 40, 35, 35, 35, ... 5): no use seen (U) |
| Helpers | controller | none |

So form 1 **headbutts up close, runs in and shoulder rushes in the middle, and roars and charges from far away** (the supplement's "more headbutt dashes" is a table difference). The roar then charge (action 6, 0x68FC2, only when the hero is more than 0x80 px away) is state 0x26 for 40 frames (anim 0x12 and a spark every 8 frames) followed by the charge 0x41 (speed id 0x24 = 7 px per frame, 30 frames). Measured with one hero at level 10: state 0x12 (headbutt wait) 12 to 114 frames, 0x14 40 to 100, 0x1C 24, 0x1E 48, 0x20 30 (the charge), 0x22 5 to 15, 0x26 40, 0x2C 48 to 53 (shoulder rush) [M f94-dmg]. Damage at level 10 (FESSENDEN row 42, hero HP 100, Jack): headbutt 0x40 **22** and shoulder rush 0x42 **26** (formula 22, 26, equal) [M f94-dmg]; the others by the formula: 0x41 and 0x31 26, 0x6C and 0x4A 22, 0x4B 19, 0x2B 15.

**Grab and hit.** The creature can be grabbed and thrown (`+63 = 2`): grabbed at t = 428, thrown, **29 damage** (550 to 521), lies about 77 frames and is back in action at t = 534 [M grab-f94b].

**Death scene** (the long part, M f94-trans, code 0x6965C to 0x698E8): at HP 0 the common knock down runs about 232 frames (t = 1268 to 1500: `G.$4D6 = 3` slow motion) and then `+4 = 4`. Steps of the dying routine (`+5` = 0, 2, ..., 0x12):

| t (after HP 0) | Step | What happens |
| --- | --- | --- |
| 232 (t = 1500) | 0 | `G.$4D6 = 1` (normal speed), caption window with the doctor's portrait (a bald man with glasses), **line 0x3D "PATHETIC FOOLS! NOW WITNESS THE TRUE POWER OF MY SERUM AS IT TAKES EFFECT!"**, sound 0x22, timer 240 |
| 475 (t = 1743) | 2 to 8 | caption cleared, **line 0x3E** (the same window; the snapshots show the same words as line 0x3D until t = 2091, so the two lines may carry the same text, U), the body turns to the arena centre (X 0x1120, skip the walk if it is within 0x40 to 0x140 px of it), **kill score 8,000**, anim 0x15 (the stagger, 198 ticks) and sound 0x83 |
| 673 (t = 1941) | 0xA to 0xC | anim 0x18 (150 ticks) |
| 823 (t = 2091) | 0xE | caption **line 0x40 "I'VE FINALLY DONE IT!!"**, anim 0x1D (100 ticks): the body grows into the pink dinosaur |
| 923 (t = 2191) | 0x10 | caption **line 0x41 "ALL THE STRENGTH OF THE MIGHTY DINOSAUR AND MY EXTREME INTELLIGENCE TOGETHER IN ONE BODY!!!"**, anim 0x18 |
| 1073 (t = 2341) | 0x12 | caption **line 0x42 "THE WORLD IS MINE TO CONTROL! HAHAHAHA!"** (text as read in the snapshots, spelling of the dialogue lines to be checked in D4d), anim 0x18 |
| 1223 (t = 2491) | state 6 | caption cleared; at t = 2492 **class 84 is made** at the same X, Y, Z and facing (var 0) with HP full (640 at level 10); the HUD pointer moves to the new object; the old object is freed [R 0x698E8; M] |

The slow motion of this death is only the first 232 frames (`G.$4D6 = 3`), and **no `0xACEE` is called**, so the stage does not end here. The 8,000 points come at t = 1743.

### 6.3 Form 2 (class 84 and its parts)

**Creation** (0x61752): needs at least 5 free pool A slots (`G.$65F8 >= 5`); it makes 5 objects, sorts the 6 slot addresses and gives the **lowest address to the body (class 84)** and the other five to the parts in the order 86, 88, 85, 89, 87; the creator is one of the six slots. The parts get `+B0` = the body (and the body gets the part slots in `+B0, +B2, +B4, +B6`) [R 0x61752..0x61878]. Init of the body (0x6187A): music **0x12**, name 0x22, `+84 = 0x88`, anim table 0x64452, boxes 0x102400, **HP index 9** (`+C2` keeps the start HP), poise `+78 = 0xA`, guard counter `+BD = 2`, `+7D = 0xFF`, `+63 = 4`, controller (0x7BEDC), `+C6 = 0`, `+AC = 3`.

**HP** (index 9, band `level >> 2`): 1P 600, 620, 640, 660, 680, 700, 700, 700; 2P 650, 680, 700, 700, 720, 740, 760, 780; 3P 750, 780, 800 (x6). Level 10: **640 / 700 / 800**. Bar scale 100: 6.4 gauges at 640 (the web's "seven bars" if the HUD rounds up, U). Kill score **1,000** (0x325E6 at 0x63552). **Helpers** are the controller lists of 1.6: timed groups of four Blades (class 63) every 1800 to 2400 frames and threshold groups.

**The decision** (action 0 at 0x61AE6; the body does one of 13 actions, `+A8` = 2 x action):
1. Every **16 frames** (`+C0`) a gate rolls `rand & 31 <= T[level]` with **T = 16, 20, 21, 22, 22, 23, 23, 24, 24, 24, 25, 25, 25, 26, 26, 26, 27, 27, 27, 28, 28, 28, 29, 29, 29, 30, 30, 30, 31, 31, 31, 31** for level 0 to 31 (level 10: 25, so 26 in 32) [R 0x62CFE, 0x64E00].
2. On success the **action picker** (0x62C62) takes the X distance from the target in 48 px classes (0 to 47, 48 to 95, ..., 288 or more) and reads `table[class][rand & 31]` from 0x64E20 (7 rows of 32): value 0 = both heads bite (action 3), 1 = both heads breathe fire (action 4), 2 = arm strike 0x5F (action 5), 3 = arm strike 0x60 (action 6), 4 = arm strike 0x61 (action 7), 5 = heads pose and walk to the **grab** (action 11), 6 = grab with the arm pose 8 (action 12). `+C6` is set to 0x1E0 (480 frames; at 0 the body goes back to action 0, 0x61ADA).

| dx class | 0 bite | 1 fire | 2 arm 0x5F | 3 arm 0x60 | 4 arm 0x61 | 5 grab A | 6 grab B |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 (0 to 47) | 5 | 3 | 4 | 4 | **14** | 1 | 1 |
| 1 (48 to 95) | 4 | 3 | 4 | 4 | **15** | 1 | 1 |
| 2 (96 to 143) | 6 | 4 | 6 | 9 | 3 | 1 | 3 |
| 3 (144 to 191) | 5 | 6 | 8 | 7 | 0 | 2 | 4 |
| 4 (192 to 239) | 5 | 7 | 7 | 8 | 0 | 1 | 4 |
| 5 (240 to 287) | 5 | **11** | 5 | 5 | 1 | 1 | 4 |
| 6 (288 or more) | 5 | **10** | 5 | 7 | 1 | 1 | 3 |

(counts of 32 rows) [R 0x64E20]. The body also **walks, dashes and steps** when idle: action 1 (0x61B3E) and 2 walk with random choices, action 8 (0x6231C) chases the target at 2 px per frame in X and 1.5 in Z for up to 180 frames, action 9 dashes 4 px per frame for 80 px, action 10 steps 2 px per frame in Z for 36 px (these are used by the counter system below and by action 0).

**Counter reaction (the "defence", 0x6314E and 0x6326E).** On every frame, for each hero in the normal state (`+4 = 0x200`) that stands within the zone (Z within 8 to 16 px, X within 256 px in front of the body, table 0x65044) **and is doing one of four moves** (hero `+C8`: 0x14 jump attack, 0x02 running jump attack, 0x13 dash attack, 0x15 the command move down, up and attack), a roll with the mask of the HP fraction decides if the body reacts: 12 in 32 while HP is at least 3/4 of the start HP, **15 in 32** at 1/2 to 3/4, **22 in 32** at 1/4 to 1/2, **27 in 32** below 1/4 [R masks 0xFFF00000, 0xFFFE0000, 0xFFFFFC00, 0xFFFFFFE0 at 0x6504A]. The reaction is chosen by the hero's move from 32 entries (22 of the preferred value, 2 each of the other five): jump attack: **Z step dodge** (action 10); running jump attack: **arm strike 0x60**; dash attack: **X dash dodge** (action 9); command move: **arm strike 0x61** [R 0x6505A]. This is why a grounded fight with jabs is safe and jumping in is punished more and more as HP drops.

**Parts and attacks** (damage at level 10, FESSENDEN row 42, Jack, hero HP 100; reaction ids as in 1.4):

| Attack | Id | Base, poise, reaction, stop | Box (x, w, y, h; active ticks) | At level 10 |
| --- | --- | --- | --- | --- |
| Arm strike A (class 86 anim 2, 43 ticks) | 0x5F | 20, 11, 6, 24 | box 176: -121, 45, -42, 26; ticks 11 to 22 | **26** [M f84-dmg] |
| Arm strike B (anim 3, 44 ticks) | 0x60 | 16, 6, 20, 22 | box 179: -120, 42, -36, 29; ticks 17 to 31 | **21** [M] |
| Arm strike C (anim 4, 16 ticks) | 0x61 | 10, 3, 2, 22 | box 181: -89, 40, -80, 38; ticks 5 to 11 | **13** [M] |
| Head bite (class 88, 89, anim 1) | 0x5E | 12, 4, 0, 22 | the head's own boxes (offsets table 0x649F2 and 0x64A76) | **15** [M]; the bite repeats while it hits, up to `+AE` times |
| Body grab (0x4E9A6 test, 0x627C4) | 0x64 | 16, 11, 0, 16 | held hero, applied by 0x125C0 | formula 21 (U) |
| Chew | 0x65 | 12, 4, 0, 16 | held hero, once per anim | formula 15 (U) |
| Fireball (class 95) | 0x62 | 20, 11, 10 burn, 16 | boxes 99 to 103 on the flight | not separated (6, 9, 11 and 18 were also seen from fire and thugs, stale rows HU08) |

Arm anims are those of table 0x647D8; boxes are given relative to the arm part, mirrored by facing. **Head attacks** (`+A4` for the left head, `+A5` for the right; requests 2, 4 and 6 from the body):
- request 2 (action 3): anim 1, attack 0x5E; on a hit (`+59 != 0`) the bite starts again until `+AE` is used up (`+AE` is copied from the body's `+AE`);
- request 4 (action 4, the **fire**): anim 5; at the animation frame where `+2C` is set (0x6415A) the head fires: sound 0xA1DC, a muzzle flash (0x92422 with d3 = 0x13), mouth position X -62 (+62 when facing right), Y -25 from the head, and **one class 95 fireball** by 0x69932 with a variant `v` from the **distance to the target**: `v = T1[dx / 32] + T2[rand & 31]` (dx = X distance between the body and the target, taken modulo 512), clamped to 0 to 8, **T1 = 16, 16, 14, 12, 12, 10, 8, 6, 4, 2, 2, 2, 2, 0, 0, 0**, T2 = +2 (x4), -2 (x4), 0 (x8), repeated [R 0x64210..0x642AE]. The fireball (class 95, 0x6995E) flies at about **10 px per frame at an angle**: variant 0 flat (-10, 0), 1 (-9.81, -1.56), 2 (-9.24, -3.06), 3 (-8.31, -4.45), 4 (-7.07, -5.65), 5 (-5.56, -6.64), 6 (-3.83, -7.39), 7 (-1.95, -7.85), 8 straight down (0, -8) px per frame (x sign by facing, y is the height: a negative value falls) [R 0x69C74]. So a close hero gets a steep ball that lands at his feet and a far hero a flat one. It explodes on a hit (anim 2, `+59`) or on the ground (anim 1, spark effect 0x9F12) with attack 0x62. The long action 4 repeats these volleys (**134 to 430 frames, mean 228** in M f94-trans) and the two heads alternate (`+A4` and `+A5`) [M];
- request 6 (action 10): the head only sways (anim 0 when the body's `+AC` bit 1 is set).

**The grab** (action 11 and 12; sub-routine `+5 = 4`, 0x6276E): the body walks to a point at X -112 or -144 px from the hero (tables 0x64F54 and 0x64F58), then tests `0x4E9A6` (hero alive, normal state, not airborne) at the key frame of anim 5 or 7. If it can, the hero is put in state `0x0206` (held, `+2E = 0xE`) with the pointers `+70` linked, `+C8 = 0x40`. The body then plays anim 6, at its frame 2 hits the held hero with **0x64** (16 base) and drops it in front (hero `+4..+7 = 0x02020000`, `+6 = 8`, at body X +-0x18, Y +0x70), then anim 8 and the chew: **0x65** (12 base) once per animation, `+AE` times from the table **0x650DA by level: 2 (level 0 to 7), 3 (8 to 15), 4 (16 to 23), 5 (24 to 31)**. The hold ends if the hero leaves state 0x206 or the count ends [R 0x627C4..0x62940].

**Poise and knock down** (hit handler 0x63460, run when the body's HP changed): the hit clears `+C8, +C9, +A8` and the part requests, tallies, and at HP 0 calls **`0xACEE`** (0x63492). Poise `+78 = 10` is reset at each hit and the guard counter `+BD` (starts at 2) is reduced by the word of the table at 0x6350C indexed by the body state `+6` (0, 0, 1, 1, 0, 1, 1, 0 ...: only hits taken in the action phases 4, 6, 10 and 12 count). When `+BD` reaches 0 the poise is set to 0 (a knock down) and `+BD` goes back to 2. Measured with a jab every 10 frames, the body fell on the 4th hit three times (t = 143, 203, 453, 527) and the supplement's "about six hits" is therefore **not exact** [M f84-rapid]. A grab by the hero worked at t = 42 to 44 (`+5 = 6`); the throw gave **5, 5 and 12** damage in two runs (class 94 gave 29) and the body was not kept down (U) [M grab-f84b, f84-rapid].

**Target** (`0x32032`, SPEC 5.2): the boss uses the common rule: the present player with the **fewest enemies already targeting him, ties to the later player**; so with equal counts it attacks the **highest numbered player** (P3 before P2 before P1), which is the supplement's "targets the rearmost player" [R].

### 6.4 Death of the final form and what follows

Code 0x63538 (dying) after the hit handler's `0xACEE` at HP 0 (`G.sub` 2). Measured in `f84-kill` (hero god mode; t counted from the spawn):

| t | Step (`+5`) | What happens |
| --- | --- | --- |
| 1527 | hit handler | HP 0, `0xACEE` (`G.$4D1 = 2`, slow motion `G.$4D6 = 4`, `G.$50E = 1`, stage index 7 sets the ending flags `G.$4D7` and `G.$4DB`), `G.sub = 2` at t = 1530 |
| 1895 | 0 (0x63552) | **score 1,000**, `0x6148` marks the last-blow hero, timer `+80 = 0x708` (1,800); `G.sub = 4` at t = 1900 |
| 1896 to 2498 | 2 | wait while `+80 >= 0x4B0` (600 frames) or `G.$503` is set |
| 2498 | 4 | anim 3, sound 0xA09E, the body walks at 1 px per frame to X camX + 0x120 |
| 2673 | 6 | anim 0x10 at the spot, 360 frames: at `+80 = 0x78` effect 0x8446C and anim 0xF |
| 3033 | 8 | anim 4 and a **hop to the left** (vx -1 px per frame, vy 0x300), landing: `G.$755C = 1` (screen shake), sound 0x9796, bounce |
| 3105 | 0xA | 180 frames of `G.$755C = 3` every 4th frame (quake) and a burst of explosion effects (0x27DAA) and debris (0x9764E) every 8th frame at random spots round the body |
| 3286 | 0xC | `G.$755C = 3` every 4th frame, `+A6 = 1`, `+1 = 0` (the body is hidden); this `+A6` flag is the one the Episode 8 script waits for (D4c open item 3) |
| 3700 | `G.sub` 6 | the tally starts (1,800 frames after step 0, as for every boss) |
| 7300 | mode 0x1C | the tally wait ended at 3,600 frames (the Episode 8 timer, not by the script clear: 7300 - 3700 = 3600) and the ending sequence begins |
| 12670 / 12850 | `G.sub` 8 / mode 0x18 | the end of the ending sequence and mode 0x18 (credits) |

[R 0x63538..0x63770; M f84-kill]. See section 7.

## 7. The Episode 8 ending trigger (answer to D4c open item 3)

What starts the ending sequence after the last boss, step by step (M `f84-kill`, one hero in god mode standing still, class 84 spawned at t = 10; R as in 6.4):

1. **Boss defeat.** The class 84 hit handler (0x63460) calls `0xACEE` the moment the body's HP is 0 (t = 1527). `0xACEE` sets `G.$4D4 = 1`, `G.$4D6 = 4` (slow motion), `G.$4D5 = 1`, `G.$4E0` = the body, `G.$50E = 1` (kill all), `G.$4D1 = 2`, and, because the stage index `G.$4D9` is 7, **`G.$4D7 = 1` and `G.$4DB = 1`** (the two ending flags) [R 0xACEE..0xAD34]. The 1,000 points and `0x6148` come later, at t = 1895 (step 0 of the dying routine).
2. **Dying routine** (6.4): about 1,400 frames of the body standing, walking to X camX + 0x120, posing, hopping, quake and explosions. At its last step (`+5 = 0xC`, t = 3286) the routine sets **`+A6 = 1` and hides the body (`+1 = 0`)** on every frame and never ends (`+4` stays 4). So `G.$4E0` keeps pointing at a hidden object whose `+A6` is 1 for the rest of the game [R 0x63754..0x63770; M].
3. **Tally.** `G.sub` is 6 at t = 3700, **1,800 frames after step 0** (the usual boss number, 1.5). The Episode 8 tally wait is 3,600 frames [R/M D4c table row 8].
4. **Ending.** `G.mode` becomes **0x1C at t = 7300**, exactly **3,600 frames after the tally started** (3700 + 3600). So in this natural kill the tally wait was ended by the **3,600 frame timer**, not by the script clearing `G.$4E2` (D4c: the script action 0x42 waits while the boss's `+A6` is set; here `+A6` was already 1 at t = 3286 and never went back to 0, so the script never cleared). **Conclusion for the port: in Episode 8 the ending always starts 3,600 frames after the tally starts** (the 540 frame clear sequence of script 24 is never reached by a normal kill; it was only seen in poked runs, D4c) [M f84-kill: G mode 0x0C sub 6 at 3700, mode 0x1C sub 4 at 7300].
5. **After.** The ending runs from t = 7300 to t = 12670 (`G.sub` 8) and then mode **0x18** at t = 12850 (credits). The content of these modes is D4d.

What the port must keep from this section: (a) `0xACEE` is called by class 84 at HP 0 (and **not** by class 94, class 91 or the parts); (b) the stage index 7 test in `0xACEE`; (c) the `+A6 = 1` flag and the hidden body staying alive in state 4; (d) the 3,600 frame wait. The probe script is `f84-kill` (`re/out/probe/d3d-r2/f84-kill`), 16,000 frames.

Open: the hero actors of the ending (class 97 stand-ins, `G.$516`) were not driven in this run (the hero stood still); D4d owns them.

## 8. Checks of the leads (user notes, web rows, supplements, blueprint)

There is no file `gpt_suggestions.md` in the repo: its rows G06 to G16 are in `plan/game/mechanics-seen.md` (lines 398 to 408) and the supplements and the blueprint are in `plan/game/sources/`. Verdicts: **C** confirmed by ROM or MAME, **P** partly true, **R** rejected, **U** not decidable in D3d (owner named).

| Id | Claim (short) | Verdict | Evidence |
| --- | --- | --- | --- |
| User 1 | The Slisaur boss throws blades, is very fast, leaves afterimages | **C** | single or fan blades (class 19), dash up to 7 px per frame, afterimage class C27, section 4 |
| User 2 | Episode 3 has one Slisaur; Episode 7 has two or three by player count | **R / C** | no code or script makes class 71 outside Episode 7 (section 4.1); the single fast blade thrower is **Slice** (Episode 4, D3c). Episode 7: 2 for one player, 3 for two or three (`G.$4EF`), section 4.2 |
| User 3 | At the start of Episode 8 tanks 1, 3, 5, 7 break and creatures come out | **R** | cylinders **01, 03, 04, 11** open (Bludge, Hopper, Bludge, Tyrog 2); 02 and 05 to 10 never open, section 5.1 |
| B07 | Slisaurs 2 for one player, 3 for two or three; D3d must find every creator of class 71 | **C** | section 4.1, 4.2 (the starter pool B 33 is the only creator) |
| W21 | Lab tanks 01, 02, 03 open and release Bludge, Hopper, Tyrog 2 | **P** | 01 and 03 open, 02 does not; 04 (a second Bludge) and 11 (Tyrog 2) do, section 5 |
| G06 | Slice has afterimages and boomerangs and is the user's "single Slisaur" | **C** (D3c has the numbers; the Slisaurs reuse the same blade class 19 and ghost class C27) | section 4 |
| G10 | Episode 5: the old villager starts to explain, Morgan shoots him; Morgan fights with fast gunfire and thrown weapons, then turns into Morgue | **C** | 2.1 (four bullets at t = 406 to 418, villager down), 2.3, 2.5 |
| G12 | Tyrog is a parasite: green host (bladed limbs), orange host (lunges, fireballs), blue host (mix and projectiles) | **C** for the three forms (the "host" is the same slot rewritten; the fireball form is orange, the grey and blue one has the lava volley) | section 3.4, 3.5 |
| G13 | Episode 7: a bridge, riders, a library with enemies from the shelves; Fessenden talks from the computer screen before the Slisaurs | **U** | scripts of Episode 7 areas 0 to 2: D3e part 3 and D4d; the Slisaur fight starts after the screen (4.2) |
| G14 | Episode 8: glass tanks break and release dinosaurs and mutants, several Bludges, Tyrog 2 as a mid-boss; pipes blow fire | **C** for the tanks and Tyrog 2, **U** for the pipes (D3e) | section 5 |
| G15 | Fessenden: first form like Morgue; final form a huge pink mutant with two reptile heads and the doctor's body (claws, projectiles, grabs, multi-part) | **C** | 6.2 (a clone of Morgue), 6.3 (pink body, heads 88 and 89, arm 86, fire, jaw grab, five parts) |
| G16 | Ending: self-destruct, heroes run, Hannah falls, Jack goes back, Cadillac | **U** (D4d); the trigger is section 7 | section 7 |
| L08 | Morgan jumps to the screen centre, Uzi, rocks, grenades, knives; Morgue has a ranged slap and a quick tackle and drops Morgan's Uzi; the second form brings henchmen back | **C** (hop 0x28 and 0x2A, class 68 var 2, 3, 4, class 26 var 0, Uzi burst; Morgue slap box 46 to 88 px and the 7 px per frame charge; the Uzi is a pool E item; Morgue makes the controller) | 2.3 to 2.6. "Phantom" is not a D3d class |
| L09 | Episode 6: no firearms; hitting Tyrog in the air always knocks him down; hosts Gneiss (slap, belly slash), Wrench T. (orange, fireballs), Walther (grey, acid spit in a circle); Tyrog finally dies on its own; spoken lines at each host change | **P / C** | the three move sets are the three forms: claws and chest blades (form 2), fireballs (form 1), the **lava volley on a cross of five spots** round the hero (form 0: offsets 0, +-48 in X, +-32 in Z, tables 0x4D49E and 0x4D51E; this is the web's "acid spit in a circle"). The host bodies of the ROM carry the name TYROG, not Gneiss, Wrench T. or Walther. The caption "SOMEONE HELP MMMPHH.. GURGLE!" is the spoken line at each host change (3.7). "Hit in the air always knocks him down" is **U** (not run). Tyrog "dies on its own" **C** (fight 3 ends the stage) |
| L10 | Episode 7: two Slisaurs for one player, three for two or more, higher HP in co-op | **C** | HP 450 / 600 / 700 each at level 10 (4.3) |
| L11 | Episode 8: jar 3 releases Bludge, jar 4 Rock Hopper, jar 11 Tyrog 2 (one form, does not revive, 1000 points); Fessenden's second form slashes, drops ground fireballs and calls Blade groups that throw knives | **P** | jar 11 Tyrog 2 **C** (300 HP, 1,000 points, no form change, 3.8); jars 3 and 4 are **swapped** in the web: cylinder 03 releases the Rock Hopper and cylinder 04 a Bludge, and the web misses cylinder 01 (a Bludge at the very start); form 2 **C** (arm strikes, fireballs, Blade groups of four from the controller) |
| M06 | Bosses have a fixed HP total regardless of rank; helper limit up by 1 or 2 with more players | **R** for the HP (it grows by level band and by player count, 1.3, 3.3, 6.2, 6.3); **C** for the helper limit `{4, 5, 6}` (1.6) | R 0x31E14, 0x7BFEE |
| M10 | Slisaurs: same patterns as Slice, more HP, no wind-up before the wake-up super, stay down longer, each tracks its own target | **P** | same HP table row as Slice (equal HP); no wake-up super exists; each body re-picks its own target every 600 frames; lying time not measured against Slice (U), section 4.8 |
| M11 | Morgan hops, throws knives, rocks, grenades, fires an Uzi, gives 0 points; Morgue has the Butcher's HP, a roaring headbutt rush and close slaps | **C** | Morgan 0 points (2.2); Morgue 512 / 576 / 768 equals the Butcher's only for 1 and 2 players (the Butcher has 640 for 3); the roar then charge and the slap are in 2.6 |
| M12 | Tyrog: chest bone spear, short arm hits, flying body press; the small head enters a fat enemy (fire) then a Walther (acid); HP refills at each change; one hit chance in the small-head phase; form 3 does not chase off screen; co-op multiplies the jellyfish HP; Tyrog 2 never absorbs and counterattacks when cornered | **P** | chest blades (class 74), claws, pounce **C**; HP refill **C**; **no "one hit chance" rule**: the host is hittable after 30 frames for 220 frames (3.7); co-op: HP 256 / 288 / 352 per form and 450 / 600 / 650 per host **C**; the arena clamp keeps every form on screen (the "does not chase" is **U**); Tyrog 2 never absorbs **C**; its "counterattack" is the wake-up claw of the recover entry (3.4, **P**) |
| M13 | Fessenden form 2 needs about six hits to be knocked down, ignores attacks mid-combo, grab and throw knocks it down at once, targets the rearmost player first | **P** | knock down on the 4th hit in the test (the guard counter `+BD` and the hit table 0x6350C, 6.3); "ignores attacks" is the guard counter and the counter reaction to jump attacks; the throw did **not** keep it down in `grab-f84b` (U); the target rule picks the **highest numbered** player among the least targeted **C** |
| N04 | Health in bars: sub-bosses (Tyrog 2) up to 3; Fessenden's last form 7 | **C / P** | Tyrog 2 has 300 HP at bar scale 100 = 3 gauges; the last form has 640 HP at level 10 = 6.4 gauges, 660 to 700 at higher levels = 7 gauges if the HUD rounds up (U, D4c) |
| FAQ 1 | Uzi shots knock Morgan off screen when well timed | **C** | one Uzi bullet knocks him down and slides him 108 px in 40 frames (2.3, M m5-uzi) |
| FAQ 2 | Jack's slide goes under Morgue's charge | **C by numbers**, not run | every Morgue box starts at height 37 or more (2.6) and the slide's body box is 35 high (D2a) |
| FAQ 3 | Tyrog: three phases chest blades, fireballs, lava spit; between phases grabbed and thrown but not combo-ed | **C / P** | 3.4; all forms grab and throw (27 damage, 3.6); the combo lock is **U** |
| FAQ 4 | Fessenden: phase 1 fights alone like Morgue; phase 2 two heads, seven health bars; about three minutes for both | **C** for the first two, **U** for the time (it depends on the player), seven bars **P** (above) | 6.2, 6.3 |
| FAQ 5 | Episode 8 opens glass cylinders: a Bludge, a dinosaur, then Tyrog 2 with hunters | **C** | section 5.1 (Tyrog 2 arrives at the second lock, 0x480, with the wave of Poachers) |
| Blueprint S5.4 | Morgue has a ranged slap and a quick tackle and drops the Uzi; beating Morgue reveals Fessenden | **C** (mechanics), the story part is D4d | 2.5, 2.6 |
| Blueprint S6.4 | A metal box opens; Tyrog attaches to a Gneiss; after each defeat a new host (Wrench T., then Walther) with a scream | **P** | a steel crate with four hunters breaks after 320 frames and a creature jumps out and becomes Tyrog; the hosts between forms are fat men whose class is 72 (3.2, 3.7). The names Gneiss, Wrench T., Walther do not occur in the code of these objects (U) |
| Blueprint S7 | Two Slisaurs, copies of Slice; the doctor on a computer screen | **P / U** | 4.4 to 4.8; the screen is D4d or D3e |
| Blueprint S8.3 | Fessenden: first form like Morgue; second form large, multi-headed, slashes, ground fireballs, Blade groups; "three heads in total (NM, PW4)" | **P** | two heads (classes 88 and 89) bite and fire; whether the doctor's torso is drawn in the body art is **U** (art, not checked); the rest **C** |
| Blueprint S8 end | Fessenden sets the lab to self-destruct, heroes run | **U** | section 7 gives the trigger; D4d the scenes |

## 9. (a) Acceptance scenarios (probe segments and what must match)

All segments are in `re/out/probe/d3d-r1` to `d3d-r4`; the generators are `re/out/d3d/gen_r1.py` to `gen_r4.py` (the segment text is in `r1.txt` to `r4.txt`). They run deterministically with the same input, so the port can replay the same input and compare these numbers. Times are probe frames `t` from the start of the segment; HP values are for one player at level 10 (rank 12 frozen) unless noted.

| ID | Segment | What must match |
| --- | --- | --- |
| BT-A1 | `m5-scene` (r4) | Episode 5 area 1 camera 0x1120: Morgan (class 35) walks left 60 frames from t = 290, points at t = 352, four bullets (class 59 var 5) created at t = 406, 410, 414, 418, villager down, class word of the slot becomes class 36 at t = 662 to 664, HP 256 |
| BT-A2 | `m5-dmg`, `m5-a` (r2, r1) | Morgan decisions: state entries 0x08 (approach, 44 to 104 frames), 0x10 (kite, 60 to 143), 0x20 (Uzi burst 40 frames, 5 shots 8 frames apart), 0x24 (throw, 30 frames per object, 3 to 5 objects), 0x1E (grenade, 80 frames), 0x28 and 0x2A (hop, 38 frames); the phase flag flips at t = 816 and 1096 |
| BT-A3 | `m5-uzi` (r3) | one Uzi bullet from the hero (11 damage): Morgan HP 256 to 245 at t = 859, knocked down, slid 108 px in 40 frames, down 100 frames; the next bullets flinch (10 each) |
| BT-A4 | `m5-trans`, `m5-b` (r2, r1) | Morgan dying `+4 = 4` at t = 2380, caption 120 frames, walk 60 frames to the centre, the Uzi item on the floor from t = 2520, class word 40 at t = 2682, HP 512, intro 136 frames with palette 0x9E, 0x9D, 0x9F, 0x6E, 0xAC, 0x00 every 2 frames |
| BT-A5 | `m5-morgue-dmg`, `m5-morgue-kill` (r2) | Morgue hits for 20 (0x6C), 17 (0x4B), 23 (0x42); kill: `G.sub` 2 at t = 3927, `+4 = 4` and `G.sub` 4 at t = 4295 (`0xACEE`, 30,000 points), `G.sub` 6 at 6096, panel at 7054 |
| BT-A6 | `s7-1p-a`, `s7-2p-a`, `s7-3p-a` (r1), `sl-dmg-*`, `sl-fine`, `sl-kill` (r2) | the starter makes 2 Slisaurs (3 with 2 or 3 players), HP 450 / 600 / 700, damage 13 (0x36) and 21 (0x37), kill chain `G.sub` 2, 4, 6 at t = 2027, 2395, 4196 and the next mode at 5144 (section 4) |
| BT-A7 | `k8-walk`, `k8-fine`, `k8-start`, `k8-t11` (r1, r2) | tank breaker objects at t = 90 (state 0), 91 (4), 92 (6), Bludge at x 48 and Hopper at x 336, shards (22 pool C objects in the first step), Bludge lands at t = 94, Hopper at t = 136; the second Bludge from cylinder 04 at t = 275 and Tyrog 2 from cylinder 11 at t = 1085, x 1488 |
| BT-A8 | `ty-full-a` (r3) | crate break at t = 416, class 76 puffs every 8 frames from t = 424, **class 4 var 0x200 at t = 648, HP 256**, first state 0x4C then 0x10 (22 frames) |
| BT-A9 | `ty-full-b`, `ty-full-c` (r3), `ty-chain` (r2) | identical state sequences: `4C(2) 10(22) 3E(10) 18(4) 30(6) 2E(6) 04(22) 3E(10) ...`; class 4 dying at t = 1506, **class 72 var 1 HP 450 at t = 1630**, class 4 var 1 HP 256 at t = 1878 (state 0x4C), dying again at t = 2706, class 72 var 0 at t = 3062, class 4 var 0 HP 256 at t = 3436; final kill chain `G.sub` 2, 4, 6 at t = 4909, 5229, 7030 and next mode at 7988 (the 4 to 5 gap is 320 frames) |
| BT-A10 | `ty4-v0-dmg`, `ty4-v1-dmg`, `ty4-v2-dmg`, `ty91-dmg` (r2) | pounce 0x4D = 26 at state 0x34; fireball 22; chest blades 27 (var 0, state 0x24) and 15 to 18 (var 1, state 0x46) on a fresh slot; state durations of 3.4 |
| BT-A11 | `grab-ty4v0`, `grab-ty4v1` (r3), `grab-ty4v2` (r4) | grabbed at t = 34, thrown, HP 256 to 229 (27), up at t = 161 (state 0x4C) in all three forms |
| BT-A12 | `ty-host`, `grab-ty72` (r3) | the host: state 0 for 30 frames, 4 for 20 to 22, 6 for 56 to 58, 8 for 140; HP 450; one hit of 14 delays the change by 124 frames; not grabbed in the test |
| BT-A13 | `f94-intro` (r4), `f94-dmg` (r2) | class 94 state 0x0E for 276 frames, then 0x08 and music 0x0F; headbutt 22, shoulder rush 26; HP 450 at level 10 |
| BT-A14 | `f94-trans` (r2) | HP 0 at t = 1268, dying at t = 1500, captions at 1500, 1743, 2091, 2191, 2341, **8,000 points at t = 1743**, state 6 at t = 2491, **class 84 with HP 640 at t = 2492**; thereafter fire volleys of 134 to 430 frames |
| BT-A15 | `f84-dmg`, `f84-trace`, `f84-rapid` (r2, r3) | arm strikes 26 (0x5F), 21 (0x60), 13 (0x61), bite 15 (0x5E); five parts created at t = 11; the body's action sequence (A8 = 12, 6, 22, 10, 8 in `f84-trace`); knock down on the 4th hit of a 10 frame jab chain (t = 143, 203, 453, 527) |
| BT-A16 | `grab-f94b`, `grab-f84b` (r4) | class 94 grabbed t = 428, thrown for 29; class 84 grabbed t = 44, released t = 57, damage 5, 5 |
| BT-A17 | `f84-kill` (r2) | `0xACEE` at HP 0 (t = 1527), `G.sub` 2 at 1530, step 0 at 1895 (1,000 points), `G.sub` 4 at 1900, steps 4, 6, 8, 0xA, 0xC at t = 2498, 2673, 3033, 3105, 3286, `G.sub` 6 at 3700, mode 0x1C at 7300, `G.sub` 8 at 12670, mode 0x18 at 12850 |

## 9. (b) Port status and the port task list

### Status

| Part | State |
| --- | --- |
| Boss HP row (`enemy_set_boss_hp`), kill chain flow (`flow_boss_defeated`), hit handler dispatch, the multi-part link 0x1128E (`enemy_combat.rs`, comment "Fessenden parts only") | DONE (not compared with these bosses) |
| Tank breaker class 44, Slisaur starter pool B 33, class 71 | MISSING |
| Morgan scene (35, 45, 67, 59 var 5), Morgan 36, class 68, Morgue 40 | MISSING |
| Tyrog scene (pool B 79, class 75, 76), class 4, 72, 74, 26 var 3 and 4, class 91 | MISSING |
| Fessenden 94, 84, parts 85 to 89, fire 95, the death scene captions, the ending flags | MISSING |
| Pool B 22 controller | MISSING (task P-D3c-1 of D3c) |
| Shared blade class 19 and ghost C27 | MISSING (task P-D3c-6; reuse) |

### Tasks (sizes in agent hours; one agent each, sequential, as D3c)

| Task | Scope | Files (new) | Size |
| --- | --- | --- | --- |
| P-D3d-1 | **Shared boss skeleton** (1.2): phase flag with timer tables, aggression gate, 32 x 8 action tables by class and form, recover entries (0x2B56C), target rule 0x32032 reuse, boss end timeline hooks (`G.$4D6`, `G.$50E`, `0xACEE`, `0x6148`, 0x325E6 scores) | `boss_common.rs` | 5 |
| P-D3d-2 | **Morgan scene and Morgan**: classes 35, 45, 67, 59 var 5, 36, 68 (variants 2, 3, 4), 26 var 0, Uzi, hop, the death talk and the rewrite to class 40, the Uzi item drop | `class35.rs`, `class36.rs`, `class68.rs`, `class45.rs` | 8 |
| P-D3d-3 | **Morgue** (class 40): moves, the roar then charge, tables, death, 30,000 and `0xACEE` | `class40.rs` | 4 |
| P-D3d-4 | **Tyrog**: starter B79, class 75 (variants of the scene and the host effects), class 76, class 4 (three forms, 5 actions, recover entry, death and rewrite), class 74, class 26 var 3 and 4, class 72 | `class4.rs`, `class72.rs`, `class74.rs`, `class75.rs`, `class76.rs`, `prop79.rs` | 12 |
| P-D3d-5 | **Tyrog 2** (class 91) on the shared class 4 code with its table differences | `class91.rs` | 2 |
| P-D3d-6 | **Slisaurs**: starter B33, class 71 AI and tables, reuse of class 19 and C27, the hit and death flow | `class71.rs`, `prop33.rs` | 6 |
| P-D3d-7 | **Tank breakers**: class 44, shard effect (pool C class 21 variants 0x0E to 0x14 and 0x24), tile rewrite from 0x83D84, creature table 0x83C36, sound 0x37 | `prop44.rs`, `effect21.rs` | 3 |
| P-D3d-8 | **Fessenden form 1** (class 94) with the human intro, music, gate tables, the death dialogue with captions 0x3D to 0x42 and the growth, the creation of class 84 | `class94.rs` | 5 |
| P-D3d-9 | **Fessenden form 2**: class 84 and parts 85 to 89, 13 actions, picker, counter reaction, guard counter, head fire and class 95, jaw grab, death sequence, `+A6` flag and the 3,600 frame wait | `class84.rs`, `class85_89.rs`, `class95.rs` | 12 |
| P-D3d-10 | **Acceptance runs** BT-A1 to BT-A17 with traces against MAME; the fix list | `harness` scenarios | 6 |

Total about **63 agent hours** (range 52 to 76; classes 4, 72 and 84 are the uncertain ones because their state tables were decoded by action, not state by state). Order: P-D3d-1, 7, 6, 2, 3, 4, 5, 8, 9, 10. Needs D3c-1 (controller), D3c-6 (class 19 and C27) and D4d (captions text lines 0x3D to 0x42, 0x50 to 0x53, 0x5A).

## 9. (c) Open items

1. **Morgan's own hits are 1 point above the formula** (0x3F 15 against 14, 0x34 18 against 17; Morgue and Fessenden are exact). Cause not found.
2. **Whether the Tyrog host (class 72) can be killed** in a real play: 450 HP in 220 frames with poise 1; only a 14 point hit was seen. If it is killed it gives 100,000 and ends the stage (BT11).
3. **The semantic names of many class 4 states** (0x0C, 0x16, 0x1A, 0x22, 0x24, 0x28, 0x2A, 0x34, 0x38, 0x3A, 0x44, 0x46, 0x4A, 0x4E, 0x50, 0x52) and the actions 0, 1 and 5, which no table uses. The state table at 0x36CA0 is listed in 3.4 by function, not by name.
4. **Slisaur lying time against Slice** (both read the same table, not measured side by side).
5. **Sounds and palettes** of all these bosses: ids are listed in the sections, the sound content is D4e.
6. **Whether a hero hit removes an Uzi bullet** (class 59).
7. **Horizontal speed rule of class 68 objects and of the lava rock** (steps of distance / 0x2800 or / 0x3800, units not decoded).
8. **HUD gauge rounding** (6.4 gauges at 640 HP; the web says seven): D4c.
9. **Fessenden grab test**: the throw damage on the final form was 5, 5 and 12 (class 94: 29) and it was not kept down; the rule for big parts (`+63 = 4`) is not decoded.
10. **Class 84 actions 1, 2, 8, 9, 10 and the reset 0x61ADA** are described by function (walk, chase, dash, step, 480 frame timer); the exact walk rules are in 0x61B3E and 0x61D26.
11. **Whether a jump clears the chest blades** (the hit box is low: y -16 to +5).
12. **Damage of class 95 fire and of the grab hits 0x64 and 0x65** (the values read were 6, 9, 11, 18 mixed with thug hits; stale rows).
13. **Texts** of the caption lines 0x3E, 0x50 to 0x53 (not read; 0x3D, 0x40 to 0x42 and 0x5A are in the sections).
14. **Class 94 palette animation** (0x67F94: sprite palette 0x1D rewritten from table 0xB7A52 by an offset that changes with `$17(a5)`): U.
15. **Whether a long chain locks a Tyrog form or class 84** the way it locks Vice (poise 11 and 10, refill `+79`).
16. **The Episode 8 ending wait**: in a natural kill with a standing hero it is the 3,600 frame timer (section 7); a run with the hero walking and the class 97 stand-ins is D4d's.
17. **`+A2` tables** (0x74B82, 0x74BA2, 0x74BC2): loaded, no use seen.
18. **ROM checksum call** at 1 in 16 Tyrog form changes (0x5A454): the port skips it; confirm that nothing else reads `G.$757A`.

## 9. (d) New rows for `mechanics-seen.md`

Format as in that file (`ID | mechanic | source | state | owner`). Source tags: R ROM address, M probe segment.

| ID | Mechanic | Source | State | Owner |
| --- | --- | --- | --- | --- |
| BT01 | Slisaur starter (pool B 33): 2 Slisaurs, 3 when `G.$4EF != 0`; the last one's hit calls `0xACEE`; the starter sums their HP and asks the controller every 300 frames | R 0x818E4, 0x81A7A; M s7-1p-a, s7-2p-a | MISSING | D3d |
| BT02 | Slisaur class 71: gate table 0x5DB0E, combo table 0x5DB8E, dash ramp 0x5DD1C, attacks 0x36 and 0x37, HP index 0 | R 0x5DD58..; M sl-dmg-1p | MISSING | D3d |
| BT03 | Slisaur blades (class 19, shared with Slice) and the afterimage (pool C 27, 6 steps of 6 frames, palettes 0x9D to 0x8B) | R 0x9639E; M sl-fine | MISSING | D3c, D3d |
| BT04 | Lab tank breakers (pool B 44): four records (cylinders 01, 03, 04, 11), 3 frame event, shards, tile rewrite, recolour and sound 0x37 for the Hopper and Tyrog 2 tanks | R 0x83BB0, 0x83C36; M k8-walk, k8-fine | MISSING | D3d |
| BT05 | Episode 5 scene: class 35 walks, shoots the villager (class 45) with four class 59 var 5 bullets, rewrites its class to 36 at t = 662 | R 0x513DA, 0x51658; M m5-scene | MISSING | D3d |
| BT06 | Morgan (class 36): gate 0x73B14, actions 0x73B54 and 0x73C54, kite, hop, throws (class 68 var 2, 3, 4), grenades (class 26 var 0), Uzi burst, 0 points, Uzi drop | R 0x51692; M m5-dmg, m5-uzi | MISSING | D3d |
| BT07 | Morgue (class 40): moves 0x40, 0x6C, 0x4B, 0x41 (charge 7 px per frame), 0x42, 0x2B, 0x31, 0x4A; the roar (40 frames, 5 sparks) then the charge; gate 0x73D54; 30,000 and `0xACEE` | R 0x53E08; M m5-b, m5-morgue-dmg, m5-morgue-kill | MISSING | D3d |
| BT08 | Tyrog crate scene: pool B 79, class 75 var 0 (320 frames), 1, 4, class 76 jump and puffs, class 4 var 0x200 at t = 648 | R 0x8A3D0, 0x5E702, 0x5EC26; M ty-full-a | MISSING | D3d |
| BT09 | Tyrog (class 4): three forms by `+26` high byte, HP index 6 halved at every init, music only for var 2, tables 0x74542 to 0x74802, actions 2, 3, 4, 6, 7, recover entry 0x36E7C with the wake-up claw | R 0x36A64; M ty-full-c, ty4-v*-dmg | MISSING | D3d |
| BT10 | Tyrog chest blades (class 74, attached child, 20 HP, low box, one hit tick) and fire (class 26 var 3 flat fireball, var 4 lava rock on a cross of five spots) | R 0x5E4F4, 0x4D59E..; M ty4-v2-dmg | MISSING | D3d |
| BT11 | Tyrog host (class 72): 30 frame intro, punch, jump, caption 0x5A, transformation anim 0x22 or 0x23, rewrite to class 4; 450 HP, poise 1, killable for 100,000 and `0xACEE` | R 0x5DD58; M ty-host, ty-full-c | MISSING | D3d |
| BT12 | Tyrog chain by class word rewrites 4 var 2, 72 var 1, 4 var 1, 72 var 0, 4 var 0; forms 2 and 1 give no score; the ROM checksum call at 1 in 16 | R 0x38DAC, 0x5E076, 0x5A454 | MISSING | D3d |
| BT13 | Tyrog 2 (class 91): fight 3 tables, 300 HP at every level, counted by the census, 1,000 points, no `0xACEE`, no music | R 0x654E2; M ty91-dmg | MISSING | D3d |
| BT14 | Fessenden form 1 (class 94): human intro 276 frames then music 0x0F, Morgue moves with gate 0x74A42 and action table 0x74A82, no helpers, 8,000 points, no `0xACEE` | R 0x67D2C; M f94-intro, f94-dmg | MISSING | D3d |
| BT15 | Fessenden death scene: 232 frame fall, five captions (0x3D, 0x3E, 0x40, 0x41, 0x42), growth anims 0x15, 0x18, 0x1D, class 84 made 1,224 frames after HP 0 with full HP | R 0x6967E..0x698E8; M f94-trans | MISSING | D3d |
| BT16 | Fessenden form 2 (class 84 + parts 85 to 89): five parts from the sorted slots, parts mirror the body's HP and hit id, 13 actions, 16 frame gate by level, picker table 0x64E20 by 48 px classes | R 0x6173E, 0x62CFE, 0x62C62; M f84-trace | MISSING | D3d |
| BT17 | Class 84 counter reaction: against hero moves 0x14, 0x02, 0x13, 0x15 with HP fraction masks (12, 15, 22, 27 in 32) it dodges or strikes with the arm | R 0x6314E, 0x6326E, 0x6504A, 0x6505A | MISSING | D3d |
| BT18 | Class 84 heads: bite 0x5E, fire volleys with class 95 variants 0 to 8 chosen by the target distance, jaw grab (0x64 once, 0x65 N times by level 2 to 5) | R 0x63E30, 0x64210, 0x69C74, 0x650DA | MISSING | D3d |
| BT19 | Class 84 poise: poise 10 reset at each hit, guard counter `+BD` (2), only hits in phases 4, 6, 10, 12 cost a point; knock down at 0 | R 0x63460, 0x6350C; M f84-rapid | MISSING | D3d |
| BT20 | Last boss end: `0xACEE` at HP 0 (stage index 7 sets `G.$4D7`, `G.$4DB`), 1,000 points at +368 frames, dying steps at 2498, 2673, 3033, 3105, 3286, `+A6 = 1` and the hidden body, tally at +1,800, ending mode 0x1C after 3,600 more frames | R 0x63538, 0xACEE; M f84-kill | MISSING | D3d, D4c |
| BT21 | Boss HP tables by band and player count for index 0, 4, 5, 6, 8, 9 (and Tyrog halved), HUD scale 100 | R 0x31E14 | PARTIAL (rows in `boss_hp`) | D3d |
| BT22 | Boss target rule: fewest attackers, ties to the later player (the supplement's rearmost) | R 0x32032 | DONE (D3a) | D3d |

## 10. Files used (all under the repo)

`re/out/d3d/` (scripts `dd.py`, `ann.py`, `moves.py`, `outline.py`, `anim2.py`, `cl.py`, `cl2.py`, `tyan.py`, `tyseq.py`, `tl.py`, `sthist.py`, `stdur.py`, `dmg.py`, `dmg2.py`, `hist.py`, `ctl.py`, `creators.py`, `cmpcls.py`, `gen_r1.py` to `gen_r4.py`, the class listings `c*_dis.txt`, `diff_40_94.txt`, `diff_4_91.txt`); the probe outputs `re/out/probe/d3d-r1` to `d3d-r4`; `re/out/dino_68k.bin`; `re/specs/enemy/stage_scripts.txt`, `hit_tables.txt`, `hp_table.txt`, `boss_hp.txt`, `SPEC.md`; `re/out/assets/anims/` and the animation tables read directly from the ROM; the probe tool `re/harness/probe/probe.sh`; the leads `plan/game/mechanics-seen.md`, `plan/game/sources/claude-doc-deep-dive-supplement.md`, `claude-doc-deep-dive-supplement-2.md`, `claude-doc-recreation-blueprint.md`; the earlier design files D2a, D2b, D2c, D3a, D3c, D4c, D4d for the shared rules.
