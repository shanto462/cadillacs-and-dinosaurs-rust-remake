# The whole game, as a player sees it (power on to the ending)

Written 2026-10-12 by the research agent for "the port feels totally different from the original".
This file is the map. The details are in the other files of this folder:

| File | What it holds |
| --- | --- |
| `README.md` (this file) | The ordered flow of the whole game, the rules that cross episodes, the gap list |
| `attract-and-front-end.md` | Attract loop, title, coin and start, select, join, continue, game over, ranking, ending |
| `episode-1.md` .. `episode-8.md` | One file per episode: story, scenes in order, enemies, bosses, items, port state |
| `mechanics-seen.md` | Flat list of every small rule met during research (seed for the design phase) |
| `NEW-PLAN.md` | Design phase first, then the milestones to build the real game |

## 0. How the facts were collected

- **[M] MAME observation.** I ran the original ROM in MAME with a small Lua probe (inputs by frame,
  RAM pokes, screenshots by frame). Frame numbers are 60 Hz frames counted from power on, unless said
  otherwise. I looked at the screenshots and describe them in my own words. The screenshots and the
  probe scripts stay in the scratchpad (not in the repo, no game art is committed). Scratchpad folder:
  `/private/tmp/claude-501/-Volumes-Shanto-Repos-Personal-Github-roms/9337dc4f-36ab-46a5-800b-fa02f2ab2b75/scratchpad/research/`
  (`probe.lua`, `run_probe.sh`, `attract.lua`, `digest.py`). How to use it: `NEW-PLAN.md`, section "Tools".
- **[R] Repo file.** `re/specs/*/SPEC.md`, `re/specs/enemy/stage_scripts.txt`, `plan/*.md`, port code.
  Addresses are 68000 ROM addresses. `G.$xxxx` is RAM `0xFF8000 + xxxx`.
- **[W] Web.** Links are listed at the end of this file. Many sites blocked the fetch tool (403 or 402:
  StrategyWiki, GameFAQs, TCRF, Arcade Otaku, Namu wiki, Capcom fandom, Giant Bomb). What worked:
  Wikipedia (en and ja), Hardcore Gaming 101, Arcade Quartermaster, arcade-history.com, and web search
  summaries. Web facts are used only where no ROM proof exists, and are marked as such.
- Story captions in the game are **paraphrased**, not copied.

Status words used for the port: **DONE** (ported and checked against MAME), **PARTIAL** (some of it
ported or not checked), **STUB** (a named empty function exists), **MISSING** (no code), **UNKNOWN**
(we do not yet understand the original).

## 1. What the port does today (checked by running it)

- `main.rs` always calls `start_stage(stage, chars)`. There is no boot, no attract, no title, no coin
  or start, no select screen. `flow_boot()` exists in `flow.rs` but nothing calls it.
- `--stage 0` (Episode 1) runs: terrain, Jack and the 3 other characters, thugs (classes 2, 10, 20),
  containers, items, weapons, HUD, camera and waves for the first areas. No opening scene: the player
  just stands at the left edge. Blade/Razor (class 14) is being ported by another agent right now; Vice T.
  (class 18) and every prop and scenery object of the stage are missing, so after the first thugs the
  stage is empty.
- `--stage 1` to `--stage 7` **crash at once** (tested with 1, 2, 3 and 7; 4 to 6 take the same code path) with a panic: the music table at 0xA582 was never
  extracted (`flow_area_music`, panic text "ROM text table byte 0xa586 was not extracted"). So the port
  cannot even show Episodes 2 to 8.
