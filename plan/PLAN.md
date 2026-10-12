# Plan: a native Rust port that plays exactly like the original

The target is not "similar". For the same inputs, the port must produce **the same game as the arcade original**, frame by frame,
from power on (the attract loop) to the ending. The user judges the result by playing it.

**The build plan is `plan/game/design/D5-final-plan.md`** (written 2026-10-13 after the design phase). It has the milestones, 261 task
blocks (scope, original routines, data, files, design sections, MAME check, size, worker, dependencies), the rules of execution, the
coverage of every row of `plan/game/mechanics-seen.md`, the risk list and the estimate. This file is the short map. The old one file per
phase plan is history (each `plan/phase-*.md` carries a banner that names the milestone that replaced it).

## What "the same" means (acceptance rule for every task)

Run the original in MAME and the port with the same input stream, from the same starting point, and compare once per frame:

1. **Object state:** every live object (players, pools A to E), the fields the port models, by original offset.
2. **Globals:** RNG (`G.$14`), frame counter, rank and level, camera, stage and area, lock and GO flags, scores, lives, credits, timers.
3. **Video output:** sprite list, scroll positions, the three tilemaps, palette memory; for front end screens a picture compare.
4. **Sound requests:** the stream of sound codes with their parameters (and, from M4, the QSound register writes).

A task is done when its comparison shows no difference, or each remaining difference is listed with its cause in the milestone allow file.
The tools: `re/harness/` (record, port, diff, `regress.sh`), the probe tool `re/harness/probe/`, and the tools the plan adds (power-on traces,
timed pokes, probe segments as scenarios, picture compare, sound register compare).

## Rules (full text in D5 section 3)

- Data comes from the ROM through `dino-extract` (exact table copies); code is ported by hand; no emulation; no ROM data in git.
- Keep the original's integer math, order of operations, RNG call order and quirks. Every ported routine names its original address.
- **One code worker at a time** on `dino-rs`; at most two workers at once, the second only for documents, harness scripts and MAME recordings.
  Workers never start workers and never commit. The main session reviews, commits and pushes each green task.
- Every task ends with: build and tests green, `re/harness/regress.sh` not worse, the named MAME comparison, a look at the real game.
- MAME runs are batched and headless (one process for many probes, SDL dummy drivers, never a GUI).
- Placeholders keep the game finishable until a class is ported (M3); every real class task removes its registry entry.

## Milestones

| M | Name | Hours | Status | What the user sees | Replaces |
| --- | --- | --- | --- | --- | --- |
| M0 | Every episode starts | 1.5 | Done (`f8e3c93`) | Episodes 2 to 8 start | new |
| M1 | Attract loop, exact from power on | 24 | Next | boot, title, profiles, ranking, slides, movie, demos | phase 2 (first half) |
| M2 | Coin, select, Episode 1 opening, clock, join, continue, game over, tally | 52 | Pending | the real start of a game and the link scenes | phase 2 (second half), phase 1 tasks 6 and 8 |
| M3 | The whole game skeleton | 51 | Pending | power on to the ending through all 8 episodes, placeholder fights | phases 4, 12 (ending), 13 (part) |
| M4 | Sound | 35 | Pending | music and effects | phase 3 |
| M5 | Episode 1 for real | 37 | Pending | Episode 1 to Vice T. and the tally | phase 1 tasks 4 to 9, phase 5 (part) |
| M6 | Episodes 2 and 3 | 59 | Pending | poachers, dinosaurs, the Butcher, the car, Hogg | phases 6, 7 |
| M7 | Episodes 4 and 5 | 50 | Pending | Slice, Bludge, Zeke, Morgan and Morgue, the village | phases 8, 9 |
| M8 | Episodes 6, 7 and 8 | 66 | Pending | the foot, Tyrog, Slisaurs, tanks, Fessenden, the real ending | phases 10, 11, 12 |
| M9 | Small things pass | 25 | Pending | tag team, audit of every mechanics row, fixes | new |
| M10 | Exactness and finish | 58 | Pending | whole episode runs against MAME, settings, sound, pacing, README | phase 13 |
| | Total | 456 listed, about 600 expected | | | |

Order: M1, M2, M3, M4, M5, M6, M7, M8, M9, M10. Lane B tasks (documents, harness tools, MAME recordings) run beside the code tasks as soon as their needs are met.
The first ten tasks to start are listed in D5 section 10 (M1-00 first).

## Phases of the old plan that are already done

Phase 0 (harness), and in phase 1: terrain collision, items, weapons and pickup, enemy grab, pool A classes 10, 14 (with 15, 34, 41, 60) and 20.
Their code and scenarios stay; `re/harness/regress.sh` guards them.

## Where things are

- Build plan: `plan/game/design/D5-final-plan.md`. Design documents: `plan/game/design/`. The game as a player sees it: `plan/game/README.md`.
- Specs and handoff notes: `re/specs/<area>/SPEC.md`, `HANDOFF.md`. Code rules and file ownership: `dino-rs/ARCHITECTURE.md`. Overall status: `STATUS.md`.
- Errata found while building: `plan/game/design/D5-errata.md` (append only).
