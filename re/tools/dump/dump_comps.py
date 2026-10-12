"""Renders every sprite composition (3513) as contact sheets grouped by owner. Output: re/out/assets/comps/."""
import json
import os
import sys

import numpy as np
from PIL import Image, ImageDraw

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from romio import Rom, outdir, hx
from render import Painter, to_pil, text, BG
import catalog as cat_mod

KIND_PRIORITY = {"hero": 0, "boss": 1, "dinosaur": 1, "enemy": 1, "actor": 1, "item": 2, "prop": 3, "effect": 4, "frontend": 5, "unknown": 9}


def comp_owners(cat):
    """{comp byte index: (kind, group, label, pal_set, table)} using the best owner among all tables that draw it."""
    best = {}
    for t, d in sorted(cat["tables"].items()):
        key = (KIND_PRIORITY.get(d["kind"], 9), t)
        for f in d["frames"]:
            pass
    rom = cat["_rom"]
    users = {}
    for t, d in sorted(cat["tables"].items()):
        pr = KIND_PRIORITY.get(d["kind"], 9)
        for f in d["frames"]:
            for ci in cat_mod.frame_comps(rom, f):
                if ci not in users or (pr, t) < users[ci][0]:
                    users[ci] = ((pr, t), t)
    for v, d in cat["single"].items():
        for f in d["chain"]:
            for ci in cat_mod.frame_comps(rom, f):
                if ci not in users:
                    users[ci] = ((8, v), -v)
    out = {}
    for ci, (_, t) in users.items():
        if t > 0:
            d = cat["tables"][t]
            out[ci] = (d["kind"], d["group"], d["label"], d["pal_set"], t)
        else:
            d = cat["single"][-t]
            out[ci] = (d["kind"], d["group"], d["label"], d["pal_set"], -t)
    return out


def run(rom=None, cat=None, painter=None, only=None):
    rom = rom or Rom()
    if cat is None:
        cat, _ = cat_mod.build(rom, verbose=False)
    cat["_rom"] = rom
    P = painter or Painter(rom)
    owners = comp_owners(cat)
    n = P.n_comp
    root = outdir("comps")
    # neighbours for unreferenced comps
    referenced = sorted(owners)
    def neighbor(ci):
        import bisect
        i = bisect.bisect_left(referenced, ci)
        lo = referenced[i - 1] if i > 0 else None
        hi = referenced[i] if i < len(referenced) else None
        return lo, hi
    groups = {}
    meta = []
    for k in range(n):
        ci = k * 4
        if ci in owners:
            kind, group, label, pal_set, tbl = owners[ci]
            ref = True
        else:
            lo, hi = neighbor(ci)
            a = owners.get(lo)
            b = owners.get(hi)
            hint = ""
            if a and b and a[1] == b[1]:
                hint = f"between neighbours of {a[2]}"
            elif a and b:
                hint = f"between {a[2]} and {b[2]}"
            elif a or b:
                hint = f"after {(a or b)[2]}"
            kind, group, label, pal_set = "unreferenced", "unreferenced", hint or "unreferenced", (a or b or (None,) * 4)[3]
            tbl = None
            ref = False
        groups.setdefault(group, []).append((ci, label, pal_set))
        meta.append(dict(index=k, byte_index=ci, addr=P.comp(ci)["addr"], kind=kind, group=group, label=label, referenced=ref,
                         entries=len(P.comp(ci)["ents"]), table=tbl, pal_set=pal_set))
    # sheets
    total_sheets = 0
    index = {}
    for group, items in sorted(groups.items()):
        gdir = os.path.join(root, group)
        os.makedirs(gdir, exist_ok=True)
        rendered = []
        for ci, label, pal_set in items:
            st, ar = pal_set if pal_set else (0, 0)
            pals, nums = P.obj_set(st, ar)
            spr = P.comp_sprites(ci)
            img, ox, oy = P.render(spr, pals, margin=1)
            rendered.append((ci, img, st, ar))
        # sheets of up to 40 comps
        files = []
        per = 40
        for s0 in range(0, len(rendered), per):
            chunk = rendered[s0:s0 + per]
            mw = max(c[1].shape[1] for c in chunk)
            mh = max(c[1].shape[0] for c in chunk)
            sc = 2 if max(mw, mh) <= 72 else 1
            cw = min(max(mw * sc + 8, 110), 230)
            chh = min(max(mh * sc + 28, 80), 260)
            cols = max(1, min(8, 1800 // cw))
            rows = (len(chunk) + cols - 1) // cols
            sheet = Image.new("RGB", (cols * cw, rows * chh), BG)
            d = ImageDraw.Draw(sheet)
            for i, (ci, img, st, ar) in enumerate(chunk):
                r, c = divmod(i, cols)
                x0, y0 = c * cw, r * chh
                d.rectangle([x0, y0, x0 + cw - 1, y0 + chh - 1], outline=(78, 82, 108))
                pim = to_pil(img, scale=sc)
                if pim.width > cw - 4 or pim.height > chh - 26:
                    f = min((cw - 4) / pim.width, (chh - 26) / pim.height)
                    pim = pim.resize((max(1, int(pim.width * f)), max(1, int(pim.height * f))), Image.NEAREST)
                sheet.paste(pim, (x0 + (cw - pim.width) // 2, y0 + 24 + (chh - 26 - pim.height) // 2))
                text(d, (x0 + 3, y0 + 1), f"#{ci // 4}  b{ci:04X}", size=10)
                text(d, (x0 + 3, y0 + 11), f"@{P.comp(ci)['addr']:06X}  s{st}a{ar}", fill=(255, 255, 160), size=9)
            name = f"sheet_{s0 // per + 1:02d}.png"
            sheet.save(os.path.join(gdir, name), optimize=False)
            files.append(dict(file=f"{group}/{name}", first=chunk[0][0] // 4, last=chunk[-1][0] // 4, n=len(chunk)))
            total_sheets += 1
        index[group] = dict(n=len(items), sheets=files, label=items[0][1])
    with open(os.path.join(outdir("data"), "comps.json"), "w") as f:
        json.dump(dict(groups=index, comps=meta), f)
    print(f"comps: {n} compositions, {len(groups)} groups, {total_sheets} sheets, {sum(1 for m in meta if not m['referenced'])} unreferenced")
    return index


if __name__ == "__main__":
    run()
