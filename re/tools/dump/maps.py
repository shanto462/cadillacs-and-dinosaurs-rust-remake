"""Stage maps, background palettes and camera events (formats: re/specs/stage/SPEC.md sections 2 to 4)."""
import os
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from romio import N_AREAS

S2_HDR, S2_BLOCKS, S2_SCREENS = 0x21580, 0x21454, 0x2155E
S3_HDR, S3_SCREENS = 0x2311E, 0x177288
S1_SCREENS, S1_METATILES = 0x138C48, 0x15CFE8
SP2_SCREENS, SP2_BLOCKS = 0x145DC8, 0x175378       # fixed tables used by scenes outside the stages (0x204CA)
PAL_S1, PAL_S2, PAL_S3, PAL_OBJ = 0xBBA52, 0x8AE0, 0x8C52, 0x8E8E
EVENTS = 0x21B2A

CAMERA_MODES = {
    0x00: "free follow", 0x01: "end of area, fade", 0x02: "diagonal down", 0x03: "diagonal up", 0x04: "lock (fight)",
    0x05: "elevator ride", 0x06: "stage end", 0x07: "descend with players", 0x08: "sub-area switch", 0x09: "slow fade change",
    0x0A: "slope 1/4", 0x0B: "slope down", 0x0C: "creep up", 0x0D: "slope up", 0x0E: "debug stream", 0x0F: "debug stream",
    0x10: "wait then clear flag", 0x11: "auto scroll", 0x12: "area change no fade", 0x13: "stage end, fade",
}


def area_rec(rom, table, stage, area):
    r = rom.u32(table + 4 * stage) + 10 * area
    return dict(w=rom.b[r], h=rom.b[r + 1], screens=rom.u32(r + 2), x=rom.u16(r + 6), y=rom.u16(r + 8), addr=r)


def s2_bank_addr(rom, stage, area):
    return rom.u32(PAL_S2 + 4 * (stage * 4 + area))


def s3_bank_addr(rom, stage, area):
    return rom.u32(PAL_S3 + 4 * (stage * 4 + area))


def put(img, mask, pens, x, y, attr, pal):
    if attr & 0x20:
        pens = pens[:, ::-1]
    if attr & 0x40:
        pens = pens[::-1, :]
    h, w = pens.shape
    H, W = mask.shape
    if x < 0 or y < 0 or x + w > W or y + h > H:
        return
    opaque = pens != 15
    sub = img[y:y + h, x:x + w]
    sub[opaque] = pal[attr & 0x1F][pens][opaque]
    mask[y:y + h, x:x + w] |= opaque


def render_s2(rom, stage, area, pal, rec=None):
    rec = rec or area_rec(rom, S2_HDR, stage, area)
    W, H = rec["w"], rec["h"]
    blocks = rom.u32(S2_BLOCKS + 4 * stage)
    screens = rom.u32(S2_SCREENS + 4 * stage)
    img = np.zeros((H * 256, W * 256, 3), np.uint8)
    mask = np.zeros((H * 256, W * 256), bool)
    t16 = rom.t16
    used = {}
    for sx in range(W):
        for sy in range(H):
            sid = rom.u16(rec["screens"] + 2 * (sx * H + sy))
            oy = (H - 1 - sy) * 256
            for cx in range(8):
                for p in range(8):
                    blk = rom.u16(screens + sid * 128 + cx * 16 + p * 2)
                    ba = blocks + blk * 16
                    for j, (dx, dy) in enumerate(((0, 0), (0, 1), (1, 0), (1, 1))):
                        code, attr = rom.u16(ba + 4 * j), rom.u16(ba + 4 * j + 2)
                        if code == 0:
                            continue
                        used.setdefault(code, attr & 0x1F)
                        put(img, mask, t16[code & 0x7FFF], sx * 256 + cx * 32 + dx * 16, oy + p * 32 + dy * 16, attr, pal)
    return img, mask, used


def render_s3(rom, stage, area, pal, rec=None):
    rec = rec or area_rec(rom, S3_HDR, stage, area)
    W, H = rec["w"], rec["h"]
    img = np.zeros((H * 256, W * 256, 3), np.uint8)
    mask = np.zeros((H * 256, W * 256), bool)
    t32 = rom.t32
    used = {}
    for sx in range(W):
        for sy in range(H):
            sid = rom.u16(rec["screens"] + 2 * (sx * H + sy)) & 0x7F
            oy = (H - 1 - sy) * 256
            for cx in range(8):
                for p in range(8):
                    ea = S3_SCREENS + sid * 256 + cx * 32 + p * 4
                    code, attr = rom.u16(ea), rom.u16(ea + 2)
                    if code == 0:
                        continue
                    used.setdefault(code & 0x1FFF, attr & 0x1F)
                    put(img, mask, t32[code & 0x1FFF], sx * 256 + cx * 32, oy + p * 32, attr, pal)
    return img, mask, used


