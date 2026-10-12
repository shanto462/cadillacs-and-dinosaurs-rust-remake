# Episode 3: Hell Road (internal stage 2)

Sources: **[R]** `re/specs/enemy/stage_scripts.txt` (STAGE 3), `re/specs/enemy/SPEC.md` 8.5 and 13.4,
`re/specs/stage/HANDOFF.md` (stage 2 controller and car mode), `re/specs/stage/SPEC.md` 4.6, 7.3;
**[M]** MAME 2026-10-12 (story panel, card, demo 2); **[W]** web (`README.md` section 5).
Port state in one line: **cannot start (panic), no car mode, no row scroll, none of its special classes.**

## Summary

| Item | Value |
| --- | --- |
| Title in game | "Hell Road" [M]; the first caption of the level is "Desert of Death" [M demo 2] |
| Place | A desert road and open desert with rock towers and cacti (scene 2 in `re/specs/stage/SPEC.md`) |
| Areas | 1 very long area: the map is 60 screens wide (X up to 0x7FF0), one camera event only (stage end at X 0x7FF0) |
| Boss | Hogg (class 31, boss HP index 2), a biker who throws grenades [W] |
| Vehicle | The Cadillac. The heroes ride it through the desert. It arrives after a RADIO item is taken [W AQM, R]. The car destroys and runs over gang members and barrels [W HG101, ja Wikipedia]. |
| Time | 3:00 at the start, then 4:00 (max); the script sets 3:50 when Hogg appears (X 0x2D00). In the car section the time over ends the drive instead of killing the players [R rank.rs 0x103FE]. |

## Link scenes before this episode [M]

- **Story panel** (250 frames): a portrait of a blond man (the Episode 4 boss type) says the heroes look busy,
  so this is a good time to take Jack's garage.
- **Episode card**: the map panel shows a canyon region; "EPISODE 3 / HELL ROAD" appears while enemies
  already wait in the desert.

## Story

[W] The Black Marketeers' men flee across the Desert of Death toward their garage hideout. The heroes
chase them. [M demo 2] At the start the heroes fight a pack of thugs and bikers on foot; a small box lies
on the ground; then the Cadillac drives in and the heroes fight on and around it while the desert scrolls.
[W] If the car is destroyed, the heroes keep fighting on foot, and "kill enemies until a shotgun drops" is
the tip for Hogg.

## Scenes in order

| # | Camera and script [R] | What happens |
| --- | --- | --- |
| 1 | Camera: stage 2 controller (0x22398), mode 0 follow, then **mode 8 car drive** while G.$518 != 0 (the camera follows the car at 8..20 px per frame). Row scroll (0x2335A) bends the road. | Start on foot at X 0..0x300: 4 bikers/thugs (Ferris, Gneiss, Driver) at trigger 0, Hammer at 0x160, Walther (class 18 variant 0x200) at 0x1A0, 20 containers (drums and tires, E class 5, many with the same drop row 0x31 and 0x28) along the whole road. |
| 2 | Triggers 0x300..0x2500 | The long scripted run: bladesmen at 0x400, poachers at 0x640 (3) and 0xE00..0xF00 (6), Punk/Thug groups at 0x7C0..0x820 (5) and 0x1910..0x1A50 (8 in a row), a Walther trio at 0xB40, four "alt" Punk/Wrench T. (class 64) at 0xD00, class 70 (unknown) at 0x1010..0x1800 (5), Hammer pairs at 0x1280..0x14E0, ten Ferris-type bikers at 0x2380..0x25C0. Pool B class 95 spawns controllers at 0x1200, 0x2500, 0x3100, 0x3300, 0x3700, 0x3A00, 0x3D00 (variants 0..0xC): car section events (unknown). |
| 3 | Triggers 0x2710..0x2AF0 | A column of 22 class 65 enemies (one every 0x30 pixels, rows in Z): probably motorbikes passing the car (class 65 is also used in Episodes 6, 7, 8) [I]. One carries a drop (row 0x31). |
| 4 | Trigger 0x2D00 (X 0x2EC0, Z 0x40) | Script sets the time to 3:50. **Hogg** (class 31) appears. |

