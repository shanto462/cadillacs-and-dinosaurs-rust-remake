//! Bludge (class 6), part 3 (M7-09): it eats the items that lie on the floor (the only enemy that looks at the item list) and gives them back.
//!
//! * 0x3AFFA / 0x3B006 (`enemy_c6_claim_item`): scan the item list (G.$69EE, the count, G.$69F0 the entries): an entry that is alive, not a container
//!   (0x20A) and not claimed (`+$AF`) is a candidate, the one with the lowest category byte (the high byte of +$A0: 0 food, 1 ammo, 2 weapon, 3 treasure)
//!   is claimed (`+$AF` = 1, handle in `+$D0`); `+$D8` is the number of entries that are containers or claimed, `+$B6` = 0xFF when all of them are.
//! * 0x3AFA4 (`enemy_c6_check_eat`): less than 8 items in the stomach (`+$D6`), `+$B6` allows it (it is cleared when a new item came), a roll against the
//!   byte 0x3AFF2 by aggression and some item around (0x32762): the state 8.
//! * The state 8 (0x39AD2): walk to a point beside the item (60 frames at most), wait by level, eat: the item is put into the ring of 8 words at the
//!   start of the pool D block (`+$D2` the end, `+$D4` the write pointer, `+$D6` the count) and the object is switched off (`+$00` = 0, its slot is kept).
//! * 0x3B086: the last item comes back (alive, state 0, act 2, at the Bludge's place) when the Bludge is hit while it eats and when it dies; 0x3B0D8: on
//!   death every other item of the ring gets the mode 6 (the item code frees it).

use super::Enemy;
use crate::game::{Game, Slot, slot_of};

/// 0x3AFF2: the chance of the roll by aggression.
const EAT_CHANCE: u32 = 0x3AFF2;
/// 0x39B7E: the wait before the bite by level.
const BITE_WAIT: u32 = 0x39B7E;

impl Game {
    /// 0x3AFFA: claim the item of the lowest category.
    pub(super) fn enemy_c6_claim_item(&mut self, e: &mut Enemy) {
        let list = self.combat.get(0x69EE).to_vec();
        if list.is_empty() {
            return; // 0x3B006: beq 0x3B084
        }
        let mut count = 0u16;
        let mut best = 0xFFu8;
        let mut claim: Option<usize> = None;
        for &h in &list {
            let Some(Slot::Item(i)) = slot_of(h) else { continue };
            let it = &self.items.slots[i];
            if it.b.live == 0 {
                continue;
            }
            if it.m.w(0xA0) == 0x20A || it.m.b(0xAF) != 0 {
                count += 1;
                continue;
            }
            let category = (it.m.w(0xA0) >> 8) as u8; // move.b $A0(a0),d0: the high byte of the word
            if category < best {
                best = category;
                claim = Some(i);
            }
        }
        if count as usize == list.len() {
            e.m.sb(0xB6, 0xFF);
        }
        e.m.sw(0xD8, count);
        match claim {
            Some(i) => {
                e.m.sw(0xD0, self.items.handle(i));
                self.items.slots[i].m.sb(0xAF, 1);
            }
            None => e.m.sw(0xD0, 0),
        }
    }

    /// 0x3AFA4: the Bludge goes to eat (carry).
    pub(super) fn enemy_c6_check_eat(&mut self, e: &mut Enemy) -> bool {
        if (e.m.w(0xD6) as i16) >= 8 {
            return false;
        }
        if (e.m.b(0xB6) as i8) < 0 && (self.combat.get(0x69EE).len() as i16) > e.m.w(0xD8) as i16 {
            e.m.sb(0xB6, 0);
        }
        if e.m.b(0xB6) != 0 {
            return false;
        }
        let limit = self.assets.enemy.t.u8(EAT_CHANCE, self.rank.aggression as u32 & 7);
        if ((self.rand() & 0x1F) as u8 as i8) >= limit as i8 {
            return false;
        }
        self.script_floor_items(true) // jsr 0x32762
    }

