# New plan: design first, then the whole-game backbone, then real content, then exactness

Written 2026-10-12 (research agent). It reorders `plan/PLAN.md` and the phase files. It does not edit them: the
main session does that after step D5 (list of edits in section 7).

**The plan starts with a design phase (section 1, no game code).** Deep documents with numbers and original
addresses for the player (D2), every enemy, dinosaur, boss, prop and effect (D3), and the world systems (D4), merged
into the final milestone plan (D5). After D5 come the build milestones M0 to M10 (section 6): first the backbone of
the whole game with placeholder fights, then real content episode by episode, then exactness against MAME.

## 1. Part A: the design phase (documents only)

Output folder: `plan/game/design/` (new). One document per task, each ending with **acceptance scenarios**
(a MAME poke script plus what must be seen or traced) and **open items**. Every statement gets a source tag
(`R:` repo or ROM address, `M:` MAME run, `W:` web). Agents may use `re/tools/d ADDR N`, `re/ghidra/decomp.c`
(read only, never copied into files), the probe scripts, and the web. No ROM bytes or text in the documents.

### D1 (done): research and structure

This folder. Output: `README.md`, `episode-1.md` to `episode-8.md`, `attract-and-front-end.md`, `mechanics-seen.md`, this file.

### T0: probe tool and recipe book (1 task, 1.5 h) - run first

Move the probe into `re/harness/`, add a `jump to (episode, area, camera X)` helper (poke stage, area, camera event
index and run the area start hook; or load a snapshot made by the harness at that point), a `spawn class N`
helper (exists: `DINO.spawn_enemy`, `spawn_item`; add pool B), a contact sheet script, and write
`plan/game/design/capture-recipes.md`. Done when: any episode and area can be reached in under 60 seconds of
emulated time from a script line, and the document lists the recipe for every screen in `README.md` section 2.

### D2: player mechanics (3 tasks, 2 to 3 h each)

Sources: `re/specs/player/SPEC.md` and `HANDOFF.md`, `dino-game/src/game/player*.rs`, the tables 0xA26FC (attack
data), 0x6C3A (invulnerability), 0x2C88E (hero factors), `re/specs/anim/`, MAME probes (poke HP, lives, enemies,
weapons), web: Arcade Quartermaster characters page (all move names), Hardcore Gaming 101.

| Task | Scope | Output |
| --- | --- | --- |
| D2a Moves | Every move of the four heroes: input, state ids, animation, attack id, damage, stun, reaction, hit stop, spark, sound, costs, which frames are immune, ranges. Differences between heroes. Hannah's knife stab. Tag team moves (what they do, which state, how they are triggered). Wall jump. | `design/D2a-player-moves.md` |
| D2b State rules | HP and lives; extra life thresholds and the 1UP; respawn; join and continue (bazooka, rank effects); every invulnerability window on one timeline; hurt, knockdown, get up, dizzy numbers; mercy rule; damage formula with all tables; special HP cost; scoring (hit, kill, container, food, treasure, bonuses); score popups. Settle the hero factor order (the two specs disagree). | `design/D2b-player-state-rules.md` |
| D2c Weapons and multiplayer | All weapons (15 types): ammo, damage, attack ids, charge swing and critical hit, throw, wear and breaking, the torch and the knife; pickup and drop rules; heavy objects. 2 and 3 players: friendly fire (is it on?), shared and per player rules (camera, spawns, thresholds, boss HP, drops), simultaneous join. | `design/D2c-weapons-multiplayer.md` |

### D3: every enemy, dinosaur, boss, prop and effect (6 tasks, 2 to 3 h each)

Sources: `re/specs/enemy/SPEC.md` (sections 3, 7, 10, 11, 12, 17), `stage_scripts.txt`, the class map
(`re/specs/enemy/classmap.txt`), `re/tools/d`, `re/tools/d2`, MAME with `DINO.spawn_enemy` (poke any class at any
place and watch it), web (Arcade Quartermaster enemies and bosses, HG101, TV Tropes, YouTube boss videos, Namu wiki
via search). Method that worked in phase 1: read the listing with `d2`, write the behaviour in words and tables, then
confirm with a poke scenario and screenshots.

