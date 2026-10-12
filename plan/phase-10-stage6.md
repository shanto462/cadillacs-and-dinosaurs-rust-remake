> **Replaced by milestone M8, Episode 6 tasks M8-01 to M8-12. Do not start tasks from this file.** See `plan/game/design/D5-final-plan.md` (written 2026-10-13). The text below is history.

# Phase 10: stage 6 (internal stage 5)

**Goal:** stage 6 plays exactly like the original, from its first frame to its stage clear.

## Notes

Mine with lamps (palette effects). Check the boss of this stage from the scripts.

## Classes used by this stage's scripts (from `re/specs/enemy/stage_scripts.txt`)

- Pool A: 2 Ferris/Gneiss/Driver (ported in an earlier phase), 3 Lash T., 6 Bludge, 10 Hammer T./Blk Elmer/Wrench T. (ported in an earlier phase), 13 Poacher J/Skinner/Gutter, 14 Blade/Razor (ported in an earlier phase), 18 Vice T./Walther (ported in an earlier phase), 22, 65.
- Pool B: 3, 4, 7, 8, 9, 15 (ported in an earlier phase), 20 (ported in an earlier phase), 31 (ported in an earlier phase), 39, 55, 56, 60, 66 (ported in an earlier phase), 69 (ported in an earlier phase), 79.
- Pool E: 5 (ported in an earlier phase), 9.

Classes marked "ported in an earlier phase" are shared with earlier stages. Names come from
`re/specs/enemy/SPEC.md` section 3.2; classes without a name are not decoded yet.

## Tasks (one agent each, in order)

1. **Decode** the new classes (pool A [3, 6, 13, 22, 65], pool B [3, 4, 7, 8, 9, 39, 55, 56, 60, 79], pool E [9]): role, states,
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
