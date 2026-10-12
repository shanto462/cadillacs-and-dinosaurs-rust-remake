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
| P41 | Players can pick which hero they play at the start and at a continue (can change hero) | M | STUB | D4d: list EVERY moment a hero can be changed (start, continue, join in during play, any other?) and whether two players may pick the same hero; the quick select strip during play (10 s) |
| P42 | Idle behaviour during play: what a hero does when the player does not touch the controls (idle animation or pose, if any), and the idle "GO" hint after 600 f (W23) | user 2026-10-12 | UNKNOWN | D2a: check in MAME with no input for 2000 f per hero; D4c: the hint |

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
| D02 | Poachers' hits wake a dinosaur and it attacks poachers as well as heroes | en 11, W:ah, U (2026-10-12: "yep in few stages, 2 and probably 5 and 6") | MISSING | targets the nearest enemy with chance 14/16. D3b: the dinosaur hunter (A79, a poacher type) wakes a sleeping Shivat with its third swing (0x533FA); a rifle poacher (A13) did not wake it in a spawned test. **D3a must check in natural play** (real scripts, no spawns) in Episodes 2, 5 and 6: which poachers wake or anger which dinosaurs, and how (swing, bullet, other) |
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
| W07 | RADIO in Episode 3 calls the Cadillac and triggers kill-all | en 13.4, ph1, U | MISSING | pool B class 0x58; see W15 for the drive (user description) |
| W08 | Torches set enemies on fire | W:aqm | PARTIAL | burn reaction DONE; torch overlay class 6 missing |
| W09 | Fire bombs, dynamite, grenades, bazooka rockets explode with splash | en 13, ph1 | DONE | effect objects (debris, smoke) partly missing |
| W10 | Falling rocks on the mountain side (Episode 5) | W:aqm | MISSING | pool B 48? |
| W11 | Burning huts and flames in the village (Episode 5) | W:aqm | MISSING | pool B 13, 6, 10? |
| W12 | Flame jets in the sewers, lure enemies into them (Episode 8) | W:aqm | MISSING | |
| W13 | Barrels rolling down the cave slope (Episode 8), rolling drums with items in the library (Episode 7) | W:aqm | MISSING | |
| W14 | Elevator ride with enemies riding in at each stop (Episode 7) | W:aqm, st 6.6 | PARTIAL | camera mode 5 DONE, scroll 1 background MISSING |
| W15 | Cadillac drive: camera mode 8, car state G.$518, special collision passes, runs over enemies and barrels | st HANDOFF, W:hg101, U (2026-10-12) | MISSING | **User:** in Episode 3, when the RADIO is taken, the Cadillac appears with Jack (or another hero) and from then on the episode is driven by car; the boss (Hogg) can bomb the car and kick heroes off it; otherwise the car can be used to eliminate the boss. D4c measured the start (car HP 120 drives in at 5 px/f, heroes aboard 280 f after the pick up, camera 11 px/f, section ends when the car HP is 0). **D3c must cover the whole drive:** who drives and who rides, how the players steer, accelerate, brake and attack from the car, what the car does to enemies, barrels and the boss (damage, ramming), Hogg's bombs and kicks against the car and the heroes, car HP and what breaks it, what happens when heroes are kicked off or the car breaks, and how the boss fight ends in both cases (car or on foot) |
| W16 | Row scroll (road bends) in Episode 3 | st 7.3 | MISSING | needs a per line clip in the renderer |
| W17 | Mine lamps (palette cycling), rain | st 2.5, W:aqm | MISSING | |
| W18 | Screen shake on stomps, landings, the lift (0x222D6) | st 6.7 | STUB | struct and logic are documented |
| W19 | Tile animation (water, fire) and palette cycling objects | st 2.5, 2.6 | PARTIAL | |
| W20 | Suits of armor in the Episode 1 hall hide items | W:aqm | UNKNOWN | probably container variant 0xA |
| W21 | Lab tanks (01, 02, 03) open and release Bludge, Hopper, Tyrog 2 (Episode 8) | M card, W:aqm, U (2026-10-12: "last episode, from background dinosaurs, 1, 3, 5, 7 these breaks and comes out, in the beginning of the stage") | MISSING | pool B 44 (three tank objects in the scripts of area 0 at X 0x30, 0x150, 0x5D0). **User:** at the start of Episode 8 the dinosaurs in the background tanks numbered 1, 3, 5 and 7 break their tanks and come out. D3e part 3 must walk Episode 8 area 0 in natural play and list every tank (number, X, which creature, what triggers the break, timing, the glass break art and sound, what the creature does after) and D3d covers any boss among them |
| W22 | Garage props (Episode 4), doors, gates | stage scripts | MISSING | pool B 0, 1, 59 |
| W23 | The "GO" burst arrow at the right edge and the idle hint after 600 f | M, en 8.3, ph1 | MISSING | pool B 20 is the hint |
| W24 | Pool B classes with unknown role: 59 distinct in the scripts, 57 not ported | stage scripts | MISSING | D3 list |
| W25 | Breakable parts of the background: pipes in walls (the user saw them), and probably windows, signs, wall panels and other scenery that break when hit and may drop items. How they work is unknown: a pool B object with a hit box that rewrites scroll2 tiles, or a container variant drawn as background | user 2026-10-12 | UNKNOWN | D3e: walk every area of every episode in MAME, hit every wall and background object, and list each breakable or interactive part: where, how it is hit, what it shows, what it drops, which class or routine does it |
| W26 | Catch-all: every other interactive thing in the levels (switches, doors, things that fall or roll when touched, objects that react to the players or to enemies) | user 2026-10-12 ("and many things") | UNKNOWN | D3e and D4c: the area-by-area walk above lists them all; nothing in a level is "background only" until checked |

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

