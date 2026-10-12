"""OPTIONAL step, not part of the default dump (the user asked to keep MAME use to a minimum).
Captures the screens outside the stages from MAME in ONE process (soft reset between the recipes): boot, legal, title, profiles, ranking, demo, story slides, opening movie,
select, story panels and episode cards of Episodes 2 to 8, clear tally, ending, continue, game over, time over.
Each recipe is a power-on run with scripted input and RAM pokes (see plan/game/NEW-PLAN.md section 5). Output: re/out/assets/frontend/."""
import json
import os
import shutil
import subprocess
import sys
import time

from PIL import Image, ImageDraw

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from romio import outdir, RE, ROOT
from render import font

LUA = os.path.join(os.path.dirname(os.path.abspath(__file__)), "mame", "shots.lua")
START = "600-607:C\n700-705:S\n900-905:A\n"
PANEL_POKES = "1300 FF84D9 1 {stage}\n1300 FF84D1 1 0\n1300 FF84D0 1 8\n"

SCENARIOS = {
    "attract": dict(title="Power on and attract loop (no input): boot, legal, title, profiles, ranking, demo 1, story slides, demo 2, opening movie",
                    input="", pokes="", snaps="every:30:0:15700", frames=15700),
    "start": dict(title="Coin, start, character select, Episode 1 opening scene", input=START, pokes="", snaps="every:20:600:1800", frames=1800),
    "tally": dict(title="Stage clear tally (poked at frame 1300)", input=START,
                  pokes="1300 FF84D1 1 4\n1300 FF8508 2 0\n1300 FF84D5 1 1\n1300 FF84D4 1 1\n", snaps="every:20:1301:2150", frames=2150),
    "ending": dict(title="Ending, staff roll and ranking (poked at frame 1300)", input=START,
                   pokes="1300 FF84D9 1 7\n1300 FF84D7 1 1\n1300 FF84DB 1 1\n1300 FF8507 1 1\n1300 FF84D1 1 0\n1300 FF84D0 1 0x1C\n1300 FF84D1 1 4\n",
                   snaps="every:30:1301:6800", frames=6800),
    "continue": dict(title="Continue screen and game over (lives and health poked to 0 at frame 1500)", input=START,
                     pokes="1500 FFB317 1 0\n1500 FFB2DE 2 0\n1500 FFB2E0 2 0\n1500 FFB2E2 2 0\n", snaps="every:30:1500:3600", frames=3600),
    "timeover": dict(title="Clock and TIME OVER (time poked to 13 seconds at frame 1300)", input=START,
                     pokes="1300 FF84E8 2 0x0013\n", snaps="every:20:1300:2100", frames=2100),
}
for n in range(1, 8):
    SCENARIOS[f"episode{n + 1}_card"] = dict(
        title=f"Link scenes before Episode {n + 1}: story panel, map panel, episode card, first seconds (stage poked at frame 1300)", input=START,
        pokes=PANEL_POKES.format(stage=n), snaps="every:20:1301:2300", frames=2300)


def run_all(names, work):
    """ONE MAME process for all scenarios (tools/dump/mame/shots.lua soft-resets between them).
    Dummy SDL drivers: no Dock icon, no focus change on macOS."""
    shutil.rmtree(work, ignore_errors=True)
    for sub in ("cfg", "nvram", "snap", "diff", "scn"):
        os.makedirs(os.path.join(work, sub))
    lines = []
    for n in names:
        sc = SCENARIOS[n]
        inp = os.path.join(work, "scn", n + ".input.txt")
        pk = os.path.join(work, "scn", n + ".pokes.txt")
        open(inp, "w").write(sc["input"])
        open(pk, "w").write(sc["pokes"])
        lines.append(f"{n}|{inp}|{pk}|{sc['snaps']}|{sc['frames']}")
    manifest = os.path.join(work, "manifest.txt")
    open(manifest, "w").write("\n".join(lines) + "\n")
    env = dict(os.environ, SH_MANIFEST=manifest, SH_LOG=os.path.join(work, "log.txt"), SDL_VIDEODRIVER="dummy", SDL_AUDIODRIVER="dummy")
    cmd = ["mame", "dino", "-rompath", f"{ROOT};{os.path.join(RE, 'mame', 'roms')}", "-video", "none", "-sound", "none", "-nothrottle",
           "-cfg_directory", os.path.join(work, "cfg"), "-nvram_directory", os.path.join(work, "nvram"), "-snapshot_directory", os.path.join(work, "snap"),
           "-diff_directory", os.path.join(work, "diff"), "-autoboot_script", LUA]
    subprocess.run(cmd, cwd=ROOT, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=3600)


