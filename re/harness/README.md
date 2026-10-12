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
throw, first frame of a swing, the player side is task 3) differ in the item classes.

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

### `DINO_STEP_PLAYERS=1` (step mode only)

In `step` mode every object starts a frame from MAME's state after the previous frame. The players
run before pool E in the same frame, and the port's player does not lift, throw or swing weapons yet
(task 3), so a player difference would hide what the items do. With `DINO_STEP_PLAYERS=1` the port
also reads the players of MAME's record of the frame being run (one record ahead) and loads them
after its own players' update, just before pool E runs, so the items see the same player as in the original. Limits: the item
classes then compare exactly, but the player fields themselves are copied, not tested, and
the score the items add to the player shows doubled in the diff (the loaded player already contains it).
Use it only for the item scenarios.

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
