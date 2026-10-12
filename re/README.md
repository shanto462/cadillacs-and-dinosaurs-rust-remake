# Reverse-engineering workspace: Cadillacs and Dinosaurs (CPS1, 1993)

Everything here is derived from `../dino.zip` (MAME set "dino", World 930201). Do not publish it.

| Path | What it is |
| --- | --- |
| `rom/` | The 23 files from `dino.zip`, extracted with REA (`extract_artifact`). |
| `out/dino_68k.bin` | 68000 program image (1.5 MB, big-endian, loaded at address 0). |
| `out/sweep.txt` | Linear-sweep disassembly of the whole image (Capstone). Data areas show up as noise. |
| `out/gfx.npy`, `out/sheet16_*.png` | Interleaved 4 MB graphics ROM and decoded 16x16 tile sheets (grey). |
| `ghidra/` | Ghidra 12.1.4 headless project (`proj/`), import ELF, scripts, and **`decomp.c`**: decompiled C for about 3,560 functions (`decomp.c.index.tsv` lists them). RAM is writable/uninitialized so RAM reads are not folded; `decomp_v1.c` is the first run without that fix. |
| `tools/` | Python helpers. `tools/d <hexaddr> <count>` disassembles a range. |
| `tools/dump_assets.sh`, `tools/dump/` | Asset dump: reads `../dino.zip`, renders every tile, palette, sprite composition, animation (GIF and strip with hit boxes), stage map, text string and sound table into `out/assets/` and builds the local gallery `out/assets/index.html`. No emulator. Catalog in words: `../plan/game/design/asset-catalog.md`. |
| `out/assets/` | Output of the asset dump (git-ignored, ROM derived). Open `index.html`. |
| `specs/CONTEXT.md` | Shared facts: memory map, task kernel, object pools, sprite format, frame order. |
| `specs/anim/` | Animation, sprite composition and hitbox formats. |
| `specs/stage/` | Palettes, background layers, stage maps, camera. |
| `specs/player/` | Player state machine and combat rules. |
| `specs/enemy/` | Enemy AI, stage event scripts, items. |

Python: `.venv/bin/python -I` (capstone, numpy, pillow).

How to search the decompiled C: `grep -n '^// ==== 00013fa6' ghidra/decomp.c` (function headers are
`// ==== <address> <name> size=<bytes>`). The game passes arguments in registers, so calls show no
parameters and callees read `in_A0`, `unaff_A6` and so on; check `tools/d` when argument flow matters.

Re-run Ghidra:

```sh
export JAVA_HOME=/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home
/opt/homebrew/opt/ghidra/libexec/support/analyzeHeadless ghidra/proj dino -import ghidra/dino_68k.elf -overwrite \
  -processor 68000:BE:32:default -scriptPath ghidra/scripts \
  -preScript DinoSetup.java ghidra/entries.txt \
  -postScript DinoExport.java ghidra/entries.txt ghidra/decomp.c
```
