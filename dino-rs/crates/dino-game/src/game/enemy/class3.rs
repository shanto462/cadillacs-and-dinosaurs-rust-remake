//! LASH T., class 3 (+$20 = 0xC, handler 0x358B2..0x36A0A, 4,530 bytes): the mace fighter, the mid boss of Episode 2. D3a section 6, H19, H10.
//! One stage record makes THREE objects (0x358CE): the lowest address of the three free pool A slots is the Lash, the middle one the mace (class 62, `class62.rs`),
//! the highest the chain piece (class 8, `class62.rs`). The Lash has the mace in +$A6 (a word); they have him in +$A0 (a long).
//!
//! +$04 mode: 0 (sub 0 the three objects, sub 2 the init 0x35998), 2 alive (sub 0 the AI 0x35A80, 2 the hit reaction 0x2A606, 4 the death 0x369D0, 6 held
//! 0x2B35A), 4 death (0x369D0), 6 free. The AI states (+$06, table 0x35B24): 0 drift to the slot, 2 wait, 4 the swing, 6 walk to the attack point,
//! 8 walk in X, 0xA walk in Z, 0x10 the end of a swing, 0x12 run back onto the screen, 0x14 side steps, 0x18 airborne, 0x1A the next swing of a series.
//! Dead code (no caller): 0xC / 0xE (0x362EC, 0x36394, the walk that copies the hero's input), 0x16 (0x35F5A with its entry 0x366CC), 0x36926, 0x3699C.
//!
//! The decisions (0x3680E first, 0x368C2 attack): the table 0x72754 (or 0x72854 without the mace, +$BE) by `(rand & 0x1F) * 8 + X class` gives 1 / 3 state 6,
//! 2 state 0, 4 state 8, 5 state 0xA, 6 "mace gone"; the attack gate 0x72954 and the attack choice 0x72994 give a swing by 0x36740 (rows 0x367CC: +$BB, +$BC
//! the series and its length, the animation). Own bytes: +$A0 busy, +$A1 / +$A2 the step pause, +$A3 / +$A4 the aim timer and the old direction, +$A5 the X
//! class, +$A9 the slot timer, +$AD the walk flag, +$AE the state timer (word), +$B0 / +$B4 the step longs, +$B8 / +$B9 the attack delay, +$BB / +$BC the
//! swing and its repeats, +$BD the slot, +$BE no mace, +$BF the mace's direction flag, +$C0 the flee timer (word), +$C2 the flee flag, +$C4 a frame flag of the chain.
//!
//! Owner: enemy port (M6-07 / M6-08).

use super::{off::*, Enemy};
use crate::game::{slot_of, Game, Handle, Slot};

/// Tables read from the ROM at run time (`dino-extract` TABLES).
mod t {
    /// 0x35BDE: the flee timer, positive phase (16 words); 0x35BFE: the negative phase (16 words).
    pub const FLEE_POS: u32 = 0x35BDE;
    pub const FLEE_NEG: u32 = 0x35BFE;
    /// 0x7431A: the step pause by level (+ the variant byte >> 5); 0x7433A: the same for the repeat pause.
    pub const PAUSE: u32 = 0x7431A;
    /// 0x72754 / 0x72854: the first decision without and with the mace gone; 0x72954 the attack gate; 0x72994 the attack choice.
    pub const DECIDE: u32 = 0x72754;
    pub const DECIDE_NO_MACE: u32 = 0x72854;
    pub const GATE: u32 = 0x72954;
    pub const ATTACK: u32 = 0x72994;
    /// 0x367CC: rows of 4 (+$BB, +$BC, the animation, 0) by the attack value - 1; 0x36222: the animation by +$BB * 4 + +$BC of a series.
    pub const SWINGS: u32 = 0x367CC;
    pub const SERIES: u32 = 0x36222;
    /// 0x74DA2 (by `rand & 0x3E`) and 0x74E62 (by the level) give the length of the state 0x1A.
    pub const REPEAT_A: u32 = 0x74DA2;
    pub const REPEAT_B: u32 = 0x74E62;
    /// 0x32F32: the slot offsets of 0x32EC0 (words by the slot number).
    pub const SLOT_OFFSETS: u32 = 0x32F32;
}

const BUSY: usize = 0xA0;
const STEP_FLAG: usize = 0xA1;
const STEP_PAUSE: usize = 0xA2;
const AIM_T: usize = 0xA3;
const OLD_DIR: usize = 0xA4;
const XCLASS: usize = 0xA5;
const MACE: usize = 0xA6;
const SLOT_T: usize = 0xA9;
const WALK: usize = 0xAD;
const TIMER: usize = 0xAE;
const VX: usize = 0xB0;
const VZ: usize = 0xB4;
const ATK_DELAY: usize = 0xB8;
const ATK_FLAG: usize = 0xB9;
const SWING: usize = 0xBB;
const REPEAT: usize = 0xBC;
const SLOT: usize = 0xBD;
const NO_MACE: usize = 0xBE;
const MACE_FLIP: usize = 0xBF;
const FLEE: usize = 0xC0;
const FLEE_FLAG: usize = 0xC2;
const HIT: usize = 0xC4;

