# Exactness harness

Proves that the port plays like the original: MAME records the original, the port plays the same
inputs, and `diff.py` compares them frame by frame (object fields by their original offsets, globals,
tilemaps, palette, sprite list, sound requests). Plan: `plan/phase-00-exactness-harness.md`.

Needs: MAME 0.289 command line (`brew install mame`), `dino.zip` in the repo root, the placeholder
ROMs in `re/mame/roms/` (see the phase 0 plan), `re/.venv` (numpy). Everything the harness writes goes
to `re/out/` (git-ignored): traces in `re/out/traces/`, MAME's own files (cfg, nvram, screenshots,
`record.log`) in `re/out/mame/`.

## Record, run, diff

From the repo root:

```sh
re/harness/record.sh re/harness/scripts/stage1_jack.txt        # 1. MAME: re/out/traces/stage1_jack.{state,mame.trc}
re/harness/port.sh   re/harness/scripts/stage1_jack.txt        # 2. port:  re/out/traces/stage1_jack.port.trc
re/.venv/bin/python -I re/harness/diff.py \
    re/out/traces/stage1_jack.mame.trc re/out/traces/stage1_jack.port.trc    # 3. compare
```

`record.sh SCRIPT [NAME] [FRAMES]` boots the game in MAME (no window, no sound, full speed), plays the
boot script (coin, start, character select), stops at the sync point, writes the snapshot, then plays
the test script for FRAMES (default 600) game frames and writes one record per frame. It takes a few
seconds and is bit-for-bit repeatable. `port.sh SCRIPT [NAME] [FRAMES]` loads the snapshot into the
port and plays the same script. Both take `NAME` (default: the script name without `.txt`).

### Three ways to run the port (`MODE=...`)

| mode | command | what it checks |
| --- | --- | --- |
| `run` (default) | `port.sh SCRIPT` | the port plays on its own from the snapshot. The real test: it must match for the whole run. A difference at frame k also causes differences after k. |
| `follow` | `MODE=follow port.sh SCRIPT` | like `run`, but the random number state `G.$14` is copied from MAME before every frame. Objects of classes the port does not run still call `rand()` in the original; this stops that drift from hiding the ported code. Writes `NAME.follow.trc`. |
| `step` | `MODE=step port.sh SCRIPT` | before every frame the port is loaded with MAME's state after the previous frame, so a difference at frame k comes from frame k alone, never from an earlier difference. Use it to find out why something differs. Writes `NAME.step.trc`. |

Start with `step`: it tells you which classes and routines are wrong or missing. Use `run` to prove
the whole run is exact. In `step` mode a VRAM or sound write that an unported routine makes shows up
as a difference in that one frame only.

## Reference scenarios

### Power-on references

Use `DINO_CLOCK=video` with an empty boot script and `DINO_SYNC=vblank:1`. This records every
video frame, including the black boot frames and busy frames. No player-control sync is needed.
The snapshot is after video frame 1; trace frame -1 holds it, and trace frame k is video frame
k + 2. `vblank:0` also selects frame 1. The default `game` clock still uses the IRQ tap and skips
busy frames for the existing gameplay tests. The video clock samples at the end of each video
frame, as the design probes do, rather than before the IRQ flush.

All commands below are headless. `record.sh` sets both SDL drivers to `dummy`, disables video
and audio output, and clears the EEPROM before each run. Two MAME runs make all three references:

```sh
DINO_CLOCK=video DINO_BOOT=re/harness/scripts/attract.txt DINO_SYNC=vblank:1 \
  re/harness/record.sh re/harness/scripts/attract.txt attract2 26500
python3 re/harness/trace_prefix.py re/out/traces/attract2.mame.trc \
  re/out/traces/attract.mame.trc --frames 13400
cp re/out/traces/attract2.state re/out/traces/attract.state

DINO_CLOCK=video DINO_BOOT=re/harness/scripts/attract.txt DINO_SYNC=vblank:1 \
  re/harness/record.sh re/harness/scripts/coinstart.txt coinstart_full 3000
python3 re/harness/trace_prefix.py re/out/traces/coinstart_full.mame.trc \
  re/out/traces/coinstart.mame.trc --control-frames 600
cp re/out/traces/coinstart_full.state re/out/traces/coinstart.state
```

`attract.txt` has no inputs. `coinstart.txt` uses the `boot_jack.txt` video timings: coin at 600,
start at 700, confirm Jack at 900. Its test indexes are shifted by 2 because of the frame-1
snapshot. The prefix tool stops after 600 game-counter increments from Jack's first controllable
frame in Episode 1. It preserves the sync record and updates the frame count in the header.

Verified references (each trace also has one frame -1 record):

| name | recorded frames | trace bytes | snapshot bytes |
| --- | ---: | ---: | ---: |
| `attract` | 13,400 video frames | 26,025,859 | 278,959 |
| `attract2` | 26,500 video frames | 50,154,999 | 278,959 |
| `coinstart` | 2,322 video frames, including 600 game frames after control | 3,973,923 | 278,959 |

Jack first has control at video frame 1723; `coinstart` ends at 2323. These lengths follow the
requested budgets. Boot adds time before the 13,104-frame attract cycle: state 0 returns at
13411, and the second return is about 26516, just beyond these cuts. `attract2` includes demos
1 to 4 (Episodes 2, 3, 8 and 5), and the RNG reset to 0x1C3 at each demo setup.

M1 coverage: the full 64 KB RAM includes mode at `0xFF84D0`, attract state at `0xFF0FD0`, task
states, counters, camera shadows, RNG, and sound ring and indexes. The 192 KB `gfx` region
includes all scroll1/2/3 tilemaps, palettes, row-scroll data and both sprite buffers. `cpsa`
and `cpsb` include live scroll registers, layer bases, layer order and priority masks. No tilemap
layer or scroll register is missing. The current diff does not compare the CPS register regions;
M1 will need that comparison as well as the new front-end RAM fields on the port side.

Missing from the recorder for a stricter display/sound comparison:

- Explicit sound enqueue events and sound commands actually sent or muted. The raw ring permits
  ordinary request reconstruction, but a RAM clear during demo setup can lose or confuse events
  between samples. Add per-frame `snd` events with code, d1 and d2 at the enqueue routine, plus
  flush events with the sent/muted result. Keep the raw ring too.
- The video device's latched sprite buffer and decoded palette, and the timing of register/VRAM
  writes within a frame. RAM and registers are end-of-frame samples, not the device's rendered
  output. For exact displayed-frame checks, add a rendered-frame hash or optional frame images;
  expose the latched video state or timestamp write taps if diagnosing raster timing is needed.

Those extensions are not added here. References stay under ignored `re/out/traces/`; they are
not in `regress.sh` because the port cannot run the attract loop yet.

Player 1 = Jack in stage 1. Record once, then run the port in the three modes. `NAME` is the name of
the trace set; `FRAMES` the number of test frames.

