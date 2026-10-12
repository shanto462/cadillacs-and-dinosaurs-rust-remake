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
| B07 | Slisaurs: 2 for 1 player, 3 for 2 or 3 players | W:aqm, U (2026-10-12) | MISSING | **User:** the Slisaur boss (the user calls it "Slithors") throws blades, moves really fast, and leaves an afterimage trail when it moves. D3d must cover: the blade throw (object class, path, damage, count), the speeds and movement patterns, the afterimage effect (how many copies, delay, palette or blink, which routine draws it: probably earlier positions kept in the object and drawn again), the count by player number, HP and phases (pool A class 71, starters class 41). **User (2nd note):** Episode 3 has only ONE Slisaur; the second-last episode (Episode 7) has two or three by player count. The spawn scripts only start Slisaurs in Episode 7 (three pool B class 41 starters, var 0 / 4 / 8, at trigger X 0x260 of the first area), so the Episode 3 one must come from a run-time spawn (another class or a boss script): D3d must search the code for every creator of class 71 and check Episode 3 in MAME. Likely answer (G06): the single fast blade thrower with afterimages is **Slice**, the Episode 4 boss; the Slisaurs of Episode 7 are reptile versions of Slice |
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
| F07 | Opening scenes: Episode 1 walk in and fight, Episodes 2 and 3 car arrival | M, U (2026-10-12) | MISSING | **User (seen in the port after M0):** Episode 2 is broken: in the original the hero arrives in Jack's car, jumps off, and then the episode begins; the port starts with the hero already standing. Needs the scripted control (D4c T-C1, script interpreter 0x1DB5E), the car object (pool A class 16, also the Episode 3 truck), and the opening timeline (control at 573 game frames, T15). Milestone M2 / M3 |
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
| T16 | Episode 1 opening pans the camera at 3 px per frame while the heroes stand at X 0x420, 0x410, 0x400; clamp off | M o1, o1p3 | MISSING **User (2026-10-12):** the hero does not start at the far left; the camera pans on, and inside the building the hero is surrounded by thugs while the first boss (Vice T.) orders them to beat him up; the hero does the special (mega crush), all the thugs die, and the episode begins. Pieces: the pan (this row), opening objects pool A 54 / 55 (ported) and 56 / 57 (not ported, probably Vice and the caption), the scripted player states 0x2A / 0x2C and the scripted special 0x1E2EE (P54), the caption banners (pool B 66, master 74). Harness scenario `intro` covers frames 0..600 |
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

