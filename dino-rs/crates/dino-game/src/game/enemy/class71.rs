//! The Slisaurs, the bosses of Episode 7 area 3: pool A class 71 (handler 0x5C818..0x5DD58), name 0x18, table 0x5D1C0. The starter (pool B 33,
//! `prop/b33.rs`) makes two of them (three with two or three players) at X 0x580; each keeps its own target (`+$76`, re-picked every 600
//! frames), walks round it (four walk modes: walk beside, hop, walk to an arena point, run with afterimages), and after a gate by level
//! (0x5DAE8, then a 180 frame cool-down) plays a combo script chosen by its HP phase and the distance (0x5DB2E): four slashes (0x36), run
//! and one boomerang (class 19), dash with afterimages and a kick (0x37), or a fan of 5, 7 or 9 blades. The hit goes to the starter's
//! bookkeeping (0x81A7A); a dead Slisaur lies drawn for good after its 70,000 points. D3d 4, BT01 to BT03. Owner: enemy port.
//!
//! Not reached and left out: the wait state 6 set-up 0x5CCC0 (no caller; its count 0x5CCD6 is kept) and the debug marker 0x939B6.

use super::{off::*, Enemy};
use crate::game::{slot_of, Game, Handle, Slot};

/// Own bytes and words (the original offsets).
pub(super) mod o {
    /// `+$A0` word: the walk aim timer (its high byte is cleared at the walk starts). `+$A2` / `+$A4` / `+$A6` words: the present heroes;
    /// `+$A8` byte: the index of the target among them. `+$AA` word: the retarget time (600 frames).
    pub const AIM_T: usize = 0xA0;
    pub const LIST: usize = 0xA2;
    pub const PICK: usize = 0xA8;
    pub const A9: usize = 0xA9;
    pub const RETARGET: usize = 0xAA;
    /// `+$AC` the walk mode (0, 2 hop, 4, 6 run; 0xFF after a start), `+$AD` the previous one, `+$AE` word a timer.
    pub const MODE: usize = 0xAC;
    pub const PREV: usize = 0xAD;
    pub const TIMER: usize = 0xAE;
    /// `+$B0` the hop step, `+$B1` the run leg step, `+$B2` the legs left, `+$B3` the attack cool-down, `+$B4` long the leg sides.
    pub const HOP: usize = 0xB0;
    pub const LEG: usize = 0xB1;
    pub const LEGS: usize = 0xB2;
    pub const COOL: usize = 0xB3;
    pub const SIDES: usize = 0xB4;
    /// `+$B8` / `+$B9` the dash ramp timer and index; `+$BA` word the combo; `+$BC` word the script index; `+$BE` word the start HP.
    pub const RAMP_T: usize = 0xB8;
    pub const RAMP: usize = 0xB9;
    pub const COMBO: usize = 0xBA;
    pub const SCRIPT: usize = 0xBC;
    pub const MAX_HP: usize = 0xBE;
    /// `+$C0` the afterimage timer, `+$C1` the wait of state 6, `+$C6` word the starter, `+$C8` the on-screen test timer, `+$7C` the side.
    pub const GHOST_T: usize = 0xC0;
    pub const WAIT6: usize = 0xC1;
    pub const STARTER: usize = 0xC6;
    pub const SCREEN_T: usize = 0xC8;
    pub const SIDE: usize = 0x7C;
}

/// The tables of the handler (read at run time through `enemy.t`; dino-extract/src/boss.rs).
mod t {
    pub const KD_LIMIT: u32 = 0x5C9E8;
    pub const SCRIPTS: u32 = 0x5CD9A;
    pub const BLADE_TYPE: u32 = 0x5CF06;
    pub const FAN_COUNT: u32 = 0x5D058;
    pub const FAN_DIRS: u32 = 0x5D078;
    pub const WAIT_PICK: u32 = 0x5D12E;
    pub const WAIT_TIME: u32 = 0x5D14E;
    pub const FIRST_PICK: u32 = 0x5D46E;
    pub const NEXT_PICK: u32 = 0x5D4FE;
    pub const SIDE_PICK: u32 = 0x5D666;
    pub const LEG_SIDES: u32 = 0x5D79A;
    pub const BESIDE: u32 = 0x5D7F8;
    pub const POINTS: u32 = 0x5D818;
    pub const COMBO_DX: u32 = 0x5D87A;
    pub const WALK_MODE: u32 = 0x5DA3A;
    pub const GATE: u32 = 0x5DB0E;
    pub const COMBOS: u32 = 0x5DB8E;
    pub const WALK_SPEED: u32 = 0x5DCEE;
    pub const RAMP: u32 = 0x5DD1C;
}

/// The player bodies (G.$3274, $33F4, $3574).
const PLAYERS: [Handle; 3] = [0xB274, 0xB3F4, 0xB574];

