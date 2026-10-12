"""Differential test data for the terrain collision port (phase 1 task 1).

Runs the ORIGINAL 68000 routines on random inputs and writes the results, so that
`cargo test -p dino-game terrain_matches_the_68000_code` can check that the Rust port gives the
same answers:

    re/.venv/bin/python -I re/harness/terrain_oracle.py [VECTORS [SEED]]

The default output is re/out/terrain_vectors.txt (git-ignored, derived from the ROM). The test
skips itself when that file is missing.

This is a tiny 68000 interpreter, just big enough for the routines of 0x12C90..0x13700 (about 50
opcode forms: moves, add / sub / cmp, logic, shifts, branches, bsr / jsr / rts). It reads the
program image `re/out/dino_68k.bin` (built from the user's ROM; never committed) and keeps its own
RAM and video RAM. It stops with an error on an opcode it does not know, so a mistake in the
interpreter shows up loudly. Routines run from their entry address until the matching `rts`.

Memory the routines read from video RAM (scroll2 attributes at 0x90C000, the vertical scene map
at 0x91C000) is made up from a seed, lazily, and every cell that was read is written to the
vector, so the Rust side can set exactly those cells.
"""

import os
import random
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
IMAGE = os.path.join(ROOT, "re", "out", "dino_68k.bin")

A5 = 0xFF8000
SENTINEL = 0x00F00000  # return address that ends a run (an odd-free value outside the ROM)


COVERED = set()  # every instruction address that was executed (see main)


class Stop(Exception):
    pass


def sext(v, bits):
    v &= (1 << bits) - 1
    return v - (1 << bits) if v >> (bits - 1) else v


