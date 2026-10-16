# Background / camera port - handoff

## M3-08 to M3-12: the Episode 3 camera, the car drive camera and the row scroll (2026-10-17)

Done: stage 2 (Episode 3) runs its own scroll2 controller and the row scroll task, as `0x2042E` does (`background_update`: stage 2 = 0x2335A, 0x22398,
0x22CAA; the others 0x206D4, 0x22CAA). The foot road of Episode 3 runs end to end in the port with the placeholders (`--episode 3`, kill all every frame, R held: ten locks, the Hogg placeholder
at X 0x2D00, the clear tally, the story panel and the card of Episode 4, 12800 frames), with the locks, the row scroll and the map equal to MAME where
a recording reaches them; the car drive camera is ported and waits for the car object of M6.

| original | port (`game/camera.rs`) | notes |
| --- | --- | --- |
| 0x2335A (0x23370 init, 0x23390 run) | `rowscroll_task` | struct G.$7780 (`Camera::row`: state +$04, base word +$08). Init clears 0x915800 .. 0x918000 (`Vram::rowscroll`, 0x1400 words) and falls into the run in the same frame. Run: the 32 words 0x90 .. 0xAF (screen lines 128 to 159, word index = line + 16) from the camera X, 68000 word and long arithmetic as in the original. |
| 0x22398 | `scroll2_task_stage2` | state 0 = `scroll2_init_stage2` (0x223AE), state 2 = `scroll2_run_stage2` (0x224A4). |
| 0x22404 | `stage2_reinit` | the map of the controller: header 0x22AFA (`Stage::controller_map`), 32 columns of 14 blocks from x - 0x20 (x = 0: from 0), no pre-draw at +0xBC8. `Scroll2::own_map` says which header ran last (the demo's area setup 0x20766 uses the table's header). |
| 0x224A4 | `scroll2_run_stage2` | G.$518 != 0 forces mode 8. The event check (X only, unsigned `ev.x <= camX`) is skipped in the modes 1, 3, 6, 8. Extents, follow X and follow Y are the standard routines (0x226xx are copies of 0x20Axx). |
| mode table 0x2250A | 0 follow, 1 `mode_area_end`, 2 `mode_lock_stage2`, 3 `mode_stage_end_stage2`, 4 `mode_sub_area`, 5 camX += 3, 6 `mode_area_change`, 7 `mode_stage_end_fade`, 8 `mode_car_chase` | modes 1, 4, 6, 7 are the standard routines (the stage 2 copies differ only in `reinit` 0x22404 and in the missing hook 0x9236C of mode 4: `stage == 2` tests in `area_end_next_area` and `mode_sub_area`). The event table 0x22B9E has only the modes 0 and 2; the car sets 8 and 3. Modes 1, 4, 5, 6, 7 are not reachable in the game and not measured. |
| 0x22A04 mode 2 | `mode_lock_stage2` | the fight lock. QUIRK kept: G.$7B4 (`lock78`) is set only in a vertical scene, so it stays 0 during the foot locks (MAME: lock = 0 in every record of `e3footn`, `m308_foot*`). No stream. Ends when `go` is set: mode = next mode (0), sub 0. |
| 0x2290C mode 3 | `mode_stage_end_stage2` | the end of the drive: sub 0 stores the camera X as the event X (`x4c = camX`), lock state 2, then holds it; `go` raises G.$4D4 (`w.stage_clear`). The go flag is not cleared. No stream. |
| 0x22A68 mode 8 | `mode_car_chase`, `car_chase_sub2`, `car_chase_sub4`, `car_chase_end` | sub 0 steps to 2 and goes on at once. G.$518 = 1: camX += clamp(car X - (camX + 0xC0), 8, 20) (word subtract: a negative word gives 8). G.$518 = 2 (time over): the camera takes the car's velocity +$14 and acceleration +$1A, goes to sub 4 and integrates (0x189C = `Scroll2::integrate`, X, Y and Z of the 16.16 camera; Z is the new field `cam_z`, G.$74C). G.$518 = 0: velocity X and acceleration X cleared, mode 3, sub 0. Stream (0x2126E) follows in every case. The car is `Camera::car` (G.$51A, a handle), read through `Game::body`. |
| 0x2A12C (radio) | `item/small.rs` `item_treasure_collect` | still the stand in: the radio gives its 3000 points, the event 0x8BD24 (car, G.$4D5, G.$4DB, G.$50E) is M6. |

