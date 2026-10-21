# Cadillacs and Dinosaurs (CPS1, set "dino") - palettes, background layers, stage maps, camera

Scope: everything needed to import the background graphics/maps/palettes from the user's ROM and to
re-implement (natively, no emulation) the camera, background streaming, parallax and palette fades.

Tags used on every claim:
- **[O] observed**: read directly in the 68000 code or data (address given), or verified by rendering.
- **[I] inferred**: strong deduction (from code shape, MAME knowledge, or rendering) but not traced end to end.
- **[U] unknown**: not resolved.

Conventions:
- All ROM values are big-endian. `w(a)` = u16 at program address a, `l(a)` = u32, `b(a)` = u8.
- RAM names: `$xxx` = 0xFF8000 + xxx (A5 base). Example: `$744` = 0xFF8744.
- 68000 integer semantics matter. In the pseudocode `u16`/`i16` are 16-bit with wrapping (`wrapping_add`
  etc.). "unsigned compare" means the original used `bcs/bcc/bhi/bls`; "signed" means `blt/bge/bgt/ble/bmi/bpl`.
- Program image = `re/out/dino_68k.bin` (0x180000 bytes); graphics = `re/out/gfx.npy` (4 MB, see CONTEXT.md).
- Reference implementation of every format in this file: `re/specs/stage/render_stage.py`.

---------------------------------------------------------------------------------------------------

## 1. Hardware model the game uses

### 1.1 Tilemap entry (all three scroll layers) [O]
Each tilemap cell is 2 big-endian words (4 bytes): `code`, then `attr`.

| attr bits | meaning | tag |
|---|---|---|
| 0-4  | palette (0-31) inside the layer's palette bank | [O] (render matches) |
| 5    | X flip | [O] |
| 6    | Y flip | [O] |
| 7-8  | priority group 0-3 (selects one of the 4 CPS-B priority masks, section 1.4) | [I] (MAME) |
| 9    | unused by hardware | [I] |
| 10-15 (scroll2) | game data, not hardware. Debug camera mode 0x0E (routine 0x21474) draws `0x6F80 + (attr >> 10)` instead of the tile, so these 6 bits are a per-tile attribute (probably terrain/collision class). No reader found yet | [O] debug view, meaning [U] |

Scroll3 entries: the hardware uses `code & 0x3FFF` [I, MAME].

### 1.2 Tilemap memory layout (VRAM) [O]
Each layer is a 64x64-cell map of 4-byte entries (0x4000 bytes). Offsets in bytes from the layer base:

| layer | base | cell | map size | byte offset of cell (col, row) | derived from |
|---|---|---|---|---|---|
| scroll1 | 0x908000 | 8x8   | 512x512   | `col*0x80 + (row&31)*4 + (row>>5)*0x2000` | 0x20650 [O] |
| scroll2 | 0x90C000 | 16x16 | 1024x1024 | `col*0x40 + (row&15)*4 + (row>>4)*0x1000` | 0x214C0 [O] |
| scroll3 | 0x910000 | 32x32 | 2048x2048 | `col*0x20 + (row&7)*4  + (row>>3)*0x800`  | 0x23090 [O] |

