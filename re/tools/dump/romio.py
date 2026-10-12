"""Loads the user's dino.zip (MAME set "dino") at run time and decodes graphics, palettes, tables.

Used by the asset dump tools in this folder. Nothing from the ROM is stored in the repo: every output goes
to re/out/assets/ (git-ignored). Run every tool with the project venv in isolated mode:
    re/.venv/bin/python -I re/tools/dump/<tool>.py
"""
import os
import struct
import sys
import zipfile
import zlib

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
RE = os.path.dirname(os.path.dirname(HERE))          # .../re
ROOT = os.path.dirname(RE)                           # repo root
OUT = os.path.join(RE, "out", "assets")

CRC = {
    "cde_23a.8f": 0x8f4e585e, "cde_22a.7f": 0x9278aa12, "cde_21a.6f": 0x66d23de2,
    "cd-1m.3a": 0x8da4f917, "cd-3m.5a": 0x6c40f603, "cd-2m.4a": 0x09c8fc2d, "cd-4m.6a": 0x637ff38f,
    "cd-5m.7a": 0x470befee, "cd-7m.9a": 0x22bfb7a3, "cd-6m.8a": 0xe7599ac4, "cd-8m.10a": 0x211b4b15,
}
GFX_ORDER = [("cd-1m.3a", 0x000000, 0), ("cd-3m.5a", 0x000000, 2), ("cd-2m.4a", 0x000000, 4), ("cd-4m.6a", 0x000000, 6),
             ("cd-5m.7a", 0x200000, 0), ("cd-7m.9a", 0x200000, 2), ("cd-6m.8a", 0x200000, 4), ("cd-8m.10a", 0x200000, 6)]
Z80_KEYS = dict(swap1=0x76543210, swap2=0x24601357, addr=0x4343, xor=0x43)


def find_zip():
    for p in (os.environ.get("DINO_ROM"), os.path.join(ROOT, "dino.zip"), "dino.zip", "../dino.zip"):
        if p and os.path.exists(p):
            return os.path.abspath(p)
    sys.exit("dino.zip not found: set DINO_ROM or put it in the repo root")


def outdir(*parts):
    p = os.path.join(OUT, *parts)
    os.makedirs(p, exist_ok=True)
    return p


# ------------------------------------------------------------------------------------------ Z80 Kabuki
def _swap_pair(src, pair):
    lo = 1 << (2 * pair)
    hi = lo << 1
    return (src & ~(lo | hi) & 0xFF) | ((src & lo) << 1) | ((src & hi) >> 1)


def _bitswap1(src, key, select):
    for pair, shift in enumerate((0, 4, 8, 12)):
        if select & (1 << ((key >> shift) & 7)):
            src = _swap_pair(src, pair)
    return src


def _bitswap2(src, key, select):
    for pair, shift in enumerate((12, 8, 4, 0)):
        if select & (1 << ((key >> shift) & 7)):
            src = _swap_pair(src, pair)
    return src


def _rol(v):
    return ((v << 1) | (v >> 7)) & 0xFF


def _bytedecode(src, select):
    k = Z80_KEYS
    src = _bitswap1(src, k["swap1"] & 0xFFFF, select & 0xFF)
    src = _rol(src)
    src = _bitswap2(src, k["swap1"] >> 16, select & 0xFF)
    src ^= k["xor"]
    src = _rol(src)
    src = _bitswap2(src, k["swap2"] & 0xFFFF, select >> 8)
    src = _rol(src)
    src = _bitswap1(src, k["swap2"] >> 16, select >> 8)
    return src


def kabuki_decrypt(raw):
    """Returns (opcode image, data image). Only the first 0x8000 bytes are encrypted."""
    op, data = bytearray(raw), bytearray(raw)
    for a in range(0x8000):
        op[a] = _bytedecode(raw[a], (a + Z80_KEYS["addr"]) & 0xFFFF)
        data[a] = _bytedecode(raw[a], ((a ^ 0x1FC0) + Z80_KEYS["addr"] + 1) & 0xFFFF)
    return bytes(op), bytes(data)


