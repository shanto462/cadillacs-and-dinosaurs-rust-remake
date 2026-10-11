# Enemy HP table at 0x31454: [name_id][level 0..31] words (routine 0x31424)
import sys, struct
img = open(sys.argv[1], "rb").read()
def name(i):
    off = struct.unpack(">h", img[0x17d78+2*i:0x17d7a+2*i])[0]; a = 0x17d78+off
    return img[a+1:img.index(b'@',a)].decode().strip()
for i in range(0x27):
    v = struct.unpack(">32H", img[0x31454+64*i:0x31454+64*i+64])
    print(f"{i:#04x} {name(i):10s} " + " ".join(f"{x:3d}" for x in v))
