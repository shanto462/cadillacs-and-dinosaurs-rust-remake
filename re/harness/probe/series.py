"""Turn the W (watch) lines of a probe log into a table, one row per sample.

    re/.venv/bin/python -I re/harness/probe/series.py LOG SELECTOR [FIELD ...] [--delta] [--all]

LOG       re/out/probe/NAME/log.txt
SELECTOR  the label used in the log: A#20, B#3, E#5, P1, G ...  (what comes after "W ")
FIELD     fields to print (default: all fields of the first sample). Values are decimal or hex as logged.
--delta   also print the change of each numeric field since the previous row (speeds per frame)
--all     print every video frame; without it only frames where the game frame counter gf changed
          (a slow-down frame repeats the old state)

Output columns: t (frames after the jump), gf (game frame), then the fields. Example (Zeke dive):
    series.py re/out/probe/zeke/log.txt A#20 x y z --delta
"""
import re
import sys


def main(argv):
    flags = [a for a in argv if a.startswith("--")]
    args = [a for a in argv if not a.startswith("--")]
    if len(args) < 2:
        print(__doc__)
        return 1
    log, sel = args[0], args[1]
    want = args[2:]
    rows = []
    last_gf = None
    for line in open(log, errors="replace"):
        m = re.match(r"t=(\S+) f=(\d+) gf=(\d+) W (\S+) ?(.*)", line.rstrip("\n"))
        if not m or m.group(4) != sel:
            continue
        t, f, gf, _, rest = m.groups()
        if "--all" not in flags and gf == last_gf:
            continue
        last_gf = gf
        vals = dict(kv.split("=", 1) for kv in rest.split() if "=" in kv)
        rows.append((t, gf, vals))
    if not rows:
        print("no samples for", sel)
        return 1
    fields = want or list(rows[0][2].keys())
    print("t gf " + " ".join(fields) + ("  | change" if "--delta" in flags else ""))
    prev = None
    for t, gf, vals in rows:
        out = [t, gf] + [vals.get(k, "-") for k in fields]
        line = " ".join(out)
        if "--delta" in flags:
            d = []
            for k in fields:
                try:
                    v = int(vals[k])
                    d.append("%+d" % (v - int(prev[k])) if prev and k in prev else "0")
                except (KeyError, ValueError):
                    d.append("-")
            line += "  | " + " ".join(d)
        print(line)
        prev = vals
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
