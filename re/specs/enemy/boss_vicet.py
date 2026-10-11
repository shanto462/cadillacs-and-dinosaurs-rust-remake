# Vice T. (pool A class 18, 0x42590) combo/attack tables. usage: boss_vicet.py <bin> <sweep>
import sys, struct, re
img = open(sys.argv[1], "rb").read()
lines = {}
for l in open(sys.argv[2]):
    try: a = int(l[:6], 16)
    except: continue
    if 0x42590 <= a < 0x45082: lines[a] = l[8:].strip()
addrs = sorted(lines)
def scan(start, n=60):
    out = []; i = addrs.index(start) if start in lines else None
    if i is None: return "?"
    for a in addrs[i:i+n]:
        t = lines[a]
        m = re.search(r"move\.b +#\$([0-9a-f]+), \$58\(a6\)", t)
        if m: out.append(f"atk={int(m.group(1),16):#x}")
        m = re.search(r"jsr +\$(120e|17a4|a63c|4884|483c|4860)\.l", t)
        if m: out.append({"120e":"anim","17a4":"speed","a63c":"snd","4884":"spawnC","483c":"spawnA","4860":"spawnB"}[m.group(1)] + "(" + (prev or "") + ")")
        mm = re.match(r"moveq +#\$([0-9a-f]+), d0", t); prev = mm.group(1) if mm else None
        if t.startswith("rts") or t.startswith("jmp      $4"): break
    return " ".join(out)
prev = None
print("attack-combo choice 0x44C8C (rand&31; +32 for Walther):", list(img[0x44c8c:0x44c8c+32]), list(img[0x44cac:0x44cac+32]))
acts = 0x4306c
nact = 16
offs = [struct.unpack(">h", img[acts+2*k:acts+2*k+2])[0] for k in range(nact)]
for c in range(13):
    b = img[0x43004+8*c:0x4300c+8*c]
    seq = []
    for x in b:
        if x == 0xff: break
        seq.append(x)
    print(f"combo {c:2d} (+$b8={8*c:#x}): bytes {b.hex(' ')} -> actions {[s//2 for s in seq]}")
for k in range(nact):
    t = acts + offs[k]
    print(f"action {k:2d} (byte {2*k:#x}) -> {t:#x}: {scan(t)}")
