//! Slice's moves (class 25, `class25.rs`): the decision 0x4C11C, the four runs (`+$AC`: 0 to a point beside the target, 2 the jump, 4 the
//! approach, 6 the route of three points with the afterimages), the attack scripts 0x4C40A and their actions, and the point and turn helpers
//! 0x4CF1C .. 0x4D458. D3c section 6.3. Owner: enemy port.

use super::class25::o;
use super::{off::*, Enemy};
use crate::game::{Game, Handle};

/// The tables of the handler (read at run time through `enemy.t`, dino-extract/src/boss.rs).
mod t {
    /// The attack scripts (6 bytes each, 0xFF ends) and the action offsets.
    pub const SCRIPTS: u32 = 0x4C4AE;
    /// The variant of a single boomerang by `rand() & 31`.
    pub const SINGLE: u32 = 0x4C61A;
    /// The boomerang count of a fan by `rand() & 31`; the eight fan orders (16 bytes each).
    pub const FAN_COUNT: u32 = 0x4C79A;
    pub const FAN_ORDER: u32 = 0x4C7BA;
    /// The pause after a kick: the row by `rand() & 31`, the frames by the row + (level & 0x18).
    pub const PAUSE_ROW: u32 = 0x4C870;
    pub const PAUSE: u32 = 0x4C890;
    /// Run 4's point numbers by the point number before (rows of 32).
    pub const POINT_ROWS: u32 = 0x4CDBE;
    /// The routes of run 6 (four longs by `rand() & 0xC`).
    pub const ROUTES: u32 = 0x4CEF2;
    /// The points beside the target of runs 0 and 2 (two words, X offsets); the fixed points of runs 4 and 6 (X, Z words).
    pub const BESIDE: u32 = 0x4CF52;
    pub const POINTS: u32 = 0x4CF72;
    /// The X distance from the target by script (walk and run, 0x4CF9E).
    pub const SCRIPT_DX: u32 = 0x4CFD2;
    /// The next run kind by the run kind before (rows of 32 by `+$AC` & 6).
    pub const RUN_ROWS: u32 = 0x4D194;
    /// The attack gate by level; the scripts by HP and distance (3 x 3 rows of 32).
    pub const GATE: u32 = 0x4D268;
    pub const SCRIPT_ROWS: u32 = 0x4D2E8;
    /// The speed ids of the accelerating run.
    pub const ACCEL: u32 = 0x4D476;
}

impl Game {
    /// 0x4C11C: the decision. A target that is gone or in the action class 0x23, or 600 frames, re-target (0x4CBFC). A run in progress (`+$07`
    /// set) goes on (0x4C180). Else, with `+$B3` at 0, the attack gate (0x4D242) may start an attack script (0x4C3FA); otherwise a new run:
    /// state 0x02000202, 300 frames, the run kind (0x4D16E), its point number (0x4CD5E), the start of that run (table 0x4C178).
    pub(super) fn c25_decide(&mut self, e: &mut Enemy) {
        // d1 of the original at a run's afterimage: the act table word of 0x4C04C (0xB2), or what the re-target leaves.
        let mut d1: u16 = 0xB2;
        let tgt = e.m.w(TARGET);
        let retarget = if self.c25_live(tgt) == 0 || self.c25_byte_c8(tgt) == 0x23 {
            true
        } else {
            let t = e.m.w(o::RETARGET_T);
            e.m.sw(o::RETARGET_T, t.wrapping_sub(1));
            t == 0
        };
        if retarget {
            e.m.sw(o::RETARGET_T, 600);
            d1 = self.c25_retarget(e);
        }
        if e.b.step != 0 {
            return match e.m.b(o::RUN) {
                0 => self.c25_run0(e),
                2 => self.c25_jump(e),
                4 => self.c25_run4(e),
                _ => self.c25_route(e, d1),
            };
        }
        if e.m.b(o::GATE_T) == 0 && self.c25_gate() {
            // 0x4C3FA
            e.b.set_state(0x0200_0400);
            e.m.sb(o::GATE_T, 0xB4);
            return;
        }
        e.b.set_state(0x0200_0202);
        e.m.sw(o::RUN_T, 300);
        self.c25_run_kind(e); // 0x4D16E
        self.c25_point_no(e); // 0x4CD5E
        match e.m.b(o::RUN) {
            0 => {
                // 0x4C196
                if e.m.b(o::RUN_PREV) != 0 {
                    self.enemy_anim(&mut e.b, 1);
                }
                self.c25_run_speed(e);
                e.m.sb(o::AIM_T, 0);
                e.m.sb(o::SCREEN_T, 0x0A);
                self.c25_run0(e);
            }
            2 => self.c25_jump_start(e),
            4 => {
                // 0x4C2D6
                if e.m.b(o::RUN_PREV) != 4 {
                    self.enemy_anim(&mut e.b, 1);
                }
                self.c25_run_speed(e);
                e.m.sb(o::AIM_T, 0);
                self.c25_run4(e);
            }
            _ => {
                // 0x4C324
                self.enemy_anim(&mut e.b, 4);
                e.m.sb(o::ROUTE_STEP, 0);
                e.m.sb(o::LEGS, 2);
                let r = (self.rand() & 0xC) as u32; // 0x4CEDE
                let route = self.assets.enemy.t.u32(t::ROUTES, r);
                e.m.sl(o::ROUTE, route);
                self.c25_route(e, route as u16);
            }
        }
    }

