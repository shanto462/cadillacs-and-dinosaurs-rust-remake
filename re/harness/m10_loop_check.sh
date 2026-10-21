#!/bin/sh
# M10-06: the attract loop from power on, two full loops (29475 records of re/out/traces/attract3.mame.trc, no input), against MAME:
#   1. states: the port from power on (no poke file), diff.py from record 25 with allow/m1.txt + allow/m10.txt (the demo setups)
#   2. sound: the requests the flush 0x610 sends (snd_stream.py --heard: the attract filter) and, for the record, all queued requests
#   3. pictures: the headless renderer at the 416 video frames of the M1-15 probe batches (re/out/probe/m115_loop, m115_loop2, MAME
#      snapshots), shotdiff.py --clock f; the port picture of video frame f is the screenshot after f - 1 ticks. Skipped when the batches
#      are missing (they are recorded with re/harness/probe/probe.sh, see allow/m1.txt section 4).
# Usage: [PORT_OUT=DIR] [PROBE=DIR (the folder holding m115_loop and m115_loop2)] re/harness/m10_loop_check.sh
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
PO=${PORT_OUT:-$ROOT/re/out/m10loop}; mkdir -p "$PO"
PROBE=${PROBE:-$ROOT/re/out/probe}
PY="$ROOT/re/.venv/bin/python -I"
MAME=$ROOT/re/out/traces/attract3.mame.trc
[ -f "$MAME" ] || { echo "m10_loop_check: $MAME is missing"; exit 2; }
cd "$ROOT/dino-rs" && (cargo build --release -q -p dino-game 2>/dev/null || cargo build --release -p dino-game) || exit 1
GAME=$ROOT/dino-rs/target/release/dino-game
bad=0
"$GAME" --rom "$ROOT/dino.zip" --trace "$PO/loop.port.trc" --frames 29500 2>&1 | grep -v 'not ported (freed)' | grep -v '^trace:'
cat "$ROOT/re/harness/allow/m1.txt" "$ROOT/re/harness/allow/m10.txt" > "$PO/loop.allow"
out=$($PY "$ROOT/re/harness/diff.py" "$MAME" "$PO/loop.port.trc" --from 25 --allow "$PO/loop.allow")
echo "states:   $(echo "$out" | grep -E '^RESULT|^SUMMARY' | head -1)"
echo "$out" | grep -q '^RESULT: no differences' || { echo "$out" | awk '/^FIRST DIFFERENCE/{p=4} p>0{print "   " $0; p--}'; bad=1; }
echo "sound:    $($PY "$ROOT/re/harness/snd_stream.py" "$MAME" "$PO/loop.port.trc" --heard | head -1)" || bad=1
$PY "$ROOT/re/harness/snd_stream.py" "$MAME" "$PO/loop.port.trc" --heard > /dev/null || bad=1
echo "          (all queued: $($PY "$ROOT/re/harness/snd_stream.py" "$MAME" "$PO/loop.port.trc" --allow "$ROOT/re/harness/allow/m10_runs.txt" | head -1 | sed 's/^SOUND STREAM: //'))"
for b in m115_loop m115_loop2; do
  [ -f "$PROBE/$b/log.txt" ] || { echo "pictures: $PROBE/$b is missing, skipped"; continue; }
  L=$(grep SNAP "$PROBE/$b/log.txt" | sed 's/.* f=\([0-9]*\) .*/\1/' | awk '{printf "%s%d", (NR>1?",":""), $1-1}')
  rm -rf "$PO/$b" "$PO/${b}_f"; mkdir -p "$PO/$b" "$PO/${b}_f"
  "$GAME" --rom "$ROOT/dino.zip" --headless --screenshots "$L" --shot-dir "$PO/$b" --frames 29100 2>&1 | grep -v 'not ported (freed)' | grep -v '^trace:'
  for f in "$PO/$b"/f*.png; do n=$(basename "$f" .png | sed 's/^f0*//'); cp "$f" "$PO/${b}_f/f$(printf %05d $((${n:-0} + 1))).png"; done
  $PY "$ROOT/re/harness/probe/shotdiff.py" "$PROBE/$b" "$PO/${b}_f" --clock f --worst 8 --out "$PO/$b.sheet.png" > "$PO/$b.shotdiff.txt" 2>&1
  echo "pictures $b: $(grep -c '^frame' "$PO/$b.shotdiff.txt") frames, $(grep '^frame' "$PO/$b.shotdiff.txt" | grep -vc ' 0.00%') differ: $(grep '^frame' "$PO/$b.shotdiff.txt" | grep -v ' 0.00%' | awk '{printf "%s ", $2}')"
done
echo "(the differing pictures and their causes: allow/m1.txt section 4)"
exit $bad
