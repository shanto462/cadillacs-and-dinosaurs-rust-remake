//! The Cadillac of Episode 3, pool B class 88 (handler 0x8BDDC..0x8C886): the radio event 0x8BD24 makes three objects, the car (variant 0) and
//! its two wheels (variant 2 the front, 4 the rear, `+$76` = the car). D3c section 5, BS17 to BS22.
//!
//! The car: mode 0 is the arrival (`+$05`: 0 the init, 2 drive in at 5 px, 4 brake, 6 wait 120, 8 wait 60, 0xA the boarding jumps, 0xC accelerate,
//! 0xE the start of the drive), mode 2 the drive (the riders steer, the lane and screen band, the body bump, the smoke; `+$05` = 2 is the hit flash),
//! mode 4 the end (the heroes jump down, the wreck rolls off), mode 6 the free. The passes that let it ram and be hit are 0x121D8..0x12454 (M6-20).
//! Its pool C helpers (the seats 0x96FD4 class 36, the teammate 0x971D6 class 37, the dust and smoke 0x977D2 class 41) are made here; their handlers
//! are M6-22. Owner: enemy port.

use crate::game::item::Item;
use crate::game::prop::Prop;
use crate::game::{collide, slot_of, Game, Handle, Slot};

/// Own bytes of the car (the original offsets).
mod o {
    /// `+$76` word (wheels): the car. `+$80` word: the timers of the arrival; byte: the blink counter of the low HP flash (car), the
    /// turn timer (front wheel). `+$A0` byte (front wheel): the steering frame.
    pub const CAR: usize = 0x76;
    pub const T80: usize = 0x80;
    pub const A0: usize = 0xA0;
    /// `+$A2..+$A4` bytes: the seats (characters, 0xFF = empty; `+$A2` is first the delivering teammate), `+$A5..+$A7` the seats being
    /// built, `+$A8` the riders' bits (player numbers), `+$A9` the bump timer, `+$AE` word the bump step, `+$B0` / `+$B2` words the target vx
    /// and vz of the steering, `+$B4` byte the teammate's character.
    pub const SEAT: usize = 0xA2;
    pub const NEW_SEAT: usize = 0xA5;
    pub const RIDERS: usize = 0xA8;
    pub const BUMP_T: usize = 0xA9;
    pub const BUMP: usize = 0xAE;
    pub const TVX: usize = 0xB0;
    pub const TVZ: usize = 0xB2;
    pub const MATE: usize = 0xB4;
    /// The tables read at run time (extracted by `dino-extract/src/boss.rs`).
    pub const STICK: u32 = 0x8C886;
    pub const PAIRS_B: u32 = 0x8C896;
    pub const PAIRS: u32 = 0x8C8B6;
    pub const BUMPS: u32 = 0x8C404;
    pub const WHEEL_ANIM: u32 = 0x8C5CC;
}

/// The car's class word (+$20 = 88 * 4), read by the wheels.
const CAR_K20: u16 = 0x160;

impl Game {
    // ------------------------------------------------------------------ the radio event

    /// 0x8BD24: the radio is taken (`it`, a6 = the radio) in Episode 3 with no kill-all running. With 3 free pool B slots: three class 88 objects,
    /// sorted by address (the lowest is the car, the middle the front wheel, the highest the rear wheel); the car gets the taker's `+$02` and
    /// `+$03` (the wheels' writes of 0x8BD9C / 0x8BDB8 also go to the car: a slip of the original, kept). G.$4D5 = G.$4DB = 1, the kill-all
    /// G.$50E = 1 with the credit G.$4E0 = the radio's `+$70` (the taker).
    pub(crate) fn prop_car_radio(&mut self, it: &Item) {
        if self.spawn.g.kill_all != 0 || self.props.free_count() < 3 {
            return;
        }
        let mut s = [0usize; 3];
        for k in &mut s {
            *k = self.props.alloc().expect("3 free pool B slots");
        }
        let hd = |g: &Game, i: usize| g.props.handle(i);
        // cmp.w ; bcc ; exg (unsigned words): a three step sort.
        if hd(self, s[1]) < hd(self, s[0]) {
            s.swap(0, 1);
        }
        if hd(self, s[2]) < hd(self, s[1]) {
            s.swap(1, 2);
        }
        if hd(self, s[1]) < hd(self, s[0]) {
            s.swap(0, 1);
        }
        let car = self.props.handle(s[0]);
        for (k, &i) in s.iter().enumerate() {
            let p = &mut self.props.slots[i];
            p.b.live = 1;
            p.b.k20 = CAR_K20;
            p.b.var26 = 2 * k as u16;
            if k > 0 {
                p.m.sw(o::CAR, car);
            }
        }
        let c = &mut self.props.slots[s[0]];
        c.b.who = it.b.who;
        c.b.id = it.b.id;
        self.spawn.g.b4d5 = 1;
        self.flow.b4db = 1;
        self.spawn.g.kill_all = 1;
        self.spawn.g.killer = it.b.partner;
    }

    // ------------------------------------------------------------------ the handler

    /// 0x8BDDC: class 88 (mode table 0x8BDE8): 0 the arrival (0x8BDEC), 2 the drive or a wheel (0x8C0A4), 4 the end (0x8C810), 6 the free
    /// (0x8C880).
    pub(super) fn prop_car(&mut self, p: &mut Prop, h: Handle, i: usize) {
        match p.b.mode {
            0 => self.prop_car_arrival(p, h, i),
            2 => {
                // 0x8C0A4: by the variant (table 0x8C0B6), then 0x49C0.
                if p.b.var26 == 0 {
                    self.prop_car_drive(p, h);
                } else {
                    self.prop_car_wheel(p, i);
                }
                self.prop_car_draw(p, h);
            }
            4 => self.prop_car_end(p, h),
            _ => self.prop_free(p, i), // 0x8C880: jmp 0x4938
        }
    }

