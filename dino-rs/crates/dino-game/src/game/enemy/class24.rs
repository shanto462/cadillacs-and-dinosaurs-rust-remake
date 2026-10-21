//! Shivat, the T. rex, class 24 (handler 0x4838C..0x49448, tables up to 0x49574): Episode 2 (asleep at X 0x678) and Episode 5 (asleep at
//! X 0x450). D3b section 5, DS03 to DS05, DS14; `re/specs/enemy/SPEC.md` section 11. The original is one routine with branches into the middle of
//! other states, so blocks keep their addresses as names.
//!
//! +$04 mode: 0 init (the init ends in the MICHI entry and returns, the AI starts the next frame), 2 alive, 4 and 6 free (0x4908C). In mode 2 the sub
//! state (+$05, table 0x484A4) is 0 the AI (0x484B4), 2 the hit reaction (0x48D16), 4 the grab (0x48E62, `class24_grab.rs`), 6 nothing (0x4908A).
//! The behaviour mode word +$A8 (table 0x48510): 0 MICHI (asleep), 2 MATOWARI (neutral), 4 OKORI (angry, `class24_ai.rs`), 6 MANZOKU (satisfied,
//! leaves), 8 COLOR (the colour flash, then angry). Own fields: +$A4 angry, +$A5 / +$A6 / +$A7 colour flash, +$AA word mode timer, +$AC byte roll timer,
//! +$AD byte walk animation variant, +$AE long anchor X (integer word +$AE, the fraction word +$B0 is also the retarget counter of OKORI), +$B4 word
//! attack cooldown, +$B6 word saved mode of the grab, +$B9 hits taken, +$BA hostile berserk (no more pauses), +$BB turns left, +$BC Z mark timer,
//! +$BD palette to restore, +$C0 hunt flag (0xFF asleep), +$C1 claimed by a hunter, +$76 target.
//! Movement is per animation frame: the step table 0x49460 moves the anchor and Z when the frame's flag byte (+$2D) changes (0x4937E).
//!
//! Owner: enemy port (M6-04 / M6-05).

use super::{off::*, Enemy};
use crate::game::{slot_of, Game, Handle, Slot};

/// Own offsets of Shivat (the original offsets).
pub(super) mod o {
    pub const MODE: usize = 0xA8;
    pub const ANGRY: usize = 0xA4;
    pub const TIMER: usize = 0xAA;
    pub const ROLL: usize = 0xAC;
    pub const WALK: usize = 0xAD;
    pub const ANCHOR: usize = 0xAE;
    pub const RETARGET: usize = 0xB0;
    pub const COOLDOWN: usize = 0xB4;
    pub const SAVED_MODE: usize = 0xB6;
    pub const HITS: usize = 0xB9;
    pub const BERSERK: usize = 0xBA;
    pub const TURNS: usize = 0xBB;
    pub const Z_TIMER: usize = 0xBC;
    pub const PALETTE: usize = 0xBD;
    pub const HUNT: usize = 0xC0;
    pub const CLAIMED: usize = 0xC1;
    pub const COUNT: usize = 0x81;
    /// Tables (`dino-extract` TABLES): the step table of the walk (32 x vx, vz), the frame X offsets, the bite and the long reach rolls, the hit slide.
    pub const STEPS: u32 = 0x49460;
    pub const FRAME_X: u32 = 0x493E2;
    pub const BITE_ROLL: u32 = 0x49520;
    pub const REACH_ROLL: u32 = 0x49560;
    pub const SLIDE: u32 = 0x49570;
}
use o::*;

