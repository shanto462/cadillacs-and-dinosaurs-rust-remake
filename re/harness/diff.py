"""Compare a MAME reference trace with a port trace, frame by frame.

Usage:
    re/.venv/bin/python -I re/harness/diff.py MAME.trc PORT.trc [--allow FILE] [--frames N]
        [--first N] [--from FRAME] [--field KEY]

Report:
  1. the first frame with a difference (not allowed), and every differing field at that frame
     (object handle and kind, field offset and name, MAME value, port value);
  2. a summary per field: frames with a difference, first frame, first values;
  3. video and sound: tilemap cells, palette words, sprite list entries, sound requests;
  4. "not modeled": bytes that change in MAME but that the port does not trace (per object kind
     and offset, and the busiest global addresses). These are not counted as differences.

Frame -1 is the sync state: MAME's snapshot against the port right after --load-state. A
difference there means the loader missed a field.

Allow file (one rule per line, '#' starts a comment): a key pattern (fnmatch) and an optional
frame range, e.g.
    enemy.*            # pool A classes 0x36 / 0x37 (cut-scene thugs) are not ported
    global.rng 0-120   # ...
Keys: "<kind>.<field>" for objects (kinds: player enemy prop item effect special), "global.<name>",
"scroll1", "scroll2", "scroll3", "palette", "sprites", "sounds".
"""

import argparse
import fnmatch
import os
import sys
from collections import defaultdict

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trace as T  # noqa: E402

RAM = 0xFF0000


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
        if not path:
            return
        with open(path) as f:
            for line in f:
                line = line.split("#")[0].strip()
                if not line:
                    continue
                parts = line.split()
                lo, hi = -1, 1 << 30
                if len(parts) > 1:
                    a, _, b = parts[1].partition("-")
                    lo = int(a)
                    hi = int(b) if b else lo
                self.rules.append((parts[0], lo, hi))

    def ok(self, key, frame):
        return any(fnmatch.fnmatchcase(key, p) and lo <= frame <= hi for p, lo, hi in self.rules)


LAYERS = [("scroll1", 0x8000), ("scroll2", 0xC000), ("scroll3", 0x10000)]


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("mame")
    ap.add_argument("port")
    ap.add_argument("--allow", help="allow file of approved differences")
    ap.add_argument("--frames", type=int, default=1 << 30, help="compare at most N frames")
    ap.add_argument("--from", dest="start", type=int, default=-1, help="first frame to compare")
    ap.add_argument("--first", type=int, default=40, help="fields listed for the first differing frame")
    ap.add_argument("--field", help="print this field (key pattern) every frame it differs")
    args = ap.parse_args()

    tm, tp = T.Trace(args.mame), T.Trace(args.port)
    if tp.source != "port":
        tm, tp = tp, tm
    hdr = tp.header
    fields, index, obj_of, slots = build_fields(hdr)
    strides = {p["kind"]: p["stride"] for p in hdr["pools"]}
    allow = Allow(args.allow)

    per_field = {}  # key -> [frames, first frame, label, mame, port]
    first_frame = None
    first_list = []
    allowed = defaultdict(int)
    video = {k: [0, None, None] for k in ("scroll1", "scroll2", "scroll3", "palette", "sprites", "sounds")}
    changed = np.zeros(0x10000, dtype=bool)
    lag_frames = []
    prev_ram = None
    frames = 0
    watch = []

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
        sm = T.mame_sprite_list(mram, mgfx)
        sp = T.sprite_entries(rp.images["obj"])
        if f >= 0 and sm != sp:
            if allow.ok("sprites", f):
                allowed["sprites"] += 1
            else:
                k = next((i for i in range(min(len(sm), len(sp))) if sm[i] != sp[i]), min(len(sm), len(sp)))
                em = sm[k] if k < len(sm) else None
                ep = sp[k] if k < len(sp) else None
                h = lambda e: "end" if e is None else "x %04X y %04X code %04X attr %04X" % e  # noqa: E731
                desc = f"MAME {len(sm)} entries, port {len(sp)}; first difference at entry {k}: MAME {h(em)} | port {h(ep)}"
                frame_diffs.append((0x1020000, "sprites", desc, ""))
                v = video["sprites"]
                v[0] += 1
                if v[1] is None:
                    v[1], v[2] = f, desc
        if f >= 0:
            cm = [c for c, _, _ in sounds_m]
            cp = list(rp.extra.get("snd", []))
            if cm != cp:
                if allow.ok("sounds", f):
                    allowed["sounds"] += 1
                else:
                    desc = "MAME [" + " ".join(f"{c:04X}" for c in cm) + "] port [" + " ".join(f"{c:04X}" for c in cp) + "]"
                    frame_diffs.append((0x1030000, "sounds", desc, ""))
                    v = video["sounds"]
                    v[0] += 1
                    if v[1] is None:
                        v[1], v[2] = f, desc

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
    if per_field:
        print(f"\nFIELDS WITH DIFFERENCES ({len(per_field)}): frames differing, first frame, first values")
        for key, e in sorted(per_field.items(), key=lambda kv: (kv[1][1], kv[0])):
            print(f"  {key:32s} {e[0]:5d} frames, first {e[1]:5d}  {e[2]}: MAME {e[3]} port {e[4]}")
    vids = [(k, v) for k, v in video.items() if v[0]]
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
    print(f"\nSUMMARY: {len(per_field)} fields and {len(vids)} video/sound items differ; first difference at frame {first_frame}")
    return 0 if first_frame is None else 1


if __name__ == "__main__":
    sys.exit(main())
