"""Which (class, mode, sub, act, step) combinations of the pool A / B objects occur in a trace (debug
helper: how much of a class's code a recording reaches).
Usage: re/.venv/bin/python -I re/harness/cover.py TRACE [KIND] [CLASS ...]    KIND = enemy (default) | prop
Prints, per class, the set of state words (+$04..+$07) seen."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trace as T  # noqa: E402

POOLS = {"enemy": (0xFFC8F4, 0xE0, 24), "prop": (0xFFB6F4, 0xC0, 24)}


def main():
    t = T.Trace(sys.argv[1])
    kind = sys.argv[2] if len(sys.argv) > 2 else "enemy"
    want = {int(x, 0) for x in sys.argv[3:]}
    base, stride, n = POOLS[kind]
    seen = {}
    for rec in t.records():
        ram = rec.images["ram"]
        for i in range(n):
            a = base + i * stride
            if T.u8(ram, a) == 0:
                continue
            c = T.u16(ram, a + 0x20) // 4
            if want and c not in want:
                continue
            seen.setdefault(c, {}).setdefault(T.u32(ram, a + 4), []).append(rec.frame)
    for c in sorted(seen):
        print(f"class {c} ({c:#x}):")
        for st, fr in sorted(seen[c].items()):
            print(f"   state {st:08X}: {len(fr)} frames, first {fr[0]}, last {fr[-1]}")


if __name__ == "__main__":
    main()
