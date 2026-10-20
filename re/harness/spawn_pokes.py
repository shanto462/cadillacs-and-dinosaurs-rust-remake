"""Turn the `spawn` notes of a probe sidecar into RAM writes the port can replay (M6-01 / M6-02 harness helper).

A probe segment such as `5 spawn A 13 var=0x0002 x=+0x100 z=0x30` makes the object in MAME only (the sidecar `NAME.pokes.txt` has the comment
`# frame 5: spawn A 13 ...`). In `step` mode the port is loaded with MAME's state of the record before, so the object would appear already
initialised and its init (the frame it is made in) would never run in the port. This tool writes the same RAM changes as the probe's allocator
copy (probe.lua `do_spawn`): the free list pointer and count of the pool, `+$00`, `+$20`, `+$08`, `+$0C`, `+$10`, `+$26`, `+$5C`, `+$5B`, `+$96`
of the slot on top of the free list (read from the MAME record before the spawn), as lines of the sidecar format (`frame hexaddr size value`).

Usage: spawn_pokes.py MAME.trc SIDECAR OUT      (OUT = the sidecar with the writes added after its header; the pools A, B, C, D and E are handled)
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trace as T  # noqa: E402

G = 0xFF8000
# free list pointer / count words of the five pools (allocators 0x483C A, 0x4860 B, 0x4884 C, 0x48A8 D, 0x4818 E; probe.lua POOLS)
POOLS = {"A": {"ptr": G + 0x6602, "cnt": G + 0x65F8}, "B": {"ptr": G + 0x6600, "cnt": G + 0x65F6}, "E": {"ptr": G + 0x65FE, "cnt": G + 0x65F4},
         "C": {"ptr": G + 0x6604, "cnt": G + 0x65FA}, "D": {"ptr": G + 0x6606, "cnt": G + 0x65FC}}


def num(s):
    s = s.strip()
    return int(s, 16) if s.lower().startswith(("0x", "+0x", "-0x")) else int(s)


def main():
    trc, side, out = sys.argv[1:4]
    notes = []
    text = open(side).read().splitlines()
    for line in text:
        m = re.match(r"# frame (\d+): spawn (\w) (\d+)(.*?) \(MAME only", line)
        if m:
            notes.append((int(m.group(1)), m.group(2), int(m.group(3)), dict(kv.split("=", 1) for kv in m.group(4).split() if "=" in kv)))
    imgs = {}
    want = {k - 1 for k, _, _, _ in notes}
    if want:
        for rec in T.Trace(trc).records():
            if rec.frame in want:
                imgs[rec.frame] = bytes(rec.images["ram"])
            if rec.frame >= max(want):
                break
    lines = []
    taken = {}
    for k, pool, cls, o in notes:
        if pool.upper() not in POOLS:
            print(f"spawn_pokes: frame {k}: a spawn in pool {pool} is not replayed (unknown pool)", file=sys.stderr)
            continue
        P = POOLS[pool.upper()]
        ram = imgs[k - 1]
        r16 = lambda a: (ram[a - 0xFF0000] << 8) | ram[a - 0xFF0000 + 1]
        key = (k, pool.upper())
        ptr, cnt = r16(P["ptr"]) + 2 * taken.get(key, 0), r16(P["cnt"]) - taken.get(key, 0)
        taken[key] = taken.get(key, 0) + 1
        slot = 0xFF0000 + r16(0xFF0000 + ptr)
        xs = o.get("ax") or o.get("x") or "+0x120"
        rel = "ax" not in o and xs[0] in "+-"
        x = ((r16(G + 0x744) + num(xs)) if rel else num(xs)) & 0xFFFF
        z = num(o["z"]) if "z" in o else r16(0xFF0000 + 0xB274 + 0x10)
        w = [(P["ptr"], 2, ptr + 2), (P["cnt"], 2, cnt - 1), (slot, 1, 1), (slot + 0x20, 2, cls * 4), (slot + 8, 2, x),
             (slot + 0xC, 2, num(o["y"]) if "y" in o else 0), (slot + 0x10, 2, z), (slot + 0x26, 2, num(o["var"]) if "var" in o else 0),
             (slot + 0x5C, 2, num(o["w5c"]) if "w5c" in o else 0), (slot + 0x5B, 1, num(o["b5b"]) if "b5b" in o else 0),
             (slot + 0x96, 1, num(o.get("drop", o.get("b96", "0"))))]
        for a, s, v in w:
            lines.append(f"{k} {a:06X} {s} 0x{v & ((1 << 8 * s) - 1):X}")
    # insert after the `#!` header lines; the writes of one frame stay after the probe's own writes of that frame (same order of application)
    head = [l for l in text if l.startswith("#!")]
    rest = [l for l in text if not l.startswith("#!")]
    with open(out, "w") as fh:
        fh.write("\n".join(head + rest + lines) + "\n")


if __name__ == "__main__":
    main()
