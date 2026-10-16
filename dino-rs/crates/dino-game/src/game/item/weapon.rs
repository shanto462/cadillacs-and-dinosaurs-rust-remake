//! Class 0: weapons (handler 0x24D44). 15 types (+$26 = type * 2), spawned by drop rolls
//! (act = 2) or by stage scripts.
//!
//! | var | kind (+$A0) | what | var | kind | what |
//! |---|---|---|---|---|---|
//! | 0 | 0x205 | rifle (6 shots) | 0xE | 0x203 | rod, 10 uses |
//! | 2 | 0x204 | pistol | 0x10 | 0x202 | stick |
//! | 4 | 0x202 | knife (counts on screen) | 0x12 | 0x203 | butcher knife, re-inits when it leaves |
//! | 6 | 0x206 | Uzi (48) | 0x14 | 0x203 | club |
//! | 8 | 0x207 | M-16 (48) | 0x16 | 0x203 | torch, with an overlay (class 6) |
//! | 0xA | 0x208 | shotgun (6) | 0x18 | 0x201 | dynamite |
//! | 0xC | 0x209 | bazooka (4) | 0x1A | 0x201 | grenade |
//! | | | | 0x1C | 0x201 | stone (G.$4F1 counts them) |
//!
//! Kind 0x201 = thrown only, 0x202 / 0x203 = melee (swing), 0x204..0x209 = guns (ammo in +$6C;
//! an empty gun becomes a melee 0x203 or a throwable 0x201).
//!
//! * mode 0 init (0x24D58, per type 0x24D8A..0x25020, common 0x25060), mode 2 alive, 4 / 6
//!   cleanup (0x264D0, type 0x12 re-initialises instead of freeing).
//! * sub (+$05): 0 on the floor (0x2463C), 4 lifted (0x24780), 8 held (0x25178, per type),
//!   0xC dropped (0x247FA), 0x10..0x28 thrown / flying (per type).
//!
//! The held weapon reads the player each frame: `+$C4` (attack flag, cleared when the shot is
//! used), `+$2C` (frame byte: 1 / 2 = the swing frame), `+$C5` (drop request), `+$B4` / `+$B6`
//! (what the player holds; the weapon sets +$B6 = 0 when thrown, or its new kind when it runs
//! empty). See the module doc of `item.rs`.

use super::*;
use crate::game::enemy::Enemy;

/// Animation tables by type (0x24D6C..): the first record of each is the init of that type.
const ANIMS: [u32; 15] = [
    0x265F4, 0x268BC, 0x26A3C, 0x26BBC, 0x26D3C, 0x26F7E, 0x270FE, 0x2727E, 0x273FE, 0x2757E, 0x276FE, 0x276FE, 0x2787E, 0x27B12, 0x27CD2,
];
/// (kind, +$A2, ammo / uses) by type.
const KINDS: [(u16, u16, u16); 15] = [
    (0x205, 0x04, 6),
    (0x204, 0x00, 6),
    (0x202, 0x0C, 6),
    (0x206, 0x10, 0x30),
    (0x207, 0x14, 0x30),
    (0x208, 0x1C, 6),
    (0x209, 0x20, 4),
    (0x203, 0x24, 0xA),
    (0x202, 0x28, 6),
    (0x203, 0x2C, 6),
    (0x203, 0x30, 0xA),
    (0x203, 0x30, 0xA),
    (0x201, 0x34, 0xA),
    (0x201, 0x38, 0xA),
    (0x201, 0x38, 0xA),
];
/// Weapon tables 0x26408..: explosion offsets, throw vectors (0x26534 + 0x10 * k), hand offsets
/// (0x265B4), the random depth speeds (0x265D4).
const THROW_BASE: u32 = 0x26534;
const HAND_OFFSETS: u32 = 0x265B4;
const SPEED_Z: u32 = 0x265D4;
const CHAIN: u32 = 0x26408;
const HIT_CHANCE: u32 = 0x2CBA0;
const BREAK_MASKS: u32 = 0x2CB80;
const GUN_KIND_FLAG: u32 = 0x2494A;

impl Game {
    /// 0x24D44 weapon.
    pub(super) fn item_weapon(&mut self, it: &mut Item, h: Handle, i: usize) {
        match it.b.mode {
            0 => {
                if it.b.sub != 0 {
                    return self.weapon_drop_flight(it, h); // 0x250F0
                }
                if !self.weapon_init(it, h) {
                    return;
                }
            }
            2 => match it.b.sub {
                0 => self.item_weapon_ground(it, h),
                4 => self.item_lift_a(it, h),
                8 => self.weapon_held(it, h),
                0xC => self.weapon_dropped(it, h),
                0x10 => self.weapon_fly_10(it, h),
                0x14 => self.weapon_fly_14(it, h),
                0x18 => self.weapon_fly_18(it, h),
                0x1C => self.weapon_fly_1c(it, h),
                0x20 => self.weapon_fly_20(it, h),
                0x24 => self.weapon_fly_24(it, h),
                0x28 => self.weapon_fly_28(it),
                _ => {}
            },
            _ => self.weapon_cleanup(it, i), // 0x264D0
        }
    }

    // ------------------------------------------------------------------ init

    /// 0x24D58 init by type (0x24D8A..0x25020), then the common part (0x25060). Returns false
    /// for a variant outside the 15 types (the 68000 would run into data); freed.
    fn weapon_init(&mut self, it: &mut Item, h: Handle) -> bool {
        let var = it.b.var26;
        let k = (var / 2) as usize;
        if var % 2 != 0 || k >= KINDS.len() {
            it.b.mode = 6;
            return false;
        }
        it.b.anim_table = ANIMS[k];
        let (kind, a2, ammo) = KINDS[k];
        it.m.sw(KIND, kind);
        it.m.sw(A2, a2);
        match k {
            2 => it.m.sb(B80, 5),
            3 | 4 => it.m.sb(B80, 0),
            _ => {}
        }
        // +$6C is set by every type; types 7 and 12..14 leave +$6E / +$6A alone.
        it.b.hp = ammo;
        if !matches!(k, 7 | 12 | 13 | 14) {
            it.b.hp_prev = ammo;
            it.b.hp_bar = ammo;
        }
        match k {
            11 => self.weapon_spawn_overlay(it, h),
            12 => {
                it.m.sw(B0, 0);
                it.m.sw(B2, 3);
                it.m.sw(B4, 0);
                if it.b.step != 0 {
                    // 0x24FDC: armed at once (thrown by an enemy): lit, owner 0xFF.
                    self.weapon_init_common(it, h);
                    self.weapon_arm(it);
                    it.b.who = 0xFF;
                    it.m.sw(B0, 1);
                    it.m.sw(B2, 1);
                    it.b.step = 0;
                    return false;
                }
            }
            14 => {
                let w5c = it.m.w(o::W5C);
                it.b.anim_table = match w5c {
                    0 => 0x27CD2,
                    1 => 0x27D1A,
                    _ => 0x27D62,
                };
                self.spawn.g.b4f1 = self.spawn.g.b4f1.wrapping_add(1);
            }
            _ => {}
        }
        self.weapon_init_common(it, h);
        true
    }

