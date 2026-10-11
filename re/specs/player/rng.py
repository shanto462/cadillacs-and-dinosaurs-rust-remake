# rng.py <start> <end> : print sweep.txt lines in [start,end)
import sys
s = int(sys.argv[1], 16); e = int(sys.argv[2], 16)
for line in open("/Volumes/Shanto/Repos/Personal/Github/roms/re/out/sweep.txt"):
    try: a = int(line[:6], 16)
    except ValueError: continue
    if a >= e: break
    if a >= s: sys.stdout.write(line)
