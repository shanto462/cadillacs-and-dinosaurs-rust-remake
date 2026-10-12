# D4a: items and drops (food, treasure, containers, drop rows of all 8 episodes, lifetime rules)

Written 2026-10-12 by the D4a research agent. Part of the design phase (`plan/game/NEW-PLAN.md` section 1, D4). Documents only: no code was changed.
Nothing here is ROM data (no byte dumps, no listings): only descriptions, numbers and addresses. The weapons themselves (15 types, wear, throws) are in
`design/D2c-weapons-multiplayer.md` and are not repeated here; this file links to it.

How to read this file

- **Tags.** `R:` = ROM address or repo file (68000 addresses; `G.$xxxx` = RAM `0xFF8000 + xxxx`; `+$xx` = a field of an object). `M:` = measured in MAME with the probe tool (the
  segment name is given; logs are in `re/out/probe/d4a-r1` to `d4a-r4`, git-ignored). `W:` = web (links in section 15). `U:` = the user. **MAME and the ROM win over the web.**
- **Runs.** Four headless MAME runs (all through `re/harness/probe/probe.sh`, which sets `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy`): `d4a-r1` (108 segments: every food, every
  treasure, containers, stones, drop rules, despawn, flags), `d4a-r2` (16 follow-ups), `d4a-r3` (4: what else breaks a container), `d4a-r4` (9: pop-in, tire stacks). The generators
  and the decoders are in `re/out/d4a/` (`gen_r1.py` to `gen_r4.py`, `batch1.txt` to `batch4.txt`, `decode_drops.py`, `mk_tables.py`, `mk_b22.py`, `an_cont.py`).
- **Time.** Frames `t` are game frames after the probe `jump` (one clock second is 51 game frames). A key pressed at `t` shows in the log at `t + 1`. The hero was Jack with
  `god on` at camera X 928, depth 296, unless a segment name says otherwise.
- **Pool E** = the 24 item objects (`item.rs` header): class 0 weapon, 1 explosion, 2 ammo box, 3 bullet, 4 food, 5 container, 6 torch glow, 7 shell, 8 projectile, 9 treasure,
  10 pellet, 11 attack box. `+$26` is the variant, `+$A0` the kind word (0x00tt food, 0x0100 ammo, 0x02kk weapon or container, 0x03tt treasure). R: SPEC 13.1.
- **Row** = a drop row: 32 bytes, one row per `+$96` value; the code byte is picked with `rand() & 0x1F` ("of 32" in the tables).

## 0. Summary of what is new

1. **There are 17 foods, not 18.** The food heal table 0x28B10 has 17 entries (0x28B10 to 0x28B53). The 18th "entry" that the old spec lists (BARBECUE, heal 36, bonus 4000) is the first
   four bytes of the animation table that follows at 0x28B54. A food with `+$26 = 18` still works: MAME heals 36, gives 4000 at full HP, draws no sprite and shows the name BARBECUE with the
   text "10000 pts" while the score is 4000. No drop row can make it. R: 0x28B10, 0x28B54; M: f18_food18. (Section 2.2.)
2. **Every food and treasure number is confirmed in MAME** (17 foods hurt and at full HP, 17 treasure ids). The ROM has six heal tiers 16, 32, 48, 64, 80 and 100 HP and four full-HP bonus
   tiers 1,000 / 3,000 / 5,000 / 10,000. The GameFAQs tiers "15, 33, 50, 66, 80 and 100 percent" are right; the fan list M17 is wrong for steak (80, not 90), lobster and sushi (64, not
   80), salad (bonus 5,000, not 3,000), donut and gum (16, not 20 and 10). Treasures: M16 is right except that it misses THE CUP and DINOBONE and invents a "book". (Sections 2.3, 3.2.)
3. **A food is eaten one frame after the press; a treasure scores 11 frames after it.** Food heals at once (press t = 10, HP at t = 11); treasure adds its points when the 10 frame lift ends
   (t = 21). The HUD message follows 10 to 20 frames later, shows a "FooD" or "Pts" icon, the name, "GOOD !!" or "N pts", and stays about 60 frames. M: f01, t01, hud_clean. (Section 2.4.)
4. **Containers: 7 variants have code, 13 numbers exist.** `+$26` is 0 to 0xC but only the even values have art and code: 0 pail, 2 oil drum, 4 wooden barrel, 6 stone, 8 boulder, 0xA golden
   armor statue, 0xC tire stack. The scripts use only 5 of them (pail 17, drum 35, barrel 37, statue 4, tires 14 records); stone and boulder are never placed, they exist as the source
   of debris. R: 0x28C8A, stage_scripts; M: cont*. (Section 5.1.)
5. **One hit of any hero breaks every normal container** (HP 4; Jack's jab does 10). +hit score (100 for a jab) and +500 at the break. An explosive container has HP 50 and takes
   10, 10, 15, 15 from Jack's chain. M: cj_*, tnt_5hits. (Section 5.3.)
6. **What breaks a container** (M): any hero attack frame, a bullet, a thrown knife, a thrown barrel, an explosion. (R only: a falling or flying body does 8, an enemy attack frame destroys it
   with no credit; six MAME tries of the enemy case did not trigger the swing.) (Section 5.4.)
7. **Tire stacks and statues cannot be lifted** (M: 9 offsets for tires, 3 for the statue); a barrel is lifted only when the hero stands 24 px beside it with the stick pushed towards it,
   not on top of it. (Section 5.5.)
8. **Drop tables:** containers, rolling barrels (class 65) and falling rocks (class 66) read the **container table** 0xB64F2 (65 rows); every other enemy, also the biker (class 70), reads the
   **enemy table** 0xB6D12. M: barrel65_drop, biker70_drop, edrop_kill. The scripts use 42 container rows and 32 enemy rows (the boss helper lists add 8 more), 262 non-empty drop records in all. (Section 6.)
9. **Row code 0x0F "SPECIAL"** is an ammo box when any gun item (rifle, gun, Uzi, M-16, shotgun, bazooka; held or lying) exists in pool E, otherwise one of six cheap treasures. M: sp_*. (6.3.)
10. **Stones from debris are real** (closes D2c open item 1): broken stone and boulder containers and falling rocks make 9 debris pieces; each piece can turn into a stone item with chances
    2/16, 8/16, 3/16 while fewer than 3 stones are counted in `G.$4F1`. MAME: 4 and 7 stones from 8 and 16 breaks (D2c broke its 20 containers at the same RNG state, which explains its
    zero). Boulder pieces give small and medium stones; stones never expire. M: stones_*, stones2_boulder, stone_items. (Section 8.4.)
11. **Player count and level change what is placed.** 42 item or drop records exist only for 2 or 3 players and 23 only below or above a level. The web claim L12 is only partly right: extra guns
    exist in Episodes 1 and 4 (rod and club; shotgun and M-16) but extra food and treasure go on to Episodes 5, 6 and 7. The level gates swap the content of the same container: Episode 7 area 0
    barrel holds an Uzi 31/32 at level 16 or less and 8/32 above. M: e1a0_c*p, e4a2_*, e7a0_lv*, e4a2b_m16_*. (Section 7.4.)
12. **Episode 3 (L02):** the area scripts hold only radios (12 of them: 4 containers and 8 enemies); no food, no weapon. The Hogg helper list (pool B 22) brings the guns in the order Uzi,
    shotgun, rifle, shotgun, shotgun, M-16, Uzi, **and two small foods** (donut 28/32, a 50 percent coffee row). So "no recovery items" is true for the road and not quite for the boss.
    R: stage_scripts, b22_lists. (Section 7.6.)
13. **Episode 2's fourth treasure (the pearl) is never seen**: its record has X 0x1A8 but fires at camera X 0x428, so it is created 640 px behind the camera and freed after 2 frames.
    The other three (diamond, 1000$, dime) lie at depth 16, height 64, behind the big trunks. A second record of this kind (derived, not seen in MAME): the 3 player drum of Episode 5 area 0 (X 0x2D0, trigger 0x80) is 0x24B px right of the camera when it fires, past the far window, and is freed in its first frame. M: e2_pearl, e2_start, e2_diamond. (Section 8.1.)
14. **Despawn windows measured:** food, treasure and containers live while X is in `[camX - 0xC0, camX + 0x240)`; weapons, ammo boxes and rolling barrels in `[camX - 0x40, camX + 0x200)`.
    Nothing on the floor times out (3,800 frames checked for food, treasure, ammo box, placed and dropped guns, stones). M: despawn_x, life_long. (Section 9.)
15. **Dynamite put down by swapping a weapon is lit when it lands** (4 stages of 15 frames, 60 frames); picking it up pauses the fuse, putting it down resumes it. Confirms M18. M: dyn_swap_*. (9.4.)
16. **A full pool loses the drop** (22 foods plus a container: the drop appears; 23: it does not). M: pool_22, pool_23. A drop bounces back in from the screen edge (5 times at most). (9.2, 9.3.)
17. **The GameFAQs barrel notes (N08 to N12) agree with the ROM rows in every case checked** (section 10). The fan sentence "rolling barrels carry items only if you go slowly" is the near
    despawn window: a barrel that rolls 64 px left of the screen is freed with its item.

## 1. Pool E recap and the rules every item shares

- **Update.** Pool E runs after the players and before pool A (R: 0x233C6, SPEC 13.1). Port: `game/item.rs`, `item/{shared,small,container,hits,weapon,blast}.rs`; all of it was checked against
  MAME in phase 1 task 2 (`plan/phase-01-stage1-complete.md`), except what section 12 lists.
- **Modes** (`+$04`): 0 init, 2 alive, 4 break (containers) or free, 6 free next frame. **Subs** (`+$05`): 0 on the ground, 4 being lifted, 8 held, 0xC dropped, 0x10 and up thrown or flying. `+$06`
  is 2 for an object that was spawned airborne by a drop roll (it hops out) and 0 for an object placed by the script (it lies where it is). R: `item.rs` header.
- **Pick up.** No auto pick up; the hero presses attack on top of the item, lowest kind word first (food, then ammo box, then weapons and containers, then treasure). Details, the 40 frame weapon
  lock and the two player rule: D2c section 2.1. Only an item with `mode 2, sub 0` can be taken, so a dropped item cannot be taken while it hops (about 48 frames, section 6.5).
- **Lists.** A live item registers in the pick up list 0x69EE (food, treasure, ammo box, weapons, liftable containers), the attacker list 0x671A (containers, thrown and swung items) and, for
  containers, the hurtable list 0x6AFC and the obstacle list (solid ones). R: 0x4AC2, 0x4B80, 0x4BDA, 0x4A96.
- **Floor and box.** Items snap to the floor under them (0x3140C) and use the box table 0xC0100000; their `Z` is the script depth, `Y` starts at 0. A treasure at depth 16 in Episode 2 area 0 sits at
  height 64 because the floor there is 64 high (M: e2_start, e2_diamond: hero y 64 at z 16).
- **Only the pool counts.** There is no item limit except the 24 slots shared with weapons in hand, bullets, explosions, shells and containers (section 9.2).

## 2. Food (pool E class 4, handler 0x2898C)

### 2.1 The 17 foods

Names are the ROM name strings (R: 0x240F0..0x241DE; the message prints them). Art: the animation table 0x28B54 (`re/out/assets/anims/item_e04/`, contact sheet `re/out/d4a/food_sheet.png`);
every picture is a floating plate or snack. Heal is in HP points (a hero has at most 100). The bonus is what the food gives instead when the hero has exactly 100 HP.

| `+$26` | ROM name | What the picture shows | Animation (id, frames) | Heal HP | Bonus at 100 HP | Message text at 100 HP | Fan list M17 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | BARBECUE | roast meat on the bone, steam sparks | 0, 3 frames (30 ticks, loops) | 100 | 10,000 | " 10000 pts" | 100 / 10,000 (same) |
| 2 | STEAK | steak on a hot iron plate, steam | 1, 4 frames (30 ticks) | 80 | 10,000 | " 10000 pts" | 90 |
| 3 | LOBSTER | red lobster on a white plate | 2, 1 frame | 64 | 10,000 | " 10000 pts" | 80 |
| 4 | SUSHI | sushi platter | 3, 1 frame | 64 | 10,000 | " 10000 pts" | 80 |
| 5 | BURGER | hamburger | 4, 1 frame | 48 | 5,000 | " 5000 pts" | 50 / 5,000 (same) |
| 6 | HOTDOG | hot dog | 5, 1 frame | 48 | 5,000 | " 5000 pts" | 50 / 5,000 (same) |
| 7 | PIZZA | pizza slice | 6, 1 frame | 48 | 5,000 | " 5000 pts" | 50 / 5,000 (same) |
| 8 | SALAD | green salad with an egg | 7, 1 frame | 32 | **5,000** | " 5000 pts" | 30 / 3,000 (bonus differs) |
| 9 | CAKE | cake slice with a strawberry | 8, 1 frame | 32 | 3,000 | " 3000 pts" | 30 / 3,000 (same) |
| 10 | PAFFE | parfait glass (the ROM spells it PAFFE) | 9, 1 frame | 32 | 3,000 | " 3000 pts" | 30 / 3,000 (same) |
| 11 | PUDDING | pudding in a glass dish | 10, 1 frame | 32 | 3,000 | " 3000 pts" | 30 / 3,000 (same) |
| 12 | POTATO | pack of French fries | 11, 1 frame | 32 | 3,000 | " 3000 pts" | 30 / 3,000 (same) |
| 13 | GUM | green pack of gum | 12, 1 frame | 16 | 1,000 | " 1000 pts" | 10 |
| 14 | CHOCOLATE | chocolate bar | 13, 1 frame | 16 | 1,000 | " 1000 pts" | 15 |
| 15 | DONUT | chocolate donut | 14, 1 frame | 16 | 1,000 | " 1000 pts" | 20 / 1,000 |
| 16 | COFFEE | cup of coffee on a saucer | 15, 1 frame | 16 | 1,000 | " 1000 pts" | 15 |
| 17 | CROISSANT | croissant | 16, 1 frame | 16 | 1,000 | " 1000 pts" | 15 |

Sources: heal and bonus R: table 0x28B10 (2 words per food: heal, offset into the score table 0x10292 where 0x38 = 1,000, 0x40 = 3,000, 0x48 = 5,000, 0x4C = 10,000), strings 0x240CA;
M: f01 to f17 (HP 40 to 40 + heal capped at 100; score at 100 HP exactly as listed). All 17 rows were measured, none differs from the ROM. The picture names come from my reading of the
dump (`item_e04`); the names are the ROM strings. The picture sheet is `re/out/d4a/food_sheet.png`.

### 2.2 The 18th entry is not a food

- `+$26 = 18` reads 4 bytes after the table: heal 0x24 = 36, offset 0x44 = 4,000. The animation id 17 does not exist, so the object has no picture. The message prints the name table entry 18, which
  wraps to BARBECUE, and the points text entry 17, which wraps to " 10000 pts". M: f18_food18 (HP 40 to 76; at full HP score 4,000 while the HUD says 10000). R: 0x28B54.
- No drop row, no script record and no code creates it. The port reads the same bytes (`item/small.rs` reads the extracted 0x48 byte table) so it already behaves like MAME. Keep the quirk,
  do not "fix" it (it can only appear from a poke).
- Food ids 19 and 20 would read the next words of the animation table (heal 110, 134) and are not reachable either (the drop row codes 0x26 and 0x27 would make them; no row contains them).

### 2.3 Fan table M17 and the FAQ tiers against the ROM

- ROM tiers (HP): 16 (5 foods), 32 (5 foods), 48 (3 foods), 64 (2 foods), 80 (1 food), 100 (1 food). Bonus tiers: 1,000 (5 foods), 3,000 (4), 5,000 (4), 10,000 (4).
- The GameFAQs "approximately 15, 33, 50, 66, 80 and 100 percent" (supplement 2) equals 16, 32, 48, 64, 80, 100. Confirmed. (W: GameFAQs ninjasrok FAQ in supplement 2; R.)
- M17 (Namu, machine translated): barbecue 100 / 10,000 exact; hamburger, pizza and hot dog 50 / 5,000 about right (48); cake, fries, parfait and pudding 30 / 3,000 about right (32);
  chocolate, coffee and croissant 15 about right (16). **Wrong:** steak 90 (ROM 80), sushi and lobster 80 (ROM 64), salad 30 / 3,000 (ROM 32 / 5,000), donut 20 (ROM 16), gum 10 (ROM 16).
  The ROM wins. Steak, lobster and sushi pay 10,000 at full HP (the fan list names only the barbecue).

### 2.4 Eating, the message and the sound

- **Heal.** The hero presses attack on the food (state 0x14, lift animation 0:0xA, 10 frames). The food is eaten on the **first frame of the lift**: `hp = min(100, hp + heal)`; at exactly 100 HP the
  bonus is added to the score of the eater instead (the compare is `hp == 100`). The life bar is redrawn. M: f01 (press t = 10, HP 100 at t = 11, lift state t = 11 to 20, score +10,000 at t = 79 for
  the full HP eat at t = 78). R: 0x28AA4, 0x2495A.
- **Sound.** When the lift ends (food in the hand frame) the food plays the eating voice by hero and voice bank (+$118): Jack, Mustapha, Mess 0x7C, 0xFD, 0xFE; Hannah 0x7D, 0xFF, 0x100, with
  a pan from the food's screen X (table 0xA66A), then the object is freed. The press itself plays the pick up voice 0x9C20 (D2c). R: 0x28A86, 0x9F54, 0x9FAA, 0x9FB0. The sound codes
  belong to D4e.
- **HUD message.** Each hero block shows a message ring. A food pushes a record (kind 3) and the block shows, 10 to 20 frames after the press: a 2 by 3 tile icon with the word "FooD", the name on
  the first text row, and " GOOD !!" on the second (when healed) or " N pts" (at 100 HP). It stays 60 frames; a second message waits in the ring (M: f01 shows BARBECUE GOOD !! from t about 20 to
  88, then " 10000 pts" from t about 90). The message replaces an enemy bar (the GNEISS bar of the jump disappears while it shows). R: 0x1693C, 0x23F24, tile table 0x24016, name table
  0x240A2, points strings 0x240CA, text " GOOD !!" at 0x242CE; M: f01_BARBECUE, f09_CAKE, hud_clean (pictures at 700 to 925).
- **Eaten by Bludge.** D3a (HU09): Bludge eats floor items, lowest kind first (so food first).

## 3. Treasure (pool E class 9, handler 0x2A028)

### 3.1 The 16 treasures

Art: animation table 0x2A174 (`item_e09`, sheet `re/out/d4a/treasure_sheet.png`). Score offsets: bytes at 0x2A164 into the score table 0x10292 (0x38 = 1,000, 0x40 = 3,000, 0x48 = 5,000,
0x4C = 10,000). All 17 ids were picked in MAME (t01 to t17).

| `+$26` | ROM name | Picture | Animation (id, frames) | Points | M16 fan list |
| --- | --- | --- | --- | --- | --- |
| 1 | GOLD | stack of gold bars, sparkles | 0, 5 frames (120 ticks) | 10,000 | gold bar 10,000 |
| 2 | DIAMOND | white diamond, sparkles | 1, 5 frames (120 ticks) | 10,000 | diamond 10,000 |
| 3 | THE CUP | big golden trophy cup | 2, 1 frame | 10,000 | **missing** |
| 4 | EGG | green dinosaur egg with a baby curled in it | 3, 2 frames | 10,000 | dinosaur egg 10,000 |
| 5 | GOLDDUST | brown sack with gold dust spilling out | 4, 2 frames | 5,000 | gold dust 5,000 |
| 6 | DINOBONE | fossil dinosaur skull | 5, 1 frame | 5,000 | **missing** |
| 7 | AMMONITE | brown ammonite fossil slab | 6, 1 frame | 5,000 | ammonite 5,000 |
| 8 | PEARL | open pink shell with a pearl | 7, 2 frames | 5,000 | pearl shell 5,000 |
| 9 | NECKLACE | gold chain with red and green stones | 8, 1 frame | 3,000 | necklace 3,000 |
| 10 | RING | gold ring with a green stone | 9, 1 frame | 3,000 | ring 3,000 |
| 11 | WATCH | wrist watch | 10, 1 frame | 3,000 | watch 3,000 |
| 12 | RADIO | grey radio box with an antenna | 11, 1 frame | 3,000 and the Episode 3 event | **missing** (the fan lists name no radio) |
| 13 | DIME | a coin that spins on its edge | 12, 17 frames (144 ticks, loops frame 16) | 1,000 | coin 1,000 |
| 14 | 1000$ | bundle of banknotes | 13, 2 frames | 1,000 | bill 1,000 |
| 15 | SUNGLASS | blue sunglasses | 14, 1 frame | 1,000 | sunglasses 1,000 |
| 16 | GROVE | leather glove (the ROM spells it GROVE) | 15, 1 frame | 1,000 | glove 1,000 |
| 17 | (none) | a small flat grey plate, 4 frames; no name, no score entry | 16, 4 frames | 10 (reads byte 0 of the animation table, offset 0 of the score table) | "book 1,000" does not exist in the ROM |

R: 0x2A164, strings 0x24436 (the name and the points on one string such as "GOLD / 10000 pts"). M: t01 to t17 (score after the pick up: 10,000 for 1 to 4, 5,000 for 5 to 8, 3,000 for 9 to 12,
1,000 for 13 to 16, 10 for 17). No row can create id 17; MAME shows a garbled message for it (open item 4).

### 3.2 What the ROM confirms of M16

- Confirmed: all four tiers and the members of each, "partly random" (the rows 0x02, 0x22, 0x29 mix them).
- Not in M16: THE CUP (10,000), DINOBONE (5,000), RADIO (3,000). Not in the ROM: the "book".
- Where they sit (hidden behind scenery or appearing when the camera reaches a point): section 8.1.

### 3.3 Pick up, score and message

- The score is added when the **lift ends**, not at the press: press t = 10, lift state t = 11 to 20, score at t = 21 (M: t01 to t17; R: 0x2A10E). The treasure has no sound of its own; the
  hero's pick up voice plays at the press (R: 0x2A10E has no sound call).
- The message (kind 4, `0x16A5A`): icon with the word "Pts", the treasure name on the first row (GOLD, RADIO, DIAMOND ...), " N pts" on the second; about 10 frames after the score, 60 frames
  long. When an enemy bar is on the HUD the old enemy name stays in the first row and only " N pts" shows (M: t01_GOLD with the jump's GNEISS bar). Clean pictures: M: hud_clean.
- RADIO: when the stage index is 2 (Episode 3), `G.$50E == 0` and `+$26 == 12`, the collect routine also calls 0x8BD24 (the car event, D3c section 5). R: 0x2A12C to 0x2A150.

## 4. Ammo box and the other things that make items

- **Ammo box** (class 2, 0x283DE): D2c section 10 (refill, +1,000, the "next gun is filled" flag). Floor rules here: `+$A4 = 0` (never expires), near despawn window, hops out when dropped.
  It is made only by the drop code 0x0F and by no script record. R: 0x24C80.
- **Weapons as items:** D2c section 11 lists the sources. Added here: placed weapons per episode are in the table of section 8.1 (torches in Episode 5, a rifle, a shotgun and two M-16 in
  Episode 4 area 2, stones in Episodes 4 and 7).
- **Other creators of pool E objects** (R: the 29 call sites of the allocator 0x4818): the join bazooka (0x65A4); the drop roll (0x24B86 weapon, 0x24BCA food, 0x24C04 treasure, 0x24C80 ammo box);
  the carrier's dynamite stick (0x5F04E, D3a HU26); the Blade knife part (0x51394, D2c); the debris stones (0x93770, section 8.4); pool A class 26 (0x4D87A: a dynamite item with
  `+$26 = 0x18`, or a grenade `0x1A` when its own variant is 0); pool A class 67 (0x5BFA4: an Uzi item when its variant is 0; owner unknown, D3d); pool A class 68 (0x5C1BE: a dynamite item when
  its variant is 1); the torch glow overlay, a class 6 object (0x24F6E for a held torch, 0x6A970); the rest are explosions, bullets, pellets and projectiles (D2c, D3f). Only the drop roll and
  the script records make food, treasure and ammo boxes.

## 5. Containers (pool E class 5, handler 0x28C76)

### 5.1 The variants (`+$26`) and what each looks like

`+$26` is a word 0 to 0xC (13 numbers) but the init handler has a jump table of 7 entries indexed by the even values; an odd or larger number would jump into data. So **7 variants exist**.
The scripts place only 5 of them: 107 container records in list 2 of the 8 episodes (pail 17, drum 35, barrel 37, statue 4, tire stack 14). Stone and boulder are never placed; they are the
objects that make debris stones (section 8.4) and exist for the `spawn E 5` of the harness. R: 0x28C8A, 0x28C96 table; stage_scripts.txt; M: cont tests of D3e (all 7 spawn and break).
Art: animation tables 0x2995E, 0x2998C, 0x299FE, 0x29A66, 0x29A86, 0x29AA6, 0x29AC8 (`re/out/assets/anims/item_e05/`, sheet `re/out/d4a/container_sheet.png`).

| `+$26` | Name | Picture (size in px) | Animations | Solid (blocks walking) | Liftable | Used by scripts | Break kind |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | pail | small orange bucket (80 x 136) | 0 idle, 1 and 2 lying, 3 crushed (112 x 112) | yes | yes | 17 (Episodes 1, 3, 4, 5) | the body flies off sideways and blinks away |
| 2 | oil drum | orange steel drum (112 x 166) | 0 idle, 1 lying, 2 rolling (8 frames) | yes | yes | 35 | 2 flying pieces |
| 4 | barrel | wooden barrel (144 x 168) | 0 idle, 1 lying, 2 rolling (7 frames) | yes | yes | 37 (Episodes 2, 3, 5, 6, 7, 8) | a puff of wood pieces (10 objects) |
| 6 | stone | grey rock (80 x 104) | 0 idle, 1 and 2 thrown | **no** | yes, no stick direction needed | never | 9 debris pieces (stone list) |
| 8 | boulder | tan boulder (112 x 100) | 0 idle, 1 and 2 thrown | **no** | yes, no stick direction needed | never | 9 debris pieces (boulder list) |
| 0xA | armor statue | golden knight with a spear (112 x 264) | 0, 1, 2 the same picture, 3 wreck (legs and spear) | yes | **no** | 4 (Episode 1 hall) | puff of 8 pieces |
| 0xC | tire stack | four black tires (176 x 168) | 0 to 3 the same picture | yes | **no in practice** (section 5.5) | 14 (Episodes 1, 2, 3, 4) | puff of 5 pieces |

R: lists in 0x28F0C (variants 0, 2, 4, 0xC register as hurtable, attacker, obstacle and pick up object; 6 and 8 as hurtable, attacker and pick up, not obstacle; 0xA as hurtable, attacker and
obstacle, not pick up); M: lift_* (section 5.5), thug_swing_* (a thug could not walk through a barrel).
Where they stand: the tables of section 7 (variant name, X, trigger, row). The Episode 1 hall statues are containers, not background (R: stage_scripts area 1; M: e1a1_statues shows two golden
statues with `+$26 = 0xA` at X 129 and 331, rows 0x0C and 0x27; D3e agrees). That answers W20.

### 5.2 Object fields at init (R: 0x28C8A)

- Kind word `+$A0 = 0x20A` (the lowest weapon kind 0x201 to 0x209 is taken before a container, a treasure after it); box table 0xC0100000; draw flag 0x10; `+$A2 = 0x3C` (hand index).
- **HP 4** (`+$6C`, bar `+$6A`). With `+$5C != 0` (explosive): HP 0x32 = 50, bar scale 100, idle shimmer counter `+$80 = 0x32`.
- Thrown attack id `+$58` from the table 0x29860 by variant: 0x3C (pail), 0x3E (drum), 0x40 (barrel), 0x42 (stone), 0x44 (boulder, statue, tires). New serial from `G.$67BE`.
- Home position `+$B2` (X) and `+$B4` (Z) for the hit shake. The drop row `+$96` comes from the script record.

### 5.3 Hits, HP and score

- **A hit.** The hurt pass sets `+$00 = 2`, stores the attacker, lowers HP by the damage and gives both sides the attack's hit stop. While the hit stop runs the container shakes: X = home +1 or
  -1 on alternate frames (`+$59` counts down: 7 frames for a Jack jab). A hit that leaves HP above 0 only shakes (explosive drums). HP 0 means break next frame. R: `item/hits.rs`, `container.rs`
  (0x28E00); M: tnt_2 (hit stop 7 at t = 14).
- **Normal containers die to the first hit of any hero**: Jack, Mustapha, Hannah and Mess all broke a barrel with the first jab (HP 4 to 0 at t = 12, Mess t = 13). The score of that hit is the
  attack record's own score (+100 for a jab, M: cj_*, ch_4_barrel_*). One frame later mode 4 starts; one more frame later the break routine adds **+500** to the credited player (table 0x10292
  offset 0x24), runs the drop roll and starts the debris. M: cj_4_barrel (hit t = 12, mode 4 at t = 13, score 600 and the food object at t = 14).
