# Cadillacs and Dinosaurs (CPS1 "dino", World 930201): enemies, AI, stage events, items

Topic owner: enemy agent. Folder: `re/specs/enemy/`. All addresses are 68000 program addresses (ROM image `out/dino_68k.bin`).
RAM globals are written as `$xxxx(a5)` = 0xFF8000+xxxx (a5 = 0xFF8000).

Evidence tags used in this document:
- **[O]** observed: read directly in code or data.
- **[I]** inferred: a reasonable reading of observed code. The behaviour is clear but the meaning or name is a guess.
- **[U]** unknown: not analysed yet.

Decoded data files in this folder. Each `.txt` file is produced by the script with the matching name, run as `.venv/bin/python -I <script>.py out/dino_68k.bin [args]`.

| file (script) | content |
|---|---|
| `stage_scripts.txt` (`decode_all.py`) | All 8 stages: spawn lists 1 and 2 per area, and all wave scripts |
| `camera_stage1.txt` (`camera_segments.py <stage-1>`) | Camera segment table (lock points) for a stage |
| `class2_tables.txt` (`class2_tables.py`) | Every table used by the Ferris/Gneiss/Driver AI |
| `rank_tables.txt` (`rank_tables.py`) | Rank, level, aggression and initial-rank tables |
| `damage_tables.txt` (`dmg.py`) | Enemy attack table 0xA2EA4 and the enemy-to-player damage scaling tables |
| `hp_table.txt` (`hp.py`), `boss_hp.txt` (`boss_hp.py`) | HP by name id and level; boss HP by index, player count and level |
| `score_table.txt` (`score.py`) | Kill score per name id |
| `classmap.txt` (`classmap.py bin sweep`) | Pool A class to handler and name ids |
| `speed_tables.txt` (`speeds.py`) | Movement speed tables 0x17CE |
| `hit_tables.txt` (`hit_tables.py`) | Hit-reaction tables (knockback, lie time, dizzy, resume pointers) |
| `items_tables.txt`, `items_drops.txt` (`items_tables.py`, `items_drops.py`, `items_names.py`) | Food, treasure and weapon stats; drop tables |
| `boss_vicet.py` | Vice T. combo, action and choice tables |
| `c2.txt`, `hit_2a606.txt`, `boss_vicet_dis.txt`, `dino_shivat_dis.txt` | Raw disassembly extracts |
| `strtab2.py`, `longs.py` | Helpers: print an offset string table / a long or word pointer table |
| `sw` | Helper: prints `out/sweep.txt` lines for an address range |

Contents:
- 0 conventions
- 1 pools
- 2 object layout
- 3 types and stats
- 4 library
- 5 census and targeting
- 6 difficulty
- 7 Ferris/Gneiss/Driver AI
- 8 spawn scripts, waves, camera lock
- 9 stage 1 decoded
- 10 hit reaction and death
- 11 dinosaur Shivat
- 12 boss Vice T.
- 13 items
- 14 AI design answers
- 15 anti-tamper
- 16 open gaps

---------------------------------------------------------------------------------------------------

## 0. Conventions for the port

### Fixed point [O]
- Positions are signed 16.16 longs: `+$08` X (world pixels), `+$0C` Y = height above the floor plane (up is positive), and `+$10` Z = depth.
  On screen, sprite Y = -(Y + Z) + camera terms. A larger Z is further away and higher on screen.
- Only the high word is used as "the position" in nearly every test. Example: `move.w $8(a6),d0` reads the integer pixel.
- Velocities are signed words: `+$14` vx, `+$16` vy, `+$18` vz. Units are 1/256 px per frame.
  Accelerations are signed words in the same units: `+$1A` ax, `+$1C` ay, `+$1E` az.
- Physics step `0x189C`, axes in the order X, Y, Z: `v = v.wrapping_add(a); pos = pos.wrapping_add((v as i16 as i32) << 8)`.
- Move along a direction `0x1862`: `vx = SPD[dir].x + ax; X += vx<<8; vz = SPD[dir].z + az; Z += vz<<8`.
  Y is not touched. Note that here +$1A and +$1E act as velocity offsets, not accelerations.
- Directions `+$22` run 0..31. 0 = +Z (away), 8 = +X (right), 16 = -Z (toward the camera), 24 = -X (left).
- Facing `+$24`: 1 = facing right (+X), 0 = facing left. Drawing mirrors the sprite when the byte is nonzero.

### Unsigned window idiom
The game tests "is a in [c-w, c+w)" with one unsigned compare: `(a - c + w) as u16 < 2*w`.

It also tests "is a < b, for nearby values" with an `addi.w #$400` carry: `((b - a) as u16).checked_add(0x400)` overflows exactly when `-0x400 <= (b-a) < 0`.

The pseudocode writes these as `in_win(a - c, w)` and `is_neg(d)`. **[O]**

### RNG
There are two generators. Never mix them up; the AI uses only the second.
- `0xA9A` (LFSR in `$4(a5)`): CONTEXT.md lists it. It is not used by the enemy code examined here.
- `rng()` = **0x119C**, a 16-bit generator in `$14(a5)`. Every RNG call in this document means this routine. **[O]**

```rust
fn rng() -> u16 {                       // 0x119C, state = W($14)
    let mut x = st14; if x == 0 { x = 0x01C3 }
    let t = x.wrapping_mul(3);                       // add.w d0,d0; add.w d1,d0
    let lo = (x as u8).wrapping_add((t >> 8) as u8); // add.b d2,d1 (d1 = old x)
    let r = (t & 0xFF00) | lo as u16;                // move.b d1,d0
    st14 = r; r                                      // callers mask with &0xF, &0x1F, &0x3E, ...
}
```
The call order matters for exact reproduction. The pseudocode below keeps every `rng()` call in source order.

### Per-frame order (in-game task `game_frame` 0xAAF6) [O]

```
0x186CC  rank_tick()                      // difficulty/aggression timers (section 6); NOT the spawn script
0xE90E   objects_update():
           0x3242E enemy_census()          // counts alive enemies -> $4F3, per-player counters (section 5)
           0x18834 players input/update
           0x233C6 pool E update (items, containers)   ($874(a5), 24 x 0xC0)
           0x23420 pool A update (enemies)      ($48F4(a5), 24 x 0xE0)
           0x222D6 ... ; 0x23604 pool B update (controllers, props) ; 0x237B2 pool C ; 0x188DA ...
0x10546  collision: hit tests (0x10DCC, 0x10F96 player->enemy, 0x115B0 enemy->player, ...) then camera/layers
0x16BEA  HUD ; 0x13FA6 build sprite list ; 0x105E8 reset per-frame lists ; 0x103FE / 0x1037E stage time limit
0xFBF4   spawn_script_tick()               // stage spawn lists by camera X (section 8). Runs LAST in the frame.
```
CONTEXT.md labels 0xFBF4 as "background streaming" and 0x186CC as "stage events". The code shows something different:
0xFBF4 is the object spawn interpreter and 0x186CC is the rank tick. **[O]**

---------------------------------------------------------------------------------------------------

## 1. Object pools [O]

