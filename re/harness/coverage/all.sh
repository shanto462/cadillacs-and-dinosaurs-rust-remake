#!/bin/sh
# Every coverage batch, one after the other (one MAME process at a time), then the map. About three hours (2026-10-21).
#   re/harness/coverage/all.sh [JOBFILE ...]      (default: every job file in re/harness/coverage/jobs/, in the order below)
# The summary lines go to stdout; the details are in re/out/coverage/logs/. See REPORT.md here, "Method".
ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
J=$ROOT/re/harness/coverage/jobs
LIST=${*:-"$J/natural_ep1.txt $J/natural_ep2_4.txt $J/natural_ep5.txt $J/natural_ep6_7.txt $J/natural_ep8.txt $J/flows.txt $J/probes.txt $J/scenarios.txt"}
for f in $LIST; do
  echo "== $(basename "$f") $(date '+%H:%M:%S')"
  "$ROOT/re/harness/coverage/run.sh" "$f"
done
echo "== map $(date '+%H:%M:%S')"
"$ROOT/re/.venv/bin/python" -I "$ROOT/re/harness/coverage/map.py"
