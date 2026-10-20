//! Routines shared by several pool E classes: the hand position of a lifted / held item
//! (0x24A9A), the landing test (0x24CE6), the two "being lifted" states (0x24780, 0x24A2A), the
//! ground idle (0x249DC) and the sounds the items use.

use super::*;

/// Sound requests of the item code (0x9FB6 / 0x9E60 / 0x9FF8 / 0xA404 toggle bits of G.$43B).
impl Game {
    /// `bchg.b #bit,$43B(a5)`: flips the bit and returns what it was.
    fn item_snd_toggle(&mut self, bit: u8) -> bool {
        let old = self.spawn.g.snd_toggle_b >> bit & 1 != 0;
        self.spawn.g.snd_toggle_b ^= 1 << bit;
        old
    }

    /// 0x9FB6: break / landing thud of a container: 0x6E, or 0xF5 when bit 1 of G.$43B was clear.
    /// d1 = the pan byte of the item's X (`x`), d2 = 0.
    pub(super) fn item_snd_9fb6(&mut self, x: i16) {
        let old = self.item_snd_toggle(1);
        let pan = self.snd_pan_x(x) as u32;
        self.sound_ex(if old { 0x6E } else { 0xF5 }, pan, 0);
    }

    /// 0x9E60: 0x6F / 0xF6 (bit 0 of G.$43B): the break sound with the pan of `x` (the same routine as `snd_break`).
    pub(super) fn item_snd_9e60(&mut self, x: i16) {
        let old = self.item_snd_toggle(0);
        let pan = self.snd_pan_x(x) as u32;
        self.sound_ex(if old { 0x6F } else { 0xF6 }, pan, 0);
    }

    /// 0x9FF8: 0x74 / 0xFA (bit 2 of G.$43B). d1 = the pan byte of `x`, d2 = 0.
    pub(super) fn item_snd_9ff8(&mut self, x: i16) {
        let old = self.item_snd_toggle(2);
        let pan = self.snd_pan_x(x) as u32;
        self.sound_ex(if old { 0x74 } else { 0xFA }, pan, 0);
    }

    /// d1 of the gun sounds 0x9C76, 0x9CD8, 0x9D44, 0x9D9C and 0xA442: the mode 3, or 4 when the object's +$24 is nonzero, in the
    /// high word, the middle byte 0x20, and the pan byte of the object's X (`x`, +$08) in the low byte.
    fn item_snd_gun_d1(&self, x: i16, facing: u8) -> u32 {
        let mode = if facing != 0 { 4 } else { 3 };
        (mode << 16) | 0x2000 | self.snd_pan_x(x) as u32
    }

    /// 0x9C76: the shot sound of the rifle. For an object with +$82 != 0 (every item: the pool tag) the fixed code 0xEA; the
    /// player row (table 0x9CD4 by the owner's voice bank) is only for an object with +$82 == 0, which no caller is (the six
    /// callers are the weapon items and the Slice / Morgan style enemies, all with a tag). d1 = `item_snd_gun_d1`, d2 = 0x15000000.
    pub(super) fn item_snd_9c76(&mut self, x: i16, facing: u8) {
        let d1 = self.item_snd_gun_d1(x, facing);
        self.sound_ex(0xEA, d1, 0x1500_0000);
    }

    /// 0x9CD8: the pistol, 0xE7 for an item (the player row 0x9D40 is for +$82 == 0, no caller). d2 = 0x15000000.
    pub fn item_snd_9cd8(&mut self, x: i16, facing: u8) {
        let d1 = self.item_snd_gun_d1(x, facing);
        self.sound_ex(0xE7, d1, 0x1500_0000);
    }

    /// 0x9D44: 0x64 / 0xE3 (bit 7 of G.$43A, shared with the player voices).
    /// d1 = (3, or 4 when +$24 is nonzero) << 16 | 0x2000 | the pan byte of `x`, d2 = 0x2400000.
    pub(super) fn item_snd_9d44(&mut self, x: i16, facing: u8) {
        let old = self.w.snd_toggle >> 7 & 1 != 0;
        self.w.snd_toggle ^= 0x80;
        let mode = if facing != 0 { 4 } else { 3 };
        let pan = self.snd_pan_x(x) as u32;
        self.sound_ex(if old { 0x64 } else { 0xE3 }, (mode << 16) | 0x2000 | pan, 0x0240_0000);
    }

    /// 0x9D9C: the Uzi / M-16, 0xDC for an item (the player row 0x9E04 is for +$82 == 0, no caller). d2 = 0x2400000.
    pub(super) fn item_snd_9d9c(&mut self, x: i16, facing: u8) {
        let d1 = self.item_snd_gun_d1(x, facing);
        self.sound_ex(0xDC, d1, 0x0240_0000);
    }

