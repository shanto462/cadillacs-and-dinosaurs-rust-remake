//! Player-side collision passes and damage (owner: player port).
//! Spec: `re/specs/player/SPEC.md` sections 4.2, 4.3, 4.6, 4.8, 5.1, 6.
//!
//! Damage formulas keep the 68000 arithmetic: `mulu.w` uses only the low 16 bits of its
//! operands, `lsr.w` shifts only the low word. Large products wrap exactly like the original.

use super::{Game, Handle, Slot, collide, slot_of};
use dino_assets::AttackRecord;
use dino_assets::player::PlayerData;

/// `mulu.w`: 16 x 16 -> 32 bits.
fn mulu(a: u32, b: u32) -> u32 {
    (a & 0xFFFF) * (b & 0xFFFF)
}

/// 0x2B904 player -> enemy damage. Inputs are the values the routine reads; returns
/// (damage word, counter-hit flag G.$4F4). `r1`, `r2` are the two rand() results in call order.
#[allow(clippy::too_many_arguments)]
pub fn player_damage(
    t: &PlayerData,
    base: u8,
    attack_id: u8,
    power: u8,
    level: u16,
    enemy_kind: u8,
    enemy_atk: u8,
    enemy_w84: u16,
    enemy_hp: u16,
    r1: u16,
    r2: u16,
) -> (u16, bool) {
    let mut d1: u32 = base as u32;
    let mut counter = false;
    if enemy_kind == 4 {
        // counter hit: the enemy is attacking (word index = enemy +$58)
        let m = t.counter_scale.get(enemy_atk as usize).copied().unwrap_or(0) as u32;
        if m != 0 {
            d1 = (d1 & 0xFFFF_0000) | ((mulu(d1, m) as u16 >> 5) as u32);
            counter = true;
        }
    }
    // weakness pairs at 0xA32FA + enemy +$84: (id / 2, factor, id / 2, factor)
    let w = enemy_w84 as usize;
    let wb = |k: usize| t.weakness.get(w + k).copied().unwrap_or(0);
    let m = if wb(0).wrapping_add(wb(0)) == attack_id {
        Some(wb(1))
    } else if wb(2).wrapping_add(wb(2)) == attack_id {
        Some(wb(3))
    } else {
        None
    };
    if let Some(m) = m {
        let p = mulu(d1, m as u32);
        d1 = (p & 0xFFFF_0000) | ((p as u16 >> 5) as u32);
        // 0x2B984 `st.b $4F5(a5)`: the next byte of G.$4F4; the pass tests the word (`tst.w $4F4(a5)`, 0x10AAE), so a weakness hit is a counter hit
        // too: the second spark (type 5) and 400 points (M5-18: Jack's running jump kick on a Ferris, A7)
        counter = true;
    }
    d1 = mulu(d1, power as u32);
    let k = ((enemy_w84 << 3).wrapping_add(level)) as i16 as isize;
    let lt = if k >= 0 { t.level_type_scale.get(k as usize).copied().unwrap_or(0x20) } else { 0x20 };
    d1 = mulu(d1, lt as u32);
    d1 >>= 10;
    if d1 == 0 {
        d1 = 1;
    }
    let mut d3 = enemy_hp;
    let mut d2: u16 = 0;
    while d3 > 100 {
        d2 = d2.wrapping_add(0x40);
        d3 -= 100;
    }
    d2 = d2.wrapping_add(r1 & 0x3E);
    let w1 = t.hp_hundreds_scale.get(d2 as usize / 2).copied().unwrap_or(32) as u32;
    d1 = mulu(d1, w1);
    let band: u16 = if d3 >= 0x4B {
        0
    } else if d3 >= 0x32 {
        0x40
    } else if d3 >= 0x19 {
        0x80
    } else {
        0xC0
    };
    let w2 = t.hp_band_scale[((band + (r2 & 0x3E)) / 2) as usize & 127] as u32;
    d1 = mulu(d1, w2);
    let r = (d1 as u16) >> 10; // lsr.w: the high word is ignored
    (if r == 0 { 1 } else { r }, counter)
}

/// 0x6422 enemy -> player damage. `r` is the rand() result.
pub fn enemy_damage(t: &PlayerData, base: u8, level: u16, enemy_w84: u16, chr: u8, hp: u16, r: u16) -> u16 {
    let k = ((enemy_w84 << 3) as usize) + (level & 0x1F) as usize;
    let m = t.enemy_level_scale.get(k).copied().unwrap_or(0x20) as u32;
    let mut d1 = (mulu(base as u32, m) as u16) >> 5;
    let band: u16 = if hp >= 0x4C {
        0
    } else if hp >= 0x33 {
        0x40
    } else if hp >= 0x1A {
        0x80
    } else {
        0xC0
    };
    let red = t.enemy_hp_reduce[((band + (r & 0x3E)) / 2) as usize & 127];
    let s = d1.wrapping_sub(red);
    d1 = if (s as i16) < 0 { 1 } else { s };
    d1 = (mulu(d1 as u32, t.defense[chr as usize & 3] as u32) as u16) >> 5;
    let hs = t.enemy_hp_scale[(((hp >> 2) & 0x1E) / 2) as usize];
    d1 = (mulu(d1 as u32, hs as u32) as u16) >> 5;
    if d1 == 0 {
        d1 = 1;
    }
    if hp != 1 && hp <= d1 && d1 - hp < 5 {
        d1 = hp - 1; // an overkill of 0..4 leaves 1 HP
    }
    d1
}

