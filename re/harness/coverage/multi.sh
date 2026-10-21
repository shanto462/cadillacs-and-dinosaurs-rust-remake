#!/bin/sh
# Run the recordings of a multi.lua list in ONE headless MAME process (see multi.lua, REPORT.md "Method").
#   re/harness/coverage/multi.sh LIST [FROM [TO]]
# FROM / TO: only the runs FROM..TO of the list (1-based, comment lines not counted). The recordings go to
# MR_OUT (default re/out/traces of the root this script is in), MAME's files to re/out/mame-multi/NAME-FROM-TO-TIME/.
set -e
ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
LIST=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
FROM=${2:-1}; TO=${3:-999999}
MAMEDIR=$ROOT/re/out/mame-multi/$(basename "$LIST" .list)-$FROM-$TO-$(date +%s)
mkdir -p "$MAMEDIR" "${MR_OUT:-$ROOT/re/out/traces}"
grep -v '^[[:space:]]*#' "$LIST" | grep . | awk -v a="$FROM" -v b="$TO" 'NR >= a && NR <= b' > "$MAMEDIR/runs.list"
N=$(grep -c . "$MAMEDIR/runs.list")
export MR_LIST=$MAMEDIR/runs.list MR_RECORD_LUA=$ROOT/re/harness/record.lua MR_OUT=${MR_OUT:-$ROOT/re/out/traces}
export SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy
cd "$ROOT"
perl -e 'alarm shift; exec @ARGV' "${DINO_TIMEOUT:-3600}" \
  mame dino -rompath ".;re/mame/roms" -video none -sound none -nothrottle \
  -cfg_directory "$MAMEDIR/cfg" -nvram_directory "$MAMEDIR/nvram" -state_directory "$MAMEDIR/sta" \
  -snapshot_directory "$MAMEDIR/snap" -diff_directory "$MAMEDIR/diff" \
  -autoboot_script "$ROOT/re/harness/coverage/multi.lua" > "$MAMEDIR/multi.log" 2>&1 || true
grep -a 'multi.lua\|record.lua: recorded\|error' "$MAMEDIR/multi.log" | tail -3
grep -aq "all $N runs done" "$MAMEDIR/multi.log" || { echo "multi.sh: not all $N runs finished, see $MAMEDIR/multi.log"; exit 1; }
echo "multi.sh: $N runs recorded"
