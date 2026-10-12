# Mechanics seen (flat list, the seed for the design phase D2 to D4)

Every small rule or mechanic met during the research, even when I did not dig into it. One row each.
This is **not** a spec. Each row says where the fact comes from and what the port does today.
The design agents D2 (player), D3 (enemies, dinosaurs, bosses) and D4 (world systems) take the rows of
their area, dig to numbers and addresses, and add what is missing. Add new rows at the end of a group.

**Port state:** DONE (ported and checked against MAME), PARTIAL (ported, not fully checked, or part missing),
STUB (empty named function), MISSING (no code), UNKNOWN (original not understood yet).

**Source tags:** `pl x.y` = `re/specs/player/SPEC.md` section; `en x` = `re/specs/enemy/SPEC.md` section;
`fl x` = `re/specs/flow/SPEC.md`; `st x` = `re/specs/stage/SPEC.md`; `ph1` = `plan/phase-01-stage1-complete.md`;
`M` = MAME observation 2026-10-12 (README section 0); `W:aqm`, `W:hg101`, `W:ja`, `W:en`, `W:ah` = web
(Arcade Quartermaster, Hardcore Gaming 101, Japanese Wikipedia, English Wikipedia, arcade-history.com; links in
`README.md` section 5). Epi = the episode(s) where it matters.

## A. Player

