//! Explosions and the projectile classes of pool E: 1 explosion (0x27E16), 3 bullet of a held gun
//! (0x2860A), 6 torch overlay of weapon type 11 (0x29B06, not ported), 7 rocket shell (0x29D1E),
//! 8 ballistic projectile (0x29EC4), 10 shotgun pellet (0x2A39A), 11 invisible attack box
//! (0x2A534).
//!
//! The explosions and projectiles are attackers: they join the list `$671A` (4BDA) and carry
//! their own attack id (+$58), so the passes in `hits.rs` and `enemy_combat.rs` hurt enemies,
//! containers and (later) players with the player attack records.

use super::*;

const EXPLOSION_TABLE: u32 = 0x28102;
const BULLET_TABLE: u32 = 0x28904;
const SHELL_TABLE: u32 = 0x29EB4;
const PROJECTILE_TABLE: u32 = 0x2A00A;
const PELLET_TABLE: u32 = 0x2A49C;
/// Bullet records (8 bytes: vx, animation, attack id, aim) by variant * 4; aim tables.
const BULLET_RECORDS: u32 = 0x28814;
const BULLET_PTRS: u32 = 0x2884C;
const BULLET_VY: u32 = 0x2885C;
const AIM_A: u32 = 0x288B4;
const AIM_B: u32 = 0x288C4;
const AIM_VZ: u32 = 0x288D4;
/// Pellet records (vx, vy) by variant.
const PELLET_RECORDS: u32 = 0x2A484;

impl Game {
    /// 0x27DD8: an explosion (class 1, kind `kind` in +$26) at the source: copies +$02 / +$03,
    /// X / Y / Z, the airborne flag, and +$70 = the source. A full pool loses it.
    pub fn item_explosion_body(&mut self, b: &Body, h: Handle, kind: u16) {
        let pos = (b.xi(), b.yi(), b.zi());
        let Some(j) = self.items.alloc() else { return };
        let n = &mut self.items.slots[j];
        n.b.live = 1;
        n.b.who = b.who;
        n.b.id = b.id;
        n.b.k20 = class::BLAST * 4;
        put_hi(&mut n.b.x, pos.0);
        put_hi(&mut n.b.y, pos.1);
        put_hi(&mut n.b.z, pos.2);
        n.b.var26 = kind;
        n.b.airborne = b.airborne;
        n.b.partner = h;
    }

    pub(super) fn item_explosion(&mut self, src: &Item, h: Handle, kind: u16) {
        self.item_explosion_body(&src.b, h, kind);
    }

    /// 0x9F12: the explosion sound.
    fn item_snd_blast(&mut self) {
        self.sound(0x66);
    }

    /// `d7` of the 68000 pool loop: 23 - slot (the counter `dbra` runs down).
    fn item_d7(i: usize) -> u16 {
        23 - i as u16
    }

    /// The first frame of a blast: +$A0 = 0, +$25 = 0x30, the table, no box table of its own.
    fn blast_setup(it: &mut Item) {
        it.m.sw(KIND, 0);
        it.b.draw_flags = 0x30;
        it.b.anim_table = EXPLOSION_TABLE;
    }

    /// 0x27E16 explosion, by kind (+$26): 0 big (0x27E34), 1 rolling (0x27EDE), 2 (0x27F70), 3
    /// (0x27FEE, belongs to nobody), 4+ small flames (0x28098). Modes 0 init, 2 run, 4 / 6 free.
    pub(super) fn item_blast(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.var26 {
            0 => self.blast_kind0(it, h, i),
            1 => self.blast_kind1(it, h, i),
            2 => self.blast_kind2(it, h, i),
            3 => self.blast_kind3(it, h, i),
            _ => self.blast_small(it, h, i),
        }
    }

