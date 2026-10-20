//! Shivat, OKORI (angry, mode word +$A8 = 4, 0x4878C) and its attacks, and the walk helpers (0x48A72, 0x491F8 .. 0x4937E). Part of `class24.rs`.
//! D3b section 5.2 and 5.3. The states (+$06): 0 pick the target, 2 walk (long reach stomp, bite range claw or grab, pause, turn), 4 turn round at the screen
//! edge, 6 the attack (+$07: 0 / 2 claw 0x38, 4 / 6 tail 0x39, 8 / 0xA / 0xC three stomps 0x3A, 0xE grab), 8 after the attack, 0xA the pause.
//!
//! Owner: enemy port (M6-04).

use super::class24::o::*;
use super::{off::*, Enemy};
use crate::game::{Game, Handle};

impl Game {
    /// 0x4878C: OKORI. The leave test (0x490F6) first; after 1800 frames a roll every 10th frame (1 in 16) makes him satisfied (MANZOKU).
    pub(super) fn c24_okori(&mut self, e: &mut Enemy, h: Handle) {
        if self.c24_leave_test(e) {
            return self.c24_enter_manzoku(e); // bcs 0x48C06
        }
        e.m.sw(TIMER, e.m.w(TIMER).wrapping_add(1));
        if e.m.w(TIMER) >= 0x708 {
            let old = e.m.b(ROLL);
            e.m.sb(ROLL, old.wrapping_sub(1));
            if old == 0 {
                e.m.sb(ROLL, 0xA);
                if self.rand() & 0xF == 0 {
                    return self.c24_enter_manzoku(e); // beq 0x48C06
                }
            }
        }
        match e.b.act {
            0 => {
                // 0x487D8: the anchor, a new target, the walk animation (variant by the aggression and the berserk flag)
                e.m.sw(ANCHOR, e.b.xi() as u16);
                e.m.sw(TARGET, self.enemy_pick_target(&e.b)); // 0x32032
                e.m.sw(RETARGET, 0);
                e.b.act = e.b.act.wrapping_add(2);
                let mut d = if self.rank.aggression >= 8 { 2 } else { 1 };
                if e.m.b(BERSERK) != 0 {
                    d += 1;
                }
                e.m.sb(WALK, d);
                self.c24_walk_anim(e); // bra 0x49334
            }
            2 => self.c24_okori_walk(e, h),
            4 => {
                // 0x489D6: the turn round; after 10 turns he is satisfied
                if !self.enemy_step(&mut e.b) {
                    return;
                }
                let n = e.m.b(TURNS).wrapping_sub(1);
                e.m.sb(TURNS, n);
                if n == 0 {
                    return self.c24_enter_manzoku(e);
                }
                e.b.facing ^= 1;
                e.b.dir = e.b.dir.wrapping_add(0x10) & 0x1F;
                e.b.act = 0;
                self.c24_walk_anim(e);
            }
            6 => self.c24_attack(e, h),
            8 => self.c24_after_attack(e, h),
            0xA => {
                // 0x48A60: the pause (animation 0xA), 120 frames
                let old = e.m.b(T80);
                e.m.sb(T80, old.wrapping_sub(1));
                if old == 0 {
                    e.b.act = 0;
                }
                self.enemy_step(&mut e.b); // jmp 0x121E
            }
            _ => {}
        }
    }

    /// The target is a free hero or enemy standing: mode 2 and sub state 0 (`cmpi.b #2,$4(a0)` and `cmpi.b #0,$5(a0)`).
    fn c24_target_free(&self, e: &Enemy, h: Handle) -> bool {
        let t = e.m.w(TARGET);
        if t == h {
            return e.b.mode == 2 && e.b.sub == 0;
        }
        let b = self.body(t);
        b.mode == 2 && b.sub == 0
    }

