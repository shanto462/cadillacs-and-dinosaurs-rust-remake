//! Pool A class 95 (handler 0x6995E, table 0x69B1C): the fireball of Fessenden form 2's heads, made by 0x69932 at the mouth. Its variant (0 .. 8)
//! picks the speed pair of 0x69C74 (X mirrored by the facing, Y, both in 1/256 px): flat for a far hero, steep for a close one. It flies
//! with attack 0x62, explodes on a hit (animation 2) or on the ground (animation 1) with the rumble (0x9F12) and is freed at the end of that
//! animation. D3d 6.3, BT18. Owner: enemy port.

use super::{off::*, Enemy};
use crate::game::{Game, Handle};

/// 0x69C74: the speed words (X, Y) of the variants (the variant indexes words).
const SPEEDS: u32 = 0x69C74;

impl Game {
    /// 0x69932: a fireball (class 95) at (x, y, z) with the variant and the facing (a full pool makes none).
    pub(super) fn c95_spawn(&mut self, x: i16, y: i16, z: i16, var: u16, facing: u8) {
        let Some(j) = self.enemies.alloc() else { return };
        let n = &mut self.enemies.slots[j];
        n.b.live = 1;
        n.b.k20 = 95 * 4;
        let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
        put(&mut n.b.x, x);
        put(&mut n.b.y, y);
        put(&mut n.b.z, z);
        n.b.var26 = var;
        n.b.facing = facing;
    }

    /// 0x6995E (mode table 0x6996A): 0 the init 0x69972 (falls into 2), 2 the flight 0x699F0, 4 and 6 the free.
    pub(super) fn enemy_class95(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.c95_init(e);
                self.c95_run(e, h, i);
            }
            2 => self.c95_run(e, h, i),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x69972: the table 0x69B1C, draw flags 0x10, `+$7D` and `+$63` 0xFF, in the air; `+$A0` / `+$A4` the X / Y steps (16.16) from the
    /// variant's words (X negated when facing right); the box table 0x100000; attack 0x62; animation 0.
    fn c95_init(&mut self, e: &mut Enemy) {
        e.b.anim_table = 0x69B1C;
        e.b.draw_flags = 0x10;
        e.m.sb(ATTACKING, 0xFF);
        e.b.hitstop = 0;
        e.b.airborne = 1;
        e.b.b63 = 0xFF;
        let off = (e.b.var26 as u32).wrapping_mul(2);
        let tb = &self.assets.enemy.t;
        let mut vx = ((tb.i16(SPEEDS, off) as i32) << 16) >> 8;
        if e.b.facing != 0 {
            vx = vx.wrapping_neg();
        }
        e.m.sl(0xA0, vx as u32);
        // `move.w (a0)+,d0; swap d0; asr.l #8,d0`: the low half of d0 before the swap is the high word of the X step, so its top byte lands
        // in the lowest byte of the Y step (the X step's own low byte comes from the dispatcher's d0, 0 here).
        let raw = ((tb.u16(SPEEDS, off + 2) as u32) << 16) | ((vx as u32) >> 16);
        let vy = (raw as i32) >> 8;
        e.m.sl(0xA4, vy as u32);
        e.b.box_table = 0x100000;
        e.b.multi_box = false;
        self.enemy_begin_attack(&mut e.b, 0x62);
        self.enemy_anim(&mut e.b, 0);
        e.b.mode = e.b.mode.wrapping_add(2);
    }

    /// 0x699F0: in flight (`+$06` 0): animate, X and Y move by the steps; a hit (hit stop set) or the ground (0x12CB4) starts the burst (attack
    /// 0x62 again, animation 2 / 1, the rumble 0x9F12, `+$06` 2); the burst animates and is freed at its end. The hurtable list, the far test
    /// (0x173C) and the draw check.
    fn c95_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.act != 0 {
            if self.enemy_step(&mut e.b) {
                return self.enemy_free_pool_a(e, i);
            }
        } else {
            self.enemy_step(&mut e.b);
            e.b.x = e.b.x.wrapping_add(e.m.l(0xA0) as i32);
            e.b.y = e.b.y.wrapping_add(e.m.l(0xA4) as i32);
            if e.b.hitstop != 0 {
                self.c95_burst(e, 2);
            } else {
                if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
                    e.m.sw(OBSTACLE, ob);
                }
                if e.b.airborne == 0 {
                    self.c95_burst(e, 1);
                }
            }
        }
        self.enemy_push_hurtable(h);
        self.enemy_despawn_if_far(e, i);
        self.enemy_draw_check(&mut e.b, h);
    }

    fn c95_burst(&mut self, e: &mut Enemy, anim: u16) {
        self.enemy_begin_attack(&mut e.b, 0x62);
        self.enemy_anim(&mut e.b, anim);
        self.snd_rumble(e.b.xi(), e.b.facing); // 0x9F12
        e.b.act = 2;
    }
}
