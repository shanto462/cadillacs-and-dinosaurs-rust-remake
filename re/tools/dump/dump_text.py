"""Every string the game prints. Two formats:
  1. '@'-terminated ASCII strings (print routines 0xC0A, 0xBB6, 0xC44): names, items, menus, staff roll, HUD words.
  2. Word strings of the tall font (print routine 0x84F72): story panels, profile cards, ending, legal text.
Output: re/out/assets/text/ (text files and rendered pictures of the tall strings)."""
import json
import os
import re
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from romio import Rom, outdir, hx
from render import font

SPECIAL = {0x2727: "'", 0x5E5E: '"', 0x2C2C: ",", 0x2E2E: ".", 0x5B5B: "!", 0x5D5D: "?"}
TALL = {0x160 + i: chr(65 + i) for i in range(15)}
TALL.update({0x190 + i: chr(80 + i) for i in range(11)})
TALL.update({0x680: " ", 0x120: "!"})

REGIONS = [  # (start, end, label)
    (0x83E0, 0x8440, "credit line words (CREDIT, COIN, FREE PLAY)"),
    (0xB390, 0xB400, "character select panel: hero names and stat labels"),
    (0xBD70, 0xC100, "stage clear tally words"),
    (0x16420, 0x16450, "HUD hero names"),
    (0x17DC0, 0x17F90, "name table: heroes, enemies, bosses, dinosaurs (words printed in the HUD)"),
    (0x23BF0, 0x23DE0, "weapon names"),
    (0x240F0, 0x24320, "food names and point words"),
    (0x24450, 0x24610, "treasure names"),
    (0x32FD0, 0x33340, "enemy AI state names (debug, romaji)"),
    (0x502C0, 0x502D0, "debug word in the car stage"),
    (0x7C7E0, 0x7CC00, "stage clear tally words (second copy)"),
    (0x8B110, 0x8B140, "ending word"),
    (0x8B7E0, 0x8BC90, "staff roll"),
    (0x97000, 0x97200, "front end text"),
    (0x97F40, 0x98100, "title: copyright lines, PLAY DEMO"),
    (0x98800, 0x98900, "title: INSERT COIN / FREE PLAY"),
    (0x98D70, 0x98E50, "front end text"),
    (0x9AC00, 0x9AC50, "profile text"),
    (0x9D990, 0x9E400, "coin and credit system text"),
    (0xA4F80, 0xA5700, "story panel word strings (tall font) region"),
    (0xA6E00, 0xAAC40, "test mode menus (service menu)"),
]


def looks_like_text(s):
    t = s.strip()
    if len(t) < 3:
        return False
    letters = sum(c.isalpha() for c in t)
    if letters < 3:
        return False
    odd = sum(1 for c in t if not (c.isalnum() or c in " .,'-!?:/()$#%&"))
    if odd > 1:
        return False
    if len(t) <= 6 and any(c in "=;<>*+|\\" for c in t):
        return False
    if len(set(t)) <= 3 and len(t) > 6:
        return False
    # code bytes look like short mixed-case noise
    if len(t) <= 5 and not t.isupper() and not t.isdigit() and not t[0].isupper():
        return False
    if t in ("DA=", "NuS", "RAF", "QNu="):
        return False
    return True


def region_of(a):
    for lo, hi, lab in REGIONS:
        if lo <= a < hi:
            return lab
    return "other"


def ascii_strings(rom):
    b = rom.b
    pat = re.compile(rb"[\x20-\x3f\x41-\x7e]{3,}@")
    out = []
    for m in pat.finditer(b[:0xB7000]):
        s = m.group()[:-1].decode("latin1")
        if looks_like_text(s):
            out.append((m.start(), s, b[m.start() - 1] if m.start() else 0))
    return out


def valid_word(w):
    return (w >> 8) == 1 or w == 0x0680 or w in SPECIAL


def parse_words(rom, p):
    segs, i, total = [], p, 0
    b = rom.b
    for _ in range(40):
        if i + 4 > len(b):
            return None
        col, row, attr = b[i], b[i + 1], rom.u16(i + 2)
        if col > 0x3F or row > 0x3F or attr > 0x1F:
            return None
        i += 4
        words = []
        while True:
            if i + 2 > len(b):
                return None
            w = rom.u16(i)
            i += 2
            if w == 0x4040:
                segs.append((col, row, attr, words))
                return (segs, i) if total >= 3 else None
            if w == 0x2F2F:
                segs.append((col, row, attr, words))
                break
            if not valid_word(w):
                return None
            words.append(w)
            total += 1
            if len(words) > 80:
                return None
    return None


def word_strings(rom):
    out, p = [], 0x1000
    while p < 0xB6000:
        res = parse_words(rom, p)
        if res:
            out.append((p, res[1], res[0]))
            p = res[1]
            continue
        p += 2
    return out


def transcribe(segs):
    lines = {}
    for col, row, attr, words in segs:
        txt = ""
        for w in words:
            if w in SPECIAL:
                txt += SPECIAL[w]
            elif w in TALL:
                txt += TALL[w]
            else:
                txt += f"<{w:03X}>"
        lines.setdefault(row, []).append((col, txt))
    out = []
    for row in sorted(lines):
        out.append(" | ".join(t for c, t in sorted(lines[row])))
    return out


