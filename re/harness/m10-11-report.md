# M10-11: Episode 5 whole runs

2026-10-21, worktree `wt/codex-m10-11`, baseline `01d5845`.

## Done

Checked the existing natural recordings of Jack, Hannah, Mess, Mustapha and
three players, each for 65,000 gameplay records plus its sync state. The
references and lagsync runs reach the Episode 5 tally and Episode 6. No new
MAME recording was needed.

Fixed two snapshot loader faults found by the whole runs:

- Camera events, 0x2099A / 0x209C0: prefer the current stage's event list when
  decoding its saved pointer. The old stage's base could accept the same pointer
  on its 16-byte grid and produce the wrong index. Jack first missed Episode 6's
  camera lock at record 55,394. This caused later camera and free-run drift.
- Typewriter, 0x7EC48: restore the pool B object's text peer from its saved read
  pointer, cursor, countdown, period and skip flag. The three-player transition
  creates it inside lag records 61,108 / 61,109. Without restoration,
  `prop_class26` freed it at 61,110 and lost six records of title text and the
  pool B free-list state. The text still comes from the runtime asset pack.

Added synthetic regression tests for both faults. Added narrow step-mode
rules to `allow/m10_runs.txt` for the confirmed skip timing limits below.
No ROM data, extracted tables, commits or pushes were added.

Gates:

- G1: `cargo build` passed; `cargo test` passed, 700 tests, 12 ignored,
  zero failures. Both new loader tests and ending autoplay passed.
- G2: `re/harness/regress.sh`: 52 of 52 at zero, baseline 52 of 52 at zero.
- G3: `PORT_OUT="$PWD/re/out/port-m10-11-final"` with
  `re/harness/m10_check.sh`, each name below followed by `:65000`.
  Step and lagsync results are below. All have zero differences after the
  listed rules. Lagsync needs no allow rule for any of these recordings.
- G4: one headless play run from `ep5_jack.state`, with
  `re/harness/scripts/ep5_jack.txt`, 65,000 video ticks and screenshots at
  600, 12,000, 40,000, 54,000 and 65,000. Viewed all five in
  `re/out/shots/m10-11/`: village approach and burning enemy, cave fight and
  dropped guns, burning village, Morgan with his gun and the next episode's
  ruined buildings. These are visual checks of free play, not frame equality
  claims against the game-clock records.
- G5: original addresses are documented; no new `Game` methods or ROM table
  copies. `git diff --check` passed.
- Separate `dino-game --headless --autoplay --frames 600000` reached mode
  `0x1c` at frame 343,802 and the attract loop at 349,422.

| Recording | Step records compared | Step before rules: fields / video categories | Lagsync records compared | Resyncs | First run difference |
| --- | ---: | ---: | ---: | ---: | ---: |
| ep5_jack | 64,921 | 15 / 3 | 64,918 | 83 | 6,135 |
| ep5_hannah | 64,951 | 19 / 3 | 64,950 | 51 | 8,477 |
| ep5_mess | 64,943 | 11 / 0 | 64,942 | 59 | 8,362 |
| ep5_mustapha | 64,931 | 0 / 0 | 64,931 | 70 | 3,223 |
| ep5_3p | 64,903 | 0 / 0 | 64,903 | 98 | 1,993 |

Compared counts include the sync state. Step excludes flagged MAME lag
records; lagsync also excludes the five catch-up records below. Raw lagsync
has zero fields and zero video/sound categories different in all five runs.

## Issues

- Cause K, skip task 0x9E38C: Jack records 40,143, 40,150 and 40,161, and
  Mess record 45,375, run five game frames in MAME versus the port's fixed
  three. Each is scripted dialogue/entrance with attack held. Hannah record
  57,870, the stage-clear walk and tally, runs four versus three. The missing
  task passes explain the counters, movement, palette phase, text, tally
  numbers and allocations. Four unnamed tally bytes at that Hannah record
  share this cause. Each rule is limited to its field and single record.
  Lagsync excludes and restores these records. No gameplay bug is allowed.
- Run mode: every first difference in the table is the first MAME lag record.
  All prior state, video and sound-code records and the sync state match.
  Original slowdown is not modelled, by plan decision. The full run tails diverge; no rule allows them.
- Sound arguments remain separate from diff.py's equality summary. Lagsync
  sound codes agree, but caller register leftovers and shared hit sound
  mode/pan still differ. These remain M10-22 work, without allow rules:

| Recording | Frames with argument differences | Paired requests | First record |
| --- | ---: | ---: | ---: |
| ep5_jack | 143 | 5,141 | 3,132 |
| ep5_hannah | 172 | 5,242 | 213 |
| ep5_mess | 169 | 4,763 | 908 |
| ep5_mustapha | 168 | 5,139 | 762 |
| ep5_3p | 203 | 13,343 | 282 |

The initial diagnostic batch requested one record beyond the references.
Final runs use exactly 65,000; those are the acceptance traces.

## Follow-up

No unexplained state, video or sound code difference remains in these
comparisons. M10-22 should use the per-record
`ep5_*-sound-arguments.json` lists in `re/out/m10-11/` for exact caller/pan
tracking. Codes include the known GO hint, hit sounds, landing voices and
boss music calls. No typewriter or story-panel rule was needed here.

Evidence is worktree-local: `re/out/m10-11/accepted-check.log`,
`final-run-check.log`, `build-final.log`, `test-final.log`,
`regress-final.log`, `autoplay-final.log`, `metadata.jsonl`, `sounds-final.jsonl`,
the first-difference details and the two loader tests. Port traces are in
`re/out/port-m10-11-final/`; MAME references were only read.