    /// The run part of the big blasts: tick the animation (free when it ended), join the attacker
    /// list and draw when on screen (0x27EC2).
    fn blast_run(&mut self, it: &mut Item, h: Handle, i: usize) {
        if self.item_anim_step(it) {
            return self.item_free(it, i);
        }
        self.item_register_attacker(h);
        self.draw_list_add_visible_item(it, h);
    }

    /// 0x49C0 for an item taken out of its slot: draw when the on-screen flag is set.
    fn draw_list_add_visible_item(&mut self, it: &Item, h: Handle) {
        if it.b.visible != 0 {
            self.draw_list.push(((it.b.zi() as u16 as u32) << 16) | h as u32);
        }
    }

    /// Kind 0 (0x27E34): on the floor a big explosion that shakes the screen (shake 3, attack
    /// 0x56), in the air a smaller one (attack 0x54).
    fn blast_kind0(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.mode {
            0 => {
                it.b.visible = 1; // 0x27E48: move.b #1,$1(a6)
                it.b.box_table = 0x0010_0000;
                it.b.multi_box = false;
                Self::blast_setup(it);
                let (anim, atk) = if it.b.airborne != 0 {
                    (1, 0x54)
                } else {
                    self.cam.shake_req = 3;
                    (0, 0x56)
                };
                it.b.atk = atk;
                it.b.knock = 0;
                it.b.serial = self.w.attack_serial;
                self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
                self.item_anim(it, anim);
                self.item_snd_blast();
                let pos = (it.b.xi(), it.b.yi().wrapping_add(0x20), it.b.zi().wrapping_sub(1));
                self.item_in_slot(it, h, |g| g.fx_spawn_spark_at(h, pos.0, pos.1, pos.2, 4));
                it.b.mode += 2;
                it.b.hitstop = 0;
                self.blast_run(it, h, i);
            }
            2 => {
                it.b.hitstop = 0;
                self.blast_run(it, h, i);
            }
            _ => self.item_free(it, i),
        }
    }

    /// Kind 1 (0x27EDE): a rolling explosion (attack 0x54): it moves (floor test while sub 0, the
    /// first wall stops the motion).
    fn blast_kind1(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.mode {
            0 => {
                it.b.visible = 1;
                it.b.box_table = 0x0010_0000;
                it.b.multi_box = false;
                self.blast_attack(it, 0x54);
                Self::blast_setup(it);
                self.item_anim(it, 1);
                it.b.mode += 2;
                self.blast_kind1_run(it, h, i);
            }
            2 => self.blast_kind1_run(it, h, i),
            _ => self.item_free(it, i),
        }
    }

    fn blast_kind1_run(&mut self, it: &mut Item, h: Handle, i: usize) {
        if it.b.sub == 0 {
            self.item_terrain(it, h);
            if it.b.wall != 0 {
                it.b.sub += 2;
            }
            it.b.motion();
        }
        self.blast_run(it, h, i);
    }

    /// `+$58 = atk; +$5A = 0; +$59 = 0; +$5E = G.$67BE++`.
    fn blast_attack(&mut self, it: &mut Item, atk: u8) {
        it.b.atk = atk;
        it.b.knock = 0;
        it.b.hitstop = 0;
        it.b.serial = self.w.attack_serial;
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
    }

    /// Kind 2 (0x27F70): a blast (attack 0x6E) with the sound and a screen shake.
    fn blast_kind2(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.mode {
            0 => {
                it.b.visible = 1;
                it.b.box_table = 0x0010_0000;
                it.b.multi_box = false;
                self.blast_attack(it, 0x6E);
                Self::blast_setup(it);
                self.item_anim(it, 2);
                self.item_snd_blast();
                self.cam.shake_req = 3;
                it.b.mode += 2;
                self.blast_run(it, h, i);
            }
            2 => self.blast_run(it, h, i),
            _ => self.item_free(it, i),
        }
    }

