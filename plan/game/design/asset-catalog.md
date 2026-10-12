# Asset catalog: what is really in the ROM set

Written 2026-10-12 by task X0. Source: every asset of `dino.zip` was extracted and rendered with the dump tool
(`re/tools/dump_assets.sh`, code in `re/tools/dump/`). Nothing from the ROM is in this file: only descriptions, counts and
addresses. The pictures, sounds and text dumps are in `re/out/assets/` (git-ignored). Open `re/out/assets/index.html` in a
browser to look at everything (plain HTML, local files only).

Tags: **[O]** observed in the extracted data or in the code (address given), **[I]** inferred from pictures or code shape,
**[U]** not known yet. Addresses are 68000 ROM addresses unless said. "G.$xxxx" is RAM `0xFF8000 + xxxx`.

## 0. How to run it, and what was not done

- One command from the repo root: `re/tools/dump_assets.sh` (about 45 seconds, needs `dino.zip` in the root or `DINO_ROM`).
  It reads the zip at run time and writes only to `re/out/assets/`. **No emulator is started** (the user asked to keep MAME out of it).
- Python tools in `re/tools/dump/` (run with `re/.venv/bin/python -I`): `romio.py` (zip loader, tiles, palettes, Kabuki), `scan.py` and
  `catalog.py` (finds every animation table and its owner), `render.py`, `dump_comps.py`, `dump_anims.py`, `dump_stages.py`,
  `dump_gfx.py`, `dump_text.py`, `sequences.py` + `dump_sound.py`, `build_index.py`, `dump_all.py`. The Rust workspace is unchanged
  (`cargo build` and `cargo test` are green).
- The 1273 front end screenshots in `re/out/assets/frontend/` were taken **once** from MAME before the user asked to stop using it
  (13 power-on runs with the recipes of `plan/game/NEW-PLAN.md` section 5). The default dump only indexes them. The optional step
  `dump_all.py --with-mame` (`dump_frontend.py`, one process, dummy SDL drivers) would repeat them; it is **not tested** in its
  final one-process form.
- **No audio of music or composed effects exists yet.** It comes later from our own sound driver (milestone M4). What exists:
  the 124 raw QSound samples as WAV, the decoded command table, and a text listing of every music track and effect.

## 1. New findings (the plan does not know these yet)

Cross-checked with `plan/game/mechanics-seen.md` (IDs in brackets) and `plan/game/README.md`.

