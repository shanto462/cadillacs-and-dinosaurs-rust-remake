"""Print chosen RAM values of a trace (MAME or port) for a range of frames; only the frames where a value changed.

Usage: re/.venv/bin/python -I re/harness/watch.py TRACE FIRST LAST SPEC [SPEC ...] [--all] [--region ram|gfx]
  SPEC  ADDR:SIZE[:NAME]  ADDR is hex (0xFF844D, FF844D or a G.$ offset such as G+744 = 0xFF8744), SIZE 1, 2 or 4 bytes
        (a trailing s makes it signed: 2s). With a G+ address the name defaults to that text.
  --all  print every frame, not only the changes.
  --region gfx  the addresses are in the gfx region (base 0x900000, for example the row scroll table 0x915800).
Example (camera of the stage 2 controller): watch.py re/out/traces/ep3_jack.mame.trc 0 3000 G+741:1:mode G+744:2:camx G+7B4:1:lock G+518:1:car
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trace as T  # noqa: E402


def parse(spec):
    parts = spec.split(":")
    a = parts[0]
    if a.upper().startswith("G+"):
        addr = 0xFF8000 + int(a[2:], 16)
        name = parts[2] if len(parts) > 2 else a
    else:
        addr = int(a, 16)
        if addr < 0x10000:
            addr += 0xFF0000
        name = parts[2] if len(parts) > 2 else a
    size = parts[1] if len(parts) > 1 else "1"
    signed = size.endswith("s")
    return addr, int(size.rstrip("s")), signed, name


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    flags = [a for a in sys.argv[1:] if a.startswith("--")]
    region = "ram"
    if "--region" in sys.argv:
        region = sys.argv[sys.argv.index("--region") + 1]
        args.remove(region)
    t = T.Trace(args[0])
    first, last = int(args[1]), int(args[2])
    specs = [parse(s) for s in args[3:]]
    every = "--all" in flags
    base = T.GFX_BASE if region == "gfx" else T.RAM_BASE
    prev = None
    for rec in t.records():
        if rec.frame < first:
            continue
        if rec.frame > last:
            break
        img = rec.images[region]
        vals = []
        for addr, size, signed, _ in specs:
            o = addr - base
            v = int.from_bytes(bytes(img[o:o + size]), "big", signed=signed)
            vals.append(v)
        if every or vals != prev:
            print(f"{rec.frame:6d}  " + "  ".join(f"{n}={v:#x}" if v >= 0 else f"{n}={v}" for (_, _, _, n), v in zip(specs, vals)))
        prev = vals


if __name__ == "__main__":
    main()
