# Running dino-rs

The Rust port of Cadillacs and Dinosaurs. It needs your own `dino.zip` in the repo root (the game reads it at run time).
All commands run from the `dino-rs` folder.

## 1. Build and play

    cd dino-rs
    cargo run --release -p dino-game -- --rom ../dino.zip

The game starts from power on: the boot screens, then the attract mode. Put in a coin and press start.

Keys (player 1 on the keyboard):

- Move: arrow keys, or W A S D.
- Attack: J or Z. Jump: K or X.
- Start: 1, 2, 3 (Enter also starts player 1). Coin: 5, 6, 7 (for slots 1, 2, 3).
- SERVICE: 9 (adds a credit for every player).
- Players 2 and 3 use gamepads: the first three pads drive players 1 to 3. D-pad or left stick moves, South (A / Cross) attacks, East (B / Circle) jumps, Start starts, Select / Back is a coin.

Options:

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

A jump starts the game at a named place: an episode start, an area start, or the place just before a boss fight.

    cargo run --release -p dino-game -- --rom ../dino.zip --list-jumps
    cargo run --release -p dino-game -- --rom ../dino.zip --jump slice --chars 2

- `--list-jumps` prints every point (name, title, episode, area, camera X) and exits.
- `--jump NAME` starts at that point. It takes `--chars` (the heroes, for example `--chars 0,2` for Jack and Hannah). An unknown name prints the list and exits with an error.
- You can also start with the flags that a jump stands for: `--episode E --area A --camx X --chars 0`. `--episode 1..8`, `--area 0..3` and `--camx` (hex with 0x, or decimal) start that area with the camera at X. `--jump` cannot be combined with these flags.

Jump points. The boss points are the snapshot positions just before each fight. The fight starts when the hero walks on, so a headless run with no input may not show the boss yet.
Hogg (Episode 3) and Tyrog (Episode 6 area 2) are not listed: a right walk from their camera X never reaches the fight (Hogg's camera stops at 0x300 in the car stage; Tyrog's camera goes back to 0x180 in the first frames).

| Name | Title | Episode | Area | Camera X |
| --- | --- | ---: | ---: | ---: |
| ep1 | Episode 1 start: flooded city | 1 | 0 | 0x0000 |
| ep1a1 | Episode 1 area 1: the building | 1 | 1 | 0x0000 |
| ep1a2 | Episode 1 area 2: 47th street | 1 | 2 | 0x0000 |
| vice | Episode 1: Vice T. | 1 | 2 | 0x00A0 |
| ep2 | Episode 2 start: swamp forest | 2 | 0 | 0x0000 |
| ep2a1 | Episode 2 area 1: forest | 2 | 1 | 0x0000 |
| ep2a2 | Episode 2 area 2: Butcher yard | 2 | 2 | 0x0000 |
| butcher | Episode 2: Butcher | 2 | 2 | 0x01A0 |
| ep3 | Episode 3 start: hell road | 3 | 0 | 0x0000 |
| car | Episode 3: Cadillac drive | 3 | 0 | 0x0140 |
| ep4 | Episode 4 start: Jack's garage | 4 | 0 | 0x0000 |
| ep4a1 | Episode 4 area 1: inside the garage | 4 | 1 | 0x0000 |
| ep4a2 | Episode 4 area 2: boiler hall | 4 | 2 | 0x0000 |
| slice | Episode 4: Slice | 4 | 2 | 0x03D8 |
| ep5 | Episode 5 start: village of flame | 5 | 0 | 0x0000 |
| ep5a1 | Episode 5 area 1: rock cliff | 5 | 1 | 0x0000 |
| morgan | Episode 5: Morgan | 5 | 1 | 0x1120 |
| ep6 | Episode 6 start: jungle and mine | 6 | 0 | 0x0000 |
| ep6a1 | Episode 6 area 1: jungle swamp | 6 | 1 | 0x0000 |
| ep6a2 | Episode 6 area 2: giant head | 6 | 2 | 0x0000 |
| ep7 | Episode 7 start: the vault | 7 | 0 | 0x0000 |
| ep7a1 | Episode 7 area 1: elevator | 7 | 1 | 0x0000 |
| lift | Episode 7: the lift | 7 | 1 | 0x0000 |
| ep7a2 | Episode 7 area 2: stone room | 7 | 2 | 0x0000 |
| slisaur | Episode 7: Slisaurs | 7 | 3 | 0x0240 |
| ep8 | Episode 8 start: the under world | 8 | 0 | 0x0000 |
| ep8a1 | Episode 8 area 1 | 8 | 1 | 0x0000 |
| ep8a2 | Episode 8 area 2 | 8 | 2 | 0x0000 |
| ep8a3 | Episode 8 area 3 | 8 | 3 | 0x0000 |
| fessenden | Episode 8: Fessenden | 8 | 3 | 0x0F30 |

## 4. Headless runs (no window)

- `--headless --autoplay`: a bot (god mode, the flow test) plays from power on through the eight episodes to the ending and back to the attract loop. It stops there (a few seconds). Mode changes go to stderr.
- `--headless --screenshots 60,600 --shot-dir DIR`: runs the game and saves the pictures at frames 60 and 600 as `DIR/f00060.png` and `DIR/f00600.png`. Add `--jump NAME` (or `--episode`, `--area`, `--camx`) to choose the start.
- `--test --headless --screenshots 30 --shot-dir DIR`: draws the test menu over the picture at frame 30.

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

`--help` prints all the options.