## I. Rows added by the design phase (merged by the main session; D5 sorts them into the groups above)

### From `design/D2a-player-moves.md` (D2a player moves), 14 rows

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| P43 | Tag team: both heroes still and overlapped (less than 32 px in X, 0 to 15 in Z, no input, class 0) for 90 frames; initiator swings the partner (Jack: partner runs 43 frames at super speed and can attack with id 0x1A; Mustapha, Hannah, Mess: partner is thrown up and forward, vx 3 px/f, vy 6.5 px/f, can air attack with id 0x10); both blink invulnerable 180 frames | R: 0x6728..0x6922, M: d2a_tie | MISSING | D2a section 9; replaces the STUB row P11 |
| P44 | Friendly fire is active when fewer than 3 enemies are alive (`G.$4F3` is the enemy census count), depth window -1..1 / 0..1 / 0 by count, 1 HP per hit with the record's reaction, special immune | R: 0x10664, 0x106F2, M: d2a_D ff_* | PARTIAL (gate constant 3) | corrects P12 and the HANDOFF note |
| P45 | 1UP pose per hero (set 0 id 0xE): Jack, Mess ground state 0x18; Mustapha, Hannah air states 0x418, 0x41A; starts when idle within 240 frames after crossing the extra life score; invulnerable, then 180 frame blink | R: 0x6B98, 0x19996, M: d2a_D oneup_* | PARTIAL (score hook ported in `hud.rs`, pose states MISSING) | extra life thresholds (500000, 1000000 for the default setting) belong to D2b |
| P46 | Frames without a hurt box: special, throw, pick up, 1UP pose, tag team swing, drop in; dash recovery and dive landing for Mustapha and Hannah (Mess: zero size box at the feet) | R: dump | PARTIAL (data driven) | D2a section 15 |
| P47 | Dive kick window is 2 to 3 frames at the top of the jump (Jack and Mess top = split height) | M: d2a_basic jattack_* | DONE (data driven) | |
| P48 | Dive (0x0E) and running jump attack (0x10) deal 1.5 times the record damage to almost all enemy types (table 0xA32FA) | R: 0x2B904 | DONE | |
| P49 | Attack power factor `$C6` is 32 in 27 of 32 rolls at HP >= 75, re-rolled when the hero is hit; 64 possible below 25 HP | R: 0x11738, 0x2C9B6 | DONE | |
| P50 | Wall jump needs terrain wall bit 4 (or 0x18, 0x1C), found in Episode 1 areas 0 to 2, Episode 5 area 0, Episode 6 area 0, Episode 8 area 1; the triangle kick is the jump attack during the wall jump | R: 0x1A744, M: d2a_wallscan, d2a_wj | PARTIAL | D3e or D4c to list real spots |
| P51 | Thrown enemies hurt other enemies for 5 damage (enemy attack row 0x11), knock down | R: 0x1185A | DONE | |
| P52 | Score per hit from the record's score code: 100 / 200 / 300 / 400 / 700 / 800 / 1000 | R: 0x10292 | DONE | |
| P53 | Hannah's knife stab rate comes from her 5 frame swing animation; the item flashes red when an enemy is in reach; all heroes swing it at close range | M: d2a_D knife_* | DONE (D2c) | web statement corrected |
| P54 | Scripted player states 0x2A (two hops) and 0x2C (pose with an effect, leader only) and the scripted special (0x1E2EE) | R: 0x19EA6, 0x19F40, 0x1E2EE | MISSING | D4c |
| P55 | Stage clear and idle: no pose, no taunt; heroes idle during the tally | M: d2a_D clear_* | DONE | |
| P56 | Run start burst (7 frames Jack, 5 others at class 8 speed), double tap window 12 frames, run lasts 240 frames | R + M: d2a_dtap3 | DONE | |

