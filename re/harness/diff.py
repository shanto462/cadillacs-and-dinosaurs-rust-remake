"""Compare a MAME reference trace with a port trace, frame by frame.

Usage:
    re/.venv/bin/python -I re/harness/diff.py MAME.trc PORT.trc [--allow FILE] [--frames N]
        [--first N] [--from FRAME] [--field KEY]

Report:
  1. the first frame with a difference (not allowed), and every differing field at that frame
     (object handle and kind, field offset and name, MAME value, port value);
  2. a summary per field: frames with a difference, first frame, first values;
  3. video and sound: tilemap cells, palette words, sprite list entries, sound requests;
  4. by object class: which object classes the differing fields belong to, with the handler
     address from the original's dispatch table (needs the program image, --rom-image) and whether
     the port runs that class. Differences of a class the port does not run are "unported": the
     class is a task, not a bug. They are held frozen at the snapshot state (Game::trace_hold).
  5. "not modeled": bytes that change in MAME but that the port does not trace (per object kind
     and offset, and the busiest global addresses). These are not counted as differences.

Frame -1 is the sync state: MAME's snapshot against the port right after --load-state. A
difference there means the loader missed a field.

Allow file (one rule per line, '#' starts a comment): a key pattern (fnmatch) and an optional
frame range, e.g.
    enemy.*            # pool A classes 0x36 / 0x37 (cut-scene thugs) are not ported
    global.rng 0-120   # ...
Keys: "<kind>.<field>" for objects (kinds: player enemy prop item effect special), "global.<name>",
"scroll1", "scroll2", "scroll3", "palette", "rowscroll", "sprites", "sounds".
"""

import argparse
import difflib
import fnmatch
import os
import struct
import sys
from collections import defaultdict

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trace as T  # noqa: E402

RAM = 0xFF0000

# Class dispatch tables of the original (u32 handler per class): kind -> (table address, entries,
# index shift). Pool A / B / E tables are indexed by +$20 (class * 4); pool C by +$20 too.
HANDLERS = {"enemy": (0x2346C, 102), "prop": (0x2362E, 97), "item": (0x233F0, 12), "effect": (0x237E6, 42)}


class Field:
    __slots__ = ("key", "label", "addr", "size")

    def __init__(self, key, label, addr, size):
        self.key, self.label, self.addr, self.size = key, label, addr, size


def build_fields(hdr):
    """Field table from the port header and a per-byte index into it (for the 64 KB RAM)."""
    fields = []
    index = np.full(0x10000, -1, dtype=np.int32)
    obj_of = np.full(0x10000, -1, dtype=np.int32)  # pool slot id per byte (for "not modeled")
    slots = []
    for p in hdr["pools"]:
        kind = p["kind"]
        flist = hdr["fields"].get(kind, [])
        for i in range(p["count"]):
            base = p["base"] + i * p["stride"]
            sid = len(slots)
            slots.append((kind, i, base))
            obj_of[base - RAM : base - RAM + p["stride"]] = sid
            for off, size, name in flist:
                fid = len(fields)
                fields.append(Field(f"{kind}.{name}", f"{kind}[{i}] {base & 0xFFFF:04X} +${off:02X} {name}", base + off, size))
                index[base + off - RAM : base + off - RAM + size] = fid
    for addr, size, name in hdr["globals"]:
        fid = len(fields)
        g = addr - 0xFF8000
        where = f"G.${g:X}" if g >= 0 else f"G.-${-g:X}"
        fields.append(Field(f"global.{name}", f"{where} {name}", addr, size))
        index[addr - RAM : addr - RAM + size] = fid
    return fields, index, obj_of, slots


def value(img, addr, size):
    o = addr - RAM
    v = 0
    for k in range(size):
        v = (v << 8) | int(img[o + k])
    return v


def fmt(v, size):
    return f"{v:0{2 * size}X}"


