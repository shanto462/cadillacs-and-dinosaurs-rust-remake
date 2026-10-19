//! Dr. Fessenden, form 2: pool A class 84 (handler 0x6173E..0x63778), name 0x22, the big pink two-headed dinosaur. Made by form 1 (class 94) at
//! its death: in its first frame it takes five more pool A slots, sorts the six addresses and keeps the lowest as the body; the others become the
//! parts (85 the body layer that draws it, 86 the arm, 87 the second limb, 88 / 89 the left / right head, `class85_89.rs`). The body itself is
//! never drawn (it takes its own entry off the draw list); the parts copy its HP and a hit on a part is a hit on the body (0x62F12). It walks,
//! chases, dashes and steps, and every 16 frames a gate by level (0x62CFE) lets the picker (0x62C62, by 48 px classes) choose a bite, a fire
//! volley, one of three arm strikes or a grab; heroes who jump or dash in are countered (0x6314E). HP row 9 (640 at level 10), 1,000 points,
//! 0xACEE at HP 0 and a long death that hides the body and sets `+$A6` (the Episode 8 ending waits for it). D3d 6.3, 6.4, 7, BT16 to BT20.
//! Owner: enemy port.
//!
//! Not ported: the test-mode HUD of the actions (0x63328, with G.$4AA and the service switch) and the debug marker 0x939B6.

use super::{off::*, Enemy};
use crate::game::{slot_of, Game, Handle, Slot};

/// Own bytes and words (the original offsets).
pub(super) mod o {
    /// `+$A2` arm request (2, 4, 6 the strikes, 8 the grab pose), `+$A3` limb request (8), `+$A4` / `+$A5` left / right head request (2 bite,
    /// 4 fire, 6 sway), `+$A6` the hidden flag of the dead body, `+$A8` the action * 2.
    pub const ARM: usize = 0xA2;
    pub const LIMB: usize = 0xA3;
    pub const HEAD_L: usize = 0xA4;
    pub const HEAD_R: usize = 0xA5;
    pub const HIDDEN: usize = 0xA6;
    pub const ACTION: usize = 0xA8;
    /// `+$AC` the heads that can act (bit 1 left, bit 0 right), `+$AD` the head of a bite, `+$AE` the repeats (bites, volleys, chews).
    pub const HEADS: usize = 0xAC;
    pub const BITE_HEAD: usize = 0xAD;
    pub const REPEATS: usize = 0xAE;
    /// `+$B0` arm, `+$B2` limb, `+$B4` left head, `+$B6` right head (handles). `+$B8` word: the idle time. `+$BA`, `+$BB` bytes of the walk.
    pub const P_ARM: usize = 0xB0;
    pub const P_LIMB: usize = 0xB2;
    pub const P_HEAD_L: usize = 0xB4;
    pub const P_HEAD_R: usize = 0xB6;
    pub const IDLE: usize = 0xB8;
    pub const BA: usize = 0xBA;
    pub const BB: usize = 0xBB;
    /// `+$BC` the close test timer, `+$BD` the guard counter, `+$BE` the facing timer, `+$BF` the walk animation runs backwards.
    pub const CLOSE_T: usize = 0xBC;
    pub const GUARD: usize = 0xBD;
    pub const FACE_T: usize = 0xBE;
    pub const BACK: usize = 0xBF;
    /// `+$C0` byte the gate timer, `+$C2` word the start HP, `+$C4` byte the time out of reach (`+$00` 2), `+$C6` word the 480 frame timer,
    /// `+$C8` byte the counters are off, `+$C9` byte a counter happened.
    pub const GATE_T: usize = 0xC0;
    pub const START_HP: usize = 0xC2;
    pub const AWAY: usize = 0xC4;
    pub const LIMIT: usize = 0xC6;
    pub const BUSY: usize = 0xC8;
    pub const COUNTERED: usize = 0xC9;
    /// `+$80` byte or word timer, `+$81` byte the volleys fired, `+$92` word a timer or an end coordinate.
    pub const T81: usize = 0x81;
    pub const T92: usize = 0x92;
}

/// The tables of the body (read at run time through `enemy.t`; dino-extract/src/boss.rs).
mod t {
    pub const WALK_BANDS: u32 = 0x64AFA;
    pub const WALK_POINT: u32 = 0x64B06;
    pub const WALK_PICK: u32 = 0x64B46;
    pub const WINDOWS: u32 = 0x64D46;
    pub const IDLE_TIME: u32 = 0x64D8E;
    pub const WALK_TIME: u32 = 0x64DCE;
    pub const CLOSE_BOX: u32 = 0x64DF0;
    pub const FOLLOW: u32 = 0x64DFC;
    pub const GATE: u32 = 0x64E00;
    pub const PICKER: u32 = 0x64E20;
    pub const BITE_AIM: u32 = 0x64F00;
    pub const BITE_COUNT: u32 = 0x64F04;
    pub const FIRE_AIM: u32 = 0x64F24;
    pub const FIRE_COUNT: u32 = 0x64F28;
    pub const ARM_AIM: [u32; 3] = [0x64F48, 0x64F4C, 0x64F50];
    pub const GRAB_A: u32 = 0x64F54;
    pub const GRAB_B: u32 = 0x64F58;
    pub const KO_MASK: u32 = 0x64FA2;
    pub const KO_PICK: u32 = 0x64FB2;
    pub const KO_BANDS: u32 = 0x64FBE;
    pub const AWAY_TIME: u32 = 0x64FC4;
    pub const COUNTER_BOX: u32 = 0x65044;
    pub const COUNTER_MASK: u32 = 0x6504A;
    pub const COUNTER_PICK: u32 = 0x6505A;
    pub const CHEWS: u32 = 0x650DA;
    pub const GUARD_COST: u32 = 0x6350C;
}

/// The player bodies' handles (G.$3274, $33F4, $3574).
const P1: Handle = 0xB274;
const P2: Handle = 0xB3F4;
const P3: Handle = 0xB574;

