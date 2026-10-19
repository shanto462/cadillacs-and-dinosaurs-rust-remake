//! Morgue (class 40), handler 0x53E08.
use super::{
    Enemy,
    boss_common::{BossAi, BossRecover},
    class36::morgan_cell,
};
use crate::game::{Game, Handle, PaletteRam};

impl Game {
    /// 0x53E08: Morgue mode dispatch (init 0, fight 2, dying 4, dead 6).
    pub(super) fn enemy_class40(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => self.c40_init(e, h, i),
            2 => self.c40_fight(e, h, i),
            4 => self.c40_dying(e, h),
            6 => self.enemy_draw_check(&mut e.b, h), // 0x55860
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x53E1E: Morgue init and palette setup.
    fn c40_init(&mut self, e: &mut Enemy, h: Handle, _i: usize) {
        e.b.mode = 2;
        e.m.sb(0x87, 0);
        e.b.id = 0x16; // MORGUE name id
        e.m.sw(0x84, 0x58);
        e.b.anim_table = 0x6EA48;
        e.b.b63 = 2;
        self.enemy_anim(&mut e.b, 11);
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_set_speed(&mut e.b, 7);
        e.b.dir = 0x18;
        self.enemy_set_boss_hp(&mut e.b, 5); // Row 5: 512 1P, 576 2P, 768 3P
        self.boss_pick_target(e);
        e.b.stun = 11; // Poise 11
        e.m.sb(0x79, 0);
        e.m.sb(0x7A, 0);
        e.m.sb(0x7B, 0);
        e.m.sb(0x7D, 0);
        e.b.b72 = 0xC8; // 200
        e.m.sw(0xAC, 0x14); // 20
        e.m.sb(0xBC, 0);
        e.m.sb(0xB9, 0);
        e.m.sw(0xA6, 1);
        e.m.sb(0xA0, 1); // Aggressive phase
        e.m.sb(0xAA, 0);
        e.m.sb(0xAE, 0);
        self.w.morgue_7054 = 0;
        self.enemy_floor_collision(e);
        e.m.sb(0x7D, 0xFF); // Census flag: not counted
        self.lib_slot6_new(e);
        self.hud.boss = h;
        self.spawn.g.b4fc = 0;
        self.boss_spawn_controller(h); // 0x7BEDC
        e.b.palette_override = 0x9D;
        self.w.morgue_init_hp = e.b.hp;

        // 0x53EE0: Rewrite OBJ palette 0x1D (16 words from 0xB7D32) preserving brightness nibble
        for n in 0..16 {
            let k = PaletteRam::OBJ + 0x1D * 16 + n;
            self.pal.words[k] = (self.pal.words[k] & 0xF000)
                | self.assets.enemy.t.u16(0xB7D32, n as u32 * 2);
        }
        self.pal.dirty = true;

        // 0x55174: Prepare intro state 0x0E
        e.m.sb(0xA1, 1); // Busy
        e.m.sw(0xAC, 0x1E); // 30
        e.b.act = 0x0E;
        self.enemy_anim(&mut e.b, 11);
        self.enemy_set_speed(&mut e.b, 5);
    }

    /// 0x53F10: Morgue active combat loop.
    fn c40_fight(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.sub == 4 && e.b.act as i8 > 0x0E {
            e.b.act = 0;
        }
        if e.b.state() > 0x02020200 {
            e.m.sb(0xAE, 1);
        }

        // Arena clamp: camX - 0x50 .. camX + 0x1D0; when HP is 0 clamps tighter to camX - 0x10 .. camX + 0x190
        let cam_x = self.cam.s2.cam_x as i16;
        let lo = cam_x.wrapping_sub(0x50);
        let hi = cam_x.wrapping_add(0x1D0);
        if e.b.xi().wrapping_sub(lo) < 0 {
            e.b.set_xi_keep(lo);
        }
        if hi.wrapping_sub(e.b.xi()) < 0 {
            e.b.set_xi_keep(hi);
        }
        if e.b.hp == 0 {
            let dlo = cam_x.wrapping_sub(0x10);
            let dhi = cam_x.wrapping_add(0x190);
            if e.b.xi().wrapping_sub(dlo) < 0 {
                e.b.set_xi_keep(dlo);
            }
            if dhi.wrapping_sub(e.b.xi()) < 0 {
                e.b.set_xi_keep(dhi);
            }
        }

        match e.b.sub {
            0 => self.c40_alive(e, h, i),
            2 => {
                self.c40_hit_credit(e, h);
                self.enemy_hit_reaction(e, h);
            }
            4 => self.enemy_hit_reaction(e, h),
            6 => self.enemy_grabbed(e, h),
            _ => {}
        }
    }

    /// 0x53FDA: Morgue AI alive tick.
    fn c40_alive(&mut self, e: &mut Enemy, h: Handle, _i: usize) {
        // Poise recovery (0x53FDA)
        let p = e.m.b(0x79);
        if p != 0 {
            let p2 = p.wrapping_sub(1);
            e.m.sb(0x79, p2);
            if p2 == 0 {
                e.b.stun = 11;
            }
        }

        self.enemy_floor_collision(e);

        if e.b.hitstop != 0 {
            e.b.hitstop = e.b.hitstop.wrapping_sub(1);
            e.b.palette_override = 1;
            self.draw_list_add_check(h);
            self.enemy_push_hurtable(h);
            return;
        }

        let t7a = e.m.b(0x7A);
        if t7a != 0 {
            e.m.sb(0x7A, t7a.wrapping_sub(1));
            self.draw_list_add_check(h);
            self.enemy_push_hurtable(h);
            return;
        }

        self.boss_slot4_keep(e);
        if e.m.b(0xA1) == 0 {
            self.lib_face_target(e);
        }

        self.c40_act(e, h);

        self.c40_check_close_melee(e);
        self.c40_check_reinforcement(e);
        self.c40_check_roar(e);
        self.c40_check_phase_tick(e);

        self.draw_list_add_check(h);
        self.enemy_push_hurtable(h);
    }

    /// 0x54066: Morgue action dispatch.
    fn c40_act(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0x02 => self.c40_act_02(e),
            0x04 => self.c40_act_04(e, h),
            0x06 => self.c40_act_06(e, h),
            0x08 => self.c40_act_08(e, h),
            0x0A | 0x0C => self.c40_act_0a(e, h),
            0x0E => self.c40_act_0e(e, h),
            0x10 => self.c40_act_10(e, h),
            0x12 => self.c40_act_12(e, h),
            0x14 => self.c40_act_14(e, h),
            0x16 => self.c40_act_16(e, h),
            0x18 => self.c40_act_18(e, h),
            0x1A => self.c40_act_1a(e),
            0x1C => self.c40_act_1c(e),
            0x1E => self.c40_act_1e(e, h),
            0x20 => self.c40_act_20(e),
            0x22 => self.c40_act_22(e),
            0x24 => self.c40_act_24(e),
            0x26 => self.c40_act_26(e),
            0x28 => self.c40_act_28(e),
            0x2A => self.c40_act_2a(e),
            0x2C => self.c40_act_2c(e, h),
            0x2E => self.c40_act_2e(e),
            0x30 => self.c40_act_30(e),
            0x32 => self.c40_act_32(e, h),
            0x34 => self.c40_act_34(e, h),
            _ => {}
        }
    }

