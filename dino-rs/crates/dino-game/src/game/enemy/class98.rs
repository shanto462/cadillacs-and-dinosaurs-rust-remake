//! Pool A class 98 (`+$20 = 0x188`, handler 0x6A20A): the big logo of the title and press start
//! screens ("Cadillacs" and "Dinosaurs" letters made of sprites). M1-03.
//!
//! The variant (`+$26`) says which piece it is:
//!
//! | variant | piece | start (X from the scroll2 camera G.$744, Y from G.$748) | animation | life |
//! | --- | --- | --- | --- | --- |
//! | 0 | first word, slides in from the right | +0x174, +0xB0, depth 1, step -0x18 | 0 | 600 frames |
//! | 1 | second word, slides in from the left | -0x5C, +0x90, depth 0, step +0x18 | 1 | 600 frames |
//! | 2 | first word, fixed (press start screen) | 0xA0, 0xB0, depth 1 | 0 | until the pool is reset |
//! | 3 | second word, fixed (press start screen) | 0x7C, 0x90, depth 1 | 1 | until the pool is reset |
//! | 4 | a third piece that slides in from the right | 0x244, 0x80, depth 1, step -0x18 | 2 | 608 frames |
//! | 0x10, 0x11 | as 0 and 1, 0x20 pixels higher (`+$A4` = 0x20) | | | |
//!
//! A sliding piece moves for the 9 frames after its first one (`+$A2` counts down from 0x258) and
//! stays there; at `+$A2 == 0x24E` the title tune 0x88 is queued (variants 0 and 4 only), and at 0
//! the object frees itself. Mode 0 initialises and runs the first frame at once.

use super::lib32::put_hi;
use super::Enemy;
use crate::game::body::Body;
use crate::game::{Game, Handle};

/// Offsets of this class.
mod f {
    /// word: X step per frame while sliding in.
    pub const STEP: usize = 0xA0;
    /// word: frames to live (counts down from 0x258).
    pub const LIFE: usize = 0xA2;
    /// word: how far the piece is raised (0x20 for variants 0x10 and 0x11).
    pub const RAISE: usize = 0xA4;
}

/// `add.w #d,$8(a6)`: the integer word of X, the fraction stays.
fn add_x(b: &mut Body, d: i16) {
    let w = b.xi().wrapping_add(d);
    put_hi(&mut b.x, w);
}

/// `add.w #d,$c(a6)`: the integer word of Y.
fn add_y(b: &mut Body, d: i16) {
    let w = b.yi().wrapping_add(d);
    put_hi(&mut b.y, w);
}

impl Game {
    /// The spawn code of 0x9A4E4 / 0x979CA: `jsr 0x483C`, `+$00 = 1`, `+$20 = 0x188`, `+$26 = variant`.
    /// A full pool loses the object (the original would write through a0 = 0; the title never
    /// has more than a few objects).
    pub fn enemy_spawn_logo(&mut self, variant: u16) {
        let Some(i) = self.enemies.alloc() else { return };
        let e = &mut self.enemies.slots[i];
        e.b.live = 1;
        e.b.k20 = 0x188;
        e.b.var26 = variant;
    }

