//! The alternate Blade, pool A class 63 (handler 0x5AA6E..0x5AFC2): the BLADE / RAZOR art (tables 0x6DA40 / 0x6DF2A by the variant high byte)
//! with its own name 0x1C (`+$84` 0x70). It dashes in (attack 0x1C), throws one blade (class 41, on the frame byte 0x2F), waits, then runs off
//! the near side of the screen and is freed there. HP by the name table (36 at every level), the kill score by its name. D3a section 8.1, HT6,
//! HU12. Owner: enemy port.
//!
//! The handler also holds a knife drop (0x5ABC8) and a knife part placement (0x5AE64, like class 14's parts); nothing calls them, so they are
//! not ported.

use super::{off::*, Enemy};
use crate::game::{Game, Handle};

/// Own bytes of the object (the original offsets).
mod o {
    /// `+$A0` byte: 1 after the dash, 0 after the throw. `+$A3` byte: set to 0x1E by the dash and the run away (not read here).
    pub const FLAG: usize = 0xA0;
    pub const A3: usize = 0xA3;
    /// `+$A6` word: the timer of the dash, the wait and the run away. `+$BE` byte: blades left.
    pub const TIMER: usize = 0xA6;
    pub const BLADES: usize = 0xBE;
    /// `+$80` word: the blink time of the death.
    pub const BLINK: usize = 0x80;
}

