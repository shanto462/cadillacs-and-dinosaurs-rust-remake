# Collect likely function entry points from the linear sweep: call targets, task entries, vectors
import re, sys, struct
img = open(sys.argv[1], "rb").read(); lines = open(sys.argv[2]).read().splitlines()
ent = set([0x41a, 0x542])
for i in range(2, 48):
    v = struct.unpack(">I", img[i*4:i*4+4])[0]
    if 0x40 <= v < 0x1000 and not (12 <= i < 24): ent.add(v)
call = re.compile(r"^([0-9a-f]{6}): (jsr|bsr\.w|bsr\.b|jmp) +\$([0-9a-f]+)(\.l)?$")
lea = re.compile(r"^([0-9a-f]{6}): (lea\.l|movea\.l) +#?\$([0-9a-f]+)(\.l)?, a0$")
for idx, l in enumerate(lines):
    m = call.match(l)
    if m:
        t = int(m.group(3), 16)
        if t < len(img) and t % 2 == 0: ent.add(t)
        continue
    m = lea.match(l)
    if m:
        t = int(m.group(3), 16)
        nxt = " ".join(lines[idx+1:idx+4])
        if t < len(img) and t % 2 == 0 and re.search(r"\$8ce|\$a16|trap +#\$0|trap +#\$7", nxt): ent.add(t)
open(sys.argv[3], "w").write("\n".join("%x" % e for e in sorted(ent)) + "\n")
print(len(ent), "entries")
