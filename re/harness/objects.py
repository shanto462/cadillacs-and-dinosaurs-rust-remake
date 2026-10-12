"""List the live objects of a trace or state file at given frames (debug helper).

Usage: re/.venv/bin/python -I re/harness/objects.py TRACE [FRAME ...]   (default: frame -1)
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trace as T  # noqa: E402

POOLS = [("player", 0xFFB274, 0x180, 3), ("enemy", 0xFFC8F4, 0xE0, 24), ("prop", 0xFFB6F4, 0xC0, 24),
         ("item", 0xFF8874, 0xC0, 24), ("effect", 0xFF9A74, 0x80, 48), ("special", 0xFFDDF4, 0x80, 16)]


def show(ram, frame):
    print(f"frame {frame}: G.$0000={T.u16(ram, 0xFF8000):04X} rng={T.u16(ram, 0xFF8014):04X} cam=({T.u16(ram, 0xFF8744):04X},{T.u16(ram, 0xFF8748):04X})")
    for kind, base, stride, n in POOLS:
        for i in range(n):
            a = base + i * stride
            if T.u8(ram, a) == 0:
                continue
            print(f"  {kind}[{i:2d}] {a & 0xFFFF:04X} live {T.u8(ram, a):02X} class {T.u16(ram, a + 0x20) // 4:#04x} "
                  f"state {T.u32(ram, a + 4):08X} x {T.u16(ram, a + 8):04X} y {T.u16(ram, a + 12):04X} z {T.u16(ram, a + 16):04X} "
                  f"frame {T.u32(ram, a + 0x28):06X} flags {T.u8(ram, a + 0x25):02X} var {T.u16(ram, a + 0x26):04X}")


def main():
    t = T.Trace(sys.argv[1])
    want = [int(x) for x in sys.argv[2:]] or [-1]
    for rec in t.records():
        if rec.frame in want:
            show(rec.images["ram"], rec.frame)
        if rec.frame >= max(want):
            break


if __name__ == "__main__":
    main()
