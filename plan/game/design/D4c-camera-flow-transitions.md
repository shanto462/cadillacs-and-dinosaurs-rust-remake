# D4c: camera, bounds, scene flow and transitions (all 8 episodes)

Written 2026-10-12 by the D4c research agent. Nothing here is ROM data: only descriptions, numbers and addresses.

Source tags: **R:** ROM address or repo file, **M:** MAME probe (the segment name is given; the scripts are in `re/out/d4c/`,
the logs in `re/out/probe/d4c-b1` to `d4c-b4`, all git-ignored), **W:** web. MAME wins over the web. `[I]` marks an inference that
no run or listing proves. `G.$xxxx` is RAM `0xFF8000 + xxxx`. "Episode N" is stage N - 1 (`G.$4D9`); "area" is the camera area
`G.$786`. Positions are map pixels: X right, Z depth (a larger Z is higher on the screen), Y height.

**Time units (read this first).** The probe logs two clocks. `t` counts video frames after the jump. `gf` is the game frame
counter `G.$0000`, which is the clock of the port (one `Game::tick()` is one original game task run). They agree most of the
time, but in some stretches (area loads) the probe logged 2 to 4 game frames between two video frames (probably video frames that
MAME skipped while running unthrottled [I]), and the story panel (mode 8) and the boss slow motion (task period 4) stop or slow `gf`. **All durations below are in game frames (gf) unless the text says "video
frames"**; fades and the story panel are vblank based and are given in video frames. Example of the difference: Episode 4 area 1
start script, 66 video frames but 93 game frames [M d4c-b2/e4a0n].

How the numbers were made: 4 MAME batches of 72, 87, 14 and 11 probe segments (one MAME process each), a
reading of the scripted control code (0x1D95E to 0x1E9F0, 0xB9BE, 0xAB2C, 0x7B336, 0x7BB68, 0x7BEF6, 0x22398) and of the port
(`camera.rs`, `spawn.rs`, `flow.rs`, `prop.rs`, `player.rs`).

## 0. Summary of what is new or different from the plan

1. **The plan was wrong about Episode 3.** Stage 2 has its own camera controller (0x22398) with its own mode numbers. The ten
   events of its table (X 0x300 to 0x2D00, all "mode 2") are **wave locks**, not diagonal descents, and the car drive is not an
   event: it is mode 8, entered while `G.$518 != 0`. The port reads the standard mode table for this stage, so Episode 3 is wrong
   in the port today (sections 2.3, 7.1 and 9). [R 0x2250A, 0x22B9E; M e3footn, car1]
2. **Area changes are run by a script interpreter** ("scripted control", 0x1DB5E): 6 byte records, 41 actions, 26 scripts at
   0x1E6FC, three entry points (area start 0x1DA10, area end or boss death 0x1D95E, boss area start 0x1DAC4). Every transition,
   every opening walk-in and every walk-off after a boss is one of these scripts. The actions are in 5.4 and the 26 scripts in 5.5. The port has none
   of it (`TEMP` shortcut in `prop.rs`: the camera gets `go` at once). This is the largest missing piece of M2/M3.
3. **There is no GO arrow at the end of a wave.** The only GO picture is the red "GO" burst of pool B class 20. It blinks 3 times
   (30 frames on, 5 off, sound 0x3E each time) in two cases: (a) forced, right at the start of every area end script
   (action 0x46); (b) as an idle hint, 180 frames after the camera lock ended, when the camera moved less than 0x80 pixels, and
   again every 600 + 105 frames while the players stand still. [R 0x7BB68; M go1, go2, l_e1a0_k2]
4. **The camera never leaves the leftmost player behind.** The rightmost player only pulls the camera while the spread is small;
   an idle player holds the camera back (measured: camera stops at leftmost X - 16). A player that is left of the camera is
   clamped to camera X + 0x10; the right limit is camera X + 0x16F. [R 0x20A22, 0x4FCE; M clampR, lag2, lag3]
5. **Heroes are untouchable during scripts** (the enemy attack passes are skipped while `G.$4DD != 0`) but they can still hit.
   [R 0x10546 call order, port `combat.rs`]
6. **Openings.** First level frame to player control: Episode 1 648 gf, Episode 2 573, Episode 3 321, Episode 4 894, Episodes 5 to 8
   321 each (of which 255 are the map card). Section 6. [M o1 to o8]
7. **End of an episode, measured on Vice T.:** boss HP 0 to normal speed 94 gf (368 video frames of slow motion), walk-off script
   369 gf to the first tally frame, tally 951 gf, story panel 306 video frames, next opening. Section 8. [M d4c-b3/b1c]
8. **Car section** (Episode 3): the radio makes a car (HP 120) drive in at 5 px per frame; the players jump aboard 280 frames after
   the pick up; the camera then rides at 11.0 px per frame; the section ends when the car's HP reaches 0 (the camera locks where it
   is, mode 3). Section 7.1. [M car1, car3]
9. **Lift** (Episode 7 area 1): ride speed rises by 6/256 px per frame squared to 6 px per frame; 4 stops at Y 0x3000, 0x1E00,
   0x1400, 0x800 and the end lock at Y 0; whole ride about 5800 gf with fights. Section 7.2. [M e7a1n, lift2]
10. **Screen shake** is verified: 29 requesters, strengths 1 to 4, a strength s shake lasts 6 s - 2 game frames. Section 7.4.
11. **Row scroll formula of the plan matches MAME** at three camera positions. Section 7.3. [M rows_300, rows_1000, rows_2400]
12. Corrections to older documents: class 96 (music starter) is created only by the sub-area switch (camera mode 8), not by every
    area start (D3e part 1 said every area): full area changes start the music directly (hook 0xACD0 calls 0xA55A). Episode 8
    area 3 has no valid start position row (the table 0x4E56 ends early; reading it gives code bytes), but a
    sub-area switch never places players, so it is harmless. [R 0x20EC8 only caller of 0x9236C; 0x4E0A]

Port state in one line: the standard camera (modes, events, follow, locks, wave controller, push rules, clamp) is DONE; the
scripted control, the stage 2 controller, class 20, shake, row scroll, the lift background, the opening and map card tasks and the
clear task are MISSING or STUB (full table in section 9).

## 1. The three layers that run a level

| Layer | Routine | What it owns | Port |
| --- | --- | --- | --- |
| Camera events | scroll2 controller 0x206D4 (stage 2: 0x22398) with the event table 0x21B2A (stage 2: 0x22B9E) | where the camera stops, which map area is active, what happens at the end of an area | DONE (not stage 2) |
| Spawn lists and waves | spawn interpreter 0xFBF4, wave controller = pool B class 15 (0x7B336), clock class 31, hint class 20 (0x7BB68) | enemies, props, locks that end by "enemies left <= n", GO | DONE except class 20 and 31 |
| Scripted control | 0x1DA10 (area start), 0x1D95E (area end, boss death), 0x1DAC4 (boss area start), per hero step 0x1DB5E | every walk-in, walk-off, kick, pose; when the hero has control | MISSING |

The three talk through five variables: the camera lock state `G.$7B4` (0 free, 1 locked, 2 stage end lock), the camera `go` flag
`G.$7A9` (set by the wave end, by the script action 0x2C and 0x36, by class 97; cleared by the camera), the scripted mask `G.$4DD`
(bit per hero; set to the chosen players `G.$4E4` by 0x1DA10, 0x1D95E and 0x1DAC4), the script id `G.$4DE`, and the intro and
clear flags `G.$4E2` and `G.$4E3`.

## 2. Camera

### 2.1 Controller state and per frame order (summary; full pseudo code is in `re/specs/stage/SPEC.md` 5 and 6)

- The camera is updated at the end of the object update (0x2042E), after all objects moved and before the sprite list is built
  (so objects see the camera of the previous frame). Order in a frame: rank tick, objects (census, players, items, enemies, shake
  0x222D6, props, effects, camera 0x2042E), collision, HUD, sprites, time, spawn lists. [R spec 5]
- `G.$744` camX and `G.$748` camY are 16.16 values; `G.$786` area; `G.$741` mode; `G.$742` sub step; `G.$7A8` event index;
  `G.$792` pointer to the next event; `G.$7B4` lock; `G.$7B5` "events trigger on Y"; `G.$4DC` vertical scene flag (only Episode 7
  area 1). [R spec 5.1]
- An event is a 16 byte record `{X, Y, push, up, down, area, mode, vfollow, flag70, next mode, by Y}`. The next record loads when
  `camX >= X` (or `camY <= Y` for by-Y events, signed), one per frame, not in modes 1, 9, 0xE, 0xF, 0x12. Loading copies the fields
  into the controller, sets `$5A = X`, `$5C = camY` and the area byte (so the area changes the moment the event loads). [R 0x20898]
- `push` replaces the default push line (0xC0 for 1 player, 0xE8 for 2 or 3) only when `flag70 = 1`. `up` and `down` are the
  vertical follow thresholds and are used only when `vfollow = 1`. `next mode` is the mode used after a lock ends (0 = follow;
  Episode 2 area 2 uses 2 = diagonal, Episode 7 area 1 uses 5 = lift ride). [R spec 5.2]

### 2.2 Modes of the standard table (0x2092E), what each does in play, measured numbers

| Mode | Name | Behaviour | Measured |
| --- | --- | --- | --- |
| 0x00 | follow | camX follows the rightmost player (rules in 2.4), vertical follow only if `vfollow` | max 6 px per frame (6.0 px per frame during the chase [M clampR]) |
| 0x01 | area end with fade | sub 0: lock = 1, go = 0. sub 2: camX fixed at the event X, waits for `go`; on `go`: music fade out request 0xFF07 (hook 0xA5EA), palette fade out task. sub 4: waits for the fade, lock = 0, skips to the next event with X = 0, loads that area (hook 0xACD0: players placed, pools and rank reset, time, music, HUD), palettes, scroll3 re-init. sub 6: HUD redraw, fade in task. sub 8: when `go` is 0 again loads the next event | `go` to new area 16 gf, fade out 16 vblanks, fade in 16 vblanks [M e4a0n, e4a1n, e6a0n] |
| 0x02 | diagonal, camY falls | camY goes from its value at the event down to the event Y while camX advances (the players walk down the screen to the right); then mode 0 | Episode 2 area 2, Episode 5 area 1 |
| 0x03 | diagonal, camY rises | mirror of 0x02 (camY rises to the event Y) | Episode 6 area 2 (event 12) |
| 0x04 | lock | first frame lock = 1, go = 0; every frame camX (or camY if by Y) = event value; when `go`: mode = next mode, lock = 0 | camera snaps back to the event X at once (it can have overshot by up to 5 px) [M] |
| 0x05 | lift ride | stage 6 area 1 only, section 7.2 | |
| 0x06 | stage end lock | lock = 2, camX fixed at the event X; on `go`: `G.$4D4 = 1` (stage clear flag; the only reader found revives a dying hero with 1 HP, 0x1BF82 [I]) | Episodes 1, 2, 4, 5, 6 |
| 0x07 | descend with the players | camY = lowest player + 0x100 - threshold, never above 0x100 (unsigned compare) | Episode 1 area 2: camY 0x300 to 0x100 in 124 gf [M e1a1n] |
| 0x08 | sub-area switch | lock = 1 and camX = event X until `go`; then area + 1, mode 0, new OBJ palettes, **no fade**; time, floor table and spawn lists of the new area are reset (0xAAE2), player count and rank speed (0xACE0), creates pool B class 96 (music) | Episodes 2, 5, 6, 8 [M e2a0n, e5a0n, e6a1n, e8a2n] |
| 0x09 | area end, slow fade | like mode 1 (the longer fade delay is overwritten, the fade is 16 steps of 1 vblank) | Episode 8 area 1 [M e8a1n] |
| 0x0A | slope (X/4) | camY = (camX - $5A) / 4 + $5C | Episode 6 area 2 |
| 0x0B | slope down | camY = $5C - (camX - $5A) >> shift (table 0x20F58: shift 1 for 0.0, 2 for 5.2 and 7.2, else 0), not below the event Y | Episodes 1, 6, 8 |
| 0x0C | creep up | camY - 1 per frame while the rightmost player is at X >= 0x750, not below 0 | Episode 2 area 1: sinks into the swamp |
| 0x0D | slope up | camY = (camX - $5A) / 2 + $5C, not above the event Y | Episode 2 area 1 |
| 0x10 | stage end (Episode 8) | camX fixed; on `go`: `G.$4D4 = 1`, mode 0 | Episode 8 area 3 (event 21) |
| 0x11 | auto scroll | camX + 3 per frame (no follow) | Episode 1 opening: X 0 to 0x380 in 299 gf [M o1] |
| 0x12 | area change, no fade | like mode 1 but without fade and without the music fade. The new area is loaded in the frame `go` is seen; the palettes are reloaded at brightness 0 and set to full brightness by 0x892C, so there are 2 to 4 black video frames | Episode 1 area 1 to 2 [M n1a1] |
| 0x13 | stage end with fade | camX fixed; on `go`: `G.$4D4 = 1` and a fade out of the three scroll banks (mask 0x0D: the HUD layer stays) | Episode 7 area 3 (event 19) |

Episode 7 area 1 events trigger on Y (`by_y`): the lift ride, locks and the end.

### 2.3 The camera events of every episode (116 records), in the order a player meets them

Columns: `#` record index; `area` the area the event switches to; `mode` (name above); `push` the push line (`-` = default 0xC0 or
0xE8); `up/down` vertical thresholds; `vf` vertical follow flag; `next` mode after a lock; the last column is the wave whose
spawn list record has the same trigger (`n spawns`, end type: 0 = GO, 2 = walk to the next area (script 0x1D95E), 4 = boss area
(script 0x1DAC4); `left <= [1P, 2P, 3P]` is the final wait). Source: `re/out/assets/data/stages.json` (X, Y, mode) and
`re/specs/enemy/stage_scripts.txt` (waves); the table is generated by `re/out/d4c/gen_events_md.py`. [R 0x21B2A table]

### Episode 1 (stage 0)

| # | area | X | Y | mode | push | up/down | vf | next | wave at this stop |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 0 | 0x0 | 0x100 | 11 auto scroll 3 | - | - | 0 | 0 |  |
| 1 | 0 | 0x380 | 0x100 | 00 follow | - | 0x80/0x20 | 0 | 0 |  |
| 2 | 0 | 0x408 | 0xD0 | 0B slope down | - | - | 0 | 0 |  |
| 3 | 0 | 0x460 | 0xD0 | 00 follow | - | - | 0 | 0 |  |
| 4 | 0 | 0x4D0 | 0xD0 | 04 lock | 0x128 | - | 0 | 0 | W0: 2 spawns, end type 2, left <= [0, 0, 0] |
| 5 | 0 | 0x580 | 0x0 | 01 area end, fade | 0x128 | 0x80/0x20 | 1 | 0 |  |
| 6 | 1 | 0x0 | 0x0 | 04 lock | - | - | 0 | 0 | W1: 5 spawns, end type 0, left <= [4, 5, 6] |
| 7 | 1 | 0x1B0 | 0x0 | 04 lock | - | - | 0 | 0 | W2: 3 spawns, end type 0, left <= [4, 5, 5] |
| 8 | 1 | 0x220 | 0x0 | 12 area change, no fade | - | - | 0 | 0 | W3: 0 spawns, end type 2, left <= [0, 0, 0] |
| 9 | 2 | 0x0 | 0x300 | 07 descend with players | - | 0x100/0x20 | 1 | 0 |  |
| 10 | 2 | 0x0 | 0x100 | 04 lock | - | - | 0 | 0 | W4: 3 spawns, end type 0, left <= [0, 0, 0] |
| 11 | 2 | 0xA0 | 0x100 | 04 lock | 0x78 | - | 0 | 0 | W5: 1 spawns, end type 4, left <= [99, 99, 99]; set time 3:50 |
| 12 | 2 | 0x1D0 | 0x100 | 06 stage end lock | 0xF0 | - | 0 | 0 |  |

### Episode 2 (stage 1)

