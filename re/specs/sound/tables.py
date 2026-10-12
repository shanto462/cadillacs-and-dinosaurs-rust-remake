"""Checks behind the tables and formulas of SPEC.md sections 12 to 15 (task M4-00).

Everything is computed from the user's own ROM at run time (dino.zip: cd_q.5k, decrypted with kabuki.py, and the
four sample ROMs). Nothing is written to disk and no ROM bytes are printed except the few single driver constants
the spec names. Each check prints one line, "ok" or "FAIL" with the reason; the exit code is 1 if any check fails.

usage: python3 -I re/specs/sound/tables.py [--zip PATH]        (default: dino.zip in the repo root)

What it checks
  tables       length, index domain and the formula of every table the driver reads (0x0C50 ... 0x28A9)
  multiply     0x2A81 and 0x4A81 are exact nibble times byte products; the routines 0x29AA and 0x29CB, run on
               z80mini.py over the real bytes, give B*C and (HL*DE)>>16 for every test input
  samples      sample table 0x6A82 (count, order, banks), instrument table 0x6E62, command table, ADPCM headers
  pitch clamp  what the clamp at 0x139D does for every value, and how many notes of the whole ROM reach it
"""
import os
import random
import sys
import zipfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(HERE)))
sys.path.insert(0, HERE)
import kabuki  # noqa: E402
from z80mini import Z80  # noqa: E402

FAILS = []


def check(name, ok, info=""):
    print(f"{'ok  ' if ok else 'FAIL'} {name}" + (f"  ({info})" if info else ""))
    if not ok:
        FAILS.append(name)


# ---------------------------------------------------------------------------------------------------------
# ROM access
# ---------------------------------------------------------------------------------------------------------
def load(zip_path):
    z = zipfile.ZipFile(zip_path)
    raw = z.read("cd_q.5k")
    op, data = kabuki.decrypt(raw)
    pcm = b"".join(z.read(f"cd-q{i}.{i}k") for i in (1, 2, 3, 4))
    return op, data, pcm


class Rom:
    def __init__(self, op, data, pcm):
        self.op, self.d, self.pcm = op, data, pcm

    def b(self, a):
        return self.d[a]

    def w(self, a):                       # little endian word (the Z80 reads tables this way)
        return self.d[a] | (self.d[a + 1] << 8)

    def words(self, a, n):
        return [self.w(a + 2 * i) for i in range(n)]

    def bytes(self, a, n):
        return list(self.d[a:a + n])


