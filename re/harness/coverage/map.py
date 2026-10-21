# Map the coverage hits (run.sh) to the routines of the original and to the port (re/harness/coverage/REPORT.md, "Method").
#   re/.venv/bin/python -I re/harness/coverage/map.py [BATCH ...]
# BATCH = a folder name under re/out/coverage/hits (default: every batch folder there except the test runs pilot*,
# ptest*, t_*). Reads re/ghidra/entries.txt, re/out/coverage/blocks.txt and blocks.edges.txt (blocks.py), the class
# dispatch tables of the original (the addresses diff.py uses), the Ghidra index when it is there (names only), and
# the port sources (dino-rs/crates/*/src: every hex number and every identifier suffix such as pass_10cd0).
# Writes (git-ignored, they list ROM addresses):
#   re/out/coverage/routines.tsv  one row per routine start: the entries of entries.txt and the class handlers
#   re/out/coverage/blocks_hit.tsv one row per block leader: leader, routine, instructions, hit
#   re/out/coverage/block_gates.tsv, never_blocks.tsv, never.tsv, gates.tsv: the gates, the never-run blocks and routines
# and prints the summary numbers that REPORT.md quotes.
import bisect
import glob
import os
import re
import struct
import sys
from collections import Counter, defaultdict

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
OUT = os.path.join(ROOT, "re/out/coverage")
img = open(os.path.join(ROOT, "re/out/dino_68k.bin"), "rb").read()


def l32(a):
    return struct.unpack(">I", img[a:a + 4])[0]


entries = sorted({int(x.split()[0], 16) for x in open(os.path.join(ROOT, "re/ghidra/entries.txt")) if x.strip()})
entry_set = set(entries)
# Only the addresses the batches watched (addrs.txt) count: a leader that blocks.py finds later has no data.
measured = {int(x, 16) for x in open(os.path.join(OUT, "addrs.txt")) if x.strip()}
blocks = []        # (leader, start kind or "-", instructions, leader kind)
for line in open(os.path.join(OUT, "blocks.txt")):
    a, sk, n, k = line.split()
    if int(a, 16) in measured:
        blocks.append((int(a, 16), sk, int(n), k))
leaders = [b[0] for b in blocks if b[2] > 0]
leader_set = set(leaders)
nins = {b[0]: b[2] for b in blocks}
start_kind = {b[0]: b[1] for b in blocks if b[1] != "-"}

# Class dispatch tables (u32 handler per class): pool kind -> (table, entries). Same as re/harness/diff.py HANDLERS.
HANDLERS = {"enemy": (0x2346C, 102), "prop": (0x2362E, 97), "item": (0x233F0, 12), "effect": (0x237E6, 42)}
POOL = {"enemy": "A", "prop": "B", "item": "E", "effect": "C"}
handler_of = defaultdict(list)   # address -> ["A10", ...]
for kind, (t, n) in HANDLERS.items():
    for c in range(n):
        v = l32(t + 4 * c)
        if v and v % 2 == 0 and v < len(img):
            handler_of[v].append("%s%d" % (POOL[kind], c))

# Vector table: exception and TRAP handlers.
vectors = {}
for i in range(1, 64):
    v = l32(4 * i)
    if v < 0x2000 and v % 2 == 0:
        vectors.setdefault(v, []).append(i)

names = {}
for idx in (os.path.join(ROOT, "re/ghidra/decomp.c.index.tsv"),):
    if os.path.exists(idx):
        for line in open(idx):
            p = line.rstrip("\n").split("\t")
            if len(p) >= 2 and not p[1].startswith("FUN_"):
                names[int(p[0], 16)] = p[1]

# Routine starts: the entries (e), the class handlers (h) and the port routines (p) of blocks.py, and every call target
# (c: the target of a bsr / jsr the descent saw), so a routine ends where the next one starts.
raw_edges = []
for line in open(os.path.join(OUT, "blocks.edges.txt")):
    p = line.split()
    if p[0] != "END" and not (p[2] == "p" and int(p[1], 16) < 0x1000):   # a small constant is a number, not a pointer
        raw_edges.append((int(p[0], 16), int(p[1], 16), p[2]))
for f, t, k in raw_edges:
    if k == "c" and t in leader_set and t not in start_kind:
        start_kind[t] = "c"
starts = sorted(start_kind)
start_set = set(starts)


def owner(a):
    i = bisect.bisect_right(starts, a) - 1
    return starts[i] if i >= 0 else a