    /// 0x49C0 on the object in hand: on the draw list when its on-screen byte is set.
    fn prop_car_draw(&mut self, p: &Prop, h: Handle) {
        if p.b.visible != 0 {
            self.draw_list.push(((p.b.zi() as u16 as u32) << 16) | h as u32);
        }
    }

    /// `move.w` into the integer word of a position (the fraction stays).
    fn put(v: &mut i32, w: i16) {
        *v = (*v & 0xFFFF) | ((w as i32) << 16);
    }

    /// A new attack serial with the attack 0x66 (0x8C074 and 0x8C0D4).
    fn prop_car_arm(&mut self, p: &mut Prop) {
        p.b.atk = 0x66;
        p.b.knock = 0;
        p.b.hitstop = 0;
        p.b.serial = self.w.attack_serial;
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
    }

    /// 0x8BDF0: mode 0, the arrival, by `+$05` (table 0x8BDFC).
    fn prop_car_arrival(&mut self, p: &mut Prop, h: Handle, i: usize) {
        match p.b.sub {
            0 => return self.prop_car_init(p, h),
            2 => {
                // 0x8BF1E: drive in (the engine sound every frame); past camera X + 0x80 (signed) the wheels turn and it brakes.
                self.prop_car_sound_engine();
                p.b.motion();
                let lim = (self.cam.s2.cam_x as i16).wrapping_add(0x80);
                if lim < p.b.xi() {
                    self.enemy_anim(&mut p.b, 1);
                    p.b.ax = -0x30;
                    p.b.sub += 2;
                }
            }
            4 => {
                // 0x8BF52: brake until vx turns negative: stop, the teammate (0x971D6) and the caption (0x86D50), 120 frames.
                p.b.motion();
                if p.b.vx < 0 {
                    self.enemy_anim(&mut p.b, 0);
                    p.b.vx = 0;
                    p.b.ax = 0;
                    p.m.sw(o::T80, 0x3C);
                    self.prop_car_mate(p, p.m.b(o::SEAT) as u16);
                    self.prop_car_caption();
                    p.m.sw(o::T80, 0x78);
                    p.m.sb(o::SEAT, 0xFF);
                    p.b.sub += 2;
                }
            }
            6 => {
                // 0x8BFA6: 120 frames, the caption again, 60 frames.
                if self.prop_car_count(p) {
                    self.prop_car_caption();
                    p.m.sw(o::T80, 0x3C);
                    p.b.sub += 2;
                }
            }
            8 => {
                // 0x8BFCC: the seats (0x8C642) and the boarding jumps (0x8C6C2), 45 frames.
                if self.prop_car_count(p) {
                    self.prop_car_seats(p);
                    self.prop_car_board(p);
                    p.m.sw(o::T80, 0x2D);
                    p.b.sub += 2;
                }
            }
            0xA => {
                // 0x8BFEC: the seats are taken, the pads are free, the engine and the drive sounds, G.$516 = 1, accelerate.
                if self.prop_car_count(p) {
                    for k in 0..3 {
                        p.m.sb(o::SEAT + k, p.m.b(o::NEW_SEAT + k));
                    }
                    for pl in self.players.iter_mut() {
                        pl.input_lock = 0;
                    }
                    self.prop_car_sound_engine();
                    self.sound_ex(0x70, 0x10, 0); // 0xA4B2
                    self.flow.script.b516 = 1;
                    p.b.vx = 0;
                    p.b.ax = 0x40;
                    p.b.sub += 2;
                }
            }
            0xC => {
                // 0x8C03C: accelerate to 0x800 (unsigned), the heroes on the car.
                p.b.motion();
                self.prop_car_carry(p);
                if p.b.vx as u16 > 0x800 {
                    p.b.vx = 0x800;
                    p.b.ax = 0;
                    p.b.sub += 2;
                }
            }
            _ => {
                // 0x8C064: the drive starts: G.$518 = 1, the attack 0x66, mode 2, the clock and the kill-all end.
                p.b.motion();
                self.prop_car_carry(p);
                self.spawn.g.car_mode = 1;
                self.prop_car_arm(p);
                p.b.mode = 2;
                p.b.sub = 0;
                self.spawn.g.b4d5 = 0;
                self.flow.b4db = 0;
                self.spawn.g.kill_all = 0;
            }
        }
        let _ = i;
        self.prop_car_draw(p, h); // jmp 0x49C0
    }

    /// `subq.w #1,$80 ; bcc`: true when the timer word runs out (borrow).
    fn prop_car_count(&self, p: &mut Prop) -> bool {
        let t = p.m.w(o::T80);
        p.m.sw(o::T80, t.wrapping_sub(1));
        t == 0
    }

    /// 0xA49E: the engine sound 0x80.
    fn prop_car_sound_engine(&mut self) {
        self.sound_ex(0x80, 0x0007_2000, 0x0400_0000);
    }

