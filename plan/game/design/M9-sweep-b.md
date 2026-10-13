# M9-07: open item probe sweep B (enemies, dinosaurs, bosses, camera, sound)

Written 2026-10-13 (task M9-07 of `D5-final-plan.md`, lane B). **Status: paused before the first MAME batch.** The coordinator stopped all work for a week. No probe was run for this task, so
there is no new measured answer in this file. This file holds the plan, the constants that were found while preparing, and the exact list of what is left, so the work can restart without
reading the design documents again. Source tags are as in `M9-sweep-a.md`: **M:** measured in MAME, **R:** read in the program code, **P:** planned only (not run).

Batches used: 0 of 7. Files made: `re/out/probe/_input/m907_an.py` (log reader, see section 2). No other file was changed. Nothing was committed.

## 1. Result rows

Every open item of the task block is listed with its status. "Not run" means a recipe is written below (section 3) but MAME was not started. Items marked **closed earlier** have an answer
in an older design document or in an old probe log that I read again; they are not new measurements.

### 1.1 Bosses and dinosaurs (first priority)

| id | open item | status | recipe (section 3) or note |
| --- | --- | --- | --- |
| D3c-1 | bazooka and shotgun against the bosses | not run | recipe B1: Butcher, Slice, Vice at their natural trigger, plus Walther and thug shotgun controls |
| D3c-2 | kill score when the car kills Hogg | not run | recipe B2: old segment `h8-ramkill` of `d3c-r2` with a `watch 1 P1 score` line added |
| D3c-3 | full foot run of Episode 3 without the radio, camera X 0xA000 end, time over path `G.$518 = 2` | not run | recipe B3. Old log `d3c-r2/h13-footrun` shows why the old run was not a foot run: the hero picked up a radio by itself (car flag set and camera X 6060 at t = 3000) |
| D3c-4 | Hogg car sub states 6 and 8, wheelie, front box against thugs | not run | recipe B4 (thugs in the ride path); the sub states need the old car logs read again |
| D3c-5 | Slice fan invulnerability, Butcher hittable at get up, Vice `+$BE` timer | not run | recipe B5 (hit spam on a pinned boss with high HP) |
| D3c-6 | seat drawing | not reached by design | only a screenshot could show it; the draw routines are in D3c |
| D3c-9 | sounds of the car and the bomb flight | not run | sound recorder (section 2.3) in B2 and B4 |
| D3d-2 | can the Tyrog host (class 72) be killed | not run | recipe B6 (`jump 6 2 0xE00`, spawn class 72 variant 0, hit spam) |
| D3d-4 | Slisaur lying time against Slice | not run | recipe B5 (same spam, both bosses, read the knock down lengths) |
| D3d-6 | does a hero hit remove an Uzi bullet (class 59) | not run | recipe H2 (bullet spawned in front of the hero, jab spam, class 59 variants 0 and 5) |
| D3d-9 | Fessenden grab test, rule for big parts | not run | old grab segments exist in `re/out/d3d/r3.txt` (`grab-f84`, `grab-f84b`); nothing new planned |
| D3d-12 | damage of class 95 fire and grab hits 0x64 and 0x65 | not run | recipe B6 (class 84 alone, hero HP reset every 25 frames, level 12 and level 0) |
| D3d-15 | does a long chain lock a Tyrog form or class 84 | not run | recipe B5 (hit spam, compare with Vice, who is known to lock) |
| D3b-1 | other wake sources of a sleeping Shivat | not run | recipe D1 |
| D3b-2 | fire against dinosaurs | not run | recipe D2 (hostile Hopper and Shivat next to the hero, torch at several depth offsets, blast and flame vent) |
| D3b-3 | Zeke, the foot and the Hopper against enemies | not run | recipe D3. The foot against poachers is **closed earlier** (D3a HU15, probe `h-foot-poacher`: 30 damage and knock down) |
| D3b-4 | Shivat claw, tail and stomps against a hero | not run | recipe D4 |
| D3b-5 | KYOURAN and DIFFENCE of the Hopper | not run | recipe D5 |
| D3b-8 | dinosaur alive at an area change or stage end | not run | recipe D6 |
| D3b-9 | hero grab escape | not run | recipe D7 (forced Zeke dive grab, then stick, jump and attack mashing) |
| D3b-12 | 3 player test of Zeke's target and the Hopper hold | not run | recipe D8 |
| D3a-2 | poacher bullet hit by a hero | not run | recipe H2 |
| D3a-3 | hunter swing against a hero | not run | recipe H3 (class 79, god off, HP reset every 25 frames) |
| D3a-4 | Bludge hit while it eats | not run | recipe H4 (old `bl-eat` moved to Episode 1, aggression 7) |
| D3a-6 | class 64 blast damage, knock down drop of the dynamite, pick up of the lit item | not run | recipe H5 |
| D3a-9 | Episode 7 area 0 bikers | not run | recipe H6 |
| D3a-13 | 2 and 3 player rows (poachers, hunters, Bludge) | not run | recipe H7 (the records with `players>=2` and `players==3` are in `re/specs/enemy/stage_scripts.txt`) |
| D3a-14 | which units ignore the radio kill-all | not run | recipe H8 (flag 1 and flag 2 on carriers and bikers) |