| Task | Scope | Output |
| --- | --- | --- |
| D3a Humans | Pool A classes 3 (Lash), 6 (Bludge), 13 (poachers: rifle logic), 14 (finish the spec), 16, 18 (Walther), 22?, 27, 35, 43, 45, 63, 64, 65, 66, 70, 78, 79, 80: role, variants (+$26), entrances, AI states, attacks and damage ids, speeds, HP by level, score, drops, handler address, where used. Identify every "unknown" class by spawning it in MAME. | `design/D3a-humans.md` |
| D3b Dinosaurs and flyers | Shivat (finish), Mack, R Hopper, **Zeke (the pteranodon: flight path, dive, damage, why it can only be hit by jumping)**, sky birds, the red bird, the brontosaurus (class 22?), Vice's raptor; the hostility model (asleep, wander, enraged, calm, leave X), how poachers and fire wake or anger them, colours, what each does to heroes and to poachers, drops and scores. | `design/D3b-dinosaurs.md` |
| D3c Bosses 1 | Vice (copy and check the spec), the Butcher (+ class 5), Hogg (+ car section), Slice: intro, every attack, HP and bars, phases, helpers, weapons the player can take, music, death scene. | `design/D3c-bosses-1.md` |
| D3d Bosses 2 | Morgan and Morgue (+ classes 35, 45), Tyrog three fights (classes 4, 72, 91), Slisaurs (class 71 and starters 41), Fessenden both forms (84, 86 to 89, 94 and starters 71, 83, 84), the lair. | `design/D3d-bosses-2.md` |
| D3e Pool B inventory | All 59 pool B classes used by the scripts (and others spawned at run time): role, who spawns it, when, what it draws or does, size and difficulty to port, which episodes. Wave end type 0xE (Episode 2 wave 4). Scenery, hazards (falling rocks, flame jets, rolling barrels, lift parts, tanks, doors, rain). | `design/D3e-pool-b-inventory.md` |
| D3f Effects | All pool C classes (about 40, table 0x237E6) and pool D: who spawns each, what it looks like, timing, sound, used in which episode. | `design/D3f-effects.md` |

### D4: world systems (5 tasks, 2 to 3 h each)

| Task | Scope | Output |
| --- | --- | --- |
| D4a Items and drops | 18 foods, 16 treasures, 15 weapons, containers (13 variants) with their art, HP and drops; every drop row used by the 8 scripts decoded to a list; hidden items (behind trees); the radio. | `design/D4a-items-drops.md` |
| D4b Time, rank, score | Stage time table, set and add events (scripts, deaths, 0x10482, 0x104A8), **clock display rules** (class 31: 3:00, 30 s marks, 15 s, red, beep), TIME OVER words, car mode time; rank, level, aggression tables per stage and area; extra life; score table 0x10292; bonus; settings that change them (EEPROM "timer level", difficulty). | `design/D4b-time-rank-score.md` |
| D4c Camera, bounds, transitions | All camera events per episode (table 0x21B2A) with the meaning of each; player bounds; GO arrow object and hint; area changes (fade, no fade, auto walk 0x1D95E, stage exit 0x1DAC4); scripted control 0x1DA10 / 0x1DB0C / 0x1DB5E; opening scenes of Episodes 1 to 3; car mode; lift; row scroll; the end flow of a stage (boss to tally). | `design/D4c-camera-flow-transitions.md` |
| D4d Front end and story | Everything in `attract-and-front-end.md` to the last open question: boot, attract (title, profiles, ranking, demo, slides, movie) with data lists to extract, coin and settings, select, join, continue, game over, link scenes (tally, map card, 7 story panels, episode card), ending and staff roll, ranking and the missing name entry, 2 and 3 player differences. | `design/D4d-front-end-and-story.md` |
| D4e Sound cues and effects per event | Music per episode, area and boss (table 0xA582, 0xA5C8), title, select, continue, ending tunes; every sound code used by the game with its event (hit, pickup, clock beep 0x34, GO hint 0x3E, voices, explosions), pan rules; demo mute. Hooks the game must call. | `design/D4e-sound-cues.md` |

