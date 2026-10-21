# Running dino-rs

The Rust port of Cadillacs and Dinosaurs. It needs your own `dino.zip` in the repo root (the game reads it at run time).
All commands run from the `dino-rs` folder.

## 1. Build and play

    cd dino-rs
    cargo run --release -p dino-game -- --rom ../dino.zip

The game starts from power on: the boot screens, then the attract mode. Put in a coin and press start.

Keys (player 1 on the keyboard):

- Move: arrow keys.
- Attack: J or Z. Jump: K or X.
- Special moves, one key each (player 1). Each key presses the original game's own buttons for the move, with the right timing, so the game plays it exactly as if you pressed them by hand:

| Key | Move | What the key presses for you |
| --- | --- | --- |
| A | command move ("sure kill": Jack's flip kick, Mustapha's somersault, Hannah's rolling elbow, Mess's flying lariat) | down, up, then up + attack |
| S | dive kick (jump + left / right + attack at the top of the jump) | jump, then at the top attack + the direction |
| D | jump attack | jump, then attack held |
| F | special ("mega crush", costs a little HP when it hits) | attack + jump together |
| G | dash attack | the direction twice (a run), then attack |

  The dive kick and the dash attack go the way of the arrow you hold, or the way you last moved. While a move key plays (up to 24 frames), the other player 1 keys wait. The command move works when the hero is standing or walking (not in the air). Checked with all four heroes (the attack ids 0x70 to 0x76, 0x0E, 0x0C, 0x12, 0x5A of D2a).
- Combo toggle: Q turns it on or off (a yellow "COMBO" shows at the bottom left while it is on). While it is on, the third attack press of a chain (three presses, each within 40 frames of the one before) plays the command move (the A key) in place of the third hit. The move waits until the hero can walk again (a few frames after the second hit), and the third press itself is not passed on. Example: turn it on with Q, then press J three times quickly: jab, jab, then the flip kick (Jack).
- Pause: P pauses and continues the game (a "PAUSED" text shows, the sound stops). It works in the game window (not in the asset viewer, and not while the test menu is open).
- Start: 1, 2, 3 (Enter also starts player 1). Coin: 5, 6, 7 (for slots 1, 2, 3).
- SERVICE: 9 (adds a credit for every player).
- Fullscreen: F11 or Alt+Enter (Option+Enter on Mac) toggles fullscreen. Window scaling uses integer zoom to preserve the arcade 384x224 aspect ratio with crisp pixels and black borders.
- Players 2 and 3 use gamepads: the first three pads drive players 1 to 3. D-pad or left stick moves, South (A / Cross) attacks, East (B / Circle) jumps, Start starts, Select / Back is a coin.

Options:

- `--fullscreen`: start in fullscreen mode.
- `--frames N`: exit automatically after N frames (works in windowed and headless runs).
- `--mute`: no sound (for windowed runs).
- `--config FILE`: read the settings from this file. Without it, `dino.cfg` next to the ROM is read if it exists.
- `--print-config`: print the factory settings file (a starting point for `dino.cfg`).

## 2. Test menu and debug keys

- `--test`: a windowed run with the debug keys on and the test menu open at start.
- `--debug-keys`: the debug keys in a normal windowed run (F2 to F6 below). With them on, F1 opens and closes the test menu. Without them, F1 shows the hit boxes.

The menu is a dark box over the game. The game is paused while it is open.

| Key | What it does |
| --- | --- |
| Up / Down | Move the cursor (headings are skipped) |
| Enter | Jump to the point under the cursor (the menu closes). On `Hero` or `Players`: change the value. On `God mode`: toggle it |
| Left / Right | Change `Hero` (Jack, Mustapha, Hannah, Mess) and `Players` (1 to 3). Players 2 and 3 take the next heroes |
| Esc or F1 | Close the menu |

The menu lines: the jump points grouped by episode, then `Options` with `Hero: Jack`, `Players: 1` and `God mode: off`.

Debug keys (with `--debug-keys` or `--test`):

    F1 menu  F2 kill all  F3 skip wave  F4 time 5:00  F5 god  F6 next area

- F2: kill all enemies on the screen.
- F3: skip the wave (the enemies die and the next group comes).
- F4: set the stage time to 5:00.
- F5: god mode (the players keep their HP). The same as `God mode` in the menu.
- F6: go to the next area.

## 3. Jumps

A jump starts the game at a named place: an episode start or an area start.

    cargo run --release -p dino-game -- --rom ../dino.zip --list-jumps
    cargo run --release -p dino-game -- --rom ../dino.zip --jump ep4a2 --chars 2

- `--list-jumps` prints every point (name, title, episode, area, camera X) and exits.
- `--jump NAME` starts at that point. It takes `--chars` (the heroes, for example `--chars 0,2` for Jack and Hannah). An unknown name prints the list and exits with an error.
- You can also start with the flags that a jump stands for: `--episode E --area A --camx X --chars 0`. `--episode 1..8`, `--area 0..3` and `--camx` (hex with 0x, or decimal) start that area with the camera at X. `--jump` cannot be combined with these flags.

