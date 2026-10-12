"""Renders every stage map (scroll2 and scroll3, all areas) with the right palettes, a copy with the camera events
marked, 384x224 views, scroll1 pictures and the scene screens outside the stages. Output: re/out/assets/stages/."""
import json
import os
import sys

import numpy as np
from PIL import Image, ImageDraw

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from romio import Rom, outdir, hx, N_AREAS, STAGE_NAMES
from render import text, swatch_image, font
import maps as M

EV_COLORS = [(255, 80, 80), (255, 200, 40), (80, 220, 255), (120, 255, 120), (255, 120, 255), (255, 160, 80)]


def save(img, path):
    Image.fromarray(img).save(path)


def mark_events(img, mask, rec2, evs, by_y_all):
    """Copy of the scroll2 image with camera event lines and labels."""
    H = img.shape[0]
    base = img.copy()
    base[~mask] = (40, 44, 60)
    im = Image.fromarray(base)
    d = ImageDraw.Draw(im, "RGBA")
    f = font(16)
    # initial camera window
    cx, cy = rec2["x"], rec2["y"]
    d.rectangle([cx, H - 0xF0 - cy, cx + 383, H - 0xF0 - cy + 223], outline=(60, 255, 60), width=2)
    d.text((cx + 4, H - 0xF0 - cy + 4), "start", fill=(60, 255, 60), font=f)
    prev_by_y = 0
    rows = {}
    for e in evs:
        col = EV_COLORS[e["area"] % len(EV_COLORS)]
        axis_y = prev_by_y or by_y_all
        label = f"#{e['index']} m{e['mode']:02X} {M.CAMERA_MODES.get(e['mode'], '?')} a{e['area']}"
        if axis_y:
            y = H - e["y"]
            d.line([0, y, img.shape[1], y], fill=col + (200,), width=3)
            d.text((6 + 330 * (e["index"] % 3), y - 20), label, fill=col, font=f)
        else:
            x = e["x"]
            d.line([x, 0, x, H], fill=col + (200,), width=3)
            d.text((x + 4, 4 + 20 * (e["index"] % max(1, min(8, H // 24)))), label, fill=col, font=f)
        prev_by_y = e["by_y"]
    return im


def run(rom=None):
    rom = rom or Rom()
    out = outdir("stages")
    meta = dict(stages=[])
    tile_use = {"s1": {}, "s2": {}, "s3": {}}
    for stage in range(8):
        smeta = dict(stage=stage, episode=stage + 1, areas=[], events=[])
        evs = M.events(rom, stage)
        smeta["events"] = evs
        # which area owns each event: events carry the area number; sub-areas (scroll2 width 0) share the previous map
        recs2 = [M.area_rec(rom, M.S2_HDR, stage, a) for a in range(N_AREAS[stage])]
        map_of = {}
        last = 0
        for a, r in enumerate(recs2):
            if r["w"]:
                last = a
            map_of[a] = last
        for area in range(N_AREAS[stage]):
            r2 = recs2[area]
            r3 = M.area_rec(rom, M.S3_HDR, stage, area)
            b2 = M.s2_bank_addr(rom, stage, area)
            b3 = M.s3_bank_addr(rom, stage, area)
            am = dict(area=area, s2=dict(w=r2["w"], h=r2["h"], x=r2["x"], y=r2["y"], hdr=r2["addr"], bank=b2),
                      s3=dict(w=r3["w"], h=r3["h"], x=r3["x"], y=r3["y"], hdr=r3["addr"], bank=b3),
                      shift=M.s3_shift(rom, stage, area), yfollow=M.s3_yfollow(rom, stage, area))
            tag = f"ep{stage + 1}_a{area}"
            p2, p3 = rom.bank(b2), rom.bank(b3)
            if r2["w"] == 0:
                am["note"] = "sub-area: keeps the map of the previous area (camera mode 8), new OBJ palettes only"
                smeta["areas"].append(am)
                continue
            img2, m2, u2 = M.render_s2(rom, stage, area, p2, r2)
            save(img2, os.path.join(out, f"{tag}_scroll2.png"))
            for c, p in u2.items():
                tile_use["s2"].setdefault(c, (b2, p))
            r3img = None
            if r3["w"] and r3["screens"]:
                img3, m3, u3 = M.render_s3(rom, stage, area, p3, r3)
                save(img3, os.path.join(out, f"{tag}_scroll3.png"))
                r3img = (img3, m3)
                for c, p in u3.items():
                    tile_use["s3"].setdefault(c, (b3, p))
            else:
                am["s3"]["note"] = "no far layer in this area"
            # events on a copy
            own_evs = [e for e in evs if map_of.get(e["area"], e["area"]) == area]
            mark = mark_events(img2, m2, r2, own_evs, 1 if (stage == 6 and area == 1) else 0)
            mark.save(os.path.join(out, f"{tag}_scroll2_events.png"))
            am["events"] = [e["index"] for e in own_evs]
            # views: composite at several camera X along the map
            layers = []
            if r3img:
                layers.append((r3img[0], r3img[1], r3["h"], lambda x, y, s=stage, a=area: M.s3_camera(rom, s, a, x, y)))
            layers.append((img2, m2, r2["h"], lambda x, y: (x, y)))
            W2 = r2["w"] * 256
            n_views = 1 if W2 <= 400 else min(6, max(2, W2 // 1200))
            if r2["h"] > 8 and r2["w"] <= 3:       # tall map: vary camY
                cams = [(r2["x"], int(r2["h"] * 256 - 0xF0 - k * (r2["h"] * 256 - 240) / 5)) for k in range(6)]
                n_views = 6
            else:
                cams = [(int(k * max(0, W2 - 384) / max(1, n_views - 1)) if n_views > 1 else r2["x"], r2["y"]) for k in range(n_views)]
            shots = [M.view(layers, cx, cy) for cx, cy in cams]
            if r2["h"] > 8:
                shots = [M.view(layers, cx, max(0, cy)) for cx, cy in cams]
            cols = 3 if len(shots) > 3 else len(shots)
            rows = (len(shots) + cols - 1) // cols
            sheet = np.zeros((rows * 228, cols * 388, 3), np.uint8)
            for i, s in enumerate(shots):
                rr, cc = divmod(i, cols)
                sheet[rr * 228:rr * 228 + 224, cc * 388:cc * 388 + 384] = s
            save(sheet, os.path.join(out, f"{tag}_views.png"))
            am["views"] = [list(c) for c in cams]
            smeta["areas"].append(am)
        # scroll1 elevator background of stage 6 area 1
        if stage == 6:
            pal1 = rom.bank(M.PAL_S1)
            img = np.concatenate([M.s1_screen(rom, sid, pal1)[0] for sid in (1, 2)], axis=1)
            save(img, os.path.join(out, "ep7_a1_scroll1_elevator_cage.png"))
        meta["stages"].append(smeta)
    # scene screens outside the stages
    sp = outdir("stages", "scenes")
    sets = scene_sets(rom)
    meta["scene_sets"] = sets
    s3_used_by_stage = {}
    for sm in meta["stages"]:
        pass
    render_scenes(rom, sp, sets, meta, tile_use)
    with open(os.path.join(outdir("data"), "stages.json"), "w") as f:
        json.dump(meta, f)
    return meta, tile_use


def scene_calls(rom):
    """Calls of the three picture writers with the page id that code passes (moveq or move.w #id,d5 just before the call):
    0x20592 scroll3 page (0x177288), 0x204CA scroll2 scene page (0x145DC8), 0x20454 scroll1 picture (0x138C48)."""
    b = rom.b
    names = {0x20592: "scroll3", 0x204CA: "scroll2", 0x20454: "scroll1"}
    out = []
    for x in range(0x1000, 0xB7000, 2):
        w = (b[x] << 8) | b[x + 1]
        t = None
        if w in (0x4EB9, 0x4EF9) and rom.u32(x + 2) in names:
            t = rom.u32(x + 2)
        elif w in (0x6100, 0x6000) and (x + 2 + rom.i16(x + 2)) in names:
            t = x + 2 + rom.i16(x + 2)
        if t is None:
            continue
        pid = None
        for back in range(2, 60, 2):
            y = x - back
            wy = (b[y] << 8) | b[y + 1]
            if wy == 0x3A3C:
                pid = rom.u16(y + 2)
                break
            if (wy & 0xFE00) == 0x7A00 and wy & 0x100 == 0:
                pid = wy & 0xFF
                break
        out.append((x, names[t], pid))
    return out


def scene_sets(rom):
    """The six palette sets of special screens (0x89E6): (OBJ, scroll1+2, scroll3, priority) (stage slot, area) pairs."""
    out = []
    for k in range(6):
        a = 0x8A5E + k * 8
        pairs = [(rom.b[a + 2 * i], rom.b[a + 2 * i + 1]) for i in range(4)]
        # observed in SPEC section 2.3: bytes are (stage, area) pairs
        out.append(dict(index=k, obj=pairs[0], s12=pairs[1], s3=pairs[2], prio=pairs[3],
                        s2_bank=M.s2_bank_addr(rom, *pairs[1]), s3_bank=M.s3_bank_addr(rom, *pairs[2])))
    return out


def render_scenes(rom, sp, sets, meta, tile_use):
    """Every scroll3 screen id 0..127, every fixed scroll2 screen and every scroll1 picture screen, in each scene palette."""
    b = rom.b
    banks3 = []
    for s in sets:
        if s["s3_bank"] not in [x[0] for x in banks3]:
            banks3.append((s["s3_bank"], f"set{s['index']}"))
    banks2 = []
    for s in sets:
        if s["s2_bank"] not in [x[0] for x in banks2]:
            banks2.append((s["s2_bank"], f"set{s['index']}"))
    # which scroll3 screen ids do the stage maps use?
    used = {}
    for stage in range(8):
        for area in range(N_AREAS[stage]):
            r3 = M.area_rec(rom, M.S3_HDR, stage, area)
            if r3["w"] and r3["screens"]:
                for k in range(r3["w"] * r3["h"]):
                    used.setdefault(rom.u16(r3["screens"] + 2 * k) & 0x7F, set()).add((stage + 1, area))
    meta["s3_screen_use"] = {str(k): sorted(v) for k, v in used.items()}
    banks = [(a, n) for a, n in banks3]
    # sheet of 8 screens per image, one row per screen: [stage bank (if used)] + scene banks, half scale
    scene_s3, scene_s2 = {}, {}
    tile_use["scene_s3"], tile_use["scene_s2"] = scene_s3, scene_s2
    f = font(12)
    for first in range(0, 128, 8):
        cols = len(banks) + 1
        sheet = Image.new("RGB", (cols * 132 + 4, 8 * 150 + 20), (30, 30, 38))
        d = ImageDraw.Draw(sheet)
        d.text((4, 2), f"scroll3 screens {first}..{first + 7} (256x256, shown at half size). Columns: stage palette, then scene palette banks " +
               ", ".join(f"{n} {hx(a)}" for a, n in banks), fill=(255, 255, 255), font=font(10))
        for i in range(8):
            sid = first + i
            y0 = 20 + i * 150
            d.text((4, y0), f"screen {sid:#04x}" + (" used by " + ",".join(f"{s}.{a}" for s, a in sorted(used[sid])[:6]) if sid in used else " unused by stages"), fill=(255, 255, 160), font=font(10))
            variants = []
            if sid in used:
                st, ar = sorted(used[sid])[0]
                variants.append(("stage", M.s3_bank_addr(rom, st - 1, ar)))
            else:
                variants.append(None)
            for a, n in banks:
                variants.append((n, a))
            for c, v in enumerate(variants):
                if v is None:
                    continue
                img, mask = M.s3_screen(rom, sid, rom.bank(v[1]), scene_s3)
                img[~mask] = (50, 54, 72)
                im = Image.fromarray(img).resize((128, 128), Image.NEAREST)
                sheet.paste(im, (4 + c * 132, y0 + 14))
        sheet.save(os.path.join(sp, f"scroll3_screens_{first:03d}_{first + 7:03d}.png"))
    # fixed scroll2 screens (0x204CA): scan the ids until block numbers leave the block table (497 blocks)
    n_blocks = (0x177288 - M.SP2_BLOCKS) // 16
    ids = []
    for sid in range(0, 700):
        a = M.SP2_SCREENS + sid * 128
        if a + 128 > M.S1_METATILES:
            break
        vals = [rom.u16(a + 2 * k) for k in range(64)]
        if max(vals) >= n_blocks:
            break
        ids.append(sid)
    # drop the empty pages at the end (only block 0 of the scene tables)
    while ids and not M.s2_special_screen(rom, ids[-1], rom.bank(M.s2_bank_addr(rom, 12, 0)))[1].any():
        ids.pop()
    meta["scene_s2_screens"] = len(ids)
    for first in range(0, len(ids), 8):
        chunk = ids[first:first + 8]
        sheet = Image.new("RGB", (len(banks2) * 132 + 4, len(chunk) * 150 + 20), (30, 30, 38))
        d = ImageDraw.Draw(sheet)
        d.text((4, 2), "fixed scroll2 screens, palette banks: " + ", ".join(f"{n} {hx(a)}" for a, n in banks2), fill=(255, 255, 255), font=font(10))
        for i, sid in enumerate(chunk):
            y0 = 20 + i * 150
            d.text((4, y0), f"screen {sid:#04x}", fill=(255, 255, 160), font=font(10))
            for c, (a, n) in enumerate(banks2):
                img, mask = M.s2_special_screen(rom, sid, rom.bank(a), scene_s2)
                img[~mask] = (50, 54, 72)
                sheet.paste(Image.fromarray(img).resize((128, 128), Image.NEAREST), (4 + c * 132, y0 + 14))
        sheet.save(os.path.join(sp, f"scroll2_scene_screens_{chunk[0]:03d}_{chunk[-1]:03d}.png"))
    # scroll1 picture screens (7 screens, 0x138C48): bank 0 and a few palettes
    pal1 = rom.bank(M.PAL_S1)
    n1 = (0x138FC8 - M.S1_SCREENS) // 128
    sheet = Image.new("RGB", (n1 * 260 + 4, 3 * 268), (30, 30, 38))
    d = ImageDraw.Draw(sheet)
    for sid in range(n1):
        for row, pidx in enumerate((None,)):
            img, mask = M.s1_screen(rom, sid, pal1)
            img[~mask] = (50, 54, 72)
            sheet.paste(Image.fromarray(img), (4 + sid * 260, 14))
            d.text((4 + sid * 260, 2), f"scroll1 screen {sid}", fill=(255, 255, 160), font=font(10))
    sheet.crop((0, 0, n1 * 260 + 4, 280)).save(os.path.join(sp, "scroll1_picture_screens.png"))
    meta["scene_s1_screens"] = n1
    calls = scene_calls(rom)
    with open(os.path.join(sp, "picture_calls.txt"), "w") as f:
        f.write("# Code address of each call to a picture writer and the page id it passes. Pages are drawn into scroll VRAM; the scene is the routine that holds the call.\n")
        for addr, layer, pid in calls:
            f.write(f"{addr:06X}  {layer}  page {pid if pid is None else hex(pid)}\n")
    meta["scene_calls"] = len(calls)


if __name__ == "__main__":
    run()
