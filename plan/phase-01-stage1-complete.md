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
2. **Items (enemy area). DONE (2026-10-12).** Ported in `dino-game/src/game/item.rs` and
   `item/{shared,container,hits,small,weapon,blast}.rs` (tables through `dino-extract/src/{sprites,enemy}.rs`):
   - Pool E update 0x233C6 with the class table 0x233F0, allocator 0x4818, free 0x48D4 (free list G.$65FE /
     $65F4), registration into the lists 0x69EE (pickup), 0x6AFC (hurtable), 0x671A (attackers) and the
     obstacle list (0x4A96, `item_register_obstacle`: the `item_hold_obstacle` imitation is gone).
   - Class 0 weapons 0x24D44 (all 15 types: melee, guns, thrown bombs and sticks), class 1 explosions
     0x27E16, class 2 ammo boxes 0x283DE (refill 0x284E2), class 3 gun bullets 0x2860A, class 4 food
     0x2898C (eat 0x28AA4), class 5 containers 0x28C76 (all variants and the explosive barrel: hits by
     players, thugs, thrown items and crushing bodies, hit sparks, break sounds 0xF5 / 0x6E, debris effect
     0x934F6 in pool C), class 7 shell 0x29D1E, class 8 projectile 0x29EC4, class 9 treasure 0x2A028,
     class 10 pellet 0x2A39A, class 11 attack box 0x2A534. **Class 6 (0x29B06, glow overlay of weapon
     type 11) is not ported** (it only draws; the harness holds it frozen).
   - Hit passes 0x11F86 / 0x1201C / 0x120D2 (`item/hits.rs`); items as attackers in the enemy pass
     0x11008 (`player_move_word` / `player_action` in `enemy_combat.rs` read an item's +$A0 / +$C8).
   - Drop rolls 0x24B36 (table 0xB6D12) and 0x24B48 (0xB64F2, fallback row 0xB64D2) from enemy deaths
     and container breaks (`item_drop_from_enemy`, `item_drop_roll`), food heal and score (0x28AA4 with
     the table 0x28B10), treasure scores (0x2A164), the drop hop and landing test (0x24CE6).
   - API for the player (task 3): documented at the top of `item.rs`: `item_list()`, `item_kind(h)`,
     `item_pickup_begin(item, player)`, `item_drop_held(item)`; the held weapon runs inside the item
     class and reads the player's `+$B4 / +$B6 / +$C4 / +$C5 / +$2C / +$A8` (see Findings of task 2).
   - Sound 0x34 (every 51 frames from frame 967 of stage1_fight) is **not an item**: it comes from pool B
     class 31 (0x81046, task 4).
   **Checks:** `cargo build` / `cargo test` green (59 unit tests, 10 new in `item/tests.rs`: food heal and
   score, the drop tables, the fuse, the landing test, the hand position). `PORTED_CLASSES` has the 11 item
   classes. Step runs of `stage1_jack`, `stage1_late`, `explore`, `stage1_fight`, `obstacle`: no pool E
   difference (`item` lines no longer `[NOT PORTED]`, none listed) and the other ported classes are
   unchanged (same enemy class 2 grab difference as task 1). `follow` runs: no item difference in
   stage1_fight, explore, obstacle. `run` runs of stage1_fight still diverge from frame 118 (unported
   enemies call `rand()`), and the items follow from frame 789. Seven new scenarios record items in
   MAME (`items`, `misc`, `containers`..`containers3`, `guns`, `guns2`, `bombs`, `ammo`; README) with
   pokes that place the items; step runs with `DINO_STEP_PLAYERS=1`: the only item differences are the
   transition frames where the player port (task 3) changes state (pick-up, throw, swing). Not exercised
   by any recording (desk-reviewed against the listing): the gun shot code of the player's own
   fire, melee swings that hit, the flights 0x10 and 0x14 of weapons, `weapon_enemy_near`, `item_eat`
   with an enemy-dropped item (food and treasure are unit-tested). Phase 0 evidence of the task:
   explore / stage1_fight frames 109, 789, 791.
3. **Weapons and pickup (player area). DONE (2026-10-12).** Ported in `dino-game/src/game/player_weapon.rs`
   (a child module of `player.rs`: it uses the private helpers of the state machine; tables through
   `dino-extract/src/player.rs`, `PlayerData::t` and `comp_extra`):
   - `find_pickup` 0x5AA2 (item list of the previous frame, the lowest kind wins, a heavy object needs the
     stick towards it except variants 6 and 8, weapons are skipped for 40 frames after a pick-up, nothing in the
     air or with G.$506), the pick-up state 0x14 (entry 0x19832: voice 0x9C20, `item_pickup_begin`, the old
     weapon falls with `item_drop_held`, the waiting ammo refill `+$10F` 0x284E2; handler 0x198B4: 40 frames
     `+$F8`, invulnerability `INV[8]`), the callers in idle 0x191C4 and walk 0x19356 (G.$506 is honoured now).
   - The weapon states: attack 0x16 (0x198DC / 0x19954: a club swings only at an enemy in reach 0x6248, otherwise
     the player takes the throw pose), guns 0x1C standing fire (0x19AB0 with the auto aim 0x64F6) and 0x1E walking
     fire (0x19B60), charge 0x20 (0x19C36: 8 and 45 frame thresholds), recoil 0x22 (0x19CE0), run swing 0x202
     (0x1A174 / 0x1A18C / 0x1A2F2 with 0x13EA and the tail of the run entry 0x19FDE), heavy object jump and
     landing 0x1E / 0x20 (0x1B038 / 0x1B0CA) and fall pose 0x1A8AA.
   - The poses and the **split body** (`+$25` bit 0: upper body = main frame, legs = a second animation layer
     `+$30..+$3F`): idle 0x5BDC, walk 0x5CA6, swing 0x5E5C, gun walk 0x5F76, run 0x1A0F0, layer start 0x1506,
     load 0x1564 (the extra bytes are read at the frame's *first* word, `comp_extra`), step 0x1542 (walk 0x193B8,
     run 0x1A0DE, gun walk 0x19C20), and the `d0` quirk of walk_move 0x1BD94 (`player_pstep_d0`: 0x1A220 stores
     the composition byte in `+$B2`). A layer started before `+$38` was ever set reads the ROM vector table (the
     first walking fire): the extractor keeps the first 0x200 ROM bytes for it.
   - 0x65A4 `item_give_bazooka` (a player that joins in the middle of a stage starts with a bazooka; called
     from the respawn reset with `+$07 == 0`), the enemy fields the player damage reads (`player_enemy_w84` /
     `player_enemy_b87` now read the enemy; the two stubs made every damage wrong by the enemy factor).
   - Item side (`item.rs`, `item/small.rs`, `item/weapon.rs`): `item_enemy_near` (0x6248 for any body: the weapon
     and the player share it), `item_ammo_refill` visible to the player, the `allow(dead_code)` of the API gone.
     **No mistake of the item side showed up**: every item transition matches MAME with the real player.
   **Checks:** `cargo build` / `cargo test` green (69 tests, 10 new in `player_weapon.rs`: find_pickup rules, the
   held-item test, auto aim, the layer load / step, the stale offset pointer, the walk move byte, the bazooka of
   a joining player). Step runs (`MODE=step`, no `DINO_STEP_PLAYERS`) of the old item scenarios (`items`, `misc`, `containers`..`containers3`,
   `guns`, `guns2`, `bombs`, `ammo`): **0 differences in the player and in every pool E class** (HEAD before the
   task: 31 to 77 (class, field) pairs in ported classes per scenario; `items` frame 1064 is MAME's lag frame). Six
   new scenarios (README): `melee` 820 frames, `heavy` 420, `gunwalk` 600, `empty` 1150, `pickups` 360, `armed`
   1000, and `melee`, `heavy` and `gunwalk` again with Hannah, Mustapha and Mess (boot scripts `boot_*.txt`): they
   cover every new state (pick-up, 0x16, 0x1C, 0x1E, 0x20, 0x22, 0x202, 0x1E / 0x20 of the air, the
   layer, the stale layer pointer, the ammo refill, a weapon swapped for another, emptied guns, a bat that hits and
   breaks, a drop on being hit): no player or item difference in any of them (except the prop hit of `armed` below). `stage1_jack`, `stage1_late`,
   `explore`, `stage1_fight`, `obstacle`: the same differences as at HEAD (1, 0, 22, 0 and 15 (class, field) pairs:
   `player.kills` at frame 48 and the enemy class 2 grab, see task 1). `follow` runs of the new scenarios: no player
   or item difference either (`armed` drifts after frame 564, see below). `DINO_STEP_PLAYERS=1` is obsolete (README):
   with it the item scenarios still differ on the frames where the item itself changes the player (throw, shot,
   empty gun), because the loaded record already has the change.
   **Left over (causes):** the HUD weapon display 0x2389C..0x23EDA (scroll1 cells under the life bar, redrawn at a
   pick-up and when the ammo changes: `hud_weapon` is a stub, HUD port); the muzzle and spark effects of the shots
   (pool C classes 0x1F / 0x21, effects port) and the **dust of the recoil** 0x92E5E (pool C class 7, 0x92ECC: not
   spawned, the effects port will need it for task 7); the unported enemy classes and props of the first wave
   (frame 564 of `armed`: the jump kick hits the pool B class 0x28 prop, MAME gives `+$59` and score, the port
   does not); enemy class 2 hit-reaction details (frame 609 of `armed`: `+$76` of an attacking thug is 0x57B6 in MAME;
   a thug killed in a hurt reaction by a fire bomb explosion starts no death animation in the port, seen when
   the bomb of `melee` is thrown to the right, not kept as a scenario).
3b. **Enemy grab (enemy area, with fixes on the player side). DONE (2026-10-12).** Ported in
   `dino-game/src/game/enemy_combat.rs` (tables through `dino-extract/src/enemy.rs`):
   - `enemy_grabbed` 0x2B35A (sub 6; act 0: HUD bar request 0x16B36 kind 1 and on as act 2; act 2: while the holder is
     in mode 2 / sub 4 / frame class 0xC the enemy goes into its hand: table 0x2D0E0 by the holder's `+$20`, the
     enemy's `+$84 / 2` and the attach row `(+$60 high byte) & 0x3F` = (dx, dy, animation of the table 0x2E724,
     facing), X / Y / Z are word writes (fractions stay), dx negated and facing flipped when the holder faces right;
     act 4: check only; the release 0x2B464 was already there). The 0x2D0E0 table (0x15AC bytes) and the grabbed
     animation tables 0x30B64 / 0x30BB0 / 0x30BFC are new extractor entries. The holder's tag `+$82 == 4` skips the
     bar and the placing (`enemy_holder_tag`, a non-player holder).
   - Pass 0x1185A (`pass_1185a`, was a stub): a thrown or fallen enemy body (list 0x6B30) against the other enemies
     (frame class 0 / 2 / 4, body boxes 0x128F8): state 0x02020400, HP minus the enemy attack record 0x11, stun 0, both
     get its hit stop, the victim faces the way the body flies, spark 0. This is the "throw into another enemy".
   - Player side (the trace showed it): the knee's HUD bar request (0x1D792: kind 2 when the enemy's `+$82` is 0, else 0,
     skipped when the knee put the enemy into a reaction), the knee spark of a killing second knee uses the first
     knee's offset (0x1D7B2), and the lie-down and dizzy tables read the level with `move.b $5ee(a5)`: the HIGH byte of
     the level word, so the row is always 0 (`player_level_band`; the port took the low byte and mis-read the lie
     time of every knockdown from level 8 up).
   - Harness: `DINO.spawn_enemy` in `record.lua` (poke a pool A object), `pokes/grabs.lua` and the scenarios
     `grabs`, `grabs2`, `grabs3` (README), run with all four characters.
   **Checks:** `cargo build` / `cargo test` green (75 tests, 6 new: the placement and its facing flip, act 0, the checks,
   the holder tag, the thrown body, the level band). Plain `MODE=step`: `explore` 22 -> 1 (class, field) pairs (the held
   thug of frame 462 matches: only `player.kills` of frame 48 is left, see task 1), `obstacle` 15 -> 0 (frame 207), `stage1_jack`
   1, `stage1_late` 0, `stage1_fight` 0, `melee` 0 and `armed` 4 (the same prop hit and enemy `+$76` of task 3). The new
   scenarios (grabs, both knees, front and back throws, throws into other thugs, releases by a jump or by being hit,
   enemies that die in the hold) run clean for Jack, Mustapha and Mess (Hannah: `player.kills` of frame 2 only; `grabs2` with
   Mess: the enemy item below). `follow` (HEAD of task 3 -> now, pairs in ported classes): `explore` 152 -> 131, `obstacle`
   117 -> 70. The rest is not the grab: the enemies drift from frame 118 (the unported bird class 0x32 draws `rand()`),
   the held thug sits in another pool slot after that (`player.partner` D374 / D454 at obstacle 206), and the wave
   controller and the effects drift with them.
   **Left over (causes):** G.$755C `shake_req` after a thrown enemy lands (frame 1032 of `grabs`, 370 of `grabs2`: the landing asks
   for a shake, `screen_shake` 0x222D6 is a stub of the camera port, MAME has consumed it); enemy class 2 AI details that are not the
   grab: at the start of an attack (state 0x0A04) MAME stores a stack address in `+$76` (0x57B6 / 0x57D6 / ... instead of
   the player: armed 609, `g2` frame 1106) and a different `dir` / speed when the approach starts while the player is in the
   air (`grabs2_mess` 593) or after a get-up (151 of a Hannah run); the unported bird / opening-scene classes as in
   task 2. **Pass 0x10DCC** (items hurting players, 0x10DCC..0x10F94) is not small (about 110 instructions of its own plus
   the dispatch) and nothing the harness can record reaches it with one player (a second player does not join in a
   recording: `2C` / `2S` in a script did nothing); it stays noted in the findings of task 3.
