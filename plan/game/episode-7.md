# Episode 7: The Vault (internal stage 6)

Sources: **[R]** `re/specs/enemy/stage_scripts.txt` (STAGE 7), `re/specs/stage/SPEC.md` 4.6, 6.6 (mode 5
elevator ride), `re/specs/stage/HANDOFF.md` item 4, `plan/phase-11-stage7.md`; **[M]** MAME 2026-10-12
(story panel, card); **[W]** web (`README.md` section 5). Captions paraphrased.
Port state in one line: **cannot start (panic); the elevator background is not ported; none of the boss
setup exists.**

## Summary

| Item | Value |
| --- | --- |
| Title in game | "The Vault" [M]; web: "The Basement" (ja Wikipedia: "Going Underground") |
| Place | An orange coloured city street in front of the vault, a long elevator shaft, a library on basement level 40, a computer lab [R scene 6, W AQM] |
| Areas | 4 (0 outside, 1 elevator shaft (vertical, 56 screens), 2 library, 3 computer lab) |
| Boss | Slisaur: 2 for one player, 3 for two or three players [W AQM, ja Wikipedia]; pool A class 71, score 70000 each |
| Time at area start | 4:00, 2:50, 3:30, 1:20; waves in the shaft set 1:50 (three times) and 3:50 [R] |

## Link scenes before this episode [M]

- **Story panel** (368 frames): a hero with a green cap (Mustapha) says the man on the picture looks familiar, he is
  the genius Dr. Fessenden, and the machine looks like the staff in the underground ruins; the picture shows a
  brain-like device in a white lab.
- **Episode card**: the map panel and "EPISODE 7 / THE VAULT"; behind it an orange coloured stone street with two
  poachers waiting.

## Story

[W] The trail leads to a hidden vault in the City in the Sea. The poachers guard it. The heroes ride a lift down
many floors, fight in a library full of bookshelves, and reach the computer lab where Slice, now changed by
Dr. Fessenden into a dinosaur man, waits as "Slisaurs" for revenge on Jack [W AQM].

## Scenes in order

| # | Area and camera [R] | What happens | Waves |
| --- | --- | --- | --- |
| 1 | Area 0 outside the vault. Lock 0x1B0, 0x580, 0x790, 0x930; area end with fade 0x9E0 | Ferris-type thugs x5 at start, Zeke (class 82, pteranodon, leave at X 0x480) at 0x80, Blade/Razor x5, class 70 x2, Punk/Thug x6 at 0x388 (entrance 0xA: drop from above; some carry item row 0x1C), poachers x4 at 0x600..0x6A0, containers x6 (rows 0x0F, 0x21, 0x06 or 0x2C by rank, 0x25, 0x10), a weapon at 0x590 (variant 0x1C), treasures x2 | W0 at 0x1B0: Blade/Razor + 3 class 70, GO. W1 at 0x580: 5 Hammers, sets 3:50, GO. W2 at 0x790: 6 poachers, GO. W3 at 0x930: 6 Ferris, 3 Hammers, 4 Blade (class 63), 2 Blade/Razor; walk to area 1 |
| 2 | Area 1 elevator shaft. Camera is vertical (events trigger on Y): mode 5 elevator ride from Y 0x3700 (sound 0x86, vertical speed and acceleration, strength 2 screen shake), locks at Y 0x3000, 0x1E00, 0x1400, 0x800, area end with fade at Y 0. Scroll 1 is used as the background of this area (0x205C6). | The lift goes down; at each stop a group rides in: Ferris, Blade, Hammer, poachers in records triggered by Y (0x3700, 0x2F00, 0x2E00, 0x2D00, 0x2C00, 0x1C00, 0x1B00, 0x1900, 0x600). Pool B class 64 x4 and 57 at Y 0x4000 (lift parts and doors, unknown). | W4 at Y 0x3000: 6 Ferris + pool B class 54; sets 3:50. W5 (no record; poachers x4; sets 1:50). W6 at 0x1E00: 6 Punk/Thug + 2 Hammers; sets 1:50. W7 at 0x1400: 5 poachers; sets 1:50. W8 at 0x800: 4 Ferris + Hammer; sets 3:50. W9 at Y 0: walk to area 2 |
| 3 | Area 2 library, basement 40. Locks at X 0, 0x1D0, 0x640; area end with fade 0x700 | Bookshelves hide enemies [W AQM]: class 65 x10 (items 0x22, 0x03, 0x29, ...), Bludge x3 (class 6) at 0x580..0x5C0, Ferris x3; **rolling drums with items** (pool B class 1 at X 0xE0 and X 0x7C0) [W AQM]; 3 treasures | W10: 4 Ferris, GO. W11 at 0x1D0: 3 Punk/Thug, 6 poachers, 3 Lash T.; sets 2:50. W12 at 0x640: pool B class 52, 3 Ferris, 3 class 65; walk to area 3 |
| 4 | Area 3 computer lab. Follow from 0, lock 0x240, lock 0x450, stage end with fade 0x4F0 | 5 containers (rows 3, 0x1F, 4, 0x32 / 0x33 by rank), 6 Hammers at 0x100..0x180 (variants 8, 0x208, 0x408), then the boss setup: pool B classes 61, 63, 41 x3 (at X 0x430; variants 0, 4, 8: probably one per Slisaur, shown by player count), 30 at (0x4B0, Y 0xD0), 33 at 0x3C0 | W13 at 0x240: 4 Blade/Razor; sets 3:50; boss area (type 4) |

