# Enemy port: handoff (state at the wrap-up)

Code: `dino-rs/crates/dino-game/src/game/{enemy.rs, enemy_combat.rs, spawn.rs, rank.rs, prop.rs, item.rs}`,
`dino-rs/crates/dino-assets/src/enemy.rs`, `dino-rs/crates/dino-extract/src/enemy.rs`. Spec: `SPEC.md` (this folder).

## Ported (faithful to the listing, with addresses in doc comments)
- **Extractor** (`dino-extract/src/enemy.rs`): spawn lists 1 and 2 for all stages and areas (0xFC2E / 0xFC4E),
  wave scripts (0x7B37C), boss reinforcement lists (0x7BF36 / 0x7BF56), HP (0x31454), boss HP (0x31E52),
  enemy attack records (0xA2EA4), speed tables (0x17CE), the reverse-step map for 0x1380, and about 50
  small tables as byte blobs addressed by ROM address (`ByteTables`, list `TABLES`).
- **rank.rs**: 0x17F8A game start, 0x1800E stage start, 0x18048 area start, 0x186CC tick, 0x18592 level,
  0x182A2 aggression re-roll, 0x18566 / 0x18410 / 0x1842C / 0x1843C / 0x184E2 rank events,
  0x1031A / 0x1037E / 0x103FE / 0x104A8 stage time limit (BCD).
- **spawn.rs**: 0xFBF4 interpreter (lists 1 and 2, commands 0..0x10), 0xFDAE / 0xFE6A record spawn with
  player-count and level conditions, 0xFBE8 reset, pool resets of 0xAA0E / 0xAA92..0xAAD8. Enemy-side
  globals (`spawn.g`: G.$4F3, $50E, $4E0, $50C, $518, $502, kills, census counters).
- **prop.rs**: pool B update 0x23604; wave controller class 15 (0x7B336) complete (thresholds, delays,
  7200-frame budget, kill-all on timeout, end types 0/2/4, 0x1E5F6 leader).
- **enemy.rs**: pool A update 0x23420, free 0x48FC (partial clear), census 0x3242E, target 0x32032, the
  library (0x17A4, 0x1862, 0x103A, 0x191E, 0x324D4, 0x1380, 0x324EE, 0x193E, 0x173C, 0x31424, 0x31E14,
  0x326F8 kill-all, 0x325E6 kill score, BCD score add 0x10036), and **class 2 FERRIS / GNEISS / DRIVER
  complete** (init, 6 entrances, AI states 0/2/4/6/8/A/C/E/10, slots, combos, dash, death 0x3484E).
- **enemy_combat.rs**: pass 0x10F96 -> 0x11008 player hits enemy (with 0x2BA2A damage scaling),
  pass 0x115B0 loop + box test (call to the player port commented), hit reaction 0x2A606 types
  0/2/4/6/8/A/C/E..18/1A/1C, get-up / dizzy 0x2AF86..0x2B2B6, wall slump, recover 0x2B54E.
- **Held enemy (phase 1 task 3b)**: `enemy_grabbed` 0x2B35A (sub 6): act 0 queues the HUD bar (0x16B36, kind 1) and
  falls into act 2, act 2 puts the enemy into the holder's hand (table 0x2D0E0 by the holder's `+$20`, the enemy's
  `+$84 / 2` and the holder's attach row `(+$60 high byte) & 0x3F`: dx, dy, animation of the table 0x2E724, facing),
  act 4 only checks; the holder must be in mode 2 / sub 4 with frame class 0xC, else the release (floor snap,
  reaction 0x20, 0x2B54E). A knee gives act 0 or 2 (the reaction byte of the attack record), a throw sub 2 act 8
  (the player port). Pass 0x1185A (`pass_1185a`): a thrown or fallen body (list 0x6B30) hurts the other enemies
  (frame class 0 / 2 / 4, attack record 0x11, state 0x02020400, spark 0). The grabbed animation tables
  0x30B64 / 0x30BB0 / 0x30BFC are registered (`OBJECT_ANIM_TABLES`). Checked with the `grabs`, `grabs2` and `grabs3`
  scenarios of all four characters (README).