4a. **Pool A classes of stage 1, first part (enemy area). PARTLY DONE (2026-10-12).** Ported, each class checked in step mode
   against MAME (`MODE=step`, 0 differences in every ported class of every scenario below):
   - Class 49 (0x56A70, three background bird controllers), class 50 (0x56C80, birds that fly in, 7 variants), class 54
     (0x576D2, bystander of the opening scene), class 55 (0x5785C, the scripted thugs of the opening scene: their hit
     reaction and grab are the shared 0x2A606 / 0x2B35A), all in `game/enemy/scenery.rs`. Class 55's death is the shared
     0x2AF86 (kills `+$10B`, rank 0x182A2, `rand()`, voice 0x9796), so the `player.kills` / `aggression` difference at frame
     48 of stage1_jack and explore is gone. The bird spawner pool B class 73 (0x89ABA, in `prop.rs`) is ported too: it is what
     makes class 50 appear at run time (it draws the variant and the next delay: two `rand()` calls per bird).
   - Class 10 (0x3B828, HAMMER T. / BLK ELMER / WRENCH T.) in `game/enemy/class10*.rs`: init and the five entrances (walk,
     drop, jump attacks 4 / 6, wait), the AI states 0..0x12, the attack scripts (table 0x3C1E6: punches A / B, the charging
     punch C, the dive D), the paired attack on a downed player (0x6606 = `enemy/claim.rs`, called by the player port, lists
     0x6A5A / 0x6A8E cleared at the start of `enemies_update`), the grab machine and the stomp machine (0x3C60C), the hit on the
     held player (0x125C0), recover (0x2B56C entries 0xC / 0x28 / 0x2C) and death (the class 2 routine).
   - Shared library fixes found on the way: `enemy_angle_to` (0x103A) adds with `add.b` (only the low byte of the
     accumulator changes), which matters for distances above 0xFF (class 2 never had them in a recording); the victim of
     the thrown-body pass 0x1185A turns after the hit spark is placed (0x11950 after 0x1194C); class 2 stores a junk word
     in `+$76` when a dash attack starts (0x35782, a0 is still the table pointer): the leftover `+$76` of task 3b.
     Player side (found by the held-by-an-enemy scenes): `player_mash` (0x6012) continues with the attack press after a stick tap
     (`bcc` at 0x6024), `player_downed_claim` calls the claim routine.
   - Class 20 (0x457B0, PUNK / THUG) in `game/enemy/class20*.rs`: init, the six entrances (two standing waits, walk, dash,
     fade with the palette steps 0x45C32, drop), the AI states 0 / 2 / 4 / 6 / 8 / 0xA, the three actions of state 4 (walk about
     and keep the distance 0x46C24, hops and a jump at the player, the choice machine `+$A5` with a strafe, a hop in place and a
     jump), the attack script (table 0x463F6: punches, the jump kick, the lunge, the leaping dive with the division by 0x33) and
     the recoveries; death is the class 2 routine. Hit recover entries 0x20 / 0x24 -> 0x45D74.
   - New tools: `re/harness/{check.sh, regress.sh, objhex.py, cover.py, genscript.py}`, `re/tools/{dj.py, d2, dc.py}` (README).
   **Not done: class 14 (0x4020E, BLADE / RAZOR) with its parts (classes 60, 34, 41, 15, 16, 17)**; it stays `[NOT PORTED]`
   (frozen in step runs). What the analysis found is in "Findings of task 4a" below and in `re/specs/enemy/HANDOFF.md`.
   **Checks (`re/harness/regress.sh` repeats the step runs):** `cargo build` / `cargo test` green (80 + 4 tests, 5 new: the `add.b` of 0x103A, the 0x6606 claim rules, the division of the dive; the extractor test now uses 0x57D88 as the unregistered
   frame). Plain `MODE=step` runs: `stage1_jack`, `stage1_late`, `explore`, `stage1_fight`, `obstacle`, `melee`, `grabs` and the
   other `grabs*` runs: 0 differences in ported classes; `armed`: the 2 known ones (the prop B#40 hit at frame 564, the
   `+$76` junk of task 3b is fixed). New scenarios `intro` (the opening scene from the stage start: classes 54 / 55 / 49 / 50
   with the master prop loaded from MAME), `c10a`, `c10b`, `c10c` (+ `_hannah`, `_mustapha`, `_mess`), `c20a`, `c20b` (+ the
   same three characters): 0 differences in ported classes in 900 to 2400 frames (the opening scene run `intro`: classes 54 / 55
   differ only in the frame of the spawn by the unported master prop, the player differs in the scripted attack of the scene,
   frames 522 to 556: the scripted player states are not ported). `follow` / `run` of `explore` and `stage1_fight`, pairs of (class, field)
   in ported classes before -> after: explore follow 131 -> 9, run 153 -> 10; stage1_fight follow 96 -> 28, run 193 -> 225
   (more classes run now, so more consequences of one drift are counted). First difference in a ported class or a global:
   before frame 48 (`player.kills`) and the `rand()` drift from frame 102 / 118; after: explore follow 227, run 157,
   stage1_fight follow 227, run 102. All of them come from pool B (task 4b): the first `rand()` difference of a `run` is frame 30
   (class 0x42, 0x86D76, the scroll1 banners draw one), after that the birds and the spawner differ; in `follow` the wave
   controller (class 15) at frame 227 sits in another pool B slot because class 0x42 / 0x28 are frozen instead of freeing
   themselves like in MAME.
   **Remaining differences (all causes):** pool B classes 0x14, 0x1F, 0x28, 0x42, 0x4A (task 4b) and the special object
   (pool D slot 15 at 0xE574, class 31's list); scroll1 banner cells and one palette word (class 0x14 / 0x42); the effects
   classes 3, 7 (dust 0x92E5E), 0xB (debris 0x934F6), 0x13 (dizzy stars 0x94338) and the flames, not ported yet (effects port);
   G.$755C `shake_req` (the landing of a thrown or diving enemy asks for a shake, `screen_shake` 0x222D6 is still a stub of
   the camera port); `armed` frame 564 (the prop hit); the opening scene classes 56 (0x57C9C) and 57 (0x57D94) (only in the
   `intro` scenario).
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

In progress. Tasks 1 to 3b done and 4a partly done (2026-10-12). Handoff notes: `re/specs/enemy/HANDOFF.md`,
`re/specs/player/HANDOFF.md`, `re/specs/flow/HANDOFF.md`.
Tasks 1 to 3, 6 and 7 come from the phase 0 comparison (2026-10-12).

### Findings of task 1 for the later tasks

- **Items / task 2:** containers (pool E class 5, types `$26` = 0, 1, 2, 5, 6) call `item_register_obstacle`
  (0x4A96, from 0x28F46 / 0x28F70) every frame they are alive: it clears +$AC and pushes the object on
  the obstacle list; the obstacle test sets +$AC = 1 when something climbs onto it (the container
  handler reads +$AC at 0x28ECE). (Done in task 2: the hold `item_hold_obstacle` is gone, the real registration runs.) The harness hold only imitated this for
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


### Findings of task 2 for the later tasks

- **Player port (task 3):** the held weapon is run by the item (class 0), the player only keeps its
  state machine. The two sides talk through fields that are set and used inside one frame, so a
  post-frame snapshot does not show them: `+$C4` of the player (attack flag, set by the player,
  cleared by the item when it used the frame), `+$2C` (frame byte: 0 bending, 1 the moment of the grab,
  1 / 2 the swing frame), `+$B6` (held kind, the item sets 0 when thrown or broken, or its new kind when
  a gun runs empty), `+$C5` (drop request), `+$A8`. Until task 3 lands, `DINO_STEP_PLAYERS=1` (README) lets
  the item scenarios compare the items against MAME's player. Without it, step runs of the item
  scenarios show the player's missing pick-up and throw as item differences.
- **`find_pickup` 0x5AA2:** read `item_list()` (the list of the previous frame: cleared in
  `objects_update` after the players); an item qualifies when `live == 1`, `mode == 2`, `sub == 0`.
  Call `item_pickup_begin` / `item_drop_held`. `item_ammo_refill` (0x284E2) is private to `item/small.rs`
  (the ammo box calls it); make it `pub` if the player needs it.
- **Mailbox with enemies:** items attack enemies and crush containers through the lists 0x671A / 0x6AC2 /
  0x671E / 0x6AFC. `player_hit_by_enemy` is still not called (task 4), so enemy attacks hurting a player
  holding a weapon (drop on hit) are untested.
- **Edits in files owned by others (reviewed):** `collide.rs` (box table flag bits masked to 24 bits),
  `trace.rs` (item fields, `PORTED_CLASSES`, `trace_load_players`), `harness.rs` (`DINO_STEP_PLAYERS`),
  `spawn.rs` (G.$43B toggles and the step-players buffer), `enemy_combat.rs` (items as attackers),
  `dino-extract/src/{sprites,enemy}.rs` (item animation tables, drop rows, food / treasure tables, hand position and animation tables, gun tables), and
  `re/harness/record.lua` (`DINO_POKE`, `DINO.spawn_item`).
- **RADIO (stage 3):** the food-like item `+$26 = 0xC` with G.$4D9 = 2 calls 0x8BD24 (three pool B
  objects of class 0x58, the kill-all event). TODO(phase 7) in `item/small.rs`.
- **`enemy_despawn_if_far` (0x173C) in `enemy.rs`** misses the Z + Y limit of the vertical scene (stage 7
  elevator: 0x1782 tests `|X|` and `Z + Y`); the item version (`item_despawn_far`) has it. Fix when stage 7
  is ported (phase 11).
- **Effects:** the dust trail of a thrown weapon (0x92E5E) is still not called (task 7).
- **Scenario cost:** `items` takes the stage1_fight run to break two containers (1250 frames); the poked
  scenarios are quick (200 to 900 frames).

### Findings of task 3 for the later tasks

- **Grab of an enemy (`enemy_grabbed` 0x2B35A, the next step; not ported):** the player side (0x1D652 and the
  states 0x1D746 / 0x1D850 / 0x1D72A, all ported) is exact, only the enemy half is missing. What the listing says:
  the handler dispatches on `+$06` (words at 0x2B366: act 0 = 0x2B36C, 2 = 0x2B38E, 4 = 0x2B434). Act 0: the
  partner `+$70` (the player); if the player's `+$82` is 4 go to act 4, else call 0x16B36 with d0 = 1, `+$06 += 2`,
  `+$62 = 0` and fall into act 2. Act 2 (and 4, without the placing): the partner must be alive, in mode 2, sub 4
  (grabbing) and frame class `+$2E == 0xC`; if not, release (0x2B464: `+$50 = 0`, floor snap 0x3140C, `+$7B = 0x20`,
  continue at 0x2B54E). Placing (act 2 only, when the high byte of the player's `+$60` is negative): the table
  pointer is `long[0x2D0E0 + player +$20]`, the word offset is read at `+ enemy +$84 / 2`, the row is
  `(+$60 & 0x3F) * 8` bytes further, `(dx, dy, anim, facing)`: X = player X + dx (dx negated and the facing bit
  flipped when the player faces RIGHT, `+$24 != 0`; ported in task 3b), Y = player Y + dy, Z = player Z, `+$24` = the facing, and the
  animation is word `anim` of the table `0x2E724[enemy +$84]` loaded with 0x1236 (the grabbed animations; the hit
  animations are 0x2E68C). MAME shows state 0x0206 act 2 for the held thug (explore frame 461, obstacle 206).
  0x2B54E, 0x2B478 (the wait `+$7A` by 0x2B8C4), 0x2B4BA (`+$80` by 0x2B614 / 0x2B604) and 0x2B506 (wrap X by
  0x400 at the camera edges) follow in the same listing. The held-player tables of the other direction are
  `held_offsets` (0x2CBE0, already extracted); 0x2D0E0.. still has to go through `dino-extract`.
- **HUD weapon display (flow / HUD port):** `hud_weapon` 0x2389C..0x23EDA is a stub. It draws the weapon icon, name
  and ammo digits as scroll1 cells at `HUD area + 0x5C` (rows of the player's message area, 0x80 bytes apart) when
  the held kind (`+$B6 & 0xFF` - 4) or the ammo (`+$6C` of `+$B4`) changed (caches G.-$70A8 and G.-$70A0 per player,
  dirty flag G.-$7098 `weapon_dirty`). All fields it reads are in the player and traced. In the weapon scenarios the
  scroll1 differences are exactly the frames of a pick-up and of an ammo change.
- **Effects (effects port, task 7 and phase 5):** the recoil of the bazooka calls 0x92E5E (d0 = 2, pool C class 7,
  handler 0x92ECC) every other frame while the player slides (`player_recoil`, guns2 frames 105 to 122); the item
  side already skips the dust of thrown weapons. The shots spawn the muzzle classes 0x1F / 0x21 (handlers 0x96800 /
  0x96B78); they keep the pool C free list apart (`global.free_count`), the diff shows them as `effect class
  0x01 [ported]` rows in `guns`, `guns2`, `empty` and `armed`.
- **Items that hurt players (0x10DCC, `pass_10dcc` is still a stub in `enemy_combat.rs`):** attackers in the list
  0x671A (frame class 4: swung or thrown weapons and containers) hurt a player of another `+$02` (friendly fire of
  1 HP with 2 or 3 players; items owned by an enemy, `+$02` negative, use the damage of the attack record) and set
  the hurt state 0x02020000 like `pass_10664` does, but the weapon is dropped without the Hannah exception of 0x1177A
  (`+$B6 = 0`, `+$C5 = 1`, 0x10EA2). Needed for stage 1 with 2 or 3 players. No recording exercises it (the harness
  has no join yet).
- **Not weapons, but in the player code:** the extra life pose (states 0x18 / 0x1A, `+$10E`, started by the score
  table 0x6B98 when the score passes a threshold, calls 0x1843C / 0xA4CA / 0x162F8), the two-player tie-up (0x6728..
  0x6922: state 0x1A with the partner, run state 4 0x1A330, `+$C8` 0x12), the scripted states 0x2A / 0x2C, the air
  states 0x18 / 0x1A / 0x1C / 0x22..0x26 and 0x34.
- **`DINO_STEP_PLAYERS` is obsolete** (README). New fields in the trace: none (everything the weapon states touch was
  traced: `+$B4..+$BA`, `+$C4`, `+$C5`, `+$EA`, `+$F8`, `+$10F` and the split layer `+$30..+$3F`).
- **Layer quirk to keep in mind:** the offsets of the upper body come from `+$38 + 4 * (signed +$35)`. `+$38` is
  set by 0x5CA6, 0x5F76 and 0x1A0F0, and 0x19B60 starts the layer *before* 0x5F76: with `+$38 == 0` the first walking
  fire reads the ROM vector table (`dx3c` = 0x00FF, `dy3e` = 0x0E66 in MAME). Reproduced; `draw.rs` already draws the
  split body.

### Findings of task 4a for the later tasks

- **Class 14 (BLADE / RAZOR, 0x4020E..0x41F1A) and its parts.** One script record makes THREE objects: init sub 0 (0x40224) takes
  two more pool A slots, sorts the three addresses and turns the middle one into the class 14 object (`+$A8` / `+$AA` = the
  other two), the others into class 60 (`+$20 = 0xF0`, 0x580C6, `+$26` = 0 / 0x200, anim table 0x6FB6A, `+$A0` = the master):
  small draw-only parts (knife / weapon art) that follow `+$BE` (knives left, 0..2) of the master (0x41D6E places them with the tables
  0x58178 / 0x58308). The alive code is a table-driven state machine (`+$06` 0..0x3A, 30 handlers at 0x40EBC..0x41154 listed in the
  table at 0x407F8; the entrance by `+$27` is table 0x403E8; the "state entry" routines 0x41278..0x41954 are reached through the word
  tables 0x41C3A / 0x41D56, the decision 0x41BE8 reads the ROM tables 0x73214 / 0x73414 / 0x73494 / 0x73514 / 0x73714 / 0x73914 by
  variant, aggression and `rand()`). It throws knives (class 34, 0x5129E: `0x40466 -> pool A class 0x88`) and class 41 (0xA4, 0x55866: a
  thrown blade) and spawns class 15 / 16 / 17 pieces (0x41EEA, 0x41F1A, 0x420EA, 0x423A4: debris of a hit knife?). Shared library
  routines it needs (0x32xxx, most are tiny): 0x32A58 / 0x32A70 / 0x32A88 / 0x32B68 / 0x32B94 / 0x32C5E / 0x32D86 / 0x32D90 /
  0x32F56 (aim and slot helpers, they read `+$E6` / `+$E7` of the player), 0x3282A (is it on screen), 0x3293C (distance class) and
  0x9D44 (sound with pan). Death (0x41E38): one `rand()` (the 1/64 anti-tamper call 0x2A5B6 does nothing on a genuine ROM), kill
  score, 60 frames of blinking.
- **Class 20 (PUNK / THUG)** is done (see above); its listing needed care because a table before the code made the disassembler print
  the first instruction of many handlers as garbage (0x45F64 `clr.b $a5(a6)` and 0x4611C `move.b #2,$a4(a6)` were found that way):
  print each handler start again with `re/tools/d2 ADDR 4` before trusting a reading. Things the port gets from the listing that are
  easy to misread: `add.b d0,$14(a6)` adds to the HIGH byte of the word at +$14 (big endian), `bcc` after `bsr` of a window routine
  is the opposite polarity of `bcs` (0x45FF8 jumps when the window test fails), the fall-through of one handler into the next
  one in memory (0x45FCE -> 0x45FF8, 0x46606 -> 0x46642), and `bls` after the direction turn keeps the NEW direction.
- **Method that worked for class 10 (reuse it):** read the listing with `re/tools/d2` (tables decoded), port function by function with
  the original address in the comment, record a poke scenario (`DINO.spawn_enemy`, all entrances and variants, 5..8 enemies, Jack with
  HP 0x190, a `genscript.py` input), run `MODE=step`, fix the first difference, repeat; `cover.py` shows which states a recording
  reaches. The step run found: the `add.b` of 0x103A, the spark order of 0x1185A, the junk `+$76`, the mash routine, the missing
  hit-recover entry, the sound toggle 0xA062 (G.$43B bit 3). Tables go through `dino-extract` (`TABLES` in `dino-extract/src/enemy.rs`).
- **Quirks of class 10 kept on purpose:** the grab test 0x3C726 and the stomp test 0x3C9C8 add a bias of 0x400 to the X difference
  (the intended window sits 0x400 px away), so the thug never grabs in a real stage (no recording shows the 0x0204xx04 states);
  0x3D0B4 adds the zero `d2` to the direction (no effect); the first thug of a claim can use the player's stale `+$88` (0x6606
  scratch) as its aim point (0x3CCC0).
- **For 4b (pool B):** class 73 (0x89ABA, bird spawner) is done. Classes 12, 14, 20 (GO hint), 31, 40, 66, 69, 74 are not; class 74
  (0x89B48) is the master of the opening scene: its `+$05` is what classes 54 and 55 follow (`+$A4` = its handle), it also spawns class 54
  (0x89B90.., `+$20 = 0xD8`) and probably 55 / 56 / 57. Class 66 (0x86D76) draws one `rand()` at frame 30 of every recording and is the
  first source of the `run` drift.

