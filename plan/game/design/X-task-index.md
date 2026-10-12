# D5 input: port task index

Input for `design/D5-final-plan.md`. One row per piece of port work. A later doc wins when two docs list the same work. Hours are agent hours.

Hour rule when a doc uses a letter and no number: D2a, D2b, D2c, D4a use S = 0.5, M = 1.5, L = 3, "M to L" = 2.5. D3e uses its own key: S = 0.4, M = 1, L = 2.5. A number printed in the task row wins over the letter. Pool B rows for Episodes 3 to 8 use the part 1 letter. Part 2 did not sign those classes off.

| Status | Meaning |
| --- | --- |
| open | No commit does this work |
| DONE | A commit on `main` does the task. Hours left = 0 |
| PARTIAL | A commit does part of the task. Hours left is an index estimate, marked `est` |

Commits used: `f8e3c93` M0, `76ea52e` IT-P1, `a823109` friendly fire gate (`player_g4f3` returns `spawn.g.alive`), `87bc612` death rank calls `0x184E2` / `0x1842C` and extra life `0x6BDC`, `46ab87b` `boot_2p` / `boot_3p`. `c7c63fb` ported class 14. D3a section 15 calls that class done, so NEW-PLAN T5.1 has no open row.

## Missing or partial source docs

| Doc | State |
| --- | --- |
| D3e part 2 (`D3e2-prestudy-ep3-5.md`) | Desk pre-study of Episodes 3 to 5 only. Trigger lists and a probe checklist. No port task rows. No MAME |
| D3e part 3 | No file. Episodes 6 to 8 have no area walk |
| D3f (`design/D3f-effects.md` in NEW-PLAN) | No file. Pool C and pool D have no class list |
| X-rank-crosscheck | No task ids. Open calls are the rest of T-TR3 (join `0x18410`, continue `0x18566`, title wait `G.$4F6`) |
| `capture-recipes.md`, `asset-catalog.md` | T0 and X0 outputs. No port tasks |

D3e section 2 marks pool B classes 15, 22 and 73 as already ported. They are not rows. Class 22 still has follow-up work in P-D3c-1 and T-C4.

## Same id, different work

D4d renumbered M2. Use the D4d id in this index.

| NEW-PLAN id | What it is | Kept id |
| --- | --- | --- |
| T1.1 to T1.4 | Same names as D4d, smaller hours (M1 was 10h) | D4d T1.1 to T1.4 |
| T1.5 | Slides and the movie in one row | D4d T1.6, T1.7a, T1.7b |
| T1.6 | Attract state machine | D4d T1.8 |
| T2.3 | Episode 1 opening | T-C1 |
| T2.4 | Join | D4d T2.3 |
| T2.5 | Continue and game over | D4d T2.4 |
| T2.6 | Clock, TIME OVER, GO | T-TR1, T-TR2, T-C3 |
| T2.7 | Tally | D4d T2.6 |
| T2.8 | HUD leftovers | P-D2c-5, P-D2a-3, P-D2b-2 |
| T3.3, T3.4 | Story panels and cards | D4d T2.7 (D4d moved them to M2) |
| T3.5 | Car skeleton | T-C5 in M3, P-D3c-5 in M6 |
| T3.7 | Ending | Four unnamed D4d tasks, no hours (see the gap table) |
| T3.8 | Live ranking | D4d T1.4 and T2.5 |
| T4.5 | Cue wiring | S1, S2, S3, S4, S10, S11, plus P-D2a-7 and P-D2c-9 |
| T5.1 | Finish class 14 | No open row. D3a says ported |
| T5.2 | Vice, bullet 59, Walther | P-D3c-2, then HT8 in M6 |
| T5.3 | Episode 1 pool B | B40, B11, B12, B14, B66, B74 |
| T5.4 | Episode 1 effects | P-D2a-8, P-D2c-6, IT-P2. D3f is missing, so the effect list is not closed |
| T5.5 | Rock Hopper of the Vice fight | P4. D3b says yes |
| T5.6 | Whole Episode 1 MAME run | No single owner. Pieces are P-D3c-7, IT-P3, P-D2a-4 |

## M0

| ID | Scope | Hours | Left | Routines or classes | Depends on | Status | Source |
| --- | --- | --- | --- | --- | --- | --- | --- |
| M0.1 | Extract area music tables, every episode starts, `--area` / `--episode`, debug boxes off by default | 1.5 | 0 | `0xA582`, `0xA5C8`, `flow_area_music` | none | DONE `f8e3c93` | NEW-PLAN s6 (no design doc) |

## M1 attract

D4d section 14. Sum of this table: 17.5h. NEW-PLAN had 10h for six coarser rows.

| ID | Scope | Hours | Left | Routines or classes | Depends on | Status | Source |
| --- | --- | --- | --- | --- | --- | --- | --- |
| T1.1 | Front end scene runner, text printers, picture loaders, block printers. Boot into the attract loop | 2 | 2 | `0xBAE`, `0xBB6`, `0xC0A`, `0xC44`, `0xC9A`, `0x84F72`, `0x84736`, `0x204CA`, `0x20592`, `0x98BAE` | none | open | D4d s14 |
| T1.2 | Boot screens, title, INSERT COIN blink, logo pool A class 98 | 2 | 2 | `0x9DCF6`, `0x9DD80`, `0x9A3CE`, `0x98874`, `0x983EE`, `0x6A20A` | T1.1 | open | D4d s14 |
| T1.3 | Four profile cards, pool B 93 and 94, pool A 100 | 2 | 2 | `0x9A7E6`, `0x9A92E` to `0x9AB32`, `0x91FB4`, `0x921DA`, `0x6A5F4` | T1.1 | open | D4d s14 |
| T1.4 | Ranking screen from the default table, scroll, fades | 1.5 | 1.5 | `0x1516C`, `0x15594`, `0x15748`, `0x15888`, `0x1591C` | T1.1 | open | D4d s14 |
| T1.5 | Demo rows, 30 input streams, RNG reset, playback | 3 | 3 | `0x99E88`, `0x99F10`, `0x9A146`, `0x9A1E2`, streams `0xC869A` | T1.1, M0.1 | open | D4d s14 |
| T1.6 | Five story slides | 1.5 | 1.5 | `0x988C0` | T1.1 | open | D4d s14 |
| T1.7a | Opening movie, handlers 10, 200, 500 | 1.5 | 1.5 | `0x9BF50` to `0x9C29C` | T1.1 | open | D4d s14 |
| T1.7b | Opening movie, handlers 600 to 1450, helpers | 2.5 | 2.5 | `0x9C29C` to `0x9C79C`, `0x9C7C4`, `0x9C7F8` | T1.7a | open | D4d s14 |
| T1.8 | Attract state byte, loop order, coin edge, press start, `G.$4F6` counter | 1.5 | 1.5 | `0x97C20`, `0x9D048`, `0x9D4BE`, `0x9D326` | T1.2 | open | D4d s14 |

