# D3e part 1: pool B inventory, and the area walk of Episodes 1 and 2

Written 2026-10-12 by the D3e research agent. Part 1 of D3e. Later agents add parts for Episodes 3 to 8 (section 11 says
what is left). Nothing here is ROM data: only descriptions, numbers and addresses.

Source tags: **R:** ROM address or repo file, **M:** MAME probe (the segment name is given; recipes in section 7),
**W:** web. MAME wins over the web. `[I]` marks an inference that no run or listing proves yet. "G.$xxxx" is RAM
`0xFF8000 + xxxx`. Pool B classes are numbered 0 to 96 (the class number is the long offset `+$20` divided by 4; the
scripts print the same number as `B#nn`).

How the work was done: a recursive-descent reader over the 97 handlers of the class table 0x2362E (instruction counts
below are real reachable instructions, data tables not counted); the probe tool with 12 batches of segments (a sweep
walker in every area of Episodes 1 and 2, spawn tests of all 97 classes, hit tests with the hero placed by `poke`);
the asset dump (maps, unreferenced compositions). Probe output and my scripts are in `re/out/probe/d3e-*` and
`re/out/d3e/` (git-ignored).

## 0. Summary of the findings

1. **Pool B has 97 classes** (table 0x2362E, handlers 0x74FA2 to 0x92382). The 8 stage scripts use **59 distinct classes**
   (`stage_scripts.txt`, as W24 of `mechanics-seen.md` says). The other 38 classes are spawned by routines or by other
   classes, or by nothing I could find (7 of them: 16, 35, 46, 51, 58, 65, 75). [R: scripts, static search of every call of the pool B allocator 0x4860]
2. **By role** (section 2.1): 23 tile or palette animations, 18 one shot scenery events, 6 breakables, 10 controllers,
   5 spawners, 4 caption or dialogue objects, 8 scenery effects, 22 front end, ending or story objects, 1 debug object.
   Port size with the sizes of section 8: 72 level classes (42 S, 25 M, 5 L), about 54 agent hours once the shared
   services exist, plus 22 front end classes (about 20 hours, they belong to milestones M1 to M3).
3. **W25 answer: the breakable pipes of the user are pool B class 40** (handler 0x836A4), in Episode 1 area 0 at map
   X 0x620, depth Z 0x130. A normal punch (hero depth between 296 and 320) breaks them; they stay broken and
   **blow steam for ever** (pool A class 15 pieces, 3 pairs alive at all times). The same class appears in Episode 8
   (4 records). [M e1-pipez*, e1-pipebreak]
4. **Only 6 pool B classes can be hit by players: 0, 1, 3, 12, 40, 45** (they register in the player hit list 0x671C
   with `0x4BFA`). Class 0 is a generic breakable wall panel with 15 variants, class 1 a chain-link fence (the two
   "fence" pictures of the catalog), class 3 a wall, class 12 var 4 the hall window of Episode 1, class 40 the steam
   pipes, class 45 a flame jet vent (Episode 8) that can be destroyed. No pool B class hurts players: hostile
   things are pool A (flames are A83). [R: static search of 0x4BFA, 0x4BEA, 0x4BDA, 0x4C0A, 0x4BB0, 0x4BC0]
5. **Most pool B classes are scroll tile writers.** They swap a block of 16x16 scroll2 tiles (or scroll1 8x8 tiles,
   scroll3 32x32 tiles) for a "broken" or "next picture" block, or cycle palettes. That is why only 7 pool B classes
   have an animation table (N13 of the asset catalog): the pictures are tiles in the stage map data, not sprites. Section 1.4 has the address
   formulas so the port can address tiles by map column and row.
6. **Things the plan did not know** (all [M]): (a) **a respawn air strike** (class 18): when a player drops back in,
   bombs fall in a line from the sky, a white blast and fire surround the hero (P22 and P40 of `mechanics-seen.md` are
   wrong about class 18, it is not a 1UP marker); (b) the end door of Episode 1 area 0 is smashed by the hero
   (class 11, spawned by a scripted control action); (c) the hall door opens and the hall window shatters (class 12);
   (d) the building front gets a hole and a wooden door bursts open in area 2 (class 14); (e) class 69 is a debug
   object that frees itself on a real game; class 66 contains an anti tamper check; class 96 is the area music
   starter; class 95 blends the sky colours (Episode 3); classes 83 to 85 are debris showers with quake.
