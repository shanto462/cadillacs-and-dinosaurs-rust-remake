# M10-21 settings variants

2026-10-21, `wt/codex-m10-21`, base `a84305f`, including the earlier `aadb952` work.

Status: implementation and verification complete; commit blocked by the session filesystem sandbox. The approved countdown and select prompt patches are applied and verified. Final gates pass; no push or merge was made.

## Changes

The earlier WIP restores the select picture records' +$A0 and +$A4 fields (0x7B7DE..0x7B7F2) and preserves the masked free-play byte (0x9CFD2..0x9CFDA) in the snapshot loader. The merged harness has `--keep-dip`, which keeps the port's own configuration when loading MAME snapshots. This check uses it throughout. No change to Config::cook or rank.rs was needed.

Added recording-specific timing and loader differences to `allow/m10_runs.txt`. The settings, credit, lives and rank fields have no allow rules. The two-player countdown and select prompt fixes pass without allow rules. Auditing the chute-2 power rules found none redundant: each still covers an observed CPU timing difference. No rule was removed or added for either fix.

All generated inputs, configurations, MAME traces, runtime EEPROM seed, port traces and detailed results are under this worktree's ignored `re/out/m21/`. No ROM bytes or table contents were added to tracked files. The three prompt-origin table registrations in `dino-extract/src/frontend.rs` read those tables from the supplied ROM at runtime.

Applied `countdown-fix.patch` and `select-prompt-fix.patch` unchanged. `flow/join.rs` now prints both countdown digits at the restored a1, matching 0x7E02 / 0x7E1A and 0xCD2. `flow/select.rs` reads the three-player prompt origins at 0x824E and the two-player generic origins at 0x829A; two-player insert/blank strings retain the separate 0x80FC origin. Original call addresses remain in the comments. No change to config.rs or rank.rs was needed.

## Gates

- G1: `cargo build` passed. `cargo test` passed: 710 tests passed, 13 ignored, none failed. The autoplay test ran with the ROM present. The explicit `dino-game --headless --autoplay --frames 600000` also reached mode 0x1c at frame 343802 and returned to attract at 349618. With an explicit frame limit the bot continues into another game after that.
- G2: `PORT_OUT=$PWD/re/out/m21/regress2 re/harness/regress.sh`: 52 of 52 scenarios at zero ported-class field pairs. Baseline: 52 of 52 at zero.
- G3: 107 of 107 recording/mode comparisons pass after the listed timing and loader rules. Regenerated the chute-mode 2 power select replay and both gameplay recordings (`m21e_chute2`, 5000 records; `m21c_chute2`, 5099 records), each in step and lagsync. All four gameplay comparisons have zero fields and zero video/sound items differing even without allow rules; the power replay has zero scroll1 differences. Rechecked all 107 comparisons with the final allow list; the other traces are retained from the earlier checks. Remaining timing/loader differences have named causes and exact recording ranges in `allow/m10_runs.txt`.
- G4 (earlier verification, retained): one headless renderer run with `re/harness/scripts/flow_play.txt`, screenshots 680, 760, 1000, 1800 and 1950 in `re/out/shots/m10-21/`. Viewed all five. The title has one P1 credit and PUSH START, the four portraits and Jack's selection panel are clear, and Jack reaches the rooftop and the first fight. No game window was opened.
- G5: original addresses are retained in the WIP loader comments and the applied countdown correction. No RNG or integer arithmetic changes.

## Settings checks

21 configurations: factory; difficulty 0 and 7; timer level 0 and 3; bonus life 1 and 3; start lives 0 and 3; continue off; coin modes 1 through 8; chute modes 1 and 2; free play. Factory covers coin mode 0 and chute mode 0.

### Longer play

`m21b_factory`, `m21b_diff0`, `m21b_diff7`, `m21b_timer0`, `m21b_timer3`: 12000 gameplay records each, step and lagsync. All modeled fields, video items and sound codes agree, with no settings allow rules. God mode keeps the player alive while normal rank and enemy logic run.

Difficulty 0 starts at rank 6 and reaches 24 at record 8013. Difficulty 7 starts at 18 and reaches 24 at 2862. The factory starts at 12 and reaches 24 at 5472. Timer level 0 starts at speed 1, with the first rank rise at 497 and rank 24 at 6107. Timer level 3 starts at speed 4, with the first rise at 377 and rank 24 at 4834. These are observed run results, not copied ROM table entries.

### Before the game and the first events

