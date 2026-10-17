//! Class 10: the small routines at 0x3CB76..0x3D100 (slots, steering, window tests, speeds) and
//! the death. Every routine returning a carry returns `bool` (true = carry set).

use super::class10::{f, t};
use super::{Enemy, off::*};
use crate::game::Game;
use crate::game::enemy::neg_near;

impl Game {
    /// 0x3CB76: choose the target (0x32032), then its side (0x3CB80).
    pub(super) fn c10_pick_target_slot(&mut self, e: &mut Enemy) {
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        self.c10_3cb80(e);
    }

    /// 0x3CB80: the slot 0..3 round the target: 0 / 1 left of it (near / far), 2 / 3 right of it.
    pub(super) fn c10_3cb80(&mut self, e: &mut Enemy) {
        let tx = self.players[self.c10_target(e)].b.xi();
        let (d3, d2): (u8, i16) = if neg_near(e.b.xi().wrapping_sub(tx)) { (0, -0x60) } else { (2, 0x60) };
        let d0 = (tx as u16).wrapping_add(0x400).wrapping_add(d2 as u16);
        let d1 = (e.b.xi() as u16).wrapping_add(0x400);
        let carry = d0 < d1; // cmp.w d1,d0
        e.m.sb(f::SLOT, d3.wrapping_add(carry as u8));
    }

    /// The slot table entry `slot` of 0x3CC04 as (dx, dz).
    fn c10_slot_offset(&self, slot: u8) -> (i16, i16) {
        let t = &self.assets.enemy.t;
        (t.i16(t::SLOTS, slot as u32 * 4), t.i16(t::SLOTS, slot as u32 * 4 + 2))
    }

    /// 0x3CBC0: the point of the slot beside the target -> +$88 / +$8A, turn towards it by at most
    /// one step (0x3CE12), face the target.
    pub(super) fn c10_3cbc0(&mut self, e: &mut Enemy) {
        let tp = self.c10_target(e);
        let (dx, dz) = self.c10_slot_offset(e.m.b(f::SLOT));
        let tb = &self.players[tp].b;
        let d1 = dx.wrapping_add(tb.xi());
        let d2 = dz.wrapping_add(tb.zi());
        e.m.sw(DEST_X, d1 as u16);
        e.m.sw(DEST_Z, d2 as u16);
        self.c10_3ce12(e, d1 as u16, d2 as u16);
        let tx = self.players[tp].b.xi();
        e.b.facing = if neg_near(tx.wrapping_sub(e.b.xi())) { 0 } else { 1 };
    }

    /// 0x3CC56: next slot of the four round the target, turn about.
    pub(super) fn c10_3cc56(&mut self, e: &mut Enemy) {
        let d0 = e.m.b(f::SLOT).wrapping_add(2) & 3;
        e.m.sb(f::SLOT, d0);
        e.b.dir ^= 0x10;
    }

    /// 0x3CC6E: the other of the slots 4 / 5, turn about.
    pub(super) fn c10_3cc6e(&mut self, e: &mut Enemy) {
        let d0 = (e.m.b(f::SLOT).wrapping_sub(4).wrapping_add(1) & 1).wrapping_add(4);
        e.m.sb(f::SLOT, d0);
        e.b.dir ^= 0x10;
    }

    /// 0x3CC8A: the point of the slot -> +$88 / +$8A and the direction straight at it.
    pub(super) fn c10_3cc8a(&mut self, e: &mut Enemy) {
        let tp = self.c10_target(e);
        let (dx, dz) = self.c10_slot_offset(e.m.b(f::SLOT));
        let tb = &self.players[tp].b;
        let d1 = dx.wrapping_add(tb.xi());
        let d2 = dz.wrapping_add(tb.zi());
        e.m.sw(DEST_X, d1 as u16);
        e.m.sw(DEST_Z, d2 as u16);
        e.b.dir = Self::enemy_angle_to(&e.b, d1 as u16, d2 as u16);
    }

    /// 0x3CCC0: the direction at the point the *target* object keeps at +$88 / +$8A (a player's
    /// +$88 is the scratch word of 0x6606; its +$8A is never written: 0 in every recording).
    pub(super) fn c10_3ccc0(&mut self, e: &mut Enemy) {
        let tp = self.c10_target(e);
        let (d1, d2) = (self.players[tp].w88, 0u16);
        e.b.dir = Self::enemy_angle_to(&e.b, d1, d2);
    }

