#!/bin/sh
# Run the port headless from a MAME recording and write its trace. Usage (from anywhere):
#   [MODE=run|follow|step] re/harness/port.sh TEST_SCRIPT [NAME] [FRAMES]
#
#   MODE=run (default) free run from NAME.state: the port plays on its own      -> NAME.port.trc
#   MODE=follow        free run, but the random number state is copied from the
#                      MAME trace before every frame (objects of classes the port
#                      does not run call rand(), which the port cannot repeat)  -> NAME.follow.trc
#   MODE=step          before every frame the port is loaded with MAME's state
#                      after the previous frame: each frame is checked alone     -> NAME.step.trc
#
# NAME defaults to the script name without .txt, FRAMES to 600. Inputs come from the same
# files as record.sh (re/out/traces/NAME.state, NAME.mame.trc). See README.md.
set -e
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
SCRIPT=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
NAME=${2:-$(basename "$1" .txt)}
FRAMES=${3:-600}
OUT=$ROOT/re/out/traces
cd "$ROOT/dino-rs"
cargo build --release -q -p dino-game 2>/dev/null || cargo build --release -p dino-game
case "${MODE:-run}" in
  run)    OPTS="--load-state $OUT/$NAME.state";                                 TRACE=$OUT/$NAME.port.trc ;;
  follow) OPTS="--load-state $OUT/$NAME.state --follow-rng $OUT/$NAME.mame.trc"; TRACE=$OUT/$NAME.follow.trc ;;
  step)   OPTS="--step-from $OUT/$NAME.mame.trc";                               TRACE=$OUT/$NAME.step.trc ;;
  *) echo "MODE must be run, follow or step"; exit 1 ;;
esac
# shellcheck disable=SC2086
./target/release/dino-game --rom "$ROOT/dino.zip" $OPTS \
  --input-file "$SCRIPT" --frames "$FRAMES" --trace "$TRACE" 2>&1 | grep -v 'not ported (freed)'
