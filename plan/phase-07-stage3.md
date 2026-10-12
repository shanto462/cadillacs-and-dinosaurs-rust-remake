> **Replaced by milestone M6, Episode 3 tasks M6-18 to M6-32, with the camera tasks M3-08 to M3-12. Do not start tasks from this file.** See `plan/game/design/D5-final-plan.md` (written 2026-10-13). The text below is history.

# Phase 7: stage 3 (internal stage 2)

**Goal:** stage 3 plays exactly like the original, from its first frame to its stage clear.

## Notes

The desert road (looping maps and rowscroll, phase 4 task 2). The stage boss is Hogg (class 31, boss index 2), the car stage. Check whether players drive the Cadillac in this stage and port that mode (player area) if so.

## Classes used by this stage's scripts (from `re/specs/enemy/stage_scripts.txt`)

- Pool A: 2 Ferris/Gneiss/Driver (ported in an earlier phase), 10 Hammer T./Blk Elmer/Wrench T. (ported in an earlier phase), 13 Poacher J/Skinner/Gutter, 14 Blade/Razor (ported in an earlier phase), 18 Vice T./Walther (ported in an earlier phase), 20 Punk/Thug (ported in an earlier phase), 31 Hogg (boss), 64 Punk/Wrench T. (0x1D/0x24), 65, 70.
- Pool B: 15 (ported in an earlier phase), 20 (ported in an earlier phase), 31 (ported in an earlier phase), 66 (ported in an earlier phase), 69 (ported in an earlier phase), 95.
- Pool E: 5 (ported in an earlier phase).

Classes marked "ported in an earlier phase" are shared with earlier stages. Names come from
`re/specs/enemy/SPEC.md` section 3.2; classes without a name are not decoded yet.

## Tasks (one agent each, in order)

1. **Decode** the new classes (pool A [13, 31, 64, 65, 70], pool B [95], pool E []): role, states,
   tables, spawn variants. Add the findings to `re/specs/enemy/` (one section per class).
2. **Port** the new pool A classes, normal enemies first, then the boss.
3. **Port** the new pool B and pool E classes (props, controllers, scenery, items).
4. **Stage specifics** listed in the notes above (special mechanics, scripted scenes, area changes).
5. **Exactness pass:** phase 0 comparison of a full run of this stage (1 and 3 players), started
   from a snapshot at the stage's first frame. Fix differences.

## Done when

A full run of the stage matches the original in the phase 0 comparison.

## Status

Pending.