The existing cooked-byte recordings cover power on through coin acceptance, credit conversion, start, select and the first level. Settings were poked at power frame 70, before system initialization. `flow_m21b_*` covers difficulty, timer, bonus and four lives; `flow_m21e_*` covers factory, one life, continue off, all coin modes, both common chutes and free play. Port replay uses its config file, not settings pokes. Compare records 70 through 1798. Every settings, rank-start, life and credit field agrees. Partial video writes and CPU timing cuts differ as listed under Q21. The coin-ratio cursor difference is U21. The two-player select prompt origin correction is described below.

The matching gameplay recordings use 5000 records, except `m21e_cont_off` with 2250. `m21f_coin7` replaces the earlier coin-7 setup whose continue input had too few coins. Coin and credit accounting, two-credit start versus one-credit continue, the common pool, free start, initial spare lives, death and continue initialization agree in step and lagsync. Bonus option 1 grants lives at 300000 and 700000; option 3 grants its first life at 1000000. Start-lives 0 has no spare life; start-lives 3 has three spares and restores three on continue. Continue off skips the playable continue countdown.

### Original EEPROM cooking and additional event check

A new single headless MAME batch ran all 21 configurations. It loaded a runtime factory EEPROM seed into MAME's EEPROM device buffer and changed the named option byte before the boot reader ran. MAME read that device using its normal serial EEPROM path and executed 0xAC794 / 0xAC7BC. This was device-buffer seeding, not a persistent nvram-file test. No cooked settings were poked in this batch.

`normal/traces/flow_m21n_*.mame.trc` records the full power-on setup and play. At record 100 all 21 original cooked results equal the corresponding config's result in the port. The already valid EEPROM cooks at record 44; the port's boot model cooks at 68 because it models the fresh factory-write boot delay. The cooking equality does not claim equal boot timing. A factory-config negative control against difficulty 7 differs in G.$777B, so configuration errors are detected.

The original batch takes an attract snapshot at power frame 400, then inserts coins, starts and selects Jack. It records 6500 game-clock records per configuration, except continue-off with 3500. Timer writes force life-loss events, score writes cross bonus thresholds, and all such writes are replayed by the port using the generated sidecars. These are controlled event checks, not natural whole-episode recordings.

For an independent gameplay comparison, `control/m21c_*` is a lossless suffix of each MAME recording: its sync record is original record 1400, the first controllable frame; inputs and poke frame numbers are shifted by 1401. There is no new emulator run or change to recorded state. The suffix has 5099 records, or 2099 for continue-off. Compare step and lagsync with the same configs and `--keep-dip`.

This also checks beyond the first bonus: option 1 grants two lives, at 300000 and 700000, and grants none at 1000000 or 2000000. Option 3 grants one at 1000000 and none at 2000000. Both suffixes are exact in both modes. The one-life suffix includes expiry, game over and a new game; its extra tail has the existing X, A2 and N loader limits listed below.

An attract-snapshot replay was also tried directly (`m21n_*`). It exposes title task and sprite-divider restoration limits before the first lag resync. It is diagnostic, not the acceptance comparison. Use the full power-on comparison for coins/start/select, and the independent control suffix for later events. Its port traces are retained for diagnosis; do not treat them as exact attract snapshots.

## Differences left and causes

- Two-player countdown, fixed: `m21e_chute2` previously first differed at 2141 in both modes; `m21c_chute2` at 2840. At 0x7E02 and 0x7E1A both calls print at the same a1 because 0xCD2 saves and restores it. The port now overwrites the first digit at `at`, matching the original. Both recordings now agree in step and lagsync without allow rules.
- Two-player select prompt, fixed: `flow_m21e_chute2` previously differed in scroll1 at records 856 through 1123. Generic prompt calls at 0x8274 now read table 0x829A; insert/blank calls at 0x8072, 0x80C4 and 0x8126 read 0x80FC. The three-player origin at 0x824E also comes through the extractor. The regenerated power replay has no scroll1 differences anywhere in records 70 through 1798, without a scroll1 allow rule.
- Q21, power-on video timing: the original can stop part way through title blocks, palette writes, select setup or level initialization at a vertical blank. The port models these actions as frame waits. Each differing field and record range is listed by recording in `m10_runs.txt`. First residual: record 680 for most variants; 319 for common-chute and free-play title drawing. These are the same CPU-time boundary limit accepted for the front end, not setting-value differences.
- U21, stale address register: 0x81E6 calls 0x81FE, which can return early while HUD prompts are hidden by G.$4D5. It then still writes the ratio digit at a1+4, using a cursor left by earlier code. The port suppresses the hidden prompt write. One scroll1 cell differs at record 4 of `m21e_coin6` and `m21f_coin7` in step mode and records 4 through 21 in lagsync; the unsynchronized power traces keep that cell until another HUD redraw. Exact ranges are listed. No credit or coin field differs.
- X, A2 and N: the EEPROM one-life run goes past continue expiry into a new game. In step mode the game-over sprite wipe differs at 2649 through 2652; continue-off has the same effect at 1953 through 1956. Step mode loses the new-game task waits from 2800 onward. Lagsync keeps all gameplay fields equal but misses the initial new-level tile strips from 3155 (scroll2) and 3156 (scroll3) to the end. These are snapshot/task restoration limits already documented by M10-19/M10-20. No life, continue or bonus discrepancy precedes them.
- Sound arguments left in unused CPU registers remain different in some requests. The codes agree. They have the existing R1, R2 and R4 causes in `m10_runs.txt`; the sound exactness task covers their interpretation.

