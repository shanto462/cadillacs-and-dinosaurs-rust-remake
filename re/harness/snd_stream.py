"""M10-22: compare the sound request streams (code, d1, d2) of a MAME trace and a port trace, record by record.

    snd_stream.py MAME.mame.trc PORT.trc [--allow FILE] [--state DIFF.txt] [--list N] [--lag-jumps] [--json OUT]

MAME side: the ring entries queued in the record (`trace.mame_sounds`: ring at G.$3A, write index G.$34, enqueue 0xA63C).
Port side: the record's `snd` / `snd_d` (code, d1, d2 of every `Game::sound_ex`). Only the RAM blocks of G.$0 and the ring are
decoded, so a 200 MB trace takes seconds (diff.py compares the codes too, but only the d1 / d2 of the records whose code lists agree).

Records left out as in diff.py: the MAME lag records (flags & 1) when the port trace is a step or lagsync run, and the catch-up records
(an unflagged record whose G.$0 moves by 2 to 0x100) in lagsync mode, or in step mode with --lag-jumps.

`--allow FILE`: an allow file in the diff.py format (`key range # reason`, fnmatch keys); the keys used here are `sounds` (the whole
record) and `sound.CODE`, `sound.CODE.d1`, `sound.CODE.d2` (the code in hex, two digits at least: `sound.3E.d2`, `sound.0F.*`).

`--heard`: only the requests that the flush 0x610 sends to the Z80 (the attract filter: a 0xFFxx command, the coin 0x0030, or any code while
G.$4CC is set (in a game) or the demo sound bit 5 of G.$777C is set; the filter state of the record is MAME's after the record).

`--state DIFF.txt`: the output of `diff.py MAME PORT --frame-list` for the same pair. A difference in a record whose state differs too (a
field, a tile, a sprite: any item of diff.py other than the sound codes) is counted apart, as a consequence of that difference: in step mode
the sound of such a record comes from code that already went another way (the pan of a camera that differs, a flow step a frame early).

Prints `SOUND STREAM: ...` with the counts, then the differences grouped by code. Exit 0 when no sound-only difference is left outside
the allow file (the records with a state difference belong to the checks of that state).
"""

import argparse
import fnmatch
import json
import re
import struct
import sys
from collections import defaultdict

MAGIC = b"DINOTRC1"


def records(path, lo, hi):
    """Yields the header, then (frame, flags, ram_window, extra) per record. ram_window is a bytearray of the RAM offsets lo..hi (updated in place)."""
    with open(path, "rb") as f:
        d = f.read()
    if d[:8] != MAGIC:
        raise SystemExit(f"{path}: not a trace file")
    (n,) = struct.unpack_from("<I", d, 8)
    hdr = json.loads(d[12:12 + n])
    pos = 12 + n
    bs = hdr["block"]
    regions = hdr["regions"]
    win = bytearray(hi - lo)
    b_lo, b_hi = lo // bs, (hi + bs - 1) // bs
    yield hdr
    ln = len(d)
    unpack = struct.unpack_from
    while pos < ln:
        frame, flags, _vb, _lag = unpack("<iIII", d, pos)
        pos += 16
        for r in regions:
            (count,) = unpack("<I", d, pos)
            pos += 4
            if r["name"] != "ram":
                pos += count * (4 + bs)
                continue
            for _ in range(count):
                (idx,) = unpack("<I", d, pos)
                pos += 4
                if b_lo <= idx < b_hi:
                    a = idx * bs
                    s0, s1 = max(a, lo), min(a + bs, hi)
                    win[s0 - lo:s1 - lo] = d[pos + s0 - a:pos + s1 - a]
                pos += bs
        (n,) = unpack("<I", d, pos)
        pos += 4
        extra = json.loads(d[pos:pos + n]) if n else {}
        pos += n
        yield frame, flags, win, extra


LO, HI = 0x8000, 0xF780  # G.$0 .. G.$777C (RAM offsets from 0xFF0000): the frame counter, the ring, G.$4CC and the demo sound bit


