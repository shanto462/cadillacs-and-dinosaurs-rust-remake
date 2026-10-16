#!/bin/sh
# Pictures of a cut run against the MAME snapshots of the same segment (M2-26 to M2-28).
# Usage: re/harness/shot_check.sh KIND NAME T1,T2,...      KIND = flow_links | flow_panels, NAME = link-ep2 | pan-ep4-jack | ...
# The port runs headless (no window) from re/out/traces/m228_NAME.state with its poke file (re/harness/link_check.sh makes both) and takes the pictures of the
# frames T and T+1 (the picture of the record T is the frame T + 1; a moving picture, a palette play, may be a frame off: the better one counts) in ONE run into re/out/shots/m228/NAME/. Each is compared pixel for pixel
# with re/out/flow/KIND/NAME/snap/ (the files 0000.png .. in the order of the `snaps` line of the segment, which must be the same list as T1,T2,...).
set -e
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
KIND=$1; NAME=$2; TIMES=$3
OUT=$ROOT/re/out/shots/m228/$NAME
mkdir -p "$OUT"
FRAMES=$(echo "$TIMES" | tr ',' '\n' | awk '{printf "%s%d,%d", (NR>1?",":""), $1, $1+1}')
"$ROOT/dino-rs/target/release/dino-game" --rom "$ROOT/dino.zip" --headless --load-state "$ROOT/re/out/traces/m228_$NAME.state" \
  --poke-file "$ROOT/re/out/traces/m228_$NAME.pokes.txt" ${INPUT:+--input-file "$INPUT"} --screenshots "$FRAMES" --shot-dir "$OUT"
"$ROOT/re/.venv/bin/python" -I - "$ROOT/re/out/flow/$KIND/$NAME/snap" "$OUT" "$TIMES" <<'PY'
import sys
from PIL import Image
import numpy as np
snap, out, times = sys.argv[1], sys.argv[2], [int(x) for x in sys.argv[3].split(",")]
for i, t in enumerate(times):
    m = np.array(Image.open(f"{snap}/{i:04d}.png").convert("RGB"))
    best = None
    for f in (t, t + 1):
        p = np.array(Image.open(f"{out}/f{f:05d}.png").convert("RGB"))
        n = int((m != p).any(axis=2).sum()) if m.shape == p.shape else -1
        if best is None or n < best[0]:
            best = (n, f)
    print(f"t = {t}: {'equal' if best[0] == 0 else str(best[0]) + ' pixels differ'} (frame {best[1]})")
PY
