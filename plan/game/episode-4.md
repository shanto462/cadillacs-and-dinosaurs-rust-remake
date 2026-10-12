# Episode 4: Jack's Garage (internal stage 3)

Sources: **[R]** `re/specs/enemy/stage_scripts.txt` (STAGE 4), `re/specs/enemy/SPEC.md` 8, `re/specs/stage/SPEC.md`
4.6 (scene 3, "fortress gate" in the render notes); **[M]** MAME 2026-10-12 (story panel, card);
**[W]** web (`README.md` section 5). Captions paraphrased.
Port state in one line: **cannot start (panic); classes 25, 27, 43, 70 and the garage props are missing.**

## Summary

| Item | Value |
| --- | --- |
| Title in game | "Jack's Garage" [M] |
| Place | Outside the garage (barrels, tires, a night sea sky), inside the garage, a final hall [M card, W AQM] |
| Areas | 3 (0 outside, 1 inside the garage, 2 final area) |
| Boss | Slice (class 25, HP index 0, kill score 40000), a very fast boomerang thrower [W] |
| Time at area start | 1:20, 4:00, 2:50 [R] |

## Link scenes before this episode [M]

- **Story panel** (309 frames): a hero with a green cap (Mustapha) says "Jack, look..." and, over a small desert
  picture of the garage with smoke, "Our garage! Somebody is going to pay!".
- **Episode card**: map panel with the name plate "Jack's Garage"; "EPISODE 4 / JACK'S GARAGE" typed in; behind it
  barrels and tires, one thug with orange hair walking in.

## Story

[W] While the heroes were chasing Hogg, another poacher group took over Jack's Garage, their base. The heroes
fight through the yard and the garage to its leader Slice, "cool headed, very fast, one of Jack's old enemies"
[W AQM]. Slice later returns as a dinosaur hybrid (Episode 7).

## Scenes in order

| # | Area and camera [R] | What happens | Waves |
| --- | --- | --- | --- |
| 1 | Area 0 outside. Follow to X 0x280, area end with fade at 0x280 | 14 containers (drums, barrels, tires; E class 5, variants 2 and 0xC) at X 0x20..0xE0 and 0x2A8..0x3D0; one weapon lying on the ground (E class 0, variant 0x1C, +$5C = 2) at (0x14B, 0x52); guards: class 43 at (0x160, 0x40), three class 27 at (0xEA..0x112), Ferris x2 at 0x110, Blade/Razor x2 at 0x180. More drops at 0x1A0 to 0x210. | Wave 0 at 0x280: 4 Ferris-type bikers (3 players: more); ends with the walk to the next area (fade) |
| 2 | Area 1 inside the garage. Follow from 0, lock 0x190, lock 0x580, area end with fade 0x880 | Pool B props: class 0 x3 (at X 0x590 Y 0xB0, 0x900 Y 0xE0, 0x980 Y 0xB0), class 1 at 0x610 (garage fittings, unknown), Hammers x2 (0x100), Blade/Razor (0x101), class 70 x3 (0x360), poachers x3 (0x400..0x420), Ferris x2 (0x670..0x680), 8 containers, one treasure at 0x200. Forced aggression 7 until 0x581. | Wave 1 at 0x190: 2 Walther (class 18 var 0x200), 2 Punk/Thug, 3 Ferris, 3 class 64, 3 class 70; ends GO. Wave 2 at 0x580: no enemies, sets 3:50; GO. Wave 3 at 0x880: 7 Punk/Thug, 4 Blade/Razor; walk to area 2 (fade). |
| 3 | Area 2 final hall. Follow from 0, lock 0x100, lock 0x2C0, stage end 0x470 | Pool B class 59 x3 at 0 (lights or gates, unknown), 8 Hammer-type fighters at X 0x1A0 (var 0x404/0x204/0x4), 4 weapons on the floor (E class 0, variants 0xA, 0, 8, 8), 2 treasures, 1 drum. **Slice** (class 25) at trigger 0x3D8 (X 0x570, Z 0x20). | Wave 4 at 0x100: no enemies, GO. Wave 5 at 0x2C0: Ferris x5, Hammer x3, poachers x4, Blade/Razor x2; sets 3:50; boss area (type 4) |

## Enemies [R]

| Class | Name | Port |
| --- | --- | --- |
| A 2, 10, 14 | as before | DONE, DONE, in progress |
| A 13 (3) | Poachers | MISSING |
| A 18 (2 in wave 1) | Walther | MISSING |
| A 64, 70 | alt Punk/Wrench T., unknown | MISSING |
| A 27 (3), 43 (1) | unknown (garage guards? classes 27 0x4D900.., 43 0x55F2A..) | MISSING |
| A 25 | Slice | MISSING |
| B 0 (3), 1, 59 (3), 20, 31, 66, 69 | props, GO hint, clock, banners | MISSING |
| E 0 (5), 5 (14 + 8 + 1), 9 (3) | weapons, containers, treasure | DONE |

Weapons lie on the floor in this episode, and "firearms and extra ammo are abundant, an M-16A1 can appear in
the second area" [W AQM].

## Boss: Slice (class 25, handler 0x4BE70..0x4D48A, 5,600 bytes)

- [W] Boomerangs that he can multiply for a wide area, very fast, runs in and slashes when the player is
  pushed off screen (Namu wiki summary), "a punk with shadow attacks" (HG101). Tips: stock up on health and
  ammo, dash attacks are safe.
- [R] HP table index 0 (350..750 by players and level), score 40000. Name id 5.
- Port: MISSING, listing not decoded. D3 task.

## End and what follows

Slice defeated: slow motion, walk-off, tally, map, story panel for Episode 5 (a radio call asking for help
from a village attacked by crazy dinosaurs).

## Port state

Terrain floors: DONE (stage 3). Camera modes used here (0, 4, 1, 6) are DONE. Everything else missing; the
stage panics at start (music table).

## Open questions

1. What are classes 27, 43, 70, pool B 0, 1, 59?
2. How many containers hold the M-16, and where?
3. Slice's attack list, boomerang rules, how many HP bars.
4. Does anything special happen at the area 0 to 1 switch (a gate)?
