> **Replaced by milestone M10 (M10-01 to M10-26). Do not start tasks from this file.** See `plan/game/design/D5-final-plan.md` (written 2026-10-13). The text below is history.

# Phase 13: whole-game verification and finish

**Goal:** the complete game, from power-on to the ending, matches the original, and the project is
easy to build and run.

## Tasks (one agent each, in order)

1. **Full-game traces:** record complete MAME runs (one per character, plus a 3-player run, plus a
   run with continues and a game over) and compare them with the port from power-on.
2. **Settings:** every DIP setting (difficulty levels, lives, extend scores, coin modes, 2P/3P
   cabinet, free play, mono/QSound) in a config file, each compared with a short trace.
3. **Edge cases:** pool overflows, simultaneous hits, player-vs-player hits when they apply, the
   anti-tamper checks (0x2A5B6, 0x29ADE) behave as with a genuine ROM, 16-bit wraparounds.
4. **Performance and robustness:** 60 Hz with headroom on a laptop, no panics in long runs (fuzzed
   random inputs for hours, comparing with MAME on samples).
5. **Docs:** README with build/run steps, controls, how the port is organized, and the learning
   notes on how the original works ("how a 1993 game could be so good").

## Done when

The full-game comparisons show no unexplained difference, and a new user can build and play from
the README.

## Status

Pending.
