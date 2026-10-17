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
- `flow/select.rs`: the character select (M2-03, M2-04) and the join task before the level. The system task is
  `flow/system.rs` (M2-02, complete); the attract loop, title and press start screen are done (`boot.rs`, `attract.rs`).
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
4. Select 0xAD46 (0xAF7C, 0xB06A, 0xB122, 0xB25E); the system task, title and press start are done.
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
| `config.rs` | the settings file `dino.cfg` (`Config` named options, `Settings` = the three cooked bytes G.$777A..C) | 0xAC794 / 0xAC7BC (cooking), factory image 0xACBF4 |
| `system.rs` | the system task: coin interrupt code, coins and credits, all coin and chute modes, free play, SERVICE, lockout and counter outputs, credit text and prompts, the start of a game (M2-02) | 0x9D048 .. 0x9D8EE, 0x9D4BE, 0x9CFC6 |
| `attract.rs` | the attract state byte, the routines of its states, the stub screens (M1-04) | 0x97C46, 0x97C94, 0x97D84, 0x97CC0, 0x97D02, 0x983D8 |
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
  `main.rs` reads the file (`--config`, `$DINO_CONFIG`, `dino.cfg` next to the ROM, `./dino.cfg`, `../dino.cfg`; plain `key = value` lines, see "M2-01 and M2-02" below) and calls
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

## What M1-05 and M2-02 start from (M1-03 and M1-04 are done)

Power on, the title, the attract state machine, the coin and the start of a game are exact against MAME.
`cargo test -p dino-game power_on_report -- --ignored --nocapture` lists the records that differ from a recording by
kind (`DINO_TRACE` picks the recording, default `attract3.mame.trc`; `DINO_LAST`, `DINO_FROM`).

## The recordings of the M1 tests

They are git-ignored, in `re/out/traces/`; each test skips when its file is missing. Plain recordings (record k =
video frame k + 2, an input script line `f` reaches the game at record `f`):
`DINO_CLOCK=video DINO_BOOT=re/harness/scripts/attract.txt DINO_SYNC=vblank:1 re/harness/record.sh SCRIPT NAME FRAMES`:
`attract` / `attract2` (13400 / 26500 records, `attract.txt`, see `re/harness/README.md`), `attract3` (29500 records, same
script), `boot_skip_a` / `boot_skip_b` (340 records, `98-101:A` and `228-231:2J`), `m104_free` and `m104_coin`
(3100 records; `m104_free` with `DINO_POKES=file` holding `100 FF846B 1 1`, an empty script; `m104_coin` the script
`448-455:C`). The flow scenario batch of M2-31 (`flow_c-title`, `flow_s-nocredit`, `flow_s-nocredit-demo`,
`flow_freeplay`) was recorded with `record.sh --batch`: a script line `f` reaches the game one record earlier there
(`f - 1`), the tests of `mame_check.rs` give the port the shifted input.

## M1-03: power on to the end of the first title

`flow/boot.rs` is the whole path from power on to the title, equal to `attract.mame.trc` in every record from 25 to
918 (tilemaps, the four palette banks, the sprite list, layer control G.$28, title counter G.-$7022, sound codes):
`mame_check.rs` `power_on_boot_and_title_match_mame`.

* Slot 0x10 is `front.task`, one `Scene` whose state is the script position (`boot.rs`, `mod st`, shared with
  `attract.rs`): the reset wait, the boot task 0xA69A (hi-score table and sound code at frame 41, the settings cooked
  at 68, the three player inits with their `rand()` calls at 70), the boot screens, the head of the attract task
  0x97C20 and the title 0x9A3CE (`TITLE_START` .. `TITLE_LOOP`, 600 passes, counter in `front.title` = G.-$7022).
  `flow_attract_task` runs it, then the blink task (`front.blink`, slot 0x20) and the press start task
  (`front.press`, slot 0x20).
* The title logo is pool A class 98 (`enemy/class98.rs`, `enemy_spawn_logo(variant)`): variants 0 and 1 (title),
  2 and 3 (press start), 4 and 0x10 / 0x11 are ported too (unit tests in the file). It uses the animation table
  0x71F72 (added to `OBJECT_ANIM_TABLES`). The title tune 0x88 is queued by variants 0 and 4 at their 10th and
  18th pass. The Capcom sign 0x110C is `flow_capcom_sign(col, row)` (tile codes 0x113C in `FRONT_TABLES`); the green
  of the background is `title_green` (0x980B0).
* `flow_press_start_screen()` is 0x983EE, called by the system task. It replaces the blink task by the press start
  task 0x978EE (green blocks, the two fixed logo pieces). The call takes about 3 frames of CPU time in the
  recording: the task first runs 3 frames after the call and its first object pass 2 frames after that.
* The skip of the boot screens (attack or jump of player 1 or 2 after 30 frames of a screen) is in `boot_wait`.
* Not done on purpose: records 0 to 24 (the reset code's clear and RAM test of the video memory: the port starts from
  the state at record 25), the debug flag and test mode switch of 0xA69A, the skip wait task 0x9E38C, the anti-tamper
  roll of the title (only its `rand()` is made).

## M1-04: the attract loop, the coin and the start

* `flow/attract.rs`: the attract state byte `front.attract_state` (G.-$7030) and the routines of the states
  0x97D84 (0, 4), 0x97CC0 (2), 0x97D02 (6), 0x97D42 (8), 0x97DBE (0xA), the pass 0x97C46 with the game over ranking
  0x97C94. The state is kept across games (`flow_attract_start` after a game over resumes it). The title is real; the
  profile cards, the demo, the story slides, the ranking and the movie are **stubs**: a cleared screen with a label
  that lasts exactly as long as the real screen did in `attract3.mame.trc` (constants `PROFILE_FRAMES`,
  `RANKING_FRAMES`, `DEMO_FRAMES`, `SLIDES_FRAMES`, `MOVIE_FRAMES` at the top of the file). Replace one stub at a time
  by the real screen of M1-05 (profiles), M1-07 (ranking), M1-08 (slides), M1-09 to M1-11 (movie), M1-12 and M1-13 (demo)
  and keep its length until the real screen reproduces it. The state byte equals the recording in every record to 28960
  (the change frames are video frames 2744, 6874, 9311, 13411, 15848, 19979, 22416, 26516, 28953), and every title of
  the ten passes equals it from its setup frame to its end: `attract_loop_state_and_titles_match_mame`.
