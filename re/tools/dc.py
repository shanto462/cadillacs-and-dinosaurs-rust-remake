# Compact listing of a code range: only branch targets keep their address (as L<addr>), data tables are shown as
# one line of words. Usage: dc.py image start end [T:count ...]   (reads the dj.py output through a pipe)
import re, sys
lines = [l.rstrip("\n") for l in sys.stdin]
tg = set()
for l in lines:
    for m in re.finditer(r"\$([0-9a-f]{5,6})(?:\.l)?\b", l.split(":", 1)[1] if ":" in l else l):
        tg.add(int(m.group(1), 16))
    m = re.search(r"-> ([0-9a-f]{6})", l)
    if m:
        tg.add(int(m.group(1), 16))
out = []
tabrow = []
def flush():
    global tabrow
    if tabrow:
        out.append("   tbl " + " ".join(tabrow)); tabrow = []
for l in lines:
    a, rest = l.split(":", 1)
    addr = int(a, 16)
    rest = rest.strip()
    m = re.match(r"dc\.w\s+\$([0-9a-f]+)\s+-> ([0-9a-f]{6})", rest)
    if m:
        tabrow.append(f"L{m.group(2)}"); continue
    flush()
    rest = re.sub(r"\$([0-9a-f]{5,6})(?:\.l)?\b", lambda m: "L" + m.group(1), rest)
    rest = rest.replace(".l,", ",").replace("jsr      L", "jsr L").replace(", ", ",")
    rest = re.sub(r"\s+", " ", rest)
    lab = f"L{addr:06x}:" if addr in tg else ""
    out.append(f"{lab:9}{rest}")
flush()
print("\n".join(out))
