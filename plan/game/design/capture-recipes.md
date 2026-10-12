# Capture recipes: the probe tool and a tested recipe for every screen

Written 2026-10-12 (task T0 of `plan/game/NEW-PLAN.md`). This is the manual of the probe tool in
`re/harness/probe/` and the book of recipes that put the original game (MAME, set `dino`) on any screen.
Design agents use it to look at the original and to measure it.

## 0. Rules (read first)

1. **Batch your probes. One MAME process for many probes.** A MAME start is slow for the user's desktop
   (Dock icon, focus) and costs seconds. Put every probe you need in ONE input file as `segment NAME`
   blocks (section 3) and start `probe.sh` once. Do not run `probe.sh` once per question. The recipes of this
   file run as one process too (`recipes.py`).
2. **Headless only.** `probe.sh` starts the command line `mame` with `-video none -sound none` and
   `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy`. With the dummy drivers macOS never registers MAME as an
   application (no Dock icon, no focus change). Never start the MAME4mac app or any program with a window,
   and keep these two variables if you write your own MAME command.
3. **No ROM data in the repo.** Logs, screenshots and states go to `re/out/probe/` (git-ignored). Do not
   copy screenshots, `peek` dumps or `events` output into documents. Write findings in your own words and
   numbers (speeds, timers, damage), with the recipe or probe lines that produced them.
4. **MAME is the judge.** What the probe shows is the original code running in MAME. The probe changes the
   game only where it says so (pokes, cheats, `jump`).

## 1. Use in five lines

```sh
# 1. write the probes (one segment per question) into a text file, for example re/out/my.txt:
#      segment zeke            (see sections 3 to 5 for the lines)
#      jump 5 0
#      5 spawn A 82 var=0 w5c=0x7fff x=+0x100
#      watch 1 A:82 x y z st
#      watch 1 P1 hp
#      frames 400
#      snaps every:40:0:400
# 2. run all segments in one MAME process (about 20x real time):
PR_INPUT=re/out/my.txt re/harness/probe/probe.sh mybatch
# 3. read re/out/probe/mybatch/zeke/log.txt and look at re/out/probe/mybatch/zeke/sheet.png (contact sheet)
# 4. numbers out of a watch: re/.venv/bin/python -I re/harness/probe/series.py re/out/probe/mybatch/zeke/log.txt A#20 x y --delta
# 5. the recipes of this file: re/.venv/bin/python -I re/harness/probe/recipes.py --list | NAME ... | --prefix area- | --all
```

Files of the tool (all in `re/harness/probe/`): `probe.lua` (runs inside MAME), `probe.sh` (starts MAME),
`sheet.py` (contact sheets), `series.py` (watch lines to a table), `recipes.py` (runs the recipes of this file).
Run Python with `re/.venv/bin/python -I`.

## 2. How it works

* **Time base.** Without `jump` the frame numbers of the input file count video frames from power on. With a
  `jump` line, `t = 0` is the first frame after the jump (the player can act) and every frame number of the
  segment counts from there. The log shows `t` (frames after the jump), `f` (frames from power on) and `gf`
  (game frame counter G.$0000, it stops during a slow-down frame: use `gf` to count game frames).
* **Segments.** A line `segment NAME` starts a new probe. Every segment starts from the power-on state of
  the same MAME process (the probe saves the state of the first frame and loads it again), so segments are
  independent and repeatable (checked: the same segment twice gives the same frame numbers and positions).
  Lines before the first `segment` line are added to every segment. A file without `segment` lines is one
  probe, written to `re/out/probe/NAME/` itself.
* **Inputs.** Key lines act like the harness scripts (`re/harness/README.md`): `a-b:KEYS` with `U D L R`
  (stick), `A` attack, `J` jump, `S` start, `C` coin; the digits `1 2 3` choose the player for the letters
  that follow (`10-17:2C` = coin of player 2). One line may hold several segments separated by commas.
* **The jump.** `jump EP AREA [CAMX] [char=NAMES] [at=start]` boots the game (coin 600, start 700, choose
  at 900), enters episode EP with the story-panel poke (stage, sub mode 0, mode 8), moves through the
  areas with the game's own end-of-area camera events (it reads the event table of the ROM at run time)
  and, if CAMX is given, walks the camera to CAMX one map column per frame (the maps stream in one column
  per frame, a faster move leaves holes) while the spawn script skips the records before CAMX (no enemies,
  waves or commands before it; props are kept from CAMX - 0x400). During the jump the players are
  invulnerable; this is switched off at the end. `EP` is 1 to 8, `AREA` is the camera area number G.$786
  (0-based). `char=jack,hannah,mess` sets the heroes of players 1 to 3 (names `jack mustapha hannah mess`
  or ids 0 to 3; the cursor moves are searched, the heroes must differ). `at=start` stops at the first frame
  of the level (before the walk-in scene and the episode card) instead of at the first frame of control.
  Time to reach (emulated frames from power on, from the logs of the last full run): between 1726 (Episode 1
  area 0) and 3155 (Episode 4 area 2; its opening scene is the longest) for all 23 areas, so under 60 seconds
  (3600 frames) from one line. A CAMX adds one frame per 32 pixels (Episode 3 up to X 0x7FE0: about 1050 more,
  about 3350 in all). The table is in group G.
* **Limits of the jump** are in section 8.

## 3. The input language

One command per line, `[FRAME] COMMAND arguments`. A number in front is the frame (t) when it runs; no number
means frame 0. `#` starts a comment, `;` also ends a line. Numbers are decimal or `0x` hex. Several lines for
one frame run in file order.

| Command | What it does |
| --- | --- |
| `segment NAME` | start a new probe (own log and screenshots in `NAME/`). |
| `frames N` | run N frames after the jump (default 600, or the `PR_FRAMES` variable). |
| `snaps LIST` | screenshots at these frames: `snaps 0,30,59` or `snaps every:STEP:FROM:TO` (default: the `PR_SNAPS` variable). |
| `jump EP AREA [CAMX] [char=..] [at=start]` | see section 2. One per segment. |
| `snap` | one screenshot now (frame of the line). |
| `poke ADDR SIZE VALUE` | write RAM now. ADDR is a full address (`0xFF84E8`; `G.$xxxx` = `0xFF8000 + xxxx`), SIZE 1, 2 or 4. The old form `FRAME HEXADDR SIZE VALUE` still works. |
| `hold ADDR SIZE VALUE` / `release ADDR` | write every frame until released. |
| `peek ADDR [WORDS]` | log words of RAM or ROM (for your own reading; do not copy them into documents). |
| `events` | log the camera events of the current episode. |
| `spawn POOL CLASS [key=value ..]` | create an object like the stage script does (below). |
| `objset SELECTOR FIELD VALUE` | write a field of the selected objects, for example `200 objset A:18 hp 1`. |
| `hp V [P1..P3]` | player vitality (and the bar) to V (0 kills; 100 is full, more shows extra bars). |
| `lives N [P]`, `score N [P]` | extra lives, score (decimal digits). |
| `god on|off [P]` | invulnerable player: the probe keeps the hit-test bit and the invulnerability timer set every frame. Enemies still swing and miss. |
| `time M:SS` | the stage clock (BCD word G.$4E8). One second is 51 frames. `time 0:05` gives TIME OVER in about 255 frames. |
| `killall` | kill-all flag 2 (no score, like a wave timeout) for 7 frames: every enemy that runs its kill-all check dies. For a permanent kill use `hold 0xFF850E 1 2`. |
| `rank R`, `level L`, `aggr A`, and `... free` | freeze the rank clock and set rank (0..31), level (0..31, the difficulty index of most tables) or aggression (0..7). `rank free` releases them. |
| `watch STEP SELECTOR [FIELDS..]` / `unwatch` | log object fields every STEP frames (section 4). |
| `log TEXT`, `stop` | write a note into the log; end the segment now. |

