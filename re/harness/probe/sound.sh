#!/bin/sh
# Record QSound register writes for one or more sound requests (M4-01).
# One headless MAME process runs the whole list. Traces go to re/out/sound/ (git-ignored).
#
#   re/harness/probe/sound.sh 0x00 0x30
#   re/harness/probe/sound.sh 0x30:0:0 0x66:0x10:0
#   SQ_INPUT=requests.txt re/harness/probe/sound.sh
#
# A request file line is `sound CODE [d1] [d2]` (the same command the task names).
# Numbers are decimal or 0x hex. d1 and d2 default to 0.
# Each argument is CODE or CODE:d1:d2.
#
# SQ_FRAMES   video frames after the ring write (default 240, about 4 seconds)
# SQ_TIMEOUT  seconds for the MAME process (default 600)
# SQ_SOUND    force the MAME -sound value (default: try none, then auto)
#
# -sound none still runs the Z80. This script checks that the QSound ports were written
# during boot. If they were not, it starts the same batch once more with -sound auto
# and SDL_AUDIODRIVER=dummy, so the device is clocked and no audio device is opened.
set -e
ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
OUT=$ROOT/re/out/sound
MDIR=$OUT/qs-mame
REQ=$OUT/qs-batch.txt
STATUS=$OUT/qs-status.txt
LOG=$OUT/qs-mame.log

abspath() { echo "$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"; }

mkdir -p "$OUT"
: >"$REQ"
if [ -n "$SQ_INPUT" ]; then
  SQ_INPUT=$(abspath "$SQ_INPUT")
  cat "$SQ_INPUT" >>"$REQ"
fi
for arg in "$@"; do
  code=$(printf '%s' "$arg" | cut -d: -f1)
  d1=$(printf '%s' "$arg" | cut -d: -f2 -s)
  d2=$(printf '%s' "$arg" | cut -d: -f3 -s)
  printf 'sound %s %s %s\n' "$code" "${d1:-0}" "${d2:-0}" >>"$REQ"
done
if ! grep -q '[^[:space:]]' "$REQ"; then
  echo "usage: sound.sh CODE [CODE ...]    or    SQ_INPUT=file sound.sh" >&2
  echo "       CODE is 0x30 or 48; CODE:d1:d2 sets the two longs. File lines: sound CODE [d1] [d2]" >&2
  exit 2
fi

run_mame() {
  mode=$1
  rm -rf "$MDIR"
  mkdir -p "$MDIR/cfg" "$MDIR/nvram" "$MDIR/sta"
  export SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy
  export SQ_OUT=$OUT SQ_REQ=$REQ SQ_STATUS=$STATUS SQ_SOUND=$mode
  export SQ_FRAMES="${SQ_FRAMES:-240}"
  cd "$ROOT"
  # Fresh EEPROM (the nvram directory was just created). No window, no joystick.
  perl -e 'alarm shift; exec @ARGV' "${SQ_TIMEOUT:-600}" \
    mame dino -rompath ".;re/mame/roms" -video none -sound "$mode" -nothrottle \
    -keyboardprovider none -mouseprovider none -joystickprovider none -lightgunprovider none \
    -cfg_directory "$MDIR/cfg" -nvram_directory "$MDIR/nvram" -state_directory "$MDIR/sta" \
    -snapshot_directory "$MDIR/snap" -diff_directory "$MDIR/diff" \
    -autoboot_script "$ROOT/re/harness/probe/sound_tap.lua" >"$LOG" 2>&1 || true
  rm -rf "$MDIR/sta"
}

status_field() {
  awk -v k="$1" '$1 == k { print $2; exit }' "$STATUS" 2>/dev/null || true
}

if [ -n "$SQ_SOUND" ]; then
  run_mame "$SQ_SOUND"
else
  run_mame none
  writes=$(status_field qsound_writes)
  err=$(awk '$1 == "error" { $1 = ""; sub(/^ /, ""); print; exit }' "$STATUS" 2>/dev/null || true)
  if [ "${writes:-0}" -eq 0 ] || printf '%s' "$err" | grep -q 'no QSound'; then
    echo "sound.sh: -sound none kept the Z80 from writing the QSound ports ($err). Retrying with -sound auto." >&2
    run_mame auto
  fi
fi

if [ ! -f "$STATUS" ]; then
  echo "sound.sh: MAME wrote no status file. See $LOG" >&2
  exit 1
fi
cat "$STATUS"
err=$(awk '$1 == "error" { $1 = ""; sub(/^ /, ""); print; exit }' "$STATUS" || true)
if [ -n "$err" ]; then
  echo "sound.sh: $err" >&2
  exit 1
fi
awk '$1 == "code" { printf "sound %s: %s lines, flushed %s\n", $2, $4, $6 }' "$STATUS"
