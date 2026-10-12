# D3b: dinosaurs and flying creatures (every one, with numbers)

Written 2026-10-12 by the D3b research agent. Output of task D3b of `plan/game/NEW-PLAN.md`. Nothing here is ROM data:
only descriptions, numbers and addresses.

Source tags: **R:** ROM address or repo file, **M:** MAME probe (the segment name is given, scripts in
`re/out/d3b/`, logs in `re/out/probe/d3b-*`, git-ignored), **W:** web. **MAME wins over the web.** `[I]` marks an
inference that no run or listing proves yet. `G.$xxxx` is RAM `0xFF8000 + xxxx`. Pool A classes are numbered by
`+$20 / 4` (the number the scripts print as `A#nn`). Box records are `zo, zs, xo, xw, yo, ys` (`re/specs/anim/SPEC.md`
4.2); a box is mirrored when the creature faces right. Heights (`y`) are pixels above the floor.

How the work was done: I read the handlers of classes 22, 24, 29, 38, 82 and the helpers they call (the listings are
in `re/out/d3b/*_dis.txt`), searched the whole ROM for every spawn of every creature class (section 8), looked at the
asset dump (`re/out/assets/anims/dinosaur_*`), and then ran probe batches (about 105 segments in 10 headless MAME processes: a scouting run `s1`, then `b1` to `b8`, `b8` twice; generators `gen_b1.py` to `gen_b4.py`, plain files `b5.txt` to `b8.txt`). Where my first probe was
wrong (a forced mode without its init, a kill-all hold that also killed the test poacher) I say so and the numbers
come from the repeat.

## 0. Summary of the findings

1. **Five real creatures act in the game** (section 1.1): Shivat (A24, T. rex), Mack (A38, triceratops), Rock Hopper
   (A29, velociraptor), Zeke (A82, pteranodon) and the brontosaurus foot (A22, with its helper A23). All are "monsters":
   they live in the pool A hostile list `0x6B64` and hurt heroes **and enemies** (section 1.2). They never hurt each other.
2. **They are not enemies in the wave sense.** `+$7D = 0xFF` (not counted in the census `G.$4F3`), no kill-all check
   (boss death and wave timeouts do not touch them), no kill score, no drop, never `0x325E6` or `0x24B36`. A dinosaur
   that reaches 0 HP does not die: it turns calm and walks (or flies) away. A hit still scores the normal 100 / 200 / 400
   points of the hero's attack record. [R: no calls in the 5 handlers; M: z-kill, m-kill, s8-kill, h3-kill: score +0x100
   only, from the hit]
3. **Hostility model** (section 1.4, measured). Names of the behaviour modes are in the ROM debug strings: MICHI
   (dormant), KIMAMA (carefree), MATOWARI (follows), OKORI (angry), KYOURAN (berserk, Hopper only), MANZOKU (satisfied,
   leaves), COLOR (colour flash), DIFFENCE (block, Hopper only). A **sleeping Shivat cannot be hit by anyone** (`+0 = 2`);
   the only thing I found that wakes it is a **dinosaur hunter (A79)**: its third swing calls a helper (0x533FA) that puts the
   dinosaur into a hit reaction without damage, then the dinosaur flashes for 88 frames and turns hostile. A **neutral**
   dinosaur (Hopper in KIMAMA / MATOWARI, Shivat in MATOWARI) also turns hostile **by itself** after 600 to 900 frames
   (a 1 in 16 roll every 10 frames) or leaves after a time limit. Any hit on a neutral one makes it hostile (flash, then
   OKORI). Mack (always hostile) and Zeke (always hostile) have no neutral phase. [M: s4-wake, h3-hit, h3-neutral]
4. **Zeke** (section 2): HP 20 but he takes **double damage** (type factor 64 instead of 32), so one punch or one jump
   kick kills him. He flies in world coordinates, 320 px above the floor at spawn, patrols at heights of 74 to 203 px
   (level 0, the "altitude check" runs only once per wing cycle, so he overshoots), and every 180 frames rolls for one
   of three attacks: a low swoop (atk 0x5B, 10 damage at level 0), a triple peck (atk 0x5C, 10 each) or a dive grab
   (hold 31 frames, then 18 damage). **Why only a jump reaches him:** his hurt box spans `y-7 .. y+47`, so at patrol
   height (box bottom 67 to 196) a standing punch (top between 65 and 89 for Jack) misses; a jump kick (box top between
   89 and 113) reaches only the lowest dips. During his swoops and pecks he comes down to 15 to 90 px and a standing
   hero can hit him. A hero who is in the air when the dive ends is not grabbed. [M: z-hit-*, z-force-*, z-solo-*]
5. **Zeke carries the grabbed hero toward world X = 0xC0**, not toward the screen centre: the original reads a word
   at `a6 - 0x78BC` (a typo for the camera X) which is 0 in RAM, so the target is `0 + 0xC0`. On the first screen of
   an area he carries the hero right, everywhere else left. [M: z-grab-far x=1827 goes left, z-force-c x=64 goes right;
   RAM at 0xFF5038, 0xFF61B8, 0xFF5660 is 0 at t=500..2000]
6. **Mack** is a charger, not a sleeper (the sleeper variant 4 is never scripted): it ignores the X of its spawn
   record, enters from the screen side the record is on, gallops across in strides (32 frames of motion + 22 frames
   standing), hits with 15 base damage and knocks down, turns after 120 frames and charges back if the record asked for
   two passes (`+5B = 2`), then is freed. HP 128 at every level.
7. **Rock Hopper** (section 4) is Vice T.'s pet in Episode 1 (Vice's init spawns it; the scripts never do) and in
   Episode 5 it is scripted twice with variant 0x200 (neutral at first). 12.5 percent block chance when enraged.
8. **Brontosaurus** (section 6): not "legs that crush at two places" but one **huge foot** (A22) with a helper (A23) that
   rises off the top of the screen, hovers over the heroes, then slams down: 30 damage and a knockdown, strong screen
   shake. Two feet in Episode 6 area 1. It cannot be hit.
9. **Unused creatures** (section 8): A12 = the golden raptors of the opening movie (the front end spawns them), A28, A37, A39,
   A44, A47 = sprite viewers of the test mode (they read the input bytes `G.$17` / `G.$1C` and change the animation
   number), A51 = the carcass in the Butcher's yard (class 5 spawns it), A46 = the long tailed flock bird (pool B 72),
   A78 and A80 = harmless ground critters of Episode 2. **No code spawns a gameplay creature that the scripts do not
   already place**, except Vice's Hopper.
10. **Port**: classes 22, 24, 29, 38, 82 are all MISSING (freed at once), the four monster hit passes are missing. Only
    the sky scenery 49, 50 (and pool B 73) exist. Port task list in section 13 (about 15 agent hours).

## 1. Shared facts

### 1.1 The class table (everything creature-like in pool A)

| Class | Handler (code size) | What it is | Spawned by | Port |
| --- | --- | --- | --- | --- |
| 24 | 0x4838C (13,252 B; own tail 0x4BE70 is Slice) | Shivat, T. rex, name 7 | script Ep 2 (1), Ep 5 (1) | MISSING |
| 29 | 0x4DCF6 (7,708 B) | R Hopper, velociraptor, name 4 | script Ep 5 (2), Vice's init (0x4DCC6, via class 18 var 0 at 0x425C2) | MISSING |
| 38 | 0x5341C (2,218 B) | Mack, triceratops, name 0x11 | script Ep 2 (2), Ep 5 (2) | MISSING |
| 82 | 0x606D4 (3,720 B) | Zeke, pteranodon, name 0x21 | script Ep 5 (1), Ep 7 (1), Ep 8 (2) | MISSING |
| 22 + 23 | 0x47C42 (1,572 B) + 0x48266 | brontosaurus foot (A22) and its helper (A23, +$A2 = parent) | script Ep 6 area 1 (2) | MISSING |
| 49 | 0x56A70 | background bird controller, harmless | script Ep 1 | DONE (`scenery.rs`) |
| 50 | 0x56C80 | sky bird (the "red bird"), harmless | script Ep 1, pool B 73, boss code | DONE |
| 46, 48 | 0x5656C, 0x56924 | long tailed flock bird and its trail piece, harmless | pool B 72 (Ep 2) | MISSING (D3e) |
| 78, 80 | 0x5F198, 0x5FF2E | ground critters, harmless | script Ep 2 area 0 | MISSING |
| 12, 28, 37, 39, 44, 47, 51 | see section 8 | not gameplay creatures | front end, test mode, class 5 | not needed |

Registration: every creature calls `0x4C0A` every frame (count at `0x6B64`, list pointer at `0x6B66`) and `0x49CA` / `0x49C0` (draw).
None of them calls `0x4BEA` (the list `0x6726` of enemies that heroes hit). [R: handler tails 0x4850A (Shivat), 0x53570 (Mack), 0x60784 (Zeke), 0x4E0AE (Hopper), 0x47D88 (foot)]

### 1.2 The passes that involve monsters (order of `0x10546`, every frame)

