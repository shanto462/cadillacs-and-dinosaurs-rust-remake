import sys, struct
img = open(sys.argv[1], "rb").read()
base = int(sys.argv[2], 16); n = int(sys.argv[3]); sbase = int(sys.argv[4],16) if len(sys.argv)>4 else base
def s(a):
    e = img.index(b'@', a); return img[a:e]
for i in range(n):
    off = struct.unpack(">h", img[base+2*i:base+2*i+2])[0]
    a = sbase + off
    b = s(a)
    print(f"{i:3d} {i:#04x} off={off & 0xffff:#06x} addr={a:#07x} attr={b[0]:#04x} {b[1:].decode('latin1').rstrip()!r}" if len(b)<40 else f"{i:3d} off={off:#x} ??")
