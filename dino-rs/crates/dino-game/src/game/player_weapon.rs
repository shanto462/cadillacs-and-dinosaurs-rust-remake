//! Held weapons and pick-up of the players (phase 1 task 3). Owner: player port. A child module of
//! `player` (it uses the private helpers of the state machine). The item side is `item/weapon.rs`;
//! the API of the two sides is documented at the top of `item.rs`.
//!
//! # How the two sides work together
//! The player keeps the state machine, the item runs the weapon. They talk through fields that
//! are set and read inside one frame (the players run before pool E, `objects_update` 0xE90E):
//! * `+$B4` / `+$B6` (`item` / `item_type`): the held item and its kind word (0x2xx). The item
//!   changes `+$B6` itself: 0 when it is thrown or broken, a new kind when a gun runs empty
//!   (0x203 club, 0x201 throwable). The player drops the held item by clearing `+$B6` (and sets
//!   `+$C5`, the drop request, which the dropped item copies into its own `+$AE`).
//! * `+$C4` (`weapon_atk`): "the player is in a weapon attack": set by every weapon state, cleared
//!   by the idle entry, and by the item when it used it (a shot).
//! * `+$2C` (`param_a`): the frame byte of the swing or throw frame (1 or 2, cleared by the item).
//! * `+$B8` / `+$BA` (`cand` / `cand_type`): the pick-up candidate of `find_pickup`.
//! * `+$F8` (`pickup_cd`): 40 frames after a pick-up weapons are not picked up again.
//! * `+$10F` (`b10f`): an ammo box that found no gun to refill (set by the item) waits for the next gun.
//! * `+$EA` (`aim`): the nearest enemy, the target of the bullets.
//!
//! # The split body (`+$25` bit 0)
//! A player that holds a weapon walks, runs and fires with two animations: the upper body (the
//! main frame, `+$28`, set 5 animations of the weapon pose tables) and the legs (a second layer
//! `+$30..+$3F`: `frame2`, `w34` = the frame bytes +4 / +5, `w36` = frame bytes +6 / +7 with the
//! timer in `+$37`, `dx3c` / `dy3e` = where the upper body is drawn, from the offset table
//! `offs_table` = `+$38` indexed by `+$35`). The layer is started by 0x1506 and stepped by 0x1542;
//! the drawing is `draw.rs` `draw_split` (0x14136).
//!
//! # Not ported here
//! * The dust of the recoil (0x92E5E, pool C class 0x1C): effects port.
//! * The score extra life pose (states 0x18 / 0x1A, `+$10E`), the two-player tie-up (0x6866,
//!   state 0x1A, run state 4) and the second half of the heavy object code (0x1B0F6..): not weapons.

use super::{ATK, JMP};
use crate::game::item::o as io;
use crate::game::{Game, Slot, slot_of};
use dino_assets::FrameId;
use dino_assets::enemy::ByteTables;

/// 0x5BDC / 0x5C36: idle pose tables.
const IDLE_POSE: u32 = 0x5C36;
/// 0x5D1C (split-body offset tables), 0x5D2C (walk pose tables).
const WALK_OFFSETS: u32 = 0x5D1C;
const WALK_POSE: u32 = 0x5D2C;
/// 0x5EDE: swing pose tables.
const SWING_POSE: u32 = 0x5EDE;
/// 0x5FF2: gun walk pose tables.
const GUN_WALK_POSE: u32 = 0x5FF2;
/// 0x1A15C: run pose by character and weapon class, 0x1A164: split-body offset table pointers of
/// the run pose, 0x1A228: run swing pose by character.
const RUN_POSE: u32 = 0x1A15C;
const RUN_OFFSETS: u32 = 0x1A164;
const RUN_SWING_POSE: u32 = 0x1A228;

/// A word of an extracted table, `None` when the address is in none of them (a stale pointer).
fn table_word(t: &ByteTables, addr: u32) -> Option<u16> {
    t.blobs.iter().find(|(s, d)| addr >= *s && ((addr - s) as usize) + 1 < d.len()).map(|(s, d)| {
        let o = (addr - s) as usize;
        u16::from_be_bytes([d[o], d[o + 1]])
    })
}

impl Game {
    // ---------------------------------------------------------------------------------------
    // find_pickup 0x5AA2
    // ---------------------------------------------------------------------------------------

    /// 0x5AA2 find_pickup: the pick-up candidate in `+$B8` / `+$BA` (0x8000 = none). Walks the item
    /// list of the previous frame (newest first) and keeps the **lowest** kind (+$A0) of the items
    /// that lie on the floor (live 1, mode 2, sub 0), whose first hit box holds the player's X and Z
    /// and whose height is within 16 of `Y - 8`. A heavy object (0x20A, except variants 6 and 8)
    /// needs the stick pushed towards it; weapons (kind high byte 2) are skipped for 40 frames
    /// after a pick-up (+$F8). Nothing is found in the air or while G.$506 is set (0x5AAC).
    pub(super) fn player_find_pickup(&mut self, i: usize) {
        {
            let p = &mut self.players[i];
            p.cand_type = 0x8000;
            p.cand = 0;
        }
        if self.w.no_clamp != 0 || self.players[i].b.airborne != 0 {
            return;
        }
        let (px, py, pz, stick, cooldown) = {
            let p = &self.players[i];
            (p.b.xi(), p.b.yi(), p.b.zi(), p.held, p.pickup_cd)
        };
        for h in self.item_list().to_vec() {
            let b = self.body(h);
            if b.live != 1 || b.mode != 2 || b.sub != 0 {
                continue;
            }
            // 0x5AEA: the first hit box of the item (the attack box index +$49 of its frame).
            let Some(bx) = self.assets.sprites.hitboxes.try_single(b.box_table & 0xFF_FFFF, b.attack_box) else { continue };
            // Z: the player's depth lies inside the box.
            let d0 = b.zi().wrapping_add(bx.zoff).wrapping_sub(pz).wrapping_add(bx.zsize);
            if d0 as u16 > bx.zsize as u16 {
                continue;
            }
            // X: the box is mirrored when the item faces right (+$24 != 0).
            let (mut d1, d2) = (bx.xoff, bx.xsize);
            if b.facing != 0 {
                d1 = d1.wrapping_neg().wrapping_sub(d2);
            }
            let d0 = b.xi().wrapping_add(d1).wrapping_add(d2).wrapping_sub(px);
            if d0 as u16 > d2 as u16 {
                continue;
            }
            // Y: the player's height is 0..16 above the item's top (Y - 8).
            let d0 = b.yi().wrapping_sub(8).wrapping_sub(py).wrapping_add(0x10);
            if d0 as u16 > 0x10 {
                continue;
            }
            let kind = self.item_kind(h);
            if kind == 0 {
                continue;
            }
            if kind == 0x20A && b.var26 != 6 && b.var26 != 8 {
                // 0x5B68: the stick must push towards the object (X first, then Z; compares unsigned).
                if stick & 0xF == 0 {
                    continue;
                }
                let x_ok = if (px as u16) >= (b.xi() as u16) { stick & 2 != 0 } else { stick & 1 != 0 };
                let z_ok = if (pz as u16) >= (b.zi() as u16) { stick & 4 != 0 } else { stick & 8 != 0 };
                if !(x_ok || z_ok) {
                    continue;
                }
            }
            if kind > self.players[i].cand_type {
                continue; // `bhi`: only an equal or lower kind replaces the candidate
            }
            if cooldown != 0 && kind >> 8 == 2 {
                continue;
            }
            let p = &mut self.players[i];
            p.cand_type = kind;
            p.cand = h;
        }
    }

