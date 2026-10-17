//! The shared pool A library at 0x32xxx that many classes call (aim at the target, face it, the
//! slots round the target, the screen test, the distance class, the target check) and the sound
//! with a pan 0x9D44. Phase 1 task 4a2 (class 14 is the first user; classes 3, 4, 6, 7, 8, 13 and
//! the bosses call the same routines).
//!
//! `a0` of the original is the object the enemy hunts (`+$76`, always a player). Routines that
//! end with a carry or a flag in the original return `bool` here.

use super::{Enemy, off::*};
use crate::game::Game;
use crate::game::body::Body;

/// Offsets of the library (class independent).
pub(super) mod lib {
    /// `+$BD`: the slot number (1..6) the object chose round its target.
    pub const SLOT_NO: usize = 0xBD;
}

impl Game {
    /// The player body an enemy hunts (`+$76`). A word that is no player handle reads player 1.
    pub(super) fn lib_target(&self, e: &Enemy) -> &Body {
        &self.players[self.c10_target(e)].b
    }

    /// 0x32A58 / 0x32A70 (the second one loads `+$88` / `+$8A` first, but jumps into 0x1032, which
    /// loads the target's X and Z again): `+$22` = the direction to the target.
    pub(super) fn lib_aim_at_target(&self, e: &mut Enemy) {
        let t = self.lib_target(e);
        let (tx, tz) = (t.xi() as u16, t.zi() as u16);
        e.b.dir = Self::enemy_angle_to(&e.b, tx, tz);
    }

    /// 0x32A88: the direction to the point `d0` beside the target: the point is on the side of the
    /// target that faces the object (target to the right of it: minus `d0`).
    pub(super) fn lib_aim_beside_target(&self, e: &mut Enemy, d0: i16) {
        let t = self.lib_target(e);
        let (mut d1, d2) = (t.xi(), t.zi());
        let off = if d1 > e.b.xi() { d0.wrapping_neg() } else { d0 }; // cmp.w $8(a6),d1 ; ble
        d1 = d1.wrapping_add(off);
        e.b.dir = Self::enemy_angle_to(&e.b, d1 as u16, d2 as u16);
    }

    /// 0x32B68: face the target (`+$24` = 1 when its biased X is larger than ours, signed).
    pub(super) fn lib_face_target(&self, e: &mut Enemy) {
        let tx = self.lib_target(e).xi().wrapping_add(0x400);
        let x = e.b.xi().wrapping_add(0x400);
        e.b.facing = (tx > x) as u8;
    }

    #[allow(dead_code)] // the classes that call 0x32B94 (3, 4, 6, 7, 8, 13) are not ported yet
    /// 0x32B94: the cooked input byte (G.$16 / $17 / $18) of the player the target is. The second
    /// result d1 (G.$1B / $1C / $1D) is not modeled; no ported caller reads it.
    pub(super) fn lib_target_input(&self, e: &Enemy) -> u8 {
        let who = self.lib_target(e).who.wrapping_sub(1) as i8;
        let p = if who == 0 {
            1
        } else if who > 0 {
            2
        } else {
            0
        };
        self.w.input[p]
    }

    /// 0x32F56: when the target is not in the normal live state (+$00 != 1) choose a new one.
    pub(super) fn lib_check_target(&self, e: &mut Enemy) {
        if self.lib_target(e).live != 1 {
            let t = self.enemy_pick_target(&e.b);
            e.m.sw(TARGET, t);
        }
    }

    /// 0x3282A: true when the object is off screen (the original returns d5 = 1; callers test `beq`
    /// for "on screen"): camera X + 0x400 - 0x18 > X + 0x400, or camera X + 0x400 + 0x180 < X + 0x400.
    pub(super) fn lib_off_screen(&self, b: &Body) -> bool {
        let mut d0 = (self.cam.s2.cam_x as i16).wrapping_add(0x400).wrapping_sub(0x18);
        let d1 = b.xi().wrapping_add(0x400);
        if d0 > d1 {
            return true;
        }
        d0 = d0.wrapping_add(0x198);
        d0 < d1
    }

    /// 0x3293C: (d0, d1) = the distance class of the target in X (0..7) and in Z (0..3). The class is
    /// the highest index whose window (unsigned `tx + T1[i] - x <= T2[i]`) the target is outside of
    /// (0 when it is inside all of them).
    pub(super) fn lib_distance_class(&self, e: &Enemy) -> (u16, u16) {
        let t = self.lib_target(e);
        let tab = &self.assets.enemy.t;
        let mut dx = 0u16;
        for i in (0..8u32).rev() {
            let d2 = t.xi().wrapping_add(tab.u16(0x32988, 2 * i) as i16).wrapping_sub(e.b.xi()) as u16;
            if d2 > tab.u16(0x32998, 2 * i) {
                dx = i as u16;
                break;
            }
        }
        let mut dz = 0u16;
        for i in (0..4u32).rev() {
            let d2 = t.zi().wrapping_add(tab.u16(0x329A8, 2 * i) as i16).wrapping_sub(e.b.zi()) as u16;
            if d2 > tab.u16(0x329B0, 2 * i) {
                dz = i as u16;
                break;
            }
        }
        (dx, dz)
    }