- **Pool A scenery and opening scene (phase 1 task 4a)**, `game/enemy/scenery.rs`: class 49 (0x56A70) background bird controller,
  class 50 (0x56C80) flying scenery (variants 0..0xC), class 54 (0x576D2) opening scene bystander, class 55 (0x5785C) scripted
  thugs (act list `+$06` 0..0x12 driven by the master prop's `+$05`; sub 2 / 6 run the shared hit reaction / grab). Pool B class 73
  (0x89ABA) bird spawner in `prop.rs`.
- **Class 10 (0x3B828, phase 1 task 4a)**, `game/enemy/class10{,_ai,_moves,_hold,_lib}.rs` and `claim.rs`: HAMMER T. / BLK ELMER /
  WRENCH T. complete (see `plan/phase-01-stage1-complete.md` task 4a). `claim.rs` = 0x6606 (a downed player lets class 10 thugs claim
  him through the lists 0x6A5A / 0x6A8E, which `enemies_update` clears like 0xE948). `enemy_hit_held_player` = 0x125C0.
  `enemy_recover_other` knows the recover entries of class 10 (0x2B56C: 0xC, 0x28, 0x2C -> 0x3BC38). New shared pieces: the repeat count
  0x32540 (`enemy_rep_count`), the sound 0xA062 (`enemy_snd_a062`, bit 3 of G.$43B), 0x32566 / class 10 tables in `dino-extract`.

- **Class 20 (0x457B0, phase 1 task 4a)**, `game/enemy/class20{,_ai,_attack,_lib}.rs`: PUNK / THUG complete (state 0xA is never
  entered in the listing). Recover entries 0x20 / 0x24 (0x2B56C) -> `c20_reset_ai` (0x45D74).

- **Class 14 (0x4020E, phase 1 task 4a2)**, `game/enemy/class14{,_ai,_states,_entry,_parts}.rs`, `class15.rs`, `lib32.rs`: BLADE / RAZOR complete (three
  objects per record, seven entrances, 24 reachable states, decisions with the ROM tables, guard 0x404CA wired into the player's hit pass, death, drops
  through the shared code), its parts class 60 (0x580C6), class 34 (0x5129E, dropped knife) and class 41 (0x55866, thrown blade), and class 15 (0x41F1A,
  the flying pieces of the prop B#40; `enemy_spawn_piece` is the spawner 0x41EEA for the pool B port). Classes 16 / 17 (0x420EA / 0x423A4, the truck of stage 2)
  are not ported. `lib32.rs` is the shared library 0x32xxx (aim, face, slots, screen test, distance class) and the sounds 0x9D44 / 0x9732 / 0x96CE.
  `enemy_combat.rs`: pass 0x11962 (`player_hits_item`: player attacks against the attacker list 0x671A, how a punch breaks a container).
  Details: `SPEC.md` section 17.5. Scenarios `c14a` .. `c14h` (+ the three other characters) and `c15a` (`re/harness/README.md`).

## Stubbed (TODO markers in the code)
- Pool A classes other than 2, 10, 14, 15, 20, 34, 41, 49, 50, 54, 55, 60 are freed at once (`enemy_class_unported`): next is 18 (Vice T.), then 56 and
  57 (opening scene), and 16 / 17 (the truck of stage 2).
  `enemy_recover_other` (0x2B56C for other classes) only resets to state 0.
- Pool B classes other than 15 and 73 are freed at once (props, scenery, B#12/#14/#20/#31/#40/#66/#69/#73/#74).
- **item.rs**: done in phase 1 task 2 (see `plan/phase-01-stage1-complete.md`); only class 6 (0x29B06, glow of weapon type 11), the RADIO event 0x8BD24 and the dust trail 0x92E5E are missing.
- `enemy_hud_hit` (0x16B36) is ported (HUD ring message, phase 0; `enemy_hud_request` is the call for the player port). `enemy_grabbed` is ported (phase 1 task 3b, see Ported).
- Floor collision: done in phase 1 task 1 (`game/terrain.rs`: 0x12CB4 with floor lookup, walls, street edges and the obstacle test; `enemy_floor_collision` / `enemy_snap_to_floor`).
- Sounds 0x9796 / 0xA354 / 0x97D2 are issued (`snd.rs`, phase 0); effects 0x606E, 0x92E5E, 0x94338, 0x2AD98 not called.
- 0x1D95E / 0x1DAC4 (scripted walk to the next area / boss): TEMP fallback in prop.rs gives the camera
  go flag while `lock78 == 1` after a wave ended with type 2/4 (players must walk right themselves).
- `rank_tick` runs game start + stage start + time init itself on its first call (TEMP until flow calls them).
- `spawn_script_tick` notices an area change itself (TEMP until flow calls `spawn_area_reset_objects`,
  `rank_area_start`, `rank_time_area_start`, `spawn_reset` from `hook_acd0`).

## Verified
- `cargo build` and `cargo test -p dino-game` pass (14 tests, incl. angle, window, BCD tests).
- `--input "0-2000:R" --frames 1500` -> `re/out/shots/e_wave1_1500.png`: a FERRIS thug spawned by the
  stage 1 list and approaching Jack (spawn interpreter + class 2 AI + draw working).

## Next steps, in order
1. Build and check: `cargo build`, then `cargo run -p dino-game -- --rom ../dino.zip --chars 0 --screenshot
   ../re/out/shots/e_wave1.png --frames 600` (and 900, 1200); look for thugs entering at camera X 0x388.
   If borrow-check errors appear in my files (never reached so far because other files did not compile), fix them.
2. Append the hit animation tables to `OBJECT_ANIM_TABLES` (sprites.rs, allowed edit): 0x2E7BC (Ferris),
   0x2E928 (Gneiss), 0x2EA94 (Driver) and the other entries of 0x2E68C / 0x2E724 that stage 1 uses.
3. (done, phase 1 task 2) item.rs: pool E, all classes but 6, drop rolls, hit passes in `item/hits.rs`; the pickup interface for
   the player port is documented at the top of item.rs.
4. (done) Container hits: passes 0x11F86 / 0x1201C / 0x120D2 in `item/hits.rs`.
5. Uncomment `player_hit_by_enemy(p, e)` in `pass_115b0` once the player port has it.
6. Stage 1 classes: (#10, #14 and #20 done) #18 VICE T. (0x42590, notes in SPEC section 12) and pool B class 22 (0x7BEF6); then
   Shivat (#24, 0x4838C). Each needs its anim tables added to OBJECT_ANIM_TABLES.
7. Stage 1 pool B props (B#12/#14/#20/#31/#40/#66/#69/#73/#74): read their handlers (table 0x2362E).
8. Ask the player port for a shared floor collision (they have `floor_lookup` / floor segments).

## Known issues / notes
- `rank.dip` is assumed 3 (normal); the real source of G.$777B is unknown.
- `pick_target` uses player +$A0 as (held << 8 | previous input), which is what the original reads.
- The original adds a hit spark via `spawn_hit_spark` (effects port) in 0x11008; it is called.
- Anti-tamper checks (0x2A5B6, 0x29ADE) are not ported on purpose (always pass with a genuine ROM).

## How to build a scenery prop with tile_block (T-B0, M2-05 and M2-06)

The shared service for the pool B tile props (door smashes, breaking walls, tile animations, one shot tile changes).
Read the module doc of `game/prop/tile_block.rs` first (it has the three code shapes with the addresses of classes 11,
12 and 40) and the top of `game/prop.rs` (the prop contract and the hit pass).

What exists:

| need | use |
| --- | --- |
| write tiles | `TilePos::at(0x90F288)` (an address of the original code) or `TilePos::new(Layer::Scroll2, col, row)` (ring, wraps at 64) or `TilePos::scroll2_at_world(x, y)` (0x75208); `prop_tile_put`, `prop_tile_code`, `prop_tile_attr(keep, or)`, `prop_tile_block(at, cols, rows, rom_addr, TileMode)`, `prop_tile_attr_block` |
| animation | `BlockTable { table, count, at, cols, rows, mode, period, looped }`, `start(p, delay)` in mode 0, `prop_block_table_step(p, &table)` every frame (state in `+$A0` timer and `+$A2` index, as class 12 has it) |
| a prop that can be hit | `p.hit_init(box_table, hp)` in mode 0, `prop_register_hit(handle)` every idle frame (0x4BFA), the break in mode 4. The pass `pass_10cd0` does the rest |
| break sound | `snd_break(x)` (0x9E60: 0xF6 / 0x6F by bit 0 of G.$43B, pan from X); the request with d1 and d2 is in `snd_take_requests()` for the sound port |
| script action spawns a prop | `spawn_script_prop(0x40 / 0x46 / 0x24)` or the generic `spawn_prop(class, var, Some((x, y, z)))` |

Steps for a new prop class:
1. Extract its data: add the tile code tables and the box table (12 bytes a box) to `TABLES` in `dino-extract/src/enemy.rs`
   (exact copies, no bytes in source). The writer panics when a table byte was not extracted, so a wrong address shows up
   in the first test run.
2. New file `game/prop/bNN.rs`, declared in `prop.rs` with `#[path]`, one dispatch line in `pool_b_update`, the class number in
   `PORTED_CLASSES` ("prop") of `trace.rs`. Name the original address of every routine in its doc comment.
3. Test with synthetic tables (`AssetPack::default()`, `enemy.t.blobs`), as `tile_block.rs` and `prop.rs` do.

Things found while building it:
- A block that is taller than the rest of its page runs on into the next column, as the 68000 address does. `down(n)` keeps that.
- The hit box bank (`assets.sprites.hitboxes`) holds 0x100000 to 0x11D220 only, but the prop box tables are in program ROM
  (0x8380A, 0x750B4, 0x76666 ...). The hit pass reads the prop box from `enemy.t` (`prop_body_box`) and has its own copy of the
  three range tests of 0x128F8 (`collide::overlap` is private). If the main session makes `HitBoxBank` cover those tables,
  `prop_box_hit` becomes `collide::boxes_hit`.
- The harness holds unported props as MAME had them but never registers them in the hit list, and its `Prop::trace_visit` does not
  load `+$44`, `+$48` or `+$6C` into the `Body`. So an unported breakable prop cannot be hit in a harness run until its class is ported
  (the `armed` scenario shows this at frame 564, the jump kick on B40).
- D3e part 1 section 1.4 has the scroll3 row formula wrong: the high row bits are `(i >> 9) & 7`, not `(i >> 10) & 3`
  (`Vram::index3` is right).
- The hero's attack box is not always 8 deep: the jab and kick boxes are about 8 deep, the others differ per frame. The rule
  "Z + 8 >= start and Z <= end" of D3e part 3 section 2.6 is the summary for the jab; the pass runs the real box test.

## M6-09 and M6-10: the Butcher (classes 5, 51, 52, 53, 58, 61)

Code: `game/enemy/class5.rs` (0x38DDE the starter), `class51_58.rs` (0x5717E carcass, 0x57630 machete helper, 0x57E5E knife holder),
`class52.rs` (0x57298 the knife), `class61.rs` (0x58498: init, mode dispatch, the alive handler, the checks and decisions, hit, death, knife drop and
follow), `class61_states.rs` (the state bodies, table 0x588E0) and `class61_moves.rs` (the move entry blocks 0x5988C .. 0x5A186). The class 5 and 61 entries
of the placeholder registry are gone (`--list-placeholders`). Check: `re/harness/m610_check.sh` (recordings of `re/harness/pokes/m610_butcher.txt`).
- **Result.** Nine scenarios of the Butcher alone (b1, b4, c1 to c7: heroes Jack, Hannah, Mess, Mustapha, standing, walking, fleeing, 4500 to 7000
  frames, one with his death) are equal to MAME in every frame (0 fields of any class, 0 globals), and so are `m516_butcher_*` and `m600_bs_b1_b10_nat2`
  up to the first Poacher. The mutation test (vx of the far slash + 1) shows the check sees a one byte difference. Left: the sound arguments (below).
- **The harness limit.** The Poachers of the boss controller (class 13, not run by the port) draw one random number each frame, before the Butcher in slot
  order, so the m600 scenarios stop being comparable at the first group (about 1100 frames after the start; `m600_bs_b1_b10_nat2` at 1585: MAME draws
  3 numbers in the frame, the port 1, and the Butcher's decision uses the second). The m610 scenarios hold the Butcher's `+$AA` at 0x1E (it asks the
  controller for a group by G.$502 each time it runs out), so no group comes. After M6-01 / M6-02 (Poachers) the m600 scenarios can be used again.
- **Frame facts (b10_nat2).** The stage record makes the object in frame 441, the init runs in 442, the chop wait is 180 updates (`+$A2` 0xB4 .. 0), the talk
  step (B50 var 1, sound 0x59 with the pan of X) in 623, the turn (carcass `+$A2` = 1: it slides right 4 px a frame until `0x173C` frees it) in 804, the
  camera mover B76 in 865, the change in 926. The class 61 init runs in the next frame (927), no frame in between.
- **Pointers** between objects are sign extended RAM addresses: `0xFFFF0000 | handle` (`class5::ptr`). The words `+$B0` / `+$B2` / `+$A6` / `+$A8` /
  `+$B4` / `+$B6` are plain handles. The boss controller's `+$AA` is `0xFFFF0000 | boss` too (`boss_spawn_controller` writes `0x00FF0000`: Vice and the
  Butcher overwrite it).
- **Knife holder A58** draws and moves only while the Butcher's `+$AC` is not 0 (the talk step); its offsets are rows of 8 bytes by the Butcher's frame byte
  `+$2C` (tables 0x57F0E / 0x57F26). In the class 61 init the knife holders are freed by their own mode 4 in frame 927 and the two A52 knives take their
  slots (free list order: the holder that frees last gives its slot to the first knife).
- **The decision 0x58D40** is the init's tail and the get-up entry of the table 0x2B56C (name 0x3C; `enemy_recover_other` calls `c61_decide`).
  The roll tables are 0x73ED4 / 0x73FD4 / 0x740D4 (no / one / two knives, 32 rows of 8 by the X class): 1 to 4 a combo, 5 the spin, 6 / 7 the leap,
  8 the first leap, 9 the dash (the word after the table 0x5A26C). With two knives and class 0 the rows 0 to 15 are the spin.
- **State machine.** Acts reached in play: 0 (slash), 2 and 0x1A (leaps), 4 / 0x34 (wait), 6 (stand), 0xA (landing), 0xC (spin), 0x10 (re-arm), 0x12 (dash),
  0x18 (walk), 0x1C (pause), 0x1E / 0x20 (far slash), 0x2A (crouch), mode 4 (death) and mode 6. Dead code kept (never reached, only read): 0xE, 0x14 / 0x16, 0x22, 0x24, 0x26,
  0x28 with sub 4, 0x2C, 0x2E (a chain that starts in the unreferenced block 0x59D12: the Butcher never grabs a hero) and 0x30 / 0x32 (their table words point at the table
  itself; 0x5A090 is reached only through `+$CA`, which nothing sets). They are ported from the listing and not exercised.
- **Register leftovers.** 0x32AE4 (`c61_598f2`) reads `move.b $B0,d1; tst.w d1`: the port takes the upper byte as 0. `0x5A1AC` compares with the word d1 of
  `move.b table,d1`: the upper byte is taken as 0. Both ran without a difference in all recordings.
- **Globals.** G.-$704E and G.-$7052 (`World::leap_dx`, `leap_dz`, traced) keep a quarter of the distance at the start of a leap for the apex (0x592EE).
- **Death.** Mode 4: the music fade, G.$4D6 = 1, `+$80` = 0x78, the kill score; sub 2 counts 120 frames, then 0x6148 (`enemy_boss_leader`: the leader flags `+$C0`) and
  mode 6. Mode 6 only draws: the body is never freed by itself (D3c 3.6 was wrong).
- **Known differences.** (1) The sounds 0x21 (d1 0x12) and 0x0F (d2 0x10000) of the inits carry register leftovers; the port sends 0 / 0. (2) Sound arguments of the
  attack object hit sounds (0x62, 0x39, 0x3E) are the shared pass of M5-02, not the Butcher. (3) `m610_c5_jump` frame 1166: two HUD cells of scroll1 (a score digit of
  Mustapha) for 5 frames: a frame cut in the HUD draw of a busy frame, no object field differs. (4) `hud.rs`: the boss bar message word 4 is 0xFFFF
  (`move.l a3,(a4)` with a sign extended a3), it was 0x00FF.
- **For M6-11 / M6-12.** Class 52 is complete here (init, held, the drop 0x573DE / 0x5743C / 0x57368, the item class 0 var 0x12 made at the landing, the follow tail 0x5A280);
  what is left of M6-11 is the check of the knife item in play (pick up by heroes, D2c 5.3) and the helper groups (the Poachers of the controller). The target rule 0x58CD6 is in
  `c61_pick_target`. M6-12 has the acceptance runs (and the death scores, which are already in `c61_dying`).


## M6-01 and M6-02: the Poachers (class 13: POACHER J., SKINNER, GUTTER)

Code: `game/enemy/class13.rs` (init 0x3DDFE, alive frame 0x3DF68, AI frame 0x3E0B8, flee timer 0x3E1FA, slot refresh 0x3E4A2, attack start tests 0x3E420, the
guard 0x3DFC6, the ten entrances 0x3E504 .. 0x4001C, death 0x401CE), `class13_ai.rs` (the state handlers of the table 0x3E1AA, the decisions 0x4005C and
0x40118 / 0x40156 with the state entries), `class13_attack.rs` (dash, rifle volley with the bullet 0x57F62 / flash 0x3EA8A, melee chain, kick, hop back),
`class13_tests.rs` (formula tests with made-up tables). Data: 25 tables in `dino-extract/src/enemy.rs` (`TABLES` "class 13", the fade palettes 0x3F008 as extra blobs)
and nine animation tables in `OBJECT_ANIM_TABLES` (0x6C57A, 0x6CA24, 0x6CF3E, hit 0x2EFEC / 0x2F176 / 0x2F2F6, grabbed 0x30D2C / 0x30D84 / 0x30DDC). Wiring: the dispatch line
and the recover entry (0x2B56C: +$84 = 8 / 0x48 / 0x4C -> 0x3E504) in `enemy.rs`, `trace.rs` PORTED_CLASSES, the guard call in `player_combat.rs` (next to the class 14
guard), `lib_slot6_keep` (0x32C54) in `lib32.rs`. The class 13 entry of the placeholder registry is gone (`--list-placeholders`).
- **Check.** `re/harness/m601_check.sh` (step mode; `MODE=run` for a free run). The D3a H5 to H10 recordings, the two batches of this task (`re/harness/pokes/m601_poachers.txt`:
  barrel, pail, far walkers, walk away; `m601_far.txt`: twelve segments with four far poachers each) and the Butcher scenarios. Record the two batches again with
  `PR_SNAPS=none PR_SHEET=0 re/harness/probe/probe_record.sh re/harness/pokes/m601_poachers.txt all` (and `m601_far.txt`; about 25 s each, one MAME process).
- **Result (step mode, ported class 0x0d fields).** H9 (nine entrances, 360 frames each) equal, also in free run mode (the random numbers too); H6 (level 0 and 31) and H7 (five spawn
  frames) equal for 900 frames (shot at 192 / 243 / 255 / 261 / 321 as D3a says, the sound 0xEA with d1 / d2); H8 (melee, three variants, two levels, 1500 frames) equal,
  except `v0_l31` after frame 1312 (a Shivat, class 24, not ported, draws random numbers before the poachers of the slot order); H10 (kill scores 2,100 / 3,100 / 4,100) equal; barrel and pail (state
  0x48), 12 far segments (state 0xE and the decisions), walk away equal up to the unported classes. The Butcher scenarios `m600_bs_b1_b1_nat` (6500 frames) and `m600_bs_b1_b10_nat2`
  (2200 frames: the two Gutters of the first group, entrance 0xC) and `m600_bs_b2_b4_face` are equal in every object field and global in step mode AND in a free run (the random numbers too).
  The first difference of `m600_bs_b1_b10_nat2` is now frame 252 and is not an object: the scroll1 text cells of the talk step and the sound arguments (known, Butcher notes).
- **Frame facts and traps.**
  - D3a 4.1 / 3.4 said "no guard": wrong. The init sets `+$87` = 1 and the player's hit pass calls 0x3DFC6 through the table 0x10B50 (entries 8, 0x48, 0x4C). The chance table 0x741D4 of the
    ROM is all 0, so it never blocks, but it **draws one random number and throws it away** (`moveq #0,d0` between the draw and the compare) for every hit on a free poacher on the
    ground that faces the target. Without the call the random sequence is wrong from the first hit (mutation test on `m600_bs_b1_b1_nat`: frame 1109). The arenas of the D3a recordings (`jump 2 0 0x400`) have the
    floor at height 0x40, so `+$0C` is not 0 there and the guard returns at its first test; the Butcher arena and the stages have floor 0. The block path (`+$BC`, hop back 0x3FBE4, state 0x28)
    is ported but never runs on a genuine ROM.
  - Mode 4 (0x401CE): sub 0 writes `+$80` = 0x3C first (the table 0x401DA has two words only; the next instructions are code, D3a read them as table entries), then the kill score; sub 2 counts `+$80`
    down and draws on EVEN counts.
  - The muzzle flash (class 57) takes `+$24` = the NEGATED facing (`neg.b`): 0xFF for a right facing poacher; for a left facing one the X goes back by 0x56 (net - 0x2B). The bullet is the shared
    `enemy_spawn_bullet` (0x57F62: dx 0x30, dy 0x40, variant 0). The shot sound 0x9C76 is the pool A one (code 0xEA, d1 = (3 / 4) << 16 | 0x2000 | pan, d2 = 0x15000000).
  - The far gate (0x40118) reads the X class `+$A5` of the PREVIOUS decision (not a fresh one), by `add.b` (the carry is lost); the roll `rand & 0x1F` must be below the table byte (signed).
    0x4005C stores a fresh class; 0x40156 (after the gate, also the end of the close in state 0x16) too.
  - Entrance 0x12 reads `sub.w $744(a5),d1` on the FRACTION of X (word access to a long: the low word of d1), not on the integer part; the scripts spawn on whole pixels, so the quirk
    is kept as it reads (`enemy_c13_3fe60`). Entrances 0x10 / 0x12 start with Z = 0 and a Z speed from the distance (z 0 at spawn, D3a).
  - `0x3E3D4` (the jump of a stuck or airborne poacher) and the ledge entry `0x3E38C` end the AI frame without drawing (the original `jmp 0x120E` returns to the caller).
  - The state handler for state 0 and 0x22 / 0x2E point into the table itself (never entered); the dead routines are listed in the module doc of `class13.rs`.
  - `+$A4` is written 0 and never read; `+$BD` is the slot number of `lib32.rs`; the pause table 0x7435A is indexed by the level plus the variant byte asr 5 (a byte add).
- **Not checked against MAME (no recording reaches it, ported from the listing):** state 0x12 (the jump) and its two entries 0x3E38C / 0x3E3D4 (a ledge or an airborne poacher: seen only in
  recordings of other classes, Episode 8), the hop back 0x28 (never runs on the ROM). Everything else of the live code is checked by at least one recording equal to MAME.
- **Harness.** `spawn_pokes.py` turns the `spawn` notes of a probe sidecar into RAM writes so that the INIT frame of a spawned poacher runs in the port (without it step mode loads the
  object after its init). A poke cannot set the free list: `global.free_count` / `free_ptr` differ at the spawn frame in step mode (and for the whole run in free run mode), and a spawn in
  another pool than A is not replayed (the item containers of `m601_barrel`). Both are artifacts of the replay, not of the class.
- **Known differences left.** (1) The scroll1 cells of the hero's HP bar at the frame a bullet or a blow hits him (3 cells at H6 frame 255 and H5 frame 36, the same in `c14a`): the HUD, not the
  poacher (object fields and sounds are equal). (2) Recordings with a Hopper, Mack, a barrel or the Cadillac (m600_h14 / h15 / h16, ep3_*, m800_*, m700_*) show differences of the poacher only where
  an unported class hits it or draws random numbers before it in the slot order.
