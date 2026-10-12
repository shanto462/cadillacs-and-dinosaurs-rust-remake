//! Class 14: the small routines of the alive frame (slot, pause, flee timer, tick), the guard
//! (0x404CA, called by the player's hit pass) and the death (0x41E38). See `class14.rs`.

use super::class14::f;
use super::{Enemy, off::*};
use crate::game::{Game, Handle, Slot, slot_of};

impl Game {
    /// 0x405CA: every 0xB4 frames a new slot (14 in 16 times), then face the target unless busy.
    pub(super) fn c14_405ca(&mut self, e: &mut Enemy) {
        if e.m.b(f::SLOT_T) == 0 {
            e.m.sb(f::SLOT_T, 0xB4);
            let r = self.rand() & 0xF;
            if r < 0xE {
                self.lib_slot3_new(e); // 0x32D90
                if e.m.b(f::BUSY) == 0 {
                    self.lib_face_target(e); // 0x32B68
                }
            }
        }
        e.m.sb(f::SLOT_T, e.m.b(f::SLOT_T).wrapping_sub(1));
    }

    /// 0x40604: when the object reached its point (`+$AD`), choose the pause `+$AC` by the level.
    pub(super) fn c14_40604(&mut self, e: &mut Enemy) {
        if e.m.b(f::MOVED) == 0 {
            return;
        }
        // d1 = level word, then `add.b d2,d1` with d2 = the variant byte asr 5.
        let d2 = ((e.b.var26 >> 8) as u8 as i8 >> 5) as u8;
        let idx = (self.w.level as u8).wrapping_add(d2) as u32;
        let v = self.assets.enemy.t.u8(0x7441A, idx);
        e.m.sb(f::PAUSE, v);
        e.m.sb(f::MOVED, 0);
    }

    /// 0x4062C: the "run off screen" timer `+$B8` (positive: counts down to a negative wait, negative:
    /// counts up to a positive one). An object that is off screen, free to move and past its first
    /// cycle turns the point round and runs (state 0x36, 0x41A96).
    pub(super) fn c14_4062c(&mut self, e: &mut Enemy) {
        if e.m.w(f::FLEE_T) == 0 {
            return;
        }
        let st = e.b.act;
        let onscreen_or_skip = st == 0x36
            || st == 0x38
            || e.m.w(f::FLAG_C0) == 0
            || e.m.b(f::BUSY) != 0
            || e.b.airborne != 0
            || !self.lib_off_screen(&e.b);
        if !onscreen_or_skip {
            let x = e.b.xi();
            let d0 = (e.m.w(DEST_X) as i16).wrapping_sub(x).wrapping_neg().wrapping_add(x);
            e.m.sw(DEST_X, d0 as u16);
            e.m.sw(f::FLEE_T, 0x3C);
            return self.c14_41a96(e);
        }
        // 0x40686
        let d0 = (self.rand() & 0x1E) as u32;
        let t = e.m.w(f::FLEE_T) as i16;
        if t < 0 {
            let n = t.wrapping_add(1);
            e.m.sw(f::FLEE_T, n as u16);
            if n == 0 {
                let v = self.assets.enemy.t.u16(0x406C0, d0);
                e.m.sw(f::FLEE_T, v);
            }
        } else {
            let n = t.wrapping_sub(1);
            e.m.sw(f::FLEE_T, n as u16);
            if n == 0 {
                let v = self.assets.enemy.t.u16(0x406E0, d0);
                e.m.sw(f::FLEE_T, v);
                e.m.sw(f::FLAG_C0, 1);
            }
        }
    }

    /// 0x40834: the tick `+$AF` (0xA..1), face the target, and two checks that start an action: a
    /// target close in X and level in Z (0x4178E: stand and wait), or a target further away in a
    /// gap of the flee timer (0x41C8C).
    pub(super) fn c14_40834(&mut self, e: &mut Enemy) {
        let tk = e.m.b(f::TICK).wrapping_sub(1);
        e.m.sb(f::TICK, if tk == 0 { 0xA } else { tk });
        if e.m.b(f::BUSY) != 0 {
            return;
        }
        self.lib_face_target(e);
        if (e.m.b(f::DIST) as i8) <= 1 {
            let d0 = self.lib_target(e).zi().wrapping_add(9).wrapping_sub(e.b.zi()) as u16;
            if d0 > 0xA {
                return;
            }
            return self.c14_4178e(e);
        }
        // 0x40876
        if self.lib_off_screen(&e.b) {
            return;
        }
        if e.b.act == 0x36 || e.b.act == 0x38 || e.m.b(f::TICK) != 1 {
            return;
        }
        if (e.m.w(f::FLEE_T) as i16) >= 0 {
            return;
        }
        let d0 = self.lib_target(e).zi().wrapping_add(7).wrapping_sub(e.b.zi()) as u16;
        if d0 <= 0xA {
            self.c14_41c8c(e);
        }
    }

    /// 0x408BA: a pending `+$7A` wait turns into state 0xE (never true after 0x4073E).
    pub(super) fn c14_408ba(&mut self, e: &mut Enemy) {
        let w = e.m.b(WAIT);
        if w == 0 {
            return;
        }
        e.m.sb(WAIT, w - 1);
        e.m.sw(f::TIMER, 0x14);
        e.b.act = 0xE;
        self.enemy_anim(&mut e.b, 0);
    }