def render_words(rom, segs, pal):
    W, H = 64, 32
    cv = np.full((H * 8, W * 8, 3), (20, 20, 30), np.uint8)
    t8 = rom.t8

    def cell(col, row, code, attr):
        if not (0 <= col < W and 0 <= row < H):
            return
        pens = t8[code & 0xFFFF][col & 1]
        rgb = pal[attr & 0x1F][pens]
        m = pens != 15
        sub = cv[row * 8:row * 8 + 8, col * 8:col * 8 + 8]
        sub[m] = rgb[m]

    for col, row0, attr, words in segs:
        row = row0 % 32          # rows 32..63 are the second page of scroll1: shown on the same canvas
        c = col
        for w in words:
            if w == 0x2727:
                cell(c, row, 0x2C, 0)
                c += 1
            elif w == 0x5E5E:
                cell(c, row, 0x22, 0)
                c += 1
            elif w == 0x2C2C:
                cell(c + 1, row, 0x2C, 0)
                c += 2
            elif w == 0x2E2E:
                cell(c + 1, row, 0x2E, 0)
                c += 2
            elif w == 0x5B5B:
                cell(c - 1, row, 0x18F, 0)
            elif w == 0x5D5D:
                cell(c + 2, row, 0x1BF, 0)
                c += 3
            else:
                cell(c, row, w, attr)
                cell(c, row + 1, w + 0x10, attr)
                c += 1
    ys, xs = np.where((cv != (20, 20, 30)).any(axis=2))
    if len(ys) == 0:
        return None
    y0, y1, x0, x1 = max(ys.min() - 4, 0), ys.max() + 5, max(xs.min() - 4, 0), xs.max() + 5
    return cv[y0:y1, x0:x1]


def run(rom=None):
    rom = rom or Rom()
    out = outdir("text")
    imgs = outdir("text", "images")
    meta = {}
    # ---- ASCII strings
    strs = ascii_strings(rom)
    with open(os.path.join(out, "ascii_strings.txt"), "w") as f:
        f.write("# '@' terminated ASCII strings of the 68000 program. Format: address, byte before the string (attribute or separator), text\n")
        last = None
        for a, s, pre in strs:
            lab = region_of(a)
            if lab != last:
                f.write(f"\n## {lab}\n")
                last = lab
            f.write(f"{a:06X}  {pre:02X}  {s!r}\n")
    meta["ascii_strings"] = len(strs)
    # ---- name table including entries no id points to
    names = []
    refd = set()
    for i in range(0x27):
        refd.add(0x17D78 + rom.u16(0x17D78 + 2 * i))
    for i in range(4):
        refd.add(0x17D70 + rom.u16(0x17D70 + 2 * i))
    a = 0x17DC2
    with open(os.path.join(out, "names_table.txt"), "w") as f:
        f.write("# name table at 0x17DC2: every string, and whether the id table (0x17D78, 0x27 entries) or the hero table (0x17D70) points to it\n")
        while a < 0x17F90:
            end = a
            while rom.b[end] != 0x40:
                end += 1
            s = bytes(rom.b[a + 1:end]).decode("latin1")
            ids = [i for i in range(0x27) if 0x17D78 + rom.u16(0x17D78 + 2 * i) == a]
            heroes = [i for i in range(4) if 0x17D70 + rom.u16(0x17D70 + 2 * i) == a]
            tag = ("ids " + ",".join(hex(i) for i in ids)) if ids else ("hero %s" % heroes if heroes else "NOT REFERENCED by the id table")
            f.write(f"{a:06X}  {s.strip()!r:14}  {tag}\n")
            names.append((a, s.strip(), bool(ids or heroes)))
            a = end + 1
    meta["names"] = [dict(addr=a, name=s, referenced=r_) for a, s, r_ in names]
    # ---- tall font word strings
    pal = rom.bank(0xBBA52)
    ws = word_strings(rom)
    lines_out = []
    entries = []
    for k, (a, e, segs) in enumerate(ws):
        tr = transcribe(segs)
        img = render_words(rom, segs, pal)
        fn = f"w{a:06X}.png"
        if img is not None:
            Image.fromarray(img).resize((img.shape[1] * 2, img.shape[0] * 2), Image.NEAREST).save(os.path.join(imgs, fn))
        entries.append(dict(addr=a, end=e, text=tr, image=fn, segments=[(c, r_, at, len(w)) for c, r_, at, w in segs]))
        lines_out.append(f"{a:06X}-{e:06X}  (segments at col,row): " + "  ".join(f"({c},{r_})" for c, r_, at, w in segs))
        for ln in tr:
            lines_out.append("        " + ln)
    with open(os.path.join(out, "tall_font_strings.txt"), "w") as f:
        f.write("# Strings of the tall font (print routine 0x84F72): story panels, profile cards, ending and legal text.\n"
                "# Letters A to Z are decoded; <XXX> is a tile code that has no letter (kana or symbol). Pictures: images/wADDRESS.png\n\n")
        f.write("\n".join(lines_out) + "\n")
    meta["tall_strings"] = len(ws)
    with open(os.path.join(out, "tall_font_strings.json"), "w") as f:
        json.dump(entries, f)
    # ---- staff roll (ordered, one per line)
    with open(os.path.join(out, "staff_roll.txt"), "w") as f:
        f.write("# staff roll strings at 0x8B7EA (12 character fields)\n")
        for a, s, pre in strs:
            if 0x8B7E0 <= a < 0x8BC90:
                f.write(f"{a:06X}  {s.rstrip()}\n")
    with open(os.path.join(outdir("data"), "text.json"), "w") as f:
        json.dump(dict(meta=meta, ascii=[(a, s) for a, s, p in strs]), f)
    print(f"text: {len(strs)} ASCII strings, {len(ws)} tall-font strings, {len(names)} names")
    return meta


if __name__ == "__main__":
    run()
