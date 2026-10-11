import os, sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import z80dis as z
D = z.DATA
def w(a): return D[a] | D[a+1] << 8
TABLES = [(0x269, 8), (0x5a8, 8), (0x796, 12), (0xc84, 32), (0x101d, 8), (0x11ff, 6), (0x2211, 8)]
EXTRA = [0x099d]
ents = [0, 0x38, 0x66] + EXTRA
for t, n in TABLES:
    ents += [w(t + 2*i) for i in range(n)]
seen, labels = z.walk(ents)
if __name__ == "__main__":
    z.listing(seen, labels)
    # gaps
    a = 0; gaps = []
    cov = set()
    for s,(n,t) in seen.items():
        cov.update(range(s, s+n))
    start=None
    for a in range(0, 0x2b00):
        if a not in cov and start is None: start=a
        if a in cov and start is not None:
            gaps.append((start,a)); start=None
    print("GAPS", file=sys.stderr)
    for s,e in gaps:
        if e-s>=1: print(f"{s:04x}-{e:04x} ({e-s})", file=sys.stderr)
