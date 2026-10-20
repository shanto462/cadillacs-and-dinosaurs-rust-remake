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

### Four ways to run the port (`MODE=...`)

| mode | command | what it checks |
| --- | --- | --- |
| `run` (default) | `port.sh SCRIPT` | the port plays on its own from the snapshot. The real test: it must match for the whole run. A difference at frame k also causes differences after k. |
| `follow` | `MODE=follow port.sh SCRIPT` | like `run`, but the random number state `G.$14` is copied from MAME before every frame. Objects of classes the port does not run still call `rand()` in the original; this stops that drift from hiding the ported code. Writes `NAME.follow.trc`. |
| `step` | `MODE=step port.sh SCRIPT` | before every frame the port is loaded with MAME's state after the previous frame, so a difference at frame k comes from frame k alone, never from an earlier difference. Use it to find out why something differs. Writes `NAME.step.trc`. |
| `lagsync` | `MODE=lagsync port.sh SCRIPT` | like `run`, but after every MAME lag record (`flags & 1`: the original's game frame overran a vblank, one port tick cannot replay it) the port is loaded with MAME's state of that record, the state a step run starts the next frame from. The lag records are not compared (diff.py, as in `step`). The header has `lagsync_resyncs` (the number of resyncs). Writes `NAME.lagsync.trc`. Use it for a free run whose first difference would otherwise be a lag record. |

Since M10-08 a record in which MAME ran another number of game frames than the port (G.$0 moved by another step in the record) is a resync point too, not only a lag record: the skip wait of `flow/skip.rs` runs a fixed 3 rounds a tick in the port while the original runs as many as the CPU time allows, and a story panel can start inside a lag record. The record itself is written before the resync, so its difference shows (and is listed with its cause in `allow/m10.txt`); the frames after it start from MAME's state again. `DINO_LAGSYNC_TICKS=0` turns this off (lag records only). The whole run checks of Episodes 2 and 3: `re/harness/m10_check.sh` (step, lagsync and run of the natural recordings, the allowed differences in `allow/m10.txt` by recording and mode).

`lagsync` is a free run between the lag records: a difference after a resync is a logic or state difference of the port, not the lag. It is not exact like `step`: free runs drift in state that a resync does not restore (for example stale bytes of free slots), so a difference can still appear a few frames after a resync. Find those with the diagnostic `DINO_LAGSYNC_EVERY=N` (resync every N frames as well; N = 1 is `step`; a smaller N moves the first difference later).

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
dino-game --rom dino.zip --load-state NAME.state  --lagsync NAME.mame.trc --input-file SCRIPT ... # lagsync
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

## Recorder options: write watch and ROM directory (flow fixes, 2026-10-17)

- `DINO_WATCH_WRITE=ADDR` (hex RAM address, for example `FF84FC`): `record.lua` and `probe.lua` log every CPU write to the
  even word that holds `ADDR`, as `WATCH frame N addr A data D mask M pc P a6 A` in the MAME log (`record.log`, or
  `re/out/flow/NAME/mame/record.log` for a batch). `pc` is the instruction after the write (about one instruction late),
  `a6` the object or routine base. Use it to find which routine writes a variable. Frame numbers are MAME screen frames:
  they are one or two records off the trace records, so match the write by value and order, not by the record number.
  Each hit costs one line; for a busy byte (the frame flag at 0xFF0152) the log grows past 100k lines, so watch
  only quiet variables. One batch run per address.
- `DINO_ROMS=DIR` (absolute path): an extra ROM directory for `record.sh`. A git worktree has no `re/mame/roms`, so use
  `DINO_ROMS=/path/to/main/re/mame/roms`.

Example (Episode 4 link, one segment, from the flow_links script cut to one segment):
`DINO_ROMS=... DINO_WATCH_WRITE=FF84FC sh re/harness/record.sh --batch SCRIPT.txt NAME POKES.txt`. A batch overwrites the
segment traces of `re/out/traces` (`flow_<segment>.mame.trc`); copy them first if the run must not change them.

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

## Episode 2 and 3 preparation scenarios (M6-00)

The four `pokes/m600_*.txt` files are probe scenario batches for M6-00. Each
segment and input script has the `m600_` prefix. They can record alongside
other workers without sharing scenario names or MAME output directories.
`scripts/m600_scenarios.tsv` lists the case, scenario name, batch, trace
length, recipe and timed mutation frames. Its rows are inputs and metadata,
not trace data. The batches are self contained and do not need the old
research generators under `re/out/d3a`, `d3b` or `d3c`.

| Batch | Cases | Scenarios |
| --- | --- | --- |
| `m600_humans.txt` | D3a H1 to H27, including the variants in section 14 | 83 |
| `m600_dinosaurs.txt` | D3b A9 to A11 and A16 to A21 | 13 |
| `m600_bosses.txt` | D3c BS-B1/B2, BS-H1, BS-C1 to C8, BS-V1 to V3, BS-E1 | 35 |
| `m600_replay.txt` | Companions for hunter, poacher, Lash, Blade, carriers, hazards, Mack, Shivat, Butcher, Hogg and car | 18 |

The first three batches preserve the probe controls of the design recipes.
Their snapshots use `keep`. God mode, rank holds and pins can continue;
spawns, HP resets, forced states and kill-all commands after the snapshot
change MAME only. `mutation_frames` lists the timed commands. They are
reference timelines for preparation, not proof that a free port run equals
MAME. Use step mode to examine a routine, with those external changes in
mind. A kept god control refreshes invulnerability each frame; a kept pin
can override movement each frame. Neither is replayed by `port.sh`.

The `_replay` companions build the objects before the snapshot, drop probe
controls at the snapshot, omit later mutations and replay only keys. Holds
stop refreshing their fields; the saved values remain in the state. Their
short setup can include keys. Input after the snapshot starts at frame 0.
These are separate scenarios: ending god mode and pins changes
the encounter, so do not read their frames as the design timeline's frames.
The original probe uses video frames; the recorder uses game frames after
the snapshot. Lag can also change the timing relative to the old probes.

Record one headless MAME process per batch, sequentially:

```sh
for batch in humans dinosaurs bosses replay; do
  PR_TIMEOUT=3600 PR_SNAPS=none PR_SHEET=0 \
    re/harness/probe/probe_record.sh re/harness/pokes/m600_${batch}.txt all
done
```

Run every input through the port from the recorded state:

```sh
tail -n +2 re/harness/scripts/m600_scenarios.tsv |
while IFS="$(printf '\t')" read -r case_id name batch frames recipe mutations; do
  MODE=step re/harness/port.sh re/harness/scripts/${name}.txt "$name" "$frames"
done
```

Compare a chosen scenario without allow rules:

```sh
name=m600_h1_hun_v400_replay
re/.venv/bin/python -I re/harness/diff.py \
  re/out/traces/${name}.mame.trc re/out/traces/${name}.step.trc
```

The `.state`, `.mame.trc`, `.step.trc` and recorder-generated `.input.txt`
files stay under ignored `re/out/traces/`. Batch logs stay under
`re/out/probe/m600_*-rec/`. For these cases, use `probe_record.sh` to record:
`record.sh` and `check.sh` do not reconstruct their probe scene setup.
The same input files can be used with `MODE=run` once the relevant classes
are implemented, especially the replay companions.

## Player moves and respawn preparation scenarios (M5-15)

The scenario batches in `re/harness/pokes/m515_moves.txt` and `re/harness/pokes/m515_respawn.txt`
prepare ground truth MAME recordings for D2a scenarios A1 to A14 across all four heroes
(Mustapha, Hannah, Mess, and Jack as control) and D2b scenarios S1 to S21.
Every scenario name is prefixed with `m515_` so recordings never clash with parallel workers.
`re/harness/scripts/m515_scenarios.tsv` catalogues all 90 scenarios, batches, and frame counts.

| Batch | Scope | Scenarios |
| --- | --- | --- |
| `m515_moves.txt` | D2a A1 to A14: idle, walk, run, double tap, jumps, jump attacks, running jumps, chain, combo throw, grabs, special, dash, command moves, wall jump | 55 |
| `m515_respawn.txt` | D2b S1 to S21: lost life respawn (4 heroes), air strike timeline, strike on enemies, HP floor, blast reach, boss/monster strikes, item/TNT reactions, multiplayer respawn, join with coin, continue, time over invulnerability, hero damage scaling, hurt sequence, lie/mash, dizzy, 1UP, HUD lives/bars, HP carry, score items/kills, time over | 35 |

Record the batches in headless MAME using SDL dummy drivers:

```sh
re/harness/probe/probe_record.sh re/harness/pokes/m515_moves.txt all
re/harness/probe/probe_record.sh re/harness/pokes/m515_respawn.txt all
```

Replay each scenario through the port in step mode:

```sh
tail -n +2 re/harness/scripts/m515_scenarios.tsv |
while IFS="$(printf '\t')" read -r name batch frames topic; do
  MODE=step re/harness/port.sh "re/harness/scripts/${name}.txt" "$name" "$frames"
done
```

Compare with diff.py:

```sh
re/.venv/bin/python -I re/harness/diff.py \
  re/out/traces/m515_a2_walk_mustapha.mame.trc re/out/traces/m515_a2_walk_mustapha.step.trc
```

All 90 scenarios produce complete traces (`.state`, `.mame.trc`, `.input.txt`) and run cleanly in `port.sh` without panics.
Jack move recordings reproduce existing move behavior with 0 ported-class pairs and 0 globals differing.

### M6-00 completion report (2026-10-13)

Prepared four batches, 149 input scripts and the scenario manifest. All 51
requested acceptance IDs have references. H16 was built from its section 14
recipe; H6 also has a level 31 variant. The 18 replay companions end probe
controls at the snapshot. No game code, recorder code or allow rules changed.

- G1: `cargo build` passed. `cargo test`: 153 passed, 0 failed, 1 ignored.
- G2: skipped by the lane B task instruction. No regression claim.
- G3 recording: 149 complete MAME scenarios, 256,968 game frames and 1,143
  lag vblanks. All trace record numbers and lengths were checked. All 149
  input scripts match the recorder output. Four sequential headless MAME
  processes recorded the four batches; all exited normally.
- G3 port comparison: all 149 scenarios were attempted through `port.sh`
  in step mode, without allow rules. 133 port traces completed. Fifteen
  car traces stopped after 363 game frames; the car replay stopped before
  frame 0. All have the same bounds panic at `dino-assets/src/stage.rs:87`,
  requesting area 1 from a stage with one area, during `objects_update`.
  The total compared port output contains 224,979 game frames. The car
  replay's comparer exit 0 checks only its sync record and is not a pass.
  No completed scenario is equal to MAME with the current port.
- G4: game run and game screenshots skipped by the lane B instruction.
- G5: only scenario commands, inputs and documentation were added. No
  routines or ROM tables were ported. Runtime traces and states stay ignored.

The first differences of the 148 comparisons with a difference are at
frame -1 for 16 cases, 0 for 40, 1 for 63, 2 for 9, 5 for 12, 6 for 7,
and 27 for 1. Kept god mode causes the 63 frame-1 invulnerability differences.
Timed MAME-only spawns and kills cause allocation and census differences.
Unported enemies, effects, scene actors and car behavior account for the
remaining target-class differences. The replay companions remove ongoing
probe controls but still need those classes to be implemented.

Open shared-code questions: the snapshot loader reconstructs different spawn
pointers in the 16 frame-minus-one cases; the HUD has text-layer differences
in the bullet and Episode 3 carrier cases; the Episode 5 hunter walk starts
with floor-item motion, animation and RNG differences. Their precise causes
need their owners. The car area-index panic also needs the camera/stage
owner. Preparation is complete; port parity remains implementation work.

Detailed evidence is under `re/out/probe/_input/`:
`m600_audit.log`, `m600_build.log`, `m600_test.log`, `m600_results.tsv`,
`m600_first_differences.tsv`, `m600_car_backtrace.log`, and each scenario's
`.port.log` and `.diff.log`. MAME's PLD and QSound checksum warnings are the
existing placeholder-ROM warnings noted in the harness limits above.

## Episode 4 and 5 preparation scenarios (M7-00)

The five `pokes/m700_*.txt` batches prepare all 39 requested acceptance IDs.
Every scenario name starts with `m700_`. The batches are self contained:
the old research recipes are cited for provenance, but are not loaded when
recording. `scripts/m700_scenarios.tsv` lists each case, scenario, batch,
frame count, recipe, timed mutation frames and kept probe controls.

| Batch | Cases | Scenarios |
| --- | --- | --- |
| `m700_bosses.txt` | D3c BS-S1/S2; D3d BT-A1 to BT-A5 | 11 |
| `m700_humans.txt` | D3a H17, H18, H26, Bludge item eating and return | 3 |
| `m700_dinosaurs.txt` | D3b A1 to A8, A12 to A14, including all six punch and jump heights and both hostile levels | 25 |
| `m700_scenery.txt` | D3e part 2 A207 to A224, with all eight bridge barrels, scenery controls, placed container hits and player-count and level gates | 99 |
| `m700_replay.txt` | Companions for Bludge, Zeke, Hopper, Slice, Morgan's scene, garage breakables and the last pail | 14 |

The first four batches preserve the design recipes with `snapshot keep`.
Timed spawns, forced states, HP resets and camera/player pokes after the
snapshot affect MAME only. God mode, rank and level holds, kill-all holds
and background clock freezes can also continue through a trace. `port.sh`
does not replay those commands. These are preparation references, so a
port difference at a timed mutation is not evidence of a routine bug.
The manifest names those frames and controls. Frame numbers after a
snapshot count game frames; the old recipes counted video frames.
Slowdown can shift events relative to the old design timelines.

The replay companions finish their setup before the snapshot and end
probe controls there. They contain only keys after the snapshot. Saved
field values remain in RAM, but are no longer refreshed by a probe hold.
These companions are separate encounters with their own frame zero.

A219 records the partial natural crossing from D3e part 2. That recipe
establishes the start of the diagonal descent; it does not prove a full
uninterrupted bridge crossing. A224 adds 1P, 2P and 3P presence checks for
the gated yard, garage, meadow, cliff and torch records. A214 adds the gun
yard player-count and level boundary checks. The final pail also has level
16 and 17 hit scenarios. These probes let the original stage scripts
choose their records at run time. No table rows are copied into the batch.

Record each batch with one headless MAME process, sequentially:

```sh
for batch in humans dinosaurs bosses scenery replay; do
  PR_TIMEOUT=3600 PR_SNAPS=none PR_SHEET=0 \
    re/harness/probe/probe_record.sh re/harness/pokes/m700_${batch}.txt all
done
```

Run the recorded scenarios through the port in step mode:

```sh
tail -n +2 re/harness/scripts/m700_scenarios.tsv |
while IFS="$(printf '\t')" read -r case_id name batch frames recipe mutations controls; do
  MODE=step re/harness/port.sh re/harness/scripts/${name}.txt "$name" "$frames"
done
```

Compare one scenario without allow rules:

```sh
name=m700_a3_z_force_c_replay
re/.venv/bin/python -I re/harness/diff.py \
  re/out/traces/${name}.mame.trc re/out/traces/${name}.step.trc
```

Use `probe_record.sh` for these scene setups. `record.sh` and `check.sh`
do not reconstruct them. States, MAME and port traces, generated inputs,
probe logs and validation evidence stay under ignored `re/out/` paths.
The input scripts and manifest contain commands and metadata only.

## Weapons and items preparation scenarios (M5-16)

The batches in `re/harness/pokes/m516_*.txt` (the same text is in
`re/out/probe/_input/`) record D2c W1 to W14 and D4a A01 to A25.
A26 and A27 are not here (task M9). Every scenario name starts with `m516_`.
`re/harness/scripts/m516_scenarios.tsv` lists the case, name, batch, trace
length, source segment, snapshot frame and the frames of MAME-only commands.
`re/harness/scripts/m516_*.txt` is the keys after the snapshot, for `port.sh`.

| Batch | Cases | Scenarios |
| --- | --- | --- |
| `m516_w_pick.txt` | W1 to W4 | 19 |
| `m516_w_melee.txt` | W5 to W9 | 58 |
| `m516_w_throw.txt` | W10 to W12 and W14 | 31 |
| `m516_w_life.txt` | W13 | 33 |
| `m516_a_food.txt` | A01, A02, A03, A04, A22 | 36 |
| `m516_a_box.txt` | A05, A06, A07, A08, A09, A10 | 32 |
| `m516_a_drop.txt` | A11 to A15 and A17 | 21 |
| `m516_a_long.txt` | A16 `life_long` | 1 |
| `m516_a_world.txt` | A18 to A21 and A23 to A25 | 25 |

256 scenarios. A shared segment is recorded once. A02 is the second eat
inside the A01 food segments. A06 is the `cj_*` part of A05. `stone_items`
is both A11 and A16. A22's food and treasure pictures are `f01`, `f09` and
`t01`, plus `hud_clean`.

Rows the design names for one hero stay that hero. Pistol, bazooka, knife,
stick, butcher knife, torch, dynamite and the barrel are recorded for Jack,
Mustapha, Hannah and Mess. Rifle, M-16, Uzi and the shotgun distances are
the Jack rows in W3. Grenade, stones, ammo and the heavy objects other than
the barrel are the Jack rows in W10 to W12. Other-hero copies that exist in
the old weapon batch but are not named in D2c section 19 were not recorded.

Lines before `snapshot` build the scene: the jump, the spawns, the pins,
god mode and the opening HP. `keep` is set when god or a pin is on, so MAME
keeps refreshing them. A later spawn, `objset`, `hp` or wear poke runs in
MAME only. `port.sh` does not replay it. In step mode the next MAME record
already contains the effect. In run mode the port diverges there. Trace
frame 0 is the first game frame after the snapshot. The `snap` column is
the old probe frame of that snapshot, so an old probe time is about
`trace frame + snap`. These recordings had no slowdown, so the clocks stay
within one step. A key written as frame N is the input of record N. The old
probes set the key at the end of a video frame and saw the result on the
next sample, so a design line such as "press at 10, state at 11" is one
trace record here.

Record one headless MAME process per batch:

```sh
for batch in w_pick w_melee w_throw w_life a_food a_box a_drop a_long a_world; do
  PR_TIMEOUT=7200 PR_SNAPS=none PR_SHEET=0 \
    re/harness/probe/probe_record.sh re/harness/pokes/m516_${batch}.txt all
done
```

Replay:

```sh
tail -n +2 re/harness/scripts/m516_scenarios.tsv |
while IFS="$(printf '\t')" read -r case_id name batch frames source snap muts; do
  MODE=step re/harness/port.sh re/harness/scripts/${name}.txt "$name" "$frames"
done
```

The `.state`, `.mame.trc` and recorder `.input.txt` files stay under ignored
`re/out/traces/`. Logs stay under `re/out/probe/m516_*-rec/`.

### M5-16 completion report (2026-10-13)

Nine batches, 256 input scripts and the manifest. Every W1 to W14 row and
every A01 to A25 row has a trace. No game code and no recorder code changed.

- G1: the first `cargo build` failed while another worker was mid-edit
  (`prop_card_board_finish` was called before that method was visible).
  After 10 minutes the retry succeeded. `cargo test`: dino-game 151 passed,
  0 failed, 1 ignored; dino-extract 7 passed.
- G2: skipped. This task does not change game code.
- G3 recording: 256 complete scenarios, 75,129 game frames, 0 lag vblanks.
  Header lengths match the manifest. Script key lines match the recorder
  input. Nine headless MAME processes, one per batch, all exited normally
  (about 5 minutes of wall time).
- G3 port: all 256 scenarios completed `port.sh` in step mode, with no panic.
  None is equal to MAME. The summary is `re/out/probe/_input/m516_port_sum.txt`.
- G4: game run and screenshots skipped, as this task changes no game code.
- G5: scenario commands, inputs and this section only. No ported routine and
  no ROM table.

First difference: frame -1 in 12, frame 0 in 10, frame 1 in 208, frame 7 in
1, frame 8 in 25.

Frame 1 is kept god mode. MAME writes the invulnerability timer full every
frame. The port counts it down. `m516_e7a0_lv10` and `m516_e7a0_lv20` differ
in that timer only. The other frame-1 runs then diverge because a pin is
not replayed (the thug walks), because a later `objset` or `hp` or spawn is
MAME-only, or because an enemy class is not ported. On `m516_pistol_jack`
the pickup sets the weapon-dirty HUD flag in MAME and not in the port, the
pinned thug's position differs from frame 2, and the last-round HP poke
(trace frame 116) is MAME-only.

Frame 8 is the 25 W13 rows with no god mode. The friend jab or the discard
press is the first event. The first fields are the weapon-dirty flag and
the text layer. Frame 7 is `die_with_weapon`: the text layer only, one
frame before the attack key.

Frame 0: the four torch rows, the torch wear row and the torch run-swing
differ in the torch item's height and timer before any key. Food 18's
animation frame is empty in the port (the design says that object has no
picture). Class 65 and class 70 are unported, so the barrel and the biker
move in MAME and stay frozen in the port. Episode 5 area 0 start differs
in the RNG and in floor items on the first frame.

Frame -1: twelve jumps into later episodes. The snapshot loader rebuilds
the spawn-list pointers and a wave pointer differently. The same loader
question is open on the M6-00 recordings. The scenes still run.

Open question: W3's rifle, M-16, Uzi and shotgun distances, and W10 to W12's
grenade, stones and ammo, were measured for Jack. The other three heroes
were not recorded for those rows. Say if those copies should be added.

### M7-00 completion report (2026-10-13)

Prepared five batches, 152 input scripts and one manifest. All 39 requested
acceptance IDs have MAME references. Fourteen scenarios are replay
companions. No game code, recorder code or allow rules changed.

- G1: `cargo build` passed. `cargo test`: 158 passed, 0 failed, 1 ignored.
- G2: skipped by the lane B instruction. No regression claim or gate-number
  comparison with an earlier run.
- G3 recording: 152 complete MAME scenarios, 143,545 game frames and 84 lag
  vblanks. Every trace record number and requested length was checked.
  Every state has its snapshot record. All 152 saved input scripts match
  the recorder output, ignoring its comment header and treating the
  explicit empty-key interval as an empty input. Five sequential headless
  MAME processes recorded the final five batches. All completed normally.
- G3 port comparison: all 152 scenarios ran through `port.sh` in step mode,
  without allow rules. All 152 port traces completed at their requested
  lengths, totaling 143,545 game frames. No game panics remained. None
  equals MAME with the current port. First differences: frame -1 in 91
  cases, frame 0 in 51, frame 1 in 10.
- G4: game run and game screenshots skipped by the lane B instruction.
- G5: only probe commands, input scripts and documentation were added.
  No routines or ROM tables were ported. Runtime states and traces remain
  under ignored `re/out/` paths.

The frame-minus-one differences are spawn or active-wave pointers that the
snapshot loader reconstructs differently. The ten frame-one differences
are kept god controls refreshing player invulnerability in MAME only.
Of the frame-zero cases, 32 first differ in the depth and animation of
floor items in the Episode 5 scenes; five first differ in RNG; fourteen
first differ in scene flags or enemy state. The class breakdowns identify
unported actors, dinosaurs, bosses, poachers, props and effects. Those
classes and timed MAME-only commands explain much of the later object,
census, video and sound differences. They do not excuse a ported routine
bug. Exact causes of the shared floor-item, RNG and ported-field differences
remain implementation work for their owners.

Open questions: snapshot spawn/wave pointer reconstruction; floor-item
motion and animation at the start of the Episode 5 test scenes; AI RNG call
order when those scenes contain unported classes; and the complete natural
bridge crossing missing from the original A219 recipe. The new multiplayer
and level probes prepare the placed-item gates, but do not establish port
parity. Preparation is complete.

Early port attempts read trace files before recording had finished and
reported incomplete-file reader panics. They were stopped and rerun after
each segment finished. The final results above contain only full traces.
MAME emitted the existing placeholder PLD/QSound checksum warnings.

Detailed evidence is under `re/out/probe/_input/`: `m700_audit.log`,
`m700_build.log`, `m700_test.log`, `m700_summary.log`, `m700_results.tsv`,
`m700_first_differences.tsv`, the five `m700_*_record.log` files and each
scenario's `.port.log` and `.diff.log`. Batch logs are under
`re/out/probe/m700_*-rec/`; states and traces are under `re/out/traces/`.

## Episode 1 full runs (M5-17)

Five power-on recordings of Episode 1, from the first frame of control through
the clear tally. No god mode, no pins, and no writes after the snapshot. The
port can replay the keys. `scripts/m517_scenarios.tsv` lists the lengths.
`pokes/m517_boot.txt` only documents the boot timings. The full runs use
`record.sh` and the existing `boot_*.txt` files.

| Scenario | Boot | Frames | Boss frame | Vice HP | Level | Heroes |
| --- | --- | --- | --- | --- | --- | --- |
| `m517_ep1_jack` | `boot_jack.txt` | 29857 | 11905 | 500 | 16 | Jack |
| `m517_ep1_3p` | `boot_3p.txt` | 24336 | 9456 | 750 | 21 | Jack, Mustapha, Mess |
| `m517_ep1_mustapha` | `boot_mustapha.txt` | 30544 | 23537 | 550 | 24 | Mustapha |
| `m517_ep1_hannah` | `boot_hannah.txt` | 15456 | 8547 | 500 | 16 | Hannah |
| `m517_ep1_mess` | `boot_mess.txt` | 16145 | 10480 | 550 | 20 | Mess |

Vice is pool A class 18, handler 0x42590. The HP above is the value at the
first frame he is alive. It matches the D3c section 1.2 row for that player
count and level band (1 player 16 to 19 is 500, 20 to 23 is 550, 24 to 27 is
550, 3 players 20 to 23 is 750). These runs are not locked at level 12, so
they are not the BS-A1 sample of 500 / 600 / 700.

Each trace stops on the last frame before the next episode. The tally frame
is inside the trace (Jack 29792, 3p 24240, Mustapha 30500, Hannah 15390,
Mess 16080). The 40000 frame source is `re/out/traces/<name>.full.mame.trc`.

Jack's input is the quiet-window script from the first pass: direction keys
are removed while he is absent or the mode is the continue screen, and the
trace shows only hero 0. The 3 player script walks and attacks with all
three players. Its trace keeps slot 0 as Jack, slot 1 as Mustapha and slot 2
as Mess, including at the boss. Mustapha, Hannah and Mess hold Right on
every frame. A continue strip starts on the previous hero and moves only on
a left or right edge (D4d 6.5), so attack confirms the same hero. Their
traces contain only that hero.

Record one headless MAME process per run:

```sh
DINO_BOOT=re/harness/scripts/boot_jack.txt \
  re/harness/record.sh re/harness/scripts/m517_ep1_jack.txt m517_ep1_jack 40000
```

Use `boot_mustapha.txt`, `boot_hannah.txt`, `boot_mess.txt` or `boot_3p.txt`
for the other four. Replay in step mode:

```sh
tail -n +2 re/harness/scripts/m517_scenarios.tsv |
while IFS="$(printf '\t')" read -r name boot frames rest; do
  MODE=step re/harness/port.sh "re/harness/scripts/${name}.txt" "$name" "$frames"
done
```

### M5-17 completion report (2026-10-13)

Five scenarios. Each reaches Vice and the tally with the hero named above.
No game code and no recorder code changed. No allow rules were added.

- G1: `cargo build` passed. `cargo test`: dino-extract 7 passed; dino-game
  164 passed, 0 failed, 2 ignored.
- G2: skipped. This task does not change game code.
- G3 recording: 5 complete traces, 116338 game frames, 2399 lag vblanks.
  Jack and the 3 player run were already complete. Mustapha's earlier file
  was a partial write and was recorded again. Hannah and Mess were recorded
  again because the first traces changed hero on continue. Three sequential
  headless MAME processes recorded the new runs. Each logged `recorded 40000
  frames` and exited normally. The saved traces are the prefix through the
  tally.
- G3 port: all 5 scenarios finished `port.sh` in step mode, with no panic.
  None equals MAME. Ported-class pairs: Jack 420, 3 players 551, Mustapha
  265, Hannah 333, Mess 290. Unported-class pairs: 499, 534, 446, 406, 414.
  Globals: 120, 117, 115, 110, 114. First difference frame: Jack 23, 3
  players 40, Mustapha 8, Hannah 20, Mess 12.
- G4: game run and screenshots skipped. This task changes no game code.
- G5: scripts, the boot note, the manifest and this section only. No ported
  routine and no ROM table.

The first difference is the opening, before any fight. Jack, Hannah and Mess
differ at scroll 1 cell 0xa24 (VRAM 90A890): MAME 0020018B, port 04C0018B.
Mustapha (frame 8) and the 3 player run (frame 40) differ in special slot 15
at E574: `ay` 4E19 against 4E1A, and draw flags 29 against 18. The same
special pair shows up a few frames later on the other runs. `global.rng`
then differs around frame 30. In step mode the port is loaded from MAME
before the frame, so that gap is the rand calls of that frame. Unported
objects still call `rand` in MAME.

Vice himself is class 0x12, handler 0x42590, and the diff marks that class
NOT PORTED. His object is the long difference once the fight starts (Jack
from frame 11823, 3 players from 9440, Mustapha from 23488, Hannah from
8496, Mess from 10416). Player fields differ later, after the first combat
(Jack frame 321, Hannah frame 1349, Mustapha frame 3298, Mess frame 2405,
3 player slots from frame 321). Those player gaps sit on top of the opening
mismatch and the unported enemies. They are listed, not treated as a pass.

Open questions: the shared scroll 1 cell at the start of control, the
special slot 15 `ay` and draw flags in the opening, and the step-mode RNG
gap while unported objects still call `rand`. A match across the boss waits
until class 18 is ported. The level is whatever the natural rank reached.
M5-22 owns the comparison and the fixes.

Logs: `re/out/probe/_input/m517_final_summary.log`, `m517_*_rerecord.log`,
`m517_*.port.log`, `m517_*.diff.log`. States and traces stay under
`re/out/traces/`.

## Episodes 6 to 8 preparation scenarios (M8-00)

The four `pokes/m800_*.txt` files are probe scenario batches for M8-00. Each
segment and input script uses the `m800_` prefix. They record alongside other
workers without sharing scenario names or MAME output directories.
`scripts/m800_scenarios.tsv` catalogues the case, scenario name, batch, frame
count, recipe, and timed mutation frames.

| Batch | Cases | Scenarios |
| --- | --- | --- |
| `m800_bosses.txt` | D3d BT-A6 to BT-A17: Slices, K-8, Tyrog forms 1 to 4 and host, Fessenden forms 1 and 2, transformations, grab reactions, defeat | 36 |
| `m800_ep6.txt` | D3e part 3 A01 to A13, A32; D3b A20: walls, doors, lamps, storm, lightning, dinosaur foot, barrels, swamp, water, Tyrog intro, Ep6 area 0 scene script | 24 |
| `m800_ep7.txt` | D3e part 3 A14 to A18: lift full and natural, lift wave 4, fences, Ep7 area 2 end script, Ep7 area 3 dialogue | 10 |
| `m800_ep8.txt` | D3e part 3 A19 to A31: lab vents, flames and level scaling, flame enemy hits, pipes, sewer doors, sewer waves, end scripts, cave meat barrels, acid room, fire glow, ending defeat script | 38 |

The batches preserve the probe controls and timings of the design recipes.
Snapshots use `snapshot keep` when god mode, rank holds, pins, or clock freezes
continue past the snapshot boundary. Directives after the snapshot (`poke`,
`spawn`, `hp`, `killall`, `objset`) change MAME only; the port does not replay
them without sidecars. Use step mode to inspect routines up to those external
mutations.

In `m800_a14_n_e7a1_full`, the lift in Episode 7 Area 1 ends around frame 4950
to begin transitioning to Area 2. Directives with `killall` calls after the
snapshot write to 0xFF850E during area transition, which held MAME tasks in
state 4 (lag) permanently. Truncating the clean 4954 recorded frames in
`m800_a14_n_e7a1_full.mame.trc` and header resolved the issue without re-triggering
the lag loop. `m800_a14_e7a1_nat` (11,000 frames) recorded completely in 15 seconds
without `killall` interference.

Record each batch with one headless MAME process, sequentially:

```sh
for batch in bosses ep6 ep7 ep8; do
  PR_TIMEOUT=3600 PR_SNAPS=none PR_SHEET=0 \
    re/harness/probe/probe_record.sh re/harness/pokes/m800_${batch}.txt all
done
```

Replay each scenario through the port in step mode:

```sh
tail -n +2 re/harness/scripts/m800_scenarios.tsv |
while IFS="$(printf '\t')" read -r case_id name batch frames recipe mutations; do
  MODE=step re/harness/port.sh re/harness/scripts/${name}.txt "$name" "$frames"
done
```

Compare a chosen scenario without allow rules:

```sh
name=m800_bta6_s7_1p_a
re/.venv/bin/python -I re/harness/diff.py \
  re/out/traces/${name}.mame.trc re/out/traces/${name}.step.trc
```

States, MAME traces, port traces, generated inputs, probe logs, and audit logs
remain under ignored `re/out/` paths. The input scripts and manifest contain
commands and metadata only.

### M8-00 completion report (2026-10-13)

Prepared four batches, 108 input scripts, and one manifest. All 44 requested
acceptance IDs have MAME references. No game code, recorder code, or allow rules
changed.

- G1: `cargo build` passed. `cargo test`: 164 passed, 0 failed, 2 ignored.
- G2: skipped by the lane B instruction. No game code changes.
- G3 recording: 108 complete MAME scenarios across 44 design cases, 198,787
  game frames, and 377 lag vblanks. All trace record numbers, lengths, states,
  and input scripts audited and checked. Four sequential headless MAME
  processes recorded the four batches using SDL dummy drivers. All exited
  normally.
- G3 port comparison: all 108 scenarios ran through `port.sh` in step mode,
  without allow rules. 96 port traces completed their full requested lengths.
  12 scenarios stopped early due to the known stage bounds panic at
  `crates/dino-assets/src/stage.rs:87:43` (`index out of bounds: the len is 1
  but the index is 2` or 3) for Stage 8 Area 2 and Area 3 (Fessenden BT-A13 to
  BT-A17, cave meat after frame 576 in A28, acid room A29 b53_a3, and ending
  EE-F84 kill in A31). Total compared port frames: 147,512 game frames. Diff
  exits: 97 with differences found, 11 exit 0 (stopped at frame 0 due to the
  stage panic, so 0 frames compared).
- G4: game run and game screenshots skipped by the lane B instruction.
- G5: only probe batches, input scripts, metadata, and documentation were added.
  No routines or ROM tables were ported. Runtime states and traces remain under
  ignored `re/out/` paths.

The first differences of the 97 comparisons with differences are at frame 0
for 33 cases, frame 1 for 13, frame 2 for 9, frame 3 for 4, frame 4 for 1,
frame 5 for 20, frame 8 for 9, frame 15 for 4, frame 17 for 1, frame 20 for 2,
and frame 37 for 1.

Frame 1 is kept god mode refreshing player invulnerability timer
(`player[0] B274 +$F9 invuln`) in MAME only. Frame 5 and frame 8 differences
are prop mode changes (`prop[19] C534 +$04 mode`) and unported flame/acid
mechanics. Frame 0 differences stem from palette bank 2 / palette 16 mismatches
in scenery checks, initial scroll 1 offsets (cell 0xa64), unported scene/effect
objects (effect[37] ACF4), and AI RNG order differences. Frame 15, 17, and 20
differences arise from scroll cell and camera synchronization as players
advance into fence or dialogue triggers.

Open questions: stage bounds panics in unported Stage 8 Areas 2 and 3; AI RNG
call sequence divergence when scenes contain unported enemy or prop classes;
and handling `killall` directives during stage or area transitions without
stalling in lag vblanks.

Detailed evidence is under `re/out/probe/_input/`: `m800_audit.log`,
`m800_build.log`, `m800_test.log`, `m800_summary.log`, `m800_results.tsv`,
`m800_first_differences.tsv`, and each scenario's `.port.log` and `.diff.log`.
Batch logs are under `re/out/probe/m800_*-rec/`; states and traces are under
`re/out/traces/`.


## Port-side pokes and probe sidecars (M2-00 port side, 2026-10-13)

- `dino-game --poke F:ADDR:SIZE:VAL` (repeatable), `--poke-file FILE` (the `frame addr size value` format of
  `DINO_POKES`, plus `#!clock game|power` and `#!unknown warn|error`), `--poke-clock power|game`. Addresses are full
  hex or `G.$4D0` style; every field the trace models can be poked; an unknown address is an error.
- `probe.lua` writes `NAME.pokes.txt` next to every probe scenario: one line per write the probe made after the
  snapshot (god mode, holds, hp, lives, score, time, poke, objset), frame k = before the tick of frame k.
  `port.sh` applies it by itself; `POKES=none` turns it off, `POKES=FILE` uses another file, `PORT_OPTS` adds options.
  Since task 18, a sidecar with `spawn` notes goes through `spawn_pokes.py` first (the spawns become RAM writes in `re/out/NAME.spawnpokes.txt`).
  Spawns after a snapshot are not replayable (marked as a comment).
- The snapshot loader reads stage and area before the objects, so later episodes load correctly (frame -1
  differences of the m515 / m516 / m600 / m700 scenarios went from 123 to 0).

## QSound register traces (M4-01)

`re/harness/probe/sound.sh` records the Z80 writes to the QSound ports for a list of sound requests. One headless MAME process runs the whole list (`SDL_VIDEODRIVER=dummy`, `SDL_AUDIODRIVER=dummy`). The script is separate from `probe.lua` and `record.lua`.

```
re/harness/probe/sound.sh 0x00 0x30
re/harness/probe/sound.sh 0x30:0:0
SQ_INPUT=requests.txt re/harness/probe/sound.sh
SQ_FRAMES=40 re/harness/probe/sound.sh 0x30
```

A file line is `sound CODE [d1] [d2]`. An argument is `CODE` or `CODE:d1:d2`. Numbers are decimal or `0x` hex. `d1` and `d2` default to 0.

Each request is one ring entry, same layout as enqueue `0xA63C`: 16 bytes at `0xFF803A` plus the write index at `0xFF8034` (code word, d1 long, d2 long). The game flush `0x610` delivers it on a later vblank. While that code is recorded the script sets bit 5 of `G.$777C` (demo sound) so the attract filter keeps it. After the window it clears that bit and queues `0xFF00` (stop everything, helper `0xA5D2`), then waits until the QSound ports have been quiet for 8 video frames before the next request. A state load is not used: with `-sound none` it does not restore the Z80, so the previous sound would keep playing. After the entry is flushed, a write tap on `0xFF8034` forces later write-index stores back to the read index, so an attract enqueue during the window is not committed.

`-sound none` still runs the Z80. The script checks that the register port was written during boot. If it was not, the same batch runs once more with `-sound auto`.

Traces are git-ignored, under `re/out/sound/`:

- `CODE.qs.trc`, or `CODE_D1_D2.qs.trc` when d1 or d2 is not 0. `CODE` is four hex digits.
- Header: `# sound code=0030 d1=00000000 d2=00000000 frames=240`
- Register commit (Z80 port `0xD002`): `f <frame> r <hh> v <hhhh>`
- Bank write (Z80 port `0xD003`): `f <frame> b <hh>`

Frame 0 is the first video frame after the ring write. The value is `(0xD000 << 8) | 0xD001`, which is what helper `0x0A13` commits. `SQ_FRAMES` is the window in video frames (default 240, about 4 seconds). `SQ_TIMEOUT` is the process limit in seconds (default 600). `SQ_SOUND` forces the MAME `-sound` value. A looping tune keeps writing after the default window. The same code recorded alone and recorded after another code can sit on a different sequencer step, so those two files need not match. Two runs of the same list do match.

The shell prints one line per code, `sound 0030: N lines, flushed 1`. The status file is `re/out/sound/qs-status.txt`.

## QSound register compare (M4-09)

`re/harness/sound_cmp.py` compares a MAME `.qs.trc` with the port driver's register write trace for the same code. Both sides use the text from the section "QSound register traces (M4-01)":

```
# sound code=0030 d1=00000000 d2=00000000 frames=240
f <frame> r <hh> v <hhhh>
f <frame> b <hh>
```

`r` is a commit on Z80 port `0xD002` (register, value `(0xD000 << 8) | 0xD001`, helper `0x0A13`). `b` is a write to port `0xD003` (ROM bank, the driver stores `0x80 | bank`). Frame 0 is the first video frame after the ring write. Blank lines and `#` comments other than that header are ignored.

The port example `dump_regs CODE [d1] [d2]` will live in `crates/dino-sound` and print this same text to stdout. That crate is not in the tree yet (the driver starts at M4-02). Until it exists, both inputs are `.qs.trc` files. Redirect the example when it lands:

```
dump_regs 0x30 > re/out/sound/port/0030.qs.trc
```

From the repo root:

```
re/.venv/bin/python -I re/harness/sound_cmp.py MAME.qs.trc PORT.qs.trc
re/.venv/bin/python -I re/harness/sound_cmp.py --align-first MAME.qs.trc PORT.qs.trc
re/.venv/bin/python -I re/harness/sound_cmp.py --mame re/out/sound --port re/out/sound-port 0x00 0x30
re/.venv/bin/python -I re/harness/sound_cmp.py --mame DIR --port DIR --list codes.txt
```

A list line is `sound CODE [d1] [d2]`, `CODE`, or `CODE:d1:d2`. Numbers are decimal or `0x` hex, the same rule as `sound.sh`. The file name is `CODE.qs.trc` (four hex digits), or `CODE_D1_D2.qs.trc` when d1 or d2 is not 0.

The report gives the difference count and the first write that differs: 1-based index, video frame, register (or bank), both values, and a few writes on either side (`--context N`, default 2). The count is every write that does not match, in order, plus each write that only one side has. Time is that video frame. The millisecond figure uses 59.6374 Hz (MAME, sound spec section 2). A later `result` line is one record per pair for scripts. Exit 0 when every pair matches. Exit 1 when a write differs. Exit 2 when a file is missing or a line cannot be parsed.

`--align-first` reads the frame of the first register commit (`r`, not a bank write) on each side and subtracts the difference from every frame of the port file. M4-01 records one code on its own and the same code after another code on different sequencer steps, so the writes can sit about one video frame apart. The option removes that constant offset. A different register or value still counts. The shift is printed. If either file has no register write, the frames stay as they are. Two runs of the same `sound.sh` list already match without this option.


## Whole-episode recordings for Jack, Episodes 2 to 8 (M10-01)

M10-01 provides scripted full runs of Episodes 2 through 8 for Jack (Episode 1 exists from M5-17).
Each recording starts at power on with the documented story-panel poke (`DINO_POKES=re/harness/pokes/epN_boot.txt`)
and syncs at the first controllable frame (`DINO_SYNC=control`).

The scripts follow natural play rules only:
- No cheats, no god mode, no kill-all, and no HP pokes after the start.
- Continues and coins are natural player actions. The scripts insert coins (`1C`) and press start (`1S`) periodically.
- Holding direction `1R` continuously avoids horizontal edge transitions on the continue strip, locking player 1 to Jack upon continue.
- Navigation uses a four-phase Z cycle (Down, neutral, Up, neutral) alternating with walk windows and combat chains (punches, jump kicks, special moves). This allows navigating terrain boundaries, breaking gates (such as gate B#1 in Episode 7 Area 2), and defeating enemy waves without stalling.
- Episode 3 picks up the radio item (pool E class 9) to summon the playable Cadillac (pool B class 88), driving across the desert before fighting Hogg.

### Episode recording summary

| Episode | Script | Boot poke | Recorded frames | Boss defeated | Tally start (`sub=4`) | Tally exit (`sub=6`) | Transition / Area reached |
| --- | --- | --- | ---: | --- | ---: | ---: | --- |
| Episode 2 | `ep2_jack.txt` | `ep2_boot.txt` | 42,000 | Butcher | 41,432 | 41,796 | Transitions to Stage 2 (Episode 3) |
| Episode 3 | `ep3_jack.txt` | `ep3_boot.txt` | 10,500 | Hogg | 9,185 | 9,463 | Transitions to Stage 3 (Episode 4) |
| Episode 4 | `ep4_jack.txt` | `ep4_boot.txt` | 28,000 | Slice | 27,206 | 27,490 | Transitions to Stage 4 (Episode 5) |
| Episode 5 | `ep5_jack.txt` | `ep5_boot.txt` | 65,000 | Morgan / Morgue | 53,916 | 54,200 | Transitions to Stage 5 (Episode 6) |
| Episode 6 | `ep6_jack.txt` | `ep6_boot.txt` | 70,000 | Tyrog | 65,185 | 65,469 | Transitions to Stage 6 (Episode 7) |
| Episode 7 | `ep7_jack.txt` | `ep7_boot.txt` | 60,000 | Slisaurs | 53,059 | 53,333 | Transitions to Stage 7 (Episode 8) |
| Episode 8 | `ep8_jack.txt` | `ep8_boot.txt` | 70,000 | Fessenden (Forms 1 and 2) | 62,646 | 62,842 | Transitions to Ending (Mode 0x1C) |

Each recording has a complete MAME trace (`re/out/traces/epN_jack.mame.trc`) and snapshot (`re/out/traces/epN_jack.state`).
All seven recordings successfully reach the clear tally of their episode (`G.$4D1` steps 4 and 6).

### Three-player whole-episode recordings (M10-02)

M10-02 provides scripted full runs of Episodes 1 through 8 for 3 players (boot_3p: Jack, Mustapha, Mess).
Each recording boots via `boot_3p.txt` (P1 Jack, P2 Mustapha, P3 Mess). Episodes 2 through 8 use the
documented story-panel boot poke (`DINO_POKES=re/harness/pokes/epN_boot.txt`) and sync at the first
controllable frame (`DINO_SYNC=control`), matching M10-01. Episode 1 boots directly from power on.

The scripts (`ep1_3p.txt` through `ep8_3p.txt`) use natural play only:
- Replicated navigation and continuous Right hold (`1R2R3R`) so continues lock to Jack, Mustapha, and Mess.
- Synchronized periodic continues (`1C2C3C`, `1S2S3S`) every 380 frames.
- Multi-directional Z navigation, punch chains, throws, jump kicks, and Cadillac driving in Episode 3.

| Episode | Script | Boot poke | Recorded frames | Boss defeated | Tally start (`sub=4`) | Tally exit (`sub=6`) | Transition / Area reached |
| --- | --- | --- | ---: | --- | ---: | ---: | --- |
| Episode 1 | `ep1_3p.txt` | none | 24,336 | Vice T. | 23,877 | 24,240 | Transitions to Stage 1 (Episode 2) |
| Episode 2 | `ep2_3p.txt` | `ep2_boot.txt` | 42,000 | Butcher | 21,811 | 22,175 | Transitions to Stage 2 (Episode 3) |
| Episode 3 | `ep3_3p.txt` | `ep3_boot.txt` | 10,500 | Hogg | 7,885 | 8,165 | Transitions to Stage 3 (Episode 4) |
| Episode 4 | `ep4_3p.txt` | `ep4_boot.txt` | 28,000 | Slice | 26,944 | 27,243 | Transitions to Stage 4 (Episode 5) |
| Episode 5 | `ep5_3p.txt` | `ep5_boot.txt` | 65,000 | Morgan / Morgue | 60,756 | 61,063 | Transitions to Stage 5 (Episode 6) |
| Episode 6 | `ep6_3p.txt` | `ep6_boot.txt` | 70,000 | Tyrog | 41,981 | 42,478 | Transitions to Stage 6 (Episode 7) |
| Episode 7 | `ep7_3p.txt` | `ep7_boot.txt` | 60,000 | Slisaurs | 38,026 | 38,438 | Transitions to Stage 7 (Episode 8) |
| Episode 8 | `ep8_3p.txt` | `ep8_boot.txt` | 70,000 | Fessenden (Forms 1 and 2) | 47,031 | 47,228 | Transitions to Ending (Mode 0x1C) |

Each recording has a complete MAME trace (`re/out/traces/epN_3p.mame.trc`) and snapshot (`re/out/traces/epN_3p.state`).
All eight recordings reach the clear tally of their episode (`G.$4D1` steps 4 and 6), and Episode 8 transitions to the ending (Mode 0x1C at frame 49,267).

### One-hero recordings for Mustapha, Hannah and Mess (M10-02)

Episode 1 reuses the M5-17 traces. There are no new Episode 1 scripts. Each trace contains only that hero.

| Hero | Trace | Tally start (`sub=4`) | Tally exit (`sub=6`) |
| --- | --- | ---: | ---: |
| Mustapha (1) | `m517_ep1_mustapha.mame.trc` | 30,305 | 30,500 |
| Hannah (2) | `m517_ep1_hannah.mame.trc` | 15,104 | 15,390 |
| Mess (3) | `m517_ep1_mess.mame.trc` | 15,767 | 16,080 |

Episodes 3, 5 and 8 boot with `boot_mustapha.txt`, `boot_hannah.txt` or `boot_mess.txt`, the story-panel poke (`DINO_POKES=re/harness/pokes/epN_boot.txt`), and `DINO_SYNC=control`. Coin and start frames match the Jack scripts. Right is held on every frame so a continue stays on the same hero. Episodes 5 and 8 copy the Jack inputs. Episode 3 changes a short walk so each hero takes the radio (the pickup runs only on an attack press): Mustapha holds Right from frame 1,690 to 2,100 and attacks at 1,762-1,763; Hannah holds Down+Right from 1,690 to 1,800 and attacks at 1,770-1,771; Mess holds Right from 1,900 to 2,100 and attacks at 1,952-1,953.

| Episode | Script | Frames | Hero | Deaths | Tally start (`sub=4`) | Tally exit (`sub=6`) | Then |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| 3 | `ep3_mustapha.txt` | 10,500 | 1 | 0 | 7,116 | 7,396 | Stage 3 story at 7,469 |
| 3 | `ep3_hannah.txt` | 10,500 | 2 | 0 | 7,081 | 7,361 | Stage 3 story at 7,425 |
| 3 | `ep3_mess.txt` | 10,500 | 3 | 0 | 9,419 | 9,699 | Stage 3 story at 9,736 |
| 5 | `ep5_mustapha.txt` | 65,000 | 1 | 8 | 63,845 | 64,202 | Stage 5 story at 64,280 |
| 5 | `ep5_hannah.txt` | 65,000 | 2 | 6 | 57,567 | 57,866 | Stage 5 at 57,901 (mode 0x0C, no mode 0x08 sample) |
| 5 | `ep5_mess.txt` | 65,000 | 3 | 7 | 63,319 | 63,618 | Stage 5 story at 63,685 |
| 8 | `ep8_mustapha.txt` | 70,000 | 1 | 10 | 61,188 | 61,365 | Ending (mode 0x1C) at 63,325 |
| 8 | `ep8_hannah.txt` | 70,000 | 2 | 10 | 64,540 | 64,702 | Ending (mode 0x1C) at 66,837 |
| 8 | `ep8_mess.txt` | 70,000 | 3 | 8 | 56,818 | 56,980 | Ending (mode 0x1C) at 59,084 |

Traces and snapshots are `re/out/traces/ep{3,5,8}_{mustapha,hannah,mess}.mame.trc` and `.state`. During the stage, player 1 is only the named hero. After the Episode 8 ending the game returns to the title and the cursor is Jack. That later hero 0 is not a change during the episode.

All twelve one-hero runs reach the clear tally (`G.$4D1` steps 4 and 6).


## Flow transitions, end chains, links and ending references (M3-28)

The `m328_` recipes prepare M3-29. They contain scenario inputs and RAM writes,
not ROM tables. The runtime `jump` helper reads the user's `dino.zip`.

| File under `scripts/` | Scenarios | Coverage |
| --- | ---: | --- |
| `m328_areas.txt` | 13 | D4c A4, every measured area change in section 5.7 |
| `m328_end_chain.txt` | 8 | D4c A9, `f_ep1` through `f_ep8` |
| `m328_links.txt` | 7 | D4d A19, `link-ep2` through `link-ep8` |
| `m328_ending.txt` | 8 | D4d A20 and A21, four heroes, mixed party, 3P, high score, object audit and fine cuts |
| `m328_power.txt` | 1 | Power-on ending entry using `pokes/m328_power.txt` |

`m328_scenarios.tsv` gives the requested game-frame length of every probe
scenario. The power reference records 7200 video frames. Run each MAME batch
in the foreground, sequentially. Both record tools use SDL dummy drivers.

```sh
DINO_SYNC=power DINO_POKES=re/harness/pokes/m328_power.txt \
  re/harness/record.sh re/harness/scripts/m328_power.txt m328_power 7200
for group in areas end_chain links ending; do
  PR_SHEET=0 re/harness/probe/probe_record.sh \
    re/harness/scripts/m328_$group.txt all || break
done
```

The probe scenarios snapshot at the end of `jump`, before the tested writes.
Keys and timed writes after the snapshot use the game clock. `snapshot keep`
keeps the setup's player protection; every later protection, kill-all and
script-entry write goes into the `.pokes.txt` sidecar. `port.sh` loads that
sidecar automatically. There are no post-snapshot spawns. The area scripts
stop shortly after control returns, before held kill-all reaches later boss
flow. D4c's lift entry is the specified script-16 poke; the other twelve area
changes use the design's natural movement and attack recipes.

Every `watch 1 G` line logs `t`, power frame `f` and game-frame counter `gf`,
plus camera, lock, GO, area, clock, scripted mask, script id and clear flags.
Use the first nonzero scripted mask in the old area as S, then subtract its
`gf` from the area-load and control-release `gf`. This is the section 5.7
clock. The ending batch also watches object states, the ending flag and name
entry state. `end-fine` records every game frame for the fine cuts; no pictures
are requested, as specified for this lane B task. RAM traces stay under ignored
`re/out/`.

```sh
tail -n +2 re/harness/scripts/m328_scenarios.tsv |
while read -r name frames group; do
  MODE=run re/harness/port.sh re/out/traces/$name.input.txt "$name" "$frames" \
    > "re/out/$name.port.log" 2>&1
  re/.venv/bin/python -I re/harness/diff.py \
    "re/out/traces/$name.mame.trc" "re/out/traces/$name.port.trc" \
    > "re/out/$name.diff.log" 2>&1
done
```

A port panic leaves a partial trace. Count its records to report the last
completed frame, and include the panic's source file and line. `port.sh` uses
a pipeline, so its shell status alone does not prove that the game finished.
Check the log and trace length. The power trace is a boot reference and is
separate from these snapshot replays.

### M3-28 completion report (2026-10-13)

G3 recording: 36/36 probe scenarios, 129300 game frames, plus the 7200-video-frame power reference. Every probe trace has continuous records from -1 to N-1, increasing vblank numbers, a snapshot, input file and poke sidecar. All 13 area traces reach the next area and return control. Each group ran sequentially in one headless MAME process with SDL dummy drivers. No screenshots were requested.

G3 replay: every scenario completed its full length in both free and step modes. No panic occurred, so there are no panic source locations to report. Each mode applied 162457 sidecar writes and skipped nine. The skipped writes are all the zero write to G.$514, which the port does not model: `m328_l_e7a0_k16.pokes.txt:147` and each `m328_f_epN.pokes.txt:27`, at game frame 20. The sidecars retain these writes so M3-29 can replay them once the field is modeled. No allow rules were added.

The table gives each scenario's completed frames in each mode and the first difference from MAME. Every probe snapshot matches at frame -1. Full field reports are in the corresponding `.diff.log` and `.step.diff.log` files under `re/out/`.

| Scenario | Free frames | Step frames | First free difference | First step difference |
| --- | ---: | ---: | ---: | ---: |
| `m328_e1a0` | 1000 | 1000 | 18 | 18 |
| `m328_n1a1` | 1800 | 1800 | 48 | 48 |
| `m328_e2a0n` | 1200 | 1200 | 100 | 100 |
| `m328_e2a1n` | 1000 | 1000 | 45 | 45 |
| `m328_e4a0n` | 900 | 900 | 18 | 18 |
| `m328_e4a1n` | 1200 | 1200 | 686 | 686 |
| `m328_e5a0n` | 1600 | 1600 | 0 | 0 |
| `m328_e6a0n` | 1600 | 1600 | 30 | 30 |
| `m328_e6a1n` | 1000 | 1000 | 10 | 10 |
| `m328_l_e7a0_k16` | 600 | 600 | 20 | 20 |
| `m328_e8a0n` | 1200 | 1200 | 179 | 179 |
| `m328_e8a1n` | 900 | 900 | 1 | 1 |
| `m328_e8a2n` | 1400 | 1400 | 0 | 0 |
| `m328_f_ep1` | 6500 | 6500 | 9 | 9 |
| `m328_f_ep2` | 6500 | 6500 | 17 | 17 |
| `m328_f_ep3` | 6500 | 6500 | 11 | 11 |
| `m328_f_ep4` | 6500 | 6500 | 1 | 1 |
| `m328_f_ep5` | 6500 | 6500 | 0 | 0 |
| `m328_f_ep6` | 6500 | 6500 | 0 | 0 |
| `m328_f_ep7` | 6500 | 6500 | 1 | 1 |
| `m328_f_ep8` | 9000 | 9000 | 0 | 0 |
| `m328_link-ep2` | 1800 | 1800 | 0 | 0 |
| `m328_link-ep3` | 1800 | 1800 | 0 | 0 |
| `m328_link-ep4` | 1800 | 1800 | 0 | 0 |
| `m328_link-ep5` | 1800 | 1800 | 0 | 0 |
| `m328_link-ep6` | 1800 | 1800 | 0 | 0 |
| `m328_link-ep7` | 1800 | 1800 | 0 | 0 |
| `m328_link-ep8` | 1800 | 1800 | 0 | 0 |
| `m328_end-jack` | 5700 | 5700 | 0 | 0 |
| `m328_end-hannah` | 5700 | 5700 | 0 | 0 |
| `m328_end-mess-jack` | 5700 | 5700 | 0 | 0 |
| `m328_end-3p` | 5700 | 5700 | 0 | 0 |
| `m328_end-mustapha` | 5700 | 5700 | 0 | 0 |
| `m328_end-hiscore` | 6900 | 6900 | 21 | 21 |
| `m328_end-objs` | 5700 | 5700 | 0 | 0 |
| `m328_end-fine` | 5700 | 5700 | 0 | 0 |

The first object differences belong to classes marked unported by the replay metadata: pool D special 0, enemy 13, 75 and 94, GO prop 20 and effect 21. These objects are held by the harness rather than run. The two early Episode 2 differences are RNG calls from the incomplete object simulation. Link scenes differ at frame 0 because mode 8 still calls the direct stage-start fallback (`game/flow.rs:283`). Mode 0x1C still uses the game-over fallback (`game/flow.rs:290`), so ending replays complete ticks without reproducing the ending. Other first differences are the area fade mask, Episode 6 and 8 scroll2 cells, and HUD scroll1 before the high-score ending entry. Their exact code causes remain for M3-29; no game code was changed here.

`re/out/m328_events.log` records S, lock, GO, area load and control release with t, f and gf for every area. The following measured delays use gf relative to S; the clock column is sampled after area initialization, at control release.

| Change | Area load after S | Control after S | New clock |
| --- | ---: | ---: | --- |
| Ep1 a0 to a1 | 456 | 544 | 02:20 |
| Ep1 a1 to a2 | 258 | 384 | 01:20 |
| Ep2 a0 to a1 | 185 | 604 | 03:20 |
| Ep2 a1 to a2 | 199 | 292 | 01:20 |
| Ep4 a0 to a1 | 123 | 216 | 04:00 |
| Ep4 a1 to a2 | 199 | 292 | 02:50 |
| Ep5 a0 to a1 | 123 | 125 | 04:00 |
| Ep6 a0 to a1 | 268 | 511 | 04:00 |
| Ep6 a1 to a2 | 123 | 125 | 01:20 |
| Ep7 a0 to a1 | 362 | 364 | 02:50 |
| Ep8 a0 to a1 | 514 | 516 | 04:00 |
| Ep8 a1 to a2 | 253 | 255 | 04:00 |
| Ep8 a2 to a3 | 124 | 743 | 04:00 |

These logs sample every video callback. The older design sampled many transitions every two or three callbacks and used video-clock keys and writes. The new post-snapshot keys and writes use the game clock. The biggest area delay difference from D4c 5.7 is Ep8 a1 to a2: +253/+255 instead of +248/+251. The exact contribution of input-clock normalization versus script and snapshot phase needs a follow-up comparison; these are new MAME references, not a claim that every old timing was reproduced.

The measured end-chain clear delays from the entry poke are 367, 376, 376, 376, 376, 376, 285 and 254 gf for Episodes 1 through 8. D4c A9 lists 366, 372, 372, 372, 372, 372, 285 and 255. The snapshot boundary and game-clock poke setup differ from those older video-clock recipes; the exact +4 gf cause in Episodes 2 through 6 remains open. Episode 8 reaches mode 0x1C at t=3875, after the long tally wait.

Ending acceptance values match D4d A20: sub 4 at t=3, sub 6 at 5364, mode 0x14 at 5546. The high-score ending matches A21: G.$507 clears at t=5019 and mode 0x14 starts at 5776. The links and their following openings are recorded through 1800 game frames. Their mode and submode changes are in `m328_events.log`.

The power reference completed a 7200-frame `port.sh` replay with all six writes applied. Its first difference is frame -1: importing power-frame-1 RAM reports uninitialized free-list pointers and initializes port fields. This supplemental snapshot replay is not a boot-equality result. Use an absolute POKES path when running it, because `port.sh` changes directory to `dino-rs`.

The initial long Ep4 a1 trace reached a later boss lock under held kill-all and stopped advancing gf. The replacement 1200-frame recipe captures the transition and finishes. Evidence is retained in `re/out/m328_stall.log` and `m328_areas.first-attempt.log`. The stalled process exited before the replacement MAME batches started. All MAME batches have now exited; no manual stop is needed.

G1, G2 and G4 were skipped under the lane B instructions. The release build inside `port.sh` ran as part of replay; no standalone cargo build, cargo test, regress or game screenshot run was requested. G5 is a script task: no ported routine, ROM table, Rust code or ROM-derived source data was added. No commit or push was made.

Outputs: `re/out/traces/m328_*`, `re/out/probe/m328_GROUP-rec/`, `re/out/m328_GROUP.record.log`, `re/out/m328_NAME.port.log`, `.step.log`, `.diff.log`, `.step.diff.log`, `m328_inventory.tsv`, `m328_replay.tsv` and `m328_events.log`. The two ignored audit helpers are `m328_audit.py` and `m328_report.py`. The open questions are the old/new setup-clock timing offsets, G.$514 replay support, and the remaining flow, fade, tile and HUD differences for M3-29.

## M1-12 and M1-13: the demo recordings

Power-on batch recordings (`record.sh --batch`, one MAME process each, SDL dummy drivers; record k = video frame k + 2 like the plain power-on
traces, a script line f reaches the game at record f - 1):

```sh
DINO_TIMEOUT=1500 re/harness/record.sh --batch re/harness/scripts/m112_demo.txt m112_demo re/harness/scripts/m112_demo_pokes.txt
DINO_TIMEOUT=2400 re/harness/record.sh --batch re/harness/scripts/m113_long.txt m113_long re/harness/scripts/m113_long_pokes.txt
```

| trace (`re/out/traces/`) | what | used by |
| --- | --- | --- |
| `flow_m112-row5` | G.$757A poked to 4 at power frame 1000: the first demo is row 5 (Episode 6 area 1), 4900 records | `demo_rows_5_and_0_set_up_and_run_in_step_with_mame` |
| `flow_m112-row0` | G.$757A poked to 5: the first demo is row 0 (Episode 1 area 1, the sixth demo of the loop), 4900 records | same |
| `flow_m113-coin` | a coin at 4000 to 4007 in the first demo, 4600 records | `coin_during_the_demo_matches_mame` |
| `flow_m113-freestart` | free play (G.$46B = 1 at frame 100), a start at 4000 to 4005 in the first demo, 4600 records | `start_during_the_demo_goes_to_select` |
| `flow_m113-long` | 32000 records from power on, no input: the demos 1 to 5 (Episodes 2, 3, 8, 5, 6) | `demos_set_up_and_run_in_step_with_mame`, `attract_loop_32000_frames_match_mame` |

The port side of the poke is `dino-game --poke 1000:FFF57A:2:5 --trace OUT --frames 4700` (power frames); in the tests `before(k, g)` sets
`g.flow.front.demo_counter` at record 1000. `cargo test -p dino-game demo_report -- --ignored --nocapture` (`DINO_TRACE`, `DINO_POKE_COUNTER`,
`DINO_LAST`, `DINO_DEMO`) lists, per demo, the first record (relative to the call A) where each kind of difference shows; `re/harness/diff.py MAME PORT
--from 3300 --frames N --at FRAME` lists the player object fields at the checkpoints (G.$0 = 60, 300, 600).

## Stage clock and TIME OVER scenarios (M2-13, M2-14)

`pokes/m213_clock.txt` is a probe batch (record it like the M5-15 batches, one MAME process): `m213_clock` (2400 frames: Jack stands
still, the clock is poked to 3:02 before the snapshot and to 2:31, 1:01 and 0:17 after it, so the clock shows at 3:00, 2:30, 1:00 and
0:15, beeps every second in the last 15 seconds, runs out into TIME OVER, Jack dies and respawns with +3:30) and `m213_over_life`,
`m213_over_nolife`, `m213_over_2p_one`, `m213_over_2p_both` (600 frames each: D2b S21 long enough to reach the respawn). The `snaps`
line of `m213_clock` takes ten pictures (3:00, 2:30, 2:29, red 1:00, red 0:15, red 0:12, the words flying in, stopped and held).

```sh
re/harness/probe/probe_record.sh re/harness/pokes/m213_clock.txt all
for n in m213_clock:2400 m213_over_life:600 m213_over_nolife:600 m213_over_2p_one:600 m213_over_2p_both:600; do
  MODE=step re/harness/port.sh re/out/traces/${n%%:*}.input.txt ${n%%:*} ${n##*:}     # MODE=run too (m213_clock)
  re/.venv/bin/python -I re/harness/diff.py re/out/traces/${n%%:*}.mame.trc re/out/traces/${n%%:*}.step.trc
done
# pictures: the port at the same frame (N = probe t - 9), then the contact sheet and the pixel count
dino-rs/target/release/dino-game --rom dino.zip --load-state re/out/traces/m213_clock.state --poke-file re/out/traces/m213_clock.pokes.txt \
  --input-file re/out/traces/m213_clock.input.txt --screenshot re/out/shots/m213/f00109.png --frames 100
re/.venv/bin/python -I re/harness/probe/shotdiff.py re/out/probe/m213_clock-rec/m213_clock re/out/shots/m213 --frames 109,319,779,1129
```

Result (2026-10-13): no difference in the class 31 object, in the pool D slot 15 (the first 0x48 bytes, every frame), in the words
(pool C class 0x70), in the clock word and the wave timer G.$50C at the respawns, or in the 27 sound requests 0x34 (frame and d1 = 0x10,
d2 = 0) in `m213_clock`, in `MODE=run` as in `MODE=step`. What differs belongs to unported classes: the GO hint (pool B 20), the respawn air
strike (pool B 18, from the respawn on), the respawn sparkles and dust (pool C), the continue screen (`m213_over_nolife`), and the lag frames
of `m213_over_2p_both` (53 slowdown frames in MAME). The pictures equal MAME in the clock rectangle (x 160 to 232, y 48 to 80) and in the band of
the words (y 100 to 150) for all ten frames; the rest of the picture differs by up to 2.4 percent in the top line (the join prompts of players
2 and 3, which blink in another phase) and in the flying bird.

## Cutting a snapshot out of a recording, and the clear tally check (M2-24, M2-25)

- `trace_slice.py SOURCE.mame.trc NAME RECORD|ADDR=VALUE [--frames N]`: writes `re/out/traces/NAME.state` (the whole state after one record of a power-on or batch
  recording, as frame -1) and `NAME.mame.trc` (that state, then the records after it as frames 0 ..), without another MAME run. `FF84D1=6` is the first record of the clear
  tally. `check.sh run|step NAME` (with `SCRIPT=` an input script, `FRAMES=`) and `dino-game --load-state NAME.state` then start from that record.
- `tally_check.sh [SCENARIO ...]`: the 31 `flow_tally` recordings (D4d A18), each cut at the first record of the tally (`m224_NAME`), the port run without input
  (`scripts/m224_tally.txt`) and the first difference printed. Every scenario is equal to MAME up to the story panel (mode 8, M2-28: the port's stub starts the next stage
  there, so the first difference is `spawn_list` / `floor_ptr` at that record; `tv-k150` and `tv-fb` end before it). For the four recordings with a busy panel frame
  (`tv-2p-eq`, `tv-3p-a`, `td-3p-231`, `td-3p-tie`) the first difference is the scroll1 cells of the panel draw (below); the script allows that one record and checks on.
  The same equality is the test `clear_tally_matches_mame` (`cargo test -p dino-game clear_tally`).
- Pictures: `dino-game --rom dino.zip --headless --load-state re/out/traces/m224_tv-hp100.state --screenshots 59,99,119,159 --shot-dir DIR` equal the MAME snapshots
  `re/out/flow/flow_tally/tv-hp100/snap/0000..0003.png` (t = 60, 100, 120, 160: the snapshot of time t is the picture of the tick t - 1).

## Story panels, map cards and the level start after them (M2-26 to M2-28)

- `link_check.sh [NAME ...]`: the recordings of `flow_links` (`link-ep2` .. `link-ep8`: the panel of an episode, the card, the walk in, 1100 frames from the poke of the mode 8) and
  `flow_panels` (`pan-ep4-jack` ..: the panel by party, 330 frames; `pan-skip-ep2`: the attack skip, `pan-skip-ep2-jump`: jump does nothing). Each is cut at the record before its poke
  (`link_prep.py`, which also writes the poke file of the port: the stage, G.$4D1 = 0, G.$4D0 = 8 and the live bytes of the heroes), the port runs it without input (`check.sh run`) with the allow
  rules of `story_allow.py` (the file `allow/m228.txt` and the frame ranges that depend on the recording) and the first difference is printed. `STORY_ONLY=1` stops at the first level frames.
  Episodes 6 and 8 are equal in all records; the others are equal to the first scripted actor or enemy that the port does not have (the list is in `specs/flow/HANDOFF.md`).
- `story_allow.py MAME.trc`: writes the allow rules for a (cut) recording that has a mode 8: the derived pointers of the stage that is poked in, the first level frame, the partial draws at the video
  frame boundaries and the frames where the display shadows G.$860 / G.$868 lag the cameras. `tally_check.sh` uses it for the panel after the tally.
- `shot_check.sh KIND NAME T1,T2,...`: the pictures of a cut run (headless, one `dino-game` run) against the MAME snapshots of the segment (`KIND` is `flow_links` or `flow_panels`).
- `diff.py --allow` knows two more rules: `cell LAYER MAME PORT` (a tilemap cell that is MAME in MAME and PORT in the port is no difference) and `rect scroll1 COL0 COL1 ROW0 ROW1` (cells of scroll1
  in these columns and rows, index = column * 32 + row, are not compared).
- `trace_slice.py SOURCE NAME ADDR=VALUE:-N` cuts N records before the first record where the byte has the value (the record before a poke).


## The Episode 3 camera and the row scroll (M3-08 to M3-12)

- `pokes/m308.txt` is a probe batch (one MAME process, nine scenarios, a few minutes): `m308_foot1p` (Jack alone, the empty waves at 0x300 and 0x500, 1000 frames), `m308_foot3p` (three
  heroes, kill all every frame, the locks of 0x300, 0x500 and the biker wave of 0x900, 1500 frames), `m308_car`, `m308_car_end`, `m308_car_time` (the radio at camera X 0x300: the car, the heroes
  board, G.$518 = 1 at about t = 360 and the chase camera; at t = 500 G.$518 is poked to 0 or 2), `m308_row300`, `m308_row1000`, `m308_row2400` (the 32 row scroll words and pictures at the
  three camera positions of D4c A13) and `m308_rowmove` (pictures of the road while the camera moves).
  Record: `PR_SNAPS=none PR_SHEET=0 re/harness/probe/probe_record.sh re/harness/pokes/m308.txt all`.
- `m308_check.sh [NAME[:FRAMES] ...]` runs every scenario in `step` mode (`port.sh`), prints the first difference, the summary, the differences of the camera fields, the row scroll words and the
  tilemaps, and checks the chase formula on the recording of the radio. The car scenarios use `allow/m3.txt` (the causes are in the file and in `specs/stage/HANDOFF.md`).
- `diff.py` compares the row scroll RAM 0x915800 .. 0x918000 as a video item `rowscroll` (the allow key `rowscroll`; the port trace masks it in every stage, MAME has zeros outside Episode 3).
- `watch.py TRACE FIRST LAST ADDR:SIZE[:NAME] ...` prints chosen RAM values of a trace (MAME or port) for a range of frames, only the frames where one changed (`G+744:2:camx` = G.$744 as a word).
- Pictures: `dino-game --headless --load-state re/out/traces/m308_row300.state --poke-file re/out/traces/m308_row300.pokes.txt --input-file re/out/traces/m308_row300.input.txt --screenshots 12 --shot-dir DIR`
  equals the probe snapshot `re/out/probe/m308-rec/m308_row300/snap/0001.png`: the snapshot is at t = 10, so the picture of probe time t is the port's frame t - 10 (t = 22, frame 12; the first
  two frames after a load show the video latency of the loaded state, so take later ones). One headless run, then a pixel compare with the PNG (`PIL`, as `shot_check.sh`).


## The Butcher alone (M6-10)

`re/harness/pokes/m610_butcher.txt` has nine segments of the Butcher's fight (Episode 2 area 2 from camera X 0x1A0, the class 5 object already made at the snapshot): the scenes of
`m600_bs_b1_b1_nat` and `m600_bs_b2_b4_face` and seven more (a standing, walking, fleeing hero; Hannah, Mess, Mustapha, Jack). In every segment the group timer of the Butcher (`+$AA`,
enemy slot 23 = 0xDD14) is held at 0x1E, so the boss controller never gets a group and no Poacher (class 13, not run by the port yet, one random number a frame) disturbs the Butcher's
frames. Record (one MAME process, about 40 seconds) and check:

```sh
PR_SNAPS=none PR_SHEET=0 re/harness/probe/probe_record.sh re/harness/pokes/m610_butcher.txt all
re/harness/m610_check.sh                    # every scenario in step mode: 0 fields expected
```

## Music, hero move and Episode 4 acceptance (M4-16, M5-18, M7-06)

The results and the causes of what is left are in `allow/m4.txt` (section 6: area and boss music), `allow/m5.txt` (55 hero move scenarios, 0 differences with its one rule) and
`allow/m7.txt` (Episode 4: 57 scenarios in step mode, 48 equal, the rest with causes). To repeat a check without overwriting a recording, give the port a new name and link the MAME
files to it (`ln -s NAME.mame.trc a416_NAME.mame.trc`, the same for `.state`, `.pokes.txt`, `.input.txt`), then
`MODE=step re/harness/port.sh re/harness/scripts/NAME.txt a416_NAME FRAMES` and `diff.py re/out/traces/a416_NAME.mame.trc re/out/traces/a416_NAME.step.trc --allow re/harness/allow/m5.txt`
(`MODE=run` and the `.port.trc` for the free run; `DINO_GAME_CLOCK=1` for the area change scenarios of M3-29). The scenario lists: `m515_a1_idle_*` to `m515_a14_wj_*` (M5-18), the Episode 4 rows of
`m700_a207` to `m700_a214` and `m700_a224`, `m600_h13_*`, `m600_h22_e4_open`, `m600_h23_car_trace` and `m516_e4a*` (M7-06). The music requests of a recording are the ring entries with a code
below 0x27, 0xFF00 or 0xFF07 (`trace.mame_sounds`); `DINO_WATCH_WRITE=ADDR probe_record.sh BATCH SEGMENT NEWNAME` logs which routine writes a byte (the wall jump terrain skip was found with the wall
byte `FFB2C4`, the log is in `re/out/probe/BATCH-rec/mame.log`).

### Verify batches of M5-19 to M5-21 (respawn, weapons, items)

Recipes in `re/harness/pokes/` (snapshot style of `probe_record.sh`, one MAME process for the whole file; the input scripts are copied to `re/harness/scripts/NAME.txt`, the table is `re/harness/scripts/m519_scenarios.tsv`):

| recipe | scenarios | what |
| --- | --- | --- |
| `m519_verify.txt` | 54 | D2b S6 (28 enemy kinds under the respawn explosion, pinned), S13 (hero damage at level 31 and low HP), S15 / S16 (lie, mash, dizzy), mercy rule, S10 continue mode, S20 scores |
| `m519b_verify.txt` | 15 | D2b S1 respawn in nine other areas, 2 and 3 players (the drop spot, the strike over other terrain) |
| `m520_verify.txt`, `m520b_verify.txt` | 67, 60 | D2c: 15 weapons against 11 boss and monster kinds (Vice, Walther, Slice, Butcher, Hogg, Morgan, Mack, Rock Hopper, Tyrog, Zeke, Ferris); friendly fire with weapons: dz 1, no enemies, the barrel, three players |
| `m520h_verify.txt` | 66 | D2c weapon scenarios of `m516_w_*.txt` for Hannah, Mustapha and Mess |
| `m521_verify.txt`, `m521b_verify.txt` | 5, 2 | D4a: the Episode 1 hall with the armor statues (W20); item messages in the 2 and 3 player HUD |

Record: `PR_TIMEOUT=3600 re/harness/probe/probe_record.sh re/harness/pokes/m520_verify.txt all` (the recordings go to the shared `re/out/traces/`). Compare: `MODE=step` and `MODE=run` of `port.sh` (with `PORT_OUT=` a folder of your own) and `diff.py`.

`spawn_pokes.py` (called by `port.sh` for a sidecar with `spawn` notes) replays the probe's spawns of all five pools, and the port takes a poked pool pointer word as an allocation (`trace_poke_extra`), so free runs of recordings that spawn explosions, items and enemies stay in step. A probe spawn writes only a few fields of the slot (+$00, +$20, +$08, +$0C, +$10, +$26, +$5C, +$5B, +$96), the rest keeps what the slot had in MAME: the owner `+$02` of a spawned explosion is 0 there (player 1, power `$C6` 42).
Hold a hero still for 92 frames within 31 / 15 pixels of another hero and MAME starts the tag team, which the port does not have yet (M9-01): keep the heroes apart or the recording short.
