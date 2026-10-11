# Decompilers for Motorola 68000 code: survey and plan

Date of research: 2026-10-12. Target: Capcom "Cadillacs and Dinosaurs" (CPS1, 1993), 1.5 MB 68000 program ROM
wrapped in an m68k ELF, analyzed with Ghidra 12.1.4 headless (`68000:BE:32:default`).

How to read the labels in this report:

- **Verified**: I read the official docs, source code, or repository.
- **Measured**: I counted or checked it in our own files (`ghidra/decomp.c`, `out/sweep.txt`, the Ghidra project).
- **Unverified**: I could not confirm it. Test it before you depend on it.

No software was installed. One read-only headless Ghidra run (`-readOnly`, nothing saved) was used to print the
memory map of our existing project.

---

## 1. Short answer

1. **Keep Ghidra as the primary tool.** It is still the best 68000 decompiler you can get in late 2026. Hex-Rays
   (IDA), JEB, RetDec, Snowman and Boomerang have no 68000 decompiler at all. Binary Ninja only has an unmaintained
   community lifter. radare2/rizin use the same Ghidra engine with less analysis around it.
2. **Use Reko as a second opinion for register signatures.** Reko finds register parameters and register
   outputs by itself (for example `fn0000127C(Register (ptr32 int8) d2, Register out ... a5Out)`), which is exactly
   where Ghidra is weak for this game. Its C is noisier, so use it to check, not to read.
3. **Our current Ghidra output has fixable problems that are bigger than the decompiler's own limits.** The most
   serious one is a setup bug, not a Ghidra bug (see section 2.1): work RAM and all I/O blocks were imported as
   read-only and zero-filled, so the decompiler silently treats some RAM reads as the constant 0 and deletes code.

---

## 2. What our current output shows (measured)

Numbers come from `re/ghidra/decomp.c` (3,557 functions) and `re/out/sweep.txt` (linear sweep, so counts that come
from the sweep include some noise from data areas).

| Symptom | Count | Meaning |
| --- | --- | --- |
| `unaff_A6` uses | 14,025 | Object pointer in A6 is read but is not a parameter |
| `in_A0` / `in_A1` / `in_D0w` uses | 3,423 / 1,205 / 1,363 | Register arguments not shown as parameters |
| `extraout_A0` uses | 6,385 | Callers use A0 returned by a callee |
| `in_ZF` / `in_CF` uses | 635 / 651 | Condition flags read before they are set; usually flags returned by a callee |
| `WARNING: Read-only address (ram,0x00ffxxxx) is written` | 2,406 | Decompiler thinks work RAM is read-only (see 2.1) |
| `WARNING: Removing unreachable block` | 9,368 | Code removed as dead; part of it is caused by 2.1 |
| Functions that contain `halt_baddata()` | 1,613 of 3,557 | Control flow runs into data, mostly from jump tables |
| `switch` statements recovered | 21 | Versus about 620 `move.w tbl(pc,dN.w),dN` + `jmp tbl(pc,dN.w)` pairs in the sweep |
| `WARNING: Could not recover jumptable ... Too many branches` | 22 | Switch variable has no known bound |
| A5-relative operands / `.w` absolute RAM operands / `.l` `$FFxxxx` operands | 16,150 / 400 / 82 | How RAM is addressed (sweep) |

### 2.1 Bug in our setup: RAM and I/O are read-only and zero-filled (high impact)

The ELF wrapper (`tools/build_elf2.py`) marks the RAM, I/O, GFX RAM and QSound segments as RW with no file
bytes. But the Ghidra project shows every one of them as `init=true w=false` (initialized with zeros and not
writable):

```
segment_1  00800000-008001ff init=true r=true w=false   (I/O, CPS-A, CPS-B)
segment_2  00900000-0092ffff init=true r=true w=false   (GFX RAM)
segment_3  00f18000-00f19fff init=true r=true w=false   (QSound shared RAM)
segment_5  00ff0000-00ffffff init=true r=true w=false   (work RAM)
```

The decompiler option "Respect read-only flags" is on by default. The Ghidra help says that with this option "the
initial value present in the Program is pulled into the data flow of the function as a constant." `DinoExport.java`
uses `new DecompileOptions()`, so the default applies.

Concrete damage, function `FUN_000075b2`:

```
007600: tst.w    $32(a5)        ; read RAM 0xFF8032
007604: bne.w    $7624
```

The decompiled C has no test of 0xFF8032 at all, and the header says
`WARNING: Removing unreachable block (ram,0x00007624)`. The read was folded to 0, so the branch was deleted.
Across the file, 6,489 of the 6,557 `UNK_00ffxxxx` names appear only as write targets, which suggests that reads
of those RAM addresses were folded away in the same way. **Until this is fixed, do not trust any decompiled
condition that reads RAM.**

Fix (in `DinoSetup.java`, before analysis): make RAM uninitialized and writable, make I/O and QSound shared RAM
writable and volatile, and also set `opts.setRespectReadOnly(false)` in the export script as a safety net (keep ROM
read-only so constant tables still fold). See section 6, step 1.

### 2.2 Two addresses for the same RAM