    /// 0x3CCD8: the target's position -> +$88 / +$8A, direction by the map 0x3CD14, face the target.
    pub(super) fn c10_3ccd8(&mut self, e: &mut Enemy) {
        let tp = self.c10_target(e);
        let (d1, d2) = (self.players[tp].b.xi(), self.players[tp].b.zi());
        e.m.sw(DEST_X, d1 as u16);
        e.m.sw(DEST_Z, d2 as u16);
        let a = Self::enemy_angle_to(&e.b, d1 as u16, d2 as u16);
        e.b.dir = self.assets.enemy.t.u8(t::DIR_MAP, a as u32);
        e.b.facing = if neg_near(d1.wrapping_sub(e.b.xi())) { 0 } else { 1 };
    }

    /// 0x3CD34: close to the point 0x30 in front (by the direction) of the target? Then start the
    /// slide (0x3CD5A). Returns the carry.
    pub(super) fn c10_3cd34(&mut self, e: &mut Enemy) -> bool {
        let tp = self.c10_target(e);
        let mut d0 = 0x30i16;
        if e.b.dir & 0x10 != 0 {
            d0 = -d0;
        }
        let d1 = self.players[tp].b.xi().wrapping_add(d0);
        let d0 = e.b.xi().wrapping_sub(d1).wrapping_add(8) as u16;
        if d0 >= 0x10 {
            return false;
        }
        self.c10_3cd5a(e);
        true
    }

    /// 0x3CD5A: brake: the accelerations are minus a thirty-second of the speed, vy = ay = 0, `+$A8` = 8.
    pub(super) fn c10_3cd5a(&mut self, e: &mut Enemy) {
        e.b.ax = e.b.vx.wrapping_neg() >> 5;
        e.b.az = e.b.vz.wrapping_neg() >> 5;
        e.b.vy = 0;
        e.b.ay = 0;
        e.m.sw(f::SUB, 8);
    }

    /// 0x3CD8C: close to the point 0x68 away from the target (by the direction)? Returns the carry.
    pub(super) fn c10_3cd8c(&mut self, e: &mut Enemy) -> bool {
        let tp = self.c10_target(e);
        let mut d0 = 0x68i16;
        if e.b.dir & 0x10 == 0 {
            d0 = -d0;
        }
        let d1 = self.players[tp].b.xi().wrapping_add(d0);
        e.b.xi().wrapping_sub(d1).wrapping_add(8) as u16 <= 0x0F
    }

    /// 0x3CDBC: aim at the target (turn by at most two steps, 0x3CE4C).
    pub(super) fn c10_3cdbc(&mut self, e: &mut Enemy) {
        let tp = self.c10_target(e);
        let (d1, d2) = (self.players[tp].b.xi(), self.players[tp].b.zi());
        e.m.sw(DEST_X, d1 as u16);
        e.m.sw(DEST_Z, d2 as u16);
        self.c10_3ce4c(e, d1 as u16, d2 as u16);
    }

    /// 0x3CDD4: in front of the target by 0x38 (facing side)? Then walk straight at the stored point.
    pub(super) fn c10_3cdd4(&mut self, e: &mut Enemy) {
        let tp = self.c10_target(e);
        let mut d0 = 0x38i16;
        if e.b.facing != 0 {
            d0 = -d0;
        }
        let d1 = (self.players[tp].b.xi() as u16).wrapping_add(0x400).wrapping_add(d0 as u16);
        let d0 = (e.b.xi() as u16).wrapping_add(0x400).wrapping_sub(d1).wrapping_add(8);
        if d0 >= 0x10 {
            return;
        }
        e.m.sb(f::WALK_KIND, 1);
        let (d1, d2) = (e.m.w(DEST_X), e.m.w(DEST_Z));
        self.c10_3ce4c(e, d1, d2);
    }