    /// 0x39AC8: the state 8 (act 8, step 0).
    pub(super) fn enemy_c6_start_eat(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0800);
    }

    /// 0x39AD2: act 8, the step: 0 / 2 the start, 4 the walk, 6 the wait, 8 the bite.
    pub(super) fn enemy_c6_state_eat(&mut self, e: &mut Enemy) {
        match e.b.step {
            0 => {
                e.b.step += 2; // 0x39AEA
                if e.m.w(0xD0) == 0 {
                    return self.enemy_c6_eat_end(e);
                }
                self.enemy_c6_eat_start(e);
            }
            2 => self.enemy_c6_eat_start(e),
            4 => self.enemy_c6_eat_walk(e),
            6 => self.enemy_c6_eat_wait(e),
            _ => self.enemy_c6_eat_bite(e),
        }
    }

    /// 0x39CA2.
    fn enemy_c6_eat_end(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0000);
    }

    /// 0x39AF6.
    fn enemy_c6_eat_start(&mut self, e: &mut Enemy) {
        e.b.step += 2;
        e.m.sb(0xAA, 0xF0);
        self.enemy_anim(&mut e.b, 0);
        self.enemy_c6_set_speed_by_level(e); // bra 0x3ACB2
    }

    /// The claimed item while it still lies there (alive, in the state 0x0200): its handle.
    fn c6_eat_item(&self, e: &Enemy) -> Option<crate::game::Handle> {
        let h = e.m.w(0xD0);
        let b = self.body(h);
        (b.live != 0 && b.mode == 2 && b.sub == 0).then_some(h)
    }

    /// 0x39B0C: the walk to the point beside the item.
    fn enemy_c6_eat_walk(&mut self, e: &mut Enemy) {
        let Some(item) = self.c6_eat_item(e) else {
            return self.enemy_c6_eat_end(e);
        };
        self.enemy_c6_eat_aim(e, item); // 0x3AAB6
        let (aa, borrow) = e.m.b(0xAA).overflowing_sub(1);
        e.m.sb(0xAA, aa);
        if borrow {
            return self.enemy_c6_eat_end(e);
        }
        if self.c6_eat_near(e) {
            e.b.step += 2; // 0x39B40
            self.enemy_anim(&mut e.b, 0xF);
            let wait = self.assets.enemy.t.u16(BITE_WAIT, (self.w.level & 0x1F) as u32 * 2);
            e.m.sw(0x80, wait);
            return self.enemy_c6_eat_wait(e); // falls into 0x39B5C
        }
        self.enemy_move_dir(&mut e.b); // 0x1862
        self.enemy_walk_anim(&mut e.b); // 0x324D4
    }

    /// 0x3AAB6: the point 0x44 pixels beside the item (on the side of the screen centre) and the facing toward it.
    fn enemy_c6_eat_aim(&mut self, e: &mut Enemy, item: crate::game::Handle) {
        let (ix, iz) = {
            let b = self.body(item);
            (b.xi(), b.zi())
        };
        let side = (self.cam.s2.cam_x as i16).wrapping_add(0xC0).wrapping_sub(ix);
        let d1: i16 = if side >= 0 { 0x44 } else { -0x44 };
        e.m.sw(0x88, ix.wrapping_add(d1) as u16);
        e.m.sw(0x8A, iz as u16);
        self.enemy_c6_steer_angle(e);
        e.b.facing = (ix >= e.b.xi()) as u8;
    }

    /// 0x3ABBE: at the point (the box 24 by 10 pixels, the Z window is one sided: -10 to -1).
    fn c6_eat_near(&self, e: &Enemy) -> bool {
        let (dest_x, dest_z) = (e.m.w(0x88) as i16, e.m.w(0x8A) as i16);
        (e.b.xi().wrapping_sub(dest_x).wrapping_add(0xC) as u16) < 0x18 && (e.b.zi().wrapping_sub(dest_z).wrapping_add(0xA) as u16) < 0xA
    }

    /// 0x39B5C: the wait before the bite (+$80 counts down, the item must stay).
    fn enemy_c6_eat_wait(&mut self, e: &mut Enemy) {
        let (t, borrow) = e.m.w(0x80).overflowing_sub(1);
        e.m.sw(0x80, t);
        if borrow {
            e.b.step += 2; // 0x39BBE
            self.enemy_anim(&mut e.b, 0x10);
            e.m.sb(0xDA, 1);
            e.b.stun = 1;
            return self.enemy_c6_eat_bite(e);
        }
        if self.c6_eat_item(e).is_none() {
            return self.enemy_c6_eat_end(e);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x39BD6: the bite: at the frame flag the item goes into the stomach; at the end of the animation the Bludge is a normal one again.
    fn enemy_c6_eat_bite(&mut self, e: &mut Enemy) {
        if e.b.param_b != 0 {
            e.b.param_b = 0;
            self.enemy_c6_swallow(e);
        }
        if self.enemy_step(&mut e.b) {
            // 0x39BF0: G.$0 = 0x17: the anti tamper scan 0x86B3C finds nothing in an original program (no carry).
            e.m.sb(0xDA, 0);
            e.b.stun = 0xA;
            self.enemy_c6_eat_end(e);
        }
    }

    // ---------------------------------------------------------------- the stomach (the pool D block of +$98)

    /// The block of the stomach and the offset of an address in it.
    fn c6_stomach(&self, e: &Enemy) -> Option<usize> {
        match slot_of(e.m.w(0x98)) {
            Some(Slot::Special(s)) => Some(s),
            _ => None,
        }
    }

    /// 0x3B05E: the item goes into the ring (the write pointer starts again at the end every 8 items) and is switched off.
    fn enemy_c6_swallow(&mut self, e: &mut Enemy) {
        let Some(s) = self.c6_stomach(e) else { return };
        let base = e.m.w(0x98);
        if e.m.w(0xD6) & 7 == 0 {
            e.m.sw(0xD4, e.m.w(0xD2));
        }
        let item = e.m.w(0xD0);
        let ptr = e.m.w(0xD4).wrapping_sub(2);
        self.specials.slots[s].m.sw(ptr.wrapping_sub(base) as usize, item);
        e.m.sw(0xD4, ptr);
        e.m.sw(0xD6, e.m.w(0xD6).wrapping_add(1));
        if let Some(Slot::Item(i)) = slot_of(item) {
            self.items.slots[i].b.live = 0; // clr.b 0(a0)
        }
    }

    /// 0x3B086: the item that was eaten last comes back where the Bludge stands (alive, state 0, act 2).
    pub(super) fn enemy_c6_give_back(&mut self, e: &mut Enemy) {
        if e.m.b(0xB6) == 1 || e.m.w(0xD6) == 0 {
            return;
        }
        let Some(s) = self.c6_stomach(e) else { return };
        let base = e.m.w(0x98);
        let at = e.m.w(0xD4);
        let item = self.specials.slots[s].m.w(at.wrapping_sub(base) as usize);
        let mut next = at.wrapping_add(2);
        e.m.sw(0xD6, e.m.w(0xD6).wrapping_sub(1));
        if next == e.m.w(0xD2) {
            next = base;
        }
        e.m.sw(0xD4, next);
        self.specials.slots[s].m.sw(at.wrapping_sub(base) as usize, 0);
        if let Some(Slot::Item(i)) = slot_of(item) {
            let (x, y, z) = (e.b.xi(), e.b.yi(), e.b.zi());
            let it = &mut self.items.slots[i];
            it.b.live = 1;
            it.b.set_state(0);
            it.b.act = 2;
            it.b.set_xi(x);
            it.b.set_yi(y);
            it.b.set_zi(z);
        }
    }

    /// 0x3B0D8: on death the other items of the ring get the mode 6 (freed by the item code).
    pub(super) fn enemy_c6_give_all(&mut self, e: &mut Enemy) {
        if e.m.b(0xB6) == 1 || e.m.w(0xD6) == 0 {
            return;
        }
        let Some(s) = self.c6_stomach(e) else { return };
        let end = e.m.w(0xD2).wrapping_sub(e.m.w(0x98)) as usize;
        for k in 1..=8usize {
            let item = self.specials.slots[s].m.w(end - 2 * k);
            if item == 0 {
                continue;
            }
            if let Some(Slot::Item(i)) = slot_of(item) {
                self.items.slots[i].b.live = 1;
                self.items.slots[i].b.mode = 6;
            }
        }
    }
}
