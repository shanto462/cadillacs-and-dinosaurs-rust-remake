//! Object drawing, ported from the sprite list builder (0x13FA6) and the per-object draw
//! routines (0x141F8, 0x14230, 0x142AA, 0x14494, 0x14180).
//!
//! The original writes hardware sprite entries (x, y, code, attr) where the first entry has the
//! highest priority. We keep exactly that list, then hand it to the GPU renderer back to front.

use super::Game;
use super::Handle;
use super::body::Body;
use crate::render::{Gpu, Layer};
use dino_assets::{Composition, FrameDraw, SpriteData};

#[derive(Clone, Copy, Debug)]
pub struct ObjEntry {
    pub x: i16,
    pub y: i16,
    pub code: u16,
    pub attr: u16,
    /// Not in the original entry: white hit flash amount for the renderer.
    pub flash: u8,
}

/// Camera base used by the draw code: $69B6 = camera X - 0x40, $69B8 = camera Y.
#[derive(Clone, Copy, Debug, Default)]
pub struct DrawCamera {
    pub x: i16,
    pub y: i16,
}

impl DrawCamera {
    pub fn new(camera_x: i16, camera_y: i16) -> DrawCamera {
        DrawCamera { x: camera_x.wrapping_sub(0x40), y: camera_y }
    }
}

#[derive(Default)]
pub struct SpriteList {
    pub entries: Vec<ObjEntry>,
}

impl SpriteList {
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Draws one object's current frame (0x14230 and the paths it dispatches to).
    pub fn draw_body(&mut self, s: &SpriteData, b: &Body, cam: DrawCamera, flash: u8) {
        let mut d4 = b.xi();
        let mut d5 = b.yi().wrapping_add(b.zi());
        if b.draw_flags & 0x80 == 0 {
            d4 = d4.wrapping_sub(cam.x);
            d5 = d5.wrapping_sub(cam.y);
        }
        let frame = s.frame(b.frame);
        match &frame.draw {
            FrameDraw::Comp { main, overlay } => {
                // Main composition first (on top), then the overlay.
                let d5s = screen_y(d5);
                self.draw_comp(s.comp(*main), b, d4, d5s, flash);
                if let Some(o) = overlay {
                    self.draw_comp(s.comp(*o), b, d4, d5s, flash);
                }
            }
            FrameDraw::Compound(parts) => {
                // 0x14180: each part moves X by +-dx and the height by dy, then draws.
                for p in parts {
                    let x = if b.facing != 0 { d4.wrapping_sub(p.dx) } else { d4.wrapping_add(p.dx) };
                    let y = d5.wrapping_add(p.dy);
                    self.draw_comp(s.comp(p.comp), b, x, screen_y(y), flash);
                }
            }
        }
    }

    /// 0x142BE (normal format) / 0x14494 (alt format): place every entry of a composition.
    fn draw_comp(&mut self, c: &Composition, b: &Body, d4: i16, d5: i16, flash: u8) {
        for e in &c.entries {
            let mut attr = e.attr;
            if b.palette_override & 0x80 != 0 {
                attr = (attr & 0xFFE0) | (b.palette_override as u16 & 0x1F);
            }
            let width = if c.alt { ((attr >> 4) & 0xF0) as i16 + 16 } else { 16 };
            let x = if b.facing == 0 {
                d4.wrapping_add(e.dx)
            } else {
                attr ^= 0x20;
                d4.wrapping_sub(e.dx).wrapping_sub(width)
            };
            // Cull exactly like the original: it only skips output.
            if x > 0x1C0 || (x <= 0x40 && (width as u16) < (0x40 - x) as u16) {
                continue;
            }
            self.entries.push(ObjEntry { x, y: d5.wrapping_add(e.dy), code: e.code, attr, flash });
        }
    }

    /// Hands the list to the GPU, last entry first, so the first entry ends on top.
    pub fn submit(&self, gpu: &mut Gpu) {
        for e in self.entries.iter().rev() {
            let sy = (e.y as i32) & 0x1FF;
            let sy = if sy >= 0x1F0 { sy - 0x200 } else { sy };
            gpu.sprite(Layer::Sprites, e.x as i32 - 0x40, sy - 16, e.code, e.attr, e.flash);
        }
    }
}

/// `d5 = ((d5 & 0xFF00) | (!d5 & 0x00FF)) + 1`, i.e. (0x100 - v) mod 0x200 for the usual range:
/// turns "height above the bottom" into a hardware sprite Y.
pub fn screen_y(d5: i16) -> i16 {
    (((d5 as u16 & 0xFF00) | (!d5 as u16 & 0x00FF)) as i16).wrapping_add(1)
}

impl Game {
    /// 0x141F8: draw one object of the sorted draw list (grab pairs are drawn as a pair).
    pub fn draw_object(&mut self, h: Handle, cam: DrawCamera) {
        let body = self.body(h).clone();
        self.sprites.draw_body(&self.assets.sprites, &body, cam, 0);
    }

    /// 0x14EC0: dynamic shadows for the draw list, after all objects (so below them). Not ported yet.
    pub fn shadow_pass(&mut self, _list: &[u32], _cam: DrawCamera) {}
}
