//! Hogg, the boss of Episode 3: the rider, class 31 (handler 0x4FCB4..0x50FAC), on his motorbike, class 32 (`class32.rs`). D3c section 4,
//! BS12, BS15, BS23.
//!
//! The bike is the moving body: the rider moves it (`move_dir` 0x1862 on foot, `motion` 0x189C in car mode) and then sits on it (0x5078E).
//! The rider is the object that is hit, throws, owns the HP and runs the two behaviours, chosen every frame by the car flag G.$518:
//! - on foot (M6-23): ride across the screen at the speed id 0x21 (5.5 px per frame), turning one step every 30 frames toward the
//!   target's depth, wait 120 frames off screen, come back from the other side (sub states 0 / 2 / 4 at `+$07`);
//! - in car mode (M6-24): a random walk on a 32 by 8 cell grid around the car (the zone maps 0xC821A and 0xC6252), a new cell every 2 to 40
//!   frames (0xC861A), the aim point recomputed every 25 frames; then the coast when the car is gone (sub states 0 to 8).
//!
//! The throw (0x502D0) makes the bomb, class 42 (0x55A7E), which is M6-25. The death sends the score and the leader flags (mode 4).
//! Owner: enemy port.

use super::lib32::put_hi;
use super::{off::*, Enemy};
use crate::game::{slot_of, Game, Handle, Slot};

/// Own bytes of the rider (the original offsets).
pub(super) mod o {
    /// `+$A0` word: the bike (class 32), which keeps the rider in its own `+$A0`.
    pub const BIKE: usize = 0xA0;
    /// `+$A3` byte: the side he rides from: 0 = from the left (going right), 1 = from the right (going left).
    pub const FROM_RIGHT: usize = 0xA3;
    /// `+$A4` word: the steering timer (foot, 30 frames) or the re-aim timer (car mode, 25 frames).
    pub const A4: usize = 0xA4;
    /// `+$A6` byte: the wheelie ride; `+$A7` byte: its frames left.
    pub const WHEELIE: usize = 0xA6;
    pub const WHEELIE_T: usize = 0xA7;
    /// `+$A8` byte: the grid cell the bike is in (car mode); `+$A9` byte: the aim cell.
    pub const CELL: usize = 0xA8;
    pub const AIM: usize = 0xA9;
    /// `+$AA` / `+$AC` words: cleared at every aim pick (not read).
    pub const AA: usize = 0xAA;
    pub const AC: usize = 0xAC;
    /// `+$AE` byte: the car behaviour is on (a copy of G.$518 != 0). `+$AF` byte: a throw is running.
    pub const CAR: usize = 0xAE;
    pub const THROWING: usize = 0xAF;
    /// `+$B2` byte: the throw roll timer (60 frames on foot, 30 in car mode). `+$B3` byte: throws left in this volley.
    pub const ROLL_T: usize = 0xB2;
    pub const VOLLEY: usize = 0xB3;
    /// `+$B4` word: the step of the throw (0 or 2). `+$B6` word: the HP at the start (the volley size quarters).
    pub const THROW_STEP: usize = 0xB4;
    pub const START_HP: usize = 0xB6;
    /// `+$B8` byte: the bomb variant / 2 (0 or 1 at the car, 2 or 3 at a hero to the left or right). `+$BA` byte: set by a car ram.
    pub const B8: usize = 0xB8;
    pub const BA: usize = 0xBA;
    /// `+$BB` byte: the first target number (the cycle 0x4FE46). `+$BC` / `+$BE` / `+$C0` words: the heroes in play (0x50D08).
    pub const BB: usize = 0xBB;
    pub const HEROES: usize = 0xBC;
    /// `+$C2` byte: the index of the target in the hero list. `+$C4` word: the cruise time (car mode).
    pub const C2: usize = 0xC2;
    pub const CRUISE: usize = 0xC4;
    /// `+$C6` word: the frames he is untouchable (`+$00` = 2). `+$C8` word: the 300 frame timer of the controller (G.$502).
    pub const SAFE: usize = 0xC6;
    pub const GROUP: usize = 0xC8;

    /// The tables read at run time (extracted by `dino-extract/src/boss.rs`).
    pub const FIRST_TARGET: u32 = 0x4FE46;
    pub const SEAT: u32 = 0x507D0;
    pub const ENTRY: u32 = 0x50816;
    pub const ENTRY_W: u32 = 0x5083E;
    pub const THROW_T: u32 = 0x50B0A;
    pub const TARGET_ROWS: u32 = 0x50C62;
    pub const DEATH_V: u32 = 0x504E0;
    pub const RAM_V: u32 = 0x50616;
    pub const GRID: u32 = 0xC821A;
    pub const CELLS: u32 = 0xC6252;
    pub const CRUISE_T: u32 = 0xC861A;
    pub const REAIM_T: u32 = 0xC865A;
}

/// 0x50B74: the throws of a volley by the HP left: 1 above 3/4 of the start HP, 2 down to 1/2, 3 down to 1/4, 4 at 1/4 or less
/// (signed word compares, the quarter rounded down).
pub(super) fn c31_volley(start_hp: u16, hp: u16) -> u8 {
    let q = start_hp >> 2;
    let mut d1 = 4u8;
    let mut d2 = q;
    for _ in 0..3 {
        if d2 as i16 >= hp as i16 {
            break;
        }
        d2 = d2.wrapping_add(q);
        d1 -= 1;
    }
    d1
}

/// 0x509C6: the grid cell of the rider relative to the car: the row from the depth (`(car Z - Z + 0x40) & 0xFFF0`, times 2) plus
/// the column from X (`X - car X + 0x100` clamped to 0 .. 0x1FF, over 16). Only the low byte is kept.
pub(super) fn c31_cell(car_x: i16, car_z: i16, x: i16, z: i16) -> u8 {
    let row = (car_z.wrapping_sub(z).wrapping_add(0x40) as u16 & 0xFFF0).wrapping_mul(2);
    let mut d1 = x.wrapping_sub(car_x).wrapping_add(0x100);
    if d1 < 0 {
        d1 = 0;
    }
    if d1 > 0x200 {
        d1 = 0x1FF;
    }
    row.wrapping_add((d1 as u16) >> 4) as u8
}

