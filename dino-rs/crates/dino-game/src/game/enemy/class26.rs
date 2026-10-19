//! Grenade and dynamite flight, handler 0x4D48A. Morgan uses variant zero.
use super::Enemy;
use crate::game::{Game, Handle};
impl Game {
    /// 0x4D59E / 0x4D79A / 0x4D850 / 0x4D8E4: init, flight, detonation and impact animation.
    pub(super) fn enemy_class26(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                e.m.sb(0x87, 0);
                e.m.sb(0xAF, 0);
                e.b.hitstop = 0;
                e.b.box_table = 0x103C00;
                e.b.multi_box = false;
                self.enemy_begin_attack(&mut e.b, 0x44);
                e.b.live = 2;
                e.m.sb(0x7D, 0xFF);
                e.b.who = 0xFF;
                e.b.b63 = 0xFF;
                e.b.anim_table = 0x71408;
                e.m.sb(0xA5, 0);
                e.m.sw(0xBA, 0);
                let owner = e.m.l(0xA0) as Handle;
                if let Some(crate::game::Slot::Enemy(j)) = crate::game::slot_of(owner) {
                    e.m.sw(0x76, self.enemies.slots[j].m.w(0x76));
                }
                let t = self.lib_target(e);
                let (x, z) = (t.x, t.z);
                e.m.sl(0xB0, x as u32);
                e.m.sl(0xB4, z as u32);
                let k = (self.rand() & 31) as u32 * 4;
                let x = x.wrapping_add(self.assets.enemy.t.u32(0x4D49E, k) as i32);
                let z = z.wrapping_add(self.assets.enemy.t.u32(0x4D51E, k) as i32);
                e.m.sl(0xB0, x as u32);
                e.m.sl(0xB4, z as u32);
                self.enemy_floor_collision(e);
                e.b.mode += 2;
                e.b.act = 0;
                e.b.airborne = 1;
                if e.b.var26 == 4 {
                    e.b.palette_override = 0x81;
                    self.enemy_set_speed(&mut e.b, 9);
                    e.b.vy = 0x400;
                    self.enemy_begin_attack(&mut e.b, 0x4E);
                } else {
                    e.b.vx = grenade_velocity(x.wrapping_sub(e.b.x));
                    e.b.vz = grenade_velocity(z.wrapping_sub(e.b.z));
                    e.b.vy = 0x700;
                }
                e.b.ax = 0;
                e.b.ay = -64;
                e.b.az = 0;
                let a = match e.b.var26 {
                    0 => 0,
                    3 => {
                        e.b.vx = if self.body(owner).facing == 0 {
                            -0x800
                        } else {
                            0x680
                        };
                        e.b.vy = 0;
                        e.b.vz = 0;
                        e.b.ay = -56;
                        self.enemy_begin_attack(&mut e.b, 0x67);
                        12
                    }
                    4 => 10,
                    _ => 1,
                };
                self.enemy_anim(&mut e.b, a);
                self.enemy_draw_check(&mut e.b, h);
            }
            2 => {
                if e.b.sub != 0 {
                    self.enemy_c26_explode(e, h);
                    return;
                }
                if e.b.act == 2 {
                    if self.enemy_step(&mut e.b) {
                        self.enemy_free_pool_a(e, i);
                    } else {
                        self.enemy_push_hurtable(h);
                        self.enemy_draw_check(&mut e.b, h);
                    }
                    return;
                }
                if e.b.var26 < 3 && e.b.hitstop != 0 {
                    self.enemy_c26_explode(e, h);
                    return;
                }
                e.b.motion();
                self.enemy_floor_collision(e);
                if e.b.yi() < 0 || e.b.airborne == 0 {
                    if e.b.var26 >= 3 {
                        let anim = if e.b.var26 == 4 { 11 } else { 13 };
                        self.enemy_anim(&mut e.b, anim);
                        e.b.box_table = 0x103000;
                        e.b.multi_box = false;
                        e.b.act = 2;
                        if self.enemy_step(&mut e.b) {
                            self.enemy_free_pool_a(e, i);
                        } else {
                            self.enemy_push_hurtable(h);
                            self.enemy_draw_check(&mut e.b, h);
                        }
                    } else if e.b.var26 == 0 {
                        self.enemy_c26_explode(e, h);
                    } else {
                        self.enemy_anim(&mut e.b, 0);
                        if let Some(j) = self.items.alloc() {
                            let r = &mut self.items.slots[j];
                            r.b.live = 1;
                            r.b.k20 = 0;
                            r.b.var26 = 24;
                            r.b.step = 2;
                            r.b.set_xi_keep(e.b.xi());
                            r.b.set_yi_keep(e.b.yi());
                            r.b.set_zi_keep(e.b.zi());
                        }
                        e.b.mode = 6;
                    }
                    return;
                }
                self.enemy_step(&mut e.b);
                e.m.sw(0xBA, e.m.w(0xBA).wrapping_add(1));
                self.enemy_push_hurtable(h);
                self.enemy_draw_check(&mut e.b, h);
            }
            _ => self.enemy_free_pool_a(e, i),
        }
    }
    /// 0x4D850: grenade explosion is kind 3 even for variant zero.
    fn enemy_c26_explode(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.var26 >= 3 {
            e.b.sub = 0;
        } else {
            self.item_explosion(
                &crate::game::item::Item {
                    b: e.b.clone(),
                    ..Default::default()
                },
                h,
                3,
            );
            e.b.mode = 6;
        }
    }
}
/// 0x4D672 / 0x4D692: strict subtraction loop, signed word velocity.
fn grenade_velocity(delta: i32) -> i16 {
    let v = (delta.wrapping_abs() as u32 / 0x3800).wrapping_add(1) as i16;
    if delta < 0 { v.wrapping_neg() } else { v }
}
