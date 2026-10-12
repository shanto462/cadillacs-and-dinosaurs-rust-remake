# D2b: player state rules (HP, lives, death, respawn, invulnerability, hurt, damage, score, time)

Written 2026-10-12 by the D2b research agent. Part of the design phase (`plan/game/NEW-PLAN.md` section 1, D2). Documents only: no code was
changed. The numbers are meant to be built exactly by later port agents.

How to read this file

- **Tags.** `R:` = ROM address (68000 code, `G.$xxxx` = RAM `0xFF8000 + xxxx`, `+$xx` = field of an object) or repo file.
  `M:` = measured in MAME with the probe tool (`re/harness/probe/`, manual `plan/game/design/capture-recipes.md`); the segment name is
  written after it, the logs are in `re/out/probe/d2b-*/<segment>/log.txt` and the input files in `re/out/d2b/` (both git-ignored).
  `W:` = web. `U:` = the user. MAME wins over the web, the ROM wins over the old repo specs.
- **Hero ids** (`+$03`): **0 Jack, 1 Mustapha, 2 Hannah, 3 Mess**. Every per-hero table here is in this id order.
- **Time.** `gf` is the game frame counter G.$0000 (it stops during slow-down frames, so use it to count). `R` is the game frame of the
  respawn start (section 2.3). Frames per second: the game clock tick is 51 frames (`mechanics-seen.md` F01); I give frames, not seconds.
- **Repo reading.** Specs: `re/specs/player/SPEC.md` (sections 4, 5), `re/specs/enemy/SPEC.md` (3, 6, 13), D2a (`D2a-player-moves.md`),
  D4d (`D4d-front-end-and-story.md`), D3e (`D3e-pool-b-inventory.md`), D3b (`D3b-dinosaurs.md`), port code in
  `dino-rs/crates/dino-game/src/game/` (`player.rs`, `player_combat.rs`, `hud.rs`, `rank.rs`, `item/blast.rs`, `enemy_combat.rs`).
- **Source dumps used.** `re/tools/d ADDR N` (disassembly, only read, never copied), `re/out/d2b/*.py` (table dumps and log tools, mine).

## 0. Summary and what the user said (verified)

The user said (2026-10-12): "after hero dies once, respawn comes with invincibility for a time and bombing and damaging enemies but not
eliminating terminal enemies. new coin respawn comes with a bazooka". Result, measured and read in the ROM:

