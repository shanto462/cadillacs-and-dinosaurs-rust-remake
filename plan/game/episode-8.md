# Episode 8: The Under World (internal stage 7)

Sources: **[R]** `re/specs/enemy/stage_scripts.txt` (STAGE 8), `re/specs/enemy/SPEC.md` 3.2 (classes 84, 86..89,
91, 94), `re/specs/stage/SPEC.md` 4.6 (scene 7 "lab with specimen tanks"), `plan/phase-12-stage8-and-ending.md`;
**[M]** MAME 2026-10-12 (story panel, card, the ending); **[W]** web (`README.md` section 5). Captions paraphrased.
Port state in one line: **cannot start (panic); 19 of 20 pool B classes and 9 of 11 pool A classes are missing; the
final boss and the ending do not exist.**

## Summary

| Item | Value |
| --- | --- |
| Title in game | "The Under World" [M]; web: "Deep Down", "Deep Deep Down", ja Wikipedia "Final Battle" |
| Place | Dr. Fessenden's lab with numbered specimen tanks, sewers with flame jets, a long corridor, a cave with rolling barrels, his lair [W AQM, R scene 7] |
| Areas | 4 (0 lab, 1 sewers, 2 corridor and cave with slopes, 3 final lair). Maps: 7.0 7x1, 7.1 14x4, 7.2 20x3 screens |
| Boss | Dr. Simon Fessenden, two forms: first a Morgue-like monster in another colour, then a bigger mutant (web sources disagree: "three headed" [Wikipedia], "two headed, fire breathing" [ja Wikipedia]). Classes 84, 86 to 89 (form 1 and its parts), 94 (form 2); boss HP indices 8 and 9. |
| Time at area start | 4:00 in all areas; wave 5 sets 2:50 [R] |
| Tally timer | 3600 frames instead of 1800 (G.$508 for stage 7) [R flow.rs] |

## Link scenes before this episode [M]

- **Story panel** (313 frames): a bald scientist in a white coat, arms wide (Dr. Fessenden), says: so far you only
  saw a taste of my power; if you dare, come and see true genius.
- **Episode card**: the map panel and "EPISODE 8 / THE UNDER WORLD" in a lab hall with three glass tanks marked 01, 02,
  03, holding a green raptor, an empty tank, and a brown dinosaur.

## Story

[W] Fessenden wants to create new life forms by mixing humans and dinosaurs. His lair lies below the City in the Sea.
He is "the mastermind who turned himself into a dinosaur" [ja Wikipedia]. After his defeat he starts the self destruction
of the base (see the ending). "Bludge, Hopper and Tyrog 2 break free in the lab" [W AQM].

## Scenes in order

