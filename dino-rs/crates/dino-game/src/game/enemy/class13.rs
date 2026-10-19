//! Pool A class 13 (`+$20 = 0x34`, handler 0x3DDEA, 9,252 bytes): POACHER J. (name 2, variant 0), SKINNER (name 0x12, variant 2) and
//! GUTTER (name 0x13, variant 4 and larger), the ex soldiers with rifles. M6-01 / M6-02.
//!
//! Files: this one has the init (0x3DDFE), the alive frame (0x3DF68, AI frame 0x3E0B8), the small routines of the frame (flee timer
//! 0x3E1FA, slot refresh 0x3E4A2, the attack start tests 0x3E420, the pause chooser 0x3E4DC), the guard (0x3DFC6), the ten
//! entrances (table 0x3DF54, routines 0x3E504 .. 0x4001C) and the death (0x401CE). `class13_ai.rs` has the state handlers (table
//! 0x3E1AA) and the two decisions (0x4005C, 0x40118) with the state entries they call; `class13_attack.rs` has the dash tackle, the
//! rifle volley, the melee, the kick and the bullet / flash spawns.
//!
//! The low byte of `+$26` is the entrance (0 walk in, 2 stand 1200 frames, 4 rise with a fade, 6 jump in from above, 8 / 0xA / 0xE dash
//! in, 0xC run in, 0x10 / 0x12 jump on), the high byte is the variant. `+$87` is 1: the player's hit pass calls the guard 0x3DFC6 (table
//! 0x10B50 by `+$84` = 8 / 0x48 / 0x4C) for a hit from the front. The chance table 0x741D4 of the shipped ROM is all 0, so the guard never blocks (the
//! hop back 0x3FBE4 / state 0x28 is ported but never runs), but it draws one random number and throws it away (`moveq #0,d0` before the compare)
//! when a hit meets a poacher on the ground, in sub 0, not committed and facing the target: the random sequence needs that call (checked against MAME
//! in `m600_bs_b1_b1_nat`: without the draw the first hit at frame 1109 shows the difference).
//!
//! Every routine names its original address. A routine that ends in a carry in the original returns `bool` here (true = carry set).
//! Dead code (no caller, no table entry), not ported: 0x3E2CE, 0x3E30C, 0x3E328, 0x3E39C (stuck checks), 0x3E94E / 0x3EE6E (states 0x30 and
//! 0x32), 0x3EBB6 (state 0x1A) with its entry 0x3F7AC, 0x3ED02 (state 0x20) with its entry 0x3F95A, 0x3EB22 (state 0x3A) with 0x3F708 and
//! 0x3F728, 0x3EB72 (state 0x40) with 0x3F656, 0x3F292, 0x3F58C, 0x3FA7E and 0x400D2. The decision tables hold only the values the live
//! entries serve: the tables of the ROM are checked when read (a value of a dead entry panics).

use super::lib32::put_hi;
use super::{Enemy, off::*};
use crate::game::{Game, Handle};

/// Offsets (this class' meaning). The harness trace has these bytes as plain `r..` bytes.
pub(super) mod f {
    pub const PREV_ACT: usize = 0xA0; // byte: the state before the last decision
    pub const BUSY: usize = 0xA1; // byte: 1 = in a shot / dash / pause state: no turning, no slot walk
    pub const STILL_T: usize = 0xA2; // byte: frames to stand still at a point (drift states)
    pub const ARRIVED: usize = 0xA3; // byte: 1 = the point was reached (the pause is chosen next, 0x3E4DC)
    pub const DIST: usize = 0xA5; // byte: X distance class of the target, stored by the last decision
    pub const AIM_T: usize = 0xA8; // byte: frames until the next re-aim
    pub const OLD_DIR: usize = 0xA9; // byte: the direction before a turn (the turn limit)
    pub const TICK: usize = 0xAA; // byte: 10 .. 1 counter of the attack start tests
    pub const TIMER: usize = 0xAC; // word: frames left in the state
    pub const SLOT_T: usize = 0xAE; // byte: frames until the next slot is drawn
    pub const COMMIT: usize = 0xAF; // byte: 1 = committed (no slot walk, no target check, no floor test)
    pub const VX: usize = 0xB0; // long (16.16): X step of the drift states
    pub const VZ: usize = 0xB4; // long (16.16): Z step
    pub const FLEE_T: usize = 0xB8; // word (signed): the flee timer (positive: counts down, negative: counts up)
    pub const MELEE_A: usize = 0xBA; // byte: melee kind (row of the animation table)
    pub const MELEE_N: usize = 0xBB; // byte: hits left of the chain
    pub const LUNGE: usize = 0xBC; // byte: flag started by the guard (0x3FBE4, never reached)
    pub const COUNT: usize = 0xBE; // byte (signed): the guard counter that walks toward 0
    pub const SPEED: usize = 0xC0; // long: the drift speed chosen by the decision 4
    pub const SHOTS: usize = 0xC4; // word: shots left of the volley
    pub const FLEE_FLAG: usize = 0xC6; // word: 1 = the flee timer may send the object off screen
    pub const FROM_X: usize = 0xC8; // long: the X the jump on entrances started from
    pub const FROM_Z: usize = 0xCC; // long
}
use f::*;