/// BCD add of 4 bytes (abcd chain), saturating at 99999999 on carry (0xFFC8).
pub fn bcd_add(a: u32, b: u32) -> u32 {
    let mut out = 0u32;
    let mut carry = 0u32;
    for k in 0..8 {
        let s = ((a >> (4 * k)) & 0xF) + ((b >> (4 * k)) & 0xF) + carry;
        carry = (s >= 10) as u32;
        out |= (if s >= 10 { s - 10 } else { s }) << (4 * k);
    }
    if carry != 0 { 0x9999_9999 } else { out }
}

impl Game {
    fn player_attack_record(&self, i: usize) -> AttackRecord {
        let p = &self.players[i];
        let v = &self.assets.player.attacks[p.b.id as usize & 3];
        v.get(p.b.atk as usize / 2).copied().unwrap_or_default()
    }

    /// 0xFFC8: add score `table[off]` to player `i` (byte offset `off` from attack byte 5).
    pub fn player_add_score(&mut self, i: usize, off: u8) {
        // No score in a scripted scene of kind 1 (G.$4DD, G.$4DE), nor for +$02 < 0.
        if self.w.scripted != 0 && self.spawn.g.b4de == 1 {
            return;
        }
        let who = self.players[i].b.who;
        if (who as i8) < 0 {
            return;
        }
        // HUD redraw flags: -$7133 / -$7132 / -$7131 by +$02, then -$7134.
        self.hud.score_dirty_p[(who as usize).min(2)] = true;
        self.hud.score_dirty = true;
        let k = (off as i8 as isize + 4) / 4 - 1;
        let Some(&v) = (k >= 0).then(|| self.assets.player.score_bcd.get(k as usize)).flatten() else { return };
        let p = &mut self.players[i];
        p.score = bcd_add(p.score, v);
    }

    /// Enemy +$87 guard flag (a pool A enemy; 0 for any other object).
    fn player_enemy_b87(&self, h: Handle) -> u8 {
        match slot_of(h) {
            Some(Slot::Enemy(i)) => self.enemies.slots[i].m.b(super::enemy::off::B87),
            _ => 0,
        }
    }

    /// G.$4F3, copied from G.$4FB by the enemy census (0x32440): the enemies alive in the last
    /// frame. 3 or more disables player vs player hits (D2a section 10, D2c).
    fn player_g4f3(&self) -> u8 {
        self.spawn.g.alive
    }

    /// 0x10664 player vs player (attacker frame class 4): 1 HP, never below 1.
    pub fn pass_10664(&mut self) {
        if self.w.scripted != 0 {
            return;
        }
        let list: Vec<Handle> = self.combat.get(0x6718).to_vec();
        if list.len() < 2 {
            return;
        }
        let s = &self.assets.sprites;
        for &a in &list {
            let Some(Slot::Player(ai)) = slot_of(a) else { continue };
            if self.players[ai].b.kind != 4 {
                continue;
            }
            for &v in &list {
                let Some(Slot::Player(vi)) = slot_of(v) else { continue };
                if v == a || self.players[vi].b.live & 2 != 0 {
                    continue;
                }
                if self.players[ai].b.serial == self.players[vi].hit_serial {
                    continue;
                }
                if self.players[ai].b.attack_box == 0 {
                    break;
                }
                if self.players[vi].b.body_box == 0 || matches!(self.players[vi].b.kind, 6 | 8 | 0xA) {
                    continue;
                }
                let (ab, vb) = (&self.players[ai].b, &self.players[vi].b);
                let Some((ha, hb)) = collide::boxes_hit(s, ab, ab.attack_box, vb, vb.body_box) else { continue };
                let a6 = self.snd_a6_128f8(ab, vb, vb.body_box, &hb); // 0x10706: the box test leaves a6 on the box record
                let g = self.player_g4f3();
                if g >= 3 {
                    continue;
                }
                let (add, limit) = [(1u16, 2u16), (0, 1), (0, 0)][g as usize];
                let dz = (ab.zi().wrapping_sub(vb.zi()) as u16).wrapping_add(add);
                if dz > limit {
                    continue;
                }
                let rec = self.player_attack_record(ai);
                let (serial, knock) = (self.players[ai].b.serial, self.players[ai].b.knock);
                let vp = &mut self.players[vi];
                vp.b.live = 2;
                vp.b.set_state(0x0202_0000);
                vp.b7a = 0;
                vp.b.attacker = a;
                vp.hits_t = 0x3C;
                vp.hits = vp.hits.wrapping_add(1);
                vp.slow = 0;
                vp.combo = 0;
                vp.hit_serial = serial;
                vp.b.hp_prev = vp.b.hp;
                vp.b.hp = vp.b.hp.wrapping_sub(1);
                if (vp.b.hp as i16) <= 0 {
                    vp.b.hp = 1;
                }
                vp.stun_refill = 0x78;
                vp.b.stun = vp.b.stun.saturating_sub(rec.stun);
                vp.b.knock = knock;
                let r = if vp.action == 0x22 { 0x22 } else { rec.reaction };
                vp.b.reaction = r;
                vp.b.act = r;
                vp.b.step = 0;
                vp.b.hitstop = rec.hit_stop;
                self.players[ai].b.hitstop = rec.hit_stop;
                if rec.sound != 0 {
                    // 0x107D8: a6 is still the box record of the box test 0x128F8 (0x10706).
                    let (ax, a6_24) = (self.players[ai].b.xi(), self.snd_a6_byte(a6));
                    self.snd_hit(rec.sound, ax, a6_24); // 0x962A
                }
                self.players[vi].b.b9a = rec.b7;
                self.spawn_hit_spark(a, v, ha, hb, rec.spark);
                self.player_drop_check(vi); // 0x1177A
                self.player_power_factor(vi); // 0x11738
            }
        }
    }