impl Game {
    /// 0x6173E (mode table 0x6174A): 0 the creation 0x61752 (then the init), 2 the frame 0x61960, 4 the death 0x63538, 6 the free 0x63772.
    pub(super) fn enemy_class84(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 if e.b.sub == 0 => self.c84_create(e, h, i),
            0 => self.c84_init(e, h),
            2 => self.c84_frame(e, h),
            4 => self.c84_dying(e, h),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// The handle in the word `off` of the body as a pool A index.
    fn c84_slot(e: &Enemy, off: usize) -> Option<usize> {
        match slot_of(e.m.w(off)) {
            Some(Slot::Enemy(k)) => Some(k),
            _ => None,
        }
    }

    /// A byte of the part in `off` (0 when there is none).
    fn c84_part_b(&self, e: &Enemy, off: usize, field: usize) -> u8 {
        Self::c84_slot(e, off).map(|k| self.enemies.slots[k].m.b(field)).unwrap_or(0)
    }

    /// 0x6175A: with at least 5 free pool A slots: five more objects; the six addresses (this object's too) sorted up (five passes of
    /// compare and exchange); the lowest is the body (class 84, sub 2, at this place, with the parts in `+$B0` / `+$B2` / `+$B4` / `+$B6`), the
    /// others the parts 86, 88 (`+$5C` 0), 85, 89 (`+$5C` -1), 87 in that order, each with `+$B0` = the body and `+$63` 4.
    fn c84_create(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if self.enemies.free_count() < 5 {
            return;
        }
        let mut d = [0u16; 6];
        for k in 0..5 {
            let Some(j) = self.enemies.alloc() else { return };
            d[k] = self.enemies.handle(j);
        }
        d[5] = h;
        for _ in 0..5 {
            for k in 0..5 {
                if d[k + 1] < d[k] {
                    d.swap(k, k + 1);
                }
            }
        }
        let (x, y, z) = (e.b.xi(), e.b.yi(), e.b.zi());
        // The object being run is one of the six: it is written through `e`, the others through the pool.
        let write = |g: &mut Game, e: &mut Enemy, hd: Handle, f: &mut dyn FnMut(&mut Enemy)| {
            if hd == h {
                f(e);
            } else if let Some(Slot::Enemy(k)) = slot_of(hd) {
                f(&mut g.enemies.slots[k]);
            }
        };
        let body = d[0];
        write(self, e, body, &mut |n: &mut Enemy| {
            n.b.live = 1;
            n.b.k20 = 84 * 4;
            let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
            put(&mut n.b.x, x);
            put(&mut n.b.y, y);
            put(&mut n.b.z, z);
            n.b.sub = n.b.sub.wrapping_add(2);
            n.m.sw(o::P_ARM, d[1]);
            n.m.sw(o::P_HEAD_L, d[2]);
            n.m.sw(o::P_HEAD_R, d[4]);
            n.m.sw(o::P_LIMB, d[5]);
            n.b.b63 = 4;
        });
        for (hd, class, w5c) in [(d[1], 86u16, None), (d[2], 88, Some(0u16)), (d[3], 85, None), (d[4], 89, Some(0xFFFF)), (d[5], 87, None)] {
            write(self, e, hd, &mut |n: &mut Enemy| {
                n.b.live = 1;
                n.b.k20 = class * 4;
                n.m.sw(0xB0, body);
                if let Some(v) = w5c {
                    n.m.sw(W5C, v);
                }
                n.b.b63 = 4;
            });
        }
        let _ = i;
    }

    /// 0x6187A: G.$7B4 = 2, the music 0x12, name 0x22, `+$84` 0x88, the table 0x64452, HP row 9 (kept in `+$C2`), the HUD bar, the box table
    /// 0x102400, poise 10, guard 2, `+$7D` 0xFF, `+$72` 0xC, the timers, animation 0, the target, the floor, the controller (0x7BEDC); visible,
    /// mode 2.
    fn c84_init(&mut self, e: &mut Enemy, h: Handle) {
        self.cam.s2.lock78 = 2; // G.$7B4
        self.sound(0x12);
        e.b.id = 0x22;
        e.m.sw(W84, 0x88);
        e.b.anim_table = 0x64452;
        self.enemy_set_boss_hp(&mut e.b, 9); // 0x31E14 row 9
        e.m.sw(o::START_HP, e.b.hp);
        self.hud.boss = h; // G.-$70BC
        e.b.box_table = 0x102400;
        e.b.multi_box = false;
        e.b.stun = 0xA;
        e.m.sb(o::GUARD, 2);
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.m.sb(B87, 0);
        e.b.hitstop = 0;
        e.m.sb(ATTACKING, 0xFF);
        e.m.sb(o::FACE_T, 0);
        e.b.b72 = 0xC;
        e.m.sb(o::ACTION, 0);
        e.m.sw(o::IDLE, 0);
        e.m.sb(o::BA, 0);
        e.m.sb(o::BB, 0);
        e.m.sb(o::CLOSE_T, 1);
        e.m.sb(o::BACK, 0);
        e.m.sb(o::GATE_T, 0xF);
        e.m.sl(0xA0, 0);
        e.m.sw(0xA4, 0);
        e.m.sb(o::HEADS, 3);
        e.m.sb(o::AWAY, 0);
        e.m.sw(o::LIMIT, 0);
        e.m.sb(o::BUSY, 0);
        e.m.sb(o::COUNTERED, 0);
        e.m.sb(o::HIDDEN, 0);
        self.enemy_anim(&mut e.b, 0);
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        self.enemy_snap_to_floor(e); // 0x3140C
        self.boss_spawn_controller(h); // 0x7BEDC
        e.b.visible = 1;
        e.b.mode = e.b.mode.wrapping_add(2);
        e.b.sub = 0;
    }

    /// 0x61960: the poise refill (the guard counter back to 2), the time out of reach (`+$C4`, then `+$00` 1); the heads' readiness (0x62A04)
    /// and the hit on a part (0x62F12); by `+$05` (the long table 0x619E6): 0 the fight 0x619F6, 2 the hit 0x63460, 4 the grab 0x6276E, 6 held
    /// (0x6352A); then, outside the grab, the body takes its own entry off the draw list (the body layer, class 85, draws it).
    fn c84_frame(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(POISE_T);
        if t != 0 {
            e.m.sb(POISE_T, t - 1);
            if t == 1 {
                e.m.sb(o::GUARD, 2);
            }
        }
        let a = e.m.b(o::AWAY);
        if a != 0 {
            e.m.sb(o::AWAY, a - 1);
            if a == 1 {
                e.b.live = 1;
            }
        }
        self.c84_heads_ready(e);
        self.c84_part_hit(e);
        match e.b.sub {
            0 => self.c84_fight(e, h),
            2 => self.c84_hit(e, h),
            4 => self.c84_grab(e, h),
            _ => {
                // 0x6352A
                e.m.sl(0xA0, 0);
                e.m.sw(0xA4, 0);
                self.enemy_grabbed(e, h);
            }
        }
        if !(e.b.mode == 2 && e.b.sub == 4) && self.draw_list.last().map(|v| *v as u16 == h).unwrap_or(false) {
            self.draw_list.pop();
        }
    }

    /// 0x62A04: `+$AC` 3, bit 1 off when the left head's `+$A8` word is negative, bit 0 off for the right head.
    fn c84_heads_ready(&self, e: &mut Enemy) {
        let mut d0 = 3u8;
        if let Some(k) = Self::c84_slot(e, o::P_HEAD_L) {
            if (self.enemies.slots[k].m.w(0xA8) as i16) < 0 {
                d0 = 1;
            }
        }
        if let Some(k) = Self::c84_slot(e, o::P_HEAD_R) {
            if (self.enemies.slots[k].m.w(0xA8) as i16) < 0 {
                d0 &= !1;
            }
        }
        e.m.sb(o::HEADS, d0);
    }

    /// 0x62F12: while the body is live (1) and not already hit: a part in the hit state (0x0202) gives its hit to the body: live 2, the hit id
    /// and the attacker, state 0x02020000, the damage (the part's HP before minus now) off the body's HP, the reaction, the hit stop (or 10);
    /// the action is dropped.
    fn c84_part_hit(&mut self, e: &mut Enemy) {
        if e.b.live != 1 || ((e.b.mode as u16) << 8 | e.b.sub as u16) == 0x0202 {
            return;
        }
        let mut hit = None;
        for off in [o::P_ARM, o::P_LIMB, o::P_HEAD_L, o::P_HEAD_R] {
            if let Some(k) = Self::c84_slot(e, off) {
                let p = &self.enemies.slots[k];
                if ((p.b.mode as u16) << 8 | p.b.sub as u16) == 0x0202 {
                    hit = Some(k);
                    break;
                }
            }
        }
        let Some(k) = hit else { return };
        let p = &self.enemies.slots[k];
        let (serial, who, w68, dmg, act, hs) = (
            p.b.serial,
            p.b.who,
            p.b.attacker,
            { let d = p.b.hp_prev.wrapping_sub(p.b.hp) as i16; if d < 0 { 0 } else { d as u16 } },
            p.b.act,
            p.b.hitstop,
        );
        e.b.live = 2;
        e.b.serial = serial;
        e.b.who = who;
        e.b.set_state(0x0202_0000);
        e.m.sb(WAIT, 0);
        e.b.attacker = w68;
        e.b.hp_prev = e.b.hp;
        e.b.hp = e.b.hp.wrapping_sub(dmg);
        e.b.act = act;
        e.b.reaction = act;
        e.b.hitstop = if hs != 0 { hs } else { 0xA };
        e.m.sb(o::ACTION, 0);
        e.m.sl(0xA0, 0);
        e.m.sw(0xA4, 0);
    }

    /// 0x619F6: the fight: every 31 frames face the target; the counter clean-up (0x62996); the action of `+$A8` (table 0x61A52); the counter
    /// test (0x6314E, its reaction 0x6326E); the floor (0x12E20); the screen clamp (0x632EA); the hurtable list.
    fn c84_fight(&mut self, e: &mut Enemy, h: Handle) {
        let f = e.m.b(o::FACE_T);
        if f != 0 {
            e.m.sb(o::FACE_T, f - 1);
        } else {
            let tx = self.lib_target(e).xi();
            e.b.facing = (tx >= e.b.xi()) as u8;
            e.m.sb(o::FACE_T, 0x1E);
        }
        self.c84_counter_clear(e);
        match e.m.b(o::ACTION) {
            0x00 => self.c84_action0(e),
            0x02 => self.c84_walk(e),
            0x04 => self.c84_follow(e),
            0x06 => self.c84_bite(e),
            0x08 => self.c84_fire(e),
            0x0A => self.c84_arm(e, 0),
            0x0C => self.c84_arm(e, 1),
            0x0E => self.c84_arm(e, 2),
            0x10 => self.c84_back_off(e),
            0x12 => self.c84_dash(e),
            0x14 => self.c84_step(e),
            0x16 => self.c84_grab_walk(e, false),
            _ => self.c84_grab_walk(e, true),
        }
        if let Some(p) = self.c84_counter_test(e) {
            self.c84_counter(e, p);
        }
        self.terrain_collide_free(&mut e.b); // 0x12E20
        self.c84_screen_clamp(e);
        self.enemy_push_hurtable(h);
    }

    // ------------------------------------------------------------------ the action set-ups and resets

    /// 0x61A6C: the recover entry of `+$84` 0x88 (table 0x2B56C), also the end of a grab: state 0x02000000, visible, idle 0x1E, animation 0,
    /// the requests and timers cleared, poise refill 0xF0; after a knock down the time out of reach (0x63102) and the knock down answer
    /// (0x62FCC); else the reset (0x61ABA).
    pub(super) fn c84_recover(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0000);
        e.b.visible = 1;
        e.m.sw(o::IDLE, 0x1E);
        self.enemy_anim(&mut e.b, 0);
        e.m.sl(0xA0, 0);
        e.m.sw(0xA4, 0);
        e.m.sw(o::LIMIT, 0);
        e.m.sb(o::BUSY, 0);
        e.m.sb(o::COUNTERED, 0);
        e.m.sb(POISE_T, 0xF0);
        if e.b.reaction == 4 {
            self.c84_away(e);
            if self.c84_knockdown_answer(e) {
                return;
            }
        }
        Self::c84_reset(e);
    }

    /// 0x61ABA: action 0, the gate timer, the counter flags, `+$06` / `+$07` and the requests cleared.
    fn c84_reset(e: &mut Enemy) {
        e.m.sb(o::ACTION, 0);
        e.m.sb(o::GATE_T, 0);
        e.m.sb(o::BUSY, 0);
        e.m.sb(o::COUNTERED, 0);
        e.b.act = 0;
        e.b.step = 0;
        e.m.sl(0xA0, 0);
        e.m.sw(0xA4, 0);
    }

    /// 0x61ADA: a new target (0x32032) and the reset.
    fn c84_retarget(&mut self, e: &mut Enemy) {
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        Self::c84_reset(e);
    }

    /// 0x61B18: the walk (action 1) with the 480 frame limit.
    fn c84_walk_start(e: &mut Enemy) {
        e.m.sl(0xA0, 0);
        e.m.sw(0xA4, 0);
        e.m.sw(o::IDLE, 0);
        e.m.sb(o::ACTION, 2);
        e.b.act = 0;
        e.b.step = 0;
        e.m.sw(o::LIMIT, 0x1E0);
        e.m.sb(o::BUSY, 0);
        e.m.sb(o::COUNTERED, 0);
    }

    /// 0x61CFE: the follow (action 2): `+$92` 0x3C frames, the 480 frame limit.
    fn c84_follow_start(e: &mut Enemy) {
        e.m.sb(o::ACTION, 4);
        e.b.act = 0;
        e.b.step = 0;
        e.m.sw(o::T92, 0x3C);
        e.m.sw(o::LIMIT, 0x1E0);
        e.m.sb(o::BUSY, 0);
        e.m.sb(o::COUNTERED, 0);
        e.m.sl(0xA0, 0);
        e.m.sw(0xA4, 0);
    }