T1.4 names pool B 86 (`0x8B428`) as the ranking car. D3e calls the same handler the continue-screen object. D5 picks one.

T1.5 can be checked by trace before Episodes 2 to 8 have real art. It looks like the arcade only after M6 to M8.

The sound filter bullet inside T1.1 is the same rule as S1. S1 is the M4 flush. Do the flush in S1.

## M2 coin, select, opening, join, continue, clock, tally

Sum of this table: 47.0h listed, 45.25h left.

| ID | Scope | Hours | Left | Routines or classes | Depends on | Status | Source |
| --- | --- | --- | --- | --- | --- | --- | --- |
| T2.1 | Coin and chute modes, credit text, SERVICE, lockout | 2 | 2 | `0x9D048` to `0x9D8EE` | T1.8 | open | D4d s14 |
| T2.2 | Character select: panels, 10s timer, cursors, stars, auto pick | 3 | 3 | `0xAD46`, `0xAF7C`, `0xB06A`, `0xB122`, `0xB25E`, `0xB444`, `0xB810` | T2.1 | open | D4d s14 |
| T2.3 | Join task, prompts, quick select strip, drop in | 3 | 3 | `0x6C4A`, `0x6D60`, `0x7562`, `0x65A4`, `0x1049E` | T2.2 | open | D4d s14 |
| T2.4 | Continue countdown, villain screen, game over, back to attract | 3 | 3 | `0x732E`, `0xE9E6`, `0xAC06`, `0xAC26`, `0x10490` | T2.1 | open | D4d s14 |
| T2.5 | Name entry, insert and sort, profanity filter | 1.5 | 1.5 | `0x15AE2` to `0x16270`, `0x74A4` | T2.4 | open | D4d s14 |
| T2.6 | Clear tally `0xB97E` and the map panel | 3 | 3 | `0xB97E` to `0xCCFE` | T1.1 | open | D4d s14 |
| T2.7 | Intro cards (states 0 to 2) and the 7 story panels | 3 | 3 | `0xB9BE`, `0xD898`, `0x84736`, `0x7EC20` | T1.1, T2.6 | open | D4d s14 |
| T-C1 | Scripted control: extract 26 scripts, 41 actions, walk, slot assign. Episode 1 opening first | 6 | 6 | `0x1DA10`, `0x1D95E`, `0x1DAC4`, `0x1DB0C`, `0x1DB5E`, `0x1DF10` | T-B0, P-D2a-6, M0.1 | open | D4c s11 |
| T-C2 | Intro handshake `G.$4E2` / `G.$4E3`, intro states 4 and 6, HUD slide limit | 1.5 | 1.5 | `flow.rs` intro task | T-C1, T2.7 | open | D4c s11 |
| T-C3 | GO hint pool B 20 variants 0 and 2, sound `0x3E` | 1 | 1 | `0x7BB68`, `G.$4FE`, `G.$500` | T-B0 | open | D4c s11 |
| P-D2a-6 | Scripted player states `0x2A`, `0x2C`, air `0x32`, `0x34`, run state 6 | 1.5 | 1.5 | `0x19EA6`, `0x19F40`, `0x1E2EE` | T-C1 (same change) | open | D2a s18 |
| T-B0 | Tile block writer, prop hit pass `0x10CD0`, break sound `0x9E60`, script action hook | 3 | 3 | `0x10CD0`, `0x9E60`, list `0x6724` | M0.1 | open | D3e s8 |
| T-TR1 | Clock display pool B 31, pool D slot 15, red under 1:00, beep `0x34` | 1.5 | 1.5 | `0x81046`, `0x811CA` | T-B0 | open | D4b s6 |
| T-TR2 | TIME OVER words pool C `0x70`, and the +3:30 / wave +600 link | 1 | 1 | `0x96450`, `0x10482` | none | open | D4b s6 |
| T-TR3 | Rank event wiring: death and extra life are in. Join, continue, and `G.$4F6` are not | 1.5 | 0.75 est | `0x184E2`, `0x1842C`, `0x1843C` done. Still `0x18410`, `0x18566`, `G.$4F6` | T2.3, T2.4, T1.8 | PARTIAL `87bc612` | D4b s6 |
| T-TR7 | Config file for difficulty, timer, extra lives, start lives, chute mode | 1 | 1 | `0xACBF4` factory byte, D4d s4 | T2.1 | open | D4b s6 |
| P-D2b-3 | Respawn air strike controller pool B 18, five bombs | 1.5 | 1.5 | `0x7B65E`, `0x18C5E`, `0x12CAC` | none | open | D2b s10 |
| P-D2a-3 | 1UP pose states `0x18`, `0x418`, `0x41A`, voice `0x9814`, 180 frame blink | 0.5 | 0.5 | `0x6B98`, `0x19996`, `0x9814` | none | open | D2a s18 |
| P-D2b-2 | Speech bubble and "1UP!" marker (pool C 14), sparkles pool C 23 | 1.5 | 1.5 | `0x93B28`, `0x93BF8`, `0x958E4` | P-D2a-3 | open | D2b s10 |
| P-D2c-5 | HUD weapon name, tile, ammo, bar, one block per player | 1.5 | 1.5 | `0x2389C` to `0x23EDA`, names `0x23D0B` | none | open | D2c s20 |
| B66 | Caption banners, var picks the string. Needs the tall font from T1.1 | 1 | 1 | `0x86D76`, table `0x87710` | T1.1 | open | D3e s8 |
| B74 | Master of the Episode 1 opening (cast A54, A55, A56) | 1 | 1 | `0x89B48` | T-C1 | open | D3e s8 |
| IT-P4 | HUD item messages against the MAME pictures, queue, 60 frame timer | 0.5 | 0.5 | `hud_food_msg`, `hud_treasure_msg` | none | open | D4a s12 |
| P-D2c-7 | Harness: 2 and 3 players, second and third input, snapshots, scenarios M1 to M10 | 3 | 2 est | `re/harness` | none | PARTIAL `46ab87b` (boot only) | D2c s20 |

