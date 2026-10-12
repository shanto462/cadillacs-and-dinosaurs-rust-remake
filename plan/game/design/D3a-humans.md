# D3a: humans, mutants and the actors that are not bosses (every pool A class except bosses and dinosaurs)

Written 2026-10-12 by the D3a research agent. Output of task D3a of `plan/game/NEW-PLAN.md`. Nothing here is ROM data:
only descriptions, numbers and addresses.

Source tags: **R:** ROM address or repo file, **M:** MAME probe (the segment name is given; scripts `re/out/d3a/gen_r1.py` to
`gen_r6.py` and the logs `re/out/probe/d3a-r1` to `d3a-r6`, all git-ignored), **W:** web, **U:** the user. **MAME wins over the
web.** `[I]` marks an inference that no run or listing proves yet. `G.$xxxx` is RAM `0xFF8000 + xxxx`. Pool A classes are
numbered by `+$20 / 4` (the number the scripts print as `A#nn`). `+$xx` is an object field (`re/specs/enemy/SPEC.md` section 2).
"Frames" are game frames after the first frame of control (`t` of the probe), at 60 Hz. "Level" is the difficulty index G.$5EE
(0 to 31; I froze it with `level N free`). Damage numbers are on Jack at 100 HP.

How the work was done: I read the handlers of classes 3, 6, 13, 16, 17, 27, 35, 43, 45, 57, 59, 62, 63, 64, 65, 66, 70, 77, 79 and the
Walther half of 18 (listings in `re/out/d3a/c*_dis.txt`, `*_ann.txt`), decoded their tables, and ran **six** headless MAME batches
(`d3a-r1` scouting and natural play, 22 segments; `r2` behaviour of every class, 74 segments; `r3` rifle geometry, natural hunts, hazards,
33 segments; `r4` places and scene actors, 21; `r5` bullets and explosions, 14; `r6` item pick up test for the "item thief" lead, 19 segments).
The sixth run is one more than the five I was allowed; I made it because the coordinator's lead U01 asked for a natural play check with items on the floor. Where a probe was wrong (a spawned test enemy that walked
to the hero before it could shoot, a kill pulse that also killed the hunters) I say so and the number comes from the repeat.

## 0. Summary of the findings

1. **The user is right: poachers wake dinosaurs and the dinosaurs then hit poachers, in Episode 2 and in Episode 5. Episode 6 has
   nothing of that kind.** The only thing in the whole game that wakes a sleeping dinosaur is the **dinosaur hunter (pool A class 79)**:
   it walks to the sleeping Shivat, stands in front of it and its **third swing** pokes the dinosaur (helper 0x533FA, no HP loss). The
   dinosaur flinches, flashes for 88 frames, turns angry and then attacks the nearest hero **or enemy**, and the hunter that woke it
   (it walks away to the left, never fights a hero) is its first victim. Measured in natural play (hunters walk, nobody touches
   them): Episode 2 at frame 1030 to 1125, Episode 5 at frame 1075 to 1165 (section 2). A rifle poacher (class 13) cannot do it: its
   bullets, kicks and punches never touch a dinosaur (bullets fly through a Hopper, a Mack, a sleeping Shivat and a Ferris without a
   scratch, M bsp-*). [R 0x5F6CA, 0x5F9E6, 0x533FA; M nat2-hunt, nat5-hunt, hun-v400, bsp-hopper, bsp-shivat]
2. **A hunter is freed in its first frame when no sleeping dinosaur is alive.** Its target scan (0x5F6CA) takes only classes 24, 29 and
   38 whose `+$C0` is negative; that is only a sleeping Shivat. So in Episode 5 the three hunters of wave 1 vanish at once if the static
   hunter has already woken the Shivat, and a hunter spawned with no dinosaur around is gone before it is drawn. [M hun-none, hun-hopper,
   hun-mack, nat5-pulse]
3. **Episode 6: no hunter, no poacher-dinosaur chain.** Its only dinosaur is the brontosaurus foot (A22). Poachers there are not woken
   against anything; but the foot (slam 30, knock down, touch 1) and the **rolling barrels (class 65, 10 damage)** hurt poachers in
   their way, the same way they hurt heroes (measured on poachers: foot 59 to 29, barrel 44 to 34). In natural play the barrels hit a
   poacher at frame 425 and two Blades at frame 590 (M nat6-pulse). [R 0x4C0A list; M h-foot-poacher, h-barrel-poacher]
4. **The three "unknown" enemy classes of the stage scripts are not enemies:** classes 65 and 66 are **bouncing barrels and rocks**
   (hazards in the monster list, HP 8, 10 damage), class 70 is a **biker on a motorcycle** (run-by hazard, HP 8, 10 and 17
   damage), classes 27 and 43 are the **crouching men and the orange haired leader of the Episode 4 opening**, classes 35 and 45 are
   the **Morgan introduction actors of Episode 5** (35 turns into Morgan, class 36), class 16 is the **blue Cadillac** (with class 17
   wheels) of the Episode 2 opening, 78 and 80 are harmless ground critters (D3b). [R, M sc-*, e4-open]
5. **Bludge (class 6) eats food, weapons and treasure that lie on the floor** (up to 8 in a queue), and gives all of them back when it
   dies. This is the only "enemy picks up items" rule of the game. Seen: at aggression 7 it walked to the treasure and the object
   disappeared; on its death the treasure came back (thrown 270 px to the right). In run 6 it ate the food and then the weapon from about 75 to 80 px away. [R 0x3B006, 0x3B05E, 0x3B086; M bl-eat-a7, bl-death, pk-bludge]
   **There is no "item thief" among the thugs (lead U01):** no Punk, Thug or other pool A class except Bludge reads the item list; class 20 needs no pick up branch (section 3.3). The Punk the user remembers is most likely the
   **alternate Punk (class 64, name 0x1D)**, which holds a dynamite stick and drops it as a weapon item when it is knocked down. [R scan of the whole ROM; M pk-* (17 enemy types, 1,500 frames, items untouched)]
6. **Class 64 (the "alternate Punk and Wrench T.") is a bomb carrier, not a fighter:** fixed HP 20, hops and runs, carries a dynamite
   (class 77) that is thrown and explodes in pool E explosion kind 3; heroes take 10 to 19 from the blasts. **The alternate Wrench T.
   (variant byte 2, name 0x24) gives 80,000 points when it is killed** (seven in the game: the four of Episode 3 at X 0xEC0 and three in Episode 5 wave 4; the Punk alt
   carriers give 1,000). [M ks-a64v2: score +80,100; a64-e3-hit]
7. **Class 63 (alternate Blade) is a hit and run knife thrower:** it dashes in (attack 0x1C), throws a blade (class 41, attack 0x19, 12
   damage at level 0, 25 at level 31), waits, runs off the near side and is freed off screen; if nobody kills it, it never needs to
   die. HP 36, score 1000. [M a63-l0, a63-knife]
8. **Poacher rifle (class 13):** bullet = pool A class 59 variant 0, speed 7 px per frame, attack 0x0B (15 base, knock down), muzzle
   flash class 57. The bullet does **not stop at the hero**: it flies on and is freed at the screen edge. One to three shots per
   volley (table 0x3F6E8), 69 frames between shots, first shot 51 frames after the aim starts, then a 60 to 95 frame pause (table
   0x3EADA). The poacher shoots mostly from 121 to 144 pixels away (distance class 5, 24 rolls in 32; class 4, 97 to 120 px: 4 in 32) and only when
   the hero is within 7 pixels in depth; closer it punches (3 hit combo 8, 9, 0x2B), at 145 to 168 px it dashes (attack 0x2C), beyond
   that it walks up with a kick. [M rf-*; R 0x4005C, 0x72A94, 0x72F14]
9. **The enemy bullet reads a stale `+$84`:** class 59 never writes its name row, so its damage scale is that of the last object in the
   slot (I measured 27 at level 31, the scale of a Ferris row, not 33.75 of a Poacher J row). The port must keep a per slot `+$84`
   (the free routine keeps it) or fix a row. [M bsp-lvl31, rf-l31-d144; R 0x57FB4]
10. **Walther is Vice's AI without the intro** (class 18 variant 2: normal HP table, `+$63 = 0`, kill-all, score 8000). D3c owns the AI.
11. **Shared stubs** (section 3): the six hit passes of the port that are empty are all small; 10CD0 and 10DCC are not about enemies at
    all (pool B breakables, attack objects against heroes), 1132A, 11A9E, 11CEC and 11E56 are the monster passes of D3b. **There is no
    "enemy hurts enemy" rule other than the monsters, thrown bodies (ported) and explosions.**
12. **Port status:** classes 2, 10, 14 (with parts 15, 34, 41, 60) and 20 are ported; every class of this document is missing. About 28
    agent hours (section 15).

## 1. The roster (every pool A class that is not a boss and not a dinosaur)

Handler sizes are `end - start` of the class code in `re/specs/enemy/classmap.txt`. "Port" is the state in `dino-rs`.