Optional D4f (if D3f and D4a leave gaps): hazards, fire, water and explosions with numbers (flames of a torch, burn
damage, falling rocks, flame jets, barrels), `design/D4f-hazards-fire-water-explosions.md`.

### D5: merge (1 task, 3 h)

Read all documents, resolve conflicts, and write `design/D5-final-plan.md`: the final milestones of Part B with one
brief per task (scope, files, routines, data to extract, acceptance scenarios copied from D2 to D4), a risk list, and the
exact edits to `plan/PLAN.md`, the phase files and `STATUS.md`. The main session reviews it with the user before any
implementation starts.

### Suggested order

T0 first. Then D4d and D4c (they decide milestones M1 to M3, the biggest visible gaps), D3e (class inventory sizes
the work), D3b (the dinosaurs the user named), D2a to D2c, D3a, D3c, D3d, D3f, D4a, D4b, D4e, D5. With three research
agents in parallel: lane 1 = T0, D4d, D4c, D4e; lane 2 = D3e, D3b, D3a, D3f; lane 3 = D2a to D2c, D4a, D4b, D3c, D3d.
D5 starts when all three lanes are done. About 16 tasks, 35 agent hours.

## 2. Why the plan changes

The user ran the port and said it is "totally different" from the original. They are right. Facts [see `README.md`]:

- The port starts directly in a level. No boot, no attract loop, no title, no coin, no select, no story.
- Episodes 2 to 8 cannot even start (a panic in the music table lookup). Episode 1 plays with 5 of 7 enemy
  classes (and only 2 of 10 prop classes), no boss, no opening scene, no tally.
- 57 of 59 prop classes, 36 of about 40 effect classes, all 9 bosses, all dinosaurs, the clock, the GO arrow,
  the car, the lift, the ending and all sound are missing.
- The old plan finished one level to frame exactness before the next level had even a title card. It was
  built from the bottom (objects) upward. The user judges the game from the top (what a player sees).
- The user added: "there are many small things, like a bird dino flying and giving damage, invincibility,
  whatever. You really need to dig very deep into game design first, implement later."

So the new order is: **(A) design phase** (documents with numbers and addresses, no game code), **(B) the
backbone** (the whole flow from power on to the ending with placeholder fights), **(C) real content episode by
episode**, **(D) exactness**. Every milestone has a thing the user can play and look at.

## 3. Rules that stay and rules that change

Stay (from `plan/PLAN.md`, `dino-rs/ARCHITECTURE.md`, memory): one task is one agent run of about 1 to 2 hours
and ends with a green build, evidence, review and commit by the main session; agents never start agents; no ROM
data or derived data in git; every ported routine names its original address; data goes through `dino-extract`.

Change:

1. **Design before code.** No more "decode and port in one task" for new areas. A design agent writes a document
   (numbers, addresses, behaviour, acceptance scenarios); a port agent then works from that document.
2. **Placeholders for the skeleton.** Until a class is ported, the skeleton uses a generic placeholder enemy
   and an invisible prop so that waves, locks, GO, the clock and bosses' end flow still work (milestone M3).
   Each real class port deletes its placeholder use. This is the "even with some enemies simplified" rule.
3. **The user checks each milestone by playing**, so each milestone below lists "what to look at".
4. **Research agents may run in parallel** (they only write documents under `plan/game/design/` and use MAME).
   Code agents stay one at a time. This is a proposal for the user: it cuts the design phase from about 8 working days to
   about 4.
