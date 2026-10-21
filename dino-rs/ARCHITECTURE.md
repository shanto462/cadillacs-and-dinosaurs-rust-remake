# dino-rs architecture

A native Rust port of the game logic of Capcom's *Cadillacs and Dinosaurs* (CPS1, 1993). It loads
graphics and data tables from the user's own `dino.zip` at runtime. **No emulation:** no 68000 code
runs, and there is no memory image. Every original routine is rewritten as plain Rust.

## Crates

| crate | role |
| --- | --- |
| `dino-assets` | Data model shared by extractor and game (`AssetPack`). Plain types, no ROM access. |
| `dino-extract` | Reads `dino.zip`, decodes tiles, palettes, sprite frames, hitboxes, maps, tables into `AssetPack`. The only crate that reads ROM bytes. `cargo run -p dino-extract` prints a summary (`src/main.rs`; `anim <table_hex> <id>` dumps one animation's frames). |
| `dino-game` | The game: `game/` is the logic port, `render.rs` the GPU renderer, `present.rs` turns game state into draw commands, `main.rs` the window and fixed 60 Hz loop. `input.rs` reads keys and pads (see Input), `audio.rs` the sound output. |
| `dino-sound` | Port of the QSound Z80 sound driver: `driver.rs` (mailbox, 250 Hz IRQ, commands), `seq.rs` (sequencer, events, notes), `voice.rs` (per-frame voice steps, LFO, envelope), `pan.rs` (pan setup), `map.rs` (Z80 RAM map and field offsets), `mixer.rs` (16 voices, echo, f32 stereo out), `tests.rs`. Reads the sound data from `dino-assets` (`sound.rs`); no ROM access and no dependency on `dino-game`. |

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
- Sounds: `Game::sound(id)` (and `sound_ex(id, d1, d2)`) queues the request in `w.snd_reqs`, cleared each tick. `audio.rs` feeds the
  requests to `dino-sound` after the tick. The game never reads the sound side.

## Screen latency (what the picture shows)

The real board shows the game a few frames late. The port keeps that, so MAME pictures and ours match. The game state is always
the live one; only `present.rs` (through the latches in `Game::tick`) reads the older values:

- Scroll X registers of scroll1 and scroll2, and the scroll3 X and Y: two frames late (`camera::VIDEO_LATENCY` = 2, `Camera::shown_scroll`).
- Scroll Y of scroll1 and scroll2: one frame late (same function).
- Layer registers (layer control, video control, scroll3 base): one frame late (`LayerRegs`, `Camera::shown_layers`).
- Sprite list: two frames late (`SpriteList::latch`, `submit`). Palette colours: one frame late (`PaletteRam::latch`).
- Tile maps and the priority masks: at once.

The first latch fills the history, so a game that starts in a scene shows its scroll and sprites at once.

## Input and the combo toggle (`input.rs`, `main.rs`)

- `Inputs::poll(hero)` builds one `input::Frame` per video frame in the window: player 1 from the keyboard, players 2 and 3
  from gamepads (gilrs; keyboard still works without a pad), system bits (start, SERVICE) and coins.
- `Player1Keys::frame` is the pure per-frame function that the tests drive (`input.rs` tests). It takes the held keys, the Q press,
  the special move key pressed (if any) and `Hero`. A move key starts its sequence (`Move::frames`, the way to go is the last
  horizontal direction held). While a sequence runs it replaces player 1's buttons.
- Special move keys (`MOVE_KEYS`): A `Command`, S `Dive`, D `JumpAttack`, F `Special`, G `Dash`. Each plays the original's own input sequence.
- Q toggles `Combo`. While it is on, the attack press that would be the third hit of a punch chain plays the command move
  instead. The chain is the game's own hit count `+$B2` (it counts only hits that land, a miss sets it back to 0). The move waits up
  to `Combo::WAIT` (60) frames until the hero is ready, and only while the chain stays at its third hit; the press stays off until the key is let go.
- `HeroWatch` (`main.rs`, window loop only) reads the hero's chain state from the game before each tick and fills `input::Hero`
  (`third_next`, `ready`). The game logic does not see the combo.
- P toggles `user_paused` in `main.rs`: no game tick runs while paused, the picture stays and the sound stops (the audio frame runs only for the test menu's pause). The PAUSED text and the COMBO label (while Q is on)
  are drawn in the window only.

## Check tools

These modules are review aids, not game logic: `autoplay.rs` (the bot), `planner.rs` (`--autoplay smart`, a beam search on copies of
`Game`), `debug.rs` (debug keys), `menu.rs` and `jumps.rs` (test menu and jump points), `headless.rs` and `harness.rs` (CPU screenshots, trace runs).

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
0x13FA6, `combat_lists_reset` 0x105E8, `time_limit`, `spawn_script_tick` 0xFBF4; then the fade task. `Game::tick` (`game/mod.rs`) runs `flow_frame`, then `skip_wait_rounds` (0x9E38C), then the latches that the screen reads (see Screen latency).

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
| player port | `game/player.rs`, `game/player_combat.rs`, `game/player_weapon.rs` (a child module of `player.rs`, declared there with `#[path]`), `game/player_tag.rs` (the tag team move, a child module of `player.rs`), `dino-assets/src/player.rs`, `dino-extract/src/player.rs` |
| draw / effects port | `game/draw.rs`, `game/effect.rs`, `dino-assets/src/effect.rs`, `dino-extract/src/effect.rs` |
| enemy port | `game/enemy.rs`, `enemy_combat.rs`, `prop.rs`, `item.rs`, `spawn.rs`, `rank.rs`, `enemy/claim.rs` (the claim of a downed player, 0x6606), `enemy/scene_actors.rs` (opening scene actors), `enemy/scenery.rs` (scenery and scene classes), `item/*.rs` (child modules of `item.rs`: weapons, containers, blasts, small items, the item hit passes), `dino-assets/src/enemy.rs`, `dino-extract/src/enemy.rs` |
| sound port | `crates/dino-sound/`, `dino-game/src/audio.rs`, `game/snd.rs` (the 68000 sound requests), `dino-assets/src/sound.rs`, `dino-extract/src/sound.rs` |
| flow / HUD port | `game/hud.rs`, `game/flow.rs` (also the `hook_*` stage routines the camera calls), `dino-assets/src/text.rs`, `dino-extract/src/text.rs`; `game/flow/*.rs` (boot, attract, movie, demo, slides, profiles, ranking, picture, scene, text, system, select, front, join, cont, name_entry, tally, cards, ending, config, skip, mame_check), `game/script.rs` (scripted control), `dino-assets/src/frontend.rs`, `script.rs`, `dino-extract/src/frontend.rs`, `script.rs` |
| enemy port (bosses and classes) | `game/enemy/classNN*.rs` (one file or a small family per class), `game/boss.rs`, `game/boss_common.rs` (the shared boss skeleton), `game/car.rs`, `game/prop/*.rs` (child modules of `prop.rs` with `#[path]`, one per pool B class or family), `dino-assets/src/boss.rs`, `dino-extract/src/boss.rs` |
| draw / effects port (pool C) | `game/effect/fxNN*.rs` (child modules of `effect.rs` with `#[path]`, one per pool C class or family), `game/effect/fx_cloud.rs` (ranking clouds), `fx_tally.rs` (stage clear tally), `fx_time_over.rs` (the TIME OVER words) |
| review tools (main session) | `autoplay.rs` (the review bot), `debug.rs` (debug keys), `menu.rs`, `jumps.rs`, `font5x7.rs` (the test menu and jump points), `headless.rs` (the CPU renderer), `harness.rs` and `game/trace.rs` (the trace writer and the step loader), `re/harness/*` |

Allowed small edits in files you do not own (re-read the file right before editing):
- the edits a task block of `plan/game/design/D5-final-plan.md` names as allowed (workers edit main session files only then);
- the wiring lines a new class needs: its `mod` line and dispatch arm (`enemy.rs`, `prop.rs`, `effect.rs`), `PORTED_CLASSES` and trace fields in `game/trace.rs`, ROM table entries in the `TABLES` lists of `dino-extract`;
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