### From `design/D3e-pool-b-inventory.md` (D3e pool B and area walk part 1), 12 rows

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| W27 | Steam pipes in the end wall of Episode 1 area 0: break with one punch (hero depth 296 to 320), then steam puffs forever (6 pieces alive) | M e1-pipez*, e1-pipebreak; R 0x836A4 | MISSING | B40, pieces ported. Answers W25 for Episode 1 |
| W28 | Episode 1 end door of area 0 is smashed by the hero during the scripted walk (dust, black doorway) | M e1a0-door; R 0x1DD30, 0x789C8 | MISSING | B11 |
| W29 | Hall double door opens at the start of wave 1; the hall window shatters when the scripted hero kicks it; the next area starts with a hole in the facade and later a wooden door bursts open | M e1a1-hall, e1a2-enter; R 0x78B52, 0x7B0BE | MISSING | B12, B14; flag G.$506 arms the window |
| W30 | Respawn air strike: five bombs fall in a line, white blast, explosions around the dropping hero, speech bubble | M respawn; R 0x18B9C, 0x7B65E; user 2026-10-12 | MISSING | B18 (not a 1UP marker: corrects P40 and P22). **User:** after a hero loses a life, the respawn comes with invincibility for a while and the bombing, which damages enemies but does not finish them off (enemies near death survive it). D2b must measure: damage per bomb and per enemy type, the rule that keeps enemies alive (HP floor?), bosses, items and containers hit, players hit, timing |
| W31 | Breakable family of pool B: classes 0 (generic wall panel, 15 variants), 1 (chain-link fence), 3 (wall), 12 var 4 (window), 40 (steam pipes), 45 (flame vent, HP 0x30, can be destroyed); only these 6 register in the player hit list 0x4BFA; one hit always breaks | R static search | MISSING | no pool B class hurts players |
| W32 | Episode 2 ambient life: small blue bird flocks (B72, A46 with A48 trail) harmless; ground critters (A78, A80) harmless; jungle sound maker (B80, sound 0x78) | M e2-birds2; R | MISSING | |
| W33 | Tile animation family of pool B: periodic scroll2, scroll3 or palette writers (23 classes), some keyed to a global tile clock G.$7AE | R | MISSING | one generic block table driver can run most |
| W34 | Class 69 is a debug object (frees itself unless G.$4AA); class 66 has an anti tamper check (0x86B3C); class 96 starts the area music (table 0xA582) | R | n/a (69, 66 check), MISSING (96) | |
| W35 | Class 95 blends 3 palettes toward a target set along the Episode 3 road (sky colours); classes 83, 84, 85 are debris showers with quake (Episode 8 lair, ending) | R | MISSING | |
| W36 | Container variants of E class 5: 0 pail, 2 drum, 4 barrel, 6 stone, 8 boulder, 0xA armor statue (solid), 0xC tire stack | M cont0 .. cont12 | DONE (pool E) | names only |
| W37 | "STAGE CLEARED!" and digits 1 to 8 are frames of pool C class 9 (0x93184); 1P, 2P, 3P tags are B68; red pillars are B64; fences are B1 | R | MISSING | answers N1 of the asset catalog for those groups |
| W38 | Treasures of Episode 2 area 0 sit at depth 16, height 64, behind the big yellow trunks (E class 9 var 2, 8, 0xE, 0xD) | M e2-treasure | DONE (pool E) | |