5. **MAME is the judge.** Where the web and MAME disagree, MAME wins (example: the web says the heroes "drive" in
   Episode 3; MAME shows the heroes fighting on and beside the car after taking a radio).

## 4. Decisions I made while writing (no questions were possible)

1. Ground truth order: ROM code and MAME runs, then repo specs, then web sources.
2. No high score name entry was seen in MAME. Treat the ranking table as display only until D4d proves otherwise.
3. The World set (`dino`) only. Regional name changes (Japanese set) are ignored. Boss names follow the web and the game.
4. Settings (coins, lives, difficulty, demo sound) come from a config file with the factory values; the EEPROM test
   menu is out of scope until D5 decides (it is in `mechanics-seen.md` F13, F14).
5. Existing ports (player, items, class 2, 10, 20) are kept, not redone.
6. The demo and attract loop must be exact from power on (MAME can record from power on, see Tools), because the
   user will see them first.

## 5. Tools (used in this research, to be kept)

Everything lives in the scratchpad now (`/private/tmp/claude-501/-Volumes-Shanto-Repos-Personal-Github-roms/9337dc4f-36ab-46a5-800b-fa02f2ab2b75/scratchpad/research/`),
because the task rules forbade scripts in the repo. Recommendation: the main session moves `probe.lua` and
`run_probe.sh` into `re/harness/` (they contain no ROM data) and task T0 (below) documents them.

| Tool | What it does |
| --- | --- |
| `probe.lua` + `run_probe.sh NAME` | Runs the original in MAME with no window. Env: `PR_INPUT` (input script file, same format as the harness: `a-b:KEYS`, frames from power on), `PR_POKES` (lines `frame hexaddr size value`), `PR_SNAPS` (`f1,f2,..` or `every:N:from:to`), `PR_FRAMES`, `PR_WATCH` (`hexaddr:size,..` logged each 30 frames). Writes `p_NAME/log.txt` (mode, sub mode, stage, area changes and watches) and screenshots in `p_NAME/snap/dino/`. About 20x real time. |
| `attract.lua` | Same idea for the attract loop; logs state changes of G.$0FD0 and the title and story counters. |
| `sheet.py` | Makes a contact sheet (grid of labelled thumbnails) from a screenshot folder, so 16 to 30 frames can be read at once. |
| `digest.py`, `cover.py`, `compact.py`, `sum.py` | Turn `re/specs/enemy/stage_scripts.txt` into compact per episode digests and class coverage counts. |
| Port runs | `./target/release/dino-game --rom ../dino.zip --stage N --input-file FILE --screenshot OUT.png --frames N` (stage 1..7 panic today). |

Recipes proved in this research (RAM addresses are `0xFF0000 + G.$xxxx`; stage = G.$4D9 `0xFF84D9`, mode = G.$4D0
`0xFF84D0`, sub mode G.$4D1 `0xFF84D1`, area G.$786, time word G.$4E8 `0xFF84E8`, player 1 object `0xFFB274`):

| Goal | Recipe |
| --- | --- |
| Start a game from power on | input `600-607:C`, `700-705:S`, `900-905:A`; the level starts at about frame 1060 |
| Show the story panel and card of episode N | at frame 1300 poke stage = N, sub mode = 0, mode = 8; episode N is entered after the panel (250 to 440 frames) |
| Show the clear tally | at frame 1300 poke sub mode = 4 and G.$508 (word `0xFF8508`) = 0, G.$4D5 = 1, G.$4D4 = 1 |
| Show the ending | at frame 1300 poke stage = 7, `0xFF84D7`, `0xFF84DB`, `0xFF8507` = 1, sub mode = 0, mode = 0x1C |
| Show the continue and game over | poke player 1 lives (`0xFFB317`) = 0 and HP words (`0xFFB2DE`, `0xFFB2E0`, `0xFFB2E2`) = 0 |
| Show the clock and TIME OVER | poke the time word `0xFF84E8` = 0x0013 (13 seconds) during play |
| Continue with a credit | coin at 1760, start at 1800 after the death above |

