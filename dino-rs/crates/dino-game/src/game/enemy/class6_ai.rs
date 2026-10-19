//! Pool A class 6: Bludge hop AI, states, targeting and slot movement (0x39686..0x3AFB0).
//!
//! Bludge circles the target using 12 approach slots, doing short hops, medium hops,
//! or large parabolic leaps, steering towards each slot point in turn.

use super::Enemy;
use crate::game::{Game, Handle, Slot, slot_of};

impl Game {
    /// 0x39686: Bludge alive frame in mode 2 sub 0.
    pub fn enemy_c6_alive(&mut self, e: &mut Enemy, h: Handle) {
        // 0x39686: poise timer
        let poise = e.m.b(0x79);
        if poise != 0 {
            let p = poise - 1;
            e.m.sb(0x79, p);
            if p == 0 {
                e.b.stun = 10; // move.b #$a, $78(a6)
            }
        }

        // 0x3969A: dispatch on $6(a6) (act)
        match e.b.act {
            0 => self.enemy_c6_state_wait(e, h),
            2 => self.enemy_c6_state_hop(e, h),
            4 => self.enemy_c6_state_taunt(e),
            6 => self.enemy_c6_state_attack(e),
            8 => self.enemy_c6_state_eat(e),
            _ => self.enemy_c6_reset_ai(e),
        }

        // 0x396A6: jsr $12cb4.l (terrain collide, store obstacle in +$94)
        if let Some(o) = self.terrain_collide(&mut e.b, h, false) {
            e.m.sw(0x94, o);
        }

        // 0x396AC: jsr $49ca.l
        self.enemy_draw_check(&mut e.b, h);
        // 0x396B2: jmp $4bea.l
        self.enemy_push_hurtable(h);
    }

    /// 0x396DC: State 0 wait timer, check eat, fall into State 2.
    fn enemy_c6_state_wait(&mut self, e: &mut Enemy, h: Handle) {
        let wait = e.m.b(0x7A);
        if wait != 0 {
            let next_w = wait.wrapping_sub(1);
            e.m.sb(0x7A, next_w);
            if next_w != 0 {
                return;
            }
        }

        // 0x396EA: check eat (0x3AFFA, 0x3AFA4)
        self.enemy_c6_claim_item(e);
        if self.enemy_c6_check_eat(e) {
            // 0x396F2: bcs 0x39ac8 -> state 8 (eating)
            e.b.set_state(0x0200_0800);
            return;
        }

        // 0x396F6: fall through into State 2
        self.enemy_c6_state_hop(e, h);
    }

    /// 0x396F6: State 2 decision cycle & hop movement.
    fn enemy_c6_state_hop(&mut self, e: &mut Enemy, h: Handle) {
        // 0x396F6: target check
        let target = e.m.w(0x76);
        let invalid = if let Some(Slot::Player(pi)) = slot_of(target) {
            let p = &self.players[pi];
            p.b.live == 0 || p.action == 0x23
        } else {
            true
        };

        let b0 = e.m.w(0xB0).wrapping_sub(1);
        e.m.sw(0xB0, b0);
        if invalid || (b0 as i16) < 0 {
            e.m.sw(0xB0, 0x258); // 600 frames
            self.enemy_c6_retarget(e); // 0x3A5D2
        }

        // 0x39718: decision timer $AE
        let ae = e.m.b(0xAE).wrapping_sub(1);
        e.m.sb(0xAE, ae);
        if (ae as i8) < 0 {
            if e.b.airborne != 0 {
                e.m.sb(0xAE, 0);
            } else {
                e.m.sb(0xAE, 0x5A); // reload 90 frames
                if self.enemy_c6_check_attack(e) {
                    // 0x39734: bcs 0x39c1a (start attack)
                    self.enemy_c6_start_attack(e);
                    return;
                } else if self.enemy_c6_check_taunt(e) {
                    // 0x3973C: bcs 0x39aa8 (taunt)
                    e.b.set_state(0x0200_0400);
                    self.enemy_anim(&mut e.b, 3);
                    return;
                } else {
                    // 0x39740: level threshold check
                    self.enemy_c6_check_level_threshold(e);
                }
            }
        }

        // 0x39744: step check
        if e.b.step == 0 {
            // 0x3974C: init step 2
            e.b.set_state(0x0200_0202);
            e.m.sb(0x7D, 0);
            e.m.sb(0xAA, 0xF0);
            e.m.sb(0xAC, 0xFF);
            e.m.sb(0xAD, 0);
            e.m.sb(0xA9, 0);
            self.enemy_c6_pick_hop_type(e);
        }

        // 0x39780: step 2 update
        self.enemy_c6_step2_update(e, h);
    }

