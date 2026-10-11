# hex helper: hx.py <addr> <count> [b|w|l|jt <base>]
#   b/w/l : dump bytes/words/longs; jt : word offset table relative to base (default addr); prints targets
import sys, struct
img = open("/Volumes/Shanto/Repos/Personal/Github/roms/re/out/dino_68k.bin", "rb").read()
a = int(sys.argv[1], 16); n = int(sys.argv[2]); k = sys.argv[3] if len(sys.argv) > 3 else "w"
if k == "b":
    for i in range(0, n, 16):
        row = img[a+i:a+min(i+16, n)]
        print(f"{a+i:06x}: " + " ".join(f"{x:02x}" for x in row))
elif k == "w":
    for i in range(0, n, 8):
        ws = [struct.unpack(">H", img[a+2*j:a+2*j+2])[0] for j in range(i, min(i+8, n))]
        print(f"{a+2*i:06x}: " + " ".join(f"{x:04x}" for x in ws))
elif k == "l":
    for i in range(n):
        print(f"{a+4*i:06x}: {struct.unpack('>I', img[a+4*i:a+4*i+4])[0]:08x}")
elif k == "jt":
    base = int(sys.argv[4], 16) if len(sys.argv) > 4 else a
    for i in range(n):
        o = struct.unpack(">h", img[a+2*i:a+2*i+2])[0]
        print(f"[{i*2:02x}] -> {base+o:06x}")