## 6. Part B to D: the build milestones (draft; D5 confirms or changes them)

Each task is one agent, 1 to 2 hours, sequential. "Check" = what the user looks at. "MAME" = the comparison that
proves it. Original routine addresses are from `attract-and-front-end.md`, `README.md` and the specs.

### M0: every episode starts (1 task)

- M0.1 Extract the missing text tables (0xA582, 0xA5C8 and what else `flow_area_music` and the flow read), remove the
  panic, add `--area` and `--episode` options, show unported objects as small labelled boxes with the class number (debug
  aid, off by default). Files: `dino-extract/src/text.rs`, `game/flow.rs`, `main.rs`.
- Check: `--stage 1` to `--stage 7` run and show the level; no panic in 8 x 3 screenshots (script in the scratchpad).
- Depends on: nothing. Replaces: nothing (new). Do it right after D5 (or earlier if the user wants to see backgrounds now).

### M1: boot to title to attract loop

"The first five minutes of the original." The user sees: boot text, warning, title with blinking INSERT COIN, the four
profiles, the ranking, the demo play (it plays whatever the levels have at that time: Episode 1 for real, the other
levels with the placeholders of M3, real content after M5 to M8), story slides, the opening movie, and the loop again
(sound later in M4).

| Task | Scope | Routines | Files | MAME check |
| --- | --- | --- | --- | --- |
| T1.1 Front end services | Rust "task" helper for sleeping state machines; string printers for scroll 1 (0xBB6, 0xC0A, 0xC44, 0xC9A, 0xB52, 0xF94); picture block loader (0x98BAE, 0x84F72, 0x20592, 0x204CA); data extraction (strings, picture blocks); `main.rs` boots into the attract loop by default (`--stage` keeps the direct start) | `flow_boot`, `front.rs` | `game/flow/front.rs`, `dino-extract/src/text.rs`, `main.rs` | unit tests; one screen printed equal to MAME's tilemap |
| T1.2 Boot screens and title | boot text, warning, title (0x9A3CE: logo objects pool B class 98 variants 0 and 1, INSERT COIN task 0x98874, text swap at frame 360, 600 frames) | 0x9A3CE, 0x98874 | `flow/attract.rs` (new) | tilemap, palette and sprite list equal to MAME at 5 frames |
| T1.3 Profiles and ranking | 0x9A7E6 with tables 0x9A8E4, 0x9A908, the portraits and poses; 0x98330 ranking with the default table | 0x9A7E6, 0x98330 | `flow/attract.rs`, extractor | frames 1000, 1150, 1300, 2000, 2400 equal |
| T1.4 Demo | 0x99F10, 0x99E88, input playback 0x9A146 and the data streams, the demo table 0x9A1E2 / 0x9A1EE; falls back to rows whose level exists | 0x99E88, 0x99F10, 0x9A146 | `flow/attract.rs`, extractor, `input.rs` | demo 1 (Episode 2) needs M0 and M6 content to look right; until then check the first 300 frames by trace |
| T1.5 Story slides and opening movie | 0x988C0 and 0x9BF50 with 0x9C7C4 and 0x9C7F8, the picture data, timelines | 0x988C0, 0x9BF50 | `flow/attract.rs`, extractor | frames of each slide and of the movie equal to MAME |
| T1.6 Attract state machine | 0x97C20 with the state byte, loop order, coin and start edge from the system task (minimal), "press start" 0x983EE leads to the select (stub starts Episode 1) | 0x97C20, 0x9D048 | `flow/front.rs` | state change frames equal to MAME (2731, 6862, 9299, 13399) |

- Dependencies: D4d, T0 (and M0 for the demo). Replaces: `phase-02` tasks 1 and 2 (reordered, and design first).
- Honest risk: demo exactness needs the RNG state from power on to match; the harness must support recording from
  power on (the `vblank:N` sync already exists). The demo streams are data in the ROM (not yet located).

