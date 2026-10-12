//! Pool C (effects, sparks, shots; update 0x237B2) and pool D (special, not ported).
//!
//! An effect is a pool C object (allocator 0x4884, free 0x4964) whose class word +$20 selects
//! its handler from the long table at 0x237E6. Handlers dispatch on +$04 like every object:
//! mode 0 = init (falls through into mode 2 the same frame), mode 2 = run, mode 4/6 = free.
//!
//! Ported classes (+$20):
//! * `0x04` hit spark / impact picture, 0x924AA: plays animation +$26 of table 0x924F6 once.
//! * `0x10` water splash when landing in water, 0x92A48 (spawned by the player code 0x606E).
//! * `0x20` burning flames attached to a body, 0x92FB4 (player 0x1C86E, enemy 0x2AD98).
//! * `0x68` water ripple around a wading body, 0x9631C (player code 0x60E2), raw-list frames.
//!
//! Any other class is freed on its first update (logged once in debug builds) until it is ported.
//!
//! Spawn functions for the other ports (they mirror the original spawn sites; all return
//! `false` when the pool is empty, the 68000 carry):
//! * [`Game::spawn_hit_spark`] = 0x12720 (after a confirmed box hit, spark = attack record byte 6)
//! * [`Game::fx_spawn_hit_spark_alt`] = 0x127FC (2 callers: signed X, +16 Y, depth - 2, facing 0)
//! * [`Game::fx_spawn_spark_at`] = 0x92422 (position given, facing of the calling object)
//! * [`Game::fx_spawn_spark_jitter`] = 0x92450 (position given + jitter by frame counter, facing 0)
//! * [`Game::fx_spawn_spark_simple`] = 0x128C8 (type 0 at x/y, depth and facing of the caller)
//! * [`Game::fx_spawn_water_splash`] = body of 0x606E (caller does the checks, +$83 = 1, sound 0x9AF4)
//! * [`Game::fx_spawn_water_ripple`] = body of 0x60E2 (caller does the checks and the +$97 timer)
//! * [`Game::fx_spawn_flame`] = one iteration of the loops at 0x1C876 / 0x2ADAE

use super::body::Body;
use super::{Game, Handle, POOL_C, Slot, collide, slot_of};
use dino_assets::effect::EffectData;
use dino_assets::{FrameId, HitBox};

/// Effect classes (+$20) ported here.
pub const FX_SPARK: u16 = 0x04;
pub const FX_WATER_SPLASH: u16 = 0x10;
pub const FX_FLAMES: u16 = 0x20;
pub const FX_RIPPLE: u16 = 0x68;

/// Hit spark animation table (0x924BE stores it in +$40).
const SPARK_TABLE: u32 = 0x924F6;
/// Water splash animation table (0x92A72).
const SPLASH_TABLE: u32 = 0x92AC0;
/// Flame frame lists: airborne loop and dying out, for var (+$26) == 0 and != 0.
const FLAMES_AIR: [u32; 2] = [0x93096, 0x930F4];
const FLAMES_END: [u32; 2] = [0x930D4, 0x9313C];
/// Ripple raw-list animation (0x96330).
const RIPPLE_FRAMES: u32 = 0x96360;

#[derive(Clone, Debug, Default)]
pub struct Effect {
    pub b: Body,
    /// +$58 owner (water splash: the body whose splash drawing is suppressed).
    pub owner58: Handle,
    /// +$5A the owner's class word +$20 at spawn time (water splash).
    pub owner_class: u16,
    /// +$5C index of the effect in its spawn loop (flames: start frame = index & 3).
    pub w5c: u16,
    /// +$60 owner of an attached effect (flames).
    pub owner60: Handle,
    /// +$62 / +$64 offset from the owner (X mirrored by the owner's facing).
    pub off_x: i16,
    pub off_y: i16,
}

#[derive(Clone, Debug, Default)]
pub struct Special {
    pub b: Body,
}

/// 0x160A: load a frame record, copying only +$2C..+$2F (no boxes, no extra bytes).
pub fn fx_load(b: &mut Body, fx: &EffectData, id: FrameId) {
    let f = fx.frame(id);
    b.frame = id;
    b.param_a = f.param_a;
    b.param_b = f.param_b;
    b.kind = f.kind;
    b.timer = f.duration;
}