- **Explosive containers** (HP 50): Jack's jab chain does 10, 10, 15, 15 (50 to 40 to 30 to 15 to 0); each hit gives the hit score (+100, +100, +200, +400 in M: tnt_5hits). HP 0 makes it
  explode at once; the explosion path gives **no +500** (the score of 4 hits was 800). M: tnt_5hits (explosion and food drop at t = 51).
- **Credit.** The player in `+$02` at the last hit gets the +500 (the shooter of a bullet, the thrower of a knife or barrel, the owner of an explosion). A hit by an enemy sets `+$02 = 0xFF`:
  no score. R: 0x1201C, 0x29858.

### 5.4 What can break a container

| Cause | Damage | Credit | Source |
| --- | --- | --- | --- |
| Hero attack frames (fist, kick, throw frame class 4 or 6, special) | HP minus the attack record damage, plus the record score | the hero | R 0x11962 (`pass_11962`, hero attack boxes against list 0x671A); M cj_*, tnt_5hits |
| A bullet of a held gun (pistol, rifle, Uzi, M-16, shotgun pellets, bazooka shell) | the bullet's record | the shooter | R 0x120D2 (frame class 4 attackers); M gun_hits_barrel (pistol bullet at 8 px per frame: hit t = 49, +500 at t = 51, no hit score) |
| A thrown knife, stick or other swung or thrown weapon | the weapon's record | the thrower | R 0x120D2; M knife_hits_barrel (thrown knife hit t = 54, +500, knife freed at the hit) |
| A thrown container (a barrel thrown at a second barrel) | record 0x3E (drum, barrel) etc. | the thrower | R 0x120D2; M throw_hits_barrel (second barrel HP 0 at t = 78 when the first arrived, first barrel flew on and broke at t = 91) |
| An explosion object (dynamite, grenade, container, car bomb) | the blast record (0x54 weak, 0x56 big) | the owner | R 0x120D2; M expl_hits_barrel and expl_hits_pail (an explosion object created beside them: HP 0 four frames later) |
| A knocked down, thrown or burning fighter (body list 0x6AC2) crushes a container of frame class 0 | HP minus 8, facing away from the body | the body's owner | R 0x11F86 (not seen in MAME) |
| An enemy attack frame (list 0x671E, frame class 4, no hit stop) overlapping the container's body box | HP set to 0 at once | nobody (`+$02 = 0xFF`) | R 0x1201C. **Not seen in MAME**: six tries (thug_swing_0 and _1, thug_swing2_0 and _1, thug_swing3_stone and _boulder: the thug never swung inside the reach; a solid barrel stopped it from walking up) |
| The Cadillac of Episode 3 | HP 0 | the radio taker | D3c BS20 |

The hero's own container hits use the player attack table 0xA26FC (D2a, D2c section 7.1). Weapons lying on the floor are in the same list and the same pass tests them (R: `player_hits_item` accepts frame classes 0, 2, 4, 6, 0xC); what a hit does to a lying weapon is D2c.

### 5.5 Lifting, carrying and throwing

- Rules for the barrel, drum, pail, stone and boulder: D2c section 9 (lift state 0x14 with animation 0:0x16, 21 frames; carry; throw table; +500 at the throw; 30 damage and +1,000 per enemy hit;
  no +500 at the break after a hit; the thrown container flies 4 px nearer than the thrower; Hannah cannot jump with one; a thrown explosive drum explodes where it lands).
- **Lift position** (new): the hero must stand beside the container, not on it. With the stick pushed towards a barrel at 24 px (left or right) the hero lifted it (state 0x14, 20 frames, the
  barrel held at t = 28); at 0 px (stick right) the press became a jab and broke it. M: lift_barrel_dx-24, _dx+24 (lift), _dx+0 (jab). R: 0x5AA2 (`find_pickup`: the hero's X and Z inside the item's first
  box, `|dY| <= 8`, stick direction towards heavy objects).
- **Tire stacks and statues were never lifted.** Tire stack: 9 offsets from -56 to +56 px with the stick towards it: at -56 to +8 the press became a jab that broke it, at +24 to +56 the jab
  missed; the lift never started. Statue: -24, 0, +24: jab or miss. The tire stack still registers in the pick up list (R 0x28F0C) so the box test must always fail. This answers D2c open item 15:
  treat var 0xC as "hit it, never lift". M: lift_tires_dx*, lift_statue_dx*.
- A jab breaks a tire stack that stands at the back row of the Episode 4 garage (Z 0x58): hero Z 80 and 88 hit it at once (HP 4 to 0 at t = 14), Z 72 missed. The walkable depth there is
  16 to 111 (D4c section 3.2). So the pizza inside needs no dive kick (the GameFAQs text); a hero only has to stand at Z 0x50 or deeper. M: e4a1_tires_jab_z58, _z50, _z48.

### 5.6 Break effects, sounds and debris (R: 0x296AC and the per variant routines; M: cj_*)

| Variant | What happens in the break frame | Sound (request codes, toggled by `G.$43B`) | Effect objects (M: most alive at once, life) |
| --- | --- | --- | --- |
| pail 0 | +500, drop roll, the body flies off: vx +-0x300 (3 px per frame, away from the hit), vy 0x400 (4 px up), gravity 0x20, bounces with half vy, blinks 40 frames (`+$B1`), animation 3 | 0x6E or 0xF5 (0x9FB6), again on each bounce | none (the pail itself is the effect) |
| drum 2 | +500, vx +-0x380, 2 flying pieces (effect class 0x2C, pictures 2 and 4; 0x60 and 0x10 above, vy 0x1C0 and 0x140, half the horizontal speed), drop roll | 0x6E or 0xF5 | 2 pieces, gone by t = 73 (59 frames) |
| barrel 4 | +500, puff effect class 0x54 variant 0x1A (the wood), drop roll | 0x6F or 0xF6 (0x9E60) | 10 effect objects, 1 to 61 frames each |
| stone 6 | +500, 9 debris pieces from the list 0x295F4 (pictures 0x18, 0x1A, 0x1C three times), drop roll | 0x74 or 0xFA (0x9FF8) | 9 pieces, 39 to 90 frames each; some become stones (8.4) |
| boulder 8 | +500, 9 debris pieces from the list 0x29650 (pictures 0x20 and 0x22), drop roll | 0x74 or 0xFA | 9 pieces, 39 to 90 frames; some become stones |
| statue 0xA | +500, puff class 0x54 variant 0 facing away, drop roll | 0x89 | 8 effect objects, up to 61 frames |
| tires 0xC | +500, puff class 0x54 variant 0x22, drop roll | 0x8C or 0x105 (0xA404) | 5 effect objects, up to 61 frames |
| any explosive | explosion (pool E class 1 kind 0, the big blast 0x56), then the drop roll, no +500 | the explosion's own | D2c section 9 |

The numbers 0x2C, 0x54 and 0x64 are class words (`+$20`); the probe class number is the word divided by 4 (11, 21, 25). M: counts and times from cj_0 to cj_C (log lines of the effect pools: pail 0 objects, drum 2, barrel 10, stone 9, boulder 9, statue 8, tires 5, all created at t = 14). The effect classes
0x2C (debris), 0x54 (puff) and 0x64 (fuse spark) are not ported yet (section 12); D3f owns them. A thrown container breaks on its first contact, with a debris roll at the first landing for the drum
(two landings, the second breaks it into 2 pieces). R: `container.rs` `container_thrown_*`.

### 5.7 The explosive container

- Made by the script flag `+$5C != 0` (3 records: Episode 4 area 0 drum at X 0x350 (row 0x22), area 1 drum at X 0x9C0 (row 0x07), Episode 5 area 0 barrel at X 0x7E8 (row 0x03)); the harness can
  turn any variant explosive with `w5c=`. It looks like the normal one and **shimmers (palette 0x84) every 51 frames** while idle (R `+$80` counts 0x32 down). M: tnt_idle.
- **Fuse.** The first hit that leaves HP above 0, or something standing on it (`+$AC`, set by the obstacle test), starts a fuse of 4 stages: stage 3 two blinks of 6 frames, stage 2 five blinks of 3,
  stage 1 ten blinks of 3, stage 0 twenty blinks of 2 (tables 0x28F24 and 0x28F28; a blink is a palette flash 0x84). A fuse spark (effect class 0x19 at X + 10, Y + 0x39) is made at the start.
  M: tnt_2 (hit t = 12, first fuse frame t = 13, explosion object at t = 124: 112 frames in all); tnt_5hits (the same stages from t = 13 to t = 49, cut off by HP 0 at t = 50).
- **Explosion.** `item_explosion` kind 0 (the big blast, record 0x56: 40 base damage, shake 3, D2c 6.2) at the barrel, then the drop roll at the same spot, then the barrel is freed. No +500, no
  hit score for the exploding hit. M: tnt_2 (steak E#21 at t = 124 at x 1059), tnt_5hits (t = 51).
- A thrown explosive container explodes where it lands (D2c).

## 6. Drop rolls

### 6.1 The two tables and who reads them

- **Container table**: 0xB64F2, 65 rows (0x00 to 0x40) of 32 bytes. Row -1 at 0xB64D2 is the fallback row of the code 0x0F (section 6.3). Read by `0x24B48` (`item_drop_from_container`).
- **Enemy table**: 0xB6D12, rows 0x00 to 0x3A are real (rows 0x3B and above read other data and are never used). Read by `0x24B36` (`item_drop_from_enemy`).
- Both routines call the same body at 0x24B56. The row number is `+$96` of the object that drops (a byte, set from the script record, byte 15 of a 16 byte list 2 record or of a wave record).
  The same number means a different row in the two tables. R: SPEC 13.3, `item.rs` `item_drop_roll`.

| Who rolls | Table | When | Source |
| --- | --- | --- | --- |
| Container (class 5), break routine | container | on the first frame of its break, for every variant; for a thrown one at the landing or contact; for an explosive one when it explodes | R 0x28E8C, 0x2914E, 0x2921C, 0x29254, 0x29388, 0x293D6, 0x2941E, 0x296D2 to 0x29848; M cj_* (item made at press + 4) |
| Rolling barrel (class 65) | **container** | when a hero's hit breaks it (the hit gives +100, the item one frame later) | R 0x5BAB4, 0x5BAD0; M barrel65_drop (row 0x0B gave a barbecue, the container table's row; hit t = 115, food made t = 116) |
| Falling rock (class 66) | **container** | when it lands and breaks (debris of the boulder list, then the roll) | R 0x5BD8A; M stones_rock (the rock breaks at t about 115, 240, 355; no script rock has a row) |
| Biker (class 70) | enemy | when a hero's hit throws the rider (+100 for the hit, +200 for the kill) | R 0x5C6CE; M biker70_drop (row 0x0B gave a lobster, the enemy table's row; hit t = 75, food made t = 76) |
| Every other pool A enemy (thugs, Hammer, Elmer, Wrench, poachers, Blades, Walther, Punks, hunters, Bludge) | enemy | **at the killing blow**, inside the shared knock down, thrown and burn handlers, before the body falls; the kill score comes much later (mode 4) | R 0x2A758, 0x2A8E2, 0x2AB4A, 0x2ABDE, 0x2AC64, 0x2ACD2, 0x2ACE6; M edrop_kill (HP 1 thug: HP 0 at t = 14, lobster made at t = 15, the +1,000 only at t = 108); edrop_survive (a thug that is hit and lives drops nothing) |
| Boss and dinosaur classes | enemy | the same callers exist in the class 24 code (0x499FC to 0x49F8A) but no script gives a dinosaur a row | R |

- **Once only.** The routine clears `+$96` before it rolls, so an object cannot roll twice (an explosive pail calls the routine twice in its break; the second call finds 0). R: 0x24B56;
  `container.rs` `container_break_0`.
- **No code sets a row.** A search of the whole program for a constant written to `+$96` found nothing; rows come only from script records (list 2 and wave records) and the boss helper lists
  (section 7.3). R: static search.
- **The random pick** is `rand() & 0x1F`; the routine 0x119C has weak low bits (SPEC "RNG"), which is why a row such as "2 of 16" does not behave like a fair die over a few rolls (section 8.4). The port
  uses the same routine; keep it.

### 6.2 The code bytes of a row

| Code | Result | Object made |
| --- | --- | --- |
| 0 | nothing | none |
| 0x01 to 0x13 | weapon of type `code - 1` (`+$26 = type * 2`). Codes: 1 rifle, 2 gun, 3 knife, 4 Uzi, 5 M-16, 6 shotgun, 7 bazooka, 8 rod, 9 stick, 10 butcher knife, 11 club, 12 torch, 13 dynamite, 14 grenade, 15 stone (type names: D2c section 1) | pool E class 0 (0x24B86) |
| 0x0F | special, see 6.3 (inside the weapon range, it would be the stone, so **a stone is never dropped by a row**) | ammo box (class 2) or a treasure |
| 0x14 to 0x27 | food `code - 0x13` (1 to 20; only 1 to 17 are real, section 2.2) | pool E class 4 (0x24BCA) |
| 0x28 and up | treasure `code - 0x27` (1 to 16) | pool E class 9 (0x24C04) |