### M2: coin, select, start, Episode 1 opening, join, clock, continue, game over

The user can play the **start** of the real game: coin, press start, select a hero (10 second timer), watch the opening
scene of Episode 1, play, see the clock count down in red and the TIME OVER, die, get the villain's continue screen, pick
a hero again, or lose and see GAME OVER and the ranking.

| Task | Scope | Routines |
| --- | --- | --- |
| T2.1 System task | coins, credits, free play, config file for the factory settings, credit text, start edge, press start | 0x9D048, 0x9D4BE, 0x9D386, 0x9D196, 0x983EE |
| T2.2 Character select | panels, 10 s timer, cursor order, stars, JOIN-IN and INSERT COIN blink, auto pick at 0 | 0xAD46, 0xAF7C, 0xB06A, 0xB122, 0xB25E, 0xB444 |
| T2.3 Scripted control and the Episode 1 opening | the walk in script (G.$4DD, G.$4E3), caption banners (pool B 66), master 74, classes 56, 57, intro task state 4 | 0x1DA10, 0x1DB0C, 0x1DB5E, 0xB9BE, 0x89B48 |
| T2.4 Join task | in level prompts (48 on, 80 off), join with a credit, quick select strip, drop in | 0x6C4A, 0x7562, 0x65A4 |
| T2.5 Continue and game over | villain screen, 20 to 0, taunts, accepting start, red flash, GAME OVER, rank effects | 0xE9E6, 0xAC06, 0xAC26, 0x184E2 |
| T2.6 Clock, TIME OVER, GO hint, GO arrow | pool B class 31 with pool D slot 15, effect class 0x70, class 20 hint and sound 0x3E, the arrow object | 0x81046, 0x96450, 0x96488, 0x7BB68 |
| T2.7 Clear tally | the 0xB97E panels and counts | 0xB97E |
| T2.8 HUD leftovers | weapon display, extra life pose and 1UP marker, score pop-ups | 0x2389C, 0x6B98, 0x1843C |

- Dependencies: D2b, D4b, D4c, D4d, M1.T1.1. Replaces: `phase-01` task 6; `phase-02` tasks 1, 3, 4; part of task 8.
- MAME: the scripted runs from power on (coin, start, select, 600 frames of Episode 1) with the harness; the clock and the
  continue by poke recipes (section 5).

### M3: the whole game skeleton (all 8 episodes, placeholder fights)

The user can play **from power on to the ending** (with a debug key that kills all enemies, so it takes minutes):
every link scene, every camera lock, GO arrow, clock, boss gate, tally, intermission, episode card, ending, staff roll,
back to the attract loop. Fights are placeholders.

| Task | Scope | Routines |
| --- | --- | --- |
| T3.1 Background and camera services | stage 2 controller and row scroll, screen shake, scroll 1 elevator background, tile animation, palette cycling objects | 0x22398, 0x2335A, 0x222D6, 0x205C6, 0x89746, 0x8B058 |
| T3.2 Area changes | scripted walk to the next area and stage exit, remove the TEMP shortcuts | 0x1D95E, 0x1DAC4, 0x1E5F6 |
| T3.3 Story panels | the 7 mode 8 entries | 0xD898 and entries |
| T3.4 Cards | map panel, episode card with typed title, intro states 0 to 2 | 0xB9BE |
| T3.5 Car skeleton | car state G.$518, radio event, camera mode 8, players on the car, Hogg gate | 0x8BD24, 0x8C06E, 0x18A02, 0x121D8.. |
| T3.6 Placeholders | one generic enemy for every unported pool A class (HP from the table, walks, hits, dies, uses census and waves); invisible pool B stubs; placeholder bosses with HP bars that end the episode; classification `PLACEHOLDER` visible in debug | new `game/placeholder.rs` |
| T3.7 Ending and staff roll | mode 0x1C, scenes, staff strings, end to ranking | 0xA69B8 |
| T3.8 Ranking | live ranking table with real scores, ranking screens | 0x16230, 0x16260, 0x98330 |
| T3.9 Review aids | `--episode N --area A`, debug keys (kill all, skip wave, set time), and a bot input script per episode | `main.rs`, `input.rs` |

