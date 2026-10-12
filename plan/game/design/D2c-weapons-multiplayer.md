# D2c: weapons and multiplayer (all 15 weapon types, friendly fire, tag team, what 2 or 3 players change)

Written 2026-10-12 by the D2c research agent. Part of the design phase (`plan/game/NEW-PLAN.md` section 1, D2). Documents only: no code was changed.
Everything here is meant to be built exactly by later port agents.

How to read this file

- **Tags.** `R:` = ROM address or repo file (68000 addresses; `G.$xxxx` = RAM `0xFF8000 + xxxx`; `+$xx` = field of an object). `M:` = measured in MAME with the probe
  tool (the run and segment names are given; the logs are in `re/out/probe/<run>/<segment>/log.txt`, git-ignored). `W:` = web (links at the end of section 22). `U:` = the user.
  MAME wins over the web, the ROM wins over the old repo specs.
- **Runs.** Four headless MAME runs (all with the probe, `SDL_VIDEODRIVER=dummy`): `d2c-smoke` (10 segments, a check of the scripts), `d2c-big` (290 segments: weapons, friendly
  fire, tag team, contact, counts), `d2c-f` (154 segments: fixes and new questions), `d2c-g` (25 segments: debris, barrel, 3 player HUD). The generators and the input files are
  in `re/out/d2c/` (`gen_w.py`, `gen_m.py`, `gen_f.py`, `gen_g.py`, `batchW_big.txt`, `batchM_big.txt`, `batchF.txt`, `batchG.txt`; helper scripts `an.py`, `hits.py`, `ffsum.py`).
  All weapon tests use Episode 1 area 0 at camera X 928, where no script enemy is near, a pinned level 0 thug (class 2, HP 44, set to 100 before each hit) and `god on`.
  In the solo weapon runs the hero's attack power factor `$C6` was 42 (read from `+$C6` in every log; D2a section 14), so the damage numbers below are "record * 42/32 * enemy factors" (in some two player runs a hero had another `$C6`, noted where it matters).
- **Hero ids**: 0 Jack, 1 Mustapha, 2 Hannah, 3 Mess (D2a). Tables are in this id order.
- **Units.** Speeds in 8.8 fixed point (divide by 256 for pixels per frame). Time in game frames (51 per second). A frame `t` of a probe log is counted after the jump helper; an
  input pressed at `t` shows in the log at `t + 1`.
- **Weapon vocabulary.** "Type" is the weapon number `+$26 / 2` (0 to 14), "var" is `+$26` (0, 2, 4 ... 0x1C), "kind" is the word `+$A0` (0x201 to 0x20A) that the hero
  copies into `+$B6`. The port module is `dino-rs/crates/dino-game/src/game/item/weapon.rs` (the item side) and `player_weapon.rs` (the hero side).

## 0. Summary of what is new

1. **The 15 weapons are settled with their HUD names** (table 0x23D0B, section 1): 0 RIFLE, 1 GUN (revolver), 2 KNIFE, 3 UZI, 4 M-16A1, 5 SHOT GUN, 6 BAZOOKA, 7 ROD, 8 STICK
   (the rod after it broke), 9 B. KNIFE (butcher knife), 10 CLUB, 11 TORCH, 12 DYNAMITE, 13 GRENADE, 14 STONE (three sizes). The code comments of the port that call types 12, 13, 14
   "time bomb, fire bomb, explosive stick" are old names. The HUD shows a picture, the ammo number and a cartridge bar only for the six guns; the other nine show a generic
   "ARM" tile and the name.
2. **Charge swing and critical hit (section 5.5).** Every melee weapon of kind 0x203 (rod, butcher knife, club, torch, empty rifle, empty M-16) can be charged: the hero holds the
   attack button, a counter runs to 60, and the damage is multiplied by 1.0 (counter 15 or less), 1.25 (16 to 30), 1.5 (31 to 45), 2.0 (46 and more). The club has two hits:
   Jack and Hannah hit first with record 0x58 (10) and then 0x60 (30); Mustapha and Mess have only the strong hit 0x60. A full charge club kills most things in one swing
   (Jack: 28 + 72 on a thug, Mustapha 82). M: d2c-big club_hold0 to club_hold60.
3. **Friendly fire has two separate passes** (section 14). The fist pass 0x10664 (gate = enemies alive, windows -1..1, 0..1, 0) was measured with 3 players for n = 0 to 4
   enemies and agrees with D2a exactly. The **weapon pass 0x10DCC has no enemy gate**: bullets, pellets, swings, thrown weapons, explosions of a player hurt every other
   player for 1 HP (never below 1) when the attack lies at **exactly the same depth Z**, always, also with 4 enemies on the screen. M: d2c-big ffi_*. A thrown barrel flies 4 px nearer
   (Z - 4) so it missed a friend at the thrower's depth. The pass is a stub in the port.
4. **Tag team (section 15) rules are complete**: the trigger code is decoded to the byte; the counter is paused by the initiator's own invulnerability and reset by the partner's;
   with three players the lower player number is checked first; weapons in hand do not matter; the partner of Jack's "Let's Go" attacks with record 0x1A when a direction is held
   and hits a thug. M: d2c-big tie_*, tie3_*, tierun_*; d2c-f tiez_*, tierun2_*, pick_*.
5. **The four blocked checks of D2a are done** (section 18): 3 player friendly fire (done), dive window per hero (22-24, 21-25, 22-25, 22-23 frames after the jump press), Jack's partner
   run attack (record 0x1A), triangle kick after the wall jump (starts 14 frames after the wall jump; Hannah low attack 0x0C, Mustapha dive 0x0E).
6. **Players walk through each other** (no contact, no push) in all tests (section 17). Draw order is depth Z, then object address (P1 before P2 before P3, items before players before enemies).
7. **New facts about weapons not in any spec**: the bazooka shell does only half damage to Slice, Vice, Butcher, Morgan, Morgue, Slisaur, Tyrog 0x19 and Fessenden 0x22 (table 0xA32FA, id 0x52,
   factor 16/32), shotgun pellets do 0.75 to Vice and Walther; Tyrog (class 4) has half the HP of the table row 6 (256 / 288 / 352, M: d2c-big boss_tyrog_*); the dropped dynamite lights itself on landing
   and explodes 60 frames later (hurting everyone but its owner); a dropped grenade does nothing; the empty last-shot gun is let go at the end of the animation, an empty M-16 is kept as a club.
8. **The player count `G.$4EF` is read in 26 places** (section 13), recomputed by 0x655E when a player dies for good (61 frames after the death sequence starts) and from the scripted mask during scripted control.

## 1. The 15 weapon types

R: init table 0x24D6C and `weapon_init` (0x24D58), HUD names 0x23D0B, item art `re/out/assets/anims/item_e00/` (16 animation tables: type 14 has three), refill table 0x2855A. M: weapon art checked in `re/out/d2c/weapon_lying.png`
(a picture made from the dump; not committed).

| type | var | HUD name | what it is | kind `+$A0` | rounds `+$6C` | held and used as | attack ids (swing / shot / throw) | empty or broken |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0x00 | RIFLE | long brown hunting rifle | 0x205 | 6 | fires in state 0x16 | bullet 0x50 | last round: becomes kind 0x203 (club), the hero lets go; the empty rifle can be picked up again and used as a club (butt 0x32) |
| 1 | 0x02 | GUN | revolver | 0x204 | 6 | fires in state 0x16 | bullet 0x4E, thrown (empty) 0x1E | last round: kind 0x201, the hero lets go; the empty gun can be picked up and thrown once |
| 2 | 0x04 | KNIFE | blue blade | 0x202 | (6, not used) | swings (0x16) when an enemy is in reach, else throws | swing 0x2C, thrown 0x24 | a thrown knife is used up (it vanishes after its flight) |
| 3 | 0x06 | UZI | small grey submachine gun | 0x206 | 48 | automatic fire, state 0x1C (standing) or 0x1E (walking) | bullets 0x46 and 0x48, thrown (empty) 0x20 | last round: kind 0x201, the hero lets go |
| 4 | 0x08 | M-16A1 | black assault rifle | 0x207 | 48 | automatic fire, states 0x1C / 0x1E | bullets 0x4A and 0x4C, butt 0x34 | last round: kind 0x203, **the hero keeps it as a club** |
| 5 | 0x0A | SHOT GUN | pump shotgun | 0x208 | 6 | fires in state 0x16, 6 pellets | pellets 0x5E, thrown (empty) 0x22 | last round: kind 0x201, the hero lets go |
| 6 | 0x0C | BAZOOKA | rocket tube | 0x209 | 4 | fires in state 0x22 (recoil) | shell 0x52, blast 0x54 | empty: the hero lets go; an empty bazooka that lands breaks (blinks, gone) |
| 7 | 0x0E | ROD | long wooden bar | 0x203 | (10, not used) | charge swing, state 0x20 | swing 0x30 | after a hit it may break into type 8 |
| 8 | 0x10 | STICK | short broken rod | 0x202 | (6, not used) | like the knife (swing or throw) | swing 0x2E, thrown 0x2A | thrown: used up |
| 9 | 0x12 | B. KNIFE | long black knife with red | 0x203 | (6, not used) | charge swing | swing 0x36 | never expires on the floor, re-initialises instead of being freed |
| 10 | 0x14 | CLUB | wooden bat | 0x203 | (10, not used) | charge swing, two hits | 0x58 (weak) and 0x60 (strong) | wear only |
| 11 | 0x16 | TORCH | the same bat with a flame | 0x203 | (10, not used) | charge swing, flame overlay (class 6) | swing 0x38 (burn) | wear only |
| 12 | 0x18 | DYNAMITE | bundle of red sticks with a fuse | 0x201 | (10, not used) | thrown | thrown 0x28, blast 0x56 | lights itself when it comes to rest |
| 13 | 0x1A | GRENADE | grey grenade | 0x201 | (10, not used) | thrown | thrown 0x26, chain of 3 explosions 0x54 or 0x56 | explodes on a hit or the 3rd floor contact |
| 14 | 0x1C | STONE | rock, 3 sizes by `+$5C` = 0, 1, 2 | 0x201 | (10, not used) | thrown | thrown 0x1C | vanishes after 4 bounces |

Notes. The rounds of the melee and thrown kinds are set at init but never changed: only **wear** (section 8) can end them. The 15 HUD names are in this order in the ROM table 0x23D0B (BAZOOKA, RIFLE, GUN, SHOT GUN,
M-16A1, UZI, KNIFE, ROD, STICK, B. KNIFE, CLUB, TORCH, DYNAMITE, GRENADE, STONE; 13 characters each). Type 12 and 13 come in two forms: the one a player throws and the one an enemy throws (section 6.2).
The weapon item is pool E class 0 (handler 0x24D44), the same as the bullets of its gun (class 3, 7, 10) and the explosions (class 1) are other pool E classes.
Source of the item art tables: `ANIMS` in `item/weapon.rs` (0x265F4 rifle, 0x268BC gun, 0x26A3C knife, 0x26BBC Uzi, 0x26D3C M-16, 0x26F7E shotgun, 0x270FE bazooka, 0x2727E rod, 0x273FE stick, 0x2757E butcher knife,
0x276FE club and torch, 0x2787E dynamite, 0x27B12 grenade, 0x27CD2, 0x27D1A, 0x27D62 stones).

## 2. Picking up, holding, dropping

### 2.1 Picking up (R: `find_pickup` 0x5AA2, entry 0x19832, handler 0x198B4; port `player_weapon.rs`)

- There is **no automatic pick up**. The hero stands on the item (his X and Z inside the item's first hit box, his height within 8 px of the item's height, not in the air, `G.$506 = 0`) and presses attack.
- Items that can be taken are pool E objects on the floor (mode 2, sub 0): weapons, ammo boxes, food, treasure, liftable containers. The **lowest kind word wins** (0x00tt food, 0x0100 ammo,
  0x0201 .. 0x0209 weapons, 0x020A heavy objects, 0x03tt treasure), equal kinds: the newest item of the list. M: d2c-big pick_lowest (knife 0x202, pistol 0x204, rifle 0x205, Uzi 0x206 on
  one spot: first press took the knife, the second the pistol while the knife hopped away, the third the knife again as it had landed: the two ping-pong, the rifle and Uzi stay).
- Heavy objects (0x20A) need the stick pushed towards the object (an X or Z direction towards it), except variants 6 and 8 (stone, boulder).
- **40 frame lock**: for 40 frames after a pick up (`+$F8`) weapons are not taken again; food and treasure are. M: pick_lockout (the press at +20 after the pick up attacked instead: the knife was
  thrown, the next pick up came at +70).
- The pick up state is 0x14: animation 0:0xA of 11 frames (heavy objects 0:0x16, 21 frames), no hurt box, voice 0x9C20. The item becomes held (sub 8) on the frame byte 1 of the animation
  (frame 9 or 10 after the press). When it ends the hero is invulnerable for `INV[8 + hero]` = 10 / 8 / 20 / 10 frames (D2a section 15). M: all weapon segments (press at t = 10, held at t = 20, idle at t = 21).