class CPU:
    def __init__(self, rom, vram_value):
        self.rom = rom
        self.ram = {}
        self.vram_value = vram_value  # function(address) -> word for the made-up video RAM cells
        self.vram_read = {}  # address (even) -> word, the cells that were read
        self.d = [0] * 8
        self.a = [0] * 8
        self.pc = 0
        self.x = self.n = self.z = self.v = self.c = 0
        self.steps = 0
        self.trace = None
        self.a[7] = 0xFFFF00

    # ---- memory ----
    def rb(self, addr):
        addr &= 0xFFFFFF
        if addr < len(self.rom):
            return self.rom[addr]
        if 0x900000 <= addr < 0x930000:
            if addr in self.ram:
                return self.ram[addr]
            w = self.vram_value(addr & ~1)
            self.vram_read[addr & ~1] = w
            return (w >> 8) & 0xFF if addr & 1 == 0 else w & 0xFF
        return self.ram.get(addr, 0)

    def wb(self, addr, v):
        addr &= 0xFFFFFF
        assert addr >= len(self.rom), "write to ROM at %x (pc %x)" % (addr, self.pc)
        self.ram[addr] = v & 0xFF

    def rd(self, addr, size):
        v = 0
        for k in range(size):
            v = (v << 8) | self.rb(addr + k)
        return v

    def wr(self, addr, size, v):
        for k in range(size):
            self.wb(addr + k, (v >> (8 * (size - 1 - k))) & 0xFF)

    def fetch(self):
        w = (self.rom[self.pc] << 8) | self.rom[self.pc + 1]
        self.pc += 2
        return w

    # ---- effective addresses ----
    # An operand is a tuple: ("d", n), ("a", n), ("m", address), ("i", value)
    def ea(self, mode, reg, size):
        if mode == 0:
            return ("d", reg)
        if mode == 1:
            return ("a", reg)
        if mode == 2:
            return ("m", self.a[reg])
        if mode == 3:
            addr = self.a[reg]
            self.a[reg] = (addr + (2 if (reg == 7 and size == 1) else size)) & 0xFFFFFFFF
            return ("m", addr)
        if mode == 4:
            self.a[reg] = (self.a[reg] - (2 if (reg == 7 and size == 1) else size)) & 0xFFFFFFFF
            return ("m", self.a[reg])
        if mode == 5:
            return ("m", (self.a[reg] + sext(self.fetch(), 16)) & 0xFFFFFFFF)
        if mode == 6:
            base = self.a[reg]
            return ("m", (base + self.index_ext()) & 0xFFFFFFFF)
        # mode 7
        if reg == 0:
            return ("m", sext(self.fetch(), 16) & 0xFFFFFFFF)
        if reg == 1:
            hi = self.fetch()
            lo = self.fetch()
            return ("m", (hi << 16) | lo)
        if reg == 2:
            base = self.pc
            return ("m", (base + sext(self.fetch(), 16)) & 0xFFFFFFFF)
        if reg == 3:
            base = self.pc
            return ("m", (base + self.index_ext()) & 0xFFFFFFFF)
        if reg == 4:
            if size == 4:
                hi = self.fetch()
                lo = self.fetch()
                return ("i", (hi << 16) | lo)
            w = self.fetch()
            return ("i", w & (0xFF if size == 1 else 0xFFFF))
        raise Stop("bad ea mode 7 reg %d at %x" % (reg, self.pc))

    def index_ext(self):
        e = self.fetch()
        disp = sext(e & 0xFF, 8)
        r = (e >> 12) & 7
        v = self.a[r] if e & 0x8000 else self.d[r]
        v = sext(v, 32) if e & 0x0800 else sext(v, 16)
        return disp + v

    def read(self, op, size):
        k, v = op
        mask = (1 << (8 * size)) - 1
        if k == "d":
            return self.d[v] & mask
        if k == "a":
            return self.a[v] & mask
        if k == "m":
            return self.rd(v, size)
        return v & mask

    def write(self, op, size, val):
        k, v = op
        mask = (1 << (8 * size)) - 1
        val &= mask
        if k == "d":
            self.d[v] = (self.d[v] & ~mask & 0xFFFFFFFF) | val
        elif k == "a":
            self.a[v] = sext(val, 8 * size) & 0xFFFFFFFF if size == 2 else val
        elif k == "m":
            self.wr(v, size, val)
        else:
            raise Stop("write to immediate")

    # ---- flags ----
    def nz(self, r, size):
        bits = 8 * size
        r &= (1 << bits) - 1
        self.n = (r >> (bits - 1)) & 1
        self.z = 1 if r == 0 else 0

    def logic_flags(self, r, size):
        self.nz(r, size)
        self.v = 0
        self.c = 0

    def add_flags(self, s, d, size, x=False):
        bits = 8 * size
        mask = (1 << bits) - 1
        r = (s & mask) + (d & mask)
        res = r & mask
        self.nz(res, size)
        self.c = 1 if r > mask else 0
        sm, dm, rm = (s >> (bits - 1)) & 1, (d >> (bits - 1)) & 1, (res >> (bits - 1)) & 1
        self.v = 1 if (sm == dm and rm != sm) else 0
        if x:
            self.x = self.c
        return res

    def sub_flags(self, s, d, size):
        # d - s
        bits = 8 * size
        mask = (1 << bits) - 1
        s &= mask
        d &= mask
        res = (d - s) & mask
        self.nz(res, size)
        self.c = 1 if s > d else 0
        sm, dm, rm = (s >> (bits - 1)) & 1, (d >> (bits - 1)) & 1, (res >> (bits - 1)) & 1
        self.v = 1 if (sm != dm and rm != dm) else 0
        return res

    def cond(self, cc):
        n, z, v, c = self.n, self.z, self.v, self.c
        return [
            True, False, (not c and not z), (c or z), not c, bool(c), not z, bool(z),
            not v, bool(v), not n, bool(n), n == v, n != v, (not z and n == v), (z or n != v),
        ][cc]

    # ---- execution ----
    def run(self, entry, max_steps=20000):
        self.pc = entry
        self.a[7] -= 4
        self.wr(self.a[7], 4, SENTINEL)
        while True:
            self.steps += 1
            if self.steps > max_steps:
                raise Stop("too many steps")
            if self.step():
                return

    def step(self):
        pc0 = self.pc
        if self.trace is not None:
            self.trace.append(pc0)
        COVERED.add(pc0)
        op = self.fetch()
        top = op >> 12
        if top == 0:
            return self.group0(op, pc0)
        if top in (1, 2, 3):
            return self.move(op, top)
        if top == 4:
            return self.group4(op, pc0)
        if top == 5:
            return self.group5(op, pc0)
        if top == 6:
            return self.branch(op, pc0)
        if top == 7:
            reg = (op >> 9) & 7
            v = sext(op & 0xFF, 8)
            self.d[reg] = v & 0xFFFFFFFF
            self.logic_flags(v, 4)
            return False
        if top in (8, 0xC):
            return self.group_logic(op, pc0)
        if top in (9, 0xD):
            return self.group_addsub(op, pc0)
        if top == 0xB:
            return self.group_cmp(op, pc0)
        if top == 0xE:
            return self.shift(op, pc0)
        raise Stop("opcode %04x at %x" % (op, pc0))

    def move(self, op, top):
        size = {1: 1, 3: 2, 2: 4}[top]
        src = self.ea((op >> 3) & 7, op & 7, size)
        dmode, dreg = (op >> 6) & 7, (op >> 9) & 7
        v = self.read(src, size)
        if dmode == 1:  # movea
            self.a[dreg] = (sext(v, 16) if size == 2 else v) & 0xFFFFFFFF
            return False
        dst = self.ea(dmode, dreg, size)
        self.logic_flags(v, size)
        self.write(dst, size, v)
        return False

    def group0(self, op, pc0):
        if op & 0xFF00 == 0x0800:  # static bit operations
            kind = (op >> 6) & 3
            bit = self.fetch() & 7
            dst = self.ea((op >> 3) & 7, op & 7, 1)
            if dst[0] == "d":
                raise Stop("bit op on register at %x" % pc0)
            v = self.read(dst, 1)
            self.z = 0 if (v >> bit) & 1 else 1
            if kind == 3:  # bset
                self.write(dst, 1, v | (1 << bit))
            elif kind == 2:  # bclr
                self.write(dst, 1, v & ~(1 << bit))
            elif kind == 1:  # bchg
                self.write(dst, 1, v ^ (1 << bit))
            return False
        kind = (op >> 9) & 7
        sz = (op >> 6) & 3
        if sz == 3 or kind not in (0, 1, 2, 3, 5, 6):
            raise Stop("opcode %04x at %x" % (op, pc0))
        size = [1, 2, 4][sz]
        imm = self.ea(7, 4, size)[1]
        dst = self.ea((op >> 3) & 7, op & 7, size)
        d = self.read(dst, size)
        if kind == 1:  # andi
            r = d & imm
            self.logic_flags(r, size)
            self.write(dst, size, r)
        elif kind == 0:  # ori
            r = d | imm
            self.logic_flags(r, size)
            self.write(dst, size, r)
        elif kind == 5:  # eori
            r = d ^ imm
            self.logic_flags(r, size)
            self.write(dst, size, r)
        elif kind == 6:  # cmpi
            self.sub_flags(imm, d, size)
        elif kind == 2:  # subi
            r = self.sub_flags(imm, d, size)
            self.x = self.c
            self.write(dst, size, r)
        else:  # addi
            r = self.add_flags(imm, d, size, True)
            self.write(dst, size, r)
        return False

    def group4(self, op, pc0):
        if op == 0x4E75:  # rts
            ret = self.rd(self.a[7], 4)
            self.a[7] += 4
            if ret == SENTINEL:
                return True
            self.pc = ret
            return False
        if op & 0xFFC0 == 0x4EC0 or op & 0xFFC0 == 0x4E80:  # jmp / jsr
            t = self.ea((op >> 3) & 7, op & 7, 4)
            if op & 0xFFC0 == 0x4E80:
                self.a[7] -= 4
                self.wr(self.a[7], 4, self.pc)
            self.pc = t[1]
            return False
        if op & 0xF1C0 == 0x41C0:  # lea
            t = self.ea((op >> 3) & 7, op & 7, 4)
            self.a[(op >> 9) & 7] = t[1] & 0xFFFFFFFF
            return False
        if op & 0xFFC0 == 0x44C0:  # move to ccr
            v = self.read(self.ea((op >> 3) & 7, op & 7, 2), 2)
            self.c, self.v, self.z, self.n, self.x = v & 1, (v >> 1) & 1, (v >> 2) & 1, (v >> 3) & 1, (v >> 4) & 1
            return False
        sz = (op >> 6) & 3
        if sz == 3:
            raise Stop("opcode %04x at %x" % (op, pc0))
        size = [1, 2, 4][sz]
        kind = op & 0xFF00
        dst = self.ea((op >> 3) & 7, op & 7, size)
        if kind == 0x4200:  # clr
            self.write(dst, size, 0)
            self.n, self.z, self.v, self.c = 0, 1, 0, 0
        elif kind == 0x4400:  # neg
            r = self.sub_flags(self.read(dst, size), 0, size)
            self.x = self.c
            self.write(dst, size, r)
        elif kind == 0x4600:  # not
            r = ~self.read(dst, size)
            self.logic_flags(r, size)
            self.write(dst, size, r)
        elif kind == 0x4A00:  # tst
            self.logic_flags(self.read(dst, size), size)
        else:
            raise Stop("opcode %04x at %x" % (op, pc0))
        return False

    def group5(self, op, pc0):
        if (op >> 6) & 3 == 3:
            if (op >> 3) & 7 == 1 and op & 0xF000 == 0x5000:  # dbcc
                cc = (op >> 8) & 0xF
                disp = sext(self.fetch(), 16)
                reg = op & 7
                if not self.cond(cc):
                    v = ((self.d[reg] & 0xFFFF) - 1) & 0xFFFF
                    self.d[reg] = (self.d[reg] & 0xFFFF0000) | v
                    if v != 0xFFFF:
                        self.pc = pc0 + 2 + disp
                return False
            raise Stop("scc %04x at %x" % (op, pc0))
        size = [1, 2, 4][(op >> 6) & 3]
        data = (op >> 9) & 7 or 8
        dst = self.ea((op >> 3) & 7, op & 7, size)
        d = self.read(dst, size)
        if dst[0] == "a":
            d = self.a[dst[1]]
            self.a[dst[1]] = (d - data if op & 0x100 else d + data) & 0xFFFFFFFF
            return False
        if op & 0x100:
            r = self.sub_flags(data, d, size)
            self.x = self.c
        else:
            r = self.add_flags(data, d, size, True)
        self.write(dst, size, r)
        return False

    def branch(self, op, pc0):
        cc = (op >> 8) & 0xF
        disp = sext(op & 0xFF, 8)
        if op & 0xFF == 0:
            disp = sext(self.fetch(), 16)
        elif op & 0xFF == 0xFF:
            raise Stop("long branch at %x" % pc0)
        target = pc0 + 2 + disp
        if cc == 1:  # bsr
            self.a[7] -= 4
            self.wr(self.a[7], 4, self.pc)
            self.pc = target
        elif self.cond(cc):
            self.pc = target
        return False

    def group_logic(self, op, pc0):
        reg = (op >> 9) & 7
        opm = (op >> 6) & 7
        is_and = (op >> 12) == 0xC
        if opm == 3 and is_and:  # mulu.w
            s = self.read(self.ea((op >> 3) & 7, op & 7, 2), 2)
            r = (self.d[reg] & 0xFFFF) * s
            self.d[reg] = r & 0xFFFFFFFF
            self.logic_flags(r, 4)
            return False
        if opm in (3, 7) or (is_and and opm in (4, 5, 6) and (op >> 3) & 7 in (0, 1)):
            raise Stop("opcode %04x at %x" % (op, pc0))
        size = [1, 2, 4][opm & 3]
        ea = self.ea((op >> 3) & 7, op & 7, size)
        if opm < 3:  # <ea> op Dn -> Dn
            s = self.read(ea, size)
            d = self.d[reg] & ((1 << (8 * size)) - 1)
            r = (d & s) if is_and else (d | s)
            self.logic_flags(r, size)
            self.write(("d", reg), size, r)
        else:  # Dn op <ea> -> <ea>
            s = self.d[reg] & ((1 << (8 * size)) - 1)
            d = self.read(ea, size)
            r = (d & s) if is_and else (d | s)
            self.logic_flags(r, size)
            self.write(ea, size, r)
        return False

    def group_addsub(self, op, pc0):
        reg = (op >> 9) & 7
        opm = (op >> 6) & 7
        is_add = (op >> 12) == 0xD
        if opm in (3, 7):  # adda / suba
            size = 2 if opm == 3 else 4
            s = self.read(self.ea((op >> 3) & 7, op & 7, size), size)
            if size == 2:
                s = sext(s, 16)
            s &= 0xFFFFFFFF
            self.a[reg] = ((self.a[reg] + s) if is_add else (self.a[reg] - s)) & 0xFFFFFFFF
            return False
        if opm in (4, 5, 6) and (op >> 3) & 7 in (0, 1):
            raise Stop("addx/subx at %x" % pc0)
        size = [1, 2, 4][opm & 3]
        ea = self.ea((op >> 3) & 7, op & 7, size)
        if opm < 3:
            s = self.read(ea, size)
            d = self.d[reg] & ((1 << (8 * size)) - 1)
            r = self.add_flags(s, d, size, True) if is_add else self.sub_flags(s, d, size)
            if not is_add:
                self.x = self.c
            self.write(("d", reg), size, r)
        else:
            s = self.d[reg] & ((1 << (8 * size)) - 1)
            d = self.read(ea, size)
            r = self.add_flags(s, d, size, True) if is_add else self.sub_flags(s, d, size)
            if not is_add:
                self.x = self.c
            self.write(ea, size, r)
        return False

    def group_cmp(self, op, pc0):
        reg = (op >> 9) & 7
        opm = (op >> 6) & 7
        if opm in (3, 7):  # cmpa
            size = 2 if opm == 3 else 4
            s = self.read(self.ea((op >> 3) & 7, op & 7, size), size)
            if size == 2:
                s = sext(s, 16)
            self.sub_flags(s & 0xFFFFFFFF, self.a[reg], 4)
            return False
        if opm < 3:
            size = [1, 2, 4][opm]
            s = self.read(self.ea((op >> 3) & 7, op & 7, size), size)
            self.sub_flags(s, self.d[reg], size)
            return False
        raise Stop("eor/cmpm %04x at %x" % (op, pc0))

    def shift(self, op, pc0):
        sz = (op >> 6) & 3
        if sz == 3:
            raise Stop("memory shift at %x" % pc0)
        size = [1, 2, 4][sz]
        bits = 8 * size
        mask = (1 << bits) - 1
        cnt = (op >> 9) & 7
        left = (op >> 8) & 1
        kind = (op >> 3) & 3
        reg = op & 7
        if (op >> 5) & 1:
            count = self.d[cnt] & 63
        else:
            count = cnt or 8
        v = self.d[reg] & mask
        c = None
        if kind == 1:  # lsl / lsr
            for _ in range(count):
                if left:
                    c = (v >> (bits - 1)) & 1
                    v = (v << 1) & mask
                else:
                    c = v & 1
                    v >>= 1
            if count:
                self.x = self.c = c
            else:
                self.c = 0
        elif kind == 3:  # rol / ror
            for _ in range(count):
                if left:
                    c = (v >> (bits - 1)) & 1
                    v = ((v << 1) | c) & mask
                else:
                    c = v & 1
                    v = (v >> 1) | (c << (bits - 1))
            self.c = c if count else 0
        else:
            raise Stop("asl/asr/rox at %x" % pc0)
        self.v = 0
        self.nz(v, size)
        self.write(("d", reg), size, v)
        return False