def collect_all(names, work, out):
    """Splits the one log and the numbered snapshots of the single run into scenarios."""
    snapdir = os.path.join(work, "snap", "dino")
    files = sorted(os.listdir(snapdir)) if os.path.isdir(snapdir) else []
    per = {}
    cur = None
    for line in open(os.path.join(work, "log.txt")):
        if line.startswith("SCENARIO"):
            cur = line.split()[1]
            per[cur] = dict(frames=[], modes=[])
        elif cur and "SNAP" in line:
            per[cur]["frames"].append(int(line.split()[0][2:]))
        elif cur and "mode=" in line:
            per[cur]["modes"].append((int(line.split()[0][2:]), line.split(" ", 1)[1].strip()))
    meta, k = [], 0
    for n in names:
        sc = SCENARIOS[n]
        info = per.get(n, dict(frames=[], modes=[]))
        sdir = os.path.join(out, n)
        shutil.rmtree(sdir, ignore_errors=True)
        os.makedirs(sdir)
        items = []
        for fr in info["frames"]:
            if k >= len(files):
                break
            dst = f"f{fr:05d}.png"
            shutil.copy(os.path.join(snapdir, files[k]), os.path.join(sdir, dst))
            k += 1
            items.append(dict(frame=fr, file=f"{n}/{dst}"))
        tw, th, per_sheet = 192, 112, 30
        sheets = []
        for s0 in range(0, len(items), per_sheet):
            chunk = items[s0:s0 + per_sheet]
            cols = 5
            rows = (len(chunk) + cols - 1) // cols
            sheet = Image.new("RGB", (cols * (tw + 4), rows * (th + 16)), (24, 24, 32))
            dr = ImageDraw.Draw(sheet)
            for i, it in enumerate(chunk):
                r, c = divmod(i, cols)
                im = Image.open(os.path.join(out, it["file"])).convert("RGB").resize((tw, th), Image.LANCZOS)
                sheet.paste(im, (c * (tw + 4), r * (th + 16) + 14))
                dr.text((c * (tw + 4) + 2, r * (th + 16) + 1), f"frame {it['frame']}", fill=(255, 255, 160), font=font(10))
            fn = f"{n}_sheet_{s0 // per_sheet + 1:02d}.png"
            sheet.save(os.path.join(out, fn))
            sheets.append(fn)
        meta.append(dict(name=n, title=sc["title"], frames=items, sheets=sheets, modes=info["modes"]))
    return meta


def index_existing():
    """Rebuilds data/frontend.json from the captures that are already in re/out/assets/frontend (no emulator)."""
    out = outdir("frontend")
    meta = []
    for n, sc in SCENARIOS.items():
        d = os.path.join(out, n)
        if not os.path.isdir(d):
            continue
        frames = sorted(f for f in os.listdir(d) if f.startswith("f") and f.endswith(".png"))
        sheets = sorted(f for f in os.listdir(out) if f.startswith(n + "_sheet_"))
        meta.append(dict(name=n, title=sc["title"], frames=[dict(frame=int(f[1:6]), file=f"{n}/{f}") for f in frames], sheets=sheets, modes=[]))
    with open(os.path.join(outdir("data"), "frontend.json"), "w") as f:
        json.dump(meta, f)
    return meta


def run(only=None):
    out = outdir("frontend")
    work = outdir("frontend", "_work")
    names = [n for n in SCENARIOS if not only or n in only]
    t0 = time.time()
    print(f"frontend: one MAME process for {len(names)} scenarios ...", flush=True)
    run_all(names, work)
    meta = collect_all(names, work, out)
    shutil.rmtree(work, ignore_errors=True)
    with open(os.path.join(outdir("data"), "frontend.json"), "w") as f:
        json.dump(meta, f)
    print(f"frontend: {sum(len(m['frames']) for m in meta)} screenshots in {len(meta)} scenarios, {time.time() - t0:.0f}s")
    return meta


if __name__ == "__main__":
    run(sys.argv[1:] or None)