(These are MAME's `tilemap0/1/2_scan` functions; the game code computes exactly these offsets.)

Base registers: CPS-A 0x800102/4/6 = `$2a/$2c/$2e`; boot values 0x9080/0x90C0/0x9100 [O 0x47A].
`$2a` is rewritten every vblank to 0x9080 (or 0x9180 in a debug case) [O 0x58E].

### 1.3 World coordinates and scroll registers [O]
World X grows to the right, world Y grows **upward** (larger Y = higher on screen). The world-to-VRAM
mapping for a pixel at world (x, y) of each layer:

```
scroll1: col = (x & 0x1FF) >> 3,  row = (0x1FF - (y & 0x1FF)) >> 3
scroll2: col = (x & 0x3FF) >> 4,  row = (0x3FF - (y & 0x3FF)) >> 4
scroll3: col = (x & 0x7FF) >> 5,  row = (0x7FF - (y & 0x7FF)) >> 5
```
(from the address routines: they use `0x3E0 - (y & 0x3E0)` etc. for the block top row) [O]

Camera variables (16.16 fixed point longs; the integer part is the high word) [O 0x6C0]:

| layer | camera X | camera Y | controller struct (a6) |
|---|---|---|---|
| scroll1 | `$6c4` | `$6c8` | `$6bc` |
| scroll2 (playfield, "the camera") | `$744` | `$748` | `$73c` |
| scroll3 | `$7e4` | `$7e8` | `$7dc` |

VBlank (0x6C0) writes, using the value latched at the previous vblank [O]:
```
scroll1X = cam1X_prev + 0xFFC0  (= -0x40)     scroll1Y = 0x100 - cam1Y_prev
scroll2X = cam2X_prev - 0x40                   scroll2Y = 0x300 - cam2Y_prev
scroll3X = cam3X_prev - 0x40                   scroll3Y = 0x700 - cam3Y_prev
```
`camN_prev` is copied from the camera at the end of each vblank (`$85c/$860`, `$864/$868`, `$86c/$870`).
The one-frame delay matches the CPS1 sprite buffer delay, so for a native port use the **same camera
value for background and sprites of a frame** [I].

Resulting screen mapping (same for all three layers, 384x224 visible) [I, uses MAME's 64/16 pixel
visible-area offsets; verified by composites that look like the arcade]:
```
screen x (0..383)  shows world x = camX + sx
screen line s (0..223) shows world y = camY + 0xEF - s      (top line = camY+239, bottom = camY+16)
```
Sprites use the same camera: OBJ X = objX - camX + dx + 0x40, OBJ Y = 0x100 - (objY + obj.$10 - camY) + dy
(0x14258; note the routine uses `not.b`, i.e. only the low byte is inverted) [O].

### 1.4 Layer order, enables, priority masks [O/I]
- CPS-B layer control 0x80014A = `$28` each frame (0x80A). Values: 0x12D6 normally (back to front:
  scroll3, scroll2, sprites, scroll1) and 0x06D6 in stage 6 area 1 (scroll3, scroll2, scroll1, sprites)
  [O 0x2076A/0x2078A; order decode I from MAME].
- Priority masks (CPS-B-21: 0x80014C, 0x80014E, 0x800140, 0x800142 = group 0..3) written by 0x8D92 from an
  8-byte record: `w0->0x80014C, w1->0x80014E, w2->0x800140, w3->0x800142` [O]. Record = `0x8DEC + stage*8`,
  except stage 6 uses `0x8DCC + area*8` [O]. Meaning: for a scroll-layer tile in group g, pen p is drawn
  in front of sprites if `mask[g] & (1<<p)` (pen 15 is always transparent) [I, MAME `set_transmask`].

| stage | grp0 | grp1 | grp2 | grp3 |
|---|---|---|---|---|
| 0 | 0000 | 7F7F | 7FFF | 59FF |
| 1 | 0000 | 00FE | 7FFE | 7F00 |
| 2 | 0000 | 0FFF | 01FF | 0000 |
| 3 | 0000 | 01FF | 00FF | 7FFF |
| 4 | 0000 | 00FF | 00BE | 01FF |
| 5 | 0000 | 00FE | 7F80 | 7F87 |
| 6 (areas 0,2,3) | 0000 | 78FF | 007F | 001F |
| 6 area 1 | 0000 | 001F | 8000 | 7FFF |
| 7 | 0000 | 7E01 | 00FF | 01FF |
| 8 (special scene) | 0000 | 7F00 | 7FFE | 0001 |

- Video control 0x800122 = `$26 | ((DIP $777c bit5) ? 0x8000 : 0)` (0x826). `$26` bit 0 = scroll2
  rowscroll enable: set for stage 2 (0x20718, 0x223BE), cleared otherwise [O].
- 0x800144 = 0x003F and 0x80010A = 0x9140 every frame (0x814): palette upload of all pages [O/I].

### 1.5 Rowscroll [O/I]
"Other" RAM base 0x800108 = 0x9158 -> table at 0x915800 (1024 words). Offset register 0x800120 = `$777e`,
which is only ever cleared (0x4AC) -> always 0 [O]. Per MAME [I]: visible line s uses
`scroll2X + other[(s + 16) & 0x3FF]`. Only stage 2 writes it (section 7.3).

---------------------------------------------------------------------------------------------------

## 2. Palettes

### 2.1 Palette RAM and colour format [O]
0x914000: 4 banks x 32 palettes x 16 colours x u16. Bank 0x914000 OBJ, 0x914400 scroll1,
0x914800 scroll2, 0x914C00 scroll3. Colour word `0xBRGB` (brightness nibble then R, G, B). To RGB888
(MAME formula, used by the renderer):
```
bright = 0x0F + ((w >> 12) << 1)
r8 = ((w >> 8) & 0xF) * 0x11 * bright / 0x2D   (same for g, b; integer division)
```
Pen 15 of scroll layers is transparent.

**All palette data in ROM has brightness nibble 0** (example 0xBBA52 = 0FFF 0DDD ...). Every load is
followed by either `0x892C` (OR 0xF000 into all 4 banks) or a fade-in that adds 0x1000 per step. So the
importer should OR 0xF000 for the normal look [O].

### 2.2 Palette sources in ROM [O]
| what | table | entry | data |
|---|---|---|---|
| OBJ | 0x8E8E: long[stage*4 + area] (stages 0-7) | -> 32 words = palette numbers p | colours at `0xB7A52 + p*32` (32 bytes each) |
| scroll1 | fixed | - | bank at 0xBBA52 (0x400 bytes) |
| scroll2 | 0x8AE0: long[stage*4 + area] (20 stage slots, 8-19 are special scenes) | -> 0x400-byte bank | 0xBCA52.. (0x400 apart) |
| scroll3 | 0x8C52: long[stage*4 + area] (20 slots) | -> 0x400-byte bank | 0xC2252.. |
| priority masks | 0x8DEC / 0x8DCC | see 1.4 | |

Per-area values for stages 0-7 are in the table of section 4.6.

### 2.3 Load routines [O]
```
0x8A44 load_all():          clr.w 0x9157FE; obj(); s1(); s2(); s3(); prio()
0x8E4C obj():   list = l(0x8E8E + (stage*4+area)*4); for i in 0..32: copy 32 bytes 0xB7A52 + w(list+2i)*32 -> 0x914000 + i*32
0x8A8E s1():    copy 0x400 bytes 0xBBA52 -> 0x914400
0x8AAE s2():    copy 0x400 bytes l(0x8AE0 + (stage*16 + area*4)) -> 0x914800
0x8C20 s3():    copy 0x400 bytes l(0x8C52 + (stage*16 + area*4)) -> 0x914C00
0x8D92 prio():  section 1.4
0x89E6 load_set(d0): uses 8-byte record 0x8A5E + d0*8 = 4 (stage, area) pairs; temporarily sets
                $4d9/$786 to pair0 for obj(), pair1 for s1()+s2(), pair2 for s3(), pair3 for prio(), then restores.
```
`stage` = `$4d9`, `area` = `$786` (= scroll2 controller field `$4a`).
0x89E6 sets (special screens): 0:(1,3)(12,0)(11,0)(8,0) 1:(0,3)(13,0)(12,0)(10,0) 2:(0,0)(13,0)(13,0)(0,0)
3:(2,0)(18,0)(9,0)(9,0) 4:(2,0)x4 5:(0,0)x4 [O].

### 2.4 Brightness and fades [O]
Fade control block at `$44a` (0xFF844A):
```
+0  u8  bank mask: bit0 OBJ, bit1 scroll1, bit2 scroll2, bit3 scroll3 (0 = idle; callers poll it)
+1  u8  steps remaining
+2  u16 frames to sleep between steps (0 = do all steps in the same frame)
+4  u32[4] bank addresses 0x914000, 0x914400, 0x914800, 0x914C00
```
```rust
// 0x88F4 - one step on one bank (512 words). delta = +0x1000 (fade in) or -0x1000 (fade out, stored as 0xFFFFF000)
fn fade_bank_step(bank: &mut [u16; 512], delta_in: bool) {
    let first = bank[0];
    let probe = if delta_in { first.wrapping_add(0x1000) } else { first };
    if probe & 0xF000 == 0 { return; }            // fade-in: already at F; fade-out: already at 0
    for c in bank.iter_mut() { *c = if delta_in { c.wrapping_add(0x1000) } else { c.wrapping_sub(0x1000) }; }
}   // NOTE: only colour 0 is tested; every word is changed (can wrap words with a different nibble) - keep it.

// 0x88B0 - one step on all banks selected by mask, in order OBJ, s1, s2, s3
fn fade_step(f: &mut Fade, delta_in: bool) { for i in 0..4 { if f.mask & (1<<i) != 0 { fade_bank_step(bank(i), delta_in) } } }

// Asynchronous fade task (created with trap #0, e.g. slot 0x40):
//   0x8824 fade-in,  steps=16, delay=1      0x8838 fade-in,  caller sets $44a/$44b/$44c
//   0x8840 fade-out, steps=16, delay=1      0x8854 fade-out, caller sets $44a/$44b/$44c
// 0x8824/0x8840 themselves write steps=16 and delay=1. Callers set the mask first ($0F, or $0D in
// camera mode 0x13) and often also write a delay (2 in modes 1/0x13, 12 in mode 9), but that store is
// overwritten when the task starts, so these fades always take 16 steps x 1 frame [O 0x882A/0x8846].
fn fade_task(delta_in: bool) {        // body 0x885A
    let f = fade_block();
    if f.mask != 0 && f.delay != 0 && f.steps != 0 {
        loop {
            fade_step(f, delta_in);          // first step is applied immediately
            f.steps -= 1; if f.steps == 0 { break; }
            sleep_frames(f.delay);           // 0x95A
        }
    }
    f.mask = 0;                              // signals "fade finished"
    exit_task();                             // 0x910
}
// Synchronous variants (run inside the caller's task): 0x879A in 16 steps delay 1 all banks;
// 0x87B2 in d0 steps; 0x87BA out 16 steps delay 1 all banks; 0x87D2 out d0 steps. Body 0x87D8:
// same loop, but if delay == 0 it loops without sleeping.

// Instant brightness:
// 0x892C: mask = 0x0F; for selected banks: every word |= 0xF000          (full brightness)
// 0x8932: same OR, keeps the current mask
// 0x8938: for banks in current mask: every word &= 0x0FFF                 (brightness 0 = 1/3 level)
```
Note: brightness 0 is **not black** with the CPS1 formula (factor 15/45) [I MAME].

### 2.5 Palette animation (cycling) [O, partial]
Implemented by ordinary objects in pool B (props, 24 x 0xC0 at `$36f4`, type word +$20, jump table
0x2362E). Found so far:
- **0x89746** (pool B type 0x11C, table pointer at 0x2374A): every 8 frames (`+$84` counter) copy
  palette frame k (k = 0..5, then restart; `+$8a` counter) for 13 destination palettes. Source table
  0x898C8: 13 rows x 6 words (12 bytes per row); destination i gets `0xBCA52 + row_i[k]` (or `0xC2252 + ...`
  for the last two). Destinations in order: s2 pal 1, 5, 6, 7, 8, 10, 12, 13, 14, 15, 25, s3 pal 1, s3 pal 17.
  Copy rule (0x89964): `dst[j] = (dst[j] & 0xF000) | src[j]` for 16 colours (keeps the current brightness).
  Used in stage 0 (bank 0xBCA52 is the stage-0 scroll2 bank).
- 0x8B058 (same pattern, s2 pals 20-25, stage-0 bank, table at `+$aa`), 0x0E0FE..0x0E17A (s3 pals),
  0x0EC92/0x0ECC4 (s3 pals 2, 3), 0x92308/0x9233A (save/restore s2 pals 2, 3, 11 to 0xFF0E68 around
  stage 2's map screen). Exact triggers [U].
- There is **no** palette cycling in the background controllers themselves.

### 2.6 Tile animation (water/fire style) [O]
Animated background tiles are done by objects that rewrite scroll2 VRAM codes (e.g. 0x7863E writes 13
rows x 3 codes at 0x90D4B4 from one of 4 frame tables at 0x78664). Phase counters live in the scroll2
controller and tick only in two stages:
```rust
// called at the start of scroll2_run (0x20898)
fn tile_anim_tick(s: &mut Scroll2) {
    match stage {
        4 => { s.t71 -= 1; if s.t71 == 0 { s.t71 = 8;    s.phase72 = (s.phase72 + 4) & 0xF; return; } } // 0x20956
        7 => { s.t75 -= 1; if s.t75 == 0 { s.t75 = 0x18; s.phase72 = (s.phase72 + 4) & 0xF; return; } } // 0x2097C
        _ => return,
    }
    s.prev73 = s.phase72;     // objects redraw when phase72 != prev73 (i.e. on the frame it changed)
}   // init: t71 = 1, t75 = 1, phase72 = 0, prev73 = 0xFF
```
Map blocks whose code is 0 (out of the scroll2 graphics range, drawn transparent) are the holes these
objects fill (seen in stages 1, 3, 4) [O render / I purpose].

---------------------------------------------------------------------------------------------------

## 3. Tile code -> graphics ROM address [O by rendering, mapper ranges I]
Graphics ROM = 4 MB, planar 4 bpp: 4 bytes per 8 pixels, byte p = bit plane p (weight 1<<p), MSB = left pixel.

| layer | tile | bytes | address of tile `code` | row stride | pixel x layout |
|---|---|---|---|---|---|
| scroll1 | 8x8  | 64  | `code * 64` | 8 bytes | even tilemap column: bytes 0-3 of each row; odd column: bytes 4-7 |
| scroll2 | 16x16| 128 | `code * 128` | 8 bytes | bytes 0-3 = x 0-7, bytes 4-7 = x 8-15 |
| scroll3 | 32x32| 512 | `(code & 0x3FFF) * 512` | 16 bytes | 4 groups of 4 bytes = x 0-7, 8-15, 16-23, 24-31 |
| OBJ     | 16x16| 128 | `code * 128` | 8 bytes | as scroll2 |

So a scroll1 code covers a 16x8 strip: the half shown depends on the column parity (the font stores each
small glyph twice, big glyphs as left/right halves). Verified: `png/scroll1_font.png`, stage-6 elevator.

Bank mapper (CPS-B "CD63B"): within the ranges below the address is the identity formula above; a code
outside its layer's range is not drawn (transparent). Ranges in 64-byte units (`code<<0` scroll1,
`code<<1` sprites/scroll2, `code<<3` scroll3):

| layer | units | codes | evidence |
|---|---|---|---|
| scroll1 | 0x0000-0x0FFF | 0x000-0xFFF | font + elevator codes 0x20-0xFF1 [O]; range end [I] |
| sprites | 0x1000-0xDFFF | 0x0800-0x6FFF | [I] |
| scroll2 | 0x8000-0xDFFF | 0x4000-0x6FFF | all map codes are 0x5800-0x6F7F or 0 [O]; clear tile 0x5800 is all pen 15 [O] |
| scroll3 | 0xE000-0xFFFF | 0x1C00-0x1FFF | all map codes are 0x1C00-0x1FE7 or 0 [O]; clear tile 0x1C00 all pen 15 [O]; **but see below** |

Code 0 in scroll2/scroll3 data must be treated as "not drawn": tile 0 of the ROM is a font glyph and would
show garbage (checked) [O].

Correction for scroll3 (2026-10-21, M10-06 pictures) [O]: MAME draws scroll3 cells of every code 0x0000-0x1FFF (code & 0x3FFF; the
ROM holds 0x2000 tiles of 512 bytes): a cell 0 (attr 0xFFFF, palette 31, flipped) shows tile 0 of the ROM, pens 0 and 5, where scroll2 is
transparent (attract recording, video frames 12900 and 12910), and the stale cells of the last clear of the movie show too (13380). The
renderer draws every scroll3 code below 0x2000 (`dino_assets::TILES32_COUNT`); those three pictures are exact with it, no other of the
picture checks changes. So the scroll3 row of the table above (and maybe the [I] ranges of the other layers) is not the CD63B of MAME.
Scroll2 and scroll1 keep the ranges above: drawing every code there changes none of the checked pictures (no visible cell has such a code).

---------------------------------------------------------------------------------------------------

## 4. Stage map format (ROM)

Terminology: a **screen** is 256x256 px = 8x8 **blocks** of 32x32 px. Maps are W x H screens.
Stage = `$4d9` (0-7 for the game; in-game order 1-8 [I]), area = `$786`.

### 4.1 Area header (10 bytes) - same layout for scroll2 and scroll3 [O 0x207CA, 0x22CC0]
```
+0 u8  W   map width in screens
+1 u8  H   map height in screens
+2 u32 screen-array pointer: u16[W*H], column-major: index = sx*H + sy, sy = 0 is the BOTTOM row
+6 u16 initial camera X (world)
+8 u16 initial camera Y (world)
```
- scroll2 headers: `l(0x21580 + stage*4) + area*10`.
- scroll3 headers: `l(0x2311E + stage*4) + area*10`.
- W = 0 (scroll2): "sub-area" that keeps the previous map; it is entered by camera mode 8 (section 6.6),
  which only increments the area number and reloads OBJ palettes [O]. W = 0 (scroll3): no far layer for that
  area. The original still runs its fill loop reading screen ids from address 0 (hidden behind opaque
  scroll2); an importer should treat it as empty [I].
- Stage 2 uses its own header copy at 0x22AFA (same content as 0x215E8) [O 0x22404].

### 4.2 Screen lookup [O 0x214F0, 0x230BE]
```rust
fn screen_id(hdr, x: u16, y: u16, layer) -> u16 {
    let mut xs = x & 0xFFE0;                                  // scroll3 uses x unmasked
    if stage == 2 { while xs >= LOOP_END { xs -= LOOP_BACK } } // scroll2: 0x4D00 / 0xF00, scroll3: 0xA00 / 0x100 (unsigned)
    let col = (xs as i16 >> 8) as u16;                         // asr
    let row = ((y & 0xFFE0) as i16 >> 8) as u16;               // scroll3: (y as i16 >> 8)
    let id = w(hdr.screens + 2 * (col * hdr.h as u16 + row));
    if layer == Scroll3 { id & 0x7F } else { id }
}
```
(Stage 2 is the endless desert road: the scroll2 map loops screens 0x3E..0x4C, scroll3 loops screen 9.)

### 4.3 scroll2 screens and blocks [O]
- Screen data: `l(0x2155E + stage*4) + id*128` = u16[64] block numbers, **column-major, top row first**:
  word index `cx*8 + p` where cx = block column 0..7 (left to right) and p = block row 0..7 from the top.
  For world (x, y): `cx = (x & 0xE0) >> 5`, `p = 7 - ((y & 0xE0) >> 5)` (0x2152C).
- Block data: `l(0x21454 + stage*4) + block*16` = 4 tilemap entries (code, attr), column-major:
  `[ (col0,row0), (col0,row1), (col1,row0), (col1,row1) ]` with row0 the upper tile (0x2142A).

| stage | block table | screen table |
|---|---|---|
| 0 | 0x162068 | 0x138FC8 |
| 1 | 0x164198 | 0x139AC8 |
| 2 | 0x166758 | 0x13BAC8 |
| 3 | 0x166C98 | 0x13BDC8 |
| 4 | 0x1683E8 | 0x13DDC8 |
| 5 | 0x16B3D8 | 0x13FDC8 |
| 6 | 0x16E188 | 0x141DC8 |
| 7 | 0x172158 | 0x143DC8 |

### 4.4 scroll3 screens [O 0x230F6]
One table for all stages: `0x177288 + id*256` = 64 tilemap entries (4 bytes each), column-major, top row
first: entry index `cx*8 + p` (byte offset `cx*32 + p*4`), `cx = (x & 0xE0) >> 5`, `p = 7 - ((y & 0xE0) >> 5)`.
No block indirection (one 32x32 tile per block).

### 4.5 scroll1 used as a background (stage 6 area 1 only) [O 0x205C6]
- Screens: `0x138C48 + id*128` = u16[64] metatile numbers, same order as 4.3.
- Metatiles: `0x15CFE8 + n*64` = 16 scroll1 entries, column-major 4x4: entry j -> column `j/4`, row `j%4`.
- Init draws screen 1 at (x=0, y=0x100) and screen 2 at (x=0x100, y=0x100), 8 columns x 9 metatile rows
  each, starting at the screen's bottom metatile row (so rows wrap), with layer control 0x06D6 (scroll1
  behind sprites). It is the elevator cage (`png/stage6/st6_a1_scroll1_elevator.png`). The palette is the
  fixed scroll1 bank. Who sets the scroll1 camera Y for this area [U] (`$6c8` is set to 0x100 by 0xAC54 /
  0xCC70 and to 0 by 0xAE0 / 0xC536 / 0xCC5C).
