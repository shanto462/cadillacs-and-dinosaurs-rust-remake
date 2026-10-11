# Recursive-descent 68000 disassembly from vectors + task entries; writes listing + call graph info
import sys, struct, capstone, re, json
img = open(sys.argv[1], "rb").read(); out = sys.argv[2]
md = capstone.Cs(capstone.CS_ARCH_M68K, capstone.CS_MODE_BIG_ENDIAN | capstone.CS_MODE_M68K_000)
seen = {}; work = []; funcs = set(); calls = {}
def add(a, f=False):
    if 0 <= a < len(img) and a % 2 == 0:
        if f: funcs.add(a)
        if a not in seen: work.append(a)
for i in range(1, 64):
    v = struct.unpack(">I", img[i*4:i*4+4])[0]
    if 0x40 <= v < 0x1000 and i not in range(12, 24) and i < 48: add(v, True)
add(0x41a, True)
abs_re = re.compile(r"^\$([0-9a-f]+)(\.l|\.w)?$")
while work:
    a = work.pop()
    while a not in seen and a < len(img):
        ins = next(md.disasm(img[a:a+16], a), None)
        if ins is None: break
        seen[a] = (ins.size, ins.mnemonic, ins.op_str)
        m, op = ins.mnemonic, ins.op_str
        nxt = a + ins.size
        tgt = None
        mm = abs_re.match(op.split(",")[0].strip()) if op else None
        if mm: tgt = int(mm.group(1), 16)
        if m in ("bsr.w", "bsr.b", "bsr.s", "jsr", "bsr"):
            if tgt is not None: add(tgt, True); calls.setdefault(tgt, []).append(a)
        elif m.startswith("b") and m[1:3] in ("ra",) or m == "jmp":
            if tgt is not None: add(tgt)
            break
        elif m.startswith("b") and len(m) >= 3 and m not in ("bset", "bclr", "btst", "bchg") and not m.startswith(("bset", "bclr", "btst", "bchg")):
            if tgt is not None: add(tgt)
        elif m.startswith("db"):
            parts = [p.strip() for p in op.split(",")]
            mm = abs_re.match(parts[-1]) if parts else None
            if mm: add(int(mm.group(1), 16))
        elif m in ("rts", "rte", "rtr", "illegal"):
            break
        # task entry: lea $xxxxx.l, a0 followed later by trap #0 via bsr $8ce -> treat lea targets in ROM as possible code
        if m == "lea.l" and op.endswith("a0"):
            mm = abs_re.match(op.split(",")[0].strip())
            if mm:
                t = int(mm.group(1), 16)
                if t < len(img): pass
        a = nxt
with open(out + "/rd_listing.txt", "w") as f:
    for a in sorted(seen):
        s, m, o = seen[a]
        f.write(f"{a:06x}: {m:8s} {o}\n")
json.dump({"funcs": sorted(funcs), "calls": {str(k): v for k, v in calls.items()}}, open(out + "/rd_graph.json", "w"))
code_bytes = sum(v[0] for v in seen.values())
print("instructions", len(seen), "code bytes", hex(code_bytes), "functions", len(funcs))
