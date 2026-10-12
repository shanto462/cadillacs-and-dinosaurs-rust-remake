//! Class 20: the choice machine of action 4 and the small routines (slots, steering, windows).

use super::class20::{f, t};
use super::{Enemy, off::*};
use crate::game::{Game, Handle};

impl Game {
    /// 0xA062-like sound 0x9C20 for an object that is no player: 0xE0 / 0xE1 (toggle bit 6 of G.$43A).
    pub(super) fn snd_9c20(&mut self) {
        let old = self.w.snd_toggle >> 6 & 1 != 0;
        self.w.snd_toggle ^= 1 << 6;
        self.sound(if old { 0xE0 } else { 0xE1 });
    }

    // ------------------------------------------------------------------ action 4: the choice machine

    /// 0x460B4: action 4 starts: pick a sub action (+$A5) different from the last.
    pub(super) fn c20_460b4(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sw(f::ACTION, 4);
        self.c20_460bc(e, h);
    }

    /// 0x460BC.
    fn c20_460bc(&mut self, e: &mut Enemy, h: Handle) {
        loop {
            e.b.step = 0;
            let r = (self.rand() & 0xF) as u32;
            let d1 = self.assets.enemy.t.u8(t::CHOICES, r);
            if d1 == e.m.b(f::CHOICE) {
                continue;
            }
            e.m.sb(f::CHOICE, d1);
            e.m.sb(f::BUSY, 1);
            break;
        }
        self.c20_460de(e, h);
    }

    /// 0x460DE.
    pub(super) fn c20_460de(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.b(f::BUSY) == 0 {
            return self.c20_460bc(e, h);
        }
        match e.m.b(f::CHOICE) {
            0 => self.c20_46108(e),
            2 => self.c20_46198(e),
            4 => self.c20_46252(e),
            v => panic!("class 20 choice {v:#x}"),
        }
    }

    /// 0x46108 choice 0: two jumps in place.
    fn c20_46108(&mut self, e: &mut Enemy) {
        match e.b.step {
            0 => {
                // 0x4611C
                e.m.sb(f::COUNT, 2);
                self.c20_46122(e);
            }
            2 => self.c20_46158(e),
            4 => self.c20_46184(e),
            s => panic!("class 20 choice 0 step {s:#x}"),
        }
    }

    fn c20_46122(&mut self, e: &mut Enemy) {
        e.b.step = 2;
        e.m.sb(f::BUSY, 1);
        self.enemy_anim(&mut e.b, 8);
        e.b.airborne = 1;
        e.b.vx = 0;
        e.b.ax = 0;
        e.b.vy = 0x600;
        e.b.ay = 0xFFB0u16 as i16;
        e.b.vz = 0;
        e.b.az = 0;
        self.c20_46158(e);
    }

    fn c20_46158(&mut self, e: &mut Enemy) {
        if e.b.airborne != 0 {
            if e.b.param_b == 0 {
                e.b.motion();
            }
            self.enemy_step(&mut e.b);
            return;
        }
        self.snd_9c20();
        e.b.step = 4;
        self.enemy_anim(&mut e.b, 9);
        self.c20_46184(e);
    }

