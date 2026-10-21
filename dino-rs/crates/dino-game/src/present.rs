//! Turns the game state into GPU draw commands: the three tile layers from the tilemap
//! buffers, the sprite list, and the priority overlay. This is presentation only; the game
//! logic never reads anything back from here.

use crate::game::Game;
use crate::game::camera::{LayerRegs, layer_origin};
use crate::game::vram::Vram;
use crate::render::{Atlas, Bank, DrawTarget, Layer, TileDraw};
use dino_assets::{SCREEN_H, TILES8_COUNT, TILES32_COUNT};

pub fn draw(g: &mut Game, gpu: &mut impl DrawTarget) {
    if g.pal.dirty {
        gpu.upload_palettes(g.pal.shown_words());
        g.pal.dirty = false;
    }
    // The screen shows the game a few frames late (`camera::VIDEO_LATENCY`): the tile layers are drawn from
    // the scroll registers, which hold the camera X shadows of two frames ago and the Y of the frame before
    // (`Camera::shown_scroll`), with the layer control, the video control and the scroll3 base of the frame
    // before (`Camera::shown_layers`: the vblank IRQ writes them after each frame); the sprite list is the one
    // of two frames ago (`SpriteList::submit`, so the sprites stay in step with the tiles) and the colours are
    // the ones of the frame before (`PaletteRam::latch`). The tile maps and the priority masks (written in the
    // frame, 0x8D92) are shown at once.
    let live = LayerRegs { layer_ctrl: g.w.layer_ctrl, video_ctrl: g.w.video_ctrl, scroll3_base: g.w.scroll3_base };
    let regs = g.cam.shown_layers().unwrap_or(live);
    let scroll = g.cam.shown_scroll();
    draw_scroll3(g, gpu, scroll.s3, regs);
    draw_scroll2(g, gpu, scroll.s2, regs.video_ctrl & 1 != 0);
    draw_scroll1(g, gpu, scroll.cam1, if regs.layer_ctrl == 0x06D6 { Layer::TextBehind } else { Layer::Hud });
    g.sprites.submit(gpu);
}

/// Scroll3: 32x32 tiles, codes 0..0x1FFF of the code masked with 0x3FFF (`TILES32_COUNT`: every code the graphics ROM holds).
fn draw_scroll3(g: &Game, gpu: &mut impl DrawTarget, cam: (u16, u16), regs: LayerRegs) {
    let layer = if regs.layer_ctrl == 0x1396 { Layer::BackdropFront } else { Layer::Backdrop };
    let (ox, oy) = layer_origin(cam.0, cam.1, 0x700);
    let (c0, r0) = ((ox & 0x7FF) as i32 >> 5, (oy & 0x7FF) as i32 >> 5);
    let (fx, fy) = ((ox & 31) as i32, (oy & 31) as i32);
    for r in 0..8 {
        for c in 0..13 {
            let cells = if regs.scroll3_base == 0x9200 && regs.layer_ctrl == 0x1396 && !g.vram.intro3.is_empty() { &g.vram.intro3 } else { &g.vram.scroll3 };
            let cell = cells[Vram::index3(((c0 + c) & 63) as usize, ((r0 + r) & 63) as usize)];
            let code = cell.code & 0x3FFF;
            if code >= TILES32_COUNT {
                continue;
            }
            gpu.tile(TileDraw {
                layer,
                atlas: Atlas::T32,
                x: c * 32 - fx,
                y: r * 32 - fy,
                tile: code as u32,
                bank: Bank::Backdrop,
                palette: (cell.attr & 0x1F) as u8,
                flip_x: cell.attr & 0x20 != 0,
                flip_y: cell.attr & 0x40 != 0,
                flash: 0,
                mask: 0xFFFF,
                rows: None,
            });
        }
    }
}

/// Scroll2: 16x16 tiles, codes 0x4000..0x6FFF. Tiles whose priority group mask is nonzero are
/// drawn again over the sprites with only the masked pens.
///
/// With the row scroll on (`row_scroll`: video control bit 0, stage 2 only) screen line s shows the layer shifted by `rowscroll[(s + 16) & 0x3FF]`
/// more (CPS-A register 0x108 = 0x9158, offset register 0x120 always 0; the rows of the hardware are the screen lines). The lines
/// are drawn in bands of equal offsets, a tile that crosses a band in pieces (`TileDraw::rows`). Without row scroll, or with
/// every offset of the 224 lines zero, the whole screen is one band and the tiles are drawn whole, as before.
fn draw_scroll2(g: &Game, gpu: &mut impl DrawTarget, cam: (u16, u16), row_scroll: bool) {
    let offsets: Vec<u16> = (0..SCREEN_H as usize).map(|s| g.vram.rowscroll[(s + 16) & 0x3FF]).collect();
    if !row_scroll || offsets.iter().all(|&o| o == 0) {
        draw_scroll2_band(g, gpu, cam, None);
        return;
    }
    let mut top = 0;
    while top < offsets.len() {
        let mut end = top + 1;
        while end < offsets.len() && offsets[end] == offsets[top] {
            end += 1;
        }
        draw_scroll2_band(g, gpu, (cam.0.wrapping_add(offsets[top]), cam.1), Some((top as i32, end as i32)));
        top = end;
    }
}

