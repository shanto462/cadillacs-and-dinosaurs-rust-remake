# Flow / HUD port: handoff

## Ported (dino-rs/crates/dino-game/src/game/hud.rs, flow.rs)
- HUD player blocks (face, lives, name, vitality bar), scores, rank text, extra life, low HP
  blink, enemy / player / food / treasure messages with the 2-step bar drain and the death blink,
  boss bar, 2-player layout variants. Screenshot: `re/out/shots/hud.png` (3 players).
- Tables: `dino-assets/src/text.rs` (`TABLES`), extractor `dino-extract/src/text.rs`.
- Flow: game start (mode 0), stage setup 0xA9BE, area start 0xAA0E, stage start 0xAC4E, intro
  HUD slide-in (stage 0 variant), the G.$4D1 stage clear switch, boss defeat 0xACEE, all camera
  hooks (0x12BE4 documented no-op), `start_stage` (direct start, used by --stage / --chars).

## Stubbed (TODO(port) in code)
- `flow/front.rs`: system task (coins, credits, start), title, press start, character select
  (mode 4 currently starts the stage at once).
- `flow/join.rs`: join / continue task 0x6C4A, continue mode 0x10 (currently game over).
- Clear task 0xB97E tally (waits 120 frames), intro states 0-2 (map, EPISODE title), mode 8
  cut-scenes, ending 0x1C, weapon display 0x2389C (needs player +$B4/+$B6).

## Next steps, in order
1. Main session: call `self.hud_objects_tail()` in `objects_update` at the `0x100DA, 0x1667E`
   comment (hud_update runs it only as a fallback); main.rs: call `flow_boot()` when neither
   --stage nor --chars is given.
2. Other ports call: `hud_hit` / `hud_hit_credit` (enemy hit sites), `hud_add_score`,
   `hud_life_bar(i)` after player HP changes, `hud_food` / `hud_treasure`, set `g.hud.boss`
   in boss inits, `flow_boss_defeated` (0xACEE), set `g.flow.b4e3` from the player scripts
   (then delete the TEMP in `flow_intro_task`). Lives / score / rank are in `g.hud.stats[i]`.
3. Port `join.rs` (0x6C4A states, strings 0x84D6.., positions in SPEC section 6 / fork notes).
4. Port `front.rs`: system task coin logic 0x9D4BE / 0x9D386 / 0x9D196, title 0x9A3CE, press
   start 0x983EE, select 0xAD46 (0xAF7C, 0xB06A, 0xB122, 0xB25E).
5. Clear task tally 0xB97E, intro map / episode title, weapon display.

## Key mapping needed (already wired by the main session)
G.$1A bit0-2 = Start 1-3, G.$20 bit0-2 = Coin 1-3. Optional: Service = G.$1A bit7. Do not map
Test (bit6: resets into test mode).