Renderer (M3-12): `present.rs` `draw_scroll2` draws scroll2 in bands of lines with the same offset `rowscroll[(s + 16) & 0x3FF]` when the video control bit 0 is set
(G.$26, set by the stage 2 init, not in the demo) and some offset of the 224 lines is not 0; a tile row that a band cuts is drawn in pieces (`TileDraw::rows`, lines r0 .. r1
of the tile; `render.rs` crops the source and destination rectangle, `headless.rs` limits the lines). With the bit off, or all offsets 0, the code draws the same whole tiles as
before (unit test `without_row_scroll_the_tiles_are_whole`). The table is shown at once like the tile RAM (the scroll registers are the shadows of two frames ago): checked on pictures
with the camera moving. The GPU path (`render.rs`) is written but was not run (no window): the headless rasterizer is what the pictures show.

Trace (`game/trace.rs`): `cam_z` (G.$74C), `rowscroll_state` / `rowscroll_base` (G.$7784 / G.$7788), `car_handle` (G.$51A), `map_ptr` (G.$77E, stage 2 only, once the controller runs:
the screen array pointer 0x22B04 of the controller's own header or the table's 0x2173E, kept as `own_map`), the row scroll RAM 0x915800 .. 0x918000 in the gfx region (compared by
`diff.py`, new item `rowscroll`). The wave controller (pool B class 15) pointer +$A0 of a controller that has ended (mode 6) is the end word + 6, as in RAM (the port keeps the group index).
`story_allow.py`: the rule that allowed all scroll2 differences of Episode 3's level ("the camera map of stage 2 is not ported") is gone, and `map_ptr` is a derived pointer of the stage that is poked in.

Scenarios (`re/harness/pokes/m308.txt`, one MAME process, 9 segments; check script `re/harness/m308_check.sh`, allow rules `re/harness/allow/m3.txt`, trace viewer `re/harness/watch.py`):

| scenario | what | result in `step` mode |
| --- | --- | --- |
| `m308_foot1p` | Jack alone from X 0x2C0, no kill all: the empty waves at 0x300 and 0x500 | 1000 frames, no difference at all (camera, row scroll words, all tilemaps, every object) |
| `m308_foot3p` | three heroes, kill all every frame, locks 0x300, 0x500 and the wave of four bikers at 0x900 | camera, row scroll and the three tilemaps equal in all 1500 frames; the first difference is frame 729 (biker and Hammer placeholders, unported classes) |
| `m308_row300`, `row1000`, `row2400` | D4c A13: the 32 row scroll words and the picture at three camera positions | no difference; the pictures at t = 22 (all three) and t = 32 (X 0x1000, 0x2400) are pixel equal with MAME, sprites included (at t = 32 of X 0x300 an enemy sprite of an unported class enters) |
| `m308_rowmove` | Jack runs, the camera moves at 3 pixels a frame | the road lines (128 to 159) are pixel equal with MAME at 8 frames in every column that no sprite difference touches (the sprites that differ are the placeholders of unported enemies) |
| `m308_car`, `car_end`, `car_time` | the radio at X 0x300 (D4c car1): car, boarding, G.$518 = 1 at t = 360, the chase; at t = 500 G.$518 = 0 (end of the drive) or 2 (time over) | the chase formula equals MAME in all 839 frames with the car X of the same frame; the end of the drive (mode 3, w14 = a1a = 0, x4c, `go` then G.$4D4) and the time over steps (sub 4, velocity and acceleration of the car, integrate) equal in every frame the car object does not decide |
| `link-ep3` (`link_check.sh`) | the level start of Episode 3: init 0x223AE, the 32 column map, the row scroll init | scroll2, scroll3 and the row scroll words equal MAME for all 1100 records (the first difference stays 915, the score digits of the HUD) |

Differences left, with causes:
* Car scenarios in `step` mode: the camera X is 3 pixels behind MAME in the whole drive. The original moves the car (pool B update) before the camera in the same frame; in `step` mode the
  car is an unported class and stands where it stood one frame earlier, 11 pixels back, so the step is 8 instead of 11. Exact with the car X of the frame (the check in `m308_check.sh`,
  the unit tests); it will be exact in `step` mode once the car (M6) runs. The slower scroll3, the stream of the maps and the hero extents follow (allow/m3.txt).