| # | area | X | Y | mode | push | up/down | vf | next | wave at this stop |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 0 | 0x0 | 0x40 | 00 follow | - | - | 0 | 0 |  |
| 1 | 0 | 0x568 | 0x40 | 08 sub-area switch | - | - | 0 | 0 | W2: 10 spawns, end type 2, left <= [0, 0, 0] |
| 2 | 1 | 0x620 | 0x0 | 0C creep up | - | - | 0 | 0 |  |
| 3 | 1 | 0x7C0 | 0x0 | 00 follow | - | - | 0 | 0 |  |
| 4 | 1 | 0x860 | 0x0 | 04 lock | - | - | 0 | 0 | W3: 5 spawns, end type 0, left <= [0, 0, 0] |
| 5 | 1 | 0xA00 | 0x10 | 0D slope up | - | - | 0 | 0 |  |
| 6 | 1 | 0xA20 | 0x10 | 00 follow | - | - | 0 | 0 |  |
| 7 | 1 | 0xB80 | 0x0 | 01 area end, fade | - | - | 0 | 0 | W5: 5 spawns, end type 2, left <= [0, 0, 0] |
| 8 | 2 | 0x0 | 0x0 | 02 diagonal (camY falls) | 0x100 | 0x100/0x20 | 1 | 0 |  |
| 9 | 2 | 0x118 | 0x0 | 00 follow | 0x78 | 0x100/0x20 | 1 | 0 |  |
| 10 | 2 | 0x180 | 0x0 | 04 lock | 0x78 | 0x100/0x20 | 1 | 2 | W6: 3 spawns, end type 4, left <= [99, 99, 99]; set time 3:50 |
| 11 | 2 | 0x200 | 0x0 | 00 follow | 0x78 | - | 0 | 0 |  |
| 12 | 2 | 0x260 | 0x0 | 06 stage end lock | - | - | 0 | 0 |  |

### Episode 3 (stage 2)

| # | area | X | Y | mode | push | up/down | vf | next | wave at this stop |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 0 | 0x0 | 0x0 | 00 follow | - | - | 0 | 0 |  |
| 1 | 0 | 0x300 | 0x0 | 02 lock | - | - | 0 | 0 | W0: 0 spawns, end type 0, left <= [0, 0, 0] |
| 2 | 0 | 0x500 | 0x0 | 02 lock | - | - | 0 | 0 | W1: 0 spawns, end type 0, left <= [0, 0, 0] |
| 3 | 0 | 0x900 | 0x0 | 02 lock | - | - | 0 | 0 | W2: 4 spawns, end type 0, left <= [0, 0, 0] |
| 4 | 0 | 0xC00 | 0x0 | 02 lock | - | - | 0 | 0 | W3: 4 spawns, end type 0, left <= [0, 0, 0] |
| 5 | 0 | 0x1000 | 0x0 | 02 lock | - | - | 0 | 0 | W4: 4 spawns, end type 0, left <= [0, 0, 0] |
| 6 | 0 | 0x1900 | 0x0 | 02 lock | - | - | 0 | 0 | W5: 0 spawns, end type 0, left <= [0, 0, 0] |
| 7 | 0 | 0x1D00 | 0x0 | 02 lock | - | - | 0 | 0 | W6: 0 spawns, end type 0, left <= [0, 0, 0] |
| 8 | 0 | 0x2300 | 0x0 | 02 lock | - | - | 0 | 0 | W7: 0 spawns, end type 0, left <= [0, 0, 0] |
| 9 | 0 | 0x2700 | 0x0 | 02 lock | - | - | 0 | 0 | W8: 0 spawns, end type 0, left <= [0, 0, 0] |
| 10 | 0 | 0x2D00 | 0x0 | 02 lock | - | - | 0 | 0 |  |

### Episode 4 (stage 3)

| # | area | X | Y | mode | push | up/down | vf | next | wave at this stop |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 0 | 0x0 | 0x0 | 00 follow | - | - | 0 | 0 |  |
| 1 | 0 | 0x280 | 0x0 | 01 area end, fade | - | - | 0 | 0 | W0: 4 spawns, end type 2, left <= [0, 0, 0] |
| 2 | 1 | 0x0 | 0x0 | 00 follow | - | - | 0 | 0 |  |
| 3 | 1 | 0x190 | 0x0 | 04 lock | - | - | 0 | 0 | W1: 13 spawns, end type 0, left <= [3, 3, 3] |
| 4 | 1 | 0x580 | 0x0 | 04 lock | - | - | 0 | 0 | W2: 0 spawns, end type 0, left <= [99, 99, 99]; set time 3:50 |
| 5 | 1 | 0x880 | 0x0 | 01 area end, fade | - | - | 0 | 0 | W3: 11 spawns, end type 2, left <= [0, 0, 0] |
| 6 | 2 | 0x0 | 0x0 | 00 follow | - | - | 0 | 0 |  |
| 7 | 2 | 0x100 | 0x0 | 04 lock | - | - | 0 | 0 | W4: 0 spawns, end type 0, left <= [0, 0, 0] |
| 8 | 2 | 0x2C0 | 0x0 | 04 lock | 0x78 | - | 0 | 0 | W5: 14 spawns, end type 4, left <= [99, 99, 99]; set time 3:50 |
| 9 | 2 | 0x470 | 0x0 | 06 stage end lock | - | - | 0 | 0 |  |

### Episode 5 (stage 4)

| # | area | X | Y | mode | push | up/down | vf | next | wave at this stop |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 0 | 0x0 | 0x0 | 00 follow | - | - | 0 | 0 |  |
| 1 | 0 | 0x100 | 0x0 | 04 lock | - | - | 0 | 0 | W0: 1 spawns, end type 0, left <= [1, 1, 1] |
| 2 | 0 | 0x3D0 | 0x0 | 04 lock | - | - | 0 | 0 | W1: 8 spawns, end type 0, left <= [0, 0, 0] |
| 3 | 0 | 0x740 | 0x0 | 04 lock | - | - | 0 | 0 | W2: 4 spawns, end type 0, left <= [99, 99, 99]; set time 3:50 |
| 4 | 0 | 0x940 | 0x0 | 04 lock | - | - | 0 | 0 | W3: 8 spawns, end type 0, left <= [2, 3, 3] |
| 5 | 0 | 0xA60 | 0x200 | 08 sub-area switch | 0xC0 | - | 0 | 0 | W4: 16 spawns, end type 2, left <= [0, 0, 0] |
| 6 | 1 | 0xCD0 | 0x0 | 04 lock | 0xC0 | 0x100/0x20 | 0 | 0 | W5: 5 spawns, end type 0, left <= [99, 99, 99]; set time 3:50 |
| 7 | 1 | 0xCE0 | 0x0 | 00 follow | 0xC0 | 0x100/0x20 | 0 | 0 |  |
| 8 | 1 | 0xD50 | 0x100 | 02 diagonal (camY falls) | 0xC0 | 0x100/0x20 | 0 | 0 |  |
| 9 | 1 | 0xF00 | 0x100 | 00 follow | 0xC0 | - | 0 | 0 |  |
| 10 | 1 | 0xF20 | 0x0 | 04 lock | 0x78 | - | 0 | 0 | W6: 12 spawns, end type 4, left <= [99, 99, 99]; set time 3:50 |
| 11 | 1 | 0x11C0 | 0x0 | 06 stage end lock | 0x78 | - | 0 | 0 |  |

### Episode 6 (stage 5)

| # | area | X | Y | mode | push | up/down | vf | next | wave at this stop |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 0 | 0x0 | 0x0 | 00 follow | - | - | 0 | 0 |  |
| 1 | 0 | 0x200 | 0x0 | 04 lock | - | - | 0 | 0 | W0: 8 spawns, end type 0, left <= [99, 99, 99]; set time 3:50 |
| 2 | 0 | 0x730 | 0x0 | 04 lock | 0x100 | - | 0 | 0 | W2: 10 spawns, end type 2, left <= [0, 0, 0] |
| 3 | 0 | 0x780 | 0x0 | 01 area end, fade | 0x100 | - | 0 | 0 |  |
| 4 | 1 | 0x0 | 0x0 | 00 follow | - | 0x100/0x10 | 0 | 0 |  |
| 5 | 1 | 0x300 | 0x0 | 04 lock | - | 0x100/0x10 | 1 | 0 | W4: 2 spawns, end type 0, left <= [0, 0, 0] |
| 6 | 1 | 0x8C0 | 0x100 | 04 lock | 0x100 | - | 0 | 0 | W5: 4 spawns, end type 0, left <= [99, 99, 99]; set time 2:50 |
| 7 | 1 | 0x930 | 0xE0 | 0B slope down | 0x100 | - | 0 | 0 |  |
| 8 | 1 | 0x970 | 0xE0 | 00 follow | 0x100 | - | 0 | 0 |  |
| 9 | 1 | 0xAE0 | 0xE0 | 08 sub-area switch | 0x100 | - | 0 | 0 | W6: 9 spawns, end type 2, left <= [0, 0, 0] |
| 10 | 2 | 0xCA0 | 0x100 | 0A slope (X/4) | 0x100 | 0x80/0x0 | 1 | 0 |  |
| 11 | 2 | 0xD20 | 0x100 | 00 follow | 0x100 | - | 0 | 0 |  |
| 12 | 2 | 0xE00 | 0x170 | 03 diagonal (camY rises) | 0x100 | 0x80/0x0 | 1 | 0 |  |
| 13 | 2 | 0xF00 | 0x170 | 04 lock | 0x78 | - | 0 | 0 | W7: 1 spawns, end type 4, left <= [99, 99, 99]; set time 3:50 |
| 14 | 2 | 0x1000 | 0x170 | 06 stage end lock | 0x78 | - | 0 | 0 |  |

### Episode 7 (stage 6)

| # | area | X | Y | mode | push | up/down | vf | next | wave at this stop |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 0 | 0x0 | 0x0 | 00 follow | - | - | 0 | 0 |  |
| 1 | 0 | 0x1B0 | 0x0 | 04 lock | - | - | 0 | 0 | W0: 4 spawns, end type 0, left <= [1, 1, 1] |
| 2 | 0 | 0x580 | 0x0 | 04 lock | - | - | 0 | 0 | W1: 5 spawns, end type 0, left <= [99, 99, 99]; set time 3:50 |
| 3 | 0 | 0x790 | 0x0 | 04 lock | - | - | 0 | 0 | W2: 6 spawns, end type 0, left <= [3, 3, 4] |
| 4 | 0 | 0x930 | 0x0 | 04 lock | - | - | 0 | 0 | W3: 15 spawns, end type 2, left <= [0, 0, 0] |
| 5 | 0 | 0x9E0 | 0x0 | 01 area end, fade | - | - | 0 | 0 |  |
| 6 | 1 | 0x0 | 0x3700 | 05 lift ride | - | - | 0 | 0 |  |
| 7 | 1 | 0x0 | 0x3000 | 04 lock | - | - | 0 | 5 | W4: 7 spawns, end type 0, left <= [99, 99, 99]; set time 3:50 |
| 8 | 1 | 0x0 | 0x1E00 | 04 lock | - | - | 0 | 5 | W6: 8 spawns, end type 0, left <= [99, 99, 99]; set time 1:50 |
| 9 | 1 | 0x0 | 0x1400 | 04 lock | - | - | 0 | 5 | W7: 5 spawns, end type 0, left <= [99, 99, 99]; set time 1:50 |
| 10 | 1 | 0x0 | 0x800 | 04 lock | - | - | 0 | 5 | W8: 5 spawns, end type 0, left <= [99, 99, 99]; set time 3:50 |
| 11 | 1 | 0x0 | 0x0 | 01 area end, fade | - | - | 0 | 0 | W9: 0 spawns, end type 2, left <= [0, 0, 0] |
| 12 | 2 | 0x0 | 0x0 | 04 lock | - | - | 0 | 0 | W10: 4 spawns, end type 0, left <= [0, 0, 0] |
| 13 | 2 | 0x1D0 | 0x0 | 04 lock | - | - | 0 | 0 | W11: 12 spawns, end type 0, left <= [99, 99, 99]; set time 2:50 |
| 14 | 2 | 0x640 | 0x0 | 04 lock | 0x128 | - | 0 | 0 | W12: 7 spawns, end type 2, left <= [0, 0, 0] |
| 15 | 2 | 0x700 | 0x0 | 01 area end, fade | 0x128 | - | 0 | 0 |  |
| 16 | 3 | 0x0 | 0x0 | 00 follow | - | - | 0 | 0 |  |
| 17 | 3 | 0x240 | 0x0 | 04 lock | 0x78 | - | 0 | 0 | W13: 4 spawns, end type 4, left <= [99, 99, 99]; set time 3:50 |
| 18 | 3 | 0x450 | 0x0 | 04 lock | 0x120 | - | 0 | 0 |  |
| 19 | 3 | 0x4F0 | 0x0 | 13 stage end, fade | 0x120 | - | 0 | 0 |  |

### Episode 8 (stage 7)

| # | area | X | Y | mode | push | up/down | vf | next | wave at this stop |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 0 | 0x0 | 0x0 | 00 follow | - | - | 0 | 0 |  |
| 1 | 0 | 0x120 | 0x0 | 04 lock | - | - | 0 | 0 | W0: 5 spawns, end type 0, left <= [0, 0, 0] |
| 2 | 0 | 0x2D0 | 0x0 | 04 lock | - | - | 0 | 0 | W1: 5 spawns, end type 0, left <= [0, 0, 0] |
| 3 | 0 | 0x480 | 0x0 | 04 lock | 0xA0 | - | 0 | 0 | W2: 8 spawns, end type 2, left <= [0, 0, 0] |
| 4 | 0 | 0x580 | 0x0 | 01 area end, fade | 0xA0 | - | 0 | 0 |  |
| 5 | 1 | 0x0 | 0x300 | 00 follow | - | - | 0 | 0 |  |
| 6 | 1 | 0x1A0 | 0x300 | 04 lock | - | - | 0 | 0 | W3: 7 spawns, end type 0, left <= [0, 0, 0] |
| 7 | 1 | 0x370 | 0x300 | 04 lock | - | - | 0 | 0 | W4: 5 spawns, end type 0, left <= [0, 0, 0] |
| 8 | 1 | 0x570 | 0x300 | 04 lock | - | - | 0 | 0 | W5: 4 spawns, end type 0, left <= [99, 99, 99]; set time 2:50 |
| 9 | 1 | 0x840 | 0x300 | 04 lock | - | - | 0 | 0 | W6: 9 spawns, end type 0, left <= [0, 0, 0] |
| 10 | 1 | 0xA00 | 0x300 | 04 lock | 0xC0 | - | 0 | 0 | W7: 2 spawns, end type 2, left <= [0, 0, 0] |
| 11 | 1 | 0xBC0 | 0x300 | 09 area end, slow fade | 0xC0 | - | 0 | 0 |  |
| 12 | 2 | 0x0 | 0x200 | 04 lock | 0x100 | - | 0 | 0 | W8: 1 spawns, end type 0, left <= [0, 0, 0] |
| 13 | 2 | 0x100 | 0x180 | 0B slope down | 0x100 | - | 0 | 0 |  |
| 14 | 2 | 0x300 | 0x180 | 00 follow | 0x100 | - | 0 | 0 |  |
| 15 | 2 | 0x400 | 0x100 | 0B slope down | 0x100 | - | 0 | 0 |  |
| 16 | 2 | 0x600 | 0x100 | 00 follow | 0x100 | - | 0 | 0 |  |
| 17 | 2 | 0x730 | 0x100 | 04 lock | 0x100 | - | 0 | 0 | W9: 4 spawns, end type 0, left <= [1, 2, 2] |
| 18 | 2 | 0x900 | 0x100 | 04 lock | 0x100 | - | 0 | 0 | W10: 5 spawns, end type 0, left <= [0, 0, 0] |
| 19 | 2 | 0xCB0 | 0x100 | 08 sub-area switch | 0x78 | - | 0 | 0 | W11: 8 spawns, end type 4, left <= [0, 0, 0] |
| 20 | 3 | 0x1120 | 0x100 | 04 lock | 0x80 | - | 0 | 0 |  |
| 21 | 3 | 0x1180 | 0x100 | 10 stage end (4D4) | 0x80 | - | 0 | 0 |  |