    // ---------------------------------------------------------------------------------------
    // Pick-up state 0x14: entry 0x19832, handler 0x198B4
    // ---------------------------------------------------------------------------------------

    /// 0x9C20: the landing voice without the wading test, used as the pick-up voice (the voice
    /// table 0x9C72 by the voice bank +$118; the pan from the X position is not recorded).
    fn player_snd_pickup(&mut self, i: usize) {
        let voice = self.players[i].voice as u32;
        let code = self.assets.text.u8(0x9C72, voice);
        self.sound(code as u16);
    }

    /// 0x19832 pick-up entry (falls through into the handler). The candidate of `find_pickup` gets
    /// `+$02` = the player, sub 4 ("being lifted"), `+$70` = the player; a weapon or heavy object
    /// (kind high byte 2) becomes the held item and the old one is dropped (sub 0xC).
    pub(super) fn player_pickup_entry(&mut self, i: usize) {
        self.players[i].b.set_state(0x0200_0014);
        self.player_snd_pickup(i);
        let anim = if self.players[i].cand_type == 0x20A { 0x16 } else { 0xA };
        self.player_panim(i, 0, anim);
        self.players[i].b.draw_flags = 0;
        let me = Game::player_handle(i);
        let (cand, cand_type) = (self.players[i].cand, self.players[i].cand_type);
        self.item_pickup_begin(cand, me); // 0x19858..0x1986C
        if cand_type >> 8 == 2 {
            if self.players[i].item_type != 0 {
                let old = self.players[i].item;
                self.item_drop_held(old); // 0x1987E: the old weapon falls
                self.players[i].drop = 0;
            }
            let p = &mut self.players[i];
            p.item = p.cand;
            p.item_type = p.cand_type;
            if p.b10f != 0 {
                // 0x198A0: an ammo box that found no gun refills this one at once (0x284E2).
                let who = self.players[i].b.who;
                self.item_ammo_refill(i, who);
            }
        }
        self.player_pickup(i);
    }

    /// 0x198B4: the lift animation. At its end the player is invulnerable for a moment
    /// (`INV[8 + character]`), may not pick up a weapon for 40 frames, and goes back to idle.
    pub(super) fn player_pickup(&mut self, i: usize) {
        if !self.player_pstep(i) {
            return;
        }
        let p = &mut self.players[i];
        p.dtap = 0;
        p.pickup_cd = 0x28;
        p.b.live = 2;
        self.player_set_invuln(i, 8);
        self.player_idle_entry(i);
    }

    // ---------------------------------------------------------------------------------------
    // Weapon attack state 0x16: entry 0x198DC, handler 0x19954
    // ---------------------------------------------------------------------------------------

    /// The ammo of the held item (+$6C of +$B4). A pointer that is not an item reads as 0 (the
    /// 68000 would read the ROM vector table there, whose high word is 0).
    fn player_item_ammo(&self, i: usize) -> u16 {
        match slot_of(self.players[i].item) {
            Some(Slot::Item(j)) => self.items.slots[j].b.hp,
            _ => 0,
        }
    }

    /// `clr.b $a8(a0)` / `move.b #1,$a8(a0)` on the held item (+$A8: the item threw or swung
    /// because of a run attack). A pointer that is not an item writes into the ROM: no effect.
    fn player_set_item_a8(&mut self, i: usize, v: u8) {
        if let Some(Slot::Item(j)) = slot_of(self.players[i].item) {
            self.items.slots[j].m.sb(io::A8, v);
        }
    }

    /// 0x198DC weapon attack entry (the attack button with a weapon in the hand): guns, the heavy
    /// weapon and the bazooka have their own states; everything else (clubs, throwables, containers)
    /// starts state 0x16 whose swing or throw frames the item reads from +$2C.
    pub(super) fn player_weapon_attack_entry(&mut self, i: usize) {
        match self.players[i].item_type {
            0x206 | 0x207 => return self.player_gun_entry(i),
            0x209 => return self.player_recoil_entry(i),
            0x203 => return self.player_charge_entry(i),
            _ => {}
        }
        self.players[i].b.set_state(0x0200_0016);
        self.players[i].b.draw_flags = 0;
        self.player_swing_pose(i); // 0x5E5C
        self.player_set_item_a8(i, 0);
        if self.players[i].item_type == 0x202 {
            // 0x19928: a club swings only at an enemy in reach, otherwise the player takes the
            // pose of a throw (the pose of kind 0x201) and the club is thrown.
            let near = self.item_enemy_near(&self.players[i].b); // 0x6248
            if !near {
                self.players[i].item_type = 0x201;
                self.player_swing_pose(i);
                self.players[i].item_type = 0x202;
            }
        }
        let p = &mut self.players[i];
        p.weapon_atk = 1;
        p.b.hitstop = 0;
        p.action = 0x10;
        self.player_weapon_attack(i);
    }

    /// 0x19954 state 0x16. At the end of the animation an empty weapon (no ammo, not a club) is let go.
    pub(super) fn player_weapon_attack(&mut self, i: usize) {
        if self.player_special_check(i) {
            return;
        }
        if self.player_hitstop_wait(i) {
            return;
        }
        if !self.player_pstep(i) {
            return;
        }
        if self.players[i].item_type != 0x202 && self.player_item_ammo(i) == 0 {
            let p = &mut self.players[i];
            p.item_type = 0;
            p.drop = 0;
        }
        self.player_idle_entry(i);
    }

