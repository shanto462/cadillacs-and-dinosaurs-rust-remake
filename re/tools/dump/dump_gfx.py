"""Tile sheets (8x8, 16x16, 32x32) and every palette table of the ROM. Output: re/out/assets/gfx/ and palettes/."""
import json
import os
import sys

import numpy as np
from PIL import Image, ImageDraw

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from romio import Rom, outdir, hx, N_AREAS
from render import text, swatch_image, font, Painter
import maps as M

GREY = np.array([(i * 17,) * 3 for i in range(16)], np.uint8)


def tile_sheet(tiles, first, count, tile_px, cols, pal_for, title, scale=1, labelw=44):
    """tiles: [N,h,w] pens. pal_for(code) -> uint8[16,3]. Row labels = code of the first tile in the row."""
    rows = (count + cols - 1) // cols
    h, w = tile_px
    cw, ch = w * scale + 1, h * scale + 1
    W, H = labelw + cols * cw, 18 + rows * ch
    im = Image.new("RGB", (W, H), (30, 30, 38))
    d = ImageDraw.Draw(im)
    text(d, (4, 3), title, size=11)
    arr = np.zeros((rows * ch, cols * cw, 3), np.uint8)
    arr[:] = (46, 50, 66)
    for i in range(count):
        code = first + i
        if code >= len(tiles):
            break
        r, c = divmod(i, cols)
        pens = tiles[code]
        pal = pal_for(code)
        rgb = pal[pens]
        rgb[pens == 15] = (46, 50, 66)
        if scale != 1:
            rgb = np.repeat(np.repeat(rgb, scale, 0), scale, 1)
        arr[r * ch:r * ch + h * scale, c * cw:c * cw + w * scale] = rgb
    im.paste(Image.fromarray(arr), (labelw, 18))
    for r in range(rows):
        text(d, (2, 18 + r * ch + 2), f"{first + r * cols:04X}", size=9)
    return im


