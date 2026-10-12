"""Print how the bytes of one RAM object (or any RAM range) change from frame to frame in a trace
(debug helper, MAME or port trace).

Usage: re/.venv/bin/python -I re/harness/objhex.py TRACE ADDR LEN FIRST [LAST]
  ADDR   hex RAM address or handle (0xD294 or FFD294), LEN bytes, frames FIRST..LAST (default FIRST)
Prints the full bytes at FIRST, then for every later frame the changed bytes as `+off old>new`.
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trace as T  # noqa: E402


def main():
    t = T.Trace(sys.argv[1])
    addr = int(sys.argv[2], 16)
    if addr < 0x10000:
        addr += 0xFF0000
    ln = int(sys.argv[3], 0)
    first = int(sys.argv[4])
    last = int(sys.argv[5]) if len(sys.argv) > 5 else first
    prev = None
    for rec in t.records():
        if rec.frame < first:
            continue
        if rec.frame > last:
            break
        ram = rec.images["ram"]
        cur = bytes(ram[addr - 0xFF0000 : addr - 0xFF0000 + ln])
        if prev is None:
            print(f"frame {rec.frame}:")
            for o in range(0, ln, 16):
                print(f"  +{o:02X}: " + " ".join(f"{b:02X}" for b in cur[o : o + 16]))
        else:
            ch = [f"+{o:02X} {prev[o]:02X}>{cur[o]:02X}" for o in range(ln) if cur[o] != prev[o]]
            if ch:
                print(f"frame {rec.frame}: " + "  ".join(ch))
        prev = cur


if __name__ == "__main__":
    main()
