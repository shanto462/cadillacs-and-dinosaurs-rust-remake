# D2a: player moves (the four heroes, every move and action)

Written 2026-10-12 by the D2a research agent. Part of the design phase (`plan/game/NEW-PLAN.md` section 1, D2). Documents only: no code was
changed. Everything here is meant to be built exactly by later port agents.

How to read this file

- **Tags.** `R:` = ROM address or repo file (68000 addresses; `G.$xxxx` = RAM `0xFF8000 + xxxx`; `+$xx` = field of the player object).
  `M:` = measured in MAME with the probe tool (the batch and segment names are given; logs are in `re/out/probe/<batch>/<segment>/log.txt`,
  git-ignored). `W:` = web (the links are written next to the facts; list at the end of section 19). MAME wins over the web, the ROM wins over the old repo specs.
- **Hero ids** (the ids the code uses, `+$03`): **0 Jack, 1 Mustapha, 2 Hannah, 3 Mess**. Every table in this file is in this id order.
  Warning for readers of `re/specs/player/tables.txt`: its row labels for ids 1 and 2 are swapped (row "Hannah" is Mustapha, row "Mustapha" is Hannah).
  I checked this against MAME for walk speed, run speed, jump height, wall jump and dive speed (all match the id order used here).
- **Units.** Speeds are 8.8 fixed point (divide by 256 for pixels per frame). One tick of an animation is one game frame. A frame number
  `t` in a probe log is the frame after the jump helper; an input pressed at `t` shows in the log at `t + 1`.
- **Frame class** (`+$2E`) values: 0 neutral, 2 walk (grab probe), 4 attack active, 6 special (immune), 8 hurt, 0xA falling, 0xC grabbing, 0xE grabbed.
  Frames of class 6, 8, 0xA, and frames with body box 0, cannot be hit by enemy attacks (R: SPEC 5.1, `re/specs/anim/SPEC.md` 4.4).
- Source specs read: `re/specs/player/SPEC.md`, `HANDOFF.md`, `re/specs/anim/SPEC.md`, `plan/phase-01-stage1-complete.md` (tasks 3 and 3b),
  `plan/game/mechanics-seen.md` rows P01 to P42, the asset dump (`re/out/assets/anims/hero_*`), the port code
  (`dino-rs/crates/dino-game/src/game/player.rs`, `player_combat.rs`, `player_weapon.rs`).

## 0. Summary of what is new in this document

1. **Tag team exists and is fully decoded** (section 9): two heroes stand still on top of each other for 90 frames, then one (the initiator) swings the other
   (the partner). Four versions, one per initiator hero. The port has none of it (MISSING).
2. **Friendly fire is active** in normal 2 and 3 player play, but only while fewer than 3 enemies are alive (section 10). `G.$4F3` is the
   **enemy** count of the last frame, not a player count. The port hard codes 3 (off). One line fix.
3. **1UP pose** (section 11): every hero has a real victory pose (Jack and Mess flex, Mustapha somersaults and gives a thumbs up, Hannah cheers).
   It plays when a hero crosses an extra life score and then returns to idle within 240 frames. MISSING in the port.
4. **Idle**: no idle animation beyond a slow breathing loop. No taunts exist in the player code. At stage clear the heroes just stand (section 11).
5. **Invulnerable frames** that were not in the old list: no hurt box during pick up, special, throw, dash recovery and dive landing (Mustapha and Hannah: no box; Mess: a zero size box at the feet), drop in, 1UP pose and the tag team pose (section 15).
6. **Dive kick window**: the dive (attack at the top of a jump) only starts in the 2 or 3 frames around the top of the jump. Jack's and Mess's top is
   exactly the split height (section 4).
7. **Air attacks hit 1.5 times harder** than their record says against almost all enemy types (dive 0x0E and running jump attack 0x10, table 0xA32FA).
8. **Wall jump** works only on terrain with wall bit 4 (or 0x18 / 0x1C). One place found and tested: Episode 1 area 2 (section 4.4).
9. The attack power factor `$C6` is 32 (neutral) in 27 of 32 rolls at HP 75 or more, and it is re-rolled each time the hero is hit (section 14).

## 1. The four heroes

| id | name | OBJ palette | hit box table | dump folder | notes |
| --- | --- | --- | --- | --- | --- |
| 0 | Jack Tenrec | 0 | 0x100000 | `re/out/assets/anims/hero_jack/` | slide dash, ground special, no wall jump |
| 1 | Mustapha Cairo | 2 | 0x100C00 | `.../hero_mustapha/` | fastest walker, air special, wall jump, alternate finisher, reverses a hold |
| 2 | Hannah Dundee | 1 | 0x101800 | `.../hero_hannah/` | highest jump, wall jump, cannot jump with a heavy object, keeps guns when hit, fast knife stab |
| 3 | Mess O'Bradovich | 3 | 0x102400 | `.../hero_mess/` | slowest, hits hardest, two throw directions, keeps the bazooka when hit |

