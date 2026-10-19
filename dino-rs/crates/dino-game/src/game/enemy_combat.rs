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
    /// 0x10DCC / 0x10E34: list 0x671A attack objects against heroes.
    pub fn pass_10dcc(&mut self) {
        for a in self.combat.get(0x671A).to_vec() {
            if self.body(a).kind != 4 { continue; }
            for d in self.combat.get(0x6718).to_vec() {
                let (ab, db) = (self.body(a), self.body(d));
                // This pass compares +$5E, not the last-hit serial at +$E9.
                if db.live & 2 != 0 || ab.who == db.who || ab.serial == db.serial
                    || matches!(db.kind, 8 | 0xA) { continue; }
                self.enemy_attack_object_hits_hero(a, d);
            }
        }
    }

    /// 0x10E4A: raw object record damage, unconditional weapon drop, then 0x11738.
    fn enemy_attack_object_hits_hero(&mut self, a: Handle, d: Handle) {
        let (ab, db) = (self.body(a).clone(), self.body(d).clone());
        if ab.attack_box == 0 || db.body_box == 0 { return; }
        let Some((ha, hb)) = collide::boxes_hit(&self.assets.sprites, &ab, ab.attack_box, &db, db.body_box) else { return };
        let friendly = (ab.who as i8) >= 0;
        if (friendly && ab.zi() != db.zi()) || matches!(ab.atk, 0x6E | 0x6C) { return; }
        let Some(Slot::Player(i)) = slot_of(d) else { return };
        let ch = if friendly { ab.id as usize } else { 0 };
        let rec = self.assets.sprites.player_attacks[ch][ab.atk as usize / 2];
        let p = &mut self.players[i];
        p.b.live = 2;
        p.b.set_state(0x0202_0000);
        p.b7a = 0;
        p.item_type = 0;
        p.drop = 1;
        p.b.attacker = a;
        p.hits_t = 0x3C;
        p.hits = p.hits.wrapping_add(1);
        p.slow = 0;
        p.hit_serial = ab.serial;
        p.b.hp_prev = db.hp;
        let damage = if friendly { 1 } else { rec.damage as u16 };
        // SUB.W followed by BGT uses N == V, including signed subtraction overflow.
        p.b.hp = if (db.hp as i16) > damage as i16 {
            db.hp.wrapping_sub(damage)
        } else { u16::from(friendly) };
        p.stun_refill = 0x78;
        p.b.stun = p.b.stun.saturating_sub(rec.stun);
        p.b.knock = ab.knock;
        let reaction = if p.action == 0x22 { 0x22 } else { rec.reaction };
        p.b.reaction = reaction;
        p.b.act = reaction;
        p.b.step = 0;
        p.b.hitstop = rec.hit_stop;
        p.b.b9a = rec.b7;
        self.body_mut(a).hitstop = rec.hit_stop;
        let paired = matches!(self.player_move_word(a), 0x202 | 0x203);
        if paired {
            self.body_mut(ab.partner).hitstop = rec.hit_stop; // 0x10F6E: a6 = the partner (+$70)
        }
        if rec.sound != 0 {
            // 0x10F7C jsr 0x962A: the pan from a2 (the attack object), the mode from +$24 of a6, which is the partner after
            // 0x10F6E and otherwise a register left from before the pass (the attack object's facing stands in, as at the other sites)
            let ax = self.body(a).xi();
            let facing = if paired { self.body(ab.partner).facing } else { self.body(a).facing };
            self.snd_hit(rec.sound, ax, facing);
        }
        self.spawn_hit_spark(a, d, ha, hb, rec.spark); // 0x12720
        self.player_power_factor(i); // 0x11738, the only RNG call in this pass
    }

    /// 0x1132A / 0x11396: attack objects against the hostile list, without a guard test.
    pub fn pass_1132a(&mut self) {
        for a in self.combat.get(0x671A).to_vec() {
            if self.body(a).kind != 4 { continue; }
            for d in self.combat.get(0x6B64).to_vec() {
                let db = self.body(d);
                if db.live & 2 == 0 && db.serial != self.body(a).serial
                    && !matches!(db.kind, 8 | 0xA) {
                    self.enemy_monster_hit(a, d, false);
                }
            }
        }
    }

    /// 0x11A9E / 0x11AD8: hero attacks against monsters, including specials and frame class 8.
    pub fn pass_11a9e(&mut self) {
        for a in self.combat.get(0x6718).to_vec() {
            if !matches!(self.body(a).kind, 4 | 6 | 8) { continue; }
            for d in self.combat.get(0x6B64).to_vec() {
                let db = self.body(d);
                if db.live & 2 == 0 && db.serial != self.body(a).serial
                    && matches!(db.kind, 0 | 2 | 4 | 6 | 0xC) {
                    self.enemy_monster_hit(a, d, true);
                }
            }
        }
    }

    /// 0x11B36 / 0x11396: one overlapping hero or attack object hits a monster.
    fn enemy_monster_hit(&mut self, a: Handle, d: Handle, hero: bool) {
        let (ab, db) = (self.body(a).clone(), self.body(d).clone());
        if ab.attack_box == 0 || db.body_box == 0 { return; }
        let Some((ha, hb)) = collide::boxes_hit(&self.assets.sprites, &ab, ab.attack_box, &db, db.body_box) else { return };
        let Some(Slot::Enemy(di)) = slot_of(d) else { return };
        self.body_mut(d).serial = ab.serial;
        self.body_mut(d).who = ab.who;
        if hero && self.enemies.slots[di].m.b(B87) != 0 && ab.kind != 6
            && !matches!(self.player_action(a), 0x12 | 0x13)
            && self.enemy_monster_guard(a, di) {
            // 0x11CB6 -> 0x96CE: alternate the block sound.
            let old = self.w.snd_toggle & 2 != 0;
            self.w.snd_toggle ^= 2;
            self.sound(if old { 0x65 } else { 0xE4 });
            self.spawn_hit_spark(a, d, ha, hb, 2);
            return;
        }
        {
            let e = &mut self.enemies.slots[di];
            e.b.live = 2;
            e.b.set_state(0x0202_0000);
            e.m.sb(WAIT, 0);
            e.b.attacker = a;
            e.b.hp_prev = db.hp;
        }
        let ch = if hero { (ab.k20 / 4) as usize } else if (ab.who as i8) >= 0 { ab.id as usize } else { 0 };
        let rec = self.assets.sprites.player_attacks[ch][(ab.atk / 2) as usize];
        let nonlethal = !hero && matches!(ab.atk, 0x6C | 0x6E);
        let mut damage = rec.damage as u32;
        if !hero && !nonlethal && self.player_move_word(a) == 0x203 {
            damage = self.enemy_charge_scale(a, damage);
        }
        let (damage, counter) = self.enemy_monster_damage(damage, a, di, hero);
        let mapped = self.assets.enemy.t.u16(0x11CC4, rec.reaction as u32) as u8;
        {
            let e = &mut self.enemies.slots[di];
            let hp = (db.hp as i16).wrapping_sub(damage as i16);
            e.b.hp = if hp > 0 { hp as u16 } else { u16::from(nonlethal) };
            if !nonlethal { e.b.stun = e.b.stun.saturating_sub(rec.stun); }
            e.b.reaction = rec.reaction;
            e.b.act = mapped;
            e.b.step = 0;
            e.b.hitstop = rec.hit_stop;
            e.b.b9a = rec.b7;
        }
        self.body_mut(a).hitstop = rec.hit_stop;
        if !hero && !nonlethal && matches!(self.player_move_word(a), 0x202 | 0x203) {
            let p = ab.partner;
            if p != 0 { self.body_mut(p).hitstop = rec.hit_stop; }
        }
        // 0x11C0A: a6 holds the table address 0x11CC4 here (0x11BEC), so 0x962A reads the byte 0x11CE8 as the facing: 0.
        if rec.sound != 0 { self.snd_hit(rec.sound, self.body(a).xi(), 0); }
        self.enemy_hit_score(ab.who, rec.score);
        self.spawn_hit_spark(a, d, ha, hb, rec.spark);
        if hero && counter {
            self.fx_spawn_hit_spark_alt(a, d, ha, hb, 5); // 0x127FC
            self.enemy_hit_score(ab.who, 0x20);
        }
        if db.k20 != 0x7C {
            self.body_mut(d).facing = if hero { ab.facing ^ 1 } else {
                let owner = if (ab.who as i8) >= 0 { Game::player_handle(ab.who as usize) } else { a };
                (self.body(owner).xi() > db.xi()) as u8
            };
        }
        if hero && ab.kind == 6 {
            if db.k20 != 0x7C { self.body_mut(d).facing = (ab.xi() > db.xi()) as u8; }
            if self.w.scripted == 0 {
                if let Some(Slot::Player(i)) = slot_of(a) {
                    let p = &mut self.players[i];
                    let cost = if p.special_budget == 0x14 { 10 } else { 4 };
                    p.special_budget = p.special_budget.wrapping_sub(cost);
                    if (p.special_budget as i16) >= 0 {
                        let hp = (p.b.hp as i16).wrapping_sub(cost as i16);
                        p.b.hp = if hp > 0 { hp as u16 } else { 1 };
                        self.hud_life_bar(i); // 0x16446
                    }
                }
            }
        }
    }

    /// 0x11CE4 dispatch and 0x4F3E8: Hopper guard. RNG is drawn only while angry, grounded
    /// and facing an attacker that faces back. A failed guard clears +$B8.
    fn enemy_monster_guard(&mut self, a: Handle, di: usize) -> bool {
        let e = &self.enemies.slots[di];
        let routine = self.assets.enemy.t.u32(0x11CE4, e.m.b(B87) as u32);
        if routine != 0x4F3E8 { return false; } // 0x11C8A: carry clear
        let ab = self.body(a);
        let left = ab.xi() as u16 <= e.b.xi() as u16;
        let eligible = e.m.b(0xA4) != 0 && e.b.airborne == 0
            && if left { e.b.facing == 0 && ab.facing != 0 } else { e.b.facing != 0 && ab.facing == 0 };
        let blocked = eligible && self.rand() & 0xF >= 14;
        self.enemies.slots[di].m.sb(0xB8, u8::from(blocked));
        blocked
    }

    /// 0x2B904 (hero) / 0x2BA2A (attack object): damage with the direct hero's +$C6 or
    /// the credited owner's +$C6 respectively, and exactly two RNG calls.
    fn enemy_monster_damage(&mut self, base: u32, a: Handle, di: usize, hero: bool) -> (u16, bool) {
        let ab = self.body(a);
        let atk = ab.atk;
        let power = if hero {
            match slot_of(a) { Some(Slot::Player(i)) => Some(self.players[i].power), _ => None }
        } else if (ab.who as i8) >= 0 {
            Some(self.players[((ab.who & 3) % 3) as usize].power)
        } else { None };
        self.enemy_damage_core(base, atk, power, di, hero)
    }

    /// The body of 0x2B904 / 0x2BA2A: `atk` is the attacker's attack id (+$58), `power` its +$C6 (None: no factor).
    pub(super) fn enemy_damage_core(&mut self, base: u32, atk: u8, power: Option<u8>, di: usize, hero: bool) -> (u16, bool) {
        let e = &self.enemies.slots[di];
        let t = &self.assets.enemy.t;
        let mut d = base;
        let mut counter = false;
        let shift_word = |p: u32| (p & 0xFFFF_0000) | ((p as u16 >> 5) as u32);
        if e.b.kind == 4 {
            let factor = t.u16(0xA320C, 2 * e.b.atk as u32) as u32;
            if factor != 0 {
                let product = (d as u16 as u32) * factor;
                d = if hero { shift_word(product) } else { product >> 5 };
                counter = true;
            }
        }
        let w84 = e.m.w(W84) as u32;
        for offset in [0, 2] {
            if t.u8(0xA32FA, w84 + offset).wrapping_mul(2) == atk {
                d = shift_word((d as u16 as u32) * t.u8(0xA32FA, w84 + offset + 1) as u32);
                counter = true; // G.$4F5 also makes the word G.$4F4 nonzero
                break;
            }
        }
        if let Some(power) = power { d = (d as u16 as u32) * power as u32; }
        let index = (w84 as u16).wrapping_shl(3).wrapping_add(self.w.level) as u32;
        d = ((d as u16 as u32) * t.u8(0x2BECE, index) as u32) >> 10;
        if d == 0 { d = 1; }
        let mut hp = e.b.hp;
        let mut row = 0u16;
        while hp > 100 { hp -= 100; row = row.wrapping_add(0x40); }
        let r = self.rand() & 0x3E;
        d = (d as u16 as u32) * self.assets.enemy.t.u16(0x2BC4E, row.wrapping_add(r) as u32) as u32;
        let row: u16 = if hp >= 75 { 0 } else if hp >= 50 { 0x40 } else if hp >= 25 { 0x80 } else { 0xC0 };
        let r = self.rand() & 0x3E;
        d = (d as u16 as u32) * self.assets.enemy.t.u16(0x2BB4E, row.wrapping_add(r) as u32) as u32;
        let damage = (d as u16) >> 10; // lsr.w, including the minimum after truncation
        (damage.max(1), counter)
    }

    /// 0x11CEC / 0x11D32: monster attacks against heroes. Unlike 0x115B0 there is no
    /// attacker hit-stop test. The reaction, damage, drop and power RNG service is shared.
    pub fn pass_11cec(&mut self) {
        for a in self.combat.get(0x6B64).to_vec() {
            if self.body(a).kind != 4 { continue; }
            for d in self.combat.get(0x6718).to_vec() {
                let (ab, db) = (self.body(a), self.body(d));
                if db.live & 2 != 0 || ab.attack_box == 0 || db.body_box == 0 { continue; }
                if matches!(db.kind, 6 | 8 | 0xA) { return; } // clr.l d7, both loops
                let Some(Slot::Player(i)) = slot_of(d) else { continue };
                if ab.serial == self.players[i].hit_serial && db.attacker == a { continue; }
                if collide::boxes_hit(&self.assets.sprites, ab, ab.attack_box, db, db.body_box).is_some() {
                    self.player_hit_by_enemy(i, a);
                }
            }
        }
    }

    /// 0x11E56 / 0x11E9C / 0x11EDC: raw monster damage against ordinary enemies.
    /// Negative +$7D keeps one HP; +$02 = 0xFF prevents score and kill credit.
    pub fn pass_11e56(&mut self) {
        for a in self.combat.get(0x6B64).to_vec() {
            if self.body(a).kind != 4 { return; } // 0x11E6E ends the entire pass
            for d in self.combat.get(0x671E).to_vec() {
                let (ab, db) = (self.body(a).clone(), self.body(d).clone());
                if db.live & 2 != 0 || db.serial == ab.serial || ab.attack_box == 0 || db.body_box == 0
                    || !matches!(db.kind, 0 | 2 | 4 | 6 | 0xC) { continue; }
                let Some((ha, hb)) = collide::boxes_hit(&self.assets.sprites, &ab, ab.attack_box, &db, db.body_box) else { continue };
                let Some(Slot::Enemy(di)) = slot_of(d) else { continue };
                let rec = self.assets.player.enemy_attacks[ab.atk as usize];
                let e = &mut self.enemies.slots[di];
                e.b.serial = ab.serial;
                e.b.live = 2;
                e.b.who = 0xFF;
                e.b.set_state(0x0202_0000);
                e.m.sb(WAIT, 0);
                e.b.attacker = a;
                e.b.hp_prev = db.hp;
                let hp = (db.hp as i16).wrapping_sub(rec.damage as i16);
                e.b.hp = if hp > 0 { hp as u16 } else { u16::from((e.m.b(ATTACKING) as i8) < 0) };
                e.b.stun = e.b.stun.saturating_sub(rec.stun);
                e.b.reaction = rec.reaction;
                e.b.act = rec.reaction;
                e.b.step = 0;
                e.b.hitstop = rec.hit_stop; // the monster's hit-stop is unchanged
                e.b.b9a = rec.b7;
                if rec.sound != 0 { self.snd_hit(rec.sound, self.body(a).xi(), self.body(a).facing); }
                self.spawn_hit_spark(a, d, ha, hb, rec.spark);
                self.body_mut(d).facing = ab.facing ^ 1;
                self.enemy_monster_part_hit(di);
            }
        }
    }

    /// 0x1128E: a hit on a multipart Tyrog piece makes each matching sibling unhittable.
    fn enemy_monster_part_hit(&mut self, di: usize) {
        let e = &self.enemies.slots[di];
        if e.b.b63 != 4 || !matches!(e.b.k20, 0x158 | 0x15C | 0x160 | 0x164) { return; }
        let parent = self.enemy_index(e.m.w(0xB0));
        for (offset, class) in [(0xB0, 0x158), (0xB2, 0x15C), (0xB4, 0x160), (0xB6, 0x164)] {
            let h = self.enemies.slots[parent].m.w(offset);
            if self.body(h).k20 == class { self.body_mut(h).live = 2; }
        }
    }

    /// 0x11962: player attacks (frame class 4 or 6) against the objects of the attacker list 0x671A that
    /// are alive (+$00 bit 0): containers, and weapons lying about, whose frame class can be hit
    /// (0, 2, 4, 6, 0xC). This is how a punch breaks a container.
    pub fn pass_11962(&mut self) {
        for a2 in self.combat.get(0x6718).to_vec() {
            let Some(Slot::Player(i)) = slot_of(a2) else { continue };
            if !matches!(self.players[i].b.kind, 4 | 6) {
                continue;
            }
            for a3 in self.combat.get(0x671A).to_vec() {
                let Some(Slot::Item(j)) = slot_of(a3) else { continue };
                if self.items.slots[j].b.live & 1 == 0 {
                    continue;
                }
                self.player_hits_item(i, a2, j, a3);
            }
        }
    }

    /// 0x119BA / 0x11A06: one player attack against one item. An item that this swing hit already
    /// (same attacker and serial) is skipped. A hit: serial, credited player, attacker, HP minus the
    /// damage byte of the player's attack record, reaction `+$7B` / `+$06`, hit stop on both, the hit
    /// sound, the score, spark type 1, and the item turns away from the player.
    fn player_hits_item(&mut self, i: usize, a2: Handle, j: usize, a3: Handle) {
        let (ab, ib) = (self.players[i].b.clone(), self.items.slots[j].b.clone());
        if ib.attacker == a2 && ab.serial == ib.serial {
            return;
        }
        if ab.attack_box == 0 || ib.body_box == 0 || !matches!(ib.kind, 0 | 2 | 4 | 6 | 0xC) {
            return;
        }
        let Some((ha, hb)) = collide::boxes_hit(&self.assets.sprites, &ab, ab.attack_box, &ib, ib.body_box) else {
            return;
        };
        let ch = (ab.k20 / 4) as usize & 3;
        let r = self.assets.sprites.player_attacks[ch][(ab.atk / 2) as usize];
        {
            let it = &mut self.items.slots[j];
            it.b.serial = ab.serial;
            it.b.who = ab.who;
            it.m.sb(0x7A, 0);
            it.b.attacker = a2;
            it.b.hp_prev = it.b.hp;
            let hp = (it.b.hp as i16).wrapping_sub(r.damage as i16);
            it.b.hp = if hp > 0 { hp as u16 } else { 0 };
            it.b.reaction = r.reaction;
            it.b.act = r.reaction;
            it.b.step = 0;
            it.b.hitstop = r.hit_stop;
        }
        self.players[i].b.hitstop = r.hit_stop;
        if r.sound != 0 {
            let (ax, vf) = (self.body(a2).xi(), self.body(a2).facing);
            self.snd_hit(r.sound, ax, vf); // 0x962A
        }
        self.player_add_score(i, r.score); // 0xFFC8
        self.items.slots[j].b.b9a = r.b7;
        self.spawn_hit_spark(a2, a3, ha, hb, 1);
        let px = ab.xi();
        self.items.slots[j].b.facing = (px > self.items.slots[j].b.xi()) as u8;
    }




    /// 0x1185A: a thrown or knocked down enemy body (list 0x6B30, registered by its flight and fall
    /// states) against the other hurtable enemies (list 0x671E) that are not in a reaction yet.
    pub fn pass_1185a(&mut self) {
        for a2 in self.combat.get(0x6B30).to_vec() {
            for a3 in self.combat.get(0x671E).to_vec() {
                if a3 == a2 || self.body(a3).live & 2 != 0 {
                    continue;
                }
                self.enemy_body_hits_enemy(a2, a3);
            }
        }
    }

    /// 0x1189C: only the frame classes 0, 2 and 4 of the victim count (6, 8 and 0xA return). Body box
    /// against body box (0x128F8). A hit: the victim is hurt (state 0x02020400: sub 2, reaction 4)
    /// with the enemy attack record 0x11 (damage only, stun 0, no HP bar), both get its hit stop,
    /// the victim looks the way the body flies and a spark appears (0x12720, spark 0).
    fn enemy_body_hits_enemy(&mut self, a2: Handle, a3: Handle) {
        let (ab, db) = (self.body(a2).clone(), self.body(a3).clone());
        if ab.body_box == 0 || db.body_box == 0 || !matches!(db.kind, 0 | 2 | 4) {
            return;
        }
        let Some((ha, hb)) = collide::boxes_hit(&self.assets.sprites, &ab, ab.body_box, &db, db.body_box) else { return };
        let rec = self.assets.player.enemy_attacks[0x11];
        {
            let v = self.body_mut(a3);
            v.live = 2;
            v.serial = ab.serial;
            v.who = ab.who;
            v.set_state(0x0202_0000);
            if let Slot::Enemy(i) = slot_of(a3).unwrap() {
                self.enemies.slots[i].m.sb(WAIT, 0);
            }
        }
        let v = self.body_mut(a3);
        v.attacker = a2;
        v.hp_prev = v.hp;
        let hp = (v.hp as i16).wrapping_sub(rec.damage as i16);
        v.hp = if hp > 0 { hp as u16 } else { 0 };
        v.stun = 0;
        v.act = 4;
        v.step = 0;
        v.hitstop = rec.hit_stop;
        v.b9a = rec.spark;
        self.body_mut(a2).hitstop = rec.hit_stop;
        if rec.sound != 0 {
            let (ax, vf) = (self.body(a2).xi(), self.body(a2).facing);
            self.snd_hit(rec.sound, ax, vf); // 0x962A
        }
        self.spawn_hit_spark(a2, a3, ha, hb, 0); // 0x12720 (before the victim turns, 0x11950)
        self.body_mut(a3).facing = (ab.vx < 0) as u8;
    }

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
            let (ax, vf) = (self.body(a).xi(), self.body(a).facing);
            self.snd_hit(r.sound, ax, vf); // 0x962A
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
        self.boss_hit_entry(ei); // 0x3262A and the defeat (stage bosses only)
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
            let (ax, vf) = (self.body(a).xi(), self.body(a).facing);
            self.snd_hit(r.sound, ax, vf); // 0x962A
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

    /// +$A0 word of an attacker: a player's held / previous input, an item's kind word, else 0.
    fn player_move_word(&self, a: Handle) -> u16 {
        match slot_of(a) {
            Some(Slot::Player(i)) => ((self.players[i].held as u16) << 8) | self.players[i].prev as u16,
            Some(Slot::Item(i)) => self.items.slots[i].m.w(0xA0), // an item's kind word
            _ => 0,
        }
    }

    /// Player +$C8 action class of a player attacker.
    fn player_action(&self, a: Handle) -> u8 {
        match slot_of(a) {
            Some(Slot::Player(i)) => self.players[i].action,
            Some(Slot::Item(i)) => self.items.slots[i].m.b(0xC8),
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
    pub(super) fn enemy_hud_request(&mut self, h: Handle, kind: u8) {
        let b = self.body(h).clone();
        self.enemy_hud_hit(h, &b, kind);
    }

    pub(super) fn enemy_hud_hit(&mut self, h: Handle, b: &Body, kind: u8) {
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

    /// 0x606E (also called by the player states, `player_splash_check`): a body that falls into water (+$53 positive, in the air, rising speed
    /// negative, height below 0x20) that has not splashed yet (+$83) and shows no splash (+$25 bit 5) makes the splash effect (pool C class 4) at its X and
    /// depth, sets +$83 and requests the sound 0x9AF4 (the enemy branch: +$82 is set).
    pub(super) fn enemy_splash_check(&mut self, e: &mut Enemy, h: Handle) {
        let b = &e.b;
        if e.m.b(B83) != 0 || b.draw_flags & 0x20 != 0 || (b.wading as i8) <= 0 || b.airborne == 0 {
            return;
        }
        if b.vy >= 0 || b.yi() >= 0x20 {
            return;
        }
        let (x, z, class, facing) = (b.xi(), b.zi(), b.k20, b.facing);
        if self.fx_spawn_water_splash_at(h, x, z, class) {
            e.m.sb(B83, 1);
            self.snd_splash_object(x, facing);
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
                    self.snd_enemy_death(e.b.b63, e.b.xi(), e.b.facing); // 0x2A748 / 0x2A752 (type 6: 0x2A8D2 / 0x2A8DC)
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
                self.enemy_splash_check(e, h); // 0x2A784 / 0x2A90E: jsr 0x606E
                self.combat.push(0x6AC2, h);
                e.b.motion();
                self.enemy_screen_bounce(e);
                if e.b.param_a == 0 {
                    self.enemy_step(&mut e.b);
                }
                self.enemy_floor_collision(e);
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
                    self.snd_9796(e.b.xi()); // 0x2A7E8 (type 6: 0x2A972)
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
                self.enemy_floor_collision(e);
                if e.b.airborne == 0 {
                    self.enemy_lie_time(e);
                    if e.b.hp == 0 {
                        e.m.sb(T80, 1);
                    }
                    self.snd_9796(e.b.xi()); // 0x2A832
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
                    self.snd_enemy_death(e.b.b63, e.b.xi(), e.b.facing); // 0x2AAD6 / 0x2AAE0
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
                self.enemy_floor_collision(e);
                if e.b.airborne == 0 {
                    self.enemy_lie_time(e);
                    if e.b.hp == 0 {
                        e.m.sb(T80, 0xA);
                        self.item_drop_from_enemy(e);
                    }
                    self.snd_9796(e.b.xi()); // 0x2ABE4
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
        self.enemy_splash_check(e, h); // 0x2AB00: jsr 0x606E
        self.combat.push(0x6AC2, h);
        e.b.motion();
        self.enemy_screen_bounce(e);
        if e.b.param_a == 0 {
            self.enemy_step(&mut e.b);
        }
        self.enemy_floor_collision(e);
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
            self.snd_9796(e.b.xi()); // 0x2AB7C
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
                    // Charred death 0x2AC42: the pieces (0x93D48), the body is not drawn any more (clr.b $25) and shows the charred frame 0x2AC1C (0x1236),
                    // then 0x9A84 (the voice of the killer, +$02 of the enemy: `snd_9a84`).
                    let b = e.b.clone();
                    self.fx_spawn_burn_pieces(&b);
                    e.b.draw_flags = 0;
                    let id = self.assets.sprites.frame_at(0x2AC1C);
                    e.b.load_frame(&self.assets.sprites, id);
                    self.snd_9a84(e.b.who); // 0x2AC56
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
                        self.snd_97d2(e.b.xi(), e.b.facing); // 0x2ACE0: jsr 0x97D2 (the original drops the item twice around it)
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
                // The step table 0x2AC14 sends step 4 to the shared landing block 0x2A7F6.
                self.combat.push(0x6AC2, h);
                e.b.motion();
                self.enemy_screen_bounce(e);
                if e.b.param_a == 0 {
                    self.enemy_step(&mut e.b);
                }
                self.enemy_floor_collision(e);
                if e.b.airborne == 0 {
                    self.enemy_lie_time(e);
                    if e.b.hp == 0 {
                        e.m.sb(T80, 1);
                    }
                    self.snd_9796(e.b.xi()); // 0x2A832
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
        self.enemy_splash_check(e, h); // 0x2AD1A: jsr 0x606E
        self.combat.push(0x6AC2, h);
        e.b.motion();
        self.enemy_screen_bounce(e);
        if e.b.param_a == 0 {
            self.enemy_step(&mut e.b);
        }
        self.enemy_floor_collision(e);
        if e.b.airborne == 0 {
            e.b.palette_override = 0;
            e.b.airborne = 1;
            let a = if e.b.hp != 0 { 0x2CBD4 } else { 0x2CBDC };
            let t = &self.assets.enemy.t;
            e.b.vy = t.i16(a, 0);
            let ay = t.i16(a, 2);
            e.b.ay = ay.wrapping_add(ay);
            e.b.step += 2;
            self.snd_9796(e.b.xi()); // 0x2AD74
            self.enemy_step(&mut e.b);
        }
    }

    /// 0x2AD98: flames on a burning enemy (pool C class 0x20, list 0x2CA36 by +$84 / 2).
    /// 0x2AD98 / 0x2ADAE: the list of the enemy (a word offset table 0x2CA36 indexed by +$84 / 2 bytes = the name id): a count, then (X, Y, variant) of each flame; one
    /// pool C class 8 object per entry (owner +$60, +$62 / +$64 the offset, +$26 the variant, +$5C its index). Stops when pool C is full.
    pub(crate) fn enemy_burn_flames(&mut self, h: Handle, e: &Enemy) {
        let t = &self.assets.enemy.t;
        let a = 0x2CA36u32.wrapping_add(t.u16(0x2CA36, (e.m.w(W84) >> 1) as u32) as i16 as i32 as u32);
        let count = t.u16(a, 0);
        for k in 0..count {
            let (x, y, var) = (t.i16(a, 2 + 6 * k as u32), t.i16(a, 4 + 6 * k as u32), t.u16(a, 6 + 6 * k as u32));
            if !self.fx_spawn_flame(h, x, y, var, k) {
                return;
            }
        }
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
            self.fx_spawn_dust(&e.b, 2); // 0x2AE8E / 0x2AF62: dust 0x92E5E with d0 = 2
        }
        self.enemy_floor_collision(e);
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
        // 0x2B23A: d0 = the word +$80 (0x5A08), then 0x94338: the stars over the enemy.
        let b = e.b.clone();
        self.fx_spawn_dizzy(h, &b, (0x5A << 8) | 8);
        self.enemy_getup(e, h);
    }

    /// 0x2B2B8: wall slump (type 0x1C).
    fn enemy_slump_start(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0202_1C00);
        e.m.sb(T80, 0x1E);
        self.enemy_hit_anim(e, 6);
        e.b.facing = (e.b.vx < 0) as u8;
        self.snd_9796(e.b.xi()); // 0x2B2E0
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

    /// 0x2B35A: held by a player (+$70), sub 6. `act` 0: ask the HUD for the vitality bar of the
    /// holder (0x16B36, kind 1) and continue as act 2; act 2: while the holder is still in its
    /// grabbing state (mode 2, sub 4, frame class 0xC) put the enemy into its hand; act 4: only the
    /// check. A knee sets `act` to the reaction of the attack record (0, 2): the bar request again.
    /// Otherwise the grab is over: floor snap, reaction 0x20 and back to the AI (0x2B54E).
    pub fn enemy_grabbed(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => {
                // 0x2B36C: a holder with +$82 == 4 (not a player) skips the bar and the placing.
                if self.enemy_holder_tag(e.b.partner) == 4 {
                    e.b.act = 4;
                    return;
                }
                let b = e.b.clone();
                self.enemy_hud_hit(h, &b, 1); // 0x16B36 with d0 = 1
                e.b.act += 2;
                e.b.b62 = 0;
                self.enemy_grabbed_place(e, h, true);
            }
            2 => {
                e.b.b62 = 0;
                self.enemy_grabbed_place(e, h, true);
            }
            4 => {
                e.b.b62 = 0;
                self.enemy_grabbed_place(e, h, false);
            }
            // The 68000 jumps through the word table 0x2B366 with any value; the attack records of
            // the grab give 0 and 2.
            _ => {}
        }
    }

    /// +$82 of the holder: players (and anything that is not a pool A / E object) read 0.
    pub(super) fn enemy_holder_tag(&self, p: Handle) -> u8 {
        match slot_of(p) {
            Some(Slot::Enemy(i)) => self.enemies.slots[i].m.b(0x82),
            Some(Slot::Item(i)) => self.items.slots[i].m.b(0x82),
            _ => 0,
        }
    }

    /// 0x2B392 / 0x2B434: the holder must be alive, in mode 2 / sub 4 with the frame class
    /// 0xC, else the grab ends (0x2B464). `place`: act 2 also puts the enemy at the hand.
    fn enemy_grabbed_place(&mut self, e: &mut Enemy, h: Handle, place: bool) {
        let holder = e.b.partner;
        let ok = slot_of(holder).is_some() && {
            let p = self.body(holder);
            p.live != 0 && p.mode == 2 && p.sub == 4 && p.kind == 0xC
        };
        if !ok {
            // 0x2B464: released (a missing holder reads the ROM, whose first byte is 0).
            e.b.wall = 0;
            self.enemy_snap_to_floor(e);
            e.b.reaction = 0x20;
            return self.enemy_recover(e, h);
        }
        if !place {
            return;
        }
        let p = self.body(holder).clone();
        // The high byte of the holder's attach word (+$60) is the row index, negative = no row.
        let attach = (p.attach >> 8) as u8;
        if (attach as i8) >= 0 {
            return;
        }
        let t = &self.assets.enemy.t;
        let base = t.u32(0x2D0E0, p.k20 as u32);
        let off = t.i16(base, (e.m.w(W84) >> 1) as u32);
        let row = base.wrapping_add(off as i32 as u32).wrapping_add(((attach & 0x3F) as u32) << 3);
        let (mut dx, dy, anim, mut facing) = (t.i16(row, 0), t.i16(row, 2), t.u16(row, 4), t.u8(row, 6));
        if p.facing != 0 {
            dx = dx.wrapping_neg();
            facing ^= 1;
        }
        // Word moves: the fractions of the enemy's position stay.
        let x = p.xi().wrapping_add(dx);
        let y = p.yi().wrapping_add(dy);
        e.b.x = (e.b.x & 0xFFFF) | ((x as i32) << 16);
        e.b.y = (e.b.y & 0xFFFF) | ((y as i32) << 16);
        e.b.z = (e.b.z & 0xFFFF) | ((p.zi() as i32) << 16);
        e.b.facing = facing;
        let table = self.assets.enemy.t.u32(0x2E724, e.m.w(W84) as u32);
        let first = self.assets.sprites.anim(table, anim);
        e.b.load_frame(&self.assets.sprites, first); // 0x1236
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::player::PlayerData;
    use dino_assets::{AnimTable, AssetPack, AttackRecord, Frame, FrameDraw, HitBox, HitBoxBank};

    fn frame(kind: u8, body_box: u8) -> Frame {
        Frame {
            rom_addr: 0,
            stride: 10,
            w0: 0,
            w1: 0x8000,
            draw: FrameDraw::Comp { main: 0, overlay: None },
            param_a: 0,
            param_b: 0,
            kind,
            duration: 5,
            last: true,
            body_box,
            attack_box: 0,
            attach: 0,
            extra: [0; 4],
            next: 0,
        }
    }

    /// Placement tables for 0x2B35A: character 0 (+$20 = 0) has its word table at 0x2D0F0; the
    /// enemy type 0 (+$84 = 0) finds its rows at 0x2D100; row 1 is (dx 0x20, dy 0x30, animation 1,
    /// facing 1). The grabbed animations of type 0 are the table 0x30000.
    fn game() -> Game {
        let mut a = AssetPack::default();
        let mut placement = vec![0u8; 0x40];
        placement[..4].copy_from_slice(&0x2D0F0u32.to_be_bytes());
        placement[0x10..0x12].copy_from_slice(&0x0010u16.to_be_bytes()); // type 0: rows at 0x2D100
        let row = 0x20 + 8; // row 1
        placement[row..row + 8].copy_from_slice(&[0, 0x20, 0, 0x30, 0, 1, 1, 0]);
        a.enemy.t.blobs = vec![(0x2D0E0, placement), (0x2E724, 0x30000u32.to_be_bytes().to_vec())];
        a.sprites.frames = vec![frame(0xE, 0)];
        a.sprites.anim_tables.insert(0x30000, AnimTable { rom_addr: 0x30000, stride: 10, anims: vec![None, Some(0)] });
        Game::new(Box::leak(Box::new(a)))
    }

    /// Player 0 in the grabbing state with attach row 1 (high byte 0x81).
    fn holder(g: &mut Game, facing: u8) {
        let p = &mut g.players[0].b;
        p.live = 1;
        p.set_state(0x0204_0002);
        p.kind = 0xC;
        p.attach = 0x8100;
        p.facing = facing;
        p.set_xi(100);
        p.set_yi(5);
        p.set_zi(60);
    }

    fn held_enemy(g: &Game) -> Enemy {
        let mut e = Enemy::default();
        e.b.live = 1;
        e.b.partner = Game::player_handle(0);
        e.b.x = 0x1234_5678;
        e.b.y = 0x0001_8000;
        e.b.z = 0x0002_4000;
        e.b.set_state(0x0206_0000);
        let _ = g;
        e
    }

    #[test]
    fn the_held_enemy_is_put_into_the_hand_by_the_attach_row() {
        let mut g = game();
        holder(&mut g, 0);
        let h = g.enemies.handle(0);
        let mut e = held_enemy(&g);
        e.b.act = 2;
        g.enemy_grabbed(&mut e, h);
        // facing 0: dx as it is, the facing byte of the row; the fractions of X, Y, Z stay.
        assert_eq!((e.b.xi(), e.b.x & 0xFFFF), (100 + 0x20, 0x5678));
        assert_eq!((e.b.yi(), e.b.y & 0xFFFF), (5 + 0x30, 0x8000));
        assert_eq!((e.b.zi(), e.b.z & 0xFFFF), (60, 0x4000));
        assert_eq!(e.b.facing, 1);
        assert_eq!(e.b.kind, 0xE, "the grabbed frame (the holder checks the frame class)");
        // The holder faces right (+$24 != 0): dx negated, facing bit flipped.
        holder(&mut g, 1);
        g.enemy_grabbed(&mut e, h);
        assert_eq!(e.b.xi(), 100 - 0x20);
        assert_eq!(e.b.facing, 0);
    }

    #[test]
    fn act_0_asks_the_hud_for_the_bar_and_continues_as_act_2() {
        let mut g = game();
        holder(&mut g, 0);
        let h = g.enemies.handle(0);
        let mut e = held_enemy(&g);
        e.b.act = 0;
        e.b.b62 = 7;
        e.b.who = 0;
        e.b.id = 3;
        e.b.hp = 0x20;
        e.b.hp_bar = 0x64;
        e.b.hp_prev = 0x30;
        g.hud.rings[0][0][0] = 0xFFFF; // a free ring slot
        g.enemy_grabbed(&mut e, h);
        assert_eq!((e.b.act, e.b.b62), (2, 0));
        assert_eq!(g.hud.ring_w[0], 0x10, "one message queued");
        assert_eq!(g.hud.rings[0][0][..4], [0x0103, 0x64, 0x30, 0x20], "kind 1, name, bar, HP before, HP");
        assert_eq!(e.b.xi(), 100 + 0x20, "act 0 places the enemy in the same frame");
    }

    #[test]
    fn a_holder_that_is_not_grabbing_or_a_row_without_attach_leaves_the_enemy_alone() {
        let mut g = game();
        holder(&mut g, 0);
        let h = g.enemies.handle(0);
        // Act 4 only checks. Row index negative = no row: act 2 returns too.
        let mut e = held_enemy(&g);
        e.b.act = 4;
        g.enemy_grabbed(&mut e, h);
        assert_eq!((e.b.xi(), e.b.act, e.b.sub), (0x1234, 4, 6));
        e.b.act = 2;
        g.players[0].b.attach = 0x0100;
        g.enemy_grabbed(&mut e, h);
        assert_eq!(e.b.xi(), 0x1234);
    }

    #[test]
    fn the_holder_tag_is_zero_for_players() {
        let mut g = game();
        g.enemies.slots[3].m.sb(0x82, 4);
        assert_eq!(g.enemy_holder_tag(Game::player_handle(0)), 0);
        assert_eq!(g.enemy_holder_tag(g.enemies.handle(3)), 4);
        assert_eq!(g.enemy_holder_tag(0), 0);
    }

    /// 0x1189C: a thrown body (list 0x6B30) hurts an enemy of frame class 0 / 2 / 4 whose body box
    /// it touches: reaction state 0x02020400, HP - record damage, stun 0, both stop for the
    /// record's hit stop, the victim faces the way the body flies.
    #[test]
    fn a_thrown_body_hits_another_enemy() {
        let mut a = AssetPack::default();
        a.sprites.hitboxes = HitBoxBank { base: 0x100000, words: vec![0, 0, 0, 0, 0, 0, 0, 0x20, -0x10, 0x20, 0, 0x40] }; // box 1
        let mut player = PlayerData::default();
        player.enemy_attacks = vec![AttackRecord::default(); 0x12];
        player.enemy_attacks[0x11] = AttackRecord { damage: 5, stun: 9, reaction: 9, hit_stop: 2, sound: 0, score: 0, spark: 3, b7: 0 };
        a.player = player;
        let mut g = Game::new(Box::leak(Box::new(a)));
        let (thrown, victim) = (g.enemies.handle(1), g.enemies.handle(2));
        for (h, x) in [(thrown, 0x100), (victim, 0x108)] {
            let b = g.body_mut(h);
            b.live = 1;
            b.box_table = 0x100000;
            b.body_box = 1;
            b.set_xi(x);
            b.set_zi(0x50);
        }
        {
            let t = g.body_mut(thrown);
            t.who = 1;
            t.serial = 9;
            t.vx = -0x80;
            t.kind = 4;
        }
        {
            let v = g.body_mut(victim);
            v.kind = 2;
            v.hp = 0x40;
            v.stun = 10;
            v.act = 0xA;
            v.step = 6;
        }
        g.combat.push(0x6B30, thrown);
        g.combat.push(0x671E, thrown);
        g.combat.push(0x671E, victim);
        g.pass_1185a();
        let v = g.body(victim);
        assert_eq!(v.state(), 0x0202_0400);
        assert_eq!((v.live, v.hp, v.hp_prev, v.stun, v.hitstop), (2, 0x3B, 0x40, 0, 2));
        assert_eq!((v.who, v.serial, v.attacker, v.facing, v.b9a), (1, 9, thrown, 1, 3));
        assert_eq!(g.body(thrown).hitstop, 2);
        assert_eq!(g.effects.free_count(), 47, "a spark");
        // A victim already in a reaction (live bit 1) and one of frame class 6 are left alone.
        g.body_mut(victim).live = 2;
        g.body_mut(victim).hp = 0x40;
        g.pass_1185a();
        assert_eq!(g.body(victim).hp, 0x40);
        g.body_mut(victim).live = 1;
        g.body_mut(victim).kind = 6;
        g.pass_1185a();
        assert_eq!(g.body(victim).hp, 0x40);
    }

    /// 0x11962: a player attack (frame class 4) hits an item of the attacker list 0x671A (a container) with the
    /// damage of the player's attack record; the same attacker with the same serial hits it only once; the
    /// frame classes that cannot be hit (8, 0xA, 0xE) and the player's frame classes other than 4 / 6 do nothing.
    #[test]
    fn a_player_attack_hits_an_item_of_the_attacker_list() {
        let mut a = AssetPack::default();
        a.sprites.hitboxes = HitBoxBank { base: 0x100000, words: vec![0, 0, 0, 0, 0, 0, 0, 0x20, -0x10, 0x20, 0, 0x40] }; // box 1
        a.sprites.player_attacks[0] = vec![AttackRecord { damage: 4, stun: 1, reaction: 2, hit_stop: 9, sound: 0, score: 0, spark: 1, b7: 5 }; 2];
        a.enemy.t.blobs = vec![(0x10292, vec![0; 0x90])];
        let mut g = Game::new(Box::leak(Box::new(a)));
        let (me, item) = (Game::player_handle(0), g.items.handle(3));
        {
            let p = &mut g.players[0].b;
            p.live = 1;
            p.box_table = 0x100000;
            p.attack_box = 1;
            p.kind = 4;
            p.serial = 5;
            p.who = 0;
            p.set_xi(0x100);
            p.set_zi(0x50);
        }
        {
            let it = &mut g.items.slots[3].b;
            it.live = 1;
            it.box_table = 0x100000;
            it.body_box = 1;
            it.hp = 4;
            it.set_xi(0x108);
            it.set_zi(0x50);
        }
        g.combat.push(0x6718, me);
        g.combat.push(0x671A, item);
        g.pass_11962();
        let it = &g.items.slots[3].b;
        assert_eq!((it.hp, it.hp_prev, it.hitstop, it.serial, it.attacker), (0, 4, 9, 5, me));
        assert_eq!((it.act, it.reaction, it.b9a, it.who, it.facing), (2, 2, 5, 0, 0));
        assert_eq!(g.players[0].b.hitstop, 9);
        assert_eq!(g.effects.free_count(), 47, "a spark");
        // The same swing does not hit twice.
        g.items.slots[3].b.hp = 4;
        g.pass_11962();
        assert_eq!(g.items.slots[3].b.hp, 4);
        // A new swing (new serial) does; a frame class that cannot be hit does not.
        g.players[0].b.serial = 6;
        g.items.slots[3].b.kind = 8;
        g.pass_11962();
        assert_eq!(g.items.slots[3].b.hp, 4);
        g.items.slots[3].b.kind = 0xC;
        g.pass_11962();
        assert_eq!(g.items.slots[3].b.hp, 0);
        // The player's frame class 0 is not an attack.
        g.items.slots[3].b.hp = 4;
        g.players[0].b.serial = 7;
        g.players[0].b.kind = 0;
        g.pass_11962();
        assert_eq!(g.items.slots[3].b.hp, 4);
    }
}

#[cfg(test)]
mod monster_tests {
    use super::*;
    use dino_assets::{AssetPack, AttackRecord, HitBoxBank};

    fn game() -> Game {
        // Synthetic tables deliberately use different values from the game data.
        let mut a = AssetPack::default();
        a.sprites.hitboxes = HitBoxBank { base: 0x100000, words: vec![0; 12] };
        a.player.enemy_attacks = vec![AttackRecord {
            damage: 7, stun: 3, reaction: 4, hit_stop: 5, b7: 9, ..Default::default()
        }];
        let words = |value: u16, count| (0..count).flat_map(|_| value.to_be_bytes()).collect();
        a.enemy.t.blobs = vec![
            (0xA320C, vec![0; 8]), (0xA32FA, vec![0x7F; 8]),
            (0x2BECE, vec![32; 64]), (0x2BC4E, words(32, 64)),
            (0x2BB4E, words(32, 128)),
            (0x11CC4, words(2, 16)), (0x10292, vec![0; 8]),
            (0x11CE4, [vec![0; 4], 0x4F3E8u32.to_be_bytes().to_vec()].concat()),
        ];
        let mut attacks = [AttackRecord::default(); 1];
        attacks[0] = AttackRecord { damage: 9, stun: 2, reaction: 4, hit_stop: 5, ..Default::default() };
        a.sprites.player_attacks = std::array::from_fn(|_| attacks.to_vec());
        Game::new(Box::leak(Box::new(a)))
    }

    fn pair(g: &mut Game) -> (Handle, Handle) {
        let a = g.enemies.handle(0);
        let d = g.enemies.handle(1);
        for h in [a, d] {
            let b = g.body_mut(h);
            b.live = 1;
            b.body_box = 1;
            b.attack_box = 1;
            b.box_table = 0x100000;
            b.hp = 20;
            b.stun = 10;
        }
        g.body_mut(a).kind = 4;
        g.body_mut(a).serial = 3;
        g.enemy_monster_register(a);
        g.combat.push(0x671E, d);
        (a, d)
    }

    #[test]
    fn monster_raw_damage_has_no_score_or_rng_and_does_not_stop_attacker() {
        let mut g = game();
        let (a, d) = pair(&mut g);
        g.body_mut(a).hitstop = 11;
        let rng = g.w.rng;
        let score = g.players.iter().map(|p| p.score).collect::<Vec<_>>();
        g.pass_11e56();
        let b = g.body(d);
        assert_eq!((b.hp, b.hp_prev, b.stun, b.who, b.attacker), (13, 20, 7, 0xFF, a));
        assert_eq!((b.state(), b.hitstop, b.b9a, b.facing), (0x0202_0400, 5, 9, 1));
        assert_eq!(g.body(a).hitstop, 11);
        assert_eq!(g.w.rng, rng);
        assert_eq!(g.players.iter().map(|p| p.score).collect::<Vec<_>>(), score);
        g.pass_11e56();
        assert_eq!(g.body(d).hp, 13, "one hit per serial / live bit");
    }

    #[test]
    fn monster_damage_floors_negative_census_tag_at_one_hp_only() {
        for tag in [0, 1, 0x7F, 0x80, 0xFF] {
            let mut g = game();
            let (_, d) = pair(&mut g);
            let di = g.enemy_index(d);
            g.enemies.slots[di].m.sb(ATTACKING, tag);
            g.body_mut(d).hp = 7;
            g.pass_11e56();
            assert_eq!(g.body(d).hp, u16::from((tag as i8) < 0), "tag {tag}");
        }
    }

    #[test]
    fn first_nonattacking_monster_ends_raw_pass() {
        let mut g = game();
        let (_, d) = pair(&mut g);
        let idle = g.enemies.handle(2);
        g.body_mut(idle).kind = 0;
        g.enemy_monster_register(idle);
        g.pass_11e56();
        assert_eq!(g.body(d).hp, 20);
    }

    #[test]
    fn attack_object_damage_uses_credited_owner_power_and_two_rng_calls() {
        let mut g = game();
        let (a, d) = pair(&mut g);
        let di = g.enemy_index(d);
        g.body_mut(a).who = 1;
        g.players[0].power = 3;
        g.players[1].power = 48;
        g.w.rng = 123;
        g.rand(); g.rand();
        let expected_rng = g.w.rng;
        g.w.rng = 123;
        assert_eq!(g.enemy_monster_damage(10, a, di, false).0, 15);
        assert_eq!(g.w.rng, expected_rng);
        g.body_mut(a).who = 0xFF;
        assert_eq!(g.enemy_monster_damage(10, a, di, false).0, 1);
    }

    #[test]
    fn both_monster_passes_share_the_serial_and_map_reactions() {
        let mut g = game();
        let (_, d) = pair(&mut g);
        g.combat.clear(0x6B64);
        g.enemy_monster_register(d);
        let a = Game::player_handle(0);
        let b = &mut g.players[0].b;
        b.live = 1; b.kind = 4; b.attack_box = 1; b.box_table = 0x100000; b.serial = 5;
        g.players[0].power = 32;
        g.combat.push(0x671A, a);
        g.combat.push(0x6718, a);
        g.pass_1132a();
        assert_eq!((g.body(d).hp, g.body(d).reaction, g.body(d).act), (11, 4, 2));
        let rng = g.w.rng;
        g.pass_11a9e();
        assert_eq!((g.body(d).hp, g.w.rng), (11, rng));
    }

    #[test]
    fn hopper_guard_skips_rng_until_angry_grounded_and_facing_attacker() {
        let mut g = game();
        let (a, d) = pair(&mut g);
        let di = g.enemy_index(d);
        g.enemies.slots[di].m.sb(B87, 4);
        let rng = g.w.rng;
        assert!(!g.enemy_monster_guard(a, di));
        assert_eq!(g.w.rng, rng);
        g.enemies.slots[di].m.sb(0xA4, 1);
        g.body_mut(a).facing = 1;
        g.body_mut(d).airborne = 1;
        assert!(!g.enemy_monster_guard(a, di));
        assert_eq!(g.w.rng, rng);
        g.body_mut(d).airborne = 0;
        g.rand(); let next = g.w.rng; g.w.rng = rng;
        g.enemy_monster_guard(a, di);
        assert_eq!(g.w.rng, next);
    }

    #[test]
    fn freeing_a_slot_keeps_name_scale_and_leave_x() {
        let mut g = game();
        let i = g.enemies.alloc().unwrap();
        let mut e = std::mem::take(&mut g.enemies.slots[i]);
        e.m.sw(W84, 0x24); e.m.sw(W5C, 0x1234);
        e.b.live = 1; e.m.sb(B87, 4);
        g.enemy_free_pool_a(&mut e, i);
        g.enemies.slots[i] = e;
        assert_eq!(g.enemies.alloc(), Some(i));
        let e = &g.enemies.slots[i];
        assert_eq!((e.m.w(W84), e.m.w(W5C), e.m.b(B87), e.b.live), (0x24, 0x1234, 0, 0));
    }

    #[test]
    fn sleeping_z_spawn_keeps_fractions_and_saves_the_owner_state() {
        let mut g = game();
        let i = g.effects.slots.len() - 1;
        let b = &mut g.effects.slots[i].b;
        b.x = 0x1234; b.y = 0x5678; b.z = 0x4321; b.var26 = 0xABCD;
        let mut e = Enemy::default();
        e.b.set_xi(17); e.b.set_yi(23); e.b.set_zi(31);
        e.b.set_state(0x0200_0406);
        let h = g.enemies.handle(3);
        let rng = g.w.rng;
        g.enemy_monster_spawn_z(&e, h, 3);
        let b = &g.effects.slots[i].b;
        assert_eq!((b.x, b.y, b.z), (0x111234, 0x175678, 0x1F4321));
        assert_eq!((b.k20, b.var26, b.attacker, b.hp_bar, b.hp, b.hp_prev),
            (0x78, 0x03CD, 0xFFFF, h, 0x0200, 0x0406));
        assert_eq!(g.w.rng, rng);
    }

    #[test]
    fn hunter_poke_preserves_hp_presence_and_rng() {
        let mut g = game();
        let (a, d) = pair(&mut g);
        let mut hunter = Enemy::default();
        hunter.m.sw(TARGET, d);
        let rng = g.w.rng;
        g.enemy_hunter_poke(&hunter, a);
        let b = g.body(d);
        assert_eq!((b.hp, b.live, b.state(), b.who, b.hitstop, b.attacker), (20, 1, 0x0202_0000, 0xFF, 8, a));
        assert_eq!(g.w.rng, rng);
    }
}

#[cfg(test)]
mod monster_mame_tests {
    use super::*;
    use crate::harness::TraceReader;
    use dino_assets::AssetPack;

    fn assets() -> Option<&'static AssetPack> {
        static A: std::sync::OnceLock<Option<&'static AssetPack>> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let path = std::path::Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("ROM loads"))))
        })
    }

    fn word(ram: &[u8], o: usize) -> u16 { u16::from_be_bytes([ram[o], ram[o + 1]]) }
    fn enemy(ram: &[u8], o: usize) -> Enemy {
        let mut e = Enemy::default();
        e.m.0.copy_from_slice(&ram[o..o + 0xE0]);
        e.b.k20 = word(ram, o + 0x20);
        e.b.set_state(u32::from_be_bytes(ram[o + 4..o + 8].try_into().unwrap()));
        e.b.palette_override = ram[o + 0x23];
        e
    }

    /// The flash helper alone, frame by frame from recorded inputs. The dinosaur AI is
    /// deliberately outside this service test and remains the later class task's gate.
    #[test]
    fn monster_flash_services_match_mame_recordings() {
        let Some(a) = assets() else { return };
        let mut checked = 0;
        for (name, class, mode_offset, mode) in [
            ("m700_a13_h3_hit_replay", 29, 0xA8, 0xC),
            ("m600_a17_s3_hunter_replay", 24, 0xA8, 8),
            ("m600_a10_m_kill_replay", 38, 0xA0, 4),
        ] {
            let path = format!("../../../re/out/traces/{name}.mame.trc");
            if !std::path::Path::new(&path).exists() { eprintln!("missing {path}, skipped"); continue; }
            let mut rd = TraceReader::open(&path).unwrap();
            let mut previous: Option<Vec<u8>> = None;
            let mut n = 0;
            let mut g = Game::new(a);
            while let Some(frame) = rd.next() {
                let ram = &rd.region("ram").unwrap().data;
                if let Some(prev) = &previous {
                    g.w.stage = ram[0x84D9];
                    for o in (0xC8F4..0xDDF4).step_by(0xE0) {
                        if ram[o] == 0 || word(ram, o + 0x20) / 4 != class
                            || word(prev, o + mode_offset) != mode || word(ram, o + mode_offset) != mode
                            || prev[o + 5] != 0 || ram[o + 5] != 0
                            || (class == 29 && (prev[o + 0x59] != 0 || prev[o + 6] != 2 || ram[o + 6] != 2)) { continue; }
                        let mut e = enemy(prev, o);
                        g.enemy_monster_flash_tick(&mut e);
                        assert_eq!((e.b.palette_override, &e.m.0[0xA5..0xA8]),
                            (ram[o + 0x23], &ram[o + 0xA5..o + 0xA8]), "{name} frame {frame}, slot {o:X}");
                        n += 1;
                    }
                }
                previous = Some(ram.clone());
            }
            assert!(n > 50, "{name}: actually exercises flashing");
            eprintln!("{name}: flash equal for {n} frames");
            checked += n;
        }
        assert!(checked > 0, "needs monster recordings");
    }

    /// Replay the raw hit service at every measured hit, using the recorded collision
    /// frame's boxes and the HP saved by MAME before subtraction. No AI or RNG injection.
    #[test]
    fn monster_raw_hits_match_mame_recordings() {
        let Some(a) = assets() else { return };
        for name in ["m600_a11_m_vs_thug", "m600_a17_s3_hunter_replay", "m600_a21_s3_vs_enemy"] {
            let path = format!("../../../re/out/traces/{name}.mame.trc");
            if !std::path::Path::new(&path).exists() { eprintln!("missing {path}, skipped"); continue; }
            let mut rd = TraceReader::open(&path).unwrap();
            let mut previous: Option<Vec<u8>> = None;
            let mut n = 0;
            let mut g = Game::new(a);
            while let Some(frame) = rd.next() {
                let ram = &rd.region("ram").unwrap().data;
                if let Some(prev) = &previous {
                    for o in (0xC8F4..0xDDF4).step_by(0xE0) {
                        let attacker = word(ram, o + 0x68) as usize;
                        if ram[o] != 2 || ram[o + 2] != 0xFF || word(prev, o + 0x6C) == word(ram, o + 0x6C)
                            || !(0xC8F4..0xDDF4).contains(&attacker)
                            || !matches!(word(ram, attacker + 0x20) / 4, 22 | 24 | 29 | 38 | 82) { continue; }
                        g.trace_load(ram, &rd.region("gfx").unwrap().data);
                        g.combat_lists_reset();
                        g.enemy_monster_register(attacker as u16);
                        g.combat.push(0x671E, o as u16);
                        let b = g.body_mut(o as u16);
                        b.live = 1;
                        b.hp = word(ram, o + 0x6E);
                        b.serial = prev[o + 0x5E];
                        b.stun = prev[o + 0x78];
                        g.pass_11e56();
                        let b = g.body(o as u16);
                        assert_eq!((b.hp, b.hp_prev, b.stun, b.who, b.reaction, b.act, b.hitstop, b.facing, b.b9a),
                            (word(ram, o + 0x6C), word(ram, o + 0x6E), ram[o + 0x78], ram[o + 2],
                             ram[o + 0x7B], ram[o + 6], ram[o + 0x59], ram[o + 0x24], ram[o + 0x9A]),
                            "{name} frame {frame}, slot {o:X}");
                        n += 1;
                    }
                }
                previous = Some(ram.clone());
            }
            assert!(n > 0, "{name}: actually exercises raw damage");
            eprintln!("{name}: all {n} raw hits equal MAME");
        }
    }
}

#[cfg(test)]
mod attack_object_hero_tests {
    use super::*;
    use dino_assets::{AssetPack, AttackRecord, HitBoxBank};

    fn pair() -> (Game, Handle, Handle) {
        let mut assets = AssetPack::default();
        // Synthetic boxes and records, unrelated to the ROM's contents.
        assets.sprites.hitboxes = HitBoxBank { base: 0x100000, words: vec![0, 0, 0, 0, 0, 0, -2, 4, 0, 0, 0, 0] };
        assets.player.power = vec![37; 128];
        assets.sprites.player_attacks = std::array::from_fn(|_| vec![AttackRecord {
            damage: 7, stun: 3, reaction: 4, hit_stop: 5, b7: 9, ..Default::default()
        }; 0x38]);
        let mut g = Game::new(Box::leak(Box::new(assets)));
        let a = g.items.handle(0);
        let d = Game::player_handle(1);
        for h in [a, d] {
            let b = g.body_mut(h);
            b.live = 1;
            b.box_table = 0x100000;
            b.body_box = 1;
            b.attack_box = 1;
            b.hp = 10;
            b.stun = 2;
        }
        g.body_mut(a).kind = 4;
        g.body_mut(a).serial = 3;
        g.body_mut(a).knock = 6;
        g.body_mut(d).who = 1;
        g.players[1].item_type = 0x204;
        g.players[1].combo = 8;
        g.combat.push(0x671A, a);
        g.combat.push(0x6718, d);
        (g, a, d)
    }

    #[test]
    fn friendly_objects_floor_hp_drop_every_heros_weapon_and_use_one_rng_call() {
        for id in 0..4 {
            for hp in [0, 1, 2, 100, 0x8000] {
                let (mut g, a, d) = pair();
                g.players[1].b.id = id;
                g.players[1].hits = 255;
                g.body_mut(d).hp = hp;
                let rng = g.w.rng;
                g.rand();
                let expected_rng = g.w.rng;
                g.w.rng = rng;
                g.pass_10dcc();
                let p = &g.players[1];
                assert_eq!((p.b.hp, p.b.hp_prev), (if (hp as i16) > 1 { hp - 1 } else { 1 }, hp));
                assert_eq!((p.b.state(), p.b.live, p.b.stun, p.b.knock), (0x0202_0400, 2, 0, 6));
                assert_eq!((p.item_type, p.drop, p.hits, p.hits_t, p.stun_refill), (0, 1, 0, 60, 120));
                assert_eq!((p.hit_serial, p.b.serial, p.b.attacker, p.combo), (3, 0, a, 8));
                assert_eq!((p.b.hitstop, p.b.b9a, g.body(a).hitstop), (5, 9, 5));
                assert_eq!(g.w.rng, expected_rng);
            }
        }
    }

    #[test]
    fn owner_serial_immunity_and_air_strikes_skip_without_rng() {
        for case in 0..7 {
            let (mut g, a, d) = pair();
            match case {
                0 => g.body_mut(a).who = 1,
                1 => g.body_mut(d).serial = 3,
                2 => g.body_mut(d).live = 2,
                3 => g.body_mut(d).kind = 8,
                4 => g.body_mut(d).kind = 0xA,
                5 => g.body_mut(a).atk = 0x6C,
                _ => { g.body_mut(a).atk = 0x6E; g.body_mut(a).who = 0xFF; }
            }
            let rng = g.w.rng;
            g.pass_10dcc();
            assert_eq!((g.body(d).hp, g.w.rng), (10, rng), "case {case}");
        }
        // +$E9 is deliberately not the serial filter for this pass.
        let (mut g, _, d) = pair();
        g.players[1].hit_serial = 3;
        g.body_mut(d).kind = 6;
        g.pass_10dcc();
        assert_eq!(g.body(d).hp, 9);
    }

    #[test]
    fn enemy_objects_use_full_damage_and_ignore_exact_depth_and_hero_defense() {
        for hp in [6, 7, 8] {
            let (mut g, a, d) = pair();
            g.body_mut(a).who = 0xFF;
            g.body_mut(d).set_zi(1);
            g.body_mut(d).hp = hp;
            g.players[1].action = 0x22;
            g.pass_10dcc();
            assert_eq!(g.body(d).hp, hp.saturating_sub(7));
            assert_eq!(g.body(d).reaction, 0x22);
        }
    }
    #[test]
    fn friendly_depth_is_exact_at_any_census_and_swung_weapons_stop_the_holder() {
        for alive in [0, 4] {
            for dz in [-1, 0, 1] {
                let (mut g, a, d) = pair();
                g.spawn.g.alive = alive;
                g.body_mut(d).set_zi(dz);
                let holder = Game::player_handle(0);
                g.body_mut(a).partner = holder;
                g.items.slots[0].m.sw(0xA0, 0x203);
                let rng = g.w.rng;
                g.pass_10dcc();
                assert_eq!(g.body(d).hp, if dz == 0 { 9 } else { 10 });
                assert_eq!(g.body(holder).hitstop, if dz == 0 { 5 } else { 0 });
                if dz != 0 { assert_eq!(g.w.rng, rng); }
            }
        }
    }

}


#[cfg(test)]
mod attack_object_mame_tests {
    use super::*;
    use crate::harness::TraceReader;

    fn word(ram: &[u8], o: usize) -> u16 { u16::from_be_bytes([ram[o], ram[o + 1]]) }

    /// 0x10E4A: replay every recorded attack-object hit with the measured collision boxes.
    /// Whole-frame step comparisons remain the gate for update order and RNG timing.
    #[test]
    fn attack_object_hits_match_recorded_mame_frames() {
        let root = std::path::Path::new("../../..");
        if !root.join("dino.zip").exists() { return; }
        let assets = dino_extract::extract(&root.join("dino.zip")).expect("ROM loads");
        let mut g = Game::new(Box::leak(Box::new(assets)));
        let dir = root.join("re/out/traces");
        let mut paths: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(Result::ok)
            .map(|e| e.path()).filter(|p| {
                let n = p.file_name().unwrap().to_string_lossy();
                n.ends_with(".mame.trc") && (n.starts_with("ffi_") || n.starts_with("m516_") || n.starts_with("boss_vice_"))
            }).collect();
        paths.sort();
        let mut hits = 0;
        for path in paths {
            let mut rd = TraceReader::open(path.to_str().unwrap()).unwrap();
            let mut previous: Option<Vec<u8>> = None;
            let mut n = 0;
            while let Some(frame) = rd.next() {
                let ram = &rd.region("ram").unwrap().data;
                if let Some(prev) = &previous {
                    for i in 0..3 {
                        let o = Game::player_handle(i) as usize;
                        let a = word(ram, o + 0x68);
                        if ram[o] != 2 || word(ram, o + 0x6C) == word(prev, o + 0x6C)
                            || !matches!(slot_of(a), Some(Slot::Item(_)))
                            || ram[a as usize + 0x2E] != 4 { continue; }
                        g.trace_load(ram, &rd.region("gfx").unwrap().data);
                        let p = &mut g.players[i];
                        p.b.hp = word(ram, o + 0x6E);
                        p.b.stun = prev[o + 0x78];
                        g.enemy_attack_object_hits_hero(a, o as u16);
                        let p = &g.players[i];
                        assert_eq!((p.b.hp, p.b.stun, p.b.reaction, p.b.act, p.b.hitstop, p.b.knock, p.b.b9a),
                            (word(ram, o + 0x6C), ram[o + 0x78], ram[o + 0x7B], ram[o + 6], ram[o + 0x59], ram[o + 0x5A], ram[o + 0x9A]),
                            "{} frame {frame}, player {i}", path.display());
                        assert_eq!((p.item_type, p.drop), (word(ram, o + 0xB6), ram[o + 0xC5]));
                        n += 1;
                    }
                }
                previous = Some(ram.clone());
            }
            if n > 0 { eprintln!("{}: all {n} object hits equal MAME", path.file_name().unwrap().to_string_lossy()); }
            hits += n;
        }
        assert!(hits > 0, "requires weapon friendly-fire recordings");
        eprintln!("attack objects: {hits} recorded hits equal MAME");
    }
}
