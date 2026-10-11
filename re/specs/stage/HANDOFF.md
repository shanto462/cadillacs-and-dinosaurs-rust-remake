# Background / camera port - handoff

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
2. **Stage 2 controller 0x22398** (call it instead of `scroll2_task` when stage == 2; order in
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
3. **Rowscroll 0x2335A** (stage 2, struct G.$7780): pseudocode in SPEC.md 7.3; store 1024 words in
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