* Measured differences between passes (the original's CPU time falls differently, the port does not model it): the
  frames between the setup of the title and its first pass, the cells of the four blocks written when the setup frame
  ends and the frame the blink task first runs are the table `TITLE_RUNS` in `boot.rs`; the first profile cards are 8
  frames shorter than the later ones, the first ranking one frame shorter. A clear of all layers 0xAE0 in the attract
  task is staged like the original's (sprites and scroll1, scroll2 a frame later, scroll3 one more):
  `flow_attract_clear`; the script waits 2 frames after it (`flow_attract_busy`). Waits longer than 255 frames are
  chained sleeps (`flow_attract_wait`: the kernel sleep byte holds 255).
* (moved to `flow/system.rs` by M2-02, see the last section) the system task 0x9D048 and the coin interrupt code 0x9D4BE (`flow_coin_isr`, run by `flow_frame`
  before the tasks; the port gets coins in `Game::w.coins` and start buttons in `w.system`, G.$1A bits 0 to 2): the
  debounce (two frames high and two low), the conversion with all coin modes (table 0x9D038, `flow_sys_init` = 0x9CFC6),
  the three chute modes, free play, the press start screen when a coin or credit is there, the start detection
  0x9D196 (a change of the start bit with enough credits), the game start: tasks of the slots 0x10 to 0x40 end, G.$4CC =
  1, sound 0x31, a 16 frame fade out (`start_fade_task`), a clear, the game task is created 18 frames after the press
  (G.$4D5 = 1, G.$46D = 0; `flow.game_pending` until then) and its mode 0 sets mode 4 after 4 more frames (22 after the
  press, `flow_mode_start`). The select (`flow_select_mode`) is still a stub: it gives the players that started the
  heroes Jack, Mustapha, Hannah and starts Episode 1 at once. M2-02 adds the credit text and the prompts (0x9D66A, 0x9D752;
  the press start screen has none), SERVICE, the coin lockout and counter outputs and the TEST button; it should
  replace `flow_sys_init` and the system task of `front.rs` by `system.rs`.
* Tests (`mame_check.rs`): `coin_and_start_match_mame` (A10: credit at record 458, the press start screen, the start at
  599, G.$4CC, the fade, G.$4D5 at 617, mode 4 at 621), `start_without_credit_does_nothing` (A11), `free_play_start_matches_mame`
  (A12), `wait_counter_matches_mame` (F35), `game_over_resumes_the_attract_state` (F25, port only).
* The wait counter G.$4F6 (`rank_waited_tick`, 0x1827A) counts only when the common pool has credits or free play is on
  (`tst.w $46a`), so with the factory settings (separate chutes) it stays 0. The system task also loses a pass whenever the
  attract task is busy over a frame boundary (`front.sys_skip`, and `PROFILE_LOST` for the profile cards): the free play
  recording shows 12 lost counts by record 3000 and the port reproduces them exactly. The game start 0x17F8A reads the
  counter only after the system task cleared it in the pass that creates the game task, so the starting rank band is always
  0 in the original (the counter feeds nothing): kept.
* `Game::tick` no longer counts G.$0000 outside the game task (the original keeps 0 through the boot and title); the settings
  `Flow::dip` are cooked at frame 68 like the original (`flow_apply_config` stores them for that).
* `diff.py` reads the all-zero sprite buffer of the original (no end marker before the first sprite list) as 256 entries and
  reports "sprites differ" for records 25 to 319; the Rust comparison treats it as an empty list.
* The tables in `FRONT_TABLES` have the extents of appendix B of D4d; a table that is too short panics
  with its address when read ("ROM text table byte ... was not extracted"): extend the line in
  `dino-assets/src/frontend.rs`.

## M1-05 and M1-06: the four profile cards

`flow/profiles.rs` is the routine 0x9A7E6 (the profile stub of M1-04 is gone, `PROFILE_FRAMES` and `PROFILE_LOST` too). It
runs inside the attract task: `front.task` holds the script position (`profiles::pst`, dispatched by an arm in `boot.rs`),
`task.count` the counter G.-$7018 and `task.sub` the table index G.-$7016; `front.stub` is `Stub::Profiles` while it runs.
`flow_attract_after_title` calls `flow_profile_start`; the routine ends in `st::PROFILES_END` (the pass of the counter 0x4B0),
where `attract.rs` goes on with the clear and the ranking stub as before.

* The handlers are chosen by the ROM tables 0x9A8E4 / 0x9A908 (counter values and handler offsets) exactly as the original does,
  one per pass. Cards: `flow_profile_card(0..3)` (clear scroll1, music 0x13 on the first card, label block, tall text, the four
  objects, the units). Fades: the synchronous 0x87BA / 0x879A loops (16 steps, one per frame, `fade.steps` and `fade.mask` are
  G.$44B and G.$44A); `palette.rs` keeps its own copy of the step, `flow_fade_bank_steps` is the same. The counter does not run
  while a fade or the clear sleeps (that is why 1200 counts are 1398 frames).
* The helpers 0x9AC86 and 0x9ACBA do nothing (their flags are never set). The checker moves by the object class 94 (scroll3 Y
  minus 2 per update, G.$7E8). `D5-errata.md` has the corrections of D4d.
* Objects: pool B 93 `prop/class93.rs` (the board: clears scroll3 twice, loads the board palette, draws four strips of tiles by
  address, so its first cells land in the last 8 cells of scroll2), pool B 94 `prop/class94.rs` (the scroll and its life), pool A
  100 `enemy/class100.rs` (variants 0 to 3 the portrait with OBJ palette 16, 4 to 7 the small hero; the sounds are the `+$2D`
  byte of the animation frames, queued in every update of the frame). Data: `FRONT_TABLES` (0x92158, 0xC5952, 0x6A724, 0xBB252,
  0xBB292, 0xBB3B2, 0xBB3F2) and the animation table 0x71F9E in `OBJECT_ANIM_TABLES`. B93 var 4 (a wider board) and its mode 2
  fade in (0x8824) are ported only as far as the init: nothing creates it yet (the movie may).
* CPU time, measured and written as waits (see the module comment): the pass after a card start is cut by the video frame
  boundary (`pst::CUT`, `pst::CUT_END`; B93 splits its first update in two with the model field `CUT_FIELD`), and the first run
  (attract.mame.trc) is one frame ahead of the later ones (attract3.mame.trc) at 8 places, `profile_first_run()`. The clear 0xAE0
  clears scroll3 in two frames when scroll3 has content: `CLEAR3_FIRST` cells in the first (any value from 1072 to 3576 equals the
  recording; the board's fourth strip is still there one frame).
* The check: `mame_check.rs` `profile_cards_run_like_the_original` compares all five runs of `attract3.mame.trc` record by record
  (tilemaps, palettes, sprite list, layer control, sound codes, the six scroll shadows, counter and index). The picture of the
  port now lags its state like the original's (see "What the screen shows" in the M1-07 section below).

## M1-07: the ranking screen, and what the screen shows (present.rs)

`flow/ranking.rs` is the routine 0x1516C with 0x15594 / 0x15748 / 0x15888 / 0x1591C (the stub of M1-04 and `RANKING_FRAMES`
are gone). It runs inside the attract task like the profile cards: `front.task` holds the script position (`ranking::rst`,
an arm in `boot.rs`), `task.count` is G.$7018. `flow_attract_ranking_start` (0x983D8, also the game over ranking 0x97C94)
ends the blink task and calls `flow_ranking_start`; the routine returns in `st::RANK_END` after pass 422 (frame S + 422 the
first time, S + 423 later).

* The draw is a list of writes in the original's order (`ranking_ops`: the label block, scroll2 page 0x16 four times,
  scroll2 Y 0x40, scroll3 page 0x7C eight times, the ten rows). The first frame after the call does `ranking_cut()` words of
  it (a cell is a code word and an attribute word, the frame boundary can fall between them), the next frame the rest, the end
  of the handler (G.$7018, the Cadillac) and the first pass. The cuts are measured (`CUTS`, by run; later runs by the screen
  before): `cargo test -p dino-game ranking_cut_report -- --ignored --nocapture` finds them in `attract3.mame.trc`.
* Objects: pool B 89 cloud manager `prop/class89.rs`, pool C 0x80 clouds `effect/fx_cloud.rs`, pool B 86 Cadillac
  `prop/class86.rs` (modes 0 and 2 as the ranking uses them; the ending uses the same handler with more sub states, not
  ported), pool A 17 the two heroes `enemy/class17.rs`. Data: `FRONT_TABLES` (font 0x15768, cloud lists 0x8CA2E, speeds 0x96B34),
  animation tables 0x8B5E6 and 0x42516 in `OBJECT_ANIM_TABLES`, the three cloud pictures in the extractor's `EFFECT_LISTS`.
* Random numbers: the clouds and the heroes draw them in every pass. The rankings after the story slides and the demo start
  with the state of G.$14 that those stubs do not play (M1-08, M1-12), so `ranking_runs_like_the_original` sets it from the
  recording in the frame the routine is called; the first ranking after power on needs no help.
* The check: `mame_check.rs` `ranking_runs_like_the_original` (all nine runs of `attract3.mame.trc`, record by record: tilemaps,
  palettes, sprite list, layer control, sounds, scroll shadows, G.$14 and G.$7018).

What the screen shows (`present.rs`, `camera.rs`, `draw.rs`, `vram.rs`): the original's picture lags its state. In a MAME
picture taken with the recording's sample of record k: the tile maps are those of record k (written at once), the palette that of
record k - 1, the scroll registers the camera shadows of record k - 2 (cpsa), and the sprites the list of the last build that
is 2 records old or older (a game that builds no list for a while, like the profile cards in a fade, keeps showing the last
one). `Game::tick` ends with `cam.latch_scroll()`, `sprites.latch()` and `pal.latch()`; `present::draw` uses `shown_scroll()`,
the shown sprite list (`SpriteList::submit`) and `shown_words()`. 0xAE0 (`SpriteList::wipe`) blanks the sprites from the next
frame. Checked against MAME pictures of the attract loop (71 profile frames, the title, stage 1): pixel exact except the frames
where the first run of the profile cards does two passes in one frame (its sprites and fade appear a frame early in MAME).

## M1-08 story slides (2026-10-13)

`game/flow/slides.rs` ports the slide script 0x988C0 (5 slides, 8 x 6 and 4 x 3 blocks, tall font captions,
4 fades, the hard cut, the 49-frame pan, music 0x24 / 0x25, layer control changes), 1779 frames from call to
return. No objects (B91 / B92 are not used) and no rand(). Test `story_slides_run_like_the_original` compares
both slide runs of `attract3` record by record. The ranking test still injects G.$14 where the demo (4) returns (the movie
injections went with M1-11; the demo is real since M1-12 but its levels are not complete, see the last section). Details and the D4d corrections: the module doc and
`plan/game/design/D5-errata.md`.

## M1-12 and M1-13: the demo (2026-10-13)

The last stub of the attract loop is gone: `flow/demo.rs` is the routine 0x99E88 (setup 0x99F10, frame loop, end) with the rows
(0x9A1E2 / 0x9A1EE), the 30 input streams (0xC869A..) and their playback (0x9A146, 0x9A0CE). The module comment has the timeline of
the setup frame by frame and the table of what is cleared. The state is `front.demo` (`Demo`: G.$757C, the playback state of the three
heroes, the pad bytes G.$77A4..6) and `front.demo_counter` (G.$757A). Data: the rows, the pointer tables and the streams were already in
`FRONT_TABLES` (M1-02a); new: 0x222B6 (the first camera event by stage and area, 32 bytes, read by 0x2228A).

* **RAM clear.** The port has no memory image, so `flow_demo_clear_ram` builds a fresh `Game` and moves into it what the original's
  sweeps leave alone (inputs, video registers, video memory, palettes, fade state, the system task and the attract task `front`, the
  settings, the high score table). A new state field that lives in RAM outside those ranges is cleared by it without any edit; one that
  lives in the kept ranges (G.$16..33, G.$44A..4AB, G.$6EDE..6F1D, G.$700E..735D, G.$757A, G.$777A..) must be added there.
* **Timing.** The setup takes about 10 frames of CPU time. The visible steps are waits of the measured length, in two variants: the first
  demo after power on (and the poked first demos of the recordings) and the later ones differ by a frame in the middle (`dst`, module
  comment); Episode 1 (row 0, the sixth demo) starts its first pass one frame earlier. The loop itself never loses a frame (1 game frame per
  video frame, G.$0 = 1 in A + 10, 1201 passes, the picture stays 100 frames, 16 fade steps, back in A + 1325 as the old stub did).
* **Input playback.** The pad bytes replace `w.input` for the object update only (`flow_demo_frame`): the player port reads G.$77A4..6,
  which is the cooked input in a game and the stream byte in the demo. No hook in `player.rs` or `input.rs` was needed. G.$16 itself is not
  changed (`lib_target_input`, 0x32B94, reads it; none of its callers is ported: when one is, give it the real input, `flow.demo` keeps none). The routine
  also writes 4 hex digits at the top left of scroll1 (0x18F8, a debug display of the hero number or FFFF) that are in every demo picture.
* **Camera hook.** `camera.rs`: `flow_demo_camera_init` is 0x2228A (event index from the table, state 2 at once: the palettes and the timers
  of the normal init 0x206EA are skipped, the pointer to the next event stays on the loaded one), and `reinit_map` draws the pre-draw of
  0x20520 now (areas that start at X = 0 draw their first screen a second time at +0xBC8; 256 cells at the end of scroll2, they were the
  unported cells of Episode 2 / 8). **Not done, known:** `write_cell3` skips areas without a far layer (the original writes 1C00/0010 cells
  hidden behind scroll2: 187 cells in Episode 1 area 1).
* **Start of a game after a demo / coin in the demo.** `front.rs`: the system task needs 3 frames for the call of the press start screen
  0x983EE (`sst::TAIL`: the rest of the pass, G.$4CD cleared, runs 3 frames later), and the clear 0xAE0 of the game start is staged like the
  attract task's (`flow_attract_clear`, scroll3 cut at 3587 cells, `Front::clear3_cut`). The press start call itself is still done at once (the
  recording shows it in three frames: sprites and scroll1, scroll2, scroll3 and palettes; also in the title case): the three records are
  excluded in the tests.