    /// Kind 3 (0x27FEE): the explosion of a bomb thrown by an enemy (+$02 = 0xFF: nobody is
    /// credited), attack 0x6A, with the spark. The init frame does not animate.
    fn blast_kind3(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.mode {
            0 => {
                it.b.visible = 1;
                it.b.who = 0xFF;
                it.b.box_table = 0x0010_0000;
                it.b.multi_box = false;
                Self::blast_setup(it);
                let anim = if it.b.airborne != 0 {
                    4
                } else {
                    self.cam.shake_req = 3;
                    3
                };
                self.item_anim(it, anim);
                self.blast_attack(it, 0x6A);
                self.item_snd_blast();
                let pos = (it.b.xi(), it.b.yi().wrapping_add(0x20), it.b.zi().wrapping_sub(1));
                self.item_in_slot(it, h, |g| g.fx_spawn_spark_at(h, pos.0, pos.1, pos.2, 4));
                it.b.mode += 2;
            }
            2 => self.blast_run(it, h, i),
            _ => self.item_free(it, i),
        }
    }

    /// Kinds 4 and up (0x28098): flames with no attack of their own (anim 2, or 1 when +$5C is
    /// set); they blink (odd frames of `frame + d7`).
    fn blast_small(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.mode {
            0 => {
                it.b.visible = 1;
                Self::blast_setup(it);
                let anim = if it.m.w(o::W5C) != 0 { 1 } else { 2 };
                self.item_anim(it, anim);
                self.item_snd_blast();
                it.b.mode += 2;
            }
            2 => {
                if self.item_anim_step(it) {
                    return self.item_free(it, i);
                }
                if (self.w.frame.wrapping_add(Self::item_d7(i))) & 1 != 0 {
                    self.draw_list_add_visible_item(it, h);
                }
            }
            _ => self.item_free(it, i),
        }
    }

    // ------------------------------------------------------------------ class 3: bullet

    /// 0x2860A bullet of a held gun (variants 0..0xA by the gun: records at 0x28814: speed,
    /// animation, attack id, aim). Aimed bullets (the guns of players) steer at the target the
    /// player has locked (+$EA), the others pick a vertical speed at random. It hits the floor, a
    /// wall or the water and turns into a hit spark animation (mode 4).
    pub(super) fn item_bullet(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.mode {
            0 => {
                self.bullet_init(it, h, i);
                self.bullet_run(it, h, i);
            }
            2 => self.bullet_run(it, h, i),
            4 => {
                if self.item_anim_step(it) {
                    return self.item_free(it, i);
                }
                self.item_draw_check(it, h);
            }
            _ => self.item_free(it, i),
        }
    }

