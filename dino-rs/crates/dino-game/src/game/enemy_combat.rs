//! Enemy-side collision passes and the shared enemy hit reaction (0x2A606), get-up / dizzy,
//! grab release. Owner: enemy port. See `re/specs/enemy/SPEC.md` section 10.

use super::body::Body;
use super::enemy::{Enemy, off::*};
use super::{Game, Handle, Slot, collide, slot_of};

/// Hit reaction tables.
mod th {
    pub const LAUNCH: u32 = 0x2CBB0; // (vx, ax, vy, ay) records, see SPEC 10.2
    pub const THROW: u32 = 0x2CB46;
    pub const SLIDE: u32 = 0x2CB6E;
    pub const LIE_ADD: u32 = 0x2B604;
    pub const LIE: u32 = 0x2B614;
    pub const COUNTER: u32 = 0x2B8C4;
    pub const HIT_ANIMS: u32 = 0x2E68C;
    pub const DIZZY_HP: u32 = 0x2B158;
    pub const DIZZY_ALIVE: u32 = 0x2B168;
    pub const DIZZY_ROW: u32 = 0x2B180;
    pub const DIZZY_SPECIAL: u32 = 0x2B200;
    pub const DIZZY_END: u32 = 0x2B28C;
}

impl Game {
    pub fn pass_10dcc(&mut self) {}
    pub fn pass_1132a(&mut self) {}
    pub fn pass_10cd0(&mut self) {}
    pub fn pass_11962(&mut self) {}
    pub fn pass_11a9e(&mut self) {}
    pub fn pass_11f86(&mut self) {}
    pub fn pass_1201c(&mut self) {}
    pub fn pass_120d2(&mut self) {}
    pub fn pass_11e56(&mut self) {}
    pub fn pass_11cec(&mut self) {}
    pub fn pass_1185a(&mut self) {}

    /// 0x10F96: player attacks (list 0x671A / $6722, frame class 4) against the hurtable
    /// enemies (list 0x671E / $6726).
    pub fn pass_10f96(&mut self) {
        let attackers = self.combat.get(0x671A).to_vec();
        for a in attackers {
            if self.body(a).kind != 4 {
                continue;
            }
            let defenders = self.combat.get(0x671E).to_vec();
            for d in defenders {
                let (ab, db) = (self.body(a), self.body(d));
                if db.live & 2 != 0 || ab.serial == db.serial {
                    continue;
                }
                // 0x10FF2 by defender class: 8 / 0xA are immune.
                if matches!(db.kind, 8 | 0xA) {
                    continue;
                }
                self.enemy_hit_by_player(a, d);
            }
        }
    }

    /// 0x11008: a player attack (`a`) hits enemy `d`.
    fn enemy_hit_by_player(&mut self, a: Handle, d: Handle) {
        let (ab, db) = (self.body(a).clone(), self.body(d).clone());
        if ab.attack_box == 0 || db.body_box == 0 {
            return;
        }
        let Some((ha, hb)) = collide::boxes_hit(&self.assets.sprites, &ab, ab.attack_box, &db, db.body_box) else {
            return;
        };
        let Some(Slot::Enemy(ei)) = slot_of(d) else {
            return; // pool A only in this port so far
        };
        {
            let e = &mut self.enemies.slots[ei];
            e.b.serial = ab.serial;
            e.b.who = ab.who;
        }
        let guard = self.enemies.slots[ei].m.b(B87) != 0
            && ab.kind != 6
            && !matches!(self.player_action(a), 0x12 | 0x13);
        if guard {
            // 0x10C10 per +$84: every entry used in stage 1 is `rts` with carry clear (no block).
        }
        {
            let e = &mut self.enemies.slots[ei];
            e.b.live = 2;
            e.b.set_state(0x0202_0000);
            e.m.sb(WAIT, 0);
            e.b.attacker = a;
        }
        let ch = if (ab.who as i8) >= 0 { ab.id as usize & 3 } else { 0 };
        let id = ab.atk;
        if id == 0x6E || id == 0x6C {
            return self.enemy_hit_nonlethal(a, d, ch, id);
        }
        let r = self.assets.sprites.player_attacks[ch][(id / 2) as usize];
        let mut dmg = r.damage as u8 as u32;
        self.enemies.slots[ei].b.hp_prev = self.enemies.slots[ei].b.hp;
        if self.player_move_word(a) == 0x203 {
            dmg = self.enemy_charge_scale(a, dmg);
        }
        let dmg = self.enemy_damage_scale(dmg, &ab, ei);
        let e = &mut self.enemies.slots[ei];
        let hp = (e.b.hp as i16).wrapping_sub(dmg as u16 as i16);
        e.b.hp = if hp > 0 { hp as u16 } else { 0 };
        e.b.stun = e.b.stun.saturating_sub(r.stun);
        e.b.reaction = r.reaction;
        e.b.act = r.reaction;
        e.b.step = 0;
        e.b.hitstop = r.hit_stop;
        self.body_mut(a).hitstop = r.hit_stop;
        let w = self.player_move_word(a);
        if w == 0x202 || w == 0x203 {
            let p = self.body(a).partner;
            if p != 0 {
                self.body_mut(p).hitstop = r.hit_stop;
            }
        }
        if r.sound != 0 {
            self.snd_hit(r.sound); // 0x962A
        }
        if (ab.who as i8) >= 0 {
            self.enemy_hit_score(ab.who, r.score);
        }
        let mut spark = r.spark;
        self.enemies.slots[ei].b.b9a = r.b7;
        let k20 = self.enemies.slots[ei].b.k20;
        if matches!(k20, 0xA4 | 0xEC | 0x4C) {
            spark = 0;
        }
        self.spawn_hit_spark(a, d, ha, hb, spark);
        let src = if (ab.who as i8) >= 0 { Game::player_handle(ab.who as usize % 3) } else { a };
        let sx = self.body(src).xi();
        let e = &mut self.enemies.slots[ei];
        e.b.facing = (sx > e.b.xi()) as u8;
        // 0x1128E: Fessenden parts only (+$63 == 4).
    }

