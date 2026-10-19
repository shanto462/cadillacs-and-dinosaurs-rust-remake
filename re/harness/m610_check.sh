#!/bin/sh
# M6-10: the Butcher (classes 5, 51, 52, 53, 58, 61) against the recordings of re/harness/pokes/m610_butcher.txt (the Butcher alone: his group
# timer is held, so no Poacher draws random numbers in his frames) and the M6-00 / M5-16 recordings that have him.
# Usage: re/harness/m610_check.sh [NAME[:FRAMES] ...]     (default: the scenarios below)
# Record first (one MAME process, about 40 seconds):  PR_SNAPS=none PR_SHEET=0 re/harness/probe/probe_record.sh re/harness/pokes/m610_butcher.txt all
# Each scenario runs in `step` mode and is compared with diff.py; printed: the summary line and the ported-class pairs. The m610 scenarios are
# equal in every frame (0 fields). The m600 / m516 scenarios have Poachers (class 13, not run by the port yet) that draw random numbers before the
# Butcher in the frame order, so their ported-class pairs start at the first group (about 1100 frames after the start).
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
LIST=${*:-"m610_b1_alone:6500 m610_b4_alone:7000 m610_c1_idle:4500 m610_c2_updown:4500 m610_c3_hannah:4500 m610_c4_mess:4500 m610_c5_jump:4500 m610_c6_far:4500 m610_c7_flee:4500"}
PY="$ROOT/re/.venv/bin/python -I"
mkdir -p "$ROOT/re/out/m610"
for item in $LIST; do
  n=${item%%:*}; f=${item##*:}; [ "$f" = "$item" ] && f=600
  [ -f "$ROOT/re/out/traces/$n.mame.trc" ] || { echo "$n: no recording (re/harness/probe/probe_record.sh re/harness/pokes/m610_butcher.txt all)"; continue; }
  IN="$ROOT/re/out/traces/$n.input.txt"; [ -f "$IN" ] || IN="$ROOT/re/harness/scripts/$n.txt"
  MODE=step "$ROOT/re/harness/port.sh" "$IN" "$n" "$f" > "$ROOT/re/out/m610/$n.port.log" 2>&1 || { echo "$n: the port failed, see re/out/m610/$n.port.log"; continue; }
  if grep -q panicked "$ROOT/re/out/m610/$n.port.log"; then echo "$n: PANIC"; grep -A2 panicked "$ROOT/re/out/m610/$n.port.log" | head -4; continue; fi
  # shellcheck disable=SC2086
  out=$($PY "$ROOT/re/harness/diff.py" "$ROOT/re/out/traces/$n.mame.trc" "$ROOT/re/out/traces/$n.step.trc" 2>&1)
  echo "$n: $(echo "$out" | awk '/^SUMMARY/{print}')"
  echo "$out" | awk '/object fields of ported classes/{print "   " $0}'
done