    fn bullet_init(&mut self, it: &mut Item, h: Handle, _i: usize) {
        it.b.draw_flags = 0x10;
        it.m.sw(KIND, 0);
        it.b.airborne = 1;
        it.b.hitstop = 0;
        it.b.box_table = 0xC010_0000;
        it.b.multi_box = false;
        let var = it.b.var26 as u32;
        let rec = BULLET_RECORDS + var * 4;
        let t = &self.assets.enemy.t;
        let (vx, anim, atk, aim) = (t.i16(rec, 0), t.u16(rec, 2), t.u16(rec, 4), t.i16(rec, 6));
        it.b.vx = if it.b.facing != 0 { vx } else { vx.wrapping_neg() };
        it.b.anim_table = BULLET_TABLE;
        self.item_anim(it, anim);
        it.b.atk = atk as u8;
        it.b.knock = 0;
        it.b.serial = self.w.attack_serial;
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
        it.b.mode += 2;
        if aim >= 0 {
            // 0x28746: a fixed pick: pointer table by the shooter's character, row +aim, a random
            // entry (rand() & 0xF) gives the vertical speed.
            let t = &self.assets.enemy.t;
            let row = t.u32(BULLET_PTRS, (it.b.id as u32) * 4).wrapping_add(aim as u16 as u32);
            let r = (self.rand() & 0xF) as u32;
            let pick = self.assets.enemy.t.u8(row, r) as u32;
            it.b.vy = self.assets.enemy.t.i16(BULLET_VY, pick);
            return;
        }
        // 0x2868A: aimed.
        let target = match slot_of(it.b.partner) {
            Some(Slot::Player(p)) => {
                let a = self.players[p].aim;
                if a != 0 { a } else { h }
            }
            _ => h,
        };
        let tx = (if target == h { it.b.xi() } else { self.body(target).xi() }) as u16;
        let x = it.b.xi() as u16;
        let mut d0 = tx.wrapping_sub(x);
        if tx < x {
            d0 = d0.wrapping_neg();
        }
        let (d, borrow) = d0.overflowing_sub(0xA0);
        d0 = if borrow { 0 } else { d };
        if d0 > 0xE0 {
            d0 = 0xE0;
        }
        let table = if var == 4 || var == 6 { AIM_A } else { AIM_B };
        let t = &self.assets.enemy.t;
        let mut vy = t.i16(table, ((d0 & 0xE0) >> 4) as u32);
        let a9 = it.m.b(A9) as u16;
        let d1 = a9.min(0x18).wrapping_mul(0x30);
        vy = vy.wrapping_add(d1 as i16);
        it.b.vy = if vy < 0 { vy } else { 0 };
        let mut vz: i16 = 0;
        if a9 & 1 != 0 {
            let mut k = (a9 & 0xFC) >> 1;
            if k >= 8 {
                k = 8;
            }
            vz = t.i16(AIM_VZ, k as u32);
            if a9 & 3 != 3 {
                vz = vz.wrapping_neg();
            }
        }
        it.b.vz = vz;
    }

    /// 0x28774: tick, move, collide on alternate frames (the slot parity), hit spark on contact.
    fn bullet_run(&mut self, it: &mut Item, h: Handle, i: usize) {
        self.item_anim_step(it);
        it.b.motion();
        if it.b.hitstop != 0 {
            return self.item_free(it, i);
        }
        if (self.w.frame.wrapping_add(Self::item_d7(i))) & 1 != 0 {
            self.item_terrain(it, h);
            let hit = if it.b.airborne == 0 {
                Some(1)
            } else if it.b.wall != 0 {
                if it.b.wall & 1 == 0 {
                    Some(2)
                } else {
                    return self.item_free(it, i);
                }
            } else if it.b.wading == 1 && (it.b.yi() as u16) < 0x20 {
                Some(3)
            } else {
                None
            };
            if let Some(anim) = hit {
                it.b.draw_flags = 0x10;
                it.set_mode_sub(4, 0);
                self.item_anim(it, anim);
                if self.item_anim_step(it) {
                    return self.item_free(it, i);
                }
                return self.item_draw_check(it, h);
            }
        }
        self.item_register_attacker(h);
        self.item_despawn_near(it);
        self.item_draw_check(it, h);
    }

    // ------------------------------------------------------------------ class 6: overlay

