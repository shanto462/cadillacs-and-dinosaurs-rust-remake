//! Class 5: breakable, liftable, throwable containers (handler 0x28C76). Stage scripts spawn
//! them with variant (+$26) 0, 2, 4, 6, 8, 0xA, 0xC and a drop row (+$96, table 0xB64F2). A
//! non-zero +$5C makes an explosive barrel (HP 50, a fuse, an explosion instead of debris).
//!
//! * mode 0 init (0x28C8A): animation table by variant, kind 0x20A, box table 0xC0100000, HP 4,
//!   attack id +$58 from the table 0x29860 (the attack it hits with when thrown), then it falls
//!   into the alive handler the same frame.
//! * mode 2 alive, sub (+$05): 0 idle (0x28E00), 4 lifted (0x24780), 8 held (0x28F76), 0xC dropped
//!   (0x29056), 0x10 thrown (0x2909A, per variant).
//! * mode 4 break (0x296AC, per variant): +500 points, the drop roll, debris, sounds.
//! * mode 6 free (0x29852).
//!
//! Lists (0x28F3A / 0x28F52 / 0x28F64): variants 0, 2, 4, 0xC are hurtable, attackers, solid
//! (the terrain obstacle list 0x4A96) and liftable (item list 0x4AC2); 6 and 8 are not solid;
//! 0xA is solid but not liftable.
//!
//! Hits arrive from the passes in `hits.rs`: HP drops, +$00 becomes 2 (hit), +$59 the hit stop
//! that shakes the container (X = home +-1). HP 0 breaks it (mode 4).

use super::*;
use crate::game::effect::Effect;

/// Animation tables by variant / 2 (init 0x28C8A..0x28D1C).
const ANIMS: [u32; 7] = [0x2995E, 0x2998C, 0x299FE, 0x29A66, 0x29A86, 0x29AA6, 0x29AC8];
/// Fuse stages (0x28F24: blink count, 0x28F28: blink period).
const FUSE_COUNT: u32 = 0x28F24;
const FUSE_PERIOD: u32 = 0x28F28;
/// Debris lists of the breaking variants (5 words per piece: dx, dy, vx, vy, picture).
const DEBRIS_B: u32 = 0x295F4;
const DEBRIS_C: u32 = 0x29650;
/// Chance masks of a debris piece carrying a bonus, by the number already chosen (0x29552).
const DEBRIS_MASKS: u32 = 0x29552;

impl Game {
    /// 0x28C76 container.
    pub(super) fn item_container(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.mode {
            0 => {
                if !self.container_init(it, h) {
                    return;
                }
                self.container_alive(it, h, i);
            }
            2 => self.container_alive(it, h, i),
            4 => self.container_break(it, h, i),
            6 => self.item_free(it, i),
            _ => {}
        }
    }

    /// 0x28C8A init. Returns false for a variant the 68000 has no code for (it would jump into
    /// data); the object is freed then.
    fn container_init(&mut self, it: &mut Item, h: Handle) -> bool {
        let k = (it.b.var26 / 2) as usize;
        if it.b.var26 % 2 != 0 || k >= ANIMS.len() {
            it.b.mode = 6;
            return false;
        }
        it.b.anim_table = ANIMS[k];
        it.m.sw(KIND, 0x20A);
        it.b.box_table = 0xC010_0000;
        it.b.multi_box = false;
        self.item_snap_to_floor(it, h); // 0x3140C
        it.m.sw(A2, 0x3C);
        it.b.draw_flags = 0x10;
        it.m.sw(LIFE, 0);
        it.m.sb(WEAR, 0);
        it.m.sb(A7, 0);
        it.b.hitstop = 0;
        it.b.id = 0;
        it.m.sb(B0, 0);
        it.m.sb(B6, 0);
        it.m.sb(B7, 0);
        it.m.sb(B8, 0);
        it.m.sb(AC, 0);
        it.m.sb(B9, 0);
        it.m.sb(AF, 0);
        it.m.sw(B2, it.b.xi() as u16); // home position for the hit shake
        it.m.sw(B4, it.b.zi() as u16);
        it.b.hp_bar = 4;
        it.b.hp = 4;
        it.b.hp_prev = 4;
        if it.m.w(W5C) != 0 {
            it.b.hp = 0x32;
            it.b.hp_prev = 0x32;
            it.b.hp_bar = 0x64;
            it.m.sb(B80, 0x32);
        }
        self.item_anim(it, 0);
        it.b.atk = self.assets.enemy.t.u16(0x29860, it.b.var26 as u32) as u8;
        it.b.serial = self.w.attack_serial;
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
        it.b.hitstop = 0;
        it.b.mode += 2;
        true
    }