## Enemies [R]

| Class | Name | Port |
| --- | --- | --- |
| A 2, 10, 20 | thugs | DONE |
| A 14, 63 | bladesmen | in progress / MISSING |
| A 13 | Poachers | MISSING |
| A 3 | Lash T. | MISSING |
| A 6 | Bludge | MISSING |
| A 65, 70 | unknown swarm and group | MISSING |
| A 82 | Zeke (pteranodon) | MISSING |
| A 71 | Slisaur (created by a pool B class) | MISSING |
| B 1, 20, 30, 31, 33, 41, 52, 54, 57, 61, 63, 64, 66, 69 | doors, lift, boss starters, GO hint, clock, banners (unknown) | MISSING |
| E 0, 5, 9 | weapon, containers, treasure | DONE |

## Boss: the Slisaurs (class 71, handler 0x5C818, 5,400 bytes)

- [W] Created when Fessenden turned Slice into a man-dinosaur seeking revenge on Jack. "Essentially the Episode 4
  boss, doubled" and the best boss fight in one source [W]; two for one player, three for 2 or 3 players.
- [R] Boss HP table index 0; kill score 70000 each; name id 0x18 SLISAUR. Pool B 41 x3 matches the "up to
  three" rule (records carry conditions in the flag word).
- Port: MISSING; not decoded. D3 task.

## End and what follows

All Slisaurs down: slow motion, walk-off, tally, map, then the Fessenden story panel ("so far you only saw a
taste of my power; come and see true genius").

## Port state

- Terrain: the vertical scene map exists (`terrain.rs`). Camera mode 5 (ride) DONE; the scroll 1 background
  (0x205C6, tables 0x138C48, 0x15CFE8) is MISSING; the player hook for the lift (G.$4DC vertical scene) and
  `enemy_despawn_if_far` Z+Y test are listed as TODO in `plan/phase-01-stage1-complete.md` (phase 11).
- Panic at start.

## Open questions

1. How are the lift (pool B classes 64, 57, 54) and its doors made? Does the floor move, or only the camera?
2. Is the library "rolling drums" really pool B class 1? What do the shelves do (enemies jump out)?
3. Which of the 3 Slisaur starters appear for 1, 2, 3 players? What are classes 30, 33, 41, 52, 61, 63?
4. Zeke's flight in an outdoor area right at the start.
