//! The bomb carriers, pool A class 64 (handler 0x5AFC2..0x5B606): the alternate Punk (name 0x1D, variant high byte 0) and the alternate Wrench T.
//! (name 0x24, variant high byte non zero). Each record makes two objects: the carrier and its dynamite (class 77, `class77.rs`). D3a section 8.2,
//! HU13, HU26.
//!
//! The carrier walks in from its side, then plays a small byte code list (`+$A2` points into the table 0x5B326 by the variant high byte: 0 hop
//! across, 2 jump in place, 4 the throw pose; 0xFF ends it), then a second list (0x5B4E6 by the variant low byte) that ends in a run off the screen.
//! A knock down (reaction 4 or 0x30) sends it running away (0x5B54A). HP is the constant 20. Owner: enemy port.

use super::{off::*, Enemy};
use crate::game::{Game, Handle};

/// Own bytes of the carrier (the original offsets).
pub(super) mod o {
    /// `+$A0` word: the X where the run started. `+$A2` long: the position in the byte code list (a ROM address, as the original keeps it).
    pub const START_X: usize = 0xA0;
    pub const LIST: usize = 0xA2;
    /// `+$A6` byte: the step of the current code. `+$A8` word: the dynamite (class 77). `+$80` word: the blink time of the death.
    pub const STEP: usize = 0xA6;
    pub const STICK: usize = 0xA8;
    pub const BLINK: usize = 0x80;
    /// The two byte code tables (read at run time through `enemy.t`, dino-extract/src/boss.rs).
    pub const FIRST_LISTS: u32 = 0x5B326;
    pub const RUN_LISTS: u32 = 0x5B4E6;
}