    /// 0x48810: state 2, the walk. Every frame the cooldown (+$B4) is at 0 the long reach (0x49296) may start a stomp (14 in 16); on a new animation
    /// frame: the screen edge turns him round, the bite range (0x491F8) starts the claw or the grab, a still frame may start a pause, the target is
    /// renewed every 300 frames (a hero in sight 9 in 16, else the nearest enemy), the direction aims at the target, the step is made.
    fn c24_okori_walk(&mut self, e: &mut Enemy, h: Handle) {
        let old = e.m.w(COOLDOWN);
        e.m.sw(COOLDOWN, old.wrapping_sub(1));
        if old == 0 {
            // `subq.w #1,$b4(a6); bcc`: a borrow only from 0
            e.m.sw(COOLDOWN, 0);
            if self.c24_long_reach(e, h) && self.c24_target_free(e, h) {
                let r = (self.rand() & 0xF) as u32;
                if self.assets.enemy.t.u8(REACH_ROLL, r) != 0 {
                    e.b.act = 6;
                    e.b.step = 8;
                    return;
                }
            }
        }
        e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
        self.enemy_step(&mut e.b);
        if e.b.param_b == e.b.w34 as u8 {
            return; // no new frame: 0x489BC is the step move, which does nothing then
        }
        if self.c24_edge(e) {
            // 0x489C0: turn round (animation 0xE)
            e.b.act = 4;
            self.enemy_anim(&mut e.b, 0xE);
            e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
            return;
        }
        if e.m.w(COOLDOWN) == 0 && self.c24_bite_range(e, h) && self.c24_target_free(e, h) {
            e.b.act = 6;
            // The row of the roll is `(word at a6 - 0x7A14 << 3) & 0x30`: a RAM word outside the object that stays 0 (measured, D3b 5.3).
            let r = (self.rand() & 0xF) as u32;
            e.b.step = self.assets.enemy.t.u8(BITE_ROLL, r);
            return;
        }
        if e.b.param_a == 0 {
            let r = self.rand() & 0xF;
            if (3u16 >> r) & 1 != 0 && e.m.b(BERSERK) == 0 {
                e.m.sb(T80, 0x78);
                e.b.act = 0xA;
                return self.enemy_anim(&mut e.b, 0xA);
            }
        }
        // 0x4890A: the target: an enemy or a hero in a reaction (state word other than 0x0200) is renewed at once, else every 300 frames
        let t = e.m.w(TARGET);
        let tstate = if t == h { e.b.state() >> 16 } else { self.body(t).state() >> 16 };
        let renew = if tstate != 0x0200 {
            true
        } else {
            let old = e.m.w(RETARGET);
            e.m.sw(RETARGET, old.wrapping_sub(1));
            old == 0 // `subq.w #1,$b0(a6); bcc`: a borrow only from 0 (the word is also the fraction of the anchor)
        };
        if renew {
            e.m.sw(RETARGET, 0x12C);
            let seen = self.c24_in_sight(e);
            if !seen || (0xFE00u16 >> (self.rand() & 0xF)) & 1 != 0 {
                self.c24_nearest_enemy(e, h);
            }
        }
        self.c24_aim(e, h);
        self.c24_step_move(e);
    }

    /// 0x48A08: state 8, after the attack: when the target has just been hurt (+$C8 = 0x23, and no +$82 tag) he roars for 120 frames (animation 0xA),
    /// then the cooldown +$B4 = 300 and back to state 0.
    fn c24_after_attack(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.step == 0 {
            let t = e.m.w(TARGET);
            let (tag, action) = match self.c24_grab_target(e, h, t) {
                Some((tag, _, _, _, action)) => (tag, action),
                None => (0, 0),
            };
            if tag != 0 || action != 0x23 {
                e.m.sw(COOLDOWN, 0x12C); // 0x48A54
                e.b.act = 0;
                return;
            }
            self.enemy_anim(&mut e.b, 0xA);
            e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
            e.m.sb(T80, 0x78);
            e.b.step = e.b.step.wrapping_add(2);
            self.c24_roar(e); // jsr 0xA09E
        }
        self.enemy_step(&mut e.b); // jsr 0x121E
        let old = e.m.b(T80);
        e.m.sb(T80, old.wrapping_sub(1));
        if old == 0 {
            e.m.sw(COOLDOWN, 0x12C);
            e.b.act = 0;
        }
    }

