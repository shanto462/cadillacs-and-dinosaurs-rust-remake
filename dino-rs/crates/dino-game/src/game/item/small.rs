//! Food (class 4, 0x2898C), treasure (class 9, 0x2A028) and ammo boxes (class 2, 0x283DE).
//!
//! All three are spawned by the drop roll (act = 2: they hop out of their source, Y + 0x30,
//! vy 0x300, ay -0x30, and land with the 0x24CE6 test) or by the stage script (act 0: they lie
//! where they are). Their +$26 is the type: food 1..18 (table 0x28B10: heal and score offset),
//! treasure 1..16 (table 0x2A164: score offset), ammo boxes have none. A player lifts them with
//! the pick-up state; **food is eaten on the first frame of the lift** (0x28AA4: heal, or score
//! when the player is at full HP, HUD message), treasure scores when the lift ends (0x2A10E).

use super::*;

/// Animation tables: food 0x28B54, treasure 0x2A174; the ammo box has one frame record.
const FOOD_TABLE: u32 = 0x28B54;
const TREASURE_TABLE: u32 = 0x2A174;
const AMMO_FRAME: u32 = 0x285DC;

/// What eating a food does to a player with `hp` HP (0x28AB8..0x28AF8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoodEffect {
    /// New HP (heal added, capped at 100).
    Heal(u16),
    /// The player is at 100 HP: the food is worth points.
    Score,
}

/// 0x28AB8: at exactly 100 HP the food scores; otherwise `heal` is added and the sum capped at 100
/// (`cmpi.w #$64,d0 ; bls`: unsigned).
pub fn food_effect(heal: u16, hp: u16) -> FoodEffect {
    if hp == 0x64 {
        return FoodEffect::Score;
    }
    let sum = heal.wrapping_add(hp);
    FoodEffect::Heal(if sum > 0x64 { 0x64 } else { sum })
}

impl Game {
    /// Header init shared by the three classes: box table, draw flags, snap to the floor, the
    /// scratch bytes. `a0`: kind word (+$A0).
    fn item_small_common(&mut self, it: &mut Item, h: Handle, kind: u16) {
        it.b.draw_flags = 0x10;
        it.b.box_table = 0xC010_0000;
        it.b.multi_box = false;
        it.m.sw(KIND, kind);
        self.item_snap_to_floor(it, h);
        it.b.facing = 0;
        it.m.sw(LIFE, 0);
        it.m.sb(A7, 0);
        it.m.sb(B81, 0);
        it.m.sb(AF, 0);
        it.b.palette_override = 0;
    }

    /// 0x28A0E / 0x2A0A4 / 0x28450: an item spawned by a drop (act != 0) hops out of its source.
    fn item_small_hop(&mut self, it: &mut Item) {
        it.b.airborne = 1;
        let y = it.b.yi().wrapping_add(0x30);
        put_hi(&mut it.b.y, y);
        it.b.vy = 0x300;
        it.b.ay = -0x30;
        it.b.sub += 2;
    }

    // ------------------------------------------------------------------ food

    /// 0x2898C food.
    pub(super) fn item_food(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.mode {
            0 => {
                if it.b.sub == 0 {
                    it.b.anim_table = FOOD_TABLE;
                    self.item_small_common(it, h, it.b.var26);
                    self.item_anim(it, it.b.var26.wrapping_sub(1));
                    it.b.set_frame2(it.b.frame);
                    if it.b.act == 0 {
                        it.b.mode += 2;
                        return;
                    }
                    self.item_small_hop(it);
                }
                // 0x28A2A flight; the landing keeps +$07.
                self.item_food_flight(it, h);
            }
            2 => match it.b.sub {
                0 => self.item_ground_idle(it, h),
                4 => self.item_food_lift(it, h),
                8 | 0xC => {
                    // 0x28A86: the eating sound (0x9F54), then free.
                    self.item_snd_food(it);
                    it.set_mode_sub(6, 0);
                }
                _ => {}
            },
            _ => self.item_free(it, i),
        }
    }

    /// 0x28A2A: tick, copy the frame to +$30, move, floor; landed (and not hopping back in from
    /// the screen edge): lie still (sub 0, act 0 only: `clr.b $6`).
    fn item_food_flight(&mut self, it: &mut Item, h: Handle) {
        self.item_anim_step(it);
        it.b.set_frame2(it.b.frame);
        it.b.motion();
        self.item_terrain(it, h);
        if it.b.airborne == 0 && self.item_land_check(it) {
            it.set_mode_sub(2, 0);
            it.b.act = 0;
        }
        self.item_draw_check(it, h);
    }

