//! The helpers of the Butcher's starter (`class5.rs`): the carcass A51 (0x5717E, 282 bytes), the machete helper A53
//! (0x57630, 162 bytes) and the knife holder A58 (0x57E5E, 342 bytes). D3c section 3.1.
//!
//! All three use the same two level jump table: mode 0 = the init, 2 = the update, 4 and 6 = free. Owner: enemy port (M6-09).

use super::class5::ptr;
use super::lib32::put_hi;
use super::Enemy;
use crate::game::{slot_of, Game, Handle, Slot};

/// The two offset tables of the knife holder (0x57F0E for variant 0, 0x57F26 for the other), three records of
/// four words each (dx, dy, dz, animation), chosen by the frame byte `+$2C` of the Butcher.
const HOLDER_OFFSETS: [u32; 2] = [0x57F0E, 0x57F26];

impl Game {
    // ------------------------------------------------------------------ class 51

    /// 0x5717E: the carcass of the dead dinosaur. Variant 0 (the body) makes variant 0x100 (a second layer 6 pixels lower and
    /// 3 nearer that follows it); both show animation 0 / 1. The body slides right 4 pixels a frame once the Butcher
    /// has set `+$A2` (the turn step of class 5), and frees itself when it is far from the camera.
    pub(super) fn enemy_class51(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                // 0x57192
                e.b.mode = 2;
                e.b.live = 2;
                e.b.anim_table = 0x5727A;
                e.m.sw(0xA0, 0);
                self.enemy_floor_collision(e); // 0x12CB4
                if e.b.var26 >> 8 == 0 {
                    e.b.draw_flags = 0x10;
                    e.m.sb(0xA2, 0);
                    if self.c51_make_layer(e, h) {
                        self.enemy_anim(&mut e.b, 0); // 0x57212: jmp 0x120E, the frame ends here
                        return;
                    }
                }
                self.enemy_anim(&mut e.b, 1); // 0x57218, then the update in the same frame
                self.c51_update(e, h);
            }
            2 => self.c51_update(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x57274
            }
        }
    }

    /// 0x571C2: the second layer (variant 0x100) at the same X, Y - 6, Z - 3. False when the pool is full.
    fn c51_make_layer(&mut self, e: &Enemy, h: Handle) -> bool {
        let Some(a) = self.enemies.alloc() else { return false };
        let c = &mut self.enemies.slots[a];
        c.b.live = 1;
        c.b.k20 = 0xCC;
        put_hi(&mut c.b.x, e.b.xi());
        put_hi(&mut c.b.y, e.b.yi().wrapping_sub(6));
        put_hi(&mut c.b.z, e.b.zi().wrapping_sub(3));
        c.b.var26 = 0x100;
        c.b.dir = e.b.dir;
        c.b.draw_flags = 0x10;
        c.m.sl(0xA4, ptr(h));
        true
    }

    /// 0x57220: the body slides while `+$A2` is set, the layer copies the body's position (the integer words, X + 0,
    /// Y - 6, Z - 3); then the draw check, the animation and the far test (0x57262, in this order).
    fn c51_update(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.var26 >> 8 == 0 {
            if e.m.b(0xA2) != 0 {
                e.b.set_xi_keep(e.b.xi().wrapping_add(4));
            }
        } else if let Some(Slot::Enemy(i)) = slot_of(e.m.w(0xA6)) {
            let (x, y, z) = {
                let p = &self.enemies.slots[i].b;
                (p.xi(), p.yi(), p.zi())
            };
            e.b.set_xi_keep(x);
            e.b.set_yi_keep(y.wrapping_sub(6));
            e.b.set_zi_keep(z.wrapping_sub(3));
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_step(&mut e.b);
        self.enemy_despawn_if_far(e, 0);
    }

    // ------------------------------------------------------------------ class 53

    /// 0x57630: the machete helper. Animation 1 only; the Butcher sets its mode to 6 when the talk starts and it frees
    /// itself in the next update. Draw check, animation, far test.
    pub(super) fn enemy_class53(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                // 0x57644
                e.b.mode = 2;
                e.b.live = 2;
                e.b.anim_table = 0x57682;
                e.m.sw(0x80, 0);
                self.enemy_floor_collision(e);
                self.enemy_anim(&mut e.b, 1);
                self.c53_update(e, h);
            }
            2 => self.c53_update(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x5767C
            }
        }
    }

    /// 0x5766A.
    fn c53_update(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_step(&mut e.b);
        self.enemy_despawn_if_far(e, 0);
    }

    // ------------------------------------------------------------------ class 58

    /// 0x57E5E: a knife holder. It stands where the Butcher's hand is: his position plus a row of the offset table by
    /// his frame byte `+$2C`, his facing, and the animation of the row restarted in every frame. It shows only while the
    /// Butcher's `+$AC` flag is set (the talk step). Mode 4 (set by the Butcher at the change to class 61) frees it.
    pub(super) fn enemy_class58(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                // 0x57E72
                e.b.live = 2;
                e.b.anim_table = 0x57F44;
                e.b.draw_flags = 0x10;
                e.m.sw(0xA4, 0);
                e.m.sb(0x7D, 0xFF);
                self.enemy_floor_collision(e);
                self.enemy_anim(&mut e.b, 0);
                e.b.mode = e.b.mode.wrapping_add(2);
                self.c58_follow(e, h);
            }
            2 => self.c58_follow(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x57F3E
            }
        }
    }

    /// 0x57EA2: nothing at all (not even the draw check) while the owner's `+$AC` is 0.
    fn c58_follow(&mut self, e: &mut Enemy, h: Handle) {
        let Some(Slot::Enemy(p)) = slot_of(e.m.w(0xA2)) else { return };
        let (shown, frame, x, y, z, facing) = {
            let o = &self.enemies.slots[p];
            (o.m.b(0xAC) != 0, o.b.param_a as u32, o.b.xi(), o.b.yi(), o.b.zi(), o.b.facing)
        };
        if !shown {
            return; // 0x57F0C
        }
        let table = HOLDER_OFFSETS[(e.b.var26 >> 8 != 0) as usize];
        let t = &self.assets.enemy.t;
        let row = frame * 8;
        let (dx, dy, dz, anim) = (t.i16(table, row), t.i16(table, row + 2), t.i16(table, row + 4), t.u16(table, row + 6));
        e.b.set_xi_keep(x.wrapping_add(dx));
        e.b.set_yi_keep(y.wrapping_add(dy));
        e.b.set_zi_keep(z.wrapping_add(dz));
        e.b.facing = facing;
        self.enemy_anim(&mut e.b, anim);
        self.enemy_draw_check(&mut e.b, h);
    }
}
