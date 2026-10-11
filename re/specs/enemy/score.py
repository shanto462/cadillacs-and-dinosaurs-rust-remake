# Kill score: 0x325E6 -> byte offset table 0x325F8[name_id] -> BCD long at 0x10292+off, added to player +$100 (BCD)
import sys, struct
img = open(sys.argv[1], "rb").read()
def name(i):
    off = struct.unpack(">h", img[0x17d78+2*i:0x17d7a+2*i])[0]; a = 0x17d78+off
    return img[a+1:img.index(b'@',a)].decode().strip()
for i in range(0x27):
    o = img[0x325f8+i]; v = img[0x10292+o:0x10296+o].hex()
    print(f"{i:#04x} {name(i):10s} off={o:#04x} score={int(v):d}")
