# Linear 68000 disassembly of a range: dis.py image start count
import sys, capstone
img = open(sys.argv[1], "rb").read()
start = int(sys.argv[2], 16); count = int(sys.argv[3])
md = capstone.Cs(capstone.CS_ARCH_M68K, capstone.CS_MODE_BIG_ENDIAN | capstone.CS_MODE_M68K_000)
addr = start; n = 0
while n < count:
    ins = next(md.disasm(img[addr:addr+16], addr), None)
    if ins is None:
        print(f"{addr:06x}: dc.w ${img[addr]:02x}{img[addr+1]:02x}"); addr += 2
    else:
        print(f"{ins.address:06x}: {ins.mnemonic:8s} {ins.op_str}"); addr += ins.size
    n += 1