| # | Area and camera [R] | What happens | Waves |
| --- | --- | --- | --- |
| 1 | Area 0 lab. Lock 0x120, 0x2D0, 0x480; area end with fade 0x580 | Three pool B class 44 objects (the tanks, at X 0x30, 0x150, 0x5D0, Z 0x6C, variants 0, 0x400, 0x1400): their creatures break out; class 40 at 0x4E0 (X 0x690), class 15 wave starters; three "alt Punk/Wrench T." (class 64) with item rows 0x22, 0x12, 0x22 at 0x188. Forced aggression 7 until X 0x481. | W0 at 0x120: 4 Blade/Razor (3 players: more), then a tank (class 44) opens; GO. W1 at 0x2D0: 5 Hammers; GO. W2 at 0x480: 8 poachers; walk to area 1 |
| 2 | Area 1 sewers (map 14 x 4 screens, camera Y 0x300). Lock 0x1A0, 0x370, 0x570, 0x840, 0xA00; slow fade at 0xBC0 | The place is full of pool B objects: classes 28 x5, 29 x7, 43 x7 (at Y 0x3E0, the ceiling), 45 x6 (at Y 0x30), 40 x3 (Y 0x20..0x30), 36 x2, 37 x3, 38 x6 at X 0xA70..0xB70 in 3 rows (Z 0x300, 0x200, 0x100): flame jets, pipes, a big mechanism (unknown). "Flame throwers in the sewers, lure enemies into them" [W AQM]. Blade/Razor x5, Hammers x2, poachers x3 at fixed places | W3 at 0x1A0: pool B class 42 (Z 0x370; spawner), 4 Ferris, 2 Blade (class 63); GO. W4 at 0x370: class 42, 4 Ferris; GO. W5 at 0x570: 4 class 63; sets 2:50; GO. W6 at 0x840: class 42, 4 poachers, 4 class 64; GO. W7 at 0xA00: class 42, Ferris; walk to area 2 |
| 3 | Area 2 corridor and cave (map 20 x 3, camera Y 0x200, then slopes down at 0x100 and 0x400). Locks at X 0, 0x730, 0x900; sub-area switch at 0xCB0 | A pool B class 53 controller. Ferris, Blade/Razor, poachers, containers x4 at the start, then a march of 11 class 65 enemies (X 0x150..0x73F, one every 0x80), 2 Zeke (pteranodons, leave at X 0x72F) at 0x200 and 0x400, Hammers x4 at 0x800..0xA80. The cave has "barrels rolling down a slope" [W AQM] (probably the same props as above). | W8 at X 0: Ferris, GO. W9 at 0x730: 4 Ferris, GO. W10 at 0x900: 3 Walther + 2 Lash T., GO. W11 at 0xCB0: 4 poachers + 4 class 63; boss area |
| 4 | Area 3 lair. Lock 0x1120, "go to follow" at 0x1180 | Pool B 53, 66, 50 and 71 / 83 / 84 at trigger 0x1140 (X 0x12C0, Z 0x160); **class 94** (Fessenden) at trigger 0x10A0 (X 0x1220, Z 0x130) | none |

## Enemies [R]

| Class | Name | Port |
| --- | --- | --- |
| A 2, 10 | thugs | DONE |
| A 14, 63, 64 | bladesmen, alts | in progress / MISSING / MISSING |
| A 13 | Poachers | MISSING |
| A 3, 18 | Lash T., Walther | MISSING |
| A 65 | unknown swarm (11) | MISSING |
| A 82 | Zeke | MISSING |
| A 94 | Fessenden form 2 (name 0x25, score 8000) | MISSING |
| A 84, 86 to 89, 91 | Fessenden form 1 (0x22) and parts, Tyrog 2 (0x23) | MISSING (not in the script directly) |
| B 28, 29, 36, 37, 38, 40, 42, 43, 44, 45, 50, 53, 71, 83, 84, 20, 31, 66, 69 | lab and cave scenery, flame jets, tanks, boss starters | MISSING |
| E 5 | containers x4 | DONE |

## Boss: Dr. Fessenden

- Form 1: classes 84 and 86 to 89 (a multi-part monster: the player port has the "multi part boss link" stub 0x1128E
  for it), name id 0x22, HP index 8. Form 2: class 94, name id 0x25, HP index 9, score 8000.
- [W HG101, ja Wikipedia] Form 1 is a recolour of Morgue; form 2 is a much bigger mutant (the author of HG101 found
  it easy and "ridiculous"). He "reveals a stronger form when defeated" [W AQM]. AQM says his phases are
  "DATA UNKNOWN" (no data online). The details must come from the ROM (D3).
- Port: MISSING.

## End and what follows

`flow.rs` sets the ending flags when the last boss dies (G.$4D7, G.$4DB, G.$507 = 1) and the clear timer 3600. After
the clear tally the game goes to mode 0x1C: the ending (see `README.md` section 2 J). At the end of the ending the
ranking screen and the attract loop follow.

## Port state

Terrain floors DONE (stage 7, areas 0..2). Camera: slope modes DONE. Everything else missing, panic at start.
Ending, staff roll: MISSING.

## Open questions

1. What are the 19 pool B classes (tanks, flame jets, rolling barrels, mechanism)? How do tanks open (timer, wave)?
2. Which classes make form 1 and its parts? How the form 2 change happens (a pool B class 71 / 83 / 84 trigger?).
3. How many HP bars, which attacks (fire breath, bites), and what hazards live in the lair?
4. The sewer flame jets: damage, timing, and whether they also hurt enemies (they do in the web tips).
5. Does the ending depend on the characters chosen?
