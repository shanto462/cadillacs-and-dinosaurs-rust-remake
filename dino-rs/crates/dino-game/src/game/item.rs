//! Pool E: items, food, treasure, weapons, ammo, containers and projectiles (update 0x233C6,
//! allocator 0x4818, free 0x48D4). Owner: enemy port. Spec: `re/specs/enemy/SPEC.md` section 13.
//!
//! Pool E has 24 objects of 0xC0 bytes at `$874(a5)`. Each frame `pool_e_update` (0x233C6, called
//! after the players and before pool A) runs every live slot in slot order and dispatches on the
//! class word +$20 (table 0x233F0, 12 classes):
//!
//! | class (+$20) | handler | object | module |
//! |---|---|---|---|
//! | 0 (0x00) | 0x24D44 | weapon (15 types, +$26 = type * 2) | `weapon` |
//! | 1 (0x04) | 0x27E16 | explosion (+$26 = kind 0..4) | `blast` |
//! | 2 (0x08) | 0x283DE | ammo box | `small` |
//! | 3 (0x0C) | 0x2860A | bullet of a held gun | `blast` |
//! | 4 (0x10) | 0x2898C | food (+$26 = 1..18) | `small` |
//! | 5 (0x14) | 0x28C76 | container (+$26 = variant 0..0xC) | `container` |
//! | 6 (0x18) | 0x29B06 | overlay of a held weapon (type 11) | `blast` |
//! | 7 (0x1C) | 0x29D1E | shell / spark of a gun | `blast` |
//! | 8 (0x20) | 0x29EC4 | ballistic projectile | `blast` |
//! | 9 (0x24) | 0x2A028 | treasure (+$26 = 1..16) | `small` |
//! | 10 (0x28) | 0x2A39A | gun projectile variant | `blast` |
//! | 11 (0x2C) | 0x2A534 | invisible attack box | `blast` |
//!
//! # How the items behave (all classes)
//! * **mode** (+$04): 0 init, 2 alive, 4 break (containers) / free (others), 6 free next frame.
//! * **sub** (+$05, multiples of 4) while alive: 0 lying on the ground, 4 being lifted by a player
//!   (`+$70` = player, the player is in its pick-up state 0x02000014), 8 held, 0xC dropped (falls
//!   and lands), 0x10 and up: thrown / flying (per class and type).
//! * **act** (+$06): 2 at spawn = "spawned airborne" (a drop: it hops out of its source).
//! * **kind** (+$A0): 0x00tt food, 0x0100 ammo box, 0x02kk weapon or container, 0x03tt treasure.
//!   The player compares this word with its own held kind (+$B6).
//!
//! # API for the player port (task 3: find_pickup 0x5AA2, pickup 0x19832, held weapons)
//! * **The item list `$69EE` / `$69F0`**: [`Game::item_list`] = `combat.get(0x69EE)`. Items that
//!   are alive (+$00 == 1) register themselves in their update (0x4AC2) through
//!   [`Game::item_register_pickup`]. The list is cleared in `objects_update` *after* the players
//!   ran, so `find_pickup` sees the list of the previous frame's pool E update, as in the original.
//!   The newest registration comes first (the original's stack grows downwards).
//! * **Candidate test** (0x5AA2): an item qualifies when `live == 1`, `mode == 2`, `sub == 0`; the
//!   player's (X, Z) lies inside the item's first hit box and |dY| <= 8. Read the item with
//!   `g.body(h)` (x, y, z, `box_table`, `attack_box`, `facing`, `mode`, `sub`, `live`) and its
//!   kind with [`Game::item_kind`] (+$A0), its variant with `body.var26` (+$26).
//! * **Pick up** (0x19858..0x1986C): [`Game::item_pickup_begin`] (`+$02` = player's who and id,
//!   `+$05` = 4, `+$06` = 0, `+$23` = 0, `+$70` = player). The player then stays in state
//!   0x02000014; the item follows the hand and becomes held when the player's frame byte +$2C is 1
//!   (the item reads it from the player: `Body::param_a`).
//! * **Replace a held item** (0x1987E..0x1988A): [`Game::item_drop_held`] puts the old one in
//!   its dropped state (`+$05` = 0xC).
//! * **Held weapons** read the player each frame: `+$B4` / `+$B6` (held item and its kind,
//!   `Player::item` / `item_type`), `+$C4` (attack flag, `Player::weapon_atk`), `+$C5` (drop
//!   request, `Player::drop`), `+$2C` (frame byte, `Body::param_a`), `+$A8` (pressed buttons) and
//!   the player's facing. They write `+$B6 = 0` of the player when thrown or broken, and set
//!   `+$C4 = 0` (an attack frame was used). Everything the item does to the player is in this
//!   module (`weapon.rs`); the player port keeps its state machine and these fields.
//! * **Eating food and ammo** are done by the item itself on the first frame of the pick-up
//!   (`small.rs`): heal (`Body::hp` of the player, then `hud_life_bar`), score, HUD message.
//! * **Drops** from enemies: [`Game::item_drop_from_enemy`] (0x24B36), from containers 0x24B48.

