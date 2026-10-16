# The ending (game mode 0x1C): what the code does

Written for M3-23 to M3-27 from the ROM code (`re/tools/d`), checked against the recordings `flow_end-*` (video clock, from power on) and
`m328_end-*` (game clock). Port: `game/flow/ending.rs` (the routine 0xA69B8), `game/prop/b82_85.rs`, `game/enemy/class90.rs`, `class92.rs`,
`class93.rs`, `class96.rs`. The scene timeline is in `plan/game/design/D4d-front-end-and-story.md` section 8.

## The routine 0xA69B8 (mode 0x1C, jump table 0xA69E6 on G.$4D1)

Each frame, as soon as G.$4D1 is not 0: the shake 0x222D6, pool B 0x23604, pool A 0x23420, the sprite list 0x13FA6 (in that order, no pool C,
no hero, no HUD). Then the sub state: 0 clear all and setup, 2 make the master (pool B 82), 4 count G.-$6FFA down from 0x14F0, 6 wait while a
player is in the name entry (0xEFC2: join state 0xA), 8 sleep 180 frames, mode 0x14, clear all. Sub 0 and its lag: the clear keeps the 68000
through two video frames (scroll1 and the sprite buffers blank after the first, scroll2 and part of scroll3 after the second, the rest of scroll3
and the second half of the sub in the third, where the pass of sub 2 follows in the same frame). The pool inits write the tag +$82 (4 in pool
A, 8 in pool B) into every slot and keep the other bytes of the slots (a new object sees what the old one left in +$80 and up).

## Pool B 82 (0x8AA0A), the master of the scenes

Mode 0 brightness; mode 2 runs the state `+$05` (table 0x8AA46). Times are frames after the start of the ending (record of `m328_end-jack`):
0 (record 4): palette set 3, clear all, then in the third frame the pages of scroll2 6, 7, 8, 6; 2 (record 6): four pool A 90 (variants 0 to 0x300,
X 0x20 / 0x10 / 0 / 0, Z 0x10 / 0x50 / 0x30 / 0x20), pool B 83 and 84, caption 0x69 (HURRY UP!!); 4: 460 frames of the corridor (`+$A0` counts
down from 0x1CC: showers at 0x12C, 0xDC, 0xC8 (variants 0x600 and 0x700), caption 0x6A at 0xCD, the camera speed `+$A4` 4, 2 at 0xD2, 0 at
0xCD, the shake every 16 frames from 0xC8); 6, 0xA, 0xE, 0x12, 0x18 waits; 8: camera 0, pages 9, 0xA (scroll2), 0x65, 0x66 (scroll3), two
pool A 96; 0xC: pool B 3 variant 4 (the wall breaks, M3-19) and the sound 0x56 (d1 0x50020, d2 0x800000); 0x10: pool C 21 variant 0xE
(0x94640, never updated here: the ending does not run pool C), two pool B 85 and one 83 (variant 0x100); 0x14: every pool A object gets
mode 6 and HP 0, pages 0xB, 0xC, 0x68, 0x69, pool B 84 variant 0x10, caption 0x6B; 0x16 (0x13C frames): six scroll2 palettes cycle (0x8B058,
the table 0x8B112 of offsets into 0xBCA52), at `+$A0` = 0xC0 the staff roll (pool B 87), then both layers are cleared (a lag frame); 0x1A: layer
control 0x12D6 and pool B 81 (the desert master), the master ends. Layer control 0x18D6 is set at state 0x14.

## The showers and the pieces (pool B 83, 84, 85; pool A 90, 92, 93, 96)

* Pool B 83 (0x8B212): a piece (pool A 92 with its variant) every 4 frames, the rumble 0x9F12 (sound 0x66, d1 = 5 or 6 (+$24) << 16 | 0x1200 | pan,
  d2 0x1561080) every 16, for 200 frames (variant 0 and 0x700), 50 (0x100) or 180 (0x600). It is made by a lower slot than the master's, so it runs
  first in the next frame.