    /// 0x6A20A class 98 (jump table 0x6A216: mode 0 -> 0x6A21E, 2 -> 0x6A396, 4 and 6 -> 0x6A402).
    pub fn enemy_class98(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                self.c98_init(e);
                self.c98_frame(e, h); // 0x6A38A falls into 0x6A396
            }
            2 => self.c98_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x6A402: jmp 0x48FC
            }
        }
    }

    /// 0x6A21E..0x6A392.
    fn c98_init(&mut self, e: &mut Enemy) {
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        e.b.atk = 0x0B;
        e.b.knock = 0;
        e.b.hitstop = 0;
        e.b.serial = self.w.attack_serial;
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
        e.b.live = 1;
        e.b.anim_table = 0x71F72;
        e.b.draw_flags = 0x10;
        e.m.sb(super::off::ATTACKING, 0xFF);
        e.b.b63 = 0xFF;
        e.m.sw(f::RAISE, 0);
        let (cx, cy) = (self.cam.s2.cam_x as i16, self.cam.s2.cam_y as i16);
        put_hi(&mut e.b.x, cx);
        put_hi(&mut e.b.y, cy);
        if e.b.var26 as i16 > 0xF {
            e.m.sw(f::RAISE, 0x20);
            e.b.var26 = e.b.var26.wrapping_sub(0x10);
        }
        let v = e.b.var26;
        if v == 0 {
            put_hi(&mut e.b.z, 1);
            add_x(&mut e.b, 0x174);
            add_y(&mut e.b, 0xB0);
            e.m.sw(f::STEP, 0xFFE8);
            self.enemy_anim(&mut e.b, 0);
            e.m.sw(f::LIFE, 0x258);
        } else {
            put_hi(&mut e.b.z, 0);
            add_x(&mut e.b, -0x5C); // addi.w #$FFA4
            add_y(&mut e.b, 0x90);
            e.m.sw(f::STEP, 0x18);
            self.enemy_anim(&mut e.b, 1);
            e.m.sw(f::LIFE, 0x258);
        }
        // 0x6A2E8: the fixed and the third piece overwrite the position, step and animation.
        if v == 2 {
            put_hi(&mut e.b.z, 1);
            put_hi(&mut e.b.x, 0x190u16.wrapping_add(0xFF10) as i16);
            put_hi(&mut e.b.y, 0xB0);
            e.m.sw(f::STEP, 0);
            self.enemy_anim(&mut e.b, 0);
            e.m.sw(f::LIFE, 0x258);
        }
        if v == 3 {
            put_hi(&mut e.b.z, 1);
            put_hi(&mut e.b.x, 0xFFA4u16.wrapping_add(0xD8) as i16);
            put_hi(&mut e.b.y, 0x90);
            e.m.sw(f::STEP, 0);
            self.enemy_anim(&mut e.b, 1);
            e.m.sw(f::LIFE, 0x258);
        }
        if v == 4 {
            put_hi(&mut e.b.z, 1);
            put_hi(&mut e.b.x, 0x174u16.wrapping_add(0xD0) as i16);
            put_hi(&mut e.b.y, 0x80);
            e.m.sw(f::STEP, 0xFFE8);
            self.enemy_anim(&mut e.b, 2);
            e.m.sw(f::LIFE, 0x260);
        }
        // 0x6A38A: raise the piece, then mode 2
        let up = e.m.w(f::RAISE);
        add_y(&mut e.b, (up as i16).wrapping_neg());
        e.b.mode += 2;
    }

    /// 0x6A396: the frame of a piece.
    fn c98_frame(&mut self, e: &mut Enemy, h: Handle) {
        let v = e.b.var26;
        if v != 2 && v != 3 {
            let life = e.m.w(f::LIFE).wrapping_sub(1);
            e.m.sw(f::LIFE, life);
            if life == 0 {
                let i = self.enemy_index(h);
                return self.enemy_free_pool_a(e, i); // 0x6A402
            }
            if (v == 0 || v == 4) && life == 0x24E {
                self.sound(0x88); // the title tune (0xA63C with d1 = d2 = 0)
            }
            if (life as i16) > 0x24E {
                add_x(&mut e.b, e.m.w(f::STEP) as i16); // 0x6A3F2
            }
        }
        self.enemy_draw_check(&mut e.b, h); // 0x6A3FC: jmp 0x49CA
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

    /// Runs the pool A update once per frame and returns the (X, Y, depth) of the piece in slot 23
    /// (the first one allocated) after each of `frames` frames, and the frame the sound 0x88 came.
    fn run(variant: u16, frames: usize) -> (Vec<(i16, i16, i16)>, Option<usize>, bool) {
        let mut g = Game::new(assets().unwrap());
        g.enemy_spawn_logo(variant);
        let (mut pos, mut tune) = (Vec::new(), None);
        for f in 0..frames {
            g.w.sounds.clear();
            g.enemies_update();
            let b = &g.enemies.slots[23].b;
            pos.push((b.xi(), b.yi(), b.zi()));
            if g.w.sounds.contains(&0x88) && tune.is_none() {
                tune = Some(f);
            }
        }
        (pos, tune, g.enemies.slots[23].b.live != 0)
    }

    /// Variant 0 slides in from the right for 9 frames after its first one (0x18 pixels each) and
    /// stays; the title tune is queued at the frame where `+$A2` reaches 0x24E (the 10th pass); it
    /// frees itself after 600 passes. Variant 1 comes from the left.
    #[test]
    fn the_logo_pieces_slide_in_stay_and_free_themselves() {
        if assets().is_none() {
            return;
        }
        let (p, tune, live) = run(0, 599);
        assert_eq!(p[0], (0x174 - 0x18, 0xB0, 1), "first frame: placed at +0x174 and moved once");
        assert_eq!(p[8], (0x174 - 9 * 0x18, 0xB0, 1), "the 9th move is the last one");
        assert_eq!(p[9], p[8]);
        assert_eq!(p[598], p[8]);
        assert_eq!(tune, Some(9), "tune at the 10th pass");
        assert!(live, "alive after 599 passes");
        assert!(!run(0, 600).2, "freed in the 600th pass (it draws nothing then)");
        let (p, tune, _) = run(1, 12);
        assert_eq!((p[0], p[8], p[11]), ((-0x5C + 0x18, 0x90, 0), (-0x5C + 9 * 0x18, 0x90, 0), (-0x5C + 9 * 0x18, 0x90, 0)));
        assert_eq!(tune, None, "only variants 0 and 4 queue the tune");
    }

    /// Variants 2 and 3 (press start) are fixed, depth 1, and never free themselves; variant 0x10
    /// is variant 0 raised by 0x20; variant 4 starts at 0x244 and lives 8 frames longer.
    #[test]
    fn fixed_raised_and_third_pieces() {
        if assets().is_none() {
            return;
        }
        let (p, tune, live) = run(2, 700);
        assert_eq!((p[0], p[699], tune, live), ((0xA0, 0xB0, 1), (0xA0, 0xB0, 1), None, true));
        let (p, ..) = run(3, 3);
        assert_eq!(p[2], (0x7C, 0x90, 1));
        let (p, ..) = run(0x10, 2);
        assert_eq!(p[0], (0x174 - 0x18, 0xB0 - 0x20, 1));
        let (p, tune, live) = run(4, 607);
        assert_eq!((p[0], tune, live), ((0x244 - 0x18, 0x80, 1), Some(17), true), "tune when +$A2 reaches 0x24E");
        assert!(!run(4, 608).2, "life 0x260: freed in the 608th pass");
    }
}