* **Checks** (`mame_check.rs`): `demos_set_up_and_run_in_step_with_mame` (5 demos of `flow_m113-long`), `demo_rows_5_and_0_...` (poked first
  demos), `attract_loop_32000_frames_match_mame` (A3), `coin_during_the_demo_matches_mame`, `start_during_the_demo_goes_to_select`;
  `demo_report` and `demo_input_report` (ignored) print the differences per demo (`DINO_TRACE`, `DINO_POKE_COUNTER`, `DINO_DEMO`, `DINO_CASE`).
  Recordings: `re/harness/scripts/m112_demo.txt` (+ `_pokes`) and `m113_long.txt` (+ `_pokes`), `record.sh --batch`, see the harness README.
* **What differs (all causes are things the port does not have yet):** the heroes of Episode 1 area 1, Episode 2, 3, 4, 5, 6 and 8 start in
  a scripted walk-in (G.$4DD = 7, 0x1DA10, M2-07 to M2-09) or the car scene (Episodes 2 and 4: the heroes are hidden until it ends), so the player
  objects differ from the first frame in position, state and timers, and the sprite list has the heroes where MAME has none; the objects of
  classes the port does not run draw random numbers, so G.$14 first differs 12 to 239 frames into each demo and at the end of every demo (the ranking
  test still sets it in the frame the demo returns); Episode 3 (stage 2) uses its own scroll tasks (0x22398 ...). `demo_report` lists the first frame
  of each kind per demo.

## M1-15 attract acceptance (2026-10-13): M1 done

`attract_loop_two_loops_match_mame` runs the port from power on for 26,600 records against `attract3`: all
21,276 records outside the demos equal MAME (tilemaps, palettes, sprite list, layer control, sounds, state byte,
scroll shadows, G.$14). `re/harness/allow/m1.txt` lists every remaining difference with its cause (the demo level
content until M10, single-frame picture latch differences in the first run, the movie shake until M3-10). The press
start call now takes 3 staged frames like the original; `write_cell3` writes the hidden scroll3 cells too (the
earlier "not done" note is obsolete). 369 of 416 compared video frames are pixel exact.

## M2-01 and M2-02: the settings file and the complete system task (2026-10-13)

**Settings file (M2-01).** `config.rs` reads `dino.cfg` (git-ignored; the JSON file `dino-config.json` of M1-01 is gone): one `key = value` per
line, `#` starts a comment, flags are on / off (also true / false, yes / no, 1 / 0), keys in any case. Keys and factory values: `coin_mode = 0`
(0 to 8), `chute_mode = 0` (0 to 2), `free_play = off`, `continue_allowed = on`, `demo_sound = off`, `stereo = on`, `difficulty = 3` (0 to 7),
`timer_level = 1` (0 to 3), `bonus_life = 2` (0 to 3: none, 300000 / 700000, 500000 / 1000000, 1000000), `start_lives = 1` (0 to 3). A missing file
or key is the factory value; an unknown or repeated key, a value out of range or a line without `=` is an error that names the line.
`dino-game --print-config` prints the factory file with the meaning of each key. Lookup: `--config`, `$DINO_CONFIG`, then next to the ROM
(`--rom`, `$DINO_ROM`), `./dino.cfg`, `../dino.cfg`. The cooked bytes (0x80, 0x4B, 0x41 for the factory file) feed `Flow::dip` (boot task, frame
68), `rank.dip` (`flow_mode_init`, the start rank and the rank speed), the extra life table 0x6BFE (`Flow::extend_sel`), the lives
(`flow_lives_init`), the sound filter (`snd_heard`) and the system task. Tests: `config.rs` (the default file gives dip_b 0x4B and the entry 2 of
0x6BFE, 500000 then 1000000; `difficulty = 0 / 3 / 7` gives start ranks easy < normal < hard in `rank_game_start`).

**System task (M2-02).** `flow/system.rs` is 0x9D048 to 0x9D8EE with the coin interrupt 0x9D4BE and the set-up 0x9CFC6; the minimal task of
M1-04 is gone from `front.rs` (which keeps the state `Front` and the select stub). One pass per frame in the original's order: SERVICE with
the conversion (0x9D326 / 0x9D386), the coin lockout bits of G.$21 (0x9D40E: 4, 8, 0x20 per player, 0x2C for the pool, off at the cap or in free
play), the coin counter pulse (0x9D478: G.$46E / G.$46F, bit 0 of G.$21 for 0x20 frames), the attract part 0x9D074 (the press start screen when a
coin or credit is there, the clear of the text rectangle, the credit text 0x9D66A, the prompts 0x9D752, the start detection 0x9D196, the game
start), the wait counter 0x1827A. All eight coin presets come from the table 0x9D038 (`FRONT_TABLES`), coin mode 8 is the flag G.$46C; the chute
modes are separate credits, the common pool (G.$472) and two players on one chute (G.$474 and the layout flag G.$4CF, set as 0x9CFF6 does; the
2 player layouts of the HUD, select and demo teams were never measured and are not touched). Free play: no press start screen, no credit text, a
start needs nothing. Credit cap 9. SERVICE (G.$1A bit 7, key 9 or `V` in an input script, a press only) adds a credit to every player (the pool
with one chute) at any time and plays the coin sound (without the sound stop that a coin makes). Not ported: the TEST button (0x9D5A4) and the
play time statistics 0xA6B8A (nothing visible; the test menu is out of scope).

