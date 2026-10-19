//! Bludge (class 6), part 2 (M7-08): the attack script of the act 6 (0x39C38 .. 0x3A0B2), the tongue grab of the sub 4 (0x3A0B4) and the hop hooks that
//! start them (0x3AEE4, 0x3AF64). The script is a byte string by `+$B4` (table 0x39CAC, 4 bytes a script, `+$B2` is the read position): 0 walks to a slot
//! near the hero (the step 2), 2 leaps (step 4), 4 lunges (attack 0x24, step 6), 6 charges (attack 0x25, step 8), 8 lashes the long tongue jump (attack 0x26,
//! step 0xA), 0xA spits the acid ball A7 (class 7, step 0xC), 0xC (and the sub 4 state) grabs a hero with the tongue of the part 0 (`class6_tongue.rs`);
//! a byte of 0x80 or more ends the script (0x39CA2: mode 2, sub 0, act 0). Every step reads `+$AD` as its own phase counter (0, 2, 4, 6) and the animation
//! end (the carry of 0x121E) moves on. Where a routine of the original falls into the next one it is called at the end of the function, as the 68000 does.

use super::Enemy;
use crate::game::{Game, Handle, Slot, slot_of};

/// 0x39CAC: the script bytes.
const SCRIPTS: u32 = 0x39CAC;
/// 0x39E70: the hold of the lunge by level.
const LUNGE_HOLD: u32 = 0x39E70;
/// 0x3A9E8: the slot base by script.
const SLOT_BASE: u32 = 0x3A9E8;
/// 0x3AED8: the roll limit by script.
const ROLL_LIMIT: u32 = 0x3AED8;
/// 0x3AA9E: the (dx, dz) of the six attack slots.
const SLOTS: u32 = 0x3AA9E;

impl Game {
    /// 0x39C38: the act 6 body: the hit stop counter, then the step.
    pub(super) fn enemy_c6_state_attack(&mut self, e: &mut Enemy) {
        if e.b.hitstop != 0 {
            e.b.hitstop -= 1;
            if e.b.hitstop != 0 {
                return;
            }
            e.b.timer = 1; // 0x39C48
        }
        match e.b.step {
            0 => self.c6_script_next(e),
            2 => self.c6_atk_walk(e),
            4 => self.c6_atk_leap(e),
            6 => self.c6_atk_lunge(e),
            8 => self.c6_atk_charge(e),
            0xA => self.c6_atk_tongue(e),
            0xC => self.c6_atk_spit(e),
            _ => {}
        }
    }

