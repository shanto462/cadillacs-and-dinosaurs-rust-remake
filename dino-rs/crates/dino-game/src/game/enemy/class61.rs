//! The Butcher, class 61 (handler 0x58498..0x5A47C, 8,164 bytes): the boss of Episode 2. Class 5 (`class5.rs`) rewrites its
//! class word to 61 and clears its state; the init below runs in the next frame. D3c section 3.2 to 3.4, BS09.
//!
//! Part 1 (M6-09): the init 0x584AC. Part 2 (M6-10): the alive handler 0x5861C, the checks and decisions (this file), the state
//! bodies (`class61_states.rs`), the move blocks (`class61_moves.rs`), hit reaction 0x5A33A, death 0x5A37A and the knife
//! follow tail 0x5A280. The original is one routine with branches into the middle of other states; blocks keep their addresses as names.
//!
//! Owner: enemy port.

use super::class5::ptr;
use super::lib32::put_hi;
use super::{off::*, Enemy};
use crate::game::{slot_of, Game, Handle, Slot};

/// Own bytes of the Butcher (the original offsets; the meaning is in the doc of the routine that uses them).
pub(super) mod o {
    /// `+$A1` byte: the 30 frame attack roll timer. `+$AA` byte: the timer that asks the controller for a group.
    pub const ROLL: usize = 0xA1;
    pub const GROUP: usize = 0xAA;
    /// `+$A2` byte: steps skipped in the walk states; `+$A3` byte: the 32 frame direction timer of the dash.
    pub const A2: usize = 0xA2;
    pub const A3: usize = 0xA3;
    /// `+$A4` byte: the direction before the last turn; `+$A5` byte: the X distance class of the target.
    pub const OLD_DIR: usize = 0xA4;
    pub const DIST_X: usize = 0xA5;
    /// `+$A6` / `+$A8` words: the two knives A52 (variant 0 and 0x100); `+$B4` / `+$B6` words: their copies (the knife item
    /// after it was dropped, so that the dash can pick it up).
    pub const KNIFE_A: usize = 0xA6;
    pub const KNIFE_B: usize = 0xA8;
    pub const KNIFE_A2: usize = 0xB4;
    pub const KNIFE_B2: usize = 0xB6;
    /// `+$AB` byte: the leap was chosen with the "first" flag. `+$AC` byte: a knife dropped in this hit reaction.
    pub const AB: usize = 0xAB;
    pub const FLAG_AC: usize = 0xAC;
    /// `+$AE` word: the state timer. `+$B0` byte: the slot of the ring (hop) and the slot refresh timer. `+$B8` word: the
    /// timer of the aggression toggle. `+$BA` byte: 0 = waits for the hero, 1 = presses on.
    pub const TIMER: usize = 0xAE;
    pub const B0: usize = 0xB0;
    pub const B8: usize = 0xB8;
    pub const BA: usize = 0xBA;
    /// `+$BB` / `+$BC` bytes: the combo and its steps left. `+$BE` byte: knives held (0..2). `+$BF` byte: bounces left.
    pub const BB: usize = 0xBB;
    pub const BC: usize = 0xBC;
    pub const KNIVES: usize = 0xBE;
    pub const BF: usize = 0xBF;
    /// `+$C8` word: the bar copy. `+$CA` word: never set after the init (the code behind it is dead). `+$CC` byte: the apex was passed.
    pub const BAR: usize = 0xC8;
    pub const CA: usize = 0xCA;
    pub const CC: usize = 0xCC;
    /// The tables read at run time (extracted by `dino-extract/src/boss.rs`).
    pub const TURN_TABLE: u32 = 0x5986C;
    pub const SLOT_DX: u32 = 0x32EA4;
    pub const SLOT_DZ: u32 = 0x32EB2;
}

/// The window test `addi / sub / cmpi / bhi` of the original: true when `target + add - mine` is above `limit` (unsigned word).
pub(super) fn above(target: i16, add: i16, mine: i16, limit: u16) -> bool {
    (target.wrapping_add(add).wrapping_sub(mine) as u16) > limit
}

/// The count loop of the original: `d1 = 0; do { d1 += 1; d0 -= step } while d0 >= 0` (a speed from a distance).
pub(super) fn c61_count(mut d0: i32, step: i32) -> u16 {
    let mut d1 = 0u16;
    loop {
        d1 = d1.wrapping_add(1);
        d0 = d0.wrapping_sub(step);
        if d0 < 0 {
            return d1;
        }
    }
}

impl Game {
    /// 0x58498: class 61 (the mode table at 0x584A4): 0 the init (0x584AC, which ends in the first decision), 2 the alive handler
    /// (0x5861C), 4 the death (0x5A37A), 6 the body that stays (0x5A3CA).
    pub(super) fn enemy_class61(&mut self, e: &mut Enemy, h: Handle, _i: usize) {
        match e.b.mode {
            0 => self.c61_init(e, h),
            2 => self.c61_alive(e, h),
            4 => self.c61_dying(e, h),
            _ => self.enemy_draw_check(&mut e.b, h), // 0x5A3CA: jmp 0x49CA
        }
    }

