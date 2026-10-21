# Routine starts and basic block leaders of the original program, for the coverage recorder (REPORT.md here, "Method").
#   re/.venv/bin/python -I re/harness/coverage/blocks.py [IMAGE] [ENTRIES] [OUT]
# Defaults: re/out/dino_68k.bin, re/ghidra/entries.txt, re/out/coverage/blocks.txt (git-ignored: addresses derived
# from the ROM).
#
# A recursive descent (capstone, 68000) from these seeds:
#   e  every entry of entries.txt (Ghidra's routine list),
#   h  the class handlers of the four object dispatch tables (the addresses re/harness/diff.py uses; Ghidra has none of them),
#   p  every address a doc comment of a port function names (`/// 0x192DE walk.` in dino-rs), when it passes the trial below,
#   q  every code pointer constant (`lea X(pc),aN`, `lea X.l,aN`, `movea.l #X,aN`, `move.l #X,<ea>`, `pea X`) of the decoded
#      code that the port also names somewhere, when it passes the trial.
# The trial walks the new code without keeping it: it must decode (capstone's `dc.w` is data), must not start inside an
# instruction that is already decoded, must not use instructions the game never uses (reset, stop, illegal, rtr, trapv,
# chk, tas, movep), and must not start with a run of mostly immediate or bit operations (tables of small words). Entries from 0xB0000 on (the data area) go through the trial too; the ones that fail stay routine
# starts (they still get a breakpoint) but nothing is decoded from them.
# The walk follows branches, calls and the jump tables of the usual forms (a word table after `jmp T(pc,dN.w)`, a branch
# table, a long table read by `movea.l T(pc,dN.w),aK` or `lea T,aM; movea.l (aM,dN.w),aK` before `jmp (aK)`), and stops at
# rts, rte, rtr, an indirect jump it cannot resolve, or data.
# Output: one line per leader `LEADER STARTKIND NINS KIND`: the hex address, the seed kind when it is a routine start
# (e, h, p; - otherwise), the number of instructions of the block, and why it is a leader (the seed kinds, b branch target,
# f fall through, c call target, t table target, r return point after a call or trap).
# EDGES (OUT with .edges.txt): `FROM TO KIND` for every control transfer the walk saw (b branch, c call, j jump, t table,
# f fall through, r return point, p a pointer constant that is a leader), and `END ADDR MNEMONIC` for every rts, rte, bra, jmp. map.py reads both files.
import bisect
import glob
import os
import re
import struct
import sys

import capstone

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
IMAGE = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "re/out/dino_68k.bin")
ENTRIES = sys.argv[2] if len(sys.argv) > 2 else os.path.join(ROOT, "re/ghidra/entries.txt")
OUT = sys.argv[3] if len(sys.argv) > 3 else os.path.join(ROOT, "re/out/coverage/blocks.txt")
EDGES = OUT[:-4] + ".edges.txt" if OUT.endswith(".txt") else OUT + ".edges"
DATA_FROM = 0xB0000

img = open(IMAGE, "rb").read()
END = len(img)
md = capstone.Cs(capstone.CS_ARCH_M68K, capstone.CS_MODE_BIG_ENDIAN | capstone.CS_MODE_M68K_000)
entries = sorted({int(x.split()[0], 16) for x in open(ENTRIES) if x.strip()})

# Class dispatch tables of the original (u32 handler per class), as in re/harness/diff.py.
HANDLERS = {"enemy": (0x2346C, 102), "prop": (0x2362E, 97), "item": (0x233F0, 12), "effect": (0x237E6, 42)}


def w(a):
    return (img[a] << 8) | img[a + 1]


def l(a):
    return (w(a) << 16) | w(a + 2)


def sext16(v):
    return v - 0x10000 if v & 0x8000 else v


_cache = {}


def decode(a):
    if a in _cache:
        return _cache[a]
    ins = None
    if 0 <= a < END - 1 and a % 2 == 0:
        ins = next(md.disasm(img[a:a + 16], a), None)
        if ins is not None and ins.mnemonic.startswith("dc"):
            ins = None
    _cache[a] = ins
    return ins


def code_ok(a):
    return a % 2 == 0 and 0x40 <= a < END and decode(a) is not None


