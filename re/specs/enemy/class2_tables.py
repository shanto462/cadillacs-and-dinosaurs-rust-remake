# Decode all data tables used by pool A class 2 (FERRIS/GNEISS/DRIVER, handler 0x3383A)
import sys, struct
img = open(sys.argv[1], "rb").read()
B = lambda a, n: list(img[a:a+n])
H = lambda a, n: list(struct.unpack(f">{n}h", img[a:a+2*n]))
V = ["FERRIS(var 0)", "GNEISS(var 2)", "DRIVER(var 4)"]
def spd(i):
    p = struct.unpack(">I", img[0x17ce+4*i:0x17d2+4*i])[0]
    vx = struct.unpack(">h", img[p+32:p+34])[0]; vz = struct.unpack(">h", img[p+2:p+4])[0]
    return f"{i:#04x}(vx {vx/256:.2f},vz {vz/256:.2f})"
print("Variant init (0x3387E): name id +$3, +$84, anim table +$40:")
for v,(n,a84,anim) in enumerate([(0,0,0x34892),(1,4,0x34B90),(0x1a,0x68,0x34E7C)]): print(f"  {V[v]}: +$3={n:#x} +$84={a84:#x} +$40={anim:#x}")
print("T_33FA4 ambush threshold by aggression (rand&15 < t):", B(0x33fa4, 8))
print("T_34186 (byte table): [aggr&7] -> +$a0 reload (state 6, no visible effect); [level&31] -> +$b8 pause at slot:", B(0x34186, 32))
print("T_34210 state-E pause by aggression:", B(0x34210, 8))
print("T_353B6 attack chance /32 at slot [variant][aggr]:")
for v in range(3): print("  ", V[v], B(0x353b6 + 8*v, 8))
print("T_354B2 reposition choice [variant][rand&31] (0 rotate slot, 1 MAAI, 2 back off 50f):")
for v in range(3): print("  ", V[v], B(0x354b2 + 32*v, 32))
print("T_3546A reposition duration +$80 [rand&15]:", B(0x3546a, 16))
print("T_34338 combo choice [variant][rand&31]:")
for v in range(3): print("  ", V[v], B(0x34338 + 32*v, 32))
names = {0:"A(anim3,atk0)",2:"B(anim4,atk1)",4:"C(anim5,atk2)",6:"J(jump anim8,atk3)"}
print("Combos 0x343E8 (6 bytes each, 0xFF ends):")
for c in range(8):
    seq = []
    for b in img[0x343e8+6*c:0x343e8+6*c+6]:
        if b == 0xff: break
        seq.append(names[b])
    print(f"  combo {c}: {' '.join(seq)}")
print("T_345D0 recovery index [rand&31]:", B(0x345d0, 32))
print("T_345F0 recovery frames [($5ee&0x18) + index]:")
for b in range(4): print("   band", b, B(0x345f0 + 8*b, 8))
print("SLOT_3524C approach slots (dx,dz):", [tuple(H(0x3524c+4*k,2)) for k in range(6)], " idx6..9 raw:", [tuple(H(0x3524c+4*k,2)) for k in range(6,10)])
print("ATK_352F8 attack positions (dx,dz):", [tuple(H(0x352f8+4*k,2)) for k in range(6)])
print("T_3533C side offset [rand&15] (words):", H(0x3533c, 16))
print("T_35796 dash-attack chance /32 [variant][level]:")
for v in range(3): print("  ", V[v], B(0x35796 + 32*v, 32))
print("Walk speed ids 0x35832 [variant][aggr]:")
for v in range(3): print("  ", V[v], " ".join(spd(x) for x in H(0x35832 + 16*v, 8)))
print("Run speed ids 0x35882 [variant][aggr]:")
for v in range(3): print("  ", V[v], " ".join(spd(x) for x in H(0x35882 + 16*v, 8)))
print("Entrance-4 idle timer table 0x33B28 [rand&0x3E]/2:", H(0x33b28, 32))
for a in (0x2a8, 0x2a9, 0x2aa, 0x2ab): pass
print("Attack ids used by class 2: 0x00 0x01 0x02 (punch A/B/C), 0x03 (jump attack), 0x0C (dash attack); see damage_tables.txt")