impl Game {
    /// 0x5AFC2 (mode table 0x5AFCE): 0 the start (0x5AFD6, by `+$05`: 0 the pair, 2 the init, 4 the entrance, 6 alive), 2 alive (0x5B25A), 4 the death
    /// (0x5B5A6), 6 the free (0x5B5E4).
    pub(super) fn enemy_class64(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => match e.b.sub {
                0 => self.c64_make_pair(e, h, i),
                2 => self.c64_init(e),
                4 => self.c64_entrance(e, h),
                _ => {
                    // 0x5B24E: alive from the next code on, in this frame.
                    e.b.mode = e.b.mode.wrapping_add(2);
                    e.b.sub = 0;
                    e.b.act = 0;
                    e.b.step = 0;
                    self.c64_alive(e, h, i);
                }
            },
            2 => self.c64_alive(e, h, i),
            4 => self.c64_dying(e, h),
            _ => self.enemy_free_pool_a(e, i), // 0x5B5E4: jmp 0x48FC
        }
    }

    /// 0x5AFEC: a second pool A object (a full pool frees this one, 0x5B5E4). The lower address of the two is the carrier (class 64, `+$05` = 2,
    /// the variant, direction, position and drop row of this object, `+$A8` = the other); the higher one is the dynamite (class 77, `+$A0` = the
    /// carrier, the carrier's variant).
    fn c64_make_pair(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        let Some(n) = self.enemies.alloc() else {
            return self.enemy_free_pool_a(e, i);
        };
        let nh = self.enemies.handle(n);
        let carrier_is_new = (h as i16) >= (nh as i16); // cmpa.w a0,a1 ; bge
        let (var, dir, x, y, z, drop) = (e.b.var26, e.b.dir, e.b.xi(), e.b.yi(), e.b.zi(), e.m.b(DROP));
        let (carrier_h, stick_h) = if carrier_is_new { (nh, h) } else { (h, nh) };
        {
            let c = if carrier_is_new { &mut self.enemies.slots[n] } else { &mut *e };
            c.b.live = 1;
            c.b.k20 = 64 * 4;
            c.b.var26 = var;
            c.b.dir = dir;
            c.b.x = (c.b.x & 0xFFFF) | ((x as i32) << 16);
            c.b.y = (c.b.y & 0xFFFF) | ((y as i32) << 16);
            c.b.z = (c.b.z & 0xFFFF) | ((z as i32) << 16);
            c.m.sw(o::STICK, stick_h);
            c.m.sb(DROP, drop);
            c.b.sub = 2;
        }
        let s = if carrier_is_new { &mut *e } else { &mut self.enemies.slots[n] };
        s.b.live = 1;
        s.b.k20 = 77 * 4;
        s.m.sw(0xA0, carrier_h);
        s.b.var26 = var;
    }

    /// 0x5B052: the init: Punk alt (name 0x1D, `+$84` 0x74, table 0x5B606) or Wrench T. alt (name 0x24, `+$84` 0x90, table 0x5B77C) by the variant's
    /// high byte; HP and bar 20; the box table 0x103000; animation 0; poise 10; counted; `+$72` 0x3C; with a variant low byte the side away from
    /// most heroes (0x5B0FA); the floor (0x3140C).
    fn c64_init(&mut self, e: &mut Enemy) {
        e.b.sub = e.b.sub.wrapping_add(2);
        let (id, w84, table) = if e.b.var26 >> 8 == 0 { (0x1D, 0x74, 0x5B606) } else { (0x24, 0x90, 0x5B77C) };
        e.b.id = id;
        e.m.sw(W84, w84);
        e.b.anim_table = table;
        e.b.hp_bar = 0x14;
        e.b.hp = 0x14;
        e.b.hp_prev = 0x14;
        e.b.box_table = 0x103000;
        e.b.multi_box = false;
        self.enemy_anim(&mut e.b, 0);
        e.b.stun = 0x0A;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.m.sb(ATTACKING, 0);
        e.m.sb(B87, 0);
        e.b.b63 = 0;
        e.b.b72 = 0x3C;
        if e.b.var26 & 0xFF != 0 {
            self.c64_side(e);
        }
        self.enemy_snap_to_floor(e); // 0x3140C
    }

    /// 0x5B0FA: count the heroes in play left (-1) and right (+1) of camera X + 0xC0 (`bge` for left, signed); more on the left puts the carrier
    /// at camera X + 0x1C0, else at camera X - 0x40.
    fn c64_side(&mut self, e: &mut Enemy) {
        let d0 = (self.cam.s2.cam_x as i16).wrapping_add(0xC0);
        let mut d2 = 0i16;
        for p in &self.players {
            if p.b.live == 0 {
                continue;
            }
            if d0 >= p.b.xi() {
                d2 -= 1;
            } else {
                d2 += 1;
            }
        }
        let x = if d2 < 0 { d0.wrapping_add(0x100) } else { d0.wrapping_sub(0x100) };
        e.b.x = (e.b.x & 0xFFFF) | ((x as i32) << 16);
    }

    /// 0x5B14C: the entrance by the variant's high byte (table 0x5B16A: 0 the Punk 0x5B16E, 2 the Wrench 0x5B1DE), then the draw check (0x49CA) and the
    /// hurtable list (0x4BEA). Act 0: remember X, the animation (1 or 6) and the speed id (0x1F or 0xB), face the screen centre (right when the start
    /// X is at or left of camera X + 0xC0); act 2: within 0x40 of the centre the carrier is alive from the next frame (`+$05` 6); move (0x1862), animate.
    fn c64_entrance(&mut self, e: &mut Enemy, h: Handle) {
        let wrench = e.b.var26 >> 8 != 0;
        if e.b.act == 0 {
            e.m.sw(o::START_X, e.b.xi() as u16);
            self.enemy_anim(&mut e.b, if wrench { 6 } else { 1 });
            self.enemy_set_speed(&mut e.b, if wrench { 0x0B } else { 0x1F });
            let centre = (self.cam.s2.cam_x as i16).wrapping_add(0xC0);
            let (dir, facing) = if centre.wrapping_sub(e.m.w(o::START_X) as i16) >= 0 { (8, 1) } else { (0x18, 0) };
            e.b.dir = dir;
            e.b.facing = facing;
            e.b.act = e.b.act.wrapping_add(2);
        }
        let d = (self.cam.s2.cam_x as i16).wrapping_add(0xC0).wrapping_sub(e.b.xi()).wrapping_add(0x40) as u16;
        if d < 0x80 {
            e.b.sub = e.b.sub.wrapping_add(2);
        }
        self.enemy_move_dir(&mut e.b); // 0x1862
        self.enemy_step(&mut e.b); // 0x121E
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x5B25A: alive. The kill-all check (0x326F8); by `+$05` (the long table 0x5B27C): 0 the run of the codes (0x5B28C), 2 the hit reaction (0x2A606),
    /// 6 held (0x2B35A); then with `+$06` set the screen test 0x5B5EA, else the far test 0x173C.
    fn c64_alive(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        self.enemy_kill_all_check(e);
        match e.b.sub {
            0 => self.c64_run(e, h),
            2 => self.enemy_hit_reaction(e, h),
            4 => {} // the table entry is 0: never reached
            _ => self.enemy_grabbed(e, h),
        }
        if e.b.act != 0 {
            // 0x5B5EA: outside camera X - 0x20 .. + 0x1A0 (unsigned) the carrier is freed in the next frame.
            let d = e.b.xi().wrapping_sub(self.cam.s2.cam_x as i16).wrapping_add(0x20) as u16;
            if d >= 0x1C0 {
                e.b.mode = 6;
            }
        } else {
            self.enemy_despawn_if_far(e, i); // jmp 0x173C
        }
    }

    /// 0x5B28C: the poise refill, the act (table 0x5B2BE: 0 the first list 0x5B2E8, 2 the run list 0x5B4B8, 4 the run away 0x5B556), the terrain
    /// (0x12E20), the draw check and the hurtable list.
    fn c64_run(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(POISE_T);
        if t != 0 {
            e.m.sb(POISE_T, t - 1);
            if t == 1 {
                e.b.stun = 0x0A;
            }
        }
        match e.b.act {
            0 => self.c64_first_list(e),
            2 => self.c64_run_list(e),
            _ => self.c64_run_away(e),
        }
        self.terrain_collide_free(&mut e.b); // 0x12E20
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x5B2C4: the recover entry of the names 0x74 and 0x90 (table 0x2B56C): state 0x02000000; after a knock down (reaction 4 or 0x30) the run away,
    /// else animation 0 and the first list again (from its start: the step is 0).
    pub(super) fn c64_recover(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0000);
        if e.b.reaction == 4 || e.b.reaction == 0x30 {
            return self.c64_run_away_start(e);
        }
        self.enemy_anim(&mut e.b, 0);
        self.c64_first_list(e);
    }

    /// The byte code at `+$A2`.
    fn c64_code(&self, e: &Enemy) -> u8 {
        self.assets.enemy.t.u8(e.m.l(o::LIST), 0)
    }

    /// 0x5B2E8: act 0: a wait (`+$7A`), then the first list: step 0 points `+$A2` at 0x5B326 + the variant's high byte * 2 (0x5B308); step 2 runs the
    /// code (0x5B32E; table 0x5B342: 0 hop 0x5B348, 2 jump 0x5B3E6, 4 throw pose 0x5B470); a negative code starts the run list (0x5B4B0).
    fn c64_first_list(&mut self, e: &mut Enemy) {
        let w = e.m.b(WAIT);
        if w != 0 {
            e.m.sb(WAIT, w - 1);
            if w != 1 {
                return;
            }
        }
        if e.b.step == 0 {
            e.b.step = e.b.step.wrapping_add(2);
            let hi = (e.b.var26 >> 8) as u32;
            e.m.sl(o::LIST, o::FIRST_LISTS + 2 * hi);
            e.m.sw(o::STEP, 0); // clr.w $A6 (and the byte +$A7)
        }
        let code = self.c64_code(e);
        if (code as i8) < 0 {
            e.b.set_state(0x0200_0200); // 0x5B4B0
            return self.c64_run_list(e);
        }
        self.c64_op(e, code);
    }

    /// 0x5B4B8: act 2: step 0 points `+$A2` at 0x5B4E6 + the variant's low byte * 2; step 2 runs the code (table 0x5B4FE: 0 the hop 0x5B348, 2 the run
    /// off 0x5B502). These lists have no end test.
    fn c64_run_list(&mut self, e: &mut Enemy) {
        if e.b.step == 0 {
            e.b.step = e.b.step.wrapping_add(2);
            let lo = (e.b.var26 & 0xFF) as u32;
            e.m.sl(o::LIST, o::RUN_LISTS + 2 * lo);
            e.m.sw(o::STEP, 0);
        }
        match self.c64_code(e) {
            0 => self.c64_hop(e, true),
            _ => self.c64_run_off(e),
        }
    }

    /// One code of the first list.
    fn c64_op(&mut self, e: &mut Enemy, code: u8) {
        match code {
            0 => self.c64_hop(e, true),
            2 => self.c64_hop(e, false),
            _ => self.c64_throw_pose(e),
        }
    }

    /// 0x5B348 (`across`) / 0x5B3E6: a hop by `+$A6`. Step 0: animation 5 with vx +-0x300 (+ when the carrier is right of camera X + 0xC0, signed), or
    /// animation 4 with vx 0; vy 0x600, gravity -0x50. Step 2: until the frame byte `+$2D` is set the animation runs; then in the air (fall into 4).
    /// Step 4: fly (0x189C) and animate until the landing, then animation 3 (fall into 6). Step 6: at the animation's end the next code.
    fn c64_hop(&mut self, e: &mut Enemy, across: bool) {
        match e.m.b(o::STEP) {
            0 => {
                e.m.sb(o::STEP, 2);
                if across {
                    self.enemy_anim(&mut e.b, 5);
                    let centre = (self.cam.s2.cam_x as i16).wrapping_add(0xC0);
                    e.b.vx = if centre < e.b.xi() { 0x300 } else { -0x300 };
                } else {
                    self.enemy_anim(&mut e.b, 4);
                    e.b.vx = 0;
                }
                e.b.ax = 0;
                e.b.vy = 0x600;
                e.b.ay = -0x50;
                e.b.vz = 0;
                e.b.az = 0;
            }
            2 => {
                if e.b.param_b != 0 {
                    self.enemy_step(&mut e.b);
                    return;
                }
                e.b.airborne = 1;
                e.m.sb(o::STEP, 4);
                self.c64_hop_air(e);
            }
            4 => self.c64_hop_air(e),
            _ => self.c64_hop_land(e),
        }
    }

    /// 0x5B3B6 / 0x5B440.
    fn c64_hop_air(&mut self, e: &mut Enemy) {
        if e.b.airborne != 0 {
            e.b.motion();
            self.enemy_step(&mut e.b);
            return;
        }
        e.m.sb(o::STEP, 6);
        self.enemy_anim(&mut e.b, 3);
        self.c64_hop_land(e);
    }

    /// 0x5B3D4 / 0x5B45E: at the animation's end the next code (`+$A2` + 1, `+$A6` = 0).
    fn c64_hop_land(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            e.m.sl(o::LIST, e.m.l(o::LIST).wrapping_add(1));
            e.m.sb(o::STEP, 0);
        }
    }

    /// 0x5B470: the throw pose: step 0 animation 7 (falls into 2); step 2 at the animation's end animation 0 and the next code.
    fn c64_throw_pose(&mut self, e: &mut Enemy) {
        if e.m.b(o::STEP) == 0 {
            e.m.sb(o::STEP, 2);
            self.enemy_anim(&mut e.b, 7);
        }
        if self.enemy_step(&mut e.b) {
            e.m.sb(o::STEP, 4);
            self.enemy_anim(&mut e.b, 0);
            e.m.sl(o::LIST, e.m.l(o::LIST).wrapping_add(1));
            e.m.sb(o::STEP, 0);
        }
    }

    /// 0x5B504: the run off: step 0 remembers X, animation 1, speed id 0x20, turns round (facing + 1, direction bit 4), then moves; step 2 moves
    /// (0x1862) and animates.
    fn c64_run_off(&mut self, e: &mut Enemy) {
        if e.m.b(o::STEP) == 0 {
            e.m.sw(o::START_X, e.b.xi() as u16);
            self.enemy_anim(&mut e.b, 1);
            self.enemy_set_speed(&mut e.b, 0x20);
            e.b.facing = e.b.facing.wrapping_add(1) & 1;
            e.b.dir ^= 0x10;
            e.m.sb(o::STEP, 2);
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x5B54A: the run away after a knock down: state 0x02000400, `+$A6` = 0, then act 4.
    fn c64_run_away_start(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0400);
        e.m.sb(o::STEP, 0);
        self.c64_run_away(e);
    }

    /// 0x5B556: act 4: step 0 animation 1, speed id 0x20, away from the screen centre (left when it is at or left of camera X + 0xC0, `bpl`); then it
    /// moves (0x1862) and animates.
    fn c64_run_away(&mut self, e: &mut Enemy) {
        if e.m.b(o::STEP) == 0 {
            self.enemy_anim(&mut e.b, 1);
            self.enemy_set_speed(&mut e.b, 0x20);
            let centre = (self.cam.s2.cam_x as i16).wrapping_add(0xC0);
            let (facing, dir) = if centre.wrapping_sub(e.b.xi()) >= 0 { (0, 0x18) } else { (1, 8) };
            e.b.facing = facing;
            e.b.dir = dir;
            e.m.sb(o::STEP, 2);
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x5B5A6: the death (table 0x5B5B2): step 0 the kill score (0x325E6) and 120 frames; step 2 counts down and draws on odd counts (a blink); at the
    /// end mode 6.
    fn c64_dying(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.sub == 0 {
            let b = e.b.clone();
            self.enemy_award_kill_score(&b);
            e.m.sw(o::BLINK, 0x78);
            e.b.sub = e.b.sub.wrapping_add(2);
        }
        let t = e.m.w(o::BLINK);
        if t == 0 {
            e.b.mode = e.b.mode.wrapping_add(2);
            return;
        }
        let t = t - 1;
        e.m.sw(o::BLINK, t);
        if t & 1 != 0 {
            self.enemy_draw_check(&mut e.b, h);
        }
    }
}
