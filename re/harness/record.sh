#!/bin/sh
# Record a MAME reference trace. Usage (from anywhere):
#   re/harness/record.sh TEST_SCRIPT [NAME] [FRAMES]
# Writes re/out/traces/NAME.state (RAM at the sync point) and re/out/traces/NAME.mame.trc.
# NAME defaults to the script file name without .txt, FRAMES to 600.
# Optional environment: DINO_BOOT (boot script, default scripts/boot_jack.txt), DINO_SYNC
# (power, control, control+N or vblank:N), DINO_SNAP_EVERY (PNG every N frames into re/out/mame/snap/dino/).
# DINO_CLOCK=video records every video frame from boot (default game skips slowdown).
# DINO_POKES=file applies timed writes from power on (frame hexaddr size value).
# MAME's own files (cfg, nvram, screenshots, record.log) go to re/out/mame/. See README.md.
set -e
# --batch PROBE_SCRIPT NAME [POKES] runs all segments in one process and
# records each segment from power on, including runtime-assisted jumps.
BATCH=0
if [ "${1:-}" = "--batch" ]; then BATCH=1; shift; fi
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
SCRIPT=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
NAME=${2:-$(basename "$1" .txt)}
FRAMES=${3:-600}
if [ "$BATCH" = 1 ]; then
  FRAMES=0
  export PR_INPUT=$SCRIPT PR_NAME=$NAME
  if [ -n "${3:-}" ]; then
    export DINO_BATCH_POKES=$(cd "$(dirname "$3")" && pwd)/$(basename "$3")
  fi
  export DINO_BATCH_PROBE=$ROOT/re/harness/probe/probe.lua
fi
OUT=$ROOT/re/out/traces
MAMEDIR=$ROOT/re/out/mame
if [ "$BATCH" = 1 ]; then
  MAMEDIR=$ROOT/re/out/flow/$NAME/mame
  export PR_OUTDIR=$ROOT/re/out/flow/$NAME DINO_TRACE_DIR=$OUT
  mkdir -p "$PR_OUTDIR"
  if [ -n "${DINO_BATCH_POKES:-}" ]; then
    python3 - "$SCRIPT" "$DINO_BATCH_POKES" "$PR_OUTDIR/merged.txt" <<'PYMERGE'
import re, sys
from pathlib import Path
script, pokes, output = map(Path, sys.argv[1:])
def segments(text):
    result = {}
    current = None
    for line in text.splitlines():
        if line.startswith("segment "):
            current = line.split()[1]
            if current in result: raise ValueError("duplicate segment " + current)
            result[current] = []
        elif current:
            result[current].append(line)
        elif line.strip() and not line.lstrip().startswith("#"):
            raise ValueError("expected segment before " + line)
    return result
inputs, writes = segments(script.read_text()), segments(pokes.read_text())
if inputs.keys() != writes.keys(): raise ValueError("input and poke segments differ")
for name, lines in writes.items():
    for n, line in enumerate(lines, 1):
        line = line.split("#", 1)[0].strip()
        if not line: continue
        m = re.fullmatch(r"([0-9]+) ([0-9A-Fa-f]+) ([124]) (0x[0-9A-Fa-f]+|[0-9]+)", line)
        if not m: raise ValueError(f"bad poke line {name}:{n}: {line}")
        frame, addr, size, value = m.groups()
        addr, size, value = int(addr,16), int(size), int(value,16) if value.startswith("0x") else int(value)
        if not (0xFF0000 <= addr and addr+size <= 0x1000000 and 0 <= value < 1 << (8*size) and (size == 1 or addr % 2 == 0)):
            raise ValueError(f"bad poke range {name}:{n}")
output.write_text("\n".join("segment " + name + "\n" + "\n".join(lines + writes[name]) for name, lines in inputs.items()) + "\n")
PYMERGE
    export PR_INPUT=$PR_OUTDIR/merged.txt
  fi
fi
mkdir -p "$OUT" "$MAMEDIR"
if [ -n "${DINO_POKES:-}" ]; then
  export DINO_POKES=$(cd "$(dirname "$DINO_POKES")" && pwd)/$(basename "$DINO_POKES")
fi
# A fresh EEPROM every run, so every recording boots the same way.
rm -rf "$MAMEDIR/nvram"
cd "$ROOT"
export DINO_BOOT=${DINO_BOOT:-$ROOT/re/harness/scripts/boot_jack.txt}
export DINO_INPUT=$SCRIPT DINO_FRAMES=$FRAMES
export DINO_TRACE=$OUT/$NAME.mame.trc DINO_STATE=$OUT/$NAME.state
if [ "$BATCH" = 0 ]; then rm -f "$DINO_TRACE" "$DINO_STATE"; fi
# No Dock icon or focus change on macOS: SDL with dummy drivers never registers MAME as a GUI app.
export SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy
perl -e 'alarm shift; exec @ARGV' "${DINO_TIMEOUT:-900}" \
  mame dino -rompath ".;re/mame/roms" -video none -sound none -nothrottle \
  -cfg_directory "$MAMEDIR/cfg" -nvram_directory "$MAMEDIR/nvram" \
  -state_directory "$MAMEDIR/sta" -snapshot_directory "${PR_OUTDIR:-$MAMEDIR/snap}" -diff_directory "$MAMEDIR/diff" \
  -autoboot_script "$ROOT/re/harness/record.lua" >"$MAMEDIR/record.log" 2>&1 || true
grep -a 'record.lua' "$MAMEDIR/record.log" || { echo "MAME failed, see $MAMEDIR/record.log"; exit 1; }
if grep -aE 'error:|LUA ERROR|FAILED|bad DINO_POKES' "$MAMEDIR/record.log"; then exit 1; fi
if [ "$BATCH" = 1 ]; then
  EXPECTED=$(grep -c '^segment ' "$SCRIPT")
  grep -aq "batch completed segments: $EXPECTED$" "$MAMEDIR/record.log" || {
    echo "Incomplete flow batch, see $MAMEDIR/record.log"; exit 1;
  }
else
  grep -aq "recorded $FRAMES frames" "$MAMEDIR/record.log" || {
    echo "Incomplete recording, see $MAMEDIR/record.log"; exit 1;
  }
  ls -l "$OUT/$NAME.state" "$OUT/$NAME.mame.trc"
fi