impl Game {
    /// 0x4838C: class 24. Mode 0 init, 2 the sub states, 4 and 6 free.
    pub(super) fn enemy_class24(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => self.c24_init(e),
            2 => match e.b.sub {
                0 => self.c24_alive(e, h),
                2 => self.c24_hit(e, h),
                4 => self.c24_grab(e, h),
                _ => {} // 0x4908A: rts
            },
            _ => self.enemy_free_pool_a(e, i), // 0x4908C: jmp 0x48FC
        }
    }

    /// 0x483A0: the init. Visible at once, name 7, HP by level, the multi box table 0x106000, poise 10, the own step table, the target (picked twice, as
    /// the code does), mode word 0 (MICHI: the `bra 0x4851A` ends the frame). Episode 2 and Episode 5 (stage 1 and 4) use the green palette 0x9E.
    fn c24_init(&mut self, e: &mut Enemy) {
        e.b.visible = 1;
        e.b.id = 7;
        e.m.sw(W84, 0x1C);
        e.b.anim_table = 0x49574;
        self.enemy_set_hp(&mut e.b); // 0x31424
        e.b.box_table = 0x106000;
        e.b.multi_box = true;
        e.b.stun = 10;
        for off in [POISE_T, WAIT, ANGRY, B87, HITS, BERSERK, Z_TIMER, CLAIMED] {
            e.m.sb(off, 0);
        }
        e.b.reaction = 0;
        e.b.b72 = 0xC8;
        e.b.speed_table = o::STEPS;
        e.m.sw(TARGET, self.enemy_pick_target(&e.b)); // 0x32032
        e.m.sw(MODE, 0);
        e.m.sw(TIMER, 0);
        e.m.sb(ROLL, 0);
        e.m.sb(WALK, 0);
        e.m.sb(PALETTE, 0); // 0x48424 clears +$BD; +$BB survives slot reuse
        e.m.sb(HUNT, 0xFF);
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sb(ATTACKING, 0xFF);
        self.enemy_anim(&mut e.b, 0);
        e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
        e.b.dir = 0x18;
        e.b.facing = 0;
        e.m.sw(ANCHOR, e.b.xi() as u16);
        e.m.sw(TARGET, self.enemy_pick_target(&e.b));
        if self.w.stage == 1 || self.w.stage == 4 {
            e.b.palette_override = 0x9E;
            e.m.sb(PALETTE, 0x9E);
        }
        e.b.mode = e.b.mode.wrapping_add(2);
        self.c24_enter_michi(e); // bra 0x4851A
    }

    /// 0x4851A: MICHI entry: state 0, timer 0, mode word 0.
    pub(super) fn c24_enter_michi(&mut self, e: &mut Enemy) {
        e.b.act = 0;
        e.b.step = 0;
        e.m.sw(TIMER, 0);
        e.m.sb(ROLL, 0);
        e.m.sw(MODE, 0);
    }

    /// 0x49C0: the draw list entry by the visible byte of the last 0x49CA.
    pub(super) fn c24_draw(&mut self, e: &Enemy, h: Handle) {
        if e.b.visible != 0 {
            self.draw_list.push(((e.b.zi() as u16 as u32) << 16) | h as u32);
        }
    }

    /// 0x484B4: sub 0, the AI. The poise refill timer, the skip timer +$7A, the mode word (table 0x48510), the frame position (0x493C4), the floor
    /// (0x12E20), the draw (0x49C0) and the hostile list (0x4C0A).
    fn c24_alive(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(POISE_T);
        if t != 0 {
            e.m.sb(POISE_T, t.wrapping_sub(1));
            if t == 1 {
                e.b.stun = 10;
            }
        }
        if e.m.b(WAIT) != 0 {
            e.m.sb(WAIT, e.m.b(WAIT).wrapping_sub(1));
        } else {
            match e.m.w(MODE) {
                0 => self.c24_michi(e, h),
                2 => self.c24_matowari(e, h),
                4 => self.c24_okori(e, h),
                6 => self.c24_manzoku(e),
                8 => self.c24_color(e),
                _ => {}
            }
            self.c24_position(e); // bsr 0x493C4
            self.terrain_collide_free(&mut e.b); // 0x12E20
            if e.b.airborne != 0 {
                let y = e.b.floor.wrapping_add(e.b.floor2);
                e.b.set_yi_keep(y);
            }
        }
        self.c24_draw(e, h); // jsr 0x49C0
        self.enemy_monster_register(h); // jmp 0x4C0A
    }

    /// 0x493C4: the drawn X is the anchor + the offset of the frame (table 0x493E2 by +$2C), mirrored when he faces right.
    fn c24_position(&self, e: &mut Enemy) {
        let mut d = self.assets.enemy.t.i16(o::FRAME_X, e.b.param_a as u32);
        if e.b.facing != 0 {
            d = d.wrapping_neg();
        }
        e.b.set_xi_keep(d.wrapping_add(e.m.w(ANCHOR) as i16));
    }

    // ------------------------------------------------------------------ MICHI, MATOWARI

    /// 0x48530: MICHI, asleep. State 0 (0x48548): cannot be hit (+$00 = 2), the anchor, animation 0, 30 frames; state 2 (0x4856E): a Z mark
    /// (pool C class 30, variant 2) every 0x32 + (rand & 0x3F) frames; state 4 (0x485B2): after the wake animation he is angry at once when a hero is in
    /// sight, else neutral. Then the far test.
    fn c24_michi(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => {
                e.b.live = 2;
                e.m.sw(ANCHOR, e.b.xi() as u16);
                self.enemy_anim(&mut e.b, 0);
                e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
                e.m.sb(T80, 0x1E);
                e.b.act = e.b.act.wrapping_add(2);
            }
            2 => {
                let t = e.m.b(Z_TIMER);
                e.m.sb(Z_TIMER, t.wrapping_sub(1));
                if t == 0 {
                    self.enemy_monster_spawn_z(e, h, 2); // 0x9667E with d0 = 2 (the state)
                    let r = (self.rand() & 0x3F) as u8;
                    e.m.sb(Z_TIMER, r.wrapping_add(0x32));
                }
            }
            4 => {
                if self.enemy_step(&mut e.b) {
                    if self.c24_in_sight(e) {
                        self.c24_enter_color(e); // bcs 0x48CD2
                    } else {
                        self.c24_enter_matowari(e); // bra 0x485CA
                    }
                }
            }
            _ => {}
        }
        self.enemy_despawn_if_far(e, 0); // jmp 0x173C
    }

    /// 0x485CA: MATOWARI entry.
    fn c24_enter_matowari(&mut self, e: &mut Enemy) {
        e.m.sw(MODE, 2);
        e.b.act = 0;
        e.b.step = 0;
        e.m.sw(TIMER, 0);
        e.m.sb(ROLL, 0);
        e.m.sb(HUNT, 0);
        e.m.sb(TURNS, 4);
    }

    /// 0x485EA: MATOWARI, neutral pacing. The leave test (0x490F6) first; after 900 frames a roll every 10th frame (1 in 16) makes him angry (COLOR);
    /// the states: 0 the walk animation, 2 walk (the step per frame, an idle pause 2 in 16 at a still frame, a turn at the screen edge), 4 turn round,
    /// 6 the pause.
    fn c24_matowari(&mut self, e: &mut Enemy, h: Handle) {
        if self.c24_leave_test(e) {
            return self.c24_enter_manzoku(e); // bcs 0x48C06
        }
        e.m.sw(TIMER, e.m.w(TIMER).wrapping_add(1));
        if e.m.w(TIMER) >= 0x384 {
            let old = e.m.b(ROLL);
            e.m.sb(ROLL, old.wrapping_sub(1));
            if old == 0 {
                e.m.sb(ROLL, 0xA);
                if self.rand() & 0xF == 0 {
                    return self.c24_enter_color(e); // beq 0x48CD2
                }
            }
        }
        match e.b.act {
            0 => {
                // 0x48632
                e.b.act = e.b.act.wrapping_add(2);
                let v = if self.rank.aggression >= 8 { 2 } else { 1 };
                e.m.sb(WALK, v);
                self.c24_walk_anim(e);
            }
            2 => {
                // 0x4864C
                e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
                if !self.enemy_step(&mut e.b) {
                    return self.c24_step_move(e); // bcc 0x4870E
                }
                if self.c24_edge(e) {
                    // 0x48712: turn round (animation 0x13)
                    e.b.act = 4;
                    self.enemy_anim(&mut e.b, 0x13);
                    e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
                    return;
                }
                if e.b.param_a == 0 {
                    let r = self.rand() & 0xF;
                    if (3u16 >> r) & 1 != 0 {
                        // 0x48680: a pause (animation 0x12) for 120 frames
                        e.m.sb(T80, 0x78);
                        e.b.act = 6;
                        return self.enemy_anim(&mut e.b, 0x12);
                    }
                }
                self.c24_in_sight(e); // bsr 0x49148 (sets the target as a side effect)
                self.c24_aim(e, h);
                self.c24_step_move(e);
            }
            4 => {
                // 0x48728
                if !self.enemy_step(&mut e.b) {
                    return;
                }
                let n = e.m.b(TURNS).wrapping_sub(1);
                e.m.sb(TURNS, n);
                if n == 0 {
                    return self.c24_enter_manzoku(e); // beq 0x48C06
                }
                e.b.facing ^= 1;
                e.b.dir = e.b.dir.wrapping_add(0x10) & 0x1F;
                e.b.act = 0;
                self.c24_walk_anim(e);
            }
            6 => {
                // 0x48756
                let old = e.m.b(T80);
                e.m.sb(T80, old.wrapping_sub(1));
                if old == 0 {
                    e.b.act = 0;
                }
                self.enemy_step(&mut e.b);
            }
            _ => {}
        }
    }

    /// 0x48946 and its twin 0x486AC: the direction toward the target, at most 4 steps off the side he faces (8 right, 24 left): the angle is shifted by 8
    /// (facing left: minus 8, facing right: plus 8), limited to 0xC .. 0x14, and shifted back. The destination is stored in +$88 / +$8A.
    pub(super) fn c24_aim(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(TARGET);
        let (tx, tz) = self.c24_obj_xz(e, h, t);
        e.m.sw(DEST_X, tx as u16);
        e.m.sw(DEST_Z, tz as u16);
        let a = Game::enemy_angle_to(&e.b, tx as u16, tz as u16);
        let (pre, post): (u8, u8) = if e.b.facing == 0 { (8u8.wrapping_neg(), 8) } else { (8, 8u8.wrapping_neg()) };
        let mut d = a.wrapping_add(pre) & 0x1F;
        d = d.clamp(0xC, 0x14);
        e.b.dir = d.wrapping_add(post) & 0x1F;
    }

    /// X and Z of any object by handle (Shivat himself is not in his slot while he runs).
    pub(super) fn c24_obj_xz(&self, e: &Enemy, h: Handle, t: Handle) -> (i16, i16) {
        if t == h {
            (e.b.xi(), e.b.zi())
        } else {
            let b = self.body(t);
            (b.xi(), b.zi())
        }
    }

    /// `+$00`, X and Z of any object by handle (himself included).
    pub(super) fn c24_view(&self, e: &Enemy, h: Handle, t: Handle) -> (u8, i16, i16) {
        if t == h {
            (e.b.live, e.b.xi(), e.b.zi())
        } else {
            let b = self.body(t);
            (b.live, b.xi(), b.zi())
        }
    }

    // ------------------------------------------------------------------ MANZOKU, COLOR

    /// 0x48C06: MANZOKU entry (satisfied): +$00 = 2; when he was angry the colour flashes back (0x493EC, animation 0xA, roar), else the leave walk
    /// starts at once (0x48C68).
    pub(super) fn c24_enter_manzoku(&mut self, e: &mut Enemy) {
        e.m.sw(MODE, 6);
        e.b.live = 2;
        e.b.act = 0;
        e.b.step = 0;
        e.m.sb(HUNT, 0);
        if e.m.b(ANGRY) != 0 {
            e.m.sb(ANGRY, 0);
            self.enemy_monster_flash_start(e); // bsr 0x493EC
            self.enemy_anim(&mut e.b, 0xA);
            e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
            self.c24_roar(e); // jmp 0xA09E
        } else {
            self.c24_leave_walk(e);
        }
    }

    /// 0xA09E: the roar 0x60 with the pan of the world X.
    pub(super) fn c24_roar(&mut self, e: &Enemy) {
        self.snd_request(0x60, self.snd_pan_x(e.b.xi()) as u32, 0);
    }

    /// 0x48C68: the leave walk: the anchor, walk variant 3; facing right he first turns round (state 2, animation 0xE), else he walks (state 4).
    fn c24_leave_walk(&mut self, e: &mut Enemy) {
        e.m.sw(ANCHOR, e.b.xi() as u16);
        e.m.sb(WALK, 3);
        if e.b.facing != 0 {
            e.b.act = 2;
            self.enemy_anim(&mut e.b, 0xE);
            e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
        } else {
            e.b.act = 4;
            self.c24_walk_anim(e);
        }
    }

    /// 0x48C40: MANZOKU. The far test, then the state: 0 the colour flash (0x49400) back to green, 2 the turn round, 4 the walk away.
    fn c24_manzoku(&mut self, e: &mut Enemy) {
        self.enemy_despawn_if_far(e, 0); // jsr 0x173C
        match e.b.act {
            0 => {
                if self.enemy_monster_flash_tick(e) {
                    self.c24_leave_walk(e);
                } else {
                    self.enemy_step(&mut e.b); // jmp 0x121E
                }
            }
            2 => {
                if self.enemy_step(&mut e.b) {
                    e.b.facing ^= 1;
                    e.b.dir = e.b.dir.wrapping_add(0x10) & 0x1F;
                    e.b.act = e.b.act.wrapping_add(2);
                    self.c24_walk_anim(e);
                }
            }
            4 => {
                e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
                self.enemy_step(&mut e.b);
                self.c24_step_move(e); // bra 0x4937E
            }
            _ => {}
        }
    }

    /// 0x48CD2: COLOR entry: he is angry now (+$A4 = 1), cannot be hit while he flashes, roars, the flash starts (animation 0xA).
    pub(super) fn c24_enter_color(&mut self, e: &mut Enemy) {
        e.m.sw(MODE, 8);
        e.b.act = 0;
        e.b.step = 0;
        e.m.sb(ANGRY, 1);
        e.b.live = 2;
        e.m.sb(HUNT, 0);
        self.c24_roar(e);
        self.enemy_monster_flash_start(e); // bsr 0x493EC
        self.enemy_anim(&mut e.b, 0xA);
    }

    /// 0x48CFE: COLOR: the flash (0x49400), at its end he is hittable again (+$00 = 1) and angry (OKORI entry 0x48768).
    fn c24_color(&mut self, e: &mut Enemy) {
        if self.enemy_monster_flash_tick(e) {
            e.b.live = 1;
            self.c24_enter_okori(e);
        } else {
            self.enemy_step(&mut e.b); // jmp 0x121E
        }
    }

    /// 0x48768: OKORI entry.
    fn c24_enter_okori(&mut self, e: &mut Enemy) {
        e.m.sw(MODE, 4);
        e.b.act = 0;
        e.b.step = 0;
        e.m.sw(COOLDOWN, 0);
        e.m.sw(TIMER, 0);
        e.m.sb(ROLL, 0);
        e.m.sb(HUNT, 0);
        e.m.sb(TURNS, 0xA);
    }

    // ------------------------------------------------------------------ helpers (0x490F6 .. 0x49334)

    /// 0x490F6: the leave test. `+$5C` is the camera X at which he leaves (0 = never); true (carry) when the camera has reached it.
    pub(super) fn c24_leave_test(&self, e: &Enemy) -> bool {
        let w = e.m.w(W5C);
        w != 0 && w <= self.cam.s2.cam_x
    }

    /// 0x49110: at the screen edge he faces out of? Facing left: X - camera X <= 0x30; facing right: X - camera X >= 0x150 (signed word compares).
    pub(super) fn c24_edge(&self, e: &Enemy) -> bool {
        let d = e.b.xi().wrapping_sub(self.cam.s2.cam_x as i16);
        if e.b.facing == 0 {
            d <= 0x30
        } else {
            d >= 0x150
        }
    }

    /// 0x49148 / 0x49164: is a hero in sight (|dx| <= 0xA0, |dz| <= 0x10)? The players are tried in order and every LIVE player is stored as the target
    /// before the test, so without a match the target is the last live player. True (carry) for the first one in sight.
    pub(super) fn c24_in_sight(&mut self, e: &mut Enemy) -> bool {
        for p in 0..3 {
            if self.players[p].b.live == 0 {
                continue;
            }
            e.m.sw(TARGET, Game::player_handle(p));
            let pb = &self.players[p].b;
            let dx = pb.xi().wrapping_sub(e.b.xi()).wrapping_add(0xA0) as u16;
            let dz = pb.zi().wrapping_sub(e.b.zi()).wrapping_add(0x10) as u16;
            if dx <= 0x140 && dz <= 0x20 {
                return true;
            }
        }
        false
    }

    /// 0x491A4: the nearest counted enemy of last frame's list 0x69BA (|dx| + |dz|, words; `+$7D` negative ones are skipped; ties go to the later
    /// one). Nobody in the list: himself.
    pub(super) fn c24_nearest_enemy(&mut self, e: &mut Enemy, h: Handle) {
        let list = self.enemy_c24_hurt_list();
        if list.is_empty() {
            e.m.sw(TARGET, h);
            return;
        }
        let (x, z) = (e.b.xi(), e.b.zi());
        let mut best = 0xFFFFu16;
        for a1 in list {
            if a1 == h {
                continue; // he is in last frame's list too, with +$7D = 0xFF, which the test below skips
            }
            let b = self.body(a1);
            if (self.c24_counted_byte(a1) as i8) < 0 {
                continue;
            }
            let d = (b.xi().wrapping_sub(x)).wrapping_abs() as u16;
            let d = d.wrapping_add((b.zi().wrapping_sub(z)).wrapping_abs() as u16);
            if best >= d {
                e.m.sw(TARGET, a1);
                best = d;
            }
        }
    }

    /// 0x491A4 / 0x491F8 / 0x49296 read the saved pointer and count, but 0x4BEA
    /// rebuilds the current list in the same downward-growing stack. Earlier
    /// objects have already overwritten its high entries when Shivat runs.
    pub(super) fn enemy_c24_hurt_list(&self) -> Vec<Handle> {
        let mut saved = self.combat.get(0x69BA).to_vec();
        let current = self.combat.get(0x671E);
        let overlap = saved.len().min(current.len());
        let saved_start = saved.len() - overlap;
        saved[saved_start..].copy_from_slice(&current[current.len() - overlap..]);
        saved
    }

    /// `+$7D` of any object of the hit list (pool A enemies, pool B props, pool E items).
    fn c24_counted_byte(&self, t: Handle) -> u8 {
        match slot_of(t) {
            Some(Slot::Enemy(i)) => self.enemies.slots[i].m.b(ATTACKING),
            Some(Slot::Prop(i)) => self.props.slots[i].m.b(ATTACKING),
            Some(Slot::Item(i)) => self.items.slots[i].m.b(ATTACKING),
            _ => 0,
        }
    }

    // ------------------------------------------------------------------ the hit reaction (sub 2, 0x48D16)

    /// 0x48D16: the hit reaction. State 0, first step (0x48D30): with HP, poise and on the ground the flinch (animation 0xF for the hit stop frames,
    /// the HUD bar); else, and in the states 2 and 4 (4 = a burn reaction, palette 0x84), the slide back (0x48D6E). The draw 0x49C0 is called once for the
    /// flinch and twice for the slide (its own `jmp 0x49C0` returns into the second).
    fn c24_hit(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.act == 0 {
            if e.b.step == 0 {
                if e.b.hp != 0 && e.b.stun != 0 && e.b.airborne == 0 {
                    self.enemy_anim(&mut e.b, 0xF);
                    self.enemy_hud_hit(h, &e.b, 0); // 0x16B36
                    e.b.step = e.b.step.wrapping_add(2);
                    self.c24_flinch_tick(e);
                } else {
                    self.c24_slide(e, h);
                    self.c24_draw(e, h);
                }
            } else {
                self.c24_flinch_tick(e);
            }
        } else {
            self.c24_slide(e, h);
            self.c24_draw(e, h);
        }
        self.c24_draw(e, h); // 0x48D24: jmp 0x49C0
    }

    /// 0x48D64: the hit stop counter (+$59) runs down; at 0 the recovery.
    fn c24_flinch_tick(&mut self, e: &mut Enemy) {
        e.b.hitstop = e.b.hitstop.wrapping_sub(1);
        if e.b.hitstop == 0 {
            self.c24_recover(e);
        }
    }

    /// 0x48D6E: the slide back: step 0 (0x48D84) takes the burn palette 0x84 when the state was 4, state 2, animation 0x10, the HUD bar, speed 0x500 and
    /// acceleration -0x20 (table 0x49570) away from the side he faces; step 2 (0x48DD4) moves him (list 0x6AC2, 0x189C, floor 0x12E20) until his speed
    /// has turned.
    fn c24_slide(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.step == 0 {
            if e.b.act == 4 {
                e.b.palette_override = 0x84;
            }
            e.b.act = 2;
            self.enemy_anim(&mut e.b, 0x10);
            self.enemy_hud_hit(h, &e.b, 0);
            let mut vx = self.assets.enemy.t.i16(o::SLIDE, 0);
            let mut ax = self.assets.enemy.t.i16(o::SLIDE, 2);
            if e.b.facing != 0 {
                vx = vx.wrapping_neg();
                ax = ax.wrapping_neg();
            }
            e.b.vx = vx;
            e.b.ax = ax;
            e.b.vz = 0;
            e.b.az = 0;
            e.b.step = e.b.step.wrapping_add(2);
        }
        self.combat.push(0x6AC2, h); // 0x4B58
        e.b.motion(); // 0x189C
        self.terrain_collide_free(&mut e.b); // 0x12E20
        let word = ((e.b.facing as u16) << 8) | e.b.draw_flags as u16; // tst.w $24(a6)
        if (word == 0 && e.b.vx < 0) || (word != 0 && e.b.vx >= 0) {
            self.c24_recover(e);
        }
    }

    /// 0x48E02: the recovery: the direction (0x18 left, 8 right), poise 10, the anchor, the palette back, state 0. HP 0: MANZOKU. Not yet angry (a hit on
    /// a sleeper or a neutral one): COLOR. Else the hits count; the first four make him a berserker (+$BA, no more pauses); the walk goes on.
    fn c24_recover(&mut self, e: &mut Enemy) {
        e.b.dir = if e.b.facing != 0 { 8 } else { 0x18 };
        e.b.stun = 10;
        e.m.sw(ANCHOR, e.b.xi() as u16);
        e.b.palette_override = e.m.b(PALETTE);
        e.b.sub = 0;
        e.b.act = 0;
        e.b.step = 0;
        if e.b.hp == 0 {
            return self.c24_enter_manzoku(e);
        }
        e.b.live = 1;
        if e.m.b(ANGRY) == 0 {
            return self.c24_enter_color(e);
        }
        if e.m.b(BERSERK) == 0 {
            let n = e.m.b(HITS).wrapping_add(1);
            e.m.sb(HITS, n);
            if n < 5 {
                e.m.sb(BERSERK, 1);
            }
        }
        self.c24_walk_anim(e);
    }

    // ------------------------------------------------------------------ the grab (sub 4, 0x48E62)

    /// 0x48BE0: the grab starts (OKORI state 6 step 0xE). 0x4E9A6 decides: a hero (+$82 is 0) must be at +$00 = 1, in state 0x0200, on the ground and
    /// not in the action 0x11; an enemy with +$82 set needs the same without the action test. Then sub 4 and the roar 0x5F.
    pub(super) fn c24_grab_start(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(TARGET);
        let ok = match self.c24_grab_target(e, h, t) {
            Some((tag, live, state, airborne, action)) => {
                live == 1 && state == 0x0200 && airborne == 0 && (tag != 0 || action != 0x11)
            }
            None => false,
        };
        if !ok {
            e.b.step = 0; // 0x48C00
            return;
        }
        e.m.sw(SAVED_MODE, e.m.w(MODE));
        e.b.sub = 4;
        e.b.act = 0;
        e.b.step = 0;
        self.snd_request(0x5F, self.snd_pan_x(e.b.xi()) as u32, 0); // jmp 0xA0CC
    }

    /// (+$82, +$00, the state word +$04, +$51, +$C8) of the grab target: a hero reads +$82 as 0.
    pub(super) fn c24_grab_target(&self, e: &Enemy, h: Handle, t: Handle) -> Option<(u8, u8, u16, u8, u8)> {
        if t == h {
            return Some((0, e.b.live, (e.b.state() >> 16) as u16, e.b.airborne, 0));
        }
        match slot_of(t)? {
            Slot::Player(p) => {
                let pl = &self.players[p];
                Some((0, pl.b.live, (pl.b.state() >> 16) as u16, pl.b.airborne, pl.action))
            }
            Slot::Enemy(i) => {
                let en = &self.enemies.slots[i];
                Some((en.m.b(0x82), en.b.live, (en.b.state() >> 16) as u16, en.b.airborne, en.m.b(0xC8)))
            }
            _ => None,
        }
    }

    /// 0x48E62: sub 4. State 0 (0x48E7A): the target is taken (links in +$70, state 0x02060000, action 0x40 for a hero), animation 0x1C; state 2 (0x48F6E,
    /// a hero): at the frame with +$2C clear the hold is checked (0x48F2A), the attack 0x3B hurts him (0x125C0) and he is thrown (state 8) to the side of
    /// the hand; the end of the animation lets go (0x48F56). Then the draw 0x49C0.
    pub(super) fn c24_grab(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => self.c24_grab_take(e, h),
            2 => self.c24_grab_hold_hero(e, h),
            4 => self.c24_grab_hold_enemy(e, h),
            _ => {}
        }
        self.c24_draw(e, h); // jmp 0x49C0
    }

    /// 0x48F56: the release: mode 2 sub 0, the saved mode word back, cooldown 180.
    fn c24_grab_release(&mut self, e: &mut Enemy) {
        e.b.mode = 2;
        e.b.sub = 0;
        e.b.act = 0;
        e.b.step = 0;
        e.m.sw(MODE, e.m.w(SAVED_MODE));
        e.m.sw(COOLDOWN, 0xB4);
    }

    /// 0x48E7A: take the target.
    fn c24_grab_take(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(TARGET);
        let Some((tag, live, state, airborne, _)) = self.c24_grab_target(e, h, t) else {
            return self.c24_grab_release(e);
        };
        if tag == 0 {
            if live != 1 || state != 0x0200 || airborne != 0 {
                return self.c24_grab_release(e);
            }
            let Some(Slot::Player(p)) = slot_of(t) else { return self.c24_grab_release(e) };
            e.b.partner = t;
            let pl = &mut self.players[p];
            pl.b.partner = h;
            pl.b.set_state(0x0206_0000);
            pl.action = 0x40;
            pl.b.kind = 0xE;
            pl.bc7 = 0;
            pl.b.hp_prev = pl.b.hp;
            self.enemy_anim(&mut e.b, 0x1C);
            e.b.act = e.b.act.wrapping_add(2);
        } else {
            // 0x48ED6: an enemy with +$82 set (the poachers): M6-05.
            self.c24_grab_take_enemy(e, h, t, live, state, airborne);
        }
    }

    /// 0x48F2A: the hold check: the target at +$00 = 1, state 0x0206, frame class 0xE. False = carry set = release.
    fn c24_hold_ok(&self, e: &Enemy, h: Handle) -> bool {
        let t = e.m.w(TARGET);
        let (live, state, kind) = if t == h {
            (e.b.live, (e.b.state() >> 16) as u16, e.b.kind)
        } else {
            let b = self.body(t);
            (b.live, (b.state() >> 16) as u16, b.kind)
        };
        live == 1 && state == 0x0206 && kind == 0xE
    }

    /// 0x48F6E: the hero hold.
    fn c24_grab_hold_hero(&mut self, e: &mut Enemy, h: Handle) {
        if self.enemy_step(&mut e.b) {
            return self.c24_grab_release(e); // bcs 0x48F56
        }
        if e.b.param_a != 0 {
            return;
        }
        e.b.param_a = 1;
        if !self.c24_hold_ok(e, h) {
            return self.c24_grab_release(e);
        }
        e.b.atk = 0x3B;
        e.b.knock = 0;
        e.b.hitstop = 0;
        e.b.serial = self.w.attack_serial;
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
        self.enemy_hit_held_player(e, h); // jsr 0x125C0
        let t = e.m.w(TARGET);
        let Some(Slot::Player(p)) = slot_of(t) else { return self.c24_grab_release(e) };
        if self.players[p].action != 0x40 {
            return self.c24_grab_release(e);
        }
        let (dx, facing): (i16, u8) = if e.b.facing != 0 { (-0x18, 1) } else { (0x18, 0) };
        let (x, y) = (e.b.xi().wrapping_add(dx), e.b.yi().wrapping_add(0x40));
        let pl = &mut self.players[p];
        pl.b.set_state(0x0202_0000);
        pl.b.act = 8;
        pl.b.set_xi_keep(x);
        pl.b.set_yi_keep(y);
        pl.b.facing = facing;
        self.snd_request(0x5F, self.snd_pan_x(e.b.xi()) as u32, 0); // jmp 0xA0CC
    }

    /// 0x48ED6: an enemy as the target (every pool A object has +$82 = 4): it must be at +$00 = 1, in state 0x0200, on the ground and counted
    /// (+$7D not negative: dinosaurs and bosses cannot be grabbed). He links both ways, puts it into the held state (0x02060000, frame class 0xE), gives it
    /// his credit byte, and the hold of the enemy (state 4) starts with animation 0x1C. (`move.w $6c(a3),$6e(a3)` with a register left by the caller is
    /// not ported: it only copies HP into the HP before the hit of an object nobody can name.)
    fn c24_grab_take_enemy(&mut self, e: &mut Enemy, h: Handle, t: Handle, live: u8, state: u16, airborne: u8) {
        let Some(Slot::Enemy(i)) = slot_of(t) else { return self.c24_grab_release(e) };
        if live != 1 || state != 0x0200 || airborne != 0 || (self.enemies.slots[i].m.b(ATTACKING) as i8) < 0 {
            return self.c24_grab_release(e);
        }
        e.b.partner = t;
        let en = &mut self.enemies.slots[i];
        en.b.partner = h;
        en.b.set_state(0x0206_0000);
        en.b.kind = 0xE;
        en.b.who = e.b.who;
        self.enemy_anim(&mut e.b, 0x1C);
        e.b.act = e.b.act.wrapping_add(4);
    }

    /// 0x48FF2: the hold of an enemy. The end of the animation lets go; in the frames of class 0xC the held enemy is put into his hand (0x3267A);
    /// at the frame with +$2C clear the hold is checked, the attack 0x3B hurts it (0x12686) and it is thrown (state 6) to the side of the hand.
    fn c24_grab_hold_enemy(&mut self, e: &mut Enemy, h: Handle) {
        if self.enemy_step(&mut e.b) {
            return self.c24_grab_release(e);
        }
        if e.b.kind == 0xC {
            self.c24_place_held(e); // moveq #4,d0; jsr 0x3267A
        }
        if e.b.param_a != 0 {
            return;
        }
        e.b.param_a = 1;
        if !self.c24_hold_ok(e, h) {
            return self.c24_grab_release(e);
        }
        e.b.atk = 0x3B;
        e.b.knock = 0;
        e.b.hitstop = 0;
        e.b.serial = self.w.attack_serial;
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
        self.c24_hurt_held_enemy(e, h); // jsr 0x12686
        let t = e.m.w(TARGET);
        let Some(Slot::Enemy(i)) = slot_of(t) else { return self.c24_grab_release(e) };
        if self.enemies.slots[i].b.kind != 0xE {
            return self.c24_grab_release(e);
        }
        let (dx, facing): (i16, u8) = if e.b.facing != 0 { (-0x40, 1) } else { (0x40, 0) };
        let (x, y) = (e.b.xi().wrapping_add(dx), e.b.yi().wrapping_add(0x88));
        let en = &mut self.enemies.slots[i];
        en.b.set_state(0x0202_0000);
        en.b.act = 6;
        en.b.set_xi_keep(x);
        en.b.set_yi_keep(y);
        en.b.facing = facing;
    }

    /// 0x3267A (d0 = 4): puts the held enemy (+$70) at his hand: the row of the table 0x2E4E0 (entry 4) by the enemy's name and his frame flag +$2D gives dx,
    /// dy, the animation of the held table 0x2E724 and the facing. The position words are his, plus the offsets (mirrored when he faces right).
    fn c24_place_held(&mut self, e: &Enemy) {
        let Some(Slot::Enemy(i)) = slot_of(e.b.partner) else { return };
        let t = &self.assets.enemy.t;
        let base = t.u32(0x2E4E0, 4);
        let w84 = self.enemies.slots[i].m.w(W84) as u32;
        let off = t.i16(base, w84 >> 1);
        let row = base.wrapping_add(off as i32 as u32).wrapping_add((e.b.param_b as u32) << 3);
        let (mut dx, dy, anim, mut facing) = (t.i16(row, 0), t.i16(row, 2), t.u16(row, 4), t.u8(row, 6));
        if e.b.facing != 0 {
            dx = dx.wrapping_neg();
            facing ^= 1;
        }
        let table = t.u32(0x2E724, w84);
        let first = self.assets.sprites.anim(table, anim);
        let (x, y, z) = (e.b.xi().wrapping_add(dx), e.b.yi().wrapping_add(dy), e.b.zi());
        let en = &mut self.enemies.slots[i];
        en.b.set_xi_keep(x);
        en.b.set_yi_keep(y);
        en.b.set_zi_keep(z);
        en.b.facing = facing;
        en.b.load_frame(&self.assets.sprites, first); // 0x1236
    }

    /// 0x12686: the attack 0x3B hurts the held enemy: the credit is nobody (+$02 = 0xFF), the state 0x02020000 with the reaction of the record, the damage
    /// of the record scaled by 0x2B904 with Shivat's +$C6, HP not below 1 for an uncounted enemy (here: 0), the poise, the sound byte 5 of the record (the
    /// code reads one byte too many here, kept).
    fn c24_hurt_held_enemy(&mut self, e: &Enemy, h: Handle) {
        let Some(Slot::Enemy(i)) = slot_of(e.b.partner) else { return };
        let rec = self.assets.player.enemy_attacks[e.b.atk as usize];
        {
            let en = &mut self.enemies.slots[i];
            en.b.attacker = h;
            en.b.serial = e.b.serial;
            en.b.live = 2;
            en.b.who = 0xFF;
            en.b.set_state(0x0202_0000);
            en.m.sb(WAIT, 0);
            en.b.hp_prev = en.b.hp;
        }
        let (damage, _) = self.enemy_damage_core(rec.damage as u8 as u32, e.b.atk, Some(e.m.b(0xC6)), i, true);
        let en = &mut self.enemies.slots[i];
        let hp = (en.b.hp as i16).wrapping_sub(damage as i16);
        en.b.hp = if hp > 0 {
            hp as u16
        } else if (en.m.b(ATTACKING) as i8) < 0 {
            1
        } else {
            0
        };
        en.b.stun = en.b.stun.saturating_sub(rec.stun);
        en.b.reaction = rec.reaction;
        en.b.act = rec.reaction;
        en.b.step = 0;
        en.b.b9a = rec.b7;
        if rec.score != 0 {
            self.snd_hit(rec.score, e.b.xi(), e.b.facing); // jsr 0x962A with a2 = a6 = Shivat
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::Game;
    use dino_assets::AssetPack;

    /// 0x491A4 reads a saved extent of the stack that 0x4BEA is rebuilding.
    #[test]
    fn hurt_list_keeps_its_old_extent_and_observes_overwritten_entries() {
        let mut g = Game::new(Box::leak(Box::new(AssetPack::default())));
        for h in [0xC8F4, 0xC9D4, 0xCAB4] {
            g.combat.push(0x69BA, h);
        }
        assert_eq!(g.enemy_c24_hurt_list(), [0xCAB4, 0xC9D4, 0xC8F4]);
        g.combat.push(0x671E, 0xCB94);
        assert_eq!(g.enemy_c24_hurt_list(), [0xCAB4, 0xC9D4, 0xCB94]);
        for h in [0xCC74, 0xCD54, 0xCE34] {
            g.combat.push(0x671E, h);
        }
        assert_eq!(g.enemy_c24_hurt_list(), [0xCD54, 0xCC74, 0xCB94]);
        g.combat.clear(0x69BA);
        assert!(g.enemy_c24_hurt_list().is_empty());
    }
}
