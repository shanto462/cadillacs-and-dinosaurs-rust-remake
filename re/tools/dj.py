# Linear 68000 disassembly of a range with jump tables decoded: dj.py image start count
# A "jmp T(pc,dN.w)" whose T is the next address (or any later address given by the first
# "move.w T(pc,dN.w),dM") is followed by a table of words (offsets from T). The table length is
# first_entry / 2 words. Extra argument T:count declares a table by hand. The table is printed as "dc.w $xxxx -> target".
import sys, re, capstone
img = open(sys.argv[1], "rb").read()
start = int(sys.argv[2], 16); count = int(sys.argv[3])
md = capstone.Cs(capstone.CS_ARCH_M68K, capstone.CS_MODE_BIG_ENDIAN | capstone.CS_MODE_M68K_000)
pc_re = re.compile(r"\$([0-9a-f]+)\(pc, d\d\.w\)")
def w(a): return (img[a] << 8) | img[a + 1]
def l(a): return (w(a) << 16) | w(a + 2)
ltab_re = re.compile(r"\$([0-9a-f]+)\(pc, d\d\.w\), a\d")
# Function starts known from the Ghidra index (git-ignored file; optional): a table ends before one.
import os
FUNCS = set()
_idx = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "ghidra", "decomp.c.index.tsv")
if os.path.exists(_idx):
    for _ln in open(_idx):
        FUNCS.add(int(_ln.split("\t")[0], 16))
def table_len(t, first_words):
    """Entries of a word table at t: stop at the next function start, at an entry that points into the
    table or far away, or after first_words entries."""
    nxt = min([a for a in FUNCS if a > t] or [t + 0x1000])
    k = 0
    while k < first_words and t + 2 * k < nxt:
        off = w(t + 2 * k)
        off = off - 0x10000 if off & 0x8000 else off
        tgt = t + off
        if tgt < t + 2 * (k + 1) or tgt > t + 0x6000 or tgt & 1:
            break
        k += 1
    return max(k, 1)
addr = start; n = 0
tables = {}
longtabs = {}
for a in sys.argv[4:]:  # extra tables: T:count (word table at hex T, offsets from T)
    t, k = a.split(':'); tables[int(t, 16)] = (int(t, 16), int(k))
while n < count:
    if addr in longtabs:
        k = longtabs[addr]
        for i in range(k):
            print(f"{addr + 4 * i:06x}: dc.l     ${l(addr + 4 * i):08x}")
        addr += 4 * k; n += 1
        continue
    if addr in tables:
        t, k = tables[addr]
        for i in range(k):
            off = w(addr + 2 * i); off = off - 0x10000 if off & 0x8000 else off
            print(f"{addr + 2 * i:06x}: dc.w     ${w(addr + 2 * i):04x}  -> {t + off:06x}")
        addr += 2 * k; n += 1
        continue
    ins = next(md.disasm(img[addr:addr + 16], addr), None)
    if ins is None:
        print(f"{addr:06x}: dc.w     ${w(addr):04x}"); addr += 2; n += 1; continue
    print(f"{ins.address:06x}: {ins.mnemonic:8s} {ins.op_str}")
    m = pc_re.search(ins.op_str)
    if m and ins.mnemonic in ("jmp", "jsr"):
        t = int(m.group(1), 16)
        first = w(t)
        k = table_len(t, 0x100)
        tables[t] = (t, k)
    m2 = ltab_re.search(ins.op_str)
    if m2 and ins.mnemonic in ("movea.l", "move.l"):
        t = int(m2.group(1), 16)
        first = l(t)
        longtabs[t] = ((first - t) // 4) if t < first <= t + 0x100 else 1
    addr += ins.size; n += 1
