# M9-08 audit C (M9-10): mechanics rows, groups J to N (user notes and web leads)
Updated 2026-10-21 (task 67): whole-run checks applied; see the closing section at the end.

Written by the desk Haiku worker (task 59), with the method of audit B (`m9d_gen.py`, the same code as `m9c_gen.py` for groups J to N). Verdicts come from the port code (address, class or first word of the lead in `dino-game/src/game`) and the sweep refresh 7, plus a hand check of every row the grep marks "missing" (search for the behaviour itself, not the address).

**Method:** the grep (address, class, name) and the recording link as in audit B. Every "missing" row was hand checked (player_dead in `player.rs:3865` was the example that the grep missed for T27 in task 58). Only the hand-checked rows changed verdict.

**Could check:** for each row that is only "implemented not checked", the natural whole-episode recording that could check it. The episode comes from the lead text; otherwise any natural recording.

**Limits:** the covering task is the group level list of `D5-final-plan.md` section 7. The name grep is weak (see audit A). The row ids U01 and U02 are not matched by the table regex, so they were added by hand.

## Counts

Verdicts of the main table (each row once). "Before" is the count at task 55 / 57 / 59; "after" is 2026-10-21.

| verdict | before | after |
| --- | --- | --- |
| equal | 0 | 21 |
| listed difference | 1 | 1 |
| implemented not checked | 74 | 53 |
| missing | 3 | 3 |
| not checked | 1 | 1 |
| total | 79 | 79 |

## Leads (groups J, K, L, M, N)

