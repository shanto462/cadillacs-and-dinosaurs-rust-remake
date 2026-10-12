# Player character logic and combat rules (Cadillacs and Dinosaurs, CPS1 "dino", 68000)

> **Correction (main session, after checking rendered art):** character ids are
> **0 Jack, 1 Mustapha, 2 Hannah, 3 Mess** (the animation tables at 0x163E.. are indexed by `$03`, and
> id 1 renders Mustapha, id 2 Hannah; see `re/out/shots/special_compare.png`). Every per-character value
> in this file is listed in id order and is correct; only the names for ids 1 and 2 were swapped.
> Table headers below are fixed. In the prose, "Hannah" in a `chr == 1` context means id 1 = **Mustapha**
> (so the airborne special 0x1AAE6 is Mustapha's), and "Mustapha" for id 2 means **Hannah**.


Scope: how the 3 player objects move and fight, with the original numbers, written so a native
Rust port can reproduce the logic frame by frame. All addresses are 68000 program addresses
(`out/dino_68k.bin`). `$xx` = offset inside the player object, `G.$xxxx` = global at `$xxxx(a5)`
(a5 = 0xFF8000).

Evidence tags: **[O]** observed in code or data, **[I]** inferred (strong evidence, not proven),
**[U]** unknown / not resolved.

Helper files in this folder:
- `tables.py` + `tables.txt`: dump of every numeric table named here (speeds, jump vectors, attack
  tables, damage scales, timers). Run: `../../.venv/bin/python -I tables.py ../../out/dino_68k.bin`.
- `h` / `hx.py`: hex dump (`./h <addr> <count> b|w|l|jt [base]`), `r` / `rng.py`: print a range of `out/sweep.txt`.

Note: Ghidra (`ghidra/decomp.c`) fails to decompile most player state handlers (they jump into
each other and pop return addresses). All pseudocode below was read from the capstone listing.

---------------------------------------------------------------------------------------------------

## 0. Conventions used by the pseudocode

```rust
// Fixed point (all [O], routine 0x189C):
//   position  X=$08, Y(height)=$0C, Z(depth)=$10 : i32 16.16 (high word = pixels)
//   velocity  vx=$14, vy=$16, vz=$18             : i16 8.8 pixels/frame
//   accel     ax=$1A, ay=$1C, az=$1E             : i16 8.8 pixels/frame^2
fn motion(p) {                      // 0x189C, order X, Y, Z
    p.vx = p.vx.wrapping_add(p.ax); p.x += (p.vx as i32) << 8;
    p.vy = p.vy.wrapping_add(p.ay); p.y += (p.vy as i32) << 8;
    p.vz = p.vz.wrapping_add(p.az); p.z += (p.vz as i32) << 8;
}
// Note: velocity is updated BEFORE it is added (semi-implicit Euler).
// Screen: sprite Y = -(Y + Z) + dy  (higher Y or higher Z = higher on screen). [O] (0x14230)
// "Up" on the stick increases Z (walk table dir U has vz > 0). [O]
// Facing $24: 1 = facing right, 0 = facing left. [O] (0x192A2, 0x19782)
// Hitbox x offsets are stored for facing LEFT; when $24 != 0 they are mirrored. [O] (0x128F8)

const ATK: u8 = 0x10;  // button 1
const JMP: u8 = 0x20;  // button 2
// Direction bits (active high): 1 = right, 2 = left, 4 = down, 8 = up. [O] (direction table 0x19292)

// Global RNG used by all combat code (0x119C), state G.$14 (word). [O]
fn rand() -> u16 {
    let mut s = G.rng14; if s == 0 { s = 0x01C3; }
    let t = s.wrapping_mul(3);
    let lo = (s as u8).wrapping_add((t >> 8) as u8);
    let r = (t & 0xFF00) | lo as u16;
    G.rng14 = r; r
}
// The RNG is shared with enemies and effects: the CALL ORDER matters for exact replays.

// Frame counter G.$0000 (word, G.frame). Used for blink / slow-motion parity.
```

Animation (shared engine, [O]):
- Anim sets per character: `0x13A0` set0 (`0x163E`, moves), `0x13A8` set1 (`0x164E`, attacks),
  `0x13B0` set2 (`0x165E`, hurt; also clears `$25`), `0x13BC` set3 (`0x166E`, grabbing),
  `0x13C4` set4 (`0x167E`, being grabbed), `0x13CC` set5 (`0x168E`, weapons).
  Each set is a long pointer per character (`$03`), then a word offset table indexed by anim number.
- Frame record = 12 bytes: `+0 w` composition, `+2 w` alt index (if >= 0 it selects long
  `0x11D220[idx]` into `$4C`), `+4 l` copied to `$2C..$2F` (with `$2F &= 0x7F`), `+8 w` -> `$48`
  (`$48` body box id, `$49` attack box id), `+A w` -> `$60`.
  - `$2C` per-frame flag used by states as "cancel window / stride / hit tick" [O].
  - `$2E` frame class used by the collision system: 0 = probe (combo-throw candidate), 2 = can
    start a grab, 4/6/8 = attack, 6 = special (also immune), 8/0xA = immune, 0xC = grabbing,
    0xE = grabbed [O for the dispatch tables 0x10820, 0x109B2, 0x1163E].
  - `$2F` frame duration in frames; bit 7 of the record byte +7 marks the last frame.
- `anim_step()` (0x1426): `$2F -= 1`; when it reaches 0, go to next record; if the record just
  finished had bit7 set, read the word after it (relative offset), jump there (loop) and return
  **ended = true** (carry). Otherwise ended = false.

```rust
fn anim_step(p) -> bool {           // 0x1426
    p.dur = p.dur.wrapping_sub(1); if p.dur != 0 { return false; }
    let cur = p.frame_ptr; let next = cur + 12;
    if rom[cur + 7] & 0x80 != 0 { let off = rd16s(next); load_frame(p, next + off); return true; }
    load_frame(p, next); false
}
```

---------------------------------------------------------------------------------------------------

## 1. Player object layout (0x180 bytes; P1 `G.$3274`, P2 `G.$33F4`, P3 `G.$3574`)

Init: `0x4D46/0x4D5E/0x4D76` -> `0x4D8E` clears 0x180 bytes, then `$6A=$6C=$6E=100`, `$24=1`,
`$10A=0x33`, calls `0x11738` (power factor), sets `$02` index and `$5E=index*0x10`;
`0x4DD6`: `$A3 = DIP(G.$777C) & 3` (extra lives) [O].

| off | size | meaning | tag |
|---|---|---|---|
| $00 | b | presence/vulnerability: 0 = slot free, 1 = active and hittable, 2 = active but **invulnerable** (bit1 skips all hit tests) | O |
| $02 | b | player index 0..2 | O |
| $03 | b | character: 0 Jack, 1 Mustapha, 2 Hannah, 3 Mess (verified by art; see correction) | O |
| $04..$07 | 4b | state: mode, sub, subsub, state (written as one long, e.g. 0x02000004) | O |
| $08 / $0C / $10 | l | X / Y height / Z depth, 16.16 | O |
| $14 $16 $18 | w | vx vy vz, 8.8 | O |
| $1A $1C $1E | w | ax ay az, 8.8 | O |
| $20 | w | character*4 (index for per-char long tables) | O |
| $22 | b | move direction*4 (0=U,4=UR,8=R,0xC=DR,0x10=D,0x14=DL,0x18=L,0x1C=UL), 0xFF = none | O |
| $23 | b | palette override (bit7 = active), e.g. 0x84 burning, 0x8A flash | O |
| $24 | b | facing (1 right, 0 left) | O |
| $25 | b | anim flags: bit0 = weapon overlay layer active, bit3 = hidden (death blink) | O/I |
| $28 | l | current anim frame pointer | O |
| $2C..$2F | 4b | frame flag, ?, frame class, duration counter (see §0) | O |
| $30..$3E | | second (weapon overlay) anim layer, stepped by 0x1542 | O |
| $44 | l | hitbox table pointer = `long[0x18CC4+$20]` (0x100000 + char*0xC00), 12-byte boxes | O |
| $48 / $49 | b | body box id / attack box id (0 = none) | O |
| $4A | b | box test mode (jump table 0x1290E) | O |
| $50 | b | wall contact bits this frame (from ground/wall collision 0x12CAC) | O |
| $51 | b | airborne flag | O |
| $53 | b | in water (splash effects) | I |
| $54 / $56 | w | floor height / floor offset; floor = $54+$56 | I |
| $58 | b | current attack id (even); attack data entry = $58*4 | O |
| $59 | b | hit-stop counter (set on attacker and victim by a hit) | O |
| $5A | b | "force knockdown" flag copied from the attacker | O |
| $5E | b | serial number of the current swing (from global counter G.$67BE) | O |
| $64 | l | walk/run speed table pointer (0x5036) | O |
| $68 | w | last attacker object | O |
| $6A | w | 100 (max vitality, only read by HUD code) | I |
| $6C | w | vitality (HP), 0..100 | O |
| $6E | w | HP before the last hit (HUD drain) | O |
| $70 | w | grab partner / combo-throw candidate enemy | O |
| $72 / $73 | b | hold time this object allows when grabbed (0x3C) / escape countdown while grabbing | O |
| $78 | b | stun meter, reset to 10; reaching 0 forces a knockdown | O |
| $7B | b | last hurt reaction code (4 = was knocked down) | O |
| $80 $81 $90 | | state-local counters | O |
| $A0 / $A1 / $A8 | b | input held / previous / newly pressed | O |
| $A3 | b | extra lives | O |
| $A7 / $AB | b | last facing / frames since facing changed (saturates at 0x70..0x7F) | O |
| $A9 | b | grab push-away timer (120) | O |
| $AA | b | frames attack held (stops counting when bit7 set) | O |
| $AC | b | previous run direction | O |
| $AD | b | buffered attack press (combo) | O |
| $AE | b | combo window timer (set to 0x28) | O |
| $AF | b | wall contact latch for wall jump | O |
| $B2 | b | combo hit count (0..4) | O |
| $B3 | b | post-whiff movement lock (5) | O |
| $B4 / $B6 | w | held item object / held item type (0x2xx) | O |
| $B8 / $BA | w | pickup candidate object / type (0x8000 = none) | O |
| $BC / $BD | b | double-tap timer (15) / input nibble at last walk start | O |
| $BE / $BF | b | mash meter / mash cooldown | O |
| $C4 / $C5 | b | weapon attack active / drop weapon request | O |
| $C6 | b | attack power factor (x/32), random, depends on own HP | O |
| $C8 | b | action class for other code (0x10 attack, 0x11 special, 0x13 dash, 0x14 jump attack, 0x20 hurt, 0x21 dizzy, 0x23 knockdown, 0x30 grabbing, 0x40 grabbed, 1 jump, 2 run jump) | O |
| $C9 | b | buffered special request (10 frames) | O |
| $CA / $CB | b | hits taken recently / 60-frame window | O |
| $CC | b | tech flag (always cleared by 0x1D244 in the observed paths) | O |
| $CD | b | 1 = will be dizzy after getup (slow fall), 2 = dying (slow fall) | O |
| $CE | w | dizzy immunity timer (dizzy blocked while high byte != 0) | O |
| $E8 | b | lie-down timer | O |
| $E9 | b | serial of the last attack that hit me | O |
| $EC | w | respawn invulnerability timer (blinks) | O |
| $EE | w | special move HP-cost budget (20) | O |
| $F0 | b | re-grab cooldown (10) | O |
| $F1 / $F4 / $F5 | b | command-move detected / command FSM state / FSM timer | O |
| $F6 | w | stun-meter refill timer (120) | O |
| $F8 | b | weapon re-pickup cooldown (40) | O |
| $F9 | b | invulnerability timer | O |
| $FA / $FC | w | run time left (240) / run cooldown (5) | O |
| $100 | l | score, BCD, 8 digits (HUD 0x101B6) | O (format), U (scoring rules) |
| $10E $11C $11D | | timed special status (enters state 0x18 / 0x418 / 0x41A) | U |
| $115 | b | knee counter while grabbing | O |
| $117 | b | turn around after pushback (hit from behind) | O |
| $11A | b | input lock (inputs forced to 0) | O |

---------------------------------------------------------------------------------------------------

## 2. Frame order and input

Per in-game frame (main loop 0xAAF6, order from coordinator + [O] at 0xE90E):
1. 0x186CC stage events.
2. 0xE90E object updates. Inside it, first `0x18834` = **all players** (P1, P2, P3), then items,
   enemies etc., then `0x188DA` = players that are currently held by an enemy (`$2E == 0xE`).
3. 0x10546 **collision / hit resolution** (applies all hits of this frame, see §6).
4. HUD, sprite list, etc.

So a hit detected on frame N changes the victim state at the end of frame N; the reaction code runs on frame N+1.

```rust
fn players_update() {                      // 0x18834
    irq_mask(true);                         // trap #12, d0=1
    G.snap[0..3] = [G.in16, G.in17, G.in18];   // $77A4..$77A6 (cooked inputs, active high)
    if !G.b4CC { for p in players { demo_record_or_playback(p); } }  // 0x9A146 [I]
    irq_mask(false);
    G.b77A0 = 0; /* misc */ 
    for p in players { if p.b00 != 0 && p.frame_class != 0xE { player_frame(p); } }  // 0x18952
}

fn player_frame(p) {                       // 0x18952
    p.in_prev = p.in_held;                  // $A1 = $A0
    let cur = G.snap[p.index];
    p.in_held = cur;                        // $A0
    p.in_new  = cur & (cur ^ p.in_prev);    // $A8  (rising edges only, no buffering)
    // (debug-only auto-fire / HP drain hooks at 0x18992..0x189CA when G.$4AA != 0)
    if G.b4DD != 0 { if p.bD0 == 0 { p.bD0 = G.b4DD; scripted_start(p) /*0x1DB0C*/ } scripted(p) /*0x1DB5E*/ }
    if p.b11A != 0 { p.in_held = 0; p.in_prev = 0; p.in_new = 0; }
    // G.$518 / G.$516 special-mode handling at 0x18A02..0x18A2C [U]
    match p.mode /*$04*/ { 0 => spawn(p) /*0x18A82*/, 2 => alive(p) /*0x18CD4*/,
                           4 => dead(p) /*0x1BF42*/, 6 => remove(p) /*0x18EB0*/ }
    if G.b518 == 0 {
        register_for_collision(p);          // 0x4BC0 (list G.$6720, max 3)
        if !(p.frame_class == 0xE && p.action != 0x12) && p.anim_flags & 8 == 0 { register_for_draw(p) } // 0x49CA
    }
}
```
Input notes [O]: there is no input queue. "Newly pressed" is a 1-frame edge. The only buffers are:
`$AD` (attack press during hit-stop, §4.4), `$C9` (special pressed during hurt, 10 frames), the
double-tap timer `$BC`, and the command FSM `$F4/$F5`.

---------------------------------------------------------------------------------------------------

## 3. Alive update and the state machine

### 3.1 `alive()` 0x18CD4 (runs every frame while `$04 == 2`) [O]

```rust
fn alive(p) {
    if p.wEC != 0 { if p.wEC & 3 == 0 { blink_fx(p) /*0x958E4*/ }
                    p.wEC -= 1; if p.wEC == 0 && p.bF9 == 0 { p.b00 = 1; } }
    if p.bF9 != 0 { p.bF9 -= 1; if p.bF9 == 0 && p.wEC == 0 { p.b00 = 1; p.b23 = 0; } }
    // attack-hold counter and combo window
    if p.in_held & ATK != 0 {
        if (p.bAA as i8) < 0 { goto skip_ae; }       // saturated: $AE is NOT decremented
        p.bAA += 1;
    } else { p.bAA = 0; }
    if p.bAE != 0 { p.bAE -= 1; }
  skip_ae:
    if p.facing != p.bA7 { p.bA7 = p.facing; p.bAB = 0; }
    p.bAB += 1; if (p.bAB as i8) < 0 { p.bAB = 0x70; }
    if p.bBC != 0 { p.bBC -= 1; }   if p.bB3 != 0 { p.bB3 -= 1; }
    if (p.hp as i16) <= 0 && p.mode < 4 && p.state_w04 != 0x0202 {   // HP drained outside a hit
        p.b59 = 0x10; p.state = 0x02020000; p.w68 = p; p.wB6 = 0; p.b23 = 0; p.b11B = 1; return;
    }
    if p.wF6 != 0 { p.wF6 -= 1; if p.wF6 == 0 { p.stun = 10; } }
    if p.bCB != 0 { p.bCB -= 1; if p.bCB == 0 { p.bCA = 0; } }
    if p.wCE != 0 { p.wCE -= 1; }   if p.bC9 != 0 { p.bC9 -= 1; }
    if p.bF0 != 0 { p.bF0 -= 1; }   if p.bF8 != 0 { p.bF8 -= 1; }
    if p.wFA != 0 { p.wFA -= 1; }
    if !(p.state_w04 == 0x0200 && p.b06 == 2) && p.wFC != 0 { p.wFC -= 1; }  // run cooldown frozen while running
    p.bF1 = 0; command_fsm(p);                       // 0x6AC8, §4.10
    if p.b11C != 0 { p.b11C -= 1; if p.b11C == 0 { p.b10E = 0; } }
    match p.b05 { 0 => normal(p) /*0x19090*/, 2 => hurt(p) /*0x1C02C*/,
                  4 => grabbing(p) /*0x1D652*/, 6 => grabbed(p) /*0x1D2BE*/ }
    if G.b4F0 == 0 { floor_and_walls(p) /*0x12CAC*/ } else { floor_only(p) /*0x12D4E*/ }
    if G.b506 == 0 { clamp_to_screen(p) /*0x4FCE*/ }
}
```

`floor_only` / the floor part of `0x12CAC` [O]:
```rust
let floor = p.w54 + p.w56;
if p.airborne == 0 {
    if p.y_hi > floor { p.airborne = 1; }               // walked off a ledge -> falls
    else if p.y_hi < floor { p.y_hi = floor; p.y_lo = 0; }
} else if p.y_hi < floor { p.y_hi = floor; p.y_lo = 0; p.airborne = 0; }   // landed
```
`0x12CAC` additionally runs up to 5 wall-push iterations (0x12FE4/0x1309C), the obstacle test 0x12E9A and the ceiling test 0x1361E, and sets `$50` [O]. Ported in `game/terrain.rs` (module doc).

`clamp_to_screen` 0x4FCE [O] (G.$744 = camera X, G.$748 = camera Y):
X is kept in `[camX+0x10, camX+0x170]`; `Z+$54` is kept in `[camY+0x10, camY+0xA0]`
(when G.$4DC != 0 the same limits are absolute, without the camera).

### 3.2 State map

`$04..$07` as a long. `normal()` (0x19090) first drops a carried heavy object in scripted
scenes, then dispatches on `$06` (table 0x190C4): 0 = ground (0x190CA), 2 = run (0x19FA8),
4 = air (0x1A58C). Ground and run first check `if airborne -> fall_entry (0x1A848)` (ground skips
this check in states 0x0E and 0x22).

```
mode 2 / sub 0 / ground ($06=0), table 0x190F4          entry      handler
  0x00 idle                                            0x19122 -> 0x1916E
  0x02 walk                                            0x192B8 -> 0x192DE
  0x04 jab (punch 1, attack id 0)                      0x193CE / 0x193DE(alt anim) -> 0x19416
  0x06 hit confirm: hit-stop + combo buffer            0x19464 -> 0x1947E
  0x08 punch 2 (id 0)                                  0x1950E -> 0x1953E
  0x0A punch 3 (id 2)                                  0x1956C -> 0x1959C
  0x0C finisher (id 4)                                 0x195B2/0x19614 -> 0x1965A
  0x0E special / desperation (id 0x12)                 0x1968E -> 0x196F2
  0x10 dash attack, Jack (id 0x14 -> 0x5A -> 0x5C)     0x19720 -> 0x197A2
  0x12 dash attack recovery                            0x19812 -> 0x19826
  0x14 item pickup                                     0x19832 -> 0x198B4
  0x16 weapon attack (generic)                         0x198DC -> 0x19954
  0x18 timed status pose ($10E)                        0x19996 -> 0x19A0A   [U]
  0x1A ?                                                       -> 0x19A7C   [U]
  0x1C gun, standing fire (types 0x206/0x207)          0x19AB0 -> 0x19AEA
  0x1E gun, walking fire                               0x19B60 -> 0x19B90
  0x20 charge weapon (type 0x203)                      0x19C36 -> 0x19C62
  0x22 recoil weapon (type 0x209)                      0x19CE0 -> 0x19D5E
  0x24 command-move dash (id 0x76)                     0x19DDA -> 0x19E24
  0x26..0x2C misc/scripted                             0x19E62..0x19F6A    [U]
mode 2 / sub 0 / run ($06=2), table 0x19FC0
  0x00 run                                             0x19FC6 -> 0x1A012
  0x02 run + melee weapon (0x201/0x202, id 0x16)       0x1A18C/0x1A230 -> 0x1A2F2
  0x04 ? (entered from outside player code)            0x1A330             [U]
mode 2 / sub 0 / air ($06=4), table 0x1A598
  0x00 standing jump        0x1A5D0 -> 0x1A64A     0x02 wall jump (Hannah, Mustapha) 0x1A7A4 -> 0x1A7D8
  0x04 landing              0x1A81A -> 0x1A830     0x06 fall (no jump)               0x1A848 -> 0x1A8D4
  0x08 running jump         0x1A906 -> 0x1A950     0x0A jump attack high/dive (0x0E) 0x1A9A4 -> 0x1AA16
  0x0C jump attack low (0x0C) 0x1AA54 -> 0x1AAA8   0x0E/0x10 Hannah special (0x12)   0x1AAE6 / 0x1AB74
  0x12 dash attack, Hannah/Mustapha/Mess (airborne) 0x1ABD4 -> 0x1AC5A
  0x16 running-jump attack, Jack (0x10 -> 0x70)    0x1AD06 -> 0x1AD32
  0x18/0x1A timed status (Hannah/Mustapha)          0x1ADA6 / 0x1AE62   [U]
  0x1C ? 0x1AF82  0x1E/0x20 jump while carrying 0x20A 0x1B038 / 0x1B0CA
  0x28/0x2A/0x2C running-jump attack Hannah/Mustapha/Mess 0x1B496/0x1B568/0x1B5D4
  0x30 command move (down, up+attack), per character table 0x1B72E      0x1B6EC
  0x36 dive landing                                 0x1BD56
mode 2 / sub 2 = hurt (0x1C02C), $06 = reaction code (table 0x1C03E), §5
mode 2 / sub 4 = grabbing an enemy (0x1D652), §4.8
mode 2 / sub 6 = held by an enemy (0x1D2BE), §5.7
mode 4 = dead (0x1BF42), §5.8;  mode 0 = (re)spawn (0x18A82), mode 6 = removed (0x18EB0)
```

Main transitions (text diagram) [O]:

```
            +---------------- A+J (HP>1), also buffered 10 f from hurt ---------------> SPECIAL --+
            |                                                                                     |
 IDLE <---> WALK --(double tap within 12 f, same dir +-45 deg)--> RUN --ATK--> DASH ATTACK --+    |
  |  ^        |  \--ATK--> JAB --hit--> CONFIRM --ATK--> P2 --hit--> CONFIRM --> P3 --hit-->  |    |
  |  |        |                 \whiff (5 f lock)          ...    CONFIRM --> FINISHER/COMBO THROW
  |  |        \--walk frame ($2E=2) touches enemy--> GRAB HOLD --ATK--> KNEE / (L|R)+ATK --> THROW
  |  |                                                     \--JMP / 120 f push away / escape--> IDLE
  |  +-- landing / recover <-- JUMP (JMP) --ATK held, h>=0x10--> JUMP ATTACK low / high(dive)
  |                             RUN --JMP--> RUNNING JUMP --ATK--> char-specific air attack
  +--(enemy hit)--> HURT: light / pushback --(stun over)--> IDLE
                          \ (HP 0, stun meter 0, $5A, airborne) --> KNOCKDOWN: fly, bounce, lie
                             (mash) --> GETUP --> [dizzy] --> IDLE (+20 f invuln) | HP 0 --> DEAD
DEAD --60 f blink--> lives left ? RESPAWN (drop in, 360 f invuln) : GAME OVER
```

Important control-flow rules for a faithful port [O]:
- Most `*_entry` routines **fall through into their handler in the same frame** (the handler code
  directly follows the entry). The jab entry (0x193CE) is the exception: it ends with `jmp sfx` and
  its handler first runs on the next frame. Punch 2 / punch 3 / finisher entries end with `jsr sfx`
  and then fall through.
- `special_check()` (0x63AA) and the jump-attack / wall-jump checks **pop the caller's return
  address**: when they trigger, the rest of the calling handler does not run.

### 3.3 Idle 0x19122 / 0x1916E [O]

```rust
fn idle_entry(p) {                         // 0x19122
    p.state = 0x02000000;
    p.vx = 0; p.vy = 0; p.vz = 0; p.ax = 0; p.ay = 0; p.az = 0;
    p.anim_flags = 0; p.bC4 = 0; p.bC1 = 0; p.action = 0; p.b72 = 0x3C; p.b23 = 0;
    set_speed_class(p, 0);                 // 0x5036
    idle_anim(p);                          // 0x5BDC: weapon idle anim, else drop item ptrs and anim set0 #0
    idle(p);                               // falls through
}
fn idle(p) {                               // 0x1916E
    if G.b4DD == 0 && p.b10E != 0 { p.b00 = 2; fx_2A4FC(1); return state18_entry(p); }   // [U] meaning
    if p.bC9 != 0 { return special_entry(p); }          // buffered A+J from hurt (§4.6)
    special_check(p)?;                                  // may abort (jump to special)
    let new = p.in_new;
    if p.bAD != 0 { return chain_continue(p); }
    if new & ATK != 0 {
        if p.bB2 != 0 && p.bAE != 0 { return chain_continue(p); }
        if G.b506 != 0 { return jab_entry(p, 0); }
        if p.wB6 != 0x20A { find_pickup(p); if (p.wBA as i16) >= 0 { return pickup_entry(p); } }
        if p.wB6 != 0 { return weapon_attack_entry(p); }
        return jab_entry(p, 0);
    }
    if new & JMP != 0 && !(p.wB6 == 0x20A && p.chr == 2) { return jump_entry(p); }
    if p.bB3 != 0 || p.bAE >= 0x24 { face_from_input(p); }       // movement locked
    else {
        let d = p.in_held & 0x0F;
        if d != 0 {
            if p.wFC != 0 { return walk_entry(p); }
            if p.wB6 != 0x20A {
                let diff = DIR8[d].wrapping_sub(DIR8[p.bBD]).wrapping_add(1) & 0x1F;   // u8 math
                if diff < 3 && p.bBC >= 3 { return run_entry(p); }
            }
            p.bBC = 0x0F; return walk_entry(p);
        }
    }
    if p.wB6 == 0 && G.b4DD == 0 { anim_step(p); }
}
// DIR8 (0x19292): nibble -> 0..7 (0=U clockwise), 0xFF for impossible combos
const DIR8: [u8;16] = [0xFF,2,6,0xFF,4,3,5,0xFF,0,1,7,0xFF,0xFF,0xFF,0xFF,0xFF];
fn face_from_input(p) { let d = p.in_held & 3; if d != 0 { p.facing = d & 1; } }   // 0x192A2 (R wins)
```

Run start rule ("double tap") [O]: a walk start stores the stick nibble in `$BD` and sets `$BC=15`.
`$BC` counts down every frame. If the player returns to idle and presses a direction within
±45° of `$BD` while `$BC >= 3` (about 12 frames), the run starts. `$FC` (5 frames after a run) and
carrying object 0x20A block running.

### 3.4 Walk 0x192B8 / 0x192DE [O]

```rust
fn walk_entry(p) { p.bBD = p.in_held & 0xF; face_from_input(p); walk_reenter(p) }
fn walk_reenter(p) {                       // 0x192C6
    p.state = 0x02000002; p.b80 = 0; p.anim_flags = 0;
    walk_anim(p);                          // 0x5CA6: weapon walk anim, else anim set0 #1
    walk(p)                                // falls through
}
fn walk(p) {                               // 0x192DE
    if p.b53 != 0 && p.y_hi < 0x20 { /* water: stage1-area1 hook 0x60E2; splash sound every 0x18 frames */ }
    special_check(p)?;
    if p.wB6 != 0x20A && p.bAD == 0 && p.bF1 == 2 { return command_move_entry(p); }   // §4.10
    let new = p.in_new;
    if new & ATK != 0 {
        if p.bAE != 0 { return chain_continue(p); }
        if G.b506 != 0 { return jab_entry(p, 0); }
        if p.wB6 != 0x20A { find_pickup(p); if (p.wBA as i16) >= 0 { return pickup_entry(p); } }
        if p.wB6 != 0 { return weapon_attack_entry(p); }
        return jab_entry(p, 0);
    }
    if new & JMP != 0 && !(p.wB6 == 0x20A && p.chr == 2) { return jump_entry(p); }
    let d = p.in_held & 0xF;
    if d == 0 { return idle_entry(p); }
    if p.anim_flags & 1 != 0 && anim2_step(p) /*0x1542*/ { return walk_reenter(p); }  // re-runs walk() this frame
    walk_move(p, d);
}
fn walk_move(p, d) {                       // 0x1BD94
    face_from_input(p);
    let (off, dir) = WALKDIR[d];           // 0x1BDCC: byte offset into speed table, $22 value
    p.dir22 = dir;
    p.vx = rd16s(p.speed_tab + off); p.vz = rd16s(p.speed_tab + off + 2);   // ax, az untouched (0)
    motion(p); anim_step(p);
}
// WALKDIR: nibble -> (off,dir): 1R(8,8) 2L(0x18,0x18) 4D(0x10,0x10) 5DR(0xC,0xC) 6DL(0x14,0x14)
//          8U(0,0) 9UR(4,4) 0xA UL(0x1C,0x1C), others (0,0xFF)
```
Note: `walk_move` with an impossible nibble (e.g. L+R) still reads entry 0 (U) for speed. [O]

Speed tables (`set_speed_class(p,c)` 0x5036: `$64 = long[0x5064 + (c + $20)]`, c in {0,4,8,12};
when holding 0x20A the base is 0x50A4) [O]. Values 8.8 px/frame, (vx, vz) for R and U; diagonals in `tables.txt`:

| class | use | Jack | Mustapha | Hannah | Mess |
|---|---|---|---|---|---|
| 0 | walk | R 414, U 236 | 435, 248 | 422, 241 | 409, 234 |
| 4 | run, normal frames | 844, 482 | 870, 497 | 896, 512 | 819, 468 |
| 8 | run, frames with `$2C != 0` | 1868, 1067 | 1894, 1082 | 1920, 1097 | 1843, 1053 |
| 12 | run sub-state 4 only (0x1A3A2) [U] | 2124, 1214 | 2150, 1228 | 2176, 1243 | 2099, 1199 |
| heavy | carrying object 0x20A | 389, 222 | 409, 234 | 396, 226 | 384, 219 |

### 3.5 Run (dash) 0x19FC6 / 0x1A012 [O]

```rust
fn run_entry(p) {                          // 0x19FC6
    anim0(p, 2); p.wFA = 0xF0; p.wFC = 5;
    p.anim_flags = 0; p.action = 0; p.b80 = 0;
    weapon_overlay_setup(p);               // 0x1A0F0
    p.state = 0x02000200; p.bA9 = 3;
    walk_move(p, p.in_held & 0xF);         // NOTE: first frame also moves at the previous speed class
    p.bAC = p.dir22;
    run(p)                                 // falls through
}
fn run(p) {                                // 0x1A012
    special_check(p)?;
    if p.wFA == 0 { return idle_entry(p); }                 // 240 frames max
    /* water splash or footstep sound timer $80 (0x14 / 0x0C frames) */
    set_speed_class(p, if p.b2C != 0 { 8 } else { 4 });
    if p.in_new & ATK != 0 { return run_attack(p); }        // 0x1A174
    if p.in_new & JMP != 0 { return running_jump_entry(p); }
    if G.b4DD != 0 && p.in_held & 0xF == 0 { return idle_entry(p); }
    p.bAC = p.dir22;
    // 0x1BDEC: with a direction held behave like walk_move; with NO direction keep running in $22
    let d = p.in_held & 0xF;
    if d != 0 { face_from_input(p); let (o, dir) = WALKDIR[d]; p.dir22 = dir; set_v(p, o); motion(p); anim_step(p); }
    else if (p.dir22 as i8) >= 0 { set_v(p, p.dir22); motion(p); anim_step(p); }
    if (p.dir22 as i8) >= 0 && ((p.dir22.wrapping_sub(p.bAC).wrapping_add(4)) & 0x1C) > 8 {
        p.bAA = 0; return idle_entry(p);                    // turned more than 45 degrees
    }
    if p.anim_flags & 1 != 0 { anim2_step(p); }
}
fn run_attack(p) {                         // 0x1A174
    match p.wB6 { 0x201 | 0x202 => run_weapon_attack(p) /*state 0x202, id 0x16*/, _ => dash_attack_entry(p) }
}
```
Releasing the stick does **not** stop a run (outside scripted scenes): it ends on timeout (240 f),
a turn of more than 45°, attack, jump, or falling. [O]

### 3.6 Jumps [O]

```rust
fn jump_entry(p) {                          // 0x1A5D0
    if p.wB6 == 0x20A { return heavy_jump_entry(p); }      // 0x1B038, velocity class 12
    p.state = 0x02000400; p.airborne = 1;
    face_from_input(p);
    p.dir22 = WALKDIR[p.in_held & 0xF].1;                  // table 0x1A6E8, same values
    set_jump_velocity(p, 0);                                // 0x5364
    let anim = if (p.dir22 as i8) < 0 || p.dir22 & 4 != 0 { 3 }      // neutral or diagonal
               else if p.bAB >= 3 { 4 } else { p.facing ^= 1; 5 };   // just turned: turn-jump anim
    p.anim_flags = 0; anim0(p, anim);
    p.bAF = 0; p.b83 = 0; p.b80 = 0; p.action = 1;
    jump(p)
}
fn set_jump_velocity(p, class) {            // 0x5364
    let t = rd32(0x53A2 + class + p.w20);   // class in {0,4,8,12}
    let i = if (p.dir22 as i8) < 0 { 0 } else { p.dir22 as usize / 4 + 1 };   // 0 = no direction
    let e = t + 12 * i;                     // (vx, ax, vy, ay, vz, az)
    p.vx = rd16s(e); p.ax = rd16s(e+2); p.vy = rd16s(e+4); p.ay = rd16s(e+6); p.vz = rd16s(e+8); p.az = rd16s(e+10);
}
fn jump(p) {                                // 0x1A64A (standing jump)
    if (p.vy as i16) < 0 {
        splash_check(p);                    // 0x606E (water only)
        if p.airborne == 0 { p.vy = 0; p.ay = 0; return landing_entry(p); }
    } else if p.b80 == 0 {
        let h = p.y_hi - p.w54 - p.w56;
        if (h as u16) >= 0x30 {
            let mut d1 = p.in_held & 3;
            if d1 != 0 {
                let d2 = p.in_prev & 3; if d2 != 0 && d2 != d1 { d1 ^= 3; }
                if (p.dir22 as i8) < 0 { p.b80 = 1; anim0(p, if d1 == p.facing { 7 } else { 0xC }); }  // cosmetic
            }
        }
    }
    anim_step(p); motion(p);
    air_control(p);                         // 0x1BE3C
    jump_attack_check(p)?;                  // 0x1A704
    wall_jump_check(p)?;                    // 0x1A744
}
fn air_control(p) {                         // 0x1BE3C (overwrites $22!)
    let (off, dir) = WALKDIR[p.in_held & 0xF]; p.dir22 = dir;
    if (dir as i8) >= 0 { p.vx += AIRCTL[off/4].0; p.vz += AIRCTL[off/4].1; }
}
// AIRCTL (0x1BE66) per frame: U(0,+1) UR(+1,+1) R(+2,0) DR(+1,-1) D(0,-1) DL(-1,-1) L(-2,0) UL(-1,+1)
fn landing_entry(p) { anim0(p, 6); p.state = 0x02000404; sfx(0x9C0C); landing(p) }   // 0x1A81A
fn landing(p) { if anim_step(p) { return idle_entry(p); }                            // 0x1A830
                if p.in_held & 0x3F != 0 { return idle_entry(p); } }               // any input cancels
```
Gravity is the `ay` of the jump table (constant). Standing jump class 0 ([O], 8.8 units):

| char | neutral vy | R vx / vy | U vz / vy | ay |
|---|---|---|---|---|
| Jack | 1741 | 608 / 1616 | 128 / 1664 | -74 |
| Mustapha (id 1) | 1740 | 608 / 1741 | 128 / 1741 | -74 |
| Hannah (id 2) | 2112 | 528 / 1952 | 176 / 2048 | -88 |
| Mess | 1672 | 608 / 1552 | 128 / 1600 | -72 |

(Example: Jack neutral jump peak = 1741²/(2·74)/256 ≈ 80 px, air time ≈ 2·1741/74 ≈ 47 frames.)
Class 4 = running jump (has ax for UR/R etc.), class 8 = wall jump, class 12 = jump while carrying 0x20A: full values in `tables.txt`.

Running jump 0x1A906 [O]: `state=0x02000408`, airborne, keeps old vx/vz if `$C1 != 0`, else
`set_jump_velocity(p, 4)` with the current run direction `$22`; anim set0 #8; `$C8=2`. Handler
0x1A950: attack newly pressed -> character-specific running-jump attack (0x1ACE8: Jack 0x416,
Hannah 0x428, Mustapha 0x42A, Mess 0x42C); landing -> `landing_entry`; else anim, motion,
`wall_jump_check` (no air control).

Fall 0x1A848 [O]: if coming from run, apply run speed once; `state=0x02000406`, airborne,
`vy = min(vy, 0)`, `ay = -0x20`, `vx >>= 1`, `vz >>= 1` (arithmetic); anim set0 #4. Handler:
on landing -> sound, `idle_entry` (no landing anim); otherwise only `motion()` (anim frozen).

Jump attacks, `jump_attack_check` 0x1A704 [O] (called from standing jump and wall jump only;
triggered by attack **held**, not newly pressed):
```rust
if p.in_held & ATK != 0 {
    let h = (p.y_hi - p.w54 - p.w56) as i16;
    if h >= JA_MIN[chr] { abort_caller();
        if h >= JA_SPLIT[chr] { jump_attack_high_entry(p) } else { jump_attack_low_entry(p) } }
}
// JA_MIN = 0x10 for all; JA_SPLIT: Jack 0x4C, Hannah 0x4B, Mustapha 0x5D, Mess 0x48 (0x1A734)
```
- Low (0x1AA54): `state 0x0200040C`, anim set1 #0xB, face toward vx sign, attack id 0x0C, `vz=az=0`, keeps vx/vy. Ends on landing -> `landing_entry`.
- High / dive (0x1A9A4): `state 0x0200040A`, anim set1 #0xA, face from input, attack id 0x0E,
  `vx = ±DIVE[chr].vx` (Jack 0x500, Hannah 0x680, Mustapha 0x500, Mess 0x480), `ax = ±0x20` (same sign as vx),
  `vy = DIVE[chr].vy` (Jack/Mustapha -0x200, Hannah/Mess -0x1C0), `ay = -0x20`, `vz=az=0`.
  Ends on landing -> dive landing (0x1BD56, anim set0 #0x1E, not cancelable).
- Both handlers: `if $59 != 0 { $59 -= 1; return }` (hit-stop freezes them), then landing check, anim, motion.

Wall jump 0x1A744 (Hannah and Mustapha only) [O]: `w = $50; if w == 0 { w = $AF }; $AF = w;`
if `w` is 4, 0x18 or 0x1C and jump newly pressed and `vx != 0` and height >= 0x20 -> state
0x02000402, anim set0 #0x12, face away from the wall (vx < 0 -> face right, `$22 = 8`; else face left,
`$22 = 0x18`), `set_jump_velocity(p, 8)` (Hannah: vx ±1536, vy 896; Mustapha: vx ±1152, vy 1280).

---------------------------------------------------------------------------------------------------

## 4. Attacks

### 4.1 Common attack start

Every attack start does `$58 = id; $59 = 0; $5E = G.$67BE; G.$67BE += 1;` [O]. The serial `$5E`
makes one swing hit each enemy at most once; a new serial is taken whenever a multi-hit move
wants to hit again (dash attack, knees, multi-hit air moves).

### 4.2 Attack data (player -> enemy) [O]

`entry = long[0xA26FC + $20] + $58*4`, 8 bytes:
`+0 damage, +1 stun (subtracted from victim $78), +2 reaction code (victim $06), +3 hit-stop frames
(both sides $59), +4 sound (0 = none), +5 hit effect param (0xFFC8), +6 spark type (0x12720), +7 -> victim $9A`.
Table bases differ in length: Jack 0xA270C (ids ≤ 0x76), Hannah 0xA28EC (≤ 0x7A), Mustapha 0xA2ADC (≤ 0x76), Mess 0xA2CBC (≤ 0x78).

Main values (dmg / stun / reaction / hit-stop), all characters in `tables.txt`:

| id | move | Jack | Mustapha | Hannah | Mess |
|---|---|---|---|---|---|
| 0x00 | jab, punch 2 | 10/1/00/9 | 10/1/00/9 | 10/1/00/8 | 12/1/00/9 |
| 0x02 | punch 3 | 15/3/02/9 | 15/3/02/7 | 15/2/02/8 | 17/2/02/9 |
| 0x04 | finisher | 29/11/06/13 | 29/11/06/13 | 29/11/06/12 | 32/11/04/13 |
| 0x06 | grab knee 1 | 10/3/00/8 | 10/3/00/8 | 10/3/00/8 | 15/2/00/8 |
| 0x08 | grab knee 2+ | 15/5/02/8 | 15/5/02/8 | 15/5/02/8 | 20/5/02/8 |
| 0x0A | throw | 22/11/02/2 | 20/11/02/2 | 24/11/02/2 | 29/11/02/2 |
| 0x0C | jump attack low | 8/10/10/6 | 8/10/10/6 | 10/10/10/6 | 10/10/10/6 |
| 0x0E | jump attack high (dive) | 15/10/12/6 | 15/10/12/6 | 15/10/12/6 | 15/10/12/6 |
| 0x10 | running-jump attack | 15/6/02/10 | 15/6/00/10 | 15/11/04/10 | 20/1/02/6 |
| 0x12 | special | 20/11/04/7 | 20/1/04/3 | 20/11/04/5 | 23/1/04/5 |
| 0x14 | dash attack (1st part) | 12/5/02/8 | 12/5/14/6 | 12/5/14/6 | 12/5/0E/4 |
| 0x5A | dash attack (2nd hit tick) | 20/6/14/4 | 20/6/14/4 | 20/6/18/4 | 20/6/0E/4 |
| 0x5C | dash attack (3rd+ tick) | 32/11/06/6 | 32/11/06/4 | 34/11/04/6 | 32/11/04/4 |
| 0x18 | escape attack when held | 15/11/06/8 | 15/11/06/8 | 20/11/06/8 | 20/11/06/7 |

### 4.3 Damage formula, player -> enemy (0x2B904) [O]

```rust
fn player_damage(att: &Player, vic: &Enemy, base: u8) -> u16 {
    G.w4F4 = 0;
    // debug: G.$4AA && G.$24 bit1 -> 0 ; G.$24 bit2 -> 100
    let mut d: u32 = base as u32;
    if vic.frame_class == 4 {                                  // counter hit: enemy is attacking
        let m = rd16(0xA320C + vic.atk58 as u32 * 2);           // x/32, 0 = none
        if m != 0 { d = ((d * m as u32) as u16 >> 5) as u32; G.b4F4 = 0xFF; }
    }
    let w = 0xA32FA + vic.w84 as u32;                          // per enemy type: (id/2, mult, id/2, mult)
    let m = if rd8(w)*2 == att.atk58 { Some(rd8(w+1)) }       // ids compared after doubling (8-bit add)
            else if rd8(w+2)*2 == att.atk58 { Some(rd8(w+3)) } else { None };
    if let Some(m) = m { d = (((d as u16 as u32) * m as u32) as u16 >> 5) as u32; G.b4F5 = 0xFF; }
    d = (d as u16 as u32) * att.power_c6 as u32;               // $C6, x/32
    d = (d as u16 as u32) * rd8(0x2BECE + vic.w84*8 + G.w5EE) as u32;   // rank x enemy type, x/32 (all 0x20 seen)
    d >>= 10; if d == 0 { d = 1; }
    let (mut hp, mut row) = (vic.hp, 0u16); while hp > 100 { row += 0x40; hp -= 100; }
    d = (d as u16 as u32) * rd16(0x2BC4E + row + (rand() & 0x3E)) as u32;
    let band = if hp >= 75 {0} else if hp >= 50 {0x40} else if hp >= 25 {0x80} else {0xC0};
    d = (d as u16 as u32) * rd16(0x2BB4E + band + (rand() & 0x3E)) as u32;
    let r = (d as u16) >> 10;                                  // 16-bit truncation BEFORE the shift (lsr.w)
    if r == 0 { 1 } else { r }
}
```
Notes: every `mulu.w` uses only the low 16 bits of the previous product; the last shift is
`lsr.w`, so large products wrap (base*32*35 >= 65536 when the stage-1 value is > ~58). Port the
16-bit truncation exactly. The weakness multiply uses `mulu.w` + `lsr.w #5`. The caller subtracts
with `sub.w` and clamps the enemy HP at 0.

`$C6` power factor (0x11738, recomputed at init and after every hit taken) [O]:
`row = HP >= 75 ? 0 : HP >= 50 ? 32 : HP >= 25 ? 64 : 96; $C6 = rd8(0x2C9B6 + row + (rand() & 31))`.
Values are mostly 32 (x1.0), up to 48 at high HP and up to 64 below 25 HP (low HP = stronger).

### 4.4 Punch chain (jab, jab, punch, finisher) [O]

```rust
fn jab_entry(p, alt: u8) {                 // 0x193CE (alt 0) / 0x193DE (alt 1); does NOT fall through
    p.anim_flags = 0; anim1(p, alt);
    p.state = 0x02000004; new_attack(p, 0x00); p.bB2 = 0; p.bBC = 0; sfx(0x95CE);
}
fn jab(p) {                                // 0x19416
    if p.b59 != 0 { anim1(p, 2); return hit_confirm_entry(p); }   // jab connected
    if anim_step(p) { p.bB3 = 5; p.bAE = 0; p.bB2 = 0; return idle_entry(p); }   // whiff: 5-frame move lock
    if p.b2C != 0 && p.in_new & ATK != 0 { face_from_input(p); return jab_entry(p, 1); }   // re-jab
}
fn hit_confirm_entry(p) {                  // 0x19464, falls through
    p.state = 0x02000006; p.bAD = 0; p.bB2 += 1; p.w70 = 0; p.action = 0x10; hit_confirm(p)
}
fn hit_confirm(p) {                        // 0x1947E
    if p.b59 != 0 {                         // hit-stop: animation frozen
        p.b59 -= 1; if p.b59 == 0 { anim_step(p); }
        if p.in_new & ATK != 0 { p.bAD = p.in_new; }    // buffer
        return;
    }
    if anim_step(p) {
        p.bAE = 0x28;                       // 40-frame window to continue from idle/walk
        if p.bB2 == 4 { p.bAD = 0; p.bB2 = 0; p.bAE = 0; }
        return idle_entry(p);
    }
    chain_continue(p)
}
fn chain_continue(p) {                     // 0x194D4
    if p.bAD == 0 && p.in_new & ATK == 0 { return; }
    match p.bB2 {
        1 => { face_from_input(p); punch2_entry(p) }   // 0x1950E: anim1 #0, state 0x08, id 0x00
        2 => { face_from_input(p); punch3_entry(p) }   // 0x1956C: anim1 #3, state 0x0A, id 0x02
        3 => { face_from_input(p); finisher_entry(p) } // 0x195B2
        _ => { p.bAD = 0; }                             // b2 == 0 or >= 4: press is eaten
    }
}
// punch2/punch3/finisher entries: set state, new_attack(id), $AD = 0, sound (jsr), then FALL THROUGH
// into punch_n() in the same frame (unlike jab_entry).
fn punch_n(p, hit_anim) {                  // 0x1953E / 0x1959C / 0x1965A
    if p.b59 != 0 { /* punch3 only: p.b59 += 1 */ anim1(p, hit_anim); return hit_confirm_entry(p); }
    if anim_step(p) { p.bB3 = 5; p.bB2 = 0; return idle_entry(p); }   // 0x19552
}
// hit_anim: punch2 = 2, punch3 = 5, finisher = 6
```
Finisher entry 0x195B2 (combo throw) [O]: if `$70` (enemy probed by `$2E == 0` frames, §6.3) is
alive, its state is 2/0 (normal), and the stick holds **only** up or down (no left/right):
up = throw backwards (turn around), down = throw forwards:
`facing_right ? (up -> face left, down -> face right) : (up -> face right, down -> face left)`;
then `enemy.$70 = p; enemy.state = 0x02060000; goto throw (0x1D888)` (§4.8).
Otherwise: anim1 #4 (Hannah uses #7 when `$2C == 0`), `state = 0x0200000C`, attack id 0x04.

Summary of the chain rules [O]:
- Each step needs the previous step to **connect** (`$59 != 0` set by the hit). A whiff resets `$B2` and locks movement 5 frames (`$B3`).
- After a connected hit, press attack during hit-stop (buffered in `$AD`) or during the rest of the
  hit animation to continue at once; or within 40 frames (`$AE`) from idle/walk. In idle, walking is
  locked while `$AE >= 0x24` (first 4 frames).
- The 4th hit ends the chain (`$B2 = 4` -> reset). `$B2` is also reset when the player is hit.

### 4.5 Hit-stop [O]
On a hit, the attacker's and victim's `$59` = attack byte +3. Attacker states test `$59` first and
`return` while it is nonzero (animation and motion frozen). The victim's hurt states count the
same `$59` as their stun duration (§5.1). Enemies with `$59 != 0` cannot hit players (0x115C2).

### 4.6 Special / desperation attack (attack + jump) [O]

```rust
fn special_check(p) -> Abort {             // 0x63AA, called by idle, walk, run, grab hold, weapon states
    if p.hp > 1 && ((p.in_held & ATK != 0 && p.in_new & JMP != 0) ||
                    (p.in_held & JMP != 0 && p.in_new & ATK != 0)) { abort_caller(); return special_entry(p); }
}
fn special_buffer(p) { if same condition { p.bC9 = 10; } }   // 0x63E6, run at the top of every hurt frame
fn special_entry(p) {                      // 0x1968E
    p.wEE = 20;                             // HP-cost budget
    if G.b4D5 == 0 { hp_bar(p); }  sfx(0x992C); p.action = 0x11;
    if p.chr == 1 { return hannah_special_entry(p); }      // 0x1AAE6, airborne
    p.state = 0x0200000E; new_attack(p, 0x12); p.bB2 = 0; p.anim_flags = 0;
    p.wB4 = 0; p.wB6 = 0; p.bC5 = 0;        // held weapon is discarded
    anim1(p, 8); special(p)
}
fn special(p) {                            // 0x196F2
    if p.b59 != 0 { p.b59 -= 1; return; }
    if anim_step(p) { p.b00 = 2; p.bF9 = INV[12 + chr] /*60*/; return idle_entry(p); }
}
```
HP cost (0x10B08 / 0x11C80, applied when a frame with `$2E == 6` hits a victim) [O]:
`cost = if p.wEE == 20 {10} else {4}; p.wEE -= cost; if p.wEE >= 0 { p.hp = max(p.hp - cost, 1) }`.
So 10 HP for the first victim hit, 4 for the 2nd and 3rd, nothing after (max 18 per special);
no cost if it hits nothing; it can never kill (min 1 HP). Skipped when `G.$4DD != 0`.
During the move the frame class 6 makes the player immune to enemy attacks; after it 60 frames of invulnerability.

Hannah's special 0x1AAE6: `state 0x0200040E`, anim1 #8, id 0x12, `vy = 0x4C0, ay = -0x40`, other v/a = 0, airborne.
Rising (handler 0x1AB44): frames with `$2C != 0` only animate; otherwise hit-stop, motion, and when
`vy < 0` -> state 0x410 with a 15-frame hover (`$80 = 15`, motion skipped) then falls; on landing
`$00 = 2, $F9 = 60`, `landing_entry`.

### 4.7 Dash attack (run + attack) [O]

Jack (0x19720 -> 0x197A2): `sfx 0x9984, $C8=0x13, $F1=0, $BC=0`; `state 0x02000010`, id 0x14,
`$B2 = 0`, `vy=vz=ay=az=0`, `vx = ±0x500`, `ax = ∓0x20` (decelerates), `$80 = $2C, $81 = 0`, anim1 #0xC.
```rust
fn dash_attack(p) {                         // 0x197A2
    if p.b59 != 0 { p.b59 -= 1; return; }
    motion(p);
    if anim_step(p) { p.state = 0x02000012; anim0(p, 0xD); p.bBC = 0; return recovery(p); }  // 0x19812
    let old = p.b80; p.b80 = p.b2C;
    if old != p.b2C { new_attack(p, if p.b81 == 0 { 0x5A } else { 0x5C }); p.b81 += 1; }
}
fn recovery(p) { if anim_step(p) { idle_entry(p) } }      // 0x19826
```
Hannah/Mustapha/Mess (0x1ABD4, airborne lunge): id 0x14, then 0x5A/0x5C on each `$2C` change like
above; `(vx, ax, vy, ay)` from 0x1BF2A Hannah (0x600, -0x18, 0x330, -0x30), 0x1BF32 Mustapha
(0x600, -0x18, 0x3C0, -0x38), 0x1BF3A Mess (0x400, 0, 0x280, -0x28), vx/ax sign by facing;
anim1 #0xC; on landing -> recovery (0x19812).

### 4.8 Grab, knees and throws [O]

Grab start (collision 0x10834 -> 0x108AE, during the collision phase): a player whose current frame
has class `$2E == 2` and a nonzero attack box `$49` (walk frames [I]) overlaps the body box of an
enemy that is not invulnerable, on the ground (`$51 == 0`), `$63 >= 0`, with enemy frame class 0 or 2.
Requires `G.$4DD == 0` and the player's `$F0 == 0` (10-frame re-grab cooldown). Then:
`p.$70 = e; p.$73 = e.$72; p.state = 0x02040000; p.$2E = 0xC; p.$115 = 0;
e.$70 = p; e.state = 0x02060000; e.$2E = 0xE; e.$02 = p.index; e.$6E = e.$6C`.

```rust
fn grabbing(p) {                           // 0x1D652
    if p.bF1 == 2 { return command_move_entry(p); }
    match p.b06 { 0 => hold(p), 2 => knee(p, 1), 4 => knee(p, 2), 6 => throw(p) }   // table 0x1D66E
}
fn hold(p) {                               // 0x1D676
    if p.b07 == 0 { anim3(p, 0); p.anim_flags = 0; p.action = 0x30; p.bA9 = 0x78; p.b07 = 2; }
    let e = p.w70;
    if e.frame_class != 0xE || e.b05 != 6 { return release(p); }
    p.b73 = p.b73.wrapping_sub(1); if p.b73 == 0xFF { return release(p); }    // enemy escapes
    let d = p.in_held & 0xF;
    let toward = if p.facing != 0 { d & 1 } else { d & 2 };
    if d == 0 || toward != 0 { p.bA9 = 0x78; }
    else { p.bA9 = p.bA9.wrapping_sub(1); if p.bA9 == 0xFF { return release(p); } }   // 120 f pushing away
    special_check(p)?;
    if p.in_new & ATK != 0 {
        p.b73 = e.b72;                       // refresh hold time
        if d & 3 != 0 { return throw_entry(p); }          // left or right held
        return knee_entry(p);                              // no direction, or only up/down
    }
    if p.in_new & JMP != 0 { return release(p); }
    anim_step(p);
}
fn release(p) { p.bAD = 0; p.bAE = 0; p.bF0 = 10; idle_entry(p) }        // 0x1D72A
fn knee_entry(p) {                        // first knee id 0x06 (anim3 #1), later ones id 0x08 (anim3 #2)
    if p.b115 == 0 { p.state = 0x02040200; anim3(p, 1); new_attack(p, 0x06); }
    else           { p.state = 0x02040400; anim3(p, 2); new_attack(p, 0x08); }
    p.b115 += 1; knee(p, ..)
}
fn knee(p) {                               // 0x1D778 / 0x1D810
    let old = p.b2C;
    if anim_step(p) { return hold_entry(p); }              // back to hold (0x1D67C)
    if p.b2C > old { grab_hit(p); /* + HUD and spark at offset table 0x1D93E */ }
}
fn throw_entry(p) {                        // 0x1D850
    if p.chr == 3 {                          // Mess: forward or backward throw animation
        let toward = p.in_held & (if p.facing != 0 { 1 } else { 2 });
        anim3(p, if toward != 0 { 3 } else { 4 });
    } else { let d = p.in_held & 3; if d != 0 { p.facing = d & 1; } anim3(p, 3); }
    p.state = 0x02040600; new_attack(p, 0x0A); throw(p)
}
fn throw(p) {                              // 0x1D8AE
    if anim_step(p) { p.b00 = 2; p.bF9 = INV[4 + chr]; return release(p); }
    if p.b2C != 0 && p.w70 != 0 {
        grab_hit(p);                         // damage now
        let e = p.w70; e.state = 0x02020000; e.b06 = 8; e.facing = p.facing;   // reaction 8 = thrown
        let mut k = p.w20;
        if k == 0xC && p.b2C != 2 { k += 4; e.facing = p.facing ^ 1; }       // Mess variant
        let (dx, dy) = THROW_OFF[k/4];       // 0x2CB32: Jack(-0x48,0x2B) Hannah(-0x44,0x35) Mustapha(-0x40,0x35) Mess(-0x38,8) Mess-alt(0x30,0x10)
        e.x_hi = p.x_hi + if p.facing != 0 { -dx } else { dx }; e.y_hi = p.y_hi + dy;
        p.w70 = 0;
    }
}
fn grab_hit(p) {                           // 0x12526 (no hit-stop is applied)
    let e = p.w70; e.w68 = p; e.b5E = p.b5E; e.b02 = p.index;
    let a = attack_entry(p);                 // §4.2
    e.w6E = e.hp; let d = player_damage(p, e, a.dmg);
    if (e.hp as i16).wrapping_sub(d as i16) <= 0 { e.state = 0x02020000; e.hp = 0; } else { e.hp -= d; }
    if e.stun <= a.stun { e.state = 0x02020000; e.stun = 0; } else { e.stun -= a.stun; }   // sub.b + bhi
    e.b7B = a.react; e.b06 = a.react; e.b07 = 0;
    if a.snd != 0 { sfx(a.snd) }  hit_fx(a.fx); e.b9A = a.b7;
}
```
So knees continue while the enemy's stun meter and HP last; when either runs out the enemy leaves
the grabbed state (reaction of that knee) and the hold releases. The thrown enemy flies with
reaction 8 (enemy code, out of scope). Throw damage is the 0x0A entry.

### 4.9 Pick-up and weapons (summary) [O unless noted]

- `find_pickup` (0x5AA2): not airborne and `G.$506 == 0`; scans the item list (`G.$69EE/$69F0`)
  for items with state 2/0 whose box overlaps the player in Z and X and whose Y is within 16 px of
  `player.y - 8`; picks the **lowest** item type `$A0`; type 0x20A (heavy object) needs the stick
  pushed toward it (unless item `$26` is 6 or 8); weapons (type high byte 2) are skipped while
  `$F8 != 0` (40 frames after a pickup).
- `pickup_entry` 0x19832: `state 0x02000014`, sound, anim0 #0xA (#0x16 for 0x20A); item `$05=4`,
  item `$70 = p`; if item type high byte is 2 it becomes the held weapon (`$B4/$B6`, the old one is
  dropped by setting its `$05 = 0xC`). At anim end: `$F8 = 40, $00 = 2, $F9 = INV[8+chr]` -> idle.
  Food/score items are consumed by item code [I].
- Held types seen in player code: 0x201/0x202 (melee, special run and jump attacks), 0x203 (hold to
  charge up to 60 frames; release after >= 8 / >= 45 frames changes the animation), 0x206/0x207
  (guns that can fire while walking, auto-aim at nearest enemy 0x64F6), 0x209 (strong recoil, state
  0x22: pushed backwards `vx = RECOIL[chr]` from 0x19CD8 = Jack 0x800, Hannah 0xA00, Mustapha 0xC00,
  Mess 0x600, `ax = 0x100` against it; bazooka [I]), 0x20A (heavy object, slow walk tables, no jump for Mustapha).
  Weapon names (0x23C65): RIFLE, GUN, SHOT GUN, M-16A1, BAZOOKA, UZI. Mapping type -> name [U].
- Ammo / durability = the **item object's** `$6C`; after a weapon attack ends, if it is 0 the
  weapon is dropped (`$B6 = 0`, 0x19980). Where `$6C` is decremented is item code [U].
- Getting hit drops the weapon (`0x1177A` sets `$C5 = 1, $B6 = 0`, except some 0x2xx cases) [O].

#### 4.9.1 Weapon states in detail (phase 1 task 3, ported in `player_weapon.rs`) [O]

Held kinds (`+$B6`, the item's `+$A0`, see `item.rs` / `item/weapon.rs`): 0x201 throwable (bombs, empty pistol / Uzi /
shotgun), 0x202 club or stick, 0x203 bat and heavy weapons (and an empty rifle or M-16), 0x204 pistol, 0x205 rifle, 0x206 Uzi,
0x207 M-16, 0x208 shotgun, 0x209 bazooka, 0x20A heavy object (containers). The attack button with a weapon in hand
(`weapon_attack_entry` 0x198DC) starts, by kind:

| kind | state (step) | entry / handler | what it does |
|---|---|---|---|
| 0x206, 0x207 | 0x1C standing fire, 0x1E walking fire | 0x19AB0 (aim 0x64F6, `+$81 = 3`) -> 0x19ACC / 0x19B60; 0x19AEA / 0x19B90 | the item fires on the rising edge of `+$2C`; `+$81` counts 3 frames of grace after the animation; out of ammo (`+$6C` of the item is 0) the gun is let go (a club 0x203 stays); jump button -> jump |
| 0x209 | 0x22 recoil | 0x19CE0 / 0x19D5E | `vx = RECOIL[chr]` (0x19CD8) against the facing, `ax = 0x100` the other way; moves only while `|vx| >= 0x20`; dust every other frame; airborne -> fall |
| 0x203 | 0x20 charge | 0x19C36 / 0x19C62 | `+$80` counts attack frames to 60; on release `+$2F = 1` when `+$80 >= 8`, one more animation step when `>= 0x2D` |
| other (0x201, 0x202, 0x204, 0x205, 0x208, 0x20A) | 0x16 | 0x198DC / 0x19954 | pose by kind (0x5E5C); the item swings or throws on `+$2C` = 1 / 2; a club (0x202) swings only when an enemy is in reach (0x6248), else the pose of 0x201 (a throw); at the end a non-club without ammo is let go |

All of them set `+$C4 = 1` (cleared by the idle entry and by the item that used it) and `+$C8 = 0x10` (0x1E does not
set `+$C8`); 0x16 / 0x1C / 0x20 / 0x22 clear `+$25`; 0x16 and 0x22 also clear the item's `+$A8`. Running with a club or a throwable (0x201 / 0x202) and pressing
attack is the run swing (state 0x202, entry 0x1A174 / 0x1A18C, handler 0x1A2F2; item `+$A8 = 1` makes the item throw on
its frame 2); any other kind is the dash attack. A heavy object jump (0x1A5D0 -> 0x1B038) is air state 0x1E (vector
class 0xC, animation `0x1B0C6[chr]`, no jump attacks), landing 0x20 (`0x1B0F2[chr]`); falling with it uses
`0x1A902[chr]` (set 5).

Poses (animation set 5 by kind, tables `0x5C46` idle, `0x5D3C` walk, `0x5EEE` swing (17 words per character: the
12 kinds, then variants of the heavy object at `+$26 + 0x18`), `0x6002` gun walk): 0x5BDC idle (static, no step), 0x5CA6
walk (+ legs), 0x5E5C swing, 0x5F76 gun walk, 0x1A0F0 run. All four first test: kind not 0, `+$B4` alive and in its held
state (sub 8), else `+$B4 = +$B6 = 0` and the plain animation. **Split body**: with `+$25` bit 0 the main frame (`+$28`)
is the upper body drawn at `(+$3C, +$3E)` (mirrored by the facing) and the layer `+$30` (set 0, animations 0xB / 0x10 /
0x18, 12 byte frames like the main ones) is the legs; the layer has its own timer `+$37`, frame bytes `+$34 / +$35`
and extra bytes (`+$4C..`, read at the frame's first word). The offsets table `+$38` (0x5D9C.., 0x1BE86..) is
indexed by `+$35` (signed, 4 bytes per entry). Walk with the layer: the layer's end restarts the walk (0x192C6).

Pick-up (`find_pickup` 0x5AA2): see `player_find_pickup`. Pick-up state 0x14: voice 0x9C20, anim set 0 #0xA (#0x16 for
0x20A), the item gets `+$02` = the player, sub 4, `+$70` = the player; a kind with high byte 2 becomes the held item
(the old one: sub 0xC, `+$C5 = 0`); `+$10F != 0` refills the new gun at once (0x284E2); at the end `+$BC = 0`,
`+$F8 = 40`, `+$00 = 2`, `+$F9 = INV[8 + chr]`, idle. A hit (0x1177A) clears `+$B6` and sets `+$C5 = 1`, except that
Hannah keeps the guns 0x204..0x208 and Mess keeps the bazooka; knockdown and burn do the same; being held by an enemy
and dying clear only `+$B6`; a special attack clears `+$B4`, `+$B6` and `+$C5`. A player that joins mid-stage (respawn reset with `+$07 == 0`) gets a bazooka (0x65A4, 4 rockets).

### 4.10 Command move: down, up + attack (0x6AC8) [O]

```rust
fn command_fsm(p) {                        // 0x6AC8, every alive frame
    match p.bF4 {
        0 => if p.in_held != p.in_prev && p.in_held & 0xF == 4 { p.bF5 = 10; p.bF4 = 2; },
        2 => { p.bF5 = p.bF5.wrapping_sub(1);
               if p.bF5 == 0xFF { p.bF4 = 0; }
               else if p.in_held & 0xF == 8 { p.bF5 = 10; p.bF4 = 4; }
               else if p.in_held & ATK != 0 { p.bF4 = 0; } }
        4 => { p.bF5 = p.bF5.wrapping_sub(1);
               if p.bF5 == 0xFF { p.bF4 = 0; }
               else if p.in_new & ATK != 0 { if p.in_held & 0xF == 8 { p.bF1 = 2; } p.bF4 = 0; } }
    }
}
```
Tap down (alone), then up (alone) within 10 frames, then press attack while holding up within 10
frames. Checked in walk and while grabbing -> `command_move_entry` 0x1B6EC (`state 0x02000430`,
anim1 #0xE, `$C8 = 0x15`, per-character code at table 0x1B72E: Jack 0x1B736, Hannah 0x1B85C,
Mustapha 0x1B988, Mess 0x1BAA6, attack ids 0x70..0x7A). Exact per-character motion [U, not decoded].

---------------------------------------------------------------------------------------------------

## 5. Getting hit

### 5.1 Enemy attack hits a player (0x115B0 -> 0x11652) [O]

Conditions: `G.$4DD == 0`; enemy frame class `$2E == 4`; enemy `$59 == 0`; player `$00` bit1
clear; player frame class in {0,2,4,0xC,0xE} (6, 8, 0xA = immune); enemy `$49 != 0`, player
`$48 != 0`; not (player `$E9` == enemy `$5E` and player `$68` == enemy); box overlap (§6.1).
If the player is held (`$05 == 6`) and `$C7 != 0` the alternate path 0x117D8 is used.

```rust
fn enemy_hits_player(e, p) {               // 0x11682
    p.b00 = 2;                              // invulnerable for the whole hurt sequence
    p.state = 0x02020000; p.b7A = 0; p.w68 = e;
    p.bCB = 0x3C; p.bCA += 1; p.bCD = 0; p.bB2 = 0; p.bE9 = e.b5E;
    let a = 0xA2EA4 + e.atk58 as u32 * 8;   // same 8-byte layout as §4.2
    p.w6E = p.hp;
    let d = enemy_damage(e, p, rd8(a));     // 0x6422
    p.hp = if p.hp as i16 - d as i16 > 0 { p.hp - d } else { 0 };
    p.wF6 = 0x78;                           // stun meter refills after 120 quiet frames
    p.stun = p.stun.saturating_sub(rd8(a+1));
    p.b5A = e.b5A;
    let r = if p.action == 0x22 { 0x22 } else { rd8(a+2) };
    p.b7B = r; p.b06 = r; p.b07 = 0;
    p.b59 = rd8(a+3); e.b59 = rd8(a+3);
    if rd8(a+4) != 0 { sfx(rd8(a+4)); }
    spark(e, p, rd8(a+6)); p.b9A = rd8(a+7);           // byte +5 not used on this path
    drop_weapon_check(p);                   // 0x1177A
    compute_power_factor(p);                // 0x11738 (uses rand)
}
fn enemy_damage(e, p, base: u8) -> u16 {   // 0x6422 (all scales x/32)
    // debug: G.$4AA && G.$24 bit1 -> 0
    let mut d = base as u16;
    d = ((d as u32 * rd8(0x2C3AE + e.w84*8 + (G.w5EE & 0x1F)) as u32) as u16) >> 5;   // enemy type x rank
    let hp = p.hp;
    let band = if hp >= 76 {0} else if hp >= 51 {0x40} else if hp >= 26 {0x80} else {0xC0};
    let s = d as i16 - rd16(0x2C896 + band + (rand() & 0x3E)) as i16;   // low HP: small random reduction (0..4)
    d = if s < 0 { 1 } else { s as u16 };
    d = ((d as u32 * rd16(0x2C88E + p.chr*2) as u32) as u16) >> 5;    // defense: Jack 32, Hannah 34, Mustapha 35, Mess 30
    d = ((d as u32 * rd16(0x2C996 + ((hp >> 2) & 0x1E)) as u32) as u16) >> 5;  // 25..33 by HP
    if d == 0 { d = 1; }
    if hp != 1 && hp <= d && d - hp < 5 { d = hp - 1; }   // "mercy": an overkill of 0..4 leaves 1 HP
    d
}
```

### 5.2 Hurt dispatcher 0x1C02C [O]
Every hurt frame: `special_buffer(p)` (A+J -> `$C9 = 10`), then dispatch on reaction `$06`
(table 0x1C03E): 0 light (front), 2 light alt, 4 knockdown, 6 heavy knockdown, 8 thrown,
0xA/0xC burn knockdown, 0xE/0x10/0x12 pushback (fallback knockdown 4), 0x14/0x16/0x18 pushback
(fallback knockdown 6), 0x1A light from behind, 0x1C knockdown from behind, 0x1E getup,
0x20 wall bounce, 0x22 special (grab-related) [U].

"From behind" test (used by all reactions) [O]:
`behind = ((attacker.x_hi < p.x_hi) as u8) == p.facing` (attacker on the right while facing left, or vice versa).

Knockdown override [O]: light and pushback reactions become a knockdown when
`hp == 0 || stun == 0 || $5A != 0 || airborne`.

### 5.3 Light hit (reaction 0 / 2 / 0x1A) [O]

```rust
fn light(p, anim: u8) {                    // 0x1C062 (anim 0), 0x1C124 (anim 1)
    if p.b07 == 0 {
        p.b23 = 0;
        if behind(p) { p.b06 = 0x1A; p.b07 = 0; return light_behind(p); }   // anim2 #6, flip facing at end
        if p.hp == 0 || p.stun == 0 || p.b5A != 0 || p.airborne != 0 {
            return if code == 0 { knockdown4(p) } else { knockdown6(p) };
        }
        anim2(p, anim); sfx(0x98CC); face_icon(2); hp_bar(p); p.action = 0x20; p.b07 = 2;
    }
    p.b59 = p.b59.wrapping_sub(1); if p.b59 != 0 { return; }   // stun lasts the hit-stop value
    recover(p)
}
fn recover(p) {                            // 0x1C0DA
    p.b23 = 0;
    if p.hp == 0 { p.state = 0x04000000; return dead(p); }
    rank_adjust(p);                         // 0x183E4 [U details]
    if p.b7B == 4 { p.b00 = 2; p.bF9 = INV[0 + chr] /*20*/; } else { p.b00 = 1; }
    idle_entry(p)
}
```

### 5.4 Pushback (reactions 0xE..0x18) [O]
Table 0x1D27C `(frames, vx, ax)`: 0xE/0x14 = (15, 0x500, -0x53), 0x10/0x16 = (18, 0x800, -0x71),
0x12/0x18 = (20, 0xB00, -0x96). From the front: anim2 #0, pushed backwards (vx sign opposite to
facing), `$117 = 0`. From behind: anim2 #6, pushed forwards, `$117 = 1`. `vy=ay=vz=az=0`. Then
wait `$59` frames (frozen), then slide `frames` frames with `motion()` (dust every other frame),
then flip facing if `$117`, `recover()`. A 1/64 random check calls 0x5A454 (may set `$05 = 0x5E`) [U].

