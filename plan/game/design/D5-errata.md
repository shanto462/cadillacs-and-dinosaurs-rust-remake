# D5 errata

Corrections to the design documents found while building (append only, one line per correction, newest last).
MAME wins over the design text; the design documents themselves are not edited.

- D4d s2 (boot skip): "press at 100: both screens end at 226" is wrong for text 1. A press at probe frame 100 ends the wait of text 1 early (text 2 starts at once), but text 2 still shows its full 30 + 91 frames from its early start, so the boot ends 121 frames after text 2 starts (video frame 102) and the layer control change comes 3 frames later, at video frame 226. Text 2 is only skipped by a press after its first 30 frames (boot-skip-b). M1-03.
- D4d s2, s3.2, s13 A2: frame numbers of the probe batches are video frames; a record of the harness traces is video frame - 2, and a press of a probe line `f` is seen one frame earlier by a task than the harness script line `f` (the same press gives the layer control change 0x18D6 at video frame 226 / 234 in the probe and at record 224 / 231 with `98-101:A` / `228-231:2J`). M1-03.
- D4d s4.5 and F35 (G.$4F6): the counter feeds nothing. The system task clears it (0x1827A with G.$4CC set) in the pass that creates the game task, two frames before the game start code 0x17F8A reads it, so the starting rank band is always 0; and with the factory settings (three chutes, credits per player) it never counts, because it only runs while the common pool G.$46A or free play G.$46B is not 0. M1-04.
- D4d s3.2 (attract timeline): the lengths of the profile cards (1399 frames the first time, 1407 afterwards), of the title entry and of the ranking (422 then 423) are not constant; the whole loop is 13104 frames only on average. The measured lengths are in `attract.rs` and `TITLE_RUNS` in `boot.rs`. M1-04.

