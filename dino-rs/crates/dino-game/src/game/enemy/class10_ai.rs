//! Class 10: the alive frame and the AI states (0x3BB7A, states of `+$06` 0..0x12).
//! See `class10.rs` for the overview. `h` is the handle of the object being run.

use super::class10::{f, t};
use super::{Enemy, off::*};
use crate::game::{Game, Handle};

impl Game {
    /// 0x3BB7A: alive frame. Sub 0 AI, 2 hit reaction (0x2A606), 4 the grab / stomp machine, 6 held
    /// by a player (0x2B35A).
    pub(super) fn c10_alive(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_kill_all_check(e);
        match e.b.sub {
            0 => self.c10_ai(e, h),
            2 => self.enemy_hit_reaction(e, h),
            4 => self.c10_hold(e, h),
            6 => self.enemy_grabbed(e, h),
            s => panic!("class 10 sub {s:#x}"),
        }
        let i = self.enemy_index(h);
        self.enemy_clamp_to_playfield(e, i); // jmp 0x193E
    }

    /// 0x3BBA8 AI frame.
    fn c10_ai(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(POISE_T);
        if t != 0 {
            e.m.sb(POISE_T, t - 1);
            if t - 1 == 0 {
                e.b.stun = 10;
            }
        }
        match e.b.act {
            0x00 => self.c10_s0(e, h),
            0x02 => self.c10_s2_body(e, h),
            0x04 => self.c10_s4(e, h),
            0x06 => self.c10_s6(e, h),
            0x08 => self.c10_s8(e, h),
            0x0A => self.c10_sa(e, h),
            0x0C => self.c10_sc(e, h),
            0x0E => self.c10_se(e, h),
            0x10 => self.c10_s10(e, h),
            0x12 => self.c10_s12(e, h),
            a => panic!("class 10 AI state {a:#x}"),
        }
        self.enemy_floor_collision(e);
        let t = e.m.b(f::OBST_T).wrapping_sub(1);
        e.m.sb(f::OBST_T, t);
        if t == 0 {
            e.m.sb(f::OBST_T, 0xC8);
            if e.b.wall & 0x80 != 0 {
                self.c10_attack_obstacle(e); // 0x3BC12
            }
        }
        self.c10_ai_tail(e, h);
    }

    /// 0x3BBE6: the end of the AI frame (also run by 0x3BC38).
    fn c10_ai_tail(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.wall == 0x10 {
            e.b.set_state(0x0200_1200); // 0x3CAA2: hop down a ledge
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x3BC12: attack the obstacle at +$94 with kind 9.
    fn c10_attack_obstacle(&mut self, e: &mut Enemy) {
        e.m.sw(f::KIND, 9);
        e.b.set_state(0x0200_0A00);
        let o = e.m.w(OBSTACLE);
        let ox = if o != 0 { self.body(o).xi() } else { e.b.xi() };
        e.b.facing = (ox.wrapping_sub(e.b.xi()) > 0) as u8;
    }

    /// 0x3BC38: back to state 0 (runs the AI tail, so the object is registered twice when a state
    /// calls it, as in the original).
    pub fn c10_reset_ai(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0200_0000);
        self.enemy_anim(&mut e.b, 3);
        e.m.sb(ATTACKING, 0);
        if e.m.b(WAIT) == 0 {
            e.m.sb(WAIT, 2);
        }
        self.c10_ai_tail(e, h);
    }

    /// 0x3CA32: start the fall.
    fn c10_fall_state(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0E00);
    }

    /// 0x3C916: let go of the plan (the claim flags and the state).
    pub(super) fn c10_3c916(&mut self, e: &mut Enemy) {
        e.m.sb(f::CLAIM_A, 0);
        e.m.sb(f::CLAIM_B, 0);
        e.b.set_state(0x0200_0000);
    }

    // ------------------------------------------------------------------ state 0, 2, 4, 6, 0xC

    /// 0x3BC5C state 0: wait, then decide.
    fn c10_s0(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.airborne != 0 {
            return self.c10_fall_state(e);
        }
        let w = e.m.b(WAIT);
        if w != 0 {
            e.m.sb(WAIT, w - 1);
            if w - 1 != 0 {
                return;
            }
        }
        self.c10_3bc78(e, h);
    }