* **What the screen shows** (strings read from the ROM at run time, `FRONT_TABLES` 0x9D984 now runs to its last string, and 0x9D666 is the 4 byte
  rectangle 0xF94 clears: columns 0 to 47, page rows 15 to 28). Separate chutes: a tall font prompt per player at 0x90A050 / 0x90A850 / 0x90B0D0
  (PUSH START BUTTON with a credit, INSERT COIN without, INSERT ADDITIONAL COIN and the line `nCOIN 1CREDIT` at +0x14 while coins wait) and the
  credit line at 0x90A0F0 / 0x90A8F0 / 0x90B0F0 (`CREDIT n`, `CREDIT n (c/2)` with 2 or more coins per credit, `COIN n` in coin mode 8; the
  digits are raw tile codes 0 to 9, code 0 is blank, so 0 credits show no digit; at 9 credits the pending coins show as blank). One chute: the big
  font lines in the middle (INSERT ADDITIONAL COIN and `nCOIN 1CREDIT`, PUSH START BUTTON with ONLY ONE PLAYER / UP TO TWO / UP TO THREE
  PLAYERS and the player mark; in coin mode 8 players = credits / 2 and, while an odd credit is left and fewer than 3 players (2 on the two player
  cabinet) are possible, "AND INSERT ADDITIONAL COIN(S) FOR ADDITIONAL PLAYER(S)") and one credit line in the middle column.
* **Quirk kept:** the start edge is any change of the start button (G.$1A xor G.$1F), the release counts like the press.
* **After the game task is created** (617 in `flow_c-three-start`) the first pass of the system task runs and the next two are lost (the first run
  of the game task takes the CPU for about 3 frames): `sst::AFTER`. Seen in the coin counter timer G.$46F.
* **Checks** (`mame_check.rs`, all equal to MAME in every record, the three layers, palettes, sprite list, layer control, sounds and 22 watched
  variables of the system task including all credits, coins, G.$21, G.$46E / G.$46F): `coin_and_start_match_mame` (A10, now with the scroll1 credit
  text), `credits_of_all_players_match_mame` (`flow_c-two`, `flow_c-three-start`, `flow_c-title-coinhold`), `coin_and_chute_modes_match_mame`
  (12 new segments `flow_system_modes`, see below), `free_play_start_matches_mame` (A12), `start_without_credit_does_nothing` (A11),
  `wait_counter_matches_mame`, `coin_during_the_demo_matches_mame`, `start_during_the_demo_goes_to_select`. The exclusions of scroll1 that M1 had for
  the credit text are removed (`re/harness/allow/m1.txt` listed none: the M1 loop has no coin).
* **New recordings** `re/harness/scripts/flow_system_modes.txt` + `re/harness/pokes/flow_system_modes.txt` (12 segments, one MAME process, 14 s):
  `re/harness/record.sh --batch re/harness/scripts/flow_system_modes.txt flow_system_modes re/harness/pokes/flow_system_modes.txt`. MAME has no switch
  for the coin and chute modes, so the segments poke the system task variables at power frame 100 (G.$46C, G.$470 / G.$471 as a word, G.$472,
  G.$474, G.$4CF, G.$46B): m-common, m-two-player, m-2coins, m-1coin2credits, m-1coin6credits, m-3coins, m-common-2coins, m-mode8, m-mode8-common,
  m-mode8-two-player, m-free-coin, m-cap. The port test takes both files and applies the same writes at tick 99.
* **Known differences (CPU time, not rules):** with one chute a pass of the system task is shorter, so in `m-common`, `m-two-player`,
  `m-common-2coins`, `m-mode8-common`, `m-mode8-two-player` the video frame boundary falls elsewhere in the code that follows: record 319 has 6
  more scroll2 cells of the title's first frame in MAME, record 460 (the first frame of the press start call) cuts the brightness loop 38 or 44
  palette words away from the measured factory cut, and record 462 shows the press start task's first sprite list a frame earlier (19 sprites).
  Coin ratios other than 1 : 1 move the cut of record 460 by 4 words (`m-2coins`, `m-3coins`, `m-cap`). `flow_c-title-coinhold` has 3 palette words
  different in record 483 for the same reason (a coin 25 frames later in the title than A10). Not tested against MAME: SERVICE (the probe has no
  key for it; logic from the code, unit tests, and the end to end test that it brings the screen of three coins).
* **Input:** `input.rs` maps key 9 (and the gamepad Mode button) to SERVICE, `V` in scripts; TEST (bit 6) stays unmapped.

## M2-03 and M2-04: the character select (2026-10-13)

`flow/select.rs` is game mode 4 (0xAD46: set-up 0xAD5E, main pass 0xAF7C, end 0xB810, the player step 0xB06A, the cursor 0xB122, the panels
0xB25E with 0xB2A0 / 0xB36A / 0xB3CA / 0xB3F8 / 0xB450, the picked state 0xB6A2) and the part of the join task 0x6C4A that runs before the level
(`select_join_task`: states 0, 2, 4 and 6 of a player in modes 0 and 4). The state is `Select` (`front.select`, the struct G.$62C: `bytes()` gives the
0x38 bytes of RAM). Pool B classes: 17 (the frame, `prop/class17.rs`, a raw list frame, `RAW_LISTS` 0x7B5C8), 67 (the two pictures over the group
picture, `class67.rs`), 68 (the player tags with their palette cycle, `class68.rs`) and 19 (the hero pictures of the 2 player layout, `class19.rs`,
not measured). Data: `FRONT_TABLES` (select tables, the prompt strings 0x83EB.., the palettes 0xBB2D2.., 0xBC052.., 0xBFE52.., 0xBFF92) and
`OBJECT_FRAMES` / `OBJECT_ANIM_TABLES` of `dino-extract/src/sprites.rs`.

* **Timeline** (record = video frame - 2, a start at 699 as in `flow_sel-1p`): the game task is created in 717, its first iteration counts in 718 (G.$0000 = 1; the
  port counts in 719: tolerated), mode 4 comes in 721 together with the first part of the set-up (layer control 0x06D6, the palette set 1), then
  the routine sleeps 2 frames (`Flow::resume`: the loop counters G.$0000 and G.$4DA do not count in a resumed run), the rest of the set-up and the first main
  pass are in 723 (sub 2, TIME 10, the pages, the music code 0). The join task first runs in 721 (state 0 -> 2, one random number) and 722 (the join init: one
  random number, its object is cleared, the prompt erased, state 4). A player's lock counter (+5) runs 10 frames from its first step and holds the timer at
  10; the number 10 stays up until 793 and 9 comes 60 frames later (not 735 as in D4d). The timer is printed when its 60 frame counter runs out, at 0 the three input
  bytes get an attack press (`select_main`), so every player who has not chosen picks in that frame (the press counts only when the player did not hold attack
  in the frame before: the original compares G.$16 with G.$1B, which already holds the injected value). The cursor reads press edges (left, right, attack,
  jump) and the start edge; moves are allowed during the lock, picks are not; a pick queues 0x33, makes the object present, +3 = hero id, +$20 = id * 4 and,
  for Jack and Hannah with 3 seconds left, draws one random number (the anti-tamper roll 0x5A454 passes). The picked state flashes the hero's palette for 60
  frames (6 palettes, 5 rounds of 2 frame steps), the select ends when nobody is busy and a record has finished its flash: sub 4, 16 palette steps (the
  first after 2 frames, then one a frame; the heroes that were not chosen go dark, the chosen ones bright), 60 frames of sleep, the fade out (0x8840) and the
  stage start 0xAC4E, which takes 4 frames after the fade is over (`flow_stage_start_staged`: the clear 0xAE0 in 3 stages, the palettes and the camera, the
  rest of 0xA9BE with the mode byte 4 again, then mode 0xC with the sound stop and the music): mode 0xC comes 159 records after the pick.
* **Checks** (`mame_check.rs`): `character_select_matches_mame`: `flow_sel-1p`, `sel-1p-moves`, `sel-same`, `sel-2p-late`, `sel-3p-all-left`,
  `sel-3p-timeout`, `sel-picks-sim`, every record from the boot to 3 records before the level (all three layers, the palettes, the sprite list, the layer control, the
  sound codes, G.$14, G.$4DA, G.$0000, the mode bytes, the join states, the object present flags, G.$4CD, G.$4E4 and the whole struct G.$62C) and the pans of the
  sounds 0x32 and 0x33 in the ring (table 0xB24E by the cursor position: 0x00, 0x0B, 0x15, 0x20; `Game::sound` records only the code, `select_pan` gives d1 for the sound
  port). `select_report` (ignored) prints what differs for one recording (`DINO_FLOW`, `DINO_FROM`, `DINO_LAST`). Unit tests in `select.rs` (rules, lock, timer,
  class 19).