# Hits.
hits_root = os.path.join(OUT, "hits")
want = sys.argv[1:] or sorted(d for d in os.listdir(hits_root)
                              if os.path.isdir(os.path.join(hits_root, d)) and not d.startswith(("pilot", "ptest", "t_")))
hit = set()
first_job = {}
jobs_of = defaultdict(set)
for b in want:
    for f in sorted(glob.glob(os.path.join(hits_root, b, "*.hits"))):
        if "._between." in f:
            continue
        job = b + "/" + os.path.basename(f).rsplit("-", 1)[0]
        for line in open(f):
            a = int(line, 16)
            hit.add(a)
            jobs_of[a].add(job)
            first_job.setdefault(a, job)

# Port mentions: every hex literal and every identifier suffix of 4..6 hex digits with a digit in it.
port = defaultdict(set)   # address -> files
HEX = re.compile(r"0x([0-9A-Fa-f][0-9A-Fa-f_]{2,8})")
IDS = re.compile(r"\b[a-z][a-z0-9]*_([0-9a-f]{4,6})\b")
for path in glob.glob(os.path.join(ROOT, "dino-rs/crates/*/src/**/*.rs"), recursive=True):
    rel = os.path.relpath(path, os.path.join(ROOT, "dino-rs/crates"))
    for line in open(path, encoding="utf-8", errors="replace"):
        for m in HEX.finditer(line):
            try:
                port[int(m.group(1).replace("_", ""), 16)].add(rel)
            except ValueError:
                pass
        for m in IDS.finditer(line):
            s = m.group(1)
            if re.search(r"[0-9]", s):
                port[int(s, 16)].add(rel)

# Edges: static callers of each start, and the block of each edge source.
callers = defaultdict(set)    # start -> set of source instruction addresses
edges = []                    # (from instruction, to, kind) between routines
for f, t, k in raw_edges:
    if k in ("c", "j", "t", "b", "f", "p") and owner(f) != owner(t):
        edges.append((f, t, k))
        if t in start_set and k not in ("f", "p"):
            callers[t].add(f)


# The class dispatch: the instruction that jumps through a class table reaches every handler of that table (the descent
# may stop reading a table early).
for kind, (t, n) in HANDLERS.items():
    hs = {v for v, cs in handler_of.items() if any(c[0] == POOL[kind] for c in cs)}
    srcs = {f for f, tt, k in edges if k == "t" and tt in hs}
    for f in srcs:
        for h in hs:
            if (f, h, "t") not in edges:
                edges.append((f, h, "t"))
                callers[h].add(f)


def block_of(a):
    i = bisect.bisect_right(leaders, a) - 1
    return leaders[i] if i >= 0 else a


# Per routine: blocks and instructions run.
rblocks = defaultdict(list)
for a, sk, n, k in blocks:
    if n > 0:
        rblocks[owner(a)].append(a)


def is_code(a):
    return a in leader_set


rows = []
for s in starts:
    bl = rblocks.get(s, [])
    bh = [b for b in bl if b in hit]
    ins = sum(nins[b] for b in bl)
    insh = sum(nins[b] for b in bh)
    end = starts[bisect.bisect_right(starts, s)] if bisect.bisect_right(starts, s) < len(starts) else len(img)
    inner = sorted(a for a in port if s < a < end and a in leader_set)
    pm = "named" if s in port else ("inside" if inner else "no")
    cs = callers.get(s, set())
    csh = [c for c in cs if block_of(c) in hit]
    kind = {"e": "entry", "h": "handler", "p": "port", "c": "call"}[start_kind[s]]
    label = names.get(s, "")
    if s in handler_of:
        label = (label + " " if label else "") + "class " + ",".join(handler_of[s])
    if s in vectors:
        label = (label + " " if label else "") + "vector " + ",".join(str(v) for v in vectors[s])
    rows.append(dict(addr=s, kind=kind, code=is_code(s), hit=s in hit, blocks=len(bl), bhit=len(bh), ins=ins, inshit=insh,
                     port=pm, portfiles=",".join(sorted(port.get(s, set())))[:120], callers=len(cs), callers_hit=len(csh),
                     first=first_job.get(s, ""), njobs=len(jobs_of.get(s, ())), label=label, end=end))

with open(os.path.join(OUT, "routines.tsv"), "w") as fh:
    cols = ["addr", "end", "kind", "code", "hit", "blocks", "bhit", "ins", "inshit", "port", "callers", "callers_hit", "first", "njobs", "label", "portfiles"]
    fh.write("\t".join(cols) + "\n")
    for r in rows:
        fh.write("\t".join(("%x" % r[c]) if c in ("addr", "end") else str(r[c]) for c in cols) + "\n")
