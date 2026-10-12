"""Run the recipes of plan/game/design/capture-recipes.md in ONE MAME process.

    re/.venv/bin/python -I re/harness/probe/recipes.py --list
    re/.venv/bin/python -I re/harness/probe/recipes.py NAME [NAME ...]      # these recipes, one MAME run
    re/.venv/bin/python -I re/harness/probe/recipes.py --all                # every recipe, one MAME run
    re/.venv/bin/python -I re/harness/probe/recipes.py --prefix fe- area-   # all recipes with these prefixes

A recipe is a fenced block in the document that starts with "```recipe NAME". Its lines are probe input
lines (key lines and directives, including `frames N` and `snaps SPEC`). All chosen recipes become segments
of one input file and run in one MAME process (the power-on state is loaded again for every segment).
Output: re/out/probe/recipes/NAME/{log.txt,snap/,sheet.png} for --all, re/out/probe/recipes-part/NAME/ for any
other call (so a small run never erases the full run); recipes whose names start with "area-" also get one
overview picture _areas.png in that folder.
"""
import os
import re
import subprocess
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
DOC = os.path.join(ROOT, "plan", "game", "design", "capture-recipes.md")


def load():
    text = open(DOC).read()
    out, order = {}, []
    for m in re.finditer(r"```recipe (\S+)\n(.*?)```", text, re.S):
        out[m.group(1)] = m.group(2)
        order.append(m.group(1))
    return out, order


def main(argv):
    recipes, order = load()
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__)
        return 1
    if argv[0] == "--list":
        print("\n".join(order))
        return 0
    if argv[0] == "--all":
        names = order
    elif argv[0] == "--prefix":
        names = [n for n in order if any(n.startswith(p) for p in argv[1:])]
    else:
        names = argv
    for n in names:
        if n not in recipes:
            print("unknown recipe", n)
            return 1
    d = os.path.join(ROOT, "re", "out", "probe", "_input")
    os.makedirs(d, exist_ok=True)
    path = os.path.join(d, "recipes.txt")
    with open(path, "w") as f:
        for n in names:
            f.write("segment %s\n%s\n" % (n, recipes[n].strip("\n")))
    env = dict(os.environ)
    env["PR_INPUT"] = path
    env.setdefault("PR_TIMEOUT", "3000")
    batch = "recipes" if argv[0] == "--all" else "recipes-part"     # a part run must not erase the full run
    subprocess.run([os.path.join(ROOT, "re", "harness", "probe", "probe.sh"), batch], env=env, check=False)
    base = os.path.join(ROOT, "re", "out", "probe", batch)
    areas = [os.path.join(base, n) for n in names if n.startswith("area-")]
    if areas:
        subprocess.run([os.path.join(ROOT, "re", ".venv", "bin", "python"), "-I",
                        os.path.join(ROOT, "re", "harness", "probe", "sheet.py")] + areas +
                       ["--pick", "-1", "--out", "_areas", "--cols", "4", "--width", "384"], check=False)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