    /// 0x61DF4 (bite, action 3), 0x61F2E (fire, 4), 0x620D4 / 0x6218E / 0x62248 (the arm strikes 5, 6, 7, counters off), 0x62302 (back off,
    /// 8), 0x623C6 (dash, 9), 0x62480 (step, 10): the action, `+$06` / `+$07` 0.
    fn c84_set(e: &mut Enemy, action: u8, busy: bool, clear: bool) {
        if clear {
            e.m.sl(0xA0, 0);
            e.m.sw(0xA4, 0);
        }
        e.m.sb(o::ACTION, action);
        e.b.act = 0;
        e.b.step = 0;
        e.m.sb(o::BUSY, busy as u8);
    }

    /// 0x62546 (picker 5): the heads that can act request the grab pose (6); action 11. Nothing without a head.
    fn c84_grab_a_start(e: &mut Enemy) {
        e.m.sl(0xA0, 0);
        e.m.sw(0xA4, 0);
        let d0 = e.m.b(o::HEADS);
        if d0 == 0 {
            return;
        }
        if d0 & 2 != 0 {
            e.m.sb(o::HEAD_L, 6);
        }
        if d0 & 1 != 0 {
            e.m.sb(o::HEAD_R, 6);
        }
        e.m.sb(o::ACTION, 0x16);
        e.b.act = 0;
        e.b.step = 0;
        e.m.sb(o::BUSY, 1);
    }

    /// 0x62664 (picker 6): action 12 when a head can act.
    fn c84_grab_b_start(e: &mut Enemy) {
        e.m.sl(0xA0, 0);
        e.m.sw(0xA4, 0);
        if e.m.b(o::HEADS) != 0 {
            e.m.sb(o::ACTION, 0x18);
            e.b.act = 0;
            e.b.step = 0;
            e.m.sb(o::BUSY, 1);
        }
    }

    /// 0x62C62: the picker: the X distance to the target (& 0xFF0) in classes of 0x30 (0..6, rows of 32 in 0x64E20) and `rand & 31`; the
    /// 480 frame limit restarts; 0 bite, 1 fire, 2 / 3 / 4 the arm strikes, 5 / 6 the grabs.
    fn c84_pick(&mut self, e: &mut Enemy) {
        let tx = self.lib_target(e).xi();
        let mut d0 = tx.wrapping_sub(e.b.xi());
        if d0 < 0 {
            d0 = d0.wrapping_neg();
        }
        let mut d0 = (d0 as u16) & 0xFF0;
        let mut d1 = 0u16;
        for k in 1..=6u16 {
            let (r, borrow) = d0.overflowing_sub(0x30);
            if borrow {
                break;
            }
            d0 = r;
            d1 = k * 0x20;
        }
        e.m.sw(o::LIMIT, 0x1E0);
        let r = (self.rand() & 0x1F) as u16;
        match self.assets.enemy.t.u8(t::PICKER, (d1 + r) as u32) {
            0 => Self::c84_set(e, 6, false, true),
            1 => Self::c84_set(e, 8, false, false),
            2 => Self::c84_set(e, 0xA, true, false),
            3 => Self::c84_set(e, 0xC, true, false),
            4 => Self::c84_set(e, 0xE, true, false),
            5 => Self::c84_grab_a_start(e),
            _ => Self::c84_grab_b_start(e),
        }
    }

    /// 0x62CFE: the gate, every 16 frames (`+$C0`): `rand & 31` at most the level's byte of 0x64E00.
    fn c84_gate(&mut self, e: &mut Enemy) -> bool {
        let c = e.m.b(o::GATE_T);
        e.m.sb(o::GATE_T, c.wrapping_sub(1));
        if c != 0 {
            return false;
        }
        e.m.sb(o::GATE_T, 0xF);
        let limit = self.assets.enemy.t.u8(t::GATE, self.w.level as u32) as u16;
        let r = (self.rand() & 0x1F) as u16;
        limit >= r
    }

    /// 0x629F0: the 480 frame limit `+$C6`: at its end the retarget and the reset (the action routine goes on in this frame).
    fn c84_limit(&mut self, e: &mut Enemy) {
        let t = e.m.w(o::LIMIT);
        if t != 0 {
            e.m.sw(o::LIMIT, t - 1);
            if t == 1 {
                self.c84_retarget(e);
            }
        }
    }

    /// 0x6312E: true when the target is gone (`+$00` 0) or is held by the body (`+$C8` 0x23).
    fn c84_target_lost(&self, e: &Enemy) -> bool {
        let t = e.m.w(TARGET);
        match slot_of(t) {
            Some(Slot::Player(p)) => self.players[p].b.live == 0 || self.players[p].action == 0x23,
            _ => self.body(t).live == 0,
        }
    }

    // ------------------------------------------------------------------ actions 0, 1, 2

    /// 0x61AE6 (action 0): the gate (then the picker); `+$B8` down; at its end a hero close in front (0x62D74, then the follow) or, unless `+$BA`,
    /// the walk.
    fn c84_action0(&mut self, e: &mut Enemy) {
        if self.c84_gate(e) {
            return self.c84_pick(e);
        }
        let t = e.m.w(o::IDLE);
        e.m.sw(o::IDLE, t.wrapping_sub(1));
        if t != 0 {
            return;
        }
        if let Some(p) = self.c84_close(e) {
            e.m.sw(TARGET, p);
            return Self::c84_follow_start(e);
        }
        e.m.sb(o::CLOSE_T, 0);
        if e.m.b(o::BA) != 0 {
            return Self::c84_follow_start(e);
        }
        Self::c84_walk_start(e);
    }

    /// 0x61B3E (action 1): the gate (then the picker); the walk's states (table 0x61B6A); a hero close in front: the walk animation settles
    /// (0x62E62) and the follow.
    fn c84_walk(&mut self, e: &mut Enemy) {
        if self.c84_gate(e) {
            return self.c84_pick(e);
        }
        match e.b.act {
            0 => self.c84_walk_plan(e),
            2 => self.c84_walk_go(e),
            _ => {
                // 0x61CE8: the pause, then the walk again.
                let t = e.m.w(0x80);
                e.m.sw(0x80, t.wrapping_sub(1));
                if t == 0 {
                    e.m.sb(0x80, 2);
                    e.b.act = 2;
                }
            }
        }
        if let Some(p) = self.c84_close(e) {
            e.m.sw(TARGET, p);
            self.c84_settle(e);
            Self::c84_follow_start(e);
        }
    }

    /// 0x61B70: the walk's plan: the body's band on the screen (X by 0x64AFA, Z by 0x64B00: 16 cells, `+$BB`), a cell picked from 0x64B46 by
    /// `rand & 31`, its point (0x64B06, relative to the camera) as the goal; the direction to it, speed id 7, `+$80` 2, `+$92` the walk time
    /// (0x64DCE), animation 1.
    fn c84_walk_plan(&mut self, e: &mut Enemy) {
        e.m.sb(o::BA, 0);
        let tb = &self.assets.enemy.t;
        let cam_x = self.cam.s2.cam_x as i16;
        let cam_y = self.cam.s2.cam_y as i16;
        let dx = e.b.xi().wrapping_sub(cam_x);
        let mut d1: u16 = 0xC;
        for (k, v) in [(0, 0u32), (4, 2), (8, 4)] {
            if dx < tb.i16(t::WALK_BANDS, v) {
                d1 = k;
                break;
            }
        }
        // `Z - $54 - (camera Y - $54)`: the floor offset cancels.
        let dz = e.b.zi().wrapping_sub(cam_y);
        for v in [6u32, 8, 10] {
            if dz >= tb.i16(t::WALK_BANDS, v) {
                break;
            }
            d1 += 1;
        }
        e.m.sb(o::BB, d1 as u8);
        let r = (self.rand() & 0x1F) as u16;
        let row = ((d1 & 0x1F) << 5).wrapping_add(r);
        let tb = &self.assets.enemy.t;
        let cell = tb.u8(t::WALK_PICK, row as u32);
        e.m.sb(o::BB, cell);
        let off = (cell as u32) * 4;
        let px = tb.i16(t::WALK_POINT, off).wrapping_add(cam_x);
        let pz = tb.i16(t::WALK_POINT, off + 2).wrapping_add(cam_y);
        e.m.sw(DEST_X, px as u16);
        e.m.sw(DEST_Z, pz as u16);
        e.b.dir = Self::enemy_angle_to(&e.b, px as u16, pz as u16);
        self.enemy_set_speed(&mut e.b, 7);
        e.m.sb(0x80, 2);
        e.m.sw(o::T92, self.assets.enemy.t.u16(t::WALK_TIME, 0));
        self.enemy_anim(&mut e.b, 1);
        e.b.act = e.b.act.wrapping_add(2);
    }

    /// 0x61C46: the walk: move along the direction (0x1862), the walk animation (0x62D34); at the end of `+$92` (reloaded) a level bit of
    /// 0x64DD0 against `rand & 31` decides a pause (`+$80` from 0x64D8E, state 4, the animation settles); at the goal (window 0) the idle time
    /// (0x64D8E), `+$BA` 0, the animation settles and the reset; every 3 frames the direction to the goal again.
    fn c84_walk_go(&mut self, e: &mut Enemy) {
        self.enemy_move_dir(&mut e.b); // 0x1862
        self.c84_walk_anim(e);
        let t = e.m.w(o::T92);
        e.m.sw(o::T92, t.wrapping_sub(1));
        if t == 0 {
            let tb = &self.assets.enemy.t;
            e.m.sw(o::T92, tb.u16(t::WALK_TIME, 0));
            let bits = tb.u32(t::WALK_TIME + 2, (self.w.level as u32).wrapping_mul(2));
            let r = self.rand() & 0x1F;
            if bits & (1 << r) == 0 {
                let r = (self.rand() & 0x3E) as u32;
                e.m.sw(0x80, self.assets.enemy.t.u16(t::IDLE_TIME, r));
                e.b.act = e.b.act.wrapping_add(2);
                return self.c84_settle(e);
            }
        }
        if self.c84_at_point(e, 0).0 {
            let r = (self.rand() & 0x3E) as u32;
            e.m.sw(o::IDLE, self.assets.enemy.t.u16(t::IDLE_TIME, r));
            e.m.sb(o::BA, 0);
            self.c84_settle(e);
            return Self::c84_reset(e);
        }
        let c = e.m.b(0x80);
        e.m.sb(0x80, c.wrapping_sub(1));
        if c == 0 {
            let (px, pz) = (e.m.w(DEST_X), e.m.w(DEST_Z));
            e.b.dir = Self::enemy_angle_to(&e.b, px, pz);
            e.m.sb(0x80, 2);
        }
    }