    /// 0x3CE12: direction to (tx, tz); unless +$7D is 1, turn the current direction by one step at most.
    pub(super) fn c10_3ce12(&mut self, e: &mut Enemy, tx: u16, tz: u16) {
        let mut d0 = Self::enemy_angle_to(&e.b, tx, tz);
        if e.m.b(ATTACKING) != 1 {
            let d1 = e.b.dir.wrapping_sub(d0).wrapping_add(1) & 0x1F;
            if d1 > 2 {
                let d2: u8 = if d1 >= 0x11 { 1 } else { 0xFF };
                d0 = e.b.dir.wrapping_add(d2);
            }
        }
        e.b.dir = d0 & 0x1F;
    }

    /// 0x3CE4C: direction to (tx, tz), turning by two steps at most.
    pub(super) fn c10_3ce4c(&mut self, e: &mut Enemy, tx: u16, tz: u16) {
        let mut d0 = Self::enemy_angle_to(&e.b, tx, tz);
        let d1 = e.b.dir.wrapping_sub(d0).wrapping_add(2) & 0x1F;
        if d1 > 4 {
            let d2: u8 = if d1 >= 0x12 { 2 } else { 0xFE };
            d0 = e.b.dir.wrapping_add(d2);
        }
        e.b.dir = d0 & 0x1F;
    }

    /// 0x3CE7E: at the stored point (window 0x20 x 0x18)? Returns the carry.
    pub(super) fn c10_3ce7e(&self, e: &Enemy) -> bool {
        let dx = e.b.xi().wrapping_sub(e.m.w(DEST_X) as i16).wrapping_add(0x10) as u16;
        let dz = e.b.zi().wrapping_sub(e.m.w(DEST_Z) as i16).wrapping_add(0xC) as u16;
        dx < 0x20 && dz < 0x18
    }

    /// 0x3CEAC: at the stored point (window 8 x 0xE)? Returns the carry.
    pub(super) fn c10_3ceac(&self, e: &Enemy) -> bool {
        let dx = e.b.xi().wrapping_sub(e.m.w(DEST_X) as i16).wrapping_add(4) as u16;
        let dz = e.b.zi().wrapping_sub(e.m.w(DEST_Z) as i16).wrapping_add(7) as u16;
        dx < 8 && dz < 0xE
    }

    /// 0x3CEDA: near the target (window 0x80 x 0x20)? Returns the carry.
    pub(super) fn c10_3ceda(&self, e: &Enemy) -> bool {
        let t = &self.players[self.c10_target(e)].b;
        let dx = e.b.xi().wrapping_sub(t.xi()).wrapping_add(0x40) as u16;
        let dz = e.b.zi().wrapping_sub(t.zi()).wrapping_add(0x10) as u16;
        dx < 0x80 && dz < 0x20
    }

    /// 0x3CF0C: the other slot of the pair on the same side of the target.
    pub(super) fn c10_3cf0c(&mut self, e: &mut Enemy) {
        let tx = self.players[self.c10_target(e)].b.xi();
        let d1: u8 = if neg_near(e.b.xi().wrapping_sub(tx)) { 0 } else { 2 };
        let d0 = (e.m.b(f::SLOT).wrapping_add(1) & 1).wrapping_add(d1);
        e.m.sb(f::SLOT, d0);
    }

    /// 0x3CF36: does the thug attack now? Not at a player in action 0x20 / 0x22 / 0x23, else the
    /// chance table (row of the variant, by level) against `rand() & 0x1F`. Returns the carry.
    pub(super) fn c10_3cf36(&mut self, e: &mut Enemy) -> bool {
        let tp = self.c10_target(e);
        if matches!(self.players[tp].action, 0x20 | 0x22 | 0x23) {
            return false;
        }
        let row = ((e.b.var26 >> 8) as u32 & 0xFF) << 4;
        let d1 = self.assets.enemy.t.u8(t::ATTACK_CHANCE, row + (self.w.level & 0x1F) as u32);
        let r = (self.rand() & 0x1F) as u8;
        d1 > r
    }

    /// 0x3D02E: slot 4 (target to the right of the thug) or 5.
    pub(super) fn c10_3d02e(&mut self, e: &mut Enemy) {
        let tx = self.players[self.c10_target(e)].b.xi();
        let d1 = if neg_near(e.b.xi().wrapping_sub(tx)) { 4 } else { 5 };
        e.m.sb(f::SLOT, d1);
    }

