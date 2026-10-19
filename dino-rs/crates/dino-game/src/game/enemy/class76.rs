//! Pool A class 76 (handler 0x5EC26..0x5EDF4): the creature that jumps out of the broken crate of the Tyrog scene (table 0x71D66). It leaps
//! (vy 0x600, gravity -0x40, 1 px per frame to the left) with a puff (class 75 variant 6) every 8 frames, lands (animation 1, sound 0x82),
//! rises (animation 2, sounds 0x83 and 0x60) and then becomes Tyrog: it makes class 4 variant 0x200 at its place and frees itself. D3d 3.2.
//! Owner: enemy port.

use super::{off::*, Enemy};
use crate::game::{Game, Handle};

impl Game {
    /// 0x5EC26 (mode table 0x5EC32): 0 the init 0x5EC3A (falls into 2), 2 the run 0x5EC70, 4 and 6 the free 0x5EDEE.
    pub(super) fn enemy_class76(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                // 0x5EC3A
                e.b.live = 2;
                e.b.sub = 0;
                e.b.anim_table = 0x71D66;
                e.b.draw_flags = 0x10;
                e.m.sw(0xA4, 0);
                e.m.sw(0xA0, 0);
                if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
                    e.m.sw(OBSTACLE, ob);
                }
                self.enemy_anim(&mut e.b, 0);
                e.b.mode = e.b.mode.wrapping_add(2);
                self.c76_run(e, h, i);
            }
            2 => self.c76_run(e, h, i),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x5EC70: by `+$05` (table 0x5EC8A): 0 the leap start, 2 the leap, 4 the landing, 6 the rise, 8 Tyrog; then the far test and the draw check.
    fn c76_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.sub == 0 {
            // 0x5EC94
            e.b.sub = 2;
            e.m.sw(0xB2, 0x3C);
            e.b.airborne = 1;
            e.b.vx = -0x100;
            e.b.ax = 0;
            e.b.vy = 0x600;
            e.b.ay = -0x40;
            e.b.vz = 0;
            e.b.az = 0;
        }
        if e.b.sub == 2 {
            // 0x5ECCA: the terrain, the flight; a puff every 8 frames in the air.
            if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
                e.m.sw(OBSTACLE, ob);
            }
            e.b.motion();
            if e.b.airborne != 0 {
                let n = e.m.w(0xA4).wrapping_add(1);
                e.m.sw(0xA4, n);
                if n & 7 == 0 {
                    let (x, y, z, f) = (e.b.xi(), e.b.yi(), e.b.zi(), e.b.facing);
                    self.c75_spawn(75 * 4, 6, x, y, z.wrapping_sub(1), Some(f));
                }
                self.enemy_step(&mut e.b);
                return self.c76_tail(e, h, i);
            }
            // 0x5ED30: landed.
            self.enemy_anim(&mut e.b, 1);
            self.snd_request(0x82, 0x62020, 0x540080);
            e.b.sub = 4;
        }
        if e.b.sub == 4 {
            // 0x5ED56
            if !self.enemy_step(&mut e.b) {
                return self.c76_tail(e, h, i);
            }
            self.enemy_anim(&mut e.b, 2);
            self.snd_request(0x83, 0x62020, 0x540080);
            self.snd_request(0x60, 0, 0);
            e.b.sub = 6;
        }
        if e.b.sub == 6 {
            // 0x5EDA0
            if !self.enemy_step(&mut e.b) {
                return self.c76_tail(e, h, i);
            }
            e.b.sub = 8;
        }
        // 0x5EDB2: Tyrog (class 4 variant 0x200) at its place, 1 deeper; the free in the next frame.
        let (x, y, z) = (e.b.xi(), e.b.yi(), e.b.zi());
        self.c75_spawn(4 * 4, 0x200, x, y, z.wrapping_add(1), None);
        e.b.mode = e.b.mode.wrapping_add(2);
        self.c76_tail(e, h, i);
    }

    /// 0x5EC7E: the far test (0x173C) and the draw check (0x49CA).
    fn c76_tail(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        self.enemy_despawn_if_far(e, i);
        self.enemy_draw_check(&mut e.b, h);
    }
}