The game sets its base register with `lea $8000.w,a5` (11 places). On a 68000 the `.w` address is sign-extended,
so Ghidra computes A5 = **0xFFFF8000**, not 0xFF8000. `movea.w` pointer loads (about 1,490 in the sweep) also
produce 0xFFFFxxxx. Our setup forces A5 = 0xFF8000 through the program context. Result: `vblank_irq` shows
`DAT_ffff84aa` while other functions show `DAT_00ff84aa` for the same byte. Ghidra issue
[#766](https://github.com/NationalSecurityAgency/ghidra/issues/766) (open since 2019) describes this exact 24-bit
address problem; the suggested workaround is a RAM block at 0xFFFF0000.

---

## 3. Comparison table

| Tool | m68k support level | Decompiler output quality (for our code) | Cost | Maintenance status | Notes |
| --- | --- | --- | --- | --- | --- |
| **Ghidra 12.1.4** | Built in. `68000:BE:32:default` is really a 68040 SLEIGH spec; MC68020, MC68030, ColdFire and (since 12.1.3) CPU32 variants. No pure 68000 variant ([#230](https://github.com/NationalSecurityAgency/ghidra/issues/230)) | Best available. Readable C, types, structs, switch recovery. Weak on register arguments, unbounded jump tables, 24-bit addresses | Free (Apache 2.0) | Very active. 12.1.4 released 2026-09-21; 12.2 change history (Oct 2026) exists in master | Our primary tool. Verified |
| **Reko 0.12.5** | m68k rated "thoroughly tested and stable" in its wiki. Raw loader: `--loader raw --arch ... --base` | Medium. Finds register params and register outputs automatically. Output is noisy (`SEQ(SLICE(...))`, weak names, weak types) | Free (GPL-2.0) | Active. 0.12.5 on 2026-09-29 | Best second opinion for signatures. Needs .NET 9; GUI is Windows (cross-platform GUI "under construction"), CLI should work on macOS (unverified) |
| **IDA Pro 9.3** | Disassembler only (MC680xx, CPU32). Hex-Rays has no m68k decompiler; 9.3 added V850 | None for m68k | Pro Essential from EUR 1,099/yr. IDA Home 68K was retired in 9.0; Home now offers only PC, ARM, MIPS, PPC, RISC-V | Active | Good for disassembly and ROM hacking scripts. No pseudocode |
| **Binary Ninja + binaryninja-m68k** | Community LLIL lifter (v0.5). Not official | HLIL is built from LLIL automatically, but m68k HLIL quality is unverified. Last plugin commit 2022-06 | Personal $199 (non-commercial), Commercial $1,799. Free edition cannot run plugins | Plugin unmaintained | Not recommended |
| **radare2 + r2ghidra 6.2.4** | `m68k` maps to Sleigh `68000` (verified in `ArchMap.cpp`) | Ghidra engine, but without Ghidra's analyzers, data types and program database; the radare2 book warns output differs from Ghidra | Free | Active (2026-10-05) | Handy for quick CLI checks only |
| **rizin / Cutter 2.5.0 + rz-ghidra** | Same mapping as r2ghidra | Same as above | Free | Active (Cutter 2.5.0, 2026-06-30) | GUI option; no advantage over Ghidra |
| **r2dec** | m68k listed as "experimental" | Low (pseudo-C from disassembly) | Free | Active (6.2.0, 2026-08-16) | Quick looks only |
| **angr** | P-code lifting through pypcode; pypcode lists all 68000 languages; angr has `SimCCM68k` | Decompiler on m68k: unverified, likely experimental | Free | Active | Research use, symbolic execution |
| **RetDec** | No (x86, ARM, MIPS, PIC32, PowerPC) | n/a | Free | Limited maintenance; last release v5.0 (2022) | Not usable |
| **Snowman** | No (x86/ARM family, per project; not re-verified) | n/a | Free | Archived 2024-07-23 | Not usable |
| **Boomerang** | No (x86, PPC, ST20) | n/a | Free | Inactive since 2020 | Not usable |
| **JEB** | No (x86, ARM, MIPS, RISC-V, WASM, EVM, ...) | n/a | Commercial | Active | Not usable |
| **GhIDA / Yagi** (Ghidra decompiler inside IDA) | GhIDA only tested on x86/x64; Yagi lists 68000 as not supported | n/a | Free, needs IDA | GhIDA inactive since 2021; Yagi last push 2024 | Not useful |
| **LLM tools** (ReVa, GhidraMCP, bethington/ghidra-mcp, LLM4Decompile) | Work on top of Ghidra output, so any arch. LLM4Decompile models are trained on Linux x86-64 | Good for naming and typing; can change behavior silently | Tools free; model cost | Active | Use only with differential tests (section 4.7) |

---

## 4. Tool notes

### 4.1 Ghidra (built-in 68000 decompiler)

**Quality and status.** Ghidra's 68000 support keeps getting fixes: 12.1.3 (Aug 2026) fixed `movem.w` stepping by 4
instead of 2 and added a CPU32 variant; 12.0.1 fixed an operand issue; 11.2 fixed `ext` not setting flags, which
broke 68000 switch bounds ([#6679](https://github.com/NationalSecurityAgency/ghidra/issues/6679)). Our 12.1.4
includes all of these. 12.1 also added display of structure bitfields in expressions (useful for `btst`/`bset` on
flag bytes).

**Known open 68000 issues that matter to us (verified on GitHub):**

- [#766](https://github.com/NationalSecurityAgency/ghidra/issues/766): `.w` addresses become 0xFFFFxxxx (24-bit bus not modeled).
- [#2375](https://github.com/NationalSecurityAgency/ghidra/issues/2375): cspec uses 4-byte alignment, but m68k aligns to 2. Never use "packed" (auto-aligned) structures; lay out struct fields by explicit offset.
- [#1400](https://github.com/NationalSecurityAgency/ghidra/issues/1400): `bset` does not respect structure fields.
- [#8558](https://github.com/NationalSecurityAgency/ghidra/issues/8558): `movem` with `(d8,PC,Xn)` is wrong (rare).
- [#5031](https://github.com/NationalSecurityAgency/ghidra/issues/5031): return address in a register needs a custom prototype.
- [#6189](https://github.com/NationalSecurityAgency/ghidra/issues/6189): feature request for easier manual jump table hints (still open).

**Why we see `in_A0` and `unaff_A6`.** The only calling convention for `68000:BE:32:default` is `__stdcall`:
all inputs on the stack, output in D0 (or D0:D1, FP0), `killedbycall` D0, D1, A0, A1, and `unaffected` D2-D7,
A2-A6, SP. So any register read before it is written becomes `in_X` (if it is not in the unaffected list) or
`unaff_X` (if it is). The Ghidra advanced class says: "in_ or unaff_: this typically indicates that a register is
read before it is written (and it does not contain a parameter passed to the function)." Ghidra also ships
`68000_register.cspec` (args in A0, A1, FP0, FP1, D0, D1), but it is only offered for the ColdFire language and it
does not match Capcom's style anyway.

#### Fix A: register parameters

There are three tools. Use them in this order.

1. **Per-function custom storage, set by a script (most reliable).** "Use Custom Storage" in Edit Function Signature
   lets each parameter live in any register (including sub-registers like `D0w`). The decompiler then shows the
   argument at every call site. Automate it:
   - decompile each function, collect `HighSymbol`s whose storage is a register and whose high variable starts as
     an input (`in_*`, `unaff_*`),
   - build `ParameterImpl(name, dataType, register, program)` for each, and call
     `function.updateFunction(cc, function.getReturn(), params, FunctionUpdateType.CUSTOM_STORAGE, true, SourceType.USER_DEFINED)`,
   - go bottom-up (callees first) and repeat until nothing changes (2 or 3 passes), because a caller that only
     passes its own A0 through to a callee becomes a new `in_A0` function after the callee is fixed.
   - skip `in_ZF`, `in_CF` and other flags (see Fix B), and skip interrupt handlers.
   Ghidra's own `FixupGolangFuncParamStorageScript.java` is a working example of setting custom storage from a
   script. `FindPotentialDecompilerProblems.java` lists every function that still has `in_`/`unaff_` variables, so
   you can track progress.
2. **A register-strategy prototype (try it, then compare).** The cspec format has `strategy="register"` on
   `<prototype>`, "designed for software with a lot of hand-coded assembly routines". You list a `<pentry>` for
   every register that can carry input, and "the input varnodes for a function that have a corresponding
   `<pentry>` are automatically promoted to formal parameters", with no rules about gaps. Add it without editing
   the Ghidra install through **Edit > Options for 'dino' > Specification Extensions > Import** (or headless with
   `new SpecExtension(program).addReplaceCompilerSpecExtension(xml, monitor)`), make it the convention for all
   functions, then run **Analysis > One Shot > Decompiler Parameter ID** to commit the results. Risk (unverified):
   registers that are only saved and restored with `movem` might show up as false parameters; test on 20 functions
   before applying to all. Note the Ghidra class warning: Decompiler Parameter ID "will commit the signature of each
   function" and run too early it can spread bad information.
3. **Named prototypes for indirect calls.** Object state handlers are called through tables (`jsr (a0)`, about 128
   `jmp/jsr (aN)` in the sweep). A function pointer type cannot carry custom storage, but it can carry a calling
   convention. With the register strategy, "the first `<pentry>` that hasn't been used and that fits the datatype is
   assigned", so a prototype whose first pentry is A6 makes `void (*)(Object *)` mean "object in A6". Use
   **Override Signature** at single call sites. (Design idea from the cspec docs; unverified in practice.)

Example prototype to start from (untested; register names come from the 68000 SLEIGH spec):

```xml
<prototype name="capcom_asm" extrapop="4" stackshift="4" strategy="register">
  <input>
    <pentry minsize="1" maxsize="4"><register name="A6"/></pentry>
    <pentry minsize="1" maxsize="4"><register name="A0"/></pentry>
    <pentry minsize="1" maxsize="4"><register name="A1"/></pentry>
    <pentry minsize="1" maxsize="4"><register name="D0"/></pentry>
    <pentry minsize="1" maxsize="4"><register name="D1"/></pentry>
    <!-- add D2-D7, A2-A4 if the audit shows they carry inputs -->
  </input>
  <output>
    <pentry minsize="1" maxsize="4"><register name="D0"/></pentry>
  </output>
  <unaffected>
    <register name="SP"/>
    <register name="A5"/>
  </unaffected>
  <killedbycall>
    <register name="D0"/><register name="D1"/><register name="A0"/><register name="A1"/>
    <register name="ZF"/><register name="CF"/><register name="NF"/><register name="VF"/><register name="XF"/>
  </killedbycall>
</prototype>
```

#### Fix B: callee-saved registers and flags (correctness risk)

- The default `unaffected` list tells the decompiler that every callee keeps D2-D7 and A2-A6. Hand-written asm
  does not always do that. If a callee changes A6 or D7 on purpose and the caller uses it, **Ghidra prints the old
  value and the C is wrong**. The cspec docs say a location that is in neither list "is treated as if it may hold
  different values before and after the call", which is the safe choice. Plan: run Ghidra's
  `RegisterTouchesPerFunction.java` (writes the registers each function modifies into its plate comment), build the
  transitive closure over callees, and give every function that changes D2-D7/A2-A6 without restoring them a
  prototype with a small unaffected list (like `capcom_asm` above). Functions that do preserve can keep the default.
- Flags: `in_ZF`/`in_CF` (about 650 each) usually mean "the caller tests flags that the callee returned".
  Because flags are in neither list, the decompiler treats them as passing through the call unchanged and calls
  them inputs of the caller. Putting the flags in `killedbycall` turns them into `extraout_ZF` (clearly a callee
  result). For routines that return a boolean in a flag, try a custom-storage return value in `ZF` or `CF`
  (unverified that the decompiler handles a 1-byte flag register as return storage well).
- Routines that return several registers (for example A0 and D0): one return value can use compound storage
  (custom storage supports several varnodes), but this is clumsy. For the Rust port, model them as functions that
  return a tuple, and accept `extraout_` names in Ghidra.

#### Fix C: struct types on pointer registers (a6 = object)

- Once A6 is a real parameter (Fix A), set its type to `Object *` and the decompiler prints `obj->field`.
- Build the struct with the decompiler's **Auto Create Structure / Auto Fill in Structure** action (or the
  `CreateStructure.java` script), then rename fields. Keep struct packing off (issue #2375).
- When A6 is loaded inside a function (`lea POOL,a6` loops), retype the local variable instead.
- Fields that hold 16-bit pointers (`movea.w off(a6),a0`): type them as `short` and in Rust as `u16` RAM offsets.
  After `movea.w`, retype the A0 local as `Object *`; the C then shows a cast from `short` and normal field access.

#### Fix D: jump tables

Pattern: `move.w tbl(pc,d0.w),d0` then `jmp tbl(pc,d0.w)` (each entry is a signed 16-bit offset from `tbl`).
Ghidra's switch recovery needs a bound on the index (a `cmp`/`bhi` guard). Capcom code usually indexes with a
state byte and has no guard, so recovery fails or picks a wrong length; the extra "cases" then point into data and
produce `halt_baddata()`. Issue #6679 shows the same pattern recovered correctly when a guard exists.

Built-in Ghidra helpers (verified in the source tree):

- `FindUnrecoveredSwitchesScript.java`: lists computed jumps without good destinations.
- `SwitchOverride.java`: select the jump, add `COMPUTED_JUMP` references, run the script; it writes a
  `JumpTable` override (`new JumpTable(addr, destlist, true, 0).writeOverride(function)` in 12.1) and fixes the
  function body.
- `FixSwitchStatementsWithDecompiler.java`, `AddReferencesInSwitchTable.java` (ARM-oriented, but a template).
- Decompiler option "Max Entries per Jumptable".

Plan for a project script (`DinoJumpTables.java`):

1. Match the instruction pair with the same PC-relative base `tbl`.
2. Read words at `tbl` as signed offsets; target = `tbl + entry`. Table length: if a guard exists use it,
   otherwise stop at the first entry whose target is odd, outside ROM, or before `tbl + 2*index`; a good default
   is `N = (lowest target - tbl) / 2` when the code follows the table, which is what the CPS2 plugin below does.
3. Clear wrong instructions in the table, define it as `short[N]`, add `COMPUTED_JUMP` references, write the
   override, call `CreateFunctionCmd.fixupFunctionBody`, and re-decompile.
4. Handle the related shapes too: `lea tbl(pc,d0.w),a0` (offset table to data, for example the string table at
   0x764C used by the print routine at 0xC9A: only add data references), `jsr tbl(pc,d0.w)` (about 136, call
   tables: add `COMPUTED_CALL` references), and `movea.l tbl(pc,d0.w),aN` + `jsr (aN)` (pointer tables: define
   pointer arrays).

Reusable starting point: [MBDesu/Ghidra_CPS2](https://github.com/MBDesu/Ghidra_CPS2) (CPS2 loader plus a
"CPS2 Jump Tables" analyzer for exactly this pattern; README calls it "buggy", last push 2024-09, uses the older
3-argument `JumpTable` constructor, so it needs a small port to 12.1).

#### Fix E: 16-bit sign-extended pointers and the A5 base

- Add a byte-mapped mirror so 0xFFFF0000-0xFFFFFFFF and 0xFF0000-0xFFFFFF are the same bytes
  (`memory.createByteMappedBlock("RAM_HI", 0xFFFF0000, 0xFF0000, 0x10000, false)`). This is what
  [ghidra_sega_ldr](https://github.com/lab313ru/ghidra_sega_ldr) does for Mega Drive RAM.
- Pick one canonical alias and keep labels there. Our data says A5-relative access dominates (16,150) and the game
  computes A5 = 0xFFFF8000 itself. Option 1 (cleanest for dataflow): move the real RAM block to 0xFFFF0000, set the
  A5 context to 0xFFFF8000, and put the mirror at 0xFF0000 for the 82 absolute-long uses. Option 2 (less churn):
  keep 0xFF0000 and the A5 = 0xFF8000 context, add the mirror at 0xFFFF0000, and copy labels to the mirror with a
  small script. Either way, labels in `specs/` stay valid if you treat RAM as `addr & 0xFFFF`.
- Note from issue #2140 (lab313ru): a base register value set through `ProgramContext` works, while the GUI
  "assume" did not. Our `DinoSetup.java` already uses `ProgramContext`.
- Mark the I/O area (0x800000-0x8001FF) and QSound shared RAM (0xF18000-0xF19FFF, 0xF1E000-0xF1FFFF per the MAME
  `qsound_main_map`) as **volatile**, so polling loops and repeated writes are not merged.

#### Fix F: other hand-written asm idioms to watch

- Data right after a `jsr` (the callee pops the return address and reads inline data). We have 762 "Possible PIC
  construction ... Changing call to branch" warnings and 74 `movea.l (a7)+,aN` in the sweep. Model such helpers
  with a `<callfixup>` spec extension (p-code that does what the helper does to SP and the return address) or mark
  them inline. The atari_reverse project notes Ghidra "models none of" the `addq.l #n,(a7)` return-skip trick.
- `pea label` + `rts` dispatch and shared function tails (`bra` into another function): check the listing when a
  function looks too short or has "Call to offcut address" warnings (2,379 of them).
- BCD math (`abcd`/`sbcd`, about 275) and the X flag: the C is correct but ugly; port these by hand from the
  disassembly.

#### Reusable 68000 community work (verified unless noted)

| Project | What it gives us | Status |
| --- | --- | --- |
| [lab313ru/ghidra_sega_ldr](https://github.com/lab313ru/ghidra_sega_ldr) | Mega Drive loader; RAM mirror at 0xFFFF0000 via `createByteMappedBlock`; hardware labels | Last push 2024-03 |
| [MBDesu/Ghidra_CPS2](https://github.com/MBDesu/Ghidra_CPS2) | CPS2 loader + jump table analyzer for `move.w/jmp (pc,dN)` | WIP, 2024-09 |
| [BartmanAbyss/ghidra-amiga](https://github.com/BartmanAbyss/ghidra-amiga) | Amiga loader (fork of lab313ru's); shows how a 68k loader sets up memory and symbols | 2026-01 |
| [ubuntor/m68k_mac_reversing_tools](https://github.com/ubuntor/m68k_mac_reversing_tools) | Ghidra + Binary Ninja scripts for 68k Mac (jump table thunks, syscalls) | 2025-02 |
| [Pyrdacor/AmigaAsm](https://github.com/Pyrdacor/AmigaAsm) | Exports m68k asm from Ghidra for vasm (reassemblable source; details unverified) | 2025-11 |
| [geogeo28/atari_reverse](https://github.com/geogeo28/atari_reverse) | Closest workflow to ours: Ghidra 12 headless, Java scripts, C verified against Musashi with differential tests | Active, 2026-10 |
| [lab313ru/smd_ida_tools2](https://github.com/lab313ru/smd_ida_tools2) | IDA Mega Drive tools and debugger | 2026-02 |
| [hippietrail/RetroGhidra](https://github.com/hippietrail/RetroGhidra), [tdaede/X68000Loader](https://github.com/tdaede/X68000Loader) | More retro loaders as templates | 2026-01 / 2023 |

I found no CPS1 loader and no CPS1 game decompilation. Neo Geo: only a Neo Geo Pocket loader (different CPU).

### 4.2 IDA Pro

- **No official Hex-Rays decompiler for m68k.** The decompiler limits page lists 11 variants (x86, x64, ARM,
  ARM64, PowerPC, PowerPC64, MIPS, MIPS64, RISC-V, RISC-V 64, ARC) and says "Support for other processors will
  eventually be added (no deadlines are available, sorry)." IDA 9.3 (Feb 2026) added V850; the 9.4 beta notes
  mention Hexagon and ARM extensions, not m68k.
- IDA disassembles the "Motorola/Freescale 68K family: MC680xx, CPU32 (68330), MC6301".
- No maintained third-party m68k decompiler plugin found. Ghidra-in-IDA bridges do not help: GhIDA was tested only
  on x86/x64 (2021), Yagi marks 68000 as unsupported.
- Licensing (Oct 2026 pricing page): IDA Free and Home do not cover 68K anymore ("IDA Home 68K is retired" in 9.0).
  Pro Essential from EUR 1,099/yr, Expert from EUR 4,999/yr, Ultimate from EUR 8,599/yr, yearly subscriptions.

### 4.3 Binary Ninja

- m68k comes only from the community plugin
  [galenbwill/binaryninja-m68k](https://github.com/galenbwill/binaryninja-m68k) (v0.5, MIT, "disassembles Motorola
  68k machine code and generates LLIL"). Last commit 2022-06-10; one open PR about improving LLIL lifting.
- Binary Ninja builds MLIL/HLIL/pseudo-C from LLIL for any architecture, so HLIL "works" in principle, but I found
  no report of HLIL quality on m68k and the lifter is old: **unverified, probably weak**.
- zznop's "genesis" Mega Drive plugin is listed in the old community plugin list (details unverified).
- Licensing: Free edition cannot run third-party plugins. Personal (non-commercial) $199, Commercial $1,799,
  Ultimate $3,499.

### 4.4 radare2 / rizin / Cutter

- r2ghidra (6.2.4, 2026-10-05) and rz-ghidra both map the `m68k` arch to Sleigh `68000` with `default`, `MC68020`,
  `MC68030` or ColdFire flavors (verified in their `ArchMap.cpp`). Decompile with `pdg`.
- It is the same decompiler engine, but without Ghidra's analyzers, switch analysis, custom storage UI, data type
  manager or spec extensions. The radare2 book says r2ghidra "is not providing the same analysis results that Ghidra
  would provide". No reason to switch.
- r2dec lists m68k as "experimental"; `pdc` works on all archs but is very basic.

### 4.5 Reko

- m68k is rated "Thoroughly tested and stable" (highest level) in the Reko wiki. Releases every few months
  (0.12.5 on 2026-09-29; repository active on 2026-10-09). .NET 9; Windows GUI, cross-platform CLI.
- Raw images: `reko decompile --loader raw --arch <arch> --base <addr> --entry <addr> file` (from `doc/guide/cli.md`;
  the m68k arch name comes from `reko --help`).
- Strength for us: interprocedural register analysis. Reko's own m68k regression output shows signatures such as
  `fn00001000(uipr32 d0, byte * a0)` and `fn0000127C(Register (ptr32 int8) d2, Register out (ptr32 Eq_n) a5Out)`.
- User metadata can force signatures with C attributes, for example
  `[[reko::address("00013FA6")]] void draw_object([[reko::arg(register, "a6")]] struct Object * obj);`
  and `[[reko::returns(register, "a0")]]`.
- Weakness: output style (`SEQ(SLICE(d4_n, word16, 16), ...)` for word writes into 32-bit registers, `d0_n` reused
  everywhere, generic `Eq_n` types). Jump table handling on our pattern is unverified.
- Use: run it once over the ROM and diff its inferred register signatures against what our Ghidra script commits.

### 4.6 Others

- **RetDec**: no m68k; "limited maintenance mode"; last release v5.0 (2022-12).
- **Snowman**: archived 2024-07-23; never supported m68k.
- **Boomerang**: x86, PPC, ST20 only; inactive since 2020.
- **JEB**: decompilers for x86, ARM, MIPS, RISC-V, WASM, EVM and others; no 68000.
- **angr**: can lift 68000 through pypcode (P-code from Ghidra's SLEIGH 12.1) and has a 68000 calling convention
  class; decompiler quality on m68k unverified.
- **Small or academic m68k decompilers**: `pfhore-github/m68k_decompiler` (Haskell, 13 commits, no README),
  `Prashant-Jonny/68000Decompiler` (2014). Not usable. REC Studio once supported 68k (old, unverified).
- **Matching-decomp community**: decomp.me has no 68000 platform (its platform list covers MIPS, PPC, ARM, SH,
  x86). m68k game projects are mostly hand-made disassemblies (Sonic Retro style) because most games were written
  in assembly; the Ghidra_CPS2 README says the same about CPS2 ("most/all of these were hand written in m68k/z80").
  So we cannot do a "matching" decomp; our goal is correct, readable lifted logic.
- **Reassemblable disassembly**: IRA 2.11 (2025-02, Amiga 68k reassembler, portable C by Frank Wille et al.),
  Pyrdacor/AmigaAsm (Ghidra exporter for vasm), IDA output. Useful only if we want a rebuildable asm baseline.
- **Static recompilation**: sp00nznet/macrecomp lifts 68k Mac code to C mechanically (capstone + `lift68k.py`, one
  C statement per instruction, 97 to 100% instruction coverage on HyperCard). Not readable, but a mechanical Rust
  lift of single functions could serve as an exact reference while porting (idea, not tested).
- **LLM-assisted**: ReVa (Ghidra 12.0+, MCP, headless mode), LaurieWired/GhidraMCP, bethington/ghidra-mcp all expose
  Ghidra to an LLM for renaming and retyping. LLM4Decompile refines Ghidra output but is trained on x86-64 Linux.
  A Sept 2026 paper ("When LLM Decompilers Recompile More and Preserve Less", arXiv 2609.05370) found the strongest
  LLM refinement raised Ghidra's build rate from 75% to 90% while behavioral match fell from 74% to 62%. So: let
  the LLM name and type, but every ported function must pass differential tests.

### 4.7 MAME debugger (dynamic complement)

Run with `-debug` (and `-debugscript file.txt` to load commands at startup). Useful commands (verified in the MAME
docs):

| Need | Command |
| --- | --- |
| Log every instruction | `trace file.log,maincpu,noloop,{tracelog "D0=%08X A0=%08X A6=%08X ",d0,a0,a6}` (`traceover` skips subroutines) |
| Who writes a RAM variable | `wpset ff84d9,1,w` with a condition, for example `wpset ff8032,2,w,wpdata!=0,{printf "%x",pc; g}` |
| Last writer of an address | `trackmem 1` then `pcatmem ff8032` |
| What just ran | `history maincpu,50` |
| Step back | `rewind` (RAM save states made on step) |
| Mark visited code | `trackpc 1` (helps find dead code vs real code) |
| Save notes | `comadd 7600,tst of 0xFF8032 deleted by decompiler`, `comsave` |
| Reproducible runs | `statesave` / `stateload`, `source cmds.txt` |

Lua (`-console` or `-autoboot_script`): `manager.machine.devices[":maincpu"].spaces["program"]` gives
`read_u16`/`write_u16` and `install_read_tap` / `install_write_tap(start, end, name, callback)`. A tap on
0xFF0000-0xFFFFFF can log every RAM write per frame; dump RAM each frame and compare with the Rust port.

For per-function checks without MAME, run the original routine on a 68000 core and the Rust function on the same
RAM image and diff (atari_reverse does this with Musashi). Rust cores exist: `m68000` (Stovent, 0.2.3), `r68k`
(Musashi port, 0.2.2, 2026-01), `m68k` (m68k-rs, 0.14.3, 2026-09).

---

## 5. Recommendation for our project

**Primary tool: Ghidra 12.1.4 headless**, with the project-specific setup below. Upgrade to 12.2 when it is
released (check its 68000 notes first).

**Second opinion: Reko** for register signatures (one batch run, compare, do not port from it).

**Dynamic truth: MAME debugger + Lua taps**, and later a differential test harness with a Rust 68000 core.

Improvements, in order of value:

1. Fix the memory map: RAM uninitialized + writable, I/O and QSound volatile, RAM mirror, one canonical alias,
   and `setRespectReadOnly(false)` in `DinoExport.java`. This alone should restore deleted branches.
2. Recover jump tables with a project script (pattern-based, `JumpTable.writeOverride`), so the 1,613 functions
   with `halt_baddata()` get real `switch` statements.
3. Commit register parameters with custom storage (bottom-up script), then type A6 as `Object *` and build the
   object struct.
4. Audit callee-saved registers and flags (`RegisterTouchesPerFunction.java` plus closure), and use a small
   `unaffected` list where the game really clobbers registers.
5. Handle inline-data helpers with call fixups and keep a list of hand-port-only idioms (BCD, X flag, return
   address tricks).

---

## 6. Step-by-step plan to improve our Ghidra output

Do each step, re-run the headless import, regenerate `decomp.c`, and compare the counts from section 2 (keep a
small script that prints them, so every step has a before/after number).

1. **Memory map (in `DinoSetup.java`, before analysis).**
   - `Memory mem = currentProgram.getMemory();`
   - RAM: `MemoryBlock ram = mem.getBlock(toAddr(0xFF0000)); ram = mem.convertToUninitialized(ram); ram.setWrite(true);`
   - I/O, CPS-A/B, QSound: `b.setWrite(true); b.setVolatile(true);` (GFX RAM: writable, not volatile).
   - Mirror: `mem.createByteMappedBlock("RAM_HI", toAddr(0xFFFF0000L), toAddr(0xFF0000), 0x10000, false);`
     (or move RAM to 0xFFFF0000 with `mem.moveBlock` and set A5 to 0xFFFF8000, see Fix E).
   - In `DinoExport.java`: `opts.setRespectReadOnly(false);` or load program options with
     `opts.grabFromProgram(currentProgram)` after setting the option in the program.
   - Check: "Read-only address ... is written" warnings go to 0; FUN_000075b2 shows a test of 0xFF8032; the count
     of "Removing unreachable block" drops.
2. **Jump tables.** Write `DinoJumpTables.java` (section 4.1, Fix D; port ideas from Ghidra_CPS2 and
   `SwitchOverride.java`). Run it after the first auto-analysis, then re-run "Decompiler Switch Analysis".
   Check: number of `switch` statements goes from 21 toward about 600; `halt_baddata` count drops a lot. Spot-check
   10 tables by hand against `tools/d`.
3. **Function boundaries and inline data.** Use `FindUnrecoveredSwitchesScript.java` and
   `FindPotentialDecompilerProblems.java` to list leftovers; fix "PIC construction" sites by creating call fixups or
   marking helpers inline; use the MAME `trackpc` coverage to separate code from data where unsure.
4. **Register parameters.** Write `DinoRegParams.java`: bottom-up custom storage for `in_*`/`unaff_*` registers
   (skip flags), repeat until stable. Optionally compare with the `capcom_asm` register-strategy prototype plus
   Decompiler Parameter ID on a sample. Check: `in_A0` and `unaff_A6` counts near 0; calls show arguments; the two
   identical-looking branches in FUN_000075b2 now differ (they pass different A0/A1 to the print routine at 0xC9A).
5. **Clobber audit.** Run `RegisterTouchesPerFunction.java` over all functions, compute the transitive set of
   modified registers, and assign `capcom_asm` (small unaffected list, flags killed) to functions that do not
   preserve D2-D7/A2-A6. Check: `in_ZF`/`in_CF` turn into `extraout_` or real return values; no caller relies on
   a register its callee changes.
6. **Types.** Create unpacked structs for the object pools (`specs/` already lists PLAYER1, POOL_A_ENEMIES, ...),
   type A6 params as `Object *`, use Auto Fill in Structure, then rename fields. Add enums for state bytes used as
   switch indexes.
7. **Cross-check.** Run Reko on `dino_68k.bin` (`--loader raw --base 0`) and diff its register signatures with ours.
8. **Verify dynamically.** For each function you port, confirm behavior with MAME (watchpoints, traces, RAM dumps
   per frame), and later with a Rust differential harness on a 68000 core.

Tip for the workflow: keep all of this as headless pre/post scripts in `re/ghidra/scripts/` (as now), so a full
re-import is one command and every step is repeatable.

---

## 7. Unverified items (test before relying on them)

- `strategy="register"` prototype: whether `movem`-saved registers become false parameters.
- Using a prototype's first pentry (A6) to make function pointer types mean "object in A6".
- Returning a boolean in `ZF`/`CF` through custom storage.
- Reko's handling of our jump table pattern, its m68k arch name on the CLI, and its CLI on macOS.
- Binary Ninja HLIL quality with the m68k plugin on Binary Ninja 5.x.
- angr decompiler output on 68000.
- Snowman's exact architecture list (project site was unreachable; it never listed m68k as far as I know).
- zznop's Binary Ninja "genesis" plugin features; AmigaAsm output details; REC Studio 68k support.
- Why the ELF loader produced read-only, zero-filled blocks from RW segments (the effect is measured; the cause is
  not). Fixing the blocks in the pre-script avoids the question.
- The jump table and idiom counts from `sweep.txt` are approximate because a linear sweep also decodes data.

---

## 8. Sources

Ghidra
- 68000 cspec (default `__stdcall`, unaffected/killedbycall): https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/68000/data/languages/68000.cspec
- 68000 ldefs (language variants, `68000_register.cspec` only for ColdFire): https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/68000/data/languages/68000.ldefs
- 68000 SLEIGH (`d16` signed, `movea.w` uses `sext`, `jmp` is `goto [..]`): https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/68000/data/languages/68000.sinc
- cspec documentation (`strategy="register"`, unaffected, killedbycall): https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/main/doc/cspec.xml
- ParamListRegister ("any subset of the resource list can be used"): https://www.ghidradocs.com/12.1_PUBLIC/docs/DecompilerDoxygen/html/classghidra_1_1ParamListRegister.html
- Decompiler options help (Respect read-only flags, Specification Extensions, Max Entries per Jumptable): https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/main/help/help/topics/DecompilePlugin/DecompilerOptions.html
- `READONLY_OPTIONDEFAULT = true`: https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/main/java/ghidra/app/decompiler/DecompileOptions.java
- Ghidra advanced class (in_/unaff_/extraout_, SwitchOverride, Decompiler Parameter ID, spec extensions, register values): https://github.com/NationalSecurityAgency/ghidra/blob/master/GhidraDocs/GhidraClass/Advanced/src/improvingDisassemblyAndDecompilation.tex
- Decompiler window help (Commit Params/Return, Override Signature): https://ghidradocs.com/12.1_PUBLIC/help/Decompiler/help/topics/DecompilePlugin/DecompilerWindow.html
- SwitchOverride.java: https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/ghidra_scripts/SwitchOverride.java
- Scripts folders (FindUnrecoveredSwitchesScript, RegisterTouchesPerFunction, FindPotentialDecompilerProblems, CreateStructure, FixupGolangFuncParamStorageScript): https://github.com/NationalSecurityAgency/ghidra/tree/master/Ghidra/Features/Base/ghidra_scripts and https://github.com/NationalSecurityAgency/ghidra/tree/master/Ghidra/Features/Decompiler/ghidra_scripts
- JumpTable API (12.1 constructor with `format`): https://ghidradocs.com/12.1_PUBLIC/docs/GhidraAPI_javadoc/api/ghidra/program/model/pcode/JumpTable.html
- SpecExtension API: https://ghidra.re/ghidra_docs/api/ghidra/program/database/SpecExtension.html
- Function / ParameterImpl / Memory / MemoryBlock APIs: https://ghidradocs.com/12.1_PUBLIC/docs/GhidraAPI_javadoc/api/ghidra/program/model/listing/Function.html , https://ghidradocs.com/12.1_PUBLIC/docs/GhidraAPI_javadoc/api/ghidra/program/model/listing/ParameterImpl.html , https://ghidradocs.com/12.1_PUBLIC/docs/GhidraAPI_javadoc/api/ghidra/program/model/mem/Memory.html
- Change history (12.1.3 movem.w fix and CPU32; 11.2 ext flags; 12.1 bitfields; 12.2 in progress): https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Configurations/Public_Release/src/global/docs/ChangeHistory.md
- Releases (12.1.4 on 2026-09-21): https://github.com/NationalSecurityAgency/ghidra/releases
- Issues: #766 https://github.com/NationalSecurityAgency/ghidra/issues/766 , #230 https://github.com/NationalSecurityAgency/ghidra/issues/230 , #2140 https://github.com/NationalSecurityAgency/ghidra/issues/2140 , #2375 https://github.com/NationalSecurityAgency/ghidra/issues/2375 , #6679 https://github.com/NationalSecurityAgency/ghidra/issues/6679 , #6189 https://github.com/NationalSecurityAgency/ghidra/issues/6189 , #8756 https://github.com/NationalSecurityAgency/ghidra/issues/8756 , #1400 https://github.com/NationalSecurityAgency/ghidra/issues/1400 , #8558 https://github.com/NationalSecurityAgency/ghidra/issues/8558 , #5031 https://github.com/NationalSecurityAgency/ghidra/issues/5031

Ghidra community
- ghidra_sega_ldr (RAM mirror): https://github.com/lab313ru/ghidra_sega_ldr/blob/master/src/main/java/sega/SegaLoader.java
- Ghidra_CPS2: https://github.com/MBDesu/Ghidra_CPS2
- ghidra-amiga: https://github.com/BartmanAbyss/ghidra-amiga ; blog using it with Ghidra 12.1.3 (2026-08-29): https://amiga.sebastian-bergmann.de/2026/08/29/disassembling-my-1994-self/
- m68k_mac_reversing_tools: https://github.com/ubuntor/m68k_mac_reversing_tools
- AmigaAsm: https://github.com/Pyrdacor/AmigaAsm
- atari_reverse (Ghidra 12 + Musashi differential testing): https://github.com/geogeo28/atari_reverse , https://github.com/geogeo28/atari_reverse/blob/main/docs/agent-playbook.md , https://github.com/geogeo28/atari_reverse/blob/main/docs/m68k-disassembly.md
- RetroGhidra: https://github.com/hippietrail/RetroGhidra ; X68000Loader: https://github.com/tdaede/X68000Loader

IDA
- Decompiler limits (11 variants, no m68k): https://docs.hex-rays.com/ida-9.2/user-guide/decompiler/limit
- Supported processors (68K family): https://docs.hex-rays.com/9.1/user-guide/disassembler/supported-processors
- IDA 9.0 release notes (IDA Home 68K retired): https://docs.hex-rays.com/release-notes/9_0
- IDA 9.3 release notes (V850 decompiler): https://docs.hex-rays.com/release-notes/9_3
- Pricing: https://hex-rays.com/pricing ; IDA Home: https://hex-rays.com/ida-home
- GhIDA: https://github.com/Cisco-Talos/GhIDA ; Yagi: https://github.com/airbus-cert/Yagi
- smd_ida_tools2: https://github.com/lab313ru/smd_ida_tools2

Binary Ninja
- m68k plugin: https://github.com/galenbwill/binaryninja-m68k
- Community plugin list (deprecated, lists m68k and genesis): https://github.com/Vector35/community-plugins
- Pricing: https://binary.ninja/purchase/

radare2 / rizin
- r2ghidra: https://github.com/radareorg/r2ghidra , arch map: https://github.com/radareorg/r2ghidra/blob/master/src/ArchMap.cpp
- rz-ghidra: https://github.com/rizinorg/rz-ghidra , arch map: https://github.com/rizinorg/rz-ghidra/blob/dev/src/ArchMap.cpp
- Cutter: https://github.com/rizinorg/cutter
- r2dec: https://github.com/wargio/r2dec-js
- radare2 book, decompilers: https://book.rada.re/arch/decompile.html

Reko
- Repository: https://github.com/uxmal/reko ; releases: https://github.com/uxmal/reko/releases
- Supported binaries (M68k "thoroughly tested and stable"): https://github.com/uxmal/reko/wiki/Supported-binaries
- CLI guide (raw loader): https://github.com/uxmal/reko/blob/master/doc/guide/cli.md
- User metadata (`reko::arg`, `reko::returns`): https://github.com/uxmal/reko/blob/master/doc/guide/usermetadata.md
- m68k sample output (register params and outs): https://github.com/uxmal/reko/blob/master/subjects/Hunk-m68k/FIBO.reko/FIBO_code.c

Others
- RetDec: https://github.com/avast/retdec
- Snowman (archived): https://github.com/yegord/snowman
- Boomerang: https://github.com/BoomerangDecompiler/boomerang
- JEB: https://pnfsoftware.com/jeb
- pypcode languages (68000 entries): https://docs.angr.io/projects/pypcode/en/latest/languages.html ; angr 68000 calling convention: https://github.com/angr/angr/blob/master/angr/engines/pcode/cc.py
- decomp.me platforms: https://github.com/decompme/decomp.me/blob/main/cromper/cromper/platforms.py
- IRA reassembler: https://aminet.net/package/dev/asm/ira
- macrecomp: https://github.com/sp00nznet/macrecomp
- ReVa: https://github.com/cyberkaida/reverse-engineering-assistant ; GhidraMCP: https://github.com/LaurieWired/GhidraMCP ; ghidra-mcp: https://github.com/bethington/ghidra-mcp
- LLM4Decompile: https://github.com/albertan017/LLM4Decompile
- "When LLM Decompilers Recompile More and Preserve Less" (2026-09-04): https://arxiv.org/abs/2609.05370
- Rust 68000 cores: https://crates.io/crates/m68000 , https://crates.io/crates/r68k , https://crates.io/crates/m68k

MAME
- Execution commands (trace, traceover): https://docs.mamedev.org/debugger/execution.html
- Watchpoints: https://docs.mamedev.org/debugger/watchpoint.html
- General commands (history, rewind, trackpc, trackmem, pcatmem, source): https://docs.mamedev.org/debugger/general.html
- Code annotation (comadd, comsave): https://docs.mamedev.org/debugger/annotation.html
- Lua memory API (taps): https://docs.mamedev.org/luascript/ref-mem.html
- Command line (-debug, -debugscript, -autoboot_script, -console): https://docs.mamedev.org/commandline/commandline-all.html
- CPS1 driver memory maps (main_map, qsound_main_map): https://github.com/mamedev/mame/blob/master/src/mame/capcom/cps1.cpp