    /// 0x584AC: the init. Name 0xF, the table 0x6FDEC, HP row 1, a target, the two knives A52, the HUD bar, the boss
    /// controller and the boss music; then the first decision (0x58D40). The sound 0x0F is made in the middle of
    /// the init, so it comes before the decision's own random numbers.
    fn c61_init(&mut self, e: &mut Enemy, h: Handle) {
        e.b.live = 1;
        e.m.sb(B87, 0);
        e.b.id = 0x0F;
        e.m.sw(W84, 0x3C);
        e.b.anim_table = 0x6FDEC;
        self.enemy_anim(&mut e.b, 1);
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_set_speed(&mut e.b, 7); // 0x17A4, with the direction the slot still has
        e.b.dir = 0x18;
        self.enemy_set_boss_hp(&mut e.b, 1); // 0x31E14 row 1
        let target = self.c61_pick_target(e);
        e.m.sw(TARGET, target);
        e.b.stun = 0x0B;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.m.sb(ATTACKING, 0xFF);
        e.b.b72 = 0x1E;
        e.m.sw(o::TIMER, 0x14);
        e.m.sb(o::KNIVES, 0);
        e.m.sb(o::FLAG_AC, 0);
        e.m.sb(o::BA, 0);
        e.m.sw(o::B8, 0xC8);
        e.m.sb(o::B0, 0);
        e.m.sb(o::A2, 0);
        e.m.sb(o::BF, 0);
        e.m.sb(o::ROLL, 0x1E);
        e.m.sb(o::GROUP, 0x1E);
        e.m.sw(o::CA, 0);
        self.c61_make_knives(e, h);
        self.hud.boss = h; // G.-$70BC
        self.spawn.g.b4fc = 0; // the boss intro is over (the area script waits for it)
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sb(ATTACKING, 0xFF);
        e.b.b63 = 2;
        // 0x58604: 0x7BEDC writes the controller's +$AA (the sign extended boss); the init does not write it again.
        self.boss_spawn_controller(h);
        e.m.sw(o::BAR, e.b.hp_bar);
        self.sound(0x0F);
        self.c61_decide(e, h);
    }

    /// 0x58562 .. 0x585E6: the two knives A52 (variants 0 and 0x100) on the Butcher's integer position, each with the
    /// table 0x6F8A6 and the owner in `+$A0`. The Butcher keeps their handles in `+$A6` / `+$B4` and `+$A8` / `+$B6` and
    /// the count in `+$BE`. A full pool ends the list (`bcs`), the count then stays 0 or 1.
    fn c61_make_knives(&mut self, e: &mut Enemy, h: Handle) {
        let (x, y, z) = (e.b.xi(), e.b.yi(), e.b.zi());
        for (n, (var, knife, copy)) in [(0u16, o::KNIFE_A, o::KNIFE_A2), (0x100, o::KNIFE_B, o::KNIFE_B2)].into_iter().enumerate() {
            let Some(a) = self.enemies.alloc() else { return };
            let k = &mut self.enemies.slots[a];
            k.b.live = 1;
            k.b.k20 = 0xD0;
            k.b.var26 = var;
            put_hi(&mut k.b.x, x);
            put_hi(&mut k.b.y, y);
            put_hi(&mut k.b.z, z);
            k.b.anim_table = 0x6F8A6;
            k.m.sl(0xA0, ptr(h));
            let handle = self.enemies.handle(a);
            e.m.sw(knife, handle);
            e.m.sb(o::KNIVES, n as u8 + 1);
            e.m.sw(copy, handle);
        }
    }

    /// 0x58CD6: the Butcher's own target choice. A roll `rand() & 15`: 10 or more takes the common choice 0x32032, below 10
    /// the hero with the least HP that has less than 0x70 (the first one on a tie; with nobody below 0x70 hero 1, even when
    /// he is gone).
    fn c61_pick_target(&mut self, e: &Enemy) -> Handle {
        if self.rand() & 0xF >= 0xA {
            return self.enemy_pick_target(&e.b);
        }
        let mut best = Game::player_handle(0);
        let mut least = 0x70i16;
        for p in 0..3 {
            let b = &self.players[p].b;
            if b.live != 0 && (b.hp as i16) < least {
                least = b.hp as i16;
                best = Game::player_handle(p);
            }
        }
        best
    }

