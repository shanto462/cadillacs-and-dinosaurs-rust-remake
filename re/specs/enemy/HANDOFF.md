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

## Stubbed (TODO markers in the code)
- Pool A classes other than 2 are freed at once (`enemy_class_unported`); scenery 49/50 freed.
  `enemy_recover_other` (0x2B56C for other classes) only resets to state 0.
- Pool B classes other than 15 are freed at once (props, scenery, B#12/#14/#20/#31/#40/#66/#69/#73/#74).
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
6. Stage 1 classes: #10 HAMMER/ELMER/WRENCH (0x3B828), #14 BLADE/RAZOR (0x4020E), #20 PUNK/THUG
   (0x457B0), #18 VICE T. (0x42590, notes in SPEC section 12) and pool B class 22 (0x7BEF6); then
   Shivat (#24, 0x4838C). Each needs its anim tables added to OBJECT_ANIM_TABLES.
7. Stage 1 pool B props (B#12/#14/#20/#31/#40/#66/#69/#73/#74): read their handlers (table 0x2362E).
8. Ask the player port for a shared floor collision (they have `floor_lookup` / floor segments).

## Known issues / notes
- `rank.dip` is assumed 3 (normal); the real source of G.$777B is unknown.
- `pick_target` uses player +$A0 as (held << 8 | previous input), which is what the original reads.
- The original adds a hit spark via `spawn_hit_spark` (effects port) in 0x11008; it is called.
- Anti-tamper checks (0x2A5B6, 0x29ADE) are not ported on purpose (always pass with a genuine ROM).
