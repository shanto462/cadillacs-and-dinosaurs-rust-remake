# Speed tables used by 0x17A4 (d0 = speed id): ptr table 0x17CE, each table 32 dirs x (vx, vz) s16 (1/256 px/frame)
import sys, struct
img = open(sys.argv[1], "rb").read()
L = lambda a: struct.unpack(">I", img[a:a+4])[0]
n = int(sys.argv[2]) if len(sys.argv)>2 else 32
for i in range(n):
    p = L(0x17ce + 4*i)
    if p > len(img): break
    d = [struct.unpack(">hh", img[p+4*k:p+4*k+4]) for k in range(32)]
    print(f"id {i:#04x} @{p:#x}: dir0={d[0]} dir4={d[4]} dir8={d[8]} dir16={d[16]} dir24={d[24]}")
