#!/bin/sh
# Coverage driver (re/harness/coverage/REPORT.md, "Method"): replays harness recordings in MAME with the debugger watching which
# 68000 addresses run. Usage, from anywhere:
#   re/harness/coverage/run.sh JOBS.txt [JOB ...]
# JOBS.txt has one job per line, `NAME: COMMAND` (# starts a comment line). COMMAND is a normal harness command
# (record.sh, record.sh --batch, probe_record.sh, ...) with paths relative to the repo root. It runs in a private
# copy of the root, re/out/coverage/root (links to re/harness, re/.venv, dino.zip and the MAME ROM folder), so the
# recordings it makes go to re/out/coverage/root/re/out/ and never touch re/out/traces. The shim bin/mame makes every
# MAME process of the job run under cov.lua. With JOB names only those jobs run.
# Output: re/out/coverage/hits/BATCH/NAME-N.hits (one per MAME process: the hex addresses that ran; a multi.lua process writes
# NAME-N.RECORDING.hits per recording, and the file of a recording that differs from its original goes to differ/),
# re/out/coverage/logs/BATCH/NAME.log, and one summary line per job on stdout. BATCH = the job file name.
# After each job every recording it made (re/out/coverage/root/re/out/traces/*.mame.trc) is compared with the recording of
# the same name in re/out/traces (same.py: all records equal), then emptied (they are large and nothing else reads them).
# Environment: COV_ADDRS (default re/out/coverage/addrs.txt, made from blocks.py on the first run), COV_KEEP=1 keeps
# the recordings of the private root.
set -e
ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
JOBS=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
shift
BATCH=$(basename "$JOBS" .txt)
OUT=$ROOT/re/out/coverage
R=$OUT/root
PY="$ROOT/re/.venv/bin/python -I"
mkdir -p "$OUT/logs/$BATCH" "$OUT/hits/$BATCH" "$R/re/out/traces" "$R/re/mame"
if [ -z "${COV_ADDRS:-}" ]; then
  COV_ADDRS=$OUT/addrs.txt
  if [ ! -s "$COV_ADDRS" ]; then
    [ -s "$OUT/blocks.txt" ] || $PY "$ROOT/re/harness/coverage/blocks.py" >/dev/null
    cut -d' ' -f1 "$OUT/blocks.txt" | sort -u > "$COV_ADDRS"
  fi
fi
for l in dino.zip re/harness re/.venv re/mame/roms re/ghidra re/out/dino_68k.bin; do
  [ -e "$R/$l" ] || ln -s "$ROOT/$l" "$R/$l"
done
export PATH=$ROOT/re/harness/coverage/bin:$PATH COV_ADDRS COV_HITS=$OUT/hits/$BATCH COV_ORIG=$ROOT/re/out/traces
cd "$R"
grep -v '^[[:space:]]*#' "$JOBS" | grep ':' | while IFS= read -r line; do
  name=${line%%:*}; cmd=${line#*:}
  if [ $# -gt 0 ]; then
    want=0; for j in "$@"; do [ "$j" = "$name" ] && want=1; done
    [ $want = 1 ] || continue
  fi
  t0=$(date +%s)
  status=ok
  COV_JOB=$name sh -c "$cmd" > "$OUT/logs/$BATCH/$name.log" 2>&1 < /dev/null || status=FAILED
  hits=$(cat "$OUT/hits/$BATCH/$name"-*.hits 2>/dev/null | sort -u | wc -l | tr -d ' ')
  # Every recording the job made is compared with the recording of the same name in re/out/traces, then emptied.
  for t in "$R"/re/out/traces/*.mame.trc; do
    [ -s "$t" ] || continue
    o=$COV_ORIG/$(basename "$t")
    if [ -s "$o" ]; then
      if ! $PY "$ROOT/re/harness/coverage/same.py" "$o" "$t" >> "$OUT/logs/$BATCH/$name.log" 2>&1; then
        # a recording of its own (multi.lua) that is not the original: its hits do not count
        for h in "$OUT/hits/$BATCH/$name"-*."$(basename "$t" .mame.trc)".hits; do
          [ -f "$h" ] && mkdir -p "$OUT/hits/$BATCH/differ" && mv "$h" "$OUT/hits/$BATCH/differ/"
        done
      fi
    else echo "no original: $(basename "$t")" >> "$OUT/logs/$BATCH/$name.log"; fi
    [ "${COV_KEEP:-0}" = 1 ] || : > "$t"
  done
  same=$(grep -cE '^(same|prefix): ' "$OUT/logs/$BATCH/$name.log" || true)
  diff=$(grep -cE '^DIFFERENT: ' "$OUT/logs/$BATCH/$name.log" || true)
  none=$(grep -cE '^no original: ' "$OUT/logs/$BATCH/$name.log" || true)
  echo "$name: $status, $(( $(date +%s) - t0 )) s, $hits addresses hit; replays equal to the recording $same, different $diff, without an original $none"
done