    /// A word timer counted down: true on the borrow (it was 0), which also leaves 0xFFFF.
    fn c25_count(e: &mut Enemy, at: usize) -> bool {
        let t = e.m.w(at);
        e.m.sw(at, t.wrapping_sub(1));
        t == 0
    }

    /// A byte timer counted down: true on the borrow.
    fn c25_count_b(e: &mut Enemy, at: usize) -> bool {
        let t = e.m.b(at);
        e.m.sb(at, t.wrapping_sub(1));
        t == 0
    }

    /// 0x4C1B2: run 0. Every 11 frames off screen (0x4D41E) the run becomes the jump (0x4C1FC); every 7 frames a new point and turn
    /// (0x4CF1C); at the time limit or the point (0x4D09E, 0x10 round it) the run ends on a frame with the byte `+$2D` (act 2 again); else it
    /// moves (0x1862) and animates.
    fn c25_run0(&mut self, e: &mut Enemy) {
        if Self::c25_count_b(e, o::SCREEN_T) {
            e.m.sb(o::SCREEN_T, 0x0A);
            if !self.c25_on_screen(e) {
                return self.c25_jump_start(e);
            }
        }
        if Self::c25_count(e, o::AIM_T) {
            e.m.sw(o::AIM_T, 6);
            self.c25_aim(e);
        }
        if Self::c25_count(e, o::RUN_T) || self.c25_near(e, 0x10, 0x20, 0x10, 0x20) {
            return self.c25_run_end(e);
        }
        self.enemy_move_dir(&mut e.b); // 0x1862
        self.enemy_step(&mut e.b); // 0x121E
    }

    /// 0x4C1E2 / 0x4C30A: at a frame with `+$2D` set, act 2 and step 0 (a new decision in the next frame); else the animation.
    fn c25_run_end(&mut self, e: &mut Enemy) {
        if e.b.param_b == 0 {
            self.enemy_step(&mut e.b);
            return;
        }
        e.b.act = 2;
        e.b.step = 0;
    }

    /// 0x4C1FC: the jump: run kind 2, a point and turn (0x4CF1C), animation 2, in the air with vy 0x800 and gravity -0x60; vx and vz from the
    /// distance to the point divided by 42 (0x4D214: the quotient in the high byte, the remainder * 4 in the word).
    fn c25_jump_start(&mut self, e: &mut Enemy) {
        e.m.sb(o::JUMP, 0);
        e.m.sb(o::RUN, 2);
        e.m.sb(o::RUN_PREV, 2);
        self.c25_aim(e);
        self.enemy_anim(&mut e.b, 2);
        e.b.airborne = 1;
        e.b.vy = 0x800;
        e.b.ay = -0x60;
        let dx = (e.b.xi() as u16).wrapping_sub(e.m.w(DEST_X));
        let (mut q, mut r) = Self::c25_div42(dx);
        if e.b.xi() as u16 >= e.m.w(DEST_X) {
            q = q.wrapping_neg();
            r = r.wrapping_neg();
        }
        e.b.vx = Self::c25_speed(q, r);
        e.b.ax = 0;
        let dz = (e.b.zi() as u16).wrapping_sub(e.m.w(DEST_Z));
        let (mut q, mut r) = Self::c25_div42(dz);
        if e.b.zi() as u16 >= e.m.w(DEST_Z) {
            q = q.wrapping_neg();
            if q == 0 && (r as i16) < 0 {
                q = 0xFFFF;
            }
            r = r.wrapping_neg();
        }
        e.b.vz = Self::c25_speed(q, r);
        e.b.az = 0;
    }

