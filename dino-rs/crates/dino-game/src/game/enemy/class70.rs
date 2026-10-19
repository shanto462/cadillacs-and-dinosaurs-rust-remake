//! Biker, pool A class 70 (handler 0x5C356..0x5C801). Still until the wait in +$A6 runs out, then rides. A jump variant
//! hops first. A hero in front may eat a kick (attack 0x48). Death drops from the enemy table and leaves a rider
//! (class 2, variant 0x208). Owner: enemy port. D3a section 10, H13.

use super::{off::*, Enemy};
use crate::game::{Game, Handle};

/// Words and bytes read at run time (`dino-extract` TABLES).
mod t {
    /// 0x5C39E: vx, and for a jump variant vy and ay.
    pub const SPEED: u32 = 0x5C802;
    /// 0x5C586: 16 kick chances, `rand() & 15`.
    pub const KICK: u32 = 0x5C808;
    /// Animation table stored at +$40 (0x5C412).
    pub const ANIM: u32 = 0x5C6F0;
}

impl Game {
    /// 0x5C356 (mode table 0x5C362): 0 the init and the wait (0x5C36A), 2 the ride or the death (0x5C47E),
    /// 4 the death from the drop on (0x5C6CE), 6 free (0x5C6EA).
    pub(super) fn enemy_class70(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                if e.b.sub == 0 {
                    self.enemy_c70_init(e);
                }
                // 0x5C44E. The frame the wait goes negative starts the ride in this same frame.
                if self.enemy_c70_wait(e, h) {
                    self.enemy_c70_run(e, h, i);
                }
            }
            2 => self.enemy_c70_run(e, h, i),
            4 => self.enemy_c70_death_tail(e, h, i),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x5C36A, only when +$05 is 0. Falls into the wait (the caller decrements +$A6).
    fn enemy_c70_init(&mut self, e: &mut Enemy) {
        let cam = (self.cam.s2.cam_x as i16).wrapping_add(0xC0);
        let facing = u8::from((cam as u16) >= (e.b.xi() as u16));
        e.b.facing = facing;
        e.m.sb(REPOS, facing); // +$A5, the facing restored at death
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sb(DETOUR_T, 0x1E);
        e.m.sb(DETOUR, 0);
        let tb = &self.assets.enemy.t;
        let spd = tb.i16(t::SPEED, 0);
        e.m.sw(WA0, spd as u16);
        e.b.vx = if facing == 0 { spd.wrapping_neg() } else { spd };
        if e.b.var26 >> 8 != 0 {
            e.b.vy = tb.i16(t::SPEED, 2);
            e.b.ay = tb.i16(t::SPEED, 4);
            e.m.sb(TAUNTED, 2); // hops left, +$A4
            e.b.airborne = 1;
            e.b.act = 4;
        }
        e.b.box_table = 0x0010_3000;
        e.b.multi_box = false;
        e.b.hitstop = 0;
        e.b.who = 0xFF;
        e.b.id = 0;
        e.m.sb(ATTACKING, 0); // counted
        e.m.sb(B87, 0);
        e.b.b63 = 0xFF;
        e.b.hp = 8;
        e.b.hp_prev = 8;
        e.b.hp_bar = 8;
        e.b.anim_table = t::ANIM;
        let anim = e.b.var26 >> 8; // +$26
        self.enemy_anim(&mut e.b, anim); // 0x120E
        e.b.atk = 0x47;
        e.b.knock = 0;
        e.b.hitstop = 0;
        e.b.serial = self.w.attack_serial;
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
        self.enemy_c70_engine(e); // 0xA128
        e.m.sw(WA2, 0);
        e.b.sub = e.b.sub.wrapping_add(2);
    }

    /// 0x5C44E: +$A6 counts down. A negative result starts the ride (0x5C46E) and returns true. Otherwise the
    /// animation, the hostile list, the wide despawn (0x173C) and the draw, and returns false.
    fn enemy_c70_wait(&mut self, e: &mut Enemy, h: Handle) -> bool {
        let n = e.m.b(DETOUR_T).wrapping_sub(1);
        e.m.sb(DETOUR_T, n);
        if (n as i8) < 0 {
            e.m.sb(DETOUR_T, 0x28);
            e.b.sub = 0;
            e.b.mode = 2;
            return true;
        }
        self.enemy_step(&mut e.b);
        self.enemy_monster_register(h);
        self.enemy_despawn_if_far(e, 0);
        self.enemy_draw_check(&mut e.b, h);
        false
    }

