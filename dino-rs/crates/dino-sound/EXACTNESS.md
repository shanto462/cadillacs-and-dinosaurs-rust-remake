# Driver exactness checks

The `compare` example generates request batches from the command headers loaded by
`dino-extract` from the user's `dino.zip`. It compares every DSP register address
and value, in order, without filtering volume, pitch, pan, echo or sample writes.
It supports the recorder's single additional `at=N` request for priority and
toggle checks. No ROM tables or reference register values are embedded in tests.

Run from `dino-rs/` to generate a music batch:

```sh
cargo run -p dino-sound --example compare -- --batch music ../re/out/sound/music-requests.txt
```

Run the batch from the worktree root:

```sh
SQ_INPUT=re/out/sound/music-requests.txt SQ_SOUND=none SQ_FRAMES=15000 SQ_TIMEOUT=1800 re/harness/probe/sound.sh
```

Then from `dino-rs/`:

```sh
cargo run -p dino-sound --example compare -- ../re/out/sound/music-requests.txt ../re/out/sound 15000
cargo test -p dino-sound --example compare
```

The CSV gives one row per request, a register difference count, the first differing
write and its frames/registers/values, timing counts and first timing difference,
the largest frame delta, the number of reference writes, the port tail length,
and the count of Z80 ROM bank writes. Missing or incomplete MAME recordings fail.
The example exits unsuccessfully for register differences, including early extra
port writes. It permits a port tail at the final frame boundary and runs two extra
frames to supply reference writes near that boundary. Its unit test checks wrong
values, wrong order, missing writes, early extras and the boundary allowance.

For effects, use `--batch effects` and a separate request file. Record 900 frames
per request, then compare 900 frames. The batch includes all 204 live effect
headers with fixed pan and with mode 7 delayed twins, six pan sweeps, five priority
pairs in both orders, and six toggle or player bank pairs. Priority pairs are
chosen from overlapping voices in the runtime headers and cover each lower
priority class. The five pairs and their order are visible in the generated file.
Archive the music traces and `qs-status.txt` before recording effects because the
recorder uses the same output directory. All traces, WAVs and CSVs stay in ignored
`re/out/sound/`.

The normal crate tests also sweep the first 600 frames of every music and effect
header, including the mode 7 effect variants, when those external fixtures exist.
They follow the older tests' convention of skipping unavailable fixtures. Use the
example for an acceptance run because it requires every requested fixture.
Set `DINO_SOUND_REQUIRE_BATCH=1` for the two sweep tests to fail if any fixture is
missing; their output gives the checked fixture count. To compare an earlier batch
after a later one overwrote the recorder status, set `DINO_SOUND_STATUS` to the
saved status file, such as `../re/out/sound/music-status.txt` from `dino-rs/`.

## Limits of equality

`registers_equal` means equal DSP register order and values. It does not mean
that the complete hardware trace is equal. The existing Python comparer
`re/harness/sound_cmp.py` checks frames and Z80 ROM bank writes too. Those checks
remain differences and must be reported:

- The native driver steps at 250 Hz and batches a whole sequencer pass into one
  tick. MAME executes the Z80 instructions, waits for DSP readiness, and can split
  a pass across a video frame. Batch requests also start at different IRQ phases.
  A constant `--align-first` shift does not remove all such timing differences.
- Z80 writes to D003 select its CPU ROM window. The native driver reads extracted
  linear data directly and emits no CPU ROM window writes. These are distinct
  from DSP sample bank registers, which are included in the register comparison.
- Audio output still uses the approximate mixer pan and echo model. Exact DSP
  register values do not establish audio waveform equality. WAV level checks do
  not replace a listening check.

Acceptance results use one common startup phase. No reference-value substitution
or ignored DSP register is used to make the register comparisons pass. The optional
`DINO_SOUND_IRQ_PHASE` (0 to 3) and `DINO_SOUND_FRAME_PHASE` (0 to less than 1 tick)
are explicit diagnostic seeds, not automatic per-code fitting.

## M4-10 result, 2026-10-19