    /// 0x24F4C: type 11 spawns its overlay (class 6) before the common init.
    fn weapon_spawn_overlay(&mut self, it: &mut Item, h: Handle) {
        let pos = (it.b.xi(), it.b.yi(), it.b.zi());
        let Some(j) = self.items.alloc() else { return };
        let n = &mut self.items.slots[j];
        n.b.live = 1;
        n.b.who = it.b.who;
        n.b.id = it.b.id;
        n.b.k20 = class::OVERLAY * 4;
        put_hi(&mut n.b.x, pos.0);
        put_hi(&mut n.b.y, pos.1);
        put_hi(&mut n.b.z, pos.2);
        n.m.sw(0x58, it.m.w(KIND)); // move.w $a0(a6),$58(a0): +$58 / +$59 of the overlay
        n.m.sw(0x5A, h);
    }

    /// 0x25060: box table, floor, scratch fields, then the lying (act 0) or hopping (act != 0)
    /// start.
    fn weapon_init_common(&mut self, it: &mut Item, h: Handle) {
        it.b.box_table = 0x0010_0C00;
        it.b.multi_box = false;
        it.b.draw_flags = 0x10;
        self.item_snap_to_floor(it, h);
        it.m.sw(LIFE, 0);
        it.m.sb(WEAR, 0);
        it.m.sb(A7, 0);
        it.b.hitstop = 0;
        it.m.sw(AA, 0x258);
        it.m.sb(AD, 0);
        it.m.sb(B81, 0);
        it.m.sb(AF, 0);
        it.b.palette_override = 0;
        it.m.sb(B0, 0);
        it.m.sb(B1, 0);
        self.item_anim(it, 0x11);
        if it.b.act == 0 {
            it.b.mode += 2;
            it.b.step = 0;
            return;
        }
        self.item_anim(it, 0x10);
        it.b.airborne = 1;
        let y = it.b.yi().wrapping_add(0x30);
        put_hi(&mut it.b.y, y);
        it.b.vy = 0x300;
        it.b.ay = -0x30;
        it.b.sub = 2;
        it.b.draw_flags = 0;
        self.weapon_drop_flight(it, h);
    }

    /// 0x250F0: the hop of a dropped weapon: it lands when the floor is reached (and does not
    /// hop back in from an edge).
    fn weapon_drop_flight(&mut self, it: &mut Item, h: Handle) {
        self.item_anim_step(it);
        it.b.motion();
        self.item_terrain(it, h);
        if it.b.airborne == 0 && self.item_land_check(it) {
            self.item_anim(it, 0x11);
            it.set_mode_sub(2, 0);
            it.m.sb(B81, 0);
            it.b.act = 0;
            it.b.step = 0;
            it.b.draw_flags = 0x10;
        }
        self.item_draw_check(it, h);
    }

    /// 0x264D0 modes 4 / 6: type 0x12 starts over (state 0: it re-initialises next frame), type
    /// 0x1C gives its slot in G.$4F1 back, everything else is freed.
    fn weapon_cleanup(&mut self, it: &mut Item, i: usize) {
        if it.b.var26 == 0x12 {
            it.b.set_state(0);
            return;
        }
        if it.b.var26 == 0x1C {
            let (v, borrow) = self.spawn.g.b4f1.overflowing_sub(1);
            self.spawn.g.b4f1 = if borrow { 0 } else { v };
        }
        self.item_free(it, i);
    }

    // ------------------------------------------------------------------ on the floor

    /// 0x2463C: a weapon or ammo box lying on the floor.
    pub(super) fn item_weapon_ground(&mut self, it: &mut Item, h: Handle) {
        self.item_anim_step(it);
        it.b.palette_override = 0;
        if it.b.param_a != 0 {
            it.b.palette_override = 0x8A;
            if it.b.hp == 0 {
                it.b.palette_override = 0x84;
            }
        }
        let (kind, var) = (it.m.w(KIND), it.b.var26);
        if !(kind == 0x203 && var == 0x12) {
            let life = it.m.w(LIFE);
            if life != 0 {
                it.m.sw(LIFE, life - 1);
                if life - 1 == 0 {
                    it.set_mode_sub(6, 0);
                    return;
                }
            }
        }
        if kind == 0x201 && var == 0x18 && it.m.w(B0) != 0 {
            // 0x24724: a lit dynamite counts down.
            it.b.palette_override = 0;
            if it.b.param_a != 0 {
                it.b.palette_override = 0x84;
            }
            let b4 = it.m.w(B4).wrapping_sub(1);
            it.m.sw(B4, b4);
            if b4 == 0 {
                let (b2, borrow) = it.m.w(B2).overflowing_sub(1);
                it.m.sw(B2, b2);
                if borrow {
                    let k = if (it.b.who as i8) < 0 { 3 } else { 0 };
                    self.item_explosion(it, h, k);
                    it.set_mode_sub(6, 0);
                    return;
                }
                self.weapon_arm(it);
            }
        } else {
            self.item_despawn_near(it);
        }
        self.item_register_attacker(h);
        if it.b.act != 0 {
            // A weapon that broke blinks for +$80 frames (odd frames drawn).
            let d = it.m.b(B80).wrapping_sub(1);
            it.m.sb(B80, d);
            if d == 0 {
                it.set_mode_sub(6, 0);
                return;
            }
            if d & 1 == 0 {
                return;
            }
        } else if kind == 0x202 && var == 4 {
            // At most three of these stay on the field: the others count down +$AA.
            self.spawn.g.b4fa = self.spawn.g.b4fa.wrapping_add(1);
            if self.spawn.g.b4fa > 3 {
                let aa = it.m.w(AA).wrapping_sub(1);
                it.m.sw(AA, aa);
                if aa == 0 {
                    return self.weapon_break(it);
                }
            } else {
                it.m.sw(AA, 0xB4);
            }
        }
        self.item_register_pickup(h, it.b.live);
        self.item_draw_check(it, h);
    }

    /// 0x24766: arm the bomb (B0 = 1): the picture of the current stage (anim 0x12 + B2), a
    /// 15 frame step.
    fn weapon_arm(&mut self, it: &mut Item) {
        it.m.sw(B0, 1);
        let id = 0x12u16.wrapping_add(it.m.w(B2));
        self.item_anim(it, id);
        it.m.sw(B4, 0xF);
    }

    /// 0x24934: the weapon breaks: it blinks away for 30 frames (state 0x02000200, act 2).
    fn weapon_break(&mut self, it: &mut Item) {
        it.m.sb(B80, 0x1E);
        it.b.set_state(0x0200_0200);
        self.item_anim(it, 0x11);
    }

