//! Pool A class 100 (`+$20 = 0x190`, handler 0x6A5F4): the two sprite pictures of a profile card
//! (M1-06). The card handlers 0x9A950 .. 0x9AB32 create two of them: variant 0 to 3 (the card) is
//! the big portrait of the hero on the left, variant 4 to 7 (the card + 4) is the small hero on the
//! right who performs a few moves; their animation is the animation of the same number in the table
//! 0x71F9E. The sounds of the moves come from the animation frames: every frame that holds a sound
//! code in its `+$2D` byte queues it in every frame it is shown.
//!
//! Mode 0 (0x6A608) sets up the object and goes on into mode 2 at once (0x6A6BE): the life of 300
//! updates (`+$A6`, the object frees itself when it reaches 0), the position from the scroll2 camera
//! (the portrait at +0x58 / +0x40, +0x50 for Mess, the small hero at +0x120 / +0x20), for the
//! portrait the palette (the 16 words of a table by card into OBJ palette 16, 0x6A6F6, and the
//! override `+$23 = 0x90`). Mode 2: the life, the sound of the frame, the animation step 0x121E and
//! the draw list (0x49CA). Mode 4 and 6 free it (0x6A734: jmp 0x48FC).

use super::lib32::put_hi;
use super::Enemy;
use crate::game::body::Body;
use crate::game::{Game, Handle};

/// Offsets of this class.
mod f {
    /// word: cleared (not used).
    pub const W_A4: usize = 0xA4;
    /// word: updates left (0x12C).
    pub const LIFE: usize = 0xA6;
}

/// The animation table of the cards' pictures.
const ANIM_TABLE: u32 = 0x71F9E;
/// The palette pointer table (one long per card, 0x6A724) and the OBJ palette the portrait uses (0x914200).
const PALETTE_POINTERS: u32 = 0x6A724;
const PALETTE_OBJ_16: usize = 16 * 16;

/// `add.w #d,$8(a6)` / `$c(a6)`: the integer word of X / Y.
fn add_x(b: &mut Body, d: i16) {
    let w = b.xi().wrapping_add(d);
    put_hi(&mut b.x, w);
}
fn add_y(b: &mut Body, d: i16) {
    let w = b.yi().wrapping_add(d);
    put_hi(&mut b.y, w);
}

