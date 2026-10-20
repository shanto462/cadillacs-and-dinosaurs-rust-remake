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

Work resumed 2026-10-21 after the M4 checkpoint (M1 to M4 and M6 done). Open leads: `re/out/cli/pending_fixes.md` (git-ignored). Play and test commands: `run.md`. Recording sweep (2026-10-21): 379 of 418 recordings equal MAME in step mode.

| M | What the player gets | Agent hours (plan) | State |
| --- | --- | --- | --- |
| M0 | Every episode starts | 1.5 | Done (`f8e3c93`) |
| M1 | Attract loop, exact from power on (boot, title, profiles, ranking, story slides, opening movie, demos) | 24 | Done (2026-10-13: two full loops from power on equal MAME outside the demos; the picture compare tool M1-14 in 2136d09) |
| M2 | Coin, select, Episode 1 opening, clock and TIME OVER, join, continue, game over, name entry, tally, link scenes | 52 | Done (2026-10-19: 33 of 33; the timed pokes M2-00 in 2136d09, the acceptance checks M2-32) |
| M3 | The whole game from power on to the ending, every transition and story scene, placeholder fights | 51 | Done (2026-10-19: 30 of 30: the bot plays from power on through all eight episodes to the ending and back to the attract loop; M3-30: the car allow rules gone, Episode 1 cleared without kill all, the placeholder census in D5-errata, the link scenes looked at) |
| M4 | Sound: Z80 driver port, QSound mixer, cue wiring | 35 | Done (2026-10-19: 20 of 20: the driver, mixer, audio output, every music and effect code, the front end, hero, weapon and item sounds, the acceptance M4-19 (request streams of 15 episodes, QSound registers equal), area and boss music M4-16 (217 requests equal the 0xA582 / 0xA5C8 rule)) |
| M5 | Episode 1 for real | 37 | Done (2026-10-21: 23 of 23: effects, Vice T., Rock Hopper, Walther, steam pipes, the heroes' moves M5-18, the verify passes M5-19 to M5-21 (581 scenarios), the Episode 1 full runs M5-22 / M5-23 (the four heroes and 3 players equal MAME in lagsync mode for every record; Vice's HP by player count)) |
| M6 | Episodes 2 and 3 (poachers, dinosaurs, Butcher, the Cadillac drive, Hogg) | 59 | Done (2026-10-19: 33 of 33: Episodes 2 and 3, the Episode 2 and 3 acceptance) |
| M7 | Episodes 4 and 5 (Slice, Bludge, Zeke, Morgan / Morgue, the burning village) | 50 | Done (2026-10-21: 28 of 28: Slice, Zeke, Rock Hopper, Bludge, the boss skeleton, the Morgan scene and fight, Morgan's death and Morgue M7-19 to M7-21, Episode 4 and 5 scenery, the Episode 4 acceptance M7-06, the Episode 5 acceptance M7-26 / M7-27 (68 of 69 Episode 5 recordings equal; the whole-episode step runs first differ at a dialogue CPU overrun or two effect byte items, see D5-errata)) |
| M8 | Episodes 6, 7 and 8 (the foot, the storm, Tyrog, the lift, Slisaurs, the tanks, Fessenden, the ending) | 66 | Done (2026-10-21: 37 of 37: the brontosaurus foot, the Slisaurs and the lab monitor, Tyrog, Fessenden, the lab tank breakers, Episode 6 to 8 scenery, the Episode 7 acceptance M8-18 and the Episode 8 acceptance M8-36 (the clear scene line and aura, the Rock Hopper's hurt list, Fessenden's counter row); no placeholder class is left; the smart bot clears all eight episodes with each hero) |
| M9 | Small things pass over `mechanics-seen.md` | 25 | In progress (2026-10-21: 7 of 14: the tag team move M9-01 / M9-02, M9-03, M9-04 (the clear scene flare, the dead pool B classes), M9-05 (the effect leftovers are not reached in play), M9-06, M9-07; the audits M9-08 to M9-10 as first versions (most rows implemented but not yet checked row by row); left: the audit fixes M9-11 to M9-14) |
| M10 | Exactness against MAME and finish | 58 | In progress (2026-10-21: 7 of 26: the whole-episode recordings M10-01 / M10-02, the six attract demos M10-03 to M10-05 (equal from power on), the Episode 2 and 3 whole runs M10-08 / M10-09 (seven recordings equal in step and lagsync mode; listed causes in allow/m10_runs.txt); the recording sweep: 401 of 418 equal in step mode) |
| | Total | 456 listed (about 610 expected) | |

## Done before the build plan

- Phase 0: the exactness harness (MAME reference traces, port traces, diff, step mode, regression over 52 scenarios).
- Old phase 1 work that the plan keeps: terrain collision, items, weapons and pickup, the enemy grab, classes 2, 10, 14, 20, 49, 50, 54, 55, the rank events on death and extra life, the friendly fire gate.
- The design study (2026-10-12 / 13): research, probe tool, asset dump (`re/out/assets/index.html`), D2a to D4e, the scenery walks of all 8 episodes, the task index and the final plan.
- Multiplayer harness: 2 and 3 player runs match MAME.

## How to run

From `dino-rs/`: `cargo run -p dino-game -- --rom ../dino.zip [--stage N] [--chars 0,2]`.
Keys (player 1): arrows or WASD to move, J or Z attack, K or X jump, Enter or 1 = start, 5 = coin, F1 = hit boxes; players 2 and 3 use gamepads. Viewer: `--viewer`.
