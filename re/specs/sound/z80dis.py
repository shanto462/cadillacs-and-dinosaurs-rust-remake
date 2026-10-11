"""Small Z80 disassembler for the Kabuki-encrypted dino sound program.

Opcode bytes (M1 fetches, including the second byte of CB/DD/ED/FD prefixes) come from the
opcode-decrypted image; operands, displacements and the last byte of DD CB d op come from the
data-decrypted image (they are ordinary memory reads).

usage: z80dis.py [entry_hex ...]          recursive listing from 0x0000/0x0038/0x0066 + entries
       z80dis.py --range start end        linear listing of a range
"""
import os
import sys

RE = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OP = open(RE + "/out/z80_op.bin", "rb").read()
DATA = open(RE + "/out/z80_data.bin", "rb").read()

R = ["b", "c", "d", "e", "h", "l", "(hl)", "a"]
RP = ["bc", "de", "hl", "sp"]
RP2 = ["bc", "de", "hl", "af"]
CC = ["nz", "z", "nc", "c", "po", "pe", "p", "m"]
ALU = ["add a,", "adc a,", "sub ", "sbc a,", "and ", "xor ", "or ", "cp "]
ROT = ["rlc", "rrc", "rl", "rr", "sla", "sra", "sll", "srl"]


def op(a):
    return OP[a] if a < 0x8000 else DATA[a]


def d8(a):
    return DATA[a]


def d16(a):
    return DATA[a] | (DATA[a + 1] << 8)


def s8(v):
    return v - 256 if v >= 128 else v


def decode(a):
    """Return (length, text, flow, target). flow: None, 'jp', 'jpc', 'call', 'callc', 'ret', 'retc', 'rst', 'stop'."""
    b = op(a)
    if b == 0xCB:
        b2 = op(a + 1)
        x, y, z = b2 >> 6, (b2 >> 3) & 7, b2 & 7
        if x == 0:
            return 2, f"{ROT[y]} {R[z]}", None, None
        return 2, f"{['', 'bit', 'res', 'set'][x]} {y},{R[z]}", None, None
    if b == 0xED:
        b2 = op(a + 1)
        x, y, z = b2 >> 6, (b2 >> 3) & 7, b2 & 7
        p, q = y >> 1, y & 1
        if x == 1:
            if z == 0:
                return 2, f"in {R[y] if y != 6 else 'f'},(c)", None, None
            if z == 1:
                return 2, f"out (c),{R[y] if y != 6 else '0'}", None, None
            if z == 2:
                return 2, f"{'sbc' if q == 0 else 'adc'} hl,{RP[p]}", None, None
            if z == 3:
                nn = d16(a + 2)
                if q == 0:
                    return 4, f"ld (${nn:04x}),{RP[p]}", None, None
                return 4, f"ld {RP[p]},(${nn:04x})", None, None
            if z == 4:
                return 2, "neg", None, None
            if z == 5:
                return 2, ("retn" if y != 1 else "reti"), "ret", None
            if z == 6:
                return 2, f"im {[0, 0, 1, 2, 0, 0, 1, 2][y]}", None, None
            return 2, ["ld i,a", "ld r,a", "ld a,i", "ld a,r", "rrd", "rld", "nop", "nop"][y], None, None
        if x == 2 and z <= 3 and y >= 4:
            names = [["ldi", "cpi", "ini", "outi"], ["ldd", "cpd", "ind", "outd"],
                     ["ldir", "cpir", "inir", "otir"], ["lddr", "cpdr", "indr", "otdr"]]
            return 2, names[y - 4][z], None, None
        return 2, f"db $ed,${b2:02x}", None, None
    if b in (0xDD, 0xFD):
        ix = "ix" if b == 0xDD else "iy"
        b2 = op(a + 1)
        if b2 == 0xCB:
            d = s8(d8(a + 2))
            b4 = d8(a + 3)
            x, y, z = b4 >> 6, (b4 >> 3) & 7, b4 & 7
            m = f"({ix}{d:+d})"
            if x == 0:
                t = f"{ROT[y]} {m}"
            else:
                t = f"{['', 'bit', 'res', 'set'][x]} {y},{m}"
            return 4, t, None, None
        # Substitute hl -> ix in the base instruction
        if b2 == 0xE9:
            return 2, f"jp ({ix})", "stop", None
        if b2 == 0x21:
            return 4, f"ld {ix},${d16(a + 2):04x}", None, None
        if b2 == 0x22:
            return 4, f"ld (${d16(a + 2):04x}),{ix}", None, None
        if b2 == 0x2A:
            return 4, f"ld {ix},(${d16(a + 2):04x})", None, None
        if b2 == 0x36:
            d = s8(d8(a + 2))
            return 4, f"ld ({ix}{d:+d}),${d8(a + 3):02x}", None, None
        if b2 in (0x23, 0x2B, 0x09, 0x19, 0x29, 0x39, 0xE1, 0xE5, 0xE3, 0xF9):
            t = {0x23: f"inc {ix}", 0x2B: f"dec {ix}", 0x09: f"add {ix},bc", 0x19: f"add {ix},de",
                 0x29: f"add {ix},{ix}", 0x39: f"add {ix},sp", 0xE1: f"pop {ix}", 0xE5: f"push {ix}",
                 0xE3: f"ex (sp),{ix}", 0xF9: f"ld sp,{ix}"}[b2]
            return 2, t, None, None
        x, y, z = b2 >> 6, (b2 >> 3) & 7, b2 & 7
        uses_m = (x == 1 and (y == 6 or z == 6) and not (y == 6 and z == 6)) or (x == 2 and z == 6) or \
                 (x == 0 and z in (4, 5) and y == 6) or (x == 3 and False)
        if uses_m:
            d = s8(d8(a + 2))
            m = f"({ix}{d:+d})"
            if x == 1:
                src = m if z == 6 else R[z]
                dst = m if y == 6 else R[y]
                return 3, f"ld {dst},{src}", None, None
            if x == 2:
                return 3, f"{ALU[y]}{m}", None, None
            return 3, f"{'inc' if z == 4 else 'dec'} {m}", None, None
        # undocumented ixh/ixl forms: decode as base with a note
        n, t, f, tg = decode_base(a + 1)
        return n + 1, t + f"  ;{ix}-prefixed (undocumented)", f, tg
    return decode_base(a)


