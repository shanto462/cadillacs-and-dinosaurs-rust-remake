"""Kabuki decryption of the dino QSound Z80 program (cd_q.5k).

The Kabuki Z80 encrypts only the first 0x8000 bytes. Each byte is decoded twice with an
address-dependent "select" value: once for opcode fetches (M1 cycles) and once for every other
read (operands, data). Bytes 0x8000.. (the banked music data) are plain.

Writes ../../out/z80_op.bin and ../../out/z80_data.bin (128 KB each, same layout as cd_q.5k).
Keys for this set: swap1 0x76543210, swap2 0x24601357, addr 0x4343, xor 0x43.
"""
import os

RE = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SWAP1, SWAP2, ADDR_KEY, XOR_KEY = 0x76543210, 0x24601357, 0x4343, 0x43


def _swap_pair(src, pair):
    # swap bits (2*pair) and (2*pair+1)
    lo = 1 << (2 * pair)
    hi = lo << 1
    keep = src & ~(lo | hi) & 0xFF
    return keep | ((src & lo) << 1) | ((src & hi) >> 1)


def bitswap1(src, key, select):
    for pair, shift in enumerate((0, 4, 8, 12)):
        if select & (1 << ((key >> shift) & 7)):
            src = _swap_pair(src, pair)
    return src


def bitswap2(src, key, select):
    for pair, shift in enumerate((12, 8, 4, 0)):
        if select & (1 << ((key >> shift) & 7)):
            src = _swap_pair(src, pair)
    return src


def rol(v):
    return ((v << 1) | (v >> 7)) & 0xFF


def bytedecode(src, select):
    src = bitswap1(src, SWAP1 & 0xFFFF, select & 0xFF)
    src = rol(src)
    src = bitswap2(src, SWAP1 >> 16, select & 0xFF)
    src ^= XOR_KEY
    src = rol(src)
    src = bitswap2(src, SWAP2 & 0xFFFF, select >> 8)
    src = rol(src)
    src = bitswap1(src, SWAP2 >> 16, select >> 8)
    return src


def decrypt(raw):
    op = bytearray(raw)
    data = bytearray(raw)
    for a in range(0x8000):
        op[a] = bytedecode(raw[a], (a + ADDR_KEY) & 0xFFFF)
        data[a] = bytedecode(raw[a], ((a ^ 0x1FC0) + ADDR_KEY + 1) & 0xFFFF)
    return bytes(op), bytes(data)


if __name__ == "__main__":
    raw = open(RE + "/rom/cd_q.5k", "rb").read()
    op, data = decrypt(raw)
    open(RE + "/out/z80_op.bin", "wb").write(op)
    open(RE + "/out/z80_data.bin", "wb").write(data)
    print("op  :", op[:16].hex(" "))
    print("data:", data[:16].hex(" "))