Notes per episode (what a player sees at each stop; the measured items are marked [M]):

- **Episode 1.** Event 0 is the opening: the camera pans from X 0 to 0x380 by itself (3 px per frame, 299 gf) while the heroes stand at
  X 0x420 (P1), 0x410, 0x400, so they slide into the picture from the right edge; the clamp is switched off during it (`G.$506 = 1`).
  Event 2 tilts the camera down 0x30 px over 0x60 px of X (roof slope). The lock at 0x4D0 is wave 0; after it the door script
  (kind 2) runs and event 5 at 0x580 fades to the hall. Area 1 locks at X 0 (arrival, wave 1) and 0x1B0 (wave 2); event 8 at 0x220 is
  the window (mode 0x12, no fade). Area 2 starts high (Y 0x300, falling): mode 7 follows the fall (124 gf), then locks at (0, 0x100)
  and X 0xA0; the last event (X 0x1D0, push 0xF0) freezes the camera at X 464 for the Vice fight [M b1c: camX 464].
- **Episode 2.** The first area ends in a sub-area switch at 0x568 (no reload, the swamp keeps the map): wave 2 ends with script 7 (the
  heroes jump into the water). Area 1 events: creep up at 0x620 (camera sinks 0x40 px, 1 px per frame), lock 0x860, slope up 0xA00
  (out of the mud), fade out at 0xB80. Area 2 (diagonal, vertical follow on): lock at 0x180 has `next = 2`, so after the lock the
  diagonal continues; the Butcher fight is locked at camX 0x260 = 608 [M b2c].
- **Episode 3.** Ten locks of the stage 2 controller (X 0x300, 0x500, 0x900, 0xC00, 0x1000, 0x1900, 0x1D00, 0x2300, 0x2700, 0x2D00).
  Waves 0, 1 and 5 to 8 have no spawns: the lock lasts about 32 gf and ends with GO (measured at 0x300 and 0x500, gf 2703 to 2735 and
  3509 to 3541 [M e3footn]). Waves 2, 3 and 4 spawn four enemies each (class 70, poachers, Hammers). The lock at 0x2D00 has no wave: the Hogg record
  and the time set (3:50) come from the spawn list. A radio taken before it replaces all of this by the car drive (7.1): in car
  mode no event is loaded any more (events are skipped in mode 8).
- **Episode 4.** Three simple areas. Area 0 ends at 0x280 with the fade (wave 0 type 2). Area 1: locks 0x190 (13 spawns) and 0x580
  (no spawns, sets 3:50, GO at once), fade at 0x880. Area 2: lock 0x100 (empty wave, GO), lock 0x2C0 (boss area, push 0x78), stage end
  lock 0x470 (Slice).
- **Episode 5.** Area 0: locks 0x100, 0x3D0, 0x740, 0x940, then a sub-area switch at 0xA60 (wave 4, 16 spawns, type 2: Walther
  battle before the bridge). Area 1 (bridge and village): lock 0xCD0 (wave 5 has spawns but its end wait is 99, so GO at once and
  3:50 is set), follow, diagonal descent at 0xD50 (Y 0x100), follow 0xF00, lock 0xF20 (boss area, push 0x78), stage end 0x11C0.
- **Episode 6.** Area 0: lock 0x200 (wave 0, GO at once), lock 0x730 (wave 2, push 0x100, type 2), fade at 0x780. Area 1 (vertical
  follow thresholds 0x100 / 0x10 in the first two events): lock 0x300, lock 0x8C0 (Y 0x100, time 2:50), slope down 0x930, sub-area
  switch 0xAE0 (wave 6, type 2). Area 2 is the sub-area: slope 0xCA0, follow 0xD20, diagonal up 0xE00 (Y 0x170), boss lock 0xF00
  (push 0x78), stage end 0x1000. Waves 1 and 3 of the stage have no record (unused).
- **Episode 7.** Area 0: locks 0x1B0, 0x580, 0x790, 0x930 and the fade at 0x9E0. Area 1: the lift (7.2). Area 2 (library): locks at
  0, 0x1D0, 0x640 (push 0x128), fade at 0x700. Area 3: follow, lock 0x240 (wave 13, boss area), lock 0x450 (no wave: the boss
  lock, push 0x120), stage end with fade 0x4F0 (event 19, mode 0x13).
- **Episode 8.** Area 0: locks 0x120, 0x2D0, 0x480 (push 0xA0), fade at 0x580. Area 1 (sewers, Y 0x300): locks 0x1A0, 0x370, 0x570,
  0x840, 0xA00 (push 0xC0), slow fade at 0xBC0. Area 2 (corridor and cave): lock at 0 (Y 0x200), slope down at 0x100 and 0x400,
  locks 0x730 and 0x900, sub-area switch 0xCB0 (wave 11 ends with type 4: the heroes walk to the lair, script 23). Area 3: lock 0x1120
  (push 0x80), stage end 0x1180 (mode 0x10).

### 2.4 Which player moves the camera (2 or 3 players), and who is left behind

Rules of `follow_x` (0x20A22), checked in MAME: [R spec 6.3 and 6.4; M clampR, lag2, lag3]

1. Only players that are alive (`+$00 != 0`) and in a state (`+$04 != 0`) count. A dead player waiting to drop in does not.
2. A player left of the camera (X < camX) does not hold the camera: his X is replaced by camX + 0x10.
3. `lead` = screen X of the rightmost counted player. Nothing happens while `lead <= 0x40` or `lead < push` (push 0xC0 for 1
   player, 0xE8 for 2 or 3 players, or the event's own `push` when `flag70 = 1`).
4. `spread` = rightmost X - leftmost X. If `spread >= push - 0x10` the camera moves only as far as the leftmost player allows:
   `step = clamp(leftmost X - 0x10 - camX, 0, 6)`. Otherwise `step = min(lead - push, 6)`.
5. So the camera moves at most 6 px per frame, never moves back, and never passes the leftmost player minus 16. **An idle player
   holds the camera.** Measured: Episode 2 area 0, P1 walking right, P2 and P3 idle at X 399 and 412: the camera stopped at 383
   (= 399 - 16) and P1 stopped at X 751 = camX + 0x16F (the right clamp) for the rest of the 900 frames [M lag3]. With P2 pressing
   left in Episode 1 area 1 the camera stopped at 45 = P2 X 61 - 16 [M lag2].
6. A player pushed far right by a poke (X 0x7FF, camera 0x200) is pulled back to camX + 0x16F each frame while the camera chases
   him at 6 px per frame (clamp uses the camera of the previous frame) [M clampR].
7. Vertical follow (`vfollow = 1`): `Yt = player Z + floor` of the highest and lowest player; the camera moves up or down by at most
   6 px per frame when the lowest player is beyond the `up`/`down` thresholds. The code has a quirk (a large upward overshoot
   moves the camera down by 6); keep it. [R spec 6.5]

### 2.5 Port state of the camera

| Item | State | File |
| --- | --- | --- |
| Standard modes 0x00 to 0x13, event loader, follow, vertical follow, push line, extents | DONE (reviewed against the code by the stage port; this document confirms the observed order of events in 23 areas) | `camera.rs` |
| Mode 8 hooks (0xAAE2, 0xACE0, 0x9236C) | PARTIAL: hooks exist; the class 96 object created by `hook_9236c` runs no code (prop dispatch frees it) so the music change after a sub-area switch never plays | `flow.rs`, `prop.rs` |
| Mode 5 lift ride | DONE for the camera (calls `cam_shake(2)`); the scroll1 background of the shaft is MISSING | `camera.rs` |
| Stage 2 controller 0x22398 (own mode table, mode 8 car drive, locks) | MISSING: `background_update` always runs the standard controller, so the ten Episode 3 locks load as diagonal descents | `camera.rs` |
| Stage 2 row scroll 0x2335A | MISSING (formula verified, section 7.3) | `camera.rs`, renderer |
| Screen shake 0x222D6 | STUB (`screen_shake` is empty, requests are stored) | `camera.rs`, `present.rs` |
| Scripted release of locks (script GO flag, walk) | MISSING: `prop_area_walk_fallback` gives `go` as soon as the camera locks (TEMP) | `prop.rs` |


## 3. Player bounds

### 3.1 The screen clamp (0x4FCE), applied to every present hero every frame after his move

Horizontal scenes (`G.$4DC = 0`, all areas except Episode 7 area 1): [R 0x4FCE; M clampR]

- X is forced into `[camX + 0x10, camX + 0x16F]`. A hero left of the window is moved to the left edge, one right of it to the right edge.
  The camera used is the one of the previous frame, so a hero poked far right is pinned at `camX + 0x16F` while the camera chases
  him at 6 px per frame (measured: X 880, 892, 904 ... with camX 518, 530, 542 ...).
- Screen depth: `Z + floor - camY` is forced into `[0x10, 0x9F]` (`floor` = object `+$54`, the floor height under the hero). So the
  walkable band is at most 0x90 = 144 px deep on the screen and follows the camera in Y.
- Vertical scene (Episode 7 area 1): X in `[0x10, 0x16F]` and Z in `[0x10, 0x9F]`, no camera, no floor.
- The whole clamp is skipped while `G.$506 != 0` (set by script records, section 5.3). That is how the heroes walk in from outside the
  picture (X -48 at the start of most areas) and how the Episode 1 opening leaves them at X 0x420 while the camera pans.
- Enemies and items have their own off-screen test (0x173C frees them when far away); only heroes are clamped.

### 3.2 Depth limits per area (measured)

The clamp allows 144 px, but the stage map (terrain class in the scroll2 tile attribute plus the floor segment lists 0x12C30) is
narrower in most areas. Measured with a hero holding Up for 70 frames and then Down for 110 frames at three camera positions
(25, 50 and 75 percent of the area; `front` = lowest Z reached, `back` = highest Z; the third column is the camera Y and the floor
offset at the sample). The terrain data is already ported (`terrain.rs`, checked with `terrain_oracle.py`), so these numbers are
test values, not new data. [M d4c-b2/d*_*_*; W1 walks d4c-b1/w*]

| Area | front Z / back Z at the three positions | camY, floor |
| --- | --- | --- |
| Ep1 a0 roofs | 272 / 361, 288 / 335, 288 / 335 | 256 to 244 |
| Ep1 a1 hall | 16 / 79 at all three | 0 |
| Ep1 a2 street | 272 / 319 at the start | 256 |
| Ep2 a0 forest | 16 / 79 at all three | 64, floor 64 |
| Ep2 a1 swamp | 16 / 79, 74 / 79 (camY 58), 8 / 71 (camY 16, floor 24) | 0 to 58 |
| Ep2 a2 yard | 32 / 114 at all three | 0 (the camera has left the diagonal at these X) |
| Ep3 | 16 / 79 at all three | 0 |
| Ep4 a0, a2 | 16 / 79 | 0 |
| Ep4 a1 garage | 16 / 111 | 0 |
| Ep5 a0 road | 528 / 607 | 512 (screen band 16 to 95) |
| Ep5 a1 bridge, village | 272 / 370, 272 / 351, 272 / 351 | 256 |
| Ep6 a0 cabins | 16 / 79 | 0 |
| Ep6 a1 jungle | 242 / 305 | 256, floor 30 (screen band 16 to 79) |
| Ep6 a2 | 242 / 305, 354 / 401, 354 / 419 | 248 to 368, floor 30 |
| Ep7 a0 | 80 / 127, 80 / 127, 16 / 79 | 0 |
| Ep7 a1 lift | 32 / 79 (vertical clamp, screen fixed) | |
| Ep7 a2 library | 69 / 111, 83 / 111, 96 / 111 (front limit rises along the area) | 0 |
| Ep7 a3 lab | 16 / 91 to 95 | 0 |
| Ep8 a0 | 16 / 95 | 0 |
| Ep8 a1 sewers | 768 / 831, 768 / 863, 768 / 799 | 768, floor 16 |
| Ep8 a2 | 273 / 351, 273 / 351, 288 / 351 | 392 to 272, floor 159 to 0 |

Reading: most areas give a 64 px deep band (screen band 16 to 79); the garage, sewers and cave give about 95 to 111; the library's front
limit climbs from 69 to 96 (a counter or stairs in the map). Heroes can never be nearer to the viewer than the front clamp:
there is no walking "out of the bottom".

### 3.3 What stops a hero besides the clamp

| Thing | Rule | Evidence |
| --- | --- | --- |
| Walls and obstacles (map tiles with a wall class, and solid pool E objects such as the golden statues of the Episode 1 hall and tire stacks) | the terrain pass 0x12CB4 / 0x12CAC; scripted walkers use the floor only test when `G.$4F0 = 1` (walls ignored) | R terrain.rs; M the camera in the first Episode 1 hall run stopped at 496 until the heroes punched (probably a statue [I]) |
| The camera lock | the camera stops at the event X, the right clamp stops the heroes at `lockX + 0x16F` | M every lock |
| Edge push | there is none: the left clamp moves a slow hero along only when the camera is forced (opening pan, diagonal modes, car drive); in follow mode the camera waits for him | R follow_x |
| A hero who is dead or waiting to drop in | not counted for the camera | R 6.3 |
| Drop in (respawn, join) | X = camX + offset from table 0x638C: 1 player 96; 2 players 64 and 128; 3 players 48, 96 and 144 (the order follows who is present); Z = camY - floor + 32, 48, 64 by player index; height floor + 0x100 (falls in); vertical scene: X 96 for all three, Z 32, 48, 64 | R 0x62D6, 0x638C, 0x63A4 |

### 3.4 Heroes off screen

Only two situations leave a hero outside the window: a scripted walk with `G.$506 = 1` (walk in from X -48, the Episode 1 opening,
walk off after a boss), and the first frames after a poke. Nothing else lets him be off screen, so there is no "teleport to the
leader" or damage rule in the game. A hero who stays far behind simply stops the camera (2.4).


## 4. Waves, locks and the GO

### 4.1 How a wave lock starts

1. The camera reaches the event X (`camX >= X`) and loads the event. In a mode 4 event (mode 2 in stage 2) the camera snaps to X and the
   lock byte `G.$7B4` becomes 1. The first frame of the lock clears `go`.
2. The spawn list 2 record with the same trigger creates a wave controller (pool B class 15, variant = wave number) in the same frame;
   the controller runs its first group from the next pool B update. Only one controller is meant to run at a time. [R 0xFBF4, 0x7B336]
3. The wave trigger equals the lock X exactly for 54 of the 57 locks (47 mode 4 events and the 10 Episode 3 mode 2 events); the three
   without a wave are the boss locks of Episode 3 (X 0x2D00), Episode 7 area 3 (X 0x450) and Episode 8 area 3 (X 0x1120). Area end
   events (modes 1, 8, 9, 0x12) have a wave record at their X when the area ends with a wave (type 2 or 4); the stage end events
   (mode 6, 0x10, 0x13) have none.
   Measured lock to wave end (all enemies killed at once by the probe): 86 gf for Episode 1 wave 2 [M e1a1n: lock gf 1269, released 1355],
   206 gf for the Episode 4 wave 0 with four bikers walking in [M e4a0n], 648 gf for Episode 6 wave 2 with ten spawns [M e6a0n].

### 4.2 The wave controller (0x7B336), short form of `re/specs/enemy/SPEC.md` 8.2

- A wave is a list of groups. For a group the controller waits until the alive count `G.$4F3` is at most the threshold of the player
  count (`thr1P`, `thr2P`, `thr3P`; 99 means no wait), then spawns the group's records, each after its own delay (a record whose
  conditions (player count, level) fail is skipped and does not stop the group). After the last group it waits for the final
  threshold and ends.
- `G.$4F3` = pool A objects alive in the last census with `+$7D >= 0`. Dinosaurs and bosses use `+$7D = 0xFF` and are not counted.
- **Group budget:** every group has 7200 frames (`G.$50C`). When it runs out the next group starts anyway; when it runs out before
  the final threshold, `G.$50E = 2` (kill all, no score) is set for 5 frames (0x1E down to 0x19) and every enemy still alive dies. [R]