R: SPEC section 1 and 7, `re/specs/anim/SPEC.md` 6. Names and traits: W: AQM characters page (https://www.arcadequartermaster.com/cad_characters.html),
W: HG101 ("Jack and Mustapha serve as the averages of the group", "Hannah being fast and Mess being slower, but good at grappling").

**Animation tables** (set 0 moves, 1 attacks, 2 hurt, 3 grab and throw, 4 being held, 5 weapon poses; pointer tables 0x163E, 0x164E, 0x165E, 0x166E, 0x167E,
0x168E indexed by hero id). The dump files are `re/out/assets/anims/hero_<name>/<TABLE>_<id as 2 digits>.gif` (real timing) and `_strip.png`
(frames with hit boxes). Table names per hero:

| set | Jack | Mustapha | Hannah | Mess |
| --- | --- | --- | --- | --- |
| 0 moves | 09E41C | 09F414 | 0A06AC | 0A16F0 |
| 1 attacks | 09EDFE | 09FECA | 0A115A | 0A1FFC |
| 2 hurt | 09F094 | 0A0328 | 0A13D8 | 0A232E |
| 3 grab | 09F252 | 0A04FE | 0A1596 | 0A24E0 |
| 4 held | 09F30E | 0A0596 | 0A1622 | 0A25DA |
| 5 weapon | 09EACC | 09FB94 | 0A0E8C | 0A1D32 |

Example: Jack's special is set 1 id 8: `re/out/assets/anims/hero_jack/09EDFE_08.gif`. In this file a move is written `1:8` (set:id).

**Per hero numbers** (R: tables 0x50E4.., 0x53E2.., 0x1A734, 0x1A994, 0x2C88E; M where marked)

| value | Jack | Mustapha | Hannah | Mess |
| --- | --- | --- | --- | --- |
| walk speed right, up (8.8) | 414, 236 | 435, 248 | 422, 241 | 409, 234 |
| walk speed in px per frame (R, U) | 1.62, 0.92 | 1.70, 0.97 | 1.65, 0.94 | 1.60, 0.91 |
| run speed after the start burst (R, U) | 844, 482 | 870, 497 | 896, 512 | 819, 468 |
| run start burst speed (R, U) | 1868, 1067 | 1894, 1082 | 1920, 1097 | 1843, 1053 |
| burst length in frames | 7 | 5 | 5 | 5 |
| super run speed of a tag team partner (R) | 2124 | 2150 | 2176 | 2099 |
| neutral jump launch vy / gravity | 1741 / -74 | 1740 / -74 | 2112 / -88 | 1672 / -72 |
| neutral jump peak (px), air frames, landing frames (M) | 76, 47, 6 | 76, 47, 9 | 94, 48, 9 | 72, 46, 9 |
| forward jump: vx, launch vy, peak, distance (M) | 608, 1616, 65, 100 | 608, 1741, 76, 107 | 528, 1952, 80, 89 | 608, 1552, 62, 98 |
| jump attack split height (dive needs this or more) | 76 | 75 | 93 | 72 |
| dive launch vx / vy (gravity -32, ax +-32) | 1280 / -512 | 1664 / -448 | 1280 / -512 | 1152 / -448 |
| dive distance and air frames (M) | 144 px, 23 | 187 px, 24 | 168 px, 26 | 134 px, 23 |
| damage taken factor (x/32), table 0x2C88E indexed by id | 32 | 34 | 35 | 30 |
| invulnerable frames after get up / throw / pick up / special | 20 / 12 / 10 / 60 | 20 / 10 / 8 / 60 | 20 / 14 / 20 / 60 | 20 / 18 / 10 / 60 |
| idle body box (width x height, px) | 23 x 75 | 29 x 77 | 23 x 69 | 36 x 74 |
| wall jump | no | yes (vx 1536, vy 896) | yes (vx 1152, vy 1280) | no |

The jump launch vectors for the other directions are in `re/specs/player/tables.txt` (remember the swapped labels) and in `SPEC.md` 3.6. The run speed
values were checked in MAME (M: d2a_basic walk_*, run_*): Mustapha walks 435 and runs 870, Hannah 422 and 896.
The select screen stars (0xB444: Jack 4/3/3, 3/5/3, 2/4/5, 5/2/4 for power, speed, skill) are rows in **cursor order** (Jack, Hannah, Mustapha, Mess),
not id order: speed 5 belongs to Mustapha, who is the fastest walker [I]. They are display only. The damage taken factor is read by hero id (SPEC 5.1:
`rd16(0x2C88E + chr*2)`), so Mustapha 34 and Hannah 35 (the old SPEC text lists them with the wrong names); D2b settles the rest.

## 2. Controls and timing windows (all heroes)

Buttons: attack (bit 0x10, button 1), jump (0x20, button 2); stick bits R 1, L 2, D 4, U 8. There is no input queue: a press is a one frame edge
(`+$A8`). Only these buffers exist (R: SPEC 2, 4.4, 4.6, 4.10):

| input rule | number | source |
| --- | --- | --- |
| run (double tap) | second press of the same direction (within 45 degrees) at most **12 frames after the first press**, so up to 11 idle frames between two 1 frame taps. 13 or more: walk | M: d2a_dtap3 `dt04..dt13_*` (all heroes: 11 idle frames runs, 12 walks); R: `$BC = 15`, run when `$BC >= 3` |
| run length | 240 frames, then idle. Releasing the stick does not stop it. A turn of more than 45 degrees, attack, jump or a fall ends it | M: d2a_basic `run_*` (run from t=14 to t=254); R: `$FA` |
| run cooldown | 5 frames after a run (a walk start in that time is a plain walk) | R: `$FC` |
| chain continue (idle or walk) | attack within **40 frames** (`$AE = 0x28`) of a connected hit animation ending; the first 4 frames lock walking | R: SPEC 4.4 |
| chain buffer | an attack press during hit stop or the rest of the hit animation is kept (`$AD`) | R |
| whiff | a missed jab ends the chain and locks movement for 5 frames (`$B3`) | R |
| special | attack held and jump pressed, or jump held and attack pressed (same frame or one after the other, HP must be more than 1). Also buffered 10 frames while hurt (`$C9`) | R: 0x63AA, 0x63E6 |
| command move | tap down (alone), then up (alone) within 10 frames, then attack while holding up within 10 frames. The check runs only in **walk** state and while grabbing (the taps themselves make the hero walk) | R: 0x6AC8; M: d2a_fight2 `cmd` |
| grab | walk into an enemy (walk frame probe box), re-grab cooldown 10 frames after a release | R: 0x108AE, `$F0` |
| jump attack | attack **held** (not pressed) while the jump is higher than 16 px | R: 0x1A704; M: d2a_basic `jattack_*` |
| dive | the same, started in the 2 or 3 frames at the top (height at least the split height, section 1) | M: d2a_basic `jattack_*` |
| wall jump | jump pressed again in the air while touching a wall (bit 4, 0x18 or 0x1C) or within the same jump after touching one, height at least 32 px, horizontal speed not 0 | R: 0x1A744; M: d2a_wj |
| combo throw | the 4th attack press with only up or only down held | R: 0x195B2 |
| throw from a grab | attack with left or right held (for Mess the direction picks the throw); attack with no direction or only up and down = knee | R: 0x1D676 |
| release a grab | jump, 120 frames of pushing away, or the enemy escaping | R: 0x1D676 |
| pick up | attack when standing over an item (not in the air) | R: 0x5AA2 |

Idle with no input never changes state (section 11).

## 3. Movement: idle, walk, run, jump, land, fall

### 3.1 Idle (state 0x02000000, anim 0:0)

State `0x19122` entry, `0x1916E` handler. Velocities are cleared, the hero stands in the idle animation and nothing else happens.
Idle loops (measured with 2000 frames of no input, god mode, M: d2a_idle idle_jack, idle_mustapha, idle_hannah, idle_mess; state stayed 0x02000000 the whole time):

| hero | animation 0:0 | measured |
| --- | --- | --- |
| Jack | 14 frames, 800 ticks: 3 holds of 120, 8 breathing frames of 10, 3 holds of 120; then the loop repeats from the 4th frame | first pass 357 frames, loop of 440 frames |
| Mustapha | 9 frames, 230 ticks, whole animation loops (2 x 60, 2 x 10, 4 x 20, 10) | loop of 230 |
| Hannah | 7 frames, 230 ticks (60, 60, 40, 10, 10, 10, 40) | loop of 230 |
| Mess | 6 frames, 270 ticks first pass (60, 60, 120, 10, 10, 10); loop from the 3rd frame = 150 ticks | loop of 150 |

There is no idle pose, no waiting animation and no taunt in the player code. The only idle related things on screen are the stage "GO" arrow
and its hint after 600 frames (seen at t=500 in the idle runs; D4c owns it). The HUD prompts of absent players change from INSERT COIN to JOIN-IN
over time (seen at t=2000; D4d).

### 3.2 Walk (state 0x02000002, anim 0:1)

12 frames of 4 ticks (48 tick cycle, all heroes), frame class 2 with a grab probe box. Speeds in section 1 (8 directions: diagonals use the
diagonal table entries, e.g. Jack UR (230, 196)). Measured (M: d2a_basic walk_*): 8 px per 5 frames for Jack going right (1.6 px per frame).
The hero faces the last horizontal direction held (right wins when both are held). Impossible stick combinations (left and right) still walk
with the U entry [R: SPEC 3.4].
The walk probe box (grab) in front of the hero: Jack x 7..17, z 3..5, y 45..84; Mustapha x 7..17, z 3..4, y 44..84; Hannah x 5..16, z 1..7, y 43..76;
Mess x 7..20, z 1..7, y 35..78 (px from the hero; R: dump). So a grab starts when an enemy body is 5 to 20 px in front, at head and chest height,
within a few pixels of depth.

### 3.3 Run (state 0x02000200, anim 0:2)

Entry 0x19FC6, handler 0x1A012. The run start frame is a burst: speed class 8 (see section 1) for 7 frames (Jack) or 5 frames (others), then class 4.
Measured (M: d2a_basic run_*): Jack moves 47 px in the burst then 3.3 px per frame; Mustapha 3.4, Hannah 3.5, Mess 3.2.
While running: attack = dash attack (section 7), jump = running jump (section 3.4). With a held weapon of kind 0x201 or 0x202 attack is the run swing (D2c).
The run keeps its direction when the stick is released.

### 3.4 Jump (states 0x0400 neutral or flip, 0x0408 running jump), landing 0x0404, fall 0x0406

Entry 0x1A5D0. Face the stick, pick a vector from the 9 direction table (section 1; `SPEC.md` 3.6). Animation: 0:3 for neutral and diagonal jumps,
0:4 (forward flip) for a pure direction held longer than 3 frames, 0:5 (turn flip, the hero turns around) when the direction was just reversed,
0:7 / 0:0xC later in a neutral jump when a side is pressed (cosmetic). Air control adds (2, 0) per frame for left or right, (0, 1) for up or down.
Landing: sound 0x9C0C, animation 0:6 (7 to 10 ticks), any input (stick or button) cancels it at once.
Measured (M: d2a_basic jump_*): neutral jump takes 47 air frames then 6 (Jack) or 9 (others) landing frames; peak 76, 76, 94, 72 px; forward jump goes
100, 107, 89, 98 px and up or down jumps move 22, 23, 31, 22 px in depth.
The fall (walking off a ledge, 0x1A848): anim 0:4, `vy` limited to 0, gravity -32, horizontal speed halved, landing sound, back to idle at once (no landing anim).

Running jump (0x1A906, vector class 4 with a horizontal acceleration): vx 640 plus 24 per frame (Mess 40), vy by hero: Jack 1536, Mustapha 1280, Hannah 1792,
Mess 976 (gravity -74, -64, -80, -48). M: starting from a run it flies 181, 175, 200, 225 px with a peak of 59, 47, 74, 36 px in 41, 40, 44, 40 frames,
landing 6, 9, 9, 9 frames. No air control during a running jump. Wall jump check is active.

## 4. Jump attacks, running jump attacks, wall jump

### 4.1 Jump attack low (state 0x040C, anim 1:0xB, attack 0x0C) and dive (state 0x040A, anim 1:0xA, attack 0x0E)

Attack held in a standing or wall jump at height 16 or more starts the low attack (removes depth movement: `vz = az = 0`; keeps vx and vy; faces the sign of vx). At the
split height (section 1) or more it is the dive: the hero faces the stick, `vx = +-dive vx` with `ax = +-32`, `vy` = dive vy, gravity -32.
M: d2a_basic jattack_*: attack held from +3 to +16 frames after the jump gave low attacks at heights 29 to 72; at +20 (the top) the dive started for all four heroes;
at +26 on the way down it is a low attack again. Because Jack's top (76) and Mess's top (72) equal their split height, only the frames around the top
(2 or 3 frames) start a dive. Hannah (top 94, split 93) and Mustapha (top 76, split 75) have the same narrow window.
Low attack ends on landing (landing state 0x404, 6 to 9 frames). Dive ends on landing with the **dive landing** state 0x0436 (anim 0:0x1E, 22 ticks, not cancelable):
M: 17 to 21 frames before control returns (Jack 21, Mustapha 17, Hannah 21, Mess 21). Mustapha's and Hannah's dive landing animation has **no hurt box** (invulnerable,
section 15); Mess's body box is a zero size point at his feet (only blows that reach floor level touch it); Jack's is a normal box.
Hit boxes (R: dump): Jack low attack reaches 5..52 px in front, y 23..83 (two boxes); dive reaches 2..69, y 12..45. Hannah: low 4..67, y 28..47; dive -1..53, y 0..33.
Records: low 0x0C, dive 0x0E in section 14. Both stay active the whole air time, one hit per enemy per swing (attack serial `$5E`).
The dive gets the 1.5 times air bonus against almost every enemy type (section 14).

### 4.2 Running jump attacks (attack pressed during a running jump)

All four use anim 1:0xD and attack 0x10 first. Per hero (R: 0x1ACE8, states and handlers):

| hero | state | handler | extra hits | measured (M: d2a_fight2 rjump) |
| --- | --- | --- | --- | --- |
| Jack | 0x416 | 0x1AD32 | attack 0x70 on each change of the frame flag | 0x10 hit 30, then 0x70 hit 14; air 49 frames |
| Mustapha | 0x428 | 0x1B4C8 | 0x70 on the first frame flag change, 0x72 on the later ones (the animation is cut to 1 tick after two hit stops) | 0x10 hit 30, 0x70 hit 13, 0x72 hit 13 |
| Hannah | 0x42A | 0x1B592 | none | 0x10 hit 29; air 58 frames |
| Mess | 0x42C | 0x1B600 | 0x70 | 0x10 hit 42, 0x70 hit 17 |

(Damage samples include the hero's `$C6` of 42 in those runs, see section 14.) Landing is the normal landing state 0x404.
Hit box example: Jack 8..64 px in front, y 27..60 and 35..86 (three boxes over the animation).

### 4.3 Dash jump

A running jump is the "dash jump" of the web page (W: AQM: "Dash jump, then ATTACK" gives Flying Kick, Plunger Kick, Flying Upper per hero).

### 4.4 Wall jump and the triangle jump (Mustapha and Hannah only)

R: check 0x1A744 (called from the standing jump, the running jump and the wall jump itself), state 0x0402, handler 0x1A7D8, anim 0:0x12 (7 frames; Mustapha 116 ticks, Hannah 52).
Conditions: hero id 1 or 2; wall bits `+$50` (or the latch `+$AF` set earlier in the same jump) equal 4, 0x18 or 0x1C; jump **newly pressed**; `vx != 0`; height
at least 32 px. Then: face away from the current horizontal direction (moving left: face right, `dir = 8`; moving right: face left), vector class 8: Mustapha `vx +-1536, vy 896` (gravity -74), Hannah `vx +-1152, vy 1280` (gravity -88). The hero keeps the vector, may start a jump attack
(attack held, same height rules: this is the "triangle jump" kick, with the low or high attack records), air control stays on.
M: d2a_wj: Hannah and Mustapha in Episode 1 area 2 at x=215, z=318, `LJ` then a second jump press 18 or 22 frames after the first: wall jump
(state 0x0402) started at x 159..169, height 74..80, flew to x 342 and 368. A second press at 6 to 14 frames was too early (no contact yet or too low) and nothing happened. Jack
with the same inputs never wall jumps. Mustapha's launch measured vx 1536 vy 896, Hannah's 1152 and 1280 (M: d2a_wj wj_mustapha_18, wj_hannah_22).
Where walls exist (M: d2a_wallscan, a zigzag walk in 11 areas logging `+$50`): bit 4 was seen in Episode 1 area 0 (x 84, z 51), area 1 (x 894, z 32), area 2
(slanted wall at x about 174 along z 318, which the hero slides along), Episode 5 area 0 (x 1416, z 598), Episode 6 area 0 (x 860, z 21), Episode 8 area 1
(x 1431, z 826). Bit 8 alone (map edges, x 113 in Episode 1 area 2) does **not** allow a wall jump. Which of these are real "wall jump spots" a player would use is
open (section 19).

## 5. Ground combat: chain, combo throw, grab, knees, throws

### 5.1 Punch chain (attack x4)

States 0x04 jab, 0x06 hit confirm, 0x08 punch 2, 0x0A punch 3, 0x0C finisher; animations set 1: jab 1:0 (re-press inside the `$2C` window: 1:1), jab hit pose 1:2,
punch 3 1:3 (hit pose 1:5), finisher 1:4 (hit pose 1:6; Mustapha uses the alternate 1:7 when the frame flag is 0). Attack ids 0x00, 0x00, 0x02, 0x04
(records in section 14). Each step needs the previous one to **connect**. Sounds: `sfx 0x95CE` at jab start (voices at the next steps).
Measured with presses every 12 frames (M: d2a_fight2 `chain`, hit frames after the first press, hit stop frames):

| hero | jab | punch 2 | punch 3 | finisher | chain over (idle again) |
| --- | --- | --- | --- | --- | --- |
| Jack | hit +2 (hs 9) | +13 (9) | +26 (9) | +40 (13) | +55 |
| Mustapha | +2 (9) | +13 (9) | +28 (9) | +38 (13) | +57 |
| Hannah | +2 (8) | +13 (8) | +28 (8) | +41 (12) | +59 |
| Mess | +3 (9) | +14 (9) | +28 (9) | +44 (13) | +67 |

Hit stop (frames, both sides frozen) is the record value: jab 9/9/8/9 (id order), punch 3 9/7/8/9, finisher 13/13/12/13. The jab box reaches 9..68 px in front for Jack, 13..73
Mustapha, 12..69 Hannah, 23..71 Mess (with probe frames in front of it that start combo throws). The finisher knocks down (reaction 6 for Jack, Mustapha and Hannah, 4 for Mess)
and reaches 105 px (Jack), 89 (Mustapha), 97 (Hannah). Each hit scores points (section 14).

### 5.2 Combo throw (attack, attack, attack, then up or down + attack)

Entry 0x195B2. The enemy must have been probed (a frame of class 0 with an attack box overlapping a standing enemy during the chain, R: 0x108FA). With only up held at the 4th press
the hero turns around and throws **backwards**; with only down held it throws **forwards** (the facing logic of SPEC 4.4). The enemy gets state 0x02060000 and the
throw state 0x0600 runs (section 5.4) with attack id 0x0A. M: d2a_fight2 `uthrow`, `dthrow`: Jack throw state 0x02040600 lasted 22 frames, hit the thug for 28
(base 22) and the thug flew in reaction 8 (thrown). The probe sets `$70` only on a frame of class 0 that has an attack box, and only for an enemy that is back in its normal state (`0x0200`, so after the victim's hit stun
of the third hit); the hit confirm entry clears `$70` on every connected hit. The probe frames differ per hero (R: dump): Jack has them in the jab, the punch 3 and the finisher wind up; Mustapha only in the
hit poses 1:5 and 1:6 (11 and 6 ticks); Hannah only in the first 3 ticks of the punch 3 hit pose 1:5; Mess in many frames. So the window for the 4th press is short and hero specific.
M: on a 12 frame rhythm (fourth press 8 frames after the third hit) the throw happened for Jack and Mustapha (throw state at +37) and the normal finisher came for Hannah and Mess.

### 5.3 Grab, knees (hold anim 3:0, knee 1 3:1, knee 2 3:2)

Start (0x108AE): the hero's current frame has class 2 and a probe box that overlaps the body box of an enemy that is on the ground, not invulnerable, in class 0 or 2,
and `G.$4DD == 0` and the re-grab cooldown is 0. Hero state 0x02040000 (anim 3:0, class 0xC), enemy 0x02060000 (class 0xE, follows the hero's hand with per frame offsets).
While holding (0x1D676): the hold timer is the enemy's `+$72` (0x3C) and counts down; the hero cannot be pushed away if the stick is empty or toward the enemy; pushing
away for 120 frames releases.
Attack with no left or right held starts a knee: knee 1 (state 0x02040200, anim 3:1, attack 0x06), later knees (state 0x02040400, anim 3:2, attack 0x08). A knee
is 11 to 13 ticks (damage on the frame with flag set). Each knee costs the enemy HP and stun; when its stun or HP runs out the enemy leaves the hold and the knees stop.
M (d2a_fight2 `grab_knees`, standing thug at HP 100): knee 1 14 (Jack), 13 (Mustapha), 13 (Hannah), 20 (Mess); knee 2 20, 20, 20, 27; knee 3 19, 19, 19, 25, and the thug was released
by the third knee in all four (stun meter empty). The grab itself starts on the first walk frame in contact (state 0x02040000 at +2 in these runs) and a knee can be started at once.

### 5.4 Throws (state 0x0600, anim 3:3; Mess 3:3 toward or 3:4 away)

Attack with left or right held in a grab: the hero faces that direction (Mess: the direction relative to the facing selects the animation), attack id 0x0A. The damage is dealt
on the animation frame with `$2C != 0` (Jack tick 13, Mess forward tick 31, back tick 33) and the enemy is placed with the throw offset table 0x2CB32
(table index = hero id: Jack (-72, +43), Mustapha (-68, +53), Hannah (-64, +53), Mess (-56, +8) forward and (+48, +16) back; dx is for a hero facing left, mirrored when facing right) and flies in reaction 8. The throw animation has **no hurt box**
(23 ticks, Mess 41 and 47), and the hero is invulnerable for 12/10/14/18 frames after it. M: d2a_fight2 `grab_fwd`, `grab_back`: throw state 22 frames (Mess 40
forward, 46 back); damage on a full HP thug 29, 28, 32, 41 (id order, includes C6 42).
Mess's two directions (W: AQM): Power Slam (forward) and German Suplex (backward).

**Throwing an enemy into others.** The flying body is on the body list 0x6B30. When it overlaps another enemy (0x1185A) that enemy takes **5 damage** from the fixed
enemy attack row 0x11 (stun meter set to 0, reaction 4 knock down, hit stop 2, sound 6) and the thrown enemy keeps flying. R: 0x118D2.
(Enemy side details: D3.)

### 5.5 Escape when grabbed by an enemy (record 0x18)

Attack id 0x18 (15 / 15 / 20 / 20 damage, stun 11, reaction 6, hit stop 8, score 800) is the move a hero makes when he mashes free. Mustapha instead reverses the
hold and throws the enemy (state 0x02040600). D2b describes the mash numbers.

## 6. Special attack (mega crush: attack + jump)

Entry 0x1968E: sound 0x992C, `$EE = 20` (HP cost budget), action 0x11, any held item is discarded. Hero state 0x0E (ground, all heroes except Mustapha) with attack id 0x12,
anim 1:8 (frame class 6 all through, **no hurt box**). Mustapha uses the **air special** 0x1AAE6: states 0x040E (rise, `vy = 1216`, gravity -64; frames with the flag set only
animate) then 0x0410 (a 15 frame hover, then fall), landing state 0x404, anim 1:8 (41 frames, 118 ticks).
Web names: Jack "Spiral Uppercut", Mustapha "Slash Kick", Hannah "Spiral Smash", Mess "Air Wrecker" (W: AQM).

**HP cost** (R: 0x10B08, applied when a class 6 frame hits an enemy): 10 HP for the first enemy hit, 4 for the second, 4 for the third, nothing after that (budget 20, 10 + 4 + 4 = 18
maximum); the cost can never kill (HP stays 1 or more); no cost when nothing is hit; no cost when `G.$4DD != 0`. M: d2a_fight2 `special3`, three thugs: HP 100 to 90 after the
first hit, then 82 after the other two (Jack: 90 at +5, 82 at +16; Mustapha 82 at +5 with all three in one frame; Hannah 90 at +1, 82 at +8; Mess 82 at +6).
Needs HP above 1 to start. HG101 calls it "crowd clearing but health draining".

Immunity: all frames are class 6 with no body box; at the end `$00 = 2` and `$F9 = 60` (60 more frames of invulnerability) and the hero returns to idle.
Mustapha also gets the 60 frames when the air special lands.

Measured lengths (M: d2a_basic special_*, no enemy): Jack 38 frames, Hannah 35, Mess 53; Mustapha 23 rising + 33 hover and fall + 9 landing = 65. With hit stop a hit extends it
(Jack 52, Hannah 45, Mess 58, Mustapha 26 + 33 + 9 with three hits).
Reach (R: dump, first to last box): Jack x -14 to 46 px, y 15..77, z up to 44 deep; Mustapha x -44 to 77, z -36..44, y 0..82; Hannah x -35 to 54, z 4..44; the shape is a circle around the hero.
Record 0x12: damage 20 (Mess 23), stun 11 for Jack and Hannah and 1 for Mustapha and Mess, reaction 4 (knock down), hit stop 7 / 3 / 5 / 5 (id order).
The hit spark faces the enemy away from the hero (`e.facing = (p.x > e.x)`).
Port: special entry, HP cost, discards the weapon: DONE for Jack; Mustapha's air special DONE in code (`player_air_special*`), not recorded in MAME runs.

## 7. Dash attack (run, then attack)

Entry 0x19720 (sound 0x9984, action 0x13). After a run, attack:

| hero | move | state | anim | vector (vx, ax, vy, gravity) | ends |
| --- | --- | --- | --- | --- | --- |
| Jack | slide (W: "Sliding Attack") | 0x10 then recovery 0x12 | 1:0xC then 0:0xD | 1280, -32 (slows down), no jump | animation end, then recovery 0:0xD |
| Mustapha | flying lunge (W: "Napalm Kick") | 0x412 then 0x12 | 1:0xC then 0:0xD | 1536, -24, 816, -48 | landing, then recovery |
| Hannah | flying knee (W: "Flying Knee") | 0x412 then 0x12 | 1:0xC then 0:0xD | 1536, -24, 960, -56 | landing, then recovery |
| Mess | rushing lariat (W: "Rushing Lariat") | 0x412 then 0x12 | 1:0xC then 0:0xD | 1024, 0, 640, -40 | landing, then recovery |

Hits: attack id 0x14 for the first part; every time the frame flag (`$2C`) changes the attack serial is renewed with id **0x5A** (first change) then **0x5C** (later
changes), so a dash attack can hit the same enemy again and the last hits are the strong ones (section 14: 0x5C is 32 / 32 / 34 / 32 base damage, reaction knock down).
Measured without an enemy (M: d2a_basic dash_*): Jack slide 26 frames over 83 px then 15 frames recovery; Mustapha 34 frames over 142 px + 15; Hannah 34 frames over 142 px + 12;
Mess 32 frames over 124 px + 15. With four thugs in a row (M: d2a_fight2 `dash`): Jack's near thugs took 27 to 28 (0x5A) then 42 to 45 (0x5C), the far thugs only the 0x5C hit; the others hit 15 to 16 (0x14), 27 (0x5A), 43 (0x5C).
Jack's slide has a very low body box (top at 35 px against 75 standing: enemy blows at head height pass over it), which is the "resistant to many attacks" of the web page.
Mustapha's and Hannah's **dash recovery has no hurt box** (13 to 16 ticks); Mess's is a zero size point box at his feet (nearly safe); Jack's is a normal box.

## 8. Command moves (down, up + attack)

R: entry 0x1B6EC (state 0x0430, anim 1:0xE, sound 0x9A36, action 0x15), per hero code through table 0x1B72E. "Sure-kill" moves of the web page; HG101: "rising attacks
... which can be performed without draining health" (no HP cost). The hero is hittable during them except the first few ticks (section 15). Measured with four thugs in a row (M: d2a_fight2 `cmd`, `d2a_basic command_*` without enemy):

| hero | move (W name) | sequence | hits (attack id, base dmg) | movement (M) | length (M, no enemy) |
| --- | --- | --- | --- | --- | --- |
| Jack | flip kick ("Crescent Tumble") | 3 tick wind up (1:0xE), then jump with vx 1024, ax -19, vy 1760, gravity -88 and anim 1:0xF | 0x72 (32), 0x74 (16), 0x76 (4) by frame flag | forward 99 px, peak 65 | 62 frames (state 0x430 includes the landing anim 0:0x17) |
| Mustapha | somersault ("Somersault Slash") | a front kick first (attack 0x74), then at the frame flag a jump straight up and back (the hero rises 48 px at once, vx -256, vy 1280, gravity -88) with 0x76, then 0x78 and 0x7A on later flags | 0x74 (26), 0x76 (14), 0x78 (8), 0x7A (4) | backward 36 px, peak 81 | 40 frames + 15 frames landing (state 0x28) |
| Hannah | rolling elbow ("Rolling Elbow") | state 0x430 anim 1:0xE, then at the flag a low launch (vx 768, vy 896, gravity -80, attack 0x70); attack 0x72 at the first falling frame; first landing bounces (vy halved) with 0x74; second landing starts the dash state 0x24 with attack 0x76 (vx 2048, ax -192, anim 1:0xF) | 0x70 (12), 0x72 (6), 0x74 (4), 0x76 (30) | forward 134 px, peak 17 (low roll) | 35 frames + 13 frames (state 0x24) |
| Mess | flying lariat ("Flying Lariat") | launch vx 512, vy 2048, gravity -88 with 0x72, then 0x74, 0x76, 0x78 on later frame flags | 0x72 (32), 0x74 (10), 0x76 (6), 0x78 (4) | forward 92 px, peak 89 | 46 frames + 24 frames landing (state 0x26, screen shake `G.$755C = 3`, sound 0x9B9E) |

Measured damage on thugs at full HP (C6 42 in those runs): Jack 44, 21, 4 (0x72, 0x74, 0x76); Mustapha 37, 18, 8 (0x74, 0x76, 0x78); Hannah 15, 7, 5, 42 (0x70, 0x72, 0x74 and the big 0x76 dash hit, 42 on two thugs in the same frame);
Mess 44, 13, 6, 4 (0x72 to 0x78).
The per hero records are in section 14 (ids 0x70 to 0x7A; the ids 0x70 and 0x72 of Mustapha and 0x70 of Jack and Mess are used by the running jump attacks, section 4.2).
The score of the hits uses the record's score code (section 14). Each part starts a new attack serial so every hit can land once.

## 9. Tag team ("two player moves"): Let's Go, Roll, Jump, Fly

The only two player move in the game. The trigger, the five states, the follow code and the table were decoded from the ROM (R: 0x6728..0x6922) and recorded in MAME with two players
(M: d2a_tie `tie_jack_idle`, `tie_mustapha_idle`, `tie_hannah_idle`, `tie_mess_idle`, plus `_atk` variants). W: AQM names: Jack "Let's Go!" (sends the ally running with increased
speed), Mustapha "Let's Roll!" (rolling high into the air), Hannah "Jump!" (jumping high), Mess "Let's Fly!" (tumbling high); "Overlap ally for 2 secs".

### 9.1 Trigger (0x6728, runs once per frame for each player in the player update, 0x6700)

Player A is the **initiator** for partner B when all of this holds on a frame: A is alive and not already in a tie up (`$C1 = 0`), A's byte `+$00` has bit 0 set (so **not in god mode**,
since god mode sets +$00 to 2), A holds no heavy object (0x20A) and is not input locked; A holds **no input at all** (any input resets the counter); B is alive, in normal ground mode
(`+$05 = 0`, not hurt or grabbing), holds no heavy object, frame class 0 (standing, not walking), action `+$C8 = 0` (not in any move), no input held; the X distance is less than 32 px
(`A.x - B.x` from -32 to +31) and the depth difference `A.z - B.z` is 0 to 15 (A is level with or slightly higher on screen than B). A counter per partner slot (`+$C2`, `+$C3`) counts down
from **90** each frame the rule holds and resets to 90 when it fails. At 0 the tie up starts. With equal Z the lower player number wins (P1 first).
M: both players still for 90 frames: the tie up started exactly 90 frames after the pokes.

### 9.2 Start (0x686E)

Initiator A: `+$00 = 2` (hit tests off), `+$C1 = 1`, state **0x0200001A** (ground state 0x1A, handler 0x19A7C), `+$C8 = 0x12`, frame class 0xC; facing = 1 (right) if A is less than 192 px
from the left edge of the camera, else 0 (left) [R: 0x6896..0x68AA; M: x 1000 with camera 897 faced right]; partner link `+$70 = B`. Partner B: placed on A (x, y, z - 1), same facing, `+$C1 = 1`, `+$C8 = 0x12`, class **0xE** (held), `+$00 = 2` and `+$EC = 180`
(180 frames of blinking invulnerability from this moment), link `+$70 = A`, and its state set from the byte pair table 0x6902 by the initiator's hero id:

| initiator | partner state | handler | web name |
| --- | --- | --- | --- |
| Jack | run group, step 4 = 0x02000204 | 0x1A330 | Let's Go |
| Mustapha | air, 0x0424 | 0x1B226 | Let's Roll |
| Hannah | air, 0x0422 | 0x1B0F6 | Jump |
| Mess | air, 0x0426 | 0x1B356 | Let's Fly |

(This also explains the old HANDOFF notes "state 0x1A, run state 4 0x1A330, +$C8 0x12", and the air states 0x22 to 0x26.)

### 9.3 Initiator (state 0x1A)

Anim 0:0xF (3 frames: Jack 3 x 15 ticks, Mustapha 3 x 20, Hannah 3 x 20, Mess 6 x 10 with release on the 4th frame): the swing pose, **no hurt box**. At the end of the animation (Jack 45 ticks, others 60) the initiator
gets `+$EC = 180` (180 frames blinking invulnerability), `+$00 = 2`, and goes to idle. The frame byte `$2C = 1` frame is the **release moment**: Jack after 30 ticks, Mustapha 40, Hannah 40, Mess 30.
`+$2D` (phase 0..2, Mess 0..3) selects where the partner hangs.

### 9.4 Partner follow (0x6922, called every frame of the partner's pre launch step)

Partner position = initiator position + (dx, dy) from the table below, with dx mirrored by the facing (listed for a hero facing left; for facing right use x = ix - dx), `z = iz - 1`, partner anim
restarted at frame `+$2D` of the initiator (so both stay in step). Table 0x69D4 and following (R), measured positions (M: d2a_tie):

| initiator | phase 0 (dx, dy) | phase 1 | phase 2 | phase 3 |
| --- | --- | --- | --- | --- |
| Jack | (32, 0) behind | (-16, 0) in front | (-48, 0) | - |
| Mustapha | (32, 0) | (0, 32) overhead | (-64, 32) | - |
| Hannah | (-32, 0) in front | (-32, 40) | (-32, 80) lifted overhead | - |
| Mess | (-37, 10) | (16, 64) | (-6, 64) | (-80, 48) |

If the initiator is gone (`+$00 = 0`), is no longer in normal mode (`+$05 != 0`) or its action `+$C8` is not 0x12, 0x6922 returns carry and the partner is released into idle.

### 9.5 Launch and flight

**Jack's Let's Go** (run state 4, 0x1A330): partner anim 0:0x17 (run) in step with the initiator, then at the release frame: `+$C8 = 0x12`, speed class 12 (the partner's own super run
speed, section 1, 8.3 to 8.5 px per frame), `+$81 = 0x28`. For the first 20 frames the partner runs straight ignoring the stick, then it follows the stick, and the run ends 3 frames after
`+$81` reaches 0 with no direction held. M: Hannah as partner ran 8.5 px per frame (vx 2176) for 43 frames (t=122 to 165) from x 1065 to the screen clamp 1352. While running: attack **with a direction held**
= attack id 0x1A (15 / 15 / 20 / 20 base damage, stun 11, knock down, hit stop 8, score 1000), anim 1:9 (a flat dive, boxes 15..65 px in front); attack with **no** direction held does the idle entry and the
normal jab (M: d2a_tie `tie_jack_atk`: partner state 0x04 jab, attack 0); jump = running jump (0x1A906); the partner is invulnerable (180 frame blink) the whole time.

**Air versions** (Mustapha, Hannah, Mess initiators; states 0x424, 0x422, 0x426): partner anim 0:0x1A (roll, Mustapha's), 0:0x1B (Hannah's, same pictures as the roll) or 0:0x1C (Mess's tumble; the partner's
facing flips on the frame with the flag). At the release frame: `vx = +-768` (3 px per frame in the facing direction), `vy = 1664`, gravity -69 (0xFFBB), `+$C8 = 0x12`, `+$51 = 1` (airborne).
Air control works as in a normal jump (2 units of 8.8 speed per frame toward the stick), holding attack in the air starts an **air attack** (anim 1:0xD, attack id 0x10, same record as the running jump attack; ends on landing).
Landing: normal landing state 0x404, anim 0:6, then idle. Measured (M: d2a_tie):

| initiator (partner) | launch after start | launch height | peak | flight frames | horizontal distance |
| --- | --- | --- | --- | --- | --- |
| Mustapha (Mess) | +40 | 38 | 107 | 51 | 153 px |
| Hannah (Jack) | +40 | 86 | 155 | 57 | 171 px |
| Mess (Mustapha) | +30 | 54 | 123 | 53 | 159 px |

Partner attacks held in flight: attack id 0x10 started (M: `_atk` variants: state kept, atk 16).
Both players have a blinking 180 frame invulnerability: the partner from the tie up start, the initiator from the end of its swing. M: partner rinv 180 at start+1, initiator 180 at the end of its animation. The
sparkle effect `0x958E4` is spawned every 4th frame of that timer (pool C class 0x5C).
No score, no HP cost, no hits happen in the tie up itself. Friendly fire of the air attack is covered in section 10.

### 9.6 Rules found in the code that are not obvious

- It works for any pair, with 2 or 3 players. With 3 players each player has two counters (`+$C2`, `+$C3` for the two others).
- Neither hero may carry a heavy object (0x20A); a normal weapon in hand is allowed by the code (open item 4).
- The partner's state is entered by the start routine through table 0x6902; the `$00 = 2` of the partner is cleared by the `+$EC` timer when it ends (the same code as the respawn blink).
- A held partner has frame class 0xE, so it is updated by `held_players_update` (0x188DA) and skipped by the normal player frame.

Port: **MISSING** (trigger 0x6728, state 0x1A, four partner states, follow 0x6922, table 0x6994, `player_held_follow` for a player holder, the blink sparkles). The Rust file already has a comment
at `player_run_group` (step 4) and `player_air` (0x22..0x26) saying "not ported".

## 10. Players hurting each other (friendly fire)

R: pass 0x10664 (called first in the collision phase 0x10548). **It is on in normal play.** The gate is `G.$4F3`, which is the **number of alive enemies counted by the last frame's census**
(0x3242E copies the running count `G.$4FB` into `G.$4F3`, enemies with `+$7D >= 0`; the enemy SPEC 5 says the same). It is NOT a player count:

| alive enemies (`G.$4F3`) | friendly fire | depth window (attacker z minus victim z) |
| --- | --- | --- |
| 0 | on | -1 to +1 |
| 1 | on | 0 to +1 |
| 2 | on | exactly 0 |
| 3 or more | off | - |

(Table 0x106F2: (offset, limit) = (1, 2), (0, 1), (0, 0).) Other conditions: `G.$4DD == 0`; the attacker's current frame has class **4** (so the special, class 6, never hurts a friend); the victim is
not invulnerable (`+$00` bit 1 clear, not in class 6, 8 or 0xA) and has a body box; one swing hits a friend only once (attack serial); the boxes overlap.
Effect on the victim: `+$00 = 2`, state 0x02020000, HP minus **1** (never below 1), stun meter minus the record's stun, the reaction code of the attacker's record (so a jab flinches, a finisher knocks the friend down),
hit stop from the record on both players, hit sound, hit spark, and the weapon is dropped (0x1177A); the victim's power factor is re-rolled. The attacker also gets the hit stop.
M: d2a_D `ff_jack`, `ff_mess` (two heroes, no enemy alive, same Z): each of the 4 chain hits cost the other player 1 HP (100 to 96) and made it flinch (light hurt 0x1A from behind) and then
be knocked down by the finisher; the later special did nothing to the friend.
Not tested: 3 players and the 1 and 2 enemy windows (open item).
Port: the pass is ported (`pass_10664`) but `player_g4f3()` returns the constant 3, so the whole pass is off. Fix: return the census count (`self.spawn.g.alive`, already traced as `alive` at 0x4F3).

## 11. Poses, taunts, idle behaviour, stage clear

### 11.1 The 1UP pose (state 0x18, heroes 1 and 2 use air states 0x418 and 0x41A), anim 0:0xE

R: routine 0x6B98 (score threshold check, called from the HUD score redraw 0x10114, 0x10138, 0x1015C) compares the score before and after against the list for the extra life setting
(table 0x6BFE holds four lists: never; 300,000 then 700,000; 500,000 then 1,000,000; 1,000,000 only. The test mode menu offers 300000, 500000 and none, W: `plan/game/design/asset-catalog.md` N6). When a threshold is crossed:
`+$10E = 1`, `+$11C = 240` (the pose must start within 240 frames), `+$A3 += 1` (extra life, D2b), 1UP marker and sound (0x1843C, 0xA4CA, 0x162F8). The pose is started by `idle()` (0x1916E) when the
hero is next in idle: `+$00 = 2`, effect 0x2A4FC, state 0x18 entry 0x19996, voice 0x9814, anim 0:0xE. If the hero stays busy for 240 frames the pose is skipped (`+$10E` is cleared by the timer).
M: d2a_D `oneup_*` (score poked to 499950, three jabs of 100 points): `$10E` = 1 on the hit that crosses 500000; Jack went into state 0x18 at +23 frames (the chain was interrupted at its second step)
for 38 frames; Mustapha air state 0x418 from +35 for 83 frames (y 2: he hops); Hannah air state 0x41A from +22 for 110 frames; Mess state 0x18 from +56 for 75 frames. At the end `+$EC = 180`, `+$00 = 2`
(180 frame blink invulnerability) and the hero is idle.
The animations (all have no hurt box, so the hero is **invulnerable** for the whole pose): Jack 3 frames, 33 ticks (a muscle flex, ends with a 15 tick hold, frame flag), Mustapha 8 frames, 183 ticks (a crouch,
somersault in the air, ground and a thumbs up), Hannah 7 frames, 181 ticks (hands to the chest, then arms raised twice), Mess 4 frames, 175 ticks (double biceps pose and arms over the head).
Ground heroes end the pose 20 frames (`+$11D = 0x14`) after the animation reaches its flagged frame (Jack at tick 18, Mess at tick 55); the air heroes use the same animation in their own air states (entries 0x1ADA6 and 0x1AE62) and the measured lengths above.
Port: the score hook is ported (`hud_extend_check` in `hud.rs` sets `+$10E`, `+$11C = 0xF0`, adds the life and plays the sound), but the **pose itself is MISSING**: the idle code has a TODO for the
`+$10E` test (`player_idle`), and the states 0x18, 0x418 and 0x41A fall to idle or fall (`player_ground`, `player_air`). D2b owns the thresholds.

### 11.2 Idle and taunts

No idle pose, no taunt button, no random fidget (section 3.1). The tag team swing and the 1UP pose are the only poses in the player code. At game start the heroes are placed by script (D4c).
The state 0x2A (0x19EA6: two small hops, walk with weapon pose) and 0x2C (0x19F40: an effect object at the hero's chest, 40 frame wait, then a pose, anim 0:0x1F, and `jsr 0x81FB0`) are scripted
states used by the opening scenes (script commands at 0x1DD3A and 0x1E2AA; the leader hero only for 0x2C); the scripted control can also fire the special (0x1E2EE: sound 2, special). They are D4c items.
The old note `$c0` is the **leader flag** (the hero furthest right, set by 0x1E5F6), not a pose. The 0x1A00 to 0x3A00 copy of the player code is dead (its 0x18 and 0x1A state starts at 0x22BA, 0x3680, 0x373C).

### 11.3 Stage clear

M: d2a_D `clear_jack`, `clear_hannah` (stage clear tally flow poked at +20): the hero stays in idle (state 0x02000000) for all 750 frames of the tally and map panel; the tally shows a portrait, not the hero
pose. There is **no stage clear pose or taunt**. (The walk off after a boss is a scripted walk, D4c.)

### 11.4 Respawn and drop in

R: respawn 0x18B38 (drop from the top, state 0x042E, anim 0:0x1D, **no hurt box**, `$EC = 360` frames of blinking invulnerability, 60 frame wait) and the bazooka of a joining player 0x65A4. D2b has the numbers.

## 12. Pick up and weapon moves (summary only, D2c owns the numbers)

Pick up: state 0x14, anim 0:0xA (11 ticks, no hurt box), invulnerable afterwards 10/8/20/10 frames (id order). Weapon poses are set 5; the weapon states 0x16, 0x1C, 0x1E, 0x20, 0x22 and the run swing
0x202 are DONE in the port. Hero differences seen: Hannah keeps the guns 0x204 to 0x208 when hit, Mess keeps the bazooka (R: 0x1177A); Hannah cannot jump while carrying a heavy object 0x20A.
**Hannah's knife** (the "club" kind 0x202, item type 2): the item flashes red (palette 0x8A) when an enemy is within reach, and every hero swings the knife at close range (M: d2a_D `knife_*`: all four
swung at a thug 34 px away; the web statement that only Hannah stabs is wrong for melee range). The difference is the **swing animation length** and so the stab rate:
Hannah's swing state 0x16 lasts 5 frames (10 for the first), Jack's and Mustapha's and Mess's 20 to 26 frames. With the attack button tapped every 12 frames Hannah stabbed on every tap (hits
at +72, +84, +96: 17, 17, 10 damage), Jack hit 24 once per 24 to 36 frames and Mess the same. Mustapha swung but did not reach the thug at 34 px (no hit). W: AQM items page (https://www.arcadequartermaster.com/cad_items.html):
"Hannah can stab at close range, and the knife is more effective when it flashes red".

## 13. Differences between the heroes (one table)

| feature | Jack | Mustapha | Hannah | Mess |
| --- | --- | --- | --- | --- |
| walk / run speed rank | 3rd / 3rd | 1st / 2nd | 2nd / 1st | 4th / 4th |
| jump height | 76 (neutral) | 76 | **94** | 72 |
| dive window / distance | top, 144 px | top, 187 px | top, 168 px | top, 134 px |
| running jump | 181 px, peak 59 | 175 px, peak 47 | 200 px, peak 74 | **225 px**, peak 36 |
| punch chain base | 10, 10, 15, 29 | 10, 10, 15, 29 | 10, 10, 15, 29 | **12, 12, 17, 32** |
| special | ground spiral uppercut, 38 f | **air** slash kick, 65 f | ground spiral smash, 35 f | ground clothesline, 53 f |
| special damage | 20, stun 11 | 20, stun 1 | 20, stun 11 | 23, stun 1 |
| dash attack | **ground slide** (low profile) | air lunge | air lunge (highest) | air lunge (low, short) |
| command move | flip kick forward | backward somersault | low rolling elbow | big flying lariat |
| throws | one | one, reverses a hold | one | **two** (forward slam, backward suplex) |
| combo throw damage (base) | 22 | 20 | 24 | **29** |
| wall jump | no | yes, flat and fast | yes, high | no |
| take damage factor | 1.00 | 1.06 | 1.09 | **0.94** |
| invulnerable after throw / pick up | 12 / 10 | 10 / 8 | 14 / 20 | 18 / 10 |
| keeps weapon when hit | no | no | guns | bazooka |
| heavy object | jumps | jumps | **cannot jump** | jumps |
| tag team (initiator) | Let's Go (run) | Let's Roll | Jump | Let's Fly |
| knife swing | 20 f | 20 f | **5 f stab** | 21 f |

(R: records, tables above; M where measured.)

## 14. Attack records and damage dealt to a standard enemy

Record layout (R: 0xA26FC, 8 bytes per id): damage, stun (subtracted from the victim's stun meter `+$78`, starts 10), reaction code, hit stop (frames both sides), hit sound, score code, spark type, victim `+$9A` (meaning
unknown). Cells are `damage / stun / reaction / hit stop`, then the damage range against a full HP thug (class 2 variant 0, level 0, rank 0) when the hero's `$C6` is 32.

| id | move | Jack | Mustapha | Hannah | Mess |
| --- | --- | --- | --- | --- | --- |
| 00 | jab, punch 2 | 10/1/00/9 (10) | 10/1/00/9 (10) | 10/1/00/8 (10) | 12/1/00/9 (12-13) |
| 02 | punch 3 | 15/3/02/9 (15-16) | 15/3/02/7 (15-16) | 15/2/02/8 (15-16) | 17/2/02/9 (18) |
| 04 | finisher | 29/11/06/13 (30-31) | 29/11/06/13 (30-31) | 29/11/06/12 (30-31) | 32/11/04/13 (34-35) |
| 06 | knee 1 | 10/3/00/8 (10) | 10/3/00/8 (10) | 10/3/00/8 (10) | 15/2/00/8 (15-16) |
| 08 | knee 2 and later | 15/5/02/8 (15-16) | 15/5/02/8 (15-16) | 15/5/02/8 (15-16) | 20/5/02/8 (21) |
| 0A | throw | 22/11/02/2 (23-24) | 20/11/02/2 (21) | 24/11/02/2 (25-26) | 29/11/02/2 (30-31) |
| 0C | low jump attack | 8/10/10/6 (8) | 8/10/10/6 (8) | 10/10/10/6 (10) | 10/10/10/6 (10) |
| 0E | dive | 15/10/12/6 (23-24) | 15/10/12/6 (23-24) | 15/10/12/6 (23-24) | 15/10/12/6 (23-24) |
| 10 | running jump attack | 15/6/02/10 (23-24) | 15/6/00/10 (23-24) | 15/11/04/10 (23-24) | 20/1/02/6 (31-32) |
| 12 | special | 20/11/04/7 (21) | 20/1/04/3 (21) | 20/11/04/5 (21) | 23/1/04/5 (24-25) |
| 14 | dash start | 12/5/02/8 (12-13) | 12/5/14/6 (12-13) | 12/5/14/6 (12-13) | 12/5/0E/4 (12-13) |
| 16 | weapon run attack | 7/6/12/7 | 7/1/12/7 | 10/11/12/7 | 10/1/12/10 |
| 18 | escape when held | 15/11/06/8 | 15/11/06/8 | 20/11/06/8 | 20/11/06/7 |
| 1A | tag team partner run attack | 15/11/04/8 | 15/11/04/8 | 20/11/04/8 | 20/11/04/7 |
| 5A | dash hit 2 | 20/6/14/4 (21) | 20/6/14/4 (21) | 20/6/18/4 (21) | 20/6/0E/4 (21) |
| 5C | dash hit 3 and later | 32/11/06/6 (34-35) | 32/11/06/4 (34-35) | 34/11/04/6 (36-37) | 32/11/04/4 (34-35) |
| 70 | multi hit A | 10/1/06/6 | 10/1/04/5 | 12/1/14/4 | 13/1/04/6 |
| 72 | multi hit B | 32/1/00/6 | 10/1/04/6 | 6/3/14/4 | 32/1/00/4 |
| 74 | multi hit C | 16/3/10/4 | 26/1/00/4 | 4/3/14/4 | 10/3/00/4 |
| 76 | multi hit D | 4/11/04/6 | 14/6/00/4 | 30/11/06/8 | 6/4/00/4 |
| 78 | multi hit E | - | 8/6/00/4 | - | 4/11/04/8 |
| 7A | multi hit F | - | 4/11/04/8 | - | - |

Reaction codes (R: SPEC 5.2): 00 light, 02 light alt, 04 knock down, 06 heavy knock down, 08 thrown, 0E to 12 pushback (light to strong), 14 to 18 pushback strong.
Command move ids 0x70 to 0x7A mean different things per hero (section 8). Hit sound (record byte 4): 03 light, 06 medium, 09 heavy; throws 0x15, 0x18, 0x1B, 0x1E (Jack,
Mustapha, Hannah, Mess); dash hit 2 of Mess 06. Spark type 0x18 (big) on the finisher (all) and Jack's running jump attack, else the default spark (R: 0x12720; D3f).

**Score per hit** (record byte 5, the offset into BCD table 0x10292): 0x14 = 100, 0x18 = 200, 0x1C = 300, 0x20 = 400, 0x2C = 700, 0x30 = 800, 0x38 = 1000.
Jab, knee 1, jump attacks, dive, running jump attack, dash hit 3 and later 100; punch 3, knee 2, dash start and hit 2 200; finisher, throw 400; special 400 (Jack, Mustapha, Mess) or 100 (Hannah); escape 800; tag team run attack 1000.
Points are awarded for each hit, not only for kills (R: 0xFFC8; D2b).

**Damage formula** (R: 0x2B904, SPEC 4.3): `base * C6/32 * rankfactor/32 * (enemy HP row factor) * (enemy HP band factor)`. For the standard thug at level 0 the rank factor is 32 (1.0) and the row factor 32.
The HP band factor of the **enemy** shrinks the damage as it loses HP: 34 or 35 /32 at 75 HP or more, 31 to 33 at 50 to 74, 27 to 30 at 25 to 49, 26 to 28 below 25 (table 0x2BB4E).
The hero's `$C6` (R: 0x11738) is rolled at the start and again each time the **hero** is hit, from table 0x2C9B6 by the hero's HP: at HP 75 or more it is 32 in 27 of 32 rolls and 24, 28, 36, 42, 48 in one roll each;
at HP below 25 it is 32 (22 of 32), 36, 42, 48 and once 64 (low HP hits harder). The result is a 16 bit calculation, truncated before the last shift (R: SPEC 4.3).
Air weak spot: enemy types have up to two "weak" attack ids (table 0xA32FA); for almost every type they are the running jump attack 0x10 and the dive 0x0E with a factor 48/32 = 1.5 (types with a first entry 41 use 16/32 and
33/32 for the dive). That is why the dive and running jump attack show 23 to 24 above for a record of 15.
M check: the formula reproduces MAME. In M: d2a_fight2 every hero's `$C6` happened to be 42 (god mode, never re hit), and the measured hits match the formula with C6 = 42: Jack jab 13 to 14 (formula 13-14),
punch 3 18 to 19 on a thug at 72 HP (formula 20 at full HP), finisher 39 (40-41 at full HP, thug at 54 HP), throw 28 (29-30), special 27 to 28 (27-28), dash 0x5A 28 (27-28), 0x5C 42 to 45 (44-45), running jump 30 (29-30).

## 15. Invulnerable frames (all of them for the heroes)

R: enemy attacks need the hero's frame class in {0, 2, 4, 0xC, 0xE} and a body box (SPEC 5.1). So a frame is **safe** when its class is 6, 8 or 0xA, or when its body box is 0, or when `+$00` bit 1 is set.
Timers (R: SPEC 5.9, 0x6C3A): hurt sequence (`+$00 = 2` the whole time), 20 frames after getting up, throw 12/10/14/18, pick up 10/8/20/10, special 60, respawn 360, tag team 180 for both heroes.
Animation based safe frames (R: dump; ticks of the animation; N = no hurt box; M where measured):

| move | Jack | Mustapha | Hannah | Mess |
| --- | --- | --- | --- | --- |
| special (1:8) | all 39 (class 6) | all 118 (class 6) | all 36 | all 54 |
| throw (3:3, 3:4) | all 23 | all 23 | all 23 | all 41 and 47 |
| pick up (0:0xA) | all 11 | all 11 | all 11 | all 11 |
| dash recovery (0:0xD) | none (real box 47 to 63 px high) | all 16 | all 13 | point box at the feet (0 x 0 x 0), safe except for blows that reach floor level |
| dive landing (0:0x1E) | none (real box) | all 18 | all 22 | point box at the feet, as above |
| command part 1 (1:0xE) | first 3 ticks | first 3 ticks | first 4 ticks | none |
| 1UP pose (0:0xE) | all 33 | all 183 | all 181 | all 175 |
| tag team swing (0:0xF) | all 45 | all 60 | all 60 | all 60 |
| tag team tumble start (0:0x1C) | none | first 3 ticks | first 3 ticks | none |
| drop in (0:0x1D) | all 80 | all 40 | all 40 | all 80 |
| hurt (set 2), falling | class 8 and 0xA | same | same | same (D2b) |

Not invulnerable (hurt box present): the whole chain, dive, low jump attack, running jump attack, dash start (Jack's low slide is low, see section 7), lunge flight, the command move after its first ticks,
the grab hold and knees (class 0xC counts as hittable), walking, running, jumping, landing.

## 16. Scripted and other states that exist in the player code (not moves)

0x2A / 0x2C scripted hops and poses (section 11.2); air 0x32 (0x1BBA6, wait in the vehicle until `G.$505`, then jump out), 0x34 (0x1BC8A, scripted jump to a point), 0x36 dive landing, 0x2E drop in; run group state 6
(0x1E1EE, entered from outside the player code); air 0x1C (0x1AFD0). These belong to the scripted control (0x1DA10, 0x1DB0C, 0x1DB5E) and the car section (D4c).

## 17. Acceptance scenarios

Probe lines are the ones used here (`jump 1 0 897 char=HERO` puts the hero at the start of Episode 1 area 0, camera 897, x 993, z 296; `god on` keeps hit tests off for solo tests; for the thug tests the thug is
`spawn A 2 var=0 ax=1027 z=296` pinned with `hold` on its X (+$08) and Z (+$10); `rank 0 ; level 0`). Segments are independent; each costs about 2 seconds of MAME time. The complete input files of this study are on disk in
`re/out/d2a/` (`idle.txt`, `basic.txt`, `dtap3.txt`, `fight2.txt`, `tie.txt`, `wj.txt`, `batchD.txt`, `wallscan.txt`; git-ignored, not committed); the lines below are their summaries.
Numbers are the frames or values that the port must reproduce.

| # | scenario (per hero, all four) | probe lines (summary) | must match |
| --- | --- | --- | --- |
| A1 | idle 2000 frames | `jump 1 0 897 char=H`, `god on`, `frames 2000`, `watch 1 P1 st anim ftimer` | state stays 0x02000000; anim frame durations of section 3.1 (Jack: first pass of 800 ticks then a loop of 440; Mustapha, Hannah loop 230; Mess first pass 270 then loop 150) |
| A2 | walk 8 directions, 40 frames each | key lines `10-49:R`, `80-119:L`, ... | vx, vz exactly the section 1 table; 12 frame walk cycle of 4 ticks |
| A3 | run | `10-10:R`, `13-70:R`, `71-71:L` | state 0x0200 from t=14; burst speed 7 frames (Jack) or 5 (others) then 844/870/896/819; stops on the L tap |
| A4 | double tap window | one segment per gap: `10-10:R`, `(11+g)-(21+g):R` for g = 4, 7, 8, 9, 10, 11, 12, 13 | run for g <= 11, walk for g >= 12 (second press 12 frames after the first still runs) |
| A5 | jumps | `10-10:J`, `(t-1)-t:RJ` etc. 120 frames apart | peak 76/76/94/72, air 47/47/48/46 frames, landing 6/9/9/9, forward distance 100/107/89/98 |
| A6 | jump attacks | `10-10:J`, attack held 3 frames from +3, +9, +16, +20, +26 | low attack (0x0C) at +3..+16 and +26; dive (0x0E) only at +20; dive landing state 0x436 |
| A7 | running jump + attack | run, `RJ` at +16 of the run, A at +14 after it | state 0x416/0x428/0x42A/0x42C; distance 181/175/200/225 with no attack |
| A8 | chain on a pinned thug | A at 0, 12, 24, 36 | hit frames of the table in 5.1; damage per hit follows section 14 with the hero's `$C6` |
| A9 | combo throw | chain + `UA` (back) and `DA` (forward) at the 4th press, pressed right after the 3rd hit | throw state 0x0600, 22 frames (Mess 40/46), damage 0x0A, thug reaction 8 |
| A10 | grab, knees, throws | walk R 13 frames into the thug, A at +20, +34, +48; `RA` and `LA` throws | knee 1 / 2 / 3 damage and the release after the third knee; throw state lengths 22 (Mess 40, 46) |
| A11 | special with three thugs | `AJ` with thugs at +20, +34, +46 | HP 100 -> 90 -> 82, damage 20 (Mess 23) per thug, state lengths 52/65/45/58 with hits (38/65/35/53 without) |
| A12 | dash attack | run R, A at +22 of the run | slide/lunge distances 83/142/142/124 px, recovery 15/15/12/15 frames, hit ids 0x14, 0x5A, 0x5C |
| A13 | command moves | `D` (2 frames), `U` from +4, `UA` at +8 | state 0x430 lengths 62/40+15/35+13/46+24 frames; travel 99/-36/134/92 px; peaks 65/81/17/89 |
| A14 | wall jump (Mustapha, Hannah; Jack as control) | `jump 1 2 char=H`, pokes P1 x 215, z 318, `10-60:L`, `10-10:J`, `32-32:J` | state 0x0402 at x about 165, vx 1536/1152, vy 896/1280; Jack never |
| A15 | tag team, 4 initiators | `jump 1 0 897 char=I,P` (no `god`), pokes P1 x 1000 z 296 and P2 x 1010 z 296 | tie up at +92 (90 frames of stillness); launch +30 (Jack, Mess) or +40; flight and run numbers of section 9; both blink 180 frames |
| A16 | friendly fire | two heroes, no enemies, chain on the friend | HP -1 per hit down to 96, flinch and knock down; special does nothing; with 3 or more enemies alive: no damage (to run) |
| A17 | 1UP pose | `score 499950`, chain a thug | `$10E` = 1 at the crossing hit; pose lengths 38/83/110/75 frames; then 180 frame blink |
| A18 | stage clear | pokes of recipe `fe-tally`: sub mode 4, `G.$508 = 0`, `G.$4D5 = 1`, `G.$4D4 = 1` | hero stays in idle for 750 frames |
| A19 | knife (Hannah vs Jack) | `spawn E 0 var=4`, pick up, thug at +34, A every 12 frames | Hannah swing state 0x16 lasts 5 frames (first 10); Jack 20 to 26 |

A single full probe run of all of these takes about one minute. The same inputs must be driven into the port with the harness `run` mode and compared state by state (`st`, position, `atk`, `hs`).

## 18. Port status and the port task list

Checked against `player.rs`, `player_combat.rs`, `player_weapon.rs` and the HANDOFF.

| move or system | Jack | Mustapha | Hannah | Mess | note |
| --- | --- | --- | --- | --- | --- |
| walk, run, double tap, run timeout | DONE | PARTIAL | PARTIAL | PARTIAL | shared code, per hero tables extracted; only Jack was compared in MAME runs before |
| jump, air control, landing, fall | DONE | PARTIAL | PARTIAL | PARTIAL | data driven |
| jump attacks, dive, dive landing | DONE | PARTIAL | PARTIAL | PARTIAL | split heights per hero extracted |
| running jump and its attacks | DONE | PARTIAL | PARTIAL | PARTIAL | four states 0x416, 0x428, 0x42A, 0x42C present |
| wall jump, triangle kick | n/a | PARTIAL | PARTIAL | n/a | code present (0x1A744); depends on terrain bits, untested in a MAME run of the port |
| chain, combo throw | DONE | PARTIAL | PARTIAL | PARTIAL | Mustapha's alternate finisher anim 1:7 coded |
| grab, knees, throws, Mess two throws | DONE | PARTIAL | PARTIAL | PARTIAL | Mustapha's hold reversal coded |
| throw an enemy into others (5 damage) | DONE (enemy side) | | | | R: 0x1185A in `enemy_combat` |
| special and its HP cost | DONE | PARTIAL (air special coded) | PARTIAL | PARTIAL | |
| dash attack (slide / lunge) | DONE | PARTIAL | PARTIAL | PARTIAL | |
| command moves | DONE | PARTIAL | PARTIAL | PARTIAL | all four coded (0x1B6EC..0x1BBA0) |
| tag team (tie up) | MISSING | MISSING | MISSING | MISSING | trigger 0x6728, states 0x1A, 0x204, 0x422, 0x424, 0x426, follow 0x6922 |
| friendly fire | PARTIAL | PARTIAL | PARTIAL | PARTIAL | pass ported, gate constant 3 (off); real gate is the enemy census count |
| 1UP pose | MISSING | MISSING | MISSING | MISSING | states 0x18, 0x418, 0x41A (the score hook 0x6B98 exists in `hud.rs`) |
| idle animations | DONE | DONE | DONE | DONE | |
| scripted states 0x2A, 0x2C, air 0x32, 0x34, run state 6 | MISSING | | | | flow port (M2.T2.3) |
| voices, hit sounds for moves | MISSING | | | | `snd.rs` has the voice routines; nothing is played |
| effects of moves (dust, sparkle blink, sparks, stars, water) | PARTIAL | | | | D3f |

Port task list (sizes: S under 1 hour, M 1 to 2 hours, L over 2 hours):

| task | scope | size |
| --- | --- | --- |
| P-D2a-1 | Friendly fire gate: return the census count from `player_g4f3`; scenario A16 for 2 players and 3 enemies alive | S |
| P-D2a-2 | Tag team: trigger and counters, state 0x1A, four partner states, follow 0x6922 with table 0x69D4.., `held_players_update` for a player partner, `EC` blink for both, scenario A15 for four initiators | M |
| P-D2a-3 | 1UP pose: states 0x18 (Jack, Mess), 0x418 (Mustapha), 0x41A (Hannah) with `$10E`, `$11C`, `$11D`, the idle test at 0x19176, effect 0x2A4FC, voice 0x9814 and the 180 frame blink; the hook 0x6B98 already exists in `hud.rs` | S |
| P-D2a-4 | MAME checks for Mustapha, Hannah and Mess: scenarios A1 to A13 (walk, jumps, chain, grab, special, dash, command), fix whatever differs | M |
| P-D2a-5 | Wall jump check in Episode 1 area 2 (A14), find the real spots (open item) | S |
| P-D2a-6 | Scripted states 0x2A, 0x2C, air 0x32, 0x34, run state 6 (with the flow port) | M |
| P-D2a-7 | Move voices and sounds (D4e list), hit sounds from the records | M (with M4) |
| P-D2a-8 | Effects used by moves: dust 0x92E5E, sparkle blink 0x958E4, spark types, dizzy stars, water ripple | M (D3f) |

## 19. Open items

1. 3 player friendly fire and the 1 and 2 enemy windows were not run in MAME (the batch with these probes, `re/out/d2a/batchE.txt`, was submitted in a tool call that the user rejected, so I did not retry it and did not use any output it may have left in `re/out/probe/d2a_E`; re-run it with the user's agreement before porting P-D2a-1):
   dive frame scan per hero, 3 player friendly fire (`char=jack,hannah,mess`), Jack partner run attack with a direction held, triangle kick after a wall jump.
2. Which wall places real players use for the wall jump (terrain bit 4 spots listed in 4.4 are candidates; only Episode 1 area 2 was tested).
3. What record byte 7 (victim `+$9A`, 1 for the special of Mustapha, Hannah and Mess) does (probably a hit effect flag; D3).
4. Whether the tag team works while one partner carries a normal weapon (the code allows it) and exactly how the 3 player pairing works when all three overlap (counters per slot; probably first match wins).
5. Exact knife numbers per hero belong to D2c (reach of Mustapha's knife, damage per hero).
6. D2b: the hero damage factor order (this file says Mustapha 34, Hannah 35 by id) and the C6 re-roll rule.
7. The select screen star rows are in cursor order (my reading, [I]); D4d to settle.
8. Ending scene hero poses (scripted, D4d) and the opening scene walk (D4c).
9. The prose of `SPEC.md` says "Hannah" for id 1 in 4.4 (alternate finisher 1:7), 4.6 (air special) and 5.7 (reverses a hold); all three are Mustapha's (id 1) and the port already keys them on id 1. SPEC 3.6 and 4.9 (wall jump values, dive values, recoil values) also list index 1 under "Hannah"; this file uses the id order.

Web sources used: Arcade Quartermaster characters (https://www.arcadequartermaster.com/cad_characters.html) for move names and the tag team names, Arcade Quartermaster items
(https://www.arcadequartermaster.com/cad_items.html) for the knife, Hardcore Gaming 101 (https://www.hardcoregaming101.net/cadillacs-dinosaurs/) for the character roles, the draining attacks
and the rising attacks. Where the web and MAME differ (Hannah the only one who stabs with the knife), MAME wins.

## 20. New rows for mechanics-seen.md

| ID | Mechanic | Source | Port | Notes for the design agent |
| --- | --- | --- | --- | --- |
| P43 | Tag team: both heroes still and overlapped (less than 32 px in X, 0 to 15 in Z, no input, class 0) for 90 frames; initiator swings the partner (Jack: partner runs 43 frames at super speed and can attack with id 0x1A; Mustapha, Hannah, Mess: partner is thrown up and forward, vx 3 px/f, vy 6.5 px/f, can air attack with id 0x10); both blink invulnerable 180 frames | R: 0x6728..0x6922, M: d2a_tie | MISSING | D2a section 9; replaces the STUB row P11 |
| P44 | Friendly fire is active when fewer than 3 enemies are alive (`G.$4F3` is the enemy census count), depth window -1..1 / 0..1 / 0 by count, 1 HP per hit with the record's reaction, special immune | R: 0x10664, 0x106F2, M: d2a_D ff_* | PARTIAL (gate constant 3) | corrects P12 and the HANDOFF note |
| P45 | 1UP pose per hero (set 0 id 0xE): Jack, Mess ground state 0x18; Mustapha, Hannah air states 0x418, 0x41A; starts when idle within 240 frames after crossing the extra life score; invulnerable, then 180 frame blink | R: 0x6B98, 0x19996, M: d2a_D oneup_* | PARTIAL (score hook ported in `hud.rs`, pose states MISSING) | extra life thresholds (500000, 1000000 for the default setting) belong to D2b |
| P46 | Frames without a hurt box: special, throw, pick up, 1UP pose, tag team swing, drop in; dash recovery and dive landing for Mustapha and Hannah (Mess: zero size box at the feet) | R: dump | PARTIAL (data driven) | D2a section 15 |
| P47 | Dive kick window is 2 to 3 frames at the top of the jump (Jack and Mess top = split height) | M: d2a_basic jattack_* | DONE (data driven) | |
| P48 | Dive (0x0E) and running jump attack (0x10) deal 1.5 times the record damage to almost all enemy types (table 0xA32FA) | R: 0x2B904 | DONE | |
| P49 | Attack power factor `$C6` is 32 in 27 of 32 rolls at HP >= 75, re-rolled when the hero is hit; 64 possible below 25 HP | R: 0x11738, 0x2C9B6 | DONE | |
| P50 | Wall jump needs terrain wall bit 4 (or 0x18, 0x1C), found in Episode 1 areas 0 to 2, Episode 5 area 0, Episode 6 area 0, Episode 8 area 1; the triangle kick is the jump attack during the wall jump | R: 0x1A744, M: d2a_wallscan, d2a_wj | PARTIAL | D3e or D4c to list real spots |
| P51 | Thrown enemies hurt other enemies for 5 damage (enemy attack row 0x11), knock down | R: 0x1185A | DONE | |
| P52 | Score per hit from the record's score code: 100 / 200 / 300 / 400 / 700 / 800 / 1000 | R: 0x10292 | DONE | |
| P53 | Hannah's knife stab rate comes from her 5 frame swing animation; the item flashes red when an enemy is in reach; all heroes swing it at close range | M: d2a_D knife_* | DONE (D2c) | web statement corrected |
| P54 | Scripted player states 0x2A (two hops) and 0x2C (pose with an effect, leader only) and the scripted special (0x1E2EE) | R: 0x19EA6, 0x19F40, 0x1E2EE | MISSING | D4c |
| P55 | Stage clear and idle: no pose, no taunt; heroes idle during the tally | M: d2a_D clear_* | DONE | |
| P56 | Run start burst (7 frames Jack, 5 others at class 8 speed), double tap window 12 frames, run lasts 240 frames | R + M: d2a_dtap3 | DONE | |

## Appendix A: animation and box data per hero (numbers from the ROM, generated)

Columns: move, set:id, frames/ticks and whether the animation loops, active windows (tick range, frame class, reach in px in front of the hero for facing right, depth z range and height y range of the attack box,
up to two boxes), top height of the body box, ticks with no hurt box (or class 6, 8, A). Tick ranges count from the start of the animation. Held durations (70 or 120 ticks) are end holds of looping poses.


### Hero 0 (Jack)

| move | set:id | frames/ticks | active windows (tick range, frame class, reach in front of the hero) | body box top height | no hurt box or class 6/8/A (ticks) |
|---|---|---|---|---|---|
| jab | 1:0 | 3/10 end | t1-6 c4 x 9..68 fwd, z -1..9, y 29..71 | top 78..78 | - |
| jab (re-press) | 1:1 | 2/9 end | t0-5 c4 x 9..68 fwd, z -1..9, y 29..71 | top 78..78 | - |
| jab hit pose | 1:2 | 2/3 end | - | top 78..78 | - |
| punch 3 | 1:3 | 3/16 end | t2-10 c4 x 7..68 fwd, z -1..9, y 29..57 | top 71..78 | - |
| finisher | 1:4 | 5/19 end | t4-18 c4 x 7..105 fwd, z -1..9, y 29..72 | top 70..72 | - |
| punch 3 hit pose | 1:5 | 2/4 end | - | top 71..71 | - |
| finisher hit pose | 1:6 | 2/3 end | - | top 72..72 | - |
| finisher alt | 1:7 | 2/3 end | - | top 72..72 | - |
| special | 1:8 | 9/39 end | t0-39 c6 x -14..15 fwd, z -8..8, y 20..39 / x 20..46 fwd, z -8..16, y 18..47 | no body box | t0-39 |
| tie-up run attack | 1:9 | 3/10 end | t0-10 c4 x 15..65 fwd, z -1..9, y 35..76 | top 66..66 | - |
| dive | 1:10 | 2/73 loop | t0-73 c4 x 2..28 fwd, z -1..9, y 23..45 / x 6..69 fwd, z -1..9, y 12..43 | top 78..79 | - |
| low jump attack | 1:11 | 4/86 loop | t1-16 c4 x 18..52 fwd, z -1..9, y 23..62 / x 5..40 fwd, z -1..9, y 48..83 | top 77..79 | - |
| dash / lunge | 1:12 | 3/27 loop | t0-27 c4 x -1..50 fwd, z -1..9, y 0..36 | top 35..35 | - |
| run-jump attack | 1:13 | 4/77 loop | t1-77 c4 x 8..64 fwd, z -1..9, y 27..60 / x 2..48 fwd, z -1..9, y 35..86 | top 77..82 | - |
| command part 1 | 1:14 | 2/3 loop | - | no body box | t0-3 |
| command part 2 | 1:15 | 6/98 loop | t0-23 c4 x 8..74 fwd, z -1..9, y 0..97 / x 9..55 fwd, z -1..9, y 40..90 | top 51..79 | - |
| hold | 3:0 | 1/5 loop | - | top 79..79 | - |
| knee 1 | 3:1 | 5/13 loop | - | top 79..79 | - |
| knee 2+ | 3:2 | 4/14 loop | - | top 79..79 | - |
| throw | 3:3 | 4/23 end | - | no body box | t0-23 |
| throw back (Mess) | 3:4 | 4/23 end | - | no body box | t0-23 |
| idle | 0:0 | 14/800 loop | - | top 75..75 | - |
| walk | 0:1 | 12/48 loop | - | top 75..75 | - |
| run | 0:2 | 10/54 loop | - | top 66..66 | - |
| jump (up/diagonal) | 0:3 | 5/104 loop | - | top 79..90 | - |
| jump flip fwd | 0:4 | 5/103 loop | - | top 51..83 | - |
| jump turn flip | 0:5 | 5/103 loop | - | top 51..83 | - |
| landing | 0:6 | 1/7 end | - | top 60..60 | - |
| running jump | 0:8 | 2/90 loop | - | top 78..79 | - |
| pick up | 0:10 | 2/11 loop | - | no body box | t0-11 |
| dash recovery | 0:13 | 2/16 end | - | top 47..63 | - |
| 1UP pose | 0:14 | 3/33 loop | - | no body box | t0-33 |
| tie-up initiator | 0:15 | 3/45 loop | - | no body box | t0-45 |
| wall jump | 0:18 | 1/10 loop | - | no body box | t0-10 |
| tie-up run (Jack) | 0:23 | 2/19 loop | - | top 47..63 | - |
| tie-up roll (Mustapha) | 0:26 | 9/96 loop | - | top 51..83 | - |
| tie-up jump (Hannah) | 0:27 | 7/96 loop | - | top 66..90 | - |
| tie-up tumble (Mess) | 0:28 | 6/89 loop | - | top 51..79 | - |
| drop-in | 0:29 | 1/80 end | - | no body box | t0-80 |
| dive landing | 0:30 | 2/22 end | - | top 47..63 | - |

### Hero 1 (Mustapha)

| move | set:id | frames/ticks | active windows (tick range, frame class, reach in front of the hero) | body box top height | no hurt box or class 6/8/A (ticks) |
|---|---|---|---|---|---|
| jab | 1:0 | 3/12 end | t1-6 c4 x 13..73 fwd, z -1..9, y 30..74 | top 76..80 | - |
| jab (re-press) | 1:1 | 2/11 end | t0-5 c4 x 13..73 fwd, z -1..9, y 30..74 | top 76..80 | - |
| jab hit pose | 1:2 | 2/4 end | - | top 76..80 | - |
| punch 3 | 1:3 | 4/18 end | t4-13 c4 x 13..73 fwd, z -1..9, y 30..60 | top 76..76 | - |
| finisher | 1:4 | 5/22 end | t5-17 c4 x 13..89 fwd, z -1..9, y 30..65 | top 65..71 | - |
| punch 3 hit pose | 1:5 | 2/11 end | - | top 76..76 | - |
| finisher hit pose | 1:6 | 3/7 end | - | top 71..71 | - |
| finisher alt | 1:7 | 4/19 end | t2-14 c4 x 13..89 fwd, z -1..9, y 30..65 | top 65..71 | - |
| special | 1:8 | 41/118 end | t0-118 c6 x -44..1 fwd, z -36..4, y 0..81 / x 0..44 fwd, z -36..4, y 0..81 | no body box | t0-118 |
| tie-up run attack | 1:9 | 4/15 end | t0-15 c4 x 9..47 fwd, z -1..9, y 50..70 / x 4..63 fwd, z -1..9, y 47..79 | top 70..77 | - |
| dive | 1:10 | 2/73 loop | t0-73 c4 x 10..65 fwd, z -1..9, y 20..45 | top 70..78 | - |
| low jump attack | 1:11 | 4/89 loop | t0-15 c4 x 8..72 fwd, z -1..9, y 39..57 | top 75..76 | - |
| dash / lunge | 1:12 | 3/91 loop | t0-91 c4 x -7..38 fwd, z -1..9, y 32..60 / x 1..69 fwd, z -1..9, y 43..73 | top 73..75 | - |
| run-jump attack | 1:13 | 4/84 loop | t0-14 c4 x -7..38 fwd, z -1..9, y 32..60 / x 8..72 fwd, z -1..9, y 39..57 | top 75..76 | - |
| command part 1 | 1:14 | 9/102 loop | t3-32 c4 x 9..80 fwd, z -1..9, y 19..68 / x -6..80 fwd, z -1..9, y 20..76 | top 49..74, 1 frame(s) no box | t0-3 |
| command part 2 | 1:15 | 2/16 loop | - | top 0..61 | - |
| hold | 3:0 | 1/10 loop | - | top 74..74 | - |
| knee 1 | 3:1 | 3/12 loop | - | top 74..74 | - |
| knee 2+ | 3:2 | 3/15 loop | - | top 75..75 | - |
| throw | 3:3 | 4/23 loop | - | no body box | t0-23 |
| throw back (Mess) | 3:4 | 4/23 loop | - | no body box | t0-23 |
| idle | 0:0 | 9/230 loop | - | top 77..77 | - |
| walk | 0:1 | 12/48 loop | - | top 76..76 | - |
| run | 0:2 | 10/43 loop | - | top 55..71 | - |
| jump (up/diagonal) | 0:3 | 5/103 loop | - | top 75..79 | - |
| jump flip fwd | 0:4 | 6/111 loop | - | top 62..79 | - |
| jump turn flip | 0:5 | 6/111 loop | - | top 62..79 | - |
| landing | 0:6 | 1/10 end | - | top 61..61 | - |
| running jump | 0:8 | 3/82 loop | - | top 50..65 | - |
| pick up | 0:10 | 2/11 loop | - | no body box | t0-11 |
| dash recovery | 0:13 | 2/16 end | - | no body box | t0-16 |
| 1UP pose | 0:14 | 8/183 loop | - | no body box | t0-183 |
| tie-up initiator | 0:15 | 3/60 loop | - | no body box | t0-60 |
| wall jump | 0:18 | 7/116 loop | - | top 62..76 | - |
| tie-up run (Jack) | 0:23 | 10/30 loop | - | top 55..71 | - |
| tie-up roll (Mustapha) | 0:26 | 8/28 loop | - | top 71..71 | - |
| tie-up jump (Hannah) | 0:27 | 27/4369 end | t2399-2523 c4  | top 0..79, 8 frame(s) no box | t863-2399; t4173-4369 |
| tie-up tumble (Mess) | 0:28 | 7/91 loop | - | top 64..67, 3 frame(s) no box | t0-3 |
| drop-in | 0:29 | 2/40 loop | - | no body box | t0-40 |
| dive landing | 0:30 | 2/18 end | - | no body box | t0-18 |

### Hero 2 (Hannah)

| move | set:id | frames/ticks | active windows (tick range, frame class, reach in front of the hero) | body box top height | no hurt box or class 6/8/A (ticks) |
|---|---|---|---|---|---|
| jab | 1:0 | 3/8 end | t1-4 c4 x 12..69 fwd, z -1..9, y 29..66 | top 71..71 | - |
| jab (re-press) | 1:1 | 2/7 end | t0-3 c4 x 12..69 fwd, z -1..9, y 29..66 | top 71..71 | - |
| jab hit pose | 1:2 | 2/3 end | - | top 71..71 | - |
| punch 3 | 1:3 | 2/12 end | t4-12 c4 x 12..71 fwd, z -1..9, y 29..58 | top 59..65 | - |
| finisher | 1:4 | 5/19 end | t4-16 c4 x 12..97 fwd, z -1..9, y 29..59 | top 64..68 | - |
| punch 3 hit pose | 1:5 | 2/5 end | - | top 59..59 | - |
| finisher hit pose | 1:6 | 3/7 end | - | top 64..68 | - |
| finisher alt | 1:7 | 3/7 end | - | top 64..68 | - |
| special | 1:8 | 11/36 end | t0-36 c6 x -35..6 fwd, z 4..44, y 0..73 / x 4..54 fwd, z 4..44, y 0..77 | no body box | t0-36 |
| tie-up run attack | 1:9 | 3/12 end | t0-12 c4 x -10..7 fwd, z -1..9, y 54..69 / x -10..36 fwd, z -1..9, y 52..68 | top 74..74 | - |
| dive | 1:10 | 2/72 loop | t2-72 c4 x -1..53 fwd, z -1..9, y 0..33 | top 62..62 | - |
| low jump attack | 1:11 | 3/89 loop | t5-19 c4 x 4..67 fwd, z -1..9, y 28..47 | top 59..63 | - |
| dash / lunge | 1:12 | 3/90 loop | t0-90 c4 x -3..33 fwd, z -1..9, y 24..52 | top 72..72 | - |
| run-jump attack | 1:13 | 2/72 loop | t2-72 c4 x 3..67 fwd, z -1..9, y 28..48 | top 59..63 | - |
| command part 1 | 1:14 | 5/20 loop | t4-20 c4 x -1..26 fwd, z -1..9, y 0..50 | top 35..48, 1 frame(s) no box | t0-4 |
| command part 2 | 1:15 | 2/74 loop | t4-74 c4 x 22..64 fwd, z -1..9, y 32..61 | top 48..59 | - |
| hold | 3:0 | 1/5 loop | - | top 74..74 | - |
| knee 1 | 3:1 | 3/12 loop | - | top 74..74 | - |
| knee 2+ | 3:2 | 2/12 loop | - | top 74..74 | - |
| throw | 3:3 | 4/23 end | - | no body box | t0-23 |
| throw back (Mess) | 3:4 | 4/23 end | - | no body box | t0-23 |
| idle | 0:0 | 7/230 loop | - | top 69..69 | - |
| walk | 0:1 | 12/48 loop | - | top 76..76 | - |
| run | 0:2 | 10/43 loop | - | top 55..69 | - |
| jump (up/diagonal) | 0:3 | 5/102 loop | - | top 71..76 | - |
| jump flip fwd | 0:4 | 9/112 loop | - | top 62..76 | - |
| jump turn flip | 0:5 | 9/112 loop | - | top 62..76 | - |
| landing | 0:6 | 1/10 end | - | top 55..55 | - |
| running jump | 0:8 | 2/90 loop | - | top 71..74 | - |
| pick up | 0:10 | 2/11 loop | - | no body box | t0-11 |
| dash recovery | 0:13 | 2/13 end | - | no body box | t0-13 |
| 1UP pose | 0:14 | 7/181 loop | - | no body box | t0-181 |
| tie-up initiator | 0:15 | 3/60 loop | - | no body box | t0-60 |
| wall jump | 0:18 | 7/52 loop | - | top 62..75 | - |
| tie-up run (Jack) | 0:23 | 10/30 loop | - | top 55..69 | - |
| tie-up roll (Mustapha) | 0:26 | 9/44 loop | - | top 62..74 | - |
| tie-up jump (Hannah) | 0:27 | 9/44 loop | - | top 62..74 | - |
| tie-up tumble (Mess) | 0:28 | 7/141 loop | - | top 62..74, 3 frame(s) no box | t0-3 |
| drop-in | 0:29 | 2/40 loop | - | no body box | t0-40 |
| dive landing | 0:30 | 2/22 end | - | no body box | t0-22 |

### Hero 3 (Mess)

| move | set:id | frames/ticks | active windows (tick range, frame class, reach in front of the hero) | body box top height | no hurt box or class 6/8/A (ticks) |
|---|---|---|---|---|---|
| jab | 1:0 | 5/10 end | t2-7 c4 x 23..71 fwd, z -1..9, y 29..62 | top 69..71 | - |
| jab (re-press) | 1:1 | 3/8 end | t0-5 c4 x 23..71 fwd, z -1..9, y 29..62 | top 69..71 | - |
| jab hit pose | 1:2 | 3/4 end | - | top 69..71 | - |
| punch 3 | 1:3 | 5/13 end | t4-10 c4 x 24..76 fwd, z -1..9, y 40..63 | top 64..80 | - |
| finisher | 1:4 | 6/21 end | t6-18 c4 x 15..76 fwd, z -1..9, y 40..81 | top 70..84 | - |
| punch 3 hit pose | 1:5 | 2/9 end | - | top 64..72 | - |
| finisher hit pose | 1:6 | 2/11 end | - | top 74..84 | - |
| finisher alt | 1:7 | 5/20 end | t5-17 c4 x 15..76 fwd, z -1..9, y 40..81 | top 74..84 | - |
| special | 1:8 | 11/54 end | t0-54 c6 x 6..89 fwd, z -8..16, y 0..94 / x -13..51 fwd, z 4..44, y 0..90 | no body box | t0-54 |
| tie-up run attack | 1:9 | 6/25 end | t2-10 c4 x -11..34 fwd, z -1..9, y 44..65 / x 0..76 fwd, z -1..9, y 26..73 | top 66..66 | - |
| dive | 1:10 | 2/73 loop | t3-73 c4 x 1..54 fwd, z -1..9, y 19..60 | top 82..84 | - |
| low jump attack | 1:11 | 3/83 loop | t1-13 c4 x 3..54 fwd, z -1..9, y 26..51 | top 75..82 | - |
| dash / lunge | 1:12 | 3/90 loop | t0-90 c4 x -7..43 fwd, z -1..9, y 52..60 | top 66..66 | - |
| run-jump attack | 1:13 | 4/81 loop | t1-11 c4 x -15..28 fwd, z -1..9, y 39..64 / x 14..74 fwd, z -1..9, y 35..70 | top 78..84 | - |
| command part 1 | 1:14 | 7/110 loop | t0-10 c4 x -10..37 fwd, z -1..9, y 16..93; t34-110 c4 x -49..47 fwd, z -1..9, y 2..22 | top 42..84 | - |
| command part 2 | 1:15 | 4/25 loop | - | top 31..54 | - |
| hold | 3:0 | 1/10 loop | - | top 77..77 | - |
| knee 1 | 3:1 | 2/11 loop | - | top 77..78 | - |
| knee 2+ | 3:2 | 2/13 loop | - | top 78..81 | - |
| throw | 3:3 | 7/41 loop | - | no body box | t0-41 |
| throw back (Mess) | 3:4 | 7/47 loop | - | no body box | t0-47 |
| idle | 0:0 | 6/270 loop | - | top 74..74 | - |
| walk | 0:1 | 12/48 loop | - | top 75..75 | - |
| run | 0:2 | 10/43 loop | - | top 66..66 | - |
| jump (up/diagonal) | 0:3 | 5/103 loop | - | top 81..85 | - |
| jump flip fwd | 0:4 | 5/103 loop | - | top 81..85 | - |
| jump turn flip | 0:5 | 5/103 loop | - | top 81..85 | - |
| landing | 0:6 | 3/10 end | - | top 43..47 | - |
| running jump | 0:8 | 2/80 loop | - | top 76..82 | - |
| pick up | 0:10 | 2/11 loop | - | no body box | t0-11 |
| dash recovery | 0:13 | 2/16 end | - | top 0..0 | - |
| 1UP pose | 0:14 | 4/175 loop | - | no body box | t0-175 |
| tie-up initiator | 0:15 | 6/60 loop | - | no body box | t0-60 |
| wall jump | 0:18 | 1/10 loop | - | no body box | t0-10 |
| tie-up run (Jack) | 0:23 | 10/30 loop | - | top 66..66 | - |
| tie-up roll (Mustapha) | 0:26 | 4/78 loop | - | top 66..84 | - |
| tie-up jump (Hannah) | 0:27 | 6/90 loop | - | top 74..82 | - |
| tie-up tumble (Mess) | 0:28 | 6/90 loop | - | top 74..82 | - |
| drop-in | 0:29 | 1/80 loop | - | no body box | t0-80 |
| dive landing | 0:30 | 2/22 end | - | top 0..0 | - |
