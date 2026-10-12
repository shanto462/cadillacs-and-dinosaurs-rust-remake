# Attract mode and front end: title, coin, select, join, continue, game over, ranking, ending

Everything outside a level. Source tags: **[M]** MAME observation 2026-10-12 (60 Hz frames from power
on, scripts in the scratchpad, see `README.md` section 0), **[R]** repo file, **[W]** web.
Port state: DONE, PARTIAL, STUB, MISSING, UNKNOWN. Captions of the game are paraphrased.

Port state for the whole file in one line: **everything here is MISSING or STUB.** `main.rs` starts a
level directly (`start_stage`), `flow/front.rs` and `flow/join.rs` hold empty functions with `TODO(port)`.
The game task (modes, stage setup, stage clear switch) in `flow.rs` is ported and is reused by every step below.

## 1. Task layout of the original (so the port can copy the structure)

Cooperative tasks, one run per frame in slot order (`re/specs/CONTEXT.md`):

| Slot | Task | Entry | Role |
| --- | --- | --- | --- |
| 0x00 | system task | 0x9D048 | coins, credits, start button, kills tasks 0x10..0x40 and calls 0x983EE on start |
| 0x10 | attract task, or in game the join/continue task | 0x97C20 / 0x6C4A | attract loop / per player join states |
| 0x20 | game task | 0xA870 | the mode byte G.$4D0 (below) |
| 0x30 | stage intro or stage clear task | 0xB9BE / 0xB97E | cards, slide in of the HUD / tally |
| 0x40 | fade task | 0x8824 (in) / 0x8840 (out) | palette fades, already ported in `palette.rs` |

Game modes (G.$4D0) [R] `re/specs/flow/SPEC.md` section 4, checked in MAME:
`0` game start setup (0xA978), `4` character select (0xAD46), `8` link story panel (0xD898),
`0xC` in the level (0xAAF6), `0x10` continue (0xE9E6), `0x14` / `0x18` game over (0xAC06 / 0xAC26),
`0x1C` ending (0xA69B8, a jump table on G.$4D1: sub 0 setup, 4 the 5360 frame sequence, 6 and 8 the end).
In the port, `flow_game_task` has all of these as `match` arms; arms 0x04, 0x08, 0x10 and 0x1C are stubs.

The attract and demo code is not in "game" mode: G.$4CC is 0 (no game), G.$4D0 stays 0 or 0xC while a demo
runs. The demo runs the same per frame code as a game (`FUN_00099e88` repeats 0xE90E, 0x10546, 0x16BEA,
0x13FA6, 0x105E8, 0xFBF4 like `game_frame`), so the demo needs a working level.

## 2. Boot screens [M]

| Frames | Screen | Notes |
| --- | --- | --- |
| 0 to about 100 | name of the game, ROM date code, region word, small blue text on black | probably the CPS "board info" text of the program. Address unknown. |
| about 100 to 300 | legal warning (where the game may be sold, risk of prosecution) on black | Address unknown. Region word shows the world set. |
| about 300 | fade to the title | |

Port: MISSING. Priority: low (pure text), but the user said the port is "totally different", and this is
the first thing seen. Cheap to add with the text printer.

## 3. The attract loop [M] [R]

Routine map (the table at 0x97C88 holds word offsets; state byte G.$0FD0, steps of 2):

| State | Entry | Parts in order | Seen length |
| --- | --- | --- | --- |
| 0 | 0x97D84 | title 0x9A3CE (600), profiles 0x9A7E6 (1200), ranking 0x98330 | about 2730 |
| 2 | 0x97CC0 | title (600), ranking-style pause picture (0x98330), demo 0x99E88 (1200 frames of play), slides and ranking 0x988C0 | about 4130 |
| 4 | 0x97D84 | same as state 0 | about 2440 |
| 6 | 0x97D02 | title, 0x98330, demo, clear, opening movie 0x9BF50, then state becomes 0 | about 4600 |
| 8, 10 | 0x97D42, 0x97DBE | exist but were not reached in 16000 frames | n/a |

(The measured frame numbers: state 2 at f=2731, state 4 at f=6862, state 6 at f=9299, loop restart at
f=13399 with the next state 2 at f=15836.)

Details:

1. **Title** (0x9A3CE) [M f=300..900]. Clears the object pools and runs the object loop (0x222D6 shake,
   0x23604 pool B, 0x23420 pool A, sprite list) so the logo is made of **objects**: two pool B objects of
   class 0x188 (class number 98, variants 0 and 1) carry the logo and the Capcom sign. A task from 0x98874
   makes "INSERT COIN" blink (about 30 frames on, 30 off) at tile (18, 56). Frame 30: the small text is
   drawn (printer 0xBB6, then 0x110C). Frame 360 (0x168): that task is killed, scroll 1 is cleared and the
   licence text is drawn instead (the second text uses the string printer 0xBB6 with positions). At frame
   600 the routine returns. Credits: "CREDIT n" and "PUSH START BUTTON" appear at the bottom once a
   coin is in [M f=690..]. Music: the title tune (about 20 seconds per HG101 [W]). Port: MISSING.
2. **Profiles** (0x9A7E6, 1200 frames) [M f=1000..2300]. Pool A and B objects animate the small hero on
   the right while 0x9AC86 and 0x9ACBA scroll and fade the card. Timeline table 0x9A8E4 holds the frame
   numbers (9, 10, 11, 308, 309, 310, 311, 608, ...) and table 0x9A908 the handlers (0x9A908, 0x9A94E,
   0x9A950, 0x9A92E, 0x9ABDC, 0x9A9FE, 0x9A93E, 0x9AA98): each hero gets about 300 frames with three
   actions (draw the card, change the pose, fade). Texts per hero: name, height, weight, job, skill word.
   Port: MISSING. Data to extract: the portraits (tiles), the text strings, the timeline tables.
3. **Ranking** [M f=2300..2731, 6860 and others]. Called as 0x98330: kills the INSERT COIN task, fades out
   (0x87BA), clears (0xAE0), sets scroll 3 position, draws two scroll 3 pages (0x20592 with d5 = 0x3E and
   0x3F, the desert and the car), fades in (0x879A), waits, and creates the blink task again (0x98874).
   The ranking text "BRAVES" and the rows are printed by the same screens (strings with the table at
   G.$703E, 16 byte entries: score long, 4 byte tag, character id, stage + 1). The skipped case:
   `cmpi.w #2,$32(a5)` jumps over the picture (meaning of G.$32 unknown). Port: MISSING; the table
   itself exists in `hud.rs` (`hiscores`, default table at 0x1504E, 50 entries).
4. **Demo** (0x99E88 loop, set up by 0x99F10) [M f=3340..5000 and f=9910..11200].
   0x99F10: clears most of RAM, runs the DIP decode (0xA804), 0xFF2E, 0xA836, rank 0x17F8A, 0x6B88, sets the
   in-game flag G.$84D7 = 1, creates the 2 or 3 players (0x4D46 / 0x4D5E / 0x4D76 and 0x18C88), picks the
   level from table 0x9A1E2 (3 player layout) or 0x9A1EE (2 player layout) with the counter G.$757A
   (incremented at each demo, wraps after 5): six rows of `(stage, area, frames)`, all with 1200 frames:
   `(0,1) (1,0) (2,0) (7,1) (4,0) (5,1)`. Then it starts the level music (0xA55A) and the HUD
   (0x162AE). The players are driven by **recorded input streams**: `FUN_0009a146` (0x9A146) reads a
   run-length list per player (value, count pairs) through pointer tables (0x9A13A, 0x9A2DE, 0x9A326,
   0x9A36E). These lists are data and must go through `dino-extract`.
   In the demo the HUD shows "INSERT COIN" prompts and the players do not drop weapons from continue.
   [M] Demo 1 (Episode 2 start): a 3 player team (Jack, Mustapha, Mess) gets out of the Cadillac at the
   edge of the swamp forest, fights poachers, carries a barrel, grabs an Uzi, and walks past a sleeping
   dinosaur marked with Z letters. Demo 2 (Episode 3): the caption "DESERT OF DEATH" shows at the start;
   three heroes fight bikers and thugs standing on and around the moving Cadillac in the desert.
   Port: MISSING; needs the levels of the demo table to work (Episodes 2, 3, 8, 5, 6, 1).