- When the last threshold is met the controller waits a fixed 30 game frames (0x1E; kill-all is cleared at frame 5 of them), then
  the end test below. [R 0x7B464]
- The end waits until every present hero is in the normal ground state (`+$04` word = 0x0200: idle, walking, running), so a hero who
  is jumping, hurt or holding a weapon swing delays the GO by a few frames.
- Time commands in a wave (`set time`, `add time`) run as records of a group; the final group of boss area waves is `[99,99,99]`
  followed by the time set (3:50) and end type 4.

### 4.3 End types and who gives `go`

| End type | What the controller does at the end | What then releases the camera |
| --- | --- | --- |
| 0 (GO) | sets `go` (G.$7A9) at once, sets the leader, controller state 6 (frees itself) | the camera: lock mode ends, mode = next mode (usually follow) |
| 2 (walk on) | sets the leader (rightmost hero, 0x1E5F6), calls 0x1D95E (end script of this stage and area, section 5), controller freed | the **script**: its first GO record (action 0x2C) releases the wave lock, and a later GO record releases the area end event (mode 1, 8, 9 or 0x12) |
| 4 (boss area) | sets the leader, calls 0x1DAC4 (boss area script), controller freed | the script's GO record releases the lock; the stage end event (mode 6 and so on) holds the camera for the boss |

Other sources of `go`: script actions 0x2C and 0x36, pool A class 97 (0x69FA4, Episode 8 ending actors). Measured: in the poked run of
the Episode 1 door script the wave lock ended exactly 96 gf after the script start and the area end lock at +374 gf [M l_e1a0_k2,
e1a0: gf 1238 start, 1334 lock released, 1612 mode 1 lock, 1678 go]. Waves of type 0 whose final threshold is 99 end the moment the
last group is spawned (plus the fixed 30 frame end delay of 4.2): Episode 4 wave 2, Episode 5 waves 2 and 5, Episode 6 waves 0 and 5,
Episode 7 waves 1, 4, 6, 7, 8 and 11, Episode 8 wave 5. Their lock is released at once and they mostly only set the time.

**Warning for the port (and for probes):** the `jump` helper of the probe keeps the wave controllers of the records between
`CAMX - 0x400` and `CAMX`, and such a stale controller can send a `go` at any time (it sent one that opened the Episode 1 window
early in `e1a1n`/`hallwin`). This is a probe artifact, not a game rule; the natural run `n1a1` shows the real order.

### 4.4 The GO picture and the idle hint (pool B class 20, handler 0x7BB68, sound 0x3E)

One object of class 20 variant 0 is created at trigger 0 of list 1 in 17 of the 23 areas. The sub-areas (Episode 2 area 1, Episode 5 area 1,
Episode 6 area 2, Episode 8 area 3) have none in their list but keep the object of the area before (the sub-area switch does not clear pool B
[I]); the two full areas without one are Episode 6 area 1 and Episode 7 area 1 (the lift), so they have **no idle hint**. Variant 2 is
created by the script action 0x46 (section 5.3). [R 0x7BB68; M go1, go2, l_e1a0_k2, n1a1]

| Item | Value |
| --- | --- |
| Picture | 5 columns by 4 rows of 8x8 scroll1 tiles (a red "GO" burst) at scroll1 cells column 36, rows 46 to 49, palette 10 (7 colours of the scroll1 bank rewritten when a blink starts); screen about x 288 to 327, y 96 to 127 |
| One blink | drawn at counter 35, sound 0x3E at that moment, erased at counter 5, so 30 frames on, 5 frames off |
| Blinks | 3 in a row (counter `+$81` 2 down to -1): 105 frames |
| Variant 2 (forced) | spawned by action 0x46 at the first record of the area end scripts 0, 2, 5, 7, 11, 15, 16, 18, 19, 21, 22, 23; starts at once, blinks 3 times, frees itself; not suppressed by locks |
| Variant 0 (idle hint) counter | `G.$4FE`, starts at 600 (0x258) when the area starts. While the camera is locked (`G.$7B4 != 0`), a script runs (`G.$4DD != 0`) or the stage is clearing (`G.$4D5 != 0`) the counter is held at 180 (0xB4) every frame and the reference X (`+$A0`) is updated |
| Hint trigger | the counter reaches 0: if the camera moved less than 0x80 pixels since the last check (including 0), the 3 blinks start; if it moved 0x80 or more the counter is reloaded with 600 |
| After a hint | counter 600 again; so an idle hero sees a blink series at 180 + 105, then every 600 + 105 frames |
| Seen | Episode 1 hall, no key pressed: lock released at t = 450, `G.$4FE` is 180 at that frame and counts down by 1 per game frame, the burst is on screen from t = 630 to 735 (counter 0 during the blinks), then the counter reads 600 and counts down again [M go1; go2 with the hero walking right never showed a blink] |

There is no immediate GO arrow after a type 0 wave (no code sends the 0x3E sound anywhere else: the only two `sound 0x3E` calls are in
class 20). So the GO arrow that players expect right after every wave is not in this version of the game: the player sees the variant 2 blink
at area ends, and otherwise the idle hint after 3 seconds. [R full search of the ROM for the sound call; M go1]

### 4.5 Enemy count rules that end a wave, in one list

1. Alive count = pool A objects with `+$7D >= 0` in the previous census (an enemy that is dying but not yet in state 4 still counts).
2. Compare with the threshold of the player count (index = players - 1). 99 = no wait. End thresholds are 0 for most walk-on waves,
   4/5/5 and similar for the GO waves of Episode 1.
3. 7200 frame budget per group, kill-all 2 for 5 frames on a timeout.
4. A pool B record that cannot be spawned (pool full) is dropped without waiting; a pool A record that cannot spawn is skipped.
5. Waves without a record in the spawn list never run (Episode 2 waves 0, 1, 4; Episode 6 waves 1 and 3; Episode 7 wave 5).


## 5. Area changes and scene transitions

### 5.1 The two clocks of a transition

An area change always has two parts that run at the same time: the **camera event** (mode 1, 8, 9, 0x12; what the screen does: fade
or cut, loading the next map) and the **script** (what the heroes do: walk, kick, jump, wait). They meet at the `go` flag: the script
sets it with the action 0x2C, the camera mode waits for it. The wave controller starts the script (end type 2 or 4) or, at the end of
a boss, the boss controller does (section 8).

### 5.2 Camera side: what each kind of area change does, with measured times

| Camera mode | Used by | Sequence after `go` | Measured |
| --- | --- | --- | --- |
| 1 area end, fade | Episodes 1 (a0), 2 (a1), 4 (a0, a1), 6 (a0), 7 (a0, a1, a2), 8 (a0) | music fade out request (sound 0xFF07) at `go`; palette fade out 16 steps of 1 vblank; the frame after the fade the area index changes and 0xACD0 runs (0xAA0E: heroes placed from table 0x4E56, start script 0x1DA10, pools A and B cleared, items told to leave, rank, stage time, floor table, spawn lists reset; then area music 0xA55A, HUD redraw, player count, rank speed +2); palettes loaded at brightness 0; fade in 16 steps | `go` to new area 16 gf; fade out 16 vblanks (video frames t = 494 to 509 in e4a0n); fade in 16 vblanks starting 3 to 4 video frames after the load; for script 0 (END right after the GO) the heroes are free again 2 gf after `go` while the fade is still running |
| 9 area end, slow fade | Episode 8 area 1 | same as mode 1 (the delay written by the mode is overwritten, so the fade is the same 16 steps) | e8a1n: lock at gf 2009, fade out at 2024, area at 2026 |
| 0x12 area change, no fade | Episode 1 area 1 to 2 | the frame `go` is seen: area index changes, 0xACD0 runs, palettes at brightness 0, then 0x892C sets full brightness | **hard cut**: brightness 15 to 0 at the load frame and back to 15 two to four video frames later [M n1a1: t 1452 to 1454, e1a1n: t 406 to 410] |
| 8 sub-area switch | Episodes 2 (a0 to a1), 5 (a0 to a1), 6 (a1 to a2), 8 (a2 to a3) | `lock` and camX fixed until `go`; then area + 1, OBJ palettes only (0x8E4C), stage time of the new area set, floor table, spawn lists reset (0xAAE2), player count (0xACE0), music object class 96 | **no fade, no hero placement, no start script**; the new area's time budget appears at once (Ep2 3:12 to 3:20 [M e2a0n]) |
| 6 / 0x10 / 0x13 stage end | boss areas | camera frozen; on `go` sets `G.$4D4 = 1`; mode 0x13 (Episode 7) also fades out the three scroll banks (the HUD layer stays) | `G.$4D4` is also set by the boss death (0xACEE) |

Frames, not the old plan's guesses: **a fade is always 16 steps of one vblank** (`fade_task` resets the delay to 1) [R spec 2.4; M
bri 15 to 0 in 16 video frames in e1a0, e4a0n, e2a1n].

### 5.3 The script side: scripted control

State per hero (object offsets) [R 0x1DB0C, 0x1DB5E]:

| Offset | Meaning |
| --- | --- |
| `+$C0` | leader flag (the rightmost present hero when the script was started, by 0x1E5F6; ties go to the higher player number) |
| `+$D0` | copy of `G.$4DD` made when this hero started his script (0 = not started: the player update at 0x189D6 compares and calls 0x1DB0C) |
| `+$D1` | script state (always 0 in practice) |
| `+$D2`, `+$D3` | current action id; step inside the action (0, then 2 after first run) |
| `+$D4` (long), `+$D8` (word) | script pointer; timer (counts game frames; 0xFFFF = "run the action again next frame") |
| `+$DA`, `+$DB`, `+$DC`, `+$DE` | walk rank (0 = front spot), walk step counter, target X, target Z |

Globals: `G.$4DD` scripted mask (bit i = hero i is under script; the script END clears the bit, `G.$4DD = 0` means all free),
`G.$4DE` script id, `G.$4DF` arrival bits, `G.$4EC..4EE` taken target slots, `G.$4F0`, `G.$4FD`, `G.$506` (the three bytes of the
current record), `G.$514` (see below), `G.$503` pose lock, `G.$4D8` (set by action 0x28; only the debug build reads it), `G.$4E2` and
`G.$4E3` (intro and clear gates), `G.$4FC` (boss intro running).

**Step (0x1DB5E), per hero per frame, only while his `+$04` word is 0x0200 (ground state) and his bit is set in `G.$4DD`:**
`timer -= 1`; if it is not 0 run the current action again (actions with timer 0xFFFF are "wait until" loops). When it reaches 0:
clear `+$D2`, `+$D3`, fetch the next 6 byte record (`action`, `G.$4F0`, `G.$4FD`, `G.$506`, `timer` word), keep the pointer; action
0x0A or 0x26 clears this hero's bit in `G.$4DD`; then run the action. The pad is read first (0x18952 copies the pad bytes `G.$77A4..`
into `+$A0`), and the actions overwrite it: walk and hold actions write the direction (1 right, 2 left, 4 toward the viewer,
8 away) and buttons (0x10 attack, 0x20 jump); wait actions (and most others) end with 0x1DEA6, which clears `+$A0`, `+$A8` and `+$BC`
of all three heroes; actions 0x1A, 0x1C and 0x3A leave the pad alone. [R]

Record flags: `G.$4F0 = 1` makes heroes use the floor only terrain test (0x12D4E: walls and obstacles ignored); `G.$506 = 1` turns off
the screen clamp, pickups and weapons (a jab only, 0x191C4), arms the window of Episode 1 (class 12 variant 4) and is tested by 0x1AE8,
0x1C7A, 0x5AAC, 0x19356. `G.$514` (from the area end tables) matters in the player update at 0x189xx: while a script runs (`G.$4DD != 0`) the held item type word (`+$B6`) is cleared
every frame when `G.$4D1 != 0` (a stage clear is running: any held item) or when the held item is a barrel (type 0x20A) and `G.$514 = 0`; with `G.$514 = 1`
the hero keeps a barrel through the walk. [R Ghidra decompilation of the player update; the port has the barrel part only (`player_normal`)]

**The three entry points:**

| Routine | Called by | What it does |
| --- | --- | --- |
| 0x1DA10 area start | 0xAA0E (every full area start, also the first area of an episode) | clears `+$D0` of the three heroes, `G.$4DD`, `G.$4DE`; `G.$4DE` = start id of this stage and area (table 0x1DA64); `G.$4DD = G.$4E4` (all chosen heroes); except in Episode 1 area 2 it also clears the leader flags |
| 0x1D95E area end | wave end type 2 (0x7B4F8) and the boss controller when the boss HP is 0 (0x7C0D8) | clears `+$D0` of the heroes, `G.$4DD`, `G.$4DE`; `G.$4DE` and `G.$514` = bytes of the table 0x1D9B0 for the stage and area; clears `G.$7A9` for id 5; `G.$4DD = G.$4E4`; clears -$70AA, -$70A9 |
| 0x1DAC4 boss area | wave end type 4 (0x7B50C, 0x7B53A) | like 0x1D95E but sets `G.$4FC = 1` and takes `G.$4DE` and `G.$514` from table 0x1DAFC by stage |

The tables (script ids; `/` separates areas 0 to 3):

| Stage (episode) | start ids (0x1DA64) | end ids (0x1D9B0, `G.$514` in brackets) | boss area id (0x1DAFC) |
| --- | --- | --- | --- |
| 0 (Ep 1) | 1 / 4 / 6 | 2 (0) / 5 (0) / 12 (0), the last is the boss death | 11 (1) |
| 1 (Ep 2) | 13 / (0) / 9 | 7 (0) / 15 (1) / 3 (0) | 11 (1) |
| 2 (Ep 3) | 8 | 3 (0) boss death | 0 (1) |
| 3 (Ep 4) | 14 / 9 / 9 | 0 (1) / 15 (1) / 3 (0) | 11 (1) |
| 4 (Ep 5) | 8 / (0) | 0 (1) / 3 (0) | 11 (1) |
| 5 (Ep 6) | 8 / 17 / (0) | 18 (1) / 0 (1) / 3 (0) | 11 (1) |
| 6 (Ep 7) | 8 / 25 / 9 / 9 | 16 (0) / 15 (1) / 19 (0) / 20 (0) | 11 (1) |
| 7 (Ep 8) | 8 / 25 / 25 / (0) | 21 (0) / 22 (0) / 18 (0) / 24 (0) | 23 (1) |

Ids in brackets are sub-areas: 0x1DA10 is not called for them. The script table is at 0x1E6FC (27 pointers, entry 26 repeats 0).

### 5.4 Actions (table 0x1DBD6, the id is the byte offset)