`spawn POOL CLASS` uses the free-list allocators of the game (copied from 0x483C pool A, 0x4860 B, 0x4818 E,
0x4884 C, 0x48A8 D): POOL is `A` (enemies, bosses, dinosaurs), `B` (controllers, props, scenery, boss
starters), `E` (items, containers, projectiles), `C` (effects) or `D`. CLASS is the class number (the
probe writes `class * 4` like the scripts). Keys: `var=` (+$26, the variant word of the script),
`x=+0x120` (relative to the camera; the default) or `ax=0x400` (absolute), `z=` (default: the depth of
player 1), `y=` (height), `w5c=`, `b5b=`, `drop=` (drop row +$96), `n=3 dx=0x30` (several, spaced in X),
`cw=` (raw class word). The object starts at mode 0 and runs its init, like a script spawn. The classes
and variants of the real spawns are in `re/specs/enemy/stage_scripts.txt` (record fields X, Z, var, +5c,
+5b) and `re/specs/enemy/classmap.txt`. Example: Zeke as the script spawns it needs `w5c` (the camera X
it leaves at); `w5c=0x7fff` keeps it forever.

Other probe text is in the head of `probe.lua`. Environment variables of `probe.sh` (all optional):
`PR_INPUT` (file), `PR_POKES` (second file), `PR_LINES` (lines inline, `;` separated), `PR_FRAMES`, `PR_SNAPS`,
`PR_WATCH`, `PR_SHEET=0` (no contact sheets), `PR_TIMEOUT` (seconds, default 900).

## 4. Watching objects

`watch STEP SELECTOR [FIELD ..]` writes one `W` line per object every STEP frames from the frame of the line.

* Selectors: `A#3` pool A slot 3; `A:82` every live pool A object of class 82; `A` all live objects of pool A;
  the same for `B`, `C`, `D`, `E`; `P1` `P2` `P3` the players (`P` all present); `G` game globals.
* Fields (pool objects and players): `x y z` (position, integer part; y is the height, z the depth: a larger
  z is higher on the screen), `xf` (X as 16.16 hex), `vx vy vz`, `ax ay az`, `cls` (class), `var`, `st` (the
  state long +$04..+$07, hex), `r4 r5 ai ai2` (its bytes), `dir face`, `anim` (frame record pointer), `ftimer`,
  `fcls`, `air floor`, `atk hs serial` (attack id, hit stop, attack serial), `hp hpbar hpprev poise`, `target`,
  `t80`, `w5c b5b drop`; players only: `char lives held inv rinv score input`. Raw fields:
  `+0x6C` (word), `+0xA3.b`, `+0x28.l`, `+0x6C.sw` (suffix b w l, sb sw sl signed, hb hw hl hex) and absolute
  `@0xFF84E8.w`. Names and layout: `re/specs/enemy/SPEC.md` section 2, `re/specs/player/SPEC.md` section 1.
* Globals (`G`): `time` (BCD) `camx camy cammode go lock alive players rank level aggr frame killall car scripted
  mode sub stage area ingame nextev evidx`.
* Log line: `t=45 f=2531 gf=1289 W A#20 x=205 y=277 z=582 st=02000200`. By class the label is the slot, so
  the slot of a spawned object is also in its `SPAWN` line. `series.py LOG A#20 x y --delta` turns the samples
  of one label into a table with the change per step; it skips the repeated rows of slow-down frames
  (`--all` keeps them).

## 5. Output

`re/out/probe/NAME/` for a batch (and `re/out/probe/NAME/SEGMENT/` for each segment): `log.txt`, `snap/0000.png ..`
(384 x 224, the game picture), `sheet.png` (contact sheet, `sheet_1.png ..` when more than 30 pictures; the label
of every picture is `#number t=.. f=..` and the game state `m0c s00 E2 a1` = mode, sub mode, episode, area),
`segments.txt` (end line of each segment), `mame.log`. Log tags: `S` game state changes (mode G.$4D0, sub mode
G.$4D1, stage G.$4D9 = episode - 1, area G.$786, in game flag), `J` jump steps (`done` ends it and prints the
players), `X` a command ran, `SPAWN`, `W` watch, `SNAP`, `NOTE`, `PEEK`, `EVENT`, `ERR`, `END`. Game modes:
0 start setup, 4 select, 8 story panel, 0xC in the level, 0x10 continue, 0x14 and 0x18 game over, 0x1C ending.
Bigger contact sheets or overviews: `sheet.py DIR [--cols 5 --width 256 --per 30 --from N --to N]` and
`sheet.py DIR1 DIR2 .. --pick -1 --out overview` (one picture per directory). Look at the sheets with an image
viewer (the Read tool shows PNG files).

## 6. Recipes

Run any of them: `re/.venv/bin/python -I re/harness/probe/recipes.py NAME [NAME ..]` (one MAME process; output in
`re/out/probe/recipes-part/NAME/`; `--all` writes `re/out/probe/recipes/NAME/`). The text of each block below is what runs: `recipes.py` reads it from this file.
**Status** is `tested` only when I ran the recipe and looked at its contact sheet myself (2026-10-12); otherwise it
says why not. Frame numbers are from power on for the `fe-` recipes (no jump) and from the jump for all others.


### A. Power on

#### `fe-boot`  (tested)

Black screen. At about frame 90 small blue text appears (game name, the date code 930201 and a region mark), at about 200 it changes to the legal warning (where the game may be sold) on black. Pictures at 10, 60, 100, 130, 170, 250, 310.

```recipe fe-boot
frames 330
snaps 10,60,100,130,170,250,310
```


### B. The attract loop (nobody plays)

#### `fe-title`  (tested)

The green title: logo, the line about the comic, the Capcom sign. The title starts at about frame 300 and lasts 600 frames. INSERT COIN blinks (about 30 frames on, 30 off; picture 0 is an on frame); at about 360 frames into the title the small text is replaced by the licence text of the car maker and the copyright lines (pictures 3 and 4).

```recipe fe-title
frames 900
snaps 340,420,560,700,860
```

#### `fe-profiles`  (tested)

The four hero cards, 300 frames each, in the order Jack, Hannah, Mustapha, Mess: a big portrait on a coloured checker board (blue, red, green, orange), name, height, weight, job text, a red skill word and a small animated hero doing a move on the right; each card fades to grey before the next. Two pictures per card.

```recipe fe-profiles
frames 2400
snaps 1000,1150,1300,1450,1600,1750,1900,2050,2200
```

#### `fe-ranking`  (tested)

The ranking table (title BRAVES): ten rows with rank, three letters, score, a face icon and a stage number, over the desert with the Cadillac driving by. The default table (scores 100000 down to 10000).

```recipe fe-ranking
frames 2800
snaps 2400,2500,2600,2700
```

#### `fe-demo-ep2`  (tested)

Demo play 1 (frame 3350 on): after the title the game plays itself in Episode 2 area 0: a team of three heroes (Jack, Mustapha, Mess, with the HUD and the INSERT COIN prompt in the playfield) gets out of the Cadillac at the edge of the forest, fights poachers, carries a barrel and passes a dinosaur. 1200 frames of play, then the story slides.

```recipe fe-demo-ep2
frames 5000
snaps every:200:3340:5000
```

#### `fe-slides`  (tested)

The five story slides (pictures 0 to 4) and the ranking again (5, 6): the villagers' cry for help with dinosaurs attacking, a hero blaming the black marketeer, a poacher telling a helper to hurry with the skinning, a heroine at a window who does not understand why, and a close up of the villain who says they keep upsetting the balance and he will crush them. Fades between slides.

```recipe fe-slides
frames 6900
snaps 4900,5200,5500,5900,6300,6600,6800
```

#### `fe-demo-ep3`  (tested)

Demo play 2 (frame 9924 on): Episode 3 area 0 with the caption DESERT OF DEATH, three heroes (Hannah, Mustapha, Mess) fight thugs and bikers on and around the moving Cadillac. Then the ranking.

```recipe fe-demo-ep3
frames 11300
snaps every:200:9900:11300
```

