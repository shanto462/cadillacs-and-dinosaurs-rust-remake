# Cadillacs and Dinosaurs (dino, CPS1) - animation, sprite composition and hitbox spec

Scope: frame records (object +$28), animation tables and the routines that start/advance them,
composition (sprite assembly) formats, the draw list and its depth sort, hitbox tables and the
collision test, attack property records, OBJ palettes. Written for a Rust importer + native port.

Confidence tags: **[O]** observed in code or data, **[I]** inferred (strong evidence, not proven),
**[U]** unknown / open.

All addresses are 68000 program addresses (program image `out/dino_68k.bin` loaded at 0).
All multi-byte values are big-endian. `i16`/`u16`/`u32` are read from the program ROM.
`obj+$xx` = field of an object in work RAM. `$xxxx(a5)` = 0xFF8000+xxxx.

Tools in this folder: `dinolib.py` (decoder), `render_anim.py` (renders PNG strips into `png/`),
`dump_anims.py <root> <table_hex> <10|12>` (text dump of an anim table), `stats.py` (composition stats).

---------------------------------------------------------------------------------------------------

## 1. Object fields used by animation / drawing / collision

| field | size | meaning | tag |
|---|---|---|---|
| +$01 | u8 | visible flag; 0x16FA sets 1 if on screen, else 0 | O |
| +$03 | u8 | character id (players: 0 Jack, 1 Mustapha, 2 Hannah, 3 Mess) / enemy type | O (ids identified from rendered art) |
| +$08 | i16 | X (world). Long +$08 is X in 16.16 | O |
| +$0C | i16 | height above ground (jump axis, up = +). Long +$0C 16.16 | O |
| +$10 | i16 | depth / ground line (belt axis, + = away from viewer, up the screen). Long +$10 16.16 | O |
| +$20 | u16 | players: char*4 (selects box table and attack table) | I |
| +$23 | u8 | palette override: if bit7 set, low 5 bits replace every sprite's palette | O |
| +$24 | u8 | facing: 0 = art as stored (faces LEFT), nonzero = mirrored (faces right) | O |
| +$25 | u8 | draw flags: bit0 split body (+$30 anim), bit4 no dynamic shadow, bit5 no water splash, bit6 raw sprite-list frames (8-byte records), bit7 screen space (no camera) | O |
| +$28 | u32 | pointer to current frame record | O |
| +$2C | u8 | frame byte +4 ("param A", per-state meaning; e.g. 1 = combo cancel window in punch anims) | O/I |
| +$2D | u8 | frame byte +5 ("param B", phase index: indexes the +$38 offset table, used to resync anims) | O |
| +$2E | u8 | frame byte +6 = frame type (collision class, see 4.4) | O |
| +$2F | u8 | frame timer (byte +7 & 0x7F), decremented by the tick routine | O |
| +$30 | u32 | split-body: pointer to the secondary ("legs") frame record | O |
| +$34 | u16 | secondary frame word +4 | O |
| +$36 | u16 | secondary frame word +6 (low byte & 0x7F); +$37 = its timer | O |
| +$38 | u32 | split-body: offset table, 4 bytes per phase (i16 dx, i16 dy) | O |
| +$3C/+$3E | i16 | split-body: current (dx, dy) of the upper part, = table[+$38][byte +5 of secondary frame] | O |
| +$40 | u32 | animation table pointer (non-player objects) | O |
| +$44 | u32 | hitbox table pointer | O |
| +$48 | u8 | body (hurt) box index, from frame byte +8 (0 = none) | O |
| +$49 | u8 | attack / probe box index, from frame byte +9 (0 = none) | O |
| +$4A | u8 | 0 = single-box table, 1 = multi-box table (see 4.2) | O |
| +$4C..+$4F | 4 x u8 | per-composition extra data copied from 0x11D220 (see 2.6) | O |
| +$53 | u8 | wading flag (nonzero = standing in water; draw sets bit7 when it clipped) | O |
| +$54,+$56 | i16 | floor height parts (used by the shadow pass) | I |
| +$58 | u8 | current attack id x2 (selects the attack record, 4.5) | O |
| +$59 | u8 | hit-stop counter (set from attack record byte 3 on both sides) | O |
| +$5E | u8 | attacker: attack serial (from $67BE(a5)++ at attack start); defender: serial that last hit it | O |
| +$60 | u16 | 12-byte frames: word +10, high byte = attach point index (held weapon / held enemy) | O |
| +$62 | u8 | draw-order switch for a grab pair (3.4) | O |
| +$70 | u16 | partner object (grab) | O |

