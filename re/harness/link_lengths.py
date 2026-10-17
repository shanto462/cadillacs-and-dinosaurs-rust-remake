"""M3-29: the length of the link scenes in MAME and in the port (the cut recordings m228_link-epN of re/harness/link_check.sh).

Usage: re/.venv/bin/python -I re/harness/link_lengths.py [NAME ...]    (default link-ep2 .. link-ep8; run link_check.sh first)
Prints for each link the records at which the game mode G.$4D0 changes (8 = the story panel, 4 = the stage start, 0xC = play; Episode 5 has no mode 4) and the
record of G.$4E3 (the heroes' walk in is over), MAME against the port. The panel lengths of D4c (306, 250, 308, 439, 368, 368, 313) are the records of the mode 0xC plus 1.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import trace as T  # noqa: E402

TRACES = os.path.join(HERE, "..", "out", "traces")


def scan(path):
    modes, prev, b4e3 = [], None, None
    for r in T.Trace(path).records():
        ram = r.images["ram"]
        m = int(ram[0xFF84D0 - 0xFF0000])
        if m != prev:
            modes.append((r.frame, m))
            prev = m
        if b4e3 is None and ram[0xFF84E3 - 0xFF0000] == 1:
            b4e3 = r.frame
        if r.frame > 1200:
            break
    return modes, b4e3


names = sys.argv[1:] or [f"link-ep{k}" for k in range(2, 9)]
for n in names:
    m, mb = scan(os.path.join(TRACES, f"m228_{n}.mame.trc"))
    p, pb = scan(os.path.join(TRACES, f"m228_{n}.port.trc"))
    fmt = lambda x: " ".join(f"{f}:{v:#x}" for f, v in x)
    print(f"{n}: MAME {fmt(m)} G.$4E3 {mb} | port {fmt(p)} G.$4E3 {pb} | {'equal' if (m, mb) == (p, pb) else 'DIFFERENT'}")