    /// 0x111C6: attack ids 0x6E / 0x6C (player specials): signed damage, HP floor 1.
    fn enemy_hit_nonlethal(&mut self, a: Handle, d: Handle, ch: usize, id: u8) {
        let Some(Slot::Enemy(ei)) = slot_of(d) else { return };
        let r = self.assets.sprites.player_attacks[ch][(id / 2) as usize];
        let ab = self.body(a).clone();
        let e = &mut self.enemies.slots[ei];
        e.b.hp_prev = e.b.hp;
        let hp = (e.b.hp as i16).wrapping_sub(r.damage as i16);
        e.b.hp = if hp > 0 { hp as u16 } else { 1 };
        e.b.reaction = r.reaction;
        e.b.act = r.reaction;
        e.b.step = 0;
        e.b.hitstop = r.hit_stop;
        if r.sound != 0 {
            self.snd_hit(r.sound); // 0x962A
        }
        if (ab.who as i8) >= 0 {
            self.enemy_hit_score(ab.who, r.score);
        }
        self.enemies.slots[ei].b.b9a = r.b7;
        let src = if (ab.who as i8) >= 0 { Game::player_handle(ab.who as usize % 3) } else { a };
        let sx = self.body(src).xi();
        let e = &mut self.enemies.slots[ei];
        e.b.facing = (sx > e.b.xi()) as u8;
    }

    /// 0xFFC8: per-hit score (offset into 0x10292), not during a scripted walk to area 1.
    fn enemy_hit_score(&mut self, who: u8, off: u8) {
        if self.w.scripted != 0 && self.spawn.g.b4de == 1 {
            return;
        }
        if (who as i8) < 0 {
            return;
        }
        // ext.w d0; lea 4(a1,d0.w): the long ends at 0x10292 + off + 4.
        let v = self.assets.enemy.t.u32(0x10292, off as i8 as i32 as u32);
        self.enemy_add_score(who, v);
    }

    /// Player +$A0 word of a player attacker (held / previous input), 0 for other objects.
    fn player_move_word(&self, a: Handle) -> u16 {
        match slot_of(a) {
            Some(Slot::Player(i)) => ((self.players[i].held as u16) << 8) | self.players[i].prev as u16,
            _ => 0,
        }
    }

    /// Player +$C8 action class of a player attacker.
    fn player_action(&self, a: Handle) -> u8 {
        match slot_of(a) {
            Some(Slot::Player(i)) => self.players[i].action,
            _ => 0,
        }
    }

    /// 0x11262: damage scale by the partner's +$80 (riding / charge). [I]
    fn enemy_charge_scale(&self, a: Handle, d: u32) -> u32 {
        let p = self.body(a).partner;
        let k = if p != 0 {
            match slot_of(p) {
                Some(Slot::Enemy(i)) => self.enemies.slots[i].m.b(T80),
                Some(Slot::Player(i)) => (self.players[i].w80 >> 8) as u8,
                _ => 0,
            }
        } else {
            0
        };
        if k <= 0xF {
            return d;
        }
        let m = if k <= 0x1E {
            0x28
        } else if k <= 0x2D {
            0x30
        } else {
            0x40
        };
        let v = d.wrapping_mul(m) & 0xFFFF_FFFF;
        (v & 0xFFFF_0000) | ((v & 0xFFFF) >> 5)
    }