# ------------------------------------------------------------------------------------------ ROM object
class Rom:
    def __init__(self, path=None):
        path = path or find_zip()
        z = zipfile.ZipFile(path)
        files = {}
        for name, crc in CRC.items():
            data = z.read(name)
            if zlib.crc32(data) != crc:
                raise SystemExit(f"{name}: CRC mismatch (wrong ROM set?)")
            files[name] = data
        prog = bytearray()
        for name in ("cde_23a.8f", "cde_22a.7f", "cde_21a.6f"):
            a = np.frombuffer(files[name], np.uint8).reshape(-1, 2)
            prog += a[:, ::-1].tobytes()
        self.b = bytes(prog)
        g = np.zeros(0x400000, np.uint8)
        for name, base, off in GFX_ORDER:
            d = np.frombuffer(files[name], np.uint8).reshape(-1, 2)
            idx = base + off + np.arange(d.shape[0]) * 8
            g[idx] = d[:, 0]
            g[idx + 1] = d[:, 1]
        self.g = g
        self.z80_raw = z.read("cd_q.5k")
        self.z80_op, self.z80_data = kabuki_decrypt(self.z80_raw)
        self.qs = [z.read(f"cd-q{i}.{i}k") for i in (1, 2, 3, 4)]
        self.zip_names = z.namelist()
        self._t16 = self._t32 = self._t8 = None

    # raw reads (big endian)
    def u8(self, a): return self.b[a]
    def u16(self, a): return struct.unpack_from(">H", self.b, a)[0]
    def i16(self, a): return struct.unpack_from(">h", self.b, a)[0]
    def u32(self, a): return struct.unpack_from(">I", self.b, a)[0]

    # ---- tiles (vectorised planar decode: 4 bytes = 8 pixels, byte p = bitplane p, MSB = left)
    @property
    def t16(self):
        """uint8 [0x8000, 16, 16] pens of every 16x16 tile (128 bytes each)."""
        if self._t16 is None:
            r = self.g.reshape(-1, 16, 2, 4)
            bits = np.unpackbits(r, axis=3).reshape(-1, 16, 2, 4, 8)
            px = sum(bits[:, :, :, p, :].astype(np.uint8) << p for p in range(4))
            self._t16 = px.reshape(-1, 16, 16)
        return self._t16

    @property
    def t32(self):
        """uint8 [0x2000, 32, 32] pens of every 32x32 tile (512 bytes each)."""
        if self._t32 is None:
            r = self.g.reshape(-1, 32, 4, 4)
            bits = np.unpackbits(r, axis=3).reshape(-1, 32, 4, 4, 8)
            px = sum(bits[:, :, :, p, :].astype(np.uint8) << p for p in range(4))
            self._t32 = px.reshape(-1, 32, 32)
        return self._t32

    @property
    def t8(self):
        """uint8 [0x10000, 2, 8, 8]: every 8x8 tile code (64 bytes each), [code, half, y, x]."""
        if self._t8 is None:
            r = self.g.reshape(-1, 8, 2, 4)
            bits = np.unpackbits(r, axis=3).reshape(-1, 8, 2, 4, 8)
            px = sum(bits[:, :, :, p, :].astype(np.uint8) << p for p in range(4))   # [code, y, half, x]
            self._t8 = px.transpose(0, 2, 1, 3).copy()
        return self._t8

    # ---- colours
    def colour(self, word, bright=0xF000):
        """CPS1 colour word 0xBRGB -> (r, g, b), MAME formula. ROM words carry brightness 0; the game ORs 0xF000."""
        word |= bright
        br = 0x0F + ((word >> 12) << 1)
        return tuple(((word >> s) & 0xF) * 0x11 * br // 0x2D for s in (8, 4, 0))

    def pal16(self, addr, bright=0xF000):
        """16 colours at addr -> uint8 [16, 3]."""
        return np.array([self.colour(self.u16(addr + 2 * i), bright) for i in range(16)], np.uint8)

    def bank(self, addr, n=32, bright=0xF000):
        """n palettes of 16 colours -> uint8 [n, 16, 3]."""
        return np.array([self.pal16(addr + 32 * p, bright) for p in range(n)], np.uint8)


# ------------------------------------------------------------------------------------------ small helpers
def hx(v, n=0):
    return f"0x{v:0{n}X}"


def write_json(path, obj):
    import json
    with open(path, "w") as f:
        json.dump(obj, f, indent=1, sort_keys=False)


STAGE_NAMES = ["Episode 1 (stage 0)", "Episode 2 (stage 1)", "Episode 3 (stage 2)", "Episode 4 (stage 3)",
               "Episode 5 (stage 4)", "Episode 6 (stage 5)", "Episode 7 (stage 6)", "Episode 8 (stage 7)"]
N_AREAS = {0: 3, 1: 3, 2: 1, 3: 3, 4: 2, 5: 3, 6: 4, 7: 3}
