"""List the rules of re/harness/allow/m10_runs.txt that no longer allow any difference (Task 26).

Usage: re/.venv/bin/python -I re/harness/unused_rules.py [--port-out DIR] [--modes step,lagsync,run] [--jobs N] [--remove] [NAME ...]
       re/.venv/bin/python -I re/harness/unused_rules.py --demo [--port-out DIR] [--remove]

For every recording NAME (default: every recording with a step, lagsync, run or * rule) and mode, the rules are cut out as
m10_check.sh does it (the lines "NAME.MODE:" and "NAME.*:", then the lines without a recording), and diff.py compares
re/out/traces/NAME.mame.trc with the port trace of the last m10_check.sh / m10_all.sh run (PORT_OUT, default re/out/port:
NAME.step.trc, NAME.lagsync.trc, NAME.port.trc) with `--used`: the rules that allowed a difference. A difference counts
for the first rule that matches it (as diff.py's `Allow.ok`), so a rule that is covered by an earlier one is unused too.

A rule is unused when every mode it applies to (a `*` rule: step, lagsync and run) was checked and no difference used it.
A rule with a mode that was not checked (no port trace, or not in --modes) is "not checked". Left out: the `.boot` rules
(the flow checks) and the `sound.*` keys (snd_stream.py, m10_sound_check.sh: diff.py does not read them).
Run m10_all.sh (or m10_check.sh) first, with the same build: the port traces must be current.
--remove deletes the unused lines from m10_runs.txt (the comments stay; mark a cause "(gone)" by hand).

--demo: the rules of re/harness/allow/m10.txt against the free run of m10_demo_check.sh (PORT_OUT/m1003-loop.port.trc, run that script
first): diff.py for each of its six demo windows (A .. A + 1326) with `--used`, and the window A = 3343 of the two poked first runs
(`flow_m112-row5` / `-row0`, the header of m10.txt): PORT_OUT/m112-row5.port.trc and m112-row0.port.trc, made by
    dino-game --rom dino.zip --poke 1000:FFF57A:2:4 --trace PORT_OUT/m112-row5.port.trc --frames 4900   (row0: value 5)
A rule is unused when no window used it; without the first-run traces the result says so and nothing is removed.
"""

import argparse
import os
import re
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, "..", ".."))
RULES = os.path.join(HERE, "allow", "m10_runs.txt")
DEMO_RULES = os.path.join(HERE, "allow", "m10.txt")
DEMOS = (3343, 9914, 16448, 23019, 29553, 36124)  # the demo calls of the loop recording (m10_demo_check.sh)
FIRST_RUNS = ("row5", "row0")  # flow_m112-*: G.$757A poked at power frame 1000, the first demo at A = 3343
CHECKED = ("step", "lagsync", "run")
PREFIX = re.compile(r"^([a-z0-9_]+)\.([a-z*]+):")


def parse(path):
    """(line number, name, mode, rule text) for the lines with a recording; (line number, rule text) for the others."""
    named, plain = [], []
    with open(path) as f:
        for no, line in enumerate(f, 1):
            m = PREFIX.match(line)
            if m:
                named.append((no, m.group(1), m.group(2), line[m.end():].strip()))
            elif line.strip() and not line.startswith("#"):
                plain.append((no, line.strip()))
    return named, plain


def key_of(text):
    return text.split("#")[0].split()[0] if text.split("#")[0].strip() else ""


def check(name, mode, named, plain, port_out, tmp):
    """Run diff.py for one recording and mode: (the m10_runs.txt lines used, the summary line with the rules) or None (no trace)."""
    mame = os.path.join(ROOT, "re", "out", "traces", f"{name}.mame.trc")
    port = os.path.join(port_out, f"{name}.{'port' if mode == 'run' else mode}.trc")
    if not (os.path.exists(mame) and os.path.exists(port)):
        return None
    lines = [(no, text) for no, n, m, text in named if n == name and m in (mode, "*")] + plain  # the order of m10_check.sh's cut
    allow = os.path.join(tmp, f"{name}.{mode}.allow")
    used = os.path.join(tmp, f"{name}.{mode}.used")
    with open(allow, "w") as f:
        f.write("".join(text + "\n" for _, text in lines))
    out = subprocess.run([sys.executable, "-I", os.path.join(HERE, "diff.py"), mame, port, "--allow", allow, "--used", used],
                         capture_output=True, text=True).stdout
    summary = next((ln for ln in out.splitlines() if ln.startswith("SUMMARY")), "SUMMARY: ?")
    with open(used) as f:
        hit = {lines[int(x) - 1][0] for x in f.read().split()}
    return hit, summary


