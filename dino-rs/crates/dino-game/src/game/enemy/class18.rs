//! Class 18 (0x42590): Vice T. and Walther, including death and scores.
use super::{in_win, Enemy};
use crate::game::{Game, Handle};

impl Game {
    /// 0x42590: class 18 always uses its native sprites, including Walther.
    pub(crate) fn enemy_vice_intro_active(b: &crate::game::body::Body) -> bool {
        b.k20 == 18 * 4
    }

    /// 0x42590 / 0x425A4 / 0x42884 / 0x43892 / 0x438EC.
    pub(super) fn enemy_class18(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                if e.b.sub == 0 { self.enemy_vice_init(e, h); }
                if e.b.var26 >> 8 != 0 {
                    // 0x42694 -> 0x42864 -> 0x42B96: Walther goes straight to the approach, without the draw check of 0x426A6.
                    self.enemy_vice_fight_start(e, h);
                } else {
                    self.enemy_vice_intro(e, h);
                    self.enemy_draw_check(&mut e.b, h); // 0x426A6: jmp 0x49CA
                }
            }
            2 => {
                // 0x42884 tests the whole variant word, unlike the byte tests.
                if e.b.var26 != 0 { self.enemy_kill_all_check(e); }
                for offset in [0xBE, 0xC4] {
                    e.m.sw(offset, e.m.w(offset).saturating_sub(1));
                }
                match e.b.sub {
                    0 => self.enemy_vice_alive(e, h),
                    2 => {
                        // 0x4385C: only Vice runs the boss services and fade.
                        if e.b.var26 >> 8 == 0 {
                            self.boss_hit(e, h);
                        }
                        self.enemy_hit_reaction(e, h);
                    }
                    6 => self.enemy_grabbed(e, h), // 0x428CA -> 0x2B35A
                    _ => {} // 0x43890: sub 4 is an rts in the ROM.
                }
                if e.b.var26 >> 8 != 0 { self.enemy_clamp_to_playfield(e, i); }
            }
            4 => self.enemy_vice_dying(e, h),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x42864..0x42880: only Vice owns the controller and HUD boss bar.
    fn enemy_vice_fight_start(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.var26 >> 8 == 0 {
            // 0x7BEDC (inside boss_spawn_controller) writes the controller's +$AA; the original init does not write it again.
            self.boss_spawn_controller(h);
            self.hud.boss = h;
        }
        e.b.mode = e.b.mode.wrapping_add(2);
        e.b.sub = 0;
        e.b.act = 0;
        e.b.step = 0;
        self.enemy_vice_approach(e);
    }

    /// 0x43892..0x438EA: score once; Vice stays, Walther blinks then frees.
    fn enemy_vice_dying(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.sub == 0 {
            self.enemy_award_kill_score(&e.b);
            e.b.sub = e.b.sub.wrapping_add(2);
            e.m.sb(0x80, 0x78);
            if e.b.var26 >> 8 == 0 { self.enemy_boss_leader(e.b.who); }
        }
        if e.b.var26 >> 8 == 0 {
            self.enemy_draw_check(&mut e.b, h);
        } else {
            let timer = e.m.b(0x80);
            if timer == 0 {
                e.b.mode = e.b.mode.wrapping_add(2);
            } else {
                let timer = timer.wrapping_sub(1);
                e.m.sb(0x80, timer);
                if timer & 1 != 0 { self.enemy_draw_check(&mut e.b, h); }
            }
        }
    }