    /// 0x28DE2: dispatch on +$05.
    fn container_alive(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.sub {
            0 => self.container_idle(it, h, i),
            4 => self.item_lift_a(it, h),
            8 => self.container_held(it, h),
            0xC => self.container_dropped(it, h),
            0x10 => self.container_thrown(it, h, i),
            _ => {}
        }
    }

    /// 0x27DD8 + the free + the drop of an exploding barrel (0x28E84): explosion kind 0, drop
    /// roll, free.
    fn container_explode(&mut self, it: &mut Item, h: Handle, i: usize) {
        self.item_explosion(it, h, 0);
        self.item_drop_from_container(it);
        self.item_free(it, i);
    }

    /// 0x28E00 idle: HP 0 breaks it (an explosive barrel explodes), hits shake it, an explosive
    /// barrel runs its fuse, then it registers in the lists.
    fn container_idle(&mut self, it: &mut Item, h: Handle, i: usize) {
        let explosive = it.m.w(W5C) != 0;
        if it.b.hp == 0 {
            if explosive {
                return self.container_explode(it, h, i);
            }
            it.set_mode_sub(4, 0); // move.w #$400,$4(a6): mode and sub only
            return self.item_draw_check(it, h);
        }
        it.b.facing = 0;
        let hs = it.b.hitstop;
        if hs != 0 {
            it.b.live = 2;
            let d = hs - 1;
            it.b.hitstop = d;
            if d == 0 {
                it.b.live = 1;
            }
            let d1: i16 = if d & 1 != 0 { -1 } else { 1 };
            let home = it.m.w(B2) as i16;
            put_hi(&mut it.b.x, home.wrapping_add(d1));
        }
        if explosive && self.container_fuse(it, h) {
            return self.container_explode(it, h, i);
        }
        // 0x28F0C: lists by variant.
        match it.b.var26 {
            0 | 2 | 4 | 0xC => {
                self.item_register_hurtable(h, it.b.live);
                self.item_register_attacker(h);
                self.container_register_obstacle(it, h);
                self.item_register_pickup(h, it.b.live);
            }
            6 | 8 => {
                self.item_register_hurtable(h, it.b.live);
                self.item_register_attacker(h);
                self.item_register_pickup(h, it.b.live);
            }
            0xA => {
                self.item_register_hurtable(h, it.b.live);
                self.item_register_attacker(h);
                self.container_register_obstacle(it, h);
            }
            _ => {}
        }
        self.item_despawn_far(it);
        self.item_draw_check(it, h);
    }

    /// 0x4A96 for a container that is out of its slot: +$AC = 0 and the obstacle list.
    fn container_register_obstacle(&mut self, it: &mut Item, h: Handle) {
        it.m.sb(AC, 0);
        self.item_register_obstacle(h);
    }

    /// 0x28E5A..0x28F06: the fuse of an explosive barrel. Returns true when it explodes now.
    /// +$B8 counts the frames of the current blink, +$B7 the blinks left in the stage, +$B6 the
    /// stage (3 = just lit .. 0); +$80 is the idle shimmer. A hit (HP changed) or something
    /// standing on the barrel (+$AC, set by the obstacle test) lights the fuse.
    pub(super) fn container_fuse(&mut self, it: &mut Item, h: Handle) -> bool {
        let t = &self.assets.enemy.t;
        it.b.palette_override = 0;
        let b8 = it.m.b(B8);
        if b8 != 0 {
            let v = b8 - 1;
            it.m.sb(B8, v);
            if v == 0 {
                it.b.palette_override = 0x84;
                let (b7, borrow) = it.m.b(B7).overflowing_sub(1);
                it.m.sb(B7, b7);
                if borrow {
                    let (b6, borrow2) = it.m.b(B6).overflowing_sub(1);
                    it.m.sb(B6, b6);
                    if borrow2 {
                        return true;
                    }
                    it.m.sb(B7, t.u8(FUSE_COUNT, b6 as u32));
                }
                it.m.sb(B8, t.u8(FUSE_PERIOD, it.m.b(B6) as u32));
            }
        } else {
            let (v, borrow) = it.m.b(B80).overflowing_sub(1);
            it.m.sb(B80, v);
            if borrow {
                it.m.sb(B80, 0x32);
                it.b.palette_override = 0x84;
            }
        }
        // 0x28ECE: a hit or a climber lights the fuse (once).
        if it.m.b(AC) == 0 {
            if it.b.hp == it.b.hp_prev {
                return false;
            }
            it.b.hp_prev = it.b.hp;
        }
        if it.m.b(B8) == 0 {
            it.m.sb(B6, 3);
            it.m.sb(B7, t.u8(FUSE_COUNT, 3));
            it.m.sb(B8, t.u8(FUSE_PERIOD, 3));
            self.item_spawn_fuse_spark(it, h);
        }
        false
    }