    /// 0x248F6: when a dropped weapon lands, a worn one may break: guns whose ammo is negative
    /// always, others with the chance given by the wear (+$A6 >> 5, masks 0x2CB80, bit rand() & 31).
    fn weapon_wear_check(&mut self, it: &mut Item) {
        let flag = self.assets.enemy.t.u8(GUN_KIND_FLAG, (it.m.w(KIND) & 0xF) as u32);
        if flag != 0 && (it.b.hp as i16) < 0 {
            return self.weapon_break(it);
        }
        let d0 = ((it.m.b(WEAR) & 0xE0) >> 3) as u32;
        let mask = self.assets.enemy.t.u32(BREAK_MASKS, d0);
        let r = self.rand() & 0x1F;
        if mask >> r & 1 != 0 {
            self.weapon_break(it);
        }
    }

    // ------------------------------------------------------------------ dropped

    /// 0x247FA dropped (sub 0xC): a hop, then it lies on the floor for 3600 frames; a bomb is
    /// armed, a bazooka without ammo breaks, a worn weapon may break.
    fn weapon_dropped(&mut self, it: &mut Item, h: Handle) {
        if it.b.act == 0 {
            it.b.vy = 0x300;
            it.b.ay = -0x30;
            it.b.vx = 0;
            it.b.ax = 0;
            it.b.vz = 0;
            it.b.az = 0;
            it.b.palette_override = 0;
            it.m.sb(AF, 0);
            self.item_anim(it, 0x10);
            let ae = match slot_of(it.b.partner) {
                Some(Slot::Player(p)) => self.players[p].drop,
                _ => 0,
            };
            it.m.sb(AE, ae);
            it.b.airborne = 1;
            it.b.act = 2;
        }
        self.item_anim_step(it);
        it.b.motion();
        self.item_terrain(it, h);
        if it.b.airborne == 0 {
            it.b.sub = 0;
            it.clear_act();
            it.m.sw(LIFE, 0xE10);
            self.item_anim(it, 0x11);
            let (kind, var) = (it.m.w(KIND), it.b.var26);
            if kind == 0x201 && var == 0x18 {
                self.weapon_arm(it);
            } else if kind == 0x209 && var == 0xC && it.b.hp == 0 {
                it.m.sb(B80, 0x1E);
                it.b.set_state(0x0200_0200);
                self.item_anim(it, 0x11);
                it.b.live = 2;
            } else if it.m.b(AE) != 0 && !(kind == 0x203 && var == 0x12) {
                self.weapon_wear_check(it);
            }
        }
        self.item_draw_check(it, h);
    }

    // ------------------------------------------------------------------ helpers

    /// The owner of a held weapon: `Some(player)` while the player is alive, holds this item (+$B4)
    /// and its kind matches (+$B6).
    fn weapon_owner(&self, it: &Item, h: Handle) -> Option<usize> {
        let Some(Slot::Player(p)) = slot_of(it.b.partner) else { return None };
        let pl = &self.players[p];
        (pl.b.live != 0 && pl.item == h && pl.item_type == it.m.w(KIND)).then_some(p)
    }

    /// `+$58 = id; +$5A = 0; +$59 = 0; +$5E = G.$67BE++`: a new attack.
    fn weapon_new_attack(&mut self, it: &mut Item, id: u8) {
        it.b.atk = id;
        it.b.knock = 0;
        it.b.hitstop = 0;
        it.b.serial = self.w.attack_serial;
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
    }

    /// `move.b $3(a6),d0 ; add.b d0,$49(a6)`: the attack box index grows with the character.
    fn weapon_box_by_character(it: &mut Item) {
        it.b.attack_box = it.b.attack_box.wrapping_add(it.b.id);
    }

    /// 0x264FC: before a throw the weapon moves to the thrower's side: X += (+- dx) of the table
    /// 0x265B4 (character row, second column when +$A8 is set), Y from the table.
    fn weapon_throw_origin(&self, it: &mut Item, p: usize) {
        let t = &self.assets.enemy.t;
        let o = &self.players[p].b;
        let mut row = o.k20 as u32 * 2;
        if it.m.b(A8) != 0 {
            row += 4;
        }
        let (mut dx, dy) = (t.i16(HAND_OFFSETS, row), t.i16(HAND_OFFSETS, row + 2));
        if o.facing != 0 {
            dx = dx.wrapping_neg();
        }
        let (x, y) = (dx.wrapping_add(o.xi()), dy.wrapping_add(o.yi()));
        put_hi(&mut it.b.x, x);
        put_hi(&mut it.b.y, y);
    }

    /// The throw: the thrower's sound (0x9D44), the starting point, the velocity row of
    /// `table` (second row when +$A8 is set; mirrored when the weapon faces left), a new attack
    /// `atk`, 8 frames without floor test (+$B2), the thrower lets go (+$B6 = 0), animation `anim`
    /// and flight state `sub`. Types 12..14 also clear +$25 (`clear_draw`).
    fn weapon_throw(&mut self, it: &mut Item, h: Handle, p: usize, table: u32, atk: u8, anim: u16, sub: u8, clear_draw: bool) {
        self.item_snd_9d44();
        self.weapon_throw_origin(it, p);
        let t = &self.assets.enemy.t;
        let row = table + if it.m.b(A8) != 0 { 8 } else { 0 };
        let (mut vx, mut ax, vy, ay) = (t.i16(row, 0), t.i16(row, 2), t.i16(row, 4), t.i16(row, 6));
        if it.b.facing == 0 {
            vx = vx.wrapping_neg();
            ax = ax.wrapping_neg();
        }
        it.b.vx = vx;
        it.b.ax = ax;
        it.b.vy = vy;
        it.b.ay = ay;
        it.b.vz &= 0x00FF; // move.b d0,$18(a6): only the high byte
        it.b.az &= 0x00FF; // move.b d0,$1e(a6)
        it.b.hitstop = 0;
        it.b.airborne = 1;
        self.weapon_new_attack(it, atk);
        if clear_draw {
            it.b.draw_flags = 0;
        }
        it.m.sb(B2, 8);
        it.b.wall = 0;
        self.players[p].item_type = 0;
        self.item_anim(it, anim);
        it.b.sub = sub;
        self.item_draw_check(it, h);
    }

    /// Muzzle effects of a shot: the sparks `spark` at (x, y, z) with the weapon's facing and
    /// the jittered puff `puff` 8 higher.
    fn weapon_muzzle(&mut self, it: &mut Item, h: Handle, pos: (i16, i16, i16), spark: u16, puff: u16) {
        self.item_in_slot(it, h, |g| g.fx_spawn_spark_at(h, pos.0, pos.1, pos.2, spark));
        self.fx_spawn_spark_jitter(pos.0, pos.1.wrapping_add(8), pos.2, puff);
    }

