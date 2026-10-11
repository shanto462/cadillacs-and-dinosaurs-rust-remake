# strtab.py base count [attr]: print '@'-terminated strings of a word-offset table (offsets from base)
import sys
img = open('/Volumes/Shanto/Repos/Personal/Github/roms/re/out/dino_68k.bin','rb').read()
base = int(sys.argv[1], 16); n = int(sys.argv[2]); attr = len(sys.argv) > 3
for i in range(n):
    off = int.from_bytes(img[base+2*i:base+2*i+2], 'big')
    p = base + off
    a = None
    if attr: a = img[p]; p += 1
    s = bytearray()
    while img[p] != 0x40 and len(s) < 40: s.append(img[p]); p += 1
    print(f"{i:2d} @{base+off:06x} attr={a if a is None else hex(a)} {s.decode('latin1')!r}")
