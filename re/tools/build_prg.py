# Build the 68000 program image from the CPS1 'dino' ROMs (ROM_LOAD16_WORD_SWAP order)
import sys, struct, os
rom, out = sys.argv[1], sys.argv[2]
data = bytearray()
for name in ["cde_23a.8f", "cde_22a.7f", "cde_21a.6f"]:
    b = bytearray(open(os.path.join(rom, name), "rb").read())
    b[0::2], b[1::2] = b[1::2], b[0::2]   # swap bytes in each 16-bit word
    data += b
open(out + "/dino_68k.bin", "wb").write(data)

# Wrap in a minimal big-endian ELF32 (EM_68K=4) so a disassembler loads it at 0x000000
entry = struct.unpack(">I", data[4:8])[0]
ehsize, phsize = 52, 32
off = 0x1000
eh = b"\x7fELF" + bytes([1, 2, 1, 0]) + b"\0" * 8
eh += struct.pack(">HHIIIIIHHHHHH", 2, 4, 1, entry, ehsize, 0, 0, ehsize, phsize, 1, 0, 0, 0)
ph = struct.pack(">IIIIIIII", 1, off, 0, 0, len(data), len(data), 5, 0x1000)
elf = eh + ph
elf += b"\0" * (off - len(elf)) + data
open(out + "/dino_68k.elf", "wb").write(elf)
print("size", hex(len(data)), "SSP", hex(struct.unpack(">I", data[0:4])[0]), "PC", hex(entry))
for i in range(64):
    v = struct.unpack(">I", data[i*4:i*4+4])[0]
    print(f"vec{i:02d} {v:08x}", end="  " if i % 4 != 3 else "\n")