| Id | Routine | Does |
| --- | --- | --- |
| 0x00 | 0x1DEE0 | wait (clears pad inputs) |
| 0x02 | 0x1DEF8 | walk to the end targets (table 0x1EE6A by stage); leader gets spot 0, the others the free spots nearest to them; done when each is within 0xC x 0x18 of the spot (0x10 x 0x20 in stage 2); then face right |
| 0x04 | 0x1E200 | wait until every scripted hero is alive and on the ground, then set `G.$503 = 1`, sound 0x18 and start the pose state 0x18 (the 1UP pose, 0x19996) |
| 0x06 | 0x1E262 | once: `G.$4E3 = 1` (intro done / walk-off done); then wait while `G.$503 != 0` |
| 0x08 | 0x1E28C | leader presses attack (0x10), the others release everything |
| 0x0A | 0x1E2BA | end: clears `G.$4DD`, `G.$4E3`, `G.$4D8`, pad bytes |
| 0x0C | 0x1E2EE | special attack: sound 2, player routine 0x1968E |
| 0x0E | 0x1E062 | walk to the start targets (pointer table 0x1EEEA, 16 bytes per area) |
| 0x10 | 0x1E2FC | all heroes walk to one point (table 0x1E3F4: X, Z per stage and area); done when within 0xC x 0x18 |
| 0x12 | 0x1E4BA | hold right |
| 0x14 | 0x1E4C8 | walk to one of three fixed spots by player order (table 0x1E532: (176,72), (112,24), (144,48); one player: (144,48)) |
| 0x16 | 0x1E54A | jump kick: heroes jump right in order of their walk rank; the leader (Jack uses state 0x19720, the others jump with 0x21) then presses attack 10 frames later |
| 0x18 | 0x1E5C8 | hold right and jump (pad 0x21) |
| 0x1A | 0x1E5D6 | face right and enter the air state 0x1AF82 (the fall of Episode 1 area 2; the landing makes a shake of 1) |
| 0x1C | 0x1E5E2 | nothing (pad passes through) |
| 0x1E | 0x1E11E | walk to the lineup spots (table 0x1F10A, three spots per stage, sorted by current X so nobody crosses) |
| 0x20 | 0x1DEE2 | wait while `G.$4FC != 0` (the boss intro is running) |
| 0x22 | 0x1DED4 | caption: start a banner (pool B class 66, 0x86D2E) with number `G.$4FD` |
| 0x24 | 0x1DECC | create pool B class 21 (tile attribute change; Episode 1 area 1 start) |
| 0x26 | 0x1E2BE | end but keep `G.$4DD` (heroes stay "scripted" through the tally) |
| 0x28 | 0x1DEA0 | `G.$4D8 = 1` |
| 0x2A | 0x1DE6C | wait for floor items: while any pool E object of class 0, 2, 4, 5 or 9 lies around (classes of weapons, food, containers, treasure) wait the record time (120 or 180); when none is left wait 90 more frames |
| 0x2C | 0x1DE62 | `go` flag (`G.$7A9 = 1`) |
| 0x2E | 0x1DE44 | wait while `G.$4E2 != 0` |
| 0x30 | 0x1DE02 | wait until all present heroes are on the ground |
| 0x32 | 0x1DD4C | scripted jump: target X = X + table 0x1DD88 (0x100 for Episode 7 area 0, 0x40 and 0x120 for Episode 8 areas 0 and 1), same height; player state 0x34 (0x1BBCA) |
| 0x34 | 0x1DDC8 | wait until the scripted heroes are on the ground |
| 0x36 | 0x1E4B4 | `go` flag and hold right (every frame) |
| 0x38 | 0x1DE86 | like 0x2A but items of class 0 do not count; no item: wait 160 |
| 0x3A | 0x1DD3A | two small hops (player state 0x2A, 0x19EA6) |
| 0x3C | 0x1E474 | walk right until X >= 0xB80 |
| 0x3E | 0x1E2AA | leader pose: state 0x2C (effect at the chest, 40 frames, then a pose) |
| 0x40 | 0x1DD30 | spawn pool B class 11 (the smashed door of Episode 1 area 0, 0x789C8) |
| 0x42 | 0x1E650 | clear sequence of Episode 8: sound 0x23, `G.$516 = 1`, one pool A class 97 stand in per hero (present heroes copied from the players, absent ones placed at X 0x10E0), then wait while the boss's `+$A6` is set |
| 0x44 | 0x1DD2A | `G.$4E2 = 0` |
| 0x46 | 0x1DD0E | spawn pool B class 20 variant 2 (forced GO blink) |
| 0x48 | 0x1DCFE | `G.$4E2 = 1` when in a game |
| 0x4A | 0x1DC60 | barrier: wait until every scripted hero has reached this action, has HP > 0, and none is in the invulnerable state (`+$00 = 2`, set after a hit or a respawn; the probe's god mode keeps it at 2 and so blocks the barrier for ever) |
| 0x4C | 0x1DC42 | wait while `G.$518 != 0` (car mode) |
| 0x4E | 0x1DC28 | sound request with the code `G.$4FD` |
| 0x50 | 0x1DC38 | music fade out (0xA5EA) |

Actions marked "walk" share the code at 0x1DF10: every 4th frame the direction is recomputed (pad bits toward the target, a 4 px dead
zone), the arrival test is made, and a hero that arrived presses nothing and turns right.


### 5.5 The 26 scripts (table 0x1E6FC), in plain steps

Durations are record timers in game frames. "Items" = action 0x2A (wait up to the time for floor items, else 90). "Idle" = action 0x34.
`B20` = action 0x46 (forced GO blink). Measured totals are in 5.7 and section 6.

| Id | Kind | Used by | Steps |
| --- | --- | --- | --- |
| 0 | end / exit | Ep 4 a0, Ep 5 a0, Ep 6 a1 (area ends); Ep 3 boss area | B20, items (120), GO, END (the Ep 5 and Ep 6 sub-area switches wait for this GO) |
| 1 | start | Ep 1 a0 (opening) | 4D8, wait 610, special attack, 4E3, wait 30, END |
| 2 | end | Ep 1 a0 | B20, items (180), idle, GO (releases the wave lock), 4D8, walk to the door targets (X 0x650 / 0x628 / 0x600 at Z 268, 284, 252; up to 300), wait 20, **door smash (class 11)**, leader attack (506 = 1), wait 30, walk to one point (X 0x650, Z 268; 180), hold right 120, GO (area end), wait 300, END |
| 3 | boss death | Ep 2 a2, Ep 3 a0, Ep 4 a2, Ep 5 a1, Ep 6 a2 | act 0x38 (items, up to 240, else 160), barrier, wait 60, **pose**, wait 150, 4D8, **4E3** (tally starts), hold right 300, wait 5, END |
| 4 | start | Ep 1 a1 | 4D8, walk to the three arrival spots (0x14), wait 2, class 21 (tiles), wait 5, END |
| 5 | end | Ep 1 a1 | B20, items (180), idle, 4D8, wait 60, walk to the window targets (X 856, 776, 816 at Z 48, 48, 64; 300), caption 0x1B, wait 20, walk to one point (1E3F4), **jump kick (506 = 1 arms the window)**, GO, END |
| 6 | start | Ep 1 a2 | 4D8, fall state (0x1A), pad passes for 120 frames, END |
| 7 | end | Ep 2 a0 | B20, items (180), idle, GO, 4D8, wait 60, walk to the swamp targets (X 0x740, 0x750, 0x730 at Z 56, 80, 32; 330), wait 40, hold right and jump 120, hold right 100, END |
| 8 | start | Ep 3, 5, 6, 7, 8 a0 | 4E2 = 1, 4D8, wait until 4E2 is 0 (the map card finished), hold right 60, **4E3**, END |
| 9 | start | Ep 2 a2, Ep 4 a1, a2, Ep 7 a2, a3 | 4D8, hold right 60 (506 = 1), wait 30, END |
| 10 | (unused) | none | 4D8, wait 360, END |
| 11 | boss area | stages 0, 1, 3, 4, 5, 6 (0x1DAC4) | B20, items (180), idle, GO, 4D8, wait 60, walk to the lineup spots (0x1E), wait 6, wait until 4FC is 0 (the boss intro), END |
| 12 | boss death | Ep 1 a2 | like 3 but: pose, GO, wait 60, 4D8, **4E3**, wait 330, END that keeps `G.$4DD` (the heroes stay scripted through the tally) |
| 13 | start | Ep 2 a0 (Cadillac arrival) | 4E2 = 1, 4D8, wait 4E2 = 0, wait all idle, 10, hold right 180, **4E3**, END |
| 14 | start | Ep 4 a0 | 4E2 = 1, 4D8, wait 4E2 = 0, wait all idle, 5, wait 4FC = 0 (captions and explosion objects), **4E3**, END |
| 15 | end | Ep 2 a1, Ep 4 a1, Ep 7 a1 | B20, items (180), GO, wait 300, END |
| 16 | end | Ep 7 a0 | B20, items, 4D8, wait 60, GO (4F0 = 1, 506 = 1), walk to the lift pad targets (X 2656, 2688, 2720 at Z 72, 52, 32; 300), wait 30, scripted jump (+0x100 X; 90), GO, wait 180, END |
| 17 | start | Ep 6 a1 | 4D8 (506 = 1), wait 90, hold right 120, wait 30, END |
| 18 | end | Ep 6 a0 (Ep 8 a2 table row, never used) | B20, items, idle, GO, 4D8, wait 60, GO and hold right 300, GO, wait 60, END |
| 19 | end | Ep 7 a2 | B20, items, idle, GO, 4D8, walk to the targets (X 2048, 1984, 1920; 300), wait 20, leader attack, wait 60, walk to one point, hold right 120, GO, wait 180, END |
| 20 | boss death | Ep 7 a3 | like 3 (pose, wait 60, 4D8, **4E3**), then GO (the fade of mode 0x13), walk to the end targets (X 1488, 1520, 1456 at Z 24; 330), walk to one point, GO, two hops (180), wait 60, END |
| 21 | end | Ep 8 a0 | B20, items (180), wait 30, GO, 4D8, walk to the targets (X 1552, 1504, 1472 at Z 56; 480), wait 60, leader pose, wait 240, walk to one point, scripted jump (+0x40; 30), GO, wait 480, END |
| 22 | end | Ep 8 a1 | B20, items, wait 30, GO, 4D8, walk right to X 0xB80 (300), wait 30, scripted jump (+0x120; 90), wait 300, END |
| 23 | boss area | stage 7 (0x1DAC4) | B20, items (180), idle, wait 30, GO, walk to the targets (X 3808, 3776, 3744 at Z 344; 480), walk to one point (4352, 344; 480), walk to the lineup spots (480), wait until 4FC = 0, END |
| 24 | boss death | Ep 8 a3 | act 0x38, barrier, wait 30, pose, wait 60, **4E3**, walk to the end targets (X 4512, 4544, 4576 at Z 320, 304, 288; 480), wait 5, **clear sequence (0x42)**, wait 540, `G.$4E2 = 0` (ends the clear wait: the ending starts), wait 300, END |
| 25 | start | Ep 7 a1, Ep 8 a1, a2 | END (no walk in) |

The leader-only and rank-ordered details (who jumps first, who takes which spot) are in the code at 0x1DFCA, 0x1E07E, 0x1E13A and
0x1E54A; the port agent must read those listings, they are only summarised here.

### 5.6 Where each hero is placed, and where the scripts walk him

Start positions (`0x4E0A`, table 0x4E56: X, height Y, depth Z for P1 / P2 / P3). Sub-areas have no row (never placed). [R]

| Episode, area | P1 | P2 | P3 |
| --- | --- | --- | --- |
| 1 a0 | 1056, 0, 296 | 1040, 0, 312 | 1024, 0, 328 |
| 1 a1 | 56, 0, 64 | 36, 0, 48 | 16, 0, 56 |
| 1 a2 | 32, 544, 304 | 32, 544, 320 | 32, 544, 288 |
| 2 a0 | -48, 64, 64 | -48, 64, 80 | -48, 64, 48 |
| 2 a2 | -32, 0, 352 | -16, 0, 376 | -48, 0, 328 |
| 3 a0 | -48, 0, 64 | -32, 0, 80 | -64, 0, 48 |
| 4 a0 | -48, 0, 64 | -48, 0, 80 | -48, 0, 48 |
| 4 a1, a2 | -32, 0, 64 | -16, 0, 80 | -48, 0, 48 |
| 5 a0 | -48, 0, 576 | -32, 0, 592 | -64, 0, 560 |
| 6 a0 | -48, 0, 64 | -32, 0, 80 | -64, 0, 48 |
| 6 a1 | -48, 30, 280 | -32, 30, 304 | -64, 30, 256 |
| 7 a0 | -48, 0, 64 | -32, 0, 80 | -64, 0, 48 |
| 7 a1 (lift) | 128, 0, 320 | 128, 0, 320 | 128, 0, 320 (clamped to Z <= 0x9F at once) |
| 7 a2, a3 | -32, 0, 64 | -16, 0, 80 | -48, 0, 48 |
| 8 a0 | -48, 0, 56 | -32, 0, 80 | -64, 0, 32 |
| 8 a1 | 192, 288, 808 | 224, 288, 824 | 160, 288, 784 |
| 8 a2 | 128, 544, 312 | 96, 544, 336 | 64, 544, 288 |
| 8 a3 | (the row is missing: the table ends early and the three words are read from the code at 0x4FCE; harmless because area 3 is a sub-area) | | |

Measured: 3 players at the start of Episode 1 stand at those X values while the camera pans [M o1p3]; at the start of Episodes 2 and 4
they ride in the car at X -48 (state 0x432, height 64) [M o2p3, o4p3]; at Episode 8 area 0 they walk in from -48, -32, -64 and stop at
49, 66, 31 (X + 97 after 60 frames of hold right) [M o8p3]. Height 544 or 288 means the hero starts in the air and falls.

Walk target tables (X, Z; in the order of walk rank 0, 1, 2) used by actions 0x02, 0x0E, 0x10, 0x1E (see 5.5 for the scripts that use
them): end targets by stage (0x1EE6A): stage 0 (640,320) (704,304) (768,288); stage 1 (736,64) (800,48) (864,32); stage 3 (1248,64)
(1312,48) (1376,32); stage 4 (5344,64) (5408,48) (5472,32); stage 5 (4192,400) (4256,384) (4320,368); stage 6 (1488,24) (1520,24)
(1456,24); stage 7 (4512,320) (4544,304) (4576,288); stage 2 (19200,64) (19264,48) (19328,32) is never read in play. Start targets by
stage and area (0x1EEEA): stage 0: a0 (1616,268) (1576,284) (1536,252), a1 (856,48) (776,48) (816,64); stage 1: a0 and a1 (1856,56)
(1872,80) (1840,32); stage 5: a0 (4064,400) (4128,384) (4192,368); stage 6: a0 (2656,72) (2688,52) (2720,32), a2 (2048,72) (1984,88)
(1920,56); stage 7: a0 (1552,56) (1504,56) (1472,56), a2 and a3 (3808,344) (3776,344) (3744,344); other rows are the unused default
(640,304) (704,288) (768,272). Single points (0x1E3F4): stage 0 a0 (1616,268), a1 (856,48), a2 (944,256); stage 6 a2 (2048,72), a3
(1584,24); stage 7 a0 (1552,56), a2 and a3 (4352,344). Lineup spots (0x1F10A, front hero first): stage 0 (616,296) (568,312) (520,288);
stage 1 (776,72) (728,96) (680,48); stage 3 (1288,48) (1240,72) (1192,24); stage 4 (4712,320) (4664,344) (4616,296); stage 5 (4248,384)
(4200,400) (4152,368); stage 6 (1240,56) (1192,80) (1144,32); stage 7 (4520,312) (4472,336) (4424,288); stage 2 none. The Episode
1 hall arrival spots are in action 0x14 (5.4). [R, tables decoded with `re/out/d4c/` scripts]


### 5.7 Measured area changes (3 players, all enemies dead or killed at once)

`S` is the game frame where the end script starts (the wave end); times are game frames after `S`. "lock" is the area end camera lock,
"go" the frame the camera sees `go`, "area" the frame the new area index is loaded, "control" the frame the heroes are free again
(`G.$4DD = 0`). [M d4c-b1/e1a0, d4c-b2/e2a0n ... e8a2n, d4c-b4/n1a1, l_e7a0_k16; analysis `re/out/d4c/trans.py`]

| Change | End script | lock | go | area | control | of which after the area load | New area time |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Ep 1 a0 to a1 (door) | 2 | +374 | +440 | +456 | +544 | 88 (start script 4) | 2:20 |
| Ep 1 a1 to a2 (window) | 5 | before S | +258 | +258 | +383 | 125 (start script 6, the fall) | 1:20 |
| Ep 2 a0 to a1 (sub-area) | 7 | before S | none (camera mode 8) | +186 | +605 | no start script; the walk into the swamp goes on | 3:20 |
| Ep 2 a1 to a2 | 15 | before S | +171 | +199 | +293 | 94 (start script 9) | 1:20 |
| Ep 4 a0 to a1 | 0 | before S | +106 | +122 | +215 | 93 (9) | 4:00 |
| Ep 4 a1 to a2 | 15 | before S | +182 | +198 | +292 | 94 (9) | 2:50 |
| Ep 5 a0 to a1 (sub-area) | 0 | before S | none | +124 | +126 | 2 | 4:00 |
| Ep 6 a0 to a1 | 18 | +201 | +257 | +266 | +509 | 243 (start script 17) | 4:00 |
| Ep 6 a1 to a2 (sub-area) | 0 | before S | none | +124 | +126 | 2 | 1:20 |
| Ep 7 a0 to a1 (lift) | 16 | +266 | +154 first GO, 363 second | +363 | +366 | 3 (start script 25) | 2:50 |
| Ep 8 a0 to a1 | 21 | +484 | | +513 | +516 | 3 (25) | 4:00 |
| Ep 8 a1 to a2 | 22 | +231 (mode 9) | | +248 | +251 | 3 (25) | 4:00 |
| Ep 8 a2 to a3 (stage exit, sub-area) | 23 | before S | | +124 | +744 | the walk to the lair goes on | 4:00 |

Not measured (the game hung under `hold killall`, section 12): Episode 7 area 1 to 2 (lift end, script 15, expected like Episode 2 a1
to a2: about +200 to the area and +94 to control), area 2 to 3 (script 19), and the Episode 3 area (one area only).

Reading the table: after a wave end the first 90 to 100 frames are the forced GO blink and the wait for floor items (`items` waits up
to 180 frames if items or unbroken containers lie around: Episode 1 a0 +440 with the drums standing, +96 in the poked run with no items);
the walk takes up to 300 to 480 frames; the fade and load are 16 + 16 frames; the start script is 3 to 243 frames.

### 5.8 What heroes can and cannot do during a transition

While `G.$4DD` has the hero's bit set: [R `combat.rs`/`player*.rs` hooks of the port, 0x10546, 0x189D6; M]

| | Allowed | Not allowed |
| --- | --- | --- |
| Pad | read, then overwritten by every action that writes `+$A0` or clears inputs. Actions that leave the pad alone: 0x0C, 0x1A, 0x1C, 0x2A, 0x32, 0x38, 0x3A, 0x3E. In particular **during the item waits (0x2A, 0x38: 120 to 240 frames after a wave end and after a boss death) the pad works**, so the heroes can still walk and pick up the dropped food [R, I] | steering, attacking, jumping during walk, wait and pose actions |
| Enemies | nothing attacks the heroes: the enemy attack passes (0x115B0, 0x11CEC) and the hero versus hero pass (0x10664) and the prop pass 0x10DCC are skipped | |
| Heroes hitting | the hero attack passes still run: a scripted kick or a thrown body hurts enemies; score is not given in a scene with script id 1 (the Episode 1 opening) | pickups and weapons when `G.$506 = 1`; a carried barrel is dropped |
| Camera | follows normally; the clamp is off while `G.$506 = 1` | |
| Clock and rank | the stage clock is frozen while `G.$4DD != 0` or `G.$4D5 = 1` (measured: 1:12 stays 1:12 over the 108 frame script of e4a0n; 3:19 stays during the boss death scripts); rank ticks are off in the same cases (0x186CC) | |
| Joining and continue | not examined here (D4d) | |
| Other | a hero who dies in the last scripted frames is revived with 1 HP when `G.$4D4` is set (0x1BF82) [I] | |


### 5.9 Answers to the open items of D3e part 1 (classes 11, 12, 14, 96, the hall window and `G.$506`)

- **Which script an area end uses:** the table in 5.3 (stage and area to script id) and the script list in 5.5.
- **`G.$506`** is one of the three bytes of every script record (`G.$4F0`, `G.$4FD`, `G.$506`); it keeps its value until the next record loads.
  Readers: the hero clamp (0x4FCE), the pickup and weapon entry (0x191C4), the jump and landing code (0x1AE8, 0x1C7A, 0x5AAC, 0x19356) and the
  window object (0x78FFC). Outside scripts it is 0.
- **Hall window (class 12 variant 4):** armed when `G.$506 != 0`. In the natural run the script 5 record with `G.$506 = 1` is the jump kick
  (action 0x16): flags rose at gf 2553 (242 gf after the script start), `go` and the area change followed 16 gf later (gf 2569) [M n1a1]. Whether a
  normal hero could break it while a script has `G.$506 = 1` was not tested.
- **Class 11 (smashed door, Episode 1 area 0):** created by action 0x40 of script 2, 200 gf after the script start with no items around (288 gf in the
  natural run, after the item wait), at X 0x670 [M l_e1a0_k2 gf 1204, e1a0].
- **Class 12 variant 0 (hall double door):** the first record of wave 1; **class 14 variant 0** (building front hole): list 1 trigger 0 of area 2, it
  appears 4 gf after the area load [M n1a1 gf 2573]; **variant 4** (wooden door): wave 4 record.
- **Class 96 (music):** only after a camera mode 8 sub-area switch (5.2), see T12.
- **Class 20:** section 4.4. **Class 21:** action 0x24 of script 4 (hall arrival).

## 6. The opening scene of each episode

All numbers are measured with the probe from the first frame of the level (`jump N 0 at=start`; `t` = video frame, `gf` = game
frame, they agree here). The level starts after the story panel (Episodes 2 to 8) or after the character select (Episode 1). Control
means `G.$4DD` becomes 0. [M o1 to o8, o1p3, o2p3, o4p3, o8p3]

| Episode | Start script | First frames | Hero control at | Who runs it |
| --- | --- | --- | --- | --- |
| 1 | 1 | Black fade in; the camera pans from X 0 to 0x380 at 3 px per frame (mode 0x11, ends at gf 303); the heroes stand at X 0x420 (P1), 0x410, 0x400 (the clamp is off); a place caption (2513, east coast) from t = 40, birds, the bystander and six thugs (pool B 74 creates pool A 54, 55, 56), three caption lines of the boss at t about 240 to 600; the leader does his special attack at the end | **648** (the special finishes at about 666); `G.$4E3 = 1` at 615 starts the HUD slide (12 px per frame until `cam1X > 0x180`, 33 frames) and `G.$4D5` is cleared at 618 | script 1, intro task state 4, pool B 74, 66, 73, pool A 49, 50, 54, 55, 56 |
| 2 | 13 | map card and typed title (the card runs for the first 255 game frames); `G.$4E2 = 1` at 3, `G.$4D5 = 1` at 231, `G.$4E2 = 0` at 255 (card gone); the three heroes ride in the Cadillac at X -48, height 64 (player state 0x432, "wait in the vehicle") while the car (pool A 16 with parts 17) drives in from X -0x80; they jump out at t about 330 to 384; then script: wait all on the ground, 10, hold right 180; `G.$4E3 = 1` at 570 | **573** | script 13, intro task, pool A 16 |
| 3 | 8 | map card (255), then the heroes walk in from X -48 to 49 in 60 frames, thugs and bikers wait in the open desert | **321** | script 8 |
| 4 | 14 | map card; the heroes ride in the car (state 0x432 until t about 600), jump out at 639 to 693 and land at X 167 (P1); Jack's friends talk in two captions and an explosion goes off; the script waits for `G.$4FC` to clear (set by the opening objects, cleared at gf 888) | **894** | script 14, class 59 and 66 objects |
| 5 | 8 | map card, hero walks in (Z 576), a group of burning poachers at the right | **321** | script 8 |
| 6 | 8 | map card, hero walks in, wooden mine wall | **321** | script 8 |
| 7 | 8 | map card, hero walks in, thugs crouch on the roof | **321** | script 8 |
| 8 | 8 | map card, hero walks in (Z 56, 80, 32 for three heroes), lab tubes 01 to 03 | **321** | script 8 |

Details of the common card phase for Episodes 2 to 8 (the card itself belongs to D4d): `G.$4E2` is set to 1 by the script at the first
frame (action 0x48) and cleared by the intro task when its state 4 has wiped the map panel; the scripts of Episodes 3, 5 to 8 wait for
that, then hold right for 60 frames (0.75 of a second), set `G.$4E3 = 1` and end; the intro task state 4 sub 2 waits for `G.$4E3`,
redraws the HUD (0x162AE) and slides it in from the right, 12 px per frame until `cam1X > 0x3F0` (85 frames, the control is
back before the slide has finished) and clears `G.$4D5` in every slide frame. [R 0xCBAA to 0xCCFE; M o3, o5 to o8]

Heroes that are not present (1 or 2 players) have no object; with 2 or 3 players the third column of the start table is used by the
extra heroes and the walk or ride positions are the same shifted rows (o8p3: P1 stops at 49, P2 at 66, P3 at 31).

Captions of the openings are pool B class 66 objects (D3e 3.6); the car arrival of Episodes 2 and 4 and the master of Episode 1 are
described in `D3e-pool-b-inventory.md` 3.7 and in the pool A parts of D3a and D3b.


## 7. Special camera modes and effects

### 7.1 The car section of Episode 3 (camera mode 8 of stage 2, car state `G.$518`)

What starts it: the RADIO (pool E class 9, treasure id 12, 3000 points). Pick up code 0x8BD24 (only while `G.$50E = 0` and at least
3 pool B slots are free): creates **three pool B class 88 objects** (variant 0 = the car, variants 2 and 4 = two helper objects linked to
the car by `+$76`), sets `G.$4D5 = 1`, `G.$4DB = 1` (a flag that the flow also sets for the last boss and the ending; its meaning is unknown [U]), `G.$50E = 1` (kill-all, credit to
the taker, `G.$4E0`). [R 0x8BD24, 0x8BDDC; M car1]

Measured timeline (radio taken at t = 5 with Jack at camX 0x300; the car hit points are `+$6C`): [M car1]

| t | Event |
| --- | --- |
| 16 | kill-all (every enemy dies), `G.$4D5 = 1` (the clock stops; it stays frozen for the whole drive) |
| 16 to 72 | the car (HP 120, height 5, depth 0x30) drives in from X = camX - 0xA0 at 5.0 px per frame |
| 75 to 105 | it brakes (speed 1232, 992, ... 32 over 30 frames) and stops at X = camX + 0xC2 |
| 105 to 284 | the car waits (object states 6, 8, 0xA: 120, 60 and 45 frames); the heroes stand beside it |
| 284 to 332 | the heroes jump aboard: scripted jump state 0x34, apex height 76, landing height 5 on the car |
| 330 to 365 | the car accelerates by 64 per frame squared in 1/256 px (0.25 px per frame squared), then the normal drive state |
| 364 | `G.$518 = 1`, camera mode 8, `G.$4D5 = 0` (hero control, HUD free); by t = 380 the speed is 2816 = **11.0 px per frame** and stays there |

Camera mode 8 of stage 2 (0x22A68): while `G.$518 = 1` the camera chases the car: `camX += clamp(car.X - (camX + 0xC0), 8, 20)`
per frame. With `G.$518 = 2` it would copy the car's velocity and acceleration instead (`G.$518 = 2` was never reached in my runs;
the plan and `rank.rs` say the time over sets it; unverified). Camera events are not loaded in mode 8. [R 0x22A68; M camx 1299 at
t 400, 2399 at 500, 3499 at 600: 11.0 px per frame]