MAME 0.289, one headless process, SDL dummy video and audio, sound disabled.
All 39 music codes were recorded for 15,000 video frames each (about 251.5 s).
This covers every listed tune length and the longest listed loop boundary.
All 13,492,675 DSP writes match in order and value. Music required no driver
change. The later delayed-twin correction affects SFX only; no RNG paths changed.

The strict Python comparer reports differences for all 39 codes, with no missing
traces. Its first difference is write 1, frame 0: MAME selects the Z80 ROM bank,
while the native driver starts with a DSP register write. The causes are listed
under Limits of equality. Across register-matched writes, 4,160,761 video-frame
labels differ, with a maximum displacement of two frames.

Per-code results follow. The timing count is separate from DSP register equality.
The CSV also records the first timing write and its frame on each side.

| Code | DSP differences | Timing differences | Max frame delta |
| --- | ---: | ---: | ---: |
| 0x00 | 0 | 244358 | 1 |
| 0x01 | 0 | 63770 | 1 |
| 0x02 | 0 | 167845 | 1 |
| 0x03 | 0 | 111331 | 1 |
| 0x04 | 0 | 110205 | 1 |
| 0x05 | 0 | 162154 | 2 |
| 0x06 | 0 | 87100 | 1 |
| 0x07 | 0 | 243103 | 1 |
| 0x08 | 0 | 55742 | 1 |
| 0x09 | 0 | 197617 | 1 |
| 0x0A | 0 | 87947 | 1 |
| 0x0B | 0 | 132549 | 1 |
| 0x0C | 0 | 91356 | 2 |
| 0x0D | 0 | 84191 | 1 |
| 0x0E | 0 | 248240 | 1 |
| 0x0F | 0 | 71481 | 1 |
| 0x10 | 0 | 193504 | 1 |
| 0x11 | 0 | 73288 | 1 |
| 0x12 | 0 | 141595 | 1 |
| 0x13 | 0 | 116813 | 2 |
| 0x14 | 0 | 2362 | 1 |
| 0x15 | 0 | 272166 | 1 |
| 0x16 | 0 | 58000 | 1 |
| 0x17 | 0 | 186487 | 1 |
| 0x18 | 0 | 1175 | 1 |
| 0x19 | 0 | 61396 | 1 |
| 0x1A | 0 | 92850 | 1 |
| 0x1B | 0 | 114746 | 1 |
| 0x1C | 0 | 296752 | 1 |
| 0x1D | 0 | 1271 | 1 |
| 0x1E | 0 | 3170 | 1 |
| 0x1F | 0 | 66976 | 1 |
| 0x20 | 0 | 142409 | 1 |
| 0x21 | 0 | 35385 | 1 |
| 0x22 | 0 | 15540 | 1 |
| 0x23 | 0 | 53067 | 1 |
| 0x24 | 0 | 67090 | 1 |
| 0x25 | 0 | 5109 | 1 |
| 0x26 | 0 | 621 | 1 |

Evidence under `re/out/sound/`: `music-registers.csv` (39 rows),
`music-strict.txt` (the unfiltered Python comparer), `music-status.txt`,
`music-requests.txt`, `0000.qs.trc` through `0026.qs.trc`, and `port/`.

Area tune previews: `re/out/sound/listening/code_*.wav`, 20 seconds for each of
13 area tunes. Automated checks found no silence or clipping. The ear check is
pending because the implementation worker has no listening capability. The WAVs
are review artifacts, not proof of audio equality.

## M4-11 result, 2026-10-19

One headless MAME 0.289 process recorded 430 cases, 900 frames each (about
15.1 s), with SDL dummy drivers and sound disabled. This includes at least one
listed loop of each looping effect. All 204 live effect headers are included,
including the effects without a known game sender.

| Cases | Count | Register result |
| --- | ---: | --- |
| Each effect, fixed pan | 204 | All zero differences |
| Each effect, mode 7 delayed twins | 204 | All zero differences |
| Pan modes 1 to 6 | 6 | All zero differences |
| Five priority pairs, both directions | 10 | All zero differences |
| Toggle and player bank pairs | 6 | Five zero, one phase difference below |

