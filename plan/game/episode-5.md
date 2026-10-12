# Episode 5: Village of Flame (internal stage 4)

Sources: **[R]** `re/specs/enemy/stage_scripts.txt` (STAGE 5), `re/specs/enemy/SPEC.md` 3.2, 11, 13;
`re/specs/stage/SPEC.md` 4.6 (scene 4 "savanna then rocks"), 6.6; `plan/phase-09-stage5.md`;
**[M]** MAME 2026-10-12 (story panel, card); **[W]** web (`README.md` section 5). Captions paraphrased.
Port state in one line: **cannot start (panic); 15 of 18 pool A classes and 13 of 14 pool B classes missing.
It has the largest class list of the game, with all 4 kinds of dinosaur.**

## Summary

| Item | Value |
| --- | --- |
| Title in game | "Village of Flame" (the card types it letter by letter) [M] |
| Place | A road where poachers camp, mountain sides with falling rocks, a bridge, a burning village [W AQM, HG101] |
| Areas | 2 plus a sub-area switch inside area 0 (camera mode 8 at X 0xA60, Y 0x200): area 0 road and mountain, area 1 bridge and village |
| Bosses | Walther (battle before the bridge, class 18 variant 0x200), Bludge (class 6), Morgan (class 36, HP index 4) who changes into Morgue (class 40, HP index 5) [W, R] |
| Time at area start | 3:50, 4:00, 4:00; the script sets 3:50 in waves 2, 5, 6 and adds 3:50 in wave 7 [R] |

## Link scenes before this episode [M]

- **Story panel** (439 frames, the longest): two panels. First the three heroes at a radio, one says "Hello, this is
  Jack"; an old man answers in panic that they are under attack and the dinosaurs have gone crazy, please help.
  Second panel: a hero says something has happened to them and "Let's go!".
- **Episode card**: map panel with the name plate "Village of Flame"; "EPISODE 5" and the title typed in; a
  hero stands near a camp fire with a barbecue.

## Story

[W] Poachers set a village on fire and drive its dinosaurs mad. The heroes defend the villagers. A man called
Morgan shoots an old villager, volunteers for Dr. Fessenden's experiment, and changes into a monster, Morgue
(he "volunteered to become a super strong creature" [W AQM]). [W HG101] "You witness the mutation effects
first hand".

## Scenes in order

| # | Area and camera [R] | What happens | Waves |
| --- | --- | --- | --- |
| 1 | Area 0 road, lock 0x100, lock 0x3D0, lock 0x740, lock 0x940, sub-area switch at 0xA60 | Two food items and three **torches** (E class 0, variant 0x16 = weapon type 11; probably the flaming bat, because type 11 has a glow overlay object [R phase 1] and the web lists a torch in this episode [W AQM]) lie at the start. Many thugs wait at 0..0x300 (Ferris, Poacher with variant 0x202, Driver). **Dinosaurs**: R Hopper x2 (class 29, leave at X 0x3E0) at 0x60 and 0x1C0, Zeke (class 82, pteranodon, leave at X 0x3D1) at 0xD0, Mack x2 (class 38, leave X 0x270 and 0x320) at 0x100 and 0x140, Shivat (class 24, leave X 0x400) at 0x240; poacher-dinosaur hunter (class 79) at 0x3C8. **Falling rocks** on the mountain side: pool B class 48 at X 0x810, Z 0x295 (trigger 0x670) [I]. Class 66 x3 at 0x700..0x730 (unknown). Containers (7) with a bazooka row somewhere ("bazooka in a barrel" [W AQM]). Forced aggression 7 until X 0x581. | Wave 0 at 0x100: Hammer (2+ players); GO when 1 left. Wave 1 at 0x3D0: Ferris, 3 poacher hunters (class 79), 4 Blade/Razor; GO. Wave 2 at 0x740: 2 class 66, 2 Ferris; sets 3:50; GO. Wave 3 at 0x940: 3 Hammers, 1 class 66, 2 Bludge (class 6), 2 Punk/Thug; GO. Wave 4 at 0xA60: 2 Walther, 5 Blade/Razor, 4 class 64, 5 poachers; walk to next area (the "Walther battle before the bridge") |
| 2 | Area 1 bridge and village, lock 0xCD0, follow, diagonal descent at 0xD50 (Y 0x100), follow 0xF00, lock 0xF20, stage end 0x11C0 | The bridge: 8 containers at X 0xBFA..0xCE0 with items ("an item filled bridge for arming and restocking" [W AQM]; drop rows 0x0B, 0x18, 0x0C, 0x02, 0x06, 0x12, 0x02, 0x07). The village: **fire**, many pool B objects: class 5 at 0xB00 (X 0x700), 62 at 0xB80, 10 x2, 13 x8 (one every 0x80 pixels, X 0xD20..0x1320, Y 0x100..0x200: burning huts or flames), 6 x3 at 0xF00 (X 0x800..0xA00, Y 0xA0), 70, 50. More weapons (E class 0, variant 0x16 torches, x2 with 2+ players). **Morgan encounter**: class 35 at (0x1380, Z 0x150) and class 45 at (0x1360, Z 0x100), both at trigger 0x1120 | Wave 5 at 0xCD0: 2 Hammers, 2 Punk/Thug, 1 poacher; sets 3:50; GO. Wave 6 at 0xF20: pool B class 49, Ferris, 2 Punk/Thug, Hammer, Lash T., 3 poachers, 3 class 63; sets 3:50; boss area. Wave 7 has no record in area 1 (probably started by the Morgan objects): 3 Punk/Thug, 2 Blade/Razor, 4 Hammers, 3 poachers; adds 3:50; boss area |

