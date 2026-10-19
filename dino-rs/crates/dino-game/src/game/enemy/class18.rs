//! Vice T., class 18 variant 0: intro, decide and approach. Attacks remain partial.
use super::{in_win, Enemy};
use crate::game::{Game, Handle};

impl Game {
    /// 0x42590 / 0x425BC / 0x428CE: use sprites for native intro and AI states.
    pub(crate) fn enemy_vice_intro_active(b: &crate::game::body::Body) -> bool {
        b.k20 == 18 * 4 && b.var26 >> 8 == 0
            && (b.mode == 0 || (b.mode == 2 && b.sub == 0 && b.act <= 2))
    }

    /// 0x42590 / 0x425A4 / 0x42884: init, intro and the ported alive states.
    pub(super) fn enemy_class18(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        // One switch for the partial port: all other modes and approach variants
        // still use the placeholder (or snapshot hold in the harness).
        match (e.b.var26 >> 8, e.b.mode, e.b.sub, e.b.act) {
            (0, 0, _, _) => {}
            (0, 2, 0, 0 | 2) if e.m.b(0xBD) == 0 && e.m.b(0xB1) <= 6 => {
                self.enemy_vice_alive(e, h);
                return;
            }
            _ => {
                self.enemy_class_unported(e, h, i);
                return;
            }
        }
        if e.b.sub == 0 {
            self.enemy_vice_init(e, h);
        }
        self.enemy_vice_intro(e, h);
        self.enemy_draw_check(&mut e.b, h); // 0x426A6 -> 0x49CA
    }