- 0x204CA / 0x20454 / 0x20592 are fixed-table variants of the same writers (scroll2 blocks 0x175378 with
  screens 0x145DC8; scroll1 metatiles with screens 0x138C48; scroll3 screens 0x177288), used by non-stage
  scenes [O code, use I].

### 4.6 All stage areas [O]
`s3 shift` = parallax shift (section 7.1), `Yf` = scroll3 Y-follow flag (0x22E54).

| st.area | s2 hdr | s2 WxH | s2 init X,Y | s3 hdr | s3 WxH | s3 init X,Y | s3 shift | Yf | s2 pal | s3 pal | OBJ list |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 0.0 | 215AC | 7x2 | 0,100 | 2313E | 3x1 | 0,0 | 3 | 0 | 0BCA52 | 0C2252 | 08F0E |
| 0.1 | 215B6 | 4x1 | 0,0 | 23148 | none | - | 0 | 0 | 0BCA52 | 0C2252 | 08F4E |
| 0.2 | 215C0 | 4x4 | 0,300 | 23152 | 2x3 | 0,120 | 3 | 1 | 0BCA52 | 0C2252 | 08F8E |
| 1.0 | 215CA | 15x2 | 0,40 | 2315C | 6x2 | 0,0 | 1 | 0 | 0BCE52 | 0C2652 | 0900E |
| 1.1 | 215D4 | sub-area | - | 23166 | none | - | 1 | 0 | 0BCE52 | 0C2652 | 0904E |
| 1.2 | 215DE | 6x2 | 0,100 | 23170 | 2x2 | 0,0 | 5 | 0 | 0C0652 | 0C2652 | 0908E |
| 2.0 | 215E8 | 60x1 | 0,0 | 2317A | 18x1 | 0,0 | 3 | 0 | 0BD252 | 0C2A52 | 0910E |
| 3.0 | 215F2 | 4x1 | 0,0 | 23184 | 3x1 | 0,0 | 1 | 0 | 0BD652 | 0C2E52 | 0918E |
| 3.1 | 215FC | 10x1 | 0,0 | 2318E | 6x1 | 0,0 | 1 | 0 | 0BD652 | 0C2E52 | 091CE |
| 3.2 | 21606 | 7x1 | 0,0 | 23198 | 3x1 | 0,0 | 2 | 0 | 0BD652 | 0C2E52 | 0920E |
| 4.0 | 21610 | 23x3 | 0,200 | 231A2 | 11x1 | 0,0 | 1 | 0 | 0BDA52 | 0C3252 | 0928E |
| 4.1 | 2161A | sub-area | - | 231AC | none | - | 1 | 0 | 0BDA52 | 0C3252 | 092CE |
| 5.0 | 21624 | 9x1 | 0,0 | 231B6 | 4x1 | 0,0 | 1 | 0 | 0BDE52 | 0C3652 | 0930E |
| 5.1 | 2162E | 19x3 | 0,100 | 231C0 | 10x1 | 0,0 | 1 | 0 | 0BEE52 | 0C3652 | 0934E |
| 5.2 | 21638 | sub-area | - | 231CA | none | - | 1 | 0 | 0BEE52 | 0C3652 | 0938E |
| 6.0 | 21642 | 12x1 | 0,0 | 231D4 | 3x1 | 0,0 | 2 | 0 | 0BE252 | 0C3A52 | 093CE |
| 6.1 | 2164C | 2x56 | 0,3700 | 231DE | 2x3 | 0,1A0 | 4 | 1 | 0BE252 | 0C3A52 | 0940E |
| 6.2 | 21656 | 9x1 | 0,0 | 231E8 | none | - | 0 | 0 | 0BE252 | 0C3A52 | 0944E |
| 6.3 | 21660 | 7x1 | 0,0 | 231F2 | none | - | 0 | 0 | 0BF252 | 0C3A52 | 0948E |
| 7.0 | 2166A | 7x1 | 0,0 | 231FC | 4x1 | 0,0 | 1 | 0 | 0BE652 | 0C3E52 | 094CE |
| 7.1 | 21674 | 14x4 | 0,300 | 23206 | 8x3 | 0,200 | 1 | 1 | 0BE652 | 0C3E52 | 0950E |
| 7.2 | 2167E | 20x3 | 0,200 | 23210 | 11x1 | 0,0 | 1 | 0 | 0BF652 | 0C3E52 | 0954E |