Note: code 0x0F is the special in all rows of the tables (M: sp_*); dynamite is code 13 (0x0D) and grenade code 14 (0x0E); the decoder files call them `w12` and `w13` (type numbers). R: `item.rs` `item_drop_roll`, SPEC 13.3.

### 6.3 The special code 0x0F (ammo or treasure)

- If a pool E object that is a gun (weapon class 0, kind 0x02xx, variant flag table 0x24CC6: rifle, gun, Uzi, M-16, shotgun, bazooka) is alive and its `X + 0x18` is not left of the camera X
  (unsigned compare), an **ammo box** is made (class 2, no variant). A gun in a hero's hand counts, a gun that lies counts. Knife, rod, club, torch, dynamite, grenade and stones do not count.
- Otherwise the roll goes on with the **fallback row** 0xB64D2: necklace 4, ring 4, watch 4, dime 12, 1000$ 6, sunglass 2 (of 32) and one more `rand()` pick.
- M: sp_nogun (dime), sp_knife (dime), sp_club (dime), sp_gun, sp_uzi, sp_bazooka, sp_held (ammo box). The left edge test was inconclusive in MAME: a gun placed at camera X - 0x18 and one at - 0x19
  both counted because the bounce in rule moved the gun to X 911 (section 9.3) before the roll (sp_gun_edge_in, sp_gun_edge_out). R: 0x24C3A to 0x24CB4.
- Rows that contain the special: container rows 0x07 (all 32), 0x17 (16), 0x22 (1), 0x23 (1), 0x27 (8); enemy rows 0x02 (all 32), 0x03 (16), 0x09 (3), 0x11 (8), 0x19 (4), 0x21 (1).
  So a "SPECIAL" row is a bullet drop when you are carrying or about to find a gun, and a cheap treasure when you are not.

### 6.4 The made object

- Position: the source's integer X, Y, Z (fractions 0). The object starts with `+$06 = 2` (spawned airborne) and `mode 0`: its init runs on the first frame it is updated.
- The roll stops the whole drop when the pool has no free slot (the allocator's carry): the drop is lost. M: pool_22 (22 foods plus a container: the steak appeared) and pool_23 (23 foods plus a
  container: no steak). The free count word is `0xFFE5F4` (24 at the start of a jump).

### 6.5 The hop (all dropped food, treasure, ammo boxes and weapons)

- Init of a dropped item (`+$06 != 0`): airborne, `Y + 0x30` (48 px above the source), vy 0x300 (3 px per frame up), ay -0x30 (0.1875 px per frame squared). The apex is about 70 px above the floor and the
  item lands about **48 frames** after it was made. M: hop_mid (made t = 22, y 62 at t = 28, apex 69 at t = 34 to 40, on the floor at t = 70). The hero cannot take it until it has landed (sub 0).
- **Landing test** `0x24CE6` (also for placed weapons): if X <= camera X + 0x10 the item hops again with vx +0x180 (1.5 px per frame); if X >= camera X + 0x170 with vx -0x100 (1 px per frame); the
  vertical speed is reversed (halved on the first bounce only) and `+$81` counts bounces, at most 5. Otherwise it stays. M: hop_right (container at camera X + 0x178: first landing at t = 70, second
  hop to X 1277, one bounce), hop_left (X 934: second hop to 974), hop_right2 (X 1290, below the limit 1296: no bounce), hop_mid (no bounce). R: `shared.rs` `item_land_check`.
- The drop does not move sideways at all in the first hop (a drop from a container at X 1184 landed at X 1184).

### 6.6 Rows in use

The scripts use **42 of the 65 container rows and 32 of the 59 enemy rows**; the boss helper lists (section 7.3) add 8 more enemy rows (0x0A, 0x0F, 0x1F, 0x2A to 0x2E). Biggest users: container row 0x29
(generic treasure, 23 records), enemy row 0x05 (a small food in 5 of 32, 21 records), enemy row 0x22 (treasure, 16 records), container row 0x03 (small food, 11 records), enemy row 0x10 (grenade 24 of 32,
12 records), enemy row 0x04 (rare treasure, 7 records).

### 6.7 Drop rows used by the scripts (42 container rows, 32 enemy rows)

Row notation `C:0xNN` = container table, `E:0xNN` = enemy table. "SPECIAL" is section 6.3.

| Table | Row | Contents (of 32) | Used by (episode: places) |
| --- | --- | --- | --- |
| container | 0x01 | gun 31, M-16 1 | E1 x1 |
| container | 0x02 | dime 15, pearl 3, necklace 3, ring 3, watch 3, 1000$ 3, sunglass 1, glove 1 | E1 x1, E4 x1, E5 x2 |
| container | 0x03 | gum 8, chocolate 8, donut 8, coffee 8 | E2 x1, E4 x1, E5 x1, E7 x4, E8 x3 |
| container | 0x04 | club 32 | E1 x1, E7 x1 |
| container | 0x05 | dynamite 16, grenade 16 | E5 x1 |
| container | 0x06 | Uzi 31, M-16 1 | E2 x1, E4 x1, E5 x1, E7 x1, E8 x1 |
| container | 0x07 | SPECIAL 32 | E4 x1, E5 x2 |
| container | 0x09 | steak 32 | E2 x1 |
| container | 0x0b | barbecue 32 | E2 x1, E5 x1, E8 x1 |
| container | 0x0c | salad 32 | E1 x1, E5 x1, E6 x1 |
| container | 0x0f | burger 16, pizza 16 | E1 x1, E6 x1, E7 x1 |
| container | 0x10 | hot dog 30, barbecue 1, dime 1 | E7 x1 |
| container | 0x11 | shotgun 32 | E4 x1 |
| container | 0x12 | rod 32 | E1 x1, E5 x1, E7 x1 |
| container | 0x13 | fries 32 | E4 x1, E5 x1 |
| container | 0x15 | gold dust 28, dime 4 | E1 x1, E4 x1, E5 x1, E6 x2 |
| container | 0x17 | SPECIAL 16, dime 16 | E7 x1 |
| container | 0x18 | lobster 32 | E5 x1 |
| container | 0x19 | dynamite 32 | E2 x1 |
| container | 0x1b | grenade 32 | E2 x1, E6 x2 |
| container | 0x1c | grenade 32 | E4 x1 |
| container | 0x1d | pudding 32 | E7 x1 |
| container | 0x1f | ammonite 30, dime 2 | E5 x1, E7 x1 |
| container | 0x20 | cake 32 | E5 x1 |
| container | 0x21 | bazooka 32 | E5 x1, E7 x1 |
| container | 0x22 | dime 31, SPECIAL 1 | E4 x1, E7 x1 |
| container | 0x23 | grenade 31, SPECIAL 1 | E5 x1 |
| container | 0x24 | coffee 32 | E1 x1, E4 x1, E5 x1 |
| container | 0x25 | chocolate 24, steak 4, cake 4 | E7 x1 |
| container | 0x27 | gold dust 24, SPECIAL 8 | E1 x1, E2 x1, E7 x1 |
| container | 0x28 | pizza 28, steak 2, sushi 2 | E4 x1, E6 x1 |
| container | 0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 | E2 x3, E6 x7, E7 x3, E8 x10 |
| container | 0x2a | dime 22, dynamite 10 | E1 x1, E4 x1, E7 x1 |
| container | 0x2b | gun 31, Uzi 1 | E2 x1, E5 x1 |
| container | 0x2c | dime 24, Uzi 8 | E7 x1 |
| container | 0x2f | rod 26, rifle 5, M-16 1 | E4 x1 |
| container | 0x30 | diamond 31, dime 1 | E7 x1 |
| container | 0x31 | RADIO 32 | E3 x4 |
| container | 0x32 | sushi 30, hot dog 2 | E7 x1 |
| container | 0x33 | hot dog 30, sushi 2 | E7 x1 |
| container | 0x34 | cake 28, gum 4 | E4 x1 |
| container | 0x35 | burger 30, cake 2 | E1 x1 |
| enemy | 0x01 | burger 8, hot dog 8, pizza 8, salad 8 | E4 x1 |
| enemy | 0x02 | SPECIAL 32 | E2 x1, E4 x2 |
| enemy | 0x03 | nothing 16, SPECIAL 16 | E4 x1, E7 x1 |
| enemy | 0x04 | nothing 24, pearl 1, necklace 1, ring 1, watch 1, dime 1, 1000$ 1, sunglass 1, glove 1 | E1 x2, E4 x4, E5 x1 |
| enemy | 0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 | E4 x5, E5 x4, E6 x10, E7 x2 |
| enemy | 0x07 | nothing 31, steak 1 | E4 x1 |
| enemy | 0x08 | coffee 7, croissant 7, gum 6, chocolate 6, donut 6 | E7 x2 |
| enemy | 0x09 | nothing 28, SPECIAL 3, rifle 1 | E2 x3, E6 x5 |
| enemy | 0x0b | lobster 32 | E4 x1 |
| enemy | 0x0c | fries 7, gum 7, chocolate 7, donut 7, nothing 4 | E4 x1, E5 x1 |
| enemy | 0x0e | hot dog 16, pizza 16 | E2 x1, E8 x1 |
| enemy | 0x10 | grenade 24, nothing 8 | E2 x1, E4 x3, E5 x2, E7 x6 |
| enemy | 0x11 | nothing 24, SPECIAL 8 | E2 x3, E5 x1, E7 x2 |
| enemy | 0x12 | Uzi 32 | E8 x1 |
| enemy | 0x13 | dynamite 32 | E4 x4 |
| enemy | 0x14 | nothing 31, diamond 1 | E4 x2, E5 x5, E7 x5 |
| enemy | 0x15 | nothing 16, dynamite 16 | E1 x1, E5 x1, E7 x3 |
| enemy | 0x16 | gum 6, chocolate 6, donut 6, coffee 6, croissant 6, fries 2 | E5 x1 |
| enemy | 0x17 | nothing 27, rifle 5 | E2 x1, E4 x2 |
| enemy | 0x18 | parfait 32 | E4 x1 |
| enemy | 0x19 | nothing 28, SPECIAL 4 | E1 x1, E4 x1, E5 x1 |
| enemy | 0x1c | nothing 24, dynamite 8 | E7 x9 |
| enemy | 0x1d | gun 24, nothing 8 | E7 x1 |
| enemy | 0x1e | nothing 16, coffee 4, gum 3, chocolate 3, donut 3, croissant 3 | E4 x4, E7 x3 |
| enemy | 0x20 | gun 32 | E4 x2, E5 x2 |
| enemy | 0x22 | watch 5, dime 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2, nothing 1 | E2 x4, E8 x12 |
| enemy | 0x23 | donut 30, nothing 1, barbecue 1 | E2 x1 |
| enemy | 0x24 | gun 4, knife 4, rod 4, stick 4, watch 4, dime 4, 1000$ 4, sunglass 4 | E5 x1 |
| enemy | 0x25 | grenade 32 | E1 x1, E5 x1 |
| enemy | 0x27 | shotgun 24, Uzi 7, bazooka 1 | E1 x1 |
| enemy | 0x28 | RADIO 32 | E3 x8 |
| enemy | 0x30 | steak 32 | E1 x1 |


Rows that no script and no helper list uses: container rows 0x08 (rifle 16 and M-16 16), 0x0A (sushi), 0x0D (hot dog), 0x0E (all 17 foods, 1 or 2 of each), 0x14 (hot dog 8, dime 24), 0x16 (burger 16,
fries 8, coffee 8), 0x1A (cake, parfait, pudding, fries), 0x1E (parfait), 0x26 (pizza 28, gum 4), 0x2D (pizza 16, donut 16), 0x2E (steak 16, burger 16) and 0x36 to 0x40 (steak, 11 rows); enemy rows 0x06,
0x0D, 0x1A, 0x1B, 0x21 (one roll over almost every weapon type and the special: the "random weapon" row), 0x26, 0x29 (M-16 29 of 32, bazooka 1), 0x2F and 0x31 to 0x3A (steak, 10 rows). Enemy row 0x30 is
used once (Episode 1 wave 4, a Hammer with a sure steak). The full decode of both tables is `re/specs/enemy/items_drops.txt`. Unused rows may be leftovers of tuning (open item 7, section 13).

## 7. What every episode hands out

### 7.1 How to read the tables and the totals

- Records come from `re/specs/enemy/stage_scripts.txt`: list 2 of each area (containers, placed items, enemies that carry a row) and the wave tables (enemies). "Trigger" is the camera X at which the
  record fires; an object appears where its X lies relative to the screen at that moment ("appears 132 px from the left edge" means it pops into view inside the picture). Trigger 0 uses the camera of
  the area start (normally X 0). R: SPEC 8.1.
- Flags: `2P+` = only with 2 or 3 players (flag bits 0x40), `3P` = only with 3 players (0x60), `lv<=n`, `lv>=n` = only at that level or below or above (bits 0x10 and 0x08, value in the high byte). The
  gate is tested when the record is made (`spawn_record` 0xFDAE), so a join during a wave changes later records at once. R: SPEC 8.1; M e1a0_c*p, e7a0_lv*.
- Enemy variants: `var` is the record's `+$26` word (low byte = name or kind, high byte = entrance, D3a). A row on an enemy is the loot of that unit; most units carry no row at all.

What the scripts contain (R: scan of the lists):

| Episode | Containers (empty ones) | Rolling barrels (with a row) | Falling rocks | Enemy records with a drop row | Items lying on the floor | Boss helper rows |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 11 (0) | 0 | 0 | 7 | 0 | 10 (Vice) |
| 2 | 11 (0) | 0 | 0 | 15 | 4 | 4 (Butcher) |
| 3 | 20 (17) | 22 (1) | 0 | 8 (all radio rows) | 0 | 30 (Hogg) |
| 4 | 23 (9) | 0 | 0 | 35 | 8 | 17 (Slice) |
| 5 | 19 (0) | 0 | 6 | 21 | 10 | none |
| 6 | 8 (0) | 6 (6) | 0 | 15 | 2 | none |
| 7 | 11 (0) | 13 (13) | 0 | 34 | 6 | none |
| 8 | 4 (0) | 11 (11) | 0 | 14 | 0 | none |

(The helper rows count the lines of table 7.3 before grouping.) Containers: pails 17, drums 35, barrels 37, statues 4, tires 14; none of stone and boulder.

### 7.2 The tables (every record with a row, every placed item, every container)

Short guide per episode before the table (what the player meets first):

- **Episode 1.** Area 0: a drum (a treasure from a mixed row), a drum (dime 22 or dynamite 10 of 32), a pail with a rod (2P+) and a pail with a club (3P). Area 1 hall: four golden statues hold salad,
  gold dust or ammo, a gun (31 of 32 a revolver) and gold dust; the first thug (Gneiss) carries row E:0x27 shotgun 24, Uzi 7, bazooka 1 of 32. Area 2: a pail with a burger (30 of 32) or cake, a thug
  with a grenade, a tire stack with burger or pizza (2P+), a drum with coffee (3P). Wave 4 Hammer: a steak (enemy row 0x30, which the port cannot read yet: section 12, IT-P1).
- **Episode 2.** Area 0: a barbecue barrel and an Uzi barrel at X 0x360 and 0x3A0 beside the sleeping dinosaur, three treasures behind the trunks, poachers with the special row. Area 2: eight containers
  (two tire stacks and six drums at X 0xA0 to 0x280): steak, gold dust or ammo, treasure, dynamite, small food, revolver. Waves: hunters (A79) drop treasure and donuts; the alternate Blades drop treasure.
- **Episode 3.** Only radios (section 7.6).
- **Episode 4.** Area 0: a stone on the floor, a treasure pail, a shotgun drum (X 0x2D0), an explosive drum (dime), a small food drum, a fries drum (2P+). Area 1 garage: tires with pizza and tires with gold
  dust at the back row, a cake drum (3P), grenade and dynamite drums, a ring on the floor; bikers carry grenade, steak and small food rows; a pail with rod, rifle or M-16, a coffee drum and an explosive
  drum with the special. Area 2: the weapon yard (placed rifle, shotgun for 2P+, M-16 for 3P and one more M-16 below level 9). Waves: dynamite punks, Walther with a revolver (100 percent).
- **Episode 5.** Start: the barbecues, the egg, the torches and the diamond (section 8.1). Area 0 road: a bazooka barrel, an explosive barrel with small food, a grenade pail, a gun drum. Area 1 village:
  a row of eight barrels (barbecue, lobster, salad, treasure, Uzi, rod, treasure, special) and three pails (special, ammonite, cake or coffee by level). Many Blade and Walther rows.
- **Episode 6.** Area 0 cabins: seven barrels in a row (grenade, four treasure, gold dust, burger or pizza) and a gold bar. Area 1 jungle: six rolling barrels (grenade, pizza, treasure). Area 2: the salad
  barrel before the boss. Hammers carry a 5 of 32 small food; poachers a 1 of 32 rifle (so "no firearms" of L09 is almost true, not quite).
- **Episode 7.** Area 0 roof: bazooka barrel, Uzi barrel (level gated), a small food barrel, a hot dog barrel (3P), dynamite punks, a ring and a stone on the floor. Area 2 library: ten rolling barrels
  (small food, treasure, a rod, special, dynamite) and three treasures on the floor. Area 3 lab: four barrels (small food, ammonite, club, sushi or hot dog by level). Wave 12: pudding and diamond barrels.
- **Episode 8.** Area 0: three bomb carriers (treasure, **Uzi 32 of 32**, treasure). Area 2 cave: three barrels (small food, Uzi, treasure), eleven rolling barrels, and the meat barrel at X 0xCB0.
  Alternate Blades and Punks drop treasure (row E:0x22).


#### Episode 1

| Where | Trigger | X | Object | Flags | Row | What it holds (of 32) |
| --- | --- | --- | --- | --- | --- | --- |
| area 0 | 0x410 | 0x5a0 | drum (appears off screen) | - | C:0x02 | dime 15, pearl 3, necklace 3, ring 3, watch 3, 1000$ 3, sunglass 1, glove 1 |
| area 0 | 0x420 | 0x600 | thug A2 var 0x2 | - | E:0x04 | nothing 24, pearl 1, necklace 1, ring 1, watch 1, dime 1, 1000$ 1, sunglass 1, glove 1 |
| area 0 | 0x420 | 0x630 | thug A2 var 0x202 | 3P | E:0x04 | nothing 24, pearl 1, necklace 1, ring 1, watch 1, dime 1, 1000$ 1, sunglass 1, glove 1 |
| area 0 | 0x450 | 0x5d0 | drum (appears off screen) | - | C:0x2a | dime 22, dynamite 10 |
| area 0 | 0x450 | 0x5d4 | pail (appears off screen) | 2P+ | C:0x12 | rod 32 |
| area 0 | 0x460 | 0x5f6 | pail (appears off screen) | 3P | C:0x04 | club 32 |
| area 1 | 0x0 | 0x81 | statue (appears 129 px from the left edge) | - | C:0x0c | salad 32 |
| area 1 | 0x0 | 0x14b | statue (appears 331 px from the left edge) | - | C:0x27 | gold dust 24, SPECIAL 8 |
| area 1 | 0x1 | 0x1a0 | thug A2 var 0x202 | - | E:0x27 | shotgun 24, Uzi 7, bazooka 1 |
| area 1 | 0xc0 | 0x280 | statue (appears off screen) | - | C:0x01 | gun 31, M-16 1 |
| area 1 | 0x120 | 0x2a0 | thug A2 var 0x0 | - | E:0x19 | nothing 28, SPECIAL 4 |
| area 1 | 0x182 | 0x342 | statue (appears off screen) | - | C:0x15 | gold dust 28, dime 4 |
| area 2 | 0x0 | 0x190 | drum (appears off screen) | 3P | C:0x24 | coffee 32 |
| area 2 | 0x0 | 0x1d0 | tires (appears off screen) | 2P+ | C:0x0f | burger 16, pizza 16 |
| area 2 | 0x30 | 0x1f0 | pail (appears off screen) | - | C:0x35 | burger 30, cake 2 |
| area 2 | 0x30 | 0x30 | thug A2 var 0x202 | - | E:0x25 | grenade 32 |
| wave 1 | wave | 0x190 | Punk/Thug A20 var 0x0 | 2P+ | E:0x15 | nothing 16, dynamite 16 |
| wave 4 | wave | 0xd0 | A10 (Hammer/Elmer/Wrench) var 0x202 | - | E:0x30 | steak 32 |

#### Episode 2