def decode_base(a):
    b = op(a)
    x, y, z = b >> 6, (b >> 3) & 7, b & 7
    p, q = y >> 1, y & 1
    if x == 0:
        if z == 0:
            if y == 0:
                return 1, "nop", None, None
            if y == 1:
                return 1, "ex af,af'", None, None
            t = a + 2 + s8(d8(a + 1))
            if y == 2:
                return 2, f"djnz ${t:04x}", "jpc", t
            if y == 3:
                return 2, f"jr ${t:04x}", "jp", t
            return 2, f"jr {CC[y - 4]},${t:04x}", "jpc", t
        if z == 1:
            if q == 0:
                return 3, f"ld {RP[p]},${d16(a + 1):04x}", None, None
            return 1, f"add hl,{RP[p]}", None, None
        if z == 2:
            if q == 0:
                if p == 0:
                    return 1, "ld (bc),a", None, None
                if p == 1:
                    return 1, "ld (de),a", None, None
                if p == 2:
                    return 3, f"ld (${d16(a + 1):04x}),hl", None, None
                return 3, f"ld (${d16(a + 1):04x}),a", None, None
            if p == 0:
                return 1, "ld a,(bc)", None, None
            if p == 1:
                return 1, "ld a,(de)", None, None
            if p == 2:
                return 3, f"ld hl,(${d16(a + 1):04x})", None, None
            return 3, f"ld a,(${d16(a + 1):04x})", None, None
        if z == 3:
            return 1, f"{'inc' if q == 0 else 'dec'} {RP[p]}", None, None
        if z == 4:
            return 1, f"inc {R[y]}", None, None
        if z == 5:
            return 1, f"dec {R[y]}", None, None
        if z == 6:
            return 2, f"ld {R[y]},${d8(a + 1):02x}", None, None
        return 1, ["rlca", "rrca", "rla", "rra", "daa", "cpl", "scf", "ccf"][y], None, None
    if x == 1:
        if y == 6 and z == 6:
            return 1, "halt", None, None
        return 1, f"ld {R[y]},{R[z]}", None, None
    if x == 2:
        return 1, f"{ALU[y]}{R[z]}", None, None
    # x == 3
    if z == 0:
        return 1, f"ret {CC[y]}", "retc", None
    if z == 1:
        if q == 0:
            return 1, f"pop {RP2[p]}", None, None
        return 1, ["ret", "exx", "jp (hl)", "ld sp,hl"][p], ["ret", None, "stop", None][p], None
    if z == 2:
        t = d16(a + 1)
        return 3, f"jp {CC[y]},${t:04x}", "jpc", t
    if z == 3:
        if y == 0:
            t = d16(a + 1)
            return 3, f"jp ${t:04x}", "jp", t
        if y == 2:
            return 2, f"out (${d8(a + 1):02x}),a", None, None
        if y == 3:
            return 2, f"in a,(${d8(a + 1):02x})", None, None
        return 1, ["", "", "", "", "ex (sp),hl", "ex de,hl", "di", "ei"][y], None, None
    if z == 4:
        t = d16(a + 1)
        return 3, f"call {CC[y]},${t:04x}", "callc", t
    if z == 5:
        if q == 0:
            return 1, f"push {RP2[p]}", None, None
        t = d16(a + 1)
        return 3, f"call ${t:04x}", "call", t
    if z == 6:
        return 2, f"{ALU[y]}${d8(a + 1):02x}", None, None
    return 1, f"rst ${y * 8:02x}", "call", y * 8


def raw_bytes(a, n):
    return " ".join(f"{op(a) if i == 0 else d8(a + i):02x}" for i in range(n))


def walk(entries, stops=()):
    seen = {}
    labels = set(entries)
    todo = list(entries)
    while todo:
        a = todo.pop()
        while a not in seen and a < 0x8000 and a not in stops:
            n, t, f, tg = decode(a)
            seen[a] = (n, t)
            if tg is not None and tg < 0x8000:
                labels.add(tg)
                if tg not in seen:
                    todo.append(tg)
            if f in ("jp", "ret", "stop"):
                break
            a += n
    return seen, labels


def listing(seen, labels, out=sys.stdout):
    prev_end = None
    for a in sorted(seen):
        n, t = seen[a]
        if prev_end is not None and a != prev_end:
            print("", file=out)
        if a in labels:
            print(f"L{a:04x}:", file=out)
        print(f"  {a:04x}  {raw_bytes(a, n):<12} {t}", file=out)
        prev_end = a + n


if __name__ == "__main__":
    args = sys.argv[1:]
    if args and args[0] == "--range":
        a, end = int(args[1], 16), int(args[2], 16)
        while a < end:
            n, t, f, tg = decode(a)
            print(f"  {a:04x}  {raw_bytes(a, n):<12} {t}")
            a += n
    else:
        entries = [0x0000, 0x0038, 0x0066] + [int(x, 16) for x in args]
        seen, labels = walk(entries)
        listing(seen, labels)
