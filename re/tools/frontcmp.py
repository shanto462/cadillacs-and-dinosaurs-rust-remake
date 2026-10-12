"""Compare the tilemaps and palettes of the port with a MAME power-on trace, frame by frame.

Usage (from the repo root):
  re/.venv/bin/python -I re/tools/frontcmp.py MAME.trc PORT.trc OFFSET FRAME [FRAME ...]

Both files are harness traces (re/harness/trace.py). The trace of a power-on run
(`re/out/traces/attract.mame.trc`, see re/harness/README.md) has frame k = video frame k + 2.
OFFSET is added to the port frame to get the MAME record: compare port record F with MAME record
F + OFFSET. For each FRAME the script lists the scroll1, scroll2, scroll3 cells and palette words
that differ (the port must model the bytes: where the port's `gfx_mask` is 0 the cell is skipped
and counted as "not modelled").

Layers in the gfx region (base 0x900000): scroll1 0x8000..0xBFFF, scroll2 0xC000..0xFFFF,
scroll3 0x10000..0x13FFF, palettes 0x14000..0x157FF. A cell is a code word and an attribute word.
"""
import sys

sys.path.insert(0, __file__.rsplit("/re/tools/", 1)[0] + "/re/harness")
from trace import Trace  # noqa: E402

LAYERS = [("scroll1", 0x8000, 0x4000), ("scroll2", 0xC000, 0x4000), ("scroll3", 0x10000, 0x4000)]


def images_at(path, wanted):
    """{frame: (gfx bytes, gfx_mask bytes or None)} for the wanted frame numbers."""
    t = Trace(path)
    out = {}
    last = max(wanted)
    for rec in t.records():
        if rec.frame in wanted:
            gfx = rec.images["gfx"].copy()
            mask = rec.images["gfx_mask"].copy() if "gfx_mask" in rec.images else None
            out[rec.frame] = (gfx, mask)
        if rec.frame >= last:
            break
    return out


def main():
    mame_path, port_path, offset = sys.argv[1], sys.argv[2], int(sys.argv[3])
    frames = [int(a) for a in sys.argv[4:]]
    port = images_at(port_path, set(frames))
    mame = images_at(mame_path, {f + offset for f in frames})
    for f in frames:
        pg, pm = port[f]
        mg, _ = mame[f + offset]
        total = 0
        for name, base, size in LAYERS:
            diffs = []
            skipped = 0
            for c in range(size // 4):
                a = base + 4 * c
                if pm is not None and not pm[a : a + 4].any():
                    skipped += 1
                    continue
                if (pg[a : a + 4] != mg[a : a + 4]).any():
                    diffs.append(c)
            total += len(diffs)
            print(f"frame {f} (MAME {f + offset}) {name}: {len(diffs)} cells differ, {skipped} not modelled")
            for c in diffs[:6]:
                a = base + 4 * c
                p = (int.from_bytes(pg[a : a + 2], "big"), int.from_bytes(pg[a + 2 : a + 4], "big"))
                m = (int.from_bytes(mg[a : a + 2], "big"), int.from_bytes(mg[a + 2 : a + 4], "big"))
                print(f"    cell {c} (0x{0x900000 + a:06X}): port code/attr {p[0]:04X}/{p[1]:04X}  MAME {m[0]:04X}/{m[1]:04X}")
        pal = 0
        for a in range(0x14000, 0x14800, 2):
            if pm is not None and not pm[a : a + 2].any():
                continue
            if pg[a : a + 2].tobytes() != mg[a : a + 2].tobytes():
                pal += 1
        print(f"frame {f}: {pal} palette words differ (0x914000..0x9147FF), tilemap cells differ: {total}")


if __name__ == "__main__":
    main()