| Where | Trigger | X | Object | Flags | Row | What it holds (of 32) |
| --- | --- | --- | --- | --- | --- | --- |
| area 0 | 0x0 | 0x1a8 | treasure on the floor: diamond (appears off screen) | - | - | (placed item) |
| area 0 | 0x168 | 0x30a | treasure on the floor: 1000$ (appears off screen) | - | - | (placed item) |
| area 0 | 0x1c0 | 0x360 | barrel (appears off screen) | - | C:0x0b | barbecue 32 |
| area 0 | 0x200 | 0x3a0 | barrel (appears off screen) | - | C:0x06 | Uzi 31, M-16 1 |
| area 0 | 0x280 | 0x420 | poacher A13 var 0x0 | - | E:0x11 | nothing 24, SPECIAL 8 |
| area 0 | 0x2c8 | 0x468 | treasure on the floor: dime (appears off screen) | - | - | (placed item) |
| area 0 | 0x380 | 0x520 | poacher A13 var 0x200 | 2P+ | E:0x17 | nothing 27, rifle 5 |
| area 0 | 0x428 | 0x1a8 | treasure on the floor: pearl (appears left of the screen) | - | - | (placed item) |
| area 0 | 0x480 | 0x620, 0x620 | 2 x poacher A13 var 0x0 | - | E:0x11 | nothing 24, SPECIAL 8 |
| area 1 | 0xb38 | 0xcd8 | barrel (appears off screen) | - | C:0x1b | grenade 32 |
| area 2 | 0x0 | 0xa0 | tires (appears 160 px from the left edge) | - | C:0x09 | steak 32 |
| area 2 | 0x0 | 0xf8 | tires (appears 248 px from the left edge) | - | C:0x27 | gold dust 24, SPECIAL 8 |
| area 2 | 0x70 | 0x210, 0x210 | 2 x drum (appears off screen) | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 2 | 0xa8 | 0x248 | drum (appears off screen) | - | C:0x19 | dynamite 32 |
| area 2 | 0xa8 | 0x248 | drum (appears off screen) | - | C:0x03 | gum 8, chocolate 8, donut 8, coffee 8 |
| area 2 | 0xe0 | 0x280 | drum (appears off screen) | - | C:0x2b | gun 31, Uzi 1 |
| area 2 | 0xe0 | 0x280 | drum (appears off screen) | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| wave 2 | wave | 0x5e8 | poacher A13 var 0x212 | - | E:0x09 | nothing 28, SPECIAL 3, rifle 1 |
| wave 2 | wave | 0x668 | poacher A13 var 0x212 | 2P+ | E:0x02 | SPECIAL 32 |
| wave 2 | wave | 0x708 | hunter A79 var 0x0 | - | E:0x22 | watch 5, dime 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2, nothing 1 |
| wave 2 | wave | 0x548 | hunter A79 var 0x0 | - | E:0x23 | donut 30, nothing 1, barbecue 1 |
| wave 2 | wave | 0x530, 0x530, 0x530 | 3 x Blade alt A63 var 0x0 | - | E:0x22 | watch 5, dime 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2, nothing 1 |
| wave 3 | wave | 0x840 | poacher A13 var 0x40c | - | E:0x10 | grenade 24, nothing 8 |
| wave 5 | wave | 0xd20, 0xd20 | 2 x poacher A13 var 0x400 | - | E:0x09 | nothing 28, SPECIAL 3, rifle 1 |
| wave 5 | wave | 0xd20 | poacher A13 var 0x200 | 3P | E:0x0e | hot dog 16, pizza 16 |

#### Episode 3

| Where | Trigger | X | Object | Flags | Row | What it holds (of 32) |
| --- | --- | --- | --- | --- | --- | --- |
| area 0 | 0x80 | 0x250 | barrel (appears off screen) | - | C:0x31 | RADIO 32 |
| area 0 | 0x160 | 0x320 | A10 (Hammer/Elmer/Wrench) var 0x4 | - | E:0x28 | RADIO 32 |
| area 0 | 0x380 | 0x580 | pail (appears off screen) | - | C:0x31 | RADIO 32 |
| area 0 | 0x7c0 | 0x980 | Punk/Thug A20 var 0x0 | - | E:0x28 | RADIO 32 |
| area 0 | 0xb40 | 0xd40 | Vice/Walther A18 var 0x200 | - | E:0x28 | RADIO 32 |
| area 0 | 0xd00 | 0xec0 | Punk/Wrench alt A64 var 0x200 | - | E:0x28 | RADIO 32 |
| area 0 | 0x1280 | 0x1480 | A10 (Hammer/Elmer/Wrench) var 0x8 | - | E:0x28 | RADIO 32 |
| area 0 | 0x19c0 | 0x1bc0 | Punk/Thug A20 var 0x200 | - | E:0x28 | RADIO 32 |
| area 0 | 0x1e80 | 0x2080 | drum (appears off screen) | - | C:0x31 | RADIO 32 |
| area 0 | 0x2790 | 0x2990 | rolling barrel A65 var 0x0 | - | C:0x31 | RADIO 32 |
| area 0 | | 0x500, 0x520, 0x540, 0x540, 0x560, 0xb00, 0xb70, 0xc00, 0xc80, 0xd00, 0xd80, 0x1f80, 0x2000, 0x2100, 0x2180, 0x2200, 0x2280 | 17 empty containers (5 pail, 6 tires, 6 drum) | | C:0x00 | nothing |
| wave 2 | wave | 0x8d0 | biker A70 var 0x0 | - | E:0x28 | RADIO 32 |
| wave 4 | wave | 0xfc0 | A10 (Hammer/Elmer/Wrench) var 0x4 | - | E:0x28 | RADIO 32 |

#### Episode 4

| Where | Trigger | X | Object | Flags | Row | What it holds (of 32) |
| --- | --- | --- | --- | --- | --- | --- |
| area 0 | 0x0 | 0x14b | weapon on the floor: stone (size 2) (appears 331 px from the left edge) | - | - | (placed item) |
| area 0 | 0x100 | 0x2a8 | pail (appears off screen) | - | C:0x02 | dime 15, pearl 3, necklace 3, ring 3, watch 3, 1000$ 3, sunglass 1, glove 1 |
| area 0 | 0x100 | 0x2d0 | drum (appears off screen) | - | C:0x11 | shotgun 32 |
| area 0 | 0x1a0 | 0x350 | drum (explosive) (appears off screen) | - | C:0x22 | dime 31, SPECIAL 1 |
| area 0 | 0x1b8 | 0x390 | drum (appears off screen) | - | C:0x03 | gum 8, chocolate 8, donut 8, coffee 8 |
| area 0 | 0x210 | 0x3d0 | drum (appears off screen) | 2P+ | C:0x13 | fries 32 |
| area 0 | | 0x20, 0x40, 0x60, 0x60, 0x80, 0xa0, 0xa0, 0xc0, 0xe0 | 9 empty containers (6 drum, 3 tires) | | C:0x00 | nothing |
| area 1 | 0x0 | 0xb0 | tires (appears 176 px from the left edge) | - | C:0x28 | pizza 28, steak 2, sushi 2 |
| area 1 | 0x0 | 0xf0 | tires (appears 240 px from the left edge) | - | C:0x15 | gold dust 28, dime 4 |
| area 1 | 0x0 | 0x120 | drum (appears 288 px from the left edge) | 3P | C:0x34 | cake 28, gum 4 |
| area 1 | 0x60 | 0x23a | drum (appears off screen) | - | C:0x1c | grenade 32 |
| area 1 | 0x60 | 0x26a | drum (appears off screen) | - | C:0x2a | dime 22, dynamite 10 |
| area 1 | 0x102 | 0xe0 | A10 (Hammer/Elmer/Wrench) var 0x204 | - | E:0x1e | nothing 16, coffee 4, gum 3, chocolate 3, donut 3, croissant 3 |
| area 1 | 0x200 | 0x3df | treasure on the floor: ring (appears off screen) | - | - | (placed item) |
| area 1 | 0x360 | 0x500 | biker A70 var 0x0 | - | E:0x10 | grenade 24, nothing 8 |
| area 1 | 0x360 | 0x560 | biker A70 var 0x100 | - | E:0x07 | nothing 31, steak 1 |
| area 1 | 0x360 | 0x528 | biker A70 var 0x0 | - | E:0x0c | fries 7, gum 7, chocolate 7, donut 7, nothing 4 |
| area 1 | 0x400 | 0x5b0 | poacher A13 var 0x402 | 3P | E:0x10 | grenade 24, nothing 8 |
| area 1 | 0x420 | 0x5f0 | poacher A13 var 0x2 | - | E:0x10 | grenade 24, nothing 8 |
| area 1 | 0x6e0 | 0x8c0 | pail (appears off screen) | - | C:0x2f | rod 26, rifle 5, M-16 1 |
| area 1 | 0x700 | 0x8f0 | drum (appears off screen) | - | C:0x24 | coffee 32 |
| area 1 | 0x820 | 0x9c0 | drum (explosive) (appears off screen) | - | C:0x07 | SPECIAL 32 |
| area 2 | 0x0 | 0xe0 | treasure on the floor: dime (appears 224 px from the left edge) | lv<=25 | - | (placed item) |
| area 2 | 0x0 | 0x12a | treasure on the floor: 1000$ (appears 298 px from the left edge) | 2P+ | - | (placed item) |
| area 2 | 0x8 | 0x1c0 | A10 (Hammer/Elmer/Wrench) var 0x204 | 3P | E:0x1e | nothing 16, coffee 4, gum 3, chocolate 3, donut 3, croissant 3 |
| area 2 | 0x8 | 0x1a0 | A10 (Hammer/Elmer/Wrench) var 0x4 | - | E:0x1e | nothing 16, coffee 4, gum 3, chocolate 3, donut 3, croissant 3 |
| area 2 | 0x8 | 0x0 | A10 (Hammer/Elmer/Wrench) var 0x404 | lv>=26 | E:0x1e | nothing 16, coffee 4, gum 3, chocolate 3, donut 3, croissant 3 |
| area 2 | 0x80 | 0x250 | drum (appears off screen) | - | C:0x06 | Uzi 31, M-16 1 |
| area 2 | 0x1b0 | 0x350 | weapon on the floor: shotgun (appears off screen) | 2P+ | - | (placed item) |
| area 2 | 0x1b0 | 0x350 | weapon on the floor: rifle (appears off screen) | - | - | (placed item) |
| area 2 | 0x200 | 0x3a0 | weapon on the floor: M-16 (appears off screen) | 3P | - | (placed item) |
| area 2 | 0x250 | 0x3f0 | weapon on the floor: M-16 (appears off screen) | lv<=8 | - | (placed item) |
| wave 0 | wave | 0x420 | thug A2 var 0x402 | - | E:0x19 | nothing 28, SPECIAL 4 |
| wave 0 | wave | 0x260 | thug A2 var 0x200 | - | E:0x04 | nothing 24, pearl 1, necklace 1, ring 1, watch 1, dime 1, 1000$ 1, sunglass 1, glove 1 |
| wave 0 | wave | 0x420 | thug A2 var 0x2 | - | E:0x04 | nothing 24, pearl 1, necklace 1, ring 1, watch 1, dime 1, 1000$ 1, sunglass 1, glove 1 |
| wave 1 | wave | 0x320 | Vice/Walther A18 var 0x200 | - | E:0x20 | gun 32 |
| wave 1 | wave | 0x180 | Punk/Thug A20 var 0x4 | 2P+ | E:0x13 | dynamite 32 |
| wave 1 | wave | 0x180 | Vice/Walther A18 var 0x200 | 3P | E:0x20 | gun 32 |
| wave 1 | wave | 0x180 | Punk/Wrench alt A64 var 0x0 | - | E:0x03 | nothing 16, SPECIAL 16 |
| wave 1 | wave | 0x180, 0x180 | 2 x Punk/Wrench alt A64 var 0x0 | - | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 1 | wave | 0x320, 0x180 | 2 x thug A2 var 0x402 | - | E:0x04 | nothing 24, pearl 1, necklace 1, ring 1, watch 1, dime 1, 1000$ 1, sunglass 1, glove 1 |
| wave 1 | wave | 0x320 | biker A70 var 0x0 | - | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 1 | wave | 0x180, 0x320 | 2 x biker A70 var 0x100 | - | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 3 | wave | 0x7e0, 0xa20 | 2 x Punk/Thug A20 var 0x4 | - | E:0x13 | dynamite 32 |
| wave 3 | wave | 0xa20 | Punk/Thug A20 var 0x4 | lv>=16 | E:0x13 | dynamite 32 |
| wave 5 | wave | 0x2f0 | thug A2 var 0x20a | lv<=16 | E:0x02 | SPECIAL 32 |
| wave 5 | wave | 0x2f0 | thug A2 var 0x20a | - | E:0x02 | SPECIAL 32 |
| wave 5 | wave | 0x440 | A10 (Hammer/Elmer/Wrench) var 0x0 | lv<=16 | E:0x0b | lobster 32 |
| wave 5 | wave | 0x440 | A10 (Hammer/Elmer/Wrench) var 0x200 | lv>=17 | E:0x18 | parfait 32 |
| wave 5 | wave | 0x2f0 | A10 (Hammer/Elmer/Wrench) var 0x0 | 3P | E:0x01 | burger 8, hot dog 8, pizza 8, salad 8 |
| wave 5 | wave | 0x400 | poacher A13 var 0x206 | 2P+ | E:0x17 | nothing 27, rifle 5 |
| wave 5 | wave | 0x400 | poacher A13 var 0x206 | 3P | E:0x17 | nothing 27, rifle 5 |
| wave 5 | wave | 0x400 | Blade/Razor A14 var 0x206 | lv>=27 | E:0x14 | nothing 31, diamond 1 |
| wave 5 | wave | 0x400 | Blade/Razor A14 var 0x206 | lv>=30 | E:0x14 | nothing 31, diamond 1 |

#### Episode 5

| Where | Trigger | X | Object | Flags | Row | What it holds (of 32) |
| --- | --- | --- | --- | --- | --- | --- |
| area 0 | 0x0 | 0x10 | treasure on the floor: gold dust (appears 16 px from the left edge) | - | - | (placed item) |
| area 0 | 0x0 | 0x116, 0x102 | 2 x food on the floor: barbecue (appears 278 px from the left edge) | - | - | (placed item) |
| area 0 | 0x0 | 0xfa | treasure on the floor: egg (appears 250 px from the left edge) | - | - | (placed item) |
| area 0 | 0x0 | 0xdd, 0xf4, 0x108 | 3 x weapon on the floor: torch (appears 221 px from the left edge) | - | - | (placed item) |
| area 0 | 0x80 | 0x2b0 | pail (appears off screen) | 2P+ | C:0x15 | gold dust 28, dime 4 |
| area 0 | 0x80 | 0x2d0 | drum (appears off screen) | 3P | C:0x05 | dynamite 16, grenade 16 |
| area 0 | 0x180 | 0x204 | treasure on the floor: diamond (appears 132 px from the left edge) | - | - | (placed item) |
| area 0 | 0x360 | 0x4f0 | thug A2 var 0x204 | - | E:0x11 | nothing 24, SPECIAL 8 |
| area 0 | 0x360 | 0x510 | thug A2 var 0x4 | - | E:0x24 | gun 4, knife 4, rod 4, stick 4, watch 4, dime 4, 1000$ 4, sunglass 4 |
| area 0 | 0x5d0 | 0x760 | thug A2 var 0x4 | - | E:0x04 | nothing 24, pearl 1, necklace 1, ring 1, watch 1, dime 1, 1000$ 1, sunglass 1, glove 1 |
| area 0 | 0x600 | 0x790 | thug A2 var 0x4 | - | E:0x19 | nothing 28, SPECIAL 4 |
| area 0 | 0x600 | 0x790 | barrel (appears off screen) | - | C:0x21 | bazooka 32 |
| area 0 | 0x660 | 0x7e8 | barrel (explosive) (appears off screen) | - | C:0x03 | gum 8, chocolate 8, donut 8, coffee 8 |
| area 0 | 0x880 | 0xa20 | pail (appears off screen) | - | C:0x23 | grenade 31, SPECIAL 1 |
| area 0 | 0x880 | 0xa50 | drum (appears off screen) | - | C:0x2b | gun 31, Uzi 1 |
| area 0 | 0x880 | 0xa80 | drum (appears off screen) | 3P | C:0x13 | fries 32 |
| area 1 | 0xa50 | 0xbfa | barrel (appears off screen) | - | C:0x0b | barbecue 32 |
| area 1 | 0xa80 | 0xc20 | barrel (appears off screen) | - | C:0x18 | lobster 32 |
| area 1 | 0xa90 | 0xc3a | barrel (appears off screen) | - | C:0x0c | salad 32 |
| area 1 | 0xac0 | 0xc60 | barrel (appears off screen) | - | C:0x02 | dime 15, pearl 3, necklace 3, ring 3, watch 3, 1000$ 3, sunglass 1, glove 1 |
| area 1 | 0xad0 | 0xc7a | barrel (appears off screen) | - | C:0x06 | Uzi 31, M-16 1 |
| area 1 | 0xb00 | 0xca0 | barrel (appears off screen) | - | C:0x12 | rod 32 |
| area 1 | 0xb10 | 0xcba | barrel (appears off screen) | - | C:0x02 | dime 15, pearl 3, necklace 3, ring 3, watch 3, 1000$ 3, sunglass 1, glove 1 |
| area 1 | 0xb40 | 0xce0 | barrel (appears off screen) | - | C:0x07 | SPECIAL 32 |
| area 1 | 0xd80 | 0xf68 | pail (appears off screen) | - | C:0x07 | SPECIAL 32 |
| area 1 | 0xe00 | 0xf70 | weapon on the floor: torch (appears 368 px from the left edge) | 2P+ | - | (placed item) |
| area 1 | 0xe00 | 0xfa0 | weapon on the floor: torch (appears off screen) | - | - | (placed item) |
| area 1 | 0xe00 | 0xf90 | pail (appears off screen) | - | C:0x1f | ammonite 30, dime 2 |
| area 1 | 0xe00 | 0xfb8 | pail (appears off screen) | lv<=16 | C:0x20 | cake 32 |
| area 1 | 0xe00 | 0xfb8 | pail (appears off screen) | lv>=17 | C:0x24 | coffee 32 |
| wave 1 | wave | 0x570 | Blade/Razor A14 var 0x20a | 3P | E:0x14 | nothing 31, diamond 1 |
| wave 1 | wave | 0x3b0 | Blade/Razor A14 var 0x20a | 2P+ | E:0x14 | nothing 31, diamond 1 |
| wave 3 | wave | 0xae0 | A10 (Hammer/Elmer/Wrench) var 0x0 | 3P | E:0x0c | fries 7, gum 7, chocolate 7, donut 7, nothing 4 |
| wave 4 | wave | 0xa40 | Vice/Walther A18 var 0x200 | - | E:0x20 | gun 32 |
| wave 4 | wave | 0xc40 | Blade/Razor A14 var 0x20a | - | E:0x14 | nothing 31, diamond 1 |
| wave 4 | wave | 0xc40 | Blade/Razor A14 var 0xa | 3P | E:0x14 | nothing 31, diamond 1 |
| wave 4 | wave | 0xa40 | Blade/Razor A14 var 0xa | - | E:0x14 | nothing 31, diamond 1 |
| wave 4 | wave | 0xc40 | Vice/Walther A18 var 0x200 | 2P+ | E:0x20 | gun 32 |
| wave 4 | wave | 0xb20 | poacher A13 var 0x6 | - | E:0x25 | grenade 32 |
| wave 6 | wave | 0x10a0 | A10 (Hammer/Elmer/Wrench) var 0x200 | - | E:0x16 | gum 6, chocolate 6, donut 6, coffee 6, croissant 6, fries 2 |
| wave 6 | wave | 0xec0 | poacher A13 var 0x8 | - | E:0x10 | grenade 24, nothing 8 |
| wave 7 | wave | 0x1400 | A10 (Hammer/Elmer/Wrench) var 0x406 | lv>=8 | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 7 | wave | 0x1260 | A10 (Hammer/Elmer/Wrench) var 0x406 | lv>=10 | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 7 | wave | 0x1260 | Punk/Thug A20 var 0x206 | 2P+ | E:0x15 | nothing 16, dynamite 16 |
| wave 7 | wave | 0x1400 | poacher A13 var 0x408 | lv>=20 | E:0x10 | grenade 24, nothing 8 |
| wave 7 | wave | 0x1400 | A10 (Hammer/Elmer/Wrench) var 0x204 | 2P+ | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 7 | wave | 0x1260 | A10 (Hammer/Elmer/Wrench) var 0x204 | 3P | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |

#### Episode 6

