# Coverage of the original program by the harness recordings

2026-10-22, worktree `wt/opus2-coverage` (base `c409444`). Which parts of the original 68000 program does no MAME recording of the
harness run? Those are the places where the port is not tested against MAME. Nothing in this file is ROM data: only addresses,
counts and descriptions. The outputs with address lists are in `re/out/coverage/` (git-ignored).

## 1. Summary

- **Method**: MAME debugger breakpoints that switch themselves off after the first hit (no window, `-debugger none`), on
  36,760 watched addresses (every routine start and every basic block leader of the code), while the harness tools replay their
  own recordings unchanged. Every replay was compared record by record with its original recording.
- **Recordings replayed**: 1,599 recordings, each equal to its original for every record: the 31 natural whole runs, 149 flow,
  attract and power-on recordings, 1,176 probe scenarios, 82 settings variants of M10-21, 161 plain `record.sh` scenarios.
  9 more replays differ from their recording (unknown settings, or a hang); their hits are left out.
- **Code that ran**: 77.4 % of the instructions below 0xB0000 (70,745 of 91,454) are in basic blocks that ran in at least one
  recording; 25,573 of 34,613 blocks. Of the 2,383 routines (section 3) 1,816 ran and 567 never ran.
- **Routines of entries.txt**: of the 3,803 entries, 1,870 are data, not code. Of the 1,847 code entries below 0xB0000,
  1,463 ran and 384 never ran. Ghidra's list has none of the 249 object class handlers.
- **Never-run code**: 20,709 instructions. 6,800 of them are rare branches inside routines that ran. The test menu (4,213), the
  debug mode (729), the region texts, the anti-tamper failure paths and the error handlers are code that normal play does not
  reach. What normal play can reach: the **2 player cabinet layout** (chute mode 2, 666 instructions: no recording played a level in
  it before this task), the grab and leap acts of the **Butcher**, the run and retreat of **Vice T.**, Shivat's neutral pacing, the tag team
  partner's run attack, and about 900 small branches in enemy, player, camera and flow code (section 5).
- **New recordings** (section 6): `cov_2p_play` (2 player layout in a level, step equal for 1,800 records), `cov_2p_join`
  (P2 joins in the 2 player layout: **the port differs**, first at record 161, the pan of sound 0x3E, and at record 438, a text
  cell at 0x90A618), `cov_2p_select`, and three boss scenes without god mode (equal in step mode, but they did not reach the
  target states).

## 2. Method

**Breakpoints that switch themselves off** (`cov.lua`). MAME 0.289 runs with `-debug -debugger none -video none -sound none`,
SDL dummy drivers: the debugger core is on, but there is no window of any kind. `cov.lua` sets one breakpoint per watched
address with the action `bpdisable #N; g`. The first time the 68000 reaches the address the breakpoint turns itself off and
the CPU goes on, so each address costs one stop. The breakpoint look up is per instruction but logarithmic, so the replays run
at 10 to 20 times real time with the trace writing (a 70,000 frame episode in under two minutes). The debugger only watches:
the replays are bit for bit the same recordings (below). The hits are written every 3,000 frames and at the end, one hex
address per line.

Two traps (handled in `cov.lua`): the MAME debugger reads a bare number as hex, and the hex text of some breakpoint numbers is a
register name (`a0` .. `a7`, `d0` .. `d7`): `bpdisable d0` turns off the breakpoint whose number is the value of D0. So the
action writes the number in decimal with `#`. And MAME runs an autoboot script in an environment of its own, so a function for
the inner script has to go into `_G`.