    fn c20_46184(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            e.m.sb(f::BUSY, 0);
            let n = e.m.b(f::COUNT).wrapping_sub(1);
            e.m.sb(f::COUNT, n);
            if n != 0 {
                self.c20_46122(e);
            }
        }
    }

    /// 0x46198 choice 2: walk to a point beside the target for 0xB4 frames, then strafe.
    fn c20_46198(&mut self, e: &mut Enemy) {
        match e.b.step {
            0 => {
                // 0x461AC
                e.b.step = 2;
                e.m.sb(f::BUSY, 1);
                self.enemy_anim(&mut e.b, 1);
                self.enemy_set_speed(&mut e.b, 0xB);
                e.m.sb(f::T_B1, 0xB4);
                let tp = self.c20_target(e);
                let mut d1 = 0x90i16;
                if e.b.facing != 0 {
                    d1 = -d1;
                }
                let x = self.players[tp].b.xi().wrapping_add(d1);
                e.m.sw(f::DEST_X, x as u16);
                e.m.sw(f::DEST_Z, e.b.zi() as u16);
                self.c20_46bcc(e);
                self.c20_461f2(e);
            }
            2 => self.c20_461f2(e),
            4 => self.c20_46236(e),
            s => panic!("class 20 choice 2 step {s:#x}"),
        }
    }

    fn c20_461f2(&mut self, e: &mut Enemy) {
        let b = e.m.b(f::T_B1);
        e.m.sb(f::T_B1, b.wrapping_sub(1));
        if b == 0 || self.c20_46d18(e) {
            // 0x4620A
            e.b.step = 4;
            e.m.sb(f::A8, 8);
            let d1: u8 = if e.b.dir & 8 != 0 { 2 } else { 0xFE };
            e.m.sb(f::STRAFE, d1);
            let d5 = e.b.dir;
            self.c20_46b96(e);
            e.b.dir = d5;
            self.c20_46a80(e);
            return self.c20_46236(e);
        }
        self.enemy_move_dir(&mut e.b);
        self.enemy_walk_anim_c(&mut e.b);
    }

    fn c20_46236(&mut self, e: &mut Enemy) {
        if self.c20_46be0(e) {
            e.m.sb(f::BUSY, 0); // 0x4624C
            return;
        }
        self.c20_46ab6(e);
        self.enemy_move_dir(&mut e.b);
        self.enemy_walk_anim_c(&mut e.b);
    }

    /// 0x46252 choice 4: walk to the slot, then jump at the target.
    fn c20_46252(&mut self, e: &mut Enemy) {
        match e.b.step {
            0 => {
                // 0x46268
                e.b.step += 2;
                e.m.sb(f::BUSY, 1);
                self.enemy_anim(&mut e.b, 1);
                self.enemy_set_speed(&mut e.b, 0xB);
                self.c20_46282(e);
            }
            2 => self.c20_46282(e),
            4 => self.c20_462de(e),
            6 => self.c20_46310(e),
            s => panic!("class 20 choice 4 step {s:#x}"),
        }
    }

    fn c20_46282(&mut self, e: &mut Enemy) {
        if self.c20_46d18(e) {
            // 0x462D0
            e.b.step += 2;
            self.c20_46298(e);
            self.enemy_anim(&mut e.b, 0xE);
            return self.c20_462de(e);
        }
        self.c20_46ab6(e);
        self.enemy_move_dir(&mut e.b);
        self.enemy_walk_anim_c(&mut e.b);
    }

    /// 0x46298: the jump at the target (vx +-0x380, vy 0x780, ay -0x50).
    fn c20_46298(&mut self, e: &mut Enemy) {
        let tp = self.c20_target(e);
        let mut d1 = 0x380i16;
        if (self.players[tp].b.xi() as u16) < (e.b.xi() as u16) {
            d1 = -d1;
        }
        e.b.airborne = 1;
        e.b.vx = d1;
        e.b.ax = 0;
        e.b.vy = 0x780;
        e.b.ay = 0xFFB0u16 as i16;
        e.b.vz = 0;
        e.b.az = 0;
    }

    fn c20_462de(&mut self, e: &mut Enemy) {
        if e.b.airborne != 0 {
            if e.b.param_b == 0 {
                e.b.motion();
            }
            self.enemy_step(&mut e.b);
            return;
        }
        e.b.step += 2;
        self.c20_46a80(e);
        self.c20_46d42(e);
        self.snd_9c20();
        self.enemy_anim(&mut e.b, 0xF);
        self.c20_46310(e);
    }

    fn c20_46310(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            self.c20_46d00(e);
            e.m.sb(f::BUSY, 0);
        }
    }

    /// 0x46060: a hop straight up.
    pub(super) fn c20_46060(&mut self, e: &mut Enemy) {
        e.b.airborne = 1;
        e.b.vx = 0;
        e.b.ax = 0;
        e.b.vy = 0x600;
        e.b.ay = 0xFFB0u16 as i16;
        e.b.vz = 0;
        e.b.az = 0;
    }

    /// 0x46084: a jump at the target (vx +-0x280, vy 0x780).
    pub(super) fn c20_46084(&mut self, e: &mut Enemy) {
        let tp = self.c20_target(e);
        let mut d1 = 0x280i16;
        if (self.players[tp].b.xi() as u16) < (e.b.xi() as u16) {
            d1 = -d1;
        }
        e.b.airborne = 1;
        e.b.vx = d1;
        e.b.ax = 0;
        e.b.vy = 0x780;
        e.b.ay = 0xFFB0u16 as i16;
    }

    // ------------------------------------------------------------------ slots and steering

    /// 0x46A72: choose the target, then the slot (0x46A80).
    pub(super) fn c20_46a72(&mut self, e: &mut Enemy) {
        let tg = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, tg);
        self.c20_46a80(e);
    }

    /// 0x46A80: slot 0..5 round the target (X side * 3 + Z side + Z beyond 12).
    pub(super) fn c20_46a80(&mut self, e: &mut Enemy) {
        let tp = self.c20_target(e);
        let t = &self.players[tp].b;
        let mut d1 = 0u8;
        let mut d2 = 0xCi16;
        if !((e.b.xi() as i16) < (t.xi() as i16)) {
            d1 = 3;
        }
        if (e.b.zi() as u16) < (t.zi() as u16) {
            d1 += 1;
            d2 = -d2;
        }
        if !((t.zi().wrapping_add(d2) as u16) < (e.b.zi() as u16)) {
            d1 += 1;
        }
        e.m.sb(f::SLOT, d1);
    }

    fn c20_slot_point(&self, e: &Enemy) -> (i16, i16) {
        let tp = self.c20_target(e);
        let slot = e.m.b(f::SLOT) as u32 * 4;
        let (dx, dz) = (self.assets.enemy.t.i16(t::SLOTS, slot), self.assets.enemy.t.i16(t::SLOTS, slot + 2));
        let tb = &self.players[tp].b;
        (dx.wrapping_add(tb.xi()), dz.wrapping_add(tb.zi()))
    }

    /// 0x46B96: the point of the slot -> +$88 / +$8A and the direction straight at it.
    pub(super) fn c20_46b96(&mut self, e: &mut Enemy) {
        let (d1, d2) = self.c20_slot_point(e);
        e.m.sw(f::DEST_X, d1 as u16);
        e.m.sw(f::DEST_Z, d2 as u16);
        e.b.dir = Self::enemy_angle_to(&e.b, d1 as u16, d2 as u16);
    }

    /// 0x46AB6: the point of the slot, turn towards it (0x46B0E), face the target.
    pub(super) fn c20_46ab6(&mut self, e: &mut Enemy) {
        let (d1, d2) = self.c20_slot_point(e);
        e.m.sw(f::DEST_X, d1 as u16);
        e.m.sw(f::DEST_Z, d2 as u16);
        self.c20_46b0e(e, d1 as u16, d2 as u16);
        let tp = self.c20_target(e);
        let borrow = (self.players[tp].b.xi() as u16) < (e.b.xi() as u16);
        e.b.facing = if borrow { 0 } else { 1 };
    }

    /// 0x46B0E: direction to (tx, tz); when it is more than 0xC steps off (byte compare of
    /// `dir - d0 + 6`), turn by +6 (the branch for -6 can never be taken: d1 > 0xC >= 0xA).
    fn c20_46b0e(&mut self, e: &mut Enemy, tx: u16, tz: u16) {
        let mut d0 = Self::enemy_angle_to(&e.b, tx, tz);
        let d1 = e.b.dir.wrapping_sub(d0).wrapping_add(6) & 0x1F;
        if d1 > 0xC {
            let d2: u8 = if d1 >= 0xA { 6 } else { 0xFA };
            d0 = e.b.dir.wrapping_add(d2);
        }
        e.b.dir = d0 & 0x1F;
    }

    /// 0x46BCC: the direction at +$88 / +$8A.
    pub(super) fn c20_46bcc(&mut self, e: &mut Enemy) {
        let (d1, d2) = (e.m.w(f::DEST_X), e.m.w(f::DEST_Z));
        e.b.dir = Self::enemy_angle_to(&e.b, d1, d2);
    }

    /// 0x46BE0: at 0x80 in front of the target (window 0x60 x 0x40)? Returns the carry.
    pub(super) fn c20_46be0(&self, e: &Enemy) -> bool {
        let t = &self.players[self.c20_target(e)].b;
        let mut d1 = 0x80i16;
        if (e.b.xi() as u16) < (t.xi() as u16) {
            d1 = -d1;
        }
        let dx = t.xi().wrapping_add(d1).wrapping_sub(e.b.xi()).wrapping_add(0x30) as u16;
        let dz = t.zi().wrapping_sub(e.b.zi()).wrapping_add(0x20) as u16;
        dx < 0x60 && dz < 0x40
    }

    /// 0x46C24: keep a distance of 0x80 in X (the position at 0x30 / 0x20 margins decides the turn).
    pub(super) fn c20_46c24(&mut self, e: &mut Enemy) {
        let tp = self.c20_target(e);
        let t = self.players[tp].b.clone();
        let mut d1 = 0x80i16;
        if (e.b.xi() as u16) < (t.xi() as u16) {
            d1 = -d1;
        }
        let d0 = t.xi().wrapping_add(d1);
        let dx = d0.wrapping_sub(e.b.xi());
        let dir = e.b.dir;
        let d1b: u8;
        if dx >= 0x30 {
            // 0x46CBA
            let mut v = 0x20u8.wrapping_sub(dir);
            v &= !0x10;
            d1b = v;
        } else if dx.wrapping_add(0x30) <= 0 {
            // 0x46CCA
            let mut v = 0x20u8.wrapping_sub(dir);
            v |= 0x10;
            d1b = v;
        } else {
            let d0z = t.zi();
            let dz = d0z.wrapping_sub(e.b.zi());
            if dz >= 0x20 {
                // 0x46C96
                let v = 0x18u8.wrapping_sub(dir).wrapping_sub(8);
                let mut d2 = v.wrapping_sub(8) & 0x1F;
                d1b = if d2 < 0x10 {
                    v // bcs 0x46C94: d1 as it is
                } else {
                    d2 &= !0x10;
                    d2.wrapping_sub(8)
                };
            } else if dz.wrapping_add(0x20) <= 0 {
                // 0x46C74
                let v = 8u8.wrapping_sub(dir).wrapping_sub(0x18);
                let mut d2 = v.wrapping_sub(8);
                d1b = if d2 < 0x10 {
                    v
                } else {
                    d2 &= !0x10;
                    d2.wrapping_add(8)
                };
            } else {
                return;
            }
        }
        // 0x46CD8: a little noise from the table 0x46CF0
        let r = (self.rand() & 0xF) as u32;
        let n = self.assets.enemy.t.u8(t::CIRCLE, r);
        e.b.dir = d1b.wrapping_add(n) & 0x1F;
    }

    /// 0x46D00: face the target.
    pub(super) fn c20_46d00(&mut self, e: &mut Enemy) {
        let tp = self.c20_target(e);
        e.b.facing = if (e.b.xi() as u16) < (self.players[tp].b.xi() as u16) { 1 } else { 0 };
    }

    /// 0x46D18: at the point +$88 / +$8A (window 0xE x 0xA, offsets 7 and 2)? Returns the carry.
    pub(super) fn c20_46d18(&self, e: &Enemy) -> bool {
        let dx = e.b.xi().wrapping_sub(e.m.w(f::DEST_X) as i16).wrapping_add(7) as u16;
        let dz = e.b.zi().wrapping_sub(e.m.w(f::DEST_Z) as i16).wrapping_add(2) as u16;
        dx < 0xE && dz < 0xA
    }

    /// 0x46D42: the point of the slot -> +$88 / +$8A (no direction).
    pub(super) fn c20_46d42(&mut self, e: &mut Enemy) {
        let (d1, d2) = self.c20_slot_point(e);
        e.m.sw(f::DEST_X, d1 as u16);
        e.m.sw(f::DEST_Z, d2 as u16);
    }

    /// 0x46D6E: chance to start an attack: `rand() & 0x1F < table[variant * 16 + level]`. The carry.
    pub(super) fn c20_46d6e(&mut self, e: &mut Enemy) -> bool {
        let row = ((e.b.var26 >> 8) as u32 & 0xFF) << 4;
        let d1 = self.assets.enemy.t.u8(t::CHANCE, row + self.w.level as u32) as u16;
        let r = self.rand() & 0x1F;
        r < d1
    }

    /// 0x46DD8: the point of the offset table 0x46E22 by `+$AA` (mirrored by the side), turn by 3 at
    /// most (0x46E4A), face the target.
    pub(super) fn c20_46dd8(&mut self, e: &mut Enemy) {
        let tp = self.c20_target(e);
        let t = self.players[tp].b.clone();
        let idx = e.m.w(f::ROW).wrapping_mul(4) as u32;
        let (mut d1, d2) = (self.assets.enemy.t.i16(t::OFFS, idx), self.assets.enemy.t.i16(t::OFFS, idx + 2));
        if (t.xi().wrapping_sub(e.b.xi())) >= 0 {
            d1 = -d1;
        }
        let d1 = d1.wrapping_add(t.xi());
        let d2 = d2.wrapping_add(t.zi());
        e.m.sw(f::DEST_X, d1 as u16);
        e.m.sw(f::DEST_Z, d2 as u16);
        self.c20_46e4a(e, d1 as u16, d2 as u16);
        e.b.facing = if (t.xi() as u16) < (e.b.xi() as u16) { 0 } else { 1 };
    }

    /// 0x46E4A: direction to (tx, tz), turning by 3 at most.
    fn c20_46e4a(&mut self, e: &mut Enemy, tx: u16, tz: u16) {
        let mut d0 = Self::enemy_angle_to(&e.b, tx, tz);
        let d1 = e.b.dir.wrapping_sub(d0).wrapping_add(3) & 0x1F;
        if d1 > 6 {
            let d2: u8 = if d1 >= 0x13 { 3 } else { 0xFD };
            d0 = e.b.dir.wrapping_add(d2);
        }
        e.b.dir = d0 & 0x1F;
    }

    /// 0x46E7C: slot 0 (the thug is left of the target, unsigned borrow) or 1.
    pub(super) fn c20_46e7c(&mut self, e: &mut Enemy) {
        let tp = self.c20_target(e);
        let d1 = if (e.b.xi() as u16) < (self.players[tp].b.xi() as u16) { 0 } else { 1 };
        e.m.sb(f::SLOT, d1);
    }

    /// 0x46E94: the other slot of the pair, turn about.
    pub(super) fn c20_46e94(&mut self, e: &mut Enemy) {
        let d = e.m.b(f::SLOT).wrapping_add(1) & 1;
        e.m.sb(f::SLOT, d);
        e.b.dir ^= 0x10;
    }

    /// 0x46EA6: the script row `+$AA` by `rand()` (32 per variant, table 0x46EC6).
    pub(super) fn c20_46ea6(&mut self, e: &mut Enemy) {
        let mut r = (self.rand() & 0x1F) as u32;
        if (e.b.var26 >> 8) as u8 != 0 {
            r += 0x20;
        }
        let d1 = self.assets.enemy.t.u8(t::ROW_PICK, r) as u16;
        e.m.sw(f::ROW, d1);
    }

    /// 0x46F06: direction straight at the target (+$88 / +$8A = its position).
    pub(super) fn c20_46f06(&mut self, e: &mut Enemy) {
        let tp = self.c20_target(e);
        let (d1, d2) = (self.players[tp].b.xi(), self.players[tp].b.zi());
        e.m.sw(f::DEST_X, d1 as u16);
        e.m.sw(f::DEST_Z, d2 as u16);
        e.b.dir = Self::enemy_angle_to(&e.b, d1 as u16, d2 as u16);
    }

    /// 0x46F26: |dx| < 0x80? (the carry of `cmpi.w #$100`).
    pub(super) fn c20_46f26(&self, e: &Enemy) -> bool {
        let t = &self.players[self.c20_target(e)].b;
        (t.xi().wrapping_sub(e.b.xi()).wrapping_add(0x80) as u16) < 0x100
    }

    /// 0x46F3C: |dx| < 0x60? Returns the carry.
    pub(super) fn c20_46f3c(&self, e: &Enemy) -> bool {
        let t = &self.players[self.c20_target(e)].b;
        (t.xi().wrapping_sub(e.b.xi()).wrapping_add(0x60) as u16) < 0xC0
    }

    /// 0x47120: is the target out of the window around the point 0x80 beside it (carry set = out)?
    pub(super) fn c20_47120(&self, e: &Enemy) -> bool {
        let t = &self.players[self.c20_target(e)].b;
        let mut d1 = 0x80i16;
        if (e.b.xi() as u16) < (t.xi() as u16) {
            d1 = -d1;
        }
        let dx = t.xi().wrapping_add(d1).wrapping_sub(e.b.xi()).wrapping_add(0x40) as u16;
        if dx >= 0x80 {
            return true;
        }
        let dz = t.zi().wrapping_sub(e.b.zi()).wrapping_add(0x30) as u16;
        dz >= 0x60
    }

    /// 0x4716C: a player (P1, P2, P3) standing in front of us in the normal state becomes the target.
    pub(super) fn c20_4716c(&mut self, e: &mut Enemy) -> bool {
        if e.b.airborne != 0 {
            return false;
        }
        for p in 0..3 {
            let q = &self.players[p].b;
            if q.live == 0 || q.sub != 0 {
                continue;
            }
            let dx = q.xi().wrapping_sub(e.b.xi()).wrapping_add(0x10) as u16;
            let dz = q.zi().wrapping_sub(e.b.zi()).wrapping_add(0x10) as u16;
            if dx < 0x20 && dz < 0x20 {
                e.m.sw(TARGET, Game::player_handle(p));
                return true;
            }
        }
        false
    }

    /// 0x471D8: when the thug is not 0x30 beside the target (window 0x18), skip one script entry.
    pub(super) fn c20_471d8(&mut self, e: &mut Enemy) {
        let tp = self.c20_target(e);
        let t = self.players[tp].b.xi();
        let mut d1 = 0x30i16;
        if (e.b.xi() as u16) >= (t as u16) {
            d1 = -d1;
        }
        let d0 = e.b.xi().wrapping_add(d1).wrapping_sub(t).wrapping_add(0xC) as u16;
        if d0 >= 0x18 {
            e.m.sw(f::POS, e.m.w(f::POS).wrapping_add(2));
        }
    }
}