/// 0x15E4: start animation `id` of the effect table in +$40.
pub fn fx_start(b: &mut Body, fx: &EffectData, id: u16) {
    let first = fx.anim(b.anim_table, id);
    fx_load(b, fx, first);
}

/// 0x15F4: tick. Returns true (carry) when the record was the last one and the link was taken.
pub fn fx_tick(b: &mut Body, fx: &EffectData) -> bool {
    b.timer = b.timer.wrapping_sub(1);
    if b.timer != 0 {
        return false;
    }
    let f = fx.frame(b.frame);
    let (next, last) = (f.next, f.last);
    fx_load(b, fx, next);
    last
}

/// 0x16C6: load an 8-byte raw-list record (+$28 and +$2C..+$2F).
pub fn raw_load(b: &mut Body, fx: &EffectData, id: FrameId) {
    let f = fx.raw(id);
    b.frame = id;
    b.param_a = f.param_a;
    b.param_b = f.param_b;
    b.kind = f.kind;
    b.timer = f.duration;
}

/// 0x16AE: tick for 8-byte raw-list records.
pub fn raw_tick(b: &mut Body, fx: &EffectData) -> bool {
    b.timer = b.timer.wrapping_sub(1);
    if b.timer != 0 {
        return false;
    }
    let f = fx.raw(b.frame);
    let (next, last) = (f.next, f.last);
    raw_load(b, fx, next);
    last
}

/// What 0x4964 (and the other free routines) clear: +$00..+$27 only.
fn clear_header(b: &mut Body) {
    b.live = 0;
    b.visible = 0;
    b.who = 0;
    b.id = 0;
    b.set_state(0);
    b.x = 0;
    b.y = 0;
    b.z = 0;
    b.vx = 0;
    b.vy = 0;
    b.vz = 0;
    b.ax = 0;
    b.ay = 0;
    b.az = 0;
    b.k20 = 0;
    b.dir = 0;
    b.palette_override = 0;
    b.facing = 0;
    b.draw_flags = 0;
    b.var26 = 0;
}

/// `move.w dN,$8(a0)`: writes the integer word of a 16.16 position, keeping the fraction.
fn set_word(v: &mut i32, w: i16) {
    *v = (*v & 0xFFFF) | ((w as i32) << 16);
}

/// +$82 pool tag set by the pool init routines: pool E 0xC (0x4C2E), pool A 4 (0x4C5C),
/// pool B 8 (0x4C8A). Players are cleared by 0x4D8E and never set it [inferred].
fn pool_tag(h: Handle) -> u8 {
    match slot_of(h) {
        Some(Slot::Enemy(_)) => 4,
        Some(Slot::Prop(_)) => 8,
        Some(Slot::Item(_)) => 0xC,
        _ => 0,
    }
}

