//! The two helpers of LASH T. (class 3): the mace, class 62 (+$20 = 0xF8, handler 0x5A47C) and the chain piece, class 8 (+$20 = 0x20, handler 0x3B28A). Both
//! have the Lash in `+$A0` (a sign extended long, the low word is the handle) and copy their place from the frame byte +$2D of his animation. D3a section 6.
//!
//! The mace (the picture of the weapon in his hand): sub 0 held (follows the frame rows of the table 0x5A79E, drawn), sub 4 the throw (+$05 = 4 is
//! set when the Lash shows the frame byte 0x63), sub 2 flies 20 frames at 8 px a frame, sub 6 turns, sub 8 flies back; while it is out (+$B2 not 0) the Lash
//! does nothing else (0x35ADC) and the mace is not drawn. The chain piece (class 8) is the part that hurts: attacks 0x54 / 0x55 / 0x56 by a counter, a
//! monster list entry (0x4C0A), its place from the table 0x3B52E by the Lash's frame byte.
//!
//! Owner: enemy port (M6-07).

use super::{off::*, Enemy};
use crate::game::{slot_of, Game, Handle, Slot};

/// The tables (`dino-extract` TABLES): the mace's rows by the Lash's frame byte (dx, dy, unused, animation; 8 bytes), the chain's rows (dx, dy, dz; 8 bytes).
const MACE_ROWS: u32 = 0x5A79E;
const CHAIN_ROWS: u32 = 0x3B52E;

/// Mace bytes: +$A0 long the Lash, +$A4 byte the animation shown, +$A5 byte the anim-ended flag, +$A6 word the frame byte of the Lash / the animation of the row,
/// +$AA long the X speed of the flight, +$B0 word frames left of the flight, +$B2 byte 1 flying out, 2 flying back, 0 held, +$B4 word flight length, +$B6 word
/// the height above the Lash.
const LASH: usize = 0xA0;
const SHOWN: usize = 0xA4;
const ENDED: usize = 0xA5;
const FRAME_BYTE: usize = 0xA6;
const SPEED: usize = 0xAA;
const LEFT: usize = 0xB0;
const OUT: usize = 0xB2;
const LENGTH: usize = 0xB4;
const RISE: usize = 0xB6;

impl Game {
    /// The pool A handle in the long +$A0 (the low word of the sign extended address).
    fn c62_lash(e: &Enemy) -> Handle {
        e.m.w(LASH + 2)
    }

    /// Mode, sub state and the frame byte +$2D of the Lash, and his facing, for the helper that runs.
    fn c62_lash_state(&self, e: &Enemy) -> (u8, u8, u8, u8, u8, i16, i16, i16, u8) {
        let b = self.body(Self::c62_lash(e));
        (b.mode, b.sub, b.act, b.step, b.facing, b.xi(), b.yi(), b.zi(), b.param_b)
    }

    // ------------------------------------------------------------------ class 62: the mace

    /// 0x5A47C: class 62. Mode 0 init, 2 the states, 4 and 6 free (0x5A798).
    pub(super) fn enemy_class62(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => self.c62_init(e),
            2 => self.c62_run(e, h, i),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x5A490: the init. Not hittable (+$00 = 2, +$7D and +$63 0xFF), animation table 0x7020A, attack 0x2A, on the floor, animation 1. The mode becomes 2 at
    /// once only when the Lash has left his own mode 0.
    fn c62_init(&mut self, e: &mut Enemy) {
        e.b.live = 2;
        e.m.sb(ATTACKING, 0xFF);
        e.b.b63 = 0xFF;
        e.b.anim_table = 0x7020A;
        e.b.draw_flags = 0x10;
        e.m.sb(ENDED, 0);
        e.m.sb(OUT, 0);
        e.m.sw(FRAME_BYTE, 0);
        e.m.sb(SHOWN, 0);
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_begin_attack(&mut e.b, 0x2A);
        self.enemy_snap_to_floor(e); // jsr 0x3140C
        self.enemy_floor_collision(e); // jsr 0x12CB4
        self.enemy_anim(&mut e.b, 1);
        if self.body(Self::c62_lash(e)).mode != 0 {
            e.b.mode = 2;
        }
    }

    /// 0x5A50E: mode 2. The Lash dead (mode 4 or more): free. The place from his frame (0x5A544), the depth one less than his, the state (table 0x5A53A).
    fn c62_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        let (mode, ..) = self.c62_lash_state(e);
        if mode >= 4 {
            return self.enemy_free_pool_a(e, i); // 0x5A798
        }
        self.c62_place(e);
        let lz = self.body(Self::c62_lash(e)).zi();
        e.b.set_zi_keep(lz.wrapping_sub(1));
        match e.b.sub {
            0 => self.c62_held(e, h),
            2 => self.c62_fly_out(e),
            4 => self.c62_throw(e),
            6 => self.c62_turn(e),
            8 => self.c62_fly_back(e),
            _ => {}
        }
    }