    /// 0x48A72: the attack (+$07, table 0x48A80). The attack id and a new serial come with each swing (0x67BE).
    fn c24_attack(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.step {
            0 => {
                // 0x48A90: the claw, animation 0xB or, on the other walk foot (+$2D set), 0x11; attack 0x38
                e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
                let id = if e.b.param_b != 0 { 0x11 } else { 0xB };
                self.enemy_anim(&mut e.b, id);
                self.c24_new_swing(e, 0x38);
                e.b.step = e.b.step.wrapping_add(2);
            }
            2 => {
                // 0x48AC6: the frame with the claw box (+$2C set) re-arms the swing; at the end of the animation state 8
                if e.b.param_a != 0 {
                    self.c24_new_swing(e, 0x38);
                    e.b.param_a = 0;
                }
                if self.enemy_step(&mut e.b) {
                    e.b.act = e.b.act.wrapping_add(2);
                    e.b.step = 0;
                    e.b.param_a = (e.b.w34 >> 8) as u8;
                    e.b.param_b = e.b.w34 as u8;
                    self.c24_walk_anim(e);
                }
            }
            4 => {
                // 0x48B08: the tail swing, animation 0xC, attack 0x39 (knocks down)
                self.enemy_anim(&mut e.b, 0xC);
                e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
                self.c24_new_swing(e, 0x39);
                e.b.step = e.b.step.wrapping_add(2);
            }
            6 => {
                // 0x48B34: at the end he has turned round (state 8)
                e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
                if self.enemy_step(&mut e.b) {
                    e.b.facing ^= 1;
                    e.b.dir = e.b.dir.wrapping_add(0x10) & 0x1F;
                    e.b.act = e.b.act.wrapping_add(2);
                    e.b.step = 0;
                    self.c24_walk_anim(e);
                }
            }
            8 => {
                // 0x48B64: the stomps, animation 0xD, three of them (+$81), attack 0x3A
                self.enemy_anim(&mut e.b, 0xD);
                e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
                e.m.sb(COUNT, 3);
                self.c24_new_swing(e, 0x3A);
                e.b.step = e.b.step.wrapping_add(2);
            }
            0xA => {
                if self.enemy_step(&mut e.b) {
                    e.b.step = e.b.step.wrapping_add(2);
                }
            }
            0xC => {
                let n = e.m.b(COUNT).wrapping_sub(1);
                e.m.sb(COUNT, n);
                if n == 0 {
                    e.b.act = e.b.act.wrapping_add(2);
                    e.b.step = 0;
                } else {
                    self.enemy_anim(&mut e.b, 0xD);
                    self.c24_new_swing(e, 0x3A);
                    e.b.step = 0xA;
                }
            }
            0xE => self.c24_grab_start(e, h),
            _ => {}
        }
    }

    /// The attack id, no knock down flag, no hit stop, a new serial (0x67BE).
    fn c24_new_swing(&mut self, e: &mut Enemy, atk: u8) {
        e.b.atk = atk;
        e.b.knock = 0;
        e.b.hitstop = 0;
        e.b.serial = self.w.attack_serial;
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
    }

    // ------------------------------------------------------------------ ranges (0x491F8, 0x49296)

    /// The on-screen window of both ranges: `X - 0x30 - camera X` from 0 to 0x120 (unsigned, a borrow fails).
    fn c24_window(&self, e: &Enemy) -> bool {
        let d = e.b.xi().wrapping_sub(0x30) as u16;
        let cx = self.cam.s2.cam_x;
        d >= cx && d - cx <= 0x120
    }

    /// 0x49244: the bite test of one object: `+$00` exactly 1, depth within 11 of his, X within 17 of X -/+ 0x6D (left / right). A hit stores the target.
    fn c24_bite_test(&mut self, e: &mut Enemy, h: Handle, t: Handle) -> bool {
        let (live, tx, tz) = self.c24_view(e, h, t);
        if live != 1 {
            return false;
        }
        if (tz.wrapping_sub(e.b.zi()).wrapping_add(0xB) as u16) > 0x16 {
            return false;
        }
        let off: i16 = if e.b.facing == 0 { -0x6D } else { 0x6D };
        if (off.wrapping_add(e.b.xi()).wrapping_sub(tx).wrapping_add(0x11) as u16) > 0x22 {
            return false;
        }
        e.m.sw(TARGET, t);
        true
    }