def s3_screen(rom, sid, pal, used=None):
    """One 256x256 scroll3 screen (id 0..127) with palette bank pal."""
    img = np.zeros((256, 256, 3), np.uint8)
    mask = np.zeros((256, 256), bool)
    for cx in range(8):
        for p in range(8):
            ea = S3_SCREENS + sid * 256 + cx * 32 + p * 4
            code, attr = rom.u16(ea), rom.u16(ea + 2)
            if code == 0:
                continue
            if used is not None:
                used.setdefault(code & 0x1FFF, attr & 0x1F)
            put(img, mask, rom.t32[code & 0x1FFF], cx * 32, p * 32, attr, pal)
    return img, mask


def s2_special_screen(rom, sid, pal, used=None):
    """Scroll2 screen from the fixed scene tables (0x145DC8 screens, 0x175378 blocks): 256x256."""
    img = np.zeros((256, 256, 3), np.uint8)
    mask = np.zeros((256, 256), bool)
    for cx in range(8):
        for p in range(8):
            blk = rom.u16(SP2_SCREENS + sid * 128 + cx * 16 + p * 2)
            ba = SP2_BLOCKS + blk * 16
            for j, (dx, dy) in enumerate(((0, 0), (0, 1), (1, 0), (1, 1))):
                code, attr = rom.u16(ba + 4 * j), rom.u16(ba + 4 * j + 2)
                if code == 0:
                    continue
                if used is not None:
                    used.setdefault(code & 0x7FFF, attr & 0x1F)
                put(img, mask, rom.t16[code & 0x7FFF], cx * 32 + dx * 16, p * 32 + dy * 16, attr, pal)
    return img, mask


def s1_screen(rom, sid, pal, used=None):
    """Scroll1 picture screen (0x138C48, metatiles 0x15CFE8): 256x256."""
    img = np.zeros((256, 256, 3), np.uint8)
    mask = np.zeros((256, 256), bool)
    for cx in range(8):
        for p in range(8):
            mt = rom.u16(S1_SCREENS + sid * 128 + cx * 16 + p * 2)
            for j in range(16):
                code, attr = rom.u16(S1_METATILES + mt * 64 + 4 * j), rom.u16(S1_METATILES + mt * 64 + 4 * j + 2)
                col, row = cx * 4 + j // 4, p * 4 + j % 4
                if used is not None:
                    used.setdefault(code, attr & 0x1F)
                put(img, mask, rom.t8[code & 0xFFFF][col & 1], col * 8, row * 8, attr, pal)
    return img, mask


def events(rom, stage):
    base = 0x22B9E if stage == 2 else EVENTS + rom.u16(EVENTS + stage * 2)
    out, a = [], base
    for _ in range(200):
        x = rom.u16(a)
        if x == 0xFFFF:
            break
        out.append(dict(index=len(out), addr=a, x=x, y=rom.u16(a + 2), push=rom.u16(a + 4), up=rom.u16(a + 6), down=rom.u16(a + 8),
                        area=rom.b[a + 10], mode=rom.b[a + 11], vfollow=rom.b[a + 12], flag70=rom.b[a + 13], nxt=rom.b[a + 14],
                        by_y=rom.b[a + 15]))
        a += 16
    return out


def s3_shift(rom, stage, area):
    return rom.b[0x22DE4 + rom.u16(0x22DE4 + 2 * stage) + area]


def s3_yfollow(rom, stage, area):
    return rom.b[0x22E54 + stage * 4 + area]


def s3_camera(rom, stage, area, camx, camy):
    sh = s3_shift(rom, stage, area)
    r2 = area_rec(rom, S2_HDR, stage, area)
    r3 = area_rec(rom, S3_HDR, stage, area)
    x3 = (camx & 0xFFFF) >> sh
    y3 = r3["y"]
    if s3_yfollow(rom, stage, area):
        y3 = ((camy - r2["y"]) >> sh) + r3["y"]
    return x3, y3


def view(layers, camx, camy):
    """384x224 composite: layers = [(img, mask, height_in_screens, camfn)] back to front."""
    scr = np.zeros((224, 384, 3), np.uint8)
    for img, mask, hs, camfn in layers:
        cx, cy = camfn(camx, camy)
        top = hs * 256 - 0xF0 - cy
        ys = np.arange(224) + top
        xs = np.arange(384) + cx
        vy = (ys >= 0) & (ys < img.shape[0])
        vx = (xs >= 0) & (xs < img.shape[1])
        sy = np.clip(ys, 0, img.shape[0] - 1)
        sx = np.clip(xs, 0, img.shape[1] - 1)
        pix = img[sy][:, sx]
        m = mask[sy][:, sx] & vy[:, None] & vx[None, :]
        scr[m] = pix[m]
    return scr