    /// 0x5A544: the place from the Lash's frame byte (not while the mace is out). The byte 0x63 starts the throw (the Lash's byte is set to 0x30); 0x5A is the
    /// hidden frame (row 0x28 and the Lash's byte 0x30). The row: dx (mirrored by his facing), dy, the depth of the Lash minus one, the animation to show.
    fn c62_place(&mut self, e: &mut Enemy) {
        if e.m.b(OUT) != 0 {
            return;
        }
        let lh = Self::c62_lash(e);
        let (_, _, _, _, facing, lx, ly, lz, flag) = self.c62_lash_state(e);
        let mut d0 = flag as u16;
        e.m.sw(FRAME_BYTE, d0);
        if d0 == 0x63 {
            e.b.sub = 4;
            self.c62_set_lash_flag(lh, 0x30);
            return;
        }
        if d0 == 0x5A {
            d0 = 0x28;
            self.c62_set_lash_flag(lh, 0x30);
        }
        let t = &self.assets.enemy.t;
        let row = MACE_ROWS + (d0 as u32) * 8;
        let (mut dx, dy, anim) = (t.i16(row, 0), t.i16(row, 2), t.u16(row, 6));
        if facing != 0 {
            dx = dx.wrapping_neg();
        }
        e.b.set_xi_keep(lx.wrapping_add(dx));
        e.b.set_yi_keep(ly.wrapping_add(dy));
        e.b.set_zi_keep(lz.wrapping_sub(1));
        e.m.sw(FRAME_BYTE, anim);
        if anim == 0x5A || e.m.b(SHOWN) == anim as u8 {
            return;
        }
        e.m.sb(SHOWN, anim as u8);
        self.enemy_anim(&mut e.b, anim);
    }

    /// `move.b #n,$2d(a1)` on the Lash.
    fn c62_set_lash_flag(&mut self, lh: Handle, v: u8) {
        self.body_mut(lh).param_b = v;
    }

    /// 0x5A5DC: sub 0, held: his facing, the animation steps (at its end animation 1 again), drawn unless hidden (+$A6 = 0x5A) or the Lash's state word +$06 is 4.
    fn c62_held(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sb(ENDED, 0);
        let lh = Self::c62_lash(e);
        e.b.facing = self.body(lh).facing;
        if self.enemy_step(&mut e.b) {
            e.m.sb(ENDED, 1);
            self.enemy_anim(&mut e.b, 1);
        }
        let lb = self.body(lh);
        if e.m.w(FRAME_BYTE) == 0x5A || ((lb.act as u16) << 8 | lb.step as u16) == 4 {
            return;
        }
        self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
    }

    /// 0x5A62C: sub 2, the flight out: +$AA (8 px) a frame for +$B0 frames, then sub 6.
    fn c62_fly_out(&mut self, e: &mut Enemy) {
        let n = e.m.w(LEFT).wrapping_sub(1);
        e.m.sw(LEFT, n);
        if n == 0 {
            e.b.sub = 6;
            e.m.sb(OUT, 2);
        } else {
            e.b.x = e.b.x.wrapping_add(e.m.l(SPEED) as i32);
        }
    }

