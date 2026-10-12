> **Replaced by effect tasks spread by need: M2-13, M2-14, M2-17, M2-21, M5-04 to M5-08, M6-14, M6-22, M7-03, M8-02, M8-20, M8-28, M8-34, M9-05 (the class list is D3f). Do not start tasks from this file.** See `plan/game/design/D5-final-plan.md` (written 2026-10-13). The text below is history.

# Phase 5: effects

**Goal:** every effect object of the original (pool C) and the special pool D behave the same.

## Tasks (one agent each, in order)

1. **Effect class inventory.** All classes of the pool C dispatch table 0x237B2 with their spawn
   sites (callers of 0x4884 that set `+$20`), in `re/specs/anim/` (extends the existing spec).
2. **Port the classes** in groups (debris with the floor bounce 0x12DAC, explosions, dust and smoke,
   shells, score popups, text sparks, others), each with an `fx_spawn_*` function.
3. **Pool D** (allocator 0x48A8) and its users.
4. **Exactness pass:** pool C and D objects in the phase 0 traces match.

## Done when

Pool C and D match the phase 0 traces in runs that use them.

## Status

Pending. Done so far: hit sparks, landing splash, flames, ripples (`re/specs/anim/HANDOFF.md`).
