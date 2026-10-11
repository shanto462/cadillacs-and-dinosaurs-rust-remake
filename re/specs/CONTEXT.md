# Cadillacs and Dinosaurs (CPS1, MAME set "dino", World 930201) - shared RE context

Goal of the whole project: understand how the 1993 game was built, and write a Rust
reimplementation that loads assets from the user's own dino.zip at runtime (no Capcom data is
copied into the project). The Rust runtime is our own design (not a CPS1 emulator). We need
exact data formats (for the importer) and exact gameplay rules/numbers (for the runtime).

## Paths (S = scratchpad)
S=/Volumes/Shanto/Repos/Personal/Github/roms/re
- $S/rom/                 extracted ROM files (read-only, untrusted data)
- $S/out/dino_68k.bin     68000 program image, 0x180000 bytes, big-endian, loaded at address 0
                          (cde_23a.8f + cde_22a.7f + cde_21a.6f, each byte-swapped per 16-bit word)
- $S/out/sweep.txt        linear-sweep disassembly of the whole image (capstone; data areas are noise)
- $S/out/gfx.npy          interleaved 4 MB graphics ROM (numpy uint8), see tools/gfx.py
- $S/tools/d <hexaddr> <count>   disassemble count instructions from hexaddr (shell wrapper)
- $S/tools/gfx.py         load_gfx(), tile16(g, code) decoder (16x16, 128 bytes per tile)
- Python: $S/venv/bin/python -I  (has capstone 5, numpy, pillow). Always run with -I.
- Do NOT edit files in $S/tools or $S/out. Put your own scripts/outputs in your own $S/specs/<topic>/ folder.
- REA (the RE MCP server) cannot open 68000 code here (no Ghidra; m68k ELF unsupported), so use the
  capstone listing. Hopper/Ghidra are not usable for this CPU in this session.

## Hardware memory map (CPS1, observed in code)
- 0x000000-0x17FFFF program ROM. 0xFF0000-0xFFFFFF work RAM.
- A5 = 0xFF8000 almost everywhere (`lea $8000.w,a5`), so `$xxxx(a5)` = 0xFF8000+xxxx, `-$xxxx(a5)` = 0xFF8000-xxxx.
- 0x900000 / 0x904000: sprite (OBJ) list, double-buffered; flag $69b4(a5) selects the back buffer.
  Entry = 4 words: X, Y, CODE, ATTR. ATTR bits 0-4 palette, bit5 X-flip, bit6 Y-flip,
  bits 8-11 block width-1, bits 12-15 block height-1 (tile grid of consecutive codes). ATTR=0xFF00 ends the list.
- 0x908000 scroll1 (8x8 text/HUD) tilemap, 0x90C000 scroll2 (16x16 playfield), 0x910000 scroll3 (32x32 far bg).
  Scroll1 tilemap is column-major: +4 bytes = next row, +0x80 = next column. 0x90A000.. is used for debug text.
- 0x914000 palettes: +0x000 OBJ, +0x400 scroll1, +0x800 scroll2, +0xC00 scroll3. 32 palettes x 16 colours x 16-bit.
  Colour word 0xBRGB: brightness nibble B, then R,G,B nibbles. MAME formula:
  bright = 0x0f + ((w>>12)<<1); c = nibble*0x11*bright/0x2d. Pen 15 is transparent for sprites/scroll layers.
- CPS-A regs 0x800100-0x800122: +0 OBJ base, +2/+4/+6 scroll1/2/3 base, +0xA palette base,
  +0xC/+0xE scroll1 X/Y, +0x10/+0x12 scroll2 X/Y, +0x14/+0x16 scroll3 X/Y, +0x20 rowscroll offset, +0x22 video ctrl.
  CPS-B 0x80014A layer control = 0x12D6 at boot (draw order back->front: scroll3, scroll2, sprites, scroll1).
