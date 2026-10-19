//! The part 0 of the Bludge (class 0, handler 0x33334): the tongue (M7-08). The Bludge (the parent, `+$A0`) drives it with its own bytes: `+$C3` 1 the
//! tongue goes out, 2 it holds a hero, 3 it comes back, 0 it is in; `+$C2` 1 a hero is held; `+$C4` 1 the hero was carried to the mouth. The tongue is a
//! raw sprite list in its pool D block (`+$98`, the pointer `+$A4` is the count word of the list): one segment each 16 pixels of length. It moves with the
//! speed 0x600 (0xA00 back), is at most 0xC8 pixels long and grabs the hero the box 1 of its table 0x337EA touches.

use super::Enemy;
use crate::game::{Game, Handle, Slot, collide, slot_of};

/// 0x337DA: the tongue of one segment; 0x337E2: the tongue with the grabbed hero (8 bytes each, written over the first segment).
const TIP: u32 = 0x337DA;
const HELD: u32 = 0x337E2;
/// The hit box table of the part 0 (box 1 is the tongue).
const BOX_TABLE: u32 = 0x337EA;

impl Game {
    /// 0x333D0: mode 2 of the part 0.
    pub(super) fn enemy_part0_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        let parent_h = e.m.w(0xA0);
        let Some(Slot::Enemy(pi)) = slot_of(parent_h) else {
            return self.enemy_part0_free(e, i);
        };
        let (hp, mode, sub, kind) = {
            let p = &self.enemies.slots[pi].b;
            (p.hp, p.mode, p.sub, p.kind)
        };
        if hp == 0 || (mode as i8) >= 4 {
            return self.enemy_part0_free(e, i);
        }
        if kind == 0xE || (mode, sub) == (2, 2) {
            self.enemies.slots[pi].m.sb(0xC3, 0);
            if self.enemies.slots[pi].m.b(0xC2) != 0 {
                e.b.sub = 0;
            }
        }
        let c3 = self.enemies.slots[pi].m.b(0xC3);
        if c3 == 0 {
            e.m.sb(0xA6, 0); // 0x33436
            return;
        }
        if c3 != e.m.b(0xA6) {
            e.b.act = 0;
        }
        e.m.sb(0xA6, c3);
        match c3 {
            1 => self.part0_out(e, h, pi),
            2 => self.part0_hold(e, pi),
            3 => self.part0_back(e, pi),
            _ => {}
        }
        if e.b.visible != 0 {
            self.draw_list.push(((e.b.zi() as u16 as u32) << 16) | h as u32); // 0x49C0
        }
        self.enemy_push_hurtable(h); // 0x4BEA
    }

    fn parent_c(&mut self, pi: usize, off: usize, v: u8) {
        self.enemies.slots[pi].m.sb(off, v);
    }

    /// 0x33444: the tongue goes out.
    fn part0_out(&mut self, e: &mut Enemy, h: Handle, pi: usize) {
        if e.b.act == 0 {
            e.b.act += 2;
            let (px, py, pz, pfacing, pfloor) = {
                let p = &self.enemies.slots[pi].b;
                (p.xi(), p.yi(), p.zi(), p.facing, p.floor)
            };
            self.parent_c(pi, 0xC4, 0);
            e.b.floor = pfloor;
            e.b.facing = pfacing;
            let x = px.wrapping_add(if pfacing != 0 { 0x50 } else { -0x50 });
            e.b.set_xi(x);
            e.m.sw(0xA2, x as u16);
            e.b.set_yi(py.wrapping_add(0x30));
            e.b.set_zi(pz);
            e.b.vx = if pfacing != 0 { 0x600 } else { -0x600 };
            self.part0_list_reset(e);
            return;
        }
        // 0x334A0
        e.b.motion(); // 0x189C
        self.part0_limit(e, pi);
        self.part0_count(e);
        if !self.part0_touch(e, pi) {
            return;
        }
        // 0x334BA: a hero touched: he is grabbed when he stands free on the ground, else the tongue goes back.
        if !self.part0_can_grab(e, pi) {
            self.parent_c(pi, 0xC3, 3);
            return;
        }
        self.part0_grab(e, h, pi);
        self.parent_c(pi, 0xC2, 1);
        self.parent_c(pi, 0xC3, 2);
        e.m.sb(0x80, 0x64);
        self.part0_list_held(e);
    }

    /// 0x334E4: the tongue holds him: the hero must keep the action 0x40; 100 frames, then it goes back.
    fn part0_hold(&mut self, e: &mut Enemy, pi: usize) {
        let target = self.enemies.slots[pi].m.w(0x76);
        if self.part0_action(target) != 0x40 {
            return self.part0_release(e, pi);
        }
        let (t, _) = e.m.b(0x80).overflowing_sub(1);
        e.m.sb(0x80, t);
        if t == 0 {
            self.parent_c(pi, 0xC3, 3);
        }
    }

    /// 0x33500: the tongue comes back.
    fn part0_back(&mut self, e: &mut Enemy, pi: usize) {
        if e.b.act == 0 {
            e.b.act += 2;
            let pfacing = self.enemies.slots[pi].b.facing;
            e.b.facing = pfacing;
            let d1: i16 = if pfacing != 0 { -0x58 } else { 0x58 };
            e.b.set_xi(e.b.xi().wrapping_add(d1));
            let a2 = (e.m.w(0xA2) as i16).wrapping_add(d1);
            e.m.sw(0xA2, a2 as u16);
            e.b.vx = if e.b.facing != 0 { -0xA00 } else { 0xA00 };
            return;
        }
        // 0x33536
        if self.enemies.slots[pi].m.b(0xC2) != 0 {
            let target = self.enemies.slots[pi].m.w(0x76);
            if self.part0_action(target) != 0x40 {
                return self.part0_release(e, pi);
            }
        }
        e.b.motion();
        self.part0_arrive(e, pi);
        self.part0_count(e);
    }

    /// The action byte (+$C8) of the hero the tongue holds.
    fn part0_action(&self, target: Handle) -> u8 {
        match slot_of(target) {
            Some(Slot::Player(p)) => self.players[p].action,
            _ => 0,
        }
    }

    /// 0x33598: a tongue 0xC8 pixels long goes back.
    fn part0_limit(&mut self, e: &mut Enemy, pi: usize) {
        let d = (e.m.w(0xA2) as i16).wrapping_sub(e.b.xi());
        if d.wrapping_abs() >= 0xC8 {
            self.parent_c(pi, 0xC3, 3);
        }
    }

    /// 0x335B2: the tongue is back at its anchor: with a hero it let him go at 0x60 pixels, without it the tongue is in.
    fn part0_arrive(&mut self, e: &mut Enemy, pi: usize) {
        let a2 = e.m.w(0xA2) as i16;
        let x = e.b.xi();
        if self.enemies.slots[pi].m.b(0xC2) != 0 {
            let reached = if e.b.facing == 0 { a2.wrapping_sub(0x60) <= x } else { a2.wrapping_add(0x60) >= x };
            if reached {
                self.parent_c(pi, 0xC4, 1); // 0x335DA
                e.b.kind = 0;
                self.part0_release(e, pi);
            }
            return;
        }
        let reached = if e.b.facing == 0 { a2 <= x } else { a2 >= x };
        if reached {
            self.parent_c(pi, 0xC3, 0); // 0x33608
        }
    }

    /// 0x3360E: the segments of the list: the length / 16 + 2.
    fn part0_count(&mut self, e: &mut Enemy) {
        let d = (e.m.w(0xA2) as i16).wrapping_sub(e.b.xi()).wrapping_abs() as u16;
        self.part0_special_w(e, (d >> 4) + 2);
    }

    /// 0x33626: the list of one segment (the count 1, the 8 bytes of the tip).
    fn part0_list_reset(&mut self, e: &mut Enemy) {
        self.part0_special_w(e, 1);
        self.part0_special_copy(e, TIP);
    }

    /// 0x33638: the first segment becomes the tip with the hero.
    fn part0_list_held(&mut self, e: &mut Enemy) {
        self.part0_special_copy(e, HELD);
    }

    /// The pool D block of the tongue (`+$98`).
    fn part0_special(&self, e: &Enemy) -> Option<usize> {
        match slot_of(e.m.w(0x98)) {
            Some(Slot::Special(s)) => Some(s),
            _ => None,
        }
    }

    /// The count word of the list (the block +$0C, `+$A4` points at it).
    fn part0_special_w(&mut self, e: &Enemy, v: u16) {
        if let Some(s) = self.part0_special(e) {
            self.specials.slots[s].m.sw(0xC, v);
        }
    }

    /// The first segment (+$0E, 8 bytes) from the ROM.
    fn part0_special_copy(&mut self, e: &Enemy, from: u32) {
        if let Some(s) = self.part0_special(e) {
            for k in 0..8u32 {
                let b = self.assets.enemy.t.u8(from, k);
                self.specials.slots[s].m.sb(0xE + k as usize, b);
            }
        }
    }

    /// 0x33648: the hero is caught: the part 0 holds him (sub 4, kind 0xC), the hero is held (state 0x02060000, action 0x40, frame kind 0xE).
    fn part0_grab(&mut self, e: &mut Enemy, h: Handle, pi: usize) {
        let target = self.enemies.slots[pi].m.w(0x76);
        e.b.sub = 4;
        e.m.sw(0xA8, 0x28);
        e.b.partner = target;
        e.b.kind = 0xC;
        if let Some(Slot::Player(p)) = slot_of(target) {
            let pl = &mut self.players[p];
            pl.b.partner = h; // move.w a6,$70(a0)
            pl.b.set_state(0x0206_0000);
            pl.action = 0x40;
            pl.b.kind = 0xE;
            pl.bc7 = 0;
            pl.b.hp_prev = pl.b.hp;
        }
    }

    /// 0x33688: the tongue lets the hero go and comes back.
    fn part0_release(&mut self, e: &mut Enemy, pi: usize) {
        e.b.sub = 0;
        self.parent_c(pi, 0xC3, 3);
        self.parent_c(pi, 0xC2, 0);
        self.part0_list_reset(e);
    }

    /// 0x33738: the target can be grabbed: alive (+$00 = 1), in the state 0x0200, not in the air (carry set means no).
    fn part0_can_grab(&self, e: &Enemy, pi: usize) -> bool {
        let target = self.enemies.slots[pi].m.w(0x76);
        let _ = e;
        let tb = self.body(target);
        tb.live == 1 && tb.mode == 2 && tb.sub == 0 && tb.airborne == 0
    }

    /// 0x33698: the box 1 of the tongue against the body box of the hero (z, x, y).
    fn part0_touch(&self, e: &Enemy, pi: usize) -> bool {
        let target = self.enemies.slots[pi].m.w(0x76);
        let tb = self.body(target);
        let t = &self.assets.enemy.t;
        let k = 12; // the record 1
        let own = dino_assets::HitBox {
            zoff: t.i16(BOX_TABLE, k),
            zsize: t.i16(BOX_TABLE, k + 2),
            xoff: t.i16(BOX_TABLE, k + 4),
            xsize: t.i16(BOX_TABLE, k + 6),
            yoff: t.i16(BOX_TABLE, k + 8),
            ysize: t.i16(BOX_TABLE, k + 10),
        };
        collide::boxes(&self.assets.sprites, tb, tb.body_box).iter().any(|hv| collide::box_hits(tb, hv, &e.b, &own))
    }
}