class Allow:
    def __init__(self, path):
        self.rules = []
        # `cell LAYER MAME PORT`: a tilemap cell that is MAME (8 hex digits: code, attribute) in MAME and PORT in the port is no difference
        # (the join prompt cells that the port does not draw yet, allow/m228.txt)
        self.cells = []
        # `rect scroll1 COL0 COL1 ROW0 ROW1`: the cells of scroll1 in these columns and rows (cell index = column * 32 + row) are not compared
        self.rects = []
        if not path:
            return
        with open(path) as f:
            for line in f:
                line = line.split("#")[0].strip()
                if not line:
                    continue
                parts = line.split()
                if parts[0] == "rect" and len(parts) == 6:
                    self.rects.append((parts[1], *[int(x, 0) for x in parts[2:]]))
                    continue
                if parts[0] == "cell" and len(parts) == 4:
                    self.cells.append((parts[1], int(parts[2], 16), int(parts[3], 16)))
                    continue
                lo, hi = -1, 1 << 30
                if len(parts) > 1:
                    a, _, b = parts[1].partition("-")
                    lo = int(a)
                    hi = int(b) if b else lo
                self.rules.append((parts[0], lo, hi))

    def ok(self, key, frame):
        return any(fnmatch.fnmatchcase(key, p) and lo <= frame <= hi for p, lo, hi in self.rules)


LAYERS = [("scroll1", 0x8000), ("scroll2", 0xC000), ("scroll3", 0x10000)]