    /// 0x3D04A: slot 6 or 7.
    pub(super) fn c10_3d04a(&mut self, e: &mut Enemy) {
        let tx = self.players[self.c10_target(e)].b.xi();
        let d1 = if neg_near(e.b.xi().wrapping_sub(tx)) { 6 } else { 7 };
        e.m.sb(f::SLOT, d1);
    }

    /// 0x3D066: slot 8 (the target faces left) or 9.
    pub(super) fn c10_3d066(&mut self, e: &mut Enemy) {
        let tf = self.players[self.c10_target(e)].b.facing;
        e.m.sb(f::SLOT, if tf != 0 { 9 } else { 8 });
    }

    /// 0x3D07A: close to the target (window 0x40 x 0x30)? Returns the carry (0x3D0B4 changes no
    /// field: it adds the zero `d2` to the direction).
    pub(super) fn c10_3d07a(&mut self, e: &mut Enemy) -> bool {
        let t = &self.players[self.c10_target(e)].b;
        let dx = e.b.xi().wrapping_sub(t.xi()).wrapping_add(0x400).wrapping_add(0x20) as u16;
        let dz = e.b.zi().wrapping_sub(t.zi()).wrapping_add(0x18) as u16;
        dx < 0x40 && dz < 0x30
    }

    /// 0x3D100: speed id by variant row and level band (`jmp 0x17A4`).
    pub(super) fn c10_3d100(&mut self, e: &mut Enemy) {
        let row = ((e.b.var26 >> 8) as u32 & 0xFF) << 3;
        let band = ((self.w.level & 0x1C) >> 1) as u32;
        let id = self.assets.enemy.t.u16(t::SPEEDS, row + band);
        self.enemy_set_speed(&mut e.b, id);
    }

    /// 0x3C9C8: close to the target (window 0x28 x 0x10, with the bias of 0x400 that no real
    /// position satisfies). Returns the carry.
    pub(super) fn c10_3c9c8(&self, e: &Enemy) -> bool {
        let t = &self.players[self.c10_target(e)].b;
        let dx = e.b.xi().wrapping_sub(t.xi()).wrapping_add(0x400).wrapping_add(0x14) as u16;
        let dz = e.b.zi().wrapping_sub(t.zi()).wrapping_add(8) as u16;
        dx < 0x28 && dz < 0x10
    }

    /// 0x3CC2C: turn (two steps at most) towards the target's position; face it. Nothing is stored.
    pub(super) fn c10_3cc2c(&mut self, e: &mut Enemy) {
        let tp = self.c10_target(e);
        let (d1, d2) = (self.players[tp].b.xi(), self.players[tp].b.zi());
        self.c10_3ce4c(e, d1 as u16, d2 as u16);
        e.b.facing = if neg_near(d1.wrapping_sub(e.b.xi())) { 0 } else { 1 };
    }

    /// 0x3CB32: death (kill score, blink for 120 frames, free): the code of class 2.
    pub(super) fn c10_dying(&mut self, e: &mut Enemy, h: crate::game::Handle) {
        self.enemy_dying_basic(e, h);
    }

    /// 0x32540: a repeat count 0..? from the table 0x32566 (row by level band, one `rand()`), minus 1.
    pub(super) fn enemy_rep_count(&mut self) -> u8 {
        let d1 = ((self.w.level & 0x18) << 2) as u32;
        let r = (self.rand() & 0x1F) as u32;
        self.assets.enemy.t.u8(0x32566, d1 + r).wrapping_sub(1)
    }
}

impl Game {
    /// 0xA062: a thug sound, 0x73 or 0xF9 (`bchg.b #3,$43B(a5)`: 0x73 when the bit was set).
    /// d1 = the pan byte of the thug's X (`x`), d2 = 0.
    pub(super) fn enemy_snd_a062(&mut self, x: i16) {
        let old = self.spawn.g.snd_toggle_b >> 3 & 1 != 0;
        self.spawn.g.snd_toggle_b ^= 1 << 3;
        let pan = self.snd_pan_x(x) as u32;
        self.sound_ex(if old { 0x73 } else { 0xF9 }, pan, 0);
    }
}