| ID | Mechanic | Source | Port | Notes for the design agent |
| --- | --- | --- | --- | --- |
| P01 | Walk, run by double tap; run ends by timeout (240 f), releasing the stick does not stop it | pl 3.5 | DONE (Jack checked) | other 3 heroes: tables differ |
| P02 | Jump with a vector per hero, air control, jump attacks, dive kick at the top of the jump | pl 3.6, W:aqm | PARTIAL | verify all four heroes in MAME |
| P03 | Wall jump (Hannah, Mustapha) with a "triangle jump" attack, high priority | pl 7, W:aqm | PARTIAL | where walls exist per episode |
| P04 | Punch chain (2 jabs, body hit, finisher), combo window 40 f, hit confirm | pl 4.4 | DONE (Jack) | |
| P05 | Combo throw after the chain (shoulder throw; Mess slam / suplex) | pl 4.8, W:aqm | PARTIAL | |
| P06 | Grab by walking into an enemy, rapid attack = knees (each costs enemy stun and HP), throws forward and back | pl 4.8 | DONE (grab scenarios) | |
| P07 | Dash attack: Jack slides, the others use an air lunge | pl 4.7, W:aqm | PARTIAL | |
| P08 | Special ("mega crush", attack + jump): costs 10 HP for the first victim, 4 for the 2nd and 3rd, never kills; immune during it and 60 f after | pl 4.6 | PARTIAL | HP cost rule is ported; check immunity |
| P09 | Hannah's special is airborne and hovers | pl 4.6 | PARTIAL | |
| P10 | Command move (down, up + attack) per hero ("sure kill" move) | pl 4.10, W:aqm | PARTIAL | |
| P11 | Tag team move: overlap an ally for 2 s, the ally is flung (Let's Go, Jump, Roll, Fly) | W:aqm, W:ja ("team attack") | STUB | player state 0x1A / 0x6866 tie-up not ported |
| P12 | Players hurting each other: 1 HP per hit with 2 or 3 players (Z window table), never below 1 HP | pl 6 (0x10664), ph1 | STUB | G.$4F3 returns 3 so it is off; decide with D2 whether it is active in normal play |
| P13 | Hurt reactions: light flinch, pushback with dust, knockdown (front and from behind), heavy knockdown, burn, thrown, wall bounce | pl 5.2 to 5.5 | PARTIAL | burn flame objects partly missing |
| P14 | Hit stop on both sides (frames from the attack table) | pl 4.5 | DONE | |
| P15 | Lying down: mash the stick (L or R counts 3, else 1) or the attack button to get up sooner; lie time 5, 15 or 30 f by level | pl 5.5 | DONE | |
| P16 | Dizzy (stars): 3rd hit in a 60 f chain always, 2nd 75 percent, only if HP >= 30; stars can be mashed away; immune for 1800+ f after | pl 5.6 | PARTIAL | star effect (0x94338) missing |
| P17 | Mercy rule: a hit that would kill with at most 4 HP over leaves 1 HP | pl 5.1 | DONE | |
| P18 | Damage taken scales with enemy type and rank level, hero factor, low HP random cut and low HP factor | pl 5.1, en 6.6 | DONE | the two specs list the hero factors in a different order; ids are 0 Jack, 1 Mustapha, 2 Hannah, 3 Mess: settle in D2 |
| P19 | Invulnerability windows: after get up 20 f, after a throw 12/10/14/18, after a pickup 10/8/20/10, after the special 60, respawn 360 f with blinking | pl 5.9 | DONE / PARTIAL | respawn DONE |
| P20 | Frame classes 6, 8, 0xA are immune to enemy attacks (special, some moves) | pl 5.1 | DONE | |
| P21 | Death: 60 f blink, lose one life, respawn drop-in, or game over for that player | pl 5.8 | DONE | |
| P22 | Respawn speech bubble ("they will pay for this") and bazooka at a continue | M, ph1 | PARTIAL | bubble is MISSING; bazooka DONE |
| P23 | Lives: 1 extra life by default (2 in all), extra life at 500000 points with a 1UP marker and a pose | fl 2, M | STUB | pose and marker missing |
| P24 | Score per hit (100/200/400), per kill, container +500, treasure, food at full HP | en 3.4, 13 | DONE | |
| P25 | HP bar blinks at 15 HP or less; face changes when hurt | fl 2 | DONE | |
| P26 | Weapons: pick up with the attack button (no auto pickup), hold, swing, charge swing (hold attack; critical hit chance with a club [W:aqm]), throw, wear and break | pl 4.9, en 13 | DONE | |
| P27 | Weapon list: gun, Uzi, M-16A1, rifle, shotgun, bazooka, knife, stones, grenade, dynamite, magazine (ammo), rod, club, torch, butcher knife, TNT | en 13.4, W:aqm | PARTIAL | torch glow overlay (item class 6) missing |
| P28 | Hannah stabs repeatedly with a knife when it flashes red | W:aqm | UNKNOWN | |
| P29 | Heavy objects (barrels): lift, walk, jump, throw; Hannah cannot jump with one | ph1 | DONE | |
| P30 | 40 frame pickup lock after a weapon pickup | pl 4.9 | DONE | |
| P31 | Food is eaten on the pickup press, heals or gives points when full | en 13.5 | DONE | |
| P32 | Join in during play: blinking JOIN-IN or INSERT COIN in the HUD, quick select, drop in | M, fl 6 | STUB | |
| P33 | Continue: countdown 20 to 0, taunts, quick select, drop in with a bazooka | M | STUB | |
| P34 | Time over kills every living player at once | rank.rs 0x103FE | DONE | |
| P35 | Players are kept inside the screen, cannot go left of the camera | pl 2, st 6.8 | DONE | |
| P36 | Wading in water (legs cut at the water line), splash on landing, ripples | st, anim HANDOFF | DONE (draw) | water splash effect 0x606E partial |
| P37 | Riding the Cadillac in Episode 3 and the vehicle entrance state 0x432 | pl HANDOFF, W | MISSING | |
| P38 | Scripted control: walk in at the start, automatic walk at area end, stage exit | pl 2 (0x1DA10, 0x1DB0C, 0x1DB5E) | STUB | blocks the intro, transitions |
| P39 | Per hero voices on hit, attack, special | sound SPEC | MISSING | no sound in the port |
| P40 | Respawn flash (pool C class 0x38), sparkle (0x5C), 1UP marker (pool B class 0x48) | pl 5.8 | STUB | |
| P41 | Players can pick which hero they play at the start and at a continue (can change hero) | M | STUB | |

## B. Enemies (humans and mutants)

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| E01 | Enemy census, target selection: prefer a player with fewer than 3 enemies | en 5 | DONE | |
| E02 | Aggression 0..7: forced zones from the script, re-rolled at every kill | en 6.5 | DONE | |
| E03 | Rank and level scale HP, damage, recovery, dizzy odds, level-gated spawns | en 6 | DONE | |
| E04 | Entrances: walk, dash, wait, fade in, jump, drop; spawn conditions by player count and level | en 7.3, 8.1 | DONE | |
| E05 | Attack slots around the player, 3 per side, flanking, slot rotation | en 7.4, 14 | DONE | |
| E06 | Poise: flinch until poise 0, then knockdown; counter attack after a flinch | en 10.2 | DONE | |
| E07 | Dead enemy blinks 120 f; item dropped at the killing blow; kill re-rolls aggression | en 10.3, 13.3 | DONE | |
| E08 | One dizzy enemy at a time, dizzy enemies can be hit freely; Vice and Walther never get dizzy | en 10.2 | PARTIAL | stars effect missing |
| E09 | Thrown enemy bodies hurt other enemies | en 10.2 (0x1185A) | DONE | |
| E10 | Dead big enemies bounce off the screen sides; wall slump | en 10.2 | DONE | |
| E11 | Grabbed enemy follows the hero's hand using per frame offsets | en 10.2, ph1 3b | DONE | |
| E12 | Kill-all when a boss dies (score goes to the boss killer) and when a wave takes over 7200 f (no score) | en 8.2 | DONE | |
| E13 | Enemy name, portrait, bar with drain and skull blink in the HUD message area | fl 3 | DONE | |
| E14 | Enemies far off screen are freed (0x173C), also vertical Z+Y test in the lift | en 4, ph1 | PARTIAL | vertical test missing |
| E15 | Hammer / Blk Elmer / Wrench: paired attack on a downed hero, stomp; the grab test never passes (bug kept) | en 17.2 | DONE | |
| E16 | Punk / Thug: hops, jump kicks, leaping dive | en 17.4 | DONE | |
| E17 | Blade / Razor: knife throwers, one record makes 3 objects, parts classes 60, 34, 41, 15..17, drop knives when killed [W:aqm] | ph1 | PARTIAL | another agent is porting class 14 now |
| E18 | Poachers (Poacher J, Skinner, Gutter): ex soldiers with rifles | W:aqm, W:hg101 | MISSING | |
| E19 | Lash T.: mace in Episode 2 [W:aqm]; appears in 4 later episodes | W:aqm | MISSING | |
| E20 | Bludge: human dinosaur hybrid, tongue attack, acid spit | W:aqm | MISSING | |
| E21 | Walther: leader with a gun, same class as Vice | en 12 | MISSING | |
| E22 | Zeke: pteranodon that flies in, hits the heroes; can be hit only by jumping | W:aqm, W:ah | MISSING | see dinosaurs |
| E23 | Enemies carry weapons that heroes can take (butcher knife) | W:aqm | UNKNOWN | |
| E24 | Enemy classes 65 and 70, pool A 5, 16, 22, 27, 35, 43, 45, 66, 78, 80: roles unknown | stage scripts | MISSING | D3 |
| E25 | Boss reinforcement controller (pool B 22): timed list, threshold list, only after a summon | en 12 | MISSING | |
| E26 | Anti tamper words at 0x2A5B6, 0x29ADE, 0x104F6 do nothing on a real ROM | en 15 | n/a | ignore |

## C. Dinosaurs and flyers

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| D01 | Sleeping dinosaurs (Z letters, a roar every 50 to 113 f) | en 11, M demo 1 | MISSING | Episodes 2, 5 |
| D02 | Poachers' hits wake a dinosaur and it attacks poachers as well as heroes | en 11, W:ah | MISSING | targets the nearest enemy with chance 14/16 |
| D03 | Hit by a hero it becomes hostile: palette flash, then enraged (red) | en 11, W:ja | MISSING | |
| D04 | At 0 HP the dinosaur calms down (turns green) and walks away, gives no score | en 11, W:ja | MISSING | |
| D05 | A dinosaur leaves when the camera passes the X in its spawn record | en 11 | MISSING | |
| D06 | Shivat (T. rex) attacks: bite 14, tail swing 10 with knockdown, triple stomp, grab and hold (cooldown 180) | en 11 | MISSING | listing exists |
| D07 | Mack (triceratops) hostile | W:aqm | MISSING | undecoded |
| D08 | Rock Hopper (velociraptor): Vice's partner in Episode 1, two of them in Episode 5 | W:aqm, stage 5 | MISSING | undecoded |
| D09 | Zeke (pteranodon, class 82): a big orange flyer that circles above the heroes in long sine like swoops (high flapping pass, then a low glide at head height), and after about 8 seconds dives at a hero with a growing shadow on the floor; the first dive hit took 13 HP (100 to 87) from Jack at low rank. In Episodes 5, 7, 8. Can only be hit by jumping [W:ah] | M (spawned with a poke in Episode 1: `spawnA 0x148`), W:aqm, stage scripts | MISSING | undecoded; the user's "bird dino flying and giving damage". D3b must get the flight path, the dive trigger, damage by level, the hit box and what happens when hit |
| D10 | Ambient pterodactyls in the sky (bird controllers 49, spawner 73, birds 50) | en 17.1 | DONE | |
| D11 | Brontosaurus legs crush whoever is under them at two places in Episode 6 | W:ah, W:aqm | MISSING | class 22 probably |
| D12 | Dinosaurs turn red when enraged and change colour when drained (palette) | W:ja, W:ah | MISSING | |
| D13 | Does fire or a poacher bullet also enrage a neutral dinosaur? | | UNKNOWN | |
| D14 | The time over scene has no pteranodon: a red bird flew by by chance (ambient) | M | n/a | |

## D. Bosses

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| B01 | Vice Terhune: gun volley, jump attack, rage at HP 50 and 25, summons at 1/2, 1/3, 1/4 HP | en 12 | MISSING | fully decoded in the spec |
| B02 | The Butcher: leaps, lands on the hero with his rear, two knives can be knocked away and used | W:hg101, W:ah | MISSING | |
| B03 | Hogg: grenades, 3 vitality bars, rammed by the Cadillac, flies around when the fight drags on | W:aqm, W:hg101 | MISSING | |
| B04 | Slice: boomerangs that multiply, very fast | W:aqm | MISSING | |
| B05 | Morgan changes into Morgue (wild dash); the change is shown on screen | W:aqm, W:hg101 | MISSING | |
| B06 | Tyrog three times (green, red, blue): rib attacks, acid spit; Tyrog 2 returns in Episode 8 | W:aqm | MISSING | |
| B07 | Slisaurs: 2 for 1 player, 3 for 2 or 3 players | W:aqm | MISSING | |
| B08 | Fessenden: two forms | W:hg101, W:ja | MISSING | sources disagree on the head count |
| B09 | Boss HP bar in the HUD with extra bars above 100 HP; boss HP table by players and level | fl 3, en 3.3 | DONE | |
| B10 | Boss intro scene (Vice walks in, raises his gun, boss music) and boss music change | en 12 | MISSING | |
| B11 | Slow motion (task period 4) while the boss dies, hero walk off, tally | fl 4 | PARTIAL | flow DONE, walk MISSING |
| B12 | Each boss death: rank speed +1, kill-all event, clear absent players' HUD blocks | fl 4 | DONE | |

## E. Items, containers, hazards, special scenes

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| W01 | Containers (drums, barrels, tires, buckets, armor, TNT) break in 4 HP, can be lifted, thrown, crush enemies | en 13.2 | DONE | |
| W02 | TNT barrel: fuse blink stages, then explosion that hurts all | en 13.2 | DONE | |
| W03 | Food heals 16 to 100 or gives points at full HP; 18 foods | en 13.4 | DONE | |
| W04 | Treasure gives 1000 to 10000 points; hidden behind trees in Episode 2 | en 13.4, W:aqm | DONE | |
| W05 | Drop rolls: enemy table and container table, guns reroll to ammo or a treasure | en 13.3 | DONE | |
| W06 | Dropped weapons last 3600 f; items bounce back in from the screen edge up to 5 times | en 13.3 | DONE | |
| W07 | RADIO in Episode 3 calls the Cadillac and triggers kill-all | en 13.4, ph1 | MISSING | pool B class 0x58 |
| W08 | Torches set enemies on fire | W:aqm | PARTIAL | burn reaction DONE; torch overlay class 6 missing |
| W09 | Fire bombs, dynamite, grenades, bazooka rockets explode with splash | en 13, ph1 | DONE | effect objects (debris, smoke) partly missing |
| W10 | Falling rocks on the mountain side (Episode 5) | W:aqm | MISSING | pool B 48? |
| W11 | Burning huts and flames in the village (Episode 5) | W:aqm | MISSING | pool B 13, 6, 10? |
| W12 | Flame jets in the sewers, lure enemies into them (Episode 8) | W:aqm | MISSING | |
| W13 | Barrels rolling down the cave slope (Episode 8), rolling drums with items in the library (Episode 7) | W:aqm | MISSING | |
| W14 | Elevator ride with enemies riding in at each stop (Episode 7) | W:aqm, st 6.6 | PARTIAL | camera mode 5 DONE, scroll 1 background MISSING |
| W15 | Cadillac drive: camera mode 8, car state G.$518, special collision passes, runs over enemies and barrels | st HANDOFF, W:hg101 | MISSING | |
| W16 | Row scroll (road bends) in Episode 3 | st 7.3 | MISSING | needs a per line clip in the renderer |
| W17 | Mine lamps (palette cycling), rain | st 2.5, W:aqm | MISSING | |
| W18 | Screen shake on stomps, landings, the lift (0x222D6) | st 6.7 | STUB | struct and logic are documented |
| W19 | Tile animation (water, fire) and palette cycling objects | st 2.5, 2.6 | PARTIAL | |
| W20 | Suits of armor in the Episode 1 hall hide items | W:aqm | UNKNOWN | probably container variant 0xA |
| W21 | Lab tanks (01, 02, 03) open and release Bludge, Hopper, Tyrog 2 (Episode 8) | M card, W:aqm | MISSING | pool B 44 |
| W22 | Garage props (Episode 4), doors, gates | stage scripts | MISSING | pool B 0, 1, 59 |
| W23 | The "GO" burst arrow at the right edge and the idle hint after 600 f | M, en 8.3, ph1 | MISSING | pool B 20 is the hint |
| W24 | Pool B classes with unknown role: 59 distinct in the scripts, 57 not ported | stage scripts | MISSING | D3 list |

## F. Flow, time, HUD

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| F01 | Per area time budget (1:20 up to 4:00); scripts set or add time; 51 frames per second | en 8.4, M | DONE | |
| F02 | Clock display: at 3:00 or less shows 5 s at each half minute; always in the last 15 s; red at 1:00 or less; beep each second | M, class 31 0x81046 | MISSING | |
| F03 | "TIME OVER" words fly in, all heroes die, next life with time topped up | M, 0x96450, 0x10482 | PARTIAL | HP kill DONE, words MISSING |
| F04 | In the car section the time over ends the drive and adds 3:00 | rank.rs, 0x103FE | DONE | |
| F05 | Camera lock, waves, GO release; 2 minute rule | en 8 | DONE | |
| F06 | Area changes: fade, no fade, automatic walk (0x1D95E), stage exit walk (0x1DAC4) | en 8.2 | STUB | TEMP shortcut |
| F07 | Opening scenes: Episode 1 walk in and fight, Episodes 2 and 3 car arrival | M | MISSING | |
| F08 | Map panel, episode card with typed title | M | MISSING | |
| F09 | Story panels before Episodes 2 to 8 (mode 8) | M | MISSING | |
| F10 | Clear tally: vitality, enemies, final blow +20000, best player +10000, TOTAL counts down into the score | fl 4, M | STUB | |
| F11 | Rank grows with time, falls at deaths, calm period after a continue | en 6 | DONE | |
| F12 | Rank start depends on how long the title waited with credits (G.$4F6) | en 6.1 | UNKNOWN | needs attract |
| F13 | Credits, coin modes, free play, EEPROM settings menu | fl 6, ROM strings | MISSING | |
| F14 | Test and service mode, debug mode by holding buttons at boot (starting level, invincibility, speed) | W (TCRF summary) | UNKNOWN | decide to skip or not |
| F15 | Demo sound mute setting | sound SPEC | MISSING | |
| F16 | HUD weapon display (icon, name, ammo) | ph1, fl | STUB | |
| F17 | 2 and 3 player HUD layouts | fl 2 | DONE | |
| F18 | In level prompts for absent players, 48 f on, 80 f off | fl, M | STUB | |
| F19 | Continue screen, game over screen | M | STUB | |
| F20 | Ranking screen and live ranking table; no name entry seen | M, fl 2 | PARTIAL | |
| F21 | Attract loop, title, profiles, demo, story slides, opening movie | M | MISSING | |
| F22 | Ending and staff roll | M | MISSING | |
| F23 | Random 1 in 128 extra task at the title, 1 in 16 branch in the first story panel | ROM | UNKNOWN | probably leftovers |
| F24 | Boot info and legal warning screens | M | MISSING | |

## G. Sound

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| S01 | Music per episode and area (table 0xA582, alternate 0xA5C8), boss music, title and ending tunes | fl 4, ph1 | MISSING | the table is not even extracted (panic) |
| S02 | Voices and hit sounds, pan by X | sound SPEC | MISSING | driver ported separately, not wired |
| S03 | Sound requests are recorded each frame | arch | DONE | |
| S04 | Clock beep (sound 0x34 with parameters), GO hint sound 0x3E | M, 0x81046 | MISSING | |
| S05 | Fade out music when an area ends (0xFF07) | fl 5 | DONE (request) | |

## H. Effects, camera, bounds

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| X01 | Hit sparks, landing splash, flames, ripples | anim HANDOFF | DONE | 4 of about 40 effect classes |
| X02 | Dust (0x92E5E), debris (0x934F6), dizzy stars (0x94338), burn death (0x93D48), muzzle flashes, score and text effects | ph1, effect.rs | MISSING | |
| X03 | Pool D special objects (the clock sprite list in slot 15 and others) | ph1 | MISSING | |
| C01 | Camera follow, lead by player count, max 6 px per frame, never back | en 8.3, st 6 | DONE | |
| C02 | Camera modes 0 to 0x13 | st 6.6 | DONE | stage 2 controller missing |
| C03 | Parallax (scroll 3), vertical scenes, sub-areas | st 7 | DONE | |
| C04 | Object clamp to the camera (0x193E) | st 6.8 | DONE | |
| C05 | Palette fades: area end, stage start, boss | st 2.4 | DONE | |