    /// 0x408DC: wall bit 7 (an obstacle in the way) starts state 0x34 (0x41430).
    pub(super) fn c14_408dc(&mut self, e: &mut Enemy) {
        if e.b.wall & 0x80 != 0 {
            self.c14_41430(e);
        }
    }

    /// 0x408E8: recover from a hit or a grab (entry 0x18 / 0x50 of the table 0x2B56C): back to state
    /// 0x16 with the flee timer reset; after a reaction of kind 4 (a knock down) a new slot, target and
    /// aim first. Then the decision.
    pub(super) fn c14_408e8(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_1600);
        self.enemy_set_speed(&mut e.b, 5);
        self.lib_slot3_new(e);
        self.c14_41d6e(e);
        e.m.sw(f::FLEE_T, 0xFED4);
        if e.b.reaction != 4 {
            self.c14_41d6e(e);
            return self.c14_41be8(e);
        }
        e.m.sb(ATTACKING, 0);
        self.lib_slot3_new(e);
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        self.lib_aim_at_target(e);
        self.lib_face_target(e);
        self.c14_41be8(e);
    }

    // ------------------------------------------------------------------ guard

    /// 0x10B50 table entry of BLADE / RAZOR (0x404CA): does the object block the player's attack that
    /// hits it? Called by the player's hit pass 0x109C2 with the enemy as a6; true = carry = blocked
    /// (the pass then plays 0x9732 and spawns the spark of type 2). A block asks for the back step
    /// (`+$BA`, 0x41646) and sets the guard timer `+$BF`.
    pub fn enemy_guard_c14(&mut self, h: Handle) -> bool {
        let Some(Slot::Enemy(i)) = slot_of(h) else { return false };
        let mut e = std::mem::take(&mut self.enemies.slots[i]);
        let r = self.c14_404ca(&mut e);
        self.enemies.slots[i] = e;
        r
    }

    /// True when the pool A object `h` is a BLADE / RAZOR (the entries 0x18 / 0x50 of the guard table).
    pub fn enemy_has_guard_c14(&self, h: Handle) -> bool {
        match slot_of(h) {
            Some(Slot::Enemy(i)) => matches!(self.enemies.slots[i].m.w(W84), 0x18 | 0x50),
            _ => false,
        }
    }

    pub(super) fn c14_404ca(&mut self, e: &mut Enemy) -> bool {
        if e.b.yi() != 0 || e.b.sub != 0 || matches!(e.b.act, 0x22 | 0x14 | 0xE) {
            return false;
        }
        if self.lib_target(e).facing == e.b.facing {
            return false;
        }
        let variant = (e.b.var26 >> 8) as u32 & 0xFF;
        let aggr = self.rank.aggression as u32;
        let cool = e.m.b(f::COOL) as i8;
        if cool < 0 {
            return false;
        }
        if cool == 0 {
            // 0x40516
            let r = (self.rand() & 0xF) as i8;
            let d1 = self.assets.enemy.t.u8(0x74298, aggr + (variant << 2)) as i8;
            if d1 <= r {
                return false;
            }
            let base = 0x742A8 + (variant << 4);
            let first = self.assets.enemy.t.u8(base, 0);
            let r2 = (self.rand() & 0x1F) as u32;
            let add = self.assets.enemy.t.u8(base, 1 + r2);
            e.m.sb(f::COOL, first.wrapping_add(add));
            e.m.sb(f::BLOCK, 1);
            return true;
        }
        // 0x40578: already blocking: keep blocking, or end it with a pause.
        let r = (self.rand() & 0xF) as i16;
        let d1 = self.assets.enemy.t.u8(0x742EA, aggr + (variant << 2)) as i16;
        if r < d1 {
            e.m.sb(f::BLOCK, 1); // 0x40568
            return true;
        }
        let w = self.assets.enemy.t.u16(0x742FA, aggr * 2 + (variant << 3));
        e.m.sb(f::COOL, w.wrapping_neg() as u8);
        false
    }

    // ------------------------------------------------------------------ death

    /// 0x41E38 (mode 4): sub 0 draws one `rand()` (the 1 / 64 anti tamper call 0x2A5B6 never happens on a
    /// genuine ROM), gives the kill score and starts 60 frames of blinking; sub 2 blinks (drawn on odd
    /// counts) and frees the object.
    pub(super) fn c14_dying(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => {
                let _ = self.rand() & 0x3F; // 0x2A5B6: carry clear with a genuine ROM
                e.m.sw(T80, 0x3C);
                let b = e.b.clone();
                self.enemy_award_kill_score(&b); // 0x325E6
                e.b.sub += 2;
                self.c14_dying_blink(e, h);
            }
            _ => self.c14_dying_blink(e, h),
        }
    }

    /// 0x41E76.
    fn c14_dying_blink(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(T80);
        if t == 0 {
            let i = self.enemy_index(h);
            return self.enemy_free_pool_a(e, i); // 0x41E90
        }
        let n = t - 1;
        e.m.sw(T80, n);
        if n & 1 != 0 {
            self.enemy_draw_check(&mut e.b, h);
        }
    }
}
