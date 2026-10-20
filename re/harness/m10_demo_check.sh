#!/bin/sh
# M10-03 to M10-05: the six demos of the attract loop against MAME (D4d s3.6, s13 A7). Usage (from anywhere):
#   re/harness/m10_demo_check.sh [A ...]        (default: the six demo calls of the loop recording)
#
# Needs re/out/traces/flow_m1003-loop.mame.trc (40000 records from power on, no input):
#   DINO_TIMEOUT=3000 re/harness/record.sh --batch re/harness/scripts/m1003_loop.txt m1003_loop
#
# 1. free run: the port from power on (its own RAM clear and setup of every demo), 40000 frames; diff.py from A to the first record
#    of the next screen (A + 1326) with the rules of allow/m10.txt. "RESULT: no differences" is the goal.
# 2. step-like run of each demo: the port is loaded with MAME's state after every record A + 11 .. A + 1209 (the lag-sync mode with
#    DINO_LAGSYNC_EVERY=1 and DINO_LAGSYNC_RANGE: the front end tasks of the port stay as they are), diff.py A + 12 .. A + 1210 without
#    allow rules: every difference there comes from that one frame. Expected: none, except Episode 3 at 10842 (the tag team move).
# Port traces go to PORT_OUT (default re/out/port of this checkout; re/out/traces is shared).
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
PO=${PORT_OUT:-$ROOT/re/out/port}; mkdir -p "$PO"
MAME=$ROOT/re/out/traces/flow_m1003-loop.mame.trc
[ -f "$MAME" ] || { echo "m10_demo_check: $MAME is missing (see the header of this script)"; exit 2; }
PY="$ROOT/re/.venv/bin/python -I $ROOT/re/harness/diff.py"
DEMOS=${*:-"3343 9914 16448 23019 29553 36124"}
cd "$ROOT/dino-rs" && (cargo build --release -q -p dino-game 2>/dev/null || cargo build --release -p dino-game) || exit 1
GAME=$ROOT/dino-rs/target/release/dino-game
"$GAME" --rom "$ROOT/dino.zip" --trace "$PO/m1003-loop.port.trc" --frames 40000 2>&1 | grep -v 'not ported (freed)' | grep -v '^trace:'
for a in $DEMOS; do
  echo "demo $a: free run   $($PY "$MAME" "$PO/m1003-loop.port.trc" --from "$a" --frames $((a + 1327)) \
    --allow "$ROOT/re/harness/allow/m10.txt" | grep -E '^RESULT|^FIRST')"
done
for a in $DEMOS; do
  DINO_LAGSYNC_EVERY=1 DINO_LAGSYNC_RANGE=$((a + 11))-$((a + 1209)) "$GAME" --rom "$ROOT/dino.zip" --lagsync "$MAME" \
    --trace "$PO/m1003-step-$a.port.trc" --frames $((a + 1220)) 2>&1 | grep -v 'not ported (freed)' | grep -v '^trace:'
  echo "demo $a: step-like  $($PY "$MAME" "$PO/m1003-step-$a.port.trc" --from $((a + 12)) --frames $((a + 1211)) \
    | grep -E '^RESULT|^FIRST')"
done
