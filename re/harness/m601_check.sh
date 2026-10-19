#!/bin/sh
# M6-01 / M6-02: the Poachers (class 13: POACHER J., SKINNER, GUTTER) against the M6-00 recordings of D3a H5 to H10 (re/harness/pokes/m600_humans.txt).
# Usage: re/harness/m601_check.sh [NAME[:FRAMES] ...]     (default: the scenarios below, in `step` mode)
# MODE=run runs the port free from the snapshot instead (a spawn of the probe cannot update the free list of the port, so only scenarios where the poacher makes no object are meaningful).
# Printed per scenario: the summary line of diff.py and the ported-class pairs. Frames: the number of records of the recording (m600_h9: 360).
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
H9="m600_h9_p13_ent02:360 m600_h9_p13_ent04:360 m600_h9_p13_ent06:360 m600_h9_p13_ent08:360 m600_h9_p13_ent0a:360 m600_h9_p13_ent0c:360 m600_h9_p13_ent0e:360 m600_h9_p13_ent10:360 m600_h9_p13_ent12:360"
H6="m600_h6_rf_m0:900 m600_h6_rf_m0_l31:900 m600_h7_rf_m1:900 m600_h7_rf_m2:900 m600_h7_rf_m3:900 m600_h7_rf_m4:900 m600_h7_rf_m5:900"
H8="m600_h8_p13_v0_l0:1500 m600_h8_p13_v0_l31:1500 m600_h8_p13_v2_l0:1500 m600_h8_p13_v2_l31:1500 m600_h8_p13_v4_l0:1500 m600_h8_p13_v4_l31:1500"
H10="m600_h10_ks_p13v0:400 m600_h10_ks_p13v2:400 m600_h10_ks_p13v4:400"
# scenarios with an unported class (the Hopper foot, the barrel, Mack, the Butcher group): read the lines of class 0x0d of the report, the other classes are tasks
OTHER="m600_h14_h_foot_poacher:900 m600_h15_h_barrel_poacher:600 m600_h16_h_barrel_poacher:900 m600_h25_pk_poacher:1500 m600_h25_pk_skinner:1500 m600_h25_pk_gutter:1500 m600_h5_bsp_ctrl:120 m600_h5_bsp_mack:300"
# re/harness/pokes/m601_poachers.txt and m601_far.txt (recorded by this task): the swing at a container (0x48), the approach (0xE), the flee timer
M601="m601_barrel:261 m601_pail:261 m601_far_a:1200 m601_far_b:1200 m601_far_c:1200 m601_far_d:1200 m601_far_e:1200 m601_far_f:1200 m601_walk_away:1200"
for k in 00 01 02 03 04 05 06 07 08 09 10 11; do M601="$M601 m601_far$k:420"; done
LIST=${*:-"$H9 $H6 $H8 $H10 $M601 $OTHER"}
PY="$ROOT/re/.venv/bin/python -I"
mkdir -p "$ROOT/re/out/m601"
for item in $LIST; do
  n=${item%%:*}; f=${item##*:}; [ "$f" = "$item" ] && f=600
  [ -f "$ROOT/re/out/traces/$n.mame.trc" ] || { echo "$n: no recording"; continue; }
  IN="$ROOT/re/out/traces/$n.input.txt"; [ -f "$IN" ] || IN="$ROOT/re/harness/scripts/$n.txt"
  # the port repeats the spawn of the probe (spawn_pokes.py): without it the init frame of the poacher would be loaded from MAME.
  # A recording without the sidecar (an old scenario) runs as it is.
  SIDE="$ROOT/re/out/traces/$n.pokes.txt"
  if [ -f "$SIDE" ]; then
    $PY "$ROOT/re/harness/spawn_pokes.py" "$ROOT/re/out/traces/$n.mame.trc" "$SIDE" "$ROOT/re/out/m601/$n.pokes.txt" 2>/dev/null || { echo "$n: spawn_pokes failed"; continue; }
    PK="$ROOT/re/out/m601/$n.pokes.txt"
  else
    PK=
  fi
  POKES="$PK" MODE=${MODE:-step} "$ROOT/re/harness/port.sh" "$IN" "$n" "$f" > "$ROOT/re/out/m601/$n.port.log" 2>&1 || { echo "$n: the port failed, see re/out/m601/$n.port.log"; continue; }
  if grep -q panicked "$ROOT/re/out/m601/$n.port.log"; then echo "$n: PANIC"; grep -A2 panicked "$ROOT/re/out/m601/$n.port.log" | head -4; continue; fi
  case ${MODE:-step} in step) T=step ;; follow) T=follow ;; *) T=port ;; esac
  out=$($PY "$ROOT/re/harness/diff.py" "$ROOT/re/out/traces/$n.mame.trc" "$ROOT/re/out/traces/$n.$T.trc" 2>&1)
  echo "$n: $(echo "$out" | awk '/^SUMMARY/{print}')"
  echo "$out" | awk '/object fields of ported classes/{print "   " $0}'
done
