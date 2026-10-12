#!/bin/sh
# Record a MAME reference trace. Usage (from anywhere):
#   re/harness/record.sh TEST_SCRIPT [NAME] [FRAMES]
# Writes re/out/traces/NAME.state (RAM at the sync point) and re/out/traces/NAME.mame.trc.
# NAME defaults to the script file name without .txt, FRAMES to 600.
# Optional environment: DINO_BOOT (boot script, default scripts/boot_jack.txt), DINO_SYNC
# (control, control+N or vblank:N), DINO_SNAP_EVERY (PNG every N frames into re/out/mame/snap/dino/).
# DINO_CLOCK=video records every video frame from boot (default game skips slowdown).
# MAME's own files (cfg, nvram, screenshots, record.log) go to re/out/mame/. See README.md.
set -e
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
SCRIPT=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
NAME=${2:-$(basename "$1" .txt)}
FRAMES=${3:-600}
OUT=$ROOT/re/out/traces
MAMEDIR=$ROOT/re/out/mame
mkdir -p "$OUT" "$MAMEDIR"
# A fresh EEPROM every run, so every recording boots the same way.
rm -rf "$MAMEDIR/nvram"
cd "$ROOT"
export DINO_BOOT=${DINO_BOOT:-$ROOT/re/harness/scripts/boot_jack.txt}
export DINO_INPUT=$SCRIPT DINO_FRAMES=$FRAMES
export DINO_TRACE=$OUT/$NAME.mame.trc DINO_STATE=$OUT/$NAME.state
# No Dock icon or focus change on macOS: SDL with dummy drivers never registers MAME as a GUI app.
export SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy
mame dino -rompath ".;re/mame/roms" -video none -sound none -nothrottle \
  -cfg_directory "$MAMEDIR/cfg" -nvram_directory "$MAMEDIR/nvram" \
  -snapshot_directory "$MAMEDIR/snap" -diff_directory "$MAMEDIR/diff" \
  -autoboot_script "$ROOT/re/harness/record.lua" >"$MAMEDIR/record.log" 2>&1 || true
grep 'record.lua' "$MAMEDIR/record.log" || { echo "MAME failed, see $MAMEDIR/record.log"; exit 1; }
ls -l "$OUT/$NAME.state" "$OUT/$NAME.mame.trc"