#### `fe-movie`  (tested)

The opening movie (frame 11800 to 13400): a black screen with a line about 400 years ago, a text crawl about the creation of new life and the Great Cataclysm (key words in red), CAPCOM presents, then a jungle strip with a man ('that one is huge'), a second caption about past and future colliding, dinosaurs, a big head that roars, and the logo.

```recipe fe-movie
frames 13500
snaps 11800,12000,12300,12600,12800,13000,13200,13350
```

#### `fe-demo-row-0`  (tested)

Demo table row: poking the word at 0xFFF57A (G.$757A, the demo counter) to 0 at frame 3000 makes the demo that starts at frame 3350 use Episode 2 area 0 (the normal first demo). One picture at frame 3450. The counter is used after an increment, so value N gives demo number (N+1) mod 6 of the table (stage, area): Ep 2 a0, Ep 3 a0, Ep 8 a1, Ep 5 a0, Ep 6 a1, Ep 1 a1.

```recipe fe-demo-row-0
frames 3500
snaps 3450
3000 poke 0xFFF57A 2 0
3000 peek 0xFFF57A 1
```

#### `fe-demo-row-1`  (tested)

Demo table row: poking the word at 0xFFF57A (G.$757A, the demo counter) to 1 at frame 3000 makes the demo that starts at frame 3350 use Episode 3 area 0 (the normal second demo). One picture at frame 3450. The counter is used after an increment, so value N gives demo number (N+1) mod 6 of the table (stage, area): Ep 2 a0, Ep 3 a0, Ep 8 a1, Ep 5 a0, Ep 6 a1, Ep 1 a1.

```recipe fe-demo-row-1
frames 3500
snaps 3450
3000 poke 0xFFF57A 2 1
3000 peek 0xFFF57A 1
```

#### `fe-demo-row-2`  (tested)

Demo table row: poking the word at 0xFFF57A (G.$757A, the demo counter) to 2 at frame 3000 makes the demo that starts at frame 3350 use Episode 8 area 1 (underground). One picture at frame 3450. The counter is used after an increment, so value N gives demo number (N+1) mod 6 of the table (stage, area): Ep 2 a0, Ep 3 a0, Ep 8 a1, Ep 5 a0, Ep 6 a1, Ep 1 a1.

```recipe fe-demo-row-2
frames 3500
snaps 3450
3000 poke 0xFFF57A 2 2
3000 peek 0xFFF57A 1
```

#### `fe-demo-row-3`  (tested)

Demo table row: poking the word at 0xFFF57A (G.$757A, the demo counter) to 3 at frame 3000 makes the demo that starts at frame 3350 use Episode 5 area 0 (the savanna village). One picture at frame 3450. The counter is used after an increment, so value N gives demo number (N+1) mod 6 of the table (stage, area): Ep 2 a0, Ep 3 a0, Ep 8 a1, Ep 5 a0, Ep 6 a1, Ep 1 a1.

```recipe fe-demo-row-3
frames 3500
snaps 3450
3000 poke 0xFFF57A 2 3
3000 peek 0xFFF57A 1
```

#### `fe-demo-row-4`  (tested)

Demo table row: poking the word at 0xFFF57A (G.$757A, the demo counter) to 4 at frame 3000 makes the demo that starts at frame 3350 use Episode 6 area 1 (jungle). One picture at frame 3450. The counter is used after an increment, so value N gives demo number (N+1) mod 6 of the table (stage, area): Ep 2 a0, Ep 3 a0, Ep 8 a1, Ep 5 a0, Ep 6 a1, Ep 1 a1.

```recipe fe-demo-row-4
frames 3500
snaps 3450
3000 poke 0xFFF57A 2 4
3000 peek 0xFFF57A 1
```

#### `fe-demo-row-5`  (tested)

Demo table row: poking the word at 0xFFF57A (G.$757A, the demo counter) to 5 at frame 3000 makes the demo that starts at frame 3350 use Episode 1 area 1 (the hall). One picture at frame 3450. The counter is used after an increment, so value N gives demo number (N+1) mod 6 of the table (stage, area): Ep 2 a0, Ep 3 a0, Ep 8 a1, Ep 5 a0, Ep 6 a1, Ep 1 a1.

```recipe fe-demo-row-5
frames 3500
snaps 3450
3000 poke 0xFFF57A 2 5
3000 peek 0xFFF57A 1
```


### C. Coin, credit, start

#### `fe-title-credit`  (tested)

Coin 1 at frame 450 (the title is running): the bottom changes to PUSH START BUTTON for player 1, INSERT COIN for players 2 and 3 and CREDIT 1 / 0 / 0. Picture 0 (frame 420) is before the coin.

```recipe fe-title-credit
frames 800
snaps 420,520,640,760
450-457:C
```


### D. Character select

#### `fe-select-1p`  (tested)

Coin at 600, start at 700: the select screen (mode 4, from frame 723): the group picture of the four heroes, TIME 10 counting down (one number per second), the panel of player 1 with the face, power / speed / skill diamonds and a rating text, and 2P / 3P panels that blink JOIN-IN. Nobody presses a key: at TIME 0 the hero under the cursor (Jack) is taken and the game starts (pictures 5 to 7, the Episode 1 opening with its caption).

```recipe fe-select-1p
frames 1700
snaps 740,900,1100,1300,1480,1500,1560,1650
600-607:C
700-705:S
```

#### `fe-select-pick`  (tested)

Player 1 moves the cursor once to the right (frame 800) and chooses at 900: Hannah's panel, then the picture darkens and the level starts at frame 1060. 2P / 3P show JOIN-IN and then INSERT COIN.

```recipe fe-select-pick
frames 1100
snaps 900,930,960,1000,1040,1070
600-607:C
700-705:S
800-803:R
900-905:A
```

#### `fe-select-3p`  (tested)

Three players: coins at 600, 640, 680, starts at 700, 740, 780. The labels 1P, 2P, 3P stand on the heroes (the cursors start on Jack, Hannah, Mustapha). A cursor jumps over the cursors of the others: the first right press of player 1 goes from Jack to Mess (picture 1 to 3), then the other players move. The panels show each hero's stats and rating text.

```recipe fe-select-3p
frames 1100
snaps 745,790,830,870,910,950,990,1030
600-607:C
640-647:2C
680-687:3C
700-705:S
740-745:2S
780-785:3S
830-833:R
870-873:R
910-913:2R
950-953:2R
990-993:3R
```


### E. Start of every episode (opening scenes and episode card)

#### `ep1-opening`  (tested)

Episode 1 from the first frame of the level: black fade in, the roofs of the flooded city with a place caption, a red pterodactyl crossing the sky, Jack walks in, a gang of thugs runs in with a boss figure and captions in which the villain says he is tired of the interference and sends his men; the scripted fight; at the end the HUD slides in and the player gets control (picture 17). About 700 frames.

```recipe ep1-opening
jump 1 0 at=start
frames 700
snaps every:40:0:700
```

#### `ep2-start`  (tested)

From the first frame of the level (jump at=start): the map panel with EPISODE 2 and the title typed letter by letter (THE SWAMP FOREST), the Cadillac arrives at the left edge, the hero jumps out, the HUD slides in. About 600 frames; picture 0 is the black fade in.

```recipe ep2-start
jump 2 0 at=start
frames 600
snaps every:50:0:600
```

#### `ep3-start`  (tested)

From the first frame of the level (jump at=start): the map panel with EPISODE 3 and HELL ROAD while thugs wait in the open desert; the HUD slides in; the first fights. About 600 frames; picture 0 is the black fade in.

```recipe ep3-start
jump 3 0 at=start
frames 600
snaps every:50:0:600
```

#### `ep4-start`  (tested)

From the first frame of the level (jump at=start): the map panel with EPISODE 4 and JACK'S GARAGE over the garage yard full of tyres and barrels; Jack's friends give orders in two captions (block the entrance with those, hurry with the barrels); an explosion in the last picture. About 600 frames; picture 0 is the black fade in.