use super::body::Body;
use super::enemy::{Enemy, Ram};
use super::{Game, Handle, Slot, slot_of};

mod blast;
mod container;
mod hits;
mod shared;
mod small;
mod weapon;

/// Item object classes (+$20 / 4).
pub mod class {
    pub const WEAPON: u16 = 0;
    pub const BLAST: u16 = 1;
    pub const AMMO: u16 = 2;
    pub const BULLET: u16 = 3;
    pub const FOOD: u16 = 4;
    pub const CONTAINER: u16 = 5;
    pub const OVERLAY: u16 = 6;
    pub const SHELL: u16 = 7;
    pub const PROJECTILE: u16 = 8;
    pub const TREASURE: u16 = 9;
    pub const GUN_SHOT: u16 = 10;
    pub const ATTACK_BOX: u16 = 11;
}

/// Offsets beyond the shared header (`Body`) that pool E code uses.
pub mod o {
    /// +$5C explosive flag (container / weapon spawned with it: 1 or 2), effect variant byte.
    pub const W5C: usize = 0x5C;
    /// +$80 / +$81 timers and counters (byte each): blink, bounce count, shimmer.
    pub const B80: usize = 0x80;
    pub const B81: usize = 0x81;
    /// +$96 drop row (written by the spawn record, cleared by the roll).
    pub const DROP: usize = 0x96;
    /// +$A0 kind word, +$A2 held-position / ammo table offset.
    pub const KIND: usize = 0xA0;
    pub const A2: usize = 0xA2;
    /// +$A4 frames left before an item on the floor disappears (0 = never), +$A6 wear, +$A7.
    pub const LIFE: usize = 0xA4;
    pub const WEAR: usize = 0xA6;
    pub const A7: usize = 0xA7;
    /// +$A8 (kind of the last pickup press), +$A9 / +$AA / +$AD counters (weapons).
    pub const A8: usize = 0xA8;
    pub const A9: usize = 0xA9;
    pub const AA: usize = 0xAA;
    /// +$AC "something stands on me" flag set by the obstacle test (0x12E9A); cleared by the
    /// registration (0x4A96).
    pub const AC: usize = 0xAC;
    pub const AD: usize = 0xAD;
    pub const AE: usize = 0xAE;
    pub const AF: usize = 0xAF;
    /// +$B0 / +$B1 / +$B2 / +$B4 / +$B6..: per class scratch (see the class modules).
    pub const B0: usize = 0xB0;
    pub const B1: usize = 0xB1;
    pub const B2: usize = 0xB2;
    pub const B4: usize = 0xB4;
    pub const B6: usize = 0xB6;
    pub const B7: usize = 0xB7;
    pub const B8: usize = 0xB8;
    pub const B9: usize = 0xB9;
}
use o::*;

#[derive(Clone, Debug, Default)]
pub struct Item {
    pub b: Body,
    pub m: Ram,
}

/// `move.w dN,$8(a0)`: writes the integer word of a 16.16 position, keeping the fraction.
pub fn put_hi(v: &mut i32, w: i16) {
    *v = (*v & 0xFFFF) | ((w as i32) << 16);
}

impl Item {
    /// `move.w #$xxyy,$4(a6)`: mode and sub in one word (act / step untouched).
    pub fn set_mode_sub(&mut self, mode: u8, sub: u8) {
        self.b.mode = mode;
        self.b.sub = sub;
    }
    /// `clr.w $6(a6)`: act and step.
    pub fn clear_act(&mut self) {
        self.b.act = 0;
        self.b.step = 0;
    }
}

impl Game {
    // ------------------------------------------------------------------ pool E update

