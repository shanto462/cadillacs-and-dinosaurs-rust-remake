import sys, struct
img = open(sys.argv[1], "rb").read()
a = int(sys.argv[2],16); n = int(sys.argv[3]); w = int(sys.argv[4]) if len(sys.argv)>4 else 4
for i in range(n):
    if w==4: v = struct.unpack(">I", img[a+4*i:a+4*i+4])[0]; print(f"{i:3d} +${4*i:03x} {v:#08x}")
    else: v = struct.unpack(">h", img[a+2*i:a+2*i+2])[0]; print(f"{i:3d} {v & 0xffff:#06x} -> {a+v:#07x}")