- Inputs: 0x800000/0x800001 players 1/2, 0xF1C001 player 3, 0x800018 coins/start, 0x80001A/C/E DIP switches.
  Cooked inputs in RAM: $16(a5) P1, $17(a5) P2, $18(a5) P3 (active-high after NOT); previous frame at $1b/$1c/$1d.
- 0xF18000-0xF19FFF QSound shared RAM (Z80 sound CPU). 68k sends sound commands through a 64-entry ring at $3a(a5),
  flushed in the vblank IRQ (0x610).
- Screen is 384x224. Sprite X has a +0x40 offset (visible X range 0x40..0x1BF).

## Program structure found so far
- Reset 0x41A: clears RAM, programs CPS-A/B, boot task 0xA69A (creates tasks, inits players, jumps to 0xA6CE8).
- VBlank IRQ level 2 at 0x542: flips OBJ buffer, writes scroll regs from shadows, flushes sound queue,
  reads inputs (0x738), coin logic (0x9D4BE), ticks task sleep timers, sets the frame flag.
- Cooperative task kernel: 16 task slots at -$7fae(a5) (0xFF0052), 16 bytes each:
  +0 state (0 free, 1 sleeping, 2 suspended, 4 ready, 8 running, 0xC new), +1 sleep counter, +2 SR, +4 PC, +8 USP, +C stack.
  Main loop 0x83C runs each ready task once per frame. Kernel calls via TRAP:
  #0 create task (d0 = slot*16, a0 = entry) [wrapper 0x8CE], #1 exit, #2 kill d0 [0x92E], #3 sleep d0 frames [0x95A],
  #4 yield [0x9B6], #5 suspend d0 [0x9DE], #6 wake d0 with counter d1 [0x9FA], #7 restart self at a0.
  Sleep helpers: 0xA76 (1 frame), 0xA7A (2), 0xA7E (30), 0xA82 (60), 0xA86 (120), 0xA8C (180).
- RNG 0xA9A (LFSR in $4(a5)).
- Object pools (free-list stacks of 16-bit RAM pointers):
  players: 3 fixed objects of 0x180 bytes at $3274(a5), $33f4(a5), $3574(a5) (init 0x4D46/0x4D5E/0x4D76;
           +$02 player index, +$5e index*0x10, +$6a/$6c/$6e vitality = 100, +$24 = 1, +$10a = 0x33; 0x11738 called)
  pool A: 24 x 0xE0 at $48f4(a5), tag +$82 = 4, free list $6698(a5)/$6602, count $65f8  (likely enemies)
  pool B: 24 x 0xC0 at $36f4(a5), tag +$82 = 8, free list $6668(a5)/$6600, count $65f6  (likely items/props)
  pool C: 48 x 0x80 at $1a74(a5), free list $66f8(a5)/$6604, count $65fa                 (likely effects/shots)
  pool D: 16 x 0x80 at $5df4(a5), free list $6718(a5)/$6606, count $65fc
  (pool init routines at 0x4C48..0x4CEC)
- Sprite list builder 0x13FA8 (called once per frame): camera X = $744(a5) (screen X base = camX-0x40 in $69b6),
  camera Y = $748(a5) ($69b8). Draw list of longs below $69b0(a5), count $67c0(a5): (sort key << 16 | object ptr);
  bubble-sorted by key (depth) then each object drawn by 0x141F8.