What ends it: the car's HP reaches 0 (enemies hit it; in car1 nobody defended the car and it went 120, 24, 12, 0 at t = 1800 to 1885,
about 500 frames after Hogg appeared at t = 1332; with `hp 1` poked at t = 700 it died at t = 1475 [M car3]). The car
object goes to state 4, `G.$518 = 0`, and the camera runs its end: velocities cleared, **mode = 3 (stage end lock, lock = 2) at the
current camX** (0x453A in car1, 0x3430 in car3). The fight continues on foot in that frame. Hogg (pool A class 31 with its part
class 32) is created by the spawn list when the camera passes X 0x2D00, also during the drive; he rides ahead of the camera at the
car speed. Waves in car mode: every wave controller ends at once with GO (`G.$518 != 0`); enemies that leave the screen are freed
(0x173C). The extra collision passes of car mode (0x121D8, 0x122D8, 0x12342, 0x12454), the car object states and the hero states
while riding belong to D3e part 2 (classes 88 to 90) and D3c (Hogg). Port: MISSING (all of it).

The Episode 3 map is 60 screens of 256 px = 15360 px wide, but the camera ran to X 17722 in car1: the map repeats, and the background kept
looking right (mesas, cacti) in the pictures at t = 1700 to 1900 [M car1]. The old note "one camera event, stage end at X 0x7FF0" in
`episode-3.md` is wrong: the table has 11 records.

Walk-off: Hogg's death uses script 3 (boss death, stage 2) like the other bosses.

### 7.2 The lift of Episode 7 (area 1, camera mode 5, scroll1 background)

- The area has a 2 x 56 screen map (camera starts at Y 0x3700 and goes up to 0), `G.$4DC = 1` (vertical scene: sprites drawn at camY 0,
  spawn lists and events trigger on Y, the hero clamp is fixed to the screen), and uses **scroll1 as a background** (the wooden cage and
  the painted floor numbers; built by 0x205C6 from tables 0x138C48 and 0x15CFE8). The far layer (scroll3)
  scrolls by `camY / 16 - 0x60`. [R spec 4.5, 6.6, 7.1; asset `re/out/assets/stages/ep7_a1_scroll1_elevator_cage.png`]
- Mode 5 sub steps: sub 0 plays sound 0x86 and sets velocity -0x30 (1/256 px per frame), acceleration -6, shake request 2; sub 2 integrates
  until the velocity reaches -0x600 (6 px per frame); sub 4 moves 6 px per frame more and, when the camera is within 0x300 of the next
  event Y, reverses the acceleration; sub 6 brakes and, at Y 0, switches to mode 1 with sound 0x87. The stops are mode 4 events by Y
  (0x3000, 0x1E00, 0x1400, 0x800) with `next = 5`: **after each lock the ride restarts from rest** (sub 0 again: sound 0x86, shake 2).
  [R spec 6.6]
- Measured (3 players punching, no kill-all): ride Y 0x36FF to 0x3000 in 532 gf; locks at gf 1830 (782 gf), 3628 (402), 4696 (678), 6136
  (350); rides of 1016, 666, 762 and 600 gf between them; end lock (mode 1, Y 0) at gf 7086; the heroes ride at X 128 to 287, depth 32 to 79.
  Start speed after a lock: camY 12288, 12287, 12285, 12283, 12281, 12278, 12275 on 4 frame steps (the restart from rest). [M e7a1n,
  lift2]
- Riders: list 2 records by Y: poacher and Ferris at Y 0x3700 (X 0xE8, 0x110), Ferris at 0x2F00, 0x2E00 (2P+), 0x2D00 (3P), 0x2C00,
  0x1C00, Blade/Razor at 0x1B00 (2P+) and 0x1900, Hammer at 0x600; waves 4, 6, 7, 8 at the four stops (7, 8, 5, 5 spawns, all end
  type 0 with final threshold 99) and wave 9 at Y 0 (type 2, no spawns). Pool B 64 (four red pillar pieces) and 57 are created at the
  area start (trigger Y 0x4000). Time sets: 3:50 at wave 4, 1:50 at waves 6 and 7, 3:50 at wave 8 (the start of the area is 2:50).
- The probe hung (the game frame counter stopped) twice in Episode 7 with `hold 0xFF850E 1 2` (kill-all forced every frame): in the lift
  at t about 1000 and in area 2 right after a script start. Do not use kill-all holds in Episode 7. [M e7a1, l_e7a2_k19]
- Port: camera mode 5 DONE; scroll1 background, class 57 and 64 props, the by-Y spawn triggers in the vertical scene (`spawn_trigger_reached`
  handles `vertical_scene`) and the enemy off-screen Z + Y test are the open parts.

### 7.3 Row scroll of Episode 3 (0x2335A, struct G.$7780)

`video_ctrl` bit 0 turns the CPS row scroll on in stage 2 only. Every frame the task writes 32 words (screen lines 128 to 159 of the
scroll2 row table, `other[0x90..0xAF]`) from the pseudo code of `re/specs/stage/SPEC.md` 7.3. **Verified against MAME at camera X 0x300,
0x1000 and 0x2400: all 32 words equal the formula** [M rows_300, rows_1000, rows_2400]. Example at camX 0x300 (768): the words run from 0x80
at line 128 up by 12 per line to 0xF4 at line 159 (`camX / 64` per line, taken mod 256 once 0x180 is reached). At camX 0x1000 the step
is 0x40 per line. The upper lines (0 to 127) and the area below are not shifted; `other` is a 1024 word table indexed by `(line + 16) & 0x3FF`.
Port: MISSING; the renderer needs a per line horizontal offset for scroll2 (the stage handoff suggests a clip per row range).