**Watched addresses** (`blocks.py`, 36,760, `re/out/coverage/addrs.txt`): a recursive descent over the program (capstone,
68000 mode) from these seeds:
- the 3,803 entries of `re/ghidra/entries.txt`;
- the 249 class handlers of the four object dispatch tables (pool A 0x2346C, B 0x2362E, E 0x233F0, C 0x237E6, as in `diff.py`);
- 281 routines that a doc comment of a port function names and that the descent had not reached;
- code pointer constants of the decoded code that the port also names (state handlers stored in an object, task entries).
The descent follows branches, calls and the usual jump tables (word tables after `jmp T(pc,dN.w)`, branch tables, long tables
read before `jmp (aK)`). A new seed must pass a trial walk: it must decode (capstone's `dc.w` is data), must not start inside a
decoded instruction, must not use instructions the game never uses (reset, stop, illegal, rtr, trapv, chk, tas, movep), and
must not start with a run of mostly immediate or bit operations (tables of small words look like that). 1,889 of the 1,892
entries from 0xB0000 on fail the trial, and 503 entries do not decode at all, so 1,870 entries of entries.txt are data. The
data entries keep their breakpoint (none of them ran).

**Replaying the recordings unchanged** (`run.sh`, `bin/mame`). The shim `bin/mame` is first in PATH; it adds the debugger options
and puts `cov.lua` in front of the tool's own autoboot script. So the harness tools themselves run: `record.sh` (natural whole
runs, power-on references), `record.sh --batch` (flow batches, attract loop, demos), `probe_record.sh` (probe scenarios). They
run in a private root `re/out/coverage/root` (links to `re/harness`, `dino.zip`, the MAME ROM folder), so they never write to
`re/out/traces`. The batch files of one job are joined (`merge.py`) and replayed in one MAME process. The 169 plain `record.sh`
scenarios (phase 1 classes and items, the M2-30 two and three player scenarios, M9 tag team checks, ...) were recorded one MAME
process each; `multi.lua` runs about 60 of them in one process: the first from power on, each later one from the state saved at
the end of the first video frame (as `probe.lua` does), each with a fresh copy of `record.lua` and its own settings, and with
its own hit file (all breakpoints are turned on again between recordings). `scenarios.py` finds the settings of each recording
from its header and snapshot (the boot script from the heroes in the snapshot, the sync from its vblank, the poke files by name)
and writes `jobs/scenarios.list`.

**Verification** (`same.py`, run by `run.sh` after every job): every recording a job makes is compared with the recording of the
same name in `re/out/traces`, record by record after the JSON header (`same`, or `prefix` when the original is a cut). A replay
that differs does not count: in a `multi.lua` job its hit file is moved to `differ/`.

**Mapping** (`map.py`): routines are the code entries of entries.txt, the class handlers, the port routines and the other call
targets; a routine ends where the next one starts. A **gate** is a block that never ran but that a block that ran can enter
statically (a branch never taken, a call in such a branch, a table entry never used). The **region** of a gate is the never-run
code it reaches without passing a block that ran; each never-run block is counted once, in the largest region. Gates reached
only through a pointer constant, or only through a pc-relative table entry far from its table (often a table of pairs read as
offsets), are marked doubtful and left out of the lists. The port side is a grep of every hex number and identifier suffix
(`pass_10cd0`) in `dino-rs/crates/*/src`.

**Limits**. Code that only a pointer the descent does not follow can reach is not watched. The static gates and regions can
include dead table entries. The scenes in section 5 come from the stage scripts (`re/specs/enemy/stage_scripts.txt`, `where.py`),
the design documents and the port comments, not from new probing, except where section 6 says so.

**Run it again** (about three hours, one MAME process at a time):

```sh
re/harness/coverage/all.sh                       # every job file, then map.py; summary lines on stdout
re/harness/coverage/run.sh re/harness/coverage/jobs/probes.txt p_m700     # one job
re/.venv/bin/python -I re/harness/coverage/map.py [BATCH ...]            # the numbers, re/out/coverage/*.tsv
re/.venv/bin/python -I re/harness/coverage/where.py A61 B19               # where the stage scripts spawn a class
```