### From `design/D4d-front-end-and-story.md` (D4d front end and story), 21 rows

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| F25 | The attract state is kept across games: after a game over the loop resumes in the state it was in, after one ranking screen | M: d4d-b16 | MISSING | 3.1 |
| F26 | Boot screens end early when player 1 or 2 presses attack or jump (after 30 frames) | M: d4d-b14 | MISSING | section 2 |
| F27 | Debug mode at power on (P1 up, right, attack, jump, coin 1): DIP bits become cheats, a pause task; the skip-wait task 0x9E38C doubles sleep speed in scripted scenes while a button is held | R: 0xA6C0, 0x9E38C | n/a | section 1 and 10 |
| F28 | Continue countdown: 20 lasts 100 frames, others 60; attack or jump skips one number per press from 17 (mode 0x10) or 18 (in level); coins only add credits | M: d4d-b3 | MISSING | 6.3 (replaces P33 notes) |
| F29 | Join and continue also add stage time (+3:30 join, +4:00 continue, cap 4:00), +600 frames to the wave timer, +1 point to the score of all three players, and cost 1 credit (2 in the "2 coins start" mode, continue 1) | M: d4d-b19, d4d-b3 | MISSING | 6.2 |
| F30 | Tally rules: (HP − 1) * 100 or 20000 at HP 100; kills * 100; final blow 20000; M.V.P. 10000 for the most damage (ties share); count down 100 per frame; end frame = 177 + max_total / 100 + 268 | M: d4d-b7, b8, b5 | STUB | 9.1 (replaces F10 notes) |
| F31 | Name entry exists (game over and ending): 30 glyphs (A to Z, comma, point, RUB, END), 3 letters, 420 frame idle and 20 second limits, profanity filter, parked during the ending | M: d4d-b4, b6, b18 | MISSING | 7 (replaces F20 notes) |
| F32 | The ending is independent of hero and player count (only the explosion randomness differs); fixed cast Mustapha and Mess walk, Jack and Hannah drive | M: d4d-b5 | MISSING | 8 |
| F33 | The demo resets the random generator (G.$4 to G.$E = 0, G.$14 = 0x1C3) and replays recorded input streams (30 streams, 0xC869A) | R: 0x99F10, 0x9A146 | MISSING | 3.6 |
| F34 | Anti-tamper checks (0x6A73A at the title, 0xD860 in the tally, 0x1F18A in the first panel, 0x86B3C in the join task, 0x104F6) never fire on a real ROM: do not port | R | n/a | replaces F23 |
| F35 | The rank at the start of a game depends on the frames the machine waited with a credit (G.$4F6 counted by the system task, bands 720, 3600, 10800) | R: 0x1827A, 0x17F8A | PARTIAL (`rank.rs` has the bands, no feeder) | resolves F12 |
| F36 | Free play: no press-start screen, start works at once from any attract moment, FREE PLAY texts; credits are capped at 9; SERVICE adds one credit to every player | M: d4d-b15, R | MISSING | 3.10, 4.1 |
| F37 | In the select, no two players can hold the same hero, cursors skip taken heroes, joining resets the timer to 10, auto pick presses attack for all at 0, pans 0x00, 0x0B, 0x15, 0x20 for sounds 0x32 and 0x33 | M: d4d-b4 | STUB | 5 |
| F38 | The map panel appears twice: at the end of the tally (slides in, marker blinks 120 frames) and in the intro card of the next episode (slides in, typed EPISODE title 141 frames, slides out, HUD slides in) | M: d4d-b5, b9 | MISSING | 9 |
| F39 | The sound filter: in attract and demo only codes 0xFFxx and 0x0030 reach the sound chip unless the demo sound setting is on (factory: off) | R: 0x610 | MISSING | 3.11 |
| F40 | In-level prompts of absent players alternate INSERT COIN and JOIN-IN when there is no credit (128 frame cycle), JOIN-IN only with a credit (64 frame cycle) | M: d4d-b13 | STUB | 6.1 (extends F18) |
| P57 | Join: attack alone also joins a player with a credit (three chute mode, not in the Japan set); the strip starts on the player's previous hero; a hero is held only by players in strip or play state | M: d4d-b20, b3 | MISSING | 6.2 (was P43 in D4d-front-end-and-story.md)|
| P58 | A joining or continuing player gets HP 100, extra lives (1), a bazooka, 360 frames of blinking invulnerability, drops in at camera X + 0xC0 | M: d4d-b3; user 2026-10-12 | PARTIAL (`item_give_bazooka`) | 6.2 (was P44 in D4d-front-end-and-story.md). **User:** a respawn paid with a new coin (join or continue) comes with a bazooka; a normal respawn after losing a life gets the invincibility and the air strike (W30). D2b: confirm which respawns get the bomb run, the bazooka, or both |
| P59 | The hero can be chosen at three moments only: select, join strip, continue strip | M, R | STUB | 6.5 (answers P41) (was P45 in D4d-front-end-and-story.md)|
| F41 | Story panels: Episodes 4 and 7 show the first unused hero as the speaker, Episodes 4 and 5 use the first player's hero name, Episode 6 shows the first player's portrait (blank for Hannah alone); attack skips any panel | M: d4d-b21, R: 0xE034, 0xE0AA, 0xDFCC | MISSING | 9.3 |
| P60 | The continue screen keeps the level frozen and restores it after a continue (about 41 frames after the press) | M: d4d-b3/k-coin-late | MISSING | 6.3 (was P46 in D4d-front-end-and-story.md)|

