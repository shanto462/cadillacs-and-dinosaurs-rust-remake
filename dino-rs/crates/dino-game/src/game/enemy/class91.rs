//! Tyrog 2, pool A class 91 (handler 0x654E2..0x676FE): a copy of Tyrog's class 4 (94 percent, D3d 3.8) made by the Episode 8 lab tank. It runs
//! the class 4 code (`class4.rs`, `class4_states.rs`, which test `c4_is_t2`) with these differences: name 0x23 and `+$84` 0x8C; HP from the
//! normal table (0x31424, 300 at every level and player count); counted by the wave census (`+$7D` 0, `+$63` 0); no music, no HUD bar, no
//! controller; it starts in a 15 frame wait (state 6) that ends in the hop of action 3; the kill-all check runs; the hit is the common one
//! (0x2A606, no damage credit, no stage end); the X class of the decision is 0x3293C's; its own sidestep (tables 0x74CC2, 0x74D82, 0x74D42);
//! no X speed minimum in the pounce; the level delays 0x74BA2; the recover entry's off screen branch is a breath; a plain death (1,000 points,
//! no leader flags) and the free. Owner: enemy port.

use super::class4::o;
use super::{off::*, Enemy};
use crate::game::{Game, Handle};

/// The sidestep tables of Tyrog 2 (0x6645E): the X steps (bytes), the Y speeds (words), the Z steps (bytes).
const SIDE_X: u32 = 0x74CC2;
const SIDE_VY: u32 = 0x74D82;
const SIDE_Z: u32 = 0x74D42;

