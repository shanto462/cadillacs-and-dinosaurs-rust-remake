# M9-08 audit A (second pass): mechanics rows, groups A to E and the listed design blocks
Updated 2026-10-21 (task 67): whole-run checks applied; see the closing section at the end.

Second pass (desk Haiku worker, task 55). Verdicts come from the port code (address or class grep in `dino-rs/crates`) and the recordings in the sweep refresh 6, not from the Port column of `mechanics-seen.md`. No source change.

**Method:** (1) every original address in Source, Notes and Mechanic is grepped in `dino-rs/crates` (case-insensitive, leading zeros ignored); the first hit is the port file:line. (2) If no address hits: the class number ("class N") is looked for as `classN.rs` or a dispatch `N =>`. (3) If still no hit: the first word of the mechanic is looked for in `dino-game/src/game` only. This name hit is WEAK (a common word can match unrelated code). (4) Recordings: a row is linked to a sweep recording when its MAME tag word (for example `M r2:a-e6a1-storm`) is part of a recording name; equal = first difference none; listed difference = the recording has a first difference (frame given).

**Limits:** the covering task is the group level list of `D5-final-plan.md` section 7 (the plan maps tasks per group). Only rows with a tag that matches a recording name get a checked verdict, so most rows are "implemented not checked" even when the code exists. The name hits (161 rows) are not proof of implementation.

## Counts

Verdicts of the main table (each row once). "Before" is the count at task 55 / 57 / 59; "after" is 2026-10-21.

| verdict | before | after |
| --- | --- | --- |
| equal | 46 | 73 |
| listed difference | 13 | 13 |
| implemented not checked | 306 | 279 |
| missing | 11 | 11 |
| total | 376 | 376 |

## A. Player