    /// 0x8BE0C: the init of the three objects: not counted, `+$63` = 0xFF, on screen, no shadow (`+$25` = 0x10), the table 0x8C8D6, X = camera X
    /// minus 0xA0, height 5, depth 0x30. A wheel: `+$80` = 1, `+$A0` = 2, animation 2, mode 2. The car: G.$51A, animation 0, vx 0x500, empty seats, the
    /// box table 0x100000, HP 120 and the bar 100, the heroes' `+$119`, `+$FA` cleared and their pads held (`+$11A` = 1), the teammate (0x8C784),
    /// the seat drawers 0x96FD4 (three, two in the 2 player layout G.$4CF).
    fn prop_car_init(&mut self, p: &mut Prop, h: Handle) {
        p.m.sb(0x7D, 0xFF);
        p.b.b63 = 0xFF;
        p.b.visible = 1;
        p.b.draw_flags = 0x10;
        p.b.anim_table = 0x8C8D6;
        Self::put(&mut p.b.x, (self.cam.s2.cam_x as i16).wrapping_sub(0xA0));
        Self::put(&mut p.b.y, 5);
        Self::put(&mut p.b.z, 0x30);
        if p.b.var26 != 0 {
            p.m.sb(o::T80, 1);
            p.m.sb(o::A0, 2);
            self.enemy_anim(&mut p.b, 2);
            p.b.mode = p.b.mode.wrapping_add(2);
            return;
        }
        self.cam.car = h; // G.$51A
        self.enemy_anim(&mut p.b, 0);
        p.b.vx = 0x500;
        for k in 0..6 {
            p.m.sb(o::SEAT + k, 0xFF);
        }
        p.b.box_table = 0x100000;
        p.b.multi_box = false;
        p.b.hp = 0x78;
        p.b.hp_prev = 0x78;
        p.b.hp_bar = 0x64;
        p.m.sb(o::RIDERS, 0);
        for pl in self.players.iter_mut() {
            pl.b119 = 0;
        }
        p.m.sw(o::TVX, 0);
        p.m.sw(o::TVZ, 0);
        p.m.sw(o::BUMP, 0);
        p.m.sb(o::BUMP_T, 0x1E);
        for pl in self.players.iter_mut() {
            pl.run_t = 0;
        }
        for pl in self.players.iter_mut() {
            pl.input_lock = 1;
        }
        self.prop_car_mate_pick(p);
        p.m.sb(o::MATE, p.m.b(o::SEAT));
        self.prop_car_seat_drawer(h, 0);
        self.prop_car_seat_drawer(h, 1);
        if self.flow.b4cf == 0 {
            self.prop_car_seat_drawer(h, 2);
        }
        p.b.sub = p.b.sub.wrapping_add(2);
    }

    /// 0x96FD4: a pool C class 36 object (a seat drawer, M6-22): `+$58` = the car, variant `v`.
    fn prop_car_seat_drawer(&mut self, car: Handle, v: u16) {
        let Some(j) = self.effects.alloc() else { return };
        let e = &mut self.effects.slots[j];
        e.b.live = 1;
        e.b.k20 = 0x90;
        e.owner58 = car;
        e.b.var26 = v;
    }

    /// 0x971D6: a pool C class 37 object (the delivering teammate, M6-22) of the character `ch` at the car, 0x30 higher.
    fn prop_car_mate(&mut self, p: &Prop, ch: u16) {
        let Some(j) = self.effects.alloc() else { return };
        let e = &mut self.effects.slots[j];
        e.b.live = 1;
        e.b.k20 = 0x94;
        e.b.var26 = ch;
        Self::put(&mut e.b.x, p.b.xi());
        Self::put(&mut e.b.y, p.b.yi().wrapping_add(0x30));
        Self::put(&mut e.b.z, p.b.zi());
    }

    /// 0x86D50: a caption, pool B class 66 variant 0x68 (the caller's d0 / d1 are not read).
    fn prop_car_caption(&mut self) {
        let Some(j) = self.props.alloc() else { return };
        let c = &mut self.props.slots[j];
        c.b.live = 1;
        c.b.k20 = 0x108;
        c.b.var26 = 0x68;
    }

    /// 0x977D2: a pool C class 41 object (dust `kind` 0 with the palette 0x8E, or smoke 1, M6-22) at (x, y, z).
    fn prop_car_puff(&mut self, x: i16, y: i16, z: i16, kind: u8) {
        let Some(j) = self.effects.alloc() else { return };
        let e = &mut self.effects.slots[j];
        e.b.live = 1;
        e.b.k20 = 0xA4;
        Self::put(&mut e.b.x, x);
        Self::put(&mut e.b.y, y);
        Self::put(&mut e.b.z, z);
        if kind == 0 {
            e.b.palette_override = 0x8E;
        }
    }

    /// 0x8C784: the delivering teammate `+$A2`: the first character not in play, in the order 0, 1, 2 (3 when all three are in play).
    fn prop_car_mate_pick(&mut self, p: &mut Prop) {
        let mut mask = 0u8;
        for pl in &self.players {
            if pl.b.live != 0 {
                mask |= 1 << (pl.b.id & 7);
            }
        }
        let mut d1 = 0u8;
        if mask & 1 != 0 {
            d1 = 1;
            if mask & 2 != 0 {
                d1 = 2;
                if mask & 4 != 0 {
                    d1 = 3;
                }
            }
        }
        p.m.sb(o::SEAT, d1);
    }

    /// 0x8C642: the seats being built: the owner's bit in `+$A8` (bset.b of `+$02`), `+$A5` = his character, then the other heroes in play in
    /// the order P1, P2, P3 (`+$A6`, then `+$A7`; P1 writes `+$A6` without the test).
    fn prop_car_seats(&mut self, p: &mut Prop) {
        let d5 = p.b.who;
        p.m.sb(o::RIDERS, p.m.b(o::RIDERS) | 1 << (d5 & 7));
        p.m.sb(o::NEW_SEAT, p.b.id);
        for k in 0..3 {
            let pl = &self.players[k].b;
            if pl.live == 0 || pl.who == d5 {
                continue;
            }
            if k == 0 || (p.m.b(o::NEW_SEAT + 1) as i8) < 0 {
                p.m.sb(o::NEW_SEAT + 1, pl.id);
            } else {
                p.m.sb(o::NEW_SEAT + 2, pl.id);
            }
        }
    }

