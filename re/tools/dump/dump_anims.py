"""Renders every animation of every animation table: a GIF with the real frame durations (ticks of 1/60 s)
and a strip PNG with hit boxes, labelled with table address and animation id. Output: re/out/assets/anims/<owner>/."""
import json
import os
import sys
import time

import numpy as np
from PIL import Image, ImageDraw

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from romio import Rom, outdir, hx
from render import Painter, to_pil, text, save_gif, BG, GRID
import catalog as cat_mod

TICK_MS = 1000.0 / 60.0
COL_BODY, COL_ATK, COL_PROBE = (70, 255, 70), (255, 60, 60), (255, 170, 0)


def records(rom, start, stride, limit=400):
    out, p = [], start
    while len(out) < limit:
        out.append(p)
        if rom.b[p + 7] & 0x80:
            return out, p + stride + rom.i16(p + stride)
        p += stride
    return out, None


def box_rects(rom, table, idx, facing=0):
    """List of (x0, y0, x1, y1) relative to the object origin (y down), for one box index."""
    if not table or not idx:
        return []
    out = []
    if table == 0x106000:                     # multi-box table (dinosaurs): u16 offsets, then n-1, boxes
        p = table + rom.i16(table + 2 * idx)
        n = rom.u16(p) + 1
        boxes = [p + 2 + 12 * k for k in range(min(n, 8))]
    else:
        boxes = [table + 12 * idx]
    for a in boxes:
        if a + 12 > len(rom.b):
            continue
        zoff, zsize, xoff, xsize, yoff, ysize = (rom.i16(a + 2 * k) for k in range(6))
        if abs(xsize) > 400 or abs(ysize) > 400:
            continue
        x0 = (-xoff - xsize) if facing else xoff
        out.append((x0, -yoff - ysize, x0 + xsize, -yoff))
    return out


