//! Morgan's held and thrown objects, handler 0x5C0B6.
use super::Enemy;
use crate::game::{Game, Handle};
impl Game {
    /// 0x5C0CA / 0x5C18A / 0x5C238: float in the hand until owner marker 0x18, then launch.
    pub(super) fn enemy_class68(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                e.m.sb(0x87, 0);
                e.m.sb(0xAF, 0);
                e.b.hitstop = 0;
                e.b.box_table = 0x103C00;
                e.b.multi_box = false;
                self.enemy_begin_attack(&mut e.b, 0x45);
                e.b.live = 2;
                e.m.sb(0x7D, 0xFF);
                e.b.anim_table = 0x71408;
                e.m.sb(0xA5, 0);
                e.m.sw(0xB2, 0);
                let o = e.m.l(0xA0) as Handle;
                if let Some(crate::game::Slot::Enemy(j)) = crate::game::slot_of(o) {
                    let owner = &self.enemies.slots[j];
                    for k in [0x76, 0x88, 0x8A] {
                        e.m.sw(k, owner.m.w(k));
                    }
                }
                self.enemy_floor_collision(e);
                e.m.sb(0xB0, 0);
                e.b.mode += 2;
                e.b.airborne = 1;
                e.b.vx = 0;
                e.b.vy = 0x3A0;
                e.b.vz = 0;
                e.b.ax = 0;
                e.b.ay = -64;
                e.b.az = 0;
                let anim = e.b.var26.wrapping_add(5);
                self.enemy_anim(&mut e.b, anim);
                self.enemy_draw_check(&mut e.b, h);
            }
            2 => {
                e.b.motion();
                self.enemy_floor_collision(e);
                if self.body(e.m.l(0xA0) as Handle).param_b == 24 {
                    if e.m.b(0xB0) == 0 {
                        e.m.sb(0xB0, 1);
                        e.b.vx = 0x680;
                        e.b.vy = 0x100;
                        e.b.ax = 0;
                        e.b.ay = -8;
                        e.b.az = 0;
                        e.b.vz = 0;
                        if e.b.var26 == 1 {
                            e.b.vx = 0x380;
                            e.b.ay = -32;
                            e.b.vy = 0x200;
                        }
                        if e.b.var26 == 2 {
                            self.enemy_begin_attack(&mut e.b, 0x45);
                        }
                        let anim = e.b.var26;
                        self.enemy_anim(&mut e.b, anim);
                        if e.b.facing == 0 {
                            e.b.vx = e.b.vx.wrapping_neg();
                        }
                    }
                    return;
                }
                if e.b.airborne == 0 {
                    self.enemy_anim(&mut e.b, 0);
                    if e.b.var26 == 1 {
                        if let Some(j) = self.items.alloc() {
                            let r = &mut self.items.slots[j];
                            r.b.live = 1;
                            r.b.k20 = 0;
                            r.b.var26 = 24;
                            r.b.set_xi_keep(e.b.xi());
                            r.b.set_yi_keep(e.b.yi());
                            r.b.set_zi_keep(e.b.zi());
                        }
                    }
                    e.b.mode = 4;
                    return;
                }
                if e.b.var26 == 0 && e.b.hitstop != 0 {
                    self.item_explosion(
                        &crate::game::item::Item {
                            b: e.b.clone(),
                            ..Default::default()
                        },
                        h,
                        0,
                    );
                    e.b.mode = 4;
                }
                self.enemy_step(&mut e.b);
                self.enemy_despawn_if_far(e, i);
                e.m.sw(0xB2, e.m.w(0xB2).wrapping_add(1));
                if e.m.b(0xB0) != 0 {
                    self.enemy_push_hurtable(h);
                }
                self.enemy_draw_check(&mut e.b, h);
            }
            _ => self.enemy_free_pool_a(e, i),
        }
    }
}
