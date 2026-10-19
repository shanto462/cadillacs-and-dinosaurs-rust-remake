//! Pool C classes 36 and 37 of the Cadillac (M6-21): the seat drawers and the teammate who delivers the car. The car (pool B class 88, `car.rs`)
//! makes them: three (two in the 2 player layout) seat drawers 0x96FD4 at its init, the teammate 0x971D6 when it stops. D3c section 5.3, BS22.
//!
//! The heroes aboard are not drawn by their own handler (0x18A02 returns before the draw in the drive); each seat drawer shows the hero of its seat
//! (the car's `+$A2` / `+$A3` / `+$A4`: a character, negative = empty) at the car with its own offset. The teammate (the first character not in play)
//! jumps out beside the car, lands, and stays behind on the road.

use super::*;

/// The seat drawers' animation table (0x97008 stores it) and the teammate's (0x9721E).
const SEAT_ANIMS: u32 = 0x97150;
const MATE_ANIMS: u32 = 0x972B2;
/// 0x970F6: the offset of each seat from the car (dx, dy, dz words, 6 bytes a seat), read through `enemy.t` (dino-extract/src/boss.rs).
const SEAT_OFFSETS: u32 = 0x970F6;

impl Effect {
    /// `+$5A`, `+$5B`, `+$5C` bytes of a seat drawer: the seats the drawer last saw (the high and low byte of the word `+$5A`, the high
    /// byte of the word `+$5C`).
    fn seen(&self, k: usize) -> u8 {
        match k {
            0 => (self.owner_class >> 8) as u8,
            1 => self.owner_class as u8,
            _ => (self.w5c >> 8) as u8,
        }
    }

    fn set_seen(&mut self, k: usize, v: u8) {
        match k {
            0 => self.owner_class = (self.owner_class & 0x00FF) | (v as u16) << 8,
            1 => self.owner_class = (self.owner_class & 0xFF00) | v as u16,
            _ => self.w5c = (self.w5c & 0x00FF) | (v as u16) << 8,
        }
    }
}