    /// 0x58D40: the decision, at the end of the init and as the get-up entry of the table 0x2B56C (name 0x3C): the state long
    /// becomes 0x02000400, animation 0, a target (0x58CD6), speed 3 and a slot round the target (0x32E0A). After a knock
    /// down (`+$7B` = 4) he turns to the hero and, when he is within reach, bounces on him (0x59B50, `+$BF` = 1; the roll
    /// is `rand() & 15 < 8`), else presses on (`+$BA` = 1, animation 0, speed 3, G.$502 = 1). Then the common decision 0x5A186.
    pub(super) fn c61_decide(&mut self, e: &mut Enemy, _h: Handle) {
        e.b.set_state(0x0200_0400);
        self.enemy_anim(&mut e.b, 0);
        let target = self.c61_pick_target(e);
        e.m.sw(TARGET, target);
        self.enemy_set_speed(&mut e.b, 3);
        self.c61_slot_new(e); // 0x32E0A
        if e.b.reaction == 4 {
            self.lib_face_target(e); // 0x58D72
            let t = self.c61_tgt(e);
            if !above(t.x, 0x50, e.b.xi(), 0x50) && !above(t.z, 8, e.b.zi(), 0x10) && self.rand() & 0xF < 8 {
                e.m.sb(o::BF, 1);
                return self.c61_59b50(e);
            }
            e.m.sb(ATTACKING, 0xFF); // 0x58DC0
            e.m.sw(o::TIMER, 0x14);
            e.m.sb(0xA0, 0);
            e.m.sb(o::BA, 1);
            e.m.sw(o::B8, 0xC8);
            self.enemy_anim(&mut e.b, 0);
            self.enemy_set_speed(&mut e.b, 3);
            self.lib_aim_at_target(e); // 0x32A58
            self.lib_face_target(e);
            self.spawn.g.b502 = 1;
        }
        self.c61_5a186(e);
    }

    /// 0x32E00: the point of the slot kept in `+$BD`; a new slot when it is 0 (0x32E0A).
    pub(super) fn c61_slot_keep(&mut self, e: &mut Enemy) {
        match e.m.b(super::lib32::lib::SLOT_NO) {
            0 => self.c61_slot_new(e),
            slot => self.c61_slot_point(e, slot as u32),
        }
    }

    /// 0x32E0A: a slot 1..6 round the target for the Butcher: `rand() & 7` (0 and 7 are drawn again), then its point.
    pub(super) fn c61_slot_new(&mut self, e: &mut Enemy) {
        let slot = loop {
            let d = (self.rand() & 7) as u32;
            if d != 0 && d != 7 {
                break d;
            }
        };
        e.m.sb(super::lib32::lib::SLOT_NO, slot as u8);
        self.c61_slot_point(e, slot);
    }

    /// 0x32E44: the point of slot `slot`: the target plus the offsets of the tables 0x32EA4 (X) and 0x32EB2 (Z), mirrored to the
    /// object's side (`lib_slot_mirror`).
    fn c61_slot_point(&mut self, e: &mut Enemy, slot: u32) {
        let t = &self.assets.enemy.t;
        let (dx, dz) = (t.i16(o::SLOT_DX, 2 * slot), t.i16(o::SLOT_DZ, 2 * slot));
        let (tx, tz) = {
            let b = self.lib_target(e);
            (b.xi(), b.zi())
        };
        e.m.sw(DEST_X, tx.wrapping_add(dx) as u16);
        e.m.sw(DEST_Z, tz.wrapping_add(dz) as u16);
        self.lib_slot_mirror(e, dx);
    }

    /// 0x59822: the state 6, "stand and wait" (D3c 3.3). The direction is kept in `+$A4`, a point beside the target is
    /// chosen from the table 0x5986C by the opposite direction and the Butcher turns toward it (0x32AC4), by at most two
    /// steps (0x597DC); `+$AE` = 0xF0 frames; the animation 1 starts unless the state was 6 already.
    pub(super) fn c61_state_wait(&mut self, e: &mut Enemy) {
        e.m.sb(o::OLD_DIR, e.b.dir);
        let d0 = (e.b.dir as i8).wrapping_neg() as u8 as u32 & 0x1E;
        let t = &self.assets.enemy.t;
        let (d1, d2) = (t.i16(o::TURN_TABLE, d0), t.i16(o::TURN_TABLE, d0 + 2));
        self.c61_aim_beside(e, d1, d2); // 0x32AC4
        self.c61_limit_turn(e); // 0x597DC
        e.m.sb(o::A3, 6);
        e.m.sw(o::TIMER, 0xF0);
        if e.b.act != 6 {
            self.enemy_anim(&mut e.b, 1);
        }
        e.b.act = 6;
    }

    /// 0x32AC4: the point (target X + `d1`, target Z + `d2`) into `+$88` / `+$8A`, the direction to it into `+$22`.
    pub(super) fn c61_aim_beside(&mut self, e: &mut Enemy, d1: i16, d2: i16) {
        let (tx, tz) = {
            let b = self.lib_target(e);
            (b.xi(), b.zi())
        };
        let (x, z) = (tx.wrapping_add(d1) as u16, tz.wrapping_add(d2) as u16);
        e.m.sw(DEST_X, x);
        e.m.sw(DEST_Z, z);
        e.b.dir = Game::enemy_angle_to(&e.b, x, z);
    }