| Where | Trigger | X | Object | Flags | Row | What it holds (of 32) |
| --- | --- | --- | --- | --- | --- | --- |
| area 0 | 0x290 | 0x440 | barrel (appears off screen) | - | C:0x1b | grenade 32 |
| area 0 | 0x2d0 | 0x480 | barrel (appears off screen) | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 0 | 0x310 | 0x4c0 | barrel (appears off screen) | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 0 | 0x350 | 0x500 | barrel (appears off screen) | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 0 | 0x380 | 0x3c0 | treasure on the floor: gold (appears 64 px from the left edge) | - | - | (placed item) |
| area 0 | 0x390 | 0x540 | barrel (appears off screen) | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 0 | 0x3d0 | 0x580 | barrel (appears off screen) | - | C:0x15 | gold dust 28, dime 4 |
| area 0 | 0x410 | 0x5c0 | barrel (appears off screen) | - | C:0x0f | burger 16, pizza 16 |
| area 1 | 0x0 | 0x90 | treasure on the floor: gold (appears 144 px from the left edge) | - | - | (placed item) |
| area 1 | 0x180 | 0x160 | rolling barrel A65 var 0x1 | - | C:0x15 | gold dust 28, dime 4 |
| area 1 | 0x180 | 0x160 | rolling barrel A65 var 0x1 | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 1 | 0x180 | 0x160 | rolling barrel A65 var 0x1 | - | C:0x1b | grenade 32 |
| area 1 | 0x280 | 0x260, 0x260 | 2 x rolling barrel A65 var 0x1 | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 1 | 0x280 | 0x260 | rolling barrel A65 var 0x1 | - | C:0x28 | pizza 28, steak 2, sushi 2 |
| area 2 | 0xd00 | 0xea0 | barrel (appears off screen) | - | C:0x0c | salad 32 |
| wave 1 | wave | 0x4e0 | A10 (Hammer/Elmer/Wrench) var 0x200 | - | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 1 | wave | 0x6a0 | A10 (Hammer/Elmer/Wrench) var 0x0 | 2P+ | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 2 | wave | 0x710 | A10 (Hammer/Elmer/Wrench) var 0x206 | - | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 2 | wave | 0x710 | A10 (Hammer/Elmer/Wrench) var 0x404 | - | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 2 | wave | 0x8d0 | A10 (Hammer/Elmer/Wrench) var 0x6 | - | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 2 | wave | 0x8d0 | A10 (Hammer/Elmer/Wrench) var 0x4 | 2P+ | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 2 | wave | 0x7d0 | poacher A13 var 0x404 | - | E:0x09 | nothing 28, SPECIAL 3, rifle 1 |
| wave 2 | wave | 0x8d0 | poacher A13 var 0x408 | - | E:0x09 | nothing 28, SPECIAL 3, rifle 1 |
| wave 2 | wave | 0x7d0 | poacher A13 var 0x404 | 2P+ | E:0x09 | nothing 28, SPECIAL 3, rifle 1 |
| wave 5 | wave | 0xa60, 0xa60 | 2 x A10 (Hammer/Elmer/Wrench) var 0x4 | - | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 5 | wave | 0x8a0, 0x8a0 | 2 x A10 (Hammer/Elmer/Wrench) var 0x204 | - | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 6 | wave | 0xc80, 0xac0 | 2 x poacher A13 var 0x40c | - | E:0x09 | nothing 28, SPECIAL 3, rifle 1 |

#### Episode 7

| Where | Trigger | X | Object | Flags | Row | What it holds (of 32) |
| --- | --- | --- | --- | --- | --- | --- |
| area 0 | 0x0 | 0x120 | treasure on the floor: dime (appears 288 px from the left edge) | - | - | (placed item) |
| area 0 | 0x280 | 0x470 | biker A70 var 0x100 | - | E:0x11 | nothing 24, SPECIAL 8 |
| area 0 | 0x2a0 | 0x4b0 | biker A70 var 0x0 | - | E:0x1c | nothing 24, dynamite 8 |
| area 0 | 0x388 | 0x4e0, 0x4e0 | 2 x Punk/Thug A20 var 0xa | - | E:0x1c | nothing 24, dynamite 8 |
| area 0 | 0x388 | 0x4e0 | Punk/Thug A20 var 0xa | lv>=22 | E:0x1c | nothing 24, dynamite 8 |
| area 0 | 0x388 | 0x3d0, 0x3d0 | 2 x Punk/Thug A20 var 0x20a | - | E:0x1c | nothing 24, dynamite 8 |
| area 0 | 0x388 | 0x3d0 | Punk/Thug A20 var 0x20a | lv>=22 | E:0x1c | nothing 24, dynamite 8 |
| area 0 | 0x420 | 0x5c0 | barrel (appears off screen) | 3P | C:0x0f | burger 16, pizza 16 |
| area 0 | 0x460 | 0x600 | barrel (appears off screen) | - | C:0x21 | bazooka 32 |
| area 0 | 0x560 | 0x6f5 | treasure on the floor: ring (appears off screen) | - | - | (placed item) |
| area 0 | 0x590 | 0x720 | weapon on the floor: stone (size 1) (appears off screen) | - | - | (placed item) |
| area 0 | 0x600 | 0x7a0 | poacher A13 var 0x202 | - | E:0x10 | grenade 24, nothing 8 |
| area 0 | 0x6a0 | 0x860 | poacher A13 var 0x202 | - | E:0x10 | grenade 24, nothing 8 |
| area 0 | 0x840 | 0xa00 | barrel (appears off screen) | 3P | C:0x10 | hot dog 30, barbecue 1, dime 1 |
| area 0 | 0x880 | 0xa40 | barrel (appears off screen) | lv<=16 | C:0x06 | Uzi 31, M-16 1 |
| area 0 | 0x880 | 0xa40 | barrel (appears off screen) | lv>=17 | C:0x2c | dime 24, Uzi 8 |
| area 0 | 0x8c0 | 0xa80 | barrel (appears off screen) | - | C:0x25 | chocolate 24, steak 4, cake 4 |
| area 2 | 0x200 | 0x2b1 | treasure on the floor: gold (appears 177 px from the left edge) | - | - | (placed item) |
| area 2 | 0x200 | 0x1e0 | rolling barrel A65 var 0x0 | - | C:0x22 | dime 31, SPECIAL 1 |
| area 2 | 0x200 | 0x3c0 | rolling barrel A65 var 0x0 | - | C:0x03 | gum 8, chocolate 8, donut 8, coffee 8 |
| area 2 | 0x240 | 0x400 | rolling barrel A65 var 0x0 | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 2 | 0x280 | 0x440 | rolling barrel A65 var 0x0 | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 2 | 0x2c0 | 0x480 | rolling barrel A65 var 0x0 | - | C:0x03 | gum 8, chocolate 8, donut 8, coffee 8 |
| area 2 | 0x300 | 0x4c0 | rolling barrel A65 var 0x0 | - | C:0x03 | gum 8, chocolate 8, donut 8, coffee 8 |
| area 2 | 0x340 | 0x500 | rolling barrel A65 var 0x0 | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 2 | 0x2a0 | 0x45b | treasure on the floor: 1000$ (appears off screen) | - | - | (placed item) |
| area 2 | 0x400 | 0x5c0 | rolling barrel A65 var 0x0 | - | C:0x12 | rod 32 |
| area 2 | 0x400 | 0x5c0 | rolling barrel A65 var 0x0 | - | C:0x17 | SPECIAL 16, dime 16 |
| area 2 | 0x400 | 0x5e0 | rolling barrel A65 var 0x0 | - | C:0x2a | dime 22, dynamite 10 |
| area 2 | 0x420 | 0x5c1 | treasure on the floor: watch (appears off screen) | - | - | (placed item) |
| area 3 | 0x0 | 0x100 | barrel (appears 256 px from the left edge) | - | C:0x03 | gum 8, chocolate 8, donut 8, coffee 8 |
| area 3 | 0x0 | 0x140 | barrel (appears 320 px from the left edge) | - | C:0x1f | ammonite 30, dime 2 |
| area 3 | 0x0 | 0x180 | barrel (appears off screen) | - | C:0x04 | club 32 |
| area 3 | 0x0 | 0x1c0 | barrel (appears off screen) | lv<=17 | C:0x32 | sushi 30, hot dog 2 |
| area 3 | 0x0 | 0x1c0 | barrel (appears off screen) | lv>=18 | C:0x33 | hot dog 30, sushi 2 |
| area 3 | 0x100 | 0x2c0 | A10 (Hammer/Elmer/Wrench) var 0x408 | - | E:0x1e | nothing 16, coffee 4, gum 3, chocolate 3, donut 3, croissant 3 |
| area 3 | 0x180 | 0x340 | A10 (Hammer/Elmer/Wrench) var 0x408 | - | E:0x1e | nothing 16, coffee 4, gum 3, chocolate 3, donut 3, croissant 3 |
| wave 0 | wave | 0x190 | biker A70 var 0x0 | - | E:0x1d | gun 24, nothing 8 |
| wave 0 | wave | 0x370 | biker A70 var 0x100 | - | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 0 | wave | 0x190 | biker A70 var 0x100 | - | E:0x11 | nothing 24, SPECIAL 8 |
| wave 2 | wave | 0x930 | poacher A13 var 0xc | 3P | E:0x10 | grenade 24, nothing 8 |
| wave 2 | wave | 0x770 | poacher A13 var 0x40c | 3P | E:0x10 | grenade 24, nothing 8 |
| wave 3 | wave | 0xa65 | A10 (Hammer/Elmer/Wrench) var 0x202 | - | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| wave 6 | wave | 0xffc0, 0xffc0 | 2 x Punk/Thug A20 var 0x206 | - | E:0x1c | nothing 24, dynamite 8 |
| wave 6 | wave | 0x1c0, 0x1c0 | 2 x Punk/Thug A20 var 0x6 | - | E:0x15 | nothing 16, dynamite 16 |
| wave 6 | wave | 0xc0 | A10 (Hammer/Elmer/Wrench) var 0x2 | - | E:0x08 | coffee 7, croissant 7, gum 6, chocolate 6, donut 6 |
| wave 6 | wave | 0xc0 | A10 (Hammer/Elmer/Wrench) var 0x202 | 2P+ | E:0x1e | nothing 16, coffee 4, gum 3, chocolate 3, donut 3, croissant 3 |
| wave 7 | wave | 0x40 | poacher A13 var 0x6 | - | E:0x14 | nothing 31, diamond 1 |
| wave 7 | wave | 0x110 | poacher A13 var 0x206 | lv>=20 | E:0x14 | nothing 31, diamond 1 |
| wave 7 | wave | 0x140 | poacher A13 var 0x406 | lv>=24 | E:0x14 | nothing 31, diamond 1 |
| wave 8 | wave | 0x1c0 | A10 (Hammer/Elmer/Wrench) var 0x202 | - | E:0x08 | coffee 7, croissant 7, gum 6, chocolate 6, donut 6 |
| wave 11 | wave | 0x2e0 | Punk/Thug A20 var 0x208 | - | E:0x15 | nothing 16, dynamite 16 |
| wave 11 | wave | 0x350, 0x1d0 | 2 x poacher A13 var 0x208 | - | E:0x10 | grenade 24, nothing 8 |
| wave 12 | wave | 0x610 | rolling barrel A65 var 0x0 | - | C:0x1d | pudding 32 |
| wave 12 | wave | 0x610 | rolling barrel A65 var 0x0 | - | C:0x30 | diamond 31, dime 1 |
| wave 12 | wave | 0x820 | rolling barrel A65 var 0x0 | - | C:0x27 | gold dust 24, SPECIAL 8 |
| wave 12 | wave | 0x730 | thug A2 var 0x206 | - | E:0x03 | nothing 16, SPECIAL 16 |
| wave 13 | wave | 0x3f0, 0x210 | 2 x Blade/Razor A14 var 0xa | - | E:0x14 | nothing 31, diamond 1 |

#### Episode 8

| Where | Trigger | X | Object | Flags | Row | What it holds (of 32) |
| --- | --- | --- | --- | --- | --- | --- |
| area 0 | 0x188 | 0x3b8, 0x3b8 | 2 x Punk/Wrench alt A64 var 0x0 | - | E:0x22 | watch 5, dime 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2, nothing 1 |
| area 0 | 0x188 | 0x3b8 | Punk/Wrench alt A64 var 0x0 | - | E:0x12 | Uzi 32 |
| area 1 | 0x200 | 0x3a0 | A10 (Hammer/Elmer/Wrench) var 0x200 | - | E:0x0e | hot dog 16, pizza 16 |
| area 2 | 0x0 | 0x130 | barrel (appears 304 px from the left edge) | - | C:0x03 | gum 8, chocolate 8, donut 8, coffee 8 |
| area 2 | 0x0 | 0xf0 | barrel (appears 240 px from the left edge) | - | C:0x06 | Uzi 31, M-16 1 |
| area 2 | 0x0 | 0xb0 | barrel (appears 176 px from the left edge) | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 2 | 0x180 | 0x150 | rolling barrel A65 var 0x1 | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 2 | 0x200 | 0x2bf | rolling barrel A65 var 0x201 | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 2 | 0x280 | 0x33f | rolling barrel A65 var 0x201 | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 2 | 0x300 | 0x3bf | rolling barrel A65 var 0x201 | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 2 | 0x380 | 0x43f | rolling barrel A65 var 0x201 | - | C:0x03 | gum 8, chocolate 8, donut 8, coffee 8 |
| area 2 | 0x400 | 0x4bf | rolling barrel A65 var 0x201 | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 2 | 0x480 | 0x53f | rolling barrel A65 var 0x201 | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 2 | 0x500 | 0x5bf | rolling barrel A65 var 0x201 | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 2 | 0x580 | 0x63f | rolling barrel A65 var 0x201 | - | C:0x03 | gum 8, chocolate 8, donut 8, coffee 8 |
| area 2 | 0x600 | 0x6bf | rolling barrel A65 var 0x201 | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 2 | 0x680 | 0x73f | rolling barrel A65 var 0x201 | - | C:0x29 | dime 6, watch 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2 |
| area 2 | 0xb10 | 0xcb0 | barrel (appears off screen) | - | C:0x0b | barbecue 32 |
| wave 3 | wave | 0x350, 0x350 | 2 x Blade alt A63 var 0x0 | - | E:0x22 | watch 5, dime 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2, nothing 1 |
| wave 5 | wave | 0x540, 0x540, 0x720, 0x720 | 4 x Blade alt A63 var 0x0 | - | E:0x22 | watch 5, dime 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2, nothing 1 |
| wave 6 | wave | 0x820, 0x820, 0x9e0, 0x9e0 | 4 x Punk/Wrench alt A64 var 0x0 | - | E:0x22 | watch 5, dime 5, 1000$ 5, sunglass 5, glove 5, gold dust 2, necklace 2, ring 2, nothing 1 |


### 7.3 Boss helper lists (pool B class 22, Episodes 1 to 4 only)

The boss reinforcement controller (D3c section 1.7) makes helpers from its own lists; the records carry drop rows like the script records. Episodes 5 to 8 have no such list. The groups are numbered
as in `re/out/d3c/b22_lists.txt`; "alive <=" is the gate of the group.

| Episode (boss) | Enemy | Flags | Row | What it holds (of 32) | Where in the helper list |
| --- | --- | --- | --- | --- | --- |
| E1 (Vice) | thug A2 | - | E:0x07 | nothing 31, steak 1 | 1 x: g0 |
| E1 (Vice) | thug A2 | - | E:0x04 | nothing 24, pearl 1, necklace 1, ring 1, watch 1, dime 1, 1000$ 1, sunglass 1, glove 1 | 1 x: g0 |
| E1 (Vice) | thug A2 | 3P | E:0x04 | nothing 24, pearl 1, necklace 1, ring 1, watch 1, dime 1, 1000$ 1, sunglass 1, glove 1 | 1 x: g0 |
| E1 (Vice) | A10 Hammer/Elmer/Wrench | lv>=22 | E:0x08 | coffee 7, croissant 7, gum 6, chocolate 6, donut 6 | 1 x: g1 |
| E1 (Vice) | A10 Hammer/Elmer/Wrench | - | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 | 6 x: g4, g4, g5, g5, g6, g6 |
| E2 (Butcher) | A10 Hammer/Elmer/Wrench | - | E:0x01 | burger 8, hot dog 8, pizza 8, salad 8 | 1 x: g9 |
| E2 (Butcher) | A10 Hammer/Elmer/Wrench | - | E:0x0f | donut 28, burger 3, nothing 1 | 1 x: g33 |
| E2 (Butcher) | A10 Hammer/Elmer/Wrench | 2P+ | E:0x0a | hot dog 24, nothing 4, barbecue 4 | 1 x: g34 |
| E2 (Butcher) | A10 Hammer/Elmer/Wrench | 3P | E:0x01 | burger 8, hot dog 8, pizza 8, salad 8 | 1 x: g35 |
| E3 (Hogg) | thug A2 | - | E:0x2a | Uzi 30, nothing 2 | 2 x: g0, g19 |
| E3 (Hogg) | thug A2 | 3P | E:0x2b | shotgun 30, nothing 2 | 1 x: g0 |
| E3 (Hogg) | biker A70 | - | E:0x14 | nothing 31, diamond 1 | 13 x: g2, g7, g7, g9, g9, g14, g14, g22, g22, g23, g23, g24, g24 |
| E3 (Hogg) | thug A2 | - | E:0x2b | shotgun 30, nothing 2 | 2 x: g3, g6 |
| E3 (Hogg) | thug A2 | 3P | E:0x2c | gun 30, nothing 2 | 1 x: g3 |
| E3 (Hogg) | poacher A13 | - | E:0x2d | rifle 30, nothing 2 | 1 x: g5 |
| E3 (Hogg) | A10 Hammer/Elmer/Wrench | - | E:0x0f | donut 28, burger 3, nothing 1 | 1 x: g8 |
| E3 (Hogg) | thug A2 | - | E:0x03 | nothing 16, SPECIAL 16 | 1 x: g10 |
| E3 (Hogg) | thug A2 | 2P+ | E:0x2a | Uzi 30, nothing 2 | 1 x: g10 |
| E3 (Hogg) | A10 Hammer/Elmer/Wrench | - | E:0x19 | nothing 28, SPECIAL 4 | 1 x: g12 |
| E3 (Hogg) | Blade/Razor A14 | - | E:0x2b | shotgun 30, nothing 2 | 1 x: g13 |
| E3 (Hogg) | thug A2 | 3P | E:0x2d | rifle 30, nothing 2 | 1 x: g15 |
| E3 (Hogg) | thug A2 | - | E:0x2e | M-16 30, nothing 2 | 1 x: g16 |
| E3 (Hogg) | A10 Hammer/Elmer/Wrench | - | E:0x1e | nothing 16, coffee 4, gum 3, chocolate 3, donut 3, croissant 3 | 1 x: g17 |
| E3 (Hogg) | poacher A13 | - | E:0x02 | SPECIAL 32 | 1 x: g18 |
| E3 (Hogg) | poacher A13 | 2P+ | E:0x02 | SPECIAL 32 | 1 x: g20 |
| E4 (Slice) | Blade alt A63 | - | E:0x02 | SPECIAL 32 | 1 x: timed 10 |
| E4 (Slice) | Blade alt A63 | - | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 | 7 x: timed 1800, timed 90, timed 10, timed 10, timed 10, timed 10, timed 1800 |
| E4 (Slice) | Punk/Wrench alt A64 | - | E:0x02 | SPECIAL 32 | 1 x: timed 360 |
| E4 (Slice) | A10 Hammer/Elmer/Wrench | - | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 | 6 x: g4, g12, g22, g26, g31, g36 |
| E4 (Slice) | A10 Hammer/Elmer/Wrench | 2P+ | E:0x1f | burger 8, fries 8, donut 8, coffee 8 | 1 x: g8 |
| E4 (Slice) | A10 Hammer/Elmer/Wrench | - | E:0x02 | SPECIAL 32 | 1 x: g17 |


- **The Episode 3 gun order** of the web (L02, BS27) is confirmed by the rows: Uzi (group 0, a thug that walks in after 480 frames), shotgun (3), rifle (5, a poacher), shotgun (6), shotgun (13,
  a Blade or Razor), M-16 (16), Uzi (19), each 30 of 32. Extra guns appear for 2 and 3 players in groups 0, 3, 10 and 15.
- Foods in the Hogg list: group 8 (Hammer, donut 28 and burger 3 of 32), group 17 (Hammer, a small food in 16 of 32). R only.
- Vice's list (Episode 1) gives a steak in 1 of 32 (group 0), small foods in 5 of 32 (groups 4 to 6) and, at level 22 or more, a sure small food from the Hammer of group 1. Butcher's list (Episode 2) gives a mixed food row (group 9), a donut row (group 33), for 2P+ a hot dog row with a 4 of 32 barbecue (group 34) and for 3P the mixed row again (group 35).
- Slice's list (Episode 4) gives the special several times and small foods; none gives a weapon directly.

### 7.4 Player count, level and rank (L12 and the rank effects)

