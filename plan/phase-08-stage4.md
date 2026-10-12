> **Replaced by milestone M7, Episode 4 tasks M7-01 to M7-06. Do not start tasks from this file.** See `plan/game/design/D5-final-plan.md` (written 2026-10-13). The text below is history.

# Phase 8: stage 4 (internal stage 3)

**Goal:** stage 4 plays exactly like the original, from its first frame to its stage clear.

## Notes

Check the areas in `re/specs/stage/SPEC.md` 4.6 (fortress gate). Slice (class 25) is a boss.

## Classes used by this stage's scripts (from `re/specs/enemy/stage_scripts.txt`)

- Pool A: 2 Ferris/Gneiss/Driver (ported in an earlier phase), 10 Hammer T./Blk Elmer/Wrench T. (ported in an earlier phase), 13 Poacher J/Skinner/Gutter, 14 Blade/Razor (ported in an earlier phase), 18 Vice T./Walther (ported in an earlier phase), 20 Punk/Thug (ported in an earlier phase), 25 Slice (boss), 27, 43, 64 Punk/Wrench T. (0x1D/0x24), 70.
- Pool B: 0, 1, 15 (ported in an earlier phase), 20 (ported in an earlier phase), 31 (ported in an earlier phase), 59, 66 (ported in an earlier phase), 69 (ported in an earlier phase).
- Pool E: 0, 5 (ported in an earlier phase), 9.

Classes marked "ported in an earlier phase" are shared with earlier stages. Names come from
`re/specs/enemy/SPEC.md` section 3.2; classes without a name are not decoded yet.

## Tasks (one agent each, in order)

1. **Decode** the new classes (pool A [13, 25, 27, 43, 64, 70], pool B [0, 1, 59], pool E [0, 9]): role, states,
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
