"""Percent of different pixels: MAME probe snapshots against port screenshots.

    re/.venv/bin/python -I re/harness/probe/shotdiff.py MAME_DIR PORT_DIR --frames LIST
        [--clock t|f] [--worst N] [--cols 1] [--width 192] [--per 8] [--out PATH]

MAME_DIR is a probe folder from capture-recipes section 5: log.txt plus snap/0000.png ..
(384 x 224). Each SNAP line maps a file to a frame. PORT_DIR is the same kind of folder,
or a folder of pictures named by frame (f00100.png, 0100.png, 100.png, frame_100.png).
A single png file is the port picture when --frames names one frame.

--frames LIST   100,250,330 or every:10:4600:6500 (same text as a probe snaps line,
                both ends included). With no --frames, every frame present on both
                sides is compared.
--clock         t (default) is the frame in the input file (t= in the log). f is the
                video frame from power on. They match when the probe has no jump.
--worst N       how many of the worst frames go on the contact sheet (default 8).
                0 skips the sheet. The sheet is MAME on the left, the port on the right.
--out PATH      sheet path (default: PORT_DIR/shotdiff.png). More than --per pairs
                splits into shotdiff_1.png, shotdiff_2.png, ..

A picture that is an integer multiple of the other (the port window is 3 times
384 x 224) is reduced first: the center pixel of each block when the scale is odd,
the top left pixel when the scale is even. Prints one line per frame. Exit 0 when
every frame is 0 percent. Exit 1 when a pixel differs or a picture is missing.
Exit 2 when the arguments are wrong.
"""
import os
import re
import sys

import numpy as np
from PIL import Image, ImageDraw


LINE_RE = re.compile(r"t=(\S+) f=(\d+) gf=(\d+) (\S+) (.*)")
EVERY_RE = re.compile(r"^every:(\d+):(\d+):(\d+)$")


def parse_frames(spec):
    spec = spec.strip()
    every = EVERY_RE.match(spec)
    if every:
        step, start, end = (int(v) for v in every.groups())
        if step < 1 or end < start:
            raise ValueError("bad --frames %r" % spec)
        return list(range(start, end + 1, step))
    frames = []
    for part in spec.replace(" ", "").split(","):
        if part == "":
            continue
        if not part.isdigit():
            raise ValueError("bad --frames %r" % spec)
        frames.append(int(part))
    if not frames:
        raise ValueError("bad --frames %r" % spec)
    return frames


def name_frame(stem):
    """(rank, frame) from a file stem, or None. Lower rank wins when two names share a frame."""
    m = re.match(r"^f(\d+)$", stem, re.I)
    if m:
        return 0, int(m.group(1))
    m = re.match(r"^frame_(\d+)$", stem, re.I)
    if m:
        return 1, int(m.group(1))
    m = re.match(r"^(\d+)$", stem)
    if m:
        return 2, int(m.group(1))
    return None


def resolve_root(path):
    path = os.path.abspath(path.rstrip(os.sep))
    if os.path.isfile(os.path.join(path, "log.txt")):
        return path
    if os.path.basename(path) == "snap":
        parent = os.path.dirname(path)
        if os.path.isfile(os.path.join(parent, "log.txt")):
            return parent
    return path


def pngs_in(directory):
    found = []
    if not os.path.isdir(directory):
        return found
    for name in sorted(os.listdir(directory)):
        if name.lower().endswith(".png"):
            found.append(os.path.join(directory, name))
    snap = os.path.join(directory, "snap")
    if os.path.isdir(snap):
        for name in sorted(os.listdir(snap)):
            if name.lower().endswith(".png"):
                found.append(os.path.join(snap, name))
    return found


def index_log(root, clock):
    """Frame to snap/NNNN.png from SNAP lines. Probe files are numbered by snap order, not by frame."""
    log = os.path.join(root, "log.txt")
    if not os.path.isfile(log):
        return {}
    found = {}
    for line in open(log, errors="replace"):
        m = LINE_RE.match(line.rstrip("\n"))
        if not m or m.group(4) != "SNAP":
            continue
        t_s, f_s, _gf, _tag, rest = m.groups()
        if clock == "f":
            frame = int(f_s)
        elif t_s == "-":
            continue
        else:
            frame = int(t_s)
        idx = int(rest.split()[0])
        found[frame] = os.path.join(root, "snap", "%04d.png" % idx)
    return found