The job files are `jobs/natural_*.txt` (`record.sh`, one process per whole run), `jobs/flows.txt`, `jobs/probes.txt`,
`jobs/scenarios.txt` (`multi.sh` with `scenarios.list`). The settings variants of M10-21 (`s1021*`, `m21_*`) have no tracked
batch files; they were replayed from the haiku worktree's `re/out/m21/*_batch.txt` (`re/out/coverage/jobs/extra.txt`).

## 3. Numbers

### Recordings replayed (2026-10-21 and 22)

| Job files | Recordings | Equal to the original | Different | MAME time |
| --- | ---: | ---: | ---: | ---: |
| `natural_ep1.txt` .. `natural_ep8.txt` (31 natural whole runs, 7 MAME processes for Episode 1, 24 for Episodes 2 to 8) | 31 | 31 | 0 | 38 min |
| `flows.txt` (3 flow batches in 3 processes, attract3, coinstart_full, m328_power) | 149 | 149 | 0 | 12 min |
| `probes.txt` (8 merged probe batches, 1,189 scenario segments) | 1,177 | 1,176 | 1 (hangs) | 88 min (40 of them the hang) |
| settings variants of M10-21 (one process) | 82 | 82 | 0 | 4 min |
| `scenarios.txt` (169 plain `record.sh` scenarios, 3 processes) | 169 | 161 | 8 | 27 min |

- The M5-17 Episode 1 runs were first recorded 40,000 frames long with older scripts; the current scripts give the same records
  up to the cut that the harness uses (`m517_ep1_*.mame.trc`), so the replays stop there.
- The probe segment `m800_a14_n_e7a1_full` (Episode 7 area 1) does not end in the replay: at the end of its trace the game stops
  finishing frames. It was stopped; the 74 segments before it and the 71 after it equal their recordings. `m519_ty_thug_b`
  fails its snapshot (it has no recording either). 11 probe segments have no recording of their name (`m517_boot_*`, `m601_far_*`).
- The 8 scenario replays that differ already differ at the snapshot (record -1): `flow_clock_power`, `flow_story_power`,
  `m109_free`, `m901_tierun_hannah_jump`, `_left`, `_stop`, `sk_e6`, `sk_e7` (their poke files are not known). Left out
  before the run: `m110_free`, `m223_free_wait`, `sk_e1`, `ep8_row` and the `a416_` / `a419_` copies of the natural runs.
- Of the 2,013 MAME recordings in `re/out/traces`, 415 were not replayed under their own name: 173 are `a416_` / `a419_` copies of
  recordings that were replayed, 71 are `m224_` / `m228_` cuts of flow segments that were replayed, the rest have no tracked
  recipe (`re/out/coverage/not_replayed.txt`).

### Routines and blocks

| What | Count | Ran | Never ran |
| --- | ---: | ---: | ---: |
| entries.txt | 3,803 | 1,463 | 2,340 |
| of which data, not code | 1,870 | 0 | 1,870 |
| of which code below 0xB0000 | 1,847 | 1,463 | 384 |
| class handlers (not in entries.txt): pool A / E / B / C | 100 / 12 / 97 / 41 | 89 / 12 / 84 / 35 | 30 |
| routines the port names that are not entries or handlers | 278 | 126 | 152 |
| other call targets (bsr / jsr targets that are none of the above) | 9 | 8 | 1 |
| **all routines below 0xB0000** | **2,383** | **1,816** | **567** |
| basic blocks below 0xB0000 | 34,613 | 25,573 (73.9 %) | 9,040 |
| instructions in those blocks | 91,454 | 70,745 (77.4 %) | 20,709 |

- 514 routines that ran have blocks that never ran: 4,060 blocks, 6,800 instructions (branches never taken).
- The port names 211 of the 567 never-run routines (it has them, untested); it names 1,718 of the 1,816 that ran.
- What each kind of recording adds (instructions in blocks that ran): the 31 natural whole runs alone 73.2 %; with the flow,
  attract and power-on recordings 76.0 %; with the probe scenarios 77.1 %; with the settings variants and the plain scenarios
  77.4 %; with the new recordings of section 6 77.8 %.