| # | Finding | Evidence | Plan state |
| --- | --- | --- | --- |
| N1 | **The ROM holds art that no animation table draws (319 compositions, 9 percent).** Code picks them with computed indexes. Part of it is clearly scenery, hints and unused actors (list in section 4.4): chain-link fence pieces with a broken piece (breakable scenery?), a red "BREAK!" hint banner with an arrow, a "Look out!" speech banner, a rotating spiked ball (71 frames), "STAGE CLEARED!" and the numbers 1 to 8, "1P / 2P / 3P" tags, GO style arrows in three more directions, a joystick and a button picture, round barrel lids, red pillars, chains, a manhole. | `comps/unreferenced/sheet_01..08.png`; `data/comps.json` (`referenced: false`) | W25 and W26 (breakable background parts, "many things") are UNKNOWN. These pictures are the first hard evidence. W23 knows only the right edge "GO" arrow. |
| N2 | **Unused or never-scripted creatures**: a golden velociraptor (class A12), a small orange raptor pack (A28), small lizards (A47), a long tailed bird (A46), a second pterodactyl (A37), a blue crested dinosaur in two colours (A51, no script spawns it), a robed old man pointing (A39), a sleeping or lying triceratops picture (unreferenced composition 2416), a big green raptor picture and a muscular gold man picture (compositions 2347 and 2366). The scripts of the 8 episodes never spawn A12, A28, A37, A39, A46, A47 or A51. | `anims/actor_a12`, `actor_a28`, `actor_a37`, `actor_a39`, `actor_a46`, `actor_a47`, `actor_a51`; `data/anims.json` | E24 (classes with unknown role) lists only some of them. D3b should check whether other classes spawn them (A28 and A47 may be children of a dinosaur). |
| N3 | **Six name strings that no name id points to**: a "boom" label, "kimo 2", "knife 3", "devil", "tyre", "widget". They look like test or cut objects. Addresses 0x17E13, 0x17E97, 0x17EB8, 0x17EC3, 0x17ECE, 0x17F73. | `text/names_table.txt` | New. Probably developer leftovers; no art found for "devil" or "widget". |
| N4 | **The anti-drug warning (FBI style) picture is in the World ROM but skipped.** Routine 0x98330 draws scroll3 pages 0x3E and 0x3F, and jumps over them when G.$32 = 2 (boot code sets G.$32 = 2 at 0xA736, so the World set skips it [I]). The boot warning text also has three region versions (JAPAN, USA, ETC) at 0x9DE45, 0x9DEB1, 0x9DF33. The previous notes called 0x98330 the "ranking pause picture"; the ranking background is something else. | `stages/scenes/scroll3_screens_056_063.png`, `text/ascii_strings.txt`, code 0x98330 | Corrects `attract-and-front-end.md` section 3 item 3 and open question 1. Not needed for the port (World set), keep as a documented skip. |
| N5 | **The pictures of the title, story slides, opening movie, continue and ending are plain data** (scroll3 pages 0x60 to 0x7F, scroll2 scene pages 0x01 to 0x1B, scroll1 pages 1 to 6), not objects. 35 of the 128 scroll3 pages are used by no stage. A table of call sites with page ids is in `stages/scenes/picture_calls.txt` (103 calls). Examples: title logo pieces (0x73, 0x74, 0x67, 0x6F and scroll2 0x14, 0x15), continue screen villain (scroll2 0x18 to 0x1A, scroll3 0x75, 0x76 called from 0xEA74 to 0xEA9E, part of 0xE9E6), hero group picture of the select screen (scroll2 0x01, 0x02 called at 0xADE0 and 0xADEE), jungle pictures (0x64, 0x71, 0x72, probably the opening movie), story slide skylines and dinosaur head (0x65, 0x66, 0x68, 0x69, 0x6A, 0x6B), villain face (0x6D, 0x6E), ending desert (0x6C, 0x78 to 0x7C), scroll1 select panel frames (pages 3 to 6, 1P / 2P / 3P). | `stages/scenes/`, `data/stages.json` | `attract-and-front-end.md` section 3 lists "picture data to extract" as open. They can be rendered by the importer without MAME. The portraits and poses of the profile cards were not found as pages (probably sprites). |
| N6 | **Test mode content is richer than noted**: memory tests (EEPROM, QSound RAM, CPS buffer RAM, work RAM, scroll RAMs, object RAM), viewer pages with H-flip and colour selectors [I], a scroll layer viewer (scroll2 only, scroll3 only, both, synchro modes), difficulty menus with 8 levels and 4 levels, coin menus (1 to 6 credits, 2 to 4 coins, free play, "2 coins start / 1 coin continue"), three player or two player chute modes, mono or QSound output, extra life menu (300000 / 500000 / none), a "PARAMETER CHANGE" debug page and the hit box editor strings. 82 strings in all. | `text/ascii_strings.txt` region "test mode menus" | F13 and F14 (settings menu, test mode) MISSING, "decide to skip or not". This gives the full list. |
| N7 | **A debug word list for enemy AI states (romaji)**, about 64 strings at 0x32FD9 to 0x33333, and "CADI SPEED" at 0x502C4 (car stage debug). | `text/ascii_strings.txt` | Known to the enemy spec (names of states); here they are decoded in one list. |
| N8 | **Sound command table has 243 live codes (39 music, 204 effects) and 19 empty entries** (0x27 to 0x2B, 0x92 to 0x9F). Effects have priorities 3 to 10 (108 with 7, 58 with 5, 32 with 3, 4 with 4, one each with 6 and 10). Music codes 0x00 to 0x26: stage music per area (0xA582), boss music 0x0E to 0x12 (0xA5C8), codes 0x13 to 0x16 are the story panel tunes (sent from 0xD8EA, 0xD980, 0xDAB6, 0xDBB4, 0xDDCE, 0xDE6E, 0xDF20), 0x17 is Episode 5 area 0. Music codes 0x02 and 0x1A are copies of each other (same samples, same length). 3 of 124 samples (28, 29, 123) are used by no sequence. Total 99,729 notes. | `sound/sound_codes.txt`, `sound/sound_sequences.txt`, `sound/sample_table.txt` | S01 says the music table 0xA582 is "not even extracted". The whole table with all 39 tracks is now decoded as text. 133 codes have no direct sender found (they go through wrapper tables like the hit sound pairs at 0x96AA or computed codes): D4e must trace them. |
| N9 | **Palettes**: 512 OBJ palettes in ROM, only 85 are listed by the stage OBJ sets, 54 more are pointed at by code (fades and flashes), 373 are referenced by nothing found. 22 scroll2 banks (6 never referenced: 0xBEA52, 0xC0A52, 0xC0E52, 0xC1252, 0xC1A52, 0xC1E52) and 14 scroll3 banks (0xC4252 never referenced). Slots 8 to 19 of the scroll palette tables belong to scenes, not stages. | `palettes/`, `data/gfx.json` | New. Unreferenced banks may be unused colour variants. |
| N10 | **1244 sprite or scroll2 tiles (16x16) are drawn by no composition, stage map or scene page.** Most are big font glyphs (tall letters and digits for the title and tally text), the rest are scenery pieces. | `gfx/tiles16_unused/` | New. The big font is not covered by the text printers noted so far. |
| N11 | **Cadillac and car art**: a blue open car (class A16), a red car with a woman driving (class A81, two frames) and a red hot rod picture (composition 2892). Only the blue car is spawned (Episode 2 script). A81 is never spawned by a script. | `anims/actor_a16`, `actor_a81` | W15 (car section) knows the car state, not these actors. |
| N12 | **Heroes have 411 animations in 24 tables** (6 sets per hero; set 0 has 33 ids for each hero, the weapon pose set 5 has 31, 32, 27 and 29 ids for Jack, Mustapha, Hannah and Mess). Two ids of Jack's and Mustapha's set 0 point into garbage and are kept empty. | `anims/hero_*` | Already in `re/specs/anim/SPEC.md`; here every one is a GIF with hit boxes. |
| N13 | **Pool B props almost never draw through animation tables**: only 7 of about 97 classes have one (B25, B65, B69, B70, B77, B86, B88). The scenery, banners and hazards of pool B draw compositions chosen by code (index = base + variant) or rewrite scroll tiles. That is why the 319 unreferenced compositions exist and why `W24` (57 unported pool B classes) cannot be read from tables alone. | `data/anims.json`, `comps/prop_*`, `comps/unreferenced` | W24, D3e: needs a per class code trace. |
| N14 | **Camera events**: 116 records in the 8 stages (stage 6 area 1, the elevator, triggers on Y). Modes used: 47 locks (mode 4), 24 plain follow (0), 12 diagonal descents (2), 9 end of area with fade (1), 5 stage ends (6), 4 sub-area switches (8), 4 slope downs (0xB), and one use each of modes 3, 5, 7, 9, 0xA, 0xC, 0xD, 0x10, 0x11, 0x12 and 0x13. All shown on `stages/epN_aM_scroll2_events.png`. | `data/stages.json` | Matches `re/specs/stage/SPEC.md`; first picture of where each event sits. |

