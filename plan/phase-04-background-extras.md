> **Replaced by milestone M3 (M3-08 to M3-16: stage 2 camera and car camera, screen shake, row scroll, lift, area change details, shared tile and palette drivers). Do not start tasks from this file.** See `plan/game/design/D5-final-plan.md` (written 2026-10-13). The text below is history.

# Phase 4: background and camera extras

**Goal:** every background effect of every stage matches the original.

## Tasks (one agent each, in order)

1. **Screen shake** 0x222D6 (struct `$755A`, strength `$755C`, `cam_shake` API exists).
2. **Desert road (internal stage 2):** scroll2 task 0x22398 with the header copy 0x22AFA, the looping
   maps, the rowscroll task 0x2335A, and per-line rowscroll in the renderer.
3. **Elevator (internal stage 6, area 1):** scroll1 background 0x205C6, its camera, the 0x06D6 layer
   order, the clears around it.
4. **Tile animation and palette cycling objects:** the pool B classes that rewrite tiles (e.g.
   0x7863E) and cycle palettes (0x89746, 0x8B058, ...), per stage.
5. **All stages render check:** every area of every stage against MAME screenshots of the same
   camera positions (phase 0 tilemap and palette comparison).

## Done when

Tilemaps, scroll values and palettes match the phase 0 traces in every area of every stage.

## Status

Pending. Notes: `re/specs/stage/SPEC.md`, `HANDOFF.md`.
