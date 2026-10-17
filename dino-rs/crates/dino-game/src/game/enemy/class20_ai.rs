//! Class 20: alive frame, AI states 0, 2, 4, 8, 0xA and the action 0 / 2 / 4 machine of state 4.

use super::class20::{f, t};
use super::{Enemy, off::*};
use crate::game::{Game, Handle};

impl Game {
    /// 0x45CC4: sub 0 AI, 2 hit reaction, 4 the death routine 0x46A2E, 6 grabbed.
    pub(super) fn c20_alive(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_kill_all_check(e);
        match e.b.sub {
            0 => self.c20_ai(e, h),
            2 => self.enemy_hit_reaction(e, h),
            4 => self.enemy_dying_basic(e, h),
            6 => self.enemy_grabbed(e, h),
            s => panic!("class 20 sub {s:#x}"),
        }
        let i = self.enemy_index(h);
        self.enemy_clamp_to_playfield(e, i);
    }

    /// 0x45CF2 AI frame.
    fn c20_ai(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(POISE_T);
        if t != 0 {
            e.m.sb(POISE_T, t - 1);
            if t - 1 == 0 {
                e.b.stun = 10;
            }
        }
        match e.b.act {
            0x00 => self.c20_s0(e, h),
            0x02 => self.c20_s2(e, h),
            0x04 => self.c20_s4(e, h),
            0x06 => self.c20_s6(e, h),
            0x08 => self.c20_s8(e, h),
            0x0A => self.c20_sa(e, h),
            a => panic!("class 20 AI state {a:#x}"),
        }
        self.enemy_floor_collision(e);
        let t = e.m.b(f::OBST_T).wrapping_sub(1);
        e.m.sb(f::OBST_T, t);
        if t == 0 {
            e.m.sb(f::OBST_T, 0xC8);
            if e.b.wall & 0x80 != 0 {
                // 0x45D48: attack the obstacle: script row 0xA (the byte at +$AA is the high byte of
                // the row word), position 0x50.
                e.b.set_state(0x0200_0600);
                e.m.sb(f::ROW, 0xA);
                e.m.sw(f::POS, 0x50);
                let o = e.m.w(OBSTACLE);
                let ox = if o != 0 { self.body(o).xi() } else { e.b.xi() };
                e.b.facing = (ox.wrapping_sub(e.b.xi()) > 0) as u8;
            }
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x45D74: back to state 0 (then the tail 0x45D30).
    pub fn c20_reset_ai(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0200_0000);
        self.enemy_anim(&mut e.b, 1);
        e.m.sb(ATTACKING, 0);
        if e.m.b(WAIT) == 0 {
            e.m.sb(WAIT, 2);
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x469BE: the fall.
    pub(super) fn c20_fall_state(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0800);
    }

    // ------------------------------------------------------------------ states 0 and 2

    /// 0x45D98 state 0.
    fn c20_s0(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.airborne != 0 {
            return self.c20_fall_state(e);
        }
        let w = e.m.b(WAIT);
        if w != 0 {
            e.m.sb(WAIT, w - 1);
            if w - 1 != 0 {
                return;
            }
        }
        self.c20_45db0(e, h);
    }

    /// 0x45DB0: pick a slot beside the target and start walking (state 2).
    pub(super) fn c20_45db0(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0200_0200);
        self.enemy_anim(&mut e.b, 1);
        self.enemy_set_speed(&mut e.b, 0xB);
        self.c20_46a72(e);
        self.c20_46b96(e);
        e.m.sb(f::T80, 0x3C);
        e.m.sw(f::T_A0, 0);
        e.m.sb(ATTACKING, 0);
        self.c20_s2(e, h);
    }

    /// 0x45DE0 state 2: walk to the slot; in reach the attack decision.
    fn c20_s2(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.airborne != 0 {
            return self.c20_fall_state(e);
        }
        if self.c20_46be0(e) {
            return self.c20_45e22(e, h);
        }
        let carry = if Self::enemy_stuck_check(e) {
            let mut d = e.m.b(f::SLOT).wrapping_add(3);
            if (d as i8) >= 6 {
                d = d.wrapping_sub(6);
            }
            e.m.sb(f::SLOT, d);
            e.b.dir ^= 0x10;
            true
        } else {
            self.enemy_walk_anim_c(&mut e.b)
        };
        if carry {
            self.c20_46ab6(e);
        }
        self.enemy_move_dir(&mut e.b);
    }

    /// 0x45E22: choose an action (table 0x45E64, 16 per variant) and set it up.
    fn c20_45e22(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0200_0400);
        e.m.sb(f::T80, 0xC8);
        e.m.sb(f::A9, 0x28);
        e.m.sb(f::BUSY, 0);
        let mut r = (self.rand() & 0xF) as u32;
        if (e.b.var26 >> 8) as u8 != 0 {
            r += 0x10;
        }
        let d1 = self.assets.enemy.t.u8(t::ACTION_PICK, r) as u16;
        e.m.sw(f::ACTION, d1);
        match d1 {
            0 => self.c20_45ee6(e, h),
            2 => self.c20_45f36(e, h),
            4 => self.c20_460b4(e, h),
            v => panic!("class 20 action {v:#x}"),
        }
    }

