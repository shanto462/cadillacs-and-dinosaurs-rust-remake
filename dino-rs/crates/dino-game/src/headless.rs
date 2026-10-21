//! CPU screenshots. Uses the same draw commands and video latches as the GPU.

use std::io::BufWriter;
use std::path::Path;

use dino_assets::{AssetPack, SCREEN_H, SCREEN_W, colour};
use macroquad::prelude::Color;

use crate::game::Game;
use crate::render::{Atlas, DrawTarget, TileDraw};

pub struct Cpu<'a> {
    assets: &'a AssetPack,
    palettes: Vec<[u8; 4]>,
    draws: Vec<TileDraw>,
    rects: Vec<(i32, i32, i32, i32, [u8; 4])>,
    pixels: Vec<u8>,
}

impl<'a> Cpu<'a> {
    pub fn new(assets: &'a AssetPack) -> Self {
        Self {
            assets,
            palettes: vec![[0; 4]; 2048],
            draws: Vec::new(),
            rects: Vec::new(),
            pixels: vec![0; (SCREEN_W * SCREEN_H * 4) as usize],
        }
    }

    pub fn draw(&mut self, game: &mut Game) {
        crate::present::draw(game, self);
        self.rasterize();
    }

    /// The test menu over the picture: a dark box and the same lines as the window (lower case is drawn as capitals).
    pub fn menu(&mut self, menu: &mut crate::menu::Menu, god: bool) {
        use crate::menu::{FOOTER, Style};
        let (w, h) = (SCREEN_W as i32, SCREEN_H as i32);
        self.darken(6, 4, w - 12, h - 8, 0.88);
        self.text(14, 10, "Test menu", [255, 255, 255]);
        for (k, line) in menu.lines(19, god).iter().enumerate() {
            let col = match line.style {
                Style::Heading => [140, 205, 255],
                Style::Selected => [255, 217, 51],
                Style::Normal => [235, 235, 235],
            };
            self.text(14, 24 + 9 * k as i32, &line.text, col);
        }
        for (k, f) in FOOTER.iter().enumerate() {
            self.text(14, h - 24 + 9 * k as i32, f, [180, 180, 180]);
        }
    }

    /// Blacks out a box: each colour channel keeps `1 - alpha` of its value.
    fn darken(&mut self, x: i32, y: i32, w: i32, h: i32, alpha: f32) {
        for yy in y.max(0)..(y + h).min(SCREEN_H as i32) {
            for xx in x.max(0)..(x + w).min(SCREEN_W as i32) {
                let o = ((yy * SCREEN_W as i32 + xx) * 4) as usize;
                for c in &mut self.pixels[o..o + 3] {
                    *c = (*c as f32 * (1.0 - alpha)) as u8;
                }
            }
        }
    }

    /// Draws `s` with the 5 x 7 font (one pixel per dot), 6 pixels per character.
    fn text(&mut self, x: i32, y: i32, s: &str, rgb: [u8; 3]) {
        for (k, c) in s.chars().enumerate() {
            let Some(rows) = crate::font5x7::rows(c) else { continue };
            for (r, row) in rows.iter().enumerate() {
                for (col, ch) in row.bytes().enumerate() {
                    if ch != b'#' {
                        continue;
                    }
                    let (px, py) = (x + 6 * k as i32 + col as i32, y + r as i32);
                    if px < 0 || py < 0 || px >= SCREEN_W as i32 || py >= SCREEN_H as i32 {
                        continue;
                    }
                    let o = ((py * SCREEN_W as i32 + px) * 4) as usize;
                    self.pixels[o..o + 3].copy_from_slice(&rgb);
                }
            }
        }
    }