Covering tasks (group level): M2-01, M2-03, M2-04, M2-08, M2-09, M2-12, M2-14 to M2-21, M3-30, M4-17, M5-02, M5-05, M5-07, M5-18 to M5-21, M6-14, M6-21, M9-01 to M9-03, M10-19

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| P01 | Walk, run by double tap; run ends by timeout (240 f), releasing the stick does not stop it | group A | dino-game/src/game/flow.rs:387 | no recording found by the row tag | implemented not checked |
| P02 | Jump with a vector per hero, air control, jump attacks, dive kick at the top of the jump | group A | dino-game/src/game/boss.rs:213 | no recording found by the row tag | implemented not checked |
| P03 | Wall jump (Hannah, Mustapha) with a "triangle jump" attack, high priority | group A | dino-game/src/game/terrain.rs:232 | no recording found by the row tag | implemented not checked |
| P04 | Punch chain (2 jabs, body hit, finisher), combo window 40 f, hit confirm | group A | dino-game/src/game/enemy_combat.rs:335 | no recording found by the row tag | implemented not checked |
| P05 | Combo throw after the chain (shoulder throw; Mess slam / suplex) | group A | dino-game/src/game/enemy_combat.rs:2114 | no recording found by the row tag | implemented not checked |
| P06 | Grab by walking into an enemy, rapid attack = knees (each costs enemy stun and HP), throws forward a | group A | dino-game/src/game/enemy_combat.rs:2 | no recording found by the row tag | implemented not checked |
| P07 | Dash attack: Jack slides, the others use an air lunge | group A | dino-game/src/game/enemy.rs:1140 | no recording found by the row tag | implemented not checked |
| P08 | Special ("mega crush", attack + jump): costs 10 HP for the first victim, 4 for the 2nd and 3rd, neve | group A | dino-game/src/game/prop.rs:23 | no recording found by the row tag | implemented not checked |
| P09 | Hannah's special is airborne and hovers | group A | dino-game/src/game/body.rs:30 | no recording found by the row tag | implemented not checked |
| P10 | Command move (down, up + attack) per hero ("sure kill" move) | group A | dino-game/src/game/flow.rs:975 | no recording found by the row tag | implemented not checked |
| P11 | Tag team move: overlap an ally for 2 s, the ally is flung (Let's Go, Jump, Roll, Fly) | group A | dino-game/src/game/player_weapon.rs:29 | no recording found by the row tag | implemented not checked |
| P12 | Players hurting each other: 1 HP per hit with 2 or 3 players (Z window table), never below 1 HP | group A | dino-game/src/game/player_combat.rs:172 | no recording found by the row tag | implemented not checked |
| P13 | Hurt reactions: light flinch, pushback with dust, knockdown (front and from behind), heavy knockdown | group A | dino-game/src/game/enemy_combat.rs:412 | no recording found by the row tag | implemented not checked |
| P14 | Hit stop on both sides (frames from the attack table) | group A | dino-game/src/game/flow.rs:680 | no recording found by the row tag | implemented not checked |
| P15 | Lying down: mash the stick (L or R counts 3, else 1) or the attack button to get up sooner; lie time | group A | dino-game/src/game/enemy_combat.rs:334 | no recording found by the row tag | implemented not checked |
| P16 | Dizzy (stars): 3rd hit in a 60 f chain always, 2nd 75 percent, only if HP >= 30; stars can be mashed | group A | dino-extract/src/enemy.rs:317 | no recording found by the row tag | implemented not checked |
| P17 | Mercy rule: a hit that would kill with at most 4 HP over leaves 1 HP | group A | dino-game/src/game/player_combat.rs:678 | no recording found by the row tag | implemented not checked |
| P18 | Damage taken scales with enemy type and rank level, hero factor, low HP random cut and low HP factor | group A | dino-game/src/game/boss.rs:47 | no recording found by the row tag | implemented not checked |
| P19 | Invulnerability windows: after get up 20 f, after a throw 12/10/14/18, after a pickup 10/8/20/10, af | group A | dino-game/src/game/effect.rs:18 | no recording found by the row tag | implemented not checked |
| P20 | Frame classes 6, 8, 0xA are immune to enemy attacks (special, some moves) | group A | dino-game/src/game/boss.rs:169 | no recording found by the row tag | implemented not checked |
| P21 | Death: 60 f blink, lose one life, respawn drop-in, or game over for that player | group A | dino-game/src/game/boss.rs:51 | no recording found by the row tag | implemented not checked |
| P22 | Respawn speech bubble ("they will pay for this") and bazooka at a continue | group A | dino-game/src/game/terrain.rs:13 | no recording found by the row tag | implemented not checked |
| P23 | Lives: 1 extra life by default (2 in all), extra life at 500000 points with a 1UP marker and a pose | group A | dino-game/src/game/flow.rs:166 | no recording found by the row tag | implemented not checked |
| P24 | Score per hit (100/200/400), per kill, container +500, treasure, food at full HP | group A | dino-game/src/game/flow.rs:227 | no recording found by the row tag | implemented not checked |
| P25 | HP bar blinks at 15 HP or less; face changes when hurt | group A | dino-game/src/game/flow.rs:890 | no recording found by the row tag | implemented not checked |
| P26 | Weapons: pick up with the attack button (no auto pickup), hold, swing, charge swing (hold attack; cr | group A | dino-game/src/game/enemy_combat.rs:334 | no recording found by the row tag | implemented not checked |
| P27 | Weapon list: gun, Uzi, M-16A1, rifle, shotgun, bazooka, knife, stones, grenade, dynamite, magazine ( | group A | dino-game/src/game/enemy/class6.rs:1 | no recording found by the row tag | implemented not checked |
| P28 | Hannah stabs repeatedly with a knife when it flashes red | group A | dino-game/src/game/body.rs:30 | no recording found by the row tag | implemented not checked |
| P29 | Heavy objects (barrels): lift, walk, jump, throw; Hannah cannot jump with one | group A | dino-game/src/game/enemy_combat.rs:865 | no recording found by the row tag | implemented not checked |
| P30 | 40 frame pickup lock after a weapon pickup | group A | dino-game/src/game/boss.rs:169 | no recording found by the row tag | implemented not checked |
| P31 | Food is eaten on the pickup press, heals or gives points when full | group A | dino-game/src/game/trace.rs:1154 | no recording found by the row tag | implemented not checked |
| P32 | Join in during play: blinking JOIN-IN or INSERT COIN in the HUD, quick select, drop in | group A | dino-game/src/game/flow.rs:1 | no recording found by the row tag | implemented not checked |
| P33 | Continue: countdown 20 to 0, taunts, quick select, drop in with a bazooka | group A | dino-game/src/game/flow.rs:2 | no recording found by the row tag | implemented not checked |
| P34 | Time over kills every living player at once | group A | dino-game/src/game/rank.rs:2 | no recording found by the row tag | implemented not checked |
| P35 | Players are kept inside the screen, cannot go left of the camera | group A | dino-game/src/game/boss.rs:438 | no recording found by the row tag | implemented not checked |
| P36 | Wading in water (legs cut at the water line), splash on landing, ripples | group A | dino-game/src/game/enemy_combat.rs:886 | no recording found by the row tag | implemented not checked |
| P37 | Riding the Cadillac in Episode 3 and the vehicle entrance state 0x432 | group A | dino-game/src/game/enemy_combat.rs:612 | whole run ep3_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| P38 | Scripted control: walk in at the start, automatic walk at area end, stage exit | group A | dino-game/src/game/flow.rs:163 | no recording found by the row tag | implemented not checked |
| P39 | Per hero voices on hit, attack, special | group A | dino-game/src/game/script.rs:920 | no recording found by the row tag | implemented not checked |
| P40 | Respawn flash (pool C class 0x38), sparkle (0x5C), 1UP marker (pool B class 0x48) | group A | dino-game/src/game/terrain.rs:13 | no recording found by the row tag | implemented not checked |
| P41 | Players can pick which hero they play at the start and at a continue (can change hero) | group A | dino-game/src/game/rank.rs:608 | no recording found by the row tag | implemented not checked |
| P42 | Idle behaviour during play: what a hero does when the player does not touch the controls (idle anima | group A | dino-game/src/game/palette.rs:9 | no recording found by the row tag | implemented not checked |

## B. Enemies

Covering tasks (group level): M3-02, M3-03, M3-14, M3-20, M3-30, M5-13, M5-22, M6-01 to M6-03, M6-07, M6-08, M6-11, M6-27, M6-29, M6-30, M7-07 to M7-11, M7-16, M8-01

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| E01 | Enemy census, target selection: prefer a player with fewer than 3 enemies | group B | dino-game/src/game/flow.rs:388 | no recording found by the row tag | implemented not checked |
| E02 | Aggression 0..7: forced zones from the script, re-rolled at every kill | group B | dino-game/src/game/flow.rs:1081 | no recording found by the row tag | implemented not checked |
| E03 | Rank and level scale HP, damage, recovery, dizzy odds, level-gated spawns | group B | dino-game/src/game/flow.rs:166 | no recording found by the row tag | implemented not checked |
| E04 | Entrances: walk, dash, wait, fade in, jump, drop; spawn conditions by player count and level | group B | dino-game/src/game/enemy/class13_ai.rs:3 | no recording found by the row tag | implemented not checked |
| E05 | Attack slots around the player, 3 per side, flanking, slot rotation | group B | dino-game/src/game/enemy_combat.rs:25 | no recording found by the row tag | implemented not checked |
| E06 | Poise: flinch until poise 0, then knockdown; counter attack after a flinch | group B | dino-game/src/game/body.rs:127 | no recording found by the row tag | implemented not checked |
| E07 | Dead enemy blinks 120 f; item dropped at the killing blow; kill re-rolls aggression | group B | dino-game/src/game/boss.rs:143 | no recording found by the row tag | implemented not checked |
| E08 | One dizzy enemy at a time, dizzy enemies can be hit freely; Vice and Walther never get dizzy | group B | dino-game/src/game/flow.rs:409 | no recording found by the row tag | implemented not checked |
| E09 | Thrown enemy bodies hurt other enemies | group B | dino-game/src/game/enemy_combat.rs:398 | no recording found by the row tag | implemented not checked |
| E10 | Dead big enemies bounce off the screen sides; wall slump | group B | dino-game/src/game/boss.rs:143 | no recording found by the row tag | implemented not checked |
| E11 | Grabbed enemy follows the hero's hand using per frame offsets | group B | dino-game/src/game/enemy_combat.rs:1558 | no recording found by the row tag | implemented not checked |
| E12 | Kill-all when a boss dies (score goes to the boss killer) and when a wave takes over 7200 f (no scor | group B | dino-game/src/game/boss.rs:430 | no recording found by the row tag | implemented not checked |
| E13 | Enemy name, portrait, bar with drain and skull blink in the HUD message area | group B | dino-game/src/game/boss.rs:283 | no recording found by the row tag | implemented not checked |
| E14 | Enemies far off screen are freed (0x173C), also vertical Z+Y test in the lift | group B | dino-game/src/game/enemy.rs:705 | no recording found by the row tag | implemented not checked |
| E15 | Hammer / Blk Elmer / Wrench: paired attack on a downed hero, stomp; the grab test never passes (bug  | group B | dino-game/src/game/effect.rs:52 | no recording found by the row tag | implemented not checked |
| E16 | Punk / Thug: hops, jump kicks, leaping dive | group B | dino-game/src/game/enemy/class64.rs:1 | no recording found by the row tag | implemented not checked |
| E17 | Blade / Razor: knife throwers, one record makes 3 objects, parts classes 60, 34, 41, 15..17, drop kn | group B | dino-game/src/game/enemy/class14.rs:1 | no recording found by the row tag | implemented not checked |
| E18 | Poachers (Poacher J, Skinner, Gutter): ex soldiers with rifles | group B | dino-game/src/game/placeholder.rs:76 | no recording found by the row tag | implemented not checked |
| E19 | Lash T.: mace in Episode 2 [W:aqm]; appears in 4 later episodes | group B | dino-game/src/game/placeholder.rs:74 | whole run ep2_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| E20 | Bludge: human dinosaur hybrid, tongue attack, acid spit | group B | dino-game/src/game/effect.rs:76 | no recording found by the row tag | implemented not checked |
| E21 | Walther: leader with a gun, same class as Vice | group B | dino-game/src/game/placeholder.rs:26 | no recording found by the row tag | implemented not checked |
| E22 | Zeke: pteranodon that flies in, hits the heroes; can be hit only by jumping | group B | dino-game/src/game/enemy/class82.rs:1 | no recording found by the row tag | implemented not checked |
| E23 | Enemies carry weapons that heroes can take (butcher knife) | group B | dino-game/src/game/boss.rs:29 | no recording found by the row tag | implemented not checked |
| E24 | Enemy classes 65 and 70, pool A 5, 16, 22, 27, 35, 43, 45, 66, 78, 80: roles unknown | group B | dino-game/src/game/boss.rs:39 | no recording found by the row tag | implemented not checked |
| E25 | Boss reinforcement controller (pool B 22): timed list, threshold list, only after a summon | group B | dino-game/src/game/boss.rs:1 | no recording found by the row tag | implemented not checked |
| E26 | Anti tamper words at 0x2A5B6, 0x29ADE, 0x104F6 do nothing on a real ROM | group B | dino-game/src/game/enemy.rs:1529 | no recording found by the row tag | implemented not checked |

## C. Dinosaurs and flyers

Covering tasks (group level): M5-10, M6-02 to M6-06, M6-16, M6-17, M7-10 to M7-12, M8-01, M9-07

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| D01 | Sleeping dinosaurs (Z letters, a roar every 50 to 113 f) | group C | dino-game/src/game/enemy.rs:456 | no recording found by the row tag | implemented not checked |
| D02 | Poachers' hits wake a dinosaur and it attacks poachers as well as heroes | group C | dino-game/src/game/enemy.rs:385 | no recording found by the row tag | implemented not checked |
| D03 | Hit by a hero it becomes hostile: palette flash, then enraged (red) | group C | dino-game/src/game/terrain.rs:716 | no recording found by the row tag | implemented not checked |
| D04 | At 0 HP the dinosaur calms down (turns green) and walks away, gives no score | group C | dino-game/src/game/enemy_combat.rs:1996 | no recording found by the row tag | implemented not checked |
| D05 | A dinosaur leaves when the camera passes the X in its spawn record | group C | dino-game/src/game/enemy_combat.rs:1996 | no recording found by the row tag | implemented not checked |
| D06 | Shivat (T. rex) attacks: bite 14, tail swing 10 with knockdown, triple stomp, grab and hold (cooldow | group C | dino-game/src/game/placeholder.rs:78 | no recording found by the row tag | implemented not checked |
| D07 | Mack (triceratops) hostile | group C | dino-game/src/game/placeholder.rs:80 | no recording found by the row tag | implemented not checked |
| D08 | Rock Hopper (velociraptor): Vice's partner in Episode 1, two of them in Episode 5 | group C | dino-game/src/game/effect.rs:21 | whole run ep1_3p equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| D09 | Zeke (pteranodon, class 82): a big orange flyer that circles above the heroes in long sine like swoo | group C | dino-game/src/game/enemy/class82.rs:1 | no recording found by the row tag | implemented not checked |
| D10 | Ambient pterodactyls in the sky (bird controllers 49, spawner 73, birds 50) | group C | dino-game/src/game/prop/class86.rs:10 | no recording found by the row tag | implemented not checked |
| D11 | Brontosaurus legs crush whoever is under them at two places in Episode 6 | group C | dino-game/src/game/prop.rs:281 | no recording found by the row tag | implemented not checked |
| D12 | Dinosaurs turn red when enraged and change colour when drained (palette) | group C | dino-game/src/game/placeholder.rs:10 | no recording found by the row tag | implemented not checked |
| D13 | Does fire or a poacher bullet also enrage a neutral dinosaur? | group C | dino-game/src/game/boss.rs:162 | no recording found by the row tag | implemented not checked |
| D14 | The time over scene has no pteranodon: a red bird flew by by chance (ambient) | group C | dino-game/src/game/flow.rs:101 | no recording found by the row tag | implemented not checked |

## D. Bosses

Covering tasks (group level): M3-03, M3-04, M3-29, M5-03, M5-09, M5-11 to M5-13, M6-09 to M6-12, M6-18 to M6-26, M7-01 to M7-04, M7-16 to M7-21, M8-06 to M8-15, M8-27 to M8-35

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| B01 | Vice Terhune: gun volley, jump attack, rage at HP 50 and 25, summons at 1/2, 1/3, 1/4 HP | group D | dino-game/src/game/boss.rs:29 | no recording found by the row tag | implemented not checked |
| B02 | The Butcher: leaps, lands on the hero with his rear, two knives can be knocked away and used | group D | dino-game/src/game/boss.rs:46 | no recording found by the row tag | implemented not checked |
| B03 | Hogg: grenades, 3 vitality bars, rammed by the Cadillac, flies around when the fight drags on | group D | dino-game/src/game/boss.rs:7 | no recording found by the row tag | implemented not checked |
| B04 | Slice: boomerangs that multiply, very fast | group D | dino-game/src/game/placeholder.rs:43 | no recording found by the row tag | implemented not checked |
| B05 | Morgan changes into Morgue (wild dash); the change is shown on screen | group D | dino-game/src/game/boss.rs:485 | no recording found by the row tag | implemented not checked |
| B06 | Tyrog three times (green, red, blue): rib attacks, acid spit; Tyrog 2 returns in Episode 8 | group D | dino-game/src/game/enemy_combat.rs:322 | no recording found by the row tag | implemented not checked |
| B07 | Slisaurs: 2 for 1 player, 3 for 2 or 3 players | group D | dino-game/src/game/enemy/class71.rs:1 | no recording found by the row tag | implemented not checked |
| B08 | Fessenden: two forms | group D | dino-game/src/game/enemy_combat.rs:551 | no recording found by the row tag | implemented not checked |
| B09 | Boss HP bar in the HUD with extra bars above 100 HP; boss HP table by players and level | group D | dino-game/src/game/boss.rs:1 | no recording found by the row tag | implemented not checked |
| B10 | Boss intro scene (Vice walks in, raises his gun, boss music) and boss music change | group D | dino-game/src/game/boss.rs:1 | no recording found by the row tag | implemented not checked |
| B11 | Slow motion (task period 4) while the boss dies, hero walk off, tally | group D | dino-game/src/game/boss.rs:430 | no recording found by the row tag | implemented not checked |
| B12 | Each boss death: rank speed +1, kill-all event, clear absent players' HUD blocks | group D | dino-game/src/game/boss.rs:1 | no recording found by the row tag | implemented not checked |

## E. Items, containers, hazards, special scenes

Covering tasks (group level): M2-15, M3-01, M3-08 to M3-14, M3-16, M3-19, M5-06, M5-07, M5-14, M5-21, M6-15 to M6-22, M6-29, M6-30, M7-05, M7-22 to M7-25, M8-02, M8-05, M8-18 to M8-21, M8-23, M8-24, M9-04, M9-08, M9-10

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| W01 | Containers (drums, barrels, tires, buckets, armor, TNT) break in 4 HP, can be lifted, thrown, crush  | group E | dino-game/src/game/terrain.rs:16 | no recording found by the row tag | implemented not checked |
| W02 | TNT barrel: fuse blink stages, then explosion that hurts all | group E | dino-game/src/game/enemy.rs:108 | no recording found by the row tag | implemented not checked |
| W03 | Food heals 16 to 100 or gives points at full HP; 18 foods | group E | dino-game/src/game/trace.rs:1154 | no recording found by the row tag | implemented not checked |
| W04 | Treasure gives 1000 to 10000 points; hidden behind trees in Episode 2 | group E | dino-assets/src/sound.rs:50 | whole run ep2_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| W05 | Drop rolls: enemy table and container table, guns reroll to ammo or a treasure | group E | dino-game/src/game/enemy_combat.rs:39 | no recording found by the row tag | implemented not checked |
| W06 | Dropped weapons last 3600 f; items bounce back in from the screen edge up to 5 times | group E | dino-game/src/game/enemy_combat.rs:759 | no recording found by the row tag | implemented not checked |
| W07 | RADIO in Episode 3 calls the Cadillac and triggers kill-all | group E | dino-game/src/game/car.rs:1 | whole run ep3_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| W08 | Torches set enemies on fire | group E | dino-game/src/game/enemy/class6.rs:1 | no recording found by the row tag | implemented not checked |
| W09 | Fire bombs, dynamite, grenades, bazooka rockets explode with splash | group E | dino-game/src/game/enemy_combat.rs:2258 | no recording found by the row tag | implemented not checked |
| W10 | Falling rocks on the mountain side (Episode 5) | group E | dino-game/src/game/terrain.rs:878 | whole run ep5_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| W11 | Burning huts and flames in the village (Episode 5) | group E | dino-game/src/game/enemy_combat.rs:1184 | whole run ep5_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| W12 | Flame jets in the sewers, lure enemies into them (Episode 8) | group E | dino-game/src/game/enemy_combat.rs:1185 | no recording found by the row tag | implemented not checked |
| W13 | Barrels rolling down the cave slope (Episode 8), rolling drums with items in the library (Episode 7) | group E | - | no address, class or name hit (addresses tried: none) | missing |
| W14 | Elevator ride with enemies riding in at each stop (Episode 7) | group E | dino-game/src/game/terrain.rs:20 | no recording found by the row tag | implemented not checked |
| W15 | Cadillac drive: camera mode 8, car state G.$518, special collision passes, runs over enemies and bar | group E | dino-game/src/game/prop.rs:40 | no recording found by the row tag | implemented not checked |
| W16 | Row scroll (road bends) in Episode 3 | group E | dino-game/src/game/flow.rs:540 | whole run ep3_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| W17 | Mine lamps (palette cycling), rain | group E | dino-game/src/game/prop.rs:25 | no recording found by the row tag | implemented not checked |
| W18 | Screen shake on stomps, landings, the lift (0x222D6) | group E | dino-game/src/game/mod.rs:346 | no recording found by the row tag | implemented not checked |
| W19 | Tile animation (water, fire) and palette cycling objects | group E | dino-game/src/game/terrain.rs:18 | no recording found by the row tag | implemented not checked |
| W20 | Suits of armor in the Episode 1 hall hide items | group E | - | no address, class or name hit (addresses tried: none) | missing |
| W21 | Lab tanks (01, 02, 03) open and release Bludge, Hopper, Tyrog 2 (Episode 8) | group E | dino-game/src/game/prop/b44.rs:12 | no recording found by the row tag | implemented not checked |
| W22 | Garage props (Episode 4), doors, gates | group E | dino-game/src/game/prop/b1.rs:1 | whole run ep4_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| W23 | The "GO" burst arrow at the right edge and the idle hint after 600 f | group E | dino-game/src/game/prop.rs:29 | no recording found by the row tag | implemented not checked |
| W24 | Pool B classes with unknown role: 59 distinct in the scripts, 57 not ported | group E | dino-game/src/game/boss.rs:1 | no recording found by the row tag | implemented not checked |
| W25 | Breakable parts of the background: pipes in walls (the user saw them), and probably windows, signs,  | group E | dino-game/src/game/prop.rs:6 | no recording found by the row tag | implemented not checked |
| W26 | Catch-all: every other interactive thing in the levels (switches, doors, things that fall or roll wh | group E | - | no address, class or name hit (addresses tried: none) | missing |

## Design block D2a

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| P43 | Tag team: both heroes still and overlapped (less than 32 px in X, 0 to 15 in Z, no input, class 0) f | D2a block | dino-sound/src/pan.rs:21 | no recording found by the row tag | implemented not checked |
| P44 | Friendly fire is active when fewer than 3 enemies are alive (`G.$4F3` is the enemy census count), de | D2a block | dino-game/src/game/player_combat.rs:172 | no recording found by the row tag | implemented not checked |
| P45 | 1UP pose per hero (set 0 id 0xE): Jack, Mess ground state 0x18; Mustapha, Hannah air states 0x418, 0 | D2a block | dino-assets/src/text.rs:80 | no recording found by the row tag | implemented not checked |
| P46 | Frames without a hurt box: special, throw, pick up, 1UP pose, tag team swing, drop in; dash recovery | D2a block | dino-game/src/game/boss.rs:246 | no recording found by the row tag | implemented not checked |
| P47 | Dive kick window is 2 to 3 frames at the top of the jump (Jack and Mess top = split height) | D2a block | dino-game/src/game/player.rs:2273 | no recording found by the row tag | implemented not checked |
| P48 | Dive (0x0E) and running jump attack (0x10) deal 1.5 times the record damage to almost all enemy type | D2a block | dino-game/src/game/enemy_combat.rs:216 | no recording found by the row tag | implemented not checked |
| P49 | Attack power factor `$C6` is 32 in 27 of 32 rolls at HP >= 75, re-rolled when the hero is hit; 64 po | D2a block | dino-game/src/game/enemy_combat.rs:39 | no recording found by the row tag | implemented not checked |
| P50 | Wall jump needs terrain wall bit 4 (or 0x18, 0x1C), found in Episode 1 areas 0 to 2, Episode 5 area | D2a block | dino-game/src/game/player.rs:9 | whole run ep1_3p equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| P51 | Thrown enemies hurt other enemies for 5 damage (enemy attack row 0x11), knock down | D2a block | dino-game/src/game/enemy_combat.rs:398 | no recording found by the row tag | implemented not checked |
| P52 | Score per hit from the record's score code: 100 / 200 / 300 / 400 / 700 / 800 / 1000 | D2a block | dino-assets/src/text.rs:81 | no recording found by the row tag | implemented not checked |
| P53 | Hannah's knife stab rate comes from her 5 frame swing animation; the item flashes red when an enemy  | D2a block | dino-game/src/game/body.rs:30 | no recording found by the row tag | implemented not checked |
| P54 | Scripted player states 0x2A (two hops) and 0x2C (pose with an effect, leader only) and the scripted  | D2a block | dino-game/src/game/script.rs:297 | no recording found by the row tag | implemented not checked |
| P55 | Stage clear and idle: no pose, no taunt; heroes idle during the tally | D2a block | dino-game/src/game/boss.rs:4 | no recording found by the row tag | implemented not checked |
| P56 | Run start burst (7 frames Jack, 5 others at class 8 speed), double tap window 12 frames, run lasts 2 | D2a block | dino-sound/src/driver.rs:450 | no recording found by the row tag | implemented not checked |

## Design block D2b

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| P61 | Lost life respawn timeline: death flight 183 frames (dead vectors, half speed), 61 frame blink, resp | D2b block | dino-game/src/game/player.rs:701 | no recording found by the row tag | implemented not checked |
| P62 | Respawn air strike: 5 bombs 12 frames apart in a fixed screen pattern (not aimed), explosions 59 fra | D2b block | dino-extract/src/enemy.rs:342 | no recording found by the row tag | implemented not checked |
| P63 | Bazooka (4 rockets, vanishes 600 frames after a drop) only for respawns with `+$07 == 0`: join with  | D2b block | dino-game/src/game/player.rs:746 | no recording found by the row tag | implemented not checked |
| P64 | Respawn speech bubble: pool C class 14, text "They're gonna pay for this!" for one player and "Need  | D2b block | dino-game/src/game/effect.rs:17 | no recording found by the row tag | implemented not checked |
| P65 | Respawn invulnerability is `+$00 = 2` with `$EC = 360` counted only after the 61 frame wait: 421 fra | D2b block | dino-game/src/game/player.rs:833 | no recording found by the row tag | implemented not checked |
| P66 | Each death: rank minus 4 (3 if a drop was less than 600 frames ago), calm period 1200 frames (aggres | D2b block | dino-game/src/game/rank.rs:289 | no recording found by the row tag | implemented not checked |
| P67 | Lie time, mash meter and dizzy tables use row 0 only (level band byte read bug): lie 6 (37.5 percent | D2b block | dino-game/src/game/player.rs:2824 | no recording found by the row tag | implemented not checked |
| P68 | Dizzy needs the knockdown to be the 4th or later hit of a chain (gaps of 60 frames or less): 75 perc | D2b block | dino-game/src/game/player.rs:2794 | no recording found by the row tag | implemented not checked |
| P69 | Hero damage factor table 0x2C88E by hero id: Jack 32, Mustapha 34, Hannah 35, Mess 30; enemy to hero | D2b block | dino-extract/src/enemy.rs:153 | m800_bta6_sl_dmg_1p (first diff 253), m800_bta6_sl_dmg_2p (first diff 253), m800_bta6_sl_dmg_3p (first diff 216) | listed difference |
| P70 | Extra life lists are absolute scores: 300,000 and 700,000, **500,000 and 1,000,000 (factory)**, or 1 | D2b block | dino-assets/src/text.rs:80 | no recording found by the row tag | implemented not checked |
| P71 | HP is refilled to 100 at the start of every episode (not between areas); lives and score carry over | D2b block | dino-game/src/game/flow.rs:200 | no recording found by the row tag | implemented not checked |
| P72 | There are no floating score numbers: kills, hits and pick ups only change the HUD score; pick ups sh | D2b block | dino-assets/src/sound.rs:50 | no recording found by the row tag | implemented not checked |
| P73 | Scoring numbers: hit points from the record code, kill by name table, container +500 (plus the hit), | D2b block | dino-assets/src/player.rs:112 | no recording found by the row tag | implemented not checked |
| P74 | TIME OVER sets HP 0 for every hero with a state byte below 4, also invulnerable ones; the death flow | D2b block | dino-game/src/game/rank.rs:2 | no recording found by the row tag | implemented not checked |
| F42 | The life counter in the HUD block is one tile whose code is the spare life count (digits, then lette | D2b block | dino-game/src/game/flow.rs:166 | no recording found by the row tag | implemented not checked |
| F43 | At HP 15 or less the name, lives and score rows of the player block change palette every 6 frames (r | D2b block | dino-game/src/game/mod.rs:352 | no recording found by the row tag | implemented not checked |
| W39 | Air strike side effects: containers break (+500), TNT barrels ignite (HP 50 to 20, fuse, explosion a | D2b block | dino-game/src/game/item.rs:240 | no recording found by the row tag | implemented not checked |
| W40 | Every explosion of kind 2 shakes the screen with strength 3; the bomb trail is pool C class 7 every  | D2b block | dino-sound/src/driver.rs:448 | no recording found by the row tag | implemented not checked |

## Design block D2c

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| WM01 | The 15 weapon types and names: 0 RIFLE, 1 GUN, 2 KNIFE, 3 UZI, 4 M-16A1, 5 SHOT GUN, 6 BAZOOKA, 7 RO | D2c block | dino-game/src/game/item/weapon.rs:31 | no recording found by the row tag | implemented not checked |
| WM02 | HUD weapon display: guns show a picture tile, the ammo number (red at 1), the name and a cartridge b | D2c block | dino-game/src/game/hud.rs:823 | no recording found by the row tag | implemented not checked |
| WM03 | Charge swing and critical hit: kind 0x203 weapons wait while attack is held (counter to 60); damage  | D2c block | dino-game/src/game/player_weapon.rs:444 | no recording found by the row tag | implemented not checked |
| WM04 | Knife and stick swing only when an enemy is in the reach zone (112 / 128 / 120 / 104 px, depth +-16  | D2c block | dino-extract/src/enemy.rs:334 | no recording found by the row tag | implemented not checked |
| WM05 | Rod breaks into the stick: roll per hit 19, 38, 69, 88, 81, 6, 12, 94 percent (table 0x2CBA0); stick | D2c block | dino-game/src/game/item/weapon.rs:858 | no recording found by the row tag | implemented not checked |
| WM06 | Wear and break: +0x20 per use (club 0x30); a weapon breaks only when dropped by a hit or death and l | D2c block | dino-extract/src/enemy.rs:326 | no recording found by the row tag | implemented not checked |
| WM07 | Drop rules by hero: hit drops all except Hannah keeps the 5 guns and Mess keeps the bazooka; death d | D2c block | dino-game/src/game/enemy_combat.rs:732 | no recording found by the row tag | implemented not checked |
| WM08 | Guns: bullet speeds 8 px (pistol, rifle) and 12 px (Uzi, M-16), pellets 10.5 to 13 px with vertical  | D2c block | dino-extract/src/enemy.rs:332 | no recording found by the row tag | implemented not checked |
| WM09 | Bazooka: recoil by hero (Jack 8, Mustapha 10, Hannah 12, Mess 6 px per frame start speed), shell 40  | D2c block | dino-assets/src/player.rs:63 | no recording found by the row tag | implemented not checked |
| WM10 | Dynamite lights itself when it comes to rest (4 stages of 15 frames, explodes after 60 frames: 114 t | D2c block | dino-game/src/game/item/weapon.rs:261 | no recording found by the row tag | implemented not checked |
| WM11 | Throw tables: flat 9.5 px per frame (12 px from a run) for knife, stick, empty guns, stones; lob for | D2c block | dino-extract/src/enemy.rs:331 | m600_h15_h_barrel_hero (equal), m600_h15_h_barrel_poacher (equal), m600_h16_h_barrel_poacher (equal) | equal |
| WM12 | Pick up: lowest kind wins (ammo box before weapons, food before the box); weapons 40 frame lock; two | D2c block | dino-game/src/game/item.rs:32 | no recording found by the row tag | implemented not checked |
| WM13 | Torch: swing 0x38 with the burn knock down (reaction 0xA), hit at +12, no damage over time on a thug | D2c block | dino-game/src/game/item.rs:16 | m700_a224_e5a1_torch_1p (equal), m700_a224_e5a1_torch_2p (equal), m700_a224_e5a1_torch_3p (equal) | equal |
| WM14 | Weapon attack records 0x1C to 0x60 with damage, stun, reaction, hit stop, sound, score (table in 7.1 | D2c block | dino-assets/src/sprites.rs:172 | no recording found by the row tag | implemented not checked |
| WM15 | Run and attack with a club, gun, rod, torch or butcher knife is the normal dash attack and keeps the | D2c block | dino-game/src/game/player_weapon.rs:734 | no recording found by the row tag | implemented not checked |
| WM16 | Friendly fire by fists measured with 3 and 2 players and 0 to 4 enemies: windows -1..1, 0..1, 0, off | D2c block | dino-game/src/game/player_combat.rs:172 | no recording found by the row tag | implemented not checked |
| WM17 | Friendly fire by attack objects (pass 0x10DCC): always on, no enemy gate, 1 HP per hit at exactly th | D2c block | dino-game/src/game/enemy_combat.rs:25 | no recording found by the row tag | implemented not checked |
| WM18 | Tag team trigger: counters per other player, reset by the initiator's input and by the partner's inv | D2c block | - | no address, class or name hit (addresses tried: 6728) | missing |
| WM19 | Jack's partner run attack is id 0x1A (the partner hero's own row) only with a direction held; withou | D2c block | dino-game/src/game/body.rs:30 | no recording found by the row tag | implemented not checked |
| WM20 | Dive window by hero (frames after the jump press): Jack 22 to 24, Mustapha 21 to 25, Hannah 22 to 25 | D2c block | dino-game/src/game/player.rs:2273 | no recording found by the row tag | implemented not checked |
| WM21 | Players never block or push each other (no contact in walk, run, jump); update order P1, P2, P3; dra | D2c block | dino-game/src/game/flow.rs:602 | no recording found by the row tag | implemented not checked |
| WM22 | `G.$4EF` recount: by 0x655E at area start, respawn, death end (61 frames after the death sequence st | D2c block | dino-game/src/game/flow.rs:662 | no recording found by the row tag | implemented not checked |
| WM23 | Script records for 2 and 3 players per episode, wave thresholds, boss HP by count (Vice 350 / 550 /  | D2c block | dino-assets/src/enemy.rs:126 | m800_a13_e6a2_boss_intro (equal) | equal |
| WM24 | Level column by count and the rank clamps (rank 0: level 0 / 4 / 8; rank 12: 10 / 12 / 14); rank spe | D2c block | dino-game/src/game/rank.rs:126 | no recording found by the row tag | implemented not checked |
| WM25 | Weapon sources: placed weapons only for torches (Episode 5), stones (Episodes 4, 7), rifle, shotgun, | D2c block | dino-game/src/game/enemy/class52.rs:1 | whole run ep5_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| WM26 | Enemy thrown dynamite and grenade: created lit with owner 0xFF (blast kind 3, attack 0x6A, record da | D2c block | dino-game/src/game/item/weapon.rs:126 | no recording found by the row tag | implemented not checked |
| WM27 | Thrown or lifted containers: +500 at the throw, +1000 per hit, no break bonus after a hit; a tire st | D2c block | dino-game/src/game/item/container.rs:8 | m600_h15_h_barrel_hero (equal), m600_h15_h_barrel_poacher (equal), m600_h16_h_barrel_poacher (equal) | equal |

## Design block D3a

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| HU01 | The only thing that wakes a sleeping dinosaur is a dinosaur hunter (class 79): its third swing pokes | D3a block | dino-game/src/game/enemy/class79.rs:357 | m600_h3_nat2_hunt (equal) | equal |
| HU02 | A hunter is freed in its first frame when no sleeping dinosaur (class 24, 29, 38 with `+$C0` negativ | D3a block | dino-game/src/game/enemy/class79.rs:67 | m600_h2_hun_none (equal) | equal |
| HU03 | After the poke the hunter walks left (facing left) until 0xC0 px left of the camera and is freed (no | D3a block | dino-game/src/game/enemy/class79.rs:192 | m600_h1_hun_v400 (equal), m600_h1_hun_v400_replay (equal) | equal |
| HU04 | Poacher bullets, knives, kicks and punches never hurt enemies or dinosaurs (the bullet flies through | D3a block | dino-game/src/game/enemy_combat.rs:268 | m600_h5_bsp_ctrl (equal), m600_h5_bsp_ferris (equal), m600_h5_bsp_hopper (equal), m600_h5_bsp_lvl31 (equal) | equal |
| HU05 | Poacher rifle: class 59 bullet var 0, 7 px per frame, attack 0x0B (15, knock down), muzzle flash cla | D3a block | dino-extract/src/enemy.rs:245 | m600_h6_rf_m0 (equal), m600_h6_rf_m0_l31 (equal), m600_h6_rf_m0_replay (equal), m600_h7_rf_m1 (equal) | equal |
| HU06 | Poacher dash tackle (attack 0x2C, 15, knock down B, 55 frames at 3.5 px per frame) and kick (0x0A);  | D3a block | dino-game/src/game/enemy/class13_attack.rs:31 | no recording found by the row tag | implemented not checked |
| HU07 | Poacher entrances 0 to 0x12 (stand 1200 frames, rise with fade 0x8B, jump in from above, run in with | D3a block | dino-game/src/game/enemy/class13.rs:6 | m600_h9_p13_ent02 (equal), m600_h9_p13_ent04 (equal), m600_h9_p13_ent06 (equal), m600_h9_p13_ent08 (equal) | equal |
| HU08 | Enemy bullets and thrown blades and the mace read a stale `+$84` for the damage scale (27 at level 3 | D3a block | dino-game/src/game/enemy/class59.rs:1 | m600_h5_bsp_lvl31 (equal) | equal |
| HU09 | Bludge eats floor items (lowest kind first, claim `+$AF`, queue of 8), spits all of them on death an | D3a block | dino-game/src/game/enemy/class6_eat.rs:3 | no recording found by the row tag | implemented not checked |
| HU10 | Bludge: entrance 0 hops, 4 crouches, 6 leaps 217 px; attacks charge 0x25, lunge 0x24, tongue 0x26, a | D3a block | dino-sound/src/driver.rs:448 | m600_h17_bl_vomit (equal), m700_h17_bl_vomit (equal), m700_h17_bl_vomit_replay (equal) | equal |
| HU11 | Lash T.: three objects, mace thrown from 160 px (attack 0x2A flight, 12 at level 0, 21 at 31), HP 10 | D3a block | dino-game/src/game/enemy/class3.rs:1 | m600_h19_lash_v4_l0 (equal), m600_h19_lash_v4_l0_replay (equal), m600_h19_lash_v4_l31 (equal) | equal |
| HU12 | Alternate Blade (class 63): dash 0x1C, one thrown blade (0x19, 12), run off the near side, freed off | D3a block | dino-game/src/game/enemy/class63.rs:1 | m600_h20_a63_knife (equal), m600_h20_a63_knife_replay (equal) | equal |
| HU13 | Bomb carriers (class 64 with class 77): HP 20 fixed, hop and run, throw dynamite (pool E explosion k | D3a block | dino-game/src/game/enemy/class64.rs:1 | m600_h10_ks_a64v2 (equal) | equal |
| HU14 | Barrels (65), rocks (66) and bikers (70) are monsters: they hurt heroes and enemies (10 raw to enemi | D3a block | dino-game/src/game/enemy/class65.rs:1 | m600_h11_bar_v0 (equal), m600_h11_bar_v0_replay (equal), m600_h11_bar_v1 (equal), m600_h11_bar_v201 (equal) | equal |
| HU15 | Brontosaurus foot and barrels hurt poachers in their way (foot 30 knock down and 1 touch, barrel 10) | D3a block | dino-game/src/game/placeholder.rs:77 | m600_h14_h_foot_poacher (first diff 6) | listed difference |
| HU16 | Pass 0x10DCC: attack list objects hurt heroes: 1 HP friendly fire (same Z exactly), full record dama | D3a block | dino-game/src/game/enemy_combat.rs:25 | no recording found by the row tag | implemented not checked |
| HU17 | Pass 0x10CD0: heroes break the six pool B breakable classes (a hit always breaks) | D3a block | dino-game/src/game/prop.rs:7 | no recording found by the row tag | implemented not checked |
| HU18 | Class 16 and 17: the Cadillac of Episodes 2 and 4 carries up to three heroes (`G.$504`), brakes from | D3a block | dino-extract/src/sprites.rs:109 | m600_h23_car_trace (equal) | equal |
| HU19 | Classes 27 and 43: the Episode 4 opening actors: three crouching men and the orange leader with thre | D3a block | dino-extract/src/sprites.rs:114 | m600_h22_e4_open (equal), m700_a207_e4_opening (equal) | equal |
| HU20 | Class 35 becomes class 36 (Morgan) by rewriting its class word; it fires class 59 variant 5 bullets  | D3a block | dino-game/src/game/enemy/class35.rs:156 | no recording found by the row tag | implemented not checked |
| HU21 | Walther is class 18 variant 2: Vice's AI without the intro, normal HP table, kill-all, clamp, score  | D3a block | dino-game/src/game/enemy/class18.rs:1 | no recording found by the row tag | implemented not checked |
| HU22 | Poacher gate quirk: the far decision reads the X class `+$A5` of the previous decision; the dash nee | D3a block | dino-extract/src/enemy.rs:242 | no recording found by the row tag | implemented not checked |
| HU23 | Class 63, 64, 3, 13, 79 count in waves (`+$7D = 0`), classes 27, 43, 65, 66 do not; bikers do | D3a block | dino-game/src/game/boss.rs:1 | no recording found by the row tag | implemented not checked |
| HU24 | Episode 7 library: no spawner in shelves; the three Bludges crouch and leap in from the right edge,  | D3a block | dino-game/src/game/flow.rs:86 | no recording found by the row tag | implemented not checked |
| HU25 | No thug (Punk, Thug, Ferris, Gneiss, Driver, Hammer, Elmer, Wrench, poachers, Blades, alternate Blad | D3a block | dino-game/src/game/enemy/class71.rs:1047 | m600_h25_pk_a63 (equal), m600_h25_pk_a64 (equal), m600_h25_pk_blade (equal), m600_h25_pk_driver (equal) | equal |
| HU26 | The alternate Punk and Wrench (class 64 with class 77) hold a dynamite stick: a knock down A (`+$7B` | D3a block | dino-game/src/game/enemy/class77.rs:75 | m600_h27_pk_a64w (equal), m600_h27_pk_a64w_replay (equal) | equal |

## Design block D3b

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| DS01 | Monsters (A22, A24, A29, A38, A82) live in the hostile list 0x6B64; they hurt heroes (scaled by type | D3b block | dino-game/src/game/enemy_combat.rs:105 | m600_a11_m_vs_thug (equal) | equal |
| DS02 | Dinosaurs give no kill score, no drop, are not counted by waves (`+7D = 0xFF`), are not touched by k | D3b block | dino-game/src/game/placeholder.rs:10 | m700_a7_z_kill (equal) | equal |
| DS03 | A sleeping Shivat has `+0 = 2` and cannot be hit by anyone; a dinosaur hunter (A79) wakes it with it | D3b block | dino-game/src/game/enemy.rs:385 | no recording found by the row tag | implemented not checked |
| DS04 | Hostile model names: MICHI, KIMAMA, MATOWARI, OKORI, KYOURAN, MANZOKU, COLOR, DIFFENCE; COLOR flash  | D3b block | dino-game/src/game/enemy_combat.rs:91 | m700_a13_h3_hit (equal), m700_a13_h3_hit_replay (equal) | equal |
| DS05 | Neutral Hopper: after 600 frames in MATOWARI a 1 in 16 roll every 10 frames turns it angry, after 12 | D3b block | dino-game/src/game/enemy/class24.rs:7 | m700_a12_h3_neutral (equal), m700_a12_h3_neutral_replay (equal) | equal |
| DS06 | Zeke: HP 20, damage x2 (type factor 64); one punch or jump kick kills; patrol heights 74 to 203 (ove | D3b block | dino-assets/src/player.rs:65 | m700_a6_z_hit_jkick_0 (equal), m700_a6_z_hit_jkick_120 (equal), m700_a6_z_hit_jkick_24 (equal), m700_a6_z_hit_jkick_48 (equal) | equal |
| DS07 | Zeke attacks: swoop 0x5B, triple peck 0x5C, dive grab 0x3B (hold 31 frames, 18 damage); attack roll  | D3b block | dino-extract/src/enemy.rs:432 | m700_a3_z_force_a (equal), m700_a3_z_force_b (equal), m700_a3_z_force_c (equal), m700_a3_z_force_c_replay (equal) | equal |
| DS08 | Zeke's grab only works on a grounded hero in normal control; a jump at the right moment dodges it; t | D3b block | dino-game/src/game/enemy/class82.rs:1 | m700_a4_z_force_c_jump (equal) | equal |
| DS09 | Zeke leaves straight up when `camX >= +5C` (checked only in patrol), dies by flying away, never expl | D3b block | dino-game/src/game/enemy/class82.rs:1 | m700_a8_z_leave (equal), m700_a8_z_leave_replay (equal) | equal |
| DS10 | Mack: charger with strides (32 frames moving, 22 still, 122 px), horns hurt only in the middle of a  | D3b block | dino-game/src/game/enemy/class38.rs:1 | m600_a9_m_solo_l0 (equal), m600_a9_m_solo_l0_replay (equal) | equal |
| DS11 | Rock Hopper is spawned only by Vice's init (variant 0, partner intro synchronised with Vice's `$6`); | D3b block | dino-game/src/game/enemy/class18.rs:1113 | no recording found by the row tag | implemented not checked |
| DS12 | Brontosaurus is one huge foot (A22) with a leg helper (A23): hover 180 frames over the nearest hero  | D3b block | dino-extract/src/sprites.rs:54 | m600_a20_bronto_0 (first diff 3) | listed difference |
| DS13 | Unused creature art: A12 = opening movie raptors, A28 / A37 / A39 / A44 / A47 = test mode viewer spr | D3b block | dino-game/src/game/enemy/class5.rs:1 | no recording found by the row tag | implemented not checked |
| DS14 | Hopper and Shivat burn when hit by a burn reaction (record reaction 0xA or 0xC): flames and palette  | D3b block | dino-game/src/game/enemy_combat.rs:201 | no recording found by the row tag | implemented not checked |

## Design block D3c

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| BS01 | The four bosses share only the HP row, the reinforcement controller (pool B 22) and the death chain  | D3c block | dino-game/src/game/boss.rs:46 | no recording found by the row tag | implemented not checked |
| BS02 | Boss HP by level and players (Vice, Slice 350..600, 500..700, 600..750; Butcher 512..640 / 576..704  | D3c block | dino-assets/src/enemy.rs:126 | no recording found by the row tag | implemented not checked |
| BS03 | Vice's intro: waits for the camera at X 0x1D0, talk box (sound 0x20), 40 frames, camera mover, anim  | D3c block | dino-game/src/game/boss.rs:29 | m600_bs_e1_v1_nat (first diff 5814), m600_bs_v1_v1_nat (first diff 5814) | listed difference |
| BS04 | Vice: 13 combos by a 32 cell table, attack gate T[level] (6 .. 31 of 32) every 60 frames, recovery p | D3c block | dino-extract/src/boss.rs:26 | no recording found by the row tag | implemented not checked |
| BS05 | Vice's summon: after a knockdown at HP <= max / `+$C2` (2, 3, 4): jump back, revolver fired into the | D3c block | dino-game/src/game/enemy/class18.rs:978 | m600_bs_v2_v9_alt (first diff 4092) | listed difference |
| BS06 | Vice's rage: HP <= 50 (first charge) or <= 25 (second), `+$AA = 1800`, only evaluated when state 0 d | D3c block | dino-game/src/game/enemy/class18.rs:918 | no recording found by the row tag | implemented not checked |
| BS07 | The bullet (class 59 var 2): speed 7, height 70, 0x0B (15 base, knock down), shared with the poacher | D3c block | dino-game/src/game/enemy/class59.rs:1 | m600_bs_v3_v5_face (equal) | equal |
| BS08 | The Butcher is class 5 (chopping a dead dinosaur with two blades, A51 carcass, talk box, camera move | D3c block | dino-extract/src/boss.rs:135 | m600_bs_b1_b10_nat2 (equal) | equal |
| BS09 | The Butcher's moves: walk 2 px, slash combo 0x2D / 0x1B (reach 108), far slash 0x2F / 0x30 (reach 14 | D3c block | dino-game/src/game/boss.rs:46 | no recording found by the row tag | implemented not checked |
| BS10 | Knife drop: while he holds a knife, a knockdown reaction (4 or 6) drops one with probability 8 / 16, | D3c block | dino-game/src/game/enemy/class61.rs:319 | m600_bs_b2_b6_knife (equal) | equal |
| BS11 | The Butcher targets the lowest HP hero 10 of 16 re-targets (HP below 112) | D3c block | dino-game/src/game/enemy/class61.rs:155 | no recording found by the row tag | implemented not checked |
| BS12 | Hogg is a rider (class 31) on a bike (class 32, the moving body); foot behaviour: ride across at 5.5 | D3c block | dino-game/src/game/enemy/class31.rs:340 | m600_bs_h1_h1_foot (first diff 4395), m600_bs_h1_h1_foot_replay (first diff 3995) | listed difference |
| BS13 | Hogg's bomb throws: roll every 60 frames (30 in car mode) `rng & 31 <= T[level]` (16 .. 29), 1 to 4  | D3c block | dino-extract/src/boss.rs:60 | no recording found by the row tag | implemented not checked |
| BS14 | Hogg's bomb (class 42): a lob to a 3 x 3 grid around the car (+-64 X, +-16 Z; weights 1 2 1 / 2 3 3  | D3c block | dino-game/src/game/enemy/class42.rs:70 | no recording found by the row tag | implemented not checked |
| BS15 | Hogg in car mode: a random walk on a 32 x 8 cell grid around the car (re-drawn every 25 frames), bik | D3c block | dino-game/src/game/enemy/class31.rs:751 | no recording found by the row tag | implemented not checked |
| BS16 | Hogg's death: the rider is thrown (vy 5 px up, vx 1), the bike breaks into pieces and explodes where | D3c block | dino-extract/src/boss.rs:61 | m600_bs_e1_h4_foot (first diff 4350), m600_bs_h1_h4_foot (first diff 4350) | listed difference |
| BS17 | Radio: treasure pick up in Episode 3 with `G.$50E = 0` calls 0x8BD24 (3 free pool B slots); sources: | D3c block | dino-game/src/game/car.rs:1 | whole run ep3_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| BS18 | Car arrival timeline (t after the pick up at 5): car at 16 drives in at 5 px, brakes 74 to 100, wait | D3c block | dino-game/src/game/car.rs:135 | no recording found by the row tag | implemented not checked |
| BS19 | Car steering: every rider with a stick adds 0x800 + (+-768 vx, +-256 vz, diagonals +-256, +-192) to  | D3c block | dino-game/src/game/car.rs:409 | no recording found by the row tag | implemented not checked |
| BS20 | Car versus the world: flat 20 to every enemy and monster it touches (credit to the taker), container | D3c block | dino-game/src/game/spawn.rs:34 | no recording found by the row tag | implemented not checked |
| BS21 | Car end: HP 0, or camera X >= 0xA000, or the boss death script; heroes put at camera X + 0x80 / 0xA0 | D3c block | dino-game/src/game/car.rs:90 | no recording found by the row tag | implemented not checked |
| BS22 | Car seats: owner (radio taker) first, other players in player order; the delivering NPC is the first | D3c block | dino-game/src/game/car.rs:169 | no recording found by the row tag | implemented not checked |
| BS23 | The radio works during the Hogg fight (Hogg switches to car behaviour) and a second radio gives a se | D3c block | dino-game/src/game/car.rs:1 | no recording found by the row tag | implemented not checked |
| BS24 | Slice: kick 0x36, long slash 0x37 (reach 138), single boomerang (class 19, var 0 / 2 / 4, 7 base), f | D3c block | dino-game/src/game/enemy/class25.rs:208 | no recording found by the row tag | implemented not checked |
| BS25 | Afterimage: a pool C object class 27 every 7 frames while running, copies the frame and facing, 6 pa | D3c block | dino-game/src/game/effect/fx27.rs:1 | m700_bs_s2_s4_face (equal), m700_bs_s2_s4_face_replay (first diff 3530) | equal |
| BS26 | Boomerangs can be hit by heroes (monster list) and fall; the thrower catches them on return | D3c block | dino-game/src/game/enemy/class25_ai.rs:563 | m700_bs_s1_s8_alt (equal) | equal |
| BS27 | Boss reinforcement lists by episode: group order, `G.$502` setters (Vice summon, Slice and Hogg 300  | D3c block | dino-assets/src/boss.rs:5 | no recording found by the row tag | implemented not checked |

## Design block D3d

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| BT01 | Slisaur starter (pool B 33): 2 Slisaurs, 3 when `G.$4EF != 0`; the last one's hit calls `0xACEE`; th | D3d block | dino-game/src/game/prop/b33.rs:1 | m800_bta6_s7_1p_a (first diff 253) | listed difference |
| BT02 | Slisaur class 71: gate table 0x5DB0E, combo table 0x5DB8E, dash ramp 0x5DD1C, attacks 0x36 and 0x37, | D3d block | dino-extract/src/boss.rs:156 | m800_bta6_sl_dmg_1p (first diff 253) | listed difference |
| BT03 | Slisaur blades (class 19, shared with Slice) and the afterimage (pool C 27, 6 steps of 6 frames, pal | D3d block | dino-game/src/game/effect/fx27.rs:1 | m800_bta6_sl_fine (first diff 253) | listed difference |
| BT04 | Lab tank breakers (pool B 44): four records (cylinders 01, 03, 04, 11), 3 frame event, shards, tile  | D3d block | dino-game/src/game/prop/b44.rs:1 | m800_bta7_k8_walk (equal) | equal |
| BT05 | Episode 5 scene: class 35 walks, shoots the villager (class 45) with four class 59 var 5 bullets, re | D3d block | dino-game/src/game/enemy/class35.rs:1 | m700_bt_a1_m5_scene (equal), m700_bt_a1_m5_scene_replay (equal) | equal |
| BT06 | Morgan (class 36): gate 0x73B14, actions 0x73B54 and 0x73C54, kite, hop, throws (class 68 var 2, 3,  | D3d block | dino-game/src/game/enemy/class36.rs:1 | m700_bt_a2_m5_dmg (equal) | equal |
| BT07 | Morgue (class 40): moves 0x40, 0x6C, 0x4B, 0x41 (charge 7 px per frame), 0x42, 0x2B, 0x31, 0x4A; the | D3d block | dino-game/src/game/enemy/class40.rs:1 | m700_bt_a4_m5_b (first diff 2380) | listed difference |
| BT08 | Tyrog crate scene: pool B 79, class 75 var 0 (320 frames), 1, 4, class 76 jump and puffs, class 4 va | D3d block | dino-game/src/game/prop/b79.rs:1 | m800_bta8_ty_full_a (equal) | equal |
| BT09 | Tyrog (class 4): three forms by `+26` high byte, HP index 6 halved at every init, music only for var | D3d block | dino-extract/src/boss.rs:135 | m800_bta9_ty_full_c (first diff 7066) | listed difference |
| BT10 | Tyrog chest blades (class 74, attached child, 20 HP, low box, one hit tick) and fire (class 26 var 3 | D3d block | dino-extract/src/boss.rs:141 | m800_bta10_ty4_v2_dmg (equal) | equal |
| BT11 | Tyrog host (class 72): 30 frame intro, punch, jump, caption 0x5A, transformation anim 0x22 or 0x23,  | D3d block | dino-extract/src/boss.rs:156 | m800_bta12_ty_host (equal) | equal |
| BT12 | Tyrog chain by class word rewrites 4 var 2, 72 var 1, 4 var 1, 72 var 0, 4 var 0; forms 2 and 1 give | D3d block | dino-game/src/game/player.rs:3278 | no recording found by the row tag | implemented not checked |
| BT13 | Tyrog 2 (class 91): fight 3 tables, 300 HP at every level, counted by the census, 1,000 points, no ` | D3d block | dino-game/src/game/enemy/class4.rs:131 | m800_bta10_ty91_dmg (equal) | equal |
| BT14 | Fessenden form 1 (class 94): human intro 276 frames then music 0x0F, Morgue moves with gate 0x74A42  | D3d block | dino-extract/src/boss.rs:145 | m800_bta13_f94_intro (equal) | equal |
| BT15 | Fessenden death scene: 232 frame fall, five captions (0x3D, 0x3E, 0x40, 0x41, 0x42), growth anims 0x | D3d block | dino-game/src/game/enemy/class94.rs:4 | m800_bta14_f94_trans (equal) | equal |
| BT16 | Fessenden form 2 (class 84 + parts 85 to 89): five parts from the sorted slots, parts mirror the bod | D3d block | dino-extract/src/boss.rs:151 | m800_bta15_f84_trace (equal) | equal |
| BT17 | Class 84 counter reaction: against hero moves 0x14, 0x02, 0x13, 0x15 with HP fraction masks (12, 15, | D3d block | dino-game/src/game/enemy/class84.rs:6 | no recording found by the row tag | implemented not checked |
| BT18 | Class 84 heads: bite 0x5E, fire volleys with class 95 variants 0 to 8 chosen by the target distance, | D3d block | dino-game/src/game/enemy/class85_89.rs:6 | no recording found by the row tag | implemented not checked |
| BT19 | Class 84 poise: poise 10 reset at each hit, guard counter `+BD` (2), only hits in phases 4, 6, 10, 1 | D3d block | dino-game/src/game/enemy/class84.rs:222 | m800_bta15_f84_rapid (equal) | equal |
| BT20 | Last boss end: `0xACEE` at HP 0 (stage index 7 sets `G.$4D7`, `G.$4DB`), 1,000 points at +368 frames | D3d block | dino-game/src/game/boss_common.rs:305 | m800_a31_ee_f84_kill (first diff 3732), m800_bta17_f84_kill (first diff 3732) | listed difference |
| BT21 | Boss HP tables by band and player count for index 0, 4, 5, 6, 8, 9 (and Tyrog halved), HUD scale 100 | D3d block | dino-assets/src/enemy.rs:126 | no recording found by the row tag | implemented not checked |
| BT22 | Boss target rule: fewest attackers, ties to the later player (the supplement's rearmost) | D3d block | dino-game/src/game/placeholder.rs:202 | no recording found by the row tag | implemented not checked |

## Design block D4a

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| IT01 | Only 17 foods exist; id 18 reads the first 4 bytes of the next table (heal 36, bonus 4,000, no pictu | D4a block | dino-extract/src/enemy.rs:298 | no recording found by the row tag | implemented not checked |
| IT02 | Food values: heal 100, 80, 64, 64, 48, 48, 48, 32 x 5, 16 x 5; full-HP bonus 10,000 (1 to 4), 5,000  | D4a block | dino-extract/src/enemy.rs:298 | no recording found by the row tag | implemented not checked |
| IT03 | 16 treasures 10,000 (gold, diamond, cup, egg), 5,000 (gold dust, dino bone, ammonite, pearl), 3,000  | D4a block | dino-extract/src/enemy.rs:299 | no recording found by the row tag | implemented not checked |
| IT04 | Food heals at press + 1, treasure scores at press + 11 (end of the 10 frame lift); the HUD message f | D4a block | dino-game/src/game/trace.rs:1154 | no recording found by the row tag | implemented not checked |
| IT05 | Containers: 7 variants have code (pail, drum, barrel, stone, boulder, statue, tires), 13 numbers; th | D4a block | dino-extract/src/sprites.rs:116 | no recording found by the row tag | implemented not checked |
| IT06 | One hit of any hero breaks a normal container (HP 4): +100 for the hit and +500 at the break (item m | D4a block | dino-game/src/game/enemy_combat.rs:335 | no recording found by the row tag | implemented not checked |
| IT07 | What breaks a container: hero attack frames, bullets, thrown knives, thrown containers, explosions ( | D4a block | dino-game/src/game/enemy_combat.rs:333 | no recording found by the row tag | implemented not checked |
| IT08 | Tire stacks and statues cannot be lifted (9 and 3 offsets tried); a barrel is lifted only from 24 px | D4a block | - | no address, class or name hit (addresses tried: none) | missing |
| IT09 | Explosive drum or barrel: HP 50 (jab chain 10, 10, 15, 15), shimmer every 51 frames, fuse of 4 stage | D4a block | dino-game/src/game/item/tests.rs:243 | no recording found by the row tag | implemented not checked |
| IT10 | Break effects: pail body flies and blinks 40 frames; drum 2 pieces; barrel 10 puff objects; stone an | D4a block | dino-game/src/game/item/container.rs:10 | no recording found by the row tag | implemented not checked |
| IT11 | Stones from debris (stone, boulder, falling rock): each of 9 pieces can become a stone with chances  | D4a block | dino-game/src/game/item.rs:437 | no recording found by the row tag | implemented not checked |
| IT12 | Drop tables: containers, rolling barrels (class 65) and rocks (class 66) read the container table 0x | D4a block | dino-game/src/game/item.rs:56 | no recording found by the row tag | implemented not checked |
| IT13 | Drop code 0x0F: an ammo box when any gun item (rifle, gun, Uzi, M-16, shotgun, bazooka; held or lyin | D4a block | dino-extract/src/enemy.rs:325 | m600_h5_bsp_ctrl (equal), m600_h5_bsp_ferris (equal), m600_h5_bsp_hopper (equal), m600_h5_bsp_lvl31 (equal) | equal |
| IT14 | A dropped item hops out (48 px up, 3 px per frame, lands after about 48 frames; cannot be taken befo | D4a block | dino-game/src/game/item/tests.rs:182 | no recording found by the row tag | implemented not checked |
| IT15 | A full pool E (24 slots) loses the drop | D4a block | dino-game/src/game/boss.rs:28 | no recording found by the row tag | implemented not checked |
| IT16 | Despawn windows: food, treasure, containers, enemies in `[camera X - 0xC0, + 0x240)`; weapons, ammo  | D4a block | dino-game/src/game/enemy.rs:705 | no recording found by the row tag | implemented not checked |
| IT17 | Nothing on the floor times out (food, treasure, ammo, placed or enemy dropped or container dropped w | D4a block | dino-game/src/game/boss.rs:29 | no recording found by the row tag | implemented not checked |
| IT18 | A dynamite put down by a weapon swap is lit when it lands (4 stages of 15 frames); the fuse pauses i | D4a block | dino-game/src/game/enemy.rs:104 | no recording found by the row tag | implemented not checked |
| IT19 | Placed items: 30 on the floor in 5 episodes; Episode 2's pearl record (X 0x1A8 at trigger 0x428) is | D4a block | dino-game/src/game/flow.rs:981 | whole run ep2_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| IT20 | Episode 3 area script: 12 radios (4 containers, 8 enemies), 17 empty containers, no food or weapon; | D4a block | dino-game/src/game/flow.rs:86 | whole run ep3_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| IT21 | 42 item or drop records exist only for 2 or 3 players: rod and club (Episode 1), shotgun and M-16 (E | D4a block | dino-game/src/game/terrain.rs:638 | whole run ep1_3p equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| IT22 | 23 records change with the level: Uzi 31 of 32 or 8 of 32 (Episode 7 area 0), lobster or parfait, ca | D4a block | dino-game/src/game/boss.rs:245 | no recording found by the row tag | implemented not checked |
| IT23 | Boss helper lists of Episodes 1 to 4 carry drop rows (Vice steak and small foods, Butcher food rows, | D4a block | dino-game/src/game/boss.rs:1 | no recording found by the row tag | implemented not checked |
| IT24 | Every GameFAQs barrel and drop note checked agrees with the rows (section 10) | D4a block | dino-game/src/game/boss.rs:4 | no recording found by the row tag | implemented not checked |
| IT25 | Walther (class 18 var 0x200) drops a revolver 32 of 32 in Episodes 4 and 5 and the radio in Episode  | D4a block | dino-game/src/game/enemy/class18.rs:1 | no recording found by the row tag | implemented not checked |
| IT26 | The HUD item message: FooD icon with name and GOOD !! or points; Pts icon with name and points; a ra | D4a block | dino-game/src/game/terrain.rs:638 | no recording found by the row tag | implemented not checked |
| IT27 | Rolling barrels are freed 64 px left of the camera (near window), which is the "only if you go slowl | D4a block | dino-game/src/game/enemy/class65.rs:1 | no recording found by the row tag | implemented not checked |
| IT28 | The Episode 2 treasures are drawn behind the foreground trunks; the hero at depth 16 is half hidden | D4a block | dino-game/src/game/flow.rs:86 | whole run ep2_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| IT29 | **Port bug:** the extracted drop table ends at 0xB7312 (enemy row 0x2F); Episode 1 wave 4 uses enemy | D4a block | dino-game/src/game/boss.rs:1 | whole run ep1_3p equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| IT30 | The Episode 5 area 0 drum for 3 players (X 0x2D0, trigger 0x80) is past the far window when it fires | D4a block | dino-game/src/game/flow.rs:86 | whole run ep5_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |

## Design block D3e pool B

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| W27 | Steam pipes in the end wall of Episode 1 area 0: break with one punch (hero depth 296 to 320), then | D3e pool B block | dino-extract/src/enemy.rs:360 | whole run ep1_3p equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| W28 | Episode 1 end door of area 0 is smashed by the hero during the scripted walk (dust, black doorway) | D3e pool B block | dino-game/src/game/script.rs:317 | whole run ep1_3p equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| W29 | Hall double door opens at the start of wave 1; the hall window shatters when the scripted hero kicks | D3e pool B block | dino-extract/src/enemy.rs:355 | no recording found by the row tag | implemented not checked |
| W30 | Respawn air strike: five bombs fall in a line, white blast, explosions around the dropping hero, spe | D3e pool B block | dino-extract/src/enemy.rs:342 | no recording found by the row tag | implemented not checked |
| W31 | Breakable family of pool B: classes 0 (generic wall panel, 15 variants), 1 (chain-link fence), 3 (wa | D3e pool B block | dino-game/src/game/prop.rs:13 | no recording found by the row tag | implemented not checked |
| W32 | Episode 2 ambient life: small blue bird flocks (B72, A46 with A48 trail) harmless; ground critters ( | D3e pool B block | dino-game/src/game/flow.rs:86 | whole run ep2_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| W33 | Tile animation family of pool B: periodic scroll2, scroll3 or palette writers (23 classes), some key | D3e pool B block | dino-game/src/game/terrain.rs:18 | no recording found by the row tag | implemented not checked |
| W34 | Class 69 is a debug object (frees itself unless G.$4AA); class 66 has an anti tamper check (0x86B3C) | D3e pool B block | dino-game/src/game/prop/class66.rs:9 | no recording found by the row tag | implemented not checked |
| W35 | Class 95 blends 3 palettes toward a target set along the Episode 3 road (sky colours); classes 83, 8 | D3e pool B block | dino-game/src/game/boss.rs:1 | whole run ep3_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| W36 | Container variants of E class 5: 0 pail, 2 drum, 4 barrel, 6 stone, 8 boulder, 0xA armor statue (sol | D3e pool B block | dino-game/src/game/enemy/class5.rs:1 | no recording found by the row tag | implemented not checked |
| W37 | "STAGE CLEARED!" and digits 1 to 8 are frames of pool C class 9 (0x93184); 1P, 2P, 3P tags are B68;  | D3e pool B block | dino-game/src/game/effect.rs:612 | no recording found by the row tag | implemented not checked |
| W38 | Treasures of Episode 2 area 0 sit at depth 16, height 64, behind the big yellow trunks (E class 9 va | D3e pool B block | dino-sound/src/driver.rs:458 | whole run ep2_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |

## Design block D3e part 2

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| W201 | Episode 3 uses one continuous area with separate foot-lock and car paths | D3e part 2 block | dino-assets/src/stage.rs:75 | whole run ep3_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| W202 | B95 changes 16 colours in one palette per object, not three palettes | D3e part 2 block | dino-game/src/game/prop/b95.rs:45 | no recording found by the row tag | implemented not checked |
| W203 | Seven B95 triggers use periods 15,15,2,2,2,2,2 and three destination palette slots | D3e part 2 block | dino-extract/src/boss.rs:133 | no recording found by the row tag | implemented not checked |
| W204 | Radio sources include the first barrel, last road pail, middle drum and one rolling barrel | D3e part 2 block | dino-game/src/game/car.rs:1 | no recording found by the row tag | implemented not checked |
| W205 | Six Episode 3 tire stacks are E5 containers, not map decoration | D3e part 2 block | dino-game/src/game/flow.rs:86 | whole run ep3_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| W206 | A70 road riders and A65 hopping barrels hurt heroes and enemies | D3e part 2 block | dino-game/src/game/camera.rs:1337 | no recording found by the row tag | implemented not checked |
| W207 | Outer entrance crash breaks drums, tires and A27 men; upright gate/posts stay | D3e part 2 block | dino-extract/src/sprites.rs:109 | no recording found by the row tag | implemented not checked |
| W208 | Episode 4 opening car A16/A17 differs from playable car B88 | D3e part 2 block | dino-game/src/game/flow.rs:86 | whole run ep4_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| W209 | Walther runs in beside a closed grate at X 0x320,Y 0,Z 0x20 | D3e part 2 block | dino-game/src/game/placeholder.rs:26 | no recording found by the row tag | implemented not checked |
| W210 | Garage back tires have pizza/steak/sushi and gold-dust/dime rows | D3e part 2 block | dino-game/src/game/prop/b1.rs:1 | no recording found by the row tag | implemented not checked |
| W211 | Overturned Cadillac is garage scroll2 art, with no separate break or loot | D3e part 2 block | - | no address, class or name hit (addresses tried: none) | missing |
| W212 | B0 variants 1 and 2 are thin-stem fixtures with round tops; stems break | D3e part 2 block | dino-extract/src/enemy.rs:383 | no recording found by the row tag | implemented not checked |
| W213 | B0 variant 0 is a large cylindrical column whose upper body breaks off | D3e part 2 block | dino-game/src/game/prop/b0.rs:39 | no recording found by the row tag | implemented not checked |
| W214 | B1 garage gate makes a torn opening, composition 327 and 13x6 attribute patch | D3e part 2 block | dino-game/src/game/prop/b1.rs:60 | no recording found by the row tag | implemented not checked |
| W215 | B0/B1 breaks have no container drop call or hostile attack registration | D3e part 2 block | dino-extract/src/enemy.rs:383 | no recording found by the row tag | implemented not checked |
| W216 | B59 variants 0,1,2 animate foundry far-layer colours | D3e part 2 block | dino-extract/src/enemy.rs:386 | no recording found by the row tag | implemented not checked |
| W217 | Foundry weapons are placed guns with player-count and level gates | D3e part 2 block | dino-game/src/game/prop.rs:180 | no recording found by the row tag | implemented not checked |
| W218 | Episode 5 starts in daylight with three torches, two barbecues and an egg | D3e part 2 block | dino-game/src/game/flow.rs:86 | whole run ep5_jack equal in lagsync (M10 whole-run checks; causes S, K, P of allow/m10_runs.txt) | equal |
| W219 | Torch floor fire is E6 overlay; swung torch burn comes from weapon logic | D3e part 2 block | dino-game/src/game/trace.rs:595 | no recording found by the row tag | implemented not checked |
| W220 | Meadow diamond is a placed treasure at trigger 0x180, not a broken rock reward | D3e part 2 block | - | no address, class or name hit (addresses tried: AF7F4) | missing |
| W221 | Hunters wake the sleeping Shivat; kill-all sweep can prevent the scene | D3e part 2 block | dino-game/src/game/enemy.rs:39 | no recording found by the row tag | implemented not checked |
| W222 | B48 is a 128-call no-op, not the cliff rock spawner | D3e part 2 block | dino-game/src/game/prop/b48.rs:1 | no recording found by the row tag | implemented not checked |
| W223 | Six scripted/wave rock records use A66, with one gated to 2P+ | D3e part 2 block | dino-game/src/game/flow.rs:202 | no recording found by the row tag | implemented not checked |
| W224 | Falling-rock debris can become throwable stones after bouncing | D3e part 2 block | dino-game/src/game/item.rs:437 | no recording found by the row tag | implemented not checked |
| W225 | Stone limit is checked before debris finishes, so it is not a strict final cap | D3e part 2 block | dino-game/src/game/effect.rs:153 | no recording found by the row tag | implemented not checked |
| W226 | Bridge has eight barrels in two depth rows and stays intact when they break | D3e part 2 block | - | no address, class or name hit (addresses tried: AF974, AF9E4) | missing |
| W227 | Bridge route lowers camY 0x200 to 0x100 through diagonal mode, without fade | D3e part 2 block | - | no address, class or name hit (addresses tried: none) | missing |
| W228 | B5 is far-layer chimney smoke | D3e part 2 block | dino-extract/src/enemy.rs:116 | no recording found by the row tag | implemented not checked |
| W229 | B6 is far-layer fire behind village openings, not steam | D3e part 2 block | dino-extract/src/enemy.rs:120 | no recording found by the row tag | implemented not checked |
| W230 | B10 and B13 are repeating foreground fire patches keyed to global tile clock | D3e part 2 block | dino-game/src/game/prop/b10.rs:52 | no recording found by the row tag | implemented not checked |
| W231 | Eight B13 records include two different variants at X 0xE20 | D3e part 2 block | dino-game/src/game/prop/b13.rs:5 | no recording found by the row tag | implemented not checked |
| W232 | B62 cycles fire palettes and requests sound 0x8E | D3e part 2 block | dino-game/src/game/prop/b62.rs:1 | no recording found by the row tag | implemented not checked |
| W233 | Village scenery fire has no attack box or extinguishing interaction | D3e part 2 block | dino-game/src/game/prop/b62.rs:1 | no recording found by the row tag | implemented not checked |
| W234 | Water buckets are E5 pails; break gives loot and no water splash | D3e part 2 block | dino-game/src/game/flow.rs:602 | no recording found by the row tag | implemented not checked |
| W235 | Special bucket needs a live gun on the field to become ammo | D3e part 2 block | dino-extract/src/enemy.rs:325 | no recording found by the row tag | implemented not checked |
| W236 | Fossil bucket is ammonite 30/32 or dime 2/32; last bucket selects cake/coffee by level | D3e part 2 block | - | no address, class or name hit (addresses tried: none) | missing |
| W237 | B49 opens a wall/shutter patch automatically at wave 6 | D3e part 2 block | dino-game/src/game/prop/b49.rs:1 | no recording found by the row tag | implemented not checked |
| W238 | B49's running handler does not spawn the nearby two portrait-box makers | D3e part 2 block | dino-game/src/game/prop/b49.rs:17 | no recording found by the row tag | implemented not checked |
| W239 | B70 is decorative rifle art with cartridge marks and no pickup | D3e part 2 block | dino-extract/src/sprites.rs:104 | no recording found by the row tag | implemented not checked |
| W240 | Chief scene, portrait text, shots and A35-to-A36 change are separate from wall burst | D3e part 2 block | dino-game/src/game/prop/b70.rs:1 | no recording found by the row tag | implemented not checked |
| W241 | B89 makes desert scenery, B90 is chained caption logic, neither is a car wheel | D3e part 2 block | dino-game/src/game/prop/class89.rs:4 | no recording found by the row tag | implemented not checked |
| W242 | Camera jumps can omit earlier village patch initialization and leave dark columns | D3e part 2 block | dino-game/src/game/boss.rs:7 | no recording found by the row tag | implemented not checked |

## Design block D3e part 3

| row id | the mechanic | covering task | port file:line | evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| W301 | Episode 6 mine entrance wall: one hit breaks it (hero depth 32 or more, jab, jump kick, special), so | D3e part 3 block | dino-extract/src/enemy.rs:348 | no recording found by the row tag | implemented not checked |
| W302 | Doors and a bookshelf that burst open at the start of a wave group and let the enemies out: three do | D3e part 3 block | dino-game/src/game/prop.rs:194 | no recording found by the row tag | implemented not checked |
| W303 | Prop hit rule: a player attack hits a prop box when hero Z + 8 is at least the start of the box dept | D3e part 3 block | dino-game/src/game/prop.rs:7 | no recording found by the row tag | implemented not checked |
| W304 | Mine lamps: two scroll palettes (80, 81) cycle in 4 steps of 9 frames (36 frame cycle), Episode 6 ar | D3e part 3 block | dino-extract/src/enemy.rs:399 | no recording found by the row tag | implemented not checked |
| W305 | Storm of Episode 6 area 1: rain streaks (75 streaks, scroll1, every frame, to camera X 0x500), rain  | D3e part 3 block | dino-extract/src/enemy.rs:405 | no recording found by the row tag | implemented not checked |
| W306 | Brontosaurus foot shadow: a black oval on the floor under the foot at its X and Z, following the her | D3e part 3 block | dino-game/src/game/placeholder.rs:77 | no recording found by the row tag | implemented not checked |
| W307 | Rolling barrels are freed at camera X - 0x40 or + 0x200: they carry their item but a fast hero outru | D3e part 3 block | dino-game/src/game/enemy/class65.rs:1 | no recording found by the row tag | implemented not checked |
| W308 | Swamp scenery of Episode 6: water surfaces B8 and B39 (4 pictures of 24 frames), the eye of the huge | D3e part 3 block | dino-game/src/game/prop.rs:175 | no recording found by the row tag | implemented not checked |
| W309 | Tyrog scene: three gangsters carry a metal box, three captions, the box opens and Tyrog jumps out (B | D3e part 3 block | dino-game/src/game/enemy_combat.rs:322 | no recording found by the row tag | implemented not checked |
| W310 | Lift shaft: floor labels L15 to L1, B1 to B10, blank B11 to B20 (open harbour window), B21 to B40 pa | D3e part 3 block | dino-extract/src/enemy.rs:81 | no recording found by the row tag | implemented not checked |
| W311 | Lift riders: entrances from X -48 and 432 and from above (y 255, variants 0x20A); list in 2.5 | D3e part 3 block | dino-game/src/game/enemy_combat.rs:1199 | no recording found by the row tag | implemented not checked |
| W312 | Library gates: fence B1 var 4 at the start is broken by any player attack at any time, fence B1 var  | D3e part 3 block | dino-game/src/game/prop.rs:183 | no recording found by the row tag | implemented not checked |
| W313 | Lab of Episode 7: console blink lights B41 (periods 19, 7, 13), palette cycles B61 and B63 (17 frame | D3e part 3 block | dino-game/src/game/flow.rs:86 | no recording found by the row tag | implemented not checked |
| W314 | Sewer flame vents B45: HP 48, cycle idle 60 to 90, sparks 45, growth 40, full 40 to 70, shrink 40; f | D3e part 3 block | dino-game/src/game/prop/b45.rs:1 | no recording found by the row tag | implemented not checked |
| W315 | Wall pipes B40 of Episode 8: three sets in the sewers (a jab breaks two, the third needs a jump kick | D3e part 3 block | dino-game/src/game/terrain.rs:232 | no recording found by the row tag | implemented not checked |
| W316 | Tile clock G.$7AE = ((gf + 60) / 24 mod 4) * 4, copied to G.$7AF one frame later | D3e part 3 block | dino-game/src/game/terrain.rs:18 | no recording found by the row tag | implemented not checked |
| W317 | Sewer scenery: water bands B28 and B29 (rows 12 to 14), wall torches B43 (4 pictures of 8 frames), t | D3e part 3 block | dino-game/src/game/prop.rs:94 | no recording found by the row tag | implemented not checked |
| W318 | End of the Episode 8 lab: the leader lifts a floor grate (B35), throws it right and jumps into the m | D3e part 3 block | dino-game/src/game/flow.rs:86 | no recording found by the row tag | implemented not checked |
| W319 | Fire glow: B53 rewrites 27 palettes (370 colours) while a hero lies burning (G.$7B2 = 2), only in Ep | D3e part 3 block | dino-game/src/game/player.rs:3146 | no recording found by the row tag | implemented not checked |
| W320 | The meat barrel at X 0xCB0 can be lifted and carried through the sub area switch, the scripted walk  | D3e part 3 block | - | no address, class or name hit (addresses tried: none) | missing |
| W321 | Caption banners (B66 vars 0x60 to 0x67): COAL MANE, JUNGLE, CITY IN THE SEA, LIBRARY, COMPUTER ROOM, | D3e part 3 block | dino-game/src/game/prop/class66.rs:5 | no recording found by the row tag | implemented not checked |
| W322 | The Episode 7 background is the Episode 1 skyline art in an orange palette in front of a stone terra | D3e part 3 block | dino-game/src/game/flow.rs:86 | no recording found by the row tag | implemented not checked |
| W323 | After the final boss death the ending code makes B82, the showers B83 and B84 (200 and 400 frames),  | D3e part 3 block | dino-game/src/game/boss.rs:218 | no recording found by the row tag | implemented not checked |
| W324 | Probe note: `hold 0xFF850E 1 2` does not kill Lash T. or Walther; `jump ... at=start` does not stop  | D3e part 3 block | dino-game/src/game/flow.rs:251 | no recording found by the row tag | implemented not checked |

## Fix list (after the whole-run checks, 2026-10-21)

Remaining rows that are missing, a listed difference or not checked. Missing first, then listed differences, then not checked.

| # | row id | what is missing or different | verdict |
| --- | --- | --- | --- |
| 1 | IT08 | Tire stacks and statues cannot be lifted (9 and 3 offsets tried); a barrel is lifted only from 24 px | missing |
| 2 | W13 | Barrels rolling down the cave slope (Episode 8), rolling drums with items in the library (Episode 7) | missing |
| 3 | W20 | Suits of armor in the Episode 1 hall hide items | missing |
| 4 | W211 | Overturned Cadillac is garage scroll2 art, with no separate break or loot | missing |
| 5 | W220 | Meadow diamond is a placed treasure at trigger 0x180, not a broken rock reward | missing |
| 6 | W226 | Bridge has eight barrels in two depth rows and stays intact when they break | missing |
| 7 | W227 | Bridge route lowers camY 0x200 to 0x100 through diagonal mode, without fade | missing |
| 8 | W236 | Fossil bucket is ammonite 30/32 or dime 2/32; last bucket selects cake/coffee by level | missing |
| 9 | W26 | Catch-all: every other interactive thing in the levels (switches, doors, things that fall or roll wh | missing |
| 10 | W320 | The meat barrel at X 0xCB0 can be lifted and carried through the sub area switch, the scripted walk | missing |
| 11 | WM18 | Tag team trigger: counters per other player, reset by the initiator's input and by the partner's inv | missing |
| 12 | BS03 | Vice's intro: waits for the camera at X 0x1D0, talk box (sound 0x20), 40 frames, camera mover, anim | listed difference |
| 13 | BS05 | Vice's summon: after a knockdown at HP <= max / `+$C2` (2, 3, 4): jump back, revolver fired into the | listed difference |
| 14 | BS12 | Hogg is a rider (class 31) on a bike (class 32, the moving body); foot behaviour: ride across at 5.5 | listed difference |
| 15 | BS16 | Hogg's death: the rider is thrown (vy 5 px up, vx 1), the bike breaks into pieces and explodes where | listed difference |
| 16 | BT01 | Slisaur starter (pool B 33): 2 Slisaurs, 3 when `G.$4EF != 0`; the last one's hit calls `0xACEE`; th | listed difference |
| 17 | BT02 | Slisaur class 71: gate table 0x5DB0E, combo table 0x5DB8E, dash ramp 0x5DD1C, attacks 0x36 and 0x37, | listed difference |
| 18 | BT03 | Slisaur blades (class 19, shared with Slice) and the afterimage (pool C 27, 6 steps of 6 frames, pal | listed difference |
| 19 | BT07 | Morgue (class 40): moves 0x40, 0x6C, 0x4B, 0x41 (charge 7 px per frame), 0x42, 0x2B, 0x31, 0x4A; the | listed difference |
| 20 | BT09 | Tyrog (class 4): three forms by `+26` high byte, HP index 6 halved at every init, music only for var | listed difference |
| 21 | BT20 | Last boss end: `0xACEE` at HP 0 (stage index 7 sets `G.$4D7`, `G.$4DB`), 1,000 points at +368 frames | listed difference |
| 22 | DS12 | Brontosaurus is one huge foot (A22) with a leg helper (A23): hover 180 frames over the nearest hero | listed difference |
| 23 | HU15 | Brontosaurus foot and barrels hurt poachers in their way (foot 30 knock down and 1 touch, barrel 10) | listed difference |
| 24 | P69 | Hero damage factor table 0x2C88E by hero id: Jack 32, Mustapha 34, Hannah 35, Mess 30; enemy to hero | listed difference |

## Closing (2026-10-21)

Since the second pass (task 55), the whole-run checks of M10 (the natural recordings ep2_jack, ep2_3p, ep3_jack, ep3_hannah, ep3_mess, ep3_mustapha, ep3_3p, ep4_jack, the five Episode 5 runs, and the three-player runs of Episodes 1 to 6) are equal to MAME in lagsync mode, with the known causes S (story panel inside a lag record), K (skip wait rounds) and P (the title typewriter), listed in `re/harness/allow/m10_runs.txt`. The rows of Episodes 1 to 5 whose mechanic happens in those runs are now "equal" (the recording is named in the evidence column). What is left: the rows of Episodes 6 to 8 (one-player runs still being checked, so they stay "implemented not checked"), the rows with no episode named (no recording can show them as one row), and the 13 listed differences and 11 missing rows of task 55, which are still open (the obstacle slot G.$6A32, the splash timer, the Vice summon request, the team move voice lines, and the rest in the fix list).