    /// 0x5A654: sub 4, the throw starts: attack 8, direction and speed by the side (his facing, reversed by +$BF), the height 0x3A above his, 0x14 frames, animation 5.
    /// A random number is drawn and its table byte thrown away (`moveq #$14,d0` follows).
    fn c62_throw(&mut self, e: &mut Enemy) {
        self.enemy_begin_attack(&mut e.b, 8);
        let lh = Self::c62_lash(e);
        let (lfacing, ly, bf) = {
            let b = self.body(lh);
            let bf = match slot_of(lh) {
                Some(Slot::Enemy(k)) => self.enemies.slots[k].m.b(0xBF),
                _ => 0,
            };
            (b.facing, b.yi(), bf)
        };
        e.b.sub = 2;
        e.b.facing = lfacing;
        if bf != 0 {
            e.b.facing = e.b.facing.wrapping_neg(); // neg.b $24(a6)
        }
        e.b.dir = 8;
        e.m.sw(RISE, 0x3A);
        if e.b.facing == 0 {
            e.b.dir = 0x18;
            e.m.sw(RISE, 0x3A);
        }
        e.b.set_yi_keep(ly.wrapping_add(0x3A));
        e.m.sl(SPEED, 0x80000);
        if e.b.facing == 0 {
            e.m.sl(SPEED, 0x80000u32.wrapping_neg());
        }
        let _ = self.rand() & 0xF; // jsr 0x119C; the byte of 0x5A702 is replaced by 0x14
        e.m.sw(LEFT, 0x14);
        e.m.sw(LENGTH, 0x14);
        e.m.sb(OUT, 1);
        self.enemy_anim(&mut e.b, 5);
    }

    /// 0x5A712: sub 6, the turn: sub 8, the flight back one frame shorter than out minus 4, the speed reversed, the height 0x50, animation 0; the LASH shows
    /// his animation 0x11 when he is in his sub 0 or 1 (the Lash `a6` of that `jsr 0x120E`).
    fn c62_turn(&mut self, e: &mut Enemy) {
        e.b.sub = 8;
        let len = e.m.w(LENGTH);
        e.m.sw(LEFT, len.wrapping_sub(4));
        e.m.sl(SPEED, e.m.l(SPEED).wrapping_neg());
        e.m.sw(RISE, 0x50);
        self.enemy_anim(&mut e.b, 0);
        let lh = Self::c62_lash(e);
        if self.body(lh).sub < 2 {
            if let Some(Slot::Enemy(k)) = slot_of(lh) {
                let mut b = std::mem::take(&mut self.enemies.slots[k].b);
                self.enemy_anim(&mut b, 0x11);
                self.enemies.slots[k].b = b;
            }
        }
    }

    /// 0x5A75E: sub 8, the flight back: the speed added, the height above the Lash; at the end sub 0 and the mace is in his hand again.
    fn c62_fly_back(&mut self, e: &mut Enemy) {
        let n = e.m.w(LEFT).wrapping_sub(1);
        e.m.sw(LEFT, n);
        if n == 0 {
            e.b.sub = 0;
            e.m.sb(OUT, 0);
            return;
        }
        e.b.x = e.b.x.wrapping_add(e.m.l(SPEED) as i32);
        let ly = self.body(Self::c62_lash(e)).yi();
        e.b.set_yi_keep(ly.wrapping_add(e.m.w(RISE) as i16));
    }

    // ------------------------------------------------------------------ class 8: the chain piece

