import struct, collections, sys
S = sys.argv[1]
b = open(S + '/out/dino_68k.bin', 'rb').read()
def w(a): return struct.unpack('>H', b[a:a+2])[0]
def sw(a): return struct.unpack('>h', b[a:a+2])[0]
def l(a): return struct.unpack('>I', b[a:a+4])[0]
CT, NC = l(0xFFFF4), w(0xFFFF8)
LT, NL = l(0xFFFFA), w(0xFFFFE)
print(hex(CT), NC, hex(LT), NL)
fmt = collections.Counter(); blk = collections.Counter(); bad = 0; mism = 0
lay_n = {}
mn, mx = 1 << 30, 0
for i in range(NC):
    a = l(CT + 4*i); mn = min(mn, a); mx = max(mx, a)
    cnt, n, attr, li = w(a), w(a+2), w(a+4), w(a+6)
    if attr & 0x8000:
        fmt['alt'] += 1
        ents = [(w(a+8+4*k), w(a+10+4*k)) for k in range(n)]
    else:
        fmt['norm'] += 1
        ents = [(w(a+8+2*k), attr) for k in range(n)]
        blk[('norm', attr & 0xff00)] += 1
    tot = sum((((at >> 8) & 15) + 1) * ((at >> 12) + 1) for c, at in ents)
    if tot != cnt: mism += 1
    if li >= NL: bad += 1
print(fmt, 'count mismatch', mism, 'bad layout', bad, hex(mn), hex(mx))
print(sorted(blk.items(), key=lambda x: -x[1])[:10])
