//! Pool A scene actors of the opening scenes (M3-20): the blue Cadillac (class 16, 0x420EA, Episodes 2 and 4),
//! the three crouching men of the Episode 4 opening (class 27, 0x4D900) and the orange haired leader with his
//! banners (class 43, 0x55F2A). The wheels of the car are class 17 (`class17.rs`, 0x423A4), which the car creates.
//! Owner: enemy port. See `re/specs/enemy/SPEC.md` and `plan/game/design/D3a-humans.md` section 11.
//!
//! Mode numbers are the original's (`+$04` = 0, 2, 4, 6). The sub state (`+$05`) picks the step of the mode.

use super::{Enemy, off::*};
use crate::game::body::Body;
use crate::game::{Game, Handle, slot_of};

/// Rider rows (0x42350): G.$504 * 3 longs, each a player handle as a 24-bit pointer (`0x00FF0000 | handle`).
const RIDER_ROWS: u32 = 0x42350;
/// Rider offsets (0x42374): dX, dY, dZ words for each hero that is aboard, in order.
const RIDER_OFFS: u32 = 0x42374;
/// Animation table of the car (0x42386).
const CAR_ANIM: u32 = 0x42386;
/// Variant table of the crouching men (0x4D992): name id bytes, +$84 words, animation table longs.
const MAN_VARIANTS: u32 = 0x4D992;
/// Hit box table of the body (the common table of the enemy code).
const BOX_TABLE: u32 = 0x103000;
/// Class 17 * 4 (the wheels) and class 66 * 4 (the banners of pool B).
const WHEEL_K20: u16 = 0x44;
const BANNER_CLASS: u16 = 66;
/// +$AC of a wheel: the car (a long, read by class 17).
const WHEEL_OWNER: usize = 0xAC;
/// Sound requests of the car: 0xA49E (0x80) and 0xA4BE (0x8F).
const SND_CAR_START: u16 = 0x80;
const SND_CAR_BRAKE: u16 = 0x8F;

impl Game {
    // ------------------------------------------------------------------ pool A class 16, the Cadillac