Waves: nine wave records (B class 15 at 0x300, 0x500, 0x900, 0xC00, 0x1000, 0x1900, 0x1D00, 0x2300, 0x2700).
Most are empty (end with GO): W2 spawns 4 class 70, W3 4 poachers, W4 4 Hammers. In car mode the wave
controller ends at once with GO [R SPEC 8.2: `if car_mode_518 != 0 { return end_go }`].

## The radio and the Cadillac

- A RADIO is a pool E item (+$26 = 0xC, listed as a treasure worth 3000 points) [R SPEC 13.4]. With
  stage 3 active (`G.$4D9 == 2`) taking it calls 0x8BD24, which creates three pool B objects of class 0x58 (88).
  The effect is the kill-all (`G.$50E = 1`) and the start of the car section: pool B class 88 (0x8C06E) sets
  `G.$518 = 1` (car mode). [R phase 1 findings of task 2; SPEC 8.5]
- In car mode: enemies that leave the screen are freed (0x173C), wave controllers end at once with GO, the
  camera follows the car (mode 8, car handle G.$51A), the time over adds 3:00 and sets `G.$518 = 2` instead of
  killing the players, and many special passes (0x121D8, 0x122D8, 0x12342, 0x12454) handle the car hits.
  These passes are `not ported yet` in `combat.rs`; the player port also has a stub for the player state
  while in the car (`G.$518` / `G.$516`, 0x18A02).
- The car is probably a pool B or pool A object with players standing on it. How the players steer or hit
  from the car is **unknown**: needs D2 and D3 work with MAME (see open questions).

## Enemies [R]

| Class | Name | Port |
| --- | --- | --- |
| A 2 (17 records), 10 (5), 20 (14), 14 (3) | thugs and bladesmen as before | DONE, DONE, DONE, in progress |
| A 13 (9) | Poachers (riflemen) | MISSING |
| A 18 (4) | Walther | MISSING |
| A 64 (4) | "Punk/Wrench T." alt | MISSING |
| A 65 (22) | unknown swarm (bikers?) | MISSING |
| A 70 (5) | unknown | MISSING |
| A 31 | Hogg | MISSING |
| B 95 (7), 20, 31, 66, 69 | car events, GO hint, clock, banners | MISSING |
| E 5 (20) | drums and tires | DONE |

## Boss: Hogg (class 31, handler 0x4FCB4, 0x50FAC end; boss HP index 2; kill score 1000 in the table)

- [W] Rides a motorcycle ("cruiser/Harley"), drives fast and rough, throws grenades, has 3 vitality bars
  (HP above 100 shows as extra bars). The players ram him with the Cadillac while dodging grenades. If the fight
  is not won in the car, he flies around the screen and grunts attack from all sides (HG101).
- [R] Pool B class 31 is **not** the same as pool A class 31: pool A 31 is Hogg, pool B 31 is the clock.
  The screen-relative spawner (0xFE6A, X + camera X) is used by the car section and by its boss controller [R SPEC 8.1]; who uses it for Hogg is not checked.
- Port: MISSING. The listing is not decoded. D3 task.

## End and what follows

Hogg dies: kill-all, slow motion, tally, map, story panel for Episode 4.

## Port state

- Floors and obstacles for this stage are in `terrain.rs` (stage 2 floor list). Camera: the stage 2 controller
  (`camera.rs` line 371 comment "not ported yet"), car mode, row scroll (needs `render.rs` per line clip, main
  session) are MISSING.
- Panic at start as for all stages 1 to 7.

## Open questions

1. Start on foot or in the car? (The demo shows foot first, car after the radio.) Where is the radio, which
   drum drops it (drop row 0x31?), what happens if nobody takes it?
2. How are players placed and moved relative to the car? Do they steer? Which button does what?
3. What are classes 65, 70, pool B 95, 0x58 (88) and the car object?
4. How does the car get destroyed, and what happens to players then?
5. Row scroll details (stage 2 rowscroll struct G.$7780) and map looping.