T2.1 also says "the config file". That file is T-TR7. Build it once.

T-C1 is one 6h task. M2 uses scripts 1, 8, 13, 14 and the area end scripts. M3 uses the rest of the same task. The 6h sits in M2 because the task starts there. It also gates every later area change.

P-D2a-6 and T-C1 edit the same player path. One agent should do them together.

## M3 whole-game skeleton

Sum of this table: 18.4h. Ending tasks, placeholders, and the debug keys have no hours. They are in the gap table. They still belong to M3.

| ID | Scope | Hours | Left | Routines or classes | Depends on | Status | Source |
| --- | --- | --- | --- | --- | --- | --- | --- |
| T-C4 | Boss death chain: class 22 calls `0x1D95E` at HP 0, hand off to the clear task | 1.5 | 1.5 | `0x7BEF6`, `0x1D95E`, `G.$4D1` | T-C1, P-D3c-1 | open | D4c s11 |
| T-C5 | Episode 3 camera controller, mode 8 car chase. Car objects stay in P-D3c-5 | 3 | 3 | `0x22398`, mode table `0x2250A`, events `0x22B9E` | none | open | D4c s11 |
| T-C6 | Screen shake state machine and the scroll offset | 1.5 | 1.5 | `0x222D6` | none | open | D4c s11 |
| T-C7 | Row scroll task and a per line scroll 2 offset | 3 | 3 | `0x2335A` | none | open | D4c s11 |
| T-C8 | Lift: scroll 1 background, classes 57 and 64, by-Y spawn | 2.5 | 2.5 | `0x205C6`, `0x85B64`, `0x86B64`, tables `0x138C48`, `0x15CFE8` | T-B0 | open | D4c s11 |
| T-C9 | Area change: brightness dip, class 96 music after mode 8, time and rank per area | 1.5 | 1.5 | mode `0x12`, `0x92382`, `G.$514` | T-C1 | open | D4c s11 |
| T-C10 | Scenarios A1 to A17 as harness checks, plus an end-chain trace | 3 | 3 | `re/harness` | T-C1, T-C2, T-C3 | open | D4c s11 |
| HT10 | Scene actors: Cadillac A16/A17, Episode 4 opening A27 and A43 | 2 | 2 | `0x420EA`, `0x423A4`, `0x4D900`, `0x55F2A` | T-C1, B66 | open | D3a s15 |
| B21 | One-shot tile attribute change. Script action `0x24` | 0.4 | 0.4 | `0x7BDC4`, `0x1DECC` | T-B0, T-C1 | open | D3e s8 |

## M4 sound

Sum of this table: 8.75h. The driver (NEW-PLAN T4.1 to T4.4) has no design-doc hours. See the gap table. D4e says those driver hours are outside S1 to S13.

| ID | Scope | Hours | Left | Routines or classes | Depends on | Status | Source |
| --- | --- | --- | --- | --- | --- | --- | --- |
| S1 | Attract flush filter. Keep `0x30` and `0xFFxx`. Drop other codes unless demo sound is on | 0.5 | 0.5 | `0x610` | T1.8 | open | D4e s8 |
| S2 | Store d1 and d2 on every sound request (pan, middle byte) | 2 | 2 | `Game::sound`, `0xA63C` | none | open | D4e s8 |
| S3 | Front end `sound()` lines: title, ranking, slides, select, coin, continue, game over, story, tally, ending | 2.5 | 2.5 | the screen files from D4d | T1.2, T2.2, T2.4, T2.6, ending | open | D4e s8 |
| S4 | Boss intro themes and fade `0xA5EA` on the death. One small call per boss init | 0.5 | 0.5 | each boss init | P-D3c-2 and later bosses | open | D4e s8 |
| S10 | Episode 7 lock 2 with boss `+$7D == 0xFE` plays `0x0F`, not `0x12` | 0.25 | 0.25 | `flow_area_music`, `0xA55A`, `0xA5C8` | M0.1 | open | D4e s8 |
| P-D2a-7 | Move voices and hit sounds from the attack records | 1.5 | 1.5 | `snd_player`, record table | S2 | open | D2a s18 |
| P-D2c-9 | Weapon sounds and voices (rifle, pistol, Uzi, shotgun, bazooka, throw) | 1.5 | 1.5 | `0x9C76`, `0x9CD8`, `0x9D9C`, `0xA442`, `0x9E08`, `0x9D44` | S2 | open | D2c s20 |

S3's 2.5h is spread across the screens, not one sitting. S4's 0.5h is the whole set of boss inits, not 0.5h per boss.

## M5 Episode 1 for real

Sum of this table: 31.55h listed, 31.3h left.