with open(os.path.join(OUT, "blocks_hit.tsv"), "w") as fh:
    for a, sk, n, k in blocks:
        if n > 0:
            fh.write("%x\t%x\t%d\t%d\n" % (a, owner(a), n, a in hit))

# Gates: a routine that never ran but that a routine that ran can call or jump to (the edge is a call or branch in a
# block that never ran, or a table entry the dispatch never took). Below each gate: the routines that never ran and that
# only never-run code reaches from it (its subtree), with their instructions and how many of them the port names.
row_of = {r["addr"]: r for r in rows}
succ = defaultdict(set)
for f, t, k in edges:
    succ[owner(f)].add(owner(t))
gates = {}
for f, t, k in edges:
    o, to = owner(f), owner(t)
    if row_of.get(o, {}).get("hit") and to in row_of and not row_of[to]["hit"] and row_of[to]["code"]:
        g = gates.setdefault(to, {"from": set(), "kinds": set(), "block_hit": False})
        g["from"].add(o)
        g["kinds"].add(k)
        g["block_hit"] |= block_of(f) in hit
# Each never-run routine belongs to the gates it can be reached from without passing through code that ran.
subtree = {}
for g in gates:
    seen, stack = {g}, [g]
    while stack:
        x = stack.pop()
        for y in succ.get(x, ()):
            if y not in seen and y in row_of and not row_of[y]["hit"] and row_of[y]["code"]:
                seen.add(y)
                stack.append(y)
    subtree[g] = seen
with open(os.path.join(OUT, "gates.tsv"), "w") as fh:
    fh.write("gate\tlabel\tport\tedge\tcaller_block_ran\tcallers\tsub_routines\tsub_ins\tsub_port_named\tsub_list\n")
    for g in sorted(gates, key=lambda g: -sum(row_of[x]["ins"] for x in subtree[g])):
        sub = subtree[g]
        fh.write("%x\t%s\t%s\t%s\t%d\t%s\t%d\t%d\t%d\t%s\n" % (
            g, row_of[g]["label"], row_of[g]["port"], "".join(sorted(gates[g]["kinds"])), gates[g]["block_hit"],
            ",".join("%x" % c for c in sorted(gates[g]["from"]))[:80], len(sub), sum(row_of[x]["ins"] for x in sub),
            sum(row_of[x]["port"] != "no" for x in sub), ",".join("%x" % x for x in sorted(sub))[:400]))
never = [r for r in rows if r["code"] and not r["hit"] and r["addr"] < 0xB0000]
under = set().union(*subtree.values()) if subtree else set()
print("gates (never-run routines next to code that ran): %d; never-run code routines below 0xB0000: %d, under a gate %d, "
      "not reachable from code that ran %d" % (len(gates), len(never), sum(r["addr"] in under for r in never),
                                                sum(r["addr"] not in under for r in never)))

# Groups of the never-run routines (REPORT.md section 4). Rules, in this order:
#   error  reachable from the exception vectors (bus error .. line F, spurious and unused interrupts), not from code that ran
#   test   reachable from the test mode task 0xA6CE8 (service menu), not from code that ran
#   debug / region / dip   the gate (the never-run routine next to code that ran) is reached from a block that reads the debug
#          flag G.$4AA, the region word G.$32 or the DIP switches (0x80001A..E), in that block or the two blocks before it
#   tamper the anti-tamper checks 0x2A5B6, 0x29ADE, 0x104F6 and what only their failure path reaches
#   sound  the 68000 side of the sound (the request and flush helpers 0x9500..0x9FFF) or code that writes the QSound RAM 0xF18000..
#   class  the gate is the handler of an object class (dispatch table entry): a class no recording spawns
#   layout2p (blocks only) the gate depends on the 2 player cabinet layout G.$4CF (chute mode 2)
#   game   any other gate below code that ran: a branch, a state or a call the recordings never took
#   none   no static way in from code that ran (called through a pointer the descent does not follow, or unused)
import capstone
_md = capstone.Cs(capstone.CS_ARCH_M68K, capstone.CS_MODE_BIG_ENDIAN | capstone.CS_MODE_M68K_000)


def block_text(leader):
    out, p = [], leader
    for _ in range(nins.get(leader, 0)):
        ins = next(_md.disasm(img[p:p + 16], p), None)
        if ins is None:
            break
        out.append(ins.mnemonic + " " + ins.op_str)
        p += ins.size
    return " | ".join(out)