| NAME | script | sync | FRAMES | what it covers |
| --- | --- | --- | --- | --- |
| `stage1_jack` | `scripts/stage1_jack.txt` | `control` | 600 | the planned comparison: walk, jumps, punch chain, run, a grab, knee, throw |
| `stage1_late` | `scripts/stage1_jack.txt` | `control+86` | 600 | the same after the opening scene (fewer unported objects) |
| `explore` | `scripts/explore.txt` | `control` | 600 | hold right: scrolling, streaming, the first waves, the right wall |
| `stage1_fight` | `scripts/stage1_fight.txt` | `control+86` | 1400 | walk to the first wave, then 1000 frames of punch chains, jump kicks and steps against up to four thugs |
| `obstacle` | `scripts/obstacle.txt` | `control+86` | 600 | Jack walks into the first containers (the terrain obstacle test 0x12E9A, wall bit 0x80, +$94) |

Item scenarios (phase 1 task 2). The first waves give few items, so these start from a `DINO_POKE` file
(below) that puts the items on the floor, and a script that plays with them. Record with
`DINO_POKE=re/harness/pokes/NAME.lua DINO_SYNC=control+86 re/harness/record.sh re/harness/scripts/NAME.txt NAME FRAMES`.
Compare in `MODE=step` with `DINO_STEP_PLAYERS=1` (see below).

| NAME | script / poke | FRAMES | what it covers |
| --- | --- | --- | --- |
| `items` | `scripts/items.txt` (no poke, `control` + 86) | 1250 | the stage1_fight run (two containers break) and then the bomb and the DIME they drop: lift, held, throw, bounce, arm, blast, treasure |
| `misc` | `scripts/misc.txt`, `pokes/misc.lua` | 200 | food and a treasure on the floor: eat (heal, points at full HP), take |
| `containers`, `containers2`, `containers3` | `scripts/containers.txt` with `pokes/containers.lua`, `containers2.lua`, `containers3.lua` | 400 | lift, throw, bounce and break of the container variants, the drop roll, the debris effect |
| `guns` | `scripts/guns.txt`, `pokes/guns.lua` | 600 | pistol (empty, thrown), rifle, Uzi and M-16 (automatic fire), shots, shells |
| `guns2` | `scripts/guns2.txt`, `pokes/guns2.lua` | 420 | shotgun, bazooka (rocket and its shell), bat swings |
| `bombs` | `scripts/bombs.txt`, `pokes/bombs.lua` | 900 | fire bomb, explosive sticks, time bomb: throw, bounce, explosions |
| `ammo` | `scripts/ammo.txt`, `pokes/ammo.lua` | 220 | pistol, two ammo boxes (refill, then 1000 points for a full gun) |

MAME's lag frame (frame 1064 of `items`) is ignored by the comparison. Result with the port of task 2: only frames where the player's own state machine changes (pick-up,
throw, first frame of a swing, the player side is task 3) differ in the item classes. **Since phase 1 task 3 the player is ported: run these in plain `MODE=step`, without
`DINO_STEP_PLAYERS`** (see below); the player and every pool E class then match for the whole run in all nine.

Player weapon scenarios (phase 1 task 3). Keep a key release between two presses of the same direction at least 2 frames long (the game cooks the inputs and merges a gap of one frame). Same way: `DINO_POKE=re/harness/pokes/NAME.lua DINO_SYNC=control+86 re/harness/record.sh re/harness/scripts/NAME.txt NAME FRAMES`
(the poke has the name given in the table; `empty` uses `pokes/empty.lua`). Weapons placed more than 0x200 right of the camera free themselves at once (0x1760), so
keep them left of x 0x580 at the start.

| NAME | script / poke | FRAMES | what it covers |
| --- | --- | --- | --- |
| `melee` | `scripts/melee.txt`, `pokes/melee.lua` | 820 | club, bat, stick, heavy weapon, fire bomb: pick-up, run swing (state 0x202 with a club and with a bomb), charge swings of 2, 16 and 56 frames (state 0x20), jump, walk and run with a bat, the club-throw pose (no enemy near) |
| `heavy` | `scripts/heavy.txt`, `pokes/heavy.lua` | 420 | containers (kind 0x20A): lift with the stick towards it, idle and walk with it (legs layer), standing and walking heavy jump (states 0x1E, 0x20), no run, throw; variant 6 lifted without the stick |
| `gunwalk` | `scripts/gunwalk.txt`, `pokes/gunwalk.lua` | 600 | Uzi, M-16, rifle, shotgun: standing fire (0x1C), walking fire both ways (0x1E, the legs layer, the first one with a stale `+$38`), jump while firing, run with a gun (the run pose 0x1A0F0) |
| `empty` | `scripts/empty.txt`, `pokes/empty.lua` | 1150 | an Uzi and an M-16 emptied by holding A (the empty Uzi is let go, the empty M-16 is a club: charge swing) |
| `pickups` | `scripts/pickups.txt`, `pokes/pickups.lua` | 360 | an ammo box with no gun (`+$10F`), the refill of the next gun at its pick-up, taking a weapon while holding one (the old one is dropped) |
| `melee_hannah`, `melee_mustapha`, `melee_mess`, `heavy_*`, `gunwalk_*` | the scripts and pokes above, `DINO_BOOT=re/harness/scripts/boot_hannah.txt` (or `boot_mustapha.txt`, `boot_mess.txt`) | 820, 420, 600 | the same runs with the other three characters (the select screen order is Jack, Hannah, Mustapha, Mess; the port reads the character from the snapshot): the per-character pose, run and jump tables, Hannah cannot jump with a heavy object |
| `armed` | `scripts/armed.txt`, `pokes/armed.lua` | 1000 | the stage1_fight script with a bat: swings that hit enemies and a prop, the bat breaks into a stick, a club swing at an enemy in reach, hit while holding (the weapon is dropped), picked up again |
| `grabs`, `grabs2`, `grabs3` | `scripts/grabs.txt`, `grabs2.txt`, `grabs3.txt`, `pokes/grabs.lua` | 1200, 1100, 1200 | (phase 1 task 3b) five thugs around Jack (`DINO.spawn_enemy`, entrance "wait"); a fixed pseudo random input sequence of walking bouts and presses of A, A with a direction and J: grabs, both knees, throws forwards and backwards (also Mess's two throw animations), throws into other thugs (pass 0x1185A), releases by a jump or by being hit, thugs that die in the hold. Run with all four characters (`DINO_BOOT=re/harness/scripts/boot_hannah.txt` and so on; names `grabs_hannah`, `grabs2_mustapha`, ...); `grabs3` is meant for Mess and Jack |

Pool A class scenarios (phase 1 task 4a). Record with `DINO_POKE=re/harness/pokes/NAME.lua DINO_SYNC=control+86 re/harness/record.sh re/harness/scripts/NAME.txt NAME FRAMES` (`intro`: `DINO_SYNC=vblank:1150`, no poke).

| NAME | script / poke | FRAMES | what it covers |
| --- | --- | --- | --- |
| `intro` | `scripts/intro.txt` (no input), `DINO_SYNC=vblank:1150` | 600 | (phase 1 task 4a) the opening scene from the start of the stage (no control yet): the scripted thugs (pool A class 55), the bystander (54), the three bird controllers (49) and the first birds (50, variants 0xA / 0xC from the stage script); the unported pool B master (class 0x4A) is loaded from MAME every frame in step mode, so the act list of classes 54 / 55 runs for real. Classes 56 and 57 of the scene are not ported |
| `c10a` | `scripts/c10a.txt` (`genscript.py 1 900`), `pokes/c10a.lua` | 900 | (task 4a) class 10 (Hammer T., Blk Elmer, Wrench T.): three thugs, entrances walk in, drop and wait; random fight inputs |
| `c20a` | `scripts/c20a.txt` (`genscript.py 3 1800 --hits 8`), `pokes/c20a.lua` | 1800 | (task 4a) class 20 (Punk, Thug): all five entrances (walk, dash, fade, drop, wait) and fights |
| `c20b` | `scripts/c20b.txt` (`genscript.py 21 2400 --hits 8`), `pokes/c20b.lua` | 2400 | (task 4a) eight waiting Punk / Thug, Jack's HP 0x190: all actions and attacks. `c20b_hannah`, `c20b_mustapha`, `c20b_mess` with `DINO_BOOT` |
| `c10b`, `c10c` | `scripts/c10b.txt` (`genscript.py 7 1500 --hits 8`), `scripts/c10c.txt` (`genscript.py 11 2400 --hits 8`), `pokes/c10b.lua` / `c10c.lua` | 1500, 2400 | (task 4a) five and eight class 10 thugs that wake up near Jack, Jack's HP raised to 0x190: all attack kinds, the paired attack, the stomp walk, deaths. `c10c_hannah`, `c10c_mustapha`, `c10c_mess` with `DINO_BOOT=re/harness/scripts/boot_NAME.txt` |

Class 14 scenarios (phase 1 task 4a2: BLADE / RAZOR, three pool A slots each, and the pieces they throw). Record with
`DINO_POKE=re/harness/pokes/NAME.lua DINO_SYNC=control+86 re/harness/record.sh re/harness/scripts/NAME.txt NAME FRAMES`; each with the three other
characters too (`DINO_BOOT=re/harness/scripts/boot_hannah.txt` ..., names `c14b_hannah`, `c14b_mustapha`, `c14b_mess`; the script and poke keep the
base name). Jack's HP is raised to 0x190 in all of them. A BLADE is `+$26` 0x00ee, a RAZOR 0x02ee (`ee` = the entrance: 0 walk in, 2 / 4 stand and wait
0x4B0 / 0x1E0 frames, 6 jump in from above, 8 jump in beside the target, 0xA / 0xC walk in with two knives).

