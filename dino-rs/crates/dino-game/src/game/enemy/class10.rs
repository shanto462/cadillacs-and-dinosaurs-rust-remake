//! Pool A class 10 (`+$20 = 0x28`, handler 0x3B828): HAMMER T. (name 3), BLK ELMER (0xA) and
//! WRENCH T. (0xB). Phase 1 task 4a. Variant = high byte of `+$26` (0, 2, 4), entrance = low byte
//! (0 walk in, 2 drop, 4 / 6 jump attack, 8 wait until a player is near).
//!
//! The thug works like class 2 (state byte `+$06`, step byte `+$07`, approach slots round the
//! target) with its own attacks, a grab of the player (the `+$AC` machine: it takes the player
//! into the held state 0x02060000, action `+$C8` = 0x40) and a stomp on a downed player that
//! the player code hands over through the lists 0x6A5A / 0x6A8E and the flags `+$7E` / `+$7F`
//! (0x6606, see `enemy/claim.rs`).
//!
//! Every routine says its original address. Routines that return a carry in the original return
//! `bool` here (true = carry set). Field offsets that this class uses in its own way are named in
//! `mod f`; the harness trace uses the class 2 names for the same bytes.

use super::{Enemy, off::*};
use crate::game::{Game, Handle, Slot, slot_of};

/// Offsets (this class' meaning).
pub(super) mod f {
    pub const SLOT: usize = 0x7C; // approach slot 0..9 (byte)
    pub const CLAIM_A: usize = 0x7E; // set by 0x6606 for list A
    pub const CLAIM_B: usize = 0x7F; // set by 0x6606 for list B
    pub const T80: usize = 0x80; // byte timer
    pub const WALK_KIND: usize = 0xAA; // byte: 1 = in position for the walk (state 0xA)
    pub const HOLD_T: usize = 0xAB; // byte, used by the hold machine
    pub const HOLD_STATE: usize = 0xAC; // word: 0 = grab, 2 = stomp
    pub const HOLD_TIMER: usize = 0xAE; // word
    pub const START_X: usize = 0xB2; // word: X at the start of the walk in
    pub const T_B0: usize = 0xB0; // byte timer (slot change)
    pub const T_B1: usize = 0xB1; // byte timer (attack check)
    pub const HITS_B4: usize = 0xB4; // byte: swings left of a paired attack
    pub const OBST_T: usize = 0xB5;
    pub const WAIT_B6: usize = 0xB6; // word, entrance 8
    pub const PASSED_X: usize = 0xA2; // word, X of the last attack check
    pub const KIND: usize = 0xA4; // word: attack kind (row of the script table)
    pub const STEP: usize = 0xA6; // word: position in the script row, reused as +$A6 = 0 reset
    pub const SUB: usize = 0xA8; // word: sub state of the attack machine
}

/// Table addresses of this class.
pub(super) mod t {
    pub const ENTR8_WAIT: u32 = 0x3BAA6;
    pub const IDLE_TIMES: u32 = 0x3BD84;
    pub const ATTACK_PICK: u32 = 0x3C0EE;
    pub const SCRIPTS: u32 = 0x3C1E6;
    pub const GRAB_WAIT: u32 = 0x3C35A;
    pub const DASH_SPEEDS: u32 = 0x3C3DE;
    pub const RECOVER: u32 = 0x3C5D0;
    pub const SLOTS: u32 = 0x3CC04;
    pub const DIR_MAP: u32 = 0x3CD14;
    pub const ATTACK_CHANCE: u32 = 0x3CFCA;
    pub const SPEEDS: u32 = 0x3D120;
}

