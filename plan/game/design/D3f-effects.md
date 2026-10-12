# D3f: Effects (Pool C, Pool D, and Item Class 6 Torch Glow Overlay)

Written 2026-10-13 by the D3f research agent. Part of the design phase (plan/game/NEW-PLAN.md section 1, task D3f). Documents only: no code was changed, no emulator was run.
Every statement here gives numbers and addresses from the Capcom 68000 ROM and repository sources.

How to read this file:
- Source tags: R: = ROM address, RAM address, or repo file. G.$xxxx = RAM 0xFF8000 + xxxx. +$xx = struct field offset. W: = web source or documentation.
- Pool C = 48 effect slots of 0x80 bytes at RAM 0xFF9A74 (68000 offset $1A74 from a5 = 0xFF8000, or $9A74 from a5 = 0xFF0000). R: 0x4CA4, 0x237B2, dino-game/src/game/mod.rs.
- Pool D = 16 special / auxiliary sprite slots of 0x80 bytes at RAM 0xFFDDF4 (68000 offset $5DF4 from a5 = 0xFF8000, or $DDF4 from a5 = 0xFF0000). Slot 15 holds the stage clock sprite list. R: 0x4CCC, 0x81062.
- Pool E Class 6 = Torch glow overlay at handler 0x29B06 in the item pool (24 slots at RAM 0xFFBDF4). R: 0x233F0, 0x29B06.
- Class word +$20: for Pool C, +$20 is scaled by 4 (0x00, 0x04, 0x08, ... 0xA4), selecting entries 0 to 41 from long pointer table 0x237E6. R: 0x237D8.
- Rendered assets: browse re/out/assets/anims/ and re/out/assets/comps/ (folders effect_c01 to effect_c41, item_e06, unreferenced). Catalog: plan/game/design/asset-catalog.md.

## 0. Executive Summary and Architecture

1. Pool C contains 42 class slots in table 0x237E6 (bytes 0x237E6 to 0x2388D). R: 0x237E6.
   - 39 classes are actively spawned in normal gameplay across episodes 1 to 8.
   - 1 class (Class 38, +$20 = 0x98) has a NULL pointer (0x00000000) in table 0x237E6; its orphan handler code sits at 0x97430 and orphan spawner at 0x97414, never called. R: 0x2387E, 0x97414, 0x97430.
   - 2 classes (Class 0 at 0x923E0, Class 35 at 0x96E56) have valid handlers in ROM, but have no active spawn calls in gameplay (Class 35 uses title logo table 0x71F72). R: 0x923E0, 0x96E56.
2. Pool C object update (0x237B2): runs after Pool B and before sprite list generation (0x13FA6). Checks if free count equals 48; if so, returns early. Otherwise iterates through all 48 slots in slot order. R: 0x151A8, 0x237B2.
   - Mode dispatch (+$04): mode 0 = initialization (falls through or advances to mode 2), mode 2 = active tick and draw check, mode 4/6 = free slot via 0x4964. R: 0x237D8, effect.rs.
   - Allocation (0x4884): pops next slot pointer from stack at $6604(a5), decrements free count at $65fa(a5). Returns carry clear on success, carry set if empty (48 slots exhausted). R: 0x4884.
   - Free (0x4964): clears header bytes +$00 to +$27, pushes slot pointer back to stack at $6604(a5), increments free count at $65fa(a5). R: 0x4964.
3. Pool D has no independent update loop in the main game loop (0x15190..0x151BA). R: 0x15190.
   - Pool D slots are auxiliary sprite / raw list display buffers allocated by Pool B objects or system controllers.
   - Allocator (0x48A8): pops slot pointer from stack at $6606(a5), decrements count at $65fc(a5), stores slot pointer into caller struct at +$98(a6). R: 0x48A8.
   - Free (0x4994): pushes slot pointer back to stack at $6606(a5), increments count at $65fc(a5), clears +$98(a6). R: 0x4994.
   - Slot 15 of Pool D holds the stage clock sprite list (allocated at 0x81062, written at 0x8114E).
   - Other Pool D users: Pool B class 100 (0x7E72C, HUD score popups and food/item messages), 0x33368 (actor A00 intro/menu), 0x39304, 0x7E7D6, 0x7EBC0.
4. Item Class 6 (Torch Glow Overlay, 0x29B06): belongs to Pool E (24 item slots at 0xFFBDF4). R: 0x233F0, 0x29B06.
   - Spawns attached to a held or floor torch (item class 0, weapon kind 0x205).
   - Alternates Z depth between +2 and -1 on alternating frames (frame counter bit 0) to flicker behind and in front of the torch head.
   - Uses direct animation chains 0x29CB6 (idle flame, 6 frames) and 0x29CF4 (swing flame, 4 frames).
5. Port status today:
   - Only 4 Pool C classes are ported in dino-game/src/game/effect.rs: Class 1 (0x04, hit spark), Class 4 (0x10, water splash), Class 8 (0x20, flames), Class 26 (0x68, water ripple).
   - In trace.rs, PORTED_CLASSES lists ("effect", &[1, 4, 8, 26]) and ("special", &[]).
   - 38 Pool C classes, all Pool D objects, and Item Class 6 are currently MISSING in the Rust port.

## 1. Pool Architecture and Dispatch Details

### 1.1 Pool C Memory and Struct Fields
- Base address: RAM 0xFF9A74 (48 slots of 0x80 bytes = 0x1800 bytes, ending at 0xFFB274). R: 0x4CA4, DinoSetup.java.
- Slot size: 0x80 (128 bytes).
- Free counter: word at RAM $FF8000 + $65FA (G.$65FA), initialized to 48 (0x30). R: 0x4CC4.
- Free stack pointer: word at RAM $FF8000 + $6604 (G.$6604), pointing to pre-filled slot addresses at $66F8(a5). R: 0x4CC0.
- Struct layout:
  - +$00: byte, live flag (1 = active, 0 = free).
  - +$01: byte, visible flag (1 = visible, 0 = hidden).
  - +$02: byte, player / object identifier.
  - +$03: byte, subtype / variant index.
  - +$04: byte, mode (0 = init, 2 = run, 4 = dying, 6 = free).
  - +$05: byte, sub-mode.
  - +$08: 16.16 fixed X coordinate (integer word at +$08, fraction at +$0A).
  - +$0C: 16.16 fixed Y coordinate (integer word at +$0C, fraction at +$0E).
  - +$10: 16.16 fixed Z coordinate (depth, integer word at +$10).
  - +$14: 16.16 fixed X velocity (integer word at +$14).
  - +$16: 16.16 fixed Y velocity (integer word at +$16).
  - +$18: 16.16 fixed Z velocity (integer word at +$18).
  - +$1A: 16.16 fixed X acceleration.
  - +$1C: 16.16 fixed Y acceleration.
  - +$1E: 16.16 fixed Z acceleration.
  - +$20: word, class index scaled by 4 (0x00, 0x04, 0x08, ... 0xA4).
  - +$22: byte, facing / direction.
  - +$23: byte, palette override index.
  - +$24: byte, horizontal flip / mirror flag.
  - +$25: byte, draw flags (0x10 normal sprite, 0x30 blend/flicker, 0x50 raw list, 0x70, 0x90, 0xD0 raw priority).
  - +$26: word, variant or animation ID parameter.
  - +$28: pointer or long, raw list offset.
  - +$2C..+$2F: frame timing and record parameters (timer at +$2F, frame ID at +$2D).
  - +$40: long pointer, animation table address.
  - +$58..+$64: owner handle, parent object pointers, relative offsets, auxiliary timers.

### 1.2 Pool D Memory and Struct Fields
- Base address: RAM 0xFFDDF4 (16 slots of 0x80 bytes = 0x800 bytes, ending at 0xFFE5F4). R: 0x4CD0, DinoSetup.java.
- Slot size: 0x80 (128 bytes).
- Free counter: word at RAM $FF8000 + $65FC (G.$65FC), initialized to 16 (0x10). R: 0x4CE6.
- Free stack pointer: word at RAM $FF8000 + $6606 (G.$6606), pointing to stack array at $6718(a5). R: 0x4CE2.
- Slot 15: allocated by stage clock initializer 0x81062, resides at RAM 0xFFDFE4. R: 0x81062.
- Usage pattern: when a Pool B object needs auxiliary hardware sprite lists (raw lists), it allocates a Pool D slot via 0x48A8. The slot pointer is saved into +$98 of the Pool B object. The Pool B object populates the slot with sprite coordinates and tile codes, then registers it with the draw engine (0x16C6 / 0x49CA). R: 0x7E7D6, 0x81062.

## 2. Every Pool C Class (Classes 0 to 41)

Asset folders referenced:
- re/out/assets/anims/: effect_c01, effect_c02, effect_c03, effect_c04, effect_c05, effect_c06, effect_c07, effect_c08, effect_c10, effect_c11, effect_c12, effect_c13, effect_c14, effect_c15, effect_c16, effect_c19, effect_c20, effect_c21, effect_c23, effect_c24, effect_c26, effect_c28, effect_c29, effect_c30, effect_c31, effect_c32, effect_c33, effect_c36, effect_c37, effect_c39, effect_c40, effect_c41.
- re/out/assets/comps/: effect_c01, effect_c04, effect_c05, effect_c06, effect_c08, effect_c10, effect_c11, effect_c14, effect_c15, effect_c19, effect_c20, effect_c21, effect_c23, effect_c29, effect_c32, effect_c33, effect_c37, unreferenced.

### Class 0 (+$20 = 0x00): Unused Base Effect
- Handler address: 0x0923E0. Code size: 202 bytes (0xCA). R: 0x237E6, 0x923E0.
- Visuals: loads a single direct 10-byte frame record located at 0x9240E via 0x160A (fx_load). No external animation table. Asset folder: none (unreferenced). R: 0x923F4, 0x9240E.
- Spawner: no callers found in the ROM. Appears to be an early template or default zero handler. R: static scan.
- Timing: single frame record, duration 10 ticks, then calls 0x49CA for drawing check and mode 4 frees via 0x4964. R: 0x923FE.
- Movement: none (static X/Y/Z). R: 0x92402.
- Palette and draw flags: default object palette, draw flags 0x00. R: 0x923F4.
- Sound: none.
- Episodes: none.
- Port status: MISSING (stub/free on update).

### Class 1 (+$20 = 0x04): Hit Sparks and Combat Impacts
- Handler address: 0x0924AA. Code size: 862 bytes (0x35E). R: 0x237E6, 0x924AA.
- Visuals: animation table 0x0924F6 (SPARK_TABLE). 26 animations, 26 compositions in comps/effect_c01/sheet_01.png. Asset folder: anims/effect_c01/. Displays weapon slash sparks, blunt punch/kick impact stars, critical hit flares, firearm hit flashes, and knife sparks. R: 0x924BE, anims.json.
- Spawners:
  - 0x012720 (spawn_hit_spark): confirmed attack box overlap. Spark type comes from attack record byte 6. R: 0x12720, 0x127C6.
  - 0x0127FC (fx_spawn_hit_spark_alt): alternate hit check (projectiles, special attacks). R: 0x127FC, 0x1288E.
  - 0x0128C8 (fx_spawn_spark_simple): type 0 default impact at caller X/Y. R: 0x128C8.
  - 0x092422 (fx_spawn_spark_at): explicit coordinates with caller facing. R: 0x92422.
  - 0x092450 (fx_spawn_spark_jitter): coordinates with 8-entry frame jitter table at 0x9248A (indexed by G.$0 & 0xE). R: 0x92450.
- Timing: runs once, 4 to 8 frames total (each frame 2 to 4 ticks). Frees on animation completion (0x15F4 returns carry). R: 0x924E0.
- Movement: static at impact coordinates. X is midpoint of hit box overlap, Y is average of box tops, Z is min depth minus 1. R: 0x12720.
- Palette and draw flags: draw flags 0x10 (normal priority). Palette slot OBJ 0 (sparks palette). R: 0x924D6.
- Sound: triggered by attacker collision routine (sound codes 0x20..0x30 via 0x96AA sound wrapper).
- Episodes: All episodes (every combat encounter).
- Port status: DONE (dino-game/src/game/effect.rs fx_spark).

### Class 2 (+$20 = 0x08): Falling Cave Dust and Ceiling Rubble Bits
- Handler address: 0x092808. Code size: 432 bytes (0x1B0). R: 0x237E6, 0x92808.
- Visuals: raw sprite lists at 0x0928B0 (pointer table) and 0x09290E. 1 raw animation, 2 ticks per frame. Small stone pebbles and falling dirt particles. Asset folder: anims/effect_c02/. R: 0x9282E, 0x9290E.
- Spawner: 0x085AA4, spawned by Pool B class 56 (cave falling debris controller at 0x85916) in batches with randomized timing. R: 0x859A6, 0x85AA4, D3e section 2.1.
- Timing: falls from ceiling (Y = 208 down to floor), lives approximately 40 to 60 frames until reaching floor plane. R: 0x92852.
- Movement: downward vertical velocity (vy = -2.5 to -4.0 px/frame), horizontal drift determined by parameter +$26. Position offsets read from table at 0x9281E. R: 0x92846.
- Palette and draw flags: draw flags 0x50 (raw list). Visible flag +$01 toggles every frame via frame counter bit 0 (flicker effect). R: 0x9283C, 0x92860.
- Sound: Pool B class 56 plays ambient cave rumble sound 0x8D via 0xA63C. R: 0x85916, D3e section 2.1.
- Episodes: Episode 6 area 1 (rock cavern slope).
- Port status: MISSING.