    /// 0x2495A: the lift of food: eat on the first frame, then like `item_lift_b`.
    fn item_food_lift(&mut self, it: &mut Item, h: Handle) {
        if it.b.act == 0 {
            self.item_eat(it, h);
            it.b.act += 2;
        }
        self.item_lift_b(it, h);
    }

    /// 0x28AA4: eat the food. Below 100 HP the player heals by the table value (capped at 100),
    /// the life bar is redrawn and the HUD shows the food; at 100 HP the food is worth points
    /// instead (the item's HP shows 0x63 for the HUD message).
    pub(super) fn item_eat(&mut self, it: &mut Item, h: Handle) {
        let t = &self.assets.enemy.t;
        let k = (it.b.var26 as u32).wrapping_sub(1).wrapping_mul(4);
        let (heal, score_off) = (t.u16(0x28B10, k), t.u16(0x28B10, k + 2));
        let Some(Slot::Player(p)) = slot_of(it.b.partner) else { return };
        match food_effect(heal, self.players[p].b.hp) {
            FoodEffect::Heal(hp) => {
                self.players[p].b.hp = hp;
                self.hud_life_bar(p);
                it.b.hp = 0x64;
            }
            FoodEffect::Score => {
                it.b.hp = 0x63;
                let v = self.assets.text.u32(0x10292, score_off as u32);
                self.hud_add_score_to(it.b.who, v);
            }
        }
        self.item_in_slot(it, h, |g| g.hud_food(h));
    }

    /// 0x9F54: the sound of eating: by the eater's voice bank (+$118) and character (Hannah has
    /// her own row): table 0x9FAA / 0x9FB0. d1 = the mode 7 in the high word and the pan byte of the food's X (+$08, a6),
    /// d2 = 0.
    pub(super) fn item_snd_food(&mut self, it: &Item) {
        let Some(Slot::Player(p)) = slot_of(it.b.partner) else { return };
        let (voice, id) = (self.players[p].voice as u32, self.players[p].b.id);
        let row = if id == 2 { 0x9FB0 } else { 0x9FAA };
        let code = self.assets.enemy.t.u16(row, voice * 2);
        let d1 = 0x0007_0000 | self.snd_pan_x(it.b.xi()) as u32;
        self.sound_ex(code, d1, 0);
    }

    // ------------------------------------------------------------------ treasure

    /// 0x2A028 treasure.
    pub(super) fn item_treasure(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.mode {
            0 => {
                if it.b.sub == 0 {
                    it.b.anim_table = TREASURE_TABLE;
                    self.item_small_common(it, h, 0x300 | it.b.var26);
                    self.item_anim(it, it.b.var26.wrapping_sub(1));
                    if it.b.act == 0 {
                        it.b.mode += 2;
                        return;
                    }
                    self.item_small_hop(it);
                }
                // 0x2A0C2 flight (no frame copy); the landing clears act and step.
                self.item_anim_step(it);
                it.b.motion();
                self.item_terrain(it, h);
                if it.b.airborne == 0 && self.item_land_check(it) {
                    it.set_mode_sub(2, 0);
                    it.clear_act();
                }
                self.item_draw_check(it, h);
            }
            2 => match it.b.sub {
                0 => self.item_ground_idle(it, h),
                4 => self.item_lift_b(it, h),
                8 | 0xC => self.item_treasure_collect(it, h),
                _ => {}
            },
            _ => self.item_free(it, i),
        }
    }

    /// 0x2A10E: the treasure is taken: HUD message, score (table 0x2A164), free next frame. The
    /// RADIO of stage 3 (+$26 = 0xC, G.$4D9 = 2, no kill-all) also calls 0x2A12C -> 0x8BD24, the radio event: three pool B
    /// class 0x58 objects (the car and its two helpers, G.$51A), G.$4D5 = 1, G.$4DB = 1 and G.$50E = 1. That event is a stand in
    /// until M6 (the car, D3c section 5): the radio only gives its 3000 points and the road goes on on foot. The camera side
    /// (the chase of mode 8, G.$518) is ported (`camera.rs`, M3-09) and waits for the car object to set G.$518 and G.$51A.
    fn item_treasure_collect(&mut self, it: &mut Item, h: Handle) {
        self.item_in_slot(it, h, |g| g.hud_treasure(h));
        let off = self.assets.enemy.t.u8(0x2A164, it.b.var26.wrapping_sub(1) as u32);
        let v = self.assets.text.u32(0x10292, off as u32);
        self.hud_add_score_to(it.b.who, v);
        if self.w.stage == 2 && self.spawn.g.kill_all == 0 && it.b.var26 == 0xC {
            self.prop_car_radio(it); // 0x2A150: jsr 0x8BD24
        }
        it.set_mode_sub(6, 0);
    }

