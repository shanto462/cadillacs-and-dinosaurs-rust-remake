# Linear-sweep 68000 listing of the whole program image
import sys, capstone
img = open(sys.argv[1], "rb").read()
md = capstone.Cs(capstone.CS_ARCH_M68K, capstone.CS_MODE_BIG_ENDIAN | capstone.CS_MODE_M68K_000)
md.skipdata = True
with open(sys.argv[2], "w") as f:
    for ins in md.disasm(img, 0):
        f.write(f"{ins.address:06x}: {ins.mnemonic:8s} {ins.op_str}\n")
