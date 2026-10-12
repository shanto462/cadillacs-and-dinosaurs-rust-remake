//! Pool A class 14 (`+$20 = 0x38`, handler 0x4020E): BLADE (name 6, variant 0) and RAZOR (name 0x14,
//! variant 2). Phase 1 task 4a2. Knife thugs with a state machine in `+$06` (0..0x3A, 30 handlers
//! listed in the table 0x407F8; `class14_states.rs`), state entry routines and the two decisions
//! (`class14_entry.rs`), and three objects per script record: the init (0x40224) takes two more pool A
//! slots, sorts the three handles and turns the middle one into the real object; the other two become
//! class 60 parts that follow it (`class14_parts.rs`, also the thrown knives, classes 34 and 41).
//!
//! The entrance (low byte of `+$26`): 0 walk in, 2 / 4 stand and wait (0x4B0 / 0x1E0 frames), 6 jump in
//! from above, 8 jump in at the target, 0xA / 0xC jump in with two knives (0x403E8).
//!
//! Every routine says its original address. A routine that ends in a carry or a flag in the original
//! returns `bool` here (true = carry set / condition taken).

use super::lib32::put_hi;
use super::{Enemy, off::*};
use crate::game::{Game, Handle};

/// Offsets (this class' meaning). The harness trace has these bytes as plain `r..` bytes.
pub(super) mod f {
    pub const BUSY: usize = 0xA0; // byte: 1 = committed to the state, no new decision, no turning
    pub const KNIVES_TO_THROW: usize = 0xA1; // byte: throws left of the run at the target
    pub const KNIFE_LATCH: usize = 0xA2; // byte: a knife was dropped in this hit reaction
    pub const TURN_T: usize = 0xA3; // byte: frames until the next turn towards the aim
    pub const OLD_DIR: usize = 0xA4; // byte: the direction before the turn limit
    pub const DIST: usize = 0xA5; // byte: X distance class of the target (0x3293C)
    pub const TIMER: usize = 0xA6; // word: frames left in the state
    pub const PART_A: usize = 0xA8; // word: handle of the first part
    pub const PART_B: usize = 0xAA; // word: handle of the second part
    pub const PAUSE: usize = 0xAC; // byte: frames to stand still before the next step
    pub const MOVED: usize = 0xAD; // byte: 1 = the object reached its point (the pause is chosen next)
    pub const SLOT_T: usize = 0xAE; // byte: frames until the next slot is drawn
    pub const TICK: usize = 0xAF; // byte: 0xA..1 counter
    pub const VX: usize = 0xB0; // long (16.16): X step of the sidestep states
    pub const VZ: usize = 0xB4; // long (16.16): Z step
    pub const FLEE_T: usize = 0xB8; // word (signed): timer of the "run off screen" idea
    pub const BLOCK: usize = 0xBA; // byte: 1 = the guard blocked a hit (start the back step)
    pub const ATK_NO: usize = 0xBB; // byte: attack kind (animation row)
    pub const ATK_STEP: usize = 0xBC; // byte: attacks left of the chain
    pub const KNIVES: usize = 0xBE; // byte: knives in the hands (0..2); negative = reset by the parts routine
    pub const COOL: usize = 0xBF; // byte (signed): guard timer, > 0 blocking, < 0 pause after the block
    pub const FLAG_C0: usize = 0xC0; // word: 1 = the flee timer may count (set after the first wait)
    pub const SPEED_C4: usize = 0xC4; // long: the speed chosen at the start of state 0x18
    pub const HOP_C8: usize = 0xC8; // byte: the wall hop of state 0x22 goes sideways
}