## 4. The never-run code by group

The groups follow the gate in front of the code (section 2): what the branch into it tests, or what reaches it. The instructions
are of never-run blocks below 0xB0000 (each block once).

| Group | Gates | Never-run instructions | Port has it | Can normal play reach it? |
| --- | ---: | ---: | --- | --- |
| Data in entries.txt | (1,870 entries) | - | - | no: never executed |
| Error handlers (vectors 2 to 11 and 24 to 31: bus and address error, illegal instruction, zero divide, CHK, TRAPV, privilege, trace, line A / F, spurious and unused interrupts) | - | 17 (9 routines) | 3 named | no: only on a CPU fault (seen when an impossible hold was forced, D5 errata) |
| Test mode, service menu (the TEST switch in the system task 0x9D5B6 and the test task 0xA6CE8: input and output, sound test, colour, cross hatch, configuration, object and scroll tests, memory check) | 2 | 4,213 (142 routines) | 1 (not ported by design, D4d section 10) | no: TEST switch or DIP C bit 7 |
| Debug mode (G.$4AA: the debug object B69, debug branches in the sprite list builder 0x13FA6 / 0x149B0, the player update 0x18952, the vblank IRQ 0x542, the sleep call 0x974 and the skip-wait task 0x9E38C, ...) | 27 | 729 | 5 of 7 routines named; the port has its own debug keys instead | no: P1 up, right, attack, jump and coin at power on |
| Region texts and rules (G.$32 Japan or USA: the boot text 0x9DCF6, the select ratings 0xB462 / 0xB68C, the join 0x75B2, B26) | 10 | 64 | partly | no: the World set |
| Anti-tamper failure paths (0x2A5B6, 0x29ADE, 0x104F6) | 3 | 6 | no | no: only with a modified ROM |
| Sound, 68000 side (request and flush helpers 0x9500 .. 0x9FFF) | 33 | 49 (and 6 never-run routines) | 3 of the 6 named | small branches, some reachable |
| 2 player cabinet layout (G.$4CF = 1: chute mode 2) | 43 | 666 | yes (`hud.rs`, `flow/join.rs`, `prop/class19.rs`, `car.rs`) | **yes**, with the operator setting (section 6) |
| Object classes no recording spawns | 22 | 849 | pool C dead classes named; B19 yes; the others no | see below |
| Other game code (rare branches, states, table entries) | 895 | 5,785 | yes for almost all (section 5) | **yes**, mostly |
| No static way in from code that ran (state handlers behind tables the descent does not read, unused code, and code only reached through unresolved jumps) | - | 8,348 | 59 of the 181 never-run routines without a caller are named | partly |

Object classes that no recording spawns (handler address, instructions behind it):
- **Pool A** (no spawn in the stage scripts, the port has none of them): A9 0x3B7D6, A21 0x479FE, A28 0x4DAB4, A33 0x51158,
  A37 0x53178, A39 0x53CC6, A44 0x56190, A47 0x567D0, A73 0x5E4C8, A99 0x6A408 (83), A101 0x6C01A (37). Most are 15 to 20
  instruction stubs; probably unused.
- **Pool B**: B16 0x7B520 (stage exit walker, no spawner found), B19 0x7B7C6 (the select pictures of the 2 player layout: ported,
  run now by `cov_2p_select`), B23 0x7C0E4 (palette flash of the title, story and movie), B24 0x7E2D0 (movie text crawl), B51
  0x85088 (wall break), B58 0x85E32 (map picture panel), B65 0x86C14, B75 0x89DA6, B77 / B78 0x8A090 / 0x8A156 (scenes, a scroll1
  page writer, 174), B90 0x8CAE0 (caption of the car section), B91 0x915FA (story panel text, 113), B92 0x91CF2 (portrait box).
  The front end ones may belong to other regions or to unused scenes (their makers are in code the descent does not reach).
