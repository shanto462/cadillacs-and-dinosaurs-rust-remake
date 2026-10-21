# Are two harness trace files the same recording? Compares the records after the JSON header (the header names the
# input path, which differs between a coverage replay and the original). Prints one line per pair; exit 1 on a difference.
#   re/.venv/bin/python -I re/harness/coverage/same.py ORIGINAL.mame.trc REPLAY.mame.trc [ORIGINAL2 REPLAY2 ...]
# `same:` every record is equal. `prefix:` the shorter trace is equal to the start of the longer one (a recording cut
# with trace_prefix.py, or a replay that ran longer). `DIFFERENT:` the first differing record (its frame field).
import json
import struct
import sys


def body(path):
    with open(path, "rb") as fh:
        magic = fh.read(8)
        n = struct.unpack("<I", fh.read(4))[0]
        head = json.loads(fh.read(n))
        return magic, head, fh.read()


def records(data, nregions):
    """Yield (frame, raw bytes of the record) for every record of a trace body."""
    p = 0
    while p + 16 <= len(data):
        start = p
        frame = struct.unpack_from("<i", data, p)[0]
        p += 16
        for _ in range(nregions):
            k = struct.unpack_from("<I", data, p)[0]
            p += 4 + k * 68
        x = struct.unpack_from("<I", data, p)[0]
        p += 4 + x
        yield frame, data[start:p]


bad = 0
args = sys.argv[1:]
for a, b in zip(args[0::2], args[1::2]):
    ma, ha, ba = body(a)
    mb, hb, bb = body(b)
    if ma == mb and ba == bb and ha.get("sync_vblank") == hb.get("sync_vblank"):
        print("same: %s (%d bytes of records)" % (b, len(bb)))
        continue
    nreg = len(ha.get("regions", [])) or 4
    ra, rb = list(records(ba, nreg)), list(records(bb, nreg))
    n = 0
    while n < len(ra) and n < len(rb) and ra[n][1] == rb[n][1]:
        n += 1
    if n == min(len(ra), len(rb)) and ha.get("sync_vblank") == hb.get("sync_vblank"):
        print("prefix: %s: the first %d records are equal (%d and %d records)" % (b, n, len(ra), len(rb)))
    else:
        bad = 1
        fr = ra[n][0] if n < len(ra) else None
        print("DIFFERENT: %s vs %s: %d records equal, first difference at record %d (frame %s); sync %s / %s"
              % (a, b, n, n, fr, ha.get("sync_vblank"), hb.get("sync_vblank")))
sys.exit(bad)
