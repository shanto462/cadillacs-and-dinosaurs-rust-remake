# dino-rs architecture

A native Rust port of the game logic of Capcom's *Cadillacs and Dinosaurs* (CPS1, 1993). It loads
graphics and data tables from the user's own `dino.zip` at runtime. **No emulation:** no 68000 code
runs, and there is no memory image. Every original routine is rewritten as plain Rust.

## Crates

| crate | role |
| --- | --- |
| `dino-assets` | Data model shared by extractor and game (`AssetPack`). Plain types, no ROM access. |
| `dino-extract` | Reads `dino.zip`, decodes tiles, palettes, sprite frames, hitboxes, maps, tables into `AssetPack`. The only crate that reads ROM bytes. `cargo run -p dino-extract` prints a summary. |
| `dino-game` | The game: `game/` is the logic port, `render.rs` the GPU renderer, `present.rs` turns game state into draw commands, `main.rs` the window and fixed 60 Hz loop. |

## The one rule: data vs code

- Anything that is **data** in the original (tables, frames, maps, palettes, scripts, stat rows)
  goes through `dino-extract` into typed `dino-assets` structs. The game never reads ROM bytes.
- Anything that is **code** in the original is ported by hand. Constants that live in instructions
  (`move.w #$64,...`) are written in Rust next to a comment with the instruction address.
- Keep ROM addresses as names: table addresses identify data (`assets.sprites.anim(0x34892, 3)`),
  and every ported function says which routine it ports (`/// 0x1A5D0 jump_entry`).

## Exactness rules

- One `Game::tick()` is one video frame of the original. Never use real time in logic.
- Keep the 68000 integer semantics: `u16`/`i16` with `wrapping_*`, 16.16 positions (`Fixed`),
  8.8 velocities, unsigned vs signed compares exactly as the branch used (`bcs/bhi` = unsigned,
  `blt/bgt` = signed). Keep quirks and bugs; note them in a comment.
- Random numbers: only `Game::rand()` (0x119C), called in the original order.
- Sounds: `Game::sound(id)` records the request (audio is not implemented yet).

## Objects

- `Handle` = the object's original 16-bit RAM address (players 0xB274 / 0xB3F4 / 0xB574,
  pools from `POOL_*` in `game/mod.rs`). Links between objects (target, last attacker, grab
  partner) are handles, as in the original. `slot_of(h)`, `g.body(h)`, `g.body_mut(h)` resolve them.
- `Body` (`game/body.rs`) holds the fields every object shares at the same offset with the same
  meaning (position, state bytes, velocities, animation, boxes, hit fields, HP). Kind-specific
  fields live in the kind's struct (`Player`, `Enemy`, `Item`, `Prop`, `Effect`) next to `b: Body`.
  Name fields by meaning and put the original offset in the doc comment.
- Pools (`Pool<T>`) copy the original free lists: slot N-1 is allocated first, freeing is LIFO.
  When porting a free routine, reset exactly the fields the original clears (often only +$00..+$27).
- Update order inside a pool: slot order 0..N-1, only slots with `live != 0`.

## Frame order (`game/mod.rs`)

`tick` -> `flow_frame` -> `game_frame` (0xAAF6):
`rank_tick` 0x186CC, `objects_update` 0xE90E (census 0x3242E, players 0x18834, items 0x233C6,
enemies 0x23420, shake 0x222D6, props 0x23604, effects 0x237B2, held players 0x188DA, background
0x2042E), `collision` 0x10546 (passes in `combat.rs`), `hud_update` 0x16BEA, `build_sprite_list`
0x13FA6, `combat_lists_reset` 0x105E8, `time_limit`, `spawn_script_tick` 0xFBF4; then the fade task.

## Naming rule for `impl Game` methods

Every module adds methods to `Game`, so all method names share one name space. Prefix the
methods you add with your area: `player_`, `enemy_`, `item_`, `prop_`, `spawn_`, `rank_`, `fx_`,
`draw_`, `hud_`, `flow_`, `snd_`, `boss_` (`game/boss.rs`, the boss services), `script_` (existing hook names like `pass_10664`, `hook_a5ea` stay).
Helpers that only need your own object can be methods on your own type or free functions.

## Shared services

