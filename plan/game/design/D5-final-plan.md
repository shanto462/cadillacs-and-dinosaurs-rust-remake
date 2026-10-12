# D5: the final build plan

Written 2026-10-13 by the planning agent (task D5 of `plan/game/NEW-PLAN.md`). This file merges every design document into the plan the
implementation agents follow, one task at a time, until the whole game is finished. The user does not review this plan. The user plays
the finished game and judges it by looking: it must look and behave like the original from power on (the attract loop) to the ending.

Inputs: `plan/game/NEW-PLAN.md`, `design/X-task-index.md` (the 151 index rows), all design documents D2a to D4e, D3e parts 1 to 3 (part 3 is
complete), D3f, `X-rank-crosscheck.md`, `capture-recipes.md`, `asset-catalog.md`, `mechanics-seen.md` (588 rows after the last merge),
`STATUS.md`, `dino-rs/ARCHITECTURE.md`, `re/harness/README.md` and the git log. Where two documents disagree, the later one wins and the
change is listed in section 1.

How to use this file:

- The main session reads sections 1 to 5 once. Section 3 (rules) and section 4 (the prompt preamble) go into every task prompt.
- A worker gets section 3, section 4 and **one task block** from section 6, nothing else, plus the design sections the block names.
- Section 6 is the build order. Lane A (code in `dino-rs`) runs strictly in the order of the file. Lane B (documents, harness, MAME
  recordings, no edits in `dino-rs`) runs beside lane A whenever its needs are met.
- Section 7 maps every row of `mechanics-seen.md` to a task. Section 8 holds the risks, section 9 the estimates, section 10 the first
  ten tasks, section 11 the edits the main session makes to other files.

## 1. Decisions made while writing (no questions were possible)

Ground truth order: ROM code and MAME runs, then repo specs, then the design documents, then the web. Where a design document and MAME disagree,
MAME wins and the agent writes the correction in `plan/game/design/D5-errata.md` (append only, one line per correction).

1. **Placeholders carry the skeleton** (NEW-PLAN rule 2). M3 adds one generic enemy for every pool A class that is not ported, a placeholder
   boss per episode on the real entry class, and a registry (`game/placeholder.rs`, `--list-placeholders`). Gating props (walls and fences the
   player must break) are real from M3. Every class task of M5 to M8 deletes its registry entry. The registry is removed in M9-14.
2. **Boss services move to M3.** The index put `T-C4` (boss death chain) in M3 and its prerequisite `P-D3c-1` in M5. Both are now M3-02 and M3-03,
   because the boss gate and the end of every episode need them.
3. **Name entry exists** (D4d s7, A17, A21). This replaces NEW-PLAN decision 2 ("no name entry seen"). It is task M2-22 and it runs after the ending too.
4. **Out of scope on purpose** (each has a coverage row): the EEPROM test menu and service mode (F13 menu part, F14), the debug mode at power on
   (F27, L16), the anti tamper checks (E26, F23, F34), region sets other than World, the unused art (DS13, FX30), probe notes (T28 to T30, W242, W324).
   Settings (difficulty, timer level, bonus life, lives, free play, coin mode, demo sound) come from a plain `dino.cfg` text file (M2-01).
5. **Hand port of the sound driver** (M4). The Z80 driver is ported into a new crate `dino-sound`, not interpreted. The QSound DSP internal ROM is not
   in `dino.zip`, so the pan law and the echo are approximations. Exactness is tested at the QSound register write level against MAME (M4-01, M4-09).
   A small Z80 interpreter inside `dino-sound` is the fallback only if M4-10 and M4-11 cannot reach equality; it would break the "no emulation" rule
   of `CLAUDE.md`, so it needs the user's agreement and is not planned.
6. **One acceptance tool for all design scenarios.** The scenarios of D2 to D4 are written as probe segments. M3-06 and M3-07 make any probe
   segment a harness scenario (snapshot after the setup lines, trace after it, the port loads the snapshot of any episode and area). This is
   cheaper than converting every scenario by hand. Flow screens use power-on traces with timed pokes (M1-00, M2-00) and the picture compare tool (M1-14).
7. **Stale register quirks are kept.** The per slot `+$84` (and `+$5C` of Vice's Hopper) is kept across frees as in the original (D3a open item 12,
   D3b open item 7). A fresh game has 0 there. M5-01 does it.
8. **The ranking car is pool B 86** (D4d T1.4 and D3e agree on the handler 0x8B428; the index asked D5 to pick). M2-21 reads the continue screen code once more to confirm.
9. **The title logo is pool A class 98** (D4d), not pool B 98 as NEW-PLAN wrote.
10. **Fessenden first is class 94, the final form class 84** (D3d s6; D2c table 16.3 and `episode-8.md` have it reversed). The Slisaur starter is pool B 33 (not 41).
    Hogg is rider 31 plus bike 32. Walther is a normal counted unit, not a boss (D3a).
11. **Hero damage factors** by hero id: Jack 32, Mustapha 34, Hannah 35, Mess 30 (D2b s5; the two old specs disagree, the id order wins). Unit test in M9-03.
12. **The attract demos are exact only as far as their level exists.** M1-12 and M1-13 build the plumbing and check the first frames; the six demo rows are
    compared again with real content in M10-03 to M10-05.
13. **Slowdown is not reproduced.** The original takes two vblanks for a game frame when too many objects are active. The port keeps one logic frame per
    tick (M10-24 paces the loop at 59.64 Hz). This changes how heavy scenes feel but not the logic. It is in the risk list.
14. **FX tasks follow D3f.** `P-D2a-8`, `P-D2c-6` and `IT-P2` of the index are folded into the D3f groups (M5-04 to M5-08, M2-17, M2-21, M6-14, M6-22, M7-03, M8-34).
    D3f's class 9 "STAGE CLEARED" banner was never seen in the tally: M9-05 decides with a MAME probe.
15. **D3e part 3 sizes replace the index sizes for Episodes 6 to 8** (the part is complete now). Gates and transition props move to M3 (B3, B1, B35, B47, B36 to B38);
    the shower and quake props B83 to B85 are part of the ending (M3-23). New rows of the index gap table that had no owner now have one: B76 (M5-03),
    class 56 of the Episode 1 opening (M2-11), pool B 75 to 78 flow objects (M2-24 to M2-26), ending actors (pool A 90 to 93, 96, 97: M3-23, M3-27).
16. **P4 is split**: the Rock Hopper intro states go with Vice (M5-10), the hostile modes with Episode 5 (M7-12). **P-D3d-1 is split in three** and runs before Morgan (M7-13 to M7-15).
17. **Acceptance runs follow each boss** (the 6 hour acceptance rows `P-D3c-7` and `P-D3d-10` are spread over the last task of each boss). Whole episode runs are M10.
18. **Duplicates folded** (index list kept): clock beep inside the clock task, GO sound inside the hint, config file built once (M2-01), `S2` and `T4.4` are one task (M4-13).
19. **Parallelism**: one code worker on `dino-rs` at a time, at most two workers in all (user rule). Lane B tasks never touch `dino-rs`. Using a second code worker in a git worktree for
    `crates/dino-sound` only (M4-02 to M4-11) would save about three weeks; it is off by default because the user ruled for one code worker. Noted as an option, not planned.

## 2. The milestones in build order

The principle stays: the backbone first, so the whole game is playable early. M1 and M2 give the first minutes and the start of a game. M3 gives the whole game from power on to the
ending (fights are placeholders). M4 adds sound. M5 to M8 replace the placeholders with the real classes, episode by episode. M9 sweeps every small thing. M10 proves exactness.
Hours are agent hours of the task blocks in section 6 (lane A plus lane B). Section 9 explains the estimate.

| M | Name | Tasks (A + B) | Hours (A + B) | What the user can look at when it is done |
| --- | --- | --- | --- | --- |
| M0 | Every episode starts | 1 (done) | 1.5 | Done (`f8e3c93`). Episodes 2 to 8 start, `--area`, `--episode`. |
| M1 | Attract loop, exact from power on | 16 + 1 | 24.2 (23.2 + 1.0) | Power on: boot text, warning, title with the blinking INSERT COIN, four profile cards, the ranking, the story slides, the opening movie, the demos (as far as their level exists). The loop repeats. |
| M2 | Coin, select, Episode 1 opening, clock, join, continue, game over, tally | 31 + 2 | 51.5 (47.5 + 4.0) | Coin, press start, select a hero with the 10 second timer, the real Episode 1 opening, the clock and TIME OVER, die and continue or GAME OVER, name entry, the clear tally, map card, story panel and episode card. |
| M3 | The whole game skeleton | 28 + 2 | 51.1 (47.6 + 3.5) | From power on to the ending through all 8 episodes with every scripted transition, opening, link scene, boss gate, lift, car camera and the ending with the staff roll. Enemies and bosses are placeholders. A bot plays it in minutes. |
| M4 | Sound | 17 + 3 | 35.0 (29.5 + 5.5) | Music and sound effects from the attract loop on, with the original driver logic and pan. |
| M5 | Episode 1 for real | 20 + 3 | 36.9 (32.9 + 4.0) | Episode 1 from the opening to Vice T. and the tally, with real effects, weapons, items, Rock Hopper and Walther code. |
| M6 | Episodes 2 and 3 | 32 + 1 | 58.9 (56.9 + 2.0) | Poachers, Lash, Mack, Shivat with the hunter, the Butcher; the radio, the Cadillac drive, Hogg, barrels, bikers. |
| M7 | Episodes 4 and 5 | 27 + 1 | 49.5 (47.5 + 2.0) | Slice and the garage; Bludge, Zeke, Rock Hopper, Morgan and Morgue, the burning village. |
| M8 | Episodes 6, 7 and 8 | 36 + 1 | 66.4 (64.4 + 2.0) | The foot and the storm, Tyrog three times, the lift and the Slisaurs, the tanks, the flame vents, Fessenden in two forms, the real ending. |
| M9 | Small things pass | 9 + 5 | 25.0 (15.0 + 10.0) | Tag team move, unspawned classes, the row by row audit of `mechanics-seen.md`, the fixes it finds. The placeholder registry is gone. |
| M10 | Exactness and finish | 24 + 2 | 57.5 (53.5 + 4.0) | Demos with real content, the attract loop twice, every episode run against MAME with 1 and 3 players and all heroes, settings, sound, pacing, README. |
| | **Total** | **240 + 21** | **456.0** (418.0 + 38.0) | |

