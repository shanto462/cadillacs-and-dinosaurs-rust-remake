//! Pool A class 96 (`+$20 = 0x180`, handler 0x69C98): a flying piece of the explosion in the sunset scene of the ending (M3-23,
//! made by pool B 82 at its state 8, the variants 0 and 0x100). It waits 60 frames out of sight, then flies up and to the right
//! with a gravity for `+$B2` frames (the variant 0x100 is faster and breaks a wall: the break sound) and ends.
//!
//! | mode | sub | what |
//! | --- | --- | --- |
//! | 0 | | 0x69CAC: live 2, sub 0, `+$7D = 0xFF`, no shadow, the table 0x69E00, X = camera X - 0x10, Y 0, the animation 0 (1 for the variant 0x100), direction 0x18, facing 1, mode 2 and the sub state at once |
//! | 2 | 0 | 0x69D1E: the speed table 0x17, Z 0x50, the velocities and the sound 0x10 (0x69D6E); the variant 0x100 takes its own velocities and the break sound (0x69DA8); `+$B4` = 0x3C |
//! | 2 | 2 | 0x69D4E: waits `+$B4` frames |
//! | 2 | 4 | 0x69DD8: `+$B2` frames of flight: the motion 0x189C, the animation step, the draw check |
//! | 2 | 6 | 0x69D60: mode 4 (`+$B2` = 0x3C) |
//! | 4, 6 | | 0x69DFA: free (0x48FC) |

use super::Enemy;
use crate::game::{Game, Handle};

const TABLE: u32 = 0x69E00;
const T_FLIGHT: usize = 0xB2;
const T_WAIT: usize = 0xB4;

impl Game {
    /// 0x69C98 class 96 (jump table 0x69CA4: mode 0 -> 0x69CAC, 2 -> 0x69D06, 4 and 6 -> 0x69DFA).
    pub fn enemy_class96(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                self.c96_init(e);
                self.c96_sub(e, h);
            }
            2 => self.c96_sub(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i);
            }
        }
    }

    /// 0x69CAC.
    fn c96_init(&mut self, e: &mut Enemy) {
        e.b.live = 2;
        e.b.sub = 0;
        e.m.sb(super::off::ATTACKING, 0xFF);
        e.b.draw_flags = 0x10;
        e.b.anim_table = TABLE;
        e.b.set_xi(self.cam.s2.cam_x as i16);
        e.b.set_xi(e.b.xi().wrapping_sub(0x10));
        e.b.set_yi(0);
        self.enemy_anim(&mut e.b, 0);
        if (e.b.var26 >> 8) != 0 {
            self.enemy_anim(&mut e.b, 1);
        }
        e.b.dir = 0x18;
        e.b.facing = 1;
        e.b.mode += 2;
    }

    /// 0x69D06: the sub state routine of the table 0x69D16 (the offset of the entry is in d1: the first sound of the object has it as its
    /// second argument).
    fn c96_sub(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => {
                self.enemy_set_speed(&mut e.b, 0x17); // 0x69D1E: moveq #$17,d0; jsr 0x17A4
                e.b.set_zi(0x50);
                self.c96_fly_a(e);
                if (e.b.var26 >> 8) != 0 {
                    self.c96_fly_b(e);
                }
                e.b.sub += 2;
                e.m.sw(T_WAIT, 0x3C);
            }
            2 => {
                let t = e.m.w(T_WAIT).wrapping_sub(1);
                e.m.sw(T_WAIT, t);
                if t == 0 {
                    e.b.sub += 2; // 0x69D58
                }
            }
            4 => {
                let t = e.m.w(T_FLIGHT).wrapping_sub(1);
                e.m.sw(T_FLIGHT, t);
                if t == 0 {
                    e.b.sub += 2; // 0x69DF2
                    return;
                }
                e.b.motion(); // 0x189C
                self.enemy_step(&mut e.b);
                self.enemy_draw_check(&mut e.b, h);
            }
            6 => {
                e.b.mode += 2; // 0x69D60
                e.m.sw(T_FLIGHT, 0x3C);
            }
            _ => {}
        }
    }

    /// 0x69D6E: the velocities of the variant 0 and the sound 0x10 (`moveq #$10,d0` replaces the $56 loaded before it; d1 is the
    /// offset 8 of the jump table entry, d2 0).
    fn c96_fly_a(&mut self, e: &mut Enemy) {
        e.b.vx = 0x480;
        e.b.ax = 0;
        e.b.vy = 0x440;
        e.b.ay = -0x18;
        e.b.vz = 0;
        e.b.az = 0;
        e.m.sw(T_FLIGHT, 0x50);
        self.snd_request(0x10, 8, 0);
    }

    /// 0x69DA8: the velocities of the variant 0x100 and the break sound (0x9E60, the pan of its X).
    fn c96_fly_b(&mut self, e: &mut Enemy) {
        e.b.vx = 0x580;
        e.b.ax = 0;
        e.b.vy = 0x440;
        e.b.ay = -0x18;
        e.b.vz = 0;
        e.b.az = 0;
        e.m.sw(T_FLIGHT, 0x64);
        let x = e.b.xi();
        self.snd_break(x);
    }
}