def index_names(root):
    best = {}
    for png in pngs_in(root):
        parsed = name_frame(os.path.splitext(os.path.basename(png))[0])
        if not parsed:
            continue
        rank, frame = parsed
        old = best.get(frame)
        if old is None or rank < old[0]:
            best[frame] = (rank, png)
    return {frame: path for frame, (_rank, path) in best.items()}


def index_dir(path, clock):
    """Return (root, {frame: png}). A probe log wins over numeric file names (0000.png is snap 0)."""
    if os.path.isfile(path):
        return os.path.dirname(os.path.abspath(path)), None
    root = resolve_root(path)
    logged = index_log(root, clock)
    if logged:
        return root, logged
    return root, index_names(root)


def load_rgb(path):
    im = Image.open(path)
    try:
        return np.asarray(im.convert("RGB"))
    finally:
        im.close()


def downsample(arr, scale):
    if scale == 1:
        return arr
    # Odd scale: the center of the block is the game pixel. Even scale has no single center.
    off = (scale - 1) // 2
    return np.ascontiguousarray(arr[off::scale, off::scale])


def fit_pair(mame, port):
    """Return both pictures at the same size, plus a short note when a scale was removed."""
    if mame.shape == port.shape:
        return mame, port, None
    mh, mw = mame.shape[:2]
    ph, pw = port.shape[:2]
    if mw > 0 and mh > 0 and pw % mw == 0 and ph % mh == 0:
        scale = pw // mw
        if scale == ph // mh and scale > 1 and downsample(port, scale).shape == mame.shape:
            return mame, downsample(port, scale), "port %dx%d sampled to %dx%d (x%d)" % (pw, ph, mw, mh, scale)
    if pw > 0 and ph > 0 and mw % pw == 0 and mh % ph == 0:
        scale = mw // pw
        if scale == mh // ph and scale > 1 and downsample(mame, scale).shape == port.shape:
            return downsample(mame, scale), port, "mame %dx%d sampled to %dx%d (x%d)" % (mw, mh, pw, ph, scale)
    resized = np.asarray(Image.fromarray(port, "RGB").resize((mw, mh), Image.NEAREST))
    return mame, resized, "port %dx%d resized to %dx%d" % (pw, ph, mw, mh)


def percent(mame, port):
    differ = np.any(mame != port, axis=-1)
    count = int(differ.sum())
    total = int(differ.size)
    return count, total, (100.0 * count / total) if total else 0.0


def fmt_pct(count, total):
    if count == 0 or total == 0:
        return "0.00%"
    pct = 100.0 * count / total
    text = "%.2f%%" % pct
    if text == "0.00%":
        text = "%.4f%%" % pct
    return text


def sheet_paths(out, n):
    if n == 1:
        return [out]
    base, ext = os.path.splitext(out)
    ext = ext or ".png"
    return ["%s_%d%s" % (base, i + 1, ext) for i in range(n)]


