"""Who owns what in the program: object classes, their code ranges, names, and where each class is spawned.

Dispatch tables (re/specs/enemy/SPEC.md): pool E items 0x233F0 (12 classes), pool A enemies 0x2346C (102),
pool B props and controllers 0x2362E (97), pool C effects 0x237E6 (42). A class handler starts its code (and the
data next to it, such as animation tables), so the code range of a class is [handler, next handler).
"""
import os
import re
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

POOLS = {  # pool letter: (dispatch table, class count, update/spawn pool name)
    "E": (0x233F0, 12, "items"),
    "A": (0x2346C, 102, "enemies"),
    "B": (0x2362E, 99, "props and controllers"),
    "C": (0x237E6, 42, "effects"),
}
POOL_TITLE = {"A": "enemy", "B": "prop/controller", "C": "effect", "E": "item", "P": "player", "S": "shared"}
NAME_TABLE = 0x17D78
PLAYER_NAMES = ["Jack", "Mustapha", "Hannah", "Mess"]
PLAYER_SET_PTRS = [0x163E, 0x164E, 0x165E, 0x166E, 0x167E, 0x168E]
PLAYER_SET_LABEL = ["movement", "attacks", "damage", "hold", "held", "weapon-pose"]


class Owners:
    def __init__(self, rom):
        self.rom = rom
        self.handlers = []        # (addr, pool, class)
        for pool, (tbl, n, _) in POOLS.items():
            for i in range(n):
                h = rom.u32(tbl + 4 * i)
                if 0x1000 < h < 0x100000:
                    self.handlers.append((h, pool, i))
        self.handlers.sort()
        self.addrs = [h[0] for h in self.handlers]
        self._names = None
        self.class_names = self._class_names()

    # ---- name table (ids 0..0x26)
    def name(self, nid):
        r = self.rom
        if nid > 0x26:
            return f"name{nid:#x}"
        a = NAME_TABLE + r.u16(NAME_TABLE + 2 * nid)
        s = []
        while r.b[a] != 0x40 and len(s) < 20:
            s.append(r.b[a])
            a += 1
        return bytes(s[1:]).decode("latin-1").strip().title()

    def _class_names(self):
        """Name ids a class assigns (move.b #id,$3(An)) inside its code range."""
        r, b = self.rom, self.rom.b
        out = {}
        for x in range(0x10000, 0x6E000, 2):
            w = (b[x] << 8) | b[x + 1]
            if (w & 0xF1FF) == 0x117C and r.u16(x + 4) == 3:
                nid = r.u16(x + 2) & 0xFF
                c = self.class_at(x)
                if c and c[0] == "A" and nid <= 0x26:
                    out.setdefault(c[1], [])
                    if nid not in out[c[1]]:
                        out[c[1]].append(nid)
        return out

    def class_at(self, addr):
        """(pool, class) whose code range holds addr, or None. Ranges end at the next handler, capped per pool
        (the last classes of a pool are followed by unrelated code)."""
        import bisect
        i = bisect.bisect_right(self.addrs, addr) - 1
        if i < 0:
            return None
        h, pool, cls = self.handlers[i]
        nxt = self.addrs[i + 1] if i + 1 < len(self.addrs) else h + 0x1000
        cap = {"A": 0x6E000, "C": 0x97C20, "E": 0x2B000, "B": h + 0x2800}[pool]
        if addr >= min(nxt, cap):
            return None
        return pool, cls

    def class_label(self, pool, cls):
        if pool == "A":
            ns = self.class_names.get(cls, [])
            nm = "/".join(self.name(n) for n in ns[:3])
            return f"A{cls:02d}" + (f" {nm}" if nm else "")
        return f"{pool}{cls:02d}"

    def classes_of_ref(self, ref):
        """Describes one reference location (address where a pointer to a table sits)."""
        if 0x163E <= ref < 0x16A0:
            k = (ref - 0x163E) // 0x10
            ch = ((ref - 0x163E) % 0x10) // 4
            return ("P", ch, f"{PLAYER_NAMES[ch]} {PLAYER_SET_LABEL[min(k, 5)]}")
        if 0x2E68C <= ref < 0x2E724:
            nid = (ref - 0x2E68C) // 4
            return ("H", nid, f"hit reactions of {self.name(nid)} (name {nid:#x})")
        if 0x2E724 <= ref < 0x2E7C0:
            nid = (ref - 0x2E724) // 4
            return ("G", nid, f"held-by-player poses of {self.name(nid)} (name {nid:#x})")
        c = self.class_at(ref)
        if c:
            return (c[0], c[1], self.class_label(*c))
        return None


# --------------------------------------------------------------------------------------------- spawn lists
def spawn_usage(rom):
    """{(pool, class): {stage: sorted areas}} from the 8 stage spawn lists and wave scripts (interpreter 0xFBF4,
    wave controller 0x7B336, see re/specs/enemy/SPEC.md section 8) plus the per record details for the catalog."""
    r = rom
    W, S16, L = r.u16, r.i16, r.u32
    POOL = {0: "A", 1: "B", 3: "E"}
    use = {}
    items = {}

    def note(f, c, st, ar, extra=None):
        if f & 0x80:
            return
        pool = POOL.get(f & 7)
        if pool is None:
            return
        cls = c // 4
        use.setdefault((pool, cls), {}).setdefault(st, set()).add(ar)

    for st in range(8):
        t1, t2 = L(0xFC2E + 4 * st), L(0xFC4E + 4 * st)
        seen = set()
        for ar in range(4):
            p1, p2 = L(t1 + 4 * ar), L(t2 + 4 * ar)
            if p1 > 0x180000 or p2 > 0x180000:
                break
            if (p1, p2) in seen:
                continue
            seen.add((p1, p2))
            a = p1
            for _ in range(300):
                t, f, c = W(a), W(a + 2), W(a + 4)
                if t >= 0x9FFF:
                    break
                note(f, c, st, ar)
                a += 14
            a = p2
            for _ in range(300):
                if W(a) >= 0x9FFF:
                    break
                f, c = W(a + 2), W(a + 4)
                note(f, c, st, ar)
                if not f & 0x80 and (f & 7) == 3 and c // 4 == 5:
                    items.setdefault(st, []).append(r.u8(a + 15))
                a += 16
        base = L(0x7B37C + 4 * st)
        nw = W(base) // 2
        for w in range(nw):
            a = base + S16(base + 2 * w)
            for _ in range(60):
                a += 6
                if S16(a) < 0:
                    break
                while S16(a) >= 0:
                    note(W(a + 2), W(a + 4), st, -1)
                    a += 16
                a += 16
    return use, items


def episode_list(stages):
    return ", ".join(f"{s + 1}" for s in sorted(stages))