* **Known differences:** the first game task iteration counts one frame earlier in MAME (record 718); the palette cut of the press start call in record 610 (2 words);
  the records after the select: the Episode 1 opening is still the temporary one (the HUD appears at once in the port, G.$4D5 stays 1 in MAME, the first level frame
  of MAME is 2 frames long). The class 34 sparkle of the plan is not in the select: MAME shows no pool C object there; the palette play is the picked state and the
  tag palette cycle of class 68. The 2 player layout (G.$4CF, classes 19, pages 3 and 4, the other panel origins) is from the code only. Not ported on purpose: the strip
  of heroes of an in-level join (0x723E, 0x7476, 0x7F86, state 4 in mode 0xC), the credit line of the in-level HUD (0x82E8), the switch to the continue mode and the states 8 to 0xC
  of the join task (M2-05 and later); the join task runs only in modes 0 and 4 so far.

## M2-10, M2-11, M2-12: the Episode 1 opening (done 2026-10-17)

The opening now plays from power on as the original does: the camera pans along the roof tops (3 pixels a frame, mode 0x11) under the place caption, birds fly over, the
bystander (the boss) and the six thugs come in, the boss speaks the three caption lines, the thugs close in, Jack does the special, the thugs fly away and the HUD slides in.
`cargo test` is green (265 tests in dino-game); `re/harness/regress.sh`: 49 of 52 at 0 (`intro` 0, was 3), `mp_join` 37, `armed` 4, `c14c_mustapha` 8, no worse than before.

* **M2-10, captions** (`prop/class66.rs`): checked against 0x86D76 (all steps, the erase text choice, the chain) and finished: `clr.b $AC` of the print step, two tests with the
  real ROM (`every_caption_variant_runs_to_its_end`: variants 0 to 0x67 print, erase and free; `a_banner_lives_as_long_as_its_tables_say`). In `PORTED_CLASSES`.
* **M2-11, the cast** (`prop/b74.rs`, `enemy/scenery.rs`): master 0x89B48, class 56 (0x57C9C, makes the class 57 flash), table 0x57D84. `intro` equals MAME in step mode,
  in run mode and in follow mode for all 600 frames (0 object fields, 0 globals); only the join prompt cells of scroll1 differ (below).
* **M2-12, scripted objects and the opening**: the actions 0x22, 0x24, 0x2A, 0x38, 0x40, 0x46, 0x4E, 0x50 were compared with the ROM (no change); 0x42 is new
  (`script_clear_sequence` in `script.rs`, 0x1E650, with two hooks for the class 97 stand ins that M3 fills: `script_stand_in_present` 0x69E32 and `_absent` 0x69E66);
  a test of each (`the_object_actions_make_their_objects`, `the_clear_sequence_starts_once_and_waits_for_the_boss`; script 24 of `all_26_scripts_end` gets a boss with `+$A6`).
  The flags `scripted`, `floor_only`, `no_clamp` have their setters (the script step) and their readers (combat, player, terrain). The level start after the select is
  record exact (see the errata of M2-10 to M2-12): intro task `Intro::init` / `redraw`, `Flow::lag`, the pool B init in phase 4, the first level frame in the frame of
  the mode 0xC. The harness takes over the intro task from a snapshot (`trace.rs`).
* **The acceptance test** `episode_one_opening_matches_mame` (`flow/mame_check.rs`): power on, coin 598, start 698, Jack 898 against `coinstart.mame.trc`, records 1059 to 1760:
  the three layers (scroll1 to 1677), palettes, sprite list, sound codes, game state, hero 1 and the objects of the opening equal MAME in every record; G.$4E3 comes in 1672.
  The field by field diff with its allowed differences (and why) is `re/harness/allow/m2.txt`.
* **Pictures**: `re/out/shots/m210_f*.png` (17 pictures of the opening, tick numbers from power on, made with one `--headless --screenshots` run) and the 18 MAME pictures of
  `re/harness/scripts/m210_opening.txt` (`re/out/flow/m210_opening/open/snap`, power frame f = tick f + 2): 17 of 18 equal pixel for pixel; f = 1720 differs in the
  "INSERT COIN" prompts of players 2 and 3 (below).

**Not done here (other tasks)**
* The join prompts of the absent heroes (0x7562, `flow_join_prompts` in `flow/join.rs` is an empty stub): "INSERT COIN" in the HUD slots of players 2 and 3, drawn into the
  lower half of scroll1 at every slide step, so scroll1 differs from record 1678 and the pictures from f = 1700. It belongs to M2-18 (join states 0 to 6 and prompts).
* The hint object B20 (M2-15) is freed at once, so in a power on run the later pool B slots are one lower than MAME's, and the hint sound 0x3E and its palette (record 1883)
  are missing. The stand ins of the clear sequence (class 97, M3-27) are hooks. The first level frame takes 2.5 video frames in MAME: records 1057 and 1058 differ in the
  scroll3 controller, the spawn list start, the splash timer and the map cells (allow file).

Tools for this (git-ignored, `re/out/m210/`): `timeline.py` (globals and objects of a trace), `fields.py` (named fields, MAME and port side by side), `cells.py`, `cellhist.py`,
`nonblank.py`, `screenrows.py` (tilemap cells), `pw.sh` (power on run and diff).

## M2-24 and M2-25: the stage clear tally and its map panel (done 2026-10-17)

The clear task 0xB97E (slot 0x30) is ported whole, from the three heroes' panels to the map panel that slides in (`flow/tally.rs`, `flow/cards.rs`, `prop/b25.rs`,
`effect/fx_tally.rs`). It replaces the 120 frame stub. `cargo test` green (277 in dino-game); `regress.sh` 49 of 52 at 0, `mp_join` 37, `armed` 4, `c14c_mustapha` 8 (unchanged).
All 31 `flow_tally` recordings equal MAME in every record from the first run of the task to the story panel (the three layers, palettes, sprite list, sound codes, rng, fade
mask, scores, G.$4E2 and the whole 0x80 byte struct G.$735E in `clear_tally_matches_mame`; every object field in `tally_check.sh`). Pictures t = 60, 100, 120, 160 and 700 of
`tv-hp100` are equal pixel for pixel.

* **The struct G.$735E** is shared with the intro task (`Intro`, M2-10): `Tally.raw` is its byte image with the original offsets, three blocks of 0x20 bytes at +0x20, +0x40
  and +0x60 (G.$737E, $739E, $73BE: one per hero, `blk::` names the offsets: PTR +0, BEST_VIT +4, BEST_ENEMIES +5, PRESENT +6, BEST_FB +7, X +8, Y +$A, TOTAL +$C (BCD long),
  BEST_MVP +$10, TARGET +$14, COUNTING +$16, BEST_TOTAL +$17, STEP +$18, BLINK +$1C, PHASE +$1D). The trace shows it as `tally` while the task owns it.
* **The task** (`flow_tally_task`): states 0 (clear, fade) 2 (the 4 panel steps VITALITY, ENEMIES, FINAL BLOW, M.V.P. at sub 0, 2, 4, 6, 30 frames each; TOTAL; the count down
  of the totals 100 a frame into the scores and the HUD, the winner blinks; the cleanup and the fade) 4 (the map panel is drawn) 6 (it slides in, the marker blinks) 8, 0xA (end,
  G.$4E2 = 0). The sleeps of the original (`jsr 0xA76` / `0xA7E` inside a state) are the `After` continuations; a task made by the game task runs in the same frame.
* **The numbers** are pool B class 25 objects (`prop/b25.rs`): the panel figures and the TOTAL, the score words that count. Pool C 0x58 (index 22) is the colour flash at the first
  panel and pool C 0x88 (index 34) the WINNER! sparkle (`effect/fx_tally.rs`). The class 22 effect of M5-06 (screen flash, lightning) is the same handler: M5-06 should build on it.
* **The best heroes** (`tally_best`) is the listing of 0xBC96, 0xBDD4, 0xBFFC and 0xC1BA, quirks kept: ties share the mark, but a hero who beats a tie only takes the mark
  of the last leader (so with 3, 3, 9 the first hero keeps his mark), the best starts at 0 (a negative figure is never best), vitality and M.V.P. compare signed words, the
  enemies unsigned bytes, the totals signed longs. The anti tamper 0xD860 is not ported but its random roll is: when the HP of hero 0 is even and not negative it calls
  `rand()` once (0x119C), which moves the rng and is needed to equal MAME.
* **The map panel** (`cards.rs`, used again by the intro card M2-26): scroll1 is cleared (0xB52), the palettes 1 to 13 of scroll1 come from the ROM palettes 0xBC252 by the
  stage's list (0xCD6C) and the fade mask stays 2, the panel is three blocks of tile columns (10 x 13, 8 x 15 two cells higher, 10 x 13; codes 0xCEC4, attributes 0xD1BC; the
  stages 6 and 7 add blocks 0xD4DC and 0xD6CE), the name plate is 12 codes at the panel + 0x42C. The slide copies 17 cells of each column to the page behind the right edge
  and moves scroll1 X by 12 a frame to 0x130 (26 frames); then the marker (a palette swap, `flow_card_marker`) is lit and dark in periods of 20 frames; the state goes on at
  the period end when the counter +4 (0x78) is negative: 140 frames; 60 frames of sleep; state 8, 0xA. Episode 8 (stage 7) has no panel (state 0xA at once) and, in area 3,
  leaves G.$4E2 set, so the game task waits for its timer (0xE10) and goes to the ending, mode 0x1C (`flow_ingame_switch`, the ending itself is M3).