    /// 0x42884 / 0x428CE: alive timers, poise refill, terrain and obstacle attack.
    fn enemy_vice_alive(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sw(0xAA, e.m.w(0xAA).saturating_sub(1));
        if e.m.b(0x79) != 0 {
            e.m.sb(0x79, e.m.b(0x79).wrapping_sub(1));
            if e.m.b(0x79) == 0 { e.b.stun = 11; }
        }
        if e.b.act == 4 {
            self.enemy_vice_attack(e);
        } else if e.b.act == 0xA {
            self.enemy_vice_summon(e);
        } else if e.b.act == 6 {
            self.enemy_vice_rage(e);
        } else if e.b.act == 8 {
            self.enemy_vice_fall(e);
        } else if e.b.act == 0 && e.b.airborne != 0 {
            e.b.set_state(0x0200_0800); // 0x42B3E -> 0x43738
        } else if e.b.act == 2 || e.m.b(0x7A) == 0 {
            self.enemy_vice_approach(e);
        } else {
            // 0x42B42: decide waits until zero, with same-frame fallthrough.
            e.m.sb(0x7A, e.m.b(0x7A).wrapping_sub(1));
            if e.m.b(0x7A) == 0 { self.enemy_vice_approach(e); }
        }
        if let Some(o) = self.terrain_collide(&mut e.b, h, false) {
            e.m.sw(0x94, o); // 0x12CB4: the obstacle test leaves the obstacle in +$94 (read by 0x4292E)
        }
        e.m.sb(0xC3, e.m.b(0xC3).wrapping_sub(1));
        if e.m.b(0xC3) == 0 {
            e.m.sb(0xC3, 200);
            if e.b.wall & 0x80 != 0 {
                // 0x4292E: prepare the obstacle combo.
                e.b.set_state(0x0200_0402);
                e.m.sb(0xB6, 7); // byte write, deliberately preserves +$B7
                e.m.sw(0xB8, 0x38);
                e.b.facing = (self.body(e.m.w(0x94)).xi().wrapping_sub(e.b.xi()) > 0) as u8;
            }
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x42B96..0x42C44: nearby hero checks, retarget and periodic attack roll.
    fn enemy_vice_approach(&mut self, e: &mut Enemy) {
        if e.b.airborne == 0 {
            // 0x44DEC / 0x44E12 and 0x44E52 / 0x44E6C: the table 0x44E06 lists P3, P2, P1 and d5 counts down from 2, so the scan is P1, P2, P3.
            for behind in [false, true] {
                for i in 0..3 {
                    let p = &self.players[i].b;
                    if p.live == 0 || p.sub != 0 { continue; }
                    let near = if behind {
                        let dx = if e.b.facing == 0 { 60i16 } else { -60i16 };
                        in_win(e.b.xi().wrapping_add(dx).wrapping_sub(p.xi()), 24)
                            && in_win(p.zi().wrapping_sub(e.b.zi()), 5)
                    } else {
                        in_win(p.xi().wrapping_sub(e.b.xi()), 48)
                            && in_win(p.zi().wrapping_sub(e.b.zi()), 8)
                    };
                    if !near { continue; }
                    e.m.sw(0x76, Game::player_handle(i));
                    if behind {
                        // 0x43604: enter and tick the back elbow.
                        e.b.set_state(0x0200_041C);
                        self.enemy_anim(&mut e.b, 10);
                        self.enemy_begin_attack(&mut e.b, 0x35);
                        if self.enemy_step(&mut e.b) { e.b.set_state(0x0200_0000); }
                    } else {
                        e.b.set_state(0x0200_0400); // 0x42F3E
                    }
                    return;
                }
            }
        }
        if e.m.b(0xBD) != 0 {
            self.enemy_vice_rage(e);
            return;
        }
        self.enemy_vice_approach_normal(e);
    }

    /// 0x42BB4..0x42C44: ordinary approach after the rage gate.
    fn enemy_vice_approach_normal(&mut self, e: &mut Enemy) {
        let old = e.m.w(0xB2);
        if self.body(e.m.w(0x76)).live == 0 || old == 0 {
            e.m.sw(0xB2, 600);
            self.enemy_vice_retarget(e);
            let p = self.body(e.m.w(0x76));
            let (x, z) = (p.xi() as u16, p.zi() as u16);
            e.m.sw(0x88, x);
            e.m.sw(0x8A, z);
            e.b.dir = Game::enemy_angle_to(&e.b, x, z); // 0x44974
        } else {
            e.m.sw(0xB2, old.wrapping_sub(1));
        }
        if (e.m.b(0xB1) as i8) < 4 {
            let old = e.m.b(0xB4);
            e.m.sb(0xB4, old.wrapping_sub(1));
            if old == 0 {
                if e.b.airborne != 0 { e.m.sb(0xB4, 0); }
                else {
                    e.m.sb(0xB4, 60);
                    if self.enemy_vice_attack_roll(e) {
                        e.b.act = 4;
                        e.b.step = 0;
                        return;
                    }
                }
            }
        }
        if e.b.step == 0 {
            e.b.set_state(0x0200_0202);
            e.m.sb(0xB5, 0);
            self.enemy_vice_new_cell(e);
            let r = self.rand() & 15;
            e.m.sb(0xB1, self.assets.enemy.t.u8(0x44ADE, r as u32));
            match e.m.b(0xB1) {
                0 => self.enemy_vice_walk_start(e),
                2 => {
                    // 0x44AEE compares the horizontal half of two directions.
                    let p = self.body(e.m.w(0x76));
                    let a = Game::enemy_angle_to(&e.b, p.xi() as u16, p.zi() as u16);
                    let cell = e.m.b(0x7C) as u32;
                    let (x, z) = if e.b.var26 >> 8 == 0 {
                        (self.assets.enemy.t.u16(0x44868, cell * 4), self.assets.enemy.t.u16(0x44868, cell * 4 + 2))
                    } else {
                        // 0x44B0E doubles once, rather than indexing a pair.
                        (self.assets.enemy.t.u16(0x44812, cell * 2).wrapping_add(p.xi() as u16), p.zi() as u16)
                    };
                    let b = Game::enemy_angle_to(&e.b, x, z);
                    e.m.sb(0x90, 0);
                    if (a ^ b) & 16 != 0 {
                        e.m.sb(0xB1, 0);
                        self.enemy_vice_walk_start(e);
                    } else {
                        e.m.sb(0xB5, 1);
                        e.m.sw(0xC4, 600);
                        self.enemy_anim(&mut e.b, 4);
                        self.enemy_set_speed(&mut e.b, 13);
                        e.m.sb(0x7D, if e.b.var26 >> 8 == 0 { 0xFF } else { 0 });
                        e.m.sw(0xAE, 0);
                        e.m.sb(0xB0, 120);
                    }
                }
                4 => {
                    self.enemy_vice_destination(e);
                    self.enemy_vice_jump_start(e);
                    self.enemy_anim(&mut e.b, 2);
                    self.enemy_vice_face(e);
                }
                6 => e.m.sb(0x80, 10), // 0x42E54
                8 => { self.enemy_vice_retreat_start(e); return; }
                10 => { self.enemy_vice_escape_jump_start(e); return; }
                _ => return,
            }
        }
        match e.m.b(0xB1) {
            0 | 2 => self.enemy_vice_ground_approach(e),
            4 => { // 0x42DAA
                if e.b.airborne != 0 {
                    if e.b.param_b == 0 { e.b.motion(); }
                    self.enemy_step(&mut e.b);
                } else {
                    self.enemy_anim(&mut e.b, 3);
                    e.m.sb(0xB1, 6);
                    e.m.sb(0x80, 10);
                    self.enemy_vice_approach_pause(e);
                }
            }
            6 => self.enemy_vice_approach_pause(e),
            8 => self.enemy_vice_retreat(e), // 0x42EFC
            10 => { // 0x42EAE: escape jump uses the same landing path
                if e.b.airborne != 0 {
                    if e.b.param_b == 0 { e.b.motion(); }
                    self.enemy_step(&mut e.b);
                } else {
                    self.enemy_anim(&mut e.b, 3);
                    e.m.sb(0xB1, 6);
                    e.m.sb(0x80, 10);
                    self.enemy_vice_approach_pause(e);
                }
            }
            _ => {}
        }
    }

    /// 0x44B5A: one RNG draw, signed byte compare, every 61 eligible updates.
    fn enemy_vice_attack_roll(&mut self, e: &Enemy) -> bool {
        let threshold = self.assets.enemy.t.u8(0x44B84, ((e.b.var26 >> 8) as u32 * 16) + self.w.level as u32);
        ((self.rand() & 31) as i8) < threshold as i8
    }

    /// 0x44416: the list of the present heroes in +$A2, +$A4, +$A6 (P1, P2, P3 in that order, the rest cleared) and d1 = their number - 1.
    /// With no hero d1 = 0, the first cell keeps what it had, and only an empty first cell gets player 1.
    fn enemy_vice_target_list(&self, e: &mut Enemy) -> u16 {
        let mut n = 0u16;
        let mut cell = 0xA2usize;
        for i in 0..3 {
            if self.players[i].b.live != 0 {
                e.m.sw(cell, Game::player_handle(i));
                cell += 2;
                n += 1;
            }
        }
        let mut d1 = n as i16 - 1;
        if d1 < 0 {
            d1 = 0;
            cell += 2; // addq.w #2,a1: the first cell is not cleared
        }
        while cell <= 0xA6 {
            e.m.sw(cell, 0); // clr.w (a1)+ while a1 <= a2
            cell += 2;
        }
        if e.m.w(0xA2) == 0 {
            e.m.sw(0xA2, Game::player_handle(0));
        }
        d1 as u16
    }

    /// 0x44316: rotate the packed target list using the ROM weights. When the present target is a fallen hero (+$C8 = 0x23) it goes to 0x443A6
    /// instead (M7-06: ep4_3p 6562, a hero dies while the Walther fights him): the next hero of the list, by a random choice only with three.
    fn enemy_vice_retarget(&mut self, e: &mut Enemy) {
        let target = e.m.w(0x76);
        if let Some(crate::game::Slot::Player(i)) = crate::game::slot_of(target) {
            if self.players[i].action == 0x23 {
                // 0x443A6
                let d1 = self.enemy_vice_target_list(e);
                match d1 {
                    0 => {
                        e.m.sw(0x76, e.m.w(0xA2));
                        e.m.sb(0xA0, 0);
                    }
                    1 => {
                        let d0 = e.m.b(0xA0).wrapping_add(1) & 1; // 0x443BA
                        e.m.sw(0x76, e.m.w(0xA2 + 2 * d0 as usize));
                        e.m.sb(0xA0, d0);
                    }
                    _ => {
                        let r = self.rand() & 15; // 0x443E0
                        let mut d2 = e.m.b(0xA0).wrapping_add(if r & 8 != 0 { 2 } else { 1 });
                        if (d2 as i8) > 2 {
                            d2 = d2.wrapping_sub(3);
                        }
                        e.m.sb(0xA0, d2);
                        e.m.sw(0x76, e.m.w(0xA2 + 2 * d2 as usize));
                    }
                }
                return;
            }
        }
        let r = self.rand() & 15;
        let last = self.enemy_vice_target_list(e);
        let row = self.assets.enemy.t.u16(0x44370, last as u32 * 2);
        let mut n = e.m.b(0xA0).wrapping_add(self.assets.enemy.t.u8(0x44370, row as u32 + r as u32)) as u16;
        if n as i16 > last as i16 { n = n.wrapping_sub(last + 1); }
        e.m.sb(0xA0, n as u8);
        let mut cell = n * 2;
        loop {
            if cell > 4 { cell = 0; }
            let h = e.m.w(0xA2 + cell as usize);
            if h != 0 { e.m.sw(0x76, h); break; }
            cell += 2;
        }
    }

    /// 0x44478: Walther's initial cell takes two random draws.
    fn enemy_vice_initial_cell(&mut self, e: &mut Enemy) {
        if e.b.var26 >> 8 == 0 { e.m.sb(0x7C, 4); return; }
        let mut side = if (e.b.xi() as u16) < self.body(e.m.w(0x76)).xi() as u16 { 0u8 } else { 8 };
        if self.rand() & 15 >= 11 { side = side.wrapping_add(8) & 15; }
        let r = self.rand() & 31;
        e.m.sb(0x7C, side.wrapping_add(self.assets.enemy.t.u8(0x444C0, r as u32)));
        self.enemy_vice_cell_screen(e);
    }

    /// 0x444E8..0x44570: retain the unused middle RNG draw for Walther.
    fn enemy_vice_new_cell(&mut self, e: &mut Enemy) {
        let cell = e.m.b(0x7C);
        if e.b.var26 >> 8 != 0 {
            let mut side = if (e.b.xi() as u16) < self.body(e.m.w(0x76)).xi() as u16 { 0u8 } else { 8 };
            if self.rand() & 15 >= 11 { side = side.wrapping_add(8) & 15; }
            self.rand(); // 0x44518: masked but overwritten before the table read
            let r = self.rand() & 31;
            let delta = self.assets.enemy.t.u8(0x44572, (cell as u32 & 7) * 32 + r as u32);
            e.m.sb(0x7C, side.wrapping_add(delta) & 15);
            self.enemy_vice_cell_screen(e);
        } else {
            let r = self.rand() & 31;
            let delta = self.assets.enemy.t.u8(0x44672, cell as u32 * 32 + r as u32);
            e.m.sb(0x7C, cell.wrapping_add(delta));
        }
    }

    /// 0x44792: the original indexes words, even though the cell records are pairs.
    fn enemy_vice_cell_screen(&self, e: &mut Enemy) {
        let x = self.assets.enemy.t.u16(0x44812, e.m.b(0x7C) as u32 * 2)
            .wrapping_add(self.body(e.m.w(0x76)).xi() as u16);
        let delta = self.cam.s2.cam_x.wrapping_add(0xC0).wrapping_sub(x).wrapping_add(0xA0);
        if delta >= 0x140 { e.m.sb(0x7C, e.m.b(0x7C).wrapping_add(8) & 15); }
    }

    /// 0x42C50 / 0x44F2E: walking animation and difficulty-dependent speed.
    fn enemy_vice_walk_start(&self, e: &mut Enemy) {
        self.enemy_anim(&mut e.b, 1);
        self.enemy_vice_walk_speed(e);
        e.m.sb(0x7D, if e.b.var26 >> 8 == 0 { 0xFF } else { 0 });
        e.m.sw(0xAE, 0);
        e.m.sb(0xB0, 120);
    }

    /// 0x447CE / 0x44852 / 0x44994 / 0x449C6 / 0x44CCC / 0x44CE4.
    fn enemy_vice_destination(&self, e: &mut Enemy) {
        let cell = e.m.b(0x7C) as u32 * 4;
        let (x, z) = if e.b.var26 >> 8 == 0 {
            (self.assets.enemy.t.u16(0x44868, cell), self.assets.enemy.t.u16(0x44868, cell + 2))
        } else {
            let p = self.body(e.m.w(0x76));
            (self.assets.enemy.t.u16(0x44812, cell).wrapping_add(p.xi() as u16),
                self.assets.enemy.t.u16(0x44812, cell + 2).wrapping_add(p.zi() as u16))
        };
        e.m.sw(0x88, x);
        e.m.sw(0x8A, z);
        self.enemy_vice_turn(e, e.m.b(0xB5) != 0, false);
    }

    /// 0x44994 / 0x449C6 / 0x449F8: bounded turn toward +$88 / +$8A.
    fn enemy_vice_turn(&self, e: &mut Enemy, fast: bool, away: bool) {
        let mut dir = Game::enemy_angle_to(&e.b, e.m.w(0x88), e.m.w(0x8A));
        if away { dir ^= 16; }
        let turn = if away { 3u8 } else if fast { 2u8 } else { 4u8 };
        let d = e.b.dir.wrapping_sub(dir).wrapping_add(turn) & 31;
        if d > turn * 2 {
            dir = e.b.dir.wrapping_add(if d >= 16 + turn { turn } else { turn.wrapping_neg() }) & 31;
        }
        e.b.dir = dir;
        if e.m.b(0xB5) == 0 || away { self.enemy_vice_face(e); }
        else { e.b.facing = ((dir >> 4).wrapping_add(1)) & 1; }
    }

    /// 0x44CCC: unsigned X comparison for facing the target.
    fn enemy_vice_face(&self, e: &mut Enemy) {
        e.b.facing = ((e.b.xi() as u16) < (self.body(e.m.w(0x76)).xi() as u16)) as u8;
    }

    /// 0x42C6A / 0x42CF2: refresh destination, detect arrival, then move and tick.
    fn enemy_vice_ground_approach(&mut self, e: &mut Enemy) {
        let charge = e.m.b(0xB1) == 2;
        if charge && e.m.b(0x90) != 0 {
            self.enemy_vice_charge_brake(e);
            return;
        }
        if !charge && e.b.airborne != 0 {
            e.b.set_state(0x0200_0800); // 0x43738
            return;
        }
        let old = e.m.w(0xAE);
        e.m.sw(0xAE, old.wrapping_sub(1));
        if old == 0 {
            e.m.sw(0xAE, if charge { 1 } else { 2 });
            self.enemy_vice_destination(e);
        }
        let mut arrived = Game::enemy_stuck_check(e);
        if !arrived {
            let t = e.m.b(0xB0);
            e.m.sb(0xB0, t.wrapping_sub(1));
            arrived = t == 0 || (in_win(e.b.xi().wrapping_sub(e.m.w(0x88) as i16), 24)
                && in_win(e.b.zi().wrapping_sub(e.m.w(0x8A) as i16), 8));
        }
        if arrived {
            if charge {
                e.m.sb(0x90, 2);
                e.b.ax = e.b.vx.wrapping_neg() >> 5; // 0x42D62
                e.b.az = e.b.vz.wrapping_neg() >> 5;
                e.b.vy = 0;
                e.b.ay = 0;
                self.enemy_anim(&mut e.b, 3);
                self.enemy_vice_charge_brake(e);
            } else {
                e.m.sb(0xB0, 120);
                self.enemy_vice_new_cell(e);
                e.b.act = 2;
                e.b.step = 0;
            }
        } else {
            self.enemy_move_dir(&mut e.b);
            self.enemy_step(&mut e.b);
        }
    }

    /// 0x42D3C: brake with the original unsigned speed window.
    fn enemy_vice_charge_brake(&mut self, e: &mut Enemy) {
        if (e.b.vx.wrapping_add(48) as u16) < 96 {
            e.m.sb(0x90, 0);
            e.m.sb(0xB5, 0);
            e.m.sb(0xB1, 6);
            e.m.sb(0x80, 10);
            self.enemy_vice_approach_pause(e);
        } else {
            e.b.motion();
            self.enemy_step(&mut e.b);
        }
    }

    /// 0x42DCE / 0x43390: jump toward the cell in 56 ticks, using quotient and remainder.
    fn enemy_vice_jump_start(&self, e: &mut Enemy) {
        self.enemy_vice_jump_to(e, e.m.w(0x88), e.m.w(0x8A), false);
    }

    /// 0x42E60: post-jump / charge pause, borrow then pick another cell.
    fn enemy_vice_approach_pause(&mut self, e: &mut Enemy) {
        let t = e.m.b(0x80);
        e.m.sb(0x80, t.wrapping_sub(1));
        if t == 0 {
            e.m.sb(0xB0, 120);
            self.enemy_vice_new_cell(e);
            e.b.act = 2;
            e.b.step = 0;
        }
    }

    /// 0x42F48 / 0x42F8E: hit stop, then the attack-step dispatch at 0x42F6C.
    /// 0x42922 has only six alive r6 entries (0..0xA). The extended values
    /// 0xC..0x20 belong to this r7 table, not to the following helper code.
    fn enemy_vice_attack(&mut self, e: &mut Enemy) {
        if e.b.hitstop != 0 {
            e.b.hitstop = e.b.hitstop.wrapping_sub(1);
            if e.b.hitstop != 0 { return; }
            e.b.timer = 1;
        }
        match e.b.step {
            0 => {
                if e.b.wall & 0x80 != 0 { e.b.set_state(0x0200_0000); return; }
                e.b.step = 2;
                let r = self.rand() & 31; // 0x44C5A, exactly one draw even when charged
                let combo = if e.m.b(0xB5) != 0 { 5 + (r >> 4) }
                    else { self.assets.enemy.t.u8(0x44C8C, r as u32 + if e.b.var26 >> 8 != 0 { 0x20 } else { 0 }) as u16 };
                e.m.sw(0xB6, combo);
                e.m.sw(0xB8, combo << 3);
            }
            2 => self.enemy_vice_combo_next(e),
            4 => self.enemy_vice_combo_approach(e),
            6 => self.enemy_vice_dash(e),
            8 | 0xA | 0x10 => {
                if self.enemy_step(&mut e.b) { self.enemy_vice_combo_pause(e); }
            }
            0xE | 0x12 => self.enemy_vice_shoot_frame(e),
            0xC | 0x1E => self.enemy_vice_combo_jump(e),
            0x14 => self.enemy_vice_combo_run(e),
            0x16 | 0x18 | 0x1A => self.enemy_vice_combo_brake(e),
            0x1C => {
                if self.enemy_step(&mut e.b) { e.b.set_state(0x0200_0000); }
            }
            0x20 => {
                let t = e.m.b(0x80);
                e.m.sb(0x80, t.wrapping_sub(1));
                if t == 0 { self.enemy_vice_combo_next(e); }
            }
            _ => {}
        }
    }

    /// 0x42FAE / 0x4306C: advance one script byte and enter that action in this frame.
    fn enemy_vice_combo_next(&mut self, e: &mut Enemy) {
        let target = e.m.w(0x76);
        if self.body(target).live == 0 || matches!(self.enemy_vice_hero_action(target), 0x22 | 0x23) {
            e.b.set_state(0x0200_0000);
            return;
        }
        e.m.sb(0xA9, 0);
        let cursor = e.m.w(0xB8);
        if (cursor as i16) < 0 { e.b.set_state(0x0200_0400); return; }
        let action = self.assets.enemy.t.u8(0x43004, cursor as u32);
        if (action as i8) < 0 { e.b.set_state(0x0200_0400); return; }
        e.m.sw(0xB8, cursor.wrapping_add(1));
        match action {
            0 => { // 0x4308A
                e.b.step = 4;
                self.enemy_anim(&mut e.b, if e.m.b(0xBD) == 0 { 1 } else { 6 });
                self.enemy_vice_walk_speed(e);
                self.enemy_vice_face(e);
                self.enemy_vice_combo_destination(e);
                e.m.sb(0xB0, 120);
                self.enemy_vice_combo_approach(e);
            }
            2 => { // 0x430EC
                e.b.step = 6;
                self.enemy_anim(&mut e.b, 15);
                self.enemy_set_speed(&mut e.b, 22);
                e.m.sb(0x80, 30);
                e.m.sb(0xBA, 0);
                e.b.dir = if (e.b.xi() as u16) < (self.body(target).xi() as u16) { 24 } else { 8 };
                e.m.sb(0xB0, 120);
                self.enemy_vice_face(e);
                self.enemy_vice_dash(e);
            }
            4 | 6 => { // 0x43160 / 0x431AA
                e.b.step = if action == 4 { 8 } else { 10 };
                self.enemy_vice_face(e);
                self.enemy_anim(&mut e.b, if action == 4 { 7 } else { 9 });
                self.enemy_begin_attack(&mut e.b, if action == 4 { 0x12 } else { 0x13 });
                if (self.assets.enemy.t.u8(0x43004, e.m.w(0xB8) as u32) as i8) < 0 { e.b.knock = 1; }
                if self.enemy_step(&mut e.b) { self.enemy_vice_combo_pause(e); }
            }
            8 | 24 => { // 0x43208 / 0x4364C
                self.enemy_vice_combo_jump_start(e, action == 24);
            }
            10 | 14 => { // 0x433BE / 0x43476
                e.b.step = if action == 10 { 0xE } else { 0x12 };
                if action == 10 && !in_win(self.body(target).zi().wrapping_sub(e.b.zi()), 6) {
                    e.m.sw(0xB8, 0xFFFF);
                    self.enemy_vice_combo_pause(e);
                } else {
                    if action == 14 { self.enemy_vice_face(e); }
                    self.enemy_anim(&mut e.b, if action == 10 { 14 } else { 16 });
                    if action == 10 { self.enemy_vice_face(e); }
                    self.enemy_vice_shoot_frame(e);
                }
            }
            12 => { // 0x4344C, carry is the unsigned camera window
                let d = (self.cam.s2.cam_x as i16).wrapping_add(0xC0).wrapping_sub(e.b.xi());
                if !in_win(d, 0xE0) {
                    e.m.sw(0xB8, 0xFFFF);
                    self.enemy_vice_combo_next(e);
                } else {
                    e.b.step = 0x10;
                    self.enemy_anim(&mut e.b, 12);
                    if self.enemy_step(&mut e.b) { self.enemy_vice_combo_pause(e); }
                }
            }
            16 => { // 0x434AA / 0x44DBC
                let dx = self.body(target).xi().wrapping_sub(e.b.xi());
                if in_win(dx, 0x70) {
                    let combo = e.m.w(0xB6).wrapping_sub(5);
                    e.m.sw(0xB6, combo);
                    e.m.sw(0xB8, combo << 3);
                    self.enemy_vice_combo_next(e);
                } else {
                    e.b.step = 0x14;
                    self.enemy_vice_face(e);
                    e.m.sb(0xB5, 1);
                    e.m.sb(0xBC, 20);
                    self.enemy_anim(&mut e.b, 4);
                    self.enemy_set_speed(&mut e.b, 25);
                    self.enemy_vice_target_destination(e);
                    self.enemy_vice_track_target(e);
                    self.enemy_move_dir(&mut e.b);
                    self.enemy_vice_combo_run(e);
                }
            }
            18 | 20 | 22 => { // 0x43564 / 0x435B2 (RTS) / 0x435B4
                if action == 20 { return; }
                e.b.step = if action == 18 { 0x18 } else { 0x1A };
                self.enemy_anim(&mut e.b, if action == 18 { 17 } else { 18 });
                self.enemy_begin_attack(&mut e.b, if action == 18 { 0x15 } else { 0x16 });
                self.enemy_vice_combo_brake(e);
            }
            26 => self.enemy_vice_combo_pause(e), // 0x4367E
            _ => {}
        }
    }

    /// 0x42FBA / 0x42FC4: player +$C8 is separate from the common body.
    fn enemy_vice_hero_action(&self, h: Handle) -> u8 {
        if let Some(crate::game::Slot::Player(i)) = crate::game::slot_of(h) { self.players[i].action } else { 0 }
    }

    /// 0x44F2E: level-dependent speed id.
    fn enemy_vice_walk_speed(&self, e: &mut Enemy) {
        let speed = self.assets.enemy.t.u16(0x44F4E, (e.b.var26 >> 8) as u32 * 64 + (self.w.level as u32 & 31) * 2);
        self.enemy_set_speed(&mut e.b, speed);
    }

    /// 0x44BC4: combo-specific distance from the target, then turn and face.
    fn enemy_vice_combo_destination(&self, e: &mut Enemy) {
        let p = self.body(e.m.w(0x76));
        let mut d = self.assets.enemy.t.u16(0x44C06, e.m.w(0xB6) as u32 * 2);
        if (e.b.xi() as u16) < (p.xi() as u16) { d = d.wrapping_neg(); }
        e.m.sw(0x88, (p.xi() as u16).wrapping_add(d));
        e.m.sw(0x8A, p.zi() as u16);
        self.enemy_vice_turn(e, e.m.b(0xB5) != 0, false);
        self.enemy_vice_face(e);
    }

    /// 0x430B2: approach, then run the next action immediately on arrival.
    fn enemy_vice_combo_approach(&mut self, e: &mut Enemy) {
        if e.b.airborne != 0 { e.b.set_state(0x0200_0800); return; }
        if self.enemy_vice_back_elbow(e) { return; }
        if Game::enemy_stuck_check(e) { e.b.set_state(0x0200_0000); return; }
        let t = e.m.b(0xB0);
        e.m.sb(0xB0, t.wrapping_sub(1));
        if t == 0 { e.b.set_state(0x0200_0000); return; }
        if in_win(e.b.xi().wrapping_sub(e.m.w(0x88) as i16), 4)
            && in_win(e.b.zi().wrapping_sub(e.m.w(0x8A) as i16), 7) {
            self.enemy_vice_combo_next(e);
        } else {
            self.enemy_vice_combo_destination(e);
            self.enemy_move_dir(&mut e.b);
            self.enemy_step(&mut e.b);
        }
    }

    /// 0x44E52 / 0x43604 / 0x4362C: scan P1 to P3 (the table 0x44E06 lists P3, P2, P1; d5 counts down from 2), enter and tick the back elbow.
    fn enemy_vice_back_elbow(&mut self, e: &mut Enemy) -> bool {
        for i in 0..3 {
            let p = &self.players[i].b;
            if p.live == 0 || p.sub != 0 { continue; }
            let dx = if e.b.facing == 0 { 60i16 } else { -60i16 };
            if in_win(e.b.xi().wrapping_add(dx).wrapping_sub(p.xi()), 24)
                && in_win(p.zi().wrapping_sub(e.b.zi()), 5) {
                e.m.sw(0x76, Game::player_handle(i));
                e.b.set_state(0x0200_041C);
                self.enemy_anim(&mut e.b, 10);
                self.enemy_begin_attack(&mut e.b, 0x35);
                if self.enemy_step(&mut e.b) { e.b.set_state(0x0200_0000); }
                return true;
            }
        }
        false
    }

    /// 0x4312C / 0x44CF6: dash away, latch a stop at the camera or target window.
    fn enemy_vice_dash(&mut self, e: &mut Enemy) {
        let t = e.m.b(0x80);
        e.m.sb(0x80, t.wrapping_sub(1));
        if t == 0 { self.enemy_vice_combo_next(e); return; }
        if e.m.b(0xBA) != 0 { return; }
        let center = (self.cam.s2.cam_x as i16).wrapping_add(0xC0).wrapping_sub(e.b.xi());
        let p = self.body(e.m.w(0x76));
        let offset = if (p.xi() as u16) < (e.b.xi() as u16) { 0x60i16 } else { -0x60 };
        if !in_win(center, 0xA0) || !in_win(p.xi().wrapping_add(offset).wrapping_sub(e.b.xi()), 0x60) {
            e.m.sb(0xBA, 1);
            return;
        }
        if Game::enemy_stuck_check(e) { e.b.set_state(0x0200_0000); return; }
        let t = e.m.b(0xB0);
        e.m.sb(0xB0, t.wrapping_sub(1));
        if t == 0 { e.b.set_state(0x0200_0000); return; }
        self.enemy_move_dir(&mut e.b);
        self.enemy_step(&mut e.b);
    }

    /// 0x4324C / 0x432D6 / 0x43390: divide absolute word differences, retain remainder.
    fn enemy_vice_jump_to(&self, e: &mut Enemy, x: u16, z: u16, short: bool) {
        e.b.airborne = 1;
        e.b.vy = if short { 0x580 } else { 0x700 };
        e.b.ay = -0x40;
        let n = if short { 44u16 } else { 56 };
        let velocity = |position: u16, target: u16, depth: bool| {
            let d = position.wrapping_sub(target) as i16;
            let magnitude = d.wrapping_abs() as u16;
            let (mut q, mut r) = (magnitude / n, magnitude % n);
            if position >= target {
                q = q.wrapping_neg();
                if depth && q == 0 && (r as i16) < 0 { q = 0xFFFF; }
                r = r.wrapping_neg();
            }
            if depth { r <<= 2; }
            let v = ((r & 0xFF) | (((r >> 8).wrapping_add(q) & 0xFF) << 8)) as i16;
            if !depth && q.wrapping_add(0x500) >= 0xA00 { 0x500 } else { v }
        };
        e.b.vx = velocity(e.b.xi() as u16, x, false);
        e.b.vz = velocity(e.b.zi() as u16, z, true);
        e.b.ax = 0;
        e.b.az = 0;
    }

    /// 0x43208 / 0x4364C: jump attack setup, one roll only for the normal jump.
    fn enemy_vice_combo_jump_start(&mut self, e: &mut Enemy, charge: bool) {
        e.b.step = if charge { 0x1E } else { 0xC };
        e.m.sb(0xA9, 2);
        self.enemy_anim(&mut e.b, if charge { 24 } else { 11 });
        self.enemy_begin_attack(&mut e.b, 0x14);
        self.enemy_vice_face(e);
        let short = !charge && (self.rand() & 15) < 3;
        let p = self.body(e.m.w(0x76));
        let (x, z) = (p.xi() as u16, p.zi() as u16);
        self.enemy_vice_jump_to(e, x, z, short);
    }

    /// 0x43360 / 0x43378 / 0x43384: airborne motion, landing animation, pause.
    fn enemy_vice_combo_jump(&mut self, e: &mut Enemy) {
        if e.m.b(0xA9) == 0 {
            self.enemy_vice_combo_jump_start(e, e.b.step == 0x1E);
            return;
        }
        if e.m.b(0xA9) == 2 {
            if e.b.airborne != 0 {
                if e.b.param_b == 0 { e.b.motion(); }
                self.enemy_step(&mut e.b);
                return;
            }
            e.m.sb(0xA9, 4);
            self.enemy_anim(&mut e.b, 3);
        }
        if self.enemy_step(&mut e.b) { self.enemy_vice_combo_pause(e); }
    }

    /// 0x44974: copy the target coordinates and set the direction immediately.
    fn enemy_vice_target_destination(&self, e: &mut Enemy) {
        let p = self.body(e.m.w(0x76));
        e.m.sw(0x88, p.xi() as u16);
        e.m.sw(0x8A, p.zi() as u16);
        e.b.dir = Game::enemy_angle_to(&e.b, e.m.w(0x88), e.m.w(0x8A));
    }

    /// 0x44C20: track the hero, retaining the original gradual turn.
    fn enemy_vice_track_target(&self, e: &mut Enemy) {
        let p = self.body(e.m.w(0x76));
        e.m.sw(0x88, p.xi() as u16);
        e.m.sw(0x8A, p.zi() as u16);
        self.enemy_vice_turn(e, e.m.b(0xBD) != 0, false);
    }

    /// 0x434E6 / 0x44D60 / 0x44D8E / 0x4351A: charge, brake on entry to the hit window.
    fn enemy_vice_combo_run(&mut self, e: &mut Enemy) {
        e.m.sb(0xBC, e.m.b(0xBC).saturating_sub(1));
        let row = e.m.w(0xB6).wrapping_sub(5) as u32 * 2;
        let dx = self.body(e.m.w(0x76)).xi().wrapping_sub(e.b.xi());
        let brake = self.assets.enemy.t.u16(0x44D8A, row);
        if in_win(dx, brake) {
            e.b.ax = e.b.vx.wrapping_neg() >> 5;
            e.b.az = e.b.vz.wrapping_neg() >> 5;
            let arrival = self.assets.enemy.t.u16(0x44DB8, row);
            if in_win(dx, arrival) && e.m.b(0xBC) == 0 { self.enemy_vice_combo_pause(e); }
            else {
                e.b.step = 0x16;
                self.enemy_anim(&mut e.b, 3);
                self.enemy_vice_combo_brake(e);
            }
        } else {
            self.enemy_vice_track_target(e);
            self.enemy_move_dir(&mut e.b);
            self.enemy_step(&mut e.b);
        }
    }

    /// 0x43542 / 0x4358A / 0x435DA: stop under the unsigned velocity window.
    fn enemy_vice_combo_brake(&mut self, e: &mut Enemy) {
        if (e.b.vx.wrapping_add(48) as u16) < 96 {
            e.m.sb(0xB5, 0);
            e.m.sw(0xB8, 0xFFFF);
            self.enemy_vice_combo_pause(e);
        } else {
            e.b.motion();
            if e.b.step != 0x16 { self.enemy_step(&mut e.b); }
        }
    }

    /// 0x4367E / 0x436AA: one RNG draw for the level-scaled pause, then tick it now.
    fn enemy_vice_combo_pause(&mut self, e: &mut Enemy) {
        e.b.step = 0x20;
        let r = self.rand() & 31;
        let slot = self.assets.enemy.t.u8(0x436B4, r as u32) as u32;
        e.m.sb(0x80, self.assets.enemy.t.u8(0x436D4, (self.w.level as u32 & 0x18) + slot));
        self.enemy_anim(&mut e.b, 0);
        let t = e.m.b(0x80);
        e.m.sb(0x80, t.wrapping_sub(1));
        if t == 0 { self.enemy_vice_combo_next(e); }
    }


    /// 0x433E8 / 0x43490: consume the shot flag before ticking the animation.
    fn enemy_vice_shoot_frame(&mut self, e: &mut Enemy) {
        if e.b.param_b != 0 {
            e.b.param_b = 0;
            self.enemy_vice_gun_sound(e);
            self.enemy_spawn_bullet(e, 2, 0x33, 0x46);
            self.enemy_vice_muzzle(e, false);
        }
        if self.enemy_step(&mut e.b) { self.enemy_vice_combo_pause(e); }
    }

    /// 0x9CD8: enemy revolver sound with facing channel and horizontal pan.
    fn enemy_vice_gun_sound(&mut self, e: &Enemy) {
        let pan = self.snd_pan_x(e.b.xi()) as u32;
        let channel = if e.b.facing == 0 { 3 } else { 4 };
        self.snd_request(0xE7, (channel << 16) | 0x2000 | pan, 0x1500_0000);
    }

    /// 0x43420 / 0x4382A, including the facing form of 0x92422.
    fn enemy_vice_muzzle(&mut self, e: &Enemy, air: bool) {
        if air { self.enemy_vice_gun_sound(e); }
        let dx: i16 = if air { if e.b.facing == 0 { 0x29 } else { -0x29 } }
            else if e.b.facing == 0 { -0x2B } else { 0x2B };
        let x = e.b.xi().wrapping_add(dx);
        let y = e.b.yi().wrapping_add(if air { 0x6D } else { 0x45 });
        // The enemy updater takes the owner out of its slot. Pass its facing
        // directly in this 0x92422 entry instead of looking up the empty slot.
        if let Some(i) = self.effects.alloc() {
            let b = &mut self.effects.slots[i].b;
            b.live = 1;
            b.k20 = crate::game::effect::FX_SPARK;
            b.who = 0;
            b.facing = e.b.facing;
            b.set_xi_keep(x);
            b.set_yi_keep(y);
            b.set_zi_keep(e.b.zi());
            b.var26 = if air { 0x17 } else { 0xF };
        }
        self.fx_spawn_spark_jitter(x, y.wrapping_add(8), e.b.zi(), 6);
    }

    /// 0x437AE..0x43826: retreat jump, land, then fire into the air.
    fn enemy_vice_summon(&mut self, e: &mut Enemy) {
        match e.b.step {
            0 => {
                e.b.step = 2;
                e.m.sb(0x7C, 3);
                self.enemy_vice_destination(e);
                self.enemy_vice_jump_start(e);
                self.enemy_anim(&mut e.b, 0x17);
                self.enemy_vice_face(e);
                return;
            }
            2 if e.b.airborne != 0 => {
                if e.b.param_b == 0 { e.b.motion(); }
                self.enemy_step(&mut e.b);
                return;
            }
            2 => {
                e.b.step = 4;
                e.b.facing = 0;
                self.enemy_anim(&mut e.b, 13);
                self.spawn.g.b502 = 1;
            }
            _ => {}
        }
        if e.b.param_b != 0 {
            e.b.param_b = 0;
            self.enemy_vice_muzzle(e, true);
        }
        if self.enemy_step(&mut e.b) { e.b.set_state(0x0200_0000); }
    }

    /// 0x42B52 / 0x42B88: two charges at one half and one quarter of +$6A.
    fn enemy_vice_rage_check(&mut self, e: &mut Enemy) {
        if e.m.b(0xBD) != 0 {
            if e.m.w(0xAA) == 0 { e.m.sb(0xBD, 0); }
        } else {
            let charges = e.m.b(0xA8);
            let quarter = e.b.hp_bar >> 2;
            if charges != 0 && ((e.b.hp as i16) <= quarter as i16
                || (charges == 2 && (e.b.hp as i16) <= quarter.wrapping_mul(2) as i16)) {
                e.m.sb(0xA8, charges.wrapping_sub(1));
                e.m.sb(0xBD, 2);
                e.m.sw(0xAA, 0x708);
            }
        }
    }

    /// 0x436F4: while charged, bypass the approach attack roll on the ground.
    fn enemy_vice_rage(&mut self, e: &mut Enemy) {
        let target = e.m.w(0x76);
        if self.body(target).live == 0 || matches!(self.enemy_vice_hero_action(target), 0x22 | 0x23) {
            // 0x42BB4 uses the ordinary approach without rechecking rage.
            self.enemy_vice_approach_normal(e);
            return;
        }
        let old = e.m.w(0xB2);
        e.m.sw(0xB2, old.wrapping_sub(1));
        if old == 0 {
            e.m.sw(0xB2, 600);
            self.enemy_vice_retarget(e);
            self.enemy_vice_new_cell(e);
        }
        if e.b.airborne != 0 { self.enemy_vice_approach_normal(e); }
        else { e.b.act = 4; e.b.step = 0; }
    }

    /// 0x43742: the fall state reached when a ground action loses its floor.
    fn enemy_vice_fall(&mut self, e: &mut Enemy) {
        match e.b.step {
            0 => {
                e.b.step = 2;
                self.enemy_anim(&mut e.b, 22);
                e.b.vx = 0; e.b.ax = 0; e.b.vy = 0; e.b.ay = -0x20; e.b.vz = 0; e.b.az = 0;
            }
            2 if e.b.airborne != 0 => { e.b.motion(); self.enemy_step(&mut e.b); }
            2 => { e.b.step = 4; self.enemy_anim(&mut e.b, 3);
                if self.enemy_step(&mut e.b) { self.enemy_vice_recover(e); }
            }
            _ => { if self.enemy_step(&mut e.b) { self.enemy_vice_recover(e); } }
        }
    }

    /// 0x42AE4..0x42B14: restore the AI after a generic reaction or fall.
    pub(super) fn enemy_vice_recover(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0000);
        self.enemy_anim(&mut e.b, 0);
        if e.m.b(0x7A) == 0 { e.m.sb(0x7A, 2); }
        e.m.sb(0x7D, if e.b.var26 >> 8 == 0 { 0xFF } else { 0 });
        e.m.sw(0xB2, 600);
        e.m.sb(0xB4, 60);
        e.m.sb(0xB1, 0);
        // 0x44EBA leaves d0 from the animation loader unless it divides the
        // initial HP by the summon count. 0x4295A deliberately uses that d0.
        let frame = self.assets.sprites.frame(e.b.frame);
        let mut d0 = if (frame.w0 as i16) < 0 { frame.w0 & 0x7FFF }
            else if (frame.w1 as i16) >= 0 { frame.w1 } else { frame.w0 };
        if e.b.reaction == 4 {
            let count = e.m.b(0xC2);
            if e.b.var26 >> 8 == 0 && (count as i8) < 5 {
                d0 = if count == 0 { 0 } else { e.m.w(0xC0) / count as u16 };
                if (d0 as i16) >= (e.b.hp as i16) {
                    e.m.sb(0xC2, count.wrapping_add(1));
                    e.b.act = 0xA; // 0x437A8 falls into the summon entry
                    self.enemy_vice_summon(e);
                    return;
                }
            }
            let initial = e.m.w(0xC0);
            let row = if (d0 as i16) <= ((initial >> 2) as i16) { 0 }
                else if (d0 as i16) <= ((initial >> 1) as i16) { 2 } else { 4 };
            // 0x42980..0x42994: the row is 3 * the type byte +$26 (0 Vice, 2 Walther) plus the HP step (M7-06: Walther's chances are 6, 14, 24)
            let chance = self.assets.enemy.t.u16(0x429AE, 3 * (e.b.var26 >> 8) as u32 + row);
            if ((self.rand() & 31) as i16) < (chance as i16) {
                self.enemy_vice_counterattack(e);
                return;
            }
        }
        self.enemy_vice_rage_check(e);
    }

    /// 0x44908: signed X differences, deliberately not absolute distances.
    fn enemy_vice_leftmost_target(&self, e: &mut Enemy) {
        let mut d = [0x7FFFi16; 3];
        for (i, p) in self.players.iter().enumerate() {
            if p.b.live != 0 { d[i] = p.b.xi().wrapping_sub(e.b.xi()); }
        }
        let i = if d[0] < d[1] { if d[0] < d[2] { 0 } else { 2 } }
            else if d[1] < d[2] { 1 } else { 2 };
        e.m.sw(0x76, Game::player_handle(i));
    }

    /// 0x429BA / 0x429CA / 0x42E78 / 0x42ED0: choose a counter or escape.
    fn enemy_vice_counterattack(&mut self, e: &mut Enemy) {
        let center = (self.cam.s2.cam_x as i16).wrapping_add(0xC0).wrapping_sub(e.b.xi());
        let table = if in_win(center, 0xC0) { 0x42A24 } else { 0x42A84 };
        self.enemy_vice_leftmost_target(e);
        let mut dx = self.body(e.m.w(0x76)).xi().wrapping_sub(e.b.xi()).wrapping_abs();
        if dx >= 0x60 { dx = 0x40; }
        let row = (dx as u16 & 0x60) as u32;
        let r = self.rand() & 31;
        match self.assets.enemy.t.u8(table, row + r as u32) {
            0 => {
                e.b.set_state(0x0200_0402);
                e.m.sw(0xB6, 12);
                e.m.sw(0xB8, 0x48); // intentional combo 9 script cursor
                self.enemy_vice_combo_next(e);
            }
            1 => {
                self.enemy_vice_escape_jump_start(e);
            }
            _ => self.enemy_vice_retreat_start(e),
        }
    }

    /// 0x42E78 / 0x42EAE: initialize the escape jump and move in the same frame.
    fn enemy_vice_escape_jump_start(&mut self, e: &mut Enemy) {
        e.b.act = 2; e.b.step = 2;
        e.m.sb(0xB1, 10);
        // 0x4488C: furthest arena cell, scan descending; ties retain the first.
        let mut distance = 0i16;
        let mut cell = e.m.b(0x7C);
        let (table, count) = if e.b.var26 >> 8 == 0 { (0x44868, 9) } else { (0x44812, 16) };
        for i in (0..count).rev() {
            // 0x1926: `sub.w` of the spot (the table holds the offsets round the target, a negative one is a big unsigned word) and `neg.w`
            // on the borrow, so a spot left of the thug is "far" on the unsigned side and its sum is negative for the signed `cmp.w` / `ble`
            // of 0x448AE (M7-06: the Walther of ep4_jack 4256 picks cell 12, not the farthest by the signed distance).
            let x = self.assets.enemy.t.u16(table, i * 4);
            let z = self.assets.enemy.t.u16(table, i * 4 + 2);
            let (sx, sz) = (e.b.xi() as u16, e.b.zi() as u16);
            let dx = if x < sx { sx - x } else { x - sx };
            let dz = if z < sz { sz - z } else { z - sz };
            let d = dx.wrapping_add(dz) as i16;
            if d > distance { distance = d; cell = i as u8; }
        }
        e.m.sb(0x7C, cell);
        self.enemy_vice_destination(e);
        self.enemy_vice_jump_start(e);
        self.enemy_anim(&mut e.b, 26);
        self.enemy_vice_face(e);
        // 0x42EAE is reached in the entry frame as well.
        if e.b.param_b == 0 { e.b.motion(); }
        self.enemy_step(&mut e.b);
    }

    /// 0x42ED0: initialize retreat and select the signed leftmost target.
    fn enemy_vice_retreat_start(&mut self, e: &mut Enemy) {
        e.b.act = 2; e.b.step = 2;
        e.m.sb(0xB1, 8);
        self.enemy_anim(&mut e.b, 25);
        self.enemy_vice_walk_speed(e);
        e.m.sb(0x7D, if e.b.var26 >> 8 == 0 { 0xFF } else { 0 });
        e.m.sw(0xAE, 0);
        e.m.sb(0xB0, 200);
        self.enemy_vice_leftmost_target(e);
    }

    /// 0x42EFC / 0x448EC / 0x44A88: retreat until 80 Manhattan pixels away.
    fn enemy_vice_retreat(&mut self, e: &mut Enemy) {
        if e.b.airborne != 0 { e.b.set_state(0x0200_0800); return; }
        let p = self.body(e.m.w(0x76));
        e.m.sw(0x88, p.xi() as u16);
        e.m.sw(0x8A, p.zi() as u16);
        self.enemy_vice_turn(e, false, true);
        let mut arrived = Game::enemy_stuck_check(e);
        if !arrived {
            let t = e.m.b(0xB0);
            e.m.sb(0xB0, t.wrapping_sub(1));
            let p = self.body(e.m.w(0x76));
            let d = p.xi().wrapping_sub(e.b.xi()).wrapping_abs()
                .wrapping_add(p.zi().wrapping_sub(e.b.zi()).wrapping_abs());
            arrived = t == 0 || d >= 80;
        }
        if arrived {
            e.m.sb(0xB0, 120);
            self.enemy_vice_new_cell(e);
            e.b.act = 2; e.b.step = 0;
            e.m.sb(0xB1, 0);
        } else {
            self.enemy_move_dir(&mut e.b);
            self.enemy_step(&mut e.b);
        }
    }

    /// 0x425B6..0x4268C: Vice's init. 0x939B6 is the optional debug marker.
    fn enemy_vice_init(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.var26 >> 8 == 0 {
            self.enemy_vice_hopper(e, h); // 0x4DCC6, before selecting a target
            e.b.id = 0x0C;
            e.m.sw(0x84, 0x30);
            e.b.b63 = 2;
            e.b.anim_table = 0x438F2;
            self.enemy_set_boss_hp(&mut e.b, 0);
            e.m.sw(0xC0, e.b.hp);
        } else {
            e.b.id = 0x0D;
            e.m.sw(0x84, 0x34);
            e.b.b63 = 0;
            e.b.anim_table = 0x43E54;
            self.enemy_set_hp(&mut e.b);
        }
        e.b.b72 = 60;
        self.enemy_anim(&mut e.b, 0);
        e.b.box_table = 0x103000;
        e.b.multi_box = false;
        e.b.stun = 11;
        e.b.reaction = 0;
        for o in [0x79, 0x7A, 0x7D, 0xB1, 0xBD, 0x87, 0xC3] {
            e.m.sb(o, 0);
        }
        e.m.sb(0xA8, 2);
        e.m.sw(0xAA, 0);
        e.m.sw(0xBE, 900);
        e.m.sb(0xC2, 2);
        e.m.sw(0x76, self.enemy_pick_target(&e.b)); // 0x32032
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sb(0x7D, if e.b.var26 >> 8 == 0 { 0xFF } else { 0 }); // 0x44EEC
        self.enemy_vice_initial_target(e); // 0x44298, one RNG call
        self.enemy_vice_initial_cell(e); // 0x44478
        e.b.sub = e.b.sub.wrapping_add(2);
    }

    /// 0x4DCC6: make the chained Rock Hopper. Its intro is M5-10.
    fn enemy_vice_hopper(&mut self, e: &Enemy, h: Handle) {
        let Some(i) = self.enemies.alloc() else { return };
        let hopper = &mut self.enemies.slots[i];
        hopper.b.live = 1;
        hopper.b.k20 = 29 * 4;
        hopper.m.sl(0xA0, 0xFFFF_0000 | h as u32);
        // Word writes preserve the fractional halves of the allocated slot.
        hopper.b.x = (hopper.b.x & 0xFFFF) | (0x2D0 << 16);
        hopper.b.y = (hopper.b.y & 0xFFFF) | ((e.b.yi() as i32) << 16);
        hopper.b.z = (hopper.b.z & 0xFFFF) | (0x148 << 16);
        hopper.b.var26 = 0;
    }

    /// 0x44298 / 0x44416: pack live heroes, then select with the ROM weight row.
    fn enemy_vice_initial_target(&mut self, e: &mut Enemy) {
        for o in [0xA2, 0xA4, 0xA6] {
            e.m.sw(o, 0);
        }
        e.m.sb(0xA0, 0);
        let r = self.rand() & 15;
        let mut count = 0u16;
        for i in 0..3 {
            if self.players[i].b.live != 0 {
                e.m.sw(0xA2 + 2 * count as usize, Game::player_handle(i));
                count += 1;
            }
        }
        if count == 0 {
            e.m.sw(0xA2, Game::player_handle(0));
        }
        let row = self.assets.enemy.t.u16(0x442E0, 2 * count.saturating_sub(1) as u32);
        let mut choice = self.assets.enemy.t.u8(0x442E0, row as u32 + r as u32);
        if choice as i8 > 2 {
            choice = 0;
        }
        e.m.sb(0xA0, choice);
        e.m.sw(0x76, e.m.w(0xA2 + 2 * choice as usize));
    }

    /// 0x426C0..0x42880: intro jump table (r6 = 0, 2, ..., 0x12).
    fn enemy_vice_intro(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => { // 0x426C0
                if (self.cam.s2.cam_x as i16) >= 0x1D0 {
                    e.b.act = e.b.act.wrapping_add(2);
                    if let Some(talk) = self.spawn_prop(50, 0, None) {
                        e.m.sw(0xAC, talk);
                        self.sound(0x20);
                    }
                }
            }
            2 => { // 0x426F4
                if self.body(e.m.w(0xAC)).live == 0 {
                    e.m.sb(0x80, 40);
                    e.b.act = e.b.act.wrapping_add(2);
                }
            }
            4 => { // 0x4270A, branches on zero, not borrow
                let t = e.m.b(0x80).wrapping_sub(1);
                e.m.sb(0x80, t);
                if t == 0 {
                    self.spawn_prop(76, 0, None);
                    self.enemy_anim(&mut e.b, 0x14);
                    e.b.act = e.b.act.wrapping_add(2);
                    self.enemy_vice_pull(e);
                }
            }
            6 => self.enemy_vice_pull(e),
            8 => { // 0x4274A, advances on borrow
                let t = e.m.b(0x80);
                e.m.sb(0x80, t.wrapping_sub(1));
                if t == 0 {
                    e.b.act = e.b.act.wrapping_add(2);
                    e.m.sb(0x80, 0);
                    self.enemy_anim(&mut e.b, 1);
                    self.enemy_set_speed(&mut e.b, 7);
                    e.m.sw(0x88, 0x317);
                    e.m.sw(0x8A, 0x136);
                    e.b.dir = Game::enemy_angle_to(&e.b, 0x317, 0x136);
                    self.enemy_vice_walk(e);
                }
            }
            0x0A => self.enemy_vice_walk(e),
            0x0C => self.enemy_vice_captions(e), // 0x427B2 (fallthrough from walk)
            0x0E => self.enemy_vice_gun(e, h),
            0x10 => { // 0x4284C
                let t = e.m.b(0x80);
                e.m.sb(0x80, t.wrapping_sub(1));
                if t == 0 {
                    e.b.act = e.b.act.wrapping_add(2);
                    self.spawn.g.b4fc = 0;
                    self.sound(0x0E);
                }
            }
            0x12 => self.enemy_vice_fight_start(e, h),
            _ => {}
        }
    }