5. **Story slides** (0x988C0, after demo 1) [M f=5400..6800]. A script of long sleeps (30, 60, 180
   frames), picture loads (0x84F72 with data blocks, 0x204CA, 0x20592 scroll 3 pages), caption printer
   (0x98B86, 0x98BAE for a 4 x 3 tile picture), fades (0x879A, 0x87BA) and a camera slide (X -= 8 per
   frame). Five slides: (1) a panic call from a village, an old man cries to a hero that dinosaurs went
   crazy (the dinosaurs rarely attack people); (2) two heroes decide a gang of poachers is the cause; (3)
   a poacher tells a helper to hurry skinning a dinosaur; (4) a heroine looks out of a window and says she
   does not understand why the poachers do this; (5) a close up of a villain who says they keep messing with
   the balance and he will crush them. Then the ranking again. Port: MISSING.
6. **Opening movie** (0x9BF50, after demo 2) [M f=11800..13400]. 1480 frames (0x5C8). First a black screen
   with a text crawl: about 400 years ago science nearly made new life, they went too far by changing
   evolution, nature takes care of itself, the "Great Cataclysm" (key words are red). Then a camera
   pans over a jungle: a man in the foreground says "wow, that one is huge", the head of a big dinosaur
   rises, it roars and the screen goes black. Per frame work: 0x9C7C4, 0x9C7F8, the object loops, a timeline
   table at 0x9C002 and handler table at 0x9C026. Port: MISSING.

Extract list for the attract loop (for D4 and milestone M1): title object frames, profile portraits and
poses, the four text tables, the demo tables and input streams, the slide pictures and strings, the movie
picture data and timeline, the ranking pictures. All of it is ROM data and must go through `dino-extract`.

## 4. Coin, credits, start [R] [M]

- System task 0x9D048; coin edge detect G.$20 bits 0..2 (COIN1..3), start G.$1A bits 0..2, previous
  value G.$1F. Coin logic 0x9D4BE, credit add 0x9D386, start 0x9D196. Credit text "CREDIT n" or "FREE PLAY".
- Cabinet settings come from the EEPROM (test menu): coins per credit (1 coin 1 to 6 credits, 2, 3 or 4
  coins per credit, "2 coins start, 1 coin continue"), free play, demo sound on or off, continue on or off,
  "timer level", difficulty, extra life score, lives, 2 or 3 player layout (cooked into G.$777A, G.$777B,
  G.$777C; factory values 0xACBF4 give 0x80, 0x4B, 0x41 [R] `re/specs/flow/SPEC.md` 6). The MAME driver
  has no DIP switches for these (only "Freeze"), so MAME runs the factory defaults: 1 coin 1 credit,
  3 players, difficulty 3, extra life at 500000, 1 extra life (2 lives in all; the lives field read 1
  in MAME), continue on.
- Start with a credit: the title changes to "PUSH START BUTTON" for each present player, "INSERT COIN" for
  the others [M]. Press start: the system task kills the attract and calls 0x983EE (clear, palette, scroll
  setup) and the game task starts at mode 0.
- Port: MISSING. `Settings` exists in `flow.rs` with the three factory bytes.

## 5. Character select (mode 4, 0xAD46) [M] [R]

- Layout: a group picture of the 4 heroes as the background, "TIME 10" counting down at the top centre,
  a panel per player (blue 1P, red 2P, orange 3P) at the bottom with a face, three stat bars and the
  rating word. Absent players: panel shows "INSERT COIN" or "JOIN-IN" in turn.
- Rules: cursor order Jack, Hannah, Mustapha, Mess; a character can be taken once; the 10 second timer
  (`Select` struct in `front.rs`: seconds G.$62C+9, frames +$A) picks the character under the cursor at 0;
  a joining player starts its own panel and the timer is restarted [I]. Press attack (A) to choose.
- Sound: a short tune; select sound per move; voices of the heroes when chosen [I, not checked].
- Original helpers: 0xAF7C (panel draw), 0xB06A (cursor), 0xB122, 0xB25E, stats 0xB444 [R] `player/SPEC.md` 7.
- Port: STUB (`flow_select_mode`). Needs: portraits, panels, star bars, group picture (scroll 3 or objects?).

## 6. Joining in, continue, game over [M] [R]

