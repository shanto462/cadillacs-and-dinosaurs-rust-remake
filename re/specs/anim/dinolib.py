"""Small library: decode Cadillacs and Dinosaurs (dino) sprite compositions, animations and hitboxes.
Mirrors the 68000 routines at 0x14230 (draw), 0x1236/0x143E (anim frame load) and 0x128F8 (box test).
All addresses are 68000 program addresses (program image loaded at 0)."""
import struct
import numpy as np

COMP_TABLE_PTR = 0x0FFFF4    # long -> 0x0D0CF4 (composition pointer table)
COMP_COUNT_PTR = 0x0FFFF8    # word -> 0x0DB9 (3513 compositions)
LAYOUT_TABLE_PTR = 0x0FFFFA  # long -> 0x0D43D8 (layout pointer table)
LAYOUT_COUNT_PTR = 0x0FFFFE  # word -> 0x09D4 (2516 layouts)
COMP_EXTRA_TABLE = 0x11D220  # long per composition (wading tile skip counts, splash type) -> obj +$4C..+$4F
OBJ_PAL_SET_TABLE = 0x008E8E  # long ptr per (stage*4 + area) -> 32 words (ROM palette numbers)
PAL_BANK = 0x0B7A52           # 32 bytes per ROM palette
PLAYER_ANIM_SETS = [0x163E, 0x164E, 0x165E, 0x166E, 0x167E, 0x168E]  # 6 sets x 4 characters (long ptrs)
PLAYER_BOX_TABLES = 0x018CC4  # 4 longs: box table per character (obj +$44)
CHAR_NAMES = ['Jack', 'Mustapha', 'Hannah', 'Mess']   # obj +$03 order, identified from the rendered graphics


class Rom:
    def __init__(self, prg_path, gfx_path):
        self.b = open(prg_path, 'rb').read()
        self.g = np.load(gfx_path)
        self.comp_table = self.l(COMP_TABLE_PTR)
        self.n_comps = self.w(COMP_COUNT_PTR)
        self.layout_table = self.l(LAYOUT_TABLE_PTR)
        self.n_layouts = self.w(LAYOUT_COUNT_PTR)
        self._tiles = {}

    # --- raw reads (big-endian) ---
    def w(self, a): return struct.unpack('>H', self.b[a:a + 2])[0]
    def sw(self, a): return struct.unpack('>h', self.b[a:a + 2])[0]
    def l(self, a): return struct.unpack('>I', self.b[a:a + 4])[0]

    # --- graphics ---
    def tile(self, code):
        code &= 0x7FFF
        t = self._tiles.get(code)
        if t is None:
            raw = self.g[code * 128:(code + 1) * 128].reshape(16, 2, 4)      # row, half, plane byte
            bits = np.unpackbits(raw, axis=2).reshape(16, 2, 4, 8)
            px = sum(bits[:, :, p, :].astype(np.uint8) << p for p in range(4))
            t = px.reshape(16, 16)
            self._tiles[code] = t
        return t

    def palette(self, rom_pal):
        """ROM palette -> 16 RGBA tuples. Brightness nibble forced to full (as after fade-in)."""
        a = PAL_BANK + rom_pal * 32
        out = []
        for i in range(16):
            v = self.w(a + 2 * i)
            r, g, b = (v >> 8) & 15, (v >> 4) & 15, v & 15
            out.append((r * 17, g * 17, b * 17, 0 if i == 15 else 255))
        return out

    def obj_palettes(self, stage=0, area=0):
        """The 32 OBJ palettes loaded by 0x8E4C for a stage/area: table 0x8E8E[stage*4+area] -> 32 words."""
        lst = self.l(OBJ_PAL_SET_TABLE + (stage * 16 + area * 4))
        return [self.palette(self.w(lst + 2 * i)) for i in range(32)], [self.w(lst + 2 * i) for i in range(32)]

    # --- compositions (0x14230 / 0x142AA / 0x14494) ---
    def comp(self, idx_bytes):
        """idx_bytes = frame word (byte offset into the long pointer table)."""
        a = self.l(self.comp_table + idx_bytes)
        count, n, hattr, lay = self.w(a), self.w(a + 2), self.w(a + 4), self.w(a + 6)
        la = self.l(self.layout_table + lay * 4)
        ents = []
        for k in range(n):
            dx, dy = self.sw(la + 4 * k), self.sw(la + 4 * k + 2)
            if hattr & 0x8000:
                code, attr = self.w(a + 8 + 4 * k), self.w(a + 10 + 4 * k)
            else:
                code, attr = self.w(a + 8 + 2 * k), hattr
            ents.append((dx, dy, code, attr))
        return dict(addr=a, count=count, n=n, hattr=hattr, layout=lay, alt=bool(hattr & 0x8000), ents=ents)

    def comp_sprites(self, idx_bytes, facing=0, pal_override=None):
        """Return hardware-style sprites (x, y, code, attr) relative to the object origin (feet point),
        exactly as 0x142BE (normal) / 0x14494 (alt) emit them. y grows downward (screen space)."""
        c = self.comp(idx_bytes)
        out = []
        for dx, dy, code, attr in c['ents']:
            if pal_override is not None:
                attr = (attr & 0xFFE0) | (pal_override & 0x1F)
            if facing:
                attr ^= 0x20
                if c['alt']:
                    wpx = ((attr >> 4) & 0xF0) + 0x10      # block width in pixels
                    x = -dx - wpx
                else:
                    x = -dx - 0x10                          # normal format: always 16 (blocks are 1x1 there)
            else:
                x = dx
            out.append((x, dy, code, attr))
        return out

    # --- frame records ---
    def frame_sprites(self, frame_addr, facing=0, pal_override=None):
        """0x14230: word0 = main composition (negative => compound frame, 0x14180), word1 = overlay (negative => none)."""
        w0, w1 = self.w(frame_addr), self.w(frame_addr + 2)
        out = []
        if w0 & 0x8000:
            # compound: a3 = frame+2; a3 += word1 ; [n-1] then n x (dx, dy, comp, flag)
            p = frame_addr + 2 + self.sw(frame_addr + 2)
            n = self.w(p) + 1
            p += 2
            for _ in range(n):
                dx, dy, ci = self.sw(p), self.sw(p + 2), self.w(p + 4)
                p += 8
                if facing:
                    dx = -dx
                for x, y, code, attr in self.comp_sprites(ci, facing, pal_override):
                    out.append((x + dx, y - dy, code, attr))
            return out
        out += self.comp_sprites(w0, facing, pal_override)
        if not (w1 & 0x8000):
            out += self.comp_sprites(w1, facing, pal_override)
        return out

    # --- hitboxes ---
    def box(self, table, idx):
        a = table + idx * 12
        return tuple(self.sw(a + 2 * k) for k in range(6))   # zoff, zsize, xoff, xsize, yoff, ysize


