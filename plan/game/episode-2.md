# Episode 2: The Swamp Forest (internal stage 1)

Sources: **[R]** `re/specs/enemy/stage_scripts.txt` (STAGE 2), `re/specs/enemy/SPEC.md` sections 8, 11,
`re/specs/stage/SPEC.md` 4.6 and 6; **[M]** MAME 2026-10-12 (story panel, episode card, demo 1);
**[W]** web (`README.md` section 5). Captions paraphrased.
Port state in one line: **the port cannot start this episode (panic in the music table lookup), and none of
its 11 pool A classes beyond class 2/10/20 exist.**

## Summary

| Item | Value |
| --- | --- |
| Title in game | "The Swamp Forest" [M] (ja Wikipedia: "Go to the North Forest!") |
| Place | Jungle by a lake, a mud swamp with wading water, a final clearing with drums [R stage SPEC: scene 1 jungle; W AQM] |
| Areas | 3 (0 forest entrance, 1 mud swamp, 2 final clearing). Area 0 to 1 is a "sub-area" switch without reload (camera mode 8, X 0x568) |
| Mid boss / boss | Lash Terhune (class 3, mace, after the swamp) [W AQM]; the Butcher (class 61, started by class 5 at trigger 0x1A0 in area 2) |
| Time at area start | 3:20, 3:20, 1:20 [R]. Area 0 script: sets 2:50 at X 0x150 and adds 0:30 at X 0x170. |
| Vehicle | The Cadillac arrives at the start [M] (pool A class 16 at X -0x80 is probably the car, to verify) |

## Link scenes before this episode [M]

- Clear tally of Episode 1, map panel, then a **story panel** (306 frames): a portrait of an older man with a
  beard and a few lines: the Butcher is hunting in the northern woods, do not mess with him, the man is
  really crazy. [M f=1310..1606 with a poked mode 8]
- **Episode card** (about 4 s): the map panel shows a north shore; text "EPISODE 2 / THE SWAMP FOREST"
  typed in; behind it the Cadillac drives in from the left edge of the forest, the hero jumps out and
  the camera starts to follow [M].

## Story

[W] Poachers have made the dinosaurs of the northern forest wild. The heroes drive up in the Cadillac. In the
swamp forest they fight poachers waist deep in water, meet dinosaurs that sleep until hit, and end at
the camp of the Butcher, a poacher who loves killing dinosaurs and is feared by his own side.
[M demo 1] A short fight near the car: carrying a barrel, an Uzi, and a sleeping dinosaur marked with Z.

## Scenes in order

| # | Area and camera [R] | What happens | Waves and locks |
| --- | --- | --- | --- |
| 1 | Area 0, follow from X 0 to 0x568 | Arrival by car (class 16 at X -0x80), treasures hidden behind trees (E class 9: 4 records, "point items hidden behind tree trunks" [W]), two drums (E class 5), poachers (class 13) at 0x280 to 0x480 (more with 2+ players), triceratops Mack (class 38, "leave at X 0x480") twice at 0x180 and 0x280, classes 78 x2 and 80 (unknown), a sleeping T. rex Shivat (class 24, X 0x678, leave at X 0x569) at 0x480. Pool B classes 72 x2 and 80, 69, 31 (clock), 20 (GO hint). Forced aggression 7 until X 0x567. | Waves 0 and 1 (one Punk, one Hammer, GO each) have **no wave record** in the script: something else starts them (probably the car arrival). Wave 2 at 0x568: 4 poachers, 2 poacher-hunters of dinosaurs (class 79), 3 Blade (class 63 alt), 1 Hammer; ends with the automatic walk to area 1. |
| 2 | Area 1 swamp, camera mode 8 sub-area at 0x568 then y-creep at 0x620 (the camera sinks into the swamp), follow 0x7C0, lock 0x860, slope up 0xA00 (out of the mud), follow 0xA20, area end with fade 0xB80 | Wading water (the player sprites are cut at the water line, wading draw path ported), props: pool B class 2 at 0x5C0 (X 0x750), class 7 at 0x700 (X 0x500), banners (66). One container at 0xB38. Forced aggression 7 until 0xB67. | Wave 3 at 0x860: 3 poachers, 2 Hammers, ends with GO. Wave 4: no record or spawns, end type 14 (special; unknown). Wave 5 at 0xB80: Lash T. (class 3, variant 0x400), Blade/Razor, 3 poachers; ends with the automatic walk to area 2 (fade). |
| 3 | Area 2 final clearing: diagonal descent at X 0, follow 0x118, lock 0x180, follow 0x200, stage end 0x260 | 8 drums (E class 5, a mix of drop rows 0x09, 0x27, 0x29 x3, 0x19, 0x03, 0x2B: food and firearms [W AQM]), pool B class 27 (at Y 0x100), the boss trigger **class 5** at 0x1A0 (X 0x360) | Wave 6 at 0x180: 3 poachers; sets the time to 3:50; ends with "boss area" (type 4) |

