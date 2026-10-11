//! Animation viewer: plays any player animation with the ported frame tick and shows the body
//! (green) and attack (red) boxes. A learning tool and a check that extraction + port agree.
//!
//! Keys: Tab character, Left/Right set, Up/Down animation id, F flip, Space pause,
//! Period single step, B boxes on/off.

use crate::game::body::Body;
use crate::game::collide;
use crate::game::draw::{DrawCamera, SpriteList};
use crate::render::Gpu;
use crate::game::vram::PaletteRam;
use dino_assets::AssetPack;
use macroquad::prelude::*;

pub struct Viewer {
    character: usize,
    set: usize,
    anim: u16,
    body: Body,
    paused: bool,
    show_boxes: bool,
    sprites: SpriteList,
    pub label: String,
}

const NAMES: [&str; 4] = ["Jack", "Mustapha", "Hannah", "Mess"];

impl Viewer {
    /// `start` = (character, set, anim) for scripted screenshots (`--view c,s,a`).
    pub fn new(assets: &AssetPack, start: Option<(usize, usize, u16)>) -> Viewer {
        let (character, set, anim) = start.unwrap_or((0, 1, 0));
        let mut v = Viewer {
            character,
            set,
            anim,
            body: Body::default(),
            paused: false,
            show_boxes: true,
            sprites: SpriteList::default(),
            label: String::new(),
        };
        v.restart(assets);
        v
    }

    fn table(&self, a: &AssetPack) -> u32 {
        a.sprites.player_sets[self.set][self.character]
    }

    fn restart(&mut self, a: &AssetPack) {
        let s = &a.sprites;
        let table = self.table(a);
        let count = s.table(table).anims.len() as u16;
        self.anim = self.anim.min(count - 1);
        // Skip unused ids.
        while s.table(table).anims[self.anim as usize].is_none() && self.anim > 0 {
            self.anim -= 1;
        }
        self.body.id = self.character as u8;
        self.body.anim_table = table;
        self.body.box_table = s.player_box_tables[self.character];
        self.body.x = 192 << 16;
        self.body.y = 0;
        self.body.z = 40 << 16;
        self.body.start_anim(s, self.anim);
    }

    pub fn update(&mut self, a: &AssetPack) {
        let mut changed = false;
        if is_key_pressed(KeyCode::Tab) {
            self.character = (self.character + 1) % 4;
            changed = true;
        }
        if is_key_pressed(KeyCode::Right) {
            self.set = (self.set + 1) % 6;
            self.anim = 0;
            changed = true;
        }
        if is_key_pressed(KeyCode::Left) {
            self.set = (self.set + 5) % 6;
            self.anim = 0;
            changed = true;
        }
        if is_key_pressed(KeyCode::Up) {
            self.anim += 1;
            changed = true;
        }
        if is_key_pressed(KeyCode::Down) && self.anim > 0 {
            self.anim -= 1;
            changed = true;
        }
        if is_key_pressed(KeyCode::F) {
            self.body.facing ^= 1;
        }
        if is_key_pressed(KeyCode::B) {
            self.show_boxes = !self.show_boxes;
        }
        if is_key_pressed(KeyCode::Space) {
            self.paused = !self.paused;
        }
        if changed {
            self.restart(a);
        }
        if !self.paused || is_key_pressed(KeyCode::Period) {
            self.body.tick_anim(&a.sprites);
        }
        let f = a.sprites.frame(self.body.frame);
        self.label = format!(
            "{} set {} anim {}  frame {:#07x} type {:#x} dur {} body {:#x} atk {:#x}",
            NAMES[self.character], self.set, self.anim, f.rom_addr, f.kind, f.duration, f.body_box, f.attack_box
        );
    }

    /// OBJ palettes of stage 0 area 0 at full brightness (0x8E4C + 0x892C).
    pub fn palettes(a: &AssetPack) -> PaletteRam {
        let mut pal = PaletteRam::default();
        for (slot, &n) in a.obj_palette_sets[&(0, 0)].iter().enumerate() {
            let words: Vec<u16> = a.obj_palette_bank[n as usize].iter().map(|w| w | 0xF000).collect();
            pal.copy(PaletteRam::OBJ + slot * 16, &words);
        }
        pal
    }

    pub fn draw(&mut self, a: &AssetPack, gpu: &mut Gpu) {
        let cam = DrawCamera::new(0x40, 0);
        self.sprites.clear();
        self.sprites.draw_body(&a.sprites, &self.body, cam, 0);
        self.sprites.submit(gpu);
        if !self.show_boxes {
            return;
        }
        // Screen position of the feet: hardware Y (0x100 - (y + z)) minus the 16-line border.
        let feet_y = 0x100 - (self.body.yi() + self.body.zi()) as i32 - 16;
        let s = &a.sprites;
        for (index, colour) in [(self.body.body_box, GREEN), (self.body.attack_box, RED)] {
            if index == 0 {
                continue;
            }
            for h in collide::boxes(s, &self.body, index) {
                let x = collide::left(&self.body, &h) as i32 - 0x40 + 0x40 - cam.x as i32 - 0x40;
                let top = feet_y - (h.yoff + h.ysize) as i32 - h.zoff as i32;
                gpu.debug_rect(x as f32, top as f32, h.xsize as f32, h.ysize as f32, colour);
            }
        }
    }
}
