# Enemy->player damage data used by 0x116B2 + 0x6422
import sys, struct
img = open(sys.argv[1], "rb").read()
def name(i):
    off = struct.unpack(">h", img[0x17d78+2*i:0x17d7a+2*i])[0]; a = 0x17d78+off
    return img[a+1:img.index(b'@',a)].decode().strip()
print("Attack table 0xA2EA4: 8 bytes per attack id (+$58 of attacker): dmg, poise_dmg, reaction(+$6/+$7b), hitstop(+$59), sound, b5, b6(d5), +$9a")
for k in range(0x80):
    r = img[0xa2ea4+8*k:0xa2eac+8*k]
    print(f"  atk {k:#04x}: dmg={r[0]:3d} poise={r[1]:3d} react={r[2]:#04x} stop={r[3]:3d} snd={r[4]:#04x} b5={r[5]:#04x} b6={r[6]:#04x} 9a={r[7]:#04x}")
print("Damage scale 0x2C3AE: row = attacker +$84*8 (= name_id*32), col = level $5ee&31; dmg = dmg*v>>5")
for i in range(0x27):
    v = img[0x2c3ae+32*i:0x2c3ae+32*i+32]
    print(f"  {i:#04x} {name(i):10s} " + " ".join(f"{x:2d}" for x in v))
print("Random reduction 0x2C896 (words), index = band(HP>=76:0,>=51:0x40,>=26:0x80,else 0xC0) + (rand&0x3E)")
for b in range(4):
    print("  band", b, list(struct.unpack(">32H", img[0x2c896+0x40*b:0x2c896+0x40*b+0x40])))
print("Character defence 0x2C88E (by player +$3):", struct.unpack(">4H", img[0x2c88e:0x2c896]))
print("HP-based scale 0x2C996 (index (HP>>2)&0x1E):", struct.unpack(">16H", img[0x2c996:0x2c9b6]))