    /// 0x8C6C2 / 0x8C6E2: every hero in play jumps to the car (the scripted jump 0x1BBCA) at camera X + 0xB0, depth 0x30, facing right.
    /// In the original, the 68000 sets a1 = car before the loop over players 0..2, but player_scripted_jump_entry (0x1BBCA -> 0x13A0)
    /// clobbers a1 with ROM 0x11D220. Thus only the first live player compares against the car's +$A7; subsequent players compare against
    /// ROM byte 0 (which never matches their who >= 1).
    fn prop_car_board(&mut self, p: &Prop) {
        let mut a1_seat2 = p.m.b(o::NEW_SEAT + 2);
        for k in 0..3 {
            if self.players[k].b.live == 0 {
                continue;
            }
            let cam = self.cam.s2.cam_x;
            let pl = &mut self.players[k];
            pl.w88 = cam.wrapping_add(0xB0);
            pl.w8a = 0x30;
            pl.b.facing = 1;
            if pl.b.who == a1_seat2 {
                pl.w88 = pl.w88.wrapping_sub(0x40);
            }
            a1_seat2 = 0; // 0x13A0 clobbers a1 to ROM 0x11D220, where offset +$A7 is 0
            self.player_scripted_jump_entry(k);
        }
    }

    /// 0x8C610: the three heroes' positions (integer words) are the car's.
    fn prop_car_carry(&mut self, p: &Prop) {
        let (x, y, z) = (p.b.xi(), p.b.yi(), p.b.zi());
        for pl in self.players.iter_mut() {
            Self::put(&mut pl.b.x, x);
            Self::put(&mut pl.b.y, y);
            Self::put(&mut pl.b.z, z);
        }
    }

    // ------------------------------------------------------------------ the drive

    /// 0x8C0BC: the drive. A changed HP arms a new attack serial (not during the hit flash, `+$05` != 0); G.$517 = the riders; the motion; the riders (0x8C248), the seats (0x8C642),
    /// the steering (0x8C2A2, 0x8C20A), the band (0x8C7D6), the heroes on the car (0x8C610), the animation by the speed (0x8C4B6), the bump
    /// (0x8C41C), the smoke (0x8C3A6), G.$77A2 = the car and G.$77A0 = 1; then `+$05` (table 0x8C154): 0 the end tests and the low HP flash
    /// (0x8C158), 2 the hit flash (0x8C1EA).
    fn prop_car_drive(&mut self, p: &mut Prop, h: Handle) {
        let hp = p.b.hp;
        if p.b.sub == 0 && hp != p.b.hp_prev {
            p.b.hp_prev = hp;
            self.prop_car_arm(p);
        }
        self.spawn.g.car_riders = p.m.b(o::RIDERS); // G.$517
        p.b.motion();
        self.prop_car_riders(p);
        for k in 0..3 {
            p.m.sb(o::NEW_SEAT + k, 0xFF);
        }
        self.prop_car_seats(p);
        for k in 0..3 {
            p.m.sb(o::SEAT + k, p.m.b(o::NEW_SEAT + k));
        }
        self.prop_car_steer(p);
        self.prop_car_speed(p);
        self.prop_car_band(p);
        self.prop_car_carry(p);
        let a = if (p.m.w(o::TVX) as i16).wrapping_add(0x80) >= p.b.vx { 0 } else { 1 };
        self.enemy_anim(&mut p.b, a); // 0x8C4B6: jmp 0x120E every frame
        self.prop_car_bump(p);
        self.prop_car_smoke(p);
        self.spawn.g.car_obj = h; // G.$77A2
        self.spawn.g.car_on = 1; // G.$77A0
        if p.b.sub != 0 {
            // 0x8C1EA: the hit flash.
            p.b.palette_override = 0x84;
            p.b.hitstop = p.b.hitstop.wrapping_sub(1);
            if p.b.hitstop == 0 {
                p.b.palette_override = 0;
                p.b.mode = 2;
                p.b.sub = 0;
                p.b.live = 1;
            }
            return;
        }
        // 0x8C158: camera X at or past 0xA000 (unsigned) or the time over (G.$518 = 2) zero the HP; the boss death script (G.$4DD) ends it.
        if self.cam.s2.cam_x >= 0xA000 || self.spawn.g.car_mode == 2 {
            p.b.hp = 0;
            for pl in self.players.iter_mut() {
                pl.bc0 = 0;
            }
        }
        if self.w.scripted != 0 {
            p.b.set_state(0x0400_0000);
        }
        if p.b.act == 0 {
            // 0x8C19E
            if p.b.hp < 0x1E {
                p.b.act += 2;
            }
        } else {
            // 0x8C1AE: blink red 2 of 32 frames; at HP 0 the end.
            p.b.palette_override = 0;
            let t = p.m.b(o::T80).wrapping_add(1) & 0x1F;
            p.m.sb(o::T80, t);
            if t < 2 {
                p.b.palette_override = 0x84;
            }
            if p.b.hp == 0 {
                for pl in self.players.iter_mut() {
                    pl.bc0 = 0;
                }
                p.b.set_state(0x0400_0000);
            }
        }
    }

    /// 0x8C248: the riders: every hero in play (`+$00`) with a state (`+$04` not 0) sets his bit in `+$A8` and his `+$C0`.
    fn prop_car_riders(&mut self, p: &mut Prop) {
        p.m.sb(o::RIDERS, 0);
        for k in 0..3 {
            let pl = &mut self.players[k];
            if pl.b.live != 0 && pl.b.mode != 0 {
                p.m.sb(o::RIDERS, p.m.b(o::RIDERS) | 1 << k);
                pl.bc0 = 1;
            }
        }
    }

