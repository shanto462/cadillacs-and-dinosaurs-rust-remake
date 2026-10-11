import os, struct
S = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))  # .../re
ROM = open(S + "/out/dino_68k.bin", "rb").read()
def w(a): return struct.unpack_from(">H", ROM, a)[0]
def sw(a): return struct.unpack_from(">h", ROM, a)[0]
def l(a): return struct.unpack_from(">I", ROM, a)[0]
def b(a): return ROM[a]
def hexd(a, n, width=16):
    for i in range(0, n, width):
        print("%06X: %s" % (a + i, " ".join("%04X" % w(a + i + j) for j in range(0, min(width, n - i), 2))))
