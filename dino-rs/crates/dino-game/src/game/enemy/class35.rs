//! Morgan's village scene, 0x513DA. Captions belong to B50.
use super::Enemy;
use crate::game::{Game, Handle};

impl Game {
    /// 0x513DA / 0x513EE: scene dispatch, then update the linked rifle.
    pub(super) fn enemy_class35(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => match e.b.sub {
                0 => {
                    e.b.live = 2;
                    e.b.anim_table = 0x70F02;
                    e.m.sw(0xA0, 0);
                    e.m.sw(0xA2, 100);
                    self.enemy_floor_collision(e);
                    self.enemy_anim(&mut e.b, 1);
                    e.m.sw(0xA4, 300);
                    e.b.sub = 2;
                    e.m.sw(0xAE, 0);
                    self.enemy_c35_rifle(e, h, 1);
                }
                2 | 8 | 14 => {
                    if enemy_scene_count(e) {
                        e.b.sub += 2;
                    }
                    self.enemy_draw_check(&mut e.b, h);
                }
                4 => {
                    e.m.sw(0xA4, 60);
                    e.b.sub += 2;
                    self.enemy_draw_check(&mut e.b, h);
                }
                6 => {
                    if enemy_scene_count(e) {
                        e.b.sub += 2;
                        e.m.sw(0xA4, 40);
                        self.enemy_anim(&mut e.b, 12);
                    }
                    e.b.set_xi_keep(e.b.xi().wrapping_sub(1));
                    self.enemy_step(&mut e.b);
                    self.enemy_draw_check(&mut e.b, h);
                }
                10 => {
                    self.enemy_morgan_gun_sound(e);
                    e.m.sw(0xA4, 30);
                    e.b.sub += 2;
                    self.enemy_draw_check(&mut e.b, h);
                }
                12 => {
                    if enemy_scene_count(e) {
                        e.b.sub += 2;
                        e.m.sw(0xA4, 240);
                        self.enemy_anim(&mut e.b, 0);
                    }
                    let n = e.m.w(0xA4) as i16;
                    if n <= 16 && n & 3 == 0 {
                        self.enemy_morgan_gun_sound(e);
                        self.enemy_spawn_bullet(e, 5, 32, 36);
                    }
                    self.enemy_step(&mut e.b);
                    self.enemy_draw_check(&mut e.b, h);
                }
                16 => {
                    e.m.sw(0xA4, 200);
                    e.b.sub = 0;
                    self.enemy_draw_check(&mut e.b, h);
                    self.enemy_c35_rewrite(e, h);
                }
                _ => {}
            },
            2 => {
                if enemy_scene_count(e) {
                    e.b.mode = e.b.mode.wrapping_add(2);
                }
                self.enemy_step(&mut e.b);
                if e.m.w(0xA0) == 14 {
                    e.b.palette_override = e.b.param_a.wrapping_add(128);
                }
                self.enemy_despawn_if_far(e, i);
                self.enemy_c35_rifle_pose(e);
                self.enemy_draw_check(&mut e.b, h);
                return;
            }
            4 | 6 => {
                self.enemy_c35_rewrite(e, h); // 0x5166C
                return;
            }
            _ => {
                self.enemy_free_pool_a(e, i);
                return;
            }
        }
        self.enemy_c35_rifle_pose(e);
    }

    /// 0x9D9C: enemy firearm sound, with the original facing channel and X pan.
    pub(super) fn enemy_morgan_gun_sound(&mut self, e: &Enemy) {
        let code = if e.m.b(0x82) != 0 {
            0xDC
        } else {
            let voice = match crate::game::slot_of(e.b.partner) {
                Some(crate::game::Slot::Player(i)) => self.players[i].voice,
                _ => 0,
            };
            self.assets.enemy.t.u8(0x9E04, voice as u32) as u16
        };
        let pan = self.snd_pan_x(e.b.xi()) as u32;
        self.sound_ex(
            code,
            ((if e.b.facing == 0 { 3 } else { 4 }) << 16) | 0x2000 | pan,
            0x2400000,
        );
    }

    /// 0x51448 / 0x516CC..0x51728: rifle allocation and owner link.
    pub(super) fn enemy_c35_rifle(&mut self, e: &mut Enemy, h: Handle, variant: u16) {
        if let Some(j) = self.enemies.alloc() {
            let r = &mut self.enemies.slots[j];
            r.b.live = 1;
            r.b.k20 = 67 * 4;
            r.b.var26 = variant;
            r.b.anim_table = 0x71758;
            r.m.sl(0xA0, h as i16 as i32 as u32);
            e.m.sw(0xAE, self.enemies.handle(j));
        }
    }

    /// 0x515C2: frame-marker driven rifle placement, including unchanged animation on a repeated marker.
    pub(super) fn enemy_c35_rifle_pose(&mut self, e: &Enemy) {
        let h = e.m.w(0xAE);
        let Some(crate::game::Slot::Enemy(i)) = crate::game::slot_of(h) else {
            return;
        };
        let mut r = std::mem::take(&mut self.enemies.slots[i]);
        let k = e.b.param_b as u32 * 8;
        let t = &self.assets.enemy.t;
        r.b.facing = e.b.facing;
        let dx = t.i16(0x5BFDE, k);
        r.b.set_xi_keep(e.b.xi().wrapping_add(if e.b.facing == 0 {
            dx
        } else {
            dx.wrapping_neg()
        }));
        r.b.set_yi_keep(e.b.yi().wrapping_add(t.i16(0x5BFDE, k + 2)));
        r.b.set_zi_keep(e.b.zi());
        let a = t.i16(0x5BFDE, k + 6);
        let a = if a > 47 { 25 } else { a as u16 };
        r.m.sw(0xA6, a);
        if r.m.b(0xA4) != e.b.param_b {
            r.m.sb(0xA4, e.b.param_b);
            self.enemy_anim(&mut r.b, a);
        }
        self.enemies.slots[i] = r;
    }

    /// 0x51646 / 0x51658: retire the scene rifle and rewrite the same slot to the boss.
    fn enemy_c35_rewrite(&mut self, e: &mut Enemy, h: Handle) {
        if let Some(crate::game::Slot::Enemy(i)) = crate::game::slot_of(e.m.w(0xAE)) {
            self.enemies.slots[i].b.mode = 6;
            self.enemies.slots[i].b.var26 = 1;
        }
        e.b.k20 = 36 * 4;
        e.b.set_state(0);
        self.enemy_c35_rifle_pose(e);
        self.enemy_draw_check(&mut e.b, h);
    }
}

/// 0x51558: SUBI.W, including underflow when the counter starts at zero.
pub(super) fn enemy_scene_count(e: &mut Enemy) -> bool {
    let n = e.m.w(0xA4).wrapping_sub(1);
    e.m.sw(0xA4, n);
    n == 0
}