- **Pool C** (the port names them as dead classes, M9-05 and D5 errata): C0 0x923E0, C3 0x929B8 (the shockwave of a class 10 thug
  holding a hero; the class 10 hold never starts), C9 0x93184 (STAGE CLEARED!), C12 0x939D6 (only with the debug switch), C13
  0x93A7A, C35 0x96E56.

Never-run code that the port already calls dead or unreachable (left out of section 5): Lash T. 0x362EC / 0x36394 / 0x35F5A (no
caller), the tail of the pool C 24 maker 0x95AD6, the ending wait loop 0xA6B10 (debug build), R Hopper's variant 4 roar 0x4E0E0,
Episode 3 camera modes that its controller has no entry for (0x22864, port `debug.rs`).

## 5. Game code that normal play can reach and no recording covers: top 20

By never-run instructions behind the gates of one feature (doubtful gates left out). The scene says what a new recording needs.
All addresses are original program addresses; "port" says whether the port has the code.

| # | What (gates) | Never-run instructions | Port | Scene that would reach it |
| ---: | --- | ---: | --- | --- |
| 1 | **2 player cabinet layout** G.$4CF = 1: the HUD of the present players 0x16C38 (318), the HUD block addresses 0x16336 (86), the join strip face, name and countdown 0x780C / 0x7DDA, the low vitality blink 0x16742, the flow prompts 0x8000 .. 0x825A, the select pictures B19 0x7B7C6 (43 gates) | 666 | yes | chute mode 2 (G.$777A bits 3-4 = 2, set after the boot cook, as the M10-21 settings checks do), two players, any episode; P2 joining in the level; the select screen. **Recorded now** (section 6): 27 gates, 130 instructions left |
| 2 | **Butcher** (A61): act 0x22 the short approach for the leap 0x58FFC, act 0x28 holding the hero 0x5913C, act 0x2C the leap that carries the hero 0x5937E, act 0x26 after the heavy slash 0x590E8, the aggression toggle timers 0x5893A, entries 0x5954C / 0x590DE / 0x5912E of the state table 0x588E0 (19 gates) | 385 | yes (`class61*.rs` names all 19 routines) | Episode 2 area 2 (`jump 2 2 0x1A0`), a hero he can grab (no god mode: all 9 Butcher probes of M6-10 and 5 of the 6 of M6-00 have god mode on), standing in reach after his heavy slash (act 0x26 starts the grab reach 0x59E3C) |
| 3 | **Vice T.** (A18): move mode 2, the run to a cell 0x42CB4 (picked from the roll table 0x44ADE when both directions agree), the retreat 0x42EFC, the fall state 0x43742 (a ground action loses its floor), 0x4480A, the summon count HP 0x42A20 (29 gates) | 283 | yes (27 of 29 routines named) | Episode 1 area 2 boss, and Walther (var 0x200) in Episode 3 area 0 and Episode 4; long fights with the hero at varied places; an edge or pit under him for the fall |
| 4 | **Player** code: the attack in the tag team partner's run 0x1A4CE (68, 0x1A498 region), held by another player 0x1D582 (34, the port says not ported), the stagger reaction 0x22 0x1D032 (32), others (74 gates) | 344 | yes, except 0x1D582 | two players, a tie (both still 90 frames), the partner who runs (Jack) presses attack during the run; a hero held by another hero |
| 5 | **Collision and draw core**: the multi box test 0x12B0E (72, pass 0x11E56 sub tables), an alternative composition (attribute bit 15) drawn facing left 0x14944 / 0x14916, a monster attack on a hero in sub state 6 0x11D92 (36), the partner riding damage scale 0x112A4 (27) (58 gates) | 305 | yes | objects with multi boxes; an object using an alternative composition while facing left; a monster hitting a hero who is getting up |
| 6 | **Camera and stage**: 0x226E4 (42, the camera Y follow branch of `+$4B`), the 2x2 block copy 0x21474 (24), the Episode 3 copies of the event load 0x22772 / 0x229AE / 0x22956 (33 gates) | 276 | yes for most | Episode 3 camera events of modes 1 and 6 (the port says Episode 3 has no entry for some of them: check before recording) |
| 7 | **Class 10 thugs** Hammer T., Blk Elmer, Wrench T. (A10): the grab 0x3C726 / 0x3C766, the paired attack that waits for a held hero 0x3BEB6, the close test 0x3D094 (10 gates) | 258 | yes (`class10_hold.rs`) | probably **not reachable**: the grab window (0x3C74A) carries a bias of 0x400 and never opens (harness README, task 22); the paired attack needs a hero held by another enemy next to two class 10 thugs |
| 8 | **Flow** code 0x1000 .. 0xFFFF: small branches of the story panels (0xD93C), the select, the join and the stage start (83 gates) | 220 | yes | the flow batches with other parties and timings |
| 9 | **Shivat** (A24): MATOWARI, the neutral pacing 0x485EA (92), the wake with no hero near 0x485B2, the tail swing 0x48B08 and the turn 0x48B34, the long reach test 0x4935C (16 gates) | 175 | yes (`class24*.rs`) | Episode 2 area 0 (X 0x678) or Episode 5 area 0 (X 0x450): wake him with no hero in front, then wait; `cov_shivat_far` (hero far away, 4,000 frames) did not reach it |
| 10 | **Pool B props**: small branches of the tile writers and event objects (0x7ED46 .. 0x7EE02 of B26, 0x92048, 0x85028, ...) (69 gates) | 163 | yes for most | the areas of each class (D3e-pool-b-inventory.md 2.2) with the other variant or a second hit |
| 11 | **Tyrog 2** (A91, the copy made by the Episode 8 lab tank): 0x672D0, 0x669FA, 0x674E6 (12 gates) | 164 | yes, as class 4 code (`class91.rs`): the copy's own addresses are not named | Episode 8, the lab tank: a long fight with Tyrog 2 |
| 12 | **Tyrog** (A4, Episode 6 boss): the hops of states 5 and 6 0x388F8 / 0x38AA8, 0x37FDE, 0x38B18, the claw 0x381C6 (12 gates) | 154 | yes (`class4*.rs`) | Episode 6 boss fight, long, the hero at varied distances |
| 13 | **Objects, items, enemy library** 0x23000 .. 0x32FFF: small branches (item classes 0x29056 .. 0x29748, library 0x3227E) (48 gates) | 115 | yes | weapons and items in other states (thrown at the screen edge, picked in the air, ...) |
| 14 | **Fessenden**, last form (A94, Episode 8 area 3): the phase timer 0x68014, 0x6925C, 0x692E8 (11 gates) | 102 | yes (`class94.rs`) | Episode 8 final fight, long |
| 15 | **Punk and Thug** (A20): the action pick by variant row 0x45EB4, the jump at the target 0x462F6 (14 gates) | 98 | yes | Episodes 1 to 3 waves; the variants that the natural runs do not meet |
| 16 | **Poachers** Poacher J, Skinner, Gutter (A13): keep guarding by aggression and variant 0x3E068, 0x3E0C0, 0x3E026 (15 gates) | 91 | yes | Episode 2 waves, aggression set high (`aggr 7`), a hero who attacks a guarding poacher |
| 17 | **Morgue** (A40, Episode 5 boss): the phase countdown and flip 0x5411E, 0x55550 (12 gates) | 79 | yes (`class40.rs`) | Episode 5 area 1 boss (`jump 5 1 0x1120`), a long fight |
| 18 | **Fessenden**, forms 1 and 2 (A84, Episode 8): 28 small gates (0x61C52, 0x6252C, ...) | 72 | yes (`class84.rs`) | Episode 8 boss fights with two and three players |
| 19 | **Mack** (A38): entries of the sub state table 0x53528 (0x536AE, 0x53792) (12 gates) | 69 | yes (`class38.rs`) | Episode 2 area 0 (X 0x320, 0x420) and Episode 5 area 0 |
| 20 | **Blade and Razor** (A14): 15 gates (0x4096E, 0x40DD6 ...) | 64 | yes (`class14*.rs`) | Episode 1 areas 1 and 2, Episode 3 area 0, with the entrance variants the phase 1 scenarios did not use |

