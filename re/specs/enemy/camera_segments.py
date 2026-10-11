# Camera segment table (0x21B2A: word offsets per stage). 16-byte records, loaded by 0x209C0 into the
# camera struct at $73C(a5): +0 -> +4C lock/trigger X, +2 -> +4E Y, +4 -> +50 lead, +6 -> +52, +8 -> +54,
# +A -> +4A (area), +B -> +5 (camera mode), +C -> +4B, +D -> +70, +E -> +7B (mode after GO), +F -> +79 (vertical)
import sys, struct
img = open(sys.argv[1], "rb").read()
st = int(sys.argv[2]) if len(sys.argv) > 2 else 0
base = 0x21B2A
a = base + struct.unpack(">h", img[base+2*st:base+2*st+2])[0]
print(f"stage {st+1} camera segments @ {a:#x}")
for k in range(40):
    r = img[a+16*k:a+16*k+16]
    x, y, lead, w6, w8 = struct.unpack(">5H", r[:10])
    print(f"  {k:2d} @{a+16*k:#x}: X={x:#06x} Y={y:#06x} lead={lead:#06x} +6={w6:#06x} +8={w8:#06x} area={r[10]} mode={r[11]:#04x} c={r[12]} d={r[13]} next_mode={r[14]:#04x} vert={r[15]}")
    if x == 0xffff: break