# ------------------------------------------------------------------------------------------------
# Test vectors
# ------------------------------------------------------------------------------------------------

def be16(rom, a):
    return (rom[a] << 8) | rom[a + 1]


def floor_lists(rom):
    """(stage, area) -> (address, number of segments) for the lists that exist (as the extractor)."""
    out = {}
    for stage in range(8):
        t = 0x12C30 + be16(rom, 0x12C30 + 2 * stage)
        for area in range(4):
            a = (t + be16(rom, t + 2 * area)) & 0xFFFFFF
            if a & 1 or not 0x13D00 <= a < 0x14000:
                continue
            if a < t + be16(rom, t):
                continue  # an area that does not exist: the offset points back into the offset table
            n = 0
            for k in range(64):
                n += 1
                if sext(be16(rom, a + 12 * k + 4), 16) == 0x7FFF:
                    break
            out[(stage, area)] = (a, n)
    return out


def entry_is_advance(v):
    return v["entry"] == 0x12C90


class Rig:
    """One run of one routine on one random setup."""

    OBJ = 0xFFB274  # the object under test (a6)
    ITEMS = 0xFF8874  # pool E: obstacles

    def __init__(self, rom, lists, rng):
        self.rom, self.lists, self.rng = rom, lists, rng

    def vram_fn(self, seed, density):
        # attribute word of a cell: class in bits 10..15, 0 with probability `density`
        def f(addr):
            h = random.Random((seed << 32) ^ addr).random
            r = h()
            cls = 0 if r < density else 1 + int(h() * 63)
            return (cls << 10) | int(h() * 0x400)

        return f

    def make(self, entry, tag, rng):
        rom = self.rom
        keys = sorted(self.lists)
        stage, area = rng.choice(keys)
        addr, n = self.lists[(stage, area)]
        vertical = 1 if rng.random() < 0.12 else 0
        seg = rng.randrange(0, n)
        base = rng.choice([0x10, 0x20, 0x40, rng.randrange(0, 0x100)])
        # object
        segs = [(sext(be16(rom, addr + 12 * k + 2), 16), sext(be16(rom, addr + 12 * k + 4), 16)) for k in range(n)]
        if rng.random() < 0.7:
            x0, x1 = rng.choice(segs)
            lo, hi = max(x0, 0), min(x1, 0x1000)
            x = rng.randrange(lo, hi + 1) if hi >= lo else rng.randrange(0, 0x400)
        else:
            x = rng.randrange(0, 0x400)
        o = {
            "x": (x << 16) | rng.choice([0, rng.randrange(0x10000)]),
            "y": (sext(rng.choice([0, rng.randrange(-8, 0x60), rng.randrange(-0x20, 0x200)]), 16) << 16) & 0xFFFFFFFF | rng.choice([0, rng.randrange(0x10000)]),
            "z": (rng.randrange(0, 0x400) << 16) | rng.choice([0, rng.randrange(0x10000)]),
            "vy": rng.choice([0, rng.randrange(0x10000)]),
            "air": rng.choice([0, 1]),
            "fl": rng.choice([0, rng.randrange(0, 0x60), rng.randrange(0, 0x10000)]),
            "fl2": rng.choice([0, rng.randrange(0, 0x20), rng.randrange(0, 0x10000)]),
            "wall": rng.randrange(0x100),
            "b52": rng.randrange(0x100),
            "wad": rng.randrange(0x100),
            "id": rng.randrange(4),
            "st": rng.choice([0, 0x02000000, 0x02020000, 0x02020200, rng.randrange(1 << 32)]),
            "fac": rng.choice([0, 1]),
            "obst94": 0xBEEF,
        }
        if tag == 0xC:
            o["st"] = rng.choice([0, 0x02000000, rng.randrange(1 << 32)])
        b4d5 = 1 if rng.random() < 0.1 else 0
        # obstacles
        obs = []
        if rng.random() < 0.6:
            for k in range(rng.randrange(1, 4)):
                while True:
                    tab = 0x100000 + 12 * rng.randrange(0, 2000)
                    idx = rng.randrange(0, 6)
                    rec = tab + 12 * idx
                    if rec + 12 <= 0x11D220:
                        zs, xw, ys = [sext(be16(rom, rec + 2 * j), 16) for j in (1, 3, 5)]
                        if 4 <= zs <= 200 and 4 <= xw <= 200 and ys >= 8:
                            break
                ox = (o["x"] >> 16) + rng.randrange(-60, 61)
                oz = (o["z"] >> 16) + rng.randrange(-60, 61)
                oy = rng.choice([0, rng.randrange(-10, 40)])
                obs.append(
                    {
                        "slot": rng.randrange(0, 24),
                        "x": ox & 0xFFFF,
                        "y": oy & 0xFFFF,
                        "z": oz & 0xFFFF,
                        "fac": rng.choice([0, 1, 0xFF]),
                        "tab": 0xC0000000 | tab if rng.random() < 0.7 else tab,
                        "idx": idx,
                        "fl": rng.choice([0, 0, rng.randrange(0, 0x40)]),
                    }
                )
            used = set()
            obs = [ob for ob in obs if not (ob["slot"] in used or used.add(ob["slot"]))]
        # the object itself can be in the list (it is skipped)
        me_in_list = tag != 0xC and obs and rng.random() < 0.3
        return {
            "entry": entry, "stage": stage, "area": area, "vert": vertical, "seg": seg, "base": base,
            "tag": tag, "b4d5": b4d5, "o": o, "obs": obs, "list_seed": rng.randrange(1 << 30),
            "density": rng.choice([0.2, 0.5, 0.8]), "me_in_list": bool(me_in_list),
            "addr": addr, "cam": rng.choice([rng.randrange(0, 0x200), rng.randrange(0, 0x1000), rng.randrange(0, 0x8000)]),
        }

    def run(self, v):
        rom = self.rom
        seed = v["list_seed"]
        cpu = CPU(rom, None)
        # video RAM: scroll2 at 0x90C000 (attribute word at +2 of every 4-byte cell) and the vertical
        # scene map at 0x91C000 (same cell layout)
        fn = self.vram_fn(seed, v["density"])

        def vram_value(addr):
            return fn(addr) if (addr & 3) == 2 else 0

        cpu.vram_value = vram_value
        ram = cpu.ram
        a5 = A5

        def wb(a, val):
            ram[a & 0xFFFFFF] = val & 0xFF

        def ww(a, val):
            wb(a, val >> 8)
            wb(a + 1, val)

        def wl(a, val):
            ww(a, val >> 16)
            ww(a + 2, val)

        wb(a5 + 0x4D9, v["stage"])
        wb(a5 + 0x786, v["area"])
        wb(a5 + 0x4DC, v["vert"])
        wb(a5 + 0x4D5, v["b4d5"])
        n_seg = self.lists[(v["stage"], v["area"])][1]
        if entry_is_advance(v) and v["seg"] == n_seg - 1 and be16(rom, v["addr"] + 12 * n_seg + 2) != 0x7FFF:
            v["seg"] = max(0, n_seg - 2)  # the original would read the next list's first segment
        ww(a5 + 0x744, v["cam"])
        wl(a5 + 0x6AC, v["addr"] + 12 * v["seg"])
        ww(a5 + 0x6B0, v["base"])
        o = v["o"]
        a6 = self.OBJ
        wb(a6 + 3, o["id"])
        wl(a6 + 4, o["st"])
        wl(a6 + 8, o["x"])
        wl(a6 + 0xC, o["y"])
        wl(a6 + 0x10, o["z"])
        ww(a6 + 0x16, o["vy"])
        wb(a6 + 0x24, o["fac"])
        wb(a6 + 0x50, o["wall"])
        wb(a6 + 0x51, o["air"])
        wb(a6 + 0x52, o["b52"])
        wb(a6 + 0x53, o["wad"])
        ww(a6 + 0x54, o["fl"])
        ww(a6 + 0x56, o["fl2"])
        ww(a6 + 0x94, o["obst94"])
        wb(a6 + 0x82, v["tag"])
        # obstacles: handles in the list (newest first), the previous frame's list is $6A26 / $6A28
        handles = []
        for ob in v["obs"]:
            h = 0x8874 + 0xC0 * ob["slot"]
            base = 0xFF0000 + h
            ww(base + 8, ob["x"])
            ww(base + 0xA, 0)
            ww(base + 0xC, ob["y"])
            ww(base + 0xE, 0)
            ww(base + 0x10, ob["z"])
            ww(base + 0x12, 0)
            wb(base + 0x24, ob["fac"])
            wl(base + 0x44, ob["tab"])
            wb(base + 0x48, ob["idx"])
            ww(base + 0x54, ob["fl"])
            wb(base + 0xAC, 0x55)
            handles.append(h)
        if v["me_in_list"]:
            handles.insert(0, a6 & 0xFFFF)
        ww(a5 + 0x6A26, len(handles))
        ptr = 0x6A5A + 0x8000 - 2 * len(handles)  # low word of the RAM address
        ww(a5 + 0x6A28, ptr & 0xFFFF)
        for j, h in enumerate(handles):
            ww(0xFF0000 + (ptr & 0xFFFF) + 2 * j, h)
        cpu.a[5] = a5
        cpu.a[6] = a6
        if os.environ.get("TRACE_VECTOR") == str(v.get("index")):
            cpu.trace = []
        cpu.run(v["entry"])
        if cpu.trace is not None:
            print("trace of vector %s:" % v.get("index"), " ".join("%x" % p for p in cpu.trace))
        res = {
            "x": cpu.rd(a6 + 8, 4), "y": cpu.rd(a6 + 0xC, 4), "z": cpu.rd(a6 + 0x10, 4),
            "vy": cpu.rd(a6 + 0x16, 2), "wall": cpu.rd(a6 + 0x50, 1), "air": cpu.rd(a6 + 0x51, 1),
            "b52": cpu.rd(a6 + 0x52, 1), "wad": cpu.rd(a6 + 0x53, 1), "fl": cpu.rd(a6 + 0x54, 2),
            "fl2": cpu.rd(a6 + 0x56, 2), "obst94": cpu.rd(a6 + 0x94, 2), "carry": cpu.c,
            "flag": cpu.rd(a5 + 0x6B2, 1), "ptr": cpu.rd(a5 + 0x6AC, 4),
            "ac": [cpu.rd(0xFF0000 + 0x8874 + 0xC0 * ob["slot"] + 0xAC, 1) for ob in v["obs"]],
        }
        return res, dict(cpu.vram_read)