    /// 0x29B06 the glow of a torch (the overlay of weapon type 11, made by `weapon_spawn_overlay`: +$58 the kind word of the torch, +$5A the torch). It draws the flame
    /// (effect frame lists 0x29CB6 idle, 0x29CF4 swing, 0x160A / 0x15F4) at the torch: X and Y by the frame byte of the torch (+$2D, the table 0x29C26: a long
    /// `dx, dy`, dx negated when the torch faces left), Z + 2 (Z - 1 in the frames with bit 0 of G.$1 set: the flame flickers behind and in front of the head), and its
    /// visible byte. A torch that is held by a hero in the swing act (+$06 of the holder is 2) shows the swing flame from the frame the act began (the list is
    /// loaded when the frame flag +$2C is 0); in every other case the idle flame comes back when the swing frame's flag +$2C is not 0. It ends when the torch is gone
    /// or no longer the same weapon (class word +$20, kind +$A0).
    pub(super) fn item_overlay(&mut self, it: &mut Item, h: Handle, i: usize) {
        let fx = &self.assets.effects;
        match it.b.mode {
            0 => {
                // 0x29B1A
                crate::game::effect::fx_load(&mut it.b, fx, fx.frame_at(0x29CB6));
                it.b.draw_flags = 0x30;
                it.m.sb(0x5C, 0);
                it.m.sb(0x5D, 0);
                it.b.mode += 2;
            }
            2 => {}
            _ => return self.item_free(it, i), // 0x29CAA
        }
        // 0x29B38
        let owner = it.m.w(0x5A);
        let Some(Slot::Item(j)) = slot_of(owner) else { return self.item_free(it, i) };
        let o = &self.items.slots[j];
        if o.b.live == 0 || o.b.k20 != 0 || it.m.w(0x58) != o.m.w(KIND) {
            return self.item_free(it, i);
        }
        it.b.visible = o.b.visible;
        let t = &self.assets.enemy.t;
        let (dx, dy) = (t.i16(0x29C26, o.b.param_b as u32), t.i16(0x29C26, o.b.param_b as u32 + 2));
        let y = o.b.yi().wrapping_add(dy);
        let dx = if o.b.facing != 0 { dx.wrapping_neg() } else { dx };
        let x = o.b.xi().wrapping_add(dx);
        let z = o.b.zi().wrapping_add(if self.w.frame & 1 != 0 { -1 } else { 2 });
        it.b.facing = o.b.facing;
        put_hi(&mut it.b.y, y);
        put_hi(&mut it.b.x, x);
        put_hi(&mut it.b.z, z);
        let flame = it.b.param_a != 0; // tst.b $2C(a6): the swing list is playing
        if o.b.sub == 8 {
            // 0x29BCE: held: the act of the holder
            let holder = o.b.partner;
            let act = self.body(holder).act;
            it.m.sb(0x5D, it.m.b(0x5C));
            it.m.sb(0x5C, act);
            if act == 2 {
                if it.m.b(0x5D) != 2 && !flame {
                    crate::game::effect::fx_load(&mut it.b, fx, fx.frame_at(0x29CF4)); // 0x29C10
                }
            } else if flame {
                crate::game::effect::fx_load(&mut it.b, fx, fx.frame_at(0x29CB6)); // 0x29BF0
            }
        } else if flame {
            crate::game::effect::fx_load(&mut it.b, fx, fx.frame_at(0x29CB6)); // 0x29BC0
        }
        crate::game::effect::fx_tick(&mut it.b, fx); // jsr 0x15F4
        self.draw_list_add_visible_item(it, h); // jmp 0x49C0
    }

    // ------------------------------------------------------------------ class 7: shell

    /// 0x29D1E: the shell / rocket body (spawned by the bazooka): it flies up and down in an arc
    /// around the height of its start (+$A0), leaves dust, and turns into a small explosion
    /// (class 1, kind 1) on contact.
    pub(super) fn item_shell(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.mode {
            0 => {
                it.b.draw_flags = 0x10;
                it.m.sw(KIND, 0);
                it.b.airborne = 1;
                it.b.hitstop = 0;
                it.m.sw(KIND, it.b.yi() as u16); // move.w $c(a6),$a0(a6): the floor for the arc
                it.b.box_table = 0xC010_0000;
                it.b.multi_box = false;
                let (vx, ax) = if it.b.facing != 0 { (0x100, 0x40) } else { (-0x100, -0x40) };
                it.b.vx = vx;
                it.b.ax = ax;
                it.b.vy = 0x100;
                it.b.ay = 0xC0;
                it.b.anim_table = SHELL_TABLE;
                self.item_anim(it, 0);
                self.blast_attack(it, 0x52);
                it.b.mode += 2;
                self.shell_run(it, h, i);
            }
            2 => self.shell_run(it, h, i),
            _ => self.item_free(it, i),
        }
    }