Jump points (the boss scene points were removed on 2026-10-21; start at the episode or the area and walk on):

| Name | Title | Episode | Area | Camera X |
| --- | --- | ---: | ---: | ---: |
| ep1 | Episode 1 start: flooded city | 1 | 0 | 0x0000 |
| ep1a1 | Episode 1 area 1: the building | 1 | 1 | 0x0000 |
| ep1a2 | Episode 1 area 2: 47th street | 1 | 2 | 0x0000 |
| ep2 | Episode 2 start: swamp forest | 2 | 0 | 0x0000 |
| ep2a1 | Episode 2 area 1: forest | 2 | 1 | 0x0000 |
| ep2a2 | Episode 2 area 2: Butcher yard | 2 | 2 | 0x0000 |
| ep3 | Episode 3 start: hell road | 3 | 0 | 0x0000 |
| ep4 | Episode 4 start: Jack's garage | 4 | 0 | 0x0000 |
| ep4a1 | Episode 4 area 1: inside the garage | 4 | 1 | 0x0000 |
| ep4a2 | Episode 4 area 2: boiler hall | 4 | 2 | 0x0000 |
| ep5 | Episode 5 start: village of flame | 5 | 0 | 0x0000 |
| ep5a1 | Episode 5 area 1: rock cliff | 5 | 1 | 0x0000 |
| ep6 | Episode 6 start: jungle and mine | 6 | 0 | 0x0000 |
| ep6a1 | Episode 6 area 1: jungle swamp | 6 | 1 | 0x0000 |
| ep6a2 | Episode 6 area 2: giant head | 6 | 2 | 0x0000 |
| ep7 | Episode 7 start: the vault | 7 | 0 | 0x0000 |
| ep7a1 | Episode 7 area 1: elevator | 7 | 1 | 0x0000 |
| ep7a2 | Episode 7 area 2: stone room | 7 | 2 | 0x0000 |
| ep8 | Episode 8 start: the under world | 8 | 0 | 0x0000 |
| ep8a1 | Episode 8 area 1 | 8 | 1 | 0x0000 |
| ep8a2 | Episode 8 area 2 | 8 | 2 | 0x0000 |
| ep8a3 | Episode 8 area 3 | 8 | 3 | 0x0000 |

## 4. Headless runs (no window)

Every enemy class of the game runs its own code. A class word the port does not know is freed with the message `pool A class N not ported (freed)` on stderr; the census of the bot (`DINO_AUTO_CENSUS=1`) prints `class:count` pairs without a marker.

- `--headless --autoplay`: a bot (god mode, the flow test) plays from power on through the eight episodes to the ending and back to the attract loop. It stops there (a few seconds). Mode changes go to stderr.
- `--headless --screenshots 60,600 --shot-dir DIR`: runs the game and saves the pictures at frames 60 and 600 as `DIR/f00060.png` and `DIR/f00600.png`. Add `--jump NAME` (or `--episode`, `--area`, `--camx`) to choose the start.
- `--test --headless --screenshots 30 --shot-dir DIR`: draws the test menu over the picture at frame 30.
- The attract demos against MAME (M10-03): `re/harness/m10_demo_check.sh` (from the repo root; the six demos of the loop, a free run and a step-like run each).
- The sound against MAME (M10-22, M10-06): `re/harness/m10_sound_check.sh` (the sound requests of the 30 whole runs) and `re/harness/m10_loop_check.sh` (the two attract loops from power on: states, heard sounds, pictures). The whole game through the mixer: `cargo test --release -p dino-game whole_game_mix -- --ignored --nocapture` (add `DINO_WAV_OUT=DIR` for one WAV a part, to listen to).
  The pictures of the six demos at their frame 300: `--headless --screenshots 3654,10225,16759,23330,29864,36435 --shot-dir DIR` (no `--jump`: from power on).
  A diagnostic for any power-on trace: `DINO_LAGSYNC_EVERY=1 DINO_LAGSYNC_RANGE=9925-11123 dino-game --rom ../dino.zip --lagsync TRACE --trace OUT --frames 11300`
  loads MAME's state after every record of the range only (example: the Episode 3 demo).

### The smart bot (`--autoplay smart`)

    cargo run --release -p dino-game -- --rom ../dino.zip --headless --autoplay smart
    cargo run --release -p dino-game -- --rom ../dino.zip --autoplay smart --chars 2          # watch it play (a window)

A bot that plays like a player, with a planner (`planner.rs`). It presses only inputs: no god mode, no kill all, no clock top up, no debug keys. The old `--autoplay` (the flow test with god mode) is unchanged.