The full list is `re/out/coverage/block_gates.tsv` (gate, doubtful flag, group, routine, never-run blocks and instructions, the
instruction that leads there).

## 6. New recordings (this task)

`re/harness/pokes/cov_gaps.txt` (tracked), one MAME process, recorded with the coverage on (`probe_record.sh ... all` under the
shim), compared with `MODES=step re/harness/m10_check.sh NAME:FRAMES`. The traces are in `re/out/traces/cov_*` (fresh names).

| Recording | Frames | Scene | Step result | New coverage |
| --- | ---: | --- | --- | --- |
| `cov_2p_play` | 1,800 | chute mode 2 (G.$777A = 0x90 held from power frame 150), P1 Jack and P2 Hannah walk and punch in Episode 1 | **equal** for all 1,800 records | the 2 player HUD 0x16C38, 0x16336 |
| `cov_2p_join` | 1,200 | the same layout, P1 alone, P2 joins in the level (coin at 300, start at 340, attack at 460) | **differs**: from record 161 the sound 0x3E has another d2 (MAME A/0x60, port A/0, 6 requests, the pan of the join sound); at record 438 one scroll1 cell, 0x986 at VRAM 0x90A618: MAME 0x098A018D, port 0x0020018D (a text tile in MAME, blank in the port). Nothing else differs | the 2 player join |
| `cov_2p_select` | 1,500 | the same layout, snapshot at power frame 780 (before the select screen), P2 starts at +40, both pick at +120, into the Episode 1 opening | not a valid check: it differs from record 0 (G.-$7126 MAME 5 port 4, G.$0 MAME 0 port 1, then the select screen): the snapshot is in the game start setup (mode 0), which the step loader does not take over. A power-on flow batch (`record.sh --batch`, like `flow_select_name`) with the setting is the right recording for it | the select pictures B19 0x7B7C6 |
| `cov_butcher_nogod` | 4,500 | Episode 2 boss, no god mode, P1 HP held at 100, hero mostly still, no reinforcements | equal | none of the Butcher gates of section 5 |
| `cov_vice_nogod` | 5,000 | Episode 1 boss, no god mode, HP held, a punch every 45 frames | equal | none of the Vice T. gates |
| `cov_shivat_far` | 4,000 | Episode 2 area 0, god mode, the hero stays far from Shivat | equal | none of the Shivat gates |