    /// 0x39780: update movement step 2.
    fn enemy_c6_step2_update(&mut self, e: &mut Enemy, h: Handle) {
        match e.m.b(0xAB) {
            0 => self.enemy_c6_hop_type0(e),
            2 => self.enemy_c6_hop_type2(e, h),
            4 => self.enemy_c6_hop_type4(e, h),
            6 => self.enemy_c6_hop_type6(e, h),
            _ => self.enemy_c6_hop_type0(e),
        }
    }

    /// 0x397F8: Hop Type 0 creep / walk on ground.
    fn enemy_c6_hop_type0(&mut self, e: &mut Enemy) {
        if e.m.b(0xAC) != e.m.b(0xAB) {
            e.m.sb(0xAC, e.m.b(0xAB));
            self.enemy_anim(&mut e.b, 0);
            self.enemy_c6_set_speed_by_level(e);
            e.m.sb(0x7D, 0);
            e.m.sw(0xA0, 0);
        }

        // 0x39818: update
        let t = e.m.w(0xA0).wrapping_sub(1);
        e.m.sw(0xA0, t);
        if (t as i16) < 0 {
            e.m.sw(0xA0, 4);
            self.enemy_c6_steer_to_dest(e);
        }

        let aa = e.m.b(0xAA).wrapping_sub(1);
        e.m.sb(0xAA, aa);
        if (aa as i8) < 0 || self.enemy_c6_arrived_at_slot(e) {
            self.enemy_c6_advance_slot(e); // 0x3A7AA
            e.b.act = 2;
            e.b.step = 0; // loop back to 0x0200
            return;
        }

        self.enemy_move_dir(&mut e.b); // 0x1862
        self.enemy_walk_anim(&mut e.b); // 0x324D4
    }

    /// 0x3984C: Hop Type 2 small hop.
    fn enemy_c6_hop_type2(&mut self, e: &mut Enemy, _h: Handle) {
        match e.m.b(0xAD) {
            0 => {
                // 0x39860: start hop
                e.m.sb(0xAD, 2);
                self.enemy_anim(&mut e.b, 1);
                self.enemy_set_speed(&mut e.b, 7); // speed id 7
                self.enemy_c6_face_target(e);
                e.b.airborne = 1;
                e.b.vy = 0x700;
                e.b.ay = -0x50;
                self.enemy_c6_compute_dir_to_slot(e);
                e.b.dir &= 0xFE;
                let aa = e.m.b(0xAA).wrapping_sub(1);
                e.m.sb(0xAA, if (aa as i8) < 0 { 0 } else { aa });
                self.enemy_c6_hop_flight(e, 2);
            }
            2 => self.enemy_c6_hop_flight(e, 2),
            4 => self.enemy_c6_hop_landing(e),
            _ => {
                e.b.act = 2;
                e.b.step = 0;
            }
        }
    }

    /// 0x398F4: Hop Type 4 medium hop.
    fn enemy_c6_hop_type4(&mut self, e: &mut Enemy, _h: Handle) {
        match e.m.b(0xAD) {
            0 => {
                // 0x39908: start hop
                e.m.sb(0xAD, 2);
                self.enemy_anim(&mut e.b, 1);
                self.enemy_set_speed(&mut e.b, 13); // speed id 13
                self.enemy_c6_face_target(e);
                e.b.airborne = 1;
                e.b.vy = 0x800;
                e.b.ay = -0x60;
                self.enemy_c6_compute_dir_to_slot(e);
                e.b.dir &= 0xFE;
                let aa = e.m.b(0xAA).wrapping_sub(1);
                e.m.sb(0xAA, if (aa as i8) < 0 { 0 } else { aa });
                self.enemy_c6_hop_flight(e, 4);
            }
            2 => self.enemy_c6_hop_flight(e, 4),
            4 => self.enemy_c6_hop_landing(e),
            _ => {
                e.b.act = 2;
                e.b.step = 0;
            }
        }
    }

