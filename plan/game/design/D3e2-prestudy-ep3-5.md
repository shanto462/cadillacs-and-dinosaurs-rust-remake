# D3e part 2 desk pre-study: Episodes 3, 4 and 5

Written 2026-10-13 as the desk pre-study for D3e part 2 (area walk of Episodes 3, 4 and 5; internal stages 2, 3 and 4).
Prepares the ground for the MAME probe walk. No MAME commands were executed during this study.

References:
- Inventory and Episode 1 to 2 walk: [D3e-pool-b-inventory.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D3e-pool-b-inventory.md)
- Script source: [stage_scripts.txt](file:///Volumes/Shanto/Repos/Personal/Github/roms/re/specs/enemy/stage_scripts.txt)
- Mechanics leads: [mechanics-seen.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/mechanics-seen.md) (groups J, L, M, N)
- Background documentation: [claude-doc-recreation-blueprint.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/sources/claude-doc-recreation-blueprint.md), [claude-doc-deep-dive-supplement.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/sources/claude-doc-deep-dive-supplement.md), [claude-doc-deep-dive-supplement-2.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/sources/claude-doc-deep-dive-supplement-2.md)
- Camera and area transitions: [D4c-camera-flow-transitions.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/plan/game/design/D4c-camera-flow-transitions.md)
- Drop tables and item life: [p3.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/re/out/d4a/doc/p3.md), [p4.md](file:///Volumes/Shanto/Repos/Personal/Github/roms/re/out/d4a/doc/p4.md)

Pool B roles:
- BRK: Breakable prop (registers in player hit list 0x4BFA, HP 1 or 8, any hit breaks it)
- EVT: One-shot scenery event (scroll tile rewrite, burst, door)
- ANI: Periodic scroll tile or palette animation
- CTL: Controller or helper (waves, clock, hint, music, vehicle)
- SPW: Spawner of enemies, critters, or ambient sound
- TXT: Caption banner or dialogue box
- FX: Scenery effect object
- FE: Front-end, ending, or cutscene object
- DBG: Debug object (frees itself unless G.$4AA is set)

---

## 1. Episode 3 (Internal Stage 2): Hell Road / Wasteland / Desert of Death

Stage 2 has 1 continuous area (area 0, map width 60 screens = 15,360 pixels; camera can run beyond 17,700 pixels in car mode as the map loops). It uses its own dedicated camera controller (0x22398) with mode table 0x2250A. Its ten camera events at X 0x300 through 0x2D00 act as locks (mode 2) when playing on foot. When a radio is collected, the car sequence overrides these locks with camera mode 8 (auto-scrolling chase at 11 pixels per frame).

### 1.1 Trigger-order inventory of Area 0

All pool B records, pool E records, and special pool A records (bosses, dinosaurs, vehicles, mid-bosses) from list 1 (0xAD042), list 2 (0xAE5F4), and wave scripts (0xAED24), in ascending trigger order:

| Trigger | Pool | Class / Object | Map X, Z, Y | Variant | Role / Item / Drop Row | Details and Notes |
|---|---|---|---|---|---|---|
| 0x0000 | Pool B | B#69 | (0, 0, 0) | var 0 | DBG | Debug cursor. Frees itself immediately unless G.$4AA is set. |
| 0x0000 | Pool B | B#31 | (0, 0, 0) | var 0 | CTL | Stage clock. Displays time remaining at screen (0xE8, 0xB8) when <= 3:00. |
| 0x0000 | Pool B | B#20 | (0, 0, 0) | var 0 | CTL | GO hint. Blinks red GO on scroll1 after 600 frames idle camera. |
| 0x0050 | Pool B | B#66 | (0, 0, 0) | var 0x5B | TXT | Caption banner. Intro text for Desert of Death / Hell Road. |
| 0x0080 | Pool E | E#5 | (0x250, 0x20) | var 4 | Container (barrel), row C:0x31 | First barrel on road. Holds the Radio item (3,000 pts, summons Cadillac). |
| 0x01A0 | Pool A | A#18 | (0x360, 0x30) | var 0x200 | Vice T. / Walther mid-boss | First Walther on foot. |
| 0x0300 | Pool B | B#15 | (0, 0, 0) | var 0 | CTL | Wave controller (wave 0). Lock X 0x300. Empty wave, ends with GO. |
| 0x0310 | Pool E | E#5 | (0x500, 0x50) | var 0 | Container (pail), row 0 | Small orange bucket, empty. |
| 0x0320 | Pool E | E#5 | (0x520, 0x38) | var 0 | Container (pail), row 0 | Small orange bucket, empty. |
| 0x0340 | Pool E | E#5 | (0x540, 0x20) | var 0 | Container (pail), row 0 | Small orange bucket, empty. |
| 0x0340 | Pool E | E#5 | (0x540, 0x44) | var 0 | Container (pail), row 0 | Small orange bucket, empty. |
| 0x0360 | Pool E | E#5 | (0x560, 0x2C) | var 0 | Container (pail), row 0 | Small orange bucket, empty. |
| 0x0380 | Pool E | E#5 | (0x580, 0x14) | var 0 | Container (pail), row C:0x31 | Small orange bucket. Holds the Radio item (3,000 pts, summons Cadillac). |
| 0x0500 | Pool B | B#15 | (0, 0, 0) | var 1 | CTL | Wave controller (wave 1). Lock X 0x500. Empty wave, ends with GO. |
| 0x0900 | Pool B | B#15 | (0, 0, 0) | var 2 | CTL | Wave controller (wave 2). Lock X 0x900. Spawns 4 x A#70 bikers at X 0x8D0. |
| 0x0900 (wave 2) | Pool A | A#70 | (0x8D0, 0x48) | var 0 | Vehicle / Monster (biker) | Biker on motorcycle. Delay 0. |
| 0x0900 (wave 2) | Pool A | A#70 | (0x8D0, 0x28) | var 0 | Vehicle / Monster (biker), row E:0x28 | Biker carrying Radio item on defeat. Delay 0. |
| 0x0900 (wave 2) | Pool A | A#70 | (0x8D0, 0x38) | var 0 | Vehicle / Monster (biker) | Biker on motorcycle. Delay 0x78. |
| 0x0900 (wave 2) | Pool A | A#70 | (0x8D0, 0x18) | var 0 | Vehicle / Monster (biker) | Biker on motorcycle. Delay 0. |
| 0x0901 | Pool E | E#5 | (0xB00, 0x14) | var 0xC | Container (tire stack), row 0 | Four black tires, empty obstacle. |
| 0x0970 | Pool E | E#5 | (0xB70, 0x14) | var 0xC | Container (tire stack), row 0 | Four black tires, empty obstacle. |
| 0x09E0 | Pool E | E#5 | (0xC00, 0x14) | var 0xC | Container (tire stack), row 0 | Four black tires, empty obstacle. |
| 0x0A50 | Pool E | E#5 | (0xC80, 0x14) | var 0xC | Container (tire stack), row 0 | Four black tires, empty obstacle. |
| 0x0B00 | Pool E | E#5 | (0xD00, 0x14) | var 0xC | Container (tire stack), row 0 | Four black tires, empty obstacle. |
| 0x0B40 | Pool A | A#18 | (0xD40, 0x48) | var 0x200 | Vice T. / Walther mid-boss | Walther. |
| 0x0B40 | Pool A | A#18 | (0xD40, 0x30) | var 0x200 | Vice T. / Walther mid-boss | Walther. |
| 0x0B40 | Pool A | A#18 | (0xD40, 0x18) | var 0x200 | Vice T. / Walther, row E:0x28 | Walther carrying Radio item on defeat. |
| 0x0B80 | Pool E | E#5 | (0xD80, 0x14) | var 0xC | Container (tire stack), row 0 | Four black tires, empty obstacle. |
| 0x0C00 | Pool B | B#15 | (0, 0, 0) | var 3 | CTL | Wave controller (wave 3). Lock X 0xC00. Spawns 4 x poachers at X 0xBC0. |
| 0x1000 | Pool B | B#15 | (0, 0, 0) | var 4 | CTL | Wave controller (wave 4). Lock X 0x1000. Spawns 4 x Hammers (one has Radio row 0x28). |
| 0x1010 | Pool A | A#70 | (0x1200, 0x10) | var 0 | Vehicle / Monster (biker) | Biker crossing road. |
| 0x1200 | Pool B | B#95 | (0, 0, 0) | var 0 | ANI | Sky colour blender. Blends 3 palettes toward target set. |
| 0x1200 | Pool A | A#70 | (0x1400, 0x30) | var 0 | Vehicle / Monster (biker) | Biker crossing road. |
| 0x1400 | Pool A | A#70 | (0x1600, 0x50) | var 0 | Vehicle / Monster (biker) | Biker crossing road. |
| 0x1600 | Pool A | A#70 | (0x1800, 0x30) | var 0 | Vehicle / Monster (biker) | Biker crossing road. |
| 0x1800 | Pool A | A#70 | (0x1A00, 0x10) | var 0 | Vehicle / Monster (biker) | Biker crossing road. |
| 0x1900 | Pool B | B#15 | (0, 0, 0) | var 5 | CTL | Wave controller (wave 5). Lock X 0x1900. Empty wave, ends with GO. |
| 0x1D00 | Pool B | B#15 | (0, 0, 0) | var 6 | CTL | Wave controller (wave 6). Lock X 0x1D00. Empty wave, ends with GO. |
| 0x1D80 | Pool E | E#5 | (0x1F80, 0x10) | var 2 | Container (drum), row 0 | Orange steel oil drum, empty. |
| 0x1E00 | Pool E | E#5 | (0x2000, 0x20) | var 2 | Container (drum), row 0 | Orange steel oil drum, empty. |
| 0x1E80 | Pool E | E#5 | (0x2080, 0x30) | var 2 | Container (drum), row C:0x31 | Orange steel oil drum. Holds the Radio item (3,000 pts, summons Cadillac). |
| 0x1F00 | Pool E | E#5 | (0x2100, 0x40) | var 2 | Container (drum), row 0 | Orange steel oil drum, empty. |
| 0x1F80 | Pool E | E#5 | (0x2180, 0x40) | var 2 | Container (drum), row 0 | Orange steel oil drum, empty. |
| 0x2000 | Pool E | E#5 | (0x2200, 0x10) | var 2 | Container (drum), row 0 | Orange steel oil drum, empty. |
| 0x2080 | Pool E | E#5 | (0x2280, 0x38) | var 2 | Container (drum), row 0 | Orange steel oil drum, empty. |
| 0x2300 | Pool B | B#15 | (0, 0, 0) | var 7 | CTL | Wave controller (wave 7). Lock X 0x2300. Empty wave, ends with GO. |
| 0x2500 | Pool B | B#95 | (0, 0, 0) | var 2 | ANI | Sky colour blender. Palette step 2. |
| 0x2700 | Pool B | B#15 | (0, 0, 0) | var 8 | CTL | Wave controller (wave 8). Lock X 0x2700. Empty wave, ends with GO. |
| 0x2710 to 0x2AF0 | Pool A | A#65 | X 0x2910 to 0x2CF0 | var 0 | Hazard / Monster (rolling barrel) | 22 rolling/hopping barrels across lanes (Z 0x10 to 0x50). Hit breaks for +100. Record at trig 0x2790 (X 0x2990, Z 0x30) has row C:0x31 (Radio). |
| 0x2D00 | Pool A | A#31 | (0x2EC0, 0x40) | var 0 | Boss (Hogg) | Stage boss Hogg on motorcycle. Lock X 0x2D00 sets time 3:50. |
| 0x3100 | Pool B | B#95 | (0, 0, 0) | var 4 | ANI | Sky colour blender. Extended car chase sky shift. |
| 0x3300 | Pool B | B#95 | (0, 0, 0) | var 6 | ANI | Sky colour blender. Extended car chase sky shift. |
| 0x3700 | Pool B | B#95 | (0, 0, 0) | var 8 | ANI | Sky colour blender. Extended car chase sky shift. |
| 0x3A00 | Pool B | B#95 | (0, 0, 0) | var 0xA | ANI | Sky colour blender. Extended car chase sky shift. |
| 0x3D00 | Pool B | B#95 | (0, 0, 0) | var 0xC | ANI | Sky colour blender. Extended car chase sky shift. |
| Runtime | Pool B | B#88 | Dynamic | var 0, 2, 4 | CTL | Cadillac drive controller (3 objects spawned by radio routine 0x8BD24). |
| Runtime | Pool B | B#89 | Dynamic | var 0 | CTL | Car section helper object (spawned by B#88). |
| Runtime | Pool B | B#90 | Dynamic | var 0 | TXT | Car section caption banner ("Take the car", spawned by B#89). |
| Runtime | Pool B | B#96 | Dynamic | var 0 | CTL | Area music starter (spawned by area start hook 0x9236C). Plays track from table 0xA582. |

### 1.2 Scenery and interactive elements (What a player sees)

1. **Cadillac vehicle sequence (BS17 to BS23, T21, L01, L02, G04):**
   - The stage begins on foot. The hero is in a vast desert highway surrounded by mesas and cacti.
   - At trigger 0x80, the first wooden barrel sits at X 0x250, Z 0x20. Punching it drops the Radio item (row C:0x31, HUD text "RADIO 3000 pts").
   - When the radio is picked up while G.$50E == 0, the radio routine (0x8BD24) spawns three B#88 controllers. Sixteen frames later, the blue open-top Cadillac drives in from the left at 5 px/frame, brakes between frames 74 and 100, and halts at screen X 192.
   - An NPC teammate delivers the car (first character not in play, in order Jack, Mustapha, Hannah, Mess). The heroes leap aboard between frames 284 and 332.
   - At frame 364, G.$518 = 1 is set, switching camera mode to 8 (auto-scrolling chase at 11 px/frame). The stage clock is frozen.
   - Car seating: the player who took the radio drives; the second player takes the passenger seat; the third player rides in the trunk. Jack always takes the trunk if not the radio taker.
   - Car physics: stick inputs steer horizontal and vertical target speeds (vx +-768, vz +-256). Players inside are invulnerable to normal attacks. The car deals flat 20 damage to every enemy and container it hits, instantly smashing barrels.
   - The car has 120 HP. If destroyed by Hogg's grenades or hits, heroes jump out (48 frame arc, 120 frames invulnerability), the car wreck departs, camera drops back to mode 3, and combat resumes on foot.

2. **Road obstacles and containers (W36, p2.md, p3.md):**
   - Wooden barrels (E5 var 4): placed at X 0x250 (holding the first radio).
   - Small pails / buckets (E5 var 0): a row of six buckets at X 0x500 to 0x580 (Z 0x14 to 0x50). The last pail at X 0x580 holds the second radio.
   - Tire stacks (E5 var 0xC): six stacks of four black tires blocking lanes between X 0xB00 and X 0xD80. They act as solid obstacles; heroes cannot lift them, but punching them breaks them for +500 points.
   - Oil drums (E5 var 2): seven orange steel drums between X 0x1F80 and X 0x2280. The drum at X 0x2080 holds the third radio.
   - Rolling/hopping barrels (A#65): 22 barrels rolling down the highway between X 0x2910 and X 0x2CF0 before the boss. In the car, players ram through them. On foot, they hop toward the hero (vx 2, vy 3) and hurt for 10 raw damage, but a punch breaks them (+100 hit score). Barrel at X 0x2990 drops the fourth radio.

3. **Sky colour transitions (B#95, W35):**
   - Along the highway, seven B#95 objects trigger at X 0x1200, 0x2500, 0x3100, 0x3300, 0x3700, 0x3A00, and 0x3D00.
   - Each steps three 16-colour background palettes toward target sets, shifting the desert sky gradually from bright morning blue through dusty haze into twilight and sunset.

4. **Rowscroll perspective road (T24, stage SPEC 7.3):**
   - CPS1 rowscroll is enabled in Stage 2 only (`video_ctrl` bit 0 set).
   - Lines 128 to 159 of scroll2 have per-scanline horizontal offset entries in table 0x915800, creating an authentic receding perspective road illusion as the car speeds forward.

5. **Boss fight: Hogg (A#31, BS12 to BS16, L03, G04):**
   - Hogg enters on his motorcycle at X 0x2EC0 (Z 0x40) at trigger 0x2D00.
   - Phase 1 (in Cadillac): Hogg rides beside and ahead of the car, throwing stick bombs/grenades (class 42) into a 3x3 grid around the Cadillac. Ramming Hogg's motorcycle deals 20 damage per impact.
   - Phase 2 (on foot): if the car is destroyed, Hogg rides back and forth across the screen at 5.5 px/frame, attempting to run heroes down (attack 0x6A ram, 10 damage), lobbing grenades with circular landing shadows. When defeated, Hogg is flung upward, and the bike shatters into exploding debris.

---

## 2. Episode 4 (Internal Stage 3): Jack's Garage

Episode 4 has 3 areas:
- Area 0: Garage Exterior / Gate Arrival (camX 0 to 0x280, lock at 0x280, fade transition mode 1).
- Area 1: Garage Interior / Parking Lot (camX 0 to 0x880, locks at 0x190, 0x580, fade transition mode 1 at 0x880).
- Area 2: Factory / Hideout (camX 0 to 0x470, locks at 0x100, 0x2C0, stage end lock at 0x470 for Slice).

### 2.1 Trigger-order inventory of Area 0 (Garage Exterior / Gate Arrival)

List 1 (0xAD116), list 2 (0xAEE90), wave 0 (0xAF2DC):

| Trigger | Pool | Class / Object | Map X, Z, Y | Variant | Role / Item / Drop Row | Details and Notes |
|---|---|---|---|---|---|---|
| 0x0000 | Pool B | B#69 | (0, 0, 0) | var 0 | DBG | Debug cursor. Frees itself immediately. |
| 0x0000 | Pool B | B#31 | (0, 0, 0) | var 0 | CTL | Stage clock. |
| 0x0000 | Pool B | B#20 | (0, 0, 0) | var 0 | CTL | GO hint. |
| 0x0000 | Pool A | A#43 | (0x160, 0x40) | var 0 | Scene actor (orange leader) | Cutscene actor at gate. Clears G.$4FC at frame 888. |
| 0x0000 | Pool A | A#27 | (0x0EA, 0x4A) | var 0 | Scene actor (crouching thug) | Cutscene thug ducking at gate. |
| 0x0000 | Pool A | A#27 | (0x0BC, 0x28) | var 0x200 | Scene actor (crouching thug) | Cutscene thug ducking at gate. |
| 0x0000 | Pool A | A#27 | (0x112, 0x22) | var 0x400 | Scene actor (crouching thug) | Cutscene thug ducking at gate. |
| 0x0000 | Pool E | E#5 | (0x020, 0x50) | var 2 | Container (drum), row 0 | Orange oil drum, empty. |
| 0x0000 | Pool E | E#5 | (0x040, 0x30) | var 2 | Container (drum), row 0 | Orange oil drum, empty. |
| 0x0000 | Pool E | E#5 | (0x060, 0x10) | var 2 | Container (drum), row 0 | Orange oil drum, empty. |
| 0x0000 | Pool E | E#5 | (0x060, 0x48) | var 0xC | Container (tire stack), row 0 | Four black tires, empty. |
| 0x0000 | Pool E | E#5 | (0x080, 0x28) | var 0xC | Container (tire stack), row 0 | Four black tires, empty. |
| 0x0000 | Pool E | E#5 | (0x0A0, 0x08) | var 0xC | Container (tire stack), row 0 | Four black tires, empty. |
| 0x0000 | Pool E | E#5 | (0x0A0, 0x50) | var 2 | Container (drum), row 0 | Orange oil drum, empty. |
| 0x0000 | Pool E | E#5 | (0x0C0, 0x30) | var 2 | Container (drum), row 0 | Orange oil drum, empty. |
| 0x0000 | Pool E | E#5 | (0x0E0, 0x10) | var 2 | Container (drum), row 0 | Orange oil drum, empty. |
| 0x0000 | Pool E | E#0 | (0x14B, 0x52) | var 0x1C (+5C=2) | Placed weapon (stone, size 2) | Large throwable stone on ground. |
| 0x0100 | Pool E | E#5 | (0x2A8, 0x10) | var 0 | Container (pail), row C:0x02 | Small orange bucket: dime 22/32, dynamite 10/32. |
| 0x0100 | Pool E | E#5 | (0x2D0, 0x10) | var 2 | Container (drum), row C:0x11 | Orange steel drum: shotgun 24/32, Uzi 8/32. |
| 0x01A0 | Pool E | E#5 | (0x350, 0x40) | var 2 (+5C=2) | Explosive drum, row C:0x22 | Shimmering drum: explodes, drops dime 24/32, dynamite 8/32. |
| 0x01B8 | Pool E | E#5 | (0x390, 0x40) | var 2 | Container (drum), row C:0x03 | Orange steel drum: small food item. |
| 0x0210 | Pool E | E#5 | (0x3D0, 0x40) | var 2 | Container (drum), row C:0x13 | Orange steel drum (2P+ only): grenade 24/32, dynamite 8/32. |
| 0x0280 | Pool B | B#15 | (0, 0, 0) | var 0 | CTL | Wave controller (wave 0). Lock X 0x280. Spawns 4 x Driver/Gneiss. Ends with fade (type 2). |
| Runtime | Pool A | A#16 / A#17 | Dynamic | var 0 | Vehicle (Cadillac) | Opening car ride. Heroes ride in Cadillac (state 0x432) and jump out at gate. |
| Runtime | Pool B | B#96 | Dynamic | var 0 | CTL | Area music starter. |

### 2.2 Trigger-order inventory of Area 1 (Garage Interior / Parking Lot)

List 1 (0xAD14E), list 2 (0xAF020), wave 1 (0xAF33A), wave 2 (0xAF46A), wave 3 (0xAF498):

| Trigger | Pool | Class / Object | Map X, Z, Y | Variant | Role / Item / Drop Row | Details and Notes |
|---|---|---|---|---|---|---|
| 0x0000 | Pool B | B#69 | (0, 0, 0) | var 0 | DBG | Debug cursor. Frees itself immediately. |
| 0x0000 | Pool B | B#31 | (0, 0, 0) | var 0 | CTL | Stage clock. |
| 0x0000 | Pool B | B#20 | (0, 0, 0) | var 0 | CTL | GO hint. |
| 0x0000 | Pool B | B#66 | (0, 0, 0) | var 0x5C | TXT | Caption banner. "Jack's Garage" / Hideout interior text. |
| 0x0000 | Pool E | E#5 | (0x0B0, 0x58) | var 0xC | Container (tire stack), row C:0x28 | Stack of black tires: pizza (100% drop). Standing at Z 80/88 hits it. |
| 0x0000 | Pool E | E#5 | (0x0F0, 0x58) | var 0xC | Container (tire stack), row C:0x15 | Stack of black tires: dynamite 16/32, food 16/32. |
| 0x0000 | Pool E | E#5 | (0x120, 0x20) | var 2 | Container (drum), row C:0x34 | Orange drum (3P only): cake (100% drop). |
| 0x0060 | Pool E | E#5 | (0x23A, 0x68) | var 2 | Container (drum), row C:0x1C | Orange drum: dynamite 24/32, food 8/32. |
| 0x0060 | Pool E | E#5 | (0x26A, 0x68) | var 2 | Container (drum), row C:0x2A | Orange drum: gold dust 16/32, ammo/treasure 16/32. |
| 0x0190 | Pool B | B#15 | (0, 0, 0) | var 1 | CTL | Wave controller (wave 1). Lock X 0x190. Walther mid-boss arrives from floor vent. |
| 0x0190 (wave 1) | Pool A | A#18 | (0x320, 0x20) | var 0x200 | Vice T. / Walther mid-boss, row E:0x20 | Walther enters at floor vent. Carries revolver (100% drop). Delay 0. |
| 0x0190 (wave 1) | Pool A | A#18 | (0x180, 0x30) | var 0x200 | Vice T. / Walther, row E:0x20 | Second Walther (3P only). Carries revolver (100% drop). Delay 0x5A. |
| 0x0190 (wave 1) | Pool A | A#70 | (0x320, 0x10) | var 0 | Vehicle / Monster (biker), row E:0x05 | Biker on motorcycle in wave 1 group 4. Drops small food. |
| 0x0190 (wave 1) | Pool A | A#70 | (0x180, 0x50) | var 0x100 | Vehicle / Monster (biker), row E:0x05 | Biker on motorcycle in wave 1 group 4. Delay 0x28. |
| 0x0190 (wave 1) | Pool A | A#70 | (0x320, 0x30) | var 0x100 | Vehicle / Monster (biker), row E:0x05 | Biker on motorcycle in wave 1 group 4. Delay 0x5A. |
| 0x0200 | Pool E | E#9 | (0x3DF, 0x10) | var 0xA | Placed treasure (ring) | Ring (3,000 pts) lying on the garage floor. |
| 0x0360 | Pool A | A#70 | (0x500, 0x58) | var 0 | Vehicle / Monster (biker), row E:0x10 | Biker crossing parking lot. Drops grenade (24/32). |
| 0x0360 | Pool A | A#70 | (0x560, 0x30) | var 0x100 | Vehicle / Monster (biker), row E:0x07 | Biker crossing parking lot. Drops special ammo/treasure. |
| 0x0360 | Pool A | A#70 | (0x528, 0x08) | var 0 | Vehicle / Monster (biker), row E:0x0C | Biker crossing parking lot. Drops salad. |
| 0x04A0 | Pool B | B#0 | (0x590, 0x78, Y 0xB0) | var 2 | BRK | Generic breakable wall panel. Punched open by player. Rewrites scroll2 tiles. |
| 0x04A2 | Pool B | B#1 | (0x610, 0x80, Y 0) | var 0 | BRK | Breakable chain-link fence (compositions 326/327). Punched open to enter factory. |
| 0x0580 | Pool B | B#15 | (0, 0, 0) | var 2 | CTL | Wave controller (wave 2). Lock X 0x580. Sets time 3:50; ends with GO. |
| 0x06E0 | Pool E | E#5 | (0x8C0, 0x60) | var 0 | Container (pail), row C:0x2F | Small orange bucket: rod 16/32, rifle 8/32, M-16 8/32. |
| 0x0700 | Pool E | E#5 | (0x8F0, 0x60) | var 2 | Container (drum), row C:0x24 | Orange steel drum: coffee (100% drop). |
| 0x0760 | Pool B | B#0 | (0x900, 0x60, Y 0xE0) | var 0 | BRK | Generic breakable wall panel. Rewrites scroll2 tiles, effect C21 var 0xE. |
| 0x07C0 | Pool B | B#0 | (0x980, 0x78, Y 0xB0) | var 1 | BRK | Generic breakable wall panel. Rewrites scroll2 tiles, effect C21 var 0x10. |
| 0x0820 | Pool E | E#5 | (0x9C0, 0x60) | var 2 (+5C=2) | Explosive drum, row C:0x07 | Shimmering drum: explodes, drops special (ammo box if gun held, else treasure). |
| 0x0880 | Pool B | B#15 | (0, 0, 0) | var 3 | CTL | Wave controller (wave 3). Lock X 0x880. Punks and Blades. Ends with fade (type 2). |
| Runtime | Pool B | B#96 | Dynamic | var 0 | CTL | Area music starter. |

### 2.3 Trigger-order inventory of Area 2 (Factory / Hideout)

List 1 (0xAD1E8), list 2 (0xAF1A0), wave 4 (0xAF57C), wave 5 (0xAF584):

| Trigger | Pool | Class / Object | Map X, Z, Y | Variant | Role / Item / Drop Row | Details and Notes |
|---|---|---|---|---|---|---|
| 0x0000 | Pool B | B#69 | (0, 0, 0) | var 0 | DBG | Debug cursor. Frees itself immediately. |
| 0x0000 | Pool B | B#31 | (0, 0, 0) | var 0 | CTL | Stage clock. |
| 0x0000 | Pool B | B#20 | (0, 0, 0) | var 0 | CTL | GO hint. |
| 0x0000 | Pool B | B#59 | (0x220, 0, 0) | var 0 | ANI | Stage 4 palette cycling (table 0x862B6). |
| 0x0000 | Pool B | B#59 | (0x220, 0, 0) | var 1 | ANI | Stage 4 palette cycling (variant 1). |
| 0x0000 | Pool B | B#59 | (0x220, 0, 0) | var 2 | ANI | Stage 4 palette cycling (variant 2). |
| 0x0000 | Pool E | E#9 | (0x0E0, 0x38) | var 0xD | Placed treasure (dime) | Dime (1,000 pts) on floor (only if level <= 25). |
| 0x0000 | Pool E | E#9 | (0x12A, 0x38) | var 0xE | Placed treasure (1000$) | $1,000 bill on floor (2P+ only). |
| 0x0080 | Pool E | E#5 | (0x250, 0x55) | var 2 | Container (drum), row C:0x06 | Orange drum: barbecue 4/32, small food 28/32. |
| 0x0100 | Pool B | B#15 | (0, 0, 0) | var 4 | CTL | Wave controller (wave 4). Lock X 0x100. Empty wave, ends with GO. |
| 0x01B0 | Pool E | E#0 | (0x350, 0x50) | var 0xA | Placed weapon (shotgun) | Shotgun lying on floor (2P+ only). |
| 0x01B0 | Pool E | E#0 | (0x350, 0x18) | var 0 | Placed weapon (rifle) | Hunting rifle lying on floor. |
| 0x0200 | Pool E | E#0 | (0x3A0, 0x50) | var 8 | Placed weapon (M-16A1) | M-16A1 machine gun lying on floor (3P only). |
| 0x0250 | Pool E | E#0 | (0x3F0, 0x50) | var 8 | Placed weapon (M-16A1) | Extra M-16A1 lying on floor (only if level <= 8). |
| 0x02C0 | Pool B | B#15 | (0, 0, 0) | var 5 | CTL | Wave controller (wave 5). Lock X 0x2C0. Push line 0x78. Sets time 3:50. |
| 0x03D8 | Pool A | A#25 | (0x570, 0x20) | var 0 | Boss (Slice) | Stage boss Slice. Fast boomerang and shadow attacks. |
| 0x0470 | Camera | Lock 0x470 | - | - | Mode 6 | Stage end lock for boss battle. |
| Runtime | Pool B | B#22 | Dynamic | var 0 | CTL | Boss reinforcement controller (Slice helper waves). |
| Runtime | Pool B | B#96 | Dynamic | var 0 | CTL | Area music starter. |

### 2.4 Scenery and interactive elements (What a player sees)

1. **Garage gate arrival and barricade (Area 0, G05, L06, T17, T18):**
   - The episode opens with the heroes driving into the exterior of Jack's Garage aboard the Cadillac (player state 0x432).
   - In the opening cutscene, cutscene actors A#43 (orange gang leader) and three crouching thugs (A#27) guard the entrance gate. Slice orders the gate shut to lock Jack out.
   - The Cadillac crashes through the outer barricade of oil drums (E5 var 2) and tire stacks (E5 var 0xC), blowing away the defenders. Slice retreats inside.
   - Control starts after frame 888 (when G.$4FC clears, T18). The heroes dismount and advance on foot. A large stone (E0 var 0x1C) lies on the pavement.

2. **Floor vent where Walther appears (Area 1, G07, M04, N08, HU21):**
   - As the player walks past the garage entrance into Area 1, the camera locks at X 0x190 for wave 1.
   - Near the grating/floor vent in the garage floor (X 0x190 to 0x320, Z 0x20), Walther (A#18 var 0x200) enters as the first unit of wave 1 (delay 0, X 0x320, Z 0x20).
   - Walther fights with Vice's AI (revolver shots, rushing knockdowns) and always drops a revolver (row E:0x20, 32/32) on defeat. With 3 players, a second Walther appears at X 0x180 (Z 0x30).

3. **Tire stacks hiding pizza (Area 1, N08, p2.md 5.5, p4.md 8.5):**
   - At the beginning of Area 1 (X 0x0B0 and 0x0F0, back wall depth Z 0x58), two tire stacks stand in front of the wall.
   - The stack at X 0x0B0 holds a pizza (row C:0x28, heals 48 HP / 5,000 pts). The stack at X 0x0F0 holds dynamite or food (row C:0x15).
   - Walkable depth in Area 1 is 16 to 111 (screen band 16 to 111, D4c 3.2). A hero does not need a dive kick to reach the pizza: standing at depth Z 80 or 88, a standard punch hits and breaks the tires.

4. **Breakable chain-link fence and wall panels (Area 1, B#0, B#1, W31, G07, L06):**
   - At trigger 0x4A0 (X 0x590, Z 0x78), the player reaches breakable wall panel B#0 var 2.
   - At trigger 0x4A2 (X 0x610, Z 0x80), the breakable chain-link fence B#1 var 0 blocks the passage. This fence uses sprite composition 326 (whole) and composition 327 (broken). A single punch breaks it (HP 1), rewriting 13x6 scroll2 tile attributes and throwing dust effect C21 var 0xC.
   - Further along, two more breakable wall panels B#0 appear at trigger 0x760 (X 0x900, var 0) and trigger 0x7C0 (X 0x980, var 1), breaking with debris and tile updates.

5. **The overturned Cadillac (Area 1 / Area 2 background, M26, L06):**
   - In the background of Jack's overrun garage/factory, a battered, upside-down blue Cadillac sits ruined among the debris and machinery (part of the stage map scroll2 tiles, near the middle of Area 1).

6. **The weapon yard (Area 2, L06, WM25, p3.md 7.2):**
   - Inside the factory in Area 2, before the boss arena, a supply of firearms lies directly on the floor between X 0x350 and X 0x3F0:
     - X 0x350, Z 0x18: hunting rifle (always present).
     - X 0x350, Z 0x50: shotgun (2 or 3 players only).
     - X 0x3A0, Z 0x50: M-16A1 machine gun (3 players only).
     - X 0x3F0, Z 0x50: extra M-16A1 (only if level <= 8).
   - This weapon stash arms players for the upcoming Slice battle.

7. **Boss fight: Slice (Area 2, A#25, BS24 to BS26, G06, M10):**
   - Slice enters at X 0x570 (Z 0x20) at trigger 0x3D8.
   - Moves extremely fast (run speed 3 to 7 px/frame), generating green shadow afterimages (pool C class 27) every 7 frames.
   - Throws single curved boomerangs (class 19 var 0/2/4) and fans of 12 boomerangs (var 6). Boomerangs can be punched out of the air by heroes.
   - Boss reinforcement controller (B#22) summons waves of Punks, Blades, and Hammers.

---

## 3. Episode 5 (Internal Stage 4): Village of Flame

Episode 5 has 2 areas:
- Area 0: Meadow, Dinosaur Road, Mountain Slope (camX 0 to 0xA60, locks at 0x100, 0x3D0, 0x740, 0x940, sub-area switch at 0xA60).
- Area 1: River Bridge, Burning Village (camX 0xA60/0xCD0 to 0x11C0, lock at 0xCD0, diagonal descent at 0xD50, lock at 0xF20 for Morgan/Morgue, stage end lock at 0x11C0).

### 3.1 Trigger-order inventory of Area 0 (Meadow, Dinosaur Road, Mountain Slope)

List 1 (0xAD25A), list 2 (0xAF6E4), wave 0 (0xAFAD4) through wave 4 (0xAFCD8):

| Trigger | Pool | Class / Object | Map X, Z, Y | Variant | Role / Item / Drop Row | Details and Notes |
|---|---|---|---|---|---|---|
| 0x0000 | Pool B | B#69 | (0, 0, 0) | var 0 | DBG | Debug cursor. Frees itself immediately. |
| 0x0000 | Pool B | B#31 | (0, 0, 0) | var 0 | CTL | Stage clock. |
| 0x0000 | Pool B | B#20 | (0, 0, 0) | var 0 | CTL | GO hint. |
| 0x0000 | Pool E | E#9 | (0x010, 0x210) | var 5 | Placed treasure (gold dust) | Gold dust pouch (5,000 pts) at meadow start. |
| 0x0000 | Pool E | E#4 | (0x116, 0x240) | var 1 | Placed food (barbecue) | Whole roast barbecue (100 HP) on ground. |
| 0x0000 | Pool E | E#4 | (0x102, 0x238) | var 1 | Placed food (barbecue) | Second roast barbecue (100 HP) on ground. |
| 0x0000 | Pool E | E#9 | (0x0FA, 0x248) | var 4 | Placed treasure (egg) | Dinosaur egg (10,000 pts) sitting on ground. |
| 0x0000 | Pool E | E#0 | (0x0DD, 0x23E) | var 0x16 | Placed weapon (torch) | Burning torch on floor (glow overlay E#6). |
| 0x0000 | Pool E | E#0 | (0x0F4, 0x24C) | var 0x16 | Placed weapon (torch) | Second burning torch on floor. |
| 0x0000 | Pool E | E#0 | (0x108, 0x236) | var 0x16 | Placed weapon (torch) | Third burning torch on floor. |
| 0x0010 | Pool B | B#66 | (0, 0, 0) | var 0x2D | TXT | Caption banner. Story text line 1. |
| 0x0010 | Pool B | B#66 | (0, 0, 0) | var 0x57 | TXT | Caption banner. Story text line 2. |
| 0x0010 | Pool B | B#66 | (0, 0, 0) | var 0x58 | TXT | Caption banner. Story text line 3. |
| 0x0060 | Pool A | A#29 | (0x2C0, 0x258) | var 0x200 | Dinosaur (Rock Hopper) | Neutral green raptor. Leaves after 1200 frames if unprovoked. |
| 0x0080 | Pool E | E#5 | (0x2B0, 0x218) | var 0 | Container (pail), row C:0x15 | Small orange bucket (2P+ only): dynamite 16/32, food 16/32. |
| 0x0080 | Pool E | E#5 | (0x2D0, 0x258) | var 2 | Container (drum), row C:0x05 | Orange drum (3P only): donut 16/32, gum 16/32. |
| 0x00D0 | Pool A | A#82 | (0x180, 0x238) | var 0 | Dinosaur / Flyer (Zeke) | Pterodactyl. Swoops and grab-drops heroes. Dies in 1 hit. |
| 0x0100 | Pool A | A#38 | (0x2E0, 0x220) | var 0 | Dinosaur (Mack) | Triceratops charger from the right. Horns deal heavy damage. |
| 0x0100 | Pool B | B#15 | (0, 0, 0) | var 0 | CTL | Wave controller (wave 0). Lock X 0x100. Driver and Hammer. Ends with GO. |
| 0x0140 | Pool A | A#38 | (0x100, 0x240) | var 0 | Dinosaur (Mack) | Second Mack charger, charging from the left behind players. |
| 0x0180 | Pool E | E#9 | (0x204, 0x210) | var 2 | Placed treasure (diamond) | Diamond (10,000 pts) pops into picture 132 px from left edge. |
| 0x01C0 | Pool A | A#29 | (0x360, 0x218) | var 0x200 | Dinosaur (Rock Hopper) | Second neutral Rock Hopper. |
| 0x0240 | Pool A | A#24 | (0x450, 0x251) | var 0 | Dinosaur (Shivat) | Sleeping green T. rex with Z marks. Immune to player hits. |
| 0x03C8 | Pool A | A#79 | (0x3A0, 0x228) | var 0x400 | Poacher / Hunter | Dinosaur hunter. Walks up to sleeping Shivat and pokes it 3 times. |
| 0x03D0 | Pool B | B#15 | (0, 0, 0) | var 1 | CTL | Wave controller (wave 1). Lock X 0x3D0. Spawns 3 more hunters (A#79). |
| 0x0600 | Pool E | E#5 | (0x790, 0x25C) | var 4 | Container (barrel), row C:0x21 | Wooden barrel: bazooka (100% drop). |
| 0x0660 | Pool E | E#5 | (0x7E8, 0x25C) | var 4 (+5C=2) | Explosive barrel, row C:0x03 | Shimmering barrel: explodes, drops small food. |
| 0x0670 | Pool B | B#48 | (0x810, 0x295) | var 0 | EVT | Scenery event stub. Waits 128 frames and does nothing. |
| 0x06B0 | Pool B | B#66 | (0, 0, 0) | var 0x3D | TXT | Caption banner. Mountain slope / cliff text. |
| 0x0700 | Pool A | A#66 | (0x780, 0x260) | var 0 | Hazard / Monster (rolling rock) | Boulder rolls down cliff toward screen. HP 8; crumbles into stones. |
| 0x0720 | Pool A | A#66 | (0x800, 0x260) | var 0 | Hazard / Monster (rolling rock) | Second falling boulder. |
| 0x0730 | Pool A | A#66 | (0x758, 0x260) | var 0 | Hazard / Monster (rolling rock) | Third falling boulder. |
| 0x0740 | Pool B | B#15 | (0, 0, 0) | var 2 | CTL | Wave controller (wave 2). Lock X 0x740. Sets time 3:50. Spawns 2 rocks. |
| 0x0880 | Pool E | E#5 | (0xA20, 0x258) | var 0 | Container (pail), row C:0x23 | Small orange bucket: grenade 16/32, treasure 16/32. |
| 0x0880 | Pool E | E#5 | (0xA50, 0x258) | var 2 | Container (drum), row C:0x2B | Orange steel drum: gun 24/32, shotgun 8/32. |
| 0x0880 | Pool E | E#5 | (0xA80, 0x258) | var 2 | Container (drum), row C:0x13 | Orange drum (3P only): grenade 24/32, dynamite 8/32. |
| 0x0940 | Pool B | B#15 | (0, 0, 0) | var 3 | CTL | Wave controller (wave 3). Lock X 0x940. Spawns rock A#66 and 2 Bludges A#6. |
| 0x0940 (wave 3) | Pool A | A#66 | (0x980, 0x260) | var 0 | Hazard / Monster (rolling rock) | Rolling boulder in wave 3. Delay 0x96. |
| 0x0940 (wave 3) | Pool A | A#6 | (0x9E0, 0x1E0) | var 6 | Monster (Bludge) | Dino-human hybrid sub-boss. Acid spit, tongue grab, eats items. |
| 0x0940 (wave 3) | Pool A | A#6 | (0xA50, 0x1E0) | var 6 | Monster (Bludge) | Second Bludge (3P only). |
| 0x0A60 | Pool B | B#15 | (0, 0, 0) | var 4 | CTL | Wave controller (wave 4). Lock X 0xA60 before bridge. 16 enemies. Ends with type 2. |
| 0x0A60 (wave 4) | Pool A | A#18 | (0xA40, 0x228) | var 0x200 | Vice T. / Walther mid-boss, row E:0x20 | Walther mid-boss before bridge. Drops revolver (100%). |
| 0x0A60 (wave 4) | Pool A | A#18 | (0xC40, 0x220) | var 0x200 | Vice T. / Walther, row E:0x20 | Second Walther (2P+ only). Drops revolver (100%). |
| Runtime | Camera | Lock 0xA60 | camY 0x200 | Mode 8 | Sub-area transition | Area 0 ends; switches to Area 1 (bridge) with camY set to 0x200. |
| Runtime | Pool B | B#96 | Dynamic | var 0 | CTL | Area music starter. |

### 3.2 Trigger-order inventory of Area 1 (River Bridge, Burning Village)

List 1 (0xAD2D8), list 2 (0xAF974), wave 5 (0xAFE38), wave 6 (0xAFEE2), wave 7 (0xAFFF4):

| Trigger | Pool | Class / Object | Map X, Z, Y | Variant | Role / Item / Drop Row | Details and Notes |
|---|---|---|---|---|---|---|
| 0x0A50 | Pool E | E#5 | (0xBFA, 0x240) | var 4 | Container (barrel), row C:0x0B | Barrel 1 of 8: barbecue (100 HP, 10,000 pts). |
| 0x0A80 | Pool E | E#5 | (0xC20, 0x220) | var 4 | Container (barrel), row C:0x18 | Barrel 2 of 8: lobster (64 HP). |
| 0x0A90 | Pool E | E#5 | (0xC3A, 0x240) | var 4 | Container (barrel), row C:0x0C | Barrel 3 of 8: salad (32 HP). |
| 0x0AC0 | Pool E | E#5 | (0xC60, 0x220) | var 4 | Container (barrel), row C:0x02 | Barrel 4 of 8: dime 22/32, dynamite 10/32. |
| 0x0AD0 | Pool E | E#5 | (0xC7A, 0x240) | var 4 | Container (barrel), row C:0x06 | Barrel 5 of 8: barbecue 4/32, small food 28/32. |
| 0x0B00 | Pool B | B#5 | (0x700, 0, Y 0xE0) | var 0 | ANI | Far layer animation (scroll3 0x913F00, 4 pictures x 16 frames). |
| 0x0B00 | Pool E | E#5 | (0xCA0, 0x220) | var 4 | Container (barrel), row C:0x12 | Barrel 6 of 8: rod 24/32, club 8/32. |
| 0x0B10 | Pool E | E#5 | (0xCBA, 0x240) | var 4 | Container (barrel), row C:0x02 | Barrel 7 of 8: dime 22/32, dynamite 10/32. |
| 0x0B40 | Pool E | E#5 | (0xCE0, 0x220) | var 4 | Container (barrel), row C:0x07 | Barrel 8 of 8: special (ammo box if gun held, else treasure). |
| 0x0B80 | Pool B | B#62 | (0xE00, 0, Y 0x200) | var 0 | ANI | Stage 5 palette cycling with sound 0x8E (fire crackle / roar). |
| 0x0B90 | Pool B | B#10 | (0xD20, 0, Y 0x200) | var 0 | ANI | Tile animation (keyed to tile clock G.$7AE, 63 writes). |
| 0x0B90 | Pool B | B#13 | (0xD20, 0, Y 0x200) | var 4 | EVT | Burning huts tile controller (block 1). Rewrites village scroll2 tiles. |
| 0x0C00 | Pool B | B#66 | (0, 0, 0) | var 0x34 | TXT | Caption banner. Burning village text. |
| 0x0C80 | Pool B | B#13 | (0xE20, 0, Y 0x200) | var 6 | EVT | Burning huts tile controller (block 2). |
| 0x0C80 | Pool B | B#13 | (0xE20, 0, Y 0x200) | var 0 | EVT | Burning huts tile controller (block 3). |
| 0x0CD0 | Pool B | B#15 | (0, 0, 0) | var 5 | CTL | Wave controller (wave 5). Lock X 0xCD0 at bridge. Sets time 3:50. |
| 0x0D80 | Pool B | B#66 | (0, 0, 0) | var 0x5F | TXT | Caption banner. Chief / village speech. |
| 0x0D80 | Pool B | B#10 | (0xF20, 0, Y 0x200) | var 4 | ANI | Tile animation (keyed to tile clock G.$7AE). |
| 0x0D80 | Pool B | B#13 | (0xF20, 0, Y 0x200) | var 8 | EVT | Burning huts tile controller (block 4). |
| 0x0D80 | Pool E | E#5 | (0xF68, 0x160) | var 0 | Container (pail / bucket), row C:0x07 | Water bucket 1: special (ammo box if gun held, else treasure). |
| 0x0E00 | Pool E | E#0 | (0xF70, 0x160) | var 0x16 | Placed weapon (torch) | Torch on village street (2P+ only). |
| 0x0E00 | Pool E | E#0 | (0xFA0, 0x110) | var 0x16 | Placed weapon (torch) | Torch on village street. |
| 0x0E00 | Pool E | E#5 | (0xF90, 0x160) | var 0 | Container (pail / bucket), row C:0x1F | Water bucket 2: ammonite fossil (5,000 pts). |
| 0x0E00 | Pool E | E#5 | (0xFB8, 0x160) | var 0 | Container (pail / bucket), row C:0x20 | Water bucket 3: cake (32 HP, only if level <= 16). |
| 0x0E00 | Pool E | E#5 | (0xFB8, 0x160) | var 0 | Container (pail / bucket), row C:0x24 | Water bucket 3: coffee (16 HP, only if level >= 17). |
| 0x0E80 | Pool B | B#13 | (0x1020, 0, Y 0x100) | var 0xA | EVT | Burning huts tile controller (block 5). |
| 0x0F00 | Pool B | B#6 | (0x800, 0, Y 0xA0) | var 0 | ANI | Scroll2 animation (pale blue/white steam/water block). |
| 0x0F00 | Pool B | B#6 | (0x900, 0, Y 0xA0) | var 4 | ANI | Scroll2 animation (4+4 column block). |
| 0x0F00 | Pool B | B#6 | (0xA00, 0, Y 0xA0) | var 8 | ANI | Scroll2 animation. |
| 0x0F20 | Pool B | B#15 | (0, 0, 0) | var 6 | CTL | Wave controller (wave 6). Lock X 0xF20. Boss approach. Push 0x78. Sets time 3:50. |
| 0x0F20 (wave 6) | Pool B | B#49 | (0x1000, 0x170) | var 0 | EVT | Wall burst patch. Delay 0. Spawns effect 0x16 and two B#50 dialogue boxes. |
| 0x0F20 (wave 6) | Pool A | A#3 | (0x10C0, 0x138) | var 0 | Sub-boss (Lash T.) | Sub-boss with spiked chain mace. |
| 0x0F80 | Pool B | B#13 | (0x1120, 0, Y 0x100) | var 0xC | EVT | Burning huts tile controller (block 6). |
| 0x1080 | Pool B | B#13 | (0x1220, 0, Y 0x100) | var 0xE | EVT | Burning huts tile controller (block 7). |
| 0x1120 | Pool B | B#70 | (0x1270, 0x40) | var 0 | FX | Animated scenery object with floor test (frames 0x89736). |
| 0x1120 | Pool A | A#35 | (0x1380, 0x150) | var 0 | Boss (Morgan) | Stage boss Morgan. Rewrites class to 36, shoots chief A#45 with Uzi. |
| 0x1120 | Pool A | A#45 | (0x1360, 0x100) | var 0 | Cutscene actor (old villager) | Village chief. Speaks, then gets shot by Morgan before naming villain. |
| 0x1180 | Pool B | B#50 | (0, 0, 0) | var 3 | TXT | Boss dialogue box with Morgan portrait. |
| 0x1180 | Pool B | B#13 | (0x1320, 0, Y 0x100) | var 0x10 | EVT | Burning huts tile controller (block 8). |
| 0x11C0 | Camera | Lock 0x11C0 | - | - | Mode 6 | Stage end lock for Morgue battle. |
| Runtime | Pool B | B#96 | Dynamic | var 0 | CTL | Area music starter (spawned by sub-area switch). |

### 3.3 Scenery and interactive elements (What a player sees)

1. **Torches, barbecues, and the dinosaur egg (Area 0 start, N09, p4.md 8.5):**
   - The meadow starts at night. Three burning torches (E0 var 0x16) are planted on the ground at X 0xDD, 0xF4, and 0x108. Torches have a flickering fire overlay (pool E class 6) and give burn knockdown (reaction 0xA) when swung.
   - Sitting right beside the torches are two full roast barbecues (E4 var 1 at X 0x102 and 0x116, healing 100 HP each) and the unique dinosaur egg (E9 var 4 at X 0xFA, worth 10,000 pts).
   - Because pickup priority takes food before treasure, standing near the egg and pressing Attack will pick up the barbecue first if hurt, matching player warnings from walkthroughs.
   - Gold dust (E9 var 5, 5,000 pts) lies on the ground at the far left (X 0x10).

2. **Living and sleeping dinosaurs (Area 0, DS01 to DS05, DS10, HU01 to HU03, G08, L07):**
   - Two neutral Rock Hoppers (A#29 at X 0x2C0 and 0x360): green raptors that wander peacefully unless attacked. If hit, they turn orange and attack heroes and enemies.
   - Zeke (A#82 at X 0x180): pterodactyl flyer that swoops in. Dies in one jump kick or 360 special.
   - Two charging Macks (A#38 at X 0x2E0 and 0x100): triceratops chargers that stomp through the arena. The second charges from the left behind players.
   - Sleeping Shivat (A#24 at X 0x450, Z 0x251): a green T. rex asleep on the ground with floating "Z" particles. Player attacks cannot harm it.
   - Dinosaur Hunter (A#79 at X 0x3A0, Z 0x228, plus wave 1): poachers with clubs. The hunter walks up to Shivat and delivers three swings. On the third swing, Shivat flashes for 88 frames, awakens enraged, and rampages against both poachers and players.
   - Placed diamond (E9 var 2 at X 0x204): pops into view at trigger 0x180 near the rock where Mack and Shivat appear (10,000 pts).

3. **Falling and rolling cliff rocks (Area 0, A#66, G08, HU14, p4.md 8.4):**
   - Along the mountain slope (triggers 0x700 to 0x740, and waves 2 and 3), large boulders (A#66) roll down the hill toward the camera.
   - Rocks have HP 8 and hurt heroes and enemies for 10 raw damage. Punching or kicking a boulder breaks it (+100 hit score).
   - When a boulder lands and shatters, it creates debris pieces that have an approximate 2-in-16 chance to turn into throwable stone weapons (weapon type 14, 28 damage), up to 3 stones active.

4. **Walther mid-boss battle before the bridge (Area 0 lock 0xA60, HU21, L08):**
   - At the end of Area 0, camera locks at X 0xA60. Wave 4 releases 16 enemies, led by Walther (A#18 var 0x200) wielding his revolver (drops revolver 100%), alongside Blade knife throwers, poachers, and Punks.
   - Once cleared, camera mode 8 initiates the sub-area switch to Area 1 (bridge) with camY set to 0x200.

5. **The river bridge crossing (Area 1, camX 0xCD0 to 0xD50, L08):**
   - The players arrive at the wooden rope bridge over the river.
   - Lock 0xCD0 holds wave 5. The camera descends diagonally at X 0xD50 (camY drops from 0x200 down to 0x100) leading directly into the burning village.
   - Along the approach, eight wooden barrels (E5 var 4) stand in a straight line between X 0xBFA and 0xCE0, containing food (barbecue, lobster, salad), weapons (rod/club, special ammo), and treasures.

6. **Burning huts and village ruins (Area 1, B#13, B#62, B#10, B#6, L08):**
   - As players enter the village, the huts and buildings are engulfed in flames.
   - Multi-block tile controller B#13 is instantiated eight times (triggers 0xB90 through 0x1180, variants 0 to 0x10) across X 0xD20 to 0x1320. Each rewrites large blocks of scroll2 tiles to show collapsed thatched roofs and blazing walls.
   - Palette cycler B#62 (trigger 0xB80, X 0xE00) continuously shifts scroll2 fire palettes while playing sound 0x8E (roaring fire).
   - Tile animation B#6 (trigger 0xF00, X 0x800 to 0xA00) displays pale blue/white billowing steam/smoke blocks.
   - Tile animation B#10 (triggers 0xB90 and 0xD80) cycles additional structural fire tiles keyed to global tile clock G.$7AE.

7. **Water buckets / pails (Area 1, E5 var 0, L08, p4.md 8.5):**
   - In the village streets near X 0xF68 to 0xFB8, three small buckets / pails (E5 var 0) sit between two burning torches (E0 var 0x16).
   - Bucket 1 (X 0xF68): special ammo box (row C:0x07) to load guns for the boss.
   - Bucket 2 (X 0xF90): ammonite fossil treasure (row C:0x1F, 5,000 pts).
   - Bucket 3 (X 0xFB8): health item (cake for level <= 16, coffee for level >= 17).

8. **Wall burst event (Area 1 lock 0xF20, B#49, D3e part 1 line 211):**
   - At wave 6, B#49 executes at delay 0 (X 0x1000, Z 0x170). A wall patch blows open with debris and smoke effect C21 var 0x16, spawning two B#50 dialogue boxes while enemies burst through, led by sub-boss Lash T. (A#3) wielding his spiked mace.

9. **Boss cutscene and battle: Morgan and Morgue (Area 1, A#35, A#45, B#50, G10, L08, M11):**
   - At trigger 0x1120, the camera frames the village chief (A#45, the old villager in yellow robe). The chief begins to reveal that someone burned the village because they refused to cooperate with his biological experiments.
   - Morgan (A#35, poacher boss in green vest) steps forward and shoots the chief dead with his Uzi. Dialogue box B#50 var 3 displays Morgan's portrait and lines.
   - First phase: Morgan fights with fast Uzi bursts, thrown knives, grenades, and rock kicks.
   - Second phase: Upon defeat, Morgan speaks of Dr. Fessenden and mutates into Morgue, a hulking mutant with a domed dinosaur head. Morgue charges head-first, slaps players, and drops Morgan's Uzi, which players can pick up and use against him.

---

## 4. Cross-Area Scenery and Mechanical Checklist

Summary of every specific item requested in the brief, cross-checked against groups J, L, M, N:

| Item | Where Seen | Class / IDs | Description and Mechanical Behavior |
|---|---|---|---|
| **Cadillac (playable)** | Episode 3 Area 0 | B#88, B#89, B#90, A#16, A#17 | Summoned by Radio item (row C:0x31 / E:0x28). Carries up to 3 players. 120 HP. Steered with stick. Smashes enemies (20 dmg) and barrels. Fights Hogg phase 1. Destructible. |
| **Cadillac (arrival)** | Episode 4 Area 0 | A#16, A#17, hero state 0x432 | Heroes arrive riding Cadillac. Smashes entrance barricade of drums/tires and gate defenders. G.$4FC clears at frame 888. |
| **Overturned Cadillac** | Episode 4 Area 1 / Area 2 | Stage map (scroll2 tiles) | Battered, upside-down blue Cadillac lying in background wreckage of Jack's garage. Static background art (M26, L06). |
| **Gates and barricades** | Episode 4 Area 0 | E#5 drums/tires, A#43, A#27 | Exterior gate barricade made of drums and tire stacks, guarded by Slice's men. Rammed during intro. |
| **Fences (breakable)** | Episode 4 Area 1 | B#1 var 0 (compositions 326/327) | Chain-link fence at X 0x610, Z 0x80 (trigger 0x4A2). HP 1. Punching breaks it, rewrites 13x6 scroll2 tiles, spawns dust C21 var 0xC. |
| **Wall panels (breakable)**| Episode 4 Area 1 | B#0 vars 0, 1, 2 | Breakable interior wall panels at X 0x590, 0x900, 0x980. Broken with punch; rewrites scroll2 tiles. |
| **Falling / rolling rocks** | Episode 5 Area 0 | A#66 (triggers 0x700, 0x720, 0x730, waves 2, 3) | Boulders rolling down mountain cliff. HP 8, 10 raw damage to heroes/enemies. Break into throwable stone weapons (E#0 var 0x1C). |
| **Burning huts** | Episode 5 Area 1 | B#13 (8 records), B#62, B#10, B#6 | Destroyed village huts. B#13 rewrites scroll2 tile blocks. B#62 cycles fire palette with roar sound 0x8E. B#6 adds steam/smoke blocks. |
| **River bridge** | Episode 5 Area 0 to 1 | Camera mode 8, lock 0xA60 / 0xCD0 | Wooden rope bridge crossing from Area 0 into burning village. Sub-area camera shift (camY 0x200 down to 0x100 at 0xD50). 8 barrels in row. |
| **Torches** | Episode 5 Area 0 & Area 1 | E#0 var 0x16, overlay E#6 | Placed burning torches on ground (3 at Ep 5 start, 2 in village). Glow overlay flickers at Z +-2. Swung as weapon (burn reaction 0xA). |
| **Buckets / pails** | Episodes 3, 4, 5 | E#5 var 0 | Small orange pails. Ep 3: X 0x500-0x580 (holds Radio). Ep 4: X 0x2A8, 0x8C0. Ep 5: X 0x2B0, 0x880; Area 1: X 0xF68, 0xF90, 0xFB8 (water buckets before boss). |
| **Tires (tire stacks)** | Episodes 3, 4 | E#5 var 0xC | Stacks of 4 black tires. Solid obstacles, cannot be lifted. Ep 3: X 0xB00-0xD80 (roadblocks). Ep 4: Area 0 X 0x60-0xA0; Area 1 X 0x0B0, 0x0F0 (hides pizza at back wall Z 0x58). |
| **Floor vent (Walther)** | Episode 4 Area 1 | Lock 0x190, A#18 var 0x200 | Floor vent in garage pavement at X 0x190 to 0x320, Z 0x20. Walther emerges as first unit of wave 1 with 100% revolver drop (row E:0x20). |

---

## 5. Probe Tool Sweep and Inspection Checklist

The walker will visit each listed position using the probe tool (`PR_INPUT=file re/harness/probe/probe.sh NAME`) with god mode and kill-all flags. Frame numbers and coordinates are defined per segment.

| Ep | Area | Cam X | Command / Setup | Watched Variables & Slots | What to Check & Verify |
|---|---|---|---|---|---|
| 3 | 0 | 0x000 | `jump 3 0 0x0`, `frames 300` | B:69, B:31, B:20, B:66, E:5 | Verify debug object B#69 frees itself, clock B#31 in slot 15, hint B#20, caption B#66 var 0x5B on scroll1. Verify barrel E#5 at X 0x250 (trigger 0x80). |
| 3 | 0 | 0x080 | `jump 3 0 0x70`, walk to 0x80, punch X 0x250 | E:5 st x y z, E:9, score | Punch barrel at X 0x250, verify Radio item drops (row 0x31), pickup displays "RADIO 3000 pts" in HUD. |
| 3 | 0 | 0x080 | Pickup radio at X 0x250, `frames 500` | B:88, B:89, B:90, A:16, A:17, G.$518 | Verify B#88 spawns Cadillac (A#16/17), boarding jump, G.$518 = 1 at frame 364, camera mode 8 engages at 11 px/frame. |
| 3 | 0 | 0x300 | `jump 3 0 0x2F0`, on foot, `frames 120` | B:15 st var, cam_mode, E:5 | Verify camera lock 0x300 (mode 2). Wave 0 ends with GO. Verify pails at X 0x500 to 0x580; punch pail at X 0x580 to verify Radio drop. |
| 3 | 0 | 0x900 | `jump 3 0 0x8F0`, on foot | B:15 st, A:70 x y z, E:5 | Verify lock 0x900 (mode 2). Wave 2 spawns 4 x A#70 bikers at X 0x8D0. Biker at Z 0x28 carries Radio row E:0x28. Verify tire stacks at X 0xB00-0xD80. |
| 3 | 0 | 0x1200 | `jump 3 0 0x11E0`, `frames 100` | B:95 st var, peek 0x914800 16 | Verify sky colour blender B#95 var 0 triggers. Palettes shift toward target RGB values. |
| 3 | 0 | 0x1D00 | `jump 3 0 0x1CE0` | B:15 st, E:5 var x y z | Verify lock 0x1D00. Verify oil drums at X 0x1F80 to 0x2280. Punch drum at X 0x2080 to verify Radio drop. |
| 3 | 0 | 0x2500 | `jump 3 0 0x24E0` | B:95 st var | Verify sky colour blender B#95 var 2 palette shift. |
| 3 | 0 | 0x2700 | `jump 3 0 0x26E0` | B:15 st, A:65 x y z | Verify lock 0x2700. Verify 22 x A#65 rolling barrels spawn between X 0x2910 and 0x2CF0. Barrel at X 0x2990 drops Radio. |
| 3 | 0 | 0x2D00 | `jump 3 0 0x2CE0`, `frames 400` | A:31 st x y z, B:15, time | Verify boss Hogg (A#31) enters at X 0x2EC0 on motorcycle. If on foot, verify lock sets time 3:50. If in car, test ramming bike (20 dmg) and grenade damage to car. |
| 3 | 0 | 0x3100+ | Car drive test past 0x3100 | B:95 vars 4, 6, 8, 0xA, 0xC | Verify extended sky blenders along car chase road loop. |
| 4 | 0 | 0x000 | `jump 4 0 0x0 at=start`, `frames 950` | A:43, A:27, A:16, G.$4FC | Verify Cadillac intro arrival. Cutscene actors A#43 and 3 x A#27 at gate. Control granted after G.$4FC clears at gf 888. Large stone E#0 at X 0x14B. |
| 4 | 0 | 0x280 | `jump 4 0 0x270`, walk to 0x280 | B:15 var 0, cam_mode | Verify lock 0x280. Wave 0 Driver/Gneiss. Area ends with fade (mode 1) into Area 1. |
| 4 | 1 | 0x000 | `jump 4 1 0x0`, `frames 100` | B:66 var 0x5C, E:5 var 0xC | Verify caption B#66 var 0x5C on scroll1. Verify tire stacks at X 0x0B0 and 0x0F0 (Z 0x58). Test punch at hero Z 80/88: verify pizza drops from X 0x0B0 stack. |
| 4 | 1 | 0x190 | `jump 4 1 0x180`, walk to 0x190 | B:15 var 1, A:18 x y z, A:70 | Verify lock 0x190. Walther (A#18 var 0x200) enters from floor vent at X 0x320, Z 0x20. Defeat Walther: verify 100% revolver drop (row 0x20). Verify wave 1 bikers. |
| 4 | 1 | 0x4A0 | `jump 4 1 0x480`, walk to 0x4A0 | B:0 st x y z, B:1 | Verify breakable panel B#0 var 2 at X 0x590. Punch it: verify scroll2 tile rewrite and debris. |
| 4 | 1 | 0x4A2 | Stand at X 0x5F0, Z 0x80, punch | B:1 st x y z, peek VRAM | Verify chain-link fence B#1 at X 0x610. Punch it (HP 1): verify sprite switches to composition 327, 13x6 tile attributes rewrite, dust C21 var 0xC. |
| 4 | 1 | 0x580 | `jump 4 1 0x570`, walk to 0x580 | B:15 var 2, time, snaps | Verify lock 0x580 sets time 3:50 and gives GO. Inspect background scenery for the overturned blue Cadillac. |
| 4 | 1 | 0x760 | Walk past 0x750 | B:0 vars 0 and 1 | Verify breakable panels B#0 at X 0x900 (var 0) and X 0x980 (var 1). Punch both: verify tile rewrites. |
| 4 | 1 | 0x880 | `jump 4 1 0x870` | B:15 var 3, cam_mode | Verify lock 0x880. Wave 3 Blades. Area ends with fade (mode 1) into Area 2. |
| 4 | 2 | 0x000 | `jump 4 2 0x0`, `frames 120` | B:59 vars 0, 1, 2, E:9 | Verify three palette cyclers B#59 active at X 0x220. Verify placed dime at X 0x0E0 (level <= 25) and $1,000 at X 0x12A (2P+). |
| 4 | 2 | 0x1B0 | `jump 4 2 0x180`, walk to 0x200 | E:0 vars 0, 0xA, 8 | Verify weapon yard: rifle at X 0x350 Z 0x18, shotgun at X 0x350 Z 0x50 (2P+), M-16 at X 0x3A0 Z 0x50 (3P), extra M-16 at X 0x3F0 (level <= 8). |
| 4 | 2 | 0x2C0 | `jump 4 2 0x2B0` | B:15 var 5, push_line | Verify lock 0x2C0. Wave 5 enemies. Push threshold tightened to 0x78. Sets time 3:50. |
| 4 | 2 | 0x3D8 | `jump 4 2 0x3C0`, walk to 0x3E0 | A:25 st x y z, C:27, B:22 | Verify boss Slice (A#25) enters at X 0x570. Verify shadow afterimages (C#27) every 7 frames. Verify boomerang projectiles can be punched down. |
| 5 | 0 | 0x000 | `jump 5 0 0x0 at=start`, `frames 200` | E:9, E:4, E:0, B:66 | Verify opening meadow items: gold dust at X 0x10, 2 x barbecues at X 0x102 and 0x116, egg at X 0xFA, 3 x burning torches at X 0xDD, 0xF4, 0x108. Captions B#66. |
| 5 | 0 | 0x060 | `jump 5 0 0x50` | A:29, A:82, A:38 | Verify neutral Rock Hopper at X 0x2C0. Verify Zeke at X 0x180 swoops in. Verify Mack charging at X 0x2E0 and second Mack from behind at X 0x100. |
| 5 | 0 | 0x180 | `jump 5 0 0x170` | E:9 var 2, A:24 | Verify diamond at X 0x204 pops into view at trigger 0x180. Verify sleeping Shivat at X 0x450 (Z marks, immune to player hits). |
| 5 | 0 | 0x3C8 | `jump 5 0 0x3B0` | A:79, A:24 st | Verify hunter A#79 approaches Shivat and strikes 3 times. Verify Shivat flashes for 88 frames and wakes up aggressive against poachers and heroes. |
| 5 | 0 | 0x670 | `jump 5 0 0x660` | B:48, B:66 var 0x3D, E:5 | Verify B#48 stub at X 0x810. Verify bazooka barrel at X 0x790 and explosive barrel at X 0x7E8. Caption B#66 at 0x6B0. |
| 5 | 0 | 0x700 | `jump 5 0 0x6F0`, walk past 0x740 | A:66 st x y z, B:15 var 2 | Verify rolling boulders A#66 at X 0x780, 0x800, 0x758 roll toward camera. Test punch boulder: verify it breaks and check stone weapon drop. Lock 0x740 sets time 3:50. |
| 5 | 0 | 0x940 | `jump 5 0 0x930` | B:15 var 3, A:6, A:66 | Verify lock 0x940. Verify Bludge A#6 enters (acid spit, tongue grab, eats items). Boulder A#66 rolls in. |
| 5 | 0 | 0xA60 | `jump 5 0 0xA50` | B:15 var 4, A:18, cam_mode | Verify lock 0xA60. Wave 4 Walther mid-boss at X 0xA40 with 100% revolver drop. Clear wave: verify sub-area switch (mode 8) to Area 1 with camY = 0x200. |
| 5 | 1 | 0xCD0 | `jump 5 1 0xCC0` | B:15 var 5, E:5, B:5, B:62 | Verify lock 0xCD0 on bridge. Sets time 3:50. Verify far layer animation B#5 at X 0x700. Verify palette cycler B#62 at X 0xE00 with sound 0x8E. |
| 5 | 1 | 0xCD0 | Walk past barrels at X 0xBFA to 0xCE0 | E:5 vars 4, drops | Verify 8 barrels in a row: barbecue at 0xBFA, lobster at 0xC20, salad at 0xC3A, small food at 0xC7A, rod/club at 0xCA0, special ammo at 0xCE0. |
| 5 | 1 | 0xD50 | Walk past 0xD50 | camY, B:10, B:13 | Verify diagonal camera descent (camY drops from 0x200 to 0x100). Verify burning huts tile controllers B#13 and tile clock animation B#10. |
| 5 | 1 | 0xE00 | Walk past X 0xF00 to 0xF90 | E:5 var 0, E:0, B:6 | Verify 3 water buckets at X 0xF68, 0xF90, 0xFB8 (special ammo, ammonite, cake/coffee). Verify placed torches. Verify steam tile animations B#6. |
| 5 | 1 | 0xF20 | `jump 5 1 0xF10`, walk to 0xF20 | B:15 var 6, B:49, A:3 | Verify lock 0xF20. Wall burst event B#49 blows open with effect 0x16. Sub-boss Lash T. (A#3) enters with spiked mace. |
| 5 | 1 | 0x1120 | `jump 5 1 0x1100`, walk to 0x1120 | A:35, A:45, B:50 var 3, B:70 | Verify village chief A#45 speaks. Morgan A#35 enters and shoots chief with Uzi. Dialogue box B#50 var 3 appears. |
| 5 | 1 | 0x1180 | Defeat Morgan phase 1 | A:35 st, Morgue | Morgan mutates into domed-head dinosaur Morgue. Verify dropped Uzi can be picked up. Battle Morgue until stage exit at 0x11C0. |
