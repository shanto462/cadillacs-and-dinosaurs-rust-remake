#!/bin/sh
# Step-mode regression over the recorded scenarios: prints the number of (class, field) pairs that differ in ported
# classes per scenario (0 is the goal), and the ported class lines of the report when there are some.
# Usage: re/harness/regress.sh [NAME ...]   (default: the list below; scenarios without a recording are skipped)
# A panic of the port is reported (check.sh stops on it), a scenario that ends with "?" pairs did not run.
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
C14="c14a c14b c14c c14d c14e c14f c14g c14h c15a"
for ch in hannah mustapha mess; do for s in c14a c14b c14c c14d c14e c14f c14g c14h; do C14="$C14 ${s}_$ch"; done; done
LIST=${*:-"stage1_jack stage1_late explore stage1_fight obstacle mp2 mp3 mp_join melee armed grabs grabs2 grabs3 intro c10a c10b c10c c20a c20b $C14"}
for n in $LIST; do
  [ -f "$ROOT/re/out/traces/$n.mame.trc" ] || { echo "$n: no recording"; continue; }
  case $n in
    intro) S=intro; F=600 ;; c10a) S=c10a; F=900 ;; c10b) S=c10b; F=1500 ;; c10c*) S=c10c; F=2400 ;;
    c20a) S=c20a; F=1800 ;; c20b*) S=c20b; F=2400 ;;
    c14a*) S=c14a; F=1200 ;; c14b*) S=c14b; F=2400 ;; c14c*) S=c14c; F=2400 ;; c14d*) S=c14d; F=2400 ;;
    c15a) S=c15a; F=300 ;;
    c14e*) S=c14e; F=1500 ;; c14f*) S=c14f; F=3000 ;; c14g*) S=c14g; F=1400 ;; c14h*) S=c14h; F=2000 ;;
    *) S=; F= ;;
  esac
  out=$(SCRIPT=$S FRAMES=$F "$ROOT/re/harness/check.sh" step "$n" 2>&1)
  pairs=$(echo "$out" | awk '/object fields of ported classes:/{print $6}')
  echo "$n: ported-class pairs = ${pairs:-?}"
  echo "$out" | awk '/^DIFFERENCES BY OBJECT CLASS/{p=1} /^SPRITE/{p=0} p' | grep -A1 '\[ported\]' | cut -c1-170
done
