# D3e part 3: area walk of Episodes 6, 7 and 8 (jungle and mine, the vault, the under world)

Written 2026-10-13 by the D3e part 3 research agent. Part 3 of D3e. Part 1 ([D3e-pool-b-inventory.md](D3e-pool-b-inventory.md)) has the method, the
97 pool B classes and the walk of Episodes 1 and 2. The tanks of Episode 8 area 0 and all bosses (Tyrog, the Slisaurs, Dr. Fessenden) are in
[D3d-bosses-2.md](D3d-bosses-2.md) (section 5 for the tanks): they are linked here, not repeated. Dinosaurs and the foot are in
[D3b-dinosaurs.md](D3b-dinosaurs.md), enemies and rolling barrels in [D3a-humans.md](D3a-humans.md), drops in [D4a-items-drops.md](D4a-items-drops.md),
camera, locks and end scripts in [D4c-camera-flow-transitions.md](D4c-camera-flow-transitions.md), sound codes in [D4e-sound-cues.md](D4e-sound-cues.md), pool C effects (C1, C21 variants) in [D3f-effects.md](D3f-effects.md). Part 2 ([D3e-part2-ep3-5.md](D3e-part2-ep3-5.md)) walked Episodes 3 to 5 and describes B0 and the Episode 4 use of B1 (same class as the library fences).
Nothing here is ROM data: only descriptions, numbers and addresses.

Source tags: **R:** ROM address or repo file, **M:** MAME probe (batch and segment name, see 1.2), **W:** web, **U:** the user. MAME and ROM win over the
web. `[I]` marks an inference that no run proves yet. "G.$xxxx" is RAM `0xFF8000 + xxxx`. Pool B classes are written `B3`, pool A `A22`, pool C `C21`,
pool E `E5`. Positions are map X, depth Z and height Y in pixels; `t` is the frame after the jump (first frame of control = 0).

## 0. Summary of the findings

1. **Only four pool B classes can be broken by a player in these three episodes, and nothing else in the backgrounds can**: the mine entrance wall
   **B3 var 0** (Episode 6 area 0, X 0x350), the two **chain-link fences B1** (Episode 7 area 2, X 0xE0 and X 0x7C0), the **steam pipes B40** (Episode 8:
   one set at the end of the lab, three sets in the sewers) and the six **flame vents B45** (sewers). Every other wall, door, window, lamp, shelf, tank
   and pipe in the three episodes is scroll tiles, a palette cycle or a scripted event. [R static search of the player hit list 0x4BFA (part 1); M r1 sweeps of every area, targeted tests r1, r2, r4]
2. **The hit rule is the same for all of them** (part 1 section 1.3, now measured in four more places): a normal player attack box hits a prop when the hero
   depth Z is from **8 below the start of the prop depth range up to its end** (Ep 1 pipes: 296 to 320 for a range 304 to 320; Ep 8 vent: 824 and more hits,
   816 and less misses for a range 832 to 880; mine wall: Z 32 and more hits, Z 16 misses, range 32 to 128). One hit always breaks B3, B1 and B40; the vent B45 has
   48 HP and needs 4 punches of Jack (10, 10, 15, 15). Jab, jump kick and the 360 special all work (they differ by reach and height). [M r1 e6a0-wall-*, r2 c-b45-z*, c-b40-*, r1 e7a2-fence*]
3. **Doors, shelves and walls that burst open by themselves at a wave start** (B4, B42, B52) are a third family: a tile block is swapped for a black doorway,
   planks or books fly (pool C 21), the break sound plays and the enemies of the wave walk out of the gap. Episode 6 area 0 has **three** such doors (X 0x220, 0x2A0, 0x7B0), the
   sewers **four** red doors (X 0x1C8, 0x383, 0x980, 0xB47), and the library **one bookshelf** (X 0x730, wave 12) that opens and lets a Ferris out. This corrects the
   "bookshelves are only background" note (N11): all bookshelves are background except this one. [M r1 e6a0-door1, r2 n-e8a1-w3, n-e7a2-end, r4 tc-b4*, tc-b42*, tc-b52]
4. **Episode 6 area 1 is a storm**: rain streaks (B55, scroll1, redrawn every frame), rain drops with floor splashes (B56, pool C 2, sound 0x8D every 241 frames, probably the rain loop [I]), and three
   **lightning** objects (B9). A lightning flash is a bolt drawn in the far layer (5 pictures of 3 frames), then the sky palettes flash, 8 frames later all foreground palettes
   flash (orange, white, grey steps of 3, 1, 2, 1, 2, 3 frames), and a sound (code 0x77, a 1.25 s effect, probably thunder [I]) follows 40 frames after the bolt. Pauses between flashes come from a table of
   8 values (60 to 420 frames). Rain and drops end at camera X 0x500. [R B9 0x77E76, B55 0x8574C, B56 0x85916; M r2 a-e6a1-storm, n-e6a1-lightning]
5. **The brontosaurus foot has a shadow**: a black oval on the floor under the foot (the pool A shadow of A22, at its X and Z) that follows the hero while the foot hovers. The foot hurts
   enemies as well (D3a). **Rolling barrels** and the "slow player" rule: a barrel is freed as soon as it is 0x40 left of the camera or 0x200 right of it, and it rolls 2 pixels per
   frame, so a fast hero (camera 3.4 pixels per frame) loses the barrels behind him and a locked camera loses them to the right. Measured in Episode 6 and in the cave. [M r1 e6a1-barrels-slow, e6a1-barrels-fast, e8a2-sweep-500]
6. **Episode 7 lift** (area 1): four stops at camera Y 12288, 7680, 5120, 2048 and the end at 0; wave 4 (first stop) is joined by two enemies that **smash the left and right wall of the cage**
   (B54, scroll1 blocks, once per stage); the floor labels painted on the cage wall (B57) count L15 down to L1, then B1 to B10, a blank zone (B11 to B20, a big open window with the harbour
   behind it, camera Y about 7400 to 5200), then B21 to B40. Riders enter from both sides (X -48 and 432) or fall from the top (variants 0x20A). [R B57 0x85B64, B54 0x855A8; M r2 n-e7a1-full, l-wave4]
7. **Gates**: the library fence B1 var 4 is broken by a punch (any player, any time, 1 hit; the web says it blocks the way, whether it stops walking was not tested); the computer room is entered through the second fence B1 var 8, which the leader of the end
   script 19 breaks after the walk (about 60 frames after the GO). [M r1 e7a2-fence1-z*, r2 n-e7a2-end]
8. **Sewer** (Episode 8 area 1): six flame vents B45 (HP 48, flame pieces A83 hurt heroes and enemies for 20 at level 0, 22 at level 10, burn knock down), three pipe sets B40 (punch or jump kick,
   then steam for ever), four red doors B42, water bands B28 and B29 and torches B43 on the tile clock, the waterfall B36, B37, B38 at the end. The area ends with the heroes jumping
   down the waterfall shaft into the cave; the lab (area 0) ends with the leader **lifting a round floor grate (B35, hole B47), throwing it away and jumping into the manhole**. [M r1, r2 n-e8a0-end, n-e8a1-end, c-flame-*]
9. **Cave** (area 2): barrels fall from above with a floor shadow (A65 var 0x201), two Zeke, and B53 changes 27 palettes to a fire glow while a hero lies burning (G.$7B2 = 2). The **meat barrel**
   at X 0xCB0 can be lifted and carried across the area switch and the walk to the lair (still held at X 4747 after 2600 frames). [M r4 b53-a2, r3 m-e8a2-meat]