    /// 0x26414: a bullet (class 3, variant `var`) at `pos`; copies +$02, facing, +$70 and +$A9.
    fn weapon_spawn_bullet(&mut self, it: &Item, pos: (i16, i16, i16), var: u16) {
        self.weapon_spawn_shot(it, class::BULLET * 4, pos, Some(var), true);
    }

    /// 0x26454: the side effect of a shot (class 7): no variant, no +$A9.
    fn weapon_spawn_shell(&mut self, it: &Item, pos: (i16, i16, i16)) {
        self.weapon_spawn_shot(it, class::SHELL * 4, pos, None, false);
    }

    /// 0x2648A: six gun shots (class 10, variants 5..0).
    fn weapon_spawn_pellets(&mut self, it: &Item, pos: (i16, i16, i16)) {
        for var in (0..=5u16).rev() {
            if !self.weapon_spawn_shot(it, class::GUN_SHOT * 4, pos, Some(var), true) {
                break;
            }
        }
    }

    fn weapon_spawn_shot(&mut self, it: &Item, class_word: u16, pos: (i16, i16, i16), var: Option<u16>, a9: bool) -> bool {
        let Some(j) = self.items.alloc() else { return false };
        let n = &mut self.items.slots[j];
        n.b.live = 1;
        n.b.who = it.b.who;
        n.b.id = it.b.id;
        n.b.k20 = class_word;
        put_hi(&mut n.b.x, pos.0);
        put_hi(&mut n.b.y, pos.1);
        put_hi(&mut n.b.z, pos.2);
        if let Some(v) = var {
            n.b.var26 = v;
        }
        n.b.facing = it.b.facing;
        n.b.partner = it.b.partner;
        if a9 {
            n.m.sb(A9, it.m.b(A9));
        }
        true
    }

    /// 0x96B40: the muzzle flash (effect class 0x21) at the weapon, +$60 = the weapon's kind.
    fn weapon_spawn_flash(&mut self, it: &Item) {
        let pos = (it.b.xi(), it.b.yi(), it.b.zi());
        if let Some(j) = self.item_spawn_effect(0x84, pos) {
            self.effects.slots[j].b.facing = it.b.facing;
            self.effects.slots[j].owner60 = it.m.w(KIND);
        }
    }

    /// 0x6248 for a weapon: is an enemy (the list of the previous frame) in front of the weapon
    /// within the reach of the character's box?
    pub(super) fn weapon_enemy_near(&self, it: &Item) -> bool {
        self.item_enemy_near(&it.b)
    }

    /// 0x193E for an item (it keeps the object inside the play area).
    fn weapon_clamp_playfield(&mut self, it: &mut Item) {
        let mut e = Enemy::default();
        std::mem::swap(&mut e.b, &mut it.b);
        self.enemy_clamp_to_playfield(&mut e, 0);
        std::mem::swap(&mut e.b, &mut it.b);
    }
}

impl Game {
    // ------------------------------------------------------------------ held

    /// 0x25178 held (sub 8): follow the hand, then the code of the type (table 0x251C8).
    fn weapon_held(&mut self, it: &mut Item, h: Handle) {
        let Some(p) = self.weapon_owner(it, h) else {
            // 0x251A0: the player lost it: dropped (sub 0xC, act 0), no draw.
            it.b.sub = 0xC;
            it.b.act = 0;
            return;
        };
        it.b.wading = self.players[p].b.wading;
        it.b.visible = 0;
        if self.item_follow_hand(it, p) {
            return;
        }
        match it.b.var26 / 2 {
            0 => self.weapon_held_0(it, h, p),
            1 => self.weapon_held_1(it, h, p),
            2 => self.weapon_held_2(it, h, p),
            3 => self.weapon_held_3(it, h, p),
            4 => self.weapon_held_4(it, h, p),
            5 => self.weapon_held_5(it, h, p),
            6 => self.weapon_held_6(it, h, p),
            7 => self.weapon_held_7(it, h, p),
            8 => self.weapon_held_8(it, h, p),
            9 => self.weapon_held_swing(it, h, p, 0x36, 0x20, 1),
            10 => self.weapon_held_10(it, h, p),
            11 => self.weapon_held_swing(it, h, p, 0x38, 0x20, 2),
            12 => self.weapon_held_12(it, h, p),
            13 => self.weapon_held_13(it, h, p),
            14 => self.weapon_held_14(it, h, p),
            _ => {}
        }
    }

    /// Wear of a swing or shot: +$A6 += `n`.
    fn weapon_wear(it: &mut Item, n: u8) {
        it.m.sb(WEAR, it.m.b(WEAR).wrapping_add(n));
    }

    /// `+$B0 = owner +$C4` (the last seen attack flag), then draw: the end of every melee handler.
    fn weapon_held_end(&mut self, it: &mut Item, h: Handle, p: usize) {
        it.m.sb(B0, self.players[p].weapon_atk);
        self.item_draw_check(it, h);
    }

    /// The common melee prologue: a new press of the attack flag (+$C4 differs from +$B0) wears
    /// the weapon by `wear`.
    fn weapon_press_wear(&mut self, it: &mut Item, p: usize, wear: u8) {
        if self.players[p].weapon_atk ^ it.m.b(B0) != 0 {
            Self::weapon_wear(it, wear);
        }
    }

    /// The swing frame: the owner's frame byte +$2C equals `frame`: it is cleared, the attack box
    /// grows by the character, the weapon joins the attacker list and starts attack `atk`.
    fn weapon_swing_frame(&mut self, it: &mut Item, h: Handle, p: usize, frame: u8, atk: u8) -> bool {
        if self.players[p].b.param_a != frame {
            return false;
        }
        self.players[p].b.param_a = 0;
        Self::weapon_box_by_character(it);
        self.item_register_attacker(h);
        self.weapon_new_attack(it, atk);
        true
    }

    /// One shot of a gun: `+$C4 = 0` (types that clear it), the shot bookkeeping, the muzzle.
    /// Position: (+- dx, dy) from the weapon, bullet variant, spark and puff pictures.
    fn weapon_shot_pos(it: &Item, dx: i16, dy: i16) -> (i16, i16, i16) {
        let dx = if it.b.facing != 0 { dx.wrapping_neg() } else { dx };
        (it.b.xi().wrapping_add(dx), it.b.yi().wrapping_add(dy), it.b.zi())
    }