    // ---------------------------------------------------------------------------------------
    // Guns (types 0x206 / 0x207): standing fire 0x1C (0x19AB0), walking fire 0x1E (0x19B60)
    // ---------------------------------------------------------------------------------------

    /// 0x64F6: the nearest enemy in front of the player (the list of last frame `$69BA`, enemies
    /// that are alive and in sub 0, on the side the player faces) by depth distance, at most 0x100.
    /// An equal distance takes the later entry of the list. The target is `+$EA`.
    fn player_aim_nearest(&mut self, i: usize) {
        let (px, pz, face) = {
            let b = &self.players[i].b;
            (b.xi() as u16, b.zi(), b.facing)
        };
        let mut best: u16 = 0x100;
        for a1 in self.combat.get(0x69BA).to_vec() {
            let e = self.body(a1);
            if e.live & 1 == 0 || e.sub != 0 {
                continue;
            }
            let ex = e.xi() as u16;
            // facing left: the enemy lies left (X below the player's), facing right: at or right of it
            if (face == 0) != (ex < px) {
                continue;
            }
            let dz = e.zi().wrapping_sub(pz);
            let dz = if dz < 0 { dz.wrapping_neg() } else { dz } as u16;
            if best < dz {
                continue;
            }
            best = dz;
            self.players[i].aim = a1;
        }
    }

    /// 0x19AB0 gun entry: aim, then standing fire or, with a direction held, walking fire.
    pub(super) fn player_gun_entry(&mut self, i: usize) {
        self.players[i].aim = 0;
        self.player_aim_nearest(i);
        self.players[i].set_b81(3);
        if self.players[i].held & 0xF != 0 {
            return self.player_gun_walk_entry(i);
        }
        self.player_gun_stand_entry(i);
    }

    /// 0x19ACC standing fire (falls through).
    fn player_gun_stand_entry(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.b.set_state(0x0200_001C);
        p.weapon_atk = 1;
        p.b.draw_flags = 0;
        self.player_swing_pose(i);
        self.players[i].action = 0x10;
        self.player_gun_stand(i);
    }

    /// 0x19AEA state 0x1C. The shots are the item's business (it reads +$C4 and +$2C); here the
    /// animation runs, `+$81` counts the frames the fire button is still honoured, and a gun without
    /// ammo is let go (a club, kind 0x203, stays in the hand).
    pub(super) fn player_gun_stand(&mut self, i: usize) {
        if self.player_special_check(i) {
            return;
        }
        if self.players[i].pressed & JMP != 0 {
            return self.player_jump_entry(i);
        }
        if self.players[i].item_type == 0 || self.player_item_ammo(i) == 0 {
            return self.player_gun_let_go(i, false);
        }
        if !self.player_pstep(i) {
            return;
        }
        let p = &mut self.players[i];
        p.set_b81(p.b81().wrapping_sub(1));
        if p.held & 0xF != 0 {
            return self.player_gun_walk_entry(i);
        }
        if (p.b81() as i8) > 0 {
            return;
        }
        if p.held & ATK != 0 {
            p.set_b81(3);
            return;
        }
        self.player_idle_entry(i);
    }

    /// 0x19B48 / 0x19C06: no gun or no ammo: a club stays in the hand, everything else is let go
    /// (`+$B6` = 0, no drop request); then idle (the walking variant also clears the double-tap timer).
    fn player_gun_let_go(&mut self, i: usize, clear_dtap: bool) {
        let p = &mut self.players[i];
        if p.item_type != 0x203 {
            p.item_type = 0;
            p.drop = 0;
        }
        if clear_dtap {
            p.dtap = 0;
        }
        self.player_idle_entry(i);
    }

    /// 0x19B60 walking fire entry: the legs layer (set 0 animation 0x10, the frame of `+$35 & 0xF`),
    /// the gun walk pose for the upper body, speed class 0.
    fn player_gun_walk_entry(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.b.set_state(0x0200_001E);
        p.b.draw_flags = 0;
        let idx = p.b.w34 as u8 & 0xF;
        self.player_layer_start(i, 0x10, idx);
        self.players[i].weapon_atk = 1;
        self.player_gun_walk_pose(i); // 0x5F76
        self.player_set_speed_class(i, 0);
        self.player_gun_walk(i);
    }

    /// 0x19B90 state 0x1E. Like standing fire, with the legs layer stepping (0x1542) and the walk
    /// movement; without a direction it goes back to standing fire.
    pub(super) fn player_gun_walk(&mut self, i: usize) {
        if self.player_special_check(i) {
            return;
        }
        // 0x19B96: stage 1 area 1 (G.$4D9 == 1, G.$786 == 1) calls 0x60E2 (water ripples: effects port).
        if self.players[i].pressed & JMP != 0 {
            return self.player_jump_entry(i);
        }
        if self.players[i].item_type == 0 || self.player_item_ammo(i) == 0 {
            return self.player_gun_let_go(i, true);
        }
        if self.players[i].b.draw_flags != 0 {
            // 0x19BD8
            let p = &mut self.players[i];
            p.set_b81(p.b81().wrapping_sub(1));
            if p.held & 0xF == 0 {
                return self.player_gun_stand_entry(i);
            }
            if (p.b81() as i8) <= 0 {
                if p.held & ATK == 0 {
                    p.dtap = 0;
                    return self.player_idle_entry(i);
                }
                p.set_b81(3);
            }
        }
        // 0x19C20
        self.player_layer_tick(i);
        let nib = self.players[i].held & 0xF;
        if nib == 0 {
            return self.player_gun_stand_entry(i);
        }
        self.player_walk_move(i, nib);
    }

    // ---------------------------------------------------------------------------------------
    // Heavy weapon (0x203): charge state 0x20 (0x19C36); bazooka (0x209): recoil state 0x22 (0x19CE0)
    // ---------------------------------------------------------------------------------------

    /// 0x19C36 charge entry (falls through).
    fn player_charge_entry(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.b.set_state(0x0200_0020);
        p.b.draw_flags = 0;
        self.player_swing_pose(i);
        let p = &mut self.players[i];
        p.weapon_atk = 1;
        p.b.hitstop = 0;
        p.action = 0x10;
        p.w80 = 0;
        self.player_charge(i);
    }