- From power on it puts in a coin, presses start, moves the select cursor to the hero of `--chars` (one hero: `0` Jack, `1` Mustapha, `2` Hannah, `3` Mess) and picks it. In a scene it holds attack, which skips the waits of the scripted scenes.
- In the game it decides every 8 frames. It plays about 60 candidate moves (the 8 directions and none, with attack, a two hit combo, jump, attack + jump, jump attack, and a run) on copies of the game (`Game` is `Clone`), for 96 frames, and scores the end state: the hero's HP and lives lost, the damage dealt to enemies and bosses, kills, progress to the right, the areas and episodes entered, food and weapons taken. A beam search keeps the 4 best plans of 3 moves; the first move of the best plan is played, then it plans again. The simulations run on all cores (up to 8) and are collected in order, so a run is the same every time.
- `--chars N` chooses the hero. `--smart-continues N` is the number of continues the bot may use after the last life (default 3; 0 ends the run at the game over).
- It also works with `--episode E --chars N` (a direct start in an episode) and `--frames N`.
- Settings for a test run (environment variables): `DINO_SMART_H` horizon in frames (96), `DINO_SMART_D` plan depth (3), `DINO_SMART_B` beam width (4), `DINO_SMART_THREADS`, the weights `DINO_SMART_DAMAGE`, `_KILL`, `_PROGRESS`, `_NEAR`, `_LIFE`, `_HP_FULL`, `_HP_EMPTY`, and `DINO_SMART_DIAG=N` (a state line every N frames on stderr).
- Output on stderr: one line for every episode and area, boss fight, lost life and continue, and at the end the summary line: `episodes cleared | lives lost | continues | score | frames | hp lost per episode | boss fights (episode:frames) | ending | real time, ticks per second, decisions, simulated frames, the cost of one game clone`.
- Tests: `cargo test -p dino-game planner` (fast ones) and `cargo test --release -p dino-game smart -- --ignored` (clears Episode 1 with Jack without a continue).
- What it reaches today (2026-10-21, 1 player, no continue used): from power on it clears all eight episodes and reaches the ending with each of the four heroes (Jack 2 lives lost, 136,812 frames; Mustapha 2, 130,623; Hannah 0, 135,562; Mess 2, 129,807). Episode 1 alone: `--headless --autoplay smart --episode 1 --frames 12000` clears it in about 7,200 frames (Vice T. beaten, no life lost) in about 18 s.
- Start it with `--episode E`, not `--jump NAME`: a jump skips the area scripts and can leave the camera locked.
- Speed: planning costs 4 to 10 times the game (it uses up to 8 cores); headless it runs at several hundred frames per second, so the whole game takes a few minutes; in a window it should keep up with the normal 60 frames per second (not checked yet).

`--help` prints all the options.

## Debug and test paths

Every path below is off in a plain `cargo run -p dino-game -- --rom ../dino.zip` (windowed, from power on). Only the F1 hit box overlay is reachable without a flag, and it is a player option (section 2).

- F1 hit box overlay: no flag; windowed only; turns the sprite hit boxes on or off (`main.rs`). Player option, documented in section 2.
- Debug keys F2 kill all, F3 skip wave, F4 time 5:00, F5 god mode, F6 next area: `--debug-keys` (or `--test`), windowed only (`debug.rs`).
- Test menu: `--test` (open at start), or F1 with `--debug-keys` (`menu.rs`). Headless picture of it: `--test --headless --screenshots F --shot-dir DIR`.
- God mode: F5 or `God mode` in the test menu, with the debug keys on only (`debug.rs`).
- Jump start flags `--jump`, `--episode`, `--stage`, `--area`, `--camx`, `--chars`: a start in the middle of the game, not from power on (`main.rs`, `jumps.rs`).
- Autoplay bot `--autoplay` (god mode, flow test) and `--autoplay smart` (planner, presses inputs only): the flag is needed (`autoplay.rs`, `planner.rs`).
- Bot env vars: `DINO_AUTO_MORTAL`, `DINO_AUTO_NOKILL`, `DINO_AUTO_CENSUS`, `DINO_AUTO_DIAG`, `DINO_AUTO_RANGE` (read only by `--autoplay`); `DINO_SMART_*` and `DINO_SMART_DIAG` (read only by `--autoplay smart`).
- Harness flags `--trace`, `--step-from`, `--follow-rng`, `--lagsync`, `--load-state`, `--keep-dip`, `--poke`, `--poke-file`, `--poke-clock` (`harness.rs`, `main.rs`): need the flag. Harness env vars `DINO_GAME_CLOCK`, `DINO_STEP_PLAYERS`, `DINO_LAGSYNC_EVERY`, `DINO_LAGSYNC_RANGE`, `DINO_LAGSYNC_TICKS`: read only in a `--trace` run.
- Scripted input `--input`, `--input-file` (`input.rs`): need the flag.
- Asset viewer `--viewer` or `--view char,set,anim` (`viewer.rs`): need the flag; its F key is read only there.
- Screenshots `--screenshot`, `--screenshots`, `--shot-dir`: need the flag; they make a headless run when no window is asked.
- Info exits `--help`, `--list-jumps`, `--print-config`: print and exit.
- Test-only: `DINO_WAV_OUT` (audio listening files, `audio.rs`) and the `#[ignore]` review runs (`game/flow/mame_check.rs`, `planner.rs`): only under `cargo test`.
- Messages a player can see on the terminal in a plain run: none on a normal start. `audio: ...` appears only when no audio device opens (the game runs silent). `error: ...` appears only on a failure (for example a missing `dino.zip`). `pool A class N not ported (freed)` appears only if a class is not ported (none is left).