impl Game {
    /// 0x6A5F4 class 100 (jump table 0x6A600: mode 0 -> 0x6A608, 2 -> 0x6A6BE, 4 and 6 -> 0x6A734).
    pub fn enemy_class100(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                self.enemy_c100_init(e);
                self.enemy_c100_frame(e, h); // 0x6A6BA: addq.b #2,4(a6), then 0x6A6BE
            }
            2 => self.enemy_c100_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x6A734: jmp 0x48FC
            }
        }
    }

    /// 0x6A608..0x6A6BA.
    fn enemy_c100_init(&mut self, e: &mut Enemy) {
        e.b.box_table = 0x103C00; // +$44
        e.b.multi_box = false; // +$4A
        e.b.atk = 0x0B; // +$58
        e.b.knock = 0; // +$5A
        e.b.hitstop = 0; // +$59
        e.b.serial = self.w.attack_serial; // +$5E
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1); // addq.b #1,$67BE(a5)
        e.b.live = 1;
        e.b.anim_table = ANIM_TABLE; // +$40
        e.b.draw_flags = 0x10; // +$25
        e.m.sb(super::off::ATTACKING, 0xFF); // +$7D
        e.b.b63 = 0xFF;
        e.b.hitstop = 0; // move.b #0,$59(a6)
        e.m.sw(f::W_A4, 0);
        e.m.sw(f::LIFE, 0x12C);
        let (cx, cy) = (self.cam.s2.cam_x as i16, self.cam.s2.cam_y as i16);
        put_hi(&mut e.b.x, cx); // move.w $744(a5),$8(a6)
        put_hi(&mut e.b.y, cy); // move.w $748(a5),$C(a6)
        add_x(&mut e.b, 0x50);
        add_y(&mut e.b, 0x40);
        let var = e.b.var26 as i16;
        if var > 3 {
            // the small hero
            add_x(&mut e.b, 0xD0);
            add_y(&mut e.b, -0x20);
        } else {
            // the portrait: Mess (3) does not move 8 to the right
            if var < 3 {
                add_x(&mut e.b, 8);
            }
            e.b.palette_override = 0x90; // +$23
            self.enemy_c100_palette(e.b.var26);
        }
        let anim = e.b.var26;
        self.enemy_anim(&mut e.b, anim); // 0x120E with d0 = +$26
        e.b.mode += 2; // addq.b #2,4(a6)
    }

    /// 0x6A6F6: OBJ palette 16 gets the 16 words of the card's palette under its brightness nibble.
    fn enemy_c100_palette(&mut self, var: u16) {
        let t = &self.assets.frontend.text;
        let at = t.u32(PALETTE_POINTERS, 4 * var as u32);
        let words: Vec<u16> = (0..16).map(|k| t.u16(at, 2 * k)).collect();
        let bank = self.pal.bank_mut(0);
        for (w, new) in bank[PALETTE_OBJ_16..PALETTE_OBJ_16 + 16].iter_mut().zip(words) {
            *w = (*w & 0xF000) | new;
        }
    }

    /// 0x6A6BE: the life, the sound of the frame, the animation, the draw list.
    fn enemy_c100_frame(&mut self, e: &mut Enemy, h: Handle) {
        let life = e.m.w(f::LIFE).wrapping_sub(1);
        e.m.sw(f::LIFE, life);
        if life == 0 || (life as i16) < 0 {
            let i = self.enemy_index(h);
            return self.enemy_free_pool_a(e, i); // 0x6A734
        }
        if e.b.param_b != 0 {
            self.sound(e.b.param_b as u16); // 0xA63C with d1 = d2 = 0
        }
        self.enemy_step(&mut e.b); // jsr 0x121E
        self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use std::path::Path;
    use std::sync::OnceLock;

    /// The asset pack of the user's ROM (`../dino.zip`), `None` when it is not there.
    fn assets() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// A game with one class 100 object of variant `var` in slot 23 (the card handlers' spawn).
    fn game(var: u16) -> Game {
        let mut g = Game::new(assets().unwrap());
        g.flow_profile_spawn_a(100, var);
        g
    }

    /// The portrait (variants 0 to 3) stands at the scroll2 camera + 0x58 / 0x40 (Mess, 3: 8 less to the
    /// right), uses OBJ palette 16 (override 0x90) and loads its colours under the brightness it has; the
    /// small hero (4 to 7) stands at + 0x120 / 0x20 and keeps its own palettes.
    #[test]
    fn the_pictures_stand_where_the_card_puts_them() {
        if assets().is_none() {
            return;
        }
        for (var, x, y, ovr) in [(0u16, 0x58, 0x40, 0x90u8), (1, 0x58, 0x40, 0x90), (2, 0x58, 0x40, 0x90), (3, 0x50, 0x40, 0x90), (4, 0x120, 0x20, 0), (7, 0x120, 0x20, 0)] {
            let mut g = game(var);
            g.pal.words[256..272].fill(0xF000); // OBJ palette 16 has its brightness
            g.cam.s2.cam_x = 0;
            g.enemies_update();
            let b = &g.enemies.slots[23].b;
            assert_eq!((b.xi(), b.yi(), b.palette_override, b.anim_table), (x, y, ovr, 0x71F9E), "variant {var}");
            let loaded = g.pal.words[256..272].iter().any(|&w| w & 0x0FFF != 0);
            assert_eq!(loaded, var < 4, "variant {var} loads its palette");
            assert!(g.pal.words[256..272].iter().all(|&w| var >= 4 || w & 0xF000 == 0xF000), "the brightness stays");
        }
        // the camera moves the picture with it
        let mut g = Game::new(assets().unwrap());
        (g.cam.s2.cam_x, g.cam.s2.cam_y) = (0x100, 0x20);
        g.flow_profile_spawn_a(100, 4);
        g.enemies_update();
        assert_eq!((g.enemies.slots[23].b.xi(), g.enemies.slots[23].b.yi()), (0x100 + 0x120, 0x20 + 0x20));
    }

    /// The object lives 300 updates (`+$A6` from 0x12C) and frees itself in the 300th; it draws in
    /// every update before that, and a frame with a sound code queues it in every update it is shown.
    #[test]
    fn it_lives_300_updates_and_queues_the_sound_of_its_frame() {
        if assets().is_none() {
            return;
        }
        let mut g = game(4);
        let mut sounds = Vec::new();
        for n in 1..=300 {
            g.w.sounds.clear();
            g.enemies_update();
            g.draw_list.clear();
            sounds.extend(g.w.sounds.iter().map(|&s| (n, s)));
            assert_eq!(g.enemies.slots[23].b.live != 0, n < 300, "update {n}");
        }
        assert!(sounds.iter().all(|&(_, s)| s >= 0x13), "{sounds:?}");
        assert!(!sounds.is_empty(), "the small hero's moves have sounds");
    }
}