    /// 0x9625C: the fuse spark (effect class 0x19) on the barrel: X + 10, Y + 0x39, +$60 = barrel.
    fn item_spawn_fuse_spark(&mut self, it: &Item, h: Handle) {
        let pos = (it.b.xi().wrapping_add(10), it.b.yi().wrapping_add(0x39), it.b.zi());
        if let Some(j) = self.item_spawn_effect(0x64, pos) {
            self.effects.slots[j].owner60 = h;
        }
    }

    // ------------------------------------------------------------------ held, dropped

    /// 0x29046: the holder is gone or holds something else: the container is dropped (sub 0xC)
    /// and the thrower still gets the 500 points (0x29858).
    fn container_drop_state(&mut self, it: &mut Item) {
        it.b.sub = 0xC;
        it.b.act = 0;
        self.item_score_500(it.b.who);
    }

    /// 0x28F76 held (sub 8): follows the hand; when the player's attack frame arrives (+$C4 set
    /// and the frame byte +$2C != 0) it is thrown with the velocities of the table 0x2986E by
    /// character and variant.
    fn container_held(&mut self, it: &mut Item, h: Handle) {
        let owner = match slot_of(it.b.partner) {
            Some(Slot::Player(p)) => {
                let pl = &self.players[p];
                (pl.b.live != 0 && pl.item == h && pl.item_type == it.m.w(KIND)).then_some(p)
            }
            _ => None,
        };
        let Some(p) = owner else {
            return self.container_drop_state(it);
        };
        if self.item_follow_hand(it, p) {
            it.b.visible = 0;
            return;
        }
        let (atk, frame_byte, face, k20) = {
            let pl = &self.players[p];
            (pl.weapon_atk, pl.b.param_a, pl.b.facing, pl.b.k20)
        };
        if atk == 0 || frame_byte == 0 {
            return self.item_draw_check(it, h);
        }
        self.players[p].item_type = 0;
        let t = &self.assets.enemy.t;
        let base = t.u32(0x2986E, k20 as u32);
        let off = (it.b.var26 as u32) * 4;
        let (mut vx, mut ax) = (t.i16(base, off), t.i16(base, off + 2));
        if face == 0 {
            vx = vx.wrapping_neg();
            ax = ax.wrapping_neg();
        }
        it.b.vx = vx;
        it.b.ax = ax;
        it.b.vy = t.i16(base, off + 4);
        it.b.ay = t.i16(base, off + 6);
        it.b.vz = 0;
        it.b.az = 0;
        it.b.airborne = 1;
        it.b.atk = t.u16(0x29860, it.b.var26 as u32) as u8;
        it.b.serial = self.w.attack_serial;
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
        it.b.hitstop = 0;
        it.m.sb(B0, 2);
        it.b.draw_flags = 0;
        self.item_anim(it, 2);
        it.b.sub = 0x10;
        self.item_score_500(it.b.who);
        self.item_draw_check(it, h);
    }

    /// 0x29056 dropped (sub 0xC): hops up and falls as a thrown container (sub 0x10, +$B0 = 2).
    fn container_dropped(&mut self, it: &mut Item, h: Handle) {
        it.b.vy = 0x300;
        it.b.ay = -0x30;
        it.b.vx = 0;
        it.b.ax = 0;
        it.b.vz = 0;
        it.b.az = 0;
        it.b.airborne = 1;
        it.m.sb(B0, 2);
        it.b.sub = 0x10;
        if it.m.w(W5C) != 0 {
            it.b.who = 0xFF;
        }
        self.item_draw_check(it, h);
    }

