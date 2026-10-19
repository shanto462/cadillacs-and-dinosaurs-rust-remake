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

Working mode (2026-10-19): three Claude Desktop sessions (Opus, Sonnet, Haiku) take tasks through a file mailbox (`re/out/desk/`), two Claude
sub-agents (Sonnet, Haiku) and the external CLIs codex and agy take the others; everybody works in a git worktree (`.worktrees/`) and the main
session merges into `integ`; every task is
reviewed against the disassembly, built, tested, compared with MAME and looked at in the running game before it is
merged. Review aids: `--episode N --area A --camx X`, `--autoplay`, `--debug-keys`, `--list-placeholders`, `--help`.

## Milestones

| M | What the player gets | Agent hours (plan) | State |
| --- | --- | --- | --- |
| M0 | Every episode starts | 1.5 | Done (`f8e3c93`) |
| M1 | Attract loop, exact from power on (boot, title, profiles, ranking, story slides, opening movie, demos) | 24 | Done (2026-10-13): two full loops from power on equal MAME outside the demos; the demos match as far as their levels exist |
| M2 | Coin, select, Episode 1 opening, clock and TIME OVER, join, continue, game over, name entry, tally, link scenes | 52 | In progress (2026-10-19: 32 of 33; the acceptance checks M2-32 are merged) |
| M3 | The whole game from power on to the ending, every transition and story scene, placeholder fights | 51 | In progress (2026-10-19: 29 of 30: `dino-game --headless --autoplay` plays from power on through all eight episodes to the ending and back to the attract loop; left: M3-30 fix list and docs) |
| M4 | Sound: Z80 driver port, QSound mixer, cue wiring | 35 | In progress (2026-10-19: 18 of 20: the driver, mixer and audio output, the exactness passes over all music and effect codes, front end, hero, weapon and item sounds; left: area and boss music M4-16; running: the acceptance M4-19) |
| M5 | Episode 1 for real | 37 | In progress (2026-10-19: 17 of 23: effects, Vice T., Rock Hopper, Walther, steam pipes, the scenario prep; left: the verify passes and full runs M5-18 to M5-23) |
| M6 | Episodes 2 and 3 (poachers, dinosaurs, Butcher, the Cadillac drive, Hogg) | 59 | In progress (2026-10-19: 32 of 33: Episode 2 (poachers, hunter, Lash T., Mack, Shivat, Butcher, scenery) and Episode 3 (Hogg, the Cadillac drive, bomb carriers, hazards, the acceptance M6-31 / M6-32); running: the Episode 2 acceptance M6-17) |
| M7 | Episodes 4 and 5 (Slice, Bludge, Zeke, Morgan / Morgue, the burning village) | 50 | In progress (2026-10-19: 20 of 28: Slice, Zeke, Rock Hopper, Bludge part 1, the shared boss skeleton, the Morgan scene and fight, Episode 4 and 5 scenery and events; running: Bludge parts 2 and 3, Morgan's death and Morgue M7-19 to M7-21; left: the Episode 4 and 5 acceptance) |
| M8 | Episodes 6, 7 and 8 (the foot, the storm, Tyrog, the lift, Slisaurs, the tanks, Fessenden, the ending) | 66 | In progress (2026-10-19: 11 of 37: Episode 6 scenery and lightning, Episode 7 lab events, Episode 8 sewer scenery and flame vents; running: Tyrog M8-06 to M8-12, the tank breakers M8-19; left: the foot, Slisaurs, Fessenden, the acceptance runs) |
| M9 | Small things pass over `mechanics-seen.md` | 25 | In progress (2026-10-19: 3 of 14) |
| M10 | Exactness against MAME and finish | 58 | In progress (2026-10-19: 2 of 26; the recording sweep: 266 of 418 recordings equal MAME in step mode) |
| | Total | 456 listed (about 610 expected) | |

## Done before the build plan

- Phase 0: the exactness harness (MAME reference traces, port traces, diff, step mode, regression over 52 scenarios).
- Old phase 1 work that the plan keeps: terrain collision, items, weapons and pickup, the enemy grab, classes 2, 10, 14, 20, 49, 50, 54, 55, the rank events on death and extra life, the friendly fire gate.
- The design study (2026-10-12 / 13): research, probe tool, asset dump (`re/out/assets/index.html`), D2a to D4e, the scenery walks of all 8 episodes, the task index and the final plan.
- Multiplayer harness: 2 and 3 player runs match MAME.

## How to run

From `dino-rs/`: `cargo run -p dino-game -- --rom ../dino.zip [--stage N] [--chars 0,2]`.
Keys (player 1): arrows or WASD to move, J or Z attack, K or X jump, Enter or 1 = start, 5 = coin, F1 = hit boxes; players 2 and 3 use gamepads. Viewer: `--viewer`.
