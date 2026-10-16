#!/bin/sh
# M2-26 to M2-28: the story panels (mode 8) and the level start after them against the recordings of flow_links (the panel of an episode, the map panel and the
# episode card, the opening: 1100 frames from the poke of mode 8) and flow_panels (the panel by party, 330 frames; the attack skip, 400).
# Usage: re/harness/link_check.sh [NAME ...]   (default: link-ep2 .. link-ep8 and every pan-* recording)
# Each recording is cut at the record before its poke of the mode 8 (re/harness/link_prep.py, the name m228_NAME), the port gets the same pokes at frame 0
# (check.sh run) and the first difference is printed. FRAMES= sets the frames to run (default: all of the recording), STORY_ONLY=1 stops two records after the
# level starts (the intro task of the level is M2-26 / M2-27), SCRIPT= the input script.
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
LIST=${*:-"link-ep2 link-ep3 link-ep4 link-ep5 link-ep6 link-ep7 link-ep8 pan-skip-ep2 pan-skip-ep2-jump $(cd "$ROOT/re/out/traces" && ls flow_pan-ep*.mame.trc | sed 's/^flow_//; s/\.mame\.trc$//' | tr '\n' ' ')"}
for n in $LIST; do
  src=$ROOT/re/out/traces/flow_$n.mame.trc
  [ -f "$src" ] || { echo "$n: no recording"; continue; }
  prep=$("$ROOT/re/.venv/bin/python" -I "$ROOT/re/harness/link_prep.py" "$src" "m228_$n") || { echo "$n: prepare failed"; continue; }
  set -- $prep; length=$1; level=$2
  "$ROOT/re/.venv/bin/python" -I "$ROOT/re/harness/story_allow.py" "$ROOT/re/out/traces/m228_$n.mame.trc" > "$ROOT/re/out/traces/m228_$n.allow" || { echo "$n: no story panel"; continue; }
  fe=$(awk '/^global.next_event /{split($2,a,"-"); print a[2]}' "$ROOT/re/out/traces/m228_$n.allow")
  case $n in
    pan-skip-ep2) S=m228_skip ;;
    pan-skip-ep2-jump) S=m228_skipj ;;
    *) S=${SCRIPT:-m228_none} ;;
  esac
  out=$(SCRIPT=$S FRAMES=${FRAMES:-$([ -n "$STORY_ONLY" ] && echo $fe || echo $length)} "$ROOT/re/harness/check.sh" run "m228_$n" --allow "$ROOT/re/out/traces/m228_$n.allow" $CHECK_OPTS 2>&1)
  if echo "$out" | grep -q panicked; then echo "$n: PANIC $(echo "$out" | grep -A1 panicked | tr '\n' ' ' | cut -c1-200)"; continue; fi
  first=$(echo "$out" | awk '/^FIRST DIFFERENCE/{print $4; exit}')
  items=$(echo "$out" | awk '/^FIRST DIFFERENCE/{p=1;next} p&&/^$/{exit} p{print}' | head -3 | tr '\n' '|' | cut -c1-230)
  echo "$n: first difference at frame ${first:-none}: $items"
done
