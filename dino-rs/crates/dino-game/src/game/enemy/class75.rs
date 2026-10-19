//! Pool A class 75 (handler 0x5E702..0x5EC26): the actors of the Tyrog scene and of the host (class 72), table 0x71A4E. By variant (`+$26`):
//! 0 the steel crate (320 frames, then it shakes, throws sparks and breaks into the creature, class 76), 1 and 4 the guards (variant 4 becomes a
//! thug, class 2 variant 0x200, after 500 frames; variant 1 just goes), 2 a long idle actor, 3 / 0x13 a walking actor, 5 / 0x12 a flash puff
//! (blinks out), 6 a puff, 7 a spark, 8 / 0x10 / 0x11 the host's landing marks, 9 a flash. D3d 3.1, 3.2, 3.7. Owner: enemy port.

use super::{off::*, Enemy};
use crate::game::{slot_of, Game, Handle, Slot};

/// Own bytes (the original offsets): `+$B2` word the timer, `+$B4` / `+$B8` longs the speeds of the spark (variant 7), `+$BC` long the object
/// a landing mark follows in depth (variant 8).
mod o {
    pub const TIMER: usize = 0xB2;
    pub const VX: usize = 0xB4;
    pub const VY: usize = 0xB8;
    pub const FOLLOW: usize = 0xBC;
}