* Pool B 84 (0x8B2CA): a pool A 93 every 32 frames for 400 frames, variant 0x10: every 8 frames for 100.
* Pool B 85 (0x8B34C): 100 frames: shake request 4 and the rumble every 32 frames; every 4 frames a pool A 92 variant 0x500 at X from the table 0x8B402
  (0 to 0x30 and back), Y from 0x8B412 + rand() & 0xF0 (the long `+$A2` that offsets them stays 0).
* Pool A 90 (0x650FA, table 0x6531C, 8 animations): the cloud next to the heroes. High byte of the variant: 0 and 1 run the animation 0 / 1 at speed 1.125
  / 1.0 with an extra 1.0 at timer 0xD2; 2 changes speed and animation at 0xBE, 0xAA, 0x96; 3 at 0xD2, 0xCD (sound 0xBB, d1 0x10), 0x96. It lives 460 frames
  (+$B2 from 0x1CC), moves with `+$AC += +$A8` and its X is the camera X (G.$744 as a long) plus `+$AC`.
* Pool A 92 (0x676FE, table 0x678A0): the piece. Variant 0: X = camera X + rand() & 0x1F0 ..., a random animation 0 to 3, speed 0x38000 + rand() & 3 in the high
  word; 0x100: Z 0x80 and no shadow; 0x500 (quake): speed 0xC0000 + rand() & 7, animation 6; 0x600: animation 2; 0x700: a 1 in 16 chance to end at once, animation
  5, Z from G.$74C; 0x800: nothing random. It is drawn every second frame only (`+$A4` counts) and ends with its animation.
* Pool A 93 (0x67B32, table 0x67C3A): a falling piece with the palette override 0x88; variant 0 starts at Y 0xE0, Z 0, X = camera X + rand() & 0x1F0, speed
  0x28000 + rand() & 3, animation rand() & 7, and makes a variant 0x100 (speed 0x4000, animation 8) with a 1 in 32 chance each frame; ends below Y 0.
* Pool A 96 (0x69C98, table 0x69E00): a flying piece of the sunset: 60 frames out of sight (no draw), then 80 (variant 0x100: 100) frames of flight with
  vx 0x480 / 0x580, vy 0x440, gravity -0x18; sound 0x10 with d1 = 8 (the jump table offset left in d1 by the dispatch), the variant 0x100 adds the break sound 0x9E60.
  `moveq #$10,d0` replaces the `move.w #$56,d0` before it.

## Differences between the six recordings

The cast does not matter (D4d): the heroes are removed at sub 0. The cut of the screen clear at the end of the first frame depends on the work of the
frame before the pass (cells 1367 to 1384 of scroll2 in the six recordings, `CLEAR2_FIRST`).

## The still picture, the desert and the end (M3-24 to M3-27)

* **State 0x16 of the master and the still picture.** The picture is the scroll2 pages 0xB, 0xC and the scroll3 pages 0x68, 0x69 with the camera at 0 and the palettes of
  scroll2 (20 to 25) cycling; the caption 0x6B (JACK! HANNAH!) is pool B 66, the debris showers go on (pool B 84 variant 0x10). At `+$A0` = 0xC0 the master makes pool B 87, the
  staff roll. The end of the state is a pass that overruns the video frame: the two layer clears (0xB6C, 0xB88) keep the 68000 past the frame end, so scroll2 and the first 3584 cells of
  scroll3 are cleared when it ends and the rest of scroll3, the cameras and the sub state come at the start of the next frame (`Cont::StillEnd`).
* **Pool B 87 (0x8B60E), the staff roll.** Prints the first line (STAFF, attribute 1) at the scroll1 cell of column 0x24, row 0x1E (0x90B278), waits 120 frames, then moves G.$6C8
  (the long Y.fraction: `Camera::cam1_y_frac`) up by 0x6C00 a frame and every 32 frames (`+$A0` & 0x1F = 0) prints the next line two rows lower (rows from 0x20 on are the first half of
  the column); 0xC00 frames in all, then G.$6C8 = 0 and the object ends. A line is 12 bytes of the ROM at 0x8B7EC (letters A to Z are the tiles of the table 0x8B786, other bytes tile
  codes, '%' an A with the attribute bit 0x40, '@' ends it), the attribute of a line is the word of the table 0x8BC84 (1, red header 2). The 96 lines run past the 66 of the text into
  blank ones.