### 1.2 Camera, time, rank (second priority)

| id | open item | status | recipe |
| --- | --- | --- | --- |
| D4b-1 | Episode 7 rank speed bonus (all Slisaurs) | not run | recipe C1 (watch `G.$5F0` rank at each Slisaur death) |
| D4b-2 | title wait counter `G.$4F6` | not run | recipe C2 (boot only, watch `@0xFF84F6.w`, with and without a coin) |
| D4b-4 | stale `+$84` for thrown objects | not run | recipe H9 (first poacher bullet damage on a fresh game against the level formula) |
| D4b-5 | 2 player cabinet layout (`G.$4CF`) | not run | recipe C3 (poke the word from power on, look at select and tally pictures) |
| D4c-1 | Episode 7 lift end and area 2 to 3, the hang | not run | recipe C4 (natural play, `killall` pulses instead of a held flag) |
| D4c-2 | real boss kills for Butcher and Episodes 3 to 8 | not run | recipe C5 (set HP once, let the hero finish) |
| D4c-3 | Episode 8 ending trigger with a real Fessenden kill | not run | recipe C5 |
| D4c-4 | meaning of `G.$4DB`, `$516`, `$503`, `$4D8`, `$4DF`, `$514` | not run | add them to the global watch of the C5 and B2 runs |
| D4c-5 | car: `G.$518 = 2`, helper objects, what ends the section | not run | recipe B3 (time over with Hogg active, radio taken during the Hogg fight) |
| D4c-6 | caption 0x1B text | not reached by design | the text is ROM data and may not be copied here |
| D4c-7 | 2 player cabinet layout | not run | same as D4b-5 |
| D4c-8 | sub area start script id 0 never runs | closed by reading | R: D4c s12 item 8 says the call 0x1DA10 is not made for sub areas; nothing to observe |
| D4c-9 | two wave controllers at once in Episode 5 area 1 | not run | recipe C6 (natural walk, count pool B controllers) |
| D4c-10 | pose of the four heroes at a boss death | not run | recipe C5 with each hero |
| D4c-11 | lineup slots with only P1 and P3 | not run, doubtful | the probe `jump` may not start P1 and P3 without P2; try `char=` with a gap, else mark not reached |

### 1.3 Sound (third priority)

| id | open item | status | recipe |
| --- | --- | --- | --- |
| D4e-2 | music 0x26 has no caller | closed by reading | R: D4e s10 (no caller found); a recorder run over all segments would also show it never occurs |
| D4e-3 | team move and explosive kill voice lines | not run | sound recorder during the tag team tie (two heroes 10 px apart, tie at t = 106) |
| D4e-4 | freeze at 0x5B4 and the Z80 | not reached by design | a port decision, not in the ROM |
| D4e-5 | twin voice headers, same hardware voice | not reached | not visible through the request queue |
| D4e-6 | writes to `G.$43C` | not run | add `@0xFF843C.w` to the sound watch of every fight segment |
| D4e-7 | which enemies have `+$63 > 0` | not run | one segment per class pool: spawn each class and read `+0x63.b` |
| D4e-8 | 0xA5DE has no caller | closed by reading | R: D4e s0 |
| D4e-9 | demo sound factory default | not run | watch `@0xFFF77C.w` bit 5 at power on |
| D4e-10 | album boss intros 0x20 or 0x23 | not run | sound recorder in the Vice, Butcher and Fessenden intros |
| D4e-11 | the 24 codes of section 7, family 0x2C to 0x2F | not run | collect every code seen by the recorder in all segments of all batches |
| D4e-12 | join strip 0xE0 and 0xE1 | not run | recorder while P2 joins (coin and start) |
| D4e-13 | Morgue music | not run | recorder in the Episode 5 boss fight |
| D4e-14 | the class that sends 0x82 and 0x83 | not run | static search of the ROM for the operands near 0x37Cxx and 0x666xx, then name the class |
| D4e-15 | computed codes at `jsr 0xA63C` sites | not run | recorder runs, list the codes seen |
| D4e-16 | sequence lengths against the album | not reached | needs a recording, not a request queue |

