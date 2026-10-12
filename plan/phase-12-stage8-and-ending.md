# Phase 12: stage 8 (internal stage 7)

**Goal:** stage 8 plays exactly like the original, from its first frame to its stage clear.

## Notes

The lab. Final boss Fessenden (classes 84, 86..89 and 94), the ending sequence and the staff roll (strings at 0x8B7EA), then the return to attract mode.

## Classes used by this stage's scripts (from `re/specs/enemy/stage_scripts.txt`)

- Pool A: 2 Ferris/Gneiss/Driver (ported in an earlier phase), 3 Lash T., 10 Hammer T./Blk Elmer/Wrench T. (ported in an earlier phase), 13 Poacher J/Skinner/Gutter, 14 Blade/Razor (ported in an earlier phase), 18 Vice T./Walther (ported in an earlier phase), 63 Blade (0x1C), 64 Punk/Wrench T. (0x1D/0x24), 65, 82 Zeke, 94 Fessenden, 2nd form.
- Pool B: 15 (ported in an earlier phase), 20 (ported in an earlier phase), 28, 29, 31 (ported in an earlier phase), 36, 37, 38, 40 (ported in an earlier phase), 42, 43, 44, 45, 50, 53, 66 (ported in an earlier phase), 69 (ported in an earlier phase), 71, 83, 84.
- Pool E: 5 (ported in an earlier phase).

Classes marked "ported in an earlier phase" are shared with earlier stages. Names come from
`re/specs/enemy/SPEC.md` section 3.2; classes without a name are not decoded yet.

## Tasks (one agent each, in order)

1. **Decode** the new classes (pool A [3, 13, 63, 64, 65, 82, 94], pool B [28, 29, 36, 37, 38, 42, 43, 44, 45, 50, 53, 71, 83, 84], pool E []): role, states,
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
