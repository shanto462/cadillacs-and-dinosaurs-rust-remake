# Phase 1: stage 1 complete

**Goal:** stage 1 (internal stage 0, the flooded city) plays exactly like the original from the
first frame to the stage clear screen, for 1 to 3 players.

## Classes used by stage 1 scripts (from `re/specs/enemy/stage_scripts.txt`)

- Pool A: 2 (Ferris / Gneiss / Driver, done), 10 (Hammer T., Blk Elmer, Wrench T.),
  14 (Blade, Razor), 18 (Vice T. boss, Walther), 20 (Punk, Thug), 49 / 50 (background birds).
- Pool B: 12, 14, 15 (wave controller, done), 20, 31, 40, 66, 69, 73, 74.
- Pool E: 5 (containers and the items they drop).
- Found by the phase 0 comparison, not in the scripts: pool A 54 and 55 (opening scene, see task 4).

## Tasks (one agent each, in order)

How to check a task: `re/harness/README.md`. Every task ends with a step run (`MODE=step`) of the
scenarios `stage1_jack`, `stage1_late`, `explore` and `stage1_fight` (commands in the README) in
which the classes of the task no longer appear as `[NOT PORTED]`, and the new `[ported]` lines show
no difference. Add the class to `PORTED_CLASSES` in `dino-game/src/game/trace.rs` when it is ported.

1. **Terrain collision (all actors). DONE (2026-10-12).** Ported in `dino-game/src/game/terrain.rs`
   (tables through `dino-extract/src/terrain.rs`): 0x12CB4 (floor 0x12FE4 + wall test 0x1309C in up to
   5 passes + obstacle test 0x12E9A + snap), 0x12CAC with the ceiling tail 0x1361E, 0x12D4E, 0x12E20,
   0x12DAC, all shape handlers 0x13138..0x1361A (tables 0x1370C shapes, 0x13B0C flags by stage and
   area, 0x136A6 head height), the vertical scene map 0x12C50 / 0x136E4, the floor list pointer
   G.$6AC / $6B0 (0x12BE4 at area start, 0x12C90 at the end of `objects_update`), the obstacle list
   (0x4A76 reset, 0x4A96 register, swap at 0xE9B0) and the player drop-in placement 0x62D6. Wired in:
   players (`player_floor_and_walls`, `player_floor_only` with G.$4F0, clamp skipped with G.$506,
   respawn), enemies (`enemy_floor_collision`, `enemy_snap_to_floor`, stores +$94). `Body.b52` and
   `Player.w94` are new; G.$6AC / $6B0, G.$4F0, G.$506 and the obstacle list are traced.
   **Checks:** `cargo build` / `cargo test` green (27 terrain unit tests); the new test
   `terrain_matches_the_68000_code` replays 250 000 vectors from `re/harness/terrain_oracle.py` (the
   original routines run in a small 68000 interpreter, see `re/harness/README.md`): all routines, every
   shape handler variant in the data, the 23 stage / area floor lists, obstacles, vertical scene: 0
   differences. (The oracle found two mistakes of the first port: the signed overflow of `ble` after
   `sub.w` and the vertical scene ceiling. All quirks kept on purpose are listed in the module doc.) Step runs of `stage1_jack`, `stage1_late`, `explore`,
   `stage1_fight` and the new `obstacle` scenario: the terrain differences are gone (player.wall at
   stage1_jack 313, explore 335 incl. `player.x` and the camera `max_x60` / `min_x64`, stage1_fight 186 /
   441 / 473 / 780 and the obstacle attack); no new differences in ported classes (the new `obstacle`
   run has no player difference at all, Jack pushed by a container: wall 0x80, +$94). What is left in
   ported classes is not terrain: `player.kills` / `aggression` at frame 48 (unported opening scene
   objects) and enemy class 2 held by a player from explore frame 462 / obstacle 207 (`enemy_grabbed`
   0x2B35A is a stub, see below).
   Evidence of the task as first found: stage1_jack frame 313 (`player.wall`), explore 335,
   stage1_fight 186, 441, 473, 780.
2. **Items (enemy area).** Pool E class 5 and the item objects: food (heal), treasure (points),
   weapons with ammo, drop tables (0x24B36, 0xB6D12), pickup rules, the item list the player
   reads. Spec: `re/specs/enemy/SPEC.md` section 13. Handlers from the table 0x233F0: class 0
   0x24D44 (weapons), class 5 0x28C76 (containers: hit by Jack and by thugs, hit sparks, the break
   sounds 0xF5 0x6E, debris effect class 0x0B 0x934F6; G.$67BE changes at explore frame 109),
   class 9 0x2A028 (treasure). A sound 0x34 repeats every 51 frames from frame 967 of stage1_fight;
   its source is not identified yet (probably an item). Pool E update 0x233C6 (also the free
   list G.$65FE / $65F4). Phase 0 evidence: explore / stage1_fight frames 109, 789, 791.
3. **Weapons and pickup (player area).** `find_pickup` 0x5AA2, pickup state, held weapon states
   (0x201/0x202 melee, 0x203 charge, 0x206/0x207 guns, 0x209 recoil, 0x20A heavy object), ammo,
   dropping on hit. Spec: `re/specs/player/SPEC.md` section 4.9.