    /// Type 0 (0x251E6): rifle; as a club (kind 0x203) 0x25276.
    fn weapon_held_0(&mut self, it: &mut Item, h: Handle, p: usize) {
        if it.m.w(KIND) == 0x203 {
            // 0x25276
            if self.players[p].weapon_atk != 0 {
                self.weapon_press_wear(it, p, 0x20);
                Self::weapon_box_by_character(it);
                if self.players[p].b.param_a == 1 {
                    self.players[p].b.param_a = 0;
                    self.item_register_attacker(h);
                    self.weapon_new_attack(it, 0x32);
                }
            }
            return self.weapon_held_end(it, h, p);
        }
        if self.players[p].weapon_atk == 0 {
            return self.item_draw_check(it, h);
        }
        self.players[p].weapon_atk = 0;
        self.item_snd_9c76();
        self.weapon_spawn_flash(it);
        Self::weapon_wear(it, 0x20);
        it.b.hp = it.b.hp.wrapping_sub(1);
        if it.b.hp == 0 {
            it.m.sw(KIND, 0x203);
            self.players[p].item_type = 0x203;
            it.b.act = 0;
            it.m.sw(A2, 8);
        }
        let pos = Self::weapon_shot_pos(it, -0x28, 5);
        self.weapon_spawn_bullet(it, pos, 2);
        self.weapon_muzzle(it, h, pos, 0x10, 6);
        self.item_draw_check(it, h);
    }

    /// Type 1 (0x252D2): pistol; empty (kind 0x201) it is thrown (0x2535C).
    fn weapon_held_1(&mut self, it: &mut Item, h: Handle, p: usize) {
        if it.m.w(KIND) == 0x201 {
            return self.weapon_gun_throw(it, h, p, THROW_BASE, 0x1E);
        }
        if self.players[p].weapon_atk == 0 {
            return self.item_draw_check(it, h);
        }
        self.players[p].weapon_atk = 0;
        self.item_snd_9cd8();
        self.weapon_spawn_flash(it);
        Self::weapon_wear(it, 0x20);
        it.b.hp = it.b.hp.wrapping_sub(1);
        if it.b.hp == 0 {
            it.m.sw(KIND, 0x201);
            self.players[p].item_type = 0x201;
            it.b.act = 0;
        }
        let pos = Self::weapon_shot_pos(it, -0x14, 6);
        self.weapon_spawn_bullet(it, pos, 0);
        self.weapon_muzzle(it, h, pos, 0xF, 6);
        self.item_draw_check(it, h);
    }

    /// An empty gun is thrown (0x2535C, 0x255F2, 0x2583A): the attack flag is consumed first.
    fn weapon_gun_throw(&mut self, it: &mut Item, h: Handle, p: usize, table: u32, atk: u8) {
        if self.players[p].weapon_atk == 0 {
            return self.item_draw_check(it, h);
        }
        self.players[p].weapon_atk = 0;
        self.weapon_throw(it, h, p, table, atk, 0x10, 0x10, false);
    }

    /// Type 2 (0x253FA): knife; the owner's frame byte 1 swings, 2 throws.
    fn weapon_held_2(&mut self, it: &mut Item, h: Handle, p: usize) {
        if self.players[p].weapon_atk != 0 {
            if self.players[p].weapon_atk ^ it.m.b(B0) != 0 {
                Self::weapon_wear(it, 0x20);
                self.weapon_new_attack(it, 0x2C);
            }
            let pa = self.players[p].b.param_a;
            if pa != 0 {
                if pa == 2 {
                    it.b.palette_override = 0;
                    return self.weapon_throw(it, h, p, THROW_BASE + 0x10, 0x24, 0x10, 0x14, false);
                }
                self.item_register_attacker(h);
            }
        }
        // 0x25444
        Self::weapon_box_by_character(it);
        it.m.sb(B0, self.players[p].weapon_atk);
        if it.m.b(B81) != 0 {
            it.m.sb(B81, it.m.b(B81) - 1);
            it.b.palette_override = 0x84;
        } else {
            it.b.palette_override = 0;
            let b80 = it.m.b(B80).wrapping_sub(1);
            it.m.sb(B80, b80);
            if b80 == 0 {
                it.m.sb(B80, 5);
                if self.weapon_enemy_near(it) {
                    it.b.palette_override = 0x8A;
                    it.m.sb(B81, 5);
                }
            }
        }
        self.item_draw_check(it, h);
    }

    /// Types 3 and 4 share the rising-edge test of the owner's frame flag (+$2C): returns true
    /// when a round is fired this frame.
    fn weapon_auto_fire(&mut self, it: &mut Item, p: usize) -> bool {
        let old = it.m.b(B0);
        let now = self.players[p].b.param_a;
        it.m.sb(B0, now);
        if (old ^ now) & now == 0 {
            return false;
        }
        it.m.sb(A9, it.m.b(A9).wrapping_add(1));
        true
    }

    /// Type 3 (0x2552A): Uzi (full auto: a round on every new frame flag).
    fn weapon_held_3(&mut self, it: &mut Item, h: Handle, p: usize) {
        if it.m.w(KIND) == 0x201 {
            return self.weapon_gun_throw(it, h, p, THROW_BASE + 0x20, 0x20);
        }
        if self.players[p].weapon_atk == 0 {
            it.m.sb(A9, 0);
            return self.item_draw_check(it, h);
        }
        if !self.weapon_auto_fire(it, p) {
            return self.item_draw_check(it, h);
        }
        self.item_snd_9d9c();
        self.weapon_spawn_flash(it);
        Self::weapon_wear(it, 0x20);
        it.b.hp = it.b.hp.wrapping_sub(1);
        if it.b.hp == 0 {
            it.m.sw(KIND, 0x201);
            self.players[p].item_type = 0x201;
            it.b.act = 0;
            self.players[p].weapon_atk = 0;
        }
        let pos = Self::weapon_shot_pos(it, -0x1D, 5);
        self.weapon_spawn_bullet(it, pos, if it.b.hp & 2 != 0 { 6 } else { 4 });
        self.weapon_auto_muzzle(it, h, pos, 0x12);
        self.item_draw_check(it, h);
    }

    /// The muzzle of the automatic guns: spark `spark`, then the puff 9 or 10 (+$80 toggles).
    fn weapon_auto_muzzle(&mut self, it: &mut Item, h: Handle, pos: (i16, i16, i16), spark: u16) {
        let b = it.m.b(B80).wrapping_add(1);
        it.m.sb(B80, b);
        self.weapon_muzzle(it, h, pos, spark, 9 + (b & 1) as u16);
    }

    /// Type 4 (0x25690): M-16; empty (0x203) a club (0x25756).
    fn weapon_held_4(&mut self, it: &mut Item, h: Handle, p: usize) {
        if it.m.w(KIND) == 0x203 {
            // 0x25756
            if self.players[p].weapon_atk != 0 {
                self.weapon_press_wear(it, p, 0x20);
                self.weapon_swing_frame(it, h, p, 1, 0x34);
            }
            return self.weapon_held_end(it, h, p);
        }
        if self.players[p].weapon_atk == 0 {
            it.m.sb(A9, 0);
            return self.item_draw_check(it, h);
        }
        if !self.weapon_auto_fire(it, p) {
            return self.item_draw_check(it, h);
        }
        self.item_snd_9d9c();
        self.weapon_spawn_flash(it);
        Self::weapon_wear(it, 0x20);
        it.b.hp = it.b.hp.wrapping_sub(1);
        if it.b.hp == 0 {
            it.m.sw(KIND, 0x203);
            self.players[p].item_type = 0x203;
            it.b.act = 0;
            it.m.sw(A2, 0x18);
        }
        let pos = Self::weapon_shot_pos(it, -0x36, 7);
        self.weapon_spawn_bullet(it, pos, if it.b.hp & 1 != 0 { 0xA } else { 8 });
        self.weapon_auto_muzzle(it, h, pos, 0x12);
        self.item_draw_check(it, h);
    }