impl Game {
    fn fx_data(&self) -> &'static EffectData {
        &self.assets.effects
    }

    /// 0x4884: take a pool C slot. `None` = pool empty (carry set).
    fn fx_alloc(&mut self) -> Option<usize> {
        self.effects.alloc()
    }

    /// 0x4964: push the slot back and clear +$00..+$27.
    fn fx_free(&mut self, i: usize) {
        self.effects.release(i);
        clear_header(&mut self.effects.slots[i].b);
    }

    /// 0x237B2 pool C update: nothing when all 48 slots are free; else every live slot in
    /// slot order, dispatched by +$20 (table 0x237E6).
    pub fn pool_c_update(&mut self) {
        if self.effects.free_count() == POOL_C.2 {
            return;
        }
        for i in 0..POOL_C.2 {
            if self.effects.slots[i].b.live == 0 {
                continue;
            }
            match self.effects.slots[i].b.k20 {
                FX_SPARK => self.fx_spark(i),
                FX_WATER_SPLASH => self.fx_water_splash(i),
                FX_FLAMES => self.fx_flames(i),
                FX_RIPPLE => self.fx_ripple(i),
                // TODO(port): remaining classes (handler addresses from the table at 0x237E6):
                // 0x00 0x923E0, 0x08 0x92808, 0x0C 0x929B8, 0x14 0x92B7A, 0x18 0x92C9A,
                // 0x1C 0x92ECC, 0x24 0x93184, 0x28 0x932D2, 0x2C 0x934F6 (debris, needs 0x12DAC),
                // 0x30 0x939D6, 0x34 0x93A7A, 0x38 0x93B28, 0x3C 0x93E54, 0x40 0x93FFC,
                // 0x44 0x94100, 0x48 0x942C6, 0x4C 0x94462, 0x50 0x9454E, 0x54 0x94672,
                // 0x58 0x95840, 0x5C 0x9590E, 0x60 0x959E6, 0x64 0x96296, 0x6C 0x963D6,
                // 0x70 0x96488, 0x74 0x965CE, 0x78 0x966B4, 0x7C 0x96800, 0x80 0x96A7E,
                // 0x84 0x96B78, 0x88 0x96DC4, 0x8C 0x96E56, 0x90 0x96FF4, 0x94 0x9720A,
                // 0x98 (null entry), 0x9C 0x97530, 0xA0 0x97674, 0xA4 0x97800.
                // See re/specs/anim/HANDOFF.md. Until then they are freed on their first update.
                other => {
                    #[cfg(debug_assertions)]
                    {
                        use std::sync::atomic::{AtomicU64, Ordering};
                        static SEEN: AtomicU64 = AtomicU64::new(0);
                        let bit = 1u64 << ((other >> 2) & 63);
                        if SEEN.fetch_or(bit, Ordering::Relaxed) & bit == 0 {
                            eprintln!("effect class {other:#x} is not ported yet (freed)");
                        }
                    }
                    let (h, b) = (self.effects.handle(i), self.effects.slots[i].b.clone());
                    if !self.trace_hold(h, &b) {
                        self.fx_free(i);
                    }
                }
            }
        }
    }

    /// 0x924AA hit spark.
    fn fx_spark(&mut self, i: usize) {
        let fx = self.fx_data();
        let h = self.effects.handle(i);
        let b = &mut self.effects.slots[i].b;
        match b.mode {
            0 => {
                // 0x924BE
                b.anim_table = SPARK_TABLE;
                let id = b.var26;
                fx_start(b, fx, id);
                b.visible = 1;
                b.draw_flags = 0x10;
                b.mode = 2;
            }
            2 => {}
            _ => return self.fx_free(i),
        }
        // 0x924E0
        if fx_tick(&mut self.effects.slots[i].b, fx) {
            self.fx_free(i);
        } else {
            self.draw_list_add_if_visible(h);
        }
    }

    /// 0x92A48 water splash. While it lives, the owner's draw-time splash (0x14BC6) is off.
    fn fx_water_splash(&mut self, i: usize) {
        let fx = self.fx_data();
        let h = self.effects.handle(i);
        match self.effects.slots[i].b.mode {
            0 => {
                // 0x92A5C
                let owner = self.effects.slots[i].owner58;
                let mut d0 = 0u16;
                if slot_of(owner).is_some() {
                    let ob = self.body_mut(owner);
                    ob.draw_flags |= 0x20;
                    d0 = (ob.extra[2] & 0xF) as u16;
                    if d0 != 0 {
                        d0 -= 1;
                    }
                }
                let b = &mut self.effects.slots[i].b;
                b.anim_table = SPLASH_TABLE;
                fx_start(b, fx, d0);
                b.visible = 1;
                b.draw_flags = 0x10;
                b.mode = 2;
            }
            2 => {}
            4 => return self.fx_water_splash_end(i),
            _ => return self.fx_free(i),
        }
        // 0x92A90
        if fx_tick(&mut self.effects.slots[i].b, fx) {
            self.fx_water_splash_end(i);
        } else {
            self.draw_list_add_if_visible(h);
        }
    }

    /// 0x92AA0: give the owner its splash drawing back if it is still the same object.
    fn fx_water_splash_end(&mut self, i: usize) {
        let (owner, class) = (self.effects.slots[i].owner58, self.effects.slots[i].owner_class);
        if slot_of(owner).is_some() {
            let ob = self.body_mut(owner);
            if ob.live != 0 && ob.k20 == class {
                ob.draw_flags &= !0x20;
            }
        }
        self.fx_free(i);
    }

    /// 0x92FB4 burning flames attached to +$60 at offset (+$62, +$64).
    fn fx_flames(&mut self, i: usize) {
        let fx = self.fx_data();
        let h = self.effects.handle(i);
        match self.effects.slots[i].b.mode {
            0 => {
                // 0x92FC8: start at frame (index & 3) of the airborne list (10 bytes per record).
                let e = &mut self.effects.slots[i];
                let list = FLAMES_AIR[(e.b.var26 != 0) as usize];
                let first = fx.frame_at(list + 10 * (e.w5c & 3) as u32);
                fx_load(&mut e.b, fx, first);
                e.b.draw_flags = 0x30;
                e.b.mode = 2;
            }
            2 => {}
            _ => return self.fx_free(i),
        }
        // 0x92FF6
        let owner = self.effects.slots[i].owner60;
        if slot_of(owner).is_none() {
            return self.fx_free(i);
        }
        let ob = self.body(owner).clone();
        // tst.b $82(owner) ; beq keep ; cmpi.b #$A,$2E(owner) ; bne free
        if pool_tag(owner) != 0 && ob.kind != 0xA {
            return self.fx_free(i);
        }
        let e = &mut self.effects.slots[i];
        let dx = if ob.facing != 0 { e.off_x.wrapping_neg() } else { e.off_x };
        set_word(&mut e.b.x, dx.wrapping_add(ob.xi()));
        set_word(&mut e.b.y, ob.yi().wrapping_add(e.off_y));
        set_word(&mut e.b.z, ob.zi());
        e.b.facing = ob.facing;
        if e.b.sub == 0 {
            if ob.airborne != 0 {
                // 0x9304A: keep looping while the owner is in the air (carry ignored).
                fx_tick(&mut e.b, fx);
            } else {
                // 0x93054: owner landed: switch to the dying-out list.
                let list = FLAMES_END[(e.b.var26 != 0) as usize];
                fx_load(&mut e.b, fx, fx.frame_at(list));
                e.b.sub = e.b.sub.wrapping_add(2);
                if fx_tick(&mut e.b, fx) {
                    return self.fx_free(i);
                }
            }
        } else if fx_tick(&mut e.b, fx) {
            return self.fx_free(i);
        }
        // 0x93078: drawn on odd frames only (btst #0,$1(a5) = frame counter bit 0).
        if self.w.frame & 1 != 0 {
            self.draw_list_add_check(h);
        }
    }

    /// 0x9631C water ripple (raw sprite list frames, +$25 = 0x50).
    fn fx_ripple(&mut self, i: usize) {
        let fx = self.fx_data();
        let h = self.effects.handle(i);
        let b = &mut self.effects.slots[i].b;
        match b.mode {
            0 => {
                raw_load(b, fx, fx.raw_at(RIPPLE_FRAMES));
                b.draw_flags = 0x50;
                b.visible = 1;
                b.mode = 2;
            }
            2 => {}
            _ => return self.fx_free(i),
        }
        if raw_tick(&mut self.effects.slots[i].b, fx) {
            self.fx_free(i);
        } else {
            self.draw_list_add_if_visible(h);
        }
    }

    // ---------------------------------------------------------------- spawn functions

    fn fx_spawn_spark_slot(&mut self, x: i16, y: i16, z: i16, facing: u8, who: u8, spark: u16) -> bool {
        let Some(i) = self.fx_alloc() else { return false };
        let b = &mut self.effects.slots[i].b;
        b.live = 1;
        b.k20 = FX_SPARK;
        b.who = who;
        b.facing = facing;
        set_word(&mut b.x, x);
        set_word(&mut b.y, y);
        set_word(&mut b.z, z);
        b.var26 = spark;
        true
    }

    /// 0x12720: hit spark between the two boxes of a confirmed hit (`ha` = attacker box,
    /// `hb` = defender box, as returned by `collide::boxes_hit`). `spark` = attack record byte 6.
    /// X = middle of the overlap (unsigned compares with a +0x400 bias), Y = (max of the bottoms
    /// + max of the tops) / 2 (as coded, not the overlap middle), depth = min - 1 (>= 0).
    pub fn spawn_hit_spark(&mut self, attacker: Handle, defender: Handle, ha: HitBox, hb: HitBox, spark: u8) -> bool {
        let (a, d) = (self.body(attacker), self.body(defender));
        let al = collide::left(a, &ha);
        let bl = collide::left(d, &hb);
        let (l0, l1) = (al.wrapping_add(0x400) as u16, bl.wrapping_add(0x400) as u16);
        let (r0, r1) = (al.wrapping_add(ha.xsize).wrapping_add(0x400) as u16, bl.wrapping_add(hb.xsize).wrapping_add(0x400) as u16);
        let l = if l1 < l0 { l0 } else { l1 }; // cmp.w d0,d1 ; bcs
        let r = if r1 > r0 { r0 } else { r1 }; // cmp.w d2,d4 ; bhi
        // add.l d2,d0 ; lsr.l #1,d0 (upper words are 0 at this call) ; subi.w #$400,d0
        let x = (((l as u32 + r as u32) >> 1) as u16).wrapping_sub(0x400) as i16;
        let y = hit_y(a, &ha, d, &hb);
        let mut z = a.zi();
        if d.zi() < z {
            z = d.zi();
        }
        z = z.wrapping_sub(1);
        if z < 0 {
            z = 0;
        }
        let (facing, who) = (a.facing, a.who);
        self.fx_spawn_spark_slot(x, y, z, facing, who, spark as u16)
    }

    /// 0x127FC: variant used by two hit handlers (0x10AAA area): signed compares and a word sum
    /// for X, Y + 0x10, depth = max(min - 1, 0) - 1, facing 0.
    pub fn fx_spawn_hit_spark_alt(&mut self, attacker: Handle, defender: Handle, ha: HitBox, hb: HitBox, spark: u8) -> bool {
        let (a, d) = (self.body(attacker), self.body(defender));
        let al = collide::left(a, &ha);
        let bl = collide::left(d, &hb);
        let (ar, br) = (al.wrapping_add(ha.xsize), bl.wrapping_add(hb.xsize));
        let l = if bl < al { al } else { bl }; // blt
        let r = if br > ar { ar } else { br }; // bgt
        let x = ((l.wrapping_add(r) as u16) >> 1) as i16; // add.w ; lsr.w #1
        let y = hit_y(a, &ha, d, &hb).wrapping_add(0x10);
        let mut z = a.zi();
        if d.zi() < z {
            z = d.zi();
        }
        z = z.wrapping_sub(1);
        if z < 0 {
            z = 0;
        }
        z = z.wrapping_sub(1);
        let who = a.who;
        self.fx_spawn_spark_slot(x, y, z, 0, who, spark as u16)
    }

    /// 0x92422: spark `spark` at (x, y, z) with the facing of object `caller`.
    pub fn fx_spawn_spark_at(&mut self, caller: Handle, x: i16, y: i16, z: i16, spark: u16) -> bool {
        let facing = self.body(caller).facing;
        self.fx_spawn_spark_slot(x, y, z, facing, 0, spark)
    }

    /// 0x92450: spark at (x, y, z) plus a jitter pair from 0x9248A chosen by the frame counter
    /// (G.$0 & 0xE). Facing 0.
    pub fn fx_spawn_spark_jitter(&mut self, x: i16, y: i16, z: i16, spark: u16) -> bool {
        let (jx, jy) = self.fx_data().spark_jitter[((self.w.frame & 0xE) >> 1) as usize];
        self.fx_spawn_spark_slot(x.wrapping_add(jx), y.wrapping_add(jy), z, 0, 0, spark)
    }

    /// 0x128C8: spark type 0 at (x, y) with the depth and facing of object `caller`.
    pub fn fx_spawn_spark_simple(&mut self, caller: Handle, x: i16, y: i16) -> bool {
        let (z, facing) = {
            let b = self.body(caller);
            (b.zi(), b.facing)
        };
        self.fx_spawn_spark_slot(x, y, z, facing, 0, 0)
    }

    /// Spawn part of 0x606E (player code): water splash at the owner's X / depth, height 0x20.
    /// The caller checks (+$83 == 0, +$25 bit 5 clear, +$53 > 0, +$51 != 0, vy < 0, y < 0x20),
    /// then sets +$83 = 1 and requests sound 0x9AF4 when this returns true.
    pub fn fx_spawn_water_splash(&mut self, owner: Handle) -> bool {
        let (x, z, class) = {
            let b = self.body(owner);
            (b.xi(), b.zi(), b.k20)
        };
        let Some(i) = self.fx_alloc() else { return false };
        let e = &mut self.effects.slots[i];
        e.b.live = 1;
        e.b.k20 = FX_WATER_SPLASH;
        set_word(&mut e.b.x, x);
        set_word(&mut e.b.z, z);
        set_word(&mut e.b.y, 0x20);
        e.owner58 = owner;
        e.owner_class = class;
        true
    }

    /// Spawn part of 0x60E2 (player code): ripple at the owner, depth - 2, X jittered by
    /// `(noise & 0x1F) - 0x10` where `noise` is G.$4 (the LFSR word at $FF8004; it is cleared at
    /// game start by 0x9A014 and the per-frame update 0xA9A was not found in play, so 0 is the
    /// expected value). The caller checks +$53 > 0, +$51 == 0, y == 0x18 and the +$97 timer (7).
    pub fn fx_spawn_water_ripple(&mut self, owner: Handle, noise: u16) -> bool {
        let (x, y, z) = {
            let b = self.body(owner);
            (b.xi(), b.yi(), b.zi())
        };
        let Some(i) = self.fx_alloc() else { return false };
        let b = &mut self.effects.slots[i].b;
        b.live = 1;
        b.k20 = FX_RIPPLE;
        let jitter = ((noise & 0x1F) as i16).wrapping_sub(0x10);
        set_word(&mut b.x, x.wrapping_add(jitter));
        set_word(&mut b.y, y);
        set_word(&mut b.z, z.wrapping_sub(2));
        true
    }

    /// One flame of the loops at 0x1C876 (player, 2 flames from the table at 0x1C8A6) and
    /// 0x2ADAE (enemy, table 0x2CA36 by +$84): owner, offset, var (+$26) and loop index (+$5C).
    pub fn fx_spawn_flame(&mut self, owner: Handle, off_x: i16, off_y: i16, var: u16, index: u16) -> bool {
        let Some(i) = self.fx_alloc() else { return false };
        let e = &mut self.effects.slots[i];
        e.b.live = 1;
        e.b.k20 = FX_FLAMES;
        e.owner60 = owner;
        e.off_x = off_x;
        e.off_y = off_y;
        e.b.var26 = var;
        e.w5c = index;
        true
    }
}