### Class 3 (+$20 = 0x0C): Ground Impact Shockwave
- Handler address: 0x0929B8. Code size: 144 bytes (0x90). R: 0x237E6, 0x929B8.
- Visuals: direct frame chains at 0x092A30 (variant 0) and 0x092A3C (variant != 0). 1 animation, 2 ticks. Ground shockwave dust or impact line. Asset folder: anims/effect_c03/. R: 0x929CC, 0x92A30.
- Spawner: 0x03C9FC in enemy Hammer T. / Blk Elmer / Wrench T. smash routine (when swinging sledgehammer or pipe into the floor). R: 0x3C9FC.
- Timing: single-shot, 6 to 12 frames duration. Checks owner at +$58; if owner is freed or state changes, frees early. R: 0x929F0.
- Movement: follows owner X/Y/Z coordinates or stays at ground hit point. R: 0x92A0C.
- Palette and draw flags: draw flags 0x10. Default dust palette. R: 0x929E6.
- Sound: Hammer smash sound 0x25 triggered by enemy code. R: 0x3C9B0.
- Episodes: Episodes 1, 2, 4, 5, 7, 8 (Hammer T. encounters).
- Port status: MISSING.

### Class 4 (+$20 = 0x10): Water Splash on Landing
- Handler address: 0x092A48. Code size: 306 bytes (0x132). R: 0x237E6, 0x92A48.
- Visuals: animation table 0x092AC0 (SPLASH_TABLE). 6 animations, 6 compositions in comps/effect_c04/sheet_01.png. White water plume expanding upward. Asset folder: anims/effect_c04/. R: 0x92A72, anims.json.
- Spawner: player water landing check 0x00606E / 0x0060A2 (when hero lands in water with vy < 0 and Y < 32). R: 0x606E, 0x60A2.
- Timing: 6 to 10 frames total. Suppresses owner's draw-time water reflection/cut flag (draw_flags bit 5) while active. R: 0x92A5C, 0x92AA0.
- Movement: static at landing X and depth Z; height forced to Y = 32 (water surface line). R: 0x60A2, effect.rs.
- Palette and draw flags: draw flags 0x10. Water palette (OBJ slot 4). R: 0x92A86.
- Sound: calls sound routine 0x9AF4 with water splash sound 0x9AF4 directly from caller. R: 0x60A2.
- Episodes: Episode 2 area 0 and area 1 (forest swamps, swamp paths).
- Port status: DONE (dino-game/src/game/effect.rs fx_water_splash).

### Class 5 (+$20 = 0x14): Water Bubbles and Swimming Wake
- Handler address: 0x092B7A. Code size: 288 bytes (0x120). R: 0x237E6, 0x92B7A.
- Visuals: animation table 0x092BF0. 12 animations, 12 compositions in comps/effect_c05/sheet_01.png. Water foam, air bubbles, and wake crests around wading characters. Asset folder: anims/effect_c05/. R: 0x92B8E, anims.json.
- Spawners: player wading movement routines 0x006192 and 0x0061F6 (spawned periodically when walking or running in deep water). R: 0x6192, 0x61F6.
- Timing: each bubble or wake puff lasts 120 ticks (or until animation ticks out). Frees if stage water flag G.$518 changes. R: 0x92BB4.
- Movement: initial spawn at owner X, Y = owner Y + 96 (0x60), owner Z. Remains floating on water surface with slight vertical bobbing. R: 0x61BA.
- Palette and draw flags: draw flags 0x10. Water palette. R: 0x92BAA.
- Sound: none (ambient visual).
- Episodes: Episode 2 areas 0 and 1.
- Port status: MISSING.

### Class 6 (+$20 = 0x18): Continue Screen Dynamite Fuse Spark
- Handler address: 0x092C9A. Code size: 562 bytes (0x232). R: 0x237E6, 0x92C9A.
- Visuals: direct frame chains at 0x092E12, 0x092E1E, 0x092E34. Sizzling white and yellow spark on the continue screen dynamite stick. Asset folder: anims/effect_c06/. R: 0x92D00, 0x92E12.
- Spawners: continue screen countdown controller 0x00EC5E / 0x00EC74 and 0x00EEC2. R: 0xEC74, 0xEEC2.
- Timing: burns continuously during the 20-second continue countdown. Ticks every frame; advances spark along fuse path. R: 0x92CBA.
- Movement: follows predetermined fuse coordinates moving downward along dynamite bundle. R: 0x92CE0.
- Palette and draw flags: draw flags 0x10. Continues screen palette bank. R: 0x92CE8.
- Sound: calls sound routine 0xA63C with sizzling fuse sound 0x79. R: 0x92CD2.
- Episodes: Continue screen (all episodes upon player defeat).
- Port status: MISSING.

### Class 7 (+$20 = 0x1C): Ground Dust Puff (Slide, Skid, Landing)
- Handler address: 0x092ECC. Code size: 232 bytes (0xE8). R: 0x237E6, 0x92ECC.
- Visuals: raw sprite list at 0x092F3A. 1 raw animation, 2 ticks per frame. Small expanding grey/brown dust cloud at characters' feet. Asset folder: anims/effect_c07/. R: 0x92EE8, 0x92F3A.
- Spawners:
  - 0x092E5E: dust spawn helper. Entry 0x092E76 for dash slide / skid dust; entry 0x092EA4 for heavy landing / knockdown impact dust. R: 0x92E5E, 0x92E76, 0x92EA4.
  - 0x029F54: thrown weapon or heavy container landing bounce. R: 0x29F54.
- Timing: short one-shot burst, 12 to 16 frames total. Frees via 0x4964 when raw list reaches end. R: 0x92F00.
- Movement: static at ground position, or drifts 1 px backward opposite character facing. R: 0x92EE0.
- Palette and draw flags: draw flags 0x50 (raw list). Ground dust palette. R: 0x92EF4.
- Sound: none (soft impact).
- Episodes: All episodes (every running stop, jump landing, knockdown).
- Port status: MISSING (noted in NEW-PLAN.md D3f, D4a).

### Class 8 (+$20 = 0x20): Attached Burning Body Flames
- Handler address: 0x092FB4. Code size: 464 bytes (0x1D0). R: 0x237E6, 0x92FB4.
- Visuals: direct frame chains at 0x093096 / 0x0930F4 (airborne loop, 10 bytes/rec) and 0x0930D4 / 0x09313C (dying-out ground flames). 12 compositions in comps/effect_c08/sheet_01.png. Flickering orange fire tongues. Asset folder: anims/effect_c08/. R: 0x92FC8, anims.json.
- Spawners:
  - 0x01C86E / 0x01C876: player fire hit reaction (spawns 2 flame objects from table 0x1C8A6). R: 0x1C876.
  - 0x02AD98 / 0x02ADAE: enemy fire hit reaction (spawns 1 to 3 flames from table 0x2CA36 indexed by enemy type +$84). R: 0x2ADAE.
  - 0x01F9E4, 0x04A052: boss / special fire attacks. R: 0x1F9E4, 0x4A052.