    /// 0x420EA pool A class 16 (jump table 0x420F6: mode 0 -> 0x420FE, 2 -> 0x421B8, 4 and 6 -> 0x42302 free).
    pub fn enemy_class16(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => self.car_init(e, h),
            2 => self.car_drive(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x48FC
            }
        }
    }

    /// 0x420FE: waits while G.$4E2 is set. Then it becomes mode 2, takes the animation 0 and makes its two
    /// wheels (class 17, variant 0 and 1, pool A allocations 0x483C; a full pool skips one). The riders are placed.
    fn car_init(&mut self, e: &mut Enemy, h: Handle) {
        if self.flow.b4e2 != 0 {
            return;
        }
        e.b.mode = 2;
        e.m.sb(ATTACKING, 0xFF);
        e.b.b63 = 0xFF;
        e.b.who = 0xFF;
        e.b.visible = 1;
        e.b.anim_table = CAR_ANIM;
        e.b.box_table = BOX_TABLE;
        e.b.multi_box = false;
        self.enemy_floor_collision(e);
        self.enemy_anim(&mut e.b, 0);
        for var in 0..2u16 {
            let Some(j) = self.enemies.alloc() else { continue };
            let (x, y, z, dir) = (e.b.xi(), e.b.yi(), e.b.zi(), e.b.dir);
            let w = &mut self.enemies.slots[j];
            w.b.live = 1;
            w.b.k20 = WHEEL_K20;
            w.b.set_xi(x);
            w.b.set_yi(y);
            w.b.set_zi(z);
            w.b.var26 = var;
            w.b.dir = dir;
            w.m.sl(WHEEL_OWNER, h as i16 as i32 as u32); // move.l a6: the register holds the handle sign-extended (0xFFFFxxxx in MAME)
        }
        self.car_place_riders(&e.b);
    }

    /// Mode 2: the step of the sub state, then the floor, the draw, the hit lists while the scene script runs
    /// (G.$4DD), and the far check (0x173C).
    fn car_drive(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => self.car_start(e),
            2 => self.car_approach(e),
            4 => self.car_stop(e),
            _ => {} // sub 6: 0x422B8 is an rts
        }
        self.enemy_floor_collision(e); // 0x12CB4
        if e.b.visible != 0 {
            self.draw_list.push(((e.b.zi() as u16 as u32) << 16) | h as u32); // 0x49C0
        }
        if self.w.scripted != 0 {
            self.combat.push(0x6B30, h); // 0x4BB0
            self.combat.push(0x6AC2, h); // 0x4B58
        }
        self.enemy_despawn_if_far(e, 0); // 0x173C
    }

    /// Sub 0 (0x421F2): the drive in at speed 0x500 (8.8), no acceleration, the attack id 0x3E and the
    /// attack serial of the original (G.$67BE). Sound 0x80 at the start.
    fn car_start(&mut self, e: &mut Enemy) {
        e.b.sub = 2;
        e.b.vx = 0x500;
        e.b.ax = 0;
        e.b.vy = 0;
        e.b.vz = 0;
        e.b.ay = 0;
        e.b.az = 0;
        e.b.atk = 0x3E;
        e.b.knock = 0;
        e.b.hitstop = 0;
        e.b.serial = self.w.attack_serial;
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
        self.sound_ex(SND_CAR_START, 0x0007_2000, 0x0400_0000); // 0xA49E: d1 = 0x72000, d2 = 0x4000000
    }

    /// Sub 2 (0x42230): drives until the camera X + 0xA0 reaches the car's X. The Episode 2 car (variant 0 in
    /// its high byte) brakes (ax -0x60, sound 0x8F, animation 1). The Episode 4 car (variant 0x200) does not
    /// brake: it sets G.$505 (the heroes may jump out). Then the motion and the riders.
    fn car_approach(&mut self, e: &mut Enemy) {
        let ahead = (self.cam.s2.cam_x as i16).wrapping_add(0xA0) > e.b.xi();
        if !ahead {
            e.b.sub = 4;
            if e.b.var26 >> 8 == 0 {
                self.sound_ex(SND_CAR_BRAKE, 0x10, 0); // 0xA4BE: `moveq #$10,d1 ; moveq #0,d2`
                e.b.ax = -0x60;
                self.enemy_anim(&mut e.b, 1);
            } else {
                self.spawn.g.b505 = 1;
            }
        }
        e.b.motion();
        self.car_place_riders(&e.b);
    }

    /// Sub 4 (0x42280): the Episode 2 car stops when its speed (+0x60) is under 0xC0: G.$505 = 1 and the
    /// stop pose (animation 0). The Episode 4 car keeps going and places no riders.
    fn car_stop(&mut self, e: &mut Enemy) {
        let episode2 = e.b.var26 >> 8 == 0;
        if episode2 && (e.b.vx as u16).wrapping_add(0x60) < 0xC0 {
            e.b.sub = 6;
            self.spawn.g.b505 = 1;
            self.enemy_anim(&mut e.b, 0);
        }
        e.b.motion();
        if episode2 {
            self.car_place_riders(&e.b);
        }
    }

    /// 0x42308: the heroes aboard are put on the car. The row (0x42350) is picked by G.$504 (3 longs a row, read
    /// as the original does). A hero takes the next offset triple only when he is alive (`+$00` not 0).
    fn car_place_riders(&mut self, car: &Body) {
        let assets = self.assets;
        let t = &assets.enemy.t;
        let n = self.spawn.g.b504 as u32;
        let mut k = 0; // byte offset of the next triple in 0x42374
        for r in 0..3 {
            let hd = t.u32(RIDER_ROWS, 3 * n + 4 * r) as Handle;
            if slot_of(hd).is_none() || self.body(hd).live == 0 {
                continue;
            }
            let (dx, dy, dz) = (t.i16(RIDER_OFFS, k), t.i16(RIDER_OFFS, k + 2), t.i16(RIDER_OFFS, k + 4));
            k += 6;
            let b = self.body_mut(hd);
            b.set_xi(car.xi().wrapping_add(dx));
            b.set_yi(car.yi().wrapping_add(dy));
            b.set_zi(car.zi().wrapping_add(dz));
        }
    }

    // ------------------------------------------------------------------ pool A class 27, the crouching men

    /// 0x4D900 pool A class 27 (jump table 0x4D90C: mode 0 -> 0x4D914, 2 -> 0x4D9AA, 4 -> 0x4D9EA, 6 -> 0x4DA24 free).
    pub fn enemy_class27(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => self.man_init(e),
            2 => self.man_alive(e, h),
            4 => self.man_blink(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x48FC
            }
        }
    }

    /// 0x4D914: the variant (0x4D972), the crouch animation, HP 1, the stun meter 10, not counted by the census,
    /// the floor, then mode 2.
    fn man_init(&mut self, e: &mut Enemy) {
        self.man_variant(e);
        self.enemy_anim(&mut e.b, 0);
        e.b.box_table = BOX_TABLE;
        e.b.multi_box = false;
        e.b.hp_bar = 0x14;
        e.b.hp = 1;
        e.b.hp_prev = 1;
        e.b.stun = 0x0A;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.m.sb(ATTACKING, 0);
        e.m.sb(B87, 0);
        e.b.b72 = 0xC8;
        self.enemy_floor_collision(e);
        e.m.sb(ATTACKING, 0xFF);
        e.b.mode = 2;
    }

    /// 0x4D972: the name id (`+$03`), the +$84 word (name * 4) and the animation table of the variant (high byte of +$26).
    fn man_variant(&mut self, e: &mut Enemy) {
        let assets = self.assets;
        let t = &assets.enemy.t;
        let v = (e.b.var26 >> 8) as u32;
        e.b.id = t.u8(MAN_VARIANTS, v >> 1);
        e.m.sw(W84, t.u16(MAN_VARIANTS + 4, v));
        e.b.anim_table = t.u32(MAN_VARIANTS, 2 * v + 0xC);
    }

    /// Mode 2 (0x4D9AA): while G.$4E2 is set, or sub 0, the animation step, the hurt list and the draw. Sub 2 is the
    /// shared hit reaction (0x2A606) after the HP is cleared. Sub 4 never runs (its table entry is 0).
    fn man_alive(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.sub == 2 && self.flow.b4e2 == 0 {
            e.b.hp = 0; // clr.w $6c
            self.enemy_hit_reaction(e, h);
        } else {
            self.man_wait(e, h);
        }
    }

    /// 0x4D9CE: the animation step, the hurt list (0x4BEA) and the draw (0x49CA).
    fn man_wait(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_step(&mut e.b);
        self.enemy_push_hurtable(h);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// Mode 4 (0x4D9EA): the blink after the hit. Sub 0 sets the timer 0x78. Each frame the timer counts down, the
    /// sprite shows on the odd counts, and at 0 the object goes to mode 6 (freed next frame).
    fn man_blink(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.sub == 0 {
            e.m.sw(T80, 0x78);
            e.b.sub = 2;
        }
        let t = e.m.w(T80);
        if t == 0 {
            e.b.mode += 2;
            return;
        }
        e.m.sw(T80, t - 1);
        if (t - 1) & 1 != 0 {
            self.enemy_draw_check(&mut e.b, h);
        }
    }

    // ------------------------------------------------------------------ pool A class 43, the orange leader

    /// 0x55F2A pool A class 43 (jump table 0x55F36: mode 0 -> 0x55F3E, 2 -> 0x55F6C, 4 and 6 -> 0x56100 free).
    pub fn enemy_class43(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                self.spawn.g.b4fc = 1; // 0x55F3E: move.b #1,$4FC(a5), the opening's flag (the banner wait of 0x55FA0 is the gate)
                e.b.live = 2;
                e.b.anim_table = 0x56106;
                e.m.sw(WA0, 0);
                e.m.sb(0xA3, 0);
                self.enemy_floor_collision(e);
                self.enemy_anim(&mut e.b, 3);
                e.b.mode = 2;
                self.leader_body(e, h);
            }
            2 => self.leader_body(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x48FC
            }
        }
    }

    /// 0x55F6C: the sub state step, then the floor, the draw and the far check.
    fn leader_body(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => self.leader_wait_gate(e),
            2 => self.leader_banner(e, 0x39),
            4 => self.leader_car(e),
            6 => self.leader_jump(e),
            8 => self.leader_air(e),
            0xA => self.leader_pose(e),
            0xC => self.leader_banner_flag(e),
            0xE => self.leader_timer_end(e),
            0x10 => self.leader_leave(e),
            0x12 => {
                if self.enemy_step(&mut e.b) {
                    e.b.mode += 2;
                    self.spawn.g.b4fc = 0;
                }
            }
            _ => {}
        }
        self.enemy_floor_collision(e);
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_despawn_if_far(e, 0);
    }

    /// Sub 0 (0x55FA0): waits while G.$4E2 is set, then the first banner (variant 0x38, pool B), 0xA0 frames.
    fn leader_wait_gate(&mut self, e: &mut Enemy) {
        if self.flow.b4e2 != 0 {
            return;
        }
        if self.spawn_prop(BANNER_CLASS, 0x38, None).is_some() {
            e.b.sub = 2;
            e.m.sw(T80, 0xA0);
        }
    }

    /// Sub 2 (0x55FCC): count `+$80` down; at the end the banner `var` (0x39, pool B; a full pool retries next
    /// frame), then sub 4. The animation step runs every frame.
    fn leader_banner(&mut self, e: &mut Enemy, var: u16) {
        let (t, borrow) = e.m.w(T80).overflowing_sub(1);
        e.m.sw(T80, t);
        if borrow && self.spawn_prop(BANNER_CLASS, var, None).is_some() {
            e.b.sub = 4;
            e.m.sw(T80, 0xA0);
        }
        self.enemy_step(&mut e.b);
    }

    /// Sub 4 (0x55FFC): count down; at the end the animation 0, the Episode 4 car (0x420BE) and 0x32 frames.
    fn leader_car(&mut self, e: &mut Enemy) {
        let (t, borrow) = e.m.w(T80).overflowing_sub(1);
        e.m.sw(T80, t);
        if borrow {
            e.b.sub = 6;
            self.enemy_anim(&mut e.b, 0);
            self.scene_spawn_car();
            e.m.sw(T80, 0x32);
        }
        self.enemy_step(&mut e.b);
    }

    /// Sub 6 (0x56020): count down; at the end the jump (animation 1, airborne, vy 0xA80, ay -0x50).
    fn leader_jump(&mut self, e: &mut Enemy) {
        let (t, borrow) = e.m.w(T80).overflowing_sub(1);
        e.m.sw(T80, t);
        if borrow {
            e.b.sub = 8;
            self.enemy_anim(&mut e.b, 1);
            e.b.airborne = 1;
            e.b.vy = 0xA80u16 as i16;
            e.b.ay = 0xFFB0u16 as i16;
        }
    }

    /// Sub 8 (0x56046): in the air the motion (unless the frame flag +$2D is set) and the animation; on the floor
    /// the pose (animation 2) and sub 0x0A.
    fn leader_air(&mut self, e: &mut Enemy) {
        if e.b.airborne != 0 {
            if e.b.param_b == 0 {
                e.b.motion();
            }
            self.enemy_step(&mut e.b);
        } else {
            e.b.sub = 0xA;
            self.enemy_anim(&mut e.b, 2);
            self.leader_pose(e);
        }
    }

    /// Sub 0x0A (0x5606A): the pose; when it ends, sub 0x0C.
    fn leader_pose(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            e.b.sub = 0xC;
        }
    }

    /// Sub 0x0C (0x56078): the banner 0x3A (pool B), 0x78 frames.
    fn leader_banner_flag(&mut self, e: &mut Enemy) {
        if self.spawn_prop(BANNER_CLASS, 0x3A, None).is_some() {
            e.b.sub = 0xE;
            e.m.sw(T80, 0x78);
        }
    }

    /// Sub 0x0E (0x5609E): count down; at the end the second jump (animation 1, airborne, vy 0x800, ay -0x60, vx 0x300).
    fn leader_timer_end(&mut self, e: &mut Enemy) {
        let (t, borrow) = e.m.w(T80).overflowing_sub(1);
        e.m.sw(T80, t);
        if borrow {
            e.b.sub = 0x10;
            self.enemy_anim(&mut e.b, 1);
            e.b.airborne = 1;
            e.b.vy = 0x800;
            e.b.ay = 0xFFA0u16 as i16;
            e.b.vx = 0x300;
        }
    }

    /// Sub 0x10 (0x560CA): in the air the motion and the animation; on the floor the landing pose (animation 2),
    /// and when that ends the mode goes to 4 (freed next frame) and G.$4FC (the opening end flag) is cleared.
    fn leader_leave(&mut self, e: &mut Enemy) {
        if e.b.airborne != 0 {
            if e.b.param_b == 0 {
                e.b.motion();
            }
            self.enemy_step(&mut e.b);
        } else {
            e.b.sub = 0x12;
            self.enemy_anim(&mut e.b, 2);
            if self.enemy_step(&mut e.b) {
                e.b.mode += 2;
                self.spawn.g.b4fc = 0;
            }
        }
    }

    /// 0x420BE: the Episode 4 car drives in from X -128 (variant 0x200, Z 0x30); the car's own mode 0 makes its wheels.
    fn scene_spawn_car(&mut self) {
        let Some(j) = self.enemies.alloc() else { return };
        let b = &mut self.enemies.slots[j].b;
        b.live = 1;
        b.k20 = 0x40;
        b.var26 = 0x200;
        b.set_xi(-128);
        b.set_yi(0);
        b.set_zi(0x30);
    }
}
