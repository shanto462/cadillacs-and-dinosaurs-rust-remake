# CPS1 graphics ROM: interleave the 8 mask ROMs (ROM_LOAD64_WORD) and decode 4bpp planar tiles
import sys, os, numpy as np
from PIL import Image
def load_gfx(rom):
    order = [("cd-1m.3a", 0x000000, 0), ("cd-3m.5a", 0x000000, 2), ("cd-2m.4a", 0x000000, 4), ("cd-4m.6a", 0x000000, 6),
             ("cd-5m.7a", 0x200000, 0), ("cd-7m.9a", 0x200000, 2), ("cd-6m.8a", 0x200000, 4), ("cd-8m.10a", 0x200000, 6)]
    g = np.zeros(0x400000, np.uint8)
    for name, base, off in order:
        d = np.frombuffer(open(os.path.join(rom, name), "rb").read(), np.uint8).reshape(-1, 2)
        n = d.shape[0]
        idx = base + off + np.arange(n) * 8
        g[idx] = d[:, 0]; g[idx + 1] = d[:, 1]
    return g
def decode_row_bytes(b4):
    # 4 bytes = 8 pixels, byte i holds one bitplane (MSB = leftmost pixel)
    bits = np.unpackbits(b4.reshape(-1, 4), axis=1).reshape(-1, 4, 8)   # [n, plane_byte, x]
    # plane_byte 0 -> bit0 of colour? try weights
    return bits
def tile16(g, code, weights=(1, 2, 4, 8)):
    t = g[code * 128:(code + 1) * 128].reshape(16, 2, 4)          # 16 rows, 2 halves, 4 plane bytes
    bits = np.unpackbits(t, axis=2).reshape(16, 2, 4, 8)          # row, half, plane, x
    px = sum(bits[:, :, p, :].astype(np.uint8) * weights[p] for p in range(4))
    return px.reshape(16, 16)
if __name__ == "__main__":
    rom, out = sys.argv[1], sys.argv[2]
    g = load_gfx(rom)
    np.save(out + "/gfx.npy", g)
    # sheet: 128 tiles per row, 0x8000 16x16 tiles -> 256 rows of tiles (4096 x 4096 px), split in 4 images
    for part in range(4):
        img = np.zeros((64 * 16, 128 * 16), np.uint8)
        for i in range(0x2000):
            code = part * 0x2000 + i
            r, c = divmod(i, 128)
            img[r*16:(r+1)*16, c*16:(c+1)*16] = tile16(g, code)
        Image.fromarray((img * 17).astype(np.uint8)).save(f"{out}/sheet16_{part}.png")
    print("ok")
