# Episode 6: Jungle and Mine (internal stage 5)

Sources: **[R]** `re/specs/enemy/stage_scripts.txt` (STAGE 6), `re/specs/enemy/SPEC.md` 3.2 (classes 4, 72, 91),
`re/specs/stage/SPEC.md` 4.6 (scene 5 "mine with lamps", palette cycling 2.5), `plan/phase-10-stage6.md`;
**[M]** MAME 2026-10-12 (story panel, card); **[W]** web (`README.md` section 5). Captions paraphrased.
Port state in one line: **cannot start (panic); none of its 9 pool A classes except class 2 and 10, and 14 of 15
pool B classes, exist. The boss is fought three times.**

## Summary

| Item | Value |
| --- | --- |
| Title in game | "Jungle and Mine" [M] |
| Place | Ruined cabins, a mining cave lit by lamps, a raining jungle, a swamp and the end of the forest [W AQM] |
| Areas | 3 plus a sub-area switch (area 1 to 2 at X 0xAE0): area 0 cabins and mine, area 1 jungle and swamp, area 2 end of the forest |
| Boss | Tyrog, a tentacled parasite that takes over a human body; **three fights**, each stronger [W AQM, ja Wikipedia] |
| Time at area start | 2:50, 4:00, 1:20; script sets 3:50 (waves 0, 7) and 2:50 (wave 5) [R] |
| Hazards | A giant brontosaurus whose legs crush whoever stands under them at two places [W arcade-history, AQM, HG101]; rain; mine lamps |

## Link scenes before this episode [M]

- **Story panel** (368 frames): a hero with a short shout (the doctor is mad and is creating new life forms)
  under a picture of an experiment table with a creature.
- **Episode card**: "EPISODE 6 / JUNGLE AND MINE" under the map panel; a stone ruin wall and a sandy floor.

## Story

[W] The heroes find that the doctor behind the poachers, Dr. Fessenden, is breeding new creatures. In the old
mine and the jungle they meet the first of his creations. Tyrog "possesses a human host" (here a gangster),
revives after each defeat, and is finally destroyed on the third defeat [ja Wikipedia].

## Scenes in order

| # | Area and camera [R] | What happens | Waves |
| --- | --- | --- | --- |
| 1 | Area 0 cabins and mine: lock 0x200, lock 0x730, area end with fade 0x780 | Blade/Razor x4 at 0x80..0x180, containers (7, drop rows 0x1B, 0x29 x4, 0x15, 0x0F), Ferris, poacher, a treasure at 0x380, pool B class 3 at 0x1C0 (X 0x350), 60 at 0x200, banner 66; Walther (class 18 var 0x200) and Lash T. (class 3) at 0x390 (the mine "enemies": Lash Terhunes and Walthers [W AQM]). Forced aggression 7 until X 0x8A0. | Wave 0 at 0x200: pool B class 4 x2 (unknown spawners, probably mine cart or door) and 6 Ferris-type; sets 3:50; GO. Wave 2 at 0x730: 4 Hammers, Lash T., Walther, class 4, 3 poachers; walk to area 1. (Wave 1: 2 Hammers, no record in the script.) |
| 2 | Area 1 jungle and swamp: lock 0x300, lock 0x8C0 (Y 0x100), slope down 0x930, follow 0x970, sub-area switch 0xAE0 | A treasure at the start; **class 22 x2** (var 0, 1) at 0x520 (X 0x6C0) and 0x7B0 (X 0x950): the two brontosaurus leg spots [I from the web tips]; 6 class 65 enemies at 0x180..0x280 (3 + 3 in a row, items 0x15, 0x29, 0x1B, 0x29, 0x28, 0x29); 6 poachers with variants 0x12 / 0x212 / 0x412 at 0x200..0x800 (rain jungle ambush entrance 0x12 = ?); pool B props 3, 9 x3 (at 0x00, 0x1C0, 0x380, Y 0x100), 56, 55 (0x80, 0x100), 8 (0x8C0), 7 (0xA00): rain, vines, swamp. | Wave 4 at 0x300: 2 Blade/Razor; GO. Wave 5 at 0x8C0: 4 Hammers; sets 2:50; GO. Wave 6 at 0xAE0: 3 poachers, 2 Bludge, 4 Hammers; walk to area 2 |
| 3 | Area 2 end of forest: slope 0xCA0, follow 0xD20, diagonal climb 0xE00 (Y 0x170), lock 0xF00, stage end 0x1000 | Pool B class 39 at 0xD80 (X 0xF00, Y 0x110), a container at 0xD00 (item 0x0C), pool B class 79 at 0xF80 (X 0x1120, Z 0x190) which very likely starts the boss | Wave 7 at 0xF00: 1 Ferris, then sets 3:50, boss area (type 4) |

## Enemies [R]

| Class | Name | Port |
| --- | --- | --- |
| A 2, 10 | thugs | DONE |
| A 14 | Blade/Razor (4 static, waves) | in progress |
| A 13 | Poachers | MISSING |
| A 3 | Lash T. | MISSING |
| A 18 | Walther | MISSING |
| A 6 | Bludge (2 in wave 6) | MISSING |
| A 22 | brontosaurus leg (probable) | MISSING |
| A 65 | unknown swarm | MISSING |
| B 3, 4, 7, 8, 9, 39, 55, 56, 60, 79, 20, 31, 66, 69 | scenery, spawners, boss starter (unknown) | MISSING |
| E 5, 9 | containers, treasure | DONE |

## Boss: Tyrog (classes 4, 72 and 91)

- Name table ids: 0x19 TYROG (score 60000), 0x1B TYROG (score 100000), 0x23 TYROG 2; boss HP index 6, and the
  alternate row 0 for the other form [R SPEC 3.2, 3.3, 3.4]. Class 4 (handler 0x36A64) is 9,000 bytes of code (for
  comparison Vice T. 11,000, the Butcher 8,000). Class 72 (0x5DD58) is 1,900 bytes and class 91 (0x654E2) is "Tyrog 2", the form
  that comes back as a normal enemy in the lab of Episode 8 [W AQM].
- [W AQM] Attacks: rib attacks and poisonous acid spit. After a defeat it changes colour, green to red, then red
  to blue, and comes back. [W HG101] "fight the same boss three times in a row", the later fights harder.
- Port: MISSING; the listing is not decoded. D3 task, one of the longest.

## End and what follows

Third Tyrog defeat: slow motion, walk-off, tally, map, story panel for Episode 7 (a hero thinks he knows the
man on a picture, Dr. Fessenden, and says the equipment looks like that of the underground ruins).

## Port state

Terrain floors DONE. Camera modes used: 0, 4, 1, 0x0B, 8, 0x0A, 2/3, 6: DONE. Palette cycling for lamps
(pool B classes that rewrite palettes 0x89746, 0x8B058) MISSING. Rain: probably a pool B class + palette tricks.

## Open questions

1. What are classes 4, 7, 8, 9, 39, 55, 56, 60, 79 (pool B), 22, 65 (pool A)? Which one is the rain?
2. How the three Tyrog fights are chained (one class, three classes?). Which colours and which hit points.
3. How does the brontosaurus leg hurt players and enemies (damage, area, timing)?
4. What does ambush entrance 0x12 of the poachers do?
