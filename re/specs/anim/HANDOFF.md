# Handoff: draw paths and effects (pool C / pool D)

Owner files: `dino-rs/crates/dino-game/src/game/draw.rs`, `game/effect.rs`,
`dino-assets/src/effect.rs`, `dino-extract/src/effect.rs`. Formats: `re/specs/anim/SPEC.md`.

## Done (compiles; checked in an isolated copy because other owners' files broke the shared build)

Draw (`draw.rs`), all from the disassembly:
- 0x141F8 draw-list entry with the grab-pair order (partner +$62), 0x14136 split body,
  0x14230/0x14234 frames, 0x14180 compound frames, 0x142BE/0x14494 compositions with culling,
  palette override on every path, 0x1439C/0x1463E wading clip (sets +$53 bit 7), 0x14BC6 water
  splash (phase step 0x13FE6 runs in `SpriteList::clear`), 0x147FE raw sprite lists, 0x14EC0 shadow pass.
- Debug hit box overlay: `g.sprites.debug_boxes` (F1, wired by main).

Effects (`effect.rs`): pool C update 0x237B2 (slot order, skipped when all 48 free), free 0x4964
(clears +$00..+$27 only), anim helpers 0x15E4/0x15F4/0x160A and 0x16C6/0x16AE.
Ported classes: 0x04 hit spark (0x924AA, table 0x924F6), 0x10 water splash (0x92A48),
0x20 burning flames (0x92FB4), 0x68 water ripple (0x9631C, raw lists).
Spawns: `spawn_hit_spark` 0x12720, `fx_spawn_hit_spark_alt` 0x127FC, `fx_spawn_spark_at` 0x92422,
`fx_spawn_spark_jitter` 0x92450, `fx_spawn_spark_simple` 0x128C8, `fx_spawn_water_splash` (0x606E),
`fx_spawn_water_ripple` (0x60E2), `fx_spawn_flame` (0x1C876 / 0x2ADAE loops).
Data in `assets.effects` (tagged frame ids `EFX_FRAME` / `RAW_FRAME`).
Screenshots: `re/out/shots/fx_harness_25/26/52/75.png` (harness lived only in a scratch copy).

## To do: remaining pool C classes (unported classes are freed on first update)

Handlers (table 0x237E6) and spawn sites found (`site -> spawner routine <- callers`):
- 0x00 0x923E0 static picture (mode 2 only draws; list 0x9240E). No spawn found.
- 0x08 0x92808 <- 0x85AA4 (0x859A6, 0x85B3A)
- 0x0C 0x929B8 <- 0x3C9FC (0x3C95E, enemy)
- 0x14 0x92B7A, no direct spawn found (look for +$20 written from a register)
- 0x18 0x92C9A <- 0xEC5E (0xEC18) and 0xEEC2: intro/title sequence
- 0x1C 0x92ECC trail/smoke <- 0x29F54 (prop/item, every other frame), 0x92E5E (11 callers:
  player 0x26A6/0x19DCC/0x1CA1A/0x1CB6C/0x1FB62/0x1FCB0, items 0x29E38/0x2AE8E/0x2AF62, enemy
  0x4A132/0x4A206), 0x92EA4 (0x53BD2, 0x5C6AC)
- 0x24 0x93184 <- 0x93152 (no callers found yet)
- 0x28 0x932D2, no direct spawn found
- 0x2C 0x934F6 debris pieces (bounces with 0x12DAC, may spawn a pool E item via 0x4818 at the
  end) <- 0x259E8, 0x292A6 (0x29794, 0x5BAAE), 0x2943C (0x2937E, 0x297C0, 0x5BACA),
  0x294AA (0x293CC, 0x29414, 0x297E6, 0x297FC, 0x5BD84), 0x6B3EA
- 0x30 / 0x34 0x939D6 / 0x93A7A <- 0x939B6 (8 enemy callers: 0x33866, 0x35A4C, 0x392FE, 0x425B6,
  0x4BE96, 0x4FE24, 0x5C83E, 0x61944)