def w16(win, off):
    return (win[off - LO] << 8) | win[off - LO + 1]


def w32(win, off):
    return (w16(win, off) << 16) | w16(win, off + 2)


def ring(prev_idx, win):
    a, b = prev_idx & 0x3F0, w16(win, 0x8034) & 0x3F0
    out = []
    while a != b:
        e = 0x803A + a
        out.append((w16(win, e), w32(win, e + 2), w32(win, e + 6)))
        a = (a + 0x10) & 0x3F0
    return out


class Allow:
    """The rules of a diff.py allow file (`key [lo[-hi]]`, fnmatch keys)."""

    def __init__(self, path):
        self.rules = []
        if not path:
            return
        for line in open(path):
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            parts = line.split()
            lo, hi = -1, 1 << 30
            if len(parts) > 1:
                m = re.match(r"^(\d+)(?:-(\d+))?$", parts[1])
                if m:
                    lo = int(m.group(1))
                    hi = int(m.group(2)) if m.group(2) else lo
            self.rules.append((parts[0], lo, hi))

    def ok(self, key, f):
        return any(lo <= f <= hi and fnmatch.fnmatchcase(key, p) for p, lo, hi in self.rules)


def state_frames(path):
    """The records of `diff.py --frame-list` (`DIFFERING FRAMES (first-last: items): a-b: n, c: m, ...`). Returns (all records of a run,
    the single records of a run with one item: these are a sound code difference alone when the record has one)."""
    every, single = set(), set()
    if not path:
        return every, single
    for line in open(path):
        if line.startswith("DIFFERING FRAMES"):
            for m in re.finditer(r"(\d+)(?:-(\d+))?: (\d+)", line.split(":", 1)[1]):
                a = int(m.group(1))
                b = int(m.group(2)) if m.group(2) else a
                every.update(range(a, b + 1))
                if a == b and m.group(3) == "1":
                    single.add(a)
    return every, single


def heard(c, win):
    """0x610: does the flush send this queued code (the attract filter)?"""
    return c & 0xFF00 == 0xFF00 or c == 0x0030 or win[0x84CC - LO] != 0 or win[0xF77C - LO] & 0x20 != 0