Scenes seen in the renders [O]: 0 flooded-city rooftops / mansion interior / street; 1 jungle;
2 desert road (Cadillac drive, rowscroll); 3 fortress gate; 4 savanna then rocks; 5 mine with lamps;
6 orange city then 56-screen elevator shaft; 7 lab with specimen tanks. Headers for stage slots 8-10
(0x21688.., 1 screen each) belong to special scenes [I].

---------------------------------------------------------------------------------------------------

## 5. Background controllers and per-frame order [O]

Game loop state 3 (0xAAF6) runs every frame: 0x186CC, **0xE90E** (objects; at its end 0x2042E =
background/camera update), 0x10546 (no camera or VRAM-map writes found there [O grep]; not this topic), 0x16BEA, 0x13FA6 (sprite list, reads the
final camera), 0x105E8, 0x103FE, 0x1037E, 0xFBF4 (camera-driven object spawn lists, 0xFC2E/0xFC4E; another
topic). So the camera is updated after all objects moved and before sprites are built.

```rust
fn background_update() {              // 0x2042E
    if stage == 2 { rowscroll_task(); scroll2_stage2_task(); scroll3_task(); }   // 0x2335A, 0x22398, 0x22CAA
    else          { scroll2_task(); scroll3_task(); }                             // 0x206D4, 0x22CAA
}
```
Each controller is a small task-like struct with `+4 state` (0 = init, 2 = run). 0x20402 clears
`$740/$744/$748/$796/$7e0/$7e4/$7e8/$836` (state 0 of scroll2/3 and cameras) and clears scroll2/3 VRAM
(0xB6C fills code 0x5800 attr 0; 0xB88 fills code 0x1C00 attr 0). Mode 1/0x12 area changes clear `$7e0`
(scroll3 state) so scroll3 re-inits with the new area.

