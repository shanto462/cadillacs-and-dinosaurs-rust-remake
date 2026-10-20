#!/bin/sh
# M10-08 / M10-09: the natural whole-run recordings of Episode 2 and 3 in step, lagsync and run mode (NAME[:FRAMES] ..., default the list below).
# Usage: [NORUN=1 (only compare the traces of the last run)] [MODES="step lagsync run"] re/harness/m10_check.sh [NAME[:FRAMES] ...]
# Prints per mode the summary line and the first difference. The allowed differences are in re/harness/allow/m10_runs.txt (diff.py --allow); the second
# line of each mode is the result with that file. The port traces go to re/out/traces/NAME.{step,lagsync,port}.trc, the logs to re/out/m10/.
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
LIST=${*:-"ep2_jack:42001 ep2_3p:42001 ep3_jack:10501 ep3_hannah:10501 ep3_mess:10501 ep3_mustapha:10501 ep3_3p:10501"}
MODES=${MODES:-"step lagsync run"}
PY="$ROOT/re/.venv/bin/python -I"
mkdir -p "$ROOT/re/out/m10"
for item in $LIST; do
  n=${item%%:*}; f=${item##*:}; [ "$f" = "$item" ] && f=10501
  IN="$ROOT/re/out/traces/$n.input.txt"; [ -f "$IN" ] || IN="$ROOT/re/harness/scripts/$n.txt"
  for m in $MODES; do
    [ -n "$NORUN" ] || MODE=$m "$ROOT/re/harness/port.sh" "$IN" "$n" "$f" > "$ROOT/re/out/m10/$n.$m.log" 2>&1 || { echo "$n $m: the port failed, see re/out/m10/$n.$m.log"; continue; }
    if grep -q panicked "$ROOT/re/out/m10/$n.$m.log"; then echo "$n $m: PANIC"; grep -A2 panicked "$ROOT/re/out/m10/$n.$m.log" | head -4; continue; fi
    t=$m; [ "$m" = run ] && t=port
    out=$($PY "$ROOT/re/harness/diff.py" "$ROOT/re/out/traces/$n.mame.trc" "$ROOT/re/out/traces/$n.$t.trc" 2>&1)
    echo "$n $m: $(echo "$out" | awk '/^SUMMARY/{print}')"
    echo "$out" | awk '/^FIRST DIFFERENCE/{p=3} p>0{print "   " $0; p--}'
    if [ -f "$ROOT/re/harness/allow/m10_runs.txt" ]; then
      # the rules of this recording: lines "NAME.MODE: key range" (MODE step, lagsync or run, or * for all) are cut out to a plain allow file
      awk -v n="$n" -v m="$m" 'BEGIN{FS=":"} /^[a-z0-9_]+\.[a-z*]+:/{split($1,a,"."); if (a[1]==n && (a[2]=="*" || a[2]==m)) {sub(/^[^:]*:[ ]*/, ""); print}; next} /^[^#]/ && NF{print}' "$ROOT/re/harness/allow/m10_runs.txt" > "$ROOT/re/out/m10/$n.$m.allow"
      out=$($PY "$ROOT/re/harness/diff.py" "$ROOT/re/out/traces/$n.mame.trc" "$ROOT/re/out/traces/$n.$t.trc" --allow "$ROOT/re/out/m10/$n.$m.allow" 2>&1)
      echo "   with m10_runs.txt: $(echo "$out" | awk '/^SUMMARY/{print}')"
      echo "$out" | awk '/^FIRST DIFFERENCE/{p=3} p>0{print "     " $0; p--}'
    fi
  done
done