| row id | the lead | covering task | port file:line | evidence | could check | verdict |
| --- | --- | --- | --- | --- | --- | --- |
| G01 | Episode 1: the hero leaves the building by jumping or falling down to the street (scripted area chan | group J (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | - | equal |
| G02 | Vice T.'s intro: he provokes a chained Rock Hopper into the fight; Vice rushes, moves fast, summons  | group J (plan s7) | dino-game/src/game/boss.rs:29 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| G03 | Butcher: dual blades, jumping body slam, charges; his blades can be knocked away and picked up as we | group J (plan s7) | dino-game/src/game/boss.rs:46 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| G04 | Episode 3: the radio comes from an early breakable barrel; in the car the players move between lanes | group J (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | - | equal |
| G05 | Episode 4 opening: the Cadillac crashes through the blocked garage entrance, gang members react, Sli | group J (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | - | equal |
| G06 | Slice (Episode 4 boss, single): very fast with afterimages, throws returning boomerangs, several pro | group J (plan s7) | dino-game/src/game/placeholder.rs:43 | no recording found by the row tag | - | equal |
| G07 | Episode 4: Walther as a mid-boss, motorcycle enemies crossing the arena, enemies from doors and high | group J (plan s7) | dino-sound/src/pan.rs:22 | no recording found by the row tag | - | equal |
| G08 | Falling rocks in Episode 5 leave stones the player can use as weapons | group J (plan s7) | dino-game/src/game/terrain.rs:878 | no recording found by the row tag | - | equal |
| G09 | Bludge jumps around, attacks at range, and may eat items left on the ground | group J (plan s7) | dino-game/src/game/effect.rs:76 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| G10 | Episode 5 story scene: the old villager starts to explain, Morgan shoots him; Morgan fights with fas | group J (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | - | equal |
| G11 | Episode 6: doors in the mine that the player must break; the brontosaurus foot casts a warning shado | group J (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | natural Episode 6: ep6_3p, ep6_jack | implemented not checked |
| G12 | Tyrog: a parasite that changes hosts: green host (bladed limbs), orange host (lunges, fireballs), bl | group J (plan s7) | dino-game/src/game/enemy_combat.rs:324 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| G13 | Episode 7: a bridge, motorcycle riders and flying creatures at the start; in the library enemies com | group J (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | natural Episode 7: ep7_3p, ep7_jack | implemented not checked |
| G14 | Episode 8: the glass tanks break and release dinosaurs and mutants (see W21, the user's tanks 1, 3,  | group J (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | natural Episode 8: ep8_3p, ep8_hannah, ep8_jack | implemented not checked |
| G15 | Fessenden: first form like Morgue (charges, close strikes); final form a huge pink mutant with two r | group J (plan s7) | dino-game/src/game/enemy_combat.rs:553 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| G16 | Ending: self-destruct, the heroes run, Hannah falls and Jack goes back for her, Mustapha and Mess es | group J (plan s7) | dino-game/src/game/flow.rs:27 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| G17 | Dash attacks per hero: Jack a low slide, Hannah a flying knee, Mustapha a long flying kick, Mess a f | group J (plan s7) | dino-game/src/game/enemy.rs:1147 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| G18 | Food and treasure names: pizza, burger, hot dog, steak, meat, cake, chocolate, coffee; jewels, coins | group J (plan s7) | dino-game/src/game/trace.rs:1154 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| G19 | Continue screen: an armed gangster seen in first person with the countdown | group J (plan s7) | dino-game/src/game/flow.rs:2 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| L01 | Episode 3 car seats: the player who took the radio drives, the first other player sits in the passen | group L (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | - | equal |
| L02 | Episode 3 has no recovery items at all; skipping the radio makes it much harder (time-outs nearly ce | group L (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | - | equal |
| L03 | Hogg: phase 1 must be fought from the car (car vs motorcycle, grenades, he can run players down, ram | group L (plan s7) | dino-game/src/game/boss.rs:7 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| L04 | Vice fires into the air to call more henchmen; his Rock Hopper turns from orange to green when beate | group L (plan s7) | dino-game/src/game/boss.rs:29 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| L05 | Butcher: knocking him down makes him drop a knife; his knives are weaker than his standing combo; he | group L (plan s7) | dino-game/src/game/boss.rs:46 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| L06 | Episode 4: Slice orders the gate shut; the player in the Cadillac blows away the gate defenders and | group L (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | - | equal |
| L07 | Episode 2: a scoring item hides under three foreground trees at the start; a fat enemy beats the sle | group L (plan s7) | dino-game/src/game/enemy/class79.rs:1 | no recording found by the row tag | - | equal |
| L08 | Episode 5: Bludge's tongue grabs and stuns the player; water buckets hold items; Morgan jumps to the | group L (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | - | equal |
| L09 | Episode 6: a wall at the start is punched open to enter the mine; no firearms in the whole episode;  | group L (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | natural Episode 6: ep6_3p, ep6_jack | implemented not checked |
| L10 | Episode 7: the rooftop is Episode 1's rooftop in evening colours; the pterodactyl dies in one hit (Z | group L (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | natural Episode 7: ep7_3p, ep7_jack | implemented not checked |
| L11 | Episode 8: jar 3 releases Bludge, jar 4 Rock Hopper, jar 11 Tyrog 2 (one form, does not revive, wort | group L (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | natural Episode 8: ep8_3p, ep8_hannah, ep8_jack | implemented not checked |
| L12 | Gun supply by player count: extra guns appear for two or three players up to Stage 4 (a shotgun for  | group L (plan s7) | dino-game/src/game/enemy/class18.rs:1471 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| L13 | An empty gun can be thrown for one last hit | group L (plan s7) | dino-game/src/game/terrain.rs:61 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| L14 | Continue screen: a boss aims a gun in close-up with a 20 s countdown; continuing makes the player pu | group L (plan s7) | dino-game/src/game/flow.rs:2 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| L15 | A route map of the New York area appears at each stage start (the map card D4c and D4d measured) | group L (plan s7) | dino-game/src/game/enemy/class25.rs:30 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| L16 | Debug mode (TCRF): stage select, slow motion, invincibility, object sprites and an object-debug over | group L (plan s7) | dino-game/src/game/flow.rs:69 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M01 | Enemy kill scores and Japanese names (Gneiss 2000, Ferris 1000, Driver 1000, Punk 1000, Doug 2000, B | group M (plan s7) | dino-game/src/game/boss.rs:430 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M02 | Punk sometimes throws a grenade then runs away; 'Doug' (the alternate Punk) also body-slams and jump | group M (plan s7) | dino-game/src/game/enemy/class64.rs:1 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M03 | Blade drops his knife when knocked down; a thrown knife can be punched away; some Razors drop from a | group M (plan s7) | dino-game/src/game/player_combat.rs:366 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M04 | Gutter rarely drops a rifle but leaves ammo if a gun lies on the floor; Walther never flinches (no h | group M (plan s7) | dino-game/src/game/enemy.rs:15 | no recording found by the row tag | - | equal |
| M05 | Bludge drops an item it is eating when hit, never drops a digested one, and can pick up the falling | group M (plan s7) | dino-game/src/game/effect.rs:76 | no recording found by the row tag | - | equal |
| M06 | Bosses have a fixed HP total regardless of rank; with 2+ players the boss count or HP rises and a bo | group M (plan s7) | dino-game/src/game/boss.rs:2 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M07 | Vice: opens by hitting Rock Hopper three times to provoke it; calls henchmen each time his HP drops  | group M (plan s7) | dino-game/src/game/boss.rs:29 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M08 | Butcher: each knockdown makes him drop a machete, then he runs to pick it up (lost if a hero grabs i | group M (plan s7) | dino-game/src/game/boss.rs:46 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M09 | Hogg mirrors the hero's forward and backward movement about half a beat late; holding left in the ca | group M (plan s7) | dino-game/src/game/boss.rs:7 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M10 | Slice: a single slow boomerang can be punched away; super = several boomerangs with a spin that make | group M (plan s7) | dino-game/src/game/placeholder.rs:43 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M11 | Morgan: hops, throws knives, rocks and grenades, fires an Uzi, gives 0 points; Morgue has the Butche | group M (plan s7) | dino-game/src/game/boss.rs:485 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M12 | Tyrog: bone spear from the chest, short arm hits, flying body press; when beaten only the small head | group M (plan s7) | dino-game/src/game/enemy_combat.rs:324 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M13 | Fessenden form 2 needs about six hits to be knocked down, ignores attacks mid-combo, but a grab and  | group M (plan s7) | dino-game/src/game/enemy_combat.rs:553 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M14 | Dinosaurs: a machine-gun hit makes them counterattack at once, shotgun pellets do not; only a stegos | group M (plan s7) | dino-game/src/game/placeholder.rs:10 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M15 | Scenery fauna: a grazing duck-billed dinosaur in the Episode 2 swamp; two kinds of little dinosaurs | group M (plan s7) | dino-game/src/game/prop.rs:39 | no recording found by the row tag | - | equal |
| M16 | Score item values (gold bar, diamond, dinosaur egg 10,000; gold dust, pearl shell, ammonite 5,000; w | group M (plan s7) | dino-game/src/game/flow.rs:227 | no recording found by the row tag | - | equal |
| M17 | Food heal (percent of the bar) and full-HP bonus: barbecue 100 / 10,000, steak 90, sushi and lobster | group M (plan s7) | dino-extract/src/enemy.rs:299 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M18 | Weapons: dynamite lights when thrown or merely set down to pick up another weapon, and picking it up | group M (plan s7) | dino-game/src/game/enemy_combat.rs:336 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M19 | Points per action: punch 100, kick 200, held weapon hit 400 (stone 100), throw 400, 'single smack' 5 | group M (plan s7) | dino-game/src/game/flow.rs:224 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M20 | Boss kill scores: Vice 10,000, Butcher 20,000, Hogg 1,000 (on foot only), Slice 40,000, Morgan 0, Mo | group M (plan s7) | dino-game/src/game/boss.rs:1 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M21 | The continue counter shown for every player goes up by 1 for each continue of any player; continuing | group M (plan s7) | dino-game/src/game/flow.rs:2 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M22 | Rank system as fans describe it: a normal rank and a timer rank 0..15 (how fast the normal rank rise | group M (plan s7) | dino-game/src/game/flow.rs:166 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M23 | Rage state: when the HP bar flashes at low HP most attacks hit harder (dynamite kills fat enemies, t | group M (plan s7) | dino-game/src/game/enemy/class29.rs:605 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M24 | A thrown empty gun sometimes leaves the target dazed (stars); plain beating sometimes does too | group M (plan s7) | dino-game/src/game/enemy_combat.rs:400 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| M25 | Team move voice lines (Jack "Let's Go!", Mustapha "Let's Roll!", Mess "Let's Fly!", Hannah "Jump!"); | group M (plan s7) | dino-rs/crates/dino-game/src/game/snd.rs (request codes for hero voices); the team move code (WM18 area) | really missing: hero voice lines (Let's Go, Let's Roll, Let's Fly, Jump; Take that, Gotcha, Blammo) are not in the port  | - | missing |
| M26 | Stage extras: Episode 4 shows a battered, overturned Cadillac later in the stage and a broken drum c | group M (plan s7) | dino-game/src/game/boss.rs:4 | no recording found by the row tag | - | equal |
| M27 | Overseas opening text (the story intro of the attract loop) and endings: after the credits Jack and  | group M (plan s7) | flow attract text (flow/ folder), frontend text tables | overseas opening text: no reference by name in the port (the attract text); ending data is present (`dino-extract/src/fr | - | missing |
| M28 | In MAME the invincibility cheat stops a stage from ending because the hero keeps the golden glow of  | group M (plan s7) | dino-game/src/game/boss.rs:33 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| N01 | Operator settings (in the 93C46 EEPROM, edited in the test menu; the bootleg shows them as DIPs): di | group N (plan s7) | dino-rs/crates/dino-game/src/main.rs; game/rank.rs | present in part: difficulty from the settings file (`main.rs --config`: difficulty, coin mode, free play) and G.$777B (` | - | listed difference |
| N02 | Music map from the soundtrack: 22 area tracks by stage-area code (1-1..8-3, Episode 3 = 3-A on foot | group N (plan s7) | dino-game/src/game/boss.rs:50 | no recording found by the row tag | - | equal |
| N03 | HUD: health bars are longer with 2 players than with 3 but hold the same amount; a player rank numbe | group N (plan s7) | dino-rs/crates/dino-game/src/game/hud.rs | bar length with 2 players and the bullet row: not found in `hud.rs` (player bar is `hud_life_bar`, `hud.rs:15`); hand ch | - | missing |
| N04 | Enemy health in bars: standard enemies at most 1 bar, sub-bosses (Lash, Walther, Bludge, Tyrog 2) up | group N (plan s7) | dino-game/src/game/flow/cont.rs:1075 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| N05 | Combat details from a long FAQ: a grab allows 3 follow-up hits before a throw; after a 3-hit combo u | group N (plan s7) | dino-game/src/game/boss.rs:492 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| N06 | Holding Attack or Jump speeds up the dialogue scenes (the opening, the talk before boss fights); the | group N (plan s7) | dino-game/src/game/effect.rs:13 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| N07 | Episode 3 car controls: left slows the car, right speeds it up, up and down move it; scroll is autom | group N (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | - | equal |
| N08 | Episode 4 walkthrough: Walther appears when the hero steps past a floor vent; punks on motorcycles c | group N (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | - | equal |
| N09 | Episode 5: the dinosaur egg lies near meat items at the start; boulders crumble into throwable rocks | group N (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | - | equal |
| N10 | Episode 6: two rolling barrels carry items (grenade, pizza) only if the player goes slowly; dashing  | group N (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | natural Episode 6: ep6_3p, ep6_jack | implemented not checked |
| N11 | Episode 7: the elevator brings groups of four, a knife guy drops in with a shadow marking where he l | group N (plan s7) | dino-game/src/game/enemy/class52.rs:1 | no recording found by the row tag | natural Episode 7: ep7_3p, ep7_jack | implemented not checked |
| N12 | Episode 8: four fat enemies come from the four corners; wall exhaust pipes can be destroyed; running | group N (plan s7) | dino-game/src/game/flow.rs:86 | no recording found by the row tag | natural Episode 8: ep8_3p, ep8_hannah, ep8_jack | implemented not checked |
| N13 | Enemy category names used by players: Punk, Midget (small thug), Knife Guy, Fat Boy, Hunter | group N (plan s7) | dino-game/src/game/enemy/class6_eat.rs:4 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| N14 | Hardware clocks from the MAME driver: 68000 12 MHz, Z80 8 MHz (the earlier 11.44 / 7.63 MHz figures  | group N (plan s7) | dino-game/src/game/trace.rs:2040 | no recording found by the row tag | any natural recording (no episode named) | implemented not checked |
| U01 | Some enemies pick up weapons and items lying on the ground and go away with them (PUNK, the small qu | group K (plan s7) | - | row id U01 (not in the parsed set) | - | not checked |
| U02 | A weapon may vanish if the hero is beaten up while holding it | group K (plan s7) | - | port state DONE (item side, task 2); row answered by D2c section 8 | any natural recording (no episode named) | implemented not checked |

## Fix list (after the whole-run checks, 2026-10-21)

Remaining rows that are missing, a listed difference or not checked. Missing first, then listed differences, then not checked.

| # | row id | what is missing or different | verdict |
| --- | --- | --- | --- |
| 1 | M25 | Team move voice lines (Jack "Let's Go!", Mustapha "Let's Roll!", Mess "Let's Fly!", Hannah "Jump!"); | missing |
| 2 | M27 | Overseas opening text (the story intro of the attract loop) and endings: after the credits Jack and | missing |
| 3 | N03 | HUD: health bars are longer with 2 players than with 3 but hold the same amount; a player rank numbe | missing |
| 4 | N01 | Operator settings (in the 93C46 EEPROM, edited in the test menu; the bootleg shows them as DIPs): di | listed difference |
| 5 | U01 | Some enemies pick up weapons and items lying on the ground and go away with them (PUNK, the small qu | not checked |

## Closing (2026-10-21)

Since task 59, the whole-run checks of M10 make the Episode 1 to 5 leads "equal" where the recording is named in the evidence column (lagsync, with causes S, K and P). What is left: the leads of Episodes 6 to 8 (still being checked), the leads with no episode named (the rage state, the thrown gun dazes), the team move voice lines (M25, probably not in the ROM), the overseas opening text (M27), the 2-player bar length (N03), the EEPROM operator settings (N01, partly present), and U01 (not checked by hand yet).

