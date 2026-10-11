# Boss HP routine 0x31E14 (d0 = boss index): word at 0x31E52 + idx*48 + (players-1)*16 + ((level & 0x1C) >> 1)
import sys, struct
img = open(sys.argv[1], "rb").read()
NAMES = {0:"VICE T./SLICE/SLISAUR/TYROG(alt)",1:"BUTCHER",2:"HOGG",3:"?",4:"MORGAN",5:"MORGUE",6:"TYROG",7:"?",8:"FESSENDEN(final)",9:"FESSENDEN"}
for i in range(10):
    print(f"boss idx {i} {NAMES[i]}")
    for p in range(3):
        v = struct.unpack(">8H", img[0x31e52+48*i+16*p:0x31e52+48*i+16*p+16])
        print(f"   {p+1}P level 0-3,4-7,...,28-31:", list(v))