## Reproduction and evidence

From this worktree, with the existing ignored recordings:

```sh
DINO_GAME_CLOCK=1 re/.venv/bin/python -I re/out/m21/run_checks.py factory diff0 diff7 timer0 timer3 bonus1 bonus3 lives3
DINO_GAME_CLOCK=1 M21_PREFIX=m21e_ re/.venv/bin/python -I re/out/m21/run_checks.py factory lives0 cont_off coin1 coin2 coin3 coin4 coin5 coin6 coin8 chute1 chute2 free
DINO_GAME_CLOCK=1 M21_PREFIX=m21f_ re/.venv/bin/python -I re/out/m21/run_checks.py coin7
DINO_GAME_CLOCK=1 re/.venv/bin/python -I re/out/m21/run_control.py bonus1 bonus3 factory lives0 lives3 cont_off coin1 coin2 coin3 coin4 coin5 coin6 coin7 coin8 chute1 chute2 free diff0 diff7 timer0 timer3
re/.venv/bin/python -I re/out/m21/final_diff.py
```

`final_diff.py` cuts the recording/mode rules from `m10_runs.txt` into plain allow files. Power checks also use `allow/m1.txt`. Detailed outputs: `final-results.txt`, `final-results.json`, `current-cook.txt`, `current-normal-cook.txt`, `current-event-stats.txt`, `current-control-stats.txt`, `current-tests.log`, `current-regress.log`, `current-autoplay.log`. The new MAME setup is `normal_batch.txt` and `normal_probe.lua`; its 21 completed segments are in `normal/segments.txt`. Its Lua reads the runtime EEPROM seed, not a table embedded in source.

## Final verification and follow-up

Final replay commands:

```sh
DINO_GAME_CLOCK=1 M21_PREFIX=m21e_ re/.venv/bin/python -I re/out/m21/run_checks.py chute2
DINO_GAME_CLOCK=1 re/.venv/bin/python -I re/out/m21/run_control.py chute2
M21_PREFIX=m21e_ re/.venv/bin/python -I re/out/m21/check_power.py chute2
re/.venv/bin/python -I re/out/m21/final_diff.py
PORT_OUT=$PWD/re/out/m21/regress2 re/harness/regress.sh
```

Final logs: `final2-build.log`, `final2-tests.log`, `final2-release.log`, `final2-autoplay.log`, `final2-regress.log`, `final2-chute2-checks.log`, `final2-results.txt`, `final2-chute2-power.details`, and `final2-allow-audit.txt`, all under `re/out/m21/`. `final-results.json` contains the updated 107 comparison outcomes. Build, test and autoplay ran from `dino-rs/`; autoplay used `./target/debug/dino-game --rom ../dino.zip --headless --autoplay --frames 600000`. All final Python invocations, including the power-input helper, used `-I`. The existing `re/.venv` link worked. Final verification replayed the existing MAME recordings; no emulator or game window was opened. No further implementation or verification work is required for this block. The general attract-snapshot loader limitation is separate from settings semantics. A persistent nvram-file boot test remains optional; the original EEPROM read and cooker have already been exercised through the emulated device.

Commit attempt: `git add` could not create the Git worktree metadata file `.git/worktrees/codex/index.lock` in the parent repository (Operation not permitted). That metadata directory is outside the permitted writable roots, and this session disallows permission escalation. Nothing was staged or committed. The intended commit message is `Fix two-player settings prompts and finish M10-21 checks`. Commit remains the only required follow-up.
