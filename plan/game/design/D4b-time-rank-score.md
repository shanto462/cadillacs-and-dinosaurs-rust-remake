# D4b: Time, Rank, and Score Design

Design document for the world systems covering stage time budgets, the clock display, TIME OVER, the dynamic rank system, score rules, extra lives, high scores, and EEPROM settings in Cadillacs and Dinosaurs (CPS1, Capcom 1993, set `dino`).

Ground truth sources:
- Disassembly and decompilation of original 68000 ROM code (`re/tools/d`, `re/tools/d2`, `re/ghidra/decomp.c`).
- Port implementations in [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs), [hud.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/hud.rs), [flow.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/flow.rs), [spawn.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/spawn.rs), [enemy_combat.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/enemy_combat.rs), [enemy.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/enemy.rs), [prop.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/prop.rs), and [item.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/item.rs).
- Topic specifications in [SPEC.md (enemy)](file:///Volumes/Shanto/Repos/Personal/Github/roms/re/specs/enemy/SPEC.md) (sections 6 and 8.4) and [SPEC.md (flow)](file:///Volumes/Shanto/Repos/Personal/Github/roms/re/specs/flow/SPEC.md).
- Prior design documents: [X-rank-crosscheck.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/X-rank-crosscheck.md), [D2b-player-state-rules.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D2b-player-state-rules.md), [D4c-camera-flow-transitions.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D4c-camera-flow-transitions.md), [D4d-front-end-and-story.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D4d-front-end-and-story.md), [D3c-bosses-1.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D3c-bosses-1.md), and [D4a-items-drops.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D4a-items-drops.md).
- Historical mechanics notes: [mechanics-seen.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/mechanics-seen.md) rows F01 to F04, P34, M19 to M22, N01, N03.
- Web references: [claude-doc-deep-dive-supplement.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/sources/claude-doc-deep-dive-supplement.md) ("Scoring, rank and difficulty").

---

## 1. Time

### 1.1 Time Budget of Every Area Across All 8 Episodes

The stage timer is stored in BCD minutes (`G.$4E8`, RAM `0xFF84E8`) and seconds (`G.$4E9`, RAM `0xFF84E9`). At area initialization, routine `0x1031A` (`rank_time_area_start`) loads the initial budget from table `0x1033E` using index `stage * 8 + area * 2` (R: 0x1031A, 0x1033E, [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs#L311-L318)).

Table `0x1033E` contains 64 bytes (8 stages with up to 4 areas each, 2 BCD bytes per entry):

| Episode (Stage index) | Area 0 | Area 1 | Area 2 | Area 3 | Notes |
|---|---|---|---|---|---|
| Episode 1 (Stage 0) | 01:20 (80 s) | 02:20 (140 s) | 01:20 (80 s) | 04:00 (dummy) | Area 0: City streets. Area 1: Inside building. Area 2: Alley/boss (R: 0x1033E). |
| Episode 2 (Stage 1) | 03:20 (200 s) | 03:20 (200 s) | 01:20 (80 s) | 04:00 (dummy) | Area 0: Swamp entrance. Area 1: Forest interior. Area 2: Boss clearing (R: 0x1033E). |
| Episode 3 (Stage 2) | 03:00 (180 s) | 04:00 (dummy) | 04:00 (dummy) | 04:00 (dummy) | Area 0: Highway road drive. One long area (R: 0x1033E). |
| Episode 4 (Stage 3) | 01:20 (80 s) | 04:00 (240 s) | 02:50 (170 s) | 04:00 (dummy) | Area 0: Garage approach. Area 1: Garage workshop. Area 2: Boss arena (R: 0x1033E). |
| Episode 5 (Stage 4) | 03:50 (230 s) | 04:00 (240 s) | 04:00 (dummy) | 04:00 (dummy) | Area 0: Burning village. Area 1: Mountain path and boss arena (R: 0x1033E). |
| Episode 6 (Stage 5) | 02:50 (170 s) | 04:00 (240 s) | 01:20 (80 s) | 04:00 (dummy) | Area 0: Jungle river. Area 1: Mine tunnel. Area 2: Underground lair/boss (R: 0x1033E). |
| Episode 7 (Stage 6) | 04:00 (240 s) | 02:50 (170 s) | 03:30 (210 s) | 01:20 (80 s) | Area 0: Bridge. Area 1: Hallway/library. Area 2: Computer room. Area 3: Vault boss (R: 0x1033E). |
| Episode 8 (Stage 7) | 04:00 (240 s) | 04:00 (240 s) | 04:00 (240 s) | 04:00 (240 s) | Area 0: Lab tanks. Area 1: Reactor elevator. Area 2: Bio lab corridor. Area 3: Lair (R: 0x1033E). |

**Main session check (2026-10-13):** the times match the ROM table 0x1033E byte for byte. The area names in the Notes column are guesses by the writing agent and several are wrong (Episode 1 area 0 is the rooftop, Episode 5 area 0 the road and mountain, Episode 7 area 0 the evening rooftop); use `plan/game/episode-N.md` and `D4c` for what each area is.

### 1.2 Spawn Script Time Events: Set and Add

Spawn scripts and wave scripts can override or top up the clock using two commands (R: `re/specs/enemy/stage_scripts.txt`, [spawn.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/spawn.rs#L254-L255)):
- Command `0x0C` (`0xFD9C`): `rank_set_time(arg6)`. Directly writes the BCD word to `G.$4E8`.
- Command `0x10` (`0xFDA4` / `0x104A8`): `rank_add_time(arg6)`. Adds BCD time with a maximum cap of `04:00`.

Decoded script time events per episode:
- Episode 1:
  - Boss wave 5 (`0xADFDE`, Vice T.): when living enemy count falls to 0 (`alive <= [0, 0, 0]`), command `0x0C` sets time to `03:50` (R: `stage_scripts.txt` line 104).
- Episode 2:
  - Area 0 List 2 trigger `0x0150` (`0xAE062`): command `0x0C` sets time to `02:50` (R: `stage_scripts.txt` line 117).
  - Area 0 List 2 trigger `0x0170` (`0xAE092`): command `0x10` adds `+0:30` (R: `stage_scripts.txt` line 120).
  - Boss wave 6 (`0xAE570`, Butcher): when living enemy count falls to 0, command `0x0C` sets time to `03:50` (R: `stage_scripts.txt` line 223).
- Episode 3:
  - Area 0 List 2 trigger `0x2D00` (`0xAECF4`): command `0x0C` sets time to `03:50` before Hogg appears (R: `stage_scripts.txt` line 353).
- Episode 4:
  - Area 1 wave 2 (`0xAF46A`): command `0x0C` sets time to `03:50` (R: `stage_scripts.txt` line 517).
  - Boss wave 5 (`0xAF584`, Slice): when living enemy count falls to 0, command `0x0C` sets time to `03:50` (R: `stage_scripts.txt` line 555).
- Episode 5:
  - Area 1 wave 2 (`0xAFBB6`): command `0x0C` sets time to `03:50` (R: `stage_scripts.txt` line 679).
  - Area 1 wave 5 (`0xAFE38`): command `0x0C` sets time to `03:50` (R: `stage_scripts.txt` line 723).
  - Area 1 wave 6 (`0xAFEE2`): command `0x0C` sets time to `03:50` (R: `stage_scripts.txt` line 741).
  - Boss wave 7 (`0xAFFFC`, Morgue): command `0x10` adds `+03:50` (capped at `04:00`) (R: `stage_scripts.txt` line 759).
- Episode 6:
  - Area 1 wave 0 (`0xB0406`): command `0x0C` sets time to `03:50` (R: `stage_scripts.txt` line 849).
  - Area 1 wave 5 (`0xB06A0`): command `0x0C` sets time to `02:50` (R: `stage_scripts.txt` line 889).
  - Boss wave 7 (`0xB07FE`, Tyrog): command `0x0C` sets time to `03:50` (R: `stage_scripts.txt` line 909).
- Episode 7:
  - Area 1 wave 1 (`0xB0E92`): command `0x0C` sets time to `03:50` (R: `stage_scripts.txt` line 1057).
  - Area 1 wave 4 (`0xB1136`): command `0x0C` sets time to `03:50` (R: `stage_scripts.txt` line 1101).
  - Area 1 wave 5 (`0xB11EA`): command `0x0C` sets time to `01:50` (R: `stage_scripts.txt` line 1110).
  - Area 1 wave 6 (`0xB126E`): command `0x0C` sets time to `01:50` (R: `stage_scripts.txt` line 1123).
  - Area 1 wave 7 (`0xB1332`): command `0x0C` sets time to `01:50` (R: `stage_scripts.txt` line 1133).
  - Area 2 wave 8 (`0xB13C6`): command `0x0C` sets time to `03:50` (R: `stage_scripts.txt` line 1143).
  - Area 2 wave 11 (`0xB14C0`): command `0x0C` sets time to `02:50` (R: `stage_scripts.txt` line 1170).
  - Boss wave 13 (`0xB167E`, Slisaurs): command `0x0C` sets time to `03:50` (R: `stage_scripts.txt` line 1190).
- Episode 8:
  - Boss wave 5 (`0xB1DFA`, Fessenden form 2): command `0x0C` sets time to `02:50` (R: `stage_scripts.txt` line 1400).

### 1.3 The 51-Frames-per-Second Divider

The hardware runs at 59.61 Hz. The stage time tick routine `0x1037E` (`rank_time_tick`) divides time by counting down a frame divider byte `G.$4EA` (RAM `0xFF84EA`) (R: 0x1037E, [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs#L327-L350)):
1. Initialized to `0x32` (50 decimal) at area start (R: 0x1031A).
2. Each frame during normal play, if `G.$4EA != 0`, `G.$4EA` decrements by 1.
3. When `G.$4EA == 0`:
   - Decrements BCD seconds `G.$4E9` by 1 using `sbcd` (R: 0x103C8).
   - If borrow occurs, seconds wrap to `0x59` and minutes `G.$4E8` decrement by 1 using `sbcd`.
   - Reloads `G.$4EA = 0x32` (50 decimal).
4. Period: counting 50 down to 0 takes exactly **51 game frames per clock second** (R: 0x103BA, [SPEC.md (enemy)](file:///Volumes/Shanto/Repos/Personal/Github/roms/re/specs/enemy/SPEC.md#L992-L996)).
5. Speed ratio: one clock second equals `51 / 59.61 = 0.8556` real seconds. The game timer runs approximately 17 percent faster than real wall-clock time.

### 1.4 Clock Display Rules: Pool B Class 31 and Pool D Slot 15

The on-screen countdown timer is an interactive prop object of pool B, class 31 (`0x1F`), handler at `0x81046` (R: 0x81046, [prop.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/prop.rs)):
- **Allocation**: allocated at area start from the pool B spawner. It allocates a pool D sprite entry via routine `0x48A8` and stores its pointer in `+$98(a6)` (R: 0x81062, 0x48A8).
- **Placement**: screen X = `0x00E8` (232), screen Y = `0x00B8` (184), screen Z = `0x0000`, priority = `0xD0` (top layer, centered near the top edge below the HUD) (R: 0x8106C-0x81078).
- **Digit Font**: digit tiles are looked up from word table `0x811CA` (tile words `0x4E18` to `0x4E21` corresponding to digits '0' through '9') (R: 0x81160-0x81180). Format shows 3 digits separated by colons: one digit for minutes, two digits for seconds (`M:SS`).
- **Visibility Trigger (3:00 or less)**:
  - If `G.$4E8 > 0x0300` (above 3:00 BCD), the clock is completely suppressed and does not draw (`cmpi.w #$300, $4e8; bgt $8114c`) (R: 0x810BA).
  - When `G.$4E8 <= 0x0300`:
    - On exact minute (`seconds == 0x00`) and half minute (`seconds == 0x30`): timer variable `+$A2(a6)` is loaded with `0x012C` (300 frames = exactly 5 seconds) (R: 0x810F2, 0x81116). The clock is visible for those 5 seconds.
    - In the final 15 seconds (`time <= 0x0015` BCD): `+$A2(a6)` is loaded with `0x0320` (800 frames), keeping the clock continuously visible until time runs out (R: 0x8110A-0x81116).
- **Color Change (Red at 1:00)**:
  - If `G.$4E8 <= 0x0100` (1:00 or less BCD), palette attribute byte `+$23(a6)` is set to `0x84` (red palette) (R: 0x81134-0x81142).
  - Above 1:00, palette attribute byte is `0x00` (standard white palette).
- **Beep Sound**:
  - Whenever the stage clock ticks down a second while the clock is actively visible (`+$A2(a6) != 0`), sound code `0x34` is queued via `0xA63C` with parameters `d0 = 0x34, d1 = 0x10, d2 = 0x00` (R: 0x810E2-0x810E8).
  - The beep sounds once per second while the clock is shown, and stays silent when the clock is hidden.

### 1.5 TIME OVER Sequence

Triggered by routine `0x103FE` (`rank_time_over`) when `G.$4E8 == 0`, `G.$4EA == 0`, and flags `G.$4D5`, `G.$4DD`, and `G.$4EB` are clear (R: 0x103FE, [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs#L353-L371)):
1. `G.$4EB` is set to 1 (locks further time-over triggers).
2. Spawns pool C class `0x70` effect objects via routine `0x96450` (R: 0x96450, 0x96488):
   - Variant 0 ("TIME"): spawns at X = `0x0188` (392), Y = `0x0080` (128) with velocity `vx = -0x2000` (-2 pixels/frame, flying in from the right).
   - Variant 1 ("OVER"): spawns at X = `0xFFF8` (-8), Y = `0x0080` (128) with velocity `vx = +0x2000` (+2 pixels/frame, flying in from the left).
   - Both words meet at screen center X = `0x00C0` (192) and remain stationary for 180 frames (`+$60 = 0xB4`) while flashing palette `0x82`.
3. Hero Deaths:
   - Routine `0x1046A` runs for all three player objects (P1 `0xFFB274`, P2 `0xFFB3F4`, P3 `0xFFB574`) (R: 0x10450-0x10464).
   - For any active player whose state mode byte `+$04` is less than 4 (alive and not already dying), vitality is set to 0 (`clr.w $6C(a0)`) (R: 0x1047C).
   - Invulnerable heroes, heroes mid-respawn, and heroes performing moves are all killed indiscriminately.
4. Top-up after death:
   - Dead players enter the normal death sequence (lose 1 life, rank drop).
   - If the player has spare lives (`+$A3 > 0`), the player respawns (with air strike, but without the join bazooka).
   - At respawn, routine `0x10482` tops up the stage clock by `+03:30` (`0x0330` BCD, capped at `04:00`) and adds `+600` frames (`0x0258`) to wave timer `G.$50C` (R: 0x10482, 0x104A8).
   - `G.$4EB` is cleared, enabling the timer to tick down again for the new life.
   - If no lives remain, the player proceeds to the continue countdown (R: [D2b-player-state-rules.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D2b-player-state-rules.md#L473)).

### 1.6 Time Added by Join and Continue

- **Join during play**:
  - Initiated when an absent player presses start or attack with credit available (R: `0x7072`, [D4d-front-end-and-story.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D4d-front-end-and-story.md#L576-L580)).
  - Routine `0x1049E` adds **+03:30** (`0x0330` BCD) to stage clock `G.$4E8` (capped at `04:00`).
  - Adds **+600 frames** (`0x0258`) to wave timer `G.$50C`.
  - Sets rank speed `+3` via `0x18410` (R: 0x18410).
- **Continue**:
  - Initiated when a dead player with 0 lives continues during play or on the continue screen (R: `0x73BE`, `0x10490`, [D4d-front-end-and-story.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D4d-front-end-and-story.md#L613-L615)).
  - Routine `0x10490` adds **+04:00** (`0x0400` BCD) to stage clock `G.$4E8` (capped at `04:00`).
  - Adds **+600 frames** (`0x0258`) to wave timer `G.$50C`.
  - Decreases rank speed by 2 via `0x18566` with `d0 = 0xFE` (R: 0x18566).

### 1.7 Car Mode Time (Episode 3)

In Episode 3 (Hell Road), Jack's Cadillac is driven during the highway section (pool B class 88, activated at `0x8C06E`) (R: 0x8C06E, [D3c-bosses-1.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D3c-bosses-1.md#L918-L921)):
- Activation sets car driving flag `G.$518 = 1` (RAM `0xFF8518`).
- **Clock Stop**: while `G.$518 == 1`, routine `0x1037E` returns immediately without decrementing `G.$4EA` or `G.$4E8` (`tst.b $518(a5); bne $103f8`) (R: 0x103AA). The stage clock is frozen during the car drive.
- **Hogg Boss Encounter**:
  - When Hogg appears, boss controller `0x819B8` sets flag `G.$7B4 = 2` (RAM `0xFF87B4`) (R: 0x819BE).
  - If the stage clock was running and reaches `0:00` while riding the car against Hogg (`G.$518 != 0` and `G.$7B4 != 0`), routine `0x103FE` intercepts the timeout:
    - Sets `G.$518 = 2` (car driving mode ends, heroes dismount or car is wrecked).
    - Calls `0x104A8` to add **+03:00** (`0x0300` BCD) to the clock (R: 0x10436-0x10440).
    - **No heroes are killed**; the fight transitions to on-foot combat against Hogg.

### 1.8 When the Clock Stops (Freeze Conditions)

The stage clock countdown routine `0x1037E` stops counting down whenever any of the following conditions hold (R: 0x1037E-0x103AE):
1. **Scripted Scenes (`G.$4DD != 0`)**: stage intros, dialog cutscenes, and scripted walk segments set flag `G.$4DD`.
2. **Boss Defeat / HUD Hold (`G.$4D5 != 0`)**: set by routine `0xACEE` when a boss is defeated; holds the clock frozen through the clear slow-motion and tally transition.
3. **Car Driving (`G.$518 != 0`)**: the Cadillac driving mode freezes the clock.
4. **GO Walk / Area Transition**: routine `0x1D95E` sets `G.$4DD = 1` during the automated player walk off-screen, pausing the clock until control is restored in the next area.
5. **Debug Pause / Cheats (`G.$4AA != 0`)**: active if DIP cheats pause bit 7 of `G.$23` or bit 1 of `G.$24` is held.
6. **Non-Gameplay Modes**: the game frame loop `0xAAF6` runs only in game mode `0x0C`. During attract (`mode 0`), character select (`mode 4`), story panels (`mode 8`), continue screen (`mode 0x10`), game over (`modes 0x14, 0x18`), and ending (`mode 0x1C`), the in-game clock does not tick.

---

## 2. Rank

### 2.1 Summary of X-Rank Cross-Check

The cross-check document [X-rank-crosscheck.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/X-rank-crosscheck.md) verified the internal difficulty system against fan claims (R: [X-rank-crosscheck.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/X-rank-crosscheck.md), [SPEC.md (enemy)](file:///Volumes/Shanto/Repos/Personal/Github/roms/re/specs/enemy/SPEC.md) section 6):
- **Normal Rank (`w.rank`, G.$5F0)**: range `0..31`. Primary difficulty variable.
- **Timer Rank (`rank.speed`, G.$5F6)**: range `0..15`. Determines how frequently normal rank increases.
- **Rank Timer (`rank.timer`, G.$5F8)**: reloads from table `0x187B8` according to `rank.speed`:
  `[600, 520, 480, 440, 400, 360, 300, 260, 240, 220, 200, 180, 160, 140, 120, 60]` frames (R: 0x187B8).
  At speed 0, rank rises every 10 seconds; at speed 15, rank rises every 1 second.
- **Factory Settings**: EEPROM byte `0x4B` (`G.$777B`). Bits 0..2 = difficulty 3; bits 3..4 = timer level 1. Initial rank speed = `1 + 1 = 2` (period 480 frames = 8 seconds).
- **Game Start Roll (0x17F8A)**: initial rank is rolled using difficulty setting `0..7` and title wait counter `G.$4F6` via table `0x180B4` and `0x180FA`. Default start is between 6 and 20.
- **Player Count Clamps (0x18592)**:
  - 1 Player: clamped at maximum 24.
  - 2 Players: clamped between 4 and 28.
  - 3 Players: if rank is below 8, clamped to 4; capped at 31 by `rank_tick`.
- **Stage Transitions (0x1800E)**: table `0x1823A` applies `(delta, max, min)`:
  Stage 1: `(0, 31, 0)`; Stage 2: `(+3, 23, 14)`; Stage 3: `(-2, 24, 16)`; Stage 4: `(+5, 25, 18)`; Stage 5: `(+1, 26, 20)`; Stage 6: `(+5, 27, 22)`; Stage 7: `(+4, 28, 24)`; Stage 8: `(+6, 31, 26)` (R: 0x1823A).
- **Rank Events**:
  - Player hit recovery (`0x183E4`): rank `-1`, with a 600-frame lockout in `G.$604`.
  - Player death/continue (`0x184E2`): subtracts difficulty word from `0x187E0` (`[7, 6, 5, 4, 3, 2, 1, 0]`; difficulty 3 subtracts 4). If within 10 seconds of previous death (`G.$606 != 0`), subtracts 3 instead. If no lives remain, subtracts 3 more (`0x184C4 == 0`). Sets 1200-frame calm period in `G.$608`.
  - Area change / sub-area change (`0xACD0`, `0xACE0`): rank speed `+2`.
  - Boss defeat (`0xACEE`): rank speed `+1`.
  - Player join (`0x18410`): rank speed `+3`.
  - Continue (`0x18566`): rank speed `-2`.

### 2.2 Missing Calls in the Port

As identified in [X-rank-crosscheck.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/X-rank-crosscheck.md), several calls in [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs) are implemented but not invoked by the rest of the game engine (R: [X-rank-crosscheck.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/X-rank-crosscheck.md#L31-L41)):
1. `G.$4F6`: title screen credit wait counter incremented by system task `0x1827A`.
2. Death hook: calling `rank_event_continue` (`0x184E2`) and `rank_event_1842c` (`0x1842C`) on hero death without lives.
3. Join hook: calling `rank_event_18410` (`0x18410`) on mid-game join.
4. Continue hook: calling `rank_speed_add` (`0x18566`) with `d0 = 0xFE` (-2) when continuing.
5. Extra life hook: calling `rank_event_1843c` (`0x1843C`) from score routine `0x6BDC`.
6. Pending rank modifications: writers for `G.$5FA` (pending ups) and `G.$5FC` (pending downs).
7. Rank freeze setter: setting `G.$602` during scripted transitions.
8. Area transition scripted flag: setting `G.$4DD` during automated walk `0x1D95E`.

### 2.3 Level Mapping: Table 0x1860C

Routine `0x18592` (`rank_compute_level`) calculates the effective game level `G.$5EE` (RAM `0xFF85EE`, range `0..31`) from rank `G.$5F0` and player count `G.$4EF` (0 = 1P, 1 = 2P, 2 = 3P) using word table `0x1860C` (R: 0x1860C, [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs#L113-L143)):

```
Offset = rank * 6 + player_column
where player_column is 0 for 1 player, 2 for 2 players, 4 for 3 players.
```

Table `0x1860C` mapped values (Level `G.$5EE` per Rank `0..31`):

| Rank | 1 Player (col 0) | 2 Players (col 2) | 3 Players (col 4) |
|---|---|---|---|
| 0 | 0 | 4 | 8 |
| 1 | 1 | 4 | 8 |
| 2 | 1 | 4 | 8 |
| 3 | 2 | 4 | 8 |
| 4 | 2 | 4 | 8 |
| 5 | 3 | 5 | 8 |
| 6 | 4 | 6 | 8 |
| 7 | 5 | 7 | 8 |
| 8 | 6 | 8 | 8 |
| 9 | 7 | 9 | 9 |
| 10 | 8 | 10 | 10 |
| 11 | 9 | 11 | 12 |
| 12 | 10 | 12 | 14 |
| 13 | 11 | 13 | 16 |
| 14 | 12 | 14 | 18 |
| 15 | 13 | 15 | 19 |
| 16 | 14 | 16 | 20 |
| 17 | 15 | 17 | 21 |
| 18 | 16 | 18 | 22 |
| 19 | 17 | 19 | 23 |
| 20 | 18 | 20 | 24 |
| 21 | 19 | 21 | 25 |
| 22 | 20 | 22 | 26 |
| 23 | 22 | 23 | 27 |
| 24 (1P Cap) | 24 | 24 | 28 |
| 25 | 24 | 25 | 28 |
| 26 | 24 | 26 | 29 |
| 27 | 24 | 27 | 29 |
| 28 (2P Cap) | 24 | 28 | 30 |
| 29 | 24 | 28 | 30 |
| 30 | 24 | 28 | 31 |
| 31 (3P Cap) | 25 | 29 | 31 |

Notice: 1 player rank is clamped to 24, giving a maximum level of 24. 2 players clamped to rank 28, giving level 28. 3 players can reach rank 31 and level 31.

### 2.4 Aggression Tables per Stage and Area

Aggression `G.$5EC` (RAM `0xFF85EC`, range `0..7`) governs enemy mood and attack frequency.

#### Initial Aggression per Area (Table 0x18094)
At area start, routine `0x18048` (`rank_area_start`) loads initial aggression from byte table `0x18094` using `stage * 4 + area`, subtracting 1 from the raw byte (R: 0x18094, [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs#L108-L110)):

| Episode | Area 0 | Area 1 | Area 2 | Area 3 |
|---|---|---|---|---|
| Episode 1 (Stage 0) | 0 | 0 | 5 | 0 |
| Episode 2 (Stage 1) | 4 | 6 | 4 | 0 |
| Episode 3 (Stage 2) | 3 | 0 | 0 | 0 |
| Episode 4 (Stage 3) | 5 | 3 | 4 | 0 |
| Episode 5 (Stage 4) | 6 | 4 | 0 | 0 |
| Episode 6 (Stage 5) | 4 | 2 | 5 | 0 |
| Episode 7 (Stage 6) | 4 | 6 | 1 | 5 |
| Episode 8 (Stage 7) | 3 | 1 | 7 | 6 |

#### Aggression Markov Transition Tables (0x182E4 and 0x18364)
Aggression re-rolls via routine `0x182A2` (`rank_reroll_aggression`) on every enemy kill (`0x2AFCA`), spawn command 4, and at the end of continue calm periods (`0x187B2`) (R: 0x182A2, [SPEC.md (enemy)](file:///Volumes/Shanto/Repos/Personal/Github/roms/re/specs/enemy/SPEC.md#L497-L512)). Table `0x182E4` is used normally; table `0x18364` is used when `G.$7B4 == 1`.

Each row has 16 entries indexed by `rng() & 0x0F`:

- Normal Table (`0x182E4`):
  - Current 0: `0` (6/16), `1` (10/16)
  - Current 1: `0` (2/16), `1` (4/16), `2` (10/16)
  - Current 2: `1` (2/16), `2` (4/16), `3` (10/16)
  - Current 3: `2` (2/16), `3` (4/16), `4` (10/16)
  - Current 4: `3` (2/16), `4` (4/16), `5` (10/16)
  - Current 5: `4` (2/16), `5` (6/16), `6` (8/16)
  - Current 6: `5` (2/16), `6` (6/16), `7` (8/16)
  - Current 7: `6` (2/16), `7` (14/16)
- Alternate Table (`0x18364`):
  - Even steeper upward drift: transitions to higher aggression with `11/16` probability from states 0..6.

Thus, aggression steadily drifts upwards to 7 as enemies are killed. Dying resets aggression to 0 for 1200 frames (20 seconds) of calm.

### 2.5 What Rank and Level Change in Gameplay

Rank and Level directly alter multiple gameplay mechanics (R: [SPEC.md (enemy)](file:///Volumes/Shanto/Repos/Personal/Github/roms/re/specs/enemy/SPEC.md) sections 6, 7, 10, 14):
1. **Normal Enemy HP**: routine `0x31424` sets enemy HP from table `0x31454 + name_id * 64 + level * 2` (R: 0x31424).
   - Ferris: 44 HP (level 0) up to 76 HP (level 31).
   - Gneiss: 44 HP (level 0) up to 100 HP (level 31).
   - Bludge: 76 HP (level 0) up to 200 HP (level 31).
   - Walther: 100 HP (level 0) up to 300 HP (level 31).
2. **Boss HP**: routine `0x31E14` sets boss HP using index `idx * 48 + (players - 1) * 16 + ((level & 0x1C) >> 1)` (R: 0x31E14).
   - 8 level bands per player count scale boss health bars. Vice T. scales from 350 to 600 HP (1P) and up to 750 HP (3P).
3. **Enemy Damage to Heroes**: routine `0x6422` scales base attack damage:
   `d = (base * B[0x2C3AE + atk.w84 * 8 + (level & 31)]) >> 5` (R: 0x6422).
   A factor of 32 represents 1.0x baseline. Level 31 raises damage scaling significantly (e.g. up to 1.7x).
4. **Enemy Attack Frequency and Spacing**:
   - Hover pause before attacking: table `0x34186` scales pause duration from 85 frames (level 0) down to 15 frames (level 31).
   - Attack trigger probability: e.g. Vice T attack chance per 60 frames rolls against `B[0x44B84 + variant * 16 + level] / 32` (R: 0x44B84).
   - Recovery pause after combo: table `0x436D4` / `0x4697E` indexed by `level & 0x18`.
   - Flinch counter-attack window: `+$7A` delay from table `0x2B8C4` by level.
5. **Level-Gated Enemy and Item Spawns**:
   - Spawn script record flags test level condition: flag bit 3 (`0x08`) requires `level >= L`, and flag bit 4 (`0x10`) requires `level < L` (R: [spawn.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/spawn.rs#L130-L135)).
   - Used for harder enemy reinforcements in Episode 4 area 2 (M-16A1 rifles), and food upgrades in Episode 7 area 4.

---

## 3. Score

### 3.1 Points per Action: Table 0x10292 and Callers

All score additions pass through BCD addition routine `0x10036` / `0xFFC8` (`hud_add_score` / `hud_add_score_to`), crediting the player specified in object byte `+$02` (0 = P1, 1 = P2, 2 = P3). Scores are stored as 4-byte BCD longwords in player field `+$100` (RAM `0xFFB374`, `0xFFB4F4`, `0xFFB674`), capped at `99,999,999` (R: 0x10036, 0xFFC8, [hud.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/hud.rs#L426-L440)).

**Scripted Lock**: score addition is completely blocked while scripted control flag `G.$4DE == 1` (R: 0xFFC8, [enemy_combat.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/enemy_combat.rs#L274)).

#### Table 0x10292 BCD Offset Master Table
Table `0x10292` holds 34 standard BCD longword point values (R: 0x10292):

| Offset | BCD Hex Value | Decimal Points | Primary Callers / Actions |
|---|---|---|---|
| `0x00` | `00000010` | 10 | Minor script event |
| `0x04` | `00000020` | 20 | Minor script event |
| `0x08` | `00000030` | 30 | Minor script event |
| `0x0C` | `00000040` | 40 | Minor script event |
| `0x10` | `00000050` | 50 | Minor script event |
| `0x14` | `00000100` | 100 | Jab, knee 1, jump attacks, dive, air strike hit, Hannah special |
| `0x18` | `00000200` | 200 | Punch 3, knee 2, dash start and hit 2 |
| `0x1C` | `00000300` | 300 | Multi-hit move components |
| `0x20` | `00000400` | 400 | Finisher, throws, specials of Jack, Mustapha, Mess |
| `0x24` | `00000500` | 500 | Container break bonus (0x29858) |
| `0x28` | `00000600` | 600 | Script bonus |
| `0x2C` | `00000700` | 700 | Command move main hit (down, up + attack) |
| `0x30` | `00000800` | 800 | Escape from enemy grab/hold |
| `0x34` | `00000900` | 900 | Script bonus |
| `0x38` | `00001000` | 1,000 | Tag team super run attack, barrel hit, basic enemy kill, tier 1 food/treasure |
| `0x3C` | `00002000` | 2,000 | Tier 2 enemy kill (Poacher J, Hammer, Blade, Thug) |
| `0x40` | `00003000` | 3,000 | Tier 3 enemy kill (Blk Elmer, Skinner, Razor), tier 2 treasure/food |
| `0x44` | `00004000` | 4,000 | Tier 4 enemy kill (Wrench, Gutter), food type 18 |
| `0x48` | `00005000` | 5,000 | Tier 3 food/treasure (Burger, Pizza, Golddust, Pearl) |
| `0x4C` | `00010000` | 10,000 | Boss kill: Vice T., tier 4 food/treasure (Barbecue, Steak, Diamond, Egg) |
| `0x50` | `00015000` | 15,000 | Bonus tier |
| `0x54` | `00006000` | 6,000 | Alternate enemy tier |
| `0x58` | `00007000` | 7,000 | Alternate enemy tier |
| `0x5C` | `00008000` | 8,000 | Walther, Bludge kill, Fessenden form 1 kill |
| `0x60` | `00009000` | 9,000 | Alternate enemy tier |
| `0x64` | `00020000` | 20,000 | Boss kill: Butcher, Morgan; Tally 100 HP vitality bonus, final blow bonus |
| `0x68` | `00030000` | 30,000 | Boss kill: Morgue |
| `0x6C` | `00040000` | 40,000 | Boss kill: Slice |
| `0x70` | `00050000` | 50,000 | Bonus tier |
| `0x74` | `00060000` | 60,000 | Boss kill: Tyrog (Stage 6) |
| `0x78` | `00070000` | 70,000 | Boss kill: Slisaur (each) |
| `0x7C` | `00080000` | 80,000 | Secret Wrench T. / Doug kill |
| `0x80` | `00090000` | 90,000 | Bonus tier |
| `0x84` | `00100000` | 100,000 | Boss kill: Tyrog alt (Stage 6 speed finish) |

#### Attack Record Score Codes
Every hero attack record contains a score offset byte (byte 5 of the record) which indexes table `0x10292` (R: 0xA26FC, [D2a-player-moves.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D2a-player-moves.md#L538)):
- `0x14` = 100 points: jab, knee 1, jump attacks, dive kick, running jump attack, dash hit 3 and later, Hannah's special.
- `0x18` = 200 points: punch 3, knee 2, dash start and dash hit 2.
- `0x20` = 400 points: punch finisher, normal shoulder throw / slam, specials of Jack, Mustapha, and Mess.
- `0x2C` = 700 points: main heavy hit of command move (down, up + attack).
- `0x30` = 800 points: grab escape hit.
- `0x38` = 1,000 points: Jack's tag team partner fling super run attack.

### 3.2 Kill Scores per Enemy Class and Boss

When an enemy reaches 0 HP and finishes its death sequence (approx. 79 frames after the fatal blow), routine `0x325E6` (`enemy_award_kill_score`) looks up the enemy's name ID (`+$03`, range `0x00..0x26`) in offset table `0x325F8` and awards points from table `0x10292` (R: 0x325E6, 0x325F8, [enemy.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/enemy.rs#L658-L663)).

Comparison between code table values and fan / supplement claims (R: 0x325F8, 0x10292, W: M20, [claude-doc-deep-dive-supplement.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/sources/claude-doc-deep-dive-supplement.md#L391-L442)):

| Name ID | Enemy Name | Code Offset | Code Points | Fan / Supplement Claim | Verdict / Explanation |
|---|---|---|---|---|---|
| `0x00` | FERRIS | `0x38` | 1,000 | 1,000 | CONFIRMED. Basic thug. |
| `0x01` | GNEISS | `0x3C` | 2,000 | 2,000 | CONFIRMED. |
| `0x02` | POACHER J | `0x3C` | 2,000 | 2,000 | CONFIRMED. Rifle poacher. |
| `0x03` | HAMMER T. | `0x3C` | 2,000 | 2,000 | CONFIRMED. |
| `0x04` | R HOPPER | `0x38` | 1,000 | 1,000 | CONFIRMED. Small raptor. |
| `0x05` | SLICE | `0x6C` | 40,000 | 40,000 | CONFIRMED. Stage 4 boss. |
| `0x06` | BLADE | `0x3C` | 2,000 | 2,000 | CONFIRMED. Knife thrower. |
| `0x07` | SHIVAT | `0x38` | 1,000 | 1,000 | CONFIRMED. Note: Shivat usually retreats (MANZOKU) without dying. |
| `0x08` | PUNK | `0x38` | 1,000 | 1,000 | CONFIRMED. Fast thug. |
| `0x09` | THUG | `0x3C` | 2,000 | 2,000 | CONFIRMED. |
| `0x0A` | BLK ELMER | `0x40` | 3,000 | 3,000 | CONFIRMED. |
| `0x0B` | WRENCH T. | `0x44` | 4,000 | 4,000 | CONFIRMED. |
| `0x0C` | VICE T. | `0x4C` | 10,000 | 10,000 | CONFIRMED. Stage 1 boss. |
| `0x0D` | WALTHER | `0x5C` | 8,000 | 8,000 | CONFIRMED. Gunman mid-boss. |
| `0x0E` | LASH T. | `0x38` | 1,000 | 1,000 | CONFIRMED. Mace thug. |
| `0x0F` | BUTCHER | `0x64` | 20,000 | 20,000 | CONFIRMED. Stage 2 boss. |
| `0x10` | BLUDGE | `0x5C` | 8,000 | 8,000 | CONFIRMED. Mutant hybrid. |
| `0x11` | MACK | `0x38` | 1,000 | 1,000 | CONFIRMED. |
| `0x12` | SKINNER | `0x40` | 3,000 | 3,000 | CONFIRMED. |
| `0x13` | GUTTER | `0x44` | 4,000 | 4,000 | CONFIRMED. |
| `0x14` | RAZOR | `0x40` | 3,000 | 3,000 | CONFIRMED. |
| `0x15` | MORGAN | `0x64` | 20,000 | 0 | DIFFERENT. Table awards 20,000, but in play Morgan transforms into Morgue rather than executing normal death. |
| `0x16` | MORGUE | `0x68` | 30,000 | 30,000 | CONFIRMED. Stage 5 boss. |
| `0x17` | KNIFE | `0x38` | 1,000 | 1,000 | CONFIRMED. |
| `0x18` | SLISAUR | `0x78` | 70,000 | 70,000 | CONFIRMED. Stage 7 boss (each). |
| `0x19` | TYROG | `0x74` | 60,000 | 60,000 | CONFIRMED. Stage 6 boss. |
| `0x1A` | DRIVER | `0x38` | 1,000 | 1,000 | CONFIRMED. |
| `0x1B` | TYROG (Alt) | `0x84` | 100,000 | 100,000 | CONFIRMED. Fast defeat / alternate Tyrog form gives 100,000. |
| `0x1C` | BLADE (Alt) | `0x38` | 1,000 | 1,000 | CONFIRMED. Class 63 variant. |
| `0x1D` | PUNK (Alt) | `0x38` | 1,000 | 1,000 | CONFIRMED. |
| `0x1E` | HAMMER T. (Alt) | `0x38` | 1,000 | 1,000 | CONFIRMED. |
| `0x1F` | BLK ELMER (Alt) | `0x38` | 1,000 | 1,000 | CONFIRMED. |
| `0x20` | WRENCH T. (Alt) | `0x38` | 1,000 | 1,000 | CONFIRMED. |
| `0x21` | ZEKE | `0x38` | 1,000 | 1,000 | CONFIRMED. Pteranodon flyer. |
| `0x22` | FESSENDEN Form 2 | `0x38` | 1,000 | 1,000 | CONFIRMED. Final mutant boss. (Fan noted surprising low score of 1,000). |
| `0x23` | TYROG 2 | `0x38` | 1,000 | 1,000 | CONFIRMED. Stage 8 mid-boss. |
| `0x24` | WRENCH T. (Secret) | `0x7C` | 80,000 | 80,000 | CONFIRMED. Secret "Doug" placement with corrupt palette. |
| `0x25` | FESSENDEN Form 1 | `0x5C` | 8,000 | 8,000 (Japan) / 16,000 | PARTIAL. World ROM has 8,000. Fan claim of 16,000 is not in standard table 0x10292. |
| `0x26` | HOGG | `0x38` | 1,000 | 1,000 | CONFIRMED. Stage 3 boss on foot or ram. |

### 3.3 Container and Item Scores

#### Containers (Pool E Class 5)
- Breaking a container awards **+500 points** via routine `0x29858` (`item_score_500`, table offset `0x24`) to the breaker (`+$02`), plus the normal hit score (e.g. +100 for a punch) (R: 0x29858, [item.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/item.rs#L481)).
- Thrown containers (barrel, drum, boulder, stone) award **+1,000 points** (table offset `0x38`) for each enemy struck during flight (R: 0x120D2).

#### Food Items (Table 0x28B10)
When collected, food checks player vitality `+$6C` (R: 0x28AA4, [SPEC.md (enemy)](file:///Volumes/Shanto/Repos/Personal/Github/roms/re/specs/enemy/SPEC.md#L1419-L1432)):
- If injured (`HP < 100`): restores vitality up to 100 and awards **0 points**. The HUD displays `GOOD !!`.
- If at full vitality (`HP == 100`): awards score according to food tier and displays `<points> pts` in HUD:
  - **10,000 pts** (offset `0x4C`): Barbecue (100 HP heal), Steak (80 HP), Lobster (64 HP), Sushi (64 HP).
  - **5,000 pts** (offset `0x48`): Burger (48 HP), Hot Dog (48 HP), Pizza (48 HP), Salad (32 HP).
  - **3,000 pts** (offset `0x40`): Cake (32 HP), Paffe (32 HP), Pudding (32 HP), Potato (32 HP).
  - **1,000 pts** (offset `0x38`): Chewing Gum (16 HP), Chocolate (16 HP), Donut (16 HP), Coffee (16 HP), Croissant (16 HP).
  - **4,000 pts** (offset `0x44`): Food type 18 (36 HP).

#### Treasure Items (Table 0x2A164)
Treasures always award score points upon collection (awarded 11 frames after pickup press during the pickup animation) (R: 0x2A164, [D2b-player-state-rules.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D2b-player-state-rules.md#L452)):
- **10,000 pts** (offset `0x4C`): Gold Bar, Diamond, The Cup, Dinosaur Egg.
- **5,000 pts** (offset `0x48`): Golddust, Dinobone, Ammonite, Pearl.
- **3,000 pts** (offset `0x40`): Necklace, Ring, Watch, Radio (note: Episode 3 Radio triggers kill-all).
- **1,000 pts** (offset `0x38`): Dime, 1000$ Bill, Sunglasses, Glove.

#### Ammo Box (Item Class 0 Variant 0x0E)
- Refills the held matching firearm to maximum capacity.
- If the gun is already full, or if the player holds no gun, awards **+1,000 points** (offset `0x38`) (R: 0x24D6C, [SPEC.md (enemy)](file:///Volumes/Shanto/Repos/Personal/Github/roms/re/specs/enemy/SPEC.md#L1443)). (If picked up with no gun, the next gun collected is also pre-filled).

### 3.4 Clear Tally Scoring

At the conclusion of each episode, clear tally task `0xB97E` calculates bonus points for all participating players (R: 0xB97E, [D4d-front-end-and-story.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D4d-front-end-and-story.md#L741-L760)):
1. **Vitality Bonus**:
   - If player HP is 100 (full): flat **20,000 points**.
   - If player HP is less than 100: `(HP - 1) * 100` points (e.g. 99 HP gives 9,800 points; 1 HP gives 0 points).
2. **Enemies Defeated Bonus**:
   - `kills * 100` points, where `kills` is the player's enemy kill count stored in `+$10B`.
3. **Final Blow Bonus**:
   - **20,000 points**, awarded only to the specific player who struck the killing blow on the boss (flag `+$C0` set).
4. **M.V.P. Bonus**:
   - **10,000 points**, awarded to the player(s) who dealt the highest total damage (`+$10C`) during the episode. Ties share the award. In single player, always awarded.
5. **Count Down into Score**:
   - The total bonus counts down at 100 points per frame into the player's score via `hud_add_score_to`.
   - In multiplayer games, the player with the highest total bonus receives the red `WINNER!` label.

### 3.5 Extra Lives: Thresholds and 1UP Mechanics

Routine `0x6B98` evaluates extra life qualification whenever a player's score changes (R: 0x6B98, [D2b-player-state-rules.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D2b-player-state-rules.md#L593)):
- **Threshold List (Table 0x6BFE)**: indexed by EEPROM option byte `G.$4E5` (decoded from `G.$777B` bits 5..6):
  - Option 0 (offset `0x08`): `[99999999]` (No extra lives).
  - Option 1 (offset `0x0C`): `[300000, 700000, 99999999]` (At 300,000 and 700,000 points).
  - Option 2 (offset `0x18`, Factory Default): `[500000, 1000000, 99999999]` (At 500,000 and 1,000,000 points).
  - Option 3 (offset `0x24`): `[1000000, 99999999]` (At 1,000,000 points only).
- **Award Process**:
  1. Compares current score against the next threshold in table `0x6BFE`.
  2. When crossed:
     - Sets pending 1UP flag `+$10E = 1`.
     - Initializes 1UP pose window timer `+$11C = 0xF0` (240 frames).
     - Increments spare life count: `+$A3 += 1`.
     - Calls rank event `0x1843C` (rank adjust for extra life).
     - Plays extra life sound effect `0x4A` via routine `0xA4CA`.
     - Redraws the player's HUD block life digit tile (`0x162F8` for 3P, `0x16358` for 2P).
- **1UP Pose and Floating Text**:
  - If the player becomes idle within the 240-frame window (`+$11C`), the hero enters the unique 1UP victory pose:
    - Jack and Mess: ground state `0x18`.
    - Mustapha and Hannah: jumping air state `0x418` / `0x41A`.
  - The hero is invulnerable during the pose and blinks for 180 frames afterwards.
  - Spawns floating "1UP!" text graphic: pool C class 14 variant 2 (`0x93BF8`), which displays over the hero for 180 frames.

### 3.6 Score Display and High Score Table

- **In-Game Score Display**:
  - Routine `0x101B6` draws 8 BCD digits right-aligned in the player's HUD block at column 15 (3-player layout) or column 23 (2-player layout) on scroll1 row 34 (R: 0x101B6, [SPEC.md (flow)](file:///Volumes/Shanto/Repos/Personal/Github/roms/re/specs/flow/SPEC.md#L33)).
  - Leading zeros are blanked.
  - **No Floating Scores**: there are no floating score numbers on the playfield when enemies are killed or items are collected. Points only register on the HUD score counter.
- **Rank Text**:
  - Directly underneath the score (row 35), routine `0x15974` draws the player's current place in the high score table as `1ST`, `2ND`, `3RD`, ... `10TH` (from table `0x15ACE`). If beyond 10th place, four spaces are drawn. Updated live by routine `0x15A2E`.
- **High Score Table (`G.$703E`, RAM `0xFFF03E`)**:
  - Contains 50 entries of 16 bytes each (R: 0x1504E, [D4d-front-end-and-story.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D4d-front-end-and-story.md#L654-L660)):
    - `+0x00` (4 bytes): Score (BCD longword).
    - `+0x04` (4 bytes): Name (3 ASCII letters plus trailing space).
    - `+0x08` (1 byte): Hero ID (`0` Jack, `1` Mustapha, `2` Hannah, `3` Mess).
    - `+0x09` (1 byte): Stage reached plus 1 (`1..8`).
    - `+0x0A` (1 byte): Default table rank index (`100, 98, 96, ...`).
    - `+0x0B..+0x0F` (5 bytes): Unused/padding.
  - Initialized at boot by routine `0x1504E` (`hud_hiscore_init`) with scores descending from 100,000 down by 10,000 BCD increments. Entries 11 through 50 wrap to negative signed longwords (`99,990,000` down to `96,100,000`), so a player with 0 points starts at rank 11.
- **Name Entry (`0x15AE2`, `0x15BE4`)**:
  - Triggers on game over or game completion if the player's score exceeds the 10th table entry and rank is <= 10.
  - Runs in the player's HUD block area: 20-second timer, 420-frame idle timer.
  - 30 glyph options: `A-Z`, comma `,`, period `.`, `RUB` (backspace), and `END`.
  - Profanity filter (`0x1616C`): replaces `AAA`, `SEX`, `FUK`, and `FUC` with `   `, `CPS`, `CAP`, and `COM` respectively (R: 0x1616C, 0x161B2, 0x161C6).

---

## 4. Settings that Change These: EEPROM Options and Defaults

The cabinet configuration is stored in the 93C46 serial EEPROM image `G.$765A` (128 bytes) and unpacked at boot by routine `0xAC794` into cooked setting bytes `G.$777A`, `G.$777B`, and `G.$777C` (R: 0xAC794, 0xACBF4, [D4d-front-end-and-story.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D4d-front-end-and-story.md#L450-L474)):

### 4.1 Cooked Byte Definitions and Factory Defaults

| Setting Name | Cooked Byte and Bits | Available Options | Factory Default | System Affected |
|---|---|---|---|---|
| **Difficulty ("RANK")** | `G.$777B` bits 0..2 | `0` Extra Easy, `1` Very Easy, `2` Easy, `3` Normal, `4` Medium Hard, `5` Hard, `6` Very Hard, `7` Hardest | `3` (Normal) | Sets initial rank roll band in `0x17F8A` and death penalty table `0x187E0` (R: 0x17F8A). |
| **"TIMER LEVEL"** | `G.$777B` bits 3..4 | `0` Easy, `1` Normal, `2` Hard, `3` Very Hard | `1` (Normal) | Sets starting rank speed `G.$5F6 = level + 1` (default 2), controlling rank rise frequency (R: 0x17F8A). |
| **Bonus Life** | `G.$777B` bits 5..6 | `0` None, `1` 300k & 700k, `2` 500k & 1000k, `3` 1000k only | `2` (500k & 1000k) | Sets table `0x6BFE` option pointer `G.$4E5` for extra life rewards in `0x6B98` (R: 0x6B98). |
| **Starting Lives** | `G.$777C` bits 0..1 | `0` 1 life, `1` 2 lives, `2` 3 lives, `3` 4 lives | `1` (2 lives total) | Initializes player spare life counter `+$A3` in routine `0x4DD6` (R: 0x4DD6). |
| **Free Play** | `G.$777C` bit 2 | `0` Normal coin play, `1` Free play | `0` (Normal) | Disables coin insertion requirement; start button begins play immediately (R: 0x9CFC6). |
| **Continue Allowed** | `G.$777C` bit 6 | `0` Continue OFF, `1` Continue ON | `1` (ON) | Controls whether continue countdown runs or game terminates on death (R: 0xA804). |
| **Coin Chute Mode** | `G.$777A` bits 3..4 | `0` 3 Players / 3 Chutes, `1` 3 Players / 1 Chute, `2` 2 Players / 1 Chute | `0` (3P / 3 Chutes) | Selects HUD layout `G.$4CF` (2P vs 3P) and player credit allocation (R: 0x9D048). |
| **Coin Mode** | `G.$777A` bits 0..2 | 8 coin/credit ratio presets (1 coin 1 credit to 4 coins 1 credit, plus 2-start 1-continue) | `0` (1 coin 1 credit) | Controls coin multiplier and credit deduction ratios (R: 0x9D038). |
| **Demo Sound** | `G.$777C` bit 5 | `0` OFF (muted during attract), `1` ON | `0` (OFF) | Attract sound suppression in routine `0x610` (R: 0x610). |
| **Sound Output** | `G.$777A` bit 7 | `0` Monaural, `1` QSound Stereo | `1` (QSound Stereo) | Sets QSound DSP panning and stereo output flag at `0xF19FFD` (R: 0xA69A). |

---

## 5. Port Status

Status of time, rank, and score systems in `dino-rs/`:

| Component | State | Current Implementation File | Missing Details / Work Needed |
|---|---|---|---|
| Area start time budget | DONE | [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs#L311) (`rank_time_area_start`) | Table `0x1033E` lookup matches original for all 8 stages and areas. |
| 51-frames-per-second divider | DONE | [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs#L327) (`time_limit`, `rank_time_tick`) | BCD countdown matches 51 frames per second. |
| Spawn script time set/add | DONE | [spawn.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/spawn.rs#L254) (`0x0C`, `0x10`) | Dispatches to `rank_set_time` and `rank_add_time`. |
| Time over player kill | DONE | [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs#L353) (`rank_time_over`) | Kills active players with HP = 0. |
| Car mode time handling | DONE | [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs#L358) (`rank_time_over`) | Sets `car_mode = 2` and adds +3:00 on timeout against Hogg. |
| Time top-up on join / continue | PARTIAL | [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs#L373) (`rank_add_time`) | Function exists; callers in join and continue are stubbed. |
| Clock on-screen display | MISSING | [prop.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/prop.rs), [hud.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/hud.rs) | Pool B class 31 (`0x81046`), pool D sprite slot 15, digit tiles `0x811CA`. |
| Clock beep sound | MISSING | `snd.rs` | Sound code `0x34` not played on second ticks. |
| TIME OVER words animation | MISSING | `effect.rs` | Pool C class `0x70` flying text objects from `0x96450`. |
| Rank core computation | DONE | [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs) | Rank tick, speed, timers, clamping, level mapping match ROM tables. |
| Rank event wiring | PARTIAL | [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs) | Handlers exist; callers for death, join, continue, extra life not hooked. |
| Aggression Markov tables | DONE | [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs#L205) (`rank_reroll_aggression`) | Markov tables `0x182E4` and `0x18364` ported. |
| Hit score (0x10292) | DONE | [enemy_combat.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/enemy_combat.rs#L272) (`enemy_hit_score`) | Action score code byte looked up in table `0x10292`. |
| Kill score (0x325F8) | DONE | [enemy.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/enemy.rs#L658) (`enemy_award_kill_score`) | Awards score based on enemy name id. |
| Container break score | DONE | [item.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/item.rs#L480) (`item_score_500`) | +500 points on break. |
| Food and treasure scores | DONE | [item.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/item.rs) | Food health/score tiers and treasure score tiers match ROM tables. |
| Extra life threshold check | DONE | [hud.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/hud.rs#L442) (`hud_extra_life_check`) | Threshold tables and life increments ported. |
| 1UP pose and text marker | MISSING | `player.rs`, `effect.rs` | 1UP pose states and pool C class 14 variant 2 marker text missing. |
| Clear tally score calculation | PARTIAL | [flow.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/flow.rs) (`flow_clear_task`) | Stubbed with 120-frame sleep; math not wired into score count-down. |
| High score table storage | PARTIAL | [hud.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/hud.rs) (`hud_hiscore_init`) | 50-entry table initialized; insert and sort routines missing. |
| Name entry screen | MISSING | `flow/join.rs` | Name entry state machine `0x15AE2` / `0x15BE4` missing. |
| EEPROM settings loader | PARTIAL | [flow.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/flow.rs) (`Settings`) | Hardcoded factory values in `flow.rs`; external config loader missing. |

---

## 6. Port Task List with Sizes in Agent Hours

| Task ID | Description | Primary Files to Touch | Size (Agent Hours) |
|---|---|---|---|
| **T-TR1** | **Clock Display and Beep**: Port pool B class 31 (`0x81046`), pool D sprite slot 15 digit rendering (`0x811CA`), 3:00 threshold, 5s half-minute pulse, 15s solid display, red palette at 1:00, and sound `0x34` trigger. | `prop.rs`, `hud.rs`, `snd.rs` | 1.5 h |
| **T-TR2** | **TIME OVER Words and Death Flow**: Port pool C class `0x70` flying words ("TIME" from right, "OVER" from left meeting at center `0x00C0`), 180-frame hold, and link death flow with `+03:30` clock top-up (`0x10482`). | `effect.rs`, [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs), `player.rs` | 1.0 h |
| **T-TR3** | **Rank Event Wiring**: Wire death rank drops (`0x184E2`, `0x1842C`), join speed bonus (`0x18410`), continue speed penalty (`0x18566`), extra life rank adjust (`0x1843C`), and title credit counter (`G.$4F6`). | [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs), `flow/join.rs`, `player.rs` | 1.5 h |
| **T-TR4** | **1UP Victory Pose and Marker**: Port 1UP pose states (`0x18`, `0x418`, `0x41A`) within 240 frames of extra life threshold, invulnerability blink, and pool C class 14 variant 2 "1UP!" text sprite. | `player.rs`, `effect.rs`, [hud.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/hud.rs) | 1.0 h |
| **T-TR5** | **Stage Clear Tally Scoring**: Implement complete tally calculations (Vitality bonus with 20k for full HP, enemy kills * 100, Final Blow 20k, M.V.P. 10k, and 100 pts/frame count-down into player score). | [flow.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/flow.rs), [hud.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/hud.rs) | 2.0 h |
| **T-TR6** | **High Score Table and Name Entry**: Implement table insertion and sorting (`0x161DA`, `0x16260`), 20s name entry state machine (`0x15AE2`, `0x15BE4`), profanity filter, and ranking display. | `flow/join.rs`, [hud.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/hud.rs) | 2.0 h |
| **T-TR7** | **Configuration File Loader**: Create TOML/JSON configuration loader with factory defaults for difficulty, timer level, extra life options, start lives, and chute modes. | `config.rs`, [flow.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/flow.rs) | 1.0 h |

Total effort: approximately **10.0 agent hours**.

---

## 7. Open Items

1. **Episode 7 "All Bosses Defeated" Rank Speed Bonus**: fan documentation claims timer rank increments by 1 in Episode 7 only when all Slisaurs fall. In code, `flow_boss_defeated` (`0xACEE`) increments rank speed unconditionally. Verify whether class 71 death handlers execute individual calls or a group check.
2. **Title Wait Counter G.$4F6 Accumulation**: verify exact title screen frames where `G.$4F6` increments during attract loop with coins inserted versus demo playback.
3. **High Score Entry Byte +$0A**: default high score table entries contain values `100, 98, 96, ...` at offset `+$0A`. The code never writes this byte for newly entered scores. Determine if it was used by Capcom development diagnostics.
4. **Enemy Bullet Damage Scaling with Stale Register +$84**: confirm whether thrown projectiles (knives, rocks, bullets) should read per-slot attack record damage or maintain original 68000 quirk where `+$84` stale value was reused.
5. **Two-Player Layout Select Panels and Tally**: confirm layout coordinates for 2-player cabinet layout (`G.$4CF == 1`) tally columns and select screen panels.

---

## 8. New Mechanics Rows for `plan/game/mechanics-seen.md`

| ID | Mechanic | Source | Port | Notes for the Design Agent |
|---|---|---|---|---|
| TR01 | Stage time loaded from table `0x1033E` (`stage * 8 + area * 2`) at area start; frame divider `G.$4EA` counts 50 down to 0 (51 game frames per second, 0.855 real seconds) | R: 0x1031A, 0x1033E, 0x1037E | DONE | [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs); 8 stages with 1 to 4 areas each |
| TR02 | Clock display (pool B class 31, handler `0x81046`, pool D slot 15): hidden above 3:00; at 3:00 or less displays 5 s at each half minute; continuously visible in last 15 s; red palette `0x84` at 1:00; sound `0x34` beep each visible second | R: 0x81046, 0x811CA, 0xA63C | MISSING | Screen X 0xE8, Y 0xB8; suppressed during scripted control `G.$4DD` |
| TR03 | TIME OVER: at 0:00, words fly in (pool C class `0x70`, "TIME" from right, "OVER" from left meeting at center X 0xC0); all active heroes killed regardless of invulnerability; respawn adds +3:30 (cap 4:00) | R: 0x103FE, 0x1046A, 0x10482, 0x96450 | PARTIAL | Kill logic DONE; flying words MISSING |
| TR04 | Car mode time (Episode 3): stage timer frozen while `G.$518 == 1`; timeout against Hogg with car active (`G.$518 != 0 && G.$7B4 != 0`) does not kill heroes, ends drive (`G.$518 = 2`) and adds +3:00 | R: 0x103AA, 0x10436, 0x8C06E, 0x819C0 | DONE | [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs); fight continues on foot |
| TR05 | Time additions: player join adds +3:30 (cap 4:00) and +600 wave frames (`0x1049E`); continue adds +4:00 (cap 4:00) and +600 wave frames (`0x10490`); both add +1 point to all 3 players | R: 0x10490, 0x1049E, 0x7476 | PARTIAL | Functions exist; join/continue callers stubbed |
| TR06 | Stage script time commands: `0x0C` sets time directly (`rank_set_time`); `0x10` adds time with 4:00 cap (`rank_add_time`); 23 script triggers across all 8 episodes | R: `stage_scripts.txt`, 0xFD9C, 0xFDA4 | DONE | [spawn.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/spawn.rs) |
| TR07 | Level mapping: table `0x1860C` maps rank (0..31) to Level `G.$5EE` (0..31) by player count; 1P capped at level 24, 2P at 28, 3P at 31 | R: 0x18592, 0x1860C | DONE | [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs) |
| TR08 | Initial aggression table `0x18094` per stage and area (byte - 1); Markov drift tables `0x182E4` and `0x18364` re-roll aggression upwards on enemy kills | R: 0x18048, 0x18094, 0x182A2, 0x182E4 | DONE | [rank.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/rank.rs); continue calm holds 0 for 1200 frames |
| TR09 | Action score table `0x10292`: 34 BCD values (10 to 100,000); attack records hold score code byte (100, 200, 300, 400, 700, 800, 1000); container break +500 | R: 0x10292, 0xFFC8, 0x29858 | DONE | [enemy_combat.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/enemy_combat.rs), [item.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/item.rs); blocked during scripted control `G.$4DE == 1` |
| TR10 | Kill score table `0x325F8`: maps 39 enemy name IDs (`0x00..0x26`) to table `0x10292` offsets; awards 1,000 to 100,000 points; secret Wrench T. awards 80,000; Tyrog speed finish 100,000 | R: 0x325E6, 0x325F8 | DONE | [enemy.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/enemy.rs); Morgan awards 20,000 in data table |
| TR11 | Item scores: food at full HP awards 10k, 5k, 3k, 1k, 4k (0 when hurt); treasures award 10k, 5k, 3k, 1k (11 frames after press); ammo box +1,000 if full | R: 0x28AA4, 0x28B10, 0x2A164 | DONE | [item.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/item.rs) |
| TR12 | Stage clear tally scoring: Vitality ((HP-1)*100, or 20k if 100 HP), Enemies (kills*100), Final Blow (20k), M.V.P. (10k); counts down into score at 100 pts/frame | R: 0xB97E, 0xBC16 | PARTIAL | [flow.rs](file:///Volumes/Shanto/Repos/Personal/Github/roms/dino-rs/crates/dino-game/src/game/flow.rs); stubbed in port |
| TR13 | Extra life thresholds: 500k & 1000k (factory default, table `0x6BFE` option 2); sound `0x4A`; 240-frame idle window triggers 1UP victory pose and pool C class 14 variant 2 text marker | R: 0x6B98, 0x6BFE, 0x93BF8, 0xA4CA | PARTIAL | Threshold logic DONE; pose and text marker MISSING |
| TR14 | High score table: 50 entries of 16 bytes in RAM `G.$703E`; live HUD rank text "1ST".."10TH"; 20s name entry screen with 30 glyphs, idle timer 420 frames, and profanity filter | R: 0x1504E, 0x15974, 0x15AE2, 0x1616C | PARTIAL | Table init DONE; entry and sorting MISSING |
| TR15 | EEPROM configuration: cooked bytes `G.$777A/B/C` store difficulty (0..7, default 3), timer level (0..3, default 1), bonus life option (default 2), starting lives (default 1), continue, and chutes | R: 0xAC794, 0xACBF4 | PARTIAL | Factory defaults hardcoded; config file loader MISSING |
