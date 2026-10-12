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

The full plan with one file per phase is in [`plan/PLAN.md`](plan/PLAN.md). The rule for every
phase: the port must match the original frame by frame (checked with the phase 0 harness).

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
Keys: arrows + Z/X for player 1, 1 = start, 5 = coin, F1 = hit boxes. Viewer: `--viewer`.