impl Game {
    /// 0x5C818 (mode table 0x5C824): 0 the init (`+$05` 0, 0x5C83E) and the wait for the scene (`+$05` 2, 0x5C8D6), 2 the fight 0x5C8FC,
    /// 4 the death 0x5D18C, 6 the free 0x5D1BA.
    pub(super) fn enemy_class71(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 if e.b.sub == 0 => self.c71_init(e),
            0 => {
                // 0x5C8D6 (state 0, 0x5C8EC): while G.$4FC the scene runs; then the HUD bar and the start (0x5C95E); the draw check.
                if self.spawn.g.b4fc == 0 {
                    self.hud.boss = h; // G.-$70BC
                    self.c71_recover(e, h);
                }
                self.enemy_draw_check(&mut e.b, h);
            }
            2 => self.c71_fight(e, h),
            4 => {
                // 0x5D18C (table 0x5D19A): 0 the score (0x325E6, 70,000), `+$80` 0x3C; 2 lies drawn for good.
                if e.b.sub == 0 {
                    let b = e.b.clone();
                    self.enemy_award_kill_score(&b);
                    e.m.sw(T80, 0x3C);
                    e.b.sub = e.b.sub.wrapping_add(2);
                }
                self.enemy_draw_check(&mut e.b, h);
            }
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x5C83E: name 0x18, `+$84` 0x60, the table 0x5D1C0, animation 0, the box table 0x103000, the walk speed by level (0x5DCDA), HP row 0 of
    /// 0x31E14 (kept in `+$BE`), poise 11, `+$7D` 0xFF, `+$63` 2, `+$72` 0x14, the target, the floor, the hero list (0x5D426); `+$05` 2.
    fn c71_init(&mut self, e: &mut Enemy) {
        e.b.id = 0x18;
        e.m.sw(W84, 0x60);
        e.b.anim_table = 0x5D1C0;
        self.enemy_anim(&mut e.b, 0);
        e.b.box_table = 0x103000;
        e.b.multi_box = false;
        self.c71_walk_speed(e);
        self.enemy_set_boss_hp(&mut e.b, 0);
        e.m.sw(o::MAX_HP, e.b.hp);
        e.b.stun = 0xB;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.m.sb(ATTACKING, 0xFF);
        e.m.sb(B87, 0);
        e.b.b63 = 2;
        e.b.b72 = 0x14;
        for k in [o::MODE, o::PREV, o::HOP, o::RAMP_T, o::RAMP] {
            e.m.sb(k, 0);
        }
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        self.enemy_snap_to_floor(e); // 0x3140C
        self.c71_targets_init(e);
        e.b.sub = e.b.sub.wrapping_add(2);
    }

    /// 0x5C95E: the start of the fight, also the recover entry of `+$84` 0x60 (table 0x2B56C): state 0x02000000, animation 0, the pick wait
    /// `+$7A` 2 (unless set), retarget in 600 frames, walk mode 0xFF; after a knock down a roll (0x5C9A2) may start the fan (on screen) or the
    /// dash and kick (off screen) at once (0x5C9EE).
    pub(super) fn c71_recover(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0200_0000);
        self.enemy_anim(&mut e.b, 0);
        let d0 = self.c71_frame_word(e);
        if e.m.b(WAIT) == 0 {
            e.m.sb(WAIT, 2);
        }
        e.m.sw(o::RETARGET, 0x258);
        e.m.sb(o::MODE, 0xFF);
        e.m.sb(o::A9, 0);
        if self.c71_knockdown_roll(e, d0) {
            e.b.set_state(0x0200_0402);
            let combo = if self.c71_on_screen(e) { 4 } else { 5 };
            e.m.sw(o::COMBO, combo);
            self.c71_combo_start(e, h);
        }
    }

    /// d0 left by the animation start 0x120E (0x1236): the frame record's first word (bit 15 off) when it is negative, else the second
    /// word when that is not negative, else the first.
    fn c71_frame_word(&self, e: &Enemy) -> u16 {
        let f = self.assets.sprites.frame(e.b.frame);
        if f.w0 & 0x8000 != 0 {
            f.w0 & 0x7FFF
        } else if (f.w1 as i16) >= 0 {
            f.w1
        } else {
            f.w0
        }
    }

    /// 0x5C9A2: after a knock down (`+$7B` 4): a limit of 0x5C9E8 by d0 against a quarter / a half of `+$BE` (d0 is the animation start's
    /// leftover, not HP); `rand & 31` below it.
    fn c71_knockdown_roll(&mut self, e: &Enemy, d0: u16) -> bool {
        if e.b.reaction != 4 {
            return false;
        }
        let be = e.m.w(o::MAX_HP);
        let d0 = d0 as i16;
        let k = if d0 <= (be >> 2) as i16 {
            0
        } else if d0 <= (be >> 1) as i16 {
            2
        } else {
            4
        };
        let limit = self.assets.enemy.t.u16(t::KD_LIMIT, k);
        let r = self.rand() & 0x1F;
        r < limit
    }

    /// 0x5C8FC: the cool-down `+$B3` down (not below 0); by `+$05` (the long table 0x5C914): 0 alive 0x5C924, 2 the hit 0x81A7A, 4 nothing,
    /// 6 held (0x2B35A).
    fn c71_fight(&mut self, e: &mut Enemy, h: Handle) {
        let c = e.m.b(o::COOL);
        e.m.sb(o::COOL, c.saturating_sub(1));
        match e.b.sub {
            0 => self.c71_alive(e, h),
            2 => self.c71_hit(e, h),
            4 => {}
            _ => self.enemy_grabbed(e, h),
        }
    }