/// The scroll2 tiles for the origin `cam`; with `band` = screen lines `y0 .. y1` only those lines (see `draw_scroll2`).
fn draw_scroll2_band(g: &Game, gpu: &mut impl DrawTarget, cam: (u16, u16), band: Option<(i32, i32)>) {
    let (ox, oy) = layer_origin(cam.0, cam.1, 0x300);
    let (c0, r0) = ((ox & 0x3FF) as i32 >> 4, (oy & 0x3FF) as i32 >> 4);
    let (fx, fy) = ((ox & 15) as i32, (oy & 15) as i32);
    for r in 0..15 {
        let rows = match band {
            None => None,
            Some((y0, y1)) => {
                // the lines of this tile row that lie in the band (tile row r covers the screen lines r * 16 - fy ..)
                let (a, b) = ((y0 - (r * 16 - fy)).max(0), (y1 - (r * 16 - fy)).min(16));
                if a >= b {
                    continue;
                }
                (a != 0 || b != 16).then_some((a as u8, b as u8))
            }
        };
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
                rows,
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
                rows: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::stage::Cell;
    use dino_assets::AssetPack;

    /// Collects the draw commands.
    #[derive(Default)]
    struct Rec {
        tiles: Vec<TileDraw>,
    }

    impl DrawTarget for Rec {
        fn upload_palettes(&mut self, _: &[u16]) {}
        fn tile(&mut self, draw: TileDraw) {
            self.tiles.push(draw);
        }
        fn debug_rect(&mut self, _: f32, _: f32, _: f32, _: f32, _: macroquad::prelude::Color) {}
    }

    /// A game whose scroll2 is full of one visible tile.
    fn game() -> Game {
        let mut g = Game::new(Box::leak(Box::new(AssetPack::default())));
        g.vram.scroll2.fill(Cell { code: 0x4100, attr: 0 });
        g
    }

    fn scroll2(g: &Game) -> Vec<TileDraw> {
        let mut r = Rec::default();
        draw_scroll2(g, &mut r, (0, 0), g.w.video_ctrl & 1 != 0);
        r.tiles
    }

    /// Without the row scroll bit, or with the 224 lines all at 0, the tiles are drawn whole as always.
    #[test]
    fn without_row_scroll_the_tiles_are_whole() {
        let mut g = game();
        for line in 0..224 {
            g.vram.rowscroll[line + 16] = 5;
        }
        let t = scroll2(&g); // the bit is off: the table is not looked at
        assert_eq!(t.len(), 15 * 25);
        assert!(t.iter().all(|d| d.rows.is_none() && d.x % 16 == 0));
        g.w.video_ctrl = 1;
        g.vram.rowscroll.fill(0);
        let t = scroll2(&g); // the bit is on and every offset is 0
        assert_eq!(t.len(), 15 * 25);
        assert!(t.iter().all(|d| d.rows.is_none() && d.x % 16 == 0));
    }

    /// Lines 128 to 159 with an offset of their own (the road of Episode 3): the lines above and below are whole tiles, each
    /// of the 32 lines is a one line piece of a tile with the X shifted by its own offset (the table word of line s is
    /// `rowscroll[s + 16]`).
    #[test]
    fn a_row_scroll_ramp_draws_one_line_pieces() {
        let mut g = game();
        g.w.video_ctrl = 1;
        for k in 0..32 {
            g.vram.rowscroll[0x90 + k] = k as u16 + 1; // lines 128 to 159: offsets 1 to 32
        }
        let t = scroll2(&g);
        let whole = t.iter().filter(|d| d.rows.is_none()).count();
        let pieces: Vec<&TileDraw> = t.iter().filter(|d| d.rows.is_some()).collect();
        // tile rows 0 to 7 (lines 0 to 127) and 10 to 13 (lines 160 to 223) are whole: 12 rows of 25 tiles, and
        // 32 one line pieces of 25 tiles
        assert_eq!((whole, pieces.len()), (12 * 25, 32 * 25));
        assert!(t.iter().filter(|d| d.rows.is_none()).all(|d| d.x % 16 == 0));
        // line 130 (offset 3): tile row 8, line 2 of the tile; the layer origin moves 3 pixels left
        let line130: Vec<&&TileDraw> = pieces.iter().filter(|d| d.y == 128 && d.rows == Some((2, 3))).collect();
        assert_eq!(line130.len(), 25);
        assert_eq!(line130[0].x, -3);
        assert_eq!(line130[1].x, 13);
        // line 159 (offset 32 = two whole tiles): the pieces are on the tile grid again
        let line159: Vec<&&TileDraw> = pieces.iter().filter(|d| d.y == 144 && d.rows == Some((15, 16))).collect();
        assert_eq!((line159.len(), line159[0].x), (25, 0));
    }

    /// A band that begins or ends inside a tile row draws only the lines it covers (the camera Y not on a tile row).
    #[test]
    fn a_band_inside_a_tile_row_is_cut() {
        let mut g = game();
        g.w.video_ctrl = 1;
        // lines 130 to 141 shifted by 16 (the same tile grid, other tiles): offsets as words of line + 16
        for line in 130..142 {
            g.vram.rowscroll[line + 16] = 16;
        }
        let t = scroll2(&g);
        let cut: Vec<&TileDraw> = t.iter().filter(|d| d.rows.is_some()).collect();
        // the row 8 is cut into 0..2 (offset 0), 2..14 (offset 16) and 14..16 (offset 0)
        let mut parts: Vec<(u8, u8)> = cut.iter().filter(|d| d.y == 128).map(|d| d.rows.unwrap()).collect();
        parts.sort();
        parts.dedup();
        assert_eq!(parts, vec![(0, 2), (2, 14), (14, 16)]);
    }
}
