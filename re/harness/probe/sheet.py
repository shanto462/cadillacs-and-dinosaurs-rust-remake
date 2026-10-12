"""Contact sheet of the probe screenshots, with a label per picture (frame and game state).

    re/.venv/bin/python -I re/harness/probe/sheet.py OUTDIR [--cols 5] [--width 256] [--per 30]
                                                      [--from N] [--to N] [--out sheet]
    re/.venv/bin/python -I re/harness/probe/sheet.py DIR1 DIR2 ... --pick -1 --out overview [--cols 4]

With several directories and --pick K, the sheet holds one picture per directory (the K-th screenshot,
-1 = the last), labelled with the directory name; it is written next to the first directory.

OUTDIR is re/out/probe/NAME (it holds snap/ and log.txt). Writes OUTDIR/sheet.png, or sheet_1.png,
sheet_2.png .. when there are more than --per pictures. The label of picture k is read from the k-th
"SNAP" line of log.txt: t (frames after the jump), f (frames from power on) and the last state line
(mode, sub mode, episode, area). Without log.txt the file name is used.
"""
import glob
import os
import re
import sys

from PIL import Image, ImageDraw


def overview(dirs, args):
    w = args["width"]
    shots = []
    for d in dirs:
        files = sorted(glob.glob(os.path.join(d, "snap", "*.png")))
        if files:
            shots.append((os.path.basename(d.rstrip("/")), files[args["pick"]]))
    if not shots:
        print("no screenshots")
        return 1
    cols = min(args["cols"], len(shots))
    first = Image.open(shots[0][1]).convert("RGB")
    h = int(w * first.height / first.width)
    lab_h = 14
    rows = (len(shots) + cols - 1) // cols
    sheet = Image.new("RGB", (cols * w, rows * (h + lab_h)), (24, 24, 24))
    dr = ImageDraw.Draw(sheet)
    for n, (name, f) in enumerate(shots):
        im = Image.open(f).convert("RGB").resize((w, h), Image.NEAREST)
        x, y = (n % cols) * w, (n // cols) * (h + lab_h)
        sheet.paste(im, (x, y + lab_h))
        dr.text((x + 2, y + 1), name, fill=(255, 255, 80))
    path = os.path.join(os.path.dirname(dirs[0].rstrip("/")), args["out"] + ".png")
    sheet.save(path)
    print("overview:", path, sheet.size, "(%d pictures)" % len(shots))
    return 0


def main(argv):
    args = {"cols": 5, "width": 256, "per": 30, "from": 0, "to": None, "out": "sheet", "pick": None}
    pos = []
    i = 0
    while i < len(argv):
        a = argv[i]
        if a.startswith("--") and a[2:] in args:
            v = argv[i + 1]
            args[a[2:]] = v if a[2:] == "out" else int(v)
            i += 2
        else:
            pos.append(a)
            i += 1
    if not pos:
        print(__doc__)
        return 1
    if args["pick"] is not None:
        return overview(pos, args)
    out_dir = pos[0]
    snap_dir = os.path.join(out_dir, "snap")
    if not os.path.isdir(snap_dir):
        snap_dir = out_dir
        out_dir = os.path.dirname(snap_dir.rstrip("/"))
    files = sorted(glob.glob(os.path.join(snap_dir, "*.png")) + glob.glob(os.path.join(snap_dir, "*", "*.png")))
    if not files:
        print("no screenshots in", snap_dir)
        return 1

    # labels from the log: the k-th SNAP line belongs to the k-th file
    labels = {}
    log = os.path.join(out_dir, "log.txt")
    if os.path.exists(log):
        state = ""
        for line in open(log, errors="replace"):
            m = re.match(r"t=(\S+) f=(\d+) gf=(\d+) (\S+) (.*)", line.rstrip("\n"))
            if not m:
                continue
            t, f, gf, tag, rest = m.groups()
            if tag == "S":
                mm = re.search(r"mode=(\w+) sub=(\w+) stage=(\d+) area=(\d+)", rest)
                if mm:
                    state = "m%s s%s E%d a%s" % (mm.group(1), mm.group(2), int(mm.group(3)) + 1, mm.group(4))
            elif tag == "SNAP":
                k = int(rest.split()[0])
                labels[k] = "#%d t=%s f=%s  %s" % (k, t, f, state)

    lo = args["from"]
    hi = args["to"] if args["to"] is not None else len(files)
    sel = list(range(lo, min(hi, len(files))))
    w = args["width"]
    cols = args["cols"]
    first = Image.open(files[0]).convert("RGB")
    h = int(w * first.height / first.width)
    lab_h = 14
    per = args["per"]
    sheets = [sel[j:j + per] for j in range(0, len(sel), per)]
    for si, chunk in enumerate(sheets):
        cols = min(args["cols"], len(chunk))
        rows = (len(chunk) + cols - 1) // cols
        sheet = Image.new("RGB", (cols * w, rows * (h + lab_h)), (24, 24, 24))
        dr = ImageDraw.Draw(sheet)
        for n, idx in enumerate(chunk):
            im = Image.open(files[idx]).convert("RGB").resize((w, h), Image.NEAREST)
            x = (n % cols) * w
            y = (n // cols) * (h + lab_h)
            sheet.paste(im, (x, y + lab_h))
            dr.text((x + 2, y + 1), labels.get(idx, os.path.basename(files[idx])), fill=(255, 255, 80))
        name = args["out"] + (".png" if len(sheets) == 1 else "_%d.png" % (si + 1))
        path = os.path.join(out_dir, name)
        sheet.save(path)
        print("contact sheet:", path, sheet.size, "(%d pictures)" % len(chunk))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