    /// 0x107F8: player frames vs enemies, by the player's frame class +$2E (table 0x10820).
    pub fn pass_107f8(&mut self) {
        let list: Vec<Handle> = self.combat.get(0x6718).to_vec();
        for a in list {
            let Some(Slot::Player(i)) = slot_of(a) else { continue };
            match self.players[i].b.kind {
                0 => self.player_probe_108fa(i),
                2 => {
                    if self.w.scripted == 0 && self.players[i].regrab_cd == 0 {
                        self.player_grab_1084a(i);
                    }
                }
                4 | 6 | 8 => self.player_attack_10962(i),
                _ => {}
            }
        }
    }

    /// 0x108FA combo-throw probe: remember the first normal enemy touching the probe box.
    fn player_probe_108fa(&mut self, i: usize) {
        if self.players[i].b.attack_box == 0 {
            return;
        }
        let s = &self.assets.sprites;
        for e in self.combat.get(0x671E).to_vec() {
            let eb = self.body(e);
            if !(eb.mode == 2 && eb.sub == 0) || eb.airborne != 0 || (eb.b63 as i8) < 0 {
                continue;
            }
            let pb = &self.players[i].b;
            if pb.attack_box == 0 {
                return;
            }
            if eb.body_box == 0 || eb.kind == 0xA {
                continue;
            }
            if collide::boxes_hit(s, pb, pb.attack_box, eb, eb.body_box).is_some() {
                self.players[i].b.partner = e;
                return;
            }
        }
    }

    /// 0x1084A grab test (frame class 2).
    fn player_grab_1084a(&mut self, i: usize) {
        let s = &self.assets.sprites;
        let me = Game::player_handle(i);
        for e in self.combat.get(0x671E).to_vec() {
            let eb = self.body(e);
            if eb.live & 2 != 0 || eb.airborne != 0 || (eb.b63 as i8) < 0 {
                continue;
            }
            let pb = &self.players[i].b;
            if pb.attack_box == 0 {
                return;
            }
            if eb.body_box == 0 || !matches!(eb.kind, 0 | 2) {
                continue;
            }
            if collide::boxes_hit(s, pb, pb.attack_box, eb, eb.body_box).is_none() {
                continue;
            }
            // 0x108AE
            let (b72, who) = (eb.b72, self.players[i].b.who);
            let p = &mut self.players[i];
            p.b.partner = e;
            p.escape = b72;
            p.b.set_state(0x0204_0000);
            p.b.kind = 0xC;
            p.knees = 0;
            let eb = self.body_mut(e);
            eb.partner = me;
            eb.set_state(0x0206_0000);
            eb.kind = 0xE;
            eb.who = who;
            eb.hp_prev = eb.hp;
            return;
        }
    }

    /// 0x10962 attack test (frame classes 4, 6, 8) against every enemy.
    fn player_attack_10962(&mut self, i: usize) {
        let s = &self.assets.sprites;
        for e in self.combat.get(0x671E).to_vec() {
            let eb = self.body(e);
            if eb.live & 2 != 0 || eb.serial == self.players[i].b.serial {
                continue;
            }
            if self.players[i].b.attack_box == 0 {
                return;
            }
            if eb.body_box == 0 || matches!(eb.kind, 8 | 0xA | 0xE) {
                continue;
            }
            let pb = &self.players[i].b;
            // None: 0x1128E multi-part boss link (enemy +$63 == 4). TODO(enemy port)
            if let Some((ha, hb)) = collide::boxes_hit(s, pb, pb.attack_box, eb, eb.body_box) {
                self.player_hits_enemy(i, e, ha, hb);
            }
        }
    }