    /// 0x9E08: the bazooka: by the voice bank (+$118) of the owner (+$70), table 0x9E5C. d1 = the mode 5 (6 when +$24 is
    /// nonzero) in the high word, the middle byte 0x12 and the pan byte of the weapon's X; d2 = 0xC41240.
    pub(super) fn item_snd_9e08(&mut self, it: &Item) {
        let voice = self.item_owner_voice(it);
        let code = self.assets.enemy.t.u8(0x9E5C, voice);
        let mode = if it.b.facing != 0 { 6 } else { 5 };
        let d1 = (mode << 16) | 0x1200 | self.snd_pan_x(it.b.xi()) as u32;
        self.sound_ex(code as u16, d1, 0x00C4_1240);
    }

    /// 0xA442: the shotgun: by the owner's voice bank, words at 0xA498. d1 = `item_snd_gun_d1`, d2 = 0x15000000.
    pub(super) fn item_snd_a442(&mut self, it: &Item) {
        let voice = self.item_owner_voice(it);
        let code = self.assets.enemy.t.u16(0xA498, voice * 2);
        let d1 = self.item_snd_gun_d1(it.b.xi(), it.b.facing);
        self.sound_ex(code, d1, 0x1500_0000);
    }

    /// 0xA396: the statue break, code 0x89. d1 = the pan byte of `x`, d2 = 0.
    pub(super) fn item_snd_a396(&mut self, x: i16) {
        let pan = self.snd_pan_x(x) as u32;
        self.sound_ex(0x89, pan, 0);
    }

    fn item_owner_voice(&self, it: &Item) -> u32 {
        match slot_of(it.b.partner) {
            Some(Slot::Player(p)) => self.players[p].voice as u32,
            _ => 0,
        }
    }

    /// 0xA404: 0x8C / 0x105 (bit 7 of G.$43B).
    pub(super) fn item_snd_a404(&mut self, x: i16) {
        let old = self.item_snd_toggle(7);
        let pan = self.snd_pan_x(x) as u32;
        self.sound_ex(if old { 0x8C } else { 0x105 }, pan, 0);
    }
}

impl Game {
    /// Start animation `id` of the item's own table (0x120E). A table without that animation
    /// (the 68000 would read garbage) keeps the current frame.
    pub(super) fn item_anim(&self, it: &mut Item, id: u16) {
        let s = &self.assets.sprites;
        match s.anim_tables.get(&it.b.anim_table).and_then(|t| t.anims.get(id as usize)).copied().flatten() {
            Some(first) => it.b.load_frame(s, first),
            None => {
                #[cfg(debug_assertions)]
                eprintln!("item animation {id} of table {:#x} is not extracted", it.b.anim_table);
            }
        }
    }

    /// 0x121E: tick the animation, true when it followed its link (carry).
    pub(super) fn item_anim_step(&self, it: &mut Item) -> bool {
        it.b.tick_anim(&self.assets.sprites)
    }

    /// 0x24A9A: put the item into the hand of its owner (player `p`): position from the table
    /// 0x122230 (by character and the attach index of the owner's frame, +$60 high byte),
    /// facing, depth, and the animation picked by the table 0x120A20 (by character, the item's
    /// +$A2 and the attach index). Returns the carry: true = no hand position or no animation
    /// for this frame (the caller hides the item).
    pub(super) fn item_follow_hand(&self, it: &mut Item, p: usize) -> bool {
        let o = &self.players[p].b;
        let t = &self.assets.enemy.t;
        let attach = (o.attach >> 8) as u8;
        if (attach as i8) < 0 {
            return true;
        }
        let pairs = t.u32(0x122230, o.k20 as u32);
        let k = attach as u32 * 4;
        let (mut dx, dy) = (t.i16(pairs, k), t.i16(pairs, k + 2));
        put_hi(&mut it.b.z, o.zi());
        let face = o.facing;
        if face != 0 {
            dx = dx.wrapping_neg();
        }
        let mut x = o.xi().wrapping_add(dx);
        let mut y = o.yi().wrapping_add(dy);
        if o.draw_flags & 1 != 0 {
            y = y.wrapping_add(o.dy3e);
            let d2 = if face != 0 { o.dx3c.wrapping_neg() } else { o.dx3c };
            x = x.wrapping_add(d2);
        }
        put_hi(&mut it.b.x, x);
        put_hi(&mut it.b.y, y);
        it.b.facing = face;
        let table = t.u32(0x120A20, o.k20 as u32);
        let row = t.u32(table, it.m.w(A2) as u32);
        let id = t.u8(row, attach as u32);
        it.b.dir = id;
        if (id as i8) < 0 {
            return true;
        }
        self.item_anim(it, id as u16);
        false
    }

