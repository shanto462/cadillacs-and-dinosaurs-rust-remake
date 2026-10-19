//! Village chief, handler 0x5639A.
use super::{Enemy, class35::enemy_scene_count};
use crate::game::{Game, Handle};
impl Game {
    /// 0x5639A / 0x563AE / 0x563EE / 0x5649C: chief's wait, walk, fall and retirement.
    pub(super) fn enemy_class45(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.mode == 0 {
            e.b.live = 2;
            e.b.sub = 0;
            e.b.anim_table = 0x564D6;
            e.m.sb(0x7D, 0xFF);
            e.b.set_yi_keep(self.cam.s2.cam_y as i16);
            e.m.sw(0xA2, 100);
            self.enemy_floor_collision(e);
            self.sound(0x20);
            self.enemy_anim(&mut e.b, 0);
            e.b.mode = 2;
        }
        match e.b.mode {
            2 => {
                match e.b.sub {
                    0 => {
                        e.b.sub = 2;
                        e.m.sw(0xA4, 120);
                        e.b.set_zi_keep(80);
                        if enemy_scene_count(e) {
                            e.b.sub = 4;
                            e.m.sw(0xA4, 310);
                        }
                    }
                    2 => {
                        if enemy_scene_count(e) {
                            e.b.sub = 4;
                            e.m.sw(0xA4, 310);
                        }
                    }
                    4 => {
                        if enemy_scene_count(e) {
                            e.b.sub = 6;
                            e.m.sw(0xA4, 30);
                            self.snd_request(0x79, 0, 0);
                            self.enemy_anim(&mut e.b, 1);
                        } else if e.m.w(0xA4) as i16 > 130 {
                            e.b.set_xi_keep(e.b.xi().wrapping_sub(1));
                            self.enemy_step(&mut e.b);
                        }
                        self.enemy_draw_check(&mut e.b, h);
                    }
                    6 => {
                        if self.enemy_step(&mut e.b) {
                            e.b.mode += 2;
                            e.b.sub = 0;
                        }
                        self.enemy_draw_check(&mut e.b, h);
                    }
                    _ => {}
                }
                self.enemy_floor_collision(e);
            }
            4 => {
                if e.b.sub == 0 {
                    e.m.sw(0x80, 120);
                    e.b.sub += 2;
                }
                let n = e.m.w(0x80);
                if n == 0 {
                    self.enemy_free_pool_a(e, i);
                } else {
                    e.m.sw(0x80, n - 1);
                    if (n - 1) & 1 != 0 {
                        self.enemy_draw_check(&mut e.b, h);
                    }
                }
            }
            _ => self.enemy_free_pool_a(e, i),
        }
    }
}