PCIDX = re.compile(r"^\$([0-9a-f]+)\(pc, ?([da]\d)\.[wl]\)(?:, ?(.*))?$")
PCREL = re.compile(r"^\$([0-9a-f]+)\(pc\), ?(a\d)$")
ABSL = re.compile(r"^#?\$([0-9a-f]+)(?:\.[lw])?, ?(a\d)$")
AIDX = re.compile(r"^(?:\$0)?\((a\d), ?[da]\d\.[wl]\), ?(a\d)$")
INDIR = re.compile(r"^(?:\$[0-9a-f]+)?\((a\d)\)$")
TARGET = re.compile(r"^\$([0-9a-f]+)(?:\.[lw])?$")
CONST = re.compile(r"^(?:#\$([0-9a-f]+)|\$([0-9a-f]+)\(pc\)|\$([0-9a-f]+)\.l)(?:,|$)")
COND = ("bhi", "bls", "bcc", "bcs", "bne", "beq", "bvc", "bvs", "bpl", "bmi", "bge", "blt", "bgt", "ble")
BAD = ("reset", "stop", "illegal", "rtr", "trapv", "chk", "tas", "movep")

seeds_known = set(entries)


def next_known_after(a):
    i = bisect.bisect_right(entries, a)
    return entries[i] if i < len(entries) else END


def word_table(t_tab, t_base):
    """Targets of a word offset table at t_tab (offsets from t_base)."""
    out, lo = [], next_known_after(t_tab)
    k = 0
    while k < 256 and t_tab + 2 * k + 1 < END:
        pos = t_tab + 2 * k
        if pos >= lo:
            break
        tgt = t_base + sext16(w(pos))
        if t_tab <= tgt < pos + 2 or not code_ok(tgt):
            break
        out.append(tgt)
        if tgt > t_tab:
            lo = min(lo, tgt)
        k += 1
    return out


def long_table(t):
    out = []
    lo = next_known_after(t)
    for k in range(512):
        pos = t + 4 * k
        if pos + 3 >= END or (pos >= lo and k > 0):
            break
        v = l(pos)
        if v == 0:
            continue
        if not code_ok(v):
            break
        out.append(v)
    return out


def branch_table(t):
    out, pos = [], t
    for _ in range(128):
        ins = decode(pos)
        if ins is None or not (ins.mnemonic.startswith("bra") or ins.mnemonic == "jmp"):
            break
        out.append(pos)
        pos += ins.size
    return out


leaders = {}     # address -> kind
start_kind = {}  # routine start -> seed kind
work = []
insn_start = {}  # instruction address -> size
edges = []


def add(a, kind):
    if not code_ok(a):
        return
    if a not in leaders:
        leaders[a] = kind
        work.append(a)


def inside_other(a):
    """a is not an instruction start but lies inside a decoded instruction."""
    if a in insn_start:
        return False
    for back in (2, 4, 6, 8):
        s = insn_start.get(a - back)
        if s is not None and s > back:
            return True
    return False


def step(a, ins):
    """Control flow of one instruction: (targets, falls through, table seen)."""
    m, ops = ins.mnemonic, ins.op_str.strip()
    base = m.split(".")[0]
    nxt = a + ins.size
    return base, ops, nxt


def resolve(base, ops, recent):
    """Targets of a jmp / jsr / bsr; (targets, kind, is_table)."""
    t = TARGET.match(ops)
    if t:
        return [int(t.group(1), 16)], False
    p = PCIDX.match(ops)
    q = INDIR.match(ops)
    if p:
        tb, reg = int(p.group(1), 16), p.group(2)
        for pm, pops in reversed(recent[:-1]):
            pp = PCIDX.match(pops)
            if pm.startswith("move.w") and pp and pops.endswith(reg):
                return word_table(int(pp.group(1), 16), tb), True
        if (w(tb) >> 8) == 0x60 or w(tb) in (0x4EF9, 0x4EFA):
            return branch_table(tb), True
        return word_table(tb, tb), True
    if q:
        reg = q.group(1)
        for i in range(len(recent) - 2, -1, -1):
            pm, pops = recent[i]
            pp, pr, pa, px = PCIDX.match(pops), PCREL.match(pops), ABSL.match(pops), AIDX.match(pops)
            if pp and pp.group(3) == reg and pm.split(".")[0] in ("movea", "move"):
                return long_table(int(pp.group(1), 16)), True
            if (pr and pr.group(2) == reg) or (pa and pa.group(2) == reg and pm.split(".")[0] in ("lea", "movea")):
                return [int((pr or pa).group(1), 16)], True
            if px and px.group(2) == reg and pm.split(".")[0] in ("movea", "move"):
                src = px.group(1)
                for j in range(i - 1, -1, -1):
                    qm, qops = recent[j]
                    qr, qa = PCREL.match(qops), ABSL.match(qops)
                    if (qr and qr.group(2) == src) or (qa and qa.group(2) == src and qm.split(".")[0] in ("lea", "movea")):
                        return long_table(int((qr or qa).group(1), 16)), True
                return [], True
            if pops.endswith(reg):
                return [], True
    return [], False