* **The length**: with one hero the task ends (G.$4E2 = 0) 749 records after its first run for the largest total 30600, and 100 records fewer for every 10000 less (443 + max_total / 100);
  mode 8 comes one record later. The plan's "177 + max_total / 100 + 268" is the same up to its starting point.
* **A busy frame** (`tally_state4`, the only estimate in the task): with many objects alive (3 heroes, or 2 heroes and a tie) the draw of the panel takes the 68000 through the next
  vertical blank. MAME's record of that frame shows the last cells of the panel and the name plate undrawn (the scroll1 difference the check allows), and the next pass of the
  tasks starts late: the game task counts in the next record, the tally task one record later. The port draws all cells at once and sleeps the tally task one more frame
  when the live objects of pool D plus twice those of pool A are 17 or more (the cycles are not counted; the rule classifies all 31 recordings right).

**Not done here**
* The story panel (mode 8) was M2-28 and is done (next section): the tally runs now go on through the panel and the start of the next episode.
* The tally's only sound is 0x49 (d1 = 0x10): at each of the four panel steps and every 11 frames of the count down; the map panel makes none. They equal MAME in every record.
  Episode 8's tally and the ending are not recorded (`flow_tally` is Episode 1 only): the Episode 8 path is from the listing and a unit test.

## M2-26, M2-27, M2-28: the story panels, the map card and the walk in (done 2026-10-17)

The chain between two episodes now runs as in the original: boss, clear tally, map panel, story panel (mode 8), stage start, map card with the typed title, slide
out, walk in of the heroes, HUD slide in. `cargo test` green (294 in dino-game), `regress.sh`: 49 of 52 at 0, `mp_join` 38, `armed` 4, `c14c_mustapha` 8 (the new baseline).
Checked against the MAME recordings of `flow_links` (1100 frames from the poke of the mode 8), `flow_panels` (32 recordings: the party, the skip) and `flow_tally`
(the tally then the panel of Episode 2): `re/harness/link_check.sh` and `tally_check.sh`. Files: `flow/cards.rs` (panels and card), `flow.rs` (the intro task, the stage
start phases), `prop/class26.rs` (the typewriter in pool B), `prop.rs` (classes 69 and 80), `flow/select.rs` (`pub(super) select_exit`, the lag of Episodes 3 and 5).

* **Result per recording.** Episodes 6 and 8 run equal to MAME in all 1100 records (panel, card, title, marker, slide out, walk in, HUD slide in, G.$4E2 / G.$4E3 / G.$4D5,
  sounds, the three layers, palettes). Episode 3 equal to record 915, Episode 7 to 991 (then the score digits of the HUD: a kill that unported classes make in MAME), Episode 2 to 560 and
  Episode 4 to 311 (the scripted actors of the car and the garage scenes, class 97 stand ins, M3), Episode 5 to 442 (enemies and items of the start, unported classes). The
  panels alone (up to the first level frames) are equal for every party of Episodes 4 to 8 (`flow_panels`, 330 frames), the skip by attack (`pan-skip-ep2`: mode 4 at 64) and the
  non-skip by jump (`pan-skip-ep2-jump`). The tally recordings (31) run equal to their end, through the hand over and the first frames of Episode 2's panel (up to 150 frames).
  Pictures: `re/harness/shot_check.sh` (headless, one run per segment) is pixel equal at t = 100, 250, 440 for Episodes 3, 6, 8 and 100 to 440 for 2 and 7 (later pictures differ in the join prompts).
* **The panel** (0xD898, `flow_story_mode`): frame 0 the palettes of the stage (0x8A44, 0x8932) and the clear 0xAE0 start (it keeps the CPU through the next frame); frame 2
  0xD8B4 (scene palette set 2, 0x892C) and the entry of the stage up to its first sleep; then the wait loop 0xDFCC every frame: the attack check of the present heroes
  (live byte exactly 1, the input exactly 0x10 and changed since the frame before; jump does nothing), the counter G.-$719A. Entries (stage 1 to 7): Ep2 300 frames; Ep3 241 frames of
  scroll2 X + 0x10 (no skip; the picture is scroll3 over scroll2 with the layer control 0x1396: `Layer::BackdropFront`); Ep4 300; Ep5 300 then scroll1 cleared, the second text and 64
  rounds of two pictures (skip works there too); Ep6 360; Ep7 360; Ep8 61 steps of 5 frames of palette play (0xE0E4, no skip). The panel texts are printed at once by the tall printer
  0x84F72 (not typed: the typed text is the card's title); the portraits by 0x84736 with the caller (0xE034: lowest hero id nobody plays, 1 and 2 swapped) or the playing hero (0xE0AA).
  Ep6 draws no portrait when Hannah plays alone (the original's bug, kept). 0x84736 leaves its palette word at G.$6A32 - 4 * (heroes - 1) (the stray `+$A2(a6)` of the game task, shown in
  the traces as `obst_slot`). The anti tamper rolls of Ep2 (1 in 16) and Ep4 (1 in 8) draw one `rand()` after the wait and do nothing (the ROM sum is right).
* **The exit** is 0xAC4E with its phases (`flow_stage_start_staged`, the same as after the select): the clear 0xAE0 at the end of the entry keeps the CPU 2 frames (Ep3, 4, 5, 6, 7, 8
  end with their own 0xAE0, Ep2 without), then phase 0 (the clear again), 1, 2 (camera Y 0x100, the intro task made, 0x8A44 and the scroll controllers reset: the video frame
  boundary comes between the camera and these after Episode 5), 3 (mode 4: the first part of the area start up to the pool A init) and 4 (mode 0xC, the rest of the area start, the first
  level frame). The area start is split in two (`flow_area_reset_a` / `_b`): until phase 4 the pools B, C, D, the stage time and the floor of the story panel's level are still there. After
  Episode 5 the phases 3 and 4 are one frame (`story_exit_ep5`). The first level frame takes 2 video frames after the panels, 3 for Episodes 3 and 5 (`Flow::lag`, `select_exit`), and the
  kills of the heroes (+$10B) are cleared in the tail (0xAC7E). The panel's lengths (record of the mode 4 or 0xC after the poke): 305, 249, 307, 438, 367, 367, 312 (D4d's table + 1).
* **The map card** (intro task 0xB9BE, states 0 to 6, `flow.rs` and `cards.rs`): the task is made in phase 2 and first runs with the first level frame (the first long of the struct G.$735E
  is cleared, the rest keeps the bytes of the tally). State 0 (0xC526: a sleep, then the card is drawn at 0x90A030 with the palettes of the stage and the name plate, state 2); state 2 sub 0
  (0xC642: the columns move to the page at the right, 23 cells each, the source is blanked, X + 12 a frame to 0x130, a sleep, then the typewriter 0x7EC20 with the title of the
  episode, a pool B object of class 26); sub 2 (0xCA20: the marker blinks, the same code as the tally's, 140 frames; then state 4, a sleep of 60, X + 12, G.$4D5 = 1); state 4 sub 0 (0xCBBE: the
  card slides out, the columns behind are blanked; at X 0x260 the scroll1 palettes come back, scroll1 is cleared, G.$4E2 = 0), sub 2 (the walk in: waits for G.$4E3 of the heroes' script,
  then camera Y 0x100 and the HUD redraw), sub 4 (the HUD slides in to X 0x3F0 for the stages 1 to 7, 0x180 for the stage 0, G.$4D5 = 0), state 6 (G.$4E2 = 0, the end). The handshake is
  the script's (`script.rs`: action 0x48 sets G.$4E2 when the card runs, 0x2E waits for it, the walk in sets G.$4E3): G.$4E2 = 1 3 records after the mode 0xC, G.$4D5 = 1 231 records and
  G.$4E2 = 0 255 records after the mode 4 (Episodes 3, 6, 7, 8). The title is a pool B object: slot, `+$20 = 0x68`, live byte 1 then 2, modes 0, 2, 4 as in the recordings (`prop/class26.rs`).
* **Two more pool B classes** the stage start makes: class 69 (0x88A84, the debug mode object: it frees itself in its first update when G.$4AA is off) and class 80 (0x8A582, the ambience
  of Episode 2: every 120 frames a number, below 4 of 16 a second one decides the sound 0x78).