    /// 0x5C47E (longs at 0x5C48A): sub 0 rides (0x5C49A), sub 2, 4 and 6 die (0x5C6B4).
    fn enemy_c70_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.sub == 0 {
            self.enemy_c70_ride(e, h);
        } else {
            self.enemy_c70_death(e, h, i);
        }
    }

    /// 0x5C49A: one act (word table 0x5C4BE), then the animation, the hostile list, the wide despawn and the draw.
    /// Act 0 and act 2 end in `jmp $189C`: motion returns to this tail, it does not skip it.
    fn enemy_c70_ride(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => self.enemy_c70_ground(e),
            2 => self.enemy_c70_speed(e),
            4 => self.enemy_c70_jump(e),
            _ => {}
        }
        self.enemy_step(&mut e.b);
        self.enemy_monster_register(h);
        self.enemy_despawn_if_far(e, 0);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x5C4C4: the first hero in front (0..255 px ahead, depth -5..+4) ends the plain ride. 7 of 16 rolls (table
    /// 0x5C808) start the kick. Either way the speed code of act 2 runs, and later frames stay in act 2.
    fn enemy_c70_ground(&mut self, e: &mut Enemy) {
        if enemy_c70_hero_ahead(self, e) {
            e.b.act = 2;
            let roll = (self.rand() & 0xF) as u32;
            if self.assets.enemy.t.u8(t::KICK, roll) != 0 {
                e.b.anim_table = t::ANIM;
                self.enemy_anim(&mut e.b, 1);
                e.b.atk = 0x48;
                e.b.knock = 0;
                e.b.hitstop = 0;
                e.b.serial = self.w.attack_serial;
                self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
            }
        }
        self.enemy_c70_speed(e);
    }

    /// 0x5C5BA: while +$A6 counts down the speed stays. The frame it hits 0, and every frame after, the exhaust
    /// runs and +$A0 climbs by +$A2 (which climbs by 0x12) until +$A0 is at least 0x700. Then motion.
    fn enemy_c70_speed(&mut self, e: &mut Enemy) {
        if e.m.b(DETOUR_T) != 0 {
            let n = e.m.b(DETOUR_T).wrapping_sub(1);
            e.m.sb(DETOUR_T, n);
            if n != 0 {
                e.b.motion();
                return;
            }
            e.m.sb(DETOUR, 0x1A);
            self.enemy_c70_engine(e);
        }
        self.enemy_c70_exhaust(e);
        if e.m.w(WA0) < 0x700 {
            let a2 = e.m.w(WA2).wrapping_add(0x12);
            e.m.sw(WA2, a2);
            let a0 = e.m.w(WA0).wrapping_add(a2);
            e.m.sw(WA0, a0);
            e.b.vx = if e.b.facing == 0 { (a0 as i16).wrapping_neg() } else { a0 as i16 };
        }
        e.b.motion();
    }

    /// 0x5C60A: the jump. In the air, only the exhaust. A landing with hops left bounces (vy = -vy / 2). The last
    /// landing stands, plays the engine and joins act 2. The exhaust's `rts` (or the dust's) returns to the ride tail.
    fn enemy_c70_jump(&mut self, e: &mut Enemy) {
        e.b.motion();
        self.enemy_floor_collision(e);
        if e.b.airborne != 0 {
            self.enemy_c70_exhaust(e);
            return;
        }
        e.m.sb(DETOUR, 4);
        e.b.vy = e.b.vy.wrapping_neg() >> 1; // neg.w ; asr.w #1
        e.b.airborne = 1;
        let hops = e.m.b(TAUNTED).wrapping_sub(1);
        e.m.sb(TAUNTED, hops);
        if hops != 0 {
            self.enemy_c70_exhaust(e);
            return;
        }
        e.b.vy = 0;
        e.b.ay = 0;
        e.b.airborne = 0;
        e.b.anim_table = t::ANIM;
        self.enemy_anim(&mut e.b, 0);
        self.enemy_c70_engine(e);
        e.m.sb(DETOUR, 0x1A);
        e.m.sb(DETOUR_T, 0);
        e.b.act = 2;
        self.enemy_c70_exhaust(e);
    }

    /// 0x5C670: +$A7 counts down. A set bit 2 of the new value makes no puff. Otherwise dust (0x92EA4, variant 0)
    /// behind the bike: facing 0 at x+0x21, y-1; facing right at x-0x21, y+1.
    fn enemy_c70_exhaust(&mut self, e: &mut Enemy) {
        if e.m.b(DETOUR) == 0 {
            return;
        }
        let n = e.m.b(DETOUR).wrapping_sub(1);
        e.m.sb(DETOUR, n);
        if n & 4 != 0 {
            return;
        }
        let (mut dx, mut dy) = (0x21i16, -1i16);
        if e.b.facing != 0 {
            dx = dx.wrapping_neg();
            dy = dy.wrapping_neg();
        }
        self.fx_spawn_dust_at(e.b.xi().wrapping_add(dx), e.b.yi().wrapping_add(dy), e.b.zi(), 0);
    }

    /// 0xA128: the engine. Bit 1 of G.$43E picks 0x7F or 0x90. d1 is the pan byte, d2 is 0.
    fn enemy_c70_engine(&mut self, e: &Enemy) {
        let pan = self.snd_pan_x(e.b.xi()) as u32;
        let old = self.snd_toggle_e(1);
        self.snd_request(if old { 0x7F } else { 0x90 }, pan, 0);
    }

    /// 0xA164: the fall. Bit 0 of G.$43E picks 0x7A or 0xFB.
    fn enemy_c70_fall_snd(&mut self, e: &Enemy) {
        let pan = self.snd_pan_x(e.b.xi()) as u32;
        let old = self.snd_toggle_e(0);
        self.snd_request(if old { 0x7A } else { 0xFB }, pan, 0);
    }

    /// 0x5C6B4: the death. Sound, the spawn facing, the effect 0x94640 (variant 0x1C), then the tail.
    fn enemy_c70_death(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        self.enemy_c70_fall_snd(e);
        e.b.facing = e.m.b(REPOS);
        self.fx_spawn_class21(&e.b, 0x1C, e.b.facing);
        self.enemy_c70_death_tail(e, h, i);
    }

    /// 0x5C6CE: the enemy-table drop (0x24B36), the rider (0x33802), the kill credit (0x2AF9A), 200 points
    /// (table 0x10292 offset 0x18) and the free (0x48FC). Mode 4 enters here.
    fn enemy_c70_death_tail(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        self.item_drop_from_enemy(e);
        self.enemy_c70_rider(e);
        self.enemy_c70_credit(e);
        self.hud_add_score(h, 0x18);
        self.enemy_free_pool_a(e, i);
    }

    /// 0x33802: a pool A object, class word 8 (class 2), variant 0x208, this facing, this X and Z, Y word 0x20.
    /// A full pool skips it (the carry of 0x483C). The spawn is before the free, so this slot stays taken.
    fn enemy_c70_rider(&mut self, e: &Enemy) {
        let Some(j) = self.enemy_spawn_var(8, 0x208) else { return };
        let r = &mut self.enemies.slots[j];
        r.b.facing = e.b.facing;
        r.b.set_xi_keep(e.b.xi());
        r.b.set_yi_keep(0x20);
        r.b.set_zi_keep(e.b.zi());
        r.m.sb(DROP, 0);
    }

    /// 0x2AF9A: the kill credit, then the aggression roll (0x182A2). The same block as `enemy_getup_or_die`.
    /// `who` is the hero the hit pass stored. A negative who credits nobody.
    fn enemy_c70_credit(&mut self, e: &Enemy) {
        let w = e.b.who;
        if (w as i8) >= 0 {
            if self.spawn.g.car_mode == 0 {
                let i = (w & 3) as usize % 3;
                self.spawn.g.kills[i] = self.spawn.g.kills[i].wrapping_add(1);
            } else {
                for k in &mut self.spawn.g.kills {
                    *k = k.wrapping_add(1);
                }
            }
        }
        self.rank_reroll_aggression();
    }
}

/// 0x5C4C4 / 0x5C4FE / 0x5C538: players at 0xB274, stride 0x180. Ahead is the hero's X minus ours when facing
/// right, and the swap when facing left. Negative or 0x100 and above misses. Depth is Z - ours + 5, below 10.
fn enemy_c70_hero_ahead(g: &Game, e: &Enemy) -> bool {
    for i in 0..3 {
        let p = g.body(Game::player_handle(i));
        if p.live == 0 {
            continue;
        }
        let d = if e.b.facing == 0 { e.b.xi().wrapping_sub(p.xi()) } else { p.xi().wrapping_sub(e.b.xi()) };
        if d < 0 || (d as u16) >= 0x100 {
            continue;
        }
        let dz = p.zi().wrapping_sub(e.b.zi()).wrapping_add(5);
        if (dz as u16) < 10 {
            return true;
        }
    }
    false
}