### 5.1 scroll2 controller struct (a6 = `$73c`) [O]
| off | RAM | type | meaning |
|---|---|---|---|
| +04 | $740 | u8 | state (0 init, 2 run) |
| +05 | $741 | u8 | camera mode (0x00-0x13, table 0x2092E) |
| +06 | $742 | u8 | mode sub-state (steps of 2) |
| +08 | $744 | i32 16.16 | camX |
| +0C | $748 | i32 16.16 | camY |
| +10 | $74c | i32 | camera Z (only read by spawners) [I] |
| +14 | $750 | i16 | camX before this frame's X follow; also velocity X (8.8) in mode 5 |
| +16 | $752 | i16 | camY before Y follow; velocity Y (8.8) in mode 5 |
| +18 | $754 | i16 | velocity Z (mode 5) |
| +1A/+1C/+1E | | i16 | acceleration X/Y/Z (mode 5) |
| +40 | $77c | u8 | map W (screens) |
| +41 | $77d | u8 | map H |
| +42 | $77e | u32 | screen array pointer |
| +46 | $782 | u16 | last streamed X (multiple of 32) |
| +48 | $784 | u16 | last streamed Y (multiple of 32) |
| +4A | $786 | u8 | **area** |
| +4B | $787 | u8 | vertical follow enabled |
| +4C | $788 | u16 | event X (lock / snap X) |
| +4E | $78a | u16 | event Y (target / snap Y) |
| +50 | $78c | u16 | push threshold (screen x of the leading player) |
| +52 | $78e | u16 | vertical up threshold |
| +54 | $790 | u16 | vertical down threshold |
| +56 | $792 | u32 | pointer to the next event record |
| +5A | $796 | u16 | camX when the event loaded (= event X) |
| +5C | $798 | u16 | camY when the event loaded |
| +60 | $79c | u16 | players max X |
| +62 | $79e | u16 | players max Yt |
| +64 | $7a0 | u16 | players min X (adjusted) |
| +66 | $7a2 | u16 | players min Yt |
| +6C | $7a8 | u8 | event index |
| +6D | $7a9 | u8 | **go flag** (set to 1 by stage-flow code 0x1DE62, 0x1E4B4, 0x69FA4; cleared by the camera) |
| +70 | $7ac | u8 | event overrides push threshold |
| +71..75 | | u8 | tile animation timers/phase (2.6) |
| +74 | $7b0 | u8 | slope shift for mode 0x0B (table 0x20F58) |
| +78 | $7b4 | u8 | camera lock state: 0 free, 1 locked (wave/area end), 2 stage end (read by other modules) |
| +79 | $7b5 | u8 | events trigger on Y instead of X; also freezes scroll3 parallax |
| +7B | $7b7 | u8 | mode to use after a mode-4 lock |

`Yt(player) = player.w[0x54] + player.w[0x10]`; players are the 3 fixed objects at `$3274/$33f4/$3574`.

### 5.2 Camera event table [O 0x2099A, 0x209C0]
`0x21B2A + w(0x21B2A + stage*2)` (stage 2: 0x22B9E), records of 16 bytes, terminated by X = 0xFFFF:
```
+0  u16 x      trigger X (and lock X)        -> $4c, $5a
+2  u16 y      trigger Y / target Y          -> $4e
+4  u16 push   push threshold if flag70      -> $50
+6  u16 up     vertical up threshold         -> $52
+8  u16 down   vertical down threshold       -> $54
+A  u8  area   area this event belongs to    -> $4a (= $786, changes the area immediately)
+B  u8  mode   camera mode                    -> $5 (and $6.w = 0)
+C  u8  vfollow                               -> $4b
+D  u8  flag70 (use +4 as push threshold)     -> $70
+E  u8  next   mode after a mode-4 lock       -> $7b
+F  u8  by_y   next trigger uses Y            -> $79
```
Loading also sets `$5c = camY` and `$6c += 1`. Stage 0 example: X=0 mode 0x11 (intro auto-scroll), X=0x380
mode 0, X=0x408 mode 0x0B (slope), X=0x4D0 mode 4 (fight lock), X=0x580 mode 1 (end of area 0), ...

---------------------------------------------------------------------------------------------------

## 6. Camera logic (scroll2 = main camera), faithful pseudocode

### 6.1 Init (state 0, 0x206EA) [O]
```rust
fn scroll2_init(s: &mut S2) {
    s.state = 2; s.t71 = 1; s.t75 = 1; s.phase72 = 0; s.prev73 = 0xFF;
    load_all_palettes();                  // 0x8A44
    brightness_full_current_mask();       // 0x8932
    if stage == 2 { video_ctrl |= 1 } else { video_ctrl &= !1 }   // rowscroll
    if (s.mode as i8) < 0x0E { s.area = 0; }
    s.go = 0; s.lock78 = 0; s.event_idx = 0;
    if s.mode != 0x0E && s.mode != 0x0F {
        load_event_index();               // 0x2099A: ptr = table + event_idx*16; load (6.3)
        s.next_event += 16;
        area_setup();                     // 0x20766 (below)
    } else { reinit_map(s); }
}
fn area_setup(s) {                        // 0x20766, also used by modes 1 and 0x12
    flag_4dc = 0; layer_ctrl = 0x12D6;
    if stage == 6 && s.area == 1 { flag_4dc = 1; $7b6 = 0; layer_ctrl = 0x06D6;
        fill_scroll1_bg(0, 0x100, 1); fill_scroll1_bg(0x100, 0x100, 2); }
    if stage == 6 && s.area == 2 { clear_scroll1(); call(0x162AE); }
    reinit_map(s);
}
fn reinit_map(s) {                        // 0x207CA
    let hdr = l(0x21580 + stage*4) + area*10;  $844 = hdr;
    s.w = b(hdr); s.h = b(hdr+1); s.screens = l(hdr+2); s.sx46 = w(hdr+6); s.sy48 = w(hdr+8);
    s.cam_x = s.sx46; s.cam_y = s.sy48;
    clear_scroll2_vram(); clear_scroll3_vram();
    let (mut x, y) = (s.sx46, s.sy48);
    let ncols = if x != 0 { x -= 0x20; 19 } else {
        // pre-draw the 8x8-block screen at (0, y) into VRAM at vram2(0,y) + 0xBC8 (purpose unknown) [O]
        17 };
    for _ in 0..ncols { draw_column2(x, y.wrapping_add(0x140), 0xD); x += 0x20; }   // 14 blocks per column
}
```
`flag_4dc` (`$4dc`) = "vertical scene": sprites use camY = 0, spawns trigger on camY (0x13FD8, 0xFC7E).

### 6.2 Run (state 2, 0x20898) [O]
```rust
fn scroll2_run(s: &mut S2) {
    tile_anim_tick(s);                                    // stages 4 and 7 only (2.6)
    if ![0x0E, 0x0F, 0x01, 0x12, 0x09].contains(&s.mode) {
        let ev = s.next_event;
        let by_y = s.by_y79 != 0 || (stage == 6 && s.area == 1);
        let hit = if by_y { !(w(ev+2) < s.cam_y) }        // unsigned: trigger when ev.y >= camY
                  else    { !(w(ev+0) > s.cam_x) };       // unsigned: trigger when ev.x <= camX
        if hit { load_event(s, ev); s.next_event += 16; } // at most one event per frame
    }
    player_extents(s);                                    // 0x20A8C
    MODE_TABLE_0x2092E[s.mode](s);                        // every mode ends with stream2() except 1/4/5-sub/6/8/0x10/0x12/0x13 (see 6.6)
}
```