    /// 0x45E8A state 4: the action chosen above runs until the timers end it.
    fn c20_s4(&mut self, e: &mut Enemy, h: Handle) {
        if self.c20_4716c(e) {
            return self.c20_46356(e, h);
        }
        let b = e.m.b(f::A9);
        e.m.sb(f::A9, b.wrapping_sub(1));
        if b == 0 {
            e.m.sb(f::A9, 0);
            if e.b.airborne == 0 {
                if self.c20_46d6e(e) {
                    return self.c20_46356(e, h);
                }
                return self.c20_45e22(e, h);
            }
        }
        let b = e.m.b(f::T80);
        e.m.sb(f::T80, b.wrapping_sub(1));
        if b == 0 {
            e.m.sb(f::T80, 0);
            if e.b.airborne == 0 && e.m.b(f::CHOICE) != 2 && self.c20_47120(e) {
                return self.c20_45db0(e, h);
            }
        }
        match e.m.w(f::ACTION) {
            0 => self.c20_45f0a(e, h),
            2 => self.c20_45f3c(e, h),
            4 => self.c20_460de(e, h),
            v => panic!("class 20 action {v:#x}"),
        }
    }

    /// 0x469C8 state 8: the fall (animation 0x16, then 0xF when down).
    fn c20_s8(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.step {
            0 => {
                e.b.step += 2;
                self.enemy_anim(&mut e.b, 0x16);
                e.b.vx = 0;
                e.b.ax = 0;
                e.b.vy = 0;
                e.b.ay = 0xFFE0u16 as i16;
                e.b.vz = 0;
                e.b.az = 0;
            }
            2 => {
                if e.b.airborne != 0 {
                    e.b.motion();
                    self.enemy_step(&mut e.b);
                    return;
                }
                e.b.step += 2;
                self.enemy_anim(&mut e.b, 0xF);
                if self.enemy_step(&mut e.b) {
                    self.c20_reset_ai(e, h);
                }
            }
            _ => {
                if self.enemy_step(&mut e.b) {
                    self.c20_reset_ai(e, h);
                }
            }
        }
    }

    /// 0x46342 state 0xA: wait, then the attack (0x46356). (Nothing sets state 0xA in the listing.)
    fn c20_sa(&mut self, e: &mut Enemy, h: Handle) {
        let b = e.m.b(f::T80);
        e.m.sb(f::T80, b.wrapping_sub(1));
        if b == 0 {
            self.c20_46356(e, h);
        }
    }

    // ------------------------------------------------------------------ action 0, 2, 4 of state 4

    /// 0x45EE6 (action 0 start): walk about; then 0x45F0A.
    fn c20_45ee6(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sw(f::ACTION, 0);
        e.m.sb(f::CHOICE, 0);
        e.m.sb(f::BUSY, 1);
        self.enemy_anim(&mut e.b, 1);
        let r = self.rand() & 0x1F;
        e.b.dir = r as u8;
        self.c20_45f0a(e, h);
    }

    /// 0x45F0A.
    fn c20_45f0a(&mut self, e: &mut Enemy, _h: Handle) {
        if e.b.airborne != 0 {
            return self.c20_fall_state(e);
        }
        if e.m.b(f::T80) == 0 {
            e.m.sb(f::BUSY, 0);
        }
        self.enemy_walk_anim_c(&mut e.b);
        if !self.c20_46be0(e) {
            self.c20_46c24(e);
        }
        self.c20_46d00(e);
        self.enemy_move_dir(&mut e.b);
    }