- 0x38 0x93B28 <- player 0x18C1E (0x2342, 0x36F8, 0x3806, 0x19A38, 0x1AE1E, 0x1AF2C)
- 0x3C 0x93E54 <- 0x93D48 (0x2AC42, 0x49EE6)
- 0x40 / 0x44 0x93FFC / 0x94100 <- intro 0xED04 (0xEC34); 0x44 also <- 0x9416C (0x94156)
- 0x48 0x942C6 <- 0x3B1E8 (0x3B1D2)
- 0x4C 0x94462 <- 0x94338 (0x1CE82, 0x1FFC6, 0x2B23E, 0x4A4E2)
- 0x50 0x9454E <- 0x94522 (0x4DDFE)
- 0x54 0x94672 (large) <- 0x94640 (22 callers incl. 0x29442, 0x2981E, 0x51046, 0x5C6C8, 0x75202,
  0x7665A, 0x78A4E, 0x7B186) and 0x959B4
- 0x58 0x95840 <- 0x9582A (0xBC34, 0x7C6A8)
- 0x5C 0x9590E <- player 0x958E4 (0x18CE4)
- 0x60 0x959E6 (large; may drive scene routines: 0x13FA6, 0x23420, 0x23604, 0xA76 sleep) <- 0x959B4 (0x751FC)
- 0x64 0x96296 <- 0x9625C (0x28F06)
- 0x6C 0x963D6 <- 0x9639E (0x4C37C, 0x4C67E, 0x5CC68, 0x5CF6A)
- 0x70 0x96488 <- 0x96450 (collision code 0x1044A)
- 0x74 0x965CE <- 0x965A2 (player 0x378E, 0x37D0, 0x1AEB4, 0x1AEF6)
- 0x78 0x966B4 <- 0x9667E (0x48576, 0x535B4)
- 0x7C 0x96800 <- 0x967C4 (0x83F1A, 0x840B2, 0x96C0E)
- 0x80 0x96A7E, 0x8C 0x96E56: no direct spawn found
- 0x84 0x96B78 <- 0x96B40 (12 callers in 0x25xxx and 0x6Axxx, likely weapon shots)
- 0x88 0x96DC4 <- 0x96DAA (0xC290, 0x7CD00)
- 0x90 0x96FF4 <- 0x96FD4 (0x8BEFA, 0x8BF02, 0x8BF12); 0x94 0x9720A <- 0x971D6 (0x8BF7C)
- 0x98: table entry is 0 (null) but 0x97414 spawns it: check what the update does with it
- 0x9C 0x97530 <- 0x974FA (0x69FE6); 0xA0 0x97674 <- 0x9764E (0x6374C);
  0xA4 0x97800 <- 0x977D2 (0x8C3D4, 0x8C3FC, 0x8C608)
- Pool D (allocator 0x48A8, 16 x 0x80 at 0xFFDDF4): not analysed yet.

## Next steps

1. Analyse the classes above. Seven background analysis agents (groups A-G, split as in the list
   above, pool D with G) were stopped at wrap-up before writing any spec; only scratch notes exist
   in `<scratchpad>/fxspec/` (`api.md` = the Rust API the drafts should target, `spawn_sites.txt`,
   `work/`). Restart that analysis (one agent per group, specs + draft Rust into a scratch folder),
   then check each draft against the disassembly before porting.
2. Infrastructure the drafts expect (not written yet): `Effect.ext: [u8; 0x80]` with
   `u8/w/iw/l` accessors (move the current `owner58/owner_class/w5c/owner60/off_x/off_y` fields
   into it), `fx_load_full/fx_start_full/fx_tick_full` (0x1236/0x120E/0x121E on effect frames),
   `fx_floor_bounce` (0x12DAC = 0x12FE4 lookup + snap; the player port has 0x12FE4 privately in
   `player.rs`, ask main to share it), small data tables in `EffectData` (`words`/`bytes` by address),
   submodules `game/effect/<group>.rs`, and the new addresses in `dino-extract/src/effect.rs`.
3. G.$4 (LFSR at 0xFF8004): only the ripple jitter reads it; 0x9A014 clears it at game start and
   no caller of its update 0xA9A was found, so callers pass 0. Do not add it to `World` unless a
   spawn site needs it.
4. Verify with a harness kept out of the shipped path (`#[cfg(test)]` or a debug flag).

## Known issues

- Text sparks ("BOOM!") are mirrored when spawned with facing 1; this follows the original's facing copy.
- The ripple tile 0x4534 uses palette 0x1C; with stage 0 palettes it looks brown (stage dependent).
- Shadow pass: a negative shadow type makes the original jump into the inner loop's dbra with stale
  registers; the port skips the object (never observed).
- 0x12720 X: assumes the upper words of d0/d2 are 0 at the call (true for the 0x128F8 call chain).