Order of dependencies between milestones: M1 before M2 (services), M2 before M3 (scripts, tile blocks, tally), M3 before M5 to M8 (placeholders, boss services, camera),
M4 can start after M3 (it needs the screens to wire cues; the crate itself needs nothing and the spec task M4-00 and the MAME recorder M4-01 are lane B work that can run at any time),
M5 before M6 (monster services, Vice's bullet), M6 before M7 and M8 (poachers, boss patterns), M7-13 to M7-15 before M8 bosses, M9 after M8, M10 last.
The user's list asked for sound right after M3; that is kept (M4), and the effects and passes of M5 are needed by every later episode.

## 3. Rules of execution

These rules are copied from NEW-PLAN section 3, the project `CLAUDE.md` and the standing instructions in memory, and made sharper.

1. **One code worker at a time.** Only one task that edits `dino-rs` runs at any time, whatever the worker (a Claude agent or an external CLI). At most two workers run at once,
   and the second one only runs lane B tasks (documents, harness scripts, MAME recordings, probe sweeps). Never stop a running worker to meet the limit. Workers never start workers.
2. **Task flow.** The main session hands out the task: this section, the preamble of section 4 and the task block. The worker implements, runs the gates below and writes a short report
   (what changed, the gate numbers, every difference left with its cause). The main session reviews the diff and the evidence, looks at the screenshots, then **commits and pushes**
   (one commit per task, no AI attribution, repo email `shalahuddinshanto@gmail.com`, never ROM data). Workers never commit or push. Report progress with an ETA at every milestone.
3. **The gates of every task** (the "Check" line of a block names the specific MAME comparison; these always apply):
   - G1 `cargo build` and `cargo test` from `dino-rs/` are green.
   - G2 `re/harness/regress.sh` is not worse than the last green run (write both numbers in the report).
   - G3 The new behaviour is compared with MAME: a probe segment or harness scenario named in the block, or a power-on trace and picture compare for screens. The report gives the first
     differing frame, or says "equal for N frames". A new known difference goes into the milestone allow file `re/harness/allow/mN.txt` with its cause. An allow rule never hides a code mistake.
   - G4 Look at the real game: run it with the play script (`--input-file`, `--screenshot`, `--frames`, screenshots in `re/out/shots/`), view the pictures and describe what is on screen.
     For M1 and M2 this means from power on, not from a stage jump.
   - G5 Every ported routine names its original address in a doc comment. Data comes through `dino-extract` as exact table copies; no ROM bytes in source, tests or documents.
4. **MAME is the judge and is used in batches.** One headless MAME process for a whole batch (`-video none -sound none`, `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy`), never one process per
   item, never the MAME4mac app. Outputs go to `re/out/` (the rendered asset dump `re/out/assets/` is committed by the user's choice; nothing else derived from the ROM is).
5. **The first difference rule.** When a run differs, find the first differing frame, name the cause, fix the cause. Do not add allow rules to make a run pass. If a task needs more than twice its
   size, the worker stops, writes what it knows in `D5-errata.md`, and the main session splits the task.
6. **Placeholders.** A real class task removes its class from the registry and runs `--list-placeholders` for its episode in the check. Autoplay (`--autoplay`, M3-05) from power on to the ending must
   still pass after every milestone.
7. **File ownership** follows `dino-rs/ARCHITECTURE.md` (section 11 lists the rows the main session adds). A worker edits only its files. Allowed small edits: one line in each `lib.rs` and one
   `AssetPack` field for a new assets module; appending animation table addresses to `OBJECT_ANIM_TABLES`; a hook line in a main session file when the block names it ("allowed" in its Files line).
   If the build breaks in a file the worker does not own, wait and retry. `impl Game` methods use the area prefixes of ARCHITECTURE.md.
8. **Exactness rules of ARCHITECTURE.md hold**: one `tick` is one original frame, 68000 integer semantics, `Game::rand()` only and in the original call order, quirks and bugs kept with a comment.
9. **External CLI workers** (marked X): `codex`, then `agy`, then `grok` (all with their default model, no model flag; when one hits a usage limit, move the job to the next). X tasks are the easy
   and medium, well specified ones: tables, small classes, tests, scripts. Never design research or exact boss logic. Review every diff; an X task that edits `dino-rs` takes the code slot.
10. **Claude workers** (marked C): `sonnet-xhigh-implementer` (Sonnet 5.5, xhigh) for code tasks, a general purpose agent with model sonnet for research and doc tasks.
11. **When a design document is wrong**, probe MAME first, then write the correction in `D5-errata.md` and use MAME's value. Never edit the other design documents.
12. **Milestone end**: the last task of every milestone is an acceptance task. After it the main session plays the build (from power on for M1 to M3, from the coin to the milestone's boss
    afterwards), reports what it saw, updates `STATUS.md`, and pushes.

## 4. The prompt preamble (copy into every task prompt)

```
You are an implementation worker for the Rust port of Cadillacs and Dinosaurs (CPS1) in /Volumes/Shanto/Repos/Personal/Github/roms.
Read first: CLAUDE.md, dino-rs/ARCHITECTURE.md, plan/game/design/D5-final-plan.md sections 3 and 4, and the design sections your task block names.
Do exactly one task: the block below. Rules: no sub-agents or workflows; do not commit or push; no ROM bytes or ROM derived data in source, tests or
documents (exact table copies go through dino-extract and are read from dino.zip at run time); every ported routine names its original address;
keep 68000 integer behaviour and the original RNG call order; prefix new impl Game methods with the area prefix of ARCHITECTURE.md; edit only the files
named in the block (plus the allowed small edits of section 3 rule 7). MAME runs: one headless process per batch, SDL dummy drivers, never a GUI.
When done run cargo build, cargo test, re/harness/regress.sh and the MAME comparison named in the block, then run the game with the play script and look at
the screenshots (re/out/shots/). Write the report: what changed, gate numbers, differences left with causes, open questions. If the task needs more than
twice its size, stop and report. Simple English, no em dashes.
```

## 5. Code layout the tasks use (new files and who owns them)

New files, grouped by owner row of `ARCHITECTURE.md` (the main session adds these rows, section 11):

| Owner row | New files |
| --- | --- |
| flow / HUD port | `game/flow/scene.rs`, `text.rs`, `boot.rs`, `attract.rs`, `movie.rs`, `demo.rs`, `system.rs`, `select.rs`, `cont.rs`, `name_entry.rs`, `tally.rs`, `cards.rs`, `ending.rs` (next to the existing `front.rs`, `join.rs`); `game/script.rs` (scripted control, one hook in `player.rs`); `game/config.rs`; `dino-assets/src/frontend.rs`, `script.rs`; `dino-extract/src/frontend.rs`, `script.rs` |
| enemy port | `game/boss.rs`, `boss_common.rs`, `placeholder.rs`, `car.rs`, `player_car.rs`, `player_tag.rs` (the last three are player owned in spirit; the player port edits them), `game/enemy/classNN.rs` (one file or a small family per class, as `class10_*.rs` and `class14_*.rs` do), `game/prop/*.rs` (child modules of `prop.rs` with `#[path]`, one per class or family), `dino-assets/src/boss.rs`, `dino-extract/src/boss.rs` |
| draw / effects port | `game/effect/fxNN.rs` (child modules of `effect.rs` with `#[path]`, one per pool C class or family) |
| sound port | `crates/dino-sound/` (new crate: driver, sequencer, voices, pan, mixer, examples), `dino-game/src/audio.rs`, `dino-assets/src/sound.rs`, `dino-extract/src/sound.rs` |
| main session | `re/harness/*` changes of lane B tasks, `main.rs`, `input.rs`, `present.rs`, `render.rs`, `harness.rs` (workers may edit them only when a block says "allowed") |


## 6. The task catalog (build order)

Every task ends with the gates G1 to G5 of section 3 in addition to its "Check" line. "C" is a Claude worker, "X" an external CLI worker. Sizes are agent hours. Lane A tasks edit `dino-rs` and run one at a time in this order; lane B tasks (documents, harness, MAME recordings) run beside them as soon as their needs are met and before the first task that needs them.

Lane B tasks and the first task that waits for each:

| Lane B task | Hours | Worker | Needs | First task that needs it |
| --- | --- | --- | --- | --- |
| M1-14 Picture compare tool | 1.0 | X | none | M1-15 |
| M2-30 Harness: 2 and 3 player scenarios M1 to M10 | 2.0 | X | M2-00 | M5-20 |
| M2-31 Flow scenario scripts for D4d and D4c | 2.0 | X | M2-00 | M2-32 |
| M3-06 Harness: probe segments as scenarios (snapshot and trace commands) | 1.5 | C | M2-00 | M3-07 |
| M3-28 Flow scenarios: the end chain of all 8 episodes, link scenes, camera events | 2.0 | X | M3-06, M2-31 | M3-29 |
| M4-00 Finish the sound spec: pan motion, ADPCM pan, tables | 2.0 | C | none | M4-01 |
| M4-01 QSound register trace recorder (MAME) | 2.0 | X | M4-00 | M4-09 |
| M4-09 Register trace comparer | 1.5 | X | M4-01 | M4-10 |
| M5-15 Prepare scenarios: player moves and respawn | 1.5 | X | M3-06 | M5-18 |
| M5-16 Prepare scenarios: weapons and items | 1.5 | X | M3-06 | M5-20 |
| M5-17 Prepare scenarios: Episode 1 full runs | 1.0 | X | M3-06 | M5-22 |
| M6-00 Prepare scenarios for Episodes 2 and 3 | 2.0 | X | M3-06 | M6-01 |
| M7-00 Prepare scenarios for Episodes 4 and 5 | 2.0 | X | M3-06 | M7-01 |
| M8-00 Prepare scenarios for Episodes 6 to 8 | 2.0 | X | M3-06 | M8-01 |
| M9-06 Open item probe sweep A: player, weapons, items, front end | 2.0 | C | M8-00 | M9-11 |
| M9-07 Open item probe sweep B: enemies, dinosaurs, bosses, camera, sound | 2.0 | C | M8-00 | M9-11 |
| M9-08 Mechanics rows audit A: groups A to E (player, enemies, dinosaurs, bosses, items) | 2.0 | C | M8-36 | M9-11 |
| M9-09 Mechanics rows audit B: groups F to I (flow, sound, effects, camera, design rows) | 2.0 | C | M8-36, M4-19 | M9-11 |
| M9-10 Mechanics rows audit C: groups J to N (user notes and web leads) | 2.0 | C | M8-36 | M9-11 |
| M10-01 Prepare whole-episode recordings (all 8 episodes, 1 player) | 2.0 | X | M5-17 | M10-02 |
| M10-02 Prepare whole-episode recordings for 3 players and the other heroes | 2.0 | X | M10-01 | M10-15 |

### M1: Attract loop, exact from power on

The first five minutes of the original: boot text, warning, title, profiles, ranking, slides, opening movie, demos. Sound comes in M4 (the attract filter, S1, mutes most codes anyway).

Hours: 24.2 (lane A 23.2, lane B 1.0), 17 tasks.

#### M1-00 Power-on trace mode for the harness

**2.0 h, C (Claude worker), lane A. Needs: none. From: new (NEW-PLAN M1 risk, D4d s13 A1 to A3).**

- **Scope.** Let MAME record the original from power on with no snapshot, and let the port boot from power on and write a trace in the same format. diff.py then reports the first differing frame per layer (scroll 1/2/3 cells, palette, sprite list, sound requests, G.$4D0, G.$4D1, rng G.$14). Objects are not compared in this mode. MAME side `DINO_SYNC=power`; port side `MODE=boot port.sh` runs `dino-game --boot --input-file --trace`.
- **Original.** none (tool).
- **Data to extract.** none.
- **Files.** re/harness/record.lua, record.sh, port.sh, check.sh, diff.py; dino-game/src/harness.rs, game/trace.rs, main.rs (allowed for this task).
- **Read.** re/harness/README.md, capture-recipes s2 and s8, D4d s13 (A1 to A3 frames).
- **Check.** A power-on MAME recording of 700 frames made twice is byte for byte equal. The port boots, writes a trace, diff.py prints the first difference (expected at frame 0 until M1-03 lands). Compare of the scroll 1 cells at frame 100 works end to end.

#### M1-01 Scene runner and scroll 1 text printers, boot into the attract entry

**1.2 h, C (Claude worker), lane A. Needs: M1-00. From: D4d T1.1 (part 1).**

- **Scope.** A small scene runner (sleep counters plus a step function, so a script like 0x988C0 reads like the original). Scroll 1 text printers: bytes with positions, word strings, centered and numeric variants. `main.rs` starts in the attract loop by default (`--stage N` keeps the direct start). The sound filter of D4d T1.1 is S1 in M4, not here.
- **Original.** 0xBAE, 0xBB6, 0xC0A, 0xC44, 0xC4C, 0xC9A, 0xCD2, 0xCE4.
- **Data to extract.** the string tables each printer reads (through M1-02a when they exist; otherwise a local test table).
- **Files.** game/flow/scene.rs, game/flow/text.rs (new), flow.rs, main.rs (allowed), dino-assets/src/text.rs.
- **Read.** D4d s1 and s14 (T1.1).
- **Check.** Unit tests per printer on fixed strings. A printed string equals MAME's scroll 1 cells at power-on frame 100 (M1-00 tool).

#### M1-02a Front end data tables, exact copies

**1.2 h, X (external CLI), lane A. Needs: M1-01. From: D4d T1.1 data, Appendix B.**

- **Scope.** Extract every table of D4d Appendix B into typed structs: page tables (scroll 1 base 0x15CFE8 with the page index lists, scroll 2 base 0x145DC8 with cells 0x175378, scroll 3 base 0x177288), string tables, block lists, profile card texts, timeline tables 0x9A8E4 and 0x9A908, ranking default table, story slide texts, movie tables 0x9C002 and 0x9C026, panel texts, demo rows (0x9A1E2 and the pointer tables) and the 30 input streams (0xC869A to 0xC9EE6). Exact table copies only.
- **Original.** data only.
- **Data to extract.** all of Appendix B.
- **Files.** dino-extract/src/frontend.rs, dino-assets/src/frontend.rs (new), one line in each lib.rs and one AssetPack field (allowed).
- **Read.** D4d Appendix B and s3.6 (demo data), ARCHITECTURE.md (data versus code).
- **Check.** cargo test: every table has the length written in Appendix B; `cargo run -p dino-extract` prints counts only (no ROM bytes). No ROM bytes in source.

#### M1-02b Picture loaders, block printers, tall font and typing helper

**1.2 h, C (Claude worker), lane A. Needs: M1-02a. From: D4d T1.1 (part 2).**

- **Scope.** Port the page loaders for the three scroll layers, the block printers and the tall font printer with the typing helper that B66, the story slides and the intro cards use.
- **Original.** 0x20454, 0x204CA, 0x20592, 0x98B86, 0x98BAE, 0x84F72, 0x84736, 0x7EC20.
- **Data to extract.** from M1-02a.
- **Files.** game/flow/text.rs, game/flow/scene.rs, game/vram.rs (stage owner: allowed small hook).
- **Read.** D4d s1, appendix B, asset-catalog N5 and N10 (pictures are data, big font tiles).
- **Check.** The title picture pages draw tile for tile like MAME at title frame 340 (checked once M1-03 exists). Tall font string of the title licence text equals MAME at title frame 420.

#### M1-03 Boot screens, title screen, INSERT COIN blink, logo

**2.0 h, C (Claude worker), lane A. Needs: M1-02b. From: D4d T1.2.**

- **Scope.** The two boot text screens (text 1 from frame 73 to 194, text 2 from about 196 to 317, layer control change at 317, title counter 1 at 322), early skip by attack or jump after 30 frames, the title (green blocks, logo object pool A class 98 variants 0 to 3, 0x10 and 0x11, small lines at title frame 30, licence text at frame 360, INSERT COIN blink task 30 on 30 off, press start variant).
- **Original.** 0x9DCF6, 0x9DD80, 0x9A3CE, 0x98874, 0x983EE, 0x978EE, 0x980B0, 0x6A20A.
- **Data to extract.** boot text and palettes, title strings, block lists (M1-02a).
- **Files.** game/flow/boot.rs (new), flow.rs, enemy/ (class 98 object), draw.rs (allowed hook).
- **Read.** D4d s2, s3.3, s13 A1, A2, A4; D3f class 35 (unused).
- **Check.** Power-on trace compare (M1-00) of frames 0 to 700: text 1 and 2 frames, layer change at 317, title counter at 322 exact; pictures at 100, 250, 330, 340, 420, 560, 700 equal (H-02 tool). Skip tests A2 (226, 234 and none for player 3).

#### M1-04 Attract state machine, coin edge, press start, wait counter G.$4F6

**1.5 h, C (Claude worker), lane A. Needs: M1-03. From: D4d T1.8.**

- **Scope.** The attract task with its state byte (steps of 2, order title, profiles, ranking, title, demo, slides and ranking, ...), the state kept across games, the coin edge and credit (minimal system task), press start to the select (stub that starts Episode 1 until M2-03), sound requests 0x30 and 0x31 recorded, and the G.$4F6 counter that feeds the starting rank (F35). Stub screens fill the not yet ported states with the exact frame counts.
- **Original.** 0x97C20, 0x97C94, 0x97D84, 0x97CC0, 0x97D02, 0x983D8, 0x9D048, 0x9D4BE, 0x9D326, 0x1827A.
- **Data to extract.** state tables of D4d s3.1 and s3.2.
- **Files.** game/flow/front.rs, game/flow/attract.rs (new), rank.rs (feeder hook only).
- **Read.** D4d s3.1, s3.2, s3.10, s4.1, s13 A3, A10, A11.
- **Check.** Power-on run with no input: state change frames 2744, 6874, 9311, 13411, 15848, 19979, 22416, 26516, 28953 equal MAME (A3). Coin at 450 and start at 600: credit at 460, mode 4 at 623 (A10). Start without credit does nothing (A11). Rank start wait counter G.$4F6 equals MAME after 3000 frames with a credit.

#### M1-05 Four profile cards: layout, texts, timeline

**1.0 h, C (Claude worker), lane A. Needs: M1-04. From: D4d T1.3 (part 1).**

- **Scope.** The profile timeline machine and the four card screens (portrait on the checker board, name, height, weight, job text, skill word in red), fades, card order Jack, Hannah, Mustapha, Mess, card start frames about 932, 1297, 1658, 2019.
- **Original.** 0x9A7E6, 0x9A92E to 0x9AB32 (layout part), tables 0x9A8E4, 0x9A908, 0x9AC86, 0x9ACBA.
- **Data to extract.** card texts, labels, timeline (M1-02a).
- **Files.** game/flow/attract.rs.
- **Read.** D4d s3.4, s13 A5.
- **Check.** Pictures of the cards at 932, 1297, 1658, 2019 equal MAME once M1-06 is in (the pose sprite is the only difference before). Fade frames equal.

#### M1-06 Profile card pose objects and sounds hooks

**1.0 h, C (Claude worker), lane A. Needs: M1-05. From: D4d T1.3 (part 2).**

- **Scope.** Pool B classes 93 and 94 and pool A class 100 (the small animated hero doing a move on each card), the pose sound calls (recorded, heard in M4).
- **Original.** 0x91FB4, 0x921DA, 0x6A5F4.
- **Data to extract.** none new (animation tables are in the asset pack).
- **Files.** game/enemy/ (class 100), game/prop/ (B93, B94 via #[path]), flow/attract.rs.
- **Read.** D4d s3.4, open item 1 (read from the code), asset-catalog s4.
- **Check.** A5: sprite list at the card starts equals MAME (all four cards); sound request frames of D4d 3.11 equal.

#### M1-07 Ranking screen

**1.5 h, C (Claude worker), lane A. Needs: M1-06. From: D4d T1.4.**

- **Scope.** Draw the ten rows from the default table (rank, three letters, score, face icon, stage digit), the desert pictures scrolling 8 px (scroll 2) and 4 px (scroll 3) per frame, the car crossing, fades. Decision for the open point of the index: pool B 86 (0x8B428) is the ranking car; the continue screen uses effects and picture data (M2-24 reads the handler again to confirm).
- **Original.** 0x1516C, 0x15594, 0x15748, 0x15888, 0x1591C, 0x8B428.
- **Data to extract.** label block, row tables, face tiles (M1-02a).
- **Files.** game/flow/attract.rs, hud.rs (table access), game/prop/ (B86).
- **Read.** D4d s3.5, s7, s13 A6; asset-catalog N4 (the warning picture is skipped in the World set).
- **Check.** A6 pictures at 2330, 2500, 2740 equal MAME; rows equal the default table; car position per frame.

#### M1-08 Five story slides

**1.5 h, C (Claude worker), lane A. Needs: M1-07. From: D4d T1.6.**

- **Scope.** The slide script of 0x988C0 with its blocks, tall font captions and fades; cuts at 5010, 5320 to 5340, 5610 to 5630, 5730 to 5750, 6080, 6200, 6450. D3e s8 lists pool B classes 91 and 92 as story objects: port them here if the script spawns them.
- **Original.** 0x988C0 (and the pool B 91, 92 handlers if used).
- **Data to extract.** blocks and texts (M1-02a).
- **Files.** game/flow/attract.rs.
- **Read.** D4d s3.7, s13 A8.
- **Check.** A8 pictures every 10 frames from 4600 to 6500 equal MAME (power-on trace or H-02 picture compare).

#### M1-09 Opening movie, part 1 (timeline engine, handlers 10, 200, 500)

**1.5 h, C (Claude worker), lane A. Needs: M1-08. From: D4d T1.7a.**

- **Scope.** The movie timeline engine (events table 0x9C002 and 0x9C026), the text crawl and CAPCOM handlers, the first helper tasks.
- **Original.** 0x9BF50 to 0x9C29C.
- **Data to extract.** texts, block lists (M1-02a).
- **Files.** game/flow/movie.rs (new).
- **Read.** D4d s3.8, s13 A9.
- **Check.** A9: cut list and picture frames from 11900 to 12600 equal MAME.

#### M1-10 Opening movie, part 2a (handlers 600 to 1000, jungle)

**1.3 h, C (Claude worker), lane A. Needs: M1-09. From: D4d T1.7b (half).**

- **Scope.** The jungle scenes, palettes, pages 3, 4 and 0x72, objects 11, 12, 30 and 69 (described by what they spawn, open item 2 of D4d: read the code).
- **Original.** 0x9C29C to about 0x9C4xx, 0x9C7C4, 0x9C7F8.
- **Data to extract.** palettes, pages, lists.
- **Files.** game/flow/movie.rs, game/enemy/ and game/prop/ for the movie objects (classes 11, 12, 30, 69).
- **Read.** D4d s3.8, open items 2.
- **Check.** A9 pictures from 12600 to 13000 equal MAME.

#### M1-11 Opening movie, part 2b (handlers 1000 to 1450, helpers, sounds)

**1.3 h, C (Claude worker), lane A. Needs: M1-10. From: D4d T1.7b (half).**

- **Scope.** The rest of the jungle, the roar, objects 32 and the logo, helper 0x9C818, the sound requests, the movie end and return to the loop.
- **Original.** 0x9C4xx to 0x9C79C, 0x9C818.
- **Data to extract.** as M1-10.
- **Files.** game/flow/movie.rs, game/prop/ (B23 to B26 and B32 as named in D3e s8).
- **Read.** D4d s3.8, D3e s8 (front end classes 23 to 26, 32).
- **Check.** A9: every cut and picture from 13000 to 13450 equal MAME; sound request frames equal.

#### M1-12 Demo, part 1: rows, streams, setup, RNG reset, input playback

**1.5 h, C (Claude worker), lane A. Needs: M1-04. From: D4d T1.5 (part 1).**

- **Scope.** The demo rows table (episode and area per row, G.$757A counter), the exact RAM clear and the RNG reset (G.$4 to G.$E = 0 and G.$14 = 0x1C3), the 30 recorded input streams and the playback that feeds the player pad, three demo heroes (two on a 2 player cabinet). The demo plays whatever the level has at that time (placeholders after M3).
- **Original.** 0x99E88, 0x99F10, 0x9A0CE, 0x9A146, 0x9A1E2, streams from 0xC869A.
- **Data to extract.** rows, pointer tables, streams (M1-02a).
- **Files.** game/flow/demo.rs (new), input.rs (main session file: allowed hook for playback).
- **Read.** D4d s3.6, s13 A7, F33.
- **Check.** For each of the 6 rows: RNG state, player objects and sprite list at demo frames 60, 300 and 600 equal MAME in power-on trace (A7). Exact for the parts of the level that exist (Episode 1 row 5 first); the rest is checked again in M10-03 to M10-06 once the episode is real.

#### M1-13 Demo, part 2: demo end, loop resume, INSERT COIN in the level

**1.5 h, C (Claude worker), lane A. Needs: M1-12, M1-11. From: D4d T1.5 (part 2).**

- **Scope.** The 1200 frame demo end and the fade (1205), the INSERT COIN task kill for stage 5, resume of the loop in the saved state after a game over (F25), coin during the demo, press start during the demo goes to select. Now the whole loop runs with every screen.
- **Original.** 0x18C88, 0xAC6A, 0x97D84 resume path.
- **Data to extract.** none.
- **Files.** game/flow/demo.rs, flow/attract.rs, flow/front.rs.
- **Read.** D4d s3.1, s3.6, s3.9, s13 A3, A7.
- **Check.** Run 32000 frames from power on with no input: state change frames, demo counters (1 at 3345, 2 at 9916, 3 at 16450, 4 at 23021, 5 at 29555) equal MAME (A3).

#### M1-14 Picture compare tool

**1.0 h, X (external CLI), lane B. Needs: none. From: new.**

- **Scope.** A python tool that takes a list of frames, the MAME screenshots (probe snapshots) and the port screenshots at the same frames and prints the percent of different pixels per frame, plus a side by side contact sheet of the worst frames. Used by every screen task.
- **Original.** none.
- **Data to extract.** none.
- **Files.** re/harness/probe/shotdiff.py (new, python only, run with re/.venv/bin/python -I).
- **Read.** capture-recipes s5 (output layout), re/harness/probe/sheet.py.
- **Check.** Run on two equal folders (0 percent) and on two different folders (reports them). Documented in re/harness/README.md section.

#### M1-15 Attract acceptance: whole loop twice, fix list

**2.0 h, C (Claude worker), lane A. Needs: M1-13, M1-14. From: D4d s13 A1 to A9.**

- **Scope.** Run the loop from power on for 2 full loops (about 27000 frames) against MAME with M1-00 and H-02, list every remaining difference in `re/harness/allow/m1.txt` with its cause, fix the ones that are code mistakes. The demos are allowed to differ until M10 where the level content is missing.
- **Original.** all of M1.
- **Data to extract.** none.
- **Files.** re/harness/allow/m1.txt, the M1 files.
- **Read.** D4d s13.
- **Check.** Zero unexplained differences in the boot, title, profiles, ranking, slides and movie frames; the demo parts listed with reasons. Screenshots of every screen looked at by the main session.

### M2: Coin, select, Episode 1 opening, clock, join, continue, game over, tally

The user can play the real start: coin, press start, select a hero, the Episode 1 opening scene, play with the clock and TIME OVER, die, continue or GAME OVER, name entry, the clear tally, map card and story panels.

Hours: 51.5 (lane A 47.5, lane B 4.0), 33 tasks.

#### M2-00 Timed pokes for power-on traces (MAME and port)

**1.5 h, C (Claude worker), lane A. Needs: M1-00. From: new (recipes of NEW-PLAN s5, D4d s13).**

- **Scope.** MAME side: `DINO_POKES=file` with lines `frame hexaddr size value` applied from power on (same format as probe PR_POKES). Port side: `--poke F:ADDR:SIZE:VAL` with a small poke map for the addresses the recipes use (mode G.$4D0, sub mode G.$4D1, stage G.$4D9, area G.$786, time word G.$4E8, G.$508, G.$4D4, G.$4D5, G.$4D7, G.$4DB, G.$507, kill-all G.$50E, free play G.$46B, demo row G.$757A, player lives, HP words, score). An unknown address is an error, not a silent skip.
- **Original.** none (tool).
- **Data to extract.** none.
- **Files.** re/harness/record.lua, record.sh, port.sh; dino-game/src/harness.rs, main.rs (allowed).
- **Read.** NEW-PLAN s5 (recipes), capture-recipes s6 (fe-* recipes), D4d s13 and appendix A.
- **Check.** The story panel recipe (poke stage, sub 0, mode 8 at frame 1300) and the clock recipe (G.$4E8 = 0x0013) put equal values in MAME and in the port trace (G rows only, the screens come later).

#### M2-01 Config file for the factory settings

**1.0 h, X (external CLI), lane A. Needs: M1-04. From: T-TR7, P-D2b-7, D4b s4.**

- **Scope.** A plain `key = value` text file `dino.cfg` next to the ROM (no new crate) with the factory values: difficulty 3, timer level 1, bonus life option 2 (500k and 1000k), start lives 1 (2 in all), free play off, continue on, coin mode 0, chute mode 0, demo sound off, stereo on. Defaults when the file is missing. Feeds `Settings`, `rank.dip` and the sound filter. The EEPROM test menu itself is out of scope.
- **Original.** 0xAC794, 0xACBF4 (cooked bytes G.$777A, G.$777B, G.$777C).
- **Data to extract.** none.
- **Files.** game/config.rs (new), game/flow.rs (Settings), rank.rs (dip), main.rs (allowed: load the file).
- **Read.** D4b s4, D4d s4.3.
- **Check.** Unit tests: default file gives dip_b 0x4B and the 0x6BFE option 2. Changing difficulty in the file changes `rank_game_start` start rank in a test.

#### M2-02 System task: coins, credits, chute modes, credit text, SERVICE

**2.0 h, C (Claude worker), lane A. Needs: M2-01, M1-04. From: D4d T2.1.**

- **Scope.** All coin modes of D4d s4.2 (8 presets, data driven), the chute modes (3 players 3 chutes, 3 players 1 chute, 2 players 1 chute; the 2 player layout G.$4CF = 1 was not measured, keep the 3 player layout and note it), credit cap 9, free play (no press start screen, FREE PLAY text), credit text on title and select, SERVICE adds a credit to every player, start edge handling. Replaces the minimal task of M1-04.
- **Original.** 0x9D048 to 0x9D8EE (0x9D4BE, 0x9D386, 0x9D196, 0x9D326).
- **Data to extract.** coin preset tables (D4d s4.2, M1-02a).
- **Files.** game/flow/system.rs (new), flow/front.rs, input.rs (allowed hook for coin and start edges).
- **Read.** D4d s4, F36, s13 A10 to A12.
- **Check.** A10 (coin at 450, start at 600: credit at 460, G.$46D = 0xFF, mode 4 at 623, sub 2 at 625), A11 (no credit, nothing), A12 (free play poke: select at 523) equal MAME from power on.

#### M2-03 Character select, part 1: panels, records, cursors, 10 second timer

**1.5 h, C (Claude worker), lane A. Needs: M2-02, M1-02b. From: D4d T2.2 (part 1).**

- **Scope.** The select screen with the group picture, the three panels per player, the cursor order Jack, Hannah, Mustapha, Mess, no duplicate heroes, cursors skip taken heroes, the TIME counter (10 at 725, steps of 60 from 735), auto pick at 0 (610 frames after the screen starts), palette cycling sparkle (pool C 34 at 0xC290).
- **Original.** 0xAD46, 0xAF7C, 0xB06A, 0xB122, 0xB25E.
- **Data to extract.** records, panel texts, scroll 1 panel frames pages 3 to 6 (M1-02a).
- **Files.** game/flow/select.rs (new), effect.rs (class 34 hook), flow.rs.
- **Read.** D4d s5, s13 A13, F37; D3f class 34.
- **Check.** A13 sel-1p, sel-1p-moves, sel-same: TIME 10 at 725, timer steps, auto pick at +610, no duplicate heroes. Pictures of the select at 800 and 1200 equal MAME.

#### M2-04 Character select, part 2: stats diamonds, joins during select, end sequence

**1.5 h, C (Claude worker), lane A. Needs: M2-03. From: D4d T2.2 (part 2).**

- **Scope.** Stat diamonds (display only), JOIN-IN and INSERT COIN blinks in empty panels, a coin and start during the select (timer reset to 10), 2 and 3 player picks, the end sequence (+61, +62, +79, +139 to +158, mode 0xC at +159), pan values 0x00, 0x0B, 0x15, 0x20 for sounds 0x32 and 0x33 (recorded), the five select object classes (pool B 17, 19, 67, 68 and the star objects).
- **Original.** 0xB444, 0xB810 and the select objects.
- **Data to extract.** from M1-02a.
- **Files.** game/flow/select.rs, game/prop/ (B17, B19, B67, B68).
- **Read.** D4d s5, s6.5, s13 A13; D3e s8 (front end classes).
- **Check.** A13 sel-2p-late, sel-3p-all-left, sel-3p-timeout: timer reset 8 to 10 when P2 joins at 841; end offsets exact. A real game now starts from coin, press start, select with every hero (the Episode 1 opening is still the temporary one).

#### M2-05 Tile block writer and prop hit pass (T-B0, part 1)

**1.5 h, C (Claude worker), lane A. Needs: M2-02. From: T-B0.**

- **Scope.** The tile block writer API (layer, column, row, ring buffer rules of D3e 1.4) and the prop hit pass with the prop contract (+$44, +$48, +$6C) and the registered prop list. Hit rule from D3e part 3 2.6: a player attack box hits a prop when heroZ + 8 is at least the start of the box depth range and heroZ is at most its end, plus the X and height overlap of the box test.
- **Original.** 0x10CD0, list 0x6724, box test 0x128F8.
- **Data to extract.** none.
- **Files.** game/prop.rs, game/prop/tile_block.rs (new), combat.rs (main session: add the pass call, allowed).
- **Read.** D3e part 1 s1, s8 (T-B0); D3e part 3 s2.6.
- **Check.** Unit tests of the reach rule with the numbers of D3e part 3 (mine wall Z 32 hits, 16 misses; vent 824 hits, 816 misses). Tile writes show on screen in a debug spawn.

#### M2-06 Break sound helper, script action hook, generic block table driver (T-B0, part 2)

**1.5 h, C (Claude worker), lane A. Needs: M2-05. From: T-B0.**

- **Scope.** The break sound helper with pan, the hook so a script action can spawn a prop (for B11, B20 var 2, B21), and one generic block table driver (source block pointer table, destination, size, period) that the S sized tile animation props reuse. Record the sound requests (heard in M4).
- **Original.** 0x9E60 (break sound), 0x4BFA (player hit list), action hook of 0x1DBD6.
- **Data to extract.** block tables as the classes need them (later tasks).
- **Files.** game/prop/tile_block.rs, snd.rs, spawn.rs.
- **Read.** D3e part 1 s1, s8; D4e s4 (break sound toggles G.$43B).
- **Check.** Unit test: driver writes a block table at the right period. The break sound request code alternates with the G.$43B toggle as in D4e acceptance 9.

#### M2-07 Scripted control data extraction

**1.0 h, X (external CLI), lane A. Needs: M2-06. From: T-C1 (data part).**

- **Scope.** Extract the 26 scripts (table 0x1E6FC) with their 6 byte records and the tables: start ids 0x1DA64, end ids 0x1D9B0, boss area ids 0x1DAFC, end targets 0x1EE6A, start targets 0x1EEEA, one point table 0x1E3F4, three spots 0x1E532, lineup spots 0x1F10A, jump offsets 0x1DD88. The start table 0x4E56 is already in.
- **Original.** data only.
- **Data to extract.** all tables listed.
- **Files.** dino-extract/src/script.rs, dino-assets/src/script.rs (new), one line in each lib.rs (allowed).
- **Read.** D4c s5.3 to s5.5, s11 (T-C1).
- **Check.** Unit tests: 27 pointers, record counts per script equal the table of D4c 5.5, lengths documented. No ROM bytes in source.

#### M2-08 Scripted control, core: entry points, step, flags, movement actions

**2.0 h, C (Claude worker), lane A. Needs: M2-07, M2-02. From: T-C1 (part 1).**

- **Scope.** The three entry points (area start, area end, boss area), the per hero step with its +$C0 to +$DE fields, the flags G.$4DD, G.$4DE, G.$4F0, G.$4FD, G.$506, G.$514, and the walk code with its slot assignment; the actions that only move or wait: 0x00, 0x02, 0x08, 0x0A, 0x0E, 0x10, 0x12, 0x14, 0x18, 0x1C, 0x1E, 0x20, 0x26, 0x28, 0x2C, 0x2E, 0x30, 0x34, 0x36, 0x3C, 0x44, 0x48, 0x4A, 0x4C. Remove the TEMP shortcut `prop_area_walk_fallback` and `flow_player_scripts_ported`.
- **Original.** 0x1DA10, 0x1D95E, 0x1DAC4, 0x1DB0C, 0x1DB5E, 0x1DF10, 0x1DFCA, 0x1E07E, 0x1E13A, 0x1E170, 0x1E5F6.
- **Data to extract.** from M2-07.
- **Files.** game/script.rs (new), flow.rs, prop.rs, spawn.rs, player.rs (hook at 0x189D6).
- **Read.** D4c s5.3, s5.4, s5.5, s10 A16.
- **Check.** Unit test of D4c A16: each script run with a scripted hero reaches END in the frame counts of 5.5 (script 9: 91 to 93, script 25: 3). The area end walk of Episode 1 area 0 ends with GO instead of the old shortcut.

#### M2-09 Scripted player states and the actions that use them

**2.0 h, C (Claude worker), lane A. Needs: M2-08. From: T-C1 (part 2), P-D2a-6.**

- **Scope.** Player states 0x2A (two hops), 0x2C (pose with an effect, leader only), the vehicle wait 0x32, scripted jump 0x34, run state 6, and the actions that start them: 0x04 (pose, sound 0x18), 0x06, 0x0C (special), 0x16 (jump kick), 0x1A (fall), 0x32, 0x3A, 0x3E. The player update at 0x18A28 skips the hero state machine while G.$516 is set.
- **Original.** 0x19EA6, 0x19F40, 0x1E2EE, 0x1E200, 0x1E262, 0x1E54A, 0x1E5D6, 0x1DD4C, 0x1DD3A, 0x1E2AA, 0x1BBCA.
- **Data to extract.** tables 0x1DD88 (jump offsets).
- **Files.** game/script.rs, player.rs, player_combat.rs.
- **Read.** D4c s5.4, D2a s16, s18 (P-D2a-6).
- **Check.** Probe D4c A1 start of Episode 1: special attack state 0x0E from frame 615, G.$4E3 at 615 (needs M2-12 for the full scene).

#### M2-10 Caption banners (pool B 66)

**1.0 h, C (Claude worker), lane A. Needs: M1-02b, M2-06. From: B66.**

- **Scope.** The caption banner object: variant picks the string from table 0x87710, tall font at scroll 1 position, durations 80 to 120 frames. Strings of Episodes 1 to 8 (D3e part 3 2.7 lists vars 0x60 to 0x67). The anti tamper check at 0x86B3C is not ported.
- **Original.** 0x86D76, 0x86D2E, table 0x87710.
- **Data to extract.** string table and positions.
- **Files.** game/prop/b66.rs (new), dino-extract/src/frontend.rs (strings).
- **Read.** D3e part 1 s3, part 3 s2.7, D4d s9.2.
- **Check.** Opening of Episode 1: B66 vars 0x49, 0x54, 0x4A at t = 40, 162, 284 (delays 0, 122, 244), pictures equal MAME (D3e A8 segment).

#### M2-11 Episode 1 opening master (pool B 74) and cast A56

**1.5 h, C (Claude worker), lane A. Needs: M2-10, M2-08. From: B74.**

- **Scope.** The master that spawns the opening cast: bystander A54, class 56 (one flying cast member at X 0x4E0 Y 0x60), six thugs class 55 in two groups, sequenced with +$05. Classes 54, 55 and the birds 49, 50, 73 are already ported; class 56 and the master are not.
- **Original.** 0x89B48 to 0x89DA0 and the class 56 handler (read it with re/tools/d2).
- **Data to extract.** none new.
- **Files.** game/prop/b74.rs (new), game/enemy/scenery.rs (class 56).
- **Read.** D3e part 1 s3 (class 74), s8; D4c s6 (opening).
- **Check.** Harness `intro` scenario (vblank:1150) run with the master no longer loaded from MAME: the cast positions and states equal MAME for 600 frames.

#### M2-12 Scripted object actions, item waits, Episode 1 opening acceptance

**2.0 h, C (Claude worker), lane A. Needs: M2-09, M2-11. From: T-C1 (part 3).**

- **Scope.** The actions that spawn or wait on objects: 0x22 (caption), 0x24 (class 21 hook), 0x2A and 0x38 (floor item waits), 0x40 (door class 11 hook), 0x42 (Episode 8 clear sequence hook, filled in M3), 0x46 (forced GO), 0x4E (sound), 0x50 (fade). Consumers of the flags (`scripted`, `no_clamp`, `floor_only`) now get their setters. Run the Episode 1 opening from power on.
- **Original.** 0x1DED4, 0x1DECC, 0x1DE6C, 0x1DE86, 0x1DD30, 0x1E650, 0x1DD0E, 0x1DC28, 0x1DC38.
- **Data to extract.** none.
- **Files.** game/script.rs, prop.rs, combat.rs (flags).
- **Read.** D4c s5.4, s6, s10 A1, A2.
- **Check.** D4c A1: camera mode 0x11 and camX + 3 per frame to 896, mode 0 from 303, P1 stays at (1056, 0, 296), G.$506 = 1 from 3 to 614, G.$4E3 at 615, G.$4D5 = 0 at 618, G.$4DD = 0 at 648. Look at the opening from power on: scroll, cast, captions, HUD slide.

#### M2-13 Clock display (pool B 31, pool D slot 15) and beep

**1.5 h, C (Claude worker), lane A. Needs: M2-06. From: T-TR1, FX-P5, S5.**

- **Scope.** Pool D allocator and the slot 15 sprite list, the digit tiles from the BCD word G.$4E8, hidden above 3:00, 5 seconds at each half minute under 3:00, steady in the last 15 seconds, red under 1:00, beep code 0x34 on each second tick with the +$A2 window of D4e 4.2 (recorded, heard in M4).
- **Original.** 0x81046, 0x811CA, 0x81062, 0x8114E, template 0x81194, pool D 0x48A8 and 0x4994.
- **Data to extract.** template and digit font.
- **Files.** game/prop/b31.rs (new), draw.rs (pool D), hud.rs.
- **Read.** D4b s1, D3f s3.2, D4e s4.2.
- **Check.** Recipe `time 0:13` poke: red digits and the code 0x34 every second; D3f FX-S07 frames; pictures at 3:00, 2:30, 1:00, 0:15 equal MAME.

#### M2-14 TIME OVER words and the time top-up link

**1.0 h, C (Claude worker), lane A. Needs: M2-13. From: T-TR2, FX-P4 (class 28).**

- **Scope.** Pool C class 0x70: "TIME" from the right and "OVER" from the left at 4 px per frame meeting at X 0xC0, heroes die at t = 48 (already ported in `rank.rs`), the +3:30 clock and +600 wave frames for a respawning hero (0x10482) once per hero.
- **Original.** 0x96450, 0x96488, 0x103FE, 0x10482.
- **Data to extract.** the two raw sprite lists (asset pack).
- **Files.** game/effect.rs (class 28), rank.rs.
- **Read.** D4b s1, D3f class 28, D2b S12, S21.
- **Check.** D2b S12 (time over while invulnerable: death at t about 479) and S21 (clock 0:00 to 3:30 at the respawn). Words picture equal MAME.

#### M2-15 GO arrow and idle hint (pool B 20)

**1.0 h, C (Claude worker), lane A. Needs: M2-06, M2-08. From: T-C3.**

- **Scope.** The GO burst at the right edge (3 blinks of 30 frames on, 5 off) and the idle hint 180 frames after a lock ends, counter G.$4FE and G.$500, sound 0x3E, palette 10 of scroll 1, variants 0 and 2 (variant 2 forced by script action 0x46).
- **Original.** 0x7BB68.
- **Data to extract.** glyph cells.
- **Files.** game/prop/b20.rs (new).
- **Read.** D4c s4, s10 A7, T14.
- **Check.** D4c A7: lock released at t = 450; G.$4FE = 180 then 1 per frame; class 20 blinks from t = 630 for 105 frames; counter 600 afterwards; with R held the counter never reaches 0.

#### M2-16 Respawn air strike (pool B 18)

**1.5 h, C (Claude worker), lane A. Needs: M2-06. From: P-D2b-3.**

- **Scope.** The controller with the 5 bomb record list, start values (x0 = camera X + 0x40 after the terrain push, ground + 0x120), vx division by 57, depth per record, the explosions 59 frames after each bomb (the pool E side exists), the call from the respawn, shake and sound hooks.
- **Original.** 0x7B65E, 0x18C5E, 0x12CAC, 0x18B9C.
- **Data to extract.** record list.
- **Files.** game/prop/b18.rs (new), player.rs (call).
- **Read.** D2b s2, s9 (S2, S3, S5 to S8), D3e W30.
- **Check.** D2b S2: bombs spawn at R, R + 12 ... R + 48, landing X 1179, 1059, 1000, 1000, 1119, explosions at R + 59, 71, 83, 95, 107; S3 enemies lose 30 once; S8 other players unhurt.

#### M2-17 Respawn flash, bubble, 1UP marker, sparkles and the 1UP pose

**2.0 h, C (Claude worker), lane A. Needs: M2-16, M2-09. From: P-D2a-3, P-D2b-2, FX-P4.**

- **Scope.** Pool C class 14 variant 0 (flash and speech bubble "They're gonna pay for this!" or "Need a hand?" with the 1P/2P/3P marker) and variant 2 ("1UP!" for 180 frames), class 23 sparkles every 4th frame of +$EC, the 1UP pose states 0x18, 0x418, 0x41A with voice 0x9814 and the 180 frame blink.
- **Original.** 0x93B28, 0x93BF8, 0x958E4, 0x9590E, 0x6B98, 0x19996, 0x9814.
- **Data to extract.** bubble pictures (asset pack), text strings.
- **Files.** game/effect.rs, player.rs, hud.rs.
- **Read.** D2b s2, s6; D2a s11.1, s18; D3f classes 14, 23.
- **Check.** D2b S9, S11 (bubble, marker 1P from R + 300 to R + 420), S17 (1UP at the crossing frame, pose frame 23, marker 180 frames, sound 0x4A, rinv 180); D2a A17 pose lengths 38/83/110/75.

#### M2-18 Join, part 1: join task states 0 to 6 and prompts

**1.5 h, C (Claude worker), lane A. Needs: M2-04, M2-17. From: D4d T2.3 (part 1).**

- **Scope.** The join task per absent player, the in-level prompts (INSERT COIN and JOIN-IN, 48 frames on 80 off, 128 or 64 frame cycles), start edge with a credit (attack alone also joins with a credit in the three chute mode), state 2 at 20 and 4 at 61 of the P2 timeline.
- **Original.** 0x6C4A, 0x6D60, 0x6ED8 to 0x7580, 0x75B2.
- **Data to extract.** prompt strings (M1-02a).
- **Files.** game/flow/join.rs.
- **Read.** D4d s6.1, s6.2, F29, F40, P57.
- **Check.** D4d A14 join timeline (`10-17:2C`, `60-65:2S`): state 2 at 20, state 4 at 61, prompts per frame equal MAME.

#### M2-19 Join, part 2: quick select strip, drop in, clock and score effects

**1.5 h, C (Claude worker), lane A. Needs: M2-18. From: D4d T2.3 (part 2).**

- **Scope.** The quick select strip (steps every 40 frames, the strip starts on the player's previous hero), the pick, drop in at camera X + 0xC0 with the bazooka and HP 100 and 360 frames of blink (the item side exists), +3:30 stage time (cap 4:00), +600 wave frames, +1 point to all three scores, a hero is held only by players in strip or play.
- **Original.** 0x7AE4, 0x7BC8, 0x7D5C, 0x7D8E, 0x7E70, 0x7EE8, 0x7F86, 0x80A2 to 0x82E8, 0x1049E, 0x65A4.
- **Data to extract.** none.
- **Files.** game/flow/join.rs, item.rs (bazooka), rank.rs.
- **Read.** D4d s6.2, s6.5, P58, P59; D2b S9, S10.
- **Check.** A14: strip steps every 40 frames, pick, x 192, y 384, rinv 360, held bazooka, clock to 4:00, scores +1 (d4d-b19); D2b S9 equal.

#### M2-20 Continue, part 1: in-level countdown and the continue mode

**1.5 h, C (Claude worker), lane A. Needs: M2-19. From: D4d T2.4 (part 1).**

- **Scope.** The in-level countdown (state 8) when another player is alive, the continue mode 0xE9E6 sub 0, 2, 4, 6: villain picture on red, countdown 20 (100 frames) to 0 (60 frames each), skip by attack or jump from 17 (mode 0x10) or 18 (in level), taunts every 5 numbers, accept with coin and start, the quick select over the frozen level and its restore about 41 frames after the press.
- **Original.** 0x732E, 0xE9E6 to 0xF57C, 0xEE6E, 0x10490.
- **Data to extract.** taunt strings, countdown tables.
- **Files.** game/flow/cont.rs (new), flow/join.rs.
- **Read.** D4d s6.3, s6.4, F28, P60; asset-catalog N5 (continue pictures).
- **Check.** D4d A15: mode 0x10 at 203, numbers at 302, 362 ... 1502, mash result (beeps from 425, end 493), continue start at 1341 gives mode 0xC at 1382. Rank speed effect is M2-23.

#### M2-21 Continue, part 2: game over, effects, back to the attract loop

**2.0 h, C (Claude worker), lane A. Needs: M2-20. From: D4d T2.4 (part 2), FX-P8.**

- **Scope.** The red flash, GAME OVER for about 480 frames, mode 0x14 at 1985 and 0x18 at 1989, return to the attract loop in its saved state (F25), the continue effects: fuse spark (pool C 6), spark burst (16), countdown digits (17, 51 frames per digit, second beep 0x34). Confirm the ranking car question of M1-07 against 0xE9E6.
- **Original.** 0xAC06, 0xAC26, 0x92C9A, 0x93FFC, 0x94100.
- **Data to extract.** digit font.
- **Files.** game/flow/cont.rs, effect.rs, flow.rs.
- **Read.** D4d s6.4, s13 A15, A16; D3f classes 6, 16, 17.
- **Check.** A16 game over: mode 0x14 at 1985, 0x18 at 1989, ingame 0 at 1993, ranking after. Pictures of the continue screen at 300, 1000, 1500 equal MAME.

#### M2-22 Name entry and the ranking table update

**1.5 h, C (Claude worker), lane A. Needs: M2-21, M1-07. From: D4d T2.5, T-TR6.**

- **Scope.** Name entry state 0xA for a qualifying score (game over, and parked while the ending runs): 30 glyphs (A to Z, comma, point, RUB, END), 3 letters, 420 frame idle and 20 second limits, profanity filter, insert and sort into the 50 entry table, live rank text.
- **Original.** 0x15AE2 to 0x16270, 0x74A4, 0x161DA, 0x16260.
- **Data to extract.** glyph table, filter table.
- **Files.** game/flow/name_entry.rs (new), hud.rs.
- **Read.** D4d s7, s13 A17, A21, F31; D4b TR14.
- **Check.** A17: state 0xA at 1503, music request 0x1F at 1694, table write at 1992 with name CB, state 0 at 2147, final ranking picture equals MAME.

#### M2-23 Rank event wiring: join, continue, title wait

**1.0 h, C (Claude worker), lane A. Needs: M2-21, M1-04. From: T-TR3 (rest), X-rank-crosscheck.**

- **Scope.** Call the rank events that are still open: join (0x18410, speed +3 only in mode 0xC), continue (0x18566 with -2), and make sure G.$4F6 feeds `rank_game_start`. Also set the freeze G.$602 and scripted G.$4DD paths now that scripts run. The death and extra life calls are already in.
- **Original.** 0x18410, 0x18566, 0x1827A, 0x17F8A, 0x184E2, 0x1842C.
- **Data to extract.** none.
- **Files.** rank.rs, flow/join.rs, flow/system.rs.
- **Read.** X-rank-crosscheck (all), D4b s2, D2b s2.2.
- **Check.** D2b S1 rank drop numbers, S10 (continue with a coin); unit tests for each event. G.$4F6 band start rank equals MAME after 4000 frames of waiting with a credit.

#### M2-24 Clear tally, part 1: counts and panels

**1.5 h, C (Claude worker), lane A. Needs: M1-02b, M2-12. From: D4d T2.6 (part 1), T-TR5.**

- **Scope.** The clear task: the hero portrait and panels, vitality ((HP - 1) * 100, or 20000 at HP 100), enemies (kills * 100), final blow 20000, M.V.P. 10000 for the most damage (ties share), the TOTAL counting down 100 per frame into the score, end frame 177 + max_total / 100 + 268.
- **Original.** 0xB97E to about 0xC400 (count part), anti tamper 0xD860 not ported. D3e s8 names pool B classes 75 to 78 as flow objects: read what the tally and cards use and port those here.
- **Data to extract.** panel strings, hero portrait pages (M1-02a).
- **Files.** game/flow/tally.rs (new), flow.rs.
- **Read.** D4d s9.1, s13 A18, F30; D4b TR12.
- **Check.** D4d A18: score gains 30600, 20400, 10600 ... for the tv-hp and tv-k runs, MVP by damage for 2 and 3 players, pictures at t = 60, 100, 120, 160 equal MAME.

#### M2-25 Clear tally, part 2: map panel and the hand over

**1.5 h, C (Claude worker), lane A. Needs: M2-24. From: D4d T2.6 (part 2).**

- **Scope.** The map panel that slides in at the end of the tally (marker blinks 120 frames), the hand over to the story panel, the Episode 8 path to mode 0x1C (stub until M3 ending), tally sound requests.
- **Original.** 0xC400 to 0xCCFE, the map panel object pool B 58.
- **Data to extract.** map picture pages, marker positions.
- **Files.** game/flow/tally.rs, game/flow/cards.rs (new).
- **Read.** D4d s9.1, s9.2, F38; D4c s8.
- **Check.** D4d A19 link scenes: panel and map frames of Episode 1 to 2 equal MAME (b9 link-ep2).

#### M2-26 Intro cards: map card and typed episode title (states 0 to 2)

**1.5 h, C (Claude worker), lane A. Needs: M2-25. From: D4d T2.7 (part 1).**

- **Scope.** Intro task states 0 to 2 for Episodes 2 to 8: the map panel slides in, typed "EPISODE n" and the title (141 frames), slides out, then the HUD slide in. Titles: The Swamp Forest, Hell Road, Jack's Garage, Village of Flame, Jungle and Mine, The Vault, The Under World.
- **Original.** 0xB9BE (states 0 to 2), 0x84736, 0x7EC20.
- **Data to extract.** title strings, map positions per episode.
- **Files.** game/flow/cards.rs, flow.rs.
- **Read.** D4d s9.2, F38, s13 A19; D4c T15.
- **Check.** D4c A2: G.$4E2 = 1 at 3, G.$4D5 = 1 at 231, G.$4E2 = 0 at 255 for Episodes 3, 5 to 8; card pictures of Episodes 2 to 8 equal MAME.

#### M2-27 Intro handshake G.$4E2 and G.$4E3, intro states 4 and 6, HUD slide limit

**1.5 h, C (Claude worker), lane A. Needs: M2-26, M2-12. From: T-C2.**

- **Scope.** The gates between card, script and player control, intro states 4 and 6 exactly, the HUD slide limit by stage (0x180 or 0x3F0), control frames 648, 573, 321, 894 by episode.
- **Original.** intro task in 0xB9BE states 4 and 6, 0x1DE44, 0x1DCFE, 0x1DD2A.
- **Data to extract.** none.
- **Files.** game/flow.rs, script.rs.
- **Read.** D4c s6, s10 A1 to A3; T15.
- **Check.** D4c A2 (Episodes 3, 5, 6, 7, 8: G.$4E3 at 318, G.$4DD = 0 at 321, P1 X -48 to 49 by 320) and A3 (Episode 2 control at 573, Episode 4 at 894 once their actors exist in M3).

#### M2-28 Story panels (mode 8, seven entries)

**1.5 h, C (Claude worker), lane A. Needs: M2-26. From: D4d T2.7 (part 2).**

- **Scope.** The seven mode 8 entries for Episodes 2 to 8: black screen, small picture block, two or three lines, panel lengths 306, 250, 308, 439, 368, 368, 313, speaker and hero names by party (Episodes 4 and 7 first unused hero, 4 and 5 first player's name, 6 first player's portrait), attack skips, the story tunes 0x13 to 0x16 recorded.
- **Original.** 0xD898, 0xD8E8, 0xD97E, 0xDAB4, 0xDBB2, 0xDDCC, 0xDE6C, 0xDF1E, 0xE034, 0xE0AA, 0xDFCC.
- **Data to extract.** texts, picture lists (0xD94E.., 0xDF78..).
- **Files.** game/flow/cards.rs.
- **Read.** D4d s9.3, F41, s13 A22; D4d open items 4, 12.
- **Check.** D4d A19 panel lengths and A22: portrait and name per party for Episodes 4 to 7, skip at attack (mode 4 at 64), pictures at 100, 250, 320 equal MAME. Whole link chain tally, map, panel, card, opening for Episode 2 runs.

#### M2-29 HUD weapon display and item messages

**2.0 h, C (Claude worker), lane A. Needs: M2-02. From: P-D2c-5, IT-P4.**

- **Scope.** The weapon HUD block per player (picture tile, ammo number red at 1, name, cartridge bar for the 6 guns; ARM tile and name for the other 9 types), three blocks side by side; item messages (FOOD icon and Pts icon, queue, 60 frame timer, treasure 17 quirk kept).
- **Original.** 0x2389C to 0x23EDA, names 0x23D0B, pool B 100 (0x7E72C) and pool D slot messages.
- **Data to extract.** name table, icons.
- **Files.** game/hud.rs.
- **Read.** D2c s12, s20; D4a s11 A22; D3f s3.3.
- **Check.** D2c W15 (6 guns and 9 others, 3 player blocks) and D4a A22 pictures (message absent at t = 16, present from 22, second message about 90) equal MAME.

#### M2-30 Harness: 2 and 3 player scenarios M1 to M10

**2.0 h, X (external CLI), lane B. Needs: M2-00. From: P-D2c-7.**

- **Scope.** Record the 2 and 3 player scenarios of D2c M1 to M10 (friendly fire windows, tag team ties, contact, boss HP, player count, same frame pick up, dive window) as harness scripts and MAME traces. Boot of 2 and 3 players exists (`boot_2p`, `boot_3p`). Prepare, do not fix.
- **Original.** none.
- **Data to extract.** none.
- **Files.** re/harness/scripts/, re/harness/pokes/, re/harness/README.md.
- **Read.** D2c s19 (M1 to M10), re/harness/README.md (multiplayer section).
- **Check.** Every scenario has a .mame.trc and runs in `port.sh` without a panic (differences are listed, they become fixes in M5-20 and M9-01, M9-02).

#### M2-31 Flow scenario scripts for D4d and D4c

**2.0 h, X (external CLI), lane B. Needs: M2-00. From: T-C10 (part), D4d s13.**

- **Scope.** Power-on scripts and poke files for the D4d acceptance A10 to A22 and the D4c A1, A2, A7, A8, A9 recipes, recorded in MAME once. They are the ground truth for the M2 acceptance and later M3.
- **Original.** none.
- **Data to extract.** none.
- **Files.** re/harness/scripts/flow_*.txt, re/harness/pokes/ (timed poke files), re/harness/README.md.
- **Read.** D4d s13 and appendix A, D4c s10, capture-recipes s6 (fe-* recipes), batch-mame-runs rule.
- **Check.** Each flow scenario has a MAME trace; one MAME process per batch (headless, SDL dummy).

#### M2-32 M2 acceptance: from coin to the Episode 2 card

**2.0 h, C (Claude worker), lane A. Needs: M2-28, M2-29, M2-23, M2-22, M2-15, M2-31. From: D4d s13, D4c s10.**

- **Scope.** Run the flow scenarios A10 to A22 and D4c A1, A2, A7, A8 against MAME with M1-00 and M2-00, list the differences in `re/harness/allow/m2.txt`, fix code mistakes. Play from power on: coin, start, select, opening, fight the first wave with the clock, die, continue, game over, name entry.
- **Original.** all of M2.
- **Data to extract.** none.
- **Files.** re/harness/allow/m2.txt, M2 files.
- **Read.** D4d s13, D4c s10.
- **Check.** No unexplained difference in the flow frames listed in the scenarios. Screenshots of every screen looked at by the main session.

### M3: The whole game skeleton

From power on to the ending through all 8 episodes: every area change, opening, link scene, map card, story panel, boss gate and the ending. Enemy classes that are not ported use placeholders (a registry lists them, every real port removes its entry). Gating props (walls and fences that block walking) are real.

Hours: 51.1 (lane A 47.6, lane B 3.5), 30 tasks.

#### M3-01 Placeholder enemy for every unported pool A class, and the registry

**2.0 h, C (Claude worker), lane A. Needs: M2-12. From: T3.6 (NEW-PLAN), E24.**

- **Scope.** Replace `enemy_class_unported` (frees at once) by a generic placeholder that keeps the wave, lock, GO, clock and drop flow alive. It walks to the nearest hero at 1.5 px per frame, hits for 10 every 90 frames in reach, takes player hits through the normal enemy hit pass (HP 44, flinch, flash), dies with a 30 frame blink, scores by the name table and drops by `+$96`. The census flag follows the real class: counted in waves (+$7D = 0) except classes 27, 43, 65, 66 (not counted) and the dinosaurs 22, 23, 24, 29, 38, 82 (+$7D = 0xFF, harmless, freed at their leave X). Hazard classes 65, 66, 70 cross the screen harmlessly. Drawn as a labelled box with the class number. A registry `game/placeholder.rs` lists the classes served by placeholders, `--list-placeholders` prints them, and every later real port deletes its entry.
- **Original.** census 0x3242E, hit pass 0x115B0 and following, kill score 0x325F8, drop 0x24B36.
- **Data to extract.** per class census flag and name id (from D3a s1, s13 and D3b s9).
- **Files.** game/placeholder.rs (new), enemy.rs (dispatch), present.rs (debug box).
- **Read.** D3a s1, s3; D3b s1, s9; README s1.
- **Check.** In Episode 1 area 1 and Episode 2 area 0 (no ported class there), waves start, lock, end and release with the GO arrow; kill all with the debug key (M3-05) and the wave ends; placeholder kills give the name table score. `--list-placeholders` output matches the class list of `stage_scripts.txt`.

#### M3-02 Boss services, part 1: reinforcement controller lists and class 22

**2.0 h, C (Claude worker), lane A. Needs: M3-01. From: P-D3c-1 (part 1), E25, BS27.**

- **Scope.** Extract the controller lists (timed and threshold lists, stages 0 to 7) and port pool B class 22: both spawners, the helper limit 4, 5, 6 by player count, `G.$502` (Vice summon, Slice and Hogg 300 frames, Butcher 30 frames), helper drop rows (IT23, IT-P8).
- **Original.** 0x7BEF6 and the spawners, G.$502.
- **Data to extract.** controller lists (`b22_lists` format in D3c s1.7), helper drop rows.
- **Files.** dino-extract/src/boss.rs, dino-assets/src/boss.rs (new, one line in each lib.rs allowed), game/boss.rs (new), prop.rs.
- **Read.** D3c s1.7, s9 (P-D3c-1), BS27; D4a IT23.
- **Check.** Unit tests of the list decode against the group order of D3c 1.7. A placeholder boss (M3-04) summons its group at the right threshold. D3c BS-V2 summon timeline is checked later in M5.

#### M3-03 Boss services, part 2: name, HP bar, death hooks, G.$4FC and the end chain

**2.0 h, C (Claude worker), lane A. Needs: M3-02, M2-12. From: P-D3c-1 (part 2), T-C4.**

- **Scope.** The boss name and HP bar in the HUD, the damage credit and clamp hooks, the music fade on the death (0xA5EA, recorded), `G.$4FC` (boss intro running), the common reaction entry, and the death chain: class 22 calls 0x1D95E at boss HP 0, slow motion, `G.$4D1` steps checked against D4c A8, hand off to the clear task. Remove the "boss defeat" test shortcut if one exists.
- **Original.** 0x3262A, 0x32740, 0xA5EA, 0xACEE, 0x7C0D8, 0x1D95E.
- **Data to extract.** none.
- **Files.** game/boss.rs, prop.rs, flow.rs.
- **Read.** D3c s1, s9; D4c s8, s10 A8, A9; T19, T20.
- **Check.** D4c A8 on a placeholder: HP 0 at 0, script 12 at +3, normal speed at +94, G.$4E3 at +372, tally 944 to 951, mode 8 at the right frame. The walk off after the boss (script 3) runs.

#### M3-04 Placeholder bosses and starters

**2.0 h, C (Claude worker), lane A. Needs: M3-03. From: T3.6 (bosses).**

- **Scope.** One placeholder boss per episode on the real entry classes: 18 var 0 (Vice), 5 (Butcher), 31 (Hogg), 25 (Slice), 35 then a placeholder Morgue 40 (Morgan then Morgue), 4 (Tyrog, spawned by a placeholder of starter B79), 71 times 2 or 3 (spawned by a placeholder of B33), 94 (Fessenden). HP rows from the existing `boss_hp` data, name and bar, walks and hits for 12 every 90 frames, death calls 0xACEE. Episode 8 sets G.$4D7 and G.$4DB at the death so the flow reaches mode 0x1C.
- **Original.** 0xACEE, boss HP row 0x31E14.
- **Data to extract.** none new.
- **Files.** game/placeholder.rs, boss.rs.
- **Read.** D3c s0, D3d s1, s7; D4c s8; M3-03.
- **Check.** Every episode ends: boss placeholder dies, tally, story panel, card, next episode (Episode 8: ending stub). D3c BS-A1 boss HP values at level 12 (Vice 500 / 600 / 700, Butcher 576 / 640 / 704, Hogg 300 / 400 / 500, Slice 500 / 600 / 750).

#### M3-05 Review aids: `--episode --area --camx`, debug keys, autoplay bot

**2.0 h, X (external CLI), lane A. Needs: M3-01. From: T3.9.**

- **Scope.** `--episode N --area A [--camx X] [--chars ..]` starts any area (spawn records before X are skipped like the probe `jump`). Debug keys (kept off by default): kill all (kill-all flag 2), skip wave, set time, god mode, next area. `--autoplay` is a bot that holds right, presses attack, uses kill all every N frames so the whole game runs from power on to the ending in minutes, and prints the episode, area and frame of every mode change.
- **Original.** none (tool); kill-all G.$50E.
- **Data to extract.** none.
- **Files.** main.rs, input.rs, game/flow.rs (main session files: allowed for this task), game/mod.rs.
- **Read.** NEW-PLAN s6 (T3.9), capture-recipes s2 (jump).
- **Check.** `dino-game --episode 3 --area 0 --camx 0x300` starts there; `--autoplay` from power on reaches the select and the Episode 1 start; the report prints mode changes.

#### M3-06 Harness: probe segments as scenarios (snapshot and trace commands)

**1.5 h, C (Claude worker), lane B. Needs: M2-00. From: new (HX, MAME side).**

- **Scope.** Add `snapshot` and `trace N` commands to `probe.lua`: the segment lines before `snapshot` set up the scene (jump, pokes, spawn, hold, god), the snapshot is written in the harness format, and `trace N` records N frames of harness records (objects, globals, video, sound). The key lines after the snapshot become the input script. `probe_record.sh BATCH SEGMENT` writes `.state` and `.mame.trc`. So every probe recipe of D2 to D4 becomes a harness scenario.
- **Original.** none (tool).
- **Data to extract.** none.
- **Files.** re/harness/probe/probe.lua, probe_record.sh (new), re/harness/record.lua (share the trace writer), re/harness/README.md.
- **Read.** capture-recipes s2 to s5, re/harness/README.md, re/harness/record.lua.
- **Check.** Segment `jump 1 0 897 char=jack` + `snapshot` + 300 frames of `trace` equals the existing `stage1_jack` trace for the same input (diff 0).

#### M3-07 Harness: load a snapshot of any episode and area in the port

**2.0 h, C (Claude worker), lane A. Needs: M3-06, M3-05. From: new (HX, port side).**

- **Scope.** Generalize the snapshot loader of the harness so the port starts any episode and area from a MAME snapshot: stage, area, camera, spawn list position, wave controller state, rank, time, pools, players. Probe jumps keep wave controllers before X (D4c open item 9): load them as they are.
- **Original.** none (tool).
- **Data to extract.** none.
- **Files.** dino-game/src/harness.rs, game/trace.rs, flow.rs.
- **Read.** re/harness/README.md (sync points), capture-recipes s2 and s8, D4c s4.3 warning.
- **Check.** `port.sh` runs a snapshot of Episode 2 area 0 camera 0x400 (placeholders for unported classes) without a panic and the global and camera fields match MAME for 600 frames in `step` mode.

#### M3-08 Episode 3 camera controller, part 1: modes, foot locks, row of events

**1.5 h, C (Claude worker), lane A. Needs: M2-12. From: T-C5 (part 1).**

- **Scope.** The stage 2 controller with its own mode table and event table: 10 events at X 0x300 to 0x2D00 are locks (mode 2) of 32 game frames each on foot, then mode 0; end of the road to mode 3.
- **Original.** 0x22398, mode table 0x2250A, events 0x22B9E.
- **Data to extract.** event table, mode table (check what `camera.rs` already extracted).
- **Files.** game/camera.rs, dino-extract/src/stage.rs.
- **Read.** D4c s2.3, s7, s10 A11; D3e part 2 s3 (Episode 3 area 0).
- **Check.** D4c A11 (locks at camX 0x300 and 0x500 last 32 game frames, mode 2 then 0) from the HX scenario `jump 3 0` with three heroes.

#### M3-09 Episode 3 camera controller, part 2: mode 8 chase and the radio stand in

**1.5 h, C (Claude worker), lane A. Needs: M3-08. From: T-C5 (part 2).**

- **Scope.** Camera mode 8 chase (camX += clamp(car X - (camX + 0xC0), 8, 20)), `G.$518` hooks, the end of the car section to mode 3. The radio pick up in M3 is a no-op that only sets the treasure score (the real call is M6); no car object yet.
- **Original.** 0x22398 (mode 8), 0x8BD24 (stub only).
- **Data to extract.** none.
- **Files.** game/camera.rs, item/small.rs (radio no-op).
- **Read.** D4c s7.1, T21, T22; D3c s5.
- **Check.** A unit test of the mode 8 formula with the BS-C2 numbers (11.0 px per frame at the right stick). The foot road of Episode 3 runs end to end with placeholders.

#### M3-10 Screen shake

**1.5 h, C (Claude worker), lane A. Needs: M2-12. From: T-C6.**

- **Scope.** The shake state machine (offsets -s, +s (3 frames), ... +1, length 6 s - 2) with the scroll 2 offset (scroll 1 in Episode 7 area 1) and the missing player requests (landing 1, slam 3); 29 requesters.
- **Original.** 0x222D6.
- **Data to extract.** none.
- **Files.** game/camera.rs, present.rs, player.rs.
- **Read.** D4c s7.3, T25, s10 A14.
- **Check.** D4c A14: for S = 1 to 4 the offsets of 6 S - 2 frames equal MAME (watch of G camy).

#### M3-11 Row scroll task and table (Episode 3 road)

**1.5 h, C (Claude worker), lane A. Needs: M3-08. From: T-C7 (part 1).**

- **Scope.** The row scroll task and its 32 word table from the stage spec formula, the per line offset values per frame.
- **Original.** 0x2335A.
- **Data to extract.** none (formula).
- **Files.** game/camera.rs.
- **Read.** D4c s7.2, T24, s10 A13; re/specs/stage/SPEC.md (row scroll).
- **Check.** D4c A13: 32 row scroll words at camera 0x300, 0x1000, 0x2400 equal MAME peeks.

#### M3-12 Per line scroll 2 offset in the renderer

**1.5 h, C (Claude worker), lane A. Needs: M3-11. From: T-C7 (part 2).**

- **Scope.** Render the road with a per line scroll 2 offset (render.rs and present.rs are main session files: allowed for this task). Keep the normal path unchanged.
- **Original.** none (renderer).
- **Data to extract.** none.
- **Files.** render.rs, present.rs, game/camera.rs.
- **Read.** D4c s7.2.
- **Check.** Pictures of the Episode 3 road at camera 0x300, 0x1000, 0x2400 equal MAME (H-02), regress.sh not worse (other stages unchanged).

#### M3-13 Lift (Episode 7 area 1), part 1: cage background, mode 5, stops

**1.3 h, C (Claude worker), lane A. Needs: M3-10, M2-12. From: T-C8 (part 1).**

- **Scope.** Scroll 1 cage background, camera mode 5 from camera Y 0x3700 to 0, stops at Y 0x3000, 0x1E00, 0x1400, 0x800 and the end lock, restart from rest after each lock (acceleration 6/256 per frame squared, up to 6 px per frame), lift sound 0x86 recorded, shake 2.
- **Original.** 0x205C6, tables 0x138C48 and 0x15CFE8.
- **Data to extract.** tables (check extraction).
- **Files.** game/camera.rs, dino-extract/src/stage.rs.
- **Read.** D4c s7.2, T23, s10 A12; D3e part 3 s2.5, s4.2.
- **Check.** D4c A12: stops at camY 0x3000, 0x1E00, 0x1400, 0x800 and the end lock at the gf offsets (532, 2330, 3398, 4838, 5788) of the HX scenario `jump 7 1` with three heroes.

#### M3-14 Lift, part 2: floor labels B57, cage posts B64, by-Y spawns

**1.3 h, C (Claude worker), lane A. Needs: M3-13, M2-06. From: T-C8 (part 2), B57, B64.**

- **Scope.** Pool B 57 paints the 56 floor labels as the camera falls (label k at camera Y 0x3760 - 0x100 k), class 64 draws the four cage posts (compositions 1897 to 1900), spawn triggers by Y, the Z + Y off-screen test for the riders (E14).
- **Original.** 0x85B64, 0x86B64.
- **Data to extract.** label glyph table 0x85BF6, glyphs 0x85D6C.
- **Files.** game/prop/b57.rs, game/prop/b64.rs (new), spawn.rs, enemy.rs.
- **Read.** D3e part 3 s2.5, s4.2, A14; D4c s7.2.
- **Check.** D3e part 3 A14: B57 at t = 0 with y 13920, B64 x 4 with the listed positions; pictures at the first and third stop equal MAME (labels L15 to L1, then B1 and so on).

#### M3-15 Area change details: brightness dip, class 96 music object, time and rank per area

**1.5 h, C (Claude worker), lane A. Needs: M2-12. From: T-C9, B96, S7.**

- **Scope.** The hard cut of mode 0x12 (brightness 15 to 0 for 2 to 4 black frames), class 96 music object after mode 8 (fade, then 59 frames later the music byte), new area time (2:20, 1:20, 3:20, 1:20, 4:00, 2:50, 4:00, 4:00, 1:20, 2:50, 4:00, 4:00, 4:00) and rank per area, `G.$514` heavy object rule.
- **Original.** mode 0x12, 0x92382 (class 96), 0xAA0E, 0xACD0, 0xACE0.
- **Data to extract.** none.
- **Files.** game/camera.rs, prop.rs (B96), flow.rs.
- **Read.** D4c s5.2, s5.7, s10 A4; D4e s1.3, s9 (3).
- **Check.** D4c A4 on the 13 area changes: end script start, lock, go, area load and control frames, fade lengths, new area times equal MAME; Episode 1 area 1 to 2 hard cut frames equal.

#### M3-16 Shared services for the pool B tile and palette props (T3-0)

**1.5 h, C (Claude worker), lane A. Needs: M2-06. From: D3e part 3 T3-0.**

- **Scope.** (1) A palette cycler driver: lists of colour sets, mask 0xF000 kept, period, shared by B60, B61, B63, B71 and the lightning tables; (2) the tile clock function 7AE = ((gf + 60) / 24 mod 4) * 4 with G.$7AF one frame later; (3) a burst driver for B4, B42, B52 and B11, B12, B14 (wait 1 frame, swap a tile block from a table, effect, sound, free); (4) the hero reach rule in the prop hit pass if M2-05 did not finish it.
- **Original.** B60 0x863AC, tile clock readers 0x7862A and others, burst classes 0x767D8, 0x83A0A, 0x85178.
- **Data to extract.** none yet (tables come with each class).
- **Files.** game/prop/drivers.rs (new), prop.rs.
- **Read.** D3e part 3 s2.2, s2.8, s2.9, s7(b) T3-0.
- **Check.** Unit tests: tile clock values at gf 1284, 1308, 1332, 1356, 1380 (0, 4, 8, 12, 0); palette cycler 36 frame cycle of B60 (A06 numbers).

#### M3-17 Episode 1 transition props: door smash B11, hole and wooden door B14, tile change B21

**1.2 h, C (Claude worker), lane A. Needs: M3-16, M2-12. From: B11, B14, B21.**

- **Scope.** B11 (metal end door of area 0 smashed on the scripted walk, dust, black doorway, script action 0x40), B14 (building hole var 0 at the start of area 2, wooden door var 4), B21 (one-shot tile attribute change, script action 0x24, Episode 1 area 1 start).
- **Original.** 0x789EE, 0x789C8, 0x7B0BE, 0x7BDC4, 0x1DECC, 0x1DD30.
- **Data to extract.** tile block tables.
- **Files.** game/prop/b11.rs, b14.rs, b21.rs (new).
- **Read.** D3e part 1 s3, s5 (Episode 1), A4 to A6.
- **Check.** D3e A4 (class 11 at (1648, 0, 280) near t = 980, doorway from 1000, area 1 at 1100) and A6 (hole picture at t = 1830, door burst near 2010 to 2060) on the HX scenarios.

#### M3-18 Episode 1 hall door and window (B12)

**1.0 h, C (Claude worker), lane A. Needs: M3-17. From: B12.**

- **Scope.** Hall double door (var 0) black by t = 170, hall window (var 4, armed by G.$506 during the scripted jump kick) shatters at t = 1780 to 1790 (state 0x06000000, position (906, 96, 48)).
- **Original.** 0x78B52.
- **Data to extract.** tile block tables.
- **Files.** game/prop/b12.rs (new).
- **Read.** D3e part 1 s3, s5, A5.
- **Check.** D3e A5: class 12 var 0 at t = 160, var 4 at t = 1020 with state 0x02000000 from 1040, window break at 1780, area 2 at 1800. Look at the Episode 1 area 1 end walk.

#### M3-19 Gates: mine wall B3 and chain-link fence B1

**2.0 h, C (Claude worker), lane A. Needs: M3-16, M2-12. From: B3, B1.**

- **Scope.** B3 var 0 (solid wall at X 0x350 until broken, 1 hit, HP 8, 31 code and 37 attribute words) and var 8 (mine exit wall breaks by itself 48 frames after the area load); B1 (fence sprites 326 and 327, HP 1, any player attack box breaks it, 48 attribute words for var 4, 3 for var 8, the leader of end script 19 breaks var 8; Episode 4 garage gate uses the same class). The attribute rewrite removes the wall bits the terrain reads.
- **Original.** 0x763F6, 0x75238.
- **Data to extract.** tile blocks, attribute lists.
- **Files.** game/prop/b3.rs, b1.rs (new).
- **Read.** D3e part 3 s2.6, A01 to A04, A16, A17; part 2 s5, A211; W301, W312.
- **Check.** D3e part 3 A01 to A03 (hit at t = 22 for Z 32 to 79, miss at Z 16, a walking hero stops at X 846) and A16 (fence state 04000000 at t = 22, 48 attribute words, no code word); A17 gate 2 timeline.

#### M3-20 Scene actors: Cadillac A16 and A17, Episode 4 opening A27 and A43

**2.0 h, C (Claude worker), lane A. Needs: M2-10, M2-12. From: HT10, HU18, HU19.**

- **Scope.** Pool A 16 and 17 (the blue Cadillac and its two wheels that arrives in Episode 2 and crashes the Episode 4 gate, carries up to three heroes, `G.$504`, brakes from camera X + 0xA0, stops at x 192) and pool A 27 and 43 (three crouching men, the orange leader with three banners, end flag `G.$4FC`, `G.$505`). Captions by B66.
- **Original.** 0x420EA, 0x423A4, 0x4D900, 0x55F2A.
- **Data to extract.** none new (action lists in the handlers).
- **Files.** game/enemy/scene_actors.rs (new).
- **Read.** D3a s11, H22, H23; D4c s6, A3; D3e part 2 s4, A207.
- **Check.** D3a H22 (class 27 at x 274, 188, 234, class 43 at 352; car A16 at x -98 at t = 600 to 482 at 700) and H23 (car invisible at x -128 until t = 280, x 97 at 300, x 192 sub 6 at 350); D4c A3: Episode 2 control at 573 and Episode 4 at 894.

#### M3-21 Episode 8 lab end: manhole B35, hole B47, hero pose 0x1F

**1.5 h, C (Claude worker), lane A. Needs: M3-16, M2-09. From: B35, B47.**

- **Scope.** The leader lifts a round floor grate (pool B 35 made by 0x81FB0 from the hero pose routine, hero anim 0x1F), throws it right, jumps into the manhole; B47 writes the two scroll 2 patches when G.$4F0 is set. Script 21.
- **Original.** 0x81FDE, 0x81FB0, 0x19F8A, 0x84282 (a second copy of the pose routine is at 0x2864).
- **Data to extract.** patch blocks 0x90F86C, 0x90F872.
- **Files.** game/prop/b35.rs, b47.rs (new), player.rs.
- **Read.** D3e part 3 s5.1, W318, A25; D4c script 21.
- **Check.** D3e part 3 A25: B35 at (1591, -9, 55) at t = 710, thrown at 790 to (1680, 24) by 830, down at 870, B47 at 710, area 1 at t = 960.

#### M3-22 Episode 8 sewer end: waterfall B36, B37, B38

**1.8 h, C (Claude worker), lane A. Needs: M3-16. From: B36, B37, B38.**

- **Scope.** The waterfall over the edge (B36 two blocks), the shaft water strips (B37) and the 80 tile blocks per step (B38), all on the tile clock, spawned by end script 22; the slow fade and the cave start.
- **Original.** 0x8217E, 0x82372, 0x8279C.
- **Data to extract.** block tables (0x90C038 and others).
- **Files.** game/prop/b36_38.rs (new).
- **Read.** D3e part 3 s5.2, A26.
- **Check.** D3e part 3 A26: B36 var 0 at (3104, 0, 768) at t = 452, B37 and B38 at t = 508, camera mode 9 at 530, area 2 at 550 with the cave caption at 540. Pictures of the shaft equal MAME.

#### M3-23 Ending, part 1: lab corridor scene and the explosion chain

**2.0 h, C (Claude worker), lane A. Needs: M2-10, M3-16, M3-10. From: D4d T3.7 (scene 1), B82, B83, B84, B85.**

- **Scope.** Mode 0x1C scene 1 (t 3 to 480): master pool B 82, the corridor with two heroes running right, caption HURRY UP!! (t 10 to 130) and HANNAH! (t 330), the explosion chain from t 7 (pool A classes 90 to 93 and 96, pool C 21, sound codes recorded), debris showers B83 (200 frames) and B84 (400 frames, A93 pieces every 32 frames), quake B85. The chain uses the random generator, so the same state gives the same explosions. No design document names the pool A classes 90 to 93 and 96: read the code and write a short note in re/specs/flow/.
- **Original.** 0xA69B8, 0x8AA0A, 0x8B212, 0x8B2CA, 0x8B34C.
- **Data to extract.** scene tables (read by the code).
- **Files.** game/flow/ending.rs (new), game/enemy/ (classes 90 to 93, 96), game/prop/b82_85.rs (new).
- **Read.** D4d s8; D3e part 3 s5.5, A31; W323.
- **Check.** D4d A20 first 700 frames: sub 4 at 3, cuts at 10 and 660, pictures every 100 frames equal MAME (explosions equal when the RNG state equals, 3 players differ by design).

#### M3-24 Ending, part 2: outside at sunset and the still picture

**1.5 h, C (Claude worker), lane A. Needs: M3-23. From: D4d T3.7 (scene 2).**

- **Scope.** t 480 to 975: the sunset outside (tree, sea, skyline, explosion flashes, effect class 21 at 531 and 551, pool B 3, 83, 85, 66 captions, sound 0x56 recorded), the still picture with Mess and Mustapha (class 87 at 776, caption JACK! HANNAH!), black frame at 970.
- **Original.** part of 0xA69B8 and the master 82; class 87.
- **Data to extract.** scroll 3 pages 0x6C, 0x78 to 0x7C, scroll 2 page 0x16.
- **Files.** game/flow/ending.rs, game/prop/ (B87).
- **Read.** D4d s8, A20.
- **Check.** Pictures at 500, 600, 700, 900 equal MAME; cut at 970 to 980.

#### M3-25 Ending, part 3: desert, walkers and the staff roll

**2.0 h, C (Claude worker), lane A. Needs: M3-24. From: D4d T3.7 (scene 3), B81.**

- **Scope.** Pool B 81 starts at 975 (music 0x19): the two walkers (pool A 81 from 976), the scrolling desert, the staff roll as vertical blocks of lines moving with the scroll (53 string lines from 0x8B7EA, red header and blue lines, extracted at run time from the user's ROM, not copied).
- **Original.** 0x8B7EA to 0x8BB04 (strings), class 81 and class 81 walkers.
- **Data to extract.** staff strings and block order (open item 5 of D4d: read the code).
- **Files.** game/flow/ending.rs, game/prop/ (B81), dino-extract/src/frontend.rs.
- **Read.** D4d s8, open item 5, A20; asset-catalog N5 (desert pages).
- **Check.** Pictures every 100 frames from 980 to 3900 equal MAME; the staff blocks and scroll speed equal.

#### M3-26 Ending, part 4: the Cadillac, dialogue, THANK YOU and PRODUCED BY

**2.0 h, C (Claude worker), lane A. Needs: M3-25, M3-20. From: D4d T3.7 (scene 4 and 5).**

- **Scope.** From 3956: music 0x1A, pool A 17 twice and the car with Hannah and Jack rolling in and stopping, the portraits with the typed captions at 3980, 4290, 4400, 4680, 4830, "THANK YOU FOR PLAYING" typed, "PRODUCED BY" and the CAPCOM sign, the car drives off (timer ends at 5364).
- **Original.** pool B 81 later states, pool A 17 and 81, typing helper 0x84736.
- **Data to extract.** caption strings (M1-02a).
- **Files.** game/flow/ending.rs.
- **Read.** D4d s8, A20.
- **Check.** Pictures every 50 frames from 3950 to 5364 equal MAME; caption start frames equal.

#### M3-27 Ending, part 5: sequencer, class 97 stand-ins, name entry hook, return to the ranking

**1.5 h, C (Claude worker), lane A. Needs: M3-26, M2-22. From: D4d T3.7 (end), action 0x42.**

- **Scope.** The 0xA69B8 jump table (sub 0, 2, 4, 6, 8), the 5360 frame wait, name entry for qualifying players parked until it ends (sub 6), sleep 180, mode 0x14 and the ranking; script action 0x42 (Episode 8 clear: sound 0x23, `G.$516 = 1`, one pool A class 97 stand in per hero, absent heroes at X 0x10E0, wait while the boss `+$A6` is set).
- **Original.** 0xA69B8 (0xA69F2, 0xA6ABA, 0xA6ADA, 0xA6AE8, 0xA6AF6), 0xAC48, 0x1E650, class 97.
- **Data to extract.** none.
- **Files.** game/flow/ending.rs, game/script.rs, enemy (class 97).
- **Read.** D4d s8, A20, A21; D4c s5.4 (0x42), open item 3; D3d s7.
- **Check.** D4d A20: sub 4 at 3, sub 6 at 5364, mode 0x14 at 5546; A21 name entry parked until 5019, table at 5441, mode 0x14 at 5776. The Episode 8 placeholder boss kill reaches the ending and then the attract loop.

#### M3-28 Flow scenarios: the end chain of all 8 episodes, link scenes, camera events

**2.0 h, X (external CLI), lane B. Needs: M3-06, M2-31. From: T-C10 (part).**

- **Scope.** Record in MAME the scenarios D4c A4 (13 area changes), A9 (poked end flow of all 8 episodes f_ep1 to f_ep8), the link scenes b9 link-ep2 to link-ep8, and the ending runs as harness traces. Ground truth for M3-29.
- **Original.** none.
- **Data to extract.** none.
- **Files.** re/harness/scripts/, re/harness/pokes/.
- **Read.** D4c s10, D4d s13 A19, A20.
- **Check.** Each has a MAME trace; the area change frames of D4c 5.7 are in the log.

#### M3-29 M3 acceptance, part 1: autoplay to the ending, flow compare

**2.0 h, C (Claude worker), lane A. Needs: M3-27, M3-22, M3-21, M3-19, M3-14, M3-15, M3-12, M3-09, M3-04, M3-28. From: T-C10, D4c A4, A9.**

- **Scope.** Run `--autoplay` from power on through all 8 episodes to the ending and the return to the attract loop. Compare mode, sub mode, stage, area and frame counts of every link scene and area change with the MAME flow traces; list differences in `re/harness/allow/m3.txt`.
- **Original.** all of M3.
- **Data to extract.** none.
- **Files.** re/harness/allow/m3.txt, M3 files.
- **Read.** D4c s10, D4d s13.
- **Check.** The autoplay finishes the game; link scene lengths equal D4c (panel lengths 306, 250, 308, 439, 368, 368, 313; control frames 648, 573, 321, 894); every area change frame equals MAME or is listed.

#### M3-30 M3 acceptance, part 2: fix list, placeholder report, docs

**2.0 h, C (Claude worker), lane A. Needs: M3-29. From: T-C10.**

- **Scope.** Fix the code mistakes the compare found, print the placeholder report (classes seen per episode, with counts), write the list of what the main session must change in STATUS.md and ARCHITECTURE.md (section 11 of this plan), run `regress.sh`. Play the whole game once with a human style input (no kill all) in Episodes 1 to 3 to check the pace.
- **Original.** none.
- **Data to extract.** none.
- **Files.** re/harness/allow/m3.txt, plan/game/design/D5-errata.md (append only).
- **Read.** this plan s3, s11.
- **Check.** Autoplay passes without the debug kill all in Episode 1 (placeholder Vice dies); regress.sh not worse; screenshots of every link scene looked at.

### M4: Sound

The Z80 sound driver ported by hand into a new crate, a QSound mixer, the audio output and all cue calls of D4e. Decision: hand port, no Z80 interpreter (see risks). Exactness is tested at the level of the QSound register writes (the DSP internal ROM is not in dino.zip, so pan law and echo are approximations).

Hours: 35.0 (lane A 29.5, lane B 5.5), 20 tasks.

#### M4-00 Finish the sound spec: pan motion, ADPCM pan, tables

**2.0 h, C (Claude worker), lane B. Needs: none. From: re/specs/sound/HANDOFF.md next steps 1 and 2.**

- **Scope.** Read the parts the spec only skimmed: pan motion handlers 0x1030 to 0x1196 and the ADPCM pan 0x2222 to 0x2384 of the Z80 program, dump the tables (lengths 0x0C50, 0x0C57, 0x0C5E, octaves 0x0C65, 0x13DE, 0x144A, 0x1453, 0x152B, 0x23A9 to 0x28A9), check that 0x2A81 and 0x4A81 are exact nibble times byte products, find the sample count, confirm the IRQ rate (250 Hz) from MAME. Update re/specs/sound/SPEC.md and HANDOFF.md. Can start at any time before M4-02.
- **Original.** Z80 program (decrypted with re/specs/sound/kabuki.py).
- **Data to extract.** none in git (spec text only).
- **Files.** re/specs/sound/SPEC.md, HANDOFF.md, re/specs/sound/*.py.
- **Read.** re/specs/sound/SPEC.md, HANDOFF.md, D4e s0, asset-catalog s8.
- **Check.** Every open item of HANDOFF.md section "Known issues" is answered or marked unknown with the reason; the table list has lengths and the formulas for the multiply tables are confirmed.

#### M4-01 QSound register trace recorder (MAME)

**2.0 h, X (external CLI), lane B. Needs: M4-00. From: new.**

- **Scope.** A probe command `sound CODE [d1] [d2]` that writes a request into the 68000 sound ring in RAM, and a Lua write tap on the Z80 side for the QSound ports (data high and low, register, bank) that logs frame, register and value into `re/out/sound/CODE.qs.trc` (git-ignored). First verify that `-sound none` still runs the Z80 (it should: it is a CPU); if not, use the dummy sound driver. One MAME process for a whole batch of codes (batch-mame-runs rule).
- **Original.** none (tool).
- **Data to extract.** none.
- **Files.** re/harness/probe/probe.lua, re/harness/probe/sound.sh (new), re/harness/README.md.
- **Read.** capture-recipes s0 (batching), s3, D4e s0 and s9, SPEC s2.
- **Check.** Codes 0x00 (music) and 0x30 (coin) produce a non empty trace twice and the two traces are equal.

#### M4-02 Sound data: ROM files, Kabuki decrypt, typed tables

**2.0 h, C (Claude worker), lane A. Needs: M4-00. From: T4.1.**

- **Scope.** Add `cd_q.5k` and `cd-q1.1k` to `cd-q4.4k` with their CRCs to the ROM loader, port the Kabuki decrypt (keys in SPEC s1) into the extractor, and extract the command table (0x106 entries, 39 music, 204 effects, 19 empty), sequences, instrument table, sample table (124 samples) and the pitch and multiply tables into typed structs. Exact copies, no ROM bytes in source.
- **Original.** Z80 image 0x0000 to 0x6E62+, sample ROM.
- **Data to extract.** all sound tables.
- **Files.** dino-extract/src/sound.rs, rom.rs (main session: allowed), dino-assets/src/sound.rs (new, one line in each lib.rs allowed).
- **Read.** SPEC s1 to s5, asset-catalog s8, N8.
- **Check.** Counts equal the catalog (243 live codes, 124 samples). The decrypted reset code decodes to `di; im 1; ld sp,$ffff; ld hl,$f000; ...` (unit test on a derived checksum, not bytes).

#### M4-03 `dino-sound` crate, part 1: mailbox, commands, priorities, track setup

**2.0 h, C (Claude worker), lane A. Needs: M4-02. From: T4.2 (part 1).**

- **Scope.** New crate `crates/dino-sound`: `Driver::command(code, d1, d2)` taking the 68000 ring entry format, normal and special commands (0xFF00 stop all, 0xFF05 stop music, 0xFF07 fade with speed, 0xFF0B enable), the priority rule (>= replaces), the track structures, the 250 Hz tick.
- **Original.** Z80 main loop and command dispatch (SPEC s4, s5), 0xA63C and 0x610 on the 68000 side.
- **Data to extract.** from M4-02.
- **Files.** crates/dino-sound/ (new), Cargo workspace (main session: allowed).
- **Read.** SPEC s3 to s5, D4e s0, s6.
- **Check.** Unit tests: priority replacement cases and the special command table of D4e s0; a command stream of the attract screens produces the expected track allocation.

#### M4-04 `dino-sound` part 2: the sequencer

**2.0 h, C (Claude worker), lane A. Needs: M4-03. From: T4.2 (part 2).**

- **Scope.** Sequence events (note, rest, loops, jumps, tempo, volume, pitch, instrument change and the rest of the event set), the sequencer step every 4th tick, track end.
- **Original.** Z80 sequencer (SPEC s6).
- **Data to extract.** sequences.
- **Files.** crates/dino-sound/src/seq.rs.
- **Read.** SPEC s6.
- **Check.** The note and event stream of music 0x00 and 0x01 for 20 seconds equals the walk of `re/specs/sound/walkall.py` (event count and times) in a unit test.

#### M4-05 `dino-sound` part 3: envelope, LFO, volume and pitch formulas, DSP register writes

**2.0 h, C (Claude worker), lane A. Needs: M4-04. From: T4.2 (part 3).**

- **Scope.** Envelope, LFO, the volume and pitch formulas (keep the pitch clamp at 0x139D that looks inverted), the DSP register write list per voice.
- **Original.** Z80 envelope and DSP write routines (SPEC s7, s8).
- **Data to extract.** pitch and multiply tables.
- **Files.** crates/dino-sound/src/voice.rs.
- **Read.** SPEC s7, s8, HANDOFF known issues.
- **Check.** The register write trace of code 0x30 (coin) and 0x01 equals the MAME trace of M4-01 (M4-09 tool) for the whole code.

#### M4-06 `dino-sound` part 4: pan motion, fades, effect versus music masking, ADPCM pan

**2.0 h, C (Claude worker), lane A. Needs: M4-05. From: T4.2 (part 4).**

- **Scope.** Pan motion modes (5 and 6 by facing, 7, 3 and 4) with the pan table 0xA66A index rule, music fade (0x0444 speed), the rule that an effect owning a voice masks music on it, twin headers, tempo and pitch offsets cleared by a new music code.
- **Original.** Z80 pan motion 0x1030 to 0x1196, 0x2222 to 0x2384.
- **Data to extract.** pan tables.
- **Files.** crates/dino-sound/src/pan.rs.
- **Read.** SPEC s4, s5, M4-00 output, D4e s6.
- **Check.** Register traces equal MAME for 10 effect codes with pan (hit 0x38, jab 0xA8, break 0x6F, GO 0x3E, clock 0x34, explosion 0x66, voice banks 0xCA and 0xCC, shotgun, bazooka) and for a fade of music 0x01.

#### M4-07 QSound mixer, part 1: DSP model, voices, sample playback

**1.5 h, C (Claude worker), lane A. Needs: M4-06. From: T4.3 (part 1).**

- **Scope.** The DSP register model (voice start, end, loop, pitch, volume, pan), sample playback at 24038 Hz with the pitch step, 16 voices, mixing to f32 stereo.
- **Original.** QSound DSP behaviour (not in the ROM set: from the register meaning in the SPEC).
- **Data to extract.** sample ROM.
- **Files.** crates/dino-sound/src/mixer.rs.
- **Read.** SPEC s8, HANDOFF known issues.
- **Check.** A unit test renders code 0x30 and 0x01 to a buffer; length and non silence checks; example below.

#### M4-08 QSound mixer, part 2: pan law, echo, WAV example

**1.5 h, C (Claude worker), lane A. Needs: M4-07. From: T4.3 (part 2).**

- **Scope.** Equal power pan (approximation), the plain echo (feedback register 0x93, length 0xD9), mono mode forcing pan 0x10, and an example `play` that writes WAV files of any code into re/out/sound/.
- **Original.** DSP echo registers.
- **Data to extract.** none.
- **Files.** crates/dino-sound/src/mixer.rs, examples/play.rs.
- **Read.** SPEC s8, D4e s0.
- **Check.** The WAV of music 0x00 (title) and the coin sound listened to by the main session; length per D4e 1.6 table within 1 percent.

#### M4-09 Register trace comparer

**1.5 h, X (external CLI), lane B. Needs: M4-01. From: new.**

- **Scope.** A python tool that compares a MAME `.qs.trc` with the port driver's register write trace for the same code (first differing write, count, time), and a batch mode for a list of codes. The port side is an example `dump_regs CODE` in dino-sound.
- **Original.** none (tool).
- **Data to extract.** none.
- **Files.** re/harness/sound_cmp.py (new), crates/dino-sound/examples/dump_regs.rs.
- **Read.** M4-01, M4-05.
- **Check.** A code compared against itself reports 0 differences; against a different code reports the first difference.

#### M4-10 Driver exactness pass: all 39 music codes

**2.0 h, C (Claude worker), lane A. Needs: M4-06, M4-09. From: T4.2 exactness.**

- **Scope.** Run every music code 0x00 to 0x26 through the comparer, fix the driver until the register traces equal MAME or the difference is documented.
- **Original.** Z80 driver.
- **Data to extract.** none.
- **Files.** crates/dino-sound/.
- **Read.** SPEC, D4e s1.
- **Check.** List of 39 codes with differences count 0 (or documented). Listening check of the area tunes.

#### M4-11 Driver exactness pass: effects (204 codes)

**2.0 h, C (Claude worker), lane A. Needs: M4-10. From: T4.2 exactness.**

- **Scope.** Same for the effect codes with their priority, twins and toggles (the 24 codes of D4e s7 without a sender are included for the driver only).
- **Original.** Z80 driver.
- **Data to extract.** none.
- **Files.** crates/dino-sound/.
- **Read.** D4e s2 to s5, s7.
- **Check.** List of effect codes with differences 0 (or documented); priority steals checked on 5 pairs.

#### M4-12 Audio output (cpal) and the driver tick loop

**1.5 h, C (Claude worker), lane A. Needs: M4-08. From: T4.4 (part 1).**

- **Scope.** `audio.rs` with cpal: a ring buffer, the driver ticked at 250 Hz per game frame ratio (game frame 59.6 Hz), sample rate conversion, mute and demo sound flag from the config, no sound when the device is missing (the game still runs).
- **Original.** none.
- **Data to extract.** none.
- **Files.** dino-game/src/audio.rs, main.rs (allowed), Cargo.toml (cpal).
- **Read.** SPEC s2, D4e s0.
- **Check.** The game runs with sound from the attract loop; music starts in the right frame of the title (checked with the request log); the game still starts with no audio device.

#### M4-13 `Game::sound_ex` with d1 and d2 on every request

**2.0 h, C (Claude worker), lane A. Needs: M4-12. From: S2.**

- **Scope.** Store pan mode, pan byte, middle byte and d2 on every sound request, thread them through every `snd_*` helper (the hit helper uses the X of the object a2), keep the request recording used by the harness (now compared with d1 and d2).
- **Original.** 0xA63C, wrappers 0x96xx to 0x9F60.
- **Data to extract.** pan table 0xA66A.
- **Files.** game/snd.rs, game/mod.rs (main session: allowed), callers in item/, enemy/, player*.rs.
- **Read.** D4e s0, s8 (S2), SPEC s3.
- **Check.** Harness diff of sound requests including d1 and d2 on the existing stage 1 scenarios: not worse than before and equal where MAME has the call.

#### M4-14 Attract filter and the front end sound lines

**1.5 h, C (Claude worker), lane A. Needs: M4-13, M2-32. From: S1, S3 (part 1).**

- **Scope.** The flush filter (drop a normal code when not in a game unless it is 0x30 or the demo sound setting is on; 0xFFxx passes; record the call anyway), and the `sound()` lines of the screens: boot, title 0x88, ranking 0x80 then 0x70, slides 0x24 and 0x25, profiles, movie, coin 0x30, start 0x31.
- **Original.** 0x610 (filter), D4d s3.11.
- **Data to extract.** none.
- **Files.** game/snd.rs, flow/*.rs.
- **Read.** D4e s6, s8 (S1, S3); D4d s3.11, s11.
- **Check.** D4e acceptance 5 (0x88 dropped in attract, 0x30 and 0xFF00 copied) and the sound frames of D4d A3, A5, A8, A9, A10 equal MAME from power on.

#### M4-15 Front end sound lines, part 2: select, continue, game over, story, tally, ending

**1.5 h, C (Claude worker), lane A. Needs: M4-14. From: S3 (part 2).**

- **Scope.** Select 0x00 / 0x32 / 0x33, continue 0x1B / 0x35 / 0x1C / 0xAB, game over 0xE6 / 0x1D / 0x1F, story 0x13 to 0x16, tally 0x49, ending 0x19 / 0x1A and the lab codes, name entry music 0x1F.
- **Original.** as listed in D4e s8 S3.
- **Data to extract.** none.
- **Files.** flow/*.rs.
- **Read.** D4e s8 (S3), s9 (6, 7); D4d s11.
- **Check.** D4e acceptance 6 (continue 0xFF00 then 0x1B, tick 0x35 with d1 0x10, 0x1C at 9) and 7 (ending 0x19, 0x1A); request frames of A13, A15, A17, A18, A20 equal MAME.

#### M4-16 Area and boss music, S10, class 96 timing

**1.0 h, C (Claude worker), lane A. Needs: M4-13. From: S10, S7 check, D4e s1.**

- **Scope.** Verify the area music at every stage start and area change (0xA582 first area, 0xA5C8 lock bytes), the Episode 7 lock 2 boss case `+$7D == 0xFE` playing 0x0F instead of 0x12, the class 96 delay (59 frames after the fade), the fade 0xFF07 at area ends.
- **Original.** 0xA55A, 0xA582, 0xA5C8, 0x92382.
- **Data to extract.** none.
- **Files.** game/flow.rs (`flow_area_music`).
- **Read.** D4e s1, s9 (1 to 4), s10.
- **Check.** D4e acceptance 1 to 4 in the sound request log equal MAME for all 8 episodes (HX or flow scenarios).

#### M4-17 Hero voices and move sounds from the attack records

**1.5 h, C (Claude worker), lane A. Needs: M4-13. From: P-D2a-7.**

- **Scope.** Move voices and hit sounds from the attack record table (banks, jab toggle G.$43A, punch 3 and 4, special, dash, death shout, bank 1 twins 0xCA and 0xCC for the second player), the hit id sound and the G.$43C quirk (always the first code of a pair unless something sets it).
- **Original.** 0x9814, 0x985C, 0x962A, 0x96CE, 0x9732, 0x9796 and the record sound byte.
- **Data to extract.** voice tables (asset pack).
- **Files.** game/snd.rs, player*.rs.
- **Read.** D2a s18 (P-D2a-7), D4e s2, s3, s9 (8).
- **Check.** D4e acceptance 8 (first jab 0xA8 then 0x38, punch 3 0x4F, punch 4 0x50, with a second player 0xCA and 0xCC) equals MAME in the request log; D2a A8 chain.

#### M4-18 Weapon, item and container sounds

**1.5 h, C (Claude worker), lane A. Needs: M4-17. From: P-D2c-9, S12.**

- **Scope.** Rifle 0x9C76, pistol 0x9CD8, Uzi 0x9D9C, shotgun 0xA442, bazooka 0x9E08, throw 0x9D44, explosions 0x66, container break toggles (acceptance 9), food and treasure pick up, only for callers that exist (S12 rows only if a hero-owned shot calls them).
- **Original.** as listed.
- **Data to extract.** none.
- **Files.** item/*.rs, game/snd.rs.
- **Read.** D2c s20 (P-D2c-9), D4e s4, s9 (9).
- **Check.** D4e acceptance 9 (two container breaks 0xF5 then 0x6E or the toggled order); the gun scenarios of D2c W2 to W4 request streams equal MAME.

#### M4-19 Sound acceptance: request streams and a listening check

**2.0 h, C (Claude worker), lane A. Needs: M4-18, M4-16, M4-15, M4-11. From: T4.5, D4e s9.**

- **Scope.** Compare the sound request streams (code, d1, d2) of the M1, M2 flow scenarios and the stage 1 harness scenarios with MAME, list differences in `re/harness/allow/m4.txt`, fix. The main session listens from power on through Episode 1.
- **Original.** none.
- **Data to extract.** none.
- **Files.** re/harness/allow/m4.txt, M4 files.
- **Read.** D4e s9, s10.
- **Check.** Request streams equal for the attract loop (filtered as MAME), the select, Episode 1 start and the clock beep; QSound register traces equal for the codes used in them.

### M5: Episode 1 for real

The services every later fight needs (monster passes, boss intro boxes, effects), Vice T. with his Rock Hopper and Walther, the Episode 1 props, and the MAME checks of the player, weapons, items and the whole episode. Rule for every class task of M5 to M8: delete the class from the placeholder registry (M3-01) and run `--list-placeholders` in the check.

Hours: 36.9 (lane A 32.9, lane B 4.0), 23 tasks.

#### M5-01 Monster services (hostile list, passes, colour flash, hunter poke)

**2.0 h, C (Claude worker), lane A. Needs: M3-30. From: P1, P-D2b-4 (0x1132A), HT1 (passes).**

- **Scope.** The hostile list 0x6B64 (draw and hit lists) and its passes: 0x11A9E and 0x11B36 with the guard table 0x11CE4, 0x1132A (formula path with the owner's `$C6`), 0x11CEC and 0x11D32, 0x11E56 and 0x11EDC (raw damage, credit 0xFF, minimum 1 HP when +$7D < 0), the reaction mapping 0x11CC4, the colour flash routine (tables 0x4F4C8, 0x4F4DC, 0x5396A), the hunter poke 0x533FA, the pool C class 30 "Z" spawner hook. Decision HT1: keep the per slot `+$84` and `+$5C` across frees as the original does (a fresh game has 0).
- **Original.** 0x6B64 list, 0x11A9E, 0x11B36, 0x1132A, 0x11CEC, 0x11D32, 0x11E56, 0x11EDC, 0x11CC4, 0x533FA, 0x9667E.
- **Data to extract.** guard table, flash tables.
- **Files.** game/enemy_combat.rs, combat.rs (main session: allowed for the pass registration), enemy.rs.
- **Read.** D3b s1, s11, s13 (P1); D3a s3, s15 (HT1); D2b s5.
- **Check.** D3b A1 level 31 and A13 numbers need the classes (later); here: unit tests of the pass rules (dinosaurs give no score, +$7D < 0 floor 1 HP), regress.sh not worse. Pass counts equal MAME in a Vice scenario once M5-09 exists.

#### M5-02 Pass 0x10DCC: attack objects against heroes

**1.5 h, C (Claude worker), lane A. Needs: M5-01. From: P-D2c-2, HU16, WM17.**

- **Scope.** Attackers of list 0x671A against players: owner and serial rules, same depth for player owners, ids 0x6E and 0x6C skipped, 1 HP floor for friendly fire, full record damage for enemy owned objects (explosions, bombs, bullets), drop without hero exceptions, reaction and hit stop. The enemy side of thrown dynamite is a later class task.
- **Original.** 0x10DCC, list 0x671A.
- **Data to extract.** none.
- **Files.** game/enemy_combat.rs, combat.rs.
- **Read.** D2c s14, s20 (P-D2c-2), W17, M3 scenarios; D3a H16.
- **Check.** D2c M3 (friendly fire with weapons, n = 4 enemies: 1 HP at dz 0, nothing at dz 1; same with 0 enemies) from the harness scenarios of M2-30.

#### M5-03 Boss intro services: dialogue box B50 and camera mover B76

**2.0 h, C (Claude worker), lane A. Needs: M2-10, M3-03. From: B50, new (B76 has no owner in the index).**

- **Scope.** Pool B 50 (dialogue box with portrait, text lines var 0, 1, 2, 4, used by Vice, Butcher, Slice, Morgan, Fessenden) and pool B 76 (camera mover that writes G.$28, G.$2E, G.$7E4, G.$7E8 in the intros). B76 has no design text: read the handler with re/tools/d2 and write a short note in re/specs/enemy/.
- **Original.** 0x844B4, printer 0x84F72, B76 handler (find in classmap).
- **Data to extract.** portrait tables (0x8461A, 0x8463E, 0x84696, 0x846DE as needed), box strings.
- **Files.** game/prop/b50.rs, b76.rs (new).
- **Read.** D3e part 1 s3; D3c s2.2, s10 item 7; D4d open item 4.
- **Check.** Vice intro in MAME (D3c BS-V1): B50 at t = 205, B76 at 435 with the same fields; pictures of the talk box equal MAME.

#### M5-04 Effects: container debris class 11, stones, heavy puffs class 21

**2.0 h, C (Claude worker), lane A. Needs: M5-01. From: FX-P1 (11, 21), IT-P2, P-D2c-6 (debris), IT10, IT11.**

- **Scope.** Pool C class 11 (18 animations: planks, shards, stone rubble, tires, parabolic arc and floor bounce, 9 rubble pieces from a stone break) and the stone weapon spawn (chances 2/16, 8/16, 3/16 per piece while `G.$4F1 < 3`), pool C class 21 (60 animations: barrel 10 puffs, tires 5, statue 8, pail body, sound 0x27). D3f owns the class list, D4a the rules: D3f wins on conflicts.
- **Original.** 0x934F6, 0x93770, 0x94640, 0x94672, 0x259E8, 0x292A6, 0x2944A, 0x294AA.
- **Data to extract.** animation tables 0x937AA, 0x951AC (asset pack).
- **Files.** game/effect.rs, game/effect/fx11.rs, fx21.rs (new).
- **Read.** D3f s2 classes 11, 21, FX07, FX08, FX16; D4a s5, s8.4, A06, A11; D2c s21 item 1.
- **Check.** D4a A06 (effect objects made at t = 14, drum pieces gone by t = 73, stone pieces by 104, puffs by 75) and A11 (stones at t = 272 and 274, G.$4F1 values) on the container scenarios.

#### M5-05 Effects: ground dust class 7, dizzy stars class 19, hammer shockwave class 3, with their spawn sites

**1.7 h, C (Claude worker), lane A. Needs: M5-01. From: FX-P1 (7, 19), P-D2a-8, P16.**

- **Scope.** Pool C class 3 (the Hammer T. ground shockwave, spawned at 0x3C9FC by the ported class 10, it is missing in Episode 1 wave 4), class 7 (14 frame dust of slides, skids and landings, thrown item landing) and class 19 (two animations of stars circling the head, countdown 3 per frame when mashing, loop sound 0x51 recorded), the spawn helper 0x92E5E and the stun check 0x94338 in the player and enemy code. Sparkle blink of D2a (class 23) is in M2-17.
- **Original.** 0x929B8 (class 3), 0x92E5E, 0x92ECC, 0x29F54, 0x94338, 0x94462.
- **Data to extract.** animation lists 0x92F3A, 0x944DC.
- **Files.** game/effect.rs, game/effect/fx07.rs, fx19.rs (new), player.rs, enemy.rs.
- **Read.** D3f classes 7, 19, FX06, FX09, FX-S01, FX-S02; D2a s18 (P-D2a-8); D2b s4.
- **Check.** D2a A7 and A12 dust frames, D2b S16 (dizzy with the stars) from the harness scenarios; class 7 objects at the slide start frame equal MAME.

#### M5-06 Effects: explosion class 24, fuse spark class 25, flash class 22

**1.5 h, C (Claude worker), lane A. Needs: M5-04. From: FX-P2 (24, 25, 22).**

- **Scope.** Pool C class 24 (41 animations, the dynamite, TNT and bazooka blast plume, sound 0x3A recorded), class 25 (fuse spark attached to the fuse tip at (+10, +57), sound 0x79) and class 22 (screen flash whiteout: palette words ORed with 0xF000F000, 4 frame decay, lightning use in Episode 6).
- **Original.** 0x959E6, 0x959B4, 0x96296, 0x9625C, 0x95840, 0x9582A.
- **Data to extract.** tables 0x95E28, 0x28904.
- **Files.** game/effect/fx24.rs, fx25.rs, fx22.rs (new), item/blast.rs (spawn sites).
- **Read.** D3f classes 22, 24, 25, FX17 to FX19; D4a s9.4; D2c W10.
- **Check.** D2c W10 (dynamite explosion exactly 60 frames after the rest, grenade chain objects at +1, +11, +21) and D4a A07 TNT timeline; explosion pictures equal MAME.

#### M5-07 Effects: burn death class 15, torch glow overlay, flames on enemies

**1.2 h, C (Claude worker), lane A. Needs: M5-06. From: FX-P2 (15, item 6), P-D2c-4, DS14.**

- **Scope.** Pool C class 15 (burn death collapse, 6 animations, smolder 60 to 90 frames, sound 0x23), pool E class 6 (torch glow halo, Z alternating -1 and +2, 6 frame idle and 4 frame swing), flames attached to burning enemies (class 8 exists for heroes).
- **Original.** 0x93E54, 0x93D48, 0x29B06, 0x29C26, 0x2ADAE.
- **Data to extract.** tables 0x93F42, 0x29CB6, 0x29CF4.
- **Files.** game/effect/fx15.rs, item/blast.rs (item_overlay).
- **Read.** D3f class 15, item 6, FX10, FX25, FX-S04, FX-S05; D2c s5.6, W9.
- **Check.** D2c W9 (torch hit at +12, overlay from t = 4 with Z 295 and 298 alternating) and D3f FX-S05; burn death picture sequence equals MAME in a torch kill.

#### M5-08 Effects: muzzle flash class 31 and ricochet spark class 33

**1.0 h, X (external CLI), lane A. Needs: M5-04. From: FX-P3 (31, 33), P-D2c-6.**

- **Scope.** Pool C class 31 (4 muzzle flashes: pistol, rifle, shotgun, machine gun; 2 to 4 frames at the muzzle) and class 33 (ricochet metal spark with the 4 way scatter table 0x96D8A, sound 0x2F recorded). Class 41 (car gun spark) goes with the car.
- **Original.** 0x96800, 0x967C4, 0x24ECC, 0x96B78, 0x96B42, 0x28720.
- **Data to extract.** tables 0x96884, 0x96D06, 0x96D8A.
- **Files.** game/effect/fx31.rs, fx33.rs (new), item/blast.rs (spawn).
- **Read.** D3f classes 31, 33, FX11, FX12, FX-S08; D2c s4.
- **Check.** D2c W2 and W3 shot frames: class 31 object present at t = 41 for 2 to 4 frames with the right animation per gun; D3f FX-S08.

#### M5-09 Vice T., part 1: intro (class 18 variant 0)

**2.0 h, C (Claude worker), lane A. Needs: M5-03, M5-02, M3-03. From: P-D3c-2 (part 1), BS03.**

- **Scope.** Vice init and the intro: waits for the camera at X 0x1D0, talk box (sound 0x20), 40 frames, camera mover, anim 0x14, 60 frames, walk, two captions, the gun anim, then alive. States r6 0 to 0x12 with durations 136, 187, 40, 32, 61, 12, 178, 61. Music 0x0E recorded (S4 part). Boss HP row and bar already exist.
- **Original.** 0x42590 variant 0 (intro states), tables 0x44B84, 0x44C8C, 0x43004.
- **Data to extract.** tables as listed (dino-extract/src/boss.rs).
- **Files.** game/enemy/class18.rs (new), dino-extract/src/boss.rs.
- **Read.** D3c s2.2, BS03, A BS-V1.
- **Check.** D3c BS-V1: Vice made at t = 68 at X 792, Z 296; states 0 to 0x12 at t = 68, 204, 391, 431, 463, 524, 536, 714, 775, alive at 776; pool B 50 at 205, 76 at 435, 66 vars 0x0B and 0x55 at 540, B22 at 780. HX scenario `jump 1 2 0xA0`.

#### M5-10 Rock Hopper, part 1: Vice's partner intro (class 29)

**1.5 h, C (Claude worker), lane A. Needs: M5-09, M5-01. From: P4 (part 1).**

- **Scope.** Class 29 intro states only (spawned by Vice's init, variant 0, partner intro synchronised with Vice's `$6`: modes 2, 4, 6 at t = 206, 392, 522, 8 at 538, 0xC at 748; flash palettes 0x91 and 0x92 from 817), the hop movement and step tables needed there, the leave. The hostile modes come in M7.
- **Original.** 0x4DCF6, 0x4F4E4, step 0x4FA30, offsets 0x4F3DA.
- **Data to extract.** animation tables 0x4F4E4, step table.
- **Files.** game/enemy/class29.rs (new).
- **Read.** D3b s4, s13 (P4), A15; D3c s2.2.
- **Check.** D3b A15: a Hopper at x 720, z 319 in the intro (`+5 = 4`), `$6` 2 at t = 22, 4 at 209, 6 at 338, 8 at 606, flash from 817, 0xE at 904, `+5 = 6` at 928.

#### M5-11 Vice T., part 2: AI core

**2.0 h, C (Claude worker), lane A. Needs: M5-10. From: P-D3c-2 (part 2), BS04.**

- **Scope.** 13 combos picked by a 32 cell table, attack gate T[level] (6 to 31 of 32) every 60 frames, recovery pause tables, speeds by level (2.6 to 3.5 px), hit reactions and the poise rule (a long chain locks him, `+79` refill), damage taken, get up.
- **Original.** 0x42590 (AI), 0x436B4, 0x436D4, 0x44F4E.
- **Data to extract.** tables as listed.
- **Files.** game/enemy/class18.rs.
- **Read.** D3c s2.3, BS04; D3a s3.
- **Check.** D3c BS-V3 damage dealt by Vice (jab 9 to 11, heavy 24 to 25, jump attack 16 to 21) and BS-V1 timelines after the intro; harness scenario of Vice against a pinned hero in step mode.

#### M5-12 Vice T., part 3: bullet class 59, summons, rage

**2.0 h, C (Claude worker), lane A. Needs: M5-11, M3-02. From: P-D3c-2 (part 3), BS05, BS06, BS07, HU05.**

- **Scope.** Bullet class 59 (variant 2 for Vice: speed 7, height 70, attack 0x0B, 15 base, knock down, shared with the poachers of M6) with the muzzle flash class 57, the summon (after a knockdown at HP <= max / `+$C2` 2, 3, 4: jump back, revolver into the air, `G.$502 = 1`, reinforcements by class 22) and the rage (HP <= 50 then 25, `+$AA` 1800).
- **Original.** 0x57FB4 (class 59), 0x57D94 (class 57), Vice summon and rage routines.
- **Data to extract.** none new.
- **Files.** game/enemy/class18.rs, class59.rs (new), class57.rs (new).
- **Read.** D3c s2.4, s2.5, BS05 to BS07; D3a s4 (bullet).
- **Check.** D3c BS-V2: summon states 0xA at t = 1426, 1697, 1867 with `+$C2` = 3, 4, 5 at HP 114, 44, 23; Ferris var 2 at X 448 and var 0x202 at X 880 at t = 1485 and 1490; first rage at HP 5 (t = 1935); death at 1946. D3a H5 bullet numbers (hero hit at t = 36, 100 to 85 at level 0).

#### M5-13 Vice T., part 4: death, scores, Walther (variant 2), acceptance

**1.5 h, C (Claude worker), lane A. Needs: M5-12. From: P-D3c-2 (part 4), HT8, P-D3c-7 (Vice share), S4.**

- **Scope.** Vice death (kill-all, slow motion, score 10,000, `0xACEE`, music fade 0xA5EA), the death chain acceptance BS-E1 for Vice, Walther as class 18 variant 2 (Vice's AI without the intro, HP table, never dizzy, score 8,000, Episode 4 and 5 garage units), drops of the helper lists.
- **Original.** 0x42590 (death, variant 2).
- **Data to extract.** none new.
- **Files.** game/enemy/class18.rs.
- **Read.** D3c s2.6, BS-E1; D3a s9, HU21; D4a IT25.
- **Check.** D3c BS-A1 (Vice HP 500 / 600 / 700), BS-E1 (HP 0, scripted mask at once, `+$04 = 4` after 97 game frames); D3a H10 Walther +8,100 total. Episode 1 is now played to the end of the boss and the tally with the real Vice.

#### M5-14 Steam pipes B40

**1.0 h, X (external CLI), lane A. Needs: M3-16, M2-05. From: B40, W27.**

- **Scope.** Pool B 40 (var 4 at the end wall of Episode 1 area 0, later vars 8, 0xC, 0x10 in Episode 8): one hit breaks them (HP 8), 11 tile codes change, steam puffs (pool A 15 pieces exist, 6 alive) forever. Box rules from D3e part 3 2.6 including the jump kick needed for var 0x10.
- **Original.** 0x836A4, spawner 0x41EEA.
- **Data to extract.** tile blocks, box table.
- **Files.** game/prop/b40.rs (new).
- **Read.** D3e part 1 s3.1, A1 to A3; part 3 s2.6, A23, W315.
- **Check.** D3e A1 (state 0x02000000 to 0x04020000 for Z 296 to 320 only), A2 (exactly 11 tile code words change), A3 (6 class 15 objects from t = 45).

#### M5-15 Prepare scenarios: player moves and respawn

**1.5 h, X (external CLI), lane B. Needs: M3-06. From: P-D2a-4, P-D2a-5, P-D2b-6 (prep).**

- **Scope.** Record in MAME with the HX tool the scenarios D2a A1 to A14 (idle, walk, run, double tap, jumps, jump attacks, chain, throws, special, dash, command moves, wall jump) for Mustapha, Hannah and Mess, and D2b S1 to S21. Ground truth for the verify tasks.
- **Original.** none.
- **Data to extract.** none.
- **Files.** re/harness/scripts/, re/harness/pokes/.
- **Read.** D2a s17, D2b s9, M3-06.
- **Check.** Each scenario has a trace; the Jack ones equal the existing recordings.

#### M5-16 Prepare scenarios: weapons and items

**1.5 h, X (external CLI), lane B. Needs: M3-06. From: P-D2c-8, IT-P3 (prep).**

- **Scope.** Record D2c W1 to W14 for the four heroes and D4a A01 to A25 (A26 and A27 are M9) with the HX tool.
- **Original.** none.
- **Data to extract.** none.
- **Files.** re/harness/scripts/, re/harness/pokes/.
- **Read.** D2c s19, D4a s11.
- **Check.** Each scenario has a trace.

#### M5-17 Prepare scenarios: Episode 1 full runs

**1.0 h, X (external CLI), lane B. Needs: M3-06. From: T5.6 (prep).**

- **Scope.** Record scripted full runs of Episode 1 for 1 player (Jack), 3 players, and one run each for Mustapha, Hannah, Mess, from the start of control to the tally.
- **Original.** none.
- **Data to extract.** none.
- **Files.** re/harness/scripts/ep1_*.txt.
- **Read.** D3c BS-A1, re/harness/README.md.
- **Check.** Traces exist; the boss fight is reached in each.

#### M5-18 Verify: Mustapha, Hannah, Mess moves, wall jump

**2.0 h, C (Claude worker), lane A. Needs: M5-15, M5-13. From: P-D2a-4, P-D2a-5.**

- **Scope.** Run D2a A1 to A14 for the three heroes (step and run mode), find the real wall jump spots in Episode 1 area 2, fix whatever differs.
- **Original.** player code.
- **Data to extract.** none.
- **Files.** player*.rs.
- **Read.** D2a s17, s19.
- **Check.** Every A scenario for the three heroes: 0 differences in player fields or the difference listed in `re/harness/allow/m5.txt` with a cause.

#### M5-19 Verify: respawn S1 to S21

**1.5 h, C (Claude worker), lane A. Needs: M5-15, M2-17, M2-14. From: P-D2b-6.**

- **Scope.** Run D2b S1 to S21 and fix what differs.
- **Original.** respawn and rank code.
- **Data to extract.** none.
- **Files.** player.rs, rank.rs, effect.rs.
- **Read.** D2b s9, s11.
- **Check.** S1 to S21 numbers of D2b s9 equal or listed.

#### M5-20 Verify: weapons W1 to W14 and friendly fire M1 to M3

**2.0 h, C (Claude worker), lane A. Needs: M5-16, M2-30, M5-08, M5-07, M5-06, M5-02. From: P-D2c-8.**

- **Scope.** Run D2c W1 to W14 for the four heroes and the friendly fire scenarios M1 to M3 for 2 and 3 players, fix whatever differs.
- **Original.** weapon and pass code.
- **Data to extract.** none.
- **Files.** item/*.rs, player_weapon.rs, player_combat.rs.
- **Read.** D2c s19, s21.
- **Check.** W numbers of D2c s19 equal; M1 (hp after each jab by n) equal.

#### M5-21 Verify: items A01 to A25 and the HUD messages

**1.5 h, C (Claude worker), lane A. Needs: M5-16, M5-04, M2-29. From: IT-P3.**

- **Scope.** Run D4a A01 to A25 and fix. Check the Episode 1 hall suits of armor (W20) and the treasure 17 quirk.
- **Original.** item code.
- **Data to extract.** none.
- **Files.** item/*.rs, hud.rs.
- **Read.** D4a s11, s13.
- **Check.** A01 to A25 equal or listed.

#### M5-22 Episode 1 full run, 1 player

**1.5 h, C (Claude worker), lane A. Needs: M5-17, M5-18, M5-21, M5-13, M5-14. From: T5.6.**

- **Scope.** Run the recorded Jack run in `run` mode from the start of control to the tally and compare; fix differences, list the rest.
- **Original.** all of Episode 1.
- **Data to extract.** none.
- **Files.** any Episode 1 file.
- **Read.** D3c, D3e part 1.
- **Check.** `run` mode differences listed per class and cause; Episode 1 has no placeholder left (`--list-placeholders` empty for stage 0).

#### M5-23 Episode 1 full run, 3 players and the other heroes

**1.5 h, C (Claude worker), lane A. Needs: M5-22. From: T5.6.**

- **Scope.** Same for 3 players and for Mustapha, Hannah and Mess; Vice HP by player count.
- **Original.** all of Episode 1.
- **Data to extract.** none.
- **Files.** any Episode 1 file.
- **Read.** D2c s16, D3c BS-A1.
- **Check.** Runs compared; differences listed. The main session plays Episode 1 from coin to the tally.

### M6: Episodes 2 and 3

Episode 2: poachers, Lash T., Mack, Shivat with the hunter, the Butcher, swamp scenery. Episode 3: the radio, the Cadillac drive, bikers, barrels, bomb carriers, Hogg. Each episode ends with a full MAME run in M10; here every class has its own MAME scenario.

Hours: 58.9 (lane A 56.9, lane B 2.0), 33 tasks.

#### M6-00 Prepare scenarios for Episodes 2 and 3

**2.0 h, X (external CLI), lane B. Needs: M3-06. From: D3a s14, D3b s12, D3c s8 (prep).**

- **Scope.** Record with the HX tool D3a H1 to H27 (hunter, poachers, bullets, Lash, alternate Blade, bomb carriers, barrels, rocks, bikers, scene actors), D3b A9 to A11 and A16 to A21 (Mack, Shivat), D3c BS-B1 and BS-B2, BS-H1, BS-C1 to BS-C8, BS-V and E1 for the Butcher, car and Hogg. Prepare only.
- **Original.** none.
- **Data to extract.** none.
- **Files.** re/harness/scripts/, re/harness/pokes/.
- **Read.** D3a s14, D3b s12, D3c s8, M3-06.
- **Check.** Each scenario has a MAME trace; one MAME process per batch.

#### M6-01 Poachers (class 13), part 1: data, init, entrances, decisions, movement

**2.0 h, C (Claude worker), lane A. Needs: M5-12, M6-00. From: HT2 (part 1), HU07, HU22.**

- **Scope.** Extract the 14 data tables (anim tables 0x6C57A, 0x6CA24, 0x6CF3E, distance classes table 0x32988 and the decision tables), the init, the 10 entrances (0 to 0x12: stand 1200 frames, rise with fade 0x8B, jump in from above, run in with a dash, ambush jump in), the decision logic including the far gate quirk (reads the X class of the previous decision, HU22), walking states. Poacher J, Skinner, Gutter variants.
- **Original.** 0x3DDEA (9,252 bytes), 0x32988.
- **Data to extract.** tables as listed, in dino-extract/src/enemy.rs.
- **Files.** game/enemy/class13.rs, class13_ai.rs (new), dino-extract/src/enemy.rs.
- **Read.** D3a s4 (4.1 to 4.4), H9, H22 row, s15 (HT2).
- **Check.** D3a H9: entrance 2 to 0x12 first states and timers (0x34 1200, 0x36 60, 0x38 60 then 0x44 at t = 45, 0x1C at 6 with x 1280 to 1178 at 35, 0x3E 36 with vx -878, 0x4E 60). Remove class 13 from the placeholder registry only after M6-02.

#### M6-02 Poachers, part 2: rifle volley, dash tackle, kick, melee, flee, hit, death

**2.0 h, C (Claude worker), lane A. Needs: M6-01. From: HT2 (part 2), HU04 to HU06, HU08.**

- **Scope.** Rifle volley (1 to 3 shots 69 frames apart, bullet class 59 and flash 57 from M5-12, damage with the stale `+$84` quirk, attack 0x0B 15 knock down), dash tackle (attack 0x2C, 55 frames at 3.5 px per frame) and kick (0x0A), melee, flee timer, hit recovery, death, drops and scores (Poacher J 2,100, Skinner 3,100, Gutter 4,100). Poacher bullets never hurt enemies or dinosaurs (HU04).
- **Original.** 0x3DDEA (states), 0x57FB4, 0x57D94.
- **Data to extract.** none new.
- **Files.** game/enemy/class13.rs, class13_ai.rs.
- **Read.** D3a s4 (4.5 to 4.8), H5 to H8, H10.
- **Check.** D3a H6 (state 0x10 at 192, bullet at x 1136 at t = 243, hero -15 at 255, 0x18 at 261, 0x46 at 321), H7 (volley shots 252, 321, 682, 751, 820 for a spawn at t = 11), H8 (first melee at 193: hero 100 to 90, 78, 66 at level 0), H10 scores. Poacher is out of the placeholder registry.

#### M6-03 Mack (class 38) and the ground critters A78, A80

**2.0 h, C (Claude worker), lane A. Needs: M5-01, M6-00. From: P3, P7, DS10.**

- **Scope.** Mack the triceratops: init and side choice by the record X against the camera X, charge cycle (strides of 32 frames moving and 22 still, 122 px), gallop speed table 0x22 by `dir`, depth nudge, horn gating by `dir & 0xF` (horns hurt only in the middle of a stride), hit and slide, calm and leave. Critters A78 and A80 walk to the hero and stand (harmless).
- **Original.** 0x5341C (2,218 bytes), anim 0x53BE4, 0x5F198, 0x5FF2E.
- **Data to extract.** animation tables, speed table.
- **Files.** game/enemy/class38.rs, class78_80.rs (new).
- **Read.** D3b s3, s7, s13 (P3, P7), A9 to A11; D3a H16.
- **Check.** D3b A9 (spawn x 771, states 2, 4, 6 at t = 3, 34, 65, horn box live t = 130 to 136, Jack -15 at 131 and 440, freed at 682; level 31: -38, -37), A10 (hit at t = 22 +100 points, slide vx 1248), A11 (Ferris 44 to 29 at t = 129 and 181); D3a H16 (Mack hits a poacher 44 to 29 at t = 65).

#### M6-04 Shivat (class 24), part 1: modes, steps, claw, tail, stomps

**2.0 h, C (Claude worker), lane A. Needs: M6-03. From: P5 (part 1), DS03, DS04.**

- **Scope.** The hostile model (names MICHI, KIMAMA, MATOWARI, OKORI, KYOURAN, MANZOKU, COLOR, DIFFENCE: neutral, enraged, calm; COLOR flash 16 toggles about 88 frames), the sleeping state (`+0 = 2`, cannot be hit), step per animation frame, claw bite 14, tail swing 10 with knockdown, triple stomp (attack 0x3A), hit reactions, palettes 158 and 159.
- **Original.** 0x4838C (13,252 bytes), modes in D3b s5, 0x48D8A (burn branch).
- **Data to extract.** animation table 0x49574, box tables 0x103000, 0x103C00, 0x106000.
- **Files.** game/enemy/class24.rs, class24_ai.rs (new).
- **Read.** D3b s1, s5, s13 (P5), A16, A18, A19.
- **Check.** D3b A18 (forced mode 4: HP 60 to 47 at t = 22, 35 at 51, 23 at 232, hit stop 9, +100 each; hp 1 variant: MANZOKU from t = 63) and A19 (stomps at +9, +30, +51 frames), A21 (tail hits Ferris 10 each).

#### M6-05 Shivat, part 2: grab and throw, Z marks, leave, hostile damage

**1.5 h, C (Claude worker), lane A. Needs: M6-04. From: P5 (part 2), DS02, DS05.**

- **Scope.** Grab and hold (0x48E62, cooldown 180, hold about 60 frames, hero -18), the sleeping "Z" marks (pool C class 30 every 120 frames, floats 40 frames, also used by Mack), the leave rule by the record X, scores (dinosaurs give none), poachers hurt by Shivat, hits wake sources other than the hunter if the open item test shows any (D3b open item 1).
- **Original.** 0x48E62, 0x48576, 0x535B4, 0x966B4 (class 30).
- **Data to extract.** class 30 list 0x96770.
- **Files.** game/enemy/class24.rs, game/effect/fx30.rs (new).
- **Read.** D3b s5, s14 items 1 and 4, FX27, A19.
- **Check.** D3b A16 (Shivat `+C0 = 255`, palette 158, class 30 objects at t = 5, 81, 258) and A19 (grab at N = 14: Jack -18 at t = 70, hold not broken by presses). Open item test: a poacher bullet through a sleeping Shivat for 3000 frames and a thrown knife (record the result in errata).

#### M6-06 Dinosaur hunter (class 79) and the wake up scene

**1.5 h, C (Claude worker), lane A. Needs: M6-05, M6-02. From: HT3, HU01 to HU03, DS03.**

- **Scope.** The hunter walks in, swings three times, its third swing pokes the sleeping dinosaur (helper 0x533FA, no HP loss), the dinosaur flashes then turns enraged; the hunter walks left and is freed; a hunter is freed in its first frame when no sleeping dinosaur exists; wave hunters count in waves. Acceptance of the wake up chain.
- **Original.** 0x5F5D0 (2,398 bytes), 0x533FA, 0x5FA5A.
- **Data to extract.** animation tables.
- **Files.** game/enemy/class79.rs (new).
- **Read.** D3a s2, s5, H1 to H4; D3b s5; W221.
- **Check.** D3a H1 (hunter x 1406 at t = 7, 1316 at 67, state 2 at 70, 4 at 155, 6 at 216, attack 5 at 230, Shivat reaction at t = 277, COLOR at 284, OKORI at 373, hunter HP 100, 90, 80, 70 at 385, 406, 427) and H2 (hunter gone after one frame without a dinosaur), H3 (natural Episode 2 order: hunters at t = 825).

#### M6-07 Lash T. (class 3), part 1: three objects and the mace flight

**2.0 h, C (Claude worker), lane A. Needs: M5-01, M6-00. From: HT4 (part 1), HU11.**

- **Scope.** Class 3 init and HP (100 to 300), the mace object A62 and the chain piece A8 (three objects per Lash), mace flight (attack 0x2A, 12 damage at level 0 and 21 at level 31 with the stale `+$84`), walking and decision tables 1 to 3.
- **Original.** 0x358B2 (4,530 bytes), 0x5A47C.
- **Data to extract.** decision tables, animation tables.
- **Files.** game/enemy/class3.rs, class62.rs (new).
- **Read.** D3a s6, H19, s15 (HT4).
- **Check.** D3a H19: Lash stays at x 1280, mace x 1297, 1241, 1157 at t = 300, 304, 308, hero -12 at 315 (level 0), -21 (level 31, HP 300).

#### M6-08 Lash T., part 2: swings, lunge, decision tables, death

**2.0 h, C (Claude worker), lane A. Needs: M6-07. From: HT4 (part 2).**

- **Scope.** Swing states, lunge, decision tables 4 and 5, hit recovery, death, drops and scores (+1,100). Variant 0x400 has no effect found (note).
- **Original.** 0x358B2 (states).
- **Data to extract.** none new.
- **Files.** game/enemy/class3.rs.
- **Read.** D3a s6, s16 item 7, H19, H10.
- **Check.** D3a H19 swing states 4, 0x10, 0x1A, 2 at t = 288, 369, 374, 384 (level 0; 377 at level 31); H10 Lash +1,100. Lash is out of the registry.

#### M6-09 Butcher, part 1: class 5 starter, carcass, talk, rewrite to class 61

**2.0 h, C (Claude worker), lane A. Needs: M5-03, M3-03, M6-00. From: P-D3c-3 (part 1), BS08.**

- **Scope.** Class 5 chopping the dead dinosaur with two blades (carcass A51, helpers A53, A58), talk box B50 var 1 (t 175 to 355), camera mover B76 at 415, music 0x0F recorded, then the class word rewrite to class 61 at t = 475 with HP 576 and the two A58 becoming A52 knives in the same frame.
- **Original.** 0x38DDE (1,172 bytes), 0x58CD6, class 51, 53, 58.
- **Data to extract.** animation tables, intro timeline.
- **Files.** game/enemy/class5.rs, class51_58.rs (new).
- **Read.** D3c s3.1, BS08, BS-B1.
- **Check.** D3c BS-B1: class 5 at X 864, B50 var 1 at t = 175 to 355, B76 at 415, class 5 becomes 61 at t = 475 with HP 576, the two A58 become A52 in the same frame.

#### M6-10 Butcher, part 2: class 61 moves and state table

**2.0 h, C (Claude worker), lane A. Needs: M6-09. From: P-D3c-3 (part 2), BS09.**

- **Scope.** Walk 2 px, slash combo 0x2D and 0x1B (reach 108), far slash 0x2F and 0x30 (reach 140), the leap (apex 157 px, 67 to 154 frames, lands on the hero with his rear), the state table and decision list, rare states 0x24 to 0x34, hit reactions, get up, damage numbers.
- **Original.** 0x59678 (class 61 handler).
- **Data to extract.** state and decision tables.
- **Files.** game/enemy/class61.rs (new).
- **Read.** D3c s3.2 to s3.4, BS09, BS-B2.
- **Check.** D3c BS-B2 state list with the measured durations (b1-nat, b4-face, b5-dmg); the leap apex 157.

#### M6-11 Butcher, part 3: knives class 52, knife drop, target rule, helpers

**2.0 h, C (Claude worker), lane A. Needs: M6-10, M6-02. From: P-D3c-3 (part 3), BS10, BS11.**

- **Scope.** Knives class 52 (thrown and held), knife drop (while he holds a knife a knockdown reaction 4 or 6 drops one with probability 8 in 16, the second made first; the knife becomes an item class 0 var 0x12 a hero can take, the butcher knife item hand over with D2c), re-arm, target choice (lowest HP hero 10 of 16 re-targets below HP 112), poacher groups from class 22 600 frames after the init.
- **Original.** class 52 handler, 0x59678 (knife logic), `+$C2` summon.
- **Data to extract.** none new.
- **Files.** game/enemy/class52.rs, class61.rs.
- **Read.** D3c s3.3 to s3.5, BS10, BS11, BS27; D2c s5 (butcher knife).
- **Check.** D3c BS-B2: knife drops at t = 616, 798, 1665, 1912 style (50 percent per knockdown with a knife left), first poacher group at t = 1110, knives item class 0 var 0x12.

#### M6-12 Butcher, part 4: death, scores, acceptance

**2.0 h, C (Claude worker), lane A. Needs: M6-11. From: P-D3c-3 (part 4), P-D3c-7 (Butcher share), S4.**

- **Scope.** Death (`r4 = 4` at gf + 97, freed at gf + 217, score 20,000, `0xACEE`, music fade), the Butcher acceptance BS-B1, BS-B2, BS-E1 and boss HP by player count, drop rows of the helper lists.
- **Original.** 0x59678 (death).
- **Data to extract.** none.
- **Files.** game/enemy/class61.rs.
- **Read.** D3c s3.6, BS-A1, BS-B1, BS-B2, BS-E1.
- **Check.** D3c BS-A1 (Butcher 576 / 640 / 704), BS-E1; the Episode 2 boss fight from the wave to the tally with the real Butcher. Butcher classes out of the registry.

#### M6-13 Alternate Blade (class 63)

**1.0 h, C (Claude worker), lane A. Needs: M6-02. From: HT6, HU12.**

- **Scope.** Dash 0x1C, one thrown blade (0x19, 12), run off the near side, freed off screen, HP 36 at every level, +1,000, counted in waves.
- **Original.** 0x5AA6E (1,364 bytes).
- **Data to extract.** none new (classes 41 and 60 exist).
- **Files.** game/enemy/class63.rs (new).
- **Read.** D3a s8.1, H20.
- **Check.** D3a H20: dash at t = 6, x 1231 at 21, blade at t = 46, hero -12 at 55, state 4 at 72, run away at 102, freed at 171.

#### M6-14 Effects: water bubbles C5, sweat beads C20, smoke ring C29

**1.2 h, X (external CLI), lane A. Needs: M5-04. From: FX-P7 (5, 20, 29).**

- **Scope.** Pool C class 5 (12 animations of bubbles and wake in deep water, spawned by the player wading code 0x6192 and 0x61F6), class 20 (9 sweat and shock animations, spawn 0x94522), class 29 (ground impact smoke ring, spawn 0x965A2).
- **Original.** 0x92B7A, 0x9454E, 0x965CE.
- **Data to extract.** tables 0x92BF0, 0x945CC, 0x9664E.
- **Files.** game/effect/fx05.rs, fx20.rs, fx29.rs (new).
- **Read.** D3f classes 5, 20, 29.
- **Check.** A wading hero in the Episode 2 swamp shows class 5 objects in the same frames as MAME (probe `jump 2 1 0x7C0`); spawn sites of 20 and 29 documented in errata if no scenario reaches them.

#### M6-15 Episode 2 scenery: swamp water B2, yard tile animation B27

**2.0 h, X (external CLI), lane A. Needs: M3-16. From: B2, B27.**

- **Scope.** B2 swamp water surface (records 0x7550E, rewrites the bottom rows, the same code as the later B8) and B27 (Butcher yard tile animation, region 0x7EEBA). Use the generic block driver.
- **Original.** 0x7542C, 0x7550E, 0x7EEBA.
- **Data to extract.** block tables.
- **Files.** game/prop/b2.rs, b27.rs (new).
- **Read.** D3e part 1 s3, s5, A9, A10; open item 5.
- **Check.** D3e A10: only the bottom 4 tile rows change over the full width, 4 pictures at 24 frame steps; A9 B2 at area 1 start (t = 2480).

#### M6-16 Episode 2 ambience: bird flocks B72 with A46 and A48, jungle sound B80, tile animation B7 var 4

**1.7 h, X (external CLI), lane A. Needs: M3-16. From: B72, A46, A48, B80, B7.**

- **Scope.** B72 bird flock spawner (var 0 at t = 80 and var 0x200 at t = 620), the long tailed bird A46 and its trail sparks A48, B80 jungle sound maker (sound 0x78 recorded), B7 var 4 far or near layer tile animation of Episode 2 area 1.
- **Original.** 0x89984, 0x8A582, 0x76CD2 (var 4).
- **Data to extract.** block tables.
- **Files.** game/prop/b72.rs, b80.rs, b7.rs, game/enemy/scenery.rs.
- **Read.** D3e part 1 s3, s5, A9, W32.
- **Check.** D3e A9: B80 at t = 0, B72 var 0 at 80 and var 0x200 at 620, B7 var 4 at t = 820 of e2a1-mid.

#### M6-17 Episode 2 acceptance: poachers, dinosaurs, Butcher, scenery in the area walk

**2.0 h, C (Claude worker), lane A. Needs: M6-12, M6-16, M6-15, M6-13, M6-14, M6-06, M6-08. From: D3e A9 to A12, D3a H3, IT28.**

- **Scope.** Run the Episode 2 area walk scenarios (D3e part 1 A9 to A12), the D3a H3 natural wake up order, the poacher wave with the dinosaurs (D13: does a poacher bullet or fire wake a neutral dinosaur), the treasures behind the trunks (IT28, the foreground layer over sprites), 2 and 3 player counts. Fix.
- **Original.** Episode 2.
- **Data to extract.** none.
- **Files.** any Episode 2 file, draw.rs (foreground check).
- **Read.** D3e part 1 s5, D3a s2, D4a A21.
- **Check.** Scenarios equal or listed in `re/harness/allow/m6.txt`; Episode 2 has no placeholder (`--list-placeholders` empty for stage 1); D4a A21 pearl and diamond timeline.

#### M6-18 Car, part 1: radio hook, car creation and the arrival timeline

**2.0 h, C (Claude worker), lane A. Needs: M3-09, M5-02, M6-00. From: P-D3c-5 (part 1), BS17, BS18, IT-P5.**

- **Scope.** The radio pick up (treasure in Episode 3 with `G.$50E = 0` calls 0x8BD24, 3 free pool B slots; the sources: containers at X 0x250, 0x580, 0x2080, a rolling barrel, 8 enemy rows), pool B 88 creation (HP 120, X camera - 0xA0, 5 px per frame), the timeline: brake 74 to 100, stop at camera X + 0xC2, r5 states 6, 8, 0xA, 0xC, 0xE, `G.$518 = 1` and camera mode 8 at t = 364, kill-all during the arrival, caption B66 var 0x5B.
- **Original.** 0x8BD24, 0x8BDDC, class 88 states.
- **Data to extract.** arrival tables.
- **Files.** game/car.rs (new), item/small.rs (radio call), prop.rs.
- **Read.** D3c s5.1, s5.2, BS-C1, BS17, BS18; D3e part 2 s3, A202, A203.
- **Check.** D3c BS-C1: car at t = 16 (X 608, Y 5, Z 48, vx 1280), brake 74 to 100 (X 962), r5 = 6 at 101, 8 at 222, 0xA at 283, 0xC at 329, 0xE at 362, `G.$518 = 1` at t = 364. Second radio gives a second car (BS23).

#### M6-19 Car, part 2: steering, speed, lanes

**2.0 h, C (Claude worker), lane A. Needs: M6-18. From: P-D3c-5 (part 2), BS19.**

- **Scope.** Every rider with a stick adds 0x800 plus the stick (+-768 vx, +-256 vz, diagonals) to the target speed, halved for two riders and scaled for three, lane Z 16 to 80, camera X +11.0 px per frame at the right stick, clock frozen, the clamp of the speed table (3 to 18 px).
- **Original.** class 88 steering (D3c 5.4).
- **Data to extract.** none.
- **Files.** game/car.rs.
- **Read.** D3c s5.4, BS-C2.
- **Check.** D3c BS-C2: no key vx 2048, R key 2816, three players 2304 with vz 96; camera X 1299 at t = 400, 2399 at 500, 3499 at 600.

#### M6-20 Car, part 3: collision passes, car HP and the end

**2.0 h, C (Claude worker), lane A. Needs: M6-19, M5-01. From: P-D3c-5 (part 3), BS20, BS21.**

- **Scope.** The passes 0x121D8, 0x122D8, 0x12342, 0x12454 (flat 20 to every enemy and monster it touches with credit to the radio taker, containers HP 0, monsters hurt the car by the record damage), car HP 0 or camera X >= 0xA000 ends the section (flash 10 frames, `r4 = 4`, mode 3, heroes put at camera X + 0x80 / 0xA0 / 0xC0, 120 frames invulnerable, jump arc 48 frames).
- **Original.** 0x121D8, 0x122D8, 0x12342, 0x12454.
- **Data to extract.** none.
- **Files.** game/car.rs, combat.rs.
- **Read.** D3c s5.5, s5.6, BS-C3, BS-C5; D3e part 2 A206.
- **Check.** D3c BS-C3 (every thug loses exactly 20, six tire stacks all HP 0 at t = 404, car HP unchanged until Hogg exists).

#### M6-21 Car, part 4: hero riding state, boarding, seats, delivery actor

**2.0 h, C (Claude worker), lane A. Needs: M6-20, M3-20. From: P-D3c-5 (part 4), BS22.**

- **Scope.** The hero riding state 0x0434, the boarding jump (284 to 332), seats (`+$A2` to `+$A4`: the radio taker, then the other players in order), the delivery NPC arrival actors (0x971D6, 0x86D50: the first absent character in the order Jack, Mustapha, Hannah, Mess), the seat drawing (0x96FD4, 0x971D6), `G.$516` hooks, respawn in the car (no wait, no strike, no bubble).
- **Original.** 0x96FD4, 0x971D6, 0x86D50, player hooks.
- **Data to extract.** seat tables.
- **Files.** game/car.rs, player_car.rs (new), player.rs.
- **Read.** D3c s5.3, s5.7, BS22, BS-C7; D2b s2 (car respawn).
- **Check.** D3c BS-C7: seats `+$A2..$A4` = taker, other players in order; the NPC = the first absent character. Hero state 0x0434 at t = 284.

#### M6-22 Car, part 5: scenery B89 and B90, car effects C10, C32, C36, C37, C41

**2.0 h, C (Claude worker), lane A. Needs: M6-21, M5-04. From: P-D3c-5 (part 5), FX-P6, FX28.**

- **Scope.** Pool B 89 (desert scenery) and 90 (chained captions) once their context is traced (D3e part 2 open item 3), the car effects: class 36 radio exhaust plume (sounds 0x3E, 0x3F recorded), 37 tire burnout smoke and skid tracks, 32 exhaust puff, 10 vehicle debris scatter (also Hogg's bike), 41 car gun directional spark (gun 0x8C3A6).
- **Original.** 0x96FF4, 0x9720A, 0x96A7E, 0x932D2, 0x97800, 0x8C96C, 0x8CABE.
- **Data to extract.** tables 0x97150, 0x972B2, 0x93426, 0x9786E.
- **Files.** game/effect/fx10.rs, fx32.rs, fx36_37.rs, fx41.rs (new), game/prop/b89_90.rs.
- **Read.** D3f classes 10, 32, 36, 37, 41, FX28, FX-S09; D3e part 2 open item 3, W241; D3c s5.2.
- **Check.** D3f FX-S09: class 36 and 37 objects at the radio call frames equal MAME (BS-C1 segment).

#### M6-23 Hogg (classes 31, 32), part 1: rider and bike, foot behaviour

**2.0 h, C (Claude worker), lane A. Needs: M6-22, M3-03. From: P-D3c-4 (part 1), BS12.**

- **Scope.** Rider class 31 on the bike class 32 (the moving body): foot behaviour (ride across at 5.5 px per frame 116 to 127 frames, wait 121, return), the 256 cell grid and zone maps (0xC821A, 0xC6252, 0xC861A), 3 vitality bars, rides that hurt (10 at level 14), never flinches.
- **Original.** 0x4FCB4 (class 31 rider), 0x50FAC (class 32 bike), 0x5078E, 0x4FF74, maps 0xC821A, 0xC6252, 0xC861A.
- **Data to extract.** zone maps (dino-extract/src/boss.rs).
- **Files.** game/enemy/class31.rs, class32.rs (new).
- **Read.** D3c s4.1 to s4.5, BS12, BS-H1.
- **Check.** D3c BS-H1 (ride X 11955 to 11328 in 116 frames, wait 121, ride 11414 to 12091 in 127 frames, 5.5 px per frame).

#### M6-24 Hogg, part 2: car behaviour grid and the helpers

**2.0 h, C (Claude worker), lane A. Needs: M6-23. From: P-D3c-4 (part 2), BS15, BS23.**

- **Scope.** Car mode behaviour: a random walk on a 32 by 8 cell grid around the car re-drawn every 25 frames, bike 72 to 346 px ahead at 11 px, beside the car at 8, Hogg switches to car behaviour when a radio is taken during the fight (a second radio gives a second car).
- **Original.** class 31 car mode, tables 0x50BD2, 0x50C62.
- **Data to extract.** grid tables.
- **Files.** game/enemy/class31.rs.
- **Read.** D3c s4.4, s4.7, BS-C4, BS-C6.
- **Check.** D3c BS-C4: Hogg created at camera X 11551 (t = 1332), bike 72 to 346 px ahead (mean +203) at 11 px per frame.

#### M6-25 Hogg, part 3: bomb class 42 and the throw logic

**1.5 h, C (Claude worker), lane A. Needs: M6-24, M5-02. From: P-D3c-4 (part 3), BS13, BS14.**

- **Scope.** Bomb throws (roll every 60 frames, 30 in car mode, `rng & 31 <= T[level]`, 1 to 4 bombs per throw by his HP quarter), bomb class 42 (lob to a 3 by 3 grid around the car, weights 1 2 1 / 2 3 3 / 1 2 1 of 16, 40 or 59 frames, blast hurts the car 12 plus 12 per bomb).
- **Original.** 0xC821A.., class 42 handler.
- **Data to extract.** grid weights.
- **Files.** game/enemy/class42.rs (new).
- **Read.** D3c s4.6, BS13, BS14, BS-H1 (bombs).
- **Check.** D3c BS-H1 first bomb at t = 17 (var 4), hits on Jack 10 (ride) and 15 (bomb) at level 14, volleys of 3 at HP 70; BS-C4 car loses 12 + 12 per bomb, HP 0 after 5 pairs.

#### M6-26 Hogg, part 4: death, bike pieces, scores, acceptance of the car and Hogg

**1.5 h, C (Claude worker), lane A. Needs: M6-25. From: P-D3c-4 (part 4), P-D3c-7 (car and Hogg share), S4.**

- **Scope.** Death (rider thrown vy 5 up vx 1, the bike breaks into pieces and explodes where it lands, single Hogg, score 1,000 on foot only; the car killing Hogg gives the score to the radio taker per the code), the BS-C4, BS-C5, BS-E1 chains, first reinforcement thug at t = 2376, music 0x20, 0x0F.
- **Original.** 0x97D2, 0x9796 (bike pieces), class 31 death.
- **Data to extract.** none.
- **Files.** game/enemy/class31.rs, class32.rs.
- **Read.** D3c s4.8, BS16, BS-C4, BS-C5, BS-E1; open item 2.
- **Check.** D3c BS-C4 (car HP 0: flash 10 frames, `r4 = 4` at t = 1893, `G.$518 = 0` and mode 3 at 1894), BS-C5 (rams -20 and a flinch; at HP 0 the scripted mask at 1360), BS-A1 (Hogg 300 / 400 / 500).

#### M6-27 Bomb carriers (classes 64 and 77), dynamite

**1.5 h, C (Claude worker), lane A. Needs: M5-06, M5-02. From: HT7 (part 1), HU13, HU26.**

- **Scope.** Class 64 (alternate Punk and Wrench: HP 20 fixed, hop and run, throws a dynamite stick, runs when hit) with class 77 (the dynamite), the knock down drop of the stick as a weapon item (HU26), +1,100 or +80,100 for var 0x200.
- **Original.** 0x5AFC2 (2,288 bytes), 0x5EDF4, 0x5EFAA to 0x5EFDE.
- **Data to extract.** byte code tables 0x5B326, 0x5B4E6.
- **Files.** game/enemy/class64.rs, class77.rs (new).
- **Read.** D3a s8.2, H21, H24, H27, HU26; D2c s6.
- **Check.** D3a H24 (HP 20, runs right vx 768 then 1152, freed at t = 146 or 90), H27 (carrier hops from x 1277 to 1391, E class 0 var 0x18 at x 1194 t = 38 to 68, E class 1 var 3 t = 70 to 160).

#### M6-28 Bomb carriers, part 2: the explosion against heroes, Episode 3 group

**1.0 h, C (Claude worker), lane A. Needs: M6-27. From: HT7 (part 2), WM26.**

- **Scope.** Pool E explosion kind 3 hurting heroes (8 to 19, owner 0xFF, no depth test), the four carriers at X 0xEC0 of Episode 3 and their drops.
- **Original.** 0x27E16.
- **Data to extract.** none.
- **Files.** item/blast.rs, class64.rs.
- **Read.** D3a s8.2, H21; D2c s6.
- **Check.** D3a H21: four class 64 at X 0xEC0, explosion objects at t = 214 at x 3567, z 67, 51, 35, hero -10 at 219, -10 at 438, -19 at 672.

#### M6-29 Hazards: barrel class 65 and biker class 70

**1.5 h, C (Claude worker), lane A. Needs: M5-02, M5-04. From: HT9 (part 1), HU14, IT27.**

- **Scope.** Class 65 (bouncing and rolling barrel, HP 8, hurts heroes and enemies (10 raw), a hero's hit breaks it and drops by the container table, freed 0x40 left of the camera so a fast hero outruns it, the cave variant 0x201 falls with a floor shadow) and class 70 (biker: still until t = 36 then 4.5 px per frame rising to 7, kick 17 to enemies, drops by the enemy table).
- **Original.** 0x5B8B2 (728 bytes), 0x5C356 (1,218 bytes).
- **Data to extract.** tables in the handlers.
- **Files.** game/enemy/class65.rs, class70.rs (new).
- **Read.** D3a s10, H11, H13, H15, H20 row; D4a A20; D3e part 3 W307.
- **Check.** D3a H11 (x 1390 at t = 6, vx -512, hops vy 768 at t = 80 and 180, x 962 at 220; var 0x201 y 320 then vy -3,712 at 60), H13 (still until 36 then 4.5 px per frame: x 1311 at 60, 1147 at 90 at vx -1,800); D4a A20 drops.

#### M6-30 Hazard: rock class 66 and the monster hooks

**1.0 h, C (Claude worker), lane A. Needs: M6-29. From: HT9 (part 2), IT-P7.**

- **Scope.** Class 66 (bouncing rock: y 207 at t = 6, vx -448, state 4 toward the camera at 90), break pieces and effects (class 11 stones), drops by the container table, Episode 5 area 0 records (3 + 3, one for 2P+). The stone limit rule of W225.
- **Original.** 0x5BB8A (732 bytes), 0x92EA4, 0x94640, 0x29650.
- **Data to extract.** none.
- **Files.** game/enemy/class66.rs (new).
- **Read.** D3a s10, H12; D3e part 2 A217, W223 to W225; D4a s8.4.
- **Check.** D3a H12 (y 207 at t = 6, 158 at 30, state 4 at 90 with vz); D3e A217 (stone at t = 168 in the repeated spawn test, not guaranteed).

#### M6-31 Episode 3 scenery: sky blend B95, tire stacks and radio sources

**2.0 h, C (Claude worker), lane A. Needs: M3-16, M6-22. From: B95, P205, W202, W203, W205.**

- **Scope.** B95 sky colour blend (16 colours per instance, one step per RGB nibble, periods 15, 15, 2, 2, 2, 2, 2, three destination slots), the six tire stacks (E5 containers HP 4, tire puffs, no item), the radio source rows and the empty containers as data, the row scroll look of the road with the car.
- **Original.** 0x92220, 0x922D0, 0x9227A.
- **Data to extract.** tables 0x922D0.
- **Files.** game/prop/b95.rs (new), item (rows).
- **Read.** D3e part 2 s3, A201 to A206, W201 to W206; D4a IT20.
- **Check.** D3e A204 (final object sample at t = 214, gone by 216) and A206 (every tire stack HP 4, tire puffs, no item).

#### M6-32 Episode 3 acceptance: foot path and car path

**2.0 h, C (Claude worker), lane A. Needs: M6-26, M6-28, M6-30, M6-31. From: D4c A10 to A13, D3e A201 to A205, D3c BS-C*, BS-H*.**

- **Scope.** Run D4c A10 (radio, car, Hogg), A11 (foot locks), A13 (row scroll), D3e A201 to A205 and the D3c car and Hogg scenarios with 1 and 3 players, the open item of D3c 3 (a full foot run with the attack key and the clock), fix.
- **Original.** Episode 3.
- **Data to extract.** none.
- **Files.** any Episode 3 file.
- **Read.** D3c s5, s10; D4c s7.
- **Check.** Scenarios equal or listed in `re/harness/allow/m6.txt`; `--list-placeholders` empty for stage 2; the main session plays Episode 2 and 3 from the coin.

### M7: Episodes 4 and 5

Episode 4: Slice, the garage props, Walther and the bikers (already done). Episode 5: Bludge, Zeke, the hostile Rock Hopper, the shared boss skeleton, Morgan and Morgue, the fire village scenery.

Hours: 49.5 (lane A 47.5, lane B 2.0), 28 tasks.

#### M7-00 Prepare scenarios for Episodes 4 and 5

**2.0 h, X (external CLI), lane B. Needs: M3-06. From: D3c, D3d, D3b, D3a, D3e part 2 (prep).**

- **Scope.** Record with the HX tool: D3c BS-S1, BS-S2; D3d BT-A1 to BT-A5; D3a H17, H18, H26 (Bludge); D3b A1 to A8 (Zeke) and A12 to A14 (Hopper); D3e part 2 A207 to A224. Prepare only.
- **Original.** none.
- **Data to extract.** none.
- **Files.** re/harness/scripts/, re/harness/pokes/.
- **Read.** the sections named, M3-06.
- **Check.** Each scenario has a MAME trace.

#### M7-01 Slice (class 25), part 1: init and intro

**2.0 h, C (Claude worker), lane A. Needs: M5-03, M3-20, M7-00. From: P-D3c-6 (part 1).**

- **Scope.** Slice init, name, HP row, the intro states (r6 at t = 0, 110, 297, 337, 342, 403 with durations 110, 187, 40, 5, 61, alive at about 404), music 0x10 recorded. The Episode 4 opening actors A27 and A43 exist from M3-20.
- **Original.** 0x4BE70 to 0x4D48A (5,658 bytes), intro part.
- **Data to extract.** tables 0x4D268, 0x4D2E8.
- **Files.** game/enemy/class25.rs, class25_ai.rs (new).
- **Read.** D3c s6.1, s6.2, BS-S1.
- **Check.** D3c BS-S1 intro timeline (M s1-nat).

#### M7-02 Slice, part 2: the AI, kick, long slash, speeds

**2.0 h, C (Claude worker), lane A. Needs: M7-01. From: P-D3c-6 (part 2), BS24.**

- **Scope.** Decision tables (0x4C4AE, 0x4C61A, 0x4C79A, 0x4C7BA), action routines 0x4C528 to 0x4C6DA, kick 0x36, long slash 0x37 (reach 138), run speed, repositioning, hit reactions and damage numbers, wake up behaviour.
- **Original.** 0x4C528 to 0x4C6DA.
- **Data to extract.** tables as listed.
- **Files.** game/enemy/class25.rs.
- **Read.** D3c s6.2, s6.3, BS24, s10 item 5.
- **Check.** Slice HP track 500 to 0 in the s1-nat segment; damage numbers of the s-dmg segments.

#### M7-03 Slice, part 3: boomerang class 19 and the afterimage class 27

**2.0 h, C (Claude worker), lane A. Needs: M7-02, M5-04. From: P-D3c-6 (part 3), BS25, BS26, FX26.**

- **Scope.** Class 19 (single boomerang var 0, 2, 4 base 7, fan of 12 var 6 base 13 knock down, 30 to 44 frames in the air, hit by heroes through the monster list and falls, the thrower catches it on return, the spin invulnerability) with table 0x453FE; pool C class 27 afterimage (every 7 frames while running, 36 frames, palette bytes 0x9D, 0x9C, 0x9B, 0x9A, 0x8C, 0x8B, drawn every other frame, sound 0x22). Class 19 and class 27 are shared with the Slisaurs in M8.
- **Original.** 0x453FE, 0x9639E, 0x963D6.
- **Data to extract.** table 0x453FE.
- **Files.** game/enemy/class19.rs, game/effect/fx27.rs (new).
- **Read.** D3c s6.4, s6.5, BS25, BS26; D3f class 27.
- **Check.** D3c BS-S2 (one class 27 object every 7 frames while running, each lives 36 frames, palette steps of 6 frames); BS-S1 fans at t = 4701, 4947, 5130 (HP 70, 13, 13).

#### M7-04 Slice, part 4: death, scores, acceptance

**1.0 h, C (Claude worker), lane A. Needs: M7-03. From: P-D3c-6 (part 4), P-D3c-7 (Slice share), S4.**

- **Scope.** Death (score 40,000, `0xACEE`), boss HP by players (500 / 600 / 750), the Slice acceptance BS-S1, BS-S2, BS-E1.
- **Original.** class 25 death.
- **Data to extract.** none.
- **Files.** game/enemy/class25.rs.
- **Read.** D3c s6.6, BS-S1, BS-E1.
- **Check.** BS-A1 (Slice 500 / 600 / 750), BS-S1 death at t = 5414, BS-E1. Slice out of the registry; the main session plays the Episode 4 boss.

#### M7-05 Episode 4 scenery: wall panels B0 (15 variants) and foundry palettes B59

**1.7 h, X (external CLI), lane A. Needs: M3-16. From: P202, P204, B0, B59.**

- **Scope.** B0 garage fixtures (thin stem lamps variants 1 and 2 with round tops, the large column variant 0, window and other variants: patch layouts, boxes, hit states, tile and attribute patches, effects C21 and C24, no loot) and B59 (three palette subranges, nine call reload interval, stage guard).
- **Original.** 0x74FA2 to 0x75238, boxes 0x750B4, 0x861E8, 0x862B6.
- **Data to extract.** variant patch layouts, palette tables.
- **Files.** game/prop/b0.rs, b59.rs (new).
- **Read.** D3e part 2 s5, s6, A210, A213, W212, W213, W215, W216.
- **Check.** D3e A210 (each right facing jab hits at t = 22 with the right variant, two stems snap, the large column becomes a stump) and A213 (three B59 variants animate).

#### M7-06 Episode 4 acceptance

**2.0 h, C (Claude worker), lane A. Needs: M7-04, M7-05, M6-30. From: D3e A207 to A214, D3a H22.**

- **Scope.** Run D3e part 2 A207 to A214, D3a H22 and H23 (opening car), the Walther vent entry (A209), the bikers (H13), the drums and tires drops (D4a A18, A19, A25), 1 and 3 players, fix.
- **Original.** Episode 4.
- **Data to extract.** none.
- **Files.** any Episode 4 file.
- **Read.** D3e part 2 s4 to s6.
- **Check.** Scenarios equal or listed in `re/harness/allow/m7.txt`; `--list-placeholders` empty for stage 3.

#### M7-07 Bludge (class 6), part 1: init, entrances, hop AI

**2.0 h, C (Claude worker), lane A. Needs: M5-01, M7-00. From: HT5 (part 1), HU10.**

- **Scope.** Class 6 with part 0, the 4 entrances (0 hops 72 px, 4 crouches then hops, 6 leaps 217 px, others), the hop AI, colours (red in Episode 2 arenas, green with orange mane later), HP by level, hit reactions.
- **Original.** 0x39272 (7,826 bytes).
- **Data to extract.** animation tables, decision tables.
- **Files.** game/enemy/class6.rs, class6_ai.rs (new).
- **Read.** D3a s7, H17, H18, s15 (HT5).
- **Check.** D3a G09 rows: bl-v0-l0, bl-v6-l0, bl-v4-l0 movement frames equal MAME in the scenario batch.

#### M7-08 Bludge, part 2: tongue, charge, lunge, acid spit class 7, vomit effect

**1.5 h, C (Claude worker), lane A. Needs: M7-07. From: HT5 (part 2), FX-P7 (class 18).**

- **Scope.** Attacks 0x25 (charge), 0x24 (lunge), 0x26 (long tongue) and the acid spit object A7 (lobbed, attack 0x27, 10 damage with a push), pool C class 18 (vomit and food spit effect, spawned at 0x3B1E8).
- **Original.** 0x39272 (attacks), A7 object, 0x942C6.
- **Data to extract.** table 0x9430E.
- **Files.** game/enemy/class6.rs, class7_acid.rs (new), game/effect/fx18.rs.
- **Read.** D3a s7, H17 row; D3f class 18.
- **Check.** D3a: tongue strike and spit damage -10 at t = 920 of bl-v4-l0; sheet sc-e7-bludge t = 280 and 520.

#### M7-09 Bludge, part 3: eating floor items, giving back, death drops

**1.5 h, C (Claude worker), lane A. Needs: M7-08. From: HT5 (part 3), HU09, HU25.**

- **Scope.** Item claim by the lowest kind (food, weapon, treasure, up to 8 in a queue, reach about 75 to 80 px, `+$AF`, `+$D0`, `+$D8`, `+$B6`), the eating state, spit out all items on death and the last one when hit while eating, only at higher aggression.
- **Original.** 0x3B006 to 0x3B0D8, 0x3AFB2, 0x3B086.
- **Data to extract.** none.
- **Files.** game/enemy/class6.rs, item.rs (pool E hooks).
- **Read.** D3a s7, H17, H18, H26, s16 items 4 and 5, HU25.
- **Check.** D3a H17 (the eaten items claimed, treasure gone at t = 990, `+D6 = 1`, `+B6 = 255`), H18 (an item reappears at t = 1166 at x 1450), H26 (food claimed at t = 8, gone at 74). Bludge out of the registry.

#### M7-10 Zeke (class 82), part 1: init, target rule, modes, patrol, swoop

**1.3 h, C (Claude worker), lane A. Needs: M5-01, M7-00. From: P2 (part 1), DS06.**

- **Scope.** Init (HP 20, damage x2 with type factor 64, one punch or jump kick kills), the target rule (two Zekes pick different heroes, `+$116`), modes 0, 2, 4, 6, 8, patrol heights 74 to 203, the attack roll every 180 frames (9, 16, 24, 30 in 32 by level), swoop 0x5B and triple peck 0x5C.
- **Original.** 0x606D4 (3,720 bytes), tables 0x6137A, 0x6141A, 0x6142C, 0x6143C, 0x614DC, 0x61504 to 0x61526, 0x611D2.
- **Data to extract.** animation tables 0x611D2, tables listed.
- **Files.** game/enemy/class82.rs (new).
- **Read.** D3b s2, s13 (P2), A1 to A3.
- **Check.** D3b A1 (mode changes (2, 0) (185, 2) (335, 0) (517, 4) ..., Jack HP 100 to 80 at t = 260, 60 at 602), A2 (first attack only at t = 3262 at level 0), A3 swoop (launch at t = 972, low point y 46 at 1031).

#### M7-11 Zeke, part 2: dive grab, hit handler, dying, leave

**1.3 h, C (Claude worker), lane A. Needs: M7-10. From: P2 (part 2), DS07 to DS09.**

- **Scope.** Dive grab 0x3B (hold 31 frames, 18 damage, only a grounded hero in normal control, a jump dodges it, the carry goes toward world X 0xC0: code the quirky direction read as "compare with 0xC0"), grab link and release 0x125C0, the hit handler, dying by flying away (no explosion, no score), leave straight up when camX >= `+5C`, sounds 0xA0CC recorded, the Episode 8 start mode.
- **Original.** 0x125C0, 0x606D4.
- **Data to extract.** none.
- **Files.** game/enemy/class82.rs.
- **Read.** D3b s2, s14 item 12, A3 to A8.
- **Check.** D3b A3 dive (grab at t = 1010 y 82, Jack -18 at 1051), A4 (jump at t = 1000 dodges), A5 (carry left at X 0x700), A6 and A7 (punch kills at height 0 to 72, not 96; jump kick kills to 96), A8 (mode 8 leave). Zeke out of the registry.

#### M7-12 Rock Hopper, part 2: hostile modes and attacks (class 29)

**2.0 h, C (Claude worker), lane A. Needs: M5-10, M7-00. From: P4 (part 2), DS05, DS14.**

- **Scope.** The modes 0, 2, 4, 6, 8, 0xA, 0xC, 0xE for the scripted variant 0x200 (Episode 5), neutral timers (600 frames MATOWARI then 1 in 16 per 10 frames to anger, leave after 1200), attacks 0x20, 0x21 (bite 16, 33 at level 31), 0x29, the pounce hold, block DIFFENCE (12.5 percent), KYOURAN after 5 hits, the burn branches (record reaction 0xA or 0xC, flames, palette 0x84).
- **Original.** 0x4DCF6 (7,708 bytes), 0x4EE1A.
- **Data to extract.** tables from M5-10.
- **Files.** game/enemy/class29.rs.
- **Read.** D3b s4, s11, A12 to A14, open items 2 and 5.
- **Check.** D3b A12 (modes 2 at t = 4, 4 at 104, 0xC at 887, 6 at 976, 0xA at 2042), A13 (HP 44 to 32 at t = 22, COLOR toggle frames, OKORI at 120, Jack -16 at 140), A14 (bite -16 and -33). Open item test: a torch or flame against an awake Hopper and Shivat (record in errata).

#### M7-13 Shared boss skeleton, part 1: phase flag, timers, aggression gate, action tables

**1.7 h, C (Claude worker), lane A. Needs: M7-04, M6-12. From: P-D3d-1 (part 1).**

- **Scope.** The framework of D3d 1.2: phase flag with timer tables, the aggression gate, 32 by 8 action tables by class and form.
- **Original.** 0x2B56C and the tables of D3d s1.
- **Data to extract.** action tables (dino-extract/src/boss.rs).
- **Files.** game/boss_common.rs (new), dino-extract/src/boss.rs.
- **Read.** D3d s1 (all).
- **Check.** Unit tests of the gate and table lookups against values listed in D3d s1; no behaviour change in Vice, Butcher, Slice (regress.sh and the BS scenarios unchanged).

#### M7-14 Shared boss skeleton, part 2: recover entries, target rule, scores

**1.7 h, C (Claude worker), lane A. Needs: M7-13. From: P-D3d-1 (part 2).**

- **Scope.** Recover entries (0x2B56C), the target rule 0x32032 reuse (fewest attackers, ties to the later player), scores 0x325E6, `G.$4D6`, `G.$50E`, 0x6148 hooks.
- **Original.** 0x32032, 0x325E6, 0x6148.
- **Data to extract.** none.
- **Files.** game/boss_common.rs.
- **Read.** D3d s1, BT22.
- **Check.** Unit tests; BS scenarios unchanged.

#### M7-15 Shared boss skeleton, part 3: end timeline hooks, placeholders use it

**1.6 h, C (Claude worker), lane A. Needs: M7-14. From: P-D3d-1 (part 3).**

- **Scope.** The boss end timeline hooks (dying, caption, walk, `0xACEE`) shared by the D3d bosses, and the placeholder bosses of Episodes 5 to 8 rebuilt on the skeleton so that each real boss replaces one class only.
- **Original.** 0xACEE and the boss end routines.
- **Data to extract.** none.
- **Files.** game/boss_common.rs, placeholder.rs.
- **Read.** D3d s1, s7.
- **Check.** Autoplay passes through Episodes 5 to 8 with the rebuilt placeholders (M3-29 flow numbers unchanged).

#### M7-16 Morgan scene (classes 35, 45, 67, bullets class 59 variant 5)

**2.0 h, C (Claude worker), lane A. Needs: M7-15, M5-12. From: P-D3d-2 (part 1), HT11, HU20, BT05.**

- **Scope.** Class 35 walks left 60 frames from t = 290, points at t = 352, fires four class 59 var 5 bullets (8 px, 30 frames) at t = 406, 410, 414, 418 at the villager class 45, the villager goes down, and at t = 662 the class word of the slot is rewritten to class 36 with HP 256. Class 67 (Uzi item). The chief talk box lines (B50) are in M7-25.
- **Original.** 0x513DA (696 bytes), 0x51658, 0x5639A (466 bytes).
- **Data to extract.** scene tables.
- **Files.** game/enemy/class35.rs, class45.rs, class67.rs (new).
- **Read.** D3d s2.1, BT05, BT-A1; D3a s11; D3e part 2 A223.
- **Check.** D3d BT-A1 (`m5-scene`): walk, point, four bullets at t = 406, 410, 414, 418, class word 36 at t = 662 to 664, HP 256.

#### M7-17 Morgan (class 36), part 1: gate and actions

**2.0 h, C (Claude worker), lane A. Needs: M7-16. From: P-D3d-2 (part 2), BT06.**

- **Scope.** The gate 0x73B14, action tables 0x73B54 and 0x73C54, approach (44 to 104 frames), kite (60 to 143), hop (38 frames), phase flag flips at t = 816 and 1096, hit reactions and knock down, HP 256, score 0.
- **Original.** 0x51692 (class 36 handler).
- **Data to extract.** tables as listed.
- **Files.** game/enemy/class36.rs (new).
- **Read.** D3d s2.2, s2.3, BT06, BT-A2, BT-A3, s9 (c) item 1.
- **Check.** D3d BT-A2 decisions (0x08, 0x10, 0x20, 0x24, 0x1E, 0x28, 0x2A with the listed durations) and BT-A3 (one Uzi bullet 11 damage: HP 256 to 245, slid 108 px in 40 frames).

#### M7-18 Morgan, part 2: throws, grenades, Uzi burst

**2.0 h, C (Claude worker), lane A. Needs: M7-17, M6-27. From: P-D3d-2 (part 3).**

- **Scope.** Throws (class 68 var 2, 3, 4 objects, 30 frames per object, 3 to 5 objects, rocks and knives and lava rock horizontal speed rule), grenades (class 26 var 0, 80 frames), the Uzi burst (40 frames, 5 shots 8 frames apart, Uzi class 59 bullets) and their damage (Morgan's own hits are 1 above the formula: keep the measured numbers).
- **Original.** class 36 attack routines, class 68 handler, class 26.
- **Data to extract.** none.
- **Files.** game/enemy/class36.rs, class68.rs, class26.rs (new).
- **Read.** D3d s2.4, BT06, s9 (c) items 1 and 7.
- **Check.** D3d BT-A2: states 0x24 (30 frames per object), 0x1E (grenade 80 frames), 0x20 (Uzi burst 40 frames, 5 shots 8 frames apart); damage 0x3F 15 and 0x34 18.

#### M7-19 Morgan, part 3: death talk, rewrite to Morgue, Uzi drop, acceptance

**2.0 h, C (Claude worker), lane A. Needs: M7-18. From: P-D3d-2 (part 4), BT-A4.**

- **Scope.** Morgan dying (`+4 = 4` at t = 2380), caption 120 frames, walk 60 frames to the centre, the Uzi item on the floor from t = 2520, rewrite to class 40 at t = 2682 with HP 512, the 136 frame intro with palettes 0x9E, 0x9D, 0x9F, 0x6E, 0xAC, 0x00 every 2 frames, music 0x0E.
- **Original.** class 36 death, class 40 intro.
- **Data to extract.** none.
- **Files.** game/enemy/class36.rs, class40.rs (new).
- **Read.** D3d s2.5, BT-A4.
- **Check.** D3d BT-A4 (`m5-trans`, `m5-b`) frame numbers equal. Morgan out of the registry.

#### M7-20 Morgue (class 40), part 1: moves, roar and charge

**2.0 h, C (Claude worker), lane A. Needs: M7-19. From: P-D3d-3 (part 1), BT07.**

- **Scope.** Moves 0x40, 0x6C, 0x4B, 0x41 (charge 7 px per frame), 0x42, 0x2B, 0x31, 0x4A, the roar (40 frames, 5 sparks) then the charge, gate 0x73D54, hit reactions, damage 20 (0x6C), 17 (0x4B), 23 (0x42).
- **Original.** 0x53E08 (class 40 handler).
- **Data to extract.** tables 0x73D54 and the action table.
- **Files.** game/enemy/class40.rs.
- **Read.** D3d s2.6, BT07, BT-A5.
- **Check.** D3d BT-A5 (`m5-morgue-dmg`): Morgue hits for 20, 17, 23.

#### M7-21 Morgue, part 2: death, scores, Episode 5 chain acceptance

**2.0 h, C (Claude worker), lane A. Needs: M7-20. From: P-D3d-3 (part 2), P-D3d-10 (Morgan and Morgue share), S4.**

- **Scope.** Death (30,000 points, `0xACEE`, `G.sub` 2 at t = 3927, 4 at 4295, 6 at 6096, panel at 7054), HP by players, the whole Morgan to Morgue chain acceptance.
- **Original.** class 40 death.
- **Data to extract.** none.
- **Files.** game/enemy/class40.rs.
- **Read.** D3d s2.7, BT-A5.
- **Check.** D3d BT-A5 kill timeline equal; Episode 5 boss fight from the scene to the tally.

#### M7-22 Episode 5 scenery A: far layer animations B5, B6 and fire strips B10

**1.8 h, X (external CLI), lane A. Needs: M3-16. From: P206, P207, B5, B6, B10.**

- **Scope.** B5 (chimney smoke far layer, 16 frame period), B6 (interior flames far layer, 12 frame period, middle split), B10 (two fixed foreground fire strips on the tile clock, camera lifetime).
- **Original.** 0x769E2 to 0x76B46, 0x785F0.
- **Data to extract.** patch patterns.
- **Files.** game/prop/b5.rs, b6.rs, b10.rs (new).
- **Read.** D3e part 2 s7 (area 1 tile and palette animation), A220, W228 to W230.
- **Check.** D3e A220 (`e5-far-isolate-b00`, `e5-far-isolate-f00`): layer, phase and priority equal, no missing dark columns.

#### M7-23 Episode 5 scenery B: burning huts B13, variants 1 to 5

**1.3 h, C (Claude worker), lane A. Needs: M7-22. From: P208 (part 1).**

- **Scope.** B13 as data driven patch descriptions: the first variants and their placed records (eight records, two different variants at X 0xE20), irregular patches with attributes, overlap and initialisation order.
- **Original.** 0x79218 to 0x7B0BE, variant routines 0x7863E, 0x787F4, 0x7925E.
- **Data to extract.** patch descriptions.
- **Files.** game/prop/b13.rs (new).
- **Read.** D3e part 2 s7, W230, W231, open item 6.
- **Check.** Pictures of the burning village at camera X 0xB00, 0xC00, 0xD00 equal MAME (picture compare) with the camera jump limit in mind (W242).

#### M7-24 Burning huts B13, variants 6 to 9 and the full village picture

**1.2 h, C (Claude worker), lane A. Needs: M7-23. From: P208 (part 2).**

- **Scope.** The remaining variants, overlap checks, the full area 1 picture.
- **Original.** as M7-23.
- **Data to extract.** patch descriptions.
- **Files.** game/prop/b13.rs.
- **Read.** D3e part 2 s7, open items 1 and 6.
- **Check.** Pictures along the bridge descent (camera X 0xD50 to 0xF00) equal MAME in the natural run capture.

#### M7-25 Episode 5 events: fire palette B62, no-op B48, wall burst B49, rifle B70

**1.9 h, X (external CLI), lane A. Needs: M5-03, M3-16. From: P209, P210, P211, P212, B62, B48, B49, B70.**

- **Scope.** B62 (six palette phases, sound 0x8E recorded, the hardware word toggle at 0x800142 is recorded but not shown, open item 2), B48 (waits 128 calls then frees, no cliff rock spawner), B49 (wave threshold creates the wall burst: Y adjustment, 20 tile code words, 4 attributes, class 21 var 0x16, no second portrait box) and B70 (decorative rifle art, floor draw, despawn). The chief scene text lines use B50 from M5-03.
- **Original.** 0x865F2, 0x84316, 0x8438E, 0x843C8, 0x843FA, 0x896E6.
- **Data to extract.** palette table, tile blocks.
- **Files.** game/prop/b62.rs, b48.rs, b49.rs, b70.rs (new).
- **Read.** D3e part 2 s7, A216, A222, A223, W222, W232, W237 to W240.
- **Check.** D3e A222 (twenty tile code writes, four attribute changes, C21 var 0x16, no two B50 portraits) and A223 (chief scene, rifle uncollectable).

#### M7-26 Episode 5 acceptance, part 1: meadow, hunters, barrels, rocks, buckets, torches

**2.0 h, C (Claude worker), lane A. Needs: M7-21, M7-12, M7-11, M7-09, M7-25. From: D3e A215 to A218, A221, A224, D3a H4.**

- **Scope.** Run D3e part 2 A215 to A218, A221, A224 and D3a H4 (static hunter at x 931), the bazooka and explosive barrels, the water buckets (special bucket needs a live gun on the field, fossil bucket, cake or coffee by level), torches and eggs, the 2P and 3P gated records. Fix.
- **Original.** Episode 5.
- **Data to extract.** none.
- **Files.** any Episode 5 file.
- **Read.** D3e part 2 s7, s8.
- **Check.** Scenarios equal or listed in `re/harness/allow/m7.txt`.

#### M7-27 Episode 5 acceptance, part 2: bridge, village, whole episode

**2.0 h, C (Claude worker), lane A. Needs: M7-26, M7-24. From: D3e A219, A220, open items 1 and 12, D4c.**

- **Scope.** Capture the complete natural bridge crossing (open item 1 of D3e part 2), the diagonal camera, the village fire, the Morgan and Morgue chain with 1, 2 and 3 players; whole Episode 5 run from the area 0 start to the tally.
- **Original.** Episode 5.
- **Data to extract.** none.
- **Files.** any Episode 5 file.
- **Read.** D3e part 2 s7, s12; D4c s2.3.
- **Check.** Camera Y 0x200 to 0x100 from camera X 0xD50 to 0xF00 equals MAME; `--list-placeholders` empty for stages 3 and 4; the main session plays Episodes 4 and 5.

### M8: Episodes 6, 7 and 8

Episode 6: the foot, the storm, the mine, Tyrog three times. Episode 7: the lift, the library gates, the lab, the Slisaurs. Episode 8: the tanks, the sewers, the flame vents, the cave, Fessenden in two forms, and the real ending trigger. Pool B sizes follow D3e part 3 (it replaces the index sizes).

Hours: 66.4 (lane A 64.4, lane B 2.0), 37 tasks.

#### M8-00 Prepare scenarios for Episodes 6 to 8

**2.0 h, X (external CLI), lane B. Needs: M3-06. From: D3d s9 BT-A6 to BT-A17, D3e part 3 s7 A01 to A32, D3b A20 (prep).**

- **Scope.** Record with the HX tool the D3d scenarios BT-A6 to BT-A17, the D3e part 3 scenarios A01 to A32 (walls, doors, lamps, storm, foot, barrels, swamp, Tyrog intro, lift, fences, lab, vents, flames, pipes, sewer doors, end scripts, cave, meat barrel, fire glow, tile clock) and D3b A20. Prepare only.
- **Original.** none.
- **Data to extract.** none.
- **Files.** re/harness/scripts/, re/harness/pokes/.
- **Read.** the sections named, M3-06.
- **Check.** Each scenario has a MAME trace; batches of one MAME process.

#### M8-01 Brontosaurus foot (classes 22 and 23)

**1.5 h, C (Claude worker), lane A. Needs: M5-01, M3-10, M8-00. From: P6, DS12, W306, S11 (foot sound).**

- **Scope.** Class 22 (the foot, cannot be hit, +7D 0xFF, attack 0x63 resting and 0x4C slam 30 base damage with knockdown) and class 23 (the leg helper): states 2, 4, 6 (hover 180 frames tracking the nearest hero in an 80 px home column), 8, 0xA, 0xC (slam, sound 0xA288, `G.$755C = 4` shake), 0xE, the draw by height, the shadow oval, it hurts enemies (raw 30). Two records in Episode 6 area 1 (triggers 0x520 and 0x7B0).
- **Original.** 0x47C42 (1,572 bytes), 0x48266 (294 bytes), anim table 0x718EE, tables 0x47EAC, 0x47EBC, 0x48048.
- **Data to extract.** animation table, box table.
- **Files.** game/enemy/class22_foot.rs, class23.rs (new).
- **Read.** D3b s6, s13 (P6), A20; D3e part 3 s2.1, A09, W306.
- **Check.** D3b A20 (`$6` 2 at t = 3, 4 at 43, 6 at 99, 8 at 279, 0xA at 339, 0xC at 360, 0xE at 384, 2 at 414; y 30, 256, 144, 144, 144, 259, 30; Jack -30 at t = 380; shake 4 at the landing); D3e A09 shadow oval.

#### M8-02 Episode 6 area 0 and 1: door bursts B4, mine lamps B60, rain B55 and B56

**2.0 h, X (external CLI), lane A. Needs: M3-16, M5-04. From: B4, B60, B55, B56, FX-P7 (class 2).**

- **Scope.** B4 vars 4, 8, 0xC (door burst, enemy entrance 0x50 frames later, class 21 var 0x16), B60 (lamp palettes 80 and 81, 36 frame cycle, stage 5 area 0 only), B55 (75 rain streaks of 4 scroll 1 tiles, redrawn every frame until camera X 0x500), B56 (24 slots every 20 px, pool C class 2 drops with floor splash, density 2/16, 4/16, 8/16, sound 0x8D every 241 frames) and pool C class 2 (falling cave dust, drops).
- **Original.** 0x767D8, 0x863AC, 0x8574C, 0x85916, 0x92808.
- **Data to extract.** block tables, 0x857D8, 0x8586E, 0x858D0, 0x85AE4, palette lists 0x86476, 0x86488.
- **Files.** game/prop/b4.rs, b60.rs, b55_56.rs (new), game/effect/fx02.rs.
- **Read.** D3e part 3 s2.2, s2.3, s2.9, A05, A06, A07; W302, W304, W305; D3f class 2.
- **Check.** D3e part 3 A05 (B4 var 4 seen at t = 102, enemy in the doorway at 186, var 8 at 268), A06 (palette 80 colour 0 F321 at t = 0, 4, 8, F661 at 12, 16 ... steps every 9 frames), A07 (scroll 1 changes every frame).

#### M8-03 Lightning B9, part 1: bolt strips and the object timers

**1.3 h, C (Claude worker), lane A. Needs: M8-02, M5-06. From: B9 (part 1).**

- **Scope.** Three B9 objects (far layer X 0xA0, 0x260, 0x340): the bolt of 4 tiles for 5 steps of 3 frames, restore, the pause table 0x77F38 [120, 240, 60, 120, 60, 420, 240, 120] with each object's own index.
- **Original.** 0x77E76, 0x77F74.
- **Data to extract.** bolt tile pictures, pause table.
- **Files.** game/prop/b9.rs (new).
- **Read.** D3e part 3 s2.3, A07, A08.
- **Check.** D3e A07: bolt tiles at 0x913A60 change at t = 114, 118, 120, 124 and return at 126, 366, 370, 372, 376, 378.

#### M8-04 Lightning B9, part 2: palette flashes, thunder

**1.2 h, C (Claude worker), lane A. Needs: M8-03, M3-16. From: B9 (part 2).**

- **Scope.** Sequence 1 (8 scroll 3 palettes 100 to 105, 107, 108, 6 steps of 3, 1, 2, 1, 2, 3 frames) and sequence 2 eight frames later (all 32 scroll 2 palettes 64 to 95), thunder sound 0x77 40 frames after the bolt (recorded, pan from the object X).
- **Original.** tables 0x77FF6, 0x78170, sound helper 0xA25A.
- **Data to extract.** palette sequences.
- **Files.** game/prop/b9.rs.
- **Read.** D3e part 3 s2.3, A07, A08.
- **Check.** D3e A07 (scroll 3 palette flashes start at t = 2, 126, 378, 450, 582, 654, scroll 2 flashes 8 frames later) and A08 pictures at t = 0 to 20 and 130 to 152 equal MAME.

#### M8-05 Episode 6 swamp scenery: eye B7 var 0, water B8 and B39

**1.8 h, X (external CLI), lane A. Needs: M3-16, M6-15. From: B7, B8, B39.**

- **Scope.** B7 var 0 (eye of the huge reptile, 2 by 2 tiles, 5 pictures every 24 frames, 96 frame pause, stops past camera X 0xB80), B8 (swamp water surface, 8 rows 42 to 49, 4 pictures every 24 frames, writes G.-$703C used by wading draw; shares code with B2) and B39 (5 rows 43 to 47, runs while camera Y < 0x170).
- **Original.** 0x76CD2, 0x76E0E, 0x830C6.
- **Data to extract.** block tables.
- **Files.** game/prop/b7.rs (var 0), b8.rs, b39.rs.
- **Read.** D3e part 3 s2.1, s3.2, s3.3, A11, A12, W308.
- **Check.** D3e A11 (water rows 42 to 49 change at t = 18, 42, 66, eye at t = 30, 54, 78, 102, then 198 to 294) and A12 (rows 43 to 47 at t = 6, 30, 54).

#### M8-06 Tyrog scene (B79, classes 75 and 76)

**2.0 h, C (Claude worker), lane A. Needs: M7-15, M5-03, M8-00. From: P-D3d-4 (part 1), BT08, W309.**

- **Scope.** Starter pool B 79 (three gangsters carry a metal box, three captions B66 0x50 to 0x52, the crate opens, Tyrog jumps out), class 75 actors (var 0 320 frames, 1, 4), class 76 (jump and puffs every 8 frames), sound 0x21, the class 4 var 0x200 creation at t = 648 or 904.
- **Original.** 0x8A3D0, 0x5E702, 0x5EC26.
- **Data to extract.** scene tables.
- **Files.** game/prop/b79.rs, game/enemy/class75.rs, class76.rs (new).
- **Read.** D3d s3.1, s3.2, BT08, BT-A8; D3e part 3 s3.3, A13.
- **Check.** D3d BT-A8: crate break at t = 416, class 76 puffs every 8 frames from 424, class 4 var 0x200 at t = 648 with HP 256; D3e A13 B79 states at 350, 470, 700, 820, Tyrog at 904, B22 at 910.

#### M8-07 Tyrog (class 4), part 1: forms, tables, actions

**2.0 h, C (Claude worker), lane A. Needs: M8-06. From: P-D3d-4 (part 2), BT09.**

- **Scope.** Class 4 init (three forms by the `+26` high byte, HP index 6 halved at every init, music only for var 2), tables 0x74542 to 0x74802, actions 2, 3, 4, 6, 7, the recover entry 0x36E7C with the wake up claw, first states `4C(2) 10(22) 3E(10) 18(4) 30(6) 2E(6) 04(22) 3E(10)`. The ROM checksum call at 1 in 16 form changes (0x5A454) is not ported.
- **Original.** 0x36A64 (class 4), 0x36CA0 (state table).
- **Data to extract.** tables as listed.
- **Files.** game/enemy/class4.rs, class4_tables.rs (new).
- **Read.** D3d s3.3, s3.4, BT09, BT12, BT-A9.
- **Check.** D3d BT-A9 state sequence `4C(2) 10(22) 3E(10) 18(4) 30(6) 2E(6) 04(22) 3E(10)` equal in ty-full-b and ty-full-c.

#### M8-08 Tyrog, part 2: attacks, grab, damage numbers

**2.0 h, C (Claude worker), lane A. Needs: M8-07, M5-12. From: P-D3d-4 (part 3).**

- **Scope.** The states by function (pounce 0x4D = 26 at state 0x34, fireball 22, chest blades 27 var 0 at state 0x24 and 15 to 18 var 1 at state 0x46), the semantic names of the states left open (open item 3 of D3d), grabs (grabbed at t = 34, thrown, 27 damage, up at t = 161 state 0x4C in all forms), whether a long chain locks a form.
- **Original.** class 4 states.
- **Data to extract.** none new.
- **Files.** game/enemy/class4.rs.
- **Read.** D3d s3.4, BT-A10, BT-A11, s9 (c) items 3 and 15.
- **Check.** D3d BT-A10 (pounce 26, fireball 22, chest blades 27 and 15 to 18) and BT-A11 grab timeline in all three forms.

#### M8-09 Tyrog, part 3: chest blades class 74, fire class 26 var 3 and 4

**2.0 h, C (Claude worker), lane A. Needs: M8-08, M7-18. From: P-D3d-4 (part 4), BT10.**

- **Scope.** Class 74 (chest blade child: attached, 20 HP, low box y -16 to +5, one hit tick) and class 26 var 3 (flat fireball) and var 4 (lava rock thrown on a cross of five spots).
- **Original.** 0x5E4F4, 0x4D59E.
- **Data to extract.** spot tables.
- **Files.** game/enemy/class74.rs, class26.rs.
- **Read.** D3d s3.5, BT10, BT-A10, s9 (c) items 7 and 11.
- **Check.** D3d BT-A10 `ty4-v2-dmg` numbers; the blades and rock timing in a pictured run.

#### M8-10 Tyrog host (class 72) and the class word chain

**2.0 h, C (Claude worker), lane A. Needs: M8-09. From: P-D3d-4 (part 5), BT11, BT12.**

- **Scope.** Class 72: 30 frame intro, punch, jump, caption 0x5A, transformation anim 0x22 or 0x23, rewrite back to class 4; 450 HP, poise 1 (killable for 100,000 and `0xACEE`); the chain 4 var 2, 72 var 1, 4 var 1, 72 var 0, 4 var 0; forms 2 and 1 give no score.
- **Original.** 0x5DD58 (class 72), 0x38DAC, 0x5E076.
- **Data to extract.** tables 0x5DB0E style (see D3d).
- **Files.** game/enemy/class72.rs.
- **Read.** D3d s3.6, s3.7, BT11, BT12, BT-A9, BT-A12.
- **Check.** D3d BT-A9 (class 4 dying at t = 1506, class 72 var 1 HP 450 at 1630, class 4 var 1 at 1878, dying again at 2706, class 72 var 0 at 3062, class 4 var 0 at 3436) and BT-A12 (host states 0 for 30, 4 for 20 to 22, 6 for 56 to 58, 8 for 140).

#### M8-11 Tyrog, part 4: death, chain end, acceptance

**2.0 h, C (Claude worker), lane A. Needs: M8-10. From: P-D3d-4 (part 6), P-D3d-10 (Tyrog share), S4.**

- **Scope.** The final kill chain (`G.sub` 2, 4, 6 at t = 4909, 5229, 7030, next mode at 7988, score 60,000), HP by player count (class 4 256 / 288 / 352), the Tyrog acceptance BT-A8 to BT-A12 with 1 and 3 players.
- **Original.** class 4 form 0 death 0x38CFE.
- **Data to extract.** none.
- **Files.** game/enemy/class4.rs, class72.rs.
- **Read.** D3d s3.8, BT-A9 to BT-A12; D2c M7.
- **Check.** D3d BT-A9 final chain numbers; Episode 6 boss from the crate to the tally with the real Tyrog.

#### M8-12 Tyrog 2 (class 91)

**2.0 h, C (Claude worker), lane A. Needs: M8-11. From: P-D3d-5, BT13.**

- **Scope.** Class 91 on the class 4 code with the table differences: fight 3 tables, 300 HP at every level, counted by the census, 1,000 points, no `0xACEE`, no music, started from a lab tank (M8-19 and M8-20).
- **Original.** 0x654E2.
- **Data to extract.** tables for class 91.
- **Files.** game/enemy/class91.rs.
- **Read.** D3d s3.9, BT13, BT-A10 (`ty91-dmg`).
- **Check.** D3d BT-A10 `ty91-dmg`; BT-A7 Tyrog 2 from cylinder 11 at t = 1085, x 1488 once M8-20 exists.

#### M8-13 Slisaurs, part 1: starter B33 and class 71 core

**2.0 h, C (Claude worker), lane A. Needs: M7-15, M7-03, M8-00. From: P-D3d-6 (part 1), BT01.**

- **Scope.** Starter pool B 33 (waits while `G.$4FC`, creates class 22, sound 0x11, two Slisaurs at X 0x580 Z 0x30 and 0x40, a third at Z 0x20 for 2 or 3 players, sums their HP, G.$502 every 300 frames, ends the fight on the frame the last HP reaches 0, `0xACEE` with the last body) and class 71 init and the hit handler 0x81A7A.
- **Original.** 0x818E4, 0x819A4, 0x81A7A, class 71 at 0x5C818 (5,440 bytes).
- **Data to extract.** tables 0x5DB0E, 0x5DB8E.
- **Files.** game/prop/b33.rs, game/enemy/class71.rs (new).
- **Read.** D3d s4.1 to s4.3, BT01, BT-A6.
- **Check.** D3d BT-A6: the starter makes 2 Slisaurs (3 with 2 or 3 players), HP 450 / 600 / 700.

#### M8-14 Slisaurs, part 2: gate, combo table, dash ramp, attacks

**2.0 h, C (Claude worker), lane A. Needs: M8-13. From: P-D3d-6 (part 2), BT02.**

- **Scope.** Gate table 0x5DB0E, combo table 0x5DB8E, dash ramp 0x5DD1C, attacks 0x36 (13) and 0x37 (21), hit reactions, lying time (read from the same table as Slice).
- **Original.** 0x5DD58 and following, tables listed.
- **Data to extract.** tables listed.
- **Files.** game/enemy/class71.rs.
- **Read.** D3d s4.4, s4.5, BT02, s9 (c) item 4.
- **Check.** D3d BT-A6: damage 13 (0x36) and 21 (0x37) (`sl-dmg-*`).

#### M8-15 Slisaurs, part 3: blades, afterimage, death flow, acceptance

**2.0 h, C (Claude worker), lane A. Needs: M8-14. From: P-D3d-6 (part 3), BT03, P-D3d-10 (Slisaur share), S4.**

- **Scope.** Reuse class 19 and class 27 from Slice (called from 0x5CEF4, 0x5D042, 0x5CC68, 0x5CF6A), the death flow (`G.sub` 2, 4, 6 at t = 2027, 2395, 4196, next mode at 5144, 70,000 each), the Episode 7 rank speed question (D4b open item 1: does 0xACEE run once or per body).
- **Original.** 0x4504E, 0x9639E, 0x5D19E.
- **Data to extract.** none.
- **Files.** game/enemy/class71.rs.
- **Read.** D3d s4.6, BT03, BT-A6; D4b s7 item 1.
- **Check.** D3d BT-A6 kill chain `G.sub` timeline; the Slisaur fight of Episode 7 area 3 with 1, 2, 3 players.

#### M8-16 Episode 7 lab: monitor B30 and caption B34

**1.5 h, C (Claude worker), lane A. Needs: M5-03, M3-16, M8-13. From: B30, B34, W313.**

- **Scope.** B30 (black screen, static from +60, the doctor's face with a three line caption box B34 at +180, sound 0x23 recorded, the fight starts after the talk: B33 reads G.$4FC). The static noise and the face are not decoded (open item 3 of D3e part 3): find the layer, tile or sprite source in the code.
- **Original.** 0x811DE, 0x81B3E.
- **Data to extract.** monitor pictures, caption text lines.
- **Files.** game/prop/b30.rs, b34.rs.
- **Read.** D3e part 3 s4.4, A18, open item 3.
- **Check.** D3e A18: B41 x 3 at t = 40, B33 at 350, B30 at (1200, 208) at t = 410, state 0202 at 470, 0204 at 590, B34 at 590; pictures t = 336 to 750 equal MAME.

#### M8-17 Episode 7 lab and library events: B61, B63, B41, B52, B54

**2.0 h, X (external CLI), lane A. Needs: M3-16, M3-19. From: B61, B63, B41, B52, B54.**

- **Scope.** B61 and B63 lab light palette cycles (stage 6 only, 17 frames, sound 0x6D every 121 frames), B41 console blink lights (periods 19, 7, 13), B52 bookshelf burst (wave 12: 55 codes and 70 attributes, class 21 var 0x20 books, a Ferris steps out) and B54 cage wall smash (once per stage, G.$7B6, class 21 var 0x1E, wave 4 of the lift).
- **Original.** 0x864DA, 0x869E6, 0x8383A, 0x85178, 0x855A8.
- **Data to extract.** palette lists, block tables.
- **Files.** game/prop/b61_63.rs, b41.rs, b52.rs, b54.rs.
- **Read.** D3e part 3 s2.1, s4.1 to s4.4, A15, A17, A18; W302, W310, W313.
- **Check.** D3e A15 (scroll 1 cols 5..12 and 35..42 rows 47..54 change once at t = 616), A17 (B52 at (1840, 0, 112) at t = 876, 55 code and 70 attribute words), A18 palettes 66 to 91 change every 17 frames.

#### M8-18 Episode 7 acceptance: lift, library, lab

**2.0 h, C (Claude worker), lane A. Needs: M8-15, M8-16, M8-17, M3-14. From: D4c A12, D3e part 3 A14 to A18, D4c open item 1.**

- **Scope.** Run D4c A12 and D3e part 3 A14 to A18 with natural play, the 3 player kill-all crash of open item 1 (find the class alive at the "ADDRESS ERROR"), the lift rider list, the fences, the library walk in, 1 and 3 players, fix.
- **Original.** Episode 7.
- **Data to extract.** none.
- **Files.** any Episode 7 file.
- **Read.** D3e part 3 s4, s7(c) item 1; D4c s12 item 1.
- **Check.** Scenarios equal or listed in `re/harness/allow/m8.txt`; `--list-placeholders` empty for stages 5 and 6 except Episode 8 classes.

#### M8-19 Lab tank breakers (class 44), part 1

**1.5 h, C (Claude worker), lane A. Needs: M5-04, M8-00. From: P-D3d-7 (part 1), BT04.**

- **Scope.** Pool B 44: four records (cylinders 01, 03, 04, 11), the 3 frame event (states 0, 4, 6 at t = 90, 91, 92), tile rewrite from 0x83D84, the glass shards (22 pool C objects in the first step), sound 0x37, recolour for the Hopper and Tyrog 2 tanks.
- **Original.** 0x83BB0, 0x83C36, 0x83D84.
- **Data to extract.** creature table 0x83C36, tile blocks.
- **Files.** game/prop/b44.rs (new).
- **Read.** D3d s5, BT04, BT-A7.
- **Check.** D3d BT-A7: tank breaker objects at t = 90 (state 0), 91 (4), 92 (6).

#### M8-20 Lab tank breakers, part 2: shard effect and the creatures

**1.5 h, C (Claude worker), lane A. Needs: M8-19, M7-08, M8-12. From: P-D3d-7 (part 2).**

- **Scope.** Shard effect (pool C class 21 variants 0x0E to 0x14 and 0x24), the creatures that come out: a Bludge at x 48 (lands at t = 94), a Rock Hopper at x 336 (t = 136), the second Bludge from cylinder 04 (t = 275) and Tyrog 2 from cylinder 11 (t = 1085, x 1488).
- **Original.** class 44, class 21 variants.
- **Data to extract.** shard variants.
- **Files.** game/prop/b44.rs, game/effect/fx21.rs.
- **Read.** D3d s5, BT-A7; D3f class 21.
- **Check.** D3d BT-A7 in full (`k8-walk`, `k8-fine`, `k8-start`, `k8-t11`).

#### M8-21 Episode 8 sewer scenery: water bands B28 and B29, torches B43

**1.8 h, X (external CLI), lane A. Needs: M3-16. From: B28, B29, B43.**

- **Scope.** B28 (water band rows 12 to 14, 4 pictures on the tile clock), B29 (tall block, 3 columns by 16 rows) and B43 (wall torch flame 2 by 2 tiles, 4 pictures every 8 frames after a wait of 15).
- **Original.** 0x7F72A, 0x8048A, 0x83ACA.
- **Data to extract.** block tables (0x80518 and others).
- **Files.** game/prop/b28_29.rs, b43.rs.
- **Read.** D3e part 3 s5.2, A30; W316, W317.
- **Check.** D3e A30 (G.$7AE values at gf 1284 to 1380) and pictures of the sewer water rows equal MAME.

#### M8-22 Episode 8 doors, pipes data and fire glow: B42, B40 vars, B53

**1.8 h, X (external CLI), lane A. Needs: M5-14, M3-16. From: B42, B40 (vars), B53.**

- **Scope.** B42 vars 0, 4, 0xC, 0x10 (red sewer door burst, class 21 var 0x26), the B40 data for the sewer pipes (vars 8, 0xC, 0x10 kind box table; var 0x10 needs a jump kick), B53 (27 palettes from G.$7B2 when a hero burns, only Episode 8 areas 2 and 3).
- **Original.** 0x83A0A, 0x836A4, 0x85306.
- **Data to extract.** block tables, 27 palette sets.
- **Files.** game/prop/b42.rs, b40.rs, b53.rs.
- **Read.** D3e part 3 s2.1, s2.6, s5.2, s5.3, A23, A24, A29; W315, W319.
- **Check.** D3e A23 (B40 vars 8 and 0xC HP 8 to 0 at t = 50 and 22, var 0x10 jump kick at 32), A24 (B42 at t = 150, doorway by 190), A29 (370 colours in 27 palettes change at t = 32 and 92).

#### M8-23 Flame vents B45, part 1: the vent cycle, hits, destruction

**1.3 h, C (Claude worker), lane A. Needs: M8-22, M5-06. From: B45 (part 1), W314.**

- **Scope.** Pool B 45 (HP 48, box depth [Z-16, Z+32], states idle 60 to 90, sparks 45, growth 40, full 40 to 70, shrink 40, random durations from tables 0x8413A, 0x8415A; a hit only when the hero stands at Z 824 or more; Jack 10, 10, 15, 15; destroyed socket and 120 frames of small flames).
- **Original.** 0x83E0C to 0x84198, tables 0x84122, 0x84182.
- **Data to extract.** tables listed.
- **Files.** game/prop/b45.rs (new).
- **Read.** D3e part 3 s2.4, A19, A20.
- **Check.** D3e A19 (B45 HP 48, 38, 28, 13, 0 at t = 36, 50, 64, 80 with a punch every 14 frames), A20 (no hit at Z 760 to 816, hit at 824).

#### M8-24 Flame vents, part 2: flame pieces A83, damage, burn

**1.2 h, C (Claude worker), lane A. Needs: M8-23, M5-07. From: B45 (part 2), HU, W314.**

- **Scope.** Pool A class 83 flame pieces (made by 0x6152C: start 18 px right of the vent and 73 up, velocity (-7.5, 0, -5), life = parameter + 1, attack 0x5D registered in the hostile list, 4 flame pictures), 20 damage at level 0 and 22 at level 10, burn knock down with G.$7B2 = 2, enemies take the raw 20 too.
- **Original.** 0x6155C, 0x6152C, table 0x61730.
- **Data to extract.** flame table.
- **Files.** game/enemy/class83.rs (new).
- **Read.** D3e part 3 s2.4, A21, A22.
- **Check.** D3e A21 (hero HP 100, 80, 60, 40 at t = 102, 222, 342; level 10: 100, 78, 56, 34; no damage at X 720 and 760), A22 (a Hammer takes 20 per pass).

#### M8-25 Episode 8 lair: B71, B66 var 7 and the meat barrel check

**1.0 h, C (Claude worker), lane A. Needs: M8-22, M3-27. From: B71, W320.**

- **Scope.** B71 end area palette animation (never spawned in the walks: read it with the boss kill replay once M8-36 exists, otherwise port from the code), B66 var 7 (blank string 240 frames at the lair), and the meat barrel at X 0xCB0 carried through the area switch and the walk to the lair (pool E rules, no new code expected).
- **Original.** 0x89746.
- **Data to extract.** palette lists.
- **Files.** game/prop/b71.rs.
- **Read.** D3e part 3 s5.3, s5.4, open item 2, A28.
- **Check.** D3e A28 (meat barrel held to t = 2600 at X 4747; thrown at 272 and broken, E4 var 1).

#### M8-26 Episode 8 acceptance, part 1: lab, sewers, cave

**2.0 h, C (Claude worker), lane A. Needs: M8-20, M8-21, M8-24, M8-25. From: D3e part 3 A19 to A29, D3d BT-A7.**

- **Scope.** Run D3e part 3 A19 to A29 (vents, flames, pipes, doors, lab end, sewer end, cave barrels, meat barrel, fire glow) and the tank scenarios BT-A7 with 1 and 3 players. Fix.
- **Original.** Episode 8 areas 0 to 2.
- **Data to extract.** none.
- **Files.** any Episode 8 file.
- **Read.** D3e part 3 s5, s7.
- **Check.** Scenarios equal or listed in `re/harness/allow/m8.txt`.

#### M8-27 Fessenden form 1 (class 94), part 1: init, intro, gate and action tables

**2.0 h, C (Claude worker), lane A. Needs: M7-21, M5-03, M8-00. From: P-D3d-8 (part 1), BT14.**

- **Scope.** Class 94 (a 90 percent copy of Morgue): human intro 276 frames (state 0x0E) then 0x08 and music 0x0F, gate 0x74A42, action table 0x74A82 (more headbutt dashes), no helpers, HP 450 at level 10, headbutt 22 and shoulder rush 26, 8,000 points, no `0xACEE`. Spawned by the script record at trigger 0x10A0 with dialogue box B50 var 4 (trigger 0x1120).
- **Original.** 0x67D2C (7,218 bytes), 0x6EA48 (animation table, the Morgue table), 0x67F94 (palette animation of sprite palette 0x1D).
- **Data to extract.** tables listed (reuse the class 40 tables where identical).
- **Files.** game/enemy/class94.rs (new).
- **Read.** D3d s6.1 to s6.3, BT14, BT-A13.
- **Check.** D3d BT-A13: class 94 state 0x0E for 276 frames then 0x08 and music 0x0F, headbutt 22, shoulder rush 26, HP 450 at level 10.

#### M8-28 Fessenden form 1, part 2: death scene, captions, growth, creation of class 84

**2.0 h, C (Claude worker), lane A. Needs: M8-27, M5-03. From: P-D3d-8 (part 2), BT15, FX (classes 39).**

- **Scope.** The 232 frame fall, five captions (0x3D, 0x3E, 0x40, 0x41, 0x42 and the lines 0x50 to 0x53 as needed, tall font via B66 and B50), the growth animations 0x15, 0x18, 0x1D with the aura (pool C class 39), 8,000 points at t = 1743, class 84 created 1,224 frames after HP 0 with full HP (creator 0x698E8). The class 94 palette animation (open item 14).
- **Original.** 0x6967E to 0x698E8, 0x69E66, 0x974FA (class 39 spawner), 0x97530 (class 39 handler).
- **Data to extract.** caption strings.
- **Files.** game/enemy/class94.rs, game/effect/fx39.rs.
- **Read.** D3d s6.4, BT15, BT-A14; D3f class 39.
- **Check.** D3d BT-A14 (HP 0 at t = 1268, dying at 1500, captions at 1500, 1743, 2091, 2191, 2341, 8,000 points at 1743, state 6 at 2491, class 84 with HP 640 at 2492).

#### M8-29 Fessenden form 1, part 3: acceptance

**1.0 h, C (Claude worker), lane A. Needs: M8-28. From: P-D3d-8 (part 3), P-D3d-10 (class 94 share).**

- **Scope.** Run BT-A13, BT-A14 and BT-A16 (grab of class 94: grabbed at t = 428, thrown 29) with 1 and 3 players, fix.
- **Original.** class 94.
- **Data to extract.** none.
- **Files.** game/enemy/class94.rs.
- **Read.** D3d BT-A13, BT-A14, BT-A16.
- **Check.** Scenarios equal or listed.

#### M8-30 Fessenden form 2 (class 84), part 1: body, five parts, sorted slots

**2.0 h, C (Claude worker), lane A. Needs: M8-29, M7-15. From: P-D3d-9 (part 1), BT16.**

- **Scope.** Class 84 main body (0x6173E, 8,250 bytes), five parts created from the sorted slots at t = 11 (class 85 at 0x63778, 86 at 0x63806, 87 at 0x63BC4, 88 and 89 at 0x63E30), the parts mirror the body's HP and hit id, the gate (16 frames by level), HP 640, fire creator 0x6194A (class 22 controller), the HUD gauges (rounding open item 8).
- **Original.** 0x6173E, 0x63778, 0x63806, 0x63BC4, 0x63E30, 0x6194A.
- **Data to extract.** animation tables, part tables.
- **Files.** game/enemy/class84.rs, class85_89.rs (new).
- **Read.** D3d s6.5, BT16, BT-A15.
- **Check.** D3d BT-A15: five parts created at t = 11; parts follow the body; arm strikes 26 (0x5F), 21 (0x60), 13 (0x61).

#### M8-31 Fessenden form 2, part 2: walk, chase, dash, step, the 480 frame timer

**2.0 h, C (Claude worker), lane A. Needs: M8-30. From: P-D3d-9 (part 2).**

- **Scope.** Actions 1, 2, 8, 9, 10 and the reset 0x61ADA (walk, chase, dash, step, 480 frame timer) with the walk rules at 0x61B3E and 0x61D26.
- **Original.** 0x61ADA, 0x61B3E, 0x61D26.
- **Data to extract.** none new.
- **Files.** game/enemy/class84.rs.
- **Read.** D3d s6.6, s9 (c) item 10, BT-A15.
- **Check.** D3d BT-A15 body action sequence (A8 = 12, 6, 22, 10, 8 in `f84-trace`).

#### M8-32 Fessenden form 2, part 3: the 13 actions, picker table

**2.0 h, C (Claude worker), lane A. Needs: M8-31. From: P-D3d-9 (part 3).**

- **Scope.** The remaining actions of the 13, the picker table 0x64E20 by 48 px classes, the 16 frame gate by level.
- **Original.** 0x64E20 and the action routines.
- **Data to extract.** picker tables.
- **Files.** game/enemy/class84.rs.
- **Read.** D3d s6.6, BT16.
- **Check.** D3d BT-A15 (`f84-trace`): action sequence equal for 600 frames.

#### M8-33 Fessenden form 2, part 4: counter reaction, guard counter, poise

**2.0 h, C (Claude worker), lane A. Needs: M8-32. From: P-D3d-9 (part 4), BT17, BT19.**

- **Scope.** Counter reaction against hero moves 0x14, 0x02, 0x13, 0x15 with HP fraction masks (12, 15, 22, 27 in 32: dodge or arm strike), poise 10 reset at each hit, guard counter `+BD` (2), only hits in phases 4, 6, 10, 12 cost a point, knock down at 0 (about 6 hits, 4th hit of a 10 frame jab chain at t = 143, 203, 453, 527), grab and throw rule (big parts `+63 = 4`, open item 9).
- **Original.** 0x6314E, 0x6326E, 0x6504A, 0x6505A, 0x63460, 0x6350C.
- **Data to extract.** masks.
- **Files.** game/enemy/class84.rs.
- **Read.** D3d s6.7, BT17, BT19, BT-A15, BT-A16.
- **Check.** D3d BT-A15 (`f84-rapid`) knock down at t = 143, 203, 453, 527; BT-A16 (class 84 grabbed t = 44, released t = 57, damage 5, 5).

#### M8-34 Fessenden form 2, part 5: heads, fire volleys class 95, jaw grab, slime

**2.0 h, C (Claude worker), lane A. Needs: M8-33, M5-06. From: P-D3d-9 (part 5), BT18, FX-P7 (class 40).**

- **Scope.** Bite 0x5E (15), fire volleys with class 95 variants 0 to 8 chosen by the target distance (0x63E30, 0x64210, 0x69C74; volleys every 134 to 430 frames), jaw grab (0x64 once, 0x65 N times by level 2 to 5, damage values open item 12), pool C class 40 slime and acid splash drops, class 95 fireball and flame creator 0x69932.
- **Original.** 0x6995E (class 95), 0x69932, 0x650DA, 0x97674.
- **Data to extract.** tables 0x9770A.
- **Files.** game/enemy/class95.rs, game/effect/fx40.rs.
- **Read.** D3d s6.8, BT18, s9 (c) item 12; D3f class 40.
- **Check.** D3d BT-A14 (fire volleys of 134 to 430 frames after t = 2492), BT-A15 bite 15; class 95 pictures.

#### M8-35 Fessenden form 2, part 6: death sequence, ending trigger, acceptance

**2.0 h, C (Claude worker), lane A. Needs: M8-34, M3-27. From: P-D3d-9 (part 6), P-D3d-10 (class 84 share), BT20, S4.**

- **Scope.** `0xACEE` at HP 0 (stage index 7 sets G.$4D7 and G.$4DB), 1,000 points at +368 frames, the dying steps at t = 2498, 2673, 3033, 3105, 3286, `+A6 = 1` and the hidden body, tally at +1,800, the 3,600 frame wait and mode 0x1C, B82 and the showers (M3-23), the real Fessenden kill replaces the placeholder in the autoplay.
- **Original.** 0x63538, 0xACEE.
- **Data to extract.** none.
- **Files.** game/enemy/class84.rs.
- **Read.** D3d s6.9, s7, BT20, BT-A17; D4c s12 item 3.
- **Check.** D3d BT-A17: `0xACEE` at HP 0 (t = 1527), `G.sub` 2 at 1530, step 0 at 1895 (1,000 points), `G.sub` 4 at 1900, steps 4, 6, 8, 0xA, 0xC at 2498, 2673, 3033, 3105, 3286, `G.sub` 6 at 3700, mode 0x1C at 7300, `G.sub` 8 at 12670, mode 0x18 at 12850.

#### M8-36 Episode 8 acceptance, part 2: lair, boss chain, ending and whole episodes 6 to 8

**2.0 h, C (Claude worker), lane A. Needs: M8-35, M8-26, M8-18, M8-11. From: D3e part 3 A31, A32, D3d BT-A6 to BT-A17.**

- **Scope.** Run the remaining D3e part 3 and D3d scenarios, the Episode 6, 7 and 8 whole runs from the area 0 start to the tally with 1 player, and the real ending after the real Fessenden death. S11 sound helpers (biker, barrel, boomerang, roar, foot) are checked in the sound request logs. Fix.
- **Original.** Episodes 6 to 8.
- **Data to extract.** none.
- **Files.** any file of the episodes.
- **Read.** D3e part 3 s7, D3d s9.
- **Check.** Scenarios equal or listed in `re/harness/allow/m8.txt`; `--list-placeholders` is empty for all stages; autoplay reaches the ending with no placeholder.

### M9: Small things pass

What is left in mechanics-seen.md after the episodes: the tag team move, unspawned classes, unmeasured open items of the design documents, and a row by row audit of all 588 rows against MAME. The audit creates a fix list that is worked off at the end of this milestone.

Hours: 25.0 (lane A 15.0, lane B 10.0), 14 tasks.

#### M9-01 Tag team move, part 1: trigger, counters, state 0x1A

**1.5 h, C (Claude worker), lane A. Needs: M8-36. From: P-D2c-3 (part 1), P11, P43, WM18.**

- **Scope.** The trigger (both heroes still and overlapped: less than 32 px in X, 0 to 15 in Z, no input, class 0, for 90 frames; counters per other player with the pause and reset rules of D2c 15.1), the initiator state 0x1A and the swing, three player pairing (first match wins).
- **Original.** 0x6728, 0x686E, 0x6922.
- **Data to extract.** none.
- **Files.** game/player.rs, game/player_tag.rs (new).
- **Read.** D2c s15, s20 (P-D2c-3), M4; D2a s9, A15.
- **Check.** D2c M4 scenarios `tie_*` from M2-30: tie up at +92 (90 frames of stillness), boundaries X -32 to 31, Z 0 to 15, input breaks reset the count.

#### M9-02 Tag team move, part 2: partner states, follow table, blink

**1.5 h, C (Claude worker), lane A. Needs: M9-01. From: P-D2c-3 (part 2), WM19.**

- **Scope.** The four partner states (0x204, 0x422, 0x424, 0x426) for the four initiators (Let's Go, Roll, Jump, Fly), follow table 0x69D4, `held_players_update` for a player partner, EC blink for both for 180 frames, sparkles, Jack's partner run attack id 0x1A only with a direction held, voices (web only: team move lines are not in the ROM, D4e open item 3).
- **Original.** 0x69D4, 0x6922.
- **Data to extract.** follow table.
- **Files.** game/player_tag.rs, player.rs.
- **Read.** D2c s15, D2a s9 (all), A15; D2c M4, M5.
- **Check.** D2a A15 (launch at +30 for Jack and Mess, +40 for the others, flight and run numbers, both blink 180 frames) and D2c M5 (`atk` 26 with `2R` held, the thug loses 18).

#### M9-03 Unit tests of the player rules and the weapon name renames

**1.0 h, X (external CLI), lane A. Needs: M8-36. From: P-D2b-5, P-D2c-10.**

- **Scope.** Unit tests: hero damage factor order (Jack 32, Mustapha 34, Hannah 35, Mess 30), mercy rule, `$C6` table, lie and dizzy tables row 0, extra life lists and the score cap; rename weapon types in comments and the SPEC table (rod, stick, butcher knife, club, torch, dynamite, grenade, stones; shell, pellet).
- **Original.** 0x2C88E, 0x1D17E, 0x6B98.
- **Data to extract.** none.
- **Files.** game/player_combat.rs (tests), item/weapon.rs, item/blast.rs (comments), re/specs/player/SPEC.md.
- **Read.** D2b s5, s9, s11 item list; D2c s20 (P-D2c-10).
- **Check.** cargo test with the new tests; the measured numbers of D2b s5.2 in the tests.

#### M9-04 Class audit: unspawned pool B classes and the debug stub

**2.0 h, C (Claude worker), lane A. Needs: M8-36. From: B69, B16, B46, B51, B65, W34.**

- **Scope.** B69 (debug cursor: frees itself unless G.$4AA is set, a stub is enough) and a search of the ROM and of natural play for any spawner of B16, B46, B51, B65 (no spawner found so far, D3e part 1 open item 6). Implement the ones that are reachable (about 3 hours of work in the index if all are real). Record the result per class in errata.
- **Original.** 0x88A84 (B69), the four handlers.
- **Data to extract.** none.
- **Files.** game/prop/ (as needed), plan/game/design/D5-errata.md.
- **Read.** D3e part 1 s2, s9 item 6, s10 W34.
- **Check.** A list with one line per class: spawner found (and ported with a MAME check) or "dead code" with the search that proves it.

#### M9-05 Effect leftovers: class 9 banner, class 0, class 12 and 13 markers, class 35

**1.0 h, C (Claude worker), lane A. Needs: M8-36. From: FX20, D3f open items 1 and 2, FX30.**

- **Scope.** Decide with a MAME probe whether "STAGE CLEARED!" (class 9, digits 1 to 8, 180 frame hold, theme 0x0D) is shown anywhere in a normal play (the tally of D4d did not show it); port it if shown. Classes 0, 12, 13, 35 (title transition, unused in play) and 38 (dead code): ports only if a spawner is reached; class 38 stays an empty handler.
- **Original.** 0x93184, 0x93152, 0x923E0, 0x939D6, 0x93A7A, 0x96E56.
- **Data to extract.** tables 0x93262.
- **Files.** game/effect/ (as needed), errata.
- **Read.** D3f s2 (classes 0, 9, 12, 13, 35, 38), s8, FX20, FX30.
- **Check.** The probe result in errata; if class 9 is shown, its picture at the tally equals MAME.

#### M9-06 Open item probe sweep A: player, weapons, items, front end

**2.0 h, C (Claude worker), lane B. Needs: M8-00. From: D2a s19, D2c s21, D2b s11, D4a s13, D4d s15.**

- **Scope.** Run the probe tests the design documents asked for and did not run: D2a 1 (3 player friendly fire, dive scan), 2 and 4 (wall jump places, tag with a weapon); D2c 1 (stones from debris), 2, 3, 4, 5, 6, 15 (tire stack lift), 16 (Hogg shell and jab bonuses); D2b 4 (car respawn), 6 (dizzy); D4a 1 (enemy attack frame against a container), 2, 3 (3 player drum), 5 (stone size 2), 9, 11; D4d 8, 10, 14. Write the results as rows in plan/game/design/M9-sweep-a.md, with a fix item for every difference to the port.
- **Original.** none.
- **Data to extract.** none.
- **Files.** plan/game/design/M9-sweep-a.md (new), re/harness/ (scripts only).
- **Read.** the open item lists named above, capture-recipes.
- **Check.** Each open item has a result line (measured value or "not reachable, reason") and the fix list for M9-12.

#### M9-07 Open item probe sweep B: enemies, dinosaurs, bosses, camera, sound

**2.0 h, C (Claude worker), lane B. Needs: M8-00. From: D3a s16, D3b s14, D3c s10, D3d s9 (c), D4b s7, D4c s12, D4e s10.**

- **Scope.** Same for enemies and the rest: D3a 2, 3, 4, 6, 9, 13, 14; D3b 1 (other wake sources), 2 (fire), 3, 4, 5 (KYOURAN, DIFFENCE), 8, 9, 12; D3c 1, 2, 3, 4, 5, 6, 9; D3d 2 (can the host be killed), 4, 6, 9, 12, 15; D4b 1, 2, 4, 5; D4c 1 to 11; D4e 2 to 16. Results in plan/game/design/M9-sweep-b.md with a fix item per difference.
- **Original.** none.
- **Data to extract.** none.
- **Files.** plan/game/design/M9-sweep-b.md (new), re/harness/ (scripts only).
- **Read.** the open item lists named above.
- **Check.** Each open item has a result line and the fix list for M9-11 to M9-14.

#### M9-08 Mechanics rows audit A: groups A to E (player, enemies, dinosaurs, bosses, items)

**2.0 h, C (Claude worker), lane B. Needs: M8-36. From: README, mechanics-seen.md.**

- **Scope.** Read every row of groups A to E and of the "From design" blocks for D2a, D2b, D2c, D3a, D3b, D3c, D3d, D4a, D3e. For each row write: the task that covered it (coverage table of this plan), the evidence that the port equals MAME (scenario name and result) or a reason. Mark every row that the port does not yet match in a fix list.
- **Original.** none.
- **Data to extract.** none.
- **Files.** plan/game/design/M9-audit-a.md (new).
- **Read.** mechanics-seen.md, this plan s8.
- **Check.** Every row of the groups has a verdict line; the fix list is sorted by size.

#### M9-09 Mechanics rows audit B: groups F to I (flow, sound, effects, camera, design rows)

**2.0 h, C (Claude worker), lane B. Needs: M8-36, M4-19. From: README, mechanics-seen.md.**

- **Scope.** Same for groups F, G, H and the "From design" blocks of D4b, D4c, D4d, D4e, D3f, D3e parts 2 and 3.
- **Original.** none.
- **Data to extract.** none.
- **Files.** plan/game/design/M9-audit-b.md (new).
- **Read.** mechanics-seen.md, this plan s8.
- **Check.** Every row has a verdict; fix list.

#### M9-10 Mechanics rows audit C: groups J to N (user notes and web leads)

**2.0 h, C (Claude worker), lane B. Needs: M8-36. From: mechanics-seen.md J to N.**

- **Scope.** Same for the web and user lead groups J, K, L, M, N: for every lead the answer found by the design documents, and a MAME check where the lead is about a visible behaviour the port should show (for example: Episode 8 four fat enemies from the four corners, running down the cave slope removes the pterodactyls, damage rage state at low HP M23, thrown empty gun dazes M24).
- **Original.** none.
- **Data to extract.** none.
- **Files.** plan/game/design/M9-audit-c.md (new).
- **Read.** mechanics-seen.md J to N, D3a s17, D3c s7, D3d s8.
- **Check.** Every lead has a verdict; fix list.

#### M9-11 Audit fixes, batch 1

**2.0 h, C (Claude worker), lane A. Needs: M9-06, M9-07, M9-08, M9-09, M9-10, M9-02. From: M9 audit.**

- **Scope.** Work the fix list in order of size and visibility (what the player sees first).
- **Original.** per fix.
- **Data to extract.** per fix.
- **Files.** per fix.
- **Read.** the audit and sweep files.
- **Check.** Each fix has its scenario or probe result equal MAME; build, tests, regress.sh green.

#### M9-12 Audit fixes, batch 2

**2.0 h, C (Claude worker), lane A. Needs: M9-11. From: M9 audit.**

- **Scope.** As batch 1.
- **Original.** per fix.
- **Data to extract.** per fix.
- **Files.** per fix.
- **Read.** as batch 1.
- **Check.** As batch 1.

#### M9-13 Audit fixes, batch 3

**2.0 h, C (Claude worker), lane A. Needs: M9-12. From: M9 audit.**

- **Scope.** As batch 1.
- **Original.** per fix.
- **Data to extract.** per fix.
- **Files.** per fix.
- **Read.** as batch 1.
- **Check.** As batch 1.

#### M9-14 Audit fixes, batch 4 and the closing of the registry

**2.0 h, C (Claude worker), lane A. Needs: M9-13, M9-03, M9-04, M9-05. From: M9 audit.**

- **Scope.** As batch 1, then remove `game/placeholder.rs` and `--list-placeholders` if the registry is empty.
- **Original.** per fix.
- **Data to extract.** per fix.
- **Files.** per fix, game/placeholder.rs (delete).
- **Read.** as batch 1.
- **Check.** As batch 1; autoplay from power on to the ending passes with the registry code removed.

### M10: Exactness and finish

Whole-episode runs against MAME with 1 and 3 players and all heroes, the demos with real content, the attract loop for 2 full loops, settings, sound, pacing and the player docs. Class tasks already compared their own scenarios; here the whole runs are compared and drift is chased to its first frame.

Hours: 57.5 (lane A 53.5, lane B 4.0), 26 tasks.

#### M10-01 Prepare whole-episode recordings (all 8 episodes, 1 player)

**2.0 h, X (external CLI), lane B. Needs: M5-17. From: phase-13 task 1.**

- **Scope.** Scripted full runs of Episodes 2 to 8 for Jack (Episode 1 exists from M5-17): a script that walks, fights with chains and throws, takes the radio in Episode 3, and survives to the tally; recorded in MAME with the power-on or `control` sync. One MAME process per batch.
- **Original.** none.
- **Data to extract.** none.
- **Files.** re/harness/scripts/ep2_*.txt to ep8_*.txt.
- **Read.** re/harness/README.md, capture-recipes s2.
- **Check.** Each recording reaches the tally of its episode (`G.$4D1` steps) and has a MAME trace.

#### M10-02 Prepare whole-episode recordings for 3 players and the other heroes

**2.0 h, X (external CLI), lane B. Needs: M10-01. From: phase-13 task 1.**

- **Scope.** The same runs with 3 players (boot_3p) for Episodes 1 to 8 and one run each for Mustapha, Hannah and Mess on Episodes 1, 3, 5, 8.
- **Original.** none.
- **Data to extract.** none.
- **Files.** re/harness/scripts/.
- **Read.** as M10-01.
- **Check.** Traces exist and reach the tallies.

#### M10-03 Demos with real content: rows 5 and 0 (Episode 1 area 1, Episode 2 area 0)

**2.0 h, C (Claude worker), lane A. Needs: M6-32, M5-23, M1-13. From: D4d A7, T1.5.**

- **Scope.** Compare 1200 frames of each demo against MAME from the demo start (RNG reset, streams): player objects, rank, RNG, sprite list at frames 60, 300, 600, 900, 1200 and the fade at 1205. Chase the first differing frame and fix.
- **Original.** demo path of M1-12 and the level code.
- **Data to extract.** none.
- **Files.** any.
- **Read.** D4d s3.6, s13 A7.
- **Check.** Zero unexplained differences, or listed with cause in `re/harness/allow/m10.txt`.

#### M10-04 Demos with real content: rows 1 and 2 (Episode 3 area 0, Episode 8 area 1)

**2.0 h, C (Claude worker), lane A. Needs: M10-03, M8-36. From: D4d A7.**

- **Scope.** Same for rows 1 and 2.
- **Original.** as above.
- **Data to extract.** none.
- **Files.** any.
- **Read.** as above.
- **Check.** As M10-03.

#### M10-05 Demos with real content: rows 3 and 4 (Episode 5 area 0, Episode 6 area 1)

**2.0 h, C (Claude worker), lane A. Needs: M10-04. From: D4d A7.**

- **Scope.** Same for rows 3 and 4. After this the whole attract loop is real.
- **Original.** as above.
- **Data to extract.** none.
- **Files.** any.
- **Read.** as above.
- **Check.** As M10-03.

#### M10-06 Attract loop from power on, two full loops, with sound

**2.0 h, C (Claude worker), lane A. Needs: M10-05, M4-19. From: D4d A3, D4e s9.**

- **Scope.** Run the loop twice from power on (about 27000 frames) against MAME: states, pictures, sound requests (filtered), demo RNG. Close the allow lists of M1.
- **Original.** all of M1.
- **Data to extract.** none.
- **Files.** re/harness/allow/m1.txt, any.
- **Read.** D4d s13, D4e s9.
- **Check.** Frame equality of the whole loop except the documented differences.

#### M10-07 Episode 1 whole run, 1 player, final

**2.5 h, C (Claude worker), lane A. Needs: M5-23. From: phase-13 task 1.**

- **Scope.** Re-run the Episode 1 recording in `run` mode with everything ported (sound requests included), find the first difference, fix, repeat.
- **Original.** Episode 1.
- **Data to extract.** none.
- **Files.** any.
- **Read.** the class docs of the first difference.
- **Check.** `run` mode: zero differences or each remaining one listed with its cause in `re/harness/allow/m10.txt`.

#### M10-08 Episode 2 whole run, 1 player

**2.5 h, C (Claude worker), lane A. Needs: M10-07, M10-01, M6-32. From: phase-13 task 1.**

- **Scope.** As M10-07 for Episode 2.
- **Original.** Episode 2.
- **Data to extract.** none.
- **Files.** any.
- **Read.** as above.
- **Check.** As M10-07.

#### M10-09 Episode 3 whole run, 1 player (foot and car)

**2.5 h, C (Claude worker), lane A. Needs: M10-08. From: phase-13 task 1.**

- **Scope.** As M10-07 for Episode 3, including the radio, the drive and Hogg.
- **Original.** Episode 3.
- **Data to extract.** none.
- **Files.** any.
- **Read.** as above.
- **Check.** As M10-07.

#### M10-10 Episode 4 whole run, 1 player

**2.5 h, C (Claude worker), lane A. Needs: M10-09, M7-06. From: phase-13 task 1.**

- **Scope.** As M10-07 for Episode 4.
- **Original.** Episode 4.
- **Data to extract.** none.
- **Files.** any.
- **Read.** as above.
- **Check.** As M10-07.

#### M10-11 Episode 5 whole run, 1 player

**2.5 h, C (Claude worker), lane A. Needs: M10-10, M7-27. From: phase-13 task 1.**

- **Scope.** As M10-07 for Episode 5.
- **Original.** Episode 5.
- **Data to extract.** none.
- **Files.** any.
- **Read.** as above.
- **Check.** As M10-07.

#### M10-12 Episode 6 whole run, 1 player

**2.5 h, C (Claude worker), lane A. Needs: M10-11, M8-36. From: phase-13 task 1.**

- **Scope.** As M10-07 for Episode 6.
- **Original.** Episode 6.
- **Data to extract.** none.
- **Files.** any.
- **Read.** as above.
- **Check.** As M10-07.

#### M10-13 Episode 7 whole run, 1 player

**2.5 h, C (Claude worker), lane A. Needs: M10-12. From: phase-13 task 1.**

- **Scope.** As M10-07 for Episode 7 (lift, library, lab, Slisaurs).
- **Original.** Episode 7.
- **Data to extract.** none.
- **Files.** any.
- **Read.** as above.
- **Check.** As M10-07.

#### M10-14 Episode 8 whole run, 1 player, and the ending

**2.5 h, C (Claude worker), lane A. Needs: M10-13. From: phase-13 task 1.**

- **Scope.** As M10-07 for Episode 8 up to the ending and the return to the loop (the ending is independent of hero and player count).
- **Original.** Episode 8 and the ending.
- **Data to extract.** none.
- **Files.** any.
- **Read.** as above.
- **Check.** As M10-07; D4d A20 and A21 frame numbers.

#### M10-15 3 player runs, Episodes 1 and 2

**2.5 h, C (Claude worker), lane A. Needs: M10-14, M10-02. From: phase-13 task 2.**

- **Scope.** Run the 3 player recordings of Episodes 1 and 2 (boss HP by count, wave thresholds, spawn flags for 2P+, friendly fire, join) and fix.
- **Original.** 3 player rules.
- **Data to extract.** none.
- **Files.** any.
- **Read.** D2c s16, s19.
- **Check.** As M10-07 for 3 players.

#### M10-16 3 player runs, Episodes 3 and 4

**2.5 h, C (Claude worker), lane A. Needs: M10-15. From: phase-13 task 2.**

- **Scope.** As M10-15 for Episodes 3 and 4 (car seats for three players, Hogg 500 HP).
- **Original.** 3 player rules.
- **Data to extract.** none.
- **Files.** any.
- **Read.** D3c s5.7, D2c s16.
- **Check.** As M10-07 for 3 players.

#### M10-17 3 player runs, Episodes 5 and 6

**2.5 h, C (Claude worker), lane A. Needs: M10-16. From: phase-13 task 2.**

- **Scope.** As M10-15 for Episodes 5 and 6.
- **Original.** 3 player rules.
- **Data to extract.** none.
- **Files.** any.
- **Read.** D2c s16, D3d s2, s3.
- **Check.** As M10-07 for 3 players.

#### M10-18 3 player runs, Episodes 7 and 8

**2.5 h, C (Claude worker), lane A. Needs: M10-17. From: phase-13 task 2.**

- **Scope.** As M10-15 for Episodes 7 and 8 (3 Slisaurs, lift with three riders, Fessenden).
- **Original.** 3 player rules.
- **Data to extract.** none.
- **Files.** any.
- **Read.** D2c s16, D3d s4, s6.
- **Check.** As M10-07 for 3 players.

#### M10-19 Other heroes: Mustapha, Hannah, Mess whole runs

**2.5 h, C (Claude worker), lane A. Needs: M10-18. From: phase-13 task 2.**

- **Scope.** Run the Mustapha, Hannah and Mess recordings (Episodes 1, 3, 5, 8) and fix hero specific differences (hold reversal, air special, wall jump, knife stab, bazooka recoil).
- **Original.** hero code.
- **Data to extract.** none.
- **Files.** player*.rs.
- **Read.** D2a s13.
- **Check.** As M10-07 for each hero.

#### M10-20 Join, continue and game over with real content, ending name entry

**2.0 h, C (Claude worker), lane A. Needs: M10-14. From: phase-13 task 3.**

- **Scope.** Run the D4d A14 to A17 and A21 scenarios in Episodes 1, 3 and 7 (mid fight join, continue strip over the level, restore after the continue, name entry after game over and after the ending) and fix.
- **Original.** join, continue, name entry.
- **Data to extract.** none.
- **Files.** flow/*.rs.
- **Read.** D4d s6, s7, s13.
- **Check.** Scenarios equal MAME or listed.

#### M10-21 Settings variants

**2.0 h, C (Claude worker), lane A. Needs: M10-14, M2-01. From: phase-13 task 4.**

- **Scope.** Compare runs with difficulty 0 and 7, timer level 0 and 3, bonus life options 1 and 3, start lives 0 and 3, continue off, coin modes and free play against MAME runs made with the same EEPROM settings (poke the cooked bytes G.$777A to G.$777C at boot). Fix.
- **Original.** settings code.
- **Data to extract.** none.
- **Files.** config.rs, rank.rs.
- **Read.** D4b s4, D4d s4.
- **Check.** Rank start, time, lives, extra life thresholds and credit rules equal MAME for each variant.

#### M10-22 Sound exactness over whole runs, part 1: request streams

**2.0 h, C (Claude worker), lane A. Needs: M10-14, M4-19. From: phase-13, D4e s9.**

- **Scope.** Compare the sound request streams (code, d1, d2) of every whole run recording with MAME, close the gaps (calls the port lacks, toggle bytes, the 24 codes of D4e s7 only if a caller is found).
- **Original.** sound hooks.
- **Data to extract.** none.
- **Files.** snd.rs and callers.
- **Read.** D4e s5, s7, s10.
- **Check.** Request streams equal for all runs or listed in `re/harness/allow/m10.txt`.

#### M10-23 Sound exactness, part 2: register traces for every code used

**2.0 h, C (Claude worker), lane A. Needs: M10-22, M4-11. From: phase-13, M4.**

- **Scope.** Run the QSound register comparer over every code that appears in the whole runs and the front end, fix the driver, listen to the full game once.
- **Original.** dino-sound.
- **Data to extract.** none.
- **Files.** crates/dino-sound/.
- **Read.** M4 tasks, SPEC.
- **Check.** Register traces equal MAME for every code in use or documented.

#### M10-24 Frame pacing, window, input, audio latency, performance

**1.5 h, C (Claude worker), lane A. Needs: M10-14. From: N14, phase-13.**

- **Scope.** Pace the game loop at the original frame rate (about 59.64 Hz instead of 60), keep a fixed logic step per frame, check the load with the busiest scenes (Episode 8 cave, Episode 3 car) on a typical Mac, window scaling and full screen, gamepad and keyboard mapping for 3 players, audio buffer size. Known difference kept: the original slows down when many objects are on screen (the game frame takes 2 vblanks); the port keeps one logic frame per tick (see risks).
- **Original.** none.
- **Data to extract.** none.
- **Files.** main.rs, input.rs, audio.rs (main session files, allowed for this task).
- **Read.** D4e s0, N14 row.
- **Check.** Frame time under 12 ms in the busiest scene; no audio underruns in a 10 minute run.

#### M10-25 Remove debug paths from the default build, player README, closeout

**1.5 h, X (external CLI), lane A. Needs: M10-24, M9-14. From: phase-13 task 5.**

- **Scope.** Keep the debug keys, autoplay and probe hooks behind flags, write a README for players (how to build, where to put dino.zip, keys, config file, what is not included: test menu, region sets, debug mode), update STATUS.md and plan/PLAN.md to "done", list known differences.
- **Original.** none.
- **Data to extract.** none.
- **Files.** README.md, STATUS.md, plan/PLAN.md, main.rs.
- **Read.** STATUS.md, ARCHITECTURE.md, this plan s9.
- **Check.** A fresh clone with dino.zip builds with `cargo build --release` and starts in the attract loop; README commands work.

#### M10-26 Final regression and the release checklist

**2.0 h, C (Claude worker), lane A. Needs: M10-23, M10-25, M10-21, M10-20, M10-19. From: phase-13 task 6.**

- **Scope.** Run every scenario and whole run once more (`regress.sh` with all recorded scenarios, autoplay, the M1 loop), check that no ROM data or derived data is in git (`.gitignore`, `git ls-files` scan), run clippy and tests, list every difference left with its cause in `re/harness/allow/final.txt`. The main session then plays the whole game from power on to the ending and the user gets the build.
- **Original.** none.
- **Data to extract.** none.
- **Files.** re/harness/allow/final.txt, any.
- **Read.** this plan s3, s9.
- **Check.** All scenarios green or listed; `cargo build --release` and `cargo test` green; the list of known differences is in the README.

## 7. Coverage of `mechanics-seen.md`

Every one of the 588 rows (groups A to N and the "From design" blocks) is mapped. **No row is without a task.** 15 row entries are out of scope on purpose, each with its reason (list below). Rows that are DONE in the port are still mapped to the task that checks them against MAME, so the audit tasks M9-08 to M9-10 can confirm every row once.

| Group (as in the file) | Rows | Out of scope rows | Tasks that cover them |
| --- | --- | --- | --- |
| A. Player | 42 | 0 | M2-01, M2-03, M2-04, M2-08, M2-09, M2-12, M2-14 to M2-21, M3-30, M4-17, M5-02, M5-05, M5-07, M5-18 to M5-21, M6-14, M6-21, M9-01 to M9-03, M10-19 |
| B. Enemies | 26 | 1 | M3-02, M3-03, M3-14, M3-20, M3-30, M5-13, M5-22, M6-01 to M6-03, M6-07, M6-08, M6-11, M6-27, M6-29, M6-30, M7-07 to M7-11, M7-16, M8-01 |
| C. Dinosaurs | 14 | 1 | M5-10, M6-02 to M6-06, M6-16, M6-17, M7-10 to M7-12, M8-01, M9-07 |
| D. Bosses | 12 | 0 | M3-03, M3-04, M3-29, M5-03, M5-09, M5-11 to M5-13, M6-09 to M6-12, M6-18 to M6-26, M7-01 to M7-04, M7-16 to M7-21, M8-06 to M8-15, M8-27 to M8-35 |
| E. Items, containers, hazards, special scenes | 26 | 0 | M2-15, M3-01, M3-08 to M3-14, M3-16, M3-19, M5-06, M5-07, M5-14, M5-21, M6-15 to M6-22, M6-29, M6-30, M7-05, M7-22 to M7-25, M8-02, M8-05, M8-18 to M8-21, M8-23, M8-24, M9-04, M9-08, M9-10 |
| F. Flow, time, HUD | 24 | 2 | M1-03 to M1-15, M2-01, M2-02, M2-08, M2-12 to M2-15, M2-18, M2-20 to M2-26, M2-28, M2-29, M3-15, M3-20, M3-23 to M3-27, M3-30, M4-14, M10-15 |
| G. Sound | 5 | 0 | M2-13, M2-15, M4-10, M4-12 to M4-18 |
| H. Effects, camera, bounds | 8 | 0 | M2-13, M2-17, M2-29, M3-15, M3-29, M5-04 to M5-08, M5-22 |
| Design rows from D2a (player moves, file group I) | 14 | 0 | M2-09, M2-17, M2-24, M5-02, M5-18, M5-20, M9-01 to M9-03 |
| Design rows from D3e part 1 (pool B, file group I) | 12 | 0 | M2-04, M2-05, M2-10, M2-16, M3-14 to M3-19, M3-23, M5-14, M5-21, M6-03, M6-16, M6-17, M6-31, M8-23, M9-04, M9-05 |
| Design rows from D4d (front end, file group I) | 21 | 2 | M1-03, M1-04, M1-12, M1-13, M2-02 to M2-04, M2-18 to M2-26, M2-28, M3-23 to M3-27, M4-14 |
| Design rows from D4c (camera and flow, file group I) | 30 | 3 | M2-07, M2-08, M2-11, M2-12, M2-15, M2-27, M3-03, M3-08 to M3-13, M3-15, M3-20, M3-30, M6-18 to M6-22 |
| Design rows from D3b (dinosaurs, file group I) | 14 | 1 | M5-01, M5-10, M6-03, M6-04, M6-06, M7-10 to M7-12, M8-01 |
| Design rows from D2b (player state rules, file group I) | 18 | 0 | M2-01, M2-14, M2-16, M2-17, M2-19, M2-23, M5-19, M9-03 |
| Design rows from D2c (weapons and multiplayer, file group I) | 27 | 0 | M2-29, M5-02, M5-07, M5-18, M5-20, M6-27, M6-28, M9-01, M9-02, M10-15, M10-18 |
| J. Leads from a web summary (GPT) | 19 | 0 | M2-20, M3-17 to M3-20, M3-23 to M3-27, M5-09, M5-10, M5-13, M5-18, M5-21, M6-09 to M6-11, M6-18 to M6-22, M6-29, M6-30, M7-01 to M7-09, M7-16, M8-01, M8-02, M8-06 to M8-11, M8-16, M8-17, M8-19, M8-20, M8-27 to M8-35 |
| K. Notes from the user | 2 | 0 | M5-20, M6-27, M7-09 |
| Design rows from D3a (humans, after group K) | 26 | 0 | M2-05, M3-01, M3-20, M5-01, M5-02, M5-13, M6-01, M6-02, M6-06 to M6-08, M6-13, M6-27 to M6-30, M7-07 to M7-09, M7-16, M8-01, M8-17 |
| L. Leads from the user's Recreation Blueprint | 16 | 1 | M2-20, M2-25, M2-26, M3-19, M3-20, M5-09 to M5-13, M5-20, M5-21, M6-11, M6-17, M6-18, M6-21, M6-23 to M6-26, M6-31, M7-09, M7-17, M7-26, M8-11, M8-18 to M8-25 |
| M. Leads from the Deep-Dive Supplement | 28 | 1 | M1-08, M1-09, M2-20, M2-23, M3-04, M3-26, M5-09 to M5-13, M5-18, M5-20, M5-21, M6-02, M6-03, M6-11, M6-12, M6-16, M6-24, M6-26, M6-27, M6-31, M7-03 to M7-05, M7-09, M7-17 to M7-21, M8-08 to M8-11, M8-15, M8-33, M8-35, M9-02, M9-07, M9-10 |
| Design rows from D3c (bosses 1, after group M) | 27 | 0 | M3-02 to M3-04, M5-09, M5-11, M5-12, M6-09 to M6-11, M6-18 to M6-21, M6-23 to M6-26, M7-02, M7-03 |
| N. Leads from the Deep-Dive Supplement 2 | 14 | 1 | M2-01, M2-29, M3-03, M4-16, M5-18, M6-19, M6-29, M7-06, M7-26, M8-18, M8-26, M9-10, M10-21, M10-24 |
| Design rows from D4b (time, rank, score, after group N) | 15 | 0 | M2-01, M2-13, M2-14, M2-17, M2-19, M2-20, M2-22 to M2-24, M5-21, M6-20 |
| Design rows from D4a (items and drops) | 30 | 1 | M2-29, M3-02, M5-04, M5-06, M5-13, M5-21, M6-17, M6-18, M6-29 to M6-31 |
| Design rows from D3d (bosses 2) | 22 | 0 | M3-04, M7-14, M7-16 to M7-21, M8-06 to M8-15, M8-19, M8-20, M8-27, M8-28, M8-30, M8-33 to M8-35 |
| Design rows from D3e part 2 (Episodes 3 to 5) | 42 | 1 | M3-08, M3-19, M3-20, M5-04, M5-07, M5-13, M5-21, M6-06, M6-18, M6-22, M6-29 to M6-31, M7-05, M7-06, M7-16, M7-22 to M7-27 |
| Design rows from D3f (effects) | 30 | 1 | M2-13, M2-14, M2-17, M2-21, M2-29, M5-04 to M5-08, M5-22, M6-05, M6-22, M7-03, M9-05 |
| Design rows from D3e part 3 (Episodes 6 to 8) | 24 | 1 | M2-05, M2-10, M3-14, M3-16, M3-19, M3-21 to M3-23, M5-14, M6-29, M8-01 to M8-06, M8-16 to M8-18, M8-21 to M8-25, M8-35 |

Out of scope on purpose:

| Rows | Reason |
| --- | --- |
| E26 | anti tamper words never fire on a real ROM (D4d F34); not ported on purpose |
| D14 | the red bird of the time over scene is an ambient bird, nothing to port |
| F14 | test, service and debug mode: not part of the game flow (decision 4) |
| F23 | anti tamper random tasks never fire (D4d F34) |
| F27 | debug mode at power on (decision 4) |
| F34 | anti tamper checks |
| T28-T30 | probe limits and an original hang (T29 is chased in M8-18); sub area start row never used |
| DS13 | unused creature art |
| L16 | debug mode (decision 4) |
| M28 | MAME invincibility cheat note |
| N13 | enemy category names are player slang only |
| IT24 | web notes agree, nothing to build |
| W242 | probe limitation of camera jumps |
| FX30 | dead code entry, kept as an empty handler |
| W324 | probe note for later agents |

Row level table (a row range, the tasks, a note). "DONE" in a note means the port already has it; the listed task checks it against MAME.

| Rows | Tasks | Note |
| --- | --- | --- |
| **A. Player** | | |
| P01 to P10 | M5-18, M10-19 | DONE for Jack; the three other heroes are checked in M5-18 (scenarios D2a A1 to A14) |
| P11 | M9-01, M9-02 | tag team move |
| P12 | M5-02, M5-20 | friendly fire: gate DONE (a823109), the attack object pass in M5-02, scenarios M1 to M3 in M5-20 |
| P13 | M5-07, M5-18, M5-19 | hurt reactions; burn flames and burn death effects |
| P14 to P15 | M5-18, M5-19 | DONE; checked for all heroes (S14, S15) |
| P16 | M5-05, M5-19 | dizzy stars effect; the rule is DONE |
| P17 to P18 | M9-03, M5-19 | DONE; unit tests with the measured numbers (S13) |
| P19 to P21 | M5-19, M2-16, M2-17 | invulnerability windows, death, respawn |
| P22 | M2-17, M2-19 | respawn bubble and the join bazooka |
| P23 | M2-17, M2-01 | extra life, 1UP marker and pose |
| P24 to P25 | M5-19, M5-21 | DONE; score and HUD checked in S17, S18, S20 |
| P26 to P27 | M5-20, M5-07 | weapons; torch overlay |
| P28 | M5-20 | answered by D2c (WM04, P53): Hannah stabs by her 5 frame swing; checked in W5 and W6 |
| P29 to P30 | M5-20 | heavy objects, pickup lock (W1, W11) |
| P31 | M5-21 | food (A01 to A04) |
| P32 | M2-18, M2-19 | join in |
| P33 | M2-20, M2-21 | continue |
| P34 | M2-14 | time over kills every hero |
| P35 | M3-30 | DONE (clamp), watched in the autoplay |
| P36 | M6-14 | wading draw DONE; bubble effect in M6-14 |
| P37 | M6-21 | riding the Cadillac |
| P38 | M2-08, M2-09, M2-12 | scripted control |
| P39 | M4-17 | hero voices |
| P40 | M2-16, M2-17 | respawn flash, sparkle, air strike (the 1UP marker is a pool C effect, not pool B 0x48) |
| P41 | M2-03, M2-04, M2-19, M2-20 | hero choice at the three moments |
| P42 | M2-15, M5-18 | idle: hero idle animations DONE (A1), the GO hint in M2-15 |
| **B. Enemies** | | |
| E01 to E07 | M5-22, M3-30 | DONE; whole run checks |
| E08 | M5-13 | Vice and Walther never get dizzy |
| E09 to E11 | M5-22 | DONE |
| E12 | M3-03 | kill-all and the wave timeout; boss death chain |
| E13 | M3-03 | enemy name and bar in the HUD |
| E14 | M3-14 | off screen free, Z plus Y test in the lift |
| E15 to E17 | M5-22 | DONE (class 10, 20, 14) |
| E18 | M6-01, M6-02 | poachers |
| E19 | M6-07, M6-08 | Lash T. |
| E20 | M7-07, M7-08, M7-09 | Bludge |
| E21 | M5-13 | Walther |
| E22 | M7-10, M7-11 | Zeke |
| E23 | M6-11, M6-27 | the butcher knives and the dynamite stick are the only things an enemy gives (HU25, HU26) |
| E24 | M3-20, M6-29, M6-30, M6-03, M7-16, M8-01 | classes 16, 27, 43 (M3-20); 65, 66 (M6-29, M6-30); 78, 80 (M6-03); 35, 45 (M7-16); 22 (M8-01); 5 and 61 (Butcher) |
| E25 | M3-02 | boss reinforcement controller |
| E26 | OUT | anti tamper words never fire on a real ROM (D4d F34); not ported on purpose |
| **C. Dinosaurs** | | |
| D01 | M6-04, M6-05 | sleeping Shivat and Mack, Z marks |
| D02 | M6-02, M6-06 | answered: poacher hits never wake or hurt a dinosaur (HU04); only the hunter wakes it |
| D03 to D05 | M6-04, M6-05 | hostile model, calm at 0 HP, leave X |
| D06 | M6-04, M6-05 | Shivat |
| D07 | M6-03 | Mack |
| D08 | M5-10, M7-12 | Rock Hopper |
| D09 | M7-10, M7-11 | Zeke |
| D10 | M6-16 | DONE (class 49, 50, 73); the Episode 2 flocks in M6-16 |
| D11 | M8-01 | brontosaurus foot |
| D12 | M6-04, M7-12 | colours by palette byte |
| D13 | M6-17, M7-12, M9-07 | open item probes: do fire or a poacher bullet enrage a neutral dinosaur |
| D14 | OUT | the red bird of the time over scene is an ambient bird, nothing to port |
| **D. Bosses** | | |
| B01 | M5-09, M5-11, M5-12, M5-13 | Vice |
| B02 | M6-09, M6-10, M6-11, M6-12 | Butcher |
| B03 | M6-18 to M6-26 | Hogg and the car |
| B04 | M7-01 to M7-04 | Slice |
| B05 | M7-16 to M7-21 | Morgan and Morgue |
| B06 | M8-06 to M8-12 | Tyrog three forms and Tyrog 2 |
| B07 | M8-13, M8-14, M8-15 | Slisaurs |
| B08 | M8-27 to M8-35 | Fessenden |
| B09 | M3-03, M3-04 | boss HP table DONE; bar in M3-03, values checked in M3-04 |
| B10 | M5-03, M5-09, M6-09, M7-01, M8-27 | boss intros and music (S4 inside each boss task) |
| B11 | M3-03, M3-29 | slow motion, walk off, tally |
| B12 | M3-03 | DONE; boss death rank and kill-all |
| **E. Items, containers, hazards, special scenes** | | |
| W01 to W06 | M5-21, M6-17 | DONE; item scenarios A01 to A25; trunks (IT28) in M6-17 |
| W07 | M6-18 | radio |
| W08 | M5-07 | torch fire |
| W09 | M5-06 | explosions |
| W10 | M6-30 | falling rocks |
| W11 | M7-22, M7-23, M7-24, M7-25 | village scenery and fire |
| W12 | M8-23, M8-24 | flame vents |
| W13 | M6-29 | rolling barrels |
| W14 | M3-13, M3-14, M8-18 | lift |
| W15 | M3-08, M3-09, M6-18 to M6-22 | car |
| W16 | M3-11, M3-12 | row scroll |
| W17 | M8-02 | mine lamps, rain |
| W18 | M3-10 | screen shake |
| W19 | M3-16, M5-14, M6-15, M6-16, M7-22, M8-05, M8-21 | tile animation and palette cycling |
| W20 | M5-21 | suits of armor (statue container, check the hall) |
| W21 | M8-19, M8-20 | lab tanks |
| W22 | M3-19, M7-05 | garage props |
| W23 | M2-15 | GO arrow |
| W24 | M3-01, M9-04 | unknown pool B classes: all identified by D3e; the prop tasks of M3 to M8 and the audit M9-04 |
| W25 to W26 | M9-08, M9-10 | answered by D3e parts 1 to 3 (W31 and W303); the audit checks that nothing else is left |
| **F. Flow, time, HUD** | | |
| F01 | M2-14, M2-23 | DONE; time top ups |
| F02 | M2-13 | clock display |
| F03 | M2-14 | TIME OVER words |
| F04 to F05 | M3-30, M2-15 | DONE |
| F06 | M2-08, M3-15 | area changes, scripts |
| F07 | M2-12, M3-20 | openings |
| F08 | M2-25, M2-26 | map panel, episode card |
| F09 | M2-28 | story panels |
| F10 | M2-24, M2-25 | tally |
| F11 | M2-23 | DONE; event wiring |
| F12 | M1-04, M2-23 | start rank by the title wait |
| F13 | M2-01, M2-02 | credits, coin modes, free play, settings from a config file; the EEPROM test menu is OUT (decision 4) |
| F14 | OUT | test, service and debug mode: not part of the game flow (decision 4) |
| F15 | M2-01, M4-14 | demo sound setting |
| F16 | M2-29 | HUD weapon display |
| F17 | M10-15 | DONE; 2 and 3 player layouts checked in the 3 player runs |
| F18 | M2-18 | in level prompts |
| F19 | M2-20, M2-21 | continue, game over |
| F20 | M1-07, M2-22 | ranking screen and table, name entry |
| F21 | M1-03 to M1-15 | attract loop |
| F22 | M3-23 to M3-27 | ending and staff roll |
| F23 | OUT | anti tamper random tasks never fire (D4d F34) |
| F24 | M1-03 | boot screens |
| **G. Sound** | | |
| S01 | M4-14, M4-15, M4-16, M4-10 | music |
| S02 | M4-13, M4-17, M4-18 | voices, hit sounds, pan |
| S03 | M4-13 | requests recorded DONE; now with d1 and d2 |
| S04 | M2-13, M2-15, M4-12 | clock beep and GO sound |
| S05 | M4-16 | fade request DONE; class 96 timing |
| **H. Effects, camera, bounds** | | |
| X01 | M5-22 | DONE |
| X02 | M5-04, M5-05, M5-06, M5-07, M5-08, M2-17, M2-29 | effects, dust, debris, stars, burn, muzzle, score and text effects |
| X03 | M2-13, M2-29 | pool D |
| C01 to C05 | M3-15, M3-29 | DONE; fades and events in the flow checks |
| **I. Rows added by the design phase (see the blocks "Design rows from ..." below)** | | |
| **Design rows from D2a (player moves, file group I)** | | |
| P43 | M9-01, M9-02 | tag team trigger and moves |
| P44 | M5-02, M5-20 | friendly fire by the census count (gate DONE) |
| P45 | M2-17 | 1UP pose |
| P46 to P47 | M5-18 | frames without hurt box, dive window |
| P48 to P49 | M9-03, M5-18 | DONE; tests |
| P50 | M5-18 | wall jump places |
| P51 to P53 | M5-20, M5-18 | DONE |
| P54 | M2-09 | scripted states |
| P55 to P56 | M2-24, M5-18 | DONE |
| **Design rows from D3e part 1 (pool B, file group I)** | | |
| W27 | M5-14 | steam pipes |
| W28 | M3-17 | door smash |
| W29 | M3-17, M3-18 | hall door, window, hole, wooden door |
| W30 | M2-16 | respawn air strike |
| W31 | M2-05, M3-19, M5-14, M8-23 | breakable family |
| W32 | M6-16, M6-03 | Episode 2 ambient life |
| W33 | M3-16 | tile animation family and its driver |
| W34 | M9-04, M2-10, M3-15 | class 69, 66, 96 |
| W35 | M6-31, M3-23 | class 95 sky blend, showers |
| W36 | M5-21 | DONE (container variants) |
| W37 | M9-05, M3-14, M2-04 | stage cleared banner decision, red pillars (cage posts), 1P 2P 3P tags |
| W38 | M6-17 | DONE (treasures behind trunks) |
| **Design rows from D4d (front end, file group I)** | | |
| F25 | M1-04, M1-13, M2-21 | attract state kept |
| F26 | M1-03 | boot skip |
| F27 | OUT | debug mode at power on (decision 4) |
| F28 | M2-20 | continue countdown |
| F29 | M2-19, M2-20 | join and continue effects on time and score |
| F30 | M2-24 | tally rules |
| F31 | M2-22 | name entry |
| F32 | M3-23 to M3-27 | ending independent of hero |
| F33 | M1-12 | demo RNG reset and streams |
| F34 | OUT | anti tamper checks |
| F35 | M1-04, M2-23 | start rank |
| F36 | M2-02 | free play |
| F37 | M2-03, M2-04 | select rules |
| F38 | M2-25, M2-26 | map panel twice |
| F39 | M4-14 | sound filter |
| F40 | M2-18 | prompts |
| P57 to P59 | M2-18, M2-19, M2-03, M2-20 | join, joiner values, hero choice moments |
| F41 | M2-28 | story panels by party |
| P60 | M2-20 | continue keeps the level frozen |
| **Design rows from D4c (camera and flow, file group I)** | | |
| T01 | M3-08, M3-09 | stage 2 controller |
| T02 to T05 | M3-30 | DONE; camera, clamp, depth, drop in |
| T06 to T07 | M2-08 | wave lock ends, area end script release |
| T08 to T10 | M2-07, M2-08, M2-12 | scripted control, flags, item waits |
| T11 to T12 | M3-15 | fades, sub area switch |
| T13 | M2-08, M3-15 | area start |
| T14 | M2-15 | GO burst |
| T15 to T16 | M2-11, M2-12, M2-27 | openings and control frames |
| T17 to T18 | M3-20 | Episode 2 and 4 car openings |
| T19 to T20 | M3-03 | boss death chain, walk off |
| T21 | M6-18 to M6-22 | car section |
| T22 | M3-09 | mode 8 chase |
| T23 | M3-13 | lift ride |
| T24 | M3-11, M3-12 | row scroll |
| T25 | M3-10 | screen shake |
| T26 | M3-30 | DONE (clock and rank freeze) |
| T27 | M3-03 | G.$4D4 revives a hero in the death sequence |
| T28 to T30 | OUT | probe limits and an original hang (T29 is chased in M8-18); sub area start row never used |
| **Design rows from D3b (dinosaurs, file group I)** | | |
| DS01 to DS02 | M5-01 | monster list and passes |
| DS03 to DS04 | M6-04, M6-06 | sleeping Shivat, hostile model |
| DS05 | M7-12 | neutral Hopper |
| DS06 to DS09 | M7-10, M7-11 | Zeke |
| DS10 | M6-03 | Mack |
| DS11 | M5-10 | Hopper made by Vice |
| DS12 | M8-01 | foot |
| DS13 | OUT | unused creature art |
| DS14 | M7-12, M6-04 | burn of Hopper and Shivat |
| **Design rows from D2b (player state rules, file group I)** | | |
| P61 to P66 | M2-16, M2-17, M2-19, M2-23, M5-19 | respawn timeline, strike, bazooka, bubble, invulnerability, rank per death |
| P67 to P69 | M9-03 | DONE; unit tests |
| P70 | M2-01, M2-17 | extra life lists, 1UP marker |
| P71 to P73 | M5-19 | DONE; HP refill, no floating numbers, scoring |
| P74 | M2-14 | time over |
| F42 to F43 | M5-19 | DONE (HUD lives, low HP palette), S18 |
| W39 to W40 | M2-16 | air strike side effects |
| **Design rows from D2c (weapons and multiplayer, file group I)** | | |
| WM01 | M5-20 | DONE |
| WM02 | M2-29 | HUD weapon display |
| WM03 to WM12 | M5-20 | DONE; scenarios W1 to W14 |
| WM13 | M5-07 | torch glow |
| WM14 to WM15 | M5-20 | DONE |
| WM16 | M5-20 | friendly fire fists |
| WM17 | M5-02 | pass 0x10DCC |
| WM18 to WM19 | M9-01, M9-02 | tag team |
| WM20 | M5-18 | dive window |
| WM21 | M10-15 | DONE; players never block |
| WM22 to WM24 | M10-15, M10-18 | DONE; player count rules in the 3 player runs |
| WM25 | M5-20 | weapon sources |
| WM26 | M6-27, M6-28 | enemy thrown dynamite |
| WM27 | M5-20 | DONE |
| **J. Leads from a web summary (GPT)** | | |
| G01 | M3-17, M3-18 | Episode 1 scripted fall and kick (scripts 5, 6) |
| G02 | M5-09, M5-10 | Vice intro |
| G03 | M6-09, M6-10, M6-11 | Butcher |
| G04 | M6-18 to M6-22 | radio and car |
| G05 | M3-20 | Episode 4 opening |
| G06 | M7-01 to M7-04 | Slice |
| G07 | M5-13, M6-29, M7-05, M7-06 | Walther, bikers, fence, vents (answered in D3a 17) |
| G08 | M6-30 | stones from rocks |
| G09 | M7-07, M7-08, M7-09 | Bludge |
| G10 | M7-16 | Episode 5 story scene |
| G11 | M3-19, M8-02, M8-01 | mine doors, foot shadow |
| G12 | M8-06 to M8-11 | Tyrog |
| G13 | M8-16, M8-17 | Episode 7 (shelves rejected in D3a, one real bookshelf burst) |
| G14 | M8-19, M8-20 | tanks |
| G15 | M8-27 to M8-35 | Fessenden |
| G16 | M3-23 to M3-27 | ending |
| G17 | M5-18 | dash attacks |
| G18 | M5-21 | food and treasure names |
| G19 | M2-20 | continue screen |
| **K. Notes from the user** | | |
| U01 | M7-09, M6-27 | answered: only Bludge eats items; the alternate Punk drops a dynamite stick |
| U02 | M5-20 | DONE (weapon drop rules, W13) |
| **Design rows from D3a (humans, after group K)** | | |
| HU01 to HU03 | M6-06 | hunter wakes the dinosaur |
| HU04 to HU06 | M6-02 | poacher rules |
| HU07 | M6-01 | entrances |
| HU08 | M5-01, M6-02 | stale +$84 |
| HU09 | M7-09 | Bludge eating |
| HU10 | M7-07, M7-08 | Bludge moves |
| HU11 | M6-07, M6-08 | Lash |
| HU12 | M6-13 | alternate Blade |
| HU13 | M6-27, M6-28 | bomb carriers |
| HU14 | M6-29, M6-30 | barrels, rocks, bikers |
| HU15 | M8-01, M6-29 | foot and barrels hurt poachers |
| HU16 | M5-02 | pass 0x10DCC |
| HU17 | M2-05 | pass 0x10CD0 |
| HU18 to HU19 | M3-20 | scene actors |
| HU20 | M7-16 | class 35 to 36 |
| HU21 | M5-13 | Walther |
| HU22 | M6-01 | poacher gate quirk |
| HU23 | M3-01, M6-01 | wave counting by class |
| HU24 | M6-29, M7-07, M8-17 | library barrels, Bludges, the bookshelf |
| HU25 | M7-09 | no thug takes items (nothing to port) |
| HU26 | M6-27 | dynamite stick as an item |
| **L. Leads from the user's Recreation Blueprint** | | |
| L01 | M6-21 | car seats |
| L02 | M6-18, M6-31 | no food in Episode 3, gun order |
| L03 | M6-23 to M6-26 | Hogg |
| L04 | M5-09 to M5-13 | Vice |
| L05 | M6-11 | Butcher knives |
| L06 | M3-20 | Episode 4 gate |
| L07 | M6-17 | Episode 2 hidden item and the hunter wake |
| L08 | M7-09, M7-17, M7-26 | Episode 5 |
| L09 | M3-19, M8-11 | Episode 6 wall, Tyrog |
| L10 | M8-18 | Episode 7 |
| L11 | M8-19 to M8-25 | Episode 8 |
| L12 | M5-21 | DONE (gun supply by player count, IT21) |
| L13 | M5-20 | empty gun thrown (W2) |
| L14 | M2-20 | continue screen |
| L15 | M2-25, M2-26 | route map card |
| L16 | OUT | debug mode (decision 4) |
| **M. Leads from the Deep-Dive Supplement** | | |
| M01 | M9-10 | kill scores: DONE (TR10), verified by the audit |
| M02 to M03 | M6-27, M9-10 | Punk grenade, Blade drops (answered in D3a) |
| M04 | M6-02, M5-13 | Gutter, Walther never flinch |
| M05 | M7-09 | Bludge |
| M06 | M3-04 | boss HP fixed |
| M07 | M5-09 to M5-13 | Vice |
| M08 | M6-11 | Butcher |
| M09 | M6-24 | Hogg mirror move |
| M10 | M7-03 | Slice |
| M11 | M7-17 to M7-21 | Morgan, Morgue |
| M12 | M8-08, M8-09, M8-10 | Tyrog |
| M13 | M8-33 | Fessenden form 2 |
| M14 | M9-07 | machine gun counterattack of dinosaurs (probe) |
| M15 | M6-16, M6-03 | scenery fauna |
| M16 to M17 | M5-21 | DONE (item values) |
| M18 | M5-20 | weapons |
| M19 | M5-18 | DONE (points per action) |
| M20 | M5-13, M6-12, M6-26, M7-04, M7-21, M8-11, M8-15, M8-35 | boss kill scores in each boss task |
| M21 | M2-20 | continue counter |
| M22 | M2-23 | rank (X-rank-crosscheck) |
| M23 | M9-10 | rage state at low HP (probe) |
| M24 | M9-10 | thrown empty gun dazes (probe) |
| M25 | M9-02 | team move voices are web only (the ROM sends none) |
| M26 | M7-05, M6-31 | stage extras |
| M27 | M1-08, M1-09, M3-26 | story intro text, endings |
| M28 | OUT | MAME invincibility cheat note |
| **Design rows from D3c (bosses 1, after group M)** | | |
| BS01 | M3-02, M3-03 | shared boss services |
| BS02 | M3-04 | DONE (HP rows), checked |
| BS03 | M5-09 | Vice intro |
| BS04 | M5-11 | Vice AI |
| BS05 to BS07 | M5-12 | summon, rage, bullet |
| BS08 | M6-09 | Butcher starter |
| BS09 | M6-10 | Butcher moves |
| BS10 to BS11 | M6-11 | knife drop, target |
| BS12 | M6-23 | Hogg foot behaviour |
| BS13 to BS14 | M6-25 | bombs |
| BS15 | M6-24 | car behaviour |
| BS16 | M6-26 | death |
| BS17 to BS18 | M6-18 | radio, arrival |
| BS19 | M6-19 | steering |
| BS20 to BS21 | M6-20 | car versus world, end |
| BS22 | M6-21 | seats |
| BS23 | M6-24 | radio during Hogg |
| BS24 | M7-02 | Slice moves |
| BS25 to BS26 | M7-03 | afterimage, boomerangs |
| BS27 | M3-02 | helper lists |
| **N. Leads from the Deep-Dive Supplement 2** | | |
| N01 | M2-01, M10-21 | operator settings (as config file) |
| N02 | M4-16 | music map |
| N03 | M2-29, M9-10 | HUD bar lengths and rank number |
| N04 | M3-03 | enemy health bars |
| N05 | M5-18 | combat details |
| N06 | M9-10 | holding attack or jump speeds up scripted scenes (probe, port if real) |
| N07 | M6-19 | car controls |
| N08 | M7-06 | Episode 4 walk |
| N09 | M7-26 | Episode 5 |
| N10 | M6-29 | slow player barrels |
| N11 | M8-18 | Episode 7 |
| N12 | M8-26, M9-10 | Episode 8 |
| N13 | OUT | enemy category names are player slang only |
| N14 | M10-24 | hardware clocks, frame rate |
| **Design rows from D4b (time, rank, score, after group N)** | | |
| TR01 | M2-14, M2-23 | DONE |
| TR02 | M2-13 | clock display |
| TR03 | M2-14 | TIME OVER |
| TR04 | M6-20 | DONE; car mode time |
| TR05 | M2-19, M2-20 | time additions |
| TR06 to TR11 | M5-21, M2-14 | DONE (stage time commands, level map, aggression, scores) |
| TR12 | M2-24 | tally scoring |
| TR13 | M2-17 | extra life pose and marker |
| TR14 | M2-22 | high score table and name entry |
| TR15 | M2-01 | settings |
| **Design rows from D4a (items and drops)** | | |
| IT01 to IT03 | M5-21 | DONE |
| IT04 | M2-29 | HUD messages |
| IT05 to IT09 | M5-21, M5-06 | DONE; fuse spark in M5-06 |
| IT10 to IT11 | M5-04 | break effects and stones |
| IT12 | M6-29, M6-30 | drop tables for classes 65, 66, 70 |
| IT13 to IT19 | M5-21 | DONE |
| IT20 | M3-02, M6-18, M6-31 | Episode 3 helper list and radios |
| IT21 to IT22 | M5-21 | DONE |
| IT23 | M3-02 | boss helper drop rows |
| IT24 | OUT | web notes agree, nothing to build |
| IT25 | M5-13 | Walther drop |
| IT26 | M2-29 | HUD item message |
| IT27 | M6-29 | rolling barrel free window |
| IT28 | M6-17 | foreground layer over sprites |
| IT29 | M5-21 | BUG fixed in 76ea52e; re-checked by the item scenarios |
| IT30 | M5-21 | DONE |
| **Design rows from D3d (bosses 2)** | | |
| BT01 to BT03 | M8-13, M8-14, M8-15 | Slisaurs |
| BT04 | M8-19, M8-20 | tank breakers |
| BT05 | M7-16 | Morgan scene |
| BT06 | M7-17, M7-18, M7-19 | Morgan |
| BT07 | M7-20, M7-21 | Morgue |
| BT08 | M8-06 | Tyrog crate scene |
| BT09 | M8-07, M8-08 | Tyrog forms |
| BT10 | M8-09 | blades, fire |
| BT11 to BT12 | M8-10 | host, chain |
| BT13 | M8-12 | Tyrog 2 |
| BT14 | M8-27 | Fessenden form 1 |
| BT15 | M8-28 | death scene |
| BT16 | M8-30 | form 2 parts |
| BT17 | M8-33 | counter reaction |
| BT18 | M8-34 | heads |
| BT19 | M8-33 | poise |
| BT20 | M8-35 | last boss end |
| BT21 | M3-04, M8-11, M8-15 | boss HP tables per count |
| BT22 | M7-14 | DONE (target rule), shared skeleton |
| **Design rows from D3e part 2 (Episodes 3 to 5)** | | |
| W201 | M3-08 | Episode 3 one area |
| W202 to W203 | M6-31 | B95 sky blend |
| W204 | M6-18 | radio sources |
| W205 | M6-31 | tire stacks |
| W206 | M6-29 | road riders and barrels |
| W207 to W208 | M3-20, M6-18 | opening crash, car versus playable car |
| W209 | M7-06, M5-13 | Walther entry |
| W210 | M5-21 | garage tires drops (D4a) |
| W211 | M7-06 | overturned Cadillac is map art (DONE) |
| W212 to W213 | M7-05 | B0 variants |
| W214 | M3-19 | B1 garage gate |
| W215 to W216 | M7-05 | B0 and B1 no drops, B59 |
| W217 | M5-21 | foundry guns (D4a data) |
| W218 | M7-26 | Episode 5 start items |
| W219 | M5-07 | torch floor fire |
| W220 | M7-26 | meadow diamond |
| W221 | M6-06 | hunters wake Shivat |
| W222 | M7-25 | B48 |
| W223 to W225 | M6-30, M5-04 | rocks, stones, stone limit |
| W226 to W227 | M7-26, M7-27 | bridge barrels, diagonal camera |
| W228 to W231 | M7-22, M7-23, M7-24 | B5, B6, B10, B13 |
| W232 | M7-25 | B62 |
| W233 | M7-22, M7-25 | no attack box in village fire |
| W234 to W236 | M7-26 | buckets (D4a rows) |
| W237 to W240 | M7-25, M7-16 | B49, B70, chief scene |
| W241 | M6-22 | B89 and B90 context |
| W242 | OUT | probe limitation of camera jumps |
| **Design rows from D3f (effects)** | | |
| FX01 | M5-04 | pool C table (all classes in the tasks below) |
| FX02 to FX05 | M5-22 | DONE (classes 1, 4, 8, 26) |
| FX06 | M5-05 | dust |
| FX07 to FX08 | M5-04 | debris, stones |
| FX09 | M5-05 | dizzy stars |
| FX10 | M5-07 | burn death |
| FX11 to FX12 | M5-08 | muzzle, ricochet |
| FX13 to FX15 | M2-17 | respawn flash, 1UP marker, sparkles |
| FX16 | M5-04 | puffs |
| FX17 to FX19 | M5-06 | fuse spark, explosion, flash |
| FX20 | M9-05 | stage cleared banner (probe decides) |
| FX21 | M2-14 | TIME OVER words |
| FX22 to FX23 | M2-13 | pool D, clock |
| FX24 | M2-29 | HUD messages |
| FX25 | M5-07 | torch glow |
| FX26 | M7-03 | afterimage |
| FX27 | M6-05 | snore Z |
| FX28 | M6-22 | car smoke |
| FX29 | M2-21 | continue digits |
| FX30 | OUT | dead code entry, kept as an empty handler |
| **Design rows from D3e part 3 (Episodes 6 to 8)** | | |
| W301 | M3-19 | mine wall |
| W302 | M8-02, M8-17, M8-22 | door bursts B4, B52, B42 |
| W303 | M2-05 | prop hit rule |
| W304 to W305 | M8-02, M8-03, M8-04 | lamps, storm |
| W306 | M8-01 | foot shadow |
| W307 | M6-29 | barrel free window |
| W308 | M8-05 | swamp scenery |
| W309 | M8-06 | Tyrog scene |
| W310 to W311 | M3-14, M8-17 | lift labels, posts, smash, riders |
| W312 | M3-19 | library gates |
| W313 | M8-16, M8-17 | lab |
| W314 | M8-23, M8-24 | flame vents |
| W315 | M5-14, M8-22 | wall pipes |
| W316 | M3-16, M8-21 | tile clock |
| W317 | M8-21, M3-22 | sewer scenery |
| W318 | M3-21, M3-22 | end of lab and sewers |
| W319 | M8-22 | fire glow |
| W320 | M8-25 | meat barrel |
| W321 | M2-10 | caption banners |
| W322 | M8-18 | Episode 7 background is map data (DONE), checked in the walk |
| W323 | M3-23, M8-35 | after the final boss |
| W324 | OUT | probe note for later agents |

## 8. Risks and what we do about them

| # | Risk | Why it is real | Mitigation and fallback |
| --- | --- | --- | --- |
| R1 | **The car section of Episode 3** (M6-18 to M6-26, about 17.5 h with Hogg) | Ten parts: radio, a second object class, steering, four collision passes, seats, a delivery actor, camera mode 8, car scenery. `G.$518 = 2` was never seen, B89 and B90 are not placed in a walk, the kill score when the car kills Hogg is unknown. | M3 only ports the camera (M3-08, M3-09) and makes the radio a no-op, so the foot path of Episode 3 is playable and finishes with a placeholder Hogg. The car is split into five tasks, each with the D3c scenario numbers (BS-C1 to BS-C8). If the car stalls, ship the foot path with Hogg on foot first (M6-23, M6-25, M6-26 do not need the car) and finish the car last. |
| R2 | **Multi part bosses**: Tyrog (class word rewrites 4, 72, 4, 72, 4), Fessenden form 2 (class 84 with five parts), Morgan to Morgue, three Slisaurs | D3d decoded class 4, 72 and 84 "by action, not state by state". Their state tables carry open names. 12 h each for Tyrog and Fessenden form 2. | The shared boss skeleton first (M7-13 to M7-15). Each boss is 4 to 6 tasks with the D3d scenario frames. Use `step` mode and `objhex.py` to find the first differing state. Keep the placeholder until the traces are equal. Budget plus 35 percent on M8 (section 9). |
| R3 | **The demo and the RNG from power on** | The demo resets the RNG (G.$14 = 0x1C3) and replays 30 recorded streams, so after the reset it is deterministic. But a game that starts from the attract loop carries the RNG state of the frames played since power on, so every `rand()` of the boot, title, profile, movie and demo code must be called in the original order. The demo also needs real content to look right. | M1-00 compares the RNG per frame from power on. M1-12 builds the demo plumbing and checks the first frames; the six rows are compared again with real content in M10-03 to M10-05. If a screen's RNG use cannot be matched, the harness `follow` mode (RNG copied from MAME) still verifies the gameplay code, and the difference is listed. |
| R4 | **The sound driver** (M4, 35 h listed) | The driver is 99,729 notes of data and a Z80 program of about 6 KB with pan motion, priorities and twin headers. The QSound DSP ROM is not in the set, so the mixer is an approximation. | M4-00 finishes the spec, M4-01 and M4-09 give exactness at the register write level (the only level MAME can show us), the driver is split in four tasks and two exactness passes. The fallback (a tiny Z80 interpreter inside `dino-sound`) is a rule change and is not planned (decision 5). The game runs without audio if the device is missing. |
| R5 | **The scripted control interpreter** (M2-07 to M2-12) | Every area change, every opening and every boss walk goes through it. A mistake shows up in all eight episodes. | It is split in four tasks, unit tested with the frame counts of D4c 5.5 (A16), and used by the Episode 1 opening in M2 before anything else needs it. M3-29 compares the 13 area changes and 8 end flows with MAME. |
| R6 | **Pool B prop volume and unknown pictures** | 59 plus 38 classes. B13 (burning huts), B30 (the monitor pictures), B62 (a hardware word toggle), B71 (never spawned in a walk), B9 (lightning) are only partly understood. | The generic drivers of M2-06 and M3-16 make most classes data. The bundles are 1 to 2 h. Open pictures are compared with the picture tool (M1-14) against MAME screenshots, and anything not understood stays a documented difference. |
| R7 | **Probe `jump` is not natural play** | `jump` skips spawn records and keeps wave controllers before the target (D4c 4.3 warning, open item 9). A port that matches a jump scenario may still differ in a natural run. | M3-07 loads the controllers as they are. M10-07 to M10-19 compare natural whole runs per episode; those are the final proof. |
| R8 | **Placeholders hide bugs, or never go away** | A game that "works" with boxes can pass through stages that are wrong. | The registry prints every class still served by a placeholder. Autoplay runs from power on to the ending after every milestone. M9-14 removes the registry code and autoplay must pass without it. |
| R9 | **Review bandwidth** | 261 tasks, each reviewed and committed by the main session. | The gates (section 3) make the evidence uniform; most tasks have a numeric scenario. The main session reads the report and the screenshots first and the diff second. Small data tasks go to X workers. Allow files per milestone keep known differences in one place. |
| R10 | **External CLI quality and limits** | codex has a usage limit; X workers may miss quirks. | X is only for easy and medium blocks. The block's MAME check is the judge. When a CLI hits a limit the job moves to the next one (codex, agy, grok). A bad X result is redone by a Claude worker. |
| R11 | **Some design documents were written by CLI workers** (D3f, D4b, D4e, the index, part of D3e part 2) | They were reviewed, but they contain claims that no MAME run shows (for example D3f class 9 and D4e codes with no sender). | Every task has its own MAME comparison, and the errata file records corrections. The audit M9-08 to M9-10 reads every row once more. |
| R12 | **Slowdown and pacing** | The original runs a game frame in two vblanks when objects pile up; the port does not reproduce this. The original frame rate is about 59.64 Hz. | Logic is identical (one tick is one game frame, the clock counts game frames). Pacing at 59.64 Hz in M10-24. Documented as a known difference in the README. |
| R13 | **Exactness cascades** | A difference at frame k in a long run causes differences after k, and a long run may fail at its first object. | `step` mode for diagnosis, `run` mode for proof, the first difference rule, and per episode tasks of 2.5 h plus the class level scenarios that already passed. Budget plus 50 percent on M10. |
| R14 | **Unmeasured corners** | The 2 player cabinet layout (G.$4CF = 1), the Japan and USA region sets, a third player as tag initiator, some probe items never reached (D2c open items, D3b open items). | The 3 player layout is built; the 2 player layout is accepted in the config and keeps the 3 player layout (noted). The sweeps M9-06 and M9-07 run the probes the design documents asked for. |
| R15 | **Edits in main session files** | `main.rs`, `input.rs`, `present.rs`, `render.rs`, `harness.rs`, `mod.rs` are shared. | A block says "allowed" when it needs such an edit; one code worker at a time removes the merge risk. |

## 9. Honest estimate

Hours are agent hours of the task blocks (code, MAME comparison and the fix loop of one task). They do not include the main session's review time. The expected column multiplies
each milestone by the risk factor in the third column (rework found at review, exactness chases, design document errors). Best is the listed hours, worst is 1.7 times.

| M | Listed h | Lane A h | Risk factor | Expected h | Worst h (x1.7) |
| --- | --- | --- | --- | --- | --- |
| M1 | 24.2 | 23.2 | 1.20 | 29 | 41 |
| M2 | 51.5 | 47.5 | 1.25 | 64 | 88 |
| M3 | 51.1 | 47.6 | 1.25 | 64 | 87 |
| M4 | 35.0 | 29.5 | 1.40 | 49 | 60 |
| M5 | 36.9 | 32.9 | 1.20 | 44 | 63 |
| M6 | 58.9 | 56.9 | 1.35 | 80 | 100 |
| M7 | 49.5 | 47.5 | 1.30 | 64 | 84 |
| M8 | 66.4 | 64.4 | 1.35 | 90 | 113 |
| M9 | 25.0 | 15.0 | 1.50 | 38 | 42 |
| M10 | 57.5 | 53.5 | 1.50 | 86 | 98 |
| **Total** | **456.0** | **418.0** | | **608** | **775** |

How to read the calendar: one code worker at a time finishes about 4 to 5 tasks per working day (about 7.5 agent hours with review and commit). Lane A is the critical path; lane B (38 h) runs beside
it and does not add calendar time if it starts when its needs are met. With the expected hours the lane A calendar is about 74 working days (about 15 weeks); best about 56 working
days, worst about 95. NEW-PLAN guessed 210 hours for all of it; the design phase showed that the front end (M1, M2), the sound driver (M4), the verification tasks and the bosses are bigger than a one line milestone,
and that the ending, the placeholders and the harness tools needed owners. The index counted 295.8 hours for 151 rows. This plan lists 456 hours: the extra 160 are the sound driver and its tools (about 26 more than the index had), the ending (9), placeholders, review aids and harness tools (about 22), the M9 audit and sweeps (about 20), whole run exactness in M10 (about 57), and about 26 from splitting and re-measuring the index rows.

When the user can see things (lane A working days from the start, expected): attract loop at about day 4, the start of the real game at day 12, the whole skeleton from power on to the ending at day 20,
sound at day 25, a real Episode 1 at day 30, Episodes 2 and 3 at day 41, Episodes 4 and 5 at day 49, Episodes 6 to 8 at day 60, the small things pass at day 63, the finished game at day 74.

Where the estimate can move most: the car (R1), Tyrog and Fessenden (R2), the sound driver (R4), the audit fix budget in M9 (a guess of 8 hours for an unknown list) and the whole run chases in M10 (R13).

## 10. The first ten tasks, in order

Note at the time of writing (2026-10-13): another worker already has uncommitted changes in the tree that match the start of M1: power-on references in `re/harness/` (`DINO_CLOCK=video`, `DINO_SYNC=vblank:1`, `trace_prefix.py`, the `attract` and `coinstart` scripts: this is M1-00 under another name) and new `frontend.rs` files in `dino-assets` and `dino-extract` (M1-02a), plus edits in `harness.rs`, `main.rs`, `flow.rs`, `front.rs`, `snd.rs`. The main session reviews and commits that work first, marks M1-00, M1-01 and M1-02a done where the check of the block passes (the existing power-on tool replaces the `DINO_SYNC=power` wording of M1-00), and starts the list below at the first task that is still open.

Run them in this order. Tasks 1 and 2 start together (one code worker, one lane B worker). After task 1 is committed, tasks 3 to 10 run one at a time. The lane B worker then continues with
M4-00 (the sound spec), and after M2-00 is committed with M2-30, M2-31 and M3-06, so the harness tools are ready before M2 and M3 need them.

| # | Task | Lane, worker, hours | Why first | What the main session checks |
| --- | --- | --- | --- | --- |
| 1 | M1-00 Power-on trace mode for the harness | A, C, 2.0 | Every M1 and M2 screen is proven against MAME from power on; nothing else needs a running game. | MAME power-on trace made twice is equal; the port boots and writes a trace; diff.py names the first differing frame per layer. |
| 2 | M1-14 Picture compare tool | B, X, 1.0 | Cheap, python only, used by every screen task. | Equal folders give 0 percent; different folders are listed. |
| 3 | M1-01 Scene runner, text printers, boot entry | A, C, 1.2 | The base of every front end screen. | Printer unit tests; a printed string equals MAME's scroll 1 cells at frame 100. |
| 4 | M1-02a Front end data tables | A, X, 1.2 | Pure data, well specified by D4d Appendix B. | Table lengths equal the document; no ROM bytes in source. |
| 5 | M1-02b Picture loaders, tall font, typing helper | A, C, 1.2 | Title, slides, cards and captions need them. | Title pages draw like MAME once M1-03 exists. |
| 6 | M1-03 Boot screens and title | A, C, 2.0 | First visible result: the user's first two minutes. | Frames 73 to 330 exact; pictures at 100, 250, 330, 340, 420, 560, 700 equal. |
| 7 | M1-04 Attract state machine, coin edge, press start | A, C, 1.5 | The loop runs and a game can be started. | State change frames 2744 to 28953 equal MAME; coin and start frames of A10. |
| 8 | M1-05 Profile cards, layout and timeline | A, C, 1.0 | Next screens of the loop. | Card start frames 932, 1297, 1658, 2019. |
| 9 | M1-06 Profile pose objects | A, C, 1.0 | Completes the cards. | Sprite lists of the four cards equal MAME. |
| 10 | M1-07 Ranking screen | A, C, 1.5 | Completes the first attract half. | Pictures at 2330, 2500, 2740 equal MAME. |

The full block of each task is in section 6 under its id; a hand-out is the preamble of section 4, the rules of section 3 and that block.

After these: M1-08 (slides), M1-09 to M1-11 (movie), M1-12, M1-13 (demo), M1-15 (acceptance), then M2. A worker gets the preamble (section 4), the rules (section 3) and the block of its task.

## 11. Edits the main session makes to other files

The planning agent was allowed to rewrite `plan/PLAN.md` and add banners to the phase files only. These are the edits left for the main session.

1. `STATUS.md`: replace "Plan" and "Next steps" by a pointer to this file and the milestone table of section 2; set the progress table to the D5 state (design phase done, D3e part 3 done, D5 done);
   add a row per milestone with the percent of tasks done; keep "How to run".
2. `dino-rs/ARCHITECTURE.md`: add the owner rows and file list of section 5; add the new `crates/dino-sound`; add the rule "child modules with `#[path]` for `game/prop/` and `game/effect/`";
   add "workers may edit main session files only when their task block says allowed".
3. `plan/game/mechanics-seen.md` is not edited by this plan. After M9 the main session may set the Port column from the audit files `M9-audit-*.md`.
4. `.gitignore`: nothing new is needed (`re/out/` stays ignored except the asset dump). Add `dino.cfg` to `.gitignore` so a player's settings are not committed.
5. `re/harness/README.md`: documented by the lane B tasks that add tools (M1-00, M2-00, M3-06, M4-01, M4-09).
6. `plan/phase-*.md`: each got a banner naming the milestone that replaces it (done by this task). The phase files stay as history.
7. After each milestone: push, tag the commit `milestone-Mn` (optional), update the progress table and give the user the progress and ETA line.

## 12. Gaps that remain in the design inputs, and who closes them

| Gap | Where it is closed |
| --- | --- |
| Pool B 76 (camera mover of the boss intros) has no description | M5-03 reads the handler and writes a note in `re/specs/enemy/` |
| Pool A classes 90 to 93, 96 (ending explosions) and 97 (stand ins), pool B 81 to 87 details | M3-23 to M3-27 read the code and write a note in `re/specs/flow/` |
| Pool A class 56 (Episode 1 opening cast) | M2-11 |
| Pool B 58 and 75 to 78 (flow objects), 91 and 92 (story objects) | M2-24 to M2-26, M1-08 |
| B30 monitor pictures, B71, B66 var 7, B13 shapes, B62 hardware word | M8-16, M8-25, M7-23 to M7-25 |
| Hogg zone maps and the car's open items (B89, B90, `G.$518 = 2`, kill score by the car) | M6-22, M6-26 |
| D4c open items (Episode 7 hang, Episode 8 ending trigger with the real Fessenden, 2 player layout) | M8-18, M8-35, accepted gap |
| Unmeasured probe items of D2 to D4 (open item lists) | M9-06 and M9-07 |
| Sound: pan motion, ADPCM pan, tables, DSP internals | M4-00 and approximations in M4-08 |
| D3f class 9 banner, classes 0, 12, 13, 35 | M9-05 |
| Debug and test modes, region sets | Out of scope (decision 4) |