**Player count.** A record with the 2P+ or 3P flag is made only when `G.$4EF` (players minus one) is high enough. 42 item or drop records of the area and wave scripts are gated this way (table 7.4a). They are mostly extra
containers, extra enemies that carry a row, and extra items on the floor. A drop roll itself never reads the player count. M: e1a0_c1p, _c2p, _c3p (the number of containers near X 0x5A0 to 0x600 was 2, 3
and 4: the pail with a rod appears from 2 players, the pail with a club only with 3); e4a2_w1p, _w2p, _w3p (area 2 of Episode 4: a rifle; plus a shotgun at X 848; plus an M-16 at X 928).

- L12 (web: "extra guns for two or three players up to Stage 4; after that supplies and healing match one player") is **partly right**. Extra weapons exist in Episode 1 (rod, club) and Episode 4
  (shotgun, M-16) and the poachers of Episodes 2 and 4 carry a rifle row only for 2P+ or 3P. But Episode 5 also adds a gold dust pail (2P+), a dynamite and grenade drum (3P), a fries drum (3P) and an
  extra torch; Episode 6 adds small foods (Hammer rows, 2P+) and Episode 7 burger, pizza and hot dog barrels (3P) and a small food row (2P+); only Episode 8 has nothing extra. Episode 3's area script has no gated item (the
  car section sends the same enemies to everyone, D2c 16.2), but Hogg's helper list adds guns for 2 and 3 players (section 7.3).
- The wave thresholds and enemy counts by player count are D2c 16.2.

Table 7.4a: records that exist only for 2 or 3 players (the helper lists of 7.3 hold more, flagged in their own table; the Episode 5 area 0 drum for 3 players at X 0x2D0 is freed at once, section 8.1).

| Episode | Where | Trigger | X | Object | Gate | Row | Holds (of 32) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| E1 | area 0 | 0x420 | 0x630 | thug A2 var 0x202 | 3P | E:0x04 | nothing 24, pearl 1, necklace 1, ring 1, watch 1, dime 1, 1000$ 1, sunglass 1, glove 1 |
| E1 | area 0 | 0x450 | 0x5d4 | pail | 2P+ | C:0x12 | rod 32 |
| E1 | area 0 | 0x460 | 0x5f6 | pail | 3P | C:0x04 | club 32 |
| E1 | area 2 | 0x0 | 0x190 | drum | 3P | C:0x24 | coffee 32 |
| E1 | area 2 | 0x0 | 0x1d0 | tires | 2P+ | C:0x0f | burger 16, pizza 16 |
| E1 | wave 1 | wave | 0x190 | Punk/Thug A20 var 0x0 | 2P+ | E:0x15 | nothing 16, dynamite 16 |
| E2 | area 0 | 0x380 | 0x520 | poacher A13 var 0x200 | 2P+ | E:0x17 | nothing 27, rifle 5 |
| E2 | wave 2 | wave | 0x668 | poacher A13 var 0x212 | 2P+ | E:0x02 | SPECIAL 32 |
| E2 | wave 5 | wave | 0xd20 | poacher A13 var 0x200 | 3P | E:0x0e | hot dog 16, pizza 16 |
| E4 | area 0 | 0x210 | 0x3d0 | drum | 2P+ | C:0x13 | fries 32 |
| E4 | area 1 | 0x0 | 0x120 | drum | 3P | C:0x34 | cake 28, gum 4 |
| E4 | area 1 | 0x400 | 0x5b0 | poacher A13 var 0x402 | 3P | E:0x10 | grenade 24, nothing 8 |
| E4 | area 2 | 0x0 | 0x12a | treasure on the floor: 1000$ | 2P+ | - | (placed item) |
| E4 | area 2 | 0x8 | 0x1c0 | A10 (Hammer/Elmer/Wrench) var 0x204 | 3P | E:0x1e | nothing 16, coffee 4, gum 3, chocolate 3, donut 3, croissant 3 |
| E4 | area 2 | 0x1b0 | 0x350 | weapon on the floor: shotgun | 2P+ | - | (placed item) |
| E4 | area 2 | 0x200 | 0x3a0 | weapon on the floor: M-16 | 3P | - | (placed item) |
| E4 | wave 1 | wave | 0x180 | Punk/Thug A20 var 0x4 | 2P+ | E:0x13 | dynamite 32 |
| E4 | wave 1 | wave | 0x180 | Vice/Walther A18 var 0x200 | 3P | E:0x20 | gun 32 |
| E4 | wave 5 | wave | 0x2f0 | A10 (Hammer/Elmer/Wrench) var 0x0 | 3P | E:0x01 | burger 8, hot dog 8, pizza 8, salad 8 |
| E4 | wave 5 | wave | 0x400 | poacher A13 var 0x206 | 2P+ | E:0x17 | nothing 27, rifle 5 |
| E4 | wave 5 | wave | 0x400 | poacher A13 var 0x206 | 3P | E:0x17 | nothing 27, rifle 5 |
| E5 | area 0 | 0x80 | 0x2b0 | pail | 2P+ | C:0x15 | gold dust 28, dime 4 |
| E5 | area 0 | 0x80 | 0x2d0 | drum | 3P | C:0x05 | dynamite 16, grenade 16 |
| E5 | area 0 | 0x880 | 0xa80 | drum | 3P | C:0x13 | fries 32 |
| E5 | area 1 | 0xe00 | 0xf70 | weapon on the floor: torch | 2P+ | - | (placed item) |
| E5 | wave 1 | wave | 0x570 | Blade/Razor A14 var 0x20a | 3P | E:0x14 | nothing 31, diamond 1 |
| E5 | wave 1 | wave | 0x3b0 | Blade/Razor A14 var 0x20a | 2P+ | E:0x14 | nothing 31, diamond 1 |
| E5 | wave 3 | wave | 0xae0 | A10 (Hammer/Elmer/Wrench) var 0x0 | 3P | E:0x0c | fries 7, gum 7, chocolate 7, donut 7, nothing 4 |
| E5 | wave 4 | wave | 0xc40 | Blade/Razor A14 var 0xa | 3P | E:0x14 | nothing 31, diamond 1 |
| E5 | wave 4 | wave | 0xc40 | Vice/Walther A18 var 0x200 | 2P+ | E:0x20 | gun 32 |
| E5 | wave 7 | wave | 0x1260 | Punk/Thug A20 var 0x206 | 2P+ | E:0x15 | nothing 16, dynamite 16 |
| E5 | wave 7 | wave | 0x1400 | A10 (Hammer/Elmer/Wrench) var 0x204 | 2P+ | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| E5 | wave 7 | wave | 0x1260 | A10 (Hammer/Elmer/Wrench) var 0x204 | 3P | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| E6 | wave 1 | wave | 0x6a0 | A10 (Hammer/Elmer/Wrench) var 0x0 | 2P+ | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| E6 | wave 2 | wave | 0x8d0 | A10 (Hammer/Elmer/Wrench) var 0x4 | 2P+ | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| E6 | wave 2 | wave | 0x7d0 | poacher A13 var 0x404 | 2P+ | E:0x09 | nothing 28, SPECIAL 3, rifle 1 |
| E7 | area 0 | 0x420 | 0x5c0 | barrel | 3P | C:0x0f | burger 16, pizza 16 |
| E7 | area 0 | 0x840 | 0xa00 | barrel | 3P | C:0x10 | hot dog 30, barbecue 1, dime 1 |
| E7 | wave 2 | wave | 0x930 | poacher A13 var 0xc | 3P | E:0x10 | grenade 24, nothing 8 |
| E7 | wave 2 | wave | 0x770 | poacher A13 var 0x40c | 3P | E:0x10 | grenade 24, nothing 8 |
| E7 | wave 6 | wave | 0xc0 | A10 (Hammer/Elmer/Wrench) var 0x202 | 2P+ | E:0x1e | nothing 16, coffee 4, gum 3, chocolate 3, donut 3, croissant 3 |


**Level and rank.** The level (D2c 16.4: `level = T[rank][count]`, 0 to 31; at rank 0, 5, 10, 12 it is 0 / 4 / 8, 3 / 5 / 8, 8 / 10 / 10, 10 / 12 / 14 for 1 / 2 / 3 players) gates 23 item or drop records (table 7.4b). Rank itself never changes a roll. Effects, all by swapping the content of the same
spot:
- Episode 7 area 0 barrel at X 0xA40: an Uzi in 31 of 32 at level 16 or less; at level 17 or more only 8 of 32 (and a dime in 24). M: e7a0_lv10 (row 6) and e7a0_lv20 (row 0x2C).
- Episode 4 wave 5 Hammer: lobster (64 HP) at level 16 or less, parfait (32 HP) at level 17 or more. Episode 5 area 1 last pail: cake at 16 or less, coffee (16 HP) above. Episode 7 area 3 barrel: sushi at
  level 17 or less, hot dog at 18 or more (the GameFAQs "hot dog barrel at the far right of 7-4" is the high level case).
- Episode 4 area 2: the extra M-16 at X 0x3F0 exists only at level 8 or less (M: e4a2b_m16_l0 and l5 yes, l12 no), a dime at X 0xE0 only at level 25 or less, a third Hammer (with a small food row) at
  level 26 or more.
- Enemy records that carry a row exist only from some level on: Hammers with a small food row from level 8 and 10 (Episode 5 wave 7), punks with dynamite from level 16 (Episode 4) and 22 (Episode 7),
  poachers with a grenade or diamond row from level 20 and 24, Blades with a diamond row from level 27 and 30 (Episode 4 wave 5).
- So **a good player (low rank, low level) finds more and better help**: a lobster instead of a parfait, an Uzi instead of a dime, an extra M-16. A player on a high rank loses that help but meets the rare
  diamonds. The rank clock and the tables behind the level: D4b.

Table 7.4b: records that exist only below or above a level.

| Episode | Where | Trigger | X | Object | Gate | Row | Holds (of 32) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| E4 | area 2 | 0x0 | 0xe0 | treasure on the floor: dime | lv<=25 | - | (placed item) |
| E4 | area 2 | 0x8 | 0x0 | A10 (Hammer/Elmer/Wrench) var 0x404 | lv>=26 | E:0x1e | nothing 16, coffee 4, gum 3, chocolate 3, donut 3, croissant 3 |
| E4 | area 2 | 0x250 | 0x3f0 | weapon on the floor: M-16 | lv<=8 | - | (placed item) |
| E4 | wave 3 | wave | 0xa20 | Punk/Thug A20 var 0x4 | lv>=16 | E:0x13 | dynamite 32 |
| E4 | wave 5 | wave | 0x2f0 | thug A2 var 0x20a | lv<=16 | E:0x02 | SPECIAL 32 |
| E4 | wave 5 | wave | 0x440 | A10 (Hammer/Elmer/Wrench) var 0x0 | lv<=16 | E:0x0b | lobster 32 |
| E4 | wave 5 | wave | 0x440 | A10 (Hammer/Elmer/Wrench) var 0x200 | lv>=17 | E:0x18 | parfait 32 |
| E4 | wave 5 | wave | 0x400 | Blade/Razor A14 var 0x206 | lv>=27 | E:0x14 | nothing 31, diamond 1 |
| E4 | wave 5 | wave | 0x400 | Blade/Razor A14 var 0x206 | lv>=30 | E:0x14 | nothing 31, diamond 1 |
| E5 | area 1 | 0xe00 | 0xfb8 | pail | lv<=16 | C:0x20 | cake 32 |
| E5 | area 1 | 0xe00 | 0xfb8 | pail | lv>=17 | C:0x24 | coffee 32 |
| E5 | wave 7 | wave | 0x1400 | A10 (Hammer/Elmer/Wrench) var 0x406 | lv>=8 | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| E5 | wave 7 | wave | 0x1260 | A10 (Hammer/Elmer/Wrench) var 0x406 | lv>=10 | E:0x05 | nothing 27, gum 1, chocolate 1, donut 1, coffee 1, croissant 1 |
| E5 | wave 7 | wave | 0x1400 | poacher A13 var 0x408 | lv>=20 | E:0x10 | grenade 24, nothing 8 |
| E7 | area 0 | 0x388 | 0x4e0 | Punk/Thug A20 var 0xa | lv>=22 | E:0x1c | nothing 24, dynamite 8 |
| E7 | area 0 | 0x388 | 0x3d0 | Punk/Thug A20 var 0x20a | lv>=22 | E:0x1c | nothing 24, dynamite 8 |
| E7 | area 0 | 0x880 | 0xa40 | barrel | lv<=16 | C:0x06 | Uzi 31, M-16 1 |
| E7 | area 0 | 0x880 | 0xa40 | barrel | lv>=17 | C:0x2c | dime 24, Uzi 8 |
| E7 | area 3 | 0x0 | 0x1c0 | barrel | lv<=17 | C:0x32 | sushi 30, hot dog 2 |
| E7 | area 3 | 0x0 | 0x1c0 | barrel | lv>=18 | C:0x33 | hot dog 30, sushi 2 |
| E7 | wave 7 | wave | 0x110 | poacher A13 var 0x206 | lv>=20 | E:0x14 | nothing 31, diamond 1 |
| E7 | wave 7 | wave | 0x140 | poacher A13 var 0x406 | lv>=24 | E:0x14 | nothing 31, diamond 1 |


### 7.5 Notes on single enemy rows

- **Walther** (class 18 var 0x200) carries E:0x20 (a revolver, 32 of 32) in Episode 4 wave 1 (two, the second for 3P) and Episode 5 wave 4 (two, the second for 2P+), and E:0x28 (the radio) in Episode 3. That
  confirms M04 ("a revolver 100 percent through Episode 5 except Episode 3") for these two episodes. R: stage_scripts.
- **Gneiss in Episode 1 area 1** (the thug at X 0x1A0): E:0x27 shotgun 24, Uzi 7, bazooka 1 of 32 (W: the FAQ says "usually a shotgun, sometimes an Uzi", right).
- **Punk, Thug (class 20)** carry dynamite rows (E:0x13 all 32 in Episode 4, E:0x15 half in Episodes 1, 5, 7, E:0x1C a quarter in Episode 7). **Bomb carriers** (class 64) also give a real weapon
  object when knocked down (D3a HU26) in addition to their rows.
- **Bikers** drop through the enemy table at the hit: grenade 24 of 32 (Episode 4), small food (E:0x05, E:0x0C), a gun 24 of 32 (Episode 7 wave 0), a steak 1 of 32.
- **Hunters** (A79, Episode 2 wave 2): a mixed treasure row (E:0x22) and a donut row (E:0x23); the alternate Blades of Episode 2 and Episode 8 carry E:0x22 as well.

### 7.6 Episode 3 and the radio (L02)

- The script of Episode 3 area 0 holds **12 radio sources** (row 0x31 of the container table or row 0x28 of the enemy table, each 32 of 32): barrels X 0x250 (the first barrel of the road, trigger 0x80),
  0x580 (a pail), 0x2080 (a drum) and the rolling barrel at X 0x2990; enemies Hammer X 0x320 (trigger 0x160), Punk X 0x980, Walther X 0xD40, alternate Punk X 0xEC0, Hammer X 0x1480, Punk X 0x1BC0, the
  biker at wave 2 X 0x8D0 and a Hammer at wave 4 X 0xFC0. 17 more containers (5 pails, 6 tire stacks, 6 drums) hold nothing. R: stage_scripts; D3c BS17.
- **No recovery item and no weapon in the area script.** Healing and weapons come only from the Hogg helper list (section 7.3): seven guns in a fixed order and two chances of a small food (16 HP). So
  L02 ("no recovery items at all, guns in a fixed order, two ammo") is right for the road and for the guns, and slightly wrong for Hogg's fight. The "ammo twice" of L02 matches the special rows g10, g12,
  g18 and g20 (an ammo box if a gun is about, else a treasure).
- The GameFAQs text "the radio sits in the first barrel after two enemies at the bottom" matches the first barrel (X 0x250, depth 0x20, trigger 0x80) and the 4 thugs of the area start.
- The radio is picked like a treasure: HUD "RADIO 3000 pts", +3,000, and the car event if `G.$50E == 0` (D3c section 5). Later radios give the 3,000 only. M: t12_RADIO (Episode 1: 3,000 points, no event).

## 8. Special cases

### 8.1 Hidden items, items behind scenery, items that appear when you scroll (M16, W04)

- **What "hidden" means in this game.** No item has a hidden flag. Placed items are ordinary pool E objects made when the camera reaches the record's trigger; they are "hidden" only because (a) they sit
  off screen until the camera comes, (b) they pop into the picture inside the screen, or (c) a foreground layer (drawn over the sprites) covers them. R: SPEC 8.1; M below.
- **Episode 2 area 0: the three treasures behind the big yellow trunks.** Diamond (10,000) at X 0x1A8, 1000$ at X 0x30A, dime at X 0x468, all at depth 16 (the lowest walkable row, D4c 3.2) and
  height 64 (the floor there is 64 high, the hero also stands at y 64 at depth 16). A hero at that row is half hidden by the trunk, and the diamond is behind the trunk of the first screen. A hero
  standing 14 px left of the diamond (X 410, Z 16) picked it up with one press: +10,000, message "DIAMOND 10000 pts". M: e2_start (diamond made at start, 1000$ at t = 74 when the camera passed 0x168),
  e2_diamond (press t = 20, score +10,000 at t = 31, picture at 30 to 80). W: the Blueprint L07 ("a scoring item hides under three foreground trees at the start") and the GameFAQs "behind the first tree
  a diamond (10,000) and other score items" are right: three items, the first is the diamond. R: stage_scripts area 0.
- **The fourth treasure of Episode 2 is a script bug.** The record "pearl, X 0x1A8" has the trigger 0x428. The camera is at X 0x428 when it fires, so the pearl is made 640 px behind the camera, found
  outside the far window (section 9.1) and freed. M: e2_pearl (made t = 87 at X 424, state 6 at t = 89: alive 2 frames). It was probably meant for X 0x4A8; keep the quirk (the data does it by itself).
