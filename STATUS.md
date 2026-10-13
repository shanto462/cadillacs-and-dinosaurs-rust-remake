# Status and next steps

Goal: a native Rust port of *Cadillacs and Dinosaurs* that plays exactly like the original from power on
to the ending (attract loop, all 8 episodes, every small mechanic, input, sound), loading assets from the
user's own `dino.zip`. Rules and file ownership: `dino-rs/ARCHITECTURE.md`.

## The plan

The build plan is [`plan/game/design/D5-final-plan.md`](plan/game/design/D5-final-plan.md): 261 tasks in
milestone order, each with its routines, files, checks and size (summary in [`plan/PLAN.md`](plan/PLAN.md)).
It was merged from the design study in [`plan/game/design/`](plan/game/design/) and covers every row of
[`plan/game/mechanics-seen.md`](plan/game/mechanics-seen.md) (588 rows). The game as a player sees it:
[`plan/game/README.md`](plan/game/README.md).

Working mode: at most 2 Claude sub-agents at a time, at most one worker writing game code (`dino-rs/`);
external CLIs (codex, grok, agy) take well specified tasks; every task is reviewed, built, tested,
compared with MAME and looked at in the running game before it is committed.

## Milestones

| M | What the player gets | Agent hours (plan) | State |
| --- | --- | --- | --- |
| M0 | Every episode starts | 1.5 | Done (`f8e3c93`) |
| M1 | Attract loop, exact from power on (boot, title, profiles, ranking, story slides, opening movie, demos) | 24 | Done (2026-10-13): two full loops from power on equal MAME outside the demos; the demos match as far as their levels exist |
| M2 | Coin, select, Episode 1 opening, clock and TIME OVER, join, continue, game over, name entry, tally, link scenes | 52 | In progress |
| M3 | The whole game from power on to the ending, every transition and story scene, placeholder fights | 51 | Pending |
| M4 | Sound: Z80 driver port, QSound mixer, cue wiring | 35 | Pending |
| M5 | Episode 1 for real | 37 | Pending (part done in the old phase 1: terrain, items, weapons, grabs, classes 2, 10, 14, 20) |
| M6 | Episodes 2 and 3 (poachers, dinosaurs, Butcher, the Cadillac drive, Hogg) | 59 | Pending |
| M7 | Episodes 4 and 5 (Slice, Bludge, Zeke, Morgan / Morgue, the burning village) | 50 | Pending |
| M8 | Episodes 6, 7 and 8 (the foot, the storm, Tyrog, the lift, Slisaurs, the tanks, Fessenden, the ending) | 66 | Pending |
| M9 | Small things pass over `mechanics-seen.md` | 25 | Pending |
| M10 | Exactness against MAME and finish | 58 | Pending (all reference recordings exist: every episode with Jack, 3 players and the other heroes) |
| | Total | 456 listed (about 610 expected) | |

## Done before the build plan

- Phase 0: the exactness harness (MAME reference traces, port traces, diff, step mode, regression over 52 scenarios).
- Old phase 1 work that the plan keeps: terrain collision, items, weapons and pickup, the enemy grab, classes 2, 10, 14, 20, 49, 50, 54, 55, the rank events on death and extra life, the friendly fire gate.
- The design study (2026-10-12 / 13): research, probe tool, asset dump (`re/out/assets/index.html`), D2a to D4e, the scenery walks of all 8 episodes, the task index and the final plan.
- Multiplayer harness: 2 and 3 player runs match MAME.

## How to run

From `dino-rs/`: `cargo run -p dino-game -- --rom ../dino.zip [--stage N] [--chars 0,2]`.
Keys (player 1): arrows or WASD to move, J or Z attack, K or X jump, Enter or 1 = start, 5 = coin, F1 = hit boxes; players 2 and 3 use gamepads. Viewer: `--viewer`.