impl Game {
    /// 0x5AA6E (mode table 0x5AA7C): 0 the init 0x5AA84, 2 alive 0x5AB6E, 4 the death 0x5AF2E, 6 the free 0x5AF68.
    pub(super) fn enemy_class63(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => self.c63_init(e),
            2 => self.c63_alive(e, h, i),
            4 => self.c63_dying(e, h, i),
            _ => self.enemy_free_pool_a(e, i), // 0x5AF68: jmp 0x48FC
        }
    }

    /// 0x5AA84: name 0x1C, `+$84` 0x70, the table by the variant high byte; frame byte 1; animation 0x1E; the box table 0x103C00; speed id 7;
    /// direction 0x18; HP (0x31424); the target (0x32032); poise 10; `+$72` 0xC8; `+$A6` 0x14; the floor (0x3140C); state 0x02000000; a slot
    /// (0x32D90), the target again, aim at it (0x32A70); then the dash (0x5ADF2).
    fn c63_init(&mut self, e: &mut Enemy) {
        e.b.live = 1;
        e.m.sb(B87, 0);
        e.b.id = 0x1C;
        e.m.sw(W84, 0x70);
        e.b.anim_table = if (e.b.var26 >> 8) as u8 == 0 { 0x6DA40 } else { 0x6DF2A };
        e.b.param_b = 1;
        self.enemy_anim(&mut e.b, 0x1E);
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_set_speed(&mut e.b, 7);
        e.b.dir = 0x18;
        self.enemy_set_hp(&mut e.b); // 0x31424, d0 = 0
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        e.b.stun = 0x0A;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.m.sb(ATTACKING, 0);
        e.b.b72 = 0xC8;
        e.m.sw(o::TIMER, 0x14);
        e.m.sb(o::BLADES, 0);
        e.m.sb(WA2, 0);
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sb(ATTACKING, 0);
        e.b.set_state(0x0200_0000);
        e.b.b63 = 0;
        self.lib_slot3_new(e); // 0x32D90
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        self.lib_aim_at_target(e); // 0x32A70
        e.m.sb(o::FLAG, 0);
        self.c63_dash(e);
    }

    /// 0x5AB6E: alive. The kill-all check (0x326F8); by `+$05` (the long table 0x5ABB8): 0 the run 0x5AC50, 2 the hit reaction (0x2A606), 6 held
    /// (0x2B35A); then outside camera X - 0x50 .. + 0x1C0 (signed) mode 6, else the far test (0x173C).
    fn c63_alive(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        self.enemy_kill_all_check(e);
        match e.b.sub {
            0 => self.c63_run(e, h),
            2 => self.enemy_hit_reaction(e, h),
            4 => {} // the entry is the death 0x5AF2E, which no state with `+$05` 4 reaches
            _ => self.enemy_grabbed(e, h),
        }
        let cam = self.cam.s2.cam_x as i16;
        let x = e.b.xi();
        if cam.wrapping_sub(0x50) >= x || cam.wrapping_add(0x1C0) <= x {
            e.b.mode = 6;
            return;
        }
        self.enemy_despawn_if_far(e, i); // jmp 0x173C
    }

    /// 0x5AC50: the poise refill, the terrain (0x12E20); the hit stop, then the wait `+$7A`, else the act (table 0x5ACA2: 0 the throw 0x5AD1E, 2 the
    /// run away 0x5AD12, 4 the wait 0x5AD08, 6 the dash 0x5ACF4); the draw check and the hurtable list.
    fn c63_run(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(POISE_T);
        if t != 0 {
            e.m.sb(POISE_T, t - 1);
            if t == 1 {
                e.b.stun = 0x0A;
            }
        }
        self.terrain_collide_free(&mut e.b); // 0x12E20
        if e.b.hitstop != 0 {
            e.b.hitstop -= 1;
        } else if e.m.b(WAIT) != 0 {
            e.m.sb(WAIT, e.m.b(WAIT) - 1);
        } else {
            match e.b.act {
                0 => self.c63_throw(e, h),
                2 => {
                    self.enemy_move_dir(&mut e.b); // 0x5AD12
                    self.enemy_step(&mut e.b);
                }
                4 => {
                    // 0x5AD08: the wait after the throw.
                    let t = e.m.w(o::TIMER).wrapping_sub(1);
                    e.m.sw(o::TIMER, t);
                    if t == 0 {
                        self.c63_run_away(e);
                    }
                }
                _ => {
                    // 0x5ACF4: the dash until `+$A6` runs out, then the throw (0x5ADD8).
                    let t = e.m.w(o::TIMER).wrapping_sub(1);
                    e.m.sw(o::TIMER, t);
                    if t == 0 {
                        e.m.sb(o::FLAG, 1);
                        e.b.act = 0;
                        e.m.sw(o::TIMER, 0x1E);
                        self.enemy_anim(&mut e.b, 0x1F);
                    } else {
                        self.enemy_move_dir(&mut e.b);
                        self.enemy_step(&mut e.b);
                    }
                }
            }
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x5AD1E: the throw animation; at its end the wait (0x5ADC4: `+$A0` 0, `+$A6` 0x1E, act 4). On the frame byte 0x2F the blade (class 41)
    /// leaves the hand (sound 0x9D44, `+$A0` of the blade = this object), one blade less.
    fn c63_throw(&mut self, e: &mut Enemy, h: Handle) {
        if self.enemy_step(&mut e.b) {
            e.m.sb(o::FLAG, 0);
            e.m.sw(o::TIMER, 0x1E);
            e.b.act = 4;
            return;
        }
        if e.b.param_b != 0x2F {
            return;
        }
        self.snd_9d44(e.b.xi(), e.b.facing);
        let Some(j) = self.enemies.alloc() else { return };
        let n = &mut self.enemies.slots[j];
        n.b.mode = 0;
        n.b.sub = 0;
        n.b.live = 1;
        n.b.k20 = 41 * 4;
        n.b.var26 = 0;
        n.b.facing = e.b.facing;
        super::lib32::put_hi(&mut n.b.x, e.b.xi());
        super::lib32::put_hi(&mut n.b.y, e.b.yi());
        super::lib32::put_hi(&mut n.b.z, e.b.zi());
        n.m.sl(0xA0, 0xFFFF_0000 | h as u32);
        e.m.sb(o::BLADES, e.m.b(o::BLADES).wrapping_sub(1));
    }

    /// 0x5ACAA: the recover entry of the name 0x70 (tables 0x2B56C / 0x4A7FC): state 0x02000200, speed id 5, a slot (0x32D90); after a knock down
    /// (reaction 4) not attacking, a slot, the target (0x32032), aim (0x32A58) and face it (0x32B68); then the run away.
    pub(super) fn c63_recover(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0200);
        self.enemy_set_speed(&mut e.b, 5);
        self.lib_slot3_new(e);
        if e.b.reaction == 4 {
            e.m.sb(ATTACKING, 0);
            self.lib_slot3_new(e);
            let t = self.enemy_pick_target(&e.b);
            e.m.sw(TARGET, t);
            self.lib_aim_at_target(e);
            self.lib_face_target(e);
        }
        self.c63_run_away(e);
    }

    /// 0x5AD7A: the run away: act 2, `+$A6` 0x50, `+$A3` 0x1E, speed id 0xD; to the left (facing 0, direction 0x18) when X is left of camera X
    /// + 0xC0 (signed), else to the right (facing 1, direction 8); animation 0x1E.
    fn c63_run_away(&mut self, e: &mut Enemy) {
        e.b.act = 2;
        e.m.sw(o::TIMER, 0x50);
        e.m.sb(o::A3, 0x1E);
        self.enemy_set_speed(&mut e.b, 0x0D);
        let centre = (self.cam.s2.cam_x as i16).wrapping_add(0xC0);
        let (facing, dir) = if centre > e.b.xi() { (0, 0x18) } else { (1, 8) };
        e.b.facing = facing;
        e.b.dir = dir;
        self.enemy_anim(&mut e.b, 0x1E);
    }

    /// 0x5ADF2: the dash: one blade (`+$BE` 2, the throw takes one), act 6, `+$A6` 0xF, `+$A3` 0x1E; the direction to the slot point (0x103A, then
    /// overwritten), a new slot (0x32D90), face the target (0x32B68); direction 8 (facing right) or 0x18; speed id 0xD; attack 0x1C; animation 0x1E.
    fn c63_dash(&mut self, e: &mut Enemy) {
        e.m.sb(o::BLADES, 2);
        e.b.act = 6;
        e.m.sw(o::TIMER, 0x0F);
        e.m.sb(o::A3, 0x1E);
        self.lib_slot3_new(e);
        self.lib_face_target(e);
        e.b.dir = if e.b.facing != 0 { 8 } else { 0x18 };
        self.enemy_set_speed(&mut e.b, 0x0D);
        self.enemy_begin_attack(&mut e.b, 0x1C);
        self.enemy_anim(&mut e.b, 0x1E);
    }

    /// 0x5AF2E: the death (table 0x5AF3A): step 0 `+$80` 0x3C and the kill score (0x325E6); step 2 counts down and draws on odd counts; past 0 the
    /// free (0x48FC).
    fn c63_dying(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.sub == 0 {
            e.m.sw(o::BLINK, 0x3C);
            let b = e.b.clone();
            self.enemy_award_kill_score(&b);
            e.b.sub = e.b.sub.wrapping_add(2);
        }
        let t = e.m.w(o::BLINK);
        if t == 0 {
            return self.enemy_free_pool_a(e, i);
        }
        let t = t - 1;
        e.m.sw(o::BLINK, t);
        if t & 1 != 0 {
            self.enemy_draw_check(&mut e.b, h);
        }
    }
}
