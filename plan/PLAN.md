# Plan: a native Rust port that plays exactly like the original

The target is not "similar". For the same inputs, the port must produce **the same game as the
arcade original**, frame by frame. Every phase below ends with proof of that, not only screenshots.

## What "the same" means (acceptance rule for every phase)

Run the original in MAME and the port with the same input stream, from the same starting point,
and compare once per frame:

1. **Object state:** for every live object (players, pools A/B/C/D/E), the fields the port models,
   by original offset: position, velocity, state bytes, animation frame, timers, HP, links.
2. **Globals:** RNG (`G.$14`), frame counter, rank and level, camera, stage/area, lock and GO flags,
   scores, lives, credits, timers.
3. **Video output:** the sprite list (all OBJ entries in order), scroll positions, the three
   tilemaps, palette memory.
4. **Sound requests:** the stream of sound codes (and their parameters) sent to the sound CPU.

A phase is done when its scripted runs show **no difference for the whole run**, or every
remaining difference is listed in the phase file with its cause and is approved as a known
limitation. Phase 0 builds the tools for this.

## Rules (all phases)

- Data comes from the ROM through `dino-extract`; code is ported by hand. No emulation.
- Keep the original's integer math, order of operations, RNG call order and quirks (bugs included).
- Every ported routine names its original address. Specs live in `re/specs/<area>/`.
- **One agent at a time.** It finishes, the build is green, the phase checks pass, the main session
  reviews and commits and pushes, then the next task starts. Agents never spawn their own agents.
- Never commit ROM data or anything derived from it.

## Phases

| # | Phase | Depends on | Status |
| --- | --- | --- | --- |
| 0 | [Exactness harness: MAME reference traces, port traces, diff tool](phase-00-exactness-harness.md) | MAME CLI | Next |
| 1 | [Stage 1 complete: items, weapons, all stage 1 enemies, Vice T., stage clear](phase-01-stage1-complete.md) | 0 | Pending |
| 2 | [Front end and game flow: attract, title, coins, select, join, continue, game over, high scores](phase-02-front-end-and-flow.md) | 0 | Pending |
| 3 | [Sound: Z80 driver port, QSound mixer, audio output](phase-03-sound.md) | 0 | Pending (reverse engineering done) |
| 4 | [Background and camera extras: shake, desert road, elevator, tile and palette animation](phase-04-background-extras.md) | 0 | Pending |
| 5 | [Effects: all pool C classes and pool D](phase-05-effects.md) | 0 | Pending |
| 6 | [Stage 2 (internal 1)](phase-06-stage2.md) | 1, 4, 5 | Pending |
| 7 | [Stage 3 (internal 2): desert road and driving](phase-07-stage3.md) | 4 | Pending |
| 8 | [Stage 4 (internal 3)](phase-08-stage4.md) | 6 | Pending |
| 9 | [Stage 5 (internal 4)](phase-09-stage5.md) | 6 | Pending |
| 10 | [Stage 6 (internal 5)](phase-10-stage6.md) | 6 | Pending |
| 11 | [Stage 7 (internal 6): includes the elevator shaft](phase-11-stage7.md) | 4, 6 | Pending |
| 12 | [Stage 8 (internal 7): final boss and ending](phase-12-stage8-and-ending.md) | 2, 6 | Pending |
| 13 | [Whole-game verification and finish](phase-13-final-verification.md) | all | Pending |

Phases 1-5 are mostly independent once phase 0 exists, but they still run **one at a time** in the
order above. Stage phases 6-12 each port the object classes their stage's spawn scripts use
(decoded in `re/specs/enemy/stage_scripts.txt`); classes shared with earlier stages are already done.

## Where things are

- Specs and handoff notes: `re/specs/<area>/SPEC.md`, `HANDOFF.md`.
- Code rules and file ownership: `dino-rs/ARCHITECTURE.md`. Overall status: `STATUS.md`.
- Each phase file lists its tasks; a task is one agent run and ends with a commit.