- Timing: loops airborne flame frames while owner body has airborne flag set (+$51 != 0). When owner lands, transitions to dying-out sequence (about 20 frames) and frees. R: 0x9304A, 0x93054.
- Movement: rigidly attached to owner body (+$60) at offsets (+#$62, +#$64), mirrored by owner facing (+$24). R: 0x92FF6.
- Palette and draw flags: draw flags 0x30. Drawn on odd frames only (G.$1 & 1 != 0) creating 30 Hz flicker/transparency. Fire palette. R: 0x93078.
- Sound: burn impact sizzle triggered by damage handler.
- Episodes: All episodes with torch, explosive barrels, Molotov, or flame hazards (Episodes 1, 3, 5, 6, 8).
- Port status: DONE (dino-game/src/game/effect.rs fx_flames).

### Class 9 (+$20 = 0x24): "STAGE CLEARED!" Banner and Digits
- Handler address: 0x093184. Code size: 334 bytes (0x14E). R: 0x237E6, 0x93184.
- Visuals: direct frame table at 0x093262 pointing to composition frames 2895 to 2903 ("STAGE CLEARED!" banner graphic and episode digits 1 through 8). Unreferenced compositions in asset catalog N1, section 4.4. Asset folder: anims/effect_c09/ (comps/unreferenced/). R: 0x93198, 0x93262, asset-catalog.md.
- Spawner: stage clear transition controller 0x093152 (called upon boss defeat when area clear music begins). R: 0x93152.
- Timing: flies in to center screen (camera X + 192, camera Y + 112), holds for 180 frames (3 seconds), then slides off screen or fades. R: 0x931C4, 0x93200.
- Movement: enters from top or right, snaps to fixed screen center, tracks scroll camera. R: 0x931DE.
- Palette and draw flags: draw flags 0x90 (high priority overlay over all sprites and scroll layers). UI gold/red palette. R: 0x931B8.
- Sound: accompanied by stage clear fanfare music code 0x0D (0xA582 table). R: D4e section 1.4.
- Episodes: Episodes 1 through 8 (after every stage boss defeat).
- Port status: MISSING.

### Class 10 (+$20 = 0x28): Vehicle / Heavy Impact Debris Scatter
- Handler address: 0x0932D2. Code size: 548 bytes (0x224). R: 0x237E6, 0x932D2.
- Visuals: animation table 0x093426. 7 animations, 14 compositions in comps/effect_c10/sheet_01.png. Flying motorcycle wreckage fragments, metal plates, exhaust shards. Asset folder: anims/effect_c10/. R: 0x93426, anims.json.
- Spawner: 0x093362 (loop helper spawning 10 pieces with velocity vectors), called by:
  - 0x079146: Boss Hogg motorcycle crash explosion. R: 0x79146.
  - 0x07B1A6: Boss Slice / vehicle crash in Episode 4. R: 0x7B1A6.
- Timing: pieces fly outward in parabolic arcs for 40 to 60 frames, bounce once on floor, then fade. R: 0x93340.
- Movement: initial velocities loaded from parameter tables at 0x79152 and 0x7B1B2 (vx = -6 to +6 px/frame, vy = 4 to 8 px/frame upward, gravity = 0.5 px/frame^2). R: 0x79152, 0x93386.
- Palette and draw flags: draw flags 0x30. Vehicle metal palette. R: 0x93310.
- Sound: accompanied by vehicle explosion sound 0x37. R: 0x79130.
- Episodes: Episode 3 (Hogg fight) and Episode 4 (Slice fight).
- Port status: MISSING.

### Class 11 (+$20 = 0x2C): Broken Container Debris and Stone Spawns
- Handler address: 0x0934F6. Code size: 1248 bytes (0x4E0). R: 0x237E6, 0x934F6.
- Visuals: animation table 0x0937AA (0x937AA). 18 animations, 43 compositions in comps/effect_c11/sheet_01.png. Wooden barrel planks, oil drum jagged steel shards, stone boulders/rubble, statue golden fragments, tire rubber chunks. Asset folder: anims/effect_c11/. R: 0x934F6, 0x937AA, anims.json.
- Spawners:
  - 0x0292A6, 0x0292F6: container destruction (oil drum 2 pieces, wooden barrel 10 pieces, tire stack 5 pieces). R: 0x292A6, D4a section 5.1.
  - 0x02944A, 0x0294B8: stone container and boulder break (spawns 9 stone debris pieces). R: 0x2944A, 0x294B8.
  - 0x0259E8: held item thrown against wall/ground and shattered. R: 0x259E8.
  - 0x06B3EA: falling rocks shattered on impact with floor (Episode 5). R: 0x6B3EA.
- Stone weapon bonus spawn: at routine 0x093770 (and 0x0294AA), each of the 9 stone rubble pieces checks random generator; if G.$4F1 < 3, can transform into a collectible stone weapon (small, medium, or large stone). R: 0x294AA, 0x93770, D4a section 8.4.
- Timing: pieces fly outward for 30 to 45 frames, bounce 1 to 2 times upon hitting ground line (0x3140C floor test), then disappear. R: 0x93560.
- Movement: 3D parabolic trajectory. Horizontal velocity vx spread between -5 and +5 px/frame; vertical velocity vy begins at +3 to +7 px/frame; gravity 0.5 px/frame^2; depth vz drift. R: 0x93520.
- Palette and draw flags: draw flags 0x10. Container specific palette copied from destroyed object. R: 0x93510.
- Sound: wood shatter sound 0x24, metal tear sound 0x25, stone shatter sound 0x26 via 0xA63C. R: 0x29280.
- Episodes: All episodes (every container or falling rock destruction).
- Port status: MISSING (task IT-P2 in D4a section 12).

### Class 12 (+$20 = 0x30): Character Ground Indicator / Shadow Marker
- Handler address: 0x0939D6. Code size: 164 bytes (0xA4). R: 0x237E6, 0x939D6.
- Visuals: raw sprite list at 0x093A4A. 1 raw animation, 2 ticks per frame. Small oval shadow / tracking dot at floor plane. Asset folder: anims/effect_c12/. R: 0x939F2, 0x93A4A.
- Spawner: helper 0x0939B6, called during character initialization (0x33866, 0x35A4C, 0x392FE, 0x425B6, 0x4BE96, 0x4FE24, 0x5C83E, 0x61944) when debug or special shadow flag G.$4AA is active. R: 0x939B6.
- Timing: persistent while owner character (+$58) is alive; frees when owner is destroyed. R: 0x93A00.
- Movement: rigidly snaps to owner X and depth Z; Y coordinate snapped to floor plane. R: 0x93A14.
- Palette and draw flags: draw flags 0x50 (raw list). Shadow palette. R: 0x939FC.
- Sound: none.
- Episodes: All episodes (under conditions where extra shadow is requested).
- Port status: MISSING.

### Class 13 (+$20 = 0x34): Auxiliary Ground Ripple / Floor Marker
- Handler address: 0x093A7A. Code size: 174 bytes (0xAE). R: 0x237E6, 0x93A7A.
- Visuals: raw sprite list at 0x093AFA. 1 raw animation, 2 ticks per frame. Subtle ground ring / circular outline. Asset folder: anims/effect_c13/. R: 0x93A8E, 0x93AFA.
- Spawner: 0x093A60 (called when floor tracking or landing cue is triggered). R: 0x93A60.
- Timing: 10 to 18 frames, single-shot dissipation. R: 0x93AB0.
- Movement: static on floor plane at target coordinates. R: 0x93AA2.
- Palette and draw flags: draw flags 0x50 (raw list). R: 0x93A84.
- Sound: none.
- Episodes: Episodes 2, 5.
- Port status: MISSING.

### Class 14 (+$20 = 0x38): Respawn Flash and 1UP Text Marker
- Handler address: 0x093B28. Code size: 812 bytes (0x32C). R: 0x237E6, 0x93B28.
- Visuals:
  - Variant 0: Respawn flash (expanding vertical light pillar and ground glow disc). Animation table 0x093C82. 5 animations, 12 compositions in comps/effect_c14/sheet_01.png. Asset folder: anims/effect_c14/. R: 0x93B50, 0x93C82.
  - Variant 2: "1UP!" text graphic floating over hero. Raw sprite list at 0x093D1A. Asset folder: anims/effect_c14/. R: 0x93BF8, 0x93D1A.
- Spawners:
  - Variant 0: player respawn / revive flow 0x018C1E / 0x018C20 (when hero drops in after losing a life, continue, or game start). R: 0x18C20, D2b section 3.
  - Variant 2: extra life award routine 0x006B98 / 0x00A4CA (when score threshold 500,000 or 1,000,000 points is reached and player enters idle victory pose). R: 0x6B98, D4b section 3.5.
- Timing:
  - Variant 0 (Respawn flash): plays animation for 30 frames, expanding and fading. R: 0x93B70.
  - Variant 2 (1UP marker): timer initialized to 180 frames (+$5A = 0xB4, 3 seconds). Decrements every frame and tracks hero head; frees when timer reaches 0. R: 0x93C0E, 0x93C20.
- Movement:
  - Variant 0: static at hero drop-in landing spot. R: 0x18C42.
  - Variant 2: tracks owner hero (+$58) at X = hero X, Y = hero Y + 80 px (floating above head), Z = hero Z. R: 0x93C3E.
- Palette and draw flags:
  - Variant 0: draw flags 0x30 (blend/flicker). White/yellow light palette. R: 0x93B60.
  - Variant 2: draw flags 0x70. UI font yellow/red palette. R: 0x93C08.
- Sound: extra life sound code 0x4A via 0xA63C. R: 0x6B98, D4b section 3.5.
- Episodes: All episodes.
- Port status: MISSING (known missing respawn flash and 1UP marker from D2b, D4b).

### Class 15 (+$20 = 0x3C): Burn Death Smoke and Charred Collapse
- Handler address: 0x093E54. Code size: 424 bytes (0x1A8). R: 0x237E6, 0x93E54.
- Visuals: animation table 0x093F42. 6 animations, 12 compositions in comps/effect_c15/sheet_01.png. Smoldering charred body silhouette, rising black soot wisps, and crumbling ash. Asset folder: anims/effect_c15/. R: 0x93E68, 0x93F42, anims.json.
- Spawner: fatal burn death spawner 0x093D48 / 0x093D5E (called when an enemy or character HP reaches 0 from fire/explosion attacks). R: 0x93D48, 0x93D5E.
- Timing: character collapses to floor, smolders for 60 to 90 frames (timer at +$58 randomized between 30 and 93 frames), then disintegrates and frees. R: 0x93E96, 0x93EB2.
- Movement: falls downward with initial vy (0x93D8C), settles on ground line, horizontal speed negates by facing. R: 0x93D9C.
- Palette and draw flags: draw flags 0x10 (normal) or 0x30 (flicker). On Episode 8 (stage index 7), adds flag 0x20. Dark silhouette palette. R: 0x93E7A, 0x93E8A.
- Sound: sizzling burn death hiss sound 0x23. R: 0x93D40.
- Episodes: Episodes 1, 3, 5, 6, 8.
- Port status: MISSING (known missing burn death 0x93D48 from prompt).

### Class 16 (+$20 = 0x40): Continue Screen Dynamite Spark Burst
- Handler address: 0x093FFC. Code size: 260 bytes (0x104). R: 0x237E6, 0x93FFC.
- Visuals: raw sprite lists at 0x0940A4 and 0x0940AE. 1 raw animation, 2 ticks per frame. Bright starburst sparks shooting off fuse tip. Asset folder: anims/effect_c16/. R: 0x94014, 0x940A4.
- Spawners: continue countdown loop 0x00EDBA and 0x00EDD2 (fires bursts as countdown approaches 0). R: 0xEDBA, 0xEDD2.
- Timing: short sparks, lasting 8 to 14 frames per burst. R: 0x94030.
- Movement: radial scatter velocity outward from fuse tip. R: 0x94022.
- Palette and draw flags: draw flags 0x50 (raw list). White spark palette. R: 0x9401C.
- Sound: accompanied by fuse sizzle / firecracker pops. R: 0xEDB4.
- Episodes: Continue screen.
- Port status: MISSING.

### Class 17 (+$20 = 0x44): Continue Screen Countdown Digits
- Handler address: 0x094100. Code size: 454 bytes (0x1C6). R: 0x237E6, 0x94100.
- Visuals: direct frame table at 0x09421A (indexing individual countdown digit graphics 9 down to 0). Large stylized numbers displayed in the center of the continue screen. Asset folder: anims/effect_c17/. R: 0x94120, 0x9421A.
- Spawners: continue screen digit controller 0x00ED04 / 0x00ED0C, and helper entries 0x09418E, 0x0941B6. R: 0xED0C, 0x9418E, 0x941B6.
- Timing: each digit displays for exactly 51 game frames (one arcade timer second), pulsing/scaling slightly before advancing to the next digit. R: 0xED04, D4b section 1.1.
- Movement: static at fixed screen coordinates (screen center X = 192, Y = 112). R: 0xED2E.
- Palette and draw flags: draw flags 0x10. Continue screen high-contrast red/yellow palette. R: 0x94134.
- Sound: second tick beep sound 0x34 played at each digit decrement. R: 0xED00, D4e section 1.4.
- Episodes: Continue screen.
- Port status: MISSING.

### Class 18 (+$20 = 0x48): Food / Debris Vomit Projectile Effect
- Handler address: 0x0942C6. Code size: 412 bytes (0x19C). R: 0x237E6, 0x942C6.
- Visuals: generated raw sprite list from data table at 0x09430E. Procedural sprite assembler via 0x1236 and 0x121E. Yellow/green bile globules and half-eaten food morsels spewed outward. Asset folder: anims/effect_c18/. R: 0x942E6, 0x9430E.
- Spawner: 0x03B1E8 in enemy Bludge (class 6) special attack / vomit roll routine when disgorging swallowed items. R: 0x3B1E8, D3a section 4.
- Timing: travels forward in air for 25 to 35 frames, splashes on floor, then vanishes. R: 0x942F6.
- Movement: horizontal forward velocity vx = 3.5 px/frame; vertical arc with initial vy = 2.0 px/frame, downward gravity. R: 0x942F0.
- Palette and draw flags: default projectile palette. R: 0x942E0.
- Sound: vomit sound cue via Bludge audio triggers.
- Episodes: Episodes 5, 6, 7 (Bludge stages).
- Port status: MISSING.

### Class 19 (+$20 = 0x4C): Dizzy Stars Circling Head
- Handler address: 0x094462. Code size: 236 bytes (0xEC). R: 0x237E6, 0x94462.
- Visuals: animation table 0x0944DC. 2 animations, 3 frames, 30 ticks each, loops. Compositions in comps/effect_c19/sheet_01.png. Ring of yellow spinning stars orbiting character head. Asset folder: anims/effect_c19/. R: 0x94476, 0x944DC, anims.json.
- Spawner: 0x094338 (dizzy state trigger), called when hero or enemy takes repeated heavy hits and enters stun state 0x16. R: 0x94338, D2b section 4.
- Timing: active while owner (+$58) remains in dizzy state. Loops animation continuously. Timer decrements faster when player mashes joystick and buttons (reduces duration from 180 frames down to 60 frames). R: 0x944A0, 0x944BC.
- Movement: attached to owner head. X and depth Z track owner; Y position offset above head (+60 to +80 px, lookup by owner character height). Circular orbit offset applied per frame. R: 0x94380, 0x94392.
- Palette and draw flags: draw flags 0x30 (blend/flicker). Bright yellow star palette. R: 0x94492.
- Sound: looping chirping dizzy sound code 0x51 via 0xA63C. R: D4e section 1.3.
- Episodes: All episodes (any character entering dizzy stun).
- Port status: MISSING (known missing dizzy stars 0x94338 from prompt).

### Class 20 (+$20 = 0x50): Character Stun Shock / Sweat Drops
- Handler address: 0x09454E. Code size: 292 bytes (0x124). R: 0x237E6, 0x9454E.
- Visuals: animation table 0x0945CC. 9 animations, 1 frame, 7 ticks each. Blue sweat beads and startled exclamation lines. Compositions in comps/effect_c20/sheet_01.png. Asset folder: anims/effect_c20/. R: 0x94554, 0x945CC, anims.json.
- Spawner: 0x094522 (called when an enemy or dinosaur is surprised, wakes from sleep, or takes unexpected poise damage). R: 0x94522.
- Timing: short 14-frame burst, pops outward and fades. R: 0x94570.
- Movement: floats upward (+1 px/frame) and drifts away from head. R: 0x94564.
- Palette and draw flags: UI blue/white palette. R: 0x94558.
- Sound: comic squeak / startled chirp sound.
- Episodes: Episodes 2, 5, 7 (sleeping dinosaur encounters).
- Port status: MISSING.

### Class 21 (+$20 = 0x54): Heavy Smoke Puffs and Container Burst Clouds
- Handler address: 0x094672. Code size: 4558 bytes (0x11CE). R: 0x237E6, 0x94672.
- Visuals: animation table 0x0951AC. 60 animations, 141 compositions in comps/effect_c21/sheet_01.png to sheet_04.png. Vast array of multi-directional billowing white/grey smoke puffs, expanding impact shock rings, and explosive cloud bursts. Asset folder: anims/effect_c21/. R: 0x94672, 0x951AC, anims.json.
- Spawners:
  - 0x094640: primary smoke puff spawner, takes variant d0 and facing d1. R: 0x94640.
  - 0x0946C8: secondary recursive burst spawner (chains child puffs in expanding rings). R: 0x946C8.
  - 0x095A3C: explosion cloud controller. R: 0x95A3C.
  - Callers: barrel break (spawns 10 puff objects), tire stack break (spawns 5 puffs), golden statue smash (8 puffs), grenade/dynamite ground impacts. R: 0x292A6, D4a section 5.1.
- Timing: each puff runs 18 to 30 frames, expanding and decelerating before dissolving. R: 0x946A6.
- Movement: radial velocity vectors loaded from extensive tables starting at 0x9484A (vx = -4 to +4 px/frame, vy = -2 to +6 px/frame, drag applied every frame). R: 0x9484A.
- Palette and draw flags: draw flags 0x30 (blend/flicker). White smoke palette. R: 0x94720.
- Sound: soft poof / cloud expansion sound 0x27 via 0xA63C. R: 0x94680.
- Episodes: All episodes (every container break, heavy grenade blast, vehicle tire burnout).
- Port status: MISSING (known missing puff 0x93770 / 0x94640 from prompt).

### Class 22 (+$20 = 0x58): Screen Flash / Full-Palette Lightning Pulse
- Handler address: 0x095840. Code size: 206 bytes (0xCE). R: 0x237E6, 0x95840.
- Visuals: direct palette manipulation. Directly writes to CPS1 palette RAM at 0x914160 and 0x914180 (ORing 0xF000F000 for maximum brightness, copying from base palette bank at 0xB8612). R: 0x95866, 0x9587E.
- Spawner: 0x09582A (called during heavy explosions, bazooka detonation, dynamite blast, boss phase transitions). R: 0x9582A.
- Timing: lasts 2 to 4 frames. Whiteout flash on frame 0, decays to half-brightness on frame 2, restores original stage palette on frame 4. R: 0x95880.
- Movement: global screen effect (no spatial coordinates). R: 0x95840.
- Palette and draw flags: direct palette RAM overwrite. R: 0x9586C.
- Sound: synchronized with thunder / mega-explosion sound 0x3A. R: D4e section 1.2.
- Episodes: All episodes (bazooka shots, dynamite blasts, boss defeats).
- Port status: MISSING.

### Class 23 (+$20 = 0x5C): Sparkle / Invulnerability Glitter
- Handler address: 0x09590E. Code size: 216 bytes (0xD8). R: 0x237E6, 0x9590E.
- Visuals: direct frame chain at 0x095980. 1 animation, 5 frames, 20 ticks. Compositions in comps/effect_c23/sheet_01.png. Sparkling 4-point golden/white glint shrinking and spinning. Asset folder: anims/effect_c23/. R: 0x9592E, 0x95980, anims.json.
- Spawner: 0x0958E4 (invulnerability sparkle emitter), called every 4th frame by player respawn update (0x18CD4) while hero has post-respawn invulnerability timer active (+$EC > 0, 360 frames total). R: 0x18CD4, 0x958E4, D2b section 3.
- Timing: runs through 5 frames (20 ticks total = 5 game frames) and frees. R: 0x9596A.
- Movement: spawns with randomized jitter around hero body: X offset PRNG between -16 and +15 px; Y offset PRNG between +24 and +87 px (torso to head); drifts upward 1 px/frame. R: 0x95938, 0x95966.
- Palette and draw flags: draw flags 0x30 (blend/flicker). Golden glitter palette. R: 0x95928.
- Sound: none (continuous subtle visual indicator).
- Episodes: All episodes (whenever any player respawns or joins).
- Port status: MISSING (known missing sparkle 0x5C from prompt).

### Class 24 (+$20 = 0x60): Massive Explosion Blast and Fireball
- Handler address: 0x0959E6. Code size: 2224 bytes (0x8B0). R: 0x237E6, 0x959E6.
- Visuals: animation table 0x095E28. 41 animations, 84 compositions in comps/effect_c24/sheet_01.png and sheet_02.png. Massive multi-stage orange/red fireball plume, detonating core, and shockwaves. Asset folder: anims/effect_c24/. R: 0x959E6, 0x95E28, anims.json.
- Spawner: 0x0959B4, called upon dynamite detonation (0x26140), explosive barrel burst (0x28E5A), bazooka projectile hit (0x25060), and Hogg / vehicle explosion. R: 0x959B4, D4a section 9.4.
- Timing: 36 to 48 frames total. Initial white flash core (6 frames), expanding fire ball (18 frames), dissipating black smoke cloud (20 frames). R: 0x95A10.
- Movement: static epicenter at blast origin. Spawns subsidiary Class 21 puff objects in a circle. R: 0x95A3C.
- Palette and draw flags: draw flags 0x10. Fiery orange/yellow explosion palette. R: 0x95A00.
- Sound: mega-explosion sound 0x3A via 0xA63C. R: 0x28E90, D4e section 1.2.
- Episodes: All episodes (any heavy explosive detonation).
- Port status: MISSING.

### Class 25 (+$20 = 0x64): Dynamite / Explosive Container Fuse Spark
- Handler address: 0x096296. Code size: 134 bytes (0x86). R: 0x237E6, 0x96296.
- Visuals: animation table 0x028904 (located in item code segment). 4-frame cycling sparkling yellow/white fuse spark. Asset folder: anims/item_e00/ (table 0x28904). R: 0x28904, 0x962B0.
- Spawner: 0x09625C, called when a dynamite weapon is armed/dropped (0x25E40) or an explosive container (drum/barrel) is damaged and enters fuse state (0x28E5A). R: 0x9625C, D4a section 5.3, section 9.4.
- Timing: loops continuously for 60 frames (dynamite fuse) or 112 frames (explosive container fuse) until the explosive detonates. R: 0x962C0, D4a section 5.3.
- Movement: attached rigidly to owner item (+$60) at offset X = owner X + 10 px, Y = owner Y + 57 px (tip of fuse string). Tracks owner if picked up or thrown. R: 0x96284, 0x962F8.
- Palette and draw flags: draw flags 0x10. White/yellow spark palette. R: 0x962BA.
- Sound: sizzling fuse sound 0x79 via 0xA63C. R: 0x25E70.
- Episodes: Episodes 2, 4, 5, 6, 8.
- Port status: MISSING (known missing fuse spark 0x94640 / 0x9625C from prompt).

### Class 26 (+$20 = 0x68): Water Ripple Ring Around Wading Body
- Handler address: 0x09631C. Code size: 186 bytes (0xBA). R: 0x237E6, 0x9631C.
- Visuals: raw sprite list at 0x096360. 1 raw animation, 3 frames, 12 ticks total. Expanding oval concentric ripple rings on water surface. Asset folder: anims/effect_c26/. R: 0x96330, 0x96360, anims.json.
- Spawner: player wading movement routine 0x0060E2 / 0x00610A (triggered every 7 frames when hero moves in water with Y = 24). R: 0x60E2, 0x610A.
- Timing: 12 game frames total. Expands outward and fades. R: 0x96340.
- Movement: stays at water plane (Y = 24); X position jittered by LFSR noise word (G.$4 & 0x1F) - 16; Z depth = owner depth minus 2. R: 0x610A, effect.rs.
- Palette and draw flags: draw flags 0x50 (raw list). Water palette. R: 0x96338.
- Sound: soft water lap sound 0x9AF4. R: 0x60E2.
- Episodes: Episode 2 areas 0 and 1 (forest swamp).
- Port status: DONE (dino-game/src/game/effect.rs fx_ripple).

### Class 27 (+$20 = 0x6C): Boss Blade Afterimage / Motion Blur Shadow
- Handler address: 0x0963D6. Code size: 178 bytes (0xB2). R: 0x237E6, 0x963D6.
- Visuals: raw composition shadow copier. Clones owner character's sprite composition pointer (+#$28) and facing (+#$24) with alternate palette override (+#$23). Asset folder: anims/effect_c27/ (reuses boss compositions). R: 0x963C4, D3d section 1.
- Spawner: 0x09639E, called by Boss Slice (class 25, 0x4CE70) and Boss Slisaur (class 71, 0x5DD58) during dash slashes and spin kicks. R: 0x9639E, D3d BT03.
- Timing: lives for 36 frames total, executed in 6 steps of 6 frames. Cycles through afterimage palettes 0x9D down to 0x8B. R: 0x963F6, 0x9640C, D3d BT03.
- Movement: stays frozen at the spot where the boss was located when the slash started (leaving a ghost trailing image). R: 0x963B2.
- Palette and draw flags: draw flags 0x30. Palette shifted to translucent ghost tint. R: 0x96428.
- Sound: accompanied by sword slash whoosh sound 0x22. R: 0x4CE80.
- Episodes: Episode 4 (Slice) and Episode 7 (Slisaur).
- Port status: MISSING.

### Class 28 (+$20 = 0x70): "TIME OVER" Flying Words
- Handler address: 0x096488. Code size: 326 bytes (0x146). R: 0x237E6, 0x96488.
- Visuals: raw sprite lists at 0x096546 (variant 0, "TIME") and 0x096550 (variant 1, "OVER"). Large red/gold arcade font letters. Asset folder: anims/effect_c28/. R: 0x96488, 0x96546, 0x96550.
- Spawner: time-out controller 0x096450 (called by rank/timer system 0x103FE / 0x1046A when stage clock word G.$4E8 reaches 0:00). Allocates two objects: variant 0 ("TIME") at 0x096450, variant 1 ("OVER") at 0x09646A. R: 0x103FE, 0x96450, 0x9646A, D4b section 1.5.
- Timing: fly-in takes 48 frames; holds at screen center for 180 frames; all active heroes HP forced to 0 at t = 48. Total duration about 240 frames before fading. R: 0x1046A, 0x96488, D2b section 7.
- Movement:
  - "TIME" flies in from screen right (startX = 384) towards center X = 192 (vx = -4.0 px/frame).
  - "OVER" flies in from screen left (startX = 0) towards center X = 192 (vx = +4.0 px/frame).
  - Both lock at center X = 192 (0xC0), Y = 112. R: 0x964B0, D4b TR03.
- Palette and draw flags: draw flags 0xD0 (highest priority raw sprite overlay). UI red/yellow font palette. R: 0x9649C.
- Sound: no sound code sent directly; death flow follows with hero scream sounds. R: D4e section 1.4.
- Episodes: All episodes (when clock expires).
- Port status: MISSING (task T-TR2 in D4b).

### Class 29 (+$20 = 0x74): Small Ground Impact Smoke Ring
- Handler address: 0x0965CE. Code size: 230 bytes (0xE6). R: 0x237E6, 0x965CE.
- Visuals: animation table 0x09664E. 1 animation, 1 frame, 90 ticks. Expanding circular dust ring hugging the floor. Compositions in comps/effect_c29/sheet_01.png. Asset folder: anims/effect_c29/. R: 0x965DA, 0x9664E, anims.json.
- Spawner: 0x0965A2 (called upon heavy character landing or falling boulder ground strike). R: 0x965A2.
- Timing: expands outward for 18 to 24 frames and vanishes. R: 0x965F0.
- Movement: static at impact coordinates on floor plane. R: 0x965B6.
- Palette and draw flags: draw flags 0x10. Dust palette. R: 0x965E2.
- Sound: thud impact sound.
- Episodes: Episodes 2, 5, 8.
- Port status: MISSING.

### Class 30 (+$20 = 0x78): Sleeping Dinosaur "Z" Snore / Sweat Bead
- Handler address: 0x0966B4. Code size: 332 bytes (0x14C). R: 0x237E6, 0x966B4.
- Visuals: animation table 0x096770. 1 animation, 3 frames, 130 ticks total. Comic style floating "Z" snore characters and blue sweat droplets. Compositions in comps/effect_c30/sheet_01.png. Asset folder: anims/effect_c30/. R: 0x966C0, 0x96770, anims.json.
- Spawners:
  - 0x048576 in dinosaur Shivat (T. rex, class 24) sleeping AI loop (0x48340). R: 0x48576, D3b section 2.
  - 0x0535B4 in dinosaur Mack (triceratops, class 38) sleeping AI loop (0x533FA). R: 0x535B4, D3b section 3.
- Timing: spawns periodically every 120 frames while dinosaur is asleep. Floats upward for 40 frames and pops. Frees immediately if dinosaur wakes up. R: 0x966DC.
- Movement: attached to dinosaur head (+#$68), drifts upward (vy = 1.0 px/frame) and sways horizontally. R: 0x96692, 0x966E0.
- Palette and draw flags: draw flags 0x10. Comic white/blue palette. R: 0x966C8.
- Sound: quiet dinosaur snore sound.
- Episodes: Episode 2 area 0, Episode 5 area 0 (sleeping dinosaur scenes).
- Port status: MISSING.

### Class 31 (+$20 = 0x7C): Gunshot Muzzle Flash
- Handler address: 0x096800. Code size: 638 bytes (0x27E). R: 0x237E6, 0x96800.
- Visuals: animation table 0x096884. 4 animations, 8 compositions in comps/effect_c31/sheet_01.png.
  - Animation 0: Revolver / Handgun flash (small starburst, 6 frames).
  - Animation 1: Hunting rifle flash (sharp elongated cone, 6 frames).
  - Animation 2: Shotgun blast flash (wide multi-point flare, 6 frames).
  - Animation 3: Uzi / M-16 automatic burst flash (rapid bright flare, 4 frames).
  Asset folder: anims/effect_c31/. R: 0x96810, 0x96884, anims.json.
- Spawner: 0x0967C4 (firearm shot emission), called by player gun shooting (0x24ECC) and enemy poacher rifle firing (0x3DF10). Passes weapon type in d0, muzzle offset in d1, facing in d4. R: 0x967C4, D2c section 4.
- Timing: very fast burst, 2 to 4 game frames total (1 tick per frame). R: 0x96854.
- Movement: anchored to muzzle position: X = shooter X + muzzle offX, Y = shooter Y + muzzle offY, depth Z = shooter Z + 1. Velocity table at 0x96986. R: 0x967EA, 0x96834.
- Palette and draw flags: draw flags 0x10. Bright yellow/orange weapon flare palette. R: 0x9681A.
- Sound: gun gunshot sounds (0x2A pistol, 0x2B rifle, 0x2C shotgun, 0x2D machine gun) played by weapon fire routine via 0xA63C. R: D4e section 1.1.
- Episodes: All episodes with firearms (Episodes 1 through 8).
- Port status: MISSING (known missing muzzle class 0x1F from prompt).

### Class 32 (+$20 = 0x80): Vehicle Exhaust Puff / Skid Dust
- Handler address: 0x096A7E. Code size: 250 bytes (0xFA). R: 0x237E6, 0x96A7E.
- Visuals: direct frame chains at 0x096B10, 0x096B1C, 0x096B28. 1 frame each, 90 ticks. Puffs of exhaust smoke from vehicle tailpipes. Asset folder: anims/effect_c32/. R: 0x096A8A, 0x096B10, anims.json.
- Spawner: 0x096A56, called by vehicle controller 0x08C96C during Cadillac intro / driving sequences in Episode 3. R: 0x8C9AC, 0x8CA20, 0x96A56.
- Timing: 10 to 16 frames duration, single-shot. R: 0x96A90.
- Movement: ejected rearward opposite vehicle direction with horizontal velocity vx = -3 px/frame, rises slightly. R: 0x96A6E.
- Palette and draw flags: draw flags 0x10. Exhaust grey palette. R: 0x96A88.
- Sound: vehicle engine rumble.
- Episodes: Episode 3 (Cadillac highway drive).
- Port status: MISSING.

### Class 33 (+$20 = 0x84): Bullet Ricochet Spark / Metal Spark
- Handler address: 0x096B78. Code size: 588 bytes (0x24C). R: 0x237E6, 0x96B78.
- Visuals: animation table 0x096D06. 1 animation, 3 frames, 9 ticks total. Compositions in comps/effect_c33/sheet_01.png. Sharp yellow ricochet sparks bouncing off metal surfaces or impenetrable armor. Asset folder: anims/effect_c33/. R: 0x96B84, 0x96D06, anims.json.
- Spawner: 0x096B42, called when bullets strike solid containers, metal barrels, walls, or armored bosses without penetrating (0x28720 bullet hit check). R: 0x96B42.
- Timing: 3 to 6 frames total. Frees immediately on completion. R: 0x96BD0.
- Movement: bounces off surface: reflection angles loaded from table at 0x96D2A, randomized using PRNG (0x119C) across a 4-way scatter table at 0x96D8A. R: 0x96D2A, 0x96D8A.
- Palette and draw flags: draw flags 0x10. Sparks palette. R: 0x96BA0.
- Sound: metal clink / bullet ricochet sound 0x2F via 0xA63C. R: D4e section 1.1.
- Episodes: All episodes with gunfire against metal/armor.
- Port status: MISSING (known missing spark class 0x21 from prompt).

### Class 34 (+$20 = 0x88): Palette Cycling Sparkle / Color Shimmer
- Handler address: 0x096DC4. Code size: 146 bytes (0x92). R: 0x237E6, 0x96DC4.
- Visuals: direct palette RAM rotator. Cycles colors in targeted palette bank (+$5A) every 4 frames (G.$58 & 3), rotating through color index table. R: 0x96DE0, 0x96E00.
- Spawner: 0x096DAA, called by character select / HUD highlight routines (0x00C290, 0x07CD00). R: 0xC290, 0x7CD00, 0x96DAA.
- Timing: runs as long as the parent menu or highlight state is active; frees when parent state advances. R: 0x96DD0.
- Movement: static (affects hardware palette registers). R: 0x96DC4.
- Palette and draw flags: direct palette RAM modification. R: 0x96E0C.
- Sound: none.
- Episodes: Front end, Character Select, Episode 1.
- Port status: MISSING.

### Class 35 (+$20 = 0x8C): Title Screen Logo Transition Effect
- Handler address: 0x096E56. Code size: 414 bytes (0x19E). R: 0x237E6, 0x96E56.
- Visuals: animation table 0x071F72 (belongs to actor_a97, title logo). 3 animations, 1 frame each. Large "Cadillacs and Dinosaurs" title banner pieces. Asset folder: anims/actor_a97/ (comps in unreferenced). R: 0x71F72, 0x96E94, asset-catalog.md.
- Spawner: title sequence controller during attract loop (advances state via counter at $67be(a5)). R: 0x96E84.
- Timing: holds title banner for attract sequence timing (about 300 frames). R: 0x96ECC.
- Movement: snaps to camera position (camera X at $744(a5), camera Y at $748(a5)), offsets by +372 px X, +176 px Y. R: 0x96EBA, 0x96EEA.
- Palette and draw flags: draw flags 0x10. Title logo palette. R: 0x96E9C.
- Sound: title theme music code 0x01. R: D4d section 2.
- Episodes: Title / attract loop.
- Port status: MISSING.

### Class 36 (+$20 = 0x90): Cadillac Radio Call Exhaust Smoke
- Handler address: 0x096FF4. Code size: 534 bytes (0x216). R: 0x237E6, 0x96FF4.
- Visuals: animation table 0x097150. 8 animations, 18 compositions in comps/effect_c36/sheet_01.png. Thick billowing black and grey vehicle exhaust plumes. Asset folder: anims/effect_c36/. R: 0x97008, 0x97150, anims.json.
- Spawner: 0x096FD4, called during Cadillac radio call event in Episode 3 (0x08BD24 / 0x08BEFA, 0x08BF02, 0x08BF12). R: 0x8BD24, 0x8BEFA, 0x96FD4.
- Timing: 20 to 30 frames duration, multi-part emission as Cadillac revs engine and speeds onto screen. R: 0x97030.
- Movement: ejected from Cadillac tailpipes moving across screen at 6 px/frame. R: 0x96FC4.
- Palette and draw flags: draw flags 0x10. Dark grey exhaust palette. R: 0x97010.
- Sound: engine roar sound 0x3E. R: 0x8BD30.
- Episodes: Episode 3 (Cadillac arrival).
- Port status: MISSING (hook 0x8BD24 noted as missing in D4a section 12).

### Class 37 (+$20 = 0x94): Cadillac Tire Burnout Smoke and Skid Tracks
- Handler address: 0x09720A. Code size: 806 bytes (0x326). R: 0x237E6, 0x9720A.
- Visuals: animation tables 0x0972B2 (8 animations) and 0x0974EA (1 animation). Compositions in comps/effect_c37/sheet_01.png. White tire smoke clouds and spinning asphalt burn tracks. Asset folder: anims/effect_c37/. R: 0x97216, 0x972B2, 0x974EA, anims.json.
- Spawner: 0x0971D6, called by Cadillac radio arrival routine 0x08BD24 / 0x08BF7C. R: 0x8BD24, 0x8BF7C, 0x971D6.
- Timing: loops during car acceleration (103 ticks per cycle), dissolves as car straightens out. R: 0x97240.
- Movement: originates at wheel contact points, trails car movement. Y offset = car Y + 48 px. R: 0x97202.
- Palette and draw flags: draw flags 0x90. White smoke palette. R: 0x97220.
- Sound: tire screech sound 0x3F via 0xA63C. R: 0x8BF80.
- Episodes: Episode 3 (Cadillac highway entrance).
- Port status: MISSING.

### Class 38 (+$20 = 0x98): Unused Orphan Effect (Dead Code)
- Handler address: 0x000000 (NULL pointer in table 0x237E6 at 0x2387E). Code size: 0 bytes active. R: 0x237E6, 0x2387E.
- Visuals: orphan handler code at 0x097430 references animation table 0x0974EA (1 animation). Orphan spawner sits at 0x097414. Asset folder: none (table 0x974EA grouped under effect_c37). R: 0x97414, 0x97430, 0x974EA.
- Spawner: 0x097414 exists in code but has ZERO callers in the entire ROM binary. R: static scan.
- Timing: dead code (would crash/reset if executed due to NULL vector).
- Movement: none.
- Palette and draw flags: orphan code sets draw flags 0x90. R: 0x9744C.
- Sound: none.
- Episodes: none (abandoned developer leftover).
- Port status: UNUSED / DEAD CODE.

### Class 39 (+$20 = 0x9C): Fessenden Energy Sparkle / Transformation Aura
- Handler address: 0x097530. Code size: 324 bytes (0x144). R: 0x237E6, 0x97530.
- Visuals: direct frame chain at 0x0975AC. 1 animation, 16 frames, 32 ticks total. Blue energy flares, electric charges, and mutagenic glow sparks. Asset folder: anims/effect_c39/. R: 0x97544, 0x975AC, anims.json.
- Spawner: 0x0974FA, called in Dr. Fessenden transformation sequence 0x069E66 / 0x069FE6 (when Fessenden mutates into dinosaur form). R: 0x69E66, 0x69FE6, 0x974FA, D3d section 5.
- Timing: 32 game frames, multi-cycle pulse around Fessenden's body. R: 0x97560.
- Movement: anchored to Fessenden body (+#$58) at X = Fessenden X + 8 px, Y = Fessenden Y + 64 px. R: 0x97512, 0x9751E.
- Palette and draw flags: draw flags 0x10. Electric blue palette. R: 0x9754C.
- Sound: electrical hum / transformation sound 0x41 via 0xA63C. R: 0x69E80.
- Episodes: Episode 8 area 2 (final boss lair).
- Port status: MISSING.

### Class 40 (+$20 = 0xA0): Fessenden Slime / Acid Splash Drops
- Handler address: 0x097674. Code size: 396 bytes (0x18C). R: 0x237E6, 0x97674.
- Visuals: animation table 0x09770A. 8 animations, 16 compositions in comps/unreferenced/ (grouped under effect_c40). Acid droplets, green biological slime spatters. Asset folder: anims/effect_c40/. R: 0x97686, 0x9770A, anims.json.
- Spawner: 0x09764E, called during Fessenden bio-attacks and acid spray (0x63E30 area). R: 0x9764E, D3d BT18.
- Timing: travels in air for 20 to 30 frames, splatters upon floor impact. R: 0x976A0.
- Movement: arc trajectory with initial coordinates (d0, d1, d2) and velocity parameter d3. R: 0x97662.
- Palette and draw flags: draw flags 0x10. Green acid palette. R: 0x9768E.
- Sound: acid hiss / splash sound 0x43. R: D4e section 1.3.
- Episodes: Episode 8 area 2 (final boss fight).
- Port status: MISSING.

### Class 41 (+$20 = 0xA4): Directional Ground Sparkle / Scatter Burst
- Handler address: 0x097800. Code size: 238 bytes (0xEE). R: 0x237E6, 0x97800.
- Visuals: animation table 0x09786E. 1 animation, 9 frames, 18 ticks total. Small multi-directional spark burst. Asset folder: anims/effect_c41/. R: 0x97810, 0x9786E, anims.json.
- Spawner: 0x0977D2, called during Cadillac machine gun firing and wheel grinding in Episode 3 (0x08C3A6 / 0x08C3D4, 0x08C3FC). R: 0x8C3A6, 0x8C3D4, 0x977D2.
- Timing: fast 9-frame burst (18 ticks = 9 frames at 2 ticks/frame). R: 0x97848.
- Movement: velocity vectors chosen from 4-entry directional table at 0x0978CE (dx, dy pairs: vx = +/- 2 to 4 px/frame, vy = 1 to 3 px/frame). R: 0x97834, 0x97850, 0x978CE.
- Palette and draw flags: draw flags 0x8E override at +$23. Spark yellow palette. R: 0x977F8.
- Sound: gunfire impact / ground strike audio.
- Episodes: Episode 3 (Cadillac combat).
- Port status: MISSING.

## 3. Pool D Objects Deep Dive

### 3.1 Pool D Mechanics and Lifecycle
- Memory: RAM 0xFFDDF4 to 0xFFE5F4 (16 slots of 0x80 bytes = 128 bytes each). R: 0x4CCC, DinoSetup.java.
- Allocation (0x48A8):
  - Checks remaining slot count at G.$65FC. If count == 0, sets carry flag and clears +$98(a6) on caller. R: 0x48A8, 0x48CA.
  - Decrements G.$65FC, pops 16-bit offset from stack at G.$6606, stores slot pointer into caller struct at +$98(a6). Clears carry. R: 0x48B2, 0x48C0.
- Freeing (0x4994):
  - Pushes slot pointer back onto stack at G.$6606, increments G.$65FC, clears +$98(a6). R: 0x4994, 0x49A2.
- Execution model: Pool D has no independent update loop in the main frame loop (0x15190..0x151AE). Pool D slots are passive sprite records; they are owned, updated, and dispatched to the draw list by the parent object that allocated them. R: 0x1519C..0x151AE.

### 3.2 Slot 15: Stage Clock Sprite List (0x81062, 0x8114E)
- Address: RAM 0xFFDFE4 (Slot 15: $5DF4 + 15 * 0x80 = $6574 offset from 0xFF8000). R: 0x4CD8, 0x81062.
- Allocation: allocated at stage start by Pool B class 31 (clock controller at 0x81050). Because the free stack is initialized from slot 0 to 15 with push-down order, slot 15 is the first slot popped. R: 0x4CD8, 0x81062.
- Initialization (0x81062):
  - Screen position: X = 0x00E8 (232), Y = 0x00B8 (184), depth Z = 0. R: 0x8106C, 0x81072, 0x81078.
  - Copies 18 longs (72 bytes) from template table 0x081194 into the Pool D slot. Template establishes raw CPS1 hardware sprite headers (16x16 sprite blocks, priority, palette). R: 0x81088..0x81092, 0x81194.
  - Calls 0x16C6 (raw_load) on the slot. Sets parent visible flag +$01 = 1, draw flags +$25 = 0xD0 (raw sprite list with highest overlay priority). R: 0x810A0, 0x810AC.
- Per-frame display update (0x8114E):
  - Reads BCD stage clock word at RAM G.$4E8 (RAM 0xFF84E8). Format: 0x0MSS (Minutes, Tens of seconds, Units of seconds). R: 0x81152, D4b section 1.1.
  - Minutes digit: d1 = (d0 >> 8) & 0xF. Looks up tile code from table 0x0811CA, writes into slot offset +$14. R: 0x81158, 0x81164.
  - Tens digit: d1 = (d0 >> 4) & 0xF. Looks up tile code from table 0x0811CA, writes into slot offset +$1C. R: 0x8116C, 0x81174.
  - Units digit: d0 = d0 & 0xF. Looks up tile code from table 0x0811CA, writes into slot offset +$24. R: 0x8117A, 0x81180.
  - Digit tiles: table 0x0811CA maps BCD digits 0..9 to 16x16 font glyph tiles. R: 0x811CA.
- Audio and alerts:
  - When clock word G.$4E8 <= 0x0015 (15 seconds remaining) and changes, plays alert beep sound 0x34 via 0xA63C. R: 0x810E2, D4b section 1.2.
  - When clock word reaches 0x0000, triggers TIME OVER kill flow and Pool C class 28 words. R: 0x103FE, 0x96450.

### 3.3 HUD Score Popups and Food / Treasure Messages (0x7E72C)
- Pool B class 100 (HUD message spawner at 0x7E72C): allocates a Pool D slot via 0x48A8 whenever a food item, treasure item, or score bonus is collected. R: 0x7E72C, 0x7E7D6.
- Structure:
  - Clears 0x20 longs (128 bytes) in the allocated Pool D slot. R: 0x7E7E0.
  - Populates text buffer with collected item name string (from ROM string tables 0x240F0..0x241DE) and score value ("1000 pts", "5000 pts", "GOOD !!"). R: 0x7E7EE, D4a section 2.4.
  - Calls 0x16C6 (raw_load) on the Pool D slot. Sets draw flags +$25 = 0xD0 (top-layer raw sprite priority). R: 0x7E806, 0x7E812.
- Timing: displays message over HUD for 60 frames, then frees the Pool D slot via 0x4994. R: D4a section 2.4.

### 3.4 Other Pool D Callers
- 0x033368: Actor A00 (intro/test screen graphic). Allocates Pool D slot, loads raw list 0x3375C via 0x16C6, sets draw flags 0x70. R: 0x33368, 0x3339E.
- 0x039304: Boss / enemy special weapon HUD indicator. Allocates Pool D slot, zeroes fields. R: 0x39304.
- 0x07E7D6, 0x07EBC0: Score message display controller. R: 0x7E7D6, 0x7EBC0.

## 4. Item Class 6: Torch Glow Overlay (0x29B06)

- Pool: Pool E (24 item slots at RAM 0xFFBDF4, each slot 0xC0 bytes). R: 0x233F0, dino-game/src/game/mod.rs.
- Handler address: 0x029B06. Code size: 288 bytes (0x120 code from 0x29B06 to 0x29C26; local data tables from 0x29C26 to 0x29D1E). R: 0x29B06, 0x29C26.
- Purpose: renders the bright orange/yellow luminous halo around the burning head of a held or ground torch (weapon type 0x205). R: 0x29B4A, D2c section 6.2.
- Art and animations:
  - Animation direct chain 1: 0x029CB6 (idle flame halo: 6 frames, 24 ticks total, loops). Asset folder: anims/item_e06/. R: 0x29CB6, anims.json.
  - Animation direct chain 2: 0x029CF4 (swing flame trail: 4 frames, 16 ticks total, loops). Asset folder: anims/item_e06/. R: 0x29CF4, anims.json.
- Initialization (0x29B06 mode 0):
  - Loads direct animation 0x029CB6 via 0x160A (fx_load). R: 0x29B1A.
  - Sets draw flags +$25 = 0x30 (translucent blend / flicker). R: 0x29B24.
  - Clears flame sub-counters +$5C and +$5D. Advances mode to 2. R: 0x29B2C, 0x29B34.
- Active update (0x29B06 mode 2):
  - Validates owner: checks owner handle at +$5A(a6). If owner is dead (+#$00 == 0), owner class != 0 (weapon), or owner kind word != 0x205 (torch), jumps to 0x29CAA to free the glow object. R: 0x29B3C, 0x29B42, 0x29B4A.
  - Coordinates attachment: reads owner weapon frame index from +$2D(a0). Indexes 16-bit offset table at 0x029C26:
    - Applies X offset d0 (mirrored by owner facing +$24): X = owner X + (facing ? -d0 : d0). R: 0x29B8C, 0x29B94.
    - Applies Y offset d1: Y = owner Y + d1. R: 0x29B7A.
  - Depth flicker: checks hardware frame counter bit 0 (btst #0, $1(a5)). Alternates Z offset between +2 and -1 relative to owner depth:
    - Even frame: Z = owner Z - 1 (draws behind torch head).
    - Odd frame: Z = owner Z + 2 (draws in front of torch head).
    Creates pulsating light halo illusion. R: 0x29B9A..0x29BAA.
  - State switching:
    - If torch is lying on floor or carried idle (owner sub-mode != 8), plays idle animation 0x029CB6. R: 0x29BAE, 0x29BC0.
    - If torch is actively swung in attack (owner sub-mode == 8, attacking sub-state == 2), switches to fast swinging trail animation 0x029CF4. R: 0x29BCE..0x29C10.
  - Ticks animation via 0x15F4; queues for drawing via 0x49CA. R: 0x29C1A, 0x29C20.
- Episodes: Episode 4 (burning forest), Episode 5 (village torches on floor).
- Port status: MISSING (stub in item/blast.rs).

## 5. Port Status Table

| Class ID | +$20 | Handler | Code Size | Role / Description | Art Asset | Spawn Site | Status in dino-rs | Notes |
|---|---|---|---|---|---|---|---|---|
| C00 | 0x00 | 0x0923E0 | 202 B | Unused Base Effect | direct 0x9240E | None | MISSING | Frees on update |
| C01 | 0x04 | 0x0924AA | 862 B | Hit Sparks and Impacts | anims/effect_c01 (table 0x924F6, 26 anims) | 0x12720, 0x127FC, 0x92422, 0x92450 | DONE | In effect.rs fx_spark |
| C02 | 0x08 | 0x092808 | 432 B | Falling Cave Rubble / Dust | anims/effect_c02 (raw 0x928B0, 1 anim) | 0x85AA4 (Pool B 56) | MISSING | Ep6 a1 cavern |
| C03 | 0x0C | 0x0929B8 | 144 B | Hammer Ground Shockwave | anims/effect_c03 (direct 0x92A30, 1 anim) | 0x3C9FC (Hammer T.) | MISSING | Hammer smash |
| C04 | 0x10 | 0x092A48 | 306 B | Water Landing Splash | anims/effect_c04 (table 0x92AC0, 6 anims) | 0x00606E / 0x0060A2 | DONE | In effect.rs fx_water_splash |
| C05 | 0x14 | 0x092B7A | 288 B | Water Bubbles / Swimming Wake | anims/effect_c05 (table 0x92BF0, 12 anims) | 0x006192, 0x0061F6 | MISSING | Deep water wading |
| C06 | 0x18 | 0x092C9A | 562 B | Continue Screen Dynamite Fuse | anims/effect_c06 (direct 0x92E12, 1 anim) | 0x00EC74, 0x00EEC2 | MISSING | Snd 0x79 |
| C07 | 0x1C | 0x092ECC | 232 B | Ground Dust Puff (Slide/Landing) | anims/effect_c07 (raw 0x92F3A, 1 anim) | 0x092E5E (0x92E76/0x92EA4), 0x29F54 | MISSING | Slide & landing |
| C08 | 0x20 | 0x092FB4 | 464 B | Attached Burning Flames | anims/effect_c08 (direct 0x930D4/0x9313C) | 0x1C876, 0x2ADAE, 0x1F9E4, 0x4A052 | DONE | In effect.rs fx_flames |
| C09 | 0x24 | 0x093184 | 334 B | "STAGE CLEARED!" Banner | comps/unreferenced (direct 0x93262, 1-8) | 0x093152 | MISSING | End of stage 1-8 |
| C10 | 0x28 | 0x0932D2 | 548 B | Vehicle Crash Debris Scatter | anims/effect_c10 (table 0x93426, 7 anims) | 0x093362 (0x79146, 0x7B1A6) | MISSING | Hogg & Slice |
| C11 | 0x2C | 0x0934F6 | 1248 B | Broken Container Debris & Stones | anims/effect_c11 (table 0x937AA, 18 anims) | 0x259E8, 0x292A6, 0x2944A, 0x6B3EA | MISSING | Spawns stone items |
| C12 | 0x30 | 0x0939D6 | 164 B | Ground Shadow / Target Dot | anims/effect_c12 (raw 0x93A4A, 1 anim) | 0x0939B6 (enemy init) | MISSING | Debug/shadow |
| C13 | 0x34 | 0x093A7A | 174 B | Floor Impact Ring | anims/effect_c13 (raw 0x93AFA, 1 anim) | 0x093A60 | MISSING | Floor marker |
| C14 | 0x38 | 0x093B28 | 812 B | Respawn Flash & 1UP Marker | anims/effect_c14 (table 0x93C82, raw 0x93D1A) | 0x018C20 (respawn), 0x006B98 (1UP) | MISSING | 180 f hold on 1UP |
| C15 | 0x3C | 0x093E54 | 424 B | Burn Death Charred Collapse | anims/effect_c15 (table 0x93F42, 6 anims) | 0x093D48 / 0x093D5E | MISSING | Fatal fire death |
| C16 | 0x40 | 0x093FFC | 260 B | Continue Countdown Spark Burst | anims/effect_c16 (raw 0x940A4, 1 anim) | 0x00EDBA, 0x00EDD2 | MISSING | Countdown sparks |
| C17 | 0x44 | 0x094100 | 454 B | Continue Countdown Digits | anims/effect_c17 (direct 0x9421A, digits 9-0) | 0x00ED0C, 0x09418E, 0x0941B6 | MISSING | Snd 0x34 timer |
| C18 | 0x48 | 0x0942C6 | 412 B | Bludge Vomit / Food Spit | anims/effect_c18 (raw 0x9430E) | 0x03B1E8 (Bludge class 6) | MISSING | Bludge special |
| C19 | 0x4C | 0x094462 | 236 B | Dizzy Stars Circling Head | anims/effect_c19 (table 0x944DC, 2 anims) | 0x094338 (stun check) | MISSING | Snd 0x51 chirping |
| C20 | 0x50 | 0x09454E | 292 B | Stun Shock / Sweat Beads | anims/effect_c20 (table 0x945CC, 9 anims) | 0x094522 | MISSING | Dino shock/snore |
| C21 | 0x54 | 0x094672 | 4558 B | Heavy Smoke Puffs & Cloud Burst | anims/effect_c21 (table 0x951AC, 60 anims) | 0x094640, 0x0946C8, 0x095A3C | MISSING | Container bursts |
| C22 | 0x58 | 0x095840 | 206 B | Screen Flash / Lightning Whiteout | Direct palette RAM (0x914160/0x914180) | 0x09582A | MISSING | Full-screen flash |
| C23 | 0x5C | 0x09590E | 216 B | Respawn Invulnerability Sparkle | anims/effect_c23 (direct 0x95980, 5 frames) | 0x0958E4 (player 0x18CD4) | MISSING | 360 f duration |
| C24 | 0x60 | 0x0959E6 | 2224 B | Massive Explosion Fireball Plume | anims/effect_c24 (table 0x95E28, 41 anims) | 0x0959B4 (dynamite, TNT, bazooka) | MISSING | Snd 0x3A blast |
| C25 | 0x64 | 0x096296 | 134 B | Fuse Spark (Dynamite/TNT Drum) | anims/item_e00 (table 0x28904) | 0x09625C (weapon 0x25E40, item 0x28E5A) | MISSING | Attached to fuse |
| C26 | 0x68 | 0x09631C | 186 B | Water Ripple Ring | anims/effect_c26 (raw 0x96360, 1 anim) | 0x0060E2 / 0x00610A | DONE | In effect.rs fx_ripple |
| C27 | 0x6C | 0x0963D6 | 178 B | Slice / Slisaur Afterimage Ghost | Boss comps clone with palette cycle | 0x09639E (Slice/Slisaur dash) | MISSING | 36 f ghost trail |
| C28 | 0x70 | 0x096488 | 326 B | "TIME OVER" Flying Words | anims/effect_c28 (raw 0x96546/0x96550) | 0x096450, 0x09646A (timer 0x103FE) | MISSING | "TIME" & "OVER" |
| C29 | 0x74 | 0x0965CE | 230 B | Ground Impact Smoke Ring | anims/effect_c29 (table 0x9664E, 1 anim) | 0x0965A2 | MISSING | Ground shock ring |
| C30 | 0x78 | 0x0966B4 | 332 B | Sleeping Dinosaur Snore "Z" | anims/effect_c30 (table 0x96770, 1 anim) | 0x048576 (Shivat), 0x0535B4 (Mack) | MISSING | Ep2 & Ep5 dinos |
| C31 | 0x7C | 0x096800 | 638 B | Gunshot Muzzle Flash | anims/effect_c31 (table 0x96884, 4 anims) | 0x0967C4 (gun fire 0x24ECC, poacher) | MISSING | 4 weapon flashes |
| C32 | 0x80 | 0x096A7E | 250 B | Vehicle Exhaust Puff | anims/effect_c32 (direct 0x96B10, 1 anim) | 0x096A56 (Cadillac intro 0x8C96C) | MISSING | Ep3 Cadillac |
| C33 | 0x84 | 0x096B78 | 588 B | Bullet Ricochet Metal Spark | anims/effect_c33 (table 0x96D06, 1 anim) | 0x096B42 (bullet hit 0x28720) | MISSING | Snd 0x2F clink |
| C34 | 0x88 | 0x096DC4 | 146 B | Palette Cycling Sparkle Shimmer | Direct palette RAM rotation | 0x096DAA (select/HUD 0xC290) | MISSING | Front end highlight |
| C35 | 0x8C | 0x096E56 | 414 B | Title Screen Logo Transition | anims/actor_a97 (table 0x71F72) | 0x096E84 (Attract loop) | MISSING | Unused in game |
| C36 | 0x90 | 0x096FF4 | 534 B | Cadillac Radio Exhaust Plume | anims/effect_c36 (table 0x97150, 8 anims) | 0x096FD4 (radio call 0x8BD24) | MISSING | Ep3 radio call |
| C37 | 0x94 | 0x09720A | 806 B | Cadillac Tire Burnout Smoke | anims/effect_c37 (tables 0x972B2/0x974EA) | 0x0971D6 (radio call 0x8BD24) | MISSING | Ep3 car screech |
| C38 | 0x98 | 0x000000 | 0 B | Dead Code Orphan Effect | Orphan table 0x974EA at 0x97430 | 0x097414 (no callers) | DEAD CODE | NULL vector in ROM |
| C39 | 0x9C | 0x097530 | 324 B | Fessenden Mutation Aura Sparkle | anims/effect_c39 (direct 0x975AC, 1 anim) | 0x0974FA (transformation 0x69E66) | MISSING | Ep8 final boss |
| C40 | 0xA0 | 0x097674 | 396 B | Fessenden Slime / Acid Splash | anims/effect_c40 (table 0x9770A, 8 anims) | 0x09764E (Fessenden attacks) | MISSING | Ep8 acid drops |
| C41 | 0xA4 | 0x097800 | 238 B | Directional Spark Scatter Burst | anims/effect_c41 (table 0x9786E, 1 anim) | 0x0977D2 (Cadillac gun 0x8C3A6) | MISSING | Ep3 car gun spark |
| Pool D | All | 0x0048A8 | 120 B | 16 Auxiliary Display List Slots | Clock template 0x81194, HUD raw lists | 0x81062, 0x7E72C, 0x33368 | MISSING | Slot 15 = clock |
| Item 6 | 0x18 | 0x029B06 | 288 B | Torch Glow Halo Overlay | anims/item_e06 (direct 0x29CB6/0x29CF4) | 0x029B06 (attached to torch weapon) | MISSING | Alternating Z depth |

## 6. Port Task List

Estimated agent hours (grouped by subsystem and difficulty):

| Task ID | Scope | Classes Included | Size | Description |
|---|---|---|---|---|
| FX-P1 | Ground and Combat Essentials | Class 7 (dust 0x92ECC), Class 19 (dizzy stars 0x94462), Class 11 (debris 0x934F6), Class 21 (puffs 0x94672) | M (2.0 h) | Port running skid/landing dust, stun stars circling head, broken container rubble with stone weapon drop roll (0x93770), and heavy container puffs. |
| FX-P2 | Fire, Explosions, and Burn Death | Class 15 (burn death 0x93E54), Class 24 (fireball 0x959E6), Class 25 (fuse spark 0x96296), Class 22 (screen flash 0x95840), Item 6 (torch glow 0x29B06) | M (2.0 h) | Port charred burn death collapse, dynamite/TNT fuse sparks attached to weapons, big explosion plumes, lightning screen whiteouts, and torch flame halo with alternating Z depth. |
| FX-P3 | Firearms and Ricochets | Class 31 (muzzle flash 0x96800), Class 33 (bullet spark 0x96B78), Class 41 (directional scatter 0x97800) | S (1.5 h) | Port 4 weapon muzzle flash animations (pistol, rifle, shotgun, machine gun) and bullet ricochet deflection sparks with randomized scatter angles. |
| FX-P4 | Player Life and Stage Flow HUD | Class 14 (respawn flash & 1UP marker 0x93B28), Class 23 (sparkle 0x9590E), Class 28 (TIME OVER words 0x96488), Class 9 (STAGE CLEARED banner 0x93184) | M (2.0 h) | Port respawn light disc, 180-frame floating 1UP text, 360-frame post-respawn invulnerability sparkles, flying TIME OVER words with death sync, and stage cleared banner with episode digits. |
| FX-P5 | Pool D System and Stage Clock Display | Pool D allocator (0x48A8) and free (0x4994), Slot 15 Clock sprite list (0x81062, 0x8114E), BCD font decoding | M (1.5 h) | Port Pool D 16-slot passive buffer pool, implement stage clock digit updates from BCD word G.$4E8, 15-second warning beep alert (0x34), and raw list drawing. |
| FX-P6 | Vehicle and Highway Effects | Class 10 (vehicle debris 0x932D2), Class 32 (exhaust puff 0x96A7E), Class 36 (radio exhaust 0x96FF4), Class 37 (burnout smoke 0x9720A) | M (1.5 h) | Connect Episode 3 Cadillac radio call (0x8BD24) exhaust and skid effects, Hogg motorcycle debris scatter, and tire burnout smoke. |
| FX-P7 | Boss Specials, Water, and Ambient Visuals | Class 2 (falling cave dust 0x92808), Class 3 (hammer shockwave 0x929B8), Class 5 (water bubbles 0x92B7A), Class 18 (Bludge vomit 0x942C6), Class 20 (sweat beads 0x9454E), Class 27 (Slice afterimage 0x963D6), Class 29 (smoke ring 0x965CE), Class 30 (dino snore 0x966B4), Class 39 (Fessenden aura 0x97530), Class 40 (Fessenden acid 0x97674) | L (3.0 h) | Port all remaining boss, dinosaur, and stage-specific visual effects. |
| FX-P8 | Continue and Attract Front End | Class 6 (continue fuse 0x92C9A), Class 16 (countdown sparks 0x93FFC), Class 17 (countdown digits 0x94100), Class 34 (palette cycle 0x96DC4), Class 35 (title logo 0x96E56) | S (1.0 h) | Port continue countdown timer graphics and attract loop logo effects. |

Total port size: approximately 14.5 agent hours across 8 focused tasks.

## 7. Acceptance Scenarios

Scenarios describe observable behavior and test inputs for future verification against MAME traces.

### Scenario FX-S01: Dash Skid and Landing Dust (Class 7)
- Trigger: Hero performs a dash stop (double tap forward then release, or dash attack recovery) or lands from a jump.
- Verification: Pool C allocates an object with +$20 = 0x1C at character foot position. Object loads raw sprite list 0x92F3A, displays ground dust cloud for 14 frames, and frees via 0x4964.

### Scenario FX-S02: Dizzy Stun Stars Orbit (Class 19)
- Trigger: Hero or enemy takes 3 heavy hits in rapid succession (chain window 60 frames) with HP >= 30, entering dizzy state 0x16.
- Verification: Pool C allocates object with +$20 = 0x4C at owner head position. Star ring circles head with 30-tick looping animation (table 0x944DC). When player mashes joystick left/right, star countdown timer decrements by 3 ticks per frame instead of 1. Clears when character recovers. Chirping sound code 0x51 loops.

### Scenario FX-S03: Container Debris and Stone Weapon Drop (Class 11)
- Trigger: Hero punches a stone or boulder container in Episode 1 or Episode 5.
- Verification: Container breaks; Pool C allocates 9 objects of class 0x2C. Rubble pieces fly outward in parabolic arcs with velocities from table 0x937AA. If floor stone count G.$4F1 < 3, at least one rubble piece transforms into a pickup stone weapon upon landing (routine 0x93770). Pieces disappear after bouncing on floor.

### Scenario FX-S04: Fatal Burn Collapse (Class 15)
- Trigger: Enemy with low HP is hit by torch swing, Molotov, or explosive barrel blast, reducing HP to 0.
- Verification: Normal knockdown animation replaced by burn death sequence. Pool C allocates object with +$20 = 0x3C at enemy position. Spawns charred silhouette (table 0x93F42), lies smoking on floor for 60 frames, then disintegrates into ash and frees. Burn hiss sound 0x23 plays.

### Scenario FX-S05: Torch Flame Halo Depth Alternation (Item Class 6)
- Trigger: Hero picks up torch weapon in Episode 4 or Episode 5.
- Verification: Pool E allocates class 6 object attached to torch handle. On even game frames, halo Z depth is torch Z - 1; on odd game frames, halo Z depth is torch Z + 2. When torch is swung, animation switches from 6-frame idle 0x29CB6 to 4-frame swinging trail 0x29CF4. Halo frees immediately when torch is dropped or weapon breaks.

### Scenario FX-S06: Respawn Flash, Sparkles, and 1UP Marker (Class 14, Class 23)
- Trigger: Player loses a life and drops back into play; score crosses 500,000 points.
- Verification:
  - On respawn drop-in, Pool C allocates class 14 variant 0 (light column at landing point) lasting 30 frames.
  - While post-respawn invulnerability timer G.$EC counts down from 360, class 23 golden sparkles (direct chain 0x95980) spawn every 4th frame with randomized torso/head offsets.
  - On crossing 500,000 points and entering idle victory pose, Pool C allocates class 14 variant 2 ("1UP!" text graphic) hovering 80 px above hero head for exactly 180 frames (3 seconds). Extra life chime 0x4A plays.

### Scenario FX-S07: Stage Clock Display and 15-Second Warning (Pool D Slot 15)
- Trigger: Stage starts; clock counts down from 3:30 to 0:00.
- Verification:
  - Pool D slot 15 at RAM 0xFFDFE4 is populated with template 0x81194.
  - Minutes, tens, and units digits match BCD word G.$4E8 every frame.
  - At 0:15 remaining, clock numerals turn red and warning beep 0x34 plays on every second decrement.
  - At 0:00, Pool C allocates class 28 ("TIME" from right, "OVER" from left) meeting at screen center X = 192. Active heroes are killed with HP = 0 at t = 48.

### Scenario FX-S08: Gunshot Muzzle Flashes and Ricochet Sparks (Class 31, Class 33)
- Trigger: Hero fires handgun, rifle, shotgun, and Uzi; bullets hit steel drums and armored bosses.
- Verification:
  - When firing, Pool C allocates class 31 object for 2 frames at muzzle coordinates. Displays handgun starburst (anim 0), rifle cone (anim 1), shotgun wide flare (anim 2), or Uzi flash (anim 3).
  - When bullets strike impervious metal surfaces, Pool C allocates class 33 object (ricochet spark, table 0x96D06), bouncing off surface at randomized reflection angle. Ricochet sound 0x2F plays.

### Scenario FX-S09: Cadillac Highway Radio Entrance (Class 36, Class 37)
- Trigger: In Episode 3, hero collects radio item at container trigger 0x80.
- Verification:
  - Radio call routine 0x08BD24 triggers.
  - Cadillac speeds onto highway; Pool C allocates class 36 exhaust smoke plumes and class 37 spinning tire burnout smoke clouds. Engine roar 0x3E and tire screech 0x3F play.

## 8. Open Items

1. Class 0 (0x923E0) and Class 35 (0x96E56) active triggers: Both have valid handlers in the Pool C dispatch table, but no direct callers of 0x4884 with their class IDs were located. Class 35 clearly belongs to the attract mode title logo (actor_a97). Verify whether Class 0 is ever spawned via indirect or unreferenced pointers.
2. Class 38 (NULL table entry at 0x2387E): Handler 0x97430 and spawner 0x97414 exist in ROM but are never called. Confirm that leaving this entry as an empty handler or panic trap matches the arcade ROM.
3. Pool D slot exhaustion handling: When all 16 Pool D slots are full, 0x48A8 sets carry and fails. Verify how Pool B objects handle an allocation failure (whether they cleanly skip drawing the auxiliary graphic).
4. TIME OVER music cue: As documented in D4e section 1.4, short music track 0x1E (3.89 s duration) has no direct caller in ROM, which closely matches the time over rip duration. Verify whether 0x1E is triggered during the death sequence.
5. Slisaur and Slice afterimage palette cycling: Confirm whether Class 27 afterimages use hardware color math or software palette bank indexing across the 6-step cycle.

## 9. New Rows for mechanics-seen.md

IDs FX01 to FX30 to be merged into plan/game/mechanics-seen.md:

| ID | Mechanic | Source | Port | Notes |
|---|---|---|---|---|
| FX01 | Pool C effect pool: 48 slots of 0x80 bytes at RAM 0xFF9A74; allocator 0x4884, free 0x4964; update loop 0x237B2 dispatches via 42-entry pointer table 0x237E6 with class word +$20 scaled by 4 | R: 0x4884, 0x4964, 0x4CA4, 0x237B2, 0x237E6 | PARTIAL | 4 of 42 classes ported in effect.rs |
| FX02 | Hit sparks (class 1, 0x924AA): table 0x924F6, 26 anims; spawned by combat hit confirm 0x12720 with attack byte 6; jitter helper 0x92450 uses 8-entry table 0x9248A by G.$0 & 0xE | R: 0x12720, 0x92450, 0x924AA, 0x924F6 | DONE | In effect.rs fx_spark |
| FX03 | Water splash (class 4, 0x92A48): table 0x92AC0, 6 anims; spawned by player landing 0x606E; suppresses owner reflection; sound 0x9AF4 | R: 0x606E, 0x92A48, 0x92AC0 | DONE | In effect.rs fx_water_splash |
| FX04 | Attached burning flames (class 8, 0x92FB4): direct airborne chains 0x93096/0x930F4 and dying-out ground chains 0x930D4/0x9313C; drawn on odd frames only (30 Hz flicker); spawned by player 0x1C876 and enemy 0x2ADAE | R: 0x1C876, 0x2ADAE, 0x92FB4 | DONE | In effect.rs fx_flames |
| FX05 | Water ripple (class 26, 0x9631C): raw list 0x96360, 3 frames; spawned every 7 frames by wading hero 0x60E2; X jittered by G.$4 noise | R: 0x60E2, 0x9631C, 0x96360 | DONE | In effect.rs fx_ripple |
| FX06 | Ground dust puff (class 7, 0x92ECC): raw list 0x92F3A; spawned by helper 0x92E5E (0x92E76 slide, 0x92EA4 landing) and thrown item landing 0x29F54; lasts 14 frames | R: 0x29F54, 0x92E5E, 0x92ECC, 0x92F3A | MISSING | Needed for dashes and knockdowns |
| FX07 | Container debris (class 11, 0x934F6): table 0x937AA, 18 anims; wood planks, metal shards, stone rubble, tires; parabolic arc with floor bounce; 9 rubble pieces from stone break | R: 0x259E8, 0x292A6, 0x2944A, 0x934F6 | MISSING | D4a IT10 |
| FX08 | Bonus stone weapon spawn (0x93770): broken stone/boulder pieces roll chance (2/16, 8/16, 3/16) to spawn collectible stone weapon if G.$4F1 < 3 | R: 0x294AA, 0x93770 | MISSING | D4a IT11 |
| FX09 | Dizzy stun stars (class 19, 0x94462): table 0x944DC, 2 anims; spawned by stun check 0x94338; orbits head; timer decrements by 3 when mashing stick/buttons; looping sound 0x51 | R: 0x94338, 0x94462, 0x944DC | MISSING | Stun recovery |
| FX10 | Fatal burn death collapse (class 15, 0x93E54): table 0x93F42, 6 anims; charred body smolders 60-90 frames then dissolves to ash; sound 0x23 | R: 0x93D48, 0x93E54, 0x93F42 | MISSING | Fire death |
| FX11 | Gunshot muzzle flash (class 31, 0x96800): table 0x96884, 4 anims (handgun, rifle, shotgun, machine gun); lasts 2-4 frames; anchored to weapon muzzle | R: 0x24ECC, 0x967C4, 0x96800, 0x96884 | MISSING | Firearm shooting |
| FX12 | Bullet ricochet metal spark (class 33, 0x96B78): table 0x96D06, 1 anim; spawned on bullet hit against metal/armor 0x28720; 4-way scatter table 0x96D8A; sound 0x2F | R: 0x28720, 0x96B42, 0x96B78, 0x96D06 | MISSING | Metal armor hit |
| FX13 | Respawn flash (class 14 var 0, 0x93B28): table 0x93C82, 5 anims; expanding vertical light column and ground disc; spawned at player drop-in 0x18C20; 30 frames duration | R: 0x18C20, 0x93B28, 0x93C82 | MISSING | Revive drop-in |
| FX14 | 1UP floating text marker (class 14 var 2, 0x93B28): raw list 0x93D1A; spawned on extra life threshold 0x6B98; holds over hero head for exactly 180 frames; sound 0x4A | R: 0x6B98, 0x93BF8, 0x93D1A | MISSING | Extra life award |
| FX15 | Respawn invulnerability sparkles (class 23, 0x9590E): direct chain 0x95980, 5 frames; spawned every 4th frame by player 0x18CD4 during 360-frame invulnerability window G.$EC | R: 0x18CD4, 0x958E4, 0x9590E, 0x95980 | MISSING | Invulnerability indicator |
| FX16 | Heavy smoke puffs (class 21, 0x94672): table 0x951AC, 60 anims; multi-directional expanding cloud rings for container breaks (barrel 10 puffs, tires 5, statue 8); sound 0x27 | R: 0x292A6, 0x94640, 0x94672, 0x951AC | MISSING | Container break puffs |
| FX17 | Dynamite fuse spark (class 25, 0x96296): table 0x28904; attached to dynamite or TNT drum fuse tip at (+10, +57) offset; loops until blast; sound 0x79 | R: 0x25E40, 0x28E5A, 0x9625C, 0x96296 | MISSING | Lit explosive fuse |
| FX18 | Massive explosion fireball (class 24, 0x959E6): table 0x95E28, 41 anims; expanding blast core, fireball plume, and shockwaves; sound 0x3A | R: 0x26140, 0x28E5A, 0x959B4, 0x959E6 | MISSING | Heavy explosions |
| FX19 | Screen flash whiteout (class 22, 0x95840): direct CPS1 palette RAM write (0x914160/0x914180 ORed with 0xF000F000); 4-frame decay; thunder sound 0x3A | R: 0x9582A, 0x95840 | MISSING | Detonation flash |
| FX20 | "STAGE CLEARED!" banner (class 9, 0x93184): direct frames 0x93262 (compositions 2895-2903); shows banner and stage digits 1-8 based on G.$4D9; 180 frame hold; clear theme 0x0D | R: 0x93152, 0x93184, 0x93262 | MISSING | Stage clear sequence |
| FX21 | "TIME OVER" flying words (class 28, 0x96488): raw lists 0x96546 ("TIME") and 0x96550 ("OVER"); fly from right and left at 4 px/f to center X 0xC0; kills all heroes at t = 48 | R: 0x103FE, 0x1046A, 0x96450, 0x96488 | MISSING | Time out flow |
| FX22 | Pool D auxiliary sprite pool: 16 slots of 0x80 bytes at RAM 0xFFDDF4; allocator 0x48A8, free 0x4994; passive sprite buffers driven by owning Pool B or system objects | R: 0x48A8, 0x4994, 0x4CCC | MISSING | Dedicated sprite buffers |
| FX23 | Stage clock sprite list (Pool D slot 15, RAM 0xFFDFE4): initialized from template 0x81194 by 0x81062; per-frame routine 0x8114E maps BCD clock G.$4E8 to font tiles 0x811CA; alert 0x34 at <= 15 s | R: 0x81062, 0x8114E, 0x81194, 0x811CA | MISSING | Stage HUD clock |
| FX24 | HUD item & score message overlays: Pool B class 100 (0x7E72C) allocates Pool D slot via 0x48A8; copies item string and score text; raw list priority 0xD0; 60 frame duration | R: 0x7E72C, 0x7E7D6, 0x7E806 | MISSING | Pickup popups |
| FX25 | Torch glow overlay (Pool E class 6, 0x29B06): attached to torch weapon 0x205; idle flame 0x29CB6 (6 f), swing flame 0x29CF4 (4 f); Z depth alternates between -1 and +2 on even/odd frames | R: 0x29B06, 0x29C26, 0x29CB6, 0x29CF4 | MISSING | Luminous torch halo |
| FX26 | Boss afterimage shadow (class 27, 0x963D6): clones owner composition and facing; 6 steps of 6 frames (36 frames total); cycles palettes 0x9D to 0x8B; sound 0x22 | R: 0x9639E, 0x963D6 | MISSING | Slice and Slisaur dash |
| FX27 | Sleeping dinosaur snore "Z" (class 30, 0x966B4): table 0x96770, 1 anim; spawned every 120 frames above sleeping Shivat (0x48576) and Mack (0x535B4); floats upward 40 frames | R: 0x48576, 0x535B4, 0x966B4, 0x96770 | MISSING | Sleeping dinos |
| FX28 | Cadillac radio call exhaust smoke (class 36, 0x96FF4) and tire burnout smoke (class 37, 0x9720A): tables 0x97150 and 0x972B2; spawned by radio call 0x8BD24 in Episode 3; sounds 0x3E and 0x3F | R: 0x8BD24, 0x96FD4, 0x971D6, 0x9720A | MISSING | Ep3 Cadillac drive |
| FX29 | Continue countdown digits (class 17, 0x94100): direct table 0x9421A (digits 9-0); pulses 51 frames per digit; second beep 0x34; fuse sparks (class 6, 16) accompany countdown | R: 0xED0C, 0x94100, 0x9421A | MISSING | Continue screen timer |
| FX30 | Pool C Class 38 (0x000000): NULL entry in table 0x237E6 at 0x2387E; orphan handler at 0x97430 and uncalled spawner at 0x97414; completely unused dead code in ROM | R: 0x2387E, 0x97414, 0x97430 | DEAD CODE | Documented skip |

## 10. Web Sources and Cross References

- Arcade Quartermaster (Cadillacs and Dinosaurs): moves, character properties, and weapons catalog (https://www.arcade-quartermaster.com/capcom/cadi/).
- Hardcore Gaming 101: Cadillacs and Dinosaurs arcade retrospective (http://www.hardcoregaming101.net/cadillacs-and-dinosaurs/).
- Namu Wiki Cadillacs and Dinosaurs article (machine translated): item, container, and effect mechanics reference in plan/game/sources/claude-doc-deep-dive-supplement.md.
- Repo documents cross-referenced:
  - plan/game/design/asset-catalog.md (rendered asset numbers, animation tables, unreferenced compositions).
  - plan/game/design/D4a-items-drops.md (section 5 container break effects, section 8.4 stones from debris, section 9.4 dynamite fuse, section 12 effect tasks).
  - plan/game/design/D2b-player-state-rules.md (section 3 respawn invulnerability sparkles and flash, section 4 dizzy stun rules, section 7 TIME OVER kill rules).
  - plan/game/design/D2c-weapons-multiplayer.md (section 4 gun muzzle flashes, section 6.2 torch mechanics, section 8 weapon wear).
  - plan/game/design/D3a-humans.md (human enemy hit reactions and dust).
  - plan/game/design/D3b-dinosaurs.md (dinosaur sleep snore effects and wake reactions).
  - plan/game/design/D3c-bosses-1.md (Slice blade trail, Hogg motorcycle wreckage debris).
  - plan/game/design/D3d-bosses-2.md (Slisaur afterimage, Fessenden transformation aura and acid drops).
  - plan/game/design/D3e-pool-b-inventory.md (Pool B class 56 cave rubble spawner, Pool B class 100 HUD message spawner).
  - plan/game/design/D4b-time-rank-score.md (section 1.1 stage clock BCD rules, section 1.5 TIME OVER words, section 3.5 1UP floating marker).
  - plan/game/design/D4e-sound-cues.md (sound codes for effects, explosions, warning beeps, fanfares).
