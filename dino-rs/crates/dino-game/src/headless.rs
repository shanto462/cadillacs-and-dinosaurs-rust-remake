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
            // The GPU atlas wraps at its texture edges for an out of range code.
            let tile = d.tile as usize % (pens.len() / area);
            let pens = &pens[tile * area..][..area];
            let row = d.bank as usize * 32 + (d.palette & 31) as usize;
            for y in 0.max(d.y)..(SCREEN_H as i32).min(d.y + size) {
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