- Dependencies: D4c, D4d, M2. Replaces: `phase-04` (all), `phase-12` ending part, `phase-13` task 5 partially, `phase-01`
  task 8.
- MAME: flow traces. For each episode record a MAME run with pokes that clear enemies and compare the sequence of mode,
  sub mode, stage, area, camera events, and frame counts of the clear and link scenes with the port.

### M4: sound on

| Task | Scope |
| --- | --- |
| T4.1 | sound data in `dino-assets` / `dino-extract` (Kabuki keys, tables, samples), `re/specs/sound/SPEC.md` |
| T4.2 | `dino-sound` driver port (command, sequencer, envelopes) with unit tests |
| T4.3 | QSound mixer and WAV example |
| T4.4 | `audio.rs` with cpal; `Game::sound_ex` with d1, d2 |
| T4.5 | cue wiring from D4e: music per area and boss, front end screens, clock beep, voices |

- Dependencies: D4e, M3. Replaces: `phase-03` tasks 1 to 4 (task 5 exactness goes to M10).
- Check: listen from the attract loop through Episode 1; MAME: sound request stream equal (existing harness).

### M5: Episode 1 for real

| Task | Scope |
| --- | --- |
| T5.1 | finish class 14 (Blade, Razor and parts, in progress now) |
| T5.2 | Vice T. (class 18 variant 0), bullet class 59, reinforcement controller (pool B 22), Walther |
| T5.3 | pool B classes of Episode 1 (12, 14, 40, 66, 69, 74 and others from D3e) |
| T5.4 | effect classes used in Episode 1 (dust, debris, dizzy stars, burn, muzzle, sparks) |
| T5.5 | Rock Hopper of the boss fight if D3b says so; velociraptor class 29 |
| T5.6 | Episode 1 exactness: full run in MAME with scripted input, 1 and 3 players |

- Dependencies: D3a, D3c, D3e, D3f, M3. Replaces: `phase-01` tasks 4, 4b, 5, 7, 9; `phase-05` part.
- Check: play Episode 1 from the opening to Vice's defeat and the tally; MAME: `run` mode equal for the whole episode.

### M6: Episodes 2 and 3

About 14 tasks: Episode 2: poachers (13) with rifle logic, Lash T. (3), classes 63, 79, 78, 80, 16; Shivat and the
dinosaur model (24) with sleeping, hostility, leave X; Mack (38); class 5 and the Butcher (61); pool B 2, 7, 27, 72, 80;
the wading and swamp effects. Episode 3: car mode complete (passes, car object, hits), Walther, class 65, 70, 64, B 95,
Hogg (31), radio, row scroll finishing. Each of the two ends with a full MAME run. Dependencies: D3a to D3e, D4c, M5.
Replaces: `phase-06`, `phase-07`.

### M7: Episodes 4 and 5

About 16 tasks: Episode 4: classes 27, 43, 70, props 0, 1, 59, Slice (25). Episode 5: Zeke (82, with the flight and
dive), R Hopper (29), Bludge (6), classes 35, 45, 66, torches and the glow overlay (item class 6), fire and burn, falling
rocks, flames and huts (pool B 5, 6, 10, 13, 48, 49, 50, 62, 70), Morgan (36), Morgue (40). Replaces: `phase-08`,
`phase-09`.

### M8: Episodes 6, 7 and 8

About 20 tasks: Episode 6: the brontosaurus (22), mine lamps, rain, props 3, 4, 7, 8, 9, 39, 55, 56, 60, 79, Tyrog
three fights (4, 72, 91). Episode 7: lift props (54, 57, 64), library props, classes 30, 33, 41, 52, 61, 63, Slisaurs
(71). Episode 8: tanks (44), sewer and cave props (28, 29, 36 to 38, 42, 43, 45, 53, 71, 83, 84), Zeke at the cave,
Fessenden both forms (84, 86 to 89, 94). Replaces: `phase-10`, `phase-11`, `phase-12` (boss part).

