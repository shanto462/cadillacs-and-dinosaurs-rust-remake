#!/bin/sh
# Step-mode regression over the recorded scenarios: prints the number of (class, field) pairs that differ in ported
# classes per scenario (0 is the goal), and the ported class lines of the report when there are some.
# Usage: re/harness/regress.sh [NAME ...]   (default: the list below; scenarios without a recording are skipped)
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
LIST=${*:-"stage1_jack stage1_late explore stage1_fight obstacle melee armed grabs grabs2 grabs3 intro c10a c10b c10c c20a c20b"}
for n in $LIST; do
  [ -f "$ROOT/re/out/traces/$n.mame.trc" ] || { echo "$n: no recording"; continue; }
  case $n in
    intro) S=intro; F=600 ;; c10a) S=c10a; F=900 ;; c10b) S=c10b; F=1500 ;; c10c*) S=c10c; F=2400 ;;
    c20a) S=c20a; F=1800 ;; c20b*) S=c20b; F=2400 ;; *) S=; F= ;;
  esac
  out=$(SCRIPT=$S FRAMES=$F "$ROOT/re/harness/check.sh" step "$n" 2>&1)
  pairs=$(echo "$out" | awk '/object fields of ported classes:/{print $6}')
  echo "$n: ported-class pairs = ${pairs:-?}"
  echo "$out" | awk '/^DIFFERENCES BY OBJECT CLASS/{p=1} /^SPRITE/{p=0} p' | grep -A1 '\[ported\]' | cut -c1-170
done