    fn shell_run(&mut self, it: &mut Item, h: Handle, i: usize) {
        if it.b.hitstop != 0 {
            return self.shell_end(it, h);
        }
        let floor = it.m.w(KIND) as i16;
        let y = it.b.yi();
        if it.b.ay >= 0 {
            if (floor as u16) < y as u16 {
                it.b.ay = it.b.ay.wrapping_neg();
                it.b.vy = it.b.vy.wrapping_add(0xB0);
            }
        } else if floor as u16 >= y as u16 {
            it.b.ay = it.b.ay.wrapping_neg();
            it.b.vy = it.b.vy.wrapping_sub(0xB0);
        }
        if (self.w.frame.wrapping_add(Self::item_d7(i))) & 1 != 0 {
            self.item_terrain(it, h);
            if it.b.airborne == 0 || it.b.wall != 0 || (it.b.wading == 1 && (it.b.yi() as u16) < 0x20) {
                return self.shell_end(it, h);
            }
        }
        if self.w.frame & 1 == 0 && it.b.visible != 0 {
            // 0x92E5E: the dust trail (effect class, not ported yet).
            self.item_spawn_dust(it);
        }
        it.b.motion();
        self.item_register_attacker(h);
        self.item_despawn_near(it);
        self.item_draw_check(it, h);
    }

    /// 0x92E5E with d0 = 0: a puff of dust at the object (pool C class 0x1C... of the effects port).
    fn item_spawn_dust(&mut self, it: &Item) {
        let pos = (it.b.xi(), it.b.yi(), it.b.zi());
        let _ = self.item_spawn_effect(0x1C, pos);
    }

    /// 0x29E5C: the shell is spent: it explodes (class 1, kind 1, vy 0) and goes next frame.
    fn shell_end(&mut self, it: &mut Item, _h: Handle) {
        it.set_mode_sub(6, 0);
        let pos = (it.b.xi(), it.b.yi(), it.b.zi());
        let Some(j) = self.items.alloc() else { return };
        let n = &mut self.items.slots[j];
        n.b.live = 1;
        n.b.who = it.b.who;
        n.b.id = it.b.id;
        n.b.k20 = class::BLAST * 4;
        put_hi(&mut n.b.x, pos.0);
        put_hi(&mut n.b.y, pos.1);
        put_hi(&mut n.b.z, pos.2);
        n.b.vx = it.b.vx;
        n.b.vy = 0;
        n.b.airborne = it.b.airborne;
        n.b.partner = it.b.partner;
        n.b.atk = it.b.atk;
        n.b.var26 = 1;
    }

    // ------------------------------------------------------------------ class 8: projectile

    /// 0x29EC4: a lobbed projectile (from a pool B object, +$A0 = target X, +$A2 = target height):
    /// the horizontal speed comes from a 32-bit division by 0x39 of the distance, it flies in an
    /// arc (vy -0x500 start) and explodes (class 1) when it falls below +$A2; with a non-zero
    /// variant it also leaves an attack box (class 11).
    pub(super) fn item_projectile(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.mode {
            0 => {
                it.b.draw_flags = 0x10;
                it.b.airborne = 1;
                let dx = (it.m.w(KIND).wrapping_sub(it.b.xi() as u16)) as i16;
                let d2 = dx.unsigned_abs() as u32;
                let (q, r) = (d2 / 0x39, d2 % 0x39);
                it.b.vx = r as i16;
                let hi = (it.b.vx as u16 >> 8) as u8 as u32; // add.b d0,$14(a6): the high byte
                it.b.vx = (((hi + q) & 0xFF) << 8 | (r & 0xFF)) as u16 as i16;
                it.b.vy = -0x500;
                it.b.facing = 1;
                it.b.anim_table = PROJECTILE_TABLE;
                self.item_anim(it, if it.b.vx as u16 >= 0x400 { 1 } else { 0 });
                it.b.mode += 2;
                self.projectile_run(it, h, i);
            }
            2 => self.projectile_run(it, h, i),
            _ => self.item_free(it, i),
        }
    }