    /// 0x2BA2A: player -> enemy damage scaling (2 rand() calls).
    fn enemy_damage_scale(&mut self, d0: u32, ab: &Body, ei: usize) -> u32 {
        let lo = |v: u32| v & 0xFFFF;
        let t = &self.assets.enemy.t;
        let e = &self.enemies.slots[ei];
        let mut d = d0;
        if e.b.kind == 4 {
            let m = t.u16(0xA320C, 2 * e.b.atk as u32) as u32;
            if m != 0 {
                d = lo(d) * m >> 5; // mulu.w ; lsr.l #5
            }
        }
        let w84 = e.m.w(W84) as u32;
        let a = ab.atk;
        let p0 = t.u8(0xA32FA, w84).wrapping_add(t.u8(0xA32FA, w84));
        let p2 = t.u8(0xA32FA, w84 + 2).wrapping_add(t.u8(0xA32FA, w84 + 2));
        let mult = if p0 == a {
            Some(t.u8(0xA32FA, w84 + 1))
        } else if p2 == a {
            Some(t.u8(0xA32FA, w84 + 3))
        } else {
            None
        };
        if let Some(m) = mult {
            let v = lo(d) * m as u32; // mulu.w ; lsr.w #5 (low word only)
            d = (v & 0xFFFF_0000) | ((v & 0xFFFF) >> 5);
        }
        if (ab.who as i8) >= 0 {
            let p = (ab.who & 3) as usize % 3;
            d = lo(d) * self.players[p].power as u32;
        }
        let f = t.u8(0x2BECE, (w84 << 3).wrapping_add(self.w.level as u32)) as u32;
        d = (lo(d) * f) >> 10;
        if d == 0 {
            d = 1;
        }
        let mut hp = e.b.hp;
        let mut bar = 0u32;
        while hp > 100 {
            bar += 0x40;
            hp -= 100;
        }
        let r = (self.rand() & 0x3E) as u32;
        let f = self.assets.enemy.t.u16(0x2BC4E, bar + r) as u32;
        d = lo(d) * f;
        let band = if hp >= 0x4B {
            0
        } else if hp >= 0x32 {
            0x40
        } else if hp >= 0x19 {
            0x80
        } else {
            0xC0
        };
        let r = (self.rand() & 0x3E) as u32;
        let f = self.assets.enemy.t.u16(0x2BB4E, band + r) as u32;
        (lo(d) * f) >> 10
    }