    /// 0x233C6 pool E update: every live slot in slot order, dispatch on +$20 (table 0x233F0).
    pub fn pool_e_update(&mut self) {
        self.spawn.g.b4fa = 0; // 0xE934: clr.b $4fa(a5), between the players and pool E
        if let Some(ram) = self.spawn.g.step_players.take() {
            self.trace_load_players(&ram); // harness only: the players as MAME had them this frame
        }
        for i in 0..self.items.slots.len() {
            if self.items.slots[i].b.live == 0 {
                continue;
            }
            let h = self.items.handle(i);
            let mut it = std::mem::take(&mut self.items.slots[i]);
            match it.b.k20 / 4 {
                class::WEAPON => self.item_weapon(&mut it, h, i),
                class::BLAST => self.item_blast(&mut it, h, i),
                class::AMMO => self.item_ammo(&mut it, h, i),
                class::BULLET => self.item_bullet(&mut it, h, i),
                class::FOOD => self.item_food(&mut it, h, i),
                class::CONTAINER => self.item_container(&mut it, h, i),
                class::OVERLAY => self.item_overlay(&mut it, h, i),
                class::SHELL => self.item_shell(&mut it, h, i),
                class::PROJECTILE => self.item_projectile(&mut it, h, i),
                class::TREASURE => self.item_treasure(&mut it, h, i),
                class::GUN_SHOT => self.item_gun_shot(&mut it, h, i),
                class::ATTACK_BOX => self.item_attack_box(&mut it, h, i),
                _ => self.item_unported(&mut it, h, i),
            }
            self.items.slots[i] = it;
        }
    }

    /// A class word outside the table 0x233F0 (the 68000 would jump into data). Kept as the
    /// snapshot had it in the harness, otherwise freed.
    fn item_unported(&mut self, it: &mut Item, h: Handle, i: usize) {
        if self.trace_hold(h, &it.b) {
            return;
        }
        self.item_free(it, i);
    }

    /// Runs `f` with the item put back in its slot (routines of other modules read the item by
    /// its handle: HUD messages, hit sparks). `it` is the object the update took out of the pool.
    pub(super) fn item_in_slot<R>(&mut self, it: &mut Item, h: Handle, f: impl FnOnce(&mut Game) -> R) -> R {
        let i = self.items.index_of(h).expect("pool E handle");
        std::mem::swap(&mut self.items.slots[i], it);
        let r = f(self);
        std::mem::swap(&mut self.items.slots[i], it);
        r
    }

    // ------------------------------------------------------------------ allocate / free

    /// 0x48D4 pool E free: pushes the slot back and clears +$00..+$27 only. Everything above
    /// +$28 keeps its stale value (the next user of the slot may read it).
    pub fn item_free(&mut self, it: &mut Item, i: usize) {
        super::spawn::clear_header(&mut it.b);
        self.items.release(i);
    }

    /// 0x4818 + the common spawn header of the drop routines: a new live object of class
    /// `class * 4` at (x, y, z) (integer words, fractions stay 0), act = 2 (spawned airborne).
    /// `None` = pool empty (the 68000 carry): the drop is lost.
    fn item_spawn(&mut self, class_word: u16, pos: (i16, i16, i16), var: Option<u16>) -> Option<usize> {
        let j = self.items.alloc()?;
        let n = &mut self.items.slots[j];
        n.b.live = 1;
        n.b.k20 = class_word;
        put_hi(&mut n.b.x, pos.0);
        put_hi(&mut n.b.y, pos.1);
        put_hi(&mut n.b.z, pos.2);
        if let Some(v) = var {
            n.b.var26 = v;
        }
        n.b.act = 2;
        Some(j)
    }

    // ------------------------------------------------------------------ per-frame lists

    /// 0x4AC2: an alive item (+$00 == 1) joins the pickup list `$69EE` (read by `find_pickup`).
    pub fn item_register_pickup(&mut self, h: Handle, live: u8) {
        if live == 1 {
            self.combat.push(0x69EE, h);
        }
    }

    /// 0x4B80: an alive item (+$00 == 1) joins the list of objects players and enemies can hit
    /// (`$6AFC`, read by the passes 0x11F86, 0x1201C, 0x120D2).
    pub fn item_register_hurtable(&mut self, h: Handle, live: u8) {
        if live == 1 {
            self.combat.push(0x6AFC, h);
        }
    }