7. **Unreferenced compositions** (section 2.5): fences are class 1, red pillars class 64, 1P/2P/3P tags class 68, the
   golden muscular man class 67 (select screen), "STAGE CLEARED!" and the digits are frames of **pool C class 9**
   (0x93184). "BREAK!", "Look out!", the spiked ball and the rest have no code reference: treat them as unused art
   (the spiked ball probably belongs to the Butcher's chain mace, class A62).
8. **Episode 1 and 2 walk** (sections 4 and 5): every interactive part of the 6 areas and 2 openings is listed with
   position, trigger, effect and class. Episode 2 has nothing a player can break in pool B; its scenery events are
   the water ripples (class 2), a far layer animation (class 7), a pit animation (class 27), bird flocks (class 72),
   jungle sound (class 80), and pool E barrels, tire stacks and treasures hidden behind tree trunks.

## 1. How pool B works (facts the port needs)

### 1.1 Objects and dispatch

- Pool B: 24 slots of 0xC0 bytes at `$36F4(a5)`, allocator 0x4860 (carry set when empty), free 0x4938, update loop
  0x23604 (after pool A, before pool C), dispatch `jmp CLASS[+$20]` through the table 0x2362E. [R spec section 1]
- Most handlers dispatch twice: first on the variant `+$26` (a long table of per-variant word tables, 4 bytes per
  variant: classes 3, 7, 10, 12, 14, 15, 22), then on the routine byte `+$04` (0 init, 2 run, 4 and 6 end or free).
  Others dispatch only on `+$04`. Timers live in `+$A0` and up. [R listings]
- The class numbers of the scripts are decimal (`B#40` is +$20 = 0xA0).

### 1.2 Who creates pool B objects

| Route | Facts |
| --- | --- |
| Script list 1 (props, 14 byte records: trigger, class, X, Z, var, Y) | spawn when the camera reaches the trigger X. Every full area spawns 69, 31, 20 at trigger 0 (debug object, clock, hint); the sub areas that keep the previous map (for example Episode 2 area 1) have none of them. [R stage_scripts.txt] |
| Script list 2 (16 byte records) and wave records (delay, flags, class ...) | wave controller (class 15) spawns pool B records of waves too (Episode 1: 12 var 0, 14 var 4; Episode 5: 49; Episode 6: 4; Episode 7: 52, 54; Episode 8: 42, 44) |
| Scripted control actions (table 0x1DBD6, scripts at 0x1E768, records of 6 bytes: action, G.$4F0, G.$4FD, G.$506, duration) | action 0x40 spawns the door smash (class 11), 0x46 the hint (class 20 var 2), 0x24 class 21. The flag byte G.$506 is read by class 12 var 4 (window). |
| Runtime code | respawn code 0x18BCE spawns class 18; the radio item routine 0x8BD24 spawns three class 88; the area start hook 0x9236C spawns class 96; boss code of pool A 5, 18, 25, 35, 43 spawns 50, 66, 76; the front end spawns 17, 19, 23, 24, 32, 58, 67, 68 and others |

### 1.3 Hit lists and what a hit does to a prop

The collision passes use per frame lists filled with small routines [R 0x4B9E to 0x4C18]:

| List (count, items) | Filled by | Used by |
| --- | --- | --- |
| 0x6718, 0x6720 | 0x4BC0 | player attack boxes (limit 3) |
| 0x671A, 0x6722 | 0x4BDA | attacker list: containers, weapons (pass 0x11962) |
| 0x671C, 0x6724 | **0x4BFA** | **props that players can hit** (pass 0x10CD0) |
| 0x671E, 0x6726 | 0x4BEA | hostile boxes of class 15 pieces |
| 0x6B30 | 0x4BB0 | thrown or fallen bodies (pass 0x1185A) |
| 0x6B64 | 0x4C0A | hostile pool A things (Zeke, Shivat, flames ...) |

Pass 0x10CD0 (R): for every player attack box and every prop in the list 0x6724, if the box test 0x128F8 hits
(index `+$49` of the attacker against `+$48` of the prop) it subtracts the damage of the attack table 0xA26FC from `+$6C`
(clamped at 0), copies the player index, sets the routine long `+$04` to 0x04000000 (routine 4, sub 0), stores the
attacker in `+$68`, plays the hit sound (0x962A), score hook 0xFFC8 and the hit spark (0x12720). **The prop never
checks its HP: one hit that overlaps the box always breaks it** (the 8 HP only matters for the display of the
variants that keep the number). The prop must be registered again every frame; the break routine simply stops
registering. A prop needs: `+$48 = 1` (body box index), `+$44` = box table, `+$6C` and `+$6E` = HP. [R 0x836B8, 0x78FC0]
Box record (6 words): depth offset and size, X offset and size (mirrored when facing), height offset and size
(`re/specs/anim/SPEC.md` section 4).

### 1.4 Scroll RAM tile geometry (to port the tile writers)

Each tile entry is 4 bytes: tile code word, attribute word (bits 0 to 4 palette, 5 and 6 flips, 7 and 8 more
attribute bits). The writers use `move.w` on both words, or `and` and `or` on the attribute word only. [R]

| Layer | RAM | Tile | Entry index from the offset `i = (addr - base) / 4` |
| --- | --- | --- | --- |
| scroll1 | 0x908000 | 8x8, 64 x 64 | row = `(i & 0x1F) OR (((i >> 11) & 1) << 5)`, column = `(i >> 5) & 0x3F`. +4 is the next row, +0x80 the next column |
| scroll2 | 0x90C000 | 16x16, 64 x 64 | row = `(i & 0xF) OR (((i >> 10) & 3) << 4)`, column = `(i >> 4) & 0x3F`. +4 next row, +0x40 next column |
| scroll3 | 0x910000 | 32x32, 64 x 64 | row = `(i & 7) OR (((i >> 10) & 3) << 3)`, column = `(i >> 3) & 0x3F`. +4 next row, +0x20 next column |
| palettes | 0x914000 | 32 per layer (OBJ +0, scroll1 +0x400, scroll2 +0x800, scroll3 +0xC00) | 16 colours of 2 bytes each |

The map is a ring of 64 columns, so a block address gives the map column modulo 64 only; to place a block on the map
use the object X (`+$8`) or the stage map data. The class 0 helper 0x75208 does this: it builds the address from
the object X and Y (column from X, row from Y).

### 1.5 Shared helpers used by many classes

| Address | What it does |
| --- | --- |
| 0x94640 | spawns pool C class 21 (the big debris and smoke effect, 60 animations) at the object, `d0` = effect variant, `d1` = facing |
| 0x959B4 | spawns pool C class 24 at the object (`d0` variant, `d1` facing) |
| 0x93362 | spawns one glass or debris piece effect (used 10 times by the window) |
| 0x967C4 | spawns small flame effects (class 45) |
| 0x9E60 | break sound with pan from the screen X (toggles sound 0x6F and 0xF6 with bit 0 of G.$43B; pan table 0xA66A) |
| 0x9EA2, 0x9F12 | pan helpers, rumble sound |
| 0xA63C | sound request (`d0` code, `d1`, `d2` parameters) |
| 0x41EEA | spawns a pool A class 15 piece at the object (`d0` kind 0 to 4, `d1` facing) |
| 0x483C, 0x4818, 0x4884, 0x48A8 | allocators: pool A, pool E, pool C, pool D (slot stored in `+$98`) |
| 0x1236, 0x121E, 0x120E, 0x16C6 | set an animation frame list, step it, set it by index of the table `+$40` |
| 0x49C0, 0x49CA | add the object to the draw list (the second one tests the on screen flag first) |
| 0x84F72 | tall font string printer (captions) |
| 0x1DAC4, 0x1DB5E | stage exit walk, scripted control step |
## 2. Inventory of the 97 classes

### 2.1 Counts

Roles: BRK breakable by players, EVT one shot scenery event, ANI periodic tile or palette animation, CTL controller or helper, SPW spawner, TXT caption or dialogue, FX scenery effect object, FE front end, ending, story or select screen, DBG debug.
Size is the estimate to port the class when the shared services of section 8 exist: S about 0.4 agent hours (a data driven tile writer), M about 1 hour, L about 2.5 hours (several states, many tables). `done` = already ported in `dino-rs` (classes 15, 22, 73).

| Role | Meaning | Classes | S | M | L | done |
| --- | --- | --- | --- | --- | --- | --- |
| BRK | Breakable by players (hit list 0x4BFA) | 6 | 0 | 5 | 1 | 0 |
| EVT | One shot scenery event (tile swap, shower, door) | 18 | 12 | 4 | 2 | 0 |
| ANI | Periodic tile or palette animation | 23 | 15 | 8 | 0 | 0 |
| CTL | Controller or helper (wave, clock, hint, music, boss helper, drive) | 10 | 4 | 3 | 1 | 2 |
| SPW | Spawner of pool A objects or sound | 5 | 2 | 1 | 1 | 1 |
| TXT | Caption or dialogue box | 4 | 2 | 2 | 0 | 0 |
| FX | Scenery effect object | 8 | 6 | 2 | 0 | 0 |
| FE | Front end, ending, story, select screen | 22 | 8 | 12 | 2 | 0 |
| DBG | Debug | 1 | 1 | 0 | 0 | 0 |
| all | | 97 | 50 | 37 | 7 | 3 |

Level classes (not FE, not done): 72 (42 S, 25 M, 5 L), about 54 agent hours. Front end classes: 22 (8 S, 12 M, 2 L), about 20 hours.

Script use: 59 classes are placed by the scripts (see the Episodes column). Classes with no spawner found anywhere (static search of every call of 0x4860, every script, every table): 16, 35, 46, 51, 58, 65, 75 (47 is made by 35, 21 only by script action 0x24). They may be leftovers.

### 2.2 The table

`Ins` = reachable instructions of the handler and its local routines (tile data not counted). `Handler` = address from table 0x2362E. `(n)` = number of script records. E1 a0 = Episode 1 area 0, w3 = wave 3. [R listings, scripts; the Spawned by column from a search of every `jsr 0x4860`]


| B# | Handler | Ins | Role | Spawned by | Episodes | What it does and draws | Hit, sound | Size |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0x74FA2 | 100 | BRK | script L1 | E4 a1 (3) | Generic breakable wall panel, 15 variants (box table 0x750B4, after-picture pointers 0x750F0, size bytes 0x75018). On a hit it rewrites a block of scroll2 tiles at its X,Y (address maths 0x75208), then effect C21 var 0xE (var 0) or 0x10 and, for var 0, a pool C class 24 spark (0x959B4). | Player hit list 0x4BFA every frame, HP 8, any hit breaks it | M |
| 1 | 0x75238 | 65 | BRK | script L1 | E4 a1, E7 a2 (3) | Breakable chain-link fence: sprite picture 326 (whole) then 327 (broken) from the frame list 0x75400; on a hit it rewrites 13 x 6 scroll2 tile attributes (0x75320, tables 0x75352, 0x7535E) and throws effect C21 var 0xC. Freed when the camera is 0x80 past it. | Hit list 0x4BFA, HP 1 | M |
| 2 | 0x7542C | 55 | ANI | script L1 | E2 a1 (1) | Swamp water surface: 12-byte records at 0x7550E (X limit, source and target tile rectangles); 4 pictures, one every 24 frames; also writes the phase word G.-$703C (0, 2, 4, 6). | none | M |
| 3 | 0x763F6 | 250 | BRK | script L1 | E6 a0 (var 0), E6 a1 (var 8) | Wall that breaks: var 0 registers in the hit list (HP 8, box 0x76666); the break rewrites scroll2 blocks (0x90E010, 0x90F000, 0x90F010 ...) and palette bits (0x90FD6A ...), plays the break sound with pan and spawns an effect. Variants other than 0 do the rewrite at once at spawn (the picture of a wall that was broken before: continuity into the next area). | Hit list 0x4BFA (var 0); break sound 0x6F or 0xF6 (0x9E60) | M |
| 4 | 0x767D8 | 141 | EVT | wave record (delay 0) | E6 w0 (var 4, 8), w2 (var 0xC) | Wall that bursts open at the start of a wave (enemies come through): waits 1 frame, rewrites scroll2 tiles (0x90FED0 ..; var 0xC has its own block 0x76910), pan sound, effect. | Break sound 0x6F/0xF6 | M |
| 5 | 0x769E2 | 32 | ANI | script L1 | E5 a1 (1) | Far layer (scroll3 0x913F00) animation, 4 pictures, 16 frames each, runs while the object is on or right of the camera reference G.$7E4. | none | S |
| 6 | 0x76A70 | 59 | ANI | script L1 | E5 a1 (3) | Scroll2 animation, 4 pictures of 12 frames; var 4 uses a 4+4 column block. The spawn test shows a pale blue and white 144 x 64 pixel block (steam or water, to confirm). | none | S |
| 7 | 0x76CD2 | 63 | ANI | script L1 | E2 a1 (var 4), E6 a1 (var 0) | var 0: 2 x 2 tiles at scroll2 0x90ED4C, 5 pictures of 24 frames then a pause of 96, stops when the camera passes 0xB80. var 4: 2 tiles of the far layer (0x913D04), 6 pictures of 24 frames, pause 96, stops when the camera has passed the object. | none | S |
| 8 | 0x76E0E | 107 | ANI | script L1 | E6 a1 (1) | Water surface like class 2 (15-tile rows, range list 0x76EDE, phase word G.-$703C). | none | M |
| 9 | 0x77E76 | 120 | ANI | script L1 | E6 a1 (3) | Two layer animation with random pauses: 5 pictures of 3 frames, then a second sequence; calls the sound helper 0xA25A. Region: 3 variants at X 0xA0, 0x260, 0x340 (mine). | sound helper 0xA25A | M |
| 10 | 0x785F0 | 113 | ANI | script L1 | E5 a1 (2) | Tile sets rewritten whenever the global tile clock G.$7AE changes (scroll2 0x90D4B4 and 0x90DC24, 63 tile writes); runs while the camera is within 0x100 left of its X. The routine 0x789C8 (spawns class 11) sits in this code range. | none | M |
| 11 | 0x789EE | 60 | EVT | scripted control action 0x40 (0x1DD30 then 0x789C8) | E1 a0 (area end door) | Metal door smashed by the hero: 3 frames after spawn it rewrites the door tiles (scroll2 0x90E9E4, attributes 0x90EA66), effect C21 var 2 and the break sound with pan. The doorway turns black. | Break sound 0x6F/0xF6 | S |
| 12 | 0x78B52 | 113 | BRK | script L1 (var 4), wave 1 (var 0) | E1 a1 (2) | var 0: hall double door opens (9 x 9 tiles at scroll2 0x90F288, 3 pictures of 4 frames). var 4: the window at the right end of the hall: waits for G.$506 (a scripted control record flag), then registers in the hit list (HP 8, box Z 16..96, X 880..928, Y -48..88); when hit it rewrites 3 x 7 tiles (0x90FDCC) and 12 attributes (0x90FE16), throws 10 glass shards (0x93362 at 0x38A, 0x60, 0x30), plays sound 0x37 and frees itself after 20 frames. | Hit list 0x4BFA (var 4); sound 0x37 | M |
| 13 | 0x79218 | 833 | EVT | script L1 | E5 a1 (8) | Large multi-block tile controller (833 instructions, 9 data blocks of about 700 bytes each); records at X 0xD20 to 0x1320, Y 0x100 or 0x200, var 0 to 0x10. Probably the burning huts (to confirm in part 2). | none | L |
| 14 | 0x7B0BE | 79 | EVT | script L1 (var 0), wave 4 (var 4) | E1 a2 (2) | var 0: a hole appears in the building front (4 x 7 tiles at scroll2 0x90C10C, effect C21 var 4) 8 frames after spawn. var 4: the wooden double door bursts open (4 x 9 tiles at 0x90E110, effect C21 var 6, sound 0x88, object moved to X 0x40, Z 0x120). | sound 0x88 | S |
| 15 | 0x7B336 | 113 | CTL | script L2 (wave start) | E1 to E8 (63) | Wave controller (0x7B336): spawns the wave script groups, waits for the alive count, 7200 frame rule, end types GO / area walk / stage exit. Ported. | sets G.$7A9, G.$50E | done |
| 16 | 0x7B520 | 10 | CTL | not found (unused?) | - | Stage exit walker: calls 0x1DAC4 and then runs the scripted control 0x1DB5E; frees itself when G.$4DD is 0. | none | S |
| 17 | 0x7B554 | 24 | FE | select screen 0xAE9C | front end | Select screen sprite at (0x180, 0x58) with a palette copy (0x914140 from scroll1 bank 0xBBA52). | none | S |
| 18 | 0x7B65E | 71 | FX | player respawn code (0x18BCE then 0x18C5E) | every respawn (the continue drop in uses the same player state, not tested) | Respawn air strike: five bombs (list 0x7B7A6) fall in a line from the top, 10 frames apart (pool E class 8, 0x4818), a white blast and explosions (pool E class 1 var 2) under the landing hero; the speech bubble (a threat that they will pay) comes from 0x65A4. | Bombs and explosions are pool E attack objects (damage not measured) | M |
| 19 | 0x7B7C6 | 37 | FE | select screen (table 0xB07E, 0xB0D2; var = player slot) | front end | Animated picture of the hero chosen by player N (reads the select table G.$62C + 0xC + 3 * var). | none | S |
| 20 | 0x7BB68 | 120 | CTL | script L1; scripted control action 0x46 (0x1DD0E) spawns var 2 | E1 to E8 (17) | GO hint: var 0 waits 600 frames with the camera still (G.$4FE), then blinks 3 times (35 frames each) a text block on scroll1 (0x90B238) and a palette (0x914540) with sound 0x3E; var != 0 starts the blinking at once. | sound 0x3E | S |
| 21 | 0x7BDC4 | 99 | EVT | scripted control action 0x24 (0x1DECC then 0x7BDAE) | script 4 of the scripted control table | One-shot tile attribute change: clears bit 8 of 48 words at scroll2 0x90F000 and sets attribute 0x2000 on 12 tiles at 0x90F02C (a layer swap behind the heroes). | none | S |
| 22 | 0x7BEF6 | 115 | CTL | boss code (Vice and others) via 0x7BEDC | E1 and boss fights | Boss reinforcement controller (0x7BEF6). Ported (spec section 12). | spawns pool A records (0xFDAE / 0xFE6A) | done |
| 23 | 0x7C0E4 | 42 | FE | front end (0x9915A, 0x9B4DA, 0x9C0B2) | title, story, movie | Palette flash object: two palettes (0x9149A0, 0x914A00) cycled in 8 steps, lives 300 frames. | none | S |
| 24 | 0x7E2D0 | 249 | FE | front end (0x99138, 0x9B4B8) | opening movie | Movie text crawl: raises the scroll offset G.$6C4 by 3 per frame and prints 6 tall font lines at fixed frames (table 0x7E352, 0x2A8 frames); spawns class 25 twice. | none | M |
| 25 | 0x7E7B4 | 265 | FE | class 24 | opening movie | HUD style pool D object (state 5) that reads values of two other objects (+$58, +$5C pointers); role to confirm. | none | M |
| 26 | 0x7EC48 | 177 | FE | class 25 (0x7EC22) | front end | Scroll1 text block writer: reads 4-byte records (column, row, string) and writes them into 0x908000 to 0x90A000. | none | M |
| 27 | 0x7EEBA | 344 | ANI | script L1 | E2 a2 (1) | 4 picture animation, 24 frames each, over two scroll2 regions (9 rows at 0x90E03C and a 313 write block at 0x90F000); runs while the camera is within 0x1A0 of its X. The big block lies in the lower left of the area 2 map, where the hatched green pit is (to confirm). | none | M |
| 28 | 0x7F72A | 195 | ANI | script L1 | E8 a1 (5) | Tile animation keyed to the global clock G.$7AE (blocks 0x90C030, 0x90C420, 0x90C820, 0x90CC2C ...), variants 0 to 8. | none | M |
| 29 | 0x8048A | 43 | ANI | script L1 | E8 a1 (7) | Same family as 28, table 0x80518 (var 0 to 0x18). | none | M |
| 30 | 0x811DE | 283 | EVT | script L1 | E7 a3 (1) | Machine event of the lab (283 instructions, 5 tables): sound 0x23 at init, rewrites scroll2 blocks near 0x90F2C8, sets G.$4FC, calls 0x81B24 (spawns class 34 caption). | sound 0x23 | L |
| 31 | 0x81046 | 84 | CTL | script L1 | E1 to E8 (19) | Stage clock: draws m:ss at screen (0xE8, 0xB8) from a pool D slot when time <= 3:00 (shown 300 frames at each 30 s mark, 800 frames at 0:15 or less, red at 1:00 or less), beep every second while shown. | sound 0x34 | M |
| 32 | 0x817CE | 58 | FE | movie code (0x99356, 0x9B702, 0x9C2BA) | opening movie | Black bars of the movie (letterbox) on scroll1 rows 0 and 47. | none | S |
| 33 | 0x818E4 | 109 | SPW | script L1 | E7 a3 (trig 0x3C0) | Slisaur boss starter: spawns 2 (3 with 3 players) pool A class 71 at X 0x580, Z 0x30, 0x40 (0x50), HUD name, sets G.$502, kill-all flag handling (0xACEE), sound 0x11. | sound 0x11 | M |
| 34 | 0x81B3E | 40 | TXT | class 30 (routine 0x81B24) | E7 lab | Caption box with a long villain speech (3 lines in the tall font): he boasts that the heroes cannot stop the maker of a new world (paraphrase). | none | S |
| 35 | 0x81FDE | 65 | FX | not found | boss fights [I] | Object attached to a pool A object (+$76 handle): copies its frame selection from table 0x820FE by the owner class and +$2C; draws with the floor test. Spawns class 47 through 0x8426A. Role to confirm. | none | M |
| 36 | 0x8217E | 78 | ANI | script L1 | E8 a1 (2) | Tile animation (clock G.$7AE), blocks 0x90C038 and 0x90C438. | none | S |
| 37 | 0x82372 | 37 | ANI | script L1 | E8 a1 (3) | Tile animation (clock G.$7AE), table 0x823EE, vars 0, 4, 8 (Z 0x300, 0x200, 0x100). | none | S |
| 38 | 0x8279C | 72 | ANI | script L1 | E8 a1 (6) | Tile animation (clock G.$7AE), vars 0 to 5. | none | M |
| 39 | 0x830C6 | 77 | ANI | script L1 | E6 a2 (1) | 4 pictures of 24 frames in two regions (scroll2 0x90E02C and 0x90EC2C); runs while the camera Y is below 0x170. | none | S |
| 40 | 0x836A4 | 78 | BRK | script L1 | E1 a0 (var 0), E8 a0 (var 4), a1 (var 8, 0xC, 0x10) | Steam pipes in a wall (see section 3.1). 11 tile patch, then steam pieces (pool A class 15) every 6 frames for ever. | Hit list 0x4BFA, HP 8, box 0x8380A / 0x83822 | M |
| 41 | 0x8383A | 23 | ANI | script L1 | E7 a3 (3) | 4 pictures, one step per frame counter, tables 0x838x; runs while the camera is within 0x100. | none | S |
| 42 | 0x83A0A | 34 | EVT | wave record | E8 w3, w4, w6, w7 (4) | Wave timed break (4 rows of 13 tiles, effect, pan sound 0x9E60 and sound 0x26 at the end); the lab wall opens. | sound 0x26 | M |
| 43 | 0x83ACA | 37 | ANI | script L1 | E8 a1 (7) | 4 pictures of 8 frames; runs while the object is within 0x20 of the camera. | none | S |
| 44 | 0x83BB0 | 112 | SPW | script L1 and wave 0 | E8 a0 (3 + wave) | Lab tank: waits (random 0..31 vs 0x16 gate), breaks its glass (effect C21 var 0x24), spawns a mutant (pool A, 0x483C) and plays sound 0x37. A green haired mutant jumps out (seen in the spawn test). | sound 0x37 | L |
| 45 | 0x83E0C | 168 | BRK | script L1 | E8 a1 (6) | Flame jet vent with HP 0x30: idle (random wait) then small flames (effect 0x967C4), a growing column of flames (pool A class 83 via 0x6152C), full flame, shrink; destroyed when its HP is 0. | Hit list 0x4BFA (can be hit and destroyed); flames are pool A class 83 | L |
| 46 | 0x8419E | 30 | EVT | not found | - | Wall break with pan sound at scroll2 0x90EB08 (14 tile writes); no script or code spawns it. | break sound | S |
| 47 | 0x84282 | 36 | EVT | class 35 (0x8426A) | - | Two scroll2 patches (0x90F86C, 0x90F872) written when G.$4F0 is set. | none | S |
| 48 | 0x84316 | 12 | EVT | script L1 | E5 a0 (1) | 12 instructions: waits 128 frames and does nothing visible (a stub). | none | S |
| 49 | 0x8438E | 45 | EVT | wave record | E5 w6 (1) | Wall patch (4 columns x 10 rows, then 7 more tiles) with effect 0x16, then spawns two class 50 dialogue boxes (0x84470, 0x84494). | effect C21 var 0x16 | M |
| 50 | 0x844B4 | 62 | TXT | script L1; boss code (A5 0x38F54, A18 0x426CC, A25 0x4BF5A) and class 49 | E5 a1, E8 a3; boss intros | Dialogue box with a portrait and text (var picks the line); prints with the tall font printer 0x84F72 and G.$44A / G.$44C. | none | M |
| 51 | 0x85088 | 63 | EVT | not found | - | Wall break after 96 frames (3 blocks at scroll2 0x90DA64, 0x90DA72, 0x90DAA8 ...), pan sound, effect. | break sound | S |
| 52 | 0x85178 | 29 | EVT | wave record | E7 w12 (1) | Wall break (scroll2 0x90FBC4), pan sound, effect. | break sound | S |
| 53 | 0x85306 | 34 | CTL | script L1 | E8 a2, a3 (2) | Palette set switcher: 27 palettes updated whenever G.$7B2 changes (lights or alarms of the lab); G.$7B3 keeps the last value. | none | M |
| 54 | 0x855A8 | 40 | FX | wave record | E7 w4 (1) | Two water jets on scroll1 (8 x 8 blocks at 0x90A2BC and 0x90B1BC), effect 0x1E and pan sound once (G.$7B6 flag). | break sound | S |
| 55 | 0x8574C | 44 | FX | script L1 | E6 a1 (1) | Rain: scroll1 streaks over the whole screen (0x90A000 block, helper 0xF94). | none | S |
| 56 | 0x85916 | 102 | FX | script L1 | E6 a1 (1) | Falling dust or rock bits: 6 slots, every 240 frames, spawns pool C class 2 objects with random timing; sound 0x8D. | sound 0x8D | S |
| 57 | 0x85B64 | 48 | FX | script L1 | E7 a1 (1) | Lift floor counter in red digits (scroll2 0x90C268, one number per 0x100 pixels of lift height, starts at Y 0x3760) and a scroll3 write. | none | S |
| 58 | 0x85E32 | 52 | FE | not found (probably the flow code through a table) [I] | link scenes | Map picture panel (photo with a name plate) on scroll1 0x90B408; uses G.$6C4. | none | M |
| 59 | 0x861E8 | 49 | ANI | script L1 | E4 a2 (3) | Palette cycling for stage 4 (3 variants), table 0x862B6. | none | S |
| 60 | 0x863AC | 55 | ANI | script L1 | E6 a0 (1) | Palette cycling on scroll palette 0x914A00 (stage 6 mine lamps), 8 steps. | none | S |
| 61 | 0x864DA | 50 | ANI | script L1 | E7 a3 (1) | Palette cycling for stage 7, with sound 0x6D every 120 frames. | sound 0x6D | S |
| 62 | 0x865F2 | 69 | ANI | script L1 | E5 a1 (1) | Palette cycling for stage 5 with sound 0x8E. | sound 0x8E | S |
| 63 | 0x869E6 | 42 | ANI | script L1 | E7 a3 (1) | Palette cycling for stage 7 (guarded by stage 6 test: 0x86A18). | none | S |
| 64 | 0x86B64 | 21 | EVT | script L1 | E7 a1 (4) | Red pillars of the lift shaft: sprite frames at 0x86BE4 to 0x86C08 (compositions 1897 to 1900), placed from G.$6C4 (lift height) and the var. | none | S |
| 65 | 0x86C14 | 51 | FX | not found | - | Animated scenery object with floor test and frames 0x86D00; spawns class 66 twice. | none | S |
| 66 | 0x86D76 | 93 | TXT | script L1; boss code (A18 0x427B2, A43 0x55FA6) and class 65 | E1 to E8 (27) | Caption banner on scroll1 (place names, boss speech); var 0 to 0x5B picks the string (table 0x87710), the timing tables 0x86DD8, 0x86EB4, 0x875F2, 0x87652. Contains an anti-tamper check (0x86B3C) that turns it into class 87 with 1 in 128 chance if the interrupt code is patched. | none | M |
| 67 | 0x8881A | 19 | FE | select screen (var 4, 0xC) | front end | Select screen frame art; the golden muscular man (composition 2366) frame lives at 0x888A6. | none | S |
| 68 | 0x888B2 | 67 | FE | select screen (var 0, 4, 8) | front end | Player tags 1P, 2P, 3P (compositions 2237 to 2239) over the hero; X from the join table G.$862C; palette cycling. | none | S |
| 69 | 0x88A84 | 216 | DBG | script L1 | E1 to E8 (19) | Debug object: frees itself at once unless the debug flag G.$4AA is set; then it is a free fly cursor with a secret input code, an object placer (table 0x88DDA), rank and level editor (G.$5EC, G.$5EE). | reads the pad 0xF1C001 / 0x800000 | S |
| 70 | 0x896E6 | 14 | FX | script L1 | E5 a1 (1) | Animated scenery object with floor test (frames 0x89736), freed when off screen. | none | S |
| 71 | 0x89746 | 87 | ANI | script L1 | E8 a3 (1) | Palette animation of the end area (0x914820 .. 0x914900), 6 steps. | none | S |
| 72 | 0x89984 | 59 | SPW | script L1 | E2 a0 (2) | Bird flock spawner: 5 (var 0), 7 (var 0x200) or 11 (var 0x400, unused) pool A class 46 birds, 8 frames apart, at camera X + 0xB0 + random, screen top; palette 0x98. | none | S |
| 73 | 0x89ABA | 25 | SPW | script L1 | E1 a0 (1) | Sky bird spawner. Ported. | none | done |
| 74 | 0x89B48 | 114 | CTL | script L1 | E1 a0 (1) | Master of the opening scene: spawns A54, A56, A55 (6 thugs) and sequences them with +$05. Not ported. | none | M |
| 75 | 0x89DA6 | 59 | FE | not found | - | Scene object: spawns A54, A55 and B76. | none | M |
| 76 | 0x89EDE | 62 | FE | A5, A18, A25, A35, class 75, front end | boss intros, scenes | Scripted camera mover: writes G.$28, $2E, G.$7E4 and G.$7E8. | none | M |
| 77 | 0x8A090 | 25 | FE | class 76 | scenes | Animated scenery piece with frames; spawns class 78. | none | S |
| 78 | 0x8A156 | 174 | FE | class 77 | scenes | Scroll1 full page writer (0x908000 to 0x90BFFF). | none | M |
| 79 | 0x8A3D0 | 81 | EVT | script L1 | E6 a2 (1) | Boss scene object (Tyrog area): spawns pool A class 75 pieces, sound 0x21; a group of thugs with caption lines is drawn. | sound 0x21 | M |
| 80 | 0x8A582 | 25 | SPW | script L1 | E2 a0 (1) | Ambient sound maker: every 120 frames with chance 1 in 4 and then a coin flip it plays sound 0x78 (the other branch is empty). | sound 0x78 | S |
| 81 | 0x8A5FA | 154 | FE | class 82 | ending | Ending desert: scroll2 blocks, G.$507, camera X, sounds 0x19 and 0x1A; spawns itself. | sounds 0x19, 0x1A | L |
| 82 | 0x8AA0A | 307 | FE | ending code (0xA6ABE) | ending | Ending scene controller (307 instructions): spawns pool B class 81 and pool A classes 3, 83, 84, 85, 87, 90, 96 (ending cast and fire), palette fade (0x89E6, 0x892C), sound 0x56. | sound 0x56 | L |
| 83 | 0x8B212 | 37 | EVT | script L1 (var 0x100); ending (vars 0x600, 0x700) | E8 a3 (1), ending | Debris shower: spawns pool A class 92 pieces (explosion and hook debris) every 4 frames for 50 to 200 frames (var picks the length) and the rumble helper 0x9F12 every 16 frames. | rumble helper 0x9F12 | S |
| 84 | 0x8B2CA | 26 | EVT | script L1 | E8 a3 (1) | Debris shower: spawns pool A class 93 pieces every 32 frames for 400 frames (var 0x10: every 8 frames for 100). | none | S |
| 85 | 0x8B34C | 45 | EVT | ending code | ending (E8 end) | Strong quake: every 32 frames asks for a screen shake (G.$755C = 4) with the rumble helper, spawns pool A class 92 var 0x500 debris at 8 table positions (0x8B402) every 4 frames for 100 frames. | rumble helper 0x9F12 | S |
| 86 | 0x8B428 | 62 | FE | HUD code 0x152F6 | continue | Continue screen object: spawns class 17 effect, uses the countdown G.$4DC. | none | M |
| 87 | 0x8B60E | 97 | FE | class 82 and class 66 (anti tamper) | ending | Staff roll text on scroll1 (STAFF), G.$6C8. | none | M |
| 88 | 0x8BDDC | 638 | CTL | RADIO item routine 0x8BD24 (3 objects, var 0, 2, 4) | E3 (radio) | Cadillac drive controller: sets G.$518, uses the player tables G.$327C.., camera and the car; caption "take the car". | sounds 0x977D2 ... | L |
| 89 | 0x8C982 | 41 | CTL | class 88 | E3 | Car section helper object. | none | S |
| 90 | 0x8CAE0 | 87 | TXT | class 89 | E3 | Caption of the car section. | none | S |
| 91 | 0x915FA | 214 | FE | story code | story panel | Story panel text (tall font), G.$44A, G.$6C4. | none | M |
| 92 | 0x91CF2 | 89 | FE | not found (front end code) | story, continue | Portrait box with a hero or villain face (scroll1 blocks at 0x90A00C, 0x90A01C ...); sound 0x8A. | sound 0x8A | M |
| 93 | 0x91FB4 | 110 | FE | profile card code (0x9A984..) | profiles | Profile card checker board background (scroll2/3 0x90FFE0, 0x9107E0 ..). | none | M |
| 94 | 0x921DA | 13 | FE | profile card code | profiles | Profile card piece (G.$7E8). | none | S |
| 95 | 0x92220 | 49 | ANI | script L1 | E3 a0 (7) | Sky colour blender: moves 3 palettes (16 colours) one RGB step at a time toward a target set (tables at 0x922D0, speed bytes 0x922C8), so the sky changes colour along the road; records at X 0x1200, 0x2500, 0x3100, 0x3300, 0x3700, 0x3A00, 0x3D00 (var 0 to 0xC). | none | S |
| 96 | 0x92382 | 24 | CTL | area start hook 0x9236C | every area | Area music starter: sends the music fade out (0xFF07), waits 60 frames, then plays the track from table 0xA582 (stage * 4 + area). | music code | S |

### 2.3 Spawn chains worth knowing

- Area start hook 0x9236C: class 96 (music) every area. Respawn: class 18, then pool E 8 (bombs) and pool E 1 (explosions). Radio: 0x8BD24 makes three class 88.
- Class 15 (wave) creates the pool B records of waves (door bursts, wall breaks) and pool A enemies; class 40 creates pool A class 15 steam pieces (0x41EEA); class 45 creates pool A class 83 flames (0x6152C); class 44 creates a mutant; class 72 creates pool A class 46 birds; class 46 creates pool A class 48 (trail sparks); class 74 creates the opening cast (A54, A55, A56); class 33 creates the Slisaurs (A71); classes 83 to 85 create debris pieces (A92, A93); class 65 and the boss code of A18 and A43 create caption banners (class 66); class 49 and the bosses A5, A18, A25 create dialogue boxes (class 50).
- Scripted control actions: 0x40 makes class 11, 0x46 class 20 var 2, 0x24 class 21 (see 2.4).

### 2.4 Scripted control actions that touch pool B (what D4c must finish)

The area end walks of Episode 1 are scripts of the scripted control (`0x1DB5E`, the script pointers at 0x1E6FC, 12 scripts at 0x1E768 to 0x1E9F0). A record is 6 bytes: action id (index into the table 0x1DBD6), G.$4F0, G.$4FD, G.$506, duration word. The record flags are copied when a record loads, so a flag lives until the next record. [R 0x1DB5E to 0x1DBC4]

| Action | Routine | Effect | Seen in |
| --- | --- | --- | --- |
| 0x40 | 0x1DD30 then 0x789C8 | smash the door: spawns class 11 (see 3.2) | script 2, record 12 (the area 0 end of Episode 1 [M e1a0-door]) |
| 0x46 | 0x1DD0E | spawns class 20 var 2 (forced GO blink) | scripts 0, 2, 5, 7 |
| 0x24 | 0x1DECC then 0x7BDAE | spawns class 21 (tile attribute change) | script 4 |
| 0x22 | 0x1DED4 then 0x86D2E | calls the caption helper with G.$4FD as the number | script 5 (G.$4FD = 0x1B) |
| 0x28 | 0x1DEA0 | sets G.$4D8 | scripts 1 to 7 |
| flag G.$506 = 1 | | arms the window of class 12 var 4 and probably other "hit by the scripted hero" objects | scripts 1 to 7 |

Which script an area end uses is set by the area walk code 0x1D95E (D4c). In MAME the hall end of Episode 1 shows the hero walking to the window, a flying kick, shards, and the next area starting with the hero falling [M e1a1-hall].

### 2.5 The compositions that no animation table draws

Method: for every unreferenced composition number (319, byte offset = number x 4) I searched the program ROM for the byte offset as an immediate or as a word in a data table with a plausible neighbour, looked at the classes whose code or data holds the hit, and spawned all 97 pool B classes in MAME (`re/out/probe/d3e-b2`). [R, M sp00 to sp96]

| Composition numbers | What it shows | Drawn by | Evidence |
| --- | --- | --- | --- |
| 326, 327 | chain-link fence whole, fence broken | **pool B class 1** | frame lists at 0x75400 and 0x7540A hold 0x518 and 0x51C; spawn test sp01 shows the fence. |
| 3299 | fence with a sign | not found | no word or immediate; probably an unused variant of class 1 |
| 1897 to 1900 | red pillars (lift shaft) | **pool B class 64** | frame data at 0x86BE4 to 0x86C08 |
| 2237 to 2239 | 1P, 2P, 3P tags | **pool B class 68** | frame data at 0x88A60, 0x88A6C, 0x88A78; the tag follows the join table G.$862C |
| 2366 | golden muscular man | **pool B class 67** | frame at 0x888A6; class 67 is spawned by the select screen (table 0xAEFC) |
| 2895 to 2903 | "STAGE CLEARED!" and the digits 1 to 8 | **pool C class 9** (0x93184) | 8 frame records at 0x93282 to 0x932CA: sign 0x2D3C with the digit 0x2D40 + 4n as overlay, 1 frame each; the effect slides off to the left by 12 pixels per frame when G.$6C4 is not 0. D3f owns pool C. |
| 1607, 1608 | "Look out!" bubble, red "BREAK!" banner with arrow | none found | no immediate, no table word, no spawn test shows them. Treat as unused art. |
| 2911 to 2983 | spiked ball turning, 71 frames | probably pool A class 62 (spiked mace on a chain, 0x5A47C) with a computed index [I] | no direct reference; the block sits right after A62's art |
| 3296, 3297, 3298 | two black shapes and a round grate (the "pipe pieces" of the plan) | not found | they lie between effect C21 art; probably shadow or debris parts |
| 3300, 3350, 3356 | manhole, two chains | not found | the chain on the Vice leash in area 2 is part of Vice's raptor code (A29, D3b) [I] |
| 1880 | round wooden lid | not found | |
| 268 to 278, 638 to 770, 1278 to 1295, 3187 to 3258, 3412 to 3481 | actor frames (thugs, heavy man, bound blond man, falling bodies) | none found | they lie inside the art blocks of Ferris, Mess, Walther and others: unused frames of those owners (D3a, D3b) |

**The pipes of the plan's list are not compositions.** The pipes the user saw are scroll2 tiles (class 40), and the steam is pool A class 15 whose frame table (0x4200E) is a referenced animation. No pipe sprite composition exists.

## 3. The classes met in Episodes 1 and 2, in detail

### 3.1 Class 40: the steam pipes (W25)

- **Where:** Episode 1 area 0, script list 1 record at trigger 0x460: map X 0x620, depth Z 0x130 (304), var 0. Episode 8 uses the class four times (area 0 var 4 at X 0x690, area 1 vars 8, 0xC, 0x10). [R stage_scripts.txt]
- **Look:** two vertical gray pipes with a horizontal arm at the top and an elbow near the ground, in the end wall of the roof, to the left of the metal door (the "pipes in walls" the user remembers). [M e1-pipebreak]
- **Init** (0x836B8): routine 0 sets `+$48 = 1`, `+$49 = 0`, `+$44` = box table (0x8380A when the variant kind byte is 0, 0x83822 when it is not; the kind byte comes from the table 0x83786 indexed by var / 4 and is 0, 0, 1, 1, 0 for var 0, 4, 8, 0xC, 0x10), HP `+$6C = +$6E = 8`. If the camera X + 0x20 is already beyond the object X it frees itself (0x83804).
- **Idle:** routine 2 registers in the player hit list every frame (0x4BFA). Box var 0: depth [Z, Z + 16], X [X + 0x10, X + 0x30] (mirrored when the kind is 1), height 56 to 96 pixels. A normal punch from a hero standing at X 0x5F0 hits it when the hero's depth is **296 to 320** (tested at 8 pixel steps: 288 and 328 miss, 296 to 320 hit; Jack, one punch). It breaks on the first hit, within 10 frames of the punch the state is 0x04020000. [M e1-pipez272 .. e1-pipez344]
- **Break** (routine 4, sub 0, 0x83732): rewrites an 11 tile patch of scroll2 at the object (3 columns: 3, 4 and 4 tiles; address table 0x8378C by var / 4, column step table 0x837A0, tile codes 12 words from 0x837E8: the upper arm and top of the pipes disappear), then spawns effect C21 variant 8 (kind 0) or 0x28 (kind 1) through 0x94640. There is no break sound in this class (only the generic hit sound of the hit pass). [R 0x837AA]
- **Steam for ever** (routine 4, sub 2, 0x8375A): every 6 frames (counter `+$A7` = 5) it calls 0x41EEA twice: pool A class 15 pieces of kind 0 and kind 2. Row table 0x42050: kind 0 starts 28 pixels right of the object and 64 pixels up and moves (-3, +1) pixels per frame; kind 2 starts 20 pixels right and 64 up and moves (-1, +3). A piece lives as long as its animation (about 18 frames), so **3 pairs (6 pieces) are alive at any time, as long as the object exists**. The object has no end: it stays broken and steaming until the camera passes it. The pieces are white puffs: the pipes leak steam. [M e1-pipebreak: 6 class 15 objects at every sample from t = 45 to 1290]
- **Port:** pieces are ported already (`enemy/class15.rs`, `enemy_spawn_piece`); the prop, its hit pass 0x10CD0 and the tile patch are not. Size M.

### 3.2 Class 11: the smashed door (end of area 0)

- Spawned by the scripted control action 0x40 (0x1DD30: clear the walk flags 0x1DEA6, then jump to 0x789C8 which allocates the object). The object sits at map X 0x670, Z 0x118 (280). [M e1a0-door: first seen at t = 980]
- Routine 0: HP 1, box index 1, waits 3 frames. Routine 2: break sound with pan (0x9E60), rewrites the door tiles at scroll2 0x90E9E4 (14 tile writes from 0x78A8E) and changes attribute bits of 8 + 56 more tiles at 0x90EA66, effect C21 var 2 (dust). Then it frees itself.
- Seen: the hero kicks the metal door, planks of dust fly, the doorway is black, the hero walks in; the area ends with the fade of camera event 5 (X 0x580) at about t = 1100. [M e1a0-door pictures at t = 990 to 1070]
- Size S.

### 3.3 Class 12: hall door (var 0) and hall window (var 4)

- **var 0** (wave 1 of Episode 1, delay 0xC0 after the first record, X = Z = 0): 3 pictures of a 9 x 9 tile block at scroll2 0x90F288 (tile data 0x78BE6, 0x78D2A, 0x78E6E), one every 4 frames; the double door of the hall swings open and leaves a black doorway. Then the routine frees itself. The Punk or Thug of wave 1 (entrance 8, X 0xE8) jumps out of it. [M e1a1-hall t = 160 to 200]
- **var 4** (script list 1, trigger 0x1E0, map X 0x370, Z 0x40, Y 0x30): the window at the right end of the hall (red curtains, panes, next to the stairs). Routine 0 sets HP 8, box (depth 16..96, X 880..928, height -48..88), then waits for **G.$506 != 0**. After that it registers in the hit list (0x4BFA). On a hit (routine 4): writes 3 columns x 7 tiles at 0x90FDCC (shattered glass), sets attribute 0x10 on 12 tiles at 0x90FE16, moves itself to (0x38A, Y 0x60, Z 0x30) and calls 0x93362 ten times (d1 = 0x28, table 0x79152: ten glass shards), sound 0x37 (parameters 0x72020 and 0x10000000), waits 20 frames and frees itself. [R 0x78FB8 to 0x791F2]
- **Seen:** at the end of area 1 the scripted walk takes the hero to the window and he kicks through it (G.$506 is set by the script record); the glass breaks at about t = 1780 to 1790 and area 2 begins with the hero falling outside the building. A normal hero cannot break it earlier: before the flag the window does not register. [M e1a1-hall, e1a2-enter]
- Size M.

### 3.4 Class 14: building front (var 0) and wooden door (var 4)

- **var 0** (list 1, trigger 0, map X 0x60, Z 0x120, Y 0x290, the first object of area 2): after 8 frames it writes 4 columns x 7 tiles at scroll2 0x90C10C (the window frame the hero fell out of becomes a hole) and spawns effect C21 var 4 facing 1. This is the "after" picture of the window of class 12 (continuity between the two areas). [M e1a2-enter]
- **var 4** (wave 4, delay 0x50 after the Hammer record): moves to (0x40, 0x10, 0x120), writes 4 columns x 9 tiles at 0x90E110 (the wooden double door of the building bursts open: planks fly and the doorway is black), effect C21 var 6, sound 0x88 (parameters 0x32000, 0x1C000000). [M e1a2-enter at t = 2010 to 2060]
- Size S.

### 3.5 Class 18: the respawn air strike

- Created by the player respawn code (0x18BCE: after the 0x168 frame invulnerability counter `+$EC` is set, `jsr 0x65A4` makes the speech bubble, `bra 0x18C5E` creates class 18 with the player index). [R 0x18B9C to 0x18C5C, M respawn; a continue uses the same player respawn state, not tested]
- It reads the list at 0x7B7A6 (5 records of three words, X offset, depth offset, var: (320, 64, 0), (192, 32, 0), (64, 48, 0), (128, 0, 0), (256, 16, 1), end word negative) and creates one pool E class 8 object (a bomb, `+$26` = var; the fifth is var 1) every 10 frames, falling from the top of the screen in a diagonal line, then a pool E class 1 (explosion var 2) on the ground; a white oval blast shows at the landing and fire surrounds the hero. The speech bubble is the hero's angry threat. [M respawn: B#17 class 18 seen at t = 280 to 320, pool E class 8 objects with y falling from 278 by 10 per sample, pool E class 1 var 2 at t = 340]
- The damage of the bombs and the blast to enemies is not measured here (D3f, pool E class 8 and 1 belong to D4a and D3f).
- Size M. This is **not** a 1UP marker: correct P40 and P22 of `mechanics-seen.md`.

### 3.6 Class 66: caption banners

- States: routine 0 init (copies the variant data: `+$A0` start delay from the word table 0x86EB4, `+$A2` duration from 0x86DD8, `+$A8` kind byte from 0x875F2, `+$AA` from 0x87652); routine 2 sub 0 waits `+$A0`, sub 2 prints the string (var x 4 into the pointer table 0x87710) with the tall font printer 0x84F72 onto scroll1, sub 4 waits `+$A2` frames, sub 6 clears the text block (0x871FC at scroll1 0x90A068) and frees. [R 0x86D76 to 0x87220]
- Episode 1 and 2 records and what they show (my paraphrase of the captions): var 9 (opening, 180 frames): the place and the year, the east coast and the year 2513; vars 0x49, 0x54, 0x4A at trigger 0x2A0 (delays 0, 122, 244, 120 frames each): the three lines of the boss speech in the opening scene; var 0xE (trigger 0x390): top of the building; var 0xF (area 1 start): in the building; var 0x10 (area 2, delay 80, 240 frames): 47th street; Episode 2: var 0xD (area 1 trigger 0x720): mud swamp; vars 0x27 (80 frames) and 0x56 (delay 82, 120 frames) at camera X 8 of area 2: one of them is a line about the many dead bodies. [R tables, M e1-open, e1a0-mid, e2n-mid]
- Anti tamper: the first run calls 0x86B3C which scans the interrupt code 0x542 to 0x60C for the opcodes 0x6000 and 0x4E71 (a patch); with a patch and a 1 in 128 chance per frame the object becomes class 87 (the staff roll). Real ROM: never. Ignore in the port (E26 of `mechanics-seen.md`).
- Size M (92 strings are data; the text printer belongs to the front end milestone).

### 3.7 Class 74: master of the Episode 1 opening scene

- Trigger 0x1C0 (about t = 160 in the opening). Routine sub steps by `+$05`: 0 creates the cast: pool A class 54 at (0x4C0, Z 0x130), class 56 at (0x4E0, Y 0x60, Z 0x131), class 55 var 0, 1, 2 at (0x490, Z 0x130), (0x4A0, Z 0x14E), (0x4A0, Z 0x120) and sets the first timer 0xF0 + 0x78; sub 2 creates the second group of class 55 var 3, 4, 5 (facing left) at X 0x370, 0x360, 0x370 (Z 0x130, 0x14E, 0x120) after 0x5A frames; the later subs wait 0x1E frames and so on. The cast holds the master handle in `+$A4`. [R 0x89B48 to 0x89DA0]
- Not ported (`prop.rs` frees it). The cast classes 54, 55 are ported, 56 and 57 are not. Size M.

### 3.8 Classes of Episode 2

- **Class 2** (area 1, trigger 0x5C0): swamp water. Routine 2 runs every 24 frames: advances the phase word `+$A4` (0, 2, 4, 6) which it also stores in G.-$703C, skips the records of the 12-byte list at 0x7550E (X limit word, pointer to the source tile frames, pointer to the target tile block) whose limit is left of the camera X - 0x30, then rewrites up to 13 tile rectangles from the first record that is still ahead. The changing region in MAME is the bottom 4 tile rows of the screen over the full width (the water in front of the heroes). [R 0x7542C, M e2a1-water2 diff map]
- **Class 7 var 4** (area 1, trigger 0x700, X 0x500): 2 far layer tiles (scroll3 0x913D04 and the next row at +0x20) cycle through 6 pictures, 24 frames each, then a pause of 96 frames; it stops when the object X + 0x40 is left of the camera reference G.$7E4. A small distant animation (about 32 x 64 pixels). I could not separate it in a screenshot [I: far waterfall or fire].
- **Class 27** (area 2, trigger 0, Y 0x100): 4 pictures, 24 frames each; two regions: 9 rows at scroll2 0x90E03C, a 313 tile write block at 0x90F000. Runs while the camera is less than 0x1A0 right of its X. The region is not on the first screen; I could not see it in 200 frames from the area start [I: the hatched green pit at the lower left of the map].
- **Class 72** (area 0, trigger 0x160 var 0, trigger 0x280 var 0x200): waits 8 frames between birds and spawns 5 pool A class 46 (var 0) or 7 (var 0x200; var 0x400 would give 11); each at camera X + 0xB0 + random (0..0x3C), Y at the camera top + 0x40, with palette 0x98. The birds are small blue and white flyers that dart left through the foreground at the bottom of the screen with a spark trail (class 46 creates pool A class 48 trail pieces). **They are harmless**: no hit list registration in 46, 48, 78, 80, 49, 50 (static search), and Jack, standing still in the first flock (t = 80 to 120), kept 100 HP (a later loss of 22 HP at t = 250 happened while the triceratops, class 38, was on screen). [M e2-birds2: first flock at t = 80, second at t = 620 in e2a0-mid]
- **Class 80** (area 0, trigger 0): ambient forest sound maker: every 120 frames it draws a random number; with chance 4 in 16 it draws a second one and with an even number it plays sound 0x78 (the odd branch loads code 0x61 but never sends it). About once per 960 frames. [R 0x8A582]
- **Class 96** (every area, spawned by the area start hook 0x9236C): the area music starter. Sends the fade out request 0xFF07 (0x44400000), waits 60 frames, then sends the music code `byte[0xA582 + stage * 4 + area]`. In Episode 2 area 0, class 96 is seen at the area start (t = 2480 of e2a0-mid and again at the swamp start). [R 0x92382, M e2a0-mid]
- **Pool A small animals** of area 0 are scenery critters (A78 at Z 0x20 to 0x30 and A80 at Z 0x50, spawned right after the first Mack at triggers 0x188 to 0x198): tiny lizards on the ground; no hit box. Details belong to D3b.

### 3.9 Pool E containers seen in Episodes 1 and 2 (variants of E class 5, HP 4)

From `spawn E 5 var=N` in MAME: var 0 small red pail, var 2 orange oil drum, var 4 wooden barrel, var 6 gray stone chunk, var 8 tan boulder, var 0xA golden armor statue (hall of Episode 1), var 0xC stack of black tires. Statues break like any container (state 4, HP 0 after a hit, +500 points and a drop roll; solid, not liftable, so they block walking). [M cont0 .. cont12, e1a1-hall: the statue at X 331 reached HP 0 and state 4] Drops: `re/specs/enemy/SPEC.md` 13.2 and 13.3 (D4a).

### 3.10 Class 69 (debug), class 20 (hint), class 31 (clock)

- **Class 69**: frees itself at once (`tst.b $4AA(a5)`). With the debug flag G.$4AA on, it is a free fly cursor (the pad moves X and Z, buttons change Y), a secret input code (table 0x88BBE, held 0x3C frames per step) activates it, the player objects move with it, it can spawn objects from the table 0x88DDA (26 bytes each), and it edits G.$5EC and G.$5EE (aggression and level) and shows X and Z on scroll1. Not needed for play; port as a free-at-once stub. [R 0x88A84]
- **Class 20** (hint) and **class 31** (clock): described in the table; D4c and D4b own the numbers. Measured: the hint blink and the sound 0x3E appear 600 frames after the camera stopped (opening: first blink at about t = 820 to 860 after the walk-in) [M e1-open].

## 4. Area walk, Episode 1 (internal stage 0)

**Method.** For every area: `jump 1 N` (first frame of control), `god on`, `hold 0xFF850E 1 2` (kill all, so waves end), and a sweep walker: 24 frames walking right, then 30 frames standing and punching (3 punches), repeated, for 2400 to 3200 frames. Four passes per area: at the back wall (`U` held), middle, front (`D` held) and a jump kick pass. A `watch` of all pool B objects (class, variant, state, X, Y, Z every 20 frames) and a screenshot every 100 frames (`d3e-b1`, `d3e-b3`). Then targeted tests by `poke` of the hero position. A contact sheet of each run was read. What the sweeps could not reach is said in the open items. Positions are map X, depth Z, height Y in pixels.

### 4.0 Opening scene (area 0, camera mode 0x11: auto scroll 3 pixels per frame, X 0 to 0x380) [M e1-open, 1000 frames]

| What | Where, trigger | How it runs | Shows | Class or routine |
| --- | --- | --- | --- | --- |
| Place caption | list 1 trigger 0x40 | starts at t = 40, 180 frames | small white text the east coast and the year 2513 over the sea | B66 var 9 |
| Sky birds | list 2 triggers 0, 0x40, 0x260 and list 1 trigger 0x100 | red pterodactyls cross the sky left to right at t = 20 to 130 and again later; spawner makes a bird every 0x100 to 0x2FF frames (variants 0, 2, 4, 6, 8) | no hit box, harmless | A49 (3 controllers), A50, B73 (ported) |
| Opening cast | list 1 trigger 0x1C0 (t about 160) | the master creates the bystander and 6 thugs, they run in from the right; the hero walks in; three caption lines of the boss at t about 240 to 600 (the boss says he is tired of the interference and that his men will teach the hero a lesson, paraphrased); the hero beats the thugs by script (flying bodies, hit sparks) | HUD bars slide in at about t = 640 to 700 | B74, A54, A55, A56, B66 vars 0x49, 0x54, 0x4A (delays 0, 122, 244) |
| Clock and hint | list 1 trigger 0 | clock hidden until 3:00; hint blink 600 frames after the camera stops (first blink at t about 820 to 860, sound 0x3E) | red GO burst at the right edge | B31, B20 |
| Debug object | list 1 trigger 0 | frees itself | nothing | B69 |

### 4.1 Area 0: the roof tops (map 0x700 wide, camera events 0x380, 0x408, 0x460, lock 0x4D0, end fade 0x580)

| What | Where | Trigger, how | Before and after | Drops | Effect on players or enemies | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Steam pipes** | X 0x620, Z 0x130 (Y 56 to 96 box) | any player attack box with hero Z 296 to 320 (Jack's punch tested) | two gray pipes with an arm become two broken stubs; white steam puffs rise from them for ever | nothing (no drop) | none; the puffs are no attack objects (pieces use list 0x671E only when their flag byte is set, no row sets it) | B40 (3.1), A15 pieces | M: broke in the mid pass only; Z test |
| **Metal end door** | X 0x670, Z 0x118 | scripted walk after wave 0 (script 2, action 0x40): the hero kicks it | closed gray door becomes a black doorway with dust; the hero walks in and the area fades out | none | scripted hero, enemies are dead | B11 (3.2) | M e1a0-door t = 980 to 1100 |
| Oil drums | var 2 at X 0x5A0 and 0x5D0, Z 0x150 and 0x150; with 2 players a pail (var 0) at X 0x5D4, Z 0x142; with 3 players a pail at X 0x5F6 | triggers 0x410 and 0x450 (0x460 for 3P), HP 4, punch breaks, +500 | orange drum, breaks into debris | rows 0x02, 0x2A, 0x12, 0x04 (D4a) | can be lifted, thrown and used as a weapon | E5 (3.9) | M sheets: two drums at the wall |
| Roof edge, stairs, vent pipes, rooftop city, sea | map | none: the roof is flat floor (camera event 2 slopes the camera down at 0x408) | static | | the roof edge has no fall | stage map | M: no change in any pass |
| Wave 0 | lock at 0x4D0 | the second Ferris only with 2 or more players; end type 2 = walk to the door | | | | B15 | M |

Scenery animation: none besides the birds and the steam; the sea and the buildings are static in all four passes. [M diff maps of the idle runs i1a0 and the sweeps]

### 4.2 Area 1: the hall (map 0x400 wide, locks at 0 and 0x1B0, area change without fade at 0x220)

| What | Where | Trigger, how | Before and after | Drops | Effect | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Hall double door** | scroll2 block at 0x90F288 (the golden door at the left, behind the first statue pair) | wave 1 starts it 0xC0 frames after the first record | closed door opens in 3 pictures of 4 frames; black doorway; a thug (class 20 entrance 8) jumps out | | gives wave 1 its entrance | B12 var 0 (3.3) | M e1a1-hall t = 160 to 200 |
| **Golden armor statues** | var 0xA at X 0x81 and 0x14B, Z 0x48 (triggers 0, 0), X 0x280 (0xC0), X 0x342 (0x182) | punches at hero Z within 8 of 72; HP 4; solid, not liftable | statue stands, then breaks and disappears | rows 0x0C, 0x27, 0x01, 0x15 | blocks walking until broken; +500 points | E5 var 0xA | M: statue at X 331 reached HP 0, state 4 |
| **Hall window** | X 0x370 (right end), Z 0x40, Y 0x30 | armed by the scripted walk (G.$506); the hero kicks it at about t = 1780 | lit window with red curtains becomes shattered glass, ten shards, sound 0x37; the hero jumps out | | scripted | B12 var 4 (3.3) | M e1a1-hall t = 1764 to 1796 |
| Wall sconces, cobwebs, stairs with banister | map | none | static | | | stage map | M: no change in 4 passes |
| Waves 1, 2, 3 | lock 0 (wave 1), lock 0x1B0 (wave 2), 0x220 (wave 3 = the walk to the window) | | | | | B15 | M |

Scenery animation: none. [M idle run i1a1]

### 4.3 Area 2: the street (vertical start, camera mode 7 from Y 0x300, locks (0, 0x100) and (0xA0, 0x100), stage end at 0x1D0)

| What | Where | Trigger, how | Before and after | Drops | Effect | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Hole in the building front** | X 0x60, Y 0x290, Z 0x120 (top left of the facade) | list 1 trigger 0, 8 frames after the area starts | the intact window the hero left becomes a dark hole with glass dust (effect var 4) | | none | B14 var 0 (3.4) | M e1a2-enter t = 1790 to 1880 |
| **Fall from the window** | camera mode 7 follows the players down 0x300 pixels | scripted entry | the hero falls past windows, lands on the street at about t = 1890; caption 47th street at t about 1800 to 2000 | | | camera, B66 var 0x10 | M |
| **Wooden double door** | X 0x40, Z 0x120, scroll2 0x90E110, building front at the left | wave 4 record (delay 0x50 after the first record) | door bursts open (planks fly), black doorway, sound 0x88; Hammer and Ferris come out | | gives wave 4 its entrance | B14 var 4 (3.4) | M e1a2-enter t = 2010 to 2060 |
| Containers | (0x190, Z 0x13F) var 2 (3 players), (0x1D0, Z 0x110) var 0xC (2 or more), (0x1F0, Z 0x13F) var 0 | trigger 0 and 0x30 | drum, tire stack, pail | rows 0x24, 0x0F, 0x35 | | E5 | R |
| Torn chain-link fence, brick ruin, grass | map X about 0x200 to 0x2C0 | none: static map tiles | static | | the fence is NOT class 1; it is part of the map | stage map | M: not hit by any pass; no object |
| Vice T. and the leashed raptor | trigger 0xF4 (wave 5 spawns a Ferris, the boss area follows) | boss intro: dialogue boxes (class 50), captions, boss music, chained raptor | | | D3c, D3b | A18, B50, B66, B76, B22 | M e1a2 |

Scenery animation: none. [M idle run i1a2]

## 5. Area walk, Episode 2 (internal stage 1)

Same method as section 4 (`jump 2 N`, sweeps of 3000 to 7000 frames in four passes, `d3e-b3`, `d3e-b4`, `d3e-b5`, `d3e-b6`). Area 2 cannot be entered by `jump` with working diagonal motion (probe limit), so it was walked after the natural end of area 1 (`e2n-*`; the camera needs about 600 frames of plain walking right and down to leave the diagonal, `e2n-walk`).

### 5.0 Opening (area 0 from `jump 2 0 at=start`, 580 frames to control) [M e2-open]

| What | How | Class or routine |
| --- | --- | --- |
| Map panel and card | the map panel (a coloured map of a lake and mountains) shows at the top from t = 20, then the text "Episode 2" and the title typed in, panel leaves at about t = 240 | flow code 0xB9BE and class 58 [I] |
| Cadillac arrival | the blue open car drives in from the left from t = 280, the hero jumps out of it at about t = 340 and runs, control at about t = 575 | pool A class 16 (the car) with class 17 parts, list 2 record at X -0x80 |
| Area start | the music object (class 96), clock, hint and the ambient sound maker (class 80) are created | B96, B31, B20, B80 |

### 5.1 Area 0: forest edge (camera follow 0 to 0x568, then the sub area switch)

| What | Where | Trigger, how | Shows | Drops | Effect on players or enemies | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Treasure behind tree trunks | E class 9: var 2 at X 0x1A8, var 8 at X 0x1A8 (trigger 0x428), var 0xE at X 0x30A, var 0xD at X 0x468, all Z 0x10, height 64 | placed at the trigger; the big yellow trunk covers it at X 0x1A8 | not visible when the hero stands to the side; picked up like any treasure | the treasure id = var (D4a) | points | E9 | M e2-treasure (no sprite visible at screen X 88 behind the trunk), e2a0 logs |
| Barrels | var 4 (wooden barrel) at X 0x360 and 0x3A0, Z 0x50, triggers 0x1C0 and 0x200 | punch HP 4 | | rows 0x0B and 0x06 | liftable | E5 | M sheets |
| Bird flocks | trigger 0x160 var 0 (5 birds), trigger 0x280 var 0x200 (7) | | small blue flyers cross the foreground | | harmless | B72, A46, A48 | M e2-birds2 |
| Jungle sound | from the start | | | | | B80 | R |
| Small ground critters | after the first Mack, triggers 0x188 to 0x198 | | tiny lizards | | harmless | A78, A80 | M |
| Mack (triceratops) | triggers 0x180, 0x280, leave X 0x480 | | | | hurts | A38 (D3b) | M |
| Sleeping Shivat | trigger 0x480, X 0x678, leave X 0x569 | | green T. rex asleep with Z marks | | | A24 (D3b) | M e2a0 sheets |
| Clock time events | trigger 0x150 sets 2:50, 0x170 adds 0:30 | | | | | script commands | R |
| Sub area switch | camera event at 0x568 (mode 8) | wave 2 ends with the walk (end type 2) | the camera moves into the swamp without a reload (area 1) | | | camera, B15 | M area change at t = 2480 |

Scenery animation: none seen in the forest (trees, leaves static); no pool B animation class in this area. [M idle run i2a0]
Waves 0 and 1 of the stage have **no script record that starts them** (and wave 4 neither): the spawn lists of Episode 2 start only waves 2 (area 0), 3 and 5 (area 1), 6 (area 2). Waves 0, 1 and 4 are unused leftovers. Wave 4 is the only wave with **end type 0xE**: the wave controller tests `end & 0xE` (0 = GO, 2 = walk to the next area, anything else = stage exit 0x1DAC4), so 0xE would act like type 4 (stage exit); since no record starts wave 4 it never runs. [R stage_scripts.txt, spec 8.2]

### 5.2 Area 1: the mud swamp (creep camera at 0x620, follow 0x7C0, lock 0x860, slope up 0xA00, end fade 0xB80)

| What | Where | Trigger, how | Shows | Drops | Effect | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Water surface ripples** | bottom 4 tile rows of the screen over the whole width | object created at trigger 0x5C0; 4 pictures, one every 24 frames | the water in front of the heroes shimmers; heroes wade (sprites cut at the water line, splash when they land) | | slows nothing (the wading draw only) | B2 | M e2a1-water2 diff map |
| Far distance animation | far layer tiles (32 x 64 pixels) | trigger 0x700 (X 0x500), 6 pictures x 24 frames, pause 96 | small distant detail | | | B7 var 4 | R; picture not separated |
| Caption mud swamp | trigger 0x720 | 120 frames | | | | B66 var 0xD | M |
| Barrel | var 4 at X 0xCD8, Z 0x40, trigger 0xB38 | | | row 0x1B | | E5 | M e2n: broken at t = 3320 |
| Poachers in water | | wave 3 at 0x860, wave 5 at 0xB80 (Lash T., Blade, 3 poachers) | | | | B15 | M |
| Mud bank, tree roots, mist | map | none | static | | | stage map | M |

### 5.3 Area 2: the Butcher's yard (diagonal descent at X 0, follow 0x118, lock 0x180, follow 0x200, stage end 0x260)

| What | Where | Trigger, how | Shows | Drops | Effect | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Captions | trigger 8 (camera X 8) | var 0x27 (80 frames) and var 0x56 (delay 82, 120 frames); one of them is a line about the many dead bodies, seen at t = 4100 to 4300 of e2n-mid | | | | B66 | M e2n-mid |
| Pit animation | lower left of the map [I] | trigger 0, 4 pictures x 24 frames while the camera is less than 0x1A0 right of X 0 | | | | B27 | R |
| Tire stacks | var 0xC at (0xA0, Z 0x188) and (0xF8, Z 0xE0) | trigger 0 | black stacks at the top left | rows 0x09, 0x27 | | E5 | M (visible), break not tested |
| Oil drums | var 2 at X 0x210, 0x248, 0x280, Z 0x90 and 0x28: two rows of three orange drums at the lock | triggers 0x70, 0xA8, 0xE0 | | rows 0x29, 0x29, 0x19, 0x03, 0x2B, 0x29 | liftable | E5 | M e2n-walk t = 6100 to 6500 |
| Boss intro | wave 6 at the lock 0x180 (3 poachers, then time 3:50, end type 4); A5 at (0x360, Z 0x40), trigger 0x1A0 | forced hint at t = 6280; dialogue boxes with portraits at t = 6640 to 6820 (the hero is disgusted and the boss answers with an insult); the Butcher (red vest, machete) stands in front of a skinned dinosaur body; camera mode 6 (stage end) at X 0x260 | | | D3c | A5, B50 var 1, B76, B22 (t = 6940) | M e2n-walk t = 6080 to 7000 |
| Slaughter house with thatched roof, ramp, triceratops carcass on the map | map X 0x1A0 to 0x300 and 0x300 to 0x600 | none: static map tiles | static | | | stage map | M |

Scenery animation: none seen on the first screens; the lower left hatched pit may animate (class 27).

## 6. Answer to W25 and W26 for Episodes 1 and 2

**W25 (breakable parts of the background).**
- Episode 1 area 0: the steam pipes (B40) are the thing the user saw. They are a pool B prop that registers in the player hit list, with a tile patch and a never ending steam of pool A class 15 pieces. [M]
- Episode 1 area 1: the golden armor statues are pool E containers (E5 var 0xA), not background tiles; the window (B12 var 4) is a prop that only the scripted walk can break. [M]
- Episode 2: **nothing in pool B can be broken by a player.** Only barrels, tire stacks, drums and hidden treasures (pool E). [R, M]
- The rest of the family (classes 0, 1, 3, 45) lives in Episodes 4, 6, 7 and 8 and is for part 2: class 0 is a generic panel with 15 variants (3 used in Episode 4), class 1 a fence, class 3 a wall, class 45 a flame vent that can be destroyed.

**W26 (every other interactive thing), Episodes 1 and 2:**
- Episode 1: end door smashed (B11), hall door opening (B12 var 0), hall window (B12 var 4), building front hole (B14 var 0), wooden door burst (B14 var 4), steam pipes, armor statues, drums, pails, tire stack, the respawn air strike (B18), the opening cast (B74), captions (B66), the GO hint (B20), the clock (B31), the music object (B96).
- Episode 2: water ripples (B2), far animation (B7), pit animation (B27), bird flocks (B72), critters (A78, A80), jungle sound (B80), barrels, tire stacks, drums, treasures behind trunks, car arrival (A16), sleeping and walking dinosaurs (A24, A38), the sub area switch.
- Nothing else reacts to the players in these areas: no switch, no lift, no sign, no falling object other than the scripted ones above, and no pool B hazard. [R scripts; M four sweep passes in every area except Episode 2 area 2, where the sweep walker was slow and only the plain walk `e2n-walk` covered the whole area without punching (open item 5)]

## 7. Acceptance scenarios (probe segments and what must match)

All segments run with `PR_INPUT=file re/harness/probe/probe.sh NAME` (one MAME process for all). Frame numbers are `t` after the jump. The **sweep walker** used by A4, A5, A8: for k = 0, 1, 2 ...: keys `R` (plus `U` or `D` for the back and front passes) from `54k` to `54k + 23`, then `A` pulses of 3 frames at `54k + 24`, `54k + 34`, `54k + 44`; with `god on` and `hold 0xFF850E 1 2`. The port compares against the numbers below (class, state long, X, Y, Z of pool B slots; counts of pool A and E objects; tile words).

| # | Segment (core lines) | What must match |
| --- | --- | --- |
| A1 | `jump 1 0 0x4a0`; `god on`; `2 poke 0xFFB27C 2 0x5F0`; `2 poke 0xFFB284 2 Z`; `20-22:A`; `frames 120`; `watch 10 B:40 st x y z`; Z in 272, 280, ... 344 | the prop (X 1568, Z 304) goes from state 0x02000000 to 0x04020000 within 10 frames of the punch for Z = 296, 304, 312, 320 and stays 0x02000000 for 272 to 288 and 328 to 344 |
| A2 | as A1 with Z 300, then `peek 0x90E8A0 16`, `peek 0x90E8DC 16`, `peek 0x90E91C 16` at t = 3 and t = 80 | exactly 11 tile code words change (3 at 0x90E8A0, 4 at 0x90E8DC, 4 at 0x90E91C); the attribute words stay |
| A3 | A1 with Z 300, `frames 1300`, `watch 15 A:15 x y z st var` | at t = 30 pieces at (1582, 82, 304) and (1578, 70, 304); from t = 45 on exactly 6 class 15 objects (3 var 0 and 3 var 2) at every sample until t = 1290; pipe state stays 0x04020000 |
| A4 | `jump 1 0` + sweep (3200 frames, middle pass), `watch 10 B:11 st x y z` | class 11 appears at (1648, 0, 280) near t = 980; the door is a black doorway from t = 1000; camera area changes to 1 at t = 1100 (pictures every 10 frames 900 to 1130) |
| A5 | `jump 1 1` + sweep (2400 frames) with `snaps` 100 to 270 step 10 and 1700 to 1870 step 8 | class 12 var 0 at t = 160 (hall door black by t = 170); var 4 object exists from t = 1020 at (880, 48, 64) with state 0x02000000 from t = 1040; at t = 1780 to 1790 the window breaks (state 06000000 at 1780, position (906, 96, 48)); area changes to 2 at t = 1800 where class 14 var 0 appears at (96, 656, 288) |
| A6 | `jump 1 1` + sweep (2400 frames), `snaps` 1790 to 2100 step 10 | window hole picture (class 14 var 0) at t = 1830; door burst (class 14 var 4) near t = 2010 to 2060 |
| A7 | `jump 1 0`; `20 hp 0`; `frames 600`; `watch 10 B`, `watch 10 E`, `snaps 70 .. 400 step 30` | class 18 object at t = 280 to 320; pool E class 8 objects falling (5 in the line, the fifth var 1), pool E class 1 var 2 at t = 340; speech bubble |
| A8 | `jump 1 0 at=start`, `frames 1000`, `watch 20 B`, snaps every 20 | caption B66 var 9 at t = 40; B73 at 100; B74 at 160; hero HUD at 640; GO hint blink from about 820 |
| A9 | `jump 2 0` + sweep (3600 frames); `jump 2 1` + sweep; `e2n-*`: `jump 2 1` + sweep with `frames 7000` | B80 at t = 0; B72 var 0 at t = 80 and var 0x200 at t = 620; area 1 at t = 2480 with B96, B2 (t = 2480 as listed in e2a0-mid, t = 300 in e2a1-mid) and B7 var 4 (t = 820 of e2a1-mid); area 2 at t = 3640 with B27 at 4020 |
| A10 | `jump 2 1 0x7C0`, `god on`, `frames 130`, `snaps` every 6 | the only changing pixels (besides the HUD and the hero) are in the bottom 4 tile rows of the screen over the full width (water ripples); 4 pictures at 24 frame steps |
| A11 | `jump 1 0`; `3 spawn E 5 var=N x=+0x40`, N = 0, 2, 4, 6, 8, 12; crop of the pictures | pail, drum, barrel, stone, boulder, tire stack |
| A12 | `jump 2 1` + sweep 5700 frames, then `5700-11000:RD`, `frames 11000` | area 2 camera follow after about 6060, lock (mode 4) at camera 384 with wave 6, hint var 2 at t = 6280, B50 var 1 at 6640, B76 at 6880, B22 at 6940 |

## 8. Port task list for the pool B classes of Episodes 1 and 2 (and the shared services)

Sizes: S 0.4 h, M 1 h, L 2.5 h (agent hours, one task = one class or a small group). Prerequisites: M0 (episodes start), the effects of D3f (pool C 21, 24 and the glass shard 0x93362), the text printers of M1.

| Task | Content | Size |
| --- | --- | --- |
| **T-B0 services** (do first) | tile block writer API (layer, column, row, ring buffer rules of 1.4), the prop hit pass 0x10CD0 with the `+$44 / +$48 / +$6C` prop contract, the registered prop list 0x6724, the break sound helper 0x9E60 and pan, the scripted control action table hook for B11, B20 var 2 and B21 | 3 h |
| **Episode 1** | B40 steam pipes (class 15 pieces exist), B11 door, B12 (var 0 door, var 4 window with G.$506), B14 (var 0, var 4), B66 captions (needs 0x84F72), B74 master, B20 hint, B31 clock, B69 stub, B18 respawn air strike (needs pool E classes 8 and 1), B96 music starter | B40 M, B11 S, B12 M, B14 S, B66 M, B74 M, B20 S, B31 M, B69 S, B18 M, B96 S: about 8 h |
| **Episode 2** | B2 water ripples, B7 var 4, B27, B72 with pool A 46 and 48, B80, A16 car (D3b/D3a) | B2 M, B7 S, B27 M, B72 S plus 0.5 h for A46 and A48, B80 S: about 3.7 h |
| Classes with no episode (scripted control or unknown spawner) | B16, B21, B35, B46, B47, B51, B65 | about 3.4 h if needed; decide with D4c (B21 is script action 0x24) |
| Episodes 3 to 8 (for the later parts) | by first episode of use: E3 (88 to 90 radio and car, 95 sky blender) 3.7 h, E4 (0, 1, 59) 2.4 h, E5 (5, 6, 10, 13, 48, 49, 50, 62, 70) 7.5 h, E6 (3, 4, 8, 9, 39, 55, 56, 60, 79) 6.6 h, E7 (30, 33, 34, 41, 52, 54, 57, 61, 63, 64) 6.7 h, E8 (28, 29, 36, 37, 38, 42, 43, 44, 45, 53, 71, 83, 84, 85) 12.8 h | 39.7 h |
| Front end classes (22) | belong to M1, M2, M3 (select screen 17, 19, 67, 68; movie 23, 24, 25, 26, 32; story 91 to 94; ending 81 to 87; flow 58, 75 to 78, 86) | about 20 h |

The tile writers are mostly data: one generic "block table" driver (source block pointer table, destination address, size, period) can run classes 5, 6, 7, 8, 9, 10, 27, 28, 29, 36 to 39, 41, 43 and the palette cyclers 59 to 63, 71, 95; the one shot events 3, 4, 11, 12, 14, 42, 46, 49, 51, 52 need the same driver with a trigger. If the port builds that driver first, most S tasks drop to 0.2 h.

## 9. Open items

1. **Look of the animation classes of Episodes 5 and 6** (5, 6, 9, 10, 13): sizes and periods are known, pictures are not. The spawn test shows class 6 as a pale blue and white 144 x 64 block. Part 2 must walk those areas.
2. **The global tile clock G.$7AE** (camera structure field, G.$7AF holds the last value): classes 10, 28, 29, 36 to 38 read it. In Episode 8 area 1 it steps 0, 4, 8, 12 about every 45 frames (a cycle of about 180 frames); in Episode 6 area 1 it stays 0xFF. The writer is not found (not a direct `$7AE(a5)` write in the sweep listing). [M ph6, ph8]
3. **Class 40 var 0x10** (Episode 8 area 1 at X 0x6E0): the pointer table 0x8378C has only 4 entries, var 0x10 reads the next table as a pointer (0x00340034, an unmapped address): the tile write goes nowhere on real hardware. Check the picture in part 2.
4. **Which scripted control script an area end uses** and who else reads G.$506. D4c.
5. **Episode 2 area 2**: the sweep walker was slow in the diagonal; the tire stacks (E5 var 0xC), the Butcher yard tile animation (class 27) and the region of its tile write were not seen animating. Test the pit region with a camera at X 0x40 to 0x1A0 and a screenshot every 6 frames.
6. **No spawner found:** classes 16, 35, 46, 51, 58, 65, 75 (47 only through class 35, 21 only through script action 0x24). Role unclear: 25 (HUD style object spawned by the movie text), 35 (attached to an enemy object).
7. **Unused art:** "BREAK!" and "Look out!" banners, chain-link fence with a sign (3299), spiked ball (probably A62), manhole and chains: no code found. Confirm for A62 in D3c.
8. **Damage of the respawn bombs** (pool E classes 8 and 1) and whether they hurt other players or only enemies: D4a and D3f.
9. **Window break by a normal player:** class 12 var 4 registers only when G.$506 is set; not tested whether a player attack at that moment breaks it (probably yes, the hit pass is the same).
10. **Class 0, 1, 3 pictures** (Episodes 4, 6, 7) and the 15 variants of class 0: part 2.
11. **Hazards that hurt:** none in pool B. Hostile objects are pool A (flames A83, Zeke A82 ...) and registered in 0x6B64: D3b, D3f.
12. The map panel object of the link scenes (class 58 vs the flow code) and the Episode 2 intro objects: D4c and D4d.

## 10. New rows for `mechanics-seen.md`

| ID | Mechanic | Source | Port | Notes for the design agent |
| --- | --- | --- | --- | --- |
| W27 | Steam pipes in the end wall of Episode 1 area 0: break with one punch (hero depth 296 to 320), then steam puffs forever (6 pieces alive) | M e1-pipez*, e1-pipebreak; R 0x836A4 | MISSING | B40, pieces ported. Answers W25 for Episode 1 |
| W28 | Episode 1 end door of area 0 is smashed by the hero during the scripted walk (dust, black doorway) | M e1a0-door; R 0x1DD30, 0x789C8 | MISSING | B11 |
| W29 | Hall double door opens at the start of wave 1; the hall window shatters when the scripted hero kicks it; the next area starts with a hole in the facade and later a wooden door bursts open | M e1a1-hall, e1a2-enter; R 0x78B52, 0x7B0BE | MISSING | B12, B14; flag G.$506 arms the window |
| W30 | Respawn air strike: five bombs fall in a line, white blast, explosions around the dropping hero, speech bubble | M respawn; R 0x18B9C, 0x7B65E | MISSING | B18 (not a 1UP marker: corrects P40 and P22) |
| W31 | Breakable family of pool B: classes 0 (generic wall panel, 15 variants), 1 (chain-link fence), 3 (wall), 12 var 4 (window), 40 (steam pipes), 45 (flame vent, HP 0x30, can be destroyed); only these 6 register in the player hit list 0x4BFA; one hit always breaks | R static search | MISSING | no pool B class hurts players |
| W32 | Episode 2 ambient life: small blue bird flocks (B72, A46 with A48 trail) harmless; ground critters (A78, A80) harmless; jungle sound maker (B80, sound 0x78) | M e2-birds2; R | MISSING | |
| W33 | Tile animation family of pool B: periodic scroll2, scroll3 or palette writers (23 classes), some keyed to a global tile clock G.$7AE | R | MISSING | one generic block table driver can run most |
| W34 | Class 69 is a debug object (frees itself unless G.$4AA); class 66 has an anti tamper check (0x86B3C); class 96 starts the area music (table 0xA582) | R | n/a (69, 66 check), MISSING (96) | |
| W35 | Class 95 blends 3 palettes toward a target set along the Episode 3 road (sky colours); classes 83, 84, 85 are debris showers with quake (Episode 8 lair, ending) | R | MISSING | |
| W36 | Container variants of E class 5: 0 pail, 2 drum, 4 barrel, 6 stone, 8 boulder, 0xA armor statue (solid), 0xC tire stack | M cont0 .. cont12 | DONE (pool E) | names only |
| W37 | "STAGE CLEARED!" and digits 1 to 8 are frames of pool C class 9 (0x93184); 1P, 2P, 3P tags are B68; red pillars are B64; fences are B1 | R | MISSING | answers N1 of the asset catalog for those groups |
| W38 | Treasures of Episode 2 area 0 sit at depth 16, height 64, behind the big yellow trunks (E class 9 var 2, 8, 0xE, 0xD) | M e2-treasure | DONE (pool E) | |

Corrections to older rows: P40 (1UP marker pool B 0x48) is the respawn air strike class 18; P22 (respawn bubble and bazooka) also involves class 18.

## 11. Note for the next part (Episodes 3 to 8)

- **Walk first** (recipes in section 7, the sweep walker): Episode 3 (one long area, road bends: B95 sky blender, radio B88, row scroll), Episode 4 (area 1: class 0 panels at X 0x590, 0x900, 0x980, class 1 fence at 0x610, area 2: palette B59), Episode 5 (area 1: B5, B6, B10, B13 huts, B62, B70, B48, B49, B50), Episode 6 (B3 wall, B4 wave walls, B7 var 0, B8, B9, B39, B55 rain, B56, B60, B79), Episode 7 (lift B57 and B64, B1 fences in area 2, B30 and B33 and B34 and B61 and B63 in the lab, B41, B52, B54), Episode 8 (B28, B29, B36 to B38, B42 to B45 tanks and flame vents, B53, B71, B83 to B85).
- **Classes still unknown in role or picture:** 5, 6, 9, 10, 13 (Episodes 5, 6), 28, 29, 36 to 38, 41, 43 (tile clock families), 30 (Episode 7 lab machine), 35, 25, 53, 56, 79, 49. Use `spawn B N` only for sprite classes (the tile classes need their map region on screen: jump to the trigger X and take snapshots every 6 frames).
- **Hit boxes** of classes 0, 1, 3, 45: read the box tables at 0x750B4 (15 variants), 0x75414, 0x76666, 0x84122.
- **Things to check against W26 in every episode:** objects hit by the scripted hero (G.$506), wave timed walls (B4, B42, B49, B52), scripted control actions that spawn pool B (table 0x1DBD6).
- The probe limits (diagonal and vertical areas) are in `capture-recipes.md` section 8; the diagonal areas need plain walking right and down for about 600 frames before the sweep can start.