    /// Type 5 (0x257B2): shotgun, six pellets; empty (0x201) it is thrown (0x2583A).
    fn weapon_held_5(&mut self, it: &mut Item, h: Handle, p: usize) {
        if it.m.w(KIND) == 0x201 {
            return self.weapon_gun_throw(it, h, p, THROW_BASE + 0x30, 0x22);
        }
        if self.players[p].weapon_atk == 0 {
            return self.item_draw_check(it, h);
        }
        self.players[p].weapon_atk = 0;
        self.item_snd_a442(it);
        self.weapon_spawn_flash(it);
        Self::weapon_wear(it, 0x20);
        it.b.hp = it.b.hp.wrapping_sub(1);
        if it.b.hp == 0 {
            it.m.sw(KIND, 0x201);
            self.players[p].item_type = 0x201;
            it.b.act = 0;
        }
        let pos = Self::weapon_shot_pos(it, -0x2E, 7);
        self.weapon_spawn_pellets(it, pos);
        self.weapon_muzzle(it, h, pos, 0x11, 6);
        self.item_draw_check(it, h);
    }

    /// Type 6 (0x258D8): bazooka. Empty, the player lets go of it.
    fn weapon_held_6(&mut self, it: &mut Item, h: Handle, p: usize) {
        if it.b.hp == 0 {
            self.players[p].item_type = 0;
        }
        if self.players[p].weapon_atk == 0 {
            return self.item_draw_check(it, h);
        }
        self.item_snd_9e08(it);
        self.weapon_spawn_flash(it);
        Self::weapon_wear(it, 0x20);
        it.b.hp = it.b.hp.wrapping_sub(1);
        if it.b.hp == 0 {
            it.b.hp = 0;
        }
        self.players[p].weapon_atk = 0;
        let pos = Self::weapon_shot_pos(it, -0x22, 7);
        self.weapon_spawn_shell(it, pos);
        let z = pos.2.wrapping_sub(1);
        self.item_in_slot(it, h, |g| g.fx_spawn_spark_at(h, pos.0, pos.1, z, 0x13));
        let d4: i16 = if it.b.facing != 0 { -0x48 } else { 0x48 };
        self.item_in_slot(it, h, |g| g.fx_spawn_spark_at(h, pos.0.wrapping_add(d4), pos.1, z, 0x14));
        self.fx_spawn_spark_jitter(pos.0, pos.1.wrapping_add(8), z, 6);
        self.item_draw_check(it, h);
    }

    /// Type 7 (0x25968): rod (a long wooden bar); each hit may break it into the stick of type 8.
    fn weapon_held_7(&mut self, it: &mut Item, h: Handle, p: usize) {
        if self.players[p].weapon_atk != 0 {
            if self.players[p].weapon_atk ^ it.m.b(B0) != 0 {
                self.weapon_new_attack(it, 0x30);
            }
            if self.players[p].b.param_a == 1 {
                self.players[p].b.param_a = 0;
                Self::weapon_box_by_character(it);
                self.item_register_attacker(h);
            }
        }
        // 0x259B2 (also reached without the attack flag): it hit something (+$59 from the hit pass).
        if it.b.hitstop != 0 {
            it.b.hitstop = 0;
            Self::weapon_wear(it, 0x20);
            let n = it.m.b(AD).wrapping_add(1);
            it.m.sb(AD, n);
            let chance = self.assets.enemy.t.u8(HIT_CHANCE, n as u32);
            let r = (self.rand() & 0xF) as u8;
            if r < chance {
                self.weapon_break_into_stick(it, p);
            }
        }
        self.weapon_held_end(it, h, p);
    }

    /// 0x259E8: the bat breaks: a debris piece (effect class 0x2C, picture 0) and sound 0x6F / 0xF6,
    /// then it is the stick of type 8 (var 0x10, kind 0x202, new attack 0x2E).
    fn weapon_break_into_stick(&mut self, it: &mut Item, p: usize) {
        let dx: i16 = if it.b.facing != 0 { 0x18 } else { -0x18 };
        let pos = (it.b.xi().wrapping_add(dx), it.b.yi(), it.b.zi());
        if let Some(j) = self.item_spawn_effect(0x2C, pos) {
            self.effects.slots[j].b.var26 = 0;
            self.item_snd_9e60();
        }
        it.b.anim_table = 0x273FE;
        it.m.sw(KIND, 0x202);
        it.m.sw(A2, 0x28);
        it.b.var26 = 0x10;
        self.weapon_new_attack(it, 0x2E);
        self.players[p].item_type = 0x202;
        it.m.sw(LIFE, 0);
        it.m.sb(WEAR, 0);
        it.m.sb(A7, 0);
        it.m.sb(B0, 0);
    }

    /// Type 8 (0x25A82): stick; frame byte 1 swings, 2 throws.
    fn weapon_held_8(&mut self, it: &mut Item, h: Handle, p: usize) {
        if self.players[p].weapon_atk == 0 {
            return self.weapon_held_end(it, h, p);
        }
        if self.players[p].weapon_atk ^ it.m.b(B0) != 0 {
            Self::weapon_wear(it, 0x20);
            self.weapon_new_attack(it, 0x2E);
        }
        let pa = self.players[p].b.param_a;
        if pa != 0 {
            if pa == 2 {
                return self.weapon_throw(it, h, p, THROW_BASE + 0x40, 0x2A, 0x10, 0x14, false);
            }
            Self::weapon_box_by_character(it);
            self.item_register_attacker(h);
        }
        self.weapon_held_end(it, h, p)
    }

    /// Types 9 and 11 (0x25B70, 0x25C60): a swing on the owner's frame byte `frame`.
    fn weapon_held_swing(&mut self, it: &mut Item, h: Handle, p: usize, atk: u8, wear: u8, frame: u8) {
        if self.players[p].weapon_atk != 0 {
            self.weapon_press_wear(it, p, wear);
            self.weapon_swing_frame(it, h, p, frame, atk);
        }
        self.weapon_held_end(it, h, p)
    }

    /// Type 10 (0x25BCC): club, heavy weapon with two swings (frame byte 1: attack 0x58, 2: attack 0x60).
    fn weapon_held_10(&mut self, it: &mut Item, h: Handle, p: usize) {
        if self.players[p].weapon_atk != 0 {
            self.weapon_press_wear(it, p, 0x30);
            if !self.weapon_swing_frame(it, h, p, 1, 0x58) {
                self.weapon_swing_frame(it, h, p, 2, 0x60);
            }
        }
        self.weapon_held_end(it, h, p)
    }