    /// 0x39896 / 0x3993E: in flight for hop types 2 and 4.
    fn enemy_c6_hop_flight(&mut self, e: &mut Enemy, _hop_type: u8) {
        let aa = e.m.b(0xAA).wrapping_sub(1);
        e.m.sb(0xAA, if (aa as i8) < 0 { 0 } else { aa });

        if e.b.airborne != 0 {
            if e.b.param_b == 0 {
                self.enemy_c6_hop_integrate(&mut e.b); // 0x3ABEC
            }
            self.enemy_step(&mut e.b); // 0x121E
        } else {
            // Landed
            e.m.sb(0xAD, 4);
            self.enemy_anim(&mut e.b, 2); // landing anim
        }
    }

    /// 0x398C2 / 0x3996A: landing for hop types 2 and 4.
    fn enemy_c6_hop_landing(&mut self, e: &mut Enemy) {
        let aa = e.m.b(0xAA).wrapping_sub(1);
        e.m.sb(0xAA, if (aa as i8) < 0 { 0 } else { aa });

        if self.enemy_step(&mut e.b) {
            let aa_cnt = e.m.b(0xAA).wrapping_sub(1);
            e.m.sb(0xAA, aa_cnt);
            if (aa_cnt as i8) < 0 || self.enemy_c6_arrived_at_slot(e) {
                self.enemy_c6_advance_slot(e);
                e.b.act = 2;
                e.b.step = 0;
            } else {
                e.m.sb(0xAD, 0); // repeat hop
            }
        }
    }

    /// 0x3999C: Hop Type 6 large parabolic leap.
    fn enemy_c6_hop_type6(&mut self, e: &mut Enemy, _h: Handle) {
        match e.m.b(0xAD) {
            0 => {
                // 0x399B6: start leap
                e.m.sb(0xAD, 2);
                self.enemy_anim(&mut e.b, 1);
                self.enemy_c6_steer_to_dest(e);
                e.b.airborne = 1;
                e.b.vy = 0x900;
                e.b.ay = -0x60;

                // 44 frames flight time
                let dest_x = e.m.w(0x88) as i16;
                let dest_z = e.m.w(0x8A) as i16;
                e.b.vx = self.enemy_c6_leap_speed(e.b.xi(), dest_x);
                e.b.ax = 0;
                e.b.vz = self.enemy_c6_leap_speed(e.b.zi(), dest_z);
                e.b.az = 0;

                self.enemy_c6_leap_flight(e);
            }
            2 => self.enemy_c6_leap_flight(e),
            4 => {
                // 0x39A5C: landing tick
                if self.enemy_step(&mut e.b) {
                    e.m.sb(0xAA, 0xF0);
                    self.enemy_c6_advance_slot(e);
                    e.b.act = 2;
                    e.b.step = 0;
                }
            }
            _ => {
                e.b.act = 2;
                e.b.step = 0;
            }
        }
    }

    /// 0x39A38: in flight for hop type 6.
    fn enemy_c6_leap_flight(&mut self, e: &mut Enemy) {
        if e.b.airborne != 0 {
            if e.b.param_b == 0 {
                e.b.motion(); // 0x189C
            }
            self.enemy_step(&mut e.b); // 0x121E
        } else {
            // Landed
            e.m.sb(0xAD, 4);
            self.enemy_anim(&mut e.b, 2); // landing anim
        }
    }

    /// 0x39A7A: compute leap velocity for 44 frames flight time.
    fn enemy_c6_leap_speed(&self, current: i16, dest: i16) -> i16 {
        let diff = (current as u16).wrapping_sub(dest as u16);
        let abs_d = if (diff as i16) < 0 {
            (dest as u16).wrapping_sub(current as u16)
        } else {
            diff
        };
        let mut d0 = abs_d / 44;
        let mut d1 = abs_d % 44;

        if (diff as i16) >= 0 {
            d0 = (-(d0 as i16)) as u16;
            d1 = (-(d1 as i16)) as u16;
        }

        let d1_sh = d1.wrapping_shl(2);
        let high = ((d1_sh >> 8) as u8).wrapping_add(d0 as u8);
        let low = d1_sh as u8;
        i16::from_be_bytes([high, low])
    }