---------------------------------------------------------------------------------------------------

## 2. Compositions (sprite assembly)

### 2.1 Tables [O]

| what | where |
|---|---|
| composition pointer table | `u32 @0x0FFFF4` = 0x0D0CF4, `u16 @0x0FFFF8` = 3513 entries (u32 each) |
| layout pointer table | `u32 @0x0FFFFA` = 0x0D43D8, `u16 @0x0FFFFE` = 2516 entries (u32 each) |
| composition data | 0x0D6B28 .. 0x0F0008 |
| layout data | from 0x0F0028 |
| per-composition extra (4 bytes) | 0x11D220 + comp_index (same byte index as the pointer table) |

A **composition index** (`comp_index`) is always a *byte offset* into the pointer table, i.e. a
multiple of 4: `comp_ptr = u32(0x0D0CF4 + comp_index)`. The CPU uses `(a1, d0.w)` so the index is a
signed 16-bit value; all real indices are 0..0x36E0.

### 2.2 Composition header [O]

```
+0 u16 count      total number of 16x16 hardware tiles (sum over entries of block_w*block_h).
                  Only added to the sprite-usage meter $69B2(a5) (debug display). Verified equal for all 3513.
+2 u16 n          number of entries (1..55)
+4 u16 hattr      bit15 = 1: "alt" format (per-entry attr). bit15 = 0: "normal" format (hattr used for all).
                  In alt format the other bits are not read by the draw code (they hold the palette).
+6 u16 layout     layout index (NOT a byte offset): layout_ptr = u32(0x0D43D8 + layout*4)
+8 ...            entries
```
Entries:
- normal (bit15 = 0, 261 comps): `n x u16 code`; attr = hattr for every entry. In all data hattr bits 8-15 are 0 (1x1 tiles).
- alt (bit15 = 1, 3252 comps): `n x (u16 code, u16 attr)`.

Layout: `n x (i16 dx, i16 dy)` pairs, one per entry, offset of the entry's top-left from the object
origin (origin = object X, feet). Layouts are shared between compositions.

### 2.3 Sprite attr word (CPS1 OBJ) [O]

```
bits 0-4  palette (OBJ palette 0..31)
bit  5    X flip      bit 6  Y flip      (both appear in data: 0x20, 0x40, 0x60)
bits 8-11 block width-1  (tiles), bits 12-15 block height-1
```
Block sprite (w x h tiles of 16x16) expands like MAME `cps1_render_sprites`:
tile(i, j) = `(code & ~0xF) + ((code + i) & 0xF) + 0x10*j` for column i, row j (no flip);
with X flip the column order is reversed (i -> w-1-i), with Y flip the row order is reversed.
Tile code -> graphics ROM: 128 bytes per tile at `code*128` in gfx.npy (codes used: 0x802..0x6FF8).

### 2.4 Screen placement (draw 0x14230 / 0x142AA / 0x14494) [O]

```
camXb = u16 $744(a5) - 0x40          ($69B6)        camYb = $748(a5), or 0 if $4DC(a5) != 0 ($69B8)
d4 = obj.x ; d5 = obj.y(+$0C) + obj.z(+$10)
if !(obj.flags25 & 0x80) { d4 -= camXb; d5 -= camYb }
d5 = ((d5 & 0xFF00) | (!d5 & 0x00FF)) + 1        // == (0x100 - v) mod 0x200
for each entry k (dx, dy, code, attr):
    attr = palette override (2.5)
    if obj.facing == 0:  x = d4 + dx
    else:                attr ^= 0x20
                         normal format: x = d4 - dx - 16          (always 16, even for blocks)
                         alt format:    x = d4 - dx - width_px    width_px = ((attr >> 4) & 0xF0) + 16
    y = d5 + dy
    cull (only skips output, image unchanged): if x > 0x1C0 (signed) skip;
         if x <= 0x40 and width_px < 0x40 - x (unsigned) skip
    emit OBJ (x, y, code, attr)
```
Hardware coordinates: visible X 0x40..0x1BF, so screen_x = x - 0x40. Visible Y: MAME shows sprite Y
16..239, so screen_y = y - 16 [I, from MAME visible area]. With these, a standing object with
`Y+Z-camY = v` has its feet at screen row `240 - v`.

Within one composition the first entry is emitted first and so has the highest priority (on top).

### 2.5 Palette choice [O]

The palette is the attr's low 5 bits from the data (embedded per composition / per entry).
`if obj+$23 & 0x80 { attr = (attr & 0xFFE0) | (obj+$23 & 0x1F) }` for every entry (normal, alt,
raw-list and compound paths all do this). Enemies use embedded palettes; colour variants of one enemy
family have separate anim tables whose compositions carry different palettes (e.g. table 0x34892
uses palette 13, 0x34B90 palette 14, 0x34E7C palette 26).