    /// 0x45F36: action 2 start.
    fn c20_45f36(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sw(f::ACTION, 2);
        self.c20_45f3c(e, h);
    }

    /// 0x45F3C action 2: a number of hops, then a jump at the player.
    fn c20_45f3c(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.step {
            0 => {
                // 0x45F64
                e.m.sb(f::CHOICE, 0);
                let r = (self.rand() & 0xF) as u32;
                let v = self.assets.enemy.t.u8(t::JUMPS, r);
                e.m.sb(f::COUNT, v);
                self.c20_45f78(e, h);
            }
            2 => self.c20_45f96(e, h),
            4 => self.c20_45fc2(e, h),
            6 => self.c20_45ff8(e, h),
            8 => self.c20_4601c(e, h),
            0xA => self.c20_4604c(e, h),
            s => panic!("class 20 action 2 step {s:#x}"),
        }
    }

    /// 0x45F78: one hop (or, when the count is spent, the run at the player).
    fn c20_45f78(&mut self, e: &mut Enemy, h: Handle) {
        e.b.step = 2;
        self.enemy_anim(&mut e.b, 0xE);
        self.c20_46d00(e);
        let n = e.m.b(f::COUNT).wrapping_sub(1);
        e.m.sb(f::COUNT, n);
        if (n as i8) < 0 {
            return self.c20_45fce(e, h);
        }
        self.c20_46060(e);
        self.c20_45f96(e, h);
    }

    /// 0x45F96: in the air keep moving; on landing the thud and the end animation (0x45FC2).
    fn c20_45f96(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.airborne != 0 {
            if e.b.param_b == 0 {
                e.b.motion();
            }
            self.enemy_step(&mut e.b);
            return;
        }
        self.snd_9c20(e.b.xi());
        e.b.step = 4;
        self.enemy_anim(&mut e.b, 0xF);
        self.c20_45fc2(e, h);
    }

    /// 0x45FC2: the end animation of a hop, then the next hop.
    fn c20_45fc2(&mut self, e: &mut Enemy, h: Handle) {
        if self.enemy_step(&mut e.b) {
            self.c20_45f78(e, h);
        }
    }

    /// 0x45FCE: aim at the target and run at it (animation 1, speed 9).
    fn c20_45fce(&mut self, e: &mut Enemy, h: Handle) {
        e.b.step = 6;
        let tp = self.c20_target(e);
        let (x, z) = (self.players[tp].b.xi() as u16, self.players[tp].b.zi() as u16);
        e.m.sw(f::DEST_X, x);
        e.m.sw(f::DEST_Z, z);
        self.c20_46bcc(e);
        self.enemy_anim(&mut e.b, 1);
        self.enemy_set_speed(&mut e.b, 9);
        self.c20_45ff8(e, h); // falls through into 0x45FF8
    }

    /// 0x45FF8: run at the target while the window test holds, then the jump (0x4600A).
    fn c20_45ff8(&mut self, e: &mut Enemy, h: Handle) {
        // `bsr 0x46BE0 ; bcc 0x4600A`: the jump comes when the window test fails.
        if self.c20_46be0(e) {
            self.enemy_move_dir(&mut e.b);
            self.enemy_step(&mut e.b);
            return;
        }
        e.b.step = 8;
        self.c20_46084(e);
        self.enemy_anim(&mut e.b, 0xE);
        self.c20_4601c(e, h);
    }

    /// 0x4601C: the jump at the player; the landing starts the end animation (0x4604C).
    fn c20_4601c(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.param_b == 0 {
            e.b.motion();
            if e.b.airborne == 0 {
                // 0x46034
                self.snd_9c20(e.b.xi());
                self.c20_46a80(e);
                e.b.step = 0xA;
                self.enemy_anim(&mut e.b, 0xF);
                return self.c20_4604c(e, h);
            }
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x4604C.
    fn c20_4604c(&mut self, e: &mut Enemy, h: Handle) {
        if self.enemy_step(&mut e.b) {
            self.c20_46d00(e);
            self.c20_reset_ai(e, h);
        }
    }
}