/// Y of 0x12720 / 0x127FC: d1 = max(bottoms), d3 = max(tops) (signed blt), (d1 + d3) >> 1 (lsr.w).
fn hit_y(a: &Body, ha: &HitBox, d: &Body, hb: &HitBox) -> i16 {
    let (ab, bb) = (a.yi().wrapping_add(ha.yoff), d.yi().wrapping_add(hb.yoff));
    let (at, bt) = (ab.wrapping_add(ha.ysize), bb.wrapping_add(hb.ysize));
    let bottom = if bb < ab { ab } else { bb };
    let top = if bt < at { at } else { bt };
    ((bottom.wrapping_add(top) as u16) >> 1) as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(x: i16, y: i16, z: i16, facing: u8) -> Body {
        let mut b = Body::default();
        b.set_xi(x);
        b.set_yi(y);
        b.set_zi(z);
        b.facing = facing;
        b
    }

    #[test]
    fn hit_y_uses_max_of_tops() {
        // Jack jab box 0x02 (yoff 29, ysize 42) against body box 0x1A (0, 75).
        let a = body(100, 0, 50, 0);
        let d = body(60, 0, 52, 1);
        let ha = HitBox { zoff: -1, zsize: 10, xoff: -68, xsize: 59, yoff: 29, ysize: 42 };
        let hb = HitBox { zoff: 1, zsize: 6, xoff: -12, xsize: 23, yoff: 0, ysize: 75 };
        // bottoms 29 / 0 -> 29 ; tops 71 / 75 -> 75 ; (29 + 75) >> 1 = 52
        assert_eq!(hit_y(&a, &ha, &d, &hb), 52);
    }

    #[test]
    fn clear_header_keeps_fields_past_0x27() {
        let mut b = body(1, 2, 3, 1);
        b.live = 1;
        b.k20 = 4;
        b.frame = 7;
        b.timer = 9;
        clear_header(&mut b);
        assert_eq!((b.live, b.k20, b.x, b.facing), (0, 0, 0, 0));
        assert_eq!((b.frame, b.timer), (7, 9));
    }
}
