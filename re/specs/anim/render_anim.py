#!/usr/bin/env python3
"""Render real Cadillacs and Dinosaurs animations to PNG strips, with hitboxes.

Usage (run with the project venv, isolated mode):
  python -I render_anim.py <S>                 # <S> = workspace root (roms/re). Render the default set of strips into <S>/specs/anim/png/
  python -I render_anim.py <S> sheet <table_hex> <rec 10|12> <boxtable_hex> <unused> <out.png>
                                               # contact sheet: every anim of an anim table, one row each

Inputs: <S>/out/dino_68k.bin (68000 program image) and <S>/out/gfx.npy (interleaved 4 MB graphics ROM).
Boxes: green = body box (frame byte +8 -> obj $48), red = box from frame byte +9 (obj $49) when the frame
type ($2E) is 4 (attack active), orange = same box when the type is something else (for example 2 = walk,
where the game uses it as the grab/catch probe).
"""
import os, sys
import numpy as np
from PIL import Image, ImageDraw

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dinolib import Rom, anim_frames, anim_table_entries, draw_sprites, PLAYER_ANIM_SETS, PLAYER_BOX_TABLES, CHAR_NAMES

BG = (48, 52, 72, 255)
CELL_W, CELL_H, FOOT_Y = 176, 150, 128     # cell size and the feet line inside a cell (unscaled pixels)


def frame_info(rom, f, rec):
    r = rom.b[f:f + rec]
    return dict(w0=rom.w(f), w1=rom.w(f + 2), b4=r[4], b5=r[5], ftype=r[6], dur=r[7] & 0x7F, end=bool(r[7] & 0x80),
                body=r[8], atk=r[9], extra=(rom.w(f + 10) if rec == 12 else None))


def draw_box(d, rom, table, idx, ox, oy, facing, colour):
    zoff, zsize, xoff, xsize, yoff, ysize = rom.box(table, idx)
    x0 = (-xoff - xsize) if facing else xoff          # same as 0x12954 / 0x14B66
    left, right = ox + x0, ox + x0 + xsize
    bottom, top = oy - yoff, oy - yoff - ysize       # +$0C (height) grows upward
    d.rectangle([left, top, right, bottom], outline=colour)
    # depth extent as a short bar under the feet: [zoff, zoff+zsize] relative to obj $10
    d.line([ox + x0, oy + 3 + (zoff % 4), ox + x0 + xsize, oy + 3 + (zoff % 4)], fill=colour)


def render_frames(rom, frames, rec, boxtable, pals, facing=0, labels=True, pal_override=None, show_boxes=True):
    n = len(frames)
    img = np.zeros((CELL_H, CELL_W * n, 4), np.uint8)
    img[:] = BG
    for i, f in enumerate(frames):
        ox, oy = CELL_W * i + CELL_W // 2, FOOT_Y
        draw_sprites(rom, img, rom.frame_sprites(f, facing, pal_override), ox, oy, pals)
    im = Image.fromarray(img)
    d = ImageDraw.Draw(im)
    for i, f in enumerate(frames):
        fi = frame_info(rom, f, rec)
        ox, oy = CELL_W * i + CELL_W // 2, FOOT_Y
        d.line([ox - 3, oy, ox + 3, oy], fill=(255, 255, 255, 255))      # object origin (X, feet)
        d.line([ox, oy - 3, ox, oy + 3], fill=(255, 255, 255, 255))
        if show_boxes and boxtable is not None:
            if fi['body']:
                draw_box(d, rom, boxtable, fi['body'], ox, oy, facing, (60, 255, 60, 255))
            if fi['atk']:
                col = (255, 50, 50, 255) if fi['ftype'] == 4 else (255, 170, 0, 255)
                draw_box(d, rom, boxtable, fi['atk'], ox, oy, facing, col)
        if labels:
            d.text((CELL_W * i + 3, 2), '%06X c%X' % (f, fi['w0'] & 0x7FFF), fill=(255, 255, 255, 255))
            d.text((CELL_W * i + 3, 12), 't%d T%X b%X a%X%s' % (fi['dur'], fi['ftype'], fi['body'], fi['atk'],
                                                              ' END' if fi['end'] else ''), fill=(255, 255, 160, 255))
    return im


def save_scaled(im, path, scale=2):
    im = im.resize((im.width * scale, im.height * scale), Image.NEAREST)
    bg = Image.new('RGBA', im.size, BG)
    bg.alpha_composite(im)
    bg.convert('RGB').save(path)


def player_anim(rom, char, set_idx, anim_id):
    tbl = rom.l(PLAYER_ANIM_SETS[set_idx] + 4 * char)
    return tbl + rom.w(tbl + 2 * anim_id)