| NAME | script / poke | FRAMES | what it covers |
| --- | --- | --- | --- |
| `c14a` | `scripts/c14a.txt` (`genscript.py 5 1200 --hits 8`), `pokes/c14a.lua` | 1200 | a Blade that walks in and a Razor and a Blade that wait: the AI states, the guard and its back step, hits, grabs, throws, knives dropped by a hit Blade (class 34), thrown blades (class 41) |
| `c14b` | `scripts/c14b.txt` (`genscript.py 7 2400 --hits 9`), `pokes/c14b.lua` | 2400 | two Blades and two Razors waiting (entrances 2 and 4): the fight to their deaths (death, drops, the parts) |
| `c14c` | `scripts/c14c.txt` (`genscript.py 11 2400 --hits 9`), `pokes/c14c.lua` | 2400 | the entrances 6, 8, 0xA, 0xC (jump in from above and beside the target, walk in with two knives: the parts, knife throws, the run at the target) |
| `c14d` | `scripts/c14d.txt` (`genscript.py 13 2400 --hits 9`), `pokes/c14d.lua` | 2400 | three waiting OFF SCREEN (camera X 0x381, screen X 0x369..0x501): the flee timer and the run state 0x36 |
| `c14e` | `scripts/c14e.txt` (`genscript.py 17 1500 --hits 9`), `pokes/c14e.lua` | 1500 | two walk in against containers (the obstacle bit: swing state 0x34, containers destroyed by a Blade) |
| `c14f` | `scripts/c14f.txt` (`genscript.py 19 3000 --hits 10`), `pokes/c14f.lua` | 3000 | four with drop rows, entrances 0, 4, 0xA, 0xC, fought one after the other to their deaths |
| `c14g` | `scripts/c14g.txt` (Jack stands still 1100 frames, then `genscript.py 23`), `pokes/c14g.lua` | 1400 | five waiting off screen on both sides: the states 0x36 and the chase 0x38 (the run goes away from the screen in 3 of 16 cases) |
| `c14h` | `scripts/c14h.txt` (A every 8 frames), `pokes/c14h.lua` | 2000 | Jack punches all the time against three with two knives: thrown blades are hit and fall (class 41 sub 2 / 8), knives, guard |
| `c15a` | `scripts/c15a.txt`, `pokes/c15a.lua` | 300 | the five rows of the flying pieces (pool A class 15), both facings (the poke writes `+$0C` and `+$24` after `spawn_enemy`) |

Union of the states reached (`cover.py`): 24 of the 30 states of class 14 (never: 0, 2, 4, 0x10, 0x12, 0xC: no reachable code enters them), the hit reactions, held,
thrown, death. `check.sh` stops with the panic message when the port panics (the trace is shorter than FRAMES then; look at `compared N records`).

```sh
DINO_SYNC=control+86 re/harness/record.sh re/harness/scripts/stage1_fight.txt stage1_fight 1400
for m in run follow step; do MODE=$m re/harness/port.sh re/harness/scripts/stage1_fight.txt stage1_fight 1400; done
re/.venv/bin/python -I re/harness/diff.py re/out/traces/stage1_fight.mame.trc re/out/traces/stage1_fight.step.trc
```

### Scenarios that need objects: `DINO_POKE`

