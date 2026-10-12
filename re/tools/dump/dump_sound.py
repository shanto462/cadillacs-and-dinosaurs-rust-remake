"""Sound, decoded from the ROM data only (no emulator):
  - the decrypted Z80 program (opcode and data images) and its disassembly,
  - the sound command table (code -> music or effect header),
  - the QSound sample table: every sample as a WAV (8 bit signed PCM at 24038 Hz, the rate of one pitch step),
    with the instruments and sound codes that use it,
  - every music track and effect as a readable sequence listing: voices, instruments, notes, length in steps.
Music and composed effects as audio come later from our own sound driver (milestone M4). Output: re/out/assets/sound/."""
import io
import json
import os
import sys
import wave

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from romio import Rom, outdir, hx, RE
from owners import Owners
import sequences as sq

SAMPLE_TABLE = 0x6A82
INSTR_TABLE = 0x6E62


def code_table(rom):
    z = rom.z80_data
    n = (z[0x8000] << 8) | z[0x8001]
    rows = []
    for code in range(n):
        e = z[0x8006 + 4 * code:0x8006 + 4 * code + 4]
        ptr = (e[0] << 16) | (e[1] << 8) | e[2]
        if ptr == 0:
            rows.append(dict(code=code, ptr=0, kind="none"))
            continue
        h = z[ptr]
        kind = "music" if h == 0 else ("sfx" if h < 0x80 else "adpcm")
        voices = [(z[ptr + 1 + 2 * v] << 8) | z[ptr + 2 + 2 * v] for v in range(16)]
        rows.append(dict(code=code, ptr=ptr, header=h, kind=kind, priority=h if kind == "sfx" else None,
                         voices=[v for v, o in enumerate(voices) if o], n_voices=sum(1 for o in voices if o)))
    return rows


def samples(rom):
    z = rom.z80_data
    pcm = b"".join(rom.qs)
    n = (INSTR_TABLE - SAMPLE_TABLE) // 8
    out = []
    for i in range(n):
        e = z[SAMPLE_TABLE + 8 * i:SAMPLE_TABLE + 8 * i + 8]
        bank, start, loop, end, key = e[0], e[1] | (e[2] << 8), e[3] | (e[4] << 8), e[5] | (e[6] << 8), e[7]
        a, b = bank * 0x10000 + start, bank * 0x10000 + end
        out.append(dict(index=i, bank=bank, start=start, loop=loop, end=end, key=key, length=max(0, end - start),
                        rom_offset=a, pcm=pcm[a:b]))
    return out


def instrument(rom, i):
    e = rom.z80_data[INSTR_TABLE + 8 * i:INSTR_TABLE + 8 * i + 8]
    return dict(index=i, sample=e[0], fine=e[1], env=e[4], raw=e.hex(" "))


def write_wav(path, data_i16, rate, channels=1):
    with wave.open(path, "wb") as w:
        w.setnchannels(channels)
        w.setsampwidth(2)
        w.setframerate(rate)
        w.writeframes(np.asarray(data_i16, np.int16).tobytes())


def caller_scan(rom, own):
    """Sound codes passed to the enqueue routine 0xA63C: [(code, site address, owner label)]."""
    b = rom.b
    sites = []
    for x in range(0x1000, 0xB7000, 2):
        w = (b[x] << 8) | b[x + 1]
        if w in (0x4EB9, 0x4EF9) and rom.u32(x + 2) == 0xA63C:
            sites.append(x)
        elif w in (0x6100, 0x6000) and x + 2 + rom.i16(x + 2) == 0xA63C:
            sites.append(x)
    out = []
    for s in sites:
        code = None
        for back in range(2, 60, 2):
            y = s - back
            wy = (b[y] << 8) | b[y + 1]
            if wy == 0x303C:
                code = rom.u16(y + 2)
                break
            if (wy & 0xFF00) == 0x7000 and (wy & 0x0E00) == 0:       # moveq #imm,d0
                code = wy & 0xFF
                break
            if wy in (0x4E75, 0x4EB9):
                break
        c = own.class_at(s)
        out.append((code, s, own.class_label(*c) if c else f"code at {s:#x}"))
    return out


def music_uses(rom):
    """Music codes of the stage tables: 0xA582 (stage*4+area) and the boss table 0xA5C8."""
    uses = {}
    for stage in range(8):
        for area in range(4):
            c = rom.u8(0xA582 + stage * 4 + area)
            if c:
                uses.setdefault(c, []).append(f"Episode {stage + 1} area {area}")
    for stage in range(8):
        c = rom.u8(0xA5C8 + stage)
        uses.setdefault(c, []).append(f"boss of Episode {stage + 1}")
    uses.setdefault(rom.u8(0xA5C8 + 8), []).append("final boss, second form")
    uses.setdefault(1, []).append("start of Episode 1")
    return uses