    fn projectile_run(&mut self, it: &mut Item, h: Handle, i: usize) {
        let _ = i;
        let target_y = it.m.w(o::A2) as i16;
        if target_y < it.b.yi() {
            if self.w.frame & 1 == 0 && it.b.visible != 0 {
                let pos = (it.b.xi(), it.b.yi(), it.b.zi());
                let _ = self.item_spawn_effect(0x1C, pos);
            }
            it.b.motion();
            return self.item_draw_check(it, h);
        }
        if it.b.var26 != 0 {
            self.item_attack_box_spawn(it, h, 0);
            self.cam.shake_req = 3;
        }
        // 0x29FCE: explosion kind 2.
        it.set_mode_sub(6, 0);
        let pos = (it.b.xi(), it.b.yi(), it.b.zi());
        let Some(j) = self.items.alloc() else { return };
        let n = &mut self.items.slots[j];
        n.b.live = 1;
        n.b.who = it.b.who;
        n.b.id = it.b.id;
        n.b.k20 = class::BLAST * 4;
        put_hi(&mut n.b.x, pos.0);
        put_hi(&mut n.b.y, pos.1);
        put_hi(&mut n.b.z, pos.2);
        n.b.var26 = 2;
    }

    // ------------------------------------------------------------------ class 10: pellet

    /// 0x2A39A: a shotgun pellet (records 0x2A484: vx, vy by variant 0..5): flies up to 0xC0 pixels
    /// from where it started; a hit frees it.
    pub(super) fn item_gun_shot(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.mode {
            0 => {
                it.m.sw(0xB0, it.b.xi() as u16);
                it.b.draw_flags = 0x10;
                it.m.sw(KIND, 0);
                it.b.airborne = 1;
                it.b.hitstop = 0;
                it.b.box_table = 0xC010_0000;
                it.b.multi_box = false;
                let rec = PELLET_RECORDS + it.b.var26 as u32 * 4;
                let t = &self.assets.enemy.t;
                let (vx, vy) = (t.i16(rec, 0), t.i16(rec, 2));
                it.b.vx = if it.b.facing != 0 { vx } else { vx.wrapping_neg() };
                it.b.vy = vy;
                it.b.anim_table = PELLET_TABLE;
                let d7 = Self::item_d7(i);
                self.item_anim(it, d7 & 3);
                self.blast_attack(it, 0x5E);
                it.m.sb(B80, ((d7 & 0xF) + 8) as u8);
                it.b.mode += 2;
                self.pellet_run(it, h, i);
            }
            2 => self.pellet_run(it, h, i),
            _ => self.item_free(it, i),
        }
    }

    fn pellet_run(&mut self, it: &mut Item, h: Handle, i: usize) {
        let start = it.m.w(0xB0) as i16;
        let far = it.b.xi().wrapping_sub(start).wrapping_add(0xC0) as u16 > 0x180;
        if it.b.hitstop != 0 || far {
            return self.item_free(it, i);
        }
        // 0x2A450: +$80 is never counted down, so the animation 4 switch is dead code (== 1).
        if it.m.b(B80).wrapping_sub(1) == 0 {
            self.item_anim(it, 4);
        }
        self.item_anim_step(it);
        it.b.motion();
        self.item_register_attacker(h);
        self.item_draw_check(it, h);
    }

    // ------------------------------------------------------------------ class 11: attack box