impl Game {
    /// 0x358B2: class 3. Mode 0: sub 0 makes the three objects, else the init; 2 the AI; 4 the death; 6 frees (0x36A0A).
    pub(super) fn enemy_class3(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                if e.b.sub == 0 {
                    self.c3_make_three(e, h, i)
                } else {
                    self.c3_init(e, h)
                }
            }
            2 => {
                self.enemy_kill_all_check(e); // 0x326F8
                match e.b.sub {
                    0 => self.c3_alive(e, h),
                    2 => self.enemy_hit_reaction(e, h), // 0x2A606
                    4 => self.c3_death(e, h, i),
                    6 => self.enemy_grabbed(e, h), // 0x2B35A
                    _ => {}
                }
                self.enemy_clamp_to_playfield(e, i); // jmp 0x193E
            }
            4 => self.c3_death(e, h, i),
            _ => self.enemy_free_pool_a(e, i), // 0x36A0A: jmp 0x48FC
        }
    }

    /// The object of the handle `t`: the running one or its slot.
    fn c3_obj<'a>(&'a mut self, e: &'a mut Enemy, h: Handle, t: Handle) -> &'a mut Enemy {
        if t == h {
            e
        } else {
            let i = self.enemy_index(t);
            &mut self.enemies.slots[i]
        }
    }

    /// 0x358C6: sub 0. Fewer than two free slots, or no slot: free. Else three slots (his own and two new), sorted by address: the lowest becomes the Lash (class 3,
    /// sub 2: the init follows next frame), the middle the mace (class 62, he is in +$A0, the mace in his +$A6), the highest the chain piece (class 8, animation
    /// table 0x712A0, +$76 the mace). The place, the direction and the drop row come from the record object.
    fn c3_make_three(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if self.enemies.free_count() < 2 {
            return self.enemy_free_pool_a(e, i); // blt 0x36A0A
        }
        let (Some(j), Some(k)) = (self.enemies.alloc(), self.enemies.alloc()) else {
            return self.enemy_free_pool_a(e, i); // bcs 0x36A0A
        };
        let (mut a0, mut a1, mut a2) = (h as i16, self.enemies.handle(j) as i16, self.enemies.handle(k) as i16);
        if a1 < a0 {
            std::mem::swap(&mut a0, &mut a1);
        }
        if a2 < a1 {
            std::mem::swap(&mut a2, &mut a1);
        }
        if a1 < a0 {
            std::mem::swap(&mut a0, &mut a1);
        }
        let (a0, a1, a2) = (a0 as u16, a1 as u16, a2 as u16);
        let (dir, drop, x, y, z) = (e.b.dir, e.m.b(DROP), e.b.xi(), e.b.yi(), e.b.zi());
        {
            let l = self.c3_obj(e, h, a0);
            l.b.live = 1;
            l.b.k20 = 0xC;
            l.b.var26 = 0;
            l.b.dir = dir;
            l.m.sb(DROP, drop);
            l.b.set_xi_keep(x);
            l.b.set_yi_keep(y);
            l.b.set_zi_keep(z);
            l.b.sub = 2;
        }
        let (ldir, lx, ly, lz) = {
            let l = self.c3_obj(e, h, a0);
            (l.b.dir, l.b.xi(), l.b.yi(), l.b.zi())
        };
        {
            let m = self.c3_obj(e, h, a1);
            m.b.live = 1;
            m.b.k20 = 0xF8;
            m.b.var26 = 0;
            m.b.dir = ldir;
            m.b.set_xi_keep(lx);
            m.b.set_yi_keep(ly);
            m.b.set_zi_keep(lz);
            m.m.sl(0xA0, 0xFFFF_0000 | a0 as u32);
        }
        self.c3_obj(e, h, a0).m.sw(MACE, a1);
        let mace = a1;
        let c = self.c3_obj(e, h, a2);
        c.b.live = 1;
        c.b.var26 = 0;
        c.b.k20 = 0x20;
        c.b.anim_table = 0x712A0;
        c.m.sl(0xA0, 0xFFFF_0000 | a0 as u32);
        c.m.sw(TARGET, mace);
    }

    /// 0x35998: the init. Mode 2, name 0xE (+$84 = 0x38), the animation table 0x6C0E2, boxes 0x103C00, speed 7, direction 0x18, HP by level, the target, poise 10,
    /// the flee timer -300 and the state timer 0x14, on the floor, uncounted until the AI entry says otherwise (+$7D = 0); then the AI entry 0x35D36.
    fn c3_init(&mut self, e: &mut Enemy, h: Handle) {
        e.b.mode = 2;
        e.b.sub = 0;
        e.m.sb(B87, 0);
        e.b.id = 0xE;
        e.m.sw(W84, 0x38);
        e.b.anim_table = 0x6C0E2;
        self.enemy_anim(&mut e.b, 1);
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_set_speed(&mut e.b, 7); // 0x17A4
        e.b.dir = 0x18;
        self.enemy_set_hp(&mut e.b); // 0x31424
        e.m.sw(TARGET, self.enemy_pick_target(&e.b)); // 0x32032
        e.b.stun = 10;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.m.sb(ATTACKING, 0);
        e.b.b72 = 0xC8;
        e.m.sw(TIMER, 0x14);
        for off in [ATK_DELAY, NO_MACE, MACE_FLIP, 0xBA, WAIT, WALK, HIT] {
            e.m.sb(off, 0);
        }
        e.m.sw(FLEE, 0xFED4);
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sb(ATTACKING, 0);
        e.b.b63 = 0;
        // jsr 0x939B6: the debug marker (G.$4AA), not ported
        self.c3_reset_ai(e, h); // bra 0x35D36
    }

    /// 0x35D36: the AI entry (also the recover entry of the hit reaction for the name 0xE): state long 0x02000200, animation 0, a new target, the timer 0x14,
    /// facing, speed 5, a new slot (0x32D90). A knocked down Lash (+$7B = 4) is uncounted no more and starts in state 6 (animation 1, aimed at the target,
    /// 0x36494); else state 2.
    pub(super) fn c3_reset_ai(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0200_0200);
        self.enemy_anim(&mut e.b, 0);
        e.m.sw(TARGET, self.enemy_pick_target(&e.b));
        e.m.sw(TIMER, 0x14);
        self.lib_face_target(e); // 0x32B68
        self.enemy_set_speed(&mut e.b, 5);
        self.lib_slot3_new(e); // 0x32D90
        if e.b.reaction != 4 {
            return self.c3_state2(e, h); // bne 0x360C6
        }
        e.m.sb(ATTACKING, 0);
        e.m.sb(ATK_DELAY, 0);
        e.m.sb(BUSY, 0);
        e.b.act = 6;
        self.enemy_anim(&mut e.b, 1);
        self.enemy_set_speed(&mut e.b, 5);
        self.lib_aim_at_target(e); // 0x32A58
        self.c3_enter_6(e, h); // bra 0x36494
    }

    // ------------------------------------------------------------------ the AI frame

    /// 0x3B..: the draw check (0x49CA) and the hurtable list (0x4BEA): the end of the AI frame (0x35B18).
    fn c3_end(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// The mace is out (+$B2 of the mace object not 0).
    fn c3_mace_out(&self, e: &Enemy) -> bool {
        match slot_of(e.m.w(MACE)) {
            Some(Slot::Enemy(k)) => self.enemies.slots[k].m.b(0xB2) != 0,
            _ => false,
        }
    }

    /// 0x35A80: sub 0. The poise refill, the floor (0x12CB4), a hit stop (+$59) freezes the animation, the wait +$7A, the slot (0x32EC0); while the mace is out nothing
    /// else; the flee logic (0x35B40); an airborne Lash goes to 0x36510 (no draw); the state; the gate (0x35C1E) and the slot timer (0x35CC4); faces the target
    /// unless busy; the draw and the list.
    fn c3_alive(&mut self, e: &mut Enemy, h: Handle) {
        let p = e.m.b(POISE_T);
        if p != 0 {
            e.m.sb(POISE_T, p - 1);
            if p == 1 {
                e.b.stun = 10;
            }
        }
        self.enemy_floor_collision(e); // 0x12CB4
        if e.b.hitstop != 0 {
            e.b.hitstop -= 1;
            e.b.timer = 1;
            if e.m.b(HIT) != 0 {
                e.b.hitstop = 0;
                e.b.timer = 1;
            }
            return self.c3_end(e, h);
        }
        if e.m.b(WAIT) != 0 {
            e.m.sb(WAIT, e.m.b(WAIT) - 1);
            return self.c3_end(e, h);
        }
        self.c3_slot(e); // jsr 0x32EC0
        if self.c3_mace_out(e) {
            return self.c3_end(e, h);
        }
        // bsr 0x35C64: `tst.b $7a` is 0 here, it only returns
        self.c3_flee(e, h); // bsr 0x35B40
        if e.b.airborne != 0 {
            return self.c3_airborne(e); // bne 0x36510
        }
        match e.b.act {
            0 => self.c3_state0(e, h),
            2 => self.c3_state2(e, h),
            4 => self.c3_state4(e),
            6 => self.c3_state6(e, h),
            8 => self.c3_state8(e, h),
            0xA => self.c3_state_a(e, h),
            0x10 => self.c3_state10(e),
            0x12 => self.c3_state12(e, h),
            0x14 => self.c3_state14(e, h),
            0x18 => self.c3_state18(e, h),
            0x1A => self.c3_state1a(e, h),
            s => panic!("Lash state {s:#x} (object {h:#06x}): only the states 0 to 0x1A that the code sets exist (0xC, 0xE and 0x16 are never entered)"),
        }
        self.c3_gate(e, h); // bsr 0x35C1E
        self.c3_slot_timer(e, h); // bsr 0x35CC4
        if e.m.b(BUSY) == 0 {
            self.lib_face_target(e); // jsr 0x32B68
        }
        self.c3_end(e, h);
    }

    /// 0x32EC0: the slot of +$BD (a new one 1..3 by `rand & 3` while it is 0), its point beside the target from the table 0x32F32: +$88 / +$8A.
    fn c3_slot(&mut self, e: &mut Enemy) {
        let mut d0 = e.m.b(SLOT);
        while d0 == 0 {
            d0 = (self.rand() & 3) as u8;
            e.m.sb(SLOT, d0);
        }
        let off = self.assets.enemy.t.i16(t::SLOT_OFFSETS, d0 as u32 * 2);
        let (tx, tz) = {
            let b = self.lib_target(e);
            (b.xi(), b.zi())
        };
        e.m.sw(DEST_Z, tz as u16);
        e.m.sw(DEST_X, tx.wrapping_add(off) as u16);
        self.lib_slot_mirror(e, off);
    }

    /// 0x35B40: the flee and reposition timer +$C0. A Lash off screen (not in the states 0x12 / 0x14, not busy, not airborne) mirrors his point and runs back
    /// (0x36550); else the timer runs: positive phase 180 / 240 / 300 frames, then negative -60 / -90 / -120 (a random pair from the tables), the flag +$C2 is
    /// set at the end of the positive phase, a new target at each end.
    fn c3_flee(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.w(FLEE) == 0 {
            return;
        }
        let skip = e.b.act == 0x12 || e.b.act == 0x14 || e.m.b(BUSY) != 0 || e.b.airborne != 0;
        if !skip && self.lib_off_screen(&e.b) {
            let d = (e.m.w(DEST_X) as i16).wrapping_sub(e.b.xi());
            e.m.sw(DEST_X, d.wrapping_neg().wrapping_add(e.b.xi()) as u16);
            e.m.sw(FLEE, 0x3C);
            return self.c3_enter_12(e, h); // bra 0x36550
        }
        let d0 = (self.rand() & 0x1E) as u32;
        if (e.m.w(FLEE) as i16) < 0 {
            let v = e.m.w(FLEE).wrapping_add(1);
            e.m.sw(FLEE, v);
            if v != 0 {
                return;
            }
            e.m.sw(FLEE, self.assets.enemy.t.u16(t::FLEE_POS, d0));
        } else {
            let v = e.m.w(FLEE).wrapping_sub(1);
            e.m.sw(FLEE, v);
            if v != 0 {
                return;
            }
            e.m.sw(FLEE, self.assets.enemy.t.u16(t::FLEE_NEG, d0));
            e.m.sw(FLEE_FLAG, 1);
        }
        e.m.sw(TARGET, self.enemy_pick_target(&e.b)); // 0x35BCA: jsr 0x32032
    }

    /// 0x35C1E: the attack gate. Not while the attack delay +$B8 runs or busy; faces the target (0x32B68); within 10 px in depth of the point (+$8A + 7) the gate
    /// 0x36884: on the screen the roll `rand & 0x1F` against the byte 0x72954[aggression * 8 + X class] starts the attack decision 0x368C2.
    fn c3_gate(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.b(ATK_DELAY) != 0 {
            e.m.sb(ATK_DELAY, e.m.b(ATK_DELAY) - 1);
            return;
        }
        if e.m.b(BUSY) != 0 {
            return;
        }
        self.lib_face_target(e); // jsr 0x32B68
        let d0 = (e.m.w(DEST_Z) as i16).wrapping_add(7).wrapping_sub(e.b.zi()) as u16;
        if d0 <= 0xA {
            self.c3_gate_roll(e, h); // bls 0x36884
        }
    }

    /// 0x36884.
    fn c3_gate_roll(&mut self, e: &mut Enemy, h: Handle) {
        let cam = self.cam.s2.cam_x as i16;
        let x = e.b.xi();
        if cam > x {
            return; // bgt 0x368C0
        }
        if cam.wrapping_add(0x180) < x {
            return; // blt 0x368C0
        }
        let row = (self.rank.aggression << 3).wrapping_add(e.m.b(XCLASS) as u16) as u32;
        let d1 = self.assets.enemy.t.u8(t::GATE, row) as i8;
        let d0 = (self.rand() & 0x1F) as u8 as i8;
        if d0 < d1 {
            self.c3_decide_attack(e, h); // blt 0x368C2
        }
    }

    /// 0x35CC4: the slot timer +$A9 (180 frames): off screen (not in the states 0x12 / 0x14) the point is mirrored and the Lash runs back (0x36550); else 14 in 16
    /// (`rand & 0xF` below 0xE) a new slot (0x32D90) and a turn to the target.
    fn c3_slot_timer(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.b(SLOT_T) != 0 {
            e.m.sb(SLOT_T, e.m.b(SLOT_T) - 1); // 0x35D30
            return;
        }
        e.m.sb(SLOT_T, 0xB4);
        let skip = e.b.act == 0x12 || e.b.act == 0x14;
        if !skip && self.lib_off_screen(&e.b) {
            let d = (e.m.w(DEST_X) as i16).wrapping_sub(e.b.xi());
            e.m.sw(DEST_X, d.wrapping_neg().wrapping_add(e.b.xi()) as u16);
            e.m.sw(TIMER, 0x14);
            e.m.sw(FLEE, 0x3C);
            return self.c3_enter_12(e, h); // bra 0x36550 (it returns into the caller of this routine, which ends the same way)
        }
        if self.rand() & 0xF < 0xE {
            self.lib_slot3_new(e); // jsr 0x32D90
            self.lib_face_target(e); // jsr 0x32B68
        }
        e.m.sb(SLOT_T, e.m.b(SLOT_T).wrapping_sub(1)); // 0x35D30
    }

    /// 0x35C86: the step pause: +$A2 from the table 0x7431A by the level (+ the variant byte >> 5), the walk flag +$AD, animation 0xB when the mace is gone.
    fn c3_step_pause(&mut self, e: &mut Enemy) {
        if e.m.b(STEP_FLAG) == 0 {
            return;
        }
        let d2 = ((e.b.var26 >> 8) as u8 as i8) >> 5;
        let d1 = (self.w.level as u8).wrapping_add(d2 as u8);
        e.m.sb(STEP_PAUSE, self.assets.enemy.t.u8(t::PAUSE, d1 as u32));
        e.m.sb(STEP_FLAG, 0);
        e.m.sb(WALK, 1);
        if e.m.b(NO_MACE) != 0 {
            self.enemy_anim(&mut e.b, 0xB);
        }
    }

    // ------------------------------------------------------------------ the states

    /// 0x364CA: the turn limit: the new direction is at most two steps from the old one (+$A4).
    fn c3_turn_limit(&self, e: &mut Enemy) {
        let old = e.m.b(OLD_DIR);
        let mut d0 = e.b.dir.wrapping_sub(old) & 0x1F;
        d0 = d0.wrapping_add(2) & 0x1F;
        if (d0 as i8) <= 4 {
            return;
        }
        d0 = d0.wrapping_sub(2);
        e.b.dir = if (d0 as i8) > 0x10 { old.wrapping_sub(2) & 0x1F } else { old.wrapping_add(2) & 0x1F };
    }

    /// 0x35DAE: state 0, the drift to the slot point: every fourth frame the step longs +$B0 / +$B4 by the side of the point; the pause; X and Z steps until the
    /// point is reached (then the step flag is set and the walk flag cleared); else the step pause and the animation. At the end of the timer the decision 0x3680E.
    fn c3_state0(&mut self, e: &mut Enemy, h: Handle) {
        let n = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, n);
        if n == 0 {
            return self.c3_decide(e, h);
        }
        if n & 3 == 0 {
            e.m.sl(VX, if (e.m.w(DEST_X) as i16) > e.b.xi() { 0x12000 } else { 0xFFFE_E000 });
            e.m.sl(VZ, if (e.m.w(DEST_Z) as i16) > e.b.zi() { 0x10000 } else { 0xFFFF_0000 });
        }
        if e.m.b(STEP_PAUSE) != 0 {
            e.m.sb(STEP_PAUSE, e.m.b(STEP_PAUSE) - 1);
            return;
        }
        let mut moved = false;
        if ((e.m.w(DEST_X) as i16).wrapping_add(8).wrapping_sub(e.b.xi()) as u16) >= 0x10 {
            e.b.x = e.b.x.wrapping_add(e.m.l(VX) as i32);
            moved = true;
        }
        if ((e.m.w(DEST_Z) as i16).wrapping_add(0xA).wrapping_sub(e.b.zi()) as u16) >= 0xB {
            e.b.z = e.b.z.wrapping_add(e.m.l(VZ) as i32);
            moved = true;
        }
        if !moved {
            e.m.sb(STEP_FLAG, 1);
            e.m.sb(STEP_PAUSE, 0);
            e.m.sb(WALK, 0);
            return;
        }
        self.c3_step_pause(e);
        if e.m.b(STEP_PAUSE) == 0 {
            self.enemy_step(&mut e.b); // jmp 0x121E
        }
    }

    /// 0x360C6: state 2, the wait: the timer, then the decision; the animation steps.
    fn c3_state2(&mut self, e: &mut Enemy, h: Handle) {
        let n = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, n);
        if n == 0 {
            return self.c3_decide(e, h);
        }
        self.enemy_step(&mut e.b); // jmp 0x121E
    }

    /// 0x360A8: state 4, the swing animation; at its end state 0x10.
    fn c3_state4(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            e.b.act = 0x10;
        }
    }

    /// 0x360BA: state 0x10, the end of a swing: the animation steps; at its end the pause of the next swing (0x363E0).
    fn c3_state10(&mut self, e: &mut Enemy) {
        if !self.enemy_step(&mut e.b) {
            return;
        }
        e.b.act = 0x1A; // 0x363E0
        e.m.sb(BUSY, 1);
        let _ = self.rand();
        let r = (self.rand() & 0x3E) as u32;
        let mut d = self.assets.enemy.t.u16(t::REPEAT_A, r);
        let level = (self.w.level as u32) * 2;
        d = d.wrapping_add(self.assets.enemy.t.u16(t::REPEAT_B, level));
        e.m.sw(TIMER, if (d as i16) <= 0 { 1 } else { d });
    }

    /// 0x360D4: state 0x1A, the pause between the swings of a series: at its end the repeat count +$BC runs down: more swings (0x36104: attacks 8, 0x52, the
    /// animation by the series table 0x36222 and the attacks 0x51 / 0x53 / 0x52 / 0x57 by it), none: state 2.
    fn c3_state1a(&mut self, e: &mut Enemy, _h: Handle) {
        let n = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, n);
        if n != 0 {
            self.enemy_step(&mut e.b); // jmp 0x121E
            return;
        }
        e.m.sb(MACE_FLIP, 0);
        let r = e.m.b(REPEAT).wrapping_sub(1);
        e.m.sb(REPEAT, r);
        if r == 0 {
            e.m.sw(TIMER, 0x14);
            e.b.act = 2;
            return self.enemy_anim(&mut e.b, 0); // jmp 0x120E
        }
        // 0x36104
        self.enemy_begin_attack(&mut e.b, 8);
        e.m.sb(BUSY, 1);
        e.b.act = 4;
        e.m.sw(TIMER, 0x3C);
        self.enemy_begin_attack(&mut e.b, 0x52);
        self.lib_aim_beside_target(e, 0x50); // jsr 0x32A88
        let row = (e.m.b(SWING) as u32) * 4 + e.m.b(REPEAT) as u32;
        let anim = self.assets.enemy.t.u8(t::SERIES, row);
        if e.m.b(SWING) == 0 {
            self.enemy_begin_attack(&mut e.b, 0x51);
        }
        if anim == 2 {
            self.enemy_begin_attack(&mut e.b, 0x53);
        }
        if anim == 8 && e.m.b(SWING) == 1 {
            self.enemy_begin_attack(&mut e.b, 0x52);
        }
        if anim == 8 && e.m.b(SWING) == 2 {
            self.enemy_begin_attack(&mut e.b, 0x51);
        }
        if anim == 5 {
            self.enemy_begin_attack(&mut e.b, 0x57);
        }
        self.enemy_anim(&mut e.b, anim as u16); // jmp 0x120E
    }

    /// 0x36246: state 6, the walk to the attack point beside the target (0x32A88 every 6 frames, turn limit): the timer ends in the decision; the busy flag is
    /// cleared in the last 0x3C frames; at the point (within 16 in X of target X +/- 0x38 and 10 in depth) the step flag is set, else the step pause and the move.
    fn c3_state6(&mut self, e: &mut Enemy, h: Handle) {
        let n = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, n);
        if n == 0 {
            return self.c3_decide(e, h);
        }
        if (n as i16) <= 0x3C {
            e.m.sb(BUSY, 0);
        }
        let a = e.m.b(AIM_T).wrapping_sub(1);
        e.m.sb(AIM_T, a);
        if a == 0 {
            e.m.sb(AIM_T, 6);
            e.m.sb(OLD_DIR, e.b.dir);
            self.lib_aim_beside_target(e, 0x50); // jsr 0x32A88
        }
        if e.m.b(STEP_PAUSE) != 0 {
            e.m.sb(STEP_PAUSE, e.m.b(STEP_PAUSE) - 1);
            return;
        }
        let (tx, tz) = {
            let b = self.lib_target(e);
            (b.xi(), b.zi())
        };
        let side: i16 = if tx <= e.b.xi() { 0x38 } else { -0x38 };
        let near_x = (tx.wrapping_add(side).wrapping_add(8).wrapping_sub(e.b.xi()) as u16) <= 0x10;
        let near_z = (tz.wrapping_add(7).wrapping_sub(e.b.zi()) as u16) < 0xA;
        if !(near_x && near_z) {
            self.c3_step_pause(e);
            self.enemy_move_dir(&mut e.b); // jsr 0x1862
            self.enemy_step(&mut e.b); // jmp 0x121E
            return;
        }
        e.m.sb(STEP_FLAG, 1);
        e.m.sb(STEP_PAUSE, 0);
        e.m.sb(WALK, 0);
    }

    /// 0x35F6C: state 8, the walk in X toward +$88: the step long +$B0 every fourth frame, the step pause, steps until within 16 px, then the step flag; the
    /// animation and the facing; at the end of the timer the decision.
    fn c3_state8(&mut self, e: &mut Enemy, h: Handle) {
        let n = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, n);
        if n == 0 {
            return self.c3_decide(e, h);
        }
        if n & 6 == 0 {
            e.m.sl(VX, if (e.m.w(DEST_X) as i16) > e.b.xi() { 0x15000 } else { 0xFFFE_B000 });
        }
        if e.m.b(STEP_PAUSE) != 0 {
            e.m.sb(STEP_PAUSE, e.m.b(STEP_PAUSE) - 1);
            return;
        }
        let mut moved = false;
        if ((e.m.w(DEST_X) as i16).wrapping_add(8).wrapping_sub(e.b.xi()) as u16) >= 0x10 {
            self.c3_step_pause(e);
            if e.m.b(STEP_PAUSE) != 0 {
                return; // bne 0x36008
            }
            e.b.x = e.b.x.wrapping_add(e.m.l(VX) as i32);
            moved = true;
        }
        if moved {
            self.enemy_step(&mut e.b); // jsr 0x121E
        } else {
            e.m.sb(STEP_FLAG, 1);
            e.m.sb(STEP_PAUSE, 0);
            e.m.sb(WALK, 0);
        }
        self.lib_face_target(e); // jmp 0x32B68
        let _ = h;
    }

    /// 0x3600A: state 0xA, the same in Z (+$B4, the point +$8A, 11 px).
    fn c3_state_a(&mut self, e: &mut Enemy, h: Handle) {
        let n = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, n);
        if n == 0 {
            return self.c3_decide(e, h);
        }
        if n & 6 == 0 {
            e.m.sl(VZ, if (e.m.w(DEST_Z) as i16) > e.b.zi() { 0x15000 } else { 0xFFFE_B000 });
        }
        if e.m.b(STEP_PAUSE) != 0 {
            e.m.sb(STEP_PAUSE, e.m.b(STEP_PAUSE) - 1);
            return;
        }
        let mut moved = false;
        if ((e.m.w(DEST_Z) as i16).wrapping_add(0xA).wrapping_sub(e.b.zi()) as u16) >= 0xB {
            self.c3_step_pause(e);
            if e.m.b(STEP_PAUSE) != 0 {
                return; // bne 0x360A6
            }
            e.b.z = e.b.z.wrapping_add(e.m.l(VZ) as i32);
            moved = true;
        }
        if moved {
            self.enemy_step(&mut e.b);
        } else {
            e.m.sb(STEP_FLAG, 1);
            e.m.sb(STEP_PAUSE, 0);
            e.m.sb(WALK, 0);
        }
        self.lib_face_target(e); // jmp 0x32B68
    }

    /// 0x35E80: state 0x12, the run back onto the screen: at the end of the timer 0x365E0; every 12 frames the direction to the point (turn limit); facing by the
    /// direction; the move and the animation.
    fn c3_state12(&mut self, e: &mut Enemy, h: Handle) {
        let n = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, n);
        if n == 0 {
            return self.c3_end_12(e, h);
        }
        let a = e.m.b(AIM_T).wrapping_sub(1);
        e.m.sb(AIM_T, a);
        if a == 0 {
            e.m.sb(AIM_T, 0xC);
            e.m.sb(OLD_DIR, e.b.dir);
            self.lib_aim_at_target(e); // jsr 0x32A70
            self.c3_turn_limit(e);
        }
        e.b.facing = u8::from(e.b.dir != 0 && (e.b.dir as i8) < 0x10);
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x35ED2: state 0x14, the side steps (2 px in X, 1.5 px in Z toward the target); at the end of the timer 0x36550.
    fn c3_state14(&mut self, e: &mut Enemy, h: Handle) {
        let n = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, n);
        if n == 0 {
            return self.c3_enter_12(e, h);
        }
        let (tx, tz) = {
            let b = self.lib_target(e);
            (b.xi(), b.zi())
        };
        e.b.x = e.b.x.wrapping_add(if tx > e.b.xi() { 0x20000 } else { 0xFFFE_0000u32 as i32 });
        e.b.z = e.b.z.wrapping_add(if tz > e.b.zi() { 0x18000 } else { 0xFFFE_8000u32 as i32 });
        e.b.facing = u8::from(e.b.dir != 0 && (e.b.dir as i8) < 0x10);
        self.enemy_step(&mut e.b);
    }

    /// 0x35F42: state 0x18, airborne: landed, the decision; else the motion and the animation.
    fn c3_state18(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.airborne == 0 {
            return self.c3_decide(e, h);
        }
        e.b.motion(); // jsr 0x189C
        self.enemy_step(&mut e.b);
    }

    /// 0x36510: airborne (knocked up): busy, 60 frames, state 0x18, animation 0x13, vy 0x800 and no other speed; faces the target.
    fn c3_airborne(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 1);
        e.m.sw(TIMER, 0x3C);
        e.b.airborne = 1;
        e.b.act = 0x18;
        self.enemy_anim(&mut e.b, 0x13);
        e.b.vy = 0x800;
        e.b.ay = 0;
        e.b.ax = 0;
        e.b.az = 0;
        self.lib_face_target(e); // jmp 0x32B68
    }

    // ------------------------------------------------------------------ decisions and state entries

    /// 0x36550: state 0x12 entry: on the screen the decision (0x3680E) instead; else the timer 100, a new slot (0x32C5E), the direction to the point, facing toward the
    /// screen side (the code that could set facing 1 never runs), speed 0x1C, animation 1.
    fn c3_enter_12(&mut self, e: &mut Enemy, h: Handle) {
        if !self.lib_off_screen(&e.b) {
            return self.c3_decide(e, h); // beq 0x3680E
        }
        e.b.act = 0x12;
        e.m.sw(TIMER, 0x64);
        e.m.sb(AIM_T, 0x1E);
        let _ = self.lib_slot6_new(e); // jsr 0x32C5E
        e.m.sb(OLD_DIR, e.b.dir);
        self.lib_aim_at_target(e); // jsr 0x32A70
        e.b.facing = 0;
        e.b.dir = 0x10;
        if (self.rand() & 0xF) > 0xC {
            e.b.dir = e.b.dir.wrapping_add(0x10) & 0x10;
        }
        self.enemy_set_speed(&mut e.b, 0x1C);
        self.enemy_anim(&mut e.b, 1); // jmp 0x120E
    }

    /// 0x365E0: the end of state 0x12: on the screen the decision; else state 0x14 for 100 frames with the side step longs toward the point.
    fn c3_end_12(&mut self, e: &mut Enemy, h: Handle) {
        if !self.lib_off_screen(&e.b) {
            return self.c3_decide(e, h); // beq 0x3680E
        }
        e.m.sb(BUSY, 0);
        e.m.sw(TIMER, 0x64);
        e.b.act = 0x14;
        e.m.sl(VX, if (e.m.w(DEST_X) as i16) > e.b.xi() { 0x24000 } else { 0xFFFD_C000 });
        e.m.sl(VZ, if (e.m.w(DEST_Z) as i16) > e.b.zi() { 0x1C000 } else { 0xFFFE_4000 });
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x3680E: the first decision. The attack delay is armed when +$B9 is set, speed 5, not busy, the X class of the target (0x3293C), the table 0x72754
    /// (0x72854 without the mace) by `(rand & 0x1F) * 8 + X class`, the jump table 0x36876.
    fn c3_decide(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.b(ATK_FLAG) != 0 {
            e.m.sb(ATK_DELAY, 0x78);
            e.m.sb(ATK_FLAG, 0);
        }
        self.enemy_set_speed(&mut e.b, 5);
        e.m.sb(BUSY, 0);
        let (xc, _) = self.lib_distance_class(e);
        e.m.sb(XCLASS, xc as u8);
        let row = ((self.rand() & 0x1F) << 3) as u32 + xc as u32;
        let table = if e.m.b(NO_MACE) != 0 { t::DECIDE_NO_MACE } else { t::DECIDE };
        match self.assets.enemy.t.u8(table, row) {
            1 | 3 => self.c3_enter_6(e, h),
            2 => self.c3_enter_0(e),
            4 => self.c3_enter_8(e),
            5 => self.c3_enter_a(e),
            6 => {
                // 0x36804: the mace is gone, then the decision again
                e.m.sb(NO_MACE, 1);
                self.c3_decide(e, h);
            }
            v => panic!("Lash decision value {v} (the table 0x72754 / 0x72854 has only 1 to 5)"),
        }
    }

    /// 0x36430: state 0 entry: 60 frames of drift toward the point with the step longs, animation 1 (0xE without the mace).
    fn c3_enter_0(&mut self, e: &mut Enemy) {
        e.m.sw(TIMER, 0x3C);
        e.b.act = 0;
        self.lib_face_target(e); // jsr 0x32B68
        e.m.sl(VX, if (e.m.w(DEST_X) as i16) > e.b.xi() { 0x18000 } else { 0xFFFE_8000 });
        e.m.sl(VZ, if (e.m.w(DEST_Z) as i16) > e.b.zi() { 0xC000 } else { 0xFFFF_4000 });
        let anim = if e.m.b(NO_MACE) != 0 { 0xE } else { 1 };
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x36494: state 6 entry (decisions 1 and 3): the direction beside the target (turn limit), the aim timer 6, 60 frames, animation 1 (0xE without the mace).
    fn c3_enter_6(&mut self, e: &mut Enemy, _h: Handle) {
        e.m.sb(OLD_DIR, e.b.dir);
        self.lib_aim_beside_target(e, 0x50);
        self.c3_turn_limit(e);
        e.m.sb(AIM_T, 6);
        e.m.sw(TIMER, 0x3C);
        e.b.act = 6;
        let anim = if e.m.b(NO_MACE) != 0 { 0xE } else { 1 };
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x36644: state 8 entry: the aim timer 6, 60 frames, a step long of 1 px toward the point.
    fn c3_enter_8(&mut self, e: &mut Enemy) {
        e.m.sb(AIM_T, 6);
        e.m.sw(TIMER, 0x3C);
        e.b.act = 8;
        e.m.sl(VX, 0x10000);
        e.m.sl(VZ, 0);
        if (e.m.w(DEST_X) as i16) <= e.b.xi() {
            e.m.sl(VX, 0x10000u32.wrapping_neg());
        }
        let anim = if e.m.b(NO_MACE) != 0 { 0xE } else { 1 };
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x36688: state 0xA entry: the same in Z.
    fn c3_enter_a(&mut self, e: &mut Enemy) {
        e.m.sb(AIM_T, 6);
        e.m.sw(TIMER, 0x3C);
        e.b.act = 0xA;
        e.m.sl(VX, 0);
        e.m.sl(VZ, 0x10000);
        if (e.m.w(DEST_Z) as i16) <= e.b.zi() {
            // neg.w $b4(a6): the high word of the long
            e.m.sl(VZ, 0xFFFF_0000);
        }
        let anim = if e.m.b(NO_MACE) != 0 { 0xE } else { 1 };
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x368C2: the attack decision: the delay is armed when +$B9 is set, speed 5, not busy, the X class, the table 0x72994 by `(rand & 0x1F) * 8 + X class`
    /// (4 without the mace, which is then back); the swing starts by the value minus 1 (0x36740).
    fn c3_decide_attack(&mut self, e: &mut Enemy, _h: Handle) {
        if e.m.b(ATK_FLAG) != 0 {
            e.m.sb(ATK_DELAY, 0x78);
            e.m.sb(ATK_FLAG, 0);
        }
        self.enemy_set_speed(&mut e.b, 5);
        e.m.sb(BUSY, 0);
        let (xc, _) = self.lib_distance_class(e);
        e.m.sb(XCLASS, xc as u8);
        let row = ((self.rand() & 0x1F) << 3) as u32 + xc as u32;
        let d1 = if e.m.b(NO_MACE) == 0 {
            self.assets.enemy.t.u8(t::ATTACK, row)
        } else {
            e.m.sb(NO_MACE, 0);
            4
        };
        self.c3_swing(e, (d1 as u16).wrapping_sub(1));
    }

    /// 0x36740: the swing starts: attack 8, busy, state 4, 60 frames; the row `d0` of 0x367CC: +$BB, +$BC and the animation; attack 0x51 (0x58 for the animation
    /// 0xC); the direction beside the target (0x32A88, 0x50).
    fn c3_swing(&mut self, e: &mut Enemy, d0: u16) {
        self.enemy_begin_attack(&mut e.b, 8);
        e.m.sb(BUSY, 1);
        e.b.act = 4;
        e.m.sw(TIMER, 0x3C);
        let row = d0 as u32 * 4;
        let t = &self.assets.enemy.t;
        let (bb, bc, anim) = (t.u8(t::SWINGS, row), t.u8(t::SWINGS, row + 1), t.u8(t::SWINGS, row + 2));
        e.m.sb(SWING, bb);
        e.m.sb(REPEAT, bc);
        e.m.sb(HIT, 0);
        self.enemy_begin_attack(&mut e.b, 0x51);
        if anim == 0xC {
            self.enemy_begin_attack(&mut e.b, 0x58);
        }
        self.enemy_anim(&mut e.b, anim as u16);
        self.lib_aim_beside_target(e, 0x50); // jmp 0x32A88
    }

    // ------------------------------------------------------------------ death

    /// 0x369D0: the death (mode 4, or sub 4 of mode 2): sub 0 the kill score and 120 frames, sub 2 counts them down (drawn on odd counts) and frees the object.
    fn c3_death(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.sub == 0 {
            e.m.sw(T80, 0x78);
            let b = e.b.clone();
            self.enemy_award_kill_score(&b); // jsr 0x325E6
            e.b.sub = e.b.sub.wrapping_add(2);
        }
        let w = e.m.w(T80);
        if w == 0 {
            return self.enemy_free_pool_a(e, i); // bcs 0x36A0A
        }
        e.m.sw(T80, w - 1);
        if (w - 1) & 1 != 0 {
            self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game() -> Game {
        Game::new(Box::leak(Box::new(AssetPack::default())))
    }

    /// 0x358CE: the lowest address of the three slots is the Lash, the middle one the mace, the highest the chain piece, whichever of them the record object was; the
    /// record's place and direction go to all three, the Lash is told the mace and the helpers the Lash.
    #[test]
    fn the_three_objects_are_sorted_by_address() {
        for first in [0usize, 1, 2] {
            let mut g = game();
            // take three slots in the allocator's order and make the `first`-th of the three the record object
            let slots: Vec<usize> = (0..3).map(|_| g.enemies.alloc().unwrap()).collect();
            let i = slots[first];
            let h = g.enemies.handle(i);
            let mut e = Enemy::default();
            e.b.live = 1;
            e.b.k20 = 12;
            e.b.dir = 8;
            e.b.set_xi(0x300);
            e.b.set_zi(0x30);
            g.enemies.release(slots[(first + 1) % 3]); // two of them go back to the free list for the routine to take again
            g.enemies.release(slots[(first + 2) % 3]);
            g.enemy_class3(&mut e, h, i);
            g.enemies.slots[i] = e;
            let mut by_addr: Vec<(Handle, u16)> = (0..24).filter(|&k| g.enemies.slots[k].b.live == 1).map(|k| (g.enemies.handle(k), g.enemies.slots[k].b.k20)).collect();
            by_addr.sort_by_key(|&(h, _)| h as i16);
            let classes: Vec<u16> = by_addr.iter().map(|&(_, c)| c).collect();
            assert_eq!(classes, vec![0xC, 0xF8, 0x20], "record object number {first}");
            let lash = by_addr[0].0;
            let li = g.enemy_index(lash);
            assert_eq!(g.enemies.slots[li].b.sub, 2, "the Lash inits next frame");
            assert_eq!(g.enemies.slots[li].m.w(MACE), by_addr[1].0);
        }
    }

    /// 0x364CA: the turn limit keeps the new direction within two steps of the old one.
    #[test]
    fn the_turn_limit_allows_two_steps() {
        let g = game();
        let mut e = Enemy::default();
        e.m.sb(OLD_DIR, 8);
        e.b.dir = 9;
        g.c3_turn_limit(&mut e);
        assert_eq!(e.b.dir, 9);
        e.b.dir = 20;
        g.c3_turn_limit(&mut e);
        assert!(e.b.dir == 10 || e.b.dir == 6, "got {}", e.b.dir);
    }
}
