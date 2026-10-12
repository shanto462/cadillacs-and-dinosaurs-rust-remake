#!/bin/sh
# Record probe segments as harness scenarios (plan task M3-06): a state snapshot, a MAME trace and the input script.
#   re/harness/probe/probe_record.sh BATCH SEGMENT[,SEGMENT..]|all [NAME]
# BATCH is a probe input file (`segment NAME` blocks, see plan/game/design/capture-recipes.md). A segment is a
# scenario when it has a `snapshot` line: the lines before it set up the scene (jump, poke, spawn, hold, god, keys),
# the key lines after it are the input script, `trace N` says how many game frames to record (head of probe.lua).
# ONE MAME process runs all the chosen segments (`all` = every segment that has a snapshot line). Output, in
# re/out/traces/ (git-ignored), for every segment NAME (the segment name, or the third argument for one segment):
#   NAME.state        RAM and video RAM at the snapshot (port: --load-state)
#   NAME.mame.trc     the harness trace, record -1 = the snapshot, then N game frames (same file as record.sh writes)
#   NAME.input.txt    the key lines after the snapshot, in the harness script format (port: --input-file)
# Then compare with the port like any scenario (check.sh only reads scripts from re/harness/scripts/, use port.sh):
#   MODE=step re/harness/port.sh re/out/traces/NAME.input.txt NAME N
#   re/.venv/bin/python -I re/harness/diff.py re/out/traces/NAME.mame.trc re/out/traces/NAME.step.trc
# Logs and screenshots: re/out/probe/BATCH-rec/SEGMENT/{log.txt,snap/}, MAME's own files in re/out/probe/BATCH-rec/mame/.
# Environment (all optional): PR_TIMEOUT (seconds, default 900), PR_SNAPS / PR_FRAMES / PR_WATCH (as probe.sh),
# PR_SNAP_MAXF (a segment fails when its snapshot is later than this many setup frames, default 12000),
# PR_PROBE_LUA / PR_RECORD_LUA (other copies of probe.lua and record.lua, for testing the tool itself).
# Headless, like probe.sh: no video, no sound, SDL dummy drivers (no Dock icon, no focus). A failed segment leaves no files.
set -e
ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
BATCH=${1:?usage: probe_record.sh BATCH SEGMENT[,SEGMENT..]|all [NAME]}
CHOSEN=${2:?usage: probe_record.sh BATCH SEGMENT[,SEGMENT..]|all [NAME]}
ONE_NAME=${3:-}
[ -f "$BATCH" ] || { echo "probe_record: no such batch file: $BATCH" >&2; exit 2; }
BATCH=$(cd "$(dirname "$BATCH")" && pwd)/$(basename "$BATCH")
BATCHNAME=$(basename "$BATCH" .txt)
if [ "$CHOSEN" = all ]; then PR_ONLY='*'; else PR_ONLY=$CHOSEN; fi
if [ -n "$ONE_NAME" ]; then
  case "$CHOSEN" in all | *,*) echo "probe_record: a NAME needs exactly one SEGMENT" >&2; exit 2 ;; esac
