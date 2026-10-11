import sys
lo=int(sys.argv[2],16); hi=int(sys.argv[3],16)
with open(sys.argv[1]) as f:
    for line in f:
        try: a=int(line[:6],16)
        except: continue
        if a>=hi: break
        if a>=lo: sys.stdout.write(line)
