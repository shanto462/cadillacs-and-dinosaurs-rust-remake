//! Slice, the boss of Episode 4: pool A class 25 (handler 0x4BE70..0x4D48A). D3c section 6, BS-S1.
//!
//! Mode 0 is the init (sub 0, 0x4BE96) and the intro (sub 2, 0x4BF2E: the camera, the talk box B50 variant 2, a wait, the camera mover B76,
//! the boss music 0x11, the controller). Mode 2 is the fight (0x4BFFE): the decision, the runs (`+$AC` 0 to 6, the run with the afterimages
//! of pool C class 27), the attack scripts (6 byte lists at 0x4C4AE: walk and kicks, the slash and the single boomerang, the running slash,
//! the fans), the hit (0x4C8B0) and held (0x2B35A). Mode 4 is the death (0x4C8E0): the body stays. The moves are in `class25_ai.rs`, the
//! boomerang (class 19) in `class19.rs`, the afterimage in `effect/fx27.rs`. Owner: enemy port.

use super::{off::*, Enemy};
use crate::game::{slot_of, Game, Handle, Slot};

/// Own bytes of Slice (the original offsets).
pub(super) mod o {
    /// `+$80` byte: the intro waits and the pause after a kick.
    pub const T80: usize = 0x80;
    /// `+$A0` word: the re-aim timer of the runs (the byte `+$A0` is cleared alone at a run start).
    pub const AIM_T: usize = 0xA0;
    /// `+$A2`, `+$A4`, `+$A6` words: the heroes in play (0x4CCFC); `+$A8` byte: the index of the target among them.
    pub const HEROES: usize = 0xA2;
    pub const HERO_NO: usize = 0xA8;
    /// `+$A9` byte (cleared by the alive start).
    pub const A9: usize = 0xA9;
    /// `+$AA` word: the re-target timer (600 frames).
    pub const RETARGET_T: usize = 0xAA;
    /// `+$AC` byte: the run kind (0 to 6, 0xFF none); `+$AD` the run kind before.
    pub const RUN: usize = 0xAC;
    pub const RUN_PREV: usize = 0xAD;
    /// `+$AE` word: the time limit of a run or a walk.
    pub const RUN_T: usize = 0xAE;
    /// `+$B0` byte: the step of the jump (run 2); `+$B1` the step of the route (run 6); `+$B2` the legs left of the route.
    pub const JUMP: usize = 0xB0;
    pub const ROUTE_STEP: usize = 0xB1;
    pub const LEGS: usize = 0xB2;
    /// `+$B3` byte: the frames before the attack gate is asked again.
    pub const GATE_T: usize = 0xB3;
    /// `+$B4` long: the route, three point numbers (0x4CEF2).
    pub const ROUTE: usize = 0xB4;
    /// `+$B8` byte: the frames to the next speed step; `+$B9` byte: the speed step (0x4D458).
    pub const ACCEL_T: usize = 0xB8;
    pub const ACCEL: usize = 0xB9;
    /// `+$BA` word: the attack script; `+$BC` word: the position in the script bytes.
    pub const SCRIPT: usize = 0xBA;
    pub const POS: usize = 0xBC;
    /// `+$BE` word: the HP at the start.
    pub const START_HP: usize = 0xBE;
    /// `+$C0` byte: the frames to the next afterimage; `+$C1` byte: the timer of the unused act 6.
    pub const GHOST_T: usize = 0xC0;
    pub const C1: usize = 0xC1;
    /// `+$C2` word: the talk box (B50).
    pub const TALK: usize = 0xC2;
    /// `+$C4` word: the frames to the next controller pulse (G.$502).
    pub const PULSE: usize = 0xC4;
    /// `+$C6` byte: the frames to the next screen test of run 0.
    pub const SCREEN_T: usize = 0xC6;
}