impl Game {
    /// 0x96FF4: a seat drawer (pool C class 36, `+$20` = 0x90; mode table 0x97000: 0 the init 0x97008 falling into the run 0x9703E, 2 the run,
    /// 4 and 6 the free 0x9714A).
    pub(super) fn fx_seat(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => {
                // 0x97008: the table, no shadow, on screen, no seat seen, the seat 2 drawer faces right.
                let e = &mut self.effects.slots[i];
                e.b.anim_table = SEAT_ANIMS;
                e.b.draw_flags = 0x10;
                e.b.visible = 1;
                for k in 0..3 {
                    e.set_seen(k, 0xFF);
                }
                if e.b.var26 == 2 {
                    e.b.facing = 1;
                }
                e.b.mode = e.b.mode.wrapping_add(2);
                self.fx_seat_run(i);
            }
            2 => self.fx_seat_run(i),
            _ => self.fx_free(i),
        }
    }

    /// The seat of this drawer on the car `car` (`+$A2` for variant 0, `+$A3` for 1, `+$A4` otherwise).
    fn fx_seat_of(car: &crate::game::prop::Prop, var: u16) -> u8 {
        match var {
            0 => car.m.b(0xA2),
            1 => car.m.b(0xA3),
            _ => car.m.b(0xA4),
        }
    }

    /// 0x9703E: the drawer sits at the car plus its offset (0x970F6). When the car's seats differ from the ones it saw, it takes them: with the car
    /// gone (not live, or not class 88) it frees itself (0x4964, no draw); else its seat's animation starts (0x97108: character * 2, + 1 for the
    /// variant 2 drawer, by 0x15E4). An empty seat is not drawn; else the animation runs (0x15F4) and it is drawn (0x49C0).
    fn fx_seat_run(&mut self, i: usize) {
        let owner = self.effects.slots[i].owner58;
        let car = match slot_of(owner) {
            Some(Slot::Prop(c)) => self.props.slots[c].clone(),
            _ => crate::game::prop::Prop::default(),
        };
        let var = self.effects.slots[i].b.var26;
        let row = var.wrapping_mul(6) as u32;
        let t = &self.assets.enemy.t;
        let (dx, dy, dz) = (t.i16(SEAT_OFFSETS, row), t.i16(SEAT_OFFSETS, row + 2), t.i16(SEAT_OFFSETS, row + 4));
        {
            let e = &mut self.effects.slots[i];
            // move.w: the integer words, the fractions stay.
            e.b.x = (e.b.x & 0xFFFF) | ((car.b.xi().wrapping_add(dx) as i32) << 16);
            e.b.y = (e.b.y & 0xFFFF) | ((car.b.yi().wrapping_add(dy) as i32) << 16);
            e.b.z = (e.b.z & 0xFFFF) | ((car.b.zi().wrapping_add(dz) as i32) << 16);
        }
        let seats = [car.m.b(0xA2), car.m.b(0xA3), car.m.b(0xA4)];
        let changed = (0..3).any(|k| self.effects.slots[i].seen(k) != seats[k]);
        if changed {
            for k in 0..3 {
                self.effects.slots[i].set_seen(k, seats[k]);
            }
            if car.b.live == 0 || car.b.k20 != 0x160 {
                return self.fx_free(i); // 0x9714A: jmp 0x4964
            }
            let ch = Self::fx_seat_of(&car, var);
            if (ch as i8) >= 0 {
                let mut id = (ch as u16).wrapping_mul(2);
                if var == 2 {
                    id = (id & 0xFF00) | ((id as u8).wrapping_add(1) as u16); // addq.b #1,d0
                }
                let fx = self.fx_data();
                fx_start(&mut self.effects.slots[i].b, fx, id);
            }
        }
        if (Self::fx_seat_of(&car, var) as i8) < 0 {
            return;
        }
        let fx = self.fx_data();
        fx_tick(&mut self.effects.slots[i].b, fx);
        self.draw_list_add_if_visible(self.effects.handle(i));
    }

    /// 0x9720A: the teammate who brings the car (pool C class 37, `+$20` = 0x94, variant = his character; mode table 0x97216: 0 the init 0x9721E
    /// falling into the run, 2 the run 0x97254, 4 and 6 the free 0x972AC). Init: the table 0x972B2, on screen, facing right, the animation of his
    /// character, a jump (vx -0x230, vy 0x7A0, gravity -0x58). Run (table 0x9727C by `+$05`): in the air the animation runs and he moves (0x189C);
    /// at height 0 or below the standing animation (character + 4) starts; then only the animation. When he is 0x20 or more behind the camera's left
    /// edge (signed) he is freed in the next frame (`+$04` = 6). Drawn by 0x49C0.
    pub(super) fn fx_mate(&mut self, i: usize) {
        let fx = self.fx_data();
        match self.effects.slots[i].b.mode {
            0 => {
                let e = &mut self.effects.slots[i];
                e.b.anim_table = MATE_ANIMS;
                e.b.visible = 1;
                e.b.facing = 1;
                let v = e.b.var26;
                fx_start(&mut e.b, fx, v);
                e.b.vx = -0x230;
                e.b.vy = 0x7A0;
                e.b.ay = -0x58;
                e.b.mode = e.b.mode.wrapping_add(2);
            }
            2 => {}
            _ => return self.fx_free(i),
        }
        let e = &mut self.effects.slots[i];
        if e.b.sub == 0 {
            fx_tick(&mut e.b, fx);
            e.b.motion();
            if e.b.yi() <= 0 {
                let v = e.b.var26.wrapping_add(4);
                fx_start(&mut e.b, fx, v);
                e.b.sub = e.b.sub.wrapping_add(2);
            }
        } else {
            fx_tick(&mut e.b, fx);
        }
        let lim = (self.cam.s2.cam_x as i16).wrapping_sub(0x20);
        let e = &mut self.effects.slots[i];
        if lim >= e.b.xi() {
            e.b.mode = 6;
            e.b.sub = 0;
        }
        self.draw_list_add_if_visible(self.effects.handle(i));
    }
}