def reach(roots, allow):
    seen, stack = set(roots), list(roots)
    while stack:
        x = stack.pop()
        for y in succ.get(x, ()):
            if y not in seen and allow(y):
                seen.add(y)
                stack.append(y)
    return seen


never_ok = lambda y: y in row_of and not row_of[y]["hit"] and row_of[y]["code"]
err_roots = sorted({v for v, vs in vectors.items() if any(i in range(2, 12) or i in (24, 25, 27, 28, 29, 30, 31) for i in vs)})
err_set = reach([r for r in err_roots if never_ok(r)], never_ok)
test_set = reach([0xA6CE8] if never_ok(0xA6CE8) else [], never_ok)
tamper_set = reach([a for a in (0x2A5B6, 0x29ADE, 0x104F6) if a in row_of], lambda y: never_ok(y))
leader_list = sorted(nins)


def gate_flags(g):
    flags = set()
    for f, t, k in edges:
        if owner(t) != g or not row_of.get(owner(f), {}).get("hit"):
            continue
        b = block_of(f)
        i = bisect.bisect_left(leader_list, b)
        text = " ".join(block_text(x) for x in leader_list[max(0, i - 2):i + 1])
        if "$4aa(a5)" in text:
            flags.add("debug")
        if "$32(a5)" in text:
            flags.add("region")
        if re.search(r"\$80001[ace]", text):
            flags.add("dip")
    return flags


group_of, gate_of = {}, {}
gflags = {g: gate_flags(g) for g in gates}


def gate_type(g):
    f = gflags[g]
    if g in (0xA6CE8, 0x9D5A4) or "dip" in f:
        return "test"
    if "debug" in f:
        return "debug"
    if "region" in f:
        return "region"
    if g in tamper_set:
        return "tamper"
    if handler_of.get(g):
        return "class"
    return "game"


PRIO = ["class", "game", "tamper", "region", "debug", "test"]
order_g = sorted(gates, key=lambda g: -sum(row_of[x]["ins"] for x in subtree[g]))
for r in never:
    a = r["addr"]
    my = [g for g in order_g if a in subtree[g]]
    text = " ".join(block_text(b) for b in rblocks.get(a, [])[:40])
    if my:
        best = min(my, key=lambda g: PRIO.index(gate_type(g)))
        gate_of[a] = best
        grp = gate_type(best)
    else:
        gate_of[a] = None
        grp = "error" if a in err_set else "none"
    if (0x9500 <= a < 0xA000 or "$f18" in text or "$f19" in text) and grp in ("none", "error", "test", "game"):
        grp = "sound"
    group_of[a] = grp
with open(os.path.join(OUT, "never.tsv"), "w") as fh:
    fh.write("addr\tgroup\tgate\tins\tport\tlabel\tgate_label\n")
    for r in never:
        g = gate_of[r["addr"]]
        fh.write("%x\t%s\t%s\t%d\t%s\t%s\t%s\n" % (r["addr"], group_of[r["addr"]], "%x" % g if g else "-", r["ins"], r["port"],
                                                r["label"], row_of[g]["label"] if g else ""))
gc = defaultdict(lambda: [0, 0, 0, 0])
for r in never:
    x = gc[group_of[r["addr"]]]
    x[0] += 1
    x[1] += r["ins"]
    x[2] += r["port"] != "no"
    x[3] += r["port"] == "named"
print("never-run code routines below 0xB0000 by group (routines, instructions, port names it or an address inside, port names the start):")
for k in ("error", "test", "debug", "region", "dip", "tamper", "sound", "class", "game", "none"):
    if k in gc:
        print("  %-7s %5d %7d %5d %5d" % ((k,) + tuple(gc[k])))

# Block gates (REPORT.md section 5): a block that never ran, entered by an edge from a block that ran (a branch never
# taken, a call in a branch never taken, a table entry never used). Its region: the never-run blocks it reaches without
# passing a block that ran. Each never-run block is counted once, in the largest region that reaches it.
TERM = ("rts", "rte", "rtr", "illegal", "bra", "jmp")
bsucc = defaultdict(set)
lead_sorted = sorted(nins)
last_mn = {}
for i, L in enumerate(lead_sorted):
    p, m = L, ""
    for _ in range(nins[L]):
        ins = next(_md.disasm(img[p:p + 16], p), None)
        if ins is None:
            break
        m = ins.mnemonic.split(".")[0]
        p += ins.size
    last_mn[L] = m
    if nins[L] and m not in TERM and not m.startswith("b") and not m.startswith("db") and m not in ("jsr", "trap") and p in nins:
        bsucc[L].add(p)
