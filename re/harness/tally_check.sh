#!/bin/sh
# M2-24: the clear tally against the flow_tally recordings. Usage: re/harness/tally_check.sh [SCENARIO ...] (default: all 31)
# Each scenario is cut at the first record where G.$4D1 is 6 (the tally task has run once: re/harness/trace_slice.py), the port is run from
# that snapshot (check.sh run, no input) and the first difference is printed. The map panel (M2-25) and the story panel (M2-28) differ from
# the records on where the tally ends; the number of frames before it is the check.
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
LIST=${*:-"tv-hp100 tv-hp99 tv-hp98 tv-hp90 tv-hp75 tv-hp50 tv-hp41 tv-hp40 tv-hp25 tv-hp10 tv-hp2 tv-hp1 tv-k0 tv-k1 tv-k2 tv-k5 tv-k50 tv-k99 tv-k150 tv-k255 tv-fb tv-2p-a tv-2p-b tv-2p-eq tv-3p-a td-1p-quiet td-1p-dead td-2p-12 td-2p-21 td-3p-231 td-3p-tie"}
FRAMES=${FRAMES:-900}
for n in $LIST; do
  src=$ROOT/re/out/traces/flow_$n.mame.trc
  [ -f "$src" ] || { echo "$n: no recording"; continue; }
  "$ROOT/re/.venv/bin/python" -I "$ROOT/re/harness/trace_slice.py" "$src" "m224_$n" FF84D1=6 --frames "$FRAMES" > /dev/null || { echo "$n: slice failed"; continue; }
  out=$(SCRIPT=m224_tally FRAMES=$FRAMES "$ROOT/re/harness/check.sh" run "m224_$n" 2>&1)
  first=$(echo "$out" | awk '/^FIRST DIFFERENCE/{print $4; exit}')
  items=$(echo "$out" | awk '/^FIRST DIFFERENCE/{p=1;next} p&&/^$/{exit} p{print}' | head -3 | tr '\n' '|' | cut -c1-230)
  echo "$n: first difference at frame ${first:-none}: $items"
  # The frame of the panel draw can take the CPU through the next video frame (3 heroes, a tie of 2): the interrupt hits in the middle of the draw,
  # so that record shows the last cells undrawn (scroll1 only). The port draws them at once and models the lost frame (`tally_state4`).
  # That one scroll1 difference is allowed and the run is checked on.
  if [ -n "$first" ] && echo "$items" | grep -q '^ *scroll1:'; then
    echo "scroll1 $first-$first" > "$ROOT/re/out/traces/m224_$n.allow"
    out=$(SCRIPT=m224_tally FRAMES=$FRAMES "$ROOT/re/harness/check.sh" run "m224_$n" --allow "$ROOT/re/out/traces/m224_$n.allow" 2>&1)
    first=$(echo "$out" | awk '/^FIRST DIFFERENCE/{print $4; exit}')
    items=$(echo "$out" | awk '/^FIRST DIFFERENCE/{p=1;next} p&&/^$/{exit} p{print}' | head -3 | tr '\n' '|' | cut -c1-230)
    echo "$n:   (scroll1 allowed at that frame) next first difference at frame ${first:-none}: $items"
  fi
done