Things the plan already had and the data **confirms**: 8 episodes with 22 areas (3 sub-areas that keep the previous map: 1.1, 4.1, 5.2),
the elevator cage drawn on scroll1 (stage 6 area 1), 4 hero palettes in OBJ slots 0 to 3, story captions and episode titles as tall
font strings (7 episode titles, about 40 story lines).

## 2. What is there: counts

| Kind | Count | Where in `re/out/assets/` |
| --- | --- | --- |
| Graphics ROM | 4 MB = 32768 tiles of 16x16, 8192 of 32x32, 65536 8x8 codes (only 4096 are in the scroll1 range) | `gfx/` (86 sheets: 16x16 colour and grey, unused 16x16, 8x8 in 3 palettes, 32x32) |
| Palettes | 512 OBJ ROM palettes, 22 OBJ sets (one per episode and area), 1 scroll1 bank, 22 scroll2 banks, 14 scroll3 banks, 6 scene palette sets | `palettes/` (63 swatch images), `data/gfx.json` |
| Sprite compositions | 3513 (3194 drawn by tables, 319 not) in 101 groups | `comps/<owner>/sheet_NN.png` (157 sheets), `data/comps.json` |
| Animation tables | 272: 243 tables, 17 direct frame chains, 12 raw sprite list chains; 2884 ids, **2656 animations**, 11022 frames | `anims/<owner>/TTTTTT_II.gif` and `_strip.png` (72 MB), `data/anims.json` |
| Stage maps | 8 episodes, 22 areas, 19 own maps (scroll2) and 16 far layers (scroll3) | `stages/` (plain, events marked, views), `data/stages.json` |
| Scene pages | 128 scroll3 pages (93 used by stages), 33 scroll2 scene pages, 7 scroll1 picture pages | `stages/scenes/` |
| Front end screenshots | 1273 in 13 scenarios (attract loop, start, tally, ending, continue, time over, episodes 2 to 8 link scenes) | `frontend/` (contact sheets `*_sheet_NN.png` and one PNG per frame) |
| Text | 362 ASCII strings, 109 tall font strings (story, titles, captions) with pictures, 43 names | `text/` |
| Sound | 243 codes (39 music, 204 effects), 124 samples (1.96 MB), 126 instruments in use | `sound/` |