# ---------------------------------------------------------------------------------------------------------
# Tables
# ---------------------------------------------------------------------------------------------------------
def check_tables(rom):
    print("== tables (SPEC section 12)")
    # note length tables, 7 bytes each, indexed by (note byte >> 5) - 1
    check("0x0C57 normal lengths = 3*2^k, k=0..6", rom.bytes(0x0C57, 7) == [3 << k for k in range(7)])
    check("0x0C50 triplet lengths = 2*2^k, k=0..6", rom.bytes(0x0C50, 7) == [2 << k for k in range(7)])
    dot = [(3 << k) * 3 // 2 if (3 << k) % 2 == 0 and (3 << k) * 3 // 2 < 256 else 0 for k in range(7)]
    check("0x0C5E dotted lengths = 1.5 * normal, 0 where that is not a byte integer (k=0) or above 255 (k=6)",
          rom.bytes(0x0C5E, 7) == dot)
    # octave base, 16 bytes indexed by track flags & 0x0F (bits 0-2 octave, bit 3 octave bank)
    check("0x0C65 note base = 12*(octave + 2*bank), 16 bytes",
          rom.bytes(0x0C65, 16) == [12 * ((i & 7) + 2 * (i >> 3)) for i in range(16)])
    # pitch tables
    n_ent = 0x6C
    check("0x13DE octave of n = n div 12, 108 bytes", rom.bytes(0x13DE, n_ent) == [n // 12 for n in range(n_ent)])
    check("0x144A first n of octave k = 12*k, 9 bytes", rom.bytes(0x144A, 9) == [12 * k for k in range(9)])
    t1453 = rom.words(0x1453, n_ent)
    check("0x1453 word table, 108 entries (0x1453..0x152A), entry n = entry (n mod 12) << (n div 12)",
          all(t1453[n] == t1453[n % 12] << (n // 12) for n in range(n_ent)))
    base = t1453[:12]
    check("0x1453 octave 0 is about 128*2^(s/12): every entry within 1 of it",
          all(abs(base[s] - 128 * 2 ** (s / 12)) <= 1 for s in range(12)),
          "max error %.2f" % max(abs(base[s] - 128 * 2 ** (s / 12)) for s in range(12)))
    check("0x1453 entry n = 0x3C is 0x1000 (a note at the root key plays at the QSound rate)", t1453[0x3C] == 0x1000)
    t152b = rom.bytes(0x152B, 12 * 256)
    slopes = []
    for s in range(12):
        row = t152b[s * 256:(s + 1) * 256]
        ks = [k for k in range(64) if all(row[f] == (k * f) // 256 for f in range(256))]
        slopes.append(ks[0] if ks else None)
    check("0x152B 12 rows of 256 bytes: row s = floor(k_s * f / 256), f = fraction byte, k_s a small integer",
          None not in slopes, f"k_s range {min(slopes)}..{max(slopes)}, non decreasing {slopes == sorted(slopes)}")
    check("0x152B ends at 0x212A, the code at 0x212C follows", 0x152B + 12 * 256 == 0x212B)
    # the 16 bit tables
    sizes = [("0x23A9", 0x23A9, 128), ("0x24A9", 0x24A9, 128), ("0x25A9", 0x25A9, 64), ("0x2629", 0x2629, 64),
             ("0x26A9", 0x26A9, 128), ("0x27A9", 0x27A9, 128), ("0x28A9", 0x28A9, 128)]
    end = 0x23A9
    for name, a, n in sizes:
        t = rom.words(a, n)
        check(f"{name} {n} words ({2 * n:#x} bytes), non decreasing, max {max(t):#06x}, first {t[0]}",
              all(t[i] <= t[i + 1] for i in range(n - 1)) and a == end)
        end = a + 2 * n
    check("tables 0x23A9..0x29A8 are back to back and end where the multiply code starts (0x29AA)", end == 0x29A9)
    t24 = rom.words(0x24A9, 128)
    check("0x24A9 is about linear: entry i = 512*(i+1) - 2 within 6 for i >= 1, entry 0 = 0, entry 127 = 0xFFFE",
          t24[0] == 0 and t24[127] == 0xFFFE and all(abs(t24[i] - (512 * (i + 1) - 2)) <= 6 for i in range(1, 128)),
          "max error %d" % max(abs(t24[i] - (512 * (i + 1) - 2)) for i in range(1, 128)))
    t28 = rom.words(0x28A9, 128)
    d = [t28[i + 1] - t28[i] for i in range(127)]
    check("0x28A9 is piecewise linear: step 252 for i=2..63, 508 for i=65..95, 1020 for i=96..126",
          set(d[1:63]) == {252} and set(d[65:95]) == {508} and set(d[95:127]) == {1020})
    # music master volume: one byte at linear 0x8002 indexes 0x24A9
    check("0x8002 (music master volume index) is below 128", rom.b(0x8002) < 128, f"value {rom.b(0x8002):#04x}")


def check_multiply(rom):
    print("== multiply tables and routines (SPEC section 13)")
    t_lo = rom.words(0x2A81, 0x1000)
    t_hi = rom.words(0x4A81, 0x1000)
    check("0x2A81 = 4096 words, word[nib*256 + byte] = nib * byte (nib 0..15)",
          all(t_lo[nib * 256 + b] == nib * b for nib in range(16) for b in range(256)))
    check("0x4A81 = 4096 words, word[nib*256 + byte] = (nib * byte) << 4",
          all(t_hi[nib * 256 + b] == (nib * b) << 4 for nib in range(16) for b in range(256)))
    check("0x2A81 + 0x2000 = 0x4A81 and 0x4A81 + 0x2000 = 0x6A81 (one pad byte, then the sample table 0x6A82)",
          0x2A81 + 0x2000 == 0x4A81 and 0x4A81 + 0x2000 == 0x6A81, f"pad byte {rom.b(0x6A81):#04x}")
    z = Z80(rom.op, rom.d)
    # 0x29AA: HL = B * C (8 by 8 bit)
    bad = []
    for b in range(256):
        for c in range(0, 256, 5):
            z.run(0x29AA, b=b, c=c)
            if z.hl != b * c:
                bad.append((b, c))
    check("0x29AA returns B*C in HL for B 0..255, C every 5th value (8 by 8 bit product)", not bad, f"{len(bad)} wrong")
    # 0x29CB: HL = (HL * DE) >> 16
    rnd = random.Random(1)
    tests = [(0, 0), (1, 1), (0xFFFF, 0xFFFF), (0xFFFF, 1), (1, 0xFFFF), (0x8000, 0x8000), (0x4000, 0x4000),
             (0xFFFE, 0xFFFF), (0x0369, 0x00FF), (0xFFFF, 0x0444)]
    tests += [(1 << i, 1 << j) for i in range(16) for j in range(16)]
    tests += [(rnd.getrandbits(16), rnd.getrandbits(16)) for _ in range(60000)]
    tests += [(rnd.getrandbits(16) | 0x0F0F, rnd.getrandbits(16) | 0xF0F0) for _ in range(5000)]
    bad = []
    for a, b in tests:
        z.run(0x29CB, h=a >> 8, l=a & 0xFF, d=b >> 8, e=b & 0xFF)
        if z.hl != (a * b) >> 16:
            bad.append((a, b))
    check(f"0x29CB returns (HL*DE)>>16 (floor of the 32 bit product, high word) for {len(tests)} inputs",
          not bad, f"{len(bad)} wrong, first {bad[:3]}")
    z.run(0x29CB, h=0x12, l=0x34, d=0x56, e=0x78)
    check("0x29CB leaves HL and DE in F000..F003 (F000/F001 = HL low/high, F002/F003 = DE low/high)",
          [z.rd(0xF000 + i) for i in range(4)] == [0x34, 0x12, 0x78, 0x56])
    z.run(0x29CB, h=0xFF, l=0xFF, d=0xFF, e=0xFF)
    print(f"     steps for one 0x29CB call: {z.steps} Z80 instructions (the port can use u32 arithmetic)")


# ---------------------------------------------------------------------------------------------------------
# Command table, samples, instruments
# ---------------------------------------------------------------------------------------------------------
def check_samples(rom):
    print("== command table, samples, instruments (SPEC section 14)")
    n_codes = (rom.b(0x8000) << 8) | rom.b(0x8001)
    live, heads = 0, {}
    for c in range(n_codes):
        e = rom.d[0x8006 + 4 * c:0x8006 + 4 * c + 4]
        ptr = (e[0] << 16) | (e[1] << 8) | e[2]
        if ptr:
            live += 1
            heads.setdefault(rom.b(ptr), []).append(c)
    check("command table: 0x106 entries at 0x8006, 243 live, header types music 39", n_codes == 0x106 and live == 243
          and len(heads.get(0, [])) == 39, f"{n_codes:#x} entries, {live} live")
    check("no header byte >= 0x80 (no ADPCM headers): the ADPCM player is not used by this ROM",
          all(h < 0x80 for h in heads), "priorities used " + ", ".join(f"{h:#x}" for h in sorted(heads) if h))
    n_samples = (0x6E62 - 0x6A82) // 8
    check("sample table 0x6A82..0x6E61 holds 124 entries of 8 bytes, the instrument table starts at 0x6E62",
          n_samples == 124)
    rows = [rom.d[0x6A82 + 8 * i:0x6A82 + 8 * i + 8] for i in range(n_samples)]
    ok = True
    top = 0
    for r in rows:
        bank, st, lp, en = r[0], r[1] | (r[2] << 8), r[3] | (r[4] << 8), r[5] | (r[6] << 8)
        ok &= st <= lp <= en <= 0xFFFF
        top = max(top, bank * 0x10000 + en)
    check("every sample: start <= loop start <= end, all inside the sample ROM", ok and top <= len(rom.pcm),
          f"highest used address {top:#x} of {len(rom.pcm):#x}")
    banks = sorted({r[0] for r in rows})
    check("sample banks 0..31 (64 KB each) = the 2 MB of cd-q1..cd-q4 in that order", banks == list(range(32))
          and len(rom.pcm) == 0x200000)
    print(f"     total sample bytes {sum((r[5] | (r[6] << 8)) - (r[1] | (r[2] << 8)) for r in rows)}")
    # instruments: from the filler pattern it is clear which entries are real
    inst = [rom.d[0x6E62 + 8 * i:0x6E62 + 8 * i + 8] for i in range(256)]
    real = [i for i, e in enumerate(inst) if e[0] < n_samples]
    print(f"     instrument entries whose sample number is a valid sample: {len(real)} (indexes {real[0]}..{real[-1]})")
    filler = [i for i in range(130, 256) if bytes(inst[i]) == bytes([i, 0, 0x3F, 0x3F, 0x7F, 0, 0, 0])]
    check("instrument entries 130..255 are filler (sample number = entry number, fixed envelope bytes)",
          filler == list(range(130, 256)))
    used_inst, used_smp = walk_instruments(rom, heads)
    check("instruments used by sequences: highest index 129", max(used_inst) == 129,
          f"{len(used_inst)} distinct instruments, {len(used_smp)} distinct samples, "
          f"samples never used by a sequence: {sorted(set(range(n_samples)) - used_smp)}")
    # which envelope byte indexes are really used where (instrument byte 2 = attack index into 0x25A9, bytes 3, 5, 6 =
    # decay, slow decay and release rate indexes into 0x2629, byte 4 = sustain level index into 0x24A9)
    e0 = {inst[i][2] for i in used_inst}
    e1 = {inst[i][3] for i in used_inst}
    e3 = {inst[i][5] for i in used_inst}
    e4 = {inst[i][6] for i in used_inst}
    e2 = {inst[i][4] for i in used_inst}
    over = [i for i in used_inst if max(inst[i][3], inst[i][5], inst[i][6]) >= 64]
    check("0x25A9 index (instrument byte 2) < 64, 0x24A9 index (byte 4) < 128, 0x2629 indexes (bytes 3, 5, 6) < 64 "
          "except one release index of 0x40",
          max(e0) < 64 and max(e2) < 128 and max(e1 | e3 | e4) == 0x40 and len(over) == 1
          and inst[over[0]][6] == 0x40 and rom.w(0x26A9) == 0,
          f"max {max(e0):#x}, {max(e2):#x}, {max(e1 | e3 | e4):#x}; instrument {over} has release index 0x40, "
          f"which reads word 0 of 0x26A9 (= {rom.w(0x26A9)})")


def check_operands(rom):
    """Index domains: the command operands that are table indexes, over every reachable command."""
    heads = {}
    for c in range((rom.b(0x8000) << 8) | rom.b(0x8001)):
        e = rom.d[0x8006 + 4 * c:0x8006 + 4 * c + 3]
        ptr = (e[0] << 16) | (e[1] << 8) | e[2]
        if ptr:
            heads.setdefault(rom.b(ptr), []).append(c)
    mx = {}
    for c, h, v, start in {t[3]: t for t in track_starts(rom, heads)}.values():
        for _, b, args in reachable(rom, start):
            if b in (0x07, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x08, 0x18):
                mx[b] = max(mx.get(b, 0), args[0])
    names = {0x07: "volume -> 0x23A9", 0x19: "echo send -> 0x23A9", 0x1A: "music volume -> 0x24A9",
             0x1B: "vibrato depth -> 0x27A9", 0x1C: "tremolo depth -> 0x28A9", 0x1D: "LFO speed -> 0x26A9",
             0x08: "instrument -> 0x6E62", 0x18: "music pan"}
    check("operands used as table indexes: command 0x07, 0x19, 0x1A, 0x1B, 0x1C, 0x1D are below 128",
          all(mx[b] < 128 for b in (0x07, 0x19, 0x1A, 0x1B, 0x1C, 0x1D)),
          "; ".join(f"{b:#04x} {names[b]} max {mx[b]:#04x}" for b in sorted(mx)))


def track_starts(rom, heads):
    for h, codes in heads.items():
        for c in codes:
            e = rom.d[0x8006 + 4 * c:0x8006 + 4 * c + 3]
            ptr = (e[0] << 16) | (e[1] << 8) | e[2]
            for v in range(16):
                off = (rom.b(ptr + 1 + 2 * v) << 8) | rom.b(ptr + 2 + 2 * v)
                if off:
                    yield c, h, v, ptr + off


CMD_ARGS = [0, 0, 0, 0, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 2, 0, 1, 1, 1, 1, 1, 1, 1, 0]


def s16(hi, lo):
    v = (hi << 8) | lo
    return v - 0x10000 if v & 0x8000 else v


def reachable(rom, start):
    """Static walk of one track: yields (address, command, operands) for every command reachable from `start`,
    following both ways of every loop and break."""
    seen, todo = set(), [start]
    while todo:
        p = todo.pop()
        while p not in seen:
            seen.add(p)
            b = rom.b(p)
            if b >= 0x20:
                p += 1
                continue
            n = CMD_ARGS[b]
            args = [rom.b(p + 1 + i) for i in range(n)]
            yield p, b, args
            nxt = p + 1 + n
            if b == 0x17:
                break
            if b == 0x16:
                p = nxt + s16(args[0], args[1])
                continue
            if 0x0E <= b <= 0x15:
                todo.append(nxt + s16(args[1], args[2]))
            p = nxt


def walk_instruments(rom, heads):
    used_inst = set()
    seen_tracks = set()
    for c, h, v, start in track_starts(rom, heads):
        if start in seen_tracks:
            continue
        seen_tracks.add(start)
        for _, b, args in reachable(rom, start):
            if b == 0x08:
                used_inst.add(args[0])
    smp = {rom.b(0x6E62 + 8 * i) for i in used_inst}
    return used_inst, smp


# ---------------------------------------------------------------------------------------------------------
# The pitch clamp at 0x138F..0x13A1
# ---------------------------------------------------------------------------------------------------------
def clamp_n(a):
    """a = (0x3C - root key + (H - 1)) & 0xFF, then `cp 0x6C; jp c,ok; ld a,0x6B; jp m,ok; xor a`."""
    if a < 0x6C:
        return a
    return 0x6B if ((a - 0x6C) & 0x80) else 0


def check_clamp(rom):
    print("== pitch clamp at 0x139D (SPEC section 15)")
    table = {a: clamp_n(a) for a in range(256)}
    check("a 0x00..0x6B passes, a 0x6C..0xEB becomes 0 (too high gives the LOWEST pitch), a 0xEC..0xFF becomes 0x6B "
          "(too low gives the HIGHEST pitch)",
          all(table[a] == a for a in range(0x6C)) and all(table[a] == 0 for a in range(0x6C, 0xEC))
          and all(table[a] == 0x6B for a in range(0xEC, 0x100)))
    # how often do real notes reach the clamp? static estimate over every note of every track
    hits_hi = hits_lo = notes = 0
    a_min, a_max = 0xFF, 0
    n_samples = 124
    heads = {}
    n_codes = (rom.b(0x8000) << 8) | rom.b(0x8001)
    for c in range(n_codes):
        e = rom.d[0x8006 + 4 * c:0x8006 + 4 * c + 3]
        ptr = (e[0] << 16) | (e[1] << 8) | e[2]
        if ptr:
            heads.setdefault(rom.b(ptr), []).append(c)
    worst = []
    tracks = list(track_starts(rom, heads))
    # music transpose F01A is global to a music code: try every value that any track of the code sets (and 0)
    f01a = {}
    for c, h, v, start in tracks:
        if h == 0:
            f01a.setdefault(c, {0}).update(args[0] for _, b, args in reachable(rom, start) if b == 0x0A)
    vib = rom.words(0x27A9, 128)
    risky = []
    for c, h, v, start in tracks:
        r = walk_notes(rom, start, music=(h == 0))
        depth = max([vib[args[0] & 0x7F] for _, b, args in reachable(rom, start) if b == 0x1B] or [0])
        amp = (depth + 255) // 256                # semitones the vibrato can add or subtract (8.8 depth)
        lo_a, hi_a = 0xFF, 0
        for pitch_h, root in r:
            notes += 1
            for t in (f01a.get(c, {0}) if h == 0 else {0}):
                a = (0x3C - root + ((pitch_h + t) - 1)) & 0xFF
                if a < 0x6C:
                    a_min, a_max = min(a_min, a), max(a_max, a)
                    lo_a, hi_a = min(lo_a, a), max(hi_a, a)
                if 0x6C <= a <= 0xEB:
                    hits_hi += 1
                    worst.append((c, v))
                elif a >= 0xEC:
                    hits_lo += 1
                    worst.append((c, v))
        if r and (hi_a + amp >= 0x6C or lo_a < amp):
            risky.append((c, v, amp, lo_a, hi_a))
    print(f"     {notes} note events of all tracks (both ways of every loop, every music transpose a code sets; "
          f"vibrato and portamento not added): {hits_hi} reach 'too high -> 0', {hits_lo} reach 'too low -> 0x6B'")
    print(f"     the value a of the in range notes goes from {a_min:#04x} to {a_max:#04x} (the clamp starts at 0x6C "
          f"and, below 0, at 0xEC)")
    if worst:
        print("     (code, voice) with such notes:", sorted(set(worst))[:12])
    print(f"     tracks where the vibrato depth they set could carry a note over a clamp edge: {len(risky)}"
          + (f", first {risky[:4]} (code, voice, vibrato semitones, lowest a, highest a)" if risky else ""))
    print("     music transposes set per music code:", {f"{c:#x}": sorted(v) for c, v in sorted(f01a.items()) if v != {0}})


def walk_notes(rom, start, music):
    """Rough note pitches of one track: a linear walk that takes loops once and ignores breaks. Returns a list of
    (H of the pitch before the clamp at 0x138F, root key of the instrument). The vibrato term is not added."""
    inst_tab = 0x6E62
    flags = transp = detune = fine = 0
    root = 60
    out = []
    seen = set()
    p = start
    while p not in seen:
        seen.add(p)
        b = rom.b(p)
        if b >= 0x20:
            note5 = b & 0x1F
            p += 1
            if note5:
                base = 12 * ((flags & 7) + 2 * ((flags >> 3) & 1))
                pitch = (note5 + base + transp) & 0xFF
                lo = (detune + fine) & 0xFF
                v = (pitch << 8) + (lo - 256 if lo & 0x80 else lo)
                h = (v >> 8) & 0xFF
                if h >= 0x62:
                    h = 0x61
                out.append((h, root))
            continue
        n = CMD_ARGS[b]
        args = [rom.b(p + 1 + i) for i in range(n)]
        nxt = p + 1 + n
        if b == 0x00:
            flags ^= 0x20
        elif b == 0x01:
            flags ^= 0x40
        elif b == 0x02:
            flags |= 0x10
        elif b == 0x03:
            flags ^= 0x08
        elif b == 0x04:
            flags = (flags & 0x97) | args[0]
        elif b == 0x08:
            ia = inst_tab + 8 * args[0]
            smp = rom.b(ia)
            fine = rom.b(ia + 1)
            if smp < 124:
                root = rom.b(0x6A82 + 8 * smp + 7)
        elif b == 0x09:
            flags = (flags & 0xF8) | args[0]
        elif b == 0x0B:
            transp = args[0]
        elif b == 0x0C:
            detune = args[0]
        elif b == 0x17:
            break
        elif b == 0x16:
            nxt = nxt + s16(args[0], args[1])
        p = nxt
    return out


def main():
    zp = os.path.join(ROOT, "dino.zip")
    if "--zip" in sys.argv:
        zp = sys.argv[sys.argv.index("--zip") + 1]
    op, data, pcm = load(zp)
    rom = Rom(op, data, pcm)
    check_tables(rom)
    check_multiply(rom)
    check_samples(rom)
    check_operands(rom)
    check_clamp(rom)
    print()
    print("ALL OK" if not FAILS else f"{len(FAILS)} FAILED: {FAILS}")
    sys.exit(1 if FAILS else 0)


if __name__ == "__main__":
    main()