### From `design/D4c-camera-flow-transitions.md` (D4c camera, bounds, transitions), 30 rows

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| T01 | Stage 2 (Episode 3) has its own camera controller 0x22398 and mode table 0x2250A; its 10 events at X 0x300 to 0x2D00 are locks (mode 2), mode 8 is the car drive | R 0x22398, 0x2250A, 0x22B9E; M e3footn, car1 | MISSING | the port runs the standard table for it: wrong |
| T02 | Camera never leaves the leftmost player behind; an idle hero holds the camera; push line 0xC0 (1P) or 0xE8 (2P, 3P) unless the event has its own | R 0x20A22; M lag2, lag3 | DONE | |
| T03 | Hero clamp: X in [camX + 0x10, camX + 0x16F], screen depth Z + floor - camY in [0x10, 0x9F], skipped while `G.$506 != 0` | R 0x4FCE; M clampR, d*_*_* | DONE | |
| T04 | Walkable depth per area is 64 px in most areas (screen band 16 to 79), 95 to 114 px in the garage, sewers, cave and yard; table in 3.2 | M d4c-b2/d* | DONE (terrain) | test values for terrain |
| T05 | Drop in position: X = camX + 96 (1P), 64 and 128 (2P), 48, 96, 144 (3P); Z = camY - floor + 32, 48, 64; height floor + 0x100 | R 0x62D6, 0x638C | DONE | |
| T06 | Wave lock ends: type 0 sets `go` at once, type 2 starts the area end script (0x1D95E), type 4 the boss area script (0x1DAC4); 30 frame delay at the end; all heroes must be in the normal ground state | R 0x7B464 | DONE except the scripts | |
| T07 | The area end script releases the wave lock with its own first GO record (96 frames after the wave end when no items lie around, up to 440 with items or containers) | M e1a0, l_e1a0_k2 | MISSING | |
| T08 | Scripted control: 26 scripts, 41 actions, 3 entry points, table 0x1E6FC | R 0x1DB5E and 5.4, 5.5 | MISSING | |
| T09 | Scripted heroes cannot be hurt (enemy attack passes off), cannot pick up when `G.$506 = 1`, drop a carried barrel | R 0x10546, 0x191C4 | DONE (readers), no setter | |
| T10 | Item waits (actions 0x2A, 0x38): the pad works for 120 to 240 frames after a wave end and after a boss death | R 0x1DE6C, 0x1DE86 | MISSING | |
| T11 | Fade = 16 steps of 1 vblank at area end (out) and start (in); mode 0x12 is a hard cut with 2 to 4 black frames; sub-area switch has no fade | R spec 2.4; M e4a0n, n1a1, e2a0n | DONE / PARTIAL | |
| T12 | Sub-area switch (camera mode 8) resets stage time, floor, spawn lists, player count, creates the music object class 96; it never places the heroes and never runs a start script | R 0x20E78, 0xAAE2, 0xACE0, 0x9236C | PARTIAL | corrects D3e (class 96 is not created at every area start) |
| T13 | Area start (0xAA0E): place heroes (0x4E0A), start script (0x1DA10), pools, items leave, rank, time, floor, spawn lists | R 0xAA0E | PARTIAL | |
| T14 | GO burst (class 20): 3 blinks of 30 frames on, 5 off, sound 0x3E; forced at the start of area end scripts; idle hint 180 frames after a lock ended (camera moved < 0x80), then every 600 + 105 frames | R 0x7BB68; M go1, go2 | MISSING | no GO at the end of a wave |
| T15 | Openings: control after 648 (Ep 1), 573 (Ep 2), 321 (Ep 3, 5 to 8), 894 (Ep 4) game frames; the map card is the first 255 | M o1 to o8 | MISSING | |
| T16 | Episode 1 opening pans the camera at 3 px per frame while the heroes stand at X 0x420, 0x410, 0x400; clamp off | M o1, o1p3 | MISSING | |
| T17 | Episodes 2 and 4 start with the heroes riding the Cadillac (state 0x432) and jumping out | M o2, o4 | MISSING | |
| T18 | Episode 4 opening waits for `G.$4FC` to clear (set at the start, cleared at gf 888) | M o4 | MISSING | |
| T19 | Boss death chain: 0xACEE, slow motion, class 22 calls 0x1D95E, `G.$4E3` after the script, tally, story panel, next opening; 94 + 278 + 951 game frames on Vice | M b1c | PARTIAL | |
| T20 | Walk off after a boss: wait for items, barrier, pose (sound 0x18), hold right 300 with clamp off (not Episode 1 and 8) | R scripts 3, 12, 20, 24 | MISSING | |
| T21 | Car section: radio makes car (HP 120) at 16, stops at 105, heroes jump aboard 284 to 332, `G.$518 = 1` and camera mode 8 at 364, 11 px per frame, clock frozen, ends when the car's HP is 0 (camera mode 3 at that X) | M car1, car3 | MISSING | |
| T22 | Camera mode 8 chase: camX += clamp(car X - (camX + 0xC0), 8, 20) | R 0x22A68 | MISSING | |
| T23 | Lift ride restarts from rest after every lock (sound 0x86, shake 2, acceleration 6/256 per frame squared up to 6 px per frame); stops at Y 0x3000, 0x1E00, 0x1400, 0x800; end at 0 | R 0x210E0; M e7a1n | PARTIAL | |
| T24 | Row scroll words equal the formula of the stage spec (verified at 3 camera positions) | M rows_* | MISSING | |
| T25 | Screen shake: offsets -s, +s (3), -(s-1) (3), ... +1 (3), length 6 s - 2 frames; 29 requesters, strengths 1 to 4 | R 0x222D6; M shake1 to 4 | STUB | |
| T26 | Stage clock and rank are frozen while `G.$4DD != 0` or `G.$4D5 = 1` and in car mode | M e4a0n, car2 | DONE (`rank.rs`) | |
| T27 | `G.$4D4` (stage clear flag) revives a hero in the death sequence with 1 HP | R 0x1BF82 | MISSING | [I] |
| T28 | Probe limit: `jump N A X` keeps wave controllers before X; stale controller can send `go` | M hallwin, e1a1n | n/a | |
| T29 | Original hang (game frame counter stops) with kill-all held in Episode 7 | M e7a1, l_e7a2_k19 | n/a | investigate |
| T30 | Ep 8 area 3 start row of table 0x4E56 is missing; a sub-area never uses it | R 0x4E56, 0x4FCE | n/a | |

