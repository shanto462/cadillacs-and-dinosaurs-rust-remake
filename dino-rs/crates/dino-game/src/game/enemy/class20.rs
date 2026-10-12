//! Pool A class 20 (`+$20 = 0x50`, handler 0x457B0): PUNK (name 8) and THUG (name 9). Phase 1 task 4a.
//! Variant = high byte of `+$26` (0 Punk, else Thug), entrance = low byte (0 / 2 stand and wait, 4 walk in,
//! 6 dash in, 8 fade in, 0xA drop). Not a copy of class 2: six AI states `+$06` 0..0xA, a script of attacks
//! (`+$AA` row of the table 0x463F6, `+$AC` position) and a second machine for the action 4 (`+$A5`, `+$A6`).
//! See `class10.rs` for the conventions (addresses in comments, `bool` = the carry).

use super::{Enemy, off::*};
use crate::game::{Game, Handle};

/// Offsets (this class' meaning).
pub(super) mod f {
    pub const WAIT_T: usize = 0xB2; // word, entrances 0 / 2
    pub const START_X: usize = 0xB4; // word, entrances 4 / 6
    pub const FADE_T: usize = 0xB6;
    pub const FADE_STEP: usize = 0xB7;
    pub const OBST_T: usize = 0xB8;
    pub const SLOT: usize = 0x7C;
    pub const T80: usize = 0x80;
    pub const DEST_X: usize = 0x88;
    pub const DEST_Z: usize = 0x8A;
    pub const T_A0: usize = 0xA0; // word
    pub const ACTION: usize = 0xA2; // word: 0 walk, 2 jump attack, 4 choice machine
    pub const COUNT: usize = 0xA4; // byte
    pub const CHOICE: usize = 0xA5; // byte
    pub const BUSY: usize = 0xA6; // byte
    pub const STRAFE: usize = 0xA7; // byte
    pub const A8: usize = 0xA8; // byte
    pub const A9: usize = 0xA9; // byte timer
    pub const ROW: usize = 0xAA; // word: script row
    pub const POS: usize = 0xAC; // word: script position
    pub const SUB: usize = 0xAE; // word (byte in the last action): sub step
    pub const HIT: usize = 0xB0; // byte: the swing hit (set by the hit stop code)
    pub const T_B1: usize = 0xB1;
}

/// Table addresses.
pub(super) mod t {
    pub const WAIT_A: u32 = 0x458E6;
    pub const WAIT_B: u32 = 0x459E0;
    pub const ACTION_PICK: u32 = 0x45E64;
    pub const JUMPS: u32 = 0x45F54;
    pub const SCRIPT: u32 = 0x463F6;
    pub const CHOICES: u32 = 0x460F8;
    pub const SLOTS: u32 = 0x46AF6;
    pub const CIRCLE: u32 = 0x46CF0;
    pub const ROW_PICK: u32 = 0x46EC6;
    pub const CHANCE: u32 = 0x46D98;
    pub const OFFS: u32 = 0x46E22;
    pub const REC_IDX: u32 = 0x4697E;
    pub const REC: u32 = 0x4699E;
    pub const SPEEDS: u32 = 0x47224;
    pub const FADE_SRC: u32 = 0x45C6E;
}