    /// 0x115B0: enemy attacks (frame class 4, no hit-stop) against the players.
    pub fn pass_115b0(&mut self) {
        let attackers = self.combat.get(0x671E).to_vec();
        'outer: for e in attackers {
            if self.body(e).hitstop != 0 {
                continue;
            }
            let players = self.combat.get(0x6718).to_vec();
            for p in players {
                if self.body(p).live & 2 != 0 {
                    continue;
                }
                if self.body(e).kind != 4 {
                    break; // 0x1160E: clr.w d7 ends this attacker
                }
                if self.w.scripted != 0 {
                    continue; // 0x11612
                }
                let (eb, pb) = (self.body(e).clone(), self.body(p).clone());
                if eb.attack_box == 0 || pb.body_box == 0 {
                    continue;
                }
                // 0x1163E by player class: 6 / 8 / 0xA end the whole pass (clr.l d7).
                if matches!(pb.kind, 6 | 8 | 0xA) {
                    break 'outer;
                }
                // 0x11652: same swing already hit this player (+$E9) by the same attacker (+$68).
                if let Some(Slot::Player(i)) = slot_of(p) {
                    if eb.serial == self.players[i].hit_serial && pb.attacker == e {
                        continue;
                    }
                }
                if collide::boxes_hit(&self.assets.sprites, &eb, eb.attack_box, &pb, pb.body_box).is_none() {
                    continue;
                }
                // The effect on the player (0x11670..0x1177A) lives in the player port.
                if let Some(Slot::Player(i)) = slot_of(p) {
                    self.player_hit_by_enemy(i, e);
                }
            }
        }
    }

    // ------------------------------------------------------------------ hit reaction 0x2A606

    /// 0x16B36: enemy HP bar request for the HUD: appends a message (`kind` in the high byte, the
    /// name id +$3, HP bar +$6A, HP before the hit +$6E, HP +$6C, the object address) to the ring
    /// of the credited player (+$2, none when negative) unless that ring slot is still in use.
    /// `h` is a pool A object (its address was loaded with `movea.w`, so the long is sign extended).
    fn enemy_hud_hit(&mut self, h: Handle, b: &Body, kind: u8) {
        if (b.who as i8) < 0 || b.who > 2 {
            return;
        }
        let i = b.who as usize;
        let w = self.hud.ring_w[i];
        let m = &mut self.hud.rings[i][(w >> 4) as usize];
        if (m[0] as i16) >= 0 {
            return; // the slot is still unread: the request is dropped
        }
        *m = [(kind as u16) << 8 | b.id as u16, b.hp_bar, b.hp_prev, b.hp, 0xFFFF, h, m[6], m[7]];
        self.hud.ring_w[i] = w.wrapping_add(0x10);
    }

    /// 0x2B49A: start hit animation `n` from the type's hit table (L[0x2E68C + +$84]).
    fn enemy_hit_anim(&self, e: &mut Enemy, n: u16) {
        e.b.draw_flags = 0;
        let table = self.assets.enemy.t.u32(th::HIT_ANIMS, e.m.w(W84) as u32);
        let first = self.assets.sprites.anim(table, n);
        e.b.load_frame(&self.assets.sprites, first);
    }

    /// 0x2B478: counter-attack delay (+$7A) by level band.
    fn enemy_counter_delay(&mut self, e: &mut Enemy) {
        let r = self.rand() & 0xF;
        let band = (self.w.level.wrapping_add(self.w.level)) & 0x30;
        let v = self.assets.enemy.t.u8(th::COUNTER, (band + r) as u32);
        e.m.sb(WAIT, v);
    }

    /// 0x2B4BA: lie-down time (+$80 byte).
    fn enemy_lie_time(&mut self, e: &mut Enemy) {
        if e.b.hp == 0 {
            e.m.sb(T80, 6);
            return;
        }
        let r = self.rand() & 0xF;
        let row = (e.m.w(W84) as u32) * 4;
        let mut d = self.assets.enemy.t.u8(th::LIE, row + r as u32);
        let r = self.rand() & 0xF;
        d = d.wrapping_add(self.assets.enemy.t.u8(th::LIE_ADD, r as u32));
        if (d as i8) < 0 {
            d = 0;
        }
        e.m.sb(T80, d);
    }

    /// 0x2B506: keep a flying body inside camX-0x50..camX+0x1D0 (by its direction).
    fn enemy_edge_clamp(&self, e: &mut Enemy) {
        let cx = self.cam.s2.cam_x;
        let x = e.b.xi() as u16;
        if e.b.vx < 0 {
            let lim = cx.wrapping_add(0x3B0);
            if lim >= x.wrapping_add(0x400) {
                e.b.set_xi_keep(lim.wrapping_sub(0x400) as i16);
            }
        } else {
            let lim = cx.wrapping_add(0x5D0);
            if lim < x.wrapping_add(0x400) {
                e.b.set_xi_keep(lim.wrapping_sub(0x400) as i16);
            }
        }
    }

    /// 0x2A980: dead "big" enemies bounce off the screen sides.
    fn enemy_screen_bounce(&self, e: &mut Enemy) {
        if (e.b.b63 as i8) <= 0 || e.b.hp != 0 {
            return;
        }
        let cx = self.cam.s2.cam_x;
        if e.b.vx >= 0 {
            if cx.wrapping_add(0x150) < e.b.xi() as u16 {
                e.b.vx = e.b.vx.wrapping_neg();
                e.b.ax = e.b.ax.wrapping_neg();
            }
        } else if (cx.wrapping_add(0x30) as i16) > e.b.xi() {
            e.b.vx = e.b.vx.wrapping_neg();
            e.b.ax = e.b.ax.wrapping_neg();
        }
    }

    /// Launch record (vx, ax, vy, ay) at `addr`; vx/ax negated when facing right.
    fn enemy_launch(&self, e: &mut Enemy, addr: u32) {
        let t = &self.assets.enemy.t;
        let (mut vx, mut ax) = (t.i16(addr, 0), t.i16(addr, 2));
        if e.b.facing != 0 {
            vx = vx.wrapping_neg();
            ax = ax.wrapping_neg();
        }
        e.b.vx = vx;
        e.b.ax = ax;
        e.b.vy = t.i16(addr, 4);
        e.b.ay = t.i16(addr, 6);
    }

    /// 0x2A606: dispatch on +$06 (table 0x2A620), then register for the hit lists.
    pub fn enemy_hit_reaction(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0x00 => self.enemy_flinch(e, h, 0, 4),
            0x02 => self.enemy_flinch(e, h, 1, 6),
            0x04 => self.enemy_knockdown(e, h, 4),
            0x06 => self.enemy_knockdown(e, h, 6),
            0x08 => self.enemy_thrown(e, h),
            0x0A | 0x0C => self.enemy_burn(e, h),
            0x0E | 0x10 | 0x12 => self.enemy_slide(e, h, 0x0E),
            0x14 | 0x16 | 0x18 => self.enemy_slide(e, h, 0x14),
            0x1A => self.enemy_getup(e, h),
            0x1C => self.enemy_slump(e, h),
            a => panic!("enemy hit reaction {a:#x}"),
        }
        self.enemy_push_hurtable(h);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x2A63E (anim 0, heavy 4) / 0x2A680 (anim 1, heavy 6): light flinch.
    fn enemy_flinch(&mut self, e: &mut Enemy, h: Handle, anim: u16, heavy: u8) {
        if e.b.step == 0 {
            if e.b.hp == 0 || e.b.stun == 0 || e.b.airborne != 0 {
                e.b.act = heavy;
                e.b.step = 0;
                return self.enemy_knockdown(e, h, heavy);
            }
            let b = e.b.clone();
            self.enemy_hud_hit(h, &b, 0);
            self.enemy_hit_anim(e, anim);
            self.enemy_counter_delay(e);
            e.b.step += 2;
            return;
        }
        e.b.hitstop = e.b.hitstop.wrapping_sub(1);
        if e.b.hitstop == 0 {
            self.enemy_recover(e, h);
        }
    }

    /// 0x2A6C2 (type 4: anim 2, launch 0x2CBB0) / 0x2A84C (type 6: anim 3, launch 0x2CBC0).
    fn enemy_knockdown(&mut self, e: &mut Enemy, h: Handle, ty: u8) {
        e.b.act = ty;
        self.enemy_edge_clamp(e);
        let base = if ty == 4 { th::LAUNCH } else { th::LAUNCH + 0x10 };
        match e.b.step {
            0 => {
                let b = e.b.clone();
                self.enemy_hud_hit(h, &b, 0);
                self.enemy_hit_anim(e, if ty == 4 { 2 } else { 3 });
                self.enemy_launch(e, base);
                if e.b.hp == 0 {
                    self.enemy_launch(e, base + 8);
                    self.snd_enemy_death(e.b.b63); // 0x2A748 / 0x2A752 (type 6: 0x2A8D2 / 0x2A8DC)
                    self.item_drop_from_enemy(e);
                }
                e.b.vz = 0;
                e.b.az = 0;
                e.b.airborne = 1;
                e.m.sb(B83, 0);
                e.b.reaction = 4;
                e.b.stun = 10;
                e.b.step += 2;
            }
            2 => {
                // 0x606E water splash: effects port.
                self.combat.push(0x6AC2, h);
                e.b.motion();
                self.enemy_screen_bounce(e);
                if e.b.param_a == 0 {
                    self.enemy_step(&mut e.b);
                }
                Self::enemy_floor_collision(&mut e.b);
                if e.b.wall != 0 && e.b.wall & 1 == 0 {
                    return self.enemy_slump_start(e, h);
                }
                if e.b.airborne == 0 {
                    e.b.airborne = 1;
                    let a = if e.b.hp != 0 { base + 4 } else { base + 0xC };
                    let t = &self.assets.enemy.t;
                    e.b.vy = t.i16(a, 0);
                    let ay = t.i16(a, 2);
                    e.b.ay = ay.wrapping_add(ay);
                    e.b.step += 2;
                    self.snd_9796(); // 0x2A7E8 (type 6: 0x2A972)
                    self.enemy_step(&mut e.b);
                }
            }
            4 => {
                self.combat.push(0x6AC2, h);
                e.b.motion();
                self.enemy_screen_bounce(e);
                if e.b.param_a == 0 {
                    self.enemy_step(&mut e.b);
                }
                Self::enemy_floor_collision(&mut e.b);
                if e.b.airborne == 0 {
                    self.enemy_lie_time(e);
                    if e.b.hp == 0 {
                        e.m.sb(T80, 1);
                    }
                    self.snd_9796(); // 0x2A832
                    e.b.step += 2;
                }
            }
            _ => {
                let t = e.m.b(T80);
                e.m.sb(T80, t.wrapping_sub(1));
                if t == 0 {
                    return self.enemy_getup_or_die(e, h);
                }
                self.enemy_step(&mut e.b);
            }
        }
    }

    /// 0x2A9C8: thrown by a player (+$68 = thrower).
    fn enemy_thrown(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_edge_clamp(e);
        match e.b.step {
            0 => {
                e.m.sb(B83, 0);
                e.b.reaction = 4;
                e.b.stun = 10;
                e.b.hitstop = 0;
                let t = self.body(e.b.attacker).clone();
                let (anim, rec) = if t.id == 3 {
                    if t.param_a == 2 { (9, 0x2CB5E) } else { (10, 0x2CB66) }
                } else {
                    e.b.airborne = 1;
                    (8, th::THROW + 8 * t.id as u32)
                };
                self.enemy_hit_anim(e, anim);
                let tt = &self.assets.enemy.t;
                let (mut vx, mut ax) = (tt.i16(rec, 0), tt.i16(rec, 2));
                if t.xi() as u16 >= e.b.xi() as u16 {
                    vx = vx.wrapping_neg();
                    ax = ax.wrapping_neg();
                }
                e.b.vx = vx;
                e.b.ax = ax;
                e.b.vy = tt.i16(rec, 4);
                e.b.ay = tt.i16(rec, 6);
                e.b.vz = 0;
                e.b.az = 0;
                if e.b.hp == 0 {
                    self.snd_enemy_death(e.b.b63); // 0x2AAD6 / 0x2AAE0
                }
                e.b.step += 2;
                self.enemy_thrown_fly(e, h);
            }
            2 => self.enemy_thrown_fly(e, h),
            4 => {
                self.combat.push(0x6B30, h);
                e.b.motion();
                self.enemy_screen_bounce(e);
                if e.b.param_a == 0 {
                    self.enemy_step(&mut e.b);
                }
                Self::enemy_floor_collision(&mut e.b);
                if e.b.airborne == 0 {
                    self.enemy_lie_time(e);
                    if e.b.hp == 0 {
                        e.m.sb(T80, 0xA);
                        self.item_drop_from_enemy(e);
                    }
                    self.snd_9796(); // 0x2ABE4
                    e.b.step += 2;
                }
            }
            _ => {
                let t = e.m.b(T80);
                e.m.sb(T80, t.wrapping_sub(1));
                if t == 0 {
                    return self.enemy_getup_or_die(e, h);
                }
                self.enemy_step(&mut e.b);
            }
        }
    }

    /// 0x2AAEA: thrown, flying.
    fn enemy_thrown_fly(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.hitstop != 0 {
            e.b.hitstop -= 1;
            return;
        }
        self.combat.push(0x6B30, h); // 0x4BB0
        self.combat.push(0x6AC2, h);
        e.b.motion();
        self.enemy_screen_bounce(e);
        if e.b.param_a == 0 {
            self.enemy_step(&mut e.b);
        }
        Self::enemy_floor_collision(&mut e.b);
        if e.b.wall != 0 && e.b.wall & 1 == 0 {
            self.enemy_shake_request(e); // 0x2AB8C
            let b = e.b.clone();
            self.enemy_hud_hit(h, &b, 0);
            if e.b.hp == 0 {
                self.item_drop_from_enemy(e);
            }
            return self.enemy_slump_start(e, h);
        }
        if e.b.airborne == 0 {
            e.b.vy = e.b.vy.wrapping_neg();
            e.b.ay = e.b.ay.wrapping_add(e.b.ay);
            e.b.step += 2;
            self.enemy_step(&mut e.b);
            self.enemy_shake_request(e); // 0x2AB8C
            self.snd_9796(); // 0x2AB7C
            let b = e.b.clone();
            self.enemy_hud_hit(h, &b, 0);
        }
    }

    /// 0x2AB8C: screen shake request G.$755C = 1 (3 when the thrower is Mess, id 3).
    fn enemy_shake_request(&mut self, e: &Enemy) {
        let a = e.b.attacker;
        let big = slot_of(a).is_some() && self.body(a).id == 3;
        self.cam.shake_req = if big { 3 } else { 1 };
    }

    /// 0x2AC02: burn knockdown (types 0xA / 0xC).
    fn enemy_burn(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_edge_clamp(e);
        match e.b.step {
            0 => {
                if e.b.hp == 0 && (e.m.b(ATTACKING) as i8) >= 0 && e.b.reaction == 0xC {
                    // Charred death (0x93D48 effect, frame list 0x2AC1C, 0x9A84): effects port.
                    let b = e.b.clone();
                    self.enemy_hud_hit(h, &b, 0);
                    self.item_drop_from_enemy(e);
                    return self.enemy_getup_or_die(e, h);
                }
                let b = e.b.clone();
                self.enemy_hud_hit(h, &b, 0);
                self.enemy_hit_anim(e, 7);
                e.b.palette_override = 0x84;
                self.enemy_launch(e, 0x2CBD0);
                if e.b.hp == 0 {
                    self.enemy_launch(e, 0x2CBD8);
                    self.item_drop_from_enemy(e);
                    if (e.b.b63 as i8) > 0 {
                        self.sound(0x58); // 0x2ACE0: 0x97D2 (the original drops the item twice around it)
                    }
                }
                e.b.vz = 0;
                e.b.az = 0;
                self.enemy_burn_flames(h, e);
                self.spawn.g.b7b2 = 2;
                e.b.airborne = 1;
                e.m.sb(B83, 0);
                e.b.reaction = 4;
                e.b.stun = 10;
                e.b.step += 2;
                self.enemy_burn_fly(e, h);
            }
            2 => self.enemy_burn_fly(e, h),
            4 => {
                self.combat.push(0x6AC2, h);
                e.b.motion();
                self.enemy_screen_bounce(e);
                if e.b.param_a == 0 {
                    self.enemy_step(&mut e.b);
                }
                Self::enemy_floor_collision(&mut e.b);
                if e.b.airborne == 0 {
                    self.enemy_lie_time(e);
                    if e.b.hp == 0 {
                        e.m.sb(T80, 1);
                    }
                    e.b.step += 2;
                }
            }
            _ => {
                let t = e.m.b(T80);
                e.m.sb(T80, t.wrapping_sub(1));
                if t == 0 {
                    self.spawn.g.b7b2 = 0;
                    return self.enemy_getup_or_die(e, h);
                }
                self.enemy_step(&mut e.b);
            }
        }
    }

    fn enemy_burn_fly(&mut self, e: &mut Enemy, h: Handle) {
        self.combat.push(0x6AC2, h);
        e.b.motion();
        self.enemy_screen_bounce(e);
        if e.b.param_a == 0 {
            self.enemy_step(&mut e.b);
        }
        Self::enemy_floor_collision(&mut e.b);
        if e.b.airborne == 0 {
            e.b.palette_override = 0;
            e.b.airborne = 1;
            let a = if e.b.hp != 0 { 0x2CBD4 } else { 0x2CBDC };
            let t = &self.assets.enemy.t;
            e.b.vy = t.i16(a, 0);
            let ay = t.i16(a, 2);
            e.b.ay = ay.wrapping_add(ay);
            e.b.step += 2;
            self.snd_9796(); // 0x2AD74
            self.enemy_step(&mut e.b);
        }
    }

    /// 0x2AD98: flames on a burning enemy (pool C class 0x20, list 0x2CA36 by +$84 / 2).
    fn enemy_burn_flames(&mut self, h: Handle, e: &Enemy) {
        // The effects port spawns one flame per call; the list length and entries live in its data.
        let _ = (h, e);
    }

    /// 0x2ADDE (types 0xE..0x12) / 0x2AEB2 (types 0x14..0x18): slide back without lift.
    fn enemy_slide(&mut self, e: &mut Enemy, h: Handle, first: u8) {
        let heavy = if first == 0x0E { 4 } else { 6 };
        match e.b.step {
            0 => {
                if e.b.hp == 0 || e.b.stun == 0 || e.b.airborne != 0 {
                    e.b.act = heavy;
                    e.b.step = 0;
                    return self.enemy_knockdown(e, h, heavy);
                }
                let b = e.b.clone();
                self.enemy_hud_hit(h, &b, 0);
                self.enemy_hit_anim(e, if first == 0x0E { 0 } else { 1 });
                self.enemy_counter_delay(e);
                let k = (e.b.act - first) as u32 / 2;
                let rec = th::SLIDE + 6 * k.min(2);
                let t = &self.assets.enemy.t;
                e.m.sb(T80, t.u16(rec, 0) as u8);
                let (mut vx, mut ax) = (t.i16(rec, 2), t.i16(rec, 4));
                if e.b.facing != 0 {
                    vx = vx.wrapping_neg();
                    ax = ax.wrapping_neg();
                }
                e.b.vx = vx;
                e.b.ax = ax;
                e.b.vy = 0;
                e.b.ay = 0;
                e.b.vz = 0;
                e.b.az = 0;
                e.b.step += 2;
                self.enemy_slide_wait(e, h);
            }
            2 => self.enemy_slide_wait(e, h),
            _ => self.enemy_slide_move(e, h),
        }
    }

    fn enemy_slide_wait(&mut self, e: &mut Enemy, h: Handle) {
        e.b.hitstop = e.b.hitstop.wrapping_sub(1);
        if e.b.hitstop != 0 {
            return;
        }
        e.b.step += 2;
        self.enemy_slide_move(e, h);
    }

    fn enemy_slide_move(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(T80).wrapping_sub(1);
        e.m.sb(T80, t);
        if t == 0 {
            return self.enemy_recover(e, h);
        }
        if self.w.frame & 1 != 0 {
            // 0x92E5E dust (d0 = 2): effects port.
        }
        Self::enemy_floor_collision(&mut e.b);
        if e.b.wall != 0 && e.b.wall & 1 != 0 {
            return;
        }
        e.b.motion();
    }

    /// 0x2AF86: dead -> class death routine; else get up (type 0x1A).
    fn enemy_getup_or_die(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.hp == 0 {
            e.b.set_state(0x0400_0000);
            e.b.live = 1;
            let w = e.b.who;
            if (w as i8) >= 0 {
                if self.spawn.g.car_mode == 0 {
                    let i = (w & 3) as usize % 3;
                    self.spawn.g.kills[i] = self.spawn.g.kills[i].wrapping_add(1);
                } else {
                    for k in &mut self.spawn.g.kills {
                        *k = k.wrapping_add(1);
                    }
                }
            }
            return self.rank_reroll_aggression();
        }
        if (e.b.who as i8) < 0 {
            e.b.who = 0;
        }
        e.b.set_state(0x0202_1A00);
        self.enemy_hit_anim(e, 4);
        self.enemy_getup(e, h);
    }

    /// 0x2AFF0: type 0x1A (get-up, then maybe dizzy).
    fn enemy_getup(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.step == 0 {
            if !self.enemy_step(&mut e.b) {
                return;
            }
            if self.enemy_dizzy_roll(e) {
                return self.enemy_dizzy_start(e, h);
            }
            return self.enemy_recover(e, h);
        }
        // 0x2B244 dizzy
        self.spawn.g.dizzy = self.spawn.g.dizzy.wrapping_add(1);
        let t = e.m.b(T80);
        if t != 0 {
            e.m.sb(T80, t - 1);
            self.enemy_step(&mut e.b);
            return;
        }
        e.m.sb(T80, 0x14);
        let c = e.m.b(0x81);
        e.m.sb(0x81, c.wrapping_sub(1));
        if c == 0 {
            return self.enemy_recover(e, h);
        }
        let mask = self.assets.enemy.t.u32(th::DIZZY_END, (self.w.level as u32) * 4);
        let r = self.rand() & 0x1F;
        if (mask >> r) & 1 != 0 {
            return self.enemy_recover(e, h);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x2B01E: dizzy chance after the get-up animation.
    fn enemy_dizzy_roll(&mut self, e: &mut Enemy) -> bool {
        if e.m.b(POISE_T) != 0 {
            return false;
        }
        let cx = self.cam.s2.cam_x as i16;
        if cx >= e.b.xi() || cx.wrapping_add(0x180) < e.b.xi() {
            return false;
        }
        let row = if e.b.b9a != 0 { ((e.b.b9a & 3) as u32) << 5 } else { 0 };
        let r = self.rand() & 0x1F;
        let mask = self.assets.enemy.t.u32(th::DIZZY_ROW, row + (self.rank.aggression as u32) * 4);
        if (mask >> r) & 1 == 0 {
            return false;
        }
        if e.b.b9a != 0 {
            return true;
        }
        if self.spawn.g.dizzy_prev != 0 {
            return false;
        }
        let q = e.b.hp_bar >> 4;
        let hp = e.b.hp;
        let k = if hp < q {
            0
        } else if hp < 2 * q {
            4
        } else if hp < 3 * q {
            8
        } else {
            0xC
        };
        let mask = self.assets.enemy.t.u32(th::DIZZY_HP, k);
        let r = self.rand() & 0x1F;
        if (mask >> r) & 1 == 0 {
            return false;
        }
        let n = self.spawn.g.alive.min(5) as u32;
        let mask = self.assets.enemy.t.u32(th::DIZZY_ALIVE, n * 4);
        let r = self.rand() & 0x1F;
        if (mask >> r) & 1 == 0 {
            return false;
        }
        if (e.m.b(ATTACKING) as i8) >= 0 {
            return true;
        }
        if !matches!(e.b.k20, 0x48 | 0xF4 | 0x64 | 0x90 | 0x10 | 0x11C | 0x150) {
            return false;
        }
        let mask = self.assets.enemy.t.u32(th::DIZZY_SPECIAL, 0);
        let r = self.rand() & 0x1F;
        (mask >> r) & 1 != 0
    }

    /// 0x2B204.
    fn enemy_dizzy_start(&mut self, e: &mut Enemy, h: Handle) {
        e.b.reaction = 0x30;
        if e.b.k20 == 0x48 {
            return self.enemy_recover(e, h);
        }
        e.b.set_state(0x0202_1A02);
        self.enemy_hit_anim(e, 5);
        e.m.sb(T80, 0x5A);
        e.m.sb(0x81, 8);
        e.b.live = 1;
        // 0x94338 dizzy stars (pool C class 0x4C): effects port.
        self.enemy_getup(e, h);
    }

    /// 0x2B2B8: wall slump (type 0x1C).
    fn enemy_slump_start(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0202_1C00);
        e.m.sb(T80, 0x1E);
        self.enemy_hit_anim(e, 6);
        e.b.facing = (e.b.vx < 0) as u8;
        self.snd_9796(); // 0x2B2E0
        self.enemy_slump(e, h);
    }

    fn enemy_slump(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.step == 0 {
            let t = e.m.b(T80);
            if t != 0 {
                e.m.sb(T80, t - 1);
                return;
            }
            e.b.y = e.b.y.wrapping_sub(0x18000);
            let floor = e.b.floor.wrapping_add(e.b.floor2);
            if floor < e.b.yi() {
                return;
            }
            self.enemy_lie_time(e);
            e.m.sb(T80, e.m.b(T80).wrapping_add(0xA));
            e.b.airborne = 0;
            e.b.y = (e.b.y & 0xFFFF) | ((floor as i32) << 16);
            e.b.step += 2;
            self.enemy_step(&mut e.b);
            return;
        }
        let t = e.m.b(T80);
        e.m.sb(T80, t.wrapping_sub(1));
        if t == 0 {
            e.b.stun = 10;
            return self.enemy_getup_or_die(e, h);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x2B54E: back to the type's AI (table 0x2B56C by +$84).
    fn enemy_recover(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sb(POISE_T, 0x3C);
        e.b.palette_override = 0;
        e.b.b9a = 0;
        e.b.live = 1;
        match e.m.w(W84) {
            0x00 | 0x04 | 0x68 => self.enemy_c2_reset_ai(e, h),
            w => self.enemy_recover_other(e, h, w),
        }
    }

    /// 0x2B35A: held by a player (+$70). Not ported yet: released at once.
    pub fn enemy_grabbed(&mut self, e: &mut Enemy, h: Handle) {
        e.b.wall = 0;
        Self::enemy_snap_to_floor(&mut e.b);
        e.b.reaction = 0x20;
        self.enemy_recover(e, h);
    }
}
