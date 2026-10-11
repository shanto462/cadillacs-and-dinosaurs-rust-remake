#!/usr/bin/env python3
"""Cadillacs and Dinosaurs (CPS1) stage renderer.

Reads re/out/dino_68k.bin (68000 program, big-endian) and re/out/gfx.npy (4 MB interleaved
graphics ROM) and renders:
  - scroll2 playfield panoramas (16x16 tiles, 2x2-tile blocks, 8x8-block screens)
  - scroll3 far-background panoramas (32x32 tiles, 8x8-tile screens)
  - the scroll1 8x8 font sheet in its palette
  - for stage 6: the scroll1 "elevator" background (scroll1 used as a 3rd playfield in area 1)
  - 384x224 in-game screen composites (scroll3 behind scroll2, parallax applied)
All formats are documented in SPEC.md next to this file.

Usage: python -I render_stage.py [stage] [outdir]
       stage is the game's internal stage number $4d9(a5) (0 = first stage). Default 0.
       python -I render_stage.py contact [outdir]   -> one 384x224 screen of area 0 of every stage
"""
import os
import struct
import sys

import numpy as np
from PIL import Image

S = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))  # .../re
ROM = open(os.path.join(S, "out", "dino_68k.bin"), "rb").read()
GFX = np.load(os.path.join(S, "out", "gfx.npy"))


def w(a): return struct.unpack_from(">H", ROM, a)[0]
def l(a): return struct.unpack_from(">I", ROM, a)[0]
def b(a): return ROM[a]


# ---------------------------------------------------------------- ROM tables (see SPEC.md)
S2_HDR_TABLE = 0x21580      # long[stage] -> area records (10 bytes each)
S2_BLOCK_TABLE = 0x21454    # long[stage] -> 16-byte 2x2 blocks
S2_SCREEN_TABLE = 0x2155E   # long[stage] -> 128-byte screens (8x8 block words)
S3_HDR_TABLE = 0x2311E      # long[stage] -> area records (10 bytes each)
S3_SCREENS = 0x177288       # 256-byte screens (8x8 tilemap entries), shared by all stages
S1_SCREENS = 0x138C48       # 128-byte screens (8x8 metatile words) for scroll1 used as a bg layer
S1_METATILES = 0x15CFE8     # 64-byte metatiles (4x4 tilemap entries)
PAL_S1 = 0xBBA52            # fixed scroll1 palette bank (0x400 bytes)
PAL_S2_TABLE = 0x8AE0       # long[stage*4+area] -> scroll2 palette bank (0x400 bytes)
PAL_S3_TABLE = 0x8C52       # long[stage*4+area] -> scroll3 palette bank (0x400 bytes)
PAL_OBJ_TABLE = 0x8E8E      # long[stage*4+area] -> 32 words, palette numbers into PAL_OBJ_BASE
PAL_OBJ_BASE = 0xB7A52      # 32-byte palettes

N_AREAS = {0: 3, 1: 3, 2: 1, 3: 3, 4: 2, 5: 3, 6: 4, 7: 3}


def area_record(table, stage, area):
    r = l(table + 4 * stage) + 10 * area
    return dict(w=b(r), h=b(r + 1), screens=l(r + 2), x=w(r + 6), y=w(r + 8), addr=r)


# ---------------------------------------------------------------- colours
def colour(word):
    """CPS1 colour word 0xBRGB -> (r, g, b) 8-bit, MAME formula."""
    bright = 0x0F + ((word >> 12) << 1)
    r = ((word >> 8) & 0xF) * 0x11 * bright // 0x2D
    g = ((word >> 4) & 0xF) * 0x11 * bright // 0x2D
    bl = (word & 0xF) * 0x11 * bright // 0x2D
    return (r, g, bl)


def bank(addr, bright=0xF000):
    """0x400-byte palette bank -> uint8 array [32 palettes, 16 pens, 3].
    ROM colours carry brightness nibble 0; after upload the game ORs 0xF000 into every word
    (routine 0x892C, or the fade-in task 0x8824), so full brightness is the in-game look."""
    return np.array([[colour(w(addr + 32 * p + 2 * i) | bright) for i in range(16)] for p in range(32)], np.uint8)