    /// 0x4D214: |d| (a word) divided by 42: (quotient, remainder) as words.
    fn c25_div42(d: u16) -> (u16, u16) {
        let a = (d as i16).unsigned_abs() as u32;
        ((a / 42) as u16, (a % 42) as u16)
    }

    /// `lsl.w #2,d1 ; move.w d1,v ; add.b d0,v`: the remainder * 4 as the word, the quotient added to its high byte.
    fn c25_speed(q: u16, r: u16) -> i16 {
        let w = r.wrapping_shl(2);
        let hi = ((w >> 8) as u8).wrapping_add(q as u8);
        (((hi as u16) << 8) | (w & 0xFF)) as i16
    }

    /// 0x4C28E: the jump by `+$B0` (table 0x4C29C): 0 in the air: fly (0x189C) unless the frame byte `+$2D` is set, animate; at the landing
    /// animation 3 (falls into 2); 2 at the animation's end act 2 and step 0.
    fn c25_jump(&mut self, e: &mut Enemy) {
        if e.m.b(o::JUMP) == 0 {
            if e.b.airborne != 0 {
                if e.b.param_b == 0 {
                    e.b.motion();
                }
                self.enemy_step(&mut e.b);
                return;
            }
            e.m.sb(o::JUMP, 2);
            self.enemy_anim(&mut e.b, 3);
        }
        if self.enemy_step(&mut e.b) {
            e.b.act = 2;
            e.b.step = 0;
        }
    }