### 5.5 Knockdown (reactions 4, 6, 0x1C, burn 0xA/0xC) [O]

```rust
fn knockdown4(p) { p.b06 = 4; p.b07 = 0; register_flying_body(p) /*0x4B58*/; kd(p) }   // 0x1C19C
fn kd(p) { match p.b07 { 0 => kd_start(p), 2 => kd_air1(p), 4 => kd_air2(p), 6 => kd_lie(p) } }
fn kd_start(p) {                           // 0x1C1C0
    p.b23 = 0; p.action = 0x23; p.bCC = 0;
    if behind(p) { return knockdown_behind(p); }          // reaction 0x1C
    anim2(p, 2);
    set_kd_vec(p, KD4_ALIVE);              // (vx 0x280, ax 0, vy 0x500, ay -0x40); vx,ax negated when facing right
    dizzy_roll(p);                          // 0x1D0FA
    if p.hp == 0 { set_kd_vec(p, KD4_DEAD /*(0x300,0,0x780,-0x40)*/); p.bCD = 2; sfx(0x986C); }
    p.vz = 0; p.az = 0; face_icon(2); hp_bar(p);
    p.wB6 = 0; p.bC5 = 1; p.airborne = 1; p.b83 = 0; p.b7B = 4; p.stun = 10; p.b07 = 2;
    kd_air1(p)
}
fn kd_motion(p) {                          // 0x1D0C0: slow motion for dizzy / death
    match p.bCD { 0 => motion(p),
                  1 => if G.frame & 3 != 0 { motion(p) },      // skip every 4th frame
                  _ => if G.frame & 2 != 0 { motion(p) } }      // btst #1,$1(a5): half speed
}
fn kd_air1(p) {                            // 0x1C290
    body_hits_enemies(p) /*0x6606*/; splash_check(p); kd_motion(p);
    if p.b2C == 0 { anim_step(p); }
    floor_and_walls(p);                     // extra call (the normal one also runs after dispatch)
    if p.b50 != 0 && p.b50 & 1 == 0 { return wall_bounce(p); }   // reaction 0x20 (0x1CF9C) [details U]
    if p.airborne == 0 {                    // first ground contact: bounce
        p.airborne = 1; p.vy = (if p.hp != 0 { KD4_ALIVE } else { KD4_DEAD }).vy;
        p.ay = 2 * KD.ay;                    // double gravity -> half height bounce
        motion(p); p.b07 += 2; anim_step(p); sfx(0x9796);
    }
}
fn kd_air2(p) {                            // 0x1C302
    body_hits_enemies(p); kd_motion(p); if p.b2C == 0 { anim_step(p); } floor_and_walls(p);
    if p.airborne == 0 {
        lie_setup(p);                       // 0x1D17E
        if p.bCC != 0 && p.hp != 0 { anim2(p, 9); p.facing ^= 1; p.b00 = 1; }   // tech roll (dead path: $CC is 0)
        sfx(0x9796); p.b07 += 2;
    }
}
fn lie_setup(p) {                          // two rand() calls, in this order
    let row = (G.w5EE & 0x18) as usize * 2;
    p.bE8 = rd8(0x1D1C4 + row + (rand() & 15));   // 5, 15 or 30 frames (longer at higher rank)
    p.bBE = rd8(0x1D204 + row + (rand() & 15));   // mash meter 15, 20 or 25
    p.bBF = 0;
}
fn kd_lie(p) {                             // 0x1C35A
    body_hits_enemies(p); mash(p);
    if p.bBE == 0 { return getup_entry(p); }
    p.bE8 = p.bE8.wrapping_sub(1); if p.bE8 == 0xFF { return getup_entry(p); }
    anim_step(p);
}
fn mash(p) {                               // 0x6012
    let d = p.in_new & 0xF;
    if d != 0 { p.bBE = p.bBE.saturating_sub(MASHW[d]); return; }      // MASHW (0x605E): R or L alone = 3, else 1
    if p.bBF != 0 { p.bBF -= 1; return; }
    if p.in_new & ATK != 0 { p.bBF = 6; p.bBE = p.bBE.saturating_sub(1); }
}
```
Heavy knockdown (6, 0x1C37C): same flow with anim2 #8, `KD6_ALIVE = (0x400, 0, 0x500, -0x40)`,
`KD6_DEAD = (0x400, 0, 0x4B0, -0x30)`, plus `if |vx| < 0x40 { ax = 0 }` each air frame (0x1C4F0),
and at the first ground contact `vy = -vy` then reloaded from table 0x1D2A2/0x1D2AA with `ay*2`.
From behind (0x1C, 0x1CBF2): anim2 #7, `vx = 0x340` pushed forwards, `vy = 0x500` (dead 0x540), `ay = -0x4A`.
Burn (0xA/0xC, 0x1C6E4): faces the attacker, anim2 #2, vectors 0x1D2AE (dead 0x1D2B6), `$23 = 0x84`,
spawns 2 flame objects (0x1C86E), `G.$7B2 = 2` while lying.