    /// 0x61D26 (action 2): the gate (then the picker); the follow's states (table 0x61D50); `+$92` down, at its end `+$BC` 0x3C and the walk.
    fn c84_follow(&mut self, e: &mut Enemy) {
        if self.c84_gate(e) {
            return self.c84_pick(e);
        }
        if e.b.act == 0 {
            // 0x61D54: toward the target: the direction, speed id 6, `+$80` 2, animation 1.
            let tgt = self.lib_target(e);
            let (tx, tz) = (tgt.xi() as u16, tgt.zi() as u16);
            e.m.sw(DEST_X, tx);
            e.m.sw(DEST_Z, tz);
            e.b.dir = Self::enemy_angle_to(&e.b, tx, tz);
            self.enemy_set_speed(&mut e.b, 6);
            e.m.sb(0x80, 2);
            self.enemy_anim(&mut e.b, 1);
            e.b.act = e.b.act.wrapping_add(2);
        } else {
            // 0x61D90: the goal beside the target (0x64DFC, on the side of the body); there the animation settles; else move and turn every
            // 3 frames.
            let tb = &self.assets.enemy.t;
            let mut d3 = tb.i16(t::FOLLOW, 0);
            let dz = tb.i16(t::FOLLOW, 2);
            let tgt = self.lib_target(e);
            let (tx, tz) = (tgt.xi(), tgt.zi());
            if tx < e.b.xi() {
                d3 = d3.wrapping_neg();
            }
            e.m.sw(DEST_X, tx.wrapping_add(d3) as u16);
            e.m.sw(DEST_Z, tz.wrapping_add(dz) as u16);
            if self.c84_at_point(e, 0).0 {
                self.c84_settle(e);
            } else {
                self.enemy_move_dir(&mut e.b);
                self.c84_walk_anim(e);
                let c = e.m.b(0x80);
                e.m.sb(0x80, c.wrapping_sub(1));
                if c == 0 {
                    let (px, pz) = (e.m.w(DEST_X), e.m.w(DEST_Z));
                    e.b.dir = Self::enemy_angle_to(&e.b, px, pz);
                    e.m.sb(0x80, 2);
                }
            }
        }
        let t = e.m.w(o::T92);
        e.m.sw(o::T92, t.wrapping_sub(1));
        if t == 0 {
            e.m.sb(o::CLOSE_T, 0x3C);
            Self::c84_walk_start(e);
        }
    }

    // ------------------------------------------------------------------ actions 3 .. 7

    /// The common start of the attack actions: animation 1, `+$80` 0, `+$92` `time`, speed id `speed`, the requests cleared, state 2.
    fn c84_attack_begin(&mut self, e: &mut Enemy, time: u16, speed: u16) {
        self.enemy_anim(&mut e.b, 1);
        e.m.sb(0x80, 0);
        e.m.sw(o::T92, time);
        self.enemy_set_speed(&mut e.b, speed);
        e.m.sl(0xA0, 0);
        e.m.sw(0xA4, 0);
        e.b.act = e.b.act.wrapping_add(2);
    }

    /// The approach of the attack actions: `+$92` down (at its end the picker); the aim at the point beside the target (`aim`, turning at most
    /// `turn` steps every `period` frames, 0x62A2A / 0x62ACA); at the window `win` true (with the window's last value); else walk.
    fn c84_approach(&mut self, e: &mut Enemy, aim: u32, period: u8, turn: u8, win: u32) -> Option<Option<u16>> {
        let t = e.m.w(o::T92);
        e.m.sw(o::T92, t.wrapping_sub(1));
        if t == 0 {
            self.c84_pick(e);
            return None;
        }
        self.c84_aim(e, aim, period, turn);
        let (at, d0) = self.c84_at_point(e, win);
        if at {
            return Some(Some(d0));
        }
        self.c84_walk_anim(e);
        self.enemy_move_dir(&mut e.b); // 0x1862
        Some(None)
    }

    /// 0x61E0C (action 3, the bite): no head: the picker; the target lost: the retarget; the limit (0x629F0); states (0x61E2E): 0 the start
    /// (0xB4 frames, speed id 0x1C), 2 the approach (0x64F00, window 0x10): there `+$AE` from 0x64F04, the head that bites (both heads: by the
    /// target's Z, unsigned), its request 2, 0xF0 frames, animation 0; 4 the wait while that head bites.
    fn c84_bite(&mut self, e: &mut Enemy) {
        if e.m.b(o::HEADS) == 0 {
            return self.c84_pick(e);
        }
        if self.c84_target_lost(e) {
            return self.c84_retarget(e);
        }
        self.c84_limit(e);
        match e.b.act {
            0 => self.c84_attack_begin(e, 0xB4, 0x1C),
            2 => {
                let Some(Some(d0)) = self.c84_approach(e, t::BITE_AIM, 5, 4, 0x10) else { return };
                e.m.sb(o::REPEATS, self.assets.enemy.t.u8(t::BITE_COUNT, (d0 & 0x1F) as u32));
                let heads = e.m.b(o::HEADS);
                let left = match heads {
                    3 => (self.lib_target(e).zi() as u16) < (e.b.zi() as u16),
                    1 => false,
                    _ => true,
                };
                if left {
                    e.m.sb(o::BITE_HEAD, 2);
                    e.m.sb(o::HEAD_L, 2);
                } else {
                    e.m.sb(o::BITE_HEAD, 1);
                    e.m.sb(o::HEAD_R, 2);
                }
                e.b.act = e.b.act.wrapping_add(2);
                e.m.sb(0x80, 0xF0);
                self.enemy_anim(&mut e.b, 0);
            }
            _ => {
                // 0x61EEA
                let c = e.m.b(0x80);
                e.m.sb(0x80, c.wrapping_sub(1));
                if c == 0 {
                    return Self::c84_reset(e);
                }
                if e.m.b(o::BITE_HEAD) & 1 == 0 {
                    if self.c84_part_b(e, o::P_HEAD_L, o::HEAD_L) != 2 {
                        e.m.sb(o::HEAD_L, 0);
                        Self::c84_reset(e);
                    }
                } else if self.c84_part_b(e, o::P_HEAD_R, o::HEAD_R) != 2 {
                    e.m.sb(o::HEAD_R, 0);
                    Self::c84_reset(e);
                }
            }
        }
    }

    /// 0x61F3E (action 4, the fire): states (0x61F60): 0 the start (0x78 frames, speed id 0x19); 2 the approach (0x64F24, window 0x18): there
    /// `+$AE` volleys (0x64F28), `+$80` 0x1E, `+$81` 0, `+$92` 0x168, the first request (0x61FE8), animation 0; 4 a head fires every 0x1E frames
    /// until `+$81` reaches `+$AE` or `+$92` ends (then `+$80` 0x64); 6 the wait for the heads, then the reset.
    fn c84_fire(&mut self, e: &mut Enemy) {
        if e.m.b(o::HEADS) == 0 {
            return self.c84_pick(e);
        }
        if self.c84_target_lost(e) {
            return self.c84_retarget(e);
        }
        self.c84_limit(e);
        match e.b.act {
            0 => self.c84_attack_begin(e, 0x78, 0x19),
            2 => {
                let Some(Some(d0)) = self.c84_approach(e, t::FIRE_AIM, 0xA, 2, 0x18) else { return };
                e.m.sb(o::REPEATS, self.assets.enemy.t.u8(t::FIRE_COUNT, (d0 & 0x1F) as u32));
                e.m.sb(0x80, 0x1E);
                e.m.sb(o::T81, 0);
                e.m.sw(o::T92, 0x168);
                self.c84_fire_request(e);
                e.b.act = e.b.act.wrapping_add(2);
                self.enemy_anim(&mut e.b, 0);
            }
            4 => {
                // 0x62040
                let t = e.m.w(o::T92);
                e.m.sw(o::T92, t.wrapping_sub(1));
                if t != 0 {
                    if self.c84_part_b(e, o::P_HEAD_L, o::HEAD_L) != 4 {
                        e.m.sb(o::HEAD_L, 0);
                    } else if self.c84_part_b(e, o::P_HEAD_R, o::HEAD_R) != 4 {
                        e.m.sb(o::HEAD_R, 0);
                    }
                    let c = e.m.b(0x80);
                    e.m.sb(0x80, c.wrapping_sub(1));
                    if c != 0 {
                        return;
                    }
                    e.m.sb(0x80, 0x1E);
                    self.c84_fire_request(e);
                    if e.m.b(o::T81) < e.m.b(o::REPEATS) {
                        return;
                    }
                }
                e.m.sb(0x80, 0x64);
                e.b.act = e.b.act.wrapping_add(2);
            }
            _ => {
                // 0x6209E
                if self.c84_part_b(e, o::P_HEAD_L, o::HEAD_L) == 0 {
                    e.m.sb(o::HEAD_L, 0);
                }
                if self.c84_part_b(e, o::P_HEAD_R, o::HEAD_R) == 0 {
                    e.m.sb(o::HEAD_R, 0);
                }
                let c = e.m.b(0x80);
                e.m.sb(0x80, c.wrapping_sub(1));
                if c == 0 {
                    Self::c84_reset(e);
                }
            }
        }
    }