def key(c):
    return f"sound.{c:02X}"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("mame")
    ap.add_argument("port")
    ap.add_argument("--allow")
    ap.add_argument("--state", help="diff.py --frame-list output of the same pair")
    ap.add_argument("--list", type=int, default=3, help="records listed per group")
    ap.add_argument("--lag-jumps", action="store_true")
    ap.add_argument("--heard", action="store_true", help="only the requests that pass the attract filter of the flush 0x610")
    ap.add_argument("--json", help="write every difference to this JSON file")
    args = ap.parse_args()
    allow = Allow(args.allow)
    st_all, st_single = state_frames(args.state)
    gm, gp = records(args.mame, LO, HI), records(args.port, LO, HI)
    hm, hp = next(gm), next(gp)
    if hp.get("source") != "port":
        gm, gp, hm, hp = gp, gm, hp, hm
    mode = hp.get("mode", "run")
    skip_lag = mode in ("step", "lagsync")
    skip_jumps = mode == "lagsync" or args.lag_jumps
    prev_idx = None
    prev_g0 = None
    compared = reqs = skipped = 0
    code_diffs = []  # (frame, mame list, port list, port sites, state differs)
    arg_diffs = []  # (frame, code, md1, md2, pd1, pd2, site, state differs)
    allowed = 0
    for (fm, flm, wm, _xm), (fp, _flp, _wp, xp) in zip(gm, gp):
        if fm != fp:
            print(f"record mismatch: MAME {fm}, port {fp}")
            break
        g0 = w16(wm, 0x8000)
        catch_up = not flm & 1 and prev_g0 is not None and 2 <= ((g0 - prev_g0) & 0xFFFF) <= 0x100
        prev_g0 = g0
        sm = ring(prev_idx, wm) if prev_idx is not None else []
        prev_idx = w16(wm, 0x8034)
        if fm < 0:
            continue
        if skip_lag and (flm & 1 or (catch_up and skip_jumps)):
            skipped += 1
            continue
        compared += 1
        cp = list(xp.get("snd", []))
        dp = [tuple(x) for x in xp.get("snd_d", [])]
        dp += [(0, 0)] * (len(cp) - len(dp))
        sites = list(xp.get("site", []))
        sites += [""] * (len(cp) - len(sites))
        if args.heard:
            keep = [heard(c, wm) for c in cp]
            cp, dp, sites = [x for x, k in zip(cp, keep) if k], [x for x, k in zip(dp, keep) if k], [x for x, k in zip(sites, keep) if k]
            sm = [r for r in sm if heard(r[0], wm)]
        cm = [c for c, _, _ in sm]
        reqs += len(cm)
        if cm != cp:
            if allow.ok("sounds", fm):
                allowed += 1
            else:
                code_diffs.append((fm, sm, list(zip(cp, dp)), sites, fm in st_all and fm not in st_single))
            continue
        for (c, a, b), (pa, pb), site in zip(sm, dp, sites):
            if (a, b) == (pa, pb):
                continue
            k = key(c)
            if allow.ok("sounds", fm) or allow.ok(k, fm) or (
                    (a == pa or allow.ok(k + ".d1", fm)) and (b == pb or allow.ok(k + ".d2", fm))):
                allowed += 1
                continue
            arg_diffs.append((fm, c, a, b, pa, pb, site, fm in st_all))
    n_code = [sum(1 for d in code_diffs if d[4] == s) for s in (False, True)]
    n_arg = [sum(1 for d in arg_diffs if d[7] == s) for s in (False, True)]
    st = f"; with a state difference in the record: code {n_code[1]} records, d1 / d2 {n_arg[1]} requests" if args.state else ""
    print(f"SOUND STREAM{' (heard)' if args.heard else ''}: mode {mode}, {compared} records compared ({skipped} lag / catch-up records left out), {reqs} MAME requests; "
          f"{'sound only: ' if args.state else ''}code differences in {n_code[0]} records, d1 / d2 differences in {n_arg[0]} requests{st}; {allowed} allowed")
    for state in (False, True):
        lst = [d for d in code_diffs if d[4] == state]
        if lst:
            print(f"CODE DIFFERENCES{' (state differs too)' if state else ''} (record: MAME [code d1/d2 ...] port [...]):")
            for f, sm, sp, _, _ in lst[:max(args.list, 0) * 10]:
                print(f"  {f}: MAME [" + " ".join(f"{c:04X} {a:X}/{b:X}" for c, a, b in sm) + "] port [" + " ".join(f"{c:04X} {a:X}/{b:X}" for c, (a, b) in sp) + "]")
        groups = defaultdict(list)
        for d in arg_diffs:
            if d[7] != state:
                continue
            f, c, a, b, pa, pb, site, _ = d
            what = ("d1" if a != pa else "") + ("d2" if b != pb else "")
            groups[(c, what, site)].append(d)
        if groups:
            print(f"D1 / D2 DIFFERENCES{' (state differs too)' if state else ''} by code (requests, first records):")
            for (c, what, site), lst in sorted(groups.items(), key=lambda kv: -len(kv[1])):
                ex = " ".join(f"{f}:{a:X}/{b:X}|{pa:X}/{pb:X}" for f, _, a, b, pa, pb, _, _ in lst[:args.list])
                print(f"  {c:04X} {what}{' ' + site if site else ''}: {len(lst)}  {ex}")
    if args.json:
        with open(args.json, "w") as f:
            json.dump({"mode": mode, "compared": compared, "skipped": skipped, "requests": reqs, "allowed": allowed,
                       "code": [[f, sm, [[c, a, b] for c, (a, b) in sp], st, s] for f, sm, sp, st, s in code_diffs],
                       "args": arg_diffs}, f)
    sys.exit(1 if n_code[0] or n_arg[0] else 0)


if __name__ == "__main__":
    main()