    /// 0x19C62 state 0x20: while the attack button is held `+$80` counts up to 60 and the swing
    /// waits; on release (+$81 = 1) a swing held for 8 frames or more runs faster (the frame timer
    /// +$2F is set to 1), 45 frames or more skips a frame; the item swings on the frame bytes.
    pub(super) fn player_charge(&mut self, i: usize) {
        if self.player_special_check(i) {
            return;
        }
        if self.players[i].b81() == 0 {
            self.player_face_input(i);
            let p = &mut self.players[i];
            if p.held & ATK != 0 {
                let n = p.b80().wrapping_add(1);
                p.set_b80(if n >= 0x3C { 0x3C } else { n });
                return;
            }
            p.set_b81(1);
            if p.b80() >= 8 {
                p.b.timer = 1;
                if p.b80() >= 0x2D {
                    self.player_pstep(i);
                    let p = &mut self.players[i];
                    p.b.timer = p.b.timer.wrapping_sub(1);
                }
            }
        }
        if self.player_hitstop_wait(i) {
            return;
        }
        if self.player_pstep(i) {
            self.player_idle_entry(i);
        }
    }

    /// 0x19CE0 recoil entry (falls through): the bazooka throws the player back. (The ROM
    /// tamper test at 0x19CE0..0x19CFA passes on the real ROM and does nothing.)
    fn player_recoil_entry(&mut self, i: usize) {
        self.players[i].b.set_state(0x0200_0022);
        self.players[i].b.draw_flags = 0;
        self.player_swing_pose(i);
        self.player_set_item_a8(i, 0);
        let recoil = self.player_pd().recoil[self.players[i].b.id as usize & 3];
        let p = &mut self.players[i];
        p.weapon_atk = 1;
        p.b.hitstop = 0;
        p.action = 0x10;
        (p.b.vz, p.b.az, p.b.vy, p.b.ay) = (0, 0, 0, 0);
        let (mut vx, mut ax) = (recoil, -0x100i16);
        if p.b.facing != 0 {
            vx = vx.wrapping_neg();
            ax = ax.wrapping_neg();
        }
        p.b.vx = vx;
        p.b.ax = ax;
        self.player_recoil(i);
    }

    /// 0x19D5E state 0x22: pushed back against the facing (`ax` pulls the other way); in the air it
    /// falls; at the end the bazooka is let go when its ammo is gone.
    pub(super) fn player_recoil(&mut self, i: usize) {
        if self.players[i].b.airborne != 0 {
            let p = &mut self.players[i];
            (p.b.vx, p.b.ax, p.b.vz, p.b.az, p.b.vy, p.b.ay) = (0, 0, 0, 0, 0, 0);
            return self.player_fall_entry(i);
        }
        if self.player_hitstop_wait(i) {
            return;
        }
        if self.player_pstep(i) {
            if (self.player_item_ammo(i) as i16) <= 0 {
                self.players[i].item_type = 0;
            }
            return self.player_idle_entry(i);
        }
        if (self.players[i].b.vx.wrapping_add(0x20) as u16) < 0x40 {
            return; // too slow: the player stands still
        }
        // 0x19DC0: every other frame (bit 0 of G.$1) the dust 0x92E5E (d0 = 2): TODO(effects port).
        self.player_pmove(i);
    }

    // ---------------------------------------------------------------------------------------
    // Poses: the animation of the player for the held item
    // ---------------------------------------------------------------------------------------

    /// The test every pose routine starts with (0x5BDC, 0x5CA6, 0x5E5C, 0x5F76): a kind, an item
    /// that is alive, and in its held state (sub 8). `cmp.w $a0(a0),d0` compares the item's
    /// *address* with its kind word and never matches. Returns the item's slot.
    fn player_held_item(&self, i: usize) -> Option<usize> {
        let p = &self.players[i];
        if p.item_type == 0 || p.item == 0 {
            return None;
        }
        let Some(Slot::Item(j)) = slot_of(p.item) else { return None };
        let it = &self.items.slots[j];
        (it.b.live != 0 && it.m.w(io::KIND) != p.item && it.b.sub == 8).then_some(j)
    }

    /// Word `index` of the per-character table that `ptr_table[+$20]` points to (the pose tables
    /// are 4 pointers, one per character, followed by the tables). Negative = no pose for this kind.
    fn player_pose_word(&self, i: usize, ptr_table: u32, index: u32) -> i16 {
        let t = &self.assets.player.t;
        let tbl = t.u32(ptr_table, self.players[i].b.k20 as u32);
        t.u16(tbl, index * 2) as i16
    }

    /// The held item is gone: the player lets go of it (`+$B4 = +$B6 = 0`).
    fn player_let_go(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.item = 0;
        p.item_type = 0;
    }

    /// 0x5BDC idle pose: the set 5 animation of the held kind (table 0x5C46..), else animation 0 of
    /// set 0 (and a held item that is not in the hand any more is let go).
    pub(super) fn player_idle_anim(&mut self, i: usize) {
        if self.player_held_item(i).is_none() {
            self.player_let_go(i);
            return self.player_panim(i, 0, 0);
        }
        let id = self.player_pose_word(i, IDLE_POSE, (self.players[i].item_type & 0xFF) as u32);
        if id < 0 {
            return self.player_panim(i, 0, 0); // 0x5C10: the pointers stay
        }
        self.player_panim(i, 5, id as u16);
    }

    /// 0x5CA6 walk pose: the upper body is the set 5 animation of the held kind (0x5D3C..), the legs
    /// are the layer (set 0 animation 0x10 at the frame `+$35`), with the offset table 0x5D9C...
    pub(super) fn player_walk_anim(&mut self, i: usize) {
        if self.player_held_item(i).is_none() {
            self.player_let_go(i);
            return self.player_panim(i, 0, 1);
        }
        let k20 = self.players[i].b.k20 as u32;
        self.players[i].b.offs_table = self.assets.player.t.u32(WALK_OFFSETS, k20);
        let id = self.player_pose_word(i, WALK_POSE, (self.players[i].item_type & 0xFF) as u32);
        if id < 0 {
            return self.player_panim(i, 0, 1); // 0x5CDA
        }
        self.player_panim(i, 5, id as u16);
        self.players[i].b.draw_flags = 1;
        let idx = self.players[i].b.w34 as u8;
        self.player_layer_start(i, 0x10, idx);
    }

