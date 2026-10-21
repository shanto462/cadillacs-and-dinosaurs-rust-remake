# Join probe batch files (`segment NAME` blocks, re/harness/probe/probe.lua) into one, so ONE MAME process replays them all.
#   re/.venv/bin/python -I re/harness/coverage/merge.py [--last-wins] [--skip NAME,NAME] OUT.txt BATCH.txt [BATCH.txt ...]
# Lines before the first `segment` line of a file belong to every segment of that file (probe.lua), so they are copied
# into each of its segments. A segment name that is already taken stops the merge; with --last-wins the segment of the
# later file replaces the earlier one (for old batch files where a later file recorded the segment again). --skip leaves
# segments out (a replay that never ends).
import sys

last_wins = sys.argv[1] == "--last-wins"
if last_wins:
    del sys.argv[1]
skip = set()
if sys.argv[1] == "--skip":
    skip = set(sys.argv[2].split(","))
    del sys.argv[1:3]
out, names = [], set()
blocks = {}      # with --last-wins: name -> its lines, in the order of the first appearance
for path in sys.argv[2:]:
    pre, cur = [], None
    for line in open(path):
        line = line.rstrip("\n")
        if line.strip().startswith("segment "):
            name = line.split()[1]
            if name in names and not last_wins:
                sys.exit("merge.py: segment %s of %s is already taken" % (name, path))
            names.add(name)
            cur = name
            blocks[name] = [line] + list(pre)
            continue
        elif cur is None:
            if line.strip() and not line.lstrip().startswith("#"):
                pre.append(line)
        else:
            blocks[cur].append(line)
for name, lines in blocks.items():
    if name not in skip:
        out.extend(lines)
open(sys.argv[1], "w").write("\n".join(out) + "\n")
print("merge.py: %d segments from %d files -> %s" % (len(names - skip), len(sys.argv) - 2, sys.argv[1]))