# ---------------------------------------------------------------- tile decoding
def _planar(rows):
    """rows: uint8 [h, n*4] planar bytes (4 bytes = 8 px, byte p = bitplane p, MSB left) -> pens [h, n*8]."""
    h, nb = rows.shape
    r = rows.reshape(h, nb // 4, 4)
    bits = np.unpackbits(r, axis=2).reshape(h, nb // 4, 4, 8)
    px = bits[:, :, 0] | (bits[:, :, 1] << 1) | (bits[:, :, 2] << 2) | (bits[:, :, 3] << 3)
    return px.reshape(h, (nb // 4) * 8)


_cache = {}

# CPS-B "CD63B" graphics bank mapper as observed (see SPEC.md section 3). Ranges are in 64-byte units
# (scroll1 code << 0, sprite/scroll2 code << 1, scroll3 code << 3). Inside a range the ROM address is
# simply unit * 64 (identity); a code outside its layer's range is not drawn (transparent).
MAP_SCROLL1 = (0x0000, 0x0FFF)
MAP_SCROLL2 = (0x8000, 0xDFFF)
MAP_SCROLL3 = (0xE000, 0xFFFF)
_BLANK16 = np.full((16, 16), 15, np.uint8)
_BLANK32 = np.full((32, 32), 15, np.uint8)
_BLANK8 = np.full((8, 8), 15, np.uint8)


def tile16(code):
    """scroll2 16x16 tile: 128 bytes at code*128, 8 bytes per row."""
    k = (16, code)
    if k not in _cache:
        if not MAP_SCROLL2[0] <= (code << 1) <= MAP_SCROLL2[1]:
            _cache[k] = _BLANK16
        else:
            a = code * 128
            _cache[k] = _planar(GFX[a:a + 128].reshape(16, 8))
    return _cache[k]


def tile32(code):
    """scroll3 32x32 tile: 512 bytes at code*512, 16 bytes per row (code masked to 14 bits like the hardware)."""
    code &= 0x3FFF
    k = (32, code)
    if k not in _cache:
        if not MAP_SCROLL3[0] <= (code << 3) <= MAP_SCROLL3[1]:
            _cache[k] = _BLANK32
        else:
            a = code * 512
            _cache[k] = _planar(GFX[a:a + 512].reshape(32, 16))
    return _cache[k]


def tile8(code, half):
    """scroll1 8x8 tile: 64 bytes at code*64 (8 rows of 8 bytes); half 0 = bytes 0-3, half 1 = bytes 4-7.
    The hardware picks the half from the tilemap column parity (even column = left half)."""
    k = (8, code, half)
    if k not in _cache:
        if not MAP_SCROLL1[0] <= code <= MAP_SCROLL1[1]:
            _cache[k] = _BLANK8
        else:
            a = code * 64
            _cache[k] = _planar(GFX[a:a + 64].reshape(8, 8)[:, half * 4:half * 4 + 4])
    return _cache[k]


def put(img, mask, px, x, y, attr, pal):
    """Draw tile pens px at (x, y) using attr (palette bits 0-4, X flip bit 5, Y flip bit 6). Pen 15 = transparent."""
    if attr & 0x20:
        px = px[:, ::-1]
    if attr & 0x40:
        px = px[::-1, :]
    h, wd = px.shape
    rgb = pal[attr & 0x1F][px]
    opaque = px != 15
    sub = img[y:y + h, x:x + wd]
    sub[opaque] = rgb[opaque]
    mask[y:y + h, x:x + wd][opaque] = 1


# ---------------------------------------------------------------- layer renderers
def render_scroll2(stage, area, pal):
    rec = area_record(S2_HDR_TABLE, stage, area)
    W, H = rec["w"], rec["h"]
    blocks = l(S2_BLOCK_TABLE + 4 * stage)
    screens = l(S2_SCREEN_TABLE + 4 * stage)
    img = np.zeros((H * 256, W * 256, 3), np.uint8)
    mask = np.zeros((H * 256, W * 256), np.uint8)
    for sx in range(W):
        for sy in range(H):                       # sy = world screen row, 0 = bottom (world Y grows upward)
            sid = w(rec["screens"] + 2 * (sx * H + sy))
            oy = (H - 1 - sy) * 256               # image row of the screen's top edge
            for cx in range(8):
                for p in range(8):                # p = block row from the top of the screen
                    blk = w(screens + sid * 128 + cx * 16 + p * 2)
                    ba = blocks + blk * 16
                    for j, (dx, dy) in enumerate(((0, 0), (0, 1), (1, 0), (1, 1))):
                        code, attr = w(ba + 4 * j), w(ba + 4 * j + 2)
                        put(img, mask, tile16(code), sx * 256 + cx * 32 + dx * 16, oy + p * 32 + dy * 16, attr, pal)
    return img, mask, rec


def render_scroll3(stage, area, pal):
    rec = area_record(S3_HDR_TABLE, stage, area)
    W, H = rec["w"], rec["h"]
    img = np.zeros((H * 256, W * 256, 3), np.uint8)
    mask = np.zeros((H * 256, W * 256), np.uint8)
    for sx in range(W):
        for sy in range(H):
            sid = w(rec["screens"] + 2 * (sx * H + sy)) & 0x7F
            oy = (H - 1 - sy) * 256
            for cx in range(8):
                for p in range(8):
                    ea = S3_SCREENS + sid * 256 + cx * 32 + p * 4
                    code, attr = w(ea), w(ea + 2)
                    put(img, mask, tile32(code), sx * 256 + cx * 32, oy + p * 32, attr, pal)
    return img, mask, rec


def render_scroll1_bg(screen_ids, pal):
    """Scroll1 used as a background layer (stage 6 area 1, routine 0x205C6): screens of 8x8 metatile
    words at S1_SCREENS (128 bytes each, column-major, top row first); metatile = 4x4 scroll1 entries
    (64 bytes, column-major: 4 rows of column 0, then column 1, ...)."""
    img = np.zeros((256, 256 * len(screen_ids), 3), np.uint8)
    mask = np.zeros(img.shape[:2], np.uint8)
    for k, sid in enumerate(screen_ids):
        for cx in range(8):
            for p in range(8):
                mt = w(S1_SCREENS + sid * 128 + cx * 16 + p * 2)
                for j in range(16):
                    code, attr = w(S1_METATILES + mt * 64 + 4 * j), w(S1_METATILES + mt * 64 + 4 * j + 2)
                    col, row = cx * 4 + j // 4, p * 4 + j % 4
                    put(img, mask, tile8(code, col & 1), k * 256 + col * 8, row * 8, attr, pal)
    return img, mask


def font_sheet(pal, palno=0, first=0, count=512):
    """Scroll1 8x8 codes as a 32-wide grid. Each code is drawn 16x8: its even-column half (bytes 0-3)
    then its odd-column half (bytes 4-7), which is how a 2-column-wide glyph appears on screen."""
    cols = 32
    rows = (count + cols - 1) // cols
    img = np.zeros((rows * 8, cols * 16, 3), np.uint8)
    mask = np.zeros(img.shape[:2], np.uint8)
    for i in range(count):
        x, y = (i % cols) * 16, (i // cols) * 8
        put(img, mask, tile8(first + i, 0), x, y, palno, pal)
        put(img, mask, tile8(first + i, 1), x + 8, y, palno, pal)
    return img


# ---------------------------------------------------------------- camera / parallax (see SPEC.md section 6)
S3_SHIFT_TABLE = 0x22DE4    # word offset[stage] -> byte shift[area]
S3_YFOLLOW_TABLE = 0x22E54  # byte[stage*4+area], nonzero = scroll3 Y follows camera Y


def s3_shift(stage, area):
    return b(S3_SHIFT_TABLE + w(S3_SHIFT_TABLE + 2 * stage) + area)


def scroll3_camera(stage, area, camx, camy):
    """Standard scroll3 follow (routine 0x22E0A): X = camX >> shift; Y fixed at the scroll3 area's
    initial Y unless the per-area flag is set, then Y = ((camY - scroll2 area initY) >> shift) + scroll3 initY."""
    sh = s3_shift(stage, area)
    r2 = area_record(S2_HDR_TABLE, stage, area)
    r3 = area_record(S3_HDR_TABLE, stage, area)
    x3 = (camx & 0xFFFF) >> sh
    y3 = r3["y"]
    if b(S3_YFOLLOW_TABLE + stage * 4 + area):
        y3 = ((camy - r2["y"]) >> sh) + r3["y"]
    return x3, y3


def view(layers, camx_list, camy_list):
    """Compose 384x224 screens. layers = [(panorama_rgb, mask, rec_height_screens, cam_fn)], back to front.
    Screen pixel (sx, s) shows world x = camX + sx, world y = camY + 0xEF - s for every layer."""
    out = []
    for camx, camy in zip(camx_list, camy_list):
        scr = np.zeros((224, 384, 3), np.uint8)
        for img, mask, hs, camfn in layers:
            cx, cy = camfn(camx, camy)
            top = hs * 256 - 0xF0 - cy          # panorama row of screen line 0
            ys = np.arange(224) + top
            xs = np.arange(384) + cx
            vy = (ys >= 0) & (ys < img.shape[0])
            vx = (xs >= 0) & (xs < img.shape[1])
            sub_y = np.clip(ys, 0, img.shape[0] - 1)
            sub_x = np.clip(xs, 0, img.shape[1] - 1)
            pix = img[sub_y][:, sub_x]
            m = mask[sub_y][:, sub_x].astype(bool) & vy[:, None] & vx[None, :]
            scr[m] = pix[m]
        out.append(scr)
    return out


def save(img, path, scale=1):
    im = Image.fromarray(img)
    if scale != 1:
        im = im.resize((im.width * scale, im.height * scale), Image.NEAREST)
    im.save(path)
    print("wrote", path, im.size)


def contact_sheet(out):
    shots = []
    for st in range(8):
        r2, r3 = area_record(S2_HDR_TABLE, st, 0), area_record(S3_HDR_TABLE, st, 0)
        i2, m2, _ = render_scroll2(st, 0, bank(l(PAL_S2_TABLE + 4 * st * 4)))
        i3, m3, _ = render_scroll3(st, 0, bank(l(PAL_S3_TABLE + 4 * st * 4)))
        layers = [(i3, m3, r3["h"], lambda x, y, st=st: scroll3_camera(st, 0, x, y)),
                  (i2, m2, r2["h"], lambda x, y: (x, y))]
        shots.append(np.pad(view(layers, [0x300], [r2["y"]])[0], ((0, 4), (0, 4), (0, 0))))
    rows = [np.concatenate(shots[i:i + 2], axis=1) for i in range(0, 8, 2)]
    save(np.concatenate(rows, axis=0), f"{out}/contact_stages_area0.png")


def main():
    out = sys.argv[2] if len(sys.argv) > 2 else os.path.join(os.path.dirname(os.path.abspath(__file__)), "png")
    os.makedirs(out, exist_ok=True)
    if len(sys.argv) > 1 and sys.argv[1] == "contact":
        contact_sheet(out)
        return
    stage = int(sys.argv[1]) if len(sys.argv) > 1 else 0
    for area in range(N_AREAS[stage]):
        r2 = area_record(S2_HDR_TABLE, stage, area)
        r3 = area_record(S3_HDR_TABLE, stage, area)
        p2 = bank(l(PAL_S2_TABLE + 4 * (stage * 4 + area)))
        p3 = bank(l(PAL_S3_TABLE + 4 * (stage * 4 + area)))
        layers = []
        if r3["w"] and r3["screens"]:
            img3, m3, _ = render_scroll3(stage, area, p3)
            save(img3, f"{out}/st{stage}_a{area}_scroll3.png")
            layers.append((img3, m3, r3["h"], lambda x, y, a=area: scroll3_camera(stage, a, x, y)))
        if r2["w"]:
            img2, m2, _ = render_scroll2(stage, area, p2)
            save(img2, f"{out}/st{stage}_a{area}_scroll2.png")
            layers.append((img2, m2, r2["h"], lambda x, y: (x, y)))
            # filmstrip of in-game screens along the area at the initial camera Y (parallax check)
            xs = list(range(r2["x"], r2["w"] * 256 - 384 + 1, 384))[:6]
            shots = view(layers, xs, [r2["y"]] * len(xs))
            strip = np.concatenate([np.pad(sh, ((0, 4), (0, 0), (0, 0))) for sh in shots], axis=0)
            save(strip, f"{out}/st{stage}_a{area}_screens.png")
    if stage == 6:
        img1, _ = render_scroll1_bg((1, 2), bank(PAL_S1))
        save(img1, f"{out}/st6_a1_scroll1_elevator.png")
    save(font_sheet(bank(PAL_S1), 0, 0, 512), f"{out}/scroll1_font.png", 2)


if __name__ == "__main__":
    main()
