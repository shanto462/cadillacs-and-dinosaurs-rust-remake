import struct, sys
S = sys.argv[1]
b = open(S + '/out/dino_68k.bin', 'rb').read()
def w(a): return struct.unpack('>H', b[a:a+2])[0]
def sw(a): return struct.unpack('>h', b[a:a+2])[0]
def l(a): return struct.unpack('>I', b[a:a+4])[0]
def anim(start, rec=12, maxf=80):
    out = []; a = start; seen = set()
    while len(out) < maxf:
        if a in seen: break
        seen.add(a)
        r = b[a:a+rec]
        out.append((a, r))
        if r[7] & 0x80:
            loop = a + rec + sw(a + rec)
            return out, loop
        a += rec
    return out, None
def table_ids(tbl):
    first = w(tbl); n = first // 2
    return [tbl + w(tbl + 2*i) for i in range(n)]
if __name__ == '__main__':
    tbl = int(sys.argv[2], 16); rec = int(sys.argv[3]) if len(sys.argv) > 3 else 12
    for i, st in enumerate(table_ids(tbl)):
        fr, loop = anim(st, rec)
        print(f'anim {i:2d} @{st:06x} loop={"%06x"%loop if loop else None} n={len(fr)}')
        for a, r in fr:
            words = struct.unpack('>' + 'H'*(rec//2), r)
            print('   %06x ' % a + ' '.join('%04x' % x for x in words))