```recipe ep4-start
jump 4 0 at=start
frames 600
snaps every:50:0:600
```

#### `ep5-start`  (tested)

From the first frame of the level (jump at=start): the map panel with EPISODE 5 and VILLAGE OF FLAME on the savanna, a burning group of poachers at the right; the HUD slides in. About 600 frames; picture 0 is the black fade in.

```recipe ep5-start
jump 5 0 at=start
frames 600
snaps every:50:0:600
```

#### `ep6-start`  (tested)

From the first frame of the level (jump at=start): the map panel with EPISODE 6 and JUNGLE AND MINE in front of the wooden mine wall; the hero walks in. About 600 frames; picture 0 is the black fade in.

```recipe ep6-start
jump 6 0 at=start
frames 600
snaps every:50:0:600
```

#### `ep7-start`  (tested)

From the first frame of the level (jump at=start): the map panel with EPISODE 7 and THE VAULT on the roof; thugs crouch on the roof as the HUD slides in. About 600 frames; picture 0 is the black fade in.

```recipe ep7-start
jump 7 0 at=start
frames 600
snaps every:50:0:600
```

#### `ep8-start`  (tested)

From the first frame of the level (jump at=start): the map panel with EPISODE 8 and THE UNDER WORLD in the lab with the specimen tubes (numbered 01 to 03); the hero walks in. About 600 frames; picture 0 is the black fade in.

```recipe ep8-start
jump 8 0 at=start
frames 600
snaps every:50:0:600
```


### F. Link scenes between episodes (tally, map panel, story panel, episode card)

#### `link-ep2`  (tested)

After `jump 1 0` the probe sets stage = 1, sub mode 0, mode 8 (0xD898). Pictures: the level (0), the story panel (panel: a bald man (the Butcher) hunts in the northern woods, do not mess with him (306 frames of panel)), then the map panel with EPISODE 2 and its title, and the start of the level.

```recipe link-ep2
jump 1 0
frames 900
snaps every:60:0:900
0 poke 0xFF84D9 1 1
0 poke 0xFF84D1 1 0
0 poke 0xFF84D0 1 8
```

#### `link-ep3`  (tested)