    /// 0x597DC: the turn limit. The direction `+$22` may differ from the old one `+$A4` by two steps at most; a bigger
    /// turn is cut to the old direction plus or minus 2 (0 .. 31).
    pub(super) fn c61_limit_turn(&mut self, e: &mut Enemy) {
        let old = e.m.b(o::OLD_DIR);
        let d = (e.b.dir.wrapping_sub(old) & 0x1F).wrapping_add(2) & 0x1F;
        if (d as i8) <= 4 {
            return;
        }
        e.b.dir = if (d.wrapping_sub(2) as i8) > 0x10 { old.wrapping_sub(2) & 0x1F } else { old.wrapping_add(2) & 0x1F };
    }
}

impl Game {
    /// 0x5861C: mode 2. The X clamp (camera X - 0x50 .. + 0x1D0, and with no HP left - 0x10 .. + 0x190), then the sub state
    /// by `+$05` (0x586C6: 0 alive, 2 hit reaction, 4 holding a hero, 6 held by a hero), the group timer (`+$AA` every 30
    /// frames asks the boss controller for the next group by G.$502), the knife drop roll and the knife follow tail.
    fn c61_alive(&mut self, e: &mut Enemy, h: Handle) {
        let cam = self.cam.s2.cam_x as i16;
        let lo = cam.wrapping_sub(0x50);
        if e.b.xi().wrapping_sub(lo) < 0 {
            e.b.set_xi_keep(lo);
        }
        let hi = cam.wrapping_add(0x1D0);
        if (hi as i32) - (e.b.xi() as i32) < 0 {
            e.b.set_xi_keep(hi);
        }
        if e.b.hp == 0 {
            let lo = cam.wrapping_sub(0x10);
            if e.b.xi().wrapping_sub(lo) < 0 {
                e.b.set_xi_keep(lo);
            }
            let hi = cam.wrapping_add(0x190);
            if (hi as i32) - (e.b.xi() as i32) < 0 {
                e.b.set_xi_keep(hi);
            }
        }
        match e.b.sub {
            0 => self.c61_sub0(e, h),
            2 => self.c61_hit(e, h),
            4 => self.c61_sub4(e, h),
            6 => self.enemy_grabbed(e, h), // 0x2B35A
            _ => {}
        }
        let g = e.m.b(o::GROUP).wrapping_sub(1);
        e.m.sb(o::GROUP, g);
        if g == 0 {
            e.m.sb(o::GROUP, 0x1E);
            self.spawn.g.b502 = 1;
        }
        self.c61_knife_drop(e); // 0x586D6
        self.c61_tail_knives(e); // 0x5A280
    }

    /// 0x5882A: the alive state. The knife pick up check, the poise timer (`+$79`: at 0 the poise `+$78` is 11 again), the floor,
    /// then either the hit stop (`+$59` counts down, the frame timer is held at 1) or the wait `+$7A`, or the real work: face the hero
    /// (when `+$A0` is 0), the decisions (when `+$BA` is 0 the proximity leap and the slash check), the slot point, the state, the
    /// leap roll, the far check and the slot refresh. Always the timer toggle 0x58916, the draw check and the hit list: the
    /// hostile list 0x4C0A in the air states 0x1A and 0x2A, else the hurtable list 0x4BEA.
    fn c61_sub0(&mut self, e: &mut Enemy, h: Handle) {
        self.c61_knife_pickup(e);
        let poise = e.m.b(POISE_T);
        if poise != 0 {
            e.m.sb(POISE_T, poise - 1);
            if poise - 1 == 0 {
                e.b.stun = 0xB;
            }
        }
        self.enemy_floor_collision(e); // 0x12CB4
        if e.b.hitstop != 0 {
            e.b.hitstop -= 1;
            e.b.timer = 1;
        } else if e.m.b(WAIT) != 0 {
            e.m.sb(WAIT, e.m.b(WAIT) - 1);
        } else {
            // 0x58C64 is never active (it returns at once in every case).
            if e.m.b(0xA0) == 0 {
                self.lib_face_target(e);
            }
            if e.m.b(o::BA) == 0 {
                self.c61_check_proximity(e);
                self.c61_check_slash(e);
            }
            self.c61_slot_keep(e); // 0x32E00
            // 0x32E44 ends with d1 = X + 0x400 (`move.w $8(a6),d1 ; addi.w #$400,d1`, d1 is not used after the compare): the d1 that the decision roll 0x5A1AC
            // keeps in its high byte when no state, frame load or leap roll of this frame wrote d1 after it.
            e.b.frame_w1 = e.b.xi().wrapping_add(0x400) as u16;
            self.c61_state(e, h);
            self.c61_leap_roll(e);
            self.c61_check_far(e);
            self.c61_slot_refresh(e);
        }
        self.c61_toggle(e);
        self.enemy_draw_check(&mut e.b, h);
        if e.b.act == 0x1A || e.b.act == 0x2A {
            self.enemy_monster_register(h);
        } else {
            self.enemy_push_hurtable(h);
        }
    }

