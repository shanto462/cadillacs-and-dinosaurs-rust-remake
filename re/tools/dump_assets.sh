#!/bin/sh
# Extracts and renders every asset of the ROM set in dino.zip into re/out/assets/ (git-ignored), then builds the local gallery
# re/out/assets/index.html. Needs dino.zip in the repo root (or DINO_ROM=/path/to/dino.zip). No emulator is started.
#   re/tools/dump_assets.sh
set -e
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
exec "$ROOT/re/.venv/bin/python" -I "$ROOT/re/tools/dump/dump_all.py" "$@"