def run(rom=None):
    rom = rom or Rom()
    out = outdir("sound")
    own = Owners(rom)
    z = rom.z80_data
    # ---- Z80 program
    open(os.path.join(out, "z80_opcode_image.bin"), "wb").write(rom.z80_op)
    open(os.path.join(out, "z80_data_image.bin"), "wb").write(rom.z80_data)
    for name, data in (("z80_op.bin", rom.z80_op), ("z80_data.bin", rom.z80_data)):
        p = os.path.join(RE, "out", name)
        if not os.path.exists(p):
            open(p, "wb").write(data)
    try:
        sys.path.insert(0, os.path.join(RE, "specs", "sound"))
        import walkall
        buf = io.StringIO()
        walkall.z.listing(walkall.seen, walkall.labels, out=buf)
        with open(os.path.join(out, "z80_listing.txt"), "w") as f:
            f.write("; Z80 sound driver disassembly (recursive walk, Kabuki decrypted). Opcodes from the opcode image, operands from the data image.\n")
            f.write(buf.getvalue())
    except Exception as e:  # keep going
        print("z80 listing failed:", e)
    # ---- code table, who sends each code
    rows = code_table(rom)
    callers = caller_scan(rom, own)
    by_code = {}
    for code, site, lab in callers:
        if code is not None:
            by_code.setdefault(code, []).append((site, lab))
    mus = music_uses(rom)
    hit_ids = {}
    for k in range(12):
        a = 0x96AA + 3 * k
        for c in (rom.u8(a + 1), rom.u8(a + 2)):
            if c:
                hit_ids.setdefault(c, []).append(f"hit sound id {k} (routine 0x962A, alternates with the other code of the pair)")
    for r in rows:
        r["callers"] = sorted({lab for s, lab in by_code.get(r["code"], [])})
        r["caller_sites"] = [s for s, lab in by_code.get(r["code"], [])][:8]
        r["hit_ids"] = hit_ids.get(r["code"], [])
        if r["kind"] == "music":
            r["used_by"] = mus.get(r["code"], [])
    # ---- samples
    sdir = outdir("sound", "samples")
    sam = samples(rom)
    # ---- sequences
    instr_cache = {}

    def instr(i):
        if i not in instr_cache:
            instr_cache[i] = instrument(rom, i)
        return instr_cache[i]

    sample_users = {s["index"]: dict(instruments=set(), codes=set()) for s in sam}
    seq_lines = ["# Sound sequences decoded from the Z80 data (no emulator). Time unit: sequencer step = 1/62.5 s.",
                 "# Notes: name:length (length units 3 = 1/32 note ... 192 = whole note; r = rest; . = dotted, t = triplet, ~ = tied).",
                 "# Note names are relative (pitch number 1 = c): the real pitch depends on the root key of the instrument sample.", ""]
    for r in rows:
        if not r["ptr"]:
            continue
        if r["kind"] == "adpcm":
            r["length_s"] = None
            continue
        res = sq.decode_code(z, r["ptr"], None, max_steps=15000.0)
        tr = res["tracks"]
        used_instr = sorted({i for t in tr for i in t.instruments})
        used_samples = sorted({instr(i)["sample"] for i in used_instr})
        for i in used_instr:
            s = instr(i)["sample"]
            if s in sample_users:
                sample_users[s]["instruments"].add(i)
                sample_users[s]["codes"].add(r["code"])
        ends = []
        for t in tr:
            if t.loop_len is not None:
                ends.append(t.intro + t.loop_len)
            else:
                ends.append(t.end_time if t.end_time is not None else t.t)
        longest = max(ends) if ends else 0
        loops = any(t.loop_len is not None for t in tr)
        r.update(instruments=used_instr, samples=used_samples, length_steps=longest, length_s=longest / sq.STEPS_PER_SECOND, loops=loops,
                 loop_s=max((t.loop_len for t in tr if t.loop_len is not None), default=0) / sq.STEPS_PER_SECOND,
                 notes=sum(t.notes for t in tr))
        hdr = f"== code {r['code']:#05x}  {r['kind']}" + (f" priority {r['priority']}" if r["priority"] is not None else "") + f"  header @{r['ptr']:06X}  {len(tr)} voices"
        if r["kind"] == "music" and r.get("used_by"):
            hdr += "  used by: " + ", ".join(r["used_by"])
        elif r["callers"] or r["hit_ids"]:
            hdr += "  sent by: " + ", ".join((r["hit_ids"] + r["callers"])[:6])
        seq_lines.append(hdr)
        seq_lines.append(f"   length {longest / sq.STEPS_PER_SECOND:.2f} s ({longest:.0f} steps){'  LOOPS (longest loop %.2f s)' % r['loop_s'] if loops else ''};  {r['notes']} notes;  instruments {used_instr};  samples {used_samples}")
        seq_lines += sq.fmt_tracks(res, lambda i: instr(i)["sample"])
        seq_lines.append("")
    open(os.path.join(out, "sound_sequences.txt"), "w").write("\n".join(seq_lines) + "\n")
    # ---- sample files and table
    meta_s = []
    for s in sam:
        pcm = np.frombuffer(s["pcm"], np.int8).astype(np.int16) * 256
        if len(pcm):
            write_wav(os.path.join(sdir, f"sample_{s['index']:03d}.wav"), pcm, 24038)
        u = sample_users[s["index"]]
        d = {k: v for k, v in s.items() if k != "pcm"}
        d["instruments"] = sorted(u["instruments"])
        d["codes"] = sorted(u["codes"])
        d["loops"] = s["loop"] < s["end"] and s["loop"] > s["start"] - 1 and (s["end"] - s["loop"]) > 16
        d["seconds"] = s["length"] / 24038.0
        meta_s.append(d)
    with open(os.path.join(out, "sample_table.txt"), "w") as f:
        f.write("# QSound sample table at Z80 0x6A82 (data image), 8 bytes per sample: bank, start, loop start, end, root key.\n"
                "# PCM address in the sample ROMs (cd-q1..q4 concatenated, 2 MB) = bank * 0x10000 + start. WAV: samples/sample_NNN.wav,\n"
                "# 8 bit signed PCM at 24038 Hz (pitch register 0x1000 = natural speed, played at the root key). The loop is not applied.\n"
                "# columns: index, bank, start, loop start, end, root key, bytes, seconds, ROM offset, instruments (table 0x6E62) that use it, codes that use those instruments\n")
        for s in meta_s:
            f.write(f"{s['index']:3d}  bank {s['bank']:2d}  start {s['start']:5d}  loop {s['loop']:5d}  end {s['end']:5d}  key {s['key']:3d}  {s['length']:6d} bytes  {s['seconds']:.2f}s  "
                    f"@{s['rom_offset']:06X}  instruments {s['instruments']}  codes {[hex(c) for c in s['codes']]}\n")
    with open(os.path.join(out, "instrument_table.txt"), "w") as f:
        f.write("# Instrument table at Z80 0x6E62 (8 bytes): sample index, fine tune, ?, ?, envelope index, ?, ?, ?  (only instruments used by a sequence are listed)\n")
        used = sorted(instr_cache)
        for i in used:
            e = instr_cache[i]
            f.write(f"{i:3d}  sample {e['sample']:3d}  fine {e['fine']:3d}  env {e['env']:#04x}  raw {e['raw']}\n")
    # ---- summary tables
    with open(os.path.join(out, "sound_codes.json"), "w") as f:
        json.dump(rows, f)
    with open(os.path.join(out, "sound_codes.txt"), "w") as f:
        f.write("# Sound command table of the Z80 driver (re/specs/sound/SPEC.md section 5). code, header address, type, priority, voices, length, who sends it\n"
                "# Full sequences: sound_sequences.txt. Audio of music and composed effects comes with our own sound driver (milestone M4); the raw samples are in samples/.\n")
        for r in rows:
            if not r["ptr"]:
                f.write(f"{r['code']:03X}  -\n")
                continue
            ln = r.get("length_s")
            f.write(f"{r['code']:03X}  hdr {r['ptr']:06X}  {r['kind']:5s}  prio {r['priority'] if r['priority'] is not None else '-':>2}  voices {r['n_voices']:2d}  "
                    f"{'%6.2f s' % ln if ln is not None else '   -   '}{' loops' if r.get('loops') else '      '}  samples {r.get('samples', [])}  | "
                    f"{', '.join(r.get('used_by', []) + r.get('hit_ids', []) + r.get('callers', []))}\n")
    with open(os.path.join(out, "samples.json"), "w") as f:
        json.dump(meta_s, f)
    print(f"sound: {sum(1 for r in rows if r['ptr'])} codes ({sum(1 for r in rows if r['kind'] == 'music')} music), {len(meta_s)} samples, {len(instr_cache)} instruments used")
    return rows


if __name__ == "__main__":
    run()
