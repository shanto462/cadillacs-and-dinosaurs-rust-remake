//! Morgan, handler 0x51692. Shared boss services retain the original RNG order.
use super::{
    Enemy,
    boss_common::{BossAi, BossRecover},
};
use crate::game::{Game, Handle, Slot, slot_of};

impl Game {
    /// 0x51692: init, fight, intermediate death, free.
    pub(super) fn enemy_class36(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => self.enemy_morgan_init(e, h, i),
            2 => {
                if e.b.sub == 4 && e.b.act as i8 > 14 {
                    e.b.act = 0;
                }
                match e.b.sub {
                    0 => self.enemy_morgan_alive(e, h),
                    2 => {
                        self.enemy_morgan_hit_credit(e);
                        self.enemy_hit_reaction(e, h);
                    }
                    4 => self.enemy_hit_reaction(e, h),
                    6 => self.enemy_grabbed(e, h),
                    _ => {}
                }
                if e.b.state() > 0x02020200 {
                    e.m.sb(0xC0, 1);
                }
                self.enemy_morgan_rifle_pose(e);
                if e.b.hp == 0 {
                    let lo = (self.cam.s2.cam_x as i16).wrapping_sub(16);
                    let hi = (self.cam.s2.cam_x as i16).wrapping_add(400);
                    if e.b.xi().wrapping_sub(lo) < 0 {
                        e.b.set_xi_keep(lo);
                    }
                    if hi.wrapping_sub(e.b.xi()) < 0 {
                        e.b.set_xi_keep(hi);
                    }
                }
                self.enemy_clamp_to_playfield(e, i);
            }
            4 => {
                // M7-19 owns the full death dialogue. Retain the approved placeholder rebirth.
                let spec = crate::game::placeholder::Boss {
                    hp_row: 4,
                    controller: true,
                    score: false,
                    rebirth: Some(40),
                    group: false,
                };
                self.boss_placeholder_dying(e, h, spec);
            }
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x52D7A / 0x3262A: damage credit, then intermediate-form defeat flags.
    pub(in crate::game) fn enemy_morgan_hit_credit(&mut self, e: &mut Enemy) {
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
        if e.b.hp == 0 {
            self.boss_intermediate_hit(BossAi::Morgan);
            if e.b.hp_prev != 0 {
                e.b.hp_prev = 0;
            }
        }
    }

    /// 0x516A8..0x5172C: sort the two handles using the original signed CMPA.W.
    fn enemy_morgan_init(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.sub == 0 {
            if self.enemies.free.len() < 2 {
                self.enemy_free_pool_a(e, i);
                return;
            }
            let Some(j) = self.enemies.alloc() else {
                return;
            };
            let other = self.enemies.handle(j);
            let (rifle, boss) = if (other as i16) < (h as i16) {
                (other, h)
            } else {
                (h, other)
            };
            let mut b = if boss == h {
                e.clone()
            } else {
                std::mem::take(&mut self.enemies.slots[j])
            };
            b.b.live = 1;
            b.b.k20 = 36 * 4;
            b.b.var26 = e.b.var26;
            b.m.sb(0x96, e.m.b(0x96));
            b.b.anim_table = 0x70F02;
            b.b.dir = e.b.dir;
            b.b.palette_override = e.b.palette_override;
            b.b.set_xi_keep(e.b.xi());
            b.b.set_yi_keep(e.b.yi());
            b.b.set_zi_keep(e.b.zi());
            b.m.sw(0xAE, rifle);
            b.b.sub = 2;
            let mut r = if rifle == h {
                e.clone()
            } else {
                std::mem::take(&mut self.enemies.slots[j])
            };
            r.b.live = 1;
            r.b.k20 = 67 * 4;
            r.b.var26 = 0;
            r.b.anim_table = 0x71758;
            r.m.sl(0xA0, boss as i16 as i32 as u32);
            if boss == h {
                *e = b;
                self.enemies.slots[j] = r;
            } else {
                *e = r;
                self.enemies.slots[j] = b;
            }
            return;
        }
        e.b.mode = 2;
        e.m.sb(0x87, 0);
        e.b.id = 0x15;
        e.m.sw(0x84, 0x54);
        e.b.anim_table = 0x70F02;
        self.enemy_anim(&mut e.b, 1);
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_set_speed(&mut e.b, 7);
        e.b.dir = 24;
        self.enemy_set_boss_hp(&mut e.b, 4);
        self.boss_pick_target(e);
        e.b.stun = 10;
        e.b.reaction = 0;
        e.b.b72 = 30;
        for k in [0x79, 0x7A, 0xBC, 0xB9] {
            e.m.sb(k, 0);
        }
        e.m.sb(0x7D, 0xFE);
        e.m.sw(0xAC, 20);
        e.m.sb(0xAB, 48);
        e.m.sw(0xA6, 1);
        e.m.sb(0xA0, 1);
        self.hud.boss = h;
        self.spawn.g.b4fc = 0;
        self.enemy_snap_to_floor(e);
        self.sound_ex(14, 0xFFFF0000, 0xFFFF0000);
        e.b.b63 = 2;
        self.lib_slot6_new(e);
        self.boss_spawn_controller(h);
        self.enemy_morgan_recover(e);
    }

    /// 0x51ACA: shared recovery with Morgan's jab and escape-hop continuations.
    pub(super) fn enemy_morgan_recover(&mut self, e: &mut Enemy) {
        match self.boss_recover(e, BossAi::Morgan) {
            BossRecover::WakeAttack => {
                self.lib_face_target(e);
                self.enemy_morgan_attack(e, 14, 13, 0x3F);
                e.m.sb(0xB9, 0);
            }
            BossRecover::OffscreenHop => self.enemy_morgan_hop(e, 0x28, 17),
            _ => self.enemy_morgan_kite(e),
        }
    }

    /// 0x51890: terrain, hitstop/wait, slot update, action, gate, phase, target, draw and hit list.
    fn enemy_morgan_alive(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.b(0x79) != 0 {
            let n = e.m.b(0x79) - 1;
            e.m.sb(0x79, n);
            if n == 0 {
                e.b.stun = 10;
            }
        }
        self.enemy_floor_collision(e);
        if e.b.hitstop != 0 {
            e.b.hitstop -= 1;
            e.b.timer = 1;
        } else if e.m.b(0x7A) != 0 {
            e.m.sb(0x7A, e.m.b(0x7A) - 1);
        } else {
            self.lib_slot6_keep(e);
            self.enemy_morgan_action(e, h);
            if e.m.b(0xA1) == 0 && e.m.b(0xA0) != 0 && e.m.w(0xAC) == 16 {
                self.spawn.g.b502 = 1;
                self.enemy_morgan_gate(e);
            }
            self.boss_phase_tick(e, BossAi::Morgan);
        }
        self.lib_check_target(e);
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x51B96..0x521DA: every live-state continuation.
    fn enemy_morgan_action(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            2 => {
                if e.m.b(0x7A) != 0 {
                    e.m.sb(0x7A, e.m.b(0x7A) - 1);
                }
            }
            4 | 6 | 16 => {
                if morgan_count(e) {
                    self.enemy_morgan_decide(e);
                    return;
                }
                if e.b.act != 4 {
                    let n = e.m.b(0xA8).wrapping_sub(1);
                    e.m.sb(0xA8, n);
                    if n == 0 {
                        e.m.sb(0xA8, if e.b.act == 16 { 30 } else { 6 });
                        e.m.sb(0xA9, e.b.dir);
                        if e.b.act == 16 {
                            self.enemy_morgan_grid(e, true);
                        } else {
                            self.lib_aim_at_target(e);
                            self.enemy_morgan_turn(e);
                        }
                    }
                }
                self.lib_face_target(e);
                self.enemy_move_dir(&mut e.b);
                self.enemy_step(&mut e.b);
            }
            8 | 38 => self.enemy_morgan_point(e),
            10 | 12 | 18 => {
                if morgan_count(e) {
                    self.enemy_morgan_decide(e);
                } else {
                    self.enemy_step(&mut e.b);
                }
            }
            14 => {
                if self.enemy_step(&mut e.b) {
                    self.enemy_morgan_decide(e);
                }
            }
            20 => {
                let n = e.m.b(0xA8).wrapping_sub(1);
                e.m.sb(0xA8, n);
                if n == 0 {
                    e.m.sb(0xA8, 8);
                    if !Self::enemy_stuck_check(e) {
                        self.enemy_morgan_decide(e);
                        return;
                    }
                    if morgan_count(e) {
                        e.m.sl(0xB4, (e.m.l(0xB4) as i32).wrapping_neg() as u32);
                    }
                }
                e.b.z = e.b.z.wrapping_add(e.m.l(0xB4) as i32);
                self.lib_face_target(e);
                self.enemy_step(&mut e.b);
            }
            22 => {
                if morgan_count(e) {
                    self.enemy_morgan_wait(e);
                } else {
                    self.enemy_move_dir(&mut e.b);
                    self.enemy_step(&mut e.b);
                }
            }
            24 => {
                if morgan_count(e) {
                    self.enemy_morgan_attack(e, 14, 6, 0x3F);
                    e.m.sb(0xB9, 0);
                    self.lib_aim_at_target(e);
                }
            }
            26 | 28 => {
                e.b.motion();
                if e.b.airborne == 0 {
                    self.enemy_morgan_decide(e);
                } else {
                    self.enemy_step(&mut e.b);
                }
            }
            30 => {
                if self.enemy_step(&mut e.b) {
                    self.enemy_morgan_spawn_grenade(e, h);
                }
            }
            32 => {
                if self.enemy_step(&mut e.b) {
                    self.enemy_morgan_gun_sound(e);
                    self.enemy_spawn_bullet(e, 0, 16, 36);
                    e.m.sb(0xAB, e.m.b(0xAB).wrapping_sub(1));
                    let n = e.m.b(0xA4).wrapping_sub(1);
                    e.m.sb(0xA4, n);
                    if n == 0 {
                        self.enemy_morgan_decide(e);
                    } else {
                        self.enemy_morgan_burst(e, false);
                    }
                }
            }
            34 => {
                if morgan_count(e) {
                    self.enemy_morgan_attack(e, 36, 6, 0x3F);
                } else {
                    self.enemy_step(&mut e.b);
                }
            }
            36 => {
                if self.enemy_step(&mut e.b) {
                    let n = e.m.b(0xBC).wrapping_sub(1);
                    e.m.sb(0xBC, n);
                    if n == 0 {
                        self.enemy_morgan_decide(e);
                    } else {
                        self.enemy_morgan_throw(e, h, false);
                    }
                }
            }
            40 | 42 => {
                if e.m.w(0xAC) != 0 {
                    e.m.sw(0xAC, e.m.w(0xAC) - 1);
                } else {
                    e.b.motion();
                    if e.b.airborne == 0 {
                        e.b.set_yi_keep(0);
                        if e.b.act == 40 {
                            self.enemy_morgan_burst(e, true);
                        } else {
                            self.enemy_morgan_throw(e, h, true);
                        }
                        return;
                    }
                    if e.b.vy == 0 {
                        e.b.ay = -69;
                    }
                }
                self.enemy_step(&mut e.b);
            }
            _ => {}
        }
    }

    /// 0x51CDC / 0x51DAA: signed point steps and unsigned arrival windows.
    fn enemy_morgan_point(&mut self, e: &mut Enemy) {
        if morgan_count(e) {
            self.lib_face_target(e);
            self.enemy_morgan_decide(e);
            return;
        }
        let beside = e.b.act == 38;
        let t = self.lib_target(e);
        let (tx, tz) = (t.xi(), t.zi());
        let x = if beside {
            tx.wrapping_add(if e.b.facing == 0 { 32 } else { 0 })
        } else {
            e.m.w(0x88) as i16
        };
        let z = if beside { tz } else { e.m.w(0x8A) as i16 };
        if e.m.w(0xAC) & 3 == 0 {
            morgan_steps(e, x, z, 0x28000, 0x20000);
        }
        if e.m.b(0xA2) != 0 {
            e.m.sb(0xA2, e.m.b(0xA2) - 1);
            return;
        }
        let mut moved = false;
        let wx = if beside { 32 } else { 8 };
        let dest = if beside { tx } else { x };
        if dest.wrapping_add(wx).wrapping_sub(e.b.xi()) as u16 >= (wx as u16) * 2 {
            e.b.x = e.b.x.wrapping_add(e.m.l(0xB0) as i32);
            moved = true;
        }
        if z.wrapping_add(8).wrapping_sub(e.b.zi()) as u16 >= 16 {
            e.b.z = e.b.z.wrapping_add(e.m.l(0xB4) as i32);
            moved = true;
        }
        if moved {
            if e.m.b(0xA3) != 0 {
                let row = (self.w.level & 0xFF00)
                    | (self.w.level as u8).wrapping_add((e.b.var26 >> 8) as u8 >> 5) as u16;
                e.m.sb(0xA2, self.assets.enemy.t.u8(0x7449A, row as u32));
                e.m.sb(0xA3, 0);
            }
            if e.m.b(0xA2) == 0 {
                self.enemy_step(&mut e.b);
            }
        } else if beside {
            self.enemy_morgan_melee(e);
        } else {
            e.m.sb(0xA3, 1);
            e.m.sb(0xA2, 0);
        }
    }

    /// 0x52BFC: decide by phase, without a new slot or target RNG draw.
    fn enemy_morgan_decide(&mut self, e: &mut Enemy) {
        e.m.sw(0xAC, 60);
        self.enemy_set_speed(&mut e.b, 5);
        e.m.sb(0xA1, 0);
        e.m.sb(0xA5, self.lib_distance_class(e).0 as u8);
        match e.m.b(0xA0) & 3 {
            0 => self.enemy_morgan_kite(e),
            1 => {
                self.enemy_anim(&mut e.b, 1);
                e.m.sb(0xA1, 0);
                e.m.sw(0xAC, 60);
                e.b.act = 8;
                morgan_steps(e, e.m.w(0x88) as i16, e.m.w(0x8A) as i16, 0x15000, 0x10000);
            }
            2 => self.enemy_morgan_jump_kick(e, 0x1A),
            _ => self.enemy_morgan_jump_kick(e, 0x1C),
        }
    }

    /// 0x522A8: timed grid kite, with the ambiguous-direction RNG after the duration draw.
    fn enemy_morgan_kite(&mut self, e: &mut Enemy) {
        e.m.sb(0xA9, e.b.dir);
        self.lib_aim_beside_target(e, 48);
        self.lib_face_target(e);
        self.enemy_set_speed(&mut e.b, 11);
        e.m.sb(0xA1, 0);
        e.m.sb(0xA8, 6);
        let r = (self.rand() & 30) as u32;
        e.m.sw(0xAC, self.assets.enemy.t.u16(0x5236C, r));
        e.b.act = 16;
        self.enemy_morgan_grid(e, false);
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x51E8A / 0x51ED2 / 0x52310: grid classification and optional random tie break.
    fn enemy_morgan_grid(&mut self, e: &mut Enemy, turn: bool) {
        let t = self.lib_target(e);
        let a = morgan_cell(t.xi(), t.zi());
        let b = morgan_cell(e.b.xi(), e.b.zi());
        let row = self.assets.enemy.t.u16(0x5236C, 32 + 2 * (a - 1) as u32);
        let mut d = self.assets.enemy.t.u8(0x52EF0, row as u32 + (b - 1) as u32);
        if (d as i8) > 7 {
            let r = self.rand() & 15;
            let off = (((d.wrapping_sub(8)) << 4) as u16).wrapping_add(r);
            d = self.assets.enemy.t.u8(0x51EF0, off as u32);
        }
        e.m.sb(0xBE, d.wrapping_mul(4) & 31);
        e.m.sb(0xA9, e.b.dir);
        e.b.dir = e.m.b(0xBE);
        if e.b.xi().wrapping_sub(self.cam.s2.cam_x as i16) < 0 {
            e.b.dir = 8;
        }
        let right = (self.cam.s2.cam_x as i16)
            .wrapping_add(384)
            .wrapping_sub(e.b.xi())
            < 0;
        if right {
            e.b.dir = 24;
        }
        // In 0x51C36 the turn helper is only reached through the right-edge branch.
        if turn && right {
            self.enemy_morgan_turn(e);
        }
    }

    /// 0x5246A: rotate at most two direction units using signed byte compares.
    fn enemy_morgan_turn(&self, e: &mut Enemy) {
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

    /// 0x52C50..0x52CDC: the target check returns only from its own subroutine.
    fn enemy_morgan_gate(&mut self, e: &mut Enemy) {
        if !self.boss_aggression_gate(e, BossAi::Morgan) {
            return;
        }
        self.enemy_set_speed(&mut e.b, 5);
        e.m.sb(0xA1, 0);
        e.m.sb(0xA5, self.lib_distance_class(e).0 as u8);
        match self.boss_action_pick(e, BossAi::Morgan) {
            0 => self.enemy_morgan_melee(e),
            1 => {
                self.enemy_anim(&mut e.b, 1);
                e.m.sb(0xA1, 0);
                e.m.sw(0xAC, 60);
                e.b.act = 38;
                morgan_steps(e, e.m.w(0x88) as i16, e.m.w(0x8A) as i16, 0x15000, 0x10000);
            }
            2 => {
                if self.lib_target(e).live != 1 {
                    self.enemy_morgan_decide(e);
                    return;
                }
                let r = (self.rand() & 15) as u32;
                e.m.sb(0xAA, self.assets.enemy.t.u8(0x526A0, r));
                self.enemy_morgan_grenade(e);
            }
            3 => self.enemy_morgan_hop(e, 40, 5),
            4 => self.enemy_morgan_hop(e, 42, 5),
            _ => {}
        }
    }

    /// 0x527EC / 0x528EC: jump-kick starts; both attack-serial increments are intentional.
    fn enemy_morgan_jump_kick(&mut self, e: &mut Enemy, state: u8) {
        e.m.sb(0xA1, 1);
        e.m.sw(0xAC, 60);
        e.b.act = state;
        e.b.airborne = 1;
        self.enemy_begin_attack(&mut e.b, 0x34);
        self.lib_aim_at_target(e);
        if state == 0x1A {
            if e.m.b(0xA5) as i8 <= 5 {
                e.b.dir &= 16;
            }
            if self.lib_target(e).live != 1 {
                self.enemy_morgan_decide(e);
                return;
            }
        }
        let dx =
            e.b.x
                .wrapping_sub(self.lib_target(e).x)
                .wrapping_add(if e.b.facing == 0 { -0x300000 } else { 0x300000 });
        let speed = morgan_velocity(dx.wrapping_abs(), 0x3800, false).min(0x280);
        e.b.vx = if e.b.facing == 0 {
            speed.wrapping_neg()
        } else {
            speed
        };
        e.b.vy = 0x700;
        e.b.vz = 0;
        e.b.ax = 0;
        e.b.ay = -64;
        e.b.az = 0;
        self.enemy_begin_attack(&mut e.b, 0x34);
        self.enemy_anim(&mut e.b, 5);
    }

    /// 0x529B6 / 0x52A70 / 0x52B2A: hop over the target; strict subtraction loop rounds exact multiples up.
    fn enemy_morgan_hop(&mut self, e: &mut Enemy, state: u8, anim: u16) {
        e.m.sb(0xA1, 1);
        e.m.sw(0xAC, 10);
        e.b.act = state;
        e.b.airborne = 1;
        let t = self.lib_target(e);
        let (tx, tz) = (t.x, t.z);
        let cam = (self.cam.s2.cam_x as i32) << 16;
        let x = tx.wrapping_add(if tx.wrapping_sub(cam) > 0xC00000 {
            -0xB00000
        } else {
            0xB00000
        });
        e.b.vx = morgan_velocity(x.wrapping_sub(e.b.x), 0x2800, state == 42);
        e.b.vz = morgan_velocity(tz.wrapping_sub(e.b.z), 0x2800, state == 42);
        e.b.vy = 0x380;
        e.b.ax = 0;
        e.b.ay = -64;
        e.b.az = 0;
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x525B4 / 0x52668 / 0x526B0 / 0x527AE: attack serial and common pose setup.
    fn enemy_morgan_attack(&mut self, e: &mut Enemy, state: u8, anim: u16, attack: u8) {
        self.enemy_begin_attack(&mut e.b, attack);
        e.m.sb(0xA1, 1);
        e.b.act = state;
        e.m.sw(0xAC, 60);
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x52732: delayed close punch. The second setup deliberately increments the serial again.
    fn enemy_morgan_melee(&mut self, e: &mut Enemy) {
        self.lib_face_target(e);
        self.enemy_begin_attack(&mut e.b, 0x3F);
        e.m.sb(0xA1, 1);
        e.b.act = 24;
        let r = (self.rand() & 30) as u32;
        e.m.sw(0xAC, self.assets.enemy.t.u16(0x52788, r));
        if e.m.b(0xB9) != 0 {
            self.enemy_morgan_attack(e, 14, 6, 0x3F);
            e.m.sb(0xB9, 0);
        } else {
            self.lib_aim_at_target(e);
            self.enemy_anim(&mut e.b, 6);
        }
    }

    /// 0x524D2: stop and wait after the evasive step.
    fn enemy_morgan_wait(&mut self, e: &mut Enemy) {
        e.m.sb(0xA1, 1);
        e.m.sw(0xAC, 30);
        e.b.act = 10;
        self.enemy_anim(&mut e.b, 0);
        self.enemy_set_speed(&mut e.b, 5);
    }

    /// 0x526EE / 0x526F4: five-shot Uzi burst.
    fn enemy_morgan_burst(&mut self, e: &mut Enemy, first: bool) {
        if first {
            e.m.sb(0xA4, 5);
        }
        self.lib_face_target(e);
        self.enemy_begin_attack(&mut e.b, 8);
        e.m.sb(0xA1, 1);
        e.b.act = 32;
        e.m.sw(0xAC, 60);
        self.lib_aim_at_target(e);
        self.enemy_anim(&mut e.b, 7);
    }

    /// 0x52668: one grenade cycle (the animation supplies the 80-frame period).
    fn enemy_morgan_grenade(&mut self, e: &mut Enemy) {
        self.enemy_begin_attack(&mut e.b, 0x3F);
        e.m.sb(0xA1, 1);
        e.b.act = 30;
        e.m.sw(0xAC, 60);
        self.lib_aim_at_target(e);
        self.enemy_anim(&mut e.b, 10);
    }

    /// 0x52004: allocation failure still decrements the repeat count.
    fn enemy_morgan_spawn_grenade(&mut self, e: &mut Enemy, h: Handle) {
        if let Some(j) = self.enemies.alloc() {
            let r = &mut self.enemies.slots[j];
            r.b.live = 1;
            r.b.k20 = 26 * 4;
            r.b.var26 = 0;
            r.b.facing = e.b.facing;
            r.b.set_xi_keep(e.b.xi().wrapping_add(if e.b.facing == 0 { -36 } else { 0 }));
            r.b.set_yi_keep(e.b.yi().wrapping_add(48));
            r.b.set_zi_keep(e.b.zi());
            r.m.sl(0xA0, h as i16 as i32 as u32);
        }
        let n = e.m.b(0xAA).wrapping_sub(1);
        e.m.sb(0xAA, n);
        if n == 0 {
            self.enemy_morgan_decide(e);
        } else {
            self.enemy_morgan_grenade(e);
        }
    }

    /// 0x52504 / 0x52514: only successful allocation consumes the variant RNG draw.
    fn enemy_morgan_throw(&mut self, e: &mut Enemy, h: Handle, first: bool) {
        if first {
            let r = (self.rand() & 15) as u32;
            e.m.sb(0xBC, self.assets.enemy.t.u8(0x524F4, r));
        }
        self.lib_face_target(e);
        if let Some(j) = self.enemies.alloc() {
            let r = (self.rand() & 30) as u32;
            let variant = self.assets.enemy.t.u16(0x52594, r);
            let t = &mut self.enemies.slots[j];
            t.b.live = 1;
            t.b.k20 = 68 * 4;
            t.b.var26 = variant;
            t.b.facing = e.b.facing;
            t.b.set_xi_keep(
                e.b.xi()
                    .wrapping_add(if e.b.facing == 0 { 16 } else { -16 }),
            );
            t.b.set_yi_keep(e.b.yi().wrapping_add(48));
            t.b.set_zi_keep(e.b.zi());
            t.m.sl(0xA0, h as i16 as i32 as u32);
        }
        e.m.sb(0xA1, 1);
        e.m.sw(0xAC, 15);
        e.b.act = 34;
        self.enemy_anim(&mut e.b, 11);
    }

    /// 0x52CEC: combat rifle rows use a Z offset and +$B8 marker cache.
    fn enemy_morgan_rifle_pose(&mut self, e: &Enemy) {
        let Some(Slot::Enemy(i)) = slot_of(e.m.w(0xAE)) else {
            return;
        };
        if self.enemies.slots[i].b.live == 0 {
            return;
        }
        let mut r = std::mem::take(&mut self.enemies.slots[i]);
        let k = e.b.param_b as u32 * 8;
        let t = &self.assets.enemy.t;
        let dx = t.i16(0x5BFDE, k);
        r.b.facing = e.b.facing;
        r.b.set_xi_keep(e.b.xi().wrapping_add(if e.b.facing == 0 {
            dx
        } else {
            dx.wrapping_neg()
        }));
        r.b.set_yi_keep(e.b.yi().wrapping_add(t.i16(0x5BFDE, k + 2)));
        r.b.set_zi_keep(e.b.zi().wrapping_add(t.i16(0x5BFDE, k + 4)));
        let a = t.u16(0x5BFDE, k + 6).min(26);
        r.m.sw(0xA6, a);
        if r.m.b(0xB8) != e.b.param_b {
            r.m.sb(0xB8, e.b.param_b);
            self.enemy_anim(&mut r.b, a);
        }
        self.enemies.slots[i] = r;
    }
}

/// 0x51E8A: signed Z bands and five strict X thresholds.
fn morgan_cell(x: i16, z: i16) -> u16 {
    let z = z.wrapping_sub(0x110);
    let mut n = if z <= 26 {
        13
    } else if z <= 52 {
        7
    } else {
        1
    };
    let mut x = x.wrapping_sub(0x11C0);
    for _ in 0..5 {
        x = x.wrapping_sub(64);
        if x < 0 {
            break;
        }
        n += 1;
    }
    n
}
/// 0x51CDC / 0x523CC: immediate signed 16.16 step selection.
fn morgan_steps(e: &mut Enemy, x: i16, z: i16, sx: i32, sz: i32) {
    e.m.sl(0xB0, (if x > e.b.xi() { sx } else { -sx }) as u32);
    e.m.sl(0xB4, (if z > e.b.zi() { sz } else { -sz }) as u32);
}
/// 0x51CDC: SUBQ.W preserves underflow.
fn morgan_count(e: &mut Enemy) -> bool {
    let n = e.m.w(0xAC).wrapping_sub(1);
    e.m.sw(0xAC, n);
    n == 0
}
/// 0x52A08 / 0x52B7C: repeated SUBI.L including the bit-7 correction of state 0x2A.
fn morgan_velocity(delta: i32, step: i32, correct: bool) -> i16 {
    let mut v = (delta.wrapping_abs() as u32 / (step as u32)).wrapping_add(1) as i16;
    if correct && v & 128 != 0 {
        v = v.wrapping_add(1);
    }
    if delta < 0 { v.wrapping_neg() } else { v }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0x52A08 / 0x52B7C: exact multiples take another subtraction, bit 7 is corrected only for 0x2A.
    #[test]
    fn hop_velocity_keeps_the_original_rounding_and_sign() {
        assert_eq!(morgan_velocity(0, 0x2800, false), 1);
        assert_eq!(morgan_velocity(0x2800, 0x2800, false), 2);
        assert_eq!(morgan_velocity(-0x2800, 0x2800, false), -2);
        assert_eq!(morgan_velocity(127 * 0x2800, 0x2800, false), 128);
        assert_eq!(morgan_velocity(127 * 0x2800, 0x2800, true), 129);
        assert_eq!(morgan_velocity(-127 * 0x2800, 0x2800, true), -129);
    }

    /// 0x52D7A: credit once while HP remains, no score or final boss-clear flags.
    #[test]
    fn intermediate_hit_credit_preserves_rebirth_and_score() {
        let mut g = Game::new(Box::leak(Box::new(dino_assets::AssetPack::default())));
        let mut e = Enemy::default();
        e.b.who = 0;
        e.b.hp_prev = 256;
        e.b.hp = 245;
        g.enemy_morgan_hit_credit(&mut e);
        assert_eq!((g.players[0].w10c, e.b.hp_prev), (11, 245));
        g.enemy_morgan_hit_credit(&mut e);
        assert_eq!(g.players[0].w10c, 11);
        e.b.hp = 0;
        g.enemy_morgan_hit_credit(&mut e);
        assert_eq!(g.players[0].w10c, 256);
        assert_eq!((g.flow.speed, g.spawn.g.kill_all, g.flow.sub), (3, 2, 0));
        assert_eq!(g.players[0].score, 0);
    }
}