| Pass | Attackers | Victims | What happens |
| --- | --- | --- | --- |
| 0x11A9E, per attacker 0x11AD8, hit 0x11B36 | list `0x6720` (the heroes), box mode `+$2E` = 4, 6 or 8 | monsters `0x6B64` | the hero hits the monster. Victim must not have `+0` bit 1 set, one hit per attack serial (`+$5E`). Guard: if `+$87 != 0` and the attacker is not in box mode 6 and its `+$C8` is not 0x12 or 0x13, the guard routine `L[0x11CE4 + +$87]` runs (only the Hopper uses it, 0x4F3E8) and may block. Damage = hero table 0xA26FC row, scaled by 0x2B904. Victim reaction code from the record via table 0x11CC4 |
| 0x1132A | list `0x6722` (the attack list filled by `0x4BDA`: normal hero attacks and weapon objects, box mode 4) | monsters | same damage table (index by the attacker's `+3`), no guard; the attack serial `+5E` stops a double hit by both passes |
| 0x11CEC, per victim 0x11D32 | monsters with box mode `+$2E` = 4 | heroes (`0x6718/0x6720`) | the monster hits the hero. Record 0xA2EA4, damage through `0x6422` (type x level scale, section 1.5). The hero gets the record's reaction code (`+$C8 = 0x22` forces 0x22), `+$CB = 0x3C`, `+$F6 = 0x78` |
| 0x11E56, per victim 0x11EDC | monsters with box mode 4 | enemies of `0x6726` | **raw record damage, no level scale**, credit `+2 = 0xFF` (nobody gets score), facing of the victim turned away from the attacker. A victim with `+$7D < 0` keeps at least 1 HP. Victim reaction from the record |
| 0x12462 | monsters | the Cadillac object `$77A2` | car hits (Episode 3 only; no dinosaur is in Episode 3) |

So a monster hurts every hero it touches and every enemy it touches, and heroes (and their weapons) hurt monsters. No
pass lets a monster hit a monster. [R: 0x10546, 0x11A9E, 0x1132A, 0x11CEC, 0x11E56]
The hit box test is the usual `0x128F8` (z, x, y order, `re/specs/anim/SPEC.md` 4.3). The box mode table that is read
for the victim (`+$2E` of its current frame, even values): in the hero pass 0x11AD8 the frames with box mode 8, 0xA and 0xE are immune and
0, 2, 4, 6, 0xC are hittable; in the pass 0x1132A only 8 and 0xA are immune. Zeke's hurt frame (anim 2) has box mode 8 and no body box, so he
cannot be hit while he recoils. [R: tables 0x11B26, 0x11386]

Measured: Mack run through two Ferris: each lost 15 HP per pass (raw, 44 to 29). A hostile Shivat took Ferris 44 to 34
to 24 to 14 (10 per tail hit). [M: m-vs-thug, s3-vs-enemy]

### 1.3 Fields the dinosaur code shares (offsets in the 0xE0 byte pool A object)

| Off | Meaning in these classes |
| --- | --- |
| +$0 | active: 1 normal, 2 = "reacting / not hittable" (a sleeping Shivat and the brontosaurus foot keep 2 for good) |
| +$4, +$5, +$6, +$7 | routine (2 alive), sub routine (0 AI, 2 hit reaction, 4 grabbed or grab sequence), AI state, AI sub state |
| +$5C | "leave X": camera X at which it leaves (0 = never). Records give 0x3D1, 0x3E0, 0x270, 0x320, 0x400, 0x480, 0x569, 0x72F |
| +$7D | 0xFF: not counted by the census |
| +$87 | guard index (0, except Hopper = 4) |
| +$A0 (Zeke, Mack) / +$A8 (Shivat, Hopper) | behaviour mode (a word) |
| +$A4 | "already angry" flag (Shivat, Hopper, Mack): 0 = neutral, 1 = hostile; flipped by the flash routine |
| +$C0 | 0xFF = dormant, "hunt me"; 0 = awake neutral; 1 = awake hostile (read only by hunters, A79) |
| +$C1 | 1 = a hunter has claimed this dinosaur |
| +$23 | palette override (bit 7 set: low 5 bits = palette) |
| +$70 | holder link (the hero it holds) |
| +$76 | target (a hero object address, or an enemy when neutral and hunting enemies) |

Palette bytes: normal green `0x9E` in Episodes 2 and 5 for Shivat, Mack calm flash `0x9E` / `0x9F`, Hopper `0x9E` (Ep 5),
enraged `0x9F`; Ep 1 and 2 use `0x91` / `0x92` / `0x98` (section 10).

### 1.4 The hostility model, step by step (measured)

Developer names (R: ROM debug strings at 0x490CE for Shivat, at 0x4F1C4 for the Hopper). **The numbering below is the Hopper's.** Shivat has no KIMAMA, KYOURAN or DIFFENCE and numbers its modes 0 MICHI, 2 MATOWARI, 4 OKORI, 6 MANZOKU, 8 COLOR (R: table 0x48510); Mack and Zeke use their own mode words (sections 2 and 3).

| Mode | Name | Meaning |
| --- | --- | --- |
| 0 | MICHI | dormant: lies still, roars and shows "Z" marks. Shivat always starts here. Not hittable |
| 2 | KIMAMA | Hopper: carefree wander, harmless |
| 4 | MATOWARI | Hopper and Shivat: follows or paces near the heroes, harmless |
| 6 | OKORI | angry: attacks heroes and the nearest enemy |
| 8 | KYOURAN | Hopper only: berserk (three pounces in a row, then back to OKORI) |
| 0xA | MANZOKU | satisfied: roars, walks away, freed off screen |
| 0xC | COLOR | colour flash 16 toggles (about 88 frames), then OKORI |
| 0xE | DIFFENCE | Hopper only: block, slides back |

The flow as seen in MAME (frames after the first game frame of the segment):

1. **Shivat asleep.** `+0 = 2`, `+C0 = 0xFF`. Pool C class 30 "Z" objects (variant 0x200, at about 103 px in front of the head)
   appear every 50 + (rand & 63) frames and last until it wakes. Jack's punches pass through it. [M: s3-sleep, s3-hit, s8]
2. **A hunter (A79) walks to it** (claims it at once with `+C1 = 1`, 214 px in about 150 frames, then waits), swings three times
   (attack id 5, frame boxes 17, 17, 18) and the **third swing** pokes the dinosaur: `+4 = 0x02020000`, `+7A = 0`,
   `+68 = hunter`, `+2 = 0xFF`, hit stop 8 (R: 0x533FA called from 0x5F9F8 when frame byte `+$2D` of the hunter is negative).
   The dinosaur flinches for 7 frames with **no HP loss**, then (not yet angry, `+A4 = 0`) enters COLOR. [M: s4-wake, s5-wake:
   Shivat reacts at t=274, `+68` = the hunter's address, HP stays 60]
3. **COLOR** (88 frames): palette toggles between `0x9E` and `0x9F`, intervals shrinking 8, 8, 7, 7, 6, 6, 5, 5, 4, 4, 3, 3, 2, 2 frames
   (measured on the Hopper, table 0x4F4C8, 16 or 17 toggles; Shivat and Mack use the same kind of table, 0x49448 and 0x53956, with `+A7 = 0xF`); `+A4 = 1`; `+0 = 1` again at the end; then OKORI with the palette `0x9F`. [M: h3-hit, s4-wake]
4. **OKORI**: attacks the nearest hero or enemy. The hunter that woke it is usually the first victim (10 per hit). The hunter itself sets the dinosaur's `+C0 = 1` and
   goes back to ordinary poacher behaviour. [M: s4-wake: hunter 100 to 90, 80, 70]
5. **Hit by a hero when neutral** (Hopper in KIMAMA / MATOWARI, Mack with `+A4 = 0`, an awake Shivat in MATOWARI): hit reaction
   (jab 12 damage, flinch 9 frames, poise 10 to 9), then the reaction recovery sees `+A4 = 0` and enters COLOR, so **any hit makes it hostile**.
   [M: h3-hit: Hopper 44 to 32, COLOR from t=31, OKORI from t=120]
6. **By itself** (Hopper): in KIMAMA / MATOWARI a counter `+AA` counts frames; after 600 frames every 10th frame there is a 1 in 16
   chance of COLOR (self enrage), after 1200 frames it leaves (MANZOKU). In OKORI the counter runs to 900, then every 10th frame
   a 2 in 16 chance of MANZOKU. Measured: Hopper MATOWARI from t=104, COLOR at t=887, OKORI at t=976, MANZOKU at t=2042
   (1066 frames in OKORI). Shivat: MATOWARI after 900 frames 1 in 16 per 10 frames to COLOR; OKORI after 1800 frames 1 in 16
   per 10 frames (counter `+AC`) to MANZOKU (R: SPEC 11, listing 0x485F2, 0x4878C). [M: h3-neutral]
7. **Leave X**: when the camera X reaches `+5C` the dinosaur goes to MANZOKU at once (Hopper, Shivat in MATOWARI / OKORI, Mack in
   its charge mode sets HP 0 and calms). Zeke leaves straight up (section 2.8). A sleeping Shivat has no leave check: it is freed
   by the off screen test `0x173C` (window X - camX + 0xC0 outside 0 .. 0x2FF).
8. **0 HP**: the dinosaur finishes its hit reaction, then MANZOKU: Mack flashes 88 frames then walks off left at 3 px per frame;
   Shivat and Hopper roar and walk off. No score, no drop, not counted. [M: m-kill, h3-kill, s8-kill]
9. **Fire.** The hit reaction of Hopper and Shivat has a burn branch: when the attack record's reaction maps (table 0x11CC4)
   to `+6 = 4` (record reaction 0xA or 0xC: flames, explosions, the torch swing [I]), the dinosaur spawns burn flames
   (0x2AD98, effect class 8) and takes palette `0x84`. A burn hit also ends the neutral state like any hit. Not measured
   (open item). [R: 0x4EE1A, 0x48D8A]
10. **Poacher bullets**: earlier notes (`mechanics-seen.md` D02) say that poachers' hits wake a dinosaur. I could not reproduce it: a poacher in the line of a sleeping Shivat (s3-poacher)
    did not wake it, and the only wake source in the code is the hunter. Open item 1.

Web check (a search summary, I did not open the pages): a Shivat sleeps in stages 2 and 5 (matches), draining a dangerous dinosaur makes it change colour
slightly and become harmless (matches MANZOKU), and a fan wiki says Vice wakes Rock Hopper (matches Vice's init spawning it). MAME decides where they differ.
[W: https://en.wikipedia.org/wiki/Cadillacs_and_Dinosaurs_(video_game), https://www.arcade-history.com/game/370/]

### 1.5 Damage numbers

**To a hero** (R: SPEC 6.6, `0x6422`, tables 0x2C3AE, 0x2C896, 0x2C88E, 0x2C996): `d = base * scale[type][level] >> 5`, then the small
random cut when the hero is hurt, then the hero factor (Jack 32, Mustapha 34, Hannah 35, Mess 30 over 32), then the low HP factor
(25 to 33 over 32; 33 when HP is 100), and the mercy rule (a hit that would kill with at most 4 HP over leaves 1 HP).

Attack records used (R: table 0xA2EA4; fields damage, poise, reaction, hit stop, sound, spark):

| Creature | Attack id | Base damage | Reaction | Hit stop | Used for |
| --- | --- | --- | --- | --- | --- |
| Zeke | 0x5B | 10 | 0 flinch | 16 | swoop (mode 2) |
| Zeke | 0x5C | 10 | 0 flinch | 16 | peck (mode 4) |
| Zeke | 0x3B | 18 | 0 flinch | 15 | dive grab release |
| Mack | 0x59 | 15 | 6 knock down | 20 | charge (0x5A, 15, knock down, only for the unused jump variant 2) |
| Hopper | 0x20 / 0x21 / 0x22 / 0x29 | 12 / 16 / 10 / 15 | 0 / 4 knock down / 0x10 push / 0 | 15 / 20 / 5 / 15 | claw / bite and knock down / leap / pounce bite (held) |
| Shivat | 0x38 / 0x39 / 0x3A / 0x3B | 14 / 10 / 10 / 18 | 0 / 4 knock down / 0 / 0 | 15 / 20 / 15 / 15 | claw swipe / tail swing / stomp x3 / grab release (the hero is thrown, state 8) |
| Foot (A22) | 0x4C / 0x63 | 30 / 1 | 4 knock down / 0x12 push | 18 / 16 | slam / touching the rising or resting foot |

Level scale rows (word index = level 0 to 31, `0x2C3AE + name*32`, 32 means 1.0): Zeke 32 x4, 35 x3, 36, 37, 38, 39, 43, 45 x4, 49 x4, 52 x4,
54 x4, 58 x3, 64. Mack 32, 35 x3, 38 x4, 40, 44 x3, 48 x4, 52 x4, 56 x4, 60 x4, 64 x2, 72, 80. Hopper 32 x4, 35 x3, 36, 37, 38, 39, 42,
44 x4, 48 x6, 50 x5, 56 x4, 64. Shivat 32 x4, 35 x4, 38 x4, 44 x4, 48 x4, 52 x4, 56 x4, 60 x3, 68.

| Base | Level 0 | 8 | 16 | 24 | 31 |
| --- | --- | --- | --- | --- | --- |
| Zeke 10 | 10 | 11 | 15 | 16 | 20 |
| Zeke 18 | 18 | 20 | 27 | 30 | 36 |
| Mack 15 | 15 | 18 | 24 | 28 | 37 |
| Hopper 16 (bite) | 16 | 18 | 24 | 25 | 32 |
| Shivat 14 (claw) | 14 | 16 | 21 | 24 | 29 |
| Shivat 18 (grab) | 18 | 21 | 27 | 31 | 38 |

Measured against Jack at 100 HP: Zeke peck 10 at level 0, 20 at level 31; Zeke grab 18 and 37 (36 plus the 33/32 HP factor);
Mack 15 and 38 / 37; Hopper bite 16 and 33; Shivat grab release 18; foot slam 30. [M: z-solo-l0/l31, z-force-c, m-solo-l0/l31,
h3-hostile-l0/l31, s4-atk-14, bronto-0]

**From a hero** (R: SPEC 10.1, 0x2BA2A): HP by level (`0x31454`): Zeke 20 at every level, Mack 128 at every level, Hopper 44 44 52 52 60 68 76
84 92 100 x7 110 x3 120 x3 130 x3 140 x2 150 x5 (levels 0 to 31), Shivat 60 60 68 76 84 92 100 x10 110 110 120 120 130 130 140 140 150 x4
160 180 190 200. Type factor `B[0x2BECE + name*32 + level]`: **Zeke 64 (double damage)**, all others 32. The generic weak point rows are the same
for all (attack ids 16 and 14 do 1.5x). Poise 10 for all; every hit takes the record's poise; poise 0 means the heavy reaction.
Measured Jack jab at level 0: Hopper 12, Mack 11 to 12, Shivat 12 to 13 (60 to 47 to 35 to 23), Zeke at least 20 (one hit kills 20 HP).
The hit also scores 100 (first), 200 and 400 for later chain steps (record byte), and shows the dinosaur's name and HP bar through
`0x16B36`. [M: z-hit-punch-*, h3-hit, m3-hit, s8-a]

### 1.6 Sounds the creature code calls

| Call | Code sent | Used by |
| --- | --- | --- |
| 0xA09E | 0x60 with pan from X | Mack roar (states 2 to 4 and calm), Hopper roar, Shivat roar |
| 0xA0FA | 0x5C | Hopper (pounce / bite) |
| 0xA0CC | 0x5F | Zeke grab release slam, Hopper |
| 0x9796 | 0x72 | Shivat steps and stomps |
| 0xA354, 0x97D2 | pan sounds | Shivat |
| 0xA288 | stomp | brontosaurus foot slam (with `G.$755C = 4`, screen shake) |
| 0x9667E | spawns pool C class 30 (the "Z") | Shivat and Mack sleeping, every 50 + (rand & 63) frames |

The exact sound codes of the hit and death voices are D4e's. Hits use the record's sound byte (0x0C, 0x09, 0x03 ...).

### 1.7 Waves, camera and area change

Dinosaurs come only from the spawn lists (section 9), never from wave records (checked in `stage_scripts.txt`). They do not block a wave
(`+7D = 0xFF`) and do not hold the camera lock. They are not killed by the kill-all of a boss or a wave timeout. They disappear when the
area is reset (area change fade) or when they pass the off screen test `0x173C`. A dinosaur that the heroes outrun is simply freed once the
camera has moved more than about 0xC0 px past it. [R: SPEC 8, 0x173C]

### 1.8 The hunter (pool A class 79, 0x5F5D0, "poacher that hunts dinosaurs")

Hammer / Elmer / Wrench look alike (names 0x1E to 0x20, kill score per name). Its target picker `0x5F6CA` scans all pool A objects for classes
24, 29 and 38 whose `+C0` is negative and whose `+C1` is 0, takes the one nearest in X and sets `+C1 = 1`. If there is none it fights like a
normal enemy. It walks up (214 px in about 150 frames in my probe), swings three times (attack id 5, 13 damage record, the third swing carries the poke flag), and `0x5FA5A` sets the
dinosaur's `+C0 = 1`. When it dies it clears `+C1`. Used in Episode 2 (the two of the last wave of area 0, at the 0x568 lock) and Episode 5 (one record at X 0x3A0, trigger 0x3C8, and the
three of wave 1 at the 0x3D0 lock). D3a owns the rest of its AI. [R: 0x5F6CA, 0x5F9E6, 0x5FA5A, 0x533FA; M: s4-wake]

## 2. Zeke, the pteranodon (pool A class 82, handler 0x606D4, 3,720 bytes)

The "bird dino that flies and gives damage". Episodes 5 (1), 7 (1) and 8 (2). Never part of a wave.

### 2.1 Data

| Item | Value |
| --- | --- |
| Name id | 0x21 ZEKE (`+$84 = 0x84`), HUD name and HP bar shown by `0x16B36` at the first hit |
| HP | 20 at every level (`0x31454`, 0x31424 at init); poise 10; damage taken is **x2** (type factor 64, section 1.5) |
| Animation table | 0x611D2, 8 animations (list below); box table 0x103000 (direct, 12 byte records, `+$4A = 0`) |
| Palette | base 0 (the OBJ set of the area); **Episode 8: `+$23 = 0x9D`** (red orange) from init; when it flies away or dies `0x9E` (Episode 7: `0x9F`) |
| Score, drop | none (only the hit's 100 / 200 / 400); `+$96` of the record is ignored |
| Census | `+$7D = 0xFF`, never counted, no kill-all check |
| Hurt box (flying) | id 216: `z 0..8, x -45..+25 (70 wide), y -7..+47` relative to Zeke's x and y (mirrored when facing right). During the swoop pose id 217 `(0, 8, -52, 67, -14, 56)`, during the peck pose id 219 `(0, 8, -41, 55, -9, 72)` |
| Attack boxes | swoop pose (anim 3, box mode 4) id 218: `(0, 8, -51, 16, -23, 17)` (the head and claws, 51 px in front, 6 to 23 px **below** his y). Peck pose (anim 5, mode 4) id 220: `(0, 8, -17, 27, -18, 13)` (the feet). The dive pose (anim 7, mode 0xC) has no attack box (it grabs) |
| Floor | `0x12D4E` every frame: y is never below the floor (`+$54 + +$56`) |
| Off screen | `0x173C` frees him when `x - camX + 0xC0` is outside 0 .. 0x2FF; there is no clamp to the screen, he flies out of view at times (x = -8 seen) |

Animations: 0 slow flap (7 frames, 7 ticks each = 49 ticks, box mode 0), 1 fast flap (7 x 3 = 21 ticks), 2 hurt pose (1 frame, 9 ticks, box mode 8,
no body box: immune), 3 swoop pose (9 ticks, mode 4), 4 fast flap after a pull up (7 x 2 = 14 ticks), 5 peck pose (9 ticks, mode 4), 6 falling or
leaving (8 x 6 = 48 ticks, no boxes), 7 dive pose (7 x 3 = 21 ticks, mode 0xC). Pictures: `re/out/assets/anims/dinosaur_a82_zeke/`.
A flap cycle ends when `0x121E` returns carry, which is when the flight code does its altitude check (section 2.3).

### 2.2 Spawn

The record gives X, Z, var 0 and `+5C` (leave X). Init (0x606E8): `+$A0 .. +$A4 = 0`, `+$7D = 0xFF`, target picked by `0x32032`
(with the extra rule below), snap to floor, **`y = floor + 0x140` (320 px above the floor, above the top of the screen)**, routine 2.
The script X is an absolute map X; with trigger 0xD0 the Episode 5 Zeke (X 0x180) starts 176 px from the left screen edge, above the screen.
Target rule (0x60D7C): take the target of `0x32032`; if that hero's byte `+$116` is set (another flyer picked him) try the next hero by
`(+2 + 1) & 3`, up to three times; the chosen hero gets `+$116 = 1`. So two Zekes (Episode 8) pick different heroes when 2 or 3 play.
[R: 0x60D7C; the clear of `+$116` was not found, it is probably reset by the player code `[I]`]

### 2.3 Mode 0, patrol (0x60804)

Each frame (in the AI branch, which is skipped while `+$7A != 0`):

1. **Leave check** (only here): `+5C != 0` and `+5C <= camX` -> mode 8 (section 2.8).
2. `$6 = 0` (init, once per entry): new target, a new altitude pick (below), speed table 3 (`0x17A4`: 256 per frame in X), `+A3 = 0`, **`+80 = 180`**,
   `+A4 = 600`, `$6 = 2`.
3. `$6 = 2`: step the animation (`0x121E`); on a flap cycle end run the altitude test; every frame run the approach (below); then `0x1862`
   (move by `+22` and the speed table) and `0x18E0` (Y only physics).
4. **Altitude.** `+A2` is an index 0 to 4. Target height above the floor = `W[0x6141A + 2*idx]` = 144, 136, 128, 112, 108. When the flap cycle ends the test
   asks "has he passed that height?" (going up: y above it; going down: y at or below it). If yes, the next index comes from the table at 0x6137A
   (5 rows of 32 bytes, row = the old index, entry = `rand & 31`) and `vy = +0xC0` (up, 0.75 px per frame) if the old index was not smaller than the
   new one, else `-0xC0` (down). Animation 0 (slow flap, 49 ticks) is shown when `vy < 0`, animation 1 (21 ticks) when `vy >= 0`. Rows:
   old 0: new 1 (1/32), 2 (5/32), 3 (9/32), 4 (17/32); old 1: 0 (3/32), 2 (3), 3 (11), 4 (15); old 2: 0 (11), 1 (5), 3 (5), 4 (11); old 3: 0 (15), 1 (11), 2 (3), 4 (3);
   old 4: 0 (17), 1 (9), 2 (5), 3 (1). Because the test happens only once per flap cycle (49 or 21 frames at 0.75 px per frame) he **overshoots by up to 37 px**:
   measured patrol heights at level 0 run from 74 to 203 px (level 31: 74 to 271). From spawn he descends from 320 at 0.75 px per frame (about 4 seconds).
   [M: z-solo-l0, z-solo-l31]
5. **Approach (0x60E70, row `d0`)**, evaluated every 7th frame (`+A3` counts 6 to 0, on the other frames it returns "not there"): the destination is
   `(target x + dx, target z)` where `dx` is the row's first word, made **toward the screen centre** (`camX + 0xC0`: if the hero is right of the centre the
   offset is minus, else plus). The direction `+22` (0 to 31) turns toward the destination by at most 2 per evaluation (22.5 degrees per 7 frames); within 2 it snaps.
   Arrival test per row (`(dest x - x + a) <= b` and `(hero z - z + c) <= d`): rows are 10 bytes at 0x614DC:

   | Row (`d0`) | Used by | dx | X window | Z window |
   | --- | --- | --- | --- | --- |
   | 0 | patrol | -104 | +-24 | +-24 |
   | 0xA | swoop | -144 | +-32 | +-8 |
   | 0x14 | peck | 0 | +-32 | +-8 |
   | 0x1E | dive | -16 | +-16 | +-8 |

   Horizontal speed is `256 * cos(dir)` (about 1 px per frame at most). The 600 frame timer `+A4` is reset on 6 of every 7 frames, so it never runs out (dead code).
6. **Attack roll.** `+80` counts down 180 frames; at the end it is reset to 180 and a mask is chosen from the level band `(level & 0x18) >> 1`:
   `0xFF800000` (levels 0 to 7), `0xFFFF0000` (8 to 15), `0xFFFFFF00` (16 to 23), `0xFFFFFFFC` (24 to 31), read at 0x6142C. A random `rand & 31` bit that is set
   starts an attack: **9/32, 16/32, 24/32 or 30/32 per roll**. So at level 0 the average wait is about 640 frames (my level 0 run: first attack at t=3262),
   at level 31 about 190 frames (first attack at t=185). After every attack the patrol restarts (`+80 = 180` again).
7. **Which attack** (table 0x6143C, 5 rows of 32 bytes by the altitude index, `rand & 31`): 0 = swoop (mode 2), 1 = triple peck (mode 4), 2 = dive grab (mode 6):

   | `+A2` | swoop | peck | dive |
   | --- | --- | --- | --- |
   | 0 (144) | 17/32 | 12/32 | 3/32 |
   | 1 (136) | 14/32 | 11/32 | 7/32 |
   | 2 (128) | 12/32 | 12/32 | 8/32 |
   | 3 (112) | 9/32 | 11/32 | 12/32 |
   | 4 (108) | 4/32 | 14/32 | 14/32 |

### 2.4 Mode 2, the swoop (0x60906), attack 0x5B

Approach with row 0xA (to a point 144 px toward the screen centre from the hero, depth within 8). **Launch** (facing the hero, anim 3): `vx = +-256` (toward the hero),
`vy = +768`, `ay = -32`, attack id 0x5B (10 base damage, flinch), `+5A = 0`, a new attack serial. He rises, turns over at +24 frames and comes back to the launch
height at +47 frames (second table read at 0x6150C: `vx = +-256`, `ay = -32` again, so he keeps accelerating down). When his height above the floor is under 48 (0x30)
he pulls up: `ay = +80`, `vy` capped at 1280, anim 4, and after the animation ends and the height is above 192 (0xC0) he returns to mode 0.
Measured (forced at t=900): approach 72 frames, launch t=972, back at launch height t=1019, lowest point 46 px at t=1031, patrol again at t=1101 (about 130 frames
from launch). While he falls the attack box (51 px in front, 6 to 23 px below him) is live (anim 3, box mode 4); at the low point it is at 23 to 40 px, i.e. at
a standing hero's chest. [M: z-force-a]

### 2.5 Mode 4, the triple peck (0x60A76), attack 0x5C

Three passes (`+AA = 3`). Approach with row 0x14 (directly above the hero's depth, within 8). **Launch** (anim 5): `vx = +-128`, `vy = +768`, `ay = -64`, attack 0x5C
(10 base, flinch), on the last pass `+5A = 1`. A pass ends when the attack connected (`+59 != 0`, the hit stop of the record) or when he is falling and under 64 px;
then the pull up (`ay = +768`, `vy` capped at 1024, then anim end and height above 140). Next pass or, after the third, mode 0.
Measured (forced at t=900): approach 79 frames, launch t=979, first hit t=1003 (+24 frames, height 92, 10 damage), pull up, second launch t=1025 from y 169, second hit
t=1064, third hit t=1232 after a longer re approach (he turned and came from the other side). [M: z-force-b]

### 2.6 Mode 6, the dive and grab (0x60BD4, 0x60FEE), attack 0x3B

`+A2 = 1`, approach with row 0x1E, then the dive: `vy = -512`, `ay = -64`, anim 7 (box mode 0xC). `vx` is **overwritten at once** by the move step (the launch code
falls into `0x1862`), so it is the speed table value of the current direction, 256 at best. When he is falling and under 85 px (0x55) the grab test runs:
the hero must have `+0 = 1`, state `+4 = 0x200` (normal control), `+51 = 0` (**not in the air**), his x within Zeke's x - 23 .. + 24 and his depth within Zeke's z - 15 .. + 16.
If not, mode 0 (a hero who jumps at the right moment dodges: measured, Jack pressed J at t=1000 and was not grabbed). If yes:
the hero gets `+70` link, `+4 = 0x02060000`, `+C8 = 0x40`, `+2E = 0xE` (nobody else can hit him), Zeke holds him **31 frames** (`+80 = 0x1E`, standing still, anim 7), then
flies `vx = +-896` (3.5 px per frame), `vy = +512`, `ay = +8`. Direction: toward world X `0xC0` (section 0, item 5). When his height is above 100 he applies attack 0x3B
through `0x125C0` (18 base damage scaled, hero flinch, `+F6 = 0x78`, sound 0xA0CC) and goes to mode 0. Pressing attack six times during the hold did not free the hero
(M: z6-mash). Measured (z-force-c): approach 79 frames, dive from y 123 to 82 in 31 frames, hold t=1011 to 1042, damage at t=1051 (-18). [M: z-force-c, z-force-c-jump, z-grab-far, z6-mash]

### 2.7 Hit reaction and death (0x60F1E, 0x61140)

A hit from a hero sets `+5 = 2` (any hit pass). The handler: anim 2 (immune frame), `vx = +-768` away from the hero, `vy = +512`, `ay = -32`, HUD bar; when he is falling and back at the
start height (about 31 frames) he checks HP: **0 -> routine 4 (dying)**, else poise 10, `+0 = 1`, mode 0 (**every hit cancels the attack in progress**).
Dying: anim 6, palette 0x9E (0x9F in Episode 7), `vx = +-768` (facing reversed), `ay = +16` (rising), never below floor + 16, drawn and freed off screen. He does not explode and gives no score.
Measured (hp set to 1, one punch): hit at t=32 (+100 points), recoil to t=64, dying from t=65, off screen at about t=180. [M: z-kill]

### 2.8 Leave (mode 8, 0x60D26)

At `camX >= +5C` (checked in mode 0 only): `vy = +256`, `ay = +16`, anim 1, palette 0x9E (0x9F in Episode 7); freed when his height is 512 or more. From a height of 320 that took 65 frames. [M: z-leave]
The Episode 8 Zekes leave at camX 0x72F, just before the 0x730 lock. If the camera runs away from him he is freed by the off screen test instead.

### 2.9 Why only a jump reaches him, and what a hero can do

The hurt box is `y-7 .. y+47`. Jack's standing punch hit him at y = 72 (box bottom 65) and missed at y = 96 (bottom 89). Jack's jump kick (Jack at height 73) hit at y = 96 and
missed at y = 120. [M: z-hit-punch-*, z-hit-jkick-*] Patrol heights are 74 to 203 (level 0), so a jump attack reaches him in the low part of his bobbing (y of about 100 or less) and a punch only at his lowest dips.
In a swoop (low point 15 to 46 px) and at the end of a peck (hit height 64 to 92) he is within reach of a punch; in the dive he is at 82 to 123. One hit kills (20 HP, x2 damage). When he is hit he flies
away and gives nothing. A grounded hero in the dive column is grabbed for 31 frames plus 18 damage, so the answer to the dive is to jump when he comes down.

### 2.10 What the player sees (timeline at level 31, for the acceptance run)

Enters from the top, descends about 4 s, then loops: slow flaps down, fast flaps up, drifting to a point about 100 px toward the screen centre from the hero; about every 3 s an
attack: low swoop (one pass, 130 frames), triple peck (three drops, 10 damage each) or dive (hero held, then thrown a little and hurt). Pecks and the grab flinch the hero, no knock down.

### 2.11 Where Zeke appears

| Episode, area | Record (trigger, X, Z, leave) |
| --- | --- |
| 5 area 0 | trigger 0xD0, X 0x180, Z 0x238, leave 0x3D1 (the road, with R Hoppers, Macks and Shivat around him) |
| 7 area 0 | trigger 0x80, X 0x140, Z 0x50, leave 0x480 |
| 8 area 2 | trigger 0x200, X 0x2C0, Z 0x140, leave 0x72F and trigger 0x400, X 0x4C0, Z 0x140, leave 0x72F |

Episode 8 uses palette 0x9D. With two Zekes the second picks a different hero (section 2.2). Nothing special happens to the waves: the lock at 0x730 (Episode 8) comes after both have left.

## 3. Mack, the triceratops (pool A class 38, handler 0x5341C, 2,218 bytes)

A charging obstacle. Episodes 2 (two records) and 5 (two records). Not a sleeper in the game (the sleeping variant 4 is never scripted).

### 3.1 Data

| Item | Value |
| --- | --- |
| Name | 0x11 MACK (`+$84 = 0x44`), HP **128** at every level, poise 10, `+72 = 0xC8` |
| Tables | animations 0x53BE4 (8: 0 idle, 1 head down, 2 charge run = 6 frames 64 ticks with box mode 4, 3 slide back, 4 to 6 single frames, 7 walk off = 6 frames 36 ticks), boxes 0x103C00 (direct) |
| Boxes | hurt: standing id 91 `(0, 8, -76, 129, 0, 69)`, head down 94 `(0, 8, -87, 155, 0, 76)`, charging 89 `(0, 8, -102, 159, 0, 72)`; **horns (attack) id 92 `(0, 8, -100, 66, 2, 66)`**: 66 px wide, 34 to 100 px in front, height 2 to 68 |
| Speed | `+64` = speed table id 0x22 (peak 1536 = 6 px per frame) |
| Attack | 0x59 (15, knock down, hit stop 20); damage to heroes 15 x scale (section 1.5); to enemies 15 raw |
| Palette | base 0; calm flash `0x9E` / `0x9F` (table 0x5396A); sleeper variant `0x91` (Episode 5: `0x9B`) |
| Score, drop | none |

Variants (`+$26` high byte): **0 = charger** (all records), 2 = "jumper" (leaps with attack 0x5A, 15, unused), 4 = sleeper (mode 0, `+C0 = 0xFF`, roars and Z marks every 50 + (rand & 63) frames, turns into a charger after a hit; unused). The byte `+5B` of the record is the number of passes (`+AA`): 0 or 1 = one pass, 2 = two passes.

### 3.2 Spawn

The X of the record is used only to choose the side: if the record X is **right of `camX + 0xC0`** Mack appears at `camX + 0x1E0` (just off the right edge) facing left, else at `camX - 0x60` facing right. Z and the floor come from the record. Mode 2 (charge), `+A4 = 1` (hostile from the start, so it never flashes).

| Episode | Trigger, X (side), Z, leave X, passes |
| --- | --- |
| 2 area 0 | 0x180, X 0x320 (right), Z 0x30, leave 0x480, 1 pass; 0x280, X 0x420 (right), Z 0x30, leave 0x480, **2 passes** (`+5B = 2`) |
| 5 area 0 | 0x100, X 0x2E0 (right), Z 0x220, leave 0x270, 1; 0x140, X 0x100 (**left**: it runs left to right), Z 0x240, leave 0x320, 1 |

### 3.3 The charge cycle (mode 2, 0x535CE)

Each state counts frames in `+80`:

1. State 0: anim 0, `+80 = 30`. State 2: wait 30 frames, then sound 0xA09E (roar, code 0x60), anim 1 (head down), `+80 = 30`. State 4: wait 30 frames, then pick the target (0x539C4: facing left the hero with the **largest X**, facing right the hero with the smallest X, i.e. the one nearest to a Mack that comes in from that side; if nobody is in the game it targets itself), attack id 0x59, new serial, `vz = 0`, `dir = 0`, `+AA` unchanged, anim 2, state 6. Measured: states 2, 4, 6 at t=3, 34, 65 (62 frames from spawn to the run). [M: m-solo-l0]
2. **State 6, the gallop.** The direction `+22` (0 to 31) goes up by 1 **every 2 frames**; `vx = sign(facing) * |speed table[dir].x|`, so `|vx| = 1536 * |sin(dir * pi / 16)|` (measured 299, 588, 854, 1085, 1276, 1419, 1506, 1536, 1506 ...
   per 2 frames). At `dir & 0xF == 0` the dir counter stops and `+A9 = 20` counts down (about 22 frames standing still), then the next stride starts. One stride = 32 frames of motion + 22 frames still = **122 px in 54 frames**. A depth correction is
   set once per stride: if the hero's depth is more than -8 .. +23 away, `vz = +-32 / 64 / 128` (0.125, 0.25, 0.5 px per frame by |dz| under 8 / under 16 / more, table 0x53CC0), else 0.
3. **The horns hurt only when `dir & 0xF` is 6 to 10** (the middle of a stride, `+49` is cleared otherwise by 0x53890): about 10 frames per stride, at 1419 to 1536 per frame. Measured: horn box live at t=130 to 136 and 182 to 188 (Jack hit once per pass: -15 at level 0, -38 / -37 at level 31, knocked down). A thug in the way loses 15 HP per pass (raw).
4. **End of the pass.** Facing left: when `x < camX - 0x70` (facing right: `x > camX + 0x1F0`) Mack turns round (`+24 ^= 1`), state 8 with `+80 = 120`, `--+AA`; if `+AA` is now zero or below the object is freed (`+4 = 0x600`), else after 120 frames state 4 (target and charge again).
   Measured: pass 1 from t=65 (x 771) to t=298 (x 178, 233 frames), wait 121 frames, pass 2 from t=420 to t=682, freed. With one pass it is freed at the first turn.

### 3.4 Hit reaction, calm and leave

- Mode 2 starts every frame with the **leave test** (0x53A66): `+5C != 0` and `camX >= +5C` -> HP = 0, `+0 = 2`, mode 4, flash, sound 0xA09E: it calms and walks off even at full HP.
- A hit (`+5 = 2`, handler 0x53A80): if `+7 = 0`, HP > 0, poise > 0 and on the ground: flinch anim 5 for the hit stop frames (measured 9), HUD bar, poise -1 (jab); else the slide back: anim 3, `vx = +-1280` away from the hero with `ax = -32` (about 100 px, 40 frames), a dust puff every 6 frames (0x92EA4).
  Recovery (0x53B64): poise 10, `+79 = 60` (poise refill delay), `$6 = 0` (the wind up restarts: roar, 62 frames), HP 0 -> calm, `+A4 == 0` -> calm (the sleeper).
- **Calm** (mode 4, 0x538DE): `+A4 ^= 1`, 16 palette toggles (88 frames; `0x9E` / `0x9F`), then if HP > 0 back to mode 2 (the sleeper variant becomes a charger), else mode 6: facing left, `vx = -768` (3 px per frame), anim 7, freed off screen.
  Measured (HP set to 1): hit t=22, recoil vx 1248, calm flash t=63 to 151, walk off from t=151, freed t=331. [M: m-kill]
- Jack's jab did 11 to 12 damage and a 9 frame flinch (HP 128 to 116). [M: m-hit, m3-hit]

## 4. Rock Hopper, the velociraptor (pool A class 29, handler 0x4DCF6, 7,708 bytes)

Vice T.'s pet in Episode 1; scripted twice in Episode 5 (neutral, variant 0x200); the Episode 8 link card shows a green raptor in a tank and the Hopper starts hostile in Episode 8, so a tank probably releases it [I] (pool B 44 is not decoded here, open item 6).

### 4.1 Data

| Item | Value |
| --- | --- |
| Name | 4 R HOPPER (`+$84 = 0x10`), HP by level 44 to 150 (section 1.5), poise 10, `+72 = 0xC8`, guard `+87 = 4` (block, 0x4F3E8) |
| Tables | animations 0x4F4E4 (31), boxes 0x106000 (multi, `+4A = 1`), per animation frame step table `+64 = 0x4FA30`, leap table 0x4FAF0 `(512, 0, 1280, -64)`, attack choice 0x4FAF8 |
| Hurt boxes | two: body `(-4, 16, -22, 60, 0, 58)` and head `(-4, 16, -45, 24, 34, 44)`; the other animations change the boxes per frame (see the asset strips `re/out/assets/anims/dinosaur_a29_r_hopper/`) |
| Attack boxes | claw (anim 14, frame 1, 4 ticks) `(-4, 16, -70, 17, 54, 20)`; bite (anim 15, frame 1, 4 ticks) `(-4, 16, -71, 28, 46, 29)` |
| Attacks | 0x20 claw 12, 0x21 bite 16 (knock down), 0x29 pounce bite 15 (held hero, twice), 0x22 leap 10 (the hop animations have no attack box, so it never lands) |
| Palette | Episode 1: base 0, flash `0x91` / `0x92`, angry `0x92`; Episode 2 table: `0x91` / `0x98`; Episode 5: base `0x9E`, angry `0x9F`; Episode 8: starts `0x9F` and hostile (table 0x4F4DC) |
| Score, drop | none |

### 4.2 Entrances by the variant byte (`+$26` high byte, 0x4E01E)

| Variant | What | Used |
| --- | --- | --- |
| 0 | partner of Vice: runs the scripted intro, then OKORI (`+A4 = 1`) | Vice's init only (spawned at X 0x2D0, Z 0x148, `+A0` = parent) |
| 2 (and 1, 3 and others) | KIMAMA, neutral | Episode 5 records: variant 0x200 |
| 4 | MICHI, dormant (`+C0 = 0xFF`, roars at random, 1 in 16 per 60 frames) | unused |
| any, in Episode 8 | at once MODE 6 (OKORI), `+A4 = 1` | Episode 8 |

**Vice's raptor** (Episode 1, area 2): when class 18 variant 0 (Vice) initialises it calls 0x4DCC6, which spawns this object with variant 0, `+A0 = Vice`. The Hopper does not act on its own: its `$6` follows Vice's `$6`:
`$6 = 0` waits for Vice's `$6 = 2`, `2` for Vice 4, `4` plays anim 1, `6` waits for Vice 0xE, `8` plays the long growl animation 3 (466 ticks) and advances when Vice sets his frame flag `+2D`, `0xA` waits for another flag, then **COLOR** (flash, sound 0xA09E), `0xE` plays anim 5, then the object becomes a normal Hopper in OKORI.
Measured in the second Vice (h3-vice): `$6` 2 at t=22, 4 at 209, 6 at 338, 8 at 606, flash 0xC from 817, 0xE at 904, alive and hostile at t=929 (palette 0x92). The stale `+5C` of that slot is whatever the previous object left (0 on a fresh game), so the Episode 1 Hopper never leaves by camera; it leaves only by the timers of section 1.4. [M: h3-vice; R: 0x425C2, 0x4DEE4]
The Hopper does not check kill-all, so it stays after Vice is dead until its own MANZOKU. The exact Vice states belong to D3c.

### 4.3 The modes (+A8)

| Mode | Handler | Behaviour (measured unless marked) |
| --- | --- | --- |
| 0 MICHI | 0x4E0E0 | dormant sleeper (variant 4, unused) |
| 2 KIMAMA | 0x4E168 | neutral: wanders, 4/16 chance per decision to hop (anim 13, leap table), turns at the screen edges; a hero within |dx| <= 80 and |dz| <= 16 (0x4F204) switches to MATOWARI. After 1200 frames in this mode -> MANZOKU |
| 4 MATOWARI | 0x4E3B6 | neutral, follows and paces: measured step 82 px per 50 frames, turns, idles. Does **not** attack (a neutral Hopper with two Ferris on the screen did nothing to them: h3-neutral-thug). Counter `+AA`: from 600 frames each 10th frame 1/16 -> COLOR; at 1200 frames -> MANZOKU |
| 6 OKORI | 0x4E5C0 | hostile: states `$6` 0 init (anim by aggression: `+AD = 2` or 3 when `G.$5EC >= 8`), 2 idle, 4 approach by hops (measured 82 px per 51 frames, 2.7 px per frame on the first approach), 6 turn, 8 landing wait 16 frames, 0xA attack, 0xC recovery. Attack range test 0x4F2B4 (depth within 11, a point 64 px ahead within +-16); target: the hero in the window of 0x4F204, otherwise sometimes (7/16 per re-pick every 300 frames) the nearest enemy (0x4F260) [I: same pattern as Shivat]. Counter to 900 frames then each 10th frame 2/16 -> MANZOKU |
| 8 KYOURAN | 0x4EA1E | berserk: entered **once** after the Hopper has taken 5 hits while hostile (`+B9`, `+BA`, 0x4EF3A); up to three attacks in a row (`+B2`), then back to OKORI (0x4E59C) [R, not seen] |
| 0xA MANZOKU | 0x4EC06 | satisfied: `+A4 = 0`, anim 4 (roar), sound, then walks off (anim 6), freed by 0x173C. Measured: from the kill at t=22 the Hopper slides back (130 frames), then at t=152 mode 0xA and walks left about 2.5 px per frame, off screen at t=398 |
| 0xC COLOR | 0x4ECC6 | flash: `+A4 = 1`, 16 toggles of `+23` between the pair of the table 0x4F4DC (intervals 8, 8, 7, 7, 6, 6, 5, 5, 4, 4, 3, 3, 2, 2 frames, about 89 frames), then OKORI |
| 0xE DIFFENCE | 0x4ED16 | block: anim 0x14, `vx = +-0x400` (4 px per frame), `ax = -0x24`, 30 frames, back to the old mode |

Block (0x4F3E8): when the Hopper is **hostile, on the ground, and the hit comes from the side it faces**, a random draw `rand & 15` in {14, 15} (12.5 percent) blocks: `+B8 = 1`, the attack misses and the mode becomes DIFFENCE. Leave X: `0x4F16E` (`+5C <= camX`) -> MANZOKU in modes 2, 4, 6, 8.
Pounce (mode 6, `$7 = 8`, table 0x4FAF8 gives `$7` = 0 claw 3/16, 4 bite 7/16, 8 pounce 6/16): if the target hero is grounded and in normal control (0x4E9A6) the Hopper holds him (`+5 = 4`, hero state 0x02060000, `+C8 = 0x40`, `+2E = 0xE`), bites with 0x29 through 0x125C0 and throws him away (hero state 8 at offset 24 px, 64 up). [R: 0x4E982, 0x4EF44, 0x4F050]

### 4.4 Hit reaction

`+5 = 2` handler 0x4EDA8: jab 12 damage, anim 0x11 (17), HUD bar, flinch for the hit stop frames (9 for a jab); at the end `+23 = +BB` (restores the colour), HP 0 -> MANZOKU; `+A4 == 0` -> COLOR (so any hit enrages); `+B9` counts hostile hits for KYOURAN.
The heavy reaction (poise 0, knock down) slides him back (anim 0x12, 0x4FB08) and a burn reaction (`$6 = 4`) spawns flames and sets `+23 = 0x84`.
Measured (h3-hit): jab -12 (44 to 32), hit stop 9, COLOR from t=31 to t=120, OKORI from t=120, first bite on Jack at t=140 (-16). (h3-kill, hp 1): hit at t=22, slides 130 frames, MANZOKU at t=152. [M]

## 5. Shivat, the T. rex (pool A class 24, handler 0x4838C, 13,252 bytes)

Episodes 2 (asleep, X 0x678) and 5 (asleep, X 0x450). `re/specs/enemy/SPEC.md` section 11 and `re/specs/enemy/dino_shivat_dis.txt` describe the AI skeleton; this section adds what MAME showed.

### 5.1 Data

| Item | Value |
| --- | --- |
| Name | 7 SHIVAT (`+$84 = 0x1C`), HP by level 60 to 200 (section 1.5), poise 10, `+72 = 0xC8`, `+7D = 0xFF`, `+C0 = 0xFF` |
| Tables | animations 0x49574 (29), boxes 0x106000 (multi), step table `+64 = 0x49460`, per frame offset 0x493C4 / 0x493E2 |
| Palette | `+23 = 0x9E` in Episodes 2 and 5 (green), `+BD` = the colour to restore; angry `0x9F`; other episodes 0 |
| Hurt boxes | asleep (anim 0, 1): one flat box `(-2, 12, -100, 156, 0, 56)`; walking (anims 2 to 9): head `(-2, 12, -106, 82, 67, 65)` and body `(-2, 12, -41, 75, 1, 82)` |
| Score, drop | none; never calls the kill score |

### 5.2 Modes (+A8) and the measured flow

MICHI 0 (0x48530; `+0 = 2`, "Z" every 50 + (rand & 63) frames through 0x9667E, never ends by itself) -> a hunter's poke (section 1.4) -> hit reaction (anim 0xF, 7 frames) ->
`+A4 = 0` so COLOR 8 (0x48CFE, 16 toggles `0x9E` / `0x9F`, about 88 frames, then `+0 = 1`) -> OKORI 4 (0x4878C). MATOWARI 2 (0x485EA, neutral pacing) is entered by 0x485CA after the wake animation
when no hero is in sight (|dx| <= 0xA0 and |dz| <= 0x10, 0x49148); if a hero is in sight it goes straight to COLOR. MANZOKU 6 (0x48C40).
The wake entry points 0x48590 / 0x485A0 (anim 1) have no caller in the ROM; the poke (0x533FA) works through the normal hit reaction instead.
Measured wake (s4-wake, hunter at 673): the hunter walks from x 673 to 459 (t=4 to 214), swings at t=226 (two empty swings), third swing hits at t=274; Shivat flinches 7 frames (hit stop 8), COLOR t=281 to 369, OKORI from t=369 (palette 0x9F); the hunter is hit three times (-10 each at t=382, 402, 424) and runs off.

OKORI details (R: SPEC 11, 0x4878C): picks a target every 300 frames (a hero in sight, else the nearest enemy `0x491A4`, 7/16), steps by animation frame with the step table, turns at the screen edge, idles 120 frames now and then (2/16 per frame change, not when berserk `+BA`),
and **after 1800 frames each 10th frame 1/16 -> MANZOKU**; leaves at once at `camX >= +5C`. The first hit taken while angry sets `+BA = 1` (the compare at 0x48E54 only skips it from the fifth hit on): `+AD = 2` (second walk variant) and no more idle pauses. Walking speed measured 1.2 to 1.6 px per frame.

### 5.3 Attacks (OKORI, attack state `$6 = 6`, `$7` chooses)

| `$7` | Attack | Id, base damage | Frames measured | Boxes (z, x, y relative, facing left) |
| --- | --- | --- | --- | --- |
| 0 (0x48A90) | claw swipe (anim 11 or 17 by the walk foot) | 0x38, 14 | about 75 frames, then idle 120, turn 60 | hand box `(-4, 16, -137, 27, 51, 23)` on frames 1 and 4 (15 ticks each) |
| 4 (0x48B08) | tail swing and turn round (anim 12) | 0x39, 10, knock down | about 30 frames | three boxes sweeping front to back: `(-2, 12, -37, 120, 8, 104)`, `(-24, 32, -68, 78, 10, 36)`, `(-24, 32, -157, 107, 19, 27)`, 10 ticks each |
| 8 (0x48B64) | triple stomp (anim 13 then 14 three times, `+81 = 3`) | 0x3A, 10 each | stomps at +9, +30, +51 frames | `(-24, 32, -21, 125, 4, 46)`, `(-24, 32, -76, 81, 6, 37)`, `(-24, 32, -155, 105, 8, 33)`, 20 ticks each |
| 0xE (0x48BE0) | grab and hold (anim 0x1C, `+5 = 4`) | 0x3B, 18, the hero is thrown (state 8) | hold about 60 frames, damage at +62 | hero linked as in Zeke's grab; cooldown `+B4 = 180` after a grab |

Choice: in bite range 0x49520 gives `$7` = 0 or 0xE; the row index is `((word at a6 - 0x7A14) << 3) & 0x30` and that RAM word is **0** (measured, 0xFF5660 over 2000 frames), so the row is always the first: **half claw swipes, half grabs** at bite range. At long reach 0x49560 gives the stomp with 14/16.
Bite range (0x491F8): depth within 11, x within 17 of `x +- 0x6D`; long reach (0x49296): dz -11 .. 0, x within 23 of `x +- 0x84`.
The grab check (0x48F2A) needs the hero grounded and in normal control as for Zeke; the hold is not broken by pressing attack (s6-mash: six presses, no effect, -18 at +62).
Hits on the heroes were measured only for the grab (-18 at level 0); the claw, tail and stomps did not connect in my probes because the forced Shivat walked at another depth (depth 79 against the hero's 54): an open item (acceptance scenario A19 must place him at the hero's depth).

### 5.4 Taking hits

Awake Shivat (`+0 = 1`) at level 0: jab -13, -12, -12 (60 to 47 to 35 to 23), flinch 9 frames, poise 10 to 9, +100 points each, HUD bar. HP 0: finishes the reaction, then MANZOKU (measured: hp 1, hit at t=22, MANZOKU from t=63). [M: s8-a, s8-kill]
A burn reaction sets `+23 = 0x84` and flames (R: 0x48D8A). Fall and screen shake: the stomps set `G.$755C = 1` (R: SPEC 11) [I: shake].

## 6. The brontosaurus foot (pool A class 22 with helper class 23)

Episode 6 area 1 only: record trigger 0x520, X 0x6C0, Z 0x100, variant 0; trigger 0x7B0, X 0x950, Z 0x100, variant 1 (the foot never reads its variant; it hands 0 to its helper, so both records behave alike). Handler A22 0x47C42 (1,572 bytes), helper A23 0x48266 (294 bytes).

| Item | Value |
| --- | --- |
| Init | `+0 = 2` (**cannot be hit**), `+7D = 0xFF`, `+63 = 0xFF`, attack 0x63, anim table 0x718EE (5 one frame animations of 30 ticks: 0 foot flat on the floor, 1 foot raised, 2 high, 3 foot with hurt box, 4 raised variant), boxes 0x103C00; home `(+AA, +AC) = (x, z)`; spawns A23 at its place with `+A2 = parent`; HP fields 100 (not used) |
| Boxes | foot on the floor: hurt `(0, 16, -45, 67, 0, 98)`, attack `(0, 16, -42, 62, 0, 44)`; raised: attack `(-4, 16, -26, 66, 19, 39)` |
| Attacks | slam: **0x4C, 30 base damage, knock down** (measured -30 on Jack at level 0); the foot resting or rising near the floor: 0x63 (1 damage, push). The init writes no name id (`+3`, `+84`), so the level scale row comes from a stale `+84` [I]; at level 0 every row is 32, so my measurement does not tell |
| Draw | picture by height (0x48048): floor or state 2 or 0xE anim 0, height 0x30 or less anim 0, 0x31 to 0x60 anim 1 (state 2: anim 4), above anim 2 |
| Movement | home +-80 px in X and +-32 px in Z (clamp 0x47FF6) |

Cycle (measured, `$6` states, frames from spawn, bronto-0): 2 rises from the floor (ay +80, `vy` capped at 8 px per frame) until y 256 (t=3 to 43); 4 lowers 2 px per frame to y 144 (t=43 to 99); **6 hovers 180 frames** tracking the nearest hero inside its home column
(`|hero x - home x| <= 80`, moving 2 px per frame in X and 1.5 in Z, stopping when within 8; without a hero it wanders, new random speed every 60 frames from the tables 0x47EAC / 0x47EBC) (t=99 to 279); 8 waits 60 frames (to t=339); 0xA rises to 259 in 21 frames
(attack id becomes 0x4C); 0xC falls (24 frames), lands with sound 0xA288 and **screen shake 4** (`G.$755C = 4`); 0xE rests 30 frames on the floor, then state 2 again. The period is about 380 frames; the foot never leaves until the camera passes (0x173C).
Jack standing in the home column was hit (-30, knocked down) at t=380 and the foot did not follow him during states 8 to C. A23 (the leg) is a tall sprite that follows the foot (not decoded further [I]).
The picture shows the leg coming from the top of the screen. The brown and grey flashes of the background seen at t=240, 360 and 870 are probably the rain lightning of the area [I], not the foot. [M: bronto-0, bronto-1 same cycle]
Whether the foot hurts enemies: the registration is the hostile list, so by section 1.2 it does (raw 30), not measured.

## 7. Sky creatures, birds and small animals (all harmless except Zeke)

**Zeke is the only flying creature that does damage.** Every other bird or flyer I found has no entry in any hit list (static search of the calls to 0x4BEA, 0x4BFA, 0x4C0A in their handlers, and probes). The red bird that flew by at the Time Over scene in the first research (`mechanics-seen.md` D14, ambient) is class 50.

| Class | What | Behaviour (R: SPEC 17.1, D3e 3.8; M where marked) | Port |
| --- | --- | --- | --- |
| A49 (0x56A70) | background bird controller, 3 at the start of Episode 1 (var 0 twice, var 2 once), palette 0x9D | swings in X (var 0 limit +-0xA000 acceleration +-0x100, var 2 +-0x8000 +-0xC0), follows scroll 2 and moves against scroll 3, frees itself through 0x173C | DONE |
| A50 (0x56C80) | the red pterodactyls and other sky sprites of Episode 1, variants 0 to 0xC | positions relative to the camera; 5 animations (table 0x5708C); variants 0, 2 free themselves unless `G.$748 = 0x100`; no hit box | DONE |
| B73 (0x89ABA) | bird spawner | first delay 0x80, then 0x100 + (rand & 0x1FF); creates an A50 of variant from table 0x89B32 (0, 2, 4, 6, 8, 4, 6, 8) | DONE |
| A46 + A48 (0x5656C, 0x56924) | long tailed flock bird and its trail piece | spawned by pool B 72 (Episode 2 area 0: 5 birds at trigger 0x160, 7 at 0x280, 8 frames apart, camera X + 0xB0 + (0 to 0x3C), top of the screen + 0x40, palette 0x98); flies right at about 6 px per frame (M: c-46: x 425 to 711 in 48 frames) | MISSING (D3e S) |
| A78 (0x5F198) | small ground critter (the "tiny lizards") | Episode 2 area 0, records after the first Mack (triggers 0x188 and 0x198, Z 0x30 and 0x20); walks toward the nearest hero (about 0.75 px per frame) and stops 7 px from him (state 4) (M: c-78); harmless | MISSING (S) |
| A80 (0x5FF2E) | small ground critter | record at trigger 0x190, Z 0x50; stands still (state 2) (M: c-80); creates 2 helper objects (0x483C twice) [I]; harmless | MISSING (S) |

The scripts of Episodes 3, 4 and 6 contain no flying creature. The forest sound of Episode 2 (0x78) comes from pool B 80 (D3e).

## 8. Creatures that no script spawns (and what they really are)

I searched the whole ROM for every spawn of a pool A class (allocator 0x483C followed by a class word, and pool B / pool C allocators) and for the viewer code signature.

| Class | Art (asset catalog) | Handler | Who references it | Verdict |
| --- | --- | --- | --- | --- |
| A12 | golden velociraptor, big, 3 animations | 0x3DC5C (398 B) | the front end spawns it: three at 0x9C46C (variant 0) and two at 0x9C52A (variant 1) in the opening movie, and similar blocks at 0x99508 and 0x9B8C4 | **the raptors of the opening movie** (D4d 3.9: "pool A class 12 variant 0 spawns three objects (the raptors)"). Not gameplay |
| A28 | small orange raptors | 0x4DAB4 (530 B) | nothing spawns it; its code reads `G.$17` and `G.$1C` (input bytes) and steps the animation number and a volume (0x174F2); the same code block ends with the helper 0x4DCC6 that spawns the Hopper | **a viewer sprite of the test mode** [I: the test mode lists "viewer pages"], not gameplay |
| A37 | second pterodactyl, 7 animations | 0x53178 (580 B) | nothing; same viewer code | **viewer sprite**: the pterodactyl that was never in a stage |
| A39 | robed old man pointing | 0x53CC6 | nothing; same viewer code | viewer sprite |
| A44 | golden shells | 0x56190 | nothing; same viewer code | viewer sprite |
| A47 | small lizards, 3 animations | 0x567D0 | pool B 35 spawns it through 0x8426A (D3e: "object attached to a pool A object, boss fights [I]"); the handler also holds the viewer code | role unresolved; harmless |
| A51 | blue crested dinosaur, 2 colours | 0x5717E | class 5 (the Butcher's trigger, 0x38E4A) and its own helper 0x571C2 spawn it | **the dinosaur carcass in the Butcher's yard** (Episode 2 area 2) [I]; in my spawn test it is a crested body that lies still near the hero (c-51) |
| A22 / A23 | see section 6 | | script Episode 6 | gameplay |

Conclusion: **no code spawns an extra gameplay creature**; the only spawn outside the scripts is Vice's Hopper. The catalog's "unused golden raptor, raptor pack, lizards, bird, second pterodactyl, blue crested dinosaur" are movie sprites, viewer sprites and a carcass. Nothing in this list needs porting for gameplay (A12 belongs to the opening movie, M1).

## 9. Where every creature appears (from `stage_scripts.txt`)

Positions are absolute map X, depth Z, spawn trigger = camera X. `leave` = `+5C`. None is in a wave record.

| Episode and area | Creature | Trigger | X | Z | var | leave | Other |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 area 0 | A49 x3, A50 | 0, 0x40, 0x100, 0x260 | 0x80 to 0x3E0 | 0x180 | 0, 2, 0xA, 0xC | | sky birds; B73 from 0x100 |
| 1 area 2 | **Hopper** (via Vice) | at Vice's init (record trigger 0xF4, X 0x318, Z 0x128) | 0x2D0 | 0x148 | 0 | stale | intro synchronised with Vice |
| 2 area 0 | Mack | 0x180 | 0x320 | 0x30 | 0 | 0x480 | one pass |
| 2 area 0 | A78, A80, A78 | 0x188, 0x190, 0x198 | 0x328, 0x330, 0x338 | 0x30, 0x50, 0x20 | 0 | | critters |
| 2 area 0 | Mack | 0x280 | 0x420 | 0x30 | 0 (+5B = 2) | 0x480 | two passes |
| 2 area 0 | **Shivat** (asleep) | 0x480 | 0x678 | 0x61 | 0 | 0x569 | the 0x568 lock wave has 2 hunters (A79) |
| 2 area 0 | pool B 72 flocks | 0x160, 0x280 | | | 0, 0x200 | | A46 birds |
| 5 area 0 | R Hopper | 0x60 | 0x2C0 | 0x258 | 0x200 | 0x3E0 | neutral |
| 5 area 0 | **Zeke** | 0xD0 | 0x180 | 0x238 | 0 | 0x3D1 | |
| 5 area 0 | Mack | 0x100 | 0x2E0 | 0x220 | 0 | 0x270 | from the right |
| 5 area 0 | Mack | 0x140 | 0x100 | 0x240 | 0 | 0x320 | from the left |
| 5 area 0 | R Hopper | 0x1C0 | 0x360 | 0x218 | 0x200 | 0x3E0 | neutral |
| 5 area 0 | Shivat (asleep) | 0x240 | 0x450 | 0x251 | 0 | 0x400 | |
| 5 area 0 | hunter A79 | 0x3C8 | 0x3A0 | 0x228 | 0x400 | | wave 1 at 0x3D0 has 3 more |
| 6 area 1 | brontosaurus foot A22 | 0x520 | 0x6C0 | 0x100 | 0 | | |
| 6 area 1 | brontosaurus foot A22 | 0x7B0 | 0x950 | 0x100 | 1 | | |
| 7 area 0 | **Zeke** | 0x80 | 0x140 | 0x50 | 0 | 0x480 | |
| 8 area 2 | **Zeke** x2 | 0x200, 0x400 | 0x2C0, 0x4C0 | 0x140 | 0 | 0x72F | |
| 8 area 0 | tank mutants (pool B 44) | wave 0 | | | | | not decoded here, open item 6 |

Episode 5 puts R Hopper, Zeke, Mack, Shivat and the hunters on the same road at the start of area 0: while the camera moves from 0x60 to 0x3D0 the four creature kinds appear one after another; Mack and the Hoppers leave around 0x270 to 0x3E0, Zeke at 0x3D1, Shivat at 0x400.
Episode 7 has Zeke alone at the start; Episode 8 has the two Zekes over the corridor and cave.

## 10. Colours (palette bytes written to `+$23`)

| Creature | Neutral / base | Flash pair | Angry | Calm / leaving |
| --- | --- | --- | --- | --- |
| Shivat (Ep 2, Ep 5) | `0x9E` olive green | `0x9E` / `0x9F`, 16 toggles, 88 frames | `0x9F` orange red | MANZOKU keeps the colour of the last toggle |
| Hopper Ep 1 (Vice) | 0 (OBJ set) | `0x91` / `0x92` | `0x92` | |
| Hopper Ep 5 | `0x9E` green | `0x9E` / `0x9F` | `0x9F` | |
| Hopper Ep 8 | starts `0x9F`, already angry | | | |
| Mack | 0 (OBJ set, orange brown) | `0x9E` / `0x9F` (calm flash, not angry flash) | none (always hostile, no colour change) | after the flash it walks off |
| Zeke | 0 (orange); Episode 8 `0x9D` | none | none | flying away or dying `0x9E` (Episode 7 `0x9F`) |
| Burn (Hopper, Shivat) | | | | `0x84` while burning |

So the web statement "turns red when angry and green when drained" matches the Hopper and Shivat (green to `0x9F`) and is wrong for Mack and Zeke, which never change colour when angry. Checked against MAME pictures: Hopper green then orange (h3-hit sheet), Shivat olive green asleep (s3-sleep sheet). The exact RGB of the palettes is in the asset dump (`re/out/assets/palettes/`).

## 11. What each creature does to whom (one table)

| Creature | Hurts heroes | Hurts enemies | Hurt by heroes | Hurt by enemies | At 0 HP |
| --- | --- | --- | --- | --- | --- |
| Zeke | peck 10, swoop 10, grab 18 (scaled); flinch | yes, raw 10 / 18 through the pass 0x11E56 (not measured [I]) | yes, x2, 1 hit kills | no | flies away |
| Mack | 15, knock down | yes, raw 15 (measured on Ferris) | yes, HP 128 | no | calm, walks off |
| Hopper | claw 12, bite 16 knock down, pounce 15 (only when hostile) | yes, when hostile [R], neutral does nothing (measured) | yes, 12.5 percent block when angry | no | calm, walks off |
| Shivat | claw 14, tail 10 knock down, stomps 10 x3, grab 18 and throw | yes, raw (measured: Ferris -10 per tail hit) | yes when awake | the hunter's poke (no damage) | calm, walks off |
| Foot | slam 30 knock down, touch 1 | yes [R, not measured] | no (`+0 = 2`) | no | never |

Heroes who are held (Zeke, Hopper, Shivat) cannot be hit by others and are safe until the release. 2 or 3 players change nothing for the creatures except the target choice (Zeke's `+$116` marker, the census preference of `0x32032`) and the rank level (damage and HP).

## 12. Acceptance scenarios (probe segments and what must match)

Run them with the probe tool (`re/harness/probe/probe.sh`, headless; recipes in `plan/game/design/capture-recipes.md`). Every segment starts with
`jump EP AREA`, then the lines shown; `t = 0` is the first frame of control. "Hold killall" is `hold 0xFF850E 1 2` (kills script enemies; **do not use it in a segment with a hunter**, it kills the hunter too).
The full line sets are in `re/out/d3b/gen_b1.py` to `gen_b4.py` and `b5.txt` to `b8.txt` (git-ignored; regenerate from the lines below). The port compares its own trace with these values (positions are integer pixels, `y` is the height above the floor, frame numbers are game frames after the jump).

| # | Segment (lines after `jump`) | Must match |
| --- | --- | --- |
| A1 | `jump 5 0`; hold killall; `level 31 free`; `hp 100`; `2 spawn A 82 var=0 w5c=0x7fff x=+0x100`; run 3600 | Zeke mode changes (frame, new mode): (2, 0) (185, 2) (335, 0) (517, 4) (773, 0) (955, 4) (1243, 0) (1425, 4) (1519, 0) (1701, 2) (1863, 0) (2045, 6) (2178, 0) (2361, 6) (2577, 0) ...; Jack HP 100 to 80 (t=260), 60 (602), 40 (663), 20 (744), 4 (1050), 0 (1131); starts at y 320 and descends 0.75 px per frame |
| A2 | same with `level 0 free` | the first attack only at t=3262 (mode 4), Jack 100 to 90, 80, 70 at t=3374, 3456, 3536 |
| A3 | A1 setup at level 0, then `900 objset A:82 +0xA0.w 2` and `+0x6.w 0` (swoop); same with 4 (peck) and 6 (dive) | swoop: launch t=972 (vx -256, vy 768), sub 6 at 1019, low point y 46 at 1031, pull up, mode 0 at 1101, no hit. Peck: launch 979, hits at 1003, 1064, 1232 (-10 each, hit stop 16), launch heights 95, 169. Dive: sub 4 at 979 (y 123), at 1010 y 82 grab, hold to 1042, carry vx 896 toward +X (Zeke x 64), Jack -18 at 1051, mode 0 at 1053 |
| A4 | A3 dive plus `1000-1001:J` | at t=1010 Zeke goes to mode 0 with no grab, Jack keeps 100 HP |
| A5 | `jump 5 0 0x700`, dive forced at 900 | carry direction left (`vx -896`, face 0, Zeke x 1827), Jack -18 at t=1106; the RAM words 0xFF5038 and 0xFF61B8 read 0 |
| A6 | `2 spawn A 82 var=0 w5c=0x7fff x=+0x100`, `8 objset A:82 x 140`, `8 objset A:82 y H`, `8 objset A:82 +0x7A.b 255`, Jack `30-32:A` (punch) or `30-31:J`, `50-52:A` (jump kick) | punch kills at H = 0, 24, 48, 72, not at 96, 120; jump kick kills at 0 to 96, not at 120; HP 20 to 0 at once; Jack +100 points |
| A7 | as A6 with `8 objset A:82 hp 1`, H = 40 | hit t=32, recoil to t=64 (vx 768, vy 480), dying from t=65, off screen about t=180, no extra score |
| A8 | `2 spawn A 82 ...` in `jump 5 0 0x100` with `w5c=0x80` | mode 8 from t=4, `vy` 256 + 16 per frame, freed at t=69 (y 514) |
| A9 | `jump 2 0`; hold killall; `level 0 free`; `2 spawn A 38 var=0 b5b=2 w5c=0x7fff x=+0x100` | spawn x 771 (camera 291), states 2, 4, 6 at t=3, 34, 65; vx per 2 frames 299, 588, 854, 1085, 1276, 1419, 1506, 1536, 1506 ...; stride stop at x 649 (t=96 to 117) and 527 (t=148 to 169); horn box live t=130 to 136; Jack -15 (t=131) and -15 (t=440); turn at t=298 (x 178), second pass from t=420, freed at t=682. Level 31: -38, -37 |
| A10 | A9 with `6 objset A:38 x 540`, `6 objset A:38 hp 1`, `6 objset A:38 +0xA0.w 0`, Jack `20-22:A` | hit t=22 (HP 0, +100 points), slide vx 1248, mode 4 from t=63, mode 6 (vx -768) from t=151, freed t=331 |
| A11 | A9 without killall, `3 spawn A 2 var=0 x=+0x40 z=0x30`, `3 spawn A 2 var=2 x=+0x80 z=0x30`, `god on` | each Ferris 44 to 29 at t=129 and t=181 |
| A12 | `jump 5 0`; hold killall; `god on`; `2 spawn A 29 var=0x200 w5c=0x7fff x=+0x100`; run 2300 | Hopper mode 2 at t=4, 4 at t=104, 0xC at t=887 (`+A4 = 1`), 6 at t=976 (palette 0x9F), 0xA at t=2042 |
| A13 | A12 with `6 objset A:29 x 100`, `6 objset A:29 +0x7A.b 60`, no god, Jack `20-22:A` | HP 44 to 32 at t=22 (hit stop 9, poise 9), COLOR t=31 to 120 (palette 158 / 159 toggling at t=50, 58, 66, 73, 80, 86, 92, 97, 102, 106, 110, 113, 116, 118, 120), OKORI at t=120, Jack -16 at t=140 |
| A14 | A12 forced hostile (`+0xA8.w 6`, `+0x6.w 0`, `+0xAA.w 0`, `+0xA4.b 1`, `+0xC0.b 1`) at levels 0 and 31 | the first bite (attack 33 = 0x21) at t=658: Jack -16 and -33 |
| A15 | `jump 1 2`; `level 0 free`; `god on`; `2 spawn A 18 var=0 x=+0x100`; `20 hold 0xFF8744 2 0x1D0`; run 2600 | a Hopper appears at x 720, z 319 in the intro (`+5 = 4`), `$6` 2 at t=22, 4 at 209, 6 at 338, 8 at 606, flash (palette 145 / 146 = 0x91 / 0x92) from 817, 0xE at 904, `+5 = 6` at 928, alive in mode 6 at 929 (second pair later: same offsets +2) |
| A16 | `jump 2 0`; `level 0 free`; `2 spawn A 24 var=0 w5c=0x7fff x=+0x80`; run 900 | Shivat `+0 = 2`, `+C0 = 255`, palette 158, pool C class 30 objects (variant 0x200, at x 316, y 92) appear at t=5, 81, 258 and so on; Jack's punches do nothing |
| A17 | A16 plus `3 spawn A 79 var=0x400 x=+0x180 z=0x30`, `god on`, no killall | hunter `+C1 = 1` for Shivat at t=4; hunter at x 459 by t=152; swings t=226 to 273; Shivat hit reaction at t=274 (`+68` = the hunter, hit stop 7, HP 60 unchanged), COLOR t=281 to 369, OKORI at 369 (palette 159); hunter HP 100 to 90 (382), 80 (402), 70 (424) |
| A18 | A16 plus `6 objset A:24 x 515`, mode 4 forced with `+0x0.b 1`, `+0xA4.b 1`, `+0xC0.b 1`, `+0x6.w 0`, `+0xAA.w 0`, `+0xAC.b 0`, `+0x7A.b 250`, `god on`, Jack `20-21:A`, `50-51:A`, `80-81:A` | HP 60 to 47 (t=22), 35 (t=51), 23 (t=232 with the 3rd hit); hit stop 9 each; score +100 each; hp 1 variant: HP 0 at t=22, MANZOKU (mode 6) from t=63 |
| A19 | forced attack `8 objset A:24 +0x6.b 6`, `8 objset A:24 +0x7.b N` with N = 8 | stomps (atk 0x3A) at +9, +30, +51 frames (box mode 4 each 20 frames), then idle; N = 14: grab, Jack -18 at t=70 (hold ~60 frames); the hold is not broken by `20-21:A ... 55-56:A` |
| A20 | `jump 6 1`; hold killall; `level 0 free`; `hp 100`; `2 spawn A 22 var=0 x=+0x80` | foot `$6`: 2 at t=3, 4 at 43, 6 at 99, 8 at 279, 0xA at 339, 0xC at 360, 0xE at 384, 2 at 414 (the second cycle repeats at +411), y 30, 256, 144, 144, 144, 259, 30; Jack -30 at t=380 (knocked down); screen shake 4 at the landing |
| A21 | `jump 2 0`; `level 0 free`; `god on`; `2 spawn A 24 var=0 w5c=0x7fff x=+0x80`; `4 spawn A 2 var=0 x=+0xC0`, `4 spawn A 2 var=2 x=+0xE0`, `4 spawn A 2 var=4 x=+0x100`; Shivat forced into OKORI as in A18 (without the hero's punches) | the Ferris at X 547 and 613 lose 10 HP per tail hit: 44, 34, 24, 14 at t=166, 188, 208 |

Cross check against the plan: the heroes cannot hit a sleeping Shivat; Zeke takes one hit; Mack needs about 11 jabs; a Hopper needs 4 jabs at level 0; none of them gives any score for the kill.

## 13. Port status and the port task list

State today: classes 22, 23, 24, 29, 38, 82 are **MISSING** (`enemy_class_unported` frees them at once and prints "not ported"); the monster hit passes are MISSING (`combat.rs` only clears the list 0x6B64 at 0x87); the helper `enemy_hit_held_player` (0x125C0, ported for class 10) is reusable for Zeke, Hopper and Shivat; the sky scenery A49, A50 and pool B 73 are DONE (`enemy/scenery.rs`).
What the shared code already offers: the free list and census, `+7D`, the hit reaction table 0x2A606 for ordinary enemies (not used by monsters: they have their own `+5 = 2` handlers), the floor and `0x173C` helpers, HUD message `enemy_hud_hit`.

| Task | Content | Size | Needs | Milestone |
| --- | --- | --- | --- | --- |
| P1 Monster services | list 0x6B64 (draw and hit lists), passes 0x11A9E / 0x11B36 (with the guard table 0x11CE4), 0x1132A, 0x11CEC / 0x11D32, 0x11E56 / 0x11EDC (raw damage, credit 0xFF, `+7D < 0` minimum 1 HP), the hit reaction mapping 0x11CC4, the colour flash routine (table 0x4F4C8 / 0x4F4DC / 0x5396A), the poke 0x533FA, the "Z" effect spawner 0x9667E (pool C class 30, D3f), debug probe of `+C0` / `+C1` | M, 2 h | player hit passes, effects pool C | M5 (before Vice's raptor) |
| P2 Zeke (class 82) | init and target rule, modes 0 / 2 / 4 / 6 / 8, the tables 0x6137A, 0x6141A, 0x6142C, 0x6143C, 0x614DC, 0x61504 to 0x61526, hit handler, dying, grab link and release (0x125C0), the three boxes, palettes, sounds 0xA0CC; the quirky direction read must be coded as "compare with 0xC0" (the RAM word is always 0) | L, 2.5 h | P1, `+$116` marker in the player | M7 (Episode 5), M8 (Episodes 7 and 8) |
| P3 Mack (class 38) | init and side choice, charge cycle, gallop (speed table 0x22 by `dir`), depth nudge, horn gating by `dir & 0xF`, hit and slide, calm, leave | M, 1.5 h | P1 | M6 (Episode 2), M7 |
| P4 Rock Hopper (class 29) | init and entrances by variant, modes 0 / 2 / 4 / 6 / 8 / 0xA / 0xC / 0xE, hop movement (step table 0x4FA30, offsets 0x4F3DA), attacks 0x20, 0x21, 0x29 and the pounce hold, block, KYOURAN, **Vice partner intro** (follows the parent's `$6`), flash | L, 3.5 h | P1, class 18 for the parent | M5 (the intro with Vice, T5.2), M7 (Episode 5) |
| P5 Shivat (class 24) | modes MICHI / MATOWARI / OKORI / MANZOKU / COLOR, step per animation frame, claw, tail, stomps, grab and throw (0x48E62), hit reaction, Z marks, leave, plus the hunter integration (`+C0`, `+C1`, poke) with D3a's A79 | L, 4 h | P1, A79 from D3a, effects | M6 (Episode 2, the user's first dinosaur), M7 |
| P6 Brontosaurus foot (22 and 23) | the cycle of section 6, home column, nearest hero tracking, slam with `G.$755C = 4`, draw by height, helper A23 | M, 1.5 h | P1 (hostile list), screen shake hook | M8 (Episode 6) |
| P7 Critters A78 and A80 | walk to the hero, stand | S, 0.5 h | | with D3e's small classes |

Total about 15 agent hours. NEW-PLAN M5 T5.5 asks whether the velociraptor of the boss fight is needed: **yes**, Vice's init spawns it, so P4 (at least its intro states) belongs with Vice (T5.2). Order inside M6: P1, P3 (Mack is the simplest and exercises P1), P5 with the hunter; M7: P2 and P4; M8: P6. The Vice raptor needs P1 and P4 only for its intro states (the hostile modes can come later; until then Vice's Hopper can be a harmless placeholder that follows the partner script).
Every ported routine must name its original address (rule of the plan). Data through `dino-extract`: animation tables 0x611D2, 0x53BE4, 0x4F4E4, 0x49574, 0x718EE, box tables 0x103000, 0x103C00, 0x106000 (multi), the small tables named in the sections.

## 14. Open items

1. **Other wake sources.** The hunter's poke (0x533FA) is the only way I found to wake a sleeping dinosaur. A thrown weapon, a bullet or a falling rock may reach it through the list passes in some other way: my probe with a poacher did not wake Shivat. Test: a poacher with a free line of fire through a sleeping Shivat for 3000 frames, and a thrown knife.
2. **Fire.** The burn branches of Hopper and Shivat are in the code (0x4EE1A, 0x48D8A) but I did not run a torch or flame against them. Test: `spawn B 45` flame vents or a torch in hand next to an awake Hopper.
3. **Zeke, the foot and the Hopper against enemies** were not measured (Mack and Shivat were). The code path (0x11E56) is the same.
4. **Shivat's claw, tail and stomps against a hero** were not measured at the right depth (section 5.3). The numbers follow the table; the acceptance scenario A19 must be repeated with Shivat at the hero's depth.
5. **KYOURAN** (Hopper berserk after 5 hits) and **DIFFENCE** (block, 12.5 percent) were read in the code but not observed. A probe would hit an angry Hopper 6 times with `god on`.
6. **Episode 8 tank creatures** (pool B 44 creates "a mutant", the Hopper's Episode 8 start mode) are not decoded here; D3d or D3e decide whether the Hopper is among them.
7. **Stale fields.** Vice's Hopper keeps the old `+5C` of its slot. On a fresh game it is 0. The port should use 0 and keep the `+5C` of the record for scripted ones.
8. **Stage clear and area change.** I did not check what happens to a dinosaur that is still alive at an area change fade or at the stage end (probably freed with pool A).
9. **Hero grab escape.** Six attack presses did not free a held hero. D2b may know a different escape (stick mash).
10. **Listing level only:** the leg helper A23, A47 (and why the viewer code sits in it), A80's two helper objects, the pool C "Z" effect, and the effect of the foot's variant (none found).
11. **Sounds**: the hit and death voices of the creatures are D4e's.
12. **The 3 player test.** Zeke's target marker and the Hopper's hold with 2 or 3 players were not run.

## 15. New rows for `mechanics-seen.md` (group C, IDs use the prefix DS so they do not clash)

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| DS01 | Monsters (A22, A24, A29, A38, A82) live in the hostile list 0x6B64; they hurt heroes (scaled by type and level) and enemies (raw damage, nobody gets score); no monster hurts a monster | R 0x11A9E, 0x11CEC, 0x11E56; M m-vs-thug, s3-vs-enemy | MISSING | D3b 1.2 |
| DS02 | Dinosaurs give no kill score, no drop, are not counted by waves (`+7D = 0xFF`), are not touched by kill-all; hits score 100 / 200 / 400 | R, M z-kill, m-kill, h3-kill, s8-kill | MISSING | corrects D04 ("walks away, gives no score") only in detail |
| DS03 | A sleeping Shivat has `+0 = 2` and cannot be hit by anyone; a dinosaur hunter (A79) wakes it with its third swing (helper 0x533FA, no HP loss) | R 0x533FA, 0x5F9F8; M s4-wake, s5-wake | MISSING | corrects D02 (poachers' hits do not wake it; hunters do) and D13 (bullets: not seen) |
| DS04 | Hostile model names: MICHI, KIMAMA, MATOWARI, OKORI, KYOURAN, MANZOKU, COLOR, DIFFENCE; COLOR flash 16 toggles (about 88 frames) then angry; any hit on a neutral dinosaur starts it | R 0x490CE, 0x4F1C4; M h3-hit, s4-wake | MISSING | D03, D04, D12 |
| DS05 | Neutral Hopper: after 600 frames in MATOWARI a 1 in 16 roll every 10 frames turns it angry, after 1200 it leaves; angry: 900 frames then 2 in 16 per 10 frames to leave. Shivat MATOWARI 900 frames, OKORI 1800 frames, same rolls | R, M h3-neutral | MISSING | |
| DS06 | Zeke: HP 20, damage x2 (type factor 64); one punch or jump kick kills; patrol heights 74 to 203 (overshoot because the altitude test runs once per flap cycle); standing punches reach him only when y <= about 72, jump kicks y <= about 100 | R 0x2BECE; M z-hit-*, z-solo-* | MISSING | answers D09 "hit only by jumping" |
| DS07 | Zeke attacks: swoop 0x5B, triple peck 0x5C, dive grab 0x3B (hold 31 frames, 18 damage); attack roll every 180 frames with 9, 16, 24, 30 in 32 by level band; type table by altitude index | R 0x606D4, M z-force-*, z-solo-* | MISSING | numbers in D3b 2 |
| DS08 | Zeke's grab only works on a grounded hero in normal control; a jump at the right moment dodges it; the carry goes toward world X 0xC0 (original bug: reads RAM 0) | R 0x60C8C, 0x6108A; M z-force-c-jump, z-grab-far | MISSING | |
| DS09 | Zeke leaves straight up when `camX >= +5C` (checked only in patrol), dies by flying away, never explodes; two Zekes pick different heroes (`+$116`) | R, M z-leave, z-kill | MISSING | |
| DS10 | Mack: charger with strides (32 frames moving, 22 still, 122 px), horns hurt only in the middle of a stride, side chosen by the record X against camX + 0xC0, `+5B` = number of passes, 62 frame wind up, calm walk off | R 0x5341C; M m-solo-l0 | MISSING | corrects D07 |
| DS11 | Rock Hopper is spawned only by Vice's init (variant 0, partner intro synchronised with Vice's `$6`); the scripts place it with variant 0x200 in Episode 5; angry Hopper blocks 12.5 percent of frontal hits; berserk after 5 hits | R 0x425C2, 0x4DCC6, 0x4F3E8; M h3-vice | MISSING | corrects D08 |
| DS12 | Brontosaurus is one huge foot (A22) with a leg helper (A23): hover 180 frames over the nearest hero in its 80 px home column, 60 frame wait, wind up, slam 30 damage with knock down and screen shake 4, rest 30 frames, repeat (about 380 frames); cannot be hit | R 0x47C42; M bronto-0 | MISSING | corrects D11 |
| DS13 | Unused creature art: A12 = opening movie raptors, A28 / A37 / A39 / A44 / A47 = test mode viewer sprites, A51 = carcass spawned by class 5, A46 / A48 = flock bird and trail | R spawn search, D4d | n/a | answers N2 of the asset catalog |
| DS14 | Hopper and Shivat burn when hit by a burn reaction (record reaction 0xA or 0xC): flames and palette 0x84 | R 0x4EE1A, 0x48D8A | MISSING | untested, open item 2 |

## Appendix: files used and made (all git-ignored under `re/out/`)

- Listings read: `re/out/d3b/zeke_dis.txt`, `mack_dis.txt`, `hopper_dis.txt`, `shivat_dis.txt`, `bronto_dis.txt`, `c79_dis.txt` (made with `re/tools/d2`), helper scripts `tab.py` (word and byte tables of the ROM image), `anim.py` (animation records and boxes), `an.py`, `sm.py`, `hits2.py` (probe log readers).
- Probe batches: `re/out/d3b/s1.txt`, `b1.txt` (Zeke and Mack), `b2.txt` (Hopper, Shivat, foot, small classes, Zeke extras), `b3.txt` (clean repeats), `b4.txt` (wake and Shivat attacks), `b5.txt` to `b8.txt` (details), logs in `re/out/probe/d3b-*`.
- Notes of facts in the order found: `re/out/d3b/notes.txt`.
- Pictures: `re/out/assets/anims/dinosaur_a24_shivat`, `dinosaur_a29_r_hopper`, `dinosaur_a38_mack`, `dinosaur_a82_zeke`, `actor_a22`.
- Rules kept: no sub-agents, no code edits, no other design file edited, only `plan/game/design/D3b-dinosaurs.md` written in `plan/`; MAME only through `re/harness/probe/probe.sh` (SDL dummy drivers, no GUI).
