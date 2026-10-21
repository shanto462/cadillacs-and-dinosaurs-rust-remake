//! Class 10: the attack actions C and D (state 0xA), the paired attack (state 0x10), fall and hop,
//! the grab machine (sub 4, `+$AC` = 0) and the stomp machine (`+$AC` = 2).

use super::class10::{f, t};
use super::Enemy;
use crate::game::{Game, Handle};

impl Game {
    // ------------------------------------------------------------------ state 0xA, action C

    /// 0x3C308: action C (step 8 handler too): a charging punch with a slide, `+$A8` is its phase.
    pub(super) fn c10_sa_c(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.airborne != 0 {
            return e.b.set_state(0x0200_0E00); // 0x3CA32
        }
        match e.m.w(f::SUB) {
            0 => {
                // 0x3C328
                e.b.set_state(0x0200_0A08);
                self.c10_3c330(e, h);
            }
            2 => self.c10_3c350(e, h),
            4 => self.c10_3c362(e, h),
            6 => self.c10_3c3a2(e, h),
            8 => self.c10_3c3f6(e, h),
            0xA => self.c10_3c426(e, h),
            v => panic!("class 10 action C phase {v:#x}"),
        }
    }

    /// 0x3C330: phase 2: wind up (animation 0xD) for `table[aggression]` frames.
    fn c10_3c330(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sw(f::SUB, 2);
        self.enemy_anim(&mut e.b, 0xD);
        let v = self.assets.enemy.t.u8(t::GRAB_WAIT, self.rank.aggression as u32 & 0xFFFF);
        e.m.sb(f::T80, v);
        e.m.sb(f::WALK_KIND, 0);
        self.c10_3ccd8(e);
        self.c10_3c350(e, h);
    }

    /// 0x3C350: the wind up counts down (`bmi`: the new value is negative), then 0x3C362.
    fn c10_3c350(&mut self, e: &mut Enemy, h: Handle) {
        let b = e.m.b(f::T80).wrapping_sub(1);
        e.m.sb(f::T80, b);
        if (b as i8) < 0 {
            self.c10_3c362(e, h);
        }
    }

    /// 0x3C362: phase 6, the run: attack id 6, animation 4, speed by the level band, 100 frames.
    fn c10_3c362(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sw(f::SUB, 6);
        self.enemy_begin_attack(&mut e.b, 6);
        self.enemy_anim(&mut e.b, 4);
        let band = ((self.w.level & 0x18) >> 2) as u32;
        let id = self.assets.enemy.t.u16(t::DASH_SPEEDS, band);
        self.enemy_set_speed(&mut e.b, id);
        e.m.sb(f::T80, 0x64);
        self.c10_3c3a2(e, h);
    }