    /// 0x39AB8: State 4 taunt (anim 3).
    fn enemy_c6_state_taunt(&mut self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            // 0x39AC0: reset back to State 2 step 0
            e.b.act = 2;
            e.b.step = 0;
        }
    }

    /// State 6 attack handler (M7-08).
    fn enemy_c6_state_attack(&mut self, e: &mut Enemy) {
        // Fallback for M7-07: loop back to State 2
        e.b.act = 2;
        e.b.step = 0;
        e.m.sb(0x7D, 0);
    }

    /// State 8 eating handler (M7-09).
    fn enemy_c6_state_eat(&mut self, e: &mut Enemy) {
        // Fallback for M7-07: loop back to State 2
        e.b.act = 2;
        e.b.step = 0;
        e.m.sb(0x7D, 0);
    }

    /// 0x39C1A: enter attack state.
    fn enemy_c6_start_attack(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0600);
        // 0x3ADB4: pick attack script
        let census = (self.spawn.g.alive as u16).min(6);
        let r = (self.rand() & 0x1F) as u32;
        let b4 = self.assets.enemy.t.u8(0x3ADE0, (census as u32 * 32) | r);
        e.m.sb(0xB4, b4);
        e.m.sw(0xB2, (b4 as u16) << 2);
        e.m.sb(0x7D, 1);
    }

    // =========================================================================
    // Targeting and Slot Helpers
    // =========================================================================

    /// 0x3A54C: candidate target initialization on entrance end.
    pub fn enemy_c6_target_init(&mut self, e: &mut Enemy) {
        e.m.sw(0xA2, 0);
        e.m.sw(0xA4, 0);
        e.m.sw(0xA6, 0);
        e.m.sb(0xA8, 0);

        let r = (self.rand() & 0xF) as u32;
        let p_count = self.enemy_c6_scan_players(e);
        let offset = self.assets.enemy.t.u16(0x3A59C, p_count * 2) as u32;
        let d0 = self.assets.enemy.t.u8(0x3A59C, offset + r);
        let mut target_idx = d0.wrapping_add(e.m.b(0xA8));
        if target_idx > 2 {
            target_idx = 0;
        }
        e.m.sb(0xA8, target_idx);

        let target_h = e.m.w(0xA2 + target_idx as usize * 2);
        e.m.sw(0x76, target_h);
    }

    /// 0x3A5D2: retarget when target is invalid or timer expired.
    pub fn enemy_c6_retarget(&mut self, e: &mut Enemy) {
        let r = (self.rand() & 0xF) as u32;
        let p_count = self.enemy_c6_scan_players(e);
        let offset = self.assets.enemy.t.u16(0x3A634, p_count * 2) as u32;
        let d0 = self.assets.enemy.t.u8(0x3A634, offset + r);
        let mut target_idx = d0.wrapping_add(e.m.b(0xA8));
        if target_idx > p_count as u8 {
            target_idx = target_idx.wrapping_sub(p_count as u8 + 1);
        }
        e.m.sb(0xA8, target_idx);

        let mut idx = (target_idx as usize) * 2;
        loop {
            if idx > 4 {
                idx = 0;
            }
            let target_h = e.m.w(0xA2 + idx);
            if target_h != 0 {
                e.m.sw(0x76, target_h);
                break;
            }
            idx += 2;
        }
    }

    /// 0x3A6EA: scan active players into $A2 array, returns count minus 1.
    fn enemy_c6_scan_players(&self, e: &mut Enemy) -> u32 {
        let mut count = 0u32;
        e.m.sw(0xA2, 0);
        e.m.sw(0xA4, 0);
        e.m.sw(0xA6, 0);
        for (i, p) in self.players.iter().enumerate() {
            if p.b.live != 0 {
                let off = 0xA2 + count as usize * 2;
                e.m.sw(off, Game::player_handle(i));
                count += 1;
            }
        }
        if e.m.w(0xA2) == 0 {
            e.m.sw(0xA2, Game::player_handle(0));
        }
        if count == 0 { 0 } else { count - 1 }
    }

    /// 0x3A78C: random slot initialization.
    pub fn enemy_c6_slot_init(&mut self, e: &mut Enemy) {
        let target = self.enemy_pick_target(&e.b); // 0x32032
        e.m.sw(0x76, target);
        let r = (self.rand() & 0x7) as u8;
        e.m.sb(0x7C, r); // slot = rand & 7
    }

    /// 0x3A7AA: advance approach slot (0..11).
    pub fn enemy_c6_advance_slot(&mut self, e: &mut Enemy) {
        let a9 = e.m.b(0xA9) as u32;
        let table_addr = if a9 == 0 { 0x3A7F8 } else { 0x3A8B8 };
        let cur_slot = e.m.b(0x7C);
        let mut slot_base = cur_slot;
        if slot_base >= 6 {
            slot_base -= 6;
        }
        let r = (self.rand() & 0x1F) as u32;
        let mut new_slot = self.assets.enemy.t.u8(table_addr, (slot_base as u32 * 32) | r);
        if cur_slot >= 6 {
            new_slot = new_slot.wrapping_add(6);
        }
        if new_slot >= 12 {
            new_slot = new_slot.wrapping_sub(12);
        }
        e.m.sb(0x7C, new_slot);
    }

    /// 0x3A9EE: compute destination coordinates from slot and steer.
    pub fn enemy_c6_steer_to_dest(&mut self, e: &mut Enemy) {
        let slot = (e.m.b(0x7C) as usize).min(11);
        let target_h = e.m.w(0x76);
        let target = self.body(target_h).clone();

        let dx = self.assets.enemy.t.i16(0x3AA2E, slot as u32 * 4);
        let dz = self.assets.enemy.t.i16(0x3AA2E, slot as u32 * 4 + 2);

        let dest_x = target.xi().wrapping_add(dx);
        let dest_z = target.zi().wrapping_add(dz);
        e.m.sw(0x88, dest_x as u16);
        e.m.sw(0x8A, dest_z as u16);

        self.enemy_c6_steer_angle(e);

        // Face target
        let facing = if target.xi() >= e.b.xi() { 1 } else { 0 };
        e.b.facing = facing;
    }

    /// 0x3AAF6: compute direction towards slot destination.
    pub fn enemy_c6_compute_dir_to_slot(&mut self, e: &mut Enemy) {
        let slot = (e.m.b(0x7C) as usize).min(11);
        let target_h = e.m.w(0x76);
        let target = self.body(target_h).clone();

        let dx = self.assets.enemy.t.i16(0x3AA2E, slot as u32 * 4);
        let dz = self.assets.enemy.t.i16(0x3AA2E, slot as u32 * 4 + 2);

        let dest_x = target.xi().wrapping_add(dx);
        let dest_z = target.zi().wrapping_add(dz);
        e.m.sw(0x88, dest_x as u16);
        e.m.sw(0x8A, dest_z as u16);

        let dir = Self::enemy_angle_to(&e.b, dest_x as u16, dest_z as u16); // 0x103A
        e.b.dir = dir;
    }

    /// 0x3AB62: adjust +$22 towards destination angle.
    fn enemy_c6_steer_angle(&self, e: &mut Enemy) {
        let dest_x = e.m.w(0x88) as i16;
        let dest_z = e.m.w(0x8A) as i16;
        let target_dir = Self::enemy_angle_to(&e.b, dest_x as u16, dest_z as u16); // 0x103A
        let cur_dir = e.b.dir;
        let diff = cur_dir.wrapping_sub(target_dir).wrapping_add(2) & 0x1F;
        if diff > 4 {
            let step: u8 = if diff >= 0x12 { 2 } else { 0xFE };
            e.b.dir = cur_dir.wrapping_add(step) & 0x1F;
        } else {
            e.b.dir = target_dir & 0x1F;
        }
    }

    /// 0x3AB94: check if reached slot (within dx 16, dz 12).
    fn enemy_c6_arrived_at_slot(&self, e: &Enemy) -> bool {
        let dest_x = e.m.w(0x88) as i16;
        let dest_z = e.m.w(0x8A) as i16;
        let dx = e.b.xi().wrapping_sub(dest_x).wrapping_add(8);
        if (dx as u16) >= 16 {
            return false;
        }
        let dz = e.b.zi().wrapping_sub(dest_z).wrapping_add(6);
        (dz as u16) < 12
    }

    /// 0x3ABEC: 3D hop integration (updates X and Z along +$22, integrates Y).
    pub fn enemy_c6_hop_integrate(&self, b: &mut super::Body) {
        let (sx, sz) = self.assets.enemy.speed(b.speed_table, b.dir);
        b.vx = sx.wrapping_add(b.ax);
        b.x = b.x.wrapping_add((b.vx as i32) << 8);
        b.vz = sz.wrapping_add(b.az);
        b.z = b.z.wrapping_add((b.vz as i32) << 8);

        b.vy = b.vy.wrapping_add(b.ay);
        b.y = b.y.wrapping_add((b.vy as i32) << 8);
    }

    /// 0x3AC3C: pick hop move type (0, 2, 4, 6) into +$AB.
    fn enemy_c6_pick_hop_type(&mut self, e: &mut Enemy) {
        let a9 = e.m.b(0xA9) as u32;
        let r = (self.rand() & 0x1F) as u32;
        let hop_type = self.assets.enemy.t.u8(0x3AC5A, (a9 << 5) | r);
        e.m.sb(0xAB, hop_type);
    }

    /// 0x3AC9A: face target player.
    fn enemy_c6_face_target(&self, e: &mut Enemy) {
        let target_h = e.m.w(0x76);
        let target = self.body(target_h);
        e.b.facing = if target.xi() >= e.b.xi() { 1 } else { 0 };
    }

    /// 0x3ACB2: set speed table by level.
    pub fn enemy_c6_set_speed_by_level(&mut self, e: &mut Enemy) {
        let speed_id = self.assets.enemy.t.u16(0x3ACC8, (self.w.level & 0x1F) as u32 * 2);
        self.enemy_set_speed(&mut e.b, speed_id);
    }

    /// 0x3AD08: check taunt condition.
    fn enemy_c6_check_taunt(&mut self, e: &mut Enemy) -> bool {
        let r = self.rand() & 0xF;
        if r >= 2 {
            return false;
        }
        let af = e.m.b(0xAF).wrapping_sub(1);
        if (af as i8) >= 0 {
            e.m.sb(0xAF, af);
            true
        } else {
            e.m.sb(0xAF, 0);
            false
        }
    }

    /// 0x3AD2E: check level threshold for leap frequency.
    fn enemy_c6_check_level_threshold(&mut self, e: &mut Enemy) {
        let r = (self.rand() & 0x1F) as u8;
        let lvl = (self.w.level & 0x1F) as u32;
        let threshold = self.assets.enemy.t.u8(0x3AD52, lvl);
        let a9 = if r < threshold { 2 } else { 0 };
        e.m.sb(0xA9, a9);
    }

    /// 0x3AD72: check attack start condition.
    fn enemy_c6_check_attack(&mut self, e: &Enemy) -> bool {
        let a9 = e.m.b(0xA9) as u32;
        let r = (self.rand() & 0x1F) as u8;
        let aggr = (self.rank.aggression & 0x7) as u32;
        let threshold = self.assets.enemy.t.u8(0x3ADA4, a9 * 4 + aggr);
        r < threshold
    }

    /// 0x3AFA4: check if Bludge should eat an item.
    fn enemy_c6_check_eat(&mut self, e: &mut Enemy) -> bool {
        if e.m.w(0xD6) >= 8 {
            return false;
        }
        let b6 = e.m.b(0xB6) as i8;
        if b6 < 0 {
            let total_items = self.combat.get(0x69EE).len() as u16;
            if total_items > e.m.w(0xD8) {
                e.m.sb(0xB6, 0);
            }
        }
        if e.m.b(0xB6) != 0 {
            return false;
        }

        let aggr = (self.rank.aggression & 0x7) as u32;
        let threshold = self.assets.enemy.t.u8(0x3AFF2, aggr);
        let r = (self.rand() & 0x1F) as u8;
        if r >= threshold {
            return false;
        }

        // 0x32762: global item check
        self.script_floor_items(true)
    }

    /// 0x3AFFA: scan and claim the lowest-kind floor item.
    fn enemy_c6_claim_item(&mut self, e: &mut Enemy) {
        let mut lowest_kind = 0xFFu8;
        let mut best_handle = 0u16;
        let mut count = 0u16;

        let list = self.combat.get(0x69EE).to_vec();
        for &h in &list {
            if let Some(Slot::Item(idx)) = slot_of(h) {
                let it = &self.items.slots[idx];
                if it.b.live == 0 {
                    continue;
                }
                if it.m.w(0xA0) == 0x20A || it.m.b(0xAF) != 0 {
                    count += 1;
                    continue;
                }
                let kind = (it.m.w(0xA0) & 0xFF) as u8;
                if kind < lowest_kind {
                    lowest_kind = kind;
                    best_handle = h as u16;
                }
            }
        }

        if count as usize == list.len() && !list.is_empty() {
            e.m.sb(0xB6, 0xFF);
        }
        e.m.sw(0xD8, count);
        e.m.sw(0xD0, best_handle);
        if best_handle != 0 {
            if let Some(Slot::Item(ii)) = slot_of(best_handle) {
                self.items.slots[ii].m.sb(0xAF, 1);
            }
        }
    }
}