    /// 0x8C2A2: the target speed from the riders' sticks (0x8C346 each); nobody: 0x800 / 0; two: halved; three: times 6 / 16 (asr).
    fn prop_car_steer(&mut self, p: &mut Prop) {
        let mut d6 = 0u8;
        for k in 0..3 {
            if p.m.b(o::RIDERS) & (1 << k) != 0 {
                self.prop_car_stick(p, k, &mut d6);
            }
        }
        match d6 {
            0 => {
                p.m.sw(o::TVX, 0x800);
                p.m.sw(o::TVZ, 0);
            }
            1 => {}
            2 => {
                p.m.sw(o::TVX, ((p.m.w(o::TVX) as i16) >> 1) as u16);
                p.m.sw(o::TVZ, ((p.m.w(o::TVZ) as i16) >> 1) as u16);
            }
            3 => {
                let six = |v: u16| (((v as i16 as i32) * 6) >> 4) as u16;
                p.m.sw(o::TVX, six(p.m.w(o::TVX)));
                p.m.sw(o::TVZ, six(p.m.w(o::TVZ)));
            }
            _ => {}
        }
    }

    /// 0x8C346: one rider's stick. A new press of a direction (`+$A8`) sets his `+$119` = 0x3D; the held direction (`+$A0`) picks a pair
    /// (offset table 0x8C886) from 0x8C8B6, or from the equal copy 0x8C896 while `+$119` counts down. The first rider clears the sums; each adds
    /// his pair plus 0x800.
    fn prop_car_stick(&mut self, p: &mut Prop, k: usize, d6: &mut u8) {
        let pl = &mut self.players[k];
        if pl.pressed & 0xF != 0 {
            pl.b119 = 0x3D;
        }
        let d = pl.held & 0xF;
        if d == 0 {
            return;
        }
        let t = &self.assets.enemy.t;
        let off = t.u8(o::STICK, d as u32) as u32;
        let table = if pl.b119 != 0 {
            pl.b119 -= 1;
            o::PAIRS_B
        } else {
            o::PAIRS
        };
        if *d6 == 0 {
            p.m.sw(o::TVX, 0);
            p.m.sw(o::TVZ, 0);
        }
        let (dx, dz) = (t.u16(table, off), t.u16(table, off + 2));
        p.m.sw(o::TVX, p.m.w(o::TVX).wrapping_add(dx).wrapping_add(0x800));
        p.m.sw(o::TVZ, p.m.w(o::TVZ).wrapping_add(dz));
        *d6 += 1;
    }

    /// 0x8C20A: vz = the target at once; vx moves to the target by 0x30 a frame (a difference within -0x20 .. 0x1F is taken at once; the
    /// direction by an unsigned compare).
    fn prop_car_speed(&self, p: &mut Prop) {
        p.b.vz = p.m.w(o::TVZ) as i16;
        let tvx = p.m.w(o::TVX);
        let d0 = tvx.wrapping_sub(p.b.vx as u16).wrapping_add(0x20);
        if d0 < 0x40 {
            p.b.vx = tvx as i16;
        } else if tvx >= p.b.vx as u16 {
            p.b.vx = p.b.vx.wrapping_add(0x30);
        } else {
            p.b.vx = p.b.vx.wrapping_sub(0x30);
        }
    }

    /// 0x8C7D6: X within camera X + 0x30 .. + 0x140, Z within camera Y + 0x10 .. + 0x50 (integer words).
    fn prop_car_band(&self, p: &mut Prop) {
        // subi ; bmi -> apply ; subi (the width) ; bmi -> keep ; apply: sub.w d0 from the position.
        let pull = |pos: i16, cam: u16, lo: i16, width: i16| -> Option<i16> {
            let d0 = pos.wrapping_sub(cam as i16).wrapping_sub(lo);
            if d0 < 0 {
                return Some(d0);
            }
            let d0 = d0.wrapping_sub(width);
            if d0 < 0 { None } else { Some(d0) }
        };
        if let Some(d0) = pull(p.b.xi(), self.cam.s2.cam_x, 0x30, 0x110) {
            let v = p.b.xi().wrapping_sub(d0);
            Self::put(&mut p.b.x, v);
        }
        if let Some(d0) = pull(p.b.zi(), self.cam.s2.cam_y, 0x10, 0x40) {
            let v = p.b.zi().wrapping_sub(d0);
            Self::put(&mut p.b.z, v);
        }
    }

    /// 0x8C41C: the body bump: height 5 plus the step `+$AE` of the table 0x8C404 (counting down). With no step running a new one of 15 starts
    /// in mode 4 and above, in the hit reaction (0x0202), below 20 HP, on the screens 4 and 5 (0x214F0), or at random (`+$A9` runs out: a new
    /// `rand() & 15`, then `rand() & 1` = 0).
    fn prop_car_bump(&mut self, p: &mut Prop) {
        Self::put(&mut p.b.y, 5);
        let d0 = p.m.w(o::BUMP);
        if d0 != 0 {
            let d1 = self.assets.enemy.t.u8(o::BUMPS, d0 as u32) as i8 as i16;
            let v = p.b.yi().wrapping_add(d1);
            Self::put(&mut p.b.y, v);
            p.m.sw(o::BUMP, d0.wrapping_sub(1));
            return;
        }
        let start = if p.b.mode >= 4 || (p.b.mode, p.b.sub) == (2, 2) || p.b.hp < 0x14 {
            true
        } else {
            let id = self.screen_id2(p.b.xi() as u16, p.b.zi() as u16);
            if id == 4 || id == 5 {
                true
            } else {
                let t = p.m.b(o::BUMP_T);
                p.m.sb(o::BUMP_T, t.wrapping_sub(1));
                if t != 0 {
                    return;
                }
                let r = self.rand() & 0xF;
                p.m.sb(o::BUMP_T, r as u8);
                self.rand() & 1 == 0
            }
        };
        if start {
            p.m.sw(o::BUMP, 0xF);
        }
    }