    // ------------------------------------------------------------------ ammo box

    /// 0x283DE ammo box.
    pub(super) fn item_ammo(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.mode {
            0 => {
                if it.b.sub == 0 {
                    it.b.draw_flags = 0x10;
                    it.b.box_table = 0xC010_0000;
                    it.b.multi_box = false;
                    it.m.sw(KIND, 0x100);
                    self.item_snap_to_floor(it, h);
                    it.b.facing = 0;
                    it.m.sw(LIFE, 0);
                    it.m.sb(A7, 0);
                    it.m.sb(B81, 0);
                    it.b.hp = 0xA;
                    it.m.sb(AF, 0);
                    it.b.palette_override = 0;
                    let s = &self.assets.sprites;
                    it.b.load_frame(s, s.frame_at(AMMO_FRAME)); // 0x1236
                    if it.b.act == 0 {
                        it.b.mode += 2;
                        return;
                    }
                    self.item_small_hop(it);
                }
                // 0x2846E flight: no animation.
                it.b.motion();
                self.item_terrain(it, h);
                if it.b.airborne == 0 && self.item_land_check(it) {
                    it.set_mode_sub(2, 0);
                    it.clear_act();
                }
                self.item_draw_check(it, h);
            }
            2 => match it.b.sub {
                0 => self.item_weapon_ground(it, h),
                4 => self.item_lift_b(it, h),
                8 | 0xC => self.item_ammo_pickup(it, h),
                _ => {}
            },
            _ => self.item_free(it, i),
        }
    }

    /// 0x284B4: the player that took an ammo box refills the gun in its hand (0x284E2); with
    /// nothing to refill the box is worth 1000 points and the player remembers it for the next
    /// gun (+$10F = 1). The box is freed next frame.
    pub(super) fn item_ammo_pickup(&mut self, it: &mut Item, _h: Handle) {
        let Some(Slot::Player(p)) = slot_of(it.b.partner) else {
            it.set_mode_sub(6, 0);
            return;
        };
        if !self.item_ammo_refill(p, it.b.who) {
            // moveq #$38,d0 ; jsr $10036 ; move.b #1,$10f(a0)
            let v = self.assets.text.u32(0x10292, 0x38);
            self.hud_add_score_to(it.b.who, v);
            self.players[p].b10f = 1;
        }
        it.set_mode_sub(6, 0);
    }

    /// 0x284E2: refill the gun a player holds (`+$B4` item, kind `+$B6` of class 0x02xx with a
    /// refill row in the table 0x2855A): the ammo (+$6C of the gun) goes to the maximum, a gun
    /// that was already full scores 1000. The gun's kind / +$A2 are taken from the table row.
    /// Returns the carry-clear result: true = refilled (a gun was held).
    pub(in crate::game) fn item_ammo_refill(&mut self, p: usize, who: u8) -> bool {
        let (gun, kind) = (self.players[p].item, self.players[p].item_type);
        if kind == 0 || kind & 0xF00 != 0x200 {
            return false;
        }
        let Some(Slot::Item(gi)) = slot_of(gun) else { return false };
        let g = &self.items.slots[gi];
        if g.b.k20 != 0 || g.m.w(KIND) != kind {
            return false;
        }
        let row = (g.b.var26 & 0x1E) as u32 * 4;
        let t = &self.assets.enemy.t;
        let (max, newkind, a2) = (t.u16(0x2855A, row), t.u16(0x2855A, row + 2), t.u16(0x2855A, row + 4));
        if max == 0 {
            return false;
        }
        if max == g.b.hp {
            let v = self.assets.text.u32(0x10292, 0x38);
            self.hud_add_score_to(who, v);
        }
        let g = &mut self.items.slots[gi];
        g.b.hp = max;
        g.m.sw(KIND, newkind);
        self.players[p].item_type = newkind;
        g.m.sw(A2, a2);
        self.players[p].b10f = 0;
        g.m.sb(WEAR, 0);
        true
    }
}
