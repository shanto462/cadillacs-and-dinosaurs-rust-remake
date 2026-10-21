# M10-10: Episode 4, one player

2026-10-21, worktree `wt/codex-m10-10`, baseline `ab4c87a`.
Only M10-10 was done. M10-11 remains the next task.

## Done

Checked all 28,001 records of the existing natural `ep4_jack` MAME recording,
through Slice, the tally and the Episode 5 transition. No game logic change was
needed. Fixed `m10_check.sh` to write and compare port traces using `PORT_OUT`,
with a worktree-local default. MAME references stay in the shared trace folder.
Added narrow recording/mode rules to `allow/m10_runs.txt`, with causes.
No ROM data, extracted tables, commits or pushes were added.

## Gates

- G1: `cargo build` passed. `cargo test` passed: 698 tests, 12 ignored,
  zero failures. The ending autoplay test passed.
- G2: `re/harness/regress.sh`: 52 of 52 at zero, baseline 52 of 52 at zero.
- G3: `PORT_OUT="$PWD/re/out/port" re/harness/m10_check.sh ep4_jack:28001`.
  Step: 27,954 records compared, 47 lag records excluded. Before rules:
  12 fields and four video/sound categories differ, first record 27,497.
  After rules: zero fields and zero video/sound categories.
  Lagsync: 27,952 records compared, 49 lag/catch-up records excluded,
  49 resyncs. Before rules: zero fields, six story-picture records and one
  repeated sound, first record 27,579. After rules: zero differences.
  Run: equal records 0 through 2,756 plus the sync state. First difference
  is lag record 2,757. The full run tail has 500 differing field names and
  six video/sound categories. No rule hides that tail.
- G4: one headless screenshot run from `ep4_jack.state`, using
  `re/harness/scripts/ep4_jack.txt`, 28,001 ticks, screenshots at
  600, 2400, 9000, 24000 and 28000. Viewed all five images in
  `re/out/shots/m10-10/`: waterfront, street fight and dropped weapons,
  garage and chain-link fence, foundry fight, Slice and his boomerangs.
  This is normal free play with video ticks, so its later pictures are
  visual checks, not frame equality claims against game-clock records.
- G5: no ported routine or data extraction was added. Shell syntax and
  `git diff --check` pass.
- Separate `dino-game --headless --autoplay --frames 600000` run reached
  mode 0x1c at frame 343,802, then the attract loop at 349,422.

## Issues and causes

- Step records 27,497 and 27,532: skip task 0x9E38C runs five game frames
  in MAME, versus the port's fixed three. All differences are two missing
  updates: counters, tally flash and B59 palette phase. This is existing
  cause K, the approved CPU-time limit. Lagsync resyncs after each record.
- Record 27,579: Episode 5's panel began inside lag record 27,578.
  The loader cannot restore the sleeping story routine, so 0xDBB2 repeats
  its entry sound. This is existing cause S.
- Story pictures at step records 27,609 to 27,613 and 27,616 to 27,621;
  lagsync records 27,609, 27,611, 27,613, 27,616, 27,618 and 27,620:
  the loader does not restore `Story::sub` from the sleeping routine.
  MAME's saved returns 0xDC72 and 0xDC90 identify the two picture phases
  of 0xDC54. A resync keeps the Rust phase instead. Cause S, no game
  routine was changed to compensate for the loader.
- Run mode after 2,757: original slowdown is not modelled, by plan decision.
- Sound arguments are separate from diff.py's equality summary. In lagsync,
  50 frames of 2,514 paired requests have different arguments, first at
  record 178. Codes: 0x18, 0x39, 0x3A, 0x3B, 0x3E, 0x62, 0x63, 0x66.
  Stale caller registers and hit-sound facing/pan are the known cause family.
  Exact caller tracking remains M10-22 work. No allow rule hides arguments.

## Follow-up

M10-11 Episode 5 is next. M10-22 should use the sound argument frame list
in `re/out/m10-10/lagsync-detail.txt`. Restoring the sleeping story phase
from the task stack could remove the cause S rules, but was not needed
for Episode 4 gameplay acceptance. No open Episode 4 gameplay question.

Evidence is worktree-local under `re/out/m10-10/`, with the initial check,
unit-test and regression logs at `re/out/m10-10-*.log`. Port traces are in
`re/out/port/`. No new MAME recording was needed.
