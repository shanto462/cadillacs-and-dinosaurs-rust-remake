# Phase 1: stage 1 complete

**Goal:** stage 1 (internal stage 0, the flooded city) plays exactly like the original from the
first frame to the stage clear screen, for 1 to 3 players.

## Classes used by stage 1 scripts (from `re/specs/enemy/stage_scripts.txt`)

- Pool A: 2 (Ferris / Gneiss / Driver, done), 10 (Hammer T., Blk Elmer, Wrench T.),
  14 (Blade, Razor), 18 (Vice T. boss, Walther), 20 (Punk, Thug), 49 / 50 (background birds).
- Pool B: 12, 14, 15 (wave controller, done), 20, 31, 40, 66, 69, 73, 74.
- Pool E: 5 (containers and the items they drop).

## Tasks (one agent each, in order)

1. **Items (enemy area).** Pool E class 5 and the item objects: food (heal), treasure (points),
   weapons with ammo, drop tables (0x24B36, 0xB6D12), pickup rules, the item list the player
   reads. Spec: `re/specs/enemy/SPEC.md` section 13.
2. **Weapons and pickup (player area).** `find_pickup` 0x5AA2, pickup state, held weapon states
   (0x201/0x202 melee, 0x203 charge, 0x206/0x207 guns, 0x209 recoil, 0x20A heavy object), ammo,
   dropping on hit. Spec: `re/specs/player/SPEC.md` section 4.9.
3. **Remaining stage 1 enemies (enemy area).** Classes 10, 14, 20, 49, 50 and the pool B props
   12, 14, 20, 31, 40, 66, 69, 73, 74 (decode their roles first).
4. **Vice T. and Walther (enemy area).** Class 18 boss logic, summons, rage, the bullet (class 59),
   the boss HP bar and the boss-defeated flow. Spec: section 12.
5. **Stage 1 flow (flow area).** Area changes (mansion, street), stage clear score tally,
   continue into stage 2 setup (stub the stage 2 content).
6. **Exactness pass.** Phase 0 comparison over a full stage 1 run (scripted inputs, 1 and 3
   players). Fix differences.

## Done when

The phase 0 comparison of a full stage 1 run shows no unexplained difference, and the boss fight
and stage clear screen match the original.

## Status

Pending. Handoff notes: `re/specs/enemy/HANDOFF.md`, `re/specs/player/HANDOFF.md`, `re/specs/flow/HANDOFF.md`.