    /// 0x492E2: the long reach test: depth 0 .. 11 in front (target Z - 11 .. 0), X within 23 of X -/+ 0x84.
    fn c24_reach_test(&mut self, e: &mut Enemy, h: Handle, t: Handle) -> bool {
        let (live, tx, tz) = self.c24_view(e, h, t);
        if live != 1 {
            return false;
        }
        if (tz.wrapping_sub(e.b.zi()).wrapping_add(0xB) as u16) > 0xB {
            return false;
        }
        let off: i16 = if e.b.facing == 0 { -0x84 } else { 0x84 };
        if (off.wrapping_add(e.b.xi()).wrapping_sub(tx).wrapping_add(0x17) as u16) > 0x2E {
            return false;
        }
        e.m.sw(TARGET, t);
        true
    }

    /// 0x491F8: bite range: on screen, then the heroes, then the enemies of last frame's list.
    pub(super) fn c24_bite_range(&mut self, e: &mut Enemy, h: Handle) -> bool {
        if !self.c24_window(e) {
            return false;
        }
        for p in 0..3 {
            if self.c24_bite_test(e, h, Game::player_handle(p)) {
                return true;
            }
        }
        for t in self.enemy_c24_hurt_list() {
            if self.c24_bite_test(e, h, t) {
                return true;
            }
        }
        false
    }

    /// 0x49296: long reach: the heroes and the FIRST enemy of the list use the reach test; the loop of the original then jumps back into the bite range loop
    /// (0x49236), so every further enemy is tested with the BITE test (a bug of the original, kept).
    pub(super) fn c24_long_reach(&mut self, e: &mut Enemy, h: Handle) -> bool {
        if !self.c24_window(e) {
            return false;
        }
        for p in 0..3 {
            if self.c24_reach_test(e, h, Game::player_handle(p)) {
                return true;
            }
        }
        for (n, t) in self.enemy_c24_hurt_list().into_iter().enumerate() {
            let hit = if n == 0 { self.c24_reach_test(e, h, t) } else { self.c24_bite_test(e, h, t) };
            if hit {
                return true;
            }
        }
        false
    }

    // ------------------------------------------------------------------ walk animation and the step per frame

    /// 0x49334: the walk animation: `+$AD` + 2 (+ 4 when angry), or + 0x14 on the other foot (+$2D set); the parameters are saved in +$34.
    pub(super) fn c24_walk_anim(&mut self, e: &mut Enemy) {
        let mut d = e.m.b(WALK);
        d = if e.b.param_b == 0 { d.wrapping_add(2) } else { d.wrapping_add(0x14) };
        if e.m.b(ANGRY) != 0 {
            d = d.wrapping_add(4);
        }
        self.enemy_anim(&mut e.b, d as u16);
        e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
    }

    /// 0x4937E: the step. On a new frame flag (+$2D against +$35) the screen shakes (G.$755C = 1) and the anchor (+$AE as a 16.16 long) and Z move by the
    /// step table of the direction (+$64): vx is the table word, the position moves by that word / 256 pixels.
    pub(super) fn c24_step_move(&mut self, e: &mut Enemy) {
        if e.b.param_b == e.b.w34 as u8 {
            return;
        }
        self.cam_shake(1);
        let k = 4 * e.b.dir as u32;
        let vx = self.assets.enemy.t.i16(e.b.speed_table, k);
        let vz = self.assets.enemy.t.i16(e.b.speed_table, k + 2);
        e.b.vx = vx;
        e.m.sl(ANCHOR, e.m.l(ANCHOR).wrapping_add(((vx as i32) << 8) as u32));
        e.b.vz = vz;
        e.b.z = e.b.z.wrapping_add((vz as i32) << 8);
    }
}