| need | use |
| --- | --- |
| animation | `Body::start_anim` (0x120E), `start_player_anim` (0x13A0..), `tick_anim` (returns "finished") |
| movement | `Body::motion` (0x189C), `motion_y` (0x18E0) |
| draw | `g.draw_list_add_check(h)` (0x49CA) / `draw_list_add_if_visible(h)` (0x49C0) each frame |
| hit boxes | `collide::boxes_hit(&assets.sprites, attacker, attacker.attack_box, defender, defender.body_box)` |
| per-frame lists | `g.combat.push(list_id, h)` / `get(list_id)`, keyed by the original count address (0x6718 ...) |
| camera | `g.cam.s2.cam_x/cam_y` (G.$744/$748), lock state `g.cam.s2.lock78`, release with `g.camera_go()` |
| palettes | `g.pal` (palette memory), `g.load_obj_palettes()`, fades in `palette.rs` |
| text layer | `g.vram.scroll1` cells (`Vram::index1(col,row)`) |
| enemy library 0x32xxx (aim, face, slots round the target, screen test, distance class, target check) | `lib_*` in `game/enemy/lib32.rs` (`lib_aim_at_target`, `lib_face_target`, `lib_slot3_new`, `lib_slot6_new`, `lib_off_screen`, `lib_distance_class`, ...), sounds with a toggle `snd_9d44` / `snd_9732` / `snd_96ce` |
| terrain | `g.terrain_collide(&mut body, handle, is_item)` = 0x12CB4 (enemies, props, items; returns the obstacle for +$94), `terrain_player` = 0x12CAC, `terrain_floor_only` 0x12D4E, `terrain_effect_floor` 0x12DAC, `terrain_collide_free` 0x12E20; take the body out of its slot first (`std::mem::take`) when it lives in `g` |

## File ownership (parallel work)

Each area has one owner. Edit only your files; if you need a change elsewhere, ask the main
session. If the build breaks in a file you do not own, wait and retry instead of fixing it.

| owner | files |
| --- | --- |
| terrain collision (shared, phase 1 task 1) | `game/terrain.rs`, `dino-assets/src/terrain.rs`, `dino-extract/src/terrain.rs`; checked by `re/harness/terrain_oracle.py` |
| main session | `game/mod.rs`, `body.rs`, `collide.rs`, `combat.rs`, `palette.rs`, `render.rs`, `main.rs`, `input.rs`, `viewer.rs`, both `lib.rs` files, `rom.rs`, `gfx.rs`, `sprites.rs` |
| stage / camera port | `game/camera.rs`, `game/vram.rs`, `present.rs`, `dino-assets/src/stage.rs`, `dino-extract/src/stage.rs` |
| player port | `game/player.rs`, `game/player_combat.rs`, `game/player_weapon.rs` (a child module of `player.rs`, declared there with `#[path]`), `dino-assets/src/player.rs`, `dino-extract/src/player.rs` |
| draw / effects port | `game/draw.rs`, `game/effect.rs`, `dino-assets/src/effect.rs`, `dino-extract/src/effect.rs` |
| enemy port | `game/enemy.rs`, `enemy_combat.rs`, `prop.rs`, `item.rs`, `spawn.rs`, `rank.rs`, `dino-assets/src/enemy.rs`, `dino-extract/src/enemy.rs` |
| sound port | `crates/dino-sound/`, `dino-game/src/audio.rs`, `dino-assets/src/sound.rs`, `dino-extract/src/sound.rs` |
| flow / HUD port | `game/hud.rs`, `game/flow.rs` (also the `hook_*` stage routines the camera calls), `dino-assets/src/text.rs`, `dino-extract/src/text.rs` |

Allowed small edits in files you do not own (re-read the file right before editing):
- registering a new assets module: one line in each `lib.rs` and one `AssetPack` field;
- appending animation table addresses to `OBJECT_ANIM_TABLES` in `dino-extract/src/sprites.rs`.

Cross-owner calls: the enemy pass 0x115B0 does the loop and box test, then calls the player
port's `player_hit_by_enemy` for the effect on the player; hit sparks go through the effects
port's spawn function (0x12720). Agree on such signatures by leaving a stub with a doc comment.

Do not commit or push; the main session does that.

## How to check your work

- `cargo build` / `cargo test` from `dino-rs/`. Add unit tests for formulas you port (damage, RNG use).
- `cargo run -p dino-game -- --rom ../dino.zip --chars 0 --input "0-60:R,61-64:A" --screenshot out.png --frames 90`
  runs 90 deterministic ticks with scripted player 1 input and saves a screenshot (opens a window briefly).
  Put screenshots under `../re/out/shots/` (ignored by git), then look at them.
- `--viewer` / `--view char,set,anim` shows any player animation with hit boxes.

## Build phase (from 2026-10-13)

The task plan is `plan/game/design/D5-final-plan.md`. Its section 5 lists the new files and their owners
(front end: `game/flow/scene.rs`, `game/flow/text.rs`, `game/flow/attract.rs`, `dino-extract/src/frontend.rs`;
props and effects as child modules with `#[path]` under `game/prop/` and `game/effect/`; the sound driver in a
new crate `crates/dino-sound`). A worker may edit a file owned by the main session only when its task block
says so.