    // ------------------------------------------------------------------ thrown

    /// 0x2909A thrown (sub 0x10) by variant. Variant 0xC jumps into the treasure's flight code
    /// (0x2A0D4): a thrown 0xC container just hangs in the air; kept as the 68000 does it.
    fn container_thrown(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.var26 {
            0 => self.container_thrown_0(it, h, i),
            2 => self.container_thrown_2(it, h, i),
            4 => self.container_thrown_4(it, h, i),
            6 => self.container_thrown_debris(it, h, DEBRIS_B),
            8 | 0xA => self.container_thrown_debris(it, h, DEBRIS_C),
            0xC => {
                // 0x2A0D4
                if it.b.airborne == 0 && self.item_land_check(it) {
                    it.set_mode_sub(2, 0);
                    it.clear_act();
                }
                self.item_draw_check(it, h);
            }
            _ => {}
        }
    }

    /// 0x290B2 (variant 0): bounces off walls and the floor and breaks on the first contact; the
    /// pieces (anim 3) blink for 60 frames. A hit stop that runs out also breaks it.
    fn container_thrown_0(&mut self, it: &mut Item, h: Handle, i: usize) {
        let mut brk = false;
        let hs = it.b.hitstop;
        if hs != 0 {
            it.b.hitstop = hs - 1;
            brk = hs - 1 == 0;
        }
        if !brk {
            let mut terrain = true;
            let b9 = it.m.b(B9);
            if b9 != 0 {
                it.m.sb(B9, b9 - 1);
                terrain = b9 - 1 == 0;
            }
            if terrain {
                self.item_terrain(it, h);
            }
            if it.b.wall != 0 {
                it.b.vx = it.b.vx.wrapping_neg();
                it.b.facing ^= 1;
                brk = true;
            } else if it.b.airborne == 0 {
                self.cam.shake_req = 2;
                it.b.airborne = 1;
                it.b.vx >>= 1;
                it.b.vy = it.b.vy.wrapping_neg();
                it.m.sb(B9, 5);
                it.b.wall = 0;
                self.item_snd_9fb6(it.b.xi());
                brk = true;
            }
        }
        if brk && it.m.b(B0) != 0 {
            // 0x29130
            it.b.live = 2;
            it.m.sb(B0, 0);
            it.m.sb(B81, 1);
            it.m.sb(B80, 1);
            it.m.sb(B1, 0x3C);
            self.item_drop_from_container(it);
            self.item_anim(it, 3);
            if it.m.w(W5C) != 0 {
                self.item_explosion(it, h, 0);
                return self.item_free(it, i);
            }
        }
        // 0x29170
        it.b.motion();
        if it.m.b(B0) == 0 {
            let b1 = it.m.b(B1).wrapping_sub(1);
            it.m.sb(B1, b1);
            if b1 == 0 {
                it.set_mode_sub(6, 0);
                return;
            }
            let b80 = it.m.b(B80).wrapping_sub(1);
            it.m.sb(B80, b80);
            if b80 != 0 {
                return;
            }
            let b81 = it.m.b(B81).wrapping_add(1);
            it.m.sb(B81, b81);
            it.m.sb(B80, b81);
        }
        self.item_despawn_far(it);
        self.item_register_attacker(h);
        self.item_draw_check(it, h);
    }