    /// 0x4C2EE: run 4: every 4 frames a new point and turn; at the time limit or the point (0x4D126: 0x20 in X, 7 in Z) the run ends
    /// (0x4C30A); else it moves and animates.
    fn c25_run4(&mut self, e: &mut Enemy) {
        if Self::c25_count(e, o::AIM_T) {
            e.m.sw(o::AIM_T, 3);
            self.c25_aim(e);
        }
        if Self::c25_count(e, o::RUN_T) || self.c25_near(e, 0x20, 0x40, 7, 0x0E) {
            return self.c25_run_end(e);
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x4C33A: the route (run 6) by `+$B1` (table 0x4C348): 0 a leg starts (0x4C34E: speed id 0xB, the point number from the route byte
    /// `+$B2` (0x4CF02)); 2 the leg: an afterimage every 7 frames (0x9639E), a faster speed every 4 frames (0x4D458), a new point and turn
    /// every 4 frames, at the time limit or the point the next leg; 4 the next leg, or after the third act 2 and step 0.
    fn c25_route(&mut self, e: &mut Enemy, d1: u16) {
        match e.m.b(o::ROUTE_STEP) {
            0 => {
                e.m.sb(o::ROUTE_STEP, 2);
                e.m.sw(o::AIM_T, 0);
                e.m.sb(o::ACCEL_T, 0);
                e.m.sb(o::ACCEL, 0);
                e.m.sb(o::GHOST_T, 6);
                self.enemy_set_speed(&mut e.b, 0x0B);
                let shift = (e.m.b(o::LEGS) as u32 * 8) & 63;
                let p = e.m.l(o::ROUTE).checked_shr(shift).unwrap_or(0) & 0xFF;
                e.m.sb(SLOT, p as u8);
                self.c25_route_leg(e, p as u16);
            }
            2 => self.c25_route_leg(e, d1),
            _ => {
                if Self::c25_count_b(e, o::LEGS) {
                    e.b.act = 2;
                    e.b.step = 0;
                } else {
                    e.m.sb(o::ROUTE_STEP, 0);
                }
            }
        }
    }

    /// 0x4C370 (`d1`: the register at the afterimage call).
    fn c25_route_leg(&mut self, e: &mut Enemy, d1: u16) {
        if Self::c25_count_b(e, o::GHOST_T) {
            e.m.sb(o::GHOST_T, 6);
            self.fx_spawn_afterimage(&e.b, d1); // 0x9639E
        }
        if Self::c25_count_b(e, o::ACCEL_T) {
            e.m.sb(o::ACCEL_T, 3);
            self.c25_accel(e);
        }
        if Self::c25_count(e, o::AIM_T) {
            e.m.sw(o::AIM_T, 3);
            self.c25_aim(e);
        }
        if Self::c25_count(e, o::RUN_T) || self.c25_near(e, 0x20, 0x40, 7, 0x0E) {
            e.m.sb(o::ROUTE_STEP, e.m.b(o::ROUTE_STEP).wrapping_add(2));
            return;
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x4D458: the next speed of the accelerating run (ids 0x4D476 by `+$B9`, which grows by 2 up to 0x12).
    fn c25_accel(&self, e: &mut Enemy) {
        let mut d1 = e.m.b(o::ACCEL);
        if (d1 as i8) < 0x12 {
            d1 = d1.wrapping_add(2);
        }
        e.m.sb(o::ACCEL, d1);
        let id = self.assets.enemy.t.u16(t::ACCEL, d1 as u32);
        self.enemy_set_speed(&mut e.b, id);
    }

    // ------------------------------------------------------------------ the attack scripts

    /// 0x4C40A: act 4, the attack script. The hit stop counts down (at its end the animation steps in this frame); then by `+$07` (table
    /// 0x4C42E): 0 a new script (0x4C442), 2 the next action (0x4C460), 4 the walk, 6 the kick, 8 the side step, 0xA the single boomerang,
    /// 0xC the running slash, 0xE the slash, 0x10 the fan, 0x12 the pause after a kick.
    pub(super) fn c25_script(&mut self, e: &mut Enemy, h: Handle) {
        let hs = e.b.hitstop;
        if hs != 0 {
            e.b.hitstop = hs - 1;
            if hs != 1 {
                return;
            }
            e.b.timer = 1;
        }
        match e.b.step {
            0 => {
                // 0x4C442
                e.b.step = e.b.step.wrapping_add(2);
                e.m.sb(o::RUN, 0xFF);
                self.c25_choose_script(e); // 0x4D288
                self.c25_script_start(e, h);
            }
            2 => self.c25_next_action(e, h),
            4 => self.c25_walk(e),
            6 => self.c25_kick(e),
            8 => self.c25_side(e),
            0x0A => self.c25_single(e, h),
            0x0C => self.c25_dash(e),
            0x0E => self.c25_slash(e),
            0x10 => self.c25_fan(e, h),
            _ => self.c25_pause(e),
        }
    }

    /// 0x4C450: `+$BC` = the script * 6, then the next action.
    pub(super) fn c25_script_start(&mut self, e: &mut Enemy, h: Handle) {
        let s = e.m.w(o::SCRIPT);
        e.m.sw(o::POS, s.wrapping_mul(6));
        self.c25_next_action(e, h);
    }

    /// 0x4C460: a target that is gone or in the action classes 0x22 / 0x23 ends the script (state 0); a negative `+$BC` or the end byte
    /// starts a new script (state 0x02000400); else the action of the next script byte (table 0x4C4D2).
    fn c25_next_action(&mut self, e: &mut Enemy, h: Handle) {
        let tgt = e.m.w(TARGET);
        let c8 = self.c25_byte_c8(tgt);
        if self.c25_live(tgt) == 0 || c8 == 0x23 || c8 == 0x22 {
            e.b.set_state(0x0200_0000); // 0x4C4A4
            return;
        }
        let pos = e.m.w(o::POS);
        if (pos as i16) < 0 {
            e.b.set_state(0x0200_0400); // 0x4C49A
            return;
        }
        let d1 = self.assets.enemy.t.u8(t::SCRIPTS, pos as u32);
        if (d1 as i8) < 0 {
            e.b.set_state(0x0200_0400);
            return;
        }
        e.m.sw(o::POS, pos.wrapping_add(1));
        match d1 {
            0 => {
                // 0x4C4E4: the walk to the point beside the target.
                e.b.step = 4;
                self.enemy_anim(&mut e.b, 1);
                self.enemy_set_speed(&mut e.b, 0x1C);
                self.c25_aim_beside(e);
                e.m.sw(o::RUN_T, 300);
                self.c25_walk(e);
            }
            2 => self.c25_kick_start(e),
            4 => self.c25_side_start(e, h),
            6 => {
                // 0x4C5DA
                self.c25_face(e);
                e.b.step = 0x0A;
                self.enemy_anim(&mut e.b, 6);
                self.c25_single(e, h);
            }
            8 => {
                // 0x4C63A: the running slash's run.
                e.b.step = 0x0C;
                self.enemy_anim(&mut e.b, 4);
                self.enemy_set_speed(&mut e.b, 0x0B);
                self.c25_aim_beside(e);
                e.m.sb(o::ACCEL_T, 0);
                e.m.sb(o::ACCEL, 0);
                e.m.sb(o::GHOST_T, 6);
                e.m.sw(o::RUN_T, 0xF0);
                self.c25_dash(e);
            }
            0x0A => self.c25_slash_start(e, 7),  // 0x4C6AC
            0x10 => self.c25_slash_start(e, 0xC), // 0x4C6DA
            0x0C => {
                // 0x4C710: off screen the script 2 instead (the run, then the slash).
                if !self.c25_on_screen(e) {
                    e.b.set_state(0x0200_0402);
                    e.m.sw(o::SCRIPT, 2);
                    return self.c25_script_start(e, h);
                }
                e.b.step = 0x10;
                self.c25_face(e);
                self.enemy_anim(&mut e.b, 8);
                self.c25_fan(e, h);
            }
            _ => {
                // 0x4C73E
                e.b.step = 0x10;
                self.c25_face(e);
                self.enemy_anim(&mut e.b, 0xB);
                self.c25_fan(e, h);
            }
        }
    }

    /// 0x4C504: the walk: at the time limit a new script; at the point (0x4D0CC: 4 in X, 7 in Z) the next action; else a new point (0x4CF9E),
    /// move, animate.
    fn c25_walk(&mut self, e: &mut Enemy) {
        if Self::c25_count(e, o::RUN_T) {
            e.b.set_state(0x0200_0400);
            return;
        }
        if self.c25_near(e, 4, 8, 7, 0x0E) {
            e.b.step = 2; // 0x4C520
            return;
        }
        self.c25_aim_beside(e);
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x4C528: the kick: face the target, animation 5, attack 0x36; when the next script byte is the end it knocks down (`+$5A` = 1).
    fn c25_kick_start(&mut self, e: &mut Enemy) {
        e.b.step = 6;
        self.c25_face(e);
        self.enemy_anim(&mut e.b, 5);
        self.enemy_begin_attack(&mut e.b, 0x36);
        let next = self.assets.enemy.t.u8(t::SCRIPTS, e.m.w(o::POS) as u32);
        if (next as i8) < 0 {
            e.b.knock = 1;
        }
        self.c25_kick(e);
    }

    /// 0x4C566: the kick's animation; at its end the pause (0x4C83A).
    fn c25_kick(&mut self, e: &mut Enemy) {
        if !self.enemy_step(&mut e.b) {
            return;
        }
        // 0x4C83A: step 0x12, the pause from the tables 0x4C870 / 0x4C890 (by level & 0x18), animation 0.
        e.b.step = 0x12;
        let r = (self.rand() & 0x1F) as u32;
        let row = self.assets.enemy.t.u8(t::PAUSE_ROW, r) as u16;
        let k = row.wrapping_add(self.w.level & 0x18);
        e.m.sb(o::T80, self.assets.enemy.t.u8(t::PAUSE, k as u32));
        self.enemy_anim(&mut e.b, 0);
        self.c25_pause(e);
    }

    /// 0x4C866: the pause; at its end (the borrow) the next action.
    fn c25_pause(&mut self, e: &mut Enemy) {
        if Self::c25_count_b(e, o::T80) {
            e.b.step = 2;
        }
    }

    /// 0x4C572: script byte 4. A target within 0x40 in X (0x4D408) makes it the walk and kicks (script 0); on screen (0x4D41E), or off
    /// screen with `rand() & 31` of 0x1C and more, the side step to the target's Z (0x4C59A); else state 0.
    fn c25_side_start(&mut self, e: &mut Enemy, h: Handle) {
        let tx = self.body(e.m.w(TARGET)).xi();
        if (tx.wrapping_sub(e.b.xi()).wrapping_add(0x40) as u16) < 0x80 {
            e.m.sw(o::SCRIPT, 0);
            return self.c25_script_start(e, h);
        }
        if !self.c25_on_screen(e) {
            let r = self.rand() & 0x1F;
            if r < 0x1C {
                e.b.set_state(0x0200_0000);
                return;
            }
        }
        e.b.step = 8;
        self.enemy_anim(&mut e.b, 4);
        self.enemy_set_speed(&mut e.b, 0x1F);
        self.c25_aim_z(e);
        e.m.sw(o::RUN_T, 0xF0);
        self.c25_side(e);
    }

    /// 0x4C5BA: the side step: at the time limit state 0; at the target's Z (0x4D154) the next action; else aim (0x4CFDA), move, animate.
    fn c25_side(&mut self, e: &mut Enemy) {
        if Self::c25_count(e, o::RUN_T) {
            e.b.set_state(0x0200_0000);
            return;
        }
        if self.c25_near_z(e) {
            e.b.step = 2;
            return;
        }
        self.c25_aim_z(e);
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x4C5EC: the single boomerang: on the frame byte `+$2D` (cleared) a boomerang of the variant `rand() & 31` from 0x4C61A (0x4504E,
    /// d2 = 0); at the animation's end state 0.
    fn c25_single(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.param_b != 0 {
            e.b.param_b = 0;
            let r = (self.rand() & 0x1F) as u32;
            let v = self.assets.enemy.t.u8(t::SINGLE, r);
            self.c19_spawn(e, h, v, 0);
        }
        if self.enemy_step(&mut e.b) {
            e.b.set_state(0x0200_0000);
        }
    }

    /// 0x4C668: the running slash's run: at the time limit state 0; an afterimage every 7 frames, a faster speed every 4; at the point
    /// (0x4D0F8) the next action; else a new point, move, animate.
    fn c25_dash(&mut self, e: &mut Enemy) {
        if Self::c25_count(e, o::RUN_T) {
            e.b.set_state(0x0200_0000);
            return;
        }
        if Self::c25_count_b(e, o::GHOST_T) {
            e.m.sb(o::GHOST_T, 6);
            self.fx_spawn_afterimage(&e.b, 0x03A0); // d1: the step table word of 0x4C04C for act 4
        }
        if Self::c25_count_b(e, o::ACCEL_T) {
            e.m.sb(o::ACCEL_T, 3);
            self.c25_accel(e);
        }
        if self.c25_near(e, 4, 8, 7, 0x0E) {
            e.b.step = 2;
            return;
        }
        self.c25_aim_beside(e);
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x4C6AC / 0x4C6DA: the slash (animation 7) or the running slash (animation 0xC): face, attack 0x37, then 0x4C704.
    fn c25_slash_start(&mut self, e: &mut Enemy, anim: u16) {
        e.b.step = 0x0E;
        self.c25_face(e);
        self.enemy_anim(&mut e.b, anim);
        self.enemy_begin_attack(&mut e.b, 0x37);
        self.c25_slash(e);
    }

    /// 0x4C704: at the animation's end a new script.
    fn c25_slash(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            e.b.set_state(0x0200_0400);
        }
    }

    /// 0x4C750: the fan: on the frame byte `+$2D` (cleared) the count from 0x4C79A by `rand() & 31` and one of eight orders (`rand() & 7`);
    /// a boomerang of variant 6 for each, d2 = the order byte (0x4504E); at the animation's end state 0.
    fn c25_fan(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.param_b != 0 {
            e.b.param_b = 0;
            let r = (self.rand() & 0x1F) as u32;
            let n = self.assets.enemy.t.u8(t::FAN_COUNT, r) as u16;
            let row = ((self.rand() & 7) << 4) as u32;
            // subq.w #1,d5 ; dbra: n boomerangs (0 makes 0x10000, never in the table).
            let count = if n == 0 { 0x10000 } else { n as u32 };
            for k in 0..count {
                let d2 = self.assets.enemy.t.u8(t::FAN_ORDER, row + k);
                self.c19_spawn(e, h, 6, d2);
            }
        }
        if self.enemy_step(&mut e.b) {
            e.b.set_state(0x0200_0000);
        }
    }

    // ------------------------------------------------------------------ choices

    /// 0x4D242: the attack gate: `rand() & 31` below the byte of 0x4D268 by level (signed byte compare).
    fn c25_gate(&mut self) -> bool {
        let g = self.assets.enemy.t.u8(t::GATE, (self.w.level & 0x1F) as u32) as i8;
        let r = (self.rand() & 0x1F) as i8;
        r < g
    }

    /// 0x4D288: the script: the HP group (a quarter of `+$BE` above HP: the low group 4; half above HP: 2; else 0), the distance group
    /// ((target X - X) as unsigned, at most 0x80, / 64), `rand() & 31`.
    fn c25_choose_script(&mut self, e: &mut Enemy) {
        let start = e.m.w(o::START_HP);
        let hp = e.b.hp as i16;
        let d2: u32 = if ((start >> 2) as i16) > hp {
            4
        } else if ((start >> 1) as i16) > hp {
            2
        } else {
            0
        };
        let mut d0 = self.body(e.m.w(TARGET)).xi().wrapping_sub(e.b.xi()) as u16;
        if d0 > 0x80 {
            d0 = 0x80;
        }
        let at = (d2 << 4) * 3 + (((d0 >> 6) as u32) << 5);
        let r = (self.rand() & 0x1F) as u32;
        let s = self.assets.enemy.t.u8(t::SCRIPT_ROWS, at + r);
        e.m.sw(o::SCRIPT, s as u16);
    }

    /// 0x4D16E: the next run kind from the row of the kind before (`+$AD` keeps it) by `rand() & 31`.
    fn c25_run_kind(&mut self, e: &mut Enemy) {
        let d1 = e.m.b(o::RUN);
        e.m.sb(o::RUN_PREV, d1);
        let row = ((d1 & 6) as u32) << 4;
        let r = (self.rand() & 0x1F) as u32;
        let k = self.assets.enemy.t.u8(t::RUN_ROWS, r + row);
        e.m.sb(o::RUN, k);
    }

    /// 0x4CD5E: the point number `+$7C` by the run kind: 0 the side of the target Slice is on (1 when X - target X has no borrow), 2 the side
    /// of the screen centre the target is on, 4 a row of 0x4CDBE by the number before and `rand() & 31`, 6 unchanged.
    fn c25_point_no(&mut self, e: &mut Enemy) {
        let tx = self.body(e.m.w(TARGET)).xi() as u16;
        match e.m.b(o::RUN) {
            0 => {
                let v = (e.b.xi() as u16 >= tx) as u8;
                e.m.sb(SLOT, v);
            }
            2 => {
                let c = (self.cam.s2.cam_x as u16).wrapping_add(0xC0);
                e.m.sb(SLOT, (c >= tx) as u8);
            }
            4 => {
                let row = (e.m.b(SLOT) as u32) << 5;
                let r = (self.rand() & 0x1F) as u32;
                let v = self.assets.enemy.t.u8(t::POINT_ROWS, row + r);
                e.m.sb(SLOT, v);
            }
            _ => {}
        }
    }

    // ------------------------------------------------------------------ points, turns, tests

    /// 0x4CF1C: the point: for run kinds up to 2 (signed) beside the target (X + the word of 0x4CF52 by `+$7C`, Z + 4), else the fixed point
    /// `+$7C` of 0x4CF72; then turn toward it (0x4CFFC) and face the target (0x4D086).
    fn c25_aim(&mut self, e: &mut Enemy) {
        let k = e.m.b(SLOT) as u32;
        let tgt = self.body(e.m.w(TARGET));
        let (tx, tz) = (tgt.xi() as u16, tgt.zi() as u16);
        let tb = &self.assets.enemy.t;
        let (x, z) = if (e.m.b(o::RUN) as i8) <= 2 {
            (tb.u16(t::BESIDE, 2 * k).wrapping_add(tx), tz.wrapping_add(4))
        } else {
            (tb.u16(t::POINTS, 4 * k), tb.u16(t::POINTS, 4 * k + 2))
        };
        e.m.sw(DEST_X, x);
        e.m.sw(DEST_Z, z);
        self.c25_turn(e, x, z);
        self.c25_face(e);
    }

    /// 0x4CF9E: the point beside the target by the script (0x4CFD2): on the side Slice is on (X - target X without a borrow: the right),
    /// Z the target's; turn, face.
    fn c25_aim_beside(&mut self, e: &mut Enemy) {
        let k = e.m.w(o::SCRIPT) as u32;
        let tgt = self.body(e.m.w(TARGET));
        let (tx, tz) = (tgt.xi() as u16, tgt.zi() as u16);
        let mut d1 = self.assets.enemy.t.u16(t::SCRIPT_DX, 2 * k);
        if (e.b.xi() as u16) < tx {
            d1 = d1.wrapping_neg();
        }
        let x = d1.wrapping_add(tx);
        e.m.sw(DEST_X, x);
        e.m.sw(DEST_Z, tz);
        self.c25_turn(e, x, tz);
        self.c25_face(e);
    }

    /// 0x4CFDA: the point (X, target Z), the direction straight to it (0x103A), face.
    fn c25_aim_z(&mut self, e: &mut Enemy) {
        let tz = self.body(e.m.w(TARGET)).zi() as u16;
        let x = e.b.xi() as u16;
        e.m.sw(DEST_X, x);
        e.m.sw(DEST_Z, tz);
        e.b.dir = Self::enemy_angle_to(&e.b, x, tz);
        self.c25_face(e);
    }

    /// 0x4CFFC: turn toward the point: within 3 of its direction take it, else 3 steps (up when the difference + 3 is 0x13 or more).
    fn c25_turn(&self, e: &mut Enemy, x: u16, z: u16) {
        let d0 = Self::enemy_angle_to(&e.b, x, z);
        let d1 = e.b.dir.wrapping_sub(d0).wrapping_add(3) & 0x1F;
        let v = if d1 <= 6 {
            d0
        } else {
            let d2: u8 = if d1 >= 0x13 { 3 } else { 0xFD };
            d2.wrapping_add(e.b.dir)
        };
        e.b.dir = v & 0x1F;
    }

    /// 0x4D086: face the target (`+$24` = 1 when X - target X borrows).
    fn c25_face(&self, e: &mut Enemy) {
        let tx = self.body(e.m.w(TARGET)).xi() as u16;
        e.b.facing = ((e.b.xi() as u16) < tx) as u8;
    }

    /// The point tests 0x4D09E (0x10 / 0x20 round), 0x4D0CC and 0x4D0F8 (4 / 8, 7 / 0xE), 0x4D126 (0x20 / 0x40, 7 / 0xE): X - point + `ax`
    /// below `bx` and Z - point + `az` below `bz` (unsigned).
    fn c25_near(&self, e: &Enemy, ax: u16, bx: u16, az: u16, bz: u16) -> bool {
        let dx = (e.b.xi() as u16).wrapping_sub(e.m.w(DEST_X)).wrapping_add(ax);
        if dx >= bx {
            return false;
        }
        let dz = (e.b.zi() as u16).wrapping_sub(e.m.w(DEST_Z)).wrapping_add(az);
        dz < bz
    }

    /// 0x4D154: Z only (7 / 0xE).
    fn c25_near_z(&self, e: &Enemy) -> bool {
        let dz = (e.b.zi() as u16).wrapping_sub(e.m.w(DEST_Z)).wrapping_add(7);
        dz < 0x0E
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_jump_speed_is_the_distance_over_42() {
        // 0x4D214: 100 / 42 = 2 rest 16; the word is rest * 4 with the quotient added to its high byte.
        assert_eq!(Game::c25_div42(100), (2, 16));
        assert_eq!(Game::c25_speed(2, 16), 0x0240);
        // The distance is taken as a signed word, made positive.
        assert_eq!(Game::c25_div42((-84i16) as u16), (2, 0));
        // Negated parts (Slice right of the point): -1 rest -16 gives 0xFFC0 with 0xFF added to its high byte.
        assert_eq!(Game::c25_speed(0xFFFF, 0xFFF0), 0xFEC0u16 as i16);
    }
}
