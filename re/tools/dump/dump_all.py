"""Runs every asset dump: graphics, palettes, compositions, animations, stage maps, text, sound, gallery.
Reads the user's dino.zip at run time, writes only to re/out/assets/ (git-ignored). No emulator is used.

    re/tools/dump_assets.sh            (or: re/.venv/bin/python -I re/tools/dump/dump_all.py)

Options: --with-mame  also capture the front end screens with MAME (optional, one process; the existing captures in
                      re/out/assets/frontend are kept when this is not given)."""
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from romio import Rom, outdir
import catalog
import dump_comps
import dump_anims
import dump_stages
import dump_gfx
import dump_text
import dump_sound
import build_index


def main():
    t0 = time.time()
    rom = Rom()
    print("catalog: scanning animation tables ...", flush=True)
    cat, _ = catalog.build(rom)
    catalog.save(cat)
    print("compositions ...", flush=True)
    dump_comps.run(rom, cat)
    print("animations (GIF and strips) ...", flush=True)
    dump_anims.run(rom, cat)
    print("stage maps ...", flush=True)
    _, tile_use = dump_stages.run(rom)
    print("tile sheets and palettes ...", flush=True)
    dump_gfx.run(rom, tile_use, cat)
    print("text ...", flush=True)
    dump_text.run(rom)
    print("sound ...", flush=True)
    dump_sound.run(rom)
    import dump_frontend
    if "--with-mame" in sys.argv:
        dump_frontend.run()
    else:
        m = dump_frontend.index_existing()
        print(f"front end: {sum(len(x['frames']) for x in m)} existing screenshots indexed (no new capture)")
    print("gallery ...", flush=True)
    build_index.build()
    print(f"done in {time.time() - t0:.0f} s. Open {os.path.join(outdir(), 'index.html')}")


if __name__ == "__main__":
    main()