### 7.4 Screen shake (0x222D6, struct G.$755A)

Not a camera move: the task writes the vblank shadow of the scroll2 Y register (`G.$868 = camY + offset`) or, in Episode 7 area 1, the
scroll1 Y register (`G.$860 = G.$6C8 + offset`); sprites and the camera do not move. A request is a byte write of the strength to `G.$755C`
(the largest request of a frame wins; a stronger request restarts the shake with that amplitude).

**Sequence (verified for strengths 1 to 4 [M shake1 to shake4]):** the displayed offset is `-s` for 1 frame, `+s` for 3, `-(s-1)` for 3,
`+(s-1)` for 3 and so on down to `+1` for 3 frames, then 0. Total length **6 s - 2 frames** (4, 10, 16, 22). In code: state 0 takes the
request, state 2 flips the sign every 3 frames (timer 2), and each time the sign becomes negative the amplitude and the strength drop by 1;
at strength 0 it stops and copies the scroll1 Y shadow to `G.$7B8`.

Requesters (29 writes in the ROM; the dead copy of the old player code at 0x2000 to 0x3A00 is left out): [R search of `$755C(a5)`]

| Strength | Who |
| --- | --- |
| 1 | hero landing from a fall (0x1B012, the Episode 1 area 2 fall and every high jump); pool A class 24 Shivat (0x4938C); Fessenden class 84 (0x63686); the small-enemy branch of 0x2AB9E |
| 2 | camera mode 5 restart (0x21128, lift); pool E class 5 containers breaking (0x290FE, 0x29234, 0x29364, 0x293C2, 0x2940A); pool A class 10 Hammer / Elmer / Wrench stomp (0x3C542); pool C class 6 (0x92DBC) |
| 3 | hero hard landing, player state 0x26 (0x19E70, sound 0x9B9E); pool E class 1 explosions (0x27E7C, 0x27FCE, 0x28038); pool E class 8 bombs (0x29F90); pool A class 38 Mack (0x5379A); Fessenden (0x636DA, 0x63760); enemy falling body, big enemies (0x2AB9E: 3 for a big one, else 1; port `enemy_shake_request`) |
| 4 | pool A class 22 brontosaurus leg (0x47EFE); class 30 (0x4FB9E); the Butcher class 61 (0x5925E, 0x593C0); pool B class 82 and 85 debris showers (0x8ACD2, 0x8B38E) |
| variable | Shivat (0x49E42, strength from `d0`), 0x2AB9E (above) |

Port: the request side exists in `class10_moves.rs`, `container.rs`, `blast.rs`, `enemy_combat.rs` and `camera.rs` (mode 5 calls
`cam_shake(2)`), but `screen_shake()` is empty and `present.rs` has no offset: the shake never shows. The hero landing (1) and slam (3)
requests are in the player port TODO list.

### 7.5 Vertical areas

Two areas move the camera in Y a lot: **Episode 1 area 2** (starts at camY 0x300 and follows the falling heroes down to 0x100 with mode 7,
124 gf, then is an ordinary horizontal area; the player clamp is the normal one, `G.$4DC = 0`) and **Episode 7 area 1** (the lift,
`G.$4DC = 1`). Diagonal areas (Episode 2 area 2, Episode 5 area 1, Episode 6 area 2) are slopes of the same horizontal camera, not
vertical scenes.


## 8. The end of an episode: from the boss defeat to the next level

### 8.1 The chain (code and a real kill of Vice T.)

| Step | What happens | Where | Measured on Vice (game frames from HP 0) |
| --- | --- | --- | --- |
| 1 | The boss routine calls 0xACEE: `G.$4D4 = 1`, task period `G.$4D6 = 4` (**slow motion**: the game task runs every 4th video frame), `G.$4D5 = 1`, `G.$4E0` = the boss, `G.$50E = 1` (kill-all, credit to the killer), `G.$4D1 = 2`, rank speed +1, the HUD blocks of absent players are cleared; stage 7 also sets `G.$4D7 = G.$4DB = 1` | 0xACEE | 0 (gf 2669) |
| 2 | The reinforcement controller (pool B class 22, `+$AA` = boss) sees boss `+$7D = 0xFF` and HP 0: routine 6, `go` cleared, **calls 0x1D95E**: the boss death script of the stage starts (ids in 5.3) | 0x7BF76, 0x7C0CE | +3 (script 12 started at gf 2672) |
| 3 | `G.$4D1 = 2` waits until the boss object's state byte `+$04` is 4 or more (his death animation), then normal speed (`G.$4D6 = 1`), `G.$508 = 1800`, `G.$4D1 = 4` | 0xAB44 | +94 (368 video frames of slow motion) |
| 4 | The script: wait for floor items (up to 240, else 160), barrier, wait 60, **pose** (sound 0x18, state 0x18), wait 150 (60 for script 12), then action 0x06 sets **`G.$4E3`** | scripts 3, 12, 20, 24 | +372 after the script start (369 on Vice) |
| 5 | `G.$4D1 = 4`: `G.$508 -= 1`; when it goes negative (1800 frames) or `G.$4E3 != 0`: starts the **clear task 0xB97E** (slot 0x30), `G.$4E2 = 1`, `G.$508 = 1800` (3600 in Episode 8), `G.$4D1 = 6` | 0xAB66 | +372 (gf 3041) |
| 6 | The tally (D4d): portrait, vitality, enemies, final blow, best player, the TOTAL count down into the score | 0xB97E | 951 game frames on Vice (944 to 951; 744 or 942 to 945 in the poked runs, it depends on the counted values) |
| 7 | `G.$4D1 = 6`: `G.$508 -= 1`; when negative or `G.$4E2 = 0` (the clear task ends, or script action 0x44 in Episode 8): stage < 7: `G.$4D9 = (stage + 1) & 7`, area 0, mode 8; stage 7: mode 0x1C (the ending), `G.$4D7 = G.$4DB = G.$507 = 1` | 0xABA8 | gf 3975 |
| 8 | Mode 8 (0xD898): the story panel of the next episode, blocking, the game frame counter stands still | D4d | Episode 2 306, 3 251, 4 309, 5 440, 6 368, 7 368, 8 315 video frames [M f_ep1 to f_ep7] |
| 9 | Mode 4 for one frame, mode 0xC, 0xAC4E stage start (clear screen, intro task 0xB9BE, rank per stage, music), the opening of section 6 | 0xAC4E | Episode 2 control 573 gf later |

Total from the Vice kill to control of Episode 2: 372 + 945 + 306 (panel, video frames) + 573 = about 2200 frames, 36 seconds. [M d4c-b3/b1c, f_ep1]

What the heroes do: they keep their control for up to 240 frames while the script waits for floor items (action 0x38 leaves the pad alone;
nothing can hurt them, the enemy passes are off), then the barrier (all must be out of the invulnerable blink), a 60 frame wait, the pose (the
same animation as the extra life pose: Jack flexes, Mustapha somersaults, Hannah and Mess pose; 33 to 183 ticks), 150 frames of waiting
and then (script 3) they hold right for 300 frames with the screen clamp off, walking out of the picture while the tally runs. Script 12
(Episode 1) lets them stand. Script 20 (Episode 7) releases the stage end camera (fade out of the lab, mode 0x13), walks to the end targets
and hops twice. Script 24 (Episode 8) walks them to the lair targets and spawns the four stand-in actors (pool A class 97) and sound 0x23
for the ending. The music of the victory and the tally is D4e (action 0x04 sends sound 0x18 at the pose).

### 8.2 Differences between the episodes

| Episode | Death script | Frames to `G.$4E3` (no items) | Remarks |
| --- | --- | --- | --- |
| 1 | 12 | 366 | heroes stand; keeps `G.$4DD` through the tally and the next episode's start script resets it |
| 2, 3, 4, 5, 6 | 3 | 372 | hold right 300 during the tally |
| 7 | 20 | 285 | fade out (mode 0x13) after the pose; hops |
| 8 | 24 | 255 to the tally start, but the tally wait ends only when the script clears `G.$4E2` (about 540 + 5 frames after the clear sequence starts) or after 3600 frames | in the poked run the wait ran out at 3597 frames and then mode 0x1C started; the real boss's `+$A6` flag decides when the clear sequence ends [open item] |

Port: step 1 is `flow_boss_defeated` and steps 3, 5 and 7 are `flow_ingame_switch` (DONE); step 2 (class 22 controller, 0x1D95E) and
the whole script of step 4 are MISSING, so today the port has no walk off, no pose and the tally is a 120 frame wait.


## 9. Port state today, item by item

| Item | State | File |
| --- | --- | --- |
| Standard camera: modes, event loader, follow, vertical follow, locks, `go`, push line, extents | DONE | `camera.rs` |
| Area end and sub-area hooks 0xACD0, 0xACE0, 0xAAE2, 0x162AE, 0x12BE4, 0xA5EA | DONE (`flow.rs`) except that hook 0x9236C makes a class 96 that is freed at once | `flow.rs` |
| Fades (16 steps of 1 vblank) | DONE | `palette.rs` |
| Hard cut of mode 0x12 (2 to 4 black frames) | PARTIAL (the camera mode runs, the brightness dip is not checked) | `camera.rs`, `palette.rs` |
| Player start positions 0x4E0A and the drop in 0x62D6 | DONE | `flow.rs`, `player.rs` |
| Wave controller (groups, thresholds, 7200 budget, 30 frame end delay, all-heroes-normal test, end types) | DONE | `prop.rs` |
| Leader 0x1E5F6 | DONE | `prop.rs` |
| Area end script start 0x1D95E, boss area script 0x1DAC4 | **STUB (TEMP)**: `prop_area_walk` remembers the kind, `prop_area_walk_fallback` gives `go` as soon as the camera is locked | `prop.rs`, `spawn.rs` (`area_walk`, `b4de`) |
| Area start script 0x1DA10 | MISSING (comment in `flow_area_reset`) | `flow.rs` |
| Scripted control 0x1DB0C, 0x1DB5E, 41 actions, 26 scripts and tables | MISSING (`player_frame` only copies `G.$4DD` to `bd0`) | `player.rs` |
| Consumers of the script flags: `scripted` (enemy attack passes off, no regrab, no player vs player, heavy object drop, no score in script 1), `no_clamp` (G.$506), `floor_only` (G.$4F0) | DONE as readers, **nothing sets them** | `combat.rs`, `player*.rs`, `mod.rs` |
| Player states for scripts: 0x2A hops, 0x2C pose, 0x32 vehicle wait, 0x34 scripted jump, run state 6, 0x18 pose | MISSING (D2a section 16) | `player.rs` |
| Intro task 0xB9BE states 4 and 6 (HUD slide) | PARTIAL: the slide runs at once (TEMP, `flow_player_scripts_ported()` is false); `G.$4E2` is not set; states 0 to 2 (map card, title) MISSING | `flow.rs` |
| Boss defeat 0xACEE and the `G.$4D1` switch (steps 1, 3, 5, 7 of 8.1) | DONE | `flow.rs` |
| Boss controller class 22 and its call of 0x1D95E | MISSING | `prop.rs` |
| Clear task 0xB97E (tally) | STUB (120 frame wait, clears `G.$4E2`) | `flow.rs` (D4d) |
| Mode 8 story panel, mode 0x1C ending | STUB | `flow.rs` (D4d) |
| Pool B class 20 (GO blink and idle hint) | MISSING (freed as an unported class) | `prop.rs` |
| Pool B class 31 clock, 96 music, 11, 12, 14, 21, 66, 74 | MISSING (D3e) | `prop.rs` |
| Stage 2 camera controller 0x22398, mode 8 car drive, the Episode 3 event table | MISSING | `camera.rs` |
| Car: radio 0x8BD24, class 88 to 90, `G.$518` and `G.$516`, the extra collision passes | MISSING | `item/`, `prop.rs`, `combat.rs` |
| Row scroll 0x2335A | MISSING | `camera.rs`, `render.rs` |
| Lift background 0x205C6 and class 57, 64 | MISSING | `camera.rs`, `prop.rs` |
| Screen shake 0x222D6 | STUB (requests stored, nothing drawn) | `camera.rs`, `present.rs` |
| Player clamp 0x4FCE | DONE | `player.rs` |
| Terrain and walls | DONE (not part of this task) | `terrain.rs` |

## 10. Acceptance scenarios (probe segments and what must match)

All segments run with `PR_INPUT=file re/harness/probe/probe.sh NAME` in one MAME process. The generators that produced the batches are
`re/out/d4c/gen_batch1.py` to `gen_batch4.py` (git-ignored; the main session may copy them to `re/harness/probe/`). The analysis scripts
are `ev.py` (event list with game frames), `trans.py` (section 5.7 table), `tl.py`, `tl2.py`, `bounds.py`, `wsum.py`. The port is compared
at the same game frame (`gf`), with the same keys. All frames below are game frames unless said.

| # | Segment (core lines) | What must match |
| --- | --- | --- |
| A1 | `jump 1 0 at=start`, `frames 700`, `watch 3 G ...`, `watch 3 P1 x y z st +0xD2.b` | camera mode 0x11 and camX + 3 per frame from 0 to 896 (mode 0 from frame 303); P1 stays at (1056, 0, 296); `G.$506 = 1` from frame 3 to 614; `G.$4E3 = 1` at 615; `G.$4D5 = 0` at 618; `G.$4DD = 0` at 648; P1 state 0x0E (special) from 615 |
| A2 | `jump N 0 at=start`, N = 3, 5, 6, 7, 8, `frames 400` | `G.$4E2 = 1` at 3, `G.$4D5 = 1` at 231, `G.$4E2 = 0` at 255, `G.$4F0 = 1` at 258, `G.$4E3 = 1` at 318, `G.$4DD = 0` at 321; P1 X rises from -48 (frame 258) to 49 (reached by frame 320), Z 64 (Ep 3, 6, 7), 576 (Ep 5), 56 (Ep 8); 3 players: 49, 66, 31 (Ep 8) |
| A3 | `jump 2 0 at=start` and `jump 4 0 at=start` | control at 573 (Ep 2: vehicle state 0x432 until 330, jump out 330 to 384, `G.$4E3` at 570) and 894 (Ep 4: `G.$4FC` set at 3, cleared at 888, `G.$4E3` at 891) |
| A4 | the 13 area changes of section 5.7 (jump lines in `gen_batch2.py` `E`, `gen_batch4.py` `launch`, natural run `n1a1`) | the table: end script start, lock, go, area load, control in game frames; fade out 16 and in 16 video frames; hard cut of 2 to 4 black frames for Ep 1 a1 to a2; new area time (2:20, 1:20, 3:20, 1:20, 4:00, 2:50, 4:00, 4:00, 1:20, 2:50, 4:00, 4:00, 4:00) |
| A5 | `jump 2 0 char=jack,hannah,mess` with only P1 holding R | camera stops at the leftmost hero X - 16 (383) and P1 at camX + 0x16F (751) for 900 frames |
| A6 | `jump 1 0 0x200`, `5 poke 0xFFB27C 2 0x7FF`, hold R | P1 X = camX(previous frame) + 0x16F each frame, camera chases at 6 px per frame |
| A7 | `jump 1 1`, no keys, `hold 0xFF850E 1 2` (kill-all), `watch 5 B:20 ...` | lock released at t = 450; `G.$4FE` 180 then 1 per frame; class 20 blinks from t = 630 for 105 frames with the GO burst on screen; counter 600 afterwards. With R held the counter never reaches 0 |
| A8 | Vice kill (`b1c` of `gen_batch3.py`: `jump 1 2 0xA0`, killall pulses at 30 to 900, hero walks right and mashes attack, `god off` and `objset A:18 hp 1` at t = 1500) | HP 0 at 0, script 12 at +3, normal speed at +94, `G.$4E3` at +372, tally 944 to 951, mode 8 at gf 3975, story panel 306 video frames, Episode 2 control 573 gf after the level start |
| A9 | poked end flow of all 8 episodes (`f_ep1` to `f_ep8`: the script id, `G.$4DD = 7`, `G.$4D4 = G.$4D5 = 1`, `G.$4D1 = 4`, `G.$508 = 0x708`) | `G.$4E3` at +366, 372, 372, 372, 372, 372, 285, 255 (kinds 12, 3, 3, 3, 3, 3, 20, 24); the story panel lengths of 8.1; the next opening control frames of A2 and A3; Episode 8 reaches mode 0x1C only after the 3600 frame timer in the poked run |
| A10 | `jump 3 0 0x300`, `spawn E 9 var=0xC x=+0x60`, `5-6:A`, `watch 5 B:88 ...`, `watch 4 G car camx cammode` | car created at 16 (X camX - 0xA0, HP 120, 5 px per frame); stop at 105 at camX + 0xC2; boarding jump 284 to 332; `G.$518 = 1` and mode 8 at 364; 11.0 px per frame from 380; clock frozen; with `objset B:88 hp 1`: car dead and mode 3 within 800 frames |
| A11 | `jump 3 0` with three heroes walking and punching, `hold 0xFF850E 1 2` | locks at camX 0x300 and 0x500 last 32 game frames each (mode 2 of the stage 2 controller, then mode 0) |
| A12 | `jump 7 1` with three heroes punching (no kill-all) | stops at camY 0x3000, 0x1E00, 0x1400, 0x800 and the end lock at 0 (mode 1) at gf offsets 532, 2330, 3398, 4838, 5788 from the start of control; the ride restarts from rest after each lock (camY 12288, 12287, 12285 ...) |
| A13 | `jump 3 0 0x300 / 0x1000 / 0x2400`, `20 peek 0x915900 96` | 32 row scroll words equal the pseudo code of the stage spec |
| A14 | `jump 1 0`, `10 poke 0xFFF55C 1 S` for S = 1 to 4, `watch 1 G @0xFF8868.w camy` | offsets -S (1 frame), +S (3), -(S-1) (3), +(S-1) (3) ... +1 (3); length 6 S - 2 |
| A15 | 66 depth sweeps (`d*_*_*` of `gen_batch2.py`) | the front and back Z of 3.2 |
| A16 | unit test of the interpreter: run each of the 26 scripts with a scripted hero in state 0x0200 and no items, count frames to END | totals of 5.5 (script 9: 91 to 93 game frames to END with walking; script 25: 3) |
| A17 | scripted hooks of the combat passes | no enemy attack lands on a hero while `G.$4DD != 0`; heroes can still hit |