ENTRIES = {
    "collide": 0x12CB4, "player": 0x12CAC, "floor_only": 0x12D4E, "free": 0x12E20, "effect": 0x12DAC,
    "lookup": 0x12FE4, "wall": 0x1309C, "ceiling": 0x1361E, "advance": 0x12C90,
}


def fmt(v, res, cells):
    o = v["o"]
    parts = [f"entry={v['entry_name']}"]
    parts += [f"stage={v['stage']}", f"area={v['area']}", f"vert={v['vert']}", f"seg={v['seg']}", f"base={v['base']}",
              f"tag={v['tag']}", f"b4d5={v['b4d5']}", f"cam={v['cam']:x}", f"addr={v['addr']:x}"]
    parts += [f"{k}={o[k]:x}" for k in ("x", "y", "z", "vy", "air", "fl", "fl2", "wall", "b52", "wad", "id", "st", "fac")]
    obs = ";".join(
        f"{ob['slot']:x},{ob['x']:x},{ob['y']:x},{ob['z']:x},{ob['fac']:x},{ob['tab']:x},{ob['idx']:x},{ob['fl']:x}" for ob in v["obs"]
    )
    parts.append("obs=" + (obs or "-"))
    parts.append(f"melist={int(v['me_in_list'])}")
    # cells that were read: scroll2 (0x90C000..) and scene (0x91C000..), as "cell:attr"
    s2 = ",".join(f"{(a - 0x90C000) // 4:x}:{w:x}" for a, w in sorted(cells.items()) if 0x90C000 <= a < 0x910000 and a & 3 == 2)
    sc = ",".join(f"{(a - 0x91C000) // 4:x}:{w:x}" for a, w in sorted(cells.items()) if 0x91C000 <= a < 0x91C800 and a & 3 == 2)
    parts.append("s2=" + (s2 or "-"))
    parts.append("scene=" + (sc or "-"))
    parts.append("|")
    parts += [f"{k}={res[k]:x}" for k in ("x", "y", "z", "vy", "wall", "air", "b52", "wad", "fl", "fl2", "obst94", "carry", "flag", "ptr")]
    parts.append("ac=" + (",".join(f"{a:x}" for a in res["ac"]) or "-"))
    return " ".join(parts)


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "re", "out", "terrain_vectors.txt")
    seed = int(sys.argv[2]) if len(sys.argv) > 2 else 1
    count = int(sys.argv[3]) if len(sys.argv) > 3 else 6000
    rom = open(IMAGE, "rb").read()
    lists = floor_lists(rom)
    rng = random.Random(seed)
    rig = Rig(rom, lists, rng)
    lines = []
    names = list(ENTRIES)
    weights = {"collide": 5, "player": 4, "floor_only": 1, "free": 2, "effect": 1, "lookup": 1, "wall": 4, "ceiling": 2, "advance": 1}
    pool = [n for n in names for _ in range(weights[n])]
    for k in range(count):
        name = rng.choice(pool)
        tag = 0xC if (name == "collide" and rng.random() < 0.3) else rng.choice([0, 0x4, 0x8])
        v = rig.make(ENTRIES[name], tag, rng)
        v["entry_name"] = name
        v["index"] = k
        res, cells = rig.run(v)
        lines.append(fmt(v, res, cells))
    with open(out, "w") as f:
        f.write("\n".join(lines) + "\n")
    print(f"{len(lines)} vectors -> {out}")
    if os.environ.get("COVERAGE"):
        # instructions of the routines that no vector executed (jump tables decode as garbage here)
        import capstone

        md = capstone.Cs(capstone.CS_ARCH_M68K, capstone.CS_MODE_BIG_ENDIAN | capstone.CS_MODE_M68K_000)
        missing = []
        for lo, hi in [(0x12C90, 0x12CAC), (0x12CAC, 0x13138), (0x13138, 0x1313E), (0x1315E, 0x13168), (0x13178, 0x131E4),
                       (0x131E4, 0x131EE), (0x131FE, 0x132DA), (0x132EA, 0x13458), (0x13468, 0x13518), (0x13528, 0x135B2), (0x135C2, 0x13706)]:
            addr = lo
            while addr < hi:
                ins = next(md.disasm(rom[addr:addr + 16], addr), None)
                if ins is None:
                    addr += 2
                    continue
                if addr not in COVERED:
                    missing.append("%x %s %s" % (addr, ins.mnemonic, ins.op_str))
                addr += ins.size
        print(len(missing), "instructions never executed:")
        print("\n".join(missing))


if __name__ == "__main__":
    main()