impl Game {
    /// 0x4BE70 (mode table 0x4BE7C): 0 the start (0x4BE84, by `+$05`: 0 the init 0x4BE96, 2 the intro 0x4BF2E), 2 the fight (0x4BFFE),
    /// 4 the death (0x4C8E0), 6 the free (0x4C908).
    pub(super) fn enemy_class25(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                if e.b.sub == 0 {
                    self.c25_init(e);
                } else {
                    self.c25_intro(e, h);
                }
            }
            2 => self.c25_fight(e, h),
            4 => self.c25_dying(e, h),
            _ => self.enemy_free_pool_a(e, i), // 0x4C908: jmp 0x48FC
        }
    }

    /// 0x4BE96: (0x939B6, a debug marker, is not ported) name 5, `+$84` 0x14, the table 0x4C90E, animation 0, the box table 0x103000; the run
    /// speed (0x4D434); HP row 0 (0x31E14) and `+$BE` = HP; poise 11; not counted; `+$63` 2, `+$72` 0x14; the target (0x32032); the floor
    /// (0x3140C); the first target of the hero list (0x4CB7E).
    fn c25_init(&mut self, e: &mut Enemy) {
        e.b.id = 5;
        e.m.sw(W84, 0x14);
        e.b.anim_table = 0x4C90E;
        self.enemy_anim(&mut e.b, 0);
        e.b.box_table = 0x103000;
        e.b.multi_box = false;
        self.c25_run_speed(e); // 0x4D434
        self.enemy_set_boss_hp(&mut e.b, 0); // 0x31E14 row 0
        e.m.sw(o::START_HP, e.b.hp);
        e.b.stun = 0x0B;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.m.sb(ATTACKING, 0xFF);
        e.m.sb(B87, 0);
        e.b.b63 = 2;
        e.b.b72 = 0x14;
        for k in [o::RUN, o::RUN_PREV, o::JUMP, o::ACCEL_T, o::ACCEL] {
            e.m.sb(k, 0);
        }
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        self.enemy_snap_to_floor(e); // 0x3140C
        self.c25_first_target(e); // 0x4CB7E
        e.b.sub = e.b.sub.wrapping_add(2);
    }

    /// 0x4BF2E: the intro by `+$06` (table 0x4BF42), then the draw check (0x49CA).
    fn c25_intro(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => self.c25_wait_camera(e),
            2 => {
                // 0x4BF82: until the talk box is gone; then 40 frames.
                if self.c25_live(e.m.w(o::TALK)) == 0 {
                    e.m.sb(o::T80, 0x28);
                    e.b.act = e.b.act.wrapping_add(2);
                }
            }
            4 => {
                // 0x4BF98: at the end of the wait the camera mover (B76) and animation 9.
                let t = e.m.b(o::T80).wrapping_sub(1);
                e.m.sb(o::T80, t);
                if t == 0 {
                    self.spawn_prop(76, 0, None); // 0x4860, class 0x130
                    self.enemy_anim(&mut e.b, 9);
                    e.b.act = e.b.act.wrapping_add(2);
                    self.c25_intro_anim(e);
                }
            }
            6 => self.c25_intro_anim(e),
            8 => {
                // 0x4BFD8: 60 frames, then G.$4FC cleared and the boss music 0x11.
                let t = e.m.b(o::T80);
                e.m.sb(o::T80, t.wrapping_sub(1));
                if t == 0 {
                    e.b.act = e.b.act.wrapping_add(2);
                    self.spawn.g.b4fc = 0;
                    self.sound_ex(0x11, 8, 0); // 0xA63C: d1 is the mode table word of 0x4BE74 (8); d2 is left from the object before
                }
            }
            _ => {
                // 0x4BFF0: the HUD bar (G.-$70BC), the controller (0x7BEDC), then the fight (0x4C072).
                self.hud.boss = h;
                self.boss_spawn_controller(h);
                self.c25_alive_start(e, h);
            }
        }
        self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
    }

    /// 0x4BF4E: at camera X 0x470 or more (signed) the talk box (pool B class 50 variant 2, `+$C2`) and the sound 0x23; the step moves on
    /// even when pool B is full.
    fn c25_wait_camera(&mut self, e: &mut Enemy) {
        if (self.cam.s2.cam_x as i16) < 0x470 {
            return;
        }
        e.b.act = e.b.act.wrapping_add(2);
        if let Some(t) = self.spawn_prop(50, 2, None) {
            e.m.sw(o::TALK, t);
            self.sound_ex(0x23, 8, 0); // 0xA63C: d1 is the mode table word of 0x4BE74 (8); d2 is left from the object before
        }
    }

    /// 0x4BFC4: animation 9 to its end, then a 60 frame wait.
    fn c25_intro_anim(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            e.b.act = e.b.act.wrapping_add(2);
            e.m.sb(o::T80, 0x3C);
        }
    }

    /// `+$00` of any object (0 for a word that is no object).
    pub(super) fn c25_live(&self, h: Handle) -> u8 {
        if slot_of(h).is_none() {
            return 0;
        }
        self.body(h).live
    }

    /// The byte `+$C8` of any object (a hero's action class).
    pub(super) fn c25_byte_c8(&self, h: Handle) -> u8 {
        match slot_of(h) {
            Some(Slot::Player(p)) => self.players[p].action,
            Some(Slot::Enemy(i)) => self.enemies.slots[i].m.b(0xC8),
            Some(Slot::Prop(i)) => self.props.slots[i].m.b(0xC8),
            Some(Slot::Item(i)) => self.items.slots[i].m.b(0xC8),
            _ => 0,
        }
    }

    /// 0x4BFFE: mode 2. `+$B3` counts down to 0; every 300 frames G.$502 = 1 (the controller's next group); then by `+$05` (the long table
    /// 0x4C028): 0 the fight 0x4C038, 2 the hit 0x4C8B0, 4 nothing, 6 held (0x2B35A).
    fn c25_fight(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(o::GATE_T);
        e.m.sb(o::GATE_T, t.saturating_sub(1));
        let p = e.m.w(o::PULSE);
        e.m.sw(o::PULSE, p.wrapping_sub(1));
        if p == 0 {
            e.m.sw(o::PULSE, 300);
            self.spawn.g.b502 = 1;
        }
        match e.b.sub {
            0 => self.c25_alive(e, h),
            2 => self.c25_hit(e, h),
            4 => {}
            _ => self.enemy_grabbed(e, h),
        }
    }

    /// 0x4C038: the poise refill (11), the act (table 0x4C06A: 0 the wait 0x4C0A6, 2 the decision 0x4C11C, 4 the attack script 0x4C40A,
    /// 6 the unused wait 0x4C3EA), then 0x4C058.
    fn c25_alive(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(POISE_T);
        if t != 0 {
            e.m.sb(POISE_T, t - 1);
            if t == 1 {
                e.b.stun = 0x0B;
            }
        }
        match e.b.act {
            0 => {
                // 0x4C0A6: the wait `+$7A`, then the decision.
                let w = e.m.b(WAIT);
                if w == 0 {
                    self.c25_decide(e);
                } else {
                    e.m.sb(WAIT, w - 1);
                    if w == 1 {
                        self.c25_decide(e);
                    }
                }
            }
            2 => self.c25_decide(e),
            4 => self.c25_script(e, h),
            _ => {
                // 0x4C3EA: `+$C1` frames of the animation, then state 0 (nothing sets act 6: 0x4C3D4 has no caller).
                let t = e.m.b(o::C1);
                e.m.sb(o::C1, t.wrapping_sub(1));
                if t == 0 {
                    e.b.set_state(0x0200_0000);
                } else {
                    self.enemy_step(&mut e.b);
                }
            }
        }
        self.c25_tail(e, h);
    }

    /// 0x4C058: the terrain (0x12CB4), the draw check (0x49CA) and the hurtable list (0x4BEA).
    fn c25_tail(&mut self, e: &mut Enemy, h: Handle) {
        if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
            e.m.sw(OBSTACLE, ob);
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x4C072: the fight starts (the end of the intro and the recover entry of `+$84` 0x14, tables 0x2B56C / 0x4A7FC): state 0x02000000,
    /// animation 0, a wait of 2 when `+$7A` is 0, the re-target in 600 frames, no run kind; then the wake-up roll (0x4C0B6): on success the
    /// script 4 on screen or 5 off screen (0x4C102), else 0x4C058.
    pub(super) fn c25_alive_start(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0200_0000);
        self.enemy_anim(&mut e.b, 0);
        let d0 = self.c25_anim_d0(&e.b);
        if e.m.b(WAIT) == 0 {
            e.m.sb(WAIT, 2);
        }
        e.m.sw(o::RETARGET_T, 600);
        e.m.sb(o::RUN, 0xFF);
        e.m.sb(o::A9, 0);
        if self.c25_wake_roll(e, d0) {
            // 0x4C102
            e.b.set_state(0x0200_0402);
            let s = if self.c25_on_screen(e) { 4 } else { 5 };
            e.m.sw(o::SCRIPT, s);
            return self.c25_script_start(e, h);
        }
        self.c25_tail(e, h);
    }

    /// The register d0 that 0x120E leaves (the frame loader 0x1236): the first word of the frame record without its top bit when the top
    /// bit is set, else the second word when that is not negative, else the first word.
    fn c25_anim_d0(&self, b: &crate::game::body::Body) -> u16 {
        let f = self.assets.sprites.frame(b.frame);
        if f.w0 & 0x8000 != 0 {
            f.w0 & 0x7FFF
        } else if (f.w1 as i16) >= 0 {
            f.w1
        } else {
            f.w0
        }
    }

    /// 0x4C0B6: after a knock down (reaction 4) `rand() & 31` below 3, 5 or 11 (table 0x4C0FC). The original compares the quarter and the
    /// half of `+$BE` with d0, which here is the frame word 0x120E left (not the HP).
    fn c25_wake_roll(&mut self, e: &Enemy, d0: u16) -> bool {
        if e.b.reaction != 4 {
            return false;
        }
        let d1 = e.m.w(o::START_HP);
        let d3 = if (d0 as i16) <= (d1 >> 2) as i16 {
            0
        } else if (d0 as i16) <= (d1 >> 1) as i16 {
            2
        } else {
            4
        };
        let w = self.assets.enemy.t.u16(0x4C0FC, d3) as i16;
        let r = (self.rand() & 0x1F) as i16;
        r < w
    }

    /// 0x4D41E: on screen: camera X + 0x180 - X below 0x180 (unsigned): the carry.
    pub(super) fn c25_on_screen(&self, e: &Enemy) -> bool {
        let d0 = (self.cam.s2.cam_x as i16).wrapping_add(0xC0).wrapping_sub(e.b.xi()).wrapping_add(0xC0) as u16;
        d0 < 0x180
    }

    /// 0x4D434: the run speed by level (speed ids 0x4D448 by G.$5EE & 0x1C).
    pub(super) fn c25_run_speed(&self, e: &mut Enemy) {
        let k = ((self.w.level & 0x1C) >> 1) as u32;
        let id = self.assets.enemy.t.u16(0x4D448, k);
        self.enemy_set_speed(&mut e.b, id);
    }

    /// 0x4C8B0: the hit: the damage credit (0x3262A); at HP 0 the camera clamp (0x32740) and, while `+$6E` is set, the defeat (0xACEE),
    /// `+$6E` = 0 and the music stop (0xA5EA); then the shared reaction 0x2A606.
    fn c25_hit(&mut self, e: &mut Enemy, h: Handle) {
        let fire = e.b.hp == 0 && e.b.hp_prev != 0;
        self.boss_hit(e, h);
        if fire {
            self.hook_a5ea();
        }
        self.enemy_hit_reaction(e, h);
    }

    /// 0x4C8E0: the death (table 0x4C8EE): step 0 the kill score (0x325E6) and the leader flags (0x6148); the body stays and is drawn.
    fn c25_dying(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.sub == 0 {
            let b = e.b.clone();
            self.enemy_award_kill_score(&b);
            e.b.sub = e.b.sub.wrapping_add(2);
            self.enemy_boss_leader(e.b.who);
        }
        self.enemy_draw_check(&mut e.b, h);
    }

    // ------------------------------------------------------------------ the heroes and the target

    /// 0x4CCFC: the heroes in play into `+$A2`, `+$A4`, `+$A6` in order, the rest cleared (with nobody in play `+$A2` is kept); an empty
    /// `+$A2` becomes hero 1. Returns the count - 1 (0 with nobody).
    fn c25_heroes(&mut self, e: &mut Enemy) -> u16 {
        let mut n = 0u16;
        let mut at = o::HEROES;
        for p in 0..3 {
            if self.players[p].b.live != 0 {
                n += 1;
                e.m.sw(at, Game::player_handle(p));
                at += 2;
            }
        }
        let d1 = if n == 0 {
            at += 2;
            0
        } else {
            n - 1
        };
        while at <= o::HEROES + 4 {
            e.m.sw(at, 0);
            at += 2;
        }
        if e.m.w(o::HEROES) == 0 {
            e.m.sw(o::HEROES, Game::player_handle(0));
        }
        d1
    }

    /// 0x4CB7E: the first target: the list cleared, `rand() & 15`, the heroes (0x4CCFC); the index from the table 0x4CBC6 by the hero count,
    /// above 2 it is 0.
    fn c25_first_target(&mut self, e: &mut Enemy) {
        for k in 0..3 {
            e.m.sw(o::HEROES + 2 * k, 0);
        }
        e.m.sb(o::HERO_NO, 0);
        let r = self.rand() & 0xF;
        let d1 = self.c25_heroes(e) as u32;
        let t = &self.assets.enemy.t;
        let row = t.u16(0x4CBC6, 2 * d1);
        let mut d0 = t.u8(0x4CBC6, (r.wrapping_add(row)) as u32).wrapping_add(e.m.b(o::HERO_NO));
        if (d0 as i8) > 2 {
            d0 = 0;
        }
        e.m.sb(o::HERO_NO, d0);
        let w = e.m.w(o::HEROES + 2 * d0 as usize);
        e.m.sw(TARGET, w);
    }

    /// 0x4CBFC: a new target. A target in the action class 0x23 takes the simple rule 0x4CC8C. Else `rand() & 15`, the heroes, the step from
    /// the table 0x4CC56 added to `+$A8` and wrapped by the count; the first filled entry from there (an index past 2 starts at 0).
    /// Returns the register d1 it leaves (the new target, or the hero count - 1 of 0x4CC8C).
    pub(super) fn c25_retarget(&mut self, e: &mut Enemy) -> u16 {
        if self.c25_byte_c8(e.m.w(TARGET)) == 0x23 {
            return self.c25_retarget_simple(e);
        }
        let r = self.rand() & 0xF;
        let d1 = self.c25_heroes(e);
        let mut d4 = d1 as i16;
        let t = &self.assets.enemy.t;
        let row = t.u16(0x4CC56, 2 * d1 as u32);
        let mut d0 = t.u8(0x4CC56, (r.wrapping_add(row)) as u32).wrapping_add(e.m.b(o::HERO_NO)) as i16;
        if d0 > d4 {
            d4 += 1;
            d0 = d0.wrapping_sub(d4);
        }
        e.m.sb(o::HERO_NO, d0 as u8);
        let mut k = (d0 as u8 as i16).wrapping_mul(2);
        loop {
            if k > 4 {
                k = 0;
            }
            let w = e.m.w(o::HEROES + k as usize);
            if w != 0 {
                e.m.sw(TARGET, w);
                return w;
            }
            k += 2;
        }
    }

    /// 0x4CC8C: one hero: that one; two: the other one; three: one or two places on (`rand() & 15` bit 3), wrapped at 3.
    fn c25_retarget_simple(&mut self, e: &mut Enemy) -> u16 {
        let d1 = self.c25_heroes(e);
        match d1 {
            0 => {
                e.m.sw(TARGET, e.m.w(o::HEROES));
                e.m.sb(o::HERO_NO, 0);
            }
            1 => {
                let d0 = e.m.b(o::HERO_NO).wrapping_add(1) & 1;
                e.m.sw(TARGET, e.m.w(o::HEROES + 2 * d0 as usize));
                e.m.sb(o::HERO_NO, d0);
            }
            _ => {
                let r = self.rand() & 0xF;
                let mut d2 = e.m.b(o::HERO_NO);
                d2 = d2.wrapping_add(if r & 8 != 0 { 2 } else { 1 });
                if (d2 as i8) > 2 {
                    d2 = d2.wrapping_sub(3);
                }
                e.m.sb(o::HERO_NO, d2);
                e.m.sw(TARGET, e.m.w(o::HEROES + 2 * (d2 as usize & 0xFF)));
            }
        }
        d1
    }
}
