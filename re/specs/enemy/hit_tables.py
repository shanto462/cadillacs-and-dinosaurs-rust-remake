# Dump tables used by the generic enemy hit reaction / damage code. usage: hit_tables.py dino_68k.bin
import sys, struct
img = open(sys.argv[1], "rb").read()
W = lambda a: struct.unpack(">H", img[a:a+2])[0]
S = lambda a: struct.unpack(">h", img[a:a+2])[0]
L = lambda a: struct.unpack(">I", img[a:a+4])[0]
def name(i):
    if i > 0x26: return "?"
    off = S(0x17d78+2*i); a = 0x17d78+off
    return img[a+1:img.index(b'@',a)].decode().strip()
def knock(a, n, label):
    print(label)
    for k in range(n):
        print(f"  {a+8*k:#x}: vx={S(a+8*k):6d} ax={S(a+8*k+2):5d} vy={S(a+8*k+4):6d} ay={S(a+8*k+6):5d}")
knock(0x2cbb0, 6, "Knockback records 0x2CBB0.. (8 bytes: vx, ax, vy, ay; vx/ax negated when +$24 facing!=0; bounce uses (vy, 2*ay) of the +4 offset)")
knock(0x2cb46, 3, "Throw (type 8) per thrower char +$3 (0x2CB46 + char*8)")
knock(0x2cb5e, 2, "Throw char 3 special: 0x2CB5E (+$2C==2), 0x2CB66")
print("Slide records 0x2CB6E (6 bytes: duration, vx, ax) for types 0xE/0x14, 0x10/0x16, 0x12/0x18")
for k in range(3):
    a = 0x2cb6e + 6*k; print(f"  {a:#x}: dur={S(a)} vx={S(a+2)} ax={S(a+4)}")
print("Lie-down time 0x2B614: 16 bytes per +$84/4 (name id), index rand&15; plus signed 0x2B604[rand&15]")
print("  0x2B604:", [struct.unpack('b', img[0x2b604+i:0x2b605+i])[0] for i in range(16)])
for i in range(0x27):
    print(f"  {i:#04x} {name(i):10s}", list(img[0x2b614+16*i:0x2b614+16*i+16]))
print("Counter delay +$7A 0x2B8C4: index ((level*2)&0x30) + rand&15")
for b in range(4): print("  band", b, list(img[0x2b8c4+16*b:0x2b8c4+16*b+16]))
print("Dizzy masks: HP band 0x2B158", [hex(L(0x2b158+4*i)) for i in range(4)])
print("  alive-count 0x2B168 (min(count,5))", [hex(L(0x2b168+4*i)) for i in range(6)])
for r in range(4):
    print(f"  +$9A row {r} 0x2B180+{32*r:#x} by aggression $5EC 0..7:", [hex(L(0x2b180+32*r+4*i)) for i in range(8)])
print("  special-class mask 0x2B200", hex(L(0x2b200)))
print("  early dizzy recover by level 0x2B28C", [hex(L(0x2b28c+4*i)) for i in range(32)])
print("Resume-AI routine per +$84 (0x2B56C):")
for i in range(0x27): print(f"  {i:#04x} {name(i):10s} {L(0x2b56c+4*i):#x}")
print("Damage anim tables 0x2E68C per +$84 (ptr -> word offsets, anim index d0):")
for i in range(0x27):
    p = L(0x2e68c+4*i); print(f"  {i:#04x} {name(i):10s} {p:#x}")
print("Counter-hit multiplier 0xA320C by ENEMY's current +$58 (word, /32; 0 = none):")
print("  ", [W(0xa320c+2*i) for i in range(0x78)])
print("Per-enemy weak attacks 0xA32FA (+$84 offset: [atk/2, mult, atk/2, mult], mult/32):")
for i in range(0x27): print(f"  {i:#04x} {name(i):10s}", list(img[0xa32fa+4*i:0xa32fa+4*i+4]))
print("Enemy defence by level 0x2BECE (+$84*8 + level; bytes; dmg*=v then >>10 together with player +$C6)")
for i in range(0x27): print(f"  {i:#04x} {name(i):10s}", list(img[0x2bece+32*i:0x2bece+32*i+32]))
print("HP-bar factor 0x2BC4E (words; index bar*0x40 + rand&0x3E)")
for b in range(4): print("  bar", b, [W(0x2bc4e+0x40*b+2*i) for i in range(32)])
print("In-bar HP factor 0x2BB4E (words; band >=75:0, >=50:0x40, >=25:0x80, else 0xC0; + rand&0x3E)")
for b in range(4): print("  band", b, [W(0x2bb4e+0x40*b+2*i) for i in range(32)])
print("Player attack tables 0xA26FC (per player char +$3; record = table + (+$58)*4, 8 bytes):")
for c in range(4):
    p = L(0xa26fc+4*c); print(f"  char {c}: {p:#x}")
    for k in range(0, 0x30, 2):
        r = img[p+4*k:p+4*k+8]
        print(f"    +$58={k:#04x}: dmg={r[0]} poise={r[1]} react={r[2]:#x} stop={r[3]} snd={r[4]:#x} hitscore_off={r[5]:#x} spark={r[6]:#x} 9a={r[7]:#x}")