    /// 0x5C924: the poise refill (11); the state of `+$06` (table 0x5C956: 0 the pick wait, 2 the walk, 4 the attack, 6 the wait); the terrain
    /// (0x12CB4), the draw check, the hurtable list. The jump table's word stays in d1 (0x3C, 0xB2, 0x3A0, 0x380): the afterimages get it.
    fn c71_alive(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(POISE_T);
        if t != 0 {
            e.m.sb(POISE_T, t - 1);
            if t == 1 {
                e.b.stun = 0xB;
            }
        }
        match e.b.act {
            0 => {
                // 0x5C992
                let w = e.m.b(WAIT);
                if w == 0 {
                    self.c71_walk(e, h, 0x3C);
                } else {
                    e.m.sb(WAIT, w - 1);
                    if w == 1 {
                        self.c71_walk(e, h, 0x3C);
                    }
                }
            }
            2 => self.c71_walk(e, h, 0xB2),
            4 => self.c71_attack(e, h),
            _ => {
                // 0x5CCD6
                let c = e.m.b(o::WAIT6);
                e.m.sb(o::WAIT6, c.wrapping_sub(1));
                if c == 0 {
                    e.b.set_state(0x0200_0000);
                } else {
                    self.enemy_step(&mut e.b);
                }
            }
        }
        if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
            e.m.sw(OBSTACLE, ob);
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    // ------------------------------------------------------------------ the walk (state 2)

    /// 0x5CA08: a target that is gone or held (`+$C8` 0x23) and every 600 frames: retarget (0x5D4A4). Not yet walking (`+$07` 0): with no
    /// cool-down the gate (0x5DAE8: the attack, cool-down 180); else state 0x02000202, `+$AE` 300, the next walk mode (0x5DA14) and side
    /// (0x5D606), its start (table 0x5CA64). Walking: the mode's run (table 0x5CA7A).
    fn c71_walk(&mut self, e: &mut Enemy, _h: Handle, mut d1: u16) {
        let t = e.m.w(TARGET);
        let gone = match slot_of(t) {
            Some(Slot::Player(p)) => self.players[p].b.live == 0 || self.players[p].action == 0x23,
            _ => self.body(t).live == 0,
        };
        let r = e.m.w(o::RETARGET);
        if gone || r == 0 {
            e.m.sw(o::RETARGET, 0x258);
            d1 = self.c71_retarget(e);
        } else {
            e.m.sw(o::RETARGET, r - 1);
        }
        if e.b.step != 0 {
            return match e.m.b(o::MODE) {
                0 => self.c71_walk0_run(e),
                2 => self.c71_hop_run(e),
                4 => self.c71_walk4_run(e),
                _ => self.c71_run6(e, d1),
            };
        }
        if e.m.b(o::COOL) == 0 && self.c71_gate() {
            e.b.set_state(0x0200_0400);
            e.m.sb(o::COOL, 0xB4);
            return;
        }
        e.b.set_state(0x0200_0202);
        e.m.sw(o::TIMER, 0x12C);
        self.c71_pick_mode(e);
        self.c71_side(e);
        match e.m.b(o::MODE) {
            0 => {
                // 0x5CA82
                if e.m.b(o::PREV) != 0 {
                    self.enemy_anim(&mut e.b, 1);
                }
                self.c71_walk_speed(e);
                e.m.sb(o::AIM_T, 0);
                e.m.sb(o::SCREEN_T, 0xA);
                self.c71_walk0_run(e);
            }
            2 => self.c71_hop_start(e),
            4 => {
                // 0x5CBC2
                if e.m.b(o::PREV) != 4 {
                    self.enemy_anim(&mut e.b, 1);
                }
                self.c71_walk_speed(e);
                e.m.sb(o::AIM_T, 0);
                self.c71_walk4_run(e);
            }
            _ => {
                // 0x5CC10
                self.enemy_anim(&mut e.b, 4);
                e.m.sb(o::LEG, 0);
                e.m.sb(o::LEGS, 2);
                let r = (self.rand() & 0xC) as u32;
                e.m.sl(o::SIDES, self.assets.enemy.t.u32(t::LEG_SIDES, r));
                self.c71_run6(e, d1);
            }
        }
    }

    /// 0x5CA9E: walk mode 0: every 11 frames off screen the hop instead; every 7 frames the point beside the target (0x5D7C4); at the end of
    /// `+$AE` or at the point (0x10 round it) the walk ends at a frame with `+$2D` (state 0x0200); else move (0x1862) and animate.
    fn c71_walk0_run(&mut self, e: &mut Enemy) {
        let c = e.m.b(o::SCREEN_T);
        e.m.sb(o::SCREEN_T, c.wrapping_sub(1));
        if c == 0 {
            e.m.sb(o::SCREEN_T, 0xA);
            if !self.c71_on_screen(e) {
                return self.c71_hop_start(e);
            }
        }
        if Self::c71_count_w(e, o::AIM_T) {
            e.m.sw(o::AIM_T, 6);
            self.c71_dest_by_side(e);
        }
        let done = Self::c71_count_w(e, o::TIMER) || Self::c71_at(e, 0x10, 0x20, 0x10, 0x20);
        self.c71_walk_end(e, done);
    }

    /// 0x5CBDA: walk mode 4: every 4 frames the point (an arena point, 0x5D7C4); the end at `+$AE` or the point (0x20 / 7 round it).
    fn c71_walk4_run(&mut self, e: &mut Enemy) {
        if Self::c71_count_w(e, o::AIM_T) {
            e.m.sw(o::AIM_T, 3);
            self.c71_dest_by_side(e);
        }
        let done = Self::c71_count_w(e, o::TIMER) || Self::c71_at(e, 0x20, 0x40, 7, 0xE);
        self.c71_walk_end(e, done);
    }

    /// 0x5CACE / 0x5CBF6: at the end the walk stops at a frame with `+$2D` (`+$06..+$07` = 0x0200); else (or not yet at such a frame) move
    /// and / or animate.
    fn c71_walk_end(&mut self, e: &mut Enemy, done: bool) {
        if done {
            if e.b.param_b != 0 {
                e.b.act = 2;
                e.b.step = 0;
                return;
            }
        } else {
            self.enemy_move_dir(&mut e.b); // 0x1862
        }
        self.enemy_step(&mut e.b);
    }

    /// A word counter down by one; true on the borrow (it was 0; the word is then 0xFFFF).
    fn c71_count_w(e: &mut Enemy, off: usize) -> bool {
        let v = e.m.w(off);
        e.m.sw(off, v.wrapping_sub(1));
        v == 0
    }

    /// 0x5CAE8: the hop: the point (0x5D7C4), animation 2, in the air, vy 0x800, gravity -0x60; vx / vz so that it lands there in 42 frames
    /// (|d| / 42 into the high byte, the remainder * 4 below it).
    fn c71_hop_start(&mut self, e: &mut Enemy) {
        e.m.sb(o::HOP, 0);
        e.m.sb(o::MODE, 2);
        e.m.sb(o::PREV, 2);
        self.c71_dest_by_side(e);
        self.enemy_anim(&mut e.b, 2);
        e.b.airborne = 1;
        e.b.vy = 0x800;
        e.b.ay = -0x60;
        let (x, dx) = (e.b.xi() as u16, e.m.w(DEST_X));
        e.b.vx = Self::c71_hop_speed(x, dx, false) as i16;
        e.b.ax = 0;
        let (z, dz) = (e.b.zi() as u16, e.m.w(DEST_Z));
        e.b.vz = Self::c71_hop_speed(z, dz, true) as i16;
        e.b.az = 0;
    }

    /// 0x5CB16 / 0x5CB42 with the division 0x5DABA: |pos - dest| (a word) / 42; toward the point (negated when pos is not below it).
    fn c71_hop_speed(pos: u16, dest: u16, z: bool) -> u16 {
        let d = pos.wrapping_sub(dest);
        let a = if (d as i16) < 0 { d.wrapping_neg() } else { d };
        let (mut q, mut r) = (a / 42, a % 42);
        if pos >= dest {
            q = q.wrapping_neg();
            if z && q == 0 && (r as i16) < 0 {
                q = 0xFFFF;
            }
            r = r.wrapping_neg();
        }
        let v = r.wrapping_shl(2);
        let hi = ((v >> 8) as u8).wrapping_add(q as u8);
        ((hi as u16) << 8) | (v & 0xFF)
    }

    /// 0x5CB7A by `+$B0` (table 0x5CB88): 0 in the air (fly unless `+$2D`, animate); landed: animation 3, then 2: animate and at the end the
    /// walk stops (`+$06..+$07` = 0x0200).
    fn c71_hop_run(&mut self, e: &mut Enemy) {
        if e.m.b(o::HOP) == 0 {
            if e.b.airborne != 0 {
                if e.b.param_b == 0 {
                    e.b.motion(); // 0x189C
                }
                self.enemy_step(&mut e.b);
                return;
            }
            e.m.sb(o::HOP, 2);
            self.enemy_anim(&mut e.b, 3);
        }
        if self.enemy_step(&mut e.b) {
            e.b.act = 2;
            e.b.step = 0;
        }
    }

    /// 0x5CC26 by `+$B1` (table 0x5CC34): the run with afterimages in three legs: 0 a leg starts (speed 0xB, the side from `+$B4`); 2 an
    /// afterimage every 13 frames (0x9639E, its `+$26` = d1), the speed ramp every 4 frames (0x5DCFE), the point every 4 frames; the leg
    /// ends at `+$AE` or at the point (0x20 / 7); 4 the next leg, after the last the walk stops.
    fn c71_run6(&mut self, e: &mut Enemy, d1: u16) {
        match e.m.b(o::LEG) {
            0 | 2 => {
                if e.m.b(o::LEG) == 0 {
                    e.m.sb(o::LEG, 2);
                    e.m.sw(o::AIM_T, 0);
                    e.m.sb(o::RAMP_T, 0);
                    e.m.sb(o::RAMP, 0);
                    e.m.sb(o::GHOST_T, 6);
                    self.enemy_set_speed(&mut e.b, 0xB);
                    let sides = e.m.l(o::SIDES);
                    let s = (sides >> (e.m.b(o::LEGS) as u32 * 8)) & 0xFF;
                    e.m.sb(o::SIDE, s as u8);
                }
                let c = e.m.b(o::GHOST_T);
                e.m.sb(o::GHOST_T, c.wrapping_sub(1));
                if c == 0 {
                    e.m.sb(o::GHOST_T, 0xC);
                    let b = e.b.clone();
                    self.fx_spawn_afterimage(&b, d1); // 0x9639E
                }
                let c = e.m.b(o::RAMP_T);
                e.m.sb(o::RAMP_T, c.wrapping_sub(1));
                if c == 0 {
                    e.m.sb(o::RAMP_T, 3);
                    self.c71_ramp(e);
                }
                if Self::c71_count_w(e, o::AIM_T) {
                    e.m.sw(o::AIM_T, 3);
                    self.c71_dest_by_side(e);
                }
                if Self::c71_count_w(e, o::TIMER) || Self::c71_at(e, 0x20, 0x40, 7, 0xE) {
                    e.m.sb(o::LEG, e.m.b(o::LEG).wrapping_add(2));
                    return;
                }
                self.enemy_move_dir(&mut e.b);
                self.enemy_step(&mut e.b);
            }
            _ => {
                let n = e.m.b(o::LEGS);
                e.m.sb(o::LEGS, n.wrapping_sub(1));
                if n == 0 {
                    e.b.act = 2;
                    e.b.step = 0;
                } else {
                    e.m.sb(o::LEG, 0);
                }
            }
        }
    }

    // ------------------------------------------------------------------ the attack (state 4)

    /// 0x5CCF6: the hit stop holds (its end steps the animation); by `+$07` (table 0x5CD1A): 0 a new combo (0x5DB2E), 2 the next action of
    /// the script (0x5CD4C), 4 the approach, 6 a slash, 8 the run, 0xA the boomerang, 0xC the dash, 0xE the kick, 0x10 the fan, 0x12 the
    /// pause after a slash.
    fn c71_attack(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.hitstop != 0 {
            e.b.hitstop -= 1;
            if e.b.hitstop != 0 {
                return;
            }
            e.b.timer = 1;
        }
        match e.b.step {
            0 => {
                e.b.step = 2;
                e.m.sb(o::MODE, 0xFF);
                self.c71_combo_pick(e);
                self.c71_combo_start(e, h);
            }
            2 => self.c71_next(e, h),
            4 => self.c71_approach(e),
            6 => {
                // 0x5CE52
                if self.enemy_step(&mut e.b) {
                    self.c71_pause_start(e);
                }
            }
            8 => self.c71_run(e),
            0xA => self.c71_blade(e, h),
            0xC => self.c71_dash(e),
            0xE => {
                // 0x5CFC2
                if self.enemy_step(&mut e.b) {
                    e.b.set_state(0x0200_0400);
                }
            }
            0x10 => self.c71_fan(e, h),
            _ => {
                // 0x5D124
                let c = e.m.b(T80);
                e.m.sb(T80, c.wrapping_sub(1));
                if c == 0 {
                    e.b.step = 2;
                }
            }
        }
    }

    /// 0x5CD3C: the script of the combo `+$BA` (6 bytes each, 0x5CD9A) from its start.
    fn c71_combo_start(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sw(o::SCRIPT, e.m.w(o::COMBO).wrapping_mul(6));
        self.c71_next(e, h);
    }

    /// 0x5CD4C: a target that is gone, held (0x23) or down (0x22) ends the attack (state 0); a negative entry ends the combo (state
    /// 0x02000400: the next one at once); else the action (table 0x5CDBE).
    fn c71_next(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(TARGET);
        let (live, action) = match slot_of(t) {
            Some(Slot::Player(p)) => (self.players[p].b.live, self.players[p].action),
            _ => (self.body(t).live, 0),
        };
        if live == 0 || action == 0x23 || action == 0x22 {
            return e.b.set_state(0x0200_0000);
        }
        let idx = e.m.w(o::SCRIPT);
        if (idx as i16) < 0 {
            return e.b.set_state(0x0200_0400);
        }
        let a = self.assets.enemy.t.u8(t::SCRIPTS, idx as u32);
        if (a as i8) < 0 {
            return e.b.set_state(0x0200_0400);
        }
        e.m.sw(o::SCRIPT, idx.wrapping_add(1));
        match a {
            0 => {
                // 0x5CDD0: the approach to the combo's distance (0x5D844), speed id 0x1C, 300 frames.
                e.b.step = 4;
                self.enemy_anim(&mut e.b, 1);
                self.enemy_set_speed(&mut e.b, 0x1C);
                self.c71_dest_combo(e);
                e.m.sw(o::TIMER, 0x12C);
                self.c71_approach(e);
            }
            2 => {
                // 0x5CE14: a slash (attack 0x36; the last of the chain sets `+$5A`).
                e.b.step = 6;
                self.c71_face(e);
                self.enemy_anim(&mut e.b, 5);
                self.enemy_begin_attack(&mut e.b, 0x36);
                let next = self.assets.enemy.t.u8(t::SCRIPTS, e.m.w(o::SCRIPT) as u32);
                if (next as i8) < 0 {
                    e.b.knock = 1;
                }
                if self.enemy_step(&mut e.b) {
                    self.c71_pause_start(e);
                }
            }
            4 => {
                // 0x5CE5E: within 0x40 px the slashes instead; on screen (or 4 in 32 off screen) the run, else the attack ends.
                if self.c71_near40(e) {
                    e.m.sw(o::COMBO, 0);
                    return self.c71_combo_start(e, h);
                }
                if !self.c71_on_screen(e) {
                    let r = self.rand() & 0x1F;
                    if r < 0x1C {
                        return e.b.set_state(0x0200_0000);
                    }
                }
                e.b.step = 8;
                self.enemy_anim(&mut e.b, 4);
                self.enemy_set_speed(&mut e.b, 0x1F);
                self.c71_dest_zline(e);
                e.m.sw(o::TIMER, 0xF0);
                self.c71_run(e);
            }
            6 => {
                // 0x5CEC6: the boomerang.
                self.c71_face(e);
                e.b.step = 0xA;
                self.enemy_anim(&mut e.b, 6);
                self.c71_blade(e, h);
            }
            8 => {
                // 0x5CF26: the dash (speed id 0xB, the ramp, an afterimage every 7 frames).
                e.b.step = 0xC;
                self.enemy_anim(&mut e.b, 4);
                self.enemy_set_speed(&mut e.b, 0xB);
                self.c71_dest_combo(e);
                e.m.sb(o::RAMP_T, 0);
                e.m.sb(o::RAMP, 0);
                e.m.sb(o::GHOST_T, 6);
                e.m.sw(o::TIMER, 0xF0);
                self.c71_dash(e);
            }
            0xA | 0x10 => {
                // 0x5CF98: the kick (attack 0x37).
                e.b.step = 0xE;
                self.c71_face(e);
                self.enemy_anim(&mut e.b, 7);
                self.enemy_begin_attack(&mut e.b, 0x37);
                if self.enemy_step(&mut e.b) {
                    e.b.set_state(0x0200_0400);
                }
            }
            0xC => {
                // 0x5CFCE: the fan only on screen; else the dash and kick (combo 2).
                if !self.c71_on_screen(e) {
                    e.b.set_state(0x0200_0402);
                    e.m.sw(o::COMBO, 2);
                    return self.c71_combo_start(e, h);
                }
                e.b.step = 0x10;
                self.c71_face(e);
                self.enemy_anim(&mut e.b, 8);
                self.c71_fan(e, h);
            }
            _ => {
                // 0x5CFFC: the fan.
                e.b.step = 0x10;
                self.c71_face(e);
                self.enemy_anim(&mut e.b, 0xB);
                self.c71_fan(e, h);
            }
        }
    }

    /// 0x5CDF0: the approach: `+$AE` (its end: the next combo), at the point (8 / 7 round it) the next action, else re-aim, move, animate.
    fn c71_approach(&mut self, e: &mut Enemy) {
        if Self::c71_count_w(e, o::TIMER) {
            return e.b.set_state(0x0200_0400);
        }
        if Self::c71_at(e, 8, 0x10, 7, 0xE) {
            e.b.step = 2;
            return;
        }
        self.c71_dest_combo(e);
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x5CEA6: the run to the target's depth line: `+$AE` (its end: state 0), at the line (7) the next action, else move and animate.
    fn c71_run(&mut self, e: &mut Enemy) {
        if Self::c71_count_w(e, o::TIMER) {
            return e.b.set_state(0x0200_0000);
        }
        let dz = e.b.zi().wrapping_sub(e.m.w(DEST_Z) as i16).wrapping_add(7) as u16;
        if dz < 0xE {
            e.b.step = 2;
            return;
        }
        self.c71_dest_zline(e);
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x5CED8: at the frame byte `+$2D` (cleared) a boomerang of the type 0x5CF06[`rand & 31`] (0x4504E); at the end state 0.
    fn c71_blade(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.param_b != 0 {
            e.b.param_b = 0;
            let r = (self.rand() & 0x1F) as u32;
            let d1 = self.assets.enemy.t.u8(t::BLADE_TYPE, r);
            self.c19_spawn(e, h, d1, 0);
        }
        if self.enemy_step(&mut e.b) {
            e.b.set_state(0x0200_0000);
        }
    }

    /// 0x5CF54: the dash: `+$AE` (its end: state 0), an afterimage every 7 frames (its `+$26` = 0x3A0, the jump table word in d1), the ramp
    /// every 4 frames, at the point (4 / 7 round it) the next action, else re-aim, move, animate.
    fn c71_dash(&mut self, e: &mut Enemy) {
        if Self::c71_count_w(e, o::TIMER) {
            return e.b.set_state(0x0200_0000);
        }
        let c = e.m.b(o::GHOST_T);
        e.m.sb(o::GHOST_T, c.wrapping_sub(1));
        if c == 0 {
            e.m.sb(o::GHOST_T, 6);
            let b = e.b.clone();
            self.fx_spawn_afterimage(&b, 0x3A0); // 0x9639E
        }
        let c = e.m.b(o::RAMP_T);
        e.m.sb(o::RAMP_T, c.wrapping_sub(1));
        if c == 0 {
            e.m.sb(o::RAMP_T, 3);
            self.c71_ramp(e);
        }
        if Self::c71_at(e, 4, 8, 7, 0xE) {
            e.b.step = 2;
            return;
        }
        self.c71_dest_combo(e);
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x5D00E: at the frame byte `+$2D` (cleared) the fan: n = 0x5D058[`rand & 31`] blades of type 6 along a row of 0x5D078
    /// (`(rand & 7) * 16`); at the end state 0.
    fn c71_fan(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.param_b != 0 {
            e.b.param_b = 0;
            let r = (self.rand() & 0x1F) as u32;
            let n = self.assets.enemy.t.u8(t::FAN_COUNT, r);
            let row = ((self.rand() & 7) as u32) << 4;
            for k in 0..n as u32 {
                let d2 = self.assets.enemy.t.u8(t::FAN_DIRS, row + k);
                self.c19_spawn(e, h, 6, d2);
            }
        }
        if self.enemy_step(&mut e.b) {
            e.b.set_state(0x0200_0000);
        }
    }

    /// 0x5D0F8: the pause after a slash: `+$80` from 0x5D14E by 0x5D12E[`rand & 31`] and the level band; animation 0; then 0x5D124.
    fn c71_pause_start(&mut self, e: &mut Enemy) {
        e.b.step = 0x12;
        let r = (self.rand() & 0x1F) as u32;
        let tb = &self.assets.enemy.t;
        let k = tb.u8(t::WAIT_PICK, r).wrapping_add((self.w.level & 0x18) as u8);
        e.m.sb(T80, tb.u8(t::WAIT_TIME, k as u32));
        self.enemy_anim(&mut e.b, 0);
        let c = e.m.b(T80);
        e.m.sb(T80, c.wrapping_sub(1));
        if c == 0 {
            e.b.step = 2;
        }
    }

    /// 0x81A7A (the starter's file): the hit: the credit (0x3262A); at HP 0 the camera clamp (0x32740) and, once, the starter's bookkeeping
    /// (this body's handle cleared, the count `+$AA` down, the last one kept in `+$A8`); then 0x2A606.
    fn c71_hit(&mut self, e: &mut Enemy, h: Handle) {
        self.boss_credit(e);
        if e.b.hp == 0 {
            self.boss_clamp_camera(&mut e.b);
            if e.b.hp_prev != 0 {
                e.b.hp_prev = 0;
                if let Some(Slot::Prop(k)) = slot_of(e.m.w(o::STARTER)) {
                    let s = &mut self.props.slots[k];
                    for off in [0xA0, 0xA2, 0xA4] {
                        if s.m.w(off) != 0 && s.m.w(off) == h {
                            let n = s.m.b(0xAA).wrapping_sub(1);
                            s.m.sb(0xAA, n);
                            if n == 0 {
                                s.m.sw(0xA8, h);
                            }
                            s.m.sw(off, 0);
                        }
                    }
                }
            }
        }
        self.enemy_hit_reaction(e, h);
    }

    // ------------------------------------------------------------------ helpers

    /// 0x5D92E: face the target (`+$24` 1 when X is below the target's, unsigned).
    fn c71_face(&self, e: &mut Enemy) {
        let tx = self.lib_target(e).xi() as u16;
        e.b.facing = ((e.b.xi() as u16) < tx) as u8;
    }

    /// 0x5D7C4: the walk point by the side `+$7C`: walk modes above 2 an arena point (0x5D818), else 0xB0 left or right of the target at its
    /// depth (0x5D7F8); the turn toward it (0x5D8A4) and face.
    fn c71_dest_by_side(&self, e: &mut Enemy) {
        let side = e.m.b(o::SIDE) as u32;
        let tb = &self.assets.enemy.t;
        if (e.m.b(o::MODE) as i8) > 2 {
            e.m.sw(DEST_X, tb.u16(t::POINTS, side * 4));
            e.m.sw(DEST_Z, tb.u16(t::POINTS, side * 4 + 2));
        } else {
            let tgt = self.lib_target(e);
            let (tx, tz) = (tgt.xi() as u16, tgt.zi() as u16);
            e.m.sw(DEST_X, tb.u16(t::BESIDE, side * 2).wrapping_add(tx));
            e.m.sw(DEST_Z, tz);
        }
        self.c71_turn(e);
        self.c71_face(e);
    }

    /// 0x5D844: the attack point: the combo's distance (0x5D87A by `+$BA`) on the body's side of the target, its depth + 4; turn and face.
    fn c71_dest_combo(&self, e: &mut Enemy) {
        let mut d1 = self.assets.enemy.t.u16(t::COMBO_DX, e.m.w(o::COMBO) as u32 * 2);
        let tgt = self.lib_target(e);
        let (tx, tz) = (tgt.xi() as u16, tgt.zi() as u16);
        if (e.b.xi() as u16) < tx {
            d1 = d1.wrapping_neg();
        }
        e.m.sw(DEST_X, tx.wrapping_add(d1));
        e.m.sw(DEST_Z, tz.wrapping_add(4));
        self.c71_turn(e);
        self.c71_face(e);
    }

    /// 0x5D882: the run's point: its own X at the target's depth; the direction straight to it (no turn limit); face.
    fn c71_dest_zline(&self, e: &mut Enemy) {
        let x = e.b.xi() as u16;
        let tz = self.lib_target(e).zi() as u16;
        e.m.sw(DEST_X, x);
        e.m.sw(DEST_Z, tz);
        e.b.dir = Self::enemy_angle_to(&e.b, x, tz);
        self.c71_face(e);
    }

    /// 0x5D8A4: turn toward the point by at most 3 steps (straight to it within 3).
    fn c71_turn(&self, e: &mut Enemy) {
        let ang = Self::enemy_angle_to(&e.b, e.m.w(DEST_X), e.m.w(DEST_Z));
        let d1 = e.b.dir.wrapping_sub(ang).wrapping_add(3) & 0x1F;
        if d1 <= 6 {
            e.b.dir = ang & 0x1F;
        } else {
            let step: u8 = if d1 >= 0x13 { 3 } else { 0xFD };
            e.b.dir = e.b.dir.wrapping_add(step) & 0x1F;
        }
    }

    /// 0x5D946 / 0x5D974 / 0x5D9A0 / 0x5D9CE: at the point: |X - point + ax| below bx and |Z - point + az| below bz (unsigned windows).
    fn c71_at(e: &Enemy, ax: i16, bx: u16, az: i16, bz: u16) -> bool {
        let dx = e.b.xi().wrapping_sub(e.m.w(DEST_X) as i16).wrapping_add(ax) as u16;
        let dz = e.b.zi().wrapping_sub(e.m.w(DEST_Z) as i16).wrapping_add(az) as u16;
        dx < bx && dz < bz
    }

    /// 0x5DA14: the next walk mode from the row of the previous one (0x5DA3A, rows of 32 by `rand & 31`); the old one in `+$AD`.
    fn c71_pick_mode(&mut self, e: &mut Enemy) {
        let prev = e.m.b(o::MODE);
        e.m.sb(o::PREV, prev);
        let row = ((prev & 6) as u32) << 4;
        let r = (self.rand() & 0x1F) as u32;
        e.m.sb(o::MODE, self.assets.enemy.t.u8(t::WALK_MODE, row + r));
    }

    /// 0x5D606: the side `+$7C` by the walk mode (table 0x5D61A): 0 the side the body is on, 2 the screen half of the target, 4 a random
    /// choice from the row of the old side (0x5D666), 6 unchanged.
    fn c71_side(&mut self, e: &mut Enemy) {
        let tx = self.lib_target(e).xi() as u16;
        match e.m.b(o::MODE) {
            0 => e.m.sb(o::SIDE, ((e.b.xi() as u16) >= tx) as u8),
            2 => {
                let c = (self.cam.s2.cam_x).wrapping_add(0xC0);
                e.m.sb(o::SIDE, (c >= tx) as u8);
            }
            4 => {
                let row = (e.m.b(o::SIDE) as u32) << 5;
                let r = (self.rand() & 0x1F) as u32;
                e.m.sb(o::SIDE, self.assets.enemy.t.u8(t::SIDE_PICK, row + r));
            }
            _ => {}
        }
    }

    /// 0x5DAE8: the attack gate: `rand & 31` below 0x5DB0E[level].
    fn c71_gate(&mut self) -> bool {
        let limit = self.assets.enemy.t.u8(t::GATE, (self.w.level & 0x1F) as u32) as u16;
        (self.rand() & 0x1F) < limit
    }

    /// 0x5DB2E: the combo by the HP phase (below a quarter of `+$BE`, below a half, else) and the distance band (target X - X as an
    /// unsigned word, at most 0x80, / 0x40) from 0x5DB8E, `rand & 31`.
    fn c71_combo_pick(&mut self, e: &mut Enemy) {
        let be = e.m.w(o::MAX_HP);
        let hp = e.b.hp as i16;
        let phase: u32 = if ((be >> 2) as i16) > hp {
            2
        } else if ((be >> 1) as i16) > hp {
            1
        } else {
            0
        };
        let tx = self.lib_target(e).xi() as u16;
        let d = tx.wrapping_sub(e.b.xi() as u16).min(0x80);
        let band = (d >> 6) as u32;
        let r = (self.rand() & 0x1F) as u32;
        let v = self.assets.enemy.t.u8(t::COMBOS, phase * 96 + band * 32 + r);
        e.m.sw(o::COMBO, v as u16);
    }

    /// 0x5DCAE: the target within 0x40 px in X.
    fn c71_near40(&self, e: &Enemy) -> bool {
        let tx = self.lib_target(e).xi();
        (tx.wrapping_sub(e.b.xi()).wrapping_add(0x40) as u16) < 0x80
    }

    /// 0x5DCC4: on the screen (camera X + 0x180 - X below 0x180, unsigned).
    fn c71_on_screen(&self, e: &Enemy) -> bool {
        ((self.cam.s2.cam_x as i16).wrapping_add(0x180).wrapping_sub(e.b.xi()) as u16) < 0x180
    }

    /// 0x5DCDA: the walk speed id by the level band (0x5DCEE).
    fn c71_walk_speed(&self, e: &mut Enemy) {
        let id = self.assets.enemy.t.u16(t::WALK_SPEED, ((self.w.level & 0x1C) >> 1) as u32);
        self.enemy_set_speed(&mut e.b, id);
    }

    /// 0x5DCFE: the next speed of the dash ramp (0x5DD1C, `+$B9` + 2 up to 0x12).
    fn c71_ramp(&self, e: &mut Enemy) {
        let mut d1 = e.m.b(o::RAMP);
        if (d1 as i8) < 0x12 {
            d1 = d1.wrapping_add(2);
        }
        e.m.sb(o::RAMP, d1);
        let id = self.assets.enemy.t.u16(t::RAMP, d1 as u32);
        self.enemy_set_speed(&mut e.b, id);
    }

    /// 0x5D5A4: the present heroes (`+$00` not 0, P1, P2, P3) into `+$A2..+$A6`, the rest cleared; none: P1. Returns their count - 1 (0
    /// when none).
    fn c71_hero_list(&self, e: &mut Enemy) -> u16 {
        let mut n = 0u16;
        let mut at = o::LIST;
        for (k, &p) in PLAYERS.iter().enumerate() {
            if self.players[k].b.live != 0 {
                n += 1;
                e.m.sw(at, p);
                at += 2;
            }
        }
        let d1 = if n == 0 {
            at += 2;
            0
        } else {
            n - 1
        };
        while at <= o::LIST + 4 {
            e.m.sw(at, 0);
            at += 2;
        }
        if e.m.w(o::LIST) == 0 {
            e.m.sw(o::LIST, PLAYERS[0]);
        }
        d1
    }

    /// 0x5D426: the first target: the list, then the index 0x5D46E[count][`rand & 15`] (above 2: 0).
    fn c71_targets_init(&mut self, e: &mut Enemy) {
        for k in 0..3 {
            e.m.sw(o::LIST + 2 * k, 0);
        }
        e.m.sb(o::PICK, 0);
        let r = (self.rand() & 0xF) as u32;
        let d1 = self.c71_hero_list(e) as u32;
        let tb = &self.assets.enemy.t;
        let off = tb.u16(t::FIRST_PICK, d1 * 2) as u32;
        let mut d0 = tb.u8(t::FIRST_PICK, off + r).wrapping_add(e.m.b(o::PICK));
        if (d0 as i8) > 2 {
            d0 = 0;
        }
        e.m.sb(o::PICK, d0);
        e.m.sw(TARGET, e.m.w(o::LIST + 2 * d0 as usize));
    }

    /// 0x5D4A4: the next target (d1 left: the new target, or the count for a held target): a held target (`+$C8` 0x23) moves the pick
    /// along the list (0x5D534); else 0x5D4FE[count][`rand & 15`] + `+$A8` (wrapped), skipping empty entries.
    fn c71_retarget(&mut self, e: &mut Enemy) -> u16 {
        let t = e.m.w(TARGET);
        let held = matches!(slot_of(t), Some(Slot::Player(p)) if self.players[p].action == 0x23);
        if held {
            let d1 = self.c71_hero_list(e);
            match d1 {
                0 => {
                    e.m.sw(TARGET, e.m.w(o::LIST));
                    e.m.sb(o::PICK, 0);
                }
                1 => {
                    let d0 = e.m.b(o::PICK).wrapping_add(1) & 1;
                    e.m.sw(TARGET, e.m.w(o::LIST + 2 * d0 as usize));
                    e.m.sb(o::PICK, d0);
                }
                _ => {
                    let r = self.rand() & 0xF;
                    let mut d2 = e.m.b(o::PICK).wrapping_add(if r & 8 != 0 { 2 } else { 1 });
                    if (d2 as i8) > 2 {
                        d2 = d2.wrapping_sub(3);
                    }
                    e.m.sb(o::PICK, d2);
                    e.m.sw(TARGET, e.m.w(o::LIST + 2 * d2 as usize));
                }
            }
            return d1;
        }
        let r = (self.rand() & 0xF) as u32;
        let d1 = self.c71_hero_list(e);
        let d4 = d1 as i16;
        let tb = &self.assets.enemy.t;
        let off = tb.u16(t::NEXT_PICK, d1 as u32 * 2) as u32;
        let mut d0 = tb.u8(t::NEXT_PICK, off + r).wrapping_add(e.m.b(o::PICK)) as i16;
        if d0 > d4 {
            d0 -= d4 + 1;
        }
        e.m.sb(o::PICK, d0 as u8);
        let mut k = (d0 as u8 as u16) * 2;
        loop {
            if k > 4 {
                k = 0;
            }
            let a = e.m.w(o::LIST + k as usize);
            if a != 0 {
                e.m.sw(TARGET, a);
                return a;
            }
            k += 2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::prop::Prop;
    use dino_assets::AssetPack;

    fn game() -> Game {
        static A: std::sync::OnceLock<AssetPack> = std::sync::OnceLock::new();
        Game::new(A.get_or_init(|| {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dino.zip");
            dino_extract::extract(&path).expect("Slisaur checks require the user's dino.zip")
        }))
    }

    /// D3d 4.3, BT-A6: 450 HP at one player (level 10), 600 at two (level 12), 700 at three (level 14), kept in `+$BE`; 70,000 points.
    #[test]
    fn hp_by_player_count_and_score() {
        for (players, level, hp) in [(0u8, 10u16, 450u16), (1, 12, 600), (2, 14, 700)] {
            let mut g = game();
            g.w.level = level;
            g.w.players_minus_one = players;
            let mut e = Enemy::default();
            e.b.k20 = 71 * 4;
            g.enemy_class71(&mut e, 0, 0);
            assert_eq!((e.b.hp, e.m.w(o::MAX_HP), e.b.sub, e.m.b(ATTACKING)), (hp, hp, 2, 0xFF), "{} players", players + 1);
        }
        let g = game();
        let t = &g.assets.enemy.t;
        assert_eq!(t.u32(0x10292, t.u8(0x325F8, 0x18) as u32), 0x70000);
    }

    /// D3d 4.2: the starter makes two Slisaurs at X 0x580 (Z 0x30, 0x40), a third at Z 0x20 with two or three players.
    #[test]
    fn the_starter_makes_two_or_three() {
        for (players, n) in [(0u8, 2usize), (1, 3), (2, 3)] {
            let mut g = game();
            g.w.players_minus_one = players;
            let i = g.props.alloc().unwrap();
            let h = g.props.handle(i);
            let mut p = Prop::default();
            p.b.live = 1;
            p.b.k20 = 33 * 4;
            g.prop_b33(&mut p, h, i);
            let made: Vec<_> = g.enemies.slots.iter().filter(|e| e.b.live != 0 && e.b.k20 == 71 * 4).collect();
            assert_eq!((made.len(), p.m.b(0xAA) as usize), (n, n));
            assert!(made.iter().all(|e| e.b.xi() == 0x580 && e.m.w(o::STARTER) == h));
        }
    }
}