* **Allowed differences** (`re/harness/allow/m228.txt`, `story_allow.py` writes the ranges): the join prompts of the absent heroes (scroll1 rows 4 and 5 of the columns 81 to 111, and
  their blank cells 0x20 / 0x018B: M2-18), the first level frame (2.5 video frames: the camera and scroll controller fields, the objects of the spawn tick), the cut of the pool A init and
  of the palette and layer loads at the video frame boundaries, the display shadows G.$860 / G.$868 (the main loop copies the cameras at the start of a pass; the port derives them), the
  derived pointers of the stage that is poked in, scroll2 of Episode 3's level (its camera map is not ported). `diff.py` got the allow rules `cell LAYER MAME PORT` and `rect scroll1 C0 C1 R0 R1`.

**Not done here**
* Episodes 2 and 4: the scripted scenes with the car and the garage need the class 97 stand ins (M3). Episode 5's first enemies and items are unported classes.
* The join prompts (M2-18) and the debug mode object (class 69 frees itself; the debug mode is not ported).
* A busy start of the story panel in the middle of a level snapshot is not taken over by `trace_load_tasks` (the recordings start before the poke, `link_prep.py`).

Tools: `re/harness/link_check.sh [NAME ...]` (`STORY_ONLY=1` stops at the first level frames; `FRAMES=`), `link_prep.py` (cuts the record before the poke and writes the pokes file),
`story_allow.py`, `shot_check.sh KIND NAME T1,T2,...` (pictures against the MAME snapshots), `trace_slice.py ... ADDR=VALUE:-N`.

## Running list of the main code worker (2026-10-17)

### G.$7B4 has one owner, F6 in Episode 3 (task 1, green)

* `Rank::b7b4` is deleted. G.$7B4 is the camera's lock state `cam.s2.lock78` (struct G.$73C + $78) for every reader and writer: the aggression table choice
  0x182BA (`rank_reroll_aggression`: lock 1 = table 0x18364), the time over with the car 0x1042E (`rank_time_over`), the area music 0xA55A (`flow_area_music`: lock 2 = the
  table 0xA5C8), the clear at the stage start 0xACAC and at the demo start (`flow_stage_start_tail`, `flow_demo_stage_tail`) and the hint (`prop/b20.rs`, already used it). The trace
  line `b7b4` is gone, so a snapshot loads the byte once (as `lock78`). The old trace name `b7b4` is in no script or allow file.
* 0xA55A was checked against the code: the stage 7 case `cmpi.b #$FE, $7D(a0)` with `a0 = G.-$70BC` (the boss of the HUD bar) is now real (`hud.boss`, `+$7D` = `ATTACKING` of the pool A
  object): entry 8 of the table when it is 0xFE (the old code had a stand in that never changed the entry). Tests: `the_rank_and_the_music_read_the_camera_lock_state`,
  `the_last_boss_music_follows_the_boss_mark` (`flow.rs`).
* Not done, for the class ports: the original boss inits write 2 (Hogg 0x4FDCA, class 84 0x6187A, class 91 0x687A0, the Slisaur starter B 33 every frame 0x819BE). The placeholders
  do not write it, so the boss music variant and the car time over rule need the real classes. 0x105B6 (pass 0x12454 when the lock is not 0) is still the M6 stub in `combat.rs`.
* Free runs now use the second aggression table during the camera locks of Episode 1 and the others (lock 1), as the original does.
* `debug::next_area` (F6) sets mode 6 sub 4 in stage 2 (the controller 0x22398 has no mode 0x12) and mode 0x12 elsewhere; `--area` above 0 with `--episode 3` is refused (Episode 3 has
  one area). Tests in `debug.rs`: `the_area_change_key_uses_the_mode_of_the_stage_controller`, `the_area_change_key_ends_in_episode_3` (mode 6 sub 4, sub 6, back to mode 0).
* Gates: `cargo build` and `cargo test` green (341 in dino-game with the 4 new tests, 8 ignored), `re/harness/regress.sh`: 49 of 52 at 0, `mp_join` 3, `armed` 4, `c14c_mustapha` 8 (the baseline).

### M3-23: the ending, part 1 (lab corridor scene and the explosion chain; green, record exact to the wall of the sunset scene)

* **What.** Mode 0x1C is the routine 0xA69B8 (`flow/ending.rs`: sub 0, 2, 4, 6, 8, the object loop in the ending's own order), the master pool B 82 (`prop/b82_85.rs`,
  all its states up to the desert master 81), the showers pool B 83, 84, 85 and the pool A classes 90, 92, 93, 96 (`enemy/class90.rs`, `class92.rs`, `class93.rs`,
  `class96.rs`), `snd_rumble` 0x9F12, `fx_spawn_class21` 0x94640 (the effect is made, the ending never updates pool C). The classes are in `PORTED_CLASSES`, the animation tables
  0x6531C, 0x678A0, 0x67C3A, 0x69E00 in `OBJECT_ANIM_TABLES`, the tables 0x8B112, 0xC1252, 0x8B402 in `FRONT_TABLES`. Read `re/specs/flow/ENDING.md` for what the classes do.
  New trace fields: `b5e4` (G.$5E4), `ending_timer` (G.-$6FFA). Join task: the state 6 of a player without a continue (0x7392: blank the HUD block, the qualification 0x154EE ->
  `flow_join_qualify`, `flow_hiscore_qualifies`) and the minimal state 0xC (0x74E2, `flow_join_game_over`: the counter `+$80` runs out, the block is free again). The name entry
  (state 0xA, M2-22) is not ported: `flow_name_entry_available()` is false, so a qualifying hero goes to state 0xC and the ending does not wait for him; `flow_name_entry_setup`
  (0x15AE2) and `flow_name_entry_busy` (0xEFC2, real) are the hooks for M2-22 and M3-27.
* **Checks.** Video clock: `cargo test the_ending_matches_mame` loads the record before the probe's poke of `flow_end-jack` / `end-hannah` / `end-3p`, pokes the ending in and
  compares 520 records (the three layers, the palettes, the sprite list, the sound codes, G.$14, mode and sub, the camera X of scroll2 and scroll3: all equal; scroll2 at record +0
  is not compared, see below); `ending_report` (ignored, `DINO_ENDING`, `DINO_LAST`, `DINO_ALL`) lists differences for any length. The whole of 960 records differs only in the 29
  cells of the wall of the sunset scene (pool B 3 variant 4, from record +530: the class of M3-19, see below). Game clock (`re/out/ending/run.sh m328_end-jack step|run 1000`, uses
  `DINO_GAME_CLOCK=1`, which makes `harness::run` tick through the lag frames of one record): in `run` mode (free run from the snapshot) all 1000 records equal MAME in every object
  field, the random number, the sounds and the three layers except pool B 3 / pool C 21 (var 0x14 from the wall), the staff roll (B87 from 771) and the desert (B81 from 970); the
  derived pointers of the poked stage (G.$674, G.$678, G.$792) and the frame pointer of objects that never drew (`prop.frame`, `enemy.frame`: MAME 0, the port shows the first frame)
  are harness artifacts (`re/harness/allow/m3e.txt`). The same for `m328_end-hannah`, `end-3p` and `end-mess-jack` over 700 records.
* **Timing, the lag of the 68000.** `Flow::lag` skips the next video frame (no task runs). Sub 0 runs in two parts (`Ending::phase`): the clear (the sprites and scroll1 now, scroll2
  up to `CLEAR2_FIRST` cells, then the rest of it and scroll3 up to `CLEAR3_FIRST` in the lag frame, the rest of scroll3 in the third frame) and the lag, then the pools, the palette and the
  timer and, in the same video frame, the pass of sub 2 (G.$0 counts once). The master's state 0 does the same: the palettes and the clear, a lag frame, then its four pages in the
  frame of the next pass (`Ending::pages_for`, `b82_pages`); the state 0x16 end (two layer clears) takes a lag frame too. `flow_frame` runs a clear that is under way in a lag
  frame (`flow_attract_clear_step`). The cut of the scroll2 clear in the first frame varies between 1367 and 1384 cells with the work the frame did before (six recordings): the test
  does not compare scroll2 at record +0. `Flow::lag_tick`, `Flow::game_clock`: the harness joins the pass that overran, the lag frames and the next pass into one record when the
  recording uses the game clock (`DINO_GAME_CLOCK`, set by `re/out/ending/run.sh`; `port.sh` users add it themselves).
* **Also changed.** `trace.rs`: the derived pointers G.$674 / G.$678 / G.$792 wrap instead of panicking for a snapshot poked to another stage; the spawn lists keep their
  indexes through the ending's pool reset (0xFBE8 does not clear them).
* **Waiting for other tasks.** Pool B 3 variant 4 (the wall that breaks at record 525 and its pool C 21 variant 0x14) is class 3 of M3-19 (the same code path as variant 8, 0x7667E):
  check records 525 to 560 of `m328_end-jack` after it is merged. The staff roll (B87, 0x8B60E, from record 771) and the desert (B81 / pool A 81, from 969) are M3-25 and M3-26.

