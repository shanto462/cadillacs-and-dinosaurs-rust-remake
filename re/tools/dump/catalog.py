"""Builds the catalog of animation tables, frames and compositions with their owners.

Result (dict) is cached in re/out/assets/data/catalog.json by build(). Other tools load it with load().
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from romio import Rom, outdir, hx
from scan import Scanner
from owners import Owners, spawn_usage, POOL_TITLE, PLAYER_NAMES

BOX_REGION = (0x100000, 0x11D220)
MASTER = 0xB7470            # developer list of animation tables: (u16, u32 table, u16) x 187

BOSS_NAMES = {"Vice T.", "Butcher", "Hogg", "Slice", "Morgan", "Morgue", "Tyrog", "Slisaur", "Fessenden", "Tyrog 2", "Lash T.", "Walther"}
DINO_NAMES = {"Shivat", "R Hopper", "Mack", "Zeke", "Tyrog", "Tyrog 2", "Slisaur"}


def slug(s):
    out = []
    for ch in s.lower():
        out.append(ch if ch.isalnum() else "_")
    return "_".join(p for p in "".join(out).split("_") if p)


def master_list(rom):
    rows, a = [], MASTER
    while True:
        w1, p, w2 = rom.u16(a), rom.u32(a + 2), rom.u16(a + 6)
        if p == 0 or p > 0x100000:
            break
        rows.append((a, w1, p, w2))
        a += 8
    return rows


def is_code_ref(rom, a, kind):
    """True when the reference sits in an instruction (an immediate operand), not in a data table."""
    if kind == "lea_pc":
        return True
    if kind != "ptr" or a < 2:
        return False
    w = rom.u16(a - 2)
    return (w & 0xF1FF) in (0x217C, 0x207C, 0x41F9) or w == 0x4879


def box_candidates(rom):
    """All `move.l #B,$44(An)` with B in the hit box region: [(code addr, B)]."""
    b = rom.b
    out = []
    for x in range(0, 0xA0000, 2):
        w = (b[x] << 8) | b[x + 1]
        if (w & 0xF1FF) == 0x217C and rom.u16(x + 6) == 0x44:
            v = rom.u32(x + 2)
            if BOX_REGION[0] <= v < BOX_REGION[1]:
                out.append((x, v))
    return out


def direct_refs(rom):
    """Pointers that code loads straight into a register or object field: `lea F.l,An`, `movea.l #F,An`,
    `pea F.l`, `lea d16(pc),An`, `move.l #F,$28/$30(An)`. Returns [(code addr, value)]."""
    b = rom.b
    out = []
    for x in range(0x1000, 0xA6000, 2):
        w = (b[x] << 8) | b[x + 1]
        if (w & 0xF1FF) == 0x41F9 or (w & 0xF1FF) == 0x207C or w == 0x4879:
            out.append((x + 2, rom.u32(x + 2)))
        elif (w & 0xF1FF) == 0x41FA:
            out.append((x + 2, x + 2 + rom.i16(x + 2)))
        elif (w & 0xF1FF) == 0x217C and rom.u16(x + 6) in (0x28, 0x30):
            out.append((x + 2, rom.u32(x + 2)))
    return out


def raw_list_ok(rom, p):
    """Sprite list used by 8-byte raw frames: u16 count, u16 n, then n x (dx, dy, attr, code)."""
    if p & 1 or p < 0x1000 or p + 4 > 0x180000:
        return False
    n = rom.u16(p + 2)
    if not 1 <= n <= 48 or p + 4 + 8 * n > 0x180000:
        return False
    for k in range(n):
        q = p + 4 + 8 * k
        if rom.u16(q + 6) >= 0x7000 or rom.u16(q + 6) < 0x100:
            return False
        if abs(rom.i16(q)) > 600 or abs(rom.i16(q + 2)) > 600:
            return False
    return True


def raw_chain(rom, start):
    """8-byte records: u32 list ptr, 4 bytes, last byte bit7 = end (link i16 follows). Returns record addresses or None."""
    seen, out, stack = set(), [], [start]
    while stack:
        p = stack.pop()
        while p not in seen:
            if p & 1 or p + 10 > 0x180000 or not raw_list_ok(rom, rom.u32(p)):
                return None
            seen.add(p)
            out.append(p)
            if len(out) > 80:
                return None
            if rom.b[p + 7] & 0x80:
                stack.append(p + 8 + rom.i16(p + 8))
                break
            p += 8
    return out


def build(rom=None, verbose=True):
    rom = rom or Rom()
    sc = Scanner(rom)
    own = Owners(rom)
    found = sc.find_tables()
    master = master_list(rom)
    mrow = {p: (i, a, w1, w2) for i, (a, w1, p, w2) in enumerate(master)}
    # tolerant tables from the developer list that the strict scan rejected
    for p in mrow:
        if p in found:
            continue
        for stride in (10, 12):
            res = sc.table(p, stride, tolerant=True)
            if res:
                found[p] = dict(stride=stride, n=res[0], offs=res[1], frames=res[2], refs=sc.refs.get(p, []) + [(mrow[p][1] + 2, "master")])
                break
    # tolerant: drop bad ids of master tables that passed strictly? (they passed strictly, nothing to drop)
    boxes = box_candidates(rom)
    use, items = spawn_usage(rom)
    tables = {}
    for t, f in sorted(found.items()):
        refs = [(a, k) for a, k in f["refs"] if not (0xB7400 <= a < 0xB7A60)]
        code_refs = [(a, k) for a, k in refs if is_code_ref(rom, a, k)]
        owners = []
        for a, k in (code_refs or refs):
            c = own.classes_of_ref(a)
            if c and c not in owners:
                owners.append(c)
        tables[t] = dict(addr=t, stride=f["stride"], n=f["n"], offs=f["offs"], frames=f["frames"], refs=refs,
                         owners=owners, master=mrow.get(t, (None,))[0], master_flags=(mrow[t][2], mrow[t][3]) if t in mrow else None)
    # owners for tables without a code owner: the nearest earlier master entry that has one
    order = [p for (_, _, p, _) in master]
    for i, p in enumerate(order):
        if p in tables and not tables[p]["owners"]:
            j = i - 1
            while j >= 0 and not (order[j] in tables and tables[order[j]]["owners"]):
                j -= 1
            if j >= 0:
                tables[p]["owners"] = tables[order[j]]["owners"][:1]
                tables[p]["owner_guess"] = f"master list neighbor {order[j]:#x}"
    # box table per table: nearest `move.l #B,$44` to a code reference, else class majority
    box_by_class = {}
    for x, v in boxes:
        c = own.class_at(x)
        if c:
            box_by_class.setdefault(c, {}).setdefault(v, 0)
            box_by_class[c][v] += 1
    for t, d in tables.items():
        d["box"] = None
        if d["owners"] and d["owners"][0][0] == "P":
            d["box"] = rom.u32(0x18CC4 + 4 * d["owners"][0][1])
            continue
        best = None
        for a, k in d["refs"]:
            for x, v in boxes:
                if abs(x - a) <= 0x60:
                    dist = abs(x - a)
                    if best is None or dist < best[0]:
                        best = (dist, v)
        if best:
            d["box"] = best[1]
        else:
            for o in d["owners"]:
                key = (o[0], o[1])
                if o[0] in ("H", "G"):
                    # a class that uses that name id
                    for cl, ns in own.class_names.items():
                        if o[1] in ns and ("A", cl) in box_by_class:
                            key = ("A", cl)
                            break
                if key in box_by_class:
                    d["box"] = max(box_by_class[key].items(), key=lambda kv: kv[1])[0]
                    break
    # usage: episodes and first (stage, area) per table owner
    for t, d in tables.items():
        eps, first = set(), None
        for o in d["owners"]:
            key = (o[0], o[1])
            if o[0] in ("H", "G"):
                cls = [cl for cl, ns in own.class_names.items() if o[1] in ns]
                keys = [("A", cl) for cl in cls]
            else:
                keys = [key]
            for k in keys:
                for st, ars in use.get(k, {}).items():
                    eps.add(st)
                    ars2 = sorted(a for a in ars if a >= 0) or [0]
                    cand = (st, ars2[0])
                    if first is None or cand < first:
                        first = cand
        d["episodes"] = sorted(eps)
        d["pal_set"] = first
    # kinds
    for t, d in tables.items():
        d["kind"], d["label"], d["group"] = classify(own, d)
    # frame chains and raw lists that code loads directly (no animation table)
    known_frames = set()
    for d in tables.values():
        known_frames.update(d["frames"])
    single, rawc = {}, {}
    for x, v in direct_refs(rom):
        if v in known_frames or v in single or v in rawc:
            continue
        if sc.frame_ok(v, 10):
            ch = sc.chain(v, 10)
            if ch and len(ch) <= 120 and any(rom.b[q + 7] & 0x80 for q in ch):
                single[v] = dict(addr=v, ref=x, chain=ch, owner=own.classes_of_ref(x))
                known_frames.update(ch)
                continue
        ch = raw_chain(rom, v)
        if ch:
            rawc[v] = dict(addr=v, ref=x, chain=ch, owner=own.classes_of_ref(x))
    for dd in list(single.values()) + list(rawc.values()):
        o = dd["owner"]
        dd["kind"], dd["label"], dd["group"] = classify(own, dict(owners=[o] if o else []))
        dd["episodes"], dd["pal_set"], dd["box"] = [], None, None
        if o and o[0] in ("A", "B", "C", "E"):
            eps = use.get((o[0], o[1]), {})
            dd["episodes"] = sorted(eps)
            if eps:
                st = min(eps)
                dd["pal_set"] = (st, (sorted(a for a in eps[st] if a >= 0) or [0])[0])
    # composition coverage
    comp_users = {}
    n_comp = rom.u16(0x0FFFF8)
    for t, d in tables.items():
        for f in d["frames"]:
            for ci in frame_comps(rom, f):
                comp_users.setdefault(ci, set()).add(t)
    for v, d in single.items():
        for f in d["chain"]:
            for ci in frame_comps(rom, f):
                comp_users.setdefault(ci, set()).add(-v)
    cat = dict(tables=tables, single=single, raw=rawc, comp_users=comp_users, n_comp=n_comp, master=master, use=use, items=items,
               class_names=own.class_names)
    if verbose:
        print(f"{len(tables)} animation tables ({sum(t['n'] for t in tables.values())} ids), {len(single)} direct frame chains, {len(rawc)} raw-list chains, "
              f"{len(comp_users)} of {n_comp} compositions referenced by frames")
    return cat, own


def frame_comps(rom, f):
    """Composition byte-indices a 10/12 byte frame record draws (main, overlay, compound parts)."""
    out = []
    w0, w1 = rom.u16(f), rom.u16(f + 2)
    if w0 & 0x8000:
        q = f + 2 + rom.i16(f + 2)
        n = rom.u16(q) + 1
        for k in range(n):
            out.append(rom.u16(q + 2 + 8 * k + 4))
    else:
        out.append(w0)
        if not w1 & 0x8000:
            out.append(w1)
    return out


def describe(own, o):
    return o[2]


def class_kind(own, idx):
    ns = own.class_names.get(idx, [])
    names = [own.name(n) for n in ns]
    kind = "enemy" if names else "actor"
    if any(n in DINO_NAMES for n in names):
        kind = "dinosaur"
    elif any(n in BOSS_NAMES for n in names):
        kind = "boss"
    label = f"A{idx:02d}" + (" " + "/".join(names[:3]) if names else "")
    return kind, label, f"{kind}_a{idx:02d}" + ("_" + slug(names[0]) if names else "")


def classify(own, d):
    """(kind, label, group dir name) of a table from its first owner. Hit and held tables of a name join the
    group of the class that spawns that name."""
    if not d["owners"]:
        return "unknown", "unknown", "unknown"
    pool, idx, lab = d["owners"][0]
    if pool == "P":
        return "hero", PLAYER_NAMES[idx], "hero_" + slug(PLAYER_NAMES[idx])
    if pool in ("H", "G"):
        cls = sorted(cl for cl, ns in own.class_names.items() if idx in ns)
        what = "hit reactions" if pool == "H" else "held poses"
        if cls:
            kind, label, group = class_kind(own, cls[0])
            return kind, f"{label} ({what} of {own.name(idx)})", group
        nm = own.name(idx)
        return "enemy", f"{what} of {nm}", "enemy_" + slug(nm)
    if pool == "A":
        return class_kind(own, idx)
    if pool == "B":
        return "prop", f"B{idx:02d}", f"prop_b{idx:02d}"
    if pool == "C":
        return "effect", f"C{idx:02d}", f"effect_c{idx:02d}"
    if pool == "E":
        return "item", f"E{idx:02d}", f"item_e{idx:02d}"
    return "unknown", "unknown", "unknown"


def save(cat, path=None):
    path = path or os.path.join(outdir("data"), "catalog.json")
    ser = dict(
        tables={hx(t): {k: (v if k not in ("frames", "offs", "refs", "owners") else v) for k, v in d.items()} for t, d in cat["tables"].items()},
        single={hx(v): d for v, d in cat["single"].items()},
        comp_users={str(k): sorted(v) for k, v in cat["comp_users"].items()},
    )
    with open(path, "w") as f:
        json.dump(ser, f)


if __name__ == "__main__":
    cat, own = build()
    t = cat["tables"]
    from collections import Counter
    print(Counter(d["kind"] for d in t.values()))
    noown = [hex(a) for a, d in t.items() if not d["owners"]]
    print("no owner:", len(noown), noown[:60])
    print("direct frames:", [(hex(v), d["owner"]) for v, d in list(cat["single"].items())[:40]])
    unref = [i for i in range(cat["n_comp"]) if i * 4 not in cat["comp_users"]]
    print("unreferenced comps:", len(unref))
