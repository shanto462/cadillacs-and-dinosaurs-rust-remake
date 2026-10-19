//! Hogg's bomb, class 42 (handler 0x55B06..0x55F2A): made by the rider's throw (0x55A7E, `c31_bomb` in `class31.rs`). A pool A monster
//! (the hostile list 0x4C0A): it flies to a landing point (the car's place plus one of nine offsets, or the thrown at hero's place),
//! then blasts with the attack 0x69 for 50 ticks and is freed. D3c section 4.6, BS14. Owner: enemy port.

use super::{off::*, Enemy};
use crate::game::{Game, Handle};

/// The tables read at run time (extracted by `dino-extract/src/boss.rs`).
mod t {
    /// The landing cell by `rand() & 15` (words: the index list 0, 1, 1, 2, 3, 3, 4, 4, 4, 5, 5, 5, 6, 7, 7, 8).
    pub const CELL: u32 = 0x55ED4;
    /// The nine offsets (dx, dz words) around the car.
    pub const OFFSET: u32 = 0x55EF4;
}

/// 0x55D2C: the unsigned long division of the original (32 shift steps): (quotient, remainder) of `n / d`. The callers always pass a
/// divisor of 0x28 or 0x3B; the divisor 0 branch (registers left as they were) is never reached.
pub(super) fn c42_div(n: u32, d: u32) -> (u32, u32) {
    if d == 0 {
        return (0, 0);
    }
    (n / d, n % d)
}

/// One axis of 0x55D50 (the blocks 0x55D8A, 0x55DE8, 0x55E34 and 0x55E60): `pos` and `aim` are the integer words; `frames` the flight time; `shift` the `lsl.w #2` of the Z blocks;
/// `z_rule` the zero test of the Z blocks (`neg.w d0 ; bne ; tst.w d1 ; bpl ; move.w #$FFFF,d0`), the X block of the hero variants
/// negates both without it.
pub(super) fn c42_axis(pos: i16, aim: i16, frames: u32, shift: bool, z_rule: bool) -> i16 {
    let dist = pos.wrapping_sub(aim) as u16;
    let a = (dist as i16).wrapping_abs() as u16 as u32;
    let (q, r) = c42_div(a, frames);
    let (mut d0, mut d1) = (q as u16, r as u16);
    if (pos as u16) >= (aim as u16) {
        // sub.w ; bcs not taken: the aim is at or below the position, the speed goes negative.
        d0 = d0.wrapping_neg();
        if z_rule && d0 == 0 && (d1 as i16) < 0 {
            d0 = 0xFFFF;
        }
        d1 = d1.wrapping_neg();
    }
    if shift {
        d1 = d1.wrapping_shl(2);
    }
    let hi = ((d1 >> 8) as u8).wrapping_add(d0 as u8);
    (((hi as u16) << 8) | (d1 & 0xFF)) as i16
}

