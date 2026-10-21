# The list of the plain record.sh recordings for multi.lua (re/harness/coverage/REPORT.md, "Method").
#   re/.venv/bin/python -I re/harness/coverage/scenarios.py OUT.list [TRACES_DIR]
# Reads the header and the snapshot of every MAME recording in TRACES_DIR (default re/out/traces) that record.sh made
# without the batch modes (no probe_segment in the header), and writes one multi.lua line per distinct setup:
#   NAME FRAMES DINO_BOOT=... DINO_INPUT=... DINO_SYNC=vblank:N [DINO_CLOCK=video] [DINO_POKE=...]
# The sync point is the vblank of the original (DINO_SYNC=vblank:N gives the same sync as control or control+N did).
# The boot script follows from the heroes in the snapshot (player objects +$00 and +$03); the poke file is
# pokes/NAME.lua, or the one the script names in its head comment, or the one of the name without the hero suffix, or the
# one of its family (FAMILY below); an epN_*.txt script gets the story panel poke pokes/epN_boot.txt (DINO_POKES), and the
# recordings in SPECIAL get the settings their documents give. NOT_REPLAYED: recordings whose settings are not known
# (their replay differed from the first record on, 2026-10-21).
# Left out: the natural whole runs and the power-on references and their a4NN_ copies (their own jobs), recordings whose input script is not
# in re/harness/scripts, and recordings with the same setup as a longer one (cuts made with trace_prefix.py).
import glob
import json
import os
import re
import struct
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
OUTF = sys.argv[1]
TR = sys.argv[2] if len(sys.argv) > 2 else os.path.join(ROOT, "re/out/traces")
S = os.path.join(ROOT, "re/harness/scripts")
P = os.path.join(ROOT, "re/harness/pokes")
COVERED = re.compile(r"^(a4[0-9][0-9]_)?(ep[1-8]_(jack|3p|mustapha|hannah|mess)|m517_ep1_.*|m523_ep1_.*|attract.*|coinstart.*|m328_power)$")
FAMILY = [(r"^m901_tie_", "tie_jack_hannah"), (r"^m901_tierun_hannah_", "m901_tierun_hannah_hit")]
SPECIAL = {"flow_clock_power": "DINO_SYNC=power DINO_CLOCK=video DINO_POKES=re/harness/pokes/flow_clock_power.txt"}   # re/harness/README.md
NOT_REPLAYED = {"m110_free", "m223_free_wait", "sk_e1", "ep8_row"}
BOOT = {(0,): "boot_jack", (1,): "boot_mustapha", (2,): "boot_hannah", (3,): "boot_mess", (0, 2): "boot_2p",
        (2, 0): "boot_2p_hj", (1, 3): "boot_2p_mm", (3, 1): "boot_2p_mu", (0, 1, 3): "boot_3p", (0, 2, 1): "boot_3p_jhm"}


def header(path):
    with open(path, "rb") as fh:
        if fh.read(8) != b"DINOTRC1":
            return None, None
        n = struct.unpack("<I", fh.read(4))[0]
        return json.loads(fh.read(n)), fh.read()


def ram_of_state(path):
    h, body = header(path)
    if h is None:
        return None
    ram = bytearray(0x10000)
    p = 16
    k = struct.unpack_from("<I", body, p)[0]
    p += 4
    for _ in range(k):
        i = struct.unpack_from("<I", body, p)[0]
        ram[i * 64:i * 64 + 64] = body[p + 4:p + 68]
        p += 68
    return ram


rows = {}
skipped = []
for path in sorted(glob.glob(os.path.join(TR, "*.mame.trc"))):
    name = os.path.basename(path)[:-9]
    try:
        h, _ = header(path)
    except Exception:
        continue
    if h is None or h.get("kind") != "trace" or "probe_segment" in h or "input" not in h:
        continue
    if COVERED.match(name) or name in NOT_REPLAYED:
        continue
    script = os.path.basename(h["input"])
    if not os.path.exists(os.path.join(S, script)):
        skipped.append((name, "no script " + script))
        continue
    state = os.path.join(TR, name + ".state")
    ram = ram_of_state(state) if os.path.exists(state) else None
    if ram is None:
        skipped.append((name, "no state"))
        continue
    clock = h.get("clock", "game")
    heroes = tuple(ram[o + 3] for o in (0xB274, 0xB3F4, 0xB574) if ram[o] != 0)
    if clock == "video":
        boot = "attract"
    elif heroes in BOOT:
        boot = BOOT[heroes]
    else:
        skipped.append((name, "heroes %s" % (heroes,)))
        continue
    stem = re.sub(r"_(jack|hannah|mustapha|mess)$", "", name)
    poke = None
    head = "".join(l for l in open(os.path.join(S, script)).readlines()[:6] if l.startswith("#"))
    m = re.search(r"pokes/([A-Za-z0-9_]+)\.lua", head)
    fam = [f for pat, f in FAMILY if re.match(pat, name)]
    for cand in [name, m.group(1) if m else None, stem, re.sub(r"^a416_", "", stem), script[:-4]] + fam:
        if cand and os.path.exists(os.path.join(P, cand + ".lua")):
            poke = cand
            break
    ep = re.match(r"ep([2-8])_", script)
    extra = SPECIAL.get(name, "DINO_POKES=re/harness/pokes/ep%s_boot.txt" % ep.group(1) if ep else "")
    key = (script, boot, poke, h.get("sync_vblank"), clock, extra)
    if key not in rows or rows[key][1] < h.get("frames", 0):
        rows[key] = (name, h.get("frames", 0))

with open(OUTF, "w") as fh:
    fh.write("# multi.lua list: the plain record.sh recordings of %s (scenarios.py)\n" % os.path.relpath(TR, ROOT))
    for (script, boot, poke, sync, clock, extra), (name, frames) in sorted(rows.items(), key=lambda kv: kv[1][0]):
        line = "%s %d DINO_BOOT=re/harness/scripts/%s.txt DINO_INPUT=re/harness/scripts/%s" % (name, frames, boot, script)
        if "DINO_SYNC=" not in extra:
            line += " DINO_SYNC=vblank:%d" % sync
            if clock == "video":
                line += " DINO_CLOCK=video"
        if extra:
            line += " " + extra
        if poke:
            line += " DINO_POKE=re/harness/pokes/%s.lua" % poke
        fh.write(line + "\n")
print("scenarios.py: %d setups -> %s; left out %d: %s" % (len(rows), OUTF, len(skipped), skipped[:12]))
