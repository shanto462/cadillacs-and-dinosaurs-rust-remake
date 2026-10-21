# Where the stage scripts spawn each object class (for the scenes of REPORT.md section 5).
#   re/.venv/bin/python -I re/harness/coverage/where.py [CLASS ...]      CLASS like A82, B30, E5
# Reads re/specs/enemy/stage_scripts.txt (the decoded spawn lists and waves of the 8 stages) and prints, per class,
# the episodes and areas (episode = stage + 1) with the X of the first spawn, and the conditions (players>=2 ...).
import os
import re
import sys
from collections import defaultdict

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
where = defaultdict(list)
stage = area = None
for line in open(os.path.join(ROOT, "re/specs/enemy/stage_scripts.txt")):
    m = re.match(r"#+ STAGE (\d+)", line)
    if m:
        stage = int(m.group(1)); area = None; continue
    m = re.match(r"== area (\d+)", line)
    if m:
        area = int(m.group(1)); continue
    if line.startswith("== waves"):
        area = "w"; continue
    m = re.search(r"\b([ABCDE])#(\d+)\b.*?X=0x([0-9a-f]+).*?var=0x([0-9a-f]+)", line)
    if m and stage is not None:
        cond = re.search(r"players[<>=]+\d", line)
        where["%s%d" % (m.group(1), int(m.group(2)))].append(
            "E%d %s X=%s var=%s%s" % (stage, "a%s" % area if area != "w" else "wave", m.group(3), m.group(4), " " + cond.group(0) if cond else ""))
want = sys.argv[1:] or sorted(where)
for c in want:
    v = where.get(c, [])
    print("%s: %d spawns%s" % (c, len(v), ("; " + "; ".join(v[:6])) if v else ""))
