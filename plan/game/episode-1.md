# Episode 1: City in the Sea (internal stage 0)

Sources: **[R]** `re/specs/enemy/stage_scripts.txt` (STAGE 1), `re/specs/enemy/SPEC.md` sections 8, 9, 12,
17, `re/specs/stage/SPEC.md` section 4.6 and 6; **[M]** MAME observation 2026-10-12; **[W]** web (see
`README.md` section 5). Story captions are paraphrased.
Port state of the episode in one line: **runs, but only the first thugs, containers and items work;
no opening scene, no props, no Vice T., no tally.**

## Summary

| Item | Value |
| --- | --- |
| Name | "City in the Sea" [W ja Wikipedia, Arcade Quartermaster]. The game shows no title card for this episode. A small caption says East Coast, 2513 [M]. |
| Place | A flooded 26th century city: roof tops, a building hall, a street by the water [R stage SPEC 4.6 scenes, W AQM]. |
| Areas | 3 (0 roof tops, 1 hall inside the building, 2 the drop out of the window and the street) |
| Boss | Vice Terhune (pool A class 18, variant 0), with a dinosaur helper (Rock Hopper, a velociraptor) [W AQM]. |
| Time at area start | 1:20, 2:20, 1:20 (table 0x1033E) [R] |
| Link before | Game start (no story panel, no map card). After it: the clear tally, then the story panel for Episode 2. |

## Story

[W] In the 26th century the Black Marketeers hunt dinosaurs and the animals have gone wild. The heroes
reach the city to stop them. The poacher leader of this city, Vice Terhune, is "dim witted but strong and
fast" and sends his gangs. [M] In the opening scene a gang attacks the player's hero on a roof; the boss's voice
shows as a caption in which he says he is tired of being interfered with and tells his men to teach the
heroes a lesson.

## Scenes in order

| # | Area and camera [R] | What happens | Waves and locks |
| --- | --- | --- | --- |
| 0 | Opening scene, camera mode 0x11 (auto scroll 3 px per frame) from X 0 to 0x380 | Fade in on the city and sea. A caption with the place and year, a red pterodactyl flying over the sea [M]. The hero walks in; scripted thugs (pool A class 55, variants 3 to 5) and a bystander (class 54) run the scripted fight. Bird controllers (class 49 x3) and birds (class 50) fly in the sky. The master object is pool B class 74 (trigger 0x1C0), caption banners are pool B class 66 (several variants), the bird spawner is class 73 (trigger 0x100). HUD slides in when the walk-in script sets G.$4E3. | none |
| 1 | Area 0 roof tops. Follow from 0x380; slope down at 0x408; follow at 0x460; lock at 0x4D0; area end with fade at 0x580 | Bikers (Ferris, Gneiss, Driver; class 2) walk or dash in at 0x388, 0x410, 0x420, 0x4C0 (1 to 3 per record, 2 to 3 players add more). 4 containers (E class 5) with item rows 2, 4, 0x2A, 0x12. Forced aggression 1 until camera X 0x4E1. | Wave 0 at 0x4D0: 2 bikers dash in (the second only with 2+ players); ends with the automatic walk to the next area, which happens with a fade |
| 2 | Area 1 hall. Locks at X 0, 0x1B0; area change without fade (mode 0x12) at 0x220 | "Bladesmen" (Blade/Razor, class 14) and "fat men" (Hammer T./Blk Elmer/Wrench T., class 10) in a hall with suits of armor (containers: E class 5 variant 0xA: attackable, solid, not liftable) that drop weapons and food [W AQM]. Pool B class 12 appears at 0x1E0 (X 0x370, Y 0x30; unknown role). Forced aggression 7 until X 0x220. | Wave 1 at X 0: Ferris, class 12, Punk and Thug (class 20), ends with GO when 4/5/6 are left. Wave 2 at 0x1B0: 3 class 10, GO when 4/5/5 left. Wave 3 at 0x220: no enemies, automatic walk (fade-less). |
| 3 | Area 2. Camera mode 7 (descend with the players) from Y 0x300 (the jump out of the window), locks at (0, 0x100) and (0xA0, 0x100), stage end at 0x1D0 (mode 6) | The heroes jump out of a window and land on the street. Pool B class 14 is placed at (0x60, Z 0x120, Y 0x290) at the start of the area, and wave 4 spawns another one (role unknown; maybe the window or a falling object). Ferris, Blade/Razor, then Vice T. at trigger 0xF4 (X 0x318, Z 0x128). 3 containers (item rows 0x24, 0x0F, 0x35). | Wave 4 at X 0: Hammer, class 14 prop, Ferris; GO when none left. Wave 5 at 0xA0: Ferris (3 players only), then sets the time to 3:50 and starts the boss area (end type 4). |

