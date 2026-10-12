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

    /// Enemy +$87 custom hit handler flag. TODO(enemy port): read `Enemy` once the field exists.
    fn player_enemy_b87(&self, _h: Handle) -> u8 {
        0
    }

    /// G.$4F3, copied from G.$4FB by the enemy census (0x32440); 3 or more disables player vs
    /// player hits. TODO(enemy port): expose it; 3 (off) until then.
    fn player_g4f3(&self) -> u8 {
        3
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
                    self.sound(rec.sound as u16);
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
            match collide::boxes_hit(s, pb, pb.attack_box, eb, eb.body_box) {
                Some((ha, hb)) => self.player_hits_enemy(i, e, ha, hb),
                None => {} // 0x1128E multi-part boss link (enemy +$63 == 4). TODO(enemy port)
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
            // TODO(enemy port): only enemy +$84 == 0xA4 can block.
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
        // enemy +$7A = 0 (enemy port field)
        if rec.sound != 0 {
            self.sound(rec.sound as u16);
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
        // 0x16446 HP bar redraw (HUD port).
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
            self.sound(rec.sound as u16);
        }
        self.player_add_score(i, rec.score);
    }

    /// 0x11682: enemy `e` hits player `p` (the enemy pass 0x115B0 did the loop, the serial test
    /// and the box test). Also covers 0x117D8 (player held by an enemy with +$C7 set).
    pub fn player_hit_by_enemy(&mut self, p: usize, e: Handle) {
        let me = Game::player_handle(p);
        let s = &self.assets.sprites;
        let boxes = {
            let (eb, pb) = (self.body(e), &self.players[p].b);
            collide::boxes_hit(s, eb, eb.attack_box, pb, pb.body_box)
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
                self.sound(rec.sound as u16);
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
            self.sound(rec.sound as u16);
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
        // attack id 0x10 matches weakness id 8 (x1.5): 20 -> 30.
        let (d, _) = player_damage(&t, 20, 0x10, 32, 0, 0, 0, 0, 50, 0, 0);
        assert_eq!(d, 30);
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

    #[test]
    fn bcd_score() {
        assert_eq!(bcd_add(0x0000_0990, 0x0000_0020), 0x0000_1010);
        assert_eq!(bcd_add(0x9999_9990, 0x0000_0100), 0x9999_9999);
    }
}