    /// 0x109C2: a player attack connects with enemy `e`.
    fn player_hits_enemy(&mut self, i: usize, e: Handle, ha: dino_assets::HitBox, hb: dino_assets::HitBox) {
        let me = Game::player_handle(i);
        let (serial, who) = (self.players[i].b.serial, self.players[i].b.who);
        {
            let eb = self.body_mut(e);
            eb.serial = serial;
            eb.who = who;
        }
        let p = &self.players[i];
        if self.player_enemy_b87(e) != 0 && p.b.kind != 6 && p.action != 0x13 && p.action != 0x12 {
            self.players[i].b.hitstop = 1;
            // 0x10B50[enemy +$84]: guard handler; carry = blocked (sound 0x9732, spark type 2).
            // Ported: BLADE / RAZOR (0x404CA), the poachers (0x3DFC6, names 8, 0x48, 0x4C).
            if (self.enemy_has_guard_c14(e) && self.enemy_guard_c14(e)) || (self.enemy_has_guard_c13(e) && self.enemy_guard_c13(e)) {
                self.snd_9732(self.body(e).xi());
                self.spawn_hit_spark(me, e, ha, hb, 2); // 0x10B42 -> 0x12720 with d5 = 2
                return;
            }
        }
        let rec = self.player_attack_record(i);
        let r1 = self.rand();
        let r2 = self.rand();
        let (kind, eatk, ehp) = {
            let eb = self.body(e);
            (eb.kind, eb.atk, eb.hp)
        };
        let p = &self.players[i];
        let (dmg, counter) = player_damage(
            &self.assets.player,
            rec.damage as u8,
            p.b.atk,
            p.power,
            self.w.level,
            kind,
            eatk,
            self.player_enemy_w84(e),
            ehp,
            r1,
            r2,
        );
        let (facing, pkind, px) = (p.b.facing, p.b.kind, p.b.xi());
        let eb = self.body_mut(e);
        eb.live = 2;
        eb.set_state(0x0202_0000);
        eb.attacker = me;
        eb.hp_prev = eb.hp;
        let hp = (eb.hp as i16).wrapping_sub(dmg as i16);
        eb.hp = if hp > 0 { hp as u16 } else { 0 };
        eb.stun = eb.stun.saturating_sub(rec.stun);
        eb.reaction = rec.reaction;
        eb.act = rec.reaction;
        eb.step = 0;
        eb.hitstop = rec.hit_stop;
        eb.b9a = rec.b7;
        self.players[i].b.hitstop = rec.hit_stop;
        // 0x10A2C: clr.b $7A(a3), the enemy's decision wait (pool A only).
        if let Some(Slot::Enemy(ei)) = slot_of(e) {
            self.enemies.slots[ei].m.sb(super::enemy::off::WAIT, 0);
        }
        if rec.sound != 0 {
            // 0x10A94: a6 is the last table the damage routine 0x2B904 pointed it at (`lea $2BB4E(pc),a6`, 0x2BA10), whatever the box test
            // or the guard path (0x10A12) left before.
            let (ax, a6_24) = (self.players[i].b.xi(), self.snd_a6_byte(0x2BB4E));
            self.snd_hit(rec.sound, ax, a6_24); // 0x962A
        }
        self.player_add_score(i, rec.score);
        self.spawn_hit_spark(me, e, ha, hb, rec.spark);
        if counter {
            self.fx_spawn_hit_spark_alt(me, e, ha, hb, 5);
            self.player_add_score(i, 0x20);
        }
        let ex = {
            let eb = self.body_mut(e);
            eb.facing = facing ^ 1;
            eb.xi()
        };
        if pkind == 6 {
            self.body_mut(e).facing = (px > ex) as u8;
            if self.w.scripted == 0 {
                self.player_special_cost(i);
            }
        }
        // 0x1128E multi-part boss link (enemy +$63 == 4). TODO(enemy port)
    }

    /// 0x10B08: HP cost of the special: 10 for the first victim, then 4, budget 20, min 1 HP.
    fn player_special_cost(&mut self, i: usize) {
        let p = &mut self.players[i];
        let cost: u16 = if p.special_budget == 0x14 { 10 } else { 4 };
        p.special_budget = p.special_budget.wrapping_sub(cost);
        if (p.special_budget as i16) < 0 {
            return;
        }
        let hp = (p.b.hp as i16).wrapping_sub(cost as i16);
        p.b.hp = if hp > 0 { hp as u16 } else { 1 };
        self.hud_life_bar(i); // 0x16446 HP bar redraw, after the cost
    }

    /// 0x12526: knee / throw damage to the held enemy (+$70). No hit-stop.
    pub(super) fn player_grab_hit(&mut self, i: usize) {
        let me = Game::player_handle(i);
        let e = self.players[i].b.partner;
        if e == 0 || slot_of(e).is_none() {
            return;
        }
        let (serial, who) = (self.players[i].b.serial, self.players[i].b.who);
        {
            let eb = self.body_mut(e);
            eb.attacker = me;
            eb.serial = serial;
            eb.who = who;
        }
        let rec = self.player_attack_record(i);
        let r1 = self.rand();
        let r2 = self.rand();
        let (kind, eatk, ehp) = {
            let eb = self.body(e);
            (eb.kind, eb.atk, eb.hp)
        };
        let p = &self.players[i];
        let (dmg, _) = player_damage(
            &self.assets.player,
            rec.damage as u8,
            p.b.atk,
            p.power,
            self.w.level,
            kind,
            eatk,
            self.player_enemy_w84(e),
            ehp,
            r1,
            r2,
        );
        let eb = self.body_mut(e);
        eb.hp_prev = eb.hp;
        let hp = (eb.hp as i16).wrapping_sub(dmg as i16);
        if hp > 0 {
            eb.hp = hp as u16;
        } else {
            eb.set_state(0x0202_0000);
            eb.hp = 0;
        }
        if eb.stun > rec.stun {
            eb.stun -= rec.stun;
        } else {
            eb.set_state(0x0202_0000);
            eb.stun = 0;
        }
        eb.reaction = rec.reaction;
        eb.act = rec.reaction;
        eb.step = 0;
        eb.b9a = rec.spark; // quirk: byte 6 here (byte 7 in the normal hit path)
        if rec.sound != 0 {
            let (ax, vf) = (self.players[i].b.xi(), self.players[i].b.facing);
            self.snd_hit(rec.sound, ax, vf); // 0x962A
        }
        self.player_add_score(i, rec.score);
    }

