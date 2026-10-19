//! Pool A class 7 (`+$20` = 0x1C), handler 0x3B104: the acid ball of the Bludge (A7, M7-08). Made by the spit of the Bludge (0x3A068) 0x42 pixels in front of
//! the mouth with the facing of the Bludge. Mode 0 (0x3B11A, falls into the mode 2): animation table 0x3B21C animation 0, `+$7D` and `+$63` 0xFF, the boxes
//! 0x103000, in the air, X speed 0x100 to the facing side, no Y speed, Y acceleration -0x30. Mode 2 (0x3B172): sub 0 flies (0x189C every frame, the animation
//! tick) until the floor stops it, then the attack 0x27 and animation 1 (the splash); sub 2 makes the effect (pool C class 18) at the frame flag `+$2D` and
//! ends with the animation (mode 4). Every mode 2 frame ends with the floor (0x12CB4), the draw check and the list of the hurtable. Modes 4 and 6 free it.

use super::Enemy;
use crate::game::{Game, Handle};

/// 0x3B21C: the animation table (the splash and the flight).
const ANIMS: u32 = 0x3B21C;

impl Game {
    /// 0x3B104.
    pub fn enemy_class7(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.enemy_c7_init(e);
                self.enemy_c7_run(e, h);
            }
            2 => self.enemy_c7_run(e, h),
            _ => self.enemy_free_pool_a(e, i), // 0x3B216: jmp 0x48FC
        }
    }

    /// 0x3B11A.
    fn enemy_c7_init(&mut self, e: &mut Enemy) {
        e.b.anim_table = ANIMS;
        self.enemy_anim(&mut e.b, 0);
        e.m.sb(0x7D, 0xFF);
        e.b.b63 = 0xFF;
        e.b.mode += 2;
        e.b.box_table = 0x103000;
        e.b.multi_box = false;
        e.b.airborne = 1;
        e.b.vx = if e.b.facing != 0 { 0x100 } else { -0x100 };
        e.b.ax = 0;
        e.b.vy = 0;
        e.b.ay = -0x30;
        e.b.vz = 0;
        e.b.az = 0;
    }

    /// 0x3B172.
    fn enemy_c7_run(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => {
                // 0x3B196
                if e.b.airborne != 0 {
                    e.b.motion(); // 0x189C
                    self.enemy_step(&mut e.b);
                } else {
                    e.b.sub += 2; // 0x3B1A8
                    self.enemy_begin_attack(&mut e.b, 0x27);
                    self.enemy_anim(&mut e.b, 1);
                    self.enemy_c7_splash(e);
                }
            }
            _ => self.enemy_c7_splash(e),
        }
        self.enemy_floor_collision(e); // 0x12CB4
        self.enemy_draw_check(&mut e.b, h); // 0x49CA
        self.enemy_push_hurtable(h); // 0x4BEA
    }

    /// 0x3B1CC: the frame flag makes the effect, the end of the animation ends the object.
    fn enemy_c7_splash(&mut self, e: &mut Enemy) {
        if e.b.param_b != 0 {
            self.fx_spawn_acid(e.b.xi(), e.b.yi(), e.b.zi(), e.b.wading); // 0x3B1E8
            e.b.param_b = 0;
        }
        if self.enemy_step(&mut e.b) {
            e.b.mode += 2;
        }
    }
}