| pool | RAM | count x size | alloc | free | free list / count | update loop, dispatch table (index = `+$20` long offset) |
|---|---|---|---|---|---|---|
| A enemies, bosses, dinos, scenery | `$48F4(a5)` | 24 x 0xE0 | 0x483C | 0x48FC | `$6602`/`$65F8` | 0x23420, table 0x2346C, 102 classes |
| B controllers, props, scenery [I] | `$36F4(a5)` | 24 x 0xC0 | 0x4860 | 0x4938 | `$6600`/`$65F6` | 0x23604, table 0x2362E, 97 classes |
| E items: weapons, food, treasure, ammo, breakable containers, projectiles | `$874(a5)` | 24 x 0xC0 | 0x4818 | 0x48D4 | `$65FE`/`$65F4` | 0x233C6, table 0x233F0, 12 classes |
| C effects, shots | `$1A74(a5)` | 48 x 0x80 | 0x4884 | — | `$6604`/`$65FA` | 0x237B2 |
| D special | `$5DF4(a5)` | 16 x 0x80 | 0x48A8 (stores the slot in the caller's `+$98`) | — | `$6606`/`$65FC` | — |

- The allocators pop a 16-bit RAM pointer into a0 from a stack-like free list. They return carry set when the pool is empty.
- Free (`0x48FC` for pool A) pushes the slot back and clears **only bytes +$00..+$27**, plus bytes `+$59 +$63 +$7D +$87 +$96`.
  All other fields keep stale values from the previous user. The port must reproduce this, or must prove that each class initialises the field before reading it. **[O]**
- The update loop calls `CLASS[+$20/4](obj)` for every slot whose `+$00 != 0`, in slot order 0..23.
  The handler dispatches again on `+$04` (routine, even byte offset), `+$05` (sub-routine) and so on.
- Spawn pool selector table 0xFD46, indexed by `flags & 7`: 0 → A (0x483C), 1 → B (0x4860), 2 → null (never used), 3 → E (0x4818).

---------------------------------------------------------------------------------------------------

## 2. Enemy object layout (pool A, 0xE0 bytes)
Offsets are taken from the class 2 code (Ferris, Gneiss, Driver) and the shared library. Other classes reuse the low fields the same way. AI fields from +$70 up are partly class specific.

| off | size | meaning | tag |
|---|---|---|---|
| +00 | b | active flag. 1 = alive object. The hit code sets 2 while in hit reaction (makes it invulnerable to the same hit list, see section 10). | O |
| +01 | b | on-screen flag (set by 0x16FA) | O |
| +02 | b | credited player index (0..2) for score, or 0xFF / negative = nobody | O |
| +03 | b | name id (index into the name table 0x17D78; also HP/score/attack-damage row) | O |
| +04 | b | routine: 0 init, 2 alive, 4 dying, 6 free (byte offset into the class table) | O |
| +05 | b | sub-routine. In "alive": 0 AI, 2 hit reaction (0x2A606), 6 grabbed (0x2B35A) | O |
| +06 | b | AI state / reaction type (byte offset). Debug names at 0x3300A (section 7.1) | O |
| +07 | b | AI sub-state (byte offset) | O |
| +08 | l | X 16.16 | O |
| +0C | l | Y (height) 16.16 | O |
| +10 | l | Z (depth) 16.16 | O |
| +14/+16/+18 | w | vx, vy, vz (1/256 px/frame) | O |
| +1A/+1C/+1E | w | ax, ay, az | O |
| +20 | w | class * 4 (pool A dispatch) | O |
| +22 | b | move direction 0..31 | O |
| +23 | b | palette override (bit7 set → low 5 bits = palette) | O |
| +24 | b | facing (1 = right) | O |
| +25 | b | draw flags (CONTEXT.md) | O |
| +26 | w | spawn "var" word from the script. Class 2: high byte = variant (0 Ferris, 2 Gneiss, 4 Driver), low byte = entrance type | O |
| +28 | l | current animation frame record pointer | O |
| +2C..+2F | b x4 | frame record bytes. +2C "hold position", +2D turn request, +2E hit-box mode (4 = attack frame active), +2F frame timer | O / I |
| +40 | l | animation table base (word offsets per animation number) | O |
| +44 | l | hit-box table pointer (12-byte boxes) | O |
| +48/+49 | b | body box id / attack box id of the current frame | O |
| +4C | l | hit-box record pointer (from 0x11D220) | O |
| +50 | b | terrain flags from floor collision 0x12CB4. 0x10 = hop-down ledge (class 2 state 0x10); bit7 = blocked by an object at +$94 (class 2 attacks it) | O / I |
| +51 | b | airborne (1) / on floor (0) | O |
| +54/+56 | w | floor height / extra height; ground Y = +54 + +56 | O |
| +58 | b | attack id (row in 0xA2EA4) | O |
| +59 | b | hit-stop frames (set on both attacker and victim) | O |
| +5A | b | "last hit of combo" flag (1 on the final action of a combo) | O / I |
| +5B | b | spawn byte. Class 2: facing for entrances 4 and 0xA, or 0xFF = alternate idle animation | O |
| +5C | w | spawn word (class specific) | O |
| +5E | b | attack serial (copied from the global `$67BE`, then incremented). A target is hit once per serial. | O |
| +63 | b | "big" flag (>0: bounces off screen edges when killed) | I |
| +64 | l | speed table pointer (set by 0x17A4) | O |
| +68 | w | last attacker object (RAM word pointer) | O |
| +6A | w | HP for the bar, = min(HP, 100) at spawn | O |
| +6C | w | HP | O |
| +6E | w | HP before the last hit (bar animation) | O |
| +70 | w | grabber object (while grabbed) | O |
| +72 | b | 180 at spawn | U |
| +76 | w | target player object (RAM word pointer: 0xB274 / 0xB3F4 / 0xB574) | O |
| +78 | b | poise. 10 at spawn; hits subtract the attack's poise byte; 0 → knockdown | O |
| +79 | b | poise refill timer. When it reaches 0, poise = 10 | O |
| +7A | b | "wait before deciding" counter (state 0) / counter-attack delay after a flinch | O |
| +7B | b | last reaction type | O |
| +7C | b | approach slot index 0..5 (sometimes larger, see the quirk in 7.4) | O |
| +7D | b | 1 while in attack state 8, else 0. Negative = not counted as an enemy at all | O |
| +80 | b/w | generic timer | O |
| +84 | w | name id * 4 (row index for many per-type tables) | O |
| +86 | b | stuck-check timer (30 frames) | O |
| +88/+8A | w | destination X / Z (approach slot point) | O |
| +8C/+8E | w | position saved at the last stuck check | O |
| +90 | b | jump-attack sub phase | O |
| +94 | w | obstacle object to attack (when +50 bit7) | I |
| +96 | b | item dropped on death (0 = none; table 0xB6D12, see items) | O |
| +9A | b | hit-spark / dizzy row byte from the attack record | O |
| +A0 | w | re-path countdown | O |
| +A2 | w | taunt countdown | O |
| +A4 | b | taunt already used | O |
| +A5 | b | last reposition choice (0/1/2), 0xFF at spawn | O |
| +A6/+A7 | b | detour timer / detour active | O |
| +A8 | b | chosen combo 0..7 | O |
| +A9 | b | dash-check countdown (20 at spawn) | O |
| +AA | w | combo cursor (combo*6 + step) | O |
| +AC | b | dash stop-check timer (90 at spawn, then 31-frame period) | O |
| +AD | b | dash kind (0 near, 1 far) | O |
| +AE | w | X saved at the last dash check | O |
| +B0 | b | dash turned once | O |
| +B2 | w | 300 set in state 2 (not read by class 2) | U |
| +B4 | w | entrance start X | O |
| +B6 | w | entrance 4 idle timer | O |
| +B8 | b | pause counter at slot (state 6) | O |
| +B9 | b | "no dash attack in this dash" | O |
| +BA/+BB | b | entrance 6 fade timer / fade step | O |
| +BC | b | 200-frame obstacle check timer | O |
| +BD | b | random approach sub-slot, used by the shared helpers 0x32C54.. (Lash, Tyrog) | O |

### Player fields used by the AI [O]
Players are at 0xFFB274 (P1), 0xFFB3F4 (P2) and 0xFFB574 (P3).
- `+$00` active, `+$04` (word) 0x0200 = normal control, `+$05` 0 normal / 2 being hit / 6 grabbed, `+$06` 2 = walking.
- `+$24` facing, `+$6C` vitality, `+$100` score (BCD long), `+$A0` (word) current move id.
- Per-frame enemy census counters (section 5.1): `+$E4` enemies targeting me, `+$E5` of those in attack state, `+$E6` enemies left of me, `+$E7` enemies right of me. `+$E0..+$E3` hold the previous frame's values.

---------------------------------------------------------------------------------------------------

## 3. Enemy types: pool A classes, names and per-type stats

### 3.1 Name table [O]
Routine 0x17D2E prints the HUD name from the table at **0x17D78**. It is indexed by name id. Each entry is a word offset from 0x17D78, pointing at an `attr byte + text + '@'` string. The player names are at 0x17D70 (JACK.T, MUSTAPHA, HANNAH, MESS).

| id | name | id | name | id | name | id | name |
|---|---|---|---|---|---|---|---|
| 00 | FERRIS | 0A | BLK ELMER | 14 | RAZOR | 1E | HAMMER T. |
| 01 | GNEISS | 0B | WRENCH T. | 15 | MORGAN | 1F | BLK ELMER |
| 02 | POACHER J | 0C | VICE T. | 16 | MORGUE | 20 | WRENCH T. |
| 03 | HAMMER T. | 0D | WALTHER | 17 | KNIFE | 21 | ZEKE |
| 04 | R HOPPER | 0E | LASH T. | 18 | SLISAUR | 22 | FESSENDEN |
| 05 | SLICE | 0F | BUTCHER | 19 | TYROG | 23 | TYROG 2 |
| 06 | BLADE | 10 | BLUDGE | 1A | DRIVER | 24 | WRENCH T. |
| 07 | SHIVAT | 11 | MACK | 1B | TYROG | 25 | FESSENDEN |
| 08 | PUNK | 12 | SKINNER | 1C | BLADE | 26 | HOGG |
| 09 | THUG | 13 | GUTTER | 1D | PUNK | | |

### 3.2 Pool A class map (dispatch 0x2346C, `+$20 = class*4`) [O]
Found by scanning each handler's code range for `move.b #id,$3(a6)` (`classmap.txt`).
Classes not listed are scenery, projectiles or helpers. Examples: #49/#50 are parallax birds (`+$7D = 0xFF`), #59 is the Vice T. bullet.

| class | +$20 | handler | names (+$3) | kind |
|---|---|---|---|---|
| 2 | 0x08 | 0x3383A | FERRIS 0x00, GNEISS 0x01, DRIVER 0x1A (variant = +$26 high byte 0/2/4) | basic thug (worked example, section 7) |
| 3 | 0x0C | 0x358B2 | LASH T. | |
| 4 | 0x10 | 0x36A64 | TYROG | dinosaur boss |
| 6 | 0x18 | 0x39272 | BLUDGE | |
| 10 | 0x28 | 0x3B828 | HAMMER T., BLK ELMER, WRENCH T. | |
| 13 | 0x34 | 0x3DDEA | POACHER J, SKINNER, GUTTER | |
| 14 | 0x38 | 0x4020E | BLADE, RAZOR | knife thugs |
| 18 | 0x48 | 0x42590 | VICE T. (var 0, boss), WALTHER (var 2) | stage 1 boss (section 12) |
| 20 | 0x50 | 0x457B0 | PUNK, THUG | |
| 24 | 0x60 | 0x4838C | SHIVAT | dinosaur (section 11) |
| 25 | 0x64 | 0x4BE70 | SLICE | boss |
| 29 | 0x74 | 0x4DCF6 | R HOPPER | dinosaur |
| 31 | 0x7C | 0x4FCB4 | HOGG | boss (stage 3, car stage) |
| 34 | 0x88 | 0x5129E | KNIFE | |
| 36 | 0x90 | 0x51692 | MORGAN | boss |
| 38 | 0x98 | 0x5341C | MACK | |
| 40 | 0xA0 | 0x53E08 | MORGUE | boss |
| 55 | 0xDC | 0x5785C | FERRIS, GNEISS | alternate (scripted) thug |
| 61 | 0xF4 | 0x58498 | BUTCHER | boss |
| 63 | 0xFC | 0x5AA6E | BLADE (0x1C) | |
| 64 | 0x100 | 0x5AFC2 | PUNK (0x1D), WRENCH T. (0x24) | |
| 71 | 0x11C | 0x5C818 | SLISAUR | dinosaur boss |
| 72 | 0x120 | 0x5DD58 | TYROG (0x1B) | |
| 79 | 0x13C | 0x5F5D0 | HAMMER T./BLK ELMER/WRENCH T. (0x1E..0x20) | poacher variant (hunts dinosaurs) |
| 82 | 0x148 | 0x606D4 | ZEKE | |
| 84, 86..89 | 0x150.. | 0x6173E.. | FESSENDEN (0x22) and its parts | final boss |
| 91 | 0x16C | 0x654E2 | TYROG 2 | |
| 94 | 0x178 | 0x67D2C | FESSENDEN (0x25) | final boss, 2nd form |

### 3.3 HP [O]
- **Normal enemies**: `0x31424`. `HP = W[0x31454 + name_id*64 + level*2]`, where level = `$5EE(a5)` (0..31).
  It writes `+$6A = min(HP,100)` and `+$6C = +$6E = HP`. The full table is in `hp_table.txt`. Examples for level 0 → 31: FERRIS 44 → 76, GNEISS 44 → 100, DRIVER 52 → 100, PUNK 44 → 60, BLUDGE 76 → 200, WALTHER 100 → 300, ZEKE 20.
  Class 2 calls it from its variant init (`jmp 0x31424`).
- **Bosses**: `0x31E14(d0 = boss index)`. `HP = W[0x31E52 + idx*48 + (players-1)*16 + ((level & 0x1C) >> 1)]`.
  The table has 8 level bands per player count (`boss_hp.txt`). Example: Vice T. (idx 0) 1P 350..600, 2P 500..700, 3P 600..750.
  Indices: 0 Vice T./Slice/Slisaur/Tyrog(alt), 1 Butcher, 2 Hogg, 4 Morgan, 5 Morgue, 6 Tyrog, 8/9 Fessenden.
- The HUD draws HP above 100 as extra bars. The damage code splits HP into 100-point bars (section 10).

### 3.4 Kill score [O]
`0x325E6`: `off = B[0x325F8 + name_id]`. The score is the BCD long at `0x10292 + off`. It is added (BCD) by `0x10036` to the score of player `+$2` at player `+$100`. Nothing is added if `+$2 < 0`.

| name | score | name | score | name | score |
|---|---|---|---|---|---|
| FERRIS, R HOPPER, SHIVAT, PUNK, LASH T., MACK, KNIFE, DRIVER, 0x1C..0x23, HOGG | 1000 | GNEISS, POACHER J, HAMMER T., BLADE, THUG | 2000 | BLK ELMER, SKINNER, RAZOR | 3000 |
| WRENCH T. (0x0B), GUTTER | 4000 | WALTHER, BLUDGE, FESSENDEN (0x25) | 8000 | VICE T. | 10000 |
| BUTCHER, MORGAN | 20000 | MORGUE | 30000 | SLICE | 40000 |
| TYROG (0x19) | 60000 | SLISAUR | 70000 | WRENCH T. (0x24) | 80000 |
| TYROG (0x1B) | 100000 | | | | |

Only classes that call 0x325E6 award these. Shivat never does.

### 3.5 Enemy attacks against players [O]
- Attack table **0xA2EA4**: 8 bytes per attack id (`+$58` of the attacker). The bytes are `dmg, poise_dmg, reaction(+$6/+$7B of the victim), hitstop(+$59 both), sound, b5, spark(d5), +$9A`.
  The full list is in `damage_tables.txt`. Class 2 uses ids 0x00 (8 dmg), 0x01 (6), 0x02 (10), 0x03 jump (13, knockdown), 0x0C dash (12).
- Enemy-to-player damage is applied by 0x115B0 → 0x11652 and scaled by **0x6422** (section 6.6).
- Player attack data is in 0xA26FC (per character). See section 10.

---------------------------------------------------------------------------------------------------

## 4. Shared library routines (used by all enemy classes)

```rust
// 0x17A4 set_speed(id): select one of the 32-direction speed tables and re-apply the current direction
fn set_speed(e, id: i8) { e.spd = L[0x17CE + 4*id]; e.ax = 0; e.az = 0;
    let (vx, vz) = SPD(e.spd, e.dir); e.vx = vx; e.vz = vz; }           // [O]
// Speed tables: 0x80 bytes each (32 x (vx,vz) i16), pointers at 0x17CE. Magnitudes in 1/256 px/frame
// (speed_tables.txt): id 0..0x0D = 64*(id+1) in X (0.25 .. 3.5 px/f); 0x0E..0x1D fine steps 281..742;
// 0x1E 1280, 0x1F 1024, 0x20 1152, 0x21.. 1408+128k. Z component is about 0.476 * X (depth squash). [O]
// The port can load the tables from ROM (0x1227A8.., 0x80 bytes each) or recompute them. Recomputing
// must match the exact words, so loading is safer.

fn move_dir(e) { /* 0x1862 */ let (sx, sz) = SPD(e.spd, e.dir);
    e.vx = sx.wrapping_add(e.ax); e.x += (e.vx as i32) << 8;
    e.vz = sz.wrapping_add(e.az); e.z += (e.vz as i32) << 8; }          // [O]
fn physics(e) { /* 0x189C */ for (p, v, a) in [(x,vx,ax), (y,vy,ay), (z,vz,az)] { v += a; p += (v as i32) << 8 } } // [O]
// 0x18E0 = Y axis only.

// 0x103A angle_to(e, tx, tz) -> dir 0..31 (8 sectors x 5 sub-steps) [O]
fn angle_to(e, tx: u16, tz: u16) -> u8 {
    let mut oct = 0u8;
    let (mut dx, mut dz);
    if tx < e.x { dx = e.x - tx; oct = 4 } else { dx = tx - e.x }     // compares (v+0x4000) as u16
    if tz < e.z { dz = e.z - tz; oct += 2 } else { dz = tz - e.z }
    let (small, large);
    if dz < dx { small = dz; large = dx; if oct == 0 || oct == 4 { oct += 1 } }
    else       { small = dx; large = dz; if oct == 2 || oct == 6 { oct += 1 } }
    let l8 = large >> 3; let step = l8 * 2;           // u16 arithmetic
    let mut k = 0; let mut acc = l8;
    if acc < small { k = 1; acc += step;
        if acc < small { k = 2; acc += step; if acc < small { k = 3; acc += step; if acc < small { k = 4 } } } }
    ANGLE_10CC[(oct << 3) | k]   // rows: 0:[0,1,2,3,4] 1:[8,7,6,5,4] 2:[8,9,10,11,12] 3:[16,15,14,13,12]
}                                //       4:[0,31,30,29,28] 5:[24,25,26,27,28] 6:[24,23,22,21,20] 7:[16,17,18,19,20]

fn manhattan(e, o) -> u16 { abs(o.x - e.x) + abs(o.z - e.z) }          // 0x191E [O]

// Animation [O] (details belong to the animation spec):
//  0x120E set_anim(n): a0 = +40 + W[+40 + 2n]; load frame.  0x121E step_anim(): --frame_timer(+2F); when 0
//  advance 10 bytes (loop if byte+7 bit7 of the previous record) and load; returns carry=1 when the
//  animation wrapped/ended (used as "animation finished"). 0x1380 step_anim_reverse().
//  Frame record (10 bytes): w comp, w comp2, b +2C, b +2D, b +2E, b +2F(duration, bit7 = last), w +48.

fn walk_anim(e) { if (e.dir >> 4) != e.facing { step_anim(e) } else { step_anim_reverse(e) } }  // 0x324D4 [O]

fn stuck_check(e) -> bool {                                              // 0x324EE [O]
    e.t86 = e.t86.wrapping_sub(1); if e.t86 != 0 { return false }
    e.t86 = 30;
    let stuck = in_win(e.x - e.x8c, 8) && in_win(e.z - e.z8e, 8);       // moved < 8 px in 30 frames
    e.x8c = e.x; e.z8e = e.z; stuck
}

fn clamp_to_playfield(e) {                                               // 0x193E, after every alive frame [O]
    if car_mode_518 != 0 { return despawn_if_far(e) }                    // 0x173C
    if vertical_4dc != 0 { /* X clamped to [0x10,0x170], Z to [?,0x140] */ return }
    let d = e.x - camX + 0x50; if d < 0 { e.x -= d } else if d - 0x220 >= 0 { e.x -= d - 0x220 } // X in [camX-0x50, camX+0x1D0]
    let d = e.z + e.w56 + e.w54 - camY; if d < 0 { e.z -= d } else if d - 0x100 >= 0 { e.z -= d - 0x100 }
}
fn despawn_if_far(e) { if !in_win_u(e.x - camX + 0xC0, 0x300) { e.r4 = 6 } }   // 0x173C: free next frame [O]

fn register_draw(e) { if near_screen(e) /*0x16FA: X-camX in [-0x100,0x280), Y+Z-camY in [-0x40,0x180)*/
                        { push(draw_list, (e, e.z)) } }                   // 0x49CA [O]
fn register_hurtable(e) { push(list_6726, e) }                           // 0x4BEA: enemies that players can hit [O]
fn snap_to_floor(e) { floor_collision(e); e.y = e.w54 + e.w56; e.air = 0 }   // 0x3140C [O]
// 0x12CB4 floor_collision: probes the stage collision map (stage spec), sets +50 flags, +54/+56, and
// lands (+51=0, Y=ground) or starts falling (+51=1). [O] Ported as `game/terrain.rs`; the map is the
// terrain class in bits 10..15 of the scroll2 tile attributes (see the module doc there).
```

---------------------------------------------------------------------------------------------------

## 5. Enemy census and target selection

### 5.1 Census `0x3242E` (first thing in `objects_update`) [O]
```rust
fn enemy_census() {
    for p in players { p.prev_e0 = p.cnt_e4 /* 4 bytes e4..e7 -> e0..e3 */; p.cnt_e4 = [0;4] }
    alive_4f3 = alive_4fb;  alive_4fb = 0;                        // $4F3 = LAST frame's count
    for e in poolA_slots(0..24) {
        if e.active == 0 || (e.b7d as i8) < 0 || e.r4 >= 4 { continue }
        alive_4fb += 1;
        if e.r4 == 0 { continue }                                 // still initialising
        let p = match e.target_76 { P3 => P3, P2 => P2, _ => P1 };
        p.e4 += 1;                                                // enemies targeting p
        if e.b7d != 0 { p.e5 += 1 }                               // ... of which in attack state (never read, see 14)
        if p.x >= e.x { p.e6 += 1 } else { p.e7 += 1 }           // enemy left / right of p (unsigned compare)
    }
}
```

### 5.2 Target selection `0x32032` → returns player object [O]
```rust
fn pick_target(e) -> Player {
    score = [0u8; 3];
    let present = [P1.active, P2.active, P3.active];
    if none present { return match setting_4cd { 0 => P1, b if b & 1 != 0 => P1, b if b & 2 != 0 => P2, _ => P3 } }
    // 1) spread: the present player with the fewest enemies targeting him last frame (ties -> later player)
    let (mut best, mut m) = (P1?, if P1 present {P1.e0} else {0x10});
    if P2 present && P2.e0 <= m { best = P2; m = P2.e0 }
    if P3 present && P3.e0 <= m { best = P3; m = P3.e0 }
    if m < 3 { score[best] = 1; goto choose }
    // 2) all players already have >= 3 attackers: weighted score
    nearest (Manhattan, ties -> earlier player)            : score = 10
    largest score +$100 (BCD long, ties -> earlier)        : += 1
    largest vitality +$6C (ties -> earlier)               : += 2
    for each present p {
        if p.w_a0 != 0 && (p.w_a0 & 0x0F00) == 0x0200 { score += if (4..10).contains(&(p.w_a0 as u8)) {6} else {3} }
        if e.facing == 0 { score += 4 }      // QUIRK: intended "enemy faces p"; both branches test facing==0
        if p.w4 == 0x0200 && p.b6 == 2 { score += 3 }        // p is walking
    }
  choose:
    // max score, ties -> earlier player (P1 > P2 > P3)
}
```
The "fewest attackers" rule is how the game spreads enemies over several players. **[O]**

---------------------------------------------------------------------------------------------------

## 6. Difficulty: rank, level, aggression, damage

There are two hidden variables. **Rank** `$5F0(a5)` (0..31) changes slowly. **Level** `$5EE(a5)` (0..31) is derived from rank and the player count. Most tables are indexed by level.
**Aggression** `$5EC(a5)` (0..7) is a mood value. It is re-rolled at events and can be forced by stage scripts. The struct at `$5EC(a5)`:

| addr | rank-struct offset | meaning |
|---|---|---|
| $5EC | +0 | aggression 0..7 |
| $5EE | +2 | level 0..31 (from table 0x1860C) |
| $5F0 | +4 | rank 0..31 |
| $5F2 | +6 | forced-aggression camera X limit (script command 8) |
| $5F4 | +8 | forced aggression value |
| $5F6 | +A | rank speed index 0..15 |
| $5F8 | +C | rank-up countdown |
| $5FA/$5FE | +E/+12 | pending rank-ups / timer 1800 |
| $5FC/$600 | +10/+14 | pending rank-downs / timer 1200 |
| $604..$608 | +18..+1C | misc timers; $608 = 1200-frame calm period after a continue |
| $602 | | rank system freeze (nonzero = off) |

`$4EF(a5)` = number of players - 1 (0..2). `$777B(a5)` = difficulty setting byte (copied from `$776E` at 0xA86DE). Bits 0-2 are the difficulty 0..7 and bits 3-4 set the initial rank speed. **[O]** (The exact DIP or test-menu source is [I].)

### 6.1 Game start `0x17F8A` [O]
```rust
band = if w4f6 < 720 {0} else if w4f6 < 3600 {0x10} else if w4f6 < 10800 {0x20} else {0x30};  // 0x180B4
off = W[0x180BA + band + (dip & 7)*2];                     // see rank_tables.txt
rank_5f0 = B[0x180FA + off + (rng() & 0x1F)] - 1;          // rows give 0..20
w4f6 = 0;  rank_speed_5f6 = ((dip & 0x18) >> 3) + 1;
area_start();                                              // 0x18048
```
- `$4F6` counts frames while not in a game but with `$46A` nonzero (0x1827A, called by the system task). [I: time spent at the title with credits inserted]
- A shorter wait and a higher difficulty setting give a higher initial rank.

### 6.2 Stage and area start [O]
```rust
fn stage_start() {   // 0x1800E: per-stage rank adjust, table 0x1823A (delta, max, min)
    // stage 1 (0, 31, 0); stage 2 (+3, 23, 14); stage 3 (-2, 24, 16); stage 4 (+5, 25, 18); stage 5 (+1, 26, 20);
    // stage 6 (+5, 27, 22); stage 7 (+4, 28, 24); stage 8 (+6, 31, 26)
    let mut r = rank + delta; if r < 0 { r = 0 } if r > max { r = max } if r < min { r = min } rank = r;
    area_start() }
fn area_start() {    // 0x18048
    w5f2 = w5f4 = w602.. = 0 /* +6 +8 +16 +18 +1A +1C */;
    rank_timer_5f8 = W[0x187B8 + 2*rank_speed];             // 600,520,480,440,400,360,300,260,240,220,200,180,160,140,120,60
    aggression_5ec = B[0x18094 + stage*4 + area] - 1;       // stage 1 areas 0..2: 0,0,5
    compute_level(); }
```

### 6.3 Per-frame `rank_tick` `0x186CC` and `compute_level` `0x18592` [O]
```rust
fn rank_tick() {
    if w602 != 0 || area_transition_4dd != 0 || b4d5 != 0 { return }
    if camX < w5f2 { aggression_5ec = w5f4 } else { w5f2 = 0 }       // script-forced aggression zone
    rank_timer_5f8 -= 1; if rank_timer_5f8 < 0 /* was 0 */ { rank = min(rank+1, 31); rank_timer_5f8 = W[0x187B8 + 2*rank_speed] }
    if w5fa != 0 { w5fe -= 1; if w5fe == 0 { w5fa -= 1; w5fe = 1800; rank = min(rank+1, 31) } }
    if w5fc != 0 { w600 -= 1; if w600 == 0 { w5fc -= 1; w600 = 1200; rank = max(rank-1, 0) } }
    if w604 != 0 { w604 -= 1 }  if w606 != 0 { w606 -= 1 }
    compute_level();
    if calm_608 != 0 { aggression_5ec = 0; calm_608 -= 1; if calm_608 == 0 { reroll_aggression() } }
}
fn compute_level() {
    let mut r = rank;
    match players_4ef { 0 => r = r.min(24), 1 => r = r.clamp(4, 28), _ => if r < 8 { r = 4 } }
    rank = r;
    level_5ee = W[0x1860C + 2*(r*3 + players_4ef)];   // rank_tables.txt; e.g. rank 0 -> 0/4/8, rank 31 -> 25/29/31
}
```
So rank goes up by 1 every 61..601 frames, depending on the rank speed `$5F6`. **[O]**

### 6.4 Rank events [O; caller meaning I]
- `0x184E2` (from 0x1BFF0 / 0x1C020, the player continue/join path):
  - `rank -= (w606 != 0 ? 3 : B7[0x187E0 + dip])`, where the table is 7,6,5,4,3,2,1,0. Floor at 0. Then `w606 = 600`.
  - Call `0x1843C`, then `compute_level()`, then `calm_608 = 1200`.
  - Finally, if the player's `+$A3` is nonzero: `rank_speed += W[0x187F4 + dip*8 + 2*idx]`, where idx = `$4EF` (+ rng&1 when 3 players). The values are -7..0, clamped to 0..15, and `rank_timer = 0`.
- `0x1843C` (extra life at 0x6BDC, and the continue path above):
  - `s = sum over present players of +$A3`. `k = B[0x184C4 + 2*s + (b4cf == 0)] - b4e6`.
  - If the table byte is 0: rank -= 3. If k == 2: rank += 2. If k >= 3: rank += 4. Clamp to 0..31.
- `0x18410` (player event when `$4D0 == 0xC`): rank_speed += 3. `0x1842C`: rank_speed -= 1. Boss kill (0xACEE): rank_speed += 1.
  All these go through `0x18566`: clamp to 0..15, then `rank_timer_5f8 = 0`.

### 6.5 Aggression re-roll `0x182A2` (Markov chain) [O]
```rust
fn reroll_aggression() {
    if w602 != 0 { return }
    aggression_5ec = if w5f2 != 0 { w5f4 }                        // script-forced
        else { let t = if b7b4 == 1 { 0x18364 } else { 0x182E4 };  // two 8x16 tables
               B[t + aggression_5ec*16 + (rng() & 0xF)] };          // mostly +1, sometimes -1/-2 (rank_tables.txt)
}
```
Callers:
- spawn command 4
- end of the calm period (0x187B2)
- every enemy death (0x2AFCA, `getup_or_die`)
- 0x4A26E

So aggression drifts upward as the players kill enemies, and resets to 0 for 20 s after a continue. **[O]**

### 6.6 Enemy-to-player damage `0x6422` [O]
`d1` = base damage from the attack record (0xA2EA4). a2 = enemy attacker, a3 = player.
```rust
fn enemy_damage(base: u16, atk: &Obj, p: &Player) -> u16 {
    if test_4aa != 0 && (io_24 & 2) != 0 { return 0 }                         // debug no-damage
    let mut d = (base * B[0x2C3AE + atk.w84*8 + (level & 31)] as u16) >> 5;  // per-type x level, 32 = 1.0
    let band = if p.hp >= 0x4C {0} else if p.hp >= 0x33 {0x40} else if p.hp >= 0x1A {0x80} else {0xC0};
    d = d as i16 - W[0x2C896 + band + (rng() & 0x3E)] as i16;                // small random reduction when hurt
    if d < 0 { d = 1 }                                                        // (0 stays 0)
    d = (d * W[0x2C88E + 2*p.char_3]) >> 5;                                  // char factor: Jack 32, Mustapha 34, Hannah 35, Mess 30
    d = (d * W[0x2C996 + ((p.hp >> 2) & 0x1E)]) >> 5; if d == 0 { d = 1 }     // low-HP factor 25..33
    if p.hp != 1 && d >= p.hp && d - p.hp < 5 { d = p.hp - 1 }               // "mercy": survive at 1 HP
    d
}
```
- Row `+$84*8` = name_id*32 bytes. Example: FERRIS 32 at level 0 up to 58 at level 31, so damage rises by up to 1.8x.
- All multiplies are 16x16 `mulu`, then a 16-bit `lsr #5`. **[O]**

### 6.7 Where level, aggression and player count change behaviour (summary)
- **Level `$5EE`**: HP (3.3), damage scale (6.6), class 2 pause at slot (`T_34186`), recovery between hits (`T_345F0`), dash-attack chance (`T_35796`), counter-attack delay after a flinch (0x2B8C4), get-up time and dizzy chance (section 10), level-gated spawns (flag bits 3-4).
- **Aggression `$5EC`**: class 2 walk/run speed, attack chance at slot (`T_353B6`), ambush roll (`T_33FA4`), state-E pause (`T_34210`), and the dizzy row.
- **Player count `$4EF`**: level column, spawn conditions (flag bits 5-6), wave thresholds (per 1P/2P/3P), boss HP, reinforcement limits.

---------------------------------------------------------------------------------------------------

## 7. Worked example: basic thugs FERRIS / GNEISS / DRIVER (pool A class 2, handler 0x3383A)
These are the first enemies of stage 1. Every behaviour below is **[O]** unless tagged. All tables are decoded in `class2_tables.txt`.
- `var = +$26` (word from the spawn record): high byte = **variant** (0 Ferris, 2 Gneiss, 4 Driver), low byte = **entrance** (0, 2, 4, 6, 8, 0xA).
- `aggr` = `$5EC & 7` (some tables use `$5EC` without the mask; it is always 0..7). `level` = `$5EE`.
- `vrow` = variant byte (0/2/4). Rows are `vrow*16` bytes for 32-byte tables, `vrow*4` for 8-byte tables, `vrow*8` for 8-word tables.

### 7.1 Debug state names [O]
The debug overlay (0x32F70..0x32FD2) prints `+$06` through the offset table **0x3300A**. That table is indexed by the byte value of +$06 (0, 2, 4…), so name index = +$06 / 2.
- +$06 = 2 NORMAL, 4 ARUKI (walk), 6 X NOMI, 8 Z NOMI, 0xA GOURIKI, 0xC 32 HOUKOU, 0xE DUSH, 0x10 RIFLE, 0x12 JUMP, 0x14 SUKI, 0x16 ZETTAI K.S, 0x18 NOTHING, 0x1A MAWARIKOMI (circle round), 0x1C R-TUKI, 0x1E R-TUKI2, 0x20 TIMERAG, 0x22 MAAI (keep distance), 0x24 KOUGEKI (attack), 0x26 RSOBAT, 0x28 BOUGYO (guard), 0x2A K-TUIJYU, 0x2C K-SUKI, 0x30 HAMARI, 0x32 ZETTAI (must attack), 0x34..0x3E ST SUWARI/ST OKU/ST TOBI/ST JUMP/R-SUKI/ST DUSH, 0x40 OUEN (cheer), 0x42 RSOBAT, 0x44 R-TUKI, 0x46 BOO, 0x48 ITEM K., 0x4A/0x4C FUKKI (recover).
- The same overlay prints `+$A0` through the two-entry table **0x32FD4**: 0 = TAIKI (wait), 1 = KOUGEKI (attack). It also shows the words +$A6 and +$AC, and the sign of +$B8.
- More unused names follow at 0x3307C..0x33328: TABLE, SAGURI (probe), JUMPC, MECHA, FURIKAERI, HIROI (pick up), H ARUKI, MACHI, JP TAME, NAGE, OKU DASHI, MAE DASHI, W DASHI, ST TACHI, DUSH TUKI, H. MODOSHI, HIP DROP, S. TUKI, S.T. TAME, PILE DRIV, TUKAMI, HENSHIN, POISUTE, DARUMA.

These names belong to the developers' generic AI vocabulary. Each class uses its own subset, with its own numbering. **The +$06 values of class 2 do not follow this list** (see 7.4 for the real meanings). The list is useful only as a vocabulary. [I]

### 7.2 Top level
```rust
fn class2(e) { match e.r4 { 0 => init(e), 2 => alive(e), 4 => dying(e) /*section 10*/, 6 => free_poolA(e) /*0x48FC*/, _ => {} } }

fn init(e) { match e.r5 {                                    // 0x3384E, table 0x3385C
  0 => { e.r5 = 2; f_939b6(e) /*[U] effect/sound*/;
         match e.var >> 8 {                                    // 0x3387E (byte index *2 -> long)
           0 => { e.name=0x00; e.w84=0x00; e.anim_tab=0x34892 }  // FERRIS
           2 => { e.name=0x01; e.w84=0x04; e.anim_tab=0x34B90 }  // GNEISS
           4 => { e.name=0x1A; e.w84=0x68; e.anim_tab=0x34E7C }  // DRIVER
         }
         set_anim(e,0); e.box_tab=0x00103000; e.b4a=0; set_walk_speed(e); set_hp(e) /*0x31424*/;
         e.poise=10; e.b79=0; e.b7a=0; e.b7b=0; e.b7d=0; e.b87=0; e.b63=0; e.b72=180; e.ba4=0; e.wa2=0;
         e.ba5=0xFF; e.ba9=20; e.bac=90; e.bba=0; e.bbb=0; e.bbc=0;
         e.target = pick_target(e); snap_to_floor(e); e.b7d=0; }
  2 => { kill_all_check(e) /*0x326F8, section 10*/; ENTRANCE[e.var & 0xFF](e); register_draw(e); register_hurtable(e) }
  4 => { retarget_and_slot(e) /*0x35172*/; reset_ai(e); register_draw(e) }   // 0x33E66: entrance finished
}}
fn set_walk_speed(e) { set_speed(e, W[0x35832 + vrow*8 + aggr*2]) }   // 0x35812 (ids in class2_tables.txt)
fn set_run_speed(e)  { set_speed(e, W[0x35882 + vrow*8 + aggr*2]) }   // 0x35862
```

### 7.3 Entrances (`init` sub 2; the entrance code sets `e.r5 += 2` when done) [O]
```rust
fn entr0_walk_in(e) { match e.r6 {                          // 0x339B8
  0 => { e.wb4 = e.x; set_anim(e,1); set_walk_speed(e); face_screen_centre(e); e.r6 = 2 }
  _ => { if !player_ahead(e, 0x18, 0x18, 0x30) && in_win(e.x - e.wb4, 0x80) { } else { e.r5 += 2 }
         move_dir(e); step_anim(e) } } }               // stops after 128 px or when a player is just ahead
fn face_screen_centre(e) {                                   // shared by entrances 0 and 2
    if camX + 0xC0 - e.wb4 >= 0 { e.dir = 8; e.facing = 1 } else { e.dir = 0x18; e.facing = 0 } }
fn player_ahead(e, ahead, half_w, w) -> bool {               // 0x33A28 / 0x33AC8 -> 0x33B96, players P1,P2,P3
    let px = e.x + if e.facing != 0 { ahead } else { -ahead };
    any present player p: in_win_u(px - p.x + half_w, w) && in_win_u(e.z - p.z + 0xC, 0x18) }
fn entr2_dash_in(e) { match e.r6 {                          // 0x33A42
  0 => { e.wb4 = e.x; set_anim(e,6); set_speed(e,0x1F) /*4 px/f*/; face_screen_centre(e); e.r6 = 2 }
  _ => { let moved64 = !in_win(e.x - e.wb4, 0x40);
         if (moved64 && player_ahead(e, 0x20, 0x20, 0x40)) || !in_win(e.x - e.wb4, 0x80) { e.r5 += 2 }
         move_dir(e); step_anim(e) } } }
fn entr4_wait(e) { match e.r6 {                             // 0x33AE2: stand or sit until a player is near
  0 => { e.r6 = 2; e.wb6 = W[0x33B28 + (rng() & 0x3E)];      // 352..832 frames
         if (e.b5b as i8) < 0 { set_anim(e,0x0A) } else { e.facing = e.b5b; set_anim(e,0x0B) } }
  2 => { if in_win_u(camX + 0x180 - e.x, 0x180) { e.r6 = 4 }   // became visible
         player_within(e, 0x50) }
  _ => { e.wb6 -= 1; if e.wb6 < 0 { e.r5 += 2; return } player_within(e, 0x50) } } }
fn player_within(e, d) { if any present p: manhattan(e,p) < d { e.r5 += 2 } }   // 0x33BE8 (checks P1,P2,P3)
fn entr6_fade_in(e) { match e.r6 {                          // 0x33C32: emerge from a dark doorway
  0 => { e.r6 = 2; e.pal = 0x9D; e.z += 0x10; e.vz = -0x40; e.bba = 20; fade_step(e); set_anim(e,0x0D) }
  _ => { e.bba -= 1; if e.bba == 0 { if e.bbb == 0x0C { e.r5 += 2; e.pal = 0; return }
                                      e.bba = 20; e.bbb += 4; fade_step(e) }
         physics(e); step_anim(e) } } }
// fade_step 0x33C9E: copies 16 colours into OBJ palette 0x1D (0x9143A0), keeping the brightness nibble of each.
// Source: L[0x33CDA + vrow*8 + e.bbb] (4 steps per variant). Total: 4 steps x 20 frames.
fn entr8_jump_in(e) { match e.r6 {                          // 0x33D0A: leap in (from a vehicle or ledge)
  0 => { e.r6 = 2; floor_collision(e); e.y += 0x20 << 16; e.air = 1; e.b7b = 4; e.poise = 10;
         e.vx = if e.facing != 0 { 0x600 } else { -0x600 }; e.ax = 0; e.vz = 0; e.az = 0;
         e.vy = 0x300; e.ay = -0x50; set_anim(e,0x0E) }
  2 => { f_4b58(e) /*[U] water splash*/; physics(e); if e.b2c == 0 { step_anim(e); if e.b2d != 0 { e.facing = (e.facing + e.b2d) & 1; e.b2d = 0 } }
         floor_collision(e); if e.air == 0 { e.air = 1; e.vy = -e.vy; e.ay *= 2; e.r6 = 4; land_fx(e) /*0x9796*/; step_anim(e) } }
  4 => { f_4b58(e); physics(e); if e.b2c == 0 { step_anim(e) } floor_collision(e);
         if e.air == 0 { e.b80 = if e.hp != 0 { 20 } else { 10 }; land_fx(e); e.r6 = 6 } }
  6 => { e.b80 -= 1; if e.b80 < 0 { e.r6 = 8; set_anim(e,0x0F) } else { step_anim(e) } }
  _ => { if step_anim(e) /*ended*/ { e.r5 += 2 } } } }
fn entrA_drop(e) { floor_collision(e); e.y += 0x100 << 16; e.air = 1; e.facing = e.b5b; e.r5 += 2 }  // 0x33E48: falls from 256 px
```

### 7.4 Alive frame and AI states
```rust
fn alive(e) {                                                // 0x33E76
    kill_all_check(e);
    match e.r5 { 0 => ai(e), 2 => hit_reaction(e) /*0x2A606*/, 6 => grabbed(e) /*0x2B35A*/, _ => {} }
    clamp_to_playfield(e);                                    // 0x193E
}
fn ai(e) {                                                   // 0x33EA4
    if e.b79 != 0 { e.b79 -= 1; if e.b79 == 0 { e.poise = 10 } }
    STATE[e.r6](e);                                          // 0x33EFA: 0,2,4,6,8,A,C,E,10
    floor_collision(e);
    e.bbc = e.bbc.wrapping_sub(1); if e.bbc == 0 { e.bbc = 200; if e.b50 & 0x80 != 0 { attack_obstacle(e) } }
    if e.b50 == 0x10 { e.l4 = 0x0200_1000 }                  // hop down a ledge (state 0x10)
    register_draw(e); register_hurtable(e);
}
fn reset_ai(e) {                                             // 0x33F38 "back to state 0"
    e.l4 = 0x0200_0000; set_anim(e,1); if e.b7a == 0 { e.b7a = 2 } e.ba4 = 0; set_walk_speed(e);
    e.b7d = 0; retarget_and_slot(e); steer_to_slot(e);
    /* then the tail of ai(): +50 checks, register_draw, register_hurtable */ }
fn attack_obstacle(e) { e.l4 = 0x0200_0806; e.ba8 = 7; e.waa = 0x2A;          // 0x33F0C: combo 7 = single C
    e.facing = (obj(e.w94).x - e.x > 0) as u8 }
```

**State 0 "decide" (0x33F74)**
```rust
if e.air != 0 { return goto_fall(e) }                        // 0x3474E: e.l4 = 0x0200_0C00
if e.b7a != 0 { e.b7a -= 1; if e.b7a == 0 && (rng() & 0xF) < B[0x33FA4 + aggr] /*3,4,5,6,7,8,11,15*/ { return choose_combo(e) } return }
e.w6 = 0x0200; state2(e)                                     // falls through into NORMAL this frame
```

**State 2 "approach" (0x33FB8; NORMAL / MAWARIKOMI behaviour)**
```rust
let t = e.target;
if t.active == 0 || t.r5 == 2 { retarget_to_any(e) /*0x35152: buggy movea.l; overwritten next*/; return reset_ai(e) }
if bump_player(e) { return wait_then_attack(e) }             // 0x3555A -> state E
if dash_trigger(e) { e.w6 = 0x0A00; return }                 // 0x355C0 -> state A
if e.r7 == 0 { e.l4 = 0x0200_0202; e.b7d = 0; e.wb2 = 300; set_anim(e,1); e.wa2 = 0x36; e.b80 = 1; e.wa0 = 6; e.ba6 = 0; e.ba7 = 0 }
if e.air != 0 { return goto_fall(e) }
if near_slot(e) { return at_slot(e) }                        // 0x3535C -> 0x353CE
e.wa0 -= 1; if e.wa0 < 0 { e.wa0 = 0x28; if e.ba7 == 0 { steer_to_slot(e) } }      // re-path every 41 frames
if e.ba4 == 0 { e.wa2 -= 1; if e.wa2 == 0 { e.wa2 = 0x24;
    if (0x8A88_6551u32 >> (rng() & 0x1F)) & 1 != 0 { e.ba4 = 1; e.w6 = 0x0400; return } } }   // taunt, 12/32
e.ba6 = e.ba6.wrapping_sub(1);
if e.ba6 == 0xFF /*was 0*/ { e.ba6 = 0; e.ba7 = 0; if near_slot(e) { if e.b2d == 0 { step_anim(e) } return } }
if stuck_check(e) { rotate_slot(e) }                         // 0x357F6
move_dir(e); walk_anim(e);
```

**State 4 "taunt" (0x340AE)**
```rust
if e.r7 == 0 { set_anim(e,2); e.r7 = 2;
    if obj(e.w68).char_3 == 2 && rng() & 0x3F == 0 && rom_check_2a5b6_failed() { poolA_freelist_6602 += 8 } } // anti-tamper (section 15)
if step_anim(e) { e.w6 = 0x0200; set_anim(e,1) }
```

**State 6 "keep distance / circle" (0x340FC; MAAI)**. +$07 is never set here, so the "first frame" block runs every frame.
```rust
e.b7d = 0; set_walk_speed(e); e.wa0 = 0;
if e.air != 0 { return goto_fall(e) }
e.b80 = e.b80.wrapping_sub(1); if e.b80 == 0 && near_slot(e) { return at_slot(e) }
e.bb8 = e.bb8.wrapping_sub(1); if e.bb8 != 0xFF { return }   // pausing at the slot
e.bb8 = 0;
e.wa0 -= 1; if e.wa0 < 0 { e.wa0 = B[0x34186 + aggr]; steer_to_slot(e) }    // always taken (wa0 was 0)
if near_slot(e) { e.bb8 = B[0x34186 + (level & 31)] /*85..15*/; if e.b2d == 0 { step_anim(e) } return }
if stuck_check(e) { rotate_slot(e) }
move_dir(e); walk_anim(e);
```

**At slot: attack or reposition (0x353CE)**
```rust
fn at_slot(e) {
    if B[0x353B6 + vrow*4 + aggr] > (rng() & 0x1F) as u8 { e.w6 = 0x0800; return }    // go attack
    e.b7d = 0;
    let mut c; let mut tries = 5;
    loop { c = B[0x354B2 + vrow*16 + (rng() & 0x1F)]; tries -= 1; if c != e.ba5 || tries == 0 { break } }
    e.ba5 = c; e.b80 = B[0x3546A + (rng() & 0xF)];          // 10..240 frames
    match c {
      0 => { if frame_counter_lo_8001 & 1 != 0 { e.b7c += 1; if e.b7c > 5 { e.b7c = 0 } }
             else { e.b7c = e.b7c.wrapping_sub(1); if (e.b7c as i8) < 0 { e.b7c = 5 } }
             steer_to_slot(e) }                               // next/previous slot round the target (circling)
      1 => { e.w6 = 0x0600 }                                  // MAAI: hover at the slot for b80 frames
      _ => { let d = angle_to(e, e.target.x, e.target.z); e.dir = (d + 0x10) & 0x1F;   // back off
             e.ba6 = 50; e.ba7 = 1 }                          // walk straight away for 50 frames (in state 2)
    }
}
```

**Slots around the target**
```rust
fn retarget_and_slot(e) {                                    // 0x35172
    let t = pick_target(e); e.target = t;
    let l = t.prev_e2 as i8 - 1; let r = t.prev_e3 as i8 - 1;      // last frame: enemies left / right of t
    let mut s: u8 = if l > r { 3 } else if l < r { r as u8 } else if e.x < t.x { r as u8 } else { 3 };
    // QUIRK [O]: in the "<" case and the "equal, enemy on the left" case the code keeps d1 = r
    // (= prev_e3 - 1; it was loaded into the same register) instead of 0, so s can be 0..N, or even
    // 0xFF when nobody was on either side last frame. steer_to_slot then indexes SLOT_3524C with
    // s*4 (u8 -> word). The port must read those ROM bytes the same way (s >= 6 and s = 0xFF happen).
    let mut dz = 0x13i16;
    if e.z < t.z { s += 1; dz = -0x13 }
    if t.z + dz >= e.z { s += 1 }
    e.b7c = s;
}
fn steer_to_slot(e) {                                        // 0x351C4
    let t = e.target; let mut s = e.b7c;
    if t.facing != 0 { s += 3; if s >= 6 { s -= 6 } }
    let (mut dx, dz) = SLOT_3524C[s];   // (-96,21) (-120,4) (-96,-24) (88,-24) (112,4) (88,21);
                                        // s>=6 reads the following ROM words (raw values in class2_tables.txt)
    if t.facing != 0 { dx = -dx }
    e.x88 = t.x + dx; e.z8a = t.z + dz;
    let d = angle_to(e, e.x88, e.z8a);
    let diff = (e.dir.wrapping_sub(d).wrapping_add(4)) & 0x1F;
    e.dir = if diff <= 8 { d & 0x1F } else { (e.dir + if diff >= 0x14 { 4 } else { -4i8 as u8 }) & 0x1F };  // turn max 45 deg per call
    e.facing = (t.x >= e.x) as u8;                           // (t.x - e.x) not in [-0x400,-1]
}
fn near_slot(e) -> bool { in_win(e.x - e.x88, 0x18) && in_win(e.z - e.z8a, 0xF) }     // 0x3535C: |dx|<24, |dz|<15
fn rotate_slot(e) { e.b7c += 2; if (e.b7c as i8) >= 6 { e.b7c -= 6 } e.dir ^= 0x10 }    // 0x357F6, when stuck
```

**Bump into a player → short pause → attack (0x3555A, state E 0x34204)**
```rust
fn bump_player(e) -> bool { for p in [P3, P2, P1] {                // order of table 0x35574
    if p.active != 0 && p.r5 == 0 && in_win(p.x - e.x, 0x18) && in_win_u(p.z - e.z + 2, 5) { e.target = p; return true } } false }
fn wait_then_attack(e) { e.w6 = 0x0E00; set_anim(e,0); e.b80 = B[0x34210 + aggr] /*28..3*/ }
// state E: e.b80 -= 1; if e.b80 < 0 { e.l4 = 0x0200_0800 }
```

**State 8 "attack" (0x34222).** Sub-state +$07 is taken from table 0x34258.
```rust
// pre-handler (every frame in state 8): hit-stop
if e.b59 != 0 { e.b59 -= 1; if e.b59 != 0 { return } e.b2f = 1 }   // freeze; then force the next anim step
if e.r7 != 0x0E && e.air != 0 { return goto_fall(e) }
match e.r7 {
  0 => { e.w6 = 0x0802; e.b7d = 1; set_anim(e,1); e.wa0 = 0; choose_side(e); e.b80 = 0xF0; set_run_speed(e); /* falls into 2 */ }
  2 => { e.b80 -= 1; if e.b80 < 0 { return reset_ai(e) }               // give up after 240 frames
         e.wa0 -= 1; if e.wa0 < 0 { e.wa0 = 6; steer_to_attack_pos(e) }
         if at_attack_pos(e) { if e.b2d == 0 { walk_anim(e) } e.r7 = 4; return choose_combo(e) }
         if stuck_check(e) { rotate_slot(e) } move_dir(e); walk_anim(e) }
  4 => choose_combo(e), 6 => combo_step(e),
  8 | 0xA | 0xC => { if step_anim(e) { recovery(e) } }                  // punch A/B/C running
  0xE => jump_attack_run(e), 0x10 => { e.b80 -= 1; if e.b80 < 0 { combo_step(e) } }
}
fn choose_side(e) { let base = if e.x < e.target.x { 0 } else { 3 }; e.b7c = base + W[0x3533C + 2*(rng() & 0xF)] }   // 0x35310
fn steer_to_attack_pos(e) {                                   // 0x35296: same mirroring as steer_to_slot,
    // ATK_352F8 = (-55,2) (-55,0) (-55,-2) (55,2) (55,0) (55,-2); dir = angle_to(point) with no turn limit; face target
}
fn at_attack_pos(e) -> bool { in_win(e.x - e.x88, 4) && in_win(e.z - e.z8a, 5) }    // 0x3538A
fn choose_combo(e) {                                          // 0x342D8
    if e.b50 & 0x80 != 0 { retarget_to_any(e); return reset_ai(e) }
    if e.facing == e.target.facing && (rng() & 0x1F) >= 7 { return reset_ai(e) }   // target shows its back: attack only 7/32
    // (both facing the same way = the enemy is behind the target)
    let c = B[0x34338 + vrow*16 + (rng() & 0x1F)];
    e.ba8 = c; e.waa = c as u16 * 6; e.r7 = 6; combo_step(e)
}
fn combo_step(e) {                                            // 0x34398
    let t = e.target;
    if t.active == 0 || t.r5 == 2 || t.r5 == 6 { return reset_ai(e) }
    if e.b50 & 0x80 == 0 && !in_attack_range(e) { e.w6 = 0; return }   // back to state 0
    let a = B[0x343E8 + e.waa]; if (a as i8) < 0 { return choose_combo(e) }   // combo finished -> maybe another
    e.waa += 1;
    match a { 0 => act(e, 0x0808, 3, 0), 2 => act(e, 0x080A, 4, 1), 4 => act(e, 0x080C, 5, 2), _ => jump_attack(e) }
}
fn in_attack_range(e) -> bool { let t = e.target;            // 0x35512: 0x34 px in front of t on e's side
    let px = t.x + if e.x >= t.x { 0x34 } else { -0x34 };
    in_win(e.x - px, 0x14) && in_win(e.z - t.z, 5) }
fn act(e, st: u16, anim: u8, atk: u8) {                       // 0x34420 / 0x34466 / 0x344AC
    e.w6 = st; set_anim(e, anim); e.atk_58 = atk; e.b5a = 0; e.b59 = 0;
    e.serial_5e = attack_serial_67be; attack_serial_67be += 1;
    if (B[0x343E8 + e.waa] as i8) < 0 { e.b5a = 1 }           // last action of the combo
    if step_anim(e) { recovery(e) } }
fn jump_attack(e) {                                           // 0x344F2, atk 3
    e.w6 = 0x080E; e.b90 = 0; set_anim(e,8); e.atk_58 = 3; e.b5a = 0; e.b59 = 0; e.serial_5e = next_serial();
    let right = e.target.x >= e.x;
    e.vx = if right { 0xA0 } else { -0xA0 }; e.facing = right as u8;
    e.vy = 0x480; e.ay = -0x30; e.ax = 0; e.vz = 0; e.az = 0; e.air = 1; jump_attack_run(e) }
fn jump_attack_run(e) {                                       // 0x34560
    if e.b90 == 0 { step_anim(e); if e.b2c == 0 { physics(e); if e.air == 0 { e.b90 += 2; set_anim(e,9) } } }
    else if step_anim(e) { recovery(e) } }
fn recovery(e) {                                              // 0x3459A
    e.w6 = 0x0810;
    let i = B[0x345D0 + (rng() & 0x1F)];
    e.b80 = B[0x345F0 + (level & 0x18) + i];                  // band 0: 6..27 frames ... band 3: 2..14
    set_anim(e,1); e.b80 -= 1; if e.b80 < 0 { combo_step(e) } }
```
- Combos (0x343E8, 6 bytes each, 0xFF ends): 0 = A A B, 1 = B B B, 2 = A B C, 3 = B B J, 4 = C C C, 5 = A A A A A, 6 = J, 7 = C.
- A/B/C are punches with anims 3/4/5 and attack ids 0/1/2 (8/6/10 damage). J is the jump attack, anim 8, id 3 (13 damage, knockdown).
- Combo choice per variant (32 rolls):
  - Ferris: c0 x11, c1 x7, c2 x1, c3 x4, c4 x1, c5 x7, c6 x1.
  - Gneiss: c0 x6, c1 x6, c2 x9, c3 x1, c4 x6, c5 x3, c6 x1.
  - Driver: c0 x2, c1 x3, c2 x5, c3 x6, c4 x4, c5 x4, c6 x8.

**State A "dash / charge" (0x34610)** and the trigger from state 2
```rust
fn dash_trigger(e) -> bool {                                 // 0x355C0
    let t = e.target; if t.r5 != 0 { return false }
    let dx = t.x - e.x;
    if !in_win(dx, 0xC8) {                                     // |dx| >= 200: far
        e.ba9 -= 1; if e.ba9 != 0 { return false } e.ba9 = 20; e.bad = 1;
        if (0x4541 >> (rng() & 0xF)) & 1 == 0 { return false } }        // 5/16 every 20 frames
    else {
        if t.r6 != 2 || in_win(dx, 0xA0) { return false }      // only when t walks and 160 <= |dx| < 200
        e.ba9 -= 1; if e.ba9 != 0 { return false } e.ba9 = 10; e.bad = 0;
        if (0x914B >> (rng() & 0xF)) & 1 == 0 { return false } }        // 7/16 every 10 frames
    e.facing = (t.x >= e.x) as u8; true
}
fn state_a(e) {
  if e.air != 0 { return goto_fall(e) }
  match e.r7 {
    0 => { e.r7 = 2; e.b7d = 0; set_anim(e,6); set_speed(e,0x1F); e.bb0 = 0; e.facing = (e.target.x >= e.x) as u8;
           e.wa0 = 0; aim_at_slot(e) /*0x35264: SLOT_3524C[b7c] without mirroring, dir = angle (no limit)*/; /* fall into 2 */ }
    2 => { if dash_attack_check(e) { return start_dash_attack(e) }
           if dash_stop_check(e) { return reset_ai(e) }
           e.wa0 -= 1; if e.wa0 < 0 { e.wa0 = 10; turn_to_target(e) /*0x3565C: max +-2 steps*/;
               let want = (e.dir < 0x10) as u8;
               if want != e.facing { if e.bb0 != 0 { return reset_ai(e) } e.facing = want; e.bb0 = 1 } }
           move_dir(e); step_anim(e) }
    _ => { /* 4: dash attack running */
           if e.b59 != 0 { e.b59 -= 1; if e.b59 != 0 { return } e.b2f = 1 }
           e.b80 -= 1; if e.b80 == 0 { return reset_ai(e) }
           if !in_win(e.vx, 0x30) { step_anim(e); physics(e) } else { reset_ai(e) } }   // until |vx| < 0x30
  } }
fn start_dash_attack(e) {                                     // 0x346C0, atk 0x0C
    e.ax = (-e.vx) >> 6; e.az = (-e.vz) >> 6; e.vy = 0; e.ay = 0;      // asr: friction about 1/64 per frame
    e.b80 = 60; e.r7 = 4; set_anim(e,7); e.atk_58 = 0x0C; e.b5a = 0; e.b59 = 0; e.serial_5e = next_serial() }
fn dash_attack_check(e) -> bool {                             // 0x35702
    if e.bb9 != 0 { return false } let t = e.target; if t.active == 0 || t.r5 != 0 { return false }
    let px = t.x + if t.x >= e.x { -0x60 } else { 0x60 };
    if !in_win(px - e.x, 0x40) || !in_win(t.z - e.z, 8) { return false }
    if (rng() & 0x1F) as u8 >= B[0x35796 + vrow*16 + level] { return false }      // Ferris 0; Gneiss 0..9; Driver 1..29 (/32)
    if ((e.dir.wrapping_sub(6)) & 0xF) < 5 { true } else { e.bb9 = 1; false }      // only when moving mostly along X
}
fn dash_stop_check(e) -> bool {                               // 0x356A2, every 31 frames
    e.bac = e.bac.wrapping_sub(1); if e.bac != 0xFF { return false } e.bac = 0x1E;
    let t = e.target;
    let stop = in_win(t.x - e.x, 0x70)                         // reached the target (|dx| < 112)
            || in_win(e.x - e.wae, 0x10)                        // moved < 16 px in 31 frames
            || (e.bad == 0 && t.r5 == 0 && t.r6 != 2);          // near-dash: target stopped walking
    e.wae = e.x; stop
}
```

**State C "fall" (0x34758)** and **state 0x10 "hop down" (0x347C8)**
```rust
// C: r7 0: anim 0x0C, vx=ax=vy=vz=az=0, ay=-0x20 -> r7 2: while air { physics; step_anim } else { r7=4; anim 9 }
//    -> r7 4: if step_anim ended { reset_ai }
// 0x10: r7 0: anim 0x10, vx=ax=0, vy=0x700, ay=-0x50, vz=+0x20, az=0 -> r7 2: when +2D==0 set air=1, r7=4
//       -> r7 4: while air { physics; step_anim } else { r7=6; anim 9 } -> ended: reset_ai
```

### 7.5 How class 2 plays (summary) [O]
1. **Spawn** from a script record with an entrance: walk in, dash in, wait for a player, fade in, jump in, or drop.
2. **Target**: the player with the fewest enemies on him (if any has < 3), otherwise a weighted score (section 5.2).
3. **Approach**: walk at the aggression-dependent speed toward one of 6 slots, at about 96..120 px left or right of the target and +-24 px in depth. Turn at most 45 degrees per re-path, re-path every 41 frames, and rotate the slot when stuck.
4. **At the slot**: roll the attack chance `T_353B6[variant][aggr]` (Ferris 2/32 calm .. 21/32 furious; Driver up to 31/32). Otherwise circle to the next slot, hover (MAAI), or back off for 50 frames. Taunt about once per approach.
5. **Attack**: pick an attack position 55 px from the target (240-frame limit), then pick a combo (only 7/32 if the target's back is turned). Each hit is followed by a level-dependent recovery pause. A combo can chain into another while the target stays in range.
6. **Dash**: when the target walks 160..200 px away, or is more than 200 px away, the thug may run (4 px/frame). Gneiss and Driver can turn the run into a dash attack (id 0x0C).
7. **Bumping** into a free player gives a pause of 28..3 frames, then an attack.
8. **Hit**: section 10. Light hits flinch while poise lasts; poise 10 refills 60 frames after recovery.

---------------------------------------------------------------------------------------------------

## 8. Stage pacing: spawn lists, waves, camera lock, GO, time limit

### 8.1 Spawn interpreter `spawn_script_tick` 0xFBF4 (last call of every game frame) [O]
State is the struct at `$66C(a5)`: `+0` command flag (always 0, see the quirk), `+4` init done, `+8` list-1 pointer, `+C` list-2 pointer.
`0xFBE8` clears `+4` and `+0` at area start (called from 0xAAEE).

Both lists are sorted by trigger. A record fires when the camera passes its trigger:
- horizontal stages: `camX ($744) >= trig`, unsigned.
- vertical stages (`$4DC != 0`): `camY ($748) <= trig`, signed.

There is no end marker. Each list simply ends with a sentinel trigger the camera never reaches (0x9FFF or 0xFFFF).

```rust
fn spawn_script_tick() {
    let s = &mut ram[0xFF866C..];
    if s.init == 0 {
        s.l1 = L[L[0xFC2E + 4*stage_4d9] + 4*area_786];
        s.l2 = L[L[0xFC4E + 4*stage_4d9] + 4*area_786];
        s.init = 2; return;                                   // nothing spawns on the first frame
    }
    // list 1: 14-byte records {trig, flags, class, x, z, var, y}  (props, scenery, script commands)
    loop { let r = s.l1; if !reached(W[r]) { break }
        let f = W[r+2];
        if f & 0x80 != 0 { command(r) }
        else { match ALLOC[f & 7]() { None => { s.l1 = r + 14; break }      // pool full: record is DROPPED
               Some(o) => { o.active = 1; o.w20 = W[r+4]; o.x = W[r+6]; o.y = W[r+0xC]; o.z = W[r+8]; o.var = W[r+0xA] } } }
        s.l1 = r + 14; }
    // list 2: 16-byte records (enemies, waves, breakables)
    loop { let r = s.l2; if !reached(W[r]) { break }
        let ok = spawn_record(r); s.l2 = r + 16; if !ok { break } }      // pool full: dropped, stop this frame
}
fn reached(t: u16) -> bool { if vertical_4dc != 0 { (t as i16) >= (camY as i16) } else { t <= camX } }
fn spawn_record(r) -> bool {                                 // 0xFDAE; also used by the wave controllers
    let f = W[r+2];
    if f & 0x80 != 0 { command(r); return true }
    match f & 0x60 { 0x40 => if players_4ef == 0 { return true }, 0x60 => if players_4ef != 2 { return true }, _ => {} }
    let hi = f >> 8;
    if f & 0x18 != 0 { if f & 0x10 != 0 { if hi < level_5ee { return true } }      // level <= hi
                       else if hi > level_5ee { return true } }                   // 0x08: level >= hi
    let Some(o) = ALLOC[f & 7]() else { return false };
    o.active = 1; o.w20 = W[r+4]; o.x = W[r+6]; o.z = W[r+8]; o.var = W[r+0xA];
    o.w5c = W[r+0xC]; o.b5b = B[r+0xE]; o.b96 = B[r+0xF]; true      // only integer words are set; Y stays 0
}
// 0xFE6A = the same, but X = W[r+6] + camX (screen-relative). Used by the stage 3 car section and by its boss controller.
fn command(r) { match W[r+4] {                               // table 0xFD60
    0x00 => { s.l1 += 14 }                                    // 0xFD74 (sets and clears s.flag): skips one list-1 record
                                                              // (no effect when issued from list 1: s.l1 is overwritten)
    0x04 => reroll_aggression(),                              // 0xFD88 -> 0x182A2
    0x08 => { w5f2 = W[r+6]; w5f4 = W[r+0xC] }               // force aggression = argC while camX < arg6
    0x0C => { time_4e8 = W[r+6] }                             // set the stage time limit (BCD mm:ss)
    0x10 => add_time_bcd(W[r+6]),                             // 0x104A8, capped at 04:00
    _ => {} } }
```
- Pool selector `f & 7`: 0 = A (enemies), 1 = B (controllers/props), 3 = E (items/containers).
- `class` is the dispatch offset `+$20` of that pool.
- **Script pointer tables**: list 1 = **0xFC2E**, list 2 = **0xFC4E**, 8 longs each (one per stage). Each long points to a 4-long per-area table (areas 0..3; unused areas repeat area 0).
  The wave table pointer is **0x7B37C** (8 longs).

### 8.2 Wave controller: pool B class 15 (`+$20 = 0x3C`, handler 0x7B336) [O]
A list-2 record `{pool B, class 0x3C, var = wave number}` creates one. Only one wave runs at a time. [I]
The wave script is at `T + W[T + 2*var]`, where `T = L[0x7B37C + 4*stage]`.

Format:
```
wave   := group* final
group  := thr1P:u16 thr2P:u16 thr3P:u16  spawn+  terminator(16 bytes, first word negative, e.g. 0x9FFF)
spawn  := delay:u16 flags:u16 class:u16 x:u16 z:u16 var:u16 w5c:u16 b5b:u8 b96:u8    (= list-2 record with
                                                                                       the trigger replaced by a delay)
final  := thr1P:u16 thr2P:u16 thr3P:u16  end:u16 (negative; end & 0xE = end type)
```
```rust
fn wave_ctrl(c) { match c.r4 {
  0 => { c.ptr_a0 = wave_script(stage, c.var); wave_timer_50c = 7200; c.spawned_a6 = 0; c.w80 = 0; c.r4 = 2 }
  2 => { if car_mode_518 != 0 { return end_go(c) }           // 0x7B4DE
         match c.r5 {
          0 => { if c.w80 != 0 { c.w80 -= 1; return }
                 let thr = W[c.ptr_a0 + 2*players_4ef];
                 wave_timer_50c -= 1;
                 if wave_timer_50c >= 0 && alive_4f3 > thr { return }   // wait: alive > threshold, timer not expired
                 c.ptr_a0 += 6; let d = W[c.ptr_a0] as i16;
                 if d < 0 { c.b80 = 0x1E /*high byte of w80*/; c.r5 = 4; if wave_timer_50c < 0 { kill_all_50e = 2 } return }
                 c.w80 = d as u16; c.r5 = 2 }
          2 => loop { c.w80 -= 1; if c.w80 >= 0 { return }  // wait `delay` frames (delay 0 = same frame as the previous)
                 let r = c.ptr_a0; if spawn_record(r) { c.spawned_a6 += 1 }   // a record that cannot spawn is SKIPPED
                 c.ptr_a0 = r + 16; c.w80 = W[c.ptr_a0];
                 if (c.w80 as i16) < 0 { c.ptr_a0 += 16; c.w80 = 3; wave_timer_50c = 7200; c.r4 = 2; c.r5 = 0; return } }
          _ => { // 4: end of wave, 0x7B464
                 if c.b80 != 0 { c.b80 -= 1; if c.b80 == 0x19 { kill_all_50e = 0 } return }
                 for p in players { if p.active != 0 && p.w4 != 0x0200 { return } }   // all players in normal control
                 match W[c.ptr_a0] & 0xE {
                   0 => end_go(c),                                  // GO arrow
                   2 => { set_leader(); area_transition() /*0x1D95E*/; c.w4 = 0x0600 }   // walk on to the next area
                   _ => { set_leader(); stage_end_1dac4(); c.w4 = 0x0600 } }              // 4: stage exit
               } } }
  _ => free_poolB(c) } }                                      // 4, 6: 0x4938
fn end_go(c) { cam_go_7a9 = 1; set_leader(); c.w4 = 0x0600 }
fn set_leader() { /* 0x1E5F6: clear +$C0 of all players, set +$C0 = 1 on the right-most present player */ }
```
- **2-minute rule [O]**: each group has a 7200-frame budget (`$50C`; 0x10482/0x10490/0x1049E add 600 when time is extended).
  If the budget runs out, the next group spawns anyway. If it runs out before the final threshold, `$50E = 2` for 5 frames (0x1E down to 0x19). Every enemy still alive then dies through `kill_all_check`, with no score.
- **Thresholds** compare against `$4F3` = number of alive pool A objects with `+$7D >= 0` from the previous frame's census. Dinosaurs and Vice T. use `+$7D = 0xFF` and are not counted. A threshold of 99 means "no wait".

### 8.3 Camera lock and GO [O]
- The camera (struct `$73C(a5)`, `+8` = camX `$744`, `+$6D` = `$7A9` "GO" flag) follows a per-stage **segment table at 0x21B2A**.
  It holds 16-byte records `{x, y, lead, w6, w8, area, mode, c, d, next_mode, vertical}` (`camera_segments.py`).
- A segment becomes active when `camX >= seg.x` (or the Y test when vertical). Code: 0x208DC → 0x209C0.
- **Mode 4 (0x21086) = wave lock**: `camX = seg.x` every frame, `+$78 = 1`, until `+$6D` (= `$7A9`) becomes nonzero.
  Then `mode = seg.next_mode` (usually 0 = follow), `+$78 = 0`. The flag is cleared at the end of every camera frame.
- In stage 1 every wave controller trigger X equals a mode-4 segment X: 0x4D0 (area 0), 0x000/0x1B0 (area 1), 0x000/0x0A0 (area 2). So:
  **the camera stops exactly where the wave starts and stays locked until the wave's end code sets `$7A9`.**
- End type 0 sets `$7A9` at once and shows the GO arrow. [I: the arrow itself is drawn by a task created in camera mode 1 (0x20C5A, task 0x8840 in slot 0x40)]
  End types 2 and 4 start the automatic walk to the next area or stage exit. The player code sets `$7A9` when the walk is done (0x1DE62 / 0x1E4B4).
- Follow mode 0 (0x20A18): camX advances by at most 6 px/frame. It moves when (right-most player X) - camX > lead (`+$50`; per player count from 0x20B62 = 0xC0/0xE8/0xE8 when the segment has no lead). It never passes (left-most player X - 16). Never backwards. [O; details = stage topic]

### 8.4 Stage time limit [O]
- `$4E8`/`$4E9` = minutes:seconds in BCD. Script command 0xC sets it (stage 1 wave 5 sets 03:50). Command 0x10 adds time; the result is capped at 04:00.
- `0x1037E` (per frame): if `$4EA != 0` it counts down; otherwise, if time != 0, it decrements by one BCD second and sets `$4EA = 50`. So one second = 51 frames.
- `0x103FE`: when time is 0 (and nothing paused): clear the HP of every player with `+$4 < 4`. [I] It is a hidden or HUD timeout. Whether it is drawn is a HUD question. [U]

### 8.5 Other flags
- `$518(a5)`: set to 1 by pool B class 88 (0x8C06E) [I: Cadillac driving section].
  While set: enemies that leave the screen are freed (0x173C), and wave controllers end at once with GO.
- `$50E(a5)` kill-all: 1 = boss died or a special item (stage 3 radio), crediting `$4E0`; 2 = wave timeout, no credit.
- `$4F3(a5)` previous-frame alive count. `$67BE(a5)` attack serial counter.

---------------------------------------------------------------------------------------------------

## 9. Stage 1 decoded (from `stage_scripts.txt` and `camera_stage1.txt`)
Notation: `var` = +$26 word; for class 2 `var = variant<<8 | entrance`. Conditions: `P>=2` = 2+ players; `P3` = 3 players only.
Entrances: 0 walk in, 2 dash in, 4 wait until near, 6 fade in, 8 jump in, A drop.

**Area 0** (camera segments: X 0x380 follow with lead; X 0x4D0 **lock (wave 0)**; X 0x580 area end)
- List 1 (props), all pool B:
  - trig 0: B#69, B#31, B#20 (scene controllers [I])
  - 0x40: B#66 var 0x09; 0x100: B#73; 0x1C0: B#74
  - 0x2A0: B#66 var 0x49/0x54/0x4A; 0x390: B#66 var 0x0E
  - 0x3A0: **force aggression 1 while camX < 0x4E1**
  - 0x460: B#40 at (0x620, 0x130)
- List 2:

| trig | what | pos (X, Z) | var / extra |
|---|---|---|---|
| 0x000, 0x040, 0x260 | A#49 parallax birds (scenery) | | |
| 0x040, 0x100 | A#50 scenery | | var 0x0A / 0x0C |
| 0x388 | FERRIS, walk in | 0x520, 0x140 | |
| 0x388 | FERRIS, dash in (P>=2) | 0x540, 0x150 | +5B=0xFF (no effect for this entrance) |
| 0x388 | GNEISS, dash in (P3) | 0x540, 0x140 | |
| 0x410 | container (E#5) var 2 | 0x5A0, 0x150 | drop row 0x02 |
| 0x420 | FERRIS, dash in | 0x600, 0x124 | drop 0x04 |
| 0x420 | GNEISS, dash in (P3) | 0x630, 0x128 | drop 0x04 |
| 0x450 | containers var 2 / var 0 (P>=2) | 0x5D0 / 0x5D4 | drops 0x2A / 0x12 |
| 0x460 | container (P3) | 0x5F6, 0x142 | drop 0x04 |
| 0x4C0 | FERRIS, dash in | 0x4A0, 0x148 | |
| 0x4D0 | **wave 0** controller | | |

**Area 1** (segments: X 0 lock (wave 1), X 0x1B0 lock (wave 2), X 0x220 mode 0x12)
- List 1:
  - trig 0: B#69, B#31, B#20, B#66 var 0x0F
  - 0x100: force aggression 7 while camX < 0x220
  - 0x1E0: B#12 at (0x370, 0x40, Y 0x30) var 4
- List 2:
  - trig 0:
    - **wave 1**
    - containers var 0xA at X 0x81 and 0x14B (drops 0x0C, 0x27)
  - trig 1:
    - GNEISS, dash in, (0x1A0, 0x20), drop 0x27
    - GNEISS, walk in (P3), (0x1A0, 0x60)
  - 0x40: BLADE/RAZOR (A#14) var 4 at (0x1E0, 0x48)
  - 0xC0: container drop 0x01
  - 0x120:
    - FERRIS, walk in, (0x2A0, 0x25), +5B 0xFF, drop 0x19
    - GNEISS, wait (0x2C8, 0x18)
    - BLADE/RAZOR var 0x202 (P3)
  - 0x182: container drop 0x15
  - 0x1B0: **wave 2**; 0x220: **wave 3**

**Area 2** (segments: X 0 lock (wave 4), X 0xA0 lock (wave 5), X 0x1D0 mode 6)
- List 1:
  - trig 0: B#69, B#31, B#20, B#14 at (0x60, 0x120, Y 0x290), B#66 var 0x10
- List 2:
  - trig 0:
    - **wave 4**
    - containers (P3: drop 0x24; P>=2: var 0xC drop 0x0F)
  - 0x30:
    - container drop 0x35
    - GNEISS, dash in, (0x30, 0x140), drop 0x25
  - 0x80: BLADE/RAZOR var 0x202 (P>=2)
  - 0xA0: **wave 5**
  - 0xF4: **VICE T.** (A#18, var 0) at (0x318, 0x128): stage 1 boss

**Waves of stage 1** (table 0xADE82)

| wave | groups: `[alive thresholds 1P,2P,3P] -> spawns (delay)` | end |
|---|---|---|
| 0 | [3,3,3] → GNEISS dash-in (32f) at (0x4C0, 0x12A); FERRIS dash-in P>=2 (0f) at (0x4C0, 0x128) | [0,0,0] → type 2 (walk to area 1) |
| 1 | [99,99,99] → FERRIS dash-in (32) (0x190, 0x20); B#12 (192); PUNK/THUG var 8 (32) (0xE8, 0x60); PUNK/THUG P>=2 (64) drop 0x15; FERRIS fade-in var 6 (160) | [4,5,6] → type 0 (GO) |
| 2 | [3,4,4] → HAMMER/ELMER/WRENCH var 4 (0) and var 0x404 (0) at X 0x350; var 0x206 (50) at X 0x1A0 | [4,5,5] → GO |
| 3 | (no groups) | [0,0,0] → type 2 (walk to area 2) |
| 4 | [99,99,99] → HAMMER/ELMER/WRENCH var 0x202 (160) +5B 0xFF drop 0x30; B#14 var 4 (80); FERRIS dash-in (16) | [0,0,0] → GO |
| 5 | [99,99,99] → DRIVER walk-in P3 (21) at (0x240, 0x128); then [0,0,0] → **set time 03:50** | [99,99,99] → type 4 (boss area) |

Reading of wave 1: the first group spawns at once (threshold 99). Each spawn waits its delay (32, 192, 32, 64 and 160 frames after the previous one). The camera unlocks (GO) when at most 4/5/6 enemies (1P/2P/3P) are left, so stragglers can follow the players. **[O]**
The other 7 stages are fully decoded in `stage_scripts.txt` (sections "STAGE n").

---------------------------------------------------------------------------------------------------

## 10. Enemy hit reaction and death (shared by all pool A enemies)
Analysed by a helper agent; checked against the listing at the call sites used above. Tables: `hit_tables.txt`. Disassembly: `hit_2a606.txt`.
Everything is **[O]** unless tagged.

### 10.1 Who hits whom (called from 0x10546)
| routine | attackers | victims | data |
|---|---|---|---|
| `0x10F96` → `0x11008` | list `$6722` (player attacks, `+$2E == 4`) | list `$6726` (pool A enemies, pushed by 0x4BEA) | player attack table 0xA26FC |
| `0x115B0` → `0x11652` | list `$6726` enemies with `+$2E == 4` and `+$59 == 0` | players (list `$6720`) | 0xA2EA4 + 0x6422 (section 6.6) |
| `0x10DCC` | list `$6722` | players | player vs player, 1 damage |

```rust
fn player_hits_enemy(atk, e) {                                  // 0x11008
    if e.active & 2 != 0 { return }                              // already reacting: invulnerable to this list
    if atk.serial_5e == e.serial_5e { return }                    // one hit per swing
    if atk.box_49 == 0 || e.box_48 == 0 || !hitbox_overlap(atk, e) { return }   // 0x128F8
    e.serial_5e = atk.serial_5e; e.credit_2 = atk.credit_2;
    if e.b87 != 0 && atk.b2e != 6 && atk.bc8 != 0x12 && atk.bc8 != 0x13 && GUARD[e.w84/4](e) { block_fx(); return } // only +$84=0xA4 (0x55A18) can block
    e.active = 2; e.l4 = 0x0202_0000; e.b7a = 0; e.w68 = atk;
    let tab = L[0xA26FC + 4 * if atk.credit_2 >= 0 { atk.char_3 } else { 0 }];
    let id = atk.atk_58;                                        // player attack ids are even; record stride 8
    if id == 0x6E || id == 0x6C { return special_nonlethal(e) } // 0x111C6: signed damage byte, HP floor 1
    let r = tab + id*4;
    e.hp_prev = e.hp; let mut d = B[r] as u32;
    if atk.w_a0 == 0x203 { d = charge_scale_11262(atk, d) }      // [I] vehicle/charge
    d = player_damage_scale(d, atk, e);                          // 0x2BA2A below
    e.hp = e.hp.saturating_sub(d);
    e.poise = e.poise.saturating_sub(B[r+1]);
    e.b7b = B[r+2]; e.r6 = B[r+2]; e.r7 = 0;                     // reaction type -> 0x2A606 table
    e.b59 = B[r+3]; atk.b59 = B[r+3]; if atk.w_a0 == 0x202 || atk.w_a0 == 0x203 { obj(atk.w70).b59 = B[r+3] }
    if B[r+4] != 0 { sfx(B[r+4]) }
    if atk.credit_2 >= 0 { add_score(atk.credit_2, bcd(0x10292 + B[r+5])) }   // 100/200/400 per hit; not during area walk
    spark(B[r+6]); e.b9a = B[r+7];
    e.facing = (src(atk).x > e.x) as u8;                         // turn toward the attacking player
}
fn player_damage_scale(mut d, atk, e) -> u32 {                  // 0x2BA2A (2 rng() calls)
    if debug_nodamage() { return 0 }
    if e.b2e == 4 { let m = W[0xA320C + 2*e.atk_58]; if m != 0 { d = d*m >> 5 } }      // counter-hit bonus
    let p = 0xA32FA + e.w84;                                     // per-type weak points
    if 2*B[p] == atk.atk_58 { d = (d*B[p+1]) & 0xFFFF >> 5 } else if 2*B[p+2] == atk.atk_58 { d = (d*B[p+3]) & 0xFFFF >> 5 }
    if atk.credit_2 >= 0 { d = lo16(d) * player(atk.credit_2 & 3).power_c6 }
    d = lo16(d) * B[0x2BECE + e.w84*8 + level] >> 10; if d == 0 { d = 1 }
    let (mut hp, mut bar) = (e.hp, 0); while hp > 100 { bar += 0x40; hp -= 100 }
    d = lo16(d) * W[0x2BC4E + bar + (rng() & 0x3E)];
    let band = if hp >= 75 {0} else if hp >= 50 {0x40} else if hp >= 25 {0x80} else {0xC0};
    (lo16(d) * W[0x2BB4E + band + (rng() & 0x3E)]) >> 10        // can be 0
}
```
- Hit box test `0x128F8`: boxes are 12 bytes at `+$44 + id*12` = `zo, zs, xo, xw, yo, ys`. When facing right, X start = `X - xo - xw`.
  Per axis: `(a0 - b0 + a_size) as u16 <= a_size + b_size`, tested in the order Z, X, Y.

### 10.2 Reaction dispatcher `0x2A606` (+$4 = 2, +$5 = 2), table 0x2A620 by +$06
| +$06 | handler | meaning |
|---|---|---|
| 0x00 / 0x02 | 0x2A63E / 0x2A680 | light flinch (anim 0 / 1); turns into knockdown 4 / 6 if HP == 0, poise == 0 or airborne |
| 0x04 / 0x06 | 0x2A6C8 / 0x2A852 | knockdown A (anim 2) / B (anim 3) |
| 0x08 | 0x2A9C8 | thrown by a player |
| 0x0A, 0x0C | 0x2AC02 | burn knockdown (charred death when HP 0 and type 0xC) |
| 0x0E..0x12 / 0x14..0x18 | 0x2ADDE / 0x2AEB2 | slide back without lift (flinch anim 0 / 1) |
| 0x1A | 0x2AFF0 | get up / dizzy |
| 0x1C | 0x2B2E6 | wall slump |
After the handler: `register_hurtable` (0x4BEA), `register_draw` (0x49CA).

```rust
fn flinch(e, anim, heavy) {                                     // types 0 (heavy=4) and 2 (heavy=6)
    if e.r7 == 0 {
        if e.hp == 0 || e.poise == 0 || e.air != 0 { e.w6 = heavy << 8; return knockdown(e) }
        hud_hit(0); set_hit_anim(e, anim);                       // anim list L[0x2E68C + e.w84]
        e.b7a = B[0x2B8C4 + ((level*2) & 0x30) + (rng() & 15)]; // counter-attack delay used by AI state 0
        e.r7 = 2;
    } else { e.b59 -= 1; if e.b59 == 0 { recover(e) } }          // flinch length = hit-stop byte
}
fn knockdown(e) { edge_clamp(e) /*0x2B506: X in [camX-0x50, camX+0x1D0]*/; match e.r7 {
  0 => { hud_hit(0); set_hit_anim(e, 2 /*type 6: 3*/);
         // launch (vx, ax, vy, ay); vx/ax mirrored when facing right:
         // type 4: alive (640,0,1280,-64) dead (768,0,1920,-64); type 6: alive (1024,0,1280,-64) dead (1024,0,1200,-48)
         set_launch(e); if e.hp == 0 { death_voice(); drop_item(e) /*0x24B36*/ }
         e.vz = 0; e.az = 0; e.air = 1; e.b83 = 0; e.b7b = 4; e.poise = 10; e.r7 = 2 }
  2 => { push_list_6ac4(e); physics(e); screen_bounce(e); if e.b2c == 0 { step_anim(e) } floor_collision(e);
         if e.b50 != 0 && e.b50 & 1 == 0 { return wall_slump(e) }
         if e.air == 0 { e.air = 1; e.vy = BOUNCE_VY; e.ay = 2*BOUNCE_AY; e.r7 = 4; land_fx(); step_anim(e) } }
         // bounce records at +4 of each launch record (0x2CBB4 / 0x2CBBC / 0x2CBC4 / 0x2CBCC)
  4 => { push_list_6ac4(e); physics(e); screen_bounce(e); if e.b2c == 0 { step_anim(e) } floor_collision(e);
         if e.air == 0 { lie_time(e); if e.hp == 0 { e.b80 = 1 } land_fx(); e.r7 = 6 } }
  _ => { e.b80 -= 1; if (e.b80 as i8) < 0 { getup_or_die(e) } else { step_anim(e) } } } }
fn lie_time(e) { e.b80 = if e.hp == 0 { 6 } else { max(0, B[0x2B614 + e.w84 + (rng()&15)] as i8 + B[0x2B604 + (rng()&15)] as i8) } }
fn screen_bounce(e) { if e.b63 > 0 && e.hp == 0 {               // dead big enemies bounce off the screen sides
    if e.vx >= 0 { if e.x > camX + 0x150 { e.vx = -e.vx; e.ax = -e.ax } } else if e.x < camX + 0x30 { e.vx = -e.vx; e.ax = -e.ax } } }
fn getup_or_die(e) {                                            // 0x2AF86
    if e.hp == 0 { e.l4 = 0x0400_0000; e.active = 1;              // -> class death routine
        if e.credit_2 >= 0 { if car_mode_518 == 0 { player(e.credit_2).kills_10b += 1 } else { all players += 1 } }
        return reroll_aggression() }                            // every kill re-rolls $5EC
    if (e.credit_2 as i8) < 0 { e.credit_2 = 0 }
    e.l4 = 0x0202_1A00; set_hit_anim(e, 4) }
fn recover(e) { e.b79 = 60; e.pal = 0; e.b9a = 0; e.active = 1; jump(L[0x2B56C + e.w84]) }  // per-type resume; class 2 -> 0x33F38
```
- Slide types: record `0x2CB6E + 6*k` = `(frames, vx, ax)`: (7, 1536, -128), (10, 1792, -177), (10, 2560, -256). They spawn dust on odd frames and stop at walls.
- Thrown (type 8): launch from `0x2CB46 + thrower.char*8`. Thrown bodies hit other enemies through list `$6B32`.
  On landing they bounce once (vy = -vy, ay*2). On hitting a wall they slump.
- Burn (0xA/0xC): palette `+$23 = 0x84`, flame effects (pool C class 0x20), launch (640,0,1280,-64) alive / (512,0,1920,-64) dead.
- Wall slump (0x1C): 30 frames, then slide down 1.5 px/frame to the floor, then lie time + 10.
- Get up (0x1A): when the get-up animation ends, the enemy may become **dizzy** (stars, 90 + 8x20 frames, hittable). Otherwise it recovers.
  The dizzy chance uses bit tables 0x2B180 (row by +$9A and aggression), 0x2B158 (HP quarter), 0x2B168 (alive count) and 0x2B28C (level). Only one dizzy enemy at a time. Vice T./Walther never get dizzy.
  QUIRK: 0x2B28C has 11 valid longs; levels 11..31 read the following code bytes.
- **Poise**: starts at 10. Each hit subtracts the attack's poise byte. At 0 the next hit knocks down. A knockdown resets poise to 10. AI state 0 refills it to 10 when `+$79` (60 after recover) runs out.
- **Grabbed** (+$5 = 6, 0x2B35A): the position follows the grabber `+$70` using `0x2D0E0[grabber class][e.w84/2][grabber.+$60 & 0x3F]` = `{dx, dy, anim, facing}`. On release: snap to the floor, `+$7B = 0x20`, recover.

### 10.3 Death and kill-all
```rust
fn class2_dying(e) { match e.r5 {                              // 0x3484E (+$4 = 4)
    0 => { award_kill_score(e) /*0x325E6*/; e.w80 = 120; e.r5 = 2 }
    _ => { let t = e.w80 as i16 - 1; if t < 0 { e.r4 = 6; return } e.w80 = t as u16; if t & 1 != 0 { register_draw(e) } } } }   // blink 120 frames
fn kill_all_check(e) {                                          // 0x326F8, first call of the class 2 frame
    if kill_all_50e == 0 || e.hp == 0 { return }
    e.hp = 0; e.l4 = 0x0202_0000; e.b59 = 2; e.poise = 0; e.w68 = killer_4e0;
    e.credit_2 = if kill_all_50e == 2 { 0xFF } else { obj(killer_4e0).credit_2 } }
```
- The item drop (`+$96`, section 13) happens at the killing blow, inside the knockdown/thrown/burn handlers, before the body falls.
- The census stops counting the enemy when `+$4` becomes 4.

---------------------------------------------------------------------------------------------------

## 11. Worked example: dinosaur SHIVAT (pool A class 24, handler 0x4838C)
Analysed by a helper agent (`dino_shivat_dis.txt`). Everything is **[O]** unless tagged.
- **Init**: `+$3 = 7`, `+$84 = 0x1C`, anims 0x49574, HP from 0x31424 (60..200 by level), poise 10, `+$72 = 200`.
  Own step table `+$64 = 0x49460` (not 0x17A4). `+$7D = 0xFF` (never counted in `$4F3`; waves never wait for it). `+$C0 = 0xFF` while in mode 0.
- **+$5C from the spawn record = "leave" camera X.** When `camX >= +$5C`, the dinosaur walks away (mode 6). Stage 2 uses 0x569, just after the wave trigger 0x568.
- **No kill score.** At HP 0 it does not die: it walks away (MANZOKU).
- **Movement is per animation frame**: on each new frame (`+$2D != +$35`) it adds `step[dir]` (about 28.6 px per step) to an anchor X and to Z. It sets `$755C = 1` [I: screen shake]. The drawn X = anchor + per-frame offset (0x493C4).

Behaviour modes `+$A8` (table 0x48510; debug names at 0x490CE):

| +$A8 | name | handler | behaviour |
|---|---|---|---|
| 0 | MICHI | 0x48530 | dormant, `+$0 = 2` (not hittable), roars every 50 + (rng & 63) frames; leaves this mode only via sub-state 4 (set from outside [U]) |
| 2 | MATOWARI | 0x485EA | wander, not hostile |
| 4 | OKORI | 0x4878C | hostile (below) |
| 6 | MANZOKU | 0x48C40 | satisfied: turns and walks off screen, freed by 0x173C |
| 8 | COLOR | 0x48CFE | palette flash (15 toggles), then hostile, enraged (`+$A4 = 1`) |

```rust
fn shivat_frame(o) {                                            // 0x484B4
    if o.b79 != 0 { o.b79 -= 1; if o.b79 == 0 { o.poise = 10 } }
    if o.b7a != 0 { o.b7a -= 1 } else {
        MODE[o.a8 / 2](o);
        o.x = o.anchor_x + STEP_OFS[o.b2c] * if o.facing != 0 { -1 } else { 1 };   // 0x493C4, offsets 0x493E2
        floor_collide_12e20(o); if o.air != 0 { o.y = o.w54 + o.w56 } }
    register_draw(o); push_list_4c0a(o);
}
fn okori(o) {                                                   // 0x4878C
    if o.w5c != 0 && camX >= o.w5c { return enter_manzoku(o) }
    o.aa += 1; if o.aa >= 1800 { o.ac -= 1; if o.ac < 0 { o.ac = 10; if rng() & 15 == 0 { return enter_manzoku(o) } } }
    match o.r6 {
      0 => { o.anchor_x = o.x; o.target = pick_target(o); o.b0 = 0; o.r6 = 2; o.ad = 1 + (o.ba != 0) as u8; set_walk_anim(o) }
      2 => {
        o.b4 = (o.b4 - 1).max(0);                                // attack cooldown
        if o.b4 == 0 && long_reach(o) && target_free(o) && B[0x49560 + (rng() & 15)] != 0 { o.r6 = 6; o.r7 = 8; return } // 14/16
        step_anim(o);
        if o.b2d != o.b35 {                                      // new animation frame
            if at_screen_edge_facing_out(o) { o.r6 = 4; set_anim(o, 0x0E); return }      // turn round
            if o.b4 == 0 && bite_range(o) && target_free(o) { o.r6 = 6; o.r7 = B[0x49520 + band + (rng() & 15)]; return }
            if o.b2c == 0 && (rng() & 15) < 2 && o.ba == 0 { o.b80 = 120; o.r6 = 0x0A; set_anim(o, 0x0A); return }  // idle 120
            if o.target.w4 != 0x0200 || { o.b0 -= 1; o.b0 < 0 } { o.b0 = 300;
                if !player_in_sight(o) || (0xFE00 >> (rng() & 15)) & 1 == 0 { target_nearest_enemy(o) } }  // may attack poachers
            o.dir = clamp_turn(angle_to(o, o.target.x, o.target.z), o.facing);   // +-4 around 8 or 24
        }
        step_move(o);
      }
      4 => { if anim_done(o) { o.bb -= 1; if o.bb == 0 { enter_manzoku(o) } else { o.facing ^= 1; o.dir = (o.dir + 16) & 31; o.r6 = 0 } } }
      6 => ATTACK[o.r7](o),                                       // 0x48A80
      0x0A => { o.b80 -= 1; if o.b80 < 0 { o.r6 = 0 } step_anim(o) }
      _ => {}
    } }
```
- QUIRK `band`: 0x488BA reads `move.w -$7A14(a6),d1` (a6-relative, looks like a typo for `$5EC(a5)` aggression), then `(w << 3) & 0x30`.
  Table 0x49520 has 4 rows with 8/6/4/2 zero bytes (bite 50%..12%). The port must decide: emulate the RAM read or use `$5EC`.
- Ranges:
  - bite `0x491F8`: on screen, target active, |dz| <= 11, target X within +-17 of X +- 0x6D.
  - long reach `0x49296`: dz in [-11, 0], X within +-23 of X +- 0x84.
  - in sight `0x49148`: |dx| <= 0xA0 and |dz| <= 0x10.
  - nearest enemy `0x491A4`: Manhattan distance over the previous frame's list `$69BC`, objects with `+$7D >= 0`.
- Attacks: bite (atk 0x38: 14 dmg), tail swing + turn (0x39: 10, knockdown), triple stomp (0x3A x3), grab-and-hold (0x4E9A6 check, `+$5 = 4`, links both `+$70`; the player becomes `02 06 00 00`; cooldown `+$B4 = 180`).
- When hit (own handler 0x48D16 → 0x48E02): HP 0 → MANZOKU. A dinosaur that is not yet enraged becomes COLOR → hostile. **A neutral dinosaur turns hostile when hit.**

---------------------------------------------------------------------------------------------------

## 12. Worked example: boss VICE T. (pool A class 18, handler 0x42590; stage 1)
Analysed by a helper agent (`boss_vicet.py`, `boss_vicet_dis.txt`). Everything is **[O]** unless tagged.
- Variant 0 = VICE T.:
  - `+$84 = 0x30`, `+$63 = 2`, anims 0x438F2.
  - **HP = boss table idx 0** (350..600 1P, 500..700 2P, 600..750 3P). `+$C0` = max HP.
  - `+$7D = 0xFF` (not counted in `$4F3`).
- Variant 2 = WALTHER (name 0xD): normal HP table (100..300), counted, obeys kill-all, no intro.
- Shared init: poise 11, `+$A8 = 2` rage charges, `+$C2 = 2` summon counter.
- **Intro** (Vice T. only, 0x426AC):
  - wait for a pool B class 50 prop; spawn B#76; anim 0x14
  - walk (speed id 7) to (0x317, 0x136)
  - spawn B#66 props; gun anim 0x15 with muzzle flash; music 0x0E
  - **create the reinforcement controller** (pool B class 22, 0x7BEDC); `-$70BC(a5)` = boss
- **Frame**:
```rust
fn vice_ai(o) {                                                // 0x428CE (+$4=2,+$5=0)
    o.aa = (o.aa - 1).max(0);                                   // rage timer
    if o.b79 != 0 { o.b79 -= 1; if o.b79 == 0 { o.poise = 11 } }
    STATE[o.r6](o);                                             // 0: decide, 2: approach, 4: attack, 6: rage, 8: fall, A: summon
    floor_collision(o);
    o.c3 -= 1; if o.c3 == 0 { o.c3 = 200; if o.b50 & 0x80 != 0 { goto_attack_obstacle(o) } }
    register_draw(o); register_hurtable(o);
}
fn approach(o) {                                               // state 2, 0x42B96
    if o.air == 0 {
        if let Some(p) = player_in_front(o) /*|dx|<0x30,|dz|<8, p.r5==0*/ { o.target = p; return set_state(o, 4, 0) }
        if player_behind(o) { return back_elbow(o) }            // anim 0x0A, atk 0x35
    }
    if o.bd != 0 { return set_state(o, 6, 0) }                  // rage
    if !target_alive(o) || { o.b2 -= 1; o.b2 < 0 } { o.b2 = 600; retarget_44316(o); f_44974(o) }
    if o.b1 < 4 { o.b4 -= 1; if o.b4 < 0 { if o.air != 0 { o.b4 = 0 } else { o.b4 = 60;
        if (rng() & 31) < B[0x44B84 + o.variant*16 + level] { return set_state(o, 4, 0) } } } }   // every 60 frames
    approach_substate(o);   // walk, or 3/16 charge (anim 4, speed 0x0D) to the attack point; 120-frame give-up; stuck check
}
fn pick_combo(o) { let r = rng() & 31;                          // 0x44C5A
    o.combo = if o.charged != 0 { 5 + (r >> 4) } else { B[0x44C8C + r + if o.variant != 0 { 32 } else { 0 }] };
    o.cursor = o.combo * 8 }                                    // scripts at 0x43004, 0xFF = end -> pick again
```
- Attack chance per 60 frames (/32) by level: Vice T. 6..31; Walther 1..
- **Actions** (0x4306C):
  - 0: approach to a combo-specific distance
  - 1: dash
  - 2: punch (atk 0x12: 7 dmg)
  - 3: heavy (0x13: 15)
  - 4: jump attack (0x14, vy 0x700, ay -0x40)
  - 5: **gun shot** (needs |dz| < 6; spawns pool A class 59 bullet at X +- 0x33, Y + 0x46)
  - 6: aim
  - 7: anim 0x10
  - 8: run (speed 0x19)
  - 9/10: atk 0x15 (9)
  - 11: atk 0x16 (13)
  - 12: jump
  - 13: recovery pause `B[0x436D4 + (level & 0x18) + B[0x436B4 + (rng()&31)]]`
- **Combos**:
  - 0 [0,2,2,3]
  - 1 [0,3]
  - 2 [0,6,5,1,5,1,5] gun volley
  - 3 [0,6,7,6]
  - 4 [4]
  - 5 [8,2,2,2]
  - 6 [8,3]
  - 7 [4,3]
  - 8 [2,2]
  - 9 [12]
  - 10 [0,6,5,1,5,0,3]
  - 11 [0,2x6]
  - 12 [0,2,4]
- **Summon** (Vice T. only, `0x44EBA`): after a knockdown, when `+$C2 < 5` and HP <= maxHP / `+$C2`. Then `+$C2 += 1`, so it fires at 1/2, 1/3 and 1/4 of max HP.
  The boss jumps back and calls (state A): `$502 = 1`, which enables the controller's threshold list.
- **Rage** (`0x42B52`): while `+$A8 > 0`, when HP <= `+$6A`/2 and then <= `+$6A`/4. Because `+$6A = 100` this means HP <= 50, then HP <= 25.
  It sets `+$BD = 2` and a 1800-frame rage, in which it attacks constantly.
- **Reinforcement controller** (pool B class 22, 0x7BEF6):
  - **Timed list** (per stage 0x7BF36): spawns a record every `delay` frames only while `$4F3 < [4,5,6][players-1]` (0x7BFEE). A record is skipped, not retried, when the limit is reached.
    Stage 1 timed list (0xB2144): 480: GNEISS (P>=2); 240: DRIVER (P>=2); 600: FERRIS (P3); 0: DRIVER (P3); 1800: FERRIS.
  - **Threshold list** (0x7BF56): wave-like groups. It runs only after a summon (`$502 > 0`). Stage 1 (0xB21A4) has 6 groups: thugs, hammers, class 63/64, blades, then mixes.
- **End of the fight**: on the first frame with boss HP 0 (controller check) → controller `+$4 = 6`, `$7A9 = 0`, `area_transition`.
  The boss hit handler (0x4385C) at HP 0 calls 0xACEE. That sets `$4E0 = boss`, `$50E = 1` (all helpers die and their score goes to the boss's killer), `$4D1 = 2` (stage clear flow), and rank speed +1.
- **Death** (+$4 = 4, 0x43892): kill score (Vice T. 10000, Walther 8000). The killer becomes the walk-off leader. The Vice T. body stays (never freed). Walther blinks 120 frames and is freed.
- Boss HP indices for the other bosses: 0 Slice/Slisaur/Tyrog(alt), 1 Butcher, 2 Hogg, 4 Morgan, 5 Morgue, 6 Tyrog, 8/9 Fessenden (`boss_hp.txt`).

---------------------------------------------------------------------------------------------------

## 13. Items and breakables (pool E)
Analysed by a helper agent (`items_*.py`, `items_tables.txt`, `items_drops.txt`); spot-checked (eat routine 0x28AA4, table 0x28B10). **[O]** unless tagged.

### 13.1 Pool E classes (dispatch 0x233F0, updated before pool A)
| class (+$20) | handler | object |
|---|---|---|
| 0 (0x00) | 0x24D44 | weapon (15 types, `+$26 = type*2`) |
| 1 (0x04) | 0x27E16 | explosion (`+$26` = kind 0..4; spawned by 0x27DD8) |
| 2 (0x08) | 0x283DE | ammo box (kind 0x0100) |
| 3 (0x0C) | 0x2860A | bullet from a held gun (0x26414) [I] |
| 4 (0x10) | 0x2898C | food (`+$26` = 1..18) |
| 5 (0x14) | 0x28C76 | breakable, liftable, throwable container (stage scripts) |
| 6 (0x18) | 0x29B06 | helper object for weapon type 11 [U purpose] |
| 7 (0x1C) | 0x29D1E | gun-fire side effect [I] |
| 8 (0x20) | 0x29EC4 | ballistic projectile from pool B class 18 [I] |
| 9 (0x24) | 0x2A028 | treasure (`+$26` = 1..16) |
| 10 (0x28) | 0x2A39A | gun projectile variant [I] |
| 11 (0x2C) | 0x2A534 | invisible attack box (atk 0x6E/0x6C, player special) [I] |

The item kind word `+$A0`:
- 0x00tt = food, 0x0100 = ammo, 0x02kk = weapon/container, 0x03tt = treasure.
- Lower kind = higher pickup priority.

### 13.2 Containers (pool E class 5)
- Init:
  - `+$26` variant 0..0xC picks the animation set (0x2995E..0x29AC8).
  - kind 0x020A, box table 0x100000.
  - **HP 4** (explosive variant: `+$5C != 0` → HP 50, fuse).
  - `+$58` thrown attack id from 0x29860 (0x3C..0x44).
  - Home X/Z saved for the hit shake (X = home +-1 while `+$59 > 0`).
- Lists: variants 0/2/4/0xC are attackable, solid and liftable. 6/8 are attackable and liftable (not solid). 0xA is attackable and solid (not liftable).
- Explosive fuse: 4 stages with blink periods `[20,10,5,2]` and `[2,3,3,6]`, then explosion (0x27DD8). The barrel shimmers every 50 frames.
- **Break** (0x296AC, by variant): +500 points (score offset 0x24) to the credited player, **drop roll 0x24B48 (table 0xB64F2)**, then debris.
  Variant 0 flies off: vx = +-0x300, vy = 0x400, ay = -0x20, then bounces with vy = -vy>>1, blinks 40 frames.

### 13.3 Drop roll (enemies 0x24B36 → table 0xB6D12; containers 0x24B48 → table 0xB64F2)
```rust
fn spawn_drop(src, table) {
    let id = src.b96; if id == 0 { return }
    let mut row = table + id as u32 * 32;
    loop { src.b96 = 0;                                   // tried once
        let code = B[row + (rng() & 0x1F)];
        match code {
          0 => return,
          0x0F => { if gun_exists_on_screen() { spawn_item(0x08 /*ammo*/, src, None); return }   // types 0,1,3,4,5,6 lying/held, x+0x18 >= camX
                    row = 0xB64D2; continue }               // fallback row: NECKLACE 4, RING 4, WATCH 4, DIME 12, 1000$ 6, SUNGLASS 2 (/32)
          1..=0x13 => { spawn_item(0x00, src, Some((code-1)*2)); return }    // weapon type code-1
          0x14..=0x27 => { spawn_item(0x10, src, Some(code-0x13)); return }  // food 1..20
          _ => { spawn_item(0x24, src, Some(code-0x27)); return } } }       // treasure
}
fn spawn_item(cls, src, v26) { if let Some(o) = alloc_poolE() {         // full pool = no drop
    o.active = 1; o.w20 = cls; o.x = src.x; o.y = src.y; o.z = src.z;      // integer words only
    if let Some(v) = v26 { o.var = v } o.r6 = 2 } }                        // "spawned airborne"
```
- Enemy drops happen at the killing blow (knockdown/thrown/burn handlers). Stage 1 enemy rows:
  - 0x04: 24/32 nothing, else one of 8 treasures
  - 0x15: half weapon type 12
  - 0x19: 4/32 ammo-or-treasure
  - 0x25: weapon 13
  - 0x27: UZI 7, SHOTGUN 24, BAZOOKA 1
- The same `+$96` value means different things for enemies and containers (different tables).
- **Spawn motion**: airborne, Y += 0x30, vy = 0x300, ay = -0x30. On landing `land_check` (0x24CE6):
  - near the screen edges (X <= camX+0x10 or >= camX+0x170) it bounces back in with vx = 0x180 / -0x100, up to 5 bounces;
  - otherwise it stops dead.
  QUIRK: only the first bounce halves vy.

### 13.4 Item values
- **Food** (heal, or points when HP is already 100):

  | food | heal | points at full HP |
  |---|---|---|
  | BARBECUE | 100 | 10000 |
  | STEAK | 80 | 10000 |
  | LOBSTER, SUSHI | 64 | 10000 |
  | BURGER, HOTDOG, PIZZA | 48 | 5000 |
  | SALAD | 32 | 5000 |
  | CAKE, PAFFE, PUDDING, POTATO | 32 | 3000 |
  | GUM, CHOCOLATE, DONUT, COFFEE, CROISSANT | 16 | 1000 |
  | type 18 | 36 | 4000 |

  Table 0x28B10 = (heal, score offset). HP is capped at 100.
- **Treasure points**:
  - GOLD, DIAMOND, THE CUP, EGG = 10000
  - GOLDDUST, DINOBONE, AMMONITE, PEARL = 5000
  - NECKLACE, RING, WATCH, RADIO = 3000
  - DIME, 1000$, SUNGLASS, GROVE = 1000

  Stage 3 RADIO triggers kill-all (`$50E = 1`).
- **Weapons** (init table 0x24D6C; name table 0x23C58 for guns):
  - Guns (with ammo): RIFLE 6, GUN 6, UZI 48, M-16A1 48, SHOT GUN 6, BAZOOKA 4.
  - Melee kinds 0x202/0x203: 6 or 10 uses. Thrown kinds 0x201: 10 uses. Type 12 is a fused explosive.
  - Ammo box: refills a held matching gun to max (+1000 points if already full). Otherwise +1000 points, and the next gun picked up is filled.
- **Weapon wear**: each use adds 0x20 to `+$A6`. A weapon dropped by a hit breaks when bit `rng() & 31` of the mask `L[0x2CB80 + 4*(wear >> 5)]` is set. The masks have 0, 6, 12, 16, 20, 24, 28 and 32 bits set (chance out of 32).
  An empty RIFLE becomes a club (atk 0x32). An empty GUN becomes a throwable.
- Lifetime: a dropped weapon lasts 3600 frames. Placed weapons never expire. Items are freed off screen (0x173C / 0x1760).

### 13.5 Pickup
- There is **no auto-pickup**. On a new press of the attack button (player `+$A8` bit 0x10, handlers 0x1AE0/0x1C64), `find_pickup` (0x5AA2) scans the pickable list `$69EE`.
  An item qualifies when it is idle on the ground, the player's (X, Z) is inside the item's first hit box, and |dY| <= 8. For heavy containers the stick must also point toward the box.
  The lowest kind wins, and later items win ties. There is a 40-frame lockout (`+$F8`) after a weapon pickup.
- If something is found: player state `02 00 00 14`, anim 0xA (0x16 for a box). The item gets `+$2` = player, `+$5 = 4`, `+$70` = player. A weapon replaces the held one (the old one drops).
- If nothing is found, the press attacks normally.
- Food is eaten on the first frame (`eat` 0x28AA4: heal or score, message), then freed after the pickup animation. Treasure adds points (0x2A164) and is freed.

---------------------------------------------------------------------------------------------------

## 14. Answers to the AI design questions (summary)
- **How a mode is chosen**: no central brain. Each class has its own state machine (+$06/+$07).
  Class 2 is typical: approach a slot → at the slot roll attack vs reposition (circle, hover, back off) → attack via a combo script.
  All rolls use `rng()` (0x119C) against tables indexed by variant, aggression and level. Timers and distance windows are listed in section 7. **[O]**
- **Limiting simultaneous attackers**: there is **no attack-token system**. `+$7D` marks attacking enemies and the census counts them per player (`+$E5`), but nothing reads that count (grep of `$e1`/`$e5`). **[O]**
  Crowding is controlled by:
  1. target selection, which prefers a player with < 3 enemies (section 5.2);
  2. 6 different approach slots (3 per side) and slot rotation;
  3. a low per-visit attack chance at low aggression (2/32 for Ferris at aggression 0);
  4. wave thresholds that keep the number on screen down.
  **[O]** The generic names TAIKI/KOUGEKI/ZETTAI in the debug list suggest a wait/attack/must-attack model in other classes. [I]
- **Flanking (MAWARIKOMI)**: the initial slot is chosen from last frame's left/right counts around the target (`retarget_and_slot`), to balance both sides. The "rotate slot" choice (0) walks the enemy round to the next slot. Stuck enemies rotate 2 slots and flip direction. **[O]**
- **Spacing (MAAI)**: state 6 hovers at the slot, re-steering every step frame and pausing `B[0x34186 + level]` frames (85 → 15) whenever it arrives. **[O]**
- **Target choice**: section 5.2. It is re-done on every `reset_ai` (after each combo, recovery or entrance). **[O]**
- **Reaction to hits**: section 10. Flinch while poise > 0, otherwise knockdown. After a flinch the enemy waits `+$7A` frames (level-based, 0x2B8C4) and then rolls an immediate counter-attack (`T_33FA4[aggr]`/16) in state 0. **[O]**
- **Difficulty**: section 6. Level (from rank and player count) scales HP, damage, recovery, pauses and level-gated spawns. Aggression scales speed and attack chances. The player count sets thresholds, extra spawns and boss HP. **[O]**

## 15. Anti-tamper checks [O]
Three routines compare ROM words with an offset copy. If the ROM was modified they quietly break the game. A port using a genuine ROM can ignore them (they always pass).
- `0x2A5B6`: 19 words at 0x564 vs 0x2A5E0 - 0x567. On failure the class 2 taunt adds 8 to the pool A free-list pointer `$6602` (corrupts allocation).
- `0x29ADE` (called by the boss reinforcement controller, pool B class 22, at 0x7C014 when `($0(a5) & 0xFFF) == 0x115`): on failure `$6600 += 0x18` (corrupts the pool B free list).
- `0x104F6`: 0x564 vs 0x10520 - 0x123 (caller not traced).

## 16. Open gaps
- Per-class AI for the other 30+ enemy classes (Punk/Thug #20, Hammer/Elmer/Wrench #10, Blade/Razor #14, Lash, Bludge, Poacher/Skinner/Gutter, Knife, Mack, Zeke, R Hopper, …) and the bosses Slice, Hogg, Butcher, Morgan, Morgue, Slisaur, Tyrog, Fessenden. They use the same library, census, HP, score and hit system; only their state machines differ. **[U]**
- Pool B classes in the scripts (#12, #14, #20, #31, #40, #66, #69, #73, #74, #95 …): props, scenery, lighting and event objects. Only #15 (wave controller) and #22 (boss reinforcements) are analysed. **[U]**
- Terrain flags `+$50`: the shape handler of the tile under the feet stores its wall bits (low byte of shape word 7: 0x01, 0x04, 0x08, 0x10 hop-down ledge; corner shapes force 0x08), the flag byte of the tile class (table 0x13B0C, 0 or 1) is OR-ed in, and bit 7 means the obstacle test (0x12E9A) found a container in the way (obstacle in `+$94`). The game meaning of each bit per stage is still open. **[I]**
- Camera modes other than 0 (follow) and 4 (wave lock): table 0x2092E, modes 1, 6, 7, 0x0B, 0x11, 0x12 appear in stage 1. Their exact rules are left to the stage topic. **[U]**
- The HUD side of the stage time limit (`$4E8`), and whether it is shown. **[U]**
- The animation frame record bytes `+$2C/+$2D/+$2E` (hold, turn, attack-box mode) are used as described, but the full animation format is the animation topic. **[I]**
- `f_939B6` (class 2 init), `0x4B58` (in water?), `0x9796` (landing effect): effects/sound only. **[U]**
- Player-side fields (`+$A3` lives/credit flag, `+$C6` power, `+$A0` move ids) are used by the rank and target code; their exact meaning is the player topic. **[I]**