    /// 0x61FE8: one head is asked to fire (request 4) when it is free: both heads alternate by `+$81`, else the one that can act; `+$81` + 1.
    fn c84_fire_request(&mut self, e: &mut Enemy) {
        let left = match e.m.b(o::HEADS) {
            3 => e.m.b(o::T81) & 1 == 0,
            1 => false,
            _ => true,
        };
        if left {
            if self.c84_part_b(e, o::P_HEAD_L, o::HEAD_L) != 0 {
                return;
            }
            e.m.sb(o::HEAD_L, 4);
        } else {
            if self.c84_part_b(e, o::P_HEAD_R, o::HEAD_R) != 0 {
                return;
            }
            e.m.sb(o::HEAD_R, 4);
        }
        e.m.sb(o::T81, e.m.b(o::T81).wrapping_add(1));
    }

    /// 0x620E6 / 0x621A0 / 0x6225A (actions 5, 6, 7, the arm strikes 0x5F, 0x60, 0x61): the limit, the target lost: the retarget; 0 the start
    /// (0xB4 frames, speed id 0x1C); 2 the approach (0x64F48 / 0x64F4C / 0x64F50, windows 0x20 / 0x28 / 0x30): there the arm request 2 / 4 / 6,
    /// 0xF0 frames, animation 0; 4 the requests cleared (the arm saw it), the wait while the arm strikes.
    fn c84_arm(&mut self, e: &mut Enemy, k: usize) {
        self.c84_limit(e);
        if self.c84_target_lost(e) {
            return self.c84_retarget(e);
        }
        let req = 2 * (k as u8 + 1);
        match e.b.act {
            0 => self.c84_attack_begin(e, 0xB4, 0x1C),
            2 => {
                let Some(Some(_)) = self.c84_approach(e, t::ARM_AIM[k], 5, 4, 0x20 + 8 * k as u32) else { return };
                e.m.sb(o::ARM, req);
                e.m.sw(o::T92, 0xF0);
                e.b.act = e.b.act.wrapping_add(2);
                self.enemy_anim(&mut e.b, 0);
            }
            _ => {
                e.m.sl(0xA0, 0);
                e.m.sw(0xA4, 0);
                let t = e.m.w(o::T92);
                e.m.sw(o::T92, t.wrapping_sub(1));
                if t == 0 || self.c84_part_b(e, o::P_ARM, o::ARM) != req {
                    Self::c84_reset(e);
                }
            }
        }
    }

    // ------------------------------------------------------------------ actions 8, 9, 10

    /// 0x6231C (action 8): the limit, the target lost: the retarget; 0 (0x6233A): no Y speed or accelerations, `+$80` 0xB4, animation 1; 2
    /// (0x62360): away from the target in X (2 px) and Z (1.5 px) while it is within 0x50 (|dx| + |dz|) and `+$80` lasts; then the animation
    /// settles, idle 0x1E and the reset.
    fn c84_back_off(&mut self, e: &mut Enemy) {
        self.c84_limit(e);
        if self.c84_target_lost(e) {
            return self.c84_retarget(e);
        }
        if e.b.act == 0 {
            e.b.vy = 0;
            e.b.ax = 0;
            e.b.ay = 0;
            e.b.az = 0;
            e.m.sb(0x80, 0xB4);
            self.enemy_anim(&mut e.b, 1);
            e.b.act = e.b.act.wrapping_add(2);
            return;
        }
        let c = e.m.b(0x80);
        e.m.sb(0x80, c.wrapping_sub(1));
        let near = c != 0 && {
            let tgt = self.lib_target(e).clone();
            let (mut vx, mut dir) = (0x200i16, 8u8);
            if tgt.xi() > e.b.xi() {
                vx = -0x200;
                dir = 0x18;
            }
            e.b.vx = vx;
            e.b.dir = dir;
            e.b.vz = if tgt.zi() > e.b.zi() { -0x180 } else { 0x180 };
            Self::enemy_manhattan(&e.b, &tgt) < 0x50
        };
        if near {
            self.c84_walk_anim(e);
            return e.b.motion(); // 0x189C
        }
        self.c84_settle(e);
        e.m.sw(o::IDLE, 0x1E);
        Self::c84_reset(e);
    }

    /// 0x623E0 (action 9, the dash) and 0x6249A (action 10, the step): the target lost: the retarget; 0: 4 px per frame for 0x50 px along the
    /// facing (or 2 px in Z for 0x24 px toward the arena middle), `+$80` 0x78, animation 1; 2: until the end coordinate `+$92` or `+$80`, then
    /// the animation settles and the retarget.
    fn c84_dash(&mut self, e: &mut Enemy) {
        if self.c84_target_lost(e) {
            return self.c84_retarget(e);
        }
        if e.b.act == 0 {
            e.b.vy = 0;
            e.b.ax = 0;
            e.b.ay = 0;
            e.b.vz = 0;
            e.b.az = 0;
            let (mut d0, mut d1) = (0x400i16, 0x50i16);
            if e.b.facing != 0 {
                d0 = -d0;
                d1 = -d1;
            }
            e.b.vx = d0;
            e.m.sw(o::T92, e.b.xi().wrapping_add(d1) as u16);
            e.m.sw(0x80, 0x78);
            self.enemy_anim(&mut e.b, 1);
            e.b.act = e.b.act.wrapping_add(2);
            return;
        }
        let pos = e.b.xi();
        let v = e.b.vx;
        self.c84_run_to(e, pos, v);
    }

    fn c84_step(&mut self, e: &mut Enemy) {
        if self.c84_target_lost(e) {
            return self.c84_retarget(e);
        }
        if e.b.act == 0 {
            e.b.vx = 0;
            e.b.ax = 0;
            e.b.vy = 0;
            e.b.ay = 0;
            e.b.az = 0;
            let (mut d0, mut d1) = (0x200i16, 0x24i16);
            let d2 = (self.cam.s2.cam_z as i16).wrapping_sub(e.b.floor).wrapping_add(0x1E);
            if d2 < e.b.zi() {
                d0 = -d0;
                d1 = -d1;
            }
            e.b.vz = d0;
            e.m.sw(o::T92, e.b.zi().wrapping_add(d1) as u16);
            e.m.sw(0x80, 0x78);
            self.enemy_anim(&mut e.b, 1);
            e.b.act = e.b.act.wrapping_add(2);
            return;
        }
        let pos = e.b.zi();
        let v = e.b.vz;
        self.c84_run_to(e, pos, v);
    }

    /// 0x62446 / 0x6250C: the run of the dash and the step toward the end coordinate `+$92` (`pos` the current one, `v` the speed's sign).
    fn c84_run_to(&mut self, e: &mut Enemy, pos: i16, v: i16) {
        let t = e.m.w(0x80);
        e.m.sw(0x80, t.wrapping_sub(1));
        let end = e.m.w(o::T92) as i16;
        let go = t != 0 && if v < 0 { end < pos } else { end >= pos };
        if go {
            self.c84_walk_anim(e);
            return e.b.motion(); // 0x189C
        }
        self.c84_settle(e);
        self.c84_retarget(e);
    }

    // ------------------------------------------------------------------ actions 11 and 12 (the grabs)

    /// 0x62584 / 0x62686 (actions 11 and 12): the limit, the target lost: the retarget; 0: animation 1, `+$80` 0, `+$92` 0x78, the point beside
    /// the target (0x64F54 / 0x64F58) and the speeds (0x62B44); 2: `+$92` down (the picker at its end), toward the point (0x62BBE); at the window
    /// 0x38 / 0x40 with the target live and on the ground: state 4 and animation 5 (action 12: the arm and limb requests 8, animation 7), else the
    /// animation settles and the retarget; 4: the animation (action 11: at its end the retarget); at a frame with `+$2C` the point again, then
    /// the window and the grab test (0x4E9A6) start the grab (`+$05` 4).
    fn c84_grab_walk(&mut self, e: &mut Enemy, b: bool) {
        self.c84_limit(e);
        if self.c84_target_lost(e) {
            return self.c84_retarget(e);
        }
        let (table, win) = if b { (t::GRAB_B, 0x40) } else { (t::GRAB_A, 0x38) };
        match e.b.act {
            0 => {
                self.enemy_anim(&mut e.b, 1);
                e.m.sb(0x80, 0);
                e.m.sw(o::T92, 0x78);
                e.b.vy = 0;
                e.b.ay = 0;
                e.b.az = 0;
                self.c84_grab_point(e, table, true);
                e.b.act = e.b.act.wrapping_add(2);
            }
            2 => {
                let t = e.m.w(o::T92);
                e.m.sw(o::T92, t.wrapping_sub(1));
                if t == 0 {
                    return self.c84_pick(e);
                }
                self.c84_grab_point(e, table, false);
                if !self.c84_at_point(e, win).0 {
                    self.c84_walk_anim(e);
                    return e.b.motion(); // 0x189C
                }
                let tgt = self.lib_target(e);
                if tgt.live == 1 && tgt.airborne == 0 {
                    e.b.act = e.b.act.wrapping_add(2);
                    if b {
                        e.m.sb(o::ARM, 8);
                        e.m.sb(o::LIMB, 8);
                        return self.enemy_anim(&mut e.b, 7);
                    }
                    return self.enemy_anim(&mut e.b, 5);
                }
                self.c84_settle(e);
                self.c84_retarget(e);
            }
            _ => {
                let end = self.enemy_step(&mut e.b);
                if end && !b {
                    return self.c84_retarget(e);
                }
                if e.b.param_a == 0 {
                    return;
                }
                e.b.param_a = 0;
                self.c84_grab_point(e, table, false);
                if !self.c84_at_point(e, win).0 || !self.c84_grab_test(e) {
                    return self.c84_retarget(e);
                }
                e.b.sub = 4;
                e.b.act = 0;
                e.b.step = 0;
            }
        }
    }