def remove_lines(path, drop):
    with open(path) as f:
        old = f.readlines()
    with open(path, "w") as f:
        f.writelines(line for i, line in enumerate(old, 1) if i not in drop)
    print(f"removed {len(drop)} lines from {path}")


def demo(port_out, remove):
    """The rules of allow/m10.txt that no demo window of the free run uses."""
    traces = os.path.join(ROOT, "re", "out", "traces")
    windows = [("loop", os.path.join(traces, "flow_m1003-loop.mame.trc"), os.path.join(port_out, "m1003-loop.port.trc"), a) for a in DEMOS]
    windows += [(r, os.path.join(traces, f"flow_m112-{r}.mame.trc"), os.path.join(port_out, f"m112-{r}.port.trc"), 3343) for r in FIRST_RUNS]
    hit, missing = set(), []
    with tempfile.TemporaryDirectory(dir=os.path.join(ROOT, "re", "out")) as tmp:
        for k, (run, mame, port, a) in enumerate(windows):
            if not (os.path.exists(mame) and os.path.exists(port)):
                missing.append(f"{run} {a}")
                continue
            used = os.path.join(tmp, f"{k}.used")
            out = subprocess.run([sys.executable, "-I", os.path.join(HERE, "diff.py"), mame, port, "--from", str(a), "--frames", str(a + 1327),
                                  "--allow", DEMO_RULES, "--used", used], capture_output=True, text=True).stdout
            print(f"{run} demo {a}: " + next((ln for ln in out.splitlines() if ln.startswith("RESULT")), "RESULT ?"))
            with open(used) as f:
                hit |= {int(x) for x in f.read().split()}
    unused = []
    with open(DEMO_RULES) as f:
        for no, line in enumerate(f, 1):
            if line.split("#")[0].strip() and no not in hit:
                unused.append(no)
                print(f"{no:5d}  UNUSED  {line.rstrip()}")
    print(f"\n{len(unused)} unused rules of {DEMO_RULES}")
    if missing:
        print("not checked (no trace): " + ", ".join(missing) + "; nothing is removed")
        return
    if remove and unused:
        remove_lines(DEMO_RULES, set(unused))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("names", nargs="*")
    ap.add_argument("--port-out", default=os.environ.get("PORT_OUT", os.path.join(ROOT, "re", "out", "port")))
    ap.add_argument("--modes", default="step,lagsync,run")
    ap.add_argument("--jobs", type=int, default=4)
    ap.add_argument("--remove", action="store_true", help="delete the unused lines from m10_runs.txt")
    ap.add_argument("--demo", action="store_true", help="check allow/m10.txt against the demo windows of m10_demo_check.sh instead")
    args = ap.parse_args()
    if args.demo:
        demo(args.port_out, args.remove)
        return
    modes = [m for m in args.modes.split(",") if m in CHECKED]

    named, plain = parse(RULES)
    rules = [r for r in named if r[2] in CHECKED + ("*",) and not key_of(r[3]).startswith("sound.")]
    names = args.names or sorted({r[1] for r in rules})
    jobs = [(n, m) for n in names for m in modes]
    with tempfile.TemporaryDirectory(dir=os.path.join(ROOT, "re", "out")) as tmp, ThreadPoolExecutor(args.jobs) as ex:
        results = dict(zip(jobs, ex.map(lambda j: check(j[0], j[1], named, plain, args.port_out, tmp), jobs)))

    for (n, m), r in sorted(results.items()):
        print(f"{n}.{m}: " + ("no port trace" if r is None else f"{len(r[0])} rules used; with the rules {r[1]}"))
    unused = []
    print()
    for no, n, m, text in rules:
        if n not in names:
            continue
        applies = CHECKED if m == "*" else (m,)
        done = [a for a in applies if results.get((n, a)) is not None]
        used = any(no in results[(n, a)][0] for a in done)
        if used:
            state = "used"
        elif len(done) < len(applies):
            state = "not checked (" + ", ".join(a for a in applies if a not in done) + ")"
        else:
            state = "UNUSED"
            unused.append(no)
        if state != "used":
            print(f"{no:5d}  {state:28s} {n}.{m}: {text}")
    print(f"\n{len(unused)} unused rules")
    if args.remove and unused:
        remove_lines(RULES, set(unused))


if __name__ == "__main__":
    main()