## Enemies [R]

| Class | Name | Notes | Port |
| --- | --- | --- | --- |
| A 13 | Poacher J, Skinner, Gutter | riflemen (shoot from range) [W AQM], variants 0x2, 0x8, 0xC, 0x212, 0x40C... | MISSING |
| A 10, 20, 2, 14 | as Episode 1 | | DONE / DONE / DONE / in progress |
| A 63 | Blade (alt) | Blade look-alike used in waves | MISSING |
| A 79 | Hammer/Elmer/Wrench (alt), "poacher variant that hunts dinosaurs" | targets dinosaurs | MISSING |
| A 3 | Lash T. | mid boss, mace [W AQM] | MISSING |
| A 16, 78, 80 | unknown (16 is probably the car) | | MISSING |
| A 5 | boss trigger of the Butcher (class 5 code 0x38DDE..0x39272 has no name) | | MISSING |
| B 2, 7, 27, 72, 80 | props and controllers (unknown) | | MISSING |
| B 20, 31, 66, 69 | GO hint, clock, banners, unknown | | MISSING |
| E 5, 9 | drums, treasure | | DONE |

## Dinosaurs [R SPEC section 11, W]

- **Shivat (T. rex, class 24)**: starts asleep (mode MICHI: roars every 50 to 113 frames, cannot be hit). It
  wakes up if a poacher hits it, then it attacks poachers and players. Hit by a player it turns hostile
  (COLOR mode, palette flash, then enraged). At 0 HP it does not die: it turns calm and walks off (the
  ja Wikipedia says enraged dinosaurs are red and turn green when drained). It is freed when it leaves the
  screen; "leave" camera X is in the spawn record (+$5C). It never gives a score.
  Attacks: bite (14 dmg), tail swing and turn (10, knockdown), triple stomp, grab and hold (cooldown 180).
  Spec: `re/specs/enemy/SPEC.md` section 11, listing `dino_shivat_dis.txt`.
- **Mack (triceratops, class 38)**: hostile dinosaur, leave X 0x480, HP table by level. Behaviour unknown.
- **Poachers that hunt dinosaurs** (class 79): shoot at dinosaurs; keep dinosaurs from sleeping.
- Port: class 24 is MISSING (listing exists), 38, 78, 79, 80 are not decoded.

## Items

Drums in the last clearing (food, Uzi, shotgun rows), hidden treasure behind trees (E class 9: ids 0x02,
0x0E, 0x0D, 0x08), two drums at 0x1C0 and 0x200 in area 0 (item rows 0x0B and 0x06). Port: DONE (pool E).

## Bosses

1. **Lash Terhune** (class 3, variant 0x400; HP table row by name id 0xE; score 1000) [R]. Mace fighter; seen
   again as a normal enemy in later episodes (Episodes 5, 6, 7, 8). Behaviour: not decoded.
2. **The Butcher** (class 61, handler 0x58498, 8,000 bytes of code; boss HP index 1; kill score 20000) [R]. [W]
   Leaps around and tries to land on the player with his large backside, carries two knives that a player
   can knock out of his hands and use (thrown or held), his helpers are best left alone [W].
   The trigger object is class 5 (A#5 at X 0x360). It may create the Butcher and the carcass scene
   ("butchering a dinosaur's body" [W AQM]). To verify in D3.

## End and what follows

Butcher defeated: slow motion, walk-off, tally, map, story panel for Episode 3 (a blond villain says the heroes
look busy and this is a good time to take Jack's garage), then the "Hell Road" card.

## Port state

- Start: panic (`flow_area_music` needs the table at 0xA582 and 0xA5C8 extracted: add them to
  `dino-extract/src/text.rs` TABLES). With that fixed the area runs without any of its enemies or props.
- Camera: sub-area switch (mode 8) and y creep exist in `camera.rs`; the hook 0x9236C creates a pool B object with +$20 = 0x180 (class 96) that no code runs yet.
- Wading draw, ripples and splash exist (effects 0x606E partly).

## Open questions

1. Which objects start waves 0 and 1? What is class 16, 78, 80, 72, 80 (pool B), 2, 7, 27?
2. Exact Butcher behaviour (class 61), and what class 5 does.
3. How does a sleeping dinosaur wake (hit by poacher only, or a loud noise)?
4. Where do the Cadillac and the arrival scene run (script objects, scripted control)?
5. Which items are behind which trees.
