"""Finds every animation table (and frame record chain) in the 68000 program by brute force.

A table candidate is any 24-bit value found in the program (as a 32-bit immediate or a data pointer, or as a
PC-relative LEA target) that passes a strict validation: a list of u16 offsets whose targets are all valid
frame record chains (10 or 12 bytes per record, see re/specs/anim/SPEC.md section 3).
The owner of a table is the object class whose code range holds the reference to it (see owners.py).
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from romio import Rom

CODE_END = 0x100000          # sprite data lives below the hit box region at 0x100000
CHAIN_LIMIT = 400


class Scanner:
    def __init__(self, rom):
        self.rom = rom
        self.b = rom.b
        self.n_comp = rom.u16(0x0FFFF8)
        self.comp_ok_cache = {}

    def comp_ok(self, idx):
        return idx % 4 == 0 and (idx >> 2) < self.n_comp

    def compound_ok(self, p):
        """Compound frame: w0 bit15, w1 = self relative offset (from +2) to a part list."""
        r = self.rom
        off = r.i16(p + 2)
        q = p + 2 + off
        if q < 0 or q + 2 > CODE_END or q & 1:
            return False
        n = r.u16(q) + 1
        if not 1 <= n <= 24 or q + 2 + 8 * n > CODE_END:
            return False
        for k in range(n):
            if not self.comp_ok(r.u16(q + 2 + 8 * k + 4)):
                return False
        return True

    def frame_ok(self, p, stride):
        r = self.rom
        if p & 1 or p < 0x1000 or p + stride + 2 > CODE_END:
            return False
        w0, w1 = r.u16(p), r.u16(p + 2)
        if w0 & 0x8000:
            if not self.compound_ok(p):
                return False
        else:
            if not self.comp_ok(w0):
                return False
            if not (w1 & 0x8000) and not self.comp_ok(w1):
                return False
        if self.b[p + 6] > 0x20:
            return False
        return True

    def chain(self, start, stride, seen=None):
        """Walks records from start until the end flag and follows links. Returns the list of record addresses
        reachable (including linked ones) or None when anything is invalid."""
        seen = seen if seen is not None else set()
        out = []
        stack = [start]
        r = self.rom
        while stack:
            p = stack.pop()
            while True:
                if p in seen:
                    break
                if not self.frame_ok(p, stride):
                    return None
                seen.add(p)
                out.append(p)
                if len(seen) > CHAIN_LIMIT:
                    return None
                if self.b[p + 7] & 0x80:
                    link = p + stride + r.i16(p + stride)
                    stack.append(link)
                    break
                p += stride
        return out

    def table(self, t, stride, tolerant=False):
        """Validates an animation table at t. Returns (n_ids, offsets, frames) or None. In tolerant mode ids whose
        chain is invalid are dropped (their offset is zeroed) when most ids are valid (tables from the master list)."""
        r = self.rom
        if t & 1 or t < 0x1000 or t + 4 > CODE_END:
            return None
        first = 0xFFFF
        offs = []
        i = 0
        while 2 * i < first:
            if t + 2 * i + 2 > CODE_END:
                return None
            o = r.u16(t + 2 * i)
            if o:
                if o & 1 or o >= 0x7000:
                    if not tolerant:
                        return None
                    o = 0
                else:
                    first = min(first, o)
            offs.append(o)
            i += 1
            if i > 300:
                return None
        n = len(offs)
        if n < 1 or first == 0xFFFF:
            return None
        frames = set()
        bad = 0
        for i, o in enumerate(offs):
            if o:
                ch = self.chain(t + o, stride, frames)
                if ch is None:
                    if not tolerant:
                        return None
                    offs[i] = 0
                    bad += 1
        if tolerant and (bad * 3 > sum(1 for o in offs if o) + bad):
            return None
        return n, offs, sorted(frames)

    def find_tables(self):
        """Returns {table_addr: dict(stride, n, offs, frames, refs)}."""
        r = self.rom
        b = self.b
        refs = {}                 # candidate -> list of (ref_addr, kind)
        n = len(b)
        for a in range(0, n - 3, 2):
            if b[a] != 0:
                continue
            v = (b[a + 1] << 16) | (b[a + 2] << 8) | b[a + 3]
            if 0x1000 <= v < CODE_END and not v & 1:
                refs.setdefault(v, []).append((a, "ptr"))
        for a in range(0, n - 4, 2):
            w = (b[a] << 8) | b[a + 1]
            if (w & 0xF1FF) == 0x41FA:              # lea d16(pc),An
                d = r.i16(a + 2)
                v = a + 2 + d
                if 0x1000 <= v < CODE_END and not v & 1:
                    refs.setdefault(v, []).append((a, "lea_pc"))
        self.refs = refs
        found = {}
        for v in sorted(refs):
            for stride in (10, 12):
                res = self.table(v, stride)
                if res:
                    n_ids, offs, frames = res
                    found[v] = dict(stride=stride, n=n_ids, offs=offs, frames=frames, refs=refs[v])
                    break
        return found


def load_scan(rom=None):
    rom = rom or Rom()
    sc = Scanner(rom)
    return sc, sc.find_tables()


if __name__ == "__main__":
    import time
    t0 = time.time()
    sc, found = load_scan()
    print(len(found), "tables in", round(time.time() - t0, 1), "s")
    for v in sorted(found)[:400]:
        f = found[v]
        print(hex(v), f["stride"], f["n"], len(f["frames"]), [hex(a) for a, k in f["refs"][:4]])