- **Items that pop into the picture.** The tables of 7.2 give "appears N px from the left edge". Examples (R): Episode 5 area 0 diamond X 0x204 at trigger 0x180 appears 132 px from the left edge (M:
  e5a0_diamond: made at t = 141 when the camera was 381 to 389, so at screen X 127 to 135); Episode 6 area 0 gold bar X 0x3C0 at trigger 0x380 appears 64 px from the left edge (not seen in MAME: the camera
  locks at 0x300); Episode 6 area 1 gold bar at the start, 144 px; Episode 7 area 2 gold bar 177 px; Episode 4 area 2 dime 224 px and 1000$ (2P+) 298 px. The fan claims of M16 ("a diamond in the rock where the
  triceratops appears", "a gold bar when three grunts appear after entering the mine") match the Episode 5 diamond and the Episode 6 gold bar; the triceratops (Mack) and the three grunts belong to the same
  trigger groups (D3a, D3b).
- **Items that stand off screen right** (most containers) simply scroll into view. Of the 107 container records and 30 placed items, only **two** fall outside the far window of section 9.1 at the moment they
  fire: the pearl (640 px behind the camera) and **the Episode 5 area 0 drum for 3 players** (X 0x2D0 at trigger 0x80: the camera is 0x80 to 0x85 when it fires, so the drum is 0x24B to 0x250 px to the right,
  past the limit 0x240; it is freed in its first frame). The drum case is derived from the measured window and the script data; it was **not seen in MAME** (open item 3). Its 2P+ neighbour
  (the gold dust pail, X 0x2B0, 0x230 px away) is inside the window. R: stage_scripts, `item_despawn_far`; M: despawn_x.

Items lying on the floor, placed by the scripts:

| Episode | Area | Item | X | Z | Trigger | Appears at | Gate |
| --- | --- | --- | --- | --- | --- | --- | --- |
| E2 | area 0 | treasure diamond | 0x1a8 | 0x10 | 0x0 | off screen to the right | - |
| E2 | area 0 | treasure 1000$ | 0x30a | 0x10 | 0x168 | off screen to the right | - |
| E2 | area 0 | treasure dime | 0x468 | 0x10 | 0x2c8 | off screen to the right | - |
| E2 | area 0 | treasure pearl | 0x1a8 | 0x10 | 0x428 | left of the screen (freed at once) | - |
| E4 | area 0 | stone (size 2) | 0x14b | 0x52 | 0x0 | 331 px from the left edge | - |
| E4 | area 1 | treasure ring | 0x3df | 0x10 | 0x200 | off screen to the right | - |
| E4 | area 2 | treasure dime | 0xe0 | 0x38 | 0x0 | 224 px from the left edge | lv<=25 |
| E4 | area 2 | treasure 1000$ | 0x12a | 0x38 | 0x0 | 298 px from the left edge | 2P+ |
| E4 | area 2 | shotgun | 0x350 | 0x50 | 0x1b0 | off screen to the right | 2P+ |
| E4 | area 2 | rifle | 0x350 | 0x18 | 0x1b0 | off screen to the right | - |
| E4 | area 2 | M-16 | 0x3a0 | 0x50 | 0x200 | off screen to the right | 3P |
| E4 | area 2 | M-16 | 0x3f0 | 0x50 | 0x250 | off screen to the right | lv<=8 |
| E5 | area 0 | treasure gold dust | 0x10 | 0x210 | 0x0 | 16 px from the left edge | - |
| E5 | area 0 | food barbecue | 0x116 | 0x240 | 0x0 | 278 px from the left edge | - |
| E5 | area 0 | food barbecue | 0x102 | 0x238 | 0x0 | 258 px from the left edge | - |
| E5 | area 0 | treasure egg | 0xfa | 0x248 | 0x0 | 250 px from the left edge | - |
| E5 | area 0 | torch | 0xdd | 0x23e | 0x0 | 221 px from the left edge | - |
| E5 | area 0 | torch | 0xf4 | 0x24c | 0x0 | 244 px from the left edge | - |
| E5 | area 0 | torch | 0x108 | 0x236 | 0x0 | 264 px from the left edge | - |
| E5 | area 0 | treasure diamond | 0x204 | 0x210 | 0x180 | 132 px from the left edge | - |
| E5 | area 1 | torch | 0xf70 | 0x160 | 0xe00 | 368 px from the left edge | 2P+ |
| E5 | area 1 | torch | 0xfa0 | 0x110 | 0xe00 | off screen to the right | - |
| E6 | area 0 | treasure gold | 0x3c0 | 0x10 | 0x380 | 64 px from the left edge | - |
| E6 | area 1 | treasure gold | 0x90 | 0xf0 | 0x0 | 144 px from the left edge | - |
| E7 | area 0 | treasure dime | 0x120 | 0x1e | 0x0 | 288 px from the left edge | - |
| E7 | area 0 | treasure ring | 0x6f5 | 0x4f | 0x560 | off screen to the right | - |
| E7 | area 0 | stone (size 1) | 0x720 | 0x20 | 0x590 | off screen to the right | - |
| E7 | area 2 | treasure gold | 0x2b1 | 0x10 | 0x200 | 177 px from the left edge | - |
| E7 | area 2 | treasure 1000$ | 0x45b | 0x10 | 0x2a0 | off screen to the right | - |
| E7 | area 2 | treasure watch | 0x5c1 | 0x10 | 0x420 | off screen to the right | - |


(For trigger 0 the camera of the area start is meant, normally X 0; the Episode 2 area 0 camera moves during the opening scene, M e2_start shows camera X 291 at the first control frame.)

### 8.2 The radio

Section 7.6 and D3c section 5. Summary: 12 sources in the script, +3,000, HUD "RADIO 3000 pts", the car event only when `G.$50E == 0` and the stage index is 2. The port has the treasure and the score
(`item/small.rs` `item_treasure_collect`, with a `TODO(phase 7)` for the call to 0x8BD24).

### 8.3 The Episode 8 meat barrel (L11) and the Episode 6 salad barrel (M26)

- **Episode 8 area 2 ("8-3"):** a barrel (variant 4, liftable) at X 0xCB0, depth 0x110 (272), trigger 0xB10, row C:0x0B = barbecue 32 of 32 (100 HP, 10,000 at full HP). It stands near the end of the slope, before
  the Fessenden fight. M: e8_meat (the object was made at t = 190 at X 3248, Z 272, drop row 11, HP 4). The three barrels at the start of the area hold a small food (X 0x130), an Uzi (X 0xF0, the middle one:
  31 of 32 an Uzi, 1 of 32 an M-16) and a treasure (X 0xB0); the GameFAQs "Uzi in the middle barrel" is right.
- **Episode 6 area 2 ("6-2"):** a barrel at X 0xEA0, depth 0x160 (352), trigger 0xD00, row C:0x0C = salad 32 of 32 (32 HP, 5,000 at full HP). M: e6_salad (made at t = 182 at X 3744, Z 352, drop row 12).
- Both are ordinary barrels (D2c section 9: the hero may lift and carry them, which is the "carry it to the boss" of M26 and L11; a hit breaks them at once). Which camera locks and scenes block the carry
  is D4c.

### 8.4 Stones from debris, rocks and boulders (G08)

- **Source.** The debris routine 0x294AA makes up to 9 pool C pieces (class 0x2C) for a stone, a boulder or a falling rock (class 66, which uses the boulder list 0x29650). For each piece it may mark a
  **bonus**: while `d5 < 3`, where `d5` starts as `G.$4F1` and rises by one for every bonus of this break, it rolls `rand() & 0xF` against the mask for `d5`: `d5 = 0` mask 0xC000 (2 of 16),
  `d5 = 1` mask 0xFF00 (8 of 16), `d5 = 2` mask 0xE000 (3 of 16). A bonus piece of a stone container becomes picture 0x1A. R: 0x294AA to 0x29550, table 0x29552.
- **Bonus pieces** run the state 6 of the debris routine: they bounce for `0x1E + (rand & 0x3F)` frames (30 to 93), then (on the next floor contact) make **a stone item**: pool E class 0, `+$26 = 0x1C`
  (type 14), size `+$5C` = 0 for picture 0x20, 1 for 0x22 (or any other picture above 0x1A), 2 for 0x1A and below. The piece is freed. R: 0x935D8, 0x93730 to 0x93770, `0x93770` (the maker).
- **A stone object** counts itself in `G.$4F1` when it starts and gives the slot back when it is freed (R 0x24D44 type 14, 0x264D0). `G.$4F1` is zeroed at every area start. The cap "fewer than 3" is tested
  at the moment of the **break**, and a stone only counts after its piece has bounced, so several quick breaks can pass the cap.
- **Measured.** Eight boulders broken one after the other made stones from the sixth break on: objects at t = 272, 274, 360, 408 (4 stones), `G.$4F1` was 0 until t = 267, 2 at t = 275. Eight stone
  containers (same times because the random state is the same) gave the same four. Sixteen boulders gave 7 stones (`G.$4F1` 1, 4, 5, 7 at t = 286, 326, 366, 406; sizes 1, 0, 1, 1, 0, 0, 1: boulder
  pieces give the small and the medium stone only). Three falling rocks made none (by chance). The ten breaks of D2c (d2c-g) were all made from the same random state at about the same frame; that is why
  it saw none. M: stones_boulder, stones_stone, stones2_boulder, stones_rock.
- **Stones last.** Three stones placed on the floor (sizes 0, 1, 2) were still there after 2,000 frames and `G.$4F1` stayed 3. M: stone_items. A stone is a weapon type 14 (D2c 6.4: 28 damage, 3 sizes,
  vanishes after 4 bounces when thrown).
- G08 ("falling rocks in Episode 5 leave stones the player can use as weapons") is therefore **true but by chance** (about 2 in 16 for the first piece), up to 3 stones on the field at the time of the break.
  The Episode 7 and Episode 4 placed stones are in 8.1.

### 8.5 Other notes

- **Armor statues (W20)** are the Episode 1 hall containers (5.1); they hold salad, gold dust or ammo, a revolver (31 of 32) and gold dust.
- **Episode 5 start (N09, M: e5a0_start):** gold dust at X 16, two barbecues at X 0x102 and 0x116 (depth 0x238 and 0x240), the egg at X 0xFA (depth 0x248) and three torches at X 0xDD, 0xF4 and 0x108
  (depth 0x23E, 0x24C, 0x236) are all there at the first frame. A press takes the lowest kind first, so standing on the egg and a barbecue takes the barbecue first (food kind 0x00tt is lower than
  treasure 0x03tt); that is the "easy to grab the wrong one". The torches burn on the floor (picture: flames beside the first thugs).
- **Episode 4 tires and the pizza (N08):** 5.5 (a jab reaches the back row from depth 80 and 88, not from 72).

## 9. Lifetime, despawn and floor rules

### 9.1 Windows (M: despawn_x)

Every pool E object runs a range test each frame while it lies on the floor (and a thrown or flying one, per class):

| Test | Objects | Lives while the integer X is in | Frees the object (mode 6) when |
| --- | --- | --- | --- |
| Far window `0x173C` | food, treasure and containers (lying, held away or flying), most pool A classes (all enemies, bikers) | `[camera X - 0xC0, camera X + 0x240)` | X < camera X - 0xC0 or X >= camera X + 0x240 |
| Near window `0x1760` | weapons (stones included), ammo boxes, thrown weapons, rolling barrels (class 65) and falling rocks (class 66) | `[camera X - 0x40, camera X + 0x200)` | X < camera X - 0x40 or X >= camera X + 0x200 |
| Vertical scene `0x1782` (Episode 7 area 1 lift) | all of them | X in `[-0x80, 0x200)` and `Z + Y` in `[-0x80, 0x180)` | outside either |

M: 14 foods and 12 knives placed at 0x100 to 0x280 and 0x60 to 0x20C from the camera. Food: alive at -0xC0 (192), -0xBC, -0xB0, -0x80, +0x230, +0x23C; freed at -0xC4, -0xC8, -0xD0, -0x100, +0x240, +0x244,
+0x250, +0x280. Knife: alive at -0x40, -0x3C, -0x30, +0x1E0, +0x1F8, +0x1FC; freed at -0x44, -0x48, -0x60, +0x200. (Two knives could not be made: the pool was full.) R: `item.rs`.
The camera moves at most 6 px per frame, so a hero who scrolls on leaves floor items behind and they are freed 192 px (food, treasure) or 64 px (weapons) outside the left edge of the screen.

### 9.2 The pool is shared

24 slots for food, treasure, ammo boxes, containers, weapons (also those in hands), bullets, pellets, shells, explosions, projectiles and boxes. A drop that finds no free slot is lost (6.4, M pool_22 and
pool_23); script records of pool E that find no slot are dropped as well (R SPEC 8.1: "pool full: dropped"). Every area start frees all items except a weapon in a hero's hand (R 0xAA94: `mode 6`, which also
zeroes `G.$4F1`); items left in the area behind are lost.

### 9.3 Bouncing in from the screen edge

6.5. The test runs at every landing of a dropped item and also for a placed weapon (M: a gun placed at X 904 = camera X - 0x18 was at X 911 at t = 40). An item that lands inside the screen (X from
camera X + 0x11 to camera X + 0x16F) stays where it lands.

### 9.4 Dynamite (M18)

- A dynamite (weapon type 12) that **lands after being dropped** (swap, hit, death, special attack) is lit on landing: stage 3, 4 stages of 15 frames (animation id 0x12 + stage), explosion on the 60th frame.
  M: dyn_swap_wait (swap at t = 60, lands t = 106, explosion object at t = 166). R: `weapon.rs` `weapon_dropped` calls `weapon_arm` for kind 0x201 variant 0x18.
- **While a hero holds it the countdown stops** and the stage is kept. M: dyn_swap_repick (picked up at t = 135 in stage 2, carried until t = 191, put down again, lands t = 236 still in stage 2, explodes t = 281:
  45 frames after the landing, the three remaining stages). So "picking it up again puts it out" of M18 is right only in the sense that the fuse pauses; it is not reset.
- A **thrown** dynamite flies, bounces 4 times, lies armed and explodes 60 frames after it came to rest (M dyn_swap: at rest t = 199, explosion t = 259). A grenade that is dropped does nothing (D2c 6.3).
  Enemy thrown ones are made lit (D2c, D3a).

### 9.5 What never times out, and what does

- **Never:** food, treasure, ammo boxes (`+$A4 = 0`), weapons placed by the script, weapons dropped by an enemy or by a container, stones (M life_long: food, treasure, ammo box, a placed gun and a shotgun
  dropped by a container all alive at t = 3,604; M stone_items).
- **Times out (D2c section 8, section 2.2):** a weapon dropped by a hero lies 3,600 frames; floor knives from the 4th on break after 600 frames; the join bazooka after 600 frames if dropped; a lit dynamite
  explodes after its fuse.
- **Not removed by** kill-all (items are not enemies), boss deaths, wave ends.

### 9.6 Rolling barrels and "go slowly" (L11, N10, N11)

- Classes 65 (barrel) and 66 (rock) call the near window every frame (the biker, class 70, the far window), so a barrel that rolls 64 px past the left edge of the camera is freed together with the item it carried. A hero who runs
  right drags the camera along: the barrels spawned by the camera triggers roll towards or away from the hero and many leave the near window before he reaches them. That is the "items in two rolling
  barrels only if the player goes slowly" of the GameFAQs text (Episode 6 area 1 grenade and pizza) and the "running down the slope removes most rolling barrels" (Episode 8 area 3). It is not the camera lock
  that removes them. R: 0x5B9D0 and 0x5BC90 (`jsr $1760`), 0x5C462 and 0x5C4B2 (`jsr $173C` of the biker), `item_despawn_near`.
- Barrels also leave when they are 0x200 px right of the camera; they never time out.
- The spawn side is by the camera: a barrel spawned left of camera X + 0xC0 faces right, otherwise left (D3a 10.1).

## 10. Check against the GameFAQs notes and the other web leads

The GameFAQs text (supplement 2, "Scene-by-scene walkthrough details", unverified, player written) was used as a checklist. Every item statement agrees with the rows:

| Statement | ROM rows and measurements | Verdict |
| --- | --- | --- |
| 1-2: the Gneiss usually drops a shotgun, sometimes an Uzi | E:0x27: shotgun 24, Uzi 7, bazooka 1 of 32 (Episode 1 area 1, the first thug) | right |
| 1-3: a bucket gives a health item before the boss; a punk drops a grenade | pail X 0x1F0 row C:0x35 burger 30 or cake 2; thug at X 0x30 row E:0x25 grenade 32 | right |
| 2-2: the middle bottom barrel holds a donut or gum; the top right barrel a revolver | Episode 2 area 2: drum X 0x248 depth 0x28 row C:0x03 (small food); drum X 0x280 depth 0x90 row C:0x2B (gun 31, Uzi 1) | right |
| 3: the first barrel holds the radio | X 0x250, trigger 0x80, row C:0x31 | right |
| 4-1: one barrel holds a shotgun | Episode 4 area 0 drum X 0x2D0 row C:0x11 (shotgun 32) | right |
| 4-2: tires hide a pizza (dive kick); barrels hold a grenade and sometimes a rifle or machine gun; midgets drop dynamite; a punk on a bike drops food | tires X 0xB0 depth 0x58 row C:0x28 (pizza 28 of 32; a jab from depth 80 breaks it, so no dive kick is needed); drum C:0x1C grenade; pail C:0x2F rod 26, rifle 5, M-16 1; Punk/Thug A20 row E:0x13 dynamite 32; bikers E:0x0C small foods | right (dive kick not required) |
| 4-3: the Uzi barrel; a punk drops bullets; thrown dynamite kills the fat enemies | area 2 drum X 0x250 row C:0x06 (Uzi 31); thug rows E:0x02 (special = bullets) in wave 5 | right |
| 5 start: meat and the dinosaur egg sit close together | e5a0_start: barbecues X 0x102 and 0x116, egg X 0xFA | right |
| 5 middle: barrels hold a bazooka, a grenade, an Uzi and bullets | C:0x21 bazooka 32; pail C:0x23 grenade 31; drum C:0x2B gun 31 / Uzi 1 (mostly a revolver); special rows = bullets | right, the Uzi is rare |
| 5 end: three buckets, one with bullets | pails X 0xF68 (C:0x07 special), 0xF90 (ammonite), 0xFB8 (cake or coffee) | right |
| 6-1: barrels hold a grenade (leftmost) and a pizza (rightmost) | Episode 6 area 0 barrels X 0x440 (C:0x1B grenade) ... X 0x5C0 (C:0x0F burger or pizza) | right |
| 6-2: two rolling barrels carry a grenade and a pizza; a salad barrel before the boss | Episode 6 area 1: rolling barrels C:0x1B and C:0x28 (pizza 28); area 2 barrel C:0x0C salad | right |
| 7-3: pudding in a bottom barrel; small foods in rolling barrels; 7-4 a hot dog barrel at the far right | wave 12 rolling barrel Z 0x10 C:0x1D pudding; library barrels C:0x03; area 3 barrel X 0x1C0 C:0x33 hot dog 30 (level 18 or more) | right (level dependent) |
| 8-2: the second fat enemy drops a pizza | Episode 8 area 1 Hammer var 0x200 row E:0x0E hot dog 16 / pizza 16 | right (a hot dog or a pizza) |
| 8-3: an Uzi in the middle barrel; the meat barrel can be carried | 8.3 | right |
| Blueprint L02: Episode 3 has no recovery items; guns in the fixed order Uzi, shotgun, rifle, shotgun, shotgun, M-16, Uzi with ammo twice | 7.3 and 7.6 | order right; the Hogg list also has two small foods |
| Blueprint L06: a log holds an Uzi and a hunting rifle (Episode 4) | area 2: drum X 0x250 Uzi 31 and a placed rifle X 0x350 | consistent |
| Blueprint L07: a scoring item hides under three foreground trees (Episode 2) | 8.1 | right (3 items, a 4th is lost) |
| Blueprint L08: water buckets hold items (Episode 5) | the village pails (special, ammonite, cake or coffee) and the road pails (gold dust 2P+, grenade) | right |
| Blueprint L09: no firearms in the whole of Episode 6 | no gun placed, no gun container, but five poachers carry E:0x09 (rifle 1 of 32) | almost right |
| Blueprint L11: a meat barrel stands before the last boss; barrels roll down the slope and running removes some | 8.3, 9.6 | right |
| Blueprint L12: extra guns for 2 and 3 players up to Stage 4 | 7.4 | partly right |
| Supplement M04: Walther drops a revolver 100 percent through Episode 5 except Episode 3 | 7.5 | right |
| Supplement M16, M17 | 2.3, 3.2 | partly right |
| Supplement M18: dynamite lit when set down; an unused gun vanishes; ammo gives 1,000 | 9.4; D2c 8 and 10 | right |
| Supplement M26: a broken drum can drop a rifle instead of a stick (Episode 4); salad barrel before Tyrog | the pail C:0x2F gives rod 26, rifle 5 (the rod is the "stick"); 8.3 | right |
| Supplement G08 / lead: falling rocks leave stones | 8.4 | true, by chance |
| Supplement G18: food and treasure names | 2.1, 3.1 | right (17 foods, 16 treasures) |
| User U01: some enemies pick up weapons or items | D3a HU25: only Bludge eats floor items | no thief; the carriers drop their stick |

## 11. (a) Acceptance scenarios

Every scenario is one or more segments of the probe batches `re/out/d4a/batch1.txt` to `batch4.txt` (run `PR_INPUT=re/out/d4a/batchN.txt re/harness/probe/probe.sh NAME`, one MAME process each; they
start from Episode 1 area 0 at camera X 928 with Jack and `god on` unless the name says another place). The port runs the same input and the same pokes; the numbers below must be equal frame for frame
(`t` after the jump). Scenario names A01 to A27.

