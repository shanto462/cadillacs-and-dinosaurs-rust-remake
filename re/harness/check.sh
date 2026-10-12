#!/bin/sh
# Run the port in one mode on a recorded scenario and print the diff report.
# Usage: re/harness/check.sh MODE NAME [diff.py options...]     MODE = step | follow | run
# Scenario table (script and frames) as in README.md. The MAME trace must exist (record.sh).
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
MODE=$1; NAME=$2; shift 2
case "$NAME" in
  stage1_jack|stage1_late) S=stage1_jack; F=600 ;;
  explore) S=explore; F=600 ;;
  stage1_fight) S=stage1_fight; F=1400 ;;
  obstacle) S=obstacle; F=600 ;;
  melee*) S=melee; F=820 ;;
  armed) S=armed; F=1000 ;;
  grabs_*|grabs) S=grabs; F=1200 ;;
  grabs2*) S=grabs2; F=1100 ;;
  grabs3*) S=grabs3; F=1200 ;;
  *) S=${SCRIPT:-$NAME}; F=${FRAMES:-600} ;;
esac
[ -n "$SCRIPT" ] && S=$SCRIPT
[ -n "$FRAMES" ] && F=$FRAMES
case $MODE in run) T=port ;; *) T=$MODE ;; esac
MODE=$MODE "$ROOT/re/harness/port.sh" "$ROOT/re/harness/scripts/$S.txt" "$NAME" "$F" >/dev/null 2>"$ROOT/re/out/check.err" || { cat "$ROOT/re/out/check.err"; exit 1; }
"$ROOT/re/.venv/bin/python" -I "$ROOT/re/harness/diff.py" "$ROOT/re/out/traces/$NAME.mame.trc" "$ROOT/re/out/traces/$NAME.$T.trc" "$@"