Together the six recordings run 147 addresses that no other recording runs. The 2 player join difference is a port task (the
port's `flow/join.rs` in the 2 player layout: the pan of the join sound 0x3E and one prompt cell at 0x90A618); no game code was
changed in this task.

## 7. Files

Tracked (`re/harness/coverage/`): `cov.lua` (the breakpoint recorder), `bin/mame` (the shim), `run.sh` (one job file),
`all.sh` (every job file, then the map), `multi.lua` and `multi.sh` (many `record.sh` recordings in one process), `scenarios.py`
(the list of plain recordings), `merge.py` (joins probe batch files), `same.py` (compares two traces), `blocks.py` (the watched
addresses and the edges), `map.py` (routines, blocks, gates, groups), `where.py` (spawn places of a class), `jobs/*.txt` and
`jobs/scenarios.list`, this report; `re/harness/pokes/cov_gaps.txt` (the new recordings).
Ignored (`re/out/coverage/`): `addrs.txt`, `blocks.txt`, `blocks.edges.txt`, `hits/BATCH/*.hits`, `logs/BATCH/*.log`,
`routines.tsv`, `blocks_hit.tsv`, `block_gates.tsv`, `never.tsv`, `never_blocks.tsv`, `not_replayed.txt`, the private
root `root/`.