After `jump 1 0` the probe sets stage = 2, sub mode 0, mode 8 (0xD898). Pictures: the level (0), the story panel (panel: a blond friend says the heroes look busy, a good time to take Jack's garage (250 frames)), then the map panel with EPISODE 3 and its title, and the start of the level.

```recipe link-ep3
jump 1 0
frames 900
snaps every:60:0:900
0 poke 0xFF84D9 1 2
0 poke 0xFF84D1 1 0
0 poke 0xFF84D0 1 8
```

#### `link-ep4`  (tested)

After `jump 1 0` the probe sets stage = 3, sub mode 0, mode 8 (0xD898). Pictures: the level (0), the story panel (panel: Mustapha shows a place on a map and says somebody will pay for the garage (309 frames)), then the map panel with EPISODE 4 and its title, and the start of the level.

```recipe link-ep4
jump 1 0
frames 900
snaps every:60:0:900
0 poke 0xFF84D9 1 3
0 poke 0xFF84D1 1 0
0 poke 0xFF84D0 1 8
```

#### `link-ep5`  (tested)

After `jump 1 0` the probe sets stage = 4, sub mode 0, mode 8 (0xD898). Pictures: the level (0), the story panel (panel: a radio call from a village under dinosaur attack, then Jack says something happened to him (439 frames)), then the map panel with EPISODE 5 and its title, and the start of the level.

```recipe link-ep5
jump 1 0
frames 900
snaps every:60:0:900
0 poke 0xFF84D9 1 4
0 poke 0xFF84D1 1 0
0 poke 0xFF84D0 1 8
```

#### `link-ep6`  (tested)

After `jump 1 0` the probe sets stage = 5, sub mode 0, mode 8 (0xD898). Pictures: the level (0), the story panel (panel: a hero says the doctor is mad, he is creating new life forms (368 frames)), then the map panel with EPISODE 6 and its title, and the start of the level.

```recipe link-ep6
jump 1 0
frames 900
snaps every:60:0:900
0 poke 0xFF84D9 1 5
0 poke 0xFF84D1 1 0
0 poke 0xFF84D0 1 8
```

#### `link-ep7`  (tested)

After `jump 1 0` the probe sets stage = 6, sub mode 0, mode 8 (0xD898). Pictures: the level (0), the story panel (panel: a hero recognises the genius Dr. Fessenden and the equipment (368 frames)), then the map panel with EPISODE 7 and its title, and the start of the level.

```recipe link-ep7
jump 1 0
frames 900
snaps every:60:0:900
0 poke 0xFF84D9 1 6
0 poke 0xFF84D1 1 0
0 poke 0xFF84D0 1 8
```

#### `link-ep8`  (tested)

After `jump 1 0` the probe sets stage = 7, sub mode 0, mode 8 (0xD898). Pictures: the level (0), the story panel (panel: the villain says the heroes have only seen a taste of his power and invites them to come (313 frames)), then the map panel with EPISODE 8 and its title, and the start of the level.

```recipe link-ep8
jump 1 0
frames 900
snaps every:60:0:900
0 poke 0xFF84D9 1 7
0 poke 0xFF84D1 1 0
0 poke 0xFF84D0 1 8
```

#### `link-tally`  (tested)

Stage clear tally (poke sub mode 4, G.$508 = 0, G.$4D5 = 1, G.$4D4 = 1): the hero portrait, point counts (vitality, enemies, the totals) with the TOTAL counting down while the points go into the score, over the finished level; then the map panel scrolls in and the story panel of Episode 2 follows. About 740 frames.

```recipe link-tally
jump 1 0
frames 800
snaps every:50:0:800
0 poke 0xFF84D1 1 4
0 poke 0xFF8508 2 0
0 poke 0xFF84D5 1 1
0 poke 0xFF84D4 1 1
```


### G. Every area of every episode, and the camera X

Frames from power on to the first frame of control (the `done` line of the log, from the last full run; add
one frame per 32 pixels of CAMX). All 23 areas are reached in under 3600 frames (60 seconds) from one line.

| Episode | Area 0 | Area 1 | Area 2 | Area 3 |
| --- | --- | --- | --- | --- |
| 1 | 1726 | 1814 | 1955 | - |
| 2 | 2605 | 2643 | 2756 | - |
| 3 | 2298 | - | - | - |
| 4 | 2929 | 3042 | 3155 | - |
| 5 | 2486 | 2580 | - | - |
| 6 | 2415 | 2678 | 2778 | - |
| 7 | 2415 | 2438 | 2552 | 2665 |
| 8 | 2360 | 2382 | 2471 | 2581 |

`recipes.py --prefix area-` runs the 23 area recipes in one process and writes the overview picture
`re/out/probe/recipes-part/_areas.png` (one picture per area, 384 pixels wide).

#### `area-1-0`  (tested)

`jump 1 0`: first frame of control in Episode 1 area 0 (roofs of the flooded city). One picture at t = 29.

```recipe area-1-0
jump 1 0
frames 30
snaps 29
```

#### `area-1-1`  (tested)

`jump 1 1`: first frame of control in Episode 1 area 1 (hall of the building (armour statues), caption IN THE BUILDING). One picture at t = 29.

```recipe area-1-1
jump 1 1
frames 30
snaps 29
```

#### `area-1-2`  (tested)

`jump 1 2`: first frame of control in Episode 1 area 2 (street with the ruined wall, caption 47TH STREET (vertical scene at its start)). One picture at t = 29.

```recipe area-1-2
jump 1 2
frames 30
snaps 29
```

#### `area-2-0`  (tested)

`jump 2 0`: first frame of control in Episode 2 area 0 (forest edge with the Cadillac at the left). One picture at t = 29.

```recipe area-2-0
jump 2 0
frames 30
snaps 29
```

#### `area-2-1`  (tested)

`jump 2 1`: first frame of control in Episode 2 area 1 (forest sub-area (no far layer: the sky is black between the trees)). One picture at t = 29.

```recipe area-2-1
jump 2 1
frames 30
snaps 29
```

#### `area-2-2`  (tested)

`jump 2 2`: first frame of control in Episode 2 area 2 (dusk with a tyre pile and crates (the Butcher's yard; a diagonal start)). One picture at t = 29.

```recipe area-2-2
jump 2 2
frames 30
snaps 29
```

#### `area-3-0`  (tested)

`jump 3 0`: first frame of control in Episode 3 area 0 (open desert (the only area of Episode 3)). One picture at t = 29.

```recipe area-3-0
jump 3 0
frames 30
snaps 29
```

#### `area-4-0`  (tested)

`jump 4 0`: first frame of control in Episode 4 area 0 (rubble and the sea at night in front of the garage). One picture at t = 29.

```recipe area-4-0
jump 4 0
frames 30
snaps 29
```

#### `area-4-1`  (tested)

`jump 4 1`: first frame of control in Episode 4 area 1 (inside the garage (caption), tyres and pipes). One picture at t = 29.

```recipe area-4-1
jump 4 1
frames 30
snaps 29
```

#### `area-4-2`  (tested)

`jump 4 2`: first frame of control in Episode 4 area 2 (boiler hall of the garage). One picture at t = 29.

```recipe area-4-2
jump 4 2
frames 30
snaps 29
```

#### `area-5-0`  (tested)

`jump 5 0`: first frame of control in Episode 5 area 0 (savanna with a burning group). One picture at t = 29.

```recipe area-5-0
jump 5 0
frames 30
snaps 29
```

#### `area-5-1`  (tested)

`jump 5 1`: first frame of control in Episode 5 area 1 (rock cliff (sub-area start)). One picture at t = 29.

```recipe area-5-1
jump 5 1
frames 30
snaps 29
```

#### `area-6-0`  (tested)

`jump 6 0`: first frame of control in Episode 6 area 0 (wooden mine wall). One picture at t = 29.

```recipe area-6-0
jump 6 0
frames 30
snaps 29
```

#### `area-6-1`  (tested)

`jump 6 1`: first frame of control in Episode 6 area 1 (jungle swamp at night). One picture at t = 29.

```recipe area-6-1
jump 6 1
frames 30
snaps 29
```

#### `area-6-2`  (tested)

`jump 6 2`: first frame of control in Episode 6 area 2 (swamp with the giant dinosaur head). One picture at t = 29.

```recipe area-6-2
jump 6 2
frames 30
snaps 29
```

#### `area-7-0`  (tested)

`jump 7 0`: first frame of control in Episode 7 area 0 (roof of the vault, orange sky). One picture at t = 29.

```recipe area-7-0
jump 7 0
frames 30
snaps 29
```

#### `area-7-1`  (tested)

`jump 7 1`: first frame of control in Episode 7 area 1 (wooden elevator cage (the vertical shaft)). One picture at t = 29.

```recipe area-7-1
jump 7 1
frames 30
snaps 29
```

#### `area-7-2`  (tested)

`jump 7 2`: first frame of control in Episode 7 area 2 (stone room with a fence). One picture at t = 29.

```recipe area-7-2
jump 7 2
frames 30
snaps 29
```

#### `area-7-3`  (tested)

`jump 7 3`: first frame of control in Episode 7 area 3 (metal laboratory corridor with barrels). One picture at t = 29.

```recipe area-7-3
jump 7 3
frames 30
snaps 29
```

#### `area-8-0`  (tested)

`jump 8 0`: first frame of control in Episode 8 area 0 (laboratory with the specimen tubes). One picture at t = 29.

```recipe area-8-0
jump 8 0
frames 30
snaps 29
```

#### `area-8-1`  (tested)

`jump 8 1`: first frame of control in Episode 8 area 1 (sewer, caption UNDERGROUND). One picture at t = 29.

```recipe area-8-1
jump 8 1
frames 30
snaps 29
```

#### `area-8-2`  (tested)

`jump 8 2`: first frame of control in Episode 8 area 2 (cave with barrels, caption of the cave). One picture at t = 29.

```recipe area-8-2
jump 8 2
frames 30
snaps 29
```

#### `area-8-3`  (tested)

`jump 8 3`: first frame of control in Episode 8 area 3 (the final cave area). One picture at t = 29.

```recipe area-8-3
jump 8 3
frames 30
snaps 29
```

#### `jump-camx`  (tested (with a known background glitch))

`jump 5 1 0xF20`: Episode 5 area 1 with the camera at X 0xF20 (the Morgan fight place, the village houses with flames). Known glitch of the camera shift: two 32 pixel wide dark columns where the houses' doors are drawn by objects in the original (section 8). Pictures at t = 0 and 29.

```recipe jump-camx
jump 5 1 0xF20
frames 30
snaps 0,29
```


### H. Rules of a level: GO arrow, locks, clock, TIME OVER, join, heroes, the Cadillac

#### `go-arrow`  (tested)

Episode 1 area 1 (the hall) with every enemy killed all the time (`hold 0xFF850E 1 2`): the wave ends, the camera lock (G.$7B4 lock 1, camera mode 4) ends at about t = 440, the door opens (black rectangle), and the red GO burst blinks at the right edge from about t = 630 (pictures 14 to 17: the hint that the player must walk on). The `watch` shows `lock` and `cammode`.

```recipe go-arrow
jump 1 1
god on
hold 0xFF850E 1 2
frames 900
snaps every:45:0:900
watch 30 G go lock cammode alive
```

#### `wave-lock`  (tested)

`jump 1 0 0x4A0` (just before the first lock) and Jack walks right: at camera X 0x4D0 the camera locks (lock 1, camera mode 4, camx 1232 = 0x4D0 in the watch) and wave 0 attacks; the camera stays locked until the wave is dead.

```recipe wave-lock
jump 1 0 0x4A0
god on
frames 500
snaps every:50:0:500
0-200:R
watch 20 G camx lock cammode alive
```

#### `clock-3min`  (tested)

`time 3:01` in Episode 1 area 1: the white clock 3:00 appears at the top for a few seconds and counts 2:59, 2:58 ... (pictures 2 to 8).

```recipe clock-3min
jump 1 1
frames 400
snaps every:40:0:400
0 time 3:01
```

#### `clock-1min`  (tested)

`time 1:01`: the clock shows 1:00 in red from then on (red below one minute) and counts down.

```recipe clock-1min
jump 1 1
frames 400
snaps every:40:0:400
0 time 1:01
```

#### `time-over`  (tested)

`time 0:05` (no god mode): the red clock counts 0:05 to 0:00, then the words TIME and OVER fly in from both sides, the hero's vitality is set to 0 and he falls (pictures 9 to 14); he comes back with a speech bubble and a bazooka (pictures 17 to 20).

```recipe time-over
jump 1 1
frames 700
snaps every:35:0:700
0 time 0:05
```

#### `fe-join`  (tested)

Player 2 joins during the level: coin 2 at t = 10 (the HUD shows CREDIT 1 and JOIN-IN), start 2 at t = 60 (a quick select strip with the hero's face and name), attack 2 at t = 140 chooses; Hannah drops in with a bazooka and a speech bubble (pictures 3 to 13).

```recipe fe-join
jump 1 1
frames 400
snaps every:30:0:400
10-17:2C
60-65:2S
140-145:2A
```

#### `hero-hannah`  (tested)

`char=hannah`: the whole boot (select screen) with player 1 as Hannah, then Episode 2 area 0. One picture.

```recipe hero-hannah
jump 2 0 char=hannah
frames 30
snaps 29
```

#### `hero-mustapha`  (tested)

`char=mustapha`: the whole boot (select screen) with player 1 as Mustapha, then Episode 2 area 0. One picture.

```recipe hero-mustapha
jump 2 0 char=mustapha
frames 30
snaps 29
```

#### `hero-mess`  (tested)

`char=mess`: the whole boot (select screen) with player 1 as Mess, then Episode 2 area 0. One picture.

```recipe hero-mess
jump 2 0 char=mess
frames 30
snaps 29
```

#### `players-3`  (tested)

`char=jack,hannah,mess`: three players (Jack, Hannah, Mess) enter Episode 2; the HUD shows three blocks. One picture.

```recipe players-3
jump 2 0 char=jack,hannah,mess
frames 30
snaps 29
```

#### `car-radio`  (tested)

Episode 3 at camera X 0x300: a RADIO item (pool E class 9, variant 12) is spawned under Jack and taken with attack at t = 5. The caption about taking the car appears, the Cadillac arrives and the camera starts to drive (G.$518 car = 1 in the watch; camera X +8 px per frame, camx 887 to 4014 in the watch) while bikers and thugs attack from the sides.

```recipe car-radio
jump 3 0 0x300
god on
frames 900
snaps every:60:0:900
0 spawn E 9 var=0xC x=+0x60
5-6:A
watch 30 G car camx
```

#### `lift-ride`  (tested)

`jump 7 1` (Episode 7 area 1, the elevator shaft): the wooden cage rides up with floor numbers L-15, L-14 ... painted on the wall (the camera mode 5 auto ride, camera Y falling in the watch), thugs jump in and fight on the lift; the ride stops at each floor (camera mode 4 lock) until the wave is dead.

```recipe lift-ride
jump 7 1
god on
frames 900
snaps every:60:0:900
watch 30 G camy cammode lock
```


### I. Continue, game over, ranking, ending

#### `fe-continue`  (tested)

Jack gets 0 lives and 0 vitality at t = 0 and dies: the continue screen (mode 0x10 from t = 203): a villain with a gun on a red background, a big number counting 20 to 0 (about 62 frames per number) and a speech bubble that changes four times; at 0 a red flash, blood spots and GAME OVER (mode 0x14 then 0x18 at t = 1985), then the ranking. Pictures every 100 frames.

```recipe fe-continue
jump 1 1
frames 2300
snaps every:100:0:2300
0 lives 0
0 hp 0
```

#### `fe-continue-coin`  (tested)

The same death, but a coin at t = 300 and start at t = 340 during the countdown, attack at t = 460: a quick select strip over the level (the player can choose a hero again), then the hero drops in with a bazooka and a speech bubble (pictures 8 to 18).

```recipe fe-continue-coin
jump 1 1
frames 900
snaps every:50:0:900
0 lives 0
0 hp 0
300-307:C
340-345:S
460-465:A
```

#### `fe-ending`  (tested)

Ending (poke stage 7, flags G.$4D7, G.$4DB, G.$507, sub mode 0, mode 0x1C): the lab explodes while two heroes run, a panel where they call the names of the two others, the desert at sunset with two heroes walking and the staff roll, the Cadillac arrives with the other two, a short dialogue (the madman nearly made his plan real, nature balances itself, let's go home), THANK YOU FOR PLAYING and PRODUCED BY CAPCOM; at t = 5546 mode 0x14 / 0x18 (game over flow, ranking, attract). 5365 frames of ending, one picture every 150 frames.

```recipe fe-ending
jump 1 0
frames 5600
snaps every:150:0:5600
0 poke 0xFF84D9 1 7
0 poke 0xFF84D7 1 1
0 poke 0xFF84DB 1 1
0 poke 0xFF8507 1 1
0 poke 0xFF84D1 1 0
0 poke 0xFF84D0 1 0x1C
```

#### `fe-highscore`  (tested)

Jack has a score of 500000 (`score 500000`) and dies without a continue: after GAME OVER the ranking screen shows the default table unchanged and no name entry appears, so the ranking is display only in this set (the open question in `attract-and-front-end.md`).

```recipe fe-highscore
jump 1 1
frames 2500
snaps every:100:1900:2500
0 score 500000
0 lives 0
0 hp 0
```


### J. Bosses (the boss and its intro in its own level)

#### `boss-vice`  (tested)

`jump 1 2 0xA0` (the trigger of the last wave of Episode 1), six kill-all pulses for the thug of that wave and Jack walking right (god mode on): when the wave is dead the end of the wave starts the boss area; about t = 1000 to 1300 Vice T. (the blond man in the purple coat) is announced, his velociraptors run in, he jumps in and fights (boss bar, `watch` of class 18 in the log). A jump to the boss record itself (X 0xF4) skips the wave and so skips his entrance: he then stands still at X 792.

```recipe boss-vice
jump 1 2 0xA0
god on
frames 1700
snaps every:100:0:1700
60 killall
120 killall
180 killall
240 killall
300 killall
360 killall
0-1700:R
watch 60 A:18 x hp st
```

#### `boss-butcher`  (tested)

`jump 2 2 0x1A0` (the record of class 5, the starter of the Butcher): the screen darkens, a short talk between a hero portrait and the Butcher's portrait, then the Butcher with two swords and a boss bar enters from the right.

```recipe boss-butcher
jump 2 2 0x1A0
god on
frames 720
snaps every:60:0:720
```

#### `boss-hogg`  (tested)

`jump 3 0 0x2D00`: Hogg on his motorbike (three vitality bars) rides in with biker thugs; fire and explosions.

```recipe boss-hogg
jump 3 0 0x2D00
god on
frames 720
snaps every:60:0:720
```

#### `boss-slice`  (tested)

`jump 4 2 0x3D8` and walking right: Slice's portrait talk (what are you doing in my garage / I am here to end your adventure), then Slice with a blade attacks.

```recipe boss-slice
jump 4 2 0x3D8
god on
frames 500
snaps every:50:0:500
0-400:R
```

#### `boss-morgan`  (partial: shows the intro talk and the first fights; Morgan and Morgue themselves are in the `spawn-morgan` and `spawn-morgue` recipes)

`jump 5 1 0x1120` (records of classes 35 and 45) and walking right: a talk with an old man about the poachers, a green-capped friend, then fights with the poacher group; the boss bar appears late (see 1200 frames, one picture every 100).

```recipe boss-morgan
jump 5 1 0x1120
god on
frames 1200
snaps every:100:0:1200
0-500:R
```

#### `boss-tyrog`  (partial: first fight only; the second and third fight (classes 72 and 91) are in `spawn-tyrog2` and `spawn-tyrog3`)

`jump 6 2 0xF00` and a direct spawn of the boss starter (pool B class 79 at the place of the script record, X 0x1120): a thug says you cannot go further and he will show the power of the doctor, then he fights with a boss bar (first Tyrog fight). The script spawns this starter when the camera passes X 0xF80 after the last wave; a direct spawn skips that wait.

```recipe boss-tyrog
jump 6 2 0xF00
god on
frames 600
snaps every:60:0:600
5 spawn B 79 ax=0x1120 z=0x190
```

#### `boss-slisaur`  (partial: the start of the Slisaur fight; the real fight has two or three of them)

`jump 7 3 0x240`, walking right and two kill-all pulses (t = 200 and 320) for the wave of four knife thugs: the monitors of the lab show the face of Dr. Fessenden with a message, and a green Slisaur appears and walks to the heroes.

```recipe boss-slisaur
jump 7 3 0x240
god on
frames 1100
snaps every:90:0:1100
0-900:R
200 killall
320 killall
```

#### `boss-fessenden`  (partial: first form only; the dinosaur form is in `spawn-fessenden1`)

`jump 8 3 0x10A0` and walking right: the first form of Dr. Fessenden (a man in a white coat) stands in the lair while green raptors and knife thugs attack; his boss bar is at the top.

```recipe boss-fessenden
jump 8 3 0x10A0
god on
frames 700
snaps every:60:0:700
0-500:R
```

#### `boss-vice-defeat`  (tested)

As `boss-vice`, then `objset A:18 hp 1` at t = 1500 sets Vice's vitality to 1 and Jack mashes attack while walking right: his next hit kills Vice and the boss defeat flow starts (slow motion, kill-all, the walk off, the tally). Frames to 3200, one picture every 150.

```recipe boss-vice-defeat
jump 1 2 0xA0
god on
frames 3600
snaps every:150:0:3600
60 killall
120 killall
180 killall
240 killall
300 killall
360 killall
0-1650:R
1500 objset A:18 hp 1
1520 god off
1500-1501:A,1510-1511:A,1520-1521:A,1530-1531:A,1540-1541:A,1550-1551:A,1560-1561:A,1570-1571:A,1580-1581:A,1590-1591:A,1600-1601:A,1610-1611:A,1620-1621:A,1630-1631:A,1640-1641:A,1650-1651:A,1660-1661:A,1670-1671:A,1680-1681:A,1690-1691:A
watch 60 A:18 x hp st
```

#### `hazard-bronto-legs`  (tested)

`jump 6 1 0x4A0` and walking right: the legs of a giant dinosaur stamp down in the jungle (a big shadow on the ground, then the foot, pictures 4 to 14) while thugs fight; a grey flash picture at t = 120.

```recipe hazard-bronto-legs
jump 6 1 0x4A0
god on
frames 600
snaps every:40:0:600
0-300:R
```


### K. Dinosaurs and single enemies (spawn by class)

#### `spawn-shivat`  (tested)

`jump 5 0`, then `spawn A` of Shivat (class 24) at t = 10, god mode on, one picture every 30 frames for 300 frames. The enemy is spawned like a script spawn (mode 0, its own entrance).

```recipe spawn-shivat
jump 5 0
god on
frames 300
snaps every:30:0:300
10 spawn A 24 w5c=0x7fff x=+0x100
```

#### `spawn-rhopper`  (tested)

`jump 5 0`, then `spawn A` of Rock Hopper (class 29, variant 0x200) at t = 10, god mode on, one picture every 30 frames for 300 frames. The enemy is spawned like a script spawn (mode 0, its own entrance).

```recipe spawn-rhopper
jump 5 0
god on
frames 300
snaps every:30:0:300
10 spawn A 29 var=0x200 w5c=0x7fff x=+0x100
```

#### `spawn-mack`  (tested)

`jump 5 0`, then `spawn A` of Mack (class 38), the orange horned dinosaur at t = 10, god mode on, one picture every 30 frames for 300 frames. The enemy is spawned like a script spawn (mode 0, its own entrance).

```recipe spawn-mack
jump 5 0
god on
frames 300
snaps every:30:0:300
10 spawn A 38 w5c=0x7fff x=+0x100
```

#### `spawn-zeke`  (tested)

`jump 5 0`, then `spawn A` of Zeke (class 82): first only its shadow runs over the ground, then the red pteranodon dives at t = 10, god mode on, one picture every 30 frames for 300 frames. The enemy is spawned like a script spawn (mode 0, its own entrance).

```recipe spawn-zeke
jump 5 0
god on
frames 300
snaps every:30:0:300
10 spawn A 82 w5c=0x7fff x=+0x100
```

#### `spawn-lash`  (tested)

`jump 2 1`, then `spawn A` of Lash T. (class 3) at t = 10, god mode on, one picture every 30 frames for 300 frames. The enemy is spawned like a script spawn (mode 0, its own entrance).

```recipe spawn-lash
jump 2 1
god on
frames 300
snaps every:30:0:300
10 spawn A 3 x=+0x100
```

#### `spawn-walther`  (tested)

`jump 3 0`, then `spawn A` of Walther (class 18, variant 0x200): jumps in at t = 10, god mode on, one picture every 30 frames for 300 frames. The enemy is spawned like a script spawn (mode 0, its own entrance).

```recipe spawn-walther
jump 3 0
god on
frames 300
snaps every:30:0:300
10 spawn A 18 var=0x200 x=+0x100
```

#### `spawn-morgan`  (partial: the class alone shows a short-lived figure; the real intro comes from the starters 35 and 45 (`boss-morgan`))

`jump 5 1 0xF20`, then `spawn A` of class 36 (Morgan) at t = 10, god mode on, one picture every 30 frames for 300 frames. The enemy is spawned like a script spawn (mode 0, its own entrance).

```recipe spawn-morgan
jump 5 1 0xF20
god on
frames 300
snaps every:30:0:300
10 spawn A 36 x=+0x100
```

#### `spawn-morgue`  (partial: a silver monster appears; its behaviour without the script start is not checked)

`jump 5 1 0xF20`, then `spawn A` of class 40 (Morgue): a silver monster at t = 10, god mode on, one picture every 30 frames for 300 frames. The enemy is spawned like a script spawn (mode 0, its own entrance).

```recipe spawn-morgue
jump 5 1 0xF20
god on
frames 300
snaps every:30:0:300
10 spawn A 40 x=+0x100
```

#### `spawn-tyrog1`  (tested)

`jump 6 2 0xE00`, then `spawn A` of class 4 (Tyrog, first form): a grey raptor-like monster at t = 10, god mode on, one picture every 30 frames for 300 frames. The enemy is spawned like a script spawn (mode 0, its own entrance).

```recipe spawn-tyrog1
jump 6 2 0xE00
god on
frames 300
snaps every:30:0:300
10 spawn A 4 x=+0x100
```

#### `spawn-tyrog2`  (tested)

`jump 6 2 0xE00`, then `spawn A` of class 72 (Tyrog 2): a green lizard on the ground, then a red man who screams for help at t = 10, god mode on, one picture every 30 frames for 300 frames. The enemy is spawned like a script spawn (mode 0, its own entrance).

```recipe spawn-tyrog2
jump 6 2 0xE00
god on
frames 300
snaps every:30:0:300
10 spawn A 72 x=+0x100
```

#### `spawn-tyrog3`  (tested)

`jump 6 2 0xE00`, then `spawn A` of class 91 (Tyrog 3): a grey monster like the first form at t = 10, god mode on, one picture every 30 frames for 300 frames. The enemy is spawned like a script spawn (mode 0, its own entrance).

```recipe spawn-tyrog3
jump 6 2 0xE00
god on
frames 300
snaps every:30:0:300
10 spawn A 91 x=+0x100
```

#### `spawn-slisaur`  (partial: the class alone does not stay; the real fight is started by the pool B starters 41 (`boss-slisaur`))

`jump 7 3 0x100`, then `spawn A` of class 71 (Slisaur) alone at t = 10, god mode on, one picture every 30 frames for 300 frames. The enemy is spawned like a script spawn (mode 0, its own entrance).

```recipe spawn-slisaur
jump 7 3 0x100
god on
frames 300
snaps every:30:0:300
10 spawn A 71 x=+0x100
```

#### `spawn-fessenden1`  (tested)

`jump 8 3 0x1000`, then `spawn A` of class 84 (Fessenden's dinosaur form): a big pink raptor at t = 10, god mode on, one picture every 30 frames for 300 frames. The enemy is spawned like a script spawn (mode 0, its own entrance).

```recipe spawn-fessenden1
jump 8 3 0x1000
god on
frames 300
snaps every:30:0:300
10 spawn A 84 x=+0x100
```

#### `spawn-fessenden2`  (tested)

`jump 8 3 0x1000`, then `spawn A` of class 94 (Fessenden, the man in the white coat) at t = 10, god mode on, one picture every 30 frames for 300 frames. The enemy is spawned like a script spawn (mode 0, its own entrance).

```recipe spawn-fessenden2
jump 8 3 0x1000
god on
frames 300
snaps every:30:0:300
10 spawn A 94 x=+0x100
```

#### `spawn-brontosaurus`  (partial: only a big shadow and mist show; the legs are in `hazard-bronto-legs`)

`jump 6 1 0x520`, then `spawn A` of class 22 (the giant dinosaur of Episode 6) at t = 10, god mode on, one picture every 30 frames for 300 frames. The enemy is spawned like a script spawn (mode 0, its own entrance).

```recipe spawn-brontosaurus
jump 6 1 0x520
god on
frames 300
snaps every:30:0:300
10 spawn A 22 x=+0x100
```


### L. Example measurement

#### `measure-zeke`  (tested)

Zeke spawned at t = 5 in Episode 5 area 0, `watch 1` on its slot (`A:82`) and on Jack (`P1`). The pictures show the shadow, the dive (picture 7) and the hit. The numbers are in section 7.

```recipe measure-zeke
jump 5 0
frames 400
snaps every:40:0:400
5 spawn A 82 var=0 w5c=0x7fff x=+0x100
watch 1 A:82 x y z st
watch 1 P1 hp x z st
```


### M. Cheats and watch forms (checks of the tool itself)

#### `cheat-rank`  (tested)

`rank 31` freezes the rank clock at rank 31 and sets the level that the rank table gives for one player (25); the watch shows `rank=31 level=25 aggr=0` for 150 frames. Tested values: level 25 at rank 31 for one player.

```recipe cheat-rank
jump 1 1
frames 150
rank 31
watch 30 G rank level aggr
```

#### `cheat-level-free`  (tested)

`level 20` and `aggr 5` hold level 20 and aggression 5 (the rank stays at its own value, 13); `level free` at t = 120 gives the rank clock back (the level is then computed from the rank again; check the log).

```recipe cheat-level-free
jump 1 1
frames 240
level 20
aggr 5
120 level free
watch 30 G level aggr rank
```

#### `cheat-score-lives`  (tested)

`score 123456`, `lives 7` and `hp 40` on player 1: the watch shows `score=00123456 lives=7`, the HUD shows the score and the bar at 40.

```recipe cheat-score-lives
jump 1 1
frames 40
snaps 39
score 123456
lives 7
hp 40
watch 20 P1 score lives hp
```

#### `watch-fields`  (tested)

All watch forms in one segment: a player with a raw field (`+0x6C.sw`) and an absolute word (`@0xFF84E8.w`, the clock), pool B by class, pool E by class, pool A by class, and `G`. Also shows `spawn` for pools B, E and A.

```recipe watch-fields
jump 1 1
frames 60
snaps 5,45
5 spawn B 66 var=9
5 spawn E 4 var=1 x=+0x20
5 spawn A 2 var=4 x=+0x100
watch 20 P1 hp x +0x6C.sw @0xFF84E8.w
watch 20 B:66 x y
watch 20 E:4 x y
watch 20 A:2 x hp st
watch 20 G time camx
```


## 7. Example measurement: Zeke's dive

Question: how fast does Zeke (pool A class 82, the pteranodon) move, and what does its dive cost Jack?
Recipe `measure-zeke` (Episode 5, area 0). The same lines work in any batch. Read the table with
`series.py` (slot A#20 is the slot of the spawn; see the `SPAWN` line of the log):

```sh
re/.venv/bin/python -I re/harness/probe/recipes.py measure-zeke
re/.venv/bin/python -I re/harness/probe/series.py re/out/probe/recipes-part/measure-zeke/log.txt A#20 x y z st --delta
re/.venv/bin/python -I re/harness/probe/series.py re/out/probe/recipes-part/measure-zeke/log.txt P1 hp x z
```

What the `W` lines show (frames `t` after the jump; Zeke spawned at t = 5 with `w5c=0x7fff`, `var=0`, 0x100 pixels right of the camera, camera X 0, Jack at X 49; positions are the integer parts of X, height Y and depth Z):

| t (frames) | Zeke state `st` (+$04..+$07) | What the numbers do |
| --- | --- | --- |
| 5 to 6 | 0x00000000, then 0x02000000 | init: starts at height 320 (Y) |
| 7 to 188 | 0x02000200 | cruise: X falls by 1 pixel per game frame (a little less in the first frames), height falls from 320 to about 186 (about 0.75 pixel per frame), depth Z weaves between 562 and 582 |
| 189 to 238 | 0x02000400 (the bytes +$06/+$07 go 4, then 6 at t = 239) | turn and wind-up: height stays near 211, X still -1 per frame |
| 239 to 267 | 0x02000600 | **dive**: X -1 per frame, height falls from 180 to 0 with a growing step of -3, -4, -5, -6, -7 pixels per frame; depth fixed at 571 (the depth of Jack, 576, minus a few pixels) |
| 263 | Jack: 0x02020000 | **Jack loses 15 vitality (100 to 85)** at the frame Zeke is at X 116, height 99 (the hit box of the long dive pose reaches Jack at X 49); Jack goes into his hurt state |
| 268 to 338 | 0x02000800 | recovery: it touches the ground at t = 285 (height 0), then climbs back at +3 to +4 pixels per frame while X keeps falling to 37 |
| 339 on | 0x02000200 | cruise again at the other end (height 213, X rises again), the next dive follows |

Jack's second drop (85 to 67, t = 387) comes from a thug (his state 0x02021800), not from Zeke. The damage of 15 is
for the rank and player count of this run (one player, rank at its start value); the damage tables are indexed by
level (`re/specs/enemy/SPEC.md` section 6.6), so repeat the run with `level N` for other levels.

## 8. Known limits and what does not work

* **Camera X of `jump`** skips every spawn record before X: no enemies, waves, props before X (props are kept
  from X - 0x400), and no script commands (clock set, forced aggression). The clock and the rank are those of the
  area start, not those of a natural run. Objects that draw into the background when they are created (huts, doors)
  can be missing: in Episode 5 area 1 around X 0xF00 to 0x1100 two 32 pixel wide dark columns show where doors
  are (`jump-camx`). When the exact picture matters, jump to an earlier X (or to the area start) and let the hero
  walk there (`wave-lock` is an example of walking with keys; add `god on` and `killall`).
* A camera X in the two vertical areas (Episode 1 area 2, Episode 7 area 1) is ignored (their camera moves in Y).
  X is cut to 0x10 before the end-of-area event of the area.
* In diagonal areas (Episode 2 area 2, Episode 5 area 1, Episode 6 area 2) the probe ends the diagonal camera
  modes itself and puts the players back on the screen; the heroes do not walk down the slope.
* Bosses that are started by a wave (Vice T.: the end of the last wave starts the boss area) must be reached
  through that wave: jump to the trigger of the wave (X 0xA0 in Episode 1 area 2), not to the boss record
  (`boss-vice`). Bosses started by a record (Butcher, Hogg, Slice, Fessenden) work with the record's X.
  Tyrog, Morgan and the Slisaur fights are only partly covered (see their status).
* `spawn` writes the same fields as the script spawn. A class number outside the class map of the pool (A 0..101,
  B 0..96, E 0..11) makes the game jump into garbage: use classes from `re/specs/enemy/classmap.txt` and the
  stage scripts. Spawned bosses lack the camera locks, props and counters of their script start.
* `god` keeps the hit-test bit `+$00 = 2` of the player: enemies swing and miss, no hurt reaction plays. Use
  `hp` or no god mode to measure damage.
* The probe samples at the end of each video frame, not at the vblank flag like `re/harness/record.lua`. A game
  frame that is slow shows the same state twice (`gf` does not change). `series.py` removes the repeats.
* Sound is off (`-sound none`, and the QSound and PLD ROMs are placeholders). The probe does not log sound
  requests. Sound cues must be taken from the code and the harness (`re/harness/README.md`).
* A batch of segments runs in one MAME process: the first segment starts from the real power on, the others
  after the probe loaded the saved first-frame state. I checked that the same segment twice gives the same
  state frames and the same positions. A Lua error inside one segment ends only that segment (`segments.txt`).
* The Dock behaviour (no icon, no focus) comes from the SDL dummy drivers; the coordinator checked it with
  `lsappinfo`; I only checked that screenshots, which are written by the emulation, still work with them.
* Not done: a directive that dumps the sound request queue; a `players=` option other than the `char=` list;
  jumping to the middle of a vertical area; the 2 player demo layout; a way to start a segment from a cached
  state of a level (every segment boots again, 1700 to 3200 frames, about 1 to 3 seconds).