    /// 0x58916: the aggression toggle. When `+$B8` runs out: `+$AE` = 1 (the state ends), `+$BA` flips, and the next `+$B8` comes from the
    /// table 0x74502 by a random number and `+$BA`. The branch with `+$CA` (a state 0x30 for a Butcher off screen) is never taken:
    /// `+$CA` is 0. When he turns to pressing on while he is busy (`+$A0`), he takes a new target and the high byte of `+$B8` is 1.
    fn c61_toggle(&mut self, e: &mut Enemy) {
        let t = e.m.w(o::B8).wrapping_sub(1);
        e.m.sw(o::B8, t);
        if t != 0 {
            return;
        }
        e.m.sw(o::TIMER, 1);
        e.m.sb(o::BA, (e.m.b(o::BA) + 1) & 1);
        let d0 = (self.rand() & 0x1E) as u32 + ((e.m.b(o::BA) as u32) << 4);
        e.m.sw(o::B8, self.assets.enemy.t.u16(0x74502, d0));
        if e.m.b(0xA0) == 0 || e.m.b(o::BA) != 0 {
            return;
        }
        e.m.sb(o::BA, 1);
        e.m.sb(o::B8, 1);
        let target = self.c61_pick_target(e);
        e.m.sw(TARGET, target);
    }

    /// 0x589C0: every 120 frames a roll: 14 in 16 take a new slot round the hero (0x32E0A) and, when `+$A0` is 0, face him.
    fn c61_slot_refresh(&mut self, e: &mut Enemy) {
        if e.m.b(o::B0) == 0 {
            e.m.sb(o::B0, 0x78);
            if self.rand() & 0xF < 0xE {
                self.c61_slot_new(e);
                if e.m.b(0xA0) == 0 {
                    self.lib_face_target(e);
                }
            }
        }
        e.m.sb(o::B0, e.m.b(o::B0).wrapping_sub(1));
    }

    /// 0x58A76 (with `+$BA` = 0): a hero within 0x40 left .. 0x40 right and 0x10 each way in depth, when `+$A0` and `+$A2` are 0:
    /// the proximity leap.
    fn c61_check_proximity(&mut self, e: &mut Enemy) {
        if e.m.b(0xA0) != 0 || e.m.b(o::A2) != 0 {
            return;
        }
        let t = self.c61_tgt(e);
        if above(t.x, 0x40, e.b.xi(), 0x80) || above(t.z, 0x10, e.b.zi(), 0x20) {
            return;
        }
        self.c61_599c8(e);
    }

    /// 0x58AB8: every 30 frames (`+$A1`), when pressing on (`+$BA`) and free: a roll `rand() & 31` against 8 (2 when his HP is below a
    /// quarter of the bar, the words of 0x58B0C): the leap.
    fn c61_leap_roll(&mut self, e: &mut Enemy) {
        let a1 = e.m.b(o::ROLL).wrapping_sub(1);
        e.m.sb(o::ROLL, a1);
        if a1 != 0 {
            return;
        }
        e.m.sb(o::ROLL, 0x1E);
        if e.m.b(0xA0) != 0 || e.m.b(o::A2) != 0 || e.m.b(o::BA) == 0 {
            return;
        }
        let quarter = (e.m.w(o::BAR) as i16) >> 2;
        let low = (e.b.hp as i16).wrapping_sub(quarter) < 0;
        // `sub.w d0,d1` leaves d1 = HP - a quarter of the bar (the rand routine 0x119C keeps d1): the stale d1 of the decision roll 0x5A1AC that follows in
        // the same frame (`move.b (a0),d1` only replaces its low byte), whatever frame record this frame loaded before.
        e.b.frame_w1 = (e.b.hp as i16).wrapping_sub(quarter) as u16;
        let limit = self.assets.enemy.t.u16(0x58B0C, if low { 0 } else { 2 });
        if (self.rand() & 0x1F) as i16 > limit as i16 {
            return;
        }
        self.c61_59b24(e);
    }

    /// 0x58B14 (with `+$BA` = 0): a hero within 0x40 / 0x40 in X and 0xA in depth, when `+$A2` is not 0: the decision.
    fn c61_check_slash(&mut self, e: &mut Enemy) {
        if e.m.b(0xA0) != 0 {
            return;
        }
        let t = self.c61_tgt(e);
        if above(t.x, 0x40, e.b.xi(), 0x80) || above(t.z, 0xA, e.b.zi(), 0xB) {
            return;
        }
        if e.m.b(o::A2) != 0 {
            self.c61_5a1ea(e);
        }
    }