class AnimDumper:
    def __init__(self, rom, cat):
        self.rom = rom
        self.cat = cat
        self.P = Painter(rom)
        self.sprite_cache = {}

    def frame_img(self, f, kind, stride, pal_key):
        pass

    def sprites(self, f, stride, raw):
        key = (f, raw)
        s = self.sprite_cache.get(key)
        if s is None:
            s = self.P.raw_sprites(f) if raw else self.P.frame_sprites(f)
            self.sprite_cache[key] = s
        return s

    def dump_anim(self, gdir, tag, title, start, stride, raw, pal_set, box_table, ids_label):
        """Returns metadata dict, writes GIF and strip."""
        rom, P = self.rom, self.P
        if raw:
            recs, link = [], None
            p, seen = start, set()
            while p not in seen and len(recs) < 200:
                seen.add(p)
                recs.append(p)
                if rom.b[p + 7] & 0x80:
                    link = p + 8 + rom.i16(p + 8)
                    break
                p += 8
            stride = 8
        else:
            recs, link = records(rom, start, stride)
        st, ar = pal_set if pal_set else (0, 0)
        pals, _ = P.obj_set(st, ar)
        sprs = [self.sprites(f, stride, raw) for f in recs]
        durs, types = [], []
        boxes = []
        for f in recs:
            d = rom.b[f + 7] & 0x7F
            durs.append(d if d else 256)
            types.append(rom.b[f + 6])
            if raw:
                boxes.append([])
            else:
                body, atk = rom.b[f + 8], rom.b[f + 9]
                boxes.append([(COL_BODY, r) for r in box_rects(rom, box_table, body)] +
                             [((COL_ATK if rom.b[f + 6] == 4 else COL_PROBE), r) for r in box_rects(rom, box_table, atk)])
        # union bounding box over all frames (sprites + boxes) so frames align at the origin
        xs0 = ys0 = 10 ** 6
        xs1 = ys1 = -10 ** 6
        for sp, bx in zip(sprs, boxes):
            x0, y0, x1, y1 = P.bbox(sp)
            xs0, ys0, xs1, ys1 = min(xs0, x0), min(ys0, y0), max(xs1, x1), max(ys1, y1)
            for _, r in bx:
                xs0, ys0, xs1, ys1 = min(xs0, r[0]), min(ys0, r[1]), max(xs1, r[2]), max(ys1, r[3])
        xs0, ys0, xs1, ys1 = min(xs0, -4), min(ys0, -4), max(xs1, 4), max(ys1, 12)
        m = 4
        W, H = xs1 - xs0 + 2 * m, ys1 - ys0 + 2 * m
        W, H = min(W, 700), min(H, 600)
        ox, oy = m - xs0, m - ys0
        # GIF
        gif_frames, gif_ms = [], []
        cells = []
        for k, f in enumerate(recs):
            cv = np.zeros((H, W, 4), np.uint8)
            P.draw(cv, sprs[k], ox, oy, pals)
            cells.append(cv)
        scale = 2 if max(W, H) <= 260 else 1
        for cv, d in zip(cells, durs):
            gif_frames.append(to_pil(cv, scale=scale))
            gif_ms.append(max(20, int(round(d * TICK_MS / 10.0)) * 10))
        save_gif(os.path.join(gdir, f"{tag}.gif"), gif_frames, gif_ms)
        # strip
        lab_h = 26
        head_h = 16
        cw = max(W * scale, 96)
        per_row = max(1, min(len(recs), 2400 // cw))
        rows = (len(recs) + per_row - 1) // per_row
        im = Image.new("RGB", (max(per_row * cw, 760), head_h + rows * (H * scale + lab_h)), BG)
        d = ImageDraw.Draw(im)
        loop_txt = ""
        if link is not None:
            if link in recs:
                loop_txt = f"loops to frame {recs.index(link)}"
            else:
                loop_txt = f"continues at {link:#x}"
        text(d, (4, 2), f"{title}  |  {len(recs)} frames, {sum(durs)} ticks  |  {loop_txt}  |  box table {hx(box_table) if box_table else 'n/a'}  pal s{st}a{ar}", size=10)
        for k, f in enumerate(recs):
            r_, c_ = divmod(k, per_row)
            x0 = c_ * cw
            y0 = head_h + r_ * (H * scale + lab_h)
            pim = to_pil(cells[k], scale=scale)
            d2 = ImageDraw.Draw(pim)
            ox2, oy2 = ox * scale, oy * scale
            d2.line([ox2 - 4, oy2, ox2 + 4, oy2], fill=(255, 255, 255))
            d2.line([ox2, oy2 - 4, ox2, oy2 + 4], fill=(255, 255, 255))
            for col, (a0, b0, a1, b1) in boxes[k]:
                d2.rectangle([(ox + a0) * scale, (oy + b0) * scale, (ox + a1) * scale, (oy + b1) * scale], outline=col)
            im.paste(pim, (x0 + (cw - pim.width) // 2, y0 + lab_h))
            fl = f"#{k} @{f:06X}"
            if raw:
                text(d, (x0 + 2, y0 + 1), fl, size=9)
                text(d, (x0 + 2, y0 + 11), f"t{durs[k]} T{types[k]:X}", fill=(255, 255, 160), size=9)
            else:
                text(d, (x0 + 2, y0 + 1), fl + f" c{rom.u16(f) & 0x7FFF:X}", size=9)
                text(d, (x0 + 2, y0 + 11), f"t{durs[k]} T{types[k]:X} b{rom.b[f + 8]:X} a{rom.b[f + 9]:X}" + (" END" if rom.b[f + 7] & 0x80 else ""),
                     fill=(255, 255, 160), size=9)
        im.save(os.path.join(gdir, f"{tag}_strip.png"))
        return dict(tag=tag, start=start, frames=len(recs), ticks=sum(durs), link=link, loop=(link in recs) if link is not None else None,
                    loop_to=recs.index(link) if link in recs else None, size=[W * scale, H * scale], types=sorted(set(types)),
                    comps=sorted({c for f in recs if not raw for c in cat_mod.frame_comps(rom, f)}))


def run(rom=None, cat=None, only=None, verbose=True):
    rom = rom or Rom()
    if cat is None:
        cat, _ = cat_mod.build(rom, verbose=False)
    A = AnimDumper(rom, cat)
    root = outdir("anims")
    meta = []
    t0 = time.time()
    n_anims = 0
    jobs = []
    for t, d in sorted(cat["tables"].items()):
        jobs.append(("table", t, d))
    for v, d in sorted(cat["single"].items()):
        jobs.append(("direct", v, d))
    for v, d in sorted(cat["raw"].items()):
        jobs.append(("raw", v, d))
    for kind, t, d in jobs:
        if only and only.lower() not in (hx(t).lower(), d.get("group", "")):
            continue
        if kind == "table":
            group, label, stride, box, pal_set = d["group"], d["label"], d["stride"], d["box"], d["pal_set"]
            ids = [(i, t + o) for i, o in enumerate(d["offs"]) if o]
            raw = False
        else:
            label, group, stride, box, pal_set = d["label"], d["group"], 10, None, d["pal_set"]
            ids = [(0, t)]
            raw = kind == "raw"
        gdir = os.path.join(root, group)
        os.makedirs(gdir, exist_ok=True)
        tmeta = dict(kind=kind, table=t, group=group, label=label, stride=stride, box=box, pal_set=pal_set,
                     episodes=d.get("episodes", []), anims=[])
        for i, start in ids:
            tag = f"{t:06X}_{i:02d}" if kind == "table" else f"{kind}_{t:06X}"
            try:
                m = A.dump_anim(gdir, tag, f"{label}  table {t:#x}  id {i}" if kind == "table" else f"{label}  {kind} frames {t:#x}", start, stride,
                                raw, pal_set, box, label)
            except Exception as e:  # keep going, report
                print("  failed", kind, hex(t), i, repr(e))
                continue
            m["id"] = i
            tmeta["anims"].append(m)
            n_anims += 1
        meta.append(tmeta)
        if verbose and len(meta) % 20 == 0:
            print(f"  {len(meta)}/{len(jobs)} tables, {n_anims} animations, {time.time() - t0:.0f}s", flush=True)
    with open(os.path.join(outdir("data"), "anims.json"), "w") as f:
        json.dump(meta, f)
    print(f"anims: {len(meta)} tables, {n_anims} animations (GIF + strip each), {time.time() - t0:.0f}s")
    return meta


if __name__ == "__main__":
    only = sys.argv[1] if len(sys.argv) > 1 else None
    run(only=only)