    /// 0x11682: enemy `e` hits player `p` (the enemy pass 0x115B0 did the loop, the serial test
    /// and the box test). Also covers 0x117D8 (player held by an enemy with +$C7 set).
    pub fn player_hit_by_enemy(&mut self, p: usize, e: Handle) {
        let me = Game::player_handle(p);
        let s = &self.assets.sprites;
        let (boxes, a6) = {
            let (eb, pb) = (self.body(e), &self.players[p].b);
            let boxes = collide::boxes_hit(s, eb, eb.attack_box, pb, pb.body_box);
            // 0x11668 / 0x11D80: the box test leaves a6 on the box record (the sound of 0x11720 / 0x11E38 / 0x1183E reads +$24 of it)
            let a6 = boxes.map_or(0, |(_, hb)| self.snd_a6_128f8(eb, pb, pb.body_box, &hb));
            (boxes, a6)
        };
        let eatk = self.body(e).atk;
        let rec = self.assets.player.enemy_attacks.get(eatk as usize).copied().unwrap_or_default();
        let (eserial, eknock) = {
            let eb = self.body(e);
            (eb.serial, eb.knock)
        };
        if self.players[p].b.sub == 6 && self.players[p].bc7 != 0 {
            // 0x117D8: no damage scaling, record bytes shifted by one.
            let pl = &mut self.players[p];
            pl.b7a = 0;
            pl.item_type = 0;
            pl.drop = 1;
            pl.b.attacker = e;
            pl.b.hp_prev = pl.b.hp;
            let hp = (pl.b.hp as i16).wrapping_sub(rec.damage as i16);
            pl.b.hp = if hp > 0 { hp as u16 } else { 0 };
            pl.b.stun = pl.b.stun.saturating_sub(rec.stun);
            pl.b.knock = eknock;
            pl.b.reaction = rec.reaction;
            pl.b.hitstop = rec.hit_stop;
            pl.b75 = 0xFF;
            self.body_mut(e).hitstop = rec.hit_stop;
            if rec.sound != 0 {
                // 0x1183E: a6 is the box record (no damage routine on this path)
                let (ax, a6_24) = (self.body(e).xi(), self.snd_a6_byte(a6));
                self.snd_hit(rec.sound, ax, a6_24); // 0x962A
            }
            self.players[p].b.b9a = rec.spark;
            if let Some((ha, hb)) = boxes {
                self.spawn_hit_spark(e, me, ha, hb, rec.score);
            }
            self.player_drop_check(p);
            self.player_power_factor(p);
            return;
        }
        {
            let pl = &mut self.players[p];
            pl.b.live = 2;
            pl.b.set_state(0x0202_0000);
            pl.b7a = 0;
            pl.b.attacker = e;
            pl.hits_t = 0x3C;
            pl.hits = pl.hits.wrapping_add(1);
            pl.slow = 0;
            pl.combo = 0;
            pl.hit_serial = eserial;
            pl.b.hp_prev = pl.b.hp;
        }
        let r = self.rand();
        let (hp, chr) = (self.players[p].b.hp, self.players[p].b.id);
        let dmg = enemy_damage(&self.assets.player, rec.damage as u8, self.w.level, self.player_enemy_w84(e), chr, hp, r);
        let pl = &mut self.players[p];
        let nhp = (pl.b.hp as i16).wrapping_sub(dmg as i16);
        pl.b.hp = if nhp > 0 { nhp as u16 } else { 0 };
        pl.stun_refill = 0x78;
        pl.b.stun = pl.b.stun.saturating_sub(rec.stun);
        pl.b.knock = eknock;
        let reaction = if pl.action == 0x22 { 0x22 } else { rec.reaction };
        pl.b.reaction = reaction;
        pl.b.act = reaction;
        pl.b.step = 0;
        pl.b.hitstop = rec.hit_stop;
        pl.b.b9a = rec.b7;
        self.body_mut(e).hitstop = rec.hit_stop;
        if rec.sound != 0 {
            // 0x11720 / 0x11E38: the damage routine 0x6422 saved a6 (the box record) as a word and restored it with `movea.w` (0x64EE):
            // the sign extended low word, a byte of the low program for the heroes' box tables.
            let (ax, a6_24) = (self.body(e).xi(), self.snd_a6_byte(Game::snd_a6_word(a6)));
            self.snd_hit(rec.sound, ax, a6_24); // 0x962A
        }
        if let Some((ha, hb)) = boxes {
            self.spawn_hit_spark(e, me, ha, hb, rec.spark);
        }
        self.player_drop_check(p); // 0x1177A
        self.player_power_factor(p); // 0x11738
    }