    /// Type 12 (0x25CBC): dynamite: thrown on the owner's attack flag (animation 8, sub 0x18).
    fn weapon_held_12(&mut self, it: &mut Item, h: Handle, p: usize) {
        if self.players[p].weapon_atk == 0 {
            return self.item_draw_check(it, h);
        }
        self.weapon_throw(it, h, p, THROW_BASE + 0x50, 0x28, 8, 0x18, true);
    }

    /// Type 13 (0x25D62): grenade (animation 0x10, sub 0x1C).
    fn weapon_held_13(&mut self, it: &mut Item, h: Handle, p: usize) {
        if self.players[p].weapon_atk == 0 {
            return self.item_draw_check(it, h);
        }
        self.weapon_throw(it, h, p, THROW_BASE + 0x60, 0x26, 0x10, 0x1C, true);
    }

    /// Type 14 (0x25E08): stone (sub 0x20); also clears +$80 and +$B1.
    fn weapon_held_14(&mut self, it: &mut Item, h: Handle, p: usize) {
        if self.players[p].weapon_atk == 0 {
            return self.item_draw_check(it, h);
        }
        it.m.sb(B80, 0);
        it.m.sb(B1, 0);
        self.weapon_throw(it, h, p, THROW_BASE + 0x70, 0x1C, 0x10, 0x20, true);
    }
}

// ---------------------------------------------------------------------------------------------
// Thrown and flying weapons (sub 0x10 .. 0x28)
// ---------------------------------------------------------------------------------------------

impl Game {
    /// 0x26266: an explosion (class 1) at the weapon: who / id, position, vx, airborne flag,
    /// owner, attack id; variant 1 when +$A8 is set.
    fn weapon_spawn_blast(&mut self, it: &Item) {
        let Some(j) = self.items.alloc() else { return };
        let n = &mut self.items.slots[j];
        n.b.live = 1;
        n.b.who = it.b.who;
        n.b.id = it.b.id;
        n.b.k20 = class::BLAST * 4;
        put_hi(&mut n.b.x, it.b.xi());
        put_hi(&mut n.b.y, it.b.yi());
        put_hi(&mut n.b.z, it.b.zi());
        n.b.vx = it.b.vx;
        n.b.airborne = it.b.airborne;
        n.b.partner = it.b.partner;
        n.b.atk = it.b.atk;
        n.b.var26 = (it.m.b(A8) != 0) as u16;
    }

    /// The floor test of a flying weapon: +$B2 frames after the throw it is skipped (the weapon
    /// leaves the thrower's body), then 0x12CB4 every frame.
    fn weapon_fly_terrain(&mut self, it: &mut Item, h: Handle) {
        let b2 = it.m.b(B2);
        if b2 != 0 {
            it.m.sb(B2, b2 - 1);
            if b2 - 1 != 0 {
                return;
            }
        }
        self.item_terrain(it, h);
    }

    /// `move.w #$fe00,d0 ; tst.b $24(a6) ; bne ; neg.w d0`: the bounce speed away from a wall.
    fn weapon_wall_speed(it: &Item) -> i16 {
        if it.b.facing != 0 { -0x200 } else { 0x200 }
    }

    /// 0x25EF2 / 0x25FBE: bounce off a wall: vx back, ax 0, vy 0, ay -0x45, one bounce counted.
    /// Returns true when the 4th bounce ended the flight (state 0x24).
    fn weapon_wall_bounce(it: &mut Item) -> bool {
        it.b.vx = Self::weapon_wall_speed(it);
        it.b.ax = 0;
        it.b.vy = 0;
        it.b.ay = -0x45;
        Self::weapon_count_bounce(it, 3)
    }

    /// `addq.b #1,$b1 ; cmpi.b #limit,$b1 ; bhi`: true when the flight is over.
    fn weapon_count_bounce(it: &mut Item, limit: u8) -> bool {
        let b1 = it.m.b(B1).wrapping_add(1);
        it.m.sb(B1, b1);
        if b1 > limit {
            it.b.set_state(0x0224_0000);
            return true;
        }
        false
    }

    /// 0x25F22 / 0x25FEC: landing: the weapon hops (half the speed, vy reversed and halved).
    fn weapon_floor_bounce(it: &mut Item, limit: u8) -> bool {
        it.b.airborne = 1;
        it.b.vx >>= 1;
        it.b.vy = it.b.vy.wrapping_neg() >> 1;
        Self::weapon_count_bounce(it, limit)
    }

    /// 0x25EB6 (sub 0x10): a thrown gun, club or stick. A hit (+$59) bounces it back (it can
    /// hurt only before its first bounce: the attacker list is skipped afterwards); four
    /// bounces end the flight (sub 0x24).
    fn weapon_fly_10(&mut self, it: &mut Item, h: Handle) {
        let wall;
        if it.b.hitstop != 0 {
            it.b.hitstop = 0;
            it.b.live = 2;
            wall = true;
        } else {
            self.weapon_fly_terrain(it, h);
            wall = it.m.b(B1) == 0 && it.b.wall != 0;
        }
        if wall && Self::weapon_wall_bounce(it) {
            return;
        }
        if it.b.airborne == 0 && Self::weapon_floor_bounce(it, 3) {
            return;
        }
        self.item_anim_step(it);
        it.b.motion();
        self.item_despawn_near(it);
        if it.m.b(B1) == 0 {
            self.item_register_attacker(h);
        }
        self.item_draw_check(it, h);
    }

    /// 0x25F84 (sub 0x14): like sub 0x10 but a hit destroys the weapon (mode 6) and the animation
    /// runs only after the first bounce.
    fn weapon_fly_14(&mut self, it: &mut Item, h: Handle) {
        if it.b.hitstop != 0 {
            it.b.hitstop = 0;
            it.set_mode_sub(6, 0);
            return;
        }
        self.weapon_fly_terrain(it, h);
        if it.m.b(B1) == 0 && it.b.wall != 0 && Self::weapon_wall_bounce(it) {
            return;
        }
        if it.b.airborne == 0 && Self::weapon_floor_bounce(it, 3) {
            return;
        }
        if it.m.b(B1) != 0 {
            self.item_anim_step(it);
        }
        if it.m.b(B1) == 0 {
            self.item_register_attacker(h);
        }
        it.b.motion();
        self.item_despawn_near(it);
        self.item_draw_check(it, h);
    }

