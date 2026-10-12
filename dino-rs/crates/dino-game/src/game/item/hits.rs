//! Collision passes that hit pool E objects (containers): 0x11F86, 0x1201C, 0x120D2.
//!
//! The containers register in the hurtable list `$6AFC` (0x4B80, only while +$00 == 1). A hit
//! sets +$00 = 2 (the container reads it as "hit"), the attacker in +$68, and lowers HP:
//! * 0x11F86: bodies in the list `$6AC2` (knocked down, thrown or burning fighters) crush a
//!   container of frame class 0: HP - 8, the container looks away from their motion.
//! * 0x1201C: an enemy attack (frame class 4, no hit stop) in the list `$671E` destroys it (HP 0,
//!   nobody credited: +$02 = 0xFF).
//! * 0x120D2: attackers in `$671A` (frame class 4: thrown containers and swung weapons, not bare
//!   hands) hurt it with the damage of the attacker's player attack record (table 0xA26FC), the
//!   hit stop goes to both sides.

use super::*;
use crate::game::collide;

impl Game {
    /// 0x11F86.
    pub fn pass_11f86(&mut self) {
        for a2 in self.combat.get(0x6AC2).to_vec() {
            for a3 in self.combat.get(0x6AFC).to_vec() {
                self.item_crushed(a2, a3);
            }
        }
    }

    /// 0x11FB8: body box of `a2` against the body box of the item `a3` (frame class 0).
    fn item_crushed(&mut self, a2: Handle, a3: Handle) {
        let Some(Slot::Item(j)) = slot_of(a3) else { return };
        let (ab, ib) = (self.body(a2).clone(), self.items.slots[j].b.clone());
        if ab.body_box == 0 || ib.body_box == 0 || ib.kind != 0 {
            return;
        }
        let Some((ha, hb)) = collide::boxes_hit(&self.assets.sprites, &ab, ab.body_box, &ib, ib.body_box) else { return };
        let it = &mut self.items.slots[j];
        it.b.live = 2;
        it.b.who = ab.who;
        it.m.sb(0x7A, 0);
        it.b.attacker = a2;
        it.b.hp_prev = it.b.hp;
        let hp = (it.b.hp as i16).wrapping_sub(8);
        it.b.hp = if hp > 0 { hp as u16 } else { 0 };
        self.spawn_hit_spark(a2, a3, ha, hb, 0);
        self.items.slots[j].b.facing = (ab.vx < 0) as u8;
    }

    /// 0x1201C.
    pub fn pass_1201c(&mut self) {
        for a2 in self.combat.get(0x671E).to_vec() {
            if self.body(a2).hitstop != 0 {
                continue;
            }
            for a3 in self.combat.get(0x6AFC).to_vec() {
                let Some(Slot::Item(j)) = slot_of(a3) else { continue };
                if self.items.slots[j].b.kind != 0 {
                    continue;
                }
                // 0x1206A by the enemy's frame class: only 4 (an attack frame) does anything.
                if self.body(a2).kind == 4 {
                    self.item_hit_by_enemy(a2, a3);
                }
            }
        }
    }

    /// 0x1207C: an enemy attack box against the body box of a container: it is destroyed.
    fn item_hit_by_enemy(&mut self, a2: Handle, a3: Handle) {
        let Some(Slot::Item(j)) = slot_of(a3) else { return };
        let (ab, ib) = (self.body(a2).clone(), self.items.slots[j].b.clone());
        if ab.attack_box == 0 || ib.body_box == 0 {
            return;
        }
        let Some((ha, hb)) = collide::boxes_hit(&self.assets.sprites, &ab, ab.attack_box, &ib, ib.body_box) else { return };
        let it = &mut self.items.slots[j];
        it.b.live = 2;
        it.b.who = 0xFF;
        it.m.sb(0x7A, 0);
        it.b.attacker = a2;
        it.b.hp_prev = it.b.hp;
        it.b.hp = 0;
        self.spawn_hit_spark(a2, a3, ha, hb, 0);
        // cmp.w $8(a3),d1 ; bcs: facing 0 when the attacker is left of the container.
        self.items.slots[j].b.facing = (ab.xi() as u16 >= self.items.slots[j].b.xi() as u16) as u8;
    }

    /// 0x120D2.
    pub fn pass_120d2(&mut self) {
        for a2 in self.combat.get(0x671A).to_vec() {
            if self.body(a2).kind != 4 {
                continue;
            }
            for a3 in self.combat.get(0x6AFC).to_vec() {
                let Some(Slot::Item(j)) = slot_of(a3) else { continue };
                let ab = self.body(a2).clone();
                if a2 == a3 || ab.serial == self.items.slots[j].b.serial {
                    continue;
                }
                let ib = self.items.slots[j].b.clone();
                if ab.attack_box == 0 || ib.body_box == 0 || ib.kind != 0 {
                    continue;
                }
                self.item_hit_by_attacker(a2, a3);
            }
        }
    }

    /// 0x12150: a swung or thrown attacker hurts a container with its player attack record
    /// (character +$03, or 0 when +$02 is negative; record at +$58).
    fn item_hit_by_attacker(&mut self, a2: Handle, a3: Handle) {
        let Some(Slot::Item(j)) = slot_of(a3) else { return };
        let (ab, ib) = (self.body(a2).clone(), self.items.slots[j].b.clone());
        let Some((ha, hb)) = collide::boxes_hit(&self.assets.sprites, &ab, ab.attack_box, &ib, ib.body_box) else { return };
        let ch = if (ab.who as i8) < 0 { 0 } else { ab.id as usize & 3 };
        let r = self.assets.sprites.player_attacks[ch][(ab.atk / 2) as usize];
        let it = &mut self.items.slots[j];
        it.b.live = 2;
        it.b.serial = ab.serial;
        it.b.who = ab.who;
        it.m.sb(0x7A, 0);
        it.b.attacker = a2;
        it.b.hp_prev = it.b.hp;
        let hp = (it.b.hp as i16).wrapping_sub(r.damage as u8 as i16);
        it.b.hp = if hp > 0 { hp as u16 } else { 0 };
        it.b.hitstop = r.hit_stop;
        it.b.b9a = r.score;
        self.body_mut(a2).hitstop = r.hit_stop;
        self.spawn_hit_spark(a2, a3, ha, hb, 0);
        self.items.slots[j].b.facing = ab.facing ^ 1;
    }
}