    /// 0x24CE6 landing test of a dropped item that touched the floor: near the screen edges
    /// (X <= camera + 0x10, X >= camera + 0x170) it hops back into the screen (vx 0x180 / -0x100,
    /// airborne again, vy reversed; only the first bounce halves it, at most 5 bounces) and the
    /// carry is clear. Otherwise it stays: carry set. +$81 counts the bounces.
    pub(super) fn item_land_check(&self, it: &mut Item) -> bool {
        let cam = self.cam.s2.cam_x;
        let d0 = cam.wrapping_add(0x410);
        let d1 = (it.b.xi() as u16).wrapping_add(0x400);
        if d0 >= d1 {
            it.b.vx = 0x180;
        } else if d0.wrapping_add(0x160) <= d1 {
            it.b.vx = -0x100;
        } else {
            return true;
        }
        it.b.airborne = 1;
        let mut vy = it.b.vy.wrapping_neg();
        let n = it.m.b(B81);
        if n == 0 {
            vy >>= 1;
        }
        it.b.vy = vy;
        if n >= 5 {
            return true;
        }
        it.m.sb(B81, n + 1);
        false
    }

    /// 0x24780: lifted by a player (sub 4) for weapons, containers and ammo boxes... (see
    /// `item_lift_b` for food / treasure). The owner must be alive, have the same `who` and be in
    /// its pick-up state 0x02000014. Frame byte +$2C of the owner: 0 = still bending, 1 = the
    /// moment of the grab (the item becomes held: sub 8), other = follow the hand. If the owner
    /// left that state the item is dropped (sub 0xC, act 0).
    pub(super) fn item_lift_a(&mut self, it: &mut Item, h: Handle) {
        let pa = match self.item_lift_owner(it) {
            Some(p) => p,
            None => {
                it.set_mode_sub(2, 0xC);
                it.clear_act();
                return self.item_draw_check(it, h);
            }
        };
        let (idx, pa) = pa;
        match pa {
            0 => {}
            1 => {
                it.b.sub = 8;
                it.b.act = 0;
                it.m.sb(B81, 0);
                if it.m.w(KIND) & 0xF00 == 0x100 {
                    self.item_ammo_pickup(it, h);
                }
            }
            _ => {
                if self.item_follow_hand(it, idx) {
                    return; // bcs rts: no draw
                }
            }
        }
        self.item_draw_check(it, h);
    }

    /// 0x24A2A: the lift of food-like items (treasure, ammo box in the table 0x284A4): like
    /// `item_lift_a`, but the frame byte 1 also arms the floor timer (+$A4 = 600) and an owner
    /// that left the pick-up state makes the item "held" as well (sub 8).
    pub(super) fn item_lift_b(&mut self, it: &mut Item, h: Handle) {
        match self.item_lift_owner(it) {
            Some((idx, pa)) => {
                if pa == 0 {
                    return self.item_draw_check(it, h);
                }
                if pa != 1 {
                    if self.item_follow_hand(it, idx) {
                        return;
                    }
                    return self.item_draw_check(it, h);
                }
                it.m.sw(LIFE, 0x258);
            }
            None => {}
        }
        it.b.sub = 8;
        it.b.act = 0;
        it.m.sb(B81, 0);
        if it.m.w(KIND) & 0xF00 == 0x100 {
            self.item_ammo_pickup(it, h);
        }
        self.item_draw_check(it, h)
    }

    /// The test both lift routines start with: `(player index, owner frame byte +$2C)` when the
    /// owner (+$70) is alive, has the same `who` as the item and is in state 0x02000014.
    pub(super) fn item_lift_owner(&self, it: &Item) -> Option<(usize, u8)> {
        let Some(Slot::Player(p)) = slot_of(it.b.partner) else { return None };
        let o = &self.players[p].b;
        if o.live == 0 || o.who != it.b.who || o.state() != 0x0200_0014 {
            return None;
        }
        Some((p, o.param_a))
    }

    /// 0x249DC: an item lying on the floor (sub 0): its lifetime counts down (+$A4, 0 = never),
    /// the animation runs, the frame is copied to +$30, the pickup palette blinks while the frame
    /// byte +$2C is set, and it joins the pickup and attacker lists.
    pub(super) fn item_ground_idle(&mut self, it: &mut Item, h: Handle) {
        let life = it.m.w(LIFE);
        if life != 0 {
            it.m.sw(LIFE, life - 1);
            if life - 1 == 0 {
                it.set_mode_sub(6, 0);
                return;
            }
        }
        self.item_anim_step(it);
        it.b.set_frame2(it.b.frame);
        it.b.palette_override = 0;
        if it.b.param_a != 0 {
            it.b.palette_override = 0x8B;
        }
        self.item_register_pickup(h, it.b.live);
        self.item_draw_check(it, h);
        self.item_register_attacker(h);
        self.item_despawn_far(it);
    }
}
