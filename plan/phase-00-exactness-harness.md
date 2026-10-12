# Phase 0: exactness harness

**Goal:** a repeatable way to prove that the port behaves exactly like the original, used by every
later phase.

## Needs

- MAME command-line build with Lua scripting (Homebrew `mame`, 0.289, installed 2026-10-12).
  Decision: the installed MAME4mac app ignores command-line options (checked: `-listroms` and
  `-autoboot_script` do nothing), so it cannot record traces headless. MAME4mac stays the tool for
  playing the original by hand next to the port; the CLI MAME records the reference traces.
- The user's set lacks 6 small PLD dumps and the QSound DSP ROM that MAME 0.289 requires; zero-filled
  placeholders of the right size live in `re/mame/roms/` (git-ignored). MAME warns and runs. MAME's
  audio is not authentic with the placeholder DSP ROM; traces of game state are not affected.
- The user's `dino.zip` (MAME set "dino").

## Tasks

1. **Reference recorder (MAME Lua).** A script `re/harness/record.lua` that runs `dino` headless
   (`-video none -sound none -nothrottle`), feeds inputs from a script file (same format as the
   port's `--input`, extended to all players and coin/start), and writes one record per frame:
   - work RAM ranges for globals and all object pools (0xFF8000..0xFFFFFF and the pool ranges);
   - the sprite list buffer that was displayed (0x900000 / 0x904000, picked by `$69B4`);
   - scroll registers, the three tilemaps (only changed cells), palette RAM (only changed words);
   - sound requests: writes to the QSound mailbox 0xF18000..0xF1801F (or the `$3A` queue).
   Output: a compact binary or JSON-lines trace under `re/out/traces/` (git-ignored).
2. **Port trace.** A `--trace <file>` mode in `dino-game` that writes the same record from the
   port's state. Object fields are written by their original offsets: each object struct gets a
   `trace_fields()` that lists (offset, size, value) for every field it models.
3. **Sync point.** Two ways to start both sides at the same state:
   - **From power-on** (best, needs phase 2 flow): identical inputs from reset.
   - **From a snapshot** (works now): at a chosen frame in MAME (e.g. the first in-game frame of
     stage 1 after coin, start and select), dump all RAM; a `--load-state <file>` in the port fills
     its globals and objects from that dump by the same offsets (`load_fields()`), then both runs
     continue with the same inputs.
4. **Diff tool.** `re/harness/diff.py`: reads both traces, reports the first differing frame, the
   object (handle and kind), the field (offset and name) and both values, and a short summary of
   all differences. Known, approved differences can be listed in an allow file per phase.
5. **First comparison.** Stage 1 with player 1 as Jack: snapshot at the first controllable
   frame, then 600 frames of a fixed input script (walk, jumps, punch chain, a grab). Fix every
   difference in already-ported code until the run matches or the remaining differences are
   explained (they become tasks in phase 1).

## Done when

- `re/harness/README.md` explains how to record, run and diff in three commands.
- The stage 1 comparison above runs end to end and its result is written down here.

## Status

**Done (2026-10-12).** The harness records MAME, runs the port from the same state and compares
every frame. Instructions: `re/harness/README.md`. No difference is approved (`re/harness/allow/phase0.txt`
is empty on purpose): every remaining difference is a class or routine the port does not have yet,
listed as a task in `plan/phase-01-stage1-complete.md`.

### What was built (on top of the first commit)

- **Three run modes** (`MODE=run|follow|step` of `port.sh`). `step` loads MAME's state before every
  frame, so a difference at frame k comes from frame k alone; it made the triage possible, because a
  free run diverges at the first unported object (slot, census, `rand()`). `follow` copies G.$14 from
  MAME each frame. `run` is the plain end-to-end check.
- **Load gaps closed.** Objects of unported classes are loaded and kept frozen (`Game::trace_hold`),
  so they appear in the trace and the pool free lists stay equal. Frame records of animation tables
  the extractor does not know are decoded on demand (`dino_extract::extract_with`, driven by
  `harness::scan_frames`): no load warnings are left. Stale bytes of free slots and the raw bytes of
  enemies, props and items are loaded too. Null and RAM frame pointers are kept as they are.
- **`diff.py`:** differences by object class with the handler address of the original and a
  `[ported]` / `[NOT PORTED]` mark; sprite entries split by the object that drew them (the port tags
  each entry); sound requests where only MAME has more; `--at FRAME`; `--rom-image`.
- **Recorder:** `DINO_SYNC=control+N`; MAME's cfg, nvram and screenshots go to `re/out/mame/` (the
  stray `cfg/` and `nvram/` in the repo root were deleted). Recordings are bit-for-bit repeatable.
- **Scenarios** (all from player 1 = Jack in stage 1; step mode is the triage run):
  `stage1_jack` (the planned 600-frame script, sync at the first controllable frame),
  `stage1_late` (the same script, `DINO_SYNC=control+86`, after the opening scene), `explore`
  (hold right, 600 frames), `stage1_fight` (`control+86`, 1400 frames: walk to the first wave, then
  punch chains, jump kicks and steps against up to four thugs).

### Result

Frame -1 (the loaded state) matches in every scenario. In step mode, the fields of the ported
classes (Jack, pool A class 2 Ferris / Gneiss / Driver, the wave controller, effects 4 / 0x10 / 0x20 /
0x68), the scroll2 and scroll3 tilemaps, the sprite entries of objects whose own fields match, and
every sound request of the port match, except for what is listed here. Scroll1 and the palette
differ only in text and prompts that unported classes draw (below). Fields differing over a whole run:

| scenario | frames | fields differing in ported classes | cause |
| --- | --- | --- | --- |
| stage1_jack | 600 | 2: player `kills` @48, `wall` @313 | class 0x37 death (below), wall test 0x1309C |
| stage1_late | 600 | 1: player `wall` @313 | wall test 0x1309C |
| explore | 600 | enemy class 2 and player, from frame 335 on | wall test: Jack stands 2 px beside MAME's position at the right wall, the enemies react to it (grab at 461) |
| stage1_fight | 1400 | enemy class 2: `z`, `x`, `wall`, `obstacle`, and the attack-the-obstacle state; effect class 1 `live` @789 | wall test and obstacle test (0x12CB4 / 0x12E9A); sparks of a hit container (item class 5) |

Free runs (`run`, `follow`) differ from frame 0 (frozen unported objects move in MAME) and then
diverge for good as soon as an unported object frees a pool slot, kills, or calls `rand()` (stage1_jack:
census @37, `rand()` and `aggression` @48). A class 2 enemy spawned into a slot that an unported
bird held in MAME starts from stale bytes that differ. They become the real test when phase 1
has ported the classes below.

### What differs and why (every item is a task in phase 1)

(a) Unported class or routine (handler addresses are from the original's dispatch tables):

- **Terrain tests** 0x12CB4 (floor + wall loop; the port only has the floor part), 0x12E9A (other
  objects as obstacles), 0x12FE4 / 0x1309C (floor and wall lookup in scroll2 attributes, tables
  0x13B0C and 0x1370C), 0x1361E: `+$50` wall bits, pushes of x and z (street edges, the camera lock
  at x 0x63D, the back line at z 0x120), `+$94` obstacle. The most visible difference of all runs.
- **Pool A:** class 0x37 (55, 0x5785C) and 0x36 (54, 0x576D2): the opening scene objects at the first
  controllable frame (gone by frame ~60); their death calls the shared 0x2AF86 (kills `+$10B`, rank
  0x182A2: `aggression`, `rand()`) and the voice 0x9796 (sounds F8 72). Class 0x31 (49, 0x56A70):
  three off-screen bird controllers. Class 0x32 (50, 0x56C80): birds spawned at run time.
- **Pool B:** class 0x14 (20, 0x7BB68) the idle "GO" hint (scroll1 cells, bank 1 palette 10, sound
  0x3E, timer G.$4FE); class 0x1F (31, 0x81046) draws a sprite list that lives in pool D slot 15 at
  0xE574 and changes it every frame (`special[15]`); class 0x42 (66, 0x86D76) the text banners in
  scroll1 rows 57-58 (frame ~500); class 0x28 (40, 0x836A4); present but static in these runs:
  0x45 (69, 0x88A84), 0x49 (73, 0x89ABA), 0x4A (74, 0x89B48).
- **Pool E:** class 5 containers (0x28C76; hit by Jack and by thugs: hit sparks, the break sounds
  F5 6E; G.$67BE changes at explore frame 109), class 0 weapons (0x24D44), class 9 treasure
  (0x2A028). Effect class 0x0B (0x934F6, debris). A sound 0x34 repeats every 51 frames from frame 967
  of stage1_fight, source not identified yet (probably an item).
- **Join task 0x6C4A:** the INSERT COIN prompts of absent players blink in scroll1 (rows 36-37,
  48 frames on / 80 off); `flow_join_task` is a stub.

(b) Bugs in ported code, fixed now (each found by a step run, each was a real difference):

1. Scroll2 streaming read 0 where the original reads ROM words past the end of a map's screen
   table (rows above the map): `dino-extract/src/stage.rs` keeps 32 more words.
2. Player sound requests recorded the address of the voice routine, not the code: the 13 routines
   0x95CE..0x9C0C, the hit sound table 0x962A and 0xA4CA are ported (`game/snd.rs`: toggle bits G.$43A
   and G.$43C, voice bank +$118 from 0xA502). Pan and volume (d1, d2) are not recorded yet.
3. The combat lists were visited oldest first; the original's lists are stacks that its passes walk
   newest first (`combat.rs`): the grab partner and the order of sparks differed.
4. G.$0000 was incremented at the end of the frame; the original increments it before the mode runs
   (0xA914): all parity tests (`G.$0 & 1`: slot choice, spark jitter) were off by one frame.
5. Class 2 approach (0x33FB8): the test `cmpi.b #2,$5(a6)` is on the enemy's own sub state, the port
   tested the target's (the enemy dropped its target when the player was hurt).
6. Enemy HP bar message for the HUD (0x16B36) was a no-op: ported (`enemy_hud_hit`).
7. A player's hit did not clear the enemy's `+$7A` wait (0x10A2C).
8. The rank event 0x183E4 (a player recovered from a hit: rank - 1, 600 frame cooldown) was missing.
9. The enemy hit library did not request its sounds (0x9796 landing, 0xA354 / 0x97D2 death voice,
   12 sites) nor the screen shake request G.$755C of a landing thrown enemy (0x2AB8C).

(c) Harness and loader problems, fixed: stale slot bytes and the raw enemy / prop / item bytes were
not loaded; null and RAM frame pointers raised warnings; the wave controller's `+$A0` is stale until its
first update; frame records of unknown tables.

### Not compared yet (blind spots)

Pan and volume longs of sound requests, the CPS scroll registers (the camera values behind them are
traced), task states, and every byte listed under NOT MODELED in the report. Scroll1 text is
compared cell by cell.