fi
# The scenarios of the batch: the segments that have a snapshot line (a comment starts with #, ; ends a line).
SCENARIOS=$(awk '
  { line = $0; sub(/#.*/, "", line); n = split(line, parts, ";")
    for (i = 1; i <= n; i++) {
      if (match(parts[i], /^[ \t]*segment[ \t]+[^ \t]+/)) {
        name = substr(parts[i], RSTART, RLENGTH); sub(/^[ \t]*segment[ \t]+/, "", name); cur = name; order[++count] = cur
      } else if (cur != "" && parts[i] ~ /^[ \t]*[0-9]*[ \t]*snapshot([ \t]|$)/) snap[cur] = 1
    } }
  END { for (i = 1; i <= count; i++) if (snap[order[i]]) print order[i] }' "$BATCH")
if [ "$CHOSEN" = all ]; then
  WANT=$SCENARIOS
else
  # Every named segment must exist and have a snapshot line (a typo would otherwise record nothing).
  WANT=$(echo "$CHOSEN" | tr ',' ' ')
  for S in $WANT; do
    echo "$SCENARIOS" | grep -qx -- "$S" || { echo "probe_record: segment $S: not found in $BATCH, or it has no snapshot line" >&2; exit 2; }
  done
fi
[ -n "$WANT" ] || { echo "probe_record: $BATCH has no segment with a snapshot line" >&2; exit 2; }
OUT=$ROOT/re/out/probe/$BATCHNAME-rec
TRACES=$ROOT/re/out/traces
rm -rf "$OUT"
mkdir -p "$OUT/mame" "$TRACES"
cd "$ROOT"
export PR_INPUT=$BATCH PR_ONLY PR_OUTDIR=$OUT PR_NAME=$BATCHNAME PR_TRACE_DIR=$TRACES
export PR_RECORD_LUA=${PR_RECORD_LUA:-$ROOT/re/harness/record.lua}
[ -z "$ONE_NAME" ] || export PR_TRACE_NAME=$ONE_NAME
# This tool is not the flow batch recorder of record.sh --batch: keep its variables out.
unset DINO_BATCH_PROBE DINO_POKES DINO_POKE
export SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy
perl -e 'alarm shift; exec @ARGV' "${PR_TIMEOUT:-900}" \
  mame dino -rompath ".;re/mame/roms" -video none -sound none -nothrottle \
  -keyboardprovider none -mouseprovider none -joystickprovider none -lightgunprovider none \
  -cfg_directory "$OUT/mame/cfg" -nvram_directory "$OUT/mame/nvram" -state_directory "$OUT/mame/sta" \
  -snapshot_directory "$OUT" -diff_directory "$OUT/mame/diff" \
  -autoboot_script "${PR_PROBE_LUA:-$ROOT/re/harness/probe/probe.lua}" >"$OUT/mame.log" 2>&1 || true
rm -rf "$OUT/mame/sta"
grep -a 'probe.lua' "$OUT/mame.log" >&2 || true
# segments.txt has one line per finished segment: NAME <TAB> message. A scenario ends with "trace N frames ...".
FAILED=0
for SEG in $WANT; do
  NAME=${ONE_NAME:-$SEG}
  MSG=$(awk -F '\t' -v want="$SEG" '$1 == want { msg = $2 } END { print msg }' "$OUT/segments.txt" 2>/dev/null)
  case "$MSG" in
    "trace "[0-9]*" frames"*)
      N=${MSG#trace }; N=${N%% frames*}
      if [ -s "$TRACES/$NAME.state" ] && [ -s "$TRACES/$NAME.mame.trc" ] && [ -f "$TRACES/$NAME.input.txt" ]; then
        echo "recorded $NAME: $N frames ($MSG)"
        echo "  state $(wc -c <"$TRACES/$NAME.state") bytes, trace $(wc -c <"$TRACES/$NAME.mame.trc") bytes"
        echo "  MODE=step re/harness/port.sh re/out/traces/$NAME.input.txt $NAME $N"
      else
        FAILED=1; echo "probe_record: $NAME: the output files are missing" >&2
      fi ;;
    *)
      FAILED=1
      echo "probe_record: $SEG: ${MSG:-did not finish}" >&2
      rm -f "$TRACES/$NAME.state" "$TRACES/$NAME.mame.trc" "$TRACES/$NAME.input.txt" ;;
  esac
done
if [ "$FAILED" = 1 ]; then
  echo "probe_record: not all scenarios were recorded, see $OUT/mame.log and $OUT/*/log.txt" >&2
  exit 1
fi
# Contact sheets of the screenshots, if the segments asked for any (as probe.sh does).
if [ "${PR_SHEET:-1}" != "0" ]; then
  for L in "$OUT"/*/log.txt; do
    [ -f "$L" ] || continue
    D=$(dirname "$L")
    N=$(ls "$D/snap" 2>/dev/null | grep -c '\.png$' || true)
    if [ "$N" -gt 0 ]; then "$ROOT/re/.venv/bin/python" -I "$ROOT/re/harness/probe/sheet.py" "$D" 2>&1 | sed 's/^/    /'; fi
  done
fi