def sheet(rom, tbl, rec, boxtable, pals, out, maxcols=14, ids=None):
    entries = anim_table_entries(rom, tbl)
    nz = [rom.w(tbl + 2 * i) for i in range(len(entries))]
    n = min(o for o in nz if o) // 2
    ids = ids if ids is not None else range(n)
    rows = []
    for i in ids:
        o = rom.w(tbl + 2 * i)
        if not o:
            continue
        fr, nxt = anim_frames(rom, tbl + o, rec)
        fr = fr[:maxcols]
        try:
            im = render_frames(rom, fr, rec, boxtable, pals)
        except Exception as e:      # malformed entry: keep going
            print('anim', i, 'failed', e)
            continue
        d = ImageDraw.Draw(im)
        d.text((2, CELL_H - 12), 'anim %d (0x%X) @%06X next=%s' % (i, i, tbl + o, ('%06X' % nxt) if nxt else '-'),
               fill=(120, 220, 255, 255))
        rows.append(im)
    W = max(r.width for r in rows)
    out_im = Image.new('RGBA', (W, sum(r.height for r in rows)), BG)
    y = 0
    for r in rows:
        out_im.paste(r, (0, y)); y += r.height
    save_scaled(out_im, out, 1)


def main():
    S = sys.argv[1]
    rom = Rom(os.path.join(S, 'out/dino_68k.bin'), os.path.join(S, 'out/gfx.npy'))
    pals, _ = rom.obj_palettes(0, 0)
    if len(sys.argv) > 2 and sys.argv[2] == 'sheet':
        tbl, rec, bt, out = int(sys.argv[3], 16), int(sys.argv[4]), int(sys.argv[5], 16), sys.argv[7]
        sheet(rom, tbl, rec, bt, pals, out)
        return
    outdir = os.path.join(S, 'specs/anim/png')
    os.makedirs(outdir, exist_ok=True)
    for name, char, seq, facing in DEFAULT_PLAYER_STRIPS:
        bt = rom.l(PLAYER_BOX_TABLES + 4 * char)
        frames = []
        for set_idx, anim_id in seq:
            start = player_anim(rom, char, set_idx, anim_id)
            fr, _ = anim_frames(rom, start, 12)
            frames += fr
        save_scaled(render_frames(rom, frames, 12, bt, pals, facing), os.path.join(outdir, name + '.png'))
        print('wrote', name, CHAR_NAMES[char], seq, len(frames), 'frames')
    for name, tbl, ids, bt, pal, facing in DEFAULT_ENEMY_STRIPS:
        frames = []
        for anim_id in ids:
            fr, _ = anim_frames(rom, tbl + rom.w(tbl + 2 * anim_id), 10)
            frames += fr
        save_scaled(render_frames(rom, frames, 10, bt, pals, facing, pal_override=pal), os.path.join(outdir, name + '.png'))
        print('wrote', name, 'anims', ids, len(frames), 'frames')


# (file name, character 0..3, [(anim set 0..5, anim id), ...] played back to back, facing)
# Set/anim ids come from the player state code (see SPEC.md section 6).
DEFAULT_PLAYER_STRIPS = [
    ('jack_idle', 0, [(0, 0)], 0),                         # set0 anim0 (14 records, 3 distinct poses)
    ('jack_walk', 0, [(0, 1)], 0),                         # set0 anim1, 12 x 4 ticks, loops
    ('jack_walk_right', 0, [(0, 1)], 1),                   # same, facing right (+$24 != 0)
    ('jack_punch_combo', 0, [(1, 0), (1, 0), (1, 3), (1, 4)], 0),   # jab, jab, hook, finisher (whiff versions)
    ('jack_punch_combo_onhit', 0, [(1, 0), (1, 2), (1, 3), (1, 5), (1, 4), (1, 6)], 1),  # versions shown after a hit
    ('jack_jump', 0, [(0, 3), (0, 21)], 0),                # jump, jump kick
    ('jack_hit', 0, [(2, 0), (2, 1), (2, 2), (2, 5)], 0),  # hit high, hit low, knocked down, get up
    ('mustapha_walk', 1, [(0, 1)], 0),
    ('hannah_walk', 2, [(0, 1)], 0),
    ('mess_walk', 3, [(0, 1)], 0),
    ('mustapha_combo', 1, [(1, 0), (1, 3), (1, 7)], 0),        # char 1 finisher = set1 anim 7 when $2C==0 (0x1F3A)
]
# (file name, anim table (obj +$40), [anim ids], box table (obj +$44), palette override or None, facing)
# Enemy init 0x3388A: type $3=0, $40=0x34892, $44=0x103000, $4A=0 (single boxes), 10-byte frame records.
DEFAULT_ENEMY_STRIPS = [
    ('enemy34892_idle_walk', 0x34892, [0, 1], 0x103000, None, 0),
    ('enemy34892_attacks', 0x34892, [3, 4, 5], 0x103000, None, 1),
    ('enemy34892_run_compound', 0x34892, [6], 0x103000, None, 0),   # word0 < 0: compound frames (0x14180)
    ('enemy34892_knockdown', 0x34892, [14], 0x103000, None, 0),
]

if __name__ == '__main__':
    main()