impl Game {
    /// 0x457B0 class 20.
    pub fn enemy_class20(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => self.c20_init(e, h),
            2 => self.c20_alive(e, h),
            4 => self.enemy_dying_basic(e, h), // 0x46A2E
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x46A6C
            }
        }
    }

    /// The target of the object (`+$76`) as a player index.
    pub(super) fn c20_target(&self, e: &Enemy) -> usize {
        self.c10_target(e)
    }

    // ------------------------------------------------------------------ init and entrances

    /// 0x457C4.
    fn c20_init(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => {
                // 0x457D8
                e.b.sub += 2;
                let (id, w84, table) = if (e.b.var26 >> 8) as u8 == 0 { (8, 0x20, 0x47244) } else { (9, 0x24, 0x475FE) };
                e.b.id = id;
                e.m.sw(W84, w84);
                e.b.anim_table = table;
                self.enemy_set_hp(&mut e.b);
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
                e.b.b72 = 0x3C;
                e.m.sb(f::CHOICE, 0);
                e.m.sb(f::FADE_T, 0);
                e.m.sb(f::FADE_STEP, 0);
                e.m.sb(f::OBST_T, 0);
                let tg = self.enemy_pick_target(&e.b);
                e.m.sw(TARGET, tg);
                self.enemy_snap_to_floor(e);
                e.m.sb(ATTACKING, 0);
                self.c20_entrance(e, h);
            }
            2 => self.c20_entrance(e, h),
            _ => {
                // 0x45CBC
                self.c20_46a72(e);
                self.c20_45db0(e, h);
            }
        }
    }

    /// 0x4587C.
    fn c20_entrance(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_kill_all_check(e);
        match e.b.var26 & 0xFF {
            0 => self.c20_entr_wait(e, 3, t::WAIT_A, false),
            2 => self.c20_entr_wait(e, 4, t::WAIT_B, true),
            4 => self.c20_entr_walk(e, 1, None),
            6 => self.c20_entr_walk(e, 2, Some(0x1F)),
            8 => self.c20_entr_fade(e),
            0xA => {
                // 0x45C9E
                self.enemy_floor_collision(e);
                e.b.set_yi_keep(e.b.yi().wrapping_add(0x100));
                e.b.airborne = 1;
                e.b.facing = e.m.b(B5B);
                e.b.sub += 2;
            }
            v => panic!("class 20 entrance {v:#x}"),
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x458AA (anim 3, no animation step) / 0x4599E (anim 4, animation step): wait until visible and a
    /// player is near, or the random time is over.
    fn c20_entr_wait(&mut self, e: &mut Enemy, anim: u16, table: u32, step: bool) {
        match e.b.act {
            0 => {
                e.b.act += 2;
                let r = (self.rand() & 0x3E) as u32;
                let v = self.assets.enemy.t.u16(table, r);
                e.m.sw(f::WAIT_T, v);
                e.b.facing = e.m.b(B5B);
                self.enemy_anim(&mut e.b, anim);
                if step {
                    self.enemy_step(&mut e.b); // 0x4599E steps the animation after every act
                }
                return;
            }
            2 => {
                let d = (self.cam.s2.cam_x as i16).wrapping_add(0xC0).wrapping_sub(e.b.xi()).wrapping_add(0xC0) as u16;
                if d < 0x180 {
                    e.b.act += 2;
                }
                self.enemy_player_near(e);
            }
            _ => {
                let w = e.m.w(f::WAIT_T);
                e.m.sw(f::WAIT_T, w.wrapping_sub(1));
                if w == 0 {
                    e.b.sub += 2;
                } else {
                    self.enemy_player_near(e);
                }
            }
        }
        if step {
            self.enemy_step(&mut e.b);
        }
    }

    /// 0x45A4E / 0x45AD6: walk (anim 1, speed by 0x47204) or dash (anim 2, speed 0x1F) in.
    fn c20_entr_walk(&mut self, e: &mut Enemy, anim: u16, speed: Option<u16>) {
        if e.b.act == 0 {
            e.m.sw(f::START_X, e.b.xi() as u16);
            self.enemy_anim(&mut e.b, anim);
            match speed {
                None => self.c20_47204(e),
                Some(id) => self.enemy_set_speed(&mut e.b, id),
            }
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
        let far = e.b.xi().wrapping_sub(e.m.w(f::START_X) as i16).wrapping_add(0x80) as u16 >= 0x100;
        if speed.is_none() {
            // 0x45A94: a player 0x18 ahead (window 0x18 / 0x30) or 0x80 px walked.
            if self.c20_player_ahead(e, 0x18, 0x18, 0x30) || far {
                e.b.sub += 2;
            }
        } else {
            // 0x45B20: from 0x40 px on a player 0x20 ahead (window 0x20 / 0x40) ends it, or 0x80 px.
            let walked = e.b.xi().wrapping_sub(e.m.w(f::START_X) as i16).wrapping_add(0x40) as u16;
            if (walked >= 0x80 && self.c20_player_ahead(e, 0x20, 0x20, 0x40)) || far {
                e.b.sub += 2;
            }
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x45ABC / 0x45B5A + 0x45B74: the players in turn (0x45BAA keeps d1 across the players, as in
    /// class 10). Returns the carry.
    fn c20_player_ahead(&self, e: &Enemy, ahead: i16, d2: u16, d3: u16) -> bool {
        let mut d0 = ahead;
        if e.b.facing == 0 {
            d0 = -d0;
        }
        d0 = d0.wrapping_add(e.b.xi());
        self.c10_3ba1c(e, d0, d2, d3)
    }

    /// 0x45BC6 entrance 8: fade in from a doorway (palette steps through 0x45C32).
    fn c20_entr_fade(&mut self, e: &mut Enemy) {
        if e.b.act == 0 {
            e.b.act += 2;
            e.b.palette_override = 0x9D;
            e.b.set_zi_keep(e.b.zi().wrapping_add(0x10));
            e.b.vz = 0xFFC0u16 as i16;
            e.m.sb(f::FADE_T, 0x14);
            self.c20_45c32(e);
            return self.enemy_anim(&mut e.b, 5);
        }
        let b = e.m.b(f::FADE_T).wrapping_sub(1);
        e.m.sb(f::FADE_T, b);
        if b == 0 {
            if e.m.b(f::FADE_STEP) == 0xC {
                e.b.sub += 2;
                e.b.palette_override = 0;
                return;
            }
            e.m.sb(f::FADE_T, 0x14);
            e.m.sb(f::FADE_STEP, e.m.b(f::FADE_STEP).wrapping_add(4));
            self.c20_45c32(e);
        }
        e.b.motion();
        self.enemy_step(&mut e.b);
    }

    /// 0x45C32: copy 16 colours of the step `+$B7` into OBJ palette 0x1D (0x9143A0), keeping the brightness
    /// nibble of each word. Source row = long table 0x45C6E by variant (4 longs per variant).
    fn c20_45c32(&mut self, e: &Enemy) {
        let var = (e.b.var26 >> 8) as u32 & 0xFF;
        let src = self.assets.enemy.t.u32(t::FADE_SRC, var * 8 + e.m.b(f::FADE_STEP) as u32);
        let base = 0x1D * 16;
        for k in 0..16 {
            let w = self.pal.words[base + k];
            self.pal.words[base + k] = (w & 0xF000) | self.assets.enemy.t.u16(src, 2 * k as u32);
        }
        self.pal.dirty = true;
    }

    /// 0x47204: walk speed id by variant row and aggression (`jmp 0x17A4`).
    pub(super) fn c20_47204(&mut self, e: &mut Enemy) {
        let row = ((e.b.var26 >> 8) as u32 & 0xFF) << 3;
        let id = self.assets.enemy.t.u16(t::SPEEDS, row + (self.rank.aggression as u32 & 7) * 2);
        self.enemy_set_speed(&mut e.b, id);
    }
}
