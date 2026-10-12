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
| `boot.rs` | power on to the title: the boot task, the boot screens, the head of the attract task, the title, the INSERT COIN blink task, the press start screen (M1-03) | 0xA69A, 0x9DCF6, 0x9DD80, 0x97C20, 0x97D84 (head), 0x9A3CE, 0x98874, 0x983EE, 0x978EE, 0x980B0, 0x110C |
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

## What M1-04 starts from (M1-03 is done: power on to the end of the title is exact)

`flow/boot.rs` is the whole path from power on to the title, equal to the MAME recording
`attract.mame.trc` in every record from 25 to 918 (tilemaps, the four palette banks, the sprite list,
layer control G.$28, title counter G.-$7022, sound codes): `mame_check.rs`
`power_on_boot_and_title_match_mame`. `cargo test -p dino-game power_on_report -- --ignored --nocapture`
lists the differing records by kind for any change (`DINO_LAST`, `DINO_FROM`).

* Slot 0x10 is `front.task`, one `Scene` whose state is the script position (`boot.rs`, `mod st`): the
  reset wait, the boot task 0xA69A (hi-score table at frame 42, the sound codes, the clear at 68, the three
  player inits with their three `rand()` calls at 70), the boot screens, then the attract task 0x97C20 head
  (`ATTRACT`, `PASS`) and the title 0x9A3CE (`TITLE_START` .. `TITLE_LOOP`, 600 passes, counter in
  `front.title` = G.-$7022). `flow_attract_task` runs it, then the blink task (`front.blink`, slot 0x20) and the
  press start task (`front.press`, slot 0x20). `flow_attract_start()` restarts the attract task (used after a
  game over).
* Stubs for M1-04 / M1-05 / M1-07: `st::PASS` runs the title routine of state 0 for every state and, when the
  title ends (`title_pass` returns false), starts the pass again; `front.attract_state` (G.-$7030) is written
  at boot and never advanced. Put the real state routines (0x97D84, 0x97CC0, 0x97D02, 0x97C94 with the ranking
  after a game over) at `st::PASS` and replace the restart after the title by the profile cards.
* `flow_press_start_screen()` is 0x983EE: the system task calls it (and kills `front.task` first, as the
  original kills the attract tasks). It replaces the blink task by the press start task 0x978EE (green blocks,
  the two fixed logo pieces). The call takes about 3 frames of CPU time in the recording: the task first runs
  3 frames after the call and its first object pass 2 frames after that (both are sleeps in the code).
  `press_start_screen_matches_mame` calls it at record 607 of `coinstart.mame.trc` (coin at video frame 600)
  and compares scroll2, scroll3, all palettes, the sprite list and the layer control from record 610 to 697;
  scroll1 (the prompts of the system task) and records 607 to 609 (the clear and the palette load are one
  call in the port) are not compared. M1-04 fits the call frame to the coin timing.
* The title logo is pool A class 98 (`enemy/class98.rs`, `enemy_spawn_logo(variant)`): variants 0 and 1 (title),
  2 and 3 (press start), 4 and 0x10 / 0x11 are ported too (unit tests in the file). It uses the animation table
  0x71F72 (added to `OBJECT_ANIM_TABLES`). The title tune 0x88 is queued by variants 0 and 4 at their 10th and
  18th pass.
* The Capcom sign 0x110C is `flow_capcom_sign(col, row)` (tile codes 0x113C in `FRONT_TABLES`, 3 x 16 cells,
  attribute 0x0A); the green of the background is `title_green` (0x980B0).
* Timing conventions found here (the sample of a MAME record is not the end of a task pass: a task that
  calls a short sleep right after the sample can run again before the next one, and a busy clear 0xAE0 spans
  about 1.3 frames): sleeps are written with the measured length (`wait(2, ...)` after each full clear), two
  places differ from a plain reading of the code and say so in comments (the first title pass follows the
  setup in the same frame; the fourth title block is half drawn at the end of the setup frame, the last 27
  cells come one frame later). A scene created while another scene runs in the same frame is ticked in that
  frame too: give it `sleep(n + 1)` to make it first run n frames later.
* The skip of the boot screens (attack or jump of player 1 or 2 after 30 frames of a screen) is in
  `boot_wait`; `boot_screens_skip_like_the_original` compares three recordings made with
  `re/harness/record.sh` (commands in the test). Video frame f of the probe batches is record f - 2; a press
  at probe frame f is seen one frame earlier than the script line `f` of the harness.
* Not done on purpose: records 0 to 24 (the reset code's clear and RAM test of the video memory: the port
  starts from the state at record 25), the debug flag and test mode switch of 0xA69A, the system task and
  skip-wait task creation, 0x9CFC6 (M1-04), the anti-tamper roll of the title (only its `rand()` is made).
* Known leftovers outside this task: `Game::tick` (`mod.rs`) still counts G.$0000 outside the game task; the
  original keeps 0 there until the game task (0xA914) or a demo runs, so the trace shows `global.frame`
  differing. `Flow::dip` is set before power on (the original cooks it at frame 68). `diff.py` reads the
  all-zero sprite buffer of the original (no end marker before the first sprite list) as 256 entries and
  reports "sprites differ" for records 25 to 319; the Rust comparison treats it as an empty list.
* The tables in `FRONT_TABLES` have the extents of appendix B of D4d; a table that is too short panics
  with its address when read ("ROM text table byte ... was not extracted"): extend the line in
  `dino-assets/src/frontend.rs`.