    fn rasterize(&mut self) {
        for p in self.pixels.chunks_exact_mut(4) {
            p.copy_from_slice(&[0, 0, 0, 255]);
        }
        // Stable order also preserves the sprite list order within each layer.
        self.draws.sort_by_key(|d| d.layer);
        for d in self.draws.drain(..) {
            let (pens, size) = match d.atlas {
                Atlas::T8 => (&self.assets.tiles8, 8),
                Atlas::T16 => (&self.assets.tiles16, 16),
                Atlas::T32 => (&self.assets.tiles32, 32),
            };
            let area = (size * size) as usize;
            if pens.len() < area {
                continue; // an empty atlas (a test's default assets)
            }
            // The GPU atlas wraps at its texture edges for an out of range code.
            let tile = d.tile as usize % (pens.len() / area);
            let pens = &pens[tile * area..][..area];
            let row = d.bank as usize * 32 + (d.palette & 31) as usize;
            // a piece of a tile (the row scroll of stage 2): only the lines r0 .. r1 of the tile
            let (r0, r1) = d.rows.map_or((0, size), |(a, b)| (a as i32, b as i32));
            for y in 0.max(d.y + r0)..(SCREEN_H as i32).min(d.y + r1) {
                let sy = if d.flip_y { size - 1 - (y - d.y) } else { y - d.y };
                for x in 0.max(d.x)..(SCREEN_W as i32).min(d.x + size) {
                    let sx = if d.flip_x { size - 1 - (x - d.x) } else { x - d.x };
                    let pen = pens[(sy * size + sx) as usize];
                    if pen == 15 || d.mask & (1 << pen) == 0 {
                        continue;
                    }
                    let mut c = self.palettes[row * 16 + pen as usize];
                    for ch in &mut c[..3] {
                        *ch = ((*ch as u32 * (255 - d.flash as u32) + 255 * d.flash as u32 + 127) / 255) as u8;
                    }
                    c[3] = 255;
                    let at = ((y as u32 * SCREEN_W + x as u32) * 4) as usize;
                    self.pixels[at..at + 4].copy_from_slice(&c);
                }
            }
        }
        for (x, y, w, h, c) in std::mem::take(&mut self.rects) {
            self.bar(x, y, w + 1, 1, c);
            self.bar(x, y + h, w + 1, 1, c);
            self.bar(x, y, 1, h + 1, c);
            self.bar(x + w, y, 1, h + 1, c);
        }
    }

    fn bar(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 4]) {
        for py in 0.max(y)..(SCREEN_H as i32).min(y + h) {
            for px in 0.max(x)..(SCREEN_W as i32).min(x + w) {
                let at = ((py as u32 * SCREEN_W + px as u32) * 4) as usize;
                self.pixels[at..at + 4].copy_from_slice(&c);
            }
        }
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();
        let write = || -> Result<(), Box<dyn std::error::Error>> {
            let file = BufWriter::new(std::fs::File::create(path)?);
            let mut encoder = png::Encoder::new(file, SCREEN_W, SCREEN_H);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header()?.write_image_data(&self.pixels)?;
            Ok(())
        };
        write().map_err(|e| format!("{}: {e}", path.display()))
    }
}

impl DrawTarget for Cpu<'_> {
    fn upload_palettes(&mut self, words: &[u16]) {
        self.palettes = words.iter().map(|&w| colour(w)).collect();
    }

    fn tile(&mut self, draw: TileDraw) {
        self.draws.push(draw);
    }

    fn debug_rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Color) {
        self.rects.push((x as i32, y as i32, w as i32, h as i32, c.into()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{Atlas, Bank, Layer};

    /// A tile drawn in pieces (`TileDraw::rows`, the row scroll of stage 2) is the same picture as the whole tile, also flipped in Y.
    #[test]
    fn a_tile_in_pieces_equals_the_whole_tile() {
        let mut a = AssetPack::default();
        // two 16x16 tiles; every pixel of a line has the pen (line + 3 * tile) % 14
        a.tiles16 = (0..2usize).flat_map(|t| (0..256usize).map(move |i| (((i / 16) + 3 * t) % 14) as u8)).collect();
        let a: &'static AssetPack = Box::leak(Box::new(a));
        let draw = |flip_y: bool, pieces: &[Option<(u8, u8)>]| {
            let mut cpu = Cpu::new(a);
            cpu.upload_palettes(&(0..2048u16).map(|i| i.wrapping_mul(0x111) | 0xF000).collect::<Vec<_>>());
            for &rows in pieces {
                cpu.tile(TileDraw {
                    layer: Layer::Playfield,
                    atlas: Atlas::T16,
                    x: 20,
                    y: 30,
                    tile: 1,
                    bank: Bank::Playfield,
                    palette: 2,
                    flip_x: false,
                    flip_y,
                    flash: 0,
                    mask: 0xFFFF,
                    rows,
                });
            }
            cpu.rasterize();
            cpu.pixels
        };
        for flip_y in [false, true] {
            let whole = draw(flip_y, &[None]);
            assert!(whole.iter().any(|&b| b != 0 && b != 255), "something is drawn");
            assert_eq!(draw(flip_y, &[Some((0, 5)), Some((5, 6)), Some((6, 16))]), whole, "flip_y {flip_y}");
            // one piece draws only its lines: the other lines of the tile stay background (black)
            let one = draw(flip_y, &[Some((4, 7))]);
            let px = |line: i32| ((line * SCREEN_W as i32 + 24) * 4) as usize;
            for line in 30..46 {
                let (a, b) = (px(line), px(line));
                if (34..37).contains(&line) {
                    assert_eq!(one[a..a + 4], whole[b..b + 4], "line {line} is drawn");
                } else {
                    assert_eq!(one[a..a + 4], [0, 0, 0, 255], "line {line} is not drawn");
                }
            }
        }
    }
}
