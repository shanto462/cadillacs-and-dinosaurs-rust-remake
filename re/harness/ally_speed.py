"""Ally speed table (Task 20): for every jump point, the plain tick cost (no ally) and a paced run with --ally 2.

Usage (from the repo root, release build): re/.venv/bin/python -I re/harness/ally_speed.py [BIN] [FRAMES] [JUMP ...]
Prints one row per jump: plain tick cost in microseconds (CPU time of 3000 ticks without an ally, the best of 3, minus the start), plan average / longest in ms, the wait of the game for a plan, the frames
that took longer than a video frame, the frames the ally played the default policy (decision not ready), the changes of the search level.
P1 walks right and attacks every 24 frames (re/harness/scripts/ally_walk24.txt). The paced run takes FRAMES / 59.64 seconds per jump.
"""
import os
import re
import resource
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BIN = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "dino-rs/target/release/dino-game")
FRAMES = int(sys.argv[2]) if len(sys.argv) > 2 else 3000
SCRIPT = os.path.join(ROOT, "re/harness/scripts/ally_walk24.txt")
ROM = os.path.join(ROOT, "dino.zip")


def run(args, env=None):
    """Returns the CPU seconds the run took (user + system, a steadier number than the wall time for a run of some milliseconds) and its stderr."""
    e = dict(os.environ)
    e.update(env or {})
    r0 = resource.getrusage(resource.RUSAGE_CHILDREN)
    p = subprocess.run([BIN, "--rom", ROM, "--headless", "--input-file", SCRIPT] + args, capture_output=True, text=True, env=e)
    r1 = resource.getrusage(resource.RUSAGE_CHILDREN)
    return (r1.ru_utime - r0.ru_utime) + (r1.ru_stime - r0.ru_stime), p.stderr


jumps = sys.argv[3:] or [l.split()[0] for l in subprocess.run([BIN, "--rom", ROM, "--list-jumps"], capture_output=True, text=True).stdout.splitlines() if l.strip()]
print("jump    tick us  plan avg ms  plan max ms  wait avg ms  wait max ms  late frames  default-policy frames  level changes")
for j in jumps:
    base = min(run(["--frames", "1", "--jump", j])[0] for _ in range(3))
    t = min(run(["--frames", str(FRAMES), "--jump", j])[0] for _ in range(3))
    tick_us = max(t - base, 0) / FRAMES * 1e6
    _, err = run(["--frames", str(FRAMES), "--jump", j, "--ally", "2"], {"DINO_REALTIME": "1"})
    m = re.search(r"plan in the background ([\d.]+) ms average, ([\d.]+) ms longest \| wait of the game ([\d.]+) ms average, ([\d.]+) ms longest", err)
    late = re.search(r"realtime: (\d+) of", err)
    fb = re.search(r"(\d+) frames on the default policy .*?, (\d+) changes of the search level", err)
    pa, pm, wa, wm = m.groups() if m else ("-", "-", "-", "-")
    print(f"{j:<7} {tick_us:7.1f}  {pa:>11}  {pm:>11}  {wa:>11}  {wm:>11}  {late.group(1) if late else '-':>11}  {fb.group(1) if fb else '0':>21}  {fb.group(2) if fb else '0':>13}", flush=True)
