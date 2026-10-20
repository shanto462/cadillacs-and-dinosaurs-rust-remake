#!/bin/sh
# Run the port headless from a MAME recording and write its trace. Usage (from anywhere):
#   [MODE=run|follow|step|lagsync] re/harness/port.sh TEST_SCRIPT [NAME] [FRAMES]
#
#   MODE=run (default) free run from NAME.state: the port plays on its own      -> NAME.port.trc
#   MODE=follow        free run, but the random number state is copied from the
#                      MAME trace before every frame (objects of classes the port
#                      does not run call rand(), which the port cannot repeat)  -> NAME.follow.trc
#   MODE=step          before every frame the port is loaded with MAME's state
#                      after the previous frame: each frame is checked alone     -> NAME.step.trc
#   MODE=lagsync       free run like run, but after every MAME lag record (flags & 1: the original's
#                      game frame overran a vblank) the port is loaded with MAME's state of that
#                      record, as step does for every frame. The drift is cut only at lag records.
#                      diff.py leaves the lag records out (as in step). The header counts the
#                      resyncs ("lagsync_resyncs")                               -> NAME.lagsync.trc
#
# NAME defaults to the script name without .txt, FRAMES to 600. Inputs come from the same
# files as record.sh (re/out/traces/NAME.state, NAME.mame.trc). See README.md.
#
# Pokes. If re/out/traces/NAME.pokes.txt exists (the sidecar of probe_record.sh: the holds, `god` and timed
# writes the probe made after the snapshot) the port repeats those writes at the same game frames
# (`--poke-file`, game clock: before the tick of frame k, after record k - 1). POKES=none leaves them out,
# POKES=FILE uses another file (the lines of MAME's DINO_POKES; `#!clock game` or `power`, see
# dino-rs/crates/dino-game/src/harness.rs `PokeSet`). PORT_OPTS adds options to the port, for example
# PORT_OPTS="--poke 1300:FF84D0:1:8" (power frames, as DINO_POKES).
set -e
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
SCRIPT=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
NAME=${2:-$(basename "$1" .txt)}
FRAMES=${3:-600}
OUT=$ROOT/re/out/traces
# PORT_OUT=DIR writes the port trace there instead of next to the MAME trace (re/out/traces is shared by every worktree: two workers
# with different code must not overwrite each other's .port / .follow / .step traces). diff.py takes both paths, so it reads either.
PO=${PORT_OUT:-$OUT}; mkdir -p "$PO"
cd "$ROOT/dino-rs"
cargo build --release -q -p dino-game 2>/dev/null || cargo build --release -p dino-game
case "${MODE:-run}" in
  run)    OPTS="--load-state $OUT/$NAME.state";                                 TRACE=$PO/$NAME.port.trc ;;
  follow) OPTS="--load-state $OUT/$NAME.state --follow-rng $OUT/$NAME.mame.trc"; TRACE=$PO/$NAME.follow.trc ;;
  step)   OPTS="--step-from $OUT/$NAME.mame.trc";                               TRACE=$PO/$NAME.step.trc ;;
  lagsync) OPTS="--load-state $OUT/$NAME.state --lagsync $OUT/$NAME.mame.trc"; TRACE=$PO/$NAME.lagsync.trc ;;
  *) echo "MODE must be run, follow, step or lagsync"; exit 1 ;;
esac
if [ -n "${POKES:-}" ]; then
  if [ "$POKES" != none ]; then
    [ -f "$POKES" ] || { echo "port.sh: no such poke file: $POKES" >&2; exit 2; }
    OPTS="$OPTS --poke-file $POKES"
  fi
elif [ -f "$OUT/$NAME.pokes.txt" ]; then
  # Task 18: the probe's MAME-only `spawn` notes become RAM writes (spawn_pokes.py, pool A) in a temporary file under re/out/,
  # so every step run replays the spawns; a sidecar without spawn notes is used as it is.
  POKE_FILE=$OUT/$NAME.pokes.txt
  if grep -q "spawn" "$POKE_FILE"; then
    SPAWN_POKES=$ROOT/re/out/$NAME.spawnpokes.txt
    "$ROOT/re/.venv/bin/python" -I "$ROOT/re/harness/spawn_pokes.py" "$OUT/$NAME.mame.trc" "$POKE_FILE" "$SPAWN_POKES" >/dev/null \
      && POKE_FILE=$SPAWN_POKES || echo "port.sh: spawn_pokes.py failed, the sidecar is used without the spawns" >&2
  fi
  OPTS="$OPTS --poke-file $POKE_FILE"
fi
# shellcheck disable=SC2086
./target/release/dino-game --rom "$ROOT/dino.zip" $OPTS ${PORT_OPTS:-} \
  --input-file "$SCRIPT" --frames "$FRAMES" --trace "$TRACE" 2>&1 | grep -v 'not ported (freed)'