def run(rom=None, tile_use=None, cat=None):
    rom = rom or Rom()
    P = Painter(rom)
    gfx = outdir("gfx")
    pal_dir = outdir("palettes")
    meta = {}
    tile_use = tile_use or {"s1": {}, "s2": {}, "s3": {}}
    # ---------------------------------------------------------------- palettes
    n_obj = (M.PAL_S1 - 0xB7A52) // 32
    obj_all = rom.bank(0xB7A52, n_obj)
    sets = {}
    used_obj = set()
    for stage in range(8):
        for area in range(N_AREAS[stage]):
            pals, nums = P.obj_set(stage, area)
            sets[(stage, area)] = nums
            used_obj.update(nums)
            lst = rom.u32(M.PAL_OBJ + stage * 16 + area * 4)
            im = swatch_image(pals, [f"slot {i:2d} = {n}" for i, n in enumerate(nums)], title=f"OBJ palette set, Episode {stage + 1} area {area} (list at {hx(lst)}): slot -> ROM palette number")
            im.save(os.path.join(pal_dir, f"obj_set_ep{stage + 1}_a{area}.png"))
    # palettes that code points at directly (fade, flash and colour change tables copy a palette by address)
    ptr_used = set()
    b_ = rom.b
    for a in range(0, 0xB7000, 2):
        if b_[a] == 0 and b_[a + 1] == 0x0B:
            v = (b_[a + 1] << 16) | (b_[a + 2] << 8) | b_[a + 3]
            if 0xB7A52 <= v < 0xBBA52 and (v - 0xB7A52) % 32 == 0:
                ptr_used.add((v - 0xB7A52) // 32)
    for k in range(4):
        chunk = obj_all[k * 128:(k + 1) * 128]
        labs = [f"{k * 128 + i:3d}{'' if k * 128 + i in used_obj else (' ptr' if k * 128 + i in ptr_used else ' unused')}" for i in range(len(chunk))]
        swatch_image(chunk, labs, cols=2, cell=12, title=f"OBJ ROM palettes {k * 128}..{k * 128 + len(chunk) - 1} (at {hx(0xB7A52 + k * 128 * 32)}). 'ptr' = no stage set lists it but code points at it, 'unused' = neither. Pen 15 (red cross) is transparent").save(
            os.path.join(pal_dir, f"obj_rom_palettes_{k * 128:03d}.png"))
    meta["obj_palettes"] = n_obj
    meta["obj_palettes_in_sets"] = sorted(used_obj)
    meta["obj_palettes_by_pointer"] = sorted(ptr_used - used_obj)
    meta["obj_palettes_unused"] = [i for i in range(n_obj) if i not in used_obj and i not in ptr_used]
    # scroll1 bank
    s1 = rom.bank(M.PAL_S1)
    swatch_image(s1, [f"pal {i}" for i in range(32)], title=f"scroll1 palette bank at {hx(M.PAL_S1)} (fixed, HUD and text)").save(os.path.join(pal_dir, "scroll1_bank.png"))
    # scroll2 / scroll3 banks: every bank in the region, with the users
    users2, users3 = {}, {}
    for slot in range(20):
        for area in range(4):
            users2.setdefault(rom.u32(M.PAL_S2 + 4 * (slot * 4 + area)), []).append((slot, area))
            users3.setdefault(rom.u32(M.PAL_S3 + 4 * (slot * 4 + area)), []).append((slot, area))
    def users_text(us):
        out = []
        for slot, area in us[:6]:
            out.append(f"ep{slot + 1}.{area}" if slot < 8 else f"scene slot {slot}.{area}")
        return ", ".join(out) + ("..." if len(us) > 6 else "")
    banks = {"scroll2": [], "scroll3": []}
    for name, base, end, users in (("scroll2", 0xBCA52, 0xC2252, users2), ("scroll3", 0xC2252, 0xC5A52, users3)):
        for a in range(base, end, 0x400):
            us = users.get(a)
            title = f"{name} bank {hx(a)}: " + (users_text(us) if us else "NOT referenced by the palette table (unused?)")
            swatch_image(rom.bank(a), [f"pal {i}" for i in range(32)], cols=2, cell=12, title=title).save(os.path.join(pal_dir, f"{name}_bank_{a:06X}.png"))
            banks[name].append(dict(addr=a, users=us or []))
    meta["banks"] = banks
    meta["obj_sets"] = {f"{s}.{a}": n for (s, a), n in sets.items()}
    # ---------------------------------------------------------------- tiles
    # sprite tile -> ROM palette through compositions
    sprite_pal = {}
    sprite_used = np.zeros(0x8000, bool)
    comps = json.load(open(os.path.join(outdir("data"), "comps.json")))["comps"] if os.path.exists(os.path.join(outdir("data"), "comps.json")) else []
    for m in comps:
        st, ar = m["pal_set"] if m["pal_set"] else (0, 0)
        nums = sets.get((st, ar), sets[(0, 0)])
        for dx, dy, code, attr in P.comp(m["byte_index"])["ents"]:
            nx, ny = ((attr >> 8) & 15) + 1, ((attr >> 12) & 15) + 1
            for j in range(ny):
                for i in range(nx):
                    c = ((code & ~0xF) + ((code + i) & 0xF) + 0x10 * j) & 0x7FFF
                    sprite_used[c] = True
                    sprite_pal.setdefault(c, nums[attr & 0x1F])
    # shadow / splash sprite lists and raw lists add used tiles (not coloured)
    cat_path = os.path.join(outdir("data"), "catalog_raw.json")
    for a in range(0x14F98, 0x14F98 + 32, 4):
        lst = rom.u32(a)
        if 0x1000 < lst < 0x180000:
            n = rom.u16(lst)
            for k in range(min(n + 1, 12)):
                code = rom.u16(lst + 4 + 6 * k + 4)
                if code < 0x7000:
                    sprite_used[code] = True
    pal_obj_for = lambda code: obj_all[sprite_pal[code]] if code in sprite_pal else GREY
    # 16x16 tiles in 32 sheets of 1024
    t16 = rom.t16
    blank = np.array([bool((t == 15).all()) for t in t16])
    zero = np.array([bool((t == 0).all()) for t in t16])
    s2_used = np.zeros(0x8000, bool)
    for c in tile_use["s2"]:
        s2_used[c & 0x7FFF] = True
    for c in tile_use.get("scene_s2", ()):
        s2_used[c & 0x7FFF] = True
    sheets = []
    regions = lambda lo, hi: ("scroll1 8x8 glyph area" if hi <= 0x800 else "sprites" + (" + scroll2" if lo >= 0x4000 else "") if hi <= 0x7000 else "scroll3 area (32x32 tiles, shown here as 16x16 pieces)")
    for k in range(32):
        first = k * 1024
        title = f"16x16 tiles {first:04X}..{first + 1023:04X} (graphics ROM {first * 128:06X}..{(first + 1024) * 128 - 1:06X}), " + (
            "region: scroll1 8x8 glyphs" if first < 0x800 else "region: sprites" if first < 0x4000 else "region: sprites + scroll2 map tiles" if first < 0x7000 else "region: scroll3 32x32 tiles") + ". Colour = palette of first known use, grey = none known"
        grey_im = tile_sheet(t16, first, 1024, (16, 16), 32, lambda c: GREY, "GREY " + title, scale=1)
        grey_im.save(os.path.join(outdir("gfx", "tiles16_grey"), f"tiles16_{first:04X}.png"))
        def col_for(c):
            if c in sprite_pal:
                return obj_all[sprite_pal[c]]
            if c in tile_use["s2"]:
                bank, p = tile_use["s2"][c]
                return rom.bank(bank)[p]
            return GREY
        col_im = tile_sheet(t16, first, 1024, (16, 16), 32, col_for, title, scale=1)
        col_im.save(os.path.join(outdir("gfx", "tiles16_colour"), f"tiles16_{first:04X}.png"))
    # tiles that no composition, map or scene screen uses: candidates for unused art (non blank only)
    unused = [c for c in range(0x800, 0x7000) if not sprite_used[c] and not s2_used[c] and not blank[c] and not zero[c]]
    meta["unused16_count"] = len(unused)
    for k in range(0, len(unused), 1024):
        chunk = unused[k:k + 1024]
        cols = 32
        rows = (len(chunk) + cols - 1) // cols
        im = Image.new("RGB", (44 + cols * 17, 18 + rows * 17), (30, 30, 38))
        d = ImageDraw.Draw(im)
        text(d, (4, 3), f"16x16 tiles in the sprite/scroll2 range that no composition and no stage map uses ({len(unused)} in all, non blank). Code shown per row start", size=11)
        for i, c in enumerate(chunk):
            r, cc = divmod(i, cols)
            rgb = GREY[t16[c]]
            im.paste(Image.fromarray(rgb.astype(np.uint8)), (44 + cc * 17, 18 + r * 17))
        for r in range(rows):
            text(d, (2, 18 + r * 17 + 2), f"{chunk[r * cols]:04X}", size=9)
        im.save(os.path.join(outdir("gfx", "tiles16_unused"), f"unused16_{k // 1024 + 1:02d}.png"))
    # 32x32 tiles: all 8192 in 8 sheets of 1024 (grey) and the scroll3 range coloured
    t32 = rom.t32
    s3_used = set(tile_use["s3"])
    for k in range(8):
        first = k * 1024

        def col3(c):
            if c in tile_use["s3"]:
                bank, p = tile_use["s3"][c]
                return rom.bank(bank)[p]
            return GREY
        title = f"32x32 tiles {first:04X}..{first + 1023:04X} (graphics ROM {first * 512:06X}.., " + ("scroll3 range" if first >= 0x1C00 else "NOT in the scroll3 range: shown for completeness") + ")"
        if first >= 0x1000:
            tile_sheet(t32, first, 1024, (32, 32), 32, col3, title, scale=1).save(os.path.join(gfx, f"tiles32_{first:04X}.png"))
        else:
            tile_sheet(t32, first, 1024, (32, 32), 32, lambda c: GREY, title, scale=1).save(os.path.join(gfx, f"tiles32_{first:04X}.png"))
    # 8x8 scroll1 tiles: codes 0..0xFFF as 16x8 (even + odd half)
    t8 = rom.t8
    for pi, (pname, pal) in enumerate((("grey", GREY), ("pal0", s1[0]), ("pal6", s1[6]))):
        for k in range(4):
            first = k * 1024
            cols = 32
            rows = 32
            im = Image.new("RGB", (44 + cols * 33, 18 + rows * 9), (30, 30, 38))
            d = ImageDraw.Draw(im)
            text(d, (4, 3), f"scroll1 8x8 codes {first:03X}..{first + 1023:03X}, each drawn as even column half | odd column half, palette {pname} of the fixed scroll1 bank", size=11)
            arr = np.zeros((rows * 9, cols * 33, 3), np.uint8)
            arr[:] = (46, 50, 66)
            for i in range(1024):
                r, c = divmod(i, cols)
                for half in (0, 1):
                    pens = t8[first + i][half]
                    rgb = pal[pens]
                    rgb[pens == 15] = (46, 50, 66)
                    arr[r * 9:r * 9 + 8, c * 33 + half * 16: c * 33 + half * 16 + 8] = rgb
            im.paste(Image.fromarray(arr), (44, 18))
            for r in range(rows):
                text(d, (2, 18 + r * 9), f"{first + r * cols:03X}", size=8)
            im.save(os.path.join(outdir("gfx", "tiles8"), f"tiles8_{pname}_{first:03X}.png"))
    meta["sprite_tiles_used"] = int(sprite_used.sum())
    meta["blank_tiles16"] = int(blank.sum())
    with open(os.path.join(outdir("data"), "gfx.json"), "w") as f:
        json.dump(meta, f)
    print(f"gfx: {len(os.listdir(gfx))} files in gfx/, palettes {len(os.listdir(pal_dir))} files, unused16 {len(unused)}")
    return meta


if __name__ == "__main__":
    import dump_stages
    rom = Rom()
    _, tu = dump_stages.run(rom)
    run(rom, tu)
