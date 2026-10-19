//! Tyrog, the boss of Episode 6: pool A class 4 (handler 0x36A64..0x38DDE), name 0x19, table 0x702C4. Three forms by the high byte of `+$26`:
//! 2 green (fight 1, music 0x0F and the reinforcement controller), 1 orange, 0 grey and blue (the last, 60,000 points and the stage end).
//! Between two forms the slot becomes the host (class 72) with the variant one lower. D3d 3.1 to 3.7, BT09 to BT12. The states and actions are
//! in `class4_states.rs`; the shared boss pieces (phase flag, gate, action table, recover entry) in `boss_common.rs`. Owner: enemy port.
//!
//! Not ported: the ROM checksum call 0x5A454 at 1 in 16 form changes (it only acts on a bad ROM).

use super::{off::*, Enemy};
use super::boss_common::{BossAi, BossRecover};
use crate::game::{Game, Handle};

/// Own bytes (the original offsets).
pub(super) mod o {
    /// `+$A0` byte: 1 aggressive, 0 defensive (the phase flag). `+$A1` byte: busy (no claw counter). `+$A2` / `+$A3` bytes: the step delay of
    /// the walk states and its reload flag. `+$A4` byte: an attack is pending. `+$A5` byte: the target's X class. `+$A6` word: the phase timer.
    pub const PHASE: usize = 0xA0;
    pub const BUSY: usize = 0xA1;
    pub const DELAY: usize = 0xA2;
    pub const RELOAD: usize = 0xA3;
    pub const PENDING: usize = 0xA4;
    pub const XCLASS: usize = 0xA5;
    /// `+$A8` byte: a short counter; `+$A9` byte: the direction before a turn (0x37B86); `+$AA` byte: the lava rocks left; `+$AB` byte: the
    /// sidesteps in a row; `+$AC` word: the state timer.
    pub const A8: usize = 0xA8;
    pub const OLD_DIR: usize = 0xA9;
    pub const ROCKS: usize = 0xAA;
    pub const STEPS: usize = 0xAB;
    pub const TIMER: usize = 0xAC;
    /// `+$B0` / `+$B4` longs: the X / Z speeds of the walk states (16.16). `+$B9` byte. `+$BC` byte.
    pub const VX: usize = 0xB0;
    pub const VZ: usize = 0xB4;
    pub const B9: usize = 0xB9;
    pub const BC: usize = 0xBC;
    /// `+$C2` byte: the pounce passed its top; `+$C4` / `+$C8` longs: a quarter of the pounce's X / Z distance. `+$CC` word: the repeats left.
    pub const APEX: usize = 0xC2;
    pub const QX: usize = 0xC4;
    pub const QZ: usize = 0xC8;
    pub const REPEAT: usize = 0xCC;
    /// `+$80` word: the time the body lies after a form's defeat.
    pub const LIE: usize = 0x80;
}

/// The tables of the handler and of its data block (read at run time through `enemy.t`, dino-extract/src/boss.rs).
pub(super) mod t {
    /// 0x329B8's windows: X offsets and limits (8 words each), Z offsets and limits (4 words each).
    pub const XOFF: u32 = 0x32A28;
    pub const XLIM: u32 = 0x32A38;
    pub const ZOFF: u32 = 0x32A48;
    pub const ZLIM: u32 = 0x32A50;
}