    /// 0x62B44 (`first`) / 0x62BBE: the point beside the target (`table`: X on the side of the body, kept on the screen by 0x62AA6; Z); vz 1 px
    /// toward it (0 within 4); vx 2 px with ax 0x10 toward it (0 within 4): 0x62BBE keeps a running vx until the point is passed, then stops.
    fn c84_grab_point(&mut self, e: &mut Enemy, table: u32, first: bool) {
        let tgt = self.lib_target(e).clone();
        let tb = &self.assets.enemy.t;
        let mut d3 = tb.i16(table, 0);
        let dz = tb.i16(table, 2);
        let tx = tgt.xi();
        if tx < e.b.xi() {
            d3 = d3.wrapping_neg();
        }
        d3 = self.c84_keep_on_screen(tx, d3);
        let d1 = tx.wrapping_add(d3);
        let d2 = tgt.zi().wrapping_add(dz);
        e.m.sw(DEST_X, d1 as u16);
        e.m.sw(DEST_Z, d2 as u16);
        let mut vz = if d2 >= e.b.zi() { 0x100 } else { -0x100 };
        if (d2.wrapping_sub(e.b.zi()).wrapping_add(4) as u16) <= 8 {
            vz = 0;
        }
        e.b.vz = vz;
        if !first && e.b.vx != 0 {
            let passed = if e.b.vx < 0 { d1 > e.b.xi() } else { d1 < e.b.xi() };
            if passed {
                e.b.vx = 0;
                e.b.ax = 0;
            }
            return;
        }
        let (mut vx, mut ax) = (0x200i16, 0x10i16);
        if d1 < e.b.xi() {
            vx = -vx;
            ax = -ax;
        }
        if (d1.wrapping_sub(e.b.xi()).wrapping_add(4) as u16) <= 8 {
            vx = 0;
            ax = 0;
        }
        e.b.vx = vx;
        e.b.ax = ax;
    }

    /// 0x4E9A6: the grab test: the target (a hero, `+$82` 0) live, in state 0x0200, on the ground and not in the action 0x11.
    fn c84_grab_test(&self, e: &Enemy) -> bool {
        let t = e.m.w(TARGET);
        let Some(Slot::Player(p)) = slot_of(t) else { return false };
        let pl = &self.players[p];
        pl.b.live == 1 && (pl.b.state() >> 16) == 0x0200 && pl.b.airborne == 0 && pl.action != 0x11
    }

