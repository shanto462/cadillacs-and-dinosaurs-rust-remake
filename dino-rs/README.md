# Cadillacs and Dinosaurs (Rust Port)

A native Rust port of the Capcom CPS1 arcade game *Cadillacs and Dinosaurs* (1993).

This is a native rewrite of the game logic in Rust, not an emulator. No Motorola 68000 CPU emulator runs, and there is no memory image. Game graphics, sound samples, music, and data tables are read at run time from your own `dino.zip` archive (MAME ROM set "dino").

The repository contains no copyrighted ROM data or assets.

## How to Build

You need a working Rust toolchain (Rust edition 2024). You can install Rust using [rustup.rs](https://rustup.rs).

Clone the repository, enter the `dino-rs` folder, and build the release binary:

```sh
cd dino-rs
cargo build --release -p dino-game
```

The compiled binary is placed at `target/release/dino-game`.

### Check the Build Without a Window
To see that the build and `dino.zip` work (no window, no sound), run the game headless for 3000 frames from power on and save two pictures:

```sh
./target/release/dino-game --rom ../dino.zip --headless --frames 3000 --screenshots 600,3000 --shot-dir shots
```

`shots/f00600.png` shows the Capcom title and `shots/f03000.png` the same title with INSERT COIN (checked in a fresh clone on 2026-10-21).

If the build prints `warning: stripping debug info with rust-objcopy failed`, it is harmless: the binary is built and works (the Rust installation has no `llvm-tools` component).

### System Notes
- **macOS**: Fullscreen toggle uses Option+Enter or F11. No extra system packages are required.
- **Linux / Windows**: Fullscreen toggle uses Alt+Enter or F11. On Linux, standard graphics (X11 or Wayland) and audio (ALSA) development libraries are used if compiling from source.

## Getting Started

### Where to Put dino.zip
The game requires the original arcade ROM archive `dino.zip` (MAME set "dino", World 930201 release).

You can place `dino.zip` in any of the following locations:
1. The repository root directory (`../dino.zip` relative to `dino-rs`)
2. The `dino-rs` directory (`./dino.zip`)
3. Any custom location by passing `--rom <path>` or setting the `DINO_ROM` environment variable

### Running the Game

To build and start the game from the `dino-rs` directory:

```sh
cargo run --release -p dino-game -- --rom ../dino.zip
```

Or run the compiled binary directly:

```sh
./target/release/dino-game --rom ../dino.zip
```

The game boots from power-on, showing the Capcom and QSound logos, followed by the attract mode and title sequence. Insert a coin and press Start to play.

## Controls

### Keyboard (Player 1)

| Action | Keys |
| --- | --- |
| Move | Arrow keys (Up, Down, Left, Right) |
| Attack | J or Z |
| Jump | K or X |
| Start (Player 1) | 1 (or Enter, when Alt/Option is not held) |
| Start (Player 2) | 2 |
| Start (Player 3) | 3 |
| Coin (Slot 1) | 5 |
| Coin (Slot 2) | 6 |
| Coin (Slot 3) | 7 |
| Service Coin | 9 (adds one credit for every player) |
| Fullscreen | F11, or Alt+Enter (Option+Enter on macOS) |
| Hitbox Overlay | F1 (toggles hitbox display during normal play) |

### Special Move Keys (Player 1)

The keyboard provides dedicated single-key shortcuts for special techniques. Each key plays the original arcade input sequence with exact frame timing:

| Key | Move | Description | Inputs Pressed |
| --- | --- | --- | --- |
| A | Command move ("sure kill") | Hero special technique (Jack's flip kick, Mustapha's somersault, Hannah's rolling elbow, Mess's flying lariat). Works while standing or walking on the ground. | Down, then Up, then Up + Attack |
| S | Dive kick | Aerial diving attack towards enemies. Goes in the direction you hold, or your last moved direction. | Jump, then Attack + direction at the apex of the jump |
| D | Jump attack | Low jump attack. | Jump, followed immediately by holding Attack |
| F | Special move ("mega crush") | The hero cannot be hit during it. It costs a little health, and only when it hits. | Attack + Jump together |
| G | Dash attack | Jack slides; the other heroes lunge through the air. Goes in the direction you hold, or your last moved direction. | Direction tapped twice to run, then Attack |

While a special move sequence is playing (up to 24 frames), other Player 1 inputs pause until the move finishes.

### Gamepads (Players 1, 2, and 3)

Connected USB or Bluetooth gamepads automatically map to Players 1, 2, and 3 in device order:

| Gamepad Button | Action |
| --- | --- |
| D-pad or Left Analog Stick | Move |
| South button (A on Xbox, Cross on PlayStation) | Attack |
| East button (B on Xbox, Circle on PlayStation) | Jump |
| Start button | Start |
| Select / Back button | Insert Coin |
| Guide / Mode button | Service Coin |

## Configuration Settings (`dino.cfg`)

Game options are configured using a plain text file named `dino.cfg`.

### Generating a Default Configuration
To print the default factory configuration with documentation for each setting, run:

```sh
cargo run --release -p dino-game -- --print-config
```

You can save this output directly to create your configuration file:

```sh
cargo run --release -p dino-game -- --print-config > dino.cfg
```

### Config File Lookup
The game searches for configuration in this order:
1. File specified by `--config <path>`
2. File path in the `DINO_CONFIG` environment variable
3. `dino.cfg` in the same directory as your `dino.zip` ROM
4. `./dino.cfg` (current directory)
5. `../dino.cfg` (parent directory)
6. If no file exists, factory defaults are used.

### Available Settings
The file uses `key = value` lines. Empty lines and comments starting with `#` are ignored. Boolean options accept `on` or `off` (as well as `true`/`false`, `yes`/`no`, `1`/`0`).

- `coin_mode`: Coin and credit pricing (0 to 8). Factory default is 0 (1 coin = 1 credit). Mode 8 is 2 coins to start, 1 coin to continue.
- `chute_mode`: Coin chute layout:
  - 0: Three players with three separate coin slots (factory default)
  - 1: Three players sharing one common coin chute
  - 2: Two players sharing one coin chute
- `free_play`: Set to `on` to play without inserting coins. Default is `off`.
- `continue_allowed`: Set to `on` to allow players to continue after losing their last life. Default is `on`.
- `demo_sound`: Set to `on` to play music and sound effects during the title and attract sequence. Default is `off`.
- `stereo`: Set to `on` for QSound stereo audio, or `off` for mono audio. Default is `on`.
- `difficulty`: Game difficulty rank from 0 (extra easy) to 7 (hardest). Factory default is 3 (normal).
- `timer_level`: Controls how rapidly game rank increases over time: 0 (easy), 1 (normal), 2 (hard), 3 (very hard). Default is 1.
- `bonus_life`: Score thresholds for earning extra lives:
  - 0: None
  - 1: Extra life at 300,000 and 700,000 points
  - 2: Extra life at 500,000 and 1,000,000 points (factory default)
  - 3: Extra life at 1,000,000 points only
- `start_lives`: Extra lives granted at game start from 0 to 3. Default is 1 (giving 2 lives total).
- `sound`: Set to `on` to enable audio output, or `off` to run silent. Default is `on`.

### Command-Line Options
- `--rom <path>`: Path to the `dino.zip` ROM archive.
- `--fullscreen`: Launch the game directly in fullscreen mode.
- `--mute`: Run without opening an audio device (silent, same as `sound = off` in `dino.cfg`).
- `--config <path>`: Load settings from an explicit configuration file.
- `--print-config`: Print factory configuration to standard output and exit.

## What Is Not Included

This port stays faithful to the original gameplay logic, with a few differences:

- **No Operator Test Menu**: The original arcade cabinet operator test menu (EEPROM settings screen) is not included. Machine settings, coins, difficulty, and lives are configured through `dino.cfg` instead.
- **Other Region Sets**: Only the standard World version (MAME set "dino") is supported. Other regional ROM sets (such as US or Japan variants) are not supported.
- **Arcade Debug Mode**: The original arcade board debug dip switches and test routines are omitted from normal play.
- **No Arcade Hardware Slowdown**: The original 1993 arcade hardware experienced slowdown when many enemies, projectiles, or explosions filled the screen. This port runs every game frame at the original's frame rate (about 59.64 per second) and does not reproduce that slowdown.

## Developer and Test Options

For developer options, stage jumps (`--jump`), the built-in test menu (`--test`, `--debug-keys`), autoplay bots (`--autoplay`, `--autoplay smart`), input scripting, trace diffs, and headless benchmarking, see [run.md](../run.md).
