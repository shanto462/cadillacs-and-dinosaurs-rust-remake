# Decode item name tables (offset word tables, strings = attr byte + text + '@')
import sys, struct
img = open(sys.argv[1], "rb").read()
def s(a):
    e = img.index(b'@', a); return img[a], img[a+1:e].decode('latin1').rstrip()
def tab(base, n, label):
    print(f"{label} table {base:#x}")
    for i in range(n):
        off = struct.unpack(">h", img[base+2*i:base+2*i+2])[0]
        attr, t = s(base+off)
        print(f"  {i:2d} off={off:#06x} addr={base+off:#07x} attr={attr:#04x} {t!r}")
tab(0x23c58, 6, "weapon (index -$70a8(a5), printer 0x23C2E)")
tab(0x240ca, 20, "food (index -$708c(a5), printer 0x24078)")
tab(0x240a0, 21, "table 0x240A0 (printer ~0x24050)")
tab(0x24436, 16, "treasure (index -$7076(a5), printer 0x2440E)")