bpred_src = defaultdict(set)
pred_kinds = defaultdict(set)
for f, t, k in raw_edges:
    if t in nins:
        b = block_of(f)
        bsucc[b].add(t)
        bpred_src[t].add(f)
        if b in hit:
            pred_kinds[t].add(k)
for L in lead_sorted:
    for t in bsucc[L]:
        bpred_src[t].add(L) if False else None
bgates = {}
for L in lead_sorted:
    if L in hit or L >= 0xB0000 or not nins[L]:
        continue
    srcs = [f for f in bpred_src.get(L, ()) if block_of(f) in hit]
    falls = [b for b in lead_sorted if False]
    if srcs:
        bgates[L] = srcs
# implicit fall through from a block that ran into a block that never ran is also a gate
for L in lead_sorted:
    if L in hit:
        for t in bsucc[L]:
            if t not in hit and t < 0xB0000 and nins.get(t) and t not in bgates:
                bgates[t] = [L]
region = {}
for g in bgates:
    seen, stack = {g}, [g]
    while stack:
        x = stack.pop()
        for y in bsucc.get(x, ()):
            if y not in seen and y not in hit and y in nins and nins[y] and y < 0xB0000:
                seen.add(y)
                stack.append(y)
    region[g] = seen
owner_region = {}
for g in sorted(region, key=lambda g: -sum(nins[x] for x in region[g])):
    for x in region[g]:
        owner_region.setdefault(x, g)
own = defaultdict(set)
for x, g in owner_region.items():
    own[g].add(x)


def src_text(f):
    ins = next(_md.disasm(img[f:f + 16], f), None)
    return "%x: %s %s" % (f, ins.mnemonic, ins.op_str) if ins else "%x" % f


def bgate_group(g):
    srcs = bgates[g]
    text = " ".join(block_text(x) for f in srcs for x in lead_sorted[max(0, bisect.bisect_left(lead_sorted, block_of(f)) - 2):
                                                                    bisect.bisect_left(lead_sorted, block_of(f)) + 1])
    if any(0xA6CE8 <= x < 0xAAC30 for x in region[g]) or 0x9D5A4 <= g < 0x9D5C0:
        return "test"
    if "$4aa(a5)" in text:
        return "debug"
    if "$32(a5)" in text:
        return "region"
    if re.search(r"\$80001[ace]", text):
        return "test"
    if any(owner(f) in (0x2A5B6, 0x29ADE, 0x104F6) for f in srcs):
        return "tamper"
    if "$4cf(a5)" in text or ("$4cc(a5)" in text and "$3(a6)" in text):
        return "layout2p"
    if handler_of.get(g):
        return "class"
    if 0x9500 <= g < 0xA000:
        return "sound"
    return "game"


def doubtful(g):
    """Every way into the gate is a pc-relative table entry that points far from its table: often a table of pairs
    (offset, parameter) read as a table of offsets."""
    if pred_kinds.get(g) == {"p"}:
        return True       # only a pointer constant leads there: it may be a data pointer
    ok = False
    for f in bgates[g]:
        ins = next(_md.disasm(img[f:f + 16], f), None)
        if ins is None or "(pc," not in ins.op_str.replace(" ", "") or abs(g - f) <= 0x1000:
            ok = True
    return not ok


with open(os.path.join(OUT, "block_gates.tsv"), "w") as fh:
    fh.write("gate\tdoubt\tgroup\troutine\troutine_label\town_blocks\town_ins\treach_ins\troutines\tport_named\tfrom\n")
    for g in sorted(own, key=lambda g: -sum(nins[x] for x in own[g])):
        xs = own[g]
        rs = sorted({owner(x) for x in xs})
        named = sum(1 for x in xs if x in port) + sum(1 for r in rs if r in port and not row_of[r]["hit"])
        fh.write("%x\t%d\t%s\t%x\t%s\t%d\t%d\t%d\t%d\t%d\t%s\n" % (
            g, doubtful(g), bgate_group(g), owner(g), row_of.get(owner(g), {}).get("label", ""), len(xs), sum(nins[x] for x in xs),
            sum(nins[x] for x in region[g]), len(rs), named, " ; ".join(src_text(f) for f in sorted(bgates[g])[:2])))
