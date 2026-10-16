#!/bin/sh
# M3-08 to M3-12: the Episode 3 camera (controller 0x22398, the car drive, the row scroll 0x2335A) against the recordings of re/harness/pokes/m308.txt.
# Usage: re/harness/m308_check.sh [NAME[:FRAMES] ...]     (default: the eight scenarios below)
# Record first (one MAME process, a few minutes):  PR_SNAPS=none PR_SHEET=0 re/harness/probe/probe_record.sh re/harness/pokes/m308.txt all
# Each scenario runs in `step` mode (the port is loaded with MAME's state before every frame) and is compared with diff.py. Printed per scenario: the first
# difference, the counts of ported-class and global differences, and every difference of the camera fields, the row scroll words and the tilemaps (the
# causes of the car scenarios are in re/harness/allow/m3.txt and in specs/stage/HANDOFF.md). The car chase is also checked on its own: the formula
# camX += clamp(car X - (camX + 0xC0), 8, 20) with the car X of the same frame, on every frame of the recording that is in mode 8 with G.$518 = 1.
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
LIST=${*:-"m308_foot1p:1000 m308_foot3p:1500 m308_car:1200 m308_car_end:800 m308_car_time:800 m308_row300:25 m308_row1000:25 m308_row2400:25 m308_rowmove:90"}
PY="$ROOT/re/.venv/bin/python -I"
mkdir -p "$ROOT/re/out/m308"
for item in $LIST; do
  n=${item%%:*}; f=${item##*:}; [ "$f" = "$item" ] && f=600
  [ -f "$ROOT/re/out/traces/$n.mame.trc" ] || { echo "$n: no recording (re/harness/probe/probe_record.sh re/harness/pokes/m308.txt all)"; continue; }
  MODE=step "$ROOT/re/harness/port.sh" "$ROOT/re/out/traces/$n.input.txt" "$n" "$f" > "$ROOT/re/out/m308/$n.port.log" 2>&1 || { echo "$n: the port failed, see re/out/m308/$n.port.log"; continue; }
  if grep -q panicked "$ROOT/re/out/m308/$n.port.log"; then echo "$n: PANIC"; grep -A2 panicked "$ROOT/re/out/m308/$n.port.log" | head -4; continue; fi
  case $n in m308_car*) ALLOW="--allow $ROOT/re/harness/allow/m3.txt" ;; *) ALLOW= ;; esac
  # shellcheck disable=SC2086
  out=$($PY "$ROOT/re/harness/diff.py" "$ROOT/re/out/traces/$n.mame.trc" "$ROOT/re/out/traces/$n.step.trc" $ALLOW 2>&1)
  echo "== $n ($f frames)"
  echo "$out" | awk '/^FIRST DIFFERENCE/{print; getline; print "  " $0} /^RESULT/{print} /^SUMMARY/{print} /object fields of ported classes/{print}' | cut -c1-200
  echo "$out" | sed -n '/^FIELDS WITH DIFFERENCES/,/^DIFFERENCES BY/p' | grep -E "^  global\.(cam|event|area|vfollow|go |lock|push|max_|min_|up52|down54|display|shake|rowscroll|b4d4|stage_clear)" | cut -c1-170
  echo "$out" | sed -n '/^VIDEO AND SOUND/,/^$/p' | grep -E "^  (scroll[123]|rowscroll|palette)" | cut -c1-170
done
# the chase formula on the recording of the radio (the car X of the frame, the camera X of the frame before)
if [ -f "$ROOT/re/out/traces/m308_car.mame.trc" ]; then
  $PY - "$ROOT/re/out/traces/m308_car.mame.trc" "$ROOT/re/harness" <<'PY'
import sys
sys.path.insert(0, sys.argv[2])
import trace as T
prev, n, bad = None, 0, 0
for r in T.Trace(sys.argv[1]).records():
    ram = r.images["ram"]
    camx, car, mode = T.u16(ram, 0xFF8744), T.u8(ram, 0xFF8518), T.u8(ram, 0xFF8741)
    carx = T.u16(ram, 0xFF0000 + T.u16(ram, 0xFF851A) + 8)
    if prev and prev[1] == 1 and mode == 8:
        d = (carx - (prev[0] + 0xC0)) & 0xFFFF
        d = d - 0x10000 if d >= 0x8000 else d
        step = 8 if d < 0 else (0x14 if d > 0x14 else max(d, 8))
        n += 1
        bad += (prev[0] + step) & 0xFFFF != camx
    prev = (camx, car)
print(f"== chase formula on m308_car: {n} frames in mode 8 with G.$518 = 1, {bad} differ")
PY
fi