The first segments to port-check are A1, A2, A4, A7 and A8; they exercise the interpreter, the gating flags, the camera release and the
end chain.

## 11. Port task list for M2 and M3

Sizes in agent hours (1 task = one agent run, one commit). Prerequisites: M0 (episodes start), D2a states 0x2A, 0x2C, 0x32, 0x34 and the pose
0x18 (one extra task if the player port lacks them), D3e shared services (T-B0: tile writers, break sound), D4d (cards and tally).

| Task | Scope and routines | Files | Size |
| --- | --- | --- | --- |
| T-C1 scripted control | extract the script data (`dino-extract/src/script.rs`: 0x1E6FC scripts, 0x1DA64, 0x1D9B0, 0x1DAFC, 0x1EE6A, 0x1EEEA, 0x1E3F4, 0x1E532, 0x1F10A, 0x1DD88; the start table 0x4E56 is already in); `0x1DA10`, `0x1D95E`, `0x1DAC4`, `0x1DB0C`, `0x1DB5E`, the 41 actions, walk code 0x1DF10 and the slot assignment 0x1DFCA, 0x1E07E, 0x1E13A, 0x1E170, flags `G.$4F0`, `G.$506`, `G.$4FD`, `G.$514`, `G.$4DF`; remove the TEMP of `prop.rs` and `flow_player_scripts_ported`; the opening of Episode 1 first (script 1) | new `game/script.rs`, `dino-extract/src/script.rs`, `dino-assets/src/script.rs`, hooks in `flow.rs`, `prop.rs`, `player.rs` | 6 |
| T-C2 intro gating | `G.$4E2` and `G.$4E3` handshake, intro task states 4 and 6 exactly, HUD slide limit by stage (0x180 / 0x3F0); states 0 to 2 with D4d | `flow.rs` | 1.5 |
| T-C3 GO blink and hint | pool B class 20 variants 0 and 2, `G.$4FE`, `G.$500`, palette 10 of scroll1, glyph cells, sound 0x3E | `prop.rs` (new `prop_go.rs`) | 1 |
| T-C4 boss death chain | pool B class 22 controller (0x7BEF6; reinforcement list is D3c), call of 0x1D95E at boss HP 0, `G.$4D1` steps check against A8, clear task hand-off with D4d | `prop.rs`, `flow.rs` | 1.5 |
| T-C5 stage 2 camera and the car camera | controller 0x22398 (own mode table 0x2250A, events 0x22B9E, locks mode 2, mode 8 chase, end to mode 3), `G.$518` hooks; the car objects are M6 | `camera.rs` | 3 |
| T-C6 screen shake | `screen_shake` 0x222D6 exactly (state machine and shadow Y write), `present.rs` offset (scroll2 or scroll1 in Episode 7 area 1), the missing player requests (landing 1, slam 3) | `camera.rs`, `present.rs`, `player.rs` | 1.5 |
| T-C7 row scroll | 0x2335A task and a per line scroll2 offset in the renderer | `camera.rs`, `render.rs`, `present.rs` (main session) | 3 |
| T-C8 lift | scroll1 background 0x205C6 (tables 0x138C48, 0x15CFE8), class 57 and 64, by-Y spawn triggers, the Z + Y off-screen test | `camera.rs`, `prop.rs`, `enemy.rs` | 2.5 |
| T-C9 area change details | brightness dip of mode 0x12, class 96 (music) after mode 8, area start time and rank per area, `G.$514` heavy object rule | `camera.rs`, `prop.rs`, `flow.rs` | 1.5 |
| T-C10 verification | the scenarios A1 to A17 as harness scenarios with expected values, a trace compare of the end chain | `re/harness`, tests | 3 |

Total about 25 agent hours. M2 takes T-C1 (script 1, 8, 13, 14 and the area end scripts), T-C2, T-C3; M3 takes the rest. T-C1 is on the
critical path of M2 (Episode 1 opening, the walk to the next area) and of M3 (every area change).


## 12. Open items

1. **Episode 7 lift end and area 2 to 3** were not measured: the original (or the probe) stopped advancing the game frame counter twice
   in Episode 7 while the kill-all flag was held every frame (`hold 0xFF850E 1 2`): once in the lift at about 1000 frames, once in the
   library right after a script start. A run without kill-all reached the lift end lock but did not finish the last wave (wave 9
   needs alive 0). Measure with natural play and a hero that can reach the last riders. Also find out whether the hang is a real game
   bug (a loop in a pool B class 54, 57 or 64 handler or in the classes of area 2) or a probe effect.
2. **Real boss kills for the Butcher and Episodes 3 to 8** were not achieved with the poke method (repeated `objset ... hp 1` revives
   a dying boss and keeps the slow motion for ever; the heroes die while waiting). The end chain is measured on Vice T. and by poked
   scripts for all 8 episodes; the boss specific part (the slow motion length until the body state reaches 4) is D3c and D3d work.
3. **Episode 8 ending trigger:** script 24 waits in action 0x42 while the boss's `+$A6` is set (death animation phase?), then 540 frames,
   then clears `G.$4E2`. The poked run used the hero as `G.$4E0` and so waited out the 3600 frame timer. Needs the real Fessenden kill
   (D3d) and the class 97 actors (D4d, ending).
4. **Meaning of `G.$4DB`** (set by the radio, by the last boss and by the ending), `G.$516` (set by action 0x42; while it is set the player update at 0x18A28 skips the hero state machine, so the class 97 stand-ins
   replace the heroes in the Episode 8 clear sequence), `G.$503` (pose lock; where it is cleared), `G.$4D8` (debug skip), `G.$4DF`, `G.$514` (the held item rule of 5.3 was read from the decompiled player update and
   not tested with a barrel).
5. **Car:** `G.$518 = 2` never occurred; the roles of the helper objects (class 88 variants 2 and 4); what ends the section if the car
   survives; the collision passes 0x121D8, 0x122D8, 0x12342, 0x12454. D3e part 2 and D3c.
6. **Caption 0x1B** of script 5 (the window walk) and the caption texts of the scripts: D4d and D3e (class 66).
7. **Two player cabinet layout** (`G.$4CF = 1`): 0x1DFD8 adds 4 to the end target pointer, and the HUD and join layouts change; the
   factory setting is the 3 player layout, so this was not measured.
8. **Sub-area start script:** the start id of the sub-areas (0 in the table) never runs because 0x1DA10 is not called for them. If the
   port ever calls the area start for them it must not run script 0.
9. **Wave controllers kept by the probe's `jump`** (4.3 warning). A natural run cannot have two controllers, but nothing in the code
   forbids it (a wave whose trigger is reached while the previous controller still runs): check Episode 5 area 1 (lock 0xCD0 and the
   wave 5 with final threshold 99) in natural play.
10. **Pose animation and sound per hero** in the scripts (action 0x04 uses the extra life pose of D2a section 11.1): confirm that the
    four heroes pose in the real boss death and how long the pose is held (the poked runs kept state 0x18 for the whole tally).
11. **Lineup and walk slot assignment for 2 players** (which spot the second hero takes when only P1 and P3 play): read from the code,
    not measured.

## 13. New rows for `mechanics-seen.md`

Group T (transitions, camera and bounds). IDs T01 to T40 are free; D4d should use its own group letters.

| ID | Mechanic | Source | Port | Notes for the design agent |
| --- | --- | --- | --- | --- |
| T01 | Stage 2 (Episode 3) has its own camera controller 0x22398 and mode table 0x2250A; its 10 events at X 0x300 to 0x2D00 are locks (mode 2), mode 8 is the car drive | R 0x22398, 0x2250A, 0x22B9E; M e3footn, car1 | MISSING | the port runs the standard table for it: wrong |
| T02 | Camera never leaves the leftmost player behind; an idle hero holds the camera; push line 0xC0 (1P) or 0xE8 (2P, 3P) unless the event has its own | R 0x20A22; M lag2, lag3 | DONE | |
| T03 | Hero clamp: X in [camX + 0x10, camX + 0x16F], screen depth Z + floor - camY in [0x10, 0x9F], skipped while `G.$506 != 0` | R 0x4FCE; M clampR, d*_*_* | DONE | |
| T04 | Walkable depth per area is 64 px in most areas (screen band 16 to 79), 95 to 114 px in the garage, sewers, cave and yard; table in 3.2 | M d4c-b2/d* | DONE (terrain) | test values for terrain |
| T05 | Drop in position: X = camX + 96 (1P), 64 and 128 (2P), 48, 96, 144 (3P); Z = camY - floor + 32, 48, 64; height floor + 0x100 | R 0x62D6, 0x638C | DONE | |
| T06 | Wave lock ends: type 0 sets `go` at once, type 2 starts the area end script (0x1D95E), type 4 the boss area script (0x1DAC4); 30 frame delay at the end; all heroes must be in the normal ground state | R 0x7B464 | DONE except the scripts | |
| T07 | The area end script releases the wave lock with its own first GO record (96 frames after the wave end when no items lie around, up to 440 with items or containers) | M e1a0, l_e1a0_k2 | MISSING | |
| T08 | Scripted control: 26 scripts, 41 actions, 3 entry points, table 0x1E6FC | R 0x1DB5E and 5.4, 5.5 | MISSING | |
| T09 | Scripted heroes cannot be hurt (enemy attack passes off), cannot pick up when `G.$506 = 1`, drop a carried barrel | R 0x10546, 0x191C4 | DONE (readers), no setter | |
| T10 | Item waits (actions 0x2A, 0x38): the pad works for 120 to 240 frames after a wave end and after a boss death | R 0x1DE6C, 0x1DE86 | MISSING | |
| T11 | Fade = 16 steps of 1 vblank at area end (out) and start (in); mode 0x12 is a hard cut with 2 to 4 black frames; sub-area switch has no fade | R spec 2.4; M e4a0n, n1a1, e2a0n | DONE / PARTIAL | |
| T12 | Sub-area switch (camera mode 8) resets stage time, floor, spawn lists, player count, creates the music object class 96; it never places the heroes and never runs a start script | R 0x20E78, 0xAAE2, 0xACE0, 0x9236C | PARTIAL | corrects D3e (class 96 is not created at every area start) |
| T13 | Area start (0xAA0E): place heroes (0x4E0A), start script (0x1DA10), pools, items leave, rank, time, floor, spawn lists | R 0xAA0E | PARTIAL | |
| T14 | GO burst (class 20): 3 blinks of 30 frames on, 5 off, sound 0x3E; forced at the start of area end scripts; idle hint 180 frames after a lock ended (camera moved < 0x80), then every 600 + 105 frames | R 0x7BB68; M go1, go2 | MISSING | no GO at the end of a wave |
| T15 | Openings: control after 648 (Ep 1), 573 (Ep 2), 321 (Ep 3, 5 to 8), 894 (Ep 4) game frames; the map card is the first 255 | M o1 to o8 | MISSING | |
| T16 | Episode 1 opening pans the camera at 3 px per frame while the heroes stand at X 0x420, 0x410, 0x400; clamp off | M o1, o1p3 | MISSING | |
| T17 | Episodes 2 and 4 start with the heroes riding the Cadillac (state 0x432) and jumping out | M o2, o4 | MISSING | |
| T18 | Episode 4 opening waits for `G.$4FC` to clear (set at the start, cleared at gf 888) | M o4 | MISSING | |
| T19 | Boss death chain: 0xACEE, slow motion, class 22 calls 0x1D95E, `G.$4E3` after the script, tally, story panel, next opening; 94 + 278 + 951 game frames on Vice | M b1c | PARTIAL | |
| T20 | Walk off after a boss: wait for items, barrier, pose (sound 0x18), hold right 300 with clamp off (not Episode 1 and 8) | R scripts 3, 12, 20, 24 | MISSING | |
| T21 | Car section: radio makes car (HP 120) at 16, stops at 105, heroes jump aboard 284 to 332, `G.$518 = 1` and camera mode 8 at 364, 11 px per frame, clock frozen, ends when the car's HP is 0 (camera mode 3 at that X) | M car1, car3 | MISSING | |
| T22 | Camera mode 8 chase: camX += clamp(car X - (camX + 0xC0), 8, 20) | R 0x22A68 | MISSING | |
| T23 | Lift ride restarts from rest after every lock (sound 0x86, shake 2, acceleration 6/256 per frame squared up to 6 px per frame); stops at Y 0x3000, 0x1E00, 0x1400, 0x800; end at 0 | R 0x210E0; M e7a1n | PARTIAL | |
| T24 | Row scroll words equal the formula of the stage spec (verified at 3 camera positions) | M rows_* | MISSING | |
| T25 | Screen shake: offsets -s, +s (3), -(s-1) (3), ... +1 (3), length 6 s - 2 frames; 29 requesters, strengths 1 to 4 | R 0x222D6; M shake1 to 4 | STUB | |
| T26 | Stage clock and rank are frozen while `G.$4DD != 0` or `G.$4D5 = 1` and in car mode | M e4a0n, car2 | DONE (`rank.rs`) | |
| T27 | `G.$4D4` (stage clear flag) revives a hero in the death sequence with 1 HP | R 0x1BF82 | MISSING | [I] |
| T28 | Probe limit: `jump N A X` keeps wave controllers before X; stale controller can send `go` | M hallwin, e1a1n | n/a | |
| T29 | Original hang (game frame counter stops) with kill-all held in Episode 7 | M e7a1, l_e7a2_k19 | n/a | investigate |
| T30 | Ep 8 area 3 start row of table 0x4E56 is missing; a sub-area never uses it | R 0x4E56, 0x4FCE | n/a | |