- **In-level prompts** (task 0x6C4A, redraw 0x7562). For absent players the HUD block shows a blinking
  "INSERT COIN" (no credit) or "JOIN-IN" (credit) text: 48 frames on, 80 frames off (spec) [R]; seen in all
  MAME screenshots. Pressing start with a credit joins the player: a quick select panel over the level,
  then the hero drops in with invulnerability, a speech bubble and a bazooka (`item_give_bazooka` 0x65A4
  is ported). Port: STUB (`flow_join_task`, `flow_join_prompts`).
- **Continue** (mode 0x10, 0xE9E6, per player states 0..0xC in task 0x6C4A) [M f=1557..2856]: the villain
  with the gun (art made of scroll tiles), a speech bubble and a big two digit number counting 20 to 0,
  62 frames per number. The bubble text changes at 20, 15, 10, 5 (four taunts). Press start with a credit:
  the "SELECT PLAYER!" strip (10 s counter, 10 to 0) appears on the level background and the player can
  pick any hero; the hero drops in with a bazooka [M f=1801..1842]. Coins during the countdown also
  seem to speed the counter up [M, not understood]. At 0: red flash, blood spots, "GAME OVER".
- **Game over** (0xAC06 / 0xAC26, mode 0x14 then 0x18) [M f=3339..3347]: after about 480 frames of the
  GAME OVER picture the ranking and the attract loop follow. The port's `flow_game_over` already sets the
  flags and calls `flow_attract_start()` (which is empty).
- Rank effects of dying and continuing: `rank_event_continue` exists in `rank.rs` [R].
- Port: STUB (`flow_continue_mode` jumps to game over).

## 7. Story link scenes between episodes [M] [R]

- Order seen: tally, map panel, story panel (mode 8), episode card. See `README.md` section 2 F and the
  `episode-N.md` files (the story panel text of each episode is paraphrased there).
- Durations measured with a poke of the mode: story panel 306 frames before Episode 2, 250 (Ep 3), 309 (Ep 4),
  439 (Ep 5), 368 (Ep 6), 368 (Ep 7), 313 (Ep 8).
- Routines: 0xD898 sets palette 0x89E6 (d0 = 2), fade 0x892C, then a per stage entry (table at 0xD8D8:
  0xD8E8, 0xD97E, 0xDAB4, 0xDBB2, 0xDDCC, 0xDE6C, 0xDF1E). Each entry plays a sound (0xA63C with a code,
  0x13 before Episode 2, 0x14 before Episode 3), draws a picture block (0x98BAE: a w x h tile block from two
  index lists), loads strings (0x84F72), and waits. Entry 1 also has a rare random
  branch: a 1 in 16 random check (condition routine 0x1F18A) that ends in a call to 0x97D78. Its meaning
  is unknown (it looks like a rare easter egg or leftover). Not seen in MAME.
- Port: MISSING (`flow_stage_start` is called at once).

## 8. The ending (mode 0x1C) [M]

See `README.md` section 2 J for the scenes. Frame count 5365 from the first frame of the mode to the
game over modes. Objects used: pool A / B / C (scene actors, explosions), the scroll layers for the
desert, the text printer for the staff roll (strings at 0x8B7EA). The ending is a G.$4D1 state machine
(0xA69B8): sub 0, 4 (long sequence), 6, 8 (leave). Port: MISSING (`flow_game_over` is called instead).
Open question for D4: does the ending change with the number of players or the heroes chosen?

## 9. Open questions (for the design phase D4, front end part)

1. What exactly is shown by 0x98330 and when is the picture skipped (G.$32 == 2)?
2. Where is the ranking text printed (0x98330, 0x988C0 or a task)? How is a new score entered, if at all?
3. Who calls 0x16230 (store score and character in a ranking slot)?
4. Does a coin during the continue countdown speed it up? What do 2 coins do with the "2 coins start 1 coin
   continue" setting?
5. Does the join select panel keep the old character or open the full select?
6. What does the 1 in 128 random task at the title do (0x6A73A)?
7. Is the opening movie a pure timeline of pool objects (likely) and are its pictures all in scroll 3 data?
8. Sound ids for each screen (title tune, select, continue taunts, ending).
9. Does the 2 player cabinet layout (G.$4CF) change any screen (demo entries 0x9A1EE, HUD 2P layout)?