### From `design/D2c-weapons-multiplayer.md` (D2c weapons and multiplayer), 27 rows

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| WM01 | The 15 weapon types and names: 0 RIFLE, 1 GUN, 2 KNIFE, 3 UZI, 4 M-16A1, 5 SHOT GUN, 6 BAZOOKA, 7 ROD, 8 STICK (the broken rod), 9 B. KNIFE, 10 CLUB, 11 TORCH, 12 DYNAMITE, 13 GRENADE, 14 STONE (3 sizes); kinds 0x201 thrown, 0x202 knife-like, 0x203 charge swing, 0x204 to 0x209 guns | R: 0x24D6C, 0x23D0B; M: d2c-big pistol_*, rifle_* | DONE (item); names in comments wrong | D2c sections 1, 12; replaces the guesses of P27 |
| WM02 | HUD weapon display: guns show a picture tile, the ammo number (red at 1), the name and a cartridge bar; the other 9 types show the "ARM" tile and the name; one block per player | R: 0x2389C; M: d2c-f hud_*, d2c-g hud_3p | STUB | D2c 12; task P-D2c-5 |
| WM03 | Charge swing and critical hit: kind 0x203 weapons wait while attack is held (counter to 60); damage x1.0 / 1.25 / 1.5 / 2.0 for counter up to 15 / 30 / 45 / 60, applied to the integer record damage; club hits 0x58 + 0x60 for Jack and Hannah, only 0x60 for Mustapha and Mess | R: 0x19C62, 0x11262; M: d2c-big club_hold* | DONE | D2c 5.5 |
| WM04 | Knife and stick swing only when an enemy is in the reach zone (112 / 128 / 120 / 104 px, depth +-16 or 24), else they are thrown; the thrown knife is used up on a hit | R: 0x6248, 0x62AE; M: d2c-f reach_* | DONE | D2c 5.1 |
| WM05 | Rod breaks into the stick: roll per hit 19, 38, 69, 88, 81, 6, 12, 94 percent (table 0x2CBA0); stick = knife rules | R: 0x259B2; M: rod*_* | DONE | D2c 5.2 |
| WM06 | Wear and break: +0x20 per use (club 0x30); a weapon breaks only when dropped by a hit or death and landing, chance 0, 19, 38, 50, 63, 75, 88, 100 percent by wear; no check for swap or special drops; butcher knife never; a bazooka without rounds always; floor knives: the 4th and later break after 600 frames; a dropped weapon lies 3600 frames; placed and enemy dropped weapons never expire | R: 0x248F6, 0x247FA, 0x2463C; M: d2c-f wearff_*, floor_knives_long; d2c-big drop_* | DONE | D2c 8 |
| WM07 | Drop rules by hero: hit drops all except Hannah keeps the 5 guns and Mess keeps the bazooka; death drops (with wear check); special attack discards; swap; last round of a gun lets go (the M-16 and the empty rifle differ: M-16 stays as a club, the rifle is let go) | R: 0x1177A, 0x19B48; M: ffd_*, special_discard_*, die_with_weapon | DONE | D2c 2.2 |
| WM08 | Guns: bullet speeds 8 px (pistol, rifle) and 12 px (Uzi, M-16), pellets 10.5 to 13 px with vertical spread, shell accelerating; records 0x4E, 0x50, 0x46 to 0x4C, 0x5E, 0x52, 0x54; Uzi and M-16 fire every 7 frames; auto aim steers only the vertical speed (burst climbs from -336 by +48 per round); bullets fly straight in depth (hit window about 12 px); 4 of 6 pellets hit from 60 px | R: 0x28814, 0x2A484; M: d2c-big, d2c-f aim_pistol_dz* | DONE | D2c 4 |
| WM09 | Bazooka: recoil by hero (Jack 8, Mustapha 10, Hannah 12, Mess 6 px per frame start speed), shell 40 + blast 10; the shell does half damage to Slice, Vice, Butcher, Morgan, Morgue, Slisaur, Tyrog, Fessenden and double to Hogg; pellets x0.75 against Vice and Walther | R: 0xA32FA, 0x19CD8; M: bazooka_* | DONE (data driven) | D2c 4.1, 7.3; D2b missed the ids 0x52 and 0x5E |
| WM10 | Dynamite lights itself when it comes to rest (4 stages of 15 frames, explodes after 60 frames: 114 to 173 frames after the throw); a dynamite dropped by a hero is lit on landing and explodes (not hurting its owner); a dropped grenade is not armed; grenade explodes on a hit or the 3rd floor contact and makes 3 explosions 10 frames apart (impact, -16/+16, +32); stones: 3 sizes, 28 damage, vanish after 4 bounces | R: 0x24724, 0x26182, 0x262BC, 0x263CC; M: dynamite_*, grenade_*, stones*_, dropped_* | DONE | D2c 6 |
| WM11 | Throw tables: flat 9.5 px per frame (12 px from a run) for knife, stick, empty guns, stones; lob for dynamite and grenade (4 px, 2.5 px up); thrown containers: velocity table by hero and variant, +500 at the throw, 1000 per hit, barrel flies 4 px nearer (Z - 4); thrown explosive barrel explodes where it lands | R: 0x26534, 0x2986E, 0x29860; M: barrel_*, tntbarrel_jack | DONE | D2c 6.1, 9 |
| WM12 | Pick up: lowest kind wins (ammo box before weapons, food before the box); weapons 40 frame lock; two players pressing at once: the first updated player takes it, the other jabs; ammo box refills the held gun, +1000 when full or when nothing to refill, and the next gun is filled | R: 0x5AA2, 0x284E2; M: pick_lowest, pick_lockout, ammo_*, pick_same* | DONE | D2c 2.1, 10 |
| WM13 | Torch: swing 0x38 with the burn knock down (reaction 0xA), hit at +12, no damage over time on a thug; glow overlay pool E class 6 flickers between Z + 2 and Z - 1, flame loops 6 and 4 frames; torches only in Episode 5 areas 0 and 1 | R: 0x29B06; M: torch_* | STUB (overlay) | D2c 5.4 |
| WM14 | Weapon attack records 0x1C to 0x60 with damage, stun, reaction, hit stop, sound, score (table in 7.1); Hannah's knife 0x2C and stick 0x2E flinch only | R: 0xA26FC | DONE | D2c 7.1 |
| WM15 | Run and attack with a club, gun, rod, torch or butcher knife is the normal dash attack and keeps the weapon; with a knife, stick or dynamite it is a run throw; jump and attack with any weapon is the normal low jump attack | R: 0x1A174; M: runswing_*, jumpweapon_* | DONE | D2c 3 |
| WM16 | Friendly fire by fists measured with 3 and 2 players and 0 to 4 enemies: windows -1..1, 0..1, 0, off at 3; one swing hits every friend in the box; hp floor 1; specials immune; hit score none | R: 0x10664; M: ff3_n*, ff2_n*, ff3_multi, ff3_hpfloor | PARTIAL (gate constant 3) | D2c 14.1; corrects P44 only in detail |
| WM17 | Friendly fire by attack objects (pass 0x10DCC): always on, no enemy gate, 1 HP per hit at exactly the same depth, never against the owner, ids 0x6E and 0x6C skipped; enemy owned objects use the record damage with no depth test and can kill; dropped weapon without hero exceptions | R: 0x10DCC; M: ffi_*, dropped_dynamite | STUB | D2c 14.2; task P-D2c-2 |
| WM18 | Tag team trigger: counters per other player, reset by the initiator's input and by the partner's invulnerability, class, action or input, paused by the initiator's own invulnerability; partner must have `+$00` bit 0 set; X -32 to 31, Z 0 to 15; lower number first; any pair of three; weapons ignored, heavy objects block | R: 0x6728; M: tie_*, tie3_*, tiez_* | MISSING | D2c 15.1, 15.2 |
| WM19 | Jack's partner run attack is id 0x1A (the partner hero's own row) only with a direction held; without it a jab; jump gives the running jump | R: 0x1A330; M: tierun_*, tierun2_* | MISSING | D2c 15.3 |
| WM20 | Dive window by hero (frames after the jump press): Jack 22 to 24, Mustapha 21 to 25, Hannah 22 to 25, Mess 22 to 23; the triangle kick is accepted from 14 frames after the wall jump, low attack for Hannah, dive for Mustapha | M: dive_*, diveb_*, wja_*, wjb_* | DONE (data driven, check) | D2c 18; corrects P47 |
| WM21 | Players never block or push each other (no contact in walk, run, jump); update order P1, P2, P3; draw order by Z then address (items, effects, P1, P2, P3, pool B, pool A) | M: pp_*; R: 0x13FA6, 0x18834 | DONE | D2c 17 |
| WM22 | `G.$4EF` recount: by 0x655E at area start, respawn, death end (61 frames after the death sequence starts); recomputed from the scripted mask during scripted control; 26 read sites | R: 0x655E, 0x1DB40; M: dead_* | DONE | D2c 13 |
| WM23 | Script records for 2 and 3 players per episode, wave thresholds, boss HP by count (Vice 350 / 550 / 650 at rank 0), Tyrog class 4 has half of its table row (256 / 288 / 352), one extra Slisaur for 2 and 3 players | R: stage_scripts, 0x31E14, 0x81958; M: boss_*, cnt_* | DONE (1 player compared) | D2c 16.2, 16.3 |
| WM24 | Level column by count and the rank clamps (rank 0: level 0 / 4 / 8; rank 12: 10 / 12 / 14); rank speed delta on a continue by count | R: 0x18592, 0x18542 | DONE | D2c 16.4 |
| WM25 | Weapon sources: placed weapons only for torches (Episode 5), stones (Episodes 4, 7), rifle, shotgun, 2 M-16 (Episode 4 area 2); everything else from drop rows, Blade and Razor knives (only while fewer than 2 knives lie around), the Butcher's two knives (class 52), debris stones (not seen), the join bazooka | R: scripts, rows; M: drop_enemy_life | PARTIAL | D2c 11 |
| WM26 | Enemy thrown dynamite and grenade: created lit with owner 0xFF (blast kind 3, attack 0x6A, record damage, can kill, no depth test) | R: 0x24FDC, 0x27FEE | item side DONE, throwers MISSING | D2c 6.2, D3a, D3c |
| WM27 | Thrown or lifted containers: +500 at the throw, +1000 per hit, no break bonus after a hit; a tire stack was not lifted in a test (open item 15); stone and boulder need no stick direction to lift | R: 0x28F76; M: barrel_*, tire_jack | DONE | D2c 9 |