| ID | Segments | What must match |
| --- | --- | --- |
| A01 | f01 to f17 | Hero HP 40, food made at t = 2, press t = 10: HP at t = 11 is 100, 100, 100, 100, 88, 88, 88, 72, 72, 72, 72, 72, 56, 56, 56, 56, 56 for foods 1 to 17; lift state 0x14 from t = 11 to 20; food object freed after the lift |
| A02 | f01 to f17 (second eat) | HP set to 100 at t = 68, second food made at t = 70, press t = 78: score +10,000 (foods 1 to 4), +5,000 (5 to 8), +3,000 (9 to 12), +1,000 (13 to 17) at t = 79, HP stays 100 |
| A03 | f18_food18 | HP 40 to 76; at full HP score +4,000 although the message says 10000; the object has no picture |
| A04 | t01 to t17 | Press t = 10, lift state t = 11 to 20, score at t = 21: 10,000 (1 to 4), 5,000 (5 to 8), 3,000 (9 to 12), 1,000 (13 to 16), 10 (17) |
| A05 | cj_0 to cj_C, ch_4_barrel_mustapha, _hannah, _mess | First jab at t = 12 (Mess t = 13): HP 4 to 0, score +100; state 4 at t = 13; score 600 and the steak object (row 0x09 of the container table) made at t = 14 (Mess 15) |
| A06 | cj_0 to cj_C | Effect objects made at t = 14: pail 0, drum 2 (class 0x2C), barrel 10 (class 0x54), stone 9 and boulder 9 (class 0x2C), statue 8, tires 5; the drum pieces gone by t = 73, stone pieces by t = 104, puffs by t = 75 |
| A07 | tnt_2, tnt_5hits, tnt_idle | tnt_2: HP 50 to 40 at t = 12, fuse stage 3 from t = 13, explosion object and the steak at t = 124, no +500; tnt_5hits: HP 40, 30, 15, 0 at t = 12, 23, 36, 50, score 100, 200, 400, 800, explosion and steak at t = 51; tnt_idle: shimmer every 51 frames |
| A08 | expl_hits_pail, expl_hits_barrel | An explosion object made at t = 20: container HP 0 at t = 24, state 4 at t = 25 |
| A09 | gun_hits_barrel, knife_hits_barrel | Pistol bullet 8 px per frame hits at t = 49, state 4 at t = 50, score +500 at t = 51 (no hit score); thrown knife hits at t = 54, +500 at t = 56, knife freed at t = 55 |
| A10 | lift_barrel_dx-24, _dx+24, _dx+0, lift_tires_dx*, lift_statue_dx* | Barrel lifted at +-24 (hero state 0x14 from t = 9, held t = 28), jabbed and broken at 0; tires and statue never lifted (state 0x04 attack), at +24 and more the jab misses |
| A11 | stones_boulder, stones_stone, stones2_boulder, stone_items | Stone objects first at t = 272 and 274 (then 360, 408), `G.$4F1` (word at 0xFF84F0, low byte) 0 until t = 267 and 2 at t = 275; stones2: 1, 4, 5, 7 at t = 286, 326, 366, 406; sizes 0 or 1; stones alive at the end |
| A12 | sp_nogun, sp_knife, sp_club, sp_gun, sp_uzi, sp_bazooka, sp_held | Row 0x07 container broken at t = 40: dime (or another of the fallback six) without a gun item; ammo box with a gun, Uzi, bazooka or a held gun |
| A13 | hop_mid, hop_left, hop_right, hop_right2 | Dropped steak made at t = 22 (container broke at t = 20 + 2), in the air from t = 23, y 62 at t = 28, apex 69, lands t = 70; hop_left: second hop to X 974, vx 384; hop_right: to X 1277, vx -256; hop_right2: no bounce |
| A14 | pool_22, pool_23 | Free count 24 before; 22 foods + container: a steak exists after the break; 23 foods + container: none |
| A15 | despawn_x | Food alive at camera X - 0xC0 to + 0x23C and freed outside; knife alive at - 0x40 to + 0x1FC and freed outside |
| A16 | life_long, stone_items | Food, treasure, ammo box, placed gun and a container-dropped shotgun alive at t = 3,604; stones alive at t = 2,000 |
| A17 | dyn_swap_wait, dyn_swap_repick, dyn_swap | Swap at t = 60, lands t = 106, explosion object t = 166; repick: held t = 135 to 191 (stage 2 kept), lands t = 236, explosion t = 281; thrown dynamite at rest t = 199, explosion t = 259 |
| A18 | e1a0_c1p, _c2p, _c3p, e4a2_w1p_l0, _w2p_l0, _w3p_l0 | Containers near X 0x5A0 to 0x600: 2, 3, 4 (rod for 2P+, club for 3P); Episode 4 area 2 floor weapons: rifle; + shotgun at X 848; + M-16 at X 928 |
| A19 | e7a0_lv10, e7a0_lv20, e4a2b_m16_l0, _l5, _l12 | Barrel at X 2688: row 6 at level 10, row 0x2C at level 20; the M-16 at X 1008 exists at levels 0 and 5, not at 12 |
| A20 | barrel65_drop, biker70_drop, edrop_kill, edrop_survive | Barrel (class 65) row 0x0B gives food 1 (barbecue) at the hit + 1; biker row 0x0B gives food 3 (lobster) at the hit + 1; thug with 1 HP: HP 0 at t = 14, lobster object at t = 15, score +100 at t = 14 and +1,000 at t = 108; a surviving thug drops nothing |
| A21 | e2_pearl, e2_start, e2_diamond | Pearl made at t = 87 at X 424, y 64, state 6 at t = 89; diamond at X 424 y 64 from the start, 1000$ at X 778 from t = 74; hero at X 410 Z 16 y 64 picks the diamond: +10,000 at t = 31 |
| A22 | f01, f09, t01, hud_clean (pictures) | Message absent at t = 16 and present from t = 22 after a food press at t = 10; FooD icon, name, "GOOD !!"; second message " 10000 pts" from about t = 90; treasure: "Pts" icon, name, " 10000 pts" from about 12 frames after the score; both about 60 frames long; hud_clean pictures 700 to 925 |
| A23 | e8_meat, e6_salad | Barrel var 4 at X 3248 Z 272 row 11 (Episode 8) made at t = 190; barrel var 4 at X 3744 Z 352 row 12 (Episode 6) made at t = 182 |
| A24 | e5a0_start, e5a0_diamond | At the first frame: items at X 16, 221, 244, 250, 258, 264, 278 (var 5, 0x16 x3, 4, 1, 1); the diamond (var 2, X 516) made at t = 141 when the camera is 381 to 389 |
| A25 | e4a1_tires_jab_z58, _z50, _z48 | Both tire stacks (X 176 and 240, Z 88) HP 0 at t = 14 for hero Z 88 and 80; no hit at Z 72 |
| A26 | (not measured: new segment needed) `jump 5 0 0 char=jack,hannah,mess`, watch E:5 | The 3 player drum (X 0x2D0 trigger 0x80) lives 1 or 2 frames and is freed; the 2P+ pail (X 0x2B0) stays |
| A27 | (not measured) enemy attack frame against a solid and a non solid container | Container HP 0 with who 0xFF and no score |

## 12. (b) Port status and the port task list

Files are under `dino-rs/crates/dino-game/src/game/` unless said.

| Part | State | File | Notes |
| --- | --- | --- | --- |
| Food (class 4): heal, bonus, hop, lift, eat, sound call | DONE | `item/small.rs` | reads the 0x48 byte table, so id 18 behaves as in MAME |
| Treasure (class 9) and the score | DONE | `item/small.rs` | id 17 would read past the extracted score table (panic); it can only come from a poke |
| Radio call (0x8BD24) | MISSING | `item/small.rs` (`TODO(phase 7)`) | the event is D3c (car) |
| Ammo box (class 2) | DONE | `item/small.rs` | D2c |
| Containers (class 5): all 7 variants, HP, fuse, lift, held, thrown, break, explosion | DONE | `item/container.rs` | checked in phase 1 task 2 |
| Hit passes 0x11F86, 0x1201C, 0x120D2, 0x11962 | DONE | `item/hits.rs`, `enemy_combat.rs` | the enemy attack frame case (0x1201C) is not seen in MAME (A27) |
| Drop roll 0x24B36, 0x24B48, the special, the hop, the landing test | DONE | `item.rs`, `item/shared.rs` | needs classes 65, 66, 70 to call the right table (D3a) |
| **Drop table range** | **BUG** | `dino-extract/src/enemy.rs` (`TABLES`, row 0xB64D2, length 0xE40) | the range ends at 0xB7312, so enemy rows 0x00 to 0x2F are there and **row 0x30 is not**. Episode 1 wave 4 gives a Hammer row 0x30 (steak); killing it makes `item_drop_roll` read 0xB7312 and the table lookup **panics** ("ROM table byte not extracted"). Extend the length to at least 0xE80 (row 0x3A) |
| Spawn flags (players 0x40 and 0x60, level 0x08 and 0x10) | DONE | `spawn.rs` | A18, A19 are the proof to run |
| Despawn windows 0x173C, 0x1760, 0x1782 and the bounce in | DONE | `item.rs`, `item/shared.rs` | formulas equal the measured windows |
| Dynamite lighting, fuse pause | DONE | `item/weapon.rs` | D2c |
| HUD messages `hud_food_msg`, `hud_treasure_msg` (ring, icons, strings) | PARTIAL | `hud.rs` | not compared with the MAME pictures; the treasure 17 message differs (open item 4) |
| Effects: debris class 0x2C, puff class 0x54, fuse spark class 0x64 | MISSING | `effect.rs` (the match has no arm) | needed for A06, A11 and the stones from debris |
| Torch glow overlay (item class 6) | MISSING | `item/blast.rs` stub | D2c P-D2c-4 |
| Rolling barrel, rock, biker classes (drop calls) | MISSING | `enemy/` | D3a |
| Boss helper controller with drop rows (pool B 22) | MISSING | `spawn.rs` / `prop.rs` | D3c |
| Class 67 Uzi item, class 26 and 68 items | MISSING | `enemy/` | D3a, D3d |
| Foreground layer over sprites (hides the Episode 2 treasures) | UNKNOWN (not checked) | `draw.rs` | check with the A21 pictures |

Port tasks (sizes: S under 1 hour, M 1 to 2 hours, L over 2 hours):

| Task | Scope | Size |
| --- | --- | --- |
| IT-P1 | **Fix the drop table range** (enemy rows up to 0x3A, treasure score table index 16) and add a unit test that rolls row 0x30 | S (15 minutes) |
| IT-P2 | Debris, puff and fuse spark effect classes with the stone spawn from bonus pieces (R 0x934F6, 0x93770, 0x94640, 0x9625C), scenarios A06 and A11; with D3f | M |
| IT-P3 | Item scenarios A01 to A25 as harness scenarios (inputs from the batch files), then fix what differs | M |
| IT-P4 | HUD item messages against the MAME pictures (A22), including the queue and the 60 frame timer | S |
| IT-P5 | Radio call hook (the event is D3c) | S |
| IT-P6 | Torch glow overlay | S |
| IT-P7 | Drop calls of classes 65, 66 (container table) and 70 (enemy table) with their classes (D3a tasks) | S each |
| IT-P8 | Boss helper lists with rows (D3c) and the Episode 3 order of guns (A of section 7.3) | part of D3c |
| IT-P9 | MAME check of A26 and A27 (two new probe segments) and a decision for the 3 player drum | S |

## 13. (c) Open items

1. **An enemy attack frame destroying a container** (R 0x1201C: HP 0, no credit) was not seen in MAME. Six tries failed: a solid barrel stops the thug, a pinned thug never swings, and thugs next to a stone or
   boulder (non solid) stayed in a waiting state for 900 frames. Try a thug that is already attacking the hero, a poacher's punch, or a dinosaur (A27).
2. The Episode 5 area 1 level gate (cake or coffee, trigger 0xE00) was not reached: the camera stopped at X 3443 before it (a lock). The Episode 6 gold bar pop-in (trigger 0x380) was not seen for the same reason.
3. **The Episode 5 area 0 drum for 3 players** (X 0x2D0, trigger 0x80) is freed at once by the far window according to the measured limit; not seen in MAME (A26).
4. Treasure 17 (no row can make it): MAME gives +10 points and a garbled message; the cause (string table index 16) is not decoded.
5. Stone size 2 (from a stone container's bonus piece, picture 0x1A) was not seen; only boulder pieces were measured (sizes 0 and 1). The bonus chances rest on the weak random generator: numbers of stones per
   break need the port to use the same generator (it does).
6. The HUD message tile positions for the 2 and 3 player blocks, and the exact frame when a queued message replaces the first.
7. Unused rows (11 container rows, 8 enemy rows used by nothing but the helper lists, the "steak 32" rows) and what the table words after row 0x3A are.
8. Class 67 (an Uzi item when its variant is 0), 26 (grenade or dynamite item) and 68 (dynamite item): who makes them and when (D3a, D3d).
9. A static search found no code that writes a constant row to `+$96`; a row could still be copied from a table by a routine that was not recognised (low risk).
10. The Hogg helper foods (groups 8 and 17) are read from the lists; the fight with a helper kill was not run (D3c).
11. Whether a hero with HP above 100 (extra bars from the probe command `hp`) eats for HP or for score: the code tests `hp == 100` exactly, so HP 101 and more heals nothing and gives no bonus (R only).

## 14. (d) New rows for mechanics-seen.md

IDs `IT01` to `IT30`; port column as in section 12.

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| IT01 | Only 17 foods exist; id 18 reads the first 4 bytes of the next table (heal 36, bonus 4,000, no picture, name wraps to BARBECUE, text 10000 pts) and can only come from a poke | R 0x28B10, 0x28B54; M f18_food18 | DONE | corrects the 18 foods of W03 and SPEC 13.4 |
| IT02 | Food values: heal 100, 80, 64, 64, 48, 48, 48, 32 x 5, 16 x 5; full-HP bonus 10,000 (1 to 4), 5,000 (5 to 8), 3,000 (9 to 12), 1,000 (13 to 17); the GameFAQs tiers are right, M17 is wrong for steak, lobster, sushi, salad, donut, gum | R 0x28B10; M f01 to f17 | DONE | section 2 |
| IT03 | 16 treasures 10,000 (gold, diamond, cup, egg), 5,000 (gold dust, dino bone, ammonite, pearl), 3,000 (necklace, ring, watch, radio), 1,000 (dime, 1000$, sunglass, glove); a 17th picture exists with no name and a score of 10 | R 0x2A164; M t01 to t17 | DONE (id 17 not extracted) | M16 lacks cup, dino bone, radio and invents a book |
| IT04 | Food heals at press + 1, treasure scores at press + 11 (end of the 10 frame lift); the HUD message follows 10 to 20 frames later, shows a FooD or Pts icon, the name, GOOD !! or N pts, and lasts about 60 frames; messages queue | M f01, t01, hud_clean | PARTIAL | the message is the HUD's, section 2.4 |
| IT05 | Containers: 7 variants have code (pail, drum, barrel, stone, boulder, statue, tires), 13 numbers; the scripts use 5 (pail 17, drum 35, barrel 37, statue 4, tires 14 records); stone and boulder are never placed | R 0x28C8A; stage_scripts | DONE | answers the "13 variants" of the plan |
| IT06 | One hit of any hero breaks a normal container (HP 4): +100 for the hit and +500 at the break (item made 2 frames after the hit) | M cj_*, ch_4_* | DONE | |
| IT07 | What breaks a container: hero attack frames, bullets, thrown knives, thrown containers, explosions (M); falling bodies (HP minus 8) and enemy attack frames (HP 0, no credit) (R only) | R 0x11962, 0x120D2, 0x11F86, 0x1201C; M gun_hits_barrel, knife_hits_barrel, throw_hits_barrel, expl_hits_* | DONE | enemy case unseen (A27) |
| IT08 | Tire stacks and statues cannot be lifted (9 and 3 offsets tried); a barrel is lifted only from 24 px beside it with the stick towards it, not from on top | M lift_* | DONE (check) | answers D2c open item 15 |
| IT09 | Explosive drum or barrel: HP 50 (jab chain 10, 10, 15, 15), shimmer every 51 frames, fuse of 4 stages from the first hit that leaves HP above 0 (112 frames to the explosion), HP 0 explodes at once; the explosion path gives no +500 | R 0x28E5A; M tnt_2, tnt_5hits, tnt_idle | DONE (fuse spark effect MISSING) | 3 explosive records in the scripts |
| IT10 | Break effects: pail body flies and blinks 40 frames; drum 2 pieces; barrel 10 puff objects; stone and boulder 9 pieces; statue 8 puff objects; tires 5 puff objects | M cj_*; R 0x296AC | MISSING (effects) | D3f |
| IT11 | Stones from debris (stone, boulder, falling rock): each of 9 pieces can become a stone with chances 2/16, 8/16, 3/16 while `G.$4F1 < 3` at the break; boulder pieces give size 0 or 1; stones never expire; G08 is true by chance; D2c open item 1 closed | R 0x294AA, 0x93770; M stones_*, stones2_boulder, stone_items | MISSING | needs IT10 |
| IT12 | Drop tables: containers, rolling barrels (class 65) and rocks (class 66) read the container table 0xB64F2; all other enemies and bikers (class 70) the enemy table 0xB6D12; barrels roll at the hit, enemies at the killing blow | R callers of 0x24B36 and 0x24B48; M barrel65_drop, biker70_drop, edrop_kill | PARTIAL (item side DONE) | the classes are D3a |
| IT13 | Drop code 0x0F: an ammo box when any gun item (rifle, gun, Uzi, M-16, shotgun, bazooka; held or lying) exists, else one of six cheap treasures; a stone can never be dropped by a row | R 0x24C3A; M sp_* | DONE | |
| IT14 | A dropped item hops out (48 px up, 3 px per frame, lands after about 48 frames; cannot be taken before) and bounces back in from the screen edge (X <= camera X + 0x10: +1.5 px per frame, X >= camera X + 0x170: -1 px per frame, at most 5) | R 0x24CE6; M hop_* | DONE | |
| IT15 | A full pool E (24 slots) loses the drop | M pool_22, pool_23 | DONE | |
| IT16 | Despawn windows: food, treasure, containers, enemies in `[camera X - 0xC0, + 0x240)`; weapons, ammo boxes, rolling barrels and rocks in `[camera X - 0x40, + 0x200)` | R 0x173C, 0x1760; M despawn_x | DONE | |
| IT17 | Nothing on the floor times out (food, treasure, ammo, placed or enemy dropped or container dropped weapons, stones); only a weapon dropped by a hero (3,600 frames), floor knives 4 and up (600), the join bazooka (600) | M life_long, stone_items; D2c 8 | DONE | |
| IT18 | A dynamite put down by a weapon swap is lit when it lands (4 stages of 15 frames); the fuse pauses in the hand and continues; a thrown one explodes 60 frames after rest | M dyn_swap_*; R `weapon_dropped` | DONE | confirms M18 |
| IT19 | Placed items: 30 on the floor in 5 episodes; Episode 2's pearl record (X 0x1A8 at trigger 0x428) is freed after 2 frames; the other three Episode 2 treasures lie at depth 16 height 64 behind the trunks | R stage_scripts; M e2_pearl, e2_start, e2_diamond | DONE (data driven; foreground layer UNKNOWN) | |
| IT20 | Episode 3 area script: 12 radios (4 containers, 8 enemies), 17 empty containers, no food or weapon; Hogg's helper list: guns Uzi, shotgun, rifle, shotgun, shotgun, M-16, Uzi, two small food chances | R stage_scripts, b22_lists | MISSING (controller) | corrects L02 |
| IT21 | 42 item or drop records exist only for 2 or 3 players: rod and club (Episode 1), shotgun and M-16 (Episode 4), more food and treasure up to Episode 7; none in Episode 8; L12 is partly right | R stage_scripts; M e1a0_c*p, e4a2_w*p | DONE | |
| IT22 | 23 records change with the level: Uzi 31 of 32 or 8 of 32 (Episode 7 area 0), lobster or parfait, cake or coffee, sushi or hot dog, an M-16 only at level 8 or less | R; M e7a0_lv*, e4a2b_m16_* | DONE | low rank gets better help |
| IT23 | Boss helper lists of Episodes 1 to 4 carry drop rows (Vice steak and small foods, Butcher food rows, Hogg guns, Slice specials) | R b22_lists | MISSING | D3c |
| IT24 | Every GameFAQs barrel and drop note checked agrees with the rows (section 10) | R, M | n/a | the FAQ can serve as a checklist |
| IT25 | Walther (class 18 var 0x200) drops a revolver 32 of 32 in Episodes 4 and 5 and the radio in Episode 3; confirms M04 | R stage_scripts | MISSING (class) | |
| IT26 | The HUD item message: FooD icon with name and GOOD !! or points; Pts icon with name and points; a rank label (10TH) shows after the score | M hud_clean | PARTIAL | |
| IT27 | Rolling barrels are freed 64 px left of the camera (near window), which is the "only if you go slowly" of Episodes 6 and 8; not the camera lock | R 0x5B9D0; M despawn_x | MISSING (class 65) | |
| IT28 | The Episode 2 treasures are drawn behind the foreground trunks; the hero at depth 16 is half hidden too | M e2_diamond | UNKNOWN | port draw order not checked |
| IT29 | **Port bug:** the extracted drop table ends at 0xB7312 (enemy row 0x2F); Episode 1 wave 4 uses enemy row 0x30, so a kill panics the port | R `dino-extract/src/enemy.rs`; stage_scripts | BUG | task IT-P1 |
| IT30 | The Episode 5 area 0 drum for 3 players (X 0x2D0, trigger 0x80) is past the far window when it fires and is freed at once (derived, not seen) | R; M despawn_x | DONE (data driven) | A26 |

## 15. Web sources, files

- GameFAQs, "Guide and Walkthrough" by ninjasrok, version 2.2 (December 2006): https://gamefaqs.gamespot.com/arcade/575551-cadillacs-and-dinosaurs/faqs/45433, as copied in
  `plan/game/sources/claude-doc-deep-dive-supplement-2.md` (scene by scene items and barrels, food tiers). Player written; every item statement agreed with the ROM (section 10).
- The fan lists M16, M17, M18, M26 in `plan/game/sources/claude-doc-deep-dive-supplement.md` (Namu Wiki, machine translated); L02, L06 to L12 in `claude-doc-recreation-blueprint.md`; G08, G18 in `gpt_suggestions.md`.
  Where they disagree with the ROM or MAME, the ROM and MAME were used.
- Files: `re/out/d4a/gen_r1.py` to `gen_r4.py`, `batch1.txt` to `batch4.txt` (the probe batches); `decode_drops.py`, `decode_b22.py` (row decoders), `mk_tables.py`, `mk_b22.py` (the tables of this file),
  `an_cont.py` (log analysis); contact sheets `food_sheet.png`, `treasure_sheet.png`, `container_sheet.png`, `hud_clean.png`, `hud_food_time.png`, `hud_treasure.png`, `e2_start_sheet.png`,
  `e2_diamond_sheet.png`, `e1a1_e5a0_sheet.png`. Probe logs: `re/out/probe/d4a-r1` to `d4a-r4`. The decoded tables of both drop tables: `re/specs/enemy/items_drops.txt`.