### 2.6 Per-composition extra table 0x11D220 [O/I]

`u32 @ 0x11D220 + comp_index` is copied to obj+$4C..+$4F when a frame is loaded (unless +$25 bit0,
then the secondary "legs" frame provides it):
- +$4C, +$4D (u8): number of trailing entries hidden while wading, for water level 0 / 1 [O].
- +$4E (u8): bit7 = no shadow/splash; high nibble = shadow/splash sprite set (index into 0x14F98 for the
  shadow pass and 0x14C9C for the splash); value 0x70 moves the shadow 4 px down [O].
- +$4F: [U].

### 2.7 Wading clip (0x142BE / 0x1439C) [O]

Applies to the *last* composition drawn for a frame (main comp when there is no overlay, else the overlay):
```
if obj.b53 != 0 && clip_flag_high_byte_negative && obj.y(+$0C) < 0x18 (signed):
    obj.b53 |= 0x80
    if !(obj.flags25 & 0x20) draw_splash(obj)          // 0x14BC6, animated by -$7124(a5) (0,4,8,12 every 10 frames)
    lvl = ((max(obj.y, 0) + 8) >> 4) & 1
    n_draw = n - obj.b4c[lvl]   (if negative: draw nothing)  // drop the last entries (feet/legs)
```
So entries are ordered head -> feet in the data.

---------------------------------------------------------------------------------------------------

## 3. Frame records and animations

### 3.1 Frame record formats [O]

Common part (bytes):
```
+0 u16 w0   main composition index; bit15 set => compound frame (3.3); for the extra table lookup
            the 10-byte loader uses w0 & 0x7FFF
+2 u16 w1   overlay composition index (drawn after the main one, so below it); bit15 set = none.
            For compound frames: self-relative offset (from +2) to the part list.
+4 u8  -> obj+$2C   param A
+5 u8  -> obj+$2D   param B (phase index)
+6 u8  -> obj+$2E   frame type (4.4)
+7 u8  -> obj+$2F   bit7 = LAST frame (a link word follows); bits 0-6 = duration in ticks (0 => 256)
+8 u8  -> obj+$48   body box index         (10/12-byte formats)
+9 u8  -> obj+$49   attack/probe box index (10/12-byte formats)
+10 u16 -> obj+$60  attach point (12-byte format only)
```
- **10-byte records**: enemies and most objects (start 0x120E, tick 0x121E).
- **12-byte records**: players (start 0x13A0..0x13CC / 0x13EA / 0x1416, tick 0x1426).
- **8-byte records**: objects with +$25 bit6: `u32 sprite_list_ptr, u8 $2C, u8 $2D, u8 $2E, u8 dur|end`
  (load 0x16C6, tick 0x16AE). The list is the raw sprite list of 3.5.
- **10-byte, timer-only** variant: start 0x15E4, tick 0x15F4, load 0x160A loads only obj+$2C..$2F
  (no boxes, no extra table).

**Link word**: right after a record whose byte +7 has bit7 set comes `i16 link`; the next frame is
`addr_of_link + link`. It can point back into the same anim (loop) or into another anim (e.g. a punch
that returns to idle). The tick routine returns carry = 1 when it followed a link ("anim finished").

### 3.2 Animation tables [O]

An animation table is `u16 offset[id]`, offsets relative to the table start (the CPU sign-extends:
`adda.w`). Start record = `table + offset[id]`. The table length is not stored: for tools use
`min(nonzero offsets)/2`; a zero offset marks an unused id.

Non-player objects: table = obj+$40. Players: 6 sets x 4 characters, 12-byte records:

| set | ptr table | Jack (0) | Mustapha (1) | Hannah (2) | Mess (3) | role [I from art + code] |
|---|---|---|---|---|---|---|
| 0 | 0x163E | 0x9E41C | 0x9F414 | 0xA06AC | 0xA16F0 | movement: idle, walk, run, jumps, land, pick up, throw item |
| 1 | 0x164E | 0x9EDFE | 0x9FECA | 0xA115A | 0xA1FFC | attacks: punch chain, special, jump attacks |
| 2 | 0x165E | 0x9F094 | 0xA0328 | 0xA13D8 | 0xA232E | damage: hit, knocked down, get up, dizzy (frame type 8) |
| 3 | 0x166E | 0x9F252 | 0xA04FE | 0xA1596 | 0xA24E0 | holding a grabbed enemy, throws (type 0xC) |
| 4 | 0x167E | 0x9F30E | 0xA0596 | 0xA1622 | 0xA25DA | being held (type 0xE) |
| 5 | 0x168E | 0x9EACC | 0x9FB94 | 0xA0E8C | 0xA1D32 | weapon / upper-body poses (split mode) |

