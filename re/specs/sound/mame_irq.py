"""MAME check for the sound spec (M4-00): the Z80 interrupt rate and the attract-mode mute of the 68000 flush.

One headless MAME process (command line `mame`, `-video none -sound none`, SDL dummy drivers, never a window).
A Lua frame hook reads the Z80 work RAM of the QSound CPU once per video frame:
  F004 = interrupt counter (the IRQ routine at 0x0038 adds 1 per interrupt), F005 = ticks the main loop has handled,
  F007 = write index of the 16 entry mailbox ring at F100 (the IRQ copies one 68000 request per tick).
From them it computes the IRQ rate in Hz (counter change / emulated seconds) and lists every request that the
68000 flush let through. Two pokes show what the attract mute of the flush (0x610) depends on: at POKE_AT bit 5 of G.$777C (0xFFF77C,
"demo sound" in the cabinet settings) is set; at POKE2_AT that bit is cleared again and G.$4CC (0xFF84CC, the
in-game flag) is set to 1. The log shows which requests pass in each of the three phases.

usage: python3 -I re/specs/sound/mame_irq.py [--frames N] [--poke-at N] [--poke2-at N]   (run from anywhere)
       python3 -I re/specs/sound/mame_irq.py --report LOGFILE
Output (git-ignored): re/out/sound/mame_irq.log, mame_irq.lua, mame/ (MAME's own files).
Needs MAME 0.289, dino.zip in the repo root and the placeholder ROMs in re/mame/roms (see re/harness/README.md).
"""
import os
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
OUT = os.path.join(ROOT, "re", "out", "sound")

LUA = r'''
local audio = manager.machine.devices[":audiocpu"].spaces["program"]
local main = manager.machine.devices[":maincpu"].spaces["program"]
local frames = tonumber(os.getenv("SQ_FRAMES"))
local poke_at = tonumber(os.getenv("SQ_POKE_AT"))
local poke2_at = tonumber(os.getenv("SQ_POKE2_AT"))
local f = io.open(os.getenv("SQ_OUT"), "w")
local frame = 0
local prev_wp = nil
emu.register_frame_done(function()
  frame = frame + 1
  local t = manager.machine.time:as_double()
  local irq = audio:read_u8(0xF004)
  local proc = audio:read_u8(0xF005)
  local wp = audio:read_u8(0xF007)
  if frame == poke_at then
    main:write_u8(0xFFF77C, main:read_u8(0xFFF77C) | 0x20)
    f:write(string.format("POKE %d\n", frame))
  end
  if frame == poke2_at then
    main:write_u8(0xFFF77C, main:read_u8(0xFFF77C) & 0xDF)
    main:write_u8(0xFF84CC, 1)
    f:write(string.format("POKE2 %d\n", frame))
  end
  f:write(string.format("F %d %.6f %d %d %d\n", frame, t, irq, proc, wp))
  if prev_wp ~= nil then
    local p = prev_wp
    while p ~= wp do
      local s = {}
      for i = 0, 15 do s[#s + 1] = string.format("%02x", audio:read_u8(0xF100 + p + i)) end
      f:write(string.format("R %d %s\n", frame, table.concat(s, " ")))
      p = (p + 0x10) & 0xFF
    end
  end
  prev_wp = wp
  if frame >= frames then
    f:close()
    manager.machine:exit()
  end
end)
'''


def main():
    args = sys.argv[1:]
    frames, poke_at, poke2_at = 16000, 6000, 11000
    if "--frames" in args:
        frames = int(args[args.index("--frames") + 1])
    if "--poke-at" in args:
        poke_at = int(args[args.index("--poke-at") + 1])
    if "--poke2-at" in args:
        poke2_at = int(args[args.index("--poke2-at") + 1])
    os.makedirs(OUT, exist_ok=True)
    mdir = os.path.join(OUT, "mame")
    shutil.rmtree(os.path.join(mdir, "nvram"), ignore_errors=True)   # fresh EEPROM, same as the harness
    lua = os.path.join(OUT, "mame_irq.lua")
    log = os.path.join(OUT, "mame_irq.log")
    open(lua, "w").write(LUA)
    env = dict(os.environ, SDL_VIDEODRIVER="dummy", SDL_AUDIODRIVER="dummy",
               SQ_FRAMES=str(frames), SQ_POKE_AT=str(poke_at), SQ_POKE2_AT=str(poke2_at), SQ_OUT=log)
    cmd = ["mame", "dino", "-rompath", ".;re/mame/roms", "-video", "none", "-sound", "none", "-nothrottle",
           "-keyboardprovider", "none", "-mouseprovider", "none", "-joystickprovider", "none",
           "-lightgunprovider", "none",
           "-cfg_directory", mdir + "/cfg", "-nvram_directory", mdir + "/nvram",
           "-snapshot_directory", mdir + "/snap", "-diff_directory", mdir + "/diff",
           "-autoboot_script", lua]
    subprocess.run(cmd, cwd=ROOT, env=env, stdout=open(os.path.join(OUT, "mame_irq.out"), "w"),
                   stderr=subprocess.STDOUT, timeout=900)
    report(log)


def report(log):
    rows, reqs, poke, poke2 = [], [], None, None
    for line in open(log):
        p = line.split()
        if p[0] == "F":
            rows.append((int(p[1]), float(p[2]), int(p[3]), int(p[4])))
        elif p[0] == "R":
            reqs.append((int(p[1]), bytes(int(x, 16) for x in p[2:])))
        elif p[0] == "POKE":
            poke = int(p[1])
        elif p[0] == "POKE2":
            poke2 = int(p[1])
    skip = 120                                  # skip the boot frames
    sub = rows[skip:]
    # the 8 bit counters advance about 4 per video frame, so one wrap between two reads cannot be missed
    irq = sum((b[2] - a[2]) & 0xFF for a, b in zip(sub, sub[1:]))
    proc = sum((b[3] - a[3]) & 0xFF for a, b in zip(sub, sub[1:]))
    secs = sub[-1][1] - sub[0][1]
    print(f"frames {len(rows)}  emulated seconds (from frame {skip}) {secs:.3f}  video rate {(len(sub) - 1) / secs:.4f} Hz")
    print(f"Z80 interrupt counter F004: {irq} counts  ->  {irq / secs:.3f} Hz")
    print(f"ticks handled by the main loop F005: {proc} counts  ->  {proc / secs:.3f} Hz "
          f"(the sequencer runs on every 4th: {proc / secs / 4:.3f} Hz)")

    def code(e):
        return f"{(e[0] << 8) | e[1]:04x}"
    last = rows[-1][0] + 1
    phases = [("both flags clear", 0, poke or last), ("G.$777C bit 5 set", poke or last, poke2 or last),
              ("bit 5 clear again, G.$4CC = 1", poke2 or last, last)]
    for name, a, b in phases:
        got = [(fr, e) for fr, e in reqs if a <= fr < b]
        plain = [code(e) for _, e in got if e[0] != 0xFF and code(e) != "0030"]
        print(f"phase frames {a}..{b - 1} ({name}): {len(got)} requests, "
              f"{len(plain)} that are neither 0xFFxx nor 0x0030; special codes {sorted({code(e) for _, e in got if e[0] == 0xFF})}; "
              f"other codes {sorted(set(plain))}")


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--report":
        report(sys.argv[2])
    else:
        main()
