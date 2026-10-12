# Phase 0: exactness harness

**Goal:** a repeatable way to prove that the port behaves exactly like the original, used by every
later phase.

## Needs

- MAME command-line build with Lua scripting (Homebrew `mame`, 0.289). The installed MAME4mac app
  is a GUI port and cannot run scripts headless.
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

Next. Waiting for the MAME install.