    /// 0x2A4FC: spawn an invisible attack box (class 11, variant `var`) at the source.
    pub(super) fn item_attack_box_spawn(&mut self, it: &Item, h: Handle, var: u16) {
        let pos = (it.b.xi(), it.b.yi(), it.b.zi());
        let Some(j) = self.items.alloc() else { return };
        let n = &mut self.items.slots[j];
        n.b.live = 1;
        n.b.k20 = class::ATTACK_BOX * 4;
        n.b.who = it.b.who;
        n.b.id = it.b.id;
        put_hi(&mut n.b.x, pos.0);
        put_hi(&mut n.b.y, pos.1);
        put_hi(&mut n.b.z, pos.2);
        n.b.var26 = var;
        n.b.partner = h;
    }

    /// 0x2A534: an invisible attack box that lives for one frame: attack frame class 4, attack
    /// box 0xC9 (attack 0x6E, variant 0) or 0xCA (0x6C).
    pub(super) fn item_attack_box(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.mode {
            0 => {
                it.b.box_table = 0xC010_0000;
                it.b.multi_box = false;
                it.b.kind = 4;
                if it.b.var26 == 0 {
                    it.b.attack_box = 0xC9;
                    self.blast_attack(it, 0x6E);
                } else {
                    it.b.attack_box = 0xCA;
                    self.blast_attack(it, 0x6C);
                }
                it.b.mode += 4;
                self.item_register_attacker(h);
            }
            2 => {
                it.b.mode += 2;
                self.item_register_attacker(h);
            }
            _ => self.item_free(it, i),
        }
    }
}

#[cfg(test)]
mod overlay_tests {
    use super::*;

    fn rom() -> Option<&'static dino_assets::AssetPack> {
        static A: std::sync::OnceLock<Option<&'static dino_assets::AssetPack>> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let path = std::path::Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("ROM loads"))))
        })
    }

    /// The glow of a torch (0x29B06): at the torch by the table 0x29C26, Z + 2 or - 1 by the frame parity, the swing flame while the holder swings, gone with the torch.
    #[test]
    fn the_torch_glow_follows_the_torch_and_ends_with_it() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);

        // take every item slot first: slot 3 is written directly below, not allocated
        for _ in 0..24 {
            g.items.alloc();
        }
        let torch = g.items.handle(3);
        {
            let t = &mut g.items.slots[3];
            t.b.live = 1;
            t.b.k20 = 0;
            t.b.mode = 2;
            t.b.sub = 0;
            t.b.visible = 1;
            t.b.set_xi(0x200);
            t.b.set_yi(0x10);
            t.b.set_zi(0x50);
            t.b.param_b = 0;
            t.m.sw(KIND, 0x203);
        }
        let glow = g.items.handle(5);
        {
            let o = &mut g.items.slots[5];
            o.b.live = 1;
            o.b.k20 = class::OVERLAY * 4;
            o.m.sw(0x58, 0x203);
            o.m.sw(0x5A, torch);
        }
        g.cam.s2.cam_x = 0x140; // near the torch: it is not removed for being far from the screen
        g.w.frame = 2;
        g.pool_e_update();
        let o = &g.items.slots[5];
        // frame byte 0: (dx, dy) = (-35, 10); the torch faces right (0): X - 35, Y + 10, Z + 2
        assert_eq!((o.b.mode, o.b.xi(), o.b.yi(), o.b.zi(), o.b.visible), (2, 0x200 - 35, 0x10 + 10, 0x52, 1));
        assert!(g.draw_list.iter().any(|k| *k as u16 == glow), "the glow is in the draw list");
        g.w.frame = 3;
        g.pool_e_update();
        assert_eq!(g.items.slots[5].b.zi(), 0x4F, "Z - 1 in the frames with bit 0 of G.$1 set");
        g.items.slots[3].b.facing = 1;
        g.pool_e_update();
        assert_eq!(g.items.slots[5].b.xi(), 0x200 + 35, "dx is negated for a torch that faces left");
        let _ = glow;
        g.items.slots[3].m.sw(KIND, 0x202); // another weapon in the slot: the glow ends
        g.pool_e_update();
        g.pool_e_update();
        assert_eq!(g.items.slots[5].b.live, 0);
    }
}
