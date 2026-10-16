//! Pool A class 81 (`+$20 = 0x144`, handler 0x60158): the walkers and the Cadillac of the desert scene of the ending (M3-25 / M3-26). The desert
//! master (pool B 81, `prop/b81.rs`) makes the two walkers (variants 1 and 2: the heroes who stay, Mess and Mustapha) at the start and the car
//! (variant 0) 2980 frames later. The animation table is 0x604EA.
//!
//! | mode | what |
//! | --- | --- |
//! | 0 | 0x6016C: live 2, census mark, no shadow, the table, X = camera X + 0xC0 (the car 0x1C0), animation 0, direction 0x18; the car makes its two heroes (pool A 17, variants 0 and 1, `+$AC` = the car) and faces right; a walker faces left (the second one stands 0x18 further right, animation 3) |
//! | 2 | 0x60268: X follows the camera (`+$A8` = the camera X of the frame before, a long), the sub state, then the draw check |
//!
//! Sub states: 0 (0x60296) the walking speeds and the timer `+$B2` (the car 0x3CA, a walker 0xF6E frames), then sub 2 at once; 2 (0x602E6) the script of
//! the variant (the car: stop, turn, brake; Mustapha: 0x60364; Mess: 0x603E2): the speed table changes at timer values; 4 flight (not used by the
//! ending); 6 nothing (the walkers stand, the car is parked at the end of its timer).

use super::{Enemy, off::ATTACKING};
use crate::game::{Game, Handle};

const TABLE: u32 = 0x604EA;
/// `+$B2` the timer (word), `+$BC` the stop flag (byte, set by nothing in the ending), `+$BD` the standing flag of the car, `+$A8` the camera X of the
/// frame before (a long), `+$B8` (a long, set and never read).
const TIMER: usize = 0xB2;
const STOP: usize = 0xBC;
const STAND: usize = 0xBD;
const CAM_PREV: usize = 0xA8;