def dense_immediates(x, n=24):
    """More than 60 % of the first instructions (up to a terminator, at least 4) have an opcode word below 0x1000
    (ori, andi, subi, addi, eori, cmpi, the bit operations, movep): tables of small words decode like that."""
    z = k = 0
    p = x
    for _ in range(n):
        ins = decode(p)
        if ins is None:
            break
        z += img[p] >> 4 == 0
        k += 1
        p += ins.size
        if ins.mnemonic.split(".")[0] in ("rts", "rte", "bra", "jmp"):
            break
    return k >= 4 and z > 0.6 * k


def trial(x, limit=300):
    """Walk new code from x without keeping it. True when it looks like code (see the head of the file)."""
    if not code_ok(x) or inside_other(x) or dense_immediates(x):
        return False
    seen, stack, n = set(), [x], 0
    while stack:
        a = stack.pop()
        recent = []
        while True:
            if a in seen or a in insn_start:
                break
            if inside_other(a):
                return False
            ins = decode(a)
            if ins is None:
                return False
            base, ops, nxt = step(a, ins)
            if base in BAD:
                return False
            seen.add(a)
            n += 1
            if n >= limit:
                return True
            recent.append((ins.mnemonic, ops))
            recent = recent[-8:]
            if base in ("rts", "rte"):
                break
            if base == "bra":
                t = TARGET.match(ops)
                if t:
                    stack.append(int(t.group(1), 16))
                break
            if base in COND or base.startswith("db"):
                t = TARGET.match(ops.split(",")[-1].strip())
                if t:
                    stack.append(int(t.group(1), 16))
                a = nxt
                continue
            if base == "jmp":
                tg, _ = resolve(base, ops, recent)
                stack.extend(tg)
                break
            a = nxt
    return True


def walk_all():
    while work:
        a = work.pop()
        recent = []
        prev = None
        while True:
            ins = decode(a)
            if ins is None:
                break
            if a in insn_start:   # reached code that an earlier walk decoded: the block goes on there
                if prev is not None:
                    edges.append((prev, a, "f"))
                break
            insn_start[a] = ins.size
            base, ops, nxt = step(a, ins)
            recent.append((ins.mnemonic, ops))
            recent = recent[-8:]
            if base in ("rts", "rte", "rtr", "illegal"):
                break
            if base == "bra":
                t = TARGET.match(ops)
                if t:
                    add(int(t.group(1), 16), "b")
                    edges.append((a, int(t.group(1), 16), "b"))
                break
            if base in COND or base.startswith("db"):
                t = TARGET.match(ops.split(",")[-1].strip())
                if t:
                    add(int(t.group(1), 16), "b")
                    edges.append((a, int(t.group(1), 16), "b"))
                add(nxt, "f")
                edges.append((a, nxt, "f"))
                break
            if base in ("bsr", "jsr", "jmp"):
                is_jump = base == "jmp"
                found, table = resolve(base, ops, recent)
                for f in found:
                    add(f, "t" if table else ("b" if is_jump else "c"))
                    edges.append((a, f, "t" if table else ("j" if is_jump else "c")))
                if is_jump:
                    break
                add(nxt, "r")
                edges.append((a, nxt, "r"))
                break
            if base == "trap":
                add(nxt, "r")
                edges.append((a, nxt, "r"))
                break
            prev = a
            a = nxt


def seed(a, kind):
    if a in leaders:
        start_kind.setdefault(a, kind)
        return
    if not code_ok(a):
        if kind == "e":
            start_kind[a] = kind   # still a routine start (it gets a breakpoint), but it is data
        return
    leaders[a] = kind
    start_kind[a] = kind
    work.append(a)


# 1. Entries below the data area, then the class handlers.
for e in entries:
    if e < DATA_FROM:
        seed(e, "e")
walk_all()
for kind, (t, n) in HANDLERS.items():
    for c in range(n):
        v = l(t + 4 * c)
        if v and v % 2 == 0 and v < END:
            seed(v, "h")
walk_all()

