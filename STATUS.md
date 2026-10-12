# Status and next steps

Goal: a native Rust port of *Cadillacs and Dinosaurs* (all phases, full game, input, sound),
loading assets from the user's own `dino.zip`. Rules and file ownership: `dino-rs/ARCHITECTURE.md`.

## Where things stand

| Area | State | Handoff |
| --- | --- | --- |
| Formats: tiles, palettes, sprites, frames, hitboxes, stage maps | Done, extracted by `dino-extract` | `re/specs/anim/SPEC.md`, `re/specs/stage/SPEC.md` |
| GPU renderer (palette shader, 3 tile layers, priority pens), fades | Done | `dino-rs/crates/dino-game/src/render.rs` |
| Camera (all modes), background streaming, parallax | Done; stage 1 background verified | `re/specs/stage/HANDOFF.md` |
| Desert drive stage, row-scroll, screen shake, elevator background | In progress | `re/specs/stage/HANDOFF.md` |
| Draw paths (grab pair, split body, wading, splash, shadows), hit sparks and 3 other effects | Done | `re/specs/anim/HANDOFF.md` |
| Remaining effect classes, pool D | In progress | `re/specs/anim/HANDOFF.md` |
| Player logic (moves, combos, grabs, hurt, lives) | In progress | `re/specs/player/HANDOFF.md` |
| Enemies, spawn scripts, waves and camera locks, items, rank, stage 1 boss | In progress | `re/specs/enemy/HANDOFF.md` |
| HUD, stage start/clear, continue, character select, title | In progress | `re/specs/flow/HANDOFF.md` |
| Input (keyboard for 3 players, coins/starts, gamepads, scripted input) | Done | `dino-rs/crates/dino-game/src/input.rs` |
| Exactness harness (MAME reference traces, port traces, diff, step mode) | Done | `re/harness/README.md`, `plan/phase-00-exactness-harness.md` |
| Sound (Kabuki-decrypted Z80 driver, QSound mixer, cpal output) | In progress | `re/specs/sound/HANDOFF.md` |
| Stages 2-8 enemy classes and bosses | Not started | `re/specs/enemy/SPEC.md` section 3.2 lists the classes |

## Plan

**Current direction (2026-10-12): design first.** The user played the port and found it far from the
whole game (no attract loop, no story, no scene flow, many missing mechanics). The new plan is
[`plan/game/NEW-PLAN.md`](plan/game/NEW-PLAN.md): a design study of the whole game first (T0 probe tool,
D2 player, D3 enemies / dinosaurs / bosses / scenery, D4 world systems, D5 merge), no new game code
until D5 is reviewed with the user; then the whole-game backbone (M0 to M3), sound, real content per
episode, exactness. The game as a player sees it: [`plan/game/README.md`](plan/game/README.md).

The older plan with one file per phase is in [`plan/PLAN.md`](plan/PLAN.md) (D5 rewrites it). The rule for
every phase: the port must match the original frame by frame (checked with the phase 0 harness).

## Progress: design phase (`plan/game/NEW-PLAN.md` section 1)

| Task | State |
| --- | --- |
| D1 Research: story, 8 episodes, attract, mechanics list | Done (2026-10-12), `plan/game/` |
| T0 MAME probe tool and 104 capture recipes | Done (2026-10-12), `re/harness/probe/`, `plan/game/design/capture-recipes.md` |
| X0 Asset dump (no emulator) and catalog | Done (2026-10-12), `re/out/assets/` (committed), `plan/game/design/asset-catalog.md` |
| D4d Front end and story | Running |
| D3e Pool B inventory and area walk, part 1 (Episodes 1 and 2) | Running |
| D2a Player moves | Next (stopped to keep the 2-agent limit, restarts when a slot frees) |
| D4c, D3b, D2b, D2c, D3a, D3c, D3d, D3e parts 2-3, D3f, D4a, D4b, D4e | Pending (at most 2 agents at a time) |
| D5 Merge into the final build plan | Pending |

## Progress: old phase 1 (stage 1 complete; continues as milestone M5 after the backbone)

| Task | State |
| --- | --- |
| 1 Terrain collision | Done (2026-10-12), 0 differences against the 68000 code |
| 2 Items (pool E) | Done (2026-10-12), no item differences in step runs |
| 3 Weapons and pickup (player side) | Done (2026-10-12), no player or item differences in 15 step runs |
| 3b Enemy grab (enemy_grabbed 0x2B35A), throw into enemies | Done (2026-10-12), grab differences gone |
| 4a Pool A: birds, opening scene objects, classes 10 (Hammer T. / Blk Elmer / Wrench T.) and 20 (Punk, Thug) | Done (2026-10-12), step runs match |
| 4a2 Pool A class 14 (Blade, Razor) with its parts (classes 60, 34, 41), class 15 pieces, the shared library 0x32xxx, player pass 0x11962 | Done (2026-10-12), 33 new scenarios (with all four characters) match in step mode; 16 / 17 (stage 2 truck) not ported |
| 4 Remaining stage 1 enemies and props (4b: pool B) | Paused (design first) |
| 5-9 Vice T., join prompts, leftovers, stage flow, exactness pass | Pending |

## Next steps (in order)

1. Read each `HANDOFF.md` and finish the player and enemy ports until stage 1 is playable
   (milestones M3 player controllable, M4 waves and fights, M5 HUD and effects, M6 boss and stage clear).
2. Wire sound (`audio.rs`) into `main.rs` when the sound port reports its API.
3. Port the remaining enemy classes and bosses stage by stage (stages 2-8), then the full flow
   (title, select, continue, ending).
4. Commit and push at each green, verified milestone.

Working mode: one agent at a time on this tree (parallel agents broke each other's builds).
Each agent finishes, the build is green, the milestone is committed, then the next one starts.
Agents must not spawn their own sub-agents.

## How to run

From `dino-rs/`: `cargo run -p dino-game -- --rom ../dino.zip [--stage N] [--chars 0,2]`.
Keys (player 1): arrows or WASD to move, J or Z attack, K or X jump, Enter or 1 = start, 5 = coin, F1 = hit boxes; players 2 and 3 use gamepads. Viewer: `--viewer`.
