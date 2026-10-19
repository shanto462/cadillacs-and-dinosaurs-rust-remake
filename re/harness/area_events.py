"""M3-29: the timeline of an area change in a MAME trace and in a port trace of the same scenario (re/harness/scripts/m328_areas.txt).

Usage: re/.venv/bin/python -I re/harness/area_events.py NAME [NAME ...]     (re/out/traces/NAME.mame.trc and NAME.port.trc, port run in `run` mode with
       DINO_GAME_CLOCK=1: `DINO_GAME_CLOCK=1 MODE=run re/harness/port.sh re/out/traces/NAME.input.txt NAME FRAMES`)

For each trace the records of the change: E- the last record before the area load in which the camera entered the end mode (G.$741 = 1, 0x12 or 6 with sub 0,
that is the lock released and the fade or the hard cut started), E0 the area load (G.$786 changes), the clock reload (G.$4E8 / G.$4E9 change), E+ the control
release (G.$4DD back to 0), and the records the game frame counter G.$0 jumps in (the original's blocking area start). The port's records are the ticks of the game
clock run, so the intervals are comparable; the first column is the record number. Prints one line per trace and the differences of the intervals.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import trace as T  # noqa: E402

TRACES = os.path.join(HERE, "..", "out", "traces")
# the port traces: PORT_OUT (default re/out/port), see port.sh
PORT = os.environ.get("PORT_OUT") or os.path.join(HERE, "..", "out", "port")


def events(path, limit=None):
    t = T.Trace(path)
    area0 = None
    last_end = None
    ev = {"lag": []}
    prev_g0 = None
    clock0 = None
    n = 0
    for r in t.records():
        if limit is not None and r.frame >= limit:
            break
        ram = r.images["ram"]
        f = r.frame
        g0 = T.u16(ram, 0xFF8000)
        if prev_g0 is not None and ((g0 - prev_g0) & 0xFFFF) > 1:
            ev["lag"].append((f, (g0 - prev_g0) & 0xFFFF))
        prev_g0 = g0
        area, mode, sub, scr = ram[0xFF8786 - 0xFF0000], ram[0xFF8741 - 0xFF0000], ram[0xFF8742 - 0xFF0000], ram[0xFF84DD - 0xFF0000]
        clock = (ram[0xFF84E8 - 0xFF0000], ram[0xFF84E9 - 0xFF0000])
        if area0 is None:
            area0, clock0 = area, clock
        if "E-" not in ev and mode in (1, 0x12, 6, 7, 8) and sub == 4 and "E0" not in ev:
            last_end = f
        if "E0" not in ev and area != area0:
            ev["E0"] = f
            ev["E-"] = last_end
        if "E0" in ev and "clock" not in ev and clock != clock0:
            ev["clock"] = f
        if "E0" in ev and "E+" not in ev and f > ev["E0"] and scr == 0 and mode in (0, 4, 5, 0xC, 0xB):
            ev["E+"] = f
        n += 1
    return ev


def show(name, ev):
    def d(a, b):
        return "-" if ev.get(a) is None or ev.get(b) is None else str(ev[b] - ev[a])
    lag = [x for x in ev["lag"] if ev.get("E0") is not None and ev["E0"] - 5 <= x[0] <= ev["E0"] + 700]
    print(f"{name:34s} E-={ev.get('E-')} E0={ev.get('E0')} clock={ev.get('clock')} E+={ev.get('E+')}  E-..E0 {d('E-', 'E0')}  E0..E+ {d('E0', 'E+')}  "
          f"E0..clock {d('E0', 'clock')}  G.$0 jumps near it {lag}")


for name in sys.argv[1:]:
    m = events(os.path.join(TRACES, name + ".mame.trc"))
    p = events(os.path.join(PORT, name + ".port.trc"))
    show(name + " MAME", m)
    show(name + " port", p)