    /// 0x58B54 (with `+$BA` = 1): when the hero is near in depth: the decision at once (X class 2 or less, depth within 8) or after a
    /// roll (0x5A1AC, depth within 16).
    fn c61_check_far(&mut self, e: &mut Enemy) {
        if e.m.b(0xA0) != 0 || e.m.b(o::BA) == 0 {
            return;
        }
        let t = self.c61_tgt(e);
        if (e.m.b(o::DIST_X) as i8) > 2 {
            if !above(t.z, 8, e.b.zi(), 0x10) {
                self.c61_5a1ac(e);
            }
        } else if !above(t.z, 8, e.b.zi(), 8) {
            self.c61_5a1ea(e);
        }
    }

    /// 0x5A1AC: a roll for the decision: the hero must be alive (`+$00` = 1) and not in a hit reaction (state word 0x0202); the chance
    /// is the byte of the table 0x73E94 by the aggression G.$5EC and the X class, against `rand() & 31`.
    fn c61_5a1ac(&mut self, e: &mut Enemy) {
        let b = self.lib_target(e);
        if b.live != 1 || (b.mode, b.sub) == (2, 2) {
            return;
        }
        let d0 = self.rank.aggression << 3;
        let d0 = (d0 & 0xFF00) | (d0 as u8).wrapping_add(e.m.b(o::DIST_X)) as u16;
        // `move.b (a0),d1` keeps the high byte of d1: a frame record read in this frame (0x1236: d1 = its second word, 0x8000 for the walk frames) makes
        // the word negative, so the roll `blt` cannot succeed in that frame.
        let d1 = ((e.b.frame_w1 & 0xFF00) | self.assets.enemy.t.u8(0x73E94, d0 as u32) as u16) as i16;
        if ((self.rand() & 0x1F) as i16) < d1 {
            self.c61_5a1ea(e);
        }
    }

    /// 0x5A1EA: the decision. A random row (32 rows of 8) and the X class pick a byte of the table 0x73ED4 (no knife), 0x73FD4 (one
    /// knife) or 0x740D4 (two): 1 to 4 a combo (0x59F0C), 5 the spin (0x5A014), 6 and 7 the leap (0x59B24), 8 the first leap (0x59B1E), 9 the dash
    /// (0x5A04E, the word after the table 0x5A26C proper).
    fn c61_5a1ea(&mut self, e: &mut Enemy) {
        self.enemy_set_speed(&mut e.b, 5);
        e.m.sb(0xA0, 0);
        let (dx, _dz) = self.lib_distance_class(e);
        e.m.sb(o::DIST_X, dx as u8);
        e.m.sb(o::AB, 0);
        let d0 = (((self.rand() & 0x1F) << 3) + e.m.b(o::DIST_X) as u16) as u32;
        let table = match e.m.b(o::KNIVES) {
            0 => 0x73ED4,
            1 => 0x73FD4,
            _ => 0x740D4,
        };
        let mut d1 = self.assets.enemy.t.u8(table, d0);
        if d1 == 7 {
            d1 = 6;
        }
        match d1 {
            1..=4 => self.c61_59f0c(e, d1 as u16),
            5 => self.c61_5a014(e),
            6 => self.c61_59b24(e),
            8 => self.c61_59b1e(e),
            9 => self.c61_5a04e(e), // the last word of the jump table 0x5A26C
            _ => unreachable!("decision {d1}: the table 0x5A26C has no code for it"),
        }
    }

    /// 0x5A33A: sub 2, the hit reaction. A random number is drawn in every frame (the anti tamper check 0x29ADE behind it always
    /// passes). The damage credit (0x3262A); at HP 0 with `+$6E` set the defeat (0xACEE) once; then the shared reaction 0x2A606.
    fn c61_hit(&mut self, e: &mut Enemy, h: Handle) {
        let _ = self.rand();
        self.c61_credit(e);
        if e.b.hp == 0 && e.b.hp_prev != 0 {
            self.flow_boss_defeated(h); // 0xACEE
            e.b.hp_prev = 0;
        }
        self.enemy_hit_reaction(e, h);
    }

    /// 0x3262A: the damage dealt (`+$6E` - `+$6C`) is added to the attacker's best player counter `+$10C` (all three heroes in the car
    /// drive); with HP left `+$6E` follows. A negative `+$02` (nobody) skips it.
    fn c61_credit(&mut self, e: &mut Enemy) {
        if (e.b.who as i8) < 0 {
            return;
        }
        let dmg = e.b.hp_prev.wrapping_sub(e.b.hp);
        if self.spawn.g.car_mode != 0 {
            for p in self.players.iter_mut() {
                p.w10c = p.w10c.wrapping_add(dmg);
            }
        } else {
            let p = &mut self.players[e.b.who as usize % 3];
            p.w10c = p.w10c.wrapping_add(dmg);
        }
        if e.b.hp != 0 {
            e.b.hp_prev = e.b.hp;
        }
    }

