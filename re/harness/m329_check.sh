#!/bin/sh
# M3-29: the autoplay from power on to the ending, the link scenes and the area changes against the MAME flow recordings.
# Usage: re/harness/m329_check.sh
# 1. `dino-game --headless --autoplay` (no --frames): coin, start, Jack, all eight episodes with the kill all every 600 frames, god mode and a topped up clock,
#    the ending and the attract loop. Prints one line per change of the game mode and the area, and the frame the ending began.
# 2. link_check.sh link-ep2 .. link-ep8 (the story panel, the map card, the walk in, 1100 records from the poke of mode 8): the first difference of each.
# 3. link_lengths.py: the record of every game mode change and of G.$4E3 in MAME and in the port (equal for all seven).
# 4. area_events.py for the 13 area changes of m328_areas (free run with DINO_GAME_CLOCK=1): the records of the end mode, the area load, the clock reload and the
#    control release, MAME against the port. The differences and their causes are listed in re/harness/allow/m3.txt (section M3-29).
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
PY="$ROOT/re/.venv/bin/python -I"
echo "== 1. autoplay"
(cd "$ROOT/dino-rs" && cargo build --release -q -p dino-game 2>/dev/null; ./target/release/dino-game --rom ../dino.zip --headless --autoplay 2>&1 | tail -8)
echo "== 2. link scenes"
sh "$ROOT/re/harness/link_check.sh" link-ep2 link-ep3 link-ep4 link-ep5 link-ep6 link-ep7 link-ep8 2>&1 | cut -c1-240
echo "== 3. link lengths"
$PY "$ROOT/re/harness/link_lengths.py"
echo "== 4. area changes"
LIST="m328_e1a0:1000 m328_n1a1:1800 m328_e2a0n:1200 m328_e2a1n:1000 m328_e4a0n:900 m328_e4a1n:1200 m328_e5a0n:1600 m328_e6a0n:1600 m328_e6a1n:1000 m328_l_e7a0_k16:600 m328_e8a0n:1200 m328_e8a1n:900 m328_e8a2n:1400"
NAMES=
for item in $LIST; do
  n=${item%%:*}; f=${item##*:}
  [ -f "$ROOT/re/out/traces/$n.mame.trc" ] || { echo "$n: no recording"; continue; }
  DINO_GAME_CLOCK=1 MODE=run "$ROOT/re/harness/port.sh" "$ROOT/re/out/traces/$n.input.txt" "$n" "$f" > "$ROOT/re/out/$n.m329.log" 2>&1
  NAMES="$NAMES $n"
done
# shellcheck disable=SC2086
$PY "$ROOT/re/harness/area_events.py" $NAMES
