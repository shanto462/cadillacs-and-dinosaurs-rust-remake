#!/usr/bin/env python3
"""Prepare a port run of a story panel recording (M2-26 to M2-28, re/harness/link_check.sh).

Usage: link_prep.py SOURCE.mame.trc NAME

The recording (flow_links / flow_panels, video clock) pokes G.$4D9 (the stage), G.$4D1 = 0 and G.$4D0 = 8 (mode 8) into a running level at t = 0. This cuts the
record before the first record with the mode 8 as the snapshot (trace_slice.py FF84D0=8:-1) and writes the same three pokes for the port as the sidecar
re/out/traces/NAME.pokes.txt (game clock: before the tick of frame 0), so the port runs the first mode 8 frame itself. Prints: the number of records after the
snapshot and the first record after the story panel's level start (the first record with the mode 4 after the mode 8) for the allow file.
"""
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import trace as T  # noqa: E402

ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))


def main():
    src, name = sys.argv[1], sys.argv[2]
    stage = None
    first = None
    before = None
    live = []
    last = None
    n = 0
    mode4 = None
    shadow = {"display_y1": [], "display_y2": []}
    t = T.Trace(src)
    for rec in t.records():
        if rec.frame < 0:
            continue
        ram = rec.images["ram"]
        n = rec.frame
        if first is None and ram[0x84D0] == 8:
            first = rec.frame
            stage = int(ram[0x84D9])
            # the live bytes of the heroes (the recorder's jump leaves 2 in play, the recording has 1 from t = 0 on, see allow/m228.txt)
            live = [(a, int(ram[a & 0xFFFF])) for a in (0xFFB274, 0xFFB3F4, 0xFFB574) if before is not None and int(before[a & 0xFFFF]) != int(ram[a & 0xFFFF])]
        elif first is not None and mode4 is None and ram[0x84D0] != 8:
            mode4 = rec.frame - first
        if first is not None:
            # the display shadows G.$860 / G.$868 (copied from the cameras G.$6C8 / G.$748 by the main loop at the start of its pass) lag the cameras
            # by a pass; the port derives them from the cameras: the frames where MAME's shadow is not its camera are listed (allowed in the diff)
            for key, cam, sh in (("display_y1", 0x86C8, 0x8860), ("display_y2", 0x8748, 0x8868)):
                if int(ram[cam]) != int(ram[sh]) or int(ram[cam + 1]) != int(ram[sh + 1]):
                    shadow[key].append(rec.frame - first)
        before = ram.copy()
    subprocess.run([sys.executable, "-I", os.path.join(HERE, "trace_slice.py"), src, name, "FF84D0=8:-1"], check=True, stdout=subprocess.DEVNULL)
    pokes = os.path.join(ROOT, "re", "out", "traces", name + ".pokes.txt")
    with open(pokes, "w") as f:
        f.write("# the pokes of the recording at t = 0 (re/harness/link_prep.py)\n#!clock game\n#!unknown warn\n")
        f.write(f"0 FF84D9 1 {stage}\n0 FF84D1 1 0\n0 FF84D0 1 8\n")
        for a, v in live:
            f.write(f"0 {a:X} 1 {v}\n")
    print(n - first, mode4 if mode4 is not None else -1)


main()