- Per-object draw 0x14230: a3 = object +$28 (long ptr to current animation frame record).
  Frame record word0 = composition index (byte offset into table ptr stored at ROM 0x0FFFF4 = 0x0D0CF4), negative => 0x14180 path;
  word1 = optional second composition (overlay), negative = none.
  Composition (a1): word sprite_count (stats), word n_entries, word attr (bit15 set => alternate routine 0x14494),
  word layout index (x4 into table ptr at ROM 0x0FFFFA = 0x0D43D8; layout = list of (dx,dy) word pairs),
  then n_entries tile-code words. Each entry -> OBJ (X=objX-camX+dx, Y=-(obj+$c + obj+$10)+dy, code, attr).
  Object fields used: +$8 X, +$c Y-ish, +$10 added to Y (height or depth), +$23 palette override (bit7 set => low 5 bits replace palette),
  +$24 facing (nonzero => X-flip: x = -dx-16, toggles attr bit5), +$25 flags (bit0 => 0x14136 path with +$3c/+$3e offsets,
  bit6 => 0x147FE, bit7 => screen-space, no camera), +$53/+$4c partial-draw (wading?), +$28 anim frame ptr, +$30 another ptr.
- Palettes: fade code 0x892C..0x89E4 (sets/clears brightness nibbles). Stage scroll2 palette chosen by
  $4d9(a5) (stage) and $786(a5) (area) via pointer table at 0x8AE0 (0x400-byte blocks, ~0xB0000-0xBFFFF).
  Scroll1 palette copy at 0x8A94.
- Strings use '@' (0x40) as terminator; print routine 0xC0A. Interesting string tables:
  character names 0x17DC7 (players + all enemy names), weapons 0x23C65, foods 0x240F3, treasures 0x24457,
  enemy AI mode names 0x32FD4.. (TAIKI, KOUGEKI, ... DARUMA; offset table at 0x32FD4 indexed by enemy +$a0*2),
  another list at 0x3300A indexed by +$06, debug "BODY HIT / ATACK HIT" hitbox viewer near 0xAC09A,
  "CADI SPEED" 0x502C4, staff roll 0x8B7EA, test-mode menus 0xA6E00-0xAAC30.
- Graphics ROM: 4 MB, 0x8000 16x16 tiles of 128 bytes. Row = 8 bytes: bytes 0-3 = pixels 0-7, bytes 4-7 = pixels 8-15;
  within each 4 bytes, byte p is bitplane p (weight 1<<p), MSB = leftmost pixel. 8x8 text tiles are two per 16x16 cell
  (left/right halves). Verified visually (tile sheet shows a 0..15 grey ramp, the font, and character body parts).

## Added later
- Ghidra 12.1.4 (Homebrew) is installed. Headless project: $S/ghidra/proj (project "dino"), input $S/ghidra/dino_68k.elf
  (ROM + RW segments for IO/VRAM/QSound/RAM), A5 context = 0xFF8000, entries from $S/ghidra/entries.txt.
  Decompiled C for all functions: $S/ghidra/decomp.c (headers `// ==== <addr> <name> size=<n>`), index decomp.c.index.tsv.
  Labels: task_create 0x8CE, task_kill 0x92E, task_sleep 0x95A, rng_next 0xA9A, print_string 0xC0A,
  build_sprite_list 0x13FA6 (real entry; 0x13FA8 in the sweep is mid-instruction), draw_object 0x141F8, game_frame 0xAAF6.
- In-game per-frame order (game_frame 0xAAF6), corrected: 0x186CC rank/difficulty tick, 0xE90E object updates
  (census 0x3242E, players 0x18834, pool E items 0x233C6, pool A 0x23420, shake 0x222D6, pool B 0x23604, pool C 0x237B2,
  held players 0x188DA, ..., camera/background 0x2042E at the end), 0x10546 collision passes, 0x16BEA HUD,
  0x13FA6 sprite list, 0x105E8 list reset, 0x103FE/0x1037E time limit, 0xFBF4 spawn interpreter (tables 0xFC2E/0xFC4E).
- Pool E (items, food, weapons, containers) is at $874(a5), 24 x 0xC0. Pool B holds controllers and props.
- Top level: task slot 0 = system task 0x9D048 (coins/credits/start; on start kills tasks 0x10-0x40 and calls 0x983EE),
  attract/flow task 0xA6CE8 (mode byte state machine via jump tables), slot 0xF0 task 0x9E38C.