4. **Remaining stage 1 enemies (enemy area).** Pool A classes (handlers from table 0x2346C):
   10 0x3B828, 14 0x4020E, 20 0x457B0, 49 0x56A70 (three off-screen bird controllers, present at the
   first frame), 50 0x56C80 (birds spawned at run time, one `rand()` each), and the opening scene
   objects that exist at the first controllable frame and are gone by frame ~60: **54 (0x36) 0x576D2 and
   55 (0x37) 0x5785C** (their death calls the shared 0x2AF86: kills `+$10B`, rank 0x182A2, `rand()`,
   and the voice 0x9796). Pool B classes (table 0x2362E): 12 0x78B52, 14 0x7B0BE, 20 0x7BB68 (the idle
   "GO" hint: scroll1 rows 46-49, bank 1 palette 10, sound 0x3E, timer G.$4FE, 600 frames),
   31 0x81046 (draws a sprite list kept in pool D slot 15 at 0xE574 and rewrites it every frame),
   40 0x836A4, 66 0x86D76 (scroll1 text banners, rows 57-58), 69 0x88A84, 73 0x89ABA, 74 0x89B48.
   Decode their roles first.
5. **Vice T. and Walther (enemy area).** Class 18 boss logic, summons, rage, the bullet (class 59),
   the boss HP bar and the boss-defeated flow. Spec: section 12.
6. **In-game prompts of the join task (flow area).** The INSERT COIN prompts of absent players blink
   in scroll1 during play (rows 36-37, columns 17-30 and 33-46, 48 frames on and 80 off): task 0x6C4A,
   redraw 0x7562 (`flow_join_task` and `flow_join_prompts` are stubs). The rest of the join and
   continue logic is phase 2, task 4; port at least the in-game prompts here, or the exactness
   pass cannot match.
7. **Enemy library leftovers (enemy area).** In ported code, known from the enemy handoff and not
   yet seen in a trace: effects 0x606E (water splash), 0x92E5E (dust), 0x94338 (dizzy stars),
   0x93D48 (burn death), 0x2AD98 (flames), the second drop call of the burn death (0x2ACD2 /
   0x2ACE6), 0x1D95E / 0x1DAC4 (scripted walk to the next area), the guard handlers 0x10C10 and
   0x10B50, the multi-part boss link 0x1128E. Check them with scenarios that use them.
8. **Stage 1 flow (flow area).** Area changes (mansion, street), stage clear score tally,
   continue into stage 2 setup (stub the stage 2 content).
9. **Exactness pass.** Phase 0 comparison over a full stage 1 run (scripted inputs, 1 and 3
   players): `run` mode, no `follow`, from the first controllable frame. Fix differences. Extend the
   harness first where it is blind (scroll registers, pan and volume of sound requests, see the
   phase 0 status).

## Done when

The phase 0 comparison of a full stage 1 run shows no unexplained difference, and the boss fight
and stage clear screen match the original.

## Status

In progress. Task 1 done (2026-10-12). Handoff notes: `re/specs/enemy/HANDOFF.md`,
`re/specs/player/HANDOFF.md`, `re/specs/flow/HANDOFF.md`.
Tasks 1 to 3, 6 and 7 come from the phase 0 comparison (2026-10-12).

### Findings of task 1 for the later tasks

- **Items / task 2:** containers (pool E class 5, types `$26` = 0, 1, 2, 5, 6) call `item_register_obstacle`
  (0x4A96, from 0x28F46 / 0x28F70) every frame they are alive: it clears +$AC and pushes the object on
  the obstacle list; the obstacle test sets +$AC = 1 when something climbs onto it (the container
  handler reads +$AC at 0x28ECE). The harness hold (`item_hold_obstacle` in `item.rs`) only imitates this for
  frozen containers in state 2 / 0 with HP; replace it when the class is ported. Until then `run` and
  `follow` runs have no obstacle list (the snapshot containers stay in state 0, MAME's init runs in
  frame 109 of explore / stage1_fight / obstacle). Items call 0x12CB4 themselves (`terrain_collide(b, h,
  true)`: pool E objects skip the obstacle test).
- **Hit boxes:** the box table pointer +$44 of an object may carry flag bits in its top byte (a
  container has 0xC0100000); the 68000 address bus ignores them (`& 0xFFFFFF` in the obstacle test).
  `Body.box_table` keeps them as loaded.
- **Enemy classes (task 4, boss task 5):** the calls of the unported classes use `terrain_collide`
  (0x12CB4 callers), `terrain_collide_free` (0x12E20: 0x484EA, 0x48DE0, 0x4E0A2, 0x4EE90, 0x5356A,
  0x53B42, 0x5AC62, 0x5B2AC, 0x5F830, 0x61A42, 0x63678) and `terrain_floor_lookup` + `terrain_player`
  (pool B class 14: 0x7B680, 0x7B69E, 0x7B6F2). Effects of phase 5 use `terrain_effect_floor` (0x12DAC:
  0x9361C, 0x93660, 0x936BE, 0x9370E, 0x93ECA, 0x947CE, 0x95B2A) and `terrain_floor_only` (0x12D4E:
  0x607CE, 0x60F42, 0x60FFA, 0x6114E).
- **Grabbed enemies (not in any task yet):** `enemy_grabbed` (0x2B35A) releases at once. In explore frame
  461 and obstacle frame 206 Jack grabs a thug; MAME follows with state 0x0206 / act 2 and moves the enemy
  to the player's hand (x, z, frame, `+$70` partner), the port frees it. Needs its own task (probably with
  task 3, the player's grab and throw side).
- **Scratch RAM:** G.$6B2 (class flag of the last wall lookup) and G.$6B4 (pass counter) are not modeled;
  they change in MAME and show up in the harness as NOT MODELED globals.
- **Terrain meaning of +$50 bits:** see `re/specs/enemy/SPEC.md` section 16 (what the port knows).
- **Floor lists:** areas that do not exist in a stage (stage 1 area 3, stage 2 area 1, ...) point back
  into the offset table of the stage; they are never used. Stage 1 (internal 0) areas 0..2 are one flat
  segment at height 0, so all the stage 1 terrain is in the scroll2 tile classes.