    /// 0x291B4 (variant 2): a wall bounce flips it; two floor landings (+$B0) break it into two
    /// debris pieces. The drop roll happens at the first landing. An explosive barrel explodes on
    /// the first contact.
    fn container_thrown_2(&mut self, it: &mut Item, h: Handle, i: usize) {
        let explosive = it.m.w(W5C) != 0;
        let hs = it.b.hitstop;
        if hs != 0 {
            let d = hs - 1;
            it.b.hitstop = d;
            if d != 0 {
                return self.item_draw_check(it, h);
            }
            if explosive {
                return self.container_explode(it, h, i);
            }
        }
        let mut terrain = true;
        let b9 = it.m.b(B9);
        if b9 != 0 {
            it.m.sb(B9, b9 - 1);
            terrain = b9 - 1 == 0;
        }
        if terrain {
            self.item_terrain(it, h);
        }
        if it.b.wall != 0 {
            it.b.vx = it.b.vx.wrapping_neg();
            it.b.facing ^= 1;
            self.item_snd_9fb6(it.b.xi());
            it.m.sb(B9, 5);
            it.b.wall = 0;
            if explosive {
                return self.container_explode(it, h, i);
            }
        }
        if it.b.airborne == 0 {
            self.item_snd_9fb6(it.b.xi());
            self.cam.shake_req = 2;
            if explosive {
                return self.container_explode(it, h, i);
            }
            self.item_drop_from_container(it);
            let b0 = it.m.b(B0).wrapping_sub(1);
            it.m.sb(B0, b0);
            if b0 == 0 {
                it.set_mode_sub(6, 0);
                return self.container_two_pieces(it);
            }
            it.b.airborne = 1;
            it.b.vx >>= 1;
            it.b.vy = it.b.vy.wrapping_neg() >> 1;
        }
        // 0x29288
        self.item_anim_step(it);
        it.b.motion();
        self.item_despawn_far(it);
        self.item_register_attacker(h);
        self.item_draw_check(it, h);
    }

    /// 0x292A6: two debris pieces (class 0x2C, pictures 2 and 4) fly out of the broken container,
    /// 0x60 and 0x10 above it, with half its horizontal speed.
    fn container_two_pieces(&mut self, it: &Item) {
        self.item_two_pieces(&it.b);
    }

    /// 0x29348 (variant 4): breaks on the first wall or floor contact into a puff (0x2943C).
    fn container_thrown_4(&mut self, it: &mut Item, h: Handle, _i: usize) {
        self.item_terrain(it, h);
        if it.b.wall != 0 || it.b.airborne == 0 {
            self.item_snd_9e60(it.b.xi());
            self.cam.shake_req = 2;
            if it.m.w(W5C) != 0 {
                self.item_explosion(it, h, 0);
            }
            self.container_puff(it, 0x1A, it.b.facing);
            it.set_mode_sub(6, 0);
            return self.item_drop_from_container(it);
        }
        self.item_anim_step(it);
        it.b.motion();
        self.item_despawn_far(it);
        self.item_register_attacker(h);
        self.item_draw_check(it, h);
    }

    /// 0x293AC / 0x293F4 (variants 6, 8, 0xA): breaks on the first contact into debris.
    fn container_thrown_debris(&mut self, it: &mut Item, h: Handle, table: u32) {
        self.item_terrain(it, h);
        if it.b.wall != 0 || it.b.airborne == 0 {
            self.cam.shake_req = 2;
            self.container_debris(&it.b, table);
            it.set_mode_sub(6, 0);
            return self.item_drop_from_container(it);
        }
        it.b.motion();
        self.item_despawn_far(it);
        self.item_register_attacker(h);
        self.item_draw_check(it, h);
    }

    /// 0x2943C (a1 = a debris list that this entry does not use): effect class 0x54 with
    /// variant `var` and facing `facing` at the container (0x94640).
    fn container_puff(&mut self, it: &Item, var: u16, facing: u8) {
        let pos = (it.b.xi(), it.b.yi(), it.b.zi());
        if let Some(j) = self.item_spawn_effect(0x54, pos) {
            let e = &mut self.effects.slots[j];
            e.b.var26 = var;
            e.b.facing = facing;
        }
    }

    /// 0x294AA: sound 0x74 / 0xFA and up to 16 debris pieces (effect class 0x2C) from the list
    /// at `table` (5 words per piece: dx, dy, vx, vy, picture; a negative first word ends it).
    /// A piece may carry a bonus (+$5C = 1): the chance depends on the pieces that already do
    /// (masks 0x29552, G.$4F1 = the number of explosive weapons on the field). The `rand()` call
    /// happens for every piece while that number is below 3.
    pub(super) fn container_debris(&mut self, b: &Body, table: u32) {
        self.item_snd_9ff8(b.xi());
        let mut d5 = self.spawn.g.b4f1 as u32;
        let mut a = 0u32;
        for _ in 0..16 {
            let t = &self.assets.enemy.t;
            let (dx, dy, vx, vy, var) = (t.i16(table, a), t.i16(table, a + 2), t.i16(table, a + 4), t.i16(table, a + 6), t.u16(table, a + 8));
            let next = t.i16(table, a + 10);
            a += 10;
            let pos = (b.xi().wrapping_add(dx), b.yi().wrapping_add(dy), b.zi());
            let Some(j) = self.item_spawn_effect(0x2C, pos) else { break };
            self.effects.slots[j].b.facing = b.facing;
            self.effects.slots[j].b.vx = if b.facing != 0 { vx } else { vx.wrapping_neg() };
            self.effects.slots[j].b.vy = vy;
            self.effects.slots[j].b.ay = -0x14;
            self.effects.slots[j].b.var26 = var;
            self.effects.slots[j].w5c = 0;
            if d5 < 3 {
                let mask = self.assets.enemy.t.u32(DEBRIS_MASKS, d5 * 4);
                let r = self.rand() & 0xF;
                if mask >> r & 1 != 0 {
                    d5 += 1;
                    self.effects.slots[j].w5c = 1;
                    if b.var26 == 6 {
                        self.effects.slots[j].b.var26 = 0x1A;
                    }
                }
            }
            if next < 0 {
                break;
            }
        }
    }