* **Pool B 81 (0x8A5FA), the desert.** Made by the master at the end of the still picture (record 969). Mode 0: the sound stop and music 0x19, scroll3 X + 0x180, the pages (scroll2 0x16
  four times, scroll3 0x6C six times), the walkers (pool A 81 variants 1 and 2), the camera speed `+$AC` = 1, the timer `+$A0` = 0xBA4. The camera X goes back by `+$AC` pixels a frame
  and scroll3 X by 1/16 pixel (G.$7E4 as a long: `Scroll3::cam_x_frac`) while `+$AE` (0x1036 frames) lasts and the sub state is below 0x32. Sub states: waits (13 of them) and 0x02 music
  0x1A (d1 = 0x48, d2 = 0x1388, the leftovers the recording shows) with the car (pool A 81 variant 0, 2980 frames after the start); 0x06, 0x0E, 0x16, 0x1E, 0x26 the portraits and
  texts of the heroes with 0x84736 (Mustapha 0x35, Jack 0x36, 0x39, 0x3A, Hannah 0x38; the call leaves the portrait palette word in `+$A2` of the caller); 0x0A, 0x12, 0x1A, 0x22, 0x2A blank the
  text block (45 x 13 cells at 0x90A29C); 0x2C camera speed 2; 0x30 THANK YOU FOR PLAYING and PRODUCED BY (the big font 0xCE4, printed at once, not typed) and the CAPCOM sign (0x110C at 0x11, 0x1A);
  0x34 G.$507 = 0 (a name entry that waited may start) and the end. The whole run is about 4000 frames.
* **Pool A 81 (0x60158), the walkers and the Cadillac.** Variants 1 (Mustapha) and 2 (Mess) walk in the desert for 0xF6E frames with speed tables 3, 0, 1 / 2 at the timer values 0x37A,
  0x8C, 0x78 / 0x6E (they stop and turn); the car (variant 0, 0x3CA frames) makes its two heroes (pool A 17, variants 0 and 1, owner `+$AC`), drives in, stops and drives off. X follows the
  camera (`+$A8` keeps the camera X of the frame before as a long).
* **The end.** The sequencer's sub 6 waits for a name entry (a qualifying hero is in the join state 0xA, parked while G.$507 is set; B81 clears it), sub 8 sleeps 180 frames and the pass
  after it does mode 0x14, sub 0 and the clear 0xAE0, which keeps the 68000 through three video frames (sprites and scroll1, scroll2, scroll3). Two video frames later the mode 0x14 routine
  runs (mode 0x18) and the game over of the attract loop (the ranking, `flow_game_over`) goes on. The first frame of the ranking after the ending writes 2664 words (2662 to 2666 in the six
  recordings, `CUT_AFTER_ENDING`) and the ranking of a game over is never the "first run" of the attract (its second pass is not in the frame of the first).

## Pool A 97 (0x69EA2), the stand ins of the clear scene of Episode 8

The script action 0x42 (`script.rs`) makes one for every character (`enemy_stand_in_present` 0x69E32 at each playing hero, `enemy_stand_in_absent` 0x69E66 at X 0x10E0 for the others, depth
from 0x69E9A). They stand until all four are in place (the absent ones walk to X 0x1180), the boss (G.$4E0) has `+$A6` set, then: one pool A 93 piece, 120 frames, the dialogue of the
boss (pool B 50 variant 7, not ported), 180 frames, G.$7A9 = 1 (the camera moves on), they walk right 3 pixels a frame to X 0x1200 (the three heroes' X words follow) and on; the stand in
of Mustapha makes the aura (pool C 39, `+$20` = 0x9C, not ported). They live until the ending resets the pools. Checked in step mode against `m328_f_ep8` (records 1134 to 3874: no
difference in any class 97 field).

