# Dump player/combat tables from the 68k program image.  usage: python -I tables.py dino_68k.bin > tables.txt
import struct, sys
img = open(sys.argv[1], "rb").read()
def ub(a): return img[a]
def sb(a): return struct.unpack(">b", img[a:a+1])[0]
def uw(a): return struct.unpack(">H", img[a:a+2])[0]
def sw(a): return struct.unpack(">h", img[a:a+2])[0]
def ul(a): return struct.unpack(">I", img[a:a+4])[0]
CH = ["Jack", "Hannah", "Mustapha", "Mess"]
DIRS = ["U", "UR", "R", "DR", "D", "DL", "L", "UL"]
def hdr(t): print("\n== " + t)

hdr("select screen stars POWER/SPEED/SKILL @0xB444 (display only)")
for c in range(4): print(f"  {CH[c]:9s}", [ub(0xb444 + 3*c + i) for i in range(3)])

hdr("walk/run speed tables: ptr = long[0x5064 + 4*(class + char)], class 0 walk, 4 run(slow frames), 8 run(fast frames), 12 (unused?)")
hdr("  (heavy object 0x20A carried: ptr table 0x50A4, all classes -> same 4 tables)")
for t in range(20):
    a = 0x50e4 + 0x20*t
    tag = f"class {t//4*4:2d} {CH[t%4]}" if t < 16 else f"heavy  {CH[t%4]}"
    print(f"  {a:#07x} {tag:18s}", " ".join(f"{DIRS[i]}({sw(a+4*i)},{sw(a+4*i+2)})" for i in range(8)))

hdr("jump velocity tables (0x5364): ptr = long[0x53A2 + 4*(class+char)]; 9 entries x 6 words (vx,ax,vy,ay,vz,az), entry 0 = no direction")
for k in range(16):
    p = ul(0x53a2 + 4*k)
    print(f"  class {k//4*4:2d} {CH[k%4]:9s} @{p:#x}")
    for i in range(9):
        a = p + 12*i
        print("     %-4s" % (["none"] + DIRS)[i], " ".join(f"{n}={sw(a+2*j):6d}" for j, n in enumerate(["vx", "ax", "vy", "ay", "vz", "az"])))

hdr("air control deltas per frame (0x1BE66): (dvx, dvz) per direction")
print("  ", " ".join(f"{DIRS[i]}({sw(0x1be66+4*i)},{sw(0x1be68+4*i)})" for i in range(8)))

hdr("jump attack height thresholds (0x1A734) per char: (min height, high/low split)")
for c in range(4): print(f"  {CH[c]:9s} min={sw(0x1a734+4*c):#x} split={sw(0x1a736+4*c):#x}")
hdr("high jump attack (state 0x40A) velocities (0x1A994): (vx, vy) per char, ay=-0x20, ax=+-0x20")
for c in range(4): print(f"  {CH[c]:9s} vx={sw(0x1a994+4*c)} vy={sw(0x1a996+4*c)}")

hdr("dash/other velocity blocks 0x1BF06..0x1BF41 (words)")
print("  ", [hex(uw(0x1bf06 + 2*i)) for i in range(30)])
hdr("weapon 0x209 recoil vx per char (0x19CD8)")
print("  ", [hex(uw(0x19cd8 + 2*i)) for i in range(4)])

hdr("invulnerability frames $F9 (0x6C3A): index d0+char; d0=0 getup, 4 after throw, 8 after pickup, 12 after special")
for d0 in (0, 4, 8, 12): print(f"  d0={d0:2d}", [ub(0x6c3a + d0 + c) for c in range(4)])

hdr("pushback reactions (0x1D27C): (frames, vx, ax) for reaction 0E/14, 10/16, 12/18")
for i in range(3):
    a = 0x1d27c + 6*i; print(f"  frames={uw(a)} vx={sw(a+2):#x} ax={sw(a+4)}")
hdr("knockdown launch vectors (0x1D28E): (vx, ax, vy, ay)")
for name, a in [("type4 alive", 0x1d28e), ("type4 dead", 0x1d296), ("type6 alive", 0x1d29e), ("type6 dead", 0x1d2a6), ("burn alive", 0x1d2ae), ("burn dead", 0x1d2b6)]:
    print(f"  {name:12s} @{a:#x}", [sw(a+2*i) for i in range(4)])

