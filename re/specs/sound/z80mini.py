"""A small Z80 interpreter that runs routines of the QSound driver on the real (Kabuki decrypted) bytes.

It exists to CHECK the spec and the port, not to be a full emulator: tables.py runs the multiply routines on it and
pan.py runs the pan motion handlers on it and compares them with the formulas of the spec. Flags kept: S, Z, C (the
driver only branches on those); H, N, P/V are not modelled. Not supported (raises): halt, in/out, daa, rst, im.

Kabuki rule (SPEC 1): opcode bytes (M1 fetches, including the second byte of CB/DD/ED/FD) come from the opcode
image, operands, displacements, the last byte of `DD CB d op` and all data reads come from the data image.

    cpu = Z80(op_image, data_image)
    cpu.traps[0x0A13] = lambda c: log.append((c.a, c.de))   # when pc reaches 0x0A13 (call, jp or falling through)
                                                            # run this, then act as if the routine executed `ret`
    cpu.run(0x1007, ix=0xF200)                              # runs until the routine executes `ret` at call depth 0
"""

MAIN = ("b", "c", "d", "e", "h", "l", None, "a")


class Z80:
    def __init__(self, op_img, data_img):
        self.op, self.dat = op_img, data_img
        self.ram = {}                       # writes (any address >= 0x8000) and RAM reads
        self.a = self.b = self.c = self.d = self.e = self.h = self.l = 0
        self.ix = self.iy = 0
        self.sp = 0xFFFF
        self.z = self.cy = self.s = 0
        self.pc = 0
        self.traps = {}
        self.steps = 0
        self.depth = 0

    # ---- memory
    def rd(self, a):
        a &= 0xFFFF
        if a in self.ram:
            return self.ram[a]
        return self.dat[a] if a < 0x8000 else 0

    def wr(self, a, v):
        self.ram[a & 0xFFFF] = v & 0xFF

    def rd16(self, a):
        return self.rd(a) | (self.rd(a + 1) << 8)

    def wr16(self, a, v):
        self.wr(a, v)
        self.wr(a + 1, v >> 8)

    # ---- registers
    @property
    def hl(self):
        return (self.h << 8) | self.l

    @hl.setter
    def hl(self, v):
        self.h, self.l = (v >> 8) & 0xFF, v & 0xFF

    @property
    def bc(self):
        return (self.b << 8) | self.c

    @bc.setter
    def bc(self, v):
        self.b, self.c = (v >> 8) & 0xFF, v & 0xFF

    @property
    def de(self):
        return (self.d << 8) | self.e

    @de.setter
    def de(self, v):
        self.d, self.e = (v >> 8) & 0xFF, v & 0xFF

    def rp(self, n, ix=None):               # 0 bc 1 de 2 hl(or ix/iy) 3 sp
        return (self.bc, self.de, self.hl if ix is None else getattr(self, ix), self.sp)[n]

    def set_rp(self, n, v, ix=None):
        v &= 0xFFFF
        if n == 0:
            self.bc = v
        elif n == 1:
            self.de = v
        elif n == 2:
            if ix is None:
                self.hl = v
            else:
                setattr(self, ix, v)
        else:
            self.sp = v

    def getr(self, i, ea=None):
        n = MAIN[i]
        return self.rd(ea) if n is None else getattr(self, n)

    def setr(self, i, v, ea=None):
        n = MAIN[i]
        if n is None:
            self.wr(ea, v)
        else:
            setattr(self, n, v & 0xFF)

    def push(self, v):
        self.sp = (self.sp - 2) & 0xFFFF
        self.wr16(self.sp, v)

    def pop(self):
        v = self.rd16(self.sp)
        self.sp = (self.sp + 2) & 0xFFFF
        return v

    def cond(self, n):                      # nz z nc c po pe p m
        return [not self.z, self.z, not self.cy, self.cy, True, False, not self.s, self.s][n]

    def alu(self, op, v):
        a = self.a
        if op in (0, 1):                    # add / adc
            r = a + v + (self.cy if op == 1 else 0)
            self.cy = r >> 8
        elif op in (2, 3, 7):               # sub / sbc / cp
            r = a - v - (self.cy if op == 3 else 0)
            self.cy = 1 if r < 0 else 0
        elif op == 4:
            r = a & v
            self.cy = 0
        elif op == 5:
            r = a ^ v
            self.cy = 0
        else:
            r = a | v
            self.cy = 0
        r &= 0xFF
        self.z, self.s = int(r == 0), r >> 7
        if op != 7:
            self.a = r

    # ---- run
    def run(self, pc, **regs):
        for k, v in regs.items():
            setattr(self, k, v)
        self.pc, self.steps, self.depth = pc, 0, 0
        while True:
            self.steps += 1
            if self.steps > 2_000_000:
                raise RuntimeError("runaway")
            if self.step():
                return

    def fetch_op(self):
        v = self.op[self.pc & 0xFFFF] if (self.pc & 0xFFFF) < 0x8000 else self.dat[self.pc & 0xFFFF]
        self.pc += 1
        return v

    def imm8(self):
        v = self.dat[self.pc & 0xFFFF]
        self.pc += 1
        return v

    def imm16(self):
        v = self.imm8() | (self.imm8() << 8)
        return v

    def step(self):
        """Executes one instruction. Returns True when the routine returned at depth 0."""
        ix = None
        if self.pc in self.traps:
            self.traps[self.pc](self)
            return self.do_ret()
        pc0 = self.pc
        op = self.fetch_op()
        if op in (0xDD, 0xFD):
            ix = "ix" if op == 0xDD else "iy"
            op = self.fetch_op()
        if op == 0xCB:
            return self.step_cb(ix)
        if op == 0xED:
            return self.step_ed()
        x, y, z = op >> 6, (op >> 3) & 7, op & 7
        p, q = y >> 1, y & 1

        def ea():                               # effective address of (hl) / (ix+d)
            if ix is None:
                return self.hl
            d = self.imm8()
            return (getattr(self, ix) + (d - 256 if d & 0x80 else d)) & 0xFFFF

        if x == 0:
            if z == 0:
                if y == 0:
                    pass
                elif y == 1:
                    raise ValueError("ex af,af'")
                elif y == 2:
                    d = self.imm8()
                    self.b = (self.b - 1) & 0xFF
                    if self.b:
                        self.pc += d - 256 if d & 0x80 else d
                else:
                    d = self.imm8()
                    if y == 3 or self.cond(y - 4):
                        self.pc += d - 256 if d & 0x80 else d
            elif z == 1:
                if q == 0:
                    self.set_rp(p, self.imm16(), ix)
                else:
                    r = self.rp(2, ix) + self.rp(p, ix)
                    self.cy = r >> 16
                    self.set_rp(2, r, ix)
            elif z == 2:
                if y == 0:
                    self.wr(self.bc, self.a)
                elif y == 1:
                    self.a = self.rd(self.bc)
                elif y == 2:
                    self.wr(self.de, self.a)
                elif y == 3:
                    self.a = self.rd(self.de)
                elif y == 4:
                    n = self.imm16()
                    v = self.rp(2, ix)
                    self.wr16(n, v)
                elif y == 5:
                    n = self.imm16()
                    self.set_rp(2, self.rd16(n), ix)
                elif y == 6:
                    self.wr(self.imm16(), self.a)
                else:
                    self.a = self.rd(self.imm16())
            elif z == 3:
                self.set_rp(p, self.rp(p, ix) + (1 if q == 0 else -1), ix)
            elif z in (4, 5):
                e = ea() if y == 6 else None
                v = (self.getr(y, e) + (1 if z == 4 else -1)) & 0xFF
                self.z, self.s = int(v == 0), v >> 7
                self.setr(y, v, e)
            elif z == 6:
                e = ea() if y == 6 else None
                self.setr(y, self.imm8(), e)
            else:
                if y == 0:                      # rlca
                    self.cy = self.a >> 7
                    self.a = ((self.a << 1) | self.cy) & 0xFF
                elif y == 1:                    # rrca
                    self.cy = self.a & 1
                    self.a = (self.a >> 1) | (self.cy << 7)
                else:
                    raise ValueError(f"opcode {op:02x} at {pc0:04x}")
        elif x == 1:
            if op == 0x76:
                raise ValueError("halt")
            if y == 6:                          # ld (hl),r / ld (ix+d),r  (r is a plain register)
                self.wr(ea(), self.getr(z))
            elif z == 6:                        # ld r,(hl) / ld r,(ix+d)
                self.setr(y, self.rd(ea()))
            else:
                self.setr(y, self.getr(z))
        elif x == 2:
            e = ea() if z == 6 else None
            self.alu(y, self.getr(z, e))
        else:
            if z == 0:
                if self.cond(y):
                    return self.do_ret()
            elif z == 1:
                if q == 0:
                    v = self.pop()
                    if p == 3:
                        self.a, f = v >> 8, v & 0xFF
                        self.cy, self.z, self.s = f & 1, (f >> 6) & 1, (f >> 7) & 1
                    else:
                        self.set_rp(p, v, ix)
                elif p == 0:
                    return self.do_ret()
                elif p == 2:
                    self.pc = self.rp(2, ix)
                else:
                    raise ValueError(f"opcode {op:02x} at {pc0:04x}")
            elif z == 2:
                n = self.imm16()
                if self.cond(y):
                    self.pc = n
            elif z == 3:
                if y == 0:
                    self.pc = self.imm16()
                elif y == 5:
                    self.de, self.hl = self.hl, self.de
                elif y in (6, 7):
                    pass                        # di / ei
                else:
                    raise ValueError(f"opcode {op:02x} at {pc0:04x}")
            elif z == 4:
                n = self.imm16()
                if self.cond(y):
                    self.do_call(n)
            elif z == 5:
                if q == 0:
                    if p == 3:
                        f = self.cy | (self.z << 6) | (self.s << 7)
                        self.push((self.a << 8) | f)
                    else:
                        self.push(self.rp(p, ix))
                elif p == 0:
                    self.do_call(self.imm16())
                else:
                    raise ValueError(f"opcode {op:02x} at {pc0:04x}")
            elif z == 6:
                self.alu(y, self.imm8())
            else:
                raise ValueError(f"rst at {pc0:04x}")
        return False

    def do_call(self, n):
        self.push(self.pc)
        self.depth += 1
        self.pc = n

    def do_ret(self):
        if self.depth == 0:
            return True
        self.depth -= 1
        self.pc = self.pop()
        return False

    def step_cb(self, ix):
        if ix is None:
            op = self.fetch_op()
            e = self.hl
        else:
            d = self.imm8()
            e = (getattr(self, ix) + (d - 256 if d & 0x80 else d)) & 0xFFFF
            op = self.dat[self.pc]               # the last byte of DD CB d op is an operand read (data image)
            self.pc += 1
        x, y, z = op >> 6, (op >> 3) & 7, op & 7
        v = self.rd(e) if (ix is not None or z == 6) else self.getr(z)
        if x == 1:                                # bit
            self.z = int(not (v >> y) & 1)
            self.s = int(y == 7 and bool(v & 0x80))
            return False
        if x == 2:
            r = v & ~(1 << y) & 0xFF
        elif x == 3:
            r = v | (1 << y)
        else:                                     # rotate / shift
            if y == 2:                            # rl
                t = (v << 1) | self.cy
                self.cy, r = t >> 8, t & 0xFF
            elif y == 4:                          # sla
                self.cy, r = v >> 7, (v << 1) & 0xFF
            elif y == 7:                          # srl
                self.cy, r = v & 1, v >> 1
            elif y == 3:                          # rr
                t = v | (self.cy << 8)
                self.cy, r = t & 1, t >> 1
            elif y == 0:                          # rlc
                self.cy = v >> 7
                r = ((v << 1) | self.cy) & 0xFF
            elif y == 1:                          # rrc
                self.cy = v & 1
                r = (v >> 1) | (self.cy << 7)
            else:
                raise ValueError(f"CB {op:02x}")
            self.z, self.s = int(r == 0), r >> 7
        if ix is not None or z == 6:
            self.wr(e, r)
            if ix is not None and z != 6:
                self.setr(z, r)
        else:
            self.setr(z, r)
        return False

    def step_ed(self):
        op = self.fetch_op()
        x, y, z = op >> 6, (op >> 3) & 7, op & 7
        p, q = y >> 1, y & 1
        if x == 1 and z == 2:                     # sbc / adc hl,rp
            v = self.rp(p)
            if q == 0:
                r = self.hl - v - self.cy
                self.cy = 1 if r < 0 else 0
            else:
                r = self.hl + v + self.cy
                self.cy = r >> 16
            r &= 0xFFFF
            self.z, self.s = int(r == 0), r >> 15
            self.hl = r
        elif x == 1 and z == 3:
            n = self.imm16()
            if q == 0:
                self.wr16(n, self.rp(p))
            else:
                self.set_rp(p, self.rd16(n))
        elif op == 0xB0:                          # ldir
            while True:
                self.wr(self.de, self.rd(self.hl))
                self.hl, self.de, self.bc = self.hl + 1, self.de + 1, self.bc - 1
                if self.bc == 0:
                    break
        else:
            raise ValueError(f"ED {op:02x}")
        return False