impl Game {
    /// 0x3B828 class 10.
    pub fn enemy_class10(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => self.c10_init(e, h),
            2 => self.c10_alive(e, h),
            4 => self.c10_dying(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x3CB70
            }
        }
    }

    /// The player a pool A enemy hunts (`+$76`). Reads player 1 when the word is no player.
    pub(super) fn c10_target(&self, e: &Enemy) -> usize {
        match slot_of(e.m.w(TARGET)) {
            Some(Slot::Player(i)) => i,
            _ => 0,
        }
    }

    // ------------------------------------------------------------------ init and entrances

    /// 0x3B83C: sub 0 type init, sub 2 entrance, sub 4 end of the entrance.
    fn c10_init(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => {
                // 0x3B850
                e.b.sub += 2;
                let (id, w84, table) = match (e.b.var26 >> 8) as u8 {
                    0 => (3, 0xC, 0x3D150),   // 0x3B8C6 HAMMER T.
                    2 => (0xA, 0x28, 0x3D400), // 0x3B8E2 BLK ELMER
                    4 => (0xB, 0x2C, 0x3D6B0), // 0x3B8FE WRENCH T.
                    v => panic!("class 10 variant {v:#x}"),
                };
                e.b.id = id;
                e.m.sw(W84, w84);
                e.b.anim_table = table;
                self.enemy_set_hp(&mut e.b); // jmp 0x31424
                self.enemy_anim(&mut e.b, 0);
                e.b.box_table = 0x103000;
                e.b.multi_box = false;
                e.b.stun = 10;
                e.m.sb(POISE_T, 0);
                e.m.sb(WAIT, 0);
                e.b.reaction = 0;
                e.m.sb(ATTACKING, 0);
                e.m.sb(B87, 0);
                e.b.b63 = 0;
                e.b.b72 = 0xB4;
                e.m.sb(f::OBST_T, 0);
                let t = self.enemy_pick_target(&e.b);
                e.m.sw(TARGET, t);
                self.enemy_snap_to_floor(e);
                e.m.sb(ATTACKING, 0);
                self.c10_entrance(e, h); // bra 0x3B91A
            }
            2 => self.c10_entrance(e, h),
            _ => {
                // 0x3BB6A: the entrance is over.
                self.c10_pick_target_slot(e);
                self.c10_reset_ai(e, h);
                self.enemy_draw_check(&mut e.b, h);
            }
        }
    }

    /// 0x3B91A: the entrance of the low byte of `+$26`.
    fn c10_entrance(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_kill_all_check(e);
        match e.b.var26 & 0xFF {
            0 => self.c10_entr_walk(e),
            2 => {
                // 0x3B9CE: drop from 0x100 above.
                self.enemy_floor_collision(e);
                e.b.set_yi_keep(e.b.yi().wrapping_add(0x100));
                e.b.airborne = 1;
                e.b.facing = e.m.b(B5B);
                e.b.sub += 2;
            }
            4 => self.c10_entr_attack(e, 3),
            6 => self.c10_entr_attack(e, 5),
            8 => self.c10_entr_wait(e),
            v => panic!("class 10 entrance {v:#x}"),
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x3B946 entrance 0: walk in towards the screen centre until a player is ahead, or 0x80 px.
    fn c10_entr_walk(&mut self, e: &mut Enemy) {
        if e.b.act == 0 {
            // 0x3B958
            e.m.sw(f::START_X, e.b.xi() as u16);
            self.enemy_anim(&mut e.b, 3);
            self.c10_3d100(e);
            let d = (self.cam.s2.cam_x as i16).wrapping_add(0xC0).wrapping_sub(e.m.w(f::START_X) as i16);
            if d >= 0 {
                e.b.dir = 8;
                e.b.facing = 1;
            } else {
                e.b.dir = 0x18;
                e.b.facing = 0;
            }
            e.b.act += 2;
        }
        // 0x3B98C
        if self.c10_3b9b4(e) {
            e.b.sub += 2;
        } else {
            let d = e.b.xi().wrapping_sub(e.m.w(f::START_X) as i16).wrapping_add(0x80) as u16;
            if d >= 0x100 {
                e.b.sub += 2;
            }
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x3B9B4: is a player 0x18 px ahead (window 0x18 / 0x30)? Returns the carry of 0x3BA1C.
    fn c10_3b9b4(&self, e: &Enemy) -> bool {
        let mut d0 = 0x18i16;
        if e.b.facing == 0 {
            d0 = -d0;
        }
        d0 = d0.wrapping_add(e.b.xi());
        self.c10_3ba1c(e, d0, 0x18, 0x30)
    }

    /// 0x3BA1C: the players P1, P2, P3 in turn through 0x3BA52. The routine works on d1 in place, so
    /// the second and third player are tested with the d1 the previous test left (kept).
    pub(super) fn c10_3ba1c(&self, e: &Enemy, d0: i16, d2: u16, d3: u16) -> bool {
        let mut d1 = d0 as u16;
        for p in 0..3 {
            let q = &self.players[p].b;
            if q.live == 0 {
                continue;
            }
            // 0x3BA52
            d1 = d1.wrapping_sub(q.xi() as u16).wrapping_add(d2);
            let carry = if d1 >= d3 {
                false
            } else {
                d1 = e.b.zi().wrapping_sub(q.zi()).wrapping_add(0xC) as u16;
                d1 < 0x18
            };
            if p == 2 {
                return carry;
            }
            if carry {
                return true;
            }
        }
        false
    }

    /// 0x3B9EC / 0x3BA04 entrances 4 and 6: start the attack of kind `kind` at once.
    fn c10_entr_attack(&mut self, e: &mut Enemy, kind: u16) {
        self.c10_pick_target_slot(e);
        self.c10_3ccc0(e);
        e.m.sw(f::KIND, kind);
        e.b.set_state(0x0200_0A00);
    }

    /// 0x3BA6E entrance 8: wait (standing) until visible, then until a player is near.
    fn c10_entr_wait(&mut self, e: &mut Enemy) {
        match e.b.act {
            0 => {
                // 0x3BA82
                e.b.act += 2;
                let r = (self.rand() & 0x3E) as u32;
                let v = self.assets.enemy.t.u16(t::ENTR8_WAIT, r);
                e.m.sw(f::WAIT_B6, v);
                self.enemy_anim(&mut e.b, 1);
            }
            2 => {
                // 0x3BAE6
                let d = (self.cam.s2.cam_x as i16).wrapping_add(0xC0).wrapping_sub(e.b.xi()).wrapping_add(0xC0) as u16;
                if d < 0x180 {
                    e.b.act += 2;
                }
                self.enemy_step(&mut e.b);
                self.enemy_player_near(e);
            }
            _ => {
                // 0x3BB0A
                let t = e.m.w(f::WAIT_B6);
                e.m.sw(f::WAIT_B6, t.wrapping_sub(1));
                if t == 0 {
                    e.b.sub += 2;
                    return;
                }
                self.enemy_step(&mut e.b);
                self.enemy_player_near(e);
            }
        }
    }
}