    /// The mirror step the slot routines share: `dx` is the table offset; when the target is to the
    /// right of the object (biased signed compare) the point is left of the target, else right of it.
    pub(super) fn lib_slot_mirror(&self, e: &mut Enemy, dx: i16) {
        let t = self.lib_target(e);
        let tx = t.xi();
        let mut d3 = dx;
        let right = (tx.wrapping_add(0x400)) > e.b.xi().wrapping_add(0x400);
        let flip = if right { d3 >= 0 } else { d3 < 0 };
        if flip {
            d3 = d3.wrapping_neg().wrapping_add(tx);
            e.m.sw(DEST_X, d3 as u16);
        }
    }

    /// 0x32D90: choose a new slot 1..3 (`rand() & 3`, 0 is drawn again), then 0x32D9E.
    pub(super) fn lib_slot3_new(&mut self, e: &mut Enemy) {
        loop {
            let d0 = (self.rand() & 3) as u8;
            e.m.sb(lib::SLOT_NO, d0);
            if d0 != 0 {
                break;
            }
        }
        self.lib_slot3_point(e);
    }

    /// 0x32D86: the point of the slot kept in `+$BD` (a new one when it is 0).
    pub(super) fn lib_slot3_keep(&mut self, e: &mut Enemy) {
        if e.m.b(lib::SLOT_NO) == 0 {
            return self.lib_slot3_new(e);
        }
        self.lib_slot3_point(e);
    }

    /// 0x32D9E: the point (+$88, +$8A) of slot `+$BD` (table 0x32DF8): X beside the target, Z = the
    /// target's Z.
    fn lib_slot3_point(&mut self, e: &mut Enemy) {
        let d0 = e.m.b(lib::SLOT_NO) as u32 * 2;
        let t = self.lib_target(e);
        let (tx, tz) = (t.xi(), t.zi());
        let off = self.assets.enemy.t.i16(0x32DF8, d0);
        e.m.sw(DEST_Z, tz as u16);
        e.m.sw(DEST_X, tx.wrapping_add(off) as u16);
        self.lib_slot_mirror(e, off);
    }

    /// 0x32C5E: choose a new slot 1..6 round the target (`rand() & 7`; 7 and 0 are drawn again).
    /// Returns d4: 1 when the target already has 3 or more enemies on one side (census counters
    /// +$E6 / +$E7 of the player, signed byte compare).
    pub(super) fn lib_slot6_new(&mut self, e: &mut Enemy) -> bool {
        let (d0, d4) = loop {
            let d0 = (self.rand() & 7) as u8;
            if d0 == 7 {
                continue;
            }
            let p = self.c10_target(e);
            let c = &self.spawn.g.census[p];
            let d4 = (c[2] as i8) >= 3 || (c[3] as i8) >= 3;
            e.m.sb(lib::SLOT_NO, d0);
            if d0 != 0 {
                break (d0, d4);
            }
        };
        let tab = &self.assets.enemy.t;
        let (ox, oz) = (tab.i16(0x32CF0, d0 as u32 * 2), tab.i16(0x32CFE, d0 as u32 * 2));
        let t = self.lib_target(e);
        let (tx, tz) = (t.xi(), t.zi());
        e.m.sw(DEST_X, tx.wrapping_add(ox) as u16);
        e.m.sw(DEST_Z, tz.wrapping_add(oz) as u16);
        self.lib_slot_mirror(e, ox);
        d4
    }

    /// 0x9D44: the sound of a thrown knife or blade: 0xE3 / 0x64 by the toggle bit 7 of G.$43A.
    /// d1 = (3, or 4 when a6's +$24 is nonzero) << 16 | 0x2000 | the pan byte of `x` (a6's +$08), d2 = 0x2400000.
    pub(super) fn snd_9d44(&mut self, x: i16, facing: u8) {
        let old = self.w.snd_toggle >> 7 & 1 != 0;
        self.w.snd_toggle ^= 1 << 7;
        let mode = if facing != 0 { 4 } else { 3 };
        let pan = self.snd_pan_x(x) as u32;
        self.sound_ex(if old { 0x64 } else { 0xE3 }, (mode << 16) | 0x2000 | pan, 0x0240_0000);
    }

    /// 0x9732: the sound of a blocked hit (0xFC / 0x7B by the toggle bit 2 of G.$43A).
    /// d1 = the pan byte of `x` (a6's +$08, mode 0), d2 = 0.
    pub fn snd_9732(&mut self, x: i16) {
        let old = self.w.snd_toggle >> 2 & 1 != 0;
        self.w.snd_toggle ^= 1 << 2;
        let pan = self.snd_pan_x(x) as u32;
        self.sound_ex(if old { 0x7B } else { 0xFC }, pan, 0);
    }

    /// 0x96CE: 0xE4 / 0x65 by the toggle bit 1 of G.$43A. d1 = the pan byte of `x` (a6's +$08), d2 = 0.
    pub(super) fn snd_96ce(&mut self, x: i16) {
        let old = self.w.snd_toggle >> 1 & 1 != 0;
        self.w.snd_toggle ^= 1 << 1;
        let pan = self.snd_pan_x(x) as u32;
        self.sound_ex(if old { 0x65 } else { 0xE4 }, pan, 0);
    }
}

/// `move.w src,dst` on the integer part of a 16.16 position: the fraction stays.
pub(super) fn put_hi(v: &mut i32, w: i16) {
    *v = (*v & 0xFFFF) | ((w as i32) << 16);
}