    /// 0x8C3A6: every 8 frames (G.$0) below 60 HP a smoke puff at X + 0x40, height + 0x20; below 30 a second at X - 0x20, height + 0x10.
    fn prop_car_smoke(&mut self, p: &Prop) {
        if self.w.frame & 7 != 0 || p.b.hp >= 0x3C {
            return;
        }
        let (x, y, z) = (p.b.xi(), p.b.yi(), p.b.zi().wrapping_sub(4));
        self.prop_car_puff(x.wrapping_add(0x40), y.wrapping_add(0x20), z, 1);
        if p.b.hp < 0x1E {
            self.prop_car_puff(x.wrapping_sub(0x20), y.wrapping_add(0x10), z, 1);
        }
    }

    // ------------------------------------------------------------------ the wheels

    /// 0x8C4D0: a wheel follows the car (X - 0x4B rear, X + 0x4C front, Z + 1, height 0) and is freed when the car is gone; while the car
    /// moves it turns, throws dust (0x8C5D2) and hops (`rand() & 1`). The front wheel shows the steering (table 0x8C5CC by `+$A0`).
    fn prop_car_wheel(&mut self, p: &mut Prop, i: usize) {
        let car = match slot_of(p.m.w(o::CAR)) {
            Some(Slot::Prop(c)) => self.props.slots[c].b.clone(),
            _ => crate::game::body::Body::default(),
        };
        if car.live == 0 || car.k20 != CAR_K20 {
            return self.prop_free(p, i); // 0x8C880
        }
        Self::put(&mut p.b.x, car.xi());
        Self::put(&mut p.b.z, car.zi().wrapping_add(1));
        Self::put(&mut p.b.y, 0);
        let dx = if p.b.var26 == 2 { 0x4C } else { -0x4B };
        let v = p.b.xi().wrapping_add(dx);
            Self::put(&mut p.b.x, v);
        if car.vx != 0 {
            self.enemy_step(&mut p.b);
            self.prop_car_dust(p, &car);
            let r = self.rand() & 1;
            Self::put(&mut p.b.y, r as i16);
        }
        if p.b.var26 != 2 {
            return;
        }
        p.b.vz = car.vz;
        let a0 = p.m.b(o::A0);
        if p.b.vz == 0 {
            if a0 == 2 {
                return;
            }
            let t = p.m.b(o::T80);
            p.m.sb(o::T80, t.wrapping_sub(2));
            if t >= 2 {
                return;
            }
            // cmpi.b #2 ; bcc: below 2 one up, else one down (a0 is 3 or more here, so the borrow branch is not reached).
            p.m.sb(o::A0, if a0 < 2 { a0 + 1 } else { a0 - 1 });
        } else {
            let t = p.m.b(o::T80);
            p.m.sb(o::T80, t.wrapping_sub(1));
            if t != 0 {
                return;
            }
            if p.b.vz >= 0 {
                let n = a0.wrapping_add(1);
                p.m.sb(o::A0, if n < 5 { n } else { 4 });
            } else {
                p.m.sb(o::A0, if a0 == 0 { 0 } else { a0 - 1 });
            }
        }
        let a = self.assets.enemy.t.u8(o::WHEEL_ANIM, p.m.b(o::A0) as u32);
        self.enemy_anim(&mut p.b, a as u16);
        p.m.sb(o::T80, 0xC);
    }

    /// 0x8C5D2: dust under a wheel when the car is at 3 px per frame or more (unsigned): every 8 frames at 8 px or more, else every 16.
    fn prop_car_dust(&mut self, p: &Prop, car: &crate::game::body::Body) {
        if (car.vx as u16) < 0x300 {
            return;
        }
        let m = if car.vx as u16 >= 0x800 { 7 } else { 0xF };
        if self.w.frame & m != 0 {
            return;
        }
        self.prop_car_puff(p.b.xi(), p.b.yi(), p.b.zi().wrapping_sub(4), 0);
    }

    // ------------------------------------------------------------------ the end

    /// 0x8C810: mode 4 (table 0x8C82E): step 0 (0x8C832) no flash, empty seats, no acceleration and Z motion, G.$516 = G.$518 = 0, the sound 0x8F
    /// (0xA4BE), then every hero in play jumps down (0x8C71C): X camera X + 0x80 (+ 0x20 for P2, + 0x40 for P3), depth 0x30, untouchable 120
    /// frames. Step 2 (0x8C868): past camera X + 0x200 (unsigned) mode 6. Then the bump (0x8C41C), the motion, the draw.
    fn prop_car_end(&mut self, p: &mut Prop, h: Handle) {
        if p.b.sub == 0 {
            p.b.palette_override = 0;
            for k in 0..3 {
                p.m.sb(o::SEAT + k, 0xFF);
            }
            p.b.ax = 0;
            p.b.vz = 0;
            p.b.az = 0;
            self.flow.script.b516 = 0;
            self.spawn.g.car_mode = 0;
            self.sound_ex(0x8F, 0x10, 0); // 0xA4BE
            p.b.sub = p.b.sub.wrapping_add(2);
            for k in 0..3 {
                if self.players[k].b.live == 0 {
                    continue;
                }
                let mut x = self.cam.s2.cam_x.wrapping_add(0x80);
                let who = self.players[k].b.who;
                if who != 0 {
                    x = x.wrapping_add(0x20);
                    if who != 1 {
                        x = x.wrapping_add(0x20);
                    }
                }
                let pl = &mut self.players[k];
                pl.w88 = x;
                pl.w8a = 0x30;
                pl.b.live = 2;
                pl.invuln = 0x78;
                self.player_scripted_jump_entry(k);
            }
        } else if self.cam.s2.cam_x.wrapping_add(0x200) < p.b.xi() as u16 {
            p.b.mode = 6;
            p.b.sub = 0;
        }
        self.prop_car_bump(p);
        p.b.motion();
        self.prop_car_draw(p, h);
    }