    /// 0x3BC78: pick a target and a slot, start walking (state 2).
    pub(super) fn c10_3bc78(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.reaction == 0x20 {
            return self.c10_3bff2(e, h);
        }
        self.c10_3d100(e);
        self.c10_pick_target_slot(e);
        self.c10_3cc8a(e);
        self.enemy_anim(&mut e.b, 3);
        e.m.sb(f::T_B1, 0);
        self.c10_3bc9a(e, h);
    }

    /// 0x3BC9A: enter state 2.
    fn c10_3bc9a(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0200_0200);
        e.m.sb(f::T80, 0x3C);
        e.m.sb(f::T_B0, 0x78);
        e.m.sw(WA0, 0);
        e.m.sw(f::PASSED_X, e.b.xi() as u16);
        e.m.sb(ATTACKING, 0);
        self.c10_s2_body(e, h);
    }

    /// 0x3BCBE state 2: walk to the slot beside the target.
    fn c10_s2_body(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.airborne != 0 {
            return self.c10_fall_state(e);
        }
        if e.b.live == 1 {
            self.combat.push(0x6A5A, h); // 0x4AF4
        }
        if e.m.b(f::CLAIM_A) != 0 {
            return self.c10_3c856(e, h);
        }
        if e.b.live == 1 {
            self.combat.push(0x6A8E, h); // 0x4B26
        }
        if e.m.b(f::CLAIM_B) != 0 {
            return self.c10_3be00(e, h);
        }
        let w = e.m.w(WA0);
        e.m.sw(WA0, w.wrapping_sub(1));
        if w == 0 {
            e.m.sw(WA0, 2);
            self.c10_3cbc0(e);
            self.c10_3d07a(e);
        }
        let b = e.m.b(f::T_B0);
        e.m.sb(f::T_B0, b.wrapping_sub(1));
        let flip = if b == 0 { true } else { self.c10_3ce7e(e) };
        if flip {
            // 0x3BD02
            e.m.sb(f::T_B0, 0x78);
            self.c10_3cf0c(e);
            return self.c10_3bc9a(e, h);
        }
        if Self::enemy_stuck_check(e) {
            self.c10_3cc56(e);
        }
        let b = e.m.b(f::T80);
        e.m.sb(f::T80, b.wrapping_sub(1));
        if b == 0 {
            e.m.sb(f::T80, 0x28);
            let r = self.rand() & 0xF;
            if (r as i16) <= 4 {
                return self.c10_3bd56(e, h);
            }
        }
        self.enemy_move_dir(&mut e.b);
        let b = e.m.b(f::T_B1);
        e.m.sb(f::T_B1, b.wrapping_sub(1));
        if b == 0 {
            e.m.sb(f::T_B1, 0x28);
            if self.c10_3cf36(e) {
                return self.c10_3bff2(e, h);
            }
        }
        self.enemy_walk_anim(&mut e.b);
    }

    /// 0x3BD56: a pause. 0xF0 frames of standing (state 6) or a short pause walking in place (state 4).
    fn c10_3bd56(&mut self, e: &mut Enemy, h: Handle) {
        let r = (self.rand() & 0xF) as u32;
        let d1 = self.assets.enemy.t.u8(t::IDLE_TIMES, r);
        e.m.sb(f::T80, d1);
        if d1 == 0xF0 {
            e.b.act = 6;
            // 0x3BDB4
            e.b.set_state(0x0200_0600);
            self.enemy_anim(&mut e.b, 1);
            self.c10_s6(e, h);
        } else {
            e.b.act = 4;
            self.c10_s4(e, h);
        }
    }

    /// 0x3BD94 state 4: a short pause, then back to state 2.
    fn c10_s4(&mut self, e: &mut Enemy, h: Handle) {
        let b = e.m.b(f::T80);
        e.m.sb(f::T80, b.wrapping_sub(1));
        if b != 0 {
            return;
        }
        e.b.act = 2;
        e.m.sb(f::T80, 0x3C);
        self.c10_3cb80(e);
        self.c10_3cc8a(e);
        self.c10_s2_body(e, h);
    }

    /// 0x3BDC4 state 6: stand for +$80 frames, then decide again.
    fn c10_s6(&mut self, e: &mut Enemy, h: Handle) {
        let b = e.m.b(f::T80);
        e.m.sb(f::T80, b.wrapping_sub(1));
        if b == 0 {
            return self.c10_3bc78(e, h);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x3BDD2 state 0xC: taunt (animation 2, sound 0x5A).
    pub(super) fn c10_sc(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.step == 0 {
            e.b.set_state(0x0200_0C02);
            self.enemy_anim(&mut e.b, 2);
            let pan = self.snd_pan_x(e.b.xi()) as u32;
            self.sound_ex(0x5A, pan, 0); // 0xA034: d1 = the pan byte of X, d2 = 0
        }
        if self.enemy_step(&mut e.b) {
            self.c10_3bc78(e, h);
        }
    }

    // ------------------------------------------------------------------ attack choice, state 8

    /// 0x3BFF2: an attack starts: pick the kind from the table (row of the variant, one `rand()`).
    pub(super) fn c10_3bff2(&mut self, e: &mut Enemy, h: Handle) {
        e.b.reaction = 0;
        e.b.set_state(0x0200_0800);
        e.m.sw(WA0, 0);
        e.m.sb(ATTACKING, 1);
        let r = (self.rand() & 0x1F) as u32;
        let row = ((e.b.var26 >> 8) as u32 & 0xFF) << 4;
        let d1 = self.assets.enemy.t.u8(t::ATTACK_PICK, r + row) as u16;
        e.m.sw(f::KIND, d1);
        if (2..7).contains(&(d1 as i16)) {
            if d1 == 2 {
                return self.c10_3c61e(e, h);
            }
            e.b.set_state(0x0200_0A00);
            return self.c10_sa(e, h);
        }
        // 0x3C04A: kinds 0, 1, 7, 8 first walk to the attack position.
        self.enemy_set_speed(&mut e.b, 0x17);
        self.c10_3d02e(e);
        self.c10_3cc8a(e);
        e.m.sb(f::T_B0, 0x3C);
        self.c10_s8(e, h);
    }

    /// 0x3C060 state 8: walk to the attack position.
    fn c10_s8(&mut self, e: &mut Enemy, h: Handle) {
        let tp = self.c10_target(e);
        {
            let p = &self.players[tp].b;
            if p.live == 0 || p.sub == 2 || p.sub == 6 {
                return self.c10_3c916(e);
            }
        }
        if e.b.airborne != 0 {
            return self.c10_fall_state(e);
        }
        let w = e.m.w(WA0);
        e.m.sw(WA0, w.wrapping_sub(1));
        if w == 0 {
            e.m.sw(WA0, 4);
            self.c10_3cbc0(e);
            self.c10_3d07a(e);
        }
        let b = e.m.b(f::T_B0);
        e.m.sb(f::T_B0, b.wrapping_sub(1));
        if b == 0 {
            return self.c10_3bff2(e, h);
        }
        if self.c10_3ceac(e) {
            // Arrived. Facing the same way as the target: only 7 of 32 start the attack.
            if e.b.facing == self.players[tp].b.facing {
                let r = self.rand() & 0x1F;
                if (r as i16) >= 7 {
                    return self.c10_3c1da(e, h);
                }
            }
            e.b.set_state(0x0200_0A00);
            return self.c10_sa(e, h);
        }
        if Self::enemy_stuck_check(e) {
            self.c10_3cc6e(e);
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_walk_anim(&mut e.b);
    }

    /// 0x3C1DA: the player in front of us is attacking us (frame class 8, last attacker): one
    /// chance in two to taunt (0x3C222), else give the plan up.
    pub(super) fn c10_3c1da(&mut self, e: &mut Enemy, h: Handle) {
        if self.c10_3c222(e, h) {
            self.c10_sc(e, h); // bcs.w 0x3BDD2
        } else {
            self.c10_3c916(e);
        }
    }

    /// 0x3C222: returns true (carry) and starts state 0xC.
    fn c10_3c222(&mut self, e: &mut Enemy, h: Handle) -> bool {
        let tp = self.c10_target(e);
        let (kind, atk) = (self.players[tp].b.kind, self.players[tp].b.attacker);
        if kind == 8 && atk == h {
            let r = self.rand() & 0x1F;
            if r < 0x10 {
                e.b.set_state(0x2000_C000);
                return true;
            }
        }
        false
    }

    // ------------------------------------------------------------------ state 0xA: the attacks

    /// 0x3C14E state 0xA: the script of the attack kind `+$A4` (table 0x3C1E6, rows of 6).
    pub(super) fn c10_sa(&mut self, e: &mut Enemy, h: Handle) {
        let b = e.b.hitstop;
        if b != 0 {
            e.b.hitstop = b - 1;
            if b - 1 != 0 {
                return;
            }
            e.b.timer = 1;
        }
        match e.b.step {
            0 => {
                // 0x3C17E
                e.b.step = 2;
                e.m.sw(f::STEP, 0);
                e.b.knock = 0;
                self.c10_sa_next(e, h);
            }
            2 => self.c10_sa_next(e, h),
            4 => self.c10_sa_punch_step(e, h),
            6 => self.c10_sa_punch_step(e, h),
            8 => self.c10_sa_c(e, h),
            0xA => self.c10_sa_d(e, h),
            0xC => self.c10_sa_recover(e, h),
            s => panic!("class 10 state 0xA step {s:#x}"),
        }
    }

    /// 0x3C18C: the next action of the script row.
    fn c10_sa_next(&mut self, e: &mut Enemy, h: Handle) {
        let tp = self.c10_target(e);
        {
            let p = &self.players[tp].b;
            if p.live == 0 {
                return self.c10_3c916(e);
            }
            if p.sub == 2 {
                return self.c10_3c1da(e, h);
            }
            if p.sub == 6 {
                return self.c10_3c916(e);
            }
        }
        e.m.sw(f::SUB, 0);
        let d1 = self.c10_script_byte(e);
        if (d1 as i8) < 0 {
            return self.c10_3c1da(e, h);
        }
        let s = e.m.w(f::STEP);
        e.m.sw(f::STEP, s.wrapping_add(1));
        match d1 {
            0 => self.c10_action_punch(e, h, 0xA04, 6, 4),
            2 => self.c10_action_punch(e, h, 0xA06, 7, 5),
            4 => self.c10_sa_c(e, h),
            6 => self.c10_sa_d(e, h),
            v => panic!("class 10 script action {v:#x}"),
        }
    }

    /// Script byte `row *6 + position` (0x3C1BC..0x3C1C2, 0x3C286..0x3C29C).
    fn c10_script_byte(&self, e: &Enemy) -> u8 {
        let i = (e.m.w(f::KIND).wrapping_mul(6)).wrapping_add(e.m.w(f::STEP));
        self.assets.enemy.t.u8(t::SCRIPTS, i as u32)
    }

    /// 0x3C260 / 0x3C2B4: a punch (`state` 0x0A04 / 0x0A06 in +$06/+$07, animation `anim`, attack id
    /// `atk`). The last action of the script row sets +$5A.
    fn c10_action_punch(&mut self, e: &mut Enemy, h: Handle, state: u16, anim: u16, atk: u8) {
        e.b.act = (state >> 8) as u8;
        e.b.step = state as u8;
        self.enemy_anim(&mut e.b, anim);
        self.enemy_begin_attack(&mut e.b, atk);
        if (self.c10_script_byte(e) as i8) < 0 {
            e.b.knock = 1;
        }
        self.c10_sa_punch_step(e, h);
    }

    /// 0x3C2A8 / 0x3C2FC: the punch animation runs; at its end the recovery 0x3C5AA.
    fn c10_sa_punch_step(&mut self, e: &mut Enemy, h: Handle) {
        if self.enemy_step(&mut e.b) {
            self.c10_3c5aa(e, h);
        }
    }

    /// 0x3C5AA: recovery pause after an action (act 0xA, step 0xC), by level, then 0x3C5C6.
    pub(super) fn c10_3c5aa(&mut self, e: &mut Enemy, h: Handle) {
        e.b.act = 0xA;
        e.b.step = 0xC;
        let l = (self.w.level & 0x1F) as u32;
        let v = self.assets.enemy.t.u8(t::RECOVER, l);
        e.m.sb(f::T80, v);
        self.enemy_anim(&mut e.b, 3);
        self.c10_sa_recover(e, h);
    }

    /// 0x3C5C6: count the pause down, then the next action of the script.
    fn c10_sa_recover(&mut self, e: &mut Enemy, h: Handle) {
        let b = e.m.b(f::T80);
        e.m.sb(f::T80, b.wrapping_sub(1));
        if b == 0 {
            self.c10_sa_next(e, h);
        }
    }
}