    /// 0x3C3A2: running: a sound on the frame flag, stop when stuck, after the time, or at the target.
    fn c10_3c3a2(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.param_b != 0 {
            e.b.param_b = 0;
            self.enemy_snd_a062(e.b.xi());
        }
        if Self::enemy_stuck_check(e) {
            return self.c10_3c3e6(e, h);
        }
        let b = e.m.b(f::T80).wrapping_sub(1);
        e.m.sb(f::T80, b);
        if b == 0 {
            return self.c10_3c3e6(e, h);
        }
        if self.c10_3cd34(e) {
            return self.c10_3c3f6(e, h);
        }
        if e.m.b(f::WALK_KIND) == 0 {
            self.c10_3cdd4(e);
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x3C3E6: the run is over: slide to a stop (0x3CD5A).
    fn c10_3c3e6(&mut self, e: &mut Enemy, h: Handle) {
        e.b.step = 8;
        e.m.sw(f::SUB, 8);
        self.c10_3cd5a(e);
        self.c10_3c3f6(e, h);
    }

    /// 0x3C3F6 phase 8: slide while |vx| >= 0x30, then wait 0x1E frames.
    fn c10_3c3f6(&mut self, e: &mut Enemy, h: Handle) {
        let d0 = (e.b.vx as u16).wrapping_add(0x30);
        if d0 >= 0x60 {
            self.enemy_step(&mut e.b);
            e.b.motion();
            return;
        }
        e.m.sw(f::SUB, 0xA);
        e.m.sb(f::T80, 0x1E);
        self.enemy_anim(&mut e.b, 0);
        self.c10_3c426(e, h);
    }

    /// 0x3C426 phase 0xA: wait, then the recovery 0x3C5AA.
    fn c10_3c426(&mut self, e: &mut Enemy, h: Handle) {
        let b = e.m.b(f::T80);
        e.m.sb(f::T80, b.wrapping_sub(1));
        if b == 0 {
            self.c10_3c5aa(e, h);
        }
    }

    // ------------------------------------------------------------------ state 0xA, action D

    /// 0x3C430: action D (step 0xA handler too): run punch, then a jump slam. The wind up and the
    /// start of the run are those of action C (0x3C350 -> 0x3C362), the phases from 6 on are its own.
    pub(super) fn c10_sa_d(&mut self, e: &mut Enemy, h: Handle) {
        match e.m.w(f::SUB) {
            0 => {
                // 0x3C44E
                e.b.set_state(0x0200_0A0A);
                self.c10_3c330(e, h);
            }
            2 => self.c10_3c350(e, h),
            4 => {
                // 0x3C45A
                e.m.sw(f::SUB, 6);
                self.enemy_begin_attack(&mut e.b, 6);
                self.enemy_anim(&mut e.b, 4);
                self.enemy_set_speed(&mut e.b, 0xD);
                e.m.sb(f::T80, 0x78);
                self.c10_3c48e(e, h);
            }
            6 => self.c10_3c48e(e, h),
            8 => self.c10_3c4e0(e, h),
            0xA => self.c10_3c52a(e, h),
            0xC => self.c10_3c562(e, h),
            0xE => self.c10_3c582(e, h),
            0x10 => self.c10_3c5a0(e, h),
            v => panic!("class 10 action D phase {v:#x}"),
        }
    }

    /// 0x3C48E phase 6: run at the target, to the point 0x68 in front of it.
    fn c10_3c48e(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.airborne != 0 {
            return e.b.set_state(0x0200_0E00);
        }
        if Self::enemy_stuck_check(e) {
            return self.c10_3c3e6(e, h);
        }
        let b = e.m.b(f::T80).wrapping_sub(1);
        e.m.sb(f::T80, b);
        if b == 0 {
            return self.c10_3c3e6(e, h);
        }
        if self.c10_3cd8c(e) {
            // 0x3C4CE: in position: phase 8, animation 0xA.
            e.m.sw(f::SUB, 8);
            self.enemy_anim(&mut e.b, 0xA);
            self.c10_3cdbc(e);
            return self.c10_3c4e0(e, h);
        }
        if self.c10_3cd34(e) {
            return self.c10_3c3f6(e, h);
        }
        if e.m.b(f::WALK_KIND) == 0 {
            self.c10_3cdd4(e);
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x3C4E0 phase 8: the animation, then the jump (0x3C4EA).
    fn c10_3c4e0(&mut self, e: &mut Enemy, h: Handle) {
        if !self.enemy_step(&mut e.b) {
            return;
        }
        // 0x3C4EA
        e.m.sw(f::SUB, 0xA);
        self.enemy_anim(&mut e.b, 0xB);
        self.enemy_set_speed(&mut e.b, 7);
        self.enemy_begin_attack(&mut e.b, 7);
        e.b.airborne = 1;
        e.b.vy = 0x900;
        e.b.ay = 0xFFB0u16 as i16;
        self.c10_3c52a(e, h);
    }

    /// 0x3C52A phase 0xA: in the air; the landing shakes the screen (G.$755C = 2).
    fn c10_3c52a(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.airborne != 0 {
            e.b.motion();
            self.enemy_step(&mut e.b);
            return;
        }
        self.snd_9796(e.b.xi());
        self.cam.shake_req = 2;
        e.b.airborne = 1;
        e.m.sw(f::SUB, 0xC);
        self.enemy_anim(&mut e.b, 0xE);
        e.b.vy = 0x400;
        self.c10_3c562(e, h);
    }

    /// 0x3C562 phase 0xC: the hop after the slam.
    fn c10_3c562(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.airborne != 0 {
            e.b.motion();
            self.enemy_step(&mut e.b);
            return;
        }
        e.m.sw(f::SUB, 0xE);
        self.enemy_anim(&mut e.b, 0xF);
        self.c10_3c582(e, h);
    }

    /// 0x3C582 phase 0xE: the getting up animation.
    fn c10_3c582(&mut self, e: &mut Enemy, h: Handle) {
        if !self.enemy_step(&mut e.b) {
            return;
        }
        e.m.sw(f::SUB, 0x10);
        e.m.sb(f::T80, 0);
        self.enemy_anim(&mut e.b, 0);
        self.c10_3c5a0(e, h);
    }

    /// 0x3C5A0 phase 0x10: a pause, then the recovery 0x3C5AA.
    fn c10_3c5a0(&mut self, e: &mut Enemy, h: Handle) {
        let b = e.m.b(f::T80);
        e.m.sb(f::T80, b.wrapping_sub(1));
        if b == 0 {
            self.c10_3c5aa(e, h);
        }
    }

    // ------------------------------------------------------------------ state 0xE, 0x12: fall, hop

    /// 0x3CA3C state 0xE: a fall (animation 5, then 0xF when down).
    pub(super) fn c10_se(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.step {
            0 => {
                e.b.step += 2;
                self.enemy_anim(&mut e.b, 5);
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
                self.c10_se_end(e, h);
            }
            _ => self.c10_se_end(e, h),
        }
    }

    /// 0x3CA96.
    fn c10_se_end(&mut self, e: &mut Enemy, h: Handle) {
        if self.enemy_step(&mut e.b) {
            self.c10_reset_ai(e, h);
        }
    }

    /// 0x3CAAC state 0x12: hop down a ledge.
    pub(super) fn c10_s12(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.step {
            0 => {
                e.b.step += 2;
                self.enemy_anim(&mut e.b, 5);
                e.b.vx = 0;
                e.b.ax = 0;
                e.b.vy = 0x600;
                e.b.ay = 0xFFBCu16 as i16;
                e.b.vz = 0x20;
                e.b.az = 0;
            }
            2 => {
                if e.b.param_b != 0 {
                    self.enemy_step(&mut e.b);
                    return;
                }
                e.b.airborne = 1;
                e.b.step += 2;
                self.c10_s12_air(e, h);
            }
            4 => self.c10_s12_air(e, h),
            _ => self.c10_s12_end(e, h),
        }
    }

    /// 0x3CB08: in the air; on landing animation 0xF and the end 0x3CB26 in the same frame.
    fn c10_s12_air(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.airborne != 0 {
            e.b.motion();
            self.enemy_step(&mut e.b);
            return;
        }
        e.b.step += 2;
        self.enemy_anim(&mut e.b, 0xF);
        self.c10_s12_end(e, h);
    }

    /// 0x3CB26.
    fn c10_s12_end(&mut self, e: &mut Enemy, h: Handle) {
        if self.enemy_step(&mut e.b) {
            self.c10_3c1da(e, h);
        }
    }

    // ------------------------------------------------------------------ state 0x10: paired attack

    /// 0x3BE00: two thugs of the lists A and B go for the same downed player.
    pub(super) fn c10_3be00(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0200_1000);
        self.c10_s10(e, h);
    }

    /// 0x3BE08 state 0x10.
    pub(super) fn c10_s10(&mut self, e: &mut Enemy, h: Handle) {
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
                // 0x3BE3C
                e.b.step = 2;
                self.c10_3ccc0(e);
                self.enemy_anim(&mut e.b, 3);
                self.enemy_set_speed(&mut e.b, 8);
                self.c10_3d066(e);
                self.c10_s10_walk1(e, h);
            }
            2 => self.c10_s10_walk1(e, h),
            4 => self.c10_s10_wait(e, h),
            6 => self.c10_s10_walk2(e, h),
            8 => self.c10_s10_choose(e, h),
            0xA | 0xC => {
                // 0x3BF8A / 0x3BFC6
                if self.enemy_step(&mut e.b) {
                    self.c10_s10_after_hit(e);
                }
            }
            0xE => self.c10_s10_pause(e),
            s => panic!("class 10 state 0x10 step {s:#x}"),
        }
    }

    /// 0x3BE5A: walk to the slot (8 / 9), then wait.
    fn c10_s10_walk1(&mut self, e: &mut Enemy, h: Handle) {
        self.c10_3cbc0(e);
        if !self.c10_3ceac(e) {
            self.enemy_move_dir(&mut e.b);
            self.enemy_walk_anim(&mut e.b);
            return;
        }
        e.b.step = 4;
        self.enemy_anim(&mut e.b, 0);
        self.c10_s10_wait(e, h);
    }

    /// 0x3BE7E: wait for the other thug and the player (the player must be held, or downed with
    /// both claim flags).
    fn c10_s10_wait(&mut self, e: &mut Enemy, h: Handle) {
        let tp = self.c10_target(e);
        let (action, b7e, b7f, live, sub) = {
            let p = &self.players[tp];
            (p.action, p.b7e, p.b7f, p.b.live, p.b.sub)
        };
        if action == 0x40 {
            // 0x3BEB6
            e.b.step = 6;
            let n = self.enemy_rep_count();
            e.m.sb(f::HITS_B4, n);
            self.enemy_anim(&mut e.b, 3);
            self.enemy_set_speed(&mut e.b, 8);
            self.c10_3d066(e);
            return self.c10_s10_walk2(e, h);
        }
        if action == 0x21 && (b7e == 0 || b7f == 0) {
            return self.c10_3c916(e);
        }
        if live == 0 || sub == 0 {
            self.c10_3c916(e)
        }
    }

    /// 0x3BEDA: walk to the slot beside the held player.
    fn c10_s10_walk2(&mut self, e: &mut Enemy, h: Handle) {
        self.c10_3cbc0(e);
        if !self.c10_3ceac(e) {
            self.enemy_move_dir(&mut e.b);
            self.enemy_walk_anim(&mut e.b);
            return self.c10_s10_check(e);
        }
        e.b.step = 8;
        self.c10_s10_choose(e, h);
    }

    /// 0x3BF28: the player must be alive and neither hurt (0x20 bit) nor lying (0x22, 0x23).
    fn c10_s10_check(&mut self, e: &mut Enemy) {
        let tp = self.c10_target(e);
        let (live, action) = (self.players[tp].b.live, self.players[tp].action);
        if live == 0 || action & 0x20 != 0 || action == 0x23 || action == 0x22 {
            self.c10_3c916(e);
        }
    }

    /// 0x3BEF8 step 8: hit the held player (two kinds of swing), `+$B4` swings.
    fn c10_s10_choose(&mut self, e: &mut Enemy, h: Handle) {
        let _ = h;
        let tp = self.c10_target(e);
        if self.players[tp].action != 0x40 {
            return self.c10_s10_check(e);
        }
        if self.players[tp].bc7 == 0 {
            return self.c10_3c916(e);
        }
        let b = e.m.b(f::HITS_B4).wrapping_sub(1);
        e.m.sb(f::HITS_B4, b);
        if (b as i8) < 0 {
            return self.c10_3c916(e);
        }
        let r = self.rand() & 8;
        let (step, anim, atk) = if (r >> 2) == 0 { (0xA, 6, 4) } else { (0xC, 7, 5) };
        e.b.step = step;
        self.enemy_anim(&mut e.b, anim);
        self.enemy_begin_attack(&mut e.b, atk);
        if e.m.b(f::HITS_B4) == 0 {
            e.b.knock = 1;
        }
        if self.enemy_step(&mut e.b) {
            self.c10_s10_after_hit(e);
        }
    }

    /// 0x3BFD0.
    fn c10_s10_after_hit(&mut self, e: &mut Enemy) {
        e.b.step = 0xE;
        self.enemy_anim(&mut e.b, 0);
        e.m.sb(f::T80, 0x14);
        self.c10_s10_pause(e);
    }

    /// 0x3BFE4.
    fn c10_s10_pause(&mut self, e: &mut Enemy) {
        let b = e.m.b(f::T80);
        e.m.sb(f::T80, b.wrapping_sub(1));
        if b == 0 {
            e.b.step = 8;
        }
    }
}