### 6.3 Player extents (0x20A8C) [O]
```rust
fn player_extents(s: &mut S2) {
    let (mut max_x, mut min_x, mut max_y, mut min_y) = (0u16, 0xFFFFu16, 0u16, 0xFFFFu16);
    for p in [P1, P2, P3] {                       // $3274, $33f4, $3574
        if p.b[0] == 0 || p.b[4] == 0 { continue; }
        let yt = p.w[0x54].wrapping_add(p.w[0x10]);
        if yt > max_y { max_y = yt } if yt < min_y { min_y = yt }       // unsigned
        if p.x > max_x { max_x = p.x } if p.x < min_x { min_x = p.x }   // unsigned (p.x = +8)
    }
    if (min_x as i16) < (s.cam_x as i16) { min_x = s.cam_x.wrapping_add(0x10); }  // off-screen-left player does not hold the camera
    if P1.b[0] | P2.b[0] | P3.b[0] != 0 {
        s.max_x60 = max_x; s.max_y62 = max_y; s.min_y66 = min_y;
        s.min_x64 = if (min_x as i16) < 0 { 0x10 } else { min_x };
    }
}
```

### 6.4 Horizontal follow, push scrolling (0x20A22) [O]
```rust
const PUSH_BY_PLAYERS: [u16; 3] = [0xC0, 0xE8, 0xE8];   // 0x20B62, index = $4ef = active players - 1 (0x6596)
fn follow_x(s: &mut S2) {
    if s.flag70 == 0 { s.push50 = PUSH_BY_PLAYERS[$4ef]; }      // 0x20B4C
    s.prev_x14 = s.cam_x;
    let lead = s.max_x60.wrapping_sub(s.cam_x);                  // screen x of the rightmost player
    if !((lead as i16) > 0x40) { return; }
    if lead < s.push50 { return; }                               // unsigned
    let spread = s.max_x60.wrapping_sub(s.min_x64);
    if spread >= s.push50.wrapping_sub(0x10) {                   // unsigned: players too far apart
        let mut d = (s.min_x64.wrapping_sub(0x10).wrapping_sub(s.cam_x)) as i16;   // keep leftmost >= 16 px from the left edge
        if d > 6 { d = 6 } if d < 0 { d = 0 }
        s.cam_x = s.cam_x.wrapping_add(d as u16);
    } else {
        let mut d = lead.wrapping_sub(s.push50) as i16;
        if d > 6 { d = 6 }
        s.cam_x = s.cam_x.wrapping_add(d as u16);
    }
}
```
So: the camera never scrolls left in normal play; max speed 6 px/frame; no smoothing beyond that clamp;
stop points come from events (mode 4 snaps camX to the event X).

### 6.5 Vertical follow (0x20B68, only when `$4b` != 0) [O]
```rust
fn follow_y(s: &mut S2) {
    if s.vfollow4b == 0 { return; }
    s.prev_y16 = s.cam_y;
    let low = s.min_y66.wrapping_sub(s.cam_y);           // lowest player above the camera base
    let high = s.max_y62.wrapping_sub(s.cam_y);
    let step: i16;
    if low > s.down54 {                                  // unsigned
        if (high as i16) < (s.up52 as i16) { return; }
        if !((low as i16) > 0x10) { return; }
        let d = high.wrapping_sub(s.up52) as i16;        // >= 0
        step = if d.unsigned_abs() > 6 { -6 } else { d }; // QUIRK: a large upward overshoot moves the camera DOWN by 6 - keep
    } else {
        if !((high as i16) < 0xA0) { return; }
        let d = low.wrapping_sub(s.down54) as i16;       // <= 0
        step = if d.unsigned_abs() > 6 { -6 } else { d };
    }
    s.cam_y = s.cam_y.wrapping_add(step as u16);
    if (s.cam_y as i16) < 0 { s.cam_y = 0; }
}
```

### 6.6 Camera modes (table 0x2092E) [O]
`stream2()` = section 8.1. `fade_out_task(delay)` = set `$44c = delay`, `$44a = 0x0F`, create task 0x8840
in slot 0x40 (and similar for fade-in 0x8824); the task resets the delay to 1 (2.4), so the effective
fade is 16 frames. `go` = `$6d`.

| mode | routine | behaviour |
|---|---|---|
| 0x00 | 0x20A18 | `follow_x; follow_y; stream2` (normal play) |
| 0x01 | 0x20BF4 | **end of area with fade**: sub0: lock78=1, go=0, if flag_4dc sound 0x87; sub2 (every frame): follow_y, camX=$4c; when go: call 0xA5EA, fade_out_task(2). sub4: wait `$44a==0`; lock78=0; skip events until one with X==0 (event_idx++ per skip), area = that event's area; 0xACD0; load_all_palettes; `$7e0`=0; area_setup(); 0x12BE4. sub6: 0x162AE; fade_in_task(2). sub8: when go==0: load_event_index(), next+=16, sub=0. (no stream2) |
| 0x02 | 0x20F92 | diagonal descent to the right (see below) |
| 0x03 | 0x2100C | diagonal ascent to the right (see below) |
| 0x04 | 0x21086 | **lock** (fight): first frame lock78=1, go=0 (if flag_4dc sound 0x87). Each frame: if by_y `camY=$4e` else `camX=$4c`; when go: mode=$7b, sub=0, lock78=0; go=0. (no stream2) |
| 0x05 | 0x210E0 | stage 6 elevator ride (see below) |
| 0x06 | 0x20DA6 | **stage end**: first frame lock78=2, go=0; each frame camX=$4c; when go: `$4d4`=1 (stage clear flag, read by the game state machine). (no stream2) |
| 0x07 | 0x211AE | descend with the players (0.2 start): `m` = min over players with byte+0 != 0 of `w[+0xC]` (unsigned, start 0xFFFF); `t = m + 0x100 - camY` (u16); if `!(t > $54)` (unsigned, i.e. the `sub.w $54` borrowed or gave 0): `camY += t - $54`; then `if camY <= 0x100 (unsigned) { camY = 0x100 }`; stream2 |
| 0x08 | 0x20E78 | **sub-area switch without reload**: first frame go=0, lock78=1, camX=$4c; when go: lock78=0, area += 1, mode=0, OBJ palettes (0x8E4C), `$44a`=1 + 0x8932, 0xAAE2, 0x1031A, 0xACE0, 0x9236C |
| 0x09 | 0x2120E | slow fade transition: while sub<4 and a fade runs: camY -= 4 (min 0). sub0: camX=$4c, fade_out_task(12); then same sub-steps as mode 1 (0x20C6A, 0x20CC0, 0x20CE6); stream2 |
| 0x0A | 0x20EF8 | follow_x; `camY = ((camX - $5a) >> 2 (logical)) + $5c`; stream2 |
| 0x0B | 0x20F12 | follow_x; `sh = b(0x20F58 + stage*4 + area)`; `camY = $5c - ((u16)(camX - $5a) >> sh)`; `if camY < $4e (signed) camY = $4e`; stream2 (slope down) |
| 0x0C | 0x20F78 | `if max_x60 >= 0x750 (signed) { camY -= 1; if camY < 0 { camY = 0 } }`; stream2 |
| 0x0D | 0x20ED4 | follow_x; `y = ((camX - $5a) >> 1 (logical)) + $5c; if y > $4e (signed) y = $4e; camY = y`; stream2 (slope up) |
| 0x0E, 0x0F | 0x2126E | debug: stream only (0x0E also draws the attribute view, 1.1) |
| 0x10 | 0x20DEA | first frame go=0; camX=$4c; when go: `$4d4`=1, mode=0 |
| 0x11 | 0x21266 | auto-scroll: `camX += 3`; stream2 (stage 0 intro) |
| 0x12 | 0x20CFE | area change **without** fade: first frame lock78=1, go=0; each frame follow_y, camX=$4c; when go -> mode-1 sub4 logic without the fade wait (0x20D3A), then when go==0 load next event, sub=0, 0x892C (full brightness) |
| 0x13 | 0x20E20 | stage end with fade: lock78=2, go=0, camX=$4c; when go: `$4d4`=1, fade_out_task(2) with mask 0x0D (scroll1 HUD stays) |