| claim | result | proof |
| --- | --- | --- |
| respawn after losing a life gives invincibility for a time | **yes: 61 frames of waiting above the screen, then the drop, and `+$00 = 2` (hit tests off) for 360 more frames counted from the start of the drop** (421 frames from the respawn start, 329 frames after control returns). It is shown by yellow sparkles (every 4th frame), **not** by a blinking sprite | M: resp_jack (all 4 heroes), cont_level (first hit lands the frame the timer ends); R: 0x18B9E, 0x18CD4 |
| respawn comes with bombing | **yes**: an "air strike" of 5 bombs (pool B class 18 spawns pool E class 8, they make pool E class 1 kind 2 explosions, the 5th also a screen wide invisible blast, pool E class 11) | M: resp_*, strike_* ; R: 0x7B65E, 0x29EC4, 0x27F70, 0x2A4FC |
| the bombs damage enemies but do not finish them | **yes**: every hit is a **flat 30 HP** (no hero factors) and **HP can never go below 1** (an explicit floor). Enemies already at 30 HP or less stay at 1 HP. Bosses lose 30 per hit too. | M: ty_*, hp_floor2, strike_thugs; R: 0x111C6, 0x114F6 |
| a respawn paid with a new coin comes with a bazooka | **yes, and only those**: the bazooka (4 rockets) is given when the respawn state byte `+$07` is 0, which is the case for **join with a coin and continue with a coin**. The respawn after losing a life sets `+$07 = 2`: no bazooka. **The coin respawns also get the bombing, the invincibility and the speech bubble** (the user's sentence does not say it, but they do) | M: join_p2, cont_level, cont_mode (held bazooka) against resp_*, timeover_* (nothing held); R: 0x18BC0, 0x01C00A |

Things the older notes had wrong (corrected here):

1. **Not a blink.** The respawn and tag team invulnerability is shown with sparkle objects, the hero sprite is drawn normally
   (`+$25` stays 0, M: resp_jack). Only the 61 frame **death** blink toggles the sprite every frame.
2. **The bazooka does not depend on a coin but on `+$07`**; the pool C class 14 object is the speech bubble (not a "respawn flash") and the
   pool B class 0x48/4 = 18 object is the air strike (not a "1UP marker"). The real 1UP marker is pool C class 14 variant 2 (section 1.4).
3. **The speech bubble text depends on the number of players alive, not on the kind of respawn**: one player says "They're gonna pay for
   this!", two or three players say "Need a hand?" (M: resp_2p shows "Need a hand?" at a lost life respawn).
4. **Hero damage factor order** is settled: table 0x2C88E is indexed by hero id: Jack 32, **Mustapha 34, Hannah 35**, Mess 30 (section 5.2).
   The two specs differed only in the names (player SPEC used the name order of its own list).
5. **Lie time, mash meter and dizzy tables use their first row only** (the level band is always 0 because of a byte read, section 4.4).
   "Lie time by level" does not exist for heroes.
6. **Dizzy needs a chain of 4 hits or more** that ends in a knockdown (75 percent at 4, always at 5 or more), not 3 (section 4.6).
7. **TIME OVER kills an invulnerable (respawning) hero too.**

## 1. HP and lives

### 1.1 Values

| item | value | source |
| --- | --- | --- |
| hero HP at start, join, continue, after a lost life | 100 (`+$6A` bar, `+$6C` HP, `+$6E` previous HP all 100) | R: 0x4D8E, 0x4DE8; M: start3p (HP 100 for 3 players) |
| HP between areas of one episode | kept (only the stage start 0xAC4E refills; the area start 0xAA0E does not call 0x4DC2) | R |
| HP at the start of the next episode | refilled to **100** for every player (also lives and score are kept) | R: 0xAC4E calls 0x4DC2 for the 3 players; M: hp_carry (hp 30 became 100 when mode 0xC of the next episode started) |
| maximum HP | 100 (food caps at 100). More than 100 only by poke; the HUD would draw extra bars | R: 13.4 |
| extra lives at start (`+$A3`) | `G.$777C & 3` = 0..3, factory **1** ("2 in all"); the same value is given on join and continue (0x4DD6); `G.$4E6` keeps the same number for the rank code | R: 0x4DD6; M: start3p (lives 1 for 3 players), join_p2, cont_mode (lives 1 after the continue) |
| lives are the number of **spare** lives | the current hero is not counted: at 0 the next death is game over for that player | R: 0x1BFD0 |
| life cap | none in the code (`addq.b`, 8 bit wrap); the HUD shows the count as a tile code | R: 0x6BD8 |

### 1.2 HUD of the life counter and the bar

- The player block (3 player layout: columns 0, 16, 32 from row 34; 2 player layout: the two wide blocks): face 3x3 tiles, then a `=` tile (0x3D)
  and **one tile whose code is the number of spare lives**: digits 0 to 9, then letters (10 shows `A`, 11 `B`, 12 `C`). M: lives_hud (12 lives, one death:
  the block showed `=B`), lives9_hud (`=8`). The block is redrawn only when the game redraws it (respawn, extra life, area start), a poke does not.
  R: 0x162F8 (3 player layout), 0x16358 (2 player layout), 0x163E4 (lives). Port: `hud_player_block` DONE.
- Vitality bar (R: 0x16446, port `hud_life_bar`): 3 player layout 11 segments of 8 HP with two end pieces, filled by `(hp - 1 + 2) >> 3` and a partial piece
  from the low 3 bits; HP 0 and 100 are drawn as they are, any other HP is drawn as `hp - 1`. 2 player layout (two player chute mode): HP scaled by 1.5, 18 segments.
  The bar is redrawn right after any HP change by the game code (the probe `hp` poke does not redraw it, M: hp_bar).
- **Low vitality blink**: at HP 15 or less the name row, score and lives row of the block change between two palette sets every 6 frames
  (red and white text). M: hp_bar (red at hp 15, 7 and 1, white at 16 and 14 between phases); R: 0x1667E, port `hud_low_hp_blink` DONE. The bar itself does not blink.
- Score at the right of the block (8 digits BCD) and under it the **place the score would take in the ranking table** as text `1ST`, `2ND` ... `10TH`,
  blank when worse than 10th (M: oneup: `10TH` at 10000 points, `1ST` at 500150; R: 0x15948, port `hud_rank_all`).
- The face icon changes on hits (`face_icon(2)` in the hurt code, R: SPEC 5.3). Not measured, open item 11.

### 1.3 Extra life (1UP)

R: routine 0x6B98, called after every HUD score redraw (0x10114, 0x10138, 0x1015C); port `hud_extend_check` (DONE).

| setting (`G.$777B` bits 5 and 6) | thresholds (absolute score) | note |
| --- | --- | --- |
| 0 | none | |
| 1 | 300,000 and 700,000 | |
| **2 (factory)** | **500,000 and 1,000,000** | lists at 0x6C16 |
| 3 | 1,000,000 only | |

Rule: take the first threshold above the **old** score; if the **new** score reached it (one step only: a single big score jump gives one life, not two), then
`+$10E = 1`, `+$11C = 0xF0` (the pose must start within 240 frames), `+$A3 += 1`, the rank event 0x1843C, sound **0x4A** (panned by the hero's screen X, R: 0xA4CA),
and a redraw of the block (the new `=N`). `+$10E` also blocks a second 1UP until the pose has started or the 240 frame timer ran out (R: 0x18E56). M: oneup (score 499950,
three jabs: lives 1 to 2 on the frame the score became 500050, `+$10E = 1`, `+$11C = 240`).

### 1.4 The 1UP pose and marker

D2a section 11.1 has the pose (heroes' states, lengths 38, 83, 110, 75 frames, no hurt box, then 180 frames of sparkles). New here:

- On the frame where the pose animation reaches its flagged frame (Jack: 18 ticks after the start, the raised fist) a **red "1UP!" text** (pool C class 14, variant 2, spawned by 0x18C1E, called at 0x19A38 for Jack and Mess and at 0x1AE1E and 0x1AF2C for Mustapha and Hannah; handler 0x93BF8) is created above the hero's head (hero Y + 0x58, depth - 8). It lives **180 frames** (`$5A = 0xB4`), flashes the palette every
  frame (0x84 / 0x86) and is freed at once when the hero leaves a ground state (state word 0x0200). M: oneup (object from t = 41 to 219, snapshots show `1UP!`).
- After the pose: `+$EC = 180`, `+$00 = 2`, sparkles every 4th frame (M: oneup, `rinv` 180 at the pose end).
- Port: score hook DONE, pose states MISSING (D2a task P-D2a-3), marker MISSING (task P-D2b-2 below).

## 2. Death and respawn

### 2.1 Death sequence (HP reaches 0)

R: alive check 0x18D88: HP <= 0 while the state is not 0x0202 (hurt) puts the hero into the hurt state 0x02020000 with `$59 = 0x10` (hit stop 16). A real killing blow
goes through the same hurt code. The knockdown code sees `hp == 0` and uses the **dead vectors** (KD4_DEAD, `$CD = 2` slow motion: movement only every other frame), so the flight is
about three times longer than a normal knockdown. M: resp_jack (HP poked to 0 at t = 10; HP 0 and the probe `hp 0` give the same flow):

| frame (t) | game frames after HP 0 | state | what happens |
| --- | --- | --- | --- |
| 10 | 0 | idle | HP set to 0 |
| 11 | +1 | 0x02020000 | hurt state, `$59 = 16` |
| 12 to 130 | +2 .. | 0x02020402 | flight up and back: x moves 80 px behind, rises to 108 px (peak at about t = 70, half speed), lands at t = 130 (`vy` starts at 0x780 = 1920, gravity -64 per motion step, one motion step every second game frame) |
| 131 to 187 | | 0x02020404 | bounce (vy 1664) and second flight, 57 frames |
| 188 to 193 | | 0x02020406 | lying: the usual lie roll (4.4): **6, 16 or 31 frames** (`$E8` = 5, 15 or 30; this run rolled 6, the 2 player run rolled 16); a dead hero cannot mash |
| 194 | +184 | 0x04000000 | death state starts (`$05 = 0`) |
| 195 to 255 | +185 .. | 0x04020000, `+$00 = 2` | **death blink: 61 frames**: `+$25` toggles 8 and 0 every frame (the sprite flickers); every 32 frames an anti tamper checksum runs (0x5DD30, ignore) |
| 256 | +246 | 0x00020002 | lives checked: see 2.2 |

Per hero (M: resp_*): the flight ends at t = 194 for Jack, Hannah and Mess and at t = 196 for Mustapha (animation lengths), so with a 6 frame lie the whole death is 246 to 248 frames from HP 0 to the respawn
start `R` (10 frames more with a 16 frame lie, 25 more with a 31 frame lie). The 2 and 3 player runs rolled a 16 frame lie (death state at t = 206, respawn at t = 268): that explains the 12 frame difference I first saw (M: poke_1p_again 197 and poke_2p_again 206). During the flight the hit tests stay on (`+$00 = 1`) but the frame class is 8, so nothing can hit the hero.
**A real killing blow** (M: realdeath_1p, `hp 3` and a thug with `aggr 7`): the first jab (6 damage, base 6 at HP 3) leaves **HP 1 by the mercy rule** (3 to 1); the next jab at HP 1 kills (no mercy at HP 1). The knockdown from the jab's reaction 4 is the same flight (118 frames in state 0x402, 59 in 0x404, then the lie roll, 16 frames in that run) and the death state started 194 frames after the killing blow, then 61 blink frames and the respawn.
A knockdown of reaction 6 (the thug's heavy hit) uses the states 0x0602, 0x0604, 0x0606 with a longer flight (97 and more than 48 frames, M: realdeath_2p).
If `G.$4D4` (stage clear flag) is set, the death state instead sets HP 1 and returns the hero to idle (R: 0x1BF82; mechanics row T27).

### 2.2 Losing a life, game over

R: 0x1BFC6 to 0x1C020 (port `player_dead_continue`):

- **Lives left (`+$A3 != 0`)**: `+$A3 -= 1`, `+$00 = 1`, the rank event 0x184E2 (below), the hero is re-initialised by 0x4DE8 (state long 0x00020000, HP 100, facing kept), `+$07 = 2`,
  then 0x10482: **stage clock +3:30 (BCD 0x330) capped at 4:00 and the wave timer G.$50C + 600 frames**. M: timeover_life (clock 0:00 became 3:30), cont_*; the clock is added once per respawning player
  (M: timeover_2p, the player without a life added nothing).
- **No lives left**: `+$00 = 0` (the player is gone, his HUD block goes to the continue countdown, D4d section 6.3), the player count G.$4EF is recounted (0x655E, so the **level column of the rank
  table changes when a player drops out**), rank event 0x184E2 and rank speed -1 (0x1842C). If every player is gone the game goes to the continue mode (mode 0x10, D4d 6.3).
- **Rank event of every death** (R: 0x184E2, enemy SPEC 6.4): rank minus 4 at the factory difficulty (words 7, 6, 5, 4, 3, 2, 1, 0 by difficulty 0..7), or minus 3 when another drop happened less than 600 frames ago;
  then the extra life style rank bonus 0x1843C, level recomputed, **calm period G.$608 = 1200 frames (aggression 0)**, and (when the hero still has a life) the rank speed index -4 (1 player), -3 (2 players), -3 or -2 (3 players, a random choice)
  at difficulty 3 (table 0x187F4). M: rank_death (rank 13 to 10 at the first death, which is minus 3: open item 9).
- **Rank effects of join and continue** (not re-measured; D4d 6.2 and 6.3, R: enemy SPEC 6.4): a join runs the rank event 0x18410 (rank speed index +3 while the game is in mode 0xC, clamp 0 to 15, timer reset); a continue runs 0x18566 with d0 = 0xFE after the start press (D4d 6.3).
  Both also add stage time and +1 point to every score (D4d F29).
- Port: death state DONE; `0x184E2`, `0x10482` MISSING in `player_dead_continue` (task P-D2b-1).

### 2.3 Respawn state machine and timeline (lost life, Jack, one player)

R: 0x18B38 (steps `+$06` = 0, 2, 4 at 0x18B4C, 0x18B9A, 0x18BD4). Frames are game frames after the respawn start `R` (the frame where the state becomes 0x00020402). M: resp_jack (R = gf 1192).

| game frame | state | what happens |
| --- | --- | --- |
| R - 1 | 0x00020002 | the lives check ran; HP is 100, `+$07 = 2` |
| **R + 0** | 0x00020402 | the hero is parked **above the screen** at X = camera X + 0xC0, depth Z = camera Y, height = floor + 0x180 (384). `+$00 = 2`, facing right, physics reset (0x18C82, stun meter `+$78 = 10`), **`+$EC = 360` (0x168)**, `+$80 = 60`. **If `+$07 == 0` the bazooka is created (0x65A4)**. **The air strike object (pool B class 18) is created** (0x18C5E). In car mode (`G.$518`) nothing of the last two happens and the wait is 0. |
| R + 0 .. R + 48 | | the air strike drops 5 bombs, one every 12 frames (2.5) |
| R + 1 .. R + 60 | 0x00020402 | the hero waits (`+$80` counts 59 to 0). The invulnerability counter does **not** run yet (it only counts in the alive state): `rinv` stays 360 |
| **R + 61** | 0x0200042E | **the drop**: the hero is placed at the standard drop spot (0x62D6): X = camera X + **96 (1 player), 64 or 128 (2 players: first or second), 48, 96 or 144 (3 players)** by the player count `G.$4EF` **as it was before this drop** (the recount 0x655E runs after the placement, so a hero who joins a one player game drops at camera X + 96, M: join_p2 X 96 at camera X 0, while player 1 of a two player game that lost a life drops at camera X + 64, M: resp_2p X 961 at camera X 897), depth Z = camera Y minus floor + 32, 48, 64 by player index (then the terrain push 0x12CAC raises it to a walkable depth: measured Z 320 for player 1 at camera Y 256 in Episode 1 area 0, and Z 64 for player 2 at camera Y 0 in area 1), height = floor + 0x100 (256). Air state 0x2E (0x1B67C): `vy = -256` (1 px per frame), gravity -128 per frame (0.5 px per frame squared), animation 0:0x1D. The player count is recounted (0x655E). **The speech bubble is created (pool C class 14 variant 0).** From now on `+$EC` counts down 1 per frame and a sparkle (pool C class 23, 0x958E4) is made every 4th frame. |
| R + 61 .. R + 91 | 0x0200042E | the fall: y = 256 down to 0 in 31 frames (samples every 2 frames: 256, 252, 247, 239, 230 ... 54, 25, 0). Landing: sound 0x9C0C (and the water splash 0x606E where there is water) |
| **R + 92** | 0x02000000 | landed, idle, the player has control. `+$00` stays 2 |
| R + 59, 71, 83, 95, 107 | | the 5 explosions (2.5) |
| **R + 421** (drop + 360) | | `+$EC` reaches 0, `+$00 = 1` (when `+$F9` is 0): the invulnerability ends. M: cont_level and cont_mode: drop at t = 522, `+$00 = 1` and the first enemy hit at t = 882 |

So the hero is invulnerable for 421 frames from R, 329 frames after control returns (R + 92 to R + 421). The hero can attack, pick up and use the bazooka during the invulnerability.
Other heroes' drop-in animation lengths (set 0 id 0x1D): Jack and Mess 80 ticks, Mustapha and Hannah 40 (no hurt box, D2a section 15); the timeline above is the same for all four heroes (landing at R + 92 in all four runs,
Mustapha with `R` two frames later because his death flight is two frames longer).

Hero position data (M: resp_jack, resp_2p, resp_3p): drop X 993 at camera X 897 (1 player), 961 for player 1 of 2, 993 for player 2 of 3.

### 2.4 The speech bubble (pool C class 14) and the player marker

R: handler 0x93B28 (variant 0 at 0x93B44 to 0x93C7C), variant 2 at 0x93BF8 (the 1UP marker, 1.4). M: resp_*, join_p2.

- Created at the start of the drop (R + 61). Follows the hero (X, height + 0x58 above the feet, depth - 8). Freed at once when the hero's `+$00` is not 2 (the invulnerability ended or he is hurt) or in car mode.
- **Phase 1 (239 frames): the speech balloon.** Its animation index is **0 when one player is alive ("They're gonna pay for this!") and 1 when two or three players are alive ("Need a hand?")**
  (the check is `G.$4EF != 0`, taken at the drop). It is the same for a lost life respawn, a join and a continue. M: resp_jack (text "They're gonna pay for this!"), resp_2p ("Need a hand?"),
  join_p2 ("Need a hand?").
- **Phase 2 (120 frames): the player marker** "1P", "2P" or "3P" (animation index player index + 2) above the hero. It runs until the invulnerability ends (the 120 frame counter and the end of `+$00 = 2` come within a frame of each other: 61 + 239 + 120 = 420 against 421).
  M: resp_jack (picture at t = 560: "1P").
- Port: MISSING (pool C class 14, task P-D2b-2).

### 2.5 The respawn air strike

R: pool B class 18 (handler 0x7B65E, record list 0x7B7A6), pool E class 8 (bomb, handler 0x29EC4), pool E class 1 kind 2 (explosion, 0x27F70), pool E class 11 (invisible blast box, 0x2A534 created by 0x2A4FC).
Ported already (`item/blast.rs`): E class 8, class 1 (all kinds), class 11 and the enemy hit pass 0x10F96 with the nonlethal path; **missing: the controller (pool B 18) only**.

**The controller (pool B 18).** Created with the player's index and hero id (`+$02`, `+$03`). Init (0x7B672): list pointer 0x7B7A6; X = camera X; floor lookup (0x12FE4) and terrain push (0x12CAC);
`$A6 = X + 0x40` (the start X of every bomb); `$A8 = ground height + 0x120` (the start height of every bomb). Then every 12 frames (`$80 = 10` counting down to a borrow = 11 frames plus the creating frame) it makes one bomb for the next record
of the list and ends after the 5th. Record = (X offset from the camera, depth offset, variant):

| bomb | spawn (frames after R) | X offset (camera X + ...) | depth offset | variant | landing X (camera X 897, start X 993) | landing Z (base 288) |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 0 | 320 | +64 | 0 | 1179 | 352 |
| 2 | 12 | 192 | +32 | 0 | 1059 | 320 |
| 3 | 24 | 64 | +48 | 0 | 1000 | 336 |
| 4 | 36 | 128 | 0 | 0 | 1000 | 288 |
| 5 | 48 | 256 | +16 | **1** (white blast) | 1119 | 304 |

The depth base is `camera Y - floor + 0x20` computed at the target X (288 in the measured area), so on uneven ground each bomb has its own floor (R: 0x7B6F2 to 0x7B72C). **The strike does not aim at the hero**: it is the same screen pattern
for every hero, every player count and every number of players present (M: resp_2p, resp_3p: same landing X and Z). The first bomb is created on the same frame as the respawn state change.

**The bomb (pool E 8).** Flies from (start X, ground + 288) to its target in a straight line in its own depth plane: `vy = -1280` (5 px per frame, no gravity), `vx = q * 256 + r` where `q` and `r` are the quotient and the remainder of
`|target X - start X|` divided by **57** (so the speed is `q + r / 256`, an approximation: bombs whose target lies left of the start X still drift to the **right** at `r / 256` per frame, the sign is lost; kept bug).
It lands when its height is at or below the target height (58 frames), so the explosion is made **59 frames after the bomb**. The landing X above is `start X + vx * 58 / 256` (checked for all five: 1179, 1059, 1000, 1000, 1119).
Every second frame a small trail object (pool C class 7) is made while it falls. The 5th bomb (variant 1) also makes the **white blast** (pool E class 11, variant 0) and **screen shake 3** (0x755C) at its landing.
Every explosion shakes the screen with strength 3 too (R: 0x27FCE) and plays the explosion sound (0x9F12, pan by X). M: strike_thugs, resp_jack.

**Explosion (pool E class 1, kind 2) and the blast (class 11).**

| object | attack id | box | lives | owner |
| --- | --- | --- | --- | --- |
| explosion kind 2 | 0x6E, new attack serial | boxes 90, 91, 92 of table 0x100000 follow the animation (anim 2 of table 0x28102): **X -73 to +68 (141 wide), Z -20 to +28 (48 deep), height 0 to 79 or 84**, active for the first 16 frames of a 26 frame animation | 26 frames | the respawning player (`+$02`, `+$03` copied from the bomb) |
| white blast (class 11 variant 0) | 0x6E, new serial | box 0xC9: **X -256 to +256, Z -256 to +256, height 0 to 256**: the whole screen | 1 frame | the same player |

Measured hit area of one explosion at X 1089, Z 296 (M: box_x, box_z): thugs hit at X 1009 to 1169 (not 989, not 1189) and Z 276 to 316 (not 266, not 326). That is the attack box grown by the thug's own body box.
The blast hit every thug on the screen, also at Z +-200 and X -189 (M: blast_reach).

**What the attack does** (hero attack table 0xA26FC, record 0x6E, equal for all four heroes: damage 30, stun 1, reaction 0x0A (burn knockdown), hit stop 8, hit sound 0x0F, score code 0x14, spark 0, victim byte 0).

- **Enemies on the normal hit list 0x6726 (pass 0x10F96 body 0x111C6):** `HP = HP - 30`, and **if the result is 0 or less the HP is set to 1** (R: the special path for attack ids 0x6E and 0x6C). No hero factor, no level factor,
  no HP band, no weak spot: flat 30. The enemy goes to the hurt state with reaction 0x0A (burn knockdown; the thug flies, burns, lies, gets up, about 100 to 130 frames). Stun meter not touched.
  Score: the record's score code 0x14 = **+100 points per enemy hit** to the owner (M: strike_score, +300 for three thugs). The enemy's `+$02` becomes the owner (the kill credit).
- **Monsters on the hostile list 0x6B64** (Mack, Rock Hopper, Shivat, Zeke, classes 22, 23, D3b section 1.2; pass 0x1132A, port stub): the same HP floor, but the damage is
  **computed with the player damage formula tail 0x2BA2A** (base 30, the counter hit and weak spot tables, the **owner's** `$C6`, the rank by type table, the HP row and band factors). With the owner's `$C6 = 42`
  (the value in all probe runs) Mack lost 36, 42 and 40, Rock Hopper 35. M: ty_mack, ty_rhopper, strike_dino.
- **Hit once per object.** An enemy that is in a hurt state (`+$00` bit 1) is skipped. The knockdown lasts about 100 frames and the explosions come 12 frames apart, so in a real strike **an enemy takes the 30 once**
  (M: strike_thugs: 14 thugs over the whole screen, each 44 to 14 exactly once, none dead); a second hit lands only when the enemy is back on its feet (M: ty_thug_ferris: 44 to 14 at the first explosion, 14 to 1 at the blast 100 frames later,
  then it stays at 1 HP: a further hit at t = 360 landed and HP stayed 1).
- **Players are not hurt.** Pass 0x10DCC skips attack ids 0x6E and 0x6C; the friendly fire pass only looks at player frames of class 4. M: resp_2p (player 2 stood inside bombs 3 and 4, HP stayed 100, no hurt state).
- **Containers** (pool E class 5, all 7 variants) break at once (damage 30 against HP 4, pass 0x120D2, the normal path of the table: HP 0), with the usual break: **+500 points to the owner** (and the drop roll). A TNT barrel loses 30 (HP 50 to 20) and
  **starts its fuse, then explodes** about 110 frames later. M: objs_ex, objs_blast, objs_tnt, strike_score (+300 for three thugs, +500 per container, total 2100 for 6 thugs and 3 containers).
- **Items on the floor** (weapons, ammo boxes, food, treasure) are **not touched**, even at the explosion centre. M: objs_center, objs_blast.
- **Pool B breakables** (walls, panels, steam pipes, fences, windows) are **not hit** by explosions: only players' attacks are tested against that list (pass 0x10CD0). R: 0x10CD0 reads list 0x6720 (players) only.
- **Bosses and large enemies** (all lose 30 per hit, HP floor 1): Butcher 512 to 482 to 452 to 422 to 392 to 362 (five hits, 100 frames apart), Morgan 256 down by 30 per hit, Tyrog (class 4) 256 and Tyrog 91 300 by 30, Tyrog 72
  226 and 196 after its form change, Slisaur (class 71) 350 by 30, Walther (class 18 variant 0x200) 100 to 70 to 40 to 10. M: ty_butcher, ty_morgan, ty_tyrog1, ty_tyrog91, ty_tyrog72, ty_slisaur, ty_walther.
  The Butcher took 36 once while it was in an attack state (strike_dino): the formula path applies when an enemy is also registered on list 0x6B64 in that state.
  **Not measurable with my spawns:** Vice, Slice, Hogg, Blade, Razor, Lash and the class 64 and 79 variants stayed in their intro or uninitialised states (HP stayed 0 or the intro state), so they were never hit (open item 1).
  A sleeping Shivat is not hit either (hostility model, D3b); Zeke is hit only when the box reaches its height (HP 20 to 1 by the blast).
- **TIME OVER during the strike**: the hero dies as usual (section 7).

**Damage per hit by enemy type (M: `ty_*`, one enemy pinned at the centre of one explosion, HP at level 0, hits at t = 60, 160 blast, 260, 360, 460 blast):**

| enemy (class, variant) | start HP | HP after each landed hit | per hit | path |
| --- | --- | --- | --- | --- |
| Ferris, Gneiss, Driver (class 2) | 44 | 14, 1, 1 | 30, floor | flat |
| Poacher J (13) | 44 | 14, 1 | 30, floor | flat |
| Hammer (10) | 64 | 34, 4, 1 | 30, floor | flat |
| Punk (20 variant 0) | 44 | 14, 1 | 30, floor | flat |
| Walther (18 variant 0x200) | 100 | 70, 40, 10 | 30 | flat |
| Butcher (61) | 512 | 482, 452, 422, 392, 362 | 30 (36 once, while attacking) | flat (formula while attacking) |
| Morgan (36) | 256 | 226, 196, 166, 136, 106 | 30 | flat |
| Tyrog (4) / Tyrog (91) / Tyrog (72) | 256 / 300 / 350 | 226, 196, 166 / 270, 240, 210 / 226, 196 (its HP changed from 350 to 256 by itself at t = 258, when the form changed) | 30 | flat |
| Slisaur (71) | 350 | 320, 290, 260 | 30 | flat |
| Mack (38) | 128 | 92, 50, 10, 1 (36, 42, 40, floor) | 36 to 42 | formula (monster list) |
| Rock Hopper (29 variant 0x200) | 44 | 8 | 35 or 36 | formula |
| Zeke (82) | 20 | 1 (the blast; the explosion at t = 60 missed his height) | 30 or more, floor | monster list |
| Shivat (24), asleep | 60 | none | not hit (asleep) | |
| Vice, Slice, Hogg, Blade, Razor, Lash, class 64, class 79 | | not measured (open item 1) | | |

Sound codes: the strike has the explosion sound 0x9F12 per explosion and the screen shake; the bubble has no sound; the bazooka pickup has none (D4e to list).

### 2.6 The bazooka

R: 0x65A4 (port `item_give_bazooka`, DONE): a pool E class 0 (weapon) variant 0xC = **bazooka**, at the hero, held at once (`+$B4` of the hero, item sub state 8), **4 rockets** (HUD weapon display shows `4 BAZOOKA` with a full ammo bar, M: join_p2),
and **`+$A4 = 0x258`**: it vanishes 600 frames after it is dropped. Hannah keeps guns and Mess keeps the bazooka when hit; the other heroes drop it (R: 0x1177A; M: cont_level: Jack lost it on the first hit at t = 882, `held` 0000 at t = 958).
It is created before the air strike object (0x18BC8 before 0x18BCE).

### 2.7 Which respawn gets what

| event | `+$07` | bazooka | air strike | speech bubble (text) | invulnerability | HP | lives after | stage clock | source |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| respawn after losing a life (hit, pit, or TIME OVER) | 2 | **no** | yes | yes (by player count) | 61 + 360 frames | 100 | lives - 1 | +3:30 (cap 4:00), wave timer +600 | M: resp_*, timeover_life, timeover_2p |
| join in the level with a coin (strip, then pick) | 0 | **yes (4 rockets)** | yes | yes | same | 100 | extra lives setting (1) | +3:30 (D4d F29) | M: join_p2 |
| continue with a coin while others play (strip, then pick) | 0 | **yes** | yes | yes | same | 100 | 1 | +4:00 cap 4:00 | M: cont_level |
| continue with a coin when everybody was dead (mode 0x10, then strip) | 0 | **yes** | yes | yes ("They're gonna pay" for one player) | same | 100 | 1 | +4:00 cap 4:00 | M: cont_mode |
| start of a game or of an area | n/a | no | no | no | no | 100 at episode start | | players are placed by the area start code (0x4E0A), not by the respawn | R: 0xAC4E |
| respawn in the Cadillac section (`G.$518 != 0`) | 2 | no | no | no | `$EC = 360` still set, no 60 frame wait, no drop animation | 100 | lives - 1 | +3:00 and the drive ends at time over (D4b) | R: 0x18BB2, 0x18BF6, 0x18C0A (not run, open item 4) |

Join and continue take their numbers from D4d section 6 (strip of 10 numbers of 40 frames, the pick press, +1 point to every score).

### 2.8 Game over of one player while others play

D4d 6.3 (a). From D2b: the dead hero's object is switched off (`+$00 = 0`) after the 61 frame blink and 0x655E recounts the players; the level goes on for the others. M: cont_level (P1 lives 0, HP 0 at t = 20: death flight, blink,
the object disappears at about t = 244; the HUD block shows CONTINUE with a countdown; `held` of the continue is the bazooka object 0x99B4 after the pick).

## 3. Every invulnerability window (one timeline table)

"Hurt box" means the body box (`+$48`); enemy attacks need the hero's frame class in {0, 2, 4, 0xC, 0xE} and a body box and `+$00` bit 1 clear (R: SPEC 5.1; passes 0x115B0, 0x10DCC). Timers: `+$00` (2 = hit tests off), `+$F9` (post move
timer), `+$EC` (respawn and tag team timer).

| window | length (frames) | starts | sparkles or blink | what can still hurt the hero | source |
| --- | --- | --- | --- | --- | --- |
| hurt sequence (flinch) | the record's hit stop (16 for the thug's jab) | the hit frame | none (class 8 and `+$00 = 2`) | nothing; **no extra invulnerability** after a flinch (`+$00 = 1` at once) | M: dmg_jack_L0_hp100 (hit at t = 243, `+$00 = 1` at 259) |
| knockdown, lying, getting up | flight 39 + 19, lie `$E8 + 1` (5, 15 or 30 plus 1), get up 25 | the hit frame | none | nothing (`+$00 = 2` all the time) | M: lie_L0 (402: 39, 404: 19, 406: 6, 1E00: 25) |
| after the get up (reaction code 4 knockdown) | **20** (`$F9 = INV[0 + hero]` = 20, 20, 20, 20) | end of the get up or dizzy | none | nothing | M: lie_L0 (`$F9` 20 to 0, `+$00 = 2` from t = 450 to 470); R: 0x1C0DA |
| dizzy | up to 40 to 180 | end of the get up | stars effect (MISSING) | **the hero can be hit** (`+$00 = 1`) | M: dizzy5 (`+$00 = 1` during the dizzy) |
| throw | 12, 10, 14, 18 (Jack, Mustapha, Hannah, Mess) | end of the throw | none | nothing; the throw animation itself has no hurt box | R: 0x6C3A; D2a section 15 |
| pick up an item | 10, 8, 20, 10 | end of the pick up animation | none | nothing; the pick up animation has no hurt box | R: 0x6C3A |
| special (mega crush) | 60 after the move; the move itself is class 6 (immune) | end of the special | none | nothing | R: 0x6C3A; D2a section 6 |
| **respawn** (lost life, join, continue) | **61 wait + 360 counted from the drop** (421 from R, 329 after landing) | R | **sparkles every 4th frame** (pool C 23), speech bubble then player marker | **TIME OVER** (HP is set to 0 for every hero whose state byte is below 4, also an invulnerable one) and the `hp 0` poke. Nothing else. | M: resp_jack, cont_level, timeover_inv |
| tag team | 180 for both heroes (partner from the start of the tie up, initiator from the end of the swing) | D2a section 9 | sparkles | nothing | D2a |
| 1UP pose | the pose (no hurt box) then 180 | the pose | marker "1UP!" then sparkles | nothing | M: oneup |
| death blink | 61 | death state | the sprite flickers every frame | nothing | M: resp_jack |
| scripted control (`G.$4DD != 0`) | while it lasts | scene start | none | enemy attacks are switched off | R: 0x10546, T09 |
| frames without a hurt box | per animation | | | D2a section 15: drop in, special, throw, pick up, dash recovery and dive landing of Mustapha and Hannah (Mess: a point box), 1UP pose, tag team swing | D2a |
| dash slide of Jack | the slide has a very low body box (top at 35 px) | | | blows at head height pass over; low blows still hit | D2a |

What can hurt an invulnerable hero: the clock (TIME OVER sets HP 0 in 0x1046A for every hero with a state byte below 4) and nothing in the collision passes. Explosions of TNT and bombs of enemies, thrown items of other players (pass 0x10DCC,
**a stub in the port**) do not hurt while `+$00 = 2`.

## 4. Hurt, knockdown, get up, dizzy

All from R: SPEC 5.2 to 5.7 (0x1C02C and the state routines) and measured where marked. Port: `player.rs` DONE (hurt, knockdown, get up, dizzy states), the stars effect (0x94338) MISSING.

### 4.1 Light hit (reactions 0, 2, 0x1A)

State 0x02020000 for one frame, then 0x02020002 (anim set 2 id 0 or 1, sound 0x98CC) until the **hit stop counter** from the enemy's record runs out; then back to idle with `+$00 = 1`.
The thug's attack id 1 (damage 6, stun 1, reaction 0, hit stop 16): the hero is stuck for 16 frames. M: dmg_jack_L0_hp100 (0x02020000 at t = 243, idle at 259).
A light hit becomes a **knockdown** when HP is 0, **the stun meter is 0**, the airborne flag is set, or `$5A != 0`. Hits from behind use reaction 0x1A and turn the hero around at the end.

### 4.2 Stun meter

`+$78` starts at 10 (also at respawn, 0x18CB4). Each hit takes the record's stun value (jab 1). When the meter is 0 the next light hit is a knockdown. After a knockdown starts the meter is set to 10. The refill timer `+$F6 = 0x78` is set at each hit;
when it counts to 0 (120 quiet frames) the meter is set to 10 (R: 0x18DC6). M: dmg_jack_L0_hp100 (meter 10, 9, 8, 7 after three jabs 60 frames apart; `$F6` 120 counting down).
The hit counter `+$CA` counts hits taken (reset 60 frames after the last hit by `+$CB = 0x3C`); it feeds the dizzy roll (4.6).

### 4.3 Knockdown (reactions 4, 6, 0x1C, burn 0x0A and 0x0C)

Normal knockdown (reaction 4): flight up with `vy = 0x500`, `vx = 0x280` away from the hit (front), gravity -0x40 per tick; hit stop, then 39 + 19 frames of flight (two states 0x402 and 0x404), lying (0x406), get up (0x1E00, 25 frames).
The dead hero uses `vy = 0x780`, `vx = 0x300` and half speed motion (2.1). Heavy knockdown (6) uses `vx = 0x400`; knockdown from behind (0x1C) pushes the hero forward; burn knockdowns (0x0A, 0x0C) spawn two flame objects and keep `G.$7B2 = 2` while lying.
M: lie_L0: the flight states 39 and 19 frames, lying 6, get up 25, then 20 invulnerable frames (90 frames from the hit to control, 110 to the end of the invulnerability).

### 4.4 Lie time and mashing (and the level band bug)

Lie time `$E8` and mash meter `$BE` are rolled when the hero first touches the ground (R: 0x1D17E, two random numbers in this order). **The row of the table is `(level & 0x18) * 2` read with `move.b $5EE(a5)`, which reads the HIGH byte of the level word (always 0):
only row 0 is ever used, for every level and every rank** (port `player_level_band`, R: HANDOFF). Row 0:

| table | values (16 random cells) | meaning |
| --- | --- | --- |
| lie time 0x1D1C4 row 0 | 5 (6 cells), 15 (8 cells), 30 (2 cells) | the hero lies `$E8 + 1` frames: **6 frames 37.5 percent, 16 frames 50 percent, 31 frames 12.5 percent** |
| mash meter 0x1D204 row 0 | 15 (6 cells), 20 (8 cells), 25 (2 cells) | |

Mashing (R: 0x6012): each **new** stick press (edge): right alone or left alone counts **3**, any other direction 1 (table 0x605E: 1, 3, 3, 1, 1 ...), taken from the meter. The attack button counts 1 and then waits 6 frames. When the meter is 0 the hero gets up at once.
M: lie_nomash (lie time poked to 30: lying 31 frames), lie_mash (alternate left and right every frame from t = 420: the meter 25 ended the lie after 10 frames, at t = 429), lie_mashA (attack taps every 7 frames: no change, 31 frames).
M: lie_L0, lie_L8, lie_L16, lie_L24 all showed the row 0 values (lie time 5 or 15), never a longer row.

### 4.5 Get up

State 0x02021E00, anim set 2 id 3, **25 frames** (M: lie_L0). At its end (`recover`): reaction code 4 gives `+$00 = 2` and `$F9 = 20`; any other code gives `+$00 = 1` and no timer (R: 0x1C0DA). The power factor `$C6` is not rolled here (it is rolled when the hero is hit, 5.3).

### 4.6 Dizzy

R: 0x1D0FA (roll at the start of a knockdown), 0x1CE02 (start), 0x1CE88 (each frame). The roll is skipped (hit counter cleared) when the hero is in the tech roll state `+$CC`, **HP below 30**, the dizzy immunity `+$CE >> 8 != 0` (immunity at 256 or more), or a dizzy is already on.
It uses the hit counter `n = min(+$CA, 5)` **including the hit that knocks down**: mask table 0x1D166: `n` 0 to 3: never; **n = 4: mask 0xFFF0 (12 of 16 = 75 percent)**; **n = 5 or more: 0xFFFF (always)**. So a knockdown that is the 4th or later hit with gaps of at most 60 frames makes the hero dizzy.
M: dizzy4 (counter forced to 4: no dizzy in one run, consistent with 25 percent), dizzy5 (counter forced to 5: dizzy started, `+$CD = 1`), and a natural run with 3 hits (counter 3: no dizzy).
(`mechanics-seen.md` P16 and SPEC section 5.6 said "3rd hit always, 2nd 75 percent": wrong by two.)

When the dizzy starts (the get up ended): anim set 2 id 4, `+$00 = 1` (**the hero can be hit**), the **stars to mash away** `$BE` and the **maximum dizzy frames** `$80` are rolled from row 0 of two tables, and the immunity `+$CE = $80 + 0x708` (1800) starts counting down 1 per frame, so the next dizzy is possible when it is below 256 (`$80 + 1545` frames after the dizzy started, 1585 to 1725):

| table | values (cells) |
| --- | --- |
| stars to mash `$BE` (0x1CF5C row 0, 16 cells) | 15, 35, 35, 35, 50, 50, 50, 50, 65, 65, 65, 65, 80, 80, 80, 125 |
| max dizzy frames `$80` (0x1CEDC row 0, 16 words) | 40, 40, 50, 50, 50, 70, 70, 70, 90, 90, 90, 120, 120, 120, 150, 180 |

Each mash press (the same rule as 4.4) reduces `$BE`; each press also shortens the animation. The dizzy ends at `$BE = 0` or when `$80` has counted down; then `recover`: with the reaction code of the last knockdown `+$00 = 2`, `$F9 = 20`.
M: dizzy5: the get up ended at t = 470, `$80` = 70 and `$BE` = 50, `$CE` = 1870 at the start; the dizzy ended at t = 540 (70 frames) and 20 invulnerable frames followed (t = 540 to 559); a thug hit the dizzy hero again at t = 729.
Dizzy enemies: D3.

### 4.7 Mercy rule

R: 0x6422 end: if HP is not 1 and the damage is at least the HP and **the overkill (damage minus HP) is less than 5**, the damage becomes `HP - 1` (the hero survives with 1 HP). At HP 1 there is no mercy: the next hit kills. Port: DONE (`enemy_damage`).
The general low HP effects: a small random reduction from the damage (0 to 4, section 5.1 step 3) and the HP factor 25 to 33 (step 5). M: mercy_hp5 (HP 5 and a jab: HP 1), mercy_hp12 (12 to 8), mercy_hp30 (30 to 25).

### 4.8 Held by an enemy

SPEC 5.7 (mash meter `$BE = 40`, 10 anim ticks per press, escape attack 0x18, Mustapha reverses the hold). Not re-measured here.

## 5. The damage formula

### 5.1 Enemy to hero (R: 0x6422, called from 0x11682)

Inputs: `base` = byte 0 of the enemy attack record (table 0xA2EA4, 8 bytes per attack id: the entry is `0xA2EA4 + +$58 * 8` with the attacker's `+$58`, a plain id 0, 1, 2 ...); `type` = the enemy's `+$84` (name id times 4); `level` = `G.$5EE` (0 to 31); the hero's id and HP. All 16 bit, truncating.

1. `d = (base * L[type][level]) >> 5` with the table **0x2C3AE** (39 rows of 32 bytes, row = name id, column = level, 32 = 1.0). Selected values in 5.3.
2. `r` = a word from table 0x2C896: row by the hero's HP (**76 or more: row 0; 51 to 75: row 1; 26 to 50: row 2; 25 or less: row 3**), column `rand & 0x3E` (word index 0 to 31). Row 0 is all 0; row 1 has a 2 in the last cell; row 2 has 3 in the last two cells; row 3 has 4 in the last three cells.
   `d = d - r`; if the result is negative (not zero) `d = 1`.
3. `d = (d * H[hero id]) >> 5` with **0x2C88E = 32, 34, 35, 30 for Jack, Mustapha, Hannah, Mess** (section 5.2).
4. `d = (d * F[(hp >> 2) & 0x1E]) >> 5` with 0x2C996 (words by HP / 8): HP 0 to 15: 25; 16 to 23: 27; 24 to 31: 30; 32 to 39: 31; 40 to 87: 32; 88 and more: 33. If `d` is 0 it becomes 1.
5. Mercy (4.7).

Caution: the table index `(hp >> 2) & 0x1E` wraps at HP 128: values above 100 are a poke only.
The result is subtracted from HP (HP 0 is the floor), the hurt record applies (stun, reaction code, hit stop, sound, spark), then `$C6` is re-rolled (5.3).

### 5.2 Hero factor order settled

Table 0x2C88E (words 32, 34, 35, 30) is read as `rd16(0x2C88E + hero id * 2)`. The hero ids are 0 Jack, 1 Mustapha, 2 Hannah, 3 Mess (`+$03`, checked by the walk speed table in D2a and by the `char=` start of the probe, which put Hannah at id 2).
**So Mustapha takes x34/32 (1.0625), Hannah x35/32 (1.094), Mess x30/32 (0.94), Jack x1.** The enemy SPEC (Mustapha 34, Hannah 35) is right; the player SPEC text (Hannah 34, Mustapha 35) used the cursor name order. Proof (M: dmg_*_L31_hp100, level 31, hero HP 100):

| attack | Jack | Mustapha | Hannah | Mess | steps (thug base 8, level 31: row value 58) |
| --- | --- | --- | --- | --- | --- |
| thug attack id 0 (base 8) | 14 | 14 | **15** | 13 | 8 * 58 >> 5 = 14; hero: 14, 14 * 34 >> 5 = 14, 14 * 35 >> 5 = 15, 14 * 30 >> 5 = 13; HP factor 33: 14, 14, 15, 13 |
| thug attack id 1 (base 6), level 31 | 10 | 10 | 10 | 9 | 6 * 58 >> 5 = 10 |
| thug attack id 1 (base 6), level 0 | 6 | 6 | 6 | 5 | 6 * 30 >> 5 = 5 |
| thug attack id 0 (base 8), level 0 | 8 | 8 | 8 | 7 | |
| thug attack id 1 at HP 20, level 0 | 5 | 5 | 5 | 4 | HP factor 31 |

The port already uses `defense[hero id]` (`player_combat.rs` `enemy_damage`): correct. Add a unit test with these numbers (task P-D2b-5).

### 5.3 The tables

**Level by type (0x2C3AE)**, groups with equal rows, values at level 0, 4, 8, 12, 16, 20, 24, 28, 31 (the port reads the whole table from the ROM):

| name ids | 0 | 4 | 8 | 12 | 16 | 20 | 24 | 28 | 31 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 00 Ferris | 32 | 34 | 36 | 40 | 44 | 46 | 48 | 52 | 58 |
| 01 Gneiss, 03 Hammer, 08 Punk | 32 | 35 | 37 | 44 | 48 | 50 | 52 | 56 | 62 |
| 02 Poacher J | 32 | 44 | 48 | 52 | 56 | 60 | 64 | 72 | 72 |
| 04 R Hopper | 32 | 35 | 37 | 44 | 48 | 48 | 50 | 56 | 64 |
| 05 Slice, 0C Vice, 0D Walther, 0E Lash, 0F Butcher, 10 Bludge, 18 Slisaur, 19 Tyrog, 22 Fessenden, 23 Tyrog 2, 25 Fessenden | 32 | 36 | 40 | 44 | 49 | 57 | 65 | 73 | 79 |
| 06 Blade, 07 Shivat | 32 | 35 | 38 | 44 | 48 | 52 | 56 | 60 | 68 |
| 09 Thug, 0A Blk Elmer, 0B Wrench, 14 Razor | 32 | 38 | 40 | 48 | 52 | 56 | 60 | 64 | 72 |
| 11 Mack | 32 | 38 | 40 | 48 | 52 | 56 | 60 | 64 | 80 |
| 12 Skinner, 13 Gutter | 32 | 40 | 48 | 52 | 56 | 60 | 64 | 72 | 72 |
| 15 Morgan, 16 Morgue, 17 Knife, 24 Wrench (big) | 32 | 35 | 38 | 40 | 48 | 52 | 56 | 60 | 68 |
| 1A Driver, 1B Tyrog, 1C Blade, 1D Punk, 1E Hammer, 1F Elmer, 20 Wrench, 21 Zeke | 32 | 35 | 37 | 45 | 49 | 52 | 54 | 58 | 64 |
| 26 Hogg | 24 | 26 | 30 | 32 | 34 | 36 | 38 | 40 | 56 |

(R: dump of 0x2C3AE with `re/out/d2b/tables.py`.) The **level** is `G.$5EE`, computed from the rank and the player count (table 0x1860C; 1 player: rank 0 gives level 0, rank 24 and above gives 24, rank 31 gives 25;
2 players: 4 to 29; 3 players: 8 to 31). The rank rules are D4b's.

**Attack records of the enemies** (0xA2EA4, damage, stun, reaction, hit stop, sound, score byte, spark): 64 rows in `re/specs/enemy/damage_tables.txt` and `re/specs/player/tables.txt` (the thug: ids 0 = 8, 1 = 6, 2 = 10, 3 = jump kick 13 with knockdown, 12 = dash 12 (pushback); Zeke's dive 15; poacher and boss rows in D3).

### 5.4 Hero to enemy (R: 0x2B904) and the special path

The full order (the SPEC pseudo code is right; this is the checked list): `d = base`; **counter hit**: if the enemy is mid attack (frame class 4) and its attack id has a bonus in table 0xA320C, `d = d * m >> 5` (enemy `+$58` ids 3 (the thug's jump kick), 6, 20, 31, 44, 52, 86: x48/32; ids 12 (the thug's dash attack) and 16: x64/32 (double); id 46: x33/32; the table is one word per `+$58`);
**weak spot**: two (attack id, factor) pairs per enemy type in 0xA32FA, default **0x10 (running jump attack) and 0x0E (dive) at x48/32 (1.5)**; exceptions: ids 5, 15 (Slice, Butcher) dive x33; 21, 22, 24, 25, 34 (Morgan, Morgue, Slisaur, Tyrog, Fessenden) dive x48 but no running jump bonus;
12 Vice: attack id 0x5E x24 (0.75), 13 Walther: running jump x48 and 0x5E x24, 14 Lash: id 0x0A (throw) x64 (2.0), 2, 18, 19 (Poacher J, Skinner, Gutter): throw x48, 38 Hogg: id 0x00 (jab) x64 (**double damage from jabs**);
then `d = d * C6` (the hero's `$C6`), `d = d * RT[type][level]` (table **0x2BECE**, 32 = 1.0: Zeke **64 (x2.0)**, the big Wrench (24) 48 (x1.5), Hogg 48 down to 26, Slice, Vice, Walther, Butcher, Morgan, Morgue, Slisaur, Tyrog, Fessenden fall with the level from 32 to 21 to 25),
`d >>= 10` (minimum 1); HP row factor for enemies above 100 HP (33/32 for 200 to 299, 32 otherwise in the first rows); HP band factor 0x2BB4E by `HP % 100` (**35 or 34 at 75 or more; 33 or 32 or 31 at 50 to 74; 30 to 27 at 25 to 49; 28 to 26 below 25**, the first 16 cells high, the last 16 low, one cell 16 at the very bottom); `>> 10` with a 16 bit truncation (minimum 1).
`$C6` (R: 0x11738): rolled at the start and **each time the hero is hit**, from table 0x2C9B6 by the hero's HP: HP 75 or more: 32 (27 of 32 cells), 24, 28, 36, 42, 48; 50 to 74: 32 (25 of 32); 25 to 49: 32 (22 of 32), 36, 42, 48; below 25: 32 (22 of 32), 36, 42, 48 and once 64.
The **special's HP cost** (10, 4, 4, never kills, D2a section 6) is applied by the hit pass when a class 6 frame hits an enemy (0x10B08), not by the formula. D2a section 14 lists the base damage of every move.

**Worked example per hero** (hero to enemy, finisher id 0x04, thug at full HP 100, level 0, `$C6 = 32`; the same steps as the D2a measurements): `d1 = base * 32 = 928` (Jack, Mustapha, Hannah base 29), x rank 32, `>> 10` = 29;
HP row 32: 29; band 35 (cells 0 to 15) or 34: 29 * 35 = 1015 >> 5 (the code is 16 bit: `d * 35 >> 10`) = **31** or 30. Mess (base 32): 32, band: **35 or 34**. So Jack 30 to 31, Mustapha 30 to 31, Hannah 30 to 31, Mess 34 to 35 (D2a measured 30-31 and 34-35).
Hero to enemy with the air strike: **always 30**, no factors (2.5). Enemy to hero worked examples: 5.2.

## 6. Scoring

Add routine R: 0xFFC8 (BCD add of 4 bytes into `+$100` of the credited player, **cap 99,999,999**, sets the HUD score dirty flags); **no points are added while scripted control is on with `G.$4DE == 1`**. Kill scores and tally: D4d, SPEC 3.4.

| source | points | when | source |
| --- | --- | --- | --- |
| each hero hit on an enemy or container | the record's score code: **100** (jab, knee 1, jump attacks, dive, running jump attack, dash hit 3), **200** (punch 3, knee 2, dash start and hit 2), **400** (finisher, throw, special for Jack, Mustapha, Mess), **700** (the big hit of each command move), **800** (escape from a hold), **1000** (tag team run attack), 100 for Hannah's special; the code to points table is 0x14 = 100, 0x18 = 200, 0x1C = 300, 0x20 = 400, 0x2C = 700, 0x30 = 800, 0x38 = 1000 (D2a section 14 says which move uses which) | at the hit frame | R: 0x10292 table, D2a section 14; M: score_kill (+100, +100, +200 for jab, jab, punch 3) |
| kill | by enemy name: 1000 (Ferris, R Hopper, Shivat, Punk, Lash, Mack, Knife, Driver, Hogg), 2000 (Gneiss, Poacher J, Hammer, Blade, Thug), 3000 (Blk Elmer, Skinner, Razor), 4000 (Wrench, Gutter), 8000 (Walther, Bludge), 10000 (Vice), 20000 (Butcher, Morgan), 30000 (Morgue), 40000 (Slice), 60000 (Tyrog 0x19), 70000 (Slisaur), 80000 (big Wrench 0x24), 100000 (Tyrog 0x1B); Zeke and the Fessenden and Tyrog 2 names 0x1C to 0x23 give 1000, Fessenden 0x25 gives 8000 | **when the dead enemy finishes dying** (about 79 frames after the killing blow in the probe) | M: score_kill (+1000 at t = 115, killing blow at t = 36); R: SPEC 3.4 |
| container broken | **+500** to the player credited by `+$02` (plus 100 per hit by the record) | at the break frame, 2 frames after the hit | M: score_container (+100 at the jab, +500 two frames later) |
| food at full HP | BARBECUE, STEAK, LOBSTER, SUSHI **10000**; BURGER, HOTDOG, PIZZA, SALAD **5000**; CAKE, PAFFE, PUDDING, POTATO **3000**; GUM, CHOCOLATE, DONUT, COFFEE, CROISSANT **1000**; type 18 **4000** | on the pick up press (the same frame) | M: pick_food1_full (+10000), pick_food9_full (+3000); R: SPEC 13.4 |
| food when hurt | heals 100 / 80 / 64 / 48 / 32 / 16 / 36 HP (cap 100), **no points**; the HUD says `GOOD !!` | pick up press | M: pick_food1_hurt (40 to 100), pick_food18_hurt (40 to 76) |
| treasure | GOLD, DIAMOND, THE CUP, EGG **10000**; GOLDDUST, DINOBONE, AMMONITE, PEARL **5000**; NECKLACE, RING, WATCH, RADIO **3000**; DIME, 1000$, SUNGLASS, GROVE **1000** | **11 frames after the press** (the pick up animation) | M: pick_treas1 (+10000), pick_treas9 (+3000), pick_treas16 (+1000) |
| ammo box | +1000 if the gun is already full | | R: SPEC 13.4 |
| air strike | +100 per enemy hit, +500 per container, to the respawning player | | M: strike_score |
| join, continue | +1 to the score of all three players (D4d F29) | | D4d |
| stage clear tally | (HP - 1) * 100 or 20000 at HP 100, kills * 100, final blow 20000, M.V.P. 10000 (D4d 9.1) | | D4d |

**Score display.** Only in the HUD (8 digits, right aligned in the player block, with the ranking place text under it, 1.2). **There are no floating score numbers on the playfield**: kills, hits and pick ups show nothing at the hero.
Pick ups show a message in the HUD message area of the block: a FOOD icon with the item name and `10000 pts` (full HP) or `GOOD !!` (hurt); treasure shows `NNNNN pts` under the last enemy name icon. M: pick_* (pictures); no number appeared in oneup, strike_score, pick_* sheets.
**High score:** the ranking table and the name entry exist (D4d section 7, correcting F20); the HUD shows no top score.
**Rank and level effects:** the kill and hit scores do not depend on rank or level. Level scales the enemy damage to heroes (5.1), the hero damage to some bosses (5.4) and enemy HP (D3).

## 7. Time and the player

R: 0x103FE (called every frame), 0x1046A, 0x10482 (adds to the clock after a lost life), 0x96450 (TIME OVER words, D4b). M: timeover_life, timeover_nolife, timeover_2p, timeover_inv.

- Trigger: the clock word G.$4E8 is 0 and none of G.$4D5 (HUD hold), G.$4DD (scripted), G.$4EB (already running), G.$4EA is set. The clock reached 0:00 at t = 153 after `time 0:03` (3 seconds = 153 frames).
- In the **car section** (`G.$518` and `G.$7B4` set) the clock does not kill: `G.$518 = 2` (the drive ends) and 3:00 is added (D4b, rank.rs DONE).
- Otherwise: `G.$4EB = 1`, the TIME OVER words (0x96450; in the probe HP became 0 about 48 frames after the clock showed 0:00, t = 153 to 201, the same 48 frames in timeover_inv), and then **every hero whose state byte is below 4 gets HP 0** (`clr.w $6C`), which includes heroes that are **invulnerable**, mid respawn or in the drop animation
  (M: timeover_inv: the clock ran out 150 frames after `time 0:02`, the hero in his respawn invulnerability fell into the death flight at t = 479 with `+$00 = 2`).
  Heroes that are already dead (state byte 4 and higher) are skipped.
- Each killed hero goes through the normal death (2.1) and the lives check (2.2): with a life he respawns **without a bazooka** with the air strike; the clock gets +3:30 (cap 4:00) once for each respawning hero (M: timeover_life 0:00 to 3:30; timeover_2p: only the hero with a life added time).
  A hero without a life goes to the continue countdown (M: timeover_nolife: the continue mode 0x10 started at t = 405). Rank drops by 4 (or 3) for each death as in 2.2.
- After the respawn `G.$4EB` is cleared by 0x10482 (the clock is no longer 0), so the next TIME OVER can come when the clock runs out again.
- The TIME OVER words and the clock display rules are D4b; the sub area switch resets the stage time (D4c T12).

## 8. Probe batches used (all headless, `probe.sh`, one MAME process each)

| batch | file | segments |
| --- | --- | --- |
| d2b-b1 | `batch1.txt` | resp_jack, resp_mustapha, resp_hannah, resp_mess, resp_2p, resp_3p, join_p2, cont_level, cont_mode, timeover_life, timeover_nolife, timeover_2p, oneup, lives_hud, lives9_hud, hp_bar, start3p |
| d2b-b2 | `batch2.txt` | ty_* (29 enemy types, one explosion pattern each), box_x, box_z, blast_reach, hp_floor (the `objset A#n` lines failed: `#` starts a comment), objs_ex, objs_tnt, strike_thugs |
| d2b-b2b | `batch2b.txt` | hp_floor2, objs_center, objs_blast, strike_dino, strike_score, t2_* |
| d2b-b3 | `batch3.txt` | dmg_* (4 heroes at level 0 hp 100, level 31 hp 100, level 0 hp 40, hp 20), lie_L0, lie_L8, lie_L16, lie_L24, lie_mash_L24, dizzy_L0, dizzy_L24, mercy_hp5, mercy_hp12, mercy_hp30, timeover_inv, score_kill, score_kill_gneiss, pick_*, score_container |
| d2b-b3b | `batch3b.txt` | dizzy5, dizzy4, lie_nomash, lie_mash, lie_mashA |
| d2b-b4, b5, b6, b7, ag | `batch4.txt`, `batch5.txt`, `batch6.txt`, `batch7.txt`, `ag.txt` | hp_carry, rank_death, camy (camera X and Y of the start spots), realdeath_1p, realdeath_2p, poke_1p_again, poke_2p_again, aggression test (`aggr 7` makes the thug attack; with aggression 0 it stood 143 pixels away for 570 frames) |

Probe notes that matter for repeating these: enemies of the jump stay at aggression 0 and do not attack: use `aggr 7` for damage tests; `#` starts a comment, so object selectors with a slot (`A#19`) cannot be used in `objset`; the slots of the first spawns after `jump 1 0 897` are 19, 15, 14, 10, 9, 8, 7 ..., 0 (14 slots);
pool E explosions spawned by `spawn E 1 var=2` use player 1 as owner; `spawn A` of classes 3, 14, 31, 64, 79 and class 20 variant 1 did not initialise in the first 200 frames (class 20 variant 1 stopped the game frame counter).

## 9. (a) Acceptance scenarios

All input files are in `re/out/d2b/` (`batch1.txt`, `batch2.txt`, `batch2b.txt`, `batch3.txt`, `batch3b.txt`, `batch4.txt`, `batch5.txt`, generators `gen*.py`); run `PR_INPUT=... re/harness/probe/probe.sh NAME`. The port is driven with the harness `run` mode and compared state by state.
`R` = respawn start gf.

| # | scenario | probe lines (summary) | must match |
| --- | --- | --- | --- |
| S1 | lost life respawn, one player, 4 heroes | `jump 1 0 897 char=H`; `lives 2`; `10 hp 0`; watch P1, B:18, E:8, E:1, C:14 | HP 0 at t = 10; hurt 0x02020000 at 11, air 0x402 at 12, 0x404 at 131, 0x406 at 188, death 0x04000000 at 194 (Mustapha 196), blink 61 frames, respawn at R (Jack gf 1192); parked at X 1089 Y 384 Z 256 for 60 frames; drop at R + 61 (X 993 Y 256 Z 320, `vy -256`); idle at R + 92; `rinv` 360 at R + 1 and from R + 62 down 1 per frame; `+$00 = 1` at R + 421; no held item; lives 2 to 1; HP 100 |
| S2 | air strike timeline and positions | same run, `watch B:18, E:8, E:1, E:11, C:7` | bombs spawn at R, R + 12 ... R + 48 at start X 993 Y 288 (target Y 0), landing X 1179, 1059, 1000, 1000, 1119 and Z 352, 320, 336, 288, 304; explosions made at R + 59, 71, 83, 95, 107; 5th bomb has variant 1 and makes the class 11 blast the same frame; 26 frame explosion animation |
| S3 | strike on enemies (flat 30, floor 1) | `strike_thugs`: 14 pinned thugs HP 44 over X 920 to 1284 (hold X and Z), `10 hp 0` | every thug 44 to 14 once, none dead, within frames 330 to 373 of the run; the thugs stay down about 100 frames; score `strike_score`: +300 per three thugs, +500 per container |
| S4 | HP floor and flat damage | `hp_floor2`: HP 1, 5, 20, 29, 30, 31, 35, 45, 60, 99, 100 under 3 explosions and a blast | HP after the first explosion (t = 60): 1, 1, 1, 1, 1, 1, 5, 15, 30, 69, 70; after the second (t = 180): 1 for all up to HP 60, then 39 and 40; after the third (t = 300): 9 and 10; after the blast (t = 420): 1 and 1. Nobody dies. |
| S5 | hit area | `box_x`, `box_z`, `blast_reach` (one explosion at X 1089 Z 296; thugs 20 px apart) | hits at X 1009 to 1169 and Z 276 to 316 for the explosion; every thug for the blast (X -189 to +186, Z +-200) |
| S6 | bosses and monsters | `ty_*` (one enemy pinned, explosion at t = 60, blast 160, explosions 260, 360, blast 460) | Butcher 512, 482, 452, 422, 392, 362 (30 each when idle); Morgan 256, 226, 196 ...; Walther 100, 70 (blast missed while down), 40, 10; thug 44, 14, 1, 1; Hammer 64, 34, 4, 1; Mack 128, 96 or 92 (formula 32 to 42 per hit), floor 1 |
| S7 | items and containers | `objs_ex`, `objs_center`, `objs_blast`, `objs_tnt` | containers (var 0 to 0xC) freed within 2 frames; weapon, ammo, food, treasure untouched; TNT HP 50 to 20 then explodes about 110 frames later |
| S8 | players not hurt | `resp_2p`, `resp_3p`: other players stand in the strike | HP of the others stays 100, no hurt state |
| S9 | join with a coin | `jump 1 1 char=jack`; `10-17:2C`, `60-65:2S`, `140-145:2A` | P2 state 0x00020400 at t = 141 with `held` = the bazooka object, X 192 Y 384; strike from t = 141; drop at +61 (t = 202, X 113); bubble "Need a hand?" (2 players); `HUD 4 BAZOOKA`; lives 1; `rinv` counts from the drop |
| S10 | continue with a coin | `cont_level`, `cont_mode`: lives 0 and HP 0, coin 300, start 340, attack 460 | bazooka held after the pick (t = 461); the strike; first enemy hit exactly 360 frames after the drop (t = 882); the stage clock 4:00 |
| S11 | bubble text | S1 (1 player) and `resp_2p` | "They're gonna pay for this!" for one player, "Need a hand?" for 2 or 3 players; the marker `1P` from R + 300 to R + 420 |
| S12 | time over while invulnerable | `timeover_inv`: respawn, then `330 time 0:02` | the hero dies at about t = 479 although `+$00 = 2`; respawn without bazooka; the clock +3:30 once per respawning hero (`timeover_2p`) |
| S13 | damage taken per hero | `dmg_H_L31_hp100`, `dmg_H_L0_hp100` (`aggr 7`, a thug next to the hero, `hp 100` every 30 frames) | jab base 8 at level 31: 14, 14, 15, 13 for Jack, Mustapha, Hannah, Mess; base 6 level 31: 10, 10, 10, 9; level 0: 6, 6, 6, 5 and 8, 8, 8, 7; HP 20: 5, 5, 5, 4 |
| S14 | hurt sequence | `dmg_jack_L0_hp100` | flinch 16 frames (hit at t = 243, idle at 259), stun meter 10, 9, 8, 7, hit counter 1, 2, 3, `$CB` 60; knockdown at the third hit: 39, 19, 6, 25 frames, then 20 invulnerable frames |
| S15 | lie and mash | `lie_nomash`, `lie_mash`, `lie_mashA` (lie time poked to 30 and meter to 25 at lie start) | 31 frames; 10 frames with alternating left and right; 31 frames with attack taps |
| S16 | dizzy | `dizzy5`, `dizzy4` (`+$CA` held at 5 and 4) | dizzy with 5; 4 may fail (75 percent); dizzy start values `$80` and `$BE` from row 0 of the tables; `+$00 = 1` during it |
| S17 | 1UP | `oneup`: score 499950, three jabs | lives 1 to 2 at the crossing frame, `+$10E = 1`, `+$11C = 240`; pose state at t = 23 (Jack), marker `1UP!` from the flagged frame for 180 frames, sound 0x4A, then `rinv` 180 |
| S18 | lives and HP HUD | `lives_hud`, `lives9_hud`, `hp_bar` | `=B` for 11 spare lives, `=8` for 8; name rows red at HP 15 or less in 6 frame phases |
| S19 | HP carry | `hp_carry` | HP 30 becomes 100 at the next episode start; lives and score kept |
| S20 | scores | `score_kill`, `pick_food*`, `pick_treas*`, `score_container` | +100, +100, +200, kill +1000 (79 frames after the blow); food at full HP +10000 or +3000; hurt: heal only; treasure +10000, +3000, +1000 11 frames after the press; container +100 then +500 |
| S21 | Time over 1 and 2 players | `timeover_life`, `timeover_nolife`, `timeover_2p` | clock 0:00 to 3:30 at the respawn; without a life continue mode starts at t = 405 |

## 10. (b) Port status and the port task list

Checked against `player.rs`, `player_combat.rs`, `hud.rs`, `item/blast.rs`, `enemy_combat.rs`, `flow/join.rs`.

| feature | status | file | note |
| --- | --- | --- | --- |
| start HP and lives, 0x4D8E, 0x4DD6, HP refill at episode start | DONE (lives from `Settings` via `flow_lives_init`, factory 1) | `player.rs` `player_join`, `flow.rs` | the settings need a config file (D4d 4.3) |
| life counter and HP bar HUD | DONE | `hud.rs` | |
| low HP blink | DONE | `hud.rs` | |
| extra life score hook 0x6B98 | DONE | `hud.rs` `hud_extend_check` | sound 0x4A routed (`snd_extra_life`) |
| 1UP pose and "1UP!" marker | MISSING | `player.rs` (TODO in `player_idle`), no pool C 14 | tasks P-D2a-3, P-D2b-2 |
| death state 0x1BF42 | DONE | `player.rs` `player_dead` | checksum call ignored on purpose |
| lives check 0x1BFD0 | PARTIAL | `player_dead_continue` | missing: rank event 0x184E2, calm period, clock +3:30 (0x10482), rank speed |
| respawn steps 0x18B38 (park, wait, drop, `$EC`) | DONE | `player.rs` | `respawn_t` counts only in the alive state as the original |
| bazooka for `+$07 == 0` | DONE | `item.rs` `item_give_bazooka` | |
| sparkles every 4th frame (pool C 23, 0x958E4) | MISSING | | comment in `player_alive` |
| speech bubble and "1P/2P/3P" marker (pool C 14 variant 0) | MISSING | | |
| air strike controller (pool B 18, 0x7B65E) | **MISSING** | none | the rest of the strike exists: pool E 8, 1 kind 2, 11 in `item/blast.rs`, pass 0x10F96 nonlethal path in `enemy_combat.rs`, container pass 0x120D2 in `item/hits.rs` |
| pass 0x1132A (monsters on list 0x6B64, formula path with the owner's `$C6`) | MISSING (stub) | `enemy_combat.rs` | D3b P1 |
| pass 0x10DCC (attack objects against heroes; skips ids 0x6E, 0x6C) | MISSING (stub) | `enemy_combat.rs` | needed for TNT and thrown items; D2c, D4a |
| hurt, knockdown, get up, mash, mercy | DONE | `player.rs`, `player_combat.rs` | |
| dizzy roll, dizzy state | DONE (tables read from the ROM) | `player.rs` | stars effect 0x94338 MISSING (D3f); the level band row is 0 as in the original |
| damage formulas | DONE | `player_combat.rs` | add tests with the measured numbers (5.2) |
| scoring (hit, kill, container, food, treasure) | DONE | `player_combat.rs`, `item/*` | check the HUD pick up message areas (message text `GOOD !!`) in `hud.rs` |
| tally, join and continue flow | STUB | `flow/join.rs` | D4d |
| TIME OVER kill | DONE (HP 0 for state byte < 4) | `rank.rs` | words MISSING (D4b) |
| car respawn variant (`G.$518`) | PARTIAL | `player.rs` (checks present) | untested |

Port task list (sizes: S under 1 hour, M 1 to 2 hours, L over 2 hours):

| task | scope | size |
| --- | --- | --- |
| P-D2b-1 | `player_dead_continue`: rank event 0x184E2 (drop 4 or 3, `w606 = 600`, calm 1200, rank speed table 0x187F4), 0x1842C for a hero without lives, stage clock +3:30 and wave timer +600 (0x10482), `G.$4EB` clear; scenarios S1, S12, S21 | S |
| P-D2b-2 | pool C class 14 variant 0 (bubble, phase 1 and 2) and variant 2 (the "1UP!" marker), pool C class 23 sparkles every 4th frame of `$EC`, with the asset extraction of the bubble pictures; S9, S11, S17 | M |
| P-D2b-3 | pool B class 18 controller with the 5 record list, the bomb start values (x0 = camera X + 0x40 after the terrain push 0x12CAC, ground + 0x120), `vx` division by 57, depth per record; the call from the respawn (0x18C5E); hooks for the shake and sounds; S2, S3, S5 to S8 | M |
| P-D2b-4 | passes 0x1132A (formula variant with the owner's `$C6`) and 0x10DCC (attack objects against heroes, skip ids 0x6E and 0x6C), together with the monster port (D3b); S6 | M |
| P-D2b-5 | unit tests: hero factor order (5.2), mercy, `$C6` table, lie and dizzy tables row 0, extra life thresholds (absolute BCD lists, one life per step), score cap | S |
| P-D2b-6 | MAME acceptance runs S1 to S21 against the port (harness `run` mode) and fix what differs | M |
| P-D2b-7 | make the extra lives and extra life option of `Settings` readable from a config file (D4d 4.3), no code change in `player_join` | S |

## 11. (c) Open items

1. **Vice, Slice, Hogg, Blade, Razor, Lash, class 64 and 79 variants** were not hit in my spawns (HP 0 or the intro state): measure the strike against them inside their real scripts (for example in Episode 1 area 2 with a respawn
   during the Vice fight). The pass is the same, so expect 30 per hit with floor 1 (or the formula path in states where they register on list 0x6B64).
2. Which enemy states register on the hostile list 0x6B64 (Mack, Hopper, Zeke, the Butcher while it attacks: 36 damage) decides the damage path; D3b and D3a must list it per class.
3. The owner's `$C6` was 42 in all runs; check the formula path damage with `$C6` 24, 36, 48 and 64.
4. Respawn in the Cadillac section (no wait, no strike, no bubble) was read in the code but not run; run a death during the drive (recipe `car-radio`).
5. A real killing blow by an enemy (hit stop of the record, then the same flight) was not run; the `hp 0` poke goes through the same code. (Done in part, see 2.1.)
6. Dizzy: only one sample of the dizzy length (70 frames) and the star mash numbers (the mash rule is read, not run with a dizzy hero).
7. The bombs' start X is `camera X + 0x40` followed by the terrain push (0x12CAC): measured 993 at camera X 897 in Episode 1 area 0 (961 plus 32) and 113 at camera X 0 in area 1 (64 plus 49). The hero's drop depth is `camera Y - floor` pushed to a walkable depth plus 32, 48 or 64 by player index (320 in area 0, 64 in area 1 for player 2): the push rule (which depth and X it moves to) is not decoded; the port should call its own terrain routine at the same point and be compared in a third area.
8. (Resolved) The 12 frame longer death with 2 or 3 players was the random lie time (16 instead of 6 frames), see 2.1.
9. Rank dropped by 3 at the first death of the probe run (13 to 10) while the table says 4 at difficulty 3: the 600 frame "recent drop" timer `G.$606` may already be running after a join or a start; verify with D4b.
10. Sound codes of the air strike (bomb, explosion with pan by X, white blast), of the bubble and of the pick up of the bazooka: D4e.
11. The hero face icon on hits and the low HP face (`face_icon`) were not measured.
12. The HUD number tile for 16 or more spare lives (letters G and up); only 0 to 11 seen.
13. The 1UP marker disappears when the hero leaves a ground state (read, not measured with a jump).
14. Whether pass 0x10DCC hurts heroes with a TNT explosion while `+$00 = 2` (it should not): D4a with the TNT port.

## 12. (d) New rows for mechanics-seen.md

New IDs continue after P60, F41 and W38.

| ID | Mechanic | Source | Port | Notes for the design agent |
| --- | --- | --- | --- | --- |
| P61 | Lost life respawn timeline: death flight 183 frames (dead vectors, half speed), 61 frame blink, respawn start R, 60 frames parked above the screen, drop at R + 61 (31 frames fall), control at R + 92; HP 100, lives - 1, stage clock + 3:30, wave timer + 600 | M: resp_jack/mustapha/hannah/mess; R: 0x18B38, 0x1BF42, 0x10482 | PARTIAL | D2b 2.1 to 2.3; corrects P21 |
| P62 | Respawn air strike: 5 bombs 12 frames apart in a fixed screen pattern (not aimed), explosions 59 frames after each bomb, 26 frame explosions, white blast of the 5th bomb hits the whole screen; every hit on an enemy is a flat 30 HP with an HP floor of 1 (never kills), once per enemy in practice, +100 points each to the respawning player; containers break (+500), TNT ignites, items and players are untouched, pool B breakables not hit | M: resp_*, strike_thugs, ty_*, hp_floor2, objs_*; R: 0x7B65E, 0x29EC4, 0x27F70, 0x2A534, 0x111C6 | PARTIAL (pool E side and enemy pass DONE, pool B 18 MISSING) | D2b 2.5; corrects W30 and P40 |
| P63 | Bazooka (4 rockets, vanishes 600 frames after a drop) only for respawns with `+$07 == 0`: join with a coin and continue with a coin; they also get the strike, the bubble and the invulnerability; lost life respawns get no bazooka | M: join_p2, cont_level, cont_mode against resp_*; R: 0x18BC0, 0x65A4 | DONE (bazooka), MISSING (rest) | D2b 2.6, 2.7; settles P58 |
| P64 | Respawn speech bubble: pool C class 14, text "They're gonna pay for this!" for one player and "Need a hand?" for two or three, 239 frames, then a "1P"/"2P"/"3P" marker for 120 frames, follows the hero, freed when he is no longer invulnerable | M: resp_jack, resp_2p, join_p2; R: 0x93B28 | MISSING | D2b 2.4; corrects P22 |
| P65 | Respawn invulnerability is `+$00 = 2` with `$EC = 360` counted only after the 61 frame wait: 421 frames from R; shown by sparkles (pool C 23 every 4th frame), not by sprite blinking; TIME OVER still kills the hero | M: resp_jack, cont_level, timeover_inv; R: 0x18CD4 | PARTIAL | D2b 3; corrects P19 |
| P66 | Each death: rank minus 4 (3 if a drop was less than 600 frames ago), calm period 1200 frames (aggression 0), rank speed -4 (1 player), -3 (2 players), -3 or -2 (3 players), player count recount at game over (level column changes) | R: 0x184E2, 0x655E; M: rank_death | MISSING | D2b 2.2; D4b to check |
| P67 | Lie time, mash meter and dizzy tables use row 0 only (level band byte read bug): lie 6 (37.5 percent), 16 (50 percent) or 31 (12.5 percent) frames, mash meter 15, 20 or 25 (right or left press counts 3), no lie time by level | R: 0x1D17E, HANDOFF; M: lie_* | DONE | D2b 4.4; corrects P15 |
| P68 | Dizzy needs the knockdown to be the 4th or later hit of a chain (gaps of 60 frames or less): 75 percent at 4, 100 percent at 5 or more, HP 30 or more, immunity `$80 + 1800` frames; dizzy lasts 40 to 180 frames, 15 to 125 stars to mash, the hero can be hit while dizzy | R: 0x1D0FA, table 0x1D166; M: dizzy4, dizzy5 | DONE (stars effect MISSING) | D2b 4.6; corrects P16 |
| P69 | Hero damage factor table 0x2C88E by hero id: Jack 32, Mustapha 34, Hannah 35, Mess 30; enemy to hero order of steps: type by level, low HP random cut, hero factor, HP factor 25 to 33, mercy; measured 14, 14, 15, 13 for a base 8 jab at level 31 | R: 0x6422; M: dmg_*_L31_hp100 | DONE | D2b 5; settles P18 |
| P70 | Extra life lists are absolute scores: 300,000 and 700,000, **500,000 and 1,000,000 (factory)**, or 1,000,000; one life per step; sound 0x4A; the "1UP!" text (pool C 14 variant 2, 180 frames) appears at the flagged frame of the pose | R: 0x6B98, 0x6C0A, 0x6C16, 0x6C22, 0x93BF8; M: oneup | PARTIAL | D2b 1.3, 1.4; replaces the thresholds note of P45 |
| P71 | HP is refilled to 100 at the start of every episode (not between areas); lives and score carry over | R: 0xAC4E; M: hp_carry | DONE | D2b 1.1 |
| P72 | There are no floating score numbers: kills, hits and pick ups only change the HUD score; pick ups show a message in the HUD message area (FOOD icon, name, `10000 pts` at full HP or `GOOD !!` when healed); the rank text under the score shows the place in the ranking table | M: pick_*, oneup | PARTIAL | D2b 6; answers the "score pop-ups" question |
| P73 | Scoring numbers: hit points from the record code, kill by name table, container +500 (plus the hit), treasure arrives 11 frames after the press, kill score about 79 frames after the killing blow; no score while scripted control with `G.$4DE == 1` | M: score_*, pick_*; R: 0xFFC8 | DONE | D2b 6 |
| P74 | TIME OVER sets HP 0 for every hero with a state byte below 4, also invulnerable ones; the death flow and the respawn follow (lost life rules, no bazooka) | M: timeover_*; R: 0x103FE, 0x1046A | DONE (words MISSING) | D2b 7 |
| F42 | The life counter in the HUD block is one tile whose code is the spare life count (digits, then letters from 10: A, B, ...), redrawn only by block redraws (respawn, extra life, area start) | M: lives_hud, lives9_hud | DONE | D2b 1.2 |
| F43 | At HP 15 or less the name, lives and score rows of the player block change palette every 6 frames (red and white); the bar does not blink | M: hp_bar; R: 0x1667E | DONE | D2b 1.2; refines P25 |
| W39 | Air strike side effects: containers break (+500), TNT barrels ignite (HP 50 to 20, fuse, explosion about 110 frames later), items and pool B breakables untouched | M: objs_ex, objs_blast, objs_tnt; R: 0x120D2, 0x10CD0 | PARTIAL | D2b 2.5 |
| W40 | Every explosion of kind 2 shakes the screen with strength 3; the bomb trail is pool C class 7 every second frame | R: 0x27FCE, 0x29F7C | PARTIAL | D2b 2.5; D3f for the effect classes |

Corrections to existing rows and notes: D4d 6.2 step 5 and P58 ("drops in at camera X + 0xC0": the hero is parked at camera X + 0xC0 above the screen for 60 frames and then drops at camera X + 96, 64 or 128, 48, 96 or 144), P12 (friendly fire was already corrected by P44), P15 (lie time by level: row 0 only), P16 (dizzy rule), P18 and `re/specs/player/SPEC.md` 5.1 and 7 (hero factor order), P19 (no blink, 421 frames), P21 (death timings), P22 and P40 (bubble, sparkles, air strike, 1UP marker),
P23 (thresholds 500,000 and 1,000,000), P25 (blink is a palette change of the text rows), P58 (all coin respawns also get the strike; the lost life respawn gets the strike but no bazooka), W30 (answered).