Animations by owner kind: heroes 24 tables and 411 animations, enemies 51 and 673, bosses 30 and 383, dinosaurs 14 and 181,
unnamed pool A actors (props, projectiles, scenery creatures) 69 and 328, items 34 and 417, effects 43 and 249, props 7 and 14.

## 3. Graphics and palettes

- **Tiles [O].** 16x16 sprite tiles `0x0800` to `0x6FFF` (codes below `0x800` are the scroll1 8x8 glyph area, `0x7000` and up are the
  scroll3 32x32 tiles seen as pieces). Scroll2 map tiles share the range `0x4000` to `0x6FFF` with sprites. Colour sheets use the
  palette of the first known use (composition owner or map).
- **Palette tables [O].** OBJ: list per (episode, area) at `0x8E8E`, 32 slots each, colours at `0xB7A52 + 32 * n` (512 palettes).
  Scroll1 bank `0xBBA52`. Scroll2 banks `0xBCA52` to `0xC2252` (22), scroll3 banks `0xC2252` to `0xC5A52` (14). Scene sets by
  `0x89E6` (6 sets). All colours have brightness 0 in ROM; the game ORs `0xF000`. Swatches are shown at full brightness.
- **Unreferenced banks and palettes**: see N9.

## 4. Sprites: owners and what they are

### 4.1 How owners were found

