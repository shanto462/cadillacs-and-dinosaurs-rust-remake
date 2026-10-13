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
- `flow/front.rs`: character select (mode 4 starts the stage at once with Jack, Mustapha, Hannah for the
  players that pressed start; M2-03). The system task is `flow/system.rs` (M2-02, complete); the attract loop,
  title and press start screen are done (`boot.rs`, `attract.rs`).
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