    // ------------------------------------------------------------------ break (mode 4)

    /// 0x296AC break by variant: +500 points to +$02, then drop roll, debris, sounds, free.
    fn container_break(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.var26 {
            0 => self.container_break_0(it, h, i),
            2 => {
                self.item_score_500(it.b.who);
                self.item_snd_9fb6(it.b.xi());
                let vx = if it.b.facing != 0 { -0x380 } else { 0x380 };
                it.b.vx = vx;
                self.container_two_pieces(it);
                self.item_drop_from_container(it);
                if it.m.w(W5C) != 0 {
                    self.item_explosion(it, h, 0);
                }
                self.item_free(it, i);
            }
            4 => {
                self.item_score_500(it.b.who);
                self.item_snd_9e60(it.b.xi());
                self.container_puff(it, 0x1A, it.b.facing);
                self.item_drop_from_container(it);
                if it.m.w(W5C) != 0 {
                    self.item_explosion(it, h, 0);
                }
                self.item_free(it, i);
            }
            6 | 8 => {
                self.item_score_500(it.b.who);
                self.container_debris(&it.b, if it.b.var26 == 6 { DEBRIS_B } else { DEBRIS_C });
                self.item_drop_from_container(it);
                self.item_free(it, i);
            }
            0xA => {
                self.item_score_500(it.b.who);
                self.item_snd_a396(it.b.xi()); // 0xA396
                self.container_puff(it, 0, it.b.facing ^ 1);
                self.item_drop_from_container(it);
                self.item_free(it, i);
            }
            0xC => {
                self.item_score_500(it.b.who);
                self.item_snd_a404(it.b.xi());
                self.container_puff(it, 0x22, it.b.facing ^ 1);
                self.item_drop_from_container(it);
                self.item_free(it, i);
            }
            _ => {}
        }
    }

    /// 0x296C6 (variant 0): on the first frame it scores, rolls the drop and flies off to the
    /// side (vy 0x400, ay -0x20, vx +-0x300) as pieces for 40 frames, blinking.
    fn container_break_0(&mut self, it: &mut Item, h: Handle, i: usize) {
        if it.b.sub == 0 {
            self.item_score_500(it.b.who);
            self.item_drop_from_container(it);
            it.b.vx = 0;
            it.b.vy = 0x400;
            it.b.ay = -0x20;
            it.b.vx = if it.b.facing != 0 { -0x300 } else { 0x300 };
            it.b.airborne = 1;
            it.m.sb(B1, 0x28);
            self.item_snd_9fb6(it.b.xi());
            if it.m.w(W5C) != 0 {
                self.item_explosion(it, h, 0);
                self.item_drop_from_container(it);
                return self.item_free(it, i);
            }
            self.item_anim(it, 3);
            it.b.sub += 2;
        }
        // 0x29732
        let b1 = it.m.b(B1).wrapping_sub(1);
        it.m.sb(B1, b1);
        if b1 == 0 {
            return self.item_free(it, i);
        }
        self.item_terrain(it, h);
        if it.b.airborne == 0 {
            it.b.airborne = 1;
            it.b.vy = it.b.vy.wrapping_neg() >> 1;
            self.item_snd_9fb6(it.b.xi());
        }
        it.b.motion();
        if it.m.b(B1) & 1 != 0 {
            self.item_draw_check(it, h);
        }
    }
}

#[allow(unused)]
fn _effect_type(_: &Effect) {}