## J. Leads from a web summary written by GPT (given by the user, 2026-10-12; NOT verified)

Source: `gpt_suggestions.md` in the repo root (a summary of walkthroughs, not deep). Each row is a lead for the named design task to confirm or reject in MAME and the code. Rows already covered elsewhere are not repeated.

| ID | Mechanic (as the summary says) | Source | Port | Check in |
| --- | --- | --- | --- | --- |
| G01 | Episode 1: the hero leaves the building by jumping or falling down to the street (scripted area change); a large enemy falls into the new area and another bursts through a door | W:gpt | UNKNOWN | D4c (area scripts), D3e |
| G02 | Vice T.'s intro: he provokes a chained Rock Hopper into the fight; Vice rushes, moves fast, summons gang members | W:gpt | UNKNOWN | D3c, D3b |
| G03 | Butcher: dual blades, jumping body slam, charges; his blades can be knocked away and picked up as weapons; found cutting a dead dinosaur (carcass A51) | W:gpt | UNKNOWN | D3c |
| G04 | Episode 3: the radio comes from an early breakable barrel; in the car the players move between lanes, speed up and slow down, run over gang members, dodge rolling barrels; the car shields the heroes from most attacks; Hogg rides a motorcycle and throws grenades; if the car is destroyed the heroes fight on foot | W:gpt | UNKNOWN | D3c (with W15) |
| G05 | Episode 4 opening: the Cadillac crashes through the blocked garage entrance, gang members react, Slice dodges the car and retreats, the hero gets out | W:gpt | UNKNOWN | D4c, D3c |
| G06 | Slice (Episode 4 boss, single): very fast with afterimages, throws returning boomerangs, several projectiles at once, quick repositioning. **Probably the boss the user described as the single Slisaur** (user said Episode 3; Slice is in Episode 4 and the Slisaurs of Episode 7 are reptile versions of Slice) | W:gpt | UNKNOWN | D3c (Slice), D3d (Slisaurs) |
| G07 | Episode 4: Walther as a mid-boss, motorcycle enemies crossing the arena, enemies from doors and high places, a fence the player breaks to reach the next area (fence art = pool B class 1), floor vents | W:gpt | UNKNOWN | D3a, D3e part 2 |
| G08 | Falling rocks in Episode 5 leave stones the player can use as weapons | W:gpt | UNKNOWN | D3e part 2, D4a |
| G09 | Bludge jumps around, attacks at range, and may eat items left on the ground | W:gpt | UNKNOWN | D3a |
| G10 | Episode 5 story scene: the old villager starts to explain, Morgan shoots him; Morgan fights with fast gunfire and thrown weapons, then turns into Morgue (head-first charges, heavy knockback) | W:gpt | UNKNOWN | D3d, D4d |
| G11 | Episode 6: doors in the mine that the player must break; the brontosaurus foot casts a warning shadow before it stomps and can crush enemies | W:gpt | UNKNOWN | D3e part 2, D3b |
| G12 | Tyrog: a parasite that changes hosts: green host (bladed limbs), orange host (lunges, fireballs), blue host (mix and projectiles) | W:gpt | UNKNOWN | D3d |
| G13 | Episode 7: a bridge, motorcycle riders and flying creatures at the start; in the library enemies come out from the bookshelves; Fessenden talks from the computer screen before the Slisaurs | W:gpt | UNKNOWN | D3e part 3, D3d |
| G14 | Episode 8: the glass tanks break and release dinosaurs and mutants (see W21, the user's tanks 1, 3, 5, 7), several Bludges, Tyrog 2 as a mid-boss; pipes in the corridor blow fire or exhaust that hurts | W:gpt | UNKNOWN | D3e part 3, D3d |
| G15 | Fessenden: first form like Morgue (charges, close strikes); final form a huge pink mutant with two reptile heads and the doctor's body (claws, projectiles, grabs, multi-part) | W:gpt | UNKNOWN | D3d |
| G16 | Ending: self-destruct, the heroes run, Hannah falls and Jack goes back for her, Mustapha and Mess escape, the lab blows up, a Cadillac comes from behind with Jack and Hannah alive (compare with the ending D4d measured) | W:gpt | UNKNOWN | D4d |
| G17 | Dash attacks per hero: Jack a low slide, Hannah a flying knee, Mustapha a long flying kick, Mess a flying body dive | W:gpt | UNKNOWN | D2a (check) |
| G18 | Food and treasure names: pizza, burger, hot dog, steak, meat, cake, chocolate, coffee; jewels, coins, diamonds, a dinosaur egg | W:gpt | UNKNOWN | D4a |
| G19 | Continue screen: an armed gangster seen in first person with the countdown | W:gpt | UNKNOWN | D4d (check) |

## K. More notes from the user (2026-10-12)

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| U01 | Some enemies pick up weapons and items lying on the ground and go away with them (**the user: it is PUNK**, the small quick thug). The ROM name table gives PUNK to two name ids, 0x08 and 0x1D. Pool A class 20 (PUNK / THUG, 0x457B0) is ported and its port has **no item pick-up branch** (its step runs matched MAME, but no item lay near a Punk in them), so either class 20 has an unreached branch or the second PUNK id (0x1D) belongs to another class that steals. **User:** Punk and Thug are the same type with almost the same look (both are class 20 variants), so the branch is most likely in class 20. A dropped weapon can be lost this way | U | UNKNOWN | D3a: find which classes pick up items (the enemy state name list has HIROI = pick up, `re/specs/enemy/SPEC.md` line 549), what they take, whether they use the weapon, how they leave the screen, and whether hitting them makes them drop it |
| U02 | A weapon may vanish if the hero is beaten up while holding it, "maybe after 2 times" (user not sure) | U, D2c section 8 | DONE (item side, ported in task 2) | Answered by D2c: a weapon breaks only when the hero drops it because of a hit or death and it lands; the chance comes from its wear (+0x20 per shot, swing or throw): 0 percent when new, then 19, 38, 50, 63, 75, 88 and 100 percent after 7 or more uses. So an often used weapon usually breaks on the first drop, a new one never does |

### From `design/D3a-humans.md` (D3a human enemies), 26 rows

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| HU01 | The only thing that wakes a sleeping dinosaur is a dinosaur hunter (class 79): its third swing pokes it (0x533FA); the dinosaur then flashes 88 frames, turns angry and hurts the nearest hero or enemy, first the fleeing hunters | R 0x5F9E6, 0x533FA; M nat2-hunt, nat5-hunt, hun-v400 | MISSING | answers D02 and D13 for Episodes 2 and 5; Episode 6 has none |
| HU02 | A hunter is freed in its first frame when no sleeping dinosaur (class 24, 29, 38 with `+$C0` negative) exists; wave hunters spawned after the wake vanish at once | R 0x5F6CA; M hun-none, hun-hopper, hun-mack, nat5-pulse | MISSING | |
| HU03 | After the poke the hunter walks left (facing left) until 0xC0 px left of the camera and is freed (no drop, no score); it never attacks a hero; a killed hunter scores 1,000 and drops row 0x22 / 0x23 | R 0x5FA5A, 0x5FA80; M hun-v400, ks-hunt2 | MISSING | |
| HU04 | Poacher bullets, knives, kicks and punches never hurt enemies or dinosaurs (the bullet flies through them) | R passes 0x115B0; M bsp-* | MISSING | |
| HU05 | Poacher rifle: class 59 bullet var 0, 7 px per frame, attack 0x0B (15, knock down), muzzle flash class 57, 1 to 3 shots 69 frames apart, 60 to 95 frame pause, far decisions only while `+$B8` is negative and X class 4 to 5 (121 to 144 px most), the bullet keeps flying after a hit | R 0x3EA2A, 0x57FB4, 0x57F62; M rf-*, bsp-ctrl | MISSING | D3a 4.5 |
| HU06 | Poacher dash tackle (attack 0x2C, 15, knock down B, 55 frames at 3.5 px per frame) and kick (0x0A); X distance classes are 24 px steps (table 0x32988) | R 0x3F88E, 0x3F82A; M rf-d168, p13-ent08 | MISSING | |
| HU07 | Poacher entrances 0 to 0x12 (stand 1200 frames, rise with fade 0x8B, jump in from above, run in with a dash, ambush jump in) | R 0x3DF54; M p13-ent* | MISSING | Episode 6 jungle ambush = entrance 0x12 |
| HU08 | Enemy bullets and thrown blades and the mace read a stale `+$84` for the damage scale (27 at level 31 for a bullet, 21 for the mace) | R 0x57FB4, 0x5A47C; M bsp-lvl31, lash-v0-l31 | MISSING | keep per slot `+$84` or fix a row |
| HU09 | Bludge eats floor items (lowest kind first, claim `+$AF`, queue of 8), spits all of them on death and the last one when hit while eating | R 0x3B006 to 0x3B0D8; M bl-eat-a7, bl-death | MISSING | the only enemy item pick up |
| HU10 | Bludge: entrance 0 hops, 4 crouches, 6 leaps 217 px; attacks charge 0x25, lunge 0x24, tongue 0x26, acid spit class 7 (0x27, 10, push) | R 0x393D6; M bl-v*-l* | MISSING | |
| HU11 | Lash T.: three objects, mace thrown from 160 px (attack 0x2A flight, 12 at level 0, 21 at 31), HP 100 to 300 | R 0x358B2; M lash-* | MISSING | |
| HU12 | Alternate Blade (class 63): dash 0x1C, one thrown blade (0x19, 12), run off the near side, freed off screen, HP 36 at every level, +1,000 | R 0x5AA6E; M a63-* | MISSING | a wave of them ends without kills |
| HU13 | Bomb carriers (class 64 with class 77): HP 20 fixed, hop and run, throw dynamite (pool E explosion kind 3, 8 to 19 on a hero), run when hit and drop a weapon item (type 13 or 12); the alternate Wrench T. gives 80,000 points | R 0x5AFC2, 0x5EDF4, 0x5EFAA; M ks-a64v2, a64-e3-hit | MISSING | Episode 3 (4), 4 (3), 5 (4), 8 (7) |
| HU14 | Barrels (65), rocks (66) and bikers (70) are monsters: they hurt heroes and enemies (10 raw to enemies, 17 for the bike kick), HP 8, a hero's hit breaks them, they ignore kill-all; barrel hops (vx 2 px, vy 3 px, 1 in 16 higher), rock bounces twice then rolls toward the camera, biker waits 30 frames then 4.5 to 7 px per frame | R 0x5B8B2, 0x5BB8A, 0x5C356; M bar-*, rock-trace, bike-* | MISSING | |
| HU15 | Brontosaurus foot and barrels hurt poachers in their way (foot 30 knock down and 1 touch, barrel 10) | R monster passes; M h-foot-poacher, nat6-pulse | MISSING | the Episode 6 "poachers versus dinosaur" answer |
| HU16 | Pass 0x10DCC: attack list objects hurt heroes: 1 HP friendly fire (same Z exactly), full record damage for neutral attack objects (explosions) | R 0x10DCC | MISSING | corrects "0x10DCC = player vs player" in SPEC 10.1 |
| HU17 | Pass 0x10CD0: heroes break the six pool B breakable classes (a hit always breaks) | R 0x10CD0 | MISSING | with D3e W31 |
| HU18 | Class 16 and 17: the Cadillac of Episodes 2 and 4 carries up to three heroes (`G.$504`), brakes from camera X + 0xA0, sets `G.$505`, stops at x 192 (Episode 2) | R 0x420EA; M car-trace | MISSING | D4c T17 |
| HU19 | Classes 27 and 43: the Episode 4 opening actors: three crouching men and the orange leader with three banners and the end flag `G.$4FC` | R 0x4D900, 0x55F2A; M e4-open | MISSING | D4c T18 |
| HU20 | Class 35 becomes class 36 (Morgan) by rewriting its class word; it fires class 59 variant 5 bullets (8 px, 30 frames) at the villager (class 45) | R 0x51658, 0x51512 | MISSING | D3d |
| HU21 | Walther is class 18 variant 2: Vice's AI without the intro, normal HP table, kill-all, clamp, score 8,000; first unit of the Episode 4 garage wave | R 0x42590; M wal-l0 | MISSING | answers G07 (partly) |
| HU22 | Poacher gate quirk: the far decision reads the X class `+$A5` of the previous decision; the dash needs the hero more than 100 px away; close in needs `tz - z` from -7 to +2 and far shots need 0 to 7 | R 0x40118, 0x3F82A, 0x3E420 | MISSING | |
| HU23 | Class 63, 64, 3, 13, 79 count in waves (`+$7D = 0`), classes 27, 43, 65, 66 do not; bikers do | R | MISSING | |
| HU24 | Episode 7 library: no spawner in shelves; the three Bludges crouch and leap in from the right edge, the 10 barrels roll in along the floor | R stage_scripts.txt; M sc-e7-bludge | MISSING | answers part of G07 (library) |
| HU25 | No thug (Punk, Thug, Ferris, Gneiss, Driver, Hammer, Elmer, Wrench, poachers, Blades, alternate Blade, bomb carriers) picks up or uses a floor item; only Bludge reads the item list (0x3AFB2 to 0x3B034); the class 20 port needs no pick up branch; the global item test 0x32762 / 0x327CA is used only by Bludge and two stage script waits (0x1DE72: wait until no item lies, then 90 frames; 0x1DE8C: wait until no non weapon item, then 160) | R scan of 0..0x70000 (pool E base 0x874(a5) and list `G.$69EE / 69F0` references); M pk-* (17 types, 1,500 frames) | MISSING (waits) | answers U01; the stage script waits belong to D4c |
| HU26 | The alternate Punk and Wrench (class 64 with class 77) hold a dynamite stick: a knock down A (`+$7B` 4, carrier state 0x202) frees the stick as a pool E weapon item (type 13 for variant 0, 12 otherwise, 0x24B86) and sends the carrier running (speed id 0x20); a thrown Wrench stick lies 30 frames as a weapon item (class 0 var 0x18) and then explodes (E class 1 var 3) | R 0x5B2CC, 0x5EFAA to 0x5EFDE, 0x24B86; M pk-a64w, pk-a64 | MISSING | the likely source of the user's "russel takes a weapon" memory; the knock down drop was not triggered in MAME |

## L. Leads from the user's Claude Doc "Recreation Blueprint" (2026-10-12; web research, NOT verified)

Source: `plan/game/sources/claude-doc-recreation-blueprint.md` (copy of the doc; its sources are walkthroughs PW1 to PW4, Capcom Wiki, Wikipedia, Hardcore Gaming 101, Namu Wiki). Rows below are new or more precise than the rows above; each names the design task that must confirm it.

| ID | Mechanic (as the doc says) | Source | Port | Check in |
| --- | --- | --- | --- | --- |
| L01 | Episode 3 car seats: the player who took the radio drives, the first other player sits in the passenger seat, a third in the trunk; Jack (the owner) always sits in the trunk; if the radio taker is Jack, the teammate who delivers the car follows the order Mustapha, Hannah, Mess | W:blueprint | UNKNOWN | D3c |
| L02 | Episode 3 has no recovery items at all; skipping the radio makes it much harder (time-outs nearly certain); guns appear in a fixed order (Uzi, shotgun, rifle, shotgun, shotgun, M16, Uzi) with ammo twice; a low-HP player taking the radio can be run over by bikers before the car arrives | W:blueprint | UNKNOWN | D3c, D4a |
| L03 | Hogg: phase 1 must be fought from the car (car vs motorcycle, grenades, he can run players down, ramming his bike works, 3 HP gauges); grenades destroy the car; phase 2 on foot: Hogg flies across the screen in different directions while grunts close in, grenade shadows show where they land, a dropped shotgun helps | W:blueprint | UNKNOWN | D3c |
| L04 | Vice fires into the air to call more henchmen; his Rock Hopper turns from orange to green when beaten and walks away; in the opening Vice speaks, leaves the fight to his men and goes | W:blueprint | UNKNOWN | D3c, D4c |
| L05 | Butcher: knocking him down makes him drop a knife; his knives are weaker than his standing combo; he summons henchmen | W:blueprint | UNKNOWN | D3c |
| L06 | Episode 4: Slice orders the gate shut; the player in the Cadillac blows away the gate defenders and three henchmen, barrels at the entrance are driven through; inside, a gate can be broken on foot; bikers die in one hit; fat enemies run in at a set point where thrown dynamite catches them; a log holds an Uzi and a hunting rifle; Slice throws boomerangs (also several at once), slashes, throws grenades, has 'shadow' attacks | W:blueprint | UNKNOWN | D3c, D3e part 2 |
| L07 | Episode 2: a scoring item hides under three foreground trees at the start; a fat enemy beats the sleeping T. rex awake if left alone (matches the hunter, class 79); Poacher J and Skinner carry guns and may attack green (neutral) dinosaurs and make them hostile (D3a found rifle bullets pass through dinosaurs: recheck) | W:blueprint | UNKNOWN | D3e, D3a recheck |
| L08 | Episode 5: Bludge's tongue grabs and stuns the player; water buckets hold items; Morgan jumps to the screen centre and fires an Uzi, kicks rocks, uses grenades and knives; Morgue has a ranged slap and a quick tackle and drops Morgan's Uzi, which works well against him; Morgan's second form brings henchmen back; an enemy called 'Phantom' (Japanese name?) appears | W:blueprint | UNKNOWN | D3d, D3a |
| L09 | Episode 6: a wall at the start is punched open to enter the mine; no firearms in the whole episode; hitting Tyrog in the air always knocks him down (wastes clock time); Tyrog's hosts: Gneiss (slap, belly slash), Wrench T. (orange, fireballs), Walther (gray, acid spit in a circle); Tyrog finally dies on its own; spoken lines at each host change | W:blueprint | UNKNOWN | D3d, D3e part 2 |
| L10 | Episode 7: the rooftop is Episode 1's rooftop in evening colours; the pterodactyl dies in one hit (Zeke: confirmed by D3b); the elevator goes down about 40 floors and stops for waves; the library and the computer room are each reached by breaking a gate; two Slisaurs for one player, three for two or more, with higher HP in co-op | W:blueprint | UNKNOWN | D3d, D3e part 3 |
| L11 | Episode 8: jar 3 releases Bludge, jar 4 Rock Hopper, jar 11 Tyrog 2 (one form, does not revive, worth only 1000 points); wall flamethrowers in the sewer are destroyed by punching; barrels roll down the cave slope and running ahead lets the scroll lock remove some of them before their items are taken; a meat barrel stands before the last boss; Fessenden's second form slashes, drops fireballs on the ground and calls groups of Blades who throw knives | W:blueprint | UNKNOWN | D3d, D3e part 3 |
| L12 | Gun supply by player count: extra guns appear for two or three players up to Stage 4 (a shotgun for two, an M16 for three); after that supplies and healing match one player | W:blueprint | UNKNOWN | D4a |
| L13 | An empty gun can be thrown for one last hit | W:blueprint | UNKNOWN | D2c (check) |
| L14 | Continue screen: a boss aims a gun in close-up with a 20 s countdown; continuing makes the player punch him; if the countdown ends he fires | W:blueprint | UNKNOWN | D4d (check) |
| L15 | A route map of the New York area appears at each stage start (the map card D4c and D4d measured) | W:blueprint | UNKNOWN | D4d |
| L16 | Debug mode (TCRF): stage select, slow motion, invincibility, object sprites and an object-debug overlay; could help the probe capture spawn data and hit boxes | W:blueprint | UNKNOWN | T0 / probe tool |

## M. Leads from the user's Claude Doc "Deep-Dive Supplement" (2026-10-12; mostly Namu Wiki, machine-translated; NOT verified)

Source: `plan/game/sources/claude-doc-deep-dive-supplement.md`. Several rows give numbers (scores, food, rank) that the ported code already has from the ROM: compare, and trust the ROM.

| ID | Mechanic (as the doc says) | Source | Port | Check in |
| --- | --- | --- | --- | --- |
| M01 | Enemy kill scores and Japanese names (Gneiss 2000, Ferris 1000, Driver 1000, Punk 1000, Doug 2000, Blade 2000, Razor 3000, Poacher J 2000, Skinner 3000, Gutter 4000, Blk Elmer 3000, Hammer 2000, Wrench 4000, Lash 1000, Walther 8000, Bludge 8000); one 'Doug' placement is worth 80,000 and shows a corrupted face palette and the name Wrench T. | W:supplement | UNKNOWN | D4b, D3a |
| M02 | Punk sometimes throws a grenade then runs away; 'Doug' (the alternate Punk) also body-slams and jump-attacks, throws dynamite and runs (matches D3a: class 64 var 0) | W:supplement | UNKNOWN | D3a (done, compare) |
| M03 | Blade drops his knife when knocked down; a thrown knife can be punched away; some Razors drop from above with a shadow in the second half | W:supplement | UNKNOWN | D3a |
| M04 | Gutter rarely drops a rifle but leaves ammo if a gun lies on the floor; Walther never flinches (no hit stun, also Hogg), often wakes up invulnerable, walks fast with raised shoulders at low HP, and drops a revolver 100 % on death through Episode 5 except Episode 3; Lash comes back from off-screen and his mace also hits other enemies (Walther) | W:supplement | UNKNOWN | D3a, D3c |
| M05 | Bludge drops an item it is eating when hit, never drops a digested one, and can pick up the falling rocks in Episode 5 | W:supplement | UNKNOWN | D3a (compare), D3e part 2 |
| M06 | Bosses have a fixed HP total regardless of rank; with 2+ players the boss count or HP rises and a boss's helper limit goes up by 1 or 2 | W:supplement | UNKNOWN | D3c, D3d, D2c (compare) |
| M07 | Vice: opens by hitting Rock Hopper three times to provoke it; calls henchmen each time his HP drops a set amount (bikers first, then fat chargers); at 1 HP Punks pop in from the left, throw grenades and flee; five-hit rapid chain, long straight punch, revolver with a long wind-up; Rock Hopper calms after two shotgun hits | W:supplement | UNKNOWN | D3c |
| M08 | Butcher: each knockdown makes him drop a machete, then he runs to pick it up (lost if a hero grabs it); he targets whoever holds his blade; flying punch becomes a stab with a blade; high jump butt slam from off-screen with a shadow; no wake-up invulnerability; jumps away on waking when the hero is on his left; only two Butcher blades exist | W:supplement | UNKNOWN | D3c |
| M09 | Hogg mirrors the hero's forward and backward movement about half a beat late; holding left in the car while dodging grenades makes him fall back; if the car is destroyed or the hero arrives on foot, scrolling stops and several Hoggs appear around the screen; knocking him down breaks his motorcycle; his clear scene shows smoke rising from the garage; finishing him with the car gives no kill score | W:supplement | UNKNOWN | D3c |
| M10 | Slice: a single slow boomerang can be punched away; super = several boomerangs with a spin that makes him invulnerable, often used the moment he wakes; when sent off-screen he runs back and slashes about 80 % of the time. Slisaurs: same patterns, more HP, no wind-up before the wake-up super, stay down longer, each tracks its own target | W:supplement | UNKNOWN | D3c, D3d |
| M11 | Morgan: hops, throws knives, rocks and grenades, fires an Uzi, gives 0 points; Morgue has the Butcher's HP, a roaring headbutt rush and close slaps | W:supplement | UNKNOWN | D3d |
| M12 | Tyrog: bone spear from the chest, short arm hits, flying body press; when beaten only the small head remains and enters a fat enemy (fire shots), then a Walther (acid spray); HP refills at each change; one hit chance in the small-head phase; form 3 does not chase off-screen; co-op multiplies the HP of the jellyfish form 2 or 3 times; Tyrog 2 never absorbs anyone and counterattacks when cornered | W:supplement | UNKNOWN | D3d |
| M13 | Fessenden form 2 needs about six hits to be knocked down, ignores attacks mid-combo, but a grab and throw knocks it down at once; it targets the rearmost player first | W:supplement | UNKNOWN | D3d |
| M14 | Dinosaurs: a machine-gun hit makes them counterattack at once, shotgun pellets do not; only a stegosaur carcass (the Butcher scene) ever dies | W:supplement | UNKNOWN | D3b (compare) |
| M15 | Scenery fauna: a grazing duck-billed dinosaur in the Episode 2 swamp; two kinds of little dinosaurs at the start of Episode 2, and stepping on the plated kind flips it on its back until it rights itself; a flock of pale blue birds flies up from the grass at the start of Episode 2; the sauropod's head laughs in the swamp after the foot section of Episode 6 | W:supplement | UNKNOWN | D3e, D3b |
| M16 | Score item values (gold bar, diamond, dinosaur egg 10,000; gold dust, pearl shell, ammonite 5,000; watch, necklace, ring 3,000; sunglasses, coin, glove, bill, book 1,000), partly random; many hidden behind foreground scenery or appear after scrolling to a point (Episode 5: a diamond in the rock where the triceratops appears; Episode 6: a gold bar when three grunts appear after entering the mine) | W:supplement | UNKNOWN | D4a |
| M17 | Food heal (percent of the bar) and full-HP bonus: barbecue 100 / 10,000, steak 90, sushi and lobster 80, hamburger, pizza, hot dog 50 / 5,000, salad, cake, fries, parfait, pudding 30 / 3,000, donut 20 / 1,000, chocolate, coffee, croissant 15, gum 10 | W:supplement | UNKNOWN | D4a (compare with the ported table 0x28B10) |
| M18 | Weapons: dynamite lights when thrown or merely set down to pick up another weapon, and picking it up again puts it out; an unused gun left on the floor too long vanishes; a throwable weapon (also an empty gun) can be launched by attacking while dashing and the hero keeps running; ammo picked up with no gun fills the next gun; ammo with no gun or a full gun gives 1,000 points; shiny drums catch fire when tapped and explode | W:supplement | UNKNOWN | D2c (compare), D4a |
| M19 | Points per action: punch 100, kick 200, held weapon hit 400 (stone 100), throw 400, 'single smack' 500 and 'double smack' 900 (critical hits from a jump-down-attack or a well-timed punch, and from dash-jump attacks), down-up-attack main hit 700, barrel or drum hit 1,000 scaled by enemies hit | W:supplement | UNKNOWN | D2b (compare), D4b |
| M20 | Boss kill scores: Vice 10,000, Butcher 20,000, Hogg 1,000 (on foot only), Slice 40,000, Morgan 0, Morgue 30,000, Tyrog 60,000, Tyrog 2 1,000, each Slisaur 70,000, Fessenden form 1 16,000 (8,000 Japan), form 2 1,000 | W:supplement | UNKNOWN | D4b |
| M21 | The continue counter shown for every player goes up by 1 for each continue of any player; continuing plays a short scene where the hero punches the gunman | W:supplement | UNKNOWN | D4d (compare) |
| M22 | Rank system as fans describe it: a normal rank and a timer rank 0..15 (how fast the normal rank rises over time); operator setting 1 (0..7, default 3) sets the starting rank (6..18) and the drop after a miss (-7..none), setting 2 the starting timer rank; lowest / highest rank 0 / 24 (1 player), 4 / 28 (2), 4 / 31 (3); a second miss within 10 s drops 3 more; being hit drops 1; timer rank +2 when three GO marks pass before an area change, +1 when a boss falls, +3 when a player joins; the timer stops during GO, scripted scenes, the continue screen and after a boss; three-player lowest-rank mode; stage start rank raised toward a stage minimum | W:supplement | UNKNOWN | D4b (compare with the ported rank.rs) |
| M23 | Rage state: when the HP bar flashes at low HP most attacks hit harder (dynamite kills fat enemies, the Uzi hits like an M16); it lasts after healing and ends at the first hit taken (probably the attack factor 64 below 25 HP of P49) | W:supplement | UNKNOWN | D2b (compare) |
| M24 | A thrown empty gun sometimes leaves the target dazed (stars); plain beating sometimes does too | W:supplement | UNKNOWN | D2c, D3f |
| M25 | Team move voice lines (Jack "Let's Go!", Mustapha "Let's Roll!", Mess "Let's Fly!", Hannah "Jump!"); explosive kills (grenade, bazooka) leave eyeballs in the remains and a hero line (Jack "Take that", Hannah and Mustapha "Gotcha", Mess "Blammo") | W:supplement | UNKNOWN | D4e, D3f |
| M26 | Stage extras: Episode 4 shows a battered, overturned Cadillac later in the stage and a broken drum can drop a rifle instead of a stick; Episode 6 has a wooden barrel with a salad before Tyrog that can be carried to the boss; Episode 8 sewers have handcuffs and chains on the walls; the cutscenes before the final boss do not speed up when a button is held (so the others do) | W:supplement | UNKNOWN | D3e, D4c, D4d |
| M27 | Overseas opening text (the story intro of the attract loop) and endings: after the credits Jack and Hannah arrive in the repaired Cadillac, Jack speeds up, Mustapha and Mess run alongside, Capcom logo; the attract demo ends with the logo and a scene of poachers shooting Vice; continue screen lines include a threat to feed the player to the T. rex | W:supplement | UNKNOWN | D4d (compare) |
| M28 | In MAME the invincibility cheat stops a stage from ending because the hero keeps the golden glow of a mid-game joiner | W:supplement | UNKNOWN | T0 / probe note |

### From `design/D3c-bosses-1.md` (D3c bosses part 1), 27 rows

| ID | Mechanic | Source | Port | Notes |
| --- | --- | --- | --- | --- |
| BS01 | The four bosses share only the HP row, the reinforcement controller (pool B 22) and the death chain (0x3262A, 0x32740, 0xACEE, 0xA5EA, state 4: kill score + 0x6148); intros and AI are private | R | MISSING | D3c 1 |
| BS02 | Boss HP by level and players (Vice, Slice 350..600, 500..700, 600..750; Butcher 512..640 / 576..704 / 640..768; Hogg 300 / 400 / 500 flat); the HUD bar is HP / 100 gauges | R 0x31E14, M | DONE | D3c 1.2 (corrects M06: only Hogg is fixed) |
| BS03 | Vice's intro: waits for the camera at X 0x1D0, talk box (sound 0x20), 40 frames, camera mover, anim 0x14, 60 frames, walk, two captions, the gun anim with a shot (sound 0x3B), 60 frames, music 0x0E, controller; 710 frames | R 0x426AC, M v1-nat | MISSING | D3c 2.2 |
| BS04 | Vice: 13 combos by a 32 cell table, attack gate T[level] (6 .. 31 of 32) every 60 frames, recovery pause tables, speeds by level (2.6 .. 3.5 px) | R 0x44B84, 0x44C8C, 0x43004, 0x436D4, 0x44F4E | MISSING | D3c 2.3 |
| BS05 | Vice's summon: after a knockdown at HP <= max / `+$C2` (2, 3, 4): jump back, revolver fired into the air (muzzle flash, smoke), `G.$502 = 1` releases the next controller group (Ferris pair, Hammer, Punk alt carriers) | R 0x44EBA, 0x4382A, M v9-alt | MISSING | D3c 2.4 |
| BS06 | Vice's rage: HP <= 50 (first charge) or <= 25 (second), `+$AA = 1800`, only evaluated when state 0 does not start a summon | R 0x42B52, M v7-rage | MISSING | D3c 2.5 |
| BS07 | The bullet (class 59 var 2): speed 7, height 70, 0x0B (15 base, knock down), shared with the poachers | R, M v5-face | MISSING | D3c 2.6 |
| BS08 | The Butcher is class 5 (chopping a dead dinosaur with two blades, A51 carcass, talk box, camera mover, 475 frames) that rewrites its class word to 61; sounds 0x21, 0x5F (every 32 frames), 0x0F | R 0x38DDE, M b10-nat2 | MISSING | D3c 3.1 |
| BS09 | The Butcher's moves: walk 2 px, slash combo 0x2D / 0x1B (reach 108), far slash 0x2F / 0x30 (reach 140), leap (apex 157 px, 67 to 154 frames) with a body slam landing 0x2E, dash speed 3.5; the spin 0x33 is dead code | R, M | MISSING | D3c 3.3 |
| BS10 | Knife drop: while he holds a knife, a knockdown reaction (4 or 6) drops one with probability 8 / 16, the second made first; the knife becomes an item (class 0 var 0x12) after about 50 frames; he can re-arm (state 0x10) | R 0x586D6, 0x5959E, M b6-knife, b5-dmg | MISSING | D3c 3.4 (corrects M08) |
| BS11 | The Butcher targets the lowest HP hero 10 of 16 re-targets (HP below 112) | R 0x58CD6 | MISSING | D3c 3.3 (corrects M08) |
| BS12 | Hogg is a rider (class 31) on a bike (class 32, the moving body); foot behaviour: ride across at 5.5 px per frame (116 to 127 frames), wait 121, return from the other side, 0x6A ram (10 base, knock down), steering to a hero's depth every 30 frames | R 0x4FF74, M h1-foot | MISSING | D3c 4.4 |
| BS13 | Hogg's bomb throws: roll every 60 frames (30 in car mode) `rng & 31 <= T[level]` (16 .. 29), 1 to 4 bombs per throw by his HP quarter | R 0x50AD8, 0x50B74, M | MISSING | D3c 4.4 |
| BS14 | Hogg's bomb (class 42): a lob to a 3 x 3 grid around the car (+-64 X, +-16 Z; weights 1 2 1 / 2 3 3 / 1 2 1 of 16) or to the hero; 40 or 59 frames; blast 50 frames, 0x69 (12 base, burn); the car is hit twice per bomb that lies on it | R 0x55E9A, M c3, c1 | MISSING | D3c 4.6 |
| BS15 | Hogg in car mode: a random walk on a 32 x 8 cell grid around the car (re-drawn every 25 frames), bike 72 .. 346 px ahead at 11 px, beside the car at 8; throws every 71 frames on average | R 0x5012C, M c1, c3 | MISSING | D3c 4.5 (answers M09 "mirrors the hero") |
| BS16 | Hogg's death: the rider is thrown (vy 5 px up, vx 1), the bike breaks into pieces and explodes where it lands; single Hogg only; no hit stun animation | R 0x5040E, M h4-foot | MISSING | D3c 4.7 (answers M04, M09) |
| BS17 | Radio: treasure pick up in Episode 3 with `G.$50E = 0` calls 0x8BD24 (3 free pool B slots); sources: barrels at X 0x250, 0x580, 0x2080, a rolling barrel at 0x2990, 8 script enemies (drop row 0x28); kill-all with credit to the taker; monsters (bikers) can hit the taker before the car comes | R 0x2A116, M c1, L2-biker | MISSING | D3c 5.1 (answers G04, L02) |
| BS18 | Car arrival timeline (t after the pick up at 5): car at 16 drives in at 5 px, brakes 74 to 100, waits 120 + 60, boarding jump 49 frames from 283, accelerates 64 per frame squared from 329, drive from 363 (`G.$518 = 1`, camera mode 8 at 364) | R 0x8BF1E, M c1 | MISSING | D3c 5.2 (extends T21) |
| BS19 | Car steering: every rider with a stick adds 0x800 + (+-768 vx, +-256 vz, diagonals +-256, +-192) to the target speed (one rider), halves (two), x 6/16 (three); no one pressing: 8 px per frame; car vx follows by 48 per frame; Z band 16 .. 80, X band camera + 0x30 .. + 0x140; attack and jump keys do nothing | R 0x8C2A2..0x8C402, M c1, c2, c3, c4 | MISSING | D3c 5.4 (answers W15 "who drives") |
| BS20 | Car versus the world: flat 20 to every enemy and monster it touches (credit to the taker), containers HP 0, monsters hurt the car by the record damage only while `G.$7B4 != 0`, heroes aboard are not targets | R 0x121D8..0x12454, 0x18A02, M | MISSING | D3c 5.5 |
| BS21 | Car end: HP 0, or camera X >= 0xA000, or the boss death script; heroes put at camera X + 0x80 / 0xA0 / 0xC0, 120 frames invulnerable, jump arc 48 frames; the wreck leaves; clock restarts | R 0x8C810, 0x8C73C, M c1, h8 | MISSING | D3c 5.6 |
| BS22 | Car seats: owner (radio taker) first, other players in player order; the delivering NPC is the first character not in play in the order Jack, Mustapha, Hannah, Mess; every rider is at the car position | R 0x8C642, 0x8C784, M L1-seats | MISSING | D3c 5.3 (answers L01) |
| BS23 | The radio works during the Hogg fight (Hogg switches to car behaviour) and a second radio gives a second car | M h9, h15 | MISSING | D3c 5.7 |
| BS24 | Slice: kick 0x36, long slash 0x37 (reach 138), single boomerang (class 19, var 0 / 2 / 4, 7 base), fan of 12 (var 6, 13 base, knock down), run speed 3 .. 7 px accelerating every 4 frames, script table by HP and distance, the wake-up roll 11 / 5 / 3 of 32 | R 0x4C11C.., 0x4D288, 0x4C0B6, 0x4504E | MISSING | D3c 6.3 |
| BS25 | Afterimage: a pool C object class 27 every 7 frames while running, copies the frame and facing, 6 palette steps of 6 frames, blinks every other frame, 36 frames | R 0x9639E, 0x963D6, M s4-face | MISSING | D3c 6.5 (answers G06) |
| BS26 | Boomerangs can be hit by heroes (monster list) and fall; the thrower catches them on return | R, M s8-alt | MISSING | D3c 6.4 (answers M10) |
| BS27 | Boss reinforcement lists by episode: group order, `G.$502` setters (Vice summon, Slice and Hogg 300 frames, Butcher 30 frames), helper limit 4 / 5 / 6, Hogg's list stopped in car mode, the gun order Uzi, shotgun, rifle, shotgun, shotgun, M-16, Uzi | R 0x7BEF6, `b22_lists.txt` | MISSING | D3c 1.7 |
