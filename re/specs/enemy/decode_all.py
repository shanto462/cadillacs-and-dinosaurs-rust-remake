# Decode all stage spawn lists (interpreter 0xFBF4) and wave scripts (wave controller pool B class 15 @0x7B336)
import sys, struct
img = open(sys.argv[1], "rb").read()
W = lambda a: struct.unpack(">H", img[a:a+2])[0]
S16 = lambda a: struct.unpack(">h", img[a:a+2])[0]
L = lambda a: struct.unpack(">I", img[a:a+4])[0]
ANAME = {2:"FERRIS/GNEISS/DRIVER",3:"LASH T.",4:"TYROG(dino)",6:"BLUDGE",10:"HAMMER/BLK ELMER/WRENCH",13:"POACHER J/SKINNER/GUTTER",
 14:"BLADE/RAZOR",18:"VICE T./WALTHER",20:"PUNK/THUG",24:"SHIVAT(dino)",25:"SLICE",29:"R HOPPER",31:"HOGG",34:"KNIFE",36:"MORGAN",
 38:"MACK",40:"MORGUE",55:"FERRIS/GNEISS(alt)",61:"BUTCHER",63:"BLADE(alt)",64:"PUNK/WRENCH T.(alt)",71:"SLISAUR(dino)",72:"TYROG(alt)",
 79:"HAMMER/ELMER/WRENCH(alt)",82:"ZEKE",84:"FESSENDEN",91:"TYROG 2",94:"FESSENDEN(final)"}
POOL={0:"A",1:"B",2:"-",3:"E"}
CMD={0:"SKIP-NEXT-LIST1(0xFD74)",4:"REROLL-AGGRESSION(0x182A2)",8:"FORCE-AGGRESSION($5f2=X,$5f4=C)",0xc:"SET $4e8",0x10:"CALL 0x104A8"}
def desc(f, c):
    p = f & 7
    if f & 0x80: return f"CMD {CMD.get(c, hex(c))}"
    if p == 0: return f"A#{c//4:<3d}{ANAME.get(c//4,'')}"
    return f"{POOL.get(p,'?')}#{c//4}"
def cond(f):
    r=[]
    if f & 0x60 == 0x40: r.append("players>=2")
    if f & 0x60 == 0x60: r.append("players==3")
    if f & 0x60 == 0x20: r.append("cond20?")
    if f & 0x18 == 0x08: r.append(f"level>={f>>8}")
    elif f & 0x18: r.append(f"level<={f>>8}")
    return " ".join(r)
def rec16(a, first="trig"):
    t,f,c,x,z,v26,v5c = [W(a+2*k) for k in range(7)]
    if f & 0x80: return f"{first}={t:#06x} {desc(f,c)} arg6={x:#x} argC={v5c:#x}"
    return f"{first}={t:#06x} {desc(f,c):34s} X={x:#06x} Z={z:#06x} var={v26:#06x} +5c={v5c:#x} +5b={img[a+14]:#04x} item(+96)={img[a+15]:#04x} {cond(f)}"
nst = 8
for st in range(nst):
    print(f"################ STAGE {st+1} (index {st})")
    t1 = L(0xfc2e+4*st); t2 = L(0xfc4e+4*st)
    seen=set()
    for ar in range(4):
        p1 = L(t1+4*ar); p2 = L(t2+4*ar)
        if p1 > 0x180000 or p2 > 0x180000: break
        if (p1,p2) in seen: print(f"== area {ar}: same lists as an earlier area"); continue
        seen.add((p1,p2))
        print(f"== area {ar}: list1 {p1:#x} (14-byte records), list2 {p2:#x} (16-byte records)")
        a = p1
        for i in range(200):
            t,f,c,x,z,v26,y = [W(a+2*k) for k in range(7)]
            if t >= 0x9fff: print(f"  L1 {a:#x} END (sentinel trig {t:#06x})"); break
            if f & 0x80: print(f"  L1 {a:#x} trig={t:#06x} CMD {CMD.get(c,hex(c))} arg6={x:#x} argC={y:#x}")
            else: print(f"  L1 {a:#x} trig={t:#06x} {desc(f,c):12s} X={x:#06x} Z={z:#06x} var={v26:#06x} Y={y:#06x}")
            a += 14
        a = p2
        for i in range(200):
            if W(a) >= 0x9fff: print(f"  L2 {a:#x} END (sentinel trig {W(a):#06x})"); break
            print(f"  L2 {a:#x} {rec16(a)}"); a += 16
    base = L(0x7b37c + 4*st)
    nw = W(base)//2
    print(f"== waves: table {base:#x}, {nw} waves")
    for w in range(nw):
        a = base + S16(base + 2*w)
        print(f" wave {w} @ {a:#x}")
        for g in range(40):
            thr = [W(a), W(a+2), W(a+4)]; a += 6
            if S16(a) < 0:
                print(f"   wait alive<={thr} -> END type {W(a)&0xe} ({'GO arrow' if W(a)&0xe==0 else 'special'})"); break
            print(f"   wait alive<={thr} then:")
            while S16(a) >= 0:
                print(f"     {rec16(a,'delay')}"); a += 16
            a += 16
