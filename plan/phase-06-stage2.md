> **Replaced by milestone M6, Episode 2 tasks M6-01 to M6-17. Do not start tasks from this file.** See `plan/game/design/D5-final-plan.md` (written 2026-10-13). The text below is history.

# Phase 6: stage 2 (internal stage 1)

**Goal:** stage 2 plays exactly like the original, from its first frame to its stage clear.

## Notes

Jungle stage. Dinosaurs appear (Shivat class 24): the dinosaur AI, riding/aggression rules and dino-vs-poacher fights (class 79 hunts dinosaurs) must match.

## Classes used by this stage's scripts (from `re/specs/enemy/stage_scripts.txt`)

- Pool A: 3 Lash T., 5, 10 Hammer T./Blk Elmer/Wrench T. (ported in an earlier phase), 13 Poacher J/Skinner/Gutter, 14 Blade/Razor (ported in an earlier phase), 16, 20 Punk/Thug (ported in an earlier phase), 24 Shivat (dinosaur), 38 Mack, 63 Blade (0x1C), 78, 79 poacher variant, 80.
- Pool B: 2, 7, 15 (ported in an earlier phase), 20 (ported in an earlier phase), 27, 31 (ported in an earlier phase), 66 (ported in an earlier phase), 69 (ported in an earlier phase), 72, 80.
- Pool E: 5 (ported in an earlier phase), 9.

Classes marked "ported in an earlier phase" are shared with earlier stages. Names come from
`re/specs/enemy/SPEC.md` section 3.2; classes without a name are not decoded yet.

## Tasks (one agent each, in order)

1. **Decode** the new classes (pool A [3, 5, 13, 16, 24, 38, 63, 78, 79, 80], pool B [2, 7, 27, 72, 80], pool E [9]): role, states,
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