    /// 0x42736: finish the chain-pull animation, then wait 60 (borrow) frames.
    fn enemy_vice_pull(&self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            e.b.act = e.b.act.wrapping_add(2);
            e.m.sb(0x80, 60);
        }
    }

    /// 0x42782 / 0x44AA2: unsigned arrival windows before motion and animation.
    fn enemy_vice_walk(&mut self, e: &mut Enemy) {
        let (x, z) = (e.m.w(0x88), e.m.w(0x8A));
        if in_win(e.b.xi().wrapping_sub(x as i16), 8) && in_win(e.b.zi().wrapping_sub(z as i16), 4) {
            e.b.act = e.b.act.wrapping_add(2);
            e.m.sb(0x80, 20);
            self.enemy_vice_captions(e);
        } else {
            e.b.dir = Game::enemy_angle_to(&e.b, x, z);
            self.enemy_move_dir(&mut e.b);
            self.enemy_step(&mut e.b);
        }
    }

    /// 0x427B2..0x427F4: both captions spawn immediately on arrival.
    fn enemy_vice_captions(&mut self, e: &mut Enemy) {
        if self.spawn_prop(66, 0x0B, None).is_some() && self.flow.region.0 != 0 {
            self.spawn_prop(66, 0x55, None);
        }
        self.enemy_anim(&mut e.b, 0x15);
        e.b.act = e.b.act.wrapping_add(2);
        // The 0x427F8 countdown also runs in this frame.
        e.m.sb(0x80, e.m.b(0x80).wrapping_sub(1));
    }

    /// 0x427F8..0x42846: delay, gun animation, frame-flag spark and sound.
    fn enemy_vice_gun(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(0x80);
        e.m.sb(0x80, t.wrapping_sub(1));
        if t == 0 {
            e.m.sb(0x80, 0);
            if self.enemy_step(&mut e.b) {
                e.b.act = e.b.act.wrapping_add(2);
                e.m.sb(0x80, 60);
            }
        }
        if e.b.param_a != 0 {
            e.b.param_a = 0;
            // The pool updater holds e outside the slot. Publish its facing for
            // the shared 0x92422 service, then restore the empty slot's header.
            let i = self.enemy_index(h);
            let facing = self.enemies.slots[i].b.facing;
            self.enemies.slots[i].b.facing = e.b.facing;
            self.fx_spawn_spark_facing(e.b.facing, e.b.xi().wrapping_sub(0x50), e.b.yi().wrapping_add(0x40), e.b.zi(), 0);
            self.enemies.slots[i].b.facing = facing;
            self.snd_request(0x3B, 0x18, 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::trace::{Collect, F};
    use crate::harness::TraceReader;
    use std::path::Path;

    /// 0x44316 with a fallen target (+$C8 = 0x23) goes to 0x443A6 (M7-06, ep4_3p 6562): the list of the present heroes and the next one of it, no
    /// weights. With the first hero gone and two present the cursor +$A0 goes 0 -> 1 and the target is the second cell of the list.
    #[test]
    fn a_fallen_target_is_replaced_by_the_next_hero_of_the_list() {
        let mut g = Game::new(Box::leak(Box::new(dino_assets::AssetPack::default())));
        g.players[0].b.live = 0;
        g.players[0].action = 0x23;
        g.players[1].b.live = 1;
        g.players[2].b.live = 1;
        let mut e = Enemy::default();
        e.m.sw(0x76, Game::player_handle(0));
        let before = g.w.rng;
        g.enemy_vice_retarget(&mut e);
        assert_eq!(g.w.rng, before, "two heroes: no random number");
        assert_eq!(e.m.w(0x76), Game::player_handle(2));
        assert_eq!(e.m.b(0xA0), 1);
        assert_eq!((e.m.w(0xA2), e.m.w(0xA4), e.m.w(0xA6)), (Game::player_handle(1), Game::player_handle(2), 0));
    }

    /// The recordings and ROM stay outside source control. Check each invocation
    /// against MAME, including the init RNG roll and all modeled object bytes.
    #[test]
    fn vice_intro_matches_mame_in_step() {
        let rom = Path::new("../../../dino.zip");
        let directory = Path::new("../../../re/out/traces");
        if !rom.exists() || !directory.exists() {
            return;
        }
        let assets = Box::leak(Box::new(dino_extract::extract(rom).unwrap()));
        let mut frames = 0;
        for name in [
            "m600_bs_v1_v1_nat", "boss_vice_2p_r0", "boss_vice_2p_r24",
            "boss_vice_3p_r0", "boss_vice_3p_r24",
        ] {
            let path = directory.join(format!("{name}.mame.trc"));
            if !path.exists() {
                continue;
            }
            let mut reader = TraceReader::open(path.to_str().unwrap()).unwrap();
            let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
            let mut g = Game::new(assets);
            let mut checked = 0;
            while let Some(frame) = reader.next() {
                let ram = reader.region("ram").unwrap().data.clone();
                let gfx = reader.region("gfx").unwrap().data.clone();
                if let Some((old, video)) = &previous {
                    g.trace_load(old, video);
                    for i in 0..g.enemies.slots.len() {
                        let h = g.enemies.handle(i);
                        let a = h as usize;
                        let e = &g.enemies.slots[i];
                        if e.b.live == 0 || e.b.mode != 0 || !Game::enemy_vice_intro_active(&e.b) || e.b.act == 0x12 {
                            continue;
                        }
                        let mut e = std::mem::take(&mut g.enemies.slots[i]);
                        g.enemy_class18(&mut e, h, i);
                        let mut image = Collect::default();
                        e.trace_visit(&mut F::new(&mut image, 0xFF0000 | h as u32, assets));
                        for offset in 0..0xE0 {
                            if image.mask[a + offset] == 0 {
                                continue;
                            }
                            // 0x4DF80..0x4DF8A, owned by M5-10: the Hopper
                            // consumes Vice's +$2D before Vice's own update.
                            if offset == 0x2D && old[a + offset] != 0 && ram[a + offset] == 0 {
                                assert!(g.enemies.slots.iter().any(|hopper|
                                    hopper.b.live != 0 && hopper.b.k20 == 29 * 4
                                    && hopper.b.act == 8 && hopper.m.l(0xA0) as u16 == h));
                                continue;
                            }
                            assert_eq!(image.ram[a + offset], ram[a + offset],
                                "{name} frame {frame} Vice +{offset:02X}");
                        }
                        if old[a + 5] == 0 {
                            assert_eq!(g.w.rng, u16::from_be_bytes([ram[0x8014], ram[0x8015]]),
                                "{name} frame {frame} init RNG");
                        }
                        g.enemies.slots[i] = e;
                        checked += 1;
                    }
                }
                previous = Some((ram, gfx));
            }
            assert!(checked > 0, "{name}: no Vice intro invocations");
            eprintln!("{name}: {checked} Vice intro invocations equal MAME");
            frames += checked;
        }
        assert!(frames > 0, "no Vice reference recordings found");
    }

    /// 0x428CE: step the recorded world with its input and pokes, then compare
    /// every modeled Vice byte in native AI and generic reaction frames.
    #[test]
    fn vice_native_fight_matches_mame_in_step() {
        let rom = Path::new("../../../dino.zip");
        let directory = Path::new("../../../re/out/traces");
        if !rom.exists() || !directory.exists() {
            return;
        }
        let assets = Box::leak(Box::new(dino_extract::extract(rom).unwrap()));
        let mut frames = 0;
        for name in [
            "m600_bs_v1_v1_nat", "boss_vice_2p_r0", "boss_vice_2p_r24",
            "boss_vice_3p_r0", "boss_vice_3p_r24",
            "m600_bs_v3_v2_dmg", "m600_bs_v3_v6_dmg",
            "m511b_vice_pinned", "m600_bs_v2_v9_alt",
            "m600_h10_ks_wal", "m513_wal_ai_l0", "m513_wal_ai_l31",
        ] {
            let path = directory.join(format!("{name}.mame.trc"));
            if !path.exists() {
                continue;
            }
            let mut reader = TraceReader::open(path.to_str().unwrap()).unwrap();
            let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
            let mut g = Game::new(assets);
            let input = if !name.starts_with("boss_") {
                directory.join(format!("{name}.input.txt"))
            } else { Path::new("../../../re/harness/scripts").join(format!("{name}.txt")) };
            let script = crate::input::Script::parse(&std::fs::read_to_string(input).unwrap());
            let mut pokes = crate::harness::PokeSet::default();
            let sidecar = directory.join(format!("{name}.pokes.txt"));
            if sidecar.exists() { pokes.add_file(sidecar.to_str().unwrap(), crate::harness::PokeClock::Power).unwrap(); }
            let mut checked = 0;
            while let Some(frame) = reader.next() {
                let ram = reader.region("ram").unwrap().data.clone();
                let gfx = reader.region("gfx").unwrap().data.clone();
                if let Some((old, video)) = &previous {
                    g.trace_load(old, video);
                    g.hold_unported = true;
                    for i in 0..g.enemies.slots.len() {
                        let h = g.enemies.handle(i);
                        let a = h as usize;
                        let e = &g.enemies.slots[i];
                        if e.b.live == 0 || e.b.k20 != 72 { continue; }
                        if e.b.var26 >> 8 == 0 && (e.b.mode != 2
                            || !(matches!(e.b.sub, 2 | 6) || (e.b.sub == 0 && matches!(e.b.act, 0 | 2 | 4 | 6 | 8 | 0xA)))) {
                            continue;
                        }
                        // M7-12 owns the Hopper's hostile AI. The held placeholder
                        // does not register its monster attack in this frame.
                        let attacker = u16::from_be_bytes([ram[a + 0x68], ram[a + 0x69]]) as usize;
                        if ram[a + 5] == 2 && old[a + 5] == 0 && attacker + 0x21 < ram.len()
                            && u16::from_be_bytes([ram[attacker + 0x20], ram[attacker + 0x21]]) == 29 * 4 {
                            continue;
                        }
                        pokes.apply(&mut g, frame, false).unwrap();
                        g.w.input = script.at(frame as u32).players;
                        g.tick();
                        pokes.apply(&mut g, frame, true).unwrap();
                        let mut e = std::mem::take(&mut g.enemies.slots[i]);
                        let mut image = Collect::default();
                        e.trace_visit(&mut F::new(&mut image, 0xFF0000 | h as u32, assets));
                        for offset in 0..0xE0 {
                            if image.mask[a + offset] == 0 {
                                continue;
                            }
                            // The hostile Hopper (M7-12) runs before Vice and
                            // rolls twice in its attack state. Its held AI cannot
                            // supply the pause RNG, so check the timer separately.
                            if offset == 0x80 && old[a+6] == 4
                                && matches!(old[a+7], 0xE | 0x12)
                                && ram[a+7] == 0x20 && g.w.rng != u16::from_be_bytes([ram[0x8014], ram[0x8015]])
                                && (0..i).any(|j| { let q = crate::game::POOL_A.0 as usize + j * 0xE0;
                                    old[q] != 0 && old[q+0x21] == 29*4 && old[q+4..q+8] == [2,0,4,0] }) {
                                let mut isolated = Game::new(assets);
                                isolated.w.level = g.w.level;
                                isolated.w.rng = u16::from_be_bytes([old[0x8014], old[0x8015]]);
                                isolated.rand();
                                isolated.rand(); // two upstream Hopper rolls
                                let mut vice = e.clone();
                                isolated.enemy_vice_combo_pause(&mut vice);
                                assert_eq!(vice.m.b(0x80), ram[a+0x80], "{name} frame {frame} isolated gun pause");
                                assert_eq!(isolated.w.rng, u16::from_be_bytes([ram[0x8014], ram[0x8015]]));
                                continue;
                            }
                            assert_eq!(image.ram[a + offset], ram[a + offset],
                                "{name} frame {frame} Vice +{offset:02X} old {:02X?} new {:02X?} vx {} vz {} dest {:02X?}",
                                &old[a+4..a+8], &ram[a+4..a+8], e.b.vx, e.b.vz, &old[a+0x88..a+0x8C]);
                        }
                        g.enemies.slots[i] = e;
                        checked += 1;
                    }
                }
                previous = Some((ram, gfx));
            }

            eprintln!("{name}: {checked} Vice native fight invocations equal MAME");
            frames += checked;
        }
        assert!(frames > 0, "no Vice reference recordings found");
    }

    /// 0x43892: replay every recorded Vice/Walther death invocation and score.
    #[test]
    fn vice_and_walther_death_matches_mame() {
        let rom = Path::new("../../../dino.zip");
        if !rom.exists() { return; }
        let assets = Box::leak(Box::new(dino_extract::extract(rom).unwrap()));
        let mut checked = 0;
        for name in ["m600_bs_v1_v1_nat", "m600_bs_v2_v9_alt", "m600_h10_ks_wal"] {
            let path = format!("../../../re/out/traces/{name}.mame.trc");
            if !Path::new(&path).exists() { continue; }
            let mut reader = TraceReader::open(&path).unwrap();
            let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
            let mut g = Game::new(assets);
            let mut count = 0;
            while let Some(frame) = reader.next() {
                let ram = reader.region("ram").unwrap().data.clone();
                let gfx = reader.region("gfx").unwrap().data.clone();
                if let Some((old, video)) = &previous {
                    g.trace_load(old, video);
                    for i in 0..g.enemies.slots.len() {
                        let h = g.enemies.handle(i);
                        let a = h as usize;
                        let e = &g.enemies.slots[i];
                        if e.b.live == 0 || e.b.k20 != 72 || e.b.mode != 4 { continue; }
                        // Area teardown frees all pools after the object update.
                        if ram[a] == 0 && old[0x84D9..0x84DB] != ram[0x84D9..0x84DB] { continue; }
                        let mut e = std::mem::take(&mut g.enemies.slots[i]);
                        g.enemy_class18(&mut e, h, i);
                        let mut image = Collect::default();
                        e.trace_visit(&mut F::new(&mut image, 0xFF0000 | h as u32, assets));
                        for offset in 0..0xE0 {
                            if image.mask[a + offset] != 0 {
                                assert_eq!(image.ram[a+offset], ram[a+offset], "{name} frame {frame} death +{offset:02X}");
                            }
                        }
                        if old[a+5] == 0 {
                            for j in 0..3 {
                                let p = Game::player_handle(j) as usize;
                                let expected = u32::from_be_bytes(ram[p+0x100..p+0x104].try_into().unwrap());
                                assert_eq!(g.players[j].score, expected, "{name} frame {frame} score P{j}");
                            }
                        }
                        g.enemies.slots[i] = e;
                        count += 1;
                    }
                }
                previous = Some((ram, gfx));
            }
            eprintln!("{name}: {count} death invocations equal MAME");
            checked += count;
        }
        assert!(checked > 0);
    }

    /// 0x31E14 / 0x31424: BS-A1 HP and H10's hit plus kill score.
    #[test]
    fn vice_hp_and_walther_score_acceptance() {
        let rom = Path::new("../../../dino.zip");
        if !rom.exists() { return; }
        let assets = Box::leak(Box::new(dino_extract::extract(rom).unwrap()));
        let mut g = Game::new(assets);
        g.w.level = 12;
        for (players, hp) in [(0, 500), (1, 600), (2, 700)] {
            g.w.players_minus_one = players;
            let mut e = Enemy::default();
            g.enemy_set_boss_hp(&mut e.b, 0);
            assert_eq!(e.b.hp, hp);
        }
        let mut e = Enemy::default();
        e.b.id = 0x0D;
        e.b.mode = 4;
        e.b.var26 = 0x200;
        e.b.who = 0;
        g.enemy_add_score(0, 0x100); // 0x325A4: one ordinary hit, packed BCD
        g.enemy_vice_dying(&mut e, 0);
        assert_eq!(g.players[0].score, 0x8100);
        assert_eq!((e.b.sub, e.m.b(0x80)), (2, 119));
        for _ in 0..119 { g.enemy_vice_dying(&mut e, 0); }
        assert_eq!(e.m.b(0x80), 0);
        g.enemy_vice_dying(&mut e, 0);
        assert_eq!(e.b.mode, 6);
        assert_eq!(g.players[0].score, 0x8100);
    }

    /// 0x42B52 / 0x42B88: signed HP comparisons and expiry on recovery only.
    #[test]
    fn vice_rage_uses_two_thresholds_and_recovery_expiry() {
        let mut g = Game::new(Box::leak(Box::new(dino_assets::AssetPack::default())));
        let mut e = Enemy::default();
        e.b.hp_bar = 100;
        e.m.sb(0xA8, 2);
        e.b.hp = 51;
        g.enemy_vice_rage_check(&mut e);
        assert_eq!(e.m.b(0xA8), 2);
        e.b.hp = 50;
        g.enemy_vice_rage_check(&mut e);
        assert_eq!((e.m.b(0xA8), e.m.b(0xBD), e.m.w(0xAA)), (1, 2, 1800));
        e.b.hp = 25;
        g.enemy_vice_rage_check(&mut e);
        assert_eq!(e.m.b(0xA8), 1);
        e.m.sw(0xAA, 0);
        g.enemy_vice_rage_check(&mut e);
        assert_eq!(e.m.b(0xBD), 0);
        e.b.hp = 26;
        g.enemy_vice_rage_check(&mut e);
        assert_eq!(e.m.b(0xA8), 1);
        e.b.hp = 25;
        g.enemy_vice_rage_check(&mut e);
        assert_eq!((e.m.b(0xA8), e.m.b(0xBD), e.m.w(0xAA)), (0, 2, 1800));
    }

    /// 0x4270A tests zero, while 0x4274A and 0x4284C test borrow.
    #[test]
    fn vice_wait_uses_byte_borrow_and_camera_uses_signed_words() {
        let mut g = Game::new(Box::leak(Box::new(dino_assets::AssetPack::default())));
        let mut e = Enemy::default();
        e.b.act = 0x10;
        e.m.sb(0x80, 60);
        g.spawn.g.b4fc = 1;
        for _ in 0..60 {
            g.enemy_vice_intro(&mut e, 0);
            assert_eq!(e.b.act, 0x10);
        }
        g.enemy_vice_intro(&mut e, 0);
        assert_eq!((e.b.act, e.m.b(0x80), g.spawn.g.b4fc), (0x12, 0xFF, 0));
        assert_eq!(g.w.sounds, [0x0E]);
        e.b.act = 0;
        g.cam.s2.cam_x = 0xFFFF;
        g.enemy_vice_intro(&mut e, 0);
        assert_eq!(e.b.act, 0);
        g.cam.s2.cam_x = 0x1D0;
        g.enemy_vice_intro(&mut e, 0);
        assert_eq!(e.b.act, 2);
        assert_eq!(g.w.sounds, [0x0E, 0x20]);
    }
}