## 2. How to run (what I prepared)

### 2.1 Probe tool facts that matter

* MAME runs about 32 times real time headless (sweep A: 138,000 emulated frames in 73 s). A batch of 60 to 80 segments of 2,000 to 5,000 frames takes 3 to 6 minutes. The Bash
  foreground limit is 10 minutes, so keep one batch under about 700,000 emulated frames and set `PR_TIMEOUT=560`. Use `PR_SHEET=0` and no `snaps` to save time.
* Hero position after a jump is `camera X + 0x60` for X. Known (x, z) of P1 at t = 0 (read from old logs, same jump line gives the same value): `jump 1 0 928`: (1024, 296);
  `jump 1 2 0xA0`: (256, 295); `jump 2 2 0x1A0`: (512, 49); `jump 4 2 0x3D8`: (1080, 64); `jump 3 0 0x2D00`: (11616, 64); `jump 6 2 0xE00`: (3680, 354);
  `jump 8 3 0x1000`: (4192, 320); `jump 5 1 0x1120`: (4480, 351); `jump 7 3 0x240`: (672, 64); `jump 5 0`: (49, 576). `jump 2 0` starts on the Cadillac (y 64): avoid it for small tests.
  **`jump 1 0 928` is the flat test ground for every small test** (Episode 1 area 0, camera fixed, no script spawns near).
* P1 fields: x at 0xFFB27C, y at 0xFFB280, z at 0xFFB284 (P2 +0x180, P3 +0x300). `hold ADDR 2 V` pins the hero. Pool A slots are 0xE0 bytes from 0xFFC8F4; the first free slot after a
  jump was 0xFFD994, but pins by `objset A:CLASS x V` on every frame do not depend on the slot.
* `spawn` writes the depth of player 1 by default, so a spawn at `x=+N` is at the hero's depth. A spawned bullet (class 59) flies left (speed 7).
* Weapon item variants (`spawn E 0 var=V`): rifle 0, pistol 2, knife 4, Uzi 6, M-16 8, shotgun 10, bazooka 12, rod 14, stick 16, butcher knife 18, club 20, torch 22, dynamite 24, grenade 26, stone 28.
  Barrel: `spawn E 5 var=4`. Radio (Episode 3): `spawn E 9 var=0xC`. Dynamite blast: `spawn E 1 var=3`. Flame vent: `spawn B 45`.
* Boss ready times from the jump (old logs): Butcher class 5 changes to class 61 at t = 475 (jump 2 2 0x1A0, HP 576 at level 15); Slice (class 25) alive at about t = 404 (jump 4 2 0x3D8);
  Vice (class 18) appears at t = 70 and the intro ends near t = 780 when the hero walks right for 300 frames (jump 1 2 0xA0); the camera then locks at X 464, so a hero pinned at X 620 stays in range;
  Hogg (class 31) is there at t = 0 (jump 3 0 0x2D00). Sweep A's Vice run was too short: the intro is 710 frames, so "stays in state 0x200" was only the intro (rerun with 1000 frames).
* Directive cost: `objset` lines every frame are fine for a few thousand lines per segment.

### 2.2 The reader

`re/out/probe/_input/m907_an.py` (run with `re/.venv/bin/python -I`): `series`, `drops` (hp changes with the delta and `st` / `atk`), `snd` (sound requests), `notes`, `last`.
It was written but not run on a new log (it was only meant for the batches). Check it on the first batch before trusting it.

### 2.3 Sound recorder (planned, untested)

The sound requests can be read from the probe without a new tool: one request is flushed per frame, the read index `G.$36` (0xFF8036) steps by 0x10, and the mailbox holds the code.
Add this line to every segment (all codes pass in game):
`watch 1 G @0xff8034.w @0xff8036.w @0xf18001.b @0xf18003.b @0xf1800d.b @0xf1800f.b`
(write index, read index, code high, code low, pan mode, pan; mailbox bytes are the odd bytes of 0xF18000 + 2n for Z80 C000, C001, C006, C007). `m907_an.py snd LOG` prints a row each time the read
index changes. Not tested: the first batch must confirm that the read of 0xF18001 works from the Lua side and that a request shows once. If it does not, read the ring entry at `0xFF803A + old read index` instead.

## 3. Recipes (planned segments, inputs not yet written)

The probe inputs `m907_a.txt` to `m907_g.txt` were meant to be made by a generator (`m907_gen.py`, not written yet). The recipes below are complete enough to write it.

