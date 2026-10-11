# Decode item drop tables used by 0x24B48 (containers, table 0xB64F2) and 0x24B36 (enemies, table 0xB6D12).
# Row = +$96 value (32 bytes), column = RNG(0x119C) & 0x1F. Byte code: 0 none, 0x0F special(0x24C3A),
# 1..0x13 weapon (pool E class 0, +$26=(c-1)*2), 0x14..0x27 food (pool E class 4, +$26=c-0x13),
# >=0x28 treasure (pool E class 9, +$26=c-0x27).
import sys, struct
from collections import Counter
img = open(sys.argv[1], "rb").read()
WEAP = ["RIFLE(0x205,6)","GUN(0x204,6)","w2(0x202,6)","UZI(0x206,48)","M-16A1(0x207,48)","SHOTGUN(0x208,6)","BAZOOKA(0x209,4)",
        "w7(0x203,10)","w8(0x202,6)","w9(0x203,6)","w10(0x203,10)","w11(0x203,10)","w12(0x201,10)","w13(0x201,10)","w14(0x201,10)"]
FOOD = ["BARBECUE","STEAK","LOBSTER","SUSHI","BURGER","HOTDOG","PIZZA","SALAD","CAKE","PAFFE","PUDDING","POTATO","GUM","CHOCOLATE","DONUT","COFFEE","CROISSANT","food17","food18","food19"]
TREA = ["GOLD","DIAMOND","THE CUP","EGG","GOLDDUST","DINOBONE","AMMONITE","PEARL","NECKLACE","RING","WATCH","RADIO","DIME","1000$","SUNGLASS","GROVE"]
def lab(c):
    if c == 0: return "-"
    if c == 0x0f: return "SPECIAL"
    if c < 0x14: return "W:" + (WEAP[c-1] if c-1 < len(WEAP) else f"w{c-1}?")
    if c < 0x28: return "F:" + (FOOD[c-0x14] if c-0x14 < len(FOOD) else f"f{c-0x14}?")
    return "T:" + (TREA[c-0x28] if c-0x28 < len(TREA) else f"t{c-0x28}?")
def dump(base, rows, title, first=0):
    print(f"## {title} base {base:#x}")
    for r in range(first, rows):
        row = img[base+32*r: base+32*r+32]
        cnt = Counter(row)
        parts = [f"{lab(c)} x{n}" for c, n in sorted(cnt.items())]
        print(f"  row {r:#04x}: " + ", ".join(parts))
rows = int(sys.argv[2], 0) if len(sys.argv) > 2 else 0x40
dump(0xb64f2, rows, "containers/props (0x24B48)", first=-1)
dump(0xb6d12, rows, "enemies on death (0x24B36)")