| ID | Scope | Hours | Left | Routines or classes | Depends on | Status | Source |
| --- | --- | --- | --- | --- | --- | --- | --- |
| IT-P1 | Drop table through enemy row `0x3A`, treasure score index 16, test row `0x30` | 0.25 | 0 | `dino-extract` enemy drop rows | none | DONE `76ea52e` | D4a s12 |
| P1 | Monster hit lists and passes, colour flash, poke `0x533FA` | 2 | 2 | `0x6B64`, `0x11A9E`, `0x11B36`, `0x1132A`, `0x11CEC`, `0x11E56`, `0x11CC4` | none | open | D3b s13 |
| P-D3c-1 | Boss services: class 22 lists, name, HP bar, death hooks, `G.$4FC` | 3.5 | 3.5 | `0x7BEF6`, `0x3262A`, `0x32740`, `0xA5EA` | T-B0 | open | D3c s9 |
| P-D3c-2 | Vice (class 18 variant 0), bullet class 59, Hopper hook | 7 | 7 | `0x42590` variant 0, tables `0x44B84`, `0x44C8C`, `0x43004` | P-D3c-1, P1, P4 intro | open | D3c s9 |
| P4 | Rock Hopper class 29, including the Vice partner intro | 3.5 | 3.5 | `0x4F4E4`, step `0x4FA30`, attacks `0x20`, `0x21`, `0x29` | P1, P-D3c-2 | open | D3b s13 |
| B40 | Steam pipes. Pieces (class 15) already exist | 1 | 1 | `0x836A4` | T-B0 | open | D3e s8 |
| B11 | Metal end door smashed on the scripted walk | 0.4 | 0.4 | `0x789EE`, action `0x40` | T-B0, T-C1 | open | D3e s8 |
| B12 | Hall door (var 0) and hall window (var 4, armed by `G.$506`) | 1 | 1 | `0x78B52` | T-B0, T-C1 | open | D3e s8 |
| B14 | Building hole (var 0) and wooden door (var 4) | 0.4 | 0.4 | `0x7B0BE` | T-B0 | open | D3e s8 |
| P-D2c-2 | Pass `0x10DCC`: weapon hits on players, 1 HP floor, full damage for enemy-owned objects | 1.5 | 1.5 | `0x10DCC`, list `0x671A` | none | open | D2c s20 |
| P-D2c-6 | Weapon muzzle, spark, and dust. Debris class `0x2C` itself is IT-P2 | 1.5 | 1.5 | `0x96B40`, `0x92E5E`, spark `0x1F` | D3f (missing) | open | D2c s20 |
| P-D2a-8 | Move effects: dust, sparkle blink, dizzy stars, water ripple | 1.5 | 1.5 | `0x92E5E`, `0x958E4`, `0x94338` | D3f (missing) | open | D2a s18 |
| IT-P2 | Debris `0x2C`, puff, fuse spark, stones from bonus pieces | 1.5 | 1.5 | `0x934F6`, `0x93770`, `0x94640`, `0x9625C` | D3f (missing) | open | D4a s12 |
| P-D2a-5 | Wall jump check in Episode 1 area 2, find the real spots | 0.5 | 0.5 | `0x1A744` | none | open | D2a s18 |
| P-D2a-4 | MAME check of Mustapha, Hannah, Mess moves. Fix what differs | 1.5 | 1.5 | scenarios A1 to A13 | none | open | D2a s18 |
| P-D2b-6 | MAME check of respawn scenarios S1 to S21. Fix what differs | 1.5 | 1.5 | `re/harness` | P-D2b-2, P-D2b-3, T-TR2 | open | D2b s10 |
| P-D2c-8 | Weapon scenarios W1 to W14 for four heroes. Fix what differs | 1.5 | 1.5 | `batchW_big.txt` | P-D2c-2, P-D2c-5 | open | D2c s20 |
| IT-P3 | Item scenarios A01 to A25. Fix what differs | 1.5 | 1.5 | D4a batches | IT-P2 | open | D4a s12 |

P4 is one 3.5h task. The Vice intro states gate P-D3c-2. The hostile modes are Episode 5 (M7) and reuse this same task. Hours are counted once, in M5. The doc does not split them.

P-D3c-2 and P4 depend on each other. Port the Hopper intro states in the same pass as Vice's spawn.

P-D2c-6 and IT-P2 both name class `0x2C`. IT-P2 owns that class. P-D2c-6 keeps muzzle, spark, and dust. Some of the 1.5h may still overlap. D3f would draw the line.

## M6 Episodes 2 and 3

Sum of this table: 50.6h.

| ID | Scope | Hours | Left | Routines or classes | Depends on | Status | Source |
| --- | --- | --- | --- | --- | --- | --- | --- |
| HT2 | Poachers class 13, bullet 59, muzzle flash 57 | 4 | 4 | `0x3DDEA`, `0x57FB4`, `0x57D94` | P1, P-D3c-2 (bullet 59) | open | D3a s15 |
| HT3 | Dinosaur hunter class 79 | 1 | 1 | `0x5F5D0`, poke `0x533FA` | P5 | open | D3a s15 |
| HT4 | Lash class 3, mace A62, chain piece A8 | 4 | 4 | `0x358B2`, `0x5A47C` | P1 | open | D3a s15 |
| HT6 | Alternate Blade class 63 | 1 | 1 | `0x5AA6E` | class 41 and 60 (ported) | open | D3a s15 |
| HT7 | Bomb carriers 64 and 77, dynamite | 2.5 | 2.5 | `0x5AFC2`, `0x5EDF4` | P-D2c-2, D3f (missing) | open | D3a s15 |
| HT8 | Walther, class 18 variant 2, on Vice's AI | 0.5 | 0.5 | `0x42590` variant 2 | P-D3c-2 | open | D3a s15 |
| HT9 | Barrel 65, rock 66, biker 70 | 2.5 | 2.5 | `0x5B8B2`, `0x5BB8A`, `0x5C356` | P1, D3f (missing) | open | D3a s15 |
| P3 | Mack class 38 | 1.5 | 1.5 | `0x53BE4` (charge cycle) | P1 | open | D3b s13 |
| P5 | Shivat class 24, plus the hunter poke | 4 | 4 | `0x48E62`, modes in D3b s5 | P1, HT3 | open | D3b s13 |
| P7 | Ground critters A78 and A80. Walk to the hero, stand | 0.5 | 0.5 | `0x5F198`, `0x5FF2E` | none | open | D3b s13 |
| B2 | Swamp water surface | 1 | 1 | `0x7542C`, records `0x7550E` | T-B0 | open | D3e s8 |
| B7 | Far-layer or near-layer tile anim, var 4 in Episode 2 | 0.4 | 0.4 | `0x76CD2` | T-B0 | open | D3e s8 |
| B27 | Butcher yard tile anim | 1 | 1 | `0x7EEBA` | T-B0 | open | D3e s8 |
| B72 | Bird flock spawner | 0.4 | 0.4 | `0x89984` | T-B0 | open | D3e s8 |
| A46+A48 | Birds and the trail that B72 spawns | 0.5 | 0.5 | pool A 46, 48 | B72 | open | D3e s8 |
| B80 | Jungle sound maker, sound `0x78` | 0.4 | 0.4 | `0x8A582` | S2 | open | D3e s8 |
| B95 | Episode 3 sky colour blend | 0.4 | 0.4 | `0x92220`, tables `0x922D0` | T-B0 | open | D3e s8 |
| P-D3c-3 | Butcher: starter class 5, body class 61, knives class 52 | 8 | 8 | `0x38DDE`, `0x58CD6`, `0x59678` | P-D3c-1 | open | D3c s9 |
| P-D3c-4 | Hogg, bike, bomb class 42 | 7 | 7 | `0xC821A`, `0xC6252`, class 31, 32, 42 | P-D3c-1, P-D3c-5 | open | D3c s9 |
| P-D3c-5 | Cadillac drive: pool B 88, steering, hits, boarding, radio, B89, B90 | 10 | 10 | `0x8BD24`, `0x8BDDC`, `0x121D8` | T-C5, T-C1 | open | D3c s9 |

