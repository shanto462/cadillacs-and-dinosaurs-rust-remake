//! Pool A class 17 (`+$20 = 0x44`, handler 0x423A4): one of the two heroes in the Cadillac of the ranking
//! screen (M1-07). The car (pool B class 86) creates them with the variant 0 (the driver, 0x4D to the right of
//! the car) and 1 (0x4B to the left) and its own handle in `+$AC`. Heroes are drawn with the animation table
//! 0x42516 (animation 0), live as 2 and are not counted (`+$7D` = 0xFF).
//!
//! Mode 0 (0x423B8) starts the animation, takes the Y of the car minus 2 as the resting height `+$B4` and
//! remembers X in `+$B0`, then runs the body of mode 2 (0x423FE): the hero follows the car (X, depth + 2;
//! when the car's mode is 4 or more, or the car is gone, it goes to mode 2 + 2 = 4 and frees itself next
//! frame, 0x42510), and when its X changed since the frame before the animation steps and it bobs: Y
//! moves by `+$A2` (16.16, starts at +0.5 pixel a frame) and one random number a frame (low two bits) sets
//! the turning heights `+$B4` - 8 + r (up) and `+$B4` - 4 (down). The draw list check is 0x49CA.

use super::Enemy;
use crate::game::{Game, Handle};

/// Offsets of this class.
mod f {
    /// word: X of the frame before.
    pub const LAST_X: usize = 0xB0;
    /// word: the resting height (the car's Y minus 2).
    pub const BASE_Y: usize = 0xB4;
    /// long: Y speed of the bobbing, 16.16.
    pub const SPEED_Y: usize = 0xA2;
    /// long: the car (a pool B object).
    pub const OWNER: usize = 0xAC;
}

impl Game {
    /// 0x423A4 pool A class 17 (jump table 0x423B0: mode 0 -> 0x423B8, 2 -> 0x423FE, 4 and 6 -> 0x42510).
    pub fn enemy_class17(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                e.b.live = 2; // move.b #2,$0(a6)
                e.m.sb(crate::game::enemy::off::ATTACKING, 0xFF);
                e.b.anim_table = 0x42516;
                e.m.sw(0xA0, 0); // clr.w $A0(a6)
                e.m.sl(f::SPEED_Y, 0x8000);
                self.enemy_anim(&mut e.b, 0);
                let car = e.m.l(f::OWNER) as Handle;
                let y = self.body(car).yi().wrapping_sub(2);
                e.m.sw(f::BASE_Y, y as u16);
                e.b.set_yi(y);
                e.m.sw(f::LAST_X, e.b.xi() as u16);
                e.b.mode += 2;
                self.enemy_c17_body(e, h);
            }
            2 => self.enemy_c17_body(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x42510: jmp 0x48FC
            }
        }
    }

    /// 0x423FE: follow the car, bob, draw.
    fn enemy_c17_body(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_c17_follow(e); // bsr 0x42414
        self.enemy_c17_bob(e); // bsr 0x42460
        self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
    }

    /// 0x42414.
    fn enemy_c17_follow(&mut self, e: &mut Enemy) {
        let car = self.body(e.m.l(f::OWNER) as Handle);
        let (cx, cz, cmode, clive) = (car.xi(), car.zi(), car.mode as i8, car.live);
        e.b.set_zi(cz.wrapping_add(2));
        let x = if e.b.var26 == 0 { cx.wrapping_add(0x4D) } else { cx.wrapping_sub(0x4B) };
        e.b.set_xi(x);
        if cmode >= 4 || clive == 0 {
            e.b.mode += 2; // the car is gone: free next frame
        }
    }

    /// 0x42460.
    fn enemy_c17_bob(&mut self, e: &mut Enemy) {
        if e.m.w(f::LAST_X) as i16 == e.b.xi() {
            return; // beq 0x424BE
        }
        e.m.sw(f::LAST_X, e.b.xi() as u16);
        self.enemy_step(&mut e.b); // jsr 0x121E
        e.b.y = e.b.y.wrapping_add(e.m.l(f::SPEED_Y) as i32); // add.l d0,$C(a6)
        let r = (self.rand() & 3) as i16;
        let base = e.m.w(f::BASE_Y) as i16;
        if base.wrapping_sub(8).wrapping_add(r) < e.b.yi() {
            e.m.sl(f::SPEED_Y, 0xFFFF_8000); // moving down by the turn: up
        }
        if base.wrapping_sub(4) >= e.b.yi() {
            e.m.sl(f::SPEED_Y, 0x8000);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use std::path::Path;
    use std::sync::OnceLock;

    fn assets() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// A hero stands 0x4D right (variant 0) or 0x4B left (variant 1) of the car and 2 nearer, 2 below the car's
    /// height; he bobs between the turning heights while the car moves (one random number a frame) and frees
    /// himself one frame after the car is gone.
    #[test]
    fn the_heroes_follow_the_car_and_leave_with_it() {
        if assets().is_none() {
            return;
        }
        let mut g = Game::new(assets().unwrap());
        let i = g.props.alloc().unwrap();
        g.props.slots[i].b.live = 1;
        g.props.slots[i].b.k20 = 0x158;
        g.props.slots[i].b.set_xi(0x20);
        g.props.slots[i].b.set_yi(0x60);
        g.props.slots[i].b.set_zi(3);
        g.pool_b_update(); // the car makes the heroes
        g.enemies_update();
        let (d, p) = (g.enemies.slots[23].b.clone(), g.enemies.slots[22].b.clone());
        assert_eq!((d.xi(), d.yi(), d.zi(), d.live), (0x20 + 0x4D, 0x5E, 5, 2));
        assert_eq!((p.xi(), p.zi()), (0x20 - 0x4B, 5));
        let rng = g.w.rng;
        g.pool_b_update(); // the car moves 12 pixels
        g.enemies_update();
        assert_ne!(g.w.rng, rng, "the heroes bob: one random number each");
        assert_eq!(g.enemies.slots[23].b.xi(), 0x20 + 12 + 0x4D);
        g.props.slots[i].b.live = 0;
        g.enemies_update();
        assert_ne!(g.enemies.slots[23].b.live, 0);
        g.enemies_update();
        assert_eq!(g.enemies.slots[23].b.live, 0);
    }
}