(The static spawns also carry many pool B scenery objects: classes 69, 31, 20 and 66 at trigger 0 of
every area. 31 is the **time clock**, 20 the **GO hint**, 66 text banners, 69 unknown.)

## Enemies (class numbers from the script) [R]

| Class | Names | Where | Port |
| --- | --- | --- | --- |
| A 2 | Ferris, Gneiss, Driver (variant 0, 2, 4; entrances walk, dash, wait, fade, jump, drop) | all areas | DONE |
| A 10 | Hammer T., Blk Elmer, Wrench T. ("fat men") | area 1, 2 | DONE |
| A 20 | Punk, Thug | area 1 | DONE |
| A 14 | Blade, Razor ("bladesmen"; throw knives, 3 objects per record, classes 60 / 34 / 41 / 15..17 as parts) | areas 1, 2 | in progress by another agent (working tree has class14*.rs) |
| A 18 | Vice T. (boss), Walther (reinforcement) | area 2 | MISSING |
| A 49, 50, 54, 55 | sky bird controllers, birds, bystander, scripted thugs | opening | DONE |
| A 56, 57 | opening scene extras | opening | MISSING |
| B 15 | wave controller | all | DONE |
| B 73 | bird spawner | area 0 | DONE |
| B 12, 14, 20, 31, 40, 66, 69, 74 | props and controllers (see above) | all | MISSING |
| E 5 | containers (drums, barrels, armor) | all | DONE |

## Dinosaurs

Only the scenery pterodactyls (class 49 and 50, harmless) [R], and Vice T.'s velociraptor "Rock Hopper"
[W AQM, ja Wikipedia: Vice "releases an enraged dinosaur"]. The script has no pool A class 29 in this
episode: the raptor is probably created by the boss code or by pool B class 76 (spawned in the Vice intro,
role unknown). Open question for D3.

## Items

Containers drop food, weapons (Uzi, shotgun, bazooka rows), ammo and treasure through the drop rows listed
in the script (`item(+96)`); the enemy rows for this episode are decoded in `re/specs/enemy/SPEC.md` 13.3.
Port: DONE (pool E).

## Boss: Vice Terhune (class 18, handler 0x42590) [R SPEC section 12]

- HP from the boss table (index 0): 350 to 600 with 1 player, 500 to 700 with 2, 600 to 750 with 3,
  by rank level. Kill score 10000. HUD shows the boss bar.
- Intro: waits for a pool B object, spawns pool B class 76, walks to (0x317, 0x136), raises a gun with a
  muzzle flash, starts the boss music (0x0E), creates the reinforcement controller (pool B class 22).
- Combat: 13 combos (punches, heavy hit, jump attack, a gun volley with the bullet class 59, dash, run),
  rage twice (at HP <= 50 and <= 25 of 100 for 1800 frames), and a **summon** of helpers at 1/2, 1/3 and
  1/4 of his HP after a knockdown. Helpers come from timed and threshold lists (stage 1: Gneiss, Driver,
  Ferris, then hammers, class 63 and 64, blades).
- Death: `0xACEE` (kill-all, slow motion, walk-off leader). The body stays.
- Port: MISSING (only the flow side of 0xACEE exists).

## End and what follows

Boss HP 0, slow motion (task period 4), the heroes walk off to the right, the tally (README F1), the map
panel and the story panel for Episode 2 (a mentor warns about the Butcher, see `episode-2.md`).

## Port state

- Runs with `--stage 0`. Terrain, camera (modes 0x11, 0, 0x0B, 4, 1, 0x12, 7, 6), waves, spawn lists,
  items and weapons, players, HUD: DONE or PARTIAL.
- Missing: opening scene (scripted control 0x1DA10 and classes 56, 57, 74, 66), props (8 of 10 pool B
  classes), class 18, effects (dust, debris, dizzy stars), clock display, GO hint, tally, intermission.
- The harness scenarios for this stage: `stage1_jack`, `stage1_late`, `explore`, `stage1_fight`, `intro`
  (see `re/harness/README.md`).

## Open questions

1. What are pool B classes 12, 14, 40, 69, 74 and the opening classes 56, 57 exactly?
2. How does the velociraptor of the boss fight appear?
3. How does the hall (area 1) look in MAME: where are the suits of armor, how many, which drop rows?
4. Does the ground vehicle or a car show up in this episode (no)?