- A new weapon replaces the held one: the old item is dropped (sub 0xC, drop flag `+$C5` = 0).
- **Two players at once.** Players are updated in the order P1, P2, P3. The first one that presses takes the item (it leaves sub 0 at once); the others' presses become normal attacks (a jab).
  M: d2c-f pick_same2, pick_next2b (P2 pressed one frame earlier than P1: P2 got the gun and P1 jabbed), pick_same3 (P1 got it; P2 and P3 jabbed, P3's jab hurt P2).

### 2.2 What makes a hero drop the weapon

| event | held weapon | drop flag `+$C5` (wear check later) | source |
| --- | --- | --- | --- |
| hit by anything (enemy, friend, thrown item, explosion), also knockdown and burn: routine 0x1177A | Jack and Mustapha: always dropped. Hannah: **keeps** kinds 0x204 to 0x208 (pistol, rifle, Uzi, M-16, shotgun), drops knife, stick, rod, club, torch, butcher knife, bazooka, dynamite, grenade, stones, heavy objects. Mess: keeps **only** the bazooka 0x209, drops everything else | 1 | R: 0x1177A; M: d2c-big ffd_* (friend jab: Hannah kept the pistol, dropped the bazooka and the knife; Mustapha dropped the pistol; Mess kept the bazooka, dropped the pistol) |
| dies | dropped | 1 | M: d2c-f die_with_weapon |
| special attack (attack + jump) | discarded at the start (`+$B4`, `+$B6`, `+$C5` cleared), the item hops and lies | 0 | M: d2c-f special_discard_club, _pistol, _knife |
| picks up another weapon | swapped (the old one hops where he stands) | 0 | M: pick_lowest |
| last round of a gun | let go at the end of the shot animation (the Uzi, pistol, rifle, shotgun, bazooka; not the M-16, which stays as a club) | 0 | M: pistol_*, uzi_*, rifle_*, m16_* |
| held by an enemy | `+$B6` cleared, the item drops by itself (owner gone) | n/a | R: SPEC 4.9 |
| scripted control starts while carrying a barrel | dropped | n/a | R: 0x191C4 (D4c T09) |

A dropped weapon (sub 0xC): hops with vy 0x300, ay -0x30 (rises about 22 pixels, 47 frames in the air), lands, then **lies for 3600 frames** (`+$A4` = 0xE10, counts down, then the item is freed without blinking).
M: d2c-big drop_swap_life (landed t = 121, still there at t = 3684, gone before 3720). Weapons that are **placed by the script or dropped by an enemy never expire** (`+$A4` = 0): M: drop_enemy_life (an Uzi dropped by a thug was still on
the floor 3700 frames later). A dropped weapon is picked up again exactly like a new one. Items are freed when far off the screen (0x173C and 0x1760).

### 2.3 Hero differences while holding a weapon

| | Jack | Mustapha | Hannah | Mess |
| --- | --- | --- | --- | --- |
| keeps when hit | nothing | nothing | the 5 guns | the bazooka |
| "enemy in reach" distance for knife and stick (swing instead of throw), table 0x62AE | 112 px in front, depth +-16 | 128, +-24 | 120, +-16 | 104, +-16 |
| hand height of a thrown weapon (table 0x265B4) | 53 | 50 | 57 | 58 (50 for the second row) |
| bazooka recoil push `RECOIL[id]` (0x19CD8) | 2048 (8 px per frame), moved 28 px | 2560 (10), 45 px | 3072 (12), 66 px | 1536 (6), 15 px |
| knife / stick swing first hit (frames after the press) | +6 | +6 | +2 (a 5 frame stab) | +7 |
| knife swing record 0x2C | 20 / stun 6 / knock down | same | **15 / stun 4 / flinch only** | same as Jack |
| stick swing record 0x2E | 20 / 11 / knock down | same | **20 / 4 / flinch only** (hit stop 5) | same |
| club hits | 0x58 at +9 then 0x60 at +14 | 0x60 only at +13 | 0x58 at +10 then 0x60 at +14 | 0x60 only at +13 |
| heavy object | jumps | jumps | **cannot jump** | jumps |

(R: tables; M: d2c-big knife_swing_*, stick_*, club_hold0_*, bazooka_*, reach_* in d2c-f. The old SPEC lists the recoil values with Hannah and Mustapha swapped; the ids above are right, D2a 1.)

## 3. Using a weapon: which state, which frame

The attack button with a weapon in the hand starts a weapon state by the kind (`weapon_attack_entry` 0x198DC, R: SPEC 4.9.1). All set `+$C4 = 1` ("attack flag", read by the item) and the held pose of set 5 of the hero's animations.

| kind | state | what the item does (read from the hero's flag `+$C4` and frame byte `+$2C`) |
| --- | --- | --- |
| 0x204 pistol, 0x205 rifle, 0x208 shotgun | 0x16 | fires on the **first frame** of the state (`+$C4` still set): one round, `+$C4 = 0`; the animation lasts 12 frames (Hannah 11) |
| 0x206 Uzi, 0x207 M-16 | 0x1C standing, 0x1E walking (when a direction is held; the legs layer walks) | one round on every rising edge of the frame byte: **one round every 7 frames** while attack is held (7.3 per second); `+$81` keeps the state 3 frames after the release; jump leaves the state |
| 0x209 bazooka | 0x22 | fires at once, the hero is pushed back (section 2.3), about 18 frames |
| 0x203 club, rod, torch, butcher knife, empty rifle, empty M-16 | 0x20 (charge) | counter `+$80` counts up while attack is held (cap 60); the swing frames play after the release (section 5.5) |
| 0x201, 0x202 and heavy object 0x20A | 0x16 | the item swings (0x202 with an enemy in reach) or throws on frame byte 1 or 2; a held heavy object is thrown on frame byte 1 |
| running (state 0x200) and attack | with 0x201 or 0x202: run swing state 0x202 (item gets `+$A8 = 1`, a flat fast throw); with any other kind (club, gun, rod, torch, butcher knife): the **normal dash attack** and the weapon stays in the hand | M: d2c-f runswing_* |
| jumping and attack | normal jump attacks (low 0x0C, dive 0x0E); the weapon is not used in the air and stays | M: d2c-f jumpweapon_* |

Timing, M (press at t = 40, Jack unless noted): pistol, rifle, shotgun, bazooka shot at t = 41; Uzi and M-16 rounds at 41, 48, 55, ...; thrown knife, stones, dynamite, grenade leave at t = 41 (frame byte 2 at +1);
a thrown barrel at +7 (Mess +8). Pick up held at +10.

## 4. Guns

R: bullet handlers `item/blast.rs` (class 3 bullet 0x2860A, 7 shell 0x29D1E, 10 pellet 0x2A39A, 1 explosion 0x27E16), records 0x28814 (bullets), 0x2A484 (pellets), tables below.

| gun | bullet object | speed (8.8) | attack id | what the hit does (record: damage / stun / reaction / hit stop / score) | muzzle offset in front, up |
| --- | --- | --- | --- | --- | --- |
| pistol | class 3 var 0 | 2048 (8 px per frame) | 0x4E | 20 / 4 / 0x14 (push back) / 5 / 400 | 20, 6 |
| rifle | class 3 var 2 | 2048 | 0x50 | 30 / 11 / 0x06 (heavy knock down) / 5 / 400 | 40, 5 |
| Uzi | class 3 var 4 and 6 (var 6 when `+$6C & 2`) | 3072 (12 px) | 0x46 / 0x48 | 10 / 2 / 0x00 (flinch) or 0x14 (push back) / 1 / 100 | 29, 5 |
| M-16 | class 3 var 8 and 0xA (0xA when `+$6C & 1`) | 3072 | 0x4A / 0x4C | 15 / 2 / 0x00 or 0x14 / 1 / 100 | 54, 7 |
| shotgun | 6 x class 10 var 0..5 | vx 2688, 2816, 2944, 3200, 3328, 3072 (10.5 to 13 px) and vy 0, +128, -128, +256, 0, -256 | 0x5E each | 15 / 3 / 0x02 / 1 / 100 per pellet | 46, 7 |
| bazooka | class 7 shell, then class 1 kind 1 | vx +-0x100 with ax +-0x40 per frame (it accelerates: 1 px per frame at the start, about 6 px after 20 frames), an arc around its start height (vy 0x100, ay 0xC0, flips at the start height), dust trail every 2nd frame | shell 0x52, blast 0x54 | shell 40 / 11 / 0x0C / 5 / 400 (measured 55 to 56 on a thug); blast 10 / 11 / 0x0C / 5 / 400 | 34, 7 |

(Muzzle offsets are `-dx` of the item, i.e. in front of the weapon picture; R: `weapon_held_*` in `item/weapon.rs`.)

- **Rounds and wear.** Every round: `+$6C - 1` (the HUD number), wear `+$A6` + 0x20 (a byte: it wraps after 8 rounds for the 48 round guns, so the wear index runs 0 to 7 again and again).
  Pistol 6, rifle 6, shotgun 6, bazooka 4, Uzi 48, M-16 48. M: pistol_jack (6 to 5 at t = 41), uzi_jack, m16_jack (48 to 39 over the 9 rounds of the first burst).
- **Auto aim (R: 0x64F6, `player_aim_nearest`).** At the start of a gun state the hero picks the nearest enemy in front (standing or walking, the last frame's list 0x69BA, depth distance at most 0x100;
  equal distance: the later entry). It only steers the **vertical** speed of the Uzi and M-16 bullets (aim = -1 records): `vy` = a table value by the horizontal distance minus 0xA0 (steps of 0x10 px) plus 0x30 per round of the burst
  (up to 0x18 rounds). M: uzi_jack: vy -336, -288, -240, ... 0 for the first 8 rounds of a burst (the burst "climbs"); after the 8th round vy stays 0; at the 4th round of a burst the depth speed `vz` starts to alternate (R: `AIM_VZ`).
  The pistol, rifle, shotgun and bazooka bullets are not aimed: a fixed vertical speed from a table, and they **fly straight in depth**: a pistol bullet hit a thug at 100 px that was 12 px away in depth and missed one 16 px away. M: d2c-f aim_pistol_dz*.
- **Point blank.** The pellets start 46 px in front of the hero, so at 30 px only 3 of 6 pellets touch the enemy (3 x 20, 20, 18); from 60 px on 4 of 6 hit (20, 20, 18, 17 = 75). M: d2c-f shotgun_d30 to shotgun_d200.
- **Damage on a level 0 thug at full HP (M, C6 42):** pistol 27 to 28, rifle 41, Uzi 14 per round, M-16 20 per round, shotgun 75 for 4 pellets, bazooka 55 to 56 (shell) and the blast.
  A **rifle round knocks the enemy down** (reaction 6), pistol and Uzi push back, the M-16 flinches or pushes.
- **Bullets hit a lying enemy never** (frame class 8 and 0xA are immune to attacks, `0x10FF2`). Bullets die on a hit (`+$59` hit stop), on the floor, a wall or the water (3 hit animations).
- **Last round.** Pistol, rifle, Uzi, shotgun, bazooka: see the table of section 1. The empty gun lying on the floor can be picked up and used: pistol 0x201, Uzi 0x201, shotgun 0x201 are **thrown** with the next attack press
  (records 0x1E, 0x20, 0x22: 20 damage, knock down, 1 hit; flight 9.5 px per frame flat, hits only before its first bounce, then bounces back at 2 px per frame, 4 bounces, then blinks 30 frames and is gone);
  an empty rifle or M-16 (kind 0x203) is a club with the butt records 0x32 (25 damage, reaction 0x10, hit stop 15) and 0x34 (25, 0x12, 15). M: pistol_jack (thrown pistol hit for 28 at t = 226), uzi_jack (thrown Uzi 28), rifle_jack and m16_jack (butt 35).
- **Walking fire.** With a direction held the hero fires in state 0x1E (the legs walk, a second animation layer, D2a / R: `player_layer_*`). The first walking fire after a pick up reads the ROM vector table for the layer offset (kept in the port).
- Sounds (R): rifle 0x9C76, pistol 0x9CD8, Uzi and M-16 0x9D9C, shotgun 0xA442, bazooka 0x9E08, throw 0x9D44; muzzle flash effect class 0x21 (0x96B40) with the kind in `+$60`, spark classes 0x1F, puff 6 / 9 / 10 (D3f); explosion sound 0x66 (0x9F12).

### 4.1 The bazooka in detail

Shell (class 7): created `34` px in front of the item; on the first frame it takes the item's height as its "floor"; every 2nd frame (`G.$0` even) it leaves a dust puff (effect 0x92E5E); it ends on the floor, a wall or water or on any hit (`+$59`):
it frees itself and makes the explosion (class 1 kind 1: a rolling explosion of 0x54, 26 frames). The record 0x52 (40 damage) is applied by the shell itself, the 0x54 (10 damage) by the explosion; on a body that is already in a hurt reaction
the explosion does not hit again. M: bazooka_jack: thug 100 to 45 in one frame at t = 64 (shell 55); recoil moved Jack from x 1024 to 996 in 7 frames.
**Against bosses the shell is halved** (section 7.3). A bazooka dropped with 0 rounds breaks at once (blinks 30 frames). The bazooka a joining or continuing player gets has 4 rockets and a 600 frame life when dropped (R: 0x65A4, D2b 2.6).

## 5. Melee weapons

### 5.1 Knife (type 2) and stick (type 8), kind 0x202

State 0x16. The press checks `item_enemy_near` (0x6248): an enemy in the "reach" zone (table in 2.3) in front, within the depth band: the weapon **swings** (frame byte 1), else the hero takes the throw pose and the weapon is
**thrown** (frame byte 2). Measured limit (M: d2c-f reach_*): Jack swung with the thug at 96 and 104 px (hit at +6), at 112 px the zone says "near" but the blade does not reach (no hit), from 120 px on the knife was thrown (hit at +10 to +12, the flight takes
longer the further the enemy is); Mustapha 128, Hannah 120, Mess 104 the same way. The swing box reaches about 104 px for Jack, Mustapha and Mess; Hannah's stab is shorter (no hit at 96 px).
Swing: 0x2C (knife) or 0x2E (stick); throw: 0x24 or 0x2A (30 damage, knock down, spark 0x15), flat flight vx 9.5 px per frame (the first row of table 0x26534; with a run swing 12 px per frame), 8 frames without floor test,
a **hit destroys** the thrown knife or stick (mode 6), without a hit it bounces up to 4 times (wall: vx back 2 px per frame) and blinks 30 frames. Wear per swing 0x20.
The held knife **flashes** (palette 0x84 for 5 frames after every check that finds an enemy in reach; the check runs every 5 frames and its own frame shows palette 0x8A): the web page says it is "more effective whenever the knife flashes red"; no damage difference was found,
the flash is only the sign that an enemy is in reach. Hannah's stab: her swing animation is 5 frames, so she can stab on every press (12 frame taps gave a hit each time, D2a section 12).
M: d2c-big knife_swing_*: first hit +6 / +6 / +2 / +7, damage 28 (27), 27 (28), 20, 27 (28) on a thug at HP 100. Stick: the same timing, damage 27 to 28 for all (Hannah's stick flinches).

### 5.2 Rod (type 7), kind 0x203

Charge swing, record 0x30 (25 damage, stun 11, reaction 4 knock down, hit stop 10, score 400). Measured 35 on a thug. Wear 0x20 per swing. **A hit may break the rod into the stick (type 8)**: every hit counts (`+$AD`);
the chance is `HIT_CHANCE[count] / 16` with the table 0x2CBA0 = 1, 3, 6, 11, 14, 13, 1, 2, 15, 15, ... indexed by the number of hits including this one: **19 percent after the 1st hit, 38 after the 2nd, 69 after the 3rd, 88 after the 4th, 81 after the 5th, 6 after the 6th, 12 after the 7th, 94 from the 8th on** (a random roll `rand & 15 < table value`). On a break: a debris effect (class 0x2C) at 24 px in front,
sound 0x6F, the item changes to type 8 (var 0x10, kind 0x202, wear 0), attack id 0x2E, and the hero keeps it. M: d2c-big rod0_jack (broke after the 1st hit, t = 49), rod1_jack (2nd), rod2_jack (4th), rod0_hannah (3rd), rod1_hannah (3rd).
After the break it is a stick: the knife rules of 5.1 (swing 28, flinch only for Hannah).

### 5.3 Butcher knife (type 9), kind 0x203

Charge swing, frame byte 1, record 0x36 (25 damage, stun 11, reaction 4, hit stop 15, spark 0x15, score 400): measured 34 to 35 on a thug, first hit +8 (Hannah +9). **It never expires on the floor and is not freed**:
the class re-initialises when it leaves the screen (var 0x12 is excluded from the 3600 frame life and from the despawn: R: 0x264D0, 0x2463C). It is the weapon of the Butcher boss: pool A class 52 makes two weapon items (var 0x12) that
the Butcher holds (`+$B4` and `+$B6` of the boss, R: 0x57386..0x573C6, D3c). A butcher knife dropped by a hit does not get a break check (it is excluded in 0x247FA).

### 5.4 Torch (type 11), kind 0x203

Charge swing, **frame byte 2** (the hit is at +12 after the press for all heroes), record 0x38: 25 damage, stun 11, **reaction 0x0A (burn knock down)**, hit stop 10 (Hannah 8), sound 0x12, score 400. Measured 34 to 35 on a thug; the thug then plays
the burn sequence 0x0A00, 0x0A02, 0x0A04, 0x0A06 and gets up after about 130 frames; **no further damage over time** was seen (HP stayed 65 for 350 frames). The burn reaction is the shared flame knock down (flame objects, effect class 0x20, 0x92FB4).
The torch has a **glow overlay**: pool E class 6 (handler 0x29B06, **not ported**): created together with the torch (also on the floor), it follows the item with the offset table 0x29C26 by the hero's animation frame (`+$2D`), the same facing,
depth alternating between Z + 2 and Z - 1 every frame (flicker), and plays the 6 frame flame loop (0x29CB6, 4 ticks per frame) while the hero stands or walks, or the 4 frame flame streak (0x29CF4, 16 ticks) during the swing; it frees itself when the item is gone.
M: d2c-big torch_jack: overlay E#22 present from t = 4, Z toggling 295 / 298. Torches in the game: Episode 5 area 0 (3 on the floor at the start, X 0xDD, 0xF4, 0x108) and area 1 (2 at X 0xF70 and 0xFA0), nowhere else (R: stage scripts).
Torches do not set fire to dinosaurs in the weapon code: the hostile model (D3b) does the rest.

### 5.5 Club (type 10) and the charge swing

Player state 0x20 (R: 0x19C36, 0x19C62; port `player_charge`). At the press: `+$80 = 0` (the counter), then every frame with attack held `+$80 + 1` up to 60; the swing animation waits on its first frame while the button is held and the hero can
turn (the stick turns him). On release: if the counter is 8 or more the first frame is skipped at once (`+$2F = 1`); if it is 45 or more one more animation step is skipped. The item then makes its hits on the frame bytes.
**Damage scale** (R: 0x11262 in pass 0x11008, port `enemy_charge_scale`): applied to the **integer record damage before** the other factors, by the counter value at the moment of the hit:

| counter `+$80` | 0 to 15 | 16 to 30 | 31 to 45 | 46 to 60 |
| --- | --- | --- | --- | --- |
| factor | x1.0 | x40/32 | x48/32 | x64/32 (double) |
| club tap 0x58 / 0x60 base 10 / 30 becomes | 10 / 30 | 12 / 37 | 15 / 45 | 20 / 60 |

It works for **every attack of an item with kind 0x203** (rod, butcher knife, club, torch, empty rifle, empty M-16). Measured on a level 0 thug with 100 HP, Jack (C6 42), hold time and result (M: d2c-big club_hold*_jack):

| hold (frames) | counter | first hit 0x58 | second hit 0x60 | first hit frame after the press |
| --- | --- | --- | --- | --- |
| 0 (tap) | 1 | 14 | 42 | +9, +14 |
| 10 | 11 | 14 | 42 | +15, +20 |
| 20 | 21 | 16 | 52 | +25, +30 |
| 30 | 31 | 20 | 64 | +35, +40 |
| 40 | 41 | 20 | 64 | +45, +50 |
| 50 | 51 | 28 | 72 (cut by the HP left) | +53, +58 |
| 60 | 60 | 28 | 72 (cut by the HP left) | +63, +68 |

(Full charge second hit: 60 * 42/32 * 1.0625 = 83 if the HP is there.) Mustapha and Mess have only the strong hit: tap 41, charged 82. Hannah like Jack (13 + 41, charged 28 + 72). The charged swing "delays the swing" like the web page says (W: https://www.arcadequartermaster.com/cad_items.html). The same page says for the torch that "delaying the swing does not change its damage": in the code every attack of an item of kind 0x203 is scaled,
the torch included, so that statement may be wrong; it was not measured for the torch (open item 5). The "critical hit kill" of the web page is this double damage of a fully held swing.

## 6. Thrown weapons and explosives (dynamite, grenade, stones)

### 6.1 Throw

R: `weapon_throw` (0x264FC and the throw tables 0x26534), item states 0x18, 0x1C, 0x20. On the frame byte 2 of the throw animation (+1 after the press): the item moves to the hand, the thrower's throw sound 0x9D44, velocity from the table by type
(second row when the throw came from a run: `+$A8` set), 8 frames without a floor test (`+$B2`), new attack serial.

| type | normal throw (vx, ax, vy, ay in 8.8) | thrown from a run |
| --- | --- | --- |
| pistol, Uzi, shotgun (empty), knife, stick, stones | 2432 (9.5 px per frame), ax -2 (almost flat), no lift | 3072 (12 px per frame) |
| dynamite, grenade | 1024 (4 px), ax -8, vy 640 (2.5 px up), ay -56 (a lob) | dynamite 1280, vy 384, ay -64; grenade 3072 flat |

### 6.2 Dynamite (type 12)

State 0x18 in flight: a hit on an enemy (`+$59`) makes it jump up (vy 0x500, back 2 px per frame) and counts as a bounce; walls turn it around; **4 bounces** on the floor and it lies still: it is **armed** (`+$B0 = 1`) and the picture changes
every 15 frames (`+$B4`, stage `+$B2` 3, 2, 1, 0: four pictures), and **60 frames after it came to rest it explodes** (big explosion class 1 kind 0 on the floor: attack 0x56). The web says "about 3 seconds": from the throw to the blast it was 114 frames (Jack) to 173 frames (Hannah, Mess), 2.2 to 3.4 seconds:
the flight and the bounces take 54 to 113 frames (a hit on an enemy makes it jump and bounce more), the fuse after the rest is always 60 frames (1.2 seconds). M: d2c-big dynamite_jack (thrown t = 41, at rest t = 95, exploded t = 155), dynamite_mustapha (116, 176), dynamite_hannah (153, 213), dynamite_mess (154, 214).
Direct hit in flight: record 0x28: 20 damage, stun 1, knock down (measured 28). Explosion on the floor: record 0x56: 40 damage, reaction 0x0C, hit stop 5, score 200, screen shake 3 (measured 52 to 56 on a thug; spark 4 at +0x20 height).
It can hurt every enemy near, also those behind it in depth by the box. **Dropped by a hero** (hit or death) it lands armed and explodes 60 frames later: the dropper is not hurt, a friend 34 px away got 1 HP and a knock down. M: d2c-f dropped_dynamite.
A dynamite that an **enemy** throws is created already armed with the owner 0xFF (blast kind 3, attack 0x6A, damage taken from the record without factors, can kill; D3 owns the throwers; port side: item code DONE, the hit on players is the stub of section 14).

### 6.3 Grenade (type 13)

State 0x1C in flight: it explodes **on any hit** or on the **3rd floor contact** (the first two bounce with a random depth speed from table 0x265D4, half speed and a quarter of the lift). The explosion makes a chain of **three explosions 10 frames apart**
(state 0x28, `+$80` 0xA, `+$81` = 2 down to 0): at the impact point, then (-16 px, +16 depth), then (+32 px, 0) (table 0x26408). A hit in the air gives the weak explosion (attack 0x54: 10 damage, no shake);
on the floor the big one (0x56: 40 damage, shake 3). The direct hit record 0x26: **60 damage**, reaction 0x0C (measured 82 to 85 on a thug in one frame). M: d2c-big grenade_jack, grenade_mess (thrown t = 41, hit t = 84, chain at 85, 95, 105).
A grenade dropped by a hero does nothing (it is not armed: only var 0x18 is). A grenade is freed when its chain ends (105).

### 6.4 Stones (type 14)

Three sizes (picture tables 0x27CD2, 0x27D1A, 0x27D62 by `+$5C` 0, 1, 2), the same record 0x1C: 20 damage, stun 1, reaction 6 (measured 28 or 27 for all three), flight like the knife (9.5 px flat), 2 bounces then the state 0x24 (blink 30 frames, gone). M: d2c-big stones_jack,
stones1_jack, stones2_jack (thrown t = 41, hit t = 56, gone t = 128). G.$4F1 counts the stones on the field. Sources: the script (Episode 4 area 0 at X 0x14B, Episode 7 area 0 at X 0x720) and **the debris of stone and boulder containers**
(section 11: the debris code can turn up to 3 of its 9 pieces into stones). The stone container drop was **not seen** in 20 breaks in MAME (open item 1).

### 6.5 Every swung or thrown item is an attacker

Every swung or thrown item is an "attacker": while its frame class is 4 it is in the list 0x671A and hurts enemies (pass 0x10F96), containers (0x120D2) and (section 14) the other players.

## 7. Damage

### 7.1 The attack records of weapons (R: table 0xA26FC, per hero, 8 bytes per id; identical for the four heroes except where noted)

Cells: damage / stun / reaction / hit stop / sound / score code (points) / spark. Reaction codes: 00 light, 02 light alt, 04 knock down, 06 heavy knock down, 0A burn knock down, 0C launch / burn, 10 and 12 pushback, 14 strong pushback.

| id | weapon and use | damage | stun | reaction | hit stop | sound | score | spark |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0x1C | stone thrown | 20 | 1 | 06 | 8 | 09 | 100 | 0 |
| 0x1E, 0x20, 0x22 | pistol, Uzi, shotgun thrown (empty) | 20 | 4 (0x20 Jack 3) | 06 | 8 | 09 | 100 | 0 |
| 0x24 | knife thrown | 30 | 1 | 06 | 8 | 0C | 100 | 0x15 |
| 0x26 | grenade thrown, direct hit | 60 | 1 | 0C | 8 | 0F | 100 | 0 |
| 0x28 | dynamite thrown, direct hit | 20 | 1 | 04 | 8 | 09 | 100 | 0 |
| 0x2A | stick thrown | 30 | 11 | 06 | 8 | 0C | 100 | 0x15 |
| 0x2C | knife swing | 20 (Hannah 15) | 6 (4) | 06 (02) | 5 | 0C | 400 | 0x15 |
| 0x2E | stick swing | 20 | 11 (Hannah 4) | 06 (Hannah 02) | 8 (Hannah 5) | 0C | 400 | 0x15 |
| 0x30 | rod swing | 25 | 11 | 04 | 10 | 09 | 400 | 0 |
| 0x32 | rifle butt | 25 | 4 | 10 | 15 | 09 | 400 | 0x18 |
| 0x34 | M-16 butt | 25 | 4 | 12 | 15 | 09 | 400 | 0x18 |
| 0x36 | butcher knife swing | 25 | 11 | 04 | 15 | 0C | 400 | 0x15 |
| 0x38 | torch swing | 25 | 11 | 0A | 10 (Hannah 8) | 12 | 400 | 0 (Mustapha, Hannah 0x18) |
| 0x58 | club, first hit | 10 | 6 | 00 | 1 | 09 | 200 (Jack), 400 (others) | 0 |
| 0x60 | club, strong hit | 30 | 2 | 04 | 4 | 09 | 100 | 0 |
| 0x3C, 0x3E, 0x40, 0x42, 0x44 | thrown container: pail, drum, barrel, stone, boulder (also armor, tires) | 30 (0x42: Jack 20) | 11, 6, 11, 9, 20 (0x42 and 0x44: Mess 11) | 04 (0x42: 00 except Mess 04) | 10, 5, 10, 5, 10 (Mess 5 for 0x3C, 0x42, 0x44) | 09 | 1000 | 0 |
| 0x4E | pistol bullet | 20 | 4 | 14 | 5 | 00 | 400 | 3 |
| 0x50 | rifle bullet | 30 | 11 | 06 | 5 | 00 | 400 | 3 |
| 0x46, 0x48 | Uzi bullets | 10 | 2 | 00, 14 | 1 | 00 | 100 | 3 |
| 0x4A, 0x4C | M-16 bullets | 15 | 2 | 00, 14 | 1 | 00 | 100 | 3 |
| 0x5E | shotgun pellet (6 per shot) | 15 | 3 | 02 | 1 | 00 | 100 | 3 |
| 0x52 | bazooka shell | 40 | 11 | 0C | 5 | 0F | 400 | 3 |
| 0x54 | explosion, rolling or in the air | 10 | 11 | 0C | 5 | 0F | 400 | 3 |
| 0x56 | explosion on the floor (dynamite, grenade, barrel) | 40 | 11 | 0C | 5 | 0F | 200 | 3 |
| 0x6A | blast of a bomb thrown by an enemy (owner 0xFF) | 10 | 2 | 0C | 5 | 12 | 100 | 0 |
| 0x6C, 0x6E | invisible blast boxes (air strike, D2b): 5 and 30, never kill | 5 / 30 | 2 / 1 | 04 / 0A | 5 / 8 | 0F | 100 | 0 |

Score of a hit: code 0x14 = 100, 0x18 = 200, 0x1C = 300, 0x20 = 400, 0x38 = 1000 (D2a 14). A thrown or lifted container also gives **+500 at the throw** and +500 when it breaks (`item_score_500`); M: barrel_jack score 500 at the throw, +1000 at the hit.

### 7.2 What the number on the screen is

`damage = record * charge scale (only kind 0x203) * C6/32 * type and level factor (0x2BECE / 32) * weak spot factor * HP row and band factors` (D2b 5.4 has the full order).
On a level 0 thug at full HP with `C6 = 42` the result is about `record * 1.4`: 20 gives 28, 25 gives 35, 30 gives 41, 60 gives 82. The type factor is 1.0 for all common enemies at all levels; **exceptions**: Zeke x2.0, the big Wrench (`name 0x24`) x1.5, Hogg x1.5 at level 0 falling to 0.8, and the bosses Slice, Vice, Walther, Butcher, Morgan, Morgue, Slisaur, Tyrog, Fessenden falling from 1.0 to 0.5 or less with the level (R: table 0x2BECE,
`re/out/d2c/dump_typefactor.py`). The **counter hit** bonus of D2b 5.4 (x1.5 for most attack ids of the enemy) also applies to weapons: an enemy that is hit while it is in an attack frame takes x1.5.

### 7.3 Weak spots and armour that are specific to weapons (R: table 0xA32FA, 4 bytes per name id: first attack id / 2 and factor, second attack id / 2 and factor)

The first pair of every normal enemy is the running jump attack 0x10 with factor 48 (x1.5) and the second the dive 0x0E x48. The exceptions (from the dump of the table):

| enemy name id | first pair (attack id, factor/32) | second pair | meaning |
| --- | --- | --- | --- |
| 5 Slice, 0xF Butcher | **0x52 bazooka shell x16 (half)** | dive x33 | shell does half damage |
| 0x15 Morgan, 0x16 Morgue, 0x18 Slisaur, 0x19 Tyrog, 0x22 Fessenden | 0x52 x16 | dive x48 | shell halved, no running jump bonus |
| 0xC Vice T. | 0x52 x16 | **0x5E shotgun pellet x24 (0.75)** | shell halved, pellets 0.75 |
| 0xD Walther | running jump x48 | 0x5E x24 | pellets 0.75 |
| 2, 0x12, 0x13 (Poacher J, Skinner, Gutter) | running jump x48 | throw 0x0A x48 | |
| 0xE Lash | running jump x48 | throw 0x0A x64 | |
| 0x26 Hogg and the ids above it (props) | 0x52 x64 (double) | jab id 0x00 x64 (double) | the shell and the jab do double damage to Hogg |

So the bazooka, the strongest weapon against normal enemies, is **weak against bosses**, except Hogg, who takes double. D2b 5.4 lists the same table (it names the jab bonus of Hogg and the 0x5E pellets) but not the shell id 0x52.

### 7.4 Measured on other enemy kinds (level 0, `$C6` 42, enemy set to 100 HP, M: d2c-big kind_*)

| enemy (class, variant) | knife swing (0x2C, base 20) | pistol bullet (0x4E, base 20) | bazooka shell (0x52, base 40) |
| --- | --- | --- | --- |
| Ferris (class 2) | 28 | 27 to 28 | 55 to 56 |
| Punk (class 20, variant 0) | 27 | 28 | 55 |
| Blade (class 14, variant 4) | 28 | 28 | 55 to 56 |
| class 10 variant 4 (Hammer family, HP 64 at level 0) | **42** | **41** | **82** |

The weapons give the same numbers against all common enemies. The class 10 variant 4 enemy took one and a half times in all three tests: it was in an attack animation (state 0x0A08, frame class 4 from t = 35 until the hit at t = 46) when the hit came, so this is most likely the **counter hit bonus** (the table 0xA320C by the enemy's own attack id, D2b 5.4), not a type factor; the pinned thug and the Punk stood still. Against bosses the factors of 7.3 and the level factor of 7.2 apply (not measured with weapons, open item 16).

## 8. Wear, breaking, lifetime

R: `+$A6` wear, mask table 0x2CB80, `weapon_wear_check` 0x248F6, `weapon_dropped` 0x247FA, `weapon_item_ground` 0x2463C.

- **Wear per use**: +0x20 for every shot, swing or throw; +0x30 for the club (type 10 swing). The byte wraps at 256.
- **A weapon breaks only when it is dropped by a hit or a death** (drop flag set) and lands: the hero's drop flag becomes `+$AE` of the item; with `+$AE != 0` the item checks on landing:
  bit `rand() & 31` of the mask `L[0x2CB80 + 4 * (wear >> 5)]`; the masks have 0, 6, 12, 16, 20, 24, 28, 32 of 32 bits set (chance 0, 19, 38, 50, 63, 75, 88, 100 percent for wear 0x00, 0x20, 0x40, 0x60, 0x80, 0xA0, 0xC0, 0xE0).
  A break: the item blinks for 30 frames (`+$80`, odd frames drawn) and is gone. Exceptions: the butcher knife (var 0x12) never; a bazooka with 0 rounds always; dynamite is armed instead (6.2).
  M: d2c-f wearff_*: wear 0xE0 broke the knife, pistol, club, rod, stick, torch and Uzi (butcher knife did not); wear 0x80, 0xA0, 0xC0 broke the knife; wear 0x00, 0x20, 0x40, 0x60 (3 samples at 0x60) did not (chances 0, 19, 38, 50 percent).
- A weapon dropped by a swap or a special has `+$AE = 0`: **no break check** ever (it just lies for 3600 frames).
- **On the floor**: a placed or enemy-dropped weapon has no life (`+$A4 = 0`). A dropped weapon lies 3600 frames (section 2.2). The knife (var 4) has a second rule: `G.$4FA` counts the knives lying on the floor each frame in slot order;
  the first three stay (their counter `+$AA` is reloaded to 0xB4 every frame), the 4th and later ones count `+$AA` down from the init value 0x258 (600) and **break after 600 frames** (blink 30 frames). M: d2c-f floor_knives_long (at t = 604 the 4th and 5th knife were breaking).
  The Blade and Razor knife parts (D3a) turn into a pick up knife only when fewer than 2 knives lie around (`G.$4FA < 2`, R: 0x51350).
- A thrown knife or stick vanishes on its first hit; a thrown gun bounces back after a hit and is spent after 4 bounces; stones after 4 bounces; all blink 30 frames and are freed (state 0x24).
- The wear byte of the 48 round guns wraps after 8 rounds, so the wear index of a half used Uzi is arbitrary.

## 9. Heavy objects (containers), lift, carry, throw

R: pool E class 5 (handler 0x28C76; `item/container.rs`), throw tables 0x29860 (attack id by variant) and 0x2986E (velocities by hero and variant). The containers themselves (variants, HP 4, drops) are D4a and D3e.

- Variants (`+$26`): 0 pail, 2 drum, 4 barrel, 6 stone, 8 boulder, 0xA armor statue (solid, cannot be lifted), 0xC tire stack. HP 4 (explosive barrel `+$5C != 0`: HP 50 and a fuse of 4 blink stages).
- **Lift**: attack while standing at it with the stick pushed towards it (not needed for stone and boulder). State 0x14 with animation 0:0x16 (21 frames), then the item is held. M: barrel_jack (press t = 10, held t = 30, idle t = 31). A tire stack was **not lifted** in the test (the press with the stick towards it made a jab and the stack broke, M: tire_jack): it may not be liftable at all in practice (open item 15; D3e).
- **Carry**: slow walk tables (R: SPEC 4.9.1), jump allowed with a heavy jump vector class 0xC and animation per hero (air state 0x1E, landing 0x20); **Hannah cannot jump with it**; no jump attacks; run is not possible; the special attack drops it (discards).
- **Throw**: attack: state 0x16, thrown on the frame byte 1 (7 frames after the press, Mess 8): `+500 points` at once to the thrower, attack id by variant, velocity (vx, ax, vy, ay) from the table:

| variant | attack id | Jack | Mustapha | Hannah | Mess |
| --- | --- | --- | --- | --- | --- |
| 0 pail | 0x3C | vx 1536, vy -768 | 1408, -1024 | 1280, -1024 | 1792, -1024 |
| 2 drum, 4 barrel | 0x3E, 0x40 | 896, vy -256, ay -56 | 896, 0, -56 | 768, +128, -72 | 1024, -512, -72 |
| 6 stone, 8 boulder | 0x42, 0x44 | 1536, -1024, ay -21 | 1280, -1024 | 1088, -1024 | 2048, +256, ay -40 |
| 0xA, 0xC | 0x44 | 1536, -768 | 1280, -768 | 1024, -768 | 1280, -768 |

  (Numbers of 8.8; a negative vy is downward: the object is thrown from above the head and falls. The barrel flew 24 frames and moved about 73 px (Jack); M: d2c-big barrel_* hit a thug at 150 px for 41 each.)
  The thrown object hurts enemies (30 damage, knock down, 1000 points per hit), breaks on landing or on a hit (debris, drop roll, sound 0x74 or 0xFA, shake 2) and **does not give the +500 break bonus after a hit** (M: score 500 at the throw, 1500 after the hit).
  A thrown container **flies 4 pixels nearer than the thrower's depth** (M: d2c-g ffi_barrel2_*: barrel Z 292 for the thrower at 296).
- **Explosive barrel** (`+$5C != 0`): a hit or a thrown object that touches it lights the fuse; 4 stages (blink counts 3, 2, 1, 0 of lengths 20, 10, 5, 2 and 2, 3, 3, 6 frames, R: 0x28F24 / 0x28F28), then the big explosion (0x56). A **thrown** explosive barrel hits an enemy like a plain barrel (41 on the thug) and **explodes where it lands** with the big blast 0x56 (M: d2c-big tntbarrel_jack: hit t = 88, blast object at t = 91, x 1123); the blast belongs to the thrower.

## 10. Ammo box (pool E class 2, 0x283DE, HUD word MAGAZINE on the web page)

The box is a pick up item of kind 0x0100. The pick up takes the lowest kind word, so a box (0x100) is taken **before** a weapon (0x2xx) when both lie on the same spot, and food (0x00tt) before a box. Rules (R: `item_ammo_pickup`, `item_ammo_refill` 0x284E2, table 0x2855A):

- A hero holding a gun (kind 0x204 to 0x209) with the box: the gun's rounds go to the maximum (6, 6, 48, 48, 6, 4) and its kind returns to the full gun (an emptied rifle becomes a rifle again, not a club). If the gun was **already full**: **+1000 points** instead.
- Holding a knife, rod, club or anything without a refill row: the box is worth **+1000 points** and the hero remembers it (`+$10F = 1`): the **next gun he picks up is filled to the maximum at once** (also when it is empty).
- No weapon in hand: the same as the line above.
- The refill is instant on the pick up (state 0x14, 10 frames).

M (d2c-big): ammo_refill (pistol 4 rounds, box at t = 130: 6 rounds at t = 140), ammo_full (full pistol: +1000 at t = 60), ammo_first (box first: +1000 at t = 20; then a pistol with 1 round picked at t = 70 shows 6 at t = 72), ammo_rifle (2 rounds to 6).
Drop rows give the box through the "guns on screen" rule: code 0x0F in a drop row spawns an ammo box when a gun (types 0, 1, 3, 4, 5, 6) lies or is held on the screen, else the fallback treasure row 0xB64D2 (R: SPEC 13.3).

## 11. Where weapons come from

R: scripts `re/specs/enemy/stage_scripts.txt`, drop rows `re/specs/enemy/items_drops.txt` (the full decode is D4a; this is only the weapon part), code search for the creators of pool E class 0.

| type | placed in the script | drop rows (container table 0xB64F2 / enemy table 0xB6D12) | other makers |
| --- | --- | --- | --- |
| rifle | Episode 4 area 2 (X 0x350) | containers row 0x08 (16/32), 0x2F (5/32); enemy rows 0x09, 0x17 | |
| gun | | containers rows 0x01 (31/32), 0x2B (31/32); enemy rows 0x1D, 0x20, 0x24 | |
| knife | | | **Blade and Razor**: a hit knife part becomes a pick up knife (class 34, D3a); the knife of class 41 |
| Uzi | | containers 0x06 (31/32), 0x2C (8/32), 0x2B (1/32); enemy rows 0x12, 0x27 (7/32), 0x2A (30/32) | |
| M-16 | Episode 4 area 2 (X 0x3A0, 0x3F0) | containers 0x01, 0x06, 0x08 (1 to 16 of 32), 0x2F; enemy row 0x29 (29/32) | |
| shotgun | Episode 4 area 2 (X 0x350) | container 0x11 (32/32); enemy row 0x27 (24/32) | |
| bazooka | | container 0x21 (32/32); enemy rows 0x27 (1/32), 0x29 (1/32); the join and continue bazooka (0x65A4) | |
| rod | | containers 0x12 (32/32), 0x2F (26/32); enemy row 0x24 (4/32) | |
| stick | | | the broken rod |
| butcher knife | | | Butcher boss (class 52 helper makes both) |
| club | | containers row 0x04 (32/32) | |
| torch | Episode 5 area 0 (X 0xDD, 0xF4, 0x108), area 1 (0xF70, 0xFA0) | | |
| dynamite | | containers 0x05 (16/32), 0x19 (32/32), 0x2A (10/32); enemy rows 0x13 (32/32), 0x15 (16/32), 0x1C (8/32) | thrown by enemies (D3) |
| grenade | | containers 0x05 (16/32), 0x1B, 0x1C, 0x23; enemy rows 0x10 (24/32), 0x25 (32/32) | thrown by enemies (D3) |
| stones | Episode 4 area 0 (X 0x14B, size 2), Episode 7 area 0 (X 0x720, size 1) | | **debris of stone and boulder containers** |

The weapon codes of the drop rows are `code - 1` = type (code 1 is the rifle, 2 the gun, ... 0x0F is the "guns on screen" special). The enemy that drops is the one with the row in its spawn record (`+$96`). Episode by episode the rows used are listed in the scan
`re/out/d2c/scan_players.py` (counts) and by D4a.

## 12. HUD, effects, what the player sees

- **HUD weapon display** (R: 0x2389C to 0x23EDA, redrawn at a pick up and when the ammo changes; stub in the port `hud.rs`): at the bottom of the screen in the block of the player (P1 left, P2 middle, P3 right, 128 px each, M: d2c-g hud_3p): a blue tile 24 x 24 with
  the weapon picture (guns) or the generic **"ARM"** word (knife, rod, stick, butcher knife, club, torch, dynamite, grenade, stones, and an empty gun that is now a throwable: it shows "ARM GUN"); to its right the **ammo number in white, red when 1 is left** (guns only),
  the **name** (RIFLE, GUN, SHOT GUN, M-16A1, BAZOOKA, UZI, KNIFE, ROD, STICK, B. KNIFE, CLUB, TORCH, DYNAMITE, GRENADE, STONE) and under it a yellow **cartridge bar** (guns only; its length is the rounds left divided by the maximum, so a full gun of any kind shows the same full bar: 5 of 6 rounds is a little shorter, 1 of 4 rockets a short stub). M: d2c-f hud_* and d2c-big pistol_jack,
  bazooka_jack snapshots (`re/out/probe/..`, not committed).
- Effects used by weapons (D3f owns them): muzzle flash class 0x21 (0x96B40), sparks 0x1F, shell puff and dust (0x92E5E), hit sparks (0x12720), debris class 0x2C (0x934F6), flames class 0x20, the torch glow (pool E class 6), the fuse spark class 0x19 (0x9625C) of explosive barrels,
  explosions (pool E class 1: 5 kinds, animation table 0x28102).
- Sounds: see section 4; hit sounds come from the record byte 4 (03 light, 06 medium, 09 heavy, 0C blade, 0F explosion, 12 burn); the break sound of a rod 0x6F / 0xF6; the shell explosion 0x66.

## 13. The player count `G.$4EF` (players minus one)

R: routine 0x655E (port `flow_count_players` and `player_count_players`, DONE): `G.$4EF` = the number of player objects with `+$00 != 0` (present) and the mode byte `+$04 != 0` (not removed; a player in the death sequence still counts), minus 1, at least 0.
It is recomputed by the callers 0x3864, 0x44AC, 0xAC74 (area start), 0xACE0 (sub-area), 0x18AA8 and 0x18C04 (respawn and init), 0x1AF8A, 0x1BBD2, 0x1C01A (death and join paths). **During scripted control** (`G.$4DD != 0`) the routine at 0x1DB40 recomputes it as
"number of set bits of the mask `G.$4DD` minus 1" (the scripted players), so a scene with one scripted hero sees 1 player.
M: d2c-big dead_p2_of2, dead_p3_of3, dead_p2_of3: a player poked to 0 HP with no lives left: the death sequence runs (mode 4 at t = 206), **`players` fell by one at t = 268** (61 frames later) and in the same frame the rank fell (12 to 8: level 12 to 6 for 2 players going to 1);
the other players stayed at their numbers. `G.$4EF` read by the probe is "players - 1" (0, 1, 2).

**Everything the count changes** (26 read sites found by a scan for the displacement 0x4EF of register A5 in the code area; R):

| what | address | effect of 2 and 3 players |
| --- | --- | --- |
| level column | 0x18592 (`compute_level`) | `level = W[0x1860C + 2 * (rank * 3 + count)]` with the clamps: 1 player rank at most 24, 2 players rank 4 to 28, 3 players rank below 8 is raised to 4 (table in 16.4) |
| rank speed change on a continue or join | 0x18542 | index = count (and `+ (rand & 1)` when the count is 3 players): `W[0x187F4 + dip * 8 + 2 * index]` (values -7 to 0, table in `rank_tables.txt`) |
| spawn records | 0xFDAE (screen X twin 0xFE6A) | flag bits 5 and 6: `0x40` = skipped with 1 player ("2 or 3 players"), `0x60` = skipped unless 3 players ("3 players only") |
| wave thresholds | 0x7B3C4 (pool B class 15) | `alive <= thr[count]` with three thresholds per group (1P, 2P, 3P) |
| reinforcement controllers | 0x7BFB8, 0x7C040 (pool B) | spawn the next record while the last frame's enemy count is below `{4, 5, 6}[count]` (byte table 0x7BFEE) or a 6 byte row table per group |
| boss HP | 0x31E14 | `HP = W[0x31E52 + boss index * 48 + count * 16 + ((level & 0x1C) >> 1)]` (table in 16.3) |
| Slisaur starter | 0x81958 | with 2 or 3 players one more Slisaur (class 71 at X 0x580, Z 0x20) is made (the web page: "2 for 1 player, 3 for 2 or 3 players") |
| camera lead | 0x20B4C (stage 2: 0x226C8) | `0xC0, 0xE8, 0xE8` pixels (D4c T02) |
| drop in position | 0x62D6 | X = camera X + 96 (1 player), 64 and 128 (2 players), 48, 96, 144 (3 players); depth 32, 48, 64 (D4c T05) |
| end of stage walk | 0x1E4E4 | the target position per hero index and count (D4c T20) |
| scripted recount | 0x1DB40 | see above |
| HUD, tally and bonus layouts | 0xBA76 and 0xBAE2 (clear tally panels), 0xC268 and 0x7CCD8 (JOIN-IN and message arrows), 0x7C4EA, 0x7C544 (pool B panels), 0x93B4E (the respawn flash effect picks its picture by 1 or 2+ players) | layouts for 1, 2, 3 blocks (D4d) |
| the counter itself | 0x659E | |

Not read from `G.$4EF` but from the present flags of the three player objects: the enemy target choice (SPEC 5.2), the camera (leftmost and rightmost player), the clamp, the join and continue tasks, the tag team trigger and the friendly fire passes (they need two players in the list 0x6718).

## 14. Friendly fire: two different passes

The collision phase 0x10546 runs, in this order: **0x10664 (players against players)**, 0x10DCC (attack objects against players), 0x10F96 (player attack objects against enemies), ... 0x115B0 (enemy attacks against players), 1185A (thrown bodies).
Both friendly fire passes are skipped while `G.$4DD != 0` (scripted control); 0x10DCC is also skipped for the debug flag `G.$4AA` with bit 3 of `G.$23`. R: 0x10546 to 0x10572.

### 14.1 Pass 0x10664: fists, kicks, throws, dives (a hero's own attack frames)

R: 0x10664, window table 0x106F2, hit code 0x10706. **Gate = the number of enemies alive in the last frame's census `G.$4F3`** (pool A objects with `+$7D >= 0` and routine byte `+$04 < 4`; dinosaurs and Vice T. have `+$7D = 0xFF` and are not counted; D2a section 10).
Conditions for attacker A and victim V: A's frame class is 4; V is not A; V's `+$00` bit 1 is clear (not invulnerable: not in a hurt state, not blinking, not tied up); A's attack serial `+$5E` differs from V's `+$E9` (one swing hits a friend once); A has an attack box and V a body box;
V's frame class is not 6, 8 or 0xA (specials, hurt, falling frames are immune; 0, 2, 4, 0xC, 0xE hit); the boxes overlap (0x128F8); and `G.$4F3 < 3` with the depth window `A.z - V.z + offset <= limit` (unsigned):

| `G.$4F3` (enemies alive) | 0 | 1 | 2 | 3 or more |
| --- | --- | --- | --- | --- |
| (offset, limit) | (1, 2) | (0, 1) | (0, 0) | off |
| depth difference `A.z - V.z` that hits | -1, 0, +1 | 0, +1 | 0 | none |

**Measured with three players** (M: d2c-big ff3_n0 to ff3_n4, Jack jabs, Hannah is the victim 34 px in front, Mess far away; thugs pinned at the right edge; `G.$4F3` read from the log equals n): exactly the table. With **two players** (ff2_n0 to ff2_n3) the same.
The result per step: n = 0: dz -2 no, -1 yes, 0 yes, +1 yes, +2 no; n = 1: -1 no, 0 yes, +1 yes, +2 no; n = 2: 0 yes, +1 no; n = 3 and 4: never. A swing hits **every friend in its box**: in ff3_multi the first jab at x 1000 hurt Hannah (x 1010) and Mess (x 1055) in the same frame
(both 100 to 99, light flinch), the four hits of the chain brought both to 96, the finisher knocked both down (reaction 6), the special did nothing. **HP never drops below 1**: Hannah poked to 3 HP went 3, 2, 1 and stayed at 1 through three more hits including a knock down (M: ff3_hpfloor).
The probe `jump` leaves no enemy near camera X 928, so `G.$4F3` is a clean 0 there.

Effect on the victim (R: 0x10734..0x107F4; the port `pass_10664`, DONE except the gate): `+$00 = 2`, state 0x02020000 (the hurt dispatcher), the hit timers `+$CB = 0x3C`, `+$CA + 1`, `+$E9` = the attacker's serial, `+$68` = the attacker,
HP minus 1 (a result of 0 or less becomes 1: `+$6E` keeps the old value), stun meter `+$78` minus the record's stun, the record's reaction (0 light, 4 knock down, ...; 0x22 stays 0x22 when the victim is in the bazooka recoil), hit stop of the record on both heroes, hit sound,
hit spark (0x12720) of the record, `+$9A` = record byte 7, the weapon drop 0x1177A (Hannah keeps guns, Mess the bazooka, section 2.2) and a new `$C6` roll (0x11738). Score: **none** for the attacker.

### 14.2 Pass 0x10DCC: attack objects (weapons, bullets, explosions) against players

R: 0x10DCC (jump table 0x10E34, hit code 0x10E4A). The attackers are the objects of the list 0x671A (registered by `item_register_attacker` 0x4BDA: a swung weapon on its swing frame, a thrown weapon or container in flight, every bullet, pellet and shell, explosions and blast boxes) whose frame class is 4.
For each player V in the list 0x6718: V's `+$00` bit 1 clear; the owner `+$02` of the object differs from V's `+$02` (**an object never hurts its owner**); the object's serial `+$5E` differs from V's own `+$5E`; the victim's frame class is not 8 or 0xA (the table has "hit" for 0, 2, 4, **6**, 0xC, 0xE: unlike the fist pass the special frame is
not immune here, R only, not tested); the attack box and the body box exist and overlap; then:

- **Owner is a player (`+$02 >= 0`)**: the depth must be **exactly equal** (`object Z == V Z`, not a window); the attack ids 0x6E and 0x6C (the air strike boxes) never hurt; the damage is **1 HP, never below 1** (not the record's damage).
- **Owner is an enemy (`+$02` negative: bombs, bullets, thrown things of enemies)**: no depth test beyond the box; the damage is the **record damage** of the object's attack id (hero table row 0), HP can drop to 0 (it can kill), no factors.
- Both: `+$00 = 2`, state 0x02020000, the weapon is dropped **without the hero exceptions** (`+$B6 = 0`, `+$C5 = 1` unconditionally: Hannah loses her gun to a friend's bullet, R only, open item 2), the record's stun, reaction, hit stop (also on the holder of a swung weapon), sound, spark, `+$9A`, `$C6` re-roll.

**Measured** (M: d2c-big ffi_*, Jack attacker at x 1000, friend Hannah at the same depth, 4 enemies alive so that pass 0x10664 is closed; the gate of 14.1 plays no role here):

| attack | what happened to the friend |
| --- | --- |
| pistol bullet at 100 px | 100 to 99, push back (reaction 0x14) at t = 75; the same with 0 enemies alive; **dz = 1: nothing** |
| Uzi, held fire | 100, 99, 98, 97, 96: one HP per bullet that arrives (4 bullets), flinch or push back by the bullet id |
| shotgun, 4 pellets in reach | 99, 98, 97, 96 in 4 consecutive frames (t = 74 to 77), then a knock down when the stun meter ran out (the 0x5E record itself is only reaction 2) |
| club swing | 99 and a knock down (record 0x60 reaction 4); dz = 1 nothing |
| torch swing | 99 and the **burn knock down** (reaction 0x0A) |
| butcher knife swing | 99 and a knock down |
| knife thrown | 99 and a heavy knock down (reaction 6); dz = 1 nothing |
| stones thrown | 99 and a heavy knock down |
| dynamite thrown | 99 and a knock down by the direct hit (t = 93); the blast later (t = 255) hurt again: 98 and the launch reaction 0x0C; dz = 1 nothing |
| bazooka shell | 99 and the launch reaction 0x0C |
| barrel thrown | **nothing** at 40, 60, 80, 84, 88, 92, 96 px: the barrel flies at Z 292, the thrower is at 296 (a container is held 4 px nearer), so it passes in front of the friend (the code path is the same, an equal depth should hurt, open item 4) |
| dropped dynamite (a friend jabbed the holder) | it lit itself on landing, 60 frames later it exploded: the **dropper took nothing**, the friend that had jabbed (34 px away, same depth) took 1 HP and the launch reaction 0x0C (M: d2c-f dropped_dynamite). A dropped grenade did nothing |

So with a weapon, **friendly fire is always on** (not only with fewer than 3 enemies); the rule is "same depth". A gun fired along the lane in which a friend stands hurts him, a pellet burst costs up to 4 HP, one explosion 1 HP.

### 14.3 Port

`pass_10664` is ported but `player_g4f3()` returns the constant 3 (`player_combat.rs`): the pass is off (fix: the census count `spawn.g.alive`, 1 line, task P-D2c-1). `pass_10dcc` is an empty stub in `enemy_combat.rs` (task P-D2c-2): also needed for **enemy-owned bombs hurting players** (Episodes 4, 5, 7, 8).
Nothing in the harness records two players, so the passes were never compared.

## 15. Tag team (the two player moves)

D2a section 9 has the states and numbers. D2c adds the exact trigger rules and the measurements the port needs. R: trigger 0x6728 (called for every player every frame at the start of its update), start 0x686E, follow 0x6922, tables 0x66F4 (the two other players of each player), 0x6902, 0x69D4.

### 15.1 The trigger (R: 0x6728 to 0x686C)

Every player A runs this each frame (the partner slots `+$C2` and `+$C3` are two counters for "the other player 1" and "the other player 2" in the order of the table 0x66F4: P1 checks P2 then P3, P2 checks P1 then P3, P3 checks P1 then P2):

1. A already tied (`+$C1 != 0`): nothing. A's `+$00` bit 0 clear (A is invulnerable or in god mode): **return without touching the counters (the counters are paused)**. A holds a heavy object (`+$B6 == 0x20A`) or has input locked (`+$11A`): return.
2. **A has any input (`+$A0 != 0`): both counters are reset to 90 (0x5A)** and return.
3. For each other player B, in order: the counter is **reset to 90** unless all of this holds: B's `+$00` bit 0 set (**B not invulnerable**: a hurt, blinking, just picked up or just special hero cannot be a partner), B's mode `+$05 = 0`, B does not hold a heavy object, B's frame class `+$2E = 0`
   (standing, not walking), B's action `+$C8 = 0` (and not 0x12), B's input `+$A0 = 0`, and the positions: `A.x - B.x` from **-32 to +31** and `A.z - B.z` from **0 to 15** (A is level with or deeper on the screen than B: A has the larger Z).
4. When it holds, the counter is decremented; at 0 the tie up starts with B (0x686E) and the check for the second B is skipped in that frame (B is now invulnerable: the second counter would reset).

So the initiator is the player whose Z is larger by 0 to 15 (for equal Z both could be, the player that is updated first, the lower number, starts). A tied or invulnerable player cannot be a partner. Weapons in the hands (except heavy objects) do not matter.

### 15.2 Measured

M: d2c-big tie_*, tie_dx*, tie3_*, tie_weapon*, tie_inputbreak; d2c-f tiez_*, tierun2_*, pick_*. All tie ups started **90 frames after the heroes were first eligible** (t = 92 after the jump, the counter starts at t = 2):

| test | result |
| --- | --- |
| Jack (P1) and Hannah (P2), x 1000 and 1010, same Z | P1 state 0x1A at t = 92 (until 137), P2 state 0x204 (Let's Go) until 165 |
| initiator Mustapha, Hannah, Mess with the partners Jack, Mess, Mustapha | partner states 0x424 (Let's Roll), 0x422 (Jump), 0x426 (Let's Fly), landing 0x404; t = 92 for Mustapha and Mess as initiators, **t = 152 for Hannah with Mess as the partner** (Mess is invulnerable until t = 62, see below) |
| X offset `A.x - B.x` of P1 to P2 | +31 and -32: tie, P1 initiator; +32: P2 became the initiator (seen from P2 the offset is -32); -33 and +33: **no tie** |
| depth `A.z - B.z` of P1 to P2 | 0, 1, 14, 15: tie, P1 initiator; 16 and 17: no tie; -1 and -15: P2 initiator; -16: no tie (M: d2c-big tie_dx*_dz*, d2c-f tiez_*; the playfield clamps Z to 288 to 335 in this area, so the tests used Z 330 down to 313) |
| P2 or P3 an invulnerable start state (Mess and Mustapha are in a special at the start and are invulnerable until t = 66 to 69) | the tie of Jack (P1) with such a P2 started at **t = 152** with P2 as the initiator: P2's own counter is only paused while it is invulnerable (resumed at t = 70), P1's counter is reset by P2's invulnerability and ran from 70 to 160, so P2 won by 8 frames (my reading of the code; the logs show the start at 152 and P2 as the initiator) |
| partner holds a gun | the tie works; the gun stays in the hand (Jack held a pistol as the initiator: `+$B6` 0x204 the whole time) |
| a stick tap by the initiator at t = 60 and 150 | the counters were reset at t = 61 and 151, the tie came at t = 241 (150 + 90 + 1) |
| three players at x 1000, 1010, 1020, same Z | P1 ties P2 at t = 92; P3 waits; **P1 ties P3 at t = 317** (P1's own blink of 180 frames ended at 137 + 180) |
| three players at Z 296, 300, 304 | P2 (Hannah) is the initiator with P1 (Jack, state 0x422) at t = 92; P3 cannot start later while both are blinking |
| P1 far away, P2 and P3 close | P2 initiator, P3 partner (state 0x422); P1 and P3 close: P1 initiator, P3 partner (0x204): **any pair of the three** |

### 15.3 Jack's Let's Go (the partner run)

The partner of Jack runs at its **own super speed** (D2a table: 8.5 px per frame for Hannah) for the first 20 frames straight, then with the stick. **Attack with a direction held** during the run: attack id **0x1A** (the partner's own hero row: Jack 15, Mustapha 15, Hannah 20, Mess 20 damage; stun 11, reaction 4 knock down, hit stop 8, score 1000), animation 1:9;
**with no direction held**: the idle entry and a normal jab. M: d2c-big tierun_hannah_dir, _nodir (state 0x204 with `atk` 26 = 0x1A at t = 127 when `2R` is held and `2A` pressed; the jab with `2A` alone); d2c-f tierun2_hannah_hit: the run attack hit a thug at 1215 for 18 (Hannah's `$C6` was 32 and the thug had its own 44 HP, so the HP band factor is lower) and knocked it down;
`tierun2_hannah_jump`: a jump in the run gives the running jump (state 0x408); `_left`: with the stick held left the first 20 frames of the run still go straight ahead (to the right) and the attack at t = 134 hit the thug the same way. The partner is invulnerable (blink) the whole time, so enemies cannot stop the run.
For the three air versions (Mustapha, Hannah, Mess as the initiator) D2a section 9.5 has the flight table; an attack held in the air starts record 0x10.

### 15.4 Port

MISSING (trigger, counters, state 0x1A, the four partner states, follow, blink and sparkles): task P-D2c-3 (it replaces P-D2a-2 and adds the pause and reset rules of 15.1 and the 3 player order).

## 16. What 2 or 3 players change, and what is shared

### 16.1 Shared and per player

| thing | shared or per player | rule |
| --- | --- | --- |
| camera | shared (one camera) | follows the rightmost hero with a lead of 0xC0 (1 player) or 0xE8 (2 and 3), never past the leftmost hero minus 16; one hero standing still holds the camera (D4c T02, T03) |
| bounds | shared rules, per player clamp | each hero is clamped to the screen X range and the depth band; heroes do not stop each other (section 17) |
| stage clock | shared | one clock for all; a join adds 3:30, a continue 4:00 (cap 4:00, D4d F29); time over kills everybody at once |
| rank, level, aggression | shared | one rank; the level column depends on the count (16.4); the rank speed and the continue rank drop depend on the count |
| enemy count, waves, thresholds | shared | `G.$4F3` is one number for all heroes; thresholds per count (16.2) |
| spawn script | shared | extra enemies and items for 2 and 3 players (16.2) |
| boss HP | shared (one boss) | by count at the time the boss is made (16.3); a join later does not change it |
| items on the floor and in pool E | shared | one pool of 24 for everybody; a pick up needs the player's own press; first updated player wins (2.1) |
| drops (rolls) | shared tables | the drop rows are fixed per spawn record; the count does not change a roll; with more heroes **more containers** exist (16.2) |
| score, HP, lives, weapon, charge, `C6`, invulnerability, hit counters, damage dealt (the M.V.P. count, D4d) | per player | |
| enemy attention | per player counters | an enemy picks the hero with fewer than 3 attackers first (SPEC 5.2) |
| HUD | per player blocks | 3 blocks of 128 px; the weapon display in each block (M: hud_3p) |
| music, sound | shared | |
| hero choice | per player, unique | no two players hold the same hero (D4d F37) |

### 16.2 The spawn scripts (R: `stage_scripts.txt`, flags of the records; scan `re/out/d2c/scan_players.py`)

A record can carry "2 or 3 players" (flag bits 0x40) or "3 players" (0x60), and the level gates (`level >= n` bit 0x08, `level <= n` bit 0x10, value in the high byte). The records of the scripts (counts of records, not of live enemies: a record can make a prop, a dinosaur or a boss; level gated records are counted apart):

| episode | enemy records 1P | 2P | 3P | level gated | item and container records 1P / 2P / 3P | sum of the wave thresholds 1P / 2P / 3P |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 23 | 27 | 32 | 0 | 7 / 9 / 11 | 14 / 17 / 18 |
| 2 | 30 | 35 | 37 | 0 | 15 / 15 / 15 | 12 / 19 / 21 |
| 3 | 96 | 96 | 96 | 0 | 20 / 20 / 20 | 6 / 9 / 9 |
| 4 | 41 | 46 | 56 | 14 | 24 / 27 / 29 (+2 gated) | 34 / 40 / 45 |
| 5 | 54 | 67 | 76 | 13 | 23 / 25 / 27 (+2) | 44 / 52 / 58 |
| 6 | 48 | 57 | 59 | 0 | 10 / 10 / 10 | 12 / 24 / 24 |
| 7 | 113 | 120 | 125 | 14 | 11 / 11 / 13 (+4) | 92 / 107 / 123 |
| 8 | 88 | 95 | 95 | 0 | 4 / 4 / 4 | 40 / 52 / 53 |

(Enemy records are the pool A records of list 2 plus the wave scripts; the sums of thresholds are over all waits whose limit is below 99.) Episode 3 (the car) has no extra enemies for more players: the car section always sends 96 records.
Wave thresholds (the number of enemies that may stay alive before the next group comes) grow by 0 to 2 per extra player; examples: Episode 1 wave 1 ends at 4 / 5 / 6 alive, wave 2 at 3 / 4 / 4; Episode 6 waves mostly wait for 1 / 2 / 2 enemies alive.
M: d2c-big cnt_e1a0_1p, _2p, _3p (Episode 1 area 0, walking right): `players` 0, 1, 2; the camera lock came at t = 330 (1P) and 360 (2P); enemies alive at the end of the first groups: 1P 4, 2P 4, 3P 5 (the group at X 0x388 is 1 thug for 1P, +1 for 2P, +1 more for 3P: R: records `players>=2` and `players==3`).
Because the thresholds and the flags read `G.$4EF` every time, a **join in the middle of a wave changes the wave at once**.

### 16.3 Boss HP by player count (R: 0x31E14 and `boss_hp.txt`, M: d2c-big boss_*)

`HP = W[0x31E52 + index * 48 + count * 16 + ((level & 0x1C) >> 1)]`; the level at rank 0 is 0, 4, 8 for 1, 2, 3 players (so the band is 0, 1, 2). Measured at rank 0 (1P / 2P / 3P) and the class that calls it (index in brackets):

| boss | class (index) | 1 player | 2 players | 3 players | at rank 24 (levels 24, 24, 28) |
| --- | --- | --- | --- | --- | --- |
| Vice T. | 18 (0) | 350 | 550 | 650 | 550 / 700 / 750 |
| Slice | 25 (0) | 350 | 550 | 650 | |
| Slisaur | 71 (0) | 350 | 550 | 650 | |
| Tyrog 2 (the second form) | 72 (0) | 350 | 550 | 650 | |
| Butcher | 61 (1) | 512 | 576 | 640 | |
| Hogg | 31 (2) | 300 | 400 | 500 | |
| Morgan | 36 (4) | 256 | 320 | 512 | |
| Morgue | 40 (5) | 512 | 576 | 768 | |
| Tyrog (first fight, class 4) | 4 (6) | **256 (half of the 512 of row 6)** | **288 (half of 576)** | **352 (half of 704)** | the init halves the table value (R: 0x36AD4 `asr.w #1`) |
| Fessenden (first form) | 84 (9) | 600 | 680 | 800 | |
| Fessenden (final) | 94 (8) | 400 | 520 | 650 | |
| Rock Hopper (made by Vice) | 29 | 44 | 60 | 92 | level based row |

(The class numbers of the second and third Tyrog and of the Fessenden parts are in D3d.) The Slisaur fight has **one more Slisaur** for 2 and 3 players (R: 0x81958). The reinforcement controllers that bring helpers during a boss fight wait for fewer than 4, 5 or 6 enemies (1, 2, 3 players, R: 0x7BFEE).
A hero's damage to bosses falls with the level (type factor) and some weapons are halved (section 7.3).

### 16.4 Rank and level by player count (R: rank tables)

`level = T[rank][count]` (0x1860C), after the clamp of `compute_level`: with 1 player the rank is cut to at most 24; with 2 players it is held between 4 and 28; with 3 players a rank below 8 becomes 4 (and the table gives 8 for ranks 0 to 8). Selected rows (1P, 2P, 3P): rank 0: 0, 4, 8; rank 5: 3, 5, 8; rank 10: 8, 10, 10; rank 12: 10, 12, 14 (the start rank in the probe runs; M: cnt_e1a0_*);
rank 16: 14, 16, 20; rank 24: 24, 24, 28; rank 31: 25, 29, 31. More players means a higher level at the same rank: enemies deal more damage (type table 0x2C3AE by level: Ferris 32 at level 0, 36 at level 8, 58 at level 31) and level gated enemies appear sooner;
the enemy HP rows start to rise at levels above 21 (Ferris 44 until level 21, then 52, 60, 76). The rank itself is shared and changes by time and events (D4b, D2b).

### 16.5 Join and drop in (D4d owns the details)

A joining or continuing player: HP 100, one extra life, a bazooka, 360 frames of blinking invulnerability, drops in from above the screen at the positions of D4c T05 (D2b 2.6, D4d F29, P57 and P58); the join adds 1 point to the score of all three players, 3:30 to the clock (4:00 for a continue) and 600 frames to the wave timer, costs one credit, and the count changes at once. Two or three players can join in the same frame, each gets its own strip (D4d).

## 17. Contact between players, update order, draw order

**No contact.** M: d2c-big pp_walk (Jack walks right from x 1000 through Hannah standing at 1060: 1.6 px per frame all the way to x 1142, no stop, no push, no state change), pp_both (Jack right, Hannah left: they pass at x 1050), pp_run (Jack runs through), pp_z, pp_jump, pp_three (all three passing).
A walking hero does not grab a friend (the grab probe only looks at enemies, R: 0x1084A over the list 0x671E). Players never register in the obstacle list (only containers and solid props do).

**Update order**: 0x18834 runs the players in index order P1, P2, P3 (R: SPEC 2) before pool E, pool A, pool B and pool C; players held by an enemy (frame class 0xE) are updated afterwards by 0x188DA. Everything with a "first player wins" rule follows from this (pick up, the tag team check, the target counters).
The collision list of players (`G.$6718`, at most 3) is filled in the same order. **Draw order**: `build_sprite_list` (0x13FA6) sorts the objects by the key `(Z << 16) | object address`, ascending, and draws in that order; the sprites drawn first are in front, so the object with the **smaller depth Z (lower on the screen, nearer) is in front** (R: 0x13FA6 and the port `Game::build_sprite_list`).
For equal depth the lower RAM address wins: pool E items (0x8874..) before pool C effects (0x9A74..) before P1 (0xB274), P2 (0xB3F4), P3 (0xB574), pool B (0xB6F4..), pool A (0xC8F4..), pool D (0xDDF4..).
So at equal depth a hero is drawn over an enemy, P1 over P2 over P3, and an item over a hero.

## 18. The four checks that D2a could not do

Inputs: `re/out/d2a/batchE.txt` (re-run inside d2c-big and extended in d2c-f: `diveb_*`, `wjb_*`).

### 18.1 Three player friendly fire

Done, section 14.1 (ff3_n0 to ff3_n4): the windows are exactly D2a's table for 3 players, one swing hits every friend in its box, and two players behave the same.

### 18.2 The dive window

M: d2c-big dive_<hero>_16 to _25 and d2c-f diveb_<hero>_26 to _33 (neutral jump, jump pressed at t = 10, attack held for 3 frames starting k frames after the jump press; "dive" = state 0x40A with attack id 0x0E, otherwise the low jump attack 0x40C with id 0x0C).
The frames in which the attack press **starts a dive** (frame 0 = the jump press; the hero is in the air from frame 1; a press is read in the frame after the log frame):

| hero | split height | frames with `y >=` split (neutral jump) | press frames that make a dive | width |
| --- | --- | --- | --- | --- |
| Jack | 76 | 20 to 24 | 22 to 24 | 3 frames |
| Mustapha | 75 | 19 to 25 | 21 to 25 | 5 frames |
| Hannah | 93 | 20 to 25 | 22 to 25 | 4 frames |
| Mess | 72 | 20 to 23 | 22 to 23 | 2 frames |

(Derived from the 3 frame holds: k = 20 to 24 gave a dive for Jack, 19 to 25 for Mustapha, 20 to 25 for Hannah, 20 to 23 for Mess; k <= 19 gave only the low attack for Jack, Hannah, Mess; k = 18 for Mustapha; k >= 26 only the low attack for the two long windows; the lower end of the window is two frames after the
height reaches the split, the upper end is the last frame at or above the split.) Earlier presses (frames 3 to 19) give the low attack 0x0C at any height of 16 or more, later ones too on the way down. D2a section 4.1 says "2 or 3 frames" for all heroes: that is right for Jack and Mess, **Mustapha has 5 and Hannah 4**.
After a dive the hero is in the dive landing state 0x436 (17 to 21 frames, D2a).

### 18.3 Jack's tag team run attack with a direction held

Section 15.3: attack id 0x1A when a direction is held; the partner hits a thug for the 0x1A record (18 with Hannah's `$C6` of 32 on a thug with 44 HP).

### 18.4 The triangle kick after a wall jump

M: d2c-big wja_* and d2c-f wjb_* (Hannah and Mustapha in Episode 1 area 2 at x 215, z 318, left held, jump at 10, second jump press at 32 gives the wall jump at t = 33; the attack is held for 4 frames starting at 32 + d):

| delay d | 4, 10, 11 | 12 to 16 | 20, 26 |
| --- | --- | --- | --- |
| Hannah | no attack | low jump attack 0x40C (id 0x0C, d8) | dive 0x40A (id 0x0E) then dive landing 0x436 |
| Mustapha | no attack | dive 0x40A (id 0x0E) then 0x436 | dive |

The jump attack is **ignored during the first 14 frames of the wall jump** (the first accepted press frame is t = 47, 14 frames after the wall jump began; d = 11 covers t = 43 to 46 and failed, d = 12 covers 44 to 47 and worked). Then it is the **normal jump attack by height**: Mustapha's wall jump rises above his split height of 75 at once, so his triangle kick is the dive with
its 1.5 factor (D2a section 14); Hannah's arc is lower than her split of 93 at first, so she kicks with the low attack and dives later in the arc. Jack and Mess cannot wall jump (state 0x402 needs hero 1 or 2).

## 19. (a) Acceptance scenarios

The port must reproduce these frame numbers and values. All segment names exist in the batch files of `re/out/d2c/` (generators `gen_w.py` for weapons, `gen_m.py` for two and three players, `gen_f.py`, `gen_g.py`); the common set up is
`jump 1 0 928 char=<hero[,hero,hero]>` (Episode 1 area 0, camera 928, heroes at x 993+ ... as the segments poke them), `god on` for solo weapon tests, `rank 0 ; level 0`, a pinned thug at x = camera + 96 + offset, `objset A hp 100` before each hit. The port side
uses the harness `run` or `step` mode with the same input files (the 3 player scenarios need the harness extension of task P-D2c-7). Damage numbers are for `$C6 = 42` (read `+$C6` from the log; the unit test can set it).

| # | scenario | probe segments (summary) | must match |
| --- | --- | --- | --- |
| W1 | pick up order and lock | pick_lowest (4 weapons on one spot, A at 10, 70, 130, 190), pick_lockout (knife and pistol, A at 10, 30, 80) | held kinds 0x202, 0x204, 0x202, 0x204; hero idle 11 frames after each press; the press at +20 attacks (knife thrown), +70 takes the pistol; `+$F8` 40 |
| W2 | pistol | pistol_<hero> (press 10, shots at 40, 90, 140, last round) | shot at t = 41, bullet vx 2048, round count 6, 5, 4, ... (the test sets 1 at t = 120); hit 27 to 28 at t = 51; the last round makes the kind 0x201 and the hero lets go at t = 153; the dropped pistol lands at t = 201 with `+$A4 = 3600` |
| W3 | rifle, M-16, Uzi, shotgun | rifle_jack, m16_jack, uzi_jack, shotgun_d30 to shotgun_d200 | rifle 41 and knock down; Uzi round every 7 frames, 14 each, bullet vy -336 + 48 per round; M-16 20 each, empty = kind 0x203 kept, butt 35 at +8; shotgun 3 pellets at 30 px (20, 20, 18) and 4 at 60 px and more (20, 20, 18, 17) |
| W4 | bazooka | bazooka_<hero> (4 rounds at 40, 130, 220, 310) | shell 55 to 56 on the thug; hero pushed 28 / 45 / 66 / 15 px (Jack, Mustapha, Hannah, Mess) in about 7 frames; 4 rounds then the hero lets go |
| W5 | knife, stick, butcher knife per hero | knife_swing_<hero>, stick_<hero>, butcher_<hero> | first hit +6, +6, +2, +7 (knife); damage 28 / 27 / 20 (Hannah) / 27; butcher knife +8 (Hannah +9), 34 to 35 and knock down |
| W6 | knife reach | reach_<hero>_<96..136> | Jack: swing hits at 96 and 104, nothing at 112, thrown (hit at +10) from 120; Mustapha near zone 128; Hannah 120; Mess 104 |
| W7 | rod | rod0_jack, rod1_jack, rod2_jack, rod0_hannah, rod1_hannah | 35 per rod hit; breaks into the stick after the 1st, 2nd, 4th, 3rd, 3rd hit with the RNG of those runs (the roll table 19, 38, 69, 88 percent); the stick hits 28 (27 for the others) |
| W8 | club charge | club_hold0, 10, 20, 30, 40, 50, 60 (Jack), club_hold0 and 60 for the others | table of 5.5: 14 + 42, 14 + 42, 16 + 52, 20 + 64, 20 + 64, 28 + 72 (capped), 28 + 72; Mustapha and Mess only one hit (41, charged 82) |
| W9 | torch | torch_<hero> | hit at +12, 35 (34), burn knock down 0x0A, HP unchanged afterwards; overlay object present from t = 4 with Z alternating 295 / 298 |
| W10 | thrown weapons | dynamite_<hero>, grenade_jack, stones_jack, stones1_jack, stones2_jack, dynamite_run_jack | thrown at t = 41; dynamite: direct 28, at rest +54 to +93 later, explosion exactly 60 frames after the rest, 52 to 56; grenade 82 to 85 in the hit frame, chain objects at +1, +11, +21; stones 28 at t = 56 and freed at t = 128 |
| W11 | heavy objects | barrel_<hero>, drum_jack, pail_jack, stone6_jack, boulder_jack, tntbarrel_jack | pick up 21 frames, thrown at +7 (Mess +8), score +500 at the throw and +1000 at the hit; the thug takes 41 (stone container 27); the explosive barrel explodes where it lands (t = 91) |
| W12 | ammo box | ammo_refill, ammo_full, ammo_first, ammo_rifle | 4 to 6 rounds, +1000 when full (t = 60), +1000 first (t = 20) then the next gun at 6 rounds at t = 72, 2 to 6 for the rifle |
| W13 | wear, break, drop | wearff_<weapon>_<wear> (a friend jab makes Mustapha drop), drop_swap_life, drop_enemy_life, special_discard_*, die_with_weapon, floor_knives_long, ffd_* | wear 0xE0 breaks (not the butcher knife), 0x00 never; a swap or special drop never breaks; swap drop gone 3600 frames after landing; the enemy-dropped Uzi still there after 3700; the 4th and 5th knife break at t = 604; Hannah keeps the pistol, Mess the bazooka, the others drop |
| W14 | weapon use while moving | runswing_<weapon>, jumpweapon_* | run + attack: knife thrown (34), dynamite thrown; club, pistol, butcher knife, torch: the dash attack (22 + 22) and the weapon stays; jump + attack: the low attack 0x0C (8) |
| W15 | 3 player HUD and the 15 weapon HUDs | hud_<weapon>, hud_3p (snapshot at t = 60 and 70) | picture + number + name + bar for the 6 guns, "ARM" tile and name for the 9 others; three blocks side by side |
| M1 | friendly fire fists, windows | ff3_n0 to ff3_n4, ff2_n0 to ff2_n3 (jabs at dz = -2, -1, 0, 1, 2) | hp after each jab: n = 0: loses at dz -1, 0, 1; n = 1: 0, 1; n = 2: 0; n = 3, 4: never; the same for 2 players |
| M2 | one swing, two friends; hp floor | ff3_multi, ff3_hpfloor | both friends 100 to 99 on the first jab; 96 after four; the special does nothing; hp 3 to 1 and no further |
| M3 | friendly fire with weapons | ffi_pistol, _uzi, _shotgun, _club, _torch, _butcher, _knifethrow, _dyn, _bazooka, _stones with n = 4 enemies, dz 0 and 1 | 1 HP per object hit at dz 0, nothing at dz 1; reactions of 14.2; the same with 0 enemies; the barrel needs Z - 4 |
| M4 | tag team | tie_<A>_<B> (four initiators), tie_dx*_dz*, tiez_*, tie3_all, tie3_all_z, tie3_p23, tie3_p13, tie_weapon*, tie_inputbreak | start states at t = 92 as in 15.2; boundaries X -32 to 31, Z 0 to 15; the Mess or Mustapha start at t = 152 with P2 initiator; three player pairing; the input breaks reset the count |
| M5 | partner run attack | tierun2_hannah_hit, tierun_hannah_nodir | `atk` 26 (0x1A) with `2R` held; jab without; the thug loses 18 |
| M6 | contact | pp_walk, pp_both, pp_run, pp_z, pp_jump, pp_three | x advances 1.6 px per frame through the other player; no state change |
| M7 | boss HP | boss_<boss>_<n>p_r0, boss_vice_<n>p_r24 | the table of 16.3 (Vice 350 / 550 / 650 and 550 / 700 / 750, Tyrog class 4 256 / 288 / 352, ...) |
| M8 | player count | dead_p2_of2, dead_p3_of3, dead_p2_of3, cnt_e1a0_<n>p, cnt_e1a1_<n>p | `players` drops 61 frames after the death sequence; levels 10 / 12 / 14 at rank 12; alive counts of 16.2 |
| M9 | same frame pick up | pick_same2, pick_next2, pick_same3, pick_next2b | the first updated player (or the earlier presser) gets the weapon; the others jab |
| M10 | dive window and triangle kick | dive_<hero>_16..25, diveb_<hero>_26..33, wja_*, wjb_* | section 18.2 and 18.4 |

## 20. (b) Port status and the port task list

| thing | state | file |
| --- | --- | --- |
| pick up, lock, held kinds, all weapon states (0x14, 0x16, 0x1C, 0x1E, 0x20, 0x22, run swing 0x202, heavy jump), split body | DONE (compared with MAME for the four heroes in phase 1 task 3) | `player_weapon.rs` |
| weapon item class 0, 15 types: held, thrown, flights, wear, break, drop life, floor knife limit | DONE | `item/weapon.rs` |
| bullets class 3, 7, 10, explosions class 1, 8, 11, ammo box, food, treasure | DONE | `item/blast.rs`, `item/small.rs` |
| charge scale (0x11262), item attackers against enemies, damage tail with the weak table including ids 0x52 and 0x5E | DONE | `enemy_combat.rs` |
| heavy objects: lift, carry, throw tables, hits, debris and drops of containers | DONE | `item/container.rs` |
| the joining bazooka 0x65A4 | DONE | `item.rs` |
| friendly fire fist pass 0x10664 | PARTIAL: the gate is the constant 3, so it never hits | `player_combat.rs` `player_g4f3` |
| friendly fire attack object pass 0x10DCC (also enemy-owned bombs against heroes) | STUB (empty function) | `enemy_combat.rs` |
| tag team | MISSING | `player.rs` (comments at `player_run_group` step 4 and the air states) |
| `G.$4EF` recount, spawn flags, wave thresholds, boss HP table, rank and level columns, camera lead, drop in positions | DONE and checked only for one player | `flow.rs`, `player.rs`, `spawn.rs`, `prop.rs`, `enemy.rs`, `rank.rs`, `camera.rs` |
| Slisaur extra for 2 and 3 players, reinforcement controllers with 3 thresholds | MISSING with D3d / D3c | |
| HUD weapon display (picture, ARM tile, number, name, bar, per player block) | STUB | `hud.rs` `hud_weapon` |
| torch glow (pool E class 6) | STUB: the overlay is freed at once | `item/blast.rs` `item_overlay` |
| effects of weapons: muzzle flash 0x21, sparks 0x1F, dust 0x92E5E, debris 0x2C (and the stones it drops), flames 0x20 (done), fuse spark | PARTIAL: flames DONE, others MISSING (freed on the first update) | `effect.rs` |
| Butcher's two knives (class 52 helper), enemy thrown bombs | MISSING (D3c, D3a); the item side exists | |
| weapon sounds (rifle 0x9C76, pistol 0x9CD8, Uzi 0x9D9C, shotgun 0xA442, bazooka 0x9E08, throw 0x9D44, explosion 0x66, voices) | MISSING | `snd.rs` has the routines |
| 2 and 3 player scenarios in the harness | MISSING (no join, no second input stream in `re/harness`) | |

Port tasks (sizes: S under 1 hour, M 1 to 2 hours, L over 2 hours):

| task | scope | size |
| --- | --- | --- |
| P-D2c-1 | `player_g4f3` returns the census count `self.spawn.g.alive` (same as P-D2a-1); scenario M1 for 2 and 3 players | S |
| P-D2c-2 | pass 0x10DCC: attackers of list 0x671A against players (owner, serial, same depth for player owners, ids 0x6E and 0x6C skipped, 1 HP floor 1 against the record damage for enemy owners, drop without the hero exceptions, reaction and hit stop), scenario M3; enemy owned items with D3 | M |
| P-D2c-3 | tag team: trigger and counters with the pause and reset rules (15.1), state 0x1A, four partner states, follow table 0x69D4, blink and sparkles, `held_players_update` for a player partner, three player order; scenarios M4, M5 (replaces P-D2a-2) | M to L |
| P-D2c-4 | torch glow overlay class 6 (0x29B06, tables 0x29C26, 0x29CB6, 0x29CF4) | S |
| P-D2c-5 | HUD weapon display 0x2389C to 0x23EDA with the name table 0x23D0B and the 6 gun names, tiles, the red 1, the bar, three blocks; scenario W15 | M |
| P-D2c-6 | debris effect class 0x2C (0x934F6) with the stone spawn (0x93770), together with the muzzle, spark and dust effects of weapons (D3f) | M |
| P-D2c-7 | harness: boot with 2 and 3 players (`char=` list), a second and third input stream, snapshots of player 2 and 3; then scenarios M1 to M10 | L |
| P-D2c-8 | weapon scenarios W1 to W14 as harness scenarios for the four heroes (inputs from `batchW_big.txt`) and fix whatever differs | M |
| P-D2c-9 | weapon sounds and voices (with D4e / M4) | M |
| P-D2c-10 | rename the types in the port comments (`weapon.rs` header: rod, stick, butcher knife, club, torch, dynamite, grenade, stones; `blast.rs`: shell, pellet) and the SPEC table of weapon names | S |

## 21. (c) Open items

1. **Stones from the debris of stone and boulder containers were not seen.** Twenty breaks (10 stone, 10 boulder, M: d2c-g debris2_*) made 9 debris pieces each (pool C class 0x2C) and no piece took the weapon branch (state 6) and no stone item appeared, although the code rolls 2 in 16, then 8 in 16, then 3 in 16 per piece while `G.$4F1 < 3`.
   Either `G.$4F1` was not 0 (an absolute watch was not accepted by the probe: use `peek 0xFF84F0 2`) or the branch needs something else. Probe: `peek` at the break and 40 breaks.
2. A friend's **bullet or swing makes Hannah drop her gun** (pass 0x10DCC drops without the hero exceptions, R only; the fist pass keeps it, M: ffd_hannah_pistol).
3. The **special frame** is hit by a friend's weapon in pass 0x10DCC (table says class 6 hits, R only).
4. A **thrown barrel hitting a friend** needs the friend at Z - 4 of the thrower (not measured).
5. The **torch** damage scale for charged swings (code: yes) and the web page claim "delaying the swing does not change the torch's damage" were not measured.
6. Burn from the torch: no damage over time on a thug (350 frames); other enemies, dinosaurs and the flame objects: D3f / D4f.
7. **Enemy owned bombs** against heroes (record damage, can kill): item side ported, throwers and hit pass missing (D3a, D3c, task P-D2c-2).
8. A third player as the **initiator** of a tag team (P3 with Z larger) was not seen; the code is symmetric.
9. Whether a hero holding a weapon can **grab** an enemy by walking into it (the walk with a weapon uses the split body; the grab probe belongs to the frame class 2 of the walk frames): not tested (D2a).
10. Exact pixel layout of the HUD weapon display (columns and rows of the scroll 1 cells at HUD area + 0x5C): from the code 0x2389C when the task P-D2c-5 starts.
11. The **Butcher's knives**: how a hero takes them (disarm), D3c.
12. **Two thrown things at once** (pool E full: the weapons and bullets share the 24 slots with items and food); a full pool makes drops and bullets fail silently (R: allocator carry, D4a).
13. The rank effect of a join or of a player leaving in the middle of a level is in D2b / D4b.
14. The first jump attack frames of the wall jump (14 frames of ignored input) may be tied to the animation flag of 0:0x12; the exact flag was not decoded.
15. Whether a **tire stack** (container variant 0xC) can be lifted at all: it was punched in the test; the pick up box of its frame may be too small or it is meant to be thrown only by explosions. D3e or D4a to check with a spawn at several offsets.
16. The Hogg shell and jab bonuses (x2) and the Tyrog (class 4) half HP rule were read from tables and spawn tests; a fight in MAME with each weapon against these bosses was not made (D3c, D3d).

## 22. (d) New rows for mechanics-seen.md

| ID | Mechanic | Source | Port | Notes for the design agent |
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

### Web sources

W: Arcade Quartermaster items page https://www.arcadequartermaster.com/cad_items.html (weapon list and the claims on the knife, club, torch, dynamite, stones, butcher knife, rod), Arcade Quartermaster characters page https://www.arcadequartermaster.com/cad_characters.html
(tag team names, from D2a), Hardcore Gaming 101 https://www.hardcoregaming101.net/cadillacs-dinosaurs/. Where the web and MAME differ or are incomplete: the dynamite fuse ("about 3 seconds": MAME 60 frames after it comes to rest, 2.2 to 3.4 seconds from the throw), the torch ("delaying the swing does not change damage": not measured, the code scales every 0x203 attack),
Hannah's knife (stabs for every hero in melee range, D2a).

### Appendix: files

`re/out/d2c/`: generators `gen_w.py`, `gen_m.py`, `gen_f.py`, `gen_g.py`; inputs `batchW_big.txt`, `batchM_big.txt`, `batchF.txt`, `batchG.txt`, `big.txt`, `smoke.txt`; helpers `an.py` (change events of a watched object), `hits.py` (damage events), `ffsum.py` (friendly fire windows),
`scan_players.py` (script records by player count), `find_ref.py` and `callers.py` (scan the code for a displacement or a call target), `dump_attacks.py`, `dump_bullets.py`, `dump_weak.py`, `dump_heavy.py`, `dump_typefactor.py` (table dumps; no ROM bytes are in this document).
Probe output: `re/out/probe/d2c-smoke`, `d2c-big`, `d2c-f`, `d2c-g`. Pictures (not committed): `weapon_lying.png`, `hud_all.png`, `hud_3p.png`, `hud_weapon.png`.
