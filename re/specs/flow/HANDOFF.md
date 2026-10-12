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
   comment (hud_update runs it only as a fallback). (`main.rs` calls `flow_boot()` when neither
   --stage nor --chars is given: done in T1.1.)
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

# Front end services (M1 task T1.1): what exists and how to use it for T1.2 to T1.8

Written for the tasks that port the attract loop (`plan/game/design/D4d-front-end-and-story.md`
sections 2 and 3). Everything below is in `dino-rs/crates/dino-game/src/game/flow/` unless said.

## What was added

| File | What | Original routines |
| --- | --- | --- |
| `scene.rs` | `Scene`: one task of the original as a state machine (state byte, sub state, free counter, kernel sleep counter) | task kernel 0x95A, 0xA76 .. 0xA8C, 0x9FA, 0x9E38C (skip-wait) |
| `text.rs` | string printers and the typewriter | 0xBB6 / 0xBAE, 0xC4C / 0xC44, 0xC9A, 0xCD2 / 0xCE4 / 0xCDA, 0x84F72, 0xE12 / 0xE0A, 0xE68, 0x84736 (+ 0x84E0C), 0x7EC20 / 0x7EC48 (0xC0A is `hud_print`, already in `hud.rs`) |
| `picture.rs` | page loaders, block printers, scene palette sets | 0x20454, 0x204CA, 0x20592, 0x98B86, 0x98BAE, 0x89E6 |
| `config.rs` | the settings file (`Config` named options, `Settings` = the three cooked bytes G.$777A..C) | 0xAC794 / 0xAC7BC (cooking), factory image 0xACBF4 |
| `front.rs` | the boot screens (the first screen of the new code) | 0x9DCF6, 0x9DD80, tail of 0xA69A |
| `mame_check.rs` | tests that compare the above with the MAME recordings (see "Checks") | |
| `../snd.rs` | the attract sound filter `snd_passes` / `Game::snd_heard` | 0x610..0x69E |
| `dino-assets/src/frontend.rs`, `dino-extract/src/frontend.rs` | data in the asset pack: scroll1 pages (0x138C48, blocks 0x15CFE8), scroll2 pages (0x145DC8, blocks 0x175378), the 6 scene palette sets (0x8A5E), and `FRONT_TABLES` (the ROM tables of appendix B of D4d as byte blobs) | |

Scroll3 pages are `assets.stages.scroll3_screen_cells` (table 0x177288, one table for stages and scenes).

## Running a scene script

A scene is a `Scene` in `Flow` (or in `Front`) plus one method with a `match` on its state. Call the
method from `flow_attract_task` (or the place where the original runs the task). Sleep timing is the
kernel's: `sleep(n)` in frame F runs the script again in frame F + n; `tick()` is called once per frame
and returns true when the script runs.

```rust
pub fn flow_my_scene(&mut self) {
    if !self.flow.front.my.tick() { return; }           // asleep or ended
    match self.flow.front.my.state {
        0 => { self.flow_clear_screen(); /* draw */ self.flow.front.my.wait(30, 2); }  // sleep 30, then state 2
        2 => { /* more */ self.flow.front.my.wait(60, 4); }
        _ => self.flow.front.my.kill(),
    }
}
```

* Loops of frames (title 600 passes, timelines to 0x4B0): use `scene.count`, `scene.sleep(1)` each pass.
* `jsr 0x95A` with d0 = 0 suspends (`sleep(0)`), `wake(n)` is 0x9FA. The kernel counter is a byte: the
  original's longer waits are loops (`dbra` around 0xA7E), write them as loops with `count`.
* The skip-wait task 0x9E38C (scripted control, attack or jump held): call `scene.speed_up()` for each
  sleeping scene at the end of the frame while G.$4D8 bit 0 and a present player's attack/jump is on.
  Not wired to anything yet (`G.$4D8` is not ported).
* Several tasks at once (system task, attract task, blink task, fade task): one `Scene` each, run in slot
  order in `flow_frame`. The fade task is `palette.rs` (`start_fade_task`, `fade.mask` is G.$44A).
