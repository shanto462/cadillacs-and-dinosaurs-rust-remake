#!/bin/sh
# Run the port headless from a MAME snapshot and write its trace. Usage (from anywhere):
#   re/harness/port.sh TEST_SCRIPT [NAME] [FRAMES]
# Reads re/out/traces/NAME.state, writes re/out/traces/NAME.port.trc (see README.md).
set -e
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
SCRIPT=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
NAME=${2:-$(basename "$1" .txt)}
FRAMES=${3:-600}
OUT=$ROOT/re/out/traces
cd "$ROOT/dino-rs"
cargo build --release -q -p dino-game 2>/dev/null || cargo build --release -p dino-game
./target/release/dino-game --rom "$ROOT/dino.zip" --load-state "$OUT/$NAME.state" \
  --input-file "$SCRIPT" --frames "$FRAMES" --trace "$OUT/$NAME.port.trc" 2>&1 | grep -v 'not ported (freed)' || true