/// The tables of the class (the 0x3E28E .. block is read from the ROM at run time).
pub(super) mod t {
    pub const DECISION1: u32 = 0x72A94;
    pub const FAR_GATE: u32 = 0x72D94;
    pub const FAR_DECISION: u32 = 0x72F14;
    pub const FLEE_POS: u32 = 0x3E28E;
    pub const FLEE_NEG: u32 = 0x3E2AE;
    pub const PAUSE_AFTER_SHOT: u32 = 0x3EADA;
    pub const PAUSE_AT_POINT: u32 = 0x7435A;
    pub const CLOSE_IN: u32 = 0x743BA;
    pub const CLOSE_IN_ADD: u32 = 0x3F636;
    pub const GUARD_CHANCE: u32 = 0x741D4;
    pub const GUARD_COUNT: u32 = 0x741EC;
    pub const GUARD_AGAIN: u32 = 0x7424F;
    pub const GUARD_PAUSE: u32 = 0x74268;
    pub const FADE_DEST: u32 = 0x3EFFC;
    pub const FADE_SRC: u32 = 0x3F008;
}

impl Game {
    /// 0x3DDEA class 13 (jump table 0x3DDF6: mode 0 -> 0x3DDFE, 2 -> 0x3DF68, 4 -> 0x401CE, 6 -> 0x40208).
    pub fn enemy_class13(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => self.enemy_c13_init(e, h),
            2 => self.enemy_c13_alive(e, h),
            4 => self.enemy_c13_dying(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x40208: jmp 0x48FC
            }
        }
    }

    /// The variant: the high byte of `+$26`.
    pub(super) fn enemy_c13_variant(e: &Enemy) -> u32 {
        (e.b.var26 >> 8) as u32 & 0xFF
    }

    // ------------------------------------------------------------------ init

    /// 0x3DDFE (mode 0): the type by the variant, the fields, two slots (0x32C5E), then the entrance by the low byte of `+$26`. The
    /// entrance ends the frame (no alive frame, no draw).
    fn enemy_c13_init(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sb(B87, 1);
        e.m.sb(LUNGE, 0);
        e.b.b63 = 0;
        // `move.b $26(a6),d0; subq.w #2,d0`: below 2 (bmi) Poacher J, 2 (beq) Skinner, above Gutter.
        let (id, w84, table) = match Self::enemy_c13_variant(e) {
            0 | 1 => (2, 8, 0x6C57A),
            2 => (0x12, 0x48, 0x6CA24),
            _ => (0x13, 0x4C, 0x6CF3E),
        };
        e.b.id = id;
        e.m.sw(W84, w84);
        e.b.anim_table = table;
        self.enemy_anim(&mut e.b, 1);
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_set_speed(&mut e.b, 8);
        e.b.dir = 0x18;
        self.enemy_set_hp(&mut e.b); // 0x31424, d0 = 0
        let tg = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, tg);
        e.b.stun = 0xA;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.m.sb(ATTACKING, 0);
        e.b.b72 = 0xC8;
        e.m.sw(TIMER, 0x14);
        e.m.sb(COUNT, 0);
        e.m.sb(COMMIT, 0);
        e.m.sw(FLEE_T, 0xFED4);
        e.m.sb(TICK, 0x1E);
        e.m.sb(AIM_T, 0xA);
        e.m.sb(BUSY, 0);
        e.m.sb(PREV_ACT, 0);
        e.m.sb(STILL_T, 0);
        e.m.sb(ARRIVED, 0);
        e.m.sb(0xA4, 0);
        e.m.sb(OLD_DIR, 0);
        e.m.sb(SLOT_T, 0);
        e.m.sw(0xA6, 0);
        e.m.sb(WAIT, 0);
        e.m.sw(FLEE_FLAG, 0);
        e.m.sw(SHOTS, 0);
        e.m.sl(SPEED, 0);
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sb(ATTACKING, 0);
        let _ = self.lib_slot6_new(e); // 0x32C5E
        // 0x3DF1C
        e.b.set_state(0x0200_0000);
        e.m.sb(ATTACKING, 0);
        let _ = self.lib_slot6_new(e);
        let tg = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, tg);
        self.lib_aim_at_target(e); // 0x32A70
        e.m.sb(BUSY, 0);
        match e.b.var26 & 0xFF {
            0 => self.enemy_c13_3e504(e),
            2 => self.enemy_c13_3fc3e(e),
            4 => self.enemy_c13_3fca4(e),
            6 => self.enemy_c13_3fce0(e),
            8 => self.enemy_c13_3f88e(e),
            0xA => self.enemy_c13_3ff26(e),
            0xC => self.enemy_c13_4001c(e),
            0xE => self.enemy_c13_3ffae(e),
            0x10 => self.enemy_c13_3fd3e(e),
            0x12 => self.enemy_c13_3fe60(e),
            v => panic!("class 13 entrance {v:#x} (object {h:#06x}): the table 0x3DF54 has ten entries"),
        }
    }

    // ------------------------------------------------------------------ alive frame

    /// 0x3DF68 (mode 2): a hit in sub 4 above state 0x22 resets the state to 0 (sub 4 is never entered: the table 0x3DF9C has no code
    /// for it), the kill-all check, then sub 0 the AI frame (0x3E0B8), 2 the hit reaction (0x2A606), 6 held by a player (0x2B35A);
    /// the play field clamp (0x193E).
    fn enemy_c13_alive(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.sub == 4 && (e.b.act as i8) > 0x22 {
            e.b.act = 0; // 0x3DF7C
        }
        self.enemy_kill_all_check(e); // 0x326F8
        match e.b.sub {
            0 => self.enemy_c13_ai(e, h),
            2 => self.enemy_hit_reaction(e, h),
            6 => self.enemy_grabbed(e, h),
            s => panic!("class 13 sub {s:#x} (the table 0x3DF9C has only 0, 2, 4 and 6)"),
        }
        let i = self.enemy_index(h);
        self.enemy_clamp_to_playfield(e, i); // jmp 0x193E
    }

    /// 0x3E198: the floor (0x12CB4), the draw list (0x49CA) and the hurtable list (0x4BEA): the end of the AI frame.
    fn enemy_c13_ai_end(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_floor_collision(e);
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x3E0B8: the AI frame (sub 0).
    fn enemy_c13_ai(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.b(LUNGE) != 0 {
            // 0x3E0BC: the guard blocked a hit (it set +$BC): hop back (0x3FBE4, state 0x28).
            e.m.sb(LUNGE, 0);
            self.enemy_c13_3fbe4(e);
        }
        let p = e.m.b(POISE_T);
        if p != 0 {
            e.m.sb(POISE_T, p - 1);
            if p - 1 == 0 {
                e.b.stun = 0xA;
            }
        }
        if e.b.hitstop != 0 {
            e.b.hitstop -= 1;
            e.b.timer = 1; // +$2F
            return self.enemy_c13_ai_end(e, h);
        }
        let w = e.m.b(WAIT);
        if w != 0 {
            e.m.sb(WAIT, w - 1);
            return self.enemy_c13_ai_end(e, h);
        }
        self.enemy_c13_3dfac(e);
        if e.m.b(COMMIT) == 0 {
            if e.b.act == 0x4A || e.b.act == 0x4C || e.m.b(BUSY) == 0 {
                self.enemy_c13_3e2f8(e);
                self.lib_slot6_keep(e); // 0x32C54
                if e.b.wall == 0x10 {
                    e.b.vz = 0x20; // 0x3E38C
                    e.b.az = 0;
                    return self.enemy_c13_3e3d4(e);
                }
                if e.b.airborne != 0 {
                    return self.enemy_c13_3e3d4(e);
                }
            }
            self.lib_check_target(e); // 0x32F56
        }
        self.enemy_c13_state(e, h); // 0x3E14E, table 0x3E1AA
        self.enemy_c13_3e1fa(e);
        self.enemy_c13_3e4a2(e);
        self.enemy_c13_3e420(e);
        if e.m.b(TICK) & 1 == 0 && e.m.b(BUSY) == 0 && e.b.act != 0x46 {
            let t = self.lib_target(e);
            if t.mode == 2 && t.sub == 2 {
                self.enemy_c13_3fc60(e); // the target is in a hit reaction: wait
            }
        }
        self.enemy_c13_ai_end(e, h);
    }

    /// 0x3DFAC: the guard counter `+$BE` walks toward 0.
    fn enemy_c13_3dfac(&mut self, e: &mut Enemy) {
        let v = e.m.b(COUNT) as i8;
        if v > 0 {
            e.m.sb(COUNT, (v - 1) as u8);
        } else if v < 0 {
            e.m.sb(COUNT, (v + 1) as u8);
        }
    }

    /// 0x3E2F8: an obstacle in the way (bit 7 of the wall bits) while not busy: swing at it (0x3F242).
    fn enemy_c13_3e2f8(&mut self, e: &mut Enemy) {
        if e.m.b(BUSY) == 0 && e.b.wall & 0x80 != 0 {
            self.enemy_c13_3f242(e);
        }
    }

    /// 0x3E3D4: stuck or at a ledge or in the air: jump (state 0x12). Ends the AI frame (the original jumps to 0x120E).
    fn enemy_c13_3e3d4(&mut self, e: &mut Enemy) {
        e.m.sb(COMMIT, 1);
        e.m.sb(BUSY, 1);
        e.m.sw(TIMER, 0x3C);
        e.b.act = 0x12;
        e.b.vx = 0x200;
        e.b.vy = 0x540;
        e.b.ax = 0;
        e.b.ay = 0xFFBBu16 as i16;
        e.m.sb(0xA4, 0);
        if (e.b.dir as i8) >= 0x10 {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        self.enemy_anim(&mut e.b, 0x13);
    }

    /// 0x3E1FA: the flee timer `+$B8`. Positive: counts down, then loads a negative wait and arms the flag `+$C6`. Negative: counts up,
    /// then loads a positive time. An armed object that is off screen, free and on the ground mirrors its point round itself and runs
    /// back (state 0x4A, 0x3F3D8).
    fn enemy_c13_3e1fa(&mut self, e: &mut Enemy) {
        if e.m.w(FLEE_T) == 0 {
            return;
        }
        let skip = e.b.act == 0x4A
            || e.b.act == 0x4C
            || e.m.w(FLEE_FLAG) == 0
            || e.m.b(BUSY) != 0
            || e.b.airborne != 0
            || !self.lib_off_screen(&e.b);
        if !skip {
            let x = e.b.xi();
            let d0 = (e.m.w(DEST_X) as i16).wrapping_sub(x).wrapping_neg().wrapping_add(x);
            e.m.sw(DEST_X, d0 as u16);
            e.m.sw(FLEE_T, 0x3C);
            return self.enemy_c13_3f3d8(e);
        }
        let d0 = (self.rand() & 0x1E) as u32; // 0x3E254
        let t = e.m.w(FLEE_T) as i16;
        if t < 0 {
            let n = t.wrapping_add(1);
            e.m.sw(FLEE_T, n as u16);
            if n == 0 {
                let v = self.assets.enemy.t.u16(t::FLEE_POS, d0);
                e.m.sw(FLEE_T, v);
            }
        } else {
            let n = t.wrapping_sub(1);
            e.m.sw(FLEE_T, n as u16);
            if n == 0 {
                let v = self.assets.enemy.t.u16(t::FLEE_NEG, d0);
                e.m.sw(FLEE_T, v);
                e.m.sw(FLEE_FLAG, 1);
            }
        }
    }

    /// 0x3E4A2: every 0xB4 frames a new slot (14 in 16 draws), then face the target unless busy.
    fn enemy_c13_3e4a2(&mut self, e: &mut Enemy) {
        if e.m.b(SLOT_T) == 0 {
            e.m.sb(SLOT_T, 0xB4);
            let r = self.rand() & 0xF;
            if r < 0xE {
                let _ = self.lib_slot6_new(e); // 0x32C5E
                if e.m.b(BUSY) == 0 {
                    self.lib_face_target(e); // 0x32B68
                }
            }
        }
        e.m.sb(SLOT_T, e.m.b(SLOT_T).wrapping_sub(1));
    }

    /// 0x3E4DC: when the object arrived at its point (`+$A3`), choose the pause `+$A2` by the level.
    pub(super) fn enemy_c13_3e4dc(&mut self, e: &mut Enemy) {
        if e.m.b(ARRIVED) == 0 {
            return;
        }
        // d1 = the level word, `add.b d2,d1` with d2 = the variant byte asr 5.
        let d2 = ((e.b.var26 >> 8) as u8 as i8 >> 5) as u8;
        let level = self.w.level;
        let idx = (level & 0xFF00) | (level as u8).wrapping_add(d2) as u16;
        let v = self.assets.enemy.t.u8(t::PAUSE_AT_POINT, idx as u32);
        e.m.sb(STILL_T, v);
        e.m.sb(ARRIVED, 0);
    }

    /// 0x3E420: the attack start tests of a free object (`+$A1` = 0): face the target; a target within 0x40 px in X and level in Z starts
    /// the close in (0x3F5E2); a farther one, on screen, in the negative phase of the flee timer, once in ten frames (`+$AA`), level
    /// in Z, starts the far decision (0x40118).
    fn enemy_c13_3e420(&mut self, e: &mut Enemy) {
        let n = e.m.b(TICK).wrapping_sub(1);
        e.m.sb(TICK, n);
        if n == 0 {
            e.m.sb(TICK, 0xA);
        }
        if e.m.b(BUSY) != 0 {
            return;
        }
        if e.m.b(ATTACKING) != 0xE {
            self.lib_face_target(e); // 0x32B68
        }
        let t = self.lib_target(e);
        let (tx, tz) = (t.xi(), t.zi());
        let d0 = tx.wrapping_add(0x40).wrapping_sub(e.b.xi()) as u16;
        if d0 <= 0x80 {
            let d0 = tz.wrapping_add(7).wrapping_sub(e.b.zi()) as u16;
            if d0 < 0xA {
                self.enemy_c13_3f5e2(e);
            }
            return;
        }
        // 0x3E474
        if self.lib_off_screen(&e.b) {
            return;
        }
        if e.m.b(TICK) != 1 {
            return;
        }
        if (e.m.w(FLEE_T) as i16) >= 0 {
            return;
        }
        let d0 = tz.wrapping_sub(e.b.zi()) as u16;
        if d0 < 8 {
            self.enemy_c13_40118(e);
        }
    }

    // ------------------------------------------------------------------ the guard

    /// True when the pool A object `h` is a POACHER J. / SKINNER / GUTTER (the entries 8, 0x48, 0x4C of the table 0x10B50).
    pub fn enemy_has_guard_c13(&self, h: Handle) -> bool {
        match crate::game::slot_of(h) {
            Some(crate::game::Slot::Enemy(i)) => matches!(self.enemies.slots[i].m.w(W84), 0x08 | 0x48 | 0x4C),
            _ => false,
        }
    }

    /// 0x10B50 table entry of the poachers (0x3DFC6), called by the player's hit pass 0x109C2 with the poacher as a6: true = carry = blocked
    /// (the pass then plays 0x9732 and spawns the spark of type 2).
    pub fn enemy_guard_c13(&mut self, h: Handle) -> bool {
        let Some(crate::game::Slot::Enemy(i)) = crate::game::slot_of(h) else { return false };
        let mut e = std::mem::take(&mut self.enemies.slots[i]);
        let r = self.enemy_c13_3dfc6(&mut e);
        self.enemies.slots[i] = e;
        r
    }

    /// 0x3DFC6: does the poacher block the hit? Only when on the ground (`+$0C` word = 0), in sub 0, not committed, with the target facing
    /// the other way. The counter `+$BE` is 0 (ready), positive (guarding) or negative (pause after a guard).
    pub(super) fn enemy_c13_3dfc6(&mut self, e: &mut Enemy) -> bool {
        if (e.b.y >> 16) as i16 != 0 || e.b.sub != 0 || e.m.b(COMMIT) != 0 {
            return false;
        }
        if self.lib_target(e).facing == e.b.facing {
            return false;
        }
        let cnt = e.m.b(COUNT) as i8;
        if cnt < 0 {
            return false;
        }
        let variant = Self::enemy_c13_variant(e);
        let aggr = self.rank.aggression as u32;
        if cnt != 0 {
            // 0x3E068: already guarding: the roll below the table byte keeps guarding, else a pause (the counter goes negative).
            let r = self.rand() & 0xF;
            let d1 = self.assets.enemy.t.u8(t::GUARD_AGAIN, aggr + (variant << 2)) as u16;
            if r < d1 {
                e.m.sb(LUNGE, 1); // 0x3E058
                return true;
            }
            let w = self.assets.enemy.t.u16(t::GUARD_PAUSE, aggr * 2 + (variant << 3));
            e.m.sb(COUNT, w.wrapping_neg() as u8);
            return false;
        }
        // 0x3DFFC: the random number is drawn and then thrown away (`moveq #0,d0` before the compare): the table byte decides alone.
        let _ = self.rand();
        let d1 = self.assets.enemy.t.u8(t::GUARD_CHANCE, aggr + (variant << 2)) as i8;
        if d1 <= 0 {
            return false;
        }
        let base = t::GUARD_COUNT + (variant >> 1) + (variant << 4);
        let first = self.assets.enemy.t.u8(base, 0);
        let r = (self.rand() & 0x1F) as u32;
        let add = self.assets.enemy.t.u8(base, 1 + r);
        e.m.sb(COUNT, first.wrapping_add(add));
        e.m.sb(LUNGE, 1);
        true
    }

    // ------------------------------------------------------------------ death

    /// 0x401CE (mode 4): sub 0 sets `+$80` = 0x3C, gives the kill score (0x325E6) and goes on in sub 2, which counts `+$80` down to 0 (drawn on even counts)
    /// and frees the object.
    fn enemy_c13_dying(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => {
                e.m.sw(T80, 0x3C); // 0x401DE
                let b = e.b.clone();
                self.enemy_award_kill_score(&b); // 0x401E4
                e.b.sub += 2;
                self.enemy_c13_dying_blink(e, h);
            }
            2 => self.enemy_c13_dying_blink(e, h),
            s => panic!("class 13 mode 4 sub {s:#x} (the table 0x401DA has two entries)"),
        }
    }

    /// 0x401EE.
    fn enemy_c13_dying_blink(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(T80);
        if t == 0 {
            let i = self.enemy_index(h);
            return self.enemy_free_pool_a(e, i); // 0x40208
        }
        let n = t - 1;
        e.m.sw(T80, n);
        if n & 1 == 0 {
            self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
        }
    }

    // ------------------------------------------------------------------ entrances (table 0x3DF54)

    /// 0x3E504 (entrance 0 and the recover entry of the table 0x2B56C for +$84 = 8 / 0x48 / 0x4C): walk in. State 2, slot, target, then
    /// a decision.
    pub(super) fn enemy_c13_3e504(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0000);
        self.enemy_anim(&mut e.b, 1);
        self.enemy_set_speed(&mut e.b, 6);
        e.m.sb(ATTACKING, 0);
        let _ = self.lib_slot6_new(e); // 0x32C5E
        let tg = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, tg);
        e.m.sw(TIMER, 0x14);
        e.m.sb(BUSY, 0);
        self.lib_face_target(e);
        self.enemy_c13_3f670(e);
        self.enemy_c13_4005c(e);
    }

    /// 0x3FC3E (entrance 2): stand in state 0x34 for 0x4B0 frames (anim 5).
    fn enemy_c13_3fc3e(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 1);
        e.m.sw(TIMER, 0x4B0);
        e.b.act = 0x34;
        self.enemy_anim(&mut e.b, 5);
        self.enemy_set_speed(&mut e.b, 6);
    }

    /// 0x3FCA4 (entrance 4): rise from behind a hiding place in state 0x36: 60 frames in anim 7 with the palette override 0x8B and the fade
    /// palettes (0x3EFD0), Z + 0x10. The state ends with the override 0x8F / 0x90 / 0x91 by the variant (0x3EF5E).
    fn enemy_c13_3fca4(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 1);
        e.m.sw(TIMER, 0x3C);
        e.b.act = 0x36;
        e.b.palette_override = 0x8B;
        self.enemy_c13_3efd0(e, 0);
        self.enemy_anim(&mut e.b, 7);
        e.m.sl(VZ, 0xFFFF_E000);
        e.b.set_zi_keep(e.b.zi().wrapping_add(0x10));
        self.enemy_set_speed(&mut e.b, 6);
    }

    /// 0x3EFD0: copy step `d2` (0, 4, 8, 0xC) of the fade palettes of the variant into the object palette 0x914160, keeping the high
    /// nibble of each colour word.
    pub(super) fn enemy_c13_3efd0(&mut self, e: &Enemy, d2: u32) {
        let v = Self::enemy_c13_variant(e) as u32;
        let tb = &self.assets.enemy.t;
        let dest = tb.u32(t::FADE_DEST, v * 2);
        let src = tb.u32(t::FADE_SRC, v * 8 + d2);
        let base = ((dest - 0x914000) / 2) as usize;
        for k in 0..16usize {
            let w = self.pal.words[base + k];
            self.pal.words[base + k] = (w & 0xF000) | tb.u16(src, 2 * k as u32);
        }
        self.pal.dirty = true;
    }

    /// 0x3FCE0 (entrance 6): jump in from 144 px above (state 0x38, 60 frames of fall).
    fn enemy_c13_3fce0(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 1);
        e.m.sb(COMMIT, 1);
        e.m.sw(TIMER, 0x3C);
        e.b.airborne = 1;
        e.b.act = 0x38;
        self.enemy_anim(&mut e.b, 6);
        self.enemy_set_speed(&mut e.b, 6);
        e.b.set_yi_keep(e.b.yi().wrapping_add(0x90));
        e.b.vy = 0;
        e.b.ay = 0xFFC8u16 as i16;
        e.b.vx = 0;
        e.b.ax = 0;
        e.b.vz = 0;
        e.b.az = 0;
        self.lib_face_target(e);
    }

    /// 0x3FF26 (entrance 0xA): dash in from the screen edge at the target's depth (attack 0x2C, state 0x1C, 45 frames): the right edge
    /// (camera X + 0x180), the left one (camera X - 0x10) when the target is on the right part of the screen.
    fn enemy_c13_3ff26(&mut self, e: &mut Enemy) {
        self.enemy_begin_attack(&mut e.b, 0x2C);
        let t = self.lib_target(e);
        let (tx, tz) = (t.xi(), t.zi());
        put_hi(&mut e.b.z, tz);
        let cam = self.cam.s2.cam_x as i16;
        put_hi(&mut e.b.x, cam);
        e.m.sb(BUSY, 1);
        e.b.act = 0x1C;
        e.m.sw(TIMER, 0x2D);
        self.enemy_set_speed(&mut e.b, 0xD);
        self.lib_aim_at_target(e); // 0x32A58
        e.b.facing = 0;
        e.b.set_xi_keep(e.b.xi().wrapping_add(0x180));
        e.b.dir = 0x18;
        if tx.wrapping_sub(cam) <= 0xC0 {
            // bgt 0x3FFA6 is signed: a target at or below 0xC0 on screen takes the left edge
            e.b.dir = 8;
            e.b.facing = 1;
            e.b.set_xi_keep(e.b.xi().wrapping_sub(0x190));
        }
        self.enemy_anim(&mut e.b, 0x15);
    }

    /// 0x3FFAE (entrance 0xE): dash at the depth of the target from where it stands (attack 0x2C, state 0x1C, 45 frames).
    fn enemy_c13_3ffae(&mut self, e: &mut Enemy) {
        self.enemy_begin_attack(&mut e.b, 0x2C);
        let t = self.lib_target(e);
        let (tx, tz) = (t.xi(), t.zi());
        put_hi(&mut e.b.z, tz);
        e.m.sb(BUSY, 1);
        e.b.act = 0x1C;
        e.m.sw(TIMER, 0x2D);
        self.enemy_set_speed(&mut e.b, 0xD);
        self.lib_aim_at_target(e); // 0x32A58
        e.b.facing = 0;
        e.b.dir = 0x18;
        if tx >= e.b.xi() {
            e.b.dir = 8;
            e.b.facing = 1;
        }
        self.enemy_anim(&mut e.b, 0x15);
    }

    /// 0x4001C (entrance 0xC): run in to the slot point (state 0x3E, 36 frames at speed id 0xD, anim 3).
    fn enemy_c13_4001c(&mut self, e: &mut Enemy) {
        e.b.act = 0x3E;
        e.m.sw(TIMER, 0x24);
        e.m.sb(AIM_T, 0x1E);
        let (d1, d2) = (e.m.w(DEST_X), e.m.w(DEST_Z));
        e.b.dir = Self::enemy_angle_to(&e.b, d1, d2); // 0x103A
        let _ = self.lib_slot6_new(e);
        self.lib_face_target(e);
        self.enemy_set_speed(&mut e.b, 0xD);
        self.enemy_anim(&mut e.b, 3);
    }

    /// The number of steps of 0x3800 in `d0` as the loop `d1 = 0; do { d1 += 1; d0 -= 0x3800 } while d0 >= 0` leaves it (a word).
    pub(super) fn enemy_c13_steps(mut d0: i32) -> u16 {
        let mut d1 = 0u16;
        loop {
            d1 = d1.wrapping_add(1);
            d0 = d0.wrapping_sub(0x3800);
            if d0 < 0 {
                return d1;
            }
        }
    }

    /// 0x3FD3E (entrance 0x10): jump on from the screen edge nearest the target (state 0x4E, 60 frames, anim 9), arriving 0x80 px beside
    /// it. The start is 0x80 px left of the target, 0x80 px right of it when the target is within 0xC0 px of the left screen edge.
    fn enemy_c13_3fd3e(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 1);
        e.m.sb(COMMIT, 1);
        e.m.sw(TIMER, 0x3C);
        e.b.airborne = 1;
        e.b.act = 0x4E;
        self.enemy_anim(&mut e.b, 9);
        self.enemy_set_speed(&mut e.b, 6);
        let t = self.lib_target(e);
        let (tx_l, tz_l) = (t.x, t.z);
        let cam = ((self.cam.s2.cam_x as u32) << 16 | self.cam.s2.cam_x_frac as u32) as i32;
        let mut d0 = tx_l;
        let d1 = d0.wrapping_sub(cam);
        d0 = d0.wrapping_sub(0x80_0000);
        if d1 < 0xC0_0000 {
            d0 = d0.wrapping_add(0x100_0000);
        }
        e.b.x = d0; // 0x3FD9A
        let d1 = e.b.x.wrapping_sub(cam);
        e.b.x = e.b.x.wrapping_add(0x30_0000);
        if d1 < 0xC0_0000 {
            e.b.x = e.b.x.wrapping_sub(0x60_0000);
        }
        e.m.sl(FROM_X, d0 as u32);
        let diff = if d0 < e.b.x { e.b.x.wrapping_sub(d0) } else { d0.wrapping_sub(e.b.x) };
        e.b.vx = Self::enemy_c13_steps(diff) as i16;
        e.m.sl(FROM_Z, tz_l as u32);
        let d0 = tz_l;
        let diff = if d0 < e.b.z { e.b.z.wrapping_sub(d0) } else { d0.wrapping_sub(e.b.z) };
        let mut d1 = Self::enemy_c13_steps(diff);
        if (d1 as i16) > 0xC0 {
            d1 = 0xC0;
        }
        e.b.vz = d1 as i16;
        e.b.vy = 0x900;
        e.b.ax = 0;
        e.b.ay = 0xFFC0u16 as i16;
        e.b.az = 0;
        put_hi(&mut e.b.z, 0);
        let start = e.m.l(FROM_X) as i32;
        if !(start >= e.b.x) {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        self.lib_face_target(e);
    }

    /// 0x3FE60 (entrance 0x12): the ambush: jump on from the far side (state 0x4E, 60 frames), arriving 0x30 px beside the target. The
    /// compare against the camera uses `sub.w $744(a5),d1` on the fraction of X (a quirk of the original: the fraction is 0 in the scripts).
    fn enemy_c13_3fe60(&mut self, e: &mut Enemy) {
        e.m.sb(BUSY, 1);
        e.m.sb(COMMIT, 1);
        e.m.sw(TIMER, 0x3C);
        e.b.airborne = 1;
        e.b.act = 0x4E;
        self.enemy_anim(&mut e.b, 9);
        self.enemy_set_speed(&mut e.b, 6);
        e.m.sl(FROM_Z, e.b.z as u32);
        let d0 = e.b.x;
        e.m.sl(FROM_X, d0 as u32);
        let d2 = d0;
        // d1 = d0 (long), `sub.w camera,d1`: the low word (the fraction of X) minus the integer camera X.
        let d1 = (d0 as u16).wrapping_sub(self.cam.s2.cam_x) as i16;
        e.b.set_xi_keep(e.b.xi().wrapping_add(0x30));
        if d1 < 0xC0 {
            e.b.set_xi_keep(e.b.xi().wrapping_sub(0x60));
        }
        let diff = if d0 < e.b.x { e.b.x.wrapping_sub(e.m.l(FROM_X) as i32) } else { d0.wrapping_sub(e.b.x) };
        e.b.vx = Self::enemy_c13_steps(diff) as i16;
        let d1 = Self::enemy_c13_steps(e.m.l(FROM_Z) as i32);
        e.b.vz = ((d1 as i16) >> 1) as i16;
        e.b.vy = 0x900;
        e.b.ax = 0;
        e.b.ay = 0xFFC0u16 as i16;
        e.b.az = 0;
        put_hi(&mut e.b.z, 0);
        if !(d2 >= e.b.x) {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        self.lib_face_target(e);
    }
}
