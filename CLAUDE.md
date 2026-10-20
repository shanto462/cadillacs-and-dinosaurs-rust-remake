# roms: Cadillacs and Dinosaurs reverse engineering + Rust port

- `dino.zip` is the user's ROM (MAME set "dino"). Never commit it, program/graphics images, the Ghidra
  database, decompiled C or raw disassembly dumps, and never paste ROM bytes or tables into source
  files (code reads the zip at run time through `dino-extract`). `.gitignore` covers these.
- Exception (user decision 2026-10-12, the repo is always private): the rendered asset dump
  `re/out/assets/` (made by `re/tools/dump_assets.sh`, browse `re/out/assets/index.html`) IS committed,
  so nobody has to extract it again. Re-run the dump and commit when the dump tool changes.
- `re/`: reverse-engineering workspace. Start with `re/README.md` and `re/specs/CONTEXT.md`;
  topic specs are in `re/specs/{anim,stage,player,enemy}/SPEC.md`. Decompiled C: `re/ghidra/decomp.c`.
- `dino-rs/`: the Rust port (native logic, no emulation). Read `dino-rs/ARCHITECTURE.md` before
  changing it. Build and run from `dino-rs/`: `cargo run -p dino-game -- --rom ../dino.zip`.
- Character ids: 0 Jack, 1 Mustapha, 2 Hannah, 3 Mess (verified by art; the name table order differs).
- REA cannot open 68000 code; use Ghidra headless (`re/README.md`) and `re/tools/d`.
- Sub-agents (user rule, 2026-10-17, widened 2026-10-21): **at most 3 Claude sub-agents at the same time: one Sonnet
  (heavy work), one Haiku (small, easy tasks) and one Opus, never two of the same model.** Mid-sized tasks go to the external CLIs (codex, grok,
  agy), each in its own git worktree under `.worktrees/`, merged one at a time after review. Only one worker
  writes game code in the main tree. Agents never start their own agents; never stop a running agent to get
  under the limit, let it finish.
