//! Turns the game state into GPU draw commands: the three tile layers from the tilemap
//! buffers, the sprite list, and the priority overlay. This is presentation only; the game
//! logic never reads anything back from here.

use crate::game::Game;
use crate::game::camera::layer_origin;
use crate::game::vram::Vram;
use crate::render::{Atlas, Bank, DrawTarget, Layer, TileDraw};
use dino_assets::{TILES8_COUNT, TILES32_FIRST};

pub fn draw(g: &mut Game, gpu: &mut impl DrawTarget) {
    if g.pal.dirty {
        gpu.upload_palettes(g.pal.shown_words());
        g.pal.dirty = false;
    }
    let scroll1_behind = g.w.layer_ctrl == 0x06D6;
    // The screen shows the game a few frames late (`camera::VIDEO_LATENCY`): the tile layers are drawn from
    // the scroll registers, which hold the camera shadows of two frames ago, the sprite list is the one of
    // two frames ago (`SpriteList::submit`, so the sprites stay in step with the tiles) and the colours
    // are the ones of the frame before (`PaletteRam::latch`). The tile maps themselves are shown at once.
    let scroll = g.cam.shown_scroll();
    draw_scroll3(g, gpu, scroll.s3);
    draw_scroll2(g, gpu, scroll.s2);
    draw_scroll1(g, gpu, scroll.cam1, if scroll1_behind { Layer::TextBehind } else { Layer::Hud });
    g.sprites.submit(gpu);
}

/// Scroll3: 32x32 tiles, codes 0x1C00..0x1FFF (others are hidden by the bank mapper).
fn draw_scroll3(g: &Game, gpu: &mut impl DrawTarget, cam: (u16, u16)) {
    let (ox, oy) = layer_origin(cam.0, cam.1, 0x700);
    let (c0, r0) = ((ox & 0x7FF) as i32 >> 5, (oy & 0x7FF) as i32 >> 5);
    let (fx, fy) = ((ox & 31) as i32, (oy & 31) as i32);
    for r in 0..8 {
        for c in 0..13 {
            let cell = g.vram.scroll3[Vram::index3(((c0 + c) & 63) as usize, ((r0 + r) & 63) as usize)];
            let code = cell.code & 0x3FFF;
            if !(TILES32_FIRST..TILES32_FIRST + 0x400).contains(&code) {
                continue;
            }
            gpu.tile(TileDraw {
                layer: Layer::Backdrop,
                atlas: Atlas::T32,
                x: c * 32 - fx,
                y: r * 32 - fy,
                tile: (code - TILES32_FIRST) as u32,
                bank: Bank::Backdrop,
                palette: (cell.attr & 0x1F) as u8,
                flip_x: cell.attr & 0x20 != 0,
                flip_y: cell.attr & 0x40 != 0,
                flash: 0,
                mask: 0xFFFF,
            });
        }
    }
}

/// Scroll2: 16x16 tiles, codes 0x4000..0x6FFF. Tiles whose priority group mask is nonzero are
/// drawn again over the sprites with only the masked pens.
fn draw_scroll2(g: &Game, gpu: &mut impl DrawTarget, cam: (u16, u16)) {
    let (ox, oy) = layer_origin(cam.0, cam.1, 0x300);
    let (c0, r0) = ((ox & 0x3FF) as i32 >> 4, (oy & 0x3FF) as i32 >> 4);
    let (fx, fy) = ((ox & 15) as i32, (oy & 15) as i32);
    for r in 0..15 {
        for c in 0..25 {
            let cell = g.vram.scroll2[Vram::index2(((c0 + c) & 63) as usize, ((r0 + r) & 63) as usize)];
            if !(0x4000..0x7000).contains(&cell.code) {
                continue;
            }
            let mut d = TileDraw {
                layer: Layer::Playfield,
                atlas: Atlas::T16,
                x: c * 16 - fx,
                y: r * 16 - fy,
                tile: cell.code as u32,
                bank: Bank::Playfield,
                palette: (cell.attr & 0x1F) as u8,
                flip_x: cell.attr & 0x20 != 0,
                flip_y: cell.attr & 0x40 != 0,
                flash: 0,
                mask: 0xFFFF,
            };
            gpu.tile(d);
            let front = g.w.priority_masks[((cell.attr >> 7) & 3) as usize];
            if front != 0 {
                d.layer = Layer::PlayfieldFront;
                d.mask = front;
                gpu.tile(d);
            }
        }
    }
}

/// Scroll1: 8x8 text tiles; the column parity picks the left or right half of the 16x8 cell.
fn draw_scroll1(g: &Game, gpu: &mut impl DrawTarget, cam: (u16, u16), layer: Layer) {
    let (ox, oy) = layer_origin(cam.0, cam.1, 0x100);
    let (c0, r0) = ((ox & 0x1FF) as i32 >> 3, (oy & 0x1FF) as i32 >> 3);
    let (fx, fy) = ((ox & 7) as i32, (oy & 7) as i32);
    for r in 0..29 {
        for c in 0..49 {
            let col = ((c0 + c) & 63) as usize;
            let cell = g.vram.scroll1[Vram::index1(col, ((r0 + r) & 63) as usize)];
            if cell.code >= TILES8_COUNT || cell.code == 0 {
                continue;
            }
            gpu.tile(TileDraw {
                layer,
                atlas: Atlas::T8,
                x: c * 8 - fx,
                y: r * 8 - fy,
                tile: cell.code as u32 * 2 + (col & 1) as u32,
                bank: Bank::Text,
                palette: (cell.attr & 0x1F) as u8,
                flip_x: cell.attr & 0x20 != 0,
                flip_y: cell.attr & 0x40 != 0,
                flash: 0,
                mask: 0xFFFF,
            });
        }
    }
}