    /// 0x5E5C swing pose: the set 5 animation of the held kind (table 0x5EEE..); a heavy object
    /// (0x20A) uses the words behind the 12 kinds, by its variant (+$26). (An id of 0x8000 there
    /// reads animation 0: `add.w d0,d0` drops the top bit.)
    pub(super) fn player_swing_pose(&mut self, i: usize) {
        let Some(j) = self.player_held_item(i) else {
            self.player_let_go(i);
            return self.player_panim(i, 0, 0);
        };
        let id = if self.players[i].item_type == 0x20A {
            let var = self.items.slots[j].b.var26 as u32;
            let t = &self.assets.player.t;
            let tbl = t.u32(SWING_POSE, self.players[i].b.k20 as u32);
            t.u16(tbl, var.wrapping_add(0x18)) & 0x7FFF
        } else {
            let id = self.player_pose_word(i, SWING_POSE, (self.players[i].item_type & 0xFF) as u32);
            if id < 0 {
                return self.player_panim(i, 0, 0); // 0x5E90: the pointers stay
            }
            id as u16
        };
        self.player_panim(i, 5, id);
    }

    /// 0x5F76 gun walk pose (kinds 0x206 / 0x207): the upper body is the set 5 animation of the
    /// table 0x6002.., the legs come from the layer started by the caller.
    pub(super) fn player_gun_walk_pose(&mut self, i: usize) {
        if self.player_held_item(i).is_none() {
            self.player_let_go(i);
            self.players[i].b.draw_flags = 0;
            return self.player_panim(i, 0, 1);
        }
        let n = (self.players[i].item_type & 0xFF) as i16 - 6;
        let id = if (0..2).contains(&n) { self.player_pose_word(i, GUN_WALK_POSE, n as u32) } else { -1 };
        if id < 0 {
            self.players[i].b.draw_flags = 0; // 0x5FAA
            return self.player_panim(i, 0, 1);
        }
        let k20 = self.players[i].b.k20 as u32;
        self.players[i].b.offs_table = self.assets.player.t.u32(WALK_OFFSETS, k20);
        self.players[i].b.draw_flags = 1;
        self.player_panim(i, 5, id as u16);
    }

    // ---------------------------------------------------------------------------------------
    // The split-body layer (the legs): 0x1506 start, 0x1564 load, 0x1542 step
    // ---------------------------------------------------------------------------------------

    /// The frame `idx` (a signed byte, 12 bytes per frame) after the first frame of animation
    /// `anim` of set 0 of the player (0x1506 / 0x13EA). `None` when no frame is extracted there.
    fn player_set0_frame(&self, i: usize, anim: u16, idx: u8) -> Option<FrameId> {
        let s = &self.assets.sprites;
        let table = s.player_sets[0][self.players[i].b.id as usize & 3];
        let first = s.table(table).anims.get(anim as usize).copied().flatten()?;
        let addr = s.frame(first).rom_addr.wrapping_add((idx as i8 as i32).wrapping_mul(12) as u32);
        s.frame_by_addr.get(&addr).copied()
    }

    /// 0x1506: start the layer at the frame `idx` of animation `anim` of set 0.
    pub(super) fn player_layer_start(&mut self, i: usize, anim: u16, idx: u8) {
        match self.player_set0_frame(i, anim, idx) {
            Some(f) => self.player_layer_load(i, f),
            // A frame record the extractor did not decode (the 68000 would read whatever is there).
            None => eprintln!("player {i}: layer frame {idx} of animation {anim:#x} of set 0 is not extracted"),
        }
    }

    /// 0x1564 / 0x15A0: load the frame `f` into the layer: its extra bytes (read at the first word
    /// of the record), the frame bytes +4 / +5 in `+$34`, +6 / +7 (without the link bit) in `+$36`
    /// (the timer is +$37), and the offset of the upper body from the table `+$38` at the index
    /// `+$35` (a signed byte, 4 bytes per entry).
    fn player_layer_load(&mut self, i: usize, f: FrameId) {
        let s = &self.assets.sprites;
        let fr = s.frame(f);
        let extra = self.assets.player.comp_extra.get((fr.w0 / 4) as usize).copied().unwrap_or_default();
        let (w34, w36) = ((fr.param_a as u16) << 8 | fr.param_b as u16, (fr.kind as u16) << 8 | fr.duration as u16);
        let b = &mut self.players[i].b;
        b.frame2 = f;
        b.extra = extra;
        b.w34 = w34;
        b.w36 = w36;
        let at = b.offs_table.wrapping_add(((w34 as u8 as i8 as i32) * 4) as u32);
        let t = &self.assets.player.t;
        // A stale table pointer (+$38 = 0 before the first weapon walk) reads the ROM start.
        b.dx3c = table_word(t, at).unwrap_or(0) as i16;
        b.dy3e = table_word(t, at.wrapping_add(2)).unwrap_or(0) as i16;
    }

    /// 0x1542: step the layer (only while `+$25` bit 0 is set). The carry (true) means the layer
    /// followed its link, that is the layer animation finished.
    pub(super) fn player_layer_tick(&mut self, i: usize) -> bool {
        let b = &mut self.players[i].b;
        if b.draw_flags & 1 == 0 {
            return false;
        }
        let t = (b.w36 as u8).wrapping_sub(1);
        b.w36 = (b.w36 & 0xFF00) | t as u16;
        if t != 0 {
            return false;
        }
        let f = self.assets.sprites.frame(b.frame2);
        let (next, finished) = (f.next, f.last);
        self.player_layer_load(i, next);
        finished
    }

    // ---------------------------------------------------------------------------------------
    // Running with a weapon: the pose 0x1A0F0, the run swing state 0x202 (0x1A18C, 0x1A2F2)
    // ---------------------------------------------------------------------------------------

