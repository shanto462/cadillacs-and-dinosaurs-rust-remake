#!/bin/sh
# M3-13 / M3-14: the lift of Episode 7 area 1 (cage background, camera mode 5, floor labels B57, cage posts B64, riders by Y) against the recordings.
# Usage: re/harness/m313_check.sh
# 1. m328_l_e7a0_k16 (run mode: the port plays on its own from the snapshot of the end of Episode 7 area 0 and makes the area change): the cage (scroll1) equals MAME in every record
#    but the area start record 382 (the HUD cells of the lag frame), the labels equal MAME one record late (the area start is one frame late there, see D5-errata.md).
# 2. m800_a14_n_e7a1_full (step mode, 4900 records of the ride with its four stops and the end lock): the camera, the labels (scroll2), the posts and the cage equal MAME in every record.
# 3. ep7_3p (step mode, three heroes, the whole lift is records 12306 to 17850): the same, from record 12310.
# The causes of what is left are in re/harness/allow/m313.txt and re/specs/stage/HANDOFF.md.
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
PY="$ROOT/re/.venv/bin/python -I"
ALLOW="$ROOT/re/harness/allow/m313.txt"
OUT="$ROOT/re/out/m313"
mkdir -p "$OUT"
run() { # MODE NAME FRAMES [diff options]
  mode=$1; n=$2; f=$3; shift 3
  [ -f "$ROOT/re/out/traces/$n.mame.trc" ] || { echo "$n: no recording"; return; }
  case $mode in run) t=port ;; *) t=$mode ;; esac
  MODE=$mode "$ROOT/re/harness/port.sh" "$ROOT/re/out/traces/$n.input.txt" "$n" "$f" > "$OUT/$n.$mode.log" 2>&1
  [ -f "$ROOT/re/out/traces/$n.input.txt" ] || MODE=$mode "$ROOT/re/harness/port.sh" "$ROOT/re/harness/scripts/$n.txt" "$n" "$f" > "$OUT/$n.$mode.log" 2>&1
  if grep -q panicked "$OUT/$n.$mode.log"; then echo "$n: PANIC"; grep -A2 panicked "$OUT/$n.$mode.log" | head -4; return; fi
  echo "== $n ($mode, $f records) $*"
  # shellcheck disable=SC2086
  $PY "$ROOT/re/harness/diff.py" "$ROOT/re/out/traces/$n.mame.trc" "$ROOT/re/out/traces/$n.$t.trc" --allow "$ALLOW" "$@" > "$OUT/$n.$mode.diff" 2>&1
  awk '/^FIRST DIFFERENCE/{print} /^RESULT/{print} /^SUMMARY/{print} /object fields of ported classes/{print}' "$OUT/$n.$mode.diff" | cut -c1-200
  sed -n '/^VIDEO AND SOUND/,/^$/p' "$OUT/$n.$mode.diff" | grep -E "^  (scroll[123]|palette)" | cut -c1-170
  sed -n '/^DIFFERENCES BY OBJECT CLASS/,/^SPRITE/p' "$OUT/$n.$mode.diff" | grep -E "class 0x(39|40) " | cut -c1-170
}
run run m328_l_e7a0_k16 600
run step m800_a14_n_e7a1_full 4900
run step ep7_3p 17850 --from 12310