**B1 boss weapon factors.** For Butcher (`jump 2 2 0x1A0`, ready t = 480), Slice (`jump 4 2 0x3D8`, ready t = 420) and Vice (`jump 1 2 0xA0`, walk right 0 to 320, `killall` every 30 frames to t = 700, ready t = 800;
pin the hero at (620, 295)): `god on`, `rank 0`, `level 0`. At ready minus 70 spawn the weapon at the hero, press A at ready minus 62 to pick it up, pin the boss at hero X + 100 (ranged) or + 35 (jab) and the hero's X and Z on
every frame, fire at ready + 0, 40, 80, 120 (jab: taps every 30 frames). Weapons: pistol 2, shotgun 10, bazooka 12. Read the boss HP drops with `m907_an.py drops`. Controls: thug (class 2) shotgun, and Walther (class 18 variant 0x200)
shotgun in the flat ground; sweep A already has jab 14, pistol 29, shell 58 for a level 0 thug. Expected from D2c table 7.3: shell x0.5 on Slice, Butcher and Vice, pellets x0.75 on Vice and Walther.

**B2 car kill score.** Old segment `h8-ramkill` of `re/out/d3c/r2.txt` (jump 3 0 0x300, radio at t = 0, Hogg HP poked to 20 at t = 1345) with `watch 1 P1 score` and the sound line added; compare P1 score before and after Hogg dies.

**B3 foot run and time over.** `jump 3 0`, `god on`, right held, A tap every 40 frames, and `objset E:9 active 0` on every frame so no radio exists (treasure items are lost, that is accepted). Log `G` every 20 frames
(camera X, lock, time, alive, `G.$518`). Run 36,000 frames (about 20 s of MAME). Second segment: `jump 3 0 0x2D00`, take the radio during the Hogg fight, `time 0:00` at t = 700 for the `G.$518 = 2` path.

**B4 Hogg front box.** `jump 3 0 0x2D00`, three class 2 enemies at hero depth in the ride path (x + 0x60, + 0x90, + 0xC0), watch their HP and state, 1500 frames.

**B5 hit spam.** Boss pinned next to the hero with HP set to 6000, A tapped every 7 frames for 2,500 frames. Bosses: Vice, Slice, Butcher, Slisaur (`jump 7 3 0x240`, class 71), Tyrog (`jump 6 2 0xE00`, class 4 variant 0x200),
class 84 (`jump 8 3 0x1000`, spawn at X + 0x100). Count frames in hit reaction against frames in action, the knock down lengths, and HP drops during fan, get up and spin frames.

**B6 Tyrog host and Fessenden damage.** Host: `jump 6 2 0xE00`, spawn class 72 variant 0, pin and spam as in B5, 2,000 frames; look for HP 0, the score and the stage end flag. Class 84 damage: `jump 8 3 0x1000`, rank 12, spawn class 84,
hero HP set to 100 every 25 frames, 4,200 frames, list hero HP drops with the boss `atk` and class 95 variant.

**D1 wake sources.** Flat ground, Shivat (class 24 variant 0, `w5c=0x7fff`) at hero X + 100: thrown knife, thrown barrel (as sweep A `ffb2_*`), pistol shots, bazooka, thrown dynamite, torch swing, and a poacher (class 13 variant 2)
behind it so that bullets pass through the Shivat towards the hero (the old probe `s3-poacher` placed the Shivat on the wrong side). 3,000 frames for the poacher. Watch `+0` (2 = asleep), `+0xA8`, `+0xC0`, HP.

**D2 fire.** Hostile Hopper (class 29 variant 0x200, then `objset +0xA8.w 6, +0x6.w 0, +0xAA.w 0, +0xA4.b 1, +0xC0.b 1`, HP 900) or awake hostile Shivat, pinned at hero X + 45 with the depth offset dz = 0, 4, 8, -4, -8 (the sweep A
torch swing missed a pinned dinosaur, most likely a depth miss); torch (variant 22) charged and tapped; also `spawn E 1 var=3` (blast) on the dinosaur and `spawn B 45` (flame vent). Read `pal` (0x84), HP, `st`, class 8 flame count.

**D3 monsters against enemies.** Zeke (class 82, forced modes 2, 4, 6 as the old `z-force-*` recipe) with a thug standing next to the hero; hostile Hopper with two thugs and the hero pinned far right, 1,800 frames.

**D4 Shivat attacks on a hero.** Hostile Shivat pinned at hero depth, 60 px right, god off, hero HP set to 100 every 25 frames, level 0 and level 20, 2,400 frames; list hero drops against Shivat `atk` (0x38, 0x39, 0x3A, 0x3B).