impl Game {
    /// 0x5E702 (mode table 0x5E70E): 0 the init 0x5E716 (falls into 2), 2 the run 0x5E778, 4 the blink out 0x5EC04, 6 the free 0x5EC20.
    pub(super) fn enemy_class75(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.c75_init(e, h);
                self.c75_run(e, h, i);
            }
            2 => self.c75_run(e, h, i),
            4 => {
                // 0x5EC04: `+$B2` frames, drawn when the count is even, then the free.
                let t = e.m.w(o::TIMER).wrapping_sub(1);
                e.m.sw(o::TIMER, t);
                if t == 0 {
                    return self.enemy_free_pool_a(e, i);
                }
                if t & 1 == 0 {
                    self.enemy_draw_check(&mut e.b, h);
                }
            }
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x5E716: `+$00` = 2, the table 0x71A4E, not counted (`+$7D` 0xFF, but 0 for the variants 5 and 8), `+$63` 0xFF, no shadow, the terrain
    /// (0x12CB4), animation 0.
    fn c75_init(&mut self, e: &mut Enemy, h: Handle) {
        e.b.live = 2;
        e.b.sub = 0;
        e.b.anim_table = 0x71A4E;
        e.m.sb(ATTACKING, 0xFF);
        e.b.b63 = 0xFF;
        if e.b.var26 == 5 || e.b.var26 == 8 {
            e.m.sb(ATTACKING, 0);
        }
        e.b.draw_flags = 0x10;
        e.m.sw(0xA0, 0);
        e.m.sw(0xA6, 0x10);
        if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
            e.m.sw(OBSTACLE, ob);
        }
        self.enemy_anim(&mut e.b, 0);
        e.b.mode = e.b.mode.wrapping_add(2);
    }

    /// 0x5E778: by `+$05` (table 0x5E792): 0 the setup 0x5E798 (falls into 2), 2 the run by variant 0x5E976, 4 the blink start 0x5EBF6; then
    /// the far test (0x173C) and the draw check (0x49CA).
    fn c75_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.sub {
            0 => {
                if self.c75_setup(e) {
                    self.c75_by_variant(e, h);
                }
            }
            2 => self.c75_by_variant(e, h),
            _ => {
                // 0x5EBF6
                e.b.mode = e.b.mode.wrapping_add(2);
                e.m.sw(o::TIMER, 0x78);
            }
        }
        self.enemy_despawn_if_far(e, i); // 0x173C
        self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
    }

    /// 0x5E798: `+$05` 2 and the timer, animation, speed by variant. Returns false for the variants that go to the blink out (5, 0x12: mode 4).
    fn c75_setup(&mut self, e: &mut Enemy) -> bool {
        e.b.sub = e.b.sub.wrapping_add(2);
        e.m.sw(o::TIMER, 0x140);
        if e.b.var26 != 0 {
            e.m.sw(o::TIMER, 0x172);
            self.enemy_anim(&mut e.b, 1);
            if e.b.var26 == 2 {
                e.m.sw(o::TIMER, 0x320);
            }
            if e.b.var26 == 3 {
                e.m.sw(o::TIMER, 0x22);
                self.enemy_anim(&mut e.b, 2);
                self.enemy_set_speed(&mut e.b, 3);
            }
            if e.b.var26 == 4 {
                e.m.sw(o::TIMER, 0x1F4);
                self.enemy_anim(&mut e.b, 3);
            }
        }
        if e.b.var26 == 5 {
            self.enemy_anim(&mut e.b, 4);
            e.b.mode = 4;
            e.m.sw(o::TIMER, 0x32);
            return false;
        }
        if e.b.var26 == 6 {
            e.m.sw(o::TIMER, 0x18);
            self.enemy_anim(&mut e.b, 6);
        }
        if e.b.var26 == 7 {
            self.enemy_set_speed(&mut e.b, 0x22);
            e.m.sl(o::VX, 0);
            e.m.sl(o::VY, 0x40000);
            if (self.rand() & 0xF) < 6 {
                e.m.sl(o::VX, 0x20000);
                self.enemy_set_speed(&mut e.b, 0x1E);
                e.m.sl(o::VY, 0x30000);
            }
            e.m.sw(o::TIMER, 0x1D);
            e.b.dir = 0;
            self.enemy_anim(&mut e.b, 2);
        }
        if e.b.var26 == 8 {
            e.m.sw(o::TIMER, 0x3A);
            self.enemy_anim(&mut e.b, 7);
            self.enemy_set_speed(&mut e.b, 7);
        }
        if e.b.var26 == 9 {
            e.m.sw(o::TIMER, 0x32);
            self.enemy_anim(&mut e.b, 8);
            e.b.palette_override = 0x8A;
        }
        if e.b.var26 == 0x10 {
            e.m.sw(o::TIMER, 0x3A);
            self.enemy_anim(&mut e.b, 9);
            self.enemy_set_speed(&mut e.b, 7);
            e.b.var26 = 8;
        }
        if e.b.var26 == 0x11 {
            e.m.sw(o::TIMER, 0x3A);
            self.enemy_anim(&mut e.b, 0xA);
            self.enemy_set_speed(&mut e.b, 7);
            e.b.palette_override = 0x99;
            e.b.var26 = 8;
        }
        if e.b.var26 == 0x12 {
            self.enemy_anim(&mut e.b, 0xB);
            e.b.mode = 4;
            e.m.sw(o::TIMER, 0x32);
            e.b.var26 = 5;
            return false;
        }
        if e.b.var26 == 0x13 {
            e.m.sw(o::TIMER, 0x22);
            self.enemy_anim(&mut e.b, 2);
            self.enemy_set_speed(&mut e.b, 7);
            e.b.var26 = 3;
        }
        true
    }

    /// The timer `+$B2` counted down: true when it reaches 0.
    fn c75_tick(e: &mut Enemy) -> bool {
        let t = e.m.w(o::TIMER).wrapping_sub(1);
        e.m.sw(o::TIMER, t);
        t == 0
    }

    /// 0x5E976: the run by variant.
    fn c75_by_variant(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.var26 {
            3 => {
                // 0x5EB14: walk to its facing side until the time is out.
                if Self::c75_tick(e) {
                    e.b.mode = 6;
                    return;
                }
                e.b.dir = if e.b.facing != 0 { 8 } else { 0x18 };
                self.enemy_move_dir(&mut e.b);
                self.enemy_step(&mut e.b);
            }
            6 => {
                // 0x5EB3A
                if Self::c75_tick(e) || self.enemy_step(&mut e.b) {
                    e.b.mode = 6;
                }
            }
            7 => {
                // 0x5EB4A: the spark flies by its own 16.16 speeds, turned back between camera X + 0x110 and + 0x140.
                if Self::c75_tick(e) {
                    e.b.mode = 6;
                    return;
                }
                e.b.x = e.b.x.wrapping_add(e.m.l(o::VX) as i32);
                let d0 = e.b.xi().wrapping_sub(self.cam.s2.cam_x as i16);
                if d0 < 0x110 {
                    e.m.sl(o::VX, 0x20000);
                }
                if d0 > 0x140 {
                    e.m.sl(o::VX, 0xFFFE_0000);
                }
                e.b.y = e.b.y.wrapping_add(e.m.l(o::VY) as i32);
                if self.enemy_step(&mut e.b) {
                    e.b.mode = 6;
                }
            }
            8 => self.c75_mark(e, h),
            9 => {
                // 0x5EB94
                if self.enemy_step(&mut e.b) {
                    e.b.mode = 6;
                }
            }
            0 => self.c75_crate(e),
            _ => {
                // 0x5EAC6: the guards: at the end of the time variant 4 becomes a thug (class 2 variant 0x200), variant 1 just goes.
                if !Self::c75_tick(e) {
                    self.enemy_step(&mut e.b);
                    return;
                }
                if e.b.var26 != 1 {
                    let (x, y, z) = (e.b.xi(), e.b.yi(), e.b.zi());
                    self.c75_spawn(2 * 4, 0x200, x, y, z, None);
                }
                e.b.mode = 6;
            }
        }
    }

    /// 0x5EBA0: a landing mark: at the end of the time the blink out (1 frame); else it follows the depth of its object `+$BC` by 0.5 px per frame
    /// and, while the time is above 0x14, moves (0x1862) and animates; then a draw check of its own (the run's tail draws it again).
    fn c75_mark(&mut self, e: &mut Enemy, h: Handle) {
        if Self::c75_tick(e) {
            e.b.mode = 4;
            e.m.sw(o::TIMER, 1);
            return;
        }
        let f = e.m.l(o::FOLLOW) as u16;
        let fz = match slot_of(f) {
            Some(Slot::Enemy(_) | Slot::Player(_) | Slot::Prop(_) | Slot::Item(_)) => self.body(f).zi(),
            _ => 0,
        };
        if fz > e.b.zi() {
            e.b.z = e.b.z.wrapping_add(0x8000);
        } else {
            e.b.z = e.b.z.wrapping_sub(0x8000);
        }
        if (e.m.w(o::TIMER) as i16) > 0x14 {
            self.enemy_move_dir(&mut e.b);
            self.enemy_step(&mut e.b);
        }
        self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
    }

    /// 0x5E9B0: the crate. At the end of its time it breaks (0x5EA40: the creature, class 76, at its place, 1 nearer; `+$05` 4). From the time
    /// 0x63 down it throws a spark every 4 frames (0x5EA76), one flash at 0x63 (class 75 variant 9, 0x10 higher, 2 deeper) and shakes (+4 / -4
    /// every 4 frames); the animation runs.
    fn c75_crate(&mut self, e: &mut Enemy) {
        if Self::c75_tick(e) {
            let (x, y, z) = (e.b.xi(), e.b.yi(), e.b.zi());
            self.c75_spawn(76 * 4, 0, x, y, z.wrapping_sub(1), None);
            e.b.sub = e.b.sub.wrapping_add(2);
            return;
        }
        let t = e.m.w(o::TIMER);
        if (t as i16) < 0x64 {
            if t & 3 == 0 {
                // 0x5EA76: a spark (variant 7) 0x20 left plus `rand() & 0x30`, 0x1C higher, 2 nearer.
                if let Some(j) = self.enemies.alloc() {
                    let r = (self.rand() & 0x30) as i16;
                    let n = &mut self.enemies.slots[j];
                    n.b.live = 1;
                    n.b.k20 = 75 * 4;
                    n.b.var26 = 7;
                    let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
                    put(&mut n.b.x, e.b.xi().wrapping_sub(0x20).wrapping_add(r));
                    put(&mut n.b.z, e.b.zi().wrapping_sub(2));
                    put(&mut n.b.y, e.b.yi().wrapping_add(0x1C));
                }
            }
            if t == 0x63 {
                let (x, y, z) = (e.b.xi(), e.b.yi(), e.b.zi());
                self.c75_spawn(75 * 4, 9, x, y.wrapping_add(0x10), z.wrapping_add(2), None);
            }
            let x = e.b.xi();
            match t & 7 {
                0 => e.b.x = (e.b.x & 0xFFFF) | ((x.wrapping_add(4) as i32) << 16),
                4 => e.b.x = (e.b.x & 0xFFFF) | ((x.wrapping_sub(4) as i32) << 16),
                _ => {}
            }
        }
        self.enemy_step(&mut e.b);
    }

    /// A pool A object of the class word `k20` and variant at (x, y, z) (0x483C; nothing when the pool is full); `facing` when given.
    pub(super) fn c75_spawn(&mut self, k20: u16, var: u16, x: i16, y: i16, z: i16, facing: Option<u8>) -> Option<usize> {
        let j = self.enemies.alloc()?;
        let n = &mut self.enemies.slots[j];
        n.b.live = 1;
        n.b.k20 = k20;
        n.b.var26 = var;
        let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
        put(&mut n.b.x, x);
        put(&mut n.b.z, z);
        put(&mut n.b.y, y);
        if let Some(f) = facing {
            n.b.facing = f;
        }
        Some(j)
    }
}