def write_sheets(rows, out, width, cols, per):
    if not rows:
        return []
    chunks = [rows[i:i + per] for i in range(0, len(rows), per)]
    paths = sheet_paths(out, len(chunks))
    parent = os.path.dirname(out)
    if parent:
        os.makedirs(parent, exist_ok=True)
    sample = rows[0]["mame"]
    ih = int(width * sample.shape[0] / sample.shape[1])
    lab_h = 14
    gap = 4
    for path, chunk in zip(paths, chunks):
        use_cols = min(cols, len(chunk))
        cell_w = width * 2 + gap
        rows_n = (len(chunk) + use_cols - 1) // use_cols
        sheet = Image.new("RGB", (use_cols * cell_w, rows_n * (ih + lab_h)), (24, 24, 24))
        dr = ImageDraw.Draw(sheet)
        for n, row in enumerate(chunk):
            x = (n % use_cols) * cell_w
            y = (n // use_cols) * (ih + lab_h)
            label = "#%d  %s   MAME | port" % (row["frame"], fmt_pct(row["count"], row["total"]))
            fill = (255, 255, 80) if row["count"] == 0 else (255, 120, 80)
            dr.text((x + 2, y + 1), label, fill=fill)
            for side, dx in (("mame", 0), ("port", width + gap)):
                im = Image.fromarray(row[side], "RGB").resize((width, ih), Image.NEAREST)
                sheet.paste(im, (x + dx, y + lab_h))
        sheet.save(path)
        print("contact sheet:", path, sheet.size, "(%d pictures)" % len(chunk))
    return paths


def main(argv):
    args = {"frames": None, "clock": "t", "worst": 8, "cols": 1, "width": 192, "per": 8, "out": None}
    pos = []
    i = 0
    while i < len(argv):
        a = argv[i]
        if a in ("-h", "--help"):
            print(__doc__)
            return 2
        if a.startswith("--") and a[2:] in args:
            if i + 1 >= len(argv):
                print("missing value for", a)
                return 2
            args[a[2:]] = argv[i + 1]
            i += 2
        else:
            pos.append(a)
            i += 1
    if len(pos) != 2:
        print(__doc__)
        return 2
    if args["clock"] not in ("t", "f"):
        print("bad --clock", args["clock"])
        return 2
    for key in ("worst", "cols", "width", "per"):
        try:
            args[key] = int(args[key])
        except (TypeError, ValueError):
            print("bad --%s" % key, args[key])
            return 2
        if args[key] < 0 or (key != "worst" and args[key] < 1):
            print("bad --%s" % key, args[key])
            return 2

    mame_arg, port_arg = pos
    for path in (mame_arg, port_arg):
        if not os.path.exists(path):
            print("no such path:", path)
            return 2
    try:
        frames = parse_frames(args["frames"]) if args["frames"] else None
    except ValueError as err:
        print(err)
        return 2

    mame_root, mame_map = index_dir(mame_arg, args["clock"])
    port_root, port_map = index_dir(port_arg, args["clock"])
    if mame_map is None:
        if frames is None or len(frames) != 1:
            print("port or mame path is one file; pass a folder, or one --frames value")
            return 2
        mame_map = {frames[0]: os.path.abspath(mame_arg)}
    if port_map is None:
        if frames is None or len(frames) != 1:
            print("port path is one file; pass a folder, or one --frames value")
            return 2
        port_map = {frames[0]: os.path.abspath(port_arg)}
    if frames is None:
        frames = sorted(set(mame_map) & set(port_map))
        if not frames:
            print("no shared frames in", mame_root, "and", port_root)
            return 2

    notes = []
    compared = []
    missing = 0
    for frame in frames:
        mp = mame_map.get(frame)
        pp = port_map.get(frame)
        mame_ok = bool(mp) and os.path.isfile(mp)
        port_ok = bool(pp) and os.path.isfile(pp)
        if not mame_ok or not port_ok:
            who = "mame and port" if not mame_ok and not port_ok else ("mame" if not mame_ok else "port")
            print("frame %d  MISSING %s" % (frame, who))
            missing += 1
            continue
        mame, port, note = fit_pair(load_rgb(mp), load_rgb(pp))
        if note and note not in notes:
            notes.append(note)
            print("size:", note)
        count, total, pct = percent(mame, port)
        print("frame %d  %s  %d/%d" % (frame, fmt_pct(count, total), count, total))
        compared.append({
            "frame": frame, "count": count, "total": total, "pct": pct,
            "mame": mame, "port": port,
        })

    by_frame = {}
    for row in compared:
        by_frame[row["frame"]] = row
    failures = []
    for frame in sorted(set(frames)):
        row = by_frame.get(frame)
        if row is None:
            failures.append((frame, "missing"))
        elif row["count"]:
            failures.append((frame, "diff"))
    if not failures:
        print("first difference: none")
        n = len(by_frame)
        print("all equal (0 percent): %d frame%s" % (n, "" if n == 1 else "s"))
    else:
        frame, kind = failures[0]
        if kind == "missing":
            print("first difference: frame %d missing" % frame)
        else:
            print("first difference: frame %d" % frame)
        differed = [by_frame[f] for f, k in failures if k == "diff"]
        if differed:
            worst = max(differed, key=lambda row: (row["pct"], -row["frame"]))
            print("worst: frame %d  %s" % (worst["frame"], fmt_pct(worst["count"], worst["total"])))
        else:
            print("no pixel compare (%d missing)" % missing)

    if args["worst"] and by_frame:
        ranked = sorted(by_frame.values(), key=lambda row: (-row["pct"], row["frame"]))
        show = ranked[:args["worst"]]
        out = args["out"] or os.path.join(port_root, "shotdiff.png")
        write_sheets(show, out, args["width"], args["cols"], args["per"])

    if failures:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