0x20F58 shift table (stage*4+area): 1 for 0.0, 2 for 5.2 and 7.2, else 0.

Mode 2 (0x20F92), faithful:
```rust
follow_x(s); follow_y(s);
let lim = s.max_y62.wrapping_sub(0xA0);
if !((lim as i16) < (s.cam_y as i16)) { s.cam_y = lim; s.cam_x = s.prev_x14; }
let mut d = s.cam_x.wrapping_sub(s.x5a) as i16;
let d1 = s.y5c.wrapping_sub(s.cam_y) as i16;
if d < d1 { d = d1; }
s.cam_x = s.x5a.wrapping_add(d as u16);
let mut y = s.y5c.wrapping_sub(d as u16);
if !((lim as i16) < (y as i16)) { y = lim; s.cam_x = s.y5c.wrapping_sub(y).wrapping_add(s.x5a); }
if (y as i16) < (s.y4e as i16) { y = s.y4e; s.mode = 0; }
s.cam_y = y; stream2();
```
Mode 3 (0x2100C): same with `lim = min_y66 - 0x10`, `d1 = camY - $5c`, `y = $5c + d`,
`if lim < y (signed) { y = lim; camX = y - $5c + $5a }`, and `if y >= $4e (signed) { y = $4e; mode = 0 }`;
the first clamp is `if !(lim > camY) { camY = lim; camX = prev_x14 }`.

Mode 5 (0x210E0, stage 6 shaft; positions advance by `vel * 256` per frame on the 16.16 camera, i.e. vel
is 1/256 px per frame, routine 0x189C):
```
sub0: sound 0x86; velX=0 accX=0; velY=-0x30; accY=-6; velZ=accZ=0; $755c=2 (screen shake strength 2); -> sub2 same frame
sub2: if velY > -0x600 (signed): integrate (0x189C: vel += acc; pos16.16 += vel*256) ; else -> sub4 same frame
sub4: camY -= 6; if camY <= next_event.y + 0x300 (signed): accY = -accY; -> sub6
sub6: if velY >= -0x30: accY = 0;
      if camY <= 0xFFFC (unsigned): integrate; else { camY = 0; mode = 1; sub = 0; sound 0x87; return (no stream) }
all other paths end with stream2
```
Events of stage 6 area 1 (Y = 0x3000, 0x1E00, 0x1400, 0x800, mode 4, next = 5, by_y) stop the ride at each
floor; after the fight mode 4 returns to mode 5.

### 6.7 Screen shake (0x222D6, struct `$755a`) [O]
Requested by other code writing a strength to `$755c`. It does not change the camera; it writes the
**vblank shadow** directly: `$868 = camY + offset` (scroll2 Y register only, sprites not moved), or `$860 =
cam1Y + offset` in stage 6 area 1. Offset starts at +/-strength, flips sign every 3 frames, and strength
(and remaining count) drops by 1 per full +/- cycle; when the count reaches 0 it stops (`$7b8 = $860`).

### 6.8 Object clamp to the camera (0x193E, used by some objects) [O]
```
d = obj.x - camX + 0x50;  if d < 0 { obj.x -= d } else { d -= 0x220; if d >= 0 { obj.x -= d } }   // x in [camX-0x50, camX+0x1D0]
d = obj.$10 + obj.$56 + obj.$54 - camY; if d < 0 { obj.$10 -= d } else { d -= 0x100; if d >= 0 { obj.$10 -= d } }
(flag_4dc scenes: x in [0x10, 0x170], $10 in [0, 0x140])
```
Player screen-edge limits are elsewhere [U].

---------------------------------------------------------------------------------------------------

## 7. Parallax and rowscroll

### 7.1 scroll3 controller (a6 = `$7dc`, 0x22CAA) [O]
Init (state 0, 0x22CC0): header from 0x2311E (4.1) -> `$40..$48`, `$5a=$46`, `$5c=$48`,
`shift5f = b(0x22DE4 + w(0x22DE4 + stage*2) + area)`; `cam3 = (initX, initY)`; initial fill 17 or 19
columns x 14 tiles from `y + 0x140` (same shape as scroll2).
Run (state 2, 0x22D70):
```rust
fn scroll3_run(t: &mut S3, s2: &S2) {
    if (t.cam_x as i16) < 0 { t.cam_x = 0 } if (t.cam_y as i16) < 0 { t.cam_y = 0 }
    match s2.mode {
        0x05 => { if !((s2.cam_y as i16) > 0x2000) {                         // 0x22E8C
                      let y = (s2.cam_y >> 4) as i16 - 0x60; t.cam_y = if y < 0 { 0 } else { y as u16 }; } }
        0x09 => { if !((s2.sub6 as i8) > 2) {                                  // 0x22EAE
                      t.cam_y = 0x200u16.wrapping_sub(s2.y5c.wrapping_sub(s2.cam_y) >> 1); } else { t.b5 = 0; } }
        0x0E | 0x0F => {}
        _ => {                                                                // 0x22E0A
            if s2.by_y79 == 0 {
                t.cam_x = s2.cam_x >> t.shift5f;                              // logical shift
                if b(0x22E54 + stage*4 + area) != 0 {                         // areas 0.2, 6.1, 7.1
                    let d = s2.cam_y.wrapping_sub(w(l($844) + 8)) as i16;     // camY - scroll2 area initY
                    t.cam_y = ((d >> t.shift5f) as u16).wrapping_add(t.y5c);  // arithmetic shift
                }
            }
        }
    }
    stream3(t);                                                               // 0x22ED2, section 8.2
}
```
So scroll3 speed = 1/2^shift of scroll2 (shift per area in 4.6: mostly 1 = half speed; 3 = 1/8 in stage 0
and the desert; 5 = 1/32 in 1.2; 0 = locked with the playfield where no far layer exists).

### 7.2 scroll1
Scroll1 is the HUD/text layer (camera normally X=0, Y=0x100 -> registers 0xFFC0 / 0x0000). Only stage 6
area 1 uses it as a background (4.5). The attract/title code moves `$6c4` by 0xC per frame [O 0xC5E2].

### 7.3 Stage 2 rowscroll (0x2335A, struct `$7780`) [O]
```rust
fn rowscroll_task(r: &mut RS) {
    if r.state == 0 { fill(0x915800..0x918000, 0u16); r.base8 = 0; r.state = 2; }
    // state 2 (0x23390), every frame:
    let d0_start: i32 = (cam_x.wrapping_sub(r.base8) as i16) as i32;
    let d3: u16 = (d0_start as u16).wrapping_neg();
    let d1: i32 = d0_start >> 1;
    let mut d0 = d0_start;
    for k in 1..=32 {                                   // writes other[0xAF] down to other[0x90]
        d0 -= d1;
        let lo = (d0 as u16).wrapping_add(d3);          // add.w: only the low word changes
        let d2 = ((((d0 as u32) & 0xFFFF_0000) | lo as u32) as i32) >> 5;
        let mut v = d2 as u16;
        if v >= 0x180 { v &= 0xFF; }
        other[0xB0 - k] = v;                            // address 0x915960 - 2k
    }
}
```
Meaning [I]: visible lines 128-159 get offsets from about `-camX/2` (line 128) to `-camX/64` (line 159),
a perspective band for the road (values are taken mod 256 when large, which is invisible because the road
screens repeat every 256 px).