    // ------------------------------------------------------------------ the passes of the drive (M6-20)

    /// `+$7A` of the object `h` (cleared by the passes: the wait of an enemy, a container's byte).
    fn car_clear_7a(&mut self, h: Handle) {
        match slot_of(h) {
            Some(Slot::Enemy(i)) => self.enemies.slots[i].m.sb(0x7A, 0),
            Some(Slot::Item(i)) => self.items.slots[i].m.sb(0x7A, 0),
            Some(Slot::Prop(i)) => self.props.slots[i].m.sb(0x7A, 0),
            _ => {}
        }
    }

    /// 0xFF42: the score `d0` (a byte offset into the table 0x10292, sign extended) for every hero in play with a state (`+$00`, `+$04`).
    fn car_score_all(&mut self, d0: u8) {
        let v = self.assets.enemy.t.u32(0x10292, d0 as i8 as i32 as u32);
        for k in 0..3 {
            let b = &self.players[k].b;
            if b.live != 0 && b.mode != 0 {
                self.enemy_add_score(k as u8, v);
            }
        }
        self.hud.score_dirty = true;
    }

    /// The common part of 0x12236 / 0x123A0: the car (`a2`) hits `a3` with its attack: serial and credit of the car, `+$00` = 2, state 0x0202,
    /// the attacker, the record of the car's attack id (damage down to 0, poise down to 0, the reaction, hit stop, sound). Returns the record.
    fn car_hit_common(&mut self, a2: Handle, a3: Handle) -> dino_assets::sprites::AttackRecord {
        let car = self.body(a2).clone();
        let rec = self.assets.player.enemy_attacks[car.atk as usize];
        self.car_clear_7a(a3);
        let v = self.body_mut(a3);
        v.serial = car.serial;
        v.who = car.who;
        v.live = 2;
        v.set_state(0x0202_0000);
        v.attacker = a2;
        v.hp_prev = v.hp;
        let hp = (v.hp as i16).wrapping_sub(rec.damage as u8 as i16);
        v.hp = if hp > 0 { hp as u16 } else { 0 };
        v.stun = v.stun.saturating_sub(rec.stun);
        v.reaction = rec.reaction;
        rec
    }

    /// 0x121D8: the car against the hurtable enemies (list 0x671E): skips the untouchable (`+$00` bit 1) and those in hit stop; no car attack box
    /// ends the pass; the frame classes 0, 2, 4, 6 and 0xC are hit (table 0x12226). A hit (0x12236): 20 flat with the car's credit, the reaction
    /// of the record as the state's act, the sound, the score 0x1C for every hero (0xFF42), the spark (0x12720), the victim faces away.
    pub fn pass_121d8(&mut self) {
        let a2 = self.spawn.g.car_obj;
        for a3 in self.combat.get(0x671E).to_vec() {
            let d = self.body(a3).clone();
            if d.live & 2 != 0 || d.hitstop != 0 {
                continue;
            }
            let car = self.body(a2).clone();
            if car.attack_box == 0 {
                return;
            }
            if d.body_box == 0 || !matches!(d.kind, 0 | 2 | 4 | 6 | 0xC) {
                continue;
            }
            let Some((ha, hb)) = collide::boxes_hit(&self.assets.sprites, &car, car.attack_box, &d, d.body_box) else { continue };
            let a6 = self.snd_a6_128f8(&car, &d, d.body_box, &hb); // 0x12236: the box test leaves a6 on the box record
            let rec = self.car_hit_common(a2, a3);
            let v = self.body_mut(a3);
            v.act = rec.reaction;
            v.step = 0;
            v.hitstop = rec.hit_stop;
            if rec.sound != 0 {
                self.snd_hit(rec.sound, car.xi(), self.snd_a6_byte(a6)); // 0x122AC -> 0x962A
            }
            self.car_score_all(0x1C);
            self.body_mut(a3).b9a = rec.b7;
            self.spawn_hit_spark(a2, a3, ha, hb, rec.spark); // 0x12720
            self.body_mut(a3).facing = car.facing ^ 1;
        }
    }

    /// 0x122D8: the car's body against the containers (list 0x6AFC, frame class 0): HP 0, no credit (`+$02` = 0xFF), the spark, the score 0x14
    /// for every hero, facing 0.
    pub fn pass_122d8(&mut self) {
        let a2 = self.spawn.g.car_obj;
        let car = self.body(a2).clone();
        if car.body_box == 0 {
            return;
        }
        // d0 is loaded once (0x122DE) and 0x128F8 leaves its own arithmetic in it, 0xFF42 leaves 0x14 after a hit: from the second
        // container on, the car's box is the one that this leftover word picks (a quirk of the original, kept).
        let mut d0 = car.body_box as u16;
        for a3 in self.combat.get(0x6AFC).to_vec() {
            let d = self.body(a3).clone();
            if d.body_box == 0 || d.kind != 0 {
                continue;
            }
            let (hit, left) = self.car_box_test(&car, d0, &d, d.body_box);
            // 0x12916: `mulu.w #$c,d0` leaves the product's high word in d0; the word moves after it keep it, so the spark sees it.
            let d0_hi = ((d0 as u32).wrapping_mul(12) >> 16) as u16;
            d0 = left;
            let Some((ha, hb)) = hit else { continue };
            {
                let v = self.body_mut(a3);
                v.live = 2;
                v.who = 0xFF;
            }
            self.car_clear_7a(a3);
            let v = self.body_mut(a3);
            v.attacker = a2;
            v.hp = 0;
            self.spawn_hit_spark_d0(a2, a3, ha, hb, 0, d0_hi); // 0x12720 with d5 = 0
            self.car_score_all(0x14);
            d0 = 0x14;
            self.body_mut(a3).facing = 0;
        }
    }