### M9: small things pass

Everything left in `mechanics-seen.md` with state STUB, MISSING or PARTIAL: tag team move, friendly fire, hero voices,
weapon details, remaining effect classes, easter egg and random tasks, test mode decision, `mechanics-seen.md`
re-read row by row. About 6 tasks.

### M10: exactness and finish

`phase-13` plus the exactness passes: per episode full runs for 1 and 3 players, attract loop from power on (2 full
loops), continue and game over, ending, all heroes, settings. Docs and a README for players. About 10 tasks.

## 7. How the old plan maps to the new one

| Old | New | Change |
| --- | --- | --- |
| phase-00 harness | done; add power-on recordings (T0, T1.x) and flow traces (M3) | extended |
| phase-01 tasks 1, 2, 3, 3b | done | unchanged |
| phase-01 task 4a (in progress) | M5.T5.1 | continues |
| phase-01 tasks 4, 4b, 5, 7, 9 | M5 | later, after the skeleton |
| phase-01 task 6, 8 | M2 (T2.4, T2.7) and M3 (T3.2) | earlier |
| phase-02 front end and flow | M1 and M2, after D4d | split, reordered, design first |
| phase-03 sound | M4 | earlier than the stage phases, after the skeleton |
| phase-04 background extras | M3.T3.1 | before the stage phases |
| phase-05 effects | spread over M5 to M9 by episode, plus D3f | replaced by need |
| phase-06 to 12 stages 2 to 8 | M6, M7, M8 | after the skeleton; each episode starts from a playable placeholder |
| phase-13 final verification | M10 | unchanged |

Edits the main session makes after D5 (not done by me): `plan/PLAN.md` table and status column; a banner in
`plan/phase-02` to `phase-12`; `STATUS.md` "Plan" and "Next steps"; `dino-rs/ARCHITECTURE.md` section "File ownership"
(new `flow/attract.rs`, `placeholder.rs`).

## 8. Effort, honestly

| Part | Tasks | Agent hours (1 to 2 h each) |
| --- | --- | --- |
| T0 and design D2 to D5 | 16 | 35 (range 28 to 45) |
| M0 | 1 | 1.5 |
| M1 | 6 | 10 |
| M2 | 8 | 14 |
| M3 | 9 | 16 |
| M4 | 5 | 8 |
| M5 | 6 | 10 |
| M6 | 14 | 25 |
| M7 | 16 | 28 |
| M8 | 20 | 38 |
| M9 | 6 | 9 |
| M10 | 10 | 18 |
| Total | about 117 | about 210 (range 150 to 290) |

Calendar: with one code agent at a time and one review and commit per task, about 4 to 5 finished tasks per working
day, so **about 25 to 30 working days** for Part B to D after about 5 days of design (research agents in parallel).
Using worktrees for independent classes inside M6 to M8 (two code agents at a time, different files) cuts that by about a
third. What the user sees: the attract loop after about 5 + 3 working days (M1), the start of the real game at about
5 + 6 days (M2), the whole skeleton at about 5 + 10 days (M3), sound at 5 + 12 days (M4), a real Episode 1 at 5 + 14 days,
then roughly one episode pair every week.

Biggest uncertainties (could move the numbers by 30 percent): the 59 pool B classes (D3e sizes them), the multi-part
bosses (Tyrog, Fessenden, each maybe 4 to 6 tasks), the car section, and the demo input data and RNG match from power on.

## 9. First actions

1. The user (or the main session) approves this plan, especially: design before code, research agents in parallel, MAME
   as the judge, placeholders for the skeleton.
2. Run T0, then the three research lanes of section 1.
3. Review D5 with the user, then M0 and M1.
