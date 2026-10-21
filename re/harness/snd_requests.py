"""M10-23: the request list of the register trace check, from the sound requests that MAME recordings queue.

    snd_requests.py OUT.txt TRACE.mame.trc ...

Collects every request (code, d1, d2) of the traces (the ring G.$3A / G.$34, `snd_stream.records`) and writes a `sound.sh` / `compare`
request file with one case per kind of effect request:
- pan mode 0 (d1 bits 16..18 = 0: a fixed pan, the driver reads no d2): one case per code, its most used request;
- pan modes 1 to 7: every (code, d1 without the pan byte, d2) with its most used pan below 0x11 and its most used pan from 0x11 (the two
  branches of the mode routines, 0x0251).
Music codes (0x00 .. 0x26) read no request byte (0x017D -> music_start) and are covered by M4-10; the fade 0xFF07 is added once after the
area tune 0x05 (`at=240`). Then:

    SQ_INPUT=OUT.txt SQ_FRAMES=900 re/harness/probe/sound.sh
    (cd dino-rs && cargo run --release -p dino-sound --example compare -- ../OUT.txt ../re/out/sound 900)
"""

import os
import sys
from collections import Counter, defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import snd_stream as S  # noqa: E402


def main():
    if len(sys.argv) < 3:
        raise SystemExit(__doc__)
    seen = Counter()
    for path in sys.argv[2:]:
        g = S.records(path, S.LO, S.HI)
        next(g)
        prev = None
        for _f, _fl, win, _x in g:
            if prev is not None:
                for r in S.ring(prev, win):
                    seen[r] += 1
            prev = S.w16(win, 0x8034)
    mode0 = defaultdict(Counter)
    groups = defaultdict(Counter)
    for (c, d1, d2), n in seen.items():
        if not 0x27 <= c < 0xFF00:
            continue
        if (d1 >> 16) & 7 == 0:
            mode0[c][(d1, d2)] += n
        else:
            groups[(c, d1 & 0xFFFF00, d2)][d1 & 0xFF] += n
    cases = []
    for c in sorted(mode0):
        (d1, d2), _ = mode0[c].most_common(1)[0]
        cases.append((c, d1, d2))
    for (c, d1m, d2), pans in sorted(groups.items()):
        lo = [p for p, _ in pans.most_common() if p < 0x11]
        hi = [p for p, _ in pans.most_common() if p >= 0x11]
        cases += [(c, d1m | p, d2) for p in lo[:1] + hi[:1]]
    lines = [f"sound {c:#06x} {d1:#010x} {d2:#010x}" for c, d1, d2 in dict.fromkeys(cases)]
    lines += ["sound 0x0005 0 0 name=fade_0005", "sound 0xff07 0x04440000 0 at=240"]
    with open(sys.argv[1], "w") as f:
        f.write("\n".join(lines) + "\n")
    print(f"{len(seen)} distinct requests in {len(sys.argv) - 2} traces; {len(lines) - 1} cases ({len(mode0)} codes in pan mode 0, "
          f"{len(groups)} groups of the modes 1 to 7)")


if __name__ == "__main__":
    main()
