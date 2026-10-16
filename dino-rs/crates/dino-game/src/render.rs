//! GPU renderer.
//!
//! * Tiles stay palette-indexed on the GPU: three RGBA8 atlases (16x16, 32x32, 8x8 tiles) whose
//!   red channel holds the pen (0-15).
//! * All palettes live in one small texture (16 colours x 128 rows: 4 banks x 32 palettes),
//!   uploaded from the game's palette memory when it changes. A fragment shader turns
//!   pen + palette row into a colour, so fades, flashes and palette swaps cost an 8 KB upload.
//! * Every quad carries a 16-bit pen mask: pens whose bit is clear are not drawn. That is how
//!   high-priority background pens are drawn a second time over the sprites.
//! * Everything is drawn as batched quads into a 384x224 render target, then scaled to the
//!   window with integer zoom.

use dino_assets::{AssetPack, SCREEN_H, SCREEN_W, TILES8_COUNT, colour};
use macroquad::prelude::*;

pub const PALETTE_ROWS: usize = 128;

/// Palette banks, in the order the original stores them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bank {
    Sprites = 0,
    Text = 1,
    Playfield = 2,
    Backdrop = 3,
}

/// Draw order from back to front.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    Backdrop,
    Playfield,
    /// Scroll1 when the layer control puts it behind the sprites (stage 6 elevator).
    TextBehind,
    Sprites,
    /// Background pens that the priority masks put in front of the sprites.
    PlayfieldFront,
    Hud,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Atlas {
    T16,
    T32,
    T8,
}

#[derive(Clone, Copy, Debug)]
pub struct TileDraw {
    pub layer: Layer,
    pub atlas: Atlas,
    /// Top-left corner in screen pixels.
    pub x: i32,
    pub y: i32,
    /// Index of the tile inside its atlas.
    pub tile: u32,
    pub bank: Bank,
    pub palette: u8,
    pub flip_x: bool,
    pub flip_y: bool,
    /// 0 = normal colours, 255 = solid white (hit flash).
    pub flash: u8,
    /// Pens to draw (bit n = pen n). 0xFFFF draws every pen except the transparent pen 15.
    pub mask: u16,
}

/// Draw commands shared by the GPU and the CPU screenshot renderer.
pub trait DrawTarget {
    fn upload_palettes(&mut self, words: &[u16]);
    fn tile(&mut self, draw: TileDraw);
    fn debug_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color);

    /// One sprite in the original attribute format, at a top-left screen position:
    /// attr bits 0-4 palette, 5 X flip, 6 Y flip, 8-11 block width-1, 12-15 block height-1.
    /// Blocks expand like the hardware: column i, row j -> (code & !0xF) + ((code + i) & 0xF) + 0x10 * j.
    fn sprite(&mut self, layer: Layer, x: i32, y: i32, code: u16, attr: u16, flash: u8) {
        let (w, h) = (((attr >> 8) & 0xF) as i32 + 1, ((attr >> 12) & 0xF) as i32 + 1);
        let (flip_x, flip_y) = (attr & 0x20 != 0, attr & 0x40 != 0);
        for j in 0..h {
            for i in 0..w {
                let (ti, tj) = (if flip_x { w - 1 - i } else { i }, if flip_y { h - 1 - j } else { j });
                let c = (code as u32 & !0xF) + ((code as u32 + ti as u32) & 0xF) + 0x10 * tj as u32;
                self.tile(TileDraw {
                    layer,
                    atlas: Atlas::T16,
                    x: x + 16 * i,
                    y: y + 16 * j,
                    tile: c & 0x7FFF,
                    bank: Bank::Sprites,
                    palette: (attr & 0x1F) as u8,
                    flip_x,
                    flip_y,
                    flash,
                    mask: 0xFFFF,
                });
            }
        }
    }

}

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying vec2 uv;
varying vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

const FRAGMENT: &str = r#"#version 100
precision highp float;
varying vec2 uv;
varying vec4 color;
uniform sampler2D Texture;
uniform sampler2D Palette;
void main() {
    float pen = floor(texture2D(Texture, uv).r * 255.0 + 0.5);
    if (pen > 14.5) discard;
    float m = pen < 7.5 ? floor(color.b * 255.0 + 0.5) : floor(color.a * 255.0 + 0.5);
    if (mod(floor(m / exp2(mod(pen, 8.0))), 2.0) < 0.5) discard;
    float row = floor(color.r * 255.0 + 0.5);
    vec4 c = texture2D(Palette, vec2((pen + 0.5) / 16.0, (row + 0.5) / 128.0));
    gl_FragColor = vec4(mix(c.rgb, vec3(1.0), color.g), 1.0);
}
"#;

struct AtlasTex {
    tex: Texture2D,
    size: u32,
    per_row: u32,
}

impl AtlasTex {
    /// Packs `count` square tiles of `size` pixels (row-major pens) into one texture.
    fn new(pens: &[u8], size: u32, count: u32, width: u32) -> AtlasTex {
        let per_row = width / size;
        let rows = count.div_ceil(per_row);
        let mut rgba = vec![0u8; (width * rows * size * 4) as usize];
        let area = (size * size) as usize;
        for t in 0..count {
            let (tx, ty) = ((t % per_row) * size, (t / per_row) * size);
            let src = &pens[t as usize * area..][..area];
            for y in 0..size {
                for x in 0..size {
                    let at = (((ty + y) * width + tx + x) * 4) as usize;
                    rgba[at] = src[(y * size + x) as usize];
                    rgba[at + 3] = 255;
                }
            }
        }
        let tex = Texture2D::from_rgba8(width as u16, (rows * size) as u16, &rgba);
        tex.set_filter(FilterMode::Nearest);
        AtlasTex { tex, size, per_row }
    }