- Classes in the 8 stage scripts: pool A (enemies) 32 distinct classes, 5 ported; pool B (controllers,
  props, scenery, bosses' helpers) 59 distinct classes, 2 ported; pool E (items) all 4 used classes ported.
  Effects (pool C): 4 of about 40 classes ported, pool D none. Source: `re/specs/enemy/stage_scripts.txt`
  and `dino-game/src/game/{enemy,prop,effect}.rs` dispatch tables.
- No sound is played in the port (`audio.rs` is not wired). The original game is full of music and voices.
- The stage clear is a 120 frame wait (no tally), the next stage starts at once, and the ending is
  "game over". No intermission, no map card, no episode title.

## 2. The flow, in order

Format of each step: **Sees / hears**, **Rules**, **Original** (routine addresses), **Port**.

### A. Power on

1. **Boot info screen** [M f=0..100]. Black screen with the game name, the ROM date code and the
   region word in small blue text. Sound: not checked.
   Original: address unknown (probably 0xA69A boot task). Port: MISSING.
2. **Legal warning screen** [M f=100..300]. A paragraph on a black screen about where the game may be
   sold. Original: address unknown. Port: MISSING.

### B. The attract loop (repeats forever while nobody plays)

The attract task is 0x97C20. It keeps a state byte (G.$0FD0, steps of 2) and calls one of three routines
(table at 0x97C88). Measured in MAME (power on, no input):

| Step | Routine | What the player sees | Frames |
| --- | --- | --- | --- |
| Title | 0x9A3CE | Green title screen: big logo, small "tales based upon the comic" text, a blinking "INSERT COIN" (or "PUSH START" when a credit exists) and the Capcom logo. At frame 30 of the title the text is drawn; at frame 360 (0x168) the text is replaced by the licence text of the car maker and the comic. A 1 in 128 random chance starts one extra task at the title (0x6A73A, purpose unknown). | 600 |
| Profiles | 0x9A7E6 | Four character cards, one after the other: a big portrait on a coloured checker board (blue, red, green, orange), name, height, weight, a short job text, and a one word skill rating in red, plus a small animated sprite doing a move. Order: Jack, Hannah, Mustapha, Mess. Timeline tables at 0x9A8E4 (frames) and 0x9A908 (handlers). | 1200 |
| Ranking | 0x98330 (probable) | The "BRAVES" table: ten rows, rank, three letters, score, a face icon and a stage number, over a desert picture with a car driving across. The table is the default table (see `attract-and-front-end.md`). | about 530 |
| Demo play | 0x99E88 with 0x99F10 | The game plays itself with 3 characters (2 on a 2 player cabinet), recorded inputs (playback 0x9A146), the normal HUD and the "INSERT COIN" prompt in the playfield. Which episode: table 0x9A1E2 (6 entries, a counter G.$757A walks through them). | 1200 of play |
| Story slides | 0x988C0 | After the first demo: five picture and caption slides (villagers asking for help, heroes saying a human poacher group is behind the dinosaur attacks, a villain boasting he will crush them), fades between slides, then the ranking again. | about 1400 |
| Opening movie | 0x9BF50 | After the second demo: a text crawl about the "Great Cataclysm" 400 years ago, then a jungle scene with a man, a stalking dinosaur and a big roar. 0x5C8 = 1480 frames, event tables at 0x9C002 / 0x9C026, helper tasks 0x9C7C4 / 0x9C7F8. | 1480 |

State order seen in MAME: `[title, profiles, ranking]` then `[title, demo 1, story slides + ranking]`
then `[title, profiles, ranking]` then `[title, demo 2, ranking, opening movie]`, then the loop starts again.
State 0 and state 4 are the same routine (0x97D84). One full loop is about 13,500 frames (225 s).
Demo order (table 0x9A1E2, stage and area): Episode 2 area 0, Episode 3 area 0 (seen in MAME),
then Episode 8 area 1, Episode 5 area 0, Episode 6 area 1, Episode 1 area 1 (from the table, not yet seen).
During demos the sound is muted unless the "demo sound" setting is on (`re/specs/sound/SPEC.md`).
Port: attract MISSING (`flow_attract_task` is empty), demo input MISSING, story slides MISSING, opening movie MISSING.
Details: `attract-and-front-end.md`.

### C. Coin, credit, start

- Coin input adds credits (system task 0x9D048, coin logic 0x9D4BE, credits 0x9D386, start 0x9D196). The
  credit count is printed at the bottom of the title and select screens ("CREDIT n"). The settings
  (coins per credit up to 6 credits per coin, "2 coins to start, 1 to continue", free play, demo sound,
  continue on or off, a "timer level") are edited in the test menu and kept in the EEPROM. Port: MISSING
  (`flow_system_task` only copies the input byte). Source: [M], [R] `re/specs/flow/SPEC.md` section 6.
- With a credit, the title shows "PUSH START BUTTON" for player 1 and "INSERT COIN" for players 2 and 3.
  Pressing start (0x983EE) goes to the select screen. [M f=700 start, f=723 mode 4].

### D. Character select (game mode 4, 0xAD46)

- **Sees.** A group picture of the 4 heroes behind the panels. A "TIME" counter counts 10 down to 0 (one
  step per second). Each present player has a panel with the face, three stat bars (power, speed, skill),
  and a one word rating. Players who are not in have "INSERT COIN" or "JOIN-IN" blinking in their panel.
  The cursor order is Jack, Hannah, Mustapha, Mess; a taken character cannot be taken twice.
- **Rules.** At 0 the character under the cursor is taken. Other players can join in during the countdown
  with a coin and start. [M f=725..1059 with A pressed at f=900: the select ends 150 frames later;
  without a press it ended at f=1493 = 10 seconds].
- Original: 0xAD46, helpers 0xAF7C, 0xB06A, 0xB122, 0xB25E. Port: STUB (`flow_select_mode` starts the
  stage at once). The stat stars are display only (`re/specs/player/SPEC.md` section 7).

### E. Episode 1 start (no story card, no map card)

- **Sees.** Fade in on the flooded city from the roofs. The camera scrolls right by itself (camera mode
  0x11, 3 px per frame) with a small caption of the place and the year. A red pterodactyl flies across
  the sky. The player's hero walks in. A gang runs in; a villain's voice shows as a caption (he is tired of
  the heroes interfering and sends his men). The hero knocks several of them away in a scripted fight.
  Then the HUD slides in from the right and the player gets control. [M f=1060..~1700].
- **Original.** Intro task 0xB9BE (states 0..2 are the cards, state 4 is this scene), scripted player
  control 0x1DA10 / 0x1DB0C / 0x1DB5E (sets G.$4E3 when the walk-in is done), opening scene objects: pool A
  classes 54, 55, 56, 57 and pool B class 74 (master), 66 (caption banners), birds 49, 50, 73.
- **Port.** PARTIAL: classes 54, 55, 49, 50, 73 are ported; 56, 57, 74, 66 are not; the scripted
  control is not ported, so the HUD slides in at once (`flow_intro_task`, TEMP) and the scene is skipped.

### F. Every episode, from the second on: the link scenes

Order seen in MAME (stage clear poked at f=1300): clear tally, map panel, story panel, episode card.

1. **Stage clear tally** (task 0xB97E, about 740 frames) [M f=1301..2045]. The hero's portrait and
   panels count: "vitality" (a word like PERFECT for full health), enemies defeated, final blow bonus
   (+20000), best player bonus (+10000, only with 2 or 3 players), then a big "TOTAL" counts down to 0
   while the points go into the score. The background is the finished level with the last enemies
   still lying there. Port: STUB (120 frame wait).
2. **Map panel.** A framed map of the world slides in at the top of the screen and shows the next place
   (a name plate under the map). Original: stage intro task 0xB9BE states 0..2. Port: MISSING.
3. **Story panel** (game mode 8, routine 0xD898, entries 0xD8E8, 0xD97E, 0xDAB4, 0xDBB2, 0xDDCC, 0xDE6C,
   0xDF1E for the episodes 2 to 8; 250 to 440 frames). A black screen with one small picture and two or
   three lines of text in white. A friend or the villain talks: a warning about the next boss, a voice on
   the radio asking for help, a surprised hero who sees mutant experiments, the madman inviting them to
   come. Port: MISSING (`flow_stage_start` is called at once, comment "0xD898 not ported").
4. **Episode card.** The next level is shown, the map panel stays on top, and below it "EPISODE n" and
   the title (typed letter by letter) appear for about 4 seconds while the opening scene of the level
   already runs (a car arrives, the heroes jump out, enemies wait). Port: MISSING.

Titles seen in game: 2 The Swamp Forest, 3 Hell Road, 4 Jack's Garage, 5 Village of Flame,
6 Jungle and Mine, 7 The Vault, 8 The Under World. (Episode 1 has no card.)

### G. During an episode (the rules that make it feel like the original)

- **Areas and camera.** Each episode has 3 or 4 areas. The camera follows the leading player (push
  scrolling) and stops at "lock" points where a wave of enemies attacks. The camera is released when the
  wave is small enough (the GO arrow appears) or the area ends with a fade or an automatic walk.
  Source: `re/specs/enemy/SPEC.md` sections 8.2 and 8.3, `re/specs/stage/SPEC.md` sections 5 and 6.
  Port: camera DONE (all modes except the Episode 3 controller and the elevator background), waves PARTIAL (waves run, "walk to the next area" is a TEMP shortcut
  because 0x1D95E / 0x1DAC4 are not ported).
- **Bounds.** The camera never goes back left. Players are clamped to the screen (0x4FCE). Objects that
  leave the screen are freed (0x173C). The vertical scenes (Episode 1 area 2, Episode 7 area 1) use Y.
- **Waves and the 2 minute rule.** A wave controller (pool B class 15) spawns groups when the number of
  enemies left is under a limit that depends on the player count. If a group takes more than 7200 frames
  the next group comes anyway, and at the end every enemy still alive dies with no score. Port: DONE.
- **GO arrow.** When a wave ends the camera unlocks and a red "GO!" arrow bursts at the right edge
  [M]. If nobody moves on for 600 frames a hint blinks and a sound plays (pool B class 20, sound 0x3E).
  Port: the unlock works; the arrow and the hint are MISSING (class 20 not ported, the arrow object is
  not identified yet).
- **Time limit.** Every area starts with a time budget (a table at 0x1033E: from 1:20 up to 4:00).
  Scripts reset it (for example "3:50" before a boss) or add to it. One second is 51 frames. A clock
  object (pool B class 31, 0x81046, placed in every area script) draws "m:ss" at the top when 3:00 is left:
  for 5 seconds at each full and half minute, red when 1:00 is left, and all the time with a beep every
  second in the last 15 seconds [M: red digits seen at 0:13]. At 0:00 the words "TIME" and "OVER" fly in
  (effect class 0x70, 0x96450) and every living player's HP is set to 0 (0x103FE) [M]; the next life
  starts and the time is topped up (0x10482). In the car section the time over ends the drive instead.
  Port: the countdown and the HP kill are DONE (`rank.rs`), the clock display and the TIME OVER words are MISSING.
- **Rank and difficulty.** Hidden rank grows with time and falls when a player dies. Port: DONE (`rank.rs`).
- **Items.** Drums, barrels, tires and suits of armor break and drop food, weapons, ammo and treasure.
  Food heals, treasure gives points. Port: DONE for stage 1 data (pool E).
- **Dinosaurs.** A neutral dinosaur can sleep, wander or attack; hit once it turns hostile; it turns
  calm again at 0 HP. Zeke (pteranodon) flies in and hits. Port: MISSING (classes 24, 29, 38, 82 not ported).
- **Joining and dying.** Absent players show a blinking prompt in the HUD row ("JOIN-IN" with a credit,
  "INSERT COIN" without) [M]. A dead player with lives left drops back in with blinking invulnerability
  (360 frames) and a bubble of speech. Port: lives and respawn DONE; the prompts are STUB (task 0x6C4A).

### H. The boss and the end of an episode

- The boss has its own intro, a big HP bar in the HUD (more than one bar), helpers, and a death scene:
  the game runs in slow motion (task period 4) until the boss is down, the heroes walk off by script, the
  clear tally runs (F1). Boss defeat routine 0xACEE, kill-all event G.$50E = 1, rank speed +1.
  Port: the flow part (0xACEE and the G.$4D1 switch) DONE, only Vice T. exists in the specs, no boss is ported.
- Episode 8: after the last boss the ending (mode 0x1C) starts (section J).

### I. Continue, game over (mode 0x10, 0x14, 0x18)

- All players dead and nobody can continue: the continue screen [M f=1557..2856]. A big villain with a
  gun in your face on a red background. A countdown from 20 to 0, about 62 frames per number. A speech
  bubble changes every 5 numbers (luck ran out; how dare you; I will feed you to the Shivat; eat lead).
  Insert a coin and press start: the screen turns into a quick character select over the level
  background (10 second counter, the same player can pick another hero), then the player drops in with
  a bazooka and a speech bubble [M]. If the counter reaches 0: red flash, blood splashes, "GAME OVER"
  for about 480 frames, then the ranking screen and the attract loop. Original: 0xE9E6 (continue),
  task 0x6C4A (per player states 0..0xC), 0xAC06 / 0xAC26 (game over). Port: STUB (goes to game over at once).
- Score is not reset on continue; rank drops and a 1200 frame calm period follows (`re/specs/enemy/SPEC.md` 6.4).

### J. The ending (game mode 0x1C, routine 0xA69B8) [M f=1300..6664, 5365 frames]

1. The base self destructs: a short scene in the lab corridor with heroes running, a shout, chains of
   explosions, a caption urging to hurry.
2. A still panel: two heroes calling the names of the two who are still inside.
3. Desert at sunset: two heroes walk right while the staff roll scrolls (programmers, composers,
   special thanks, a long list of names). The staff strings are at 0x8B7EA.
4. A Cadillac arrives with the two other heroes. Short dialogue (the heroes are alright, they barely made
   it; the madman almost made his plans real; nature always balances itself; let's go home).
5. "Thank you for playing", "Produced by Capcom", the car drives off.
6. The ranking screen, then the attract loop (modes 0x14 / 0x18 as after a game over).
- Which two heroes walk may depend on the characters in play (not checked). Port: MISSING.

### K. High score entry

- No name entry screen was seen. After a game over with a score of 500000 (poked) the ranking table was
  unchanged [M]. The code at 0x16230 writes a player's score, a tag, the character and the stage into a
  table entry and sorts the table (0x16260), so the table is live in RAM but who calls it is unknown.
  Port: PARTIAL (`hud.rs` keeps the table and the rank text "1ST".."10TH" in the HUD; no screen).

## 3. Per-episode summary (details in `episode-N.md`)

| Ep | Title | Areas | Boss | Special | Port today |
| --- | --- | --- | --- | --- | --- |
| 1 | City in the Sea (web), no card | 3 (roofs, hall, street) | Vice Terhune (class 18) | opening scene, vertical drop | runs, no props, no Vice |
| 2 | The Swamp Forest | 3 | Lash Terhune (mid), the Butcher (class 61 via class 5) | sleeping dinosaurs, Cadillac arrival | crashes at start |
| 3 | Hell Road | 1 very long area (60 screens) | Hogg (class 31) | Cadillac drive, row scroll, radio | crashes at start |
| 4 | Jack's Garage | 3 | Slice (class 25) | garage props, many drums | crashes at start |
| 5 | Village of Flame | 2 + sub area | Morgan then Morgue (classes 36, 40, via 35, 45) | fire, rocks, bridge, dinosaurs | crashes at start |
| 6 | Jungle and Mine | 3 | Tyrog, three times (classes 4, 72, 91) | mine lamps, giant dinosaur legs, rain | crashes at start |
| 7 | The Vault | 4 (outside, elevator shaft, library, lab) | two or three Slisaurs (class 71) | 56 screen elevator shaft | crashes at start |
| 8 | The Under World | 4 (lab, sewers, corridor, cave) | Dr. Fessenden, 2 forms (classes 84, 86 to 89, 94) | tanks break open, flame jets, rolling barrels | crashes at start |

## 4. Gap list, sorted by how much it changes what the user sees

1. **No front end at all** (boot, attract, title, profiles, ranking, demo, story slides, opening movie).
   The first 5 minutes of the original are missing. (`attract-and-front-end.md`)
2. **Only Episode 1 starts, and it has almost no content.** Episodes 2 to 8 crash; Episode 1 has 2 of 10
   pool B classes and 5 of 7 pool A classes working (6 of 7 when class 14, in progress, lands), and no boss.
3. **No story**: no opening scene, no scripted walks, no intermission panels, no map card, no episode card,
   no tally, no ending, no staff roll.
4. **No sound** (music per area, voices, hit sounds, clock beep).
5. **No scenery and props**: 57 of 59 pool B classes (lights, banners, hazards, gates, falling rocks,
   boss helpers) and 36 effect classes.
6. **Bosses and dinosaurs**: none of the 9 bosses (Vice, Butcher, Hogg, Slice, Morgan, Morgue, Tyrog,
   Slisaur, Fessenden), none of the dinosaurs (Shivat, Rock Hopper, Mack, Zeke, brontosaurus).
7. **Coin, credit, start, select, join, continue, game over** and the settings menu.
8. **Time clock and TIME OVER** display, GO arrow, GO hint.
9. **Special modes**: Cadillac drive and row scroll (Episode 3), elevator shaft (Episode 7), screen shake.
10. **Small rules**: weapon HUD display, extra life pose, tag-team move, friendly fire, boss weapons that
    players can take, flames, water effects (list in `mechanics-seen.md`).
11. **Exactness passes** against MAME for everything above.

## 5. Web sources used

- Wikipedia: https://en.wikipedia.org/wiki/Cadillacs_and_Dinosaurs_(video_game)
- Japanese Wikipedia (stage list, enraged dinosaurs, regional names): https://ja.wikipedia.org/wiki/キャディラックス_恐竜新世紀
- Hardcore Gaming 101 (stages, enemies, version notes): https://www.hardcoregaming101.net/cadillacs-dinosaurs/
- Arcade Quartermaster (areas, bosses, items, characters, enemies):
  https://www.arcadequartermaster.com/cad_bosses.html , `cad_items.html`, `cad_enemies.html`, `cad_characters.html`
- arcade-history.com (tips, versions): https://www.arcade-history.com/game/370/
- Search result summaries about the boss list and enemy types (Gamia archive wiki, TV Tropes, Namu wiki): not fetched in full.
