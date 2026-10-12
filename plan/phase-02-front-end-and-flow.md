# Phase 2: front end and game flow

**Goal:** from power-on, the port shows and plays the same screens as the original: boot, attract
mode with demo play, title, coin and credit handling, character select, join-in, continue, game
over, high score name entry.

## Tasks (one agent each, in order)

1. **Boot and system task.** Boot sequence after reset (0x41A, 0xA69A) as far as visible, the system
   task 0x9D048 (coins, credits, start, join), DIP switch settings as a config (difficulty, lives,
   extend, coin modes, 2P/3P cabinet), free play.
2. **Attract mode and title.** Flow task 0xA6CE8: title screen, story screens, demo play (the demo
   input recordings, 0x9A146 playback), high score table.
3. **Character select.** Select screen, timer, portraits and stats, 3-player handling.
4. **Join-in, continue, game over.** Joining during play, continue countdown, game over screen,
   name entry for high scores.
5. **Exactness pass.** Phase 0 comparison **from power-on**: coin, start, select, the first 600
   in-game frames; plus an attract-mode run with no input (the demo must match).

## Done when

A from-power-on trace (attract, coin, start, select, first fight) matches the original, and the
attract demo matches without input.

## Status

Pending. Notes so far: `re/specs/flow/SPEC.md`, `HANDOFF.md` (title, select, continue are stubs).