`DINO_POKE=file.lua` (record time only) runs a Lua file once at the sync point, before the snapshot is
written, so MAME and the port both start from the edited RAM. The file gets the table `DINO`
(`local DINO = ...`): `DINO.space` (the main CPU memory space) and
`DINO.spawn_item(class_word, variant, x, z [, w5c, drop_row])`, which takes a pool E slot the way the
stage script does (0xFDAE: free list G.$65FE / $65F4, class word = class * 4, variant = `+$26`). Other
pokes may write RAM through `DINO.space` (for example Jack's HP). Examples: `re/harness/pokes/`.
`DINO.spawn_enemy(class_word, variant, x, z [, w5c, b5b, drop_row])` does the same for pool A (free list G.$65F8 /
$6602, 0x483C): class word = class * 4 (class 2 = 8), variant = type * 0x100 + entrance (class 2: type 0 Ferris, 2 Gneiss,
4 Driver; entrance 0 walk in, 2 dash in, 4 wait until a player is near, 6 fade, 8 jump, 0xA drop), `b5b` = the entrance
facing (a negative byte sits). `pokes/grabs.lua` is an example.

### `DINO_STEP_PLAYERS=1` (step mode only)

In `step` mode every object starts a frame from MAME's state after the previous frame. The players
run before pool E in the same frame, and the port's player does not lift, throw or swing weapons yet
(task 3), so a player difference would hide what the items do. With `DINO_STEP_PLAYERS=1` the port
also reads the players of MAME's record of the frame being run (one record ahead) and loads them
after its own players' update, just before pool E runs, so the items see the same player as in the original. Limits: the item
classes then compare exactly, but the player fields themselves are copied, not tested, and
the score the items add to the player shows doubled in the diff (the loaded player already contains it).
Use it only for the item scenarios.

**Obsolete since phase 1 task 3.** The player port runs the real pick-up, throw, swing and fire states, and a plain `MODE=step` run now matches for the player and every
pool E class. With `DINO_STEP_PLAYERS=1` the item scenarios still show differences on the frames where the item itself changes the player (a throw clears `+$B6`, a shot
clears `+$C4`, an empty gun sets a new `+$B6`): MAME's record of the frame already has those changes, so the loaded player tells the item "I do not hold you" before it
runs. The flag is still in the code, but do not use it.

## Helper scripts (phase 1 task 4a)

- `regress.sh [NAME ...]`: the step runs of all the recorded scenarios of phase 1 (49 with the class 14 scenarios, about 40 s) (a quick check that nothing broke: the number of
  (class, field) pairs in ported classes per scenario, 0 is the goal; `armed` has 4 known ones (the prop hit at frame 564 and the class 15 pieces that the unported prop B#40 spawns), `intro` the scripted player and the scene spawns, `c14c_mustapha` 8 (a hit spark in another pool C slot because of the unported dizzy stars)).
- `check.sh MODE NAME [diff.py options]`: runs the port in `step` / `follow` / `run` mode on a recorded scenario (the
  script and frame count come from a small table in the script, or from `SCRIPT=` and `FRAMES=`) and prints the diff report. It stops with
  the message when the port panics (before task 4a2 a panic only shortened the trace and was not noticed).
- `objhex.py TRACE ADDR LEN FIRST [LAST]`: hex dump of one object (or any RAM range) at a frame, then the changed bytes
  of every later frame (MAME or port trace). `cover.py TRACE [enemy|prop] [CLASS...]`: which state words (+$04..+$07) a class
  reaches in a recording (how much of its code a scenario runs).
- `genscript.py SEED FRAMES [--hits N]`: writes a pseudo random fight input script (bouts of walking, punches, throws, jumps).
- `re/tools/d2 ADDR COUNT [TABLE:N ...]` (`dj.py`): disassembly with the word jump tables decoded (`jmp / jsr T(pc,dN.w)`;
  the length of a table comes from the Ghidra function starts, or is given as `T:count`); `re/tools/dc.py` makes a compact
  listing of a `d2` dump (labels only at branch targets).

## Checking one routine against the original code

Some routines cannot be reached by a recorded scenario (other stages, rare tiles). For those,
`re/harness/terrain_oracle.py` runs the original 68000 code in a small interpreter on random
inputs and a unit test replays them on the port. Example for the terrain collision (0x12CB4 and
friends, `dino-game/src/game/terrain.rs`):

```sh
re/.venv/bin/python -I re/harness/terrain_oracle.py re/out/terrain_vectors.txt 1 250000   # about 30 s
cd dino-rs && cargo test -p dino-game terrain_matches_the_68000_code
```

The vectors file derives from the ROM and stays in `re/out/` (git-ignored); the test skips itself
when it is missing. `COVERAGE=1` prints the instructions of the routines that no vector ran,
`TRACE_VECTOR=n` prints the instruction trace of vector n. The interpreter knows only the opcodes
of those routines and stops on any other, so reuse it for other routines by extending `CPU`.

## Reading the report

`diff.py MAME.trc PORT.trc [--allow FILE] [--at FRAME]... [--field PATTERN] [--first N] [--from FRAME]`

1. **FIRST DIFFERENCE**: the first frame with a difference and every differing item there
   (`enemy[17] D7D4 +$08 x: MAME 03BC0000  port 03BF0000` = pool kind, slot, handle, original offset,
   field name, both values). `--at 492` prints all differences of one frame.
2. **FIELDS WITH DIFFERENCES**: per field, the number of frames that differ and the first frame.
3. **DIFFERENCES BY OBJECT CLASS**: the same, grouped by pool and class (`+$20 / 4`) with the handler
   address from the original's dispatch table, and `[ported]` or `[NOT PORTED]`. Differences of a
   class the port does not run are expected: the port keeps those objects frozen at the snapshot state
   (`Game::trace_hold`), they do not move or animate. Each such class is a task, not a bug. Differences
   of a `[ported]` class, and differences in globals, need a cause each.
4. **SPRITE LIST DIFFERENCES**: the port tags every sprite entry with the object that drew it, so
   entries that differ are split into: from unported classes (expected), from ported classes or no
   object (look at these), only in MAME (counted as unported when an unported object differs in that
   frame: its shadow or new picture).
5. **SOUND REQUESTS** (the code words of the 68000's sound queue at `$3A(a5)`): "only MAME has" means
   every request of the port is there and MAME has more (callers the port lacks). Any other
   difference is listed under VIDEO AND SOUND. The pan / volume longs of a request are not compared yet.
6. **VIDEO AND SOUND**: tilemap cells (scroll1/2/3), palette words, sprite list, sounds.
7. **NOT MODELED**: bytes that change in MAME but that the port does not trace. Not counted as
   differences: this is the blind spot of the harness. To cover a byte, add the field to
   `dino-rs/crates/dino-game/src/game/trace.rs` (one line per field, original offset and name).

Frame -1 is the sync state itself: a difference there means the loader missed a field.

`re/harness/objects.py TRACE [FRAME...]` lists the live objects of a trace (class, state, position,
frame pointer): use it to see what is on screen at the sync point and later.

Approved known differences go in `re/harness/allow/PHASE.txt` (one rule per line: a key pattern such
as `enemy.*` or `global.rng` and an optional frame range, with a reason). Pass it with `--allow`.
Unported classes are not approved differences, they are tasks.

## Picking a sync frame

The sync point is the frame where both sides start from the same state. `record.sh` takes it from
`DINO_SYNC` (environment):

- `control` (default): the first frame where player 1 can be controlled in a stage (game mode 0x0C,
  state 0x02000000, no input lock). Good for "from the start of the stage".
- `control+N`: N game frames later. Use it to skip a scripted opening whose objects the port does not
  run yet. Example: `DINO_SYNC=control+86 re/harness/record.sh SCRIPT stage1_late`. Look at
  `objects.py re/out/traces/NAME.mame.trc 0 40 80` to see when the leftover objects are gone.
- `vblank:N`: the first frame at or after MAME vblank N, for a moment that is not reachable from the
  start of a stage (a boss start, a lock). Find N from the log line `sync at vblank ...` of an earlier
  run, or by taking screenshots: `DINO_SNAP_EVERY=30 re/harness/record.sh ...` writes a PNG every 30
  test frames into `re/out/mame/snap/dino/`.

Other environment variables: `DINO_BOOT` (boot script, default `scripts/boot_jack.txt`: coin, start,
choose the first character), `DINO_MAX_VBLANKS` (give up after this many vblanks, default 30000).
`re/out/mame/record.log` has MAME's output (the missing-ROM warnings are expected).

A game frame that overran its vblank (slowdown) is not recorded twice: the next record carries
the lag flag, and `diff.py` prints the lag frames.

## Input scripts

One file per test, frames counted from the sync point (frame 0 is the first frame after it). The same
file drives MAME (`record.lua`) and the port (`--input-file`). Segments `FIRST-LAST:KEYS` (both frames
inclusive) separated by commas or new lines, `#` starts a comment. Keys:
`U D L R` stick, `A` attack, `J` jump, `S` start, `C` coin; the digits `1 2 3` choose the player for the
keys that follow (player 1 by default). Several segments for one frame are combined. Examples:

```
0-29:            # stand still for 30 frames
30-59:L          # walk left
125-126:A        # one punch (press for 2 frames)
330-331:L        # double tap left, as the original needs it: tap, release, tap
334-370:L
0-599:R2L3A      # all three players: P1 right, P2 left, P3 attack
```

To write a new test: copy `scripts/stage1_jack.txt`, change the segments, then run the three commands.
Keep presses 2 frames long and 8 frames apart for combos. A script should cover one idea each
(walk, jump, combo, grab, throw, pick up) so a difference points at one routine.

`scripts/boot_jack.txt` is the boot script (frames are vblanks from power on): coin at 600, start at
700, accept the first character at 900.

### Two and three players

The factory cabinet has three separate coin slots. `boot_2p.txt` selects P1 Jack (id 0) and P2
Hannah (id 2); `boot_3p.txt` selects P1 Jack, P2 Mustapha (id 1), and P3 Mess (id 3). They coin each
slot, start the extra players while select is open, move every cursor, then confirm every player
with attack. Initial cursors are Jack, Hannah, Mustapha; left/right skip occupied heroes.

From the repo root, record these three 300-frame scenarios (all MAME runs are headless):

```sh
DINO_BOOT=re/harness/scripts/boot_2p.txt re/harness/record.sh re/harness/scripts/mp_walk.txt mp2 300
DINO_BOOT=re/harness/scripts/boot_3p.txt re/harness/record.sh re/harness/scripts/mp_walk.txt mp3 300
DINO_BOOT=re/harness/scripts/boot_jack.txt DINO_SYNC=control+86 re/harness/record.sh re/harness/scripts/mp_join.txt mp_join 300
MODE=step re/harness/port.sh re/harness/scripts/mp_walk.txt mp2 300
MODE=step re/harness/port.sh re/harness/scripts/mp_walk.txt mp3 300
MODE=step re/harness/port.sh re/harness/scripts/mp_join.txt mp_join 300
for name in mp2 mp3 mp_join; do
    re/.venv/bin/python -I re/harness/diff.py re/out/traces/$name.mame.trc re/out/traces/$name.step.trc
done
```

`mp_walk.txt` waits for the opening to finish, then walks all players, including different vertical
directions. `mp_join.txt` starts with Jack playing, coins P2 at frame 40, starts P2 at 90, and confirms
Hannah with attack at 130. **Coin and start alone leave P2 in the hero strip.** The earlier failed
400-frame join recording did accept both inputs, but never confirmed the hero; automatic selection
needs 441 frames after the request. A separate attack or start edge confirms after the strip's first
20 frames. No parser or input-port mapping change is needed.

Verified step results: `mp2` and `mp3` each have **0 ported-class pairs and 0 global differences**;
their MAME and port traces keep the expected 2 / 3 live players and hero ids for all 301 records
(sync plus 300 frames). `mp_join` has **37 ported-class pairs and 15 global fields differing**.
MAME makes P2 live at frame 130, but the port's join task is still empty
(`dino-rs/crates/dino-game/src/game/flow/join.rs:23`; coin handling is also a stub in
`flow/front.rs:28`). Step mode loads that joined player from the next MAME state; it does not prove
the join transition is ported. All three complete without a panic. Unported objects and video/sound
still differ. `re/harness/regress.sh mp2 mp3 mp_join` reruns these comparisons; they are also in its
default list. Traces and diff reports stay under ignored `re/out/traces/`.

### Multiplayer scenarios M1 to M10 (phase 2 task M2-30)

56 reference scenarios covering the two and three player mechanics of D2c section 19
(groups M1 through M10). Every scenario has a headless MAME recording (`NAME.mame.trc` and
`NAME.state`), an input script in `scripts/NAME.txt`, and a poke setup in `pokes/NAME.lua`.
Every scenario runs in `port.sh` without a panic. Differences in unported combat passes,
tag team mechanics, boss routines, and item handlers are cataloged for future tasks
(M5-20, M9-01, M9-02).

Record any scenario with:

```sh
DINO_BOOT=re/harness/scripts/BOOT.txt DINO_POKE=re/harness/pokes/NAME.lua DINO_SYNC=control+86 \
  re/harness/record.sh re/harness/scripts/NAME.txt NAME FRAMES
re/harness/port.sh re/harness/scripts/NAME.txt NAME FRAMES
re/.venv/bin/python -I re/harness/diff.py re/out/traces/NAME.mame.trc re/out/traces/NAME.port.trc
```

| Group | Topic | Scenarios | Boot | Coverage and expectations |
| --- | --- | --- | --- | --- |
| M1 | Friendly fire fist windows | `ff3_n0`..`ff3_n4`, `ff2_n0`..`ff2_n3` | `boot_3p` / `boot_2p` | Friendly fire fist gate based on alive enemy census count: 0 thugs dz in [-1,0,1], 1 thug dz in [0,1], 2 thugs dz=0 only, 3+ thugs disabled. |
| M2 | Multi-friend hit and HP floor | `ff3_multi`, `ff3_hpfloor` | `boot_3p` | One punch hits multiple friends in reach; specials cannot hurt friends; friendly fire cannot kill (stops at 1 HP). |
| M3 | Friendly fire with weapons | `ffi_pistol`, `ffi_uzi`, `ffi_shotgun`, `ffi_club`, `ffi_torch`, `ffi_butcher`, `ffi_knifethrow`, `ffi_dyn`, `ffi_bazooka`, `ffi_stones` | `boot_2p` | Weapon friendly fire ignores enemy census count (always active at dz=0), inflicts 1 HP floor, misses at dz=1. |
| M4 | Tag team ties | `tie_jack_hannah`, `tie_dx31_dz0`, `tie_dx32_dz0`, `tie_dx0_dz15`, `tie_dx0_dz16`, `tie3_all`, `tie_weapon`, `tie_inputbreak` | `boot_2p` / `boot_3p` | Stillness counter (90 frames) triggers tie at t=92; boundaries dx in [-32,31] and dz in [0,15]; input resets timer; lower player index priority in 3P. |
| M5 | Partner run attack | `tierun_hannah_hit`, `tierun_hannah_nodir` | `boot_2p` | Held direction during partner throw executes action 0x1A run attack (18 dmg to thugs); neutral stick executes standard jab. |
| M6 | Contact and overlap | `pp_walk`, `pp_both`, `pp_run`, `pp_z`, `pp_jump`, `pp_three` | `boot_2p` / `boot_3p` | Players walk, run, and jump through each other without collision boxes or pushing. |
| M7 | Boss HP scaling | `boss_vice_2p_r0`, `boss_vice_3p_r0`, `boss_vice_2p_r24`, `boss_vice_3p_r24` | `boot_2p` / `boot_3p` | Vice HP scaling by player count and rank (2P: 550 r0, 700 r24; 3P: 650 r0, 750 r24). |
| M8 | Player count and death | `dead_p2_of2`, `dead_p3_of3`, `dead_p2_of3`, `cnt_e1a0_2p`, `cnt_e1a0_3p` | `boot_2p` / `boot_3p` | Player count G.$4EF drops exactly 61 frames after final death; multiplayer wave threshold adjustments. |
| M9 | Same-frame weapon pickup | `pick_same2`, `pick_same3`, `pick_next2`, `pick_next2b` | `boot_2p` / `boot_3p` | Simultaneous attack press on weapon gives item to earliest updated player; other players perform normal attack. |
| M10 | Dive window and triangle kick | `dive_jack_22`, `dive_mustapha_21`, `dive_hannah_22`, `dive_mess_22`, `wja_hannah_10`, `wja_mustapha_10` | `boot_2p` / `boot_3p` | Dive timing window (21 to 25 frames after jump) across all four heroes; wall jump triangle kick. |



## Trace format

`re/harness/trace.py` documents it and reads it. A trace is a header (JSON) and one record per frame;
a record holds only the 64-byte blocks that changed since the previous one. MAME side regions:
`ram` (64 KB at 0xFF0000), `gfx` (192 KB at 0x900000: sprite lists, tilemaps, palettes), `cpsa`, `cpsb`.
Port side: `ram` plus `ram_mask` (which bytes the port models), `gfx` plus `gfx_mask`, `obj` (the
sprite list built this frame), `obj_src` (the object that drew each entry), and the sound requests
of the frame in each record's JSON. The port header lists every traced field (`fields`, `globals`) and
the ported classes (`ported`).

The snapshot (`NAME.state`) is a trace file with one record: all of RAM and video RAM at the sync
point. `dino-game --load-state` fills the port from it by the same field lists (`trace.rs`).

## Port side (dino-game)

```
dino-game --rom dino.zip --load-state NAME.state  --input-file SCRIPT --frames N --trace OUT     # run
dino-game --rom dino.zip --load-state NAME.state  --follow-rng NAME.mame.trc ...                 # follow
dino-game --rom dino.zip --step-from  NAME.mame.trc --input-file SCRIPT ...                      # step
```

- `src/harness.rs`: reader and writer of the trace files, the headless run loop.
- `src/game/trace.rs`: every modeled field once, by original offset; the same list is used to
  write the trace and to load the snapshot. Fields the port keeps in another form (frame ids instead
  of ROM pointers, indexes instead of list pointers) are converted both ways there.
- Frame records of animations the extractor does not know are decoded on demand: before the game is
  built, `harness::scan_frames` reads the frame pointers of the snapshot's objects and
  `dino_extract::extract_with` decodes them. The game itself never reads ROM bytes.
- Objects of unported classes are loaded and kept frozen (`Game::trace_hold`) so their position and
  draw state are in the trace; when you port a class, add it to `PORTED_CLASSES` in `trace.rs`.

## Known limits

- Only what is in `trace.rs` is compared (see NOT MODELED). Pan / volume of sound requests, scroll
  registers and task state are not compared yet.
- A `run` that starts with unported classes on screen diverges as soon as they act (they call
  `rand()`, free their slots, kill, spawn). `follow` and `step` exist for that. Sync after the
  scripted opening (`control+N`) to keep the run clean.
- MAME runs with zero-filled placeholder PLD and QSound DSP ROMs: game state is exact, audio is not.

## Timed power-on writes and flow ground truth (M2-00, M2-31)

`DINO_SYNC=power` records every video frame from power on. The input file uses
power-on frame numbers, as the probe does. Record -1 is power frame 1; record 0
is power frame 2. The `vblank` column gives the power-on frame directly.
`DINO_POKES=file` works with every sync mode. Its clock always counts video
frames from power on, including frames before the sync point and slowdown.
A frame-0 write is applied at the first video callback, before frame-1 writes.
Writes for a frame run in file order before the trace sample and before the
next frame's CPU work. They do not wait for a game-frame sync point.

```
# frame hexaddr size value
1300 FF84D9 1 1
1300 FF84D1 1 0
1300 FF84D0 1 8
1800 FF84E8 2 0x0013
```

Frames are decimal, addresses are full hexadecimal RAM addresses without `0x`,
sizes are 1, 2 or 4, and values are decimal or `0x` hexadecimal. Blank lines
and `#` comments are allowed. Out-of-order frames are sorted while preserving
file order within a frame. Bad lines, non-integer values, overflow, non-RAM
addresses, odd word/long addresses and missing files fail the recording.
This is separate from the older `DINO_POKE=file.lua` sync-point hook, which
still runs once before the state snapshot.

```sh
DINO_SYNC=power DINO_POKES=re/harness/pokes/flow_story_power.txt \
  re/harness/record.sh re/harness/scripts/flow_play.txt flow_story_power 2000
DINO_SYNC=power DINO_POKES=re/harness/pokes/flow_clock_power.txt \
  re/harness/record.sh re/harness/scripts/flow_play.txt flow_clock_power 2200
```

The clock write is at 1800 so Episode 1 has released scripted control. The
story write uses the frame-1300 recipe from NEW-PLAN section 5. These two files
are plain timed poke files, ready for `DINO_POKES` or probe `PR_POKES`.

### Recording the acceptance groups

Each `flow_GROUP.txt` input has independent `segment NAME` blocks. Its matching
poke file has the same segment names, with timed writes in the format above.
`record.sh --batch` validates and merges both files, then starts one headless
MAME process for the entire group. Never launch the groups concurrently.

```sh
for group in system join_continue select_name tally links ending panels camera end_chain; do
  re/harness/record.sh --batch re/harness/scripts/flow_$group.txt \
    flow_$group re/harness/pokes/flow_$group.txt || break
done
```

| Group | Scenarios | Acceptance coverage |
| --- | ---: | --- |
| system | 7 | D4d A10, A11, A12: credit, start, no credit, free play, held coin |
| join_continue | 15 | D4d A14, A15, A16: join strips, timeout, mash, late coin, second chute, game over |
| select_name | 11 | D4d A13, A17: moves, late join, collision, 3P timeout, simultaneous picks, typed CB name |
| tally | 31 | D4d A18: HP boundaries, kills, final blow, 1P/2P/3P, M.V.P. and damage ties |
| links | 7 | D4d A19: Episodes 2 through 8 panels and cards |
| ending | 6 | D4d A20, A21: all heroes, 3P, score set before ending name entry |
| panels | 32 | D4d A22: solo and mixed parties, Episodes 2 through 8, attack and jump skip |
| camera | 10 | D4c A1, A2, A7, A8: Episode 1 opening, five simple openings, Ep8 3P, GO idle/moving, Vice defeat |
| end_chain | 8 | D4c A9: poked end flow for every episode, including the long Episode 8 ending timer |

All nine groups were recorded and audited on 2026-10-13: 127/127 unique
scenarios, 419,550 video records and 625 selected pictures. This includes a
clean system rerun and the eight groups left pending by commit `2136d09`.
Each group ran in one headless MAME process, sequentially, with SDL dummy
drivers. Every trace parses, has continuous record and power-frame numbers,
and matches its recorder count. Every segment has a completion line.
The audit is in `re/out/m2-31-inventory.log`.

The batch runner uses the existing probe's input, hold, HP, score and runtime
jump helpers. Each segment starts at the saved first power-on video frame.
The trace includes boot and the entire assisted jump. No ROM tables, camera
event copies or snapshots are embedded in these scripts. `jump` reads the
user's ROM in MAME and streams the area as described in capture-recipes section 2.
The scripts retain the design's post-jump timing rather than replacing it with
an estimated power-on offset. With `jump`, timed writes and key lines use `t`
from the first frame after that jump; without `jump`, they use power-on frames.
The log gives both `t` and `f`. A `watch` logs the game-frame count `gf` too.
These batch scripts are probe recipes; they cannot be passed directly to the
port's plain `--input-file`. The full power traces and logged setup actions are
available for M3 scenario tooling and for comparing the matching segment.

The recorder loads a copy of `probe.lua` with checked extension points for
segment begin, video sample, segment end and fatal-error exit. The probe source on disk is not
changed. A Lua error exits the batch and makes the shell command fail. A missing
extension point fails rather than silently recording a different time base.
`DINO_TIMEOUT` limits a process to 900 seconds by default. Normal recordings
must log their full requested frame count; a partial trace is an error.

Outputs are ignored artifacts:

- `re/out/traces/flow_NAME.mame.trc`: RAM, video RAM and CPS registers every video frame from power on.
- `re/out/flow/flow_GROUP/NAME/log.txt`: setup actions, mode changes and watches with `t`, `f`, `gf`.
- `re/out/flow/flow_GROUP/NAME/snap/`: selected pictures in the design's time base.
- `re/out/flow/flow_GROUP/segments.txt`: one completion line per scenario.
- `re/out/flow/flow_GROUP/mame/record.log`: recorder completion counts and MAME messages.

For batch traces record -1 is the first sample; later record `frame` indexes
start at 0. Use `vblank` for the absolute power frame and the probe log to locate
the post-jump acceptance frame. After the first segment, the saved first frame
is restored. Its cached trace snapshot supplies record -1, followed by live
samples starting at power frame 2. This matches the first-frame restore used
by the probe and keeps a complete power-on trace for every segment.

G1 and G2 use the shared Rust tree without edits. G3 is the MAME trace check
and complete batch inventory. G4 uses `flow_play.txt` from
power on with pictures under `re/out/shots/`. G5 is a tool task: no ported game
routine and no extracted ROM data is added.

## Probe segments as scenarios (`snapshot`, `trace`, `probe_record.sh`; M3-06)

Any segment of a probe file (`re/harness/probe/probe.lua`, manual: `plan/game/design/capture-recipes.md`) can be
recorded as a harness scenario. The lines before a `snapshot` line build the scene (`jump`, `poke`, `spawn`,
`hold`, `god`, `hp`, key lines, ...). At the `snapshot` line the whole RAM and video RAM are saved as a `.state`
file. `trace N` then records N game frames, and the key lines after the snapshot are the input script. So the
port can load the state (`--load-state`) and play the same keys (`port.sh`), and `diff.py` compares them like any
other scenario. One MAME process records all the chosen segments (headless, SDL dummy drivers).

```
segment fight                        # in a file such as re/out/my_scenarios.txt
jump 1 0 char=jack                   # boot and start Episode 1 (frame numbers count from the end of the jump)
god on                               # Jack cannot die while the scene is built
5 spawn A 20 var=0x0004 x=+0x80      # a Punk walks in
40 snapshot                          # the setup ends at frame 40; god and holds stop here
0-9:                                 # key lines after it: frame 0 = the first game frame after the snapshot
10-19:R
24-25:A
trace 240                            # record 240 game frames
```

```sh
re/harness/probe/probe_record.sh re/out/my_scenarios.txt fight     # SEGMENT, a,b,c or all; third argument: NAME
MODE=step re/harness/port.sh re/out/traces/fight.input.txt fight 240
re/.venv/bin/python -I re/harness/diff.py re/out/traces/fight.mame.trc re/out/traces/fight.step.trc
```

`probe_record.sh BATCH SEGMENT|a,b,c|all [NAME]` writes, for every segment, into `re/out/traces/`:
`NAME.state` (the snapshot, record -1 of the trace too), `NAME.mame.trc` (the same trace file as `record.sh`
writes: snapshot, then N game frames, lag frames flagged) and `NAME.input.txt` (the key lines after the snapshot, in
the plain script format of `--input-file`). `NAME` is the segment name, so give a segment a name that no hand made
scenario uses. `all` means every segment that has a `snapshot` line. A segment that fails (bad `jump`, snapshot
not reached, Lua error) leaves no files and makes the script exit 1. `check.sh` only reads scripts from
`re/harness/scripts/`, so run `port.sh` and `diff.py` as above. Logs and screenshots go to
`re/out/probe/BATCH-rec/SEGMENT/`. `PR_SNAPS`, `PR_WATCH` and `watch` lines work as in the probe.

| line | meaning |
| --- | --- |
| `[FRAME] snapshot [control\|control+N] [keep]` | Ends the setup. The state is taken at the first game frame boundary (vblank IRQ, not a slowdown frame) at or after FRAME. `control`: wait until player 1 can be controlled (the `DINO_SYNC=control` rule), `control+N`: N game frames later. `keep`: holds and `god` go on during the trace (the port cannot repeat them). Without `keep` they end at the snapshot: `god` like `god off` (`+$00` back to 1; `+$F9` keeps its value), holds are dropped. |
| `trace N` | Record N game frames after the snapshot (default 600). |
| key lines `a-b:KEYS` after `snapshot` | The input script. Frame 0 is the first game frame after the snapshot, as in `scripts/*.txt`. Key lines before the snapshot are setup input (see the clock below). |
| other directives after `snapshot` | Run at their game frame after the snapshot (`watch`, `log`, `snap`, `peek` are fine). A `poke`, `spawn`, `hp` ... changes MAME only: the port does not see it (the log says `NOTE`), so in `step` mode that frame differs. |

Clock. The trace is on the game clock like `record.sh`: one record per finished game frame at the vblank IRQ
(the write of the frame flag at 0xFF0152). The probe itself runs on video frames, so the setup lines (`N poke`,
`N snapshot`, `watch`) count video frames after the `jump`. Key lines before the snapshot of a segment **without**
`jump` count IRQ ticks from power on, the clock of the boot scripts (`boot_jack.txt`): at the start of a stage
that is about 70 fewer than video frames. This makes the state bit for bit the one `record.sh` makes:

```
segment eq_stage1_jack               # boot_jack.txt, then the scenario script of stage1_jack.txt
600-607:C
700-705:S
900-905:A
snapshot control
0-29:
30-59:L                              # ... the rest of scripts/stage1_jack.txt
trace 300
```

Checked 2026-10-13 (MAME 0.289): this segment gives `eq_stage1_jack.state` and 300 frames equal to the first 300 frames
of the existing `stage1_jack` reference (every record, all four regions, the vblank and lag fields: 0 differences),
and `diff.py` against the port's step trace of `stage1_jack` shows the same report as for the reference itself
(0 ported-class pairs, 0 globals). The recorder `record.sh` with the changed `record.lua` writes the same bytes as before,
and so do `record.sh --batch` (flow batches) and old probe batches with the changed `probe.lua`.

A `jump` scene is not the same state as `DINO_SYNC=control`, so `jump 1 0 897 char=jack` + `snapshot` does not equal
`stage1_jack` (897 is the camera X of that frame). Why: `jump` boots on video frames (the attract timers in RAM differ,
for example `splash_phase`), it ends two game frames after the first controllable frame, its `god` end sets Jack's `+$00`
to 1 (it is 2 at that frame of the real run), and a CAMX moves the players to CAMX + 0x60 (Jack's X 0x420 becomes 0x3E1).
It works as a scenario of its own: `jump 1 0 897 char=jack` + `snapshot` + the `stage1_jack` keys + `trace 300` runs in
`MODE=step` with 0 ported-class pairs and 0 globals.
A scenario with spawned thugs, `god`, `hp` and a fight (`jump`, `spawn A 20`, `40 snapshot`, 240 frames) also gives 0 ported-class
pairs; the only global difference is a `poke` placed after the snapshot, as designed.

How it is built: `probe.lua` loads `record.lua` as a library (`DINO_LIB_ONLY`, it then returns its trace writer and
runs nothing else), so one writer makes both kinds of trace. The IRQ tap exists only when a segment has a `snapshot` line;
other probe runs and `record.sh --batch` keep exactly the taps they had. `probe_record.sh` passes the paths in
`PR_RECORD_LUA`, `PR_TRACE_DIR`, `PR_ONLY` (segment names or `*`), `PR_TRACE_NAME` (output name for one segment) and
`PR_SNAP_MAXF` (a segment fails when its snapshot is later than this many setup frames, default 12000; `frames N` does not
apply to a scenario).

## Picture compare (`shotdiff.py`)

Compares MAME probe screenshots with port screenshots of the same frames. Prints the percent
of pixels that differ on each frame, and writes a side by side contact sheet of the worst
frames (MAME on the left, the port on the right). Screen tasks use this for the picture check.

```sh
re/.venv/bin/python -I re/harness/probe/shotdiff.py \
    re/out/probe/recipes/fe-boot re/out/shots/fe-boot \
    --frames 10,60,100,130,170,250,310
```

`MAME_DIR` is a probe folder (`log.txt` and `snap/0000.png` .., 384 x 224: capture-recipes
section 5). The `SNAP` lines map each file to a frame. The file number is the snap order, not
the frame. `PORT_DIR` is the same kind of folder, or a folder of pictures named by frame
(`f00100.png`, `0100.png`, `100.png`, `frame_100.png`). One png file is accepted when
`--frames` names a single frame (the current `--screenshot out.png` run).

`--frames` uses the probe snap text: `100,250,330` or `every:10:4600:6500` (both ends included).
With no `--frames`, every frame present on both sides is compared. `--clock t` (default) is the
frame in the input file (`t=` in the log). `--clock f` is the video frame from power on. They
are the same when the probe has no jump.

The port window is 3 times 384 x 224. A picture that is an integer multiple of the other is
reduced to game pixels before the count: the center pixel of each block when the scale is odd,
the top left pixel when the scale is even. Any other size is resized with nearest neighbor and
the line says so.

One line per frame: `frame 100  0.00%  0/86016`. Then `first difference:` (the earliest frame)
and either `all equal (0 percent)` or `worst:`. Exit 0 when every frame is 0 percent. Exit 1
when a pixel differs or a picture is missing. Exit 2 when the arguments are wrong. The contact
sheet is `PORT_DIR/shotdiff.png` unless `--out` is set. `--worst 8` (default) is how many
frames are on it, highest percent first. `--worst 0` skips the sheet.

### Completion report (M2-31, 2026-10-13)

Recorded the remaining eight groups and reran system cleanly. The existing
input and poke files needed no changes. Updated this README with the finished
inventory. No edits to dino-rs, probe.lua, or the recorder. No commit or push.

- G1: `cargo build` passed; `cargo test` passed 144 tests, with one ignored
  and zero failed. Logs: `re/out/m2-31-build.log` and `m2-31-test.log`.
- G2: `regress.sh` completed all 52 scenarios, with 48 at zero ported-class
  pairs. Baseline and current counts are the same: `mp_join` 37/37, `armed`
  4/4, `intro` 14/14, `c14c_mustapha` 8/8. Baseline sources are this README's
  multiplayer section and `plan/phase-01-stage1-complete.md`. Known causes
  remain join/drop-in logic (first difference 49), the unported prop's hit
  and piece spawns (564), scripted intro spawns and player actions (62), and
  hit-spark pool slots affected by unported dizzy stars (836). Full result:
  `re/out/m2-31-regress.log`. No new allow rules.
- G3: all 127 batch traces passed the inventory audit. The clean coin/start
  trace retains in-game at power frame 601, mode 4 at 623 and sub-mode 2 at
  625; free-play select starts at 523. The story check recorded 2000 frames
  plus sync: all three writes apply at 1300, mode 4 follows at 1606 and mode
  0x0C at 1607. The clock check recorded 2200 frames plus sync: the word is
  0x0013 at 1800, with scripted control and clock freeze both zero, then
  counts down to 0x0005. The temporary `m2-31-stage1_jack-60` recording equals
  the existing `stage1_jack` reference in every region for all 60 frames plus
  sync; there is no first differing frame. The original reference was kept.
  Check results: `re/out/m2-31-pending-checks.log`. These are MAME ground
  truth recordings; port equality for the 127 flow scenarios belongs to the
  corresponding implementation tasks.
- G4: `flow_play.txt` completed 2200 frames from power on in the headless port.
  The windowed screenshot attempt failed before drawing, with macOS
  LaunchServices initialization failure and a miniquad null-reference panic
  in `apple_util.rs`. Logs: `re/out/m2-31-play-headless.log` and
  `m2-31-play-2200.log`. The port screenshot gate remains blocked. Fresh MAME
  pictures were inspected and copied to `re/out/shots/m2-31-*.png`: the
  Butcher story portrait and text, the rooftop fight with the red 0:07 clock,
  and the Episode 3 map with HELL ROAD. The select picture shows Jack's
  panel, TIME 09 and the 2P/3P panels. MAME pictures do not complete G4.
- G5: no ported routine or ROM data was added. All traces, snapshots and
  diagnostic logs remain ignored artifacts under `re/out/`.

The earlier hung PID 39209 no longer existed before recording started. All
12 MAME processes in this run exited normally: nine group processes, two
timed-poke checks and one compatibility recording. No timeout or process
that could not be stopped remains from this run. MAME's checksum warnings
come from the existing placeholder PLD and QSound DSP files described above;
audio remains outside these recordings' proof.

Open question: G4 still needs a port screenshot run in a macOS session where
miniquad can initialize its window. The M2-31 recording work is complete.
