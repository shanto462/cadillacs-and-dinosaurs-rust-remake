//! Pool A class 82: Zeke the pteranodon (0x606D4, 3,720 bytes).
//!
//! Init (0x606E8), patrol mode 0 (0x60804), swoop mode 2 (0x60906),
//! triple peck mode 4 (0x60A76), dive mode 6 (0x60BD4), leave mode 8 (0x60D26),
//! hit reaction (0x60F1E), dive grab (0x60FEE), dying mode 4 (0x61140).

use super::Enemy;
use crate::game::{Game, Handle};

impl Game {
    /// 0x606D4: class 82 Zeke main update.
    pub fn enemy_class82(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => self.enemy_zeke_init(e),
            2 => {
                // 0x60778: active
                match e.b.sub {
                    0 => {
                        // 0x607A0: normal actions
                        let poise_t = e.m.b(0x79);
                        if poise_t != 0 {
                            let t = poise_t - 1;
                            e.m.sb(0x79, t);
                            if t == 0 {
                                e.b.stun = 10;
                            }
                        }
                        let wait = e.m.b(0x7A);
                        if wait != 0 {
                            e.m.sb(0x7A, wait - 1);
                            // 0x607BE: skip to 0x607D4 (draw only)
                            self.enemy_draw_check(&mut e.b, h);
                        } else {
                            match e.m.w(0xA0) {
                                0 => self.enemy_zeke_mode_patrol(e),
                                2 => self.enemy_zeke_mode_swoop(e),
                                4 => self.enemy_zeke_mode_peck(e),
                                6 => self.enemy_zeke_mode_dive(e),
                                8 => self.enemy_zeke_mode_leave(e),
                                _ => self.enemy_zeke_reset_patrol(e),
                            }
                            self.terrain_floor_only(&mut e.b); // 0x12D4E
                            self.enemy_draw_check(&mut e.b, h); // 0x49CA
                        }
                    }
                    2 => {
                        // 0x60F1E: hit reaction
                        self.enemy_zeke_sub_hit(e, h);
                        self.terrain_floor_only(&mut e.b);
                        self.enemy_draw_check(&mut e.b, h);
                        // 0x60F2A: the inner step handler ends with `jmp $49ca`, so its rts comes
                        // back to 0x60F2A, which runs `jmp $49ca` again: the draw list gets Zeke twice.
                        self.enemy_draw_check(&mut e.b, h);
                    }
                    4 => {
                        // 0x60FEE: dive grab
                        self.enemy_zeke_sub_grab(e, h);
                        self.terrain_floor_only(&mut e.b);
                        self.enemy_draw_check(&mut e.b, h);
                    }
                    _ => {}
                }
                self.enemy_monster_register(h); // 0x60784
                self.enemy_despawn_if_far(e, i); // 0x6078A
            }
            4 => self.enemy_zeke_mode_dying(e, h, i),
            _ => self.enemy_free_pool_a(e, i), // 0x611CC: mode 6 jmp $48FC
        }
    }

    /// 0x606E8: Zeke class 82 init.
    fn enemy_zeke_init(&mut self, e: &mut Enemy) {
        e.b.id = 0x21; // move.b #$21, $3(a6)
        e.m.sw(0x84, 0x84); // move.w #$84, $84(a6)
        e.b.anim_table = 0x611D2; // move.l #$611d2, $40(a6)
        self.enemy_set_hp(&mut e.b); // jsr $31424.l
        e.b.box_table = 0x103000; // move.l #$103000, $44(a6)
        e.b.multi_box = false; // clr.b $4a(a6)
        e.b.stun = 10; // move.b #$a, $78(a6)
        e.m.sb(0x79, 0); // clr.b $79(a6)
        e.m.sb(0x7A, 0); // clr.b $7A(a6)
        e.m.sb(0x87, 0); // clr.b $87(a6)
        e.b.hitstop = 0; // clr.b $59(a6)
        e.m.sb(0x7D, 0xFF); // move.b #$ff, $7d(a6)
        e.m.sw(0xA0, 0); // move.w #$0, $a0(a6)
        e.m.sb(0xA2, 0); // clr.b $a2(a6)
        e.m.sb(0xA3, 0); // clr.b $a3(a6)
        e.m.sw(0xA4, 0); // clr.w $a4(a6)
        if self.w.stage == 7 {
            // cmpi.b #$7, $4d9(a5); bne ...; move.b #$9d, $23(a6)
            e.b.palette_override = 0x9D;
        }
        self.enemy_anim(&mut e.b, 0); // moveq #0, d0; jsr $120e.l
        // 0x60756: jsr $32032.l; move.w a0, $76(a6). Plain pick: no 0x60D7C rule here (no $116 test or set).
        let target = self.enemy_pick_target(&e.b);
        e.m.sw(0x76, target);
        self.enemy_snap_to_floor(e); // jsr $3140c.l
        e.b.airborne = 1; // move.b #$1, $51(a6)
        e.b.set_yi(e.b.yi().wrapping_add(0x140)); // addi.w #$140, $c(a6)
        e.b.mode = e.b.mode.wrapping_add(2); // addq.b #$2, $4(a6) -> mode 2
    }

    /// 0x60D7C: target rule (the patrol act 0 only; the init 0x606E8 has its own plain pick).
    fn enemy_zeke_pick_target(&mut self, e: &mut Enemy) {
        let target = self.enemy_pick_target(&e.b); // jsr $32032.l
        let mut chosen = target;
        if let Some(crate::game::Slot::Player(pi)) = crate::game::slot_of(target)
            && self.players[pi].b116 != 0 {
                // 0x60D82..0x60DCC: each step takes d0 = who(a0) + 1 & 3 and a0 = table[d0] (0x60DDC: P1, P2, P3, P1).
                // Candidates 1 and 2 must be live (tst.b $0(a0)); candidate 3 is taken whether it is live or not.
                let mut cand = pi;
                for k in 0..3 {
                    let d0 = self.players[cand].b.who.wrapping_add(1) & 3;
                    cand = if d0 >= 3 { 0 } else { d0 as usize };
                    if k < 2 && self.players[cand].b.live == 0 {
                        continue;
                    }
                    chosen = Game::player_handle(cand);
                    break;
                }
            }
        if let Some(crate::game::Slot::Player(pi)) = crate::game::slot_of(chosen) {
            self.players[pi].b116 = 1; // 0x60DD0: move.b #$1, $116(a0)
        }
        e.m.sw(0x76, chosen);
    }

    /// 0x60DEC: test if current altitude target has been reached. Returns carry.
    fn enemy_zeke_test_altitude(&self, e: &Enemy) -> bool {
        let alt_idx = (e.m.b(0xA2) as u32) * 2;
        let target_h = self.assets.enemy.t.u16(0x6141A, alt_idx) as i16;
        let target_y = e.b.floor.wrapping_add(e.b.floor2).wrapping_add(target_h);
        let self_y = e.b.yi();
        if e.b.vy < 0 {
            // falling: 060e16: cmp.w $c(a6), d0; blt.w $60e24
            self_y <= target_y
        } else {
            // climbing: 060e08: cmp.w $c(a6), d0; bge.w $60e24
            self_y > target_y
        }
    }

    /// 0x60E28: pick new altitude target from transition table 0x6137A.
    fn enemy_zeke_pick_altitude(&mut self, e: &mut Enemy) {
        let old_idx = e.m.b(0xA2);
        let r = self.rand();
        let off = ((old_idx as u32) << 5) + ((r as u32) & 0x1F);
        let new_idx = self.assets.enemy.t.u8(0x6137A, off);
        e.m.sb(0xA2, new_idx);
        let mut vy: i16 = 0xC0; // 192
        if old_idx < new_idx {
            vy = -vy;
        }
        e.b.vy = vy;
    }

    /// 0x60E5E: set flap animation based on vertical velocity.
    fn enemy_zeke_flap_anim(&mut self, e: &mut Enemy) {
        let anim = if e.b.vy < 0 { 0 } else { 1 };
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x60E70: approach target slot. Returns true if reached.
    fn enemy_zeke_approach(&mut self, e: &mut Enemy, row_offset: usize) -> bool {
        let t = e.m.b(0xA3);
        let (new_t, borrow) = t.overflowing_sub(1);
        e.m.sb(0xA3, new_t);
        if !borrow {
            return false;
        }
        e.m.sb(0xA3, 6);

        let target = e.m.w(0x76);
        let tb = self.body(target);
        let mut target_x = tb.xi();
        let target_z = tb.zi();

        let mut dx = self.assets.enemy.t.i16(0x614DC, row_offset as u32);
        let cam_center = (self.cam.s2.cam_x as i16).wrapping_add(0xC0);
        if cam_center >= target_x {
            dx = -dx;
        }
        target_x = target_x.wrapping_add(dx);
        e.m.sw(0x88, target_x as u16);
        e.m.sw(0x8A, target_z as u16);

        let angle = Game::enemy_angle_to(&e.b, target_x as u16, target_z as u16);
        let cur_dir = e.b.dir;
        let diff = angle.wrapping_sub(cur_dir) & 0x1F;
        let new_dir = if diff < 16 {
            if diff > 2 {
                (cur_dir.wrapping_add(2)) & 0x1F
            } else {
                angle
            }
        } else {
            if diff < 30 {
                (cur_dir.wrapping_sub(2)) & 0x1F
            } else {
                angle
            }
        };
        e.b.dir = new_dir;

        let x_off = self.assets.enemy.t.i16(0x614DC, (row_offset + 2) as u32);
        let x_limit = self.assets.enemy.t.u16(0x614DC, (row_offset + 4) as u32);
        let z_off = self.assets.enemy.t.i16(0x614DC, (row_offset + 6) as u32);
        let z_limit = self.assets.enemy.t.u16(0x614DC, (row_offset + 8) as u32);

        let diff_x = target_x.wrapping_sub(e.b.xi()).wrapping_add(x_off) as u16;
        if diff_x > x_limit {
            return false;
        }
        let diff_z = target_z.wrapping_sub(e.b.zi()).wrapping_add(z_off) as u16;
        if diff_z > z_limit {
            return false;
        }
        true
    }

    /// 0x607E6: reset to patrol mode.
    fn enemy_zeke_reset_patrol(&mut self, e: &mut Enemy) {
        e.m.sw(0xA0, 0); // patrol mode
        e.b.act = 0; // $6(a6) = 0
        e.b.step = 0; // $7(a6) = 0
        e.b.vy = 0;
        e.b.ay = 0;
        e.b.ax = 0;
        e.b.az = 0;
    }

    /// 0x608BE: choose attack based on altitude index and random roll.
    fn enemy_zeke_pick_attack(&mut self, e: &mut Enemy) {
        let alt_idx = e.m.b(0xA2) as u32;
        let r = self.rand();
        let off = (alt_idx << 5) + ((r as u32) & 0x1F);
        let choice = self.assets.enemy.t.u8(0x6143C, off);
        match choice {
            0 => {
                // Swoop (Mode 2)
                e.m.sw(0xA0, 2);
                e.b.act = 0;
                e.b.step = 0;
            }
            1 => {
                // Triple peck (Mode 4)
                e.m.sw(0xA0, 4);
                e.b.act = 0;
                e.b.step = 0;
            }
            _ => {
                // Dive (Mode 6)
                e.m.sw(0xA0, 6);
                e.b.act = 0;
                e.b.step = 0;
            }
        }
    }

    /// 0x60804: mode 0 patrol.
    fn enemy_zeke_mode_patrol(&mut self, e: &mut Enemy) {
        let w5c = e.m.w(0x5C);
        if w5c != 0 && self.cam.s2.cam_x >= w5c {
            // 060810: ble.w $60cfc -> enter Mode 8 leave
            e.m.sw(0xA0, 8);
            e.b.act = 0;
            e.b.step = 0;
            e.b.vy = 0x100;
            e.b.ay = 0x10;
            e.b.vx = 0;
            e.b.ax = 0;
            e.b.vz = 0;
            e.b.az = 0;
            return;
        }

        match e.b.act {
            0 => {
                // 060826
                self.enemy_zeke_pick_target(e);
                self.enemy_zeke_pick_altitude(e);
                self.enemy_zeke_flap_anim(e);
                self.enemy_set_speed(&mut e.b, 3);
                e.m.sb(0xA3, 0);
                e.m.sb(0x80, 180);
                e.m.sw(0xA4, 600);
                e.b.act = 2;
            }
            2 => {
                // 060850
                if self.enemy_step(&mut e.b) {
                    if self.enemy_zeke_test_altitude(e) {
                        self.enemy_zeke_pick_altitude(e);
                        self.enemy_zeke_flap_anim(e);
                    }
                    let target = e.m.w(0x76);
                    if self.body(target).live == 0 {
                        e.b.act = 0;
                        e.b.step = 0;
                        return;
                    }
                }
                // 06087c: attack roll countdown
                let t = e.m.b(0x80);
                let (new_t, borrow) = t.overflowing_sub(1);
                e.m.sb(0x80, new_t);
                if borrow {
                    e.m.sb(0x80, 180);
                    let level_band = ((self.w.level as usize) & 0x18) >> 1;
                    let mask = self.assets.enemy.t.u32(0x6142C, level_band as u32);
                    let r = self.rand();
                    let bit = (r as u32) & 0x1F;
                    if (mask & (1 << bit)) != 0 {
                        self.enemy_zeke_pick_attack(e);
                        return;
                    }
                }
                // 0608ac
                let arrived = self.enemy_zeke_approach(e, 0);
                if arrived {
                    let a4 = e.m.w(0xA4);
                    let (new_a4, borrow) = a4.overflowing_sub(1);
                    e.m.sw(0xA4, new_a4);
                    if borrow {
                        self.enemy_zeke_pick_attack(e);
                        return;
                    }
                } else {
                    e.m.sw(0xA4, 600);
                }
                self.enemy_move_dir(&mut e.b);
                e.b.motion_y();
            }
            _ => {
                e.b.act = 0;
                e.b.step = 0;
            }
        }
    }

    /// 0x60906: mode 2 swoop.
    fn enemy_zeke_mode_swoop(&mut self, e: &mut Enemy) {
        match e.b.act {
            0 => {
                // 06091c
                self.enemy_set_speed(&mut e.b, 3);
                e.b.act = 2;
            }
            2 => {
                // 06092a
                if self.enemy_step(&mut e.b)
                    && self.enemy_zeke_test_altitude(e) {
                        self.enemy_zeke_pick_altitude(e);
                        self.enemy_zeke_flap_anim(e);
                    }
                let arrived = self.enemy_zeke_approach(e, 0xA);
                if !arrived {
                    self.enemy_move_dir(&mut e.b);
                    e.b.motion_y();
                } else {
                    // Launch swoop
                    let (mut vx, mut ax) = (-256i16, 0i16);
                    let target = e.m.w(0x76);
                    let target_x = self.body(target).xi();
                    let mut facing = 0u8;
                    if target_x >= e.b.xi() {
                        facing = 1;
                        vx = -vx;
                        ax = -ax;
                    }
                    e.b.facing = facing;
                    e.b.vx = vx;
                    e.b.ax = ax;
                    e.b.vy = 768;
                    e.b.ay = -32;
                    e.b.vz = 0;
                    e.b.az = 0;
                    e.m.sw(0xA8, e.b.yi() as u16);
                    e.b.atk = 0x5B;
                    e.b.knock = 0;
                    e.b.hitstop = 0;
                    e.b.serial = self.w.attack_serial;
                    self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
                    e.b.act = 4;
                    self.enemy_anim(&mut e.b, 3);
                }
            }
            4 => {
                // 0609c2
                e.b.motion();
                if e.b.vy < 0 {
                    let launch_y = e.m.w(0xA8) as i16;
                    if e.b.yi() <= launch_y {
                        let mut vx = -256i16;
                        let mut ax = 0i16;
                        if e.b.facing != 0 {
                            vx = -vx;
                            ax = -ax;
                        }
                        e.b.vx = vx;
                        e.b.ax = ax;
                        e.b.ay = -32;
                        e.b.act = 6;
                    }
                }
            }
            6 => {
                // 060a02
                e.b.motion();
                let height = e.b.yi().wrapping_sub(e.b.floor.wrapping_add(e.b.floor2));
                if height < 48 {
                    e.b.ay = 80;
                    e.m.sw(0xA6, 1280);
                    self.enemy_anim(&mut e.b, 4);
                    e.b.act = 8;
                }
            }
            8 => {
                // 060a30
                e.b.motion();
                let cap = e.m.w(0xA6) as i16;
                if e.b.vy > cap {
                    e.b.vy = cap;
                    e.b.ay = 0;
                }
                if self.enemy_step(&mut e.b) {
                    let height = e.b.yi().wrapping_sub(e.b.floor.wrapping_add(e.b.floor2));
                    if height > 192 {
                        self.enemy_zeke_reset_patrol(e);
                    }
                }
            }
            _ => self.enemy_zeke_reset_patrol(e),
        }
    }

    /// 0x60A76: mode 4 triple peck.
    fn enemy_zeke_mode_peck(&mut self, e: &mut Enemy) {
        match e.b.act {
            0 => {
                // 060a8a
                self.enemy_set_speed(&mut e.b, 3);
                e.m.sb(0xAA, 3);
                e.b.act = 2;
            }
            2 => {
                // 060a9e
                if self.enemy_step(&mut e.b)
                    && self.enemy_zeke_test_altitude(e) {
                        self.enemy_zeke_pick_altitude(e);
                        self.enemy_zeke_flap_anim(e);
                    }
                let arrived = self.enemy_zeke_approach(e, 0x14);
                if !arrived {
                    self.enemy_move_dir(&mut e.b);
                    e.b.motion_y();
                } else {
                    // Launch peck
                    let (mut vx, mut ax) = (-128i16, 0i16);
                    let target = e.m.w(0x76);
                    let target_x = self.body(target).xi();
                    let mut facing = 0u8;
                    if target_x >= e.b.xi() {
                        facing = 1;
                        vx = -vx;
                        ax = -ax;
                    }
                    e.b.facing = facing;
                    e.b.vx = vx;
                    e.b.ax = ax;
                    e.b.vy = 768;
                    e.b.ay = -64;
                    e.b.vz = 0;
                    e.b.az = 0;
                    e.b.atk = 0x5C;
                    e.b.knock = 0;
                    e.b.hitstop = 0;
                    e.b.serial = self.w.attack_serial;
                    self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
                    if e.m.b(0xAA) == 1 {
                        e.b.knock = 1; // knockdown on last pass
                    }
                    e.b.act = 4;
                    self.enemy_anim(&mut e.b, 5);
                }
            }
            4 => {
                // 060b40
                let mut pull_up = false;
                if e.b.hitstop != 0 {
                    pull_up = true;
                } else {
                    e.b.motion();
                    if e.b.vy < 0 {
                        let height = e.b.yi().wrapping_sub(e.b.floor.wrapping_add(e.b.floor2));
                        if height < 64 {
                            pull_up = true;
                        }
                    }
                }
                if pull_up {
                    e.b.ay = 768;
                    e.m.sw(0xA6, 0xFFC0);
                    self.enemy_anim(&mut e.b, 4);
                    e.b.act = 6;
                }
            }
            6 => {
                // 060b80
                e.b.motion();
                if e.b.vy > 1024 {
                    e.b.vy = 1024;
                    e.b.ay = 0;
                }
                if self.enemy_step(&mut e.b) {
                    let height = e.b.yi().wrapping_sub(e.b.floor.wrapping_add(e.b.floor2));
                    if height > 140 {
                        let passes = e.m.b(0xAA).wrapping_sub(1);
                        e.m.sb(0xAA, passes);
                        if passes == 0 {
                            self.enemy_zeke_reset_patrol(e);
                        } else {
                            e.b.act = 2;
                        }
                    }
                }
            }
            _ => self.enemy_zeke_reset_patrol(e),
        }
    }

    /// 0x60BD4: mode 6 dive.
    fn enemy_zeke_mode_dive(&mut self, e: &mut Enemy) {
        match e.b.act {
            0 => {
                // 060be8
                self.enemy_set_speed(&mut e.b, 3);
                e.m.sb(0x81, 3);
                e.m.sb(0xA2, 1);
                e.b.act = 2;
            }
            2 => {
                // 060c02
                if self.enemy_step(&mut e.b)
                    && self.enemy_zeke_test_altitude(e) {
                        self.enemy_zeke_pick_altitude(e);
                        self.enemy_zeke_flap_anim(e);
                    }
                let arrived = self.enemy_zeke_approach(e, 0x1E);
                if arrived {
                    let (mut vx, mut ax) = (-128i16, 0i16);
                    let target = e.m.w(0x76);
                    let target_x = self.body(target).xi();
                    let mut facing = 0u8;
                    if target_x >= e.b.xi() {
                        facing = 1;
                        vx = -vx;
                        ax = -ax;
                    }
                    e.b.facing = facing;
                    e.b.vx = vx;
                    e.b.ax = ax;
                    e.b.vy = -512;
                    e.b.ay = -64;
                    e.b.vz = 0;
                    e.b.az = 0;
                    e.b.act = 4;
                }
                self.enemy_move_dir(&mut e.b);
                e.b.motion_y();
            }
            4 => {
                // 060c74
                if self.enemy_step(&mut e.b) {
                    e.b.act = 6;
                    self.enemy_anim(&mut e.b, 7);
                }
            }
            6 => {
                // 060c8c
                e.b.motion();
                if e.b.vy < 0 {
                    let height = e.b.yi().wrapping_sub(e.b.floor.wrapping_add(e.b.floor2));
                    if height < 85 {
                        let target = e.m.w(0x76);
                        let tb = self.body(target);
                        let valid = tb.live == 1 && tb.mode == 2 && tb.sub == 0 && tb.airborne == 0;
                        if valid {
                            let dx = (e.b.xi().wrapping_sub(tb.xi()).wrapping_add(0x18)) as u16;
                            let dz = (e.b.zi().wrapping_sub(tb.zi()).wrapping_add(0x10)) as u16;
                            if dx < 0x30 && dz < 0x20 {
                                // 060cf2: bra.w $60fe4
                                e.b.set_state(0x0204_0000); // mode 2, sub 4, act 0, step 0
                                return;
                            }
                        }
                        self.enemy_zeke_reset_patrol(e);
                    }
                }
            }
            _ => self.enemy_zeke_reset_patrol(e),
        }
    }

    /// 0x60D26: mode 8 leave.
    fn enemy_zeke_mode_leave(&mut self, e: &mut Enemy) {
        if e.b.act == 0 {
            // 060d36
            self.enemy_anim(&mut e.b, 1);
            let pal = if self.w.stage == 6 { 0x9F } else { 0x9E };
            e.b.palette_override = pal;
            e.b.act = 2;
            // 060d54: falls through into act 2 in the same frame!
        }
        if e.b.act == 2 {
            // 060d58
            let height = e.b.yi().wrapping_sub(e.b.floor.wrapping_add(e.b.floor2));
            if height >= 512 {
                e.b.mode = 6;
                return;
            }
            self.enemy_step(&mut e.b);
            e.b.motion();
        }
    }

    /// 0x60FEE: sub 4 dive grab.
    fn enemy_zeke_sub_grab(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => {
                // 06100c
                let target = e.m.w(0x76);
                let tb = self.body(target);
                let valid = tb.live == 1 && tb.mode == 2 && tb.sub == 0 && tb.airborne == 0;
                if !valid {
                    // 06105e
                    e.b.set_state(0x0200_0000);
                    e.b.act = 0;
                    e.b.step = 0;
                    return;
                }
                e.b.partner = target;
                e.b.kind = 0xC;
                if let Some(crate::game::Slot::Player(pi)) = crate::game::slot_of(target) {
                    let p = &mut self.players[pi];
                    p.b.partner = h;
                    p.b.set_state(0x0206_0000);
                    p.action = 0x40;
                    p.b.kind = 0x0E;
                    p.bc7 = 0;
                    p.b.hp_prev = p.b.hp;
                }
                e.m.sb(0x80, 30);
                e.b.act = 2;
            }
            2 => {
                // 06106a
                if !self.enemy_zeke_held_valid(e) {
                    e.b.set_state(0x0200_0000);
                    e.b.act = 0;
                    e.b.step = 0;
                    return;
                }
                let t = e.m.b(0x80);
                let (new_t, borrow) = t.overflowing_sub(1);
                e.m.sb(0x80, new_t);
                if borrow {
                    e.b.vy = 512;
                    e.b.ay = 8;
                    let mut vx = 896i16;
                    let mut facing = 1u8;
                    if 0xC0i16 < e.b.xi() {
                        vx = -vx;
                        facing = 0;
                    }
                    e.b.vx = vx;
                    e.b.facing = facing;
                    e.b.ax = 0;
                    e.b.vz = 0;
                    e.b.az = 0;
                    e.b.act = 4;
                }
                self.enemy_step(&mut e.b);
            }
            4 => {
                // 0610be
                if !self.enemy_zeke_held_valid(e) {
                    e.b.set_state(0x0200_0000);
                    e.b.act = 0;
                    e.b.step = 0;
                    return;
                }
                self.enemy_step(&mut e.b);
                e.b.motion();
                let height = e.b.yi().wrapping_sub(e.b.floor.wrapping_add(e.b.floor2));
                if height > 100 {
                    e.b.atk = 0x3B;
                    e.b.knock = 0;
                    e.b.hitstop = 0;
                    e.b.serial = self.w.attack_serial;
                    self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
                    // 0x610F8 does not set A2. The census (0x3248A..0x324A2) leaves
                    // its last target there; with no counted target, 0xFD02's list-2
                    // cursor survives. Both damage (0x6436) and sound pan (0x9658)
                    // read that source, not Zeke.
                    let (w84, ax) = if let Some(a2) = self.spawn.g.census_target {
                        (self.player_enemy_w84(a2), self.body(a2).xi())
                    } else {
                        let sc = &self.assets.enemy.stages[self.w.stage as usize].areas[self.cam.s2.area as usize & 3];
                        let a2 = sc.list2_addr.wrapping_add((self.spawn.l2 as u32).wrapping_mul(16));
                        (self.assets.enemy.t.u16(a2, 0x84), self.assets.enemy.t.i16(a2, 8))
                    };
                    self.enemy_hit_held_player_scale(e, h, w84, ax);
                    let pan = self.snd_pan_x(e.b.xi());
                    self.snd_request(0x5F, pan as u32, 0); // 0xA0CC
                    self.enemy_zeke_reset_patrol(e);
                }
            }
            _ => {
                self.enemy_zeke_reset_patrol(e);
            }
        }
    }

    /// 0x6110A: check if held player is still in held state.
    fn enemy_zeke_held_valid(&self, e: &Enemy) -> bool {
        let target = e.m.w(0x76);
        if let Some(crate::game::Slot::Player(pi)) = crate::game::slot_of(target) {
            let p = &self.players[pi];
            p.b.live != 0 && p.b.mode == 2 && p.b.sub == 6 && p.action == 0x40 && p.b.kind == 0x0E
        } else {
            false
        }
    }

    /// 0x60F1E: sub 2 hit reaction.
    fn enemy_zeke_sub_hit(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.step {
            0 => {
                // 060f54
                self.enemy_anim(&mut e.b, 2);
                let mut vx = 768i16;
                let mut ax = 0i16;
                if e.b.facing != 0 {
                    vx = -vx;
                    ax = -ax;
                }
                e.b.vx = vx;
                e.b.ax = ax;
                e.b.vy = 512;
                e.b.ay = -32;
                e.b.vz = 0;
                e.b.az = 0;
                e.m.sw(0xA8, e.b.yi() as u16);
                self.enemy_hud_hit(h, &e.b, 0);
                e.b.step = 2;
                // 060f98: falls through into step 2 in the same frame
                e.b.motion();
                if e.b.vy < 0 {
                    let launch_y = e.m.w(0xA8) as i16;
                    if e.b.yi() < launch_y {
                        e.b.step = 4;
                    }
                }
            }
            2 => {
                // 060f9c
                e.b.motion();
                if e.b.vy < 0 {
                    let launch_y = e.m.w(0xA8) as i16;
                    if e.b.yi() < launch_y {
                        e.b.step = 4;
                    }
                }
            }
            4 => {
                // 060fbc
                if e.b.hp == 0 {
                    e.b.mode = 4;
                    e.b.sub = 0;
                } else {
                    e.b.stun = 10;
                    e.b.sub = 0;
                    e.b.act = 0;
                    e.b.step = 0;
                    e.b.live = 1;
                    self.enemy_zeke_reset_patrol(e);
                }
            }
            _ => {
                self.enemy_zeke_reset_patrol(e);
            }
        }
    }

    /// 0x61140: mode 4 dying.
    fn enemy_zeke_mode_dying(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.sub == 0 {
            // 061164
            self.enemy_anim(&mut e.b, 6);
            let pal = if self.w.stage == 6 { 0x9F } else { 0x9E };
            e.b.palette_override = pal;
            let mut vx = 768i16;
            if e.b.facing != 0 {
                vx = -vx;
            }
            e.b.vx = vx;
            e.b.ay = 16;
            e.b.ax = 0;
            e.b.vz = 0;
            e.b.az = 0;
            e.b.facing ^= 1;
            e.b.sub = 2;
            // 0611ae: falls through into sub 2!
        }
        if e.b.sub == 2 {
            // 0611b2
            let min_y = e.b.floor.wrapping_add(16);
            if e.b.yi() < min_y {
                e.b.set_yi_keep(min_y);
            }
            e.b.motion();
        }
        self.terrain_floor_only(&mut e.b); // 0x12d4e
        self.enemy_despawn_if_far(e, i); // 0x173c
        self.enemy_draw_check(&mut e.b, h); // 0x49ca
    }
}
