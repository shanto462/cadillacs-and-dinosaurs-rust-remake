#!/bin/sh
# M6-03 to M6-08 (Mack, Shivat, the hunter, LASH T.): Mack (class 38), the critters (classes 78, 80) and Shivat (class 24, the Z marks class 30) against the M6-00 recordings
# (re/harness/pokes/m600_dinosaurs.txt) and the M6-05 recordings (re/harness/pokes/m605_shivat.txt), step mode. Every scenario differs from MAME only in
# frame 2 (the MAME-only spawn). The natural runs ep2_jack / ep2_3p differ from the first Poacher or Hunter (classes 13, 79, not ported) on.
# Usage: re/harness/m603_check.sh [NAME[:FRAMES] ...]     (default: the scenarios below)
# MODE=run runs the port free from the snapshot instead (MODE=follow copies MAME's random numbers). Each scenario runs in `step` mode by default (the port is loaded with MAME's state before every frame) and is compared with diff.py; printed:
# the summary line and the ported-class pairs.
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
LIST=${*:-"m600_a9_m_solo_l0:1100 m600_a9_m_solo_l31:1100 m600_a10_m_kill:700 m600_a11_m_vs_thug:700 m600_a16_s3_sleep:900 m600_a18_s8_a:700 m600_a18_s8_kill:700 m600_a19_s4_atk_8:330 m600_a19_s4_atk_14:330 m600_a19_s6_mash:330 m600_a21_s3_vs_enemy:1500 m605_grab_enemy:500 m605_grab_enemy_r:500 m605_wake_bullet:3000 m600_a17_s3_hunter:2000 m600_h1_hun_v400:900 m600_h2_hun_none:300 m600_h2_hun_hopper:600 m600_h2_hun_mack:600 m600_h3_nat2_hunt:3600 m600_h4_nat5_hunt:3600 m600_h19_lash_v4_l0:1800 m600_h19_lash_v4_l31:1500 m600_h10_ks_lash:400 m600_h25_pk_lash:1500 m607_fight_l0:3000 m607_fight_l16:3000 m607_fight_l31:3000"}
PY="$ROOT/re/.venv/bin/python -I"
mkdir -p "$ROOT/re/out/m603"
for item in $LIST; do
  n=${item%%:*}; f=${item##*:}; [ "$f" = "$item" ] && f=600
  [ -f "$ROOT/re/out/traces/$n.mame.trc" ] || { echo "$n: no recording"; continue; }
  IN="$ROOT/re/out/traces/$n.input.txt"; [ -f "$IN" ] || IN="$ROOT/re/harness/scripts/$n.txt"
  MODE=${MODE:-step} "$ROOT/re/harness/port.sh" "$IN" "$n" "$f" > "$ROOT/re/out/m603/$n.port.log" 2>&1 || { echo "$n: the port failed, see re/out/m603/$n.port.log"; continue; }
  if grep -q panicked "$ROOT/re/out/m603/$n.port.log"; then echo "$n: PANIC"; grep -A2 panicked "$ROOT/re/out/m603/$n.port.log" | head -4; continue; fi
  out=$($PY "$ROOT/re/harness/diff.py" "$ROOT/re/out/traces/$n.mame.trc" "$ROOT/re/out/traces/$n.$([ "${MODE:-step}" = run ] && echo port || echo "${MODE:-step}").trc" 2>&1)
  echo "$n: $(echo "$out" | awk '/^SUMMARY/{print}')"
  echo "$out" | awk '/object fields of ported classes/{print "   " $0}'
done