10. **Tile clock** G.$7AE (read by B28, B29, B36 to B38, B43 and others): it steps 0, 4, 8, 12, 0 ... every 24 game frames: `7AE = ((gf + 60) / 24 mod 4) * 4`; G.$7AF copies it one frame later. [M r1 e8a1-idle-*, 31 changes each]
11. **All pool B classes of the three episodes are now identified** (section 2, 38 classes); the two things still open are B71 (the end area palette, never spawned in my runs) and the exact pictures of the
    B30 monitor (static noise, then the doctor's face with a three line caption).
12. **Corrections**: part 1 open item 3 (B40 var 0x10 writes to an unmapped address) is wrong in effect: the sewer pipes of var 0x10 break and steam like the others (picture r2 c-b40-v10-jk);
    the Episode 7 background is the same skyline art as Episode 1 in orange palettes but the foreground is a stone terrace and bridge, not the Episode 1 roof; the lamps of the mine are palette
    cycles of two scroll palettes (B60), not a sprite.

## 1. Method and evidence

### 1.1 What was done

- Read first: the files named in the task, `re/specs/enemy/stage_scripts.txt` (stages 6, 7, 8), the asset dump `re/out/assets/stages/` and the class listings of part 1 (`re/out/d3e/cls/cNN.txt`). New small tools in `re/out/d3e3/`: `gen.py` (probe input helpers), `an.py`, `inv.py` (object inventories from watch lines), `chg.py` (which tile or palette words change, from full RAM peeks), `tilecount.py` (changed tiles between two peeks), `montage.py` and `mt` (contact sheets), `rom.py` (ROM table reader).
- Four MAME runs (headless, `SDL_VIDEODRIVER=dummy`, one process each; all output in `re/out/probe/`, git-ignored). The inputs are `re/out/d3e3/run1.txt` to `run4.txt` (generated by `gen_r1.py` to `gen_r4.py`).
- **Sprite free tile and palette capture**: new in this part. A `peek` of the whole scroll1, scroll2, scroll3 RAM (8192 words each) and the palette RAM (2048 words) every 2 to 6 frames in idle windows gives the exact tiles and palettes a class rewrites, with no sprite noise (`chg.py`). Map streaming (camera moving) must be excluded by reading idle windows only.

### 1.2 The four runs

| Batch | Segments | What |
| --- | --- | --- |
| `d3e3-r1` | 109 | broad walk: sweeps of every area (mid, back, front passes, `hold` kill-all in Episodes 6 and 8), natural runs, idle windows with 6 frame snapshots, first targeted tests (mine wall by depth, doors, lamps, foot, barrels slow and fast, fence hits, vent hits, pipe hits), kill-all crash battery of Episode 7 |
| `d3e3-r2` | 60 | sprite free animation windows of every area (`a-*`), natural sequences with end scripts (`n-*`: lift, library end, lab talk, manhole, waterfall), fine lightning frames, targeted tests v2 (`c-*`: vent depth scan, flame damage, pipe hits) |
| `d3e3-r3` | 13 | scripted ends by poke (D4c recipe: Episode 6 area 0 end, Episode 7 area 0 end), meat barrel carry, lab lights, sewer waterfalls, special attack palette test, replay of the D3d final boss kill with pool B watchers |
| `d3e3-r4` | 24 | tile counts per class (spawn the class, peek scroll2 before and after), pipe and vent tile patches, mine wall from the right, G.$7B2 palette effect of B53 |

Segment names below are written `r2:n-e7a2-end`. The segments are deterministic (same lines, same frames); a port test replays the same lines.

### 1.3 Limits met

- `jump EP AREA at=start` does not stop at the walk-in of areas above 0 (it stops at the first frame of control as without it), so area starts were read through natural runs or through the scripted end of the area before (`poke` of G.$4DE and G.$4DD, D4c recipe).
- `hold 0xFF850E 1 2` (kill-all) does not kill Lash T. and Walther (mid bosses), so Episode 6 wave 2 cannot be finished with it (`r2:n-e6a0-end` stayed at the lock); Episode 7 was walked without kill-all holds (D4c saw a crash). A single `killall` pulse and a pulse with each of 11 spawned classes did not crash (`r1:x7-k*`); the crash of `d4c-b2/e7a0n` (an "ADDRESS ERROR" screen with three players and the hold) was not reproduced (open item 1).
- `hold` of the hero X and Z keeps the hero where the test wants him (the game moves a hero out of a wall: poked inside the wall at X 900 he ended at X 846, the left side).

## 2. Pool B classes of Episodes 6, 7 and 8, all identified

`Size` is the port estimate in agent hours when the shared services of part 1 section 8 exist (S 0.4, M 1, L 2.5). "Trig" is the script list 1 trigger (camera X at which the record spawns).
Records come from `re/specs/enemy/stage_scripts.txt`. Classes 15, 20, 31, 66, 69, 96 appear in every episode (wave controller, GO hint, clock, caption, debug stub, music): see part 1; only the caption texts of these episodes are new (2.7).

### 2.1 Table

| B# | Handler | Role | Where | What it is (measured) | Key numbers | Size |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 0x75238 | BRK | E7 a2: var 4 X 0xE0 Z 0x80 (trig 0); var 8 X 0x7C0 Z 0x80 (trig 0x680) | **Chain-link fence (gate)**: a sprite (composition 326 whole, 327 broken) over scroll2 fence tiles; HP 1; any player attack box breaks it; var 8 is also broken by the leader of end script 19 | box depth [Z-80, Z-16], X [X+64, X+144], height 0..128; break: C21 var 0xC, 48 attribute words (13 x 6 area) rewritten for var 4, 3 for var 8, sprite 327, hit at t = 22 after a punch at t = 20 | M |
| 3 | 0x763F6 | BRK / EVT | E6 a0 var 0 X 0x350 Z 0x60 (trig 0x1C0); E6 a1 var 8 X 0 Z 0x130 Y 0x20 (trig 0) | **Mine entrance wall** (var 0): solid planks, 1 hit breaks, planks fly, rock mouth shows. Var 8 is the **mine exit wall**: 48 frames after the jungle starts it breaks by itself (planks fly) for the walking-in heroes | var 0: HP 8, box depth [32, 128], X [0x350, 0x370], height 0..96; break changes 31 tile code words and 37 attribute words (cols 52..58 rows 49..62), C1 spark at the hit frame, C21 pieces (11 objects, var 6 to 0xD) 2 frames later, break sound 0x6F or 0xF6; var 8: 27 codes (26 changed), 6 attributes, C21 var 0x14, no sound | M |
| 4 | 0x767D8 | EVT | wave records: E6 a0 wave 0 var 4 X 0x220 Z 0x60 and var 8 X 0x2A0 Z 0x60, wave 2 var 0xC X 0x7B0 Z 0x60 | **Door that bursts open at the start of a wave group**: waits 1 frame, swaps a 4 x 6 tile block (24 codes, 4 attributes) for a black doorway, break sound, C21 var 0x16 (planks), frees itself. The group walks out 0x50 frames later | var 4: first seen t = 102 (wave 0 starts t = 100), enemy visible in the doorway at t = 186; var 8 at t = 268 (second group); object Y = 0x60 | M |
| 7 | 0x76CD2 | ANI | E6 a1 var 0 X 0xB50 Y 0x1D0 (trig 0xA00) | **Eye of the huge swamp reptile** (the pale head in the swamp background): 2 x 2 tiles of scroll2 at col 53..54 rows 35..36; waits 15 frames, then a picture every 24 frames, 5 pictures, 96 frames pause; red iris opens and closes; stops when the camera X is past 0xB80 | change frames t = 30, 54, 78, 102, then 198 .. 294 | S |
| 8 | 0x76E0E | ANI | E6 a1 X 0xA50 Y 0x160 (trig 0x8C0) | **Swamp water surface**: 8 rows (42..49) of scroll2 over the whole width, 4 pictures, one every 24 frames; also writes the water phase word G.-$703C used by the wading draw | change every 24 frames from t = 18 | M |
| 9 | 0x77E76 | ANI + sound | E6 a1 var 0 X 0xA0, var 4 X 0x260, var 8 X 0x340, Y 0x100 (trig 0, 0x1C0, 0x380) | **Lightning** (2.3) | bolt tiles scroll3 0x9138A0, 0x913A60, 0x913B40; palettes; sound 0x77 | L |
| 28 | 0x7F72A | ANI | E8 a1 X 0x20, 0x520, 0x620, 0x720, 0xB20, vars 0 to 8 (trig 0, 0x370, 0x470, 0x570, 0x970) | **Sewer water band** at the walkway edge: 3 rows (12..14), wide block, 4 pictures on the tile clock | rewrites when G.$7AE changes, every 24 frames; runs while the camera is within 0x100 left of X | M |
| 29 | 0x8048A | ANI | E8 a1 X 0x120 .. 0xA20, vars 0 to 0x18 (7 records, Z 0x300) | **Sewer water band, tall block**: 3 columns x 16 rows, 4 pictures on the tile clock (only the water rows differ) | same clock | S |
| 30 | 0x811DE | EVT | E7 a3 X 0x4B0 Y 0xD0 (trig 0x420) | **Lab monitor**: the black screen turns to static, then shows the doctor; spawns the caption box B34 | t = 410 spawn, 470 static, 590 face and caption (r2:n-e7a3-talk); sound 0x23 | M |
| 33 | 0x818E4 | SPW | E7 a3 (trig 0x3C0) | Slisaur starter, D3d section 4 | | done in D3d |
| 34 | 0x81B3E | TXT | by B30 | Caption box of the monitor talk: 3 lines of the tall font | appears t = 590 | S |
| 35 | 0x81FDE | FX | E8 a0 end (script 21, leader action) | **Manhole cover**: a round steel grate lying on the floor at X 0x637; the leader lifts it over his head and throws it to the right; it follows the owner hero (+$76); it is made by 0x81FB0 from the hero pose routine at 0x19F8A (anim 0x1F; a second copy of the routine is at 0x2864) which copies the hero position | seen t = 710 .. 890 at (1591, -9, 55) | M |
| 36 | 0x8217E | ANI | E8 a1 X 0xC20 (var 0, trig 0xA70), X 0xD20 (var 4, trig 0xB70), Z 0x300 | **Waterfall over the edge** at the end of the walkway (two blocks, tile clock) | spawn during end script 22, t = 452 | S |
| 37 | 0x82372 | ANI | E8 a1 X 0xD20, Z 0x300 var 0, Z 0x200 var 4, Z 0x100 var 8 (trig 0xA70) | **Waterfall shaft water**, strips in the shaft below the edge (3 stacked screens) | 2 strips of 16 tiles per step | S |
| 38 | 0x8279C | ANI | E8 a1 X 0xD20, vars 0 to 5 in three rows (trig 0xA70) | **Waterfall shaft water**, 80 tile blocks per step, vars 0, 1 (row Z 0x300), 2, 3 (0x200), 4, 5 (0x100) | | M |
| 39 | 0x830C6 | ANI | E6 a2 X 0xF00 Y 0x110 (trig 0xD80) | **Swamp water surface** of the end of the forest: 5 rows (43..47) in two ring blocks (cols 0..15, 48..63), 4 pictures of 24 frames; runs while camera Y < 0x170 | change every 24 frames from t = 6 | S |
| 40 | 0x836A4 | BRK | E8 a0 var 4 X 0x690 Z 0x40 Y 0 (trig 0x4E0); E8 a1 var 8 X 0xB0 Z 0x340 Y 0x30, var 0xC X 0x580 Y 0x20, var 0x10 X 0x6E0 Y 0x20 | **Steam pipes** (part 1 section 3.1): 1 hit breaks, 11 tile codes change (3 columns of 3, 4, 4), steam puffs for ever (A15, 6 alive) | box depth [Z, Z+16], height 56..96; kind 0 X [X+16, X+48] (vars 4, 0x10), kind 1 X [X-32, X] (vars 8, 0xC) | M (class exists in Ep 1 plan) |
| 41 | 0x8383A | ANI | E7 a3 vars 0, 4, 8 at X 0x430 (trig 0x260) | **Console blinking lights**: three small tile sets (6, 8 and 3 tiles at ring cols 0..2 row 50..51 and 10..17 row 49), 4 pictures; runs while the camera is within 0x100 of X (camera 0x330 and more) | var 0 period 19, var 4 period 7, var 8 period 13 frames | S |
| 42 | 0x83A0A | EVT | wave records E8 a1: w3 var 0 X 0x1C8, w4 var 4 X 0x383, w6 var 0xC X 0x980, w7 var 0x10 X 0xB47 (Z 0x370) | **Red sewer door bursts open** (same family as B4): 4 x 6 tile codes (24) swapped for a black doorway, break sound, C21 var 0x26 | M r2 n-e8a1-w3: first seen t = 150; planks fly t = 160 to 190 | S |
| 43 | 0x83ACA | ANI | E8 a1 ceiling, Y 0x3E0, X 0x130, 0x2F0, 0x3F0, 0x500, 0x7B0, 0x940, 0xB10 (vars 0 to 0x18) | **Wall torch flame**: 2 x 2 tiles (ring cols x, rows 1..2), 4 pictures, one every 8 frames, wait 15 first | change 36 times in 300 frames | S |
| 44 | 0x83BB0 | SPW | E8 a0 | Lab tank breakers: [D3d section 5](D3d-bosses-2.md) | | done in D3d |
| 45 | 0x83E0C | BRK + hazard | E8 a1 X 0x2A0, 0x320, 0x400, 0x840, 0x8C0, 0x940 (vars 0 to 0x14), Z 0x350 Y 0x30 | **Flame vent** (2.4): round steel nozzle in the back wall, HP 48 | cycle idle 60 to 90, sparks 45, growth 40, full 40 to 70, shrink 40 | L |
| 47 | 0x84282 | EVT | made by B35 | **Manhole hole**: two scroll2 patches (0x90F86C, 0x90F872) written when G.$4F0 is set | with B35 | S |
| 50 | 0x844B4 | TXT | E8 a3 var 4 (trig 0x1120) | Dialogue box of the Fessenden intro | D3d | D3d |
| 52 | 0x85178 | EVT | wave record E7 w12 X 0x730 Z 0x70 (first record of the wave, delay 0x1E0) | **Bookshelf bursts open**: 10 columns x 7 rows (70 tiles: 55 codes and 70 attributes change) become a black gap with books flying (C21 var 0x20), break sound; a Ferris steps out of the gap | spawned t = 876 in r2:n-e7a2-end (wave 12 started t = 110) | S |
| 53 | 0x85306 | ANI | E8 a2 and a3 (trig 0, 0xD60) | **Fire glow palette set**: whenever G.$7B2 changes it rewrites 27 palettes (370 colours): warm red tones. G.$7B2 is 2 while a hero lies burning (burn reaction 0xA or 0xC, R 0x1C6E4 and the three other hero routines) and 0 otherwise | tested: poke 2 at t = 30 changes the palettes at t = 32, poke 0 restores at t = 92; no effect in Ep 8 a1 | M |
| 54 | 0x855A8 | EVT | wave record E7 w4 (X 0xC0 Z 0x30, after the first group) | **Cage walls smashed**: two 8 x 8 scroll1 blocks (cols 5..12 and 35..42, rows 47..54) become the broken lower corners of the cage, C21 var 0x1E (planks), break sound; once per stage (G.$7B6) | t = 616 in r2:l-wave4; two enemies jump in from the sides | S |
| 55 | 0x8574C | FX | E6 a1 (trig 0x100) | **Rain streaks**: 75 streaks of 4 scroll1 tiles redrawn every frame (8 patterns); cleared when the camera X is 0x500 or more | changes every frame | S |
| 56 | 0x85916 | FX | E6 a1 (trig 0x80) | **Rain drops**: 24 slots 20 pixels apart across the top, pool C class 2 drops with floor splash; density 2/16, 4/16, then 8/16 per try (phase changes at 120 and 320 frames); sound 0x8D every 241 frames; ends at camera 0x580 | | M |
| 57 | 0x85B64 | EVT | E7 a1 (trig Y 0x4000) | **Floor labels of the lift shaft** (2.5) | 56 labels, one per 0x100 pixels from Y 0x3760 | M |
| 60 | 0x863AC | ANI | E6 a0 (trig 0x200) | **Mine lamps** (2.2): palette 80 (16 colours) and 81 (8 colours from colour 7) cycle, 4 steps of 9 frames; runs only in stage 5 area 0 | 36 frame cycle | S |
| 61 | 0x864DA | ANI + sound | E7 a3 (trig 0x200) | **Lab lights A**: scroll2 palettes 84, 85, 86, 80 cycle 3 steps every 17 frames; sound 0x6D every 121 frames; stage 6 only | | S |
| 63 | 0x869E6 | ANI | E7 a3 (trig 0x200) | **Lab lights B**: 10 scroll2 palettes (79, 67, 69, 89, 66, 91, 83, 87, 88, 82) cycle 2 steps every 17 frames; stage 6 only | | S |
| 64 | 0x86B64 | EVT | E7 a1 vars 0, 4, 8, 0xC (trig Y 0x4000) | **Cage posts**: four sprite pieces (compositions 1897 to 1900) placed from the lift height; two front posts X 24 and 360 (Y 184), two back posts X 96 and 288 (Y 56, Z 128): the wooden frame of the cage in front of the shaft (the "red pillars" of part 1 are these) | | S |
| 71 | 0x89746 | ANI | E8 a3 (trig 0x1140) | End area palette animation | not spawned in my runs (boss kill replays never reach the trigger) | S |
| 79 | 0x8A3D0 | EVT | E6 a2 X 0x1120 Z 0x190 (L2 trig 0xF80) | **Tyrog scene**: three gangsters carry a metal box, three captions, the box opens, Tyrog jumps out; spawns A75 actors, A76 creature maker, sound 0x21 | B79 t = 350, states 0x04 .. 0x10 at 350, 470, 700, 820, Tyrog (A4 var 0x200) at t = 904 | M |
| 83 | 0x8B212 | EVT | E8 a3 var 0x100 (trig 0x1140), ending vars 0x600, 0x700 | Debris shower (50 to 200 frames), rumble every 16 frames | see 5.5 | S |
| 84 | 0x8B2CA | EVT | E8 a3 (trig 0x1140) | Debris shower (A93 pieces every 32 frames for 400 frames) | seen t = 7304 to 7704 after the boss death | S |

Classes of these episodes spawned by scripts only (B82, B85, B87, B89 and the ending) belong to D4d.

### 2.2 Mine lamps (B60)

- Object created at camera X 0x200 of Episode 6 area 0 and kept to the end of the area; it frees itself when stage is not 5 or area is not 0 (`cmpi.b #5,$4D9` and `tst.b $786`). [R 0x863F4]
- It rewrites the low 12 bits (keeps the top nibble) of **scroll2 palette 80** (all 16 colours, RAM 0x914A00) and of **palette 81, colours 7 to 14** (8 colours, 0x914A2E) from two lists of 4 colour sets (list A 0x86476: sets at 0xBE052, 0xBEA52, 0xBEA72, 0xBEA92; list B 0x86488). One step every 9 frames (`+A5` = 8, counter reload), 4 steps, so **a 36 frame cycle**. [R 0x863C0 to 0x8646E; M r1:e6a0-lamps]
- Measured steps of palette 80 colour 0: dark F321, bright F661, F541, F431 (colour 5: FCBA, FFFF, FEDC, FDCB): the hanging lamps of the mine flicker from dim to white and back. Steps start at t = 9, 18, 27, 36, 45 ... (checked with 4 frame samples). The cabins part (X below 0x348) has no lamp sprite, the glow only exists in the mine tiles. [M r1:e6a0-lamps, r2:a-e6a0-mine: palettes 80 and 81 change, no tile changes]
- Port: a palette cycler driven by two lists: the same driver as B61 and B63 (4 and 10 palettes, 3 and 2 steps, 17 frames) and B71.

### 2.3 Lightning (B9) and rain (B55, B56) in Episode 6 area 1

- **B55 rain streaks** (0x8574C): from camera X 0x100 (trigger) to camera X 0x500. Each frame it advances a phase `+A2` by 8 (0..0x38, 8 patterns) and redraws **75 streaks** of 4 scroll1 tiles (one column of 4, the table of 75 offsets at 0x857D8, a phase offset per streak at 0x8586E, glyphs at 0x858D0). At camera X 0x500 it clears the block (text block clear 0xF94) and frees itself. [R 0x85760 to 0x858CE; M r2:a-e6a1-storm: scroll1 changes at every sample]
- **B56 rain drops** (0x85916): from camera X 0x80. 24 slots, 20 pixels apart from camera X + 0x10; every slot tries every 5 frames, and on success (probability by a 16 entry table: 2/16 for the first 120 frames of the object, 4/16 up to 320, 8/16 after) spawns a **pool C class 2** drop at Y 0xD0 with a random depth from the table 0x85AE4 (256 to 332) and a variant 0, 4 or 8, then waits 30 frames for that slot. The drops fall and splash on the floor (small white dashes). Sound 0x8D (D4e: 4.66 s effect, probably the rain loop [I]) every 241 frames (0xF0 + 1). At camera X 0x500 it switches to its ending routine, and ends at 0x580. [R 0x8592C to 0x85B56; M r1:e6a1-idleA: C2 objects at t = 0, 42, 66, 96, 102, 106, 302, 366; pictures r1:e6a1-dust]
- **B9 lightning**: three objects at far layer X 0xA0 (var 0), 0x260 (var 4), 0x340 (var 8) (the X is the far layer column of the bolt: scroll3 column 5, 19, 26 times 32 pixels). An object runs while its X + 0x20 is not left of the far layer reference G.$7E4. One cycle:
  1. **Bolt**: it writes 4 tiles (a vertical strip of 4 scroll3 tiles of 32 x 32 pixels, at scroll3 0x9138A0, 0x913A60, 0x913B40, row 24) for 5 steps of **3 frames** (picture words from 0x77F74: step 0 is the plain sky, steps 1 to 4 draw the bolt growing downwards; when the fifth step ends the plain sky is written back). Measured var 4 object: bolt pictures at t = 114, 118, 120, 124 and the restore at 126.
  2. **Flash** (starts when the bolt ends): sequence 1 rewrites **8 scroll3 palettes** (100 to 105, 107, 108: the sky) in 6 steps with frame counts 3, 1, 2, 1, 2, 3 (table 0x77FF6); sequence 2 starts **8 frames later** (`+A8` = 9 against `+AA` = 1) and rewrites **all 32 scroll2 palettes** (64 to 95) in 6 steps with the same counts (table 0x78170). Measured: sky flash t = 126 to 136, foreground flash t = 134 to 144. What the player sees: the sky turns orange, then white; the whole foreground turns light grey, near black, white, near black, then back (frames of `r2:n-e6a1-lightning`: t = 0 orange sky, 1 white sky, 3 to 5 grey world, 6 white screen, 8 sepia, 10 dim sepia, 12 normal).
  3. **Thunder** (probably [I]; no audio in the probe): `+BE` counts 0x28 = 40 frames from the end of the bolt, then the sound helper 0xA25A plays code 0x77 (1.25 s effect in D4e, pan from the object X relative to the camera, pan table 0xA66A). [R 0xA25A]
  4. **Pause** before the next bolt: `+A0` = table 0x77F38 [120, 240, 60, 120, 60, 420, 240, 120] frames, index `+A6` (steps 2, wraps at 8 entries; every object has its own index).
- Measured flash times in `jump 6 1 0x2E0` (var 4 object active): sky flashes start at t = 2 (the old object), 126, 378, 450, 582, 654; foreground at 2, 134, 386, 458, 590, 662. Bolt pictures at 114 ..126, 366 .. 378, 438 .. 450. [M r2:a-e6a1-storm, n-e6a1-lightning]
- Port: B9 = two palette tables and one 4 tile strip per object, driven by the object timers above; B55 = a 75 streak scroll1 pattern; B56 = a drop spawner for pool C 2 (D3f owns pool C). About 4 hours together (L).

### 2.4 Flame vent B45 and the flames A83 (Episode 8 area 1)

Measured on the vents at X 0x2A0 (var 0, B17) and X 0x320 (var 4, B15), Z 0x350, Y 0x30. [R 0x83E0C to 0x84198; M r1:e8a1-vent-z8xx, r2:c-b45-z*, c-flame-*]

| Item | Value |
| --- | --- |
| Init | HP `+6C = +6E = +6A` = 0x30 (48), box record 1 of table 0x84122: depth [Z-16, Z+32], X [X+2, X+30], height 32..64, kind params from 0x84182: step 2, maximum 16, period 4 (so 5 frames); frees itself when camera X - 0x20 is past its X |
| Hit | registered in the player hit list every frame (0x4BFA); a hit subtracts the damage and sets state 4; with HP above 0 it goes back to the cycle (the vent is not interrupted). Jack: 10, 10, 15, 15: HP 48, 38, 28, 13, 0 at t = 36, 50, 64, 80 (punch every 14 frames). Destroy: 8 tile codes (cols x/16 .. +1, rows 4..7) become the broken socket, small flames at random directions every 6 frames for 120 frames, then the object is freed |
| Depth window | hero Z 824 and above hits, 816 and below misses (range 832 to 880 minus 8); the walkway band is 768 to 831 near the start and 863 in the middle, so the hero must stand at the very back of the walkway [M r2:c-b45-z760..z824] |
| Cycle (measured states `+5`) | 0 idle 60 to 90 frames (table 0x8413A, random); 2 sparks 45 frames (a small flame effect every 6 frames, 0x967C4); 4 growth: every 5 frames the length `+A8` grows by 2 up to 16 (8 steps, 40 frames) and an **A83 flame piece** is spawned with that life; 6 full flame: 40 to 70 frames (table 0x8415A), a piece of life 16 every 5 frames; 8 shrink: length falls by 2 every 5 frames (40 frames) with pieces; then idle. Measured (the idle and full durations are random): B17 0200 at t = 0, 0202 at 50, 0204 at 96, 0206 at 142 in r2:c-flame-x680; and 0200, 0202 at 60, 0204 at 110, 0206 at 150, 0208 at 210, 0200 at 260, 0202 at 340 in r2:n-e8a1-w3 |
| Flame piece A83 (0x6155C, pool A class 83, made by 0x6152C) | starts 18 pixels right of the vent and 73 up (y 121), the vent X, Y and Z; velocity (-7.5, 0, -5) pixels per frame (table 0x61730: the jet shoots left and towards the camera); life = parameter + 1 frames; attack id 0x5D (93), registered in the hostile list 0x6B64; draws one of 4 flame pictures |
| Damage | hero (Jack, level 0, `level 0 free`): **20** per flame hit (HP 100, 80, 60, 40 at t = 102, 222, 342); level 10: 22. The hero is thrown into the burn reaction (state 0xA, slides about 150 pixels, lies, `$23 = 0x84`, G.$7B2 = 2). An enemy (Hammer) takes the raw 20 too: 100, 80, 60, 40 in three passes [M r2:c-flame-enemy]. Flames hit only close to the vent: hero at X 680 (8 right of the left vent, Z 831) was hit, at X 720 and 760 not hit in 700 frames |
| Drops | none |

### 2.5 The lift of Episode 7 area 1: labels, window, cage

- **B57 labels** (0x85B64): created at area start (trigger Y 0x4000) with `+C` = 0x3760. Each frame it compares its own Y with the camera Y: when the camera Y has fallen to its Y or below, it paints the next label and lowers its own Y by 0x100, so **label k appears when the camera Y is 0x3760 - 0x100 k or less**. A label is the letter glyph (L or B), a dash and two digits (3 words of glyph numbers per label from table 0x85BF6, glyphs from 0x85D6C, painted into scroll2 at 0x90C268 with the ring advancing 0x1000 per label). The 56 labels: k = 0 to 5 **L15 to L10**, 6 to 14 **L9 to L1**, 15 to 24 **B1 to B10**, 25 to 34 **blank** (ten floors with no label), 35 to 43 **B21 to B29**, 44 to 53 **B30 to B39**, 54 **B40**, 55 blank. The four stops show L8 (Y 0x3000), a blank, B21 (Y 0x1400), B33 (Y 0x800). [R 0x85BF6; M r1:e7a1-nat pictures "L-8", "B-33" and r2:n-e7a1-full]
- **The open window**: the blank labels are camera Y 7776 to 5216. In that range the cage looks out of a big opening onto the harbour of the City in the Sea (ships, a submarine, cranes, the far layer); it shows from about Y 7450 (t = 2460 of r1:e7a1-nat) to about Y 5200 (t = 2940). The first stop of the ride (Y 0x3000) and the third (0x1400) are in rock shaft, the second (Y 0x1E00 = 7680) is at the top edge of the window. [M r1:e7a1-nat sheet, camY log]
- **B64 cage posts**: four sprites (compositions 1897 to 1900) from the lift height `G.$6C4` and `G.$7B8`: front posts at X 24 and X 360 (Y 184), back posts X 96 and X 288 (Y 56, Z 128): the wooden frame of the cage seen in front of the shaft. The floor mesh and the horizontal rails are scroll1 (D4c 7.2).
- **B54 cage smash** (wave 4, first stop): the wave record creates B54 after its first group (delay 0x1E): two 8 x 8 scroll1 blocks (cols 5..12 and 35..42, rows 47..54, 64 x 64 pixels each, the lower left and lower right corners of the cage) change to the broken pictures, C21 var 0x1E (planks) and the break sound play, and G.$7B6 is set so it never repeats. Two enemies (a poacher and a driver) jump into the cage through the gaps at that moment. Seen at t = 616 (wave 4 started t = 532). [M r2:l-wave4, r1:e7a1-nat]
- **Riders** (list 2 records by camera Y, D4c 7.2): measured entries in `r2:n-e7a1-full` (camera Y of the entry): start Y 0x3700: poacher X 232 and Ferris X 272 inside the cage; stop 1 (Y 12288): Ferris at X -48 and 432 (both sides, Z 48), then X -55 and 441; during the ride Ferris variant 0x20A **falls from above** (y 255, X 144 or 240, Z 40 or 64); stop 2 (Y 7680): Hammer var 2 falls (X 192, y 255), Punk var 0x206 / 6 enter from X -36 and 420 (Z 36 and 84); stop 3 (Y 5120): Poachers var 6, 0x206, 0x406 drop in at y 136 (X 64, 272, 319, Z 32, 79, 32); stop 4 (Y 2048): Ferris var 0x202 / 2 / 0x402 enter from X -48 and 432 (Z 36 and 84), Hammers 0x202 and 0x402 fall at y 255; end (Y 0): Ferris at X 255..304 and two Ferris var 0xA / 0x20A fall at X 80.
- Ride numbers are in D4c 7.2 (speed up to 6 pixels per frame, acceleration 6/256, stops at 0x3000, 0x1E00, 0x1400, 0x800).

### 2.6 Hit rules of all breakable parts (Episodes 6 to 8)

| Part | Where | Hero depth Z that hits | X reach | HP and damage | Attacks that broke it | After |
| --- | --- | --- | --- | --- | --- | --- |
| B3 var 0 wall | E6 a0 X 0x350 Z 0x60 | 32 to 79 hit (box depth 32 to 128), 16 misses | from the left of the wall only (a hero poked behind it ends at X 846 on the left); the wall also stops walking: a hero walking right stops at X 846 | HP 8, one hit | jab (t = 22 after t = 20), jump kick (t = 24), 360 special (t = 50) | the way is open; C1 spark at the hit frame, C21 pieces 2 frames later |
| B1 var 4 fence | E7 a2 X 0xE0 Z 0x80 | 64, 79, 100 all hit (box depth 48 to 112, the band is 69 to 111) | hero X 280 (box X 288 to 368) | HP 1 | jab at t = 22; in natural play at t = 7520 after the lift | sprite 327; no terrain change by the handler (part 2) |
| B1 var 8 fence | E7 a2 X 0x7C0 Z 0x80 | 64 and 100 hit | hero X 0x7D8 | HP 1 | jab t = 22; **the leader of the end script 19 breaks it** (r2:n-e7a2-end: GO at t = 1500, the leader stands at X 2039 at t = 1550, state 0x402 of the fence at t = 1560) | only 3 attribute words change |
| B40 a0 var 4 pipes | E8 a0 X 0x690 Z 0x40 | 56, 64, 72 hit | hero X 1648 (box 1696 to 1728) | HP 8 | jab t = 22 | **not reachable in play**: the lock 0x480 holds the camera, so a hero cannot go past X 0x5F0 (camera + 0x170), and as soon as the lock opens (GO) the end script 21 takes the heroes and they never punch it (r2:n-e8a0-end: never broke) |
| B40 a1 var 8 pipes | E8 a1 X 0xB0 Z 0x340 Y 0x30 | hero 831 (the back limit) | box X [X-32, X], hero X 128 | HP 8 | jab (broke at t = 50, the hero was busy until 48); jump kick missed | 11 tiles change, steam |
| B40 a1 var 0xC | X 0x580 Y 0x20 | 831 | box [X-32, X]; hero X 1336 | HP 8 | jab t = 22; jump kick missed | same |
| B40 a1 var 0x10 | X 0x6E0 Y 0x20 | 831 | box [X+16, X+48]; jab from X 1728 misses, jump kick hits at t = 32 | HP 8 | jump kick | same: the pipe tops go, steam puffs (r2:c-b40-v10-jk picture); part 1 open item 3 closed |
| B45 vents | E8 a1 six vents | 824 and above | within about 40 pixels in front of the box (X [X+2, X+30]) | HP 48; Jack 10, 10, 15, 15 | jab combo | destroyed socket, no more flames |

Rule for the port: for the box test the prop box depth range is [Z + d0, Z + d0 + d1) from the box record; a hero attack hits when `heroZ + 8 >= start` and `heroZ <= end`, plus the X and height overlaps of the box test 0x128F8. Prop HP only matters for B45.

### 2.7 Caption banners (B66) of these episodes

Text is the tall font printed at scroll1 tile (x, y) by 0x84F72 from the string table 0x87710 (glyph code 0x60 + letter, then 0x40 0x40 ends a string; 0x2F 0x2F starts the next line). Durations measured as the frames between state 0x0204 and the end state. [R 0x86D76; M r1, r2, r3]

| Var | Text | Where | Trigger | Frames |
| --- | --- | --- | --- | --- |
| 0x60 | COAL MANE (the ROM spells MANE) | E6 a0 | camera 0x390 | 80 to 88 |
| 0x61 | JUNGLE | E6 a1 | 0 | 120 |
| 0x62 | CITY IN THE / SEA (two lines) | E7 a0 | 0x70 | about 100 |
| 0x63 | LIBRARY | E7 a2 | 0x1B0 | not captured |
| 0x64 | COMPUTER ROOM | E7 a3 | 0x230 | 110 |
| 0x65 | BIO LAB | E8 a0 | 0x70 | 80 |
| 0x66 | UNDERGROUND | E8 a1 | 0 | 120 |
| 0x67 | CAVE | E8 a2 | 0 | 120 |
| 7 | (blank string) | E8 a3 | 0x10A8 | 240 |
| 0x50, 0x51, 0x52, 0x53 | the three talk lines of the Tyrog scene and a clear (paraphrase: he says they cannot go further, that he will show them something incredible, that it is the power of the doctor) | E6 a2 by B79 | scene | about 120, 140, 110 (at B79 + 0, +120, +350; 0x53 clears at +470) |

### 2.8 The tile clock G.$7AE

Read by B10, B28, B29, B36, B37, B38 (and others): a class rewrites its tiles when G.$7AE differs from G.$7AF and then sees G.$7AF catch up one frame later. Measured in three windows of Episode 8 area 1 (31 changes each): **G.$7AE steps 0, 4, 8, 12, 0 ... every 24 game frames, with the steps at gf = 1284, 1308, 1332 ... (value 0 at gf 1284)**: `7AE = ((gf + 60) / 24 mod 4) * 4` where gf is the game frame counter G.$0000. G.$7AF follows after 1 frame. The writer is not a direct store in the code (part 1 open item 2): the value is a pure function of the frame counter. [M r1:e8a1-idle-0, -2D0, -A70; R reads at 0x7862A and 24 more]

### 2.9 The three kinds of door burst

| Class | Block | Writes | Effect object | Sound | Enemy entrance |
| --- | --- | --- | --- | --- | --- |
| B4 vars 4, 8, 0xC | 4 columns x 6 rows (var 4 cols 34..37, var 8 cols 42..45, var 0xC cols 59..62, rows 52..57) | 24 codes + 4 attributes | C21 var 0x16 (planks) | break 0x9E60 | wave group: Ferris var 6 or 0x206 at X + 0x20, Z 0x60, 0x50 frames after the burst |
| B42 vars 0, 4, 0xC, 0x10 | 4 columns x 6 rows (cols 28..31, 56..59, 24..27, 52..55, rows 2..7) | 24 codes | C21 var 0x26 | break 0x9E60 | dark red Ferris variants fall out of the doorway |
| B52 | 10 columns x 7 rows (cols 47..56, rows 49..55) | 55 codes + 70 attributes | C21 var 0x20 (books) | break 0x9E60 | Ferris var 6 then 0x206 at X 0x730, Z 0x70 |

All three are spawned by the wave record (delay 0) and live 2 to 3 frames: routine 0 waits 1 frame (+A0 = 1), routine 2 swaps the tiles, plays the sound, makes the effect, routine 4 frees. The doors are drawn as normal map tiles before the burst. [R 0x767D8, 0x83A0A, 0x85178; M r4:tc-b4v4, tc-b4v8, tc-b4vC, tc-b42v0, tc-b42v4, tc-b42vC, tc-b42v10, tc-b52]

### 2.10 Animations by area (all measured sprite free, 240 to 800 frame windows)

| Area | Tile animation | Palette animation | Other moving scenery |
| --- | --- | --- | --- |
| E6 a0 cabins and mine | none | lamps B60: palettes 80, 81, 36 frame cycle | doors and wall by events (B4, B3) |
| E6 a1 jungle | water B8 (rows 42..49, 24 frames), eye B7 (2 x 2 tiles), rain streaks B55 (scroll1, every frame), bolt strips B9 | lightning flashes B9 (scroll3 palettes 100 to 108, scroll2 palettes 64 to 95) | rain drops B56 (pool C 2), the foot A22 |
| E6 a2 end of forest | water B39 (rows 43..47, 24 frames) | none | Tyrog scene B79 |
| E7 a0 terrace | none | none | none (captions only) |
| E7 a1 lift | floor labels B57 (written once), cage smash B54 (once) | none | posts B64 (sprites), rider falls |
| E7 a2 library | none | none | fences B1, bookshelf B52 |
| E7 a3 computer room | console lights B41 (3 sets, periods 19, 7, 13) | lab lights B61, B63 (palettes 66 to 91, 17 frames) | monitor B30 |
| E8 a0 bio lab | none (the tanks do not bubble) | none | tank breakers B44, manhole B35, B47 |
| E8 a1 sewers | water bands B28, B29 (rows 12..14, tile clock), torches B43 (8 frames), waterfall B36 to B38 | none | vents B45 + flames A83, doors B42 |
| E8 a2 cave | none | fire glow B53 while a hero burns | falling barrels, Zeke |
| E8 a3 lair | none | fire glow B53 | showers after the boss |

The blinking "JOIN-IN" and "INSERT COIN" prompts of the HUD (scroll1 rows 36 and 37, palette 42) and the GO arrow (scroll1 cols 36..40 rows 46..49, palette 42, B20) show in every idle window and are not scenery. [M r2:a-*, r3:a-*]

## 3. Area walk, Episode 6 (internal stage 5): Jungle and Mine

**Method.** Same as part 1: `jump 6 N`, `god on`, sweeps with `hold` kill-all (three passes in area 0 and 1, one in area 2), natural runs, idle windows, then targeted tests by poke. Camera events and waves are in [D4c](D4c-camera-flow-transitions.md) 2.3 and 4; drops in [D4a](D4a-items-drops.md) 8; the area start times and walk-in scenes in D4c 5 and 6. [M r1:e6a0-sweep-*, e6a1-sweep-*, e6a2-sweep-mid, r2:a-e6a*]

### 3.1 Area 0: the cabin street and the mine (map 0x800 wide; lock 0x200, lock 0x730, end fade 0x780)

| What | Where | Trigger, how | Before and after | Drops | Effect on players or enemies | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Ruined cabin street, brick wall, mountain and cloud sky | X 0 to 0x340 | none | static; no tile or palette change in 300 frames | | none | map, scroll3 | M r2:a-e6a0-cab |
| **Green cabin door** | X 0x220, Z 0x60, rows 52..57 | wave 0 starts when the camera reaches the lock 0x200 (t = 100); the object is made 2 frames later | closed plank door becomes a black doorway, planks fly (C21 var 0x16), break sound; after 80 frames the first group steps out (Ferris var 6 or 0x206 at X 0x240, and a Ferris var 0x402 walks in from the left at X 0x1E0) | | the heroes are not touched | B4 var 4 | M r1:e6a0-door1 (t = 102 spawn, enemy in the doorway at t = 186), r4:tc-b4v4 (24 codes, 4 attributes) |
| **Large double door** | X 0x2A0, Z 0x60 | second group of wave 0, after the first group is down (t = 268 with the hold) | same burst, group 2 steps out | | | B4 var 8 | M r2:n-e6a0-doors, c-b4-door8, r4:tc-b4v8 |
| Wave 0 end | lock 0x200 | the last enemy dies | time is set to 3:50 and GO | | | B15 wave 0 | M |
| **Mine entrance wall** (the "wall punched open" of the blueprint, L09) | X 0x350, Z 0x60, a column of planks at the cave mouth (3 tiles wide, 9 tall) | any player attack box with hero depth Z 32 and more: jab, jump kick, 360 special | planks wall becomes the rock mouth, planks fly 40 frames, C1 spark then 11 C21 pieces, break sound; before it the wall **stops walking** (a hero walking right stops at X 846) | none | a punched wall does not hurt anyone | B3 var 0 | M r1:e6a0-wall-z16..z79, -jk, -walk, r2:c-b3-special, r4:tc-b3v0 |
| **Mine lamps** | scroll2 palettes 80, 81 | from camera 0x200, whole area | the hanging lamps of the mine flicker dim to white, 36 frame cycle | | none | B60 (2.2) | M r1:e6a0-lamps, r2:a-e6a0-mine |
| Mine tunnel: metal ribs and arches, rock walls, a pile of rubble with a pickaxe | X 0x348 to 0x7FF | none | static | | none | map | M r2:a-e6a0-mine (no tile changes) |
| Caption "COAL MANE" | bottom left | camera 0x390 | white tall font text for about 80 frames | | | B66 var 0x60 | M r1:e6a0-door2 t = 60 to 88 |
| Seven barrels in a row (small food, treasures, grenade) and a gold bar | X 0x440 to 0x5C0 Z 0x50 (barrels, triggers 0x290 to 0x410), gold bar X 0x3C0 Z 0x10 | punch breaks (HP 4) | | rows 0x1B, 0x29 x4, 0x15, 0x0F (D4a); the gold bar is E9 var 1 | | E5 var 4, E9 var 1 | M r1:e6a0-sweep-mid pictures |
| **Mine wooden door** | X 0x7B0, Z 0x60 | last group of wave 2 (after Lash T. and Walther) | same burst; three poachers step out (A13 var 0x404, 0x408 at X 0x7D0, 0x8D0) | | | B4 var 0xC | R wave 2; M r4:tc-b4vC (24 codes) with the object spawned by hand. The run through the whole wave was blocked by the mid bosses (1.3) |
| Wave 2 end and **scripted exit** (script 18) | lock 0x730, end event 0x780 | wave 2 ends (type 2) | B20 GO blink, wait for floor items (up to 180 frames), GO, scripted walk right through the mine exit 300 frames, GO, wait 60, fade (+266 frames after the wave end), the area 1 starts | | heroes are scripted (G.$4DD), nothing breaks | script 18 | M r3:s-e6a0-k18 |

Scenery animation of area 0: the lamps only. No falling objects, no water. [M r2:a-e6a0-cab, a-e6a0-mine]

### 3.2 Area 1: the jungle, the foot and the swamp (map 0xB80+, camera Y 0x100, lock 0x300, lock 0x8C0, slope 0x930, sub area switch 0xAE0)

| What | Where | Trigger, how | Before and after | Drops | Effect on players or enemies | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Exit wall of the mine** | X 0, Z 0x130, Y 0x20 | area load; start script 17 walks the heroes right | a plank and rock wall at the left edge; 48 frames after the area load it breaks by itself, planks fly (C21 var 0x14), the heroes walk out | | no sound | B3 var 8 | M r3:s-e6a0-k18 (B3 at t = 320 to 360), r4 |
| Caption "JUNGLE" | bottom left | area load | 120 frames | | | B66 var 0x61 | M r3:s-e6a0-k18 |
| **Storm**: rain streaks, drops with splashes, lightning (2.3) | whole screen | rain from camera 0x100 to 0x500, drops from 0x80 to 0x580, lightning objects by far layer position | rain falls diagonally over everything, drops splash on the floor, flashes of orange and grey light with a bolt in the sky and thunder | | none | B55, B56, B9 | M r2:a-e6a1-storm, n-e6a1-lightning, r1:e6a1-idleA |
| Big foreground vines (green trunks with leaf curtains) | X about 0x276 and 0x46A | none | static | | none | map | M r1:e6a1-sweep-mid |
| Treasure | X 0x90 Z 0xF0 | trigger 0 | on the path at the start | E9 var 1 | pick up | E9 | R |
| **Six rolling barrels** | X 0x160 (Z 0xF0, 0x110, 0x130, items 0x15, 0x29, 0x1B) at trigger 0x180 and X 0x260 (items 0x29, 0x28, 0x29) at trigger 0x280 | made at the left edge (camera X - 0x20), roll right 2 pixels per frame, hop every 26 frames (D3a 10.1) | a barrel hit by a hero breaks and drops its row item (grenade, dime or treasure, pizza); one that touches a hero or an enemy hurts for 10 | rows C:0x15 gold dust / dime, C:0x29 treasure, C:0x1B grenade, C:0x28 pizza | hurt 10 raw to enemies too | A65 var 1 | M r1:e6a1-barrels-slow (a barrel broken at t = 556 left treasure var 5), -fast |
| **Barrels and the slow hero** (N10) | | the barrel is freed with its item when it is 0x40 left of the camera or 0x200 right of it | slow play (hero 1.5 pixels per frame): the hero reaches the barrels and breaks them; a dash (camera 3.4 pixels per frame): the barrels fall behind and one was gone after 24 frames | | | despawn window (D4a 14) | M r1:e6a1-barrels-slow, -fast |
| **Brontosaurus feet** | X 0x6C0 Z 0x100 (trigger 0x520) and X 0x950 Z 0x100 (0x7B0) | the foot rises, hovers 180 frames over the nearest hero (80 pixel column), waits 60, slams (30 damage, knock down, screen shake 4), rests 30; period about 380 frames | a pale green leg with toes comes from the top of the screen; **a black oval shadow lies on the floor under the foot at its X and Z while it is in the air** and follows the hero; the shadow is under the heroes when the slam lands | | hurts heroes (30) and enemies (30, D3a h-foot-poacher) | A22 + A23 (D3b section 6) | M r1:e6a1-foot-god (pictures every 6 frames), e6a1-foot-hit |
| Poacher ambush from the trees | X 0x2C0 to 0x920 (6 records, entrance 0x12) | camera triggers 0x200 to 0x800 | D3a | | | A13 | R |
| **Swamp**: green water, wading heroes, the big pale reptile head in the background | camera 0xA00 to 0xB80 | none | water ripples every 24 frames (B8), **the eye of the head opens (red iris) and closes** every 5 pictures + 96 frames (B7) | | heroes wade (draw only) | B8, B7 var 0 | M r2:a-e6a1-swamp, head zoom |
| Wave 6 end | camera 0xAE0 | 3 poachers, 2 Bludge, 4 Hammers | sub area switch (no reload), walk into area 2 with script 0; sets 1:20 on arrival | | | B15 wave 6 | D4c |

Waves of area 1: wave 4 at the lock 0x300 (2 Blade or Razor), wave 5 at 0x8C0 (4 Hammers, time 2:50), wave 6 at 0xAE0. [R stage_scripts; D4c]

### 3.3 Area 2: the end of the forest and the Tyrog scene (diagonal climb 0xE00 to Y 0x170, lock 0xF00, stage end 0x1000)

| What | Where | Trigger, how | Before and after | Drops | Effect | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Swamp water in front of the path** | rows 43..47, two ring blocks | while the camera Y is below 0x170 | 4 pictures, one per 24 frames | | none | B39 | M r2:a-e6a2-water |
| Salad barrel | X 0xEA0 Z 0x160 (trigger 0xD00) | punch | | row C:0x0C salad; can be carried to the boss (like the meat barrel of Episode 8) | | E5 var 4 | R |
| Wave 7 and the boss area | lock 0xF00 | 1 Ferris, then time 3:50 and type 4 (boss area script 11) | heroes walk to the line up spots | | | B15 wave 7 | M r1:e6a2-boss-intro (B15 at t = 110) |
| **Tyrog scene** | X 0x1120, Z 0x190 (record trigger 0xF80) | after the line up | three gangsters carry a metal box; captions (paraphrase): you cannot go further, I will show you the incredible, the power of the doctor; white smoke, the lid opens, **Tyrog (green tentacled creature) jumps out** and the fight starts | | | B79, A75 actors (var 0 to 9), A76 creature maker, B66 vars 0x50 to 0x53, sound 0x21 | M r1:e6a2-boss-intro: B79 at t = 350, state steps at 350, 470, 700, 820; Tyrog A4 var 0x200 at t = 904; B22 at 910 |

The fights of Tyrog are in D3d (section 3 is written there when D3d is finished; the chain of the three bodies is in its notes A). [D3d]

## 4. Area walk, Episode 7 (internal stage 6): The Vault

**Method.** Same as Episode 6, but **no `hold` kill-all** (D4c saw a crash with it): areas were walked naturally or with single `killall` pulses; the lift was run twice (natural bot, and with pulses). The start of each area was taken from the natural run of the area before. [M r1:e7a*, r2:n-e7a0-end, n-e7a1-full, l-wave4, n-e7a2-end, n-e7a3-talk, r3:s-e7a0-k16, a-e7a3-b41]

### 4.1 Area 0: the terrace and the bridge in front of the vault (map 0xB00 wide; locks 0x1B0, 0x580, 0x790, 0x930, end fade 0x9E0)

| What | Where | Trigger, how | Before and after | Drops | Effect | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Evening skyline**: the flooded city towers and the dome building of Episode 1, in an orange sunset palette | far layer (scroll3) | none | static (the user and the blueprint call it the Episode 1 roof in evening colours: the skyline art is the same, the foreground is not) | | none | scroll3, palettes | M r2:a-e7a0-bridge, a-e7a0-gate: 0 tile and 0 palette changes; assets ep1_a0 and ep7_a0 pictures |
| Stone terrace with a balustrade and **a stone bridge** (an arch over dark water below) | X 0 to 0x580 | none; a hero cannot fall off | static; bikers (A70) and Zeke (A82) cross it | | | map | M r1:e7a0-nat sheet |
| Caption "CITY IN THE SEA" (two lines) | bottom left | camera 0x70 | about 100 frames | | | B66 var 0x62 | M r1:e7a0-nat (t = 460 to 560) |
| Tower with an arched doorway and a rock wall with metal ribs (the vault front) | X 0x5C0 to 0x9E0 | none | static; the Hammers of wave 1 step out of the doorway area at X 0x5F8 to 0x710 (plain spawn, no door break) | | | map | M r1:e7a0-nat |
| Weapon on the floor: a bazooka | X 0x720 Z 0x20 (trigger 0x590) | walk over | | E0 var 0x1C | | E0 | D4a |
| Barrels (Uzi, small food, hot dog by player count) | X 0x5C0 to 0xA80, Z 0x4A to 0x6A | punch | | rows 0x0F, 0x21, 0x06 or 0x2C, 0x25, 0x10 | | E5 | D4a |
| **End of the area**: walk to the lift (script 16) | lock 0x930 (wave 3), end event 0x9E0 | wave 3 ends (type 2) | B20 blink, items, GO, scripted walk to the pad X 2656, 2688, 2720 (Z 72, 52, 32), a scripted jump to the right (+0x100), GO, wait 180, fade; the picture is a wooden frame with a mesh ramp: the top of the cage | | heroes scripted | script 16 | M r3:s-e7a0-k16 (enemies still fighting in this poked run), r2:n-e7a0-end |

No tile or palette animation and no breakable in the area. [M r2:a-e7a0-*; r1 sweeps]

### 4.2 Area 1: the lift (vertical map, 56 screens; camera mode 5; camera Y from 0x3700 to 0)

| What | Where | Trigger, how | Before and after | Effect on players or enemies | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- |
| The cage | fixed on the screen | the whole area | wooden frame (posts B64, rails and floor mesh scroll1), stone and rock shaft behind scrolling by (scroll2 labels, scroll3 far layer at camY/16 - 0x60) | the hero clamp is the screen (X 128 to 287, Z 32 to 79) | camera mode 5, B64 | D4c 7.2 |
| **Ride** | | start: sound 0x86, shake 2; speed from 0.19 to 6 pixels per frame (acceleration 6/256); at 0x300 pixels before the next stop it brakes | stops at camera Y 0x3000, 0x1E00, 0x1400, 0x800, end at 0 (fade); after every lock the ride restarts from rest | | camera | M r1:e7a1-nat: stops at t = 560 (Y 12288), 1850 (7680), 3060 (5120), 4500 (2048), end at 5320 |
| **Floor labels** L15 down to L1, B1 to B10, blank, B21 to B40 | cage wall, scroll2 | painted as the camera passes (2.5) | white tall letters on the stone wall; "L-8" at the first stop, "B-33" at the last | none | B57 | M r1:e7a1-nat, r2:n-e7a1-full |
| **Harbour window** (the blank zone) | camera Y 7776 to 5216 | | the stone wall opens to a harbour of the sea city: ships, a submarine, cranes behind a window with posts; fights go on in front | none | map data + far layer | M r1:e7a1-nat t = 2460 to 2940 |
| **Cage walls smashed** | lower left and lower right corners | wave 4 (first stop), 80 frames after the wave start | corner braces vanish, planks fly, two enemies jump in | none | B54 | M r2:l-wave4 (t = 616) |
| Riders and their entrances | both sides (X -48, 432), from above, inside at start | records by camera Y; waves 4, 6, 7, 8 at the stops | see 2.5 | enemy shadow marks the landing spot of the ones that fall (D3a entrance 0xA) | A2, A10, A13, A14, A20 | M r2:n-e7a1-full |
| Dynamite thrown by a Punk | stop 3 area | | red stick with a burning fuse, explosion "BOOM" in the cage | hurts heroes (D2c, D3a) | A20 | M r1:e7a1-nat t = 3060 picture |
| Clock | | wave 4 sets 3:50, waves 6 and 7 set 1:50, wave 8 sets 3:50 | | | wave scripts | D4c |
| **End of the ride** (script 15) | Y 0 | wave 9 (no spawns) with type 2 | B20, items, GO, wait 300, fade, area 2 starts after about +200 | | script 15 | D4c 5.7 (not measured there), M r2:n-e7a1-full: end t = 4790 |

Scenery animation: labels are written once as the camera passes, nothing else changes except the cage break. Window zone, labels and the posts are map data and three small classes. [M r2:a-*; R]

### 4.3 Area 2: the library (map 0x900 wide; locks at X 0, 0x1D0, 0x640 (push 0x128); fade at 0x700; start script 9)

| What | Where | Trigger, how | Before and after | Drops | Effect on players or enemies | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Caption "LIBRARY" | bottom left | camera 0x1B0 | | | | B66 var 0x63 | R (not captured) |
| **Gate 1: chain-link fence** at the start (the library entrance) | X 0xE0 Z 0x80 | any player attack box (hero Z 64 to 100, X reach to the box 288 to 368), at any time (also during the lock at X 0) | the fence whole becomes the broken fence (sprite 327), planks and wire fly (C21 var 0xC), the heroes can pass | none | the break changes priority attributes only; the handler does not rewrite terrain, so walking is decided by the stage terrain data (part 2 says the same for the Episode 4 gate; walking into the unbroken fence was not tested here) | B1 var 4 | M r1:e7a2-fence1-z64/79/100 (t = 22), r4:tc-b1v4 (48 attribute words), r1:e7a1-nat (broken at t = 7520 by the bot) |
| **Bookshelves, scrolls machines, books on the floor** | whole area | none | the shelves are background: no tile or palette change in 240 frames at X 0x100 and 0x600; nothing is spawned from them except the one below | | none | map | M r2:a-e7a2-a, -b |
| **Rolling barrels** | X 0x1E0 Z 0x20 .. X 0x5E0 (10 records, from both screen edges) and 3 in wave 12 | trigger by camera X | roll along the floor, 2 px/frame, hop; items: small foods, treasure, a rod (Uzi row 0x12), special, dynamite; pudding and diamond barrels in wave 12 | rows 0x22, 0x03 x3, 0x29 x3, 0x12, 0x17, 0x2A, wave 12: 0x1D, 0x30, 0x27 | hurt 10 | A65 var 0 | D3a 10.1, D4a |
| Three Bludges leap in from the right | X 0x740 to 0x780 | triggers 0x580 to 0x5C0 | | | | A6 | D3a |
| **Bookshelf bursts open** (wave 12) | X 0x730, Z 0x70, 10 columns x 7 rows | wave 12 (lock 0x640, push 0x128): the first record of the wave, with its delay 0x1E0 (B52 spawned at t = 876 although the wave began at t = 110: the group waits for the count rule, the earlier enemies were dead at t = 430) | books fly out, black gap in the shelf, a Ferris (var 6, then 0x206) walks out of the gap | | enemies only | B52 | M r2:n-e7a2-end (pictures t = 848 to 968), r4:tc-b52 |
| **Gate 2: second fence**, the way to the computer room | X 0x7C0 Z 0x80 | wave 12 ends (type 2), end script 19: B20, items, idle, GO, scripted walk to X 2048, 1984, 1920 (300 frames), wait 20, **the leader walks to X 2039 and punches: the fence breaks at once**, wait 60, walk, hold right 120, GO, wait 180, fade | fence whole becomes broken as the heroes arrive, they walk through | none | | B1 var 8, script 19 | M r2:n-e7a2-end (GO t = 1500, the hero at X 2039 at t = 1550, punch and break t = 1560, fade t = 1630, area 3 t = 1720), r1:e7a2-fence2-z64/100 |
| Items | three treasures E9 (var 1, 0xE, 0xB) at X 0x2B1, 0x45B, 0x5C1 Z 0x10 | | | | | E9 | D4a |

No tile or palette animation in the library. [M r2:a-e7a2-a, a-e7a2-b]

### 4.4 Area 3: the computer room (map 0x600; follow, lock 0x240, lock 0x450, stage end with fade 0x4F0; start script 9)

| What | Where | Trigger, how | Before and after | Drops | Effect | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Caption "COMPUTER ROOM" | bottom left | camera 0x230 | 110 frames | | | B66 var 0x64 | M r2:n-e7a3-talk |
| Barrels (small foods, ammonite, club, sushi or hot dog by level) | X 0x100 to 0x1C0 Z 0x6A | punch | | rows 3, 0x1F, 4, 0x32 or 0x33 | | E5 | D4a |
| **Console lights** | ring cols 0..2 rows 50..51, cols 10..17 row 49, X about 0x400 to 0x530 | while the camera is within 0x100 of X 0x430 (camera 0x330 and more) | three small light strips blink (periods 19, 7 and 13 frames) | | none | B41 vars 0, 4, 8 | M r3:a-e7a3-b41 (changes every 6 frames at the 3 frame sample) |
| **Lab lights** | scroll2 palettes 66 to 91 | all the area, from trigger 0x200 | the colours of the computer walls and screens cycle: palettes 84, 85, 86, 80 in 3 steps, 10 other palettes in 2 steps, one step every 17 frames; sound 0x6D every 121 frames | | none | B61, B63 | M r2:a-e7a3, r3:a-e7a3-b41 (14 change frames in 240) |
| **Fessenden on the monitor** | X 0x4B0, Y 0xD0 | the camera at 0x420 with the Slisaurs standing: B30 spawns (t = 410), the black screen shows static (t = 470 to 590), then the doctor's face with a three line caption box (t = 590) that boasts that the heroes cannot stop the maker of a new world (paraphrase), sound 0x23 (talk bed) | black screen, snow, face | | the fight starts after the talk (B33 reads G.$4FC) | B30, B34, B33 | M r2:n-e7a3-talk (pictures t = 336 to 750) |
| Slisaur fight | lock 0x450 | | | | | D3d section 4 | D3d |
| Stage end (script 20) | 0x4F0 | boss death | pose, tally, walk to X 1488, 1520, 1456 (Z 24), two hops, fade | | | script 20 | D4c |

## 5. Area walk, Episode 8 (internal stage 7): The Under World

**Method.** Same as Episode 6 (hold kill-all is safe here), plus the sprite free peeks. The tanks of area 0 and Dr. Fessenden are in D3d; everything else is below. [M r1:e8a*, r2:a-e8a*, n-e8a*, c-*, r3:m-e8a2-meat, ee-f84-kill, r4:b53-*, tc-*]

### 5.1 Area 0: the bio lab (map 0x700 wide; locks 0x120, 0x2D0, 0x480 (push 0xA0), end fade 0x580)

| What | Where | Trigger, how | Before and after | Drops | Effect | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Caption "BIO LAB" | bottom left | camera 0x70 | 80 frames | | | B66 var 0x65 | M r1:e8a0-end |
| **Eleven glass tanks** numbered 01 to 11 (green lizard man, pink brute, orange raptor repeated), pipes, steel floor, no animation at all | the whole wall | none; the tanks 01, 03, 04 and 11 are opened by their breakers B44 | D3d section 5: glass shards (about 40 frames of fall), a Bludge, a Rock Hopper, a second Bludge and Tyrog 2 come out, the empty tank stays with broken glass; the glass pieces lie on the floor as small blue shards | | | B44 (D3d) | M r1:e8a0-sweep-mid; r2:a-e8a0-a, -b: 0 tile and 0 palette changes |
| Three dynamite carriers (Punk and Wrench T. alt) | X 0x3B8, Z 0x10, 0x30, 0x50 (trigger 0x188) | camera | | rows 0x22, 0x12 (Uzi 32 of 32), 0x22 | | A64 | D3a, D4a |
| **Steam pipes in the end wall** | X 0x690, Z 0x40 | a player attack at hero Z 56 to 72 from X 1648 | 11 tiles change, steam for ever (A15) | none | **cannot be reached in play**: the lock 0x480 holds the camera so a hero cannot go past X 0x5F0 (camera + 0x170), and the end script 21 takes the heroes when the lock opens; in the natural end run the pipes spawned at t = 610 and stayed whole | B40 var 4 | M r1:e8a0-pipe-z56..z72 (broken by a poked hero), r2:n-e8a0-end (never broken) |
| Wave 2 end (8 poachers, lock 0x480) and **the manhole** (script 21) | X 0x610 to 0x660 | wave ends; B20 blink, items, GO, scripted walk to X 1552, 1504, 1472 (Z 56), the leader pose | a round steel grate lies on the floor at X 0x637 (B35, drawn with a floor test): **the leader (standing at X 1544, Z 56, lift pose state 0x2C from t = 700 to 790) lifts the grate that lies at his feet over his head (t = 710 to 730), holds it, throws it to the right at t = 790 (it flies from X 1541 to 1680 and lies there from t = 870), a black hole shows in the floor where it lay (B47 writes two tile patches at col 33, row 59) and he jumps down it (t = 905 to 948)**; fade, the sewer starts at t = 960 | none | heroes scripted | B35, B47, script 21 | M r2:n-e8a0-end (pictures t = 708 to 948), r1 e8a0 sweeps |

### 5.2 Area 1: the sewers (map 0xE00 wide, camera Y 0x300, hero Z 768 to 831 (863 in the middle); locks 0x1A0, 0x370, 0x570, 0x840, 0xA00 (push 0xC0); slow fade 0xBC0)

| What | Where | Trigger, how | Before and after | Drops | Effect on players or enemies | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Caption "UNDERGROUND" | bottom left | area load | 120 frames | | | B66 var 0x66 | M r2:n-e8a0-end |
| Walkway of stone slabs with a low edge, **green sewer water in front** (heroes wade at the edge, splashes), rock wall with **hanging chains and shackles**, round steel nozzles, window niches with water behind, five dark red doors, torches | the whole area | none | static except the classes below | | the water is draw only | map | M r1:e8a1-sweep* |
| **Water bands**: rippling green water, 4 pictures per 96 frames | rows 12..14 in sections at X 0x20, 0x120 to 0xA20 | tile clock | 4 pictures, one per 24 frames | | none | B28 (5 records), B29 (7 records) | M r2:a-e8a1-*: rows 12..14 change every 24 frames |
| **Wall torches** | ceiling line, X 0x130, 0x2F0, 0x3F0, 0x500, 0x7B0, 0x940, 0xB10 | from the camera within 0x100 left of X | orange flame flickers: 4 pictures, one per 8 frames | | none | B43 | M r2:a-e8a1-0 (ring cols 19..20 rows 1..2 change 36 times in 300 frames) |
| **Six flame vents** (the "wall flamethrowers") | X 0x2A0, 0x320, 0x400, 0x840, 0x8C0, 0x940, Z 0x350, round gray nozzle at height 48 | cycle (2.4); a punch from the back of the walkway | small sparks, then a jet of fire shoots left and towards the viewer for 40 to 70 frames, shrinks, rests 60 to 90 frames; **hit 4 times it is destroyed** (a broken socket and sparks for 2 seconds) | none | **hurt heroes 20 (level 0), knock down (burn) and enemies 20**; a burning hero recolours the scene only in areas 2 and 3 (B53), not here | B45 + A83 | M r1:e8a1-vent-z832, r2:c-b45-z*, c-flame-*, r4:tc-b45 |
| **Three pipe sets in the wall** (the "exhaust pipes") | X 0xB0 (Y 0x30), X 0x580 (Y 0x20), X 0x6E0 (Y 0x20), Z 0x340 | a jab (the first two), a jump kick (the third with the hero held 48 pixels away) | two gray pipes with an arm: the top and the arm vanish, steam puffs for ever | none | steam is harmless | B40 vars 8, 0xC, 0x10 | M r2:c-b40-v8-jab, vC-jab, v10-jk, r4:tc-b40vC (11 tile codes) |
| **Red door that bursts open** | X 0x1C8 (wave 3), 0x383 (wave 4), 0x980 (wave 6), 0xB47 (wave 7), Z 0x370 | wave start (delay 0) | dark red door becomes a black doorway, planks fly, the enemies of the wave (dark red Ferris variants) tumble out; the fifth red door (X about 0x78D, no record) is scenery | | | B42 vars 0, 4, 0xC, 0x10 | M r2:n-e8a1-w3 (spawn t = 150, planks t = 160 to 190), w4, w6, w7, r4:tc-b42* (24 codes each) |
| Wave 5 | lock 0x570 | 4 Blade alt (A63), time 2:50 | no door | | | B15 wave 5 | R |
| Fat enemies and Blade on the walkway, poachers | X 0x1A0 to 0xBA0 | camera | | row E:0x0E hot dog or pizza (Hammer at X 0x3A0) | | A10, A13, A14 | D3a, D4a |
| **Waterfall and the plunge** (end script 22) | edge at X about 0xBB8, second fall X 0xD20 | wave 7 ends (type 2): B20, items, wait 30, GO, walk right to X 0xB80 (300 frames), a scripted jump (+0x120 X, 90 frames), wait 300 | the walkway ends in a drop: water pours over the edge (B36 at X 0xC20 and 0xD20), the shaft below shows the water column in three stacked screens (B37 and B38 at X 0xD20, Z 0x300, 0x200, 0x100); **the heroes jump and fall down the shaft**, slow fade (camera mode 9, t = 530), the cave starts at t = 550 | none | heroes scripted | B36, B37, B38, script 22 | M r2:n-e8a1-end (B36 at t = 452, B37, B38 at t = 508, pictures t = 416 to 544) |

### 5.3 Area 2: the cave (map 0x1400 wide, slopes down at 0x100 and 0x400, locks X 0, 0x730, 0x900, sub area switch 0xCB0)

| What | Where | Trigger, how | Before and after | Drops | Effect on players or enemies | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Caption "CAVE" | bottom left | area load | 120 frames | | | B66 var 0x67 | M r2:n-e8a1-end |
| Cave of twisting tree root arches, a blue pool seen through the roots, rock slopes | whole area | none | static: no tile or palette change at X 0, 0x700, 0xC00 in 300 frames | | | map | M r2:a-e8a2-* |
| Three barrels at the start | X 0x130, 0xF0, 0xB0, Z 0x110 | punch | | rows 3 small foods, 6 Uzi (by level), 0x29 treasure | | E5 var 4 | D4a |
| **Falling barrels** (the "barrels rolling down the slope") | 11 records X 0x150 to 0x73F, one per 0x80 of camera X, Z 0x120 to 0x160, var 0x201 or 1 | camera trigger; made at X = trigger + 0xBF | the barrel starts 0x100 above the floor, a black **shadow oval on the floor shows where it will land**, falls, then hops and rolls to the right at 2 pixels per frame | rows 0x29 x9, 0x03 x2 (D4a) | hurts for 10; **freed when it is 0x200 right of the camera or 0x40 left of it** (measured: a barrel was freed at X 2219 with the camera at 1707); a locked camera therefore loses the barrels to the right after about 160 frames, a running camera loses them to the left | A65 var 0x201 | M r1:e8a2-sweep-500, r2:n-e8a2-slow, -fast, D3a 10.1 |
| Two pterodactyls | X 0x2C0 (trigger 0x200) and X 0x4C0 (0x400), leave at camera X 0x72F | | fly in at the top, dive | | Zeke: 1 hit kills (D3b) | A82 | D3b |
| **Fire glow** | 27 scroll palettes | a hero lies burning (G.$7B2 = 2) | the cave turns red and sandy while he lies, back to normal when he stands (2 frames delay) | | | B53 | M r4:b53-a2 (poke at t = 30, palettes change at t = 32, restored at t = 92, 370 colours) |
| **Meat barrel** | X 0xCB0, Z 0x110 (trigger 0xB10) | a hero next to it presses attack and right (the lift): he lifts it overhead, state 0x14 | a barrel of food; broken it gives row C:0x0B (barbecue or similar); **carried it stays in his hands through the sub area switch, the scripted walk to the lair and into the lair** (held at X 4747, 2600 frames later, nothing happened to it); thrown (attack) it flies about 70 pixels and breaks | | | E5 var 4 | M r3:m-e8a2-meat (lift t = 212, held to the end), m-e8a2-meat2 (thrown, broke at t = 272 with a food object) |
| Wave 11 and the way to the lair | camera 0xCB0, push 0x78 | 4 poachers + 4 Blade alt (type 4: stage exit script 23) | the camera switches to area 3 with no reload, the heroes walk to X 3808, 3776, 3744 (Z 344), then to X 4352 and to the line up spots; wait for the boss intro | | | script 23 | M r3:m-e8a2-meat (area 3 at t = 1330, lock 0x1120 at t = 1440) |
| Lair door and dais | a metal bridge piece and a green machine with steps (map art) at X 0x1190 | none | the end of the cave, the dais of the lair | | | map | M r1:e8a2-sweep-A00 |

### 5.4 Area 3: the lair (lock 0x1120 (push 0x80), stage end 0x1180)

| What | Where | Trigger, how | What | Class or routine | Verified |
| --- | --- | --- | --- | --- | --- |
| Fire glow | as area 2 | G.$7B2 | B53 | B53 (trigger 0xD60) | M r4:b53-a3 |
| Blank caption (240 frames) | | camera 0x10A8 | prints nothing | B66 var 7 | M r3:m-e8a2-meat t = 1260 |
| Fessenden walks in (a human in a white coat for 276 frames, then the dino form) and the dialogue box | X 0x1220 Z 0x130 | trigger 0x10A0, B50 at 0x1120 | D3d section 6 (to write) | A94, B50 var 4 | D3d |
| Metal floor and machine wall | | | static: 0 tile and 0 palette changes in 300 frames at X 0x1000 | map | M r2:a-e8a3 |
| **Self destruct showers** | made at camera 0x1140 (X 0x12C0 Z 0x160): B71 palette animation, B83 var 0x100 shower, B84 shower | after the boss: stage end, the quake | see 5.5 | B71, B83, B84 | R; M r3:ee-f84-kill (B83 and B84 objects) |

### 5.5 What follows the death of the last boss (pool B objects seen in the D3d replay)

Replay of `f84-kill` with pool B watchers [M r3:ee-f84-kill]: the boss dies at about t = 7296; then the ending code makes B82 (t = 7300, controller), the debris showers B84 var 0 (t = 7304 to 7704, 400 frames: A93 pieces every 32 frames), B83 var 0 (7304 to 7504, 200 frames: A92 pieces every 4 frames and the rumble sound every 16), B83 var 0x100 (7544, about 50 frames), B83 vars 0x700 and 0x600 (7564, about 190 frames), the strong quake B85 var 0x100 (7848 to 7944), the captions B66 var 0x69 (HURRY UP!!, 120 frames at 7308), 0x6A (HANNAH!, 7560), 0x6B (JACK! HANNAH!, 7952), the staff roll B87 (8076 to 11260), the ending desert B81 (8272 to 12284), then B89, B86, B94, B93 of the continue and profile screens. These belong to the ending: D4d owns the scene, the table of 2.1 gives the class roles. B71 never appeared in this replay (its script trigger 0x1140 is past the camera lock).

## 6. Checks of the web leads and the user notes

| Note | Verdict | Evidence |
| --- | --- | --- |
| G11, L09: a wall at the mine start is punched open; doors in the mine that the player must break | **The wall is true** (B3 var 0). The mine **doors are not broken by the player**: they burst by themselves at a wave start (B4) | 3.1 |
| G11: the brontosaurus foot casts a warning shadow and can crush enemies | **True**: a black oval under the foot, and the foot hurts enemies for 30 (D3a) | 3.2, D3b section 6 |
| N10: two rolling barrels carry items only if the player goes slowly | **True as a despawn rule**: all six jungle barrels carry a row, but a barrel is freed with its item 0x40 behind the camera; a dash outruns them | 3.2, 5.3; D4a 14 |
| N12, L11: running down the cave slope removes the pterodactyls and most rolling barrels; the scroll lock removes barrels | **True and explained**: barrels roll right at 2 pixels per frame and are freed at camera X + 0x200 or camera X - 0x40; Zeke has a leave X of 0x72F; at a lock the camera stands still and barrels roll off to the right | 5.3 |
| G13, N11: Episode 7 has a bridge, bikers and flying creatures at the start | **True**: stone bridge terrace, A70 bikers, Zeke | 4.1 |
| N11, L10: small thugs come out of the bookshelves; the user: the shelves are background | **Nearly**: all shelves are background except **one shelf at X 0x730 that bursts open in wave 12 and lets a Ferris out** (B52); the barrels roll in from both edges | 4.3 |
| L10: the rooftop is Episode 1's roof in evening colours | **Half**: the skyline is the same art in orange; the foreground is a different stone terrace and bridge | 4.1 |
| L10: the elevator goes down about 40 floors and stops for waves | **True**: labels L15 to L1, B1 to B40 (blank for B11 to B20), four stops | 2.5, 4.2 |
| L10: the library and the computer room are each reached by breaking a gate | **True**: fence B1 var 4 (by the players), fence B1 var 8 (by the leader of the end script) | 4.3 |
| G14, L11, N12: wall flamethrowers in the sewer are destroyed by punching; wall exhaust pipes can be destroyed; pipes blow fire that hurts | **True**: vents B45 take 4 punches, hurt 20 and knock down; the pipes B40 break with one punch (the third set needs a jump kick from far) and then blow steam that does not hurt | 2.4, 5.2 |
| M26: the sewers have handcuffs and chains on the walls | **True**: hanging chains and shackles in the wall, static map art | 5.2 |
| M26: the salad barrel before Tyrog can be carried to the boss | Plausible: E5 var 4 at X 0xEA0 is liftable like any barrel; not carried in a probe [I] | 3.3 |
| M15: the sauropod's head laughs in the swamp after the foot section | The eye of the head opens (red iris) and closes: B7 var 0 | 3.2 |
| L11: a meat barrel stands before the last boss and can be carried to the boss | **True**: lifted at X 0xCB0, still held in the lair 2600 frames later | 5.3 |
| M16: a gold bar appears when three enemies show up after entering the mine | The treasure E9 var 1 at X 0x3C0 appears at camera 0x380, just before the three enemies of trigger 0x390 | 3.1; D4a |
| User W25 and "pipes in walls, tanks that break": pipes in Episode 8 | Pipes: yes (B40, B45); tanks: D3d (1, 3, 4, 11 break) | 5.1, 5.2 |

## 6.1 Answer to W25 and W26 for Episodes 6 to 8

**W25 (breakable parts of the background).** Episode 6: the mine entrance wall (B3 var 0). Episode 7: the two chain-link fences of the library (B1 var 4 and var 8). Episode 8: the steam pipes (B40: three sets in the sewers, one set at the end of the lab that no player can reach), the six flame vents of the sewers (B45, HP 48). Nothing else is a breakable background part; the tanks of Episode 8 break by themselves (D3d), the barrels, drums and tires are pool E containers. Drops of the breakable parts: none (they are tiles and sprites, not containers). [R static; M r1 to r4]

**W26 (other interactive things).** Episode 6: doors that burst at wave starts (B4), the exit wall that breaks by itself (B3 var 8), lightning, rain and the lamps (scenery), the foot (hurts), rolling barrels (hurt, carry items), the Tyrog scene. Episode 7: the cage smash (B54), the riders that fall with a shadow, the bookshelf that bursts (B52), the fence broken by the leader (B1 var 8), the monitor talk (B30), blinking lights. Episode 8: red doors (B42), vents and flames (hurt 20), pipes (steam), the manhole (B35, B47), the waterfall plunge, falling barrels with a shadow, the fire glow, the meat barrel that can be carried to the boss. No switches, no signs, and no pool B hazard other than the flames (A83); the other hazards are the foot (A22), rolling barrels (A65), Zeke (A82) and the pool A fire objects. [M r1 to r4]

## 7. (a) Acceptance scenarios (probe segments and what must match)

All inputs are in `re/out/d3e3/run1.txt` to `run4.txt` (run one segment: copy its `segment` block to a file, `PR_INPUT=file re/harness/probe/probe.sh NAME`). `t` is the frame after the jump; the port runs the same lines and compares. "God" means `god on`; `hold kill-all` is `hold 0xFF850E 1 2`. Pool B slot numbers vary, class numbers, states (`st`) and positions do not.

| # | Segment and core lines | What must match |
| --- | --- | --- |
| A01 | `r1:e6a0-wall-z32` (also z48, z64, z79, z16): `jump 6 0 0x2C0`, god, poke hero X 0x32C and Z, `20-22:A`, watch B:3 | B3 (X 848, Z 96, HP 8) goes `st 02000000` to `04000000`, HP 0, at t = 22 for hero Z 32, 48, 64, 79; stays 02000000 for Z 16. C1 at t = 22, C21 pieces var 6 to 0xD at t = 24 (about 11) |
| A02 | `r1:e6a0-wall-jk` (hero X 0x300 Z 48, `10-40:R`, `14-15:J`, `22-24:A`); `r2:c-b3-special` (`20-22:AJ`) | break at t = 24 (state 06000000); special: break at t = 50 |
| A03 | `r1:e6a0-wall-walk` (hero X 0x330 Z 48, `4-260:R`, no attack) | the hero stops at X 846 and stays; the wall stays 02000000 |
| A04 | `r4:tc-b3v0`: peek scroll2 at t = 10 and 130 | 31 tile code words and 37 attribute words differ, all in cols 52..58, rows 49..62 |
| A05 | `r2:n-e6a0-doors`: `jump 6 0 0x1C0`, god, hold kill-all, `0-300:R` | camera 512 and lock at t = 100; B4 var 4 at (544, 96) seen t = 102, enemy in the doorway about t = 186; B4 var 8 at (672, 96) t = 268; `r4:tc-b4v4`, `tc-b4v8`, `tc-b4vC` (spawn B 4 by hand at t = 2): 24 code words and 4 attribute words change, in cols 34..37, 42..45, 59..62 rows 52..57 |
| A06 | `r1:e6a0-lamps`: `jump 6 0 0x400`, `peek 0x914A00 48` every 4 frames | palette 80 colour 0 (4 frame samples) is F321 at t = 0, 4, 8; F661 at 12, 16; F541 at 20, 24; F431 at 28, 32; F321 at 36, 40, 44; F661 at 48, 52: steps start at t = 9, 18, 27, 36, 45 ... (every 9 frames, a 36 frame cycle) |
| A07 | `r2:a-e6a1-storm`: `jump 6 1 0x2E0`, peek palette RAM every 2 frames, scroll3 tiles 0x913A60 every 2 frames | scroll3 palette flashes start at t = 2, 126, 378, 450, 582, 654 and scroll2 palette flashes 8 frames later (2, 134, 386, 458, 590, 662); the bolt tiles at 0x913A60 change at t = 114, 118, 120, 124 and return at 126, 366, 370, 372, 376, 378; the other two bolt strips stay unchanged; scroll1 rain changes every frame |
| A08 | `r2:n-e6a1-lightning` (same jump), snapshots at t = 0 to 20, 130 to 152 | pictures: t = 1 sky white, t = 3 to 5 grey world, t = 6 all white, t = 8 sepia; t = 0 is a flash already running |
| A09 | `r1:e6a1-foot-god`: `jump 6 1 0x4E0`, god, `0-100:R`, `220 poke 0xFFB27C 2 0x6C0` | A22 at (1728, 30, 256) from t = 100, state 4 at t = 140 with y 256; the shadow oval under it in the pictures; the cycle of D3b |
| A10 | `r1:e6a1-barrels-slow` / `-fast`: `jump 6 1 0x100`, god, slow bot or dash | slow: a barrel (var 1, X 354 at t = 400) is broken by t = 556 and an E9 var 5 treasure appears at t = 560; dash: the barrel A#23 is freed at t = 88 at X 406, no item |
| A11 | `r2:a-e6a1-swamp`: `jump 6 1 0xA00`, peek scroll2 every 3 frames | water rows 42..49 change at t = 18, 42, 66 ... every 24 frames; the 2 x 2 eye (cols 53..54 rows 35..36) changes at t = 30, 54, 78, 102, then 198, 222, 246, 270, 294 |
| A12 | `r2:a-e6a2-water`: `jump 6 2 0xD90` | rows 43..47 in cols 0..15 and 48..63 change at t = 6, 30, 54 ... (24 frames, 5 distinct words) |
| A13 | `r1:e6a2-boss-intro`: `jump 6 2 0xEF0`, god, hold kill-all, `0-120:R` | B15 var 7 at t = 110; B79 at (4384, 0, 400) from t = 350 with states at 350, 470, 700, 820; B66 vars 0x50 (350), 0x51 (470), 0x52 (700), 0x53 (820); Tyrog A4 var 0x200 at t = 904, B22 at 910 |
| A14 | `r2:n-e7a1-full` (or `r1:e7a1-nat`): `jump 7 1`, god, fixed screen bot | locks at camera Y 12288 (t about 550), 7680 (1850), 5120 (3060), 2048 (4500); end lock Y 0 (5320 in r1); B57 at t = 0 with y 13920 (0x3660: its 0x3760 minus the first label); B64 x 4 at t = 0 with positions (24, 184), (96, 56, Z 128), (288, 56, Z 128), (360, 184) |
| A15 | `r2:l-wave4`: `jump 7 1`, bot, peek scroll1 every 4 frames from t = 500 | wave 4 controller at t = 532; scroll1 cols 5..12 and 35..42, rows 47..54 change once at t = 616 |
| A16 | `r1:e7a2-fence1-z64` (z79, z100): `jump 7 2`, god, poke hero X 0x118, Z, `20-22:A` | B1 var 4 at (224, 0, 128): HP 1 to 0, state 04000000 at t = 22 and 04020000 at t = 24 with y 32; C1 at 22, C21 at 24; `r4:tc-b1v4`: 48 attribute words change (cols 15..27, rows 57..62) and no code word |
| A17 | `r2:n-e7a2-end`: `jump 7 2 0x5F0`, god, `0-200:R`, punches, `killall` every 90 frames | lock at camera 1600 (t = 110, wave 12); B52 at (1840, 0, 112) at t = 876; GO at t = 1500, B1 var 8 at (1984, 0, 128) from t = 1510, hero at X 2039 at t = 1550, fence `04020000` at t = 1560, fade t = 1630, area 3 at t = 1720; `r4:tc-b52`: 55 code and 70 attribute words in cols 47..56, rows 49..55 |
| A18 | `r2:n-e7a3-talk`: `jump 7 3 0x240`, god, `0-900:R`, `200 killall`, `320 killall` | B41 x 3 at t = 40, B33 at 350, **B30 at (1200, 208) at t = 410**, state 0202 at 470, 0204 at 590, B34 at 590; palettes 66 to 91 change every 17 frames |
| A19 | `r1:e8a1-vent-z832`: `jump 8 1 0x200`, god, poke hero X 0x28C Z 832, punches every 14 frames from t = 20 | B45 var 0 (672, 48, 848): HP 48, 38 (t = 36), 28 (t = 50), 13 (t = 64), 0 (t = 80); state 04000000 at each hit; B45 var 4 at X 800 untouched |
| A20 | `r2:c-b45-z760` to `z824` (`hold` hero X 0x28C and Z) | no HP change for Z 760, 776, 792, 800, 808, 816; hits at Z 824 |
| A21 | `r2:c-flame-x680`: `jump 8 1 0x200`, `level 0 free`, `hold` hero X 680, Z 831, hp 100, no god | hero HP 100, 80 (t = 102), 60 (t = 222), 40 (t = 360) with the burn state 0x0A00; `c-flame-level10`: 100, 78, 56, 34; `c-flame-x720`, `x760`: no damage in 700 frames; A83 pieces start at (X + 18, 121, Z - 22), attack 93, move (-7.5, -5) per frame |
| A22 | `r2:c-flame-enemy`: a Hammer var 0x200 spawned at X 0x2B0 Z 831 | HP 100, 80, 60, 40 at its three passes through the flame |
| A23 | `r2:c-b40-v8-jab`, `c-b40-vC-jab`, `c-b40-v10-jk`: `hold` hero 831 and X 128, 1336, 1728 | B40 vars 8, 0xC: HP 8 to 0 at t = 50 and t = 22; var 0x10: jab no, jump kick at t = 32 (jab at t = 20, jump `20-21:J`, `30-32:A`); `r4:tc-b40vC`: 11 tile code words (cols 21..23, rows 4..7) |
| A24 | `r2:n-e8a1-w3` (also w4, w6, w7): `jump 8 1 0x150`, hold kill-all, `0-200:R` | B42 var 0 at (456, 0, 880) at t = 150; the red door becomes a black doorway by t = 190; `r4:tc-b42v0`, `v4`, `vC`, `v10`: 24 tile codes in cols 28..31, 56..59, 24..27, 52..55, rows 2..7 |
| A25 | `r2:n-e8a0-end`: `jump 8 0 0x400`, god, hold kill-all, `0-520:R` | B40 var 4 at (1680, 0, 64) from t = 610, never broken; B35 at (1591, -9, 55) at t = 710, at (1541, 79) from t = 730 to 780, thrown at 790 to (1680, 24) by t = 830, down at t = 870; B47 at 710; the hero stays at X 1544, Z 56; area 1 at t = 960 |
| A26 | `r2:n-e8a1-end`: `jump 8 1 0x9B0`, god, hold kill-all, `0-900:R` | wave 7 at t = 110; B36 var 0 at (3104, 0, 768) at t = 452; B37 and B38 (X 3360, Z 256, 512, 768) at t = 508; camera mode 9 at t = 530; area 2 at t = 550 with the cave caption var 0x67 at t = 540 |
| A27 | `r1:e8a2-sweep-500`: `jump 8 2 0x500`, god, hold kill-all, sweeps | A65 var 0x201 barrels: spawn X = trigger + 0xBF (right of the camera by 0xBF), roll right 2 pixels per frame, freed at X = camera X + 512 (2219 at camera 1707) |
| A28 | `r3:m-e8a2-meat`: `jump 8 2 0xA80`, god, hold kill-all, `0-196:R`, `200 poke X 3224, Z 272`, `210-212:AR`, `240-2300:R` | E5 var 4 at (3248, 0, 272) at t = 188; lifted at t = 212 (hero state 0x14, `held` = the barrel); held to t = 2600 at X 4747 (area 3 from t = 1330); `m-e8a2-meat2` (attack at 240): thrown, broken at t = 272, food object E4 var 1 |
| A29 | `r4:b53-a2`, `b53-a3` (`30 poke 0xFF87B2 1 2`, `90 poke 0xFF87B2 1 0`, palette peeks every 2 frames); `b53-a1`, `b53-e7a3` | a2, a3: 370 palette colours in 27 palettes (65 to 122) change at t = 32 and again at t = 92 (back); a1 and e7a3: no change from the poke |
| A30 | `r1:e8a1-idle-0`, `-2D0`, `-A70` with `watch 1 G @0xFF87AE.b @0xFF87AF.b` | G.$7AE: gf 1284 value 0, 1308 value 4, 1332 value 8, 1356 value 12, 1380 value 0 (steps of 24 gf); G.$7AF copies one frame later |
| A31 | `r3:ee-f84-kill` (the D3d replay) | after the boss death at t about 7296: B82 at 7300, B84 var 0 for 400 frames from 7304, B83 var 0 for 200 frames from 7304, B66 var 0x69 at 7308 |
| A32 | captions: `r2:n-e7a3-talk`, `r1:e8a0-end`, `r3:s-e6a0-k18`, `m-e8a2-meat` | B66 var 0x64 110 frames, 0x65 80, 0x61 120, 7 240 (2.7) |

## 7. (b) Port task list for the classes and events of these episodes

Sizes S 0.4 h, M 1 h, L 2.5 h (agent hours). Prerequisites: T-B0 of part 1 (tile block writer, prop hit pass 0x10CD0, break sound helper), pool C effects 21 and 1 (D3f), the text printer of M1, D4c scripts 16, 19, 21, 22, 23. Items marked "D3x" are counted in that design's list.

| Task | Content | Size |
| --- | --- | --- |
| **T3-0 services** | (1) palette cycler driver: lists of colour sets, mask 0xF000, period, the same code for B60, B61, B63, B71 and the lightning tables; (2) the tile clock function `7AE = ((gf + 60) / 24 mod 4) * 4`; (3) a "burst" driver for B4, B42, B52 and the part 1 classes 11, 12, 14 (wait 1 frame, tile block swap from a table, effect, sound, free); (4) the hero reach rule of 2.6 in the prop hit pass | 1.5 h |
| **Episode 6** | B3 var 0 (hit, solid wall until broken, 31 code and 37 attribute words) and var 8 (48 frame timer): M; B4 vars 4, 8, 0xC (burst, enemy entrance 0x50 frames later): S; B60: S; B55 rain streaks: S; B56 drops (pool C 2): S; B9 lightning (bolt strips, 14 palette lists, thunder, pause table): L; B7 var 0: S; B8: M (shares code with B2); B39: S; B79 and the Tyrog scene actors A75, A76: M (D3d fights); foot A22 and A23: D3b P6; rolling barrels A65: D3a | 7.9 h (+1.5 D3b) |
| **Episode 7** | B1 (sprite 326 and 327, hit, attributes, script 19 break): M; B52: S; B54 (cage smash, once): S; B57 labels (56 labels, glyphs): M; B64 posts: S; lift background and the harbour window (map data, scroll1 cage): D4c T-C8; B41: S; B61 and B63 (with the driver): S together 0.8; B30 and B34 (monitor talk): M 1.5; B33 and the Slisaurs: D3d; end scripts 16 and 19: D4c | 5.9 h |
| **Episode 8** | B28 and B29 (water bands, clock): M; B36, B37, B38 (waterfall): M; B43 torches: S; B40 vars 4, 8, 0xC, 0x10 (exists as the Episode 1 pipes, add the data and the kind box table): S; B42: S; B45 and A83 flames (cycle, 6 vents, damage 20, burn reaction link): L; B35 and B47 (manhole, hero pose 0x1F, script 21): M 1.5; B53 (27 palette sets from G.$7B2): M; B71: S; B83, B84 (and B85 with D4d): S x 2; tanks B44: D3d; meat barrel carry: pool E rules (D2c), no new code | 9.4 h |
| Total | | about 24.7 h (+ 1.5 h of the foot) |

Order proposal: T3-0, then B3, B4, B60 (Episode 6 area 0 is playable), B55, B56, B9, B8, B7, B39 (area 1 and 2), then B1, B52, B54, B57, B64, B41, B61, B63, B30 (Episode 7), then B28, B29, B43, B42, B40, B45, B36 to B38, B35, B53, B71 (Episode 8).

## 7. (c) Open items

1. **The kill-all crash of D4c** (an "ADDRESS ERROR" screen in Episode 7 area 0 with three players and `hold` kill-all at camera X 0x900) was not reproduced: one player with the hold ran 600 frames (`r1:x7-hold900`), a single `killall` and 11 single spawned classes ran clean. Next: `jump 7 0 0x900 char=jack,hannah,mess` with the hold and a watch of pool A to name the class that is alive when the screen shows.
2. **B71** (end area palette animation, trigger 0x1140 of the lair) and the meaning of **B66 var 7** (a blank string shown for 240 frames at the lair) were not seen: the boss kill replay does not reach the trigger. Read after D3d section 6.
3. **The monitor of B30**: the static noise and the doctor's face are not decoded (layer, tile or sprite source), nor the three text lines of B34 (paraphrased only). A picture capture of scroll1 at t = 590 would settle it.
4. **Weapons and explosions against B3, B1, B40, B45**: only player attack boxes (jab, kick, jump kick, special) were tested. Held weapons, thrown barrels, grenades and bazooka shells were not.
5. **Score for hitting a prop** (the hit pass calls the score hook 0xFFC8) was not measured in these episodes; D4b may know the value.
6. **B1 var 8 tile effect** is only 3 attribute words (the area is mostly already in the broken state): check against the picture of the library end whether the visible change is the sprite swap only.
7. **Harbour window**: the exact scroll2 map region and the far layer picture of the window are map data; my range (camera Y 5216 to 7776 for the blank labels, window visible about 5200 to 7450) is from pictures. The asset dump shows the transparent gap at another height of the dump (probably its own row origin).
8. **Mine door B4 var 0xC in a natural fight**: the whole of wave 2 could not be played with the hold (Lash T. and Walther ignore it); the burst was tested by spawning the class by hand.
9. **The Episode 6 area 2 diagonal climb** and the salad barrel carry were not walked with a carried barrel.
10. **Area starts of areas above 0** cannot be started by `jump ... at=start`; a start script recipe (poke of the script id of the area before) is used instead. For area 2 of Episode 7 and the library walk-in (script 9) no picture sequence exists.
11. **Sounds** are not recorded by the probe; the codes named are from the code (0x6F and 0xF6 break, 0x77 thunder, 0x8D rain, 0x6D lab, 0x23 monitor talk, 0x21 Tyrog scene, 0x86 and 0x87 lift).
12. **Fence solidity**: whether the stage terrain stops a walking hero at the unbroken library fences (the handler does not change terrain, D3e part 2 section on B1) was not tested here.
13. **Hero reach for the 40 to 48 pixel limit** of a jab is approximate: Jack steps forward while punching unless held. A table of the attack box X per attack id would make the port exact (D2a has the boxes).

## 7. (d) New rows for `mechanics-seen.md`

| ID | Mechanic | Source | Port | Notes for the design agent |
| --- | --- | --- | --- | --- |
| W301 | Episode 6 mine entrance wall: one hit breaks it (hero depth 32 or more, jab, jump kick, special), solid until broken (a walking hero stops at X 846), planks fly and a rock mouth shows; the exit wall of the jungle start breaks by itself 48 frames after the area load | M r1:e6a0-wall-*, r3:s-e6a0-k18, r4:tc-b3v0; R 0x763F6 | MISSING | B3 var 0 and var 8. Answers W25 for Episode 6 |
| W302 | Doors and a bookshelf that burst open at the start of a wave group and let the enemies out: three doors in the Episode 6 mine area (X 0x220, 0x2A0, 0x7B0), four red doors in the sewers (X 0x1C8, 0x383, 0x980, 0xB47), one bookshelf in the library (X 0x730, wave 12) | M r1:e6a0-door1, r2:n-e8a1-w3, n-e7a2-end, r4:tc-b4*, tc-b42*, tc-b52 | MISSING | B4, B42, B52: tile block swap, planks or books (C21 0x16, 0x26, 0x20), break sound. Corrects N11 (one shelf bursts open) |
| W303 | Prop hit rule: a player attack hits a prop box when hero Z + 8 is at least the start of the box depth range and Z is at most its end (plus X and height overlap); one hit breaks B1, B3, B40, HP 48 needs 4 hits for B45 | M r1:e6a0-wall-z*, r2:c-b45-z*, c-b40-*; R 0x10CD0 | MISSING | extends W31 |
| W304 | Mine lamps: two scroll palettes (80, 81) cycle in 4 steps of 9 frames (36 frame cycle), Episode 6 area 0 only | M r1:e6a0-lamps; R 0x863AC | MISSING | B60; same driver as B61, B63, B71 |
| W305 | Storm of Episode 6 area 1: rain streaks (75 streaks, scroll1, every frame, to camera X 0x500), rain drops with floor splashes (pool C 2, density 2/16, 4/16, 8/16, sound 0x8D), three lightning objects (bolt strip 5 pictures of 3 frames, sky palettes then foreground palettes flash 8 frames later, thunder code 0x77 after 40 frames, pauses 60 to 420) | M r2:a-e6a1-storm, n-e6a1-lightning; R 0x8574C, 0x85916, 0x77E76 | MISSING | B55, B56, B9. Answers W17 (rain) |
| W306 | Brontosaurus foot shadow: a black oval on the floor under the foot at its X and Z, following the hero while the foot hovers; the foot hurts enemies too | M r1:e6a1-foot-god; D3b, D3a | PARTIAL (DS12) | shadow is the pool A shadow of A22 |
| W307 | Rolling barrels are freed at camera X - 0x40 or + 0x200: they carry their item but a fast hero outruns them, a locked camera loses them to the right after about 160 frames; the cave barrels (var 0x201) fall from 0x100 above with a floor shadow | M r1:e6a1-barrels-*, e8a2-sweep-500; D3a 10.1, D4a 14 | PARTIAL | answers N10 and N12 |
| W308 | Swamp scenery of Episode 6: water surfaces B8 and B39 (4 pictures of 24 frames), the eye of the huge reptile head opens and closes (B7 var 0) | M r2:a-e6a1-swamp, a-e6a2-water | MISSING | B7, B8, B39. Answers M15 |
| W309 | Tyrog scene: three gangsters carry a metal box, three captions, the box opens and Tyrog jumps out (B79, A75, A76) | M r1:e6a2-boss-intro | MISSING | links D3d Tyrog |
| W310 | Lift shaft: floor labels L15 to L1, B1 to B10, blank B11 to B20 (open harbour window), B21 to B40 painted as the camera passes (B57); cage posts (B64); the left and right cage walls are smashed once at the first stop (B54) | M r1:e7a1-nat, r2:n-e7a1-full, l-wave4; R 0x85B64, 0x855A8, 0x86B64 | MISSING | extends W14 |
| W311 | Lift riders: entrances from X -48 and 432 and from above (y 255, variants 0x20A); list in 2.5 | M r2:n-e7a1-full | PARTIAL | D4c 7.2 has the stops |
| W312 | Library gates: fence B1 var 4 at the start is broken by any player attack at any time, fence B1 var 8 at the end by the leader of the end script 19 (GO + about 60) | M r1:e7a2-fence1-*, r2:n-e7a2-end | MISSING | B1; answers L10 |
| W313 | Lab of Episode 7: console blink lights B41 (periods 19, 7, 13), palette cycles B61 and B63 (17 frames, sound 0x6D every 121 frames), monitor talk B30 (black, static at +60, the doctor's face at +180 with a three line caption B34, sound 0x23) | M r2:n-e7a3-talk, a-e7a3; r3:a-e7a3-b41 | MISSING | extends W26 |
| W314 | Sewer flame vents B45: HP 48, cycle idle 60 to 90, sparks 45, growth 40, full 40 to 70, shrink 40; flame pieces A83 (attack 93, 20 damage at level 0, 22 at level 10, burn knock down, also hurt enemies); hero must stand at Z 824 or more to punch a vent | M r1:e8a1-vent-*, r2:c-b45-*, c-flame-*; R 0x83E0C, 0x6155C | MISSING | B45 + A83. Answers W12 |
| W315 | Wall pipes B40 of Episode 8: three sets in the sewers (a jab breaks two, the third needs a jump kick from 48 pixels away) and one set at the end of the lab that cannot be reached in play; steam for ever after the break | M r2:c-b40-*, r4:tc-b40*, r2:n-e8a0-end | MISSING | B40 vars 4, 8, 0xC, 0x10. Closes part 1 open item 3 |
| W316 | Tile clock G.$7AE = ((gf + 60) / 24 mod 4) * 4, copied to G.$7AF one frame later | M r1:e8a1-idle-* | MISSING | closes part 1 open item 2 |
| W317 | Sewer scenery: water bands B28 and B29 (rows 12 to 14), wall torches B43 (4 pictures of 8 frames), the waterfall B36, B37, B38 | M r2:a-e8a1-* | MISSING | answers W19 for Episode 8 |
| W318 | End of the Episode 8 lab: the leader lifts a floor grate (B35), throws it right and jumps into the manhole (B47 hole); end of the sewers: the heroes jump into a waterfall shaft (B36 to B38), slow fade, the cave starts | M r2:n-e8a0-end, n-e8a1-end | MISSING | scripts 21 and 22 |
| W319 | Fire glow: B53 rewrites 27 palettes (370 colours) while a hero lies burning (G.$7B2 = 2), only in Episode 8 areas 2 and 3 | M r4:b53-* ; R 0x1C6E4 | MISSING | B53 |
| W320 | The meat barrel at X 0xCB0 can be lifted and carried through the sub area switch, the scripted walk and into the lair | M r3:m-e8a2-meat | PARTIAL (pool E lift exists) | answers L11 |
| W321 | Caption banners (B66 vars 0x60 to 0x67): COAL MANE, JUNGLE, CITY IN THE SEA, LIBRARY, COMPUTER ROOM, BIO LAB, UNDERGROUND, CAVE, with durations 80 to 120 frames | R 0x87710; M | MISSING | the text printer is of M1 |
| W322 | The Episode 7 background is the Episode 1 skyline art in an orange palette in front of a stone terrace and bridge; area 0 has no animation | M r2:a-e7a0-*; assets | MISSING (map) | answers L10 half |
| W323 | After the final boss death the ending code makes B82, the showers B83 and B84 (200 and 400 frames), the quake B85 and the captions B66 0x69 to 0x6B | M r3:ee-f84-kill | MISSING | D4d owns the scene |
| W324 | Probe note: `hold 0xFF850E 1 2` does not kill Lash T. or Walther; `jump ... at=start` does not stop at the walk-in of areas above 0; full RAM `peek` of scroll1, scroll2, scroll3 and the palettes gives a sprite free picture of every tile and palette animation | M r1, r2 | n/a | for T0 and later agents |