* Examples: the boot screens in `front.rs` (`flow_boot_screens`, `boot_wait` = 0x9DD80 with its `dbra`
  loop written as sub states 0 to 3).

## Printing

All printers write scroll1 cells with the original VRAM addresses (base 0x90A000 = row 32; column +0x80,
row +4). Strings come from the ROM tables as byte slices:
`let s = self.assets.frontend.text.slice_from(0x988A4); self.text_bytes(s, true);`

* Byte strings with their own position (0xBB6 print, 0xBAE erase): `text_bytes(s, print)`. INSERT COIN is
  `text_bytes(rom 0x988A4, true)` and `false`.
* 0xC0A (attribute byte first, text at a1): `hud_print(s, a1)`.
* Word strings: `text_words_at(s, print)` (0xC4C / 0xC44), `text_words(s, a1)` (0xC9A).
* Big font (2 x 2 cells per letter): `text_big(s, None, true)` (0xCE4), `text_big(s, None, false)` (0xCDA),
  `text_big(s, Some(a1), true)` (0xCD2). Tall font: `text_tall(s)` (0x84F72, own position),
  `text_tall_at(s, print)` (0xE12 / 0xE0A), `text_tall_a1(s, a1)` (0xE68).
* Portrait and text by id (story panels, in-level dialogue): `text_portrait(a0, portrait, text_id)`
  (0x84736; portrait 0x63 = none, text id < 0 = no text).
* Typewriter (episode titles, movie crawl, ending): `flow_typer_spawn(text, period)` (0x7EC20) and call
  `flow_typers_step()` once per frame where the original loop runs the pool B update 0x23604;
  `flow_typer_busy()` says whether it is still typing. It is not a pool B object here (the original takes
  a slot of class 0x68): if an exact slot order is needed later, allocate a real one. Attack alone on
  player 1 (G.$16 = 0x10, changed since G.$1B) prints the rest at once. It reads the region word
  (`flow.region`, 2 = World) for the second cell pair of a letter.
* Quirks kept on purpose (all from the code): the erase version of the big font writes 0x20, 0x30, 0x21,
  0x31 (not four spaces); the marks of the glyph strings (0x2C2C ',', 0x2E2E '.', 0x5B5B '[', 0x5D5D ']',
  0x7E7E '~', 0x2829, 0x2727, 0x5E5E) are written with attribute 0 even when erasing; the typewriter
  writes attribute 0 and wraps its address into scroll1 rows 0..31 (`& 0x909FFF | 0x908000`).

## Pictures

* `flow_page1(a0, page)` 0x20454 (scroll1, 32 x 32 cells), `flow_page2(a0, page)` 0x204CA (scroll2, 16 x 16
  cells), `flow_page3(a0, page)` 0x20592 (scroll3, 8 x 8 cells); `a0` is the absolute VRAM address as in
  the original (0x90F000, 0x913800 ...). Pages: `picture_calls.txt` in `re/out/assets/stages/scenes/`.
* `flow_block2(a0, codes, attrs, width, height)` 0x98B86 and `flow_block3` 0x98BAE: `width` = d1 + 1,
  `height` = d2 + 1; the code lists get 0x5800 (scroll2) or 0x1C00 (scroll3) added. The lists are ROM
  slices (`slice_from(0x9A5E6)`).
* `flow_scene_palettes(set)` 0x89E6 (sets 0 to 5: boot end and title = 0, select = 1, ranking = 4); it
  leaves the brightness nibble 0, call `brightness_full()` (0x892C) or a fade after it.

## Settings and sound

* `Config` (named options, defaults = factory) -> `Config::cook()` -> `Settings` (G.$777A / B / C).
  `main.rs` reads the file (`--config`, `$DINO_CONFIG`, `dino-config.json`, `../dino-config.json`) and calls
  `game.flow_apply_config(config)` before the start; `--print-config` prints the factory file. The rank code
  reads `flow.dip.dip_b` (set into `rank.dip` by `flow_mode_init`). Readers for T1.8 / T2.1: `flow.dip.free_play()`,
  `coin_mode()`, `chute_mode()`, `continue_allowed()`, `demo_sound()` ...