HT3 and P5 depend on each other. The hunter is what wakes Shivat. Do them in one pass.

HT7 and HT9 also appear in Episodes 4, 5 and 7. The hours are counted once, in M6, where they first show up.

Rock 66 is an Episode 5 object inside HT9. Same rule: one task, counted in M6.

## M7 Episodes 4 and 5

Sum of this table: 47.4h.

| ID | Scope | Hours | Left | Routines or classes | Depends on | Status | Source |
| --- | --- | --- | --- | --- | --- | --- | --- |
| HT5 | Bludge class 6, acid A7, item eating | 4.5 | 4.5 | `0x39272`, `0x3B006` | P1 | open | D3a s15 |
| P2 | Zeke class 82. Also used in Episodes 7 and 8 | 2.5 | 2.5 | `0x6137A`, `0x6141A`, grab `0x125C0` | P1 | open | D3b s13 |
| P-D3c-6 | Slice class 25, blade class 19, afterimage C27, opening actors | 7 | 7 | `0x4D268`, `0x453FE`, C27 | P-D3c-1, HT10 | open | D3c s9 |
| P-D3c-7 | Acceptance runs for Vice, Butcher, Hogg, the car, and Slice | 6 | 6 | BS-A1 to BS-E1 | P-D3c-2, P-D3c-3, P-D3c-4, P-D3c-5, P-D3c-6 | open | D3c s9 |
| P-D3d-1 | Shared boss skeleton: phase timers, action tables, recover, target rule | 5 | 5 | `0x2B56C`, `0x32032`, `0x325E6`, `G.$4D6` | P-D3c-1 | open | D3d s9 |
| P-D3d-2 | Morgan scene and Morgan (classes 35, 45, 36, 68, and the Uzi) | 8 | 8 | `0x513DA`, `0x5639A`, class 36 | P-D3d-1, B50 | open | D3d s9 |
| P-D3d-3 | Morgue class 40 | 4 | 4 | class 40 | P-D3d-2 | open | D3d s9 |
| P-D2c-4 | Torch glow overlay, item class 6 | 0.5 | 0.5 | `0x29B06`, `0x29C26` | none | open | D2c s20 |
| B0 | Breakable wall panel, 15 variants | 1 | 1 | `0x74FA2`, boxes `0x750B4` | T-B0 | open | D3e s8 |
| B1 | Breakable chain-link fence | 1 | 1 | `0x75238` | T-B0 | open | D3e s8 |
| B59 | Episode 4 palette cycle | 0.4 | 0.4 | `0x861E8`, table `0x862B6` | T-B0 | open | D3e s8 |
| B5 | Far layer anim, Episode 5 | 0.4 | 0.4 | `0x769E2` | T-B0 | open | D3e s8 |
| B6 | Scroll 2 steam or water block | 0.4 | 0.4 | `0x76A70` | T-B0 | open | D3e s8 |
| B10 | Tile sets tied to the global tile clock | 1 | 1 | `0x785F0`, `G.$7AE` | T-B0 | open | D3e s8 |
| B13 | Burning huts controller. Role still to confirm in a walk | 2.5 | 2.5 | `0x79218` | T-B0 | open | D3e s8 |
| B48 | Stub that waits 128 frames | 0.4 | 0.4 | `0x84316` | T-B0 | open | D3e s8 |
| B49 | Wall patch that spawns two dialogue boxes | 1 | 1 | `0x8438E` | T-B0, B50 | open | D3e s8 |
| B50 | Dialogue box with portrait. Also used by boss intros | 1 | 1 | `0x844B4`, printer `0x84F72` | T1.1 | open | D3e s8 |
| B62 | Episode 5 palette cycle, sound `0x8E` | 0.4 | 0.4 | `0x865F2` | T-B0 | open | D3e s8 |
| B70 | Small scenery object, freed off screen | 0.4 | 0.4 | `0x896E6` | T-B0 | open | D3e s8 |

P2 is counted in M7. Episodes 7 and 8 reuse it.

B13's picture is not confirmed. D3e part 2 is only a desk list.

## M8 Episodes 6, 7 and 8

Sum of this table: 68.2h. These pool B rows use part 1 sizes. There is no Episode 6 to 8 walk.

