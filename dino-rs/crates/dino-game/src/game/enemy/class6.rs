//! Pool A class 6: Bludge (0x39272, 7,826 bytes) and class 0 Part 0 (0x33334).
//!
//! Bludge is a lizard-man dinosaur mutant. Mode 0 init and slot pairing (0x3929C),
//! four entrances (0x393BA), hop AI and states (0x39656), hit reactions (0x3A1C8),
//! and death (0x3A1E2).
//! Part 0 is the tongue controller paired with Bludge.

use super::Enemy;
use crate::game::draw::slot_frame;
use crate::game::{Game, Handle, Slot, slot_of};

impl Game {
    // =========================================================================
    // Class 0: Part 0 (0x33334)
    // =========================================================================

    /// 0x33334: Pool A class 0, handler for Bludge's tongue / companion object.
    pub fn enemy_class0_part(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => self.enemy_part0_init(e, i),
            2 => self.enemy_part0_active(e, h, i),
            _ => self.enemy_part0_free(e, i),
        }
    }

    /// 0x3334A: Part 0 init in mode 0.
    fn enemy_part0_init(&mut self, e: &mut Enemy, i: usize) {
        let parent_h = e.m.w(0xA0);
        let parent_k20 = self.body(parent_h).k20;
        // 0x3334E: cmpi.b #$18, $20(a0); beq.w $3355e
        // Checks high byte of parent's class word ($20); 0x18 is in low byte ($21).
        if (parent_k20 >> 8) == 0x18 {
            self.enemy_part0_free(e, i);
            return;
        }
        e.b.mode = 2; // addq.b #$2, $4(a6)
        e.b.sub = 4;  // move.b #$4, $5(a6)
        e.m.sw(0x84, 0x40); // move.w #$40, $84(a6)

        // 0x33368: jsr $48a8.l (draw_special_alloc)
        let Some(spec_idx) = self.draw_special_alloc(&mut e.m) else {
            // 0x33372: free parent and self
            if let Some(Slot::Enemy(pi)) = slot_of(parent_h) {
                self.enemies.slots[pi].b.mode = 6;
            }
            self.enemy_part0_free(e, i);
            return;
        };

        // 0x33380: clear 128 bytes in special slot
        for k in 0..128 {
            self.specials.slots[spec_idx].m.sb(k, 0);
        }

        e.b.anim_table = 0x3375C;
        // 0x33396: bsr.w $3356a (copy template into special slot Ram)
        self.enemy_part0_copy_template(spec_idx);

        // 0x3339A: jsr $16c6.l (init anim ptr to special slot Ram, frame to Ram + 10)
        let spec_h = self.specials.handle(spec_idx);
        e.m.sw(0xA4, (spec_h as u16).wrapping_add(12));
        e.b.frame = slot_frame(spec_idx);
        let (a, b, c, d) = self.specials.slots[spec_idx].record_params();
        (e.b.param_a, e.b.param_b, e.b.kind, e.b.timer) = (a, b, c, d);

        e.b.box_table = 0x337EA; // move.l #$337ea, $44(a6)
        e.b.draw_flags = 0x70;   // move.b #$70, $25(a6)
        e.b.visible = 1;        // move.b #$1, $1(a6)
        e.m.sb(0x7D, 0xFF);     // move.b #$ff, $7d(a6)
        e.m.sb(0x87, 0);        // move.b d0, $87(a6)
        e.b.b63 = 0xFF;         // move.b #$ff, $63(a6)
        e.m.sb(0xA6, 0);        // move.b d0, $a6(a6)
    }

    /// 0x3356A: copy template into special slot D-block RAM.
    fn enemy_part0_copy_template(&mut self, spec_idx: usize) {
        let spec_h = self.specials.handle(spec_idx);
        let spec_ram = &mut self.specials.slots[spec_idx].m;
        let a2 = (spec_h as u32).wrapping_add(10);
        spec_ram.sl(0, a2);
        for k in 4..126u32 {
            let b = self.assets.enemy.t.u8(0x3375C, k);
            spec_ram.sb(k as usize, b);
        }
    }

    /// 0x333D0: Part 0 mode 2 update.
    fn enemy_part0_active(&mut self, e: &mut Enemy, _h: Handle, i: usize) {
        let parent_h = e.m.w(0xA0);
        let parent_body = self.body(parent_h).clone();

        // 0x333D4: if parent dead or mode >= 4, free
        if parent_body.hp == 0 || parent_body.mode >= 4 || parent_body.live == 0 {
            self.enemy_part0_free(e, i);
            return;
        }

        // 0x333E6: check parent frame class 0x0E or hit reaction (0x0202)
        if parent_body.kind == 0x0E || (parent_body.mode == 2 && parent_body.sub == 2) {
            if let Some(Slot::Enemy(pi)) = slot_of(parent_h) {
                self.enemies.slots[pi].m.sb(0xC3, 0);
            }
            if let Some(Slot::Enemy(pi)) = slot_of(parent_h) {
                if self.enemies.slots[pi].m.b(0xC2) != 0 {
                    e.b.sub = 0;
                }
            }
        }

        // 0x33404: check parent $C3
        let parent_c3 = if let Some(Slot::Enemy(pi)) = slot_of(parent_h) {
            self.enemies.slots[pi].m.b(0xC3)
        } else {
            0
        };

        if parent_c3 == 0 {
            // 0x33436: move.b $c3(a4), $a6(a6); rts (does not draw or push hurtable)
            e.m.sb(0xA6, 0);
            return;
        }

        // Parent $C3 != 0 tongue attack handling (M7-08).
        e.m.sb(0xA6, parent_c3);
    }

    /// 0x3355E: Part 0 free.
    fn enemy_part0_free(&mut self, e: &mut Enemy, i: usize) {
        self.draw_special_free(&mut e.m); // 0x498C
        self.enemy_free_pool_a(e, i);      // 0x48FC
    }

    // =========================================================================
    // Class 6: Bludge (0x39272)
    // =========================================================================

    /// 0x39272: Pool A class 6 Bludge main update.
    pub fn enemy_class6(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => self.enemy_c6_mode0(e, h, i),
            2 => {
                // 0x39656: active mode 2
                self.enemy_kill_all_check(e); // 0x326F8
                match e.b.sub {
                    0 => self.enemy_c6_alive(e, h),
                    2 => self.enemy_c6_hit(e, h),
                    4 => {} // 0x3A0B4 tongue grab on player (M7-08)
                    6 => self.enemy_grabbed(e, h), // 0x2B35A
                    _ => {}
                }
                self.enemy_clamp_to_playfield(e, i); // 0x193E
            }
            4 => self.enemy_c6_dying(e, h),
            _ => self.enemy_c6_free(e, i), // mode 6
        }
    }

    /// 0x39286: Bludge mode 0 dispatch on +$05.
    fn enemy_c6_mode0(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.sub {
            0 => self.enemy_c6_alloc_pair(e, h, i),
            2 => self.enemy_c6_init(e, h),
            4 => self.enemy_c6_entrance(e, h),
            6 => self.enemy_c6_entrance_end(e, h),
            _ => {}
        }
    }

    /// 0x3929C: mode 0 sub 0 slot allocation & handle pairing.
    fn enemy_c6_alloc_pair(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        // 0x3929C: jsr $483c.l (alloc slot in pool A)
        let Some(new_idx) = self.enemies.alloc() else {
            self.enemy_free_pool_a(e, i); // 0x3A228
            return;
        };
        let new_h = self.enemies.handle(new_idx);

        // 0x392A8: cmpa.w a0, a1; bcs 0x392b0; exg.l a0, a1
        // a0 is ALWAYS max(h, new_h), a1 is ALWAYS min(h, new_h).
        let (part0_h, bludge_h, current_is_part0) = if (h as u16) <= (new_h as u16) {
            (h, new_h, true)
        } else {
            (new_h, h, false)
        };

        if current_is_part0 {
            // Currently updating slot (e) is Part 0 (lower handle)
            let old_variant = e.b.var26;
            let old_dir = e.b.dir;
            let (old_x, old_y, old_z) = (e.b.xi(), e.b.yi(), e.b.zi());
            let old_drop = e.m.b(0x96);

            // a1 (Part 0): class 0, sub 0, +$A0 = bludge_h
            e.b.k20 = 0;
            e.b.sub = 0;
            e.m.sw(0xA0, bludge_h as u16);

            // a0 (Bludge): class 0x18, sub 2, +$C0 = part0_h, coordinates from spawning slot
            let other = &mut self.enemies.slots[new_idx];
            other.b.live = 1;
            other.b.k20 = 0x18;
            other.b.mode = 0;
            other.b.sub = 2;
            other.b.var26 = old_variant;
            other.b.dir = old_dir;
            other.b.set_xi(old_x);
            other.b.set_yi(old_y);
            other.b.set_zi(old_z);
            other.m.sw(0xC0, part0_h as u16);
            other.m.sb(0xC3, 0);
            other.m.sb(0x96, old_drop);
        } else {
            // Currently updating slot (e) is Bludge (higher handle)
            let other_h = part0_h;
            e.b.live = 1;
            e.b.k20 = 0x18;
            e.b.sub = 2;
            e.m.sw(0xC0, other_h as u16);
            e.m.sb(0xC3, 0);

            // Other slot is Part 0 (lower handle)
            let other = &mut self.enemies.slots[new_idx];
            other.b.live = 1;
            other.b.k20 = 0;
            other.b.mode = 0;
            other.b.sub = 0;
            other.m.sw(0xA0, h as u16);
        }
    }

    /// 0x392FE: Bludge mode 0 sub 2 init.
    fn enemy_c6_init(&mut self, e: &mut Enemy, _h: Handle) {
        // 0x39304: draw_special_alloc (0x48A8)
        if let Some(spec_idx) = self.draw_special_alloc(&mut e.m) {
            let spec_ram = &mut self.specials.slots[spec_idx].m;
            for k in 0..16 {
                spec_ram.sb(k, 0);
            }
            let spec_h = self.specials.handle(spec_idx);
            let buf_addr = (spec_h as u16).wrapping_add(16);
            e.m.sw(0xD2, buf_addr);
            e.m.sw(0xD4, buf_addr);
            e.m.sw(0xD6, 0);
            e.m.sw(0xD8, 0);
        } else {
            e.m.sb(0xB6, 1);
        }

        e.b.id = 0x10;             // move.b #$10, $3(a6)
        e.m.sw(0x84, 0x40);         // move.w #$40, $84(a6)
        e.b.anim_table = 0x3A234;   // move.l #$3a234, $40(a6)
        self.enemy_anim(&mut e.b, 0); // jsr $120e.l
        e.b.box_table = 0x103000;   // move.l #$103000, $44(a6)
        e.b.multi_box = false;      // clr.b $4a(a6)

        // 0x3935A: bsr.w $3acb2 (speed table by level)
        self.enemy_c6_set_speed_by_level(e);

        // 0x39360: jsr $31424.l (HP by level: 76 at level 0)
        self.enemy_set_hp(&mut e.b);

        e.b.stun = 10;              // move.b #$a, $78(a6)
        e.m.sb(0x79, 0);            // clr.b $79(a6)
        e.m.sb(0x7A, 0);            // clr.b $7a(a6)
        e.m.sb(0x7B, 0);            // clr.b $7b(a6)
        e.m.sb(0x7D, 0);            // clr.b $7d(a6)
        e.m.sb(0x87, 0);            // clr.b $87(a6)
        e.b.b63 = 0;                // clr.b $63(a6)
        e.m.sb(0xA8, 0);            // clr.b $a8(a6)
        e.m.sb(0x72, 0xB4);         // move.b #$b4, $72(a6)
        e.m.sb(0xAF, 3);            // move.b #$3, $af(a6)
        e.m.sb(0xC2, 0);            // clr.b $c2(a6)
        e.m.sb(0xC3, 0);            // clr.b $c3(a6)

        let target = self.enemy_pick_target(&e.b); // jsr $32032.l
        e.m.sw(0x76, target);       // move.w a0, $76(a6)

        self.enemy_snap_to_floor(e); // jsr $3140c.l
        e.m.sb(0x7D, 0);            // move.b #$0, $7d(a6)
        e.b.sub = e.b.sub.wrapping_add(2); // addq.b #$2, $5(a6) -> sub 4
    }

    /// 0x393BA: mode 0 sub 4 entrances dispatch by low byte of variant.
    fn enemy_c6_entrance(&mut self, e: &mut Enemy, h: Handle) {
        match (e.b.var26 & 0xFF) as u8 {
            0 => self.enemy_c6_entrance_end(e, h),
            2 => self.enemy_c6_entrance_2(e, h),
            4 => self.enemy_c6_entrance_4(e, h),
            6 => self.enemy_c6_entrance_6(e, h),
            _ => self.enemy_c6_entrance_end(e, h),
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x393E2: Entrance 2 arc jump from the back.
    fn enemy_c6_entrance_2(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => {
                // 0x393F6: step 0
                e.b.act = 2;
                self.enemy_anim(&mut e.b, 1);
                let center = self.cam.s2.cam_x.wrapping_add(0xC0);
                let d0 = (center as i16).wrapping_sub(e.b.xi());
                let vx = if d0 < 0 { 0x180 } else { -0x180 };
                e.b.vx = vx;
                e.b.ax = 0;
                e.b.vy = 0x400;
                e.b.ay = -0x48;
                e.b.vz = -0x100;
                e.b.az = 0;
                e.b.set_yi(0x18);
                e.b.set_zi(0x78);
                e.b.airborne = 1;
                e.m.sb(0x80, 0x14); // 20 frames
                self.enemy_c6_entrance_2_flight(e, h);
            }
            2 => self.enemy_c6_entrance_2_flight(e, h),
            4 => {
                // 0x39486: landing anim tick
                if self.enemy_step(&mut e.b) {
                    self.enemy_c6_entrance_end(e, h);
                }
            }
            _ => self.enemy_c6_entrance_end(e, h),
        }
    }

    fn enemy_c6_entrance_2_flight(&mut self, e: &mut Enemy, h: Handle) {
        // 0x3944C: in flight
        if e.b.airborne != 0 {
            if e.b.param_b == 0 {
                e.b.param_b = 0;
                e.b.motion(); // 0x189C
            }
            self.enemy_step(&mut e.b); // 0x121E
            let t = e.m.b(0x80).wrapping_sub(1);
            e.m.sb(0x80, t);
            if (t as i8) < 0 {
                e.m.sb(0x80, 0);
                if let Some(o) = self.terrain_collide(&mut e.b, h, false) {
                    e.m.sw(0x94, o);
                }
            }
        } else {
            // 0x3947A: landed
            e.b.act = 4;
            self.enemy_anim(&mut e.b, 2); // landing anim
        }
    }

    /// 0x39492: Entrance 6 high leap.
    fn enemy_c6_entrance_6(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => {
                // 0x394A6: step 0
                e.b.act = 2;
                self.enemy_anim(&mut e.b, 0x13);
                let center = self.cam.s2.cam_x.wrapping_add(0xC0);
                let d0 = (center as i16).wrapping_sub(e.b.xi());
                let vx = if d0 < 0 { 0x180 } else { -0x180 };
                e.b.vx = vx;
                e.b.ax = 0;
                e.b.vy = 0x900;
                e.b.ay = -0x40;
                e.b.vz = 0x100;
                e.b.az = 0;
                e.b.airborne = 1;
                e.m.sb(0x80, 0x14); // 20 frames
                self.enemy_c6_entrance_6_flight(e, h);
            }
            2 => self.enemy_c6_entrance_6_flight(e, h),
            4 => {
                // 0x3952A: landing anim tick
                if self.enemy_step(&mut e.b) {
                    self.enemy_c6_entrance_end(e, h);
                }
            }
            _ => self.enemy_c6_entrance_end(e, h),
        }
    }

    fn enemy_c6_entrance_6_flight(&mut self, e: &mut Enemy, h: Handle) {
        // 0x394F0: in flight
        if e.b.airborne != 0 {
            if e.b.param_b == 0 {
                e.b.param_b = 0;
                e.b.motion(); // 0x189C
            }
            self.enemy_step(&mut e.b); // 0x121E
            let t = e.m.b(0x80).wrapping_sub(1);
            e.m.sb(0x80, t);
            if (t as i8) < 0 {
                e.m.sb(0x80, 0);
                if let Some(o) = self.terrain_collide(&mut e.b, h, false) {
                    e.m.sw(0x94, o);
                }
            }
        } else {
            // 0x3951E: landed
            e.b.act = 4;
            self.enemy_anim(&mut e.b, 2); // landing anim
        }
    }

    /// 0x39536: Entrance 4 crouch wait.
    fn enemy_c6_entrance_4(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => {
                // 0x3954C: step 0
                e.b.act = 2;
                let r = (self.rand() & 0x3E) as u32;
                let wait = self.assets.enemy.t.u16(0x3957E, r);
                e.m.sw(0xB8, wait);
                e.b.facing = 0;
                if e.m.b(0x5B) == 0 {
                    self.enemy_anim(&mut e.b, 0x11);
                } else {
                    self.enemy_anim(&mut e.b, 0x12);
                }
            }
            2 => {
                // 0x395BE: step 2, check camera proximity
                let center = self.cam.s2.cam_x.wrapping_add(0xC0);
                let d0 = (center as u16).wrapping_sub(e.b.xi() as u16).wrapping_add(0xC0);
                if d0 < 0x180 {
                    e.b.act = 4;
                }
                // 0x395FA: check player proximity
                if self.enemy_c6_check_player_near(e) {
                    self.enemy_c6_entrance_end(e, h);
                }
            }
            4 => {
                // 0x395DC: step 4, countdown timer $B8
                let t = e.m.w(0xB8).wrapping_sub(1);
                e.m.sw(0xB8, t);
                if (t as i16) < 0 {
                    e.b.act = 6;
                } else if self.enemy_c6_check_player_near(e) {
                    self.enemy_c6_entrance_end(e, h);
                }
            }
            6 => {
                // 0x395EC: step 6, anim tick
                if self.enemy_step(&mut e.b) {
                    self.enemy_c6_entrance_end(e, h);
                }
            }
            _ => self.enemy_c6_entrance_end(e, h),
        }
    }

    /// 0x395FA: check if any active player is within 0x50 pixels.
    fn enemy_c6_check_player_near(&self, e: &Enemy) -> bool {
        let ex = e.b.xi();
        let ez = e.b.zi();
        for p in &self.players {
            if p.b.live != 0 {
                let dx = (p.b.xi().wrapping_sub(ex)).abs() as u16;
                let dz = (p.b.zi().wrapping_sub(ez)).abs() as u16;
                if dx.wrapping_add(dz) < 0x50 {
                    return true;
                }
            }
        }
        false
    }

    /// 0x39644: finish entrance, pick target & slot, reset to AI.
    pub fn enemy_c6_entrance_end(&mut self, e: &mut Enemy, _h: Handle) {
        self.enemy_c6_target_init(e); // 0x3A54C
        self.enemy_c6_slot_init(e);   // 0x3A77E, 0x3A78C
        self.enemy_c6_reset_ai(e);    // 0x396C2
    }

    /// 0x396C2: reset to AI mode 2 sub 0 act 0 step 0.
    pub fn enemy_c6_reset_ai(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0000); // move.l #$2000000, $4(a6)
        e.m.sb(0x7D, 0);            // move.b #$0, $7d(a6)
        self.enemy_anim(&mut e.b, 0); // jsr $120e.l
        e.m.sb(0xDA, 0);            // clr.b $da(a6)
    }

    /// 0x3A1C8: Bludge hit reaction (mode 2 sub 2).
    fn enemy_c6_hit(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.hp != 0 && e.m.b(0xDA) != 0 {
            e.m.sb(0xDA, 0);
            // vomit item in M7-09 (0x3B086)
        }
        self.enemy_hit_reaction(e, h); // 0x2A606
    }

    /// 0x3A1E2: Bludge dying (mode 4).
    fn enemy_c6_dying(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => {
                // 0x3A1F2: award kill score 8,000
                self.enemy_award_kill_score(&mut e.b); // 0x325E6
                e.m.sw(0x80, 0x78);                    // 120 frames
                e.b.sub = e.b.sub.wrapping_add(2);      // sub 2
                // vomit items in M7-09 (0x3B086, 0x3B0D8)
            }
            2 => {
                // 0x3A20A: countdown timer
                let t = e.m.w(0x80).wrapping_sub(1);
                if (t as i16) < 0 {
                    e.b.mode = e.b.mode.wrapping_add(2); // mode 6
                } else {
                    e.m.sw(0x80, t);
                    // Blinking on odd frame counts
                    if (t & 1) != 0 {
                        self.enemy_draw_check(&mut e.b, h);
                    }
                }
            }
            _ => {}
        }
    }

    /// 0x3A228: Bludge mode 6 free.
    fn enemy_c6_free(&mut self, e: &mut Enemy, i: usize) {
        self.draw_special_free(&mut e.m); // 0x498C
        self.enemy_free_pool_a(e, i);      // 0x48FC
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_bludge_entrances() {
        let rom = Path::new("../../../dino.zip");
        if !rom.exists() {
            return;
        }
        let assets = Box::leak(Box::new(dino_extract::extract(rom).unwrap()));
        let mut g = Game::new(assets);

        // Test Entrance 0 (bl-v0-l0): plain hop in (default entrance: transitions to AI mode 2 sub 0)
        let mut e0 = Enemy::default();
        e0.b.anim_table = 0x3A234;
        e0.b.var26 = 0;
        e0.b.sub = 4;
        g.enemy_c6_mode0(&mut e0, 0xDC34, 0);
        assert_eq!(e0.b.mode, 2);
        assert_eq!(e0.b.sub, 0);

        // Test Entrance 4 (bl-v4-l0): crouch in place (initializes to act 2)
        let mut e4 = Enemy::default();
        e4.b.anim_table = 0x3A234;
        e4.b.var26 = 4;
        e4.b.sub = 4;
        g.enemy_c6_mode0(&mut e4, 0xDC34, 0);
        assert_eq!(e4.b.act, 2);

        // Test Entrance 6 (bl-v6-l0): high leap (vy 0x900, ay -0x40, |vx| 0x180, vz +0x100)
        let mut e6 = Enemy::default();
        e6.b.anim_table = 0x3A234;
        e6.b.var26 = 6;
        e6.b.sub = 4;
        g.enemy_c6_mode0(&mut e6, 0xDC34, 0);
        assert_eq!(e6.b.vy, 0x900);
        assert_eq!(e6.b.ay, -0x40);
        assert_eq!(e6.b.vx.abs(), 0x180);
        assert_eq!(e6.b.vz, 0x100);
    }
}