hdr("lie-down time $E8 (0x1D1C4) and mash meter $BE (0x1D204): row = (rank & 0x18)>>3, col = rand&15")
for r in range(4): print("  E8 row", r, [ub(0x1d1c4 + 16*r + i) for i in range(16)])
for r in range(4): print("  BE row", r, [ub(0x1d204 + 16*r + i) for i in range(16)])
hdr("dizzy: stars $BE (0x1CF5C, row=(rank&0x18)>>3, col=rand&15) and max time $80 (0x1CEDC words, row, col=rand&15)")
for r in range(4): print("  BE row", r, [ub(0x1cf5c + 16*r + i) for i in range(16)])
for r in range(4): print("  80 row", r, [uw(0x1cedc + 32*r + 2*i) for i in range(16)])
hdr("dizzy chance masks (0x1D166) indexed by min(hits_in_60f,5): bit (rand&15) set => dizzy")
print("  ", [hex(uw(0x1d166 + 2*i)) for i in range(6)])

def atk(a):
    b = img[a:a+8]
    return f"dmg={b[0]:3d} stun={b[1]:3d} react={b[2]:#04x} stop={b[3]:2d} snd={b[4]:#04x} fx={b[5]:#04x} spark={b[6]:#04x} b7={b[7]:#04x}"
USED = {0x00: "jab/punch2", 0x02: "punch3", 0x04: "finisher", 0x06: "grab knee 1", 0x08: "grab knee 2+", 0x0a: "throw",
        0x0c: "jump attack low", 0x0e: "jump attack high (dive)", 0x10: "running-jump attack", 0x12: "special",
        0x14: "dash attack start", 0x16: "weapon run attack", 0x18: "escape attack (grabbed)", 0x1a: "attack in run state 4",
        0x5a: "dash attack hit 2", 0x5c: "dash attack hit 3+", 0x70: "air/command multi-hit", 0x72: "command move", 0x74: "command move",
        0x76: "command move / state 0x24", 0x78: "command move", 0x7a: "command move"}
hdr("player attack table: ptr = long[0xA26FC + $20]; entry = ptr + $58*4 (8 bytes)")
for c in range(4):
    p = ul(0xa26fc + 4*c)
    print(f"  {CH[c]} @{p:#x}")
    lim = [0x76, 0x7a, 0x76, 0x78][c]   # table length differs per char (bases 0xA270C/0xA28EC/0xA2ADC/0xA2CBC)
    for k in sorted(USED):
        if k > lim: print(f"    $58={k:#04x} (beyond this char's table)"); continue
        print(f"    $58={k:#04x} {USED[k]:24s} {atk(p + 4*k)}")
hdr("enemy attack table 0xA2EA4: entry = 0xA2EA4 + $58(enemy)*8 (first 64)")
for k in range(64): print(f"  $58={2*k:#04x}? idx {k:2d} {atk(0xa2ea4 + 8*k)}")

hdr("player power factor $C6 (0x2C9B6): row by player HP (>=75,>=50,>=25,<25), col rand&31")
for r in range(4): print("  ", [ub(0x2c9b6 + 32*r + i) for i in range(32)])
hdr("player defense per char (0x2C88E, x/32)")
print("  ", [uw(0x2c88e + 2*c) for c in range(4)])
hdr("enemy->player: HP band reduction (0x2C896 words), row by player HP (>=76,>=51,>=26,<26), col (rand&0x3e)/2")
for r in range(4): print("  ", [uw(0x2c896 + 64*r + 2*i) for i in range(32)])
hdr("enemy->player: HP scale (0x2C996 words) indexed by (HP>>2)&0x1e")
print("  ", [uw(0x2c996 + 2*i) for i in range(16)])
hdr("player->enemy: HP band scale 0x2BB4E words, row by enemy HP%100 band (>=75,>=50,>=25,<25), col (rand&0x3e)/2")
for r in range(4): print("  ", [uw(0x2bb4e + 64*r + 2*i) for i in range(32)])
hdr("player->enemy: 0x2BC4E words (row = number of 100s above 100 in enemy HP), first 4 rows")
for r in range(4): print("  ", [uw(0x2bc4e + 64*r + 2*i) for i in range(32)])
hdr("hitbox table ptr per char ($44 = long[0x18CC4 + $20])")
print("  ", [hex(ul(0x18cc4 + 4*c)) for c in range(4)])
hdr("animation set pointer tables (0x163E.. 6 sets x 4 chars)")
for s in range(6): print(f"  set {s} @{0x163e+16*s:#x}", [hex(ul(0x163e + 16*s + 4*c)) for c in range(4)])
