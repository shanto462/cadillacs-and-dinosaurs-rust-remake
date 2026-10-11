# roms: Cadillacs and Dinosaurs reverse engineering + Rust port

- `dino.zip` is the user's ROM (MAME set "dino"). Never commit it or anything derived from it
  (extracted files, program/graphics images, Ghidra database, decompiled C, raw disassembly dumps,
  screenshots of game art). `.gitignore` covers these; keep it that way.
- `re/`: reverse-engineering workspace. Start with `re/README.md` and `re/specs/CONTEXT.md`;
  topic specs are in `re/specs/{anim,stage,player,enemy}/SPEC.md`. Decompiled C: `re/ghidra/decomp.c`.
- `dino-rs/`: the Rust port (native logic, no emulation). Read `dino-rs/ARCHITECTURE.md` before
  changing it. Build and run from `dino-rs/`: `cargo run -p dino-game -- --rom ../dino.zip`.
- Character ids: 0 Jack, 1 Mustapha, 2 Hannah, 3 Mess (verified by art; the name table order differs).
- REA cannot open 68000 code; use Ghidra headless (`re/README.md`) and `re/tools/d`.