impl Game {
    /// 0x4020E class 14.
    pub fn enemy_class14(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => self.c14_init(e, h),
            2 => self.c14_alive(e, h),
            4 => self.c14_dying(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x41E90
            }
        }
    }

    // ------------------------------------------------------------------ init

    /// 0x40224: sub 0 makes the three objects, any other sub is the real init (0x402E4).
    fn c14_init(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.sub != 0 {
            return self.c14_init2(e, h);
        }
        let i = self.enemy_index(h);
        if self.enemies.free_count() < 2 {
            return self.enemy_free_pool_a(e, i); // 0x41E90
        }
        let Some(j1) = self.enemies.alloc() else { return self.enemy_free_pool_a(e, i) };
        let Some(j2) = self.enemies.alloc() else { return self.enemy_free_pool_a(e, i) };
        // The object under update goes back into its slot while the three are set up.
        self.enemies.slots[i] = std::mem::take(e);
        let (mut a0, mut a1, mut a2) = (h, self.enemies.handle(j1), self.enemies.handle(j2));
        // The addresses are sign extended words in the original (all pools sit above 0x8000), so the
        // signed long compares order them like unsigned words.
        if a1 < a0 {
            std::mem::swap(&mut a0, &mut a1);
        }
        if a2 < a1 {
            std::mem::swap(&mut a2, &mut a1);
        }
        if a1 < a0 {
            std::mem::swap(&mut a0, &mut a1);
        }
        let src = self.enemies.slots[i].clone();
        // The middle one is the object itself.
        {
            let s1 = self.enemy_index(a1);
            let n = &mut self.enemies.slots[s1];
            n.b.live = 1;
            n.b.k20 = 0x38;
            n.b.var26 = src.b.var26;
            n.m.sb(DROP, src.m.b(DROP));
            n.b.dir = src.b.dir; // move.w $22(a6),$22(a1): direction and palette override
            n.b.palette_override = src.b.palette_override;
            put_hi(&mut n.b.x, src.b.xi());
            put_hi(&mut n.b.y, src.b.yi());
            put_hi(&mut n.b.z, src.b.zi());
            n.m.sw(f::PART_A, a0);
            n.m.sw(f::PART_B, a2);
            n.b.sub = 2;
        }
        // The other two are the parts (class 60): +$26 = 0 and 0x200, +$A0 = the master (a long).
        for (a, var) in [(a0, 0u16), (a2, 0x200u16)] {
            let sa = self.enemy_index(a);
            let n = &mut self.enemies.slots[sa];
            n.b.live = 1;
            n.b.k20 = 0xF0;
            n.b.var26 = var;
            n.b.anim_table = 0x6FB6A;
            n.m.sl(0xA0, 0xFFFF_0000 | a1 as u32);
        }
        *e = std::mem::take(&mut self.enemies.slots[i]);
    }

    /// 0x402E4: the init of the real object (sub 2).
    fn c14_init2(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sb(B87, 1);
        e.m.sb(f::BLOCK, 0);
        let (id, w84, table) = if (e.b.var26 >> 8) as u8 == 0 { (6, 0x18, 0x6DA40) } else { (0x14, 0x50, 0x6DF2A) };
        e.b.id = id;
        e.m.sw(W84, w84);
        e.b.anim_table = table;
        e.b.param_b = 1;
        self.enemy_anim(&mut e.b, 1);
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_set_speed(&mut e.b, 7);
        e.b.dir = 0x18;
        self.enemy_set_hp(&mut e.b); // 0x31424, d0 = 0
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        e.b.stun = 10;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.m.sb(ATTACKING, 0);
        e.b.b72 = 0xC8;
        e.m.sw(f::TIMER, 0x14);
        e.m.sb(f::KNIVES, 0);
        e.m.sb(f::KNIFE_LATCH, 0);
        e.m.sb(WAIT, 0);
        e.m.sw(f::FLAG_C0, 0);
        e.m.sw(f::FLEE_T, 0xFED4);
        e.m.sb(f::TICK, 0x1E);
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sb(ATTACKING, 0);
        e.b.b63 = 0;
        e.b.set_state(0x0200_0000);
        e.m.sb(ATTACKING, 0);
        self.lib_slot3_new(e); // 0x32D90
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        self.lib_aim_at_target(e); // 0x32A70
        e.m.sb(f::BUSY, 0);
        // 0x403E8: the entrance by the low byte of +$26.
        match e.b.var26 & 0xFF {
            0 => self.c14_41954(e),
            2 => self.c14_41802(e),
            4 => self.c14_41824(e),
            6 => self.c14_41846(e),
            8 => self.c14_418b4(e),
            0xA => self.c14_419ac(e),
            0xC => self.c14_41a1c(e),
            v => panic!("class 14 entrance {v:#x} (object {h:#06x})"),
        }
    }

    // ------------------------------------------------------------------ alive frame

    /// 0x403F6: alive. Sub 0 the AI (0x40700), 2 the hit reaction (0x2A606), 6 held by a player
    /// (0x2B35A); then the knife drop (0x40428), the parts (0x41D6E) and the play field clamp (0x193E).
    fn c14_alive(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_kill_all_check(e); // 0x326F8
        match e.b.sub {
            0 => self.c14_ai(e, h),
            2 => self.enemy_hit_reaction(e, h),
            6 => self.enemy_grabbed(e, h),
            s => panic!("class 14 sub {s:#x} (0x41E38 through the long table 0x40418 is never entered)"),
        }
        self.c14_40428(e, h);
        self.c14_41d6e(e);
        let i = self.enemy_index(h);
        self.enemy_clamp_to_playfield(e, i); // jmp 0x193E
    }

    /// 0x407EC: the end of the AI frame, registered as drawn and hurtable.
    fn c14_ai_tail(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_draw_check(&mut e.b, h); // 0x49CA
        self.enemy_push_hurtable(h); // 0x4BEA
    }

    /// 0x40700: the AI frame.
    fn c14_ai(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.b(f::BLOCK) != 0 {
            e.m.sb(f::BLOCK, 0);
            self.c14_41646(e); // the back step after a block
        }
        let p = e.m.b(POISE_T);
        if p != 0 {
            e.m.sb(POISE_T, p - 1);
            if p - 1 == 0 {
                e.b.stun = 10;
            }
        }
        self.enemy_floor_collision(e); // 0x12CB4
        if e.b.hitstop != 0 {
            e.b.hitstop -= 1;
            e.b.timer = 1;
            return self.c14_ai_tail(e, h);
        }
        let w = e.m.b(WAIT);
        if w != 0 {
            e.m.sb(WAIT, w - 1);
            return self.c14_ai_tail(e, h);
        }
        self.c14_404b0(e);
        let st = e.b.act;
        let mut check_air = true; // 0x40786
        let mut run_state = false; // 0x407A2
        if st != 0x38 && st != 0x36 {
            check_air = false;
            if e.m.b(f::TICK) & 7 != 0 {
                run_state = true; // bne 0x407A2
            } else if e.m.b(f::BUSY) != 0 {
                // bne 0x4078E: skip the wall and air tests
            } else if e.b.wall == 0x10 {
                return self.c14_4146a(e); // hop down a ledge (state 0x26 with +$C8 = 1)
            } else {
                check_air = true;
            }
        }
        if !run_state {
            if check_air && e.b.airborne != 0 {
                return self.c14_41470(e);
            }
            // 0x4078E
            self.lib_slot3_keep(e); // 0x32D86
            self.lib_check_target(e); // 0x32F56
            self.c14_408dc(e);
            self.c14_408ba(e);
        }
        // 0x407A2: the state handler.
        self.c14_state(e, h);
        self.c14_40834(e);
        self.c14_405ca(e);
        self.c14_4062c(e);
        if e.m.b(f::TICK) & 1 == 0 && e.m.b(f::BUSY) == 0 && e.b.act != 8 {
            let t = self.lib_target(e);
            if t.mode == 2 && t.sub == 2 {
                self.c14_412f6(e); // the target is in a hit reaction: back off
            }
        }
        self.c14_ai_tail(e, h);
    }

    /// 0x40428: a hit Blade that is in the air drops a knife (class 34) while he still holds one.
    fn c14_40428(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.b(f::KNIVES) == 0 {
            return;
        }
        if (e.b.mode, e.b.sub) != (2, 2) || e.b.act == 8 || e.b.airborne == 0 {
            e.m.sb(f::KNIFE_LATCH, 0); // 0x404A8
            return;
        }
        if e.m.b(f::KNIFE_LATCH) != 0 {
            return;
        }
        if (self.spawn.g.b4fa as i8) >= 2 {
            return;
        }
        e.m.sb(f::KNIFE_LATCH, 1);
        let Some(j) = self.enemies.alloc() else { return };
        let n = &mut self.enemies.slots[j];
        n.b.live = 1;
        n.b.k20 = 0x88; // class 34
        n.b.var26 = 0;
        n.b.facing = e.b.facing;
        put_hi(&mut n.b.x, e.b.xi());
        put_hi(&mut n.b.y, e.b.yi().wrapping_add(0x40));
        put_hi(&mut n.b.z, e.b.zi());
        n.m.sl(0xA0, 0xFFFF_0000 | h as u32); // +$A0: the master (a long)
        e.m.sb(f::KNIVES, e.m.b(f::KNIVES).wrapping_sub(1));
    }

    /// 0x404B0: the guard timer counts to 0 from either side.
    fn c14_404b0(&mut self, e: &mut Enemy) {
        let v = e.m.b(f::COOL) as i8;
        if v > 0 {
            e.m.sb(f::COOL, (v - 1) as u8);
        } else if v < 0 {
            e.m.sb(f::COOL, (v + 1) as u8);
        }
    }
}