### From `design/D3b-dinosaurs.md` (D3b dinosaurs and flyers), 14 rows

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| DS01 | Monsters (A22, A24, A29, A38, A82) live in the hostile list 0x6B64; they hurt heroes (scaled by type and level) and enemies (raw damage, nobody gets score); no monster hurts a monster | R 0x11A9E, 0x11CEC, 0x11E56; M m-vs-thug, s3-vs-enemy | MISSING | D3b 1.2 |
| DS02 | Dinosaurs give no kill score, no drop, are not counted by waves (`+7D = 0xFF`), are not touched by kill-all; hits score 100 / 200 / 400 | R, M z-kill, m-kill, h3-kill, s8-kill | MISSING | corrects D04 ("walks away, gives no score") only in detail |
| DS03 | A sleeping Shivat has `+0 = 2` and cannot be hit by anyone; a dinosaur hunter (A79) wakes it with its third swing (helper 0x533FA, no HP loss) | R 0x533FA, 0x5F9F8; M s4-wake, s5-wake | MISSING | corrects D02 (poachers' hits do not wake it; hunters do) and D13 (bullets: not seen) |
| DS04 | Hostile model names: MICHI, KIMAMA, MATOWARI, OKORI, KYOURAN, MANZOKU, COLOR, DIFFENCE; COLOR flash 16 toggles (about 88 frames) then angry; any hit on a neutral dinosaur starts it | R 0x490CE, 0x4F1C4; M h3-hit, s4-wake | MISSING | D03, D04, D12 |
| DS05 | Neutral Hopper: after 600 frames in MATOWARI a 1 in 16 roll every 10 frames turns it angry, after 1200 it leaves; angry: 900 frames then 2 in 16 per 10 frames to leave. Shivat MATOWARI 900 frames, OKORI 1800 frames, same rolls | R, M h3-neutral | MISSING | |
| DS06 | Zeke: HP 20, damage x2 (type factor 64); one punch or jump kick kills; patrol heights 74 to 203 (overshoot because the altitude test runs once per flap cycle); standing punches reach him only when y <= about 72, jump kicks y <= about 100 | R 0x2BECE; M z-hit-*, z-solo-* | MISSING | answers D09 "hit only by jumping" |
| DS07 | Zeke attacks: swoop 0x5B, triple peck 0x5C, dive grab 0x3B (hold 31 frames, 18 damage); attack roll every 180 frames with 9, 16, 24, 30 in 32 by level band; type table by altitude index | R 0x606D4, M z-force-*, z-solo-* | MISSING | numbers in D3b 2 |
| DS08 | Zeke's grab only works on a grounded hero in normal control; a jump at the right moment dodges it; the carry goes toward world X 0xC0 (original bug: reads RAM 0) | R 0x60C8C, 0x6108A; M z-force-c-jump, z-grab-far | MISSING | |
| DS09 | Zeke leaves straight up when `camX >= +5C` (checked only in patrol), dies by flying away, never explodes; two Zekes pick different heroes (`+$116`) | R, M z-leave, z-kill | MISSING | |
| DS10 | Mack: charger with strides (32 frames moving, 22 still, 122 px), horns hurt only in the middle of a stride, side chosen by the record X against camX + 0xC0, `+5B` = number of passes, 62 frame wind up, calm walk off | R 0x5341C; M m-solo-l0 | MISSING | corrects D07 |
| DS11 | Rock Hopper is spawned only by Vice's init (variant 0, partner intro synchronised with Vice's `$6`); the scripts place it with variant 0x200 in Episode 5; angry Hopper blocks 12.5 percent of frontal hits; berserk after 5 hits | R 0x425C2, 0x4DCC6, 0x4F3E8; M h3-vice | MISSING | corrects D08 |
| DS12 | Brontosaurus is one huge foot (A22) with a leg helper (A23): hover 180 frames over the nearest hero in its 80 px home column, 60 frame wait, wind up, slam 30 damage with knock down and screen shake 4, rest 30 frames, repeat (about 380 frames); cannot be hit | R 0x47C42; M bronto-0 | MISSING | corrects D11 |
| DS13 | Unused creature art: A12 = opening movie raptors, A28 / A37 / A39 / A44 / A47 = test mode viewer sprites, A51 = carcass spawned by class 5, A46 / A48 = flock bird and trail | R spawn search, D4d | n/a | answers N2 of the asset catalog |
| DS14 | Hopper and Shivat burn when hit by a burn reaction (record reaction 0xA or 0xC): flames and palette 0x84 | R 0x4EE1A, 0x48D8A | MISSING | untested, open item 2 |