---------------------------------------------------------------------------------------------------

## 8. Tilemap streaming (keeps a 1024x1024 / 2048x2048 VRAM window filled)

For a native port there are two valid options:
1. **Faithful**: keep a 64x64 cell buffer per layer, implement 8.1/8.2 exactly, and let objects patch it
   (tile animation, holes with code 0). Recommended because objects write VRAM directly (2.6, 0x764B4,
   0x7522C...).
2. Direct: render visible cells straight from the map (4.2-4.4) each frame, plus an overlay for object
   patches. Gives the same picture while the stream keeps up (it always covers the 384x224 view).

Common helpers (scroll2):
```rust
fn vram2(x: u16, y: u16) -> usize {                     // 0x214C0: byte offset of the block's top-left cell
    let col_off = ((x & 0x3E0) << 2) as usize;          // (x/32)*0x80
    let d2 = 0x3E0 - (y & 0x3E0);
    col_off + (((d2 & 0x300) << 4) + ((d2 & 0xE0) >> 2)) as usize
}
fn block_ptr(id: u16, x: u16, y: u16) -> u32 {          // 0x2152C
    l(0x2155E + stage*4) + id as u32 * 128 + ((x & 0xE0) >> 1) as u32 + (0xE - ((y & 0xE0) >> 4)) as u32
}
fn write_block2(off: usize, blk: u16) {                 // 0x2142A: entries at off, off+4, off+0x40, off+0x44
    let src = l(0x21454 + stage*4) + blk as u32 * 16;
    vram[off..off+8] = rom[src..src+8]; vram[off+0x40..off+0x48] = rom[src+8..src+16];
}
```
### 8.1 scroll2 stream (0x2126E) [O]
```rust
fn draw_column2(x: u16, mut y: u16, n: u16) {           // 0x213F8: up to n+1 blocks downward from y
    // n = d4 of the dbra loop: callers pass 0xD (14 blocks, init) or 0x11 (18 blocks, streaming)
    let mut off = vram2(x, y); let mut id = screen_id(x, y); let mut p = block_ptr(id, x, y);
    let mut cnt = n;
    loop {
        write_block2(off, w(p)); p += 2; off += 8;
        let old_y = y;
        if y == 0 { return; }                            // stops after the y=0 block
        y = y.wrapping_sub(0x20);
        if old_y & 0xE0 == 0 { off = vram2(x, y); id = screen_id(x, y); p = block_ptr(id, x, y); }
        if cnt == 0 { return; } cnt -= 1;
    }
}
fn draw_row2(mut x: u16, y: u16, n: u16) {              // 0x213B8: n+1 blocks to the right from x
    let mut off = vram2(x, y); let mut id = screen_id(x, y); let mut p = block_ptr(id, x, y);
    let mut cnt = n;
    loop {
        write_block2(off, w(p)); p += 0x10;
        x = x.wrapping_add(0x20);
        off = vram2(x, y);                               // recomputed every block (wraps at 1024 px)
        if x & 0xE0 == 0 { id = screen_id(x, y); p = block_ptr(id, x, y); }
        if cnt == 0 { return; } cnt -= 1;
    }
}
// ge32(d): the original does `cmpi.w #$20,d0; bmi skip`, i.e. true when (d - 0x20) as i16 >= 0
fn ge32(d: u16) -> bool { (d.wrapping_sub(0x20) as i16) >= 0 }
fn abs16(d: u16) -> u16 { if d & 0x8000 != 0 { d.wrapping_neg() } else { d } }   // 0x212EE
fn stream2(s: &mut S2) {
    if s.sx46 < s.cam_x {                                // unsigned (cmp.w d0,d1; bcc)
        if ge32(s.cam_x.wrapping_sub(s.sx46)) { draw_column2(s.sx46.wrapping_add(0x200), s.sy48.wrapping_add(0x180), 0x11); }
    } else if ge32(s.sx46.wrapping_sub(s.cam_x)) && s.sx46 != 0 {
        draw_column2(s.sx46.wrapping_sub(0x40), s.sy48.wrapping_add(0x180), 0x11);
    }
    let x0 = { let v = s.sx46.wrapping_sub(0x120); if (v as i16) < 0 { 0 } else { v } };
    if s.sy48 < s.cam_y {                                // unsigned
        if ge32(s.cam_y.wrapping_sub(s.sy48)) { draw_row2(x0, s.sy48.wrapping_add(0x160), 0x19); }
    } else if ge32(s.sy48.wrapping_sub(s.cam_y)) {
        let y = s.sy48.wrapping_sub(0x80); if (y as i16) >= 0 { draw_row2(x0, y, 0x19); }
    }
    if ge32(abs16(s.cam_x.wrapping_sub(s.sx46))) { s.sx46 = round32(s.cam_x); }   // uses the new camX
    if ge32(abs16(s.cam_y.wrapping_sub(s.sy48))) { s.sy48 = round32(s.cam_y); }
}
fn round32(v: u16) -> u16 { (if v & 0x10 != 0 { v.wrapping_add(0x10) } else { v }) & 0xFFE0 }
```
Column/row positions are computed from the **old** `$46/$48`, before they are updated.

### 8.2 scroll3 stream (0x22ED2) [O]
Identical structure with these differences:
- cell = one 4-byte entry; VRAM offset `vram3(x,y) = (x & 0x7E0) + (((0x7E0-(y&0x7E0)) & 0x700) << 3) + (((0x7E0-(y&0x7E0)) & 0xE0) >> 3)`.
- entry pointer `0x177288 + id*256 + (x & 0xE0) + (0x1C - ((y & 0xE0) >> 3))`, screen id `& 0x7F`, stage-2 wrap 0xA00/0x100.
- column: copy 1 entry, `p += 4`, `off += 4`; row: copy 1 entry, `p += 0x20`, `off += 0x20`, VRAM recompute when `(x & 0x7E0) != 0`.
- right/left column and up row are skipped if their start Y (`$48+0x180`, `$48+0x160`) is negative (signed).

---------------------------------------------------------------------------------------------------

## 9. Importer checklist (Rust)
1. Palettes: banks per (stage, area) from 0x8AE0 / 0x8C52 (0x400 bytes), OBJ via 0x8E8E lists into
   0xB7A52, scroll1 0xBBA52; OR 0xF000 for display; convert with 1.1 formula.
2. Maps: headers 0x21580 / 0x2311E, screen arrays (column-major, bottom row first), scroll2 screens
   (0x2155E) -> blocks (0x21454), scroll3 screens 0x177288, scroll1 BG screens 0x138C48 -> metatiles 0x15CFE8.
3. Graphics: section 3 formulas and code ranges (code 0 = transparent).
4. Runtime: sections 5-8 (camera modes, events 0x21B2A, parallax 0x22DE4/0x22E54, rowscroll, fades).

## 10. Open gaps
- [U] Exact meaning of scroll2 attr bits 10-15 (terrain/collision class?) and whether gameplay reads them.
- [U] Triggers and full list of palette-cycling objects (only 0x89746 decoded in detail).
- [U] Who sets the scroll1 camera for the stage 6 elevator; full HUD scroll1 usage (other topic).
- [U] The purpose of the scroll2 pre-draw at `vram2(0,y)+0xBC8` when an area starts at X = 0.
- [U] Callers that set the go flag `$7a9` (0x1DE62, 0x1E4B4, 0x69FA4) belong to the stage-flow/enemy-wave
  topic; the camera only waits on it.
- [I] Bank mapper range ends (sprite range) and priority-mask semantics come from MAME, not from code.
- [I] The 1-line offsets of the screen mapping (camY + 0xEF - s) rely on MAME's visible-area constants.