| ID | Scope | Hours | Left | Routines or classes | Depends on | Status | Source |
| --- | --- | --- | --- | --- | --- | --- | --- |
| P6 | Brontosaurus feet, classes 22 and 23 | 1.5 | 1.5 | section 6 of D3b, `G.$755C = 4` | P1, T-C6 | open | D3b s13 |
| P-D3d-4 | Tyrog three forms, starter B79, classes 4, 72, 74, 75, 76 | 12 | 12 | class 4, `prop79` | P-D3d-1 | open | D3d s9 |
| P-D3d-5 | Tyrog 2 class 91, table differences on the class 4 code | 2 | 2 | class 91 | P-D3d-4 | open | D3d s9 |
| P-D3d-6 | Slisaurs: starter B33, class 71, reuse class 19 and C27 | 6 | 6 | class 71, `0x818E4` | P-D3d-1, P-D3c-6 | open | D3d s9 |
| P-D3d-7 | Tank breakers class 44, shard effect, creature table | 3 | 3 | `0x83BB0`, `0x83C36`, `0x83D84`, C21 | T-B0 | open | D3d s9 |
| P-D3d-8 | Fessenden form 1 class 94, intro, death lines, growth | 5 | 5 | class 94, captions `0x3D` to `0x42` | P-D3d-1, B66 | open | D3d s9 |
| P-D3d-9 | Fessenden form 2, classes 84 to 89, head fire 95 | 12 | 12 | class 84, 85 to 89, 95 | P-D3d-8 | open | D3d s9 |
| P-D3d-10 | Acceptance runs BT-A1 to BT-A17 | 6 | 6 | harness | P-D3d-2 through P-D3d-9 | open | D3d s9 |
| S11 | Biker, barrel, boomerang, roar, and foot sound helpers | 0.5 | 0.5 | `0xA128`, `0xA164`, `0xA3C6`, `0xA21E`, `0xA09E`, `0xA288` | HT9, P-D3c-6, P6 | open | D4e s8 |
| B3 | Breakable wall, Episode 6 | 1 | 1 | `0x763F6` | T-B0 | open | D3e s8 |
| B4 | Wave wall that bursts open | 1 | 1 | `0x767D8` | T-B0 | open | D3e s8 |
| B8 | Water surface, Episode 6 | 1 | 1 | `0x76E0E` | T-B0 | open | D3e s8 |
| B9 | Two layer anim with random pauses | 1 | 1 | `0x77E76`, sound `0xA25A` | T-B0 | open | D3e s8 |
| B39 | Four picture anim while camera Y is low | 0.4 | 0.4 | `0x830C6` | T-B0 | open | D3e s8 |
| B55 | Rain streaks | 0.4 | 0.4 | `0x8574C`, helper `0xF94` | T1.1 | open | D3e s8 |
| B56 | Falling dust, pool C class 2 | 0.4 | 0.4 | `0x85916` | D3f (missing) | open | D3e s8 |
| B60 | Mine lamp palette cycle | 0.4 | 0.4 | `0x863AC` | T-B0 | open | D3e s8 |
| B30 | Episode 7 lab machine, spawns caption B34 | 2.5 | 2.5 | `0x811DE`, sound `0x23` | T-B0, B34 | open | D3e s8 |
| B34 | Lab caption, three tall-font lines | 0.4 | 0.4 | `0x81B3E` | T1.1 | open | D3e s8 |
| B41 | Four picture anim | 0.4 | 0.4 | `0x8383A` | T-B0 | open | D3e s8 |
| B52 | Wave wall break, Episode 7 | 0.4 | 0.4 | `0x85178` | T-B0 | open | D3e s8 |
| B54 | Two water jets | 0.4 | 0.4 | `0x855A8` | T-B0 | open | D3e s8 |
| B61 | Palette cycle, sound `0x6D` | 0.4 | 0.4 | `0x864DA` | T-B0 | open | D3e s8 |
| B63 | Palette cycle guarded by a stage test | 0.4 | 0.4 | `0x869E6` | T-B0 | open | D3e s8 |
| B28 | Tile anim on the global clock, Episode 8 | 1 | 1 | `0x7F72A`, `G.$7AE` | T-B0 | open | D3e s8 |
| B29 | Same family, table `0x80518` | 0.4 | 0.4 | `0x8048A` | T-B0 | open | D3e s8 |
| B36 | Tile anim, blocks `0x90C038` | 0.4 | 0.4 | `0x8217E` | T-B0 | open | D3e s8 |
| B37 | Tile anim, vars 0, 4, 8 | 0.4 | 0.4 | `0x82372` | T-B0 | open | D3e s8 |
| B38 | Tile anim, vars 0 to 5 | 1 | 1 | `0x8279C` | T-B0 | open | D3e s8 |
| B42 | Wave timed lab wall break | 1 | 1 | `0x83A0A`, sound `0x26` | T-B0 | open | D3e s8 |
| B43 | Four pictures of 8 frames | 0.4 | 0.4 | `0x83ACA` | T-B0 | open | D3e s8 |
| B45 | Flame jet vent, HP `0x30`, pool A class 83 flames | 2.5 | 2.5 | `0x83E0C`, `0x6152C` | T-B0, D3f (missing) | open | D3e s8 |
| B53 | Lab palette set switcher | 1 | 1 | `0x85306`, `G.$7B2` | T-B0 | open | D3e s8 |
| B71 | End area palette anim | 0.4 | 0.4 | `0x89746` | T-B0 | open | D3e s8 |
| B83 | Debris shower | 0.4 | 0.4 | `0x8B212`, class 92 | T-B0 | open | D3e s8 |
| B84 | Debris shower, second pattern | 0.4 | 0.4 | `0x8B2CA`, class 93 | T-B0 | open | D3e s8 |
| B85 | Ending quake and debris. Also used by the ending scenes | 0.4 | 0.4 | `0x8B34C`, `G.$755C` | T-C6 | open | D3e s8 |

## M9 small things

Sum of this table: 4.9h listed, 4.4h left.