* Car scenarios: the pickup frame (the radio event is the stand in: G.$4D5, G.$4DB, G.$50E, G.$51A, the heroes' input lock, the three class 88 objects), the boarding jump of the heroes and
  the frames in which the car object sets or clears G.$518 (one frame each).
* The locks after X 0x900 and the last one (event 10, X 0x2D00, Hogg) are not compared with MAME: no recording of the foot road reaches them (`e3footn` ends before 0x900). Free run of the port
  with kill all: the locks last 32, 32, 573, 129, 891, 751, 32, 32, 32 frames (events 1 to 9: only the empty waves are 32 frames); the lock of event 10 stays (mode 2) while the Hogg placeholder dies
  and the clear chain of M3-03 / M3-04 runs (G.$4D4 at frame 10883, the tally, the panel at 12484, Episode 4 at 12793).

For M6 (the car, pool B class 88): make the car with `g.cam.car = handle` (G.$51A, set it in the same frame the original does, before the camera runs) and set `g.spawn.g.car_mode` (G.$518: 1 when the
heroes are aboard, 0 when the car is gone); the camera reads the car's X (`Body::x`), and its `vx` and `ax` for G.$518 = 2. The camera runs after all objects, so the car must already have moved when it does.
When the car exists, the car scenarios should lose most of `allow/m3.txt` (rules 1 and 3).

Open points for the main session:
* `Rank::b7b4` (G.$7B4, read by the aggression table choice 0x182A2, the music choice 0xA55A and the time over of the car) is the same byte as the camera's lock state `Scroll2::lock78`, but the camera
  never writes `rank.b7b4`: in a free run the second aggression table (lock = 1) and the boss music variant (lock = 2) are never selected, in any stage. It only shows in `step` mode as two fields
  that load the same MAME byte. Fix: let rank.rs and flow.rs read `cam.s2.lock78` and drop the field (and its trace line). Not done here (rank.rs, flow.rs are other owners' files).
* Review aids: F6 (`debug::next_area`) sets mode 0x12 sub 4, which is not in the stage 2 mode table (Episode 3 has one area): the camera would stand still. `--area` above 0 for Episode 3 is not meaningful.
* `--camx` (`flow_jump_camera`) moves the camera but not the event index; the events load one per frame afterwards, so a jump to X 0x1000 ends in the lock of its event.

# Older notes (before M3)

Owner files: `dino-game/src/game/camera.rs`, `game/vram.rs`, `src/present.rs`,
`dino-assets/src/stage.rs`, `dino-extract/src/stage.rs`. Format reference: `SPEC.md` (same folder).

## State at hand-off
- Reviewed `camera.rs` against the 68000 code (0x206D4..0x2152C, 0x22CAA..0x2311C). The ported
  scroll2 controller (init, events, follow X/Y, modes 0x00-0x13, stream2) and scroll3 (init,
  parallax, stream3) match the original, including the signed/unsigned compares. No logic bugs found.
- Changed (this session, compiles):
  - `Camera::default` scroll1 camera is now (0, 0) (was Y 0x100). In game 0xAE0 clears it, and the
    HUD lives at scroll1 rows 34+ (0x90A014), which is only visible with Y = 0. 0xAC4E sets
    Y = 0x100 for a non-game screen (flow port can set `g.cam.cam1_y`).
  - New API `g.cam_shake(strength)` (stores G.$755C in `g.cam.shake_req`). Mode 5 (elevator) now
    calls `cam_shake(2)` like 0x21128. `screen_shake()` itself is still a stub.
- Build: my files compile; the tree did not build at hand-off because of other owners' files
  (flow.rs / spawn.rs duplicate `spawn_area_reset_objects`, hud.rs). No new screenshot taken.

## Stubbed / not ported (in order of the next steps)
1. **Screen shake 0x222D6** (`screen_shake`, struct G.$755A: +0 state, +2 request, +3 strength,
   +4 timer, +5 sign, +6 amplitude word). Exact logic:
   - state 0 (0x222EE): `str = req`; if 0 -> idle (G.$7B8 = displayed scroll1 Y, clear state and str).
     Else state = 2, req = 0, timer = 0, amp = str, sign = 0xFF, then run state 2 now.
   - state 2 (0x22316): if `req != 0 && (req as i8) > (str as i8)`: str = req, amp = req, sign = 0xFF.
     req = 0. `off = if (sign as i8) < 0 { -amp } else { amp }`.
     Stage 6 area 1: displayed scroll1 Y = cam1_y + off; else displayed scroll2 Y = cam_y + off
     (only the BG register: sprites and the camera do not move; it lasts one frame).
     `timer -= 1`; if it borrowed (was 0): timer = 2, sign = -sign; if sign is now negative:
     amp -= 1, str -= 1, and str == 0 -> idle.
   - Implement as `cam.shake_dy2 / shake_dy1: i16` reset each frame, added in `present.rs`.
2. **Stage 2 controller 0x22398 (DONE in M3-08 / M3-09, see the top of this file)** (call it instead of `scroll2_task` when stage == 2; order in
   0x2042E: rowscroll 0x2335A, then 0x22398, then scroll3). Differences from 0x206D4:
   - init 0x223AE: no tile-anim init, no area_setup; always `video_ctrl |= 1`; events from 0x22B9E
     (already extracted for stage 2); header copy 0x22AFA (same data as the 0x21580 entry);
     reinit 0x22404 fills **32 columns** (d7 = 0x1F in both branches) of 14 blocks, no 0xBC8 pre-draw.
   - run 0x224A4: if G.$518 (car state) != 0 -> mode = 8. Event check (X only, unsigned) skipped
     for modes 1, 3, 6, 8. Then player extents, then mode table 0x2250A:
     0 = follow_x/follow_y/stream2; 1 = area end with fade (subs 0x2278A/0x22798/0x227D0/0x22826/
     0x2284C, reinit = 0x22404); 2 = lock 0x22A04 (QUIRK: lock78 = 1 only when vertical_scene);
     3 = stage end 0x2290C (sub0 also sets $4C = camX); 4 = sub-area 0x229AE (like mode 8 without
     hook 0x9236C); 5 = camX += 3 + stream2; 6 = area change without fade 0x22864; 7 = stage end
     with fade (mask 0x0D) 0x22956; 8 = **Cadillac drive** 0x22A68, always followed by stream2:
     sub0 -> sub2; sub2: if $518 == 2 { sub=4; cam vx/ax = car +$14/+$1A; integrate (0x189C, X/Y/Z
     with the camera's own +$16/+$1C/+$18/+$1E - keep, they may hold old values) ; if $518 == 0
     -> end } else if $518 == 0 -> end else { d = car.x - (camX + 0xC0); camX += 8 if d < 0,
     else clamp(d, 8, 20) }; sub4: integrate, if $518 == 0 -> end. end (0x22AE6): vx = ax = 0,
     mode = 3, sub = 0. Needs `car_state` (G.$518) and `car` handle (G.$51A) from the car port.
3. **Rowscroll 0x2335A (DONE in M3-11 / M3-12, see the top of this file)** (stage 2, struct G.$7780): pseudocode in SPEC.md 7.3; store 1024 words in
   `Vram` (`other`). `present.rs`: screen line s uses scroll2 X + other[(s + 16) & 0x3FF].
   Exact per-line drawing needs a clip in `render.rs` (main session): e.g. `TileDraw.rows:
   Option<(u8 first_src_row, u8 count)>` cropping the source rect and dest height. Without it the
   only option is one offset per 16-line tile row (approximate).
4. **Stage 6 area 1 elevator** (in `area_setup`): `fill_scroll1_bg(0, 0x100, 1)` and
   `(0x100, 0x100, 2)` = 0x205C6: 8 columns x 9 metatiles; address 0x20650 (scroll1, 32-px
   blocks), screen ptr 0x206A8 (`0x138C48 + id*128 + (x&0xE0)/2 + 0xE - (y&0xE0)/16`), metatile
   writer 0x20614 (64 bytes at 0x15CFE8 + n*64, 4 cells down then next column). The first
   metatile is the screen's bottom row at rows 28-31, then rows 32-63; it is visible with
   cam1_y = 0. Needs the two tables extracted (add to `dino-extract/src/stage.rs`).
   Area 2: `vram.clear1()` = 0xB52 (code 0x20, attr 6 in every scroll1 cell), then `hook_162ae()`.
5. **0xBC8 pre-draw** in `reinit_map` when init X = 0 (0x20856 -> 0x20520): at cell index
   `vram2(0, initY) + 0xBC8/4` (0x20856: d0 = 0, d1 = initY), 8 columns (+32 cells each) x 8 blocks
   (+2 cells each), every column draws the FIRST 8 words of screen `screen_id2(0, initY)` (0x20524
   reloads a1 each column). Probably fills tilemap x 768-1023 for stage 2's rowscroll.
6. `cam_reset()` for stage transitions = 0x20402: clear s2 state/mode/sub, cams, x5a/y5c, s3
   state, s3 cams, then `vram.clear2(); vram.clear3()`.
7. Screenshots for every stage (`--stage N`); add `--area N` if needed (start the camera at the
   area's first event: set `cam.s2.event_idx` to the index of the area's X == 0 event).

## Known issues / notes
- `screen_lookup` returns screen 0 for indexes outside the array (original reads ROM garbage).
- `write_cell3` skips areas without a scroll3 map (original draws garbage hidden behind scroll2).
- Mode 0x0E debug attribute view (0x21474) not ported (debug only).
- Fade delays written by camera modes are dead stores (task resets delay to 1): already handled in
  `palette.rs::start_fade_task`.