    fn source(&self, tile: u32) -> Rect {
        let s = self.size as f32;
        Rect::new((tile % self.per_row) as f32 * s, (tile / self.per_row) as f32 * s, s, s)
    }
}

pub struct Gpu {
    t16: AtlasTex,
    t32: AtlasTex,
    t8: AtlasTex,
    palette_tex: Texture2D,
    material: Material,
    target: RenderTarget,
    draws: Vec<TileDraw>,
    rects: Vec<(Rect, Color)>,
}

impl Gpu {
    pub fn new(assets: &AssetPack) -> Gpu {
        let t16 = AtlasTex::new(&assets.tiles16, 16, assets.tile16_count(), 4096);
        let t32 = AtlasTex::new(&assets.tiles32, 32, (assets.tiles32.len() / 1024) as u32, 1024);
        let t8 = AtlasTex::new(&assets.tiles8, 8, TILES8_COUNT as u32 * 2, 1024);

        let palette_tex = Texture2D::from_rgba8(16, PALETTE_ROWS as u16, &vec![0u8; 16 * PALETTE_ROWS * 4]);
        palette_tex.set_filter(FilterMode::Nearest);

        let material = load_material(
            ShaderSource::Glsl { vertex: VERTEX, fragment: FRAGMENT },
            MaterialParams { textures: vec!["Palette".to_string()], ..Default::default() },
        )
        .expect("palette shader");
        material.set_texture("Palette", palette_tex.clone());

        let target = render_target(SCREEN_W, SCREEN_H);
        target.texture.set_filter(FilterMode::Nearest);

        Gpu { t16, t32, t8, palette_tex, material, target, draws: Vec::with_capacity(4096), rects: Vec::new() }
    }

    /// Uploads palette memory (2048 words: 4 banks x 32 palettes x 16 colours).
    pub fn upload_palettes(&mut self, words: &[u16]) {
        let bytes: Vec<u8> = words.iter().flat_map(|&w| colour(w)).collect();
        self.palette_tex.update_from_bytes(16, PALETTE_ROWS as u32, &bytes);
    }

    pub fn tile(&mut self, draw: TileDraw) {
        self.draws.push(draw);
    }

    pub fn sprite(&mut self, layer: Layer, x: i32, y: i32, code: u16, attr: u16, flash: u8) {
        DrawTarget::sprite(self, layer, x, y, code, attr, flash);
    }

    /// Debug outline drawn on top of everything (hit box viewer).
    pub fn debug_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        self.rects.push((Rect::new(x, y, w, h), color));
    }

    /// Renders the submitted frame and shows it on the window.
    pub fn present(&mut self) {
        let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, SCREEN_W as f32, SCREEN_H as f32));
        camera.render_target = Some(self.target.clone());
        set_camera(&camera);
        clear_background(BLACK);

        // Stable sort keeps submission order inside a layer.
        self.draws.sort_by_key(|d| d.layer);
        gl_use_material(&self.material);
        for d in &self.draws {
            let atlas = match d.atlas {
                Atlas::T16 => &self.t16,
                Atlas::T32 => &self.t32,
                Atlas::T8 => &self.t8,
            };
            let row = d.bank as u8 * 32 + (d.palette & 31);
            let s = atlas.size as f32;
            draw_texture_ex(
                &atlas.tex,
                d.x as f32,
                d.y as f32,
                Color::from_rgba(row, d.flash, d.mask as u8, (d.mask >> 8) as u8),
                DrawTextureParams {
                    source: Some(atlas.source(d.tile)),
                    dest_size: Some(vec2(s, s)),
                    flip_x: d.flip_x,
                    flip_y: d.flip_y,
                    ..Default::default()
                },
            );
        }
        gl_use_default_material();
        self.draws.clear();
        for (r, c) in self.rects.drain(..) {
            // Four 1-pixel bars: thin line primitives drop edges at this resolution.
            draw_rectangle(r.x, r.y, r.w + 1.0, 1.0, c);
            draw_rectangle(r.x, r.y + r.h, r.w + 1.0, 1.0, c);
            draw_rectangle(r.x, r.y, 1.0, r.h + 1.0, c);
            draw_rectangle(r.x + r.w, r.y, 1.0, r.h + 1.0, c);
        }

        set_default_camera();
        clear_background(BLACK);
        let zoom = ((screen_width() / SCREEN_W as f32).min(screen_height() / SCREEN_H as f32)).floor().max(1.0);
        let (w, h) = (SCREEN_W as f32 * zoom, SCREEN_H as f32 * zoom);
        draw_texture_ex(
            &self.target.texture,
            ((screen_width() - w) / 2.0).floor(),
            ((screen_height() - h) / 2.0).floor(),
            WHITE,
            // Render targets come back upside down (OpenGL texture origin is bottom-left).
            DrawTextureParams { dest_size: Some(vec2(w, h)), flip_y: true, ..Default::default() },
        );
    }
}

impl DrawTarget for Gpu {
    fn upload_palettes(&mut self, words: &[u16]) { Gpu::upload_palettes(self, words); }
    fn tile(&mut self, draw: TileDraw) { Gpu::tile(self, draw); }
    fn debug_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        Gpu::debug_rect(self, x, y, w, h, color);
    }
}