    /// 0x6276E (`+$05` 4): the grab by `+$06` (table 0x62786); then the hurtable list and the draw (0x49C0).
    fn c84_grab(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => {
                // 0x62790: the test again (it leaves d0 = 0: window 0), then hold the hero (0x627C4).
                if !self.c84_grab_test(e) || !self.c84_at_point(e, 0).0 {
                    return self.c84_release(e, h);
                }
                e.b.visible = 1;
                self.c84_hold(e, h);
                if e.m.b(o::ACTION) == 0x18 {
                    e.b.act = 6;
                } else {
                    e.b.act = e.b.act.wrapping_add(2);
                }
            }
            2 => {
                // 0x62826
                if self.enemy_step(&mut e.b) {
                    e.b.act = e.b.act.wrapping_add(2);
                    self.enemy_anim(&mut e.b, 6);
                } else if !self.c84_held(e) {
                    return self.c84_release(e, h);
                }
            }
            4 => {
                // 0x62842: at the frame with `+$2C` 2 the hit 0x64 and the drop in front.
                if self.enemy_step(&mut e.b) {
                    return self.c84_release(e, h);
                }
                if e.b.param_a == 2 {
                    e.b.param_a = 1;
                    if !self.c84_held(e) {
                        return self.c84_release(e, h);
                    }
                    self.enemy_begin_attack(&mut e.b, 0x64);
                    self.enemy_hit_held_player(e, h); // 0x125C0
                    let t = e.m.w(TARGET);
                    let Some(Slot::Player(p)) = slot_of(t) else { return self.c84_release(e, h) };
                    if self.players[p].action != 0x40 {
                        return self.c84_release(e, h);
                    }
                    let (dx, f) = if e.b.facing != 0 { (-0x18i16, 1u8) } else { (0x18, 0) };
                    let (x, y) = (e.b.xi().wrapping_add(dx), e.b.yi().wrapping_add(0x70));
                    let pl = &mut self.players[p];
                    pl.b.set_state(0x0202_0000);
                    pl.b.act = 8;
                    pl.b.set_xi_keep(x);
                    pl.b.set_yi_keep(y);
                    pl.b.facing = f;
                }
            }
            6 => {
                // 0x628C2: then the chews (`+$AE` from 0x650DA by level), animation 8.
                if self.enemy_step(&mut e.b) {
                    e.m.sb(o::REPEATS, self.assets.enemy.t.u8(t::CHEWS, self.w.level as u32));
                    e.b.act = e.b.act.wrapping_add(2);
                    self.enemy_anim(&mut e.b, 8);
                } else if !self.c84_held(e) {
                    return self.c84_release(e, h);
                }
            }
            _ => {
                // 0x628F2: the chew: the hero knocked down or thrown lets go; a hero back in the normal state is held again; at the end of the
                // animation one chew less (none left: let go); at the frame with `+$2C` 2 the hit 0x65.
                if self.c84_target_down(e) {
                    return self.c84_release(e, h);
                }
                self.c84_rehold(e, h);
                if self.enemy_step(&mut e.b) {
                    let n = e.m.b(o::REPEATS).wrapping_sub(1);
                    e.m.sb(o::REPEATS, n);
                    if n == 0 {
                        return self.c84_release(e, h);
                    }
                }
                if e.b.param_a == 2 {
                    e.b.param_a = 1;
                    self.enemy_begin_attack(&mut e.b, 0x65);
                    self.enemy_hit_held_player(e, h); // 0x125C0
                }
            }
        }
        self.enemy_push_hurtable(h);
        self.enemy_draw_visible(&e.b, h); // 0x49C0
    }

    /// 0x62822: the grab ends in the recover entry (0x61A6C).
    fn c84_release(&mut self, e: &mut Enemy, h: Handle) {
        self.c84_recover(e);
        self.enemy_push_hurtable(h);
        self.enemy_draw_visible(&e.b, h);
    }

    /// 0x627C4: hold the target: the links in `+$70`, the hero in state 0x02060000, action 0x40, frame class 0xE, `+$C7` 0, HP before = HP,
    /// `+$62` 1.
    fn c84_hold(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(TARGET);
        e.b.partner = t;
        if let Some(Slot::Player(p)) = slot_of(t) {
            let pl = &mut self.players[p];
            pl.b.partner = h;
            pl.b.set_state(0x0206_0000);
            pl.action = 0x40;
            pl.b.kind = 0xE;
            pl.bc7 = 0;
            pl.b.hp_prev = pl.b.hp;
            pl.b.b62 = 1;
        }
    }

    /// 0x627F6: the hold check: the target live, in state 0x0206 and frame class 0xE.
    fn c84_held(&self, e: &Enemy) -> bool {
        let b = self.body(e.m.w(TARGET));
        b.live == 1 && (b.state() >> 16) == 0x0206 && b.kind == 0xE
    }

    /// 0x62972: true when the target was knocked down (`+$7B` 4) or is in the state 0x0200000E.
    fn c84_target_down(&self, e: &Enemy) -> bool {
        let b = self.body(e.m.w(TARGET));
        b.reaction == 4 || b.state() == 0x0200_000E
    }

    /// 0x62940: a target live, in the normal state 0x0200 and not in the frame classes 4 or 6 is held again.
    fn c84_rehold(&mut self, e: &mut Enemy, h: Handle) {
        let b = self.body(e.m.w(TARGET));
        if b.live == 1 && (b.state() >> 16) == 0x0200 && b.kind != 4 && b.kind != 6 {
            self.c84_hold(e, h);
        }
    }

    // ------------------------------------------------------------------ the walking helpers

    /// 0x62D34: the walk animation by the direction against the facing: forward (0x121E) or backward (0x1380, `+$BF` 1); straight up or down
    /// keeps the last way.
    fn c84_walk_anim(&self, e: &mut Enemy) {
        let back = if e.b.dir & 0xF == 0 {
            e.m.b(o::BACK) != 0
        } else {
            ((e.b.dir >> 4) & 1) ^ e.b.facing == 0
        };
        if back {
            e.m.sb(o::BACK, 1);
            self.enemy_step_back(&mut e.b);
        } else {
            e.m.sb(o::BACK, 0);
            self.enemy_step(&mut e.b);
        }
    }

    /// 0x62E62: the walk animation settles: up to two more frames (forward or backward by the direction) until a frame with `+$2C`.
    fn c84_settle(&self, e: &mut Enemy) {
        let fwd = ((e.b.dir >> 4) & 1) ^ e.b.facing != 0;
        for _ in 0..2 {
            if e.b.param_a != 0 {
                return;
            }
            e.b.timer = 1;
            if fwd {
                self.enemy_step(&mut e.b);
            } else {
                self.enemy_step_back(&mut e.b);
            }
        }
    }

    /// 0x62EC8: true when the point (`+$88`, `+$8A`) is inside the window `off` of 0x64D46 (X mirrored by the target's facing); also the last
    /// value computed (d0, the Z test or the X test when that failed).
    fn c84_at_point(&self, e: &Enemy, off: u32) -> (bool, u16) {
        let tb = &self.assets.enemy.t;
        let mut d0 = tb.u16(t::WINDOWS, off);
        let d1 = tb.u16(t::WINDOWS, off + 2);
        if self.lib_target(e).facing != 0 {
            d0 = d1.wrapping_sub(d0);
        }
        let d2 = e.m.w(DEST_X).wrapping_sub(e.b.xi() as u16).wrapping_add(d0);
        if d2 > d1 {
            return (false, d0);
        }
        let z = e.m.w(DEST_Z).wrapping_sub(e.b.zi() as u16).wrapping_add(tb.u16(t::WINDOWS, off + 4));
        (z <= tb.u16(t::WINDOWS, off + 6), z)
    }

    /// 0x62AA6: the offset `d3` beside the target X `d1` turns round when the point would leave the screen (0 .. 0x180 from the camera).
    fn c84_keep_on_screen(&self, d1: i16, d3: i16) -> i16 {
        let d0 = d1.wrapping_sub(self.cam.s2.cam_x as i16).wrapping_sub(d3);
        if d3 >= 0 {
            if d0 > 0x180 {
                return d3.wrapping_neg();
            }
        } else if d0 < 0 {
            return d3.wrapping_neg();
        }
        d3
    }

    /// 0x62A2A (`turn` 4, every 6 frames) / 0x62ACA (2, every 11): the point beside the target (`table`) and a turn of at most `turn` steps
    /// toward it.
    fn c84_aim(&self, e: &mut Enemy, table: u32, period: u8, turn: u8) {
        let c = e.m.b(0x80);
        e.m.sb(0x80, c.wrapping_sub(1));
        if c != 0 {
            return;
        }
        e.m.sb(0x80, period);
        let tb = &self.assets.enemy.t;
        let mut d3 = tb.i16(table, 0);
        let tgt = self.lib_target(e);
        let (tx, tz) = (tgt.xi(), tgt.zi());
        if tx < e.b.xi() {
            d3 = d3.wrapping_neg();
        }
        d3 = self.c84_keep_on_screen(tx, d3);
        let d1 = tx.wrapping_add(d3);
        let d2 = tz.wrapping_add(tb.i16(table, 2));
        e.m.sw(DEST_X, d1 as u16);
        e.m.sw(DEST_Z, d2 as u16);
        let mut d0 = Self::enemy_angle_to(&e.b, d1 as u16, d2 as u16);
        let diff = d0.wrapping_sub(e.b.dir) & 0x1F;
        if diff < 0x10 {
            if diff > turn {
                d0 = e.b.dir.wrapping_add(turn) & 0x1F;
            }
        } else if diff < 0x20 - turn {
            d0 = e.b.dir.wrapping_sub(turn) & 0x1F;
        }
        e.b.dir = d0;
    }

    /// 0x62D74: every other frame (`+$BC`), the first hero (P3 only when G.$4CF is 0) inside the box 0x64DF0 in front of the body.
    fn c84_close(&mut self, e: &mut Enemy) -> Option<Handle> {
        let c = e.m.b(o::CLOSE_T);
        e.m.sb(o::CLOSE_T, c.wrapping_sub(1));
        if c != 0 {
            return None;
        }
        e.m.sb(o::CLOSE_T, 1);
        let tb = &self.assets.enemy.t;
        let mut d0 = tb.i16(t::CLOSE_BOX, 0);
        if e.b.facing != 0 {
            d0 = d0.wrapping_neg();
        }
        let (d1, d2) = (tb.u16(t::CLOSE_BOX, 2), tb.u16(t::CLOSE_BOX, 4));
        let (z0, z1, z2) = (tb.i16(t::CLOSE_BOX, 6), tb.u16(t::CLOSE_BOX, 8), tb.u16(t::CLOSE_BOX, 10));
        for (k, p) in [P1, P2, P3].into_iter().enumerate() {
            if k == 2 && self.flow.b4cf != 0 {
                break;
            }
            let pb = &self.players[k].b;
            if pb.live != 1 {
                continue;
            }
            let dx = e.b.xi().wrapping_add(d0).wrapping_sub(pb.xi()) as u16;
            if dx.wrapping_add(d1) > d2 {
                continue;
            }
            let dz = e.b.zi().wrapping_add(z0).wrapping_sub(pb.zi()) as u16;
            if dz.wrapping_add(z1) > z2 {
                continue;
            }
            return Some(p);
        }
        None
    }

    /// 0x632EA: the body is kept in camera X - 0xA0 .. + 0x220 and its Z (+ height + floor) in camera Y .. + 0x100.
    fn c84_screen_clamp(&self, e: &mut Enemy) {
        let d0 = e.b.xi().wrapping_sub(self.cam.s2.cam_x as i16).wrapping_add(0xA0);
        if d0 < 0 {
            e.b.x = e.b.x.wrapping_sub((d0 as i32) << 16);
        } else {
            let d0 = d0.wrapping_sub(0x2C0);
            if d0 >= 0 {
                e.b.x = e.b.x.wrapping_sub((d0 as i32) << 16);
            }
        }
        let d0 = e.b.zi().wrapping_add(e.b.floor2).wrapping_add(e.b.floor).wrapping_sub(self.cam.s2.cam_y as i16);
        if d0 < 0 {
            e.b.z = e.b.z.wrapping_sub((d0 as i32) << 16);
        } else {
            let d0 = d0.wrapping_sub(0x100);
            if d0 >= 0 {
                e.b.z = e.b.z.wrapping_sub((d0 as i32) << 16);
            }
        }
    }

    // ------------------------------------------------------------------ the counters

    /// 0x62996: after a counter, a request the part has finished (its own request byte back to 0) is cleared in the body.
    fn c84_counter_clear(&self, e: &mut Enemy) {
        if e.m.b(o::COUNTERED) == 0 {
            return;
        }
        for (part, field) in [(o::P_HEAD_L, o::HEAD_L), (o::P_HEAD_R, o::HEAD_R), (o::P_ARM, o::ARM), (o::P_LIMB, o::LIMB)] {
            if self.c84_part_b(e, part, field) == 0 {
                e.m.sb(field, 0);
            }
        }
    }

    /// The HP fraction (0 at 3/4 or more of the start HP, 4 at 1/2, 8 at 1/4, 12 below) of 0x62FCC / 0x6321E.
    fn c84_hp_band(e: &Enemy) -> u32 {
        let half = e.m.w(o::START_HP) >> 1;
        let quarter = half >> 1;
        let three = half.wrapping_add(quarter);
        let hp = e.b.hp;
        if hp >= three {
            0
        } else if hp >= half {
            4
        } else if hp >= quarter {
            8
        } else {
            12
        }
    }

    /// 0x6314E: with the counters on, the first hero (P1, P2, P3) in the normal state inside the zone in front of the body (0x65044), facing
    /// it and doing a jump attack (0x14), a running jump attack (0x02), a dash attack (0x13) or the command move (0x15), when `rand & 31` hits
    /// a bit of the HP fraction's mask (0x6504A).
    fn c84_counter_test(&mut self, e: &Enemy) -> Option<usize> {
        if e.m.b(o::BUSY) != 0 {
            return None;
        }
        for k in 0..3 {
            let pb = &self.players[k].b;
            if pb.live != 1 || (pb.state() >> 16) != 0x0200 {
                continue;
            }
            let tb = &self.assets.enemy.t;
            let dz = e.b.zi().wrapping_sub(pb.zi()).wrapping_add(tb.i16(t::COUNTER_BOX, 0)) as u16;
            if dz > tb.u16(t::COUNTER_BOX, 2) {
                continue;
            }
            let reach = tb.i16(t::COUNTER_BOX, 4);
            let (px, x) = (pb.xi(), e.b.xi());
            let ok = if e.b.facing == 0 {
                px <= x && px.wrapping_add(reach) >= x && pb.facing != 0
            } else {
                px >= x && px.wrapping_sub(reach) <= x && pb.facing == 0
            };
            if !ok || !matches!(self.players[k].action, 0x14 | 0x02 | 0x13 | 0x15) {
                continue;
            }
            let mask = self.assets.enemy.t.u32(t::COUNTER_MASK, Self::c84_hp_band(e));
            let r = self.rand() & 0x1F;
            if mask & (1 << r) != 0 {
                return Some(k);
            }
        }
        None
    }

    /// 0x6326E: the counter by the hero's move (rows of 0x6505A, `rand & 31`): 1 / 2 / 3 the arm strikes, 5 the dash, 6 the step; `+$C9` 1.
    fn c84_counter(&mut self, e: &mut Enemy, k: usize) {
        let row = match self.players[k].action {
            0x14 => 0,
            0x02 => 0x20,
            0x13 => 0x40,
            _ => 0x60,
        };
        let r = (self.rand() & 0x1F) as u32;
        let v = self.assets.enemy.t.u8(t::COUNTER_PICK, row + r);
        e.m.sb(o::COUNTERED, 1);
        match v {
            1 => e.m.sb(o::ARM, 2),
            2 => e.m.sb(o::ARM, 4),
            3 => e.m.sb(o::ARM, 6),
            5 => Self::c84_set(e, 0x12, true, true),
            6 => Self::c84_set(e, 0x14, true, true),
            _ => {}
        }
    }

    /// 0x63102: after a knock down the body is out of reach (`+$00` 2) for `+$C4` frames (0x64FC4 by the level's band and `rand & 31`).
    fn c84_away(&mut self, e: &mut Enemy) {
        let r = (self.rand() & 0x1F) as u32;
        let band = ((self.w.level & 0x18) as u32) * 4;
        e.m.sb(o::AWAY, self.assets.enemy.t.u8(t::AWAY_TIME, band + r));
        e.b.live = 2;
    }

    /// 0x62FCC: the answer to a knock down, when `rand & 31` hits the HP fraction's mask (0x64FA2): the hero nearest the left of the screen
    /// becomes the target; by the body's place on the screen (0x64FB2) and the X distance (0x64FBE) the back off, the arm strike 0x61, the
    /// arm strike 0x5F or the fire. True when one started.
    fn c84_knockdown_answer(&mut self, e: &mut Enemy) -> bool {
        let mask = self.assets.enemy.t.u32(t::KO_MASK, Self::c84_hp_band(e));
        let r = self.rand() & 0x1F;
        if mask & (1 << r) == 0 {
            return false;
        }
        let sx = e.b.xi().wrapping_sub(self.cam.s2.cam_x as i16);
        let d1 = if sx <= 0x36 { 0 } else if sx <= 0x14A { 4 } else { 8 };
        let mut t = P1;
        let mut d0: u16 = 0x300;
        if self.players[0].b.live & 1 != 0 {
            d0 = self.players[0].b.xi() as u16;
        }
        if self.players[1].b.live & 1 != 0 && d0 >= self.players[1].b.xi() as u16 {
            t = P2;
            d0 = self.players[1].b.xi() as u16;
        }
        if self.flow.b4cf == 0 && self.players[2].b.live & 1 != 0 && d0 >= self.players[2].b.xi() as u16 {
            t = P3;
            d0 = self.players[2].b.xi() as u16;
        }
        e.m.sw(TARGET, t);
        let mut d = (d0 as i16).wrapping_sub(e.b.xi());
        if d < 0 {
            d = d.wrapping_neg();
        }
        let d = (d as u16) & 0xF0;
        let tb = &self.assets.enemy.t;
        let mut k = 0u32;
        for v in 0..3u32 {
            if d < tb.u16(t::KO_BANDS, v * 2) {
                break;
            }
            k = v + 1;
        }
        match tb.u8(t::KO_PICK, d1 + k) {
            0 => Self::c84_set(e, 0x10, true, true),
            1 => Self::c84_set(e, 0xE, true, false),
            2 => Self::c84_set(e, 0xA, true, false),
            3 => Self::c84_set(e, 8, false, false),
            _ => return false,
        }
        true
    }

    // ------------------------------------------------------------------ the hit and the death

    /// 0x63460: the hit (only when HP changed): the action and the requests dropped, the credit (0x3262A), HP before = HP, at 0 the stage end
    /// (0xACEE); the hit stop (or 8); poise 10; the guard counter less the cost of the state (0x6350C: the attack phases 4, 6, 0xA, 0xC): at 0
    /// a knock down (poise 0, the counter back to 2); else the states 4 and 6 become 0x12 (light reaction); at HP 0 the camera clamp; 0x2A606.
    fn c84_hit(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.hp != e.b.hp_prev {
            e.m.sb(o::BUSY, 0);
            e.m.sb(o::COUNTERED, 0);
            e.m.sb(o::ACTION, 0);
            e.m.sl(0xA0, 0);
            e.m.sw(0xA4, 0);
            self.boss_credit(e);
            e.b.hp_prev = e.b.hp;
            if e.b.hp == 0 {
                self.flow_boss_defeated(h); // 0xACEE
            }
            if e.b.hitstop == 0 {
                e.b.hitstop = 8;
            }
            e.b.stun = 0xA;
            let cost = self.assets.enemy.t.u16(t::GUARD_COST, e.b.act as u32) as u8;
            let g = e.m.b(o::GUARD).wrapping_sub(cost);
            e.m.sb(o::GUARD, g);
            if g == 0 {
                e.b.stun = 0;
                e.m.sb(o::GUARD, 2);
            } else if e.b.act != 8 && (e.b.act == 4 || e.b.act == 6) {
                e.b.act = 0x12;
                e.b.reaction = 0;
            }
        }
        if e.b.hp == 0 {
            self.boss_clamp_camera(&mut e.b); // 0x32740
        }
        self.enemy_hit_reaction(e, h);
    }

    /// 0x63538 by `+$05` (table 0x63544): 0 the score (0x325E6, 1,000) and the leader flags (0x6148), 0x708 frames; 2 the wait (while `+$80`
    /// is 0x4B0 or more or G.$503 is set), then animation 3 and the walk to camera X + 0x120 (sound 0xA09E); 4 the walk; 6 the pose (at 0x78 the
    /// dialogue box B50 variant 6 and animation 0xF; after 0x168 the hop: animation 4, vx -1, vy 3, gravity -0x20, sound 0xA09E); 8 the hop and
    /// its bounces (the screen shake, the thud 0x9796); 0xA 180 frames of quake, blasts (0x27DAA) and debris (pool C 40, 0x9764E); 0xC the quake
    /// and the hidden body with `+$A6` 1 (for good).
    fn c84_dying(&mut self, e: &mut Enemy, _h: Handle) {
        match e.b.sub {
            0 => {
                let b = e.b.clone();
                self.enemy_award_kill_score(&b); // 0x325E6
                self.enemy_boss_leader(e.b.who); // 0x6148
                e.m.sw(0x80, 0x708);
                e.b.sub = 2;
            }
            2 => {
                let t = e.m.w(0x80);
                if t >= 0x4B0 || self.flow.script.b503 != 0 {
                    e.m.sw(0x80, t.wrapping_sub(1));
                    if t != 0 {
                        return;
                    }
                }
                self.enemy_anim(&mut e.b, 3);
                let target = (self.cam.s2.cam_x as i16).wrapping_add(0x120);
                let (vx, f) = if target >= e.b.xi() { (0x100, target as u8) } else { (-0x100, 0) };
                e.b.vx = vx;
                e.b.facing = f;
                e.b.ax = 0;
                e.b.vy = 0;
                e.b.ay = 0;
                e.b.vz = 0;
                e.b.az = 0;
                self.c24_roar(e); // 0xA09E
                e.b.sub = 4;
            }
            4 => {
                self.enemy_step(&mut e.b);
                if e.b.param_a != 0 {
                    e.b.motion(); // 0x189C
                }
                let d = (self.cam.s2.cam_x as i16).wrapping_add(0x120).wrapping_sub(e.b.xi()).wrapping_add(0x10) as u16;
                if d <= 0x20 {
                    self.enemy_anim(&mut e.b, 0x10);
                    e.b.facing = 0;
                    e.m.sw(0x80, 0);
                    e.b.sub = 6;
                }
            }
            6 => {
                if e.m.w(0x80) == 0x78 {
                    self.c84_dialogue(); // 0x8446C
                    self.enemy_anim(&mut e.b, 0xF);
                }
                let t = e.m.w(0x80).wrapping_add(1);
                e.m.sw(0x80, t);
                if t >= 0x168 {
                    self.enemy_anim(&mut e.b, 4);
                    e.b.vx = -0x100;
                    e.b.vy = 0x300;
                    e.b.ay = -0x20;
                    e.b.facing = 1;
                    e.b.airborne = 1;
                    e.m.sb(0x80, 0);
                    self.c24_roar(e); // 0xA09E
                    e.b.sub = 8;
                }
                self.enemy_step(&mut e.b);
            }
            8 => {
                e.b.motion(); // 0x189C
                self.terrain_collide_free(&mut e.b); // 0x12E20
                if e.b.airborne == 0 {
                    self.cam_shake(1); // G.$755C
                    e.b.airborne = 1;
                    self.snd_9796(e.b.xi());
                    e.b.vy = (e.b.vy.wrapping_neg()) >> 1;
                    if e.m.b(0x80) != 0 {
                        e.m.sb(0x80, 0xB4);
                        e.b.sub = 0xA;
                    } else {
                        e.m.sb(0x80, 1);
                    }
                }
            }
            0x0A => {
                let c = e.m.b(0x80);
                e.m.sb(0x80, c.wrapping_sub(1));
                if c == 0 {
                    e.b.sub = 0xC;
                    return;
                }
                let c = c.wrapping_sub(1);
                if c & 3 == 0 {
                    self.cam_shake(3);
                }
                if c & 7 == 0 {
                    let d1 = ((self.rand() & 0xF) as i16).wrapping_add(e.b.yi());
                    let d0 = ((self.rand() & 0x7F) as i16).wrapping_sub(0x40).wrapping_add(e.b.xi());
                    self.c84_blast(d0, d1, e.b.zi(), 0); // 0x27DAA
                }
                let d3 = (self.rand() & 7) as u16;
                let d1 = ((self.rand() & 0x1F) as i16).wrapping_add(e.b.yi());
                let d0 = ((self.rand() & 0x7F) as i16).wrapping_sub(0x40).wrapping_add(e.b.xi());
                self.fx40_spawn(d0, d1, e.b.zi(), d3); // 0x9764E
            }
            _ => {
                // 0x63754
                if self.w.frame & 3 == 0 {
                    self.cam_shake(3);
                }
                e.m.sb(o::HIDDEN, 1);
                e.b.visible = 0;
            }
        }
    }

    /// 0x49C0: into the draw list when visible (no on-screen test).
    pub(super) fn enemy_draw_visible(&mut self, b: &crate::game::body::Body, h: Handle) {
        if b.visible != 0 {
            self.draw_list.push(((b.zi() as u16 as u32) << 16) | h as u32);
        }
    }

    /// 0x8446C: the dialogue box, pool B class 50 variant 6 (a full pool makes none).
    fn c84_dialogue(&mut self) {
        let Some(i) = self.props.alloc() else { return };
        let p = &mut self.props.slots[i];
        p.b.live = 1;
        p.b.k20 = 50 * 4;
        p.b.var26 = 6;
    }

    /// 0x27DAA: a blast (pool E class 1, kind 4) at (x, y, z) with `+$5C` = `d3` (a full pool makes none).
    fn c84_blast(&mut self, x: i16, y: i16, z: i16, d3: u16) {
        let Some(j) = self.items.alloc() else { return };
        let n = &mut self.items.slots[j];
        n.b.live = 1;
        n.b.k20 = 4;
        let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
        put(&mut n.b.x, x);
        put(&mut n.b.y, y);
        put(&mut n.b.z, z);
        n.b.var26 = 4;
        n.m.sw(W5C, d3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game() -> Game {
        static A: std::sync::OnceLock<AssetPack> = std::sync::OnceLock::new();
        Game::new(A.get_or_init(|| {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dino.zip");
            dino_extract::extract(&path).expect("Fessenden checks require the user's dino.zip")
        }))
    }

    /// D3d 6.3: at level 10 form 2 has 640 / 700 / 800 HP by player count (kept in `+$C2`); 1,000 points for the name 0x22.
    #[test]
    fn hp_by_player_count_and_score() {
        for (players, hp) in [(0u8, 640u16), (1, 700), (2, 800)] {
            let mut g = game();
            g.w.level = 10;
            g.w.players_minus_one = players;
            let mut e = Enemy::default();
            e.b.k20 = 84 * 4;
            e.b.sub = 2;
            g.enemy_class84(&mut e, 0, 0);
            assert_eq!((e.b.hp, e.m.w(o::START_HP)), (hp, hp), "{} players", players + 1);
            assert_eq!((e.b.mode, e.m.b(ATTACKING), e.m.b(o::HEADS)), (2, 0xFF, 3));
        }
        let g = game();
        let t = &g.assets.enemy.t;
        assert_eq!(t.u32(0x10292, t.u8(0x325F8, 0x22) as u32), 0x1000);
    }
}
