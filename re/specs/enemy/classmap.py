# Map pool A classes (dispatch 0x2346C, index = +$20/4) to name ids (+$3 writes) inside each handler's code range
import sys, struct, re
img = open(sys.argv[1], "rb").read(); sweep = sys.argv[2]
L = lambda a: struct.unpack(">I", img[a:a+4])[0]
def name(i):
    if i > 0x26: return "?"
    off = struct.unpack(">h", img[0x17d78+2*i:0x17d7a+2*i])[0]; a = 0x17d78+off
    return img[a+1:img.index(b'@',a)].decode().strip()
cls = [(i, L(0x2346c+4*i)) for i in range(102)]
starts = sorted(set(a for _,a in cls if a))
lines = []
for line in open(sweep):
    try: a = int(line[:6],16)
    except: continue
    if 0x31000 <= a < 0x6e000: lines.append((a, line[8:].strip()))
pat = re.compile(r"move\.b\s+#\$([0-9a-f]+), \$3\(a6\)$")
pat2 = re.compile(r"jsr\s+\$(325e6|31424)\.l")
for i, a in cls:
    if not a: print(f"#{i:3d} +$20={4*i:#05x} (null)"); continue
    nxt = min([s for s in starts if s > a] + [0x6e000])
    ids = []; extra=set()
    for (x, t) in lines:
        if a <= x < nxt:
            m = pat.search(t)
            if m: ids.append(int(m.group(1),16))
            m2 = pat2.search(t)
            if m2: extra.add(m2.group(1))
    ids = sorted(set(ids))
    print(f"#{i:3d} +$20={4*i:#05x} {a:#07x}-{nxt:#07x} names={[f'{x:#x} {name(x)}' for x in ids]} {'HP' if '31424' in extra else ''} {'SCORE' if '325e6' in extra else ''}")