### From `design/D2b-player-state-rules.md` (D2b player state rules), 18 rows

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| P61 | Lost life respawn timeline: death flight 183 frames (dead vectors, half speed), 61 frame blink, respawn start R, 60 frames parked above the screen, drop at R + 61 (31 frames fall), control at R + 92; HP 100, lives - 1, stage clock + 3:30, wave timer + 600 | M: resp_jack/mustapha/hannah/mess; R: 0x18B38, 0x1BF42, 0x10482 | PARTIAL | D2b 2.1 to 2.3; corrects P21 |
| P62 | Respawn air strike: 5 bombs 12 frames apart in a fixed screen pattern (not aimed), explosions 59 frames after each bomb, 26 frame explosions, white blast of the 5th bomb hits the whole screen; every hit on an enemy is a flat 30 HP with an HP floor of 1 (never kills), once per enemy in practice, +100 points each to the respawning player; containers break (+500), TNT ignites, items and players are untouched, pool B breakables not hit | M: resp_*, strike_thugs, ty_*, hp_floor2, objs_*; R: 0x7B65E, 0x29EC4, 0x27F70, 0x2A534, 0x111C6 | PARTIAL (pool E side and enemy pass DONE, pool B 18 MISSING) | D2b 2.5; corrects W30 and P40 |
| P63 | Bazooka (4 rockets, vanishes 600 frames after a drop) only for respawns with `+$07 == 0`: join with a coin and continue with a coin; they also get the strike, the bubble and the invulnerability; lost life respawns get no bazooka | M: join_p2, cont_level, cont_mode against resp_*; R: 0x18BC0, 0x65A4 | DONE (bazooka), MISSING (rest) | D2b 2.6, 2.7; settles P58 |
| P64 | Respawn speech bubble: pool C class 14, text "They're gonna pay for this!" for one player and "Need a hand?" for two or three, 239 frames, then a "1P"/"2P"/"3P" marker for 120 frames, follows the hero, freed when he is no longer invulnerable | M: resp_jack, resp_2p, join_p2; R: 0x93B28 | MISSING | D2b 2.4; corrects P22 |
| P65 | Respawn invulnerability is `+$00 = 2` with `$EC = 360` counted only after the 61 frame wait: 421 frames from R; shown by sparkles (pool C 23 every 4th frame), not by sprite blinking; TIME OVER still kills the hero | M: resp_jack, cont_level, timeover_inv; R: 0x18CD4 | PARTIAL | D2b 3; corrects P19 |
| P66 | Each death: rank minus 4 (3 if a drop was less than 600 frames ago), calm period 1200 frames (aggression 0), rank speed -4 (1 player), -3 (2 players), -3 or -2 (3 players), player count recount at game over (level column changes) | R: 0x184E2, 0x655E; M: rank_death | MISSING | D2b 2.2; D4b to check |
| P67 | Lie time, mash meter and dizzy tables use row 0 only (level band byte read bug): lie 6 (37.5 percent), 16 (50 percent) or 31 (12.5 percent) frames, mash meter 15, 20 or 25 (right or left press counts 3), no lie time by level | R: 0x1D17E, HANDOFF; M: lie_* | DONE | D2b 4.4; corrects P15 |
| P68 | Dizzy needs the knockdown to be the 4th or later hit of a chain (gaps of 60 frames or less): 75 percent at 4, 100 percent at 5 or more, HP 30 or more, immunity `$80 + 1800` frames; dizzy lasts 40 to 180 frames, 15 to 125 stars to mash, the hero can be hit while dizzy | R: 0x1D0FA, table 0x1D166; M: dizzy4, dizzy5 | DONE (stars effect MISSING) | D2b 4.6; corrects P16 |
| P69 | Hero damage factor table 0x2C88E by hero id: Jack 32, Mustapha 34, Hannah 35, Mess 30; enemy to hero order of steps: type by level, low HP random cut, hero factor, HP factor 25 to 33, mercy; measured 14, 14, 15, 13 for a base 8 jab at level 31 | R: 0x6422; M: dmg_*_L31_hp100 | DONE | D2b 5; settles P18 |
| P70 | Extra life lists are absolute scores: 300,000 and 700,000, **500,000 and 1,000,000 (factory)**, or 1,000,000; one life per step; sound 0x4A; the "1UP!" text (pool C 14 variant 2, 180 frames) appears at the flagged frame of the pose | R: 0x6B98, 0x6C0A, 0x6C16, 0x6C22, 0x93BF8; M: oneup | PARTIAL | D2b 1.3, 1.4; replaces the thresholds note of P45 |
| P71 | HP is refilled to 100 at the start of every episode (not between areas); lives and score carry over | R: 0xAC4E; M: hp_carry | DONE | D2b 1.1 |
| P72 | There are no floating score numbers: kills, hits and pick ups only change the HUD score; pick ups show a message in the HUD message area (FOOD icon, name, `10000 pts` at full HP or `GOOD !!` when healed); the rank text under the score shows the place in the ranking table | M: pick_*, oneup | PARTIAL | D2b 6; answers the "score pop-ups" question |
| P73 | Scoring numbers: hit points from the record code, kill by name table, container +500 (plus the hit), treasure arrives 11 frames after the press, kill score about 79 frames after the killing blow; no score while scripted control with `G.$4DE == 1` | M: score_*, pick_*; R: 0xFFC8 | DONE | D2b 6 |
| P74 | TIME OVER sets HP 0 for every hero with a state byte below 4, also invulnerable ones; the death flow and the respawn follow (lost life rules, no bazooka) | M: timeover_*; R: 0x103FE, 0x1046A | DONE (words MISSING) | D2b 7 |
| F42 | The life counter in the HUD block is one tile whose code is the spare life count (digits, then letters from 10: A, B, ...), redrawn only by block redraws (respawn, extra life, area start) | M: lives_hud, lives9_hud | DONE | D2b 1.2 |
| F43 | At HP 15 or less the name, lives and score rows of the player block change palette every 6 frames (red and white); the bar does not blink | M: hp_bar; R: 0x1667E | DONE | D2b 1.2; refines P25 |
| W39 | Air strike side effects: containers break (+500), TNT barrels ignite (HP 50 to 20, fuse, explosion about 110 frames later), items and pool B breakables untouched | M: objs_ex, objs_blast, objs_tnt; R: 0x120D2, 0x10CD0 | PARTIAL | D2b 2.5 |
| W40 | Every explosion of kind 2 shakes the screen with strength 3; the bomb trail is pool C class 7 every second frame | R: 0x27FCE, 0x29F7C | PARTIAL | D2b 2.5; D3f for the effect classes |