    /// 0x42884 / 0x428CE: alive timers, poise refill, terrain and obstacle attack.
    fn enemy_vice_alive(&mut self, e: &mut Enemy, h: Handle) {
        for offset in [0xBE, 0xC4, 0xAA] {
            e.m.sw(offset, e.m.w(offset).saturating_sub(1));
        }
        if e.m.b(0x79) != 0 {
            e.m.sb(0x79, e.m.b(0x79).wrapping_sub(1));
            if e.m.b(0x79) == 0 { e.b.stun = 11; }
        }
        if e.b.act == 0 && e.b.airborne != 0 {
            e.b.set_state(0x0200_0800); // 0x42B3E -> 0x43738
        } else if e.b.act == 2 || e.m.b(0x7A) == 0 {
            self.enemy_vice_approach(e);
        } else {
            // 0x42B42: decide waits until zero, with same-frame fallthrough.
            e.m.sb(0x7A, e.m.b(0x7A).wrapping_sub(1));
            if e.m.b(0x7A) == 0 { self.enemy_vice_approach(e); }
        }
        self.terrain_collide(&mut e.b, h, false);
        e.m.sb(0xC3, e.m.b(0xC3).wrapping_sub(1));
        if e.m.b(0xC3) == 0 {
            e.m.sb(0xC3, 200);
            if e.b.wall & 0x80 != 0 {
                // 0x4292E: prepare the obstacle combo for the unported attack state.
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
            // 0x44DEC / 0x44E12 and 0x44E52 / 0x44E6C scan P3, P2, P1.
            for behind in [false, true] {
                for i in (0..3).rev() {
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
                        // 0x43604: the back elbow entry. Its action stays unported.
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
                    if self.enemy_vice_attack_roll() {
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
                    let cell = e.m.b(0x7C) as u32 * 4;
                    let b = Game::enemy_angle_to(&e.b, self.assets.enemy.t.u16(0x44868, cell),
                        self.assets.enemy.t.u16(0x44868, cell + 2));
                    e.m.sb(0x90, 0);
                    if (a ^ b) & 16 != 0 {
                        e.m.sb(0xB1, 0);
                        self.enemy_vice_walk_start(e);
                    } else {
                        e.m.sb(0xB5, 1);
                        e.m.sw(0xC4, 600);
                        self.enemy_anim(&mut e.b, 4);
                        self.enemy_set_speed(&mut e.b, 13);
                        e.m.sb(0x7D, 0xFF);
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
            _ => {}
        }
    }

    /// 0x44B5A: one RNG draw, signed byte compare, every 61 eligible updates.
    fn enemy_vice_attack_roll(&mut self) -> bool {
        let threshold = self.assets.enemy.t.u8(0x44B84, self.w.level as u32);
        ((self.rand() & 31) as i8) < threshold as i8
    }

    /// 0x44316 / 0x44416: rotate the packed target list using the ROM weights.
    fn enemy_vice_retarget(&mut self, e: &mut Enemy) {
        let target = e.m.w(0x76);
        if let Some(crate::game::Slot::Player(i)) = crate::game::slot_of(target) {
            if self.players[i].action == 0x23 { return; }
        }
        let r = self.rand() & 15;
        let mut count = 0usize;
        for i in 0..3 {
            if self.players[i].b.live != 0 {
                e.m.sw(0xA2 + count * 2, Game::player_handle(i));
                count += 1;
            }
        }
        for i in count..3 { e.m.sw(0xA2 + i * 2, 0); }
        if count == 0 { e.m.sw(0xA2, Game::player_handle(0)); }
        let last = count.saturating_sub(1) as u16;
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

    /// 0x444E8 (Vice branch): roll a relative arena cell, retaining byte wrap.
    fn enemy_vice_new_cell(&mut self, e: &mut Enemy) {
        let cell = e.m.b(0x7C);
        let r = self.rand() & 31;
        let delta = self.assets.enemy.t.u8(0x44672, cell as u32 * 32 + r as u32);
        e.m.sb(0x7C, cell.wrapping_add(delta));
    }

    /// 0x42C50 / 0x44F2E: walking animation and difficulty-dependent speed.
    fn enemy_vice_walk_start(&self, e: &mut Enemy) {
        self.enemy_anim(&mut e.b, 1);
        let speed = self.assets.enemy.t.u16(0x44F4E, (self.w.level as u32 & 31) * 2);
        self.enemy_set_speed(&mut e.b, speed);
        e.m.sb(0x7D, 0xFF);
        e.m.sw(0xAE, 0);
        e.m.sb(0xB0, 120);
    }

    /// 0x447CE / 0x44852 / 0x44994 / 0x449C6 / 0x44CCC / 0x44CE4.
    fn enemy_vice_destination(&self, e: &mut Enemy) {
        let cell = e.m.b(0x7C) as u32 * 4;
        let x = self.assets.enemy.t.u16(0x44868, cell);
        let z = self.assets.enemy.t.u16(0x44868, cell + 2);
        e.m.sw(0x88, x);
        e.m.sw(0x8A, z);
        let mut dir = Game::enemy_angle_to(&e.b, x, z);
        let turn = if e.m.b(0xB5) == 0 { 4u8 } else { 2u8 };
        let d = e.b.dir.wrapping_sub(dir).wrapping_add(turn) & 31;
        if d > turn * 2 {
            dir = e.b.dir.wrapping_add(if d >= 16 + turn { turn } else { turn.wrapping_neg() }) & 31;
        }
        e.b.dir = dir;
        if e.m.b(0xB5) == 0 { self.enemy_vice_face(e); }
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
            e.b.set_state(0x0200_0800); // 0x43738, unported fall
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
        e.b.airborne = 1;
        e.b.vy = 0x700;
        e.b.ay = -0x40;
        let dx = (e.b.xi() as u16).wrapping_sub(e.m.w(0x88));
        let magnitude = (dx as i16).wrapping_abs() as u16;
        let mut q = magnitude / 56;
        let mut r = magnitude % 56;
        if (e.b.xi() as u16) >= e.m.w(0x88) { q = q.wrapping_neg(); r = r.wrapping_neg(); }
        e.b.vx = (r.wrapping_add((q & 0xFF) << 8)) as i16;
        if (q.wrapping_add(0x500)) >= 0xA00 { e.b.vx = 0x500; }
        e.b.ax = 0;
        let dz = (e.b.zi() as u16).wrapping_sub(e.m.w(0x8A));
        let magnitude = (dz as i16).wrapping_abs() as u16;
        let mut q = magnitude / 56;
        let mut r = magnitude % 56;
        if (e.b.zi() as u16) >= e.m.w(0x8A) {
            q = q.wrapping_neg();
            if q == 0 && (r as i16) < 0 { q = 0xFFFF; }
            r = r.wrapping_neg();
        }
        e.b.vz = ((r << 2).wrapping_add((q & 0xFF) << 8)) as i16;
        e.b.az = 0;
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

    /// 0x425B6..0x4268C: Vice's init. 0x939B6 is the optional debug marker.
    fn enemy_vice_init(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_vice_hopper(e, h); // 0x4DCC6, before selecting a target
        e.b.id = 0x0C;
        e.m.sw(0x84, 0x30);
        e.b.b72 = 60;
        e.b.b63 = 2;
        e.b.anim_table = 0x438F2;
        self.enemy_set_boss_hp(&mut e.b, 0);
        e.m.sw(0xC0, e.b.hp);
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
        e.m.sb(0x7D, 0xFF); // 0x44EEC
        self.enemy_vice_initial_target(e); // 0x44298, one RNG call
        e.m.sb(0x7C, 4); // 0x44478, Vice branch has no RNG
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
            0x12 => { // 0x42864..0x4287C; alive AI entry is M5-11
                if let Some(controller) = self.boss_spawn_controller(h) {
                    // 0x7BEDC copies sign-extended a6, not a 24-bit bus address.
                    if let Some(crate::game::Slot::Prop(i)) = crate::game::slot_of(controller) {
                        self.props.slots[i].m.sl(0xAA, 0xFFFF_0000 | h as u32);
                    }
                }
                self.hud.boss = h;
                e.b.mode = e.b.mode.wrapping_add(2);
                e.b.sub = 0;
                e.b.act = 0;
                e.b.step = 0;
                self.enemy_vice_approach(e); // 0x42880 falls into 0x42B96
            }
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
            self.fx_spawn_spark_at(h, e.b.xi().wrapping_sub(0x50), e.b.yi().wrapping_add(0x40), e.b.zi(), 0);
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
    /// every modeled Vice byte on AI frames. Hit/grab entry belongs to Part B.
    #[test]
    fn vice_decide_approach_matches_mame_in_step() {
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
        ] {
            let path = directory.join(format!("{name}.mame.trc"));
            if !path.exists() {
                continue;
            }
            let mut reader = TraceReader::open(path.to_str().unwrap()).unwrap();
            let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
            let mut g = Game::new(assets);
            let input = if name.starts_with("m600_") {
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
                        if ram[a + 5] != 0 { continue; } // hit / grab entry is outside Part A
                        if e.b.live == 0 || e.b.k20 != 72 || e.b.var26 >> 8 != 0 || e.b.mode != 2
                            || e.b.sub != 0 || e.b.act > 2 || e.m.b(0xBD) != 0 || e.m.b(0xB1) > 6 {
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

            eprintln!("{name}: {checked} Vice decide/approach invocations equal MAME");
            frames += checked;
        }
        assert!(frames > 0, "no Vice reference recordings found");
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