    /// 0x544CC: act 0x02.
    fn c40_act_02(&self, e: &mut Enemy) {
        let t = e.m.b(0x7A);
        if t != 0 {
            e.m.sb(0x7A, t.wrapping_sub(1));
        }
    }

    /// 0x544D8: act 0x04.
    fn c40_act_04(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(0xAC);
        if t == 0 {
            self.c40_decide(e, h);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            self.c40_decide(e, h);
            return;
        }
        self.lib_face_target(e);
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x544F6: act 0x06.
    fn c40_act_06(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(0xAC);
        if t == 0 {
            self.c40_decide(e, h);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            self.c40_decide(e, h);
            return;
        }
        let a8 = e.m.b(0xA8);
        if a8 == 0 || a8 == 1 {
            e.m.sb(0xA8, 6);
            e.m.sb(0xA9, e.b.dir);
            self.c40_turn(e);
        } else {
            e.m.sb(0xA8, a8.wrapping_sub(1));
        }
        self.lib_face_target(e);
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x5461A: act 0x08 approach.
    fn c40_act_08(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(0xAC);
        if t == 0 {
            if e.m.b(0xB9) != 0 {
                self.c40_action_headbutt(e);
            } else {
                self.lib_face_target(e);
                self.c40_decide(e, h);
            }
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            if e.m.b(0xB9) != 0 {
                self.c40_action_headbutt(e);
            } else {
                self.lib_face_target(e);
                self.c40_decide(e, h);
            }
            return;
        }
        if t_next & 3 == 0 {
            let dest_x = e.m.w(0x88) as i16;
            let vx = if dest_x > e.b.xi() { 0x15000i32 } else { -0x15000i32 };
            e.m.sl(0xB0, vx as u32);
            let dest_z = e.m.w(0x8A) as i16;
            let vz = if dest_z > e.b.zi() { 0x10000i32 } else { -0x10000i32 };
            e.m.sl(0xB4, vz as u32);
        }
        let dx = e.m.l(0xB0) as i32;
        e.b.set_xi_keep(e.b.xi().wrapping_add((dx >> 16) as i16));
        let dz = e.m.l(0xB4) as i32;
        e.b.set_zi_keep(e.b.zi().wrapping_add((dz >> 16) as i16));
        self.lib_face_target(e);
        self.enemy_step(&mut e.b);
    }

    /// 0x549EC: act 0x0A / 0x0C wait.
    fn c40_act_0a(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(0xAC);
        if t == 0 {
            self.c40_decide(e, h);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            self.c40_decide(e, h);
            return;
        }
        self.lib_face_target(e);
        self.enemy_step(&mut e.b);
    }

    /// 0x5489E: act 0x0E intro animation and palette cycle.
    fn c40_act_0e(&mut self, e: &mut Enemy, h: Handle) {
        if self.enemy_step(&mut e.b) {
            e.b.palette_override = 0x9F;
            self.clear_caption_box(0x90A29C);
            self.c40_decide(e, h);
            return;
        }
        let t = e.m.w(0xAC).wrapping_add(1);
        let mut d0 = if t >= 12 {
            e.m.sw(0xAC, 0);
            0
        } else {
            e.m.sw(0xAC, t);
            t
        };
        if d0 & 1 == 0 {
            d0 >>= 1;
        }
        e.b.palette_override = self.assets.enemy.t.u8(0x548EA, d0 as u32);
    }

    /// 0x54B54: act 0x10.
    fn c40_act_10(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(0xAC);
        if t == 0 {
            self.c40_decide(e, h);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            self.c40_decide(e, h);
            return;
        }
        let target = self.lib_target(e);
        if target.sub == 2 {
            self.enemy_step(&mut e.b);
            return;
        }
        let (d0, d1) = self.lib_distance_class(e);
        if d0 != d1 {
            self.enemy_anim(&mut e.b, 1);
        }
        if d0 & 1 != 0 {
            if (e.b.ax as i16) < 0 {
                e.b.ax = (e.b.ax as i16).wrapping_neg();
            }
            if (e.b.vx as i16) < 0 {
                e.b.vx = (e.b.vx as i16).wrapping_neg();
            }
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x5492A: act 0x12 headbutt.
    fn c40_act_12(&mut self, e: &mut Enemy, h: Handle) {
        if self.enemy_step(&mut e.b) {
            self.c40_decide(e, h);
        }
    }

    /// 0x54570: act 0x14 kite walk.
    fn c40_act_14(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(0xAC);
        if t == 0 {
            self.c40_decide(e, h);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            self.c40_decide(e, h);
            return;
        }
        let a8 = e.m.b(0xA8);
        if a8 == 0 || a8 == 1 {
            e.m.sb(0xA8, 0x1E);
            self.c40_update_grid_dir(e);
        } else {
            e.m.sb(0xA8, a8.wrapping_sub(1));
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x54936: act 0x16 pause with face_target.
    fn c40_act_16(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(0xAC);
        if t == 0 {
            self.c40_decide(e, h);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            self.c40_decide(e, h);
            return;
        }
        self.lib_face_target(e);
        self.enemy_step(&mut e.b);
    }

    /// 0x54534: act 0x18 depth drift.
    fn c40_act_18(&mut self, e: &mut Enemy, h: Handle) {
        let a8 = e.m.b(0xA8);
        if a8 == 0 || a8 == 1 {
            e.m.sb(0xA8, 8);
            if !Self::enemy_stuck_check(e) {
                self.c40_decide(e, h);
                return;
            }
            let t = e.m.w(0xAC);
            if t == 0 {
                let vz = (e.m.l(0xB4) as i32).wrapping_neg();
                e.m.sl(0xB4, vz as u32);
            } else {
                e.m.sw(0xAC, t.wrapping_sub(1));
            }
        } else {
            e.m.sb(0xA8, a8.wrapping_sub(1));
        }
        let dz = e.m.l(0xB4) as i32;
        e.b.set_zi_keep(e.b.zi().wrapping_add((dz >> 16) as i16));
        self.lib_face_target(e);
        self.enemy_step(&mut e.b);
    }

    /// 0x54C0A: act 0x1A.
    fn c40_act_1a(&mut self, e: &mut Enemy) {
        let t = e.m.w(0xAC);
        if t == 0 {
            self.c40_start_run_in(e);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            self.c40_start_run_in(e);
            return;
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x54A04: act 0x1C run in anim.
    fn c40_act_1c(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            // 0x54D3E: transition to shoulder rush (act 0x2C)
            self.enemy_anim(&mut e.b, 0x1A);
            e.m.sb(0xA1, 1);
            e.b.act = 0x2C;
            e.m.sw(0xAC, 0x30);
            e.b.vy = 0;
            e.b.vz = 0;
            e.b.ay = 0;
            e.b.az = 0;
            let vx = if e.b.facing == 0 { -0x900 } else { 0x900 };
            e.b.vx = vx;
            e.b.ax = if e.b.facing == 0 { 0x30 } else { -0x30 };
            self.enemy_begin_attack(&mut e.b, 0x42);
        }
    }

    /// 0x54A10: act 0x1E shoulder rush brake.
    fn c40_act_1e(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(0xAC);
        if t == 0 {
            self.c40_decide(e, h);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            self.c40_decide(e, h);
            return;
        }
        if t_next < 0x10 {
            self.enemy_begin_attack(&mut e.b, 0x42);
        }
        if t_next < 5 {
            self.enemy_anim(&mut e.b, 0);
        } else if t_next <= 0x0E {
            self.enemy_anim(&mut e.b, 0x12);
        } else {
            e.b.motion();
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x54B24: act 0x20 charge.
    fn c40_act_20(&mut self, e: &mut Enemy) {
        let t = e.m.w(0xAC);
        if t == 0 {
            self.c40_charge_to_brake(e);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            self.c40_charge_to_brake(e);
            return;
        }
        let target = self.lib_target(e);
        let d = target.xi().wrapping_add(1).wrapping_sub(e.b.xi()) as u16;
        if d <= 2 {
            self.c40_charge_to_brake(e);
            return;
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x549BE: act 0x22 headbutt delay.
    fn c40_act_22(&mut self, e: &mut Enemy) {
        let t = e.m.w(0xAC);
        if t == 0 {
            self.c40_start_headbutt(e);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            self.c40_start_headbutt(e);
        }
    }

    /// 0x54AE4: act 0x24 shoulder rush variant.
    fn c40_act_24(&mut self, e: &mut Enemy) {
        let t = e.m.w(0xAC);
        if t == 0 {
            e.m.sb(0xAA, 0);
            self.enemy_anim(&mut e.b, 0x0D);
            e.m.sb(0xA1, 1);
            e.b.act = 0x1E;
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            e.m.sb(0xAA, 0);
            self.enemy_anim(&mut e.b, 0x0D);
            e.m.sb(0xA1, 1);
            e.b.act = 0x1E;
            return;
        }
        if t_next < 0x10 {
            self.enemy_begin_attack(&mut e.b, 0x42);
        }
        e.b.motion();
        self.enemy_step(&mut e.b);
    }

    /// 0x5494A: act 0x26 roar (40 frames, 5 sparks) then charge.
    fn c40_act_26(&mut self, e: &mut Enemy) {
        let t = e.m.w(0xAC);
        if t == 0 {
            self.c40_start_charge(e);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            self.c40_start_charge(e);
            return;
        }
        if t_next & 7 == 0 {
            // Spawn spark (pool A class 75 var 0x13)
            if let Some(j) = self.enemies.alloc() {
                let s = &mut self.enemies.slots[j];
                s.b.live = 1;
                s.b.k20 = 75 * 4;
                s.b.var26 = 0x13;
                s.b.facing = e.b.facing;
                let dx = (0x28 - (t_next as i16)) as i16;
                s.b.set_xi_keep(e.b.xi().wrapping_add(if e.b.facing == 0 { dx } else { dx.wrapping_neg() }));
                s.b.set_yi_keep(e.b.yi().wrapping_sub(0x30));
                s.b.set_zi_keep(e.b.zi().wrapping_sub(1));
            }
        }
    }

    /// 0x54706: act 0x28.
    fn c40_act_28(&mut self, e: &mut Enemy) {
        let t = e.m.w(0xAC);
        if t == 0 {
            self.c40_start_act_2a(e);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            self.c40_start_act_2a(e);
            return;
        }
        let a8 = e.m.b(0xA8);
        if a8 == 0 || a8 == 1 {
            e.m.sb(0xA8, 12);
            e.m.sb(0xA9, e.b.dir);
            self.c40_turn(e);
        } else {
            e.m.sb(0xA8, a8.wrapping_sub(1));
        }
        e.b.facing = if e.b.dir <= 0x10 { 1 } else { 0 };
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x54750: act 0x2A.
    fn c40_act_2a(&mut self, e: &mut Enemy) {
        let t = e.m.w(0xAC);
        if t == 0 {
            self.c40_start_act_28(e);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            self.c40_start_act_28(e);
            return;
        }
        let target = self.lib_target(e);
        let dx = if target.xi() > e.b.xi() { 0x24000i32 } else { -0x24000i32 };
        e.b.set_xi_keep(e.b.xi().wrapping_add((dx >> 16) as i16));
        let dz = if target.zi() > e.b.zi() { 0x18000i32 } else { -0x18000i32 };
        e.b.set_zi_keep(e.b.zi().wrapping_add((dz >> 16) as i16));
        self.lib_face_target(e);
        self.enemy_step(&mut e.b);
    }

    /// 0x54A7A: act 0x2C shoulder rush from run-in.
    fn c40_act_2c(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(0xAC);
        if t == 0 {
            e.m.sb(0xAA, 0);
            self.c40_decide(e, h);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            e.m.sb(0xAA, 0);
            self.c40_decide(e, h);
            return;
        }
        if t_next < 0x10 {
            self.enemy_begin_attack(&mut e.b, 0x42);
        }
        if t_next < 5 {
            self.enemy_anim(&mut e.b, 0);
        } else if t_next <= 0x0E {
            self.enemy_anim(&mut e.b, 0x1A);
        } else {
            e.b.motion();
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x549C8: act 0x2E slap delay.
    fn c40_act_2e(&mut self, e: &mut Enemy) {
        let t = e.m.w(0xAC);
        if t == 0 {
            self.c40_start_slap(e);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            self.c40_start_slap(e);
        }
    }

    /// 0x549D2: act 0x30 slap attack.
    fn c40_act_30(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            // 0x54C1E: slap delay
            e.m.sb(0xA1, 1);
            e.b.act = 0x32;
            e.m.sw(0xAC, 1);
        }
    }

    /// 0x549DE: act 0x32 slap combo follow up.
    fn c40_act_32(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(0xAC);
        if t == 0 {
            self.c40_slap_follow_up(e, h);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            self.c40_slap_follow_up(e, h);
            return;
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x552A2: slap combo decision.
    fn c40_slap_follow_up(&mut self, e: &mut Enemy, h: Handle) {
        let c0 = e.m.w(0xC0);
        if c0 == 0 {
            self.c40_decide(e, h);
            return;
        }
        let c0_next = c0.wrapping_sub(1);
        e.m.sw(0xC0, c0_next);
        if c0_next == 0 {
            self.c40_decide(e, h);
            return;
        }
        self.lib_face_target(e);
        self.enemy_begin_attack(&mut e.b, 0x4B);
        if c0_next == 1 {
            // 0x552EA: combo continuation
            e.m.sb(0xA1, 1);
            e.b.act = 0x30;
            e.m.sw(0xAC, 0x3C);
            e.m.sb(0xB9, 0);
            let c2 = e.m.w(0xC2) as usize;
            let off = ((c2 << 2) + c0_next as usize) * 2;
            let anim = self.assets.enemy.t.u16(0x55326, off as u32);
            if anim != 0x5A {
                self.enemy_anim(&mut e.b, anim);
            }
        }
    }

    /// 0x548F0: act 0x34.
    fn c40_act_34(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(0xAC);
        if t == 0 {
            e.b.palette_override = 0x9F;
            self.clear_caption_box(0x90A29C);
            self.c40_decide(e, h);
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xAC, t_next);
        if t_next == 0 {
            e.b.palette_override = 0x9F;
            self.clear_caption_box(0x90A29C);
            self.c40_decide(e, h);
        }
    }

    /// 0x555C0: start charge attack (0x41) at speed 0x24 (7 px per frame).
    fn c40_start_charge(&mut self, e: &mut Enemy) {
        self.lib_face_target(e);
        self.enemy_set_speed(&mut e.b, 0x24);
        self.enemy_begin_attack(&mut e.b, 0x41);
        e.m.sb(0xA1, 1);
        e.b.act = 0x20;
        e.m.sw(0xAC, 0x1E); // 30 frames
        self.enemy_anim(&mut e.b, 7);
    }

    /// 0x54DA0: charge arrival brake / shoulder rush transition.
    fn c40_charge_to_brake(&mut self, e: &mut Enemy) {
        self.enemy_anim(&mut e.b, 0x0D);
        e.m.sb(0xA1, 1);
        e.b.act = 0x1E;
        e.m.sw(0xAC, 0x30);
        e.b.vx = 0;
        e.b.vy = 0;
        e.b.vz = 0;
        e.b.ax = if e.b.facing == 0 { 0x20 } else { -0x20 };
        e.b.ay = 0;
        e.b.az = 0;
    }

    /// 0x5521C: start headbutt attack (0x40).
    fn c40_start_headbutt(&mut self, e: &mut Enemy) {
        self.enemy_begin_attack(&mut e.b, 0x40);
        e.m.sb(0xA1, 1);
        e.b.act = 0x12;
        e.m.sw(0xAC, 0x3C);
        e.m.sb(0xB9, 0);
        self.enemy_anim(&mut e.b, 4);
    }

    /// 0x5525A: start slap attack (0x6C).
    fn c40_start_slap(&mut self, e: &mut Enemy) {
        self.enemy_begin_attack(&mut e.b, 0x6C);
        e.m.sb(0xA1, 1);
        e.b.act = 0x30;
        e.m.sw(0xAC, 0x3C);
        e.m.sb(0xB9, 0);
        let anim = if e.m.w(0xC2) != 0 { 0x1B } else { 4 };
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x55360: action 0 and 1 headbutt setup.
    fn c40_action_headbutt(&mut self, e: &mut Enemy) {
        self.lib_face_target(e);
        self.enemy_begin_attack(&mut e.b, 0x40);
        e.m.sb(0xA1, 1);
        e.b.act = 0x22;
        let r = (self.rand() & 0x1E) as u32;
        e.m.sw(0xAC, self.assets.enemy.t.u16(0x553B6, r));
        if e.m.b(0xB9) != 0 {
            self.c40_start_headbutt(e);
            return;
        }
        e.m.sb(0xB9, 0);
        self.enemy_anim(&mut e.b, 4);
    }

    /// 0x55600: action 2.
    fn c40_action_2(&mut self, e: &mut Enemy) {
        if e.m.b(0xB9) == 0 {
            e.m.sb(0xB9, 1);
            self.enemy_anim(&mut e.b, 5);
        }
        self.c40_aggressive_decide(e);
    }

    /// 0x55550: start swing (attack 0x2B, anim 8).
    fn c40_start_swing(&mut self, e: &mut Enemy) {
        self.enemy_begin_attack(&mut e.b, 0x2B);
        e.m.sb(0xA1, 1);
        e.b.act = 0x12;
        e.m.sw(0xAC, 0x3C);
        self.enemy_anim(&mut e.b, 8);
    }

    /// 0x55588: start flurry (attack 0x31, anim 12).
    fn c40_start_flurry(&mut self, e: &mut Enemy) {
        self.enemy_begin_attack(&mut e.b, 0x31);
        e.m.sb(0xA1, 1);
        e.b.act = 0x12;
        e.m.sw(0xAC, 0x3C);
        self.enemy_anim(&mut e.b, 12);
    }

    /// 0x54C6A: start run in (act 0x1C, anim 0x0F, sound 0x60).
    fn c40_start_run_in(&mut self, e: &mut Enemy) {
        e.m.sb(0xA1, 1);
        e.b.act = 0x1C;
        e.m.sw(0xAC, 0x1E);
        e.m.sb(0xA9, e.b.dir);
        self.c40_turn(e);
        e.b.facing = if e.b.dir <= 0x10 { 1 } else { 0 };
        self.lib_face_target(e);
        self.sound_ex(0x60, 0x71010, 0x8000000);
        self.enemy_anim(&mut e.b, 0x0F);
    }

    /// 0x5511A: roar start check and setup (40 frames, anim 0x12).
    fn c40_start_roar(&mut self, e: &mut Enemy) {
        let target = self.lib_target(e);
        let dist = (target.xi().wrapping_sub(e.b.xi()) as i16).abs();
        if dist < 0x80 {
            self.c40_decide(e, 0);
            return;
        }
        e.m.sb(0xA1, 1);
        e.m.sw(0xAC, 0x28); // 40 frames
        e.b.act = 0x26;
        self.enemy_anim(&mut e.b, 0x12);
        self.lib_face_target(e);
        self.enemy_set_speed(&mut e.b, 5);
    }

    /// 0x553E0 / 0x5546C / 0x55462 / 0x553D6: action slap (attacks 0x6C or 0x40).
    fn c40_action_slap(&mut self, e: &mut Enemy, variant: u16) {
        e.m.sw(0xC2, variant);
        self.lib_face_target(e);
        let (atk, table, anim) = if variant == 1 || variant == 2 {
            (0x40, 0x554CE, 0x1B)
        } else {
            (0x6C, 0x55442, 4)
        };
        self.enemy_begin_attack(&mut e.b, atk);
        e.m.sb(0xA1, 1);
        e.b.act = 0x2E;
        e.m.sw(0xC0, 3);
        let r = (self.rand() & 0x1E) as u32;
        e.m.sw(0xAC, self.assets.enemy.t.u16(table, r));
        if e.m.b(0xB9) != 0 {
            self.c40_start_headbutt(e);
            return;
        }
        e.m.sb(0xB9, 0);
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x5561A: start flank move (act 0x28).
    fn c40_start_act_28(&mut self, e: &mut Enemy) {
        if !self.lib_off_screen(&e.b) {
            self.c40_decide(e, 0);
            return;
        }
        e.b.act = 0x28;
        e.m.sw(0xAC, 100);
        e.m.sb(0xA8, 0x1E);
        e.m.sb(0xA9, e.b.dir);
        self.c40_turn(e);
        let d0 = (e.b.dir.wrapping_add(8) & 0x10) as u16;
        if d0 < 0x10 {
            e.b.facing = 1;
            e.b.dir = 0;
        } else {
            e.b.facing = 0;
            e.b.dir = 0x10;
        }
        if (self.rand() & 0x0F) > 0x0C {
            e.b.dir = e.b.dir.wrapping_add(0x10) & 0x10;
        }
        self.enemy_set_speed(&mut e.b, 0x0B);
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x556A0: start diagonal move (act 0x2A).
    fn c40_start_act_2a(&mut self, e: &mut Enemy) {
        if !self.lib_off_screen(&e.b) {
            self.c40_decide(e, 0);
            return;
        }
        e.m.sb(0xA1, 0);
        e.m.sw(0xAC, 100);
        e.b.act = 0x2A;
        let dest_x = e.m.w(0x88) as i16;
        let vx = if dest_x > e.b.xi() { 0x24000i32 } else { -0x24000i32 };
        e.m.sl(0xB0, vx as u32);
        let dest_z = e.m.w(0x8A) as i16;
        let vz = if dest_z > e.b.zi() { 0x18000i32 } else { -0x18000i32 };
        e.m.sl(0xB4, vz as u32);
    }

    /// 0x554EE: wake-up counterattack / shoulder rush.
    fn c40_start_wake_shoulder(&mut self, e: &mut Enemy) {
        let _ = self.rand() & 0x7F;
        self.lib_face_target(e);
        self.enemy_begin_attack(&mut e.b, 0x40);
        e.m.sb(0xA1, 1);
        e.m.sw(0xAC, 1);
        e.m.sb(0xB9, 0);
        e.b.act = 0x12;
        self.enemy_anim(&mut e.b, 0x19);
    }

    /// 0x543AC: Morgue recover entry for table 0x2B56C.
    pub(super) fn c40_recover(&mut self, e: &mut Enemy, h: Handle) {
        match self.boss_recover(e, BossAi::Morgue) {
            BossRecover::Shoulder => self.c40_start_wake_shoulder(e),
            BossRecover::Decide => self.c40_decide(e, h),
            _ => {}
        }
    }

    /// 0x55704: Morgue action decision.
    fn c40_decide(&mut self, e: &mut Enemy, _h: Handle) {
        if e.b.act == 0x0A {
            return;
        }
        e.m.sw(0xAC, 60);
        self.enemy_set_speed(&mut e.b, 5);
        e.m.sb(0xA1, 0);
        let (cls, _) = self.lib_distance_class(e);
        e.m.sb(0xA5, cls as u8);
        if e.m.b(0xA0) == 0 {
            self.c40_defensive_decide(e);
        } else {
            self.c40_aggressive_decide(e);
        }
    }

    /// 0x54F28: defensive decision (kite).
    fn c40_defensive_decide(&mut self, e: &mut Enemy) {
        e.m.sb(0xA9, e.b.dir);
        self.lib_face_target(e);
        self.enemy_set_speed(&mut e.b, 6);
        e.m.sb(0xA1, 0);
        e.m.sb(0xA8, 6);
        let r = (self.rand() & 0x1E) as u32;
        e.m.sw(0xAC, self.assets.enemy.t.u16(0x54FEE, r));
        e.b.act = 0x14;
        self.c40_update_grid_dir(e);
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x55034: aggressive decision (approach).
    fn c40_aggressive_decide(&mut self, e: &mut Enemy) {
        self.enemy_anim(&mut e.b, 1);
        e.m.sb(0xA1, 0);
        e.m.sw(0xAC, 60);
        e.b.act = 8;
        let dest_x = e.m.w(0x88) as i16;
        let vx = if dest_x > e.b.xi() { 0x15000i32 } else { -0x15000i32 };
        e.m.sl(0xB0, vx as u32);
        let dest_z = e.m.w(0x8A) as i16;
        let vz = if dest_z > e.b.zi() { 0x10000i32 } else { -0x10000i32 };
        e.m.sl(0xB4, vz as u32);
    }

    /// 0x55748: check and dispatch aggressive action.
    fn c40_aggression_gate(&mut self, e: &mut Enemy) {
        if !self.boss_aggression_gate(e, BossAi::Morgue) {
            return;
        }
        self.lib_face_target(e);
        self.enemy_set_speed(&mut e.b, 5);
        e.m.sb(0xA1, 0);
        let (cls, _) = self.lib_distance_class(e);
        e.m.sb(0xA5, cls as u8);
        let act = self.boss_action_pick(e, BossAi::Morgue);
        if e.m.b(0xB9) != 0 && act != 1 {
            return;
        }
        self.c40_dispatch_action(e, act);
    }

    /// 0x557D0: dispatch picked action.
    fn c40_dispatch_action(&mut self, e: &mut Enemy, act: u8) {
        match act {
            0 | 1 => self.c40_action_headbutt(e),
            2 => self.c40_action_2(e),
            3 => self.c40_start_swing(e),
            4 => self.c40_start_flurry(e),
            5 => self.c40_start_run_in(e),
            6 => self.c40_start_roar(e),
            7 => self.c40_action_slap(e, 0),
            8 => self.c40_action_slap(e, 1),
            9 => self.c40_action_slap(e, 2),
            10 => self.c40_action_slap(e, 3),
            _ => {}
        }
    }

    /// 0x54F70..0x54FDC: update direction from kite grid.
    fn c40_update_grid_dir(&mut self, e: &mut Enemy) {
        e.m.sb(0xA9, e.b.dir);
        let t = self.lib_target(e);
        let a = morgan_cell(t.xi(), t.zi());
        let b = morgan_cell(e.b.xi(), e.b.zi());
        let d1 = (a.saturating_sub(1)) as usize;
        let row = self.assets.enemy.t.u16(0x5500E, (d1 * 2) as u32) as usize;
        let d3 = row + (b.saturating_sub(1)) as usize;
        let mut d = self.assets.enemy.t.u8(0x53034, d3 as u32);
        // 0x54800: tie break
        if (d as i8) > 7 {
            let r = (self.rand() & 15) as u32;
            let off = (((d.wrapping_sub(8)) << 4) as u32).wrapping_add(r);
            d = self.assets.enemy.t.u8(0x5481E, off);
        }
        e.m.sb(0xBE, d.wrapping_mul(4) & 31);
        e.b.dir = e.m.b(0xBE);
        if e.b.xi().wrapping_sub(self.cam.s2.cam_x as i16) < 0 {
            e.b.dir = 8;
        }
        if (self.cam.s2.cam_x as i16).wrapping_add(384).wrapping_sub(e.b.xi()) < 0 {
            e.b.dir = 24;
        }
    }

    /// 0x55090: turn direction by at most 2 units.
    fn c40_turn(&self, e: &mut Enemy) {
        let d = (e.b.dir.wrapping_sub(e.m.b(0xA9)) & 31).wrapping_add(2) & 31;
        if (d as i8) > 4 {
            let delta = if (d.wrapping_sub(2) as i8) > 16 {
                -2i8
            } else {
                2
            };
            e.b.dir = e.m.b(0xA9).wrapping_add(delta as u8) & 31;
        }
    }

    /// 0x5426E: check close melee trigger.
    fn c40_check_close_melee(&mut self, e: &mut Enemy) {
        if e.m.b(0xA1) != 0 || e.m.b(0xA0) == 0 {
            return;
        }
        let target = self.lib_target(e);
        let dz = (target.zi().wrapping_add(7).wrapping_sub(e.b.zi()) as u16) <= 10;
        if dz {
            self.c40_aggression_gate(e);
        }
    }

    /// 0x542B0: request reinforcements when ac reaches 0x10.
    fn c40_check_reinforcement(&mut self, e: &mut Enemy) {
        if e.m.b(0xA1) == 0 && e.m.b(0xA0) != 0 && e.m.w(0xAC) == 0x10 {
            self.spawn.g.b502 = 1;
        }
    }

    /// 0x542D2: check roar trigger (HP limit from 0x5432C, dist > 0x100).
    fn c40_check_roar(&mut self, e: &mut Enemy) {
        if e.m.b(0xA0) == 0 || e.m.b(0xA1) != 0 || e.m.b(0xA2) != 0 {
            return;
        }
        let quarter = self.w.morgue_init_hp >> 2;
        let limit_idx = if (e.b.hp as i16) >= (quarter as i16) { 2 } else { 0 };
        let limit = self.assets.enemy.t.u16(0x5432C, limit_idx);
        if (self.rand() & 0x1F) <= limit {
            let target = self.lib_target(e);
            let dist = (target.xi().wrapping_add(0x100).wrapping_sub(e.b.xi())) as u16;
            if dist > 0x200 {
                self.c40_start_roar(e);
            }
        }
    }

    /// 0x540FA: phase countdown and flip.
    fn c40_check_phase_tick(&mut self, e: &mut Enemy) {
        let t = e.m.w(0xA6);
        if t == 0 {
            return;
        }
        let t_next = t.wrapping_sub(1);
        e.m.sw(0xA6, t_next);
        if t_next != 0 {
            return;
        }
        if e.b.act == 0x28 || e.b.act == 0x2A || self.w.morgue_7054 != 0 {
            if self.lib_off_screen(&e.b) {
                let dx = (e.m.w(0x88) as i16).wrapping_sub(e.b.xi()).wrapping_neg();
                e.m.sw(0x88, e.b.xi().wrapping_add(dx) as u16);
                e.m.sw(0xA6, 0x3C);
                self.c40_start_act_28(e);
                return;
            }
        }
        let r = (self.rand() & 0x1E) as u32;
        let phase = e.m.b(0xA0);
        if phase != 0 {
            e.m.sw(0xA6, self.assets.enemy.t.u16(0x541AA, r));
            e.m.sb(0xA0, 0);
        } else {
            e.m.sw(0xA6, self.assets.enemy.t.u16(0x5418A, r));
            e.m.sb(0xA0, 1);
        }
        self.boss_pick_target(e);
        self.lib_slot6_new(e);
    }

    /// 0x557F4: Morgue damage credit and defeat trigger.
    fn c40_hit_credit(&mut self, e: &mut Enemy, h: Handle) {
        if (e.b.who as i8) >= 0 {
            let damage = e.b.hp_prev.wrapping_sub(e.b.hp);
            if self.spawn.g.car_mode != 0 {
                for p in &mut self.players {
                    p.w10c = p.w10c.wrapping_add(damage);
                }
            } else if let Some(p) = self.players.get_mut(e.b.who as usize) {
                p.w10c = p.w10c.wrapping_add(damage);
            }
            if e.b.hp != 0 {
                e.b.hp_prev = e.b.hp;
            }
        }
        if e.b.hp == 0 && e.b.hp_prev != 0 {
            self.flow_boss_defeated(h); // 0xACEE
            e.b.hp_prev = 0;
        }
    }

    /// 0x55816: Morgue dying timeline (score 30,000, 120 frames, mode 6).
    fn c40_dying(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => {
                // 0x55826
                self.boss_dying_hooks(e, true, true, true);
                e.m.sw(0x80, 0x78); // 120 frames
                e.b.sub = 2;
                self.draw_list_add_check(h);
            }
            2 => {
                // 0x55842
                let t = e.m.w(0x80);
                if t != 0 {
                    e.m.sw(0x80, t.wrapping_sub(1));
                    self.draw_list_add_check(h);
                } else {
                    e.b.mode = 6;
                    self.draw_list_add_check(h);
                }
            }
            _ => self.draw_list_add_check(h),
        }
    }
}