| ID | Scope | Hours | Left | Routines or classes | Depends on | Status | Source |
| --- | --- | --- | --- | --- | --- | --- | --- |
| P-D2a-1 | Friendly fire gate: census count, not a constant 3 | 0.5 | 0 | `player_g4f3`, `0x10664`, `G.$4F3` | none | DONE `a823109` | D2a s18 |
| P-D2c-3 | Tag team: trigger, state `0x1A`, four partner states, three player order | 2.5 | 2.5 | `0x6728`, `0x686E`, `0x6922`, table `0x69D4` | P-D2c-7 | open | D2c s20 |
| P-D2b-5 | Unit tests: hero factors, mercy, `$C6`, lie, dizzy, extra life lists, score cap | 0.5 | 0.5 | `0x2C88E`, `0x1D17E`, `0x6B98` | none | open | D2b s10 |
| P-D2c-10 | Rename weapon types in comments and the SPEC table | 0.5 | 0.5 | `weapon.rs`, `blast.rs` | none | open | D2c s20 |
| IT-P9 | Measure scenarios A26 and A27. Decide the 3 player drum | 0.5 | 0.5 | new probe segments | none | open | D4a s12 |
| B69 | Debug cursor. Frees itself unless `G.$4AA` is set. A stub is enough | 0.4 | 0.4 | `0x88A84` | none | open | D3e s8 |

## Folded duplicates

These rows are real tasks in a doc. The hours are not added again.

| Dropped | Kept | Hours not added | Why |
| --- | --- | --- | --- |
| P-D2a-2 | P-D2c-3 | 1.5 | D2c says P-D2c-3 replaces P-D2a-2 |
| P-D2c-1 | P-D2a-1 | 0.5 | Same gate. The 2 and 3 player scenario is inside P-D2c-7 |
| P-D2b-1 | T-TR2, and `87bc612` | 0.5 | Rank calls `0x184E2` and `0x1842C` are in that commit. Clock +3:30 and wave +600 are T-TR2 |
| P-D2b-4 | P-D2c-2 and P1 | 1.5 | Same two passes: `0x10DCC` and `0x1132A` |
| P-D2b-7 | T-TR7 | 0.5 | Extra life settings are part of the config file |
| T-TR4 | P-D2a-3 and P-D2b-2 | 1.0 | Pose plus the "1UP!" marker |
| T-TR5 | T2.6 | 2.0 | Tally math is inside the tally task |
| T-TR6 | T1.4 and T2.5 | 2.0 | Ranking screen and name entry |
| S5 | T-TR1 | 1.0 | Clock beep `0x34` is inside the clock task |
| S6 | T-C3 | 0.25 | GO sound `0x3E` is inside T-C3 |
| S7 | T-C9 | 0.5 | Class 96 delay is inside the area change task |
| S8 | T-C1 | 0.5 | Victory sound `0x18` is script action `0x04` |
| S9 | P-D2a-3 | 0.5 | Pose voice `0x9814` is already in the pose task |
| S12 | P-D2c-9 | 0.5 | Player shot rows, and only if a hero-owned shot calls them |
| IT-P5 | P-D3c-5 | 0.5 | Radio hook is the start of the car task |
| IT-P6 | P-D2c-4 | 0.5 | Same torch glow |
| IT-P7 | HT9 | 1.5 | Drop calls of classes 65, 66, 70 ship with those classes |
| IT-P8 | P-D3c-1 | none printed | Boss helper lists are part of boss services |
| HT1 | T-B0, P-D2c-2, P1 | 1.0 | The three passes are those three tasks. The `+$84` quirk has no hours of its own |
| HT11 | P-D3d-2 | 1.0 | Morgan actors 35 and 45 are inside the Morgan task |
| B20 | T-C3 | 0.4 | Same GO hint |
| B31 | T-TR1 | 1.0 | Same clock |
| B18 | P-D2b-3 | 1.0 | Same air strike |
| B96 | T-C9 | 0.4 | Same music starter |
| B88, B89, B90 | P-D3c-5 | 3.3 | Car controller and its two helpers |
| B79 | P-D3d-4 | 1.0 | Tyrog starter |
| B33 | P-D3d-6 | 1.0 | Slisaur starter |
| B44 | P-D3d-7 | 2.5 | Tank breaker |
| B57, B64 | T-C8 | 0.8 | Lift digits and red pillars |
| 22 front end pool B classes | D4d M1 to M3 tasks | about 20 | D3e s8. Select 17, 19, 67, 68. Movie 23 to 26, 32. Story 91 to 94. Ending 81 to 87. Map 58. Continue object 86 |

Folded hours above add to about 44, plus the 20h front end bundle. They are not in the milestone totals.

## Totals

Counted rows only. Folded rows are excluded. "Left" uses the est cells for T-TR3 and P-D2c-7.

| Milestone | Counted tasks | Hours listed | Hours left | What the hours are |
| --- | --- | --- | --- | --- |
| M0 | 1 | 1.5 | 0 | Done |
| M1 | 9 | 17.5 | 17.5 | D4d attract. NEW-PLAN had 10 |
| M2 | 24 | 47.0 | 45.25 | Front end play start, scripts, clock, two partial tasks |
| M3 | 9 | 18.4 | 18.4 | Camera services and scene actors. Ending and placeholders not included |
| M4 | 7 | 8.75 | 8.75 | Cue wiring. Driver not included |
| M5 | 18 | 31.55 | 31.3 | Episode 1, boss services, Vice, Hopper. IT-P1 done |
| M6 | 20 | 50.6 | 50.6 | Episodes 2 and 3, car, Butcher, Hogg |
| M7 | 20 | 47.4 | 47.4 | Episodes 4 and 5, Slice, Morgan, Morgue, first boss acceptance |
| M8 | 37 | 68.2 | 68.2 | Episodes 6 to 8, Tyrog, Slisaurs, Fessenden |
| M9 | 6 | 4.9 | 4.4 | Tag team and small fixes. Gate is done |
| Counted | 151 | 295.8 | 291.8 | |
| NEW-PLAN M0 to M10 | about 101 build tasks | about 177.5 | | Section 8, build rows only, design hours not included |

295.8 is 1.5+17.5+47.0+18.4+8.75+31.55+50.6+47.4+68.2+4.9.

291.8 left is that sum minus M0 1.5, IT-P1 0.25, P-D2a-1 0.5, the done half of T-TR3 (0.75), and the done part of the harness (1.0). Those five subtractions are 4.0.

The counted total is higher than NEW-PLAN's 177.5 build hours because the design docs split bosses, pool B classes, and the front end into real tasks. NEW-PLAN M6 to M8 were "about 14 / 16 / 20 tasks".