    /// 0x4BDA: the object joins the attacker list `$671A` (the passes check frame class +$2E == 4).
    pub fn item_register_attacker(&mut self, h: Handle) {
        self.combat.push(0x671A, h);
    }

    #[allow(dead_code)] // called by the player port (task 3)
    /// The pickup list `$69EE`, newest registration first (`find_pickup` walks it from the top).
    pub fn item_list(&self) -> &[Handle] {
        self.combat.get(0x69EE)
    }

    #[allow(dead_code)] // called by the player port (task 3)
    /// +$A0 of an item: its kind word (0x00tt food, 0x0100 ammo, 0x02kk weapon / container, 0x03tt
    /// treasure). 0 for anything that is not a pool E object.
    pub fn item_kind(&self, h: Handle) -> u16 {
        match slot_of(h) {
            Some(Slot::Item(i)) => self.items.slots[i].m.w(KIND),
            _ => 0,
        }
    }

    #[allow(dead_code)] // called by the player port (task 3)
    /// 0x19858..0x1986C (player pickup 0x19832): the player `player` starts to lift `item`.
    /// `who` / `id` (+$02 word) are copied from the player, state sub 4, act 0, no palette
    /// override, `+$70` = player.
    pub fn item_pickup_begin(&mut self, item: Handle, player: Handle) {
        let (who, id) = {
            let p = self.body(player);
            (p.who, p.id)
        };
        let b = self.body_mut(item);
        b.who = who;
        b.id = id;
        b.sub = 4;
        b.act = 0;
        b.step = 0;
        b.palette_override = 0;
        b.partner = player;
    }

    #[allow(dead_code)] // called by the player port (task 3)
    /// 0x1987E..0x1988A: the old held item of a player that picks up a new weapon is dropped
    /// (`+$05` = 0xC: it falls and lands).
    pub fn item_drop_held(&mut self, item: Handle) {
        if let Some(Slot::Item(i)) = slot_of(item) {
            self.items.slots[i].b.sub = 0xC;
        }
    }

    // ------------------------------------------------------------------ draw and range checks

    /// 0x49CA for an item taken out of its slot.
    pub(super) fn item_draw_check(&mut self, it: &mut Item, h: Handle) {
        self.enemy_draw_check(&mut it.b, h);
    }

    /// 0x173C: free next frame (+$04 = 6, mode byte only) when far off screen. In the vertical
    /// scene (G.$4DC, stage 7 elevator) the world position is tested instead of the camera.
    pub(super) fn item_despawn_far(&self, it: &mut Item) {
        if self.w.vertical_scene != 0 {
            return self.item_despawn_vertical(it);
        }
        let d = it.b.xi().wrapping_sub(self.cam.s2.cam_x as i16).wrapping_add(0xC0) as u16;
        if d >= 0x300 {
            it.b.mode = 6;
        }
    }

    /// 0x1760: the narrower variant (weapons and ammo boxes on the floor).
    pub(super) fn item_despawn_near(&self, it: &mut Item) {
        if self.w.vertical_scene != 0 {
            return self.item_despawn_vertical(it);
        }
        let d = it.b.xi().wrapping_sub(self.cam.s2.cam_x as i16).wrapping_add(0x40) as u16;
        if d >= 0x240 {
            it.b.mode = 6;
        }
    }

    /// 0x1782: vertical scene: |X| and Z + Y limits.
    fn item_despawn_vertical(&self, it: &mut Item) {
        if (it.b.xi().wrapping_add(0x80) as u16) >= 0x280
            || (it.b.zi().wrapping_add(it.b.yi()).wrapping_add(0x80) as u16) >= 0x200
        {
            it.b.mode = 6;
        }
    }

    /// 0x3140C: terrain collision of an item (it skips the obstacle test), then stand on the
    /// floor and clear the airborne flag.
    pub(super) fn item_snap_to_floor(&mut self, it: &mut Item, h: Handle) {
        self.item_terrain(it, h);
        let f = it.b.floor.wrapping_add(it.b.floor2);
        put_hi(&mut it.b.y, f);
        it.b.airborne = 0;
    }

    /// 0x12CB4 for a pool E object.
    pub(super) fn item_terrain(&mut self, it: &mut Item, h: Handle) {
        self.terrain_collide(&mut it.b, h, true);
    }

    // ------------------------------------------------------------------ drops

