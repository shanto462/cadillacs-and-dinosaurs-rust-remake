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
    pub(super) fn item_snd_9fb6(&mut self) {
        let old = self.item_snd_toggle(1);
        self.sound(if old { 0x6E } else { 0xF5 });
    }

    /// 0x9E60: 0x6F / 0xF6 (bit 0 of G.$43B).
    pub(super) fn item_snd_9e60(&mut self) {
        let old = self.item_snd_toggle(0);
        self.sound(if old { 0x6F } else { 0xF6 });
    }

    /// 0x9FF8: 0x74 / 0xFA (bit 2 of G.$43B).
    pub(super) fn item_snd_9ff8(&mut self) {
        let old = self.item_snd_toggle(2);
        self.sound(if old { 0x74 } else { 0xFA });
    }

    /// 0x9C76: the thrower's shot sound; for an item (+$82 != 0) the fixed code 0xEA.
    pub(super) fn item_snd_9c76(&mut self) {
        self.sound(0xEA);
    }

    /// 0x9CD8: 0xE7 for an item.
    pub(super) fn item_snd_9cd8(&mut self) {
        self.sound(0xE7);
    }

    /// 0x9D44: 0x64 / 0xE3 (bit 7 of G.$43A, shared with the player voices).
    pub(super) fn item_snd_9d44(&mut self) {
        let old = self.w.snd_toggle >> 7 & 1 != 0;
        self.w.snd_toggle ^= 0x80;
        self.sound(if old { 0x64 } else { 0xE3 });
    }

    /// 0x9D9C: 0xDC for an item.
    pub(super) fn item_snd_9d9c(&mut self) {
        self.sound(0xDC);
    }

    /// 0x9E08: the bazooka: by the voice bank (+$118) of the owner (+$70), table 0x9E5C.
    pub(super) fn item_snd_9e08(&mut self, it: &Item) {
        let voice = self.item_owner_voice(it);
        let code = self.assets.enemy.t.u8(0x9E5C, voice);
        self.sound(code as u16);
    }

    /// 0xA442: the shotgun: by the owner's voice bank, words at 0xA498.
    pub(super) fn item_snd_a442(&mut self, it: &Item) {
        let voice = self.item_owner_voice(it);
        let code = self.assets.enemy.t.u16(0xA498, voice * 2);
        self.sound(code);
    }

    fn item_owner_voice(&self, it: &Item) -> u32 {
        match slot_of(it.b.partner) {
            Some(Slot::Player(p)) => self.players[p].voice as u32,
            _ => 0,
        }
    }

    /// 0xA404: 0x8C / 0x105 (bit 7 of G.$43B).
    pub(super) fn item_snd_a404(&mut self) {
        let old = self.item_snd_toggle(7);
        self.sound(if old { 0x8C } else { 0x105 });
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
        it.b.frame2 = it.b.frame;
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