Likely double count still inside the 295.8, about 2 to 4h: T-TR7 sits inside T2.1's config bullet, and P-D2c-6 still overlaps IT-P2 until D3f exists. Not subtracted.

D3e says one generic block driver (that driver is T-B0) can cut most S class rows from 0.4h to 0.2h. The totals above use 0.4.

## Critical path

The chain that the most later work waits on. Hours are hours left.

| Step | Task | Hours left | What waits on it |
| --- | --- | --- | --- |
| 1 | M0.1 | 0 | Done. Episodes 2 to 8 can start |
| 2 | T1.1 | 2 | Every front end screen, B66, B50, the tally text |
| 3 | T1.2 | 2 | Title, then the attract machine |
| 4 | T1.8 | 1.5 | Coin edge and press start |
| 5 | T2.1 | 2 | Select, join, continue |
| 6 | T2.2 | 3 | A game can be started |
| 7 | T-B0 | 3 | Every tile prop, the GO hint, the clock, scripted doors |
| 8 | P-D2a-6 with T-C1 | 7.5 | Episode 1 opening, every area change, every boss end walk |
| 9 | T-C2 | 1.5 | Control returns to the player after an intro |
| 10 | T3.6 placeholders | no hours | The playable path through all 8 episodes. No design doc owns it |
| 11 | P-D3c-1 | 3.5 | Every real boss (Vice through Fessenden) |
| 12 | P-D3d-1 | 5 | Morgan, Morgue, Tyrog, Slisaurs, Fessenden |
| 13 | P-D3d-8 | 5 | Fessenden form 2 |
| 14 | P-D3d-9 | 12 | Longest single class task |
| 15 | P-D3d-10 | 6 | Closes the last boss acceptance |

Steps 2 to 9 are 20.5h of known work before a skeleton can change areas. Step 10 has no size. Steps 11 to 15 are 31.5h and they are the longest boss tail.

Other gates, shorter than that tail:

| Gate | Hours left | Downstream it blocks |
| --- | --- | --- |
| T-C1 | 6, already on the path | About 220h of counted M3 to M9 work needs a running skeleton |
| P-D3c-1 | 3.5 | Boss tasks after it: about 108h (rest of D3c and all of D3d) |
| T-B0 | 3 | Pool B rows counted after it: about 40h, plus doors inside M2 |
| P1 | 2 | Dinosaur tasks P2 to P6 and hazard damage in HT9: about 16h |
| T-C5 | 3 | P-D3c-5 car, 10h, and Hogg which rides on that car |

P4 and P-D3c-2 block each other. The intro slice of P4 has no separate hour figure. Until D5 splits it, Vice cannot be signed off, and the 7h Vice task stays on the path into M6 (HT8, HT2's bullet).

M1 screens other than T1.1, T1.2, and T1.8 (profiles, ranking, demo, slides, movie: 12h) are not on this chain. One code agent still does them in series if nobody else takes them. They do not gate M2.

## No owner, or the size is unclear

| Item | Milestone | Problem |
| --- | --- | --- |
| M0.1 | M0 | No design doc. Size 1.5h comes from NEW-PLAN. Already done |
| T3.6 one placeholder enemy per missing class, invisible props, placeholder bosses | M3 | NEW-PLAN s6 only. No hours of its own. It is on the critical path |
| T3.9 `--episode`, debug keys, bot script per episode | M3 | NEW-PLAN s6 only. No hours |
| Ending scene 1 and 2 (pool B 82, 87, 85, 66, 83, 84) | M3 | D4d s14 names four ending tasks and gives no hours |
| Ending desert and staff roll (pool B 81) | M3 | Same |
| Ending car and dialogue | M3 | Same |
| Ending sequence and the name entry hook | M3 | Same. Name entry itself is T2.5 |
| T4.1 sound data extract (Kabuki, tables, samples) | M4 | NEW-PLAN only. D4e does not size the driver |
| T4.2 `dino-sound` driver | M4 | Same |
| T4.3 QSound mixer and a WAV example | M4 | Same |
| T4.4 `audio.rs`, cpal, `sound_ex` d1 and d2 | M4 | Overlaps S2. The driver half has no hours |
| D3f pool C and pool D catalog | M5 to M9 | File missing. P-D2a-8, P-D2c-6, IT-P2, B56, B45, and the effect calls inside HT9 have no class spec |
| D3e part 3, Episodes 6 to 8 walk | M8 | File missing. M8 prop rows are sized from the inventory only |
| D3e part 2 | M6, M7 | Pre-study only. Episode 3 to 5 prop behaviour is not signed off |
| M10, about 10 tasks, 18h | M10 | NEW-PLAN s6 and s8. No task ids. Overlaps T-C10, P-D3c-7, P-D3d-10, P-D2a-4, P-D2b-6, P-D2c-8, IT-P3, which are already counted |
| T5.6 whole Episode 1 run, 1 and 3 players | M5 or M10 | NEW-PLAN only |
| S13 stop hardcoding `tag = 0` | M4 | 0.5h only if a caller shows up. Not counted |
| B16, B35, B46, B47, B51, B65 | M9 | No spawner, or script-only. D3e says about 3.0h if D5 wants them (3.4h group minus B21, which is counted). Not counted |
| HT1 leftover: keep `+$84` across frees | M5 | No hours. D3a open item 12. D5 decides |
| P4 hour split | M5 and M7 | 3.5h is one row. Intro gates Vice. Full moves are Episode 5 |
| T-TR3 hours left | M2 | 0.75h is an index estimate. Three of the call sites are still open |
| P-D2c-7 hours left | M2 | 2h is an index estimate. Boot of 2 and 3 players is in `46ab87b`. Scenarios M1 to M10 are not |
| Easter egg, random tasks, test mode, `mechanics-seen.md` re-read | M9 | NEW-PLAN M9 names them. No task id and no hours |
| Episode 8 group total | M8 | D3e s8 says 12.8h. The letter sizes of those classes add to 12.2h. Gap 0.6h, not added |
| D3a section 15 total | | The section says about 28h. The 11 rows add to 24h. This index uses the rows |
| D3b section 13 total | | The section says about 15h. The 7 rows add to 15.5h. This index uses the rows |