## Enemies and objects [R]

| Class | Name | Port |
| --- | --- | --- |
| A 2 (9), 10, 20 | thugs | DONE |
| A 14, 63, 64 | bladesmen and alt | in progress / MISSING |
| A 13 | Poachers | MISSING |
| A 3 | Lash T. | MISSING |
| A 6 | Bludge (a human dinosaur hybrid, "tongue attack and acid spit" [W AQM], HP 76..200, score 8000) | MISSING |
| A 18 | Walther | MISSING |
| A 24 | Shivat (T. rex) | MISSING |
| A 29 | R Hopper (Rock Hopper, velociraptor) | MISSING |
| A 38 | Mack (triceratops) | MISSING |
| A 82 | Zeke (pteranodon, flies in and hits) | MISSING |
| A 79 | poacher that hunts dinosaurs | MISSING |
| A 35, 45 | Morgan encounter objects (to verify) | MISSING |
| A 66 | unknown | MISSING |
| B 5, 6, 10, 13, 48, 49, 50, 62, 70, 20, 31, 66, 69 | fire, rocks, huts, bridge, controllers (unknown) | MISSING |
| E 0, 4, 5, 9 | weapons incl. torches, food, containers, treasure | DONE (torch glow overlay class 6 not ported) |

## Dinosaurs

This episode shows all four kinds that are in the game: Shivat (T. rex), Rock Hopper (velociraptor), Mack
(triceratops) and Zeke (pteranodon). Rules known from `re/specs/enemy/SPEC.md` 11 (Shivat): neutral until hit,
then hostile, calm again at 0 HP, "leave X" from the record. The other three are not decoded.
Zeke seen in MAME (spawned by a poke): a large orange pteranodon that circles above the heroes in long swoops and then dives at one hero with a shadow on the floor (13 HP at low rank). Arcade-history: "pterodactyl-like flyers must be hit by jumping" [W]. A burning torch sets enemies and dinosaurs
on fire [W AQM].

## Bosses

- **Walther** (class 18, variant 2): gun and punch fighter, HP 100..300, score 8000, in two copies in wave 4. Same
  class as Vice T. (SPEC 12). Port: MISSING.
- **Bludge** (class 6, score 8000): fast hybrid with a tongue and acid spit [W]. Not decoded.
- **Morgan** (class 36, handler 0x51692, 6,900 bytes): HP index 4, score 20000. [W] Shoots and kills the old
  villager, then drinks the extract and changes shape. **Morgue** (class 40, handler 0x53E08, 6,700 bytes):
  HP index 5, score 30000, "wild dash is extremely powerful" [W]. Torches help [W]. Not decoded; D3 task.

## End and what follows

Morgue defeated: slow motion, walk-off, tally, map, story panel for Episode 6 (a hero sees a lab picture and
shouts that the doctor is mad and creating new life forms).

## Port state

Terrain floors DONE. Camera: sub-area switch DONE, diagonal modes DONE. Panic at start. Fire effects: only the
flame object of the hit effects (pool C class 0x20) exists.

## Open questions

1. Is class 36 spawned by 35/45 or by pool B class 70/50? Which classes are Morgan's villager, shot and rocks?
2. What are pool B classes 5, 6, 10, 13, 48, 49, 50, 62, 70 exactly (rocks, huts, flames, bridge, sparks)?
3. How do the falling rocks hurt? How does fire hurt players (burn reaction 0xA/0xC exists)?
4. What exactly makes dinosaurs go hostile besides being hit (poacher shots, fire)?
5. Zeke's flight pattern: where he comes in from, how many dives, damage, and "hit only by jumping".