impl Game {
    /// 0x36A64 (mode table 0x36A72): 0 the init 0x36A7A, 2 the fight 0x36B9A, 4 the defeat 0x38D0E, 6 the body 0x38DD8 (drawn only).
    pub(super) fn enemy_class4(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => self.c4_init(e, h),
            2 => self.c4_fight(e, h, i),
            4 => self.c4_dying(e, h),
            _ => self.enemy_draw_check(&mut e.b, h), // 0x38DD8: jmp 0x49CA
        }
    }

    /// 0x36A7A: name 0x19, `+$84` 0x64, the table 0x702C4, `+$63` 2, animation 1, the box table 0x103C00, speed id 7, direction 0x18; HP index 6
    /// of 0x31E14 halved (`asr.w`) into `+$6C` / `+$6E`; the target; poise 11; `+$72` 0x1E; `+$AC` 0x14; aggressive with the phase timer 1;
    /// the palette and `+$7D` by form (var 2 also the music 0x0F); the HUD bar; G.$4FC = 0; the floor (0x3140C); a slot (0x32C5E); var 2 the
    /// controller (0x7BEDC); then the recover entry 0x36E7C.
    fn c4_init(&mut self, e: &mut Enemy, h: Handle) {
        e.b.live = 1;
        e.m.sb(B87, 0);
        e.b.id = 0x19;
        e.m.sw(W84, 0x64);
        e.b.anim_table = 0x702C4;
        e.b.b63 = 2;
        self.enemy_anim(&mut e.b, 1);
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_set_speed(&mut e.b, 7);
        e.b.dir = 0x18;
        self.enemy_set_boss_hp(&mut e.b, 6); // 0x31E14 row 6
        let hp = ((e.b.hp as i16) >> 1) as u16;
        e.b.hp = hp;
        e.b.hp_prev = hp;
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        e.b.stun = 0x0B;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.b.b72 = 0x1E;
        e.m.sw(o::TIMER, 0x14);
        for k in [o::BC, o::B9, o::STEPS, o::PENDING] {
            e.m.sb(k, 0);
        }
        e.m.sb(WAIT, 0);
        e.m.sw(0xA6, 1);
        e.m.sb(o::PHASE, 1);
        e.m.sb(ATTACKING, 0xFF);
        let form = (e.b.var26 >> 8) as u8;
        e.b.palette_override = 0x9F;
        if form == 1 {
            e.b.palette_override = 0x9E;
            e.m.sb(ATTACKING, 0xFE);
        }
        if form == 2 {
            e.b.palette_override = 0x9D;
            e.m.sb(ATTACKING, 0xFE);
            self.sound(0x0F); // 0xA63C, d1 / d2 left from the caller
        }
        self.hud.boss = h; // G.-$70BC
        self.spawn.g.b4fc = 0;
        self.enemy_snap_to_floor(e); // 0x3140C
        self.lib_slot6_new(e); // 0x32C5E
        if form == 2 {
            self.boss_spawn_controller(h); // 0x7BEDC
        }
        self.c4_recover(e, h);
    }

    /// 0x36E7C: the recover entry (also of `+$84` 0x64 in the table 0x2B56C): the shared part (`boss_recover`), then the wake-up claw
    /// (0x382B0), the off screen pounce (0x3862A) or state 0x4C.
    pub(super) fn c4_recover(&mut self, e: &mut Enemy, h: Handle) {
        match self.boss_recover(e, Self::c4_ai(e)) {
            BossRecover::WakeAttack => self.c4_wake_claw(e),
            // Tyrog 2 (0x668F6): the breath's start with animation 0x1F (0x37EDA in class 4, which nothing there reaches).
            BossRecover::OffscreenHop if Self::c4_is_t2(e) => self.c4_breath_start(e, h, 0x1F),
            BossRecover::OffscreenHop => self.c4_pounce_crouch(e, 0x26),
            _ => {}
        }
    }

    /// Tyrog 2 (class 91, 0x654E2) runs this code with the differences of `class91.rs`.
    pub(super) fn c4_is_t2(e: &Enemy) -> bool {
        e.b.k20 == 91 * 4
    }

    /// The table set of `boss_common.rs` (0x36D36 / 0x6575C ...).
    pub(super) fn c4_ai(e: &Enemy) -> BossAi {
        if Self::c4_is_t2(e) { BossAi::Tyrog2 } else { BossAi::Tyrog }
    }

    /// The palette and `+$7D` by form (0x36B9A, 0x655BA).
    pub(super) fn c4_fight_palette(e: &mut Enemy) {
        let form = (e.b.var26 >> 8) as u8;
        e.b.palette_override = 0x9F;
        if form == 1 {
            e.b.palette_override = 0x9E;
            e.m.sb(ATTACKING, 0xFE);
        }
        if form == 2 {
            e.b.palette_override = 0x9D;
            e.m.sb(ATTACKING, 0xFE);
        }
    }

    /// 0x36B9A: mode 2. The palette and `+$7D` by form; with HP 0 X is kept in camera X - 0x10 .. + 0x190; by `+$05` (the long table 0x36C24):
    /// 0 the AI 0x36C34, 2 the hit 0x38CC6, 4 the defeat 0x38D0E, 6 held (0x2B35A); then the play field clamp (0x193E).
    fn c4_fight(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        Self::c4_fight_palette(e);
        self.c4_dead_clamp(e);
        match e.b.sub {
            0 => self.c4_alive(e, h),
            2 => self.c4_hit(e, h),
            4 => self.c4_dying(e, h),
            _ => self.enemy_grabbed(e, h),
        }
        self.enemy_clamp_to_playfield(e, i); // jmp 0x193E
    }

    /// 0x36BCC: with HP 0 X is kept in camera X - 0x10 .. + 0x190.
    pub(super) fn c4_dead_clamp(&self, e: &mut Enemy) {
        if e.b.hp == 0 {
            let cam = self.cam.s2.cam_x as i16;
            let put = |b: &mut crate::game::body::Body, x: i16| b.x = (b.x & 0xFFFF) | ((x as i32) << 16);
            if e.b.xi().wrapping_sub(cam.wrapping_sub(0x10)) < 0 {
                put(&mut e.b, cam.wrapping_sub(0x10));
            }
            if cam.wrapping_add(0x190).wrapping_sub(e.b.xi()) < 0 {
                put(&mut e.b, cam.wrapping_add(0x190));
            }
        }
    }

    /// 0x36C34: the poise refill (11), the terrain (0x12CB4); the hit stop (the animation steps when it ends, `+$2F` = 1) or the wait `+$7A`;
    /// else a slot point (0x32C54), the state of `+$06` (table 0x36CA0), the gate (0x36DF0) and the phase flag (0x36CF4); then the target
    /// check (0x32F56), the draw check and the hurtable list.
    pub(super) fn c4_alive(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(POISE_T);
        if t != 0 {
            e.m.sb(POISE_T, t - 1);
            if t == 1 {
                e.b.stun = 0x0B;
            }
        }
        if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
            e.m.sw(OBSTACLE, ob);
        }
        if e.b.hitstop != 0 {
            e.b.hitstop -= 1;
            e.b.timer = 1;
        } else if e.m.b(WAIT) != 0 {
            e.m.sb(WAIT, e.m.b(WAIT) - 1);
        } else {
            self.lib_slot6_keep(e); // 0x32C54
            self.c4_state(e, h);
            // 0x36DF0: in the aggressive phase G.$502 = 1 and the gate 0x38BD6.
            if e.m.b(o::PHASE) != 0 {
                self.spawn.g.b502 = 1;
                self.boss_aggression_gate(e, Self::c4_ai(e));
            }
            self.boss_phase_tick(e, Self::c4_ai(e)); // 0x36CF4 (0x6571A)
        }
        self.lib_check_target(e); // 0x32F56
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x38CC6: the hit: the damage credit (0x3262A); at HP 0 (or below): forms 2 and 1 slow the game (G.$4D6 = 3) and kill the helpers
    /// (G.$50E = 2); the camera clamp (0x32740); form 0 the stage end once (0xACEE, `+$6E` = 0); then the shared reaction 0x2A606.
    fn c4_hit(&mut self, e: &mut Enemy, h: Handle) {
        let form = (e.b.var26 >> 8) as u8;
        // The credit and the clamp of `boss_hit`; its defeat call is only for form 0.
        if form == 0 {
            self.boss_hit(e, h);
        } else {
            self.boss_credit(e); // 0x3262A
            if (e.b.hp as i16) <= 0 {
                self.flow.speed = 3; // G.$4D6
                self.spawn.g.kill_all = 2; // G.$50E
                self.boss_clamp_camera(&mut e.b); // 0x32740
            }
        }
        self.enemy_hit_reaction(e, h);
    }

    /// 0x38D0E: the defeat by `+$05` (table 0x38D1A). Forms 2 and 1 (0x38D20): normal speed (G.$4D6 = 1), the helpers live again, 120 frames
    /// lying (0x38D8C), then the slot becomes the host (class 72, `+$04..+$07` = 0, the variant one lower). Form 0: the music fade (0xA5EA), normal
    /// speed, the score (0x325E6), 120 frames, the leader flags (0x6148), mode 6.
    fn c4_dying(&mut self, e: &mut Enemy, h: Handle) {
        let form = (e.b.var26 >> 8) as u8;
        if e.b.sub == 0 {
            if form != 0 {
                self.flow.speed = 1;
                self.spawn.g.kill_all = 0;
                e.m.sw(o::LIE, 0x78);
                e.b.sub = 4;
            } else {
                self.hook_a5ea();
                self.flow.speed = 1;
                e.m.sw(o::LIE, 0x78);
                let b = e.b.clone();
                self.enemy_award_kill_score(&b); // 0x325E6
                e.b.sub = e.b.sub.wrapping_add(2);
            }
        }
        let t = e.m.w(o::LIE);
        if t == 0 {
            if e.b.sub == 4 {
                // 0x38D9E: the host.
                e.b.set_state(0);
                e.b.live = 1;
                e.b.k20 = 72 * 4;
                e.b.var26 = e.b.var26.wrapping_sub(0x100);
                if self.rand() & 0xF == 0 {
                    // 0x5A454: the ROM checksum test, not ported (a good ROM makes it do nothing).
                }
                return;
            }
            if form == 0 {
                self.enemy_boss_leader(e.b.who); // 0x6148
            }
            e.b.mode = 6;
            return self.enemy_draw_check(&mut e.b, h);
        }
        e.m.sw(o::LIE, t - 1);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x329B8: the target's X class (0..7) and Z class (0..3) by the windows of 0x32A28 / 0x32A48, and what it leaves in d2 and d3 (the last
    /// window's distance and the last index * 2). The Z windows compare the target's Z with this object's X, as the original does.
    pub(super) fn c4_classes(&self, e: &Enemy) -> (u16, u16, u16, u16) {
        let tgt = self.lib_target(e);
        let (tx, tz) = (tgt.xi(), tgt.zi());
        let x = e.b.xi();
        let tb = &self.assets.enemy.t;
        let window = |v: i16, off: u32, lim: u32, n: u16| -> (u16, u16, u16) {
            let mut d1 = n;
            loop {
                let d3 = d1 * 2;
                let mut d4 = tb.i16(off, d3 as u32);
                if v < x {
                    d4 = d4.wrapping_neg();
                }
                let mut d2 = v.wrapping_add(d4).wrapping_sub(x);
                if d2 < 0 {
                    d2 = d2.wrapping_neg();
                }
                if d2 as u16 > tb.u16(lim, d3 as u32) {
                    return (d1, d2 as u16, d3);
                }
                if d1 == 0 {
                    return (0, d2 as u16, d3);
                }
                d1 -= 1;
            }
        };
        let (xc, _, _) = window(tx, t::XOFF, t::XLIM, 7);
        let (zc, d2, d3) = window(tz, t::ZOFF, t::ZLIM, 3);
        (xc, zc, d2, d3)
    }
}