Lie time = whichever comes first: `$E8 + 1` frames, or mashing `$BE` down to 0. [O]

### 5.6 Getup and dizzy [O]

```rust
fn getup_entry(p) {                        // 0x1CD9A
    if p.hp == 0 { p.state = 0x04000000; p.b23 = 0; return; }
    p.state = 0x02020000; p.b06 = 0x1E; p.wF6 = 0x78; p.b5A = 0; p.stun = 10; anim2(p, 3); p.b23 = 0;
    getup(p)
}
fn getup(p) { match p.b07 {
    0 => if anim_step(p) { if p.bCD != 0 { dizzy_entry(p) } else { recover(p) } },   // recover: $00=2, $F9=20
    _ => dizzy(p) } }
fn dizzy_roll(p) {                         // 0x1D0FA, at knockdown start
    if p.bCC != 0 || p.hp < 30 || (p.wCE >> 8) != 0 || p.bCD != 0 { p.bCA = 0; p.bCD = 0; return; }
    let n = p.bCA.min(5); if n == 0 { p.bCA = 0; p.bCD = 0; return; }
    let mask = rd16(0x1D166 + n*2);         // [0, 0, 0xFFF0, 0xFFFF, 0xFFFF, 0xFFFF]
    if mask >> (rand() & 15) & 1 != 0 { p.bCA = 0; p.bCD = 1; p.b7E = 0; p.b7F = 0; }
    else { p.bCA = 0; p.bCD = 0; }
}
fn dizzy_entry(p) {                        // 0x1CE02
    p.b07 = 2; anim2(p, 4); p.bCA = 0; p.bCB = 0; p.action = 0x21;
    let r = (G.w5EE & 0x18) as usize;
    p.bBE = rd8(0x1CF5C + r*2 + (rand() & 15));     // stars to mash away: 15..150
    p.bBF = 0;
    p.w80 = rd16(0x1CEDC + r*4 + (rand() & 0x1E));  // max dizzy frames: 40..300
    p.wCE = p.w80 + 0x708;                           // immunity
    p.b00 = 1;                                       // dizzy player CAN be hit
    voice(0x94338, p.w80);
    dizzy(p)
}
fn dizzy(p) {                              // 0x1CE88
    p.b109 = p.bBE; mash(p);
    if p.b109.wrapping_sub(p.bBE) >= 1 && p.dur != 1 { p.dur = 1; }   // mashing speeds up the anim
    if p.bBE == 0 { return dizzy_end(p); }
    p.w80 = p.w80.wrapping_sub(1); if p.w80 == 0xFFFF { return dizzy_end(p); }
    anim_step(p);
}
fn dizzy_end(p) { p.bCD = 0; p.b7E = 0; p.b7F = 0; recover(p) }
```
So: being knocked down by the 3rd hit taken within a 60-frame chain always dizzies (2nd hit: 75%),
if HP >= 30 and not dizzied in the last ~1600+ frames. [O]