    /// 0x1A0F0 run pose: the legs layer (set 0 animation 0xB, or 0x18 in run state 4, at the frame
    /// of the upper body's `+$2D`, with its timer) and the upper body of the run pose table 0x1A15C.
    /// Only for kinds 0x205 and up (Jack: 0x201 and up are shown by the plain run) and for the
    /// heavy weapon (0x203, the other characters).
    pub(super) fn player_run_overlay(&mut self, i: usize) {
        let kind = self.players[i].item_type;
        if kind == 0 {
            return;
        }
        let mut class = 1u32;
        if kind < 0x205 {
            if self.players[i].b.id == 0 {
                return;
            }
            class = 0;
            if kind != 0x203 {
                return;
            }
        }
        let k20 = self.players[i].b.k20 as u32;
        let offs = self.assets.player.t.u32(RUN_OFFSETS, k20);
        let b = &mut self.players[i].b;
        b.draw_flags = 1;
        b.offs_table = offs;
        let (d1, d3, step) = (b.param_b, b.timer, b.step);
        self.player_layer_start(i, if step == 4 { 0x18 } else { 0xB }, d1);
        let b = &mut self.players[i].b;
        b.w36 = (b.w36 & 0xFF00) | d3 as u16;
        let id = self.assets.player.t.u8(RUN_POSE, self.players[i].b.id as u32 * 2 + class);
        self.player_panim(i, 5, id as u16);
    }

    /// 0x1A174: the attack button while running. With a club or a throwable (0x201 / 0x202) the
    /// player swings while running (state 0x202); anything else is the dash attack.
    pub(super) fn player_run_attack_entry(&mut self, i: usize) {
        let kind = self.players[i].item_type;
        if kind != 0x201 && kind != 0x202 {
            return self.player_dash_attack_entry(i);
        }
        // 0x1A18C
        let k20 = self.players[i].b.k20 as u32;
        let offs = self.assets.player.t.u32(RUN_OFFSETS, k20);
        let b = &mut self.players[i].b;
        b.set_state(0x0200_0202);
        b.offs_table = offs;
        b.draw_flags = 1;
        let (d1, d3) = (b.param_b, b.timer);
        self.player_layer_start(i, 0xB, d1);
        let b = &mut self.players[i].b;
        b.w36 = (b.w36 & 0xFF00) | d3 as u16;
        // 0x1A1EA: the swing pose by character (word table 0x1A228), the item swings (+$A8 = 1).
        let id = self.assets.player.t.u16(RUN_SWING_POSE, self.players[i].b.id as u32 * 2);
        self.player_panim(i, 5, id);
        self.players[i].weapon_atk = 1;
        self.player_set_item_a8(i, 1);
        self.player_set_speed_class(i, 4);
        let nib = self.players[i].held & 0xF;
        // `move.b d0,$b2(a6)`: the d0 the walk move leaves behind (see `player_pstep_d0`).
        self.players[i].combo = self.player_walk_move(i, nib);
        self.player_run_swing(i);
    }

    /// 0x1A2F2 state 0x202 (run group step 2): the swing while running. Without a direction the
    /// player stops; when the swing animation ends the run starts again (0x19FDE) with the legs
    /// layer's frame as the run frame.
    pub(super) fn player_run_swing(&mut self, i: usize) {
        if self.players[i].held & 0xF == 0 {
            return self.player_idle_entry(i);
        }
        self.player_pmove(i);
        if !self.player_pstep(i) {
            self.player_layer_tick(i);
            return;
        }
        let b = &mut self.players[i].b;
        b.draw_flags = 0;
        let (d3, d1) = (b.w36 as u8, b.w34 as u8);
        if let Some(f) = self.player_set0_frame(i, 9, d1) {
            let s = &self.assets.sprites;
            self.players[i].b.load_frame(s, f); // 0x13EA
        }
        self.players[i].b.timer = d3;
        self.player_run_entry_tail(i);
    }

    // ---------------------------------------------------------------------------------------
    // Heavy object (0x20A) in the air: jump 0x1B038 / 0x1B08C (state 0x1E), landing 0x1B0CA / 0x1B0E6 (state 0x20)
    // ---------------------------------------------------------------------------------------

    /// 0x1B038 jump with a heavy object in the hands (falls through): the heavy jump vector
    /// (class 0xC), its own animation, no jump attacks.
    pub(super) fn player_heavy_jump_entry(&mut self, i: usize) {
        let p = &mut self.players[i];
        p.b.set_state(0x0200_041E);
        p.b.airborne = 1;
        self.player_face_input(i);
        let nib = self.players[i].held & 0xF;
        self.players[i].b.dir = self.player_pd().jump_dir[nib as usize];
        self.player_set_jump_velocity(i, 0xC);
        self.players[i].b.draw_flags = 0;
        let anim = self.player_pd().heavy_jump_anim[self.players[i].b.id as usize & 3];
        self.player_panim(i, 5, anim as u16);
        let p = &mut self.players[i];
        p.wall_latch = 0;
        p.b83 = 0;
        p.action = 1;
        self.player_heavy_jump(i);
    }

    /// 0x1B08C state 0x1E.
    pub(super) fn player_heavy_jump(&mut self, i: usize) {
        if self.players[i].b.vy < 0 {
            self.player_splash_check(i);
            if self.players[i].b.airborne == 0 {
                let p = &mut self.players[i];
                p.b.vy = 0;
                p.b.ay = 0;
                return self.player_heavy_land_entry(i);
            }
        }
        self.player_pstep(i);
        if self.players[i].b.param_a == 0 {
            self.player_pmove(i);
        }
        self.player_air_control(i);
    }

    /// 0x1B0CA landing with a heavy object (falls through).
    fn player_heavy_land_entry(&mut self, i: usize) {
        self.players[i].b.set_state(0x0200_0420);
        let anim = self.player_pd().heavy_land_anim[self.players[i].b.id as usize & 3];
        self.player_panim(i, 5, anim as u16);
        self.player_heavy_land(i);
    }

