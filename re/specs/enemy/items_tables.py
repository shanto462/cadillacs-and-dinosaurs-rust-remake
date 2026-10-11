# Item stat tables (pool E item classes). usage: items_tables.py dino_68k.bin
import sys, struct
img = open(sys.argv[1], "rb").read()
W = lambda a: struct.unpack(">H", img[a:a+2])[0]
def bcd(off): return int(img[0x10292+off:0x10296+off].hex())   # score table used by 0x10036
def name(base, i):
    a = base + struct.unpack(">h", img[base+2*i:base+2*i+2])[0]
    return img[a+1:img.index(b'@', a)].decode('latin1').strip()
print("FOOD (pool E class 4, handler 0x2898C; +$26 = type 1..18; table 0x28B10 = [heal word, score offset word])")
for t in range(18):
    heal, so = W(0x28b10+4*t), W(0x28b10+4*t+2)
    print(f"  +26={t+1:2d} {name(0x240a2, t):10s} heal={heal:3d} score_if_full={bcd(so)} (off {so:#x})")
print("TREASURE (pool E class 9, handler 0x2A028; +$26 = type 1..16; score offset bytes 0x2A164)")
for t in range(16):
    so = img[0x2a164+t]
    print(f"  +26={t+1:2d} {name(0x24436, t):28s} score={bcd(so)}")
print("WEAPON ammo refill table 0x2855A (8 bytes: max_ammo, kind(+$a0), +$a2, pad) by weapon type (+$26/2)")
for t in range(8):
    a = 0x2855a + 8*t
    print(f"  type {t}: max={W(a)} kind={W(a+2):#06x} a2={W(a+4):#x}")
print("Gun names 0x23C58 (index = kind&0xFF - 4):", [name(0x23c58, i) for i in range(6)])
print("Weapon break mask 0x2CB80 by wear (+$a6>>5):", [f"{struct.unpack('>I', img[0x2cb80+4*i:0x2cb84+4*i])[0]:08x}" for i in range(8)])
print("Gun-kind flag 0x2494A by kind&0xF:", list(img[0x2494a:0x2495a]))
print("Container thrown attack id 0x29860 by variant:", [hex(W(0x29860+2*i)) for i in range(7)])
print("Container fuse tables 0x28F24 (flash count) / 0x28F28 (interval):", list(img[0x28f24:0x28f28]), list(img[0x28f28:0x28f2c]))
