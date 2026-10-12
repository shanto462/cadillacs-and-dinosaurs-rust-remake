"""Sprite composer: draws compositions, frame records and raw sprite lists with the game's own rules
(re/specs/anim/SPEC.md sections 2 and 3), plus small image helpers (labels, sheets, GIFs)."""
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

BG = (54, 58, 80)
GRID = (74, 78, 104)
_FONTS = {}


def font(size=10):
    f = _FONTS.get(size)
    if f is None:
        try:
            f = ImageFont.load_default(size=size)
        except TypeError:
            f = ImageFont.load_default()
        _FONTS[size] = f
    return f


class Painter:
    def __init__(self, rom):
        self.rom = rom
        self.n_comp = rom.u16(0x0FFFF8)
        self.comp_table = rom.u32(0x0FFFF4)
        self.layout_table = rom.u32(0x0FFFFA)
        self._pal = {}
        self._comp = {}

    # ------------------------------------------------------------------ palettes
    def obj_set(self, stage=0, area=0):
        """The 32 OBJ palettes the game loads for (stage, area): uint8 [32,16,3] and the ROM palette numbers."""
        key = (stage, area)
        if key not in self._pal:
            r = self.rom
            lst = r.u32(0x8E8E + stage * 16 + area * 4)
            nums = [r.u16(lst + 2 * i) for i in range(32)]
            pals = np.array([r.pal16(0xB7A52 + n * 32) for n in nums], np.uint8)
            self._pal[key] = (pals, nums)
        return self._pal[key]

    def grey(self):
        g = np.array([[(i * 17,) * 3 for i in range(16)]] * 32, np.uint8)
        return g

    # ------------------------------------------------------------------ compositions
    def comp(self, idx):
        """Composition at byte index idx: dict(addr, alt, ents=[(dx, dy, code, attr)])."""
        c = self._comp.get(idx)
        if c is None:
            r = self.rom
            a = r.u32(self.comp_table + idx)
            n, hattr, lay = r.u16(a + 2), r.u16(a + 4), r.u16(a + 6)
            la = r.u32(self.layout_table + lay * 4)
            ents = []
            for k in range(n):
                dx, dy = r.i16(la + 4 * k), r.i16(la + 4 * k + 2)
                if hattr & 0x8000:
                    code, attr = r.u16(a + 8 + 4 * k), r.u16(a + 10 + 4 * k)
                else:
                    code, attr = r.u16(a + 8 + 2 * k), hattr
                ents.append((dx, dy, code, attr))
            c = dict(addr=a, alt=bool(hattr & 0x8000), ents=ents, layout=lay, count=r.u16(a))
            self._comp[idx] = c
        return c

    def comp_sprites(self, idx, facing=0, override=None):
        c = self.comp(idx)
        out = []
        for dx, dy, code, attr in c["ents"]:
            if override is not None:
                attr = (attr & 0xFFE0) | (override & 0x1F)
            if facing:
                attr ^= 0x20
                x = (-dx - (((attr >> 4) & 0xF0) + 16)) if c["alt"] else (-dx - 16)
            else:
                x = dx
            out.append((x, dy, code, attr))
        return out

    def frame_sprites(self, f, facing=0, override=None):
        r = self.rom
        w0, w1 = r.u16(f), r.u16(f + 2)
        out = []
        if w0 & 0x8000:
            p = f + 2 + r.i16(f + 2)
            n = r.u16(p) + 1
            p += 2
            for _ in range(n):
                dx, dy, ci = r.i16(p), r.i16(p + 2), r.u16(p + 4)
                p += 8
                if facing:
                    dx = -dx
                if ci // 4 >= self.n_comp or ci % 4:
                    continue
                for x, y, code, attr in self.comp_sprites(ci, facing, override):
                    out.append((x + dx, y - dy, code, attr))
            return out
        if w0 // 4 < self.n_comp and w0 % 4 == 0:
            out += self.comp_sprites(w0, facing, override)
        if not (w1 & 0x8000) and w1 // 4 < self.n_comp and w1 % 4 == 0:
            out += self.comp_sprites(w1, facing, override)
        return out

    def raw_sprites(self, rec, facing=0, override=None):
        """8-byte record: u32 sprite list, list = u16 count, u16 n, n x (dx, dy, attr, code)."""
        r = self.rom
        lst = r.u32(rec)
        n = r.u16(lst + 2)
        out = []
        for k in range(n):
            q = lst + 4 + 8 * k
            dx, dy, attr, code = r.i16(q), r.i16(q + 2), r.u16(q + 4), r.u16(q + 6)
            if override is not None:
                attr = (attr & 0xFFE0) | (override & 0x1F)
            if facing:
                attr ^= 0x20
                x = -dx - (((attr >> 4) & 0xF0) + 16)
            else:
                x = dx
            out.append((x, dy, code, attr))
        return out

    # ------------------------------------------------------------------ drawing
    def block(self, code, attr, pal):
        """RGBA image (h*16, w*16) of one hardware sprite block."""
        nx, ny = ((attr >> 8) & 15) + 1, ((attr >> 12) & 15) + 1
        fx, fy = attr & 0x20, attr & 0x40
        t16 = self.rom.t16
        pens = np.empty((16 * ny, 16 * nx), np.uint8)
        for j in range(ny):
            for i in range(nx):
                ti = (nx - 1 - i) if fx else i
                tj = (ny - 1 - j) if fy else j
                c = ((code & ~0xF) + ((code + ti) & 0xF) + 0x10 * tj) & 0x7FFF
                t = t16[c]
                if fx:
                    t = t[:, ::-1]
                if fy:
                    t = t[::-1, :]
                pens[16 * j:16 * j + 16, 16 * i:16 * i + 16] = t
        rgb = pal[attr & 0x1F][pens]
        rgba = np.empty(pens.shape + (4,), np.uint8)
        rgba[..., :3] = rgb
        rgba[..., 3] = np.where(pens == 15, 0, 255)
        return rgba

    @staticmethod
    def bbox(sprites):
        if not sprites:
            return (0, 0, 16, 16)
        x0 = min(s[0] for s in sprites)
        y0 = min(s[1] for s in sprites)
        x1 = max(s[0] + 16 * (((s[3] >> 8) & 15) + 1) for s in sprites)
        y1 = max(s[1] + 16 * (((s[3] >> 12) & 15) + 1) for s in sprites)
        return (x0, y0, x1, y1)

    def draw(self, canvas, sprites, ox, oy, pals):
        """Draws onto an RGBA canvas; the first sprite of the list ends on top (CPS1 priority)."""
        H, W = canvas.shape[:2]
        for x, y, code, attr in reversed(sprites):
            blk = self.block(code, attr, pals)
            px, py = ox + x, oy + y
            h, w = blk.shape[:2]
            sx0, sy0 = max(px, 0), max(py, 0)
            sx1, sy1 = min(px + w, W), min(py + h, H)
            if sx0 >= sx1 or sy0 >= sy1:
                continue
            sub = blk[sy0 - py:sy1 - py, sx0 - px:sx1 - px]
            m = sub[..., 3] != 0
            dst = canvas[sy0:sy1, sx0:sx1]
            dst[m] = sub[m]

    def render(self, sprites, pals, margin=2, minsize=(16, 16)):
        """Tight RGBA image of a sprite list. Returns (image, ox, oy) where (ox, oy) is the object origin in the image."""
        x0, y0, x1, y1 = self.bbox(sprites)
        W, H = max(x1 - x0, minsize[0]) + 2 * margin, max(y1 - y0, minsize[1]) + 2 * margin
        canvas = np.zeros((H, W, 4), np.uint8)
        ox, oy = margin - x0, margin - y0
        self.draw(canvas, sprites, ox, oy, pals)
        return canvas, ox, oy