### M3-24 to M3-27: the rest of the ending (green; the whole ending equals MAME)

* **Result.** `the_ending_matches_mame` (`flow/mame_check.rs`) runs `flow_end-jack`, `end-hannah`, `end-3p`, `end-mess-jack`, `end-mustapha` and `end-hiscore` (video clock, from the record before the probe's poke
  of the ending) for 5700 records each: the three layers, the palettes, the sprite list, the sound codes, G.$14, mode and sub and all scroll registers (scroll1 X, Y and Y fraction, scroll2 and scroll3 X and Y) equal MAME in
  every record, from the poke through all five scenes, the staff roll, the car, the dialogue, THANK YOU FOR PLAYING, the sub 6 wait (the high score recording waits in the name entry), the sleep, the game over and the
  first frames of the ranking (the exceptions: scroll2 of record +0, whose clear cut depends on the work before it, and up to 4 cells of scroll3 in the first frame of the ranking, the same kind of cut). Game clock (`re/out/ending/run.sh
  NAME run 5538 --from 1 --allow re/harness/allow/m3e.txt`): `m328_end-jack`, `hannah`, `mess-jack`, `mustapha`, `fine`, `objs`, `3p` and `hiscore` (5700) free runs have no difference in any object field, global, tilemap,
  palette, sprite or sound for 5538 records (the allow file lists the harness artifacts). Records from 5540 on (the game over) are compared in the video clock only. `ending_report` (ignored; `DINO_ENDING`, `DINO_LAST`,
  `DINO_ALL`, `DINO_AT`, `DINO_JOIN`) lists differences for any recording and length.
* **New in this part.** `prop/b87.rs` (the staff roll), `prop/b81.rs` (the desert master, the talk, the end texts), `enemy/class81.rs` (walkers and Cadillac), `enemy/class97.rs` (the Episode 8 stand ins, `script.rs` calls them),
  the continuation `Ending::cont` (a state of the master whose pass overran the video frame: `Cont::Pages`, `Cont::StillEnd`), the staged clear after the sleep (`flow_attract_clear` + lag), `Camera::cam1_y_frac` (G.$6CA), `Scroll3::cam_x_frac`
  (G.$7E6, trace fields `cam1_y_frac`, `cam3_x_frac`), the tables 0x8B786.. (letters, lines, attributes), 0x8A956, 0x69E9A, 0x6A010 in `FRONT_TABLES`, the animation tables 0x604EA and 0x6A020, 0x6A08E, 0x6A0FC, 0x6A16A.
  The harness joins a continuation into one game clock record (`harness::run`: `lag`, `lag_tick`, `Ending::cont`).
* **Fixes on the way.** Class 17's init clears `+$A0` (0x423CC); the name entry's end (0x74C2: state 0xC, `started` bit cleared, the HUD rectangle blank 0x7F86) was missing in `name_state`; G.$5E4 was traced twice (`b5e4` and the continue
  struct): one owner now (`Flow::b5e4`); `ranking_first_run` is false for the ranking of a game over; the trace pointer sums wrap.
* **Episode 8 to the ending.** `m328_f_ep8` (the clear sequence poked in at record 20): the port reaches the mode 0x1C at record 3874 and the sub 4 at 3875, like MAME; the stand ins of class 97 equal MAME in every field from 1134 to 3874 (step
  mode). `cargo test episode_eight_reaches_the_ending -- --ignored --nocapture` runs the placeholder Episode 8 with the debug area jump, kill all and a hero who cannot die: mode 0x1C at tick 42370, sub 4 at 42373, sub 6 at 47734 (+5364),
  sub 8 at 47735, mode 0x14 at 47916 (+5546), 0x18 at 47918 and the attract loop at 47922, no panic.
* **Not done.** The pool B 50 dialogue of the boss (variant 7) and the pool C 39 aura are made by the stand ins but not ported (unported classes: freed or held); the join state 0xC is the minimal stand in of `flow_join_game_over`
  (the GAME OVER HUD text of the continue path is M2-21's `cont.rs`); the 2 player layout of the ending was not recorded.

### M3-13 and M3-14: the lift of Episode 7 area 1 (green)

* **What.** `camera.rs` `lift_background` (0x205C6, the cage on scroll1, called by the area setup 0x20766), `prop/b57.rs` (pool B 57 the floor labels 0x85B64, pool B 64 the cage posts 0x86B64), tables through `dino-extract` (0x85BF6, 0x85D6C, 0x86BBE, 0x86BD4, four 12 byte frame records), `PAST_END` 0x100
  for the scroll2 map streams that read past the end of an array (the lift is 2 x 56 screens wide streamed to X 0x340), G.$7B6 `spawn.g.cage_smashed` (cleared at the area setup, B54 will set it). Details and the one quirk (the cage's a0 / a1 carry over from column to column) are in `re/specs/stage/HANDOFF.md`, the corrections in `D5-errata.md`.
* **Checks.** `re/harness/m313_check.sh`: `m800_a14_n_e7a1_full` (4900 records) and the lift of `ep7_3p` (12306 to 17850) in step mode equal MAME in the camera, the labels, the posts and the cage in every record (the rest is unported classes and the lag records); `m328_l_e7a0_k16` (the area change into the lift, free run) has scroll1 equal in every record but the first. Free run: `--episode 7 --area 1 --autoplay` rides the lift to area 2 at tick 5395.
* **Review aid.** `debug::next_area` (F6, `--area`) enters the lift through the fade path (mode 1) and clears the fade mask of a direct start: the hard cut (mode 0x12) left the cage over the HUD.
* Gates: `cargo build`, `cargo test` (369 in dino-game, 10 ignored) green, `regress.sh` 49 of 52 at 0, `mp_join` 3, `armed` 4, `c14c_mustapha` 8 (the baseline).

### M2-23: the rank events (green; most of the wiring was already in, now checked)

* **Wired, each checked against the 68000 code and its callers.** 0x18410 (join, speed + 3 only in mode 0xC: `join.rs` 0x6E98 and `select.rs` 0x6E92), 0x18566 with -2 (continue 0x7468, `cont.rs`), +2 (0xACE0 / 0xACD0), +1 (0xACEE, boss defeated), 0x184E2 with 0x1843C
  (0x1BFF0, 0x1C020) and 0x1842C (0x1C026), 0x1843C (0x6BDC, extra life), 0x183E4 (0x1C0F4 / 0x1F276), 0x17F8A (0xA9B8 game start, 0x99F92 demo), 0x1800E, 0x18048, 0x182A2 (spawn command, kill all, the calm period), 0x1827A (system task). Every caller of the original has its call. G.$602 is
  written only by the demo (0x9A0BC, ported) and by the debug mode object (0x88BA4, B69, not ported): there is no freeze in normal play. G.$4DD is the script flag (`script.rs`, read by `rank_tick`).
* **The TEMP of `rank_tick` is gone** (it started the rank system on its own when nothing had: `Rank::started`). The flow does it (0xA978 and the demo setup); a snapshot load takes the rank fields as they are.
* **G.$4F6.** `m223_free_wait` (new MAME recording, `re/harness/scripts/m223_free_wait.txt`: free play, start at record 2600 in the profile cards, counter 2490): MAME clears G.$4F6 in the pass that creates the game task (record 2618) and
  0x17F8A runs two records later (rank 13, speed 2, the first band), so a long wait never lowers the start rank. The port equals MAME for G.$4F6, rank, speed, rng and aggression from record 2590 to 2640 (`a_long_wait_does_not_change_the_starting_rank`).
* **Natural play.** The whole-episode recordings have the rank events of deaths, continues and joins without a rank hold: `ep7_3p` (38,000 records), `ep1_3p`, `ep4_3p` in step mode: every difference of the rank fields (G.$5F0, G.$5EE, G.$5F6, G.$5F8, G.$604, G.$606, G.$608, aggression) is in a MAME lag record (2303, 2560, 4401, 8161 ...) or follows the
  boss death of an unported placeholder (`ep4_3p` 26636: the +1 comes a record later). The m515 respawn probes (`s1`, `s8`, `s9`, `s10`) hold the rank, so they say nothing about it.
* Unit tests (`rank.rs`): speed clamps and the countdown restart, join in modes 4 and 0xC, the drop by difficulty 7 to 0 and the 3 of a second miss within 600 frames, the calm period, a miss with a life left, the game over, the hit lockout, the wait counter (pool, free play, wrap), the start rank by difficulty and the speed bits.
* Gates: `cargo build`, `cargo test` (392 in dino-game, 10 ignored) green, `regress.sh` 50 of 52 at 0, `mp_join` 3, `c14c_mustapha` 8.

