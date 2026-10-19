//! Morgan's attached Uzi and its floor drop, 0x5BE86.
use super::Enemy;
use crate::game::{Game, Handle, Slot, slot_of};
impl Game {
    /// 0x5BE9E / 0x5BEDC / 0x5BF22 / 0x5BF98: attached, tossed and dropped Uzi.
    pub(super) fn enemy_class67(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                e.b.box_table = 0x103C00;
                e.b.multi_box = false;
                self.enemy_begin_attack(&mut e.b, 0x1E);
                e.b.live = 2;
                e.m.sb(0x7D, 0xFF);
                e.b.b63 = 0xFF;
                e.b.anim_table = 0x71758;
                e.b.draw_flags = 0x10;
                e.m.sb(0xA5, 0);
                self.enemy_anim(&mut e.b, 0);
                self.enemy_floor_collision(e);
                e.b.mode = 2;
            }
            2 => {
                let owner = self.body(e.m.l(0xA0) as Handle);
                if owner.mode < 4 && matches!(owner.k20, 140 | 144) {
                    self.enemy_draw_check(&mut e.b, h);
                } else {
                    e.b.mode = 4;
                    self.enemy_c67_fall(e, h, i);
                }
            }
            4 => self.enemy_c67_fall(e, h, i),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x5BF28..0x5BFD8: a scene rifle vanishes; the boss rifle flies up and becomes pool E var 6.
    fn enemy_c67_fall(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.var26 == 0 {
            if e.b.sub == 0 {
                e.b.airborne = 1;
                e.b.vx = 0;
                e.b.vy = 0x300;
                e.b.vz = 0;
                e.b.ax = 0;
                e.b.ay = -0x20;
                e.b.az = 0;
                e.b.sub = 2;
                self.enemy_anim(&mut e.b, 3);
                self.enemy_draw_check(&mut e.b, h);
                return;
            }
            e.b.motion();
            self.enemy_floor_collision(e);
            if e.b.airborne != 0 && e.b.yi() >= 0 {
                self.enemy_step(&mut e.b);
                self.enemy_draw_check(&mut e.b, h);
                return;
            }
            if let Some(j) = self.items.alloc() {
                let r = &mut self.items.slots[j];
                r.b.live = 1;
                r.b.k20 = 0;
                r.b.var26 = 6;
                r.b.set_xi_keep(e.b.xi());
                r.b.set_yi_keep(e.b.yi());
                r.b.set_zi_keep(e.b.zi());
                let item = self.items.handle(j);
                if let Some(Slot::Enemy(k)) = slot_of(e.m.l(0xA0) as Handle) {
                    self.enemies.slots[k].m.sw(0xAE, item);
                }
            }
        }
        e.b.mode = e.b.mode.wrapping_add(2);
        self.enemy_free_pool_a(e, i);
    }
}