### 5.7 Held by an enemy (`$05 = 6`, 0x1D2BE) [O]
Setup: `$C8 = 0x40, $75 = 0xFF, $BE = 40` (mash meter), `$7B = 0xFF`, `$59 = 0`, `$80 = 180`, weapon
dropped. Each frame `mash()`; each successful mash press subtracts 10 from the anim timer. When
`$BE` reaches 0 -> escape attack (`state 0x02060400`, anim4 #3, id 0x18; Hannah instead reverses
the hold and goes to her throw `0x02040600`). Released when the enemy lets go, or the stun meter,
`$5A`, or a knockdown reaction ends it (`0x1D48E` -> anim4 #2 -> getup path).

### 5.8 Death, lives, respawn [O]
- Death (`$04 = 4`, 0x1BF42): `[0]` `$00 = 2, $80 = 60, $05 = 2`; `[2]` 60 frames of blinking
  (`$25` bit3 toggles every frame), (if `G.$4D4`: HP set to 1 and back to idle [U meaning]);
  `[4]` if `$A3 > 0`: `$A3 -= 1`, `$00 = 1`, rank adjust (0x184E2), `0x4DE8` (HP 100, `$04 = 0x00020000`),
  keep facing, `$07 = 2`, then 0x10482 (adds BCD time to the stage clock G.$4E8, capped at 0x400 [I]);
  else game over for this player (`$00 = 0`).
- Respawn (`$04=0,$05=2`, 0x18B38): `[0]` X = camX + 0xC0, Z = camY (0x30 if `G.$4DC`), Y = floor + 0x180
  (drops from above); `[2]` reset physics (0x18C82: hitbox ptr, speed class 0, v=0, `$78 = 10`),
  **`$EC = 360` frames invulnerability** (blinking), face right, wait `$80 = 60` frames; `[4]` then
  `$51 = 1`, `$04 = 0x02000000` (falls in).

### 5.9 Invulnerability summary [O]
| situation | timer | frames (Jack/Hannah/Mustapha/Mess) |
|---|---|---|
| any hurt state (from the hit until recovery) | `$00 = 2` | whole sequence |
| after recovering from a knockdown / getup / dizzy | `$F9 = INV[0+chr]` | 20/20/20/20 |
| after a throw | `$F9 = INV[4+chr]` | 12/10/14/18 |
| after an item pickup | `$F9 = INV[8+chr]` | 10/8/20/10 |
| after the special (and Hannah's air special) | `$F9 = INV[12+chr]` | 60/60/60/60 |
| respawn | `$EC` | 360 (blinks every 4 frames) |
| frame classes 6, 8, 0xA (special etc.) | per frame | immune to enemy attacks |
`INV` = bytes at 0x6C3A. When the timer ends (and the other one is 0) `$00 = 1`. Light hits do
**not** give extra invulnerability (`$00 = 1` at recovery).

---------------------------------------------------------------------------------------------------

## 6. Collision system (0x10546, once per frame after all updates) [O]

Order inside 0x10548: player-vs-player 0x10664 (attacker frame class 4 vs other players: always
1 HP damage, never below 1 HP; only when `G.$4DD == 0`, `G.$4F3 < 3` and a Z window from table
0x106F2 [O]; when this is active in normal play [U]), 0x10DCC, 0x10F96, 0x1132A, **0x107F8 (player attacks vs enemies, grabs)**, 0x10CD0,
0x11962, 0x11A9E, 0x11F86, 0x1201C, 0x120D2, 0x11E56, (special-mode lists), then **0x115B0 (enemy
attacks vs players)**, 0x11CEC, 0x1185A. Exact roles of the other passes [U] (items, projectiles, thrown bodies).

### 6.1 Box test 0x128F8 (mode `$4A` = 0 for both) [O]
Box = 6 words at `$44 + id*12`: `(z_off, z_half, x_off, x_w, y_off, y_h)`.
```rust
fn overlap(a, ab: u8, b, bb: u8) -> bool {
    let (A, B) = (box(a, ab), box(b, bb));
    let dz = (a.z_hi + A.z_off) - (b.z_hi + B.z_off) + A.z_half;
    if dz as u16 > (A.z_half + B.z_half) as u16 { return false; }
    let ax = a.x_hi + if a.facing != 0 { -A.x_off - A.x_w } else { A.x_off };
    let bx = b.x_hi + if b.facing != 0 { -B.x_off - B.x_w } else { B.x_off };
    let dx = ax - bx + A.x_w;
    if dx as u16 > (A.x_w + B.x_w) as u16 { return false; }
    let dy = (a.y_hi + A.y_off) - (b.y_hi + B.y_off) + A.y_h;
    !(dy as u16 > (A.y_h + B.y_h) as u16)
}   // all 16-bit wrapping arithmetic, unsigned compares (bhi)
```
The attacker uses its attack box `$49`, the victim its body box `$48`. The hit spark (0x12720) is
placed at the middle of the overlap.

### 6.2 Player attack vs enemy (0x107F8 -> 0x10962 -> 0x109C2) [O]
For each player in the collision list, by its frame class `$2E`: 0 -> combo-throw probe (§6.3),
2 -> grab test (§4.8), 4/6/8 -> attack test, 0xA -> nothing. Attack test, for every enemy:
skip if enemy `$00` bit1, or `enemy.$5E == player.$5E` (already hit by this swing), or no boxes;
dispatch by enemy `$2E` (8/0xA immune); on overlap:
```rust
e.b5E = p.b5E; e.b02 = p.index;
if e.b87 != 0 && p.frame_class != 6 && p.action != 0x13 && p.action != 0x12 {
    p.b59 = 1; if enemy_type_handler(e)  /* 0x10B50[$84] */ { blocked_fx(); return; }   // guard [I]
}
e.b00 = 2; e.state = 0x02020000; e.b7A = 0; e.w68 = p;
let a = attack_entry(p); e.w6E = e.hp;
e.hp = (e.hp as i16 - player_damage(p, e, a.dmg) as i16).max(0) as u16;
e.stun = e.stun.saturating_sub(a.stun); e.b7B = a.react; e.b06 = a.react; e.b07 = 0;
e.b59 = a.stop; p.b59 = a.stop; if a.snd != 0 { sfx(a.snd) } hit_fx(a.fx); spark(a.spark); e.b9A = a.b7;
if G.w4F4 != 0 { /* counter-hit extra spark + effect 0x20 */ }
e.facing = p.facing ^ 1;
if p.frame_class == 6 { e.facing = (p.x_hi > e.x_hi) as u8; special_hp_cost(p); }   // §4.6
```
One swing can hit several enemies in the same frame (the loop continues). The attacker's own
`$59` is not checked here.

### 6.3 Combo-throw probe (frame class 0, 0x108FA) [O]
If the frame has an attack box, `p.$70` = the first enemy (list order) with `$04.w == 0x0200`
(mode 2, sub 0), not airborne, `$63 >= 0`, frame class != 0xA, whose body overlaps. Used by the finisher (§4.4).

---------------------------------------------------------------------------------------------------

## 7. Per-character parameters (summary)

Select screen stars (0xB444, display only, not read by gameplay) [O]:
Jack POWER 4 SPEED 3 SKILL 3; Hannah 2/4/5; Mustapha 3/5/3; Mess 5/2/4.

Real differences live in per-character tables [O]:
- walk/run speeds 0x50E4.. (§3.4), jump vectors 0x53E2.. (§3.6), jump-attack split height 0x1A734 and dive vectors 0x1A994;
- attack data 0xA26FC (damage/stun/reaction/hit-stop per move, §4.2);
- damage taken multiplier 0x2C88E: Jack 32, Hannah 34, Mustapha 35, Mess 30 (x/32);
- invulnerability after throw / pickup 0x6C3A;
- hitbox tables (`$44`) and animations (sets at 0x163E..);
- moves: Hannah has an airborne special and reverses holds; Hannah and Mustapha can wall-jump;
  Jack has a ground dash attack, the others an airborne lunge; Mess has forward/back throw animations;
  running-jump attacks and the command move are per character.

---------------------------------------------------------------------------------------------------

## 8. Open gaps

- Score rules (what adds to `$100`), food/treasure effects, and the item side of weapons (ammo
  decrement, projectile spawn, which type is which gun) [U] -> items topic.
- Exact behaviour of states 0x18/0x1A/0x26..0x2C, run sub-state 4, air states 0x18/0x1A/0x1C, wall
  bounce (reaction 0x20), reaction 0x22, and the per-character command moves [U].
- Global flags `G.$4DD` (scripted control), `G.$506`, `G.$518`, `G.$516`, `G.$4D4`, `G.$4F0` [U].
- Rank `G.$5EE` (dynamic difficulty from `G.$5F0`, routine 0x18592) feeds several tables here; its rules are not decoded [U].
- Which animation frames carry `$2C`, `$2E = 0/2/6` and which boxes: data, see the animation/hitbox spec [I].
- Enemy attack ids and their table rows (0xA2EA4) are listed raw in `tables.txt`; the mapping to enemy moves belongs to the enemy spec.
- A near copy of the player code exists at 0x1A00..0x3A00 with no absolute references found; treated as dead code [I].