impl Game {
    /// 0x60158 class 81 (jump table 0x60164: mode 0 -> 0x6016C, 2 -> 0x60268, 4 and 6 -> 0x604E4).
    pub fn enemy_class81(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                self.c81_init(e, h);
                self.c81_frame(e, h);
            }
            2 => self.c81_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x604E4: jmp 0x48FC
            }
        }
    }

    /// 0x6016C.
    fn c81_init(&mut self, e: &mut Enemy, h: Handle) {
        e.b.live = 2;
        e.b.sub = 0;
        e.m.sb(ATTACKING, 0xFF);
        e.b.draw_flags = 0x10;
        e.b.anim_table = TABLE;
        e.m.sb(STOP, 0);
        e.b.set_xi(self.cam.s2.cam_x as i16);
        e.b.set_xi(e.b.xi().wrapping_add(0xC0));
        e.b.set_yi(0);
        let var = e.b.var26;
        if var == 0 {
            e.b.set_xi(0x1C0);
        }
        self.enemy_anim(&mut e.b, 0);
        e.b.dir = 0x18;
        if var == 0 {
            // 0x601F2: the two heroes of the car
            for v in 0..2u16 {
                if let Some(j) = self.enemies.alloc() {
                    let (x, y, z, dir) = (e.b.xi(), e.b.yi(), e.b.zi(), e.b.dir);
                    let w = &mut self.enemies.slots[j];
                    w.b.live = 1;
                    w.b.k20 = 0x44;
                    w.b.set_xi(x);
                    w.b.set_yi(y);
                    w.b.set_zi(z);
                    w.b.var26 = v;
                    w.b.dir = dir;
                    w.m.sl(0xAC, h as i16 as i32 as u32); // move.l a6,$AC(a0)
                }
            }
            e.b.facing = 1;
        } else {
            self.enemy_anim(&mut e.b, 1);
            e.b.facing = 0;
            if var == 2 {
                self.enemy_anim(&mut e.b, 3);
                e.b.set_xi(e.b.xi().wrapping_add(0x18));
            }
        }
        e.b.mode += 2;
    }

    /// 0x60268.
    fn c81_frame(&mut self, e: &mut Enemy, h: Handle) {
        let cam = ((self.cam.s2.cam_x as u32) << 16) | self.cam.s2.cam_x_frac as u32;
        let d = e.m.l(CAM_PREV).wrapping_sub(cam);
        e.b.x = e.b.x.wrapping_sub(d as i32);
        match e.b.sub {
            0 => self.c81_start(e),
            2 => self.c81_script(e),
            4 => self.c81_flight(e),
            _ => {} // 0x60460: rts
        }
        e.m.sl(CAM_PREV, cam);
        self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
    }

    /// 0x60296: the speeds and the timer, then the script of sub 2 at once.
    fn c81_start(&mut self, e: &mut Enemy) {
        e.m.sb(STAND, 0);
        self.enemy_set_speed(&mut e.b, 7);
        e.b.set_zi(0x50);
        e.m.sw(TIMER, 0x244);
        if e.b.var26 != 0 {
            self.enemy_set_speed(&mut e.b, 3);
            e.b.set_zi(0x20);
            e.m.sw(TIMER, 0xDE8);
        }
        e.m.sw(TIMER, e.m.w(TIMER).wrapping_add(0x64));
        e.m.sw(TIMER, e.m.w(TIMER).wrapping_add(0x122));
        e.m.sl(0xB8, 0x8000);
        e.b.sub += 2;
        self.c81_script(e);
    }

    /// 0x602E6: the script of the variant.
    fn c81_script(&mut self, e: &mut Enemy) {
        match e.b.var26 {
            2 => self.c81_mess(e),
            0 => self.c81_car(e),
            _ => self.c81_mustapha(e),
        }
    }

    /// 0x602F8: the car.
    fn c81_car(&mut self, e: &mut Enemy) {
        let t = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, t);
        if t == 0 {
            e.b.sub = 6; // 0x6035C
            return;
        }
        if t == 0x352 {
            self.enemy_set_speed(&mut e.b, 0);
            e.m.sb(STAND, 1);
        }
        if t == 0x8C {
            self.enemy_set_speed(&mut e.b, 2);
            e.m.sb(STAND, 0);
        }
        if t == 0x50 {
            self.enemy_set_speed(&mut e.b, 1);
            e.b.dir = 8;
        }
        if e.m.b(STAND) == 0 {
            self.enemy_move_dir(&mut e.b); // jsr 0x1862
        }
        self.enemy_step(&mut e.b); // jmp 0x121E
    }

    /// 0x60364: the walker of the variant 1.
    fn c81_mustapha(&mut self, e: &mut Enemy) {
        if e.m.b(STOP) == 0 {
            e.b.facing = 0;
            let t = e.m.w(TIMER).wrapping_sub(1);
            e.m.sw(TIMER, t);
            if t == 0 {
                e.b.sub += 4; // 0x603DA
                return;
            }
            if t == 0x37A {
                self.enemy_set_speed(&mut e.b, 0);
            }
            if t == 0x8C {
                e.b.dir = 8;
                self.enemy_set_speed(&mut e.b, 1);
            }
            if t == 0x78 {
                e.b.dir = 0x18;
                self.enemy_set_speed(&mut e.b, 0);
                self.enemy_anim(&mut e.b, 2);
            }
            if (t as i16) < 0x8C {
                self.enemy_move_dir(&mut e.b);
            }
        }
        self.enemy_step(&mut e.b); // 0x603D4: jmp 0x121E
    }

    /// 0x603E2: the walker of the variant 2.
    fn c81_mess(&mut self, e: &mut Enemy) {
        if e.m.b(STOP) == 0 {
            e.b.facing = 0;
            let t = e.m.w(TIMER).wrapping_sub(1);
            e.m.sw(TIMER, t);
            if t == 0 {
                e.b.sub += 4; // 0x60458
                return;
            }
            if t == 0x37A {
                self.enemy_set_speed(&mut e.b, 0);
            }
            if t == 0x8C {
                e.b.dir = 8;
                self.enemy_set_speed(&mut e.b, 2);
            }
            if t == 0x6E {
                e.b.dir = 0x18;
                self.enemy_set_speed(&mut e.b, 0);
                self.enemy_anim(&mut e.b, 4);
            }
            if (t as i16) < 0x8C {
                self.enemy_move_dir(&mut e.b);
            }
        }
        self.enemy_step(&mut e.b); // 0x60452: jmp 0x121E
    }

    /// 0x604C8: sub 4: `+$B2` frames of motion (0x189C) and the animation, then sub 6 (nothing in the ending reaches it).
    fn c81_flight(&mut self, e: &mut Enemy) {
        let t = e.m.w(TIMER).wrapping_sub(1);
        e.m.sw(TIMER, t);
        if t == 0 {
            e.b.sub += 2;
            return;
        }
        e.b.motion();
        self.enemy_step(&mut e.b);
    }
}