never_blocks = [L for L in lead_sorted if L not in hit and L < 0xB0000 and nins[L]]
with open(os.path.join(OUT, "never_blocks.tsv"), "w") as fh:
    fh.write("leader\tins\troutine\troutine_ran\tgate\tgate_group\n")
    for L in never_blocks:
        g = owner_region.get(L)
        fh.write("%x\t%d\t%x\t%d\t%s\t%s\n" % (L, nins[L], owner(L), row_of.get(owner(L), {}).get("hit", False),
                                            "%x" % g if g else "-", bgate_group(g) if g else "-"))
bg = defaultdict(lambda: [0, 0, 0])
for g, xs in own.items():
    k = bgate_group(g)
    bg[k][0] += 1
    bg[k][1] += len(xs)
    bg[k][2] += sum(nins[x] for x in xs)
print("block gates by group (gates, never-run blocks, instructions): %s" % ", ".join(
    "%s %d/%d/%d" % ((k,) + tuple(v)) for k, v in sorted(bg.items(), key=lambda kv: -kv[1][2])))
print("block gates: %d; never-run blocks below 0xB0000: %d (%d instructions), behind a block gate %d (%d instructions)" % (
    len(bgates), len(never_blocks), sum(nins[x] for x in never_blocks), len(owner_region), sum(nins[x] for x in owner_region)))

# Summary.
E = [r for r in rows if r["addr"] in entry_set]
H = [r for r in rows if r["kind"] == "handler"]
PR = [r for r in rows if r["kind"] == "port"]
code_e = [r for r in E if r["code"]]
print("batches: %s" % " ".join(want))
print("jobs: %d, addresses hit: %d" % (len({j for js in jobs_of.values() for j in js}), len(hit)))
print("entries.txt: %d; decode as code: %d; do not decode (data): %d" % (len(E), len(code_e), len(E) - len(code_e)))
lo = [r for r in code_e if r["addr"] < 0xB0000]
hi = [r for r in code_e if r["addr"] >= 0xB0000]
print("  code entries below 0xB0000: %d, hit %d, never %d" % (len(lo), sum(r["hit"] for r in lo), sum(not r["hit"] for r in lo)))
print("  code entries from 0xB0000 (the data area): %d, hit %d" % (len(hi), sum(r["hit"] for r in hi)))
print("  all entries: hit %d, never %d" % (sum(r["hit"] for r in E), sum(not r["hit"] for r in E)))
print("class handlers (not in entries.txt): %d, hit %d" % (len(H), sum(r["hit"] for r in H)))
print("port routines (not in entries.txt, not handlers): %d, hit %d" % (len(PR), sum(r["hit"] for r in PR)))
CT = [r for r in rows if r["kind"] == "call"]
print("other call targets: %d, hit %d" % (len(CT), sum(r["hit"] for r in CT)))
allr = [r for r in rows if r["code"] and r["addr"] < 0xB0000]
print("all routines below 0xB0000 (entries, handlers, port routines, call targets): %d, hit %d, never %d" % (
    len(allr), sum(r["hit"] for r in allr), sum(not r["hit"] for r in allr)))
for pool in "AEBC":
    hs = [r for r in rows if any(c.startswith(pool) for c in handler_of.get(r["addr"], []))]
    print("  pool %s: %d handlers, hit %d" % (pool, len(hs), sum(r["hit"] for r in hs)))
bl_lo = [b for b in blocks if b[0] < 0xB0000 and b[2] > 0]
print("blocks below 0xB0000: %d, hit %d (%.1f%%); instructions %d, in hit blocks %d (%.1f%%)" % (
    len(bl_lo), sum(b[0] in hit for b in bl_lo), 100.0 * sum(b[0] in hit for b in bl_lo) / max(1, len(bl_lo)),
    sum(b[2] for b in bl_lo), sum(b[2] for b in bl_lo if b[0] in hit),
    100.0 * sum(b[2] for b in bl_lo if b[0] in hit) / max(1, sum(b[2] for b in bl_lo))))
lr = [r for r in rows if r["code"] and r["addr"] < 0xB0000]
c = Counter((r["hit"], r["port"]) for r in lr)
print("routines below 0xB0000 (entries + handlers) by hit and port: %s" % dict(sorted(c.items())))
partial = [r for r in lr if r["hit"] and r["bhit"] < r["blocks"]]
print("hit routines with blocks never run: %d (blocks never run in hit routines: %d, instructions %d)" % (
    len(partial), sum(r["blocks"] - r["bhit"] for r in partial), sum(r["ins"] - r["inshit"] for r in partial)))
