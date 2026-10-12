//! Class 20: the choice machine of action 4 (`+$A5`), the attack script (state 6) and the small
//! routines at 0x46A72..0x47198.

use super::class20::{f, t};
use super::{Enemy, off::*};
use crate::game::{Game, Handle};

impl Game {
    // ------------------------------------------------------------------ state 6: the attack script

    /// 0x46356: an attack starts: state 6, a script row by `rand()`, sound-less.
    pub(super) fn c20_46356(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0200_0600);
        self.c20_46ea6(e);
        e.m.sw(f::POS, e.m.w(f::ROW) << 3);
        e.m.sb(ATTACKING, 1);
        self.c20_s6(e, h);
    }

    /// 0x46372 state 6.
    pub(super) fn c20_s6(&mut self, e: &mut Enemy, _h: Handle) {
        let b = e.b.hitstop;
        if b != 0 {
            e.b.hitstop = b - 1;
            if b - 1 != 0 {
                return;
            }
            e.m.sb(f::HIT, 1);
            e.b.timer = 1;
        }
        match e.b.step {
            0 => self.c20_463ae(e),
            2 => self.c20_46476(e),
            4 => self.c20_464e4(e),
            6 => self.c20_4652a(e),
            8 => self.c20_46536(e),
            0xA => self.c20_46712(e),
            0xC | 0xE => self.c20_46974(e),
            0x10 => self.c20_467ca(e),
            s => panic!("class 20 state 6 step {s:#x}"),
        }
    }

    /// 0x463AE: the next action of the row (step 0).
    pub(super) fn c20_463ae(&mut self, e: &mut Enemy) {
        e.m.sw(f::SUB, 0);
        let tp = self.c20_target(e);
        {
            let p = &self.players[tp].b;
            if p.live == 0 || p.sub == 2 || p.sub == 6 {
                return e.b.set_state(0x0200_0000); // 0x463EC
            }
        }
        let d1 = self.assets.enemy.t.u8(t::SCRIPT, e.m.w(f::POS) as u32);
        if (d1 as i8) < 0 {
            return e.b.set_state(0x0200_0000);
        }
        e.m.sw(f::POS, e.m.w(f::POS).wrapping_add(1));
        match d1 {
            0 => self.c20_4645a(e),
            2 => self.c20_464aa(e),
            4 => self.c20_464f0(e),
            6 => self.c20_46536_start(e),
            8 => self.c20_46712(e),
            0xA => e.b.step = 0x10, // 0x467C2
            v => panic!("class 20 script action {v:#x}"),
        }
    }

    /// Script byte at the position (the next one after the increment).
    fn c20_next_is_end(&self, e: &Enemy) -> bool {
        (self.assets.enemy.t.u8(t::SCRIPT, e.m.w(f::POS) as u32) as i8) < 0
    }

    /// 0x4645A action 0: walk at the target.
    fn c20_4645a(&mut self, e: &mut Enemy) {
        e.b.step = 2;
        self.enemy_anim(&mut e.b, 1);
        self.c20_46e7c(e);
        self.c20_46dd8(e);
        e.m.sw(f::T_A0, 0xC8);
        self.c20_46476(e);
    }

    /// 0x46476 step 2.
    fn c20_46476(&mut self, e: &mut Enemy) {
        if e.b.airborne != 0 {
            return self.c20_fall_state(e);
        }
        let w = e.m.w(f::T_A0);
        e.m.sw(f::T_A0, w.wrapping_sub(1));
        if w == 0 {
            return e.b.set_state(0x0200_0000);
        }
        if Self::enemy_stuck_check(e) {
            self.c20_46e94(e);
        }
        if self.c20_46d18(e) {
            return self.c20_463ae(e);
        }
        self.c20_46dd8(e);
        self.enemy_move_dir(&mut e.b);
        self.enemy_walk_anim_c(&mut e.b);
    }

    /// 0x464AA action 2 / 0x464F0 action 4: a punch (anim 0xC / 0xD, atk 0xD / 0xE).
    fn c20_464aa(&mut self, e: &mut Enemy) {
        self.c20_punch(e, 4, 0xC, 0xD);
        self.c20_464e4_body(e);
    }
    fn c20_464f0(&mut self, e: &mut Enemy) {
        self.c20_punch(e, 6, 0xD, 0xE);
        self.c20_4652a_body(e);
    }
    fn c20_punch(&mut self, e: &mut Enemy, step: u8, anim: u16, atk: u8) {
        e.b.step = step;
        self.enemy_anim(&mut e.b, anim);
        self.enemy_begin_attack(&mut e.b, atk);
        if self.c20_next_is_end(e) {
            e.b.knock = 1;
        }
    }
    fn c20_464e4(&mut self, e: &mut Enemy) {
        self.c20_464e4_body(e);
    }
    fn c20_464e4_body(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            self.c20_46940(e);
        }
    }
    fn c20_4652a(&mut self, e: &mut Enemy) {
        self.c20_4652a_body(e);
    }
    fn c20_4652a_body(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            self.c20_46940(e);
        }
    }

    /// 0x46940: recovery after a punch: a pause by the level band (steps 0xC / 0xE end at 0x46974).
    fn c20_46940(&mut self, e: &mut Enemy) {
        self.enemy_set_speed(&mut e.b, 0xB);
        e.b.step = 0xE;
        let r = (self.rand() & 0x1F) as u32;
        let d0 = self.assets.enemy.t.u8(t::REC_IDX, r) as u32 + (self.w.level & 0x18) as u32;
        let v = self.assets.enemy.t.u8(t::REC, d0);
        e.m.sb(f::T80, v);
        self.enemy_anim(&mut e.b, 0);
        self.c20_46974_body(e);
    }
    fn c20_46974(&mut self, e: &mut Enemy) {
        self.c20_46974_body(e);
    }
    fn c20_46974_body(&mut self, e: &mut Enemy) {
        let b = e.m.b(f::T80);
        e.m.sb(f::T80, b.wrapping_sub(1));
        if b == 0 {
            self.c20_463ae(e);
        }
    }

    // ------------------------------------------------------------------ action 6: jump kick

    /// 0x46536 action 6 (also the step 8 handler): `+$AE` is the phase.
    fn c20_46536_start(&mut self, e: &mut Enemy) {
        // falls into the phase table with +$AE = 0 (cleared at 0x463AE)
        self.c20_46536_phase(e);
    }
    fn c20_46536(&mut self, e: &mut Enemy) {
        self.c20_46536_phase(e);
    }
    fn c20_46536_phase(&mut self, e: &mut Enemy) {
        match e.m.w(f::SUB) {
            0 => {
                // 0x46586
                e.b.step = 8;
                e.m.sw(f::SUB, 2);
                self.c20_4654e(e);
                self.enemy_anim(&mut e.b, 0x15);
                self.enemy_begin_attack(&mut e.b, 0xF);
                self.c20_46d00(e);
                self.c20_465b8(e);
            }
            2 => self.c20_465b8(e),
            4 => self.c20_465e4(e),
            6 => self.c20_465fc(e),
            8 => self.c20_46642(e),
            0xA => self.c20_466e2(e),
            v => panic!("class 20 jump kick phase {v:#x}"),
        }
    }

    /// 0x4654E: the jump at the target: vx +-0x300, vy 0x600, ay -0x50.
    fn c20_4654e(&mut self, e: &mut Enemy) {
        let tp = self.c20_target(e);
        let mut d1 = 0x300i16;
        // sub.w ; bcs: target left of us (unsigned borrow) keeps the sign
        if (self.players[tp].b.xi() as u16) >= (e.b.xi() as u16) {
            d1 = -d1;
        }
        e.b.airborne = 1;
        e.b.vx = d1;
        e.b.ax = 0;
        e.b.vy = 0x600;
        e.b.ay = 0xFFB0u16 as i16;
        e.b.vz = 0;
        e.b.az = 0;
    }

    fn c20_465b8(&mut self, e: &mut Enemy) {
        if e.b.airborne != 0 {
            if e.b.param_b == 0 {
                e.b.motion();
            }
            self.enemy_step(&mut e.b);
            return;
        }
        e.m.sw(f::SUB, 4);
        self.snd_9c20();
        self.enemy_anim(&mut e.b, 0xF);
        self.c20_465e4(e);
    }

    fn c20_465e4(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            e.m.sw(f::SUB, 6);
            self.enemy_anim(&mut e.b, 0x14);
            self.c20_465fc(e);
        }
    }

    fn c20_465fc(&mut self, e: &mut Enemy) {
        if !self.enemy_step(&mut e.b) {
            return;
        }
        // 0x46606
        e.m.sw(f::SUB, 8);
        self.enemy_anim(&mut e.b, 2);
        let tp = self.c20_target(e);
        let (x, z) = (self.players[tp].b.xi() as u16, self.players[tp].b.zi().wrapping_add(4) as u16);
        e.m.sw(f::DEST_X, x);
        e.m.sw(f::DEST_Z, z);
        self.c20_46bcc(e);
        if (e.b.dir >> 4) == e.b.facing {
            return e.b.set_state(0x0200_0000);
        }
        self.enemy_set_speed(&mut e.b, 0xD);
        self.c20_46642(e);
    }

    fn c20_46642(&mut self, e: &mut Enemy) {
        if e.b.airborne != 0 {
            return self.c20_fall_state(e);
        }
        if Self::enemy_stuck_check(e) {
            return e.b.set_state(0x0200_0000);
        }
        if !self.c20_46f3c(e) {
            self.enemy_move_dir(&mut e.b);
            self.enemy_step(&mut e.b);
            return;
        }
        // 0x46668: the lunge
        e.m.sw(f::SUB, 0xA);
        e.m.sb(f::HIT, 0);
        self.enemy_anim(&mut e.b, 0xA);
        let d5 = e.b.dir;
        let tp = self.c20_target(e);
        let (x, z) = (self.players[tp].b.xi() as u16, self.players[tp].b.zi() as u16);
        e.m.sw(f::DEST_X, x);
        e.m.sw(f::DEST_Z, z);
        self.c20_46bcc(e);
        let d1 = e.b.dir.wrapping_sub(d5).wrapping_add(1) & 0x1F;
        if d1 > 2 {
            // `bls 0x466B6` keeps the new direction; a bigger turn goes one step from the old one.
            let d2: u8 = if d1 >= 0x11 { 1 } else { 0xFF };
            e.b.dir = d5.wrapping_add(d2);
        }
        self.enemy_move_dir(&mut e.b);
        let mut d0 = 0x480i16;
        e.b.airborne = 1;
        if e.b.facing == 0 {
            d0 = -d0;
        }
        e.b.vx = d0;
        e.b.ax = 0;
        e.b.vy = 0x280;
        e.b.ay = 0xFFE0u16 as i16;
        self.c20_466e2(e);
    }

    fn c20_466e2(&mut self, e: &mut Enemy) {
        if e.m.b(f::HIT) != 0 {
            e.m.sb(f::HIT, 0);
            e.b.vx = e.b.vx.wrapping_neg() >> 2;
        }
        if e.b.airborne != 0 {
            e.b.motion();
            self.enemy_step(&mut e.b);
            return;
        }
        self.c20_46d00(e);
        self.c20_46940(e);
    }

    // ------------------------------------------------------------------ action 8 and 0xA

    /// 0x46712 action 8: a jump with a dive; `+$AE` is the phase (0, 2, 4).
    fn c20_46712(&mut self, e: &mut Enemy) {
        match e.m.w(f::SUB) {
            0 => {
                // 0x46724
                e.b.step = 0xA;
                e.m.sw(f::SUB, e.m.w(f::SUB).wrapping_add(2));
                self.enemy_anim(&mut e.b, 0x11);
                self.enemy_begin_attack(&mut e.b, 0x10);
                e.b.airborne = 1;
                e.b.vy = 0x480;
                e.b.ay = 0xFFD0u16 as i16;
                let tp = self.c20_target(e);
                let mut d0 = 0x180i16;
                let mut d2 = 1u8;
                if (self.players[tp].b.xi().wrapping_sub(e.b.xi())) < 0 {
                    d0 = -d0;
                    d2 = 0;
                }
                e.b.vx = d0;
                e.b.ax = 0;
                e.b.vz = 0;
                e.b.az = 0;
                e.b.facing = d2;
                self.c20_4678c(e);
            }
            2 => self.c20_4678c(e),
            4 => self.c20_467b6(e),
            v => panic!("class 20 action 8 phase {v:#x}"),
        }
    }

    fn c20_4678c(&mut self, e: &mut Enemy) {
        if e.b.airborne != 0 {
            if e.b.param_b == 0 {
                e.b.motion();
            }
            self.enemy_step(&mut e.b);
            return;
        }
        e.m.sw(f::SUB, e.m.w(f::SUB).wrapping_add(2));
        self.snd_9c20();
        self.enemy_anim(&mut e.b, 0xF);
        self.c20_467b6(e);
    }

    fn c20_467b6(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            self.c20_46940(e);
        }
    }

    /// 0x467CA (step 0x10): the leaping dive; `+$AE` (byte) is the phase.
    fn c20_467ca(&mut self, e: &mut Enemy) {
        match e.m.b(f::SUB) {
            0 => {
                // 0x467E0
                e.m.sb(f::SUB, 2);
                self.enemy_anim(&mut e.b, 1);
                self.enemy_set_speed(&mut e.b, 0xB);
                self.c20_46d00(e);
                e.m.sw(f::T_A0, 0);
                self.c20_467fc(e);
            }
            2 => self.c20_467fc(e),
            4 => self.c20_468fc(e),
            6 => self.c20_46920(e),
            v => panic!("class 20 dive phase {v:#x}"),
        }
    }

    fn c20_467fc(&mut self, e: &mut Enemy) {
        if self.c20_46f26(e) {
            return self.c20_4681a(e);
        }
        let w = e.m.w(f::T_A0);
        e.m.sw(f::T_A0, w.wrapping_sub(1));
        if w == 0 {
            self.c20_46f06(e);
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_walk_anim_c(&mut e.b);
    }

    /// 0x4681A: the leap: a point 0x30 in front of the target, a velocity from a division by 0x33.
    fn c20_4681a(&mut self, e: &mut Enemy) {
        e.m.sb(f::SUB, e.m.b(f::SUB).wrapping_add(2));
        self.enemy_anim(&mut e.b, 0xE);
        let tp = self.c20_target(e);
        let t = self.players[tp].b.clone();
        let mut d0 = 0x30i16;
        if (t.xi() as u16) >= (e.b.xi() as u16) {
            d0 = -d0;
        }
        if t.facing != e.b.facing {
            d0 = -d0;
        }
        let dx = t.xi().wrapping_add(d0);
        e.m.sw(f::DEST_X, dx as u16);
        e.m.sw(f::DEST_Z, t.zi() as u16);
        e.b.airborne = 1;
        e.b.vy = 0x800;
        e.b.ay = 0xFFB0u16 as i16;
        // x
        let diff = e.b.xi().wrapping_sub(dx);
        let (q, r) = Self::c20_div33(diff);
        let (mut d0, mut d1) = (q, r);
        if (e.b.xi() as u16) >= (dx as u16) {
            d0 = d0.wrapping_neg();
            d1 = d1.wrapping_neg();
        }
        // `add.b d0,$14(a6)`: the byte at +$14 is the HIGH byte of the word.
        let vx = d1 << 2;
        e.b.vx = ((vx & 0x00FF) | (((vx >> 8) as u8).wrapping_add(d0 as u8) as u16) << 8) as i16;
        e.b.ax = 0;
        // z
        let dz = t.zi();
        let diff = e.b.zi().wrapping_sub(dz);
        let (q, r) = Self::c20_div33(diff);
        let (mut d0, mut d1) = (q, r);
        if (e.b.zi() as u16) >= (dz as u16) {
            d0 = d0.wrapping_neg();
            if d0 == 0 && (d1 as i16) < 0 {
                d0 = 0xFFFF;
            }
            d1 = d1.wrapping_neg();
        }
        let vz = d1 << 2;
        e.b.vz = ((vz & 0x00FF) | (((vz >> 8) as u8).wrapping_add(d0 as u8) as u16) << 8) as i16;
        e.b.az = 0;
    }

    /// 0x468CE / 0x468D8: |word| divided by 0x33: (quotient, remainder), both as words.
    fn c20_div33(v: i16) -> (u16, u16) {
        let a = v.unsigned_abs() as u32;
        ((a / 0x33) as u16, (a % 0x33) as u16)
    }

    fn c20_468fc(&mut self, e: &mut Enemy) {
        if e.b.airborne != 0 {
            if e.b.param_b == 0 {
                e.b.motion();
            }
            self.enemy_step(&mut e.b);
            return;
        }
        e.m.sb(f::SUB, e.m.b(f::SUB).wrapping_add(2));
        self.enemy_anim(&mut e.b, 0xF);
        self.c20_46920_inner(e);
    }

    fn c20_46920(&mut self, e: &mut Enemy) {
        self.c20_46920_inner(e);
    }
    fn c20_46920_inner(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            self.c20_46d00(e);
            if e.m.w(f::ROW) == 8 {
                self.c20_471d8(e);
            }
            self.c20_463ae(e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0x468CE / 0x468D8: the quotient and remainder of |dx| / 0x33 (the leap of the dive).
    #[test]
    fn the_leap_divides_by_0x33() {
        assert_eq!(Game::c20_div33(78), (1, 27));
        assert_eq!(Game::c20_div33(-78), (1, 27));
        assert_eq!(Game::c20_div33(0), (0, 0));
        assert_eq!(Game::c20_div33(0x33), (1, 0));
    }
}