    /// 0x1177A: drop the held weapon when hit, except Hannah with types 0x204..0x208 and
    /// Mess with 0x209.
    pub(super) fn player_drop_check(&mut self, i: usize) {
        let p = &mut self.players[i];
        let keep = match p.b.id {
            0 | 1 => false,
            3 => p.item_type == 0x209,
            _ => {
                let t = p.item_type;
                t & 0xFF00 == 0x200 && (t & 0xFF) > 3 && (t & 0xFF) < 9
            }
        };
        if !keep {
            p.item_type = 0;
            p.drop = 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tables() -> PlayerData {
        // Values from the ROM tables (re/specs/player/tables.txt): flat x/32 factors.
        PlayerData {
            counter_scale: vec![0, 0, 0, 0x30, 0, 0, 0x30, 0],
            weakness: vec![8, 0x30, 7, 0x30],
            level_type_scale: vec![0x20; 64],
            hp_hundreds_scale: vec![32; 64],
            hp_band_scale: vec![32; 128],
            enemy_level_scale: vec![0x20; 64],
            enemy_hp_reduce: vec![0; 128],
            enemy_hp_scale: [32; 16],
            defense: [32, 34, 35, 30],
            ..Default::default()
        }
    }

    #[test]
    fn player_damage_neutral_scales_keep_base() {
        let t = tables();
        // base 10, power 32 (x1), all factors x1: 10 * 32 * 32 >> 10 = 10, then * 32 * 32 >> 10.
        let (d, c) = player_damage(&t, 10, 0, 32, 0, 0, 0, 0, 50, 0, 0);
        assert_eq!((d, c), (10, false));
    }

    #[test]
    fn player_damage_counter_and_weakness() {
        let t = tables();
        // enemy attacking with +$58 = 6 -> counter factor 0x30 (x1.5): 20 -> 30.
        let (d, c) = player_damage(&t, 20, 2, 32, 0, 4, 6, 0, 50, 0, 0);
        assert_eq!((d, c), (30, true));
        // attack id 0x10 matches weakness id 8 (x1.5): 20 -> 30. The weakness sets G.$4F5, the second byte of the counter word the pass tests (M5-18).
        let (d, c) = player_damage(&t, 20, 0x10, 32, 0, 0, 0, 0, 50, 0, 0);
        assert_eq!((d, c), (30, true));
        // no pair, no counter
        let (_, c) = player_damage(&t, 20, 0x0C, 32, 0, 0, 0, 0, 50, 0, 0);
        assert!(!c);
    }

    #[test]
    fn player_damage_wraps_like_lsr_w() {
        let t = tables();
        // base 80: stage 1 = 80, then 80 * 32 * 32 = 81920 = 0x14000 -> low word 0x4000 >> 10 = 16.
        let (d, _) = player_damage(&t, 80, 0, 32, 0, 0, 0, 0, 50, 0, 0);
        assert_eq!(d, 16);
    }

    #[test]
    fn enemy_damage_defense_and_mercy() {
        let t = tables();
        // Mustapha (id 1, defense 34): 10 * 34 >> 5 = 10.
        assert_eq!(enemy_damage(&t, 10, 0, 0, 1, 100, 0), 10);
        // Hannah (id 2, defense 35): 20 * 35 >> 5 = 21.
        assert_eq!(enemy_damage(&t, 20, 0, 0, 2, 100, 0), 21);
        // HP 12, damage 14: overkill 2 < 5 -> leaves 1 HP (damage 11).
        assert_eq!(enemy_damage(&t, 14, 0, 0, 0, 12, 0), 11);
        // HP 12, damage 20: overkill 8 -> full damage.
        assert_eq!(enemy_damage(&t, 20, 0, 0, 0, 12, 0), 20);
    }

    /// 0x6422 mercy: an overkill of 0 to 4 leaves 1 HP; 5 or more kills; at HP 1 there is no mercy.
    #[test]
    fn enemy_damage_mercy_edges() {
        let t = tables();
        // HP 12, damage 16: overkill 4 is inside the window, the hero keeps 1 HP (damage 11).
        assert_eq!(enemy_damage(&t, 16, 0, 0, 0, 12, 0), 11);
        // HP 12, damage 17: overkill 5, no mercy.
        assert_eq!(enemy_damage(&t, 17, 0, 0, 0, 12, 0), 17);
        // HP 1: the next hit kills, no mercy (damage 3 stays 3).
        assert_eq!(enemy_damage(&t, 3, 0, 0, 0, 1, 0), 3);
    }

    /// 0xFFC8 score cap: the BCD sum saturates at 99999999 on a carry out of the top digit.
    #[test]
    fn bcd_score() {
        assert_eq!(bcd_add(0x0000_0990, 0x0000_0020), 0x0000_1010);
        assert_eq!(bcd_add(0x9999_9990, 0x0000_0100), 0x9999_9999);
        assert_eq!(bcd_add(0x9999_9999, 0x0000_0001), 0x9999_9999);
        assert_eq!(bcd_add(0x9999_9998, 0x0000_0001), 0x9999_9999);
    }
}

/// Rule tests that read the user's `dino.zip` at run time (they return early when it is absent).
#[cfg(test)]
mod rom_tests {
    use super::*;
    use crate::game::flow::config::Config;
    use dino_assets::AssetPack;
    use std::path::Path;
    use std::sync::OnceLock;

    /// The asset pack of the user's ROM (`../../../dino.zip`), `None` when it is not there.
    fn rom() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// The enemy damage on Jack, Mustapha, Hannah and Mess (ids 0 to 3) for one enemy and attack.
    fn per_hero(t: &PlayerData, base: u8, w84: u16) -> Vec<u16> {
        (0..4).map(|chr| enemy_damage(t, base, 0, w84, chr, 100, 0)).collect()
    }

    /// 0x2C88E: the hero factors are 32, 34, 35, 30 by character id (Jack, Mustapha, Hannah, Mess).
    /// Enemy to hero damage (0x6422) multiplies by the factor of the hero's id (D2b section 5.2).
    #[test]
    fn hero_damage_factor_order_from_rom() {
        let Some(a) = rom() else { return };
        let t = &a.player;
        assert_eq!(t.defense, [32, 34, 35, 30]);
        // The level 0 and HP 100 steps are x1.0 and x33/32 here, so only the hero factor differs.
        assert_eq!(t.enemy_level_scale[0], 32, "Ferris (name 0) at level 0");
        assert_eq!(t.enemy_hp_reduce[0], 0, "row 0 of the random reduction is all zero");
        assert_eq!(t.enemy_hp_scale[12], 33, "HP 100 falls in the row of 88 and more");
        // Base 32 keeps every step exact: 32 * defense >> 5, then * 33 >> 5.
        assert_eq!(per_hero(t, 32, 0), vec![33, 35, 36, 30]);
        // Thug (name 9, enemy +$84 = 36) at level 0, jab bases 8 and 6 (D2b section 5.2 and S13).
        assert_eq!(t.enemy_level_scale[36 * 8], 32, "thug at level 0");
        assert_eq!(per_hero(t, 8, 36), vec![8, 8, 8, 7]);
        assert_eq!(per_hero(t, 6, 36), vec![6, 6, 6, 5]);
    }

    /// 0x1D1C4, 0x1D204, 0x1CF5C, 0x1CEDC, 0x1D166: the code reads row 0 only, because the level
    /// band is always 0 (`move.b` reads the high byte of the level word, D2b section 4.4). The
    /// level band lookup itself is tested in `player_weapon.rs`.
    #[test]
    fn lie_and_dizzy_tables_row_0_values_from_rom() {
        let Some(a) = rom() else { return };
        let t = &a.player;
        let count = |v: &[u8], n: u8| v[..16].iter().filter(|&&x| x == n).count();
        // Lie time: 5 (6 cells), 15 (8 cells), 30 (2 cells). Mash meter: 15, 20, 25 with the same counts.
        assert_eq!((count(&t.lie_time, 5), count(&t.lie_time, 15), count(&t.lie_time, 30)), (6, 8, 2));
        assert_eq!((count(&t.lie_mash, 15), count(&t.lie_mash, 20), count(&t.lie_mash, 25)), (6, 8, 2));
        // Dizzy: stars to mash and maximum dizzy frames (16 cells each, in the ROM order).
        assert_eq!(t.dizzy_stars[..16], [15, 35, 35, 35, 50, 50, 50, 50, 65, 65, 65, 65, 80, 80, 80, 125]);
        assert_eq!(t.dizzy_time[..16], [40, 40, 50, 50, 50, 70, 70, 70, 90, 90, 90, 120, 120, 120, 150, 180]);
        // Dizzy chance masks by hit count 0..5: never below 4 hits, 75 percent at 4, always at 5.
        assert_eq!(t.dizzy_mask, [0, 0, 0, 0, 0xFFF0, 0xFFFF]);
    }

    /// 0x2C88E hero factors: the level 31 measurements of D2b section 5.2 (HP 100, `r` = 0, so the
    /// HP reduction is 0). The 5.2 row value 58 is name 0 (Ferris) at level 31; the thug row
    /// (name 9) gives 72 there, see the open item in the report.
    #[test]
    fn hero_damage_measured_at_level_31_from_rom() {
        let Some(a) = rom() else { return };
        let t = &a.player;
        assert_eq!(t.enemy_level_scale[31], 58, "name 0 at level 31");
        let at = |base: u8, w84: u16, level: u16, hp: u16| -> Vec<u16> {
            (0..4).map(|chr| enemy_damage(t, base, level, w84, chr, hp, 0)).collect()
        };
        assert_eq!(at(8, 0, 31, 100), vec![14, 14, 15, 13], "attack id 0, base 8");
        assert_eq!(at(6, 0, 31, 100), vec![10, 10, 10, 9], "attack id 1, base 6");
        assert_eq!(at(6, 0, 0, 100), vec![6, 6, 6, 5], "attack id 1, base 6, level 0");
        assert_eq!(at(8, 0, 0, 100), vec![8, 8, 8, 7], "attack id 0, base 8, level 0");
        // HP 20 (band 0xC0, reduction word 0 is 0): HP factor 27 from the words of 0x2C996.
        assert_eq!(t.enemy_hp_scale[2], 27, "HP 16 to 23");
        assert_eq!(at(6, 0, 0, 20), vec![5, 5, 5, 4]);
    }

    /// 0x2C9B6: the power table `$C6` has four rows of 32 cells. The row is by HP: 75 or more
    /// row 0, 50 to 74 row 1, 25 to 49 row 2, below 25 row 3 (`player_power_factor`).
    /// Every row holds 32 as the most common value; the top cells grow with the lower HP.
    #[test]
    fn power_table_rows_from_rom() {
        let Some(a) = rom() else { return };
        let t = &a.player;
        assert_eq!(t.power.len(), 128);
        let row = |r: usize| &t.power[r * 32..r * 32 + 32];
        let count = |r: usize, v: u8| row(r).iter().filter(|&&x| x == v).count();
        assert_eq!([count(0, 32), count(1, 32), count(2, 32), count(3, 32)], [27, 26, 25, 22]);
        assert_eq!(row(0)[..2], [24, 28]);
        assert_eq!(row(0)[29..], [36, 42, 48]);
        assert_eq!(row(1)[0], 28);
        assert_eq!(row(3)[31], 64, "row 3 has the one 64 at the top");
        // Every row is weighted to 32 and never drops below 24.
        for r in 0..4 {
            assert!(row(r).iter().all(|&x| x >= 24 && x <= 64), "row {r}");
            assert!(count(r, 32) >= 22, "row {r}");
        }
    }

    /// 0x6BFE: the extra life lists. The pointer word at 0x6BFE + 2 * sel gives the list; each
    /// list is absolute BCD thresholds ending in 99999999 (never reached). sel = 2 * option
    /// (option 0 none, 1 300000 and 700000, 2 factory 500000 and 1000000, 3 1000000 only).
    #[test]
    fn extra_life_lists_from_rom() {
        let Some(a) = rom() else { return };
        let list = |sel: u32| -> Vec<u32> {
            let l = 0x6BFE + a.text.u16(0x6BFE, sel) as u32;
            (0..).map(|k| a.text.u32(l, 4 * k)).take_while(|&v| v != 0x9999_9999).collect()
        };
        assert_eq!(list(0), Vec::<u32>::new(), "option 0: none");
        assert_eq!(list(2), vec![0x0030_0000, 0x0070_0000], "option 1");
        assert_eq!(list(4), vec![0x0050_0000, 0x0100_0000], "option 2, factory");
        assert_eq!(list(6), vec![0x0100_0000], "option 3");
    }

    /// 0x6BFE / G.$777B bits 5-6: the factory setting (option 2) gives one extra life at 500000 and
    /// one more at 1000000, one life per step (D2b section 1.3). The crossing runs through
    /// `hud_draw_all_scores` -> `hud_extend_check` (0x6B98).
    #[test]
    fn extra_life_thresholds_factory_setting_from_rom() {
        let Some(a) = rom() else { return };
        // Crosses 500000: prev score 499950, new score 500050 gives one life (1 -> 2).
        let mut g = Game::new(a);
        g.flow_apply_config(Config::from_text("").unwrap());
        g.start_stage(0, &[0]); // a real start gives the player a rank (the place text needs one)
        assert_eq!(g.flow.extend_sel(), 4, "factory: table entry 2 of 0x6BFE");
        let p = &mut g.players[0];
        p.b.live = 1;
        p.b.who = 0;
        p.lives = 1;
        p.score = 0x0049_9950;
        g.hud.stats[0].score_prev = 0x0049_9950;
        g.players[0].score = 0x0050_0050;
        g.hud_draw_all_scores();
        assert_eq!((g.players[0].lives, g.players[0].b10e), (2, 1));
        // The 1UP pose has not started yet: no second life at the same score.
        g.hud_draw_all_scores();
        assert_eq!(g.players[0].lives, 2);
        // Pose done (+$10E cleared): crossing 1000000 gives the next life (2 -> 3).
        g.players[0].b10e = 0;
        g.hud.stats[0].score_prev = 0x0050_0050;
        g.players[0].score = 0x0100_0050;
        g.hud_draw_all_scores();
        assert_eq!(g.players[0].lives, 3);
        // One big jump from 0 to 1500000 is one life only (one step per check).
        let mut g = Game::new(a);
        g.flow_apply_config(Config::from_text("").unwrap());
        g.start_stage(0, &[0]);
        g.players[0].b.live = 1;
        g.players[0].b.who = 0;
        g.players[0].lives = 1;
        g.players[0].score = 0x0150_0000;
        g.hud_draw_all_scores();
        assert_eq!(g.players[0].lives, 2);
    }

}