def anim_frames(rom, start, rec=12, maxf=200):
    """Walk records from start until the end flag (byte +7 bit7); return (frames, next_ptr)."""
    out, a = [], start
    while len(out) < maxf:
        out.append(a)
        if rom.b[a + 7] & 0x80:
            nxt = a + rec + rom.sw(a + rec)
            return out, nxt
        a += rec
    return out, None


def anim_table_entries(rom, tbl):
    n = rom.w(tbl) // 2
    return [tbl + rom.w(tbl + 2 * i) for i in range(n)]


def draw_sprites(rom, img, sprites, ox, oy, pals):
    """Rasterise CPS1 sprites (with block expansion like MAME cps1 render) onto RGBA numpy img.
    Draw from last to first so that the first sprite in the list ends on top (CPS1 priority)."""
    H, W = img.shape[:2]
    for x, y, code, attr in reversed(sprites):
        nx, ny = ((attr >> 8) & 15) + 1, ((attr >> 12) & 15) + 1
        fx, fy = attr & 0x20, attr & 0x40
        pal = pals[attr & 0x1F]
        for j in range(ny):
            for i in range(nx):
                ti = (nx - 1 - i) if fx else i
                tj = (ny - 1 - j) if fy else j
                c = (code & ~0xF) + ((code + ti) & 0xF) + 0x10 * tj
                t = rom.tile(c)
                if fx: t = t[:, ::-1]
                if fy: t = t[::-1, :]
                px, py = ox + x + 16 * i, oy + y + 16 * j
                for yy in range(16):
                    sy = py + yy
                    if sy < 0 or sy >= H: continue
                    for xx in range(16):
                        sx = px + xx
                        if sx < 0 or sx >= W: continue
                        p = t[yy, xx]
                        if p != 15:
                            img[sy, sx] = pal[p]