    /// 0x128F8 for two single box objects (variant 0x12916, `+$4A` = 0 on both: the car and the containers): the attacker box at
    /// table + sext(d0 * 12) (the low word of the `mulu.w`, added with `adda.w`), the defender box `ib`; the Z, X and Y windows in that order
    /// (unsigned, touching edges hit). Returns the pair on a hit and the word the routine leaves in d0 (the last window's difference).
    fn car_box_test(&self, a: &crate::game::body::Body, d0: u16, b: &crate::game::body::Body, ib: u8) -> (Option<(dino_assets::HitBox, dino_assets::HitBox)>, u16) {
        assert!(!a.multi_box && !b.multi_box, "0x128F8 variants with multi boxes are not needed by the car passes");
        let bank = &self.assets.sprites.hitboxes;
        let off = (d0 as u32).wrapping_mul(12) as u16 as i16 as i32 as u32;
        let Some(ha) = bank.try_at((a.box_table & 0xFF_FFFF).wrapping_add(off)) else { return (None, d0) };
        let hb = bank.single(b.box_table & 0xFF_FFFF, ib);
        let za = a.zi().wrapping_add(ha.zoff);
        let zb = b.zi().wrapping_add(hb.zoff);
        let d = za.wrapping_sub(zb).wrapping_add(ha.zsize) as u16;
        if d > ha.zsize.wrapping_add(hb.zsize) as u16 {
            return (None, d);
        }
        let d = collide::left(a, &ha).wrapping_sub(collide::left(b, &hb)).wrapping_add(ha.xsize) as u16;
        if d > ha.xsize.wrapping_add(hb.xsize) as u16 {
            return (None, d);
        }
        let ya = a.yi().wrapping_add(ha.yoff);
        let yb = b.yi().wrapping_add(hb.yoff);
        let d = ya.wrapping_sub(yb).wrapping_add(ha.ysize) as u16;
        if d > ha.ysize.wrapping_add(hb.ysize) as u16 {
            return (None, d);
        }
        (Some((ha, hb)), d)
    }

    /// 0x12342: the car against the monsters (list 0x6B64): as 0x121D8 with the table 0x12390 (frame classes 0, 2, 4, 6, 0xC); the act is the
    /// reaction mapped by 0x11CC4, the score is the record's, and Hogg (class 31) keeps his facing.
    pub fn pass_12342(&mut self) {
        let a2 = self.spawn.g.car_obj;
        for a3 in self.combat.get(0x6B64).to_vec() {
            let d = self.body(a3).clone();
            if d.live & 2 != 0 || d.hitstop != 0 {
                continue;
            }
            let car = self.body(a2).clone();
            if car.attack_box == 0 {
                return;
            }
            if d.body_box == 0 || !matches!(d.kind, 0 | 2 | 4 | 6 | 0xC) {
                continue;
            }
            let Some((ha, hb)) = collide::boxes_hit(&self.assets.sprites, &car, car.attack_box, &d, d.body_box) else { continue };
            let rec = self.car_hit_common(a2, a3);
            let act = self.assets.enemy.t.u16(0x11CC4, rec.reaction as u32) as u8;
            let v = self.body_mut(a3);
            v.act = act;
            v.step = 0;
            v.hitstop = rec.hit_stop;
            if rec.sound != 0 {
                // 0x12420: a6 holds the reaction table 0x11CC4 (0x12406)
                self.snd_hit(rec.sound, car.xi(), self.snd_a6_byte(0x11CC4)); // 0x962A
            }
            self.car_score_all(rec.score);
            self.body_mut(a3).b9a = rec.b7;
            self.spawn_hit_spark(a2, a3, ha, hb, rec.spark); // 0x12720
            if d.k20 != 0x7C {
                self.body_mut(a3).facing = car.facing ^ 1;
            }
        }
    }

    /// 0x12454: the monsters' attacks against the car (only while G.$7B4 is set: after Hogg exists). The car must be touchable; a monster in an
    /// attack frame (class 4) with another serial and an attack box hits the car's body box (no body box ends the pass): the car takes the
    /// serial, `+$00` = 2, state 0x0202 (the hit flash, 0x8C1EA), the attacker, the record's damage (down to 0; the debug invincibility of
    /// G.$4AA is not ported), hit stop, sound and spark. Nothing else of the record is used.
    pub fn pass_12454(&mut self) {
        let a3 = self.spawn.g.car_obj;
        if self.body(a3).live & 2 != 0 {
            return;
        }
        for a2 in self.combat.get(0x6B64).to_vec() {
            let m = self.body(a2).clone();
            let car = self.body(a3).clone();
            if m.kind != 4 || m.serial == car.serial || m.attack_box == 0 {
                continue;
            }
            if car.body_box == 0 {
                return;
            }
            let Some((ha, hb)) = collide::boxes_hit(&self.assets.sprites, &m, m.attack_box, &car, car.body_box) else { continue };
            let rec = self.assets.player.enemy_attacks[m.atk as usize];
            self.car_clear_7a(a3);
            let v = self.body_mut(a3);
            v.serial = m.serial;
            v.live = 2;
            v.set_state(0x0202_0000);
            v.attacker = a2;
            v.hp_prev = v.hp;
            let hp = (v.hp as i16).wrapping_sub(rec.damage as u8 as i16);
            v.hp = if hp > 0 { hp as u16 } else { 0 };
            v.step = 0;
            v.hitstop = rec.hit_stop;
            if rec.sound != 0 {
                // 0x12514: a6 is the box record of the box test (0x124A6)
                let a6 = self.snd_a6_128f8(&m, &car, car.body_box, &hb);
                self.snd_hit(rec.sound, m.xi(), self.snd_a6_byte(a6)); // 0x962A
            }
            self.spawn_hit_spark(a2, a3, ha, hb, rec.spark); // 0x12720
        }
    }
}