    /// 0x39CA2: the script is over (or the hero is gone): the wait state.
    fn c6_attack_end(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0000);
    }

    /// 0x39C6A: the next command of the script.
    fn c6_script_next(&mut self, e: &mut Enemy) {
        let tb = self.body(e.m.w(0x76));
        if tb.live == 0 || tb.sub == 2 {
            return self.c6_attack_end(e);
        }
        let pos = e.m.w(0xB2);
        let cmd = self.assets.enemy.t.u8(SCRIPTS, pos as u32);
        if (cmd as i8) < 0 {
            return self.c6_attack_end(e);
        }
        if cmd == 0xC {
            e.b.set_state(0x0204_0000); // 0x3A0AA: the tongue grab
            return;
        }
        e.m.sw(0xB2, pos.wrapping_add(1));
        match cmd {
            0 => self.c6_cmd_walk(e),
            2 => self.c6_cmd_leap(e),
            4 => self.c6_cmd_lunge(e),
            6 => self.c6_cmd_charge(e),
            8 => self.c6_cmd_tongue(e),
            _ => self.c6_cmd_spit(e),
        }
    }

    /// 0x3AEC0: the roll: carry when `rand() & 15` is above the limit of the script.
    fn c6_roll(&mut self, e: &Enemy) -> bool {
        let d0 = self.rand() & 0xF;
        let d1 = self.assets.enemy.t.u16(ROLL_LIMIT, e.m.w(0xB4) as u32 * 2);
        d1 < d0
    }

    // ---------------------------------------------------------------- the slot of the attack approach

    /// 0x3A978: the attack slot `+$B1` (0 .. 5): 0 to 2 left of the hero, 3 to 5 right of him; the scripts 3 and 4 take the side that keeps the Bludge
    /// on the screen when the hero stands near the right edge (the screen edge + 0xC0).
    fn enemy_c6_atk_slot(&mut self, e: &mut Enemy) {
        let b4 = e.m.w(0xB4);
        let base = self.assets.enemy.t.u8(SLOT_BASE, b4 as u32) as u16;
        let tx = self.body(e.m.w(0x76)).xi();
        let mut d1 = 0u16;
        let mut normal = true;
        if b4 == 3 || b4 == 4 {
            let (window, limit) = if b4 == 3 { (0x70u16, 0xE0u16) } else { (0x50, 0xA0) };
            let d3 = (self.cam.s2.cam_x.wrapping_add(0xC0)).wrapping_sub(tx as u16);
            if d3.wrapping_add(window) >= limit {
                normal = false;
                if (d3 as i16) >= 0 {
                    d1 = 3;
                }
            }
        }
        if normal && (tx as u16) < (e.b.xi() as u16) {
            d1 = 3; // 0x3A99E: the hero is left of the Bludge
        }
        e.m.sb(0xB1, d1.wrapping_add(base) as u8);
    }

    /// 0x3AA5E: the point of the slot `+$B1` round the hero (+$88 / +$8A), the steering and the facing toward the hero.
    fn enemy_c6_atk_aim(&mut self, e: &mut Enemy) {
        let (dest_x, dest_z, tx) = self.c6_slot_point(e);
        e.m.sw(0x88, dest_x as u16);
        e.m.sw(0x8A, dest_z as u16);
        self.enemy_c6_steer_angle(e); // 0x3AB62
        e.b.facing = (tx >= e.b.xi()) as u8; // sub.w ; blt
    }

    /// 0x3AB2C: the same point, the direction taken as it is.
    fn enemy_c6_atk_dir(&mut self, e: &mut Enemy) {
        let (dest_x, dest_z, _) = self.c6_slot_point(e);
        e.m.sw(0x88, dest_x as u16);
        e.m.sw(0x8A, dest_z as u16);
        e.b.dir = Self::enemy_angle_to(&e.b, dest_x as u16, dest_z as u16); // 0x103A
    }

    /// (X, Z) of the slot `+$B1` round the target and the X of the target.
    fn c6_slot_point(&self, e: &Enemy) -> (i16, i16, i16) {
        let k = e.m.b(0xB1) as u32 * 4;
        let t = &self.assets.enemy.t;
        let tb = self.body(e.m.w(0x76));
        (tb.xi().wrapping_add(t.i16(SLOTS, k)), tb.zi().wrapping_add(t.i16(SLOTS, k + 2)), tb.xi())
    }

    // ---------------------------------------------------------------- command 0, the walk (steps 0 and 2)

    /// 0x39CD0: a roll sends the Bludge straight to the leap; else it walks to its slot for 120 frames.
    fn c6_cmd_walk(&mut self, e: &mut Enemy) {
        if self.c6_roll(e) {
            return self.c6_cmd_leap(e); // bcs 0x39D16
        }
        self.enemy_c6_atk_slot(e);
        e.b.step = 2;
        self.enemy_anim(&mut e.b, 0);
        self.enemy_c6_set_speed_by_level(e);
        e.m.sb(0x80, 0x78);
        self.enemy_c6_atk_dir(e);
        self.c6_atk_walk(e); // falls into 0x39CF6
    }

    /// 0x39CF6: the step 2: a borrow of the timer starts the command again, the arrival goes on with the script.
    fn c6_atk_walk(&mut self, e: &mut Enemy) {
        let (t, borrow) = e.m.b(0x80).overflowing_sub(1);
        e.m.sb(0x80, t);
        if borrow {
            return self.c6_cmd_walk(e); // bra 0x39CD0
        }
        if self.enemy_c6_arrived_at_slot(e) {
            return self.c6_script_next(e);
        }
        self.enemy_c6_atk_aim(e);
        self.enemy_move_dir(&mut e.b); // 0x1862
        self.enemy_walk_anim(&mut e.b); // 0x324D4
    }

    // ---------------------------------------------------------------- command 2, the leap (step 4)

    /// 0x39D16.
    fn c6_cmd_leap(&mut self, e: &mut Enemy) {
        self.enemy_c6_atk_slot(e);
        e.b.step = 4;
        e.m.sb(0xAD, 0);
        self.c6_atk_leap(e);
    }

    /// 0x39D24: the phases 0 (the take off to the slot point: 44 frames of flight), 2 (in the air), 4 (the landing).
    fn c6_atk_leap(&mut self, e: &mut Enemy) {
        match e.m.b(0xAD) {
            0 => {
                // 0x39D38
                e.m.sb(0xAD, 2);
                self.enemy_anim(&mut e.b, 1);
                self.enemy_c6_atk_aim(e);
                e.b.airborne = 1;
                e.b.vy = 0x900;
                e.b.ay = -0x60;
                let (dest_x, dest_z) = (e.m.w(0x88) as i16, e.m.w(0x8A) as i16);
                e.b.vx = self.enemy_c6_leap_speed(e.b.xi(), dest_x); // 0x39A7A
                e.b.ax = 0;
                e.b.vz = self.enemy_c6_leap_speed(e.b.zi(), dest_z);
                e.b.az = 0;
            }
            2 => {
                // 0x39DC0
                if e.b.airborne != 0 {
                    if e.b.param_b == 0 {
                        e.b.motion(); // 0x189C
                    }
                    self.enemy_step(&mut e.b);
                } else {
                    e.m.sb(0xAD, 4);
                    self.enemy_anim(&mut e.b, 2);
                    self.c6_leap_land(e);
                }
            }
            _ => self.c6_leap_land(e),
        }
    }

    /// 0x39DE4: the landing animation; its end faces the hero and takes the next command.
    fn c6_leap_land(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            self.enemy_c6_face_target(e); // 0x3AC9A
            self.c6_script_next(e);
        }
    }

    // ---------------------------------------------------------------- command 4, the lunge (step 6, attack 0x24)

    /// 0x39DF8.
    fn c6_cmd_lunge(&mut self, e: &mut Enemy) {
        e.m.sb(0xAD, 0);
        e.b.step = 6;
        self.enemy_anim(&mut e.b, 4);
        self.enemy_begin_attack(&mut e.b, 0x24);
        self.c6_atk_lunge(e);
    }

    /// 0x39E22: the swing (phase 0), the hold by level (2), the recovery (4).
    fn c6_atk_lunge(&mut self, e: &mut Enemy) {
        match e.m.b(0xAD) {
            0 => {
                // 0x39E36
                if self.enemy_step(&mut e.b) {
                    e.m.sb(0xAD, 2);
                    let hold = self.assets.enemy.t.u8(LUNGE_HOLD, self.w.level as u32);
                    e.m.sb(0x80, hold);
                    self.c6_lunge_hold(e);
                }
            }
            2 => self.c6_lunge_hold(e),
            _ => {
                // 0x39E64
                if self.enemy_step(&mut e.b) {
                    self.c6_script_next(e);
                }
            }
        }
    }

    /// 0x39E50.
    fn c6_lunge_hold(&mut self, e: &mut Enemy) {
        let (t, borrow) = e.m.b(0x80).overflowing_sub(1);
        e.m.sb(0x80, t);
        if borrow {
            e.m.sb(0xAD, 4); // 0x39E58
            self.enemy_anim(&mut e.b, 5);
            if self.enemy_step(&mut e.b) {
                self.c6_script_next(e);
            }
        }
    }

    // ---------------------------------------------------------------- command 6, the charge (step 8, attack 0x25)

    /// 0x39E90.
    fn c6_cmd_charge(&mut self, e: &mut Enemy) {
        e.m.sb(0xAD, 0);
        e.b.step = 8;
        self.c6_atk_charge(e);
    }

    /// 0x39E9A: the jump at the hero (phase 0), the flight (2), the landing (4).
    fn c6_atk_charge(&mut self, e: &mut Enemy) {
        match e.m.b(0xAD) {
            0 => {
                // 0x39EAE
                e.m.sb(0xAD, 2);
                self.enemy_anim(&mut e.b, 6);
                self.enemy_begin_attack(&mut e.b, 0x25);
                e.b.airborne = 1;
                e.b.vy = 0x400;
                e.b.ay = -0x45;
                let tx = self.body(e.m.w(0x76)).xi();
                e.b.vx = if (tx as u16) < (e.b.xi() as u16) { -0x480 } else { 0x480 };
                e.b.ax = 0;
                e.b.vz = 0;
                e.b.az = 0;
            }
            2 => {
                // 0x39F0A
                if e.b.airborne != 0 {
                    if e.b.param_b == 0 {
                        e.b.motion();
                    }
                    self.enemy_step(&mut e.b);
                } else {
                    e.m.sb(0xAD, 4);
                    self.enemy_anim(&mut e.b, 2);
                }
            }
            _ => {
                // 0x39F2E
                if self.enemy_step(&mut e.b) {
                    self.c6_script_next(e);
                }
            }
        }
    }

    // ---------------------------------------------------------------- command 8, the tongue jump (step 0xA, attack 0x26)

    /// 0x39F3A.
    fn c6_cmd_tongue(&mut self, e: &mut Enemy) {
        e.m.sb(0xAD, 0);
        e.b.step = 0xA;
        self.c6_atk_tongue(e);
    }

    /// 0x39F44: the take off (phase 0), the rise (2), the fall (4), the landing (6).
    pub(super) fn c6_atk_tongue(&mut self, e: &mut Enemy) {
        match e.m.b(0xAD) {
            0 => {
                // 0x39F5A
                e.m.sb(0xAD, 2);
                self.enemy_anim(&mut e.b, 1);
                self.enemy_begin_attack(&mut e.b, 0x26);
                e.b.airborne = 1;
                e.b.vy = 0x780;
                e.b.ay = -0x60;
                let tx = self.body(e.m.w(0x76)).xi();
                let d1: i16 = if (tx as u16) < (e.b.xi() as u16) { -0x240 } else { 0x240 };
                e.b.vx = d1 >> 1;
                e.b.ax = 0;
                e.b.vz = 0;
                e.b.az = 0;
            }
            2 => {
                // 0x39FB8
                if e.b.param_b == 0 {
                    e.b.motion();
                }
                if self.enemy_step(&mut e.b) {
                    e.m.sb(0xAD, 4); // 0x39FCE
                    self.enemy_anim(&mut e.b, 7);
                    self.c6_tongue_fall(e);
                }
            }
            4 => self.c6_tongue_fall(e),
            _ => {
                // 0x39FF8
                if self.enemy_step(&mut e.b) {
                    self.c6_script_next(e);
                }
            }
        }
    }

    /// 0x39FDA.
    fn c6_tongue_fall(&mut self, e: &mut Enemy) {
        if e.b.airborne != 0 {
            e.b.motion();
            self.enemy_step(&mut e.b);
            return;
        }
        e.m.sb(0xAD, 6); // 0x39FEC
        self.enemy_anim(&mut e.b, 2);
        if self.enemy_step(&mut e.b) {
            self.c6_script_next(e);
        }
    }

    // ---------------------------------------------------------------- command 0xA, the acid spit (step 0xC)

    /// 0x3A004.
    fn c6_cmd_spit(&mut self, e: &mut Enemy) {
        e.b.step = 0xC;
        e.m.sb(0xAD, 0);
        self.c6_atk_spit(e);
    }

    /// 0x3A00E: the wind up (phase 0, then 50 frames), the spit (the frame flag +$2D launches the ball).
    fn c6_atk_spit(&mut self, e: &mut Enemy) {
        match e.m.b(0xAD) {
            0 => {
                // 0x3A022
                e.m.sb(0xAD, 2);
                self.enemy_anim(&mut e.b, 8);
                e.m.sb(0x80, 0x32);
            }
            2 => {
                // 0x3A036
                let (t, borrow) = e.m.b(0x80).overflowing_sub(1);
                e.m.sb(0x80, t);
                if !borrow {
                    self.enemy_step(&mut e.b);
                    return;
                }
                e.m.sb(0xAD, 4); // 0x3A042
                self.enemy_anim(&mut e.b, 9);
                self.c6_spit_fire(e);
            }
            _ => self.c6_spit_fire(e),
        }
    }

    /// 0x3A04E.
    fn c6_spit_fire(&mut self, e: &mut Enemy) {
        if e.b.param_b != 0 {
            self.c6_spawn_acid(e);
            e.b.param_b = 0;
        }
        if self.enemy_step(&mut e.b) {
            self.c6_script_next(e);
        }
    }

    /// 0x3A068: the acid ball A7 (class 7) 0x42 pixels in front of the mouth, 0x39 above the feet.
    fn c6_spawn_acid(&mut self, e: &Enemy) {
        let Some(slot) = self.enemies.alloc() else { return };
        let a = &mut self.enemies.slots[slot];
        a.b.live = 1;
        a.b.k20 = 0x1C;
        let d1: i16 = if e.b.facing != 0 { 0x42 } else { -0x42 };
        a.b.set_xi(e.b.xi().wrapping_add(d1));
        a.b.set_yi(e.b.yi().wrapping_add(0x39));
        a.b.set_zi(e.b.zi());
        a.b.facing = e.b.facing;
    }

    // ---------------------------------------------------------------- the hooks of the hop AI

    /// 0x3AEE4: a hero within reach (the players 1 to 3 in turn): the target becomes him (carry).
    pub(super) fn c6_tongue_reach(&mut self, e: &mut Enemy) -> bool {
        for p in 0..3 {
            let tb = &self.players[p].b;
            if tb.live == 0 || tb.sub != 0 {
                continue;
            }
            if (tb.xi().wrapping_sub(e.b.xi()).wrapping_add(0x30) as u16) >= 0x60 {
                continue;
            }
            if (tb.zi().wrapping_sub(e.b.zi()).wrapping_add(8) as u16) >= 0x10 {
                continue;
            }
            if e.b.airborne != 0 {
                if e.b.yi() == e.b.floor {
                    continue;
                }
                if (e.b.yi().wrapping_sub(tb.yi()) as u16) >= 0x30 {
                    continue;
                }
            }
            e.m.sw(0x76, Game::player_handle(p));
            return true;
        }
        false
    }

    /// 0x3AF64: a hero is in reach during the hop: on the ground the attack script starts; in the air, moving away from his side, the tongue jump
    /// (script 2 at the position 0xA: the tongue jump alone) from its landing.
    pub(super) fn c6_tongue_react(&mut self, e: &mut Enemy) {
        if e.b.airborne == 0 {
            return self.enemy_c6_start_attack(e); // bra 0x39C1A
        }
        let leaving = (e.b.vx as u16) >> 15 != 0; // rol.w #1 ; andi.w #1
        if leaving as u8 != e.b.facing {
            e.b.set_state(0x0200_060A);
            e.m.sb(0xAD, 2);
            e.m.sw(0xB4, 2);
            e.m.sw(0xB2, 0xA);
            // bra 0x39FCE: the fall of the tongue jump
            e.m.sb(0xAD, 4);
            self.enemy_anim(&mut e.b, 7);
            self.c6_tongue_fall(e);
        }
    }

    /// 0x3A0B4 sub 4 of the mode 2: the tongue grab. Its tail is the one of the alive state: the floor, the draw check, the list of the hurtable.
    pub(super) fn enemy_c6_grab_state(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => {
                // 0x3A0D8
                e.b.act += 2;
                e.m.sb(0xC2, 0);
                e.m.sb(0xC3, 0);
                self.enemy_anim(&mut e.b, 0xA);
            }
            2 => self.c6_grab_body(e),
            _ => {}
        }
        self.enemy_c6_tail(e, h);
    }

    /// 0x3A0EC: the damage of the hold and the steps of the tongue (the part 0 sets `+$C3`: 1 the tongue is out, 2 it holds, 3 it comes back).
    fn c6_grab_body(&mut self, e: &mut Enemy) {
        if e.m.b(0xC2) != 0 {
            let (c5, borrow) = e.m.b(0xC5).overflowing_sub(1);
            e.m.sb(0xC5, c5);
            if borrow {
                e.m.sb(0xC5, 0x1E);
                let target = e.m.w(0x76);
                if let Some(Slot::Player(p)) = slot_of(target) {
                    let hp = self.players[p].b.hp;
                    self.players[p].b.hp = hp.wrapping_sub(1);
                    self.hud_life_bar(p); // jsr 0x16446 with a6 = the hero
                }
            }
        }
        match e.b.step {
            0 => {
                // 0x3A12A
                if self.enemy_step(&mut e.b) {
                    e.b.step += 2;
                    e.m.sb(0xC3, 1);
                    e.m.sb(0xC5, 0);
                    self.enemy_anim(&mut e.b, 0xB);
                }
            }
            2 => {
                // 0x3A14A
                if e.m.b(0xC2) != 0 {
                    e.b.step = 4;
                    self.c6_grab_wait(e);
                } else if e.m.b(0xC3) == 3 {
                    self.c6_grab_back(e);
                } else {
                    self.enemy_step(&mut e.b);
                }
            }
            4 => self.c6_grab_wait(e),
            6 => {
                // 0x3A17C
                if e.m.b(0xC3) != 0 {
                    self.enemy_step(&mut e.b);
                    return;
                }
                e.b.step = 8; // 0x3A188
                if e.m.b(0xC4) == 0 {
                    self.enemy_anim(&mut e.b, 0xD);
                } else {
                    self.enemy_begin_attack(&mut e.b, 0x24); // 0x3A19C
                    self.enemy_anim(&mut e.b, 0xE);
                }
            }
            _ => {
                // 0x3A1BC
                if self.enemy_step(&mut e.b) {
                    self.c6_attack_end(e);
                }
            }
        }
    }

    /// 0x3A164.
    fn c6_grab_wait(&mut self, e: &mut Enemy) {
        if e.m.b(0xC3) == 3 {
            self.c6_grab_back(e);
        }
    }

    /// 0x3A16E: the tongue is back.
    fn c6_grab_back(&mut self, e: &mut Enemy) {
        e.b.step = 6;
        self.enemy_anim(&mut e.b, 0xC);
    }
}
