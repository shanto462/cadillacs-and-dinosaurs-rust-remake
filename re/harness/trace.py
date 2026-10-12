"""Reader for the harness trace format (MAME `record.lua` and the port's `--trace`).

File layout (all integers little-endian):

    "DINOTRC1"                      8-byte magic
    u32 n, n bytes of JSON          header: {"source", "block", "regions": [{"name", "base", "size"}], ...}
    records until the end of file:
        i32 frame                   -1 = the sync state, then 0, 1, 2, ...
        u32 flags                   bit 0: lag (MAME only: the game frame overran a vblank)
        u32 vblank                  MAME vblank number (0 on the port side)
        u32 lag                     lag vblanks skipped before this record
        for each region, in header order:
            u32 count
            count x (u32 block index, `block` bytes)   blocks that changed since the last record
        u32 n, n bytes of JSON      per-record extra data ({} or empty; the port puts "snd" here)

The first record holds every block. A reader keeps one image per region and applies the blocks.

MAME regions: ram (0xFF0000, 64 KB), gfx (0x900000, 192 KB), cpsa, cpsb.
Port regions: ram + ram_mask (which bytes the port models), gfx + gfx_mask (tilemaps and palette),
obj (the sprite list built this frame, 8 bytes per entry, ends with attr 0xFF00).
"""

import json
import struct

import numpy as np

MAGIC = b"DINOTRC1"


class Record:
    __slots__ = ("frame", "flags", "vblank", "lag", "images", "extra")


class Trace:
    def __init__(self, path):
        self.path = path
        with open(path, "rb") as f:
            self.data = f.read()
        if self.data[:8] != MAGIC:
            raise ValueError(f"{path}: not a trace file")
        (n,) = struct.unpack_from("<I", self.data, 8)
        self.header = json.loads(self.data[12 : 12 + n])
        self.pos0 = 12 + n
        self.block = self.header["block"]
        self.regions = self.header["regions"]
        self.source = self.header.get("source", "?")

    def region(self, name):
        for r in self.regions:
            if r["name"] == name:
                return r
        return None

    def records(self):
        """Yields one Record per frame. `images` maps region name -> numpy uint8 array; the
        arrays are updated in place, so copy what you need to keep."""
        d = self.data
        pos = self.pos0
        bs = self.block
        images = {r["name"]: np.zeros(r["size"], dtype=np.uint8) for r in self.regions}
        while pos < len(d):
            rec = Record()
            rec.frame, rec.flags, rec.vblank, rec.lag = struct.unpack_from("<iIII", d, pos)
            pos += 16
            for r in self.regions:
                img = images[r["name"]]
                (count,) = struct.unpack_from("<I", d, pos)
                pos += 4
                for _ in range(count):
                    (idx,) = struct.unpack_from("<I", d, pos)
                    pos += 4
                    img[idx * bs : idx * bs + bs] = np.frombuffer(d, dtype=np.uint8, count=bs, offset=pos)
                    pos += bs
            (n,) = struct.unpack_from("<I", d, pos)
            pos += 4
            rec.extra = json.loads(d[pos : pos + n]) if n else {}
            pos += n
            rec.images = images
            yield rec


def load_state(path):
    """A state file is a trace with one record (frame -1). Returns {region name: bytes}."""
    t = Trace(path)
    for rec in t.records():
        return t, {k: v.copy() for k, v in rec.images.items()}
    raise ValueError(f"{path}: empty")


# ------------------------------------------------------------------------------------------
# Views of the MAME memory image
# ------------------------------------------------------------------------------------------

RAM_BASE = 0xFF0000
GFX_BASE = 0x900000


def u8(img, addr, base=RAM_BASE):
    return int(img[addr - base])


def u16(img, addr, base=RAM_BASE):
    o = addr - base
    return (int(img[o]) << 8) | int(img[o + 1])


def u32(img, addr, base=RAM_BASE):
    return (u16(img, addr, base) << 16) | u16(img, addr + 2, base)


def mame_sprite_list(ram, gfx):
    """The sprite list built in this game frame: build_sprite_list (0x13FA6) writes 0x904000 when
    $69B4 was 0, then flips $69B4, so after the frame $69B4 != 0 means 0x904000."""
    buf = 0x904000 if u16(ram, 0xFFE9B4) != 0 else 0x900000
    return sprite_entries(gfx[buf - GFX_BASE : buf - GFX_BASE + 0x800])


def sprite_entries(b):
    """(x, y, code, attr) entries up to the end marker (attr 0xFF00), 256 at most."""
    out = []
    for i in range(0, min(len(b), 0x800), 8):
        x, y, code, attr = struct.unpack_from(">HHHH", bytes(b[i : i + 8]))
        if attr == 0xFF00:
            break
        out.append((x, y, code, attr))
    return out


def mame_sounds(prev_ram, ram):
    """Sound requests queued this frame: ring at $3A(a5), 16-byte entries (word code, long d1,
    long d2), write index $34(a5) (enqueue 0xA63C)."""
    if prev_ram is None:
        return []
    a = u16(prev_ram, 0xFF8034) & 0x3F0
    b = u16(ram, 0xFF8034) & 0x3F0
    out = []
    while a != b:
        e = 0xFF803A + a
        out.append((u16(ram, e), u32(ram, e + 2), u32(ram, e + 6)))
        a = (a + 0x10) & 0x3F0
    return out