    /// 0x1B0E6 state 0x20.
    pub(super) fn player_heavy_land(&mut self, i: usize) {
        if self.player_pstep(i) {
            self.player_idle_entry(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Handle;
    use crate::game::item::Item;
    use dino_assets::{AssetPack, Frame, FrameDraw, HitBox, HitBoxBank};

    /// A game whose hit box bank holds one box at index 0 of the table 0x100000:
    /// Z 0x10 wide from -8, X 0x20 wide from -0x10.
    fn game() -> Game {
        let mut a = AssetPack::default();
        let b = HitBox { zoff: -8, zsize: 0x10, xoff: -0x10, xsize: 0x20, yoff: 0, ysize: 8 };
        a.sprites.hitboxes = HitBoxBank { base: 0x100000, words: vec![b.zoff, b.zsize, b.xoff, b.xsize, b.yoff, b.ysize] };
        Game::new(Box::leak(Box::new(a)))
    }

    /// Item slot `slot` lying on the floor at (x, y, z) with kind `kind`; registers it in the pick-up list.
    fn floor_item(g: &mut Game, slot: usize, kind: u16, x: i16, y: i16, z: i16) -> Handle {
        let h = g.items.handle(slot);
        let it: &mut Item = &mut g.items.slots[slot];
        it.b.live = 1;
        it.b.mode = 2;
        it.b.sub = 0;
        it.b.box_table = 0xC010_0000; // flag bits in the top byte are ignored (24-bit address bus)
        it.b.attack_box = 0;
        it.b.set_xi(x);
        it.b.set_yi(y);
        it.b.set_zi(z);
        it.m.sw(io::KIND, kind);
        g.combat.push(0x69EE, h);
        h
    }

    fn standing_player(g: &mut Game) {
        let p = &mut g.players[0];
        p.b.live = 1;
        p.b.set_xi(100);
        p.b.set_yi(0);
        p.b.set_zi(50);
    }

    #[test]
    fn find_pickup_takes_the_lowest_kind_in_reach() {
        let mut g = game();
        standing_player(&mut g);
        // Item height 8: Y - 8 - player Y = 0, inside the window 0..16 below.
        let rifle = floor_item(&mut g, 3, 0x205, 100, 8, 50);
        let pistol = floor_item(&mut g, 4, 0x204, 105, 8, 52);
        floor_item(&mut g, 5, 0x207, 100, 8, 50);
        g.player_find_pickup(0);
        assert_eq!((g.players[0].cand, g.players[0].cand_type), (pistol, 0x204));
        // Out of reach in X (the box is 0x20 wide from X - 0x10): the rifle only.
        g.items.slots[4].b.set_xi(150);
        g.player_find_pickup(0);
        assert_eq!((g.players[0].cand, g.players[0].cand_type), (rifle, 0x205));
    }

    #[test]
    fn find_pickup_needs_the_item_on_the_floor_and_the_player_on_the_ground() {
        let mut g = game();
        standing_player(&mut g);
        let h = floor_item(&mut g, 3, 0x205, 100, 8, 50);
        g.items.slots[3].b.sub = 8; // held by somebody
        g.player_find_pickup(0);
        assert_eq!(g.players[0].cand_type, 0x8000, "no candidate: the sentinel stays");
        g.items.slots[3].b.sub = 0;
        g.items.slots[3].b.set_yi(40); // too high above the player
        g.player_find_pickup(0);
        assert_eq!(g.players[0].cand_type, 0x8000);
        g.items.slots[3].b.set_yi(8);
        g.players[0].b.airborne = 1;
        g.player_find_pickup(0);
        assert_eq!((g.players[0].cand, g.players[0].cand_type), (0, 0x8000), "airborne: nothing, and the pointer is cleared");
        g.players[0].b.airborne = 0;
        g.w.no_clamp = 1; // G.$506
        g.player_find_pickup(0);
        assert_eq!(g.players[0].cand_type, 0x8000);
        g.w.no_clamp = 0;
        g.player_find_pickup(0);
        assert_eq!(g.players[0].cand, h);
    }

    #[test]
    fn weapons_wait_40_frames_after_a_pickup_but_food_does_not() {
        let mut g = game();
        standing_player(&mut g);
        floor_item(&mut g, 3, 0x205, 100, 8, 50);
        let food = floor_item(&mut g, 4, 0x0005, 100, 8, 50);
        g.players[0].pickup_cd = 0x28;
        g.player_find_pickup(0);
        assert_eq!((g.players[0].cand, g.players[0].cand_type), (food, 5));
        g.items.slots[4].b.live = 0;
        g.player_find_pickup(0);
        assert_eq!(g.players[0].cand_type, 0x8000, "the weapon (kind high byte 2) is skipped");
        g.players[0].pickup_cd = 0;
        g.player_find_pickup(0);
        assert_eq!(g.players[0].cand_type, 0x205);
    }

    #[test]
    fn a_heavy_object_needs_the_stick_towards_it_except_variants_6_and_8() {
        let mut g = game();
        standing_player(&mut g);
        // Player left of the item (100 < 104): RIGHT (1) points at it; Z equal (50 >= 50): DOWN (4).
        floor_item(&mut g, 3, 0x20A, 104, 8, 50);
        g.items.slots[3].b.var26 = 2;
        g.players[0].held = 0;
        g.player_find_pickup(0);
        assert_eq!(g.players[0].cand_type, 0x8000, "no stick, no pick-up");
        g.players[0].held = 2; // LEFT: away from it
        g.player_find_pickup(0);
        assert_eq!(g.players[0].cand_type, 0x8000);
        g.players[0].held = 1; // RIGHT
        g.player_find_pickup(0);
        assert_eq!(g.players[0].cand_type, 0x20A);
        g.players[0].held = 4; // DOWN: the Z test (player Z is not below the item's)
        g.player_find_pickup(0);
        assert_eq!(g.players[0].cand_type, 0x20A);
        g.players[0].held = 8; // UP
        g.player_find_pickup(0);
        assert_eq!(g.players[0].cand_type, 0x8000);
        for var in [6, 8] {
            g.items.slots[3].b.var26 = var;
            g.players[0].held = 0;
            g.player_find_pickup(0);
            assert_eq!(g.players[0].cand_type, 0x20A, "variant {var} is picked up without the stick");
        }
    }

    #[test]
    fn the_held_item_test_matches_the_pose_routines() {
        let mut g = game();
        standing_player(&mut g);
        let h = g.items.handle(7);
        assert_eq!(g.player_held_item(0), None, "no kind");
        g.players[0].item_type = 0x203;
        assert_eq!(g.player_held_item(0), None, "no item");
        g.players[0].item = h;
        assert_eq!(g.player_held_item(0), None, "the item is not alive");
        g.items.slots[7].b.live = 1;
        g.items.slots[7].b.sub = 4;
        assert_eq!(g.player_held_item(0), None, "being lifted is not held");
        g.items.slots[7].b.sub = 8;
        assert_eq!(g.player_held_item(0), Some(7));
        g.players[0].item = 0x1234;
        assert_eq!(g.player_held_item(0), None, "not a pool E handle");
    }

    #[test]
    fn auto_aim_takes_the_nearest_enemy_in_front() {
        let mut g = game();
        standing_player(&mut g);
        g.players[0].b.facing = 1; // faces right
        let near = g.enemies.handle(0);
        let far = g.enemies.handle(1);
        let behind = g.enemies.handle(2);
        let hurt = g.enemies.handle(3);
        for (slot, x, z, sub) in [(0, 150, 60, 0), (1, 200, 120, 0), (2, 50, 50, 0), (3, 150, 50, 2)] {
            let b = &mut g.enemies.slots[slot].b;
            b.live = 1;
            b.sub = sub;
            b.set_xi(x);
            b.set_zi(z);
        }
        for h in [far, behind, near, hurt] {
            g.combat.push(0x69BA, h);
        }
        g.player_aim_nearest(0);
        assert_eq!(g.players[0].aim, near, "|dz| 10 beats 70; the one behind and the one in a reaction do not count");
        g.players[0].b.facing = 0; // faces left: only the enemy at X 50
        g.player_aim_nearest(0);
        assert_eq!(g.players[0].aim, behind);
        // Nobody within 0x100 in depth: the old target stays.
        g.enemies.slots[2].b.set_zi(50 + 0x101);
        g.players[0].aim = 0;
        g.player_aim_nearest(0);
        assert_eq!(g.players[0].aim, 0);
    }

    /// Two frames of 12 bytes: frame 0 lasts 2 ticks, frame 1 lasts 1 tick and ends the animation
    /// (its link leads back to frame 0). Offsets table at 0x1000: 4 bytes per index.
    fn layer_game() -> Game {
        let mut a = AssetPack::default();
        let frame = |w0: u16, pa: u8, pb: u8, kind: u8, dur: u8, last: bool, next: u32| Frame {
            rom_addr: 0,
            stride: 12,
            w0,
            w1: 0x8000,
            draw: FrameDraw::Comp { main: w0, overlay: None },
            param_a: pa,
            param_b: pb,
            kind,
            duration: dur,
            last,
            body_box: 0,
            attack_box: 0,
            attach: 0,
            extra: [0; 4],
            next,
        };
        a.sprites.frames = vec![frame(4, 0x11, 1, 0x22, 2, false, 1), frame(8, 0x33, 0xFF, 0x44, 1, true, 0)];
        a.player.comp_extra = vec![[0; 4], [1, 2, 3, 4], [5, 6, 7, 8]];
        // dx, dy for index 1 (bytes +4, +6) and for index -1 (bytes -4, -2).
        a.player.t.blobs = vec![(0x1000, vec![0, 0, 0, 0, 0xAA, 0xBB, 0xCC, 0xDD])];
        a.player.t.blobs.push((0xFFC, vec![0x11, 0x11, 0x22, 0x22]));
        Game::new(Box::leak(Box::new(a)))
    }

    #[test]
    fn the_layer_loads_like_0x1564_and_steps_like_0x1542() {
        let mut g = layer_game();
        g.players[0].b.offs_table = 0x1000;
        g.player_layer_load(0, 0);
        let b = &g.players[0].b;
        assert_eq!((b.frame2, b.w34, b.w36), (0, 0x1101, 0x2202));
        assert_eq!(b.extra, [1, 2, 3, 4], "the extra bytes of the composition at w0 = 4");
        assert_eq!((b.dx3c, b.dy3e), (0xAABBu16 as i16, 0xCCDDu16 as i16), "index +$35 = 1: 4 bytes into the offset table");
        // Not split: nothing happens.
        assert!(!g.player_layer_tick(0));
        assert_eq!(g.players[0].b.w36, 0x2202);
        g.players[0].b.draw_flags = 1;
        assert!(!g.player_layer_tick(0), "the timer +$37 counts 2 -> 1");
        assert_eq!((g.players[0].b.frame2, g.players[0].b.w36), (0, 0x2201));
        assert!(!g.player_layer_tick(0), "1 -> 0: the next frame, no carry (the frame was not the last)");
        let b = &g.players[0].b;
        assert_eq!((b.frame2, b.w34, b.w36, b.extra), (1, 0x33FF, 0x4401, [5, 6, 7, 8]));
        assert_eq!((b.dx3c, b.dy3e), (0x1111, 0x2222), "index +$35 = 0xFF is -1: the entry before the table");
        assert!(g.player_layer_tick(0), "the last frame follows its link: carry");
        assert_eq!(g.players[0].b.frame2, 0);
    }

    #[test]
    fn a_stale_offset_table_pointer_reads_zero_when_it_is_not_extracted() {
        let mut g = layer_game();
        g.players[0].b.offs_table = 0x5000;
        g.player_layer_load(0, 0);
        assert_eq!((g.players[0].b.dx3c, g.players[0].b.dy3e), (0, 0));
    }

    /// 0x1BD94 leaves the composition word of the new frame in d0 when the timer ran out, else the low
    /// byte of the Z motion (0): `move.b d0,$b2(a6)` after it stores that.
    #[test]
    fn the_walk_move_leaves_the_composition_byte_in_d0() {
        let mut g = layer_game();
        g.players[0].b.frame = 0;
        g.players[0].b.timer = 2;
        assert_eq!(g.player_pstep_d0(0), (false, 0), "timer 2 -> 1: d0 is the Z motion, low byte 0");
        assert_eq!(g.player_pstep_d0(0), (false, 8), "timer 1 -> 0: the frame 1 loads, d0 = its w0 (w1 is negative)");
    }

    /// 0x65A4: the new player gets a bazooka (type 6, 4 rockets) in the hand, held (sub 8), with a
    /// 600 frame life when it is dropped; the player's +$B4 / +$B6 point to it.
    #[test]
    fn a_joining_player_gets_a_held_bazooka() {
        let mut g = game();
        standing_player(&mut g);
        g.players[0].b.who = 0;
        g.players[0].b.id = 1;
        g.item_give_bazooka(0);
        let h = g.items.handle(23); // the first allocation takes the last slot
        assert_eq!(g.players[0].item, h);
        assert_eq!(g.players[0].item_type, 0x209);
        let it = &g.items.slots[23];
        assert_eq!((it.b.live, it.b.k20, it.b.var26, it.b.sub, it.b.hp), (1, 0, 0xC, 8, 4));
        assert_eq!((it.b.partner, it.b.id, it.m.w(io::LIFE)), (Game::player_handle(0), 1, 0x258));
        assert_eq!(g.player_held_item(0), Some(23), "the pose routines see it as held");
        g.players[0].item_type = 0; // a full pool gives nothing and keeps the player as it is
        for _ in 0..23 {
            g.items.alloc();
        }
        g.players[0].item = 0;
        g.item_give_bazooka(0);
        assert_eq!(g.players[0].item, 0);
    }
}