def sprite_diff(sm, sp, src, owner):
    """Compares two sprite lists. `src[k]` = handle of the object that drew port entry k, and
    `owner(handle)` says whether that object's class is "ported" or "unported" (or "none").
    Returns (counts, first) where counts maps a category to the number of entries and `first` is
    a description of the first entry that is not explained by an unported object:
      unported     entries of the port that differ and come from an unported object
      ported_same  ... from a ported object whose fields differ too (a consequence of that)
      ported       ... from a ported object whose fields do not differ, or from no object (a draw bug?)
      mame_only entries that only MAME has (the port drew fewer entries)"""
    counts = defaultdict(int)
    first = None
    h = lambda e: "x %04X y %04X code %04X attr %04X" % e  # noqa: E731
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, sm, sp, autojunk=False).get_opcodes():
        if tag == "equal":
            continue
        for j in range(j1, j2):
            cat = owner(src[j] if j < len(src) else 0)
            counts[cat if cat in ("unported", "ported_same") else "ported"] += 1
            if cat not in ("unported", "ported_same") and first is None:
                first = f"port entry {j} ({h(sp[j])}) drawn by {src[j]:04X} ({cat})" + (f", MAME has {h(sm[i1 + j - j1])}" if i1 + j - j1 < i2 else ", MAME has none")
        if i2 - i1 > j2 - j1:
            n = (i2 - i1) - (j2 - j1)
            counts["mame_only"] += n
            if first is None:
                first = f"MAME has {n} entries the port lacks, starting at MAME entry {i1 + (j2 - j1)} ({h(sm[i1 + (j2 - j1)])})"
    return counts, first


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("mame")
    ap.add_argument("port")
    ap.add_argument("--allow", help="allow file of approved differences")
    ap.add_argument("--frames", type=int, default=1 << 30, help="compare at most N frames")
    ap.add_argument("--from", dest="start", type=int, default=-1, help="first frame to compare")
    ap.add_argument("--first", type=int, default=40, help="fields listed for the first differing frame")
    ap.add_argument("--field", help="print this field (key pattern) every frame it differs")
    ap.add_argument("--snd-table", action="store_true", help="per sound code: MAME (d1, d2) and port (d1, d2) counts (M4-13)")
    ap.add_argument("--at", type=int, action="append", default=[], help="list every difference at this frame (repeatable)")
    ap.add_argument("--rom-image", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "out", "dino_68k.bin"),
                    help="68000 program image, only used to print handler addresses (optional)")
    args = ap.parse_args()

    tm, tp = T.Trace(args.mame), T.Trace(args.port)
    if tp.source != "port":
        tm, tp = tp, tm
    hdr = tp.header
    fields, index, obj_of, slots = build_fields(hdr)
    strides = {p["kind"]: p["stride"] for p in hdr["pools"]}
    allow = Allow(args.allow)
    ported = hdr.get("ported", {})
    image = open(args.rom_image, "rb").read() if os.path.exists(args.rom_image) else None

    def handler(kind, cls):
        if image is None or kind not in HANDLERS:
            return "?"
        base, n = HANDLERS[kind]
        if cls >= n:
            return "?"
        a = struct.unpack_from(">I", image, base + 4 * cls)[0]
        return f"{a:#07x}" if a else "null"

    pool_list = [(p["kind"], p["base"] - RAM, p["stride"], p["count"]) for p in hdr["pools"]]
    sound_kinds = {"mame_only": [0, None], "other": [0, None]}
    sound_codes = {}  # code -> [count, last example] for the d1 / d2 differences
    snd_tab = {}  # code -> {"m": Counter of MAME (d1, d2), "p": Counter of port (d1, d2)}
    sound_args = [0, None, 0]  # frames where the codes agree but d1 or d2 differ (first frame, text); requests compared
    unported_diff_frames = set()  # frames with a difference in an object of an unported class
    sprite_cats = defaultdict(int)    # category -> differing entries (all frames)
    sprite_frames = defaultdict(int)  # category -> frames
    by_class = {}  # (kind, class) -> {"slots": set, "objframes": set, "first": frame, "fields": {name: first frame}}

    per_field = {}  # key -> [frames, first frame, label, mame, port]
    first_frame = None
    first_list = []
    allowed = defaultdict(int)
    video = {k: [0, None, None] for k in ("scroll1", "scroll2", "scroll3", "palette", "rowscroll", "sprites", "sounds")}
    changed = np.zeros(0x10000, dtype=bool)
    lag_frames = []
    prev_ram = None
    frames = 0
    watch = []
    at_lists = {}

    for rm, rp in zip(tm.records(), tp.records()):
        if rm.frame != rp.frame:
            print(f"frame mismatch: MAME record {rm.frame}, port record {rp.frame}")
            break
        f = rm.frame
        if f >= args.frames:
            break
        mram, mgfx = rm.images["ram"], rm.images["gfx"]
        if rm.flags & 1:
            lag_frames.append(f)
        if prev_ram is not None:
            delta = mram != prev_ram
            live_bytes = np.zeros(0x10000, dtype=bool)
            for kind, i, base in slots:
                if mram[base - RAM] != 0:
                    live_bytes[base - RAM : base - RAM + strides[kind]] = True
            changed |= delta & (live_bytes | (obj_of < 0))
        sounds_m = T.mame_sounds(prev_ram, mram)
        prev_ram = mram.copy()
        if f < args.start:
            continue
        frames += 1
        pram, pmask = rp.images["ram"], rp.images["ram_mask"].astype(bool)

        frame_diffs = []
        diff_handles = set()  # handles of the objects with a field difference in this frame
        idx = np.nonzero(pmask & (pram != mram))[0]
        seen = set()
        for o in idx:
            fid = int(index[o])
            if fid < 0:
                key, label, addr, size = "unnamed", f"{RAM + o:06X}", RAM + int(o), 1
            else:
                if fid in seen:
                    continue
                seen.add(fid)
                fd = fields[fid]
                key, label, addr, size = fd.key, fd.label, fd.addr, fd.size
            mv, pv = value(mram, addr, size), value(pram, addr, size)
            if allow.ok(key, f):
                allowed[key] += 1
                continue
            sid = int(obj_of[o])
            if sid >= 0:
                kind, slot_i, sbase = slots[sid]
                diff_handles.add(sbase - RAM)
                if mram[sbase - RAM] != 0:
                    cls, side = value(mram, sbase + 0x20, 2) // 4, "MAME"
                else:
                    cls, side = value(pram, sbase + 0x20, 2) // 4, "port only"
                if kind != "player" and cls not in ported.get(kind, []):
                    unported_diff_frames.add(f)
                bc = by_class.setdefault((kind, cls), {"slots": set(), "objframes": set(), "first": f, "fields": {}, "port_only": 0})
                bc["slots"].add(slot_i)
                bc["objframes"].add((slot_i, f))
                bc["fields"].setdefault(key.split(".", 1)[1], f)
                if side == "port only":
                    bc["port_only"] += 1
            frame_diffs.append((addr, label, fmt(mv, size), fmt(pv, size)))
            e = per_field.get(key)
            if e is None:
                per_field[key] = [1, f, label, fmt(mv, size), fmt(pv, size), f]
            elif e[5] != f:
                e[0] += 1
                e[5] = f
            if args.field and fnmatch.fnmatchcase(key, args.field):
                watch.append(f"  {f:5d} {label}: MAME {fmt(mv, size)} port {fmt(pv, size)}")

        # Video: tilemaps and palette (masked), sprite list, sounds.
        pgfx, gmask = rp.images["gfx"], rp.images["gfx_mask"].astype(bool)
        for name, base in LAYERS:
            m = gmask[base : base + 0x4000]
            d = np.nonzero(m & (pgfx[base : base + 0x4000] != mgfx[base : base + 0x4000]))[0]
            if len(d) and allow.rects:
                cells_d = np.unique(d // 4)
                keep = [4 * int(c) + k for c in cells_d
                        if not any(n == name and c0 <= c // 32 <= c1 and r0 <= c % 32 <= r1 for n, c0, c1, r0, r1 in allow.rects)
                        for k in range(4)]
                d = np.array(keep, dtype=d.dtype)
            if len(d) and allow.cells:
                # the cells that an allowed pair explains are not differences
                cells_d = np.unique(d // 4)
                keep = []
                mw = mgfx[base : base + 0x4000].astype(np.uint32)
                pw = pgfx[base : base + 0x4000].astype(np.uint32)
                for c in cells_d:
                    mv = (int(mw[4 * c]) << 24) | (int(mw[4 * c + 1]) << 16) | (int(mw[4 * c + 2]) << 8) | int(mw[4 * c + 3])
                    pv = (int(pw[4 * c]) << 24) | (int(pw[4 * c + 1]) << 16) | (int(pw[4 * c + 2]) << 8) | int(pw[4 * c + 3])
                    if not any(n == name and a == mv and b == pv for n, a, b in allow.cells):
                        keep.extend(range(4 * int(c), 4 * int(c) + 4))
                d = np.array(keep, dtype=d.dtype)
            if len(d):
                cells = sorted(set(int(x) // 4 for x in d))
                if allow.ok(name, f):
                    allowed[name] += 1
                else:
                    c = cells[0]
                    o = base + 4 * c
                    desc = f"{len(cells)} cells, first cell {c:#x} (VRAM {0x900000 + o:06X}): MAME {value(mgfx, RAM + o, 4):08X} port {value(pgfx, RAM + o, 4):08X}"
                    frame_diffs.append((0x1000000 + base, name, desc, ""))
                    v = video[name]
                    v[0] += 1
                    if v[1] is None:
                        v[1], v[2] = f, desc
        d = np.nonzero(gmask[0x14000:0x15000] & (pgfx[0x14000:0x15000] != mgfx[0x14000:0x15000]))[0]
        if len(d):
            words = sorted(set(int(x) // 2 for x in d))
            if allow.ok("palette", f):
                allowed["palette"] += 1
            else:
                w = words[0]
                desc = f"{len(words)} words, first word {w:#x} (bank {w // 512}, palette {(w % 512) // 16}): MAME {value(mgfx, RAM + 0x14000 + 2 * w, 2):04X} port {value(pgfx, RAM + 0x14000 + 2 * w, 2):04X}"
                frame_diffs.append((0x1014000, "palette", desc, ""))
                v = video["palette"]
                v[0] += 1
                if v[1] is None:
                    v[1], v[2] = f, desc
        # the row scroll RAM 0x915800 .. 0x918000 (stage 2: one X offset per screen line, word index = line + 16)
        d = np.nonzero(gmask[0x15800:0x18000] & (pgfx[0x15800:0x18000] != mgfx[0x15800:0x18000]))[0]
        if len(d):
            words = sorted(set(int(x) // 2 for x in d))
            if allow.ok("rowscroll", f):
                allowed["rowscroll"] += 1
            else:
                w = words[0]
                desc = f"{len(words)} words, first word {w:#x} (screen line {w - 16}): MAME {value(mgfx, RAM + 0x15800 + 2 * w, 2):04X} port {value(pgfx, RAM + 0x15800 + 2 * w, 2):04X}"
                frame_diffs.append((0x1015800, "rowscroll", desc, ""))
                v = video["rowscroll"]
                v[0] += 1
                if v[1] is None:
                    v[1], v[2] = f, desc
        sm = T.mame_sprite_list(mram, mgfx)
        sp = T.sprite_entries(rp.images["obj"])
        if f >= 0 and sm != sp:
            src = [(int(rp.images["obj_src"][2 * k]) << 8) | int(rp.images["obj_src"][2 * k + 1]) for k in range(len(sp))] if "obj_src" in rp.images else [0] * len(sp)

            def owner(handle, pram=pram):
                for kind, base, stride, n in pool_list:
                    if base <= handle < base + stride * n and (handle - base) % stride == 0:
                        cls = value(pram, RAM + handle + 0x20, 2) // 4
                        if kind == "player" or cls in ported.get(kind, []):
                            return "ported_same" if handle in diff_handles else "ported"
                        return "unported"
                return "none"

            counts, first = sprite_diff(sm, sp, src, owner)
            if counts.get("mame_only") and f in unported_diff_frames:
                # Entries only MAME has, in a frame where an unported object differs: its shadow
                # or its new / bigger picture (the port holds that object frozen). Not a bug.
                counts["unported"] += counts.pop("mame_only")
                if first is not None and first.startswith("MAME has"):
                    first = None
            if allow.ok("sprites", f):
                allowed["sprites"] += 1
            else:
                v = video["sprites"]
                for cat, n in counts.items():
                    sprite_cats[cat] += n
                    sprite_frames[cat] += 1
                if first is not None:
                    frame_diffs.append((0x1020000, "sprites", f"MAME {len(sm)} entries, port {len(sp)}; {first}", ""))
                    v[0] += 1
                    if v[1] is None:
                        v[1], v[2] = f, f"MAME {len(sm)} entries, port {len(sp)}; {first}"
        if f >= 0:
            cm = [c for c, _, _ in sounds_m]
            cp = list(rp.extra.get("snd", []))
            cpd = [tuple(x) for x in rp.extra.get("snd_d", [])]
            if cm == cp and cpd:
                sound_args[2] += len(cm)
                md = [(d1, d2) for _, d1, d2 in sounds_m]
                if args.snd_table:
                    for c, m, pp in zip(cm, md, cpd):
                        e = snd_tab.setdefault(c, {"m": defaultdict(int), "p": defaultdict(int)})
                        e["m"][m] += 1
                        e["p"][pp] += 1
                if md != cpd:
                    sound_args[0] += 1
                    if sound_args[1] is None:
                        sound_args[1] = (f, "MAME " + " ".join(f"{c:X}:{a:X}/{b:X}" for c, (a, b) in zip(cm, md)) + " port " + " ".join(f"{c:X}:{a:X}/{b:X}" for c, (a, b) in zip(cp, cpd)))
                    for c, (a, b), (pa, pb) in zip(cm, md, cpd):
                        if (a, b) != (pa, pb):
                            sound_codes.setdefault(c, [0, 0])
                            sound_codes[c][0] += 1
                            sound_codes[c][1] = f"MAME {a:X}/{b:X} port {pa:X}/{pb:X}"
            if cm != cp:
                if allow.ok("sounds", f):
                    allowed["sounds"] += 1
                else:
                    desc = "MAME [" + " ".join(f"{c:04X}" for c in cm) + "] port [" + " ".join(f"{c:04X}" for c in cp) + "]"
                    it = iter(cm)
                    if len(cm) > len(cp) and all(c in it for c in cp):
                        # Every request of the port is there; MAME has more: callers the port lacks.
                        sound_kinds["mame_only"][0] += 1
                        if sound_kinds["mame_only"][1] is None:
                            sound_kinds["mame_only"][1] = (f, desc)
                    else:
                        sound_kinds["other"][0] += 1
                        if sound_kinds["other"][1] is None:
                            sound_kinds["other"][1] = (f, desc)
                        frame_diffs.append((0x1030000, "sounds", desc, ""))
                        v = video["sounds"]
                        v[0] += 1
                        if v[1] is None:
                            v[1], v[2] = f, desc

        if f in args.at:
            at_lists[f] = sorted(frame_diffs)
        if frame_diffs and first_frame is None:
            first_frame = f
            first_list = sorted(frame_diffs)

    # ---------------------------------------------------------------- report
    print(f"MAME  {args.mame}: sync vblank {tm.header.get('sync_vblank')}, input {os.path.basename(tm.header.get('input', ''))}")
    print(f"port  {args.port}")
    for w in hdr.get("warnings", []):
        print(f"  load warning: {w}")
    print(f"compared {frames} records (frame -1 = sync state)")
    if lag_frames:
        print(f"MAME lag frames (game frame overran a vblank): {lag_frames[:20]}{' ...' if len(lag_frames) > 20 else ''}")
    if first_frame is None:
        print("\nRESULT: no differences" + (" (some allowed, see below)" if allowed else ""))
    else:
        print(f"\nFIRST DIFFERENCE: frame {first_frame} ({len(first_list)} items)")
        for addr, label, a, b in first_list[: args.first]:
            if b == "" and addr >= 0x1000000:
                print(f"  {label}: {a}")
            else:
                print(f"  {label}: MAME {a}  port {b}")
        if len(first_list) > args.first:
            print(f"  ... {len(first_list) - args.first} more")
    for f, lst in sorted(at_lists.items()):
        print(f"\nDIFFERENCES AT FRAME {f} ({len(lst)} items)")
        for addr, label, a, b in lst:
            print(f"  {label}: {a}" if b == "" and addr >= 0x1000000 else f"  {label}: MAME {a}  port {b}")
    if per_field:
        print(f"\nFIELDS WITH DIFFERENCES ({len(per_field)}): frames differing, first frame, first values")
        for key, e in sorted(per_field.items(), key=lambda kv: (kv[1][1], kv[0])):
            print(f"  {key:32s} {e[0]:5d} frames, first {e[1]:5d}  {e[2]}: MAME {e[3]} port {e[4]}")
    if by_class:
        print("\nDIFFERENCES BY OBJECT CLASS (object-frames with a difference, first frame, fields with first frame)")
        for (kind, cls), bc in sorted(by_class.items(), key=lambda kv: (kv[0][0], kv[0][1])):
            status = "ported" if cls in ported.get(kind, []) else "NOT PORTED"
            if kind == "player":
                status = "ported"
            flds = " ".join(f"{n}@{fr}" for n, fr in sorted(bc["fields"].items(), key=lambda x: (x[1], x[0])))
            slots_txt = ",".join(str(x) for x in sorted(bc["slots"]))
            extra = f" (slot freed in MAME, live in port: {bc['port_only']} fields)" if bc["port_only"] else ""
            print(f"  {kind} class {cls:#04x} handler {handler(kind, cls)} [{status}] slots {slots_txt}: {len(bc['objframes'])} object-frames, first {bc['first']}{extra}")
            print(f"      {flds}")
    vids = [(k, v) for k, v in video.items() if v[0]]
    if sprite_cats:
        print("\nSPRITE LIST DIFFERENCES: entries that differ, by source (the port tags each entry with the object that drew it)")
        print(f"  from unported classes: {sprite_cats['unported']} entries in {sprite_frames['unported']} frames (held frozen, expected)")
        print(f"  from ported classes whose fields differ too (a consequence of that): {sprite_cats['ported_same']} entries in {sprite_frames['ported_same']} frames")
        print(f"  from ported classes whose fields do not differ, or no object (draw bug?): {sprite_cats['ported']} entries in {sprite_frames['ported']} frames")
        print(f"  only in MAME (the port drew fewer): {sprite_cats['mame_only']} entries in {sprite_frames['mame_only']} frames")
    if args.snd_table:
        print("\nSOUND TABLE (code: MAME d1/d2 x count | port d1/d2 x count, only codes with a difference)")
        for c in sorted(snd_tab):
            e = snd_tab[c]
            if all(e["m"][k] == e["p"][k] for k in e["m"]) and set(e["m"]) == set(e["p"]):
                continue
            ms = " ".join(f"{a:X}/{b:X}x{n}" for (a, b), n in sorted(e["m"].items(), key=lambda kv: -kv[1])[:6])
            ps = " ".join(f"{a:X}/{b:X}x{n}" for (a, b), n in sorted(e["p"].items(), key=lambda kv: -kv[1])[:6])
            print(f"  {c:04X}: MAME {ms} | port {ps}")
    for c, (n, ex) in sorted(sound_codes.items()):
        print(f"  sound code {c:04X}: {n} requests with other d1 / d2, e.g. {ex}")
    if sound_args[0] or sound_args[2]:
        fr, desc = sound_args[1] if sound_args[1] else (-1, "")
        print(f"\nSOUND ARGUMENTS (d1, d2) differing where the codes agree: {sound_args[0]} frames of {sound_args[2]} requests compared, first {fr}: {desc}")
    if sound_kinds["mame_only"][0]:
        n, (fr, desc) = sound_kinds["mame_only"]
        print(f"\nSOUND REQUESTS only MAME has (every request of the port is there; callers the port lacks): {n} frames, first {fr}: {desc}")
    if vids:
        print("\nVIDEO AND SOUND: frames differing, first frame")
        for k, v in vids:
            print(f"  {k:10s} {v[0]:5d} frames, first {v[1]:5d}: {v[2]}")
    if allowed:
        print("\nALLOWED (approved differences, counted per field and frame):")
        for k, n in sorted(allowed.items()):
            print(f"  {k:32s} {n}")
    if watch:
        print(f"\nFIELD {args.field}:")
        print("\n".join(watch[:400]))

    # Not modeled: bytes that changed in MAME but the port never traced.
    print("\nNOT MODELED (changes in MAME, no port field; not counted as differences):")
    nm = changed & (index < 0)
    per_kind = defaultdict(lambda: defaultdict(int))
    for o in np.nonzero(nm)[0]:
        sid = int(obj_of[o])
        if sid >= 0:
            kind, i, base = slots[sid]
            per_kind[kind][RAM + int(o) - base] += 1
    for kind, offs in per_kind.items():
        runs = []
        for off in sorted(offs):
            if runs and off == runs[-1][1] + 1:
                runs[-1][1] = off
            else:
                runs.append([off, off])
        txt = ", ".join(f"+${a:02X}" if a == b else f"+${a:02X}..${b:02X}" for a, b in runs)
        print(f"  {kind}: {txt}")
    glob = [int(o) for o in np.nonzero(nm)[0] if obj_of[o] < 0]
    if glob:
        runs = []
        for o in glob:
            if runs and o <= runs[-1][1] + 2:
                runs[-1][1] = o
            else:
                runs.append([o, o])
        names = ", ".join(
            (f"G.${a + RAM - 0xFF8000:X}" if a + RAM >= 0xFF8000 else f"G.-${0xFF8000 - a - RAM:X}") + ("" if a == b else f"..+{b - a + 1}")
            for a, b in runs[:60]
        )
        print(f"  globals: {len(glob)} bytes in {len(runs)} runs: {names}{' ...' if len(runs) > 60 else ''}")
    n_port = n_unp = 0
    for (kind, cls), bc in by_class.items():
        if kind == "player" or cls in ported.get(kind, []):
            n_port += len(bc["fields"])
        else:
            n_unp += len(bc["fields"])
    n_glob = sum(1 for k in per_field if k.startswith("global."))
    print(f"\nSUMMARY: {len(per_field)} fields and {len(vids)} video/sound items differ; first difference at frame {first_frame}")
    print(f"  object fields of ported classes: {n_port} (class, field) pairs; of unported classes: {n_unp}; globals: {n_glob}")
    print("  ported-class and global differences need a cause each; unported-class differences are tasks (the class is not ported)")
    return 0 if first_frame is None else 1


if __name__ == "__main__":
    sys.exit(main())