The fixed and twin passes compare 32,916 and 66,199 DSP writes respectively.
The full batch compares 104,610 DSP writes. The unfiltered Python comparer also
ran on every case: all 430 have hardware-trace differences, none are missing.
Its first difference is the CPU ROM bank write at write 1, frame 0, as in music.
The CSV records each case's register differences and timing counts separately.

**Driver fix, `seq.rs`, Z80 0x00F5..0x0107.** The old implementation reset a
delayed twin's own voice. The original loads A from the delay, decrements the
memory byte with `DEC (IX+3F)`, then calls 0x0981. The memory decrement leaves A
unchanged. When the memory reaches zero, A is still 1, so the original resets
voice 1. The subsequent pan call 0x09E7 uses the normal current voice. The port
now keeps this quirk. SPEC 4 and 9.4 describe a reset of the twin voice, which is
incorrect here. No table extraction or RNG change is needed.

Before the fix, the comparison found 205 differing cases. For effect 0x2C with
mode 7, the first difference was DSP write 27 in frame 3: MAME reset voice 1,
while the port reset voice 11. After the fix all 204 twin cases pass. A synthetic
unit test checks the wrong-voice reset followed by the correct twin pan.

**Documented remaining case: `toggle_00c2_00c3`.** With the common startup phase,
the first difference is write 23, MAME frame 2 versus port frame 3: the first
sound writes its voice volume before the second request in MAME, whereas the
port has already started resetting the second voice. Comparing the shifted
streams counts 324 register differences. The recorder does not save IRQ phase
at injection. A diagnostic replay with IRQ phase 3 and frame phase 0.9 matches
all 354 register writes in order and value, with 15 frame-label differences
of at most one frame. This supports a request/IRQ alignment cause. It does not
measure the actual MAME seed. The primary result remains 429 equal cases plus
one documented phase case; no driver change or per-case fitting was used in it.

Priority pairs (each in both orders): 0x2C/0x30, 0x31/0x30, 0x38/0x30,
0x3E/0xA1, 0x59/0x34. Their overlapping voice sets and priorities are selected
from extracted headers at run time. Both steals and lower-priority rejections
match MAME. Toggle pairs 0x65/0xE4, 0x7B/0xFC, 0x38/0xA8, 0x6B/0xF1 and
0x4B/0xC2 pass; the sixth pair is the documented 0xC2/0xC3 case.

Evidence under `re/out/sound/`: `effects-registers.csv` (all 430 cases, including
the list of 204 codes twice), `effects-before.csv`, `effects-strict.txt`,
`effects-status.txt`, `effect-requests.txt`, the matching `.qs.trc` files and
`port/`. `phase/confirmed.csv` records the diagnostic replay. To reproduce it
from `dino-rs/` without changing the common-phase acceptance result:

```sh
DINO_SOUND_IRQ_PHASE=3 DINO_SOUND_FRAME_PHASE=0.9 cargo run -p dino-sound --example compare -- ../re/out/sound/phase/request.txt ../re/out/sound/phase 900
```

## Gates and follow-up

- G1: `cargo build`, `DINO_SOUND_REQUIRE_BATCH=1 cargo test`, and the comparer
  unit test pass. The sweep tests confirm 39/39 music and 408/408 effect fixtures.
- G2: `re/harness/regress.sh` remains 51 of 52 at zero, `mp_join` is 3,
  matching the supplied baseline.
- G3: music 39/39 register-equal for 15,000 frames; effects 429/430 register-equal
  for 900 frames, one documented clock-phase case. Full hardware traces still
  differ for the causes above.
- G4: one headless play-script run, screenshots 500, 850, 1650 and 2050 under
  `re/out/shots/m410/`, all viewed. They show the title, hero select, rooftop intro
  and Jack fighting on the roof. Autoplay reaches ending mode 0x1C at frame
  182770 within the explicit 320,000-frame run.
- G5: only `crates/dino-sound/` source and documentation changed. Original
  addresses name the corrected routine and its test. No ROM bytes, tables or
  reference write sequences were added. No commit or push.

Follow-up: record IRQ counter and fractional tick phase at injection to close
the remaining timed-request comparison, have a person listen to the area WAVs,
and carry the delayed-reset correction into the sound SPEC and D5 errata in the
main session. Those documents are outside this task's file ownership.