`table = u32(set_ptr + 4*obj.b03)`. Valid ids per set (Jack/Mustapha/Hannah/Mess):
set0 32/32/32/32, set1 16/16/16/16, set2 12/12/12/12, set3 5/5/5/5, set4 11/9/9/9, set5 30/31/26/28.
Anomaly [U]: Mustapha set0 id 27 points 2 bytes before a valid record list (its first word is the
previous anim's link word 0xFFF4); probably unused.

### 3.3 Compound frames (w0 bit15, draw path 0x14180) [O]

```
p = frame + 2 + i16(frame + 2)      // w1 is a self-relative offset
n = u16(p) + 1 ; p += 2
repeat n: i16 dx, i16 dy, u16 comp_index, u16 clip_flag      (8 bytes)
   each part: X += (facing ? -dx : dx), Y(+$0C) += dy, draw comp_index (normal or alt), restore X/Y
   clip_flag goes to the wading test (high byte negative = may be clipped)
```
Offsets are per part (not cumulative). Example: enemy 0x34892 anim 6 (run) = body part + bobbing part.
For the extra table the 10-byte loader uses `w0 & 0x7FFF` as the composition index.

### 3.4 Other draw paths [O]

- **Split body** (+$25 bit0, 0x14136): save X/Y/+$53; clear +$53; X += (facing ? -$3C : $3C);
  Y(+$0C) += $3E; draw the +$28 frame (upper body, so it is on top); restore; then draw the +$30
  frame at the object position. Used e.g. for walking while carrying (legs = set0 anim 0xB or 0x18,
  upper = set5 anim; +$38 = per-character offset table at 0x475C/0x477C/0x479C/0x47BC via 0x2A3E).
- **Raw sprite list** (+$25 bit6, 0x147FE): `list = u32(frame+0)`; `u16 count, u16 n`, then
  `n x (i16 dx, i16 dy, u16 attr, u16 code)`; flip `x = d4 - dx - width_px`. Lists may live in RAM.
- **Grab pair** (0x141F8): if obj+$05 == 4 and obj+$2E == 0xC and partner (+$70) is alive with
  +$2E == 0xE: if partner+$62 == 0 draw self then partner, else partner then self (first = on top).
- **Dynamic shadow pass** 0x14EC0 (after all objects, so below them): for each draw-list object in the
  same order, unless +$25 bit4: `fl = $54+$56`; if wading (+$53 != 0, bit7 clear) `fl = max(fl, 0x18)`;
  skip if `fl == obj.y(+$0C)` (shadow is baked into ground poses); else sprite list
  `u32(0x14F98 + (($4E & 0xF0) >> 2))`: `u16 n-1, u16 count, (n x u16 attr, i16 dx, u16 code)` at
  screen Y = (0x100 - (fl + Z - camYb)) - 16 (+4 if $4E == 0x70), X = obj.x - camXb + dx, no flip.

### 3.5 Pseudocode (Rust-like), exact constants

```rust
const EXTRA: u32 = 0x11D220;

// 0x1236 (10-byte), also reached from the tick at 0x121E / link path 0x127C
fn anim_load10(o: &mut Obj, p: u32) {
    o.frame = p;                                   // +$28
    let w0 = rom.u16(p); let w1 = rom.u16(p + 2);
    o.l2c = rom.u32(p + 4) & 0xFFFF_FF7F;          // $2C,$2D,$2E,$2F(timer)
    o.w48 = rom.u16(p + 8);                        // $48 body box, $49 attack box
    let ci = if w0 & 0x8000 != 0 { w0 & 0x7FFF } else if w1 & 0x8000 == 0 { w1 } else { w0 };
    if o.b25 & 1 == 0 { o.l4c = rom.u32(EXTRA + ci as i16 as i32 as u32); }
}
// 0x143E (12-byte)
fn anim_load12(o: &mut Obj, p: u32) {
    o.frame = p;
    let w0 = rom.u16(p); let w1 = rom.u16(p + 2);
    o.l2c = rom.u32(p + 4) & 0xFFFF_FF7F;
    o.w48 = rom.u16(p + 8);
    o.w60 = rom.u16(p + 10);
    let ci = if w1 & 0x8000 == 0 { w1 } else { w0 };          // no 0x7FFF mask here
    if o.b25 & 1 == 0 { o.l4c = rom.u32(EXTRA + ci as i16 as i32 as u32); }
}
// 0x121E (stride 10) / 0x1426 (stride 12). Return value = 68k carry ("followed a link").
fn anim_tick(o: &mut Obj, stride: u32, load: fn(&mut Obj, u32)) -> bool {
    o.b2f = o.b2f.wrapping_sub(1);
    if o.b2f != 0 { return false; }
    let cur = o.frame;
    let mut p = cur + stride;
    if rom.u8(cur + 7) & 0x80 != 0 {               // current frame was the last one
        p = p.wrapping_add(rom.i16(p) as i32 as u32);
        load(o, p);
        return true;
    }
    load(o, p);
    false
}
// 0x120E: o.anim_table = +$40
fn anim_start10(o: &mut Obj, id: u16) { let t = o.anim_table; anim_load10(o, t + rom.i16(t + 2*id) as u32); }
// 0x1416: same with anim_load12.  0x13A0/0x13A8/0x13B0/0x13BC/0x13C4/0x13CC: player set 0..5
// (0x13B0 also clears obj+$25 first)
fn player_anim_start(o: &mut Obj, set: usize, id: u16) {
    const SETS: [u32; 6] = [0x163E, 0x164E, 0x165E, 0x166E, 0x167E, 0x168E];
    let t = rom.u32(SETS[set] + 4 * o.b03 as u32);
    anim_load12(o, t + rom.i16(t + 2*id) as u32);
}
// 0x13EA: set 0, start at frame k (k = signed byte, 12 bytes per frame)
fn player_anim_start_at(o: &mut Obj, id: u16, k: i8) {
    let t = rom.u32(0x163E + 4 * o.b03 as u32);
    anim_load12(o, t + (rom.i16(t + 2*id) as i32 + 12 * k as i32) as u32);
}
// 0x1380: step BACKWARDS (10-byte): o.b2f -= 1; if 0 { p = frame - 10;
//   if rom.i16(p) < 0 { p += 2; p += i16(p); load10(p); carry } else { load10(p) } }
// Secondary (split body) anim: start 0x14C2..0x14EA (sets 0..5), 0x1506 (set 0 at frame d1),
// 0x1532 ($40 table); tick 0x1542 (only if +$25 bit0; timer +$37; stride 12; end flag byte +7).
fn anim_load_secondary(o: &mut Obj, p: u32) {          // 0x1564
    o.frame2 = p;                                      // +$30
    o.l4c = rom.u32(EXTRA + rom.i16(p) as i32 as u32);
    o.w34 = rom.u16(p + 4);
    o.w36 = rom.u16(p + 6) & 0xFF7F;                   // +$37 = timer
    let q = o.offs_table /* +$38 */ + 4 * (rom.u8(p + 5) as i8 as i32) as u32;
    o.w3c = rom.i16(q); o.w3e = rom.i16(q + 2);
}
```
Timing: the object's own state code calls the tick once per game frame, except while its hit-stop
counter +$59 is nonzero (the state code decrements +$59 instead) [O for the player punch states
0x1D88..0x1DCC]. A frame with duration d is shown for d ticks (d = 0 -> 256).

### 3.6 Jack's animations (set ids), with the punch chain from code

Set 0 (0x9E41C) [ids O, labels I from art]: 0 idle (3 x 120-tick holds, then a breathing loop; link ->
0x9E482), 1 walk (12 frames x 4 ticks, type 2, body 0x1B, probe 0x52, loops), 2 run start (-> 9 run loop),
3 vertical jump, 4 forward somersault, 5 backward somersault, 6 landing (-> idle), 8 short jump,
10/19/22/24/25 crouch / pick up, 11 and 16 legs-only walk (split mode), 13/23/30 crouch -> stand,
14/15/31 throw item poses, 20 somersault + land, 21 jump kick (type 4, attack boxes 0x0B/0x0C/0x0D),
26/28 thrown tumble (type 0xE), 27 tumble recovery.

Set 1 (0x9EDFE): 0 jab, 1 jab from extended arm, 2 jab after a hit, 3 hook, 4 finisher (overlay comp
0xA8 on 3 frames), 5 hook after a hit, 6/7 finisher after a hit, 8 special attack (type 6, no body
box), 9 back attack [I], 10-13 jump attacks (type 4), 15 [U].

Punch chain [O, code 0x1CF2..0x1F98]:
```
jab:      set1 anim 0 (or 1 when re-pressed inside its $2C!=0 window)  state 0x02000004, $58 = 0
  on hit (attacker $59 != 0): set1 anim 2, state 0x02000006, combo counter +$B2 += 1
  while in state 6, a press of button bit4 (+$A8) is buffered in +$AD; when the anim ends:
  +$B2 == 1 -> jab again   (set1 anim 0, state 0x02000008, $58 = 0)  on hit -> anim 2
  +$B2 == 2 -> hook        (set1 anim 3, state 0x0200000A, $58 = 2)  on hit -> anim 5
  +$B2 == 3 -> finisher    (set1 anim 4; char 1 uses anim 7 when $2C == 0; state 0x0200000C, $58 = 4)
               on hit -> anim 6; if a grabbable enemy is held and a direction is pressed -> throw instead
```
Set 2 (0x9F094): 0 hit high, 1 hit low, 2 knocked down, 3 back roll, 4 dizzy loop, 5 get up, 6 hit
variant, 7 knocked down variant (all type 8). Sets 3/4: grab-hold (type 0xC) / held (type 0xE) poses.

Example enemy (init 0x3388A, family dispatcher 0x33850 by +$26): +$03 = 0, +$40 = 0x34892,
+$44 = 0x103000, +$4A = 0; 10-byte records. Anims: 0 stand, 1 walk, 2 idle bounce, 3 punch (atk 0x01),
4 punch 2 (atk 0x02), 5 kick (atk 0x03), 6 run (compound frames), 7 kick 2 (atk 0x04), 8 jump kick
(atk 0x05), 14/15 knocked down (type 0xA; its comps carry palette 14) [ids O, labels I].

---------------------------------------------------------------------------------------------------

## 4. Hitboxes and collision

### 4.1 Box record (12 bytes, 6 x i16) [O: 0x12916, 0x14B5E, editor 0xAC094]

```
+0 zoff   depth  min edge, relative to obj+$10
+2 zsize  depth  extent
+4 xoff   X      min edge relative to obj+$08 when facing == 0 (art faces left)
+6 xsize  X      width
+8 yoff   height min edge (bottom) relative to obj+$0C
+A ysize  height extent (up)
```
World intervals: Z `[z+zoff, z+zoff+zsize]`; X `[x+xoff, x+xoff+xsize]` when facing == 0,
`[x-xoff-xsize, x-xoff]` when facing != 0 (mirrored about X); Y `[y+yoff, y+yoff+ysize]`.
Units are pixels. On screen the box top is `feet_y - (yoff + ysize)` (Y up). Index 0 = no box.
Example (Jack): body 0x1A = (1, 6, -12, 23, 0, 75); jab attack 0x02 = (-1, 10, -68, 59, 29, 42);
so a jab connects when the depth difference attacker.z - defender.z is within -8..+8.

### 4.2 Box tables [O]

- single (+$4A == 0): box = `table + 12*index`. Players: `u32(0x18CC4 + obj+$20)` =
  0x100000 (Jack), 0x100C00, 0x101800, 0x102400 (by char id 0..3) [table O, order I + visual check].
  Enemies: e.g. 0x103000, 0x103C00, 0x100000 (set by each enemy init).
- multi (+$4A == 1, e.g. table 0x106000): `p = table + i16(table + 2*index)`; `u16 n-1`, then n boxes.
- The debug editor ("BODY HIT" / "ATACK HIT", 0xAC094..0xAC5D2) edits +$48 / +$49, then the box words:
  +4/+8 with the stick, +6/+A (>= 0) with button, +0/+2 on a separate page.

### 4.3 Collision test 0x128F8 (Rust-like) [O]

```rust
// a = attacker (a2), ia = a.b49 ; b = defender (a3), ib = b.b48. Returns 68k carry (true = hit).
fn boxes_hit(a: &Obj, ia: u8, b: &Obj, ib: u8) -> bool {
    // dispatch index ((a.b4a*2) | b.b4a)*2 -> 0x12916 / 0x129A8 / 0x12A5C / 0x12B0E
    for ba in box_list(a, ia) {               // outer loop = A boxes
        for bb in box_list(b, ib) {           // inner loop = B boxes
            if box_overlap(a, ba, b, bb) {
                save -$7130(a5) = ba ptr, -$712C(a5) = bb ptr;   // used by the hit spark 0x12720
                return true;
            }
        }
    }
    false
}
fn box_list(o: &Obj, idx: u8) -> Vec<BoxPtr> {
    if o.b4a == 0 { vec![o.boxtab + 12 * idx as u32] }
    else { let p = o.boxtab.wrapping_add(rom.i16(o.boxtab + 2 * idx as u32) as u32);
           (0..=rom.u16(p) as u32).map(|k| p + 2 + 12 * k).collect() }
}
// all arithmetic is 16-bit wrapping; the compare is UNSIGNED (bhi), edges touching count as a hit
fn axis(a0: i16, asz: i16, b0: i16, bsz: i16) -> bool {
    (a0.wrapping_sub(b0).wrapping_add(asz) as u16) <= (asz.wrapping_add(bsz) as u16)
}
fn left(o: &Obj, xoff: i16, xsize: i16) -> i16 {
    o.x.wrapping_add(if o.facing != 0 { xoff.wrapping_neg().wrapping_sub(xsize) } else { xoff })
}
fn box_overlap(a: &Obj, ba: Box, b: &Obj, bb: Box) -> bool {
    axis(a.z + ba.zoff, ba.zsize, b.z + bb.zoff, bb.zsize)                          // depth first
    && axis(left(a, ba.xoff, ba.xsize), ba.xsize, left(b, bb.xoff, bb.xsize), bb.xsize)
    && axis(a.y + ba.yoff, ba.ysize, b.y + bb.yoff, bb.ysize)                       // height last
}
```
Hit spark (0x12720, after a confirmed hit; d5 = spark type):
`x = (max(Aleft,Bleft) + min(Aright,Bright)) / 2` (computed with a +0x400 bias),
`y = (max(Abottom,Bbottom) + max(Atop,Btop)) >> 1` (note: max of tops, as coded),
`z = max(min(a.z, b.z) - 1, 0)`; spawns an effect via 0x4884 with +$20 = 4, +$24 = attacker facing,
+$26 = spark type.

### 4.4 Frame type (+$2E) [O unless noted]

| value | meaning |
|---|---|
| 0 | neutral. A nonzero +$49 here is a grab probe (player handler 0x108FA, targets in state 0x0200) [I] |
| 2 | walking. +$49 = grab probe: overlap -> holder state 0x02040000/$2E=0xC, victim 0x02060000/$2E=0xE (0x108AE) |
| 4 | attack active: +$49 is tested against defenders' +$48 |
| 6 | special attack (Jack set1 anim 8: body box 0, so invulnerable) [I] |
| 8 | damage reaction: the hit loops skip defenders of type 8 |
| 0xA | knocked down / falling: also skipped |
| 0xC | holding a grabbed opponent |
| 0xE | being held |

Defender-type dispatch tables (u16 offsets, index = type): 0x10E34 and 0x10FF2 (hit for 0,2,4,6,0xC,0xE,
ignore 8 and 0xA), 0x10D16, 0x1089E, 0x106E2; attacker-type table 0x10820.
Hit loops (outer list x inner list, attacker needs +$2E == 4): 0x10664, 0x10CD0, 0x10DCC, 0x10F96, ...
lists are u16 object pointers at $6720(a5) (count $6718, 3 entries = players), $6722 ($671A),
$6724 ($671C), $6726 ($671E) [lists O, membership beyond players U]. Before testing, the loops require
attacker +$49 != 0, defender +$48 != 0, defender not flagged (+$00 bit1), and a different attack serial
(+$5E vs defender +$E9 or +$5E) so one swing hits a target once.

### 4.5 Attack property record (8 bytes) [O layout, meanings I]

`rec = u32(0xA26FC + 4*char) + 4*obj+$58` (0xA270C Jack, 0xA28EC, 0xA2ADC, 0xA2CBC). +$58 is even,
so record k = +$58/2 (8 bytes each). The frame record does NOT carry damage; the state code sets +$58.
```
+0 i8  damage        subtracted from defender vitality +$6C (clamp 0). Path 0x11008 (inner list $6726)
                     first runs it through 0x2BA2A (counter-hit scale u16 @0xA320C[def.$58]/32 when the
                     defender is attacking, per-type table 0xA32FA by def.$84) [O, details: enemy topic].
                     Path 0x10E4A (defender = player): if attacker +$02 >= 0 (a player) only -1, never below 1.
+1 u8  stun          subtracted from defender +$78 (clamp 0; +$78 starts at 10)          [I: dizzy meter]
+2 u8  reaction      -> defender +$7B and +$06 (hit sub-state); forced 0x22 if defender +$C8 == 0x22
+3 u8  hit-stop      -> +$59 of attacker and defender
+4 u8  sound id      0 = none, else jsr 0x962A
+5 u8  score         passed in d0 to 0xFFC8 (score award for the attacking player, +$104) [I]
+6 u8  spark type    -> effect +$26 via 0x12720
+7 u8  -> defender +$9A [U]
```
Jack: k0 jab `0A 01 00 09 03 14 00 00` (10 dmg, hit-stop 9), k1 hook `0F 03 02 09 06 18 00 00`,
k2 finisher `1D 0B 06 0D 09 20 18 00` (29 dmg, reaction 6 = knock down [I]).

---------------------------------------------------------------------------------------------------

## 5. Draw list and depth sort (0x49C0 / 0x49CA / 0x13FA8) [O]

```rust
// Each object's update code calls one of these after moving:
fn draw_list_add_if_visible(o: &Obj) { if o.b01 != 0 { push(o) } }            // 0x49C0
fn draw_list_add_check(o: &mut Obj) { o.b01 = on_screen(o) as u8; if o.b01 != 0 { push(o) } } // 0x49CA
fn on_screen(o: &Obj) -> bool {                                               // 0x16FA
    let (cx, cy) = if ram.b4dc != 0 { (0, 0) } else { (cam_x /*$744*/, cam_y /*$748*/) };
    let dx = o.x.wrapping_sub(cx).wrapping_add(0x100) as u16;
    let dy = o.y.wrapping_add(o.z).wrapping_sub(cy).wrapping_add(0x40) as u16;
    dx <= 0x380 && dy <= 0x1C0
}
fn push(o: &Obj) {                       // stack grows down from $69B0(a5), count $67C0(a5)
    let key: u32 = ((o.z as u16 as u32) << 16) | (o.ram_addr & 0xFFFF);   // z = obj+$10 (depth)
    list.push(key);
}
// 0x13FA8, once per frame:
fn build_sprite_list() {
    // 1. debug hitbox overlay first (0x149B0, only with debug flags) -> highest priority
    // 2. sort keys ascending as u32 (bubble sort of n-1 passes; result = plain ascending order)
    // 3. for key in ascending order: draw_object(key & 0xFFFF)   (0x141F8)
    // 4. shadow pass 0x14EC0 (same order)                      -> below all objects
    // 5. terminate the OBJ list with (0x0000, 0x0100, 0x0000, 0xFF00); flip buffer flag $69B4
}
```
CPS1 gives the first OBJ entry the highest priority, so: **smaller depth (+$10, nearer the viewer,
lower on screen) is drawn on top; ties are broken by the lower object RAM address on top.** Height
(+$0C) does not affect the order.

---------------------------------------------------------------------------------------------------

## 6. OBJ palettes [O]

- Load: 0x8E4C. `list = u32(0x8E8E + stage*16 + area*4)` (stage = $4D9(a5), area = $786(a5));
  OBJ palette i (i = 0..31) = 32 bytes at `0x0B7A52 + u16(list + 2*i) * 32`, copied to 0x914000 + i*32.
- Colour word `0xBRGB`. ROM palettes have B = 0; the fade-in code (0x88F4 / 0x8982) sets the brightness
  nibble to 0xF. With B = 0xF the MAME formula gives plain 4-bit RGB x 17. Pen 15 is transparent.
- Players use OBJ palettes 0..3, embedded in their compositions: **Jack 0, Hannah 1, Mustapha 2, Mess 3**
  (char ids 0, 2, 1, 3). OBJ 0..3 = ROM palettes 0..3 (0xB7A52, 0xB7A72, 0xB7A92, 0xB7AB2) in every
  stage/area set, except stage 2 area 3 where all 32 slots are ROM palette 0x33 [O].
- OBJ 4..9 are also constant across stages; the rest change per stage/area.

---------------------------------------------------------------------------------------------------

## 7. How to iterate (importer recipe)

1. Read the composition and layout tables (2.1); decode each composition to (dx, dy, code, attr) entries.
2. For an anim: `p = table + i16(table + 2*id)`; read records of the right stride until byte +7 bit7,
   then `next = p_end + i16(p_end)` where p_end = record + stride. Stop when `next` was already seen.
3. Per record: draw w0 (or the compound list), then w1 if >= 0, with facing/palette rules (2.4, 2.5);
   boxes from +$44 with indices bytes +8/+9; type byte +6; duration byte +7 & 0x7F.

## 8. Open gaps

- Meaning of frame byte +4 ($2C) outside the punch states, byte +11 (low byte of +$60), +$4F.
- Full membership of the object lists $6722/$6724/$6726 and the per-pair hit handlers (enemy topic).
- Whether the game applies a +$23 palette override to enemy variant 0 when it is knocked down (its
  fall compositions carry palette 14, not 13).
- Labels of some player anims (set1 ids 9/14/15, set0 ids 27-29) are from the art only.
- Mustapha set0 id 27 table offset anomaly (3.2).
