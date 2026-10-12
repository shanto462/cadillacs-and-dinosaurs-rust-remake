> **Replaced by milestone M8, Episode 7 tasks M8-13 to M8-18, with the lift tasks M3-13 and M3-14. Do not start tasks from this file.** See `plan/game/design/D5-final-plan.md` (written 2026-10-13). The text below is history.

# Phase 11: stage 7 (internal stage 6)

**Goal:** stage 7 plays exactly like the original, from its first frame to its stage clear.

## Notes

Orange city, then the 56-screen elevator shaft (camera mode 5 and the scroll1 background, phase 4 task 3).

## Classes used by this stage's scripts (from `re/specs/enemy/stage_scripts.txt`)

- Pool A: 2 Ferris/Gneiss/Driver (ported in an earlier phase), 3 Lash T., 6 Bludge, 10 Hammer T./Blk Elmer/Wrench T. (ported in an earlier phase), 13 Poacher J/Skinner/Gutter, 14 Blade/Razor (ported in an earlier phase), 20 Punk/Thug (ported in an earlier phase), 63 Blade (0x1C), 65, 70, 82 Zeke.
- Pool B: 1, 15 (ported in an earlier phase), 20 (ported in an earlier phase), 30, 31 (ported in an earlier phase), 33, 41, 52, 54, 57, 61, 63, 64, 66 (ported in an earlier phase), 69 (ported in an earlier phase).
- Pool E: 0, 5 (ported in an earlier phase), 9.

Classes marked "ported in an earlier phase" are shared with earlier stages. Names come from
`re/specs/enemy/SPEC.md` section 3.2; classes without a name are not decoded yet.

## Tasks (one agent each, in order)

1. **Decode** the new classes (pool A [3, 6, 13, 63, 65, 70, 82], pool B [1, 30, 33, 41, 52, 54, 57, 61, 63, 64], pool E [0, 9]): role, states,
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
