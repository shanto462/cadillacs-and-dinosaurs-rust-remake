#!/bin/sh
# Run the original game in MAME with the probe (headless, no sound, about 20x real time).
#   re/harness/probe/probe.sh NAME
# ONE MAME process runs a whole batch: put every probe you need in one input file as a segment
# (`segment NAME` line, see capture-recipes.md) and run this once. Do not start MAME once per probe.
# Environment (all optional): PR_INPUT (file of key lines and directives), PR_POKES (second file),
#   PR_LINES (directives inline, separated by ';'), PR_SNAPS ("t1,t2,.." or "every:N:from:to"),
#   PR_FRAMES (frames to run after the jump, default 600), PR_WATCH ("N SEL fields;.."),
#   PR_SHEET=0 (do not make the contact sheets), PR_TIMEOUT (seconds, default 900).
# Output (git-ignored): re/out/probe/NAME/log.txt, snap/0000.png.., sheet.png, mame.log; for a batch
#   re/out/probe/NAME/SEGMENT/{log.txt,snap/,sheet.png}.
# Manual: plan/game/design/capture-recipes.md.
set -e
ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
NAME=${1:?usage: probe.sh NAME   (settings in PR_* environment variables, see the top of this file)}
abspath() { echo "$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"; }
if [ -n "$PR_INPUT" ]; then PR_INPUT=$(abspath "$PR_INPUT"); export PR_INPUT; fi
if [ -n "$PR_POKES" ]; then PR_POKES=$(abspath "$PR_POKES"); export PR_POKES; fi
OUT=$ROOT/re/out/probe/$NAME
rm -rf "$OUT"
mkdir -p "$OUT/mame"
export PR_OUTDIR=$OUT PR_NAME=$NAME
cd "$ROOT"
# Headless on purpose: no video, no sound, no input devices, SDL dummy drivers (no Dock icon, no focus).
# The EEPROM is fresh in every run (rm above), so every run boots the same way.
export SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy
perl -e 'alarm shift; exec @ARGV' "${PR_TIMEOUT:-900}" \
  mame dino -rompath ".;re/mame/roms" -video none -sound none -nothrottle \
  -keyboardprovider none -mouseprovider none -joystickprovider none -lightgunprovider none \
  -cfg_directory "$OUT/mame/cfg" -nvram_directory "$OUT/mame/nvram" -state_directory "$OUT/mame/sta" \
  -snapshot_directory "$OUT" -diff_directory "$OUT/mame/diff" \
  -autoboot_script "$ROOT/re/harness/probe/probe.lua" >"$OUT/mame.log" 2>&1 || true
rm -rf "$OUT/mame/sta"
for L in "$OUT"/log.txt "$OUT"/*/log.txt; do
  [ -f "$L" ] || continue
  D=$(dirname "$L")
  N=$(ls "$D/snap" 2>/dev/null | grep -c '\.png$' || true)
  echo "probe ${D#$ROOT/re/out/probe/}: $(tail -n 1 "$L")   screenshots: $N"
  if [ "$N" -gt 0 ] && [ "${PR_SHEET:-1}" != "0" ]; then
    "$ROOT/re/.venv/bin/python" -I "$ROOT/re/harness/probe/sheet.py" "$D" 2>&1 | sed 's/^/    /'
  fi
done