    /// 0x3B28A: class 8. Mode 0 init, 2 the states, 4 and 6 free (0x3B528).
    pub(super) fn enemy_class8(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => self.c8_init(e, i),
            2 => self.c8_run(e, h, i),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x3B2A0: free when the Lash is dead; else animation table 0x712A0, +$00 = 2, uncounted, no shadow, at the Lash's place, attack 0x54, animation 0xA.
    fn c8_init(&mut self, e: &mut Enemy, i: usize) {
        let lh = Self::c62_lash(e);
        if self.body(lh).mode >= 4 {
            return self.enemy_free_pool_a(e, i);
        }
        e.b.anim_table = 0x712A0;
        e.b.live = 2;
        e.m.sb(ATTACKING, 0xFF);
        e.b.draw_flags = 0x10;
        e.m.sb(0xA8, 0);
        let (lx, ly, lz) = {
            let b = self.body(lh);
            (b.xi(), b.yi(), b.zi())
        };
        e.b.set_xi_keep(lx);
        e.b.set_yi_keep(ly);
        e.b.set_zi_keep(lz);
        e.m.sw(0xAA, 0);
        self.enemy_floor_collision(e); // jsr 0x12CB4
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_begin_attack(&mut e.b, 0x54);
        self.enemy_anim(&mut e.b, 0xA);
        e.b.mode = e.b.mode.wrapping_add(2);
    }

    /// 0x3B322: mode 2. The Lash's facing; a hit stop (+$59) or a wait (+$7A) is cleared; while the Lash is not in his sub 0 the counter is cleared and the
    /// attack restarts (0x3B502); dead: free; in the Lash's hit stop the attack 0x54 restarts and the piece is only drawn; else the counter +$AA runs 0 .. 0x15
    /// (attacks 0x54, 0x55 on odd counts, 0x55 at 10, 0x56 at 20), the mace's frame byte starts animation 0xB, and the place comes from the table 0x3B52E.
    fn c8_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        let lh = Self::c62_lash(e);
        e.b.facing = self.body(lh).facing;
        if e.b.hitstop != 0 {
            e.b.hitstop = 0;
            self.enemy_begin_attack(&mut e.b, 0x54);
        }
        if e.m.b(WAIT) != 0 {
            e.m.sb(WAIT, 0);
        }
        let (lmode, lsub, _, _, lfacing, lx, ly, lz, lflag) = self.c62_lash_state(e);
        if lsub != 0 {
            // 0x3B502
            e.m.sw(0xAA, 0);
            self.enemy_begin_attack(&mut e.b, 0x54);
            return self.enemy_anim(&mut e.b, 0xA);
        }
        if lmode >= 4 {
            return self.enemy_free_pool_a(e, i);
        }
        if self.body(lh).hitstop != 0 {
            self.enemy_begin_attack(&mut e.b, 0x54);
            return self.c8_end(e, h);
        }
        let n = e.m.w(0xAA).wrapping_add(1);
        e.m.sw(0xAA, n);
        self.enemy_begin_attack(&mut e.b, 0x54);
        if n & 1 != 0 {
            self.enemy_begin_attack(&mut e.b, 0x55);
        }
        if (n as i16) > 0x15 {
            e.b.timer = 1;
            e.b.hitstop = 0;
            e.m.sb(WAIT, 0);
            e.m.sb(ATTACKING, 0xFF);
            e.b.timer = 1;
            e.m.sw(0xAA, 0);
        }
        let n = e.m.w(0xAA);
        if n == 0 {
            self.enemy_begin_attack(&mut e.b, 0x54);
        }
        if n == 0xA {
            self.enemy_begin_attack(&mut e.b, 0x55);
        }
        if n == 0x14 {
            self.enemy_begin_attack(&mut e.b, 0x56);
        }
        // the mace (+$76): a new frame byte starts animation 0xB and tells the Lash (+$C4 = 1)
        let mh = e.m.w(TARGET);
        let (mace_flag, mace_sub) = {
            let m = self.body(mh);
            (m.param_b, m.sub)
        };
        if mace_flag != e.m.b(0xA8) {
            e.m.sb(0xA8, mace_flag);
            if mace_flag != 0 {
                if let Some(Slot::Enemy(k)) = slot_of(lh) {
                    self.enemies.slots[k].m.sb(0xC4, 1);
                }
                self.enemy_anim(&mut e.b, 0xB);
            }
        }
        if self.body(lh).sub != 0 {
            self.enemy_anim(&mut e.b, 0xA); // 0x3B496 (the Lash's sub was tested above: never reached)
        }
        if mace_sub >= 2 {
            let t = &self.assets.enemy.t;
            let row = CHAIN_ROWS + (lflag as u32) * 8;
            let (mut dx, dy, dz) = (t.i16(row, 0), t.i16(row, 2), t.i16(row, 4));
            if lfacing != 0 {
                dx = dx.wrapping_neg();
            }
            e.b.set_xi_keep(lx.wrapping_add(dx));
            e.b.set_yi_keep(ly.wrapping_add(dy));
            e.b.set_zi_keep(lz.wrapping_add(dz));
        }
        self.enemy_step(&mut e.b); // jsr 0x121E
        self.c8_end(e, h);
    }

    /// 0x3B4F6: the draw check and the monster list.
    fn c8_end(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_monster_register(h); // jmp 0x4C0A
    }
}