    /// 0x2606E (sub 0x18): the dynamite in flight. A hit makes it jump up; after four bounces
    /// it lies on the floor, armed (0x26158).
    fn weapon_fly_18(&mut self, it: &mut Item, h: Handle) {
        if it.b.hitstop != 0 {
            it.b.hitstop = 0;
            it.b.live = 2;
            it.b.vy = 0x500;
            it.b.ay = -0x40;
            let (vx, ax) = if it.b.facing != 0 { (0x200, 0) } else { (-0x200, 0) };
            it.b.vx = vx;
            it.b.ax = ax;
        }
        self.weapon_fly_terrain(it, h);
        if it.m.b(B1) == 0 && it.b.wall != 0 {
            it.b.vx = Self::weapon_wall_speed(it);
            it.b.ax = 0;
            it.b.ay = -0x45;
            if self.weapon_bomb_bounce(it, h, 3) {
                return;
            }
        }
        if it.b.airborne == 0 {
            it.b.airborne = 1;
            it.b.vx >>= 1;
            it.b.ax = 0;
            it.b.vy = it.b.vy.wrapping_neg() >> 1;
            if self.weapon_bomb_bounce(it, h, 3) {
                return;
            }
        }
        if it.m.b(B1) == 0 && it.b.live & 1 != 0 {
            self.item_register_attacker(h);
        }
        it.b.motion();
        self.weapon_clamp_playfield(it);
        self.item_draw_check(it, h);
    }

    /// `addq.b #1,$b1 ; cmpi.b #3,$b1 ; bhi 0x26158`: the bomb stops: it lies on the floor, armed,
    /// turned around.
    fn weapon_bomb_bounce(&mut self, it: &mut Item, h: Handle, limit: u8) -> bool {
        let b1 = it.m.b(B1).wrapping_add(1);
        it.m.sb(B1, b1);
        if b1 <= limit {
            return false;
        }
        it.b.sub = 0;
        it.b.act = 0;
        it.m.sb(B0, 0);
        self.item_snap_to_floor(it, h);
        self.weapon_arm(it);
        it.b.facing = (it.b.facing + 1) & 1;
        self.item_draw_check(it, h);
        true
    }

    /// 0x26182 (sub 0x1C): the grenade: it explodes on a hit or on the third floor contact
    /// (0x26252), after bouncing with a random depth speed (table 0x265D4).
    fn weapon_fly_1c(&mut self, it: &mut Item, h: Handle) {
        if it.b.hitstop != 0 {
            it.b.hitstop = 0;
            return self.weapon_explode(it);
        }
        self.weapon_fly_terrain(it, h);
        if it.m.b(B1) == 0 && it.b.wall != 0 {
            it.b.vx = Self::weapon_wall_speed(it);
            it.b.ax = 0;
            it.b.ay = -0x45;
        }
        if it.b.airborne == 0 {
            it.b.airborne = 1;
            let r = (self.rand() & 0x1E) as u32;
            it.b.vz = self.assets.enemy.t.i16(SPEED_Z, r);
            it.b.vy = it.b.vy.wrapping_neg();
            if it.m.b(B1) == 0 {
                it.b.ax = 0;
                it.b.vx >>= 1;
                it.b.vy >>= 2;
            }
            let b1 = it.m.b(B1).wrapping_add(1);
            it.m.sb(B1, b1);
            if b1 > 2 {
                it.m.sb(A8, 0);
                return self.weapon_explode(it);
            }
        }
        if it.m.b(B1) != 0 {
            self.item_anim_step(it);
        }
        self.item_register_attacker(h);
        it.b.motion();
        self.item_despawn_near(it);
        self.item_draw_check(it, h);
    }

    /// 0x26252: the grenade bursts: state 0x02280000 (sub 0x28: a chain of explosions) and the
    /// first explosion (0x26266).
    fn weapon_explode(&mut self, it: &mut Item) {
        it.b.set_state(0x0228_0000);
        it.m.sb(B80, 0xA);
        it.m.sb(B81, 2);
        self.weapon_spawn_blast(it);
    }

    /// 0x262BC (sub 0x20): the stone: bounces twice and then stops (state 0x24); a hit
    /// (unless +$A8) or a wall turns it around.
    fn weapon_fly_20(&mut self, it: &mut Item, h: Handle) {
        let mut turn = false;
        if it.m.b(A8) == 0 && it.b.hitstop != 0 {
            it.b.hitstop = 0;
            turn = true;
        } else {
            self.weapon_fly_terrain(it, h);
            let b80 = it.m.b(B80);
            if b80 != 0 {
                it.m.sb(B80, b80 - 1);
            } else if it.m.b(B1) == 0 && it.b.wall != 0 {
                turn = true;
            }
        }
        if turn {
            it.b.vx = Self::weapon_wall_speed(it);
            it.b.ax = 0;
            it.b.vy = 0;
            it.b.ay = -0x45;
            it.m.sb(B80, 5);
        }
        if it.b.airborne == 0 {
            it.b.airborne = 1;
            it.b.vx >>= 1;
            it.b.vy = it.b.vy.wrapping_neg() >> 1;
            let b1 = it.m.b(B1).wrapping_add(1);
            it.m.sb(B1, b1);
            if b1 > 1 {
                it.m.sb(A8, 0);
                it.b.set_state(0x0224_0000);
                return;
            }
        }
        it.b.motion();
        self.item_despawn_near(it);
        if it.m.b(B1) == 0 {
            self.item_register_attacker(h);
        }
        self.item_draw_check(it, h);
    }

    /// 0x26392 (sub 0x24): the spent weapon blinks for 30 frames and goes (odd frames drawn).
    fn weapon_fly_24(&mut self, it: &mut Item, h: Handle) {
        if it.b.act == 0 {
            it.m.sb(B80, 0x1E);
            self.item_anim(it, 0x11);
            it.b.act += 2;
        }
        let (b80, borrow) = it.m.b(B80).overflowing_sub(1);
        it.m.sb(B80, b80);
        if borrow {
            it.set_mode_sub(6, 0);
        }
        if b80 & 1 != 0 {
            self.item_draw_check(it, h);
        }
    }

    /// 0x263CC (sub 0x28): the chain of explosions of the grenade: every 10 frames one more,
    /// moved by the table 0x26408 (record +$81, two words: dx, dz).
    fn weapon_fly_28(&mut self, it: &mut Item) {
        let b80 = it.m.b(B80).wrapping_sub(1);
        it.m.sb(B80, b80);
        if b80 != 0 {
            return;
        }
        it.m.sb(B80, 0xA);
        let k = it.m.b(B81) as u32 * 4;
        let t = &self.assets.enemy.t;
        let (dx, dz) = (t.i16(CHAIN, k), t.i16(CHAIN, k + 2));
        let x = it.b.xi().wrapping_add(dx);
        let z = it.b.zi().wrapping_add(dz);
        put_hi(&mut it.b.x, x);
        put_hi(&mut it.b.z, z);
        self.weapon_spawn_blast(it);
        let b81 = it.m.b(B81).wrapping_sub(1);
        it.m.sb(B81, b81);
        if b81 == 0 {
            it.set_mode_sub(6, 0);
        }
    }
}