* The sound filter does not change `Game::sound`: the game keeps queueing every code (the harness trace
  keeps them). The audio side reads `Game::snd_heard()` (codes 0xFFxx and 0x0030 always, the rest only in a
  game or with `demo_sound`).

## `main.rs`

Without `--stage` and `--chars` the game starts from power on (`flow_boot`). `--stage N` and `--chars a,b`
start directly in a stage as before. The headless harness run (`--trace`) does the same
(`harness::Start { boot, stage, chars }`): `dino-game --rom dino.zip --trace out.trc --frames 400` records the
power-on run with the same tilemap and palette regions as the MAME recordings.

## Checks

* `cargo test`: unit tests for the scene runner, every printer, the loaders, the config and the sound filter.
* `mame_check.rs` (needs `dino.zip` and the recordings `re/out/traces/attract.mame.trc` and
  `coinstart.mame.trc`, see `re/harness/README.md`; it passes silently when they are missing): real ROM data
  against MAME, cell by cell: boot screens (records 70, 100, 191, 250, 311, 312: tilemaps and all four palette
  banks), title (scroll2 blocks, INSERT COIN, the two small lines, the licence block), the four profile cards
  (0xBB6 and 0x84F72), the ranking pictures (palette set 4, scroll2 page 0x16, scroll3 page 0x7C), the select
  screen pictures (0x20454 with scroll1 pages 5 and 6, scroll2 pages 1 and 2, scroll3 pages 6 and 9, palette set 1),
  the slide blocks (0x98B86 8 x 6, 0x98BAE 4 x 3) and the first slide's tall texts.
* `re/tools/frontcmp.py MAME.trc PORT.trc OFFSET FRAME...`: the same comparison for a port trace
  (`--trace`) against a MAME trace, listing the differing cells and palette words.
* Not compared with MAME (unit tests only): 0xC4C / 0xC44, 0xC9A, 0xCD2 / 0xCE4 / 0xCDA, 0xE12 / 0xE0A / 0xE68,
  0x84736, the typewriter, `speed_up`. Check them against a real screen when a scene first uses them
  (the credit text of the press-start screen uses 0xCE4, 0xC4C, 0xBB6 and 0xE68).

## What T1.2 starts from

* The boot task is faked: `front::BOOT_BLACK` = 70 frames of black, then `flow_boot_screens`. Text 1 appears at
  record 70, text 2 at 191, the clear at 312 with palette set 0 and full brightness, exactly as in MAME. One
  known difference: at record 69 the original still has the reset palette in scroll1 palette 0 (the first
  two actions of 0x9DCF6 fall into one video frame there); the port clears it one frame earlier. T1.2 ports 0xA69A
  (sound codes 0xFF0B at 43 and 70, 0xFF00 at 70 and 317, the debug flag, the tasks) and replaces the wait.
* After the boot screens the scene ends and the screen stays blank: the next step is the title (0x9A3CE) with
  the INSERT COIN blink task (0x98874), which should be a second `Scene` created at the title start
  (`text_bytes` print and erase, 30 frames each).
* Data for the title is in `frontend.text` already (0x97A20.. 0x9A5E6 lists, 0x97F4B strings, 0x988A4);
  the title's own scroll2 blocks, INSERT COIN, the two small lines and the licence block match MAME
  (`mame_check.rs`). The Capcom sign (0x110C at title frame 30: 3 x 16 cells, attribute 0x0A) and the green
  background extras (0x980B0) are not done.
* The tables in `FRONT_TABLES` have the extents of appendix B of D4d; a table that is too short panics
  with its address when read ("ROM text table byte ... was not extracted"): extend the line in
  `dino-assets/src/frontend.rs`.