    /// 0x5A37A: mode 4, the death. Step 0: the music fade (0xA5EA), the game speed back to 1 (G.$4D6), 120 frames in `+$80`, the kill
    /// score (0x325E6, the killer in `+$02`), and on into step 2: count down (the draw check each frame), at the end the leader flags
    /// (0x6148) and mode 6.
    fn c61_dying(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.sub == 0 {
            self.hook_a5ea();
            self.flow.speed = 1;
            e.m.sw(0x80, 0x78);
            let b = e.b.clone();
            self.enemy_award_kill_score(&b);
            e.b.sub = e.b.sub.wrapping_add(2);
        }
        let t = e.m.w(0x80);
        if t == 0 {
            self.enemy_boss_leader(e.b.who);
            e.b.mode = 6;
        } else {
            e.m.sw(0x80, t - 1);
        }
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x6148: after a boss dies: the leader flags `+$C0` of the three heroes are cleared, then the one of the killer set (all three
    /// in the car drive). A killer that is no hero (the table has three entries) sets none.
    pub fn enemy_boss_leader(&mut self, killer: u8) {
        for p in self.players.iter_mut() {
            p.bc0 = 0;
        }
        if self.spawn.g.car_mode == 0 {
            if let Some(p) = self.players.get_mut(killer as usize).filter(|_| killer < 3) {
                p.bc0 = 1;
            }
        } else {
            for p in self.players.iter_mut() {
                p.bc0 = 1;
            }
        }
    }

    // ------------------------------------------------------------------ the knives

    /// 0x586D6: the knife drop. With a knife held and in the hit reaction (state 0x0202) of the type 4 or 6 (knocked down): a roll
    /// `rand() & 15 < 8` and no drop yet in this reaction (`+$AC`): one knife less, the second made goes first, its object gets `+$05` = 8
    /// (it falls and becomes the item). Any other state clears `+$AC`.
    fn c61_knife_drop(&mut self, e: &mut Enemy) {
        let kn = e.m.b(o::KNIVES) as i8;
        if kn <= 0 {
            return;
        }
        if (e.b.mode, e.b.sub) != (2, 2) {
            e.m.sb(o::FLAG_AC, 0);
            return;
        }
        if e.b.act != 4 && e.b.act != 6 {
            return;
        }
        if self.rand() & 0xF >= 8 || e.m.b(o::FLAG_AC) != 0 {
            return;
        }
        let left = e.m.b(o::KNIVES) - 1;
        e.m.sb(o::KNIVES, left);
        let knife = if left == 0 {
            let k = e.m.w(o::KNIFE_A);
            e.m.sw(o::KNIFE_A2, k);
            k
        } else {
            let k = e.m.w(o::KNIFE_B);
            e.m.sw(o::KNIFE_B2, k);
            k
        };
        e.m.sb(o::FLAG_AC, 1);
        if let Some(Slot::Enemy(i)) = slot_of(knife) {
            self.enemies.slots[i].b.sub = 8;
        }
    }

    /// 0x58752: while he dashes (act 0x12) and holds fewer than two knives: the dropped knife he runs for (`+$B4`, or `+$B6` with one
    /// knife) lies idle (state word 0x0200) at his height within 0x20 / 0x10 pixels: he picks it up (act 0x10, `+$A0` = 1, animation 5).
    pub(super) fn c61_knife_pickup(&mut self, e: &mut Enemy) {
        if e.m.b(o::KNIVES) >= 2 || e.b.act != 0x12 {
            return;
        }
        let knife = if e.m.b(o::KNIVES) == 0 { e.m.w(o::KNIFE_A2) } else { e.m.w(o::KNIFE_B2) };
        let b = match slot_of(knife) {
            Some(Slot::Item(i)) => &self.items.slots[i].b,
            Some(Slot::Enemy(i)) => &self.enemies.slots[i].b,
            _ => return,
        };
        if (b.mode, b.sub) != (2, 0) {
            return;
        }
        if above(b.xi(), 0x20, e.b.xi(), 0x40) || above(b.zi(), 0x10, e.b.zi(), 0x20) || b.yi() != e.b.yi() {
            return;
        }
        e.m.sb(0xA0, 1);
        e.b.act = 0x10;
        self.enemy_anim(&mut e.b, 5);
    }

    /// 0x5A280: the knives follow his hand: for each held knife the position by his frame byte `+$2D` (0x5A2A4).
    pub(super) fn c61_tail_knives(&mut self, e: &mut Enemy) {
        let kn = e.m.b(o::KNIVES);
        if kn == 0 {
            return;
        }
        self.c61_knife_follow(e, e.m.w(o::KNIFE_A));
        if kn != 1 {
            self.c61_knife_follow(e, e.m.w(o::KNIFE_B));
        }
    }

    /// 0x5A2A4: one knife (`a1`). The offsets are rows of 8 bytes of the table 0x57480 (variant 0) or 0x57558 (the other) by his
    /// frame byte (0x63 and 0x5A count as 0x15): dx (mirrored with his facing), dy, dz and the knife's picture; 0x5A hides it. The
    /// picture is started when the frame byte changed (`+$A4` of the knife), with his facing.
    fn c61_knife_follow(&mut self, e: &Enemy, knife: Handle) {
        let Some(Slot::Enemy(k)) = slot_of(knife) else { return };
        let frame = e.b.param_b;
        let row = if frame == 0x63 || frame == 0x5A { 0x15 } else { frame } as u32;
        let base = if self.enemies.slots[k].b.var26 >> 8 == 0 { 0x57480 } else { 0x57558 };
        let t = &self.assets.enemy.t;
        let (mut dx, dy, dz, pic) = (t.i16(base, 8 * row), t.i16(base, 8 * row + 2), t.i16(base, 8 * row + 4), t.u16(base, 8 * row + 6));
        if e.b.facing != 0 {
            dx = dx.wrapping_neg();
        }
        let (x, y, z, facing) = (e.b.xi(), e.b.yi(), e.b.zi(), e.b.facing);
        let kn = &mut self.enemies.slots[k];
        kn.b.set_xi_keep(x.wrapping_add(dx));
        kn.b.set_yi_keep(y.wrapping_add(dy));
        kn.b.set_zi_keep(z.wrapping_add(dz));
        kn.m.sw(0xA6, pic);
        if pic == 0x5A || kn.m.b(0xA4) == frame {
            return;
        }
        kn.m.sb(0xA4, frame);
        kn.b.facing = facing;
        let kb = &mut kn.b;
        let table = kb.anim_table;
        debug_assert_eq!(table, 0x6F8A6);
        kb.start_anim(&self.assets.sprites, pic); // 0x120E on the knife
    }

    /// 0x32AE4: the next point of the ring of six round the hero for a hop: the slot `slot` (a negative one or none draws one) moves one
    /// step either way (`rand() & 1`), back to 0..5 (a 6 or 7 draws again), and the offsets (the table 0x32B48, read at byte offset
    /// 2 * slot as the original does) give the point in `+$88` / `+$8A` and the direction in `+$22`. Returns the slot (d3).
    pub(super) fn lib_slot_ring(&mut self, e: &mut Enemy, slot: u16) -> u16 {
        let mut d1 = slot;
        if (d1 as i16) < 0 {
            d1 = self.rand() & 7;
        }
        loop {
            if self.rand() & 1 != 0 {
                d1 = d1.wrapping_add(1);
            } else {
                d1 = d1.wrapping_sub(1);
            }
            d1 &= 7;
            if (d1 as i16) >= 6 {
                d1 = self.rand() & 7;
                continue;
            }
            break;
        }
        let t = &self.assets.enemy.t;
        let (dx, dz) = (t.i16(0x32B48, 2 * d1 as u32), t.i16(0x32B48, 2 * d1 as u32 + 2));
        let (tx, tz) = {
            let b = self.lib_target(e);
            (b.xi(), b.zi())
        };
        let (x, z) = (tx.wrapping_add(dx) as u16, tz.wrapping_add(dz) as u16);
        e.m.sw(DEST_X, x);
        e.m.sw(DEST_Z, z);
        e.b.dir = Game::enemy_angle_to(&e.b, x, z);
        d1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turned(old: u8, new: u8) -> u8 {
        let mut g = Game::new(Box::leak(Box::new(dino_assets::AssetPack::default())));
        let mut e = Enemy::default();
        e.m.sb(o::OLD_DIR, old);
        e.b.dir = new;
        g.c61_limit_turn(&mut e);
        e.b.dir
    }

    /// 0x597DC: a turn of two steps or less is kept; a bigger one is cut to the old direction plus or minus 2 (mod 32).
    #[test]
    fn the_turn_is_limited_to_two_steps() {
        assert_eq!(turned(0, 1), 1);
        assert_eq!(turned(0, 2), 2);
        assert_eq!(turned(0x10, 0x0E), 0x0E);
        assert_eq!(turned(0, 5), 2, "left of the target: plus 2");
        assert_eq!(turned(0, 0x1B), 0x1E, "right of the target: minus 2, wrapped");
        assert_eq!(turned(0x1F, 0x03), 0x01);
        assert_eq!(turned(0x18, 0x00), 0x1A, "8 steps to the left of the old direction");
    }

    /// The pointer fields the class 5 and 61 inits store (`move.l a6,..` after `movea.w`) have 0xFFFF in the high word.
    #[test]
    fn owner_pointers_are_sign_extended() {
        assert_eq!(ptr(0xDC34), 0xFFFF_DC34);
    }
}