impl Game {
    /// 0x4FCB4: class 31 (mode table 0x4FCD4): 0 the init (0x4FCDC), 2 alive (0x4FE56), 4 the death (0x50760). After it the debug
    /// display 0x5020A (only with the debug flag G.$4AA and bit 5 of G.$23) is not ported.
    pub(super) fn enemy_class31(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => match e.b.sub {
                0 => self.c31_make_pair(e, h, i),
                _ => self.c31_init(e, h),
            },
            2 => self.c31_alive(e, h, i),
            _ => self.c31_dead(e, h),
        }
    }

    /// The bike of the rider `e`, taken out of its slot (put it back with `c31_put_bike`).
    fn c31_take_bike(&mut self, e: &Enemy) -> Option<(usize, Enemy)> {
        match slot_of(e.m.w(o::BIKE)) {
            Some(Slot::Enemy(b)) => Some((b, std::mem::take(&mut self.enemies.slots[b]))),
            _ => None,
        }
    }

    fn c31_put_bike(&mut self, bike: Option<(usize, Enemy)>) {
        if let Some((b, k)) = bike {
            self.enemies.slots[b] = k;
        }
    }

    /// The bike's body (read only).
    fn c31_bike(&self, e: &Enemy) -> crate::game::body::Body {
        self.body(e.m.w(o::BIKE)).clone()
    }

    /// 0x12CB4 for a pool A object with its obstacle into `+$94`.
    pub(super) fn c31_terrain(&mut self, e: &mut Enemy, h: Handle) {
        if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
            e.m.sw(OBSTACLE, ob);
        }
    }

    /// 0x4FCEE: sub 0 of the init: a second pool A object (0x483C; a full pool frees this one, 0x50788). The lower address of the two
    /// becomes the rider (class 31, `+$05` = 2, the variant, direction and position of this object) and the higher one the bike (class 32,
    /// position only); each keeps the other in `+$A0`. When this object is the bike, it runs the class 32 init in the next frame.
    fn c31_make_pair(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        let Some(n) = self.enemies.alloc() else {
            return self.enemy_free_pool_a(e, i); // 0x50788: jmp 0x48FC
        };
        let nh = self.enemies.handle(n);
        // cmpa.w a0,a1 ; bge: a1 (this object) at or above the new one keeps a0 = the new one as the rider.
        let rider_is_new = (h as i16) >= (nh as i16);
        let (x, y, z, var, dir) = (e.b.xi(), e.b.yi(), e.b.zi(), e.b.var26, e.b.dir);
        let (rider_h, bike_h) = if rider_is_new { (nh, h) } else { (h, nh) };
        {
            let r = if rider_is_new { &mut self.enemies.slots[n] } else { &mut *e };
            r.b.live = 1;
            r.b.k20 = 31 * 4;
            r.b.var26 = var;
            r.b.dir = dir;
            put_hi(&mut r.b.x, x);
            put_hi(&mut r.b.y, y);
            put_hi(&mut r.b.z, z);
            r.m.sw(o::BIKE, bike_h);
            r.m.sb(DROP, 0);
            r.b.sub = 2;
        }
        let k = if rider_is_new { &mut *e } else { &mut self.enemies.slots[n] };
        k.b.live = 1;
        k.b.k20 = 32 * 4;
        k.m.sw(0xA0, rider_h);
        put_hi(&mut k.b.x, x);
        put_hi(&mut k.b.y, y);
        put_hi(&mut k.b.z, z);
    }

    /// 0x4FD5E: sub 2 of the init: name 0x26, the table 0x50DFA, the bike to the speed id 11, HP row 2 (300 / 400 / 500), the poise 10,
    /// the stage end lock G.$7B4 = 2, the side (0x50DA8), the first target number (`rand() & 15` into 0x4FE46), the boss music 0x10,
    /// the floor (0x3140C), the car handle and flag, the HUD bar (G.-$70BC) and the boss controller (0x7BEDC, `+$AA` = this object).
    fn c31_init(&mut self, e: &mut Enemy, h: Handle) {
        e.b.id = 0x26;
        e.m.sw(W84, 0x98);
        e.b.anim_table = 0x50DFA;
        self.enemy_anim(&mut e.b, 0);
        e.b.box_table = 0x103000;
        e.b.multi_box = false;
        let mut bike = self.c31_take_bike(e);
        if let Some((_, k)) = bike.as_mut() {
            self.enemy_set_speed(&mut k.b, 0x0B); // 0x17A4 with a6 = the bike
        }
        self.c31_put_bike(bike);
        self.enemy_set_boss_hp(&mut e.b, 2); // 0x31E14 row 2
        e.m.sw(o::START_HP, e.b.hp);
        e.b.stun = 0x0A;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.m.sb(ATTACKING, 0);
        e.m.sb(B87, 0);
        e.b.b63 = 0;
        e.b.b72 = 0x78;
        self.cam.s2.lock78 = 2; // G.$7B4
        e.m.sb(o::WHEELIE, 0);
        e.m.sb(o::THROWING, 0);
        e.m.sb(ATTACKING, 0xFF);
        e.m.sb(o::BB, 0);
        e.m.sw(o::CRUISE, 0);
        e.m.sw(o::SAFE, 0);
        e.m.sw(o::GROUP, 0);
        for k in 0..3 {
            e.m.sw(o::HEROES + 2 * k, 0);
        }
        self.c31_side(e); // 0x50DA8
        let r = self.rand() & 0xF;
        e.m.sb(o::BB, self.assets.enemy.t.u8(o::FIRST_TARGET, r as u32));
        self.sound(0x10);
        self.enemy_snap_to_floor(e); // 0x3140C
        e.b.sub = 0;
        // 0x4FE24: 0x939B6 with a6 = the bike makes a debug marker only while the debug flag G.$4AA is set: not ported.
        e.m.sw(TARGET, self.cam.car); // G.$51A
        e.m.sb(o::CAR, self.spawn.g.car_mode); // G.$518
        e.b.mode = e.b.mode.wrapping_add(2);
        self.hud.boss = h; // G.-$70BC
        if let Some(controller) = self.boss_spawn_controller(h) {
            // 0x7BEDC copies the sign extended a6 (as in Vice's and the Butcher's init).
            if let Some(Slot::Prop(p)) = slot_of(controller) {
                self.props.slots[p].m.sl(0xAA, 0xFFFF_0000 | h as u32);
            }
        }
    }

    /// 0x4FE56: mode 2. Every 300 frames G.$502 = 1 (the controller's next group); `+$00` is 2 (untouchable) while `+$C6` counts down,
    /// else 1. Then by `+$05` (the long table 0x4FE8E): 0 alive (0x4FE9E), 2 the hit (0x5034C), 4 nothing, 6 held (0x2B35A).
    fn c31_alive(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        let t = e.m.w(o::GROUP);
        e.m.sw(o::GROUP, t.wrapping_sub(1));
        if t == 0 {
            e.m.sw(o::GROUP, 300);
            self.spawn.g.b502 = 1;
        }
        e.b.live = 1;
        let s = e.m.w(o::SAFE);
        e.m.sw(o::SAFE, s.wrapping_sub(1));
        if s != 0 {
            e.b.live = 2;
        } else {
            e.m.sw(o::SAFE, 0);
        }
        match e.b.sub {
            0 => self.c31_ride(e, h),
            2 => self.c31_hit(e, h, i),
            4 => {}
            _ => self.enemy_grabbed(e, h),
        }
    }

    /// 0x4FE9E: alive. The poise comes back to 10 when `+$79` runs out; the behaviour (0x4FEF0, the only entry of the table 0x4FEDE);
    /// in car mode no attack box; the rider sits on the bike (0x5078E); terrain, draw and the hostile list (0x4C0A).
    fn c31_ride(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(POISE_T);
        if t != 0 {
            e.m.sb(POISE_T, t - 1);
            if t == 1 {
                e.b.stun = 0x0A;
            }
        }
        self.c31_think(e);
        if e.m.b(o::CAR) != 0 {
            e.b.attack_box = 0;
        }
        self.c31_follow(e);
        self.c31_terrain(e, h);
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_monster_register(h);
    }

    /// 0x4FEF0: the behaviour. A wait (`+$7A`) and the hit stop (`+$59`, at its end the frame timer is set to 1) come first. Then the
    /// car branch (0x50084) or the foot branch: the throw (a roll every 60 frames, 0x50AD8), else the animation; when the car flag
    /// G.$518 is set the car behaviour starts (`+$AE` = 1, `+$07` = 0), else the foot sub state (table 0x4FF6E).
    fn c31_think(&mut self, e: &mut Enemy) {
        let w = e.m.b(WAIT);
        if w != 0 {
            e.m.sb(WAIT, w - 1);
            if w != 1 {
                return;
            }
        }
        let hs = e.b.hitstop;
        if hs != 0 {
            e.b.hitstop = hs - 1;
            if hs != 1 {
                return;
            }
            e.b.timer = 1;
        }
        if e.m.b(o::CAR) != 0 {
            return self.c31_car(e);
        }
        self.c31_throw_or_anim(e, 0x3C);
        if self.spawn.g.car_mode != 0 {
            e.m.sb(o::CAR, 1);
            e.b.step = 0;
            return;
        }
        match e.b.step {
            0 => {
                self.c31_ride_start(e);
                self.c31_ride_across(e);
            }
            2 => self.c31_ride_across(e),
            _ => {
                // 0x50054: the wait off screen.
                let t = e.m.w(T80);
                e.m.sw(T80, t.wrapping_sub(1));
                if t == 0 {
                    e.b.step = 0;
                }
            }
        }
    }

    /// 0x4FF20 / 0x5008A: a throw in progress goes on (0x502D0); else every `period` frames the roll (0x50AD8) may start one, and
    /// without a throw the animation runs (0x121E).
    fn c31_throw_or_anim(&mut self, e: &mut Enemy, period: u8) {
        if e.m.b(o::THROWING) == 0 {
            let t = e.m.b(o::ROLL_T);
            e.m.sb(o::ROLL_T, t.wrapping_sub(1));
            if t != 0 || {
                e.m.sb(o::ROLL_T, period);
                !self.c31_throw_roll(e)
            } {
                self.enemy_step(&mut e.b);
                return;
            }
            e.m.sb(o::THROWING, 1);
            e.m.sw(o::THROW_STEP, 0);
        }
        self.c31_throw(e);
    }

    /// 0x4FF74: foot sub state 0, the set up of a ride: the wheelie roll (0x509A8: animation 9 for the rider, 1 for the bike, else 3 and 0),
    /// the side (0x50DA8), the ride attack 0x6A with a new serial, the entry point (0x507DC), the bike at the speed id 0x21, the direction
    /// and facing of both, the target ahead (0x50894) and the 30 frame steering timer. It falls into sub state 2.
    fn c31_ride_start(&mut self, e: &mut Enemy) {
        e.b.step = e.b.step.wrapping_add(2);
        e.m.sb(o::ROLL_T, 0x0C);
        self.c31_wheelie_roll(e);
        let (ra, ba) = if e.m.b(o::WHEELIE) == 0 { (3, 0) } else { (9, 1) };
        self.enemy_anim(&mut e.b, ra);
        let mut bike = self.c31_take_bike(e);
        if let Some((_, k)) = bike.as_mut() {
            self.enemy_anim(&mut k.b, ba);
        }
        self.c31_put_bike(bike);
        self.c31_side(e);
        e.b.atk = 0x6A;
        e.b.knock = 0;
        e.b.hitstop = 0;
        e.b.serial = self.w.attack_serial; // G.$67BE
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
        self.c31_entry(e);
        let mut bike = self.c31_take_bike(e);
        if let Some((_, k)) = bike.as_mut() {
            self.enemy_set_speed(&mut k.b, 0x21); // 0x17A4 with a6 = the bike
        }
        let (facing, dir) = if e.m.b(o::FROM_RIGHT) != 0 { (0, 0x18) } else { (1, 8) };
        e.b.facing = facing;
        e.b.dir = dir;
        if let Some((_, k)) = bike.as_mut() {
            k.b.facing = facing;
            k.b.dir = dir;
        }
        self.c31_put_bike(bike);
        self.c31_target_ahead(e);
        e.m.sw(o::A4, 0x1E);
    }

    /// 0x50016: foot sub state 2, the ride across. The wheelie ends after `+$A7` frames (0x50060); past the far edge of the screen
    /// (0x5085E) the side is picked again, `+$80` = 120 and sub state 4 follows (the rest of this frame still runs); every 30 frames
    /// one step toward the target (0x5092A); the bike moves along its direction (0x1862).
    fn c31_ride_across(&mut self, e: &mut Enemy) {
        if e.m.b(o::WHEELIE) != 0 {
            let t = e.m.b(o::WHEELIE_T);
            e.m.sb(o::WHEELIE_T, t.wrapping_sub(1));
            if t == 0 {
                e.m.sb(o::WHEELIE, 0);
                self.enemy_anim(&mut e.b, 3);
                let mut bike = self.c31_take_bike(e);
                if let Some((_, k)) = bike.as_mut() {
                    self.enemy_anim(&mut k.b, 0);
                }
                self.c31_put_bike(bike);
            }
        }
        if self.c31_past_end(e) {
            self.c31_side(e);
            e.m.sw(T80, 0x78);
            e.b.step = e.b.step.wrapping_add(2);
        }
        let t = e.m.w(o::A4);
        e.m.sw(o::A4, t.wrapping_sub(1));
        if t == 0 {
            self.c31_steer(e);
            e.m.sw(o::A4, 0x1E);
        }
        let mut bike = self.c31_take_bike(e);
        if let Some((_, k)) = bike.as_mut() {
            self.enemy_move_dir(&mut k.b); // 0x1862
        }
        self.c31_put_bike(bike);
    }

    /// 0x5078E: the rider sits on the bike: X = bike X + dx (dx negated when he faces right), Y and Z plus the row of the table 0x507D0
    /// (0 for the ride, X 34 and Y 19 for the wheelie). Integer words only.
    fn c31_follow(&mut self, e: &mut Enemy) {
        let k = self.c31_bike(e);
        let row = e.m.b(o::WHEELIE) as u32 * 6;
        let t = &self.assets.enemy.t;
        let mut dx = t.i16(o::SEAT, row);
        if e.b.facing != 0 {
            dx = dx.wrapping_neg();
        }
        let (dy, dz) = (t.i16(o::SEAT, row + 2), t.i16(o::SEAT, row + 4));
        put_hi(&mut e.b.x, k.xi().wrapping_add(dx));
        put_hi(&mut e.b.y, k.yi().wrapping_add(dy));
        put_hi(&mut e.b.z, k.zi().wrapping_add(dz));
    }

    /// 0x507DC: the entry point of a ride: one of five rows of the side (0x50816, the right side 20 bytes on) by the weights of 0x5083E
    /// (`rand() & 31`): bike X = camera X + dx (-112 or +496), bike Z = 16 .. 80.
    fn c31_entry(&mut self, e: &mut Enemy) {
        let base = if e.m.b(o::FROM_RIGHT) != 0 { 0x14 } else { 0 };
        let r = self.rand() as u32 & 0x1F;
        let t = &self.assets.enemy.t;
        let row = base + 4 * t.u8(o::ENTRY_W, r) as u32;
        let x = (self.cam.s2.cam_x as i16).wrapping_add(t.i16(o::ENTRY, row));
        let z = t.i16(o::ENTRY, row + 2);
        if let Some(Slot::Enemy(b)) = slot_of(e.m.w(o::BIKE)) {
            let k = &mut self.enemies.slots[b].b;
            put_hi(&mut k.x, x);
            put_hi(&mut k.z, z);
        }
    }

    /// 0x5085E: the ride is over (the carry): going right when the bike X is at or past camera X + 0x240, going left when it is below
    /// camera X - 0xC0 (unsigned words).
    fn c31_past_end(&self, e: &Enemy) -> bool {
        let bx = self.c31_bike(e).xi() as u16;
        let cam = self.cam.s2.cam_x;
        if e.m.b(o::FROM_RIGHT) == 0 {
            cam.wrapping_add(0x240) <= bx
        } else {
            cam.wrapping_sub(0xC0) > bx
        }
    }

    /// 0x50894: the target of the ride: of the heroes in play ahead of him (at or left of his X going left, at or right going right),
    /// the nearest in depth (the last one of a tie; unsigned compares); hero 1 when there is none.
    fn c31_target_ahead(&mut self, e: &mut Enemy) {
        let mut best: Option<Handle> = None;
        let mut d4 = 0xFFFFu16;
        let (x, z) = (e.b.xi() as u16, e.b.zi());
        for p in 0..3 {
            let b = &self.players[p].b;
            if b.live == 0 {
                continue;
            }
            let ahead = if e.b.facing == 0 { x >= b.xi() as u16 } else { x <= b.xi() as u16 };
            if !ahead {
                continue;
            }
            let d0 = z.wrapping_sub(b.zi()).wrapping_abs() as u16;
            if d4 >= d0 {
                d4 = d0;
                best = Some(Game::player_handle(p));
            }
        }
        let t = if (d4 as i16) < 0 { Game::player_handle(0) } else { best.unwrap_or(Game::player_handle(0)) };
        e.m.sw(TARGET, t);
    }

    /// 0x5092A: one steering step of the bike toward the target, only while the target seen last time (the bike's `+$88`) is still ahead
    /// (0x5097E). The target's X and Z go to the bike's `+$88` / `+$8A`; the bike's direction (`+$22`) turns one step toward the direction
    /// to it (0x103A), or takes it when it is within one step.
    fn c31_steer(&mut self, e: &mut Enemy) {
        let k = self.c31_bike(e);
        let seen = self.c31_bike_m(e).w(DEST_X);
        let behind = if e.m.b(o::FROM_RIGHT) == 0 { seen <= k.xi() as u16 } else { seen > k.xi() as u16 };
        if behind {
            return;
        }
        let tb = self.body(e.m.w(TARGET)).clone();
        let mut bike = self.c31_take_bike(e);
        if let Some((_, k)) = bike.as_mut() {
            let (tx, tz) = (tb.xi() as u16, tb.zi() as u16);
            k.m.sw(DEST_X, tx);
            k.m.sw(DEST_Z, tz);
            let d0 = Game::enemy_angle_to(&k.b, tx, tz);
            let d1 = k.b.dir.wrapping_sub(d0).wrapping_add(1) & 0x1F;
            let dir = if d1 <= 2 {
                d0
            } else {
                let step: u8 = if d1 < 0x11 { 0xFF } else { 1 };
                k.b.dir.wrapping_add(step)
            };
            k.b.dir = dir & 0x1F;
        }
        self.c31_put_bike(bike);
    }

    /// The bike's own bytes.
    fn c31_bike_m(&self, e: &Enemy) -> &super::Ram {
        match slot_of(e.m.w(o::BIKE)) {
            Some(Slot::Enemy(b)) => &self.enemies.slots[b].m,
            _ => panic!("Hogg without his bike"),
        }
    }

    /// 0x509A8: the wheelie roll: `rand() & 15 < 2` (signed) sets `+$A6` = 1 for 60 frames.
    fn c31_wheelie_roll(&mut self, e: &mut Enemy) {
        if (self.rand() & 0xF) < 2 {
            e.m.sb(o::WHEELIE, 1);
            e.m.sb(o::WHEELIE_T, 0x3C);
        }
    }

    /// 0x50DA8: the side by the bike's X: left of camera X + 0xC0 he rides right (`+$A3` = 0, facing 1, direction 8), else left
    /// (`+$A3` = 1, facing 0, direction 0x18); for the rider and the bike.
    fn c31_side(&mut self, e: &mut Enemy) {
        let bx = self.c31_bike(e).xi() as u16;
        let right = self.cam.s2.cam_x.wrapping_add(0xC0) >= bx;
        let (from_right, facing, dir) = if right { (0, 1, 8) } else { (1, 0, 0x18) };
        e.m.sb(o::FROM_RIGHT, from_right);
        e.b.facing = facing;
        e.b.dir = dir;
        if let Some(Slot::Enemy(b)) = slot_of(e.m.w(o::BIKE)) {
            let k = &mut self.enemies.slots[b].b;
            k.facing = facing;
            k.dir = dir;
        }
    }

    // ------------------------------------------------------------------ the throw

    /// 0x50AD8: the throw roll (the carry): only when he is on screen (`+$01`) and not in a wheelie, `rand() & 31 <= T[level]` (0x50B0A,
    /// signed); no random number otherwise.
    fn c31_throw_roll(&mut self, e: &Enemy) -> bool {
        if e.b.visible == 0 || e.m.b(o::WHEELIE) != 0 {
            return false;
        }
        let limit = self.assets.enemy.t.u8(o::THROW_T, self.w.level as u32 & 0x1F) as i16;
        (self.rand() & 0x1F) as i16 <= limit
    }

    /// 0x502D0: the throw, step by `+$B4` (table 0x502DC). Step 0 (0x502E0): on foot a target (0x50C08) and its side (0x50D6A), the
    /// animation (0x50B2A), the volley size (0x50B74). Step 2 (0x502FA): at the release frame (`+$2C`) a bomb (0x55A7E) at the car
    /// (variant `+$B8` * 2) or at the target (on foot only while he is on screen, 0x50D92, else the throw ends at once); when the
    /// animation ends one throw of the volley is done; after the last one animation 3 and `+$AF` = 0.
    fn c31_throw(&mut self, e: &mut Enemy) {
        if e.m.w(o::THROW_STEP) == 0 {
            e.m.sw(o::THROW_STEP, 2);
            if e.m.b(o::CAR) == 0 {
                self.c31_pick_target(e);
                self.c31_target_side(e);
            }
            self.c31_throw_anim(e);
            e.m.sb(o::VOLLEY, c31_volley(e.m.w(o::START_HP), e.b.hp));
            return;
        }
        if e.b.param_a != 0 {
            if e.m.b(o::CAR) != 0 {
                let v = e.m.b(o::B8);
                let car = self.cam.car;
                self.c31_bomb(e, v, car);
            } else {
                let d0 = self.cam.s2.cam_x.wrapping_add(0xC0).wrapping_sub(e.b.xi() as u16).wrapping_add(0xC0);
                if d0 >= 0x180 {
                    return self.c31_throw_end(e); // 0x5033E (the release flag stays)
                }
                let v = self.c31_target_side(e);
                let target = e.m.w(TARGET);
                self.c31_bomb(e, v, target);
            }
            e.b.param_a = 0;
        }
        if self.enemy_step(&mut e.b) {
            let n = e.m.b(o::VOLLEY).wrapping_sub(1);
            e.m.sb(o::VOLLEY, n);
            if n == 0 {
                self.c31_throw_end(e);
            }
        }
    }

    /// 0x5033E: the throw is over: animation 3, `+$AF` = 0.
    fn c31_throw_end(&mut self, e: &mut Enemy) {
        self.enemy_anim(&mut e.b, 3);
        e.m.sb(o::THROWING, 0);
    }

    /// 0x50B2A: the throw animation. In car mode by the bike's place: more than 0xC0 px ahead of the car's X (unsigned) animation 12 and
    /// `+$B8` = 1 (the low arc), else animation 11 and `+$B8` = 0. On foot animation 11 + `+$B8` - 2 (the side of the target).
    fn c31_throw_anim(&mut self, e: &mut Enemy) {
        if e.m.b(o::CAR) != 0 {
            let car_x = self.body(self.cam.car).xi() as u16;
            let bike_x = self.c31_bike(e).xi() as u16;
            if car_x.wrapping_add(0xC0) < bike_x {
                self.enemy_anim(&mut e.b, 0x0C);
                e.m.sb(o::B8, 1);
            } else {
                self.enemy_anim(&mut e.b, 0x0B);
                e.m.sb(o::B8, 0);
            }
            return;
        }
        let a = 0x0Bu8.wrapping_add(e.m.b(o::B8).wrapping_sub(2));
        self.enemy_anim(&mut e.b, a as u16);
    }

    /// 0x50D6A: the side of the target (`+$76`): `+$B8` = 3 when it is at or right of him, 2 when left (swapped when he faces right).
    /// Returns the value (d0).
    fn c31_target_side(&mut self, e: &mut Enemy) -> u8 {
        let tx = self.body(e.m.w(TARGET)).xi() as u16;
        let (mut d2, mut d3) = (2u8, 3u8);
        if e.b.facing != 0 {
            std::mem::swap(&mut d2, &mut d3);
        }
        let d1 = if tx < e.b.xi() as u16 { d2 } else { d3 };
        e.m.sb(o::B8, d1);
        d1
    }

    /// 0x55A7E: the bomb, a pool A class 42 object (M6-25 ports its handler): the variant `v` * 2, the target `+$76`, the thrower's facing,
    /// the start point (0x55AAC: variants with bit 1 clear 20 px in front at the height + 75, the others 76 px behind at + 54) and the
    /// thrower in `+$A0`. A full pool makes nothing.
    fn c31_bomb(&mut self, e: &Enemy, v: u8, target: Handle) {
        let Some(n) = self.enemies.alloc() else { return };
        let me = self.c31_self_handle(e);
        let b = &mut self.enemies.slots[n];
        b.b.live = 1;
        b.b.k20 = 42 * 4;
        let var = (v as u16).wrapping_mul(2);
        b.b.var26 = var;
        b.m.sw(TARGET, target);
        b.b.facing = e.b.facing;
        let (mut dx, dy) = if var & 2 == 0 { (0x14i16, 0x4Bi16) } else { (0x4C, 0x36) };
        if (var & 2 == 0) == (b.b.facing == 0) {
            dx = -dx;
        }
        put_hi(&mut b.b.x, e.b.xi().wrapping_add(dx));
        put_hi(&mut b.b.y, e.b.yi().wrapping_add(dy));
        put_hi(&mut b.b.z, e.b.zi());
        b.m.sw(0xA0, me);
    }

    /// The handle of the rider (the bike keeps it in its `+$A0`).
    fn c31_self_handle(&self, e: &Enemy) -> Handle {
        self.c31_bike_m(e).w(0xA0)
    }

    /// The byte `+$C8` of any object (a hero's action class).
    fn c31_byte_c8(&self, h: Handle) -> u8 {
        match slot_of(h) {
            Some(Slot::Player(p)) => self.players[p].action,
            Some(Slot::Enemy(i)) => self.enemies.slots[i].m.b(0xC8),
            Some(Slot::Prop(i)) => self.props.slots[i].m.b(0xC8),
            Some(Slot::Item(i)) => self.items.slots[i].m.b(0xC8),
            _ => 0,
        }
    }

    /// 0x50D08: the heroes in play (`+$00`) in `+$BC`, `+$BE`, `+$C0` in order, the rest cleared (with nobody in play `+$BC` stays);
    /// an empty `+$BC` becomes hero 1. Returns the count - 1 (0 with nobody).
    fn c31_heroes(&mut self, e: &mut Enemy) -> u16 {
        let mut n = 0u16;
        let mut at = o::HEROES;
        for p in 0..3 {
            if self.players[p].b.live != 0 {
                n += 1;
                e.m.sw(at, Game::player_handle(p));
                at += 2;
            }
        }
        let d1 = if n == 0 {
            at += 2;
            0
        } else {
            n - 1
        };
        while at <= o::HEROES + 4 {
            e.m.sw(at, 0);
            at += 2;
        }
        if e.m.w(o::HEROES) == 0 {
            e.m.sw(o::HEROES, Game::player_handle(0));
        }
        d1
    }

    /// 0x50C08: the target of a throw. When the present target's `+$C8` is 0x23 (0x50C98): one hero takes hero 1, two alternate, three
    /// step on by 1 or 2 (bit 3 of `rand() & 15`, 0x50CAC). Else a row of 0x50C62 by the heroes in play, an entry by `rand() & 15`, added
    /// to `+$C2` and wrapped by the count; empty list places are skipped.
    fn c31_pick_target(&mut self, e: &mut Enemy) {
        if self.c31_byte_c8(e.m.w(TARGET)) == 0x23 {
            let d1 = self.c31_heroes(e);
            if d1 == 0 {
                e.m.sw(TARGET, e.m.w(o::HEROES));
                e.m.sb(o::C2, 0);
            } else if d1 == 1 {
                let d0 = e.m.b(o::C2).wrapping_add(1) & 1;
                e.m.sw(TARGET, e.m.w(o::HEROES + 2 * d0 as usize));
                e.m.sb(o::C2, d0);
            } else {
                let r = self.rand() & 0xF;
                let mut d2 = e.m.b(o::C2).wrapping_add(if r & 8 != 0 { 2 } else { 1 });
                if d2 as i8 > 2 {
                    d2 = d2.wrapping_sub(3);
                }
                e.m.sb(o::C2, d2);
                e.m.sw(TARGET, e.m.w(o::HEROES + 2 * d2 as usize));
            }
            return;
        }
        let r = (self.rand() & 0xF) as u32;
        let d1 = self.c31_heroes(e);
        let t = &self.assets.enemy.t;
        let row = t.u16(o::TARGET_ROWS, 2 * d1 as u32) as u32;
        // move.b into a word whose high byte is 0, add.b, then the word compares.
        let mut d0 = t.u8(o::TARGET_ROWS, row + r).wrapping_add(e.m.b(o::C2)) as i16;
        let d4 = d1 as i16;
        if d0 > d4 {
            d0 = d0.wrapping_sub(d4 + 1);
        }
        e.m.sb(o::C2, d0 as u8);
        let mut k = d0.wrapping_mul(2);
        loop {
            if k > 4 {
                k = 0;
            }
            let h = e.m.w(o::HEROES + k as usize);
            if h != 0 {
                e.m.sw(TARGET, h);
                return;
            }
            k += 2;
        }
    }

    // ------------------------------------------------------------------ car mode (M6-24)

    /// 0x50084: the car branch. The throw roll every 30 frames, then the sub state (table 0x500C0): 0 set up (0x500CA), 2 a new aim
    /// cell and cruise time (0x500E6), 4 the cruise (0x5012C), 6 the coast (0x501CC), 8 a wait (0x501F4, not reached).
    fn c31_car(&mut self, e: &mut Enemy) {
        self.c31_throw_or_anim(e, 0x1E);
        match e.b.step {
            0 => {
                // 0x500CA: animation 3, the bike at 8 px per frame; falls into sub state 2.
                e.b.step = e.b.step.wrapping_add(2);
                self.enemy_anim(&mut e.b, 3);
                if let Some(Slot::Enemy(b)) = slot_of(e.m.w(o::BIKE)) {
                    self.enemies.slots[b].b.vx = 0x800;
                }
                self.c31_car_pick(e);
            }
            2 => self.c31_car_pick(e),
            4 => self.c31_cruise(e),
            6 => self.c31_coast(e),
            _ => {
                // 0x501F4: count down `+$80` (byte), at the end the foot behaviour.
                let t = e.m.b(T80);
                e.m.sb(T80, t.wrapping_sub(1));
                if t == 0 {
                    self.c31_car_end(e);
                }
            }
        }
    }

    /// 0x500E6: sub state 2: the cell of the bike (0x509C6), a random next cell of its row (0x50A00), a cruise time (`rand() & 31` into
    /// 0xC861A), the re-aim timer 1, facing right, the aim point (0x50A24); falls into the cruise.
    fn c31_car_pick(&mut self, e: &mut Enemy) {
        e.b.step = e.b.step.wrapping_add(2);
        e.m.sw(o::AA, 0);
        e.m.sw(o::AC, 0);
        let car = self.body(self.cam.car).clone();
        e.m.sb(o::CELL, c31_cell(car.xi(), car.zi(), e.b.xi(), e.b.zi())); // the rider's X and Z
        let r = self.rand() as u32 & 0x1F;
        e.m.sb(o::AIM, self.assets.enemy.t.u8(o::CELLS, ((e.m.b(o::CELL) as u32) << 5) + r));
        let r = self.rand() as u32 & 0x1F;
        e.m.sw(o::CRUISE, self.assets.enemy.t.u16(o::CRUISE_T, 2 * r));
        e.m.sw(o::A4, 1);
        e.b.facing = 1;
        if let Some(Slot::Enemy(b)) = slot_of(e.m.w(o::BIKE)) {
            self.enemies.slots[b].b.facing = 1;
        }
        self.c31_aim(e);
        self.c31_cruise(e);
    }

    /// 0x5012C: sub state 4, the cruise. A car flag other than 1 starts the coast (0x501AE). When the cruise time runs out a new cell
    /// is drawn in this frame (back to 0x500E6). Every 25 frames the aim point of the same cell is recomputed from the car's place (a
    /// random number is drawn for the time table 0xC865A, then the time is set to 25 anyway). The bike steers (0x50A82, 0x50AB0),
    /// moves (0x189C) and its X speed is held within 0x300 .. 0x1200.
    fn c31_cruise(&mut self, e: &mut Enemy) {
        if self.spawn.g.car_mode != 1 {
            return self.c31_coast_start(e);
        }
        let t = e.m.w(o::CRUISE);
        e.m.sw(o::CRUISE, t.wrapping_sub(1));
        if t == 0 {
            e.b.step = 2;
            return self.c31_car_pick(e);
        }
        let a = e.m.w(o::A4).wrapping_sub(1);
        e.m.sw(o::A4, a);
        if a == 0 {
            let r = self.rand() as u32 & 0x1F;
            let _unused = self.assets.enemy.t.u16(o::REAIM_T, 2 * r); // 0x5014E: overwritten at once
            e.m.sw(o::A4, 0x19);
            self.c31_aim(e);
        }
        let car = self.body(self.cam.car).clone();
        let mut bike = self.c31_take_bike(e);
        if let Some((_, k)) = bike.as_mut() {
            // 0x50A82: within 8 px of the aim X the bike takes the car's speed and stops accelerating.
            if (e.b.xi().wrapping_sub(k.m.w(DEST_X) as i16).wrapping_add(8) as u16) < 0x10 {
                k.b.vx = car.vx;
                k.b.ax = 0;
            }
            // 0x50AB0: within 8 px of the aim Z the Z motion stops.
            if (e.b.zi().wrapping_sub(k.m.w(DEST_Z) as i16).wrapping_add(8) as u16) < 0x10 {
                k.b.vz = 0;
                k.b.az = 0;
            }
            k.b.motion(); // 0x189C
            if k.b.vx > 0x1200 {
                k.b.vx = 0x1200;
                k.b.ax = 0;
            }
            if k.b.vx < 0x300 {
                k.b.vx = 0x300;
                k.b.ax = 0;
            }
        }
        self.c31_put_bike(bike);
    }

    /// 0x50A24: the aim point of the cell `+$A9`: car position + (dx, dz) of the grid 0xC821A into the bike's `+$88` / `+$8A`; the bike's
    /// vz = +-0x180 toward the aim Z (0 counts as up), ax = +-0x80 toward the aim X (unsigned: equal or right is +), az = 0.
    fn c31_aim(&mut self, e: &mut Enemy) {
        let car = self.body(self.cam.car).clone();
        let cell = e.m.b(o::AIM) as u32 * 4;
        let t = &self.assets.enemy.t;
        let (dx, dz) = (t.i16(o::GRID, cell), t.i16(o::GRID, cell + 2));
        let mut bike = self.c31_take_bike(e);
        if let Some((_, k)) = bike.as_mut() {
            let ax_ = car.xi().wrapping_add(dx) as u16;
            let az_ = car.zi().wrapping_add(dz) as u16;
            k.m.sw(DEST_X, ax_);
            k.m.sw(DEST_Z, az_);
            k.b.vz = if (az_ as i16) >= k.b.zi() { 0x180 } else { -0x180 }; // sub.w ; bge
            k.b.az = 0;
            k.b.ax = if ax_ >= k.b.xi() as u16 { 0x80 } else { -0x80 };
        }
        self.c31_put_bike(bike);
    }

    /// 0x501AE: the car is gone: sub state 6, the bike's accelerations and its Y and Z speeds cleared (X speed kept); falls into the coast.
    fn c31_coast_start(&mut self, e: &mut Enemy) {
        e.b.step = e.b.step.wrapping_add(2);
        if let Some(Slot::Enemy(b)) = slot_of(e.m.w(o::BIKE)) {
            let k = &mut self.enemies.slots[b].b;
            k.ax = 0;
            k.vy = 0;
            k.ay = 0;
            k.vz = 0;
            k.az = 0;
        }
        self.c31_coast(e);
    }

    /// 0x501CC: the coast: the bike rolls on (0x189C) until the rider is past camera X + 0x280 (unsigned), then the foot behaviour.
    fn c31_coast(&mut self, e: &mut Enemy) {
        if self.cam.s2.cam_x.wrapping_add(0x280) < e.b.xi() as u16 {
            return self.c31_car_end(e);
        }
        let mut bike = self.c31_take_bike(e);
        if let Some((_, k)) = bike.as_mut() {
            k.b.motion();
        }
        self.c31_put_bike(bike);
    }

    /// 0x501FC: back to the foot behaviour: `+$AE` = 0, sub state 0, the side (0x50DA8).
    fn c31_car_end(&mut self, e: &mut Enemy) {
        e.m.sb(o::CAR, 0);
        e.b.step = 0;
        self.c31_side(e);
    }

    // ------------------------------------------------------------------ hit and death

    /// 0x5034C: sub 2, the hit. The damage credit (0x3262A); at HP 0 the boss goes back inside the camera (0x32740) and, once, the defeat
    /// (0xACEE) and the music stop (0xA5EA). With HP left the rider sits on the bike. Then the foot (table 0x5039A by `+$06`: 0 a flinch,
    /// 2 and 4 a longer one, 6 the death) or the car reactions (0x50554), and the draw (0x49CA).
    fn c31_hit(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        let defeat = e.b.hp == 0 && e.b.hp_prev != 0;
        self.boss_hit(e, h); // 0x3262A, 0x32740, 0xACEE, clr $6E
        if defeat {
            self.hook_a5ea();
        }
        if e.b.hp != 0 {
            self.c31_follow(e);
        }
        if e.m.b(o::CAR) != 0 {
            match e.b.act {
                6 => self.c31_death(e, h, i),
                _ => self.c31_car_hit(e, h, i),
            }
        } else {
            match e.b.act {
                0 => {
                    if e.b.step == 0 && !self.c31_flinch_start(e, h, i) {
                        return self.enemy_draw_check(&mut e.b, h);
                    }
                    let t = e.b.hitstop.wrapping_sub(1);
                    e.b.hitstop = t;
                    if t == 0 {
                        self.c31_recover(e);
                    }
                }
                2 | 4 => match e.b.step {
                    0 => {
                        if self.c31_flinch_start(e, h, i) {
                            let t = e.b.hitstop.wrapping_sub(1);
                            e.b.hitstop = t;
                            if t == 0 {
                                self.c31_recover(e);
                            }
                        }
                    }
                    2 => {
                        let t = e.b.hitstop.wrapping_sub(1);
                        e.b.hitstop = t;
                        if t == 0 {
                            e.b.step = e.b.step.wrapping_add(2);
                        }
                    }
                    _ => {
                        e.m.sw(o::SAFE, 0x3C);
                        self.c31_recover(e);
                    }
                },
                _ => self.c31_death(e, h, i),
            }
        }
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x503AA: the HUD bar (0x16B36, kind 0); at HP 0 the death (0x503FA, the caller's frame ends there: returns false); else
    /// animation 4 and the next step.
    fn c31_flinch_start(&mut self, e: &mut Enemy, h: Handle, i: usize) -> bool {
        self.c31_hud(e, h, i);
        if e.b.hp == 0 {
            self.c31_death(e, h, i);
            return false;
        }
        self.enemy_anim(&mut e.b, 4);
        e.b.step = e.b.step.wrapping_add(2);
        true
    }

    /// 0x16B36 needs the object in its slot: the body is copied there for the call.
    fn c31_hud(&mut self, e: &Enemy, h: Handle, i: usize) {
        self.enemies.slots[i].b = e.b.clone();
        self.enemy_hud_request(h, 0);
    }

    /// 0x50520: the end of a flinch: the poise 10 with its 60 frame refill timer, `+$05` = `+$06` = 0, sub state 4 with 30 frames when
    /// the bike is past the end of the ride, else 2; animation 3 (0x4FEE8).
    fn c31_recover(&mut self, e: &mut Enemy) {
        e.b.stun = 0x0A;
        e.m.sb(POISE_T, 0x3C);
        e.b.sub = 0;
        e.b.act = 0;
        if self.c31_past_end(e) {
            e.b.step = 4;
            e.m.sw(T80, 0x1E);
        } else {
            e.b.step = 2;
        }
        self.enemy_anim(&mut e.b, 3);
    }

    /// 0x50570: the car mode reaction (a car ram, M6-24). Step 0 (0x50582): the HUD bar; at HP 0 the death (0x5061A); else animation 4,
    /// the bike takes the car's speed plus a knock back of 0x480 away from the car (table 0x50616), the bike's Z motion stops, `+$C6` = 0x50
    /// frames untouchable. Ahead of the car `+$80` is the byte 40; behind it the word 0x50 is written, whose high byte 0 ends the
    /// reaction in the first count (a quirk). Step 2 (0x505F6): the bike moves, terrain, the count; at its end 0x50740.
    fn c31_car_hit(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.step == 0 {
            self.c31_hud(e, h, i);
            if e.b.hp == 0 {
                return self.c31_death(e, h, i);
            }
            self.enemy_anim(&mut e.b, 4);
            let car = self.body(self.cam.car).clone();
            let t = &self.assets.enemy.t;
            let (mut d0, d1) = (t.i16(o::RAM_V, 0), 0i16);
            e.m.sb(o::BA, 1);
            let mut bike = self.c31_take_bike(e);
            if let Some((_, k)) = bike.as_mut() {
                k.b.vx = car.vx;
                if (car.xi() as u16) < k.b.xi() as u16 {
                    e.m.sb(T80, 0x28);
                } else {
                    d0 = d0.wrapping_neg();
                    e.m.sb(o::BA, 0);
                    e.m.sw(T80, 0x50);
                }
                k.b.vx = k.b.vx.wrapping_add(d0);
                k.b.ax = d1;
                k.b.vz = 0;
                k.b.az = 0;
            }
            self.c31_put_bike(bike);
            e.b.step = e.b.step.wrapping_add(2);
            e.m.sw(o::SAFE, 0x50);
        }
        let mut bike = self.c31_take_bike(e);
        if let Some((_, k)) = bike.as_mut() {
            k.b.motion();
        }
        self.c31_put_bike(bike);
        self.c31_terrain(e, h);
        let t = e.m.b(T80);
        e.m.sb(T80, t.wrapping_sub(1));
        if t == 0 {
            // 0x50740
            e.b.stun = 0x0A;
            e.m.sb(POISE_T, 0x3C);
            e.b.set_state(e.b.state() & 0xFF00_0000);
            self.enemy_anim(&mut e.b, 3);
        }
    }

    /// 0x503FA / 0x5061A: the death, by `+$07` (tables 0x50408 / 0x50628). Step 0 (0x5040E): `+$06` = 6, animation 6, thrown off with
    /// vx +-0x100 (away from his facing), vy 0x500, gravity -0x28 (0x504E0), the cry 0x97D2, in the air, reaction 4, poise 10. Steps 2 and 4
    /// (0x50462, 0x504A8): fly (0x189C), bounce off the screen sides (0x504EC), the animation unless the frame holds (`+$2C`), terrain;
    /// the first landing bounces once (vy 0x280) with the thud 0x9796, the second ends in mode 4.
    fn c31_death(&mut self, e: &mut Enemy, h: Handle, _i: usize) {
        match e.b.step {
            0 => {
                e.b.act = 6;
                self.enemy_anim(&mut e.b, 6);
                let t = &self.assets.enemy.t;
                let mut vx = t.i16(o::DEATH_V, 0);
                if e.b.facing == 0 {
                    vx = vx.wrapping_neg();
                }
                e.b.vx = vx;
                e.b.ax = t.i16(o::DEATH_V, 2);
                e.b.vy = t.i16(o::DEATH_V, 4);
                e.b.ay = t.i16(o::DEATH_V, 6);
                e.b.vz = 0;
                e.b.az = 0;
                self.snd_enemy_death(1, e.b.xi(), e.b.facing); // 0x97D2
                e.b.airborne = 1;
                e.b.reaction = 4;
                e.b.stun = 0x0A;
                e.b.step = e.b.step.wrapping_add(2);
            }
            _ => {
                e.b.motion();
                self.c31_bounce(e);
                if e.b.param_a == 0 {
                    self.enemy_step(&mut e.b);
                }
                self.c31_terrain(e, h);
                if e.b.airborne != 0 {
                    return;
                }
                if e.b.step == 2 {
                    e.b.airborne = 1;
                    let t = &self.assets.enemy.t;
                    e.b.vy = t.i16(o::DEATH_V, 8);
                    e.b.ay = t.i16(o::DEATH_V, 10);
                    e.b.step = e.b.step.wrapping_add(2);
                    self.snd_9796(e.b.xi());
                    self.enemy_step(&mut e.b);
                } else {
                    self.enemy_step(&mut e.b);
                    self.snd_9796(e.b.xi());
                    e.b.set_state(0x0400_0000);
                }
            }
        }
    }

    /// 0x504EC / 0x5070C: the thrown rider turns back at camera X + 0x150 going right (unsigned) or camera X + 0x30 going left (signed).
    fn c31_bounce(&self, e: &mut Enemy) {
        let cam = self.cam.s2.cam_x;
        let x = e.b.xi();
        let turn = if e.b.vx >= 0 { (cam.wrapping_add(0x150)) < x as u16 } else { (cam.wrapping_add(0x30) as i16) > x };
        if turn {
            e.b.vx = e.b.vx.wrapping_neg();
            e.b.ax = e.b.ax.wrapping_neg();
        }
    }

    /// 0x50760: mode 4 (table 0x5076E): step 0 the kill score (0x325E6), then the leader flags (0x6148); the body stays and is drawn.
    fn c31_dead(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.sub == 0 {
            let b = e.b.clone();
            self.enemy_award_kill_score(&b);
            e.b.sub = e.b.sub.wrapping_add(2);
            self.enemy_boss_leader(e.b.who);
        }
        self.enemy_draw_check(&mut e.b, h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volley_grows_by_the_hp_quarter() {
        assert_eq!(c31_volley(300, 300), 1);
        assert_eq!(c31_volley(300, 226), 1);
        assert_eq!(c31_volley(300, 225), 2);
        assert_eq!(c31_volley(300, 151), 2);
        assert_eq!(c31_volley(300, 150), 3);
        assert_eq!(c31_volley(300, 76), 3);
        assert_eq!(c31_volley(300, 75), 4);
        assert_eq!(c31_volley(300, 70), 4);
    }

    #[test]
    fn cell_of_a_bike_level_with_the_car_is_row_2_column_16() {
        // Z equal: (0 + 0x40) & 0xFFF0 = 0x40, times 2 = 0x80 (row 4 of 32); X equal: 0x100 / 16 = 16.
        assert_eq!(c31_cell(1000, 48, 1000, 48), 0x80 + 16);
        // Far ahead and far behind clamp to the last and the first column.
        assert_eq!(c31_cell(1000, 48, 2000, 48) & 0x1F, 31);
        assert_eq!(c31_cell(1000, 48, 0, 48) & 0x1F, 0);
    }
}