impl Game {
    /// 0x55B06: class 42 (mode table 0x55B24): 0 the init (0x55B2C, falls into the flight), 2 the flight (0x55B5A), 4 the blast (0x55B72),
    /// 6 the free (0x55BF0). After the mode code, always: terrain (0x12CB4), draw (0x49CA), the hostile list (0x4C0A). The blast does its own
    /// draw and list push before returning, so a blasting bomb is drawn and listed twice; when it frees itself the three calls still run
    /// on the freed slot (both as the original).
    pub(super) fn enemy_class42(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.c42_init(e);
                self.c42_fly(e, h, i);
            }
            2 => self.c42_fly(e, h, i),
            4 => self.c42_blast(e, h, i),
            _ => self.enemy_free_pool_a(e, i),
        }
        if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
            e.m.sw(OBSTACLE, ob);
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_monster_register(h);
    }

    /// 0x55B2C: the table 0x55BF6, animation 0 (the flight), the box table 0x103000, not counted (`+$7D`), the landing point (0x55E9A) and
    /// the speeds (0x55D50).
    fn c42_init(&mut self, e: &mut Enemy) {
        e.b.mode = e.b.mode.wrapping_add(2);
        e.b.anim_table = 0x55BF6;
        self.enemy_anim(&mut e.b, 0);
        e.b.box_table = 0x103000;
        e.b.multi_box = false;
        e.m.sb(ATTACKING, 0xFF);
        self.c42_aim(e);
        self.c42_launch(e);
    }

    /// 0x55E9A: the landing point into `+$88` / `+$8A`. Variants below 4 (signed): the car's place plus one of the nine offsets of 0x55EF4
    /// by `rand() & 15` through the cell list 0x55ED4. Variants 4 and 6 (0x55F18): the place of the target `+$76`.
    fn c42_aim(&mut self, e: &mut Enemy) {
        let (x, z) = if (e.b.var26 as i16) < 4 {
            let r = (self.rand() & 0xF) as u32;
            let tb = &self.assets.enemy.t;
            let cell = (tb.u16(t::CELL, 2 * r) as u32) << 2;
            let (dx, dz) = (tb.i16(t::OFFSET, cell), tb.i16(t::OFFSET, cell + 2));
            let car = self.body(self.cam.car);
            (car.xi().wrapping_add(dx), car.zi().wrapping_add(dz))
        } else {
            let tb = self.body(e.m.w(TARGET));
            (tb.xi(), tb.zi())
        };
        e.m.sw(DEST_X, x as u16);
        e.m.sw(DEST_Z, z as u16);
    }

    /// 0x55D50 (table 0x55D5C by the variant): in the air, and
    /// - variant 0 (0x55D64): the high lob: vy 0xA00, gravity -0x58, vx = car vx + 0x200, vz to the aim Z in 59 frames;
    /// - variant 2 (0x55DC2): the low arc: vy 0x280, gravity -0x20, vx = car vx - 0x10, vz in 40 frames;
    /// - variants 4 and 6 (0x55E22): vy 0x280, gravity -0x20, vx and vz to the aim in 40 frames.
    fn c42_launch(&mut self, e: &mut Enemy) {
        e.b.airborne = 1;
        let aim_x = e.m.w(DEST_X) as i16;
        let aim_z = e.m.w(DEST_Z) as i16;
        match e.b.var26 {
            0 | 2 => {
                let lob = e.b.var26 == 0;
                e.b.vy = if lob { 0xA00 } else { 0x280 };
                e.b.ay = if lob { -0x58 } else { -0x20 };
                let car_vx = self.body(self.cam.car).vx;
                e.b.vx = car_vx.wrapping_add(if lob { 0x200 } else { -0x10 });
                e.b.ax = 0;
                e.b.vz = c42_axis(e.b.zi(), aim_z, if lob { 0x3B } else { 0x28 }, true, true);
                e.b.az = 0;
            }
            4 | 6 => {
                e.b.vy = 0x280;
                e.b.ay = -0x20;
                e.b.vx = c42_axis(e.b.xi(), aim_x, 0x28, false, false);
                e.b.ax = 0;
                e.b.vz = c42_axis(e.b.zi(), aim_z, 0x28, true, true);
                e.b.az = 0;
            }
            v => unreachable!("bomb variant {v:#x}: the jump table 0x55D5C has four entries"),
        }
    }

    /// 0x55B5A: the flight: in the air it moves (0x189C) and animates (0x121E); on the ground the blast starts in this frame.
    fn c42_fly(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.airborne != 0 {
            e.b.motion();
            self.enemy_step(&mut e.b);
            return;
        }
        e.b.mode = e.b.mode.wrapping_add(2);
        self.c42_blast(e, h, i);
    }

    /// 0x55B72: the blast. Its first frame: animation 1, the attack 0x69 with a new serial, facing 0, the explosion effect 0x92422 (spark 4
    /// at the bomb), the sound 0x66 (variants below 4, fixed arguments) or the rumble 0x9F12. Every frame: the animation; at its end the
    /// bomb is freed (0x48FC), else it is drawn (0x49CA) and listed (0x4C0A).
    fn c42_blast(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.sub == 0 {
            e.b.sub = e.b.sub.wrapping_add(2);
            self.enemy_anim(&mut e.b, 1);
            e.b.atk = 0x69;
            e.b.knock = 0;
            e.b.hitstop = 0;
            e.b.serial = self.w.attack_serial; // G.$67BE
            self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
            e.b.facing = 0;
            // 0x92422 reads the caller's facing through its handle: the body is copied into the slot for the call.
            self.enemies.slots[i].b = e.b.clone();
            self.fx_spawn_spark_at(h, e.b.xi(), e.b.yi(), e.b.zi(), 4);
            if (e.b.var26 as i16) < 4 {
                self.snd_request(0x66, 0x0004_1820, 0x0200_0000);
            } else {
                self.snd_rumble(e.b.xi(), e.b.facing); // 0x9F12
            }
        }
        if self.enemy_step(&mut e.b) {
            return self.enemy_free_pool_a(e, i);
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_monster_register(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn division_is_quotient_and_remainder() {
        assert_eq!(c42_div(100, 40), (2, 20));
        assert_eq!(c42_div(7, 0), (0, 0));
    }

    #[test]
    fn axis_speed_takes_the_quotient_in_the_high_byte() {
        // 100 px to go in 40 frames, toward +: 2 px and the remainder 20 (times 4 = 80) as the fraction.
        assert_eq!(c42_axis(0, 100, 40, true, true), 0x0250);
        // The same distance toward -: both parts negated, then the byte add: high byte of -80 (0xFF) - 2 = 0xFD.
        assert_eq!(c42_axis(100, 0, 40, true, true) as u16, 0xFDB0);
        // The X rule of the hero variants has no shift: the remainder is the fraction.
        assert_eq!(c42_axis(0, 100, 40, false, false), 0x0214);
    }
}
