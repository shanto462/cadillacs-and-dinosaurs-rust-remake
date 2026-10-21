#!/bin/sh
# M10-22: the sound request streams (code, d1, d2) of the natural whole run recordings against MAME, in step mode (each record from MAME's
# state, so a difference comes from that record alone).
# Usage: [NORUN=1 (compare the traces of the last run)] [PORT_OUT=DIR] re/harness/m10_sound_check.sh [NAME[:RECORDS] ...]
# For each recording: the port's step trace (PORT_OUT, default re/out/m10snd), diff.py --frame-list (the records whose state differs), then
# snd_stream.py with the rules of re/harness/allow/m10_runs.txt for NAME.step (the same cut as m10_check.sh). One line per recording:
# the sound-only differences left (they must be 0) and those in records whose state differs too (they belong to the state checks of
# m10_check.sh). The details are in DIR/NAME.step.snd.txt. Exit 1 when a sound-only difference is left.
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
LIST=${*:-"ep1_3p:24337 ep2_3p:42001 ep2_jack:42001 ep3_3p:10501 ep3_hannah:10501 ep3_jack:10501 ep3_mess:10501 ep3_mustapha:10501
 ep4_3p:28001 ep4_jack:28001 ep5_3p:65001 ep5_hannah:65001 ep5_jack:65001 ep5_mess:65001 ep5_mustapha:65001 ep6_3p:70001 ep6_jack:70001
 ep7_3p:60001 ep7_jack:60001 ep8_3p:70001 ep8_hannah:70001 ep8_jack:70001 ep8_mess:70001 ep8_mustapha:70001
 m517_ep1_jack:29858 m517_ep1_hannah:15457 m517_ep1_mess:16146 m517_ep1_mustapha:30545 m523_ep1_2p:30701 m523_ep1_3pjhm:33801"}
PO=${PORT_OUT:-$ROOT/re/out/m10snd}
PY="$ROOT/re/.venv/bin/python -I"
mkdir -p "$PO"
bad=0
for item in $LIST; do
  n=${item%%:*}; f=${item##*:}; [ "$f" = "$item" ] && f=10501
  IN="$ROOT/re/out/traces/$n.input.txt"; [ -f "$IN" ] || IN="$ROOT/re/harness/scripts/$n.txt"
  if [ -z "$NORUN" ]; then
    MODE=step PORT_OUT="$PO" "$ROOT/re/harness/port.sh" "$IN" "$n" "$f" > "$PO/$n.step.log" 2>&1 || { echo "$n: the port failed, see $PO/$n.step.log"; bad=1; continue; }
  fi
  if grep -q panicked "$PO/$n.step.log" 2>/dev/null; then echo "$n: PANIC"; bad=1; continue; fi
  $PY "$ROOT/re/harness/diff.py" "$ROOT/re/out/traces/$n.mame.trc" "$PO/$n.step.trc" --frame-list > "$PO/$n.step.diff.txt" 2>&1
  awk -v n="$n" -v m=step 'BEGIN{FS=":"} /^[a-z0-9_]+\.[a-z*]+:/{split($1,a,"."); if (a[1]==n && (a[2]=="*" || a[2]==m)) {sub(/^[^:]*:[ ]*/, ""); print}; next} /^[^#]/ && NF{print}' \
    "$ROOT/re/harness/allow/m10_runs.txt" > "$PO/$n.step.allow"
  $PY "$ROOT/re/harness/snd_stream.py" "$ROOT/re/out/traces/$n.mame.trc" "$PO/$n.step.trc" --allow "$PO/$n.step.allow" --state "$PO/$n.step.diff.txt" \
    --json "$PO/$n.step.snd.json" > "$PO/$n.step.snd.txt" 2>&1 || bad=1
  echo "$n: $(head -1 "$PO/$n.step.snd.txt" | sed 's/^SOUND STREAM: mode step, //')"
done
exit $bad