impl Game {
    /// 0x654E2 (mode table 0x654F0): 0 the init 0x654F8, 2 the fight 0x655BA, 4 the death 0x6769E, 6 the free 0x676F8.
    pub(super) fn enemy_class91(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => self.c91_init(e),
            2 => self.c91_fight(e, h, i),
            4 => self.c91_dying(e, h, i),
            _ => self.enemy_free_pool_a(e, i), // jmp 0x48FC
        }
    }

    /// 0x654F8: mode 2, name 0x23, `+$84` 0x8C, the table 0x702C4, palette 0x9D, animation 1, the box table 0x103C00, speed id 7, direction 0x18; HP of
    /// 0x31424; the target; poise 11; `+$72` 0x1E; `+$AC` 0x14; aggressive with the phase timer 1; the floor (0x3140C); `+$7D` 0, `+$63` 0; a slot
    /// (0x32C5E); then 0x6643E: not busy, `+$A8` 6, `+$AC` 0xF, state 6, animation 0.
    fn c91_init(&mut self, e: &mut Enemy) {
        e.b.mode = 2;
        e.b.live = 1;
        e.m.sb(B87, 0);
        e.b.id = 0x23;
        e.m.sw(W84, 0x8C);
        e.b.anim_table = 0x702C4;
        e.b.palette_override = 0x9D;
        e.b.b63 = 2;
        self.enemy_anim(&mut e.b, 1);
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_set_speed(&mut e.b, 7);
        e.b.dir = 0x18;
        self.enemy_set_hp(&mut e.b); // 0x31424
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
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sb(ATTACKING, 0);
        e.b.b63 = 0;
        self.lib_slot6_new(e); // 0x32C5E
        // 0x6643E
        e.m.sb(o::BUSY, 0);
        e.m.sb(o::A8, 6);
        e.m.sw(o::TIMER, 0xF);
        e.b.act = 6;
        self.enemy_anim(&mut e.b, 0);
    }

    /// 0x655BA: the palette and `+$7D` by form, the HP 0 clamp (as class 4); the kill-all check (0x326F8); by `+$05` (the long table 0x6564A):
    /// 0 the AI (class 4's), 2 the common hit 0x2A606, 4 the death 0x6769E, 6 held (0x2B35A); then the play field clamp (0x193E).
    fn c91_fight(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        Self::c4_fight_palette(e);
        self.c4_dead_clamp(e);
        self.enemy_kill_all_check(e); // 0x326F8
        match e.b.sub {
            0 => self.c4_alive(e, h),
            2 => self.enemy_hit_reaction(e, h),
            4 => self.c91_dying(e, h, i),
            _ => self.enemy_grabbed(e, h),
        }
        self.enemy_clamp_to_playfield(e, i); // jmp 0x193E
    }

    /// 0x6769E by `+$05` (table 0x676AA): 0 120 frames and the score (0x325E6, 1,000), falling into 2: the count, drawn on even counts, at
    /// its end mode 6 (drawn); 4: the count, drawn, at its end the free (0x48FC).
    fn c91_dying(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.sub == 4 {
            let t = e.m.w(o::LIE);
            if t == 0 {
                return self.enemy_free_pool_a(e, i);
            }
            e.m.sw(o::LIE, t - 1);
            return self.enemy_draw_check(&mut e.b, h);
        }
        if e.b.sub == 0 {
            e.m.sw(o::LIE, 0x78);
            let b = e.b.clone();
            self.enemy_award_kill_score(&b); // 0x325E6
            e.b.sub = e.b.sub.wrapping_add(2);
        }
        let t = e.m.w(o::LIE);
        if t == 0 {
            e.b.mode = 6;
            return self.enemy_draw_check(&mut e.b, h);
        }
        e.m.sw(o::LIE, t - 1);
        if (t - 1) & 1 == 0 {
            self.enemy_draw_check(&mut e.b, h);
        }
    }

    /// 0x6599C (state 6): `+$AC` counts down, then the hop of action 3 (0x673A8, class 4's 0x389D0); until then face the target and animate.
    pub(super) fn c91_wait(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(o::TIMER).wrapping_sub(1);
        e.m.sw(o::TIMER, t);
        if t == 0 {
            return self.c4_hop(e, h, 0x3C, 0x80_0000, 2);
        }
        self.lib_face_target(e); // 0x32B68
        self.enemy_step(&mut e.b);
    }

    /// 0x6645E: the sidestep (state 4): aim 0x30 beside the target (0x32A88), face, in the air. The X class of 0x3293C plus the Z distance it
    /// leaves in d2 picks the X step of 0x74CC2 (its high byte kept), plus the class again gives the sign of vx (0x300); the last Z window
    /// index * 4 picks vy of 0x74D82; the Z class plus `rand() & 0xF` picks the Z step of 0x74D42 (its sign gives vz). Gravity -0x60, no turn
    /// toward the target, animation 0x15.
    pub(super) fn c91_sidestep_start(&mut self, e: &mut Enemy) {
        e.m.sb(o::OLD_DIR, e.b.dir);
        self.lib_aim_beside_target(e, 0x30);
        self.lib_face_target(e);
        e.m.sb(o::BUSY, 0);
        e.b.airborne = 1;
        let _ = self.rand(); // `rand() & 0xF` into d2 and d3, both lost in the call that follows
        let (xc, zc, d2, d3) = self.c91_classes(e);
        let d0 = xc.wrapping_add(d2);
        let tb = &self.assets.enemy.t;
        let bx = tb.u8(SIDE_X, d0 as i16 as i32 as u32);
        let d3 = d3.wrapping_add(d3);
        e.b.vy = tb.i16(SIDE_VY, d3 as i16 as i32 as u32);
        e.b.ay = -0x60;
        let w = ((d2 & 0xFF00) | bx as u16).wrapping_add(d0);
        e.b.vx = if w == 0 {
            0
        } else if (w as i16) < 0 {
            -0x300
        } else {
            0x300
        };
        e.b.ax = 0;
        let r = self.rand() & 0xF;
        let bz = self.assets.enemy.t.u8(SIDE_Z, zc.wrapping_add(r) as i16 as i32 as u32);
        e.b.vz = if bz == 0 {
            0
        } else if (bz as i8) < 0 {
            -0x300
        } else {
            0x300
        };
        e.b.az = 0;
        e.b.act = 4;
        e.m.sb(o::OLD_DIR, e.b.dir);
        self.enemy_anim(&mut e.b, 0x15);
    }

    /// 0x3293C with what it leaves: the X class (d0), the Z class (d1), d2 (the last Z window's distance) and d3 (the last Z index * 2).
    fn c91_classes(&self, e: &Enemy) -> (u16, u16, u16, u16) {
        let (xc, zc) = self.lib_distance_class(e);
        let t = self.lib_target(e);
        let tab = &self.assets.enemy.t;
        let mut last = 0u32;
        for i in (0..4u32).rev() {
            last = i;
            let d2 = t.zi().wrapping_add(tab.u16(0x329A8, 2 * i) as i16).wrapping_sub(e.b.zi()) as u16;
            if d2 > tab.u16(0x329B0, 2 * i) {
                break;
            }
        }
        let d2 = t.zi().wrapping_add(tab.u16(0x329A8, 2 * last) as i16).wrapping_sub(e.b.zi()) as u16;
        (xc, zc, d2, (2 * last) as u16)
    }
}