# ---------------------------------------------------------------------------------------- image helpers
def to_pil(rgba, bg=BG, scale=1):
    im = Image.fromarray(rgba, "RGBA")
    base = Image.new("RGBA", im.size, bg + (255,))
    base.alpha_composite(im)
    if scale != 1:
        base = base.resize((base.width * scale, base.height * scale), Image.NEAREST)
    return base.convert("RGB")


def text(draw, xy, s, fill=(255, 255, 255), size=10):
    draw.text(xy, s, fill=fill, font=font(size))


def swatch_image(pals, labels=None, cell=14, cols=1, title=None, hdr=18):
    """pals: uint8 [n,16,3]. One row per palette: 16 swatches + label."""
    n = len(pals)
    rows = (n + cols - 1) // cols
    lab_w = 70
    W = cols * (16 * cell + lab_w + 8) + 8
    H = rows * (cell + 3) + hdr + 6
    im = Image.new("RGB", (W, H), (30, 30, 38))
    d = ImageDraw.Draw(im)
    if title:
        text(d, (6, 3), title, size=11)
    for i, p in enumerate(pals):
        c, r = divmod(i, rows)
        x0 = 6 + c * (16 * cell + lab_w + 8)
        y0 = hdr + r * (cell + 3)
        lab = labels[i] if labels else str(i)
        text(d, (x0, y0 + 1), lab, size=9)
        for k in range(16):
            col = tuple(int(v) for v in p[k])
            d.rectangle([x0 + lab_w + k * cell, y0, x0 + lab_w + (k + 1) * cell - 1, y0 + cell - 1], fill=col)
            if k == 15:
                d.line([x0 + lab_w + k * cell, y0, x0 + lab_w + (k + 1) * cell - 1, y0 + cell - 1], fill=(200, 60, 60))
    return im


def save_gif(path, frames, durations_ms):
    """frames: list of PIL RGB images, equal size. One shared palette so colours do not flicker."""
    w, h = frames[0].size
    stack = Image.new("RGB", (w, h * len(frames)))
    for i, f in enumerate(frames):
        stack.paste(f, (0, i * h))
    pal = stack.quantize(colors=200, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    out = [f.quantize(palette=pal, dither=Image.Dither.NONE) for f in frames]
    out[0].save(path, save_all=True, append_images=out[1:], duration=durations_ms, loop=0, optimize=False, disposal=2)