# 2. Port function addresses (doc comments of `fn` items) and the entries of the data area, with the trial.
port_any, port_fn = set(), set()
HEX = re.compile(r"0x([0-9A-Fa-f][0-9A-Fa-f_]{2,8})")
for path in glob.glob(os.path.join(ROOT, "dino-rs/crates/*/src/**/*.rs"), recursive=True):
    doc = []
    for line in open(path, encoding="utf-8", errors="replace"):
        s = line.strip()
        for m in HEX.finditer(s):
            port_any.add(int(m.group(1).replace("_", ""), 16))
        if s.startswith("///") or s.startswith("#["):
            doc.append(s)
            continue
        if re.match(r"(pub(\([a-z]+\))? )?(const )?(unsafe )?fn ", s):
            for d in doc:
                if d.startswith("///"):
                    for m in HEX.finditer(d):
                        port_fn.add(int(m.group(1).replace("_", ""), 16))
        doc = []
rejected = {"p": 0, "q": 0, "e": 0}
REJECT_LOG = []
for a in sorted(port_fn):
    if 0x400 <= a < DATA_FROM and a % 2 == 0 and a not in insn_start:
        if trial(a):
            seed(a, "p")
            walk_all()
        else:
            rejected["p"] += 1
            REJECT_LOG.append(a)
for e in entries:
    if e >= DATA_FROM and e in leaders:
        start_kind.setdefault(e, "e")
    elif e >= DATA_FROM:
        if trial(e):
            seed(e, "e")
            walk_all()
        else:
            rejected["e"] += 1
            start_kind[e] = "e"

# 3. Code pointer constants of the decoded code that the port names, with the trial; repeat until nothing new.
while True:
    new = []
    for a in sorted(insn_start):
        ins = decode(a)
        m = ins.mnemonic.split(".")[0]
        if m not in ("lea", "pea", "move", "movea"):
            continue
        c = CONST.match(ins.op_str.strip())
        if not c:
            continue
        x = int(next(g for g in c.groups() if g), 16)
        if 0x400 <= x < DATA_FROM and x % 2 == 0 and x in port_any and x not in leaders and x not in insn_start:
            new.append(x)
    added = 0
    for x in sorted(set(new)):
        if x in leaders or x in insn_start:
            continue
        if trial(x):
            leaders[x] = "q"
            work.append(x)
            walk_all()
            added += 1
        else:
            rejected["q"] += 1
    if added == 0:
        break

# Pointer edges: a constant of the decoded code that is a leader (a state handler stored in an object, a task entry, ...).
for a in sorted(insn_start):
    ins = decode(a)
    if ins.mnemonic.split(".")[0] not in ("lea", "pea", "move", "movea"):
        continue
    c = CONST.match(ins.op_str.strip())
    if c:
        x = int(next(g for g in c.groups() if g), 16)
        if x in leaders and x != a:
            edges.append((a, x, "p"))

# Output.
order = sorted(set(leaders) | set(start_kind))
os.makedirs(os.path.dirname(OUT), exist_ok=True)
overlap = 0
starts = sorted(insn_start)
for i in range(1, len(starts)):
    if starts[i - 1] + insn_start[starts[i - 1]] > starts[i]:
        overlap += 1
with open(OUT, "w") as fh:
    for i, a in enumerate(order):
        n, p = 0, a
        lim = order[i + 1] if i + 1 < len(order) else END
        while p in insn_start and p < lim:
            n += 1
            p += insn_start[p]
        fh.write("%x %s %d %s\n" % (a, start_kind.get(a, "-"), n, leaders.get(a, start_kind.get(a, "-"))))
with open(EDGES, "w") as fh:
    for f, t, k in edges:
        fh.write("%x %x %s\n" % (f, t, k))
    for a in starts:
        m = decode(a).mnemonic.split(".")[0]
        if m in ("rts", "rte", "rtr", "illegal", "bra", "jmp"):
            fh.write("END %x %s\n" % (a, m))
kinds = {}
for a, k in start_kind.items():
    kinds[k] = kinds.get(k, 0) + 1
with open(OUT[:-4] + ".rejected.txt" if OUT.endswith(".txt") else OUT + ".rejected", "w") as fh:
    for a in REJECT_LOG:
        fh.write("%x\n" % a)
print("routine starts %d %s; leaders %d; instructions %d; overlapping instruction pairs %d; trial rejects %s -> %s"
      % (len(start_kind), kinds, len(order), len(insn_start), overlap, rejected, OUT))