    /// 0x24B36: item drop of an enemy whose attack record or death asked for it: row `+$96` of
    /// table 0xB6D12. Weapons, food, treasure or an ammo box appear at the enemy.
    pub fn item_drop_from_enemy(&mut self, e: &mut Enemy) {
        let pos = (e.b.xi(), e.b.yi(), e.b.zi());
        let mut row = e.m.b(super::enemy::off::DROP);
        self.item_drop_roll(pos, &mut row, 0xB6D12);
        e.m.sb(super::enemy::off::DROP, row);
    }

    /// 0x24B48: item drop of a container (table 0xB64F2). `it` is the container (out of its slot).
    pub(super) fn item_drop_from_container(&mut self, it: &mut Item) {
        let pos = (it.b.xi(), it.b.yi(), it.b.zi());
        let mut row = it.m.b(DROP);
        self.item_drop_roll(pos, &mut row, 0xB64F2);
        it.m.sb(DROP, row);
    }

    /// 0x24B36 / 0x24B48 / 0x24B56: row `*drop` of `table` (32 bytes per row), one `rand()`
    /// picks the byte. The row byte is cleared first (an object drops once). Codes: 0 nothing,
    /// 0x0F "ammo box if a gun lies on screen, else the fallback treasure row", 1..0x13 weapon
    /// (type code - 1, var = type * 2), 0x14..0x27 food (var code - 0x13), 0x28.. treasure (var
    /// code - 0x27). A full pool loses the drop. The new item has act = 2.
    fn item_drop_roll(&mut self, pos: (i16, i16, i16), drop: &mut u8, table: u32) {
        let mut d0 = *drop;
        if d0 == 0 {
            return;
        }
        let mut table = table;
        loop {
            *drop = 0;
            let row = (d0 as u32) << 5;
            let code = self.rand() & 0x1F;
            let code = self.assets.enemy.t.u8(table, row + code as u32);
            match code {
                0 => return,
                0x0F => {
                    if self.item_gun_on_screen() {
                        self.item_spawn(class::AMMO * 4, pos, None); // 0x24C8A: no var
                        return;
                    }
                    table = 0xB64D2; // fallback row 0: six treasures
                    d0 = 0;
                }
                1..=0x13 => {
                    self.item_spawn(class::WEAPON * 4, pos, Some((code as u16 - 1) * 2));
                    return;
                }
                0x14..=0x27 => {
                    self.item_spawn(class::FOOD * 4, pos, Some(code as u16 - 0x13));
                    return;
                }
                _ => {
                    self.item_spawn(class::TREASURE * 4, pos, Some(code as u16 - 0x27));
                    return;
                }
            }
        }
    }

    /// 0x24C3A..0x24CB4: a gun (weapon class, kind 0x02xx, type flag 0x24CC6) that is alive
    /// and whose X + 0x18 is not left of the camera lies in pool E.
    fn item_gun_on_screen(&self) -> bool {
        let t = &self.assets.enemy.t;
        self.items.slots.iter().any(|g| {
            if g.b.live == 0 || g.b.k20 != 0 || g.m.w(KIND) & 0xFF00 != 0x200 {
                return false;
            }
            if t.u16(0x24CC6, (g.b.var26 & 0x1E) as u32) == 0 {
                return false;
            }
            // addi.w #$18,d1 ; sub.w cam,d1 ; bcs: skip when the sum is below the camera X.
            g.b.xi().wrapping_add(0x18) as u16 >= self.cam.s2.cam_x
        })
    }

    // ------------------------------------------------------------------ small shared helpers

    /// `moveq #$24,d0 ; jmp $10036` (0x29858): +500 points for the player in +$02.
    pub(super) fn item_score_500(&mut self, who: u8) {
        let v = self.assets.text.u32(0x10292, 0x24);
        self.hud_add_score_to(who, v);
    }

    /// A pool C object (effect) with the common header every item spawn site writes: live,
    /// class word, integer X / Y / Z. Returns the slot (`None` = pool C empty, the carry).
    pub(super) fn item_spawn_effect(&mut self, class_word: u16, pos: (i16, i16, i16)) -> Option<usize> {
        let j = self.effects.alloc()?;
        let e = &mut self.effects.slots[j];
        e.b.live = 1;
        e.b.k20 = class_word;
        put_hi(&mut e.b.x, pos.0);
        put_hi(&mut e.b.y, pos.1);
        put_hi(&mut e.b.z, pos.2);
        Some(j)
    }
}

#[cfg(test)]
mod tests;