**D5 KYOURAN and DIFFENCE.** Hostile Hopper, HP 900, hero spamming jabs for 2,400 frames; watch `+0xA8` (8 = KYOURAN, 0xE = DIFFENCE) and `+0xB9`, `+0xBA`.

**D6 area change.** Spawn a Hopper or Shivat in Episode 5 area 0, then fire the end of area event as the jump does (set `G.$792` to the event address and `G.$7A8` to its index, camera X to its X, hold `G.$7A9 = 1`);
the event address is computed from the table at 0x21B2A in the generator. Watch whether the object exists after the area changes.

**D7 grab escape.** Old `z-force-c` (Zeke forced to the dive grab at t = 900, grab hold at t = 1011 to 1042), with the hero mashing from t = 1005: nothing, left and right alternate, up and down, jump, attack, all.

**D8 3 players.** `jump 5 0 char=jack,hannah,mess`, Zeke and a hostile Hopper, log `+0x76` target every 5 frames for 2,000 frames.

**H2 bullets.** Flat ground, spawn class 59 (variants 0 and 5) at hero X + 80 and heights 20, 40, 64, hero taps A every 3 frames for 40 frames (and one jump kick); watch the bullet object and hero score.
**H3 hunter** against a hero. **H4 Bludge**: old `bl-eat` (Bludge, food 4/3, weapon 0/0xC, treasure 9/5) on flat ground with aggression 7 and the hero spamming. **H5**: class 64 variant 0x200 pinned next to the hero (knock down),
blast `E 1 var=3` on a god-off hero (levels 0 and 15), `spawn E 0 var=24` under the hero with A pressed every 5 frames. **H6**: `jump 7 0`, walk right 1,500 frames, count class 70. **H7**: `jump 2 0 CAMX` with 1, 2 and 3
players at a poacher trigger. **H8**: carriers (classes 63, 64) and bikers (class 70) with `hold 0xFF850E 1 1` and `... 2`. **H9**: first poacher bullet at level 0 on a fresh game.

**C1 to C6.** C1: Episode 7 area 3, spawn the Slisaur starter (pool B 33), kill each Slisaur with `objset hp 0`, watch rank. C2: no jump, 3,600 frames, watch `@0xFF84F6.w` (with and without `450-457:C`). C3: `hold 0xFF84CF 1 1`
from power on, screenshots of select and tally. C4: `jump 7 1`, hero right and attack, `killall` pulse every 100 frames, watch the game frame counter for a stop. C5: boss at HP 1 once, the hero finishes it, watch `G.$4D4`, `$4D5`, `$4DD`, `$4DB`, `$516`, `$503`, hero
`st`; Butcher, Slice, Hogg, Morgue, Tyrog, Fessenden, each hero. C6: `jump 5 1`, walk right with attacks, count pool B controllers.

## 4. Fix list for M9-11 to M9-14

Empty: no difference was measured. One note for the port from reading old logs: the sweep A statement "Vice stays in his intro state when spawned" is explained by a 230 frame run (the intro is 710 frames).
`M9-sweep-a.md` section 4 can say the Vice factors are still open, not blocked.

## 5. Corrections for `D5-errata.md`

None yet.

## 6. Paused here, next steps:

Answered: none by a new run. Closed by reading: D4c-8, D4e-2, D4e-8 (and D3b-3 for the foot, from older work). Planned only: everything else in section 1.

Left (priority order): (1) boss and dinosaur items of section 1.1; (2) camera and time items of section 1.2; (3) the sound items of section 1.3 and the sound recorder check.

Probe inputs ready to run: none. Ready: the reader `re/out/probe/_input/m907_an.py` (untested on a new log), the recipes of section 3, the constants of section 2.1. To restart:
1. Write `re/out/probe/_input/m907_gen.py` (helpers: segment with jump, god, level, sound watch line, per frame pins, A tap lists) and make `m907_a.txt` (bosses, B1 to B6), `m907_b.txt` (dinosaurs, D1 to D8), `m907_c.txt` (humans, H2 to H9).
2. Run each batch with `PR_INPUT=re/out/probe/_input/m907_a.txt PR_SHEET=0 PR_TIMEOUT=560 re/harness/probe/probe.sh m907_a` (foreground, one MAME process). Check the sound recorder on the first batch.
3. Batch d: camera items (C1 to C6, the Episode 8 and Episode 5 boss kills); batch e: sound items; batches f and g stay free for reruns (limit is 7).
4. Fill sections 1 and 4 with numbers, the source tag and the fix item (file, routine, D5 task) for each difference.