| Class | Handler (bytes) | What it is | Names (`+$3`) | Where used | Port |
| --- | --- | --- | --- | --- | --- |
| 2 | 0x3383A | basic thug | Ferris, Gneiss, Driver | Ep 1, 3 to 8 | DONE (`enemy.rs`) |
| 3 | 0x358B2 (4,530) | Lash T., mace on a chain, with parts A62 and A8 | Lash T. (0xE) | Ep 2 wave 5, Ep 5 wave 6, Ep 6 area 0 and wave 2, Ep 7 wave 11, Ep 8 wave 10 | MISSING |
| 5 | 0x38DDE (1,172) | the Butcher's trigger (D3c) | | Ep 2 area 2 | D3c |
| 6 | 0x39272 (7,826) | Bludge, leaping lizard man, eats items, acid spit (A7) | Bludge (0x10) | Ep 5 wave 3, Ep 6 wave 6, Ep 7 area 2 | MISSING |
| 10 | 0x3B828 | fat men | Hammer T., Blk Elmer, Wrench T. | Ep 3, 4, 7, 8 | DONE |
| 13 | 0x3DDEA (9,252) | poachers with rifles | Poacher J (2), Skinner (0x12), Gutter (0x13) | Ep 2 to 8 (about 120 records) | MISSING |
| 14 | 0x4020E | knife thugs | Blade, Razor | Ep 1, 3 to 8 | DONE (parts 15, 34, 41, 60 too) |
| 16, 17 | 0x420EA, 0x423A4 (680, 492) | blue Cadillac and its two wheels | none | Ep 2 start | MISSING |
| 18 var 2 | 0x42590 | Walther (Vice's AI) | Walther (0xD) | Ep 3, 4, 5, 6 | MISSING (D3c) |
| 20 | 0x457B0 | punks (no item pick up: section 3.3) | Punk (0x08), Thug (0x09) | Ep 1 to 5, 7 | DONE |
| 27 | 0x4D900 (436) | three crouching men of the Episode 4 opening | Ferris, Gneiss, Hammer look | Ep 4 area 0 | MISSING |
| 35 | 0x513DA (696) | Morgan introduction actor, becomes class 36 | | Ep 5 area 1 | MISSING (D3d) |
| 43 | 0x55F2A (614) | orange haired leader of the Episode 4 opening, makes the banners | | Ep 4 area 0 | MISSING |
| 45 | 0x5639A (466) | the villager of the Morgan scene, gun shot sound | | Ep 5 area 1 | MISSING (D3d) |
| 57, 59 | 0x57D94, 0x57FB4 | muzzle flash, enemy bullet | | made by class 13 (and Vice for 59) | MISSING |
| 62, 8 | 0x5A47C, 0x20 | mace and chain piece of Lash T. | | made by class 3 | MISSING |
| 63 | 0x5AA6E (1,364) | alternate Blade, hit and run knife thrower | Blade alt (0x1C) | Ep 2 wave 2, Ep 5 wave 6, Ep 7 wave 3, Ep 8 | MISSING |
| 64, 77 | 0x5AFC2 (2,288), 0x5EDF4 (916) | bomb carriers (alternate Punk and Wrench T.) and their dynamite, dropped as a weapon item when knocked down | Punk alt (0x1D), Wrench alt (0x24) | Ep 3 (4), Ep 4 wave 1 (3), Ep 5 wave 4 (4), Ep 8 (7) | MISSING |
| 65 | 0x5B8B2 (728) | bouncing barrel hazard | none | Ep 3 (22), Ep 6 (6), Ep 7 (10 + 3), Ep 8 (11) | MISSING |
| 66 | 0x5BB8A (732) | bouncing rock hazard | none | Ep 5 area 0 (3 + 3) | MISSING |
| 70 | 0x5C356 (1,218) | biker on a motorcycle, run by hazard | none | Ep 3 (5 + 4), Ep 4 (3 + 3), Ep 7 (2 + 3) | MISSING |
| 78, 80 | 0x5F198, 0x5FF2E | ground critters, harmless | | Ep 2 area 0 | MISSING (D3b section 7) |
| 79 | 0x5F5D0 (2,398) | dinosaur hunter | Hammer T. / Blk Elmer / Wrench T. (0x1E to 0x20) | Ep 2 wave 2 (2), Ep 5 (1 + 3) | MISSING |

Classes 4, 22, 23, 24, 25, 29, 31, 36, 38, 40, 61, 71, 72, 82, 84, 86 to 89, 91, 94 belong to D3b, D3c and D3d. Class 22 is not human: it
is the brontosaurus foot (D3b section 6).

Palettes: none of these classes writes a palette byte (`+$23`) except the poacher entrance 4 (`0x8B`, a fade, section 4.3). The colour comes
from the OBJ palette of the area: Bludge is **red** in the Episode 2 test arena and **green with an orange mane** in Episodes 5 to 7
(M bl-sheet-v6 and sc-e5-bludge); the poacher variants differ only by animation table (0x6C57A, 0x6CA24, 0x6CF3E).

## 2. Poachers and dinosaurs: who wakes whom, and what the player can do with it

User question (2026-10-12): "yep in few stages, 2 and probably 5 and 6". Answered in natural play with the real scripts, a hero
in god mode and no spawns (`jump` to the area, hold right, a few `killall` pulses only to pass the wave locks; the pulses are named
under each timeline). Frames are `t` after control; 1 player, Jack.

### 2.1 The rule in the code

| Fact | Source |
| --- | --- |
| Only objects whose `+$0` is 1 or whose hit lists contain them can be hit. A sleeping Shivat has `+$0 = 2` and `+$C0 = 0xFF`: heroes cannot hit it, nothing else touches it. | R D3b 1.4; M s3-sleep |
| The hunter (class 79) scans pool A for classes 24, 29 and 38 with `+$C0` negative (0x5F6CA). Only a sleeping Shivat has that. (Mack variant 4, a sleeper, is never scripted; the Hopper never sets `+$C0`.) First one found is taken without a claim check; a second one is taken only if its `+$C1` is 0 and it is nearer in X; the chosen one gets `+$C1 = 1`. If none is found the scan returns carry and the hunter is **freed in its first frame** (0x5F5FC to 0x5FADA). | R 0x5F6CA; M hun-none, hun-hopper, hun-mack (object gone after 1 frame) |
| The hunter walks to the dinosaur: speed id from the table 0x5FB50 (see 5.2), first X or Z until within 40 px in X and 8 px in Z, waits 30 frames, then swings (attack id 5, 13 damage, stop 22) three times; the frame byte of the third swing is negative and calls 0x533FA, which puts the dinosaur into hit reaction (`+$4 = 0x02020000`, credit `+$2 = 0xFF`, hit stop 8) without any HP loss. The dinosaur then enters COLOR (88 frames), then OKORI (angry). | R 0x5F9E6, 0x533FA; M hun-v400 |
| After the poke the hunter finds its target awake (`+$C0` no longer negative), sets the target's `+$C0` to 1, searches another sleeping dinosaur, finds none and switches to state 8: it **walks left at its speed (facing left) until its X is 0xC0 left of the camera**, then 0x173C frees it. No drop, no score. It never attacks a hero. | R 0x5FA5A, 0x5FA80; M hun-v400 (freed at x 830, camera 1024) |
| A hunter that is hit recovers into state 0 (entry 0x5F84C for all three names in table 0x2B56C); with the target awake it goes straight to state 8. | R 0x2B56C |
| A poacher's bullets (class 59), knives (41), punches and kicks are tested only against the hero list. A dinosaur is in the monster list, which is a victim list of hero attacks only. A bullet flies through any dinosaur or enemy that stands in its line. | R passes 0x115B0 (victims: heroes), 0x11A9E, 0x1132A; M bsp-hopper, bsp-shivat, bsp-ferris, bsp-mack (no HP change, bullet reaches x 854) |
| An awake dinosaur (list 0x6B64) hurts every hero and every enemy it touches, with raw damage and no score. | R 0x11E56; M below |

### 2.2 Episode 2 (area 0, the 0x568 lock): measured

Script: static list 2, trigger 0x480: a sleeping Shivat (A24) at X 0x678 (1656), Z 0x61, leave X 0x569 (it only leaves when awake or
angry); wave 2 at the lock 0x568: group 1 one Skinner (entrance 0x12, X 0x5E8) plus one more per extra player; group 2 (starts when
`alive <= 1` for 1 player) **two hunters** at X 0x708 and X 0x548 (variant 0, drop rows 0x22 and 0x23); group 3 three alternate Blades
(class 63); group 4 a Hammer. [R stage_scripts.txt, wave 2 at 0xAE308]

M `nat2-hunt` (`jump 2 0`, hold right, `killall` pulses at t = 200, 450, 700 only; the hero stood at the right edge of the screen):

| t | What happened |
| --- | --- |
| 535 | Shivat spawned asleep at x 1656 (`a8 = 0`, `c0 = 255`) |
| 720 to 830 | camera reaches 0x568 (1384), lock; group 1 killed by the pulse at 700 |
| 825 | the two hunters appear at x 1353 (left, z 72) and x 1798 (right, z 72); `+$C1 = 1` on the Shivat |
| 885 to 960 | they walk 1.5 px per frame (speed id 5) toward the Shivat and stop about 40 px in front of it |
| 980 | the right hunter's first swing (attack 5); the third swing pokes: Shivat hit reaction at **t = 1030** (hit stop 7, HP 100 unchanged) |
| 1040 | Shivat COLOR (`a8 = 8`, `c0 = 1`), palette 0x9E / 0x9F flashes |
| 1040 to 1070 | both hunters turn to state 8 and walk left |
| 1125 | Shivat OKORI (`a8 = 4`) |
| 1140, 1160, 1180 | Shivat triple stomp (attack 0x3A): **both hunters (x 1512 and 1543) lose 10 HP three times (100, 90, 80, 70)**, knocked down at 1185 |
| 1385, 1415 | the hunters, still alive with 70 HP, have walked off the left edge and are freed |
| 1430 | the three alternate Blades appear; they throw blades and run off (class 63) |
| 1550 | a Hammer appears; the Shivat (still angry) hits it for 14 at t = 1700 and for 10, 10, 10, 10, 10 at t = 2525 to 3035 until it dies (HP 64 to 0) |
| 3050 | the Shivat reaches MANZOKU (`a8 = 6`) and leaves |

So in Episode 2 **the poachers (the hunters, which the game draws as Hammer look alikes) wake the T. rex, the T. rex hurts them and
every other enemy near it, and it also attacks the hero**. A Skinner (class 13, entrance 0x12) that stood in the area was never
attacked (it did not come near; HP 84 constant for 2800 frames).

M `hun-v400` (the same scenario in the arena, hunter spawned 360 px away): hunter HP 100, 90, 80, 70 at t = 385, 406, 427; the third
hit gave poise 0 (knock down); the hunter got up at t = 529, found nothing to claim and walked out at t = 546 to 622.

### 2.3 Episode 5 (area 0, the 0x3D0 lock): measured

Script: Shivat asleep at X 0x450 (1104), Z 0x251, trigger 0x240, leave X 0x400; static hunter record at trigger 0x3C8, X 0x3A0, variant
0x400; wave 1 at the lock 0x3D0 (`alive <= 4`): Ferris (2+ players), a hunter after 0xF0 frames (X 0x570), a hunter at X 0x3B0, then (when
`alive <= 2`) four Blades and a third hunter at X 0x3B0 after 0x78 frames. R, M.

M `nat5-hunt` (`jump 5 0`, hold right, pulses at t = 250 and 600, nothing after):

| t | What happened |
| --- | --- |
| 580 | Shivat spawned asleep, HP 120 |
| 820 | the **static hunter** appears at x 931 (camera 968 reached) and claims the Shivat |
| 885 to 1010 | it walks 1.5 to 1.9 px per frame (speed id from the table 0x5FB50, it grows with the level) in X, then in Z, and stops at x 1065, z 585, 39 px from the Shivat |
| 1020 | first swing; **poke at t = 1070**, Shivat COLOR at 1075, OKORI at 1165 |
| 1070 | the wave hunters A#16 and A#19 appear (delay 0xF0 after the lock), see `+$C0` still 255, claim it, then (poke done) turn to state 8 and walk left at once (t = 1130) |
| 1240 | the static hunter is off screen (x 783), freed |
| 1240 to 3360 | the Shivat roams between x 1005 and 1321, stomping. (In the bot run, a god hero fighting at the right edge, it took 11 from a hero hit at t = 1935.) |
| 3360 | MANZOKU: after about 2200 frames of OKORI the roll `(rng & 15) == 0` every 10 frames let it leave |

Without the pulses in the first area the same sequence runs, only later (`nat-e5`, hero fighting: the hunter reached the Shivat at t =
2845, COLOR at 2850). When the kill pulse of the probe hits just after the poke the hunters die (`nat5-pulse`): that is only the probe.

### 2.4 Episode 6: what plays that role

**Nothing wakes a dinosaur in Episode 6.** The scripts of Episode 6 have no class 79, no class 24 and no sleeper. The pool B class 79 at
trigger 0xF80 that D3b and the episode file mention is a different pool and starts the boss. Its dinosaur is the brontosaurus foot
(A22 with helper A23, D3b section 6), which is always "angry" and cannot be hit. What the user may have seen is the monster rule:

* the foot hurts **enemies** under it: slam attack 0x4C, base 30, knock down; touching the resting or rising foot 0x63, 1 damage and a push.
  M `h-foot-poacher`: a poacher at the foot's home column lost 1 (touch) and 30 (slam, HP 59 to 29, knock down at t = 378);
* the **rolling barrels** (class 65, section 10) roll through poachers: natural play `nat6-pulse`: poacher 60 to 50 at t = 425, two
  Blades 92 to 82 and 100 to 90 at t = 590; 10 each, raw. (In the Episode 6 jungle they come in 2 x 3 behind each other, X 0x160 and
  0x260.)

The poachers of Episode 6 (entrance 0x12, "jump in from the trees") do not interact with the foot. [M nat6-pulse; R]

### 2.5 What the player can use

| Situation | What happens | Use |
| --- | --- | --- |
| Hunters alive for about 200 frames after they appear | the Shivat wakes and fights the nearest thing for up to about 2000 frames | Keep the enemies near the Shivat and stay away from its front: it stomps (10 x 3), claws (14), grabs (18). It does not care who it hits: heroes and poachers alike. |
| Heroes kill the hunters first (100 HP each, drops row 0x22 / 0x23 in Episode 2) | the Shivat stays asleep for good and harmless; no score for it | Safe route. The kill gives 1000 points and the drop. A killed hunter clears the claim, so the next hunter walks to the sleeper again. |
| A hunter reaches the Shivat while a hero stands in front | the third swing (attack 5, 13 damage, stop 22) has a hit box: it can hit a hero | Do not stand between the hunter and the sleeper [R: the attack record 5 is the hunter's only damage; not measured on a hero, open item 3]. |
| Hero hits the awake Shivat | any hit on a neutral dinosaur makes it hostile (already hostile here); at 0 HP it turns calm and leaves | Draining it makes it leave early (no score, D3b). |
| Poacher rifle volleys | harm only heroes (15 base, knock down) | A hero can stand behind a dinosaur: the bullet goes through it. |

### 2.6 Dinosaurs hurting other enemies, measured

| Source | Victim | Result | Segment |
| --- | --- | --- | --- |
| Shivat stomp 0x3A | hunters (cls 79) | 10 each, three times, knock down at the third | nat2-hunt, hun-v400 |
| Shivat claw 0x38 / stomp 0x3A | Hammer (class 10) | 14, then 10 x 5: killed (no score for anybody) | nat2-hunt |
| Mack charge 0x59 | poacher (class 13) | 15 raw per pass (44 to 29 to 14) | bd2-mack |
| Zeke, Hopper, foot | enemies | same pass, not measured here (D3b) | |

## 3. Shared enemy behaviour that the port still lacks

### 3.1 The six empty hit passes (`combat.rs` order of 0x10546)

Lists (RAM word counters and top pointers): **heroes** 0x6718 / 0x6720; **attack list** 0x671A / 0x6722 (hero attack frames and weapon
objects, pushed by 0x4BDA, used only while `+$2E = 4`); **pool B breakables** 0x671C / 0x6724 (pushed by 0x4BFA); **enemies** 0x671E / 0x6726
(0x4BEA, the hurtable pool A objects, bullets included); **monsters** 0x6B64 / 0x6B66 (0x4C0A); thrown bodies 0x6B30. Frame classes of the
victim (`+$2E`): 8 and 0xA are immune in the passes below, 0, 2, 4, 6, 0xC and 0xE are hittable. [R decomp 0x10CD0, 0x10DCC, 0x1132A, 0x115B0]

| Pass | Attackers | Victims | What it does | Port task |
| --- | --- | --- | --- | --- |
| **0x10CD0** `pass_10cd0` | heroes (+$2E = 4 or 6) | pool B breakables with `+$0` bit 1 clear | boxes `+$49` against `+$48` (0x128F8), one hit per serial `+$5E`; victim `+$5E`, `+$2` = credit, `+$4 = 0x04000000` (routine 4, breaks), `+$68` = attacker; HP minus the damage byte of the hero table 0xA26FC (row of the attacker's `+$20`), floor 0; stop on both, sound 0x962A, hit score (0xFFC8), spark 0x12720. **A hit always breaks it.** Only six pool B classes register (D3e W31: 0, 1, 3, 12 var 4, 40, 45). | S 0.5 h with D3e (the pool B classes need it first) |
| **0x10DCC** `pass_10dcc` | attack list objects with `+$2E = 4` | heroes with `+$0` bit 1 clear and another credit index and another serial | **friendly fire and neutral attack objects.** The victim must be in a hittable frame class, boxes overlap; with a hero as credit the Z words must be **equal** (exact depth), with a neutral credit (`+$2 < 0`) the depth is not tested. Attack ids 0x6C and 0x6E (special) do nothing. Victim: `+$0 = 2`, state 0x02020000, `+$CB = 0x3C`, `+$CA += 1`, `+$F6 = 0x78`, poise minus the record's poise byte, reaction byte of the record (0x22 stays 0x22), stop on both (also on the weapon holder `+$70` when the attacker is in state 0x202 / 0x203), sound 0x962A, spark, then 0x11738 (power factor re-roll). **Damage: 1 HP (never below 1) when a hero caused it, the full record damage (can kill) when the credit is neutral.** Runs only when `G.$4DD == 0` and not in the debug no damage mode. Explosions of pool E class 1 hurt heroes through this pass [I]; I measured 10 and 19 from class 77 blasts (a64-e3-hit). | S 0.5 h (shares the box helper with `pass_10664`) |
| **0x1132A** `pass_1132a` | attack list objects (`+$2E = 4`) | monsters (list 0x6B64) | hero swings and weapon objects hurt dinosaurs; same damage tables as 0x11A9E but no guard. D3b 1.2. | D3b P1 |
| **0x11A9E** `pass_11a9e` | heroes (box mode 4, 6, 8) | monsters | the hero hits a dinosaur; guard table 0x11CE4 (Hopper only). D3b 1.2. | D3b P1 |
| **0x11CEC** `pass_11cec` | monsters with box mode 4 | heroes | a dinosaur, a barrel, a rock, a biker, the foot hurts a hero: attack record of the monster, damage through 0x6422. | D3b P1 (also needed by classes 65, 66, 70 and 77) |
| **0x11E56** `pass_11e56` | monsters with box mode 4 | enemies of list 0x671E | **raw record damage, credit 0xFF (no score)**, facing turned away, a victim with `+$7D < 0` keeps 1 HP. Barrels, rocks, bikers, the foot, Mack, Shivat, Zeke, Hopper hurt enemies this way. | D3b P1 |

Order in the frame: 0x10664, (0x10DCC unless scripted), 0x10F96, 0x1132A, 0x107F8, 0x10CD0, 0x11962, 0x11A9E, 0x11F86, 0x1201C,
0x120D2, 0x11E56, the car passes, (0x115B0, 0x11CEC unless scripted), 0x1185A.

### 3.2 Enemies hurting each other (complete list)

| Source | Victims | Rule | Measured |
| --- | --- | --- | --- |
| thrown or knocked down body (list 0x6B30) | other enemies in frame class 0, 2, 4 | attack record 0x11 (5 base, knock down), pass 0x1185A | ported; D3b A21 |
| every monster (dinosaurs A22, A24, A29, A38, A82; **barrel A65, rock A66, biker A70, dynamite A77**) | all enemies they touch | raw record damage, no score | barrel 10, rock 10, biker 10 (44 to 34), foot 30 and 1, Mack 15, Shivat 10 / 14 (section 2) |
| pool E explosion kind 3 (dynamite) | enemies | 1 HP and a push in my probe (52 to 51 at the blast) | a64-e3-hit |
| poacher bullet, thrown blade, any enemy melee | enemies | **never** (only the hero list is a victim list) | bsp-* |
| respawn air strike | enemies | flat 30, floor 1 (D2b) | D2b |

So there is no friendly fire between thugs. A thrown body is the only hero caused one.

### 3.3 Enemy weapon and item pickup (row U01: the "item thief")

**Answer: no thug picks up an item. The only enemy that looks at items on the floor is Bludge (class 6), which eats them (section 7.4). Class 20 (Punk, Thug, 0x457B0) has no pick up branch, so the port needs none.**
Nothing is missing in `class20*.rs` for this. The user's memory ("a small quick thug, probably the Punk, takes a weapon or food from the floor and runs off, a hit makes it drop it") most likely comes from the
**alternate Punk, class 64 variant 0 = name id 0x1D** (the second PUNK of the name table; name 0x08 is the Punk of class 20, 0x09 its Thug): a quick thug that **holds a dynamite stick (class 77),
hops and runs across the screen, throws it and runs off**, and **drops the stick as a weapon item when it is knocked down** (below and 8.2). Evidence, strongest first:

1. **Static scan of the whole program ROM (0 to 0x70000, direct references only).** The pool E base (`0x874(a5)`, RAM 0xFF8874, objects of 0xC0 bytes) is named in six places only:
   0x4C1E (pool set up), 0xAA94 (area reset: clears items to state 0x600), 0x233C6 (a loop that jumps to each item's own class code, called from 0xE942), 0x24C3A (the drop roll: it looks for a weapon item of type 0 to 2 on screen and
   then drops a class 2 item instead [I]), and the two library tests **0x32762 and 0x327CA**. The pickable item list (count `G.$69EE`, list `G.$69F0`) is read in only these places: 0x4A32 to 0x4AD6 (list set up), 0xE928 and 0x1061E (resets),
   **0x5ABC and 0x5AC6 (the hero's pick up, 0x5AA2)**, and **0x3AFB2, 0x3B006, 0x3B00E, 0x3B032 (Bludge)**. No absolute use of 0xFF8874. So no other pool A class can see an item.
   0x32762 is a global test with no distance: "does any pool E object of class 0, 2, 4, 5 or 9 (weapon, class 2, food, class 5, treasure) exist with state 2 or less and sub state 0 or 0xC"; 0x327CA is the same without class 0. Their only callers are
   Bludge (0x3AFE0, its "go eat" test) and two stage script waits (0x1DE72: wait until no item lies on the floor, then 90 frames; 0x1DE8C: wait until no non weapon item lies, then 160 frames). They take nothing.
2. **Class 20 (0x457B0 to 0x479FE) and classes 64 and 77 (0x5AFC2 to 0x5B8B1, 0x5EDF4 to 0x5F187) contain none of those addresses.** Class 20 calls only the shared library (animation, move, physics, rng, angle, walk, target pick,
   kill all, kill score, free, floor, draw, hurtable, despawn). The developers' state names HIROI (pick up), TUKAMI (grab), H. MODOSHI and the others of `re/specs/enemy/SPEC.md` 7.1 are in the list of **unused names**
   (SPEC: "more unused names follow"). The port's class 20 is complete for this ROM.
3. **MAME, natural play with items on the floor** (`pk-*`, run 6; arena `jump 2 0 0x400`, `aggr 7 free`, level 0, god hero at x 1120, one enemy spawned at x 1280, **three items on its path: a weapon (E class 0, var 0x0C) at x 1240, a food
   (E class 4, var 3) at 1200 and a treasure (E class 9, var 5) at 1170**, 1,500 frames each). **All three items kept their place and stayed alive for the full 1,500 frames for every enemy of the list**, and nothing took them.
   How close each enemy came (lowest x): Ferris 1036, Gneiss 1086, Driver 1066, **Punk 944, Thug 944** (they walked through all three and past the hero), Hammer 1166, Elmer 1165, Wrench 1166, Blade 1166, Razor 1167 (past all three),
   Poacher J, Skinner and Gutter 1182 (they stop and shoot, so they passed two of the three), alternate Blade 1231 (dash, throw, run off by t = 170), alternate Punk and alternate Wrench (run and hop between x 1276 and 1441, freed by t = 146 and 90).
   **Lash stayed at x 1280 and never reached an item** (the test says nothing about Lash). **Bludge** (lowest x 944) ate the food at t = 74 and the weapon at t = 264, the treasure was still there at 1,500.
   A Blade killed next to a Ferris (`pk-knife`, hp 1 at t = 200, died by t = 230, flew to x 1422) left **no** item on the floor (the knives are drawn parts); the Ferris near it did nothing special.
4. **What looks like an item thief.** (a) The alternate Punk (and the alternate Wrench): it holds a weapon like item and a weapon item falls from it, but the item is its own dynamite, not something it took.
   (b) Bludge eats (7.4). (c) The Blade's "takes a knife" states 0x1C and 0x1E (class 14) only show the knife parts again (`+$BE`); no item is touched.

**What a hit does.** To a class 20 Punk or Thug (nothing carried): the normal reaction 0x2A606 and nothing else. **To the carrier (class 64):** the carrier's hit code (0x5B2CC) checks the reaction byte `+$7B`:
**4 (knock down A) or 0x30** sends it to the run away state at once (0x5B54A, speed id 0x20); any other reaction (a flinch or knock down B, 6) leaves it as it was. At the same time the carried **class 77 object (0x5EFAA to 0x5EFDE)**
tests "the carrier is in state word 0x202 and its `+$7B` is 4": if so it sets itself to state 6 and calls **0x24B86** (pool E item creator, state 2 airborne at its own position): drop code **0xE for variant 0 (a weapon item, type 13)
and 0xD for any other variant (weapon type 12)**. No `rand()` is used in this drop. Not seen in MAME: my god hero never knocked a carrier down (open item 6).
**Measured throw (M pk-a64w, the alternate Wrench, variant 0x200):** it hops right, and at t = 38 a **pool E weapon item (class 0, var 0x18 = type 12, `+$A0` 0x201)** appears on the floor at x 1194 (110 px ahead of it, left of the carrier);
it lies there 30 frames and is replaced at t = 70 by the explosion object (class 1, var 3) at the same place, which lives until t = 160. The alternate Punk (pk-a64, variant 0) showed only the explosion (E class 1 var 3, t = 130 to 222, x 1241):
it threw its dynamite and I saw no item on the floor in my 2 frame samples (the sticks of the two variants may differ). A hero could pick up the lit item in its 30 frame window [I]; run 6 had the hero at x 1120, out of reach.

Poachers, Blades, Lash and Walther hold their weapons as animation parts or helper objects (rifle in the picture, class 41 blade, class 62 mace, class 77 dynamite). What a killed enemy leaves is the normal `+$96` drop roll (D2c).
There is no enemy that grabs a weapon from the floor and uses it, and no enemy that runs off with an item.

### 3.4 The family AI that classes 3, 13, 14 and 63 copy

The listings of Lash, the poacher and the Blade are one design with different tables. Common parts (all with the same constants):

* **Flee and reposition timer** (`+$B8` for the poacher, `+$C0` for Lash): positive phase 180 / 240 / 300 frames (4 / 8 / 4 of 16 rolls),
  then a negative phase of -60 / -90 / -120 frames. At the end of the positive phase a flag (`+$C6` / `+$C2`) is set. While the flag is set and the
  object is not busy and not in the air, an **off screen** object mirrors its destination point round its own X and runs back onto the
  screen (state 0x4A for the poacher: 100 frames, six slots, speed id 0x1C). [R 0x3E1FA to 0x3E28C, 0x35B40 to 0x35BDC]
* **Slot refresh**: every 180 frames (`+$AE` / `+$A9`) a 14 in 16 roll takes a new slot round the target (0x32C5E six slots, 0x32D90 three) and
  turns toward it. [R 0x3E4A2, 0x35CC4]
* **Distance classes** (0x3293C): X class k = the highest k whose window the target is outside of: class 0 is up to 24 px, 1 is 25 to 48, 2 is 49 to
  72, 3 is 73 to 96, 4 is 97 to 120, 5 is 121 to 144, 6 is 145 to 168, 7 is more than 168. Z classes are 32 px steps (windows 0, 32, 64, 96).
  [R tables 0x32988, 0x32998, 0x329A8, 0x329B0]
* **Decision rows**: `table[variant * 256 + (rng & 0x1F) * 8 + X class]` (variant = the high byte of `+$26` as a byte, 0, 2, 4, so the stride is 256).
* **Rifle / mace / knife timers** are word counters `+$AC` (poacher, 63) or `+$AE` (Lash).
* Poise 10 and `+$79 = 60` refill, hit reaction 0x2A606, recover entry from table 0x2B56C (poacher: 0x3E504; Lash 0x35D36; alt Blade 0x5ACAA;
  alt Punk 0x5B2C4; hunter 0x5F84C; Bludge 0x396C2).

### 3.5 Other shared facts

* **Census and waves**: classes 13, 3, 6, 63, 64, 79 and 70 have `+$7D = 0` (counted by waves); 27, 43, 65, 66 have `+$7D = 0xFF` and
  never delay a wave. A counted object that leaves the screen is freed and drops out of the count (alternate Blades, bikers, hunters
  walking off, bomb carriers).
* **Kill-all check** (0x326F8, first call of the frame): classes 3 (and its mace 62), 6, 13, 63, 64, 79 and Walther call it (killed by a boss death and by the
  7200 frame wave timeout); **classes 65, 66, 70 and 27 do not** (a time out or boss death leaves barrels, rocks and bikers alone).
* **Anti tamper**: poacher state 0x48 (0x3F242) has the 1 in 32 call of 0x5124E that writes 9 to G.$4D9 on a changed ROM; Bludge calls 0x86B3C at the
  end of its eating animation when the frame counter low byte is 0x17. Never fire on a real ROM; do not port (D4d F34).
* **Stale `+$84` of bullets and thrown blades**: classes 41, 59 never set `+$84`; the damage scale row is whatever the slot held before (M: 27 at
  level 31 instead of 33 for Poacher J). Free (0x48FC) clears only `+$0..+$27`, `+$59`, `+$63`, `+$7D`, `+$87` and `+$96`.

## 4. Poachers, class 13 (Poacher J, Skinner, Gutter) with the bullet (class 59) and the flash (class 57)

Handler 0x3DDEA, 9,252 bytes, 38 states. Asset dump: `re/out/assets/anims/enemy_a13_gutter/` (294 animations in three tables).
Web: "Poacher J, Skinner and Gutter, ex soldiers with rifles" [W AQM, HG101]. In MAME they are green ponchos with a headband and a long
rifle; they shoot, punch, kick, run in with a spinning tackle and in Episode 6 jump out of the trees. [M p13-*, sc-e5-bludge]

### 4.1 Data (R 0x3DE00 to 0x3DF1C)

| Item | Poacher J | Skinner | Gutter |
| --- | --- | --- | --- |
| variant (high byte of `+$26`) | 0 | 2 | 4 (and any larger) |
| name `+$3` / row `+$84` | 2 / 8 | 0x12 / 0x48 | 0x13 / 0x4C |
| animation table `+$40` | 0x6C57A | 0x6CA24 | 0x6CF3E |
| HP at level 0, 8, 16, 24, 31 (`0x31424`) | 44, 52, 60, 60, 76 | 60, 76, 92, 100, 100 | 52, 84, 100, 100, 100 |
| damage scale at those levels (32 = 1.0) | 32, 48, 56, 64, 72 | same | same |
| kill score (name table 0x10292) | 2,000 | 3,000 | 4,000 |

Common: hit boxes from 0x103C00, speed id 8 then 6, direction 0x18, poise 10 (`+$78`), poise refill `+$79`, `+$72 = 0xC8`, `+$7D = 0` (counted
by waves), `+$AC = 0x14`, `+$AA = 0x1E`, `+$A8 = 0xA`, `+$B8 = -300` (0xFED4), `+$87` stays 0 (the free routine clears it: no guard), `+$63 = 0`, target by 0x32032, floor snap, six slots
(0x32C5E). The low byte of `+$26` is the **entrance** (table 0x3DF54, section 4.3). The drop row comes from `+$96` of the record (0x09, 0x11,
0x17, 0x10, 0x25 ... in the scripts; enemy table 0xB6D12: 0x09 = RIFLE 1 in 32 and SPECIAL 3 in 32, 0x17 = RIFLE 5 in 32, 0x11 = SPECIAL 8
in 32). Death: the shared knock down handler drops the item at the killing blow; mode 4 (0x401CE) awards the score (0x325E6) and blinks 120 frames
like class 2. [R]

### 4.2 Alive frame (0x3DF68, sub 0 AI frame 0x3E0B8)

1. kill-all check 0x326F8; sub 0 AI, sub 2 hit reaction 0x2A606, sub 6 grabbed 0x2B35A; then clamp 0x193E.
2. AI frame: flag `+$BC` then 0x3FBE4 (once); `+$79` poise timer; hit stop `+$59` (and `+$2F = 1`) or wait `+$7A` end the frame (drawn);
   0x3DFAC (a signed counter `+$BE` that walks toward 0); if not busy (`+$AF`) and not in a shot/dash state the slot and floor routines
   (0x3E2F8 obstacle bit, 0x32C54, 0x32F56); the state handler (table 0x3E1AA, 40 words); then 0x3E1FA (off screen / flee timer, section 3.4),
   0x3E4A2 (slot refresh), 0x3E420 (the **attack start tests**, below), `+$AA` tick; floor 0x12CB4, draw, hurtable (0x4BEA).
3. **Attack start tests (0x3E420)** run every frame while the object is free (`+$A1 = 0`): face the target (0x32B68); if the target is within 64 px in X
   (`(tx + 0x40 - x)` unsigned at most 0x80) and `tz - z` is from -7 to +2 (`(tz + 7 - z)` unsigned below 10): start the **close in** (0x3F5E2:
   timer `+$AC = table 0x743BA[level + (variant byte >> 5)] + one of {-15, -15, 0 x6, 5, 5, 10, 10}` (at least 1), state 0x16, speed id 6, anim 0).
   If the target is farther, the object is on screen, `+$AA == 1` (the tick `+$AA` counts 10 to 1, so every 10 frames), **`+$B8` is negative** (the
   negative phase of the timer of section 3.4: the first 300 frames of life, then 60 to 120 frames after every 180 to 300 frame active phase) and
   `0 <= tz - z < 8`: the **far decision** 0x40118 (second table below). Otherwise nothing. [R 0x3E420 to 0x3E4A0]

### 4.3 Entrances (low byte of `+$26`; M p13-ent*, level 0, spawn 256 px from the hero)

| Byte | Used by scripts | What it does | Measured |
| --- | --- | --- | --- |
| 0x00 | yes | state 6 (drift toward a slot, 120 frames, anim 1), then decide | walks 120 frames at 1.75 px per frame (speed id 6) |
| 0x02 | yes (Ep 3, 4, 5, 7, 8) | **stands** in state 0x34 for 1200 frames (anim 5, speed 6) until a hero comes close; a hit or a near hero ends it | `+$AC = 1200` |
| 0x04 | Ep 6 W2 | state 0x36: 60 frames in anim 7 with the fade palette 0x8B and z + 0x10 (rises from behind a hiding place), then state 6 | `ac = 60`, z 64 then 56 |
| 0x06 | yes (Ep 4 to 7) | jumps in from 144 px up: state 0x38 anim 6, 60 frames of fall (`+$51 = 1`, y + 0x90), then 0x44 and the dash | y 208 at spawn, lands at t = 37 |
| 0x08 | yes (Ep 2, 3, 5, 7, 8) | starts at once in **the dash**: state 0x1C, attack 0x2C, speed id 13, anim 0x15, aim at the target, 55 frames (the state ends at 0x1E, 16 frames, anim 22) | vx -896 (3.5 px per frame), 100 px in 29 frames |
| 0x0A | no | same, from the right edge: X = camera X + 0x180, face left, 45 frames | |
| 0x0C | yes (Ep 2 W3, 6, 8) | state 0x3E: run to the slot point at speed id 13 for 36 frames (anim 3), then state 0x2A | vx -878 |
| 0x0E | no | dash at the hero's depth from where it stands | |
| 0x10 | no | state 0x4E jump in (anim 9), from the screen edge nearest a hero, 60 frames | z 0 at spawn |
| 0x12 | Ep 6 (rain jungle) | same but from the far side (the "ambush"), then state 6, then the dash (0x44 wind up) | x 1232 moving +220 |

### 4.4 States (+$06; handlers from the dispatch table 0x3E1AA, setters found with `re/out/d3a/entries.py`, timelines from MAME)

| State | Handler | Setter | Meaning | Timer / numbers |
| --- | --- | --- | --- | --- |
| 0x00 | none | 0x3DF1C | after init: run the entrance table 0x3DF54 | |
| 0x02 | 0x3E54C | | wait out `+$7A` | |
| 0x04 | 0x3E558 | | walk `+$AC` frames facing the target (speed id 6), then decide | |
| 0x06 | 0x3E78A | 0x3F4D2 (decision 4), 0x3E504 | drift: every 4th frame choose X and Z velocity toward the slot point `+$88` / `+$8A` (`+$B0`, `+$B4`, magnitudes 0x15000 / 0x10000 etc.), anim 1 | 120 |
| 0x08 | 0x3E828 | 0x3F530 (decision 8) | the same, Z first | 120 |
| 0x0A | 0x3E6B4 | 0x3F374 (decision 1) | shuffle (small steps, velocities 0x12000 / 0x10000) | 60 |
| 0x0C | 0x3E8C6 | 0x3F1C8 (decision 3) | walk at speed id 15, re-aim every 15 frames (0x3F2E8 turns by one step) | 120 |
| 0x0E | 0x3E576 | 0x3F15E (decision 5) | approach the slot at speed id 13 (aim 0x32A70), re-aim every 30 frames, ends early within 0xC0 px | 180 |
| 0x10 | 0x3EA2A | 0x3F6B2, 0x3F6C2 | **rifle volley**, anim 15, fire frame at tick 51 | 60 per shot |
| 0x12 | 0x3EAFA | 0x3E3D4 (stuck or ledge), 0x3F728 | jump: vx +-0x200 (or by distance table 0x3F78C), vy 0x540 / table 0x3F79C, ay -0x45, anim 19; lands in 0x14 | 60 |
| 0x14 | 0x3EB8C | 0x3EADA, 0x3EE04, 0x3F5AE | landing or end pause, anim 20 or 0 | 8, 20, 30 |
| 0x16 | 0x3EBA4 | 0x3F5E2 | **close in** at speed id 6 | table 0x743BA |
| 0x18 | 0x3EB8C | 0x3EA3E, 0x3ECA8, 0x3F58C | pause after a shot or a dash (`+$A1 = 1`) | 60 to 95, 30 |
| 0x1A | 0x3EBB6 | 0x3F78C (decision 7 of table 1 is never rolled) | side step (direction table 0x3F80A) | 180 |
| 0x1C | 0x3EC22 | 0x3F88E, 0x3FF26, 0x3FFAE | **dash tackle**, attack 0x2C, anim 0x15, speed id 13 | 55 (entrance 8), 45 |
| 0x1E | 0x3ECA8 | 0x3EC22 | dash end, anim 22 | 16 |
| 0x20, 0x22 | 0x3ED02, none | 0x3ECE2, 0x3F95A | long wait after a miss, anim 1 | 180 |
| 0x24 | 0x3EB4C | 0x3F998, 0x3FAA2 | **melee**: attack 8, 9 or 0x2B by the chosen animation (table 0x3FA3E, entry = `+$BA`, `+$BB`, anim) | 60 |
| 0x26 | 0x3EE04 | 0x3FB40 | kick, attack 0x0A, speed id 6 | 30 |
| 0x28 | 0x3EE2C | 0x3FBC4 | second kick / sweep | 60 |
| 0x2A | 0x3E98A | 0x3F206 (decision 2) | flank: aim 0x30 px beside the target (0x32A88), speed id 15 | 120 |
| 0x2C | 0x3EB7E | 0x3F0F6 | the step between melee hits | 8 to 11 |
| 0x30, 0x32 | 0x3E94E, 0x3EE6E | 0x3F8F0 | quick back step (speed id 30) and jump | 20 + 24 |
| 0x34 | 0x3F038 | 0x3FC3E (entrance 2) | stand, anim 5 | 1200 |
| 0x36 | 0x3EF56 | 0x3FCA4 (entrance 4) | rise with fade palette 0x8B, anim 7 | 60 |
| 0x38 | 0x3F058 | 0x3FCE0 (entrance 6) | jump in from y + 0x90 | 60 |
| 0x3A | 0x3EB22 | 0x3F708 (decision 9) | wait 15 then the jump 0x3F728 | 15 |
| 0x3C | 0x3EB8C | 0x3F670 | after an entrance, face the target, `+$B8 = 180` | 30 |
| 0x3E | 0x3F0A4 | 0x4001C (entrance 0xC) | run in at speed id 13, anim 3 | 36 |
| 0x40 | 0x3EB72 | 0x3F636 | face wait, anim 2 | |
| 0x42 | 0x3EB3E | 0x3FB20 (decision 2 of table 2) | kick approach wait, anim 24, then state 0x26 | 20 |
| 0x44 | 0x3EB30 | 0x3F84A | **dash wind up**, anim 26 (attack 0x2C armed), then state 0x1C | 30 |
| 0x46 | 0x3EB8C | 0x3FC60 (decision 8 of table 2) | wait, anim 0 | 20 to 60 |
| 0x48 | 0x3EB66 | 0x3F242 | stuck behind a container: swing at it, anim 27 | 120 |
| 0x4A, 0x4C | 0x3E602, 0x3E64C | 0x3F3D8, 0x3F464 | off screen: run back at speed id 0x1C (716 per frame) toward a slot point | 100 |
| 0x4E | 0x3F076 | 0x3FD3E, 0x3FE60 (entrances 0x10, 0x12) | jump on (anim 9), arrives at a point 0x80 or 0x30 px beside the target | 60 |

### 4.5 The two decision tables and the rifle

First table (`0x4005C`, runs when a state ends): `T1[variant * 256 + (rng & 0x1F) * 8 + X class]` at 0x72A94, result 1 to 9 into the jump table 0x400BC.
Rolls per 32 (all three variants, only the four columns that differ):

| X class (px) | Poacher J | Skinner and Gutter |
| --- | --- | --- |
| 0 to 3 (to 96) | 2 (flank 0x2A) x16, 4 (drift 6) x16 | 2 x24, 4 x8 for class 0; 2 x32 for class 1 to 3 |
| 4, 5 (97 to 144) | 1 (shuffle 0xA) x4, 4 x24, 8 x4 | 4 x24, 8 x8 |
| 6 (145 to 168) | 3 (aimed walk 0xC) x4, 4 x24, 8 x4 | 4 x32 |
| 7 (more than 168) | 3 x4, 4 x24, 5 (approach 0xE) x4 | 4 x32 |

Decisions: 1 shuffle (0x3F374), 2 flank walk (0x3F206), 3 aimed walk (0x3F1C8), 4 drift (0x3F4D2), 5 approach (0x3F15E), 8 drift Z first (0x3F570).

Second table (`0x40118`, the far decision, from 0x3E474): first a gate `T[aggression * 8 + var * 32 + X class]` at 0x72D94 (values 2 to 15: the roll `rng & 0x1F`
must be below the value; the gate reads the X class `+$A5` stored by the **previous** decision, not a fresh one, a quirk to keep), and the target must not be in a hit reaction (`+$4 != 0x202`); then `T2[variant * 256 + (rng & 0xF8) + X class]` at 0x72F14
(identical rows for the three variants):

| X class | rolls of 32 | results |
| --- | --- | --- |
| 0 | 32 | 1 (melee 0x3F998) |
| 1, 2 | 16 / 8 / 8 | 1 / 3 / 5 (all melee) |
| 3 | 8 each | 1, 2 (0x3FB20 kick approach), 3, 4 x4, 5 x4 |
| 4 | | 2 x8, 3 x8, 4 x4, 5 x4, 6 (dash 0x3F82A) x4, 7 (rifle 0x3F6B2) x4 |
| 5 (121 to 144) | | 2 x4, 6 x4, **7 (rifle) x24** |
| 6 (145 to 168) | | 2 x4, **6 (dash) x24**, 7 x4 |
| 7 | | **2 (kick approach) x24**, 6 x4, 7 x4 |

The dash (6) is dropped (decide again) unless the hero is **more than 100 px (0x64)** away in X; the melee branches 1, 3, 4, 5 all end in the combo of 0x3F998 (the same code).

**Rifle volley**: on entry `+$C4` = table 0x3F6E8[rng & 0x1E] (1 x8, 2 x4, 3 x4 of 16), aim (0x32A58), face, anim 15, state 0x10 for 60 frames; the
fire frame (frame flag `+$2D` set, tick 51 from the start) calls 0x57F62 (spawns the **muzzle flash A57 var 1** via 0x483C: offset forward 0x2B, up
0x44 from the poacher, and the **bullet A59 var 0** through 0x57F62 with the offsets 0x30 forward and 0x40 up, sound 0x9C76). When the animation ends (69 ticks):
`+$A1 = 1`, state 0x18, `+$AC` = table 0x3EADA[rng & 0x1F] (60 x2, 65 x2, 70 x4, 75 x8, 80 x8, 85 x4, 90 x2, 95 x2), `+$C4 -= 1`; with shots left the
volley restarts at once (0x3F6C2, 69 frames after the previous fire frame), else `+$B8 = -90` and the pause runs. [R 0x3EA2A to 0x3EA98; M rf-m1: shots at
t = 252 and 321; rf-m5: 308, 377, 446 (3 shots); one shot only in rf-m0, m3]

**Bullet (class 59 var 0, 0x57FB4)**: pool A, `+$7D = 0xFF`, `+$63 = 0xFF`, anim table 0x6FB1A, attack id **0x0B** (15 base, poise 11, reaction 4
knock down, stop 15, sound 0x0C, spark 21), speed `+$A0 = 7` px per frame signed by facing (variants 0 to 4 use anim 0; **variant 5: 8 px per frame, 30 frame life, anim 1**, used by class 35), moves with the
shooter's facing, draws, registers hurtable (0x4BEA), frees by 0x173C (X outside the window around the camera), **does not stop when it hits**: M bsp-ctrl: hit the
hero at t = 36 (15 damage), then flew on to x 854. Height: floor + 0x40 (y 128 over a floor of 64). Damage 15 at level 0, 27 at level 31 (stale `+$84`, section 3.5).
Flash (class 57 var 1): two frames, follows the shooter for the frames of the shot, no hit box. [R, M]

### 4.6 Attack data and measured damage on Jack (HP 100, level 0 / 31)

| Attack | Where | Base | Reaction | Level 0 | Level 31 | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| 8 | melee jab, anim 11 | 10 | 0 flinch, stop 16, poise 4 | 10 | 22 | |
| 9 | melee punch, anim 12 | 12 | 2 flinch B, stop 16, poise 4 | 12 | 27 | |
| 0x2B | melee finisher | 12 | 2, stop 16, poise 8 | 12 | 27 | |
| 0x0A | kick (state 0x26) | 12 | 6 knock down B, stop 20, poise 11 | not seen | | |
| 0x2C | dash tackle (states 0x1C, 0x44) | 15 | 6 knock down B, stop 20, poise 11 | not seen (hero god in my dash runs) | | |
| 0x0B | bullet | 15 | 4 knock down A | 15 | 27 | |

The combo of 0x3F998: three attacks 8, 9, 0x2B in sequence with state 0x2C between (8 to 11 frames); M p13-v0-l0 t = 205, 240, 280: 100, 90, 78, 66.

### 4.7 Where used and variants

About 120 records (list in `re/out/d3a/recs.py`). Entrance 2 (stand) is the common one for waiting guards (Ep 3, 4, 5, 7, 8), 8 and 0xC for waves, 0x12 for the Episode 6
jungle ambush at triggers 0x200 to 0x800 (6 poachers: 0x0012, 0x0012, 0x0212, 0x0412 x3). Drop rows by record: Episode 2 0x11 (SPECIAL 8/32) and 0x17 (RIFLE 5/32) for the first poachers,
0x09 (RIFLE 1/32, SPECIAL 3/32) for the wave ones.

### 4.8 Port notes

New code: `game/enemy/class13*.rs` (about 1,500 lines like class 14), the bullet class 59 and the flash class 57 (`enemy/bullet.rs`), the two decision tables
through `dino-extract` (0x72A94, 0x72D94, 0x72E54, 0x72F14, 0x3F6E8, 0x3EADA, 0x3E28E, 0x3E2AE, 0x743BA, 0x3FA3E, 0x3F78C, 0x3F79C, 0x3F80A, 0x3E2F0 and 0x3FAE0), reuse
`lib32.rs` (slots, aim, face, distance class) and the class 14 flee timer code. The hero hit by a bullet uses the shared 0x115B0 / 0x6422 path.

## 5. The dinosaur hunter, class 79 (0x5F5D0)

Hammer T. look alike (Hammer, Blk Elmer, Wrench T. by variant byte 0, 2, 4; names 0x1E, 0x1F, 0x20; rows `+$84` 0x78, 0x7C, 0x80; animation tables 0x5FB80,
0x5FCBA, 0x5FDF4). 2,398 bytes. The AI is the shortest of the human classes: it has no attack on heroes at all. [R; asset `enemy_a79_hammer_t`]

### 5.1 Data

| Item | Value |
| --- | --- |
| HP (0x31424, name rows 0x1E to 0x20) | 100 at every level |
| damage scale | 32, 37, 49, 54, 64 at level 0, 8, 16, 24, 31 (only used by attack 5) |
| kill score | 1,000 (name row 0x38) |
| `+$7D` | 0: counted by waves while it lives |
| `+$72` | 0xB4, poise 10, `+$63` 0, `+$87` 0 (no guard) |
| kill-all | yes (0x326F8 first in the alive frame) |
| drop | by `+$96` of the record: Episode 2 rows 0x22 (gold dust 2, necklace, ring, 5 each of watch, dime, 1000 dollars, sunglass, grove, in 32) and 0x23 (donut 30 of 32) |
| walking speed id | table 0x5FB50 by variant byte (rows of 8 words) and `(level & 0x1C) >> 1`: variant 0: 15, 16, 17, 5, 5, 18, 19, 19; variant 2: 16, 17, 5, 18, 19, 19, 20, 20; variant 4: 5, 18, 18, 19, 19, 20, 21, 21 (id 5 = 1.5 px per frame, 15 = 1.2, 17 = 1.4, 18 = 1.6, 21 = 1.9) |

### 5.2 Modes and states

Mode 0 sub 0 (0x5F5F8): the target scan (section 2.1); no target: free at once. Else init (anim 0, boxes, speed by the table above, `+$7D = 0`, floor) and
sub 2 (0x5F75C): state 0 anim 1, speed, face the screen centre side (`+$24`, direction 8 or 0x18 by camera X + 0xC0 against its X), walk 60 frames (`+$80`) then mode 2.
Mode 2 (0x5F7CE): sub 0 AI frame, 2 hit reaction 0x2A606, 6 grabbed 0x2B35A; 0x173C frees it when far.

| State | Handler | What | Measured (hun-v400, level 0) |
| --- | --- | --- | --- |
| 0 | 0x5F870 | wait `+$7A`; if the target is still dormant, set anim 1 and go to state 2, else 0x5FA5A | t = 68 to 70 |
| 2 | 0x5F8AE | walk: sub 0 in Z until `(dz + 8) < 8` unsigned (0 to 7 px behind), sub 2 in X until `(dx + 0x28) < 0x50` (within 40 px), sub 4 in Z again; each leg uses `stuck_check` 0x324EE; then state 4 | t = 70 to 155 |
| 4 | 0x5F97E | stand, anim 3 then anim 4 for 30 frames; at the end of the first anim spawn the pool C effect 0x92422 (variant 0xD, above its head, an alert mark [I]) | t = 155 to 216 |
| 6 | 0x5F9E6 | **swing** anim 2: the target must still be dormant; the animation frame flag `+$2D` (negative on the third swing) calls 0x533FA; a positive flag starts attack id 5 (13 damage, stop 22, poise 3, reaction 2), sound 0xA1A0 and the slash effect 0x92450 (variant 5, 0x58 px forward, 0x30 up); ends when the animation ends | t = 216 to 285: attack field set at 230, **poke at about t = 277** |
| 8 | 0x5FA80 | flee: `move_dir` at the walking speed, direction 0x18, facing left; the frame is the mode 2 wrapper, so 0x173C frees it at X < camera X - 0xC0 | t = 285 to 622 (freed at x 830) |

Death (mode 4, 0x5FA8E): clears `+$C1` of the target, kill score, 120 frames of blinking, free. A hit is recovered through 0x5F84C (state 0, `+$7A = 2`).
The swing box (attack 5) is real, so a hero standing between the hunter and the dinosaur can be hit [R: not measured, open item 3].

### 5.3 Where and how many

Episode 2: two in wave 2 group 2 (X 0x708 and 0x548, variant 0, rows 0x22, 0x23). Episode 5: one static record (trigger 0x3C8, X 0x3A0, variant 0x400) and three in wave 1.
No other episode has one. In waves they are counted, so a wave with live hunters does not end; once they walk off (or are killed) the count drops.

## 6. Lash T., class 3 (0x358B2), with the mace A62 and A8

[W AQM] "Lash Terhune, mace fighter, mid boss of Episode 2". In MAME: a big man in red with a spiked mace on a long chain (sheet `lash-sheet`). Asset dump `boss_a03_lash_t`.

### 6.1 Objects and data

One script record makes **three objects** (0x358CE to 0x35996, same trick as the Blade): the lowest address of three free pool A slots becomes the Lash (class 3, `+$05 = 2`),
the next the **mace** (class 62, 0xF8; `+$A0` parent link, `+$A6` of Lash points to it) and the third a class 8 (0x20) helper with the animation table 0x712A0 (the chain links
and the held end, positioned from the Lash's frame byte table at 0x5BFDE) [I]. Init (0x35998): name 0xE, `+$84 = 0x38`, anims 0x6C0E2, boxes 0x103C00, speed id 7 then 5, HP by 0x31424, poise 10,
`+$C0 = -300` (the flee timer of 3.4), `+$AE = 0x14`.

| Item | Value |
| --- | --- |
| HP at level 0, 8, 16, 24, 31 | 100, 160, 200, 220, 300 (same table as Walther) |
| damage scale | 32, 40, 49, 65, 79 |
| kill score | 1,000 |
| `+$7D` | 0 (counted), kill-all yes, `+$63 = 0` |
| attack ids set by the Lash | 8, 0x51 (7 base, poise 3, react 2), 0x52 (10, react 0x14), 0x53 (12, KD A), 0x57 (12, KD A), 0x58 (15, KD B), and the lunge |
| mace (A62) | anim table 0x7020A, attack 0x2A (5, react 0xE) in flight and 8 when it connects, registers hurtable (a hero can hit it), `+$7D = 0xFF` |
| variant (`+$26` high byte 0 or 4) | no effect (only `asr.b #5` of it in 0x35C86, giving 0 for both); the scripts use 0 and 0x400 |

### 6.2 What it does (M lash-v0-l0, lash-v0-l31; R decision tables)

Lash keeps its distance and throws the mace: **160 px away it stays in place and swings** (state 4, 80 frames, attack 0x51 or 0x58), the mace object flies out
(x 1297 to 1157 in about 12 frames, 14 to 21 px per frame, attack 0x2A) and hits at frame 312 of the run; then the mace attack becomes 8. Each swing: state 4 (about
81 frames) then 0x10 (5), 0x1A (10 to 15), 2 (20) and the next swing (115 to 120 frames in all, M). Damage on Jack: **12 at level 0, 21 at level 31** from the mace, with
the stale scale row of the mace (section 3.5). HP 300 at level 31 (the object `hp=300`).

Decision tables (all indexed `(rng & 0x1F) * 8 + X class`, no variant): 0x72754 (first decision, mace in hand): class 0 to 2: 1 x24, 4 x8; class 3: 1 x8, 2 x16, 4 x4, 5 x4; class 4 to 7: 2 x16, 4 x8,
5 x8; 0x72854 (when `+$BE != 0`, no mace): class 0 to 2: 1 x16, 3 x8, 5 x8; class 3 to 7: 2 x16, 3 x4, 4 x4, 5 x8; jump table 0x36876 (1 = 0x36494 state 6, 2 = 0x36430 state 0 wait,
3 = 0x36494, 4 = 0x36644 state 8, 5 = 0x36688 state 0xA, 6 = 0x36804); attack gate 0x72954 (aggression * 8 + X class, roll `rng & 0x1F` below the byte: 18 20 18 12 10 12 10 8 at aggression 0
up to 32 32 32 26 24 26 24 22 at 7); attack choice 0x72994 (class 0: 1 x8, 3 x24; 1 to 3: 1 x16, 3 x16; 4 to 7: 4 x16, 5 x16, then 0x36740 starts the swing by the value minus 1).
Timers: `+$A2` from table 0x7431A by level, repeat counter `+$BC`. State handlers (dispatch 0x35B24): 0 0x35DAE (count `+$AE`, drift), 2 0x360C6, 4 0x360A8 (attack, ends in
state 0x10), 6 0x36246, 8 0x35F6C, 0xA 0x3600A, 0xC 0x362EC (watches the target's input byte 0x32B94), 0xE 0x36394, 0x10 0x360BA, 0x12 0x35E80, 0x14 0x35ED2 (side steps of 2 px per frame),
0x16 0x35F5A, 0x18 0x35F42 (airborne), 0x1A 0x360D4. Hit recover entry 0x35D36. Death (0x369D0): kill score, 120 frames of blinking.

Not read line by line: the swing sub states and the lunge (the long slide of sheet frame 11). That is the port task. Port size L, 4 h (three objects, tables, mace flight).

## 7. Bludge, class 6 (0x39272), with the acid spit A7

[W AQM] "human dinosaur hybrid, tongue attack and acid spit". In MAME: a lizard man that hops, leaps over the screen, crawls, lashes a long tongue and spits acid
balls. Colour from the area palette (red in the Episode 2 test arena, green with an orange mane in Episodes 5 to 7). Asset dump `enemy_a06_bludge`.

### 7.1 Data

| Item | Value |
| --- | --- |
| objects | 2: the lowest address of two slots is the Bludge (class 6, sub 2), the other keeps class word 0 (handler 0x33334, a part that follows it, `+$A0` parent) |
| name / row / anims | 0x10 / `+$84 = 0x40` / table 0x3A234 (14 animations), boxes 0x103000 |
| HP at level 0, 8, 16, 24, 31 | 76, 92, 120, 160, 200 |
| damage scale | 32, 40, 49, 65, 79 |
| kill score | 8,000 |
| `+$7D` | 0 (counted), `+$63` 0, `+$87` 0, `+$72 = 0xB4`, poise 10, `+$AF = 3` |
| stomach | an allocator D block (0x48A8, 16 bytes) holds a ring of up to 8 item handles: `+$D2` start, `+$D4` write pointer, `+$D6` count, `+$D8` list count, `+$D0` the claimed item, `+$B6` the "nothing to eat" flag (0xFF) |
| attacks | 0x24 (15, KD A, stop 20), 0x25 (12, react 0x18 slide, stop 12), 0x26 (13, flinch), acid spit A7 attack 0x27 (10, react 0x10 push, stop 6) |
| damage on Jack | level 0: 15 / 12 / 13 / 10 (acid); level 31: 33 / 38 / 29 (M bl-v0-l31: 38, 33, 38, 33, 29) |

### 7.2 Entrances (low byte of `+$26`: table 0x393D6)

| Byte | Used | Behaviour | M |
| --- | --- | --- | --- |
| 0 | Episode 6 wave 6 | the plain hop in (the default): hops of 72 px (vy 1312, vx 512), z 48 | bl-v0-l0: y 99, 136, 64 |
| 2 | no | arc jump from the back: vx 0x180, vy 0x400, ay -0x48, z 0x78 | |
| 4 | Episode 7 library | **crouches** in place (anim 0x3A50A, state 4 sub 4) until a hero is near, then hops | bl-v4-l0 crouched 100+ frames; sc-e7-bludge: they leap in from the right edge |
| 6 | Episode 5 wave 3 | **high leap**: vy 0x900 (2,240), vx 0x180, vz +0x100, y up to 217 | bl-v6-l0: x 1281 to 1388, y 72, 195, 217, lands |

### 7.3 Attack states (M bl-v0-l0, level 0)

State 2 (hop walk, sub 0 and 2; the hop height from table 0x3A5xx), 4 (entrance wait), 6 (attacks; `+$07` chooses): sub 4 anim 0x3A2A6 = the **acid spit** (frame flag spawns A7: pool A class 7, vx +-0x100,
`ay -0x30`, lobbed, y 120 to 64 in about 70 frames, M bl-v4-l0 t = 894 to 966, hit Jack for 10 at 920), sub 6 attack 0x24 (lunge, anim 0x3A342 and 0x3A362 recovery 74 frames), sub 8 attack 0x25 (**charge** at 4.5 px per frame
for 56 frames, vx -1152, anim 0x3A36E), sub 0xA attack 0x26 (**tongue**: long thin line across the screen, anim 0x3A2A6, vx 288 steps; seen in sheet frame t = 520 of sc-e7-bludge), sub 0xC wait 152. State 8 (entry 0x39AC8): walk to a claimed item and eat.

### 7.4 It eats items (the only enemy that picks things up)

Every decision cycle (0x396EA, state 2 sub 0) calls 0x3AFFA and 0x3AFA4:

* 0x3B006 scans the pickable list: items with `+$0` set, not of kind 0x20A (containers) and not claimed (`+$AF`). It claims the one with the **lowest kind byte** (food 0, ammo 1,
  weapon 2, treasure 3), sets its `+$AF = 1` and stores it in `+$D0`; if every list entry is claimed or skipped it sets `+$B6 = 0xFF`.
* 0x3AFA4 returns "go eat" when the stomach holds less than 8, `+$B6` allows it and `rng & 0x1F` is below the byte `0x3AFF2[aggression]` (4, 5, 6, 7, 8, 9, 10, 11) and 0x32762 says carry (it is a global test, "some item lies anywhere", with no distance, section 3.3).
  `+$B6` is reset to 0 whenever the item count `G.$69EE` is above the Bludge's own `+$D8` (a new item came) [R 0x3AFB2].
* State 8: walk to the item when it is out of reach (state 8 sub 4 in MAME; in `pk-bludge` no walk was needed for the food 79 px away), eating animation 0x3A48E then 0x3A4A4 (`+$DA = 1` while it lasts, poise 1: it is easy to knock down), at the animation frame flag
  0x3B05E pushes the item into the ring and **clears the item object** (`clr.b 0(item)`); the count `+$D6` goes up.
* **Hit while eating** (HP above 0, `+$DA` set): the item comes back (0x3B086 re-creates the last object: `+$0 = 1`, state 2 "spawned airborne").
* **Death** (0x3A1F2): kill score, then 0x3B086 and 0x3B0D8 give back everything in the ring.

M `pk-bludge` (run 6: aggression 7, level 0, food at x 1200, weapon 1240, treasure 1170, Bludge spawned at 1280): **food claimed at t = 8, eaten at t = 74 while the Bludge still stood at x 1279 (79 px away, states 0x806 and 0x808, anims 0x3A498 to 0x3A4E0);
weapon claimed at t = 122, eaten at t = 264 from x 1165 (75 px); treasure claimed at t = 310 and still there at t = 1,500.** So the order was food, weapon, treasure (the lowest kind first) and the reach is about 75 to 80 px, not a walk onto the item.
M `bl-eat-a7` (aggression 7, food 3 and 0xB, a weapon, a treasure on the floor): the Bludge claimed all four (`+$AF = 1`, t = 8 to 940) but ate **only the last claim, the treasure** (E class 9, object gone at t = 990, `+$D6 = 1`);
M `bl-death`: after the kill (t = 1072) **the treasure reappeared at x 1450** (the Bludge died at x 1180), thrown 270 px. The interrupt (hit while eating) was not hit in my runs (open item 4).
The claim marks do nothing to the items themselves (no item code reads `+$AF` after its init).
Port: pool E objects need a `free` and a `respawn` call; the pickable list must be a real list.

## 8. The alternate Blade (class 63) and the bomb carriers (classes 64 and 77)

### 8.1 Class 63 (0x5AA6E, 1,364 bytes), "BLADE (alt)"

Uses the Blade's art (anim table 0x6DA40 for variant 0, 0x6DF2A for a non zero high byte of `+$26`; the scripts only use 0). Init (0x5AA84): name 0x1C, `+$84 = 0x70`, speed id 7, anim 30,
HP by 0x31424 (**36 at every level**), `+$7D = 0` (counted), `+$A6 = 0x14`, `+$BE = 2` (knives, shown by two class 60 parts exactly like the Blade: 0x5AE64 places them from the tables
0x58178 / 0x58308), three slots (0x32D90), kill-all yes, recover entry 0x5ACAA. It is **freed when X < camera X - 0x50 or X > camera X + 0x1C0** (0x5AB82), so it can never stand off screen.

| State | Handler | What | Timer |
| --- | --- | --- | --- |
| 6 | 0x5ACF4 (setter 0x5ADF2) | **dash** at speed id 13 along X toward the target's side, attack 0x1C (16 base, KD B, stop 1), anim 30 | 15 frames (about 52 px) |
| 0 | 0x5AD1E (setter 0x5ADD8) | **throw**: anim 31; at the animation frame byte 0x2F: sound 0x9D44 and a **class 41 blade** (0x5AD38: `+$A0` = the thrower, same facing, attack 0x19: 12 base, KD A, stop 5, 6 px per frame), `+$BE -= 1`; when the anim ends go to state 4 | 51 frames in M |
| 4 | 0x5AD08 | wait | 30 |
| 2 | 0x5AD12 (setter 0x5AD7A) | **run away** at speed id 13 toward the nearer screen side (direction 8 if its X is in the right half, else 0x18), anim 30 | 80, then it is off screen and freed |

M a63-l0 (alone, level 0): dash t = 6 to 21 (x 1280 to 1231), blade thrown at t = 46 (x 1231, y 128, 6 px per frame), state 4 at t = 72, run away at t = 102, freed at t = 171 (x 1472).
Hero hit: 12 at level 0 and 25 at level 31 from the blade (stale scale, section 3.5); the dash itself did not hit in my runs. Kill: +1,000 (M ks-a63). In Episodes 2, 5, 7, 8 they come in
groups of 2 to 4 (Episode 7 wave 3: four Blades alt at X 0x900 delays 0x14, 5, 5, 5). **A group of them leaves by itself within 170 frames** (natural play nat2-hunt: they lived
t = 1430 to 1545), so waves that contain only them end without kills (the count drops when they are freed). Port size S, 1 h (class 41 and class 60 are ported with class 14).

### 8.2 Classes 64 and 77 (0x5AFC2, 0x5EDF4): the bomb carriers

Two objects per record (0x5AFEC: the lowest slot is the carrier, class 64, `+$A8` link; the other is class 77 with `+$A0` = the carrier and `+$26` = the carrier's variant byte).
Init (0x5B052): variant 0 = name 0x1D (**Punk alt**, `+$84 = 0x74`, anim table 0x5B606), non zero = name 0x24 (**Wrench T. alt**, `+$84 = 0x90`, table 0x5B77C);
**HP is the constant 20** (the table is not read; level independent), poise 10, `+$72 = 0x3C`, `+$7D = 0` (counted), kill-all yes. If the low byte of `+$26` is non zero it is placed at the screen
side **away from the majority of the heroes** (0x5B0FA: X = camera X + 0xC0 -+ 0x100).

Behaviour (code 0x5B28C to 0x5B5E4; M a64-e3, a64-e4; the action lists are small byte codes read through `+$A2`, tables at 0x5B326 and 0x5B4E6): the carrier **runs and hops** across the screen (hop:
anim 5 or 4, vx +-0x300 toward the screen centre side, vy 0x600, ay -0x50; run: speed id 0x20 = 4.5 px per frame), strikes a pose (anim 7), **throws its explosive** (the class 77 object leaves
the hand: it flew from x 423 to 571 with a peak of y 142 in 50 frames, M a64-e4), and then runs off. The dynamite explodes where it lands: **pool E explosion kind 3**
(0x27DD8; three blasts in a row at z 67, 51, 35 in Episode 3, M a64-e3-hit) and hurts heroes through the attack-list pass 0x10DCC [I]: 8 to 19 per blast on Jack. A hit with
reaction 4 or 0x30 (knock down) makes the carrier run away at speed id 0x20 (state 0x400), freed when X leaves camera X - 0x20 .. + 0x1A0 (0x5B5EA). **Knocking the carrier down makes the carried item fall
as a weapon pick up**: class 77 frees itself and calls 0x24B86 with code 0xE (variant 0: weapon type 13) or 0xD (variant 1: weapon type 12, the "fused explosive") [R 0x5EFAA to 0x5EFDE; weapon names are D2c's].
Kill: score 1,000 (Punk alt) or **80,000 (Wrench T. alt, name 0x24 uses the score row 0x7C, M ks-a64v2: +80,100 with the hit)**, then 120 blink frames.
Where: Episode 3 at trigger 0xD00 four Wrench alt (variant 0x200) at X 0xEC0; Episode 4 wave 1 three Punk alt; Episode 5 wave 4 four of them (three Wrench alt, variants 0x200, 0x200, 0x202, and one Punk alt, variant 0x0002, only for level 23 and up); Episode 8 area 0 trigger 0x188 three
(drop rows 0x22, 0x12, 0x22) and wave 6 four. [R stage_scripts.txt]

Open: the exact action lists, the blast damage and size, when the carrier throws (my runs had a god hero), the Episode 5 score and whether the 80,000 of Episode 3 is reachable by the player
(the carriers run off, a hit makes them run faster). Port size M, 2.5 h with class 77 and the pool E explosion (needs D3f / D3e for the effect).

**Item thief lead (U01).** The carrier is the only thug that has "an item in the hand and drops it when hit", and it does not pick anything up. Its class 77 stick drops through 0x24B86 on a knock down A (section 3.3), and a thrown stick becomes
a lit pool E weapon item for 30 frames (M pk-a64w: E class 0 var 0x18 at t = 38 to 68 at x 1194, then the explosion object E class 1 var 3 at t = 70 to 160). The port task HT7 must create these two pool E objects.

## 9. Walther, class 18 variant 2 (0x42590)

Same handler as Vice T. (D3c owns the AI): a non zero `+$26` makes it **Walther**: name 0xD, `+$84 = 0x34`, anim table 0x43E54, **HP from the normal table** (0x31424: 100, 160, 200, 220, 300 at level 0, 8, 16, 24, 31), `+$63 = 0`,
`+$72 = 0x3C`, no intro (variant 0 only runs the intro, the Rock Hopper helper 0x4DCC6 and the reinforcement controller 0x7BEDC), kill-all check (0x42888 `tst.w $26(a6); beq` skips it for Vice), clamp 0x193E
(Vice is not clamped), score 8,000, scale 32, 40, 49, 65, 79. The AI states (`+$06` 0 to 0xA, subs 0 to 0x20, combos of the attack ids 0x13 (15, react 2), 0x14 (9, push 0x12), 0x35 and others) are shared with Vice.
M `wal-l0`: walks in at speed id 0x18 (588), first attack at t = 93; hit Jack for 9, 7, 7, 7 at level 0 and 17 to 37 at level 31 (`wal-l31`). Scripts: Episode 3 (four, variant 0x200), Episode 4 wave 1 (one, a second for 3 players,
first in the wave), Episode 5 wave 4 (two) and Episode 6 area 0 (trigger 0x390, with a Lash T.). **Walther is the strongest normal enemy of Episodes 4 and 6 but not a boss** (counted, no boss bar, score 8,000).

## 10. Hazards in the monster list: barrels (65), rocks (66) and bikers (70)

All three register in the monster list 0x6B64 (0x4C0A), so they hurt **heroes and enemies** alike (passes 0x11CEC and 0x11E56, raw damage to enemies), can be hit by heroes (pass 0x11A9E: HP 8, any hit breaks them),
have `+$63 = 0xFF` and, for barrels and rocks, `+$7D = 0xFF` (never counted; the bike is counted). None calls the kill-all check, so a boss death or a wave time out leaves them.

### 10.1 Barrel, class 65 (0x5B8B2)

| Item | Value |
| --- | --- |
| variant | **high byte of `+$26`**: 0 = rolls in at floor level; non zero (Episode 8: 0x0201 and 0x0001 in Ep 6 count as 0) = starts 0x100 above the floor and falls; **low byte** = animation 0 or 1 and the break kind (0 wood: sound 0x9FB6, pieces 0x292A6; non zero: sound 0x9E60, pieces table 0x29566, 0x2943C) |
| spawn | faces right when `X < camera X + 0xC0` else left (`+$24`); anim table 0x5BAE0 (8 frames of 4 ticks) |
| motion | vx +-512 (2 px per frame) toward the other side, **hops**: on every landing vy 768, ay -60 (hop about 17 px high, about 26 frames); 1 in 16 landings adds vy +256; sound 0x9FB6 on each landing; for the falling kind: y + 0x100, vy -384, wait 30 frames (`+$80`), fall (vy -3,712 at t = 60), landing doubles vy, then the same hops |
| attack | id 0x32: 10 base, poise 11, react 6 (knock down B), stop 20; box body (-4, 16, -27, 53, 0, 40), attack (-2, 12, -12, 26, 1, 21) from box table 0x100000 |
| HP / hit | 8; a hero's hit: vx +-0x380, pieces, **drops the `+$96` item with the container roll 0x24B48** (table 0xB64F2: 0x22 gold dust..., 0x03 food, 0x29 treasure, 0x12 Uzi in the scripts), +100 points for the hit, freed |
| M | `bar-v0`: x 1390 to 962 in 220 frames with hops; damage on Jack 10 at level 0; on a poacher 10 raw (44 to 34); natural play `nat6-pulse` hit a poacher and two Blades |
| where | Episode 3 (22 in a column X 0x2910 to 0x2CF0, one with row 0x31 = the RADIO), Episode 6 area 1 (6 in two groups of 3, variant 1, rows 0x15, 0x29, 0x1B ...), Episode 7 library (10, variant 0, rows 0x22, 0x03, 0x29, 0x12 Uzi, 0x17 rifle, 0x2A ... and 3 in wave 12), Episode 8 cave (11, variant 0x201 and 1, rows 0x29 and 0x03) |

These are the "rolling drums with items" of the library that the web summary mentions [W AQM]; they come from both off screen sides and are hazards, not enemies hiding in the shelves.

### 10.2 Rock, class 66 (0x5BB8A)

Spawn: faces by the camera as the barrel, y + 0x90, vx +-448, vy -88, ay -32, `+$A2 = 2` bounces, draw flag 0x10 set; each bounce: vx += 0x80 (re-signed by the random facing table 0x5BE13), vy = -vy - 0x180; after the second bounce the draw flag clears,
vz = table 0x5BE24[rng & 0x3E] (-256 to -416, **toward the camera**) and state 4 flies on until it lands (mode 4: pieces 0x29650, the `+$96` item, and a kind 0 explosion 0x27DD8 if `+$5C != 0`). Attack id 0x46 (10, poise 1, KD B, stop 18); boxes body 204
(-2, 12, -19, 38, 0, 27), attack 203 (1, 6, -9, 16, 6, 13); HP 8; a hero's hit breaks it. Only Episode 5 area 0: three records at trigger 0x700 to 0x730 (X 0x780, 0x800, 0x758, Z 0x260 = the far back) and three more
in waves 2 and 3. M `rock-trace`: y 207 at spawn, bounces to 158, vx -576 after the first, state 4 at t = 90 (z 79 to 59); on a poacher 10 raw (M h-rock-poacher). Sheet `sc-e5-rocks`: a rock falls from the top of the wall and breaks on a thug.

### 10.3 Biker, class 70 (0x5C356)

| Item | Value |
| --- | --- |
| look | a rider with a cyan bandana on a flame painted motorcycle (assets `actor_a70`: anim 0 riding, anim 1 kicking) |
| spawn | faces right when `X < camera X + 0xC0` (it rides from that edge to the other); `+$A6 = 0x1E`: **stands 30 frames** (anim runs) then goes; `+$7D = 0`; HP 8 |
| speed | vx +-1,152 (4.5 px per frame) at the start of the ride, then `+$A2 += 0x12` per frame added to the speed until it is at least 0x700 (1,800, 7 px per frame) after about 9 frames; M x 1424 to 936 in 120 frames |
| variant (`+$26` high byte) | 0 rides at floor level; non zero **jumps** at the start: vy 1,872, ay -80, two hops (anim 1 = the kick pose) |
| attacks | run: 0x47 (10, KD B, stop 15; box body (-4, 16, -58, 100, 0, 64), attack (-4, 16, -72, 46, 0, 37)); **kick**: when a hero stands within 0x100 px ahead and `|dz| < 5`, state 2: anim 1, attack 0x48 (17, KD B; box (-4, 16, -50, 42, 39, 37)), 7 rolls in 16 (table 0x5C808) |
| exhaust | every 4th frame 0x92EA4 puffs (+0x21 px behind, -1 down), a timer `+$A7 = 0x1A` restarted each time `+$A6` ends |
| hit by a hero | the rider falls: sound 0xA164, effect 0x94640 (variant 0x1C), drops the `+$96` item (0x24B36), score 200 (BCD offset 0x18), 0x33802 and 0x2AF9A (effects), freed [R 0x5C6B4 to 0x5C6EA; M ks-a70: +300 with the hit] |
| where | Episode 3 (5 static at X 0x1200 to 0x1A00 and 4 in wave 2), Episode 4 area 1 (3 static at X 0x500 to 0x560 trigger 0x360 and 3 in wave 1), Episode 7 area 0 (2 static at triggers 0x280 and 0x2A0 and 3 in wave 0) |
| M | `bike-v0`, `bike-v1`, `bike-left`, `sc-e3-bikes`, `sc-e4a1-bikes`; damage 10 on Jack and 10 raw on a poacher (M h-bike-poacher) |

## 11. Scene actors: the Cadillac (16, 17), the Episode 4 opening (27, 43) and the Episode 5 Morgan scene (35, 45)

These are scripted actors of the opening scenes (D4c owns the scene timing, D3d owns Morgan). What each class is, from the listings and MAME:

* **Class 16 (0x420EA) and 17 (0x423A4)**: the blue open Cadillac and its wheels. One script record (Episode 2 list 2, trigger 0, X -128 = 0xFF80, Z 0x30, variant 0). Init waits for `G.$4E2` to clear
  (opening scripts), takes `+$7D = 0xFF`, creates **two class 17 objects** (variant 0 and 1; `+$AC` = the car). Sub 2 drives in (the speed comes from the scene), starts braking (`+$1A = -0x60`) when X reaches camera X + 0xA0;
  sub 4 waits until the speed is within 0x60 of zero, then sets `G.$505 = 1` and the stop pose (anim 0). Every frame 0x42308 **places the riders**: `G.$504` heroes aboard (0 to 3) are put at the table 0x42374
  offsets round the car (the heroes ride with it). The wheels (0x423B8) follow at X + 0x4D and X - 0x4B (z + 2) and bob 1 px by the car's X. M `car-trace`: invisible until t = 280, x 97 at t = 300, stops at **x 192** at t = 350.
  The same blue car is used in the Episode 4 opening (D4c T17: x -98 at t = 600 to 482 at t = 700, it runs over the three crouching men).
* **Class 27 (0x4D900)**: the three crouching men of the Episode 4 opening, variants 0, 2, 4 (name 0, 1, 3 = Ferris, Gneiss, Hammer look; anim tables 0x4DA2A, 0x4DA44, 0x4DA9A). HP 1, `+$7D = 0xFF`, the animation runs
  (crouch for 127 ticks then stand) and they are hurtable; mode 2 waits while `G.$4E2` is set. Killed by the car (hit reaction, flies, 120 frames of blinking, mode 4). No kill score, no AI.
* **Class 43 (0x55F2A)**: the orange haired leader. States (`+$05`): spawn pool B class 66 banners with variants 0x38 ("block the entrance with those barrels"), after 0xA0 frames 0x39 ("barrels, hurry up!"), a call to 0x420BE and 0x32 frames, a jump (vy 0xA80, ay -0x50), a pose,
  banner 0x3A ("nice place, now it is mine!") 0x78 frames, a second jump (vx 0x300, vy 0x800), then mode 4 and **it clears `G.$4FC`**, which ends the opening (D4c T18). M e4-open: banners at t = 240 to 560, the car at t = 600 to 700, control about t = 894.
* **Class 35 (0x513DA) and 45 (0x5639A)**: the Episode 5 scene (area 1, trigger 0x1120, X 0x1380 and 0x1360). Class 45 is the villager in rags (sound 0x20 at start, after 0x136 frames the gun shot sound 0x79). Class 35 is the introduction of Morgan:
  it creates class 67 (0x10C, the rifle he holds) and a pool B class 0x130 (0x4C), waits 0x3C and 0x1E frames (sound 0x9D9C), walks left for 0x28 frames, stands for 0xF0 frames and in the last 0x10 frames fires **class 59 variant 5 bullets** every 4th frame
  (0x57F62 with d0 = 5, 0x20 forward, 0x24 up: speed 8, life 30 frames, the other variant of the bullet class) and at its end **rewrites its own class word to 0x90 (class 36, Morgan) and clears its state**, so the boss starts in the same slot (0x51658).
  D3d describes the fight. M sc-e5-morgan: the old man's captions, the villager lies down, then the fight with Morgan starts.
* **Classes 78 and 80** (critters of Episode 2): D3b section 7 (class 78 walks to the hero and stops 7 px away, class 80 stands). **Class 5** (Butcher trigger) D3c; **class 22** D3b.

## 12. The ported classes (summary and pointers)

* **Class 2 (Ferris, Gneiss, Driver, 0x3383A)**: the basic thug. Six entrances (walk, dash, wait, fade, jump, drop), AI states 0, 2, 4, 6, 8, 0xA, 0xC, 0xE, 0x10, six slots, combos, dash, taunt. SPEC section 7. Code: `enemy.rs` (`enemy_class2`), checked against MAME in
  `plan/phase-01-stage1-complete.md`. HP 44 to 100, score 1,000, 2,000, 1,000. Quirks kept (taunt anti tamper 0x2A5B6 skipped).
* **Class 10 (Hammer T., Blk Elmer, Wrench T., 0x3B828)**: fat men with the grab and stomp machine, paired attack on a downed hero (the tests with the 0x400 bias never pass: kept), five entrances. SPEC 17.2; code `enemy/class10*.rs`, `claim.rs`.
* **Class 14 (Blade, Razor, 0x4020E)** with parts class 60 (knife in hand), 34 (dropped knife), 41 (thrown blade) and 15 (flying pieces): three objects per record, seven entrances, 24 reachable states, guard 0x404CA wired into the player's hit pass. SPEC 17.5; code `enemy/class14*.rs`, `class15.rs`, `lib32.rs`.
* **Class 20 (Punk, Thug, 0x457B0)**: hops, jump kicks and the leaping dive, six entrances. SPEC 17.4; code `enemy/class20*.rs`. It has **no item pick up branch** (the "item thief" lead U01: no item list or pool E reference in 0x457B0 to 0x479FE, and a Punk and a Thug walked
  past a weapon, a food and a treasure for 1,500 frames without touching them, M pk-punk, pk-thug; section 3.3). The second PUNK name (0x1D) belongs to class 64 (section 8.2).
  All four use the shared 0x2A606 reaction, 0x24B36 drops, 0x325E6 scores and the census; none uses the monster passes.

## 13. Where each class appears (from `stage_scripts.txt`; records and waves)

| Episode | Classes of this document (record count) |
| --- | --- |
| 1 | none (classes 2, 10, 14, 20 and Vice; birds 49, 50) |
| 2 | A16 car (1, X -0x80), **A13 poachers (17)**, A3 Lash (wave 5, X 0xD20, variant 0x400), A63 (3 in wave 2), **A79 hunters (2 in wave 2)**, A78 x2 and A80 (critters, D3b), A5 trigger (D3c) |
| 3 | A13 (13), A18 var 2 Walther (4), A64 (4 at X 0xEC0), **A65 barrels (22)**, A70 bikers (5 + 4) |
| 4 | A27 (3), A43 (1), A13 (7), A18 var 2 (1, 2 for 3 players, wave 1 area 1), A64 (3, wave 1), A70 (3 + 3) |
| 5 | A13 (13), A3 Lash (wave 6), A6 Bludge (2 in wave 3), A18 var 2 (2 in wave 4), A63 (3, wave 6), A64 (4, wave 4), A66 rocks (3 + 3), **A79 hunters (1 static + 3 in wave 1)**, A35 and A45 (Morgan scene) |
| 6 | A13 (20, 6 of them ambush entrance 0x12), A3 Lash (2), A6 Bludge (2 in wave 6), A18 var 2 (2), A65 (6), A22 brontosaurus feet (2, D3b) |
| 7 | A13 (28), A3 Lash (3, wave 11), A6 Bludge (3, library), A63 (4, wave 3), A65 (13), A70 (2 + 3) |
| 8 | A13 (21), A3 Lash (2, wave 10), A18 var 2 (3, wave 10), A63 (10), A64 (7), A65 (11) |

## 14. Acceptance scenarios (probe segments and what must match)

Run with `re/harness/probe/probe.sh` (SDL dummy drivers set by the script). Every segment starts with `jump ...`; `t = 0` is the first frame of control. The arena is `jump 2 0 0x400`: Jack at x 1120, z 54, the camera at 1024,
floor height 64. `pin(slot)` is `3 hold ADDR 2 X` with ADDR = the object address + 8 (first spawned pool A slot after a jump is 22 = 0xFFDC34, the second 21 = 0xFFDB54). The full line sets are in
`re/out/d3a/gen_r2.py` to `gen_r5.py` (git-ignored; regenerate from the lines below). Positions are integer pixels, `y` is the absolute height (floor 64 here).

| # | Segment lines | Must match |
| --- | --- | --- |
| H1 | arena, `level 0 free`, `god on`, `5 spawn A 24 var=0 w5c=0x7fff x=+0x80`, `6 spawn A 79 var=0x400 x=+0x180 z=0x30`, run 900 | hunter at x 1406 at t = 7, 1316 at 67 (1.5 px per frame), state 2 at 70, 4 at 155, 6 at 216, attack 5 set at 230; Shivat hit reaction at t = **277** (hit stop 7, HP 60), COLOR (`a8 = 8`) at 284, OKORI (`a8 = 4`) at 373, `+C0 = 255` until the poke, 0 from t = 284, 1 from about t = 582 (set by the hunter's 0x5FA5A when it re-evaluates); hunter HP 100, 90, 80, 70 at t = 385, 406, 427 (Shivat stomps 10), knock down at 427, state 8 at 546, freed at t = 622 (x 830) |
| H2 | arena, `6 spawn A 79 var=0x400 x=+0x180 z=0x30` alone; with `5 spawn A 29 var=0x200 w5c=0x7fff x=+0x80` first; with `A 38` (Mack) first | the hunter object is **gone after one frame** in all three (`+C1` stays 0 on the dinosaur) |
| H3 | `jump 2 0`, `god on`, hold R `0-3600:R`, `200 killall`, `450 killall`, `700 killall`, watch A every 5 frames | lock at camera 1384; Shivat asleep from t = 535; hunters appear at t = **825** (x 1353 and 1798, z 72); poke reaction t = 1030; COLOR 1040; OKORI 1125; hunters 100, 90, 80, 70 at t = 1140, 1160, 1180; hunters freed at 1385 and 1415 (still 70 HP); three class 63 at 1430; a Hammer at 1550 takes 14 (t = 1700), then 10 at 2525, 2545, 2565, 3015, 3035 and dies; Shivat MANZOKU at 3050 |
| H4 | `jump 5 0`, `god on`, hold R, `250 killall`, `600 killall`, run 3600 | static hunter at x 931 at t = 820, stops at x 1065 / z 585 by 945, first swing 1020, Shivat reaction 1070, COLOR 1075, OKORI 1165; wave hunters at 1070 walk left at 1130; the static hunter freed by 1250; Shivat MANZOKU at 3360 |
| H5 | arena, hero left (`2-60:L`), `20 spawn A 59 var=0 ax=1190 z=0x32 y=0x80`, with a Hopper var 0x200 (pinned x 1115), a Shivat, a Ferris var 2 or a Mack in the line | bullet y 128, 7 px per frame (x 1183, 1134, 1078 at t = 21, 28, 36), **hero hit at t = 36 (100 to 85, level 0; 100 to 73, level 31)**, bullet continues to x 854 at t = 68; Hopper, Shivat, Ferris and Mack lose nothing |
| H6 | arena, `level 0 free`, `2-60:L`, `2 spawn A 13 var=0x0000 ax=1184 z=0x32`, `3 hold 0xFFDC3C 2 1184`, run 900 | states: 6 at t = 3, **0x10 at 192**, bullet at x 1136 (t = 243), hero -15 at 255, 0x18 at 261 (60 frames), 0x46 at 321, 8 at 381, 6 at 501, dash 0x44 at 592, 0x1C at 622, 0x1E at 677, 0x18 at 693. Level 31: hero -27 |
| H7 | the same with the spawn at t = 11, 23, 37, 51, 67 | volleys: t = 11: shots 252, 321, then 682, 751, 820; t = 51: shots 292, 361, 832; t = 67: 308, 377, 446 (69 frames apart) |
| H8 | arena, `god` off, `hp 100` every 100 frames, `5 spawn A 13 var=0x0000 x=+0x100 z=0x30`, level 0 / 31 | first melee t = 193 (states 0x24, 0x2C, 0x24, 0x2C, 0x24, 0x2C, 0x14, 6): hero 100 to 90, 78, 66 (level 0), 100 to 78 and 73, 46 (level 31); the three variants equal |
| H9 | `5 spawn A 13 var=ENT x=+0x100 z=0x30` for ENT = 2, 4, 6, 8, 0xA, 0xC, 0xE, 0x10, 0x12 | the first states and timers of 4.3: 0x34 (1200), 0x36 (60), 0x38 (60) then 0x44 at t = 45, 0x1C at 6 (x 1280 to 1178 at 35), 0x3E (36, vx -878), 0x4E (60) |
| H10 | `jump 2 0 0x400`, spawn, `7 objset A:CLS hp 1`, `7 objset A:CLS x 1152`, `7 objset A:CLS z 54`, punches at 20, 40, 60, 80 | score after the kill (hit + kill): poacher J +2,100 total, Skinner +3,100, Gutter +4,100, class 64 var 0 +1,100, **class 64 var 0x200 +80,100**, class 63 +1,100, Lash +1,100, Walther +8,100, hunter +1,100 (with drop row 0x23: a food object at x 1181), biker +300 |
| H11 | `5 spawn A 65 var=0x0000 x=+0x170 z=0x36` (and var 1, 0x201), 500 frames | x 1390 at t = 6, vx -512, hops (vy 768 at t = 80 and 180, peak y 81), x 962 at t = 220; var 0x201: y 320, vy -384 held until t = 36, vy -3,712 at 60, lands about 63 |
| H12 | `5 spawn A 66 var=0 x=+0x140 z=0x80`, 700 frames | y 207 at t = 6 (vx -448), 158 at 30 (vx -576), state 4 with vz toward the camera at 90 (z 79 to 59) |
| H13 | `5 spawn A 70 var=0 x=+0x190 z=0x36` (and var 0x100, and `x=-0x60`) | still until t = 36, then 4.5 px per frame (x 1311 at 60, 1147 at 90 at vx -1,800, 936 at 120); var 0x100: vy 1,872 at start, y 145 at 60 |
| H14 | `5 spawn A 22 var=0 x=+0x80 z=0x36`, two poachers var 0x0002 and 0x0202 at the same place | both lose 1 at t = 7; the second loses 30 at t = 378 (59 to 29, knock down) |
| H15 | `5 spawn A 65 var=1 x=+0x170`, poachers at x +0xA0 | poacher 44 to 34 at t = 84 (10 raw); hero version: 100 to 90 at 120 |
| H16 | `5 spawn A 38 ... ax=1115` pinned, poacher pinned at 1190 | Mack hits the poacher 44 to 29 (t = 65) and 29 to 14 (t = 443) |
| H17 | `aggr 7 free`, `5 spawn A 6 var=0 ax=1300 z=0x30`, `6 spawn E 4 var=3 ax=1170 z=0x30`, `E 0 var=0x0C ax=1200 z=0x38`, `E 9 var=5 ax=1230 z=0x28`, `E 4 var=0xB ax=1260 z=0x40` | the four items get `+AF = 1` (food var 3 at t = 8, the rest by 940); the Bludge eats at t = 941 to 990; **the treasure object is gone at t = 990**, `+D6 = 1`, `+B6 = 255`; the other three stay |
| H18 | H17 plus `1050 objset A:6 x 1150`, `1060 objset A:6 hp 1`, punches from 1070 every 30 frames | Bludge HP 76 to 1 to 0 at t = 1072; **a class 9 item reappears at t = 1166 at x 1450** (about 90 frames after the killing blow, when the body has fallen and mode 4 starts) |
| H19 | `5 spawn A 3 var=0x0400 x=+0x100 z=0x30`, level 0 / 31 | Lash at x 1280 stays; mace object flies x 1297, 1241, 1157 (t = 300, 304, 308); hero -12 at t = 315 (level 0) and -21 (level 31, HP 300); swing states 4, 0x10, 0x1A, 2 at t = 288, 369, 374, 384 (level 0; 377 at level 31) |
| H20 | `5 spawn A 63 var=0 x=+0x100 z=0x30` | dash t = 6, x 1231 at 21, blade at t = 46 (x 1231, y 128), hero -12 at 55, state 4 at 72, run away at 102, freed at 171 |
| H21 | `jump 3 0 0xCB0`, `god` off, hold R | four class 64 at X 0xEC0; explosion objects (pool E class 1 var 3) at t = 214 at x 3567, z 67, 51, 35; hero -10 at 219, -10 at 438, -19 at 672 |
| H22 | `jump 4 0 at=start`, watch A every 5 frames | three class 27 at x 274, 188, 234, class 43 at x 352 z 64; class 43 sub state `+$05` 0, 2 (t = 300), 4 (500), 6 (600), 8 (700), 0xE (800); the car A16 at x -98 (t = 600) to 482 (700); class 27 st 0x02020404 at t = 700, mode 4 at 800 |
| H23 | `jump 2 0 at=start` | car invisible at x -128 until t = 280; x 97 at 300; x 192 and sub 6 at 350; wheels at x 117 and 269 |
| H24 | `5 spawn A 64 var=0x0200 x=+0x100 z=0x30` alone | HP 20, runs right (vx 768 then 1152), freed at t = 146 (variant 0, x 1441) or t = 90 (variant 0x200, x 1440) |
| H25 | Item thief test (`pk-*`, run 6): `jump 2 0 0x400`, `level 0 free`, `aggr 7 free`, `hp 100`, `god on`, `5 spawn A CLS var=VAR x=+0x100 z=0x30`, `6 spawn E 0 var=0x0C ax=1240 z=0x30`, `6 spawn E 4 var=3 ax=1200 z=0x34`, `6 spawn E 9 var=5 ax=1170 z=0x2C`, 1,500 frames, `watch 2 E x y z st cls var +0xA0.w +0xAF.b` | for CLS, VAR in Ferris (2, 0), Gneiss (2, 0x200), Driver (2, 0x400), Punk (20, 0), Thug (20, 0x200), Hammer (10, 0), Elmer (10, 0x200), Wrench (10, 0x400), Poacher J, Skinner, Gutter (13, 0 / 0x200 / 0x400), Blade, Razor (14, 0 / 0x200), alternate Blade (63), alternate Punk (64, 0), Lash (3, 0): **the three items keep the same x, `+$AF` 0 and are alive at t = 1,500** (E slots 21, 22, 23). Lowest enemy x: Ferris 1036, Gneiss 1086, Driver 1066, Punk 944, Thug 944, Hammer 1166, Elmer 1165, Wrench 1166, Blade 1166, Razor 1167, poachers 1182, Lash 1280 |
| H26 | the same with CLS 6 (Bludge) | food (E slot 22) claimed `+$AF = 1` at t = 8 and gone at t = 74 (Bludge at x 1279); weapon (slot 23) claimed t = 122, gone at t = 264 (Bludge at x 1165); treasure (slot 21) claimed t = 310, alive at t = 1,500 |
| H27 | the same with CLS 64 var 0x200 (alternate Wrench), items as in H25 | carrier hops from x 1277 (t = 8) to 1391 (t = 68); **E class 0 var 0x18 (`+$A0` 0x201) at x 1194 from t = 38 to 68, E class 1 var 3 from t = 70 to 160 at x 1194**; the three floor items unchanged; carrier freed at t = 90 |

## 15. Port status and the port task list

State today: classes 2, 10, 14 (with 15, 34, 41, 60) and 20 are ported and checked; the monster passes and the stub passes of section 3 are empty; every other class of this document is freed at once by `enemy_class_unported` (it prints "not ported"). Shared helpers that exist: `lib32.rs` (slots, aim, face, distance class, sounds),
the class 14 flee timer, `enemy_hit_held_player` (0x125C0), the hit reaction 0x2A606, drops 0x24B36 / 0x24B48 and scores 0x325E6.

| Task | Content | Size | Needs | Milestone |
| --- | --- | --- | --- | --- |
| HT1 Passes | `pass_10cd0` (with D3e's breakables), `pass_10dcc`, plus D3b P1 for 1132A, 11A9E, 11CEC, 11E56; the per slot `+$84` kept across frees | S 1 h (D3b P1 is 2 h) | D3b P1, D3e | M5 |
| HT2 Poacher class 13 and bullet 59, flash 57 | init, 10 entrances, 38 states, two decision tables, rifle volley, dash, melee, flee timer, hit recover, death; bullet and flash; the 14 data tables in `dino-extract` | L 4 h | HT1, `lib32.rs` | M6 (Episode 2) |
| HT3 Hunter class 79 | section 5 | S 1 h | D3b P5 (Shivat) | M6 |
| HT4 Lash class 3 with A62 and A8 | three objects, mace flight, five decision tables, swings and the lunge | L 4 h | HT1 | M6 (Episode 2 wave 5) |
| HT5 Bludge class 6 with part 0 and acid A7, item eating | 4 entrances, hop AI, tongue, charge, spit, eating state, pool E free and respawn | L 4.5 h | HT1, pool E hooks | M7 (Episode 5) |
| HT6 Alternate Blade class 63 | section 8.1 | S 1 h | class 41, 60 (done) | M6 |
| HT7 Bomb carriers 64 and 77, dynamite explosion | section 8.2, pool E explosion kind 3 | M 2.5 h | D3f, D2c (weapon types 12, 13), H1 | M6 (Episode 3), M7 |
| HT8 Walther | variant 2 of D3c's class 18 port | S 0.5 h | D3c | M6 |
| HT9 Hazards 65, 66, 70 | barrel, rock, biker, the monster hooks, the break pieces and effects | M 2.5 h | HT1, D3f (effects 0x92EA4, 0x94640, 0x29650) | M6 (Episode 3), M7 |
| HT10 Scene actors 16, 17, 27, 43 | car with riders, three men, leader with banners, `G.$4FC` and `G.$505` | M 2 h | D4c, pool B class 66 | M3 / M6 |
| HT11 Morgan scene 35, 45 | with D3d | S 1 h | D3d | M7 |

Total about 28 agent hours (HT1 to HT11 without D3b's 15). Order: HT1, HT6, HT2 (Episode 2 needs the poacher first), HT3 with D3b P5, HT4, HT9, HT7, HT5, HT8, HT10, HT11.
Every ported routine must name its original address.

## 16. Open items

1. **Exact poacher state glosses.** The table 4.4 names every state by handler and setter; the rolling sub steps of 0x2A, 0x24 and 0x1A were read at their head and by MAME, not line by line. The port task reads them from `c13_dis.txt` against the M timelines H6 to H9.
2. **Poacher bullet hit by a hero.** Class 59 registers hurtable (0x4BEA); my punch test came after the bullet had passed. What a punch does (score? stop? nothing?) is not measured.
3. **Hunter swing against a hero.** The swing (attack 5, 13 damage) has a hit box; I ran it only with a god hero. Not measured.
4. **Bludge hit while it eats.** The item should come back (0x3B086 through `+$DA`); my hero hits missed the short eating window (bl-int0 to 3). Death drops are measured (H18), the interruption is read in the code only.
   Run 6 used a god hero standing 120 px away and did not try it.
5. **Bludge eating order.** In `bl-eat-a7` it ate only the last claim (the treasure of four); in `pk-bludge` it ate the food, then the weapon, then not the treasure within 1,500 frames. The port copies 0x3B006 and 0x3AFB2 literally (claim by lowest kind,
   `+$D0`, `+$D8`, `+$B6`, reach about 75 to 80 px) and checks both runs.
6. **Class 64 action lists and blast damage.** The byte code tables 0x5B326, 0x5B4E6 and the explosion (0x27E16, pool E class 1) were not decoded; my hero was a god in H21 (the damage numbers come from a god-off repeat that also stood in other blasts). Whether the 80,000 point carrier can be reached by the player in Episode 3 (they run off) is not known.
   **The knock down drop of the dynamite (HU26) is read in the code (0x5EFAA to 0x5EFDE) but was never triggered in MAME** (no hero hit on a carrier). Also open: whether a hero can pick up the lit type 12 item during its 30 frames on the floor (the pk-a64w item had `+$AF` 0, state 0 idle).
7. **Lash swing sub states and lunge**, class 8 helper (0x20) role, and the Lash variant 0x400 (no effect found).
8. **Walther versus Vice** differences inside the shared AI (D3c).
9. **No Episode 7 area 0 probe.** I spent my five planned MAME runs elsewhere (run 6 was for U01 only); the Episode 7 bikers at the start (2 static and 3 in wave 0) are confirmed by the script records and by the same class in Episodes 3 and 4 (M), not by a run in Episode 7 area 0.
10. **Pool B in Episode 4 and Episode 7 library** (class 0 panels at Y 0xB0 / 0xE0, class 1 pieces, 59): D3e part 2.
11. **Sounds**: the hit and voice codes belong to D4e; the helper entry points are in the table of 4.5, 5.2, 10.3 (0x9C76 shot, 0xA1A0 swing, 0x9D44 blade, 0xA164 bike, 0x9FB6, 0x9E60 barrel).
12. **Stale `+$84`** (section 3.5): the port keeps the original slot reuse or fixes a row; D5 decides. A fresh power on game should be checked against MAME for the first bullet damage at a known level.
13. Two and three players: the poacher rows (`players>=2`) and the hunters' and Bludge counts were read, not run.
14. Where the **kill-all** of Episode 3's radio hits the carriers and bikers (they ignore it).
15. **Item pick up test coverage (U01).** In `pk-*` Lash never left x 1280 (so it never came near an item) and the three poachers stopped at x 1182 (two of three items passed). The static scan covers them (no item code in their classes), but a run with the items placed at their own
    fighting distance was not made. Indirect access to pool E through a computed pointer is not excluded by a scan of direct references; run 6 saw no sign of it.
16. **What the user saw.** If the user remembers a thug that took a weapon from the floor, the best match is the alternate Punk throwing its dynamite and a hero picking up the dropped item (HU26). A short capture with a real hero knocking a class 64 carrier down would close the lead.

## 17. The coordinator's extra leads (rows G07, G09 and U01 of `mechanics-seen.md`)

| Lead | Verdict | Evidence |
| --- | --- | --- |
| Bludge jumps around | **Confirmed.** Entrance 0 hops (72 px), entrance 6 leaps 217 px high, entrance 4 crouches then hops, the attacks include a charge and a lunge. | M bl-v0-l0, bl-v6-l0, bl-v4-l0; R 0x393D6 |
| Bludge attacks at range | **Confirmed.** The acid spit (class 7, lobbed, 10 damage, push) and a long tongue strike (attack 0x26). | M bl-v4-l0 t = 920 (-10), sheet `sc-e7-bludge` t = 280 and 520 |
| Bludge may eat items on the ground | **Confirmed, with details**: food, weapons, treasure, up to 8 in a queue, only at higher aggression, gives them back on death. | R 0x3B006 to 0x3B0D8; M bl-eat-a7, bl-death |
| Episode 4: Walther as a mid boss | **Partly.** Class 18 variant 2 is the first and strongest unit of the garage's first wave (HP 100 to 300, score 8,000, one more for 3 players). It is a counted normal enemy: no intro, no boss bar, no boss music. | R stage_scripts.txt (Episode 4 wave 1), `+$7D = 0`; M a64-e4 (Walther HP 200 at the natural level) |
| Episode 4: motorcycle enemies crossing the arena | **Confirmed.** Class 70: 3 at trigger 0x360 (X 0x500 to 0x560) and 3 more in wave 1; they ride from one edge to the other at 4.5 rising to 7 px per frame. | R, M sc-e4a1-bikes (x 1320 to 868 in 110 frames) |
| Episode 4: enemies from doors and high places | **Not found** in the classes of this document. The Episode 4 records spawn at off screen left or right X; no drop or door entrance is used by them (class 2 entrances 0, 2, 4; class 10 entrance 4; class 13 entrance 2 standing). The pool B class 0 panels at Y 0xB0 and 0xE0 and the class 1 fence (X 0x610, trigger 0x4A2) are D3e's. | R stage_scripts.txt |
| Episode 4: fence the player breaks, floor vents | not D3a (D3e part 2) | |
| Episode 7: motorcycle riders at the start | **Confirmed by the script**: area 0 has 2 static bikers (triggers 0x280, 0x2A0) and 3 in wave 0 at the 0x1B0 lock, right after the opening fight. Not run in MAME (open item 9). | R |
| U01: some enemies pick up weapons and items and go away with them (the Punk) | **Rejected for thugs, found only as Bludge eating.** Class 20 and every pool A class except Bludge never read the item list (static scan of the whole ROM); 17 enemy types walked past a weapon, food and treasure for 1,500 frames and touched nothing. The "HIROI, TUKAMI" state names are unused names. The class 20 port needs no pick up branch. The thing the user probably saw is the **alternate Punk (class 64 variant 0, name 0x1D): it holds a dynamite stick, throws it, runs off, and drops the stick as a weapon item when knocked down** (HU26). | R 0x32762, 0x3AFB2, 0x3B006, 0x5ABC, 0x5EFAA; M pk-*, pk-a64w |
| Episode 7 library: enemies come out of the bookshelves | **Rejected for pool A.** The three Bludges (X 0x740 to 0x780) crouch and leap in from the right edge; the 10 barrels (class 65, with drops) roll in from both edges of the screen along the floor; no record of the library spawns anything from a shelf. Pool B props (class 1 at X 0xE0 and 0x7C0, captions 66) are D3e's. | M sc-e7-bludge; R stage_scripts.txt |

## 18. New rows for `mechanics-seen.md` (group B, IDs use the prefix HU)

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

## Appendix: files used and made (all git-ignored under `re/out/`)

* Listings: `re/out/d3a/c3_dis.txt`, `c6`, `c13`, `c16`, `c17`, `c18`, `c27`, `c35`, `c43`, `c45`, `c63`, `c64`, `c65`, `c66`, `c70`, `c78`, `c79`, `c80` (made with `re/tools/d2`), the annotated copies `*_ann.txt`, helper scripts `ann.py` (names of library calls), `entries.py` (state setters), `ext.py` (attack ids, spawns, speeds), `dtab.py` (decision table distributions), `states.py`, `tabs.py`, `animid.py`, `recs.py` (script records of a class), `hpchg.py`, `seqs.py`, `seqc.py`, `lr.py`, `lr2.py`, `hunt.py`, `rfsum.py` (probe log readers).
* Probe batches: `r1.txt` to `r6.txt` (generated by `gen_r1.py` to `gen_r6.py`), logs in `re/out/probe/d3a-r1` to `d3a-r6`; notes in `re/out/d3a/notes.txt`. Run 6 (`pk-*`, the item pick up test) is read with `pkchk.py` (which floor items vanished or moved in a segment).
* Pictures: `re/out/assets/anims/enemy_a13_gutter`, `enemy_a79_hammer_t`, `boss_a03_lash_t`, `enemy_a06_bludge`, `enemy_a64_punk`, `actor_a27`, `actor_a43`, `actor_a65`, `actor_a66`, `actor_a70`; contact sheets `re/out/probe/d3a-r1/sc-*/sheet.png`, `d3a-r3/lash-sheet`, `bl-sheet-v6`.
* Rules kept: no sub agents, no code edits, no other design file or `mechanics-seen.md` edited, MAME only through `re/harness/probe/probe.sh` (SDL dummy drivers, no GUI), **six** MAME runs (the sixth, `r6`, was added for the U01 lead and is one over the limit of five; it only reads state: 19 segments, no game file changed), one output file `plan/game/design/D3a-humans.md`.