A table candidate is any 24 bit value in the program (immediate, data pointer, `lea d16(pc)`) that passes a strict check: offsets are
valid frame record chains of 10 or 12 bytes (`re/specs/anim/SPEC.md` section 3). That found 237 tables. The developer list at
`0xB7470` (187 entries: a table pointer and two small numbers each, probably the hit box editor's list) adds 6 more that need a
tolerant check (a few ids point into garbage). The owner is the object class whose code range holds the reference: dispatch tables
`0x233F0` (pool E, 12), `0x2346C` (pool A, 102), `0x2362E` (pool B), `0x237E6` (pool C, 42). Hit reaction and held pose tables are
found through the name id tables `0x2E68C` and `0x2E724`. Box tables come from `move.l #box,$44(An)` near the reference.

### 4.2 Pool A: enemies, bosses, dinosaurs and actors (episodes = scripts that spawn the class)

| Class | Names or what the art shows | Tables | Anims | Episodes |
| --- | --- | --- | --- | --- |
| A02 | Ferris, Gneiss, Driver (thugs) | 9 | 102 | 1, 3 to 8 |
| A03 | Lash T. | 4 | 50 | 2, 5 to 8 |
| A04 | Tyrog (dinosaur boss) | 3 | 56 | spawned by others |
| A05 | the Butcher's first form (bald heavy man) | 1 | 15 | 2 |
| A06 | Bludge | 3 | 37 | 5 to 7 |
| A07 | green slime or acid splashes | 2 | 3 | none |
| A10 | Hammer T., Blk Elmer, Wrench T. | 9 | 108 | 1 to 8 |
| A12 | golden velociraptor, big | 1 | 3 | none |
| A13 | Gutter, Skinner, Poacher J (poachers) | 9 | 147 | 2 to 8 |
| A14 | Blade, Razor | 7 | 133 | 1 to 8 |
| A15, A19, A92, A93, A95 | dust puffs, blue slashes, explosions, hooks and debris, blue energy balls | 1 each | 1 to 9 | none |
| A16 | blue open car | 1 | 2 | 2 |
| A17 | gold plates or shields (round) | 1 | 5 | none |
| A18 | Vice T., Walther | 6 | 87 | 1, 3 to 6, 8 |
| A20 | Punk, Thug | 7 | 106 | 1 to 5, 7 |
| A22 | brontosaurus legs (green) | 1 | 5 | 6 |
| A24 | Shivat (T. rex) | 2 | 30 | 2, 5 |
| A25 | Slice | 3 | 30 | 4 |
| A26, A77 | fireballs, dynamite sticks | 1, 2 | 14, 26 | none |
| A28, A47 | small orange raptors, small lizards | 1 each | 8, 3 | none |
| A29 | R Hopper (velociraptor) | 2 | 32 | 5 |
| A31 | Hogg | 1 | 13 | 3 |
| A32 | motorcycle and wheels | 1 | 2 | none |
| A34 | Knife (thrown) | 1 | 11 | none |
| A35 | thrown guns (rifle, machine gun) | 2 | 33 | 5 |
| A36 | Morgan | 2 | 17 | none (spawned by others) |
| A37, A46 | pterodactyl, long tailed bird | 1 each | 7, 2 | none |
| A38 | Mack (triceratops) | 1 | 8 | 2, 5 |
| A39 | robed old man pointing (and a rock monster frame) | 1 | 5 | none |
| A40 | Morgue | 4 | 76 | none (spawned by others) |
| A42 | explosions (8 sizes) | 1 | 2 | none |
| A44 | golden shells or small animals | 1 | 6 | none |
| A45 | ragged rifleman | 1 | 2 | 5 |
| A49, A50 | background birds | 1 each | 3, 5 | 1 |
| A51 | blue crested dinosaur (two colours) | 1 | 2 | none |
| A52, A58 | thrown blue machetes, swords | 1, 2 | 34, 4 | none |
| A53 | red blood splashes | 1 | 2 | none |
| A54 to A58 | opening scene people, revolver, muzzle flash | 1 each | 1 to 4 | none (opening scene classes spawned by class 74) |
| A61 | Butcher | 4 | 49 | none (spawned by A05) |
| A62 | spiked mace on a chain | 2 | 11 | none |
| A64 | Punk, Wrench T. (alternate) | 1 | 8 | 3, 4, 5, 8 |
| A65 | rolling barrels | 1 | 2 | 3, 6 to 8 |
| A66 | rocks (8 frames) | 1 | 1 | 5 |
| A69 | animated credit script text lines | 1 | 3 | none |
| A70 | kicker in blue shorts | 1 | 2 | 3, 4, 7 |
| A71 | Slisaur | 3 | 30 | none |
| A72 | Tyrog (second) | 2 | 17 | none |
| A74 | flame streaks | 1 | 2 | none |
| A75 | stone statue pieces | 1 | 12 | none |
| A79 | Hammer T., Blk Elmer, Wrench T. (alternate) | 3 | 15 | 2, 5 |
| A81 | red car with a woman driving | 1 | 7 | none |
| A82 | Zeke | 1 | 8 | 5, 7, 8 |
| A83 | flames | 1 | 4 | none |
| A84, A86, A87, A89 | Fessenden parts | 3, 1, 1, 1 | 40, 7, 3, 11 | none (spawned by others) |
| A97, A99 | title logo pieces, character portraits | 5, 2 | 12, 9 | none |

Not all of these are real "unused": classes marked none are spawned by other code (bosses, controllers) or are effects.

### 4.3 Pool B, C, E

- **Pool B** (props and controllers): only B25, B65, B69, B70, B77, B86, B88 have animation tables (see N13).
- **Pool C** (effects): 32 classes have art, for example C01 (hit sparks, 26 animations), C21 (60 animations, 141 compositions), C24
  (41 animations), C11 (debris), C04 (water splash), C05, C10, C14, C15, C20, C31, C36, C37, C40. Several use raw sprite lists
  (no compositions): C02, C07, C12, C13, C16, C26, C28.
- **Pool E** (items): E00 weapons (16 tables, 336 animations), E01 explosions, E03 bullets, E04 food (18 kinds), E05 containers
  (7 tables), E06 helper, E07 shell casing, E08 projectile, E09 treasure (16 kinds), E10 pellets, E11 attack box.

### 4.4 Compositions that no table draws (selected, with byte index and address)

All of these may be used by code (see N13). Gallery: `comps/unreferenced/`.

| Composition number | What it shows |
| --- | --- |
| 268 to 278 | a thug in green and yellow clothes, four frames |
| 326, 327, 3299 | chain-link fence pieces (one broken, one with a sign) |
| 638 to 770 | a heavy man in an orange vest (thrown, kneeling and punching frames) |
| 1278 to 1283 | a bound or bandaged blond man (six similar frames) |
| 1504, 1506 | a woman in orange and white walking |
| 1607, 1608 | speech banners: a "look out" style bubble and a red "break" banner with an arrow |
| 1880, 1897 to 1900 | wooden round lid, red pillars |
| 2237 to 2239 | "1P", "2P", "3P" tags |
| 2347, 2366, 2416 | big green raptor, muscular gold man, triceratops lying |
| 2417 to 2421 | joystick, button, three arrows (down, left, diagonal) |
| 2895 to 2903 | "STAGE CLEARED!" sign and the digits 1 to 8 |
| 2913 to 2983 | rotating spiked ball (71 frames) |
| 3187 to 3258 | falling, flying and knocked down human frames (heroes, thugs in green and yellow, a woman, bandaged fighters, a dinosaur) |
| 3298 to 3300, 3350, 3356 | round grate or lid, manhole, two chains |
| 3412 to 3448, 3465, 3479 to 3481 | a knife fighter in a green vest (14 frames), hiding behind a bush, a blond man with a knife |

## 5. Stages

- 8 episodes. Areas per episode: 3, 3, 1, 3, 2, 3, 4, 3 = 22; sub-areas that reuse the previous map: 1.1, 4.1, 5.2 (so 19 maps).
- Map sizes in screens (scroll2): see `re/specs/stage/SPEC.md` section 4.6; the longest are Episode 3 (60 screens wide, the loop
  road) and Episode 7 area 1 (56 screens tall, the elevator shaft).
- Each area has: the full scroll2 map, the far scroll3 map (16 of 19 areas), a copy with camera events (coloured lines, label
  with event number, mode and area), and 384x224 views along the map. Event tables: 116 records.
- The elevator cage (scroll1 pages 1 and 2) is drawn as `ep7_a1_scroll1_elevator_cage.png`.

## 6. Front end pictures (decoded from data, no emulator)

See N5 for the table. Short list of page groups by scene (scroll3 unless said):

| Scene | Pages | Code |
| --- | --- | --- |
| title logo | 0x73, 0x74 ("Cadillacs"), 0x67, 0x6F ("Dinosaurs"); scroll2 0x14, 0x15 | 0x97E2E, 0x9814A |
| region warning picture (skipped in the World set) | 0x3E, 0x3F | 0x98330 |
| story slides | 0x64, 0x71, 0x72 (jungle), 0x65, 0x66, 0x68, 0x69 (sea and city), 0x6A, 0x6B (dinosaur head), 0x6D, 0x6E (villain face) | 0x988C0 area |
| jungle pictures (probably the opening movie) | 0x72 and scroll2 0x03, 0x04 (forest); close ups of heroes and a dinosaur in scroll2 0x10 to 0x13 | 0x99374, 0x9B720, 0x9C2D8 |
| select screen | scroll2 0x01, 0x02 (four heroes), scroll1 pages 3 to 6 (1P, 2P, 3P frames), scroll3 0x06, 0x09 | 0xADA6 to 0xAE0A |
| continue and game over | scroll2 0x18 to 0x1A (villain with a gun), 0x13, scroll3 0x75, 0x76, 0x77 | 0xEA74 to 0xEEB4 |
| ending | scroll3 0x6C, 0x78 to 0x7C (desert), scroll2 0x16 (ground) | 0x8A63C, 0x15222 |

Pages 0x7D to 0x7F look like a tile atlas, not pictures. The profile cards (portraits on a checker board), the "BRAVES" ranking,
the demo and the map panel are made of sprites and tile writes; use the screenshots in `frontend/` for the exact look.

## 7. Text

- 362 ASCII strings (print routines 0xC0A, 0xBB6, 0xC44): `text/ascii_strings.txt` with address and region label (credit line,
  select panel, tally words, HUD names, name table, weapons 20, foods 36 and treasures 17 (names and point words), AI states 64,
  staff roll 53 lines, title, coin and region text, test menus 82).
- 109 tall font strings (print routine 0x84F72, 8x16 glyphs, two tiles per letter, letters A to O at tiles `0x160` to `0x16E`,
  P to Z at `0x190` to `0x19A`): the 7 episode titles (twice), story panels, captions and the ending. A few strings contain
  hiragana tile codes (leftovers of the Japanese text). `text/tall_font_strings.txt` and one picture per string.
- Some strings (profile cards, a few captions) are blank placeholders in this ROM; the text is written at run time.

## 8. Sound

- Z80 program: decrypted with the Kabuki keys of the sound spec: `sound/z80_opcode_image.bin`, `z80_data_image.bin`, disassembly
  `z80_listing.txt`.
- Command table at linear `0x8000`: 262 entries, 243 live. Header at the entry pointer: type byte (0 music, 1 to 0x7F effect with
  priority), 16 track offsets (one per QSound voice).
- `sound/sound_codes.txt`: one line per code with type, priority, voices, length and who sends it (music uses from the stage tables,
  hit sound ids of routine 0x962A, callers found by scanning for the enqueue routine 0xA63C).
- `sound/sound_sequences.txt`: every track of every code decoded to notes, tempo, instruments and loops (length in steps of 1/62.5 s).
  Music lengths are intro plus loop; 14 tracks run over 4 minutes before they repeat (limit of the decoder).
- Samples: 124, 1,963,673 bytes, `sound/samples/sample_NNN.wav` (8 bit signed, 24038 Hz), table in `sample_table.txt` with bank,
  loop point, root key, the instruments and the codes that use each sample. At pitch register `0x1000` (root key) a sample plays at
  the QSound rate, so 24038 Hz is its natural speed.
- Music and composed effects as audio: from our own driver port (M4), not from MAME.

## 9. Not extracted, and why

| Item | Why | What to do |
| --- | --- | --- |
| Audio of music and effects | The user stopped MAME use; the QSound DSP ROM is not in the set (MAME output would have been inexact) | M4 sound driver port, then render with it |
| Pictures made of sprites and tile writes (profile cards, ranking, demo, map panel, tally) | no single picture block in the data; they are composed by code | existing screenshots in `frontend/`; "capture later" with T0's probe when the port draws them |
| Exact ownership of the 319 unreferenced compositions | index computed by code | per class trace in D3e |
| Which pool B class draws which scenery | not in animation tables | D3e |
| Directly sent sound codes for 133 codes | wrapper tables and computed codes | D4e |
| Scroll1 text and HUD pictures as layouts | built by the text printers at run time | printers are ported in M1 |
| The 373 OBJ palettes nothing references | maybe unused colour variants | check when a class asks for a palette by number |
