# ELF32 big-endian EM_68K with ROM (RX) plus uninitialised RAM / IO / VRAM / QSound segments (RW, no file bytes)
import struct, sys
data = open(sys.argv[1], "rb").read(); out = sys.argv[2]
segs = [(0x000000, data, 5), (0x800000, 0x200, 6), (0x900000, 0x30000, 6), (0xF18000, 0x2000, 6), (0xF1C000, 0x10, 6), (0xFF0000, 0x10000, 6)]
ph_off, n = 52, len(segs); off = 0x1000
eh = b"\x7fELF" + bytes([1, 2, 1, 0]) + b"\0" * 8 + struct.pack(">HHIIIIIHHHHHH", 2, 4, 1, 0x41a, ph_off, 0, 0, 52, 32, n, 0, 0, 0)
ph = b""
for va, d, fl in segs:
    if isinstance(d, bytes): ph += struct.pack(">IIIIIIII", 1, off, va, va, len(d), len(d), fl, 0x1000)
    else: ph += struct.pack(">IIIIIIII", 1, 0, va, va, 0, d, fl, 0x1000)
elf = eh + ph; elf += b"\0" * (off - len(elf)) + data
open(out, "wb").write(elf); print("ok", len(elf))
