#!/usr/bin/env python3
"""Cut a snapshot and the rest of a trace out of a power-on MAME recording, without another MAME run.

Usage: re/.venv/bin/python -I re/harness/trace_slice.py SOURCE.mame.trc NAME RECORD [--frames N]
       RECORD is a record number, or ADDR=VALUE (hex RAM address, byte value): the first record where the byte has that value
       (for example FF84D1=6, the first record of the clear tally), or ADDR=VALUE:-N: N records before that one (FF84D0=8:-1 is the last record
       before a poke of the mode 8, the story panel: the poke file of the port then writes it at frame 0).

Writes re/out/traces/NAME.state (one record, frame -1: the whole state after RECORD of the source) and
re/out/traces/NAME.mame.trc (that state as frame -1, then the records RECORD + 1 .. as frames 0 ..), so that
`re/harness/check.sh step|run|follow NAME` and `port.sh` can load the port from the middle of a power-on or batch
recording (the clear tally of `flow_tally`, the link scenes of `flow_links`, ...). The sidecars (pokes) are not copied.
"""
import argparse
import json
import os
import struct
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trace as T  # noqa: E402

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))


def blocks_of(img, prev, bs):
    """Indexes of the blocks of `img` that differ from `prev` (all blocks when prev is None)."""
    n = len(img) // bs
    if prev is None:
        return range(n)
    a = img.reshape(n, bs)
    b = prev.reshape(n, bs)
    return np.nonzero((a != b).any(axis=1))[0].tolist()


def write_record(out, t, frame, flags, vblank, lag, images, prev, extra):
    out.write(struct.pack("<iIII", frame, flags, vblank, lag))
    for r in t.regions:
        img = images[r["name"]]
        p = None if prev is None else prev[r["name"]]
        idx = list(blocks_of(img, p, t.block))
        out.write(struct.pack("<I", len(idx)))
        for i in idx:
            out.write(struct.pack("<I", i))
            out.write(img[i * t.block:(i + 1) * t.block].tobytes())
    if extra:
        e = json.dumps(extra, separators=(",", ":")).encode()
        out.write(struct.pack("<I", len(e)) + e)
    else:
        out.write(struct.pack("<I", 0))


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("source")
    ap.add_argument("name")
    ap.add_argument("record")
    ap.add_argument("--frames", type=int, default=None, help="records to keep after the snapshot (default: all)")
    args = ap.parse_args()
    t = T.Trace(args.source)
    if "=" in args.record:
        addr, val = args.record.split("=")
        before = 0
        if ":" in val:
            val, before = val.split(":")
            before = -int(before)
        addr, val = int(addr, 16) - 0xFF0000, int(val, 0)
        found = None
        for rec in t.records():
            if rec.frame >= 0 and rec.images["ram"][addr] == val:
                found = rec.frame - before
                break
        if found is None:
            sys.exit(f"no record with {args.record}")
        args.record = found
    else:
        args.record = int(args.record)
    outdir = os.path.join(ROOT, "re", "out", "traces")
    header = dict(t.header)
    head = b"DINOTRC1" + struct.pack("<I", 0)
    enc = json.dumps(header, separators=(",", ":")).encode()
    head = b"DINOTRC1" + struct.pack("<I", len(enc)) + enc
    state_path = os.path.join(outdir, args.name + ".state")
    trace_path = os.path.join(outdir, args.name + ".mame.trc")
    prev = None
    kept = 0
    with open(state_path, "wb") as st, open(trace_path, "wb") as tr:
        st.write(head)
        tr.write(head)
        for rec in t.records():
            if rec.frame < args.record:
                continue
            if rec.frame == args.record:
                snap = {k: v.copy() for k, v in rec.images.items()}
                write_record(st, t, -1, 0, rec.vblank, 0, snap, None, {})
                write_record(tr, t, -1, 0, rec.vblank, 0, snap, None, {})
                prev = snap
                continue
            if args.frames is not None and kept >= args.frames:
                break
            cur = {k: v.copy() for k, v in rec.images.items()}
            write_record(tr, t, rec.frame - args.record - 1, rec.flags, rec.vblank, rec.lag, cur, prev, rec.extra)
            prev = cur
            kept += 1
    print(f"{state_path}\n{trace_path}: snapshot of record {args.record}, {kept} records after it")


if __name__ == "__main__":
    main()
