//! Rock Hopper (0x4DCF6): entrances, neutral and hostile AI, hits and holds.
use super::Enemy;
use crate::game::{Game, Handle};

impl Game {
    /// 0x4DCF6: all entrances, alive modes, reactions, holds and pool A cleanup.
    pub(super) fn enemy_class29(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => match e.b.sub {
                0 => self.enemy_hopper_init(e, h),
                2 => match e.b.var26 >> 8 {
                    0 => {
                        e.b.sub = 4;
                        self.enemy_hopper_intro(e);
                        self.enemy_draw_check(&mut e.b, h);
                    }
                    6 => self.enemy_hopper_drop(e, h),
                    _ => self.enemy_hopper_fight_start(e),
                },
                4 => {
                    self.enemy_hopper_intro(e);
                    self.enemy_draw_check(&mut e.b, h);
                }
                6 => self.enemy_hopper_fight_start(e),
                _ => {}
            },
            2 => {
                match e.b.sub {
                    0 => self.enemy_hopper_alive(e, h, i),
                    2 => {
                        self.enemy_hopper_hit(e, h);
                        self.enemy_draw_check(&mut e.b, h);
                    }
                    4 => {
                        self.enemy_hopper_hold(e, h);
                        self.enemy_draw_check(&mut e.b, h);
                    }
                    _ => {}
                }
                e.m.sb(0xB8, 0);
            }
            4 | 6 => self.enemy_free_pool_a(e, i), // 0x4F168
            _ => {}
        }
    }

    /// 0x4DD1E..0x4DE04: shared init; +$5C deliberately remains stale.
    fn enemy_hopper_init(&mut self, e: &mut Enemy, h: Handle) {
        e.b.id = 4;
        e.m.sw(0x84, 0x10);
        e.b.anim_table = 0x4F4E4;
        self.enemy_anim(&mut e.b, 0);
        e.b.box_table = 0x106000;
        e.b.multi_box = true;
        e.b.stun = 10;
        e.b.reaction = 0;
        for o in [
            0x79, 0x7A, 0x7B, 0x7D, 0xA4, 0xB8, 0xB9, 0xBA, 0xC1, 0xAC, 0xAD, 0xBB,
        ] {
            e.m.sb(o, 0);
        }
        e.b.b72 = 0xC8;
        e.m.sb(0x87, 4);
        e.b.speed_table = 0x4FA30;
        e.m.sw(0x76, self.enemy_pick_target(&e.b));
        e.m.sw(0xA8, 0);
        e.m.sw(0xAA, 0);
        self.enemy_set_hp(&mut e.b);
        self.enemy_snap_to_floor(e);
        e.m.sb(0x7D, 0xFF);
        if self.w.stage == 4 || self.w.stage == 7 {
            let palette = if self.w.stage == 4 { 0x9E } else { 0x9F };
            e.b.palette_override = palette;
            e.m.sb(0xBB, palette);
        }
        if self.w.stage == 7 {
            e.m.sw(0xA8, 6);
            e.m.sb(0xA4, 1);
        }
        e.b.sub = e.b.sub.wrapping_add(2);
        // 0x4DDFE jmp 0x94522: the sweat beads (pool C class 20) over this hopper.
        if e.b.var26 >> 8 == 0 {
            // The allocator reads the owner through its handle while pool A has
            // taken this object out of its slot for the update.
            let i = self.enemy_index(h);
            let saved = std::mem::replace(&mut self.enemies.slots[i].b, e.b.clone());
            self.fx_spawn_sweat(h);
            self.enemies.slots[i].b = saved;
        }
    }

    /// 0x4DEE4..0x4DFEC: partner states, including same-frame fallthroughs.
    fn enemy_hopper_intro(&mut self, e: &mut Enemy) {
        let parent = e.m.l(0xA0) as u16;
        if e.b.act == 0 {
            if e.b.step == 0 {
                e.b.step = e.b.step.wrapping_add(2);
                self.enemy_anim(&mut e.b, 0);
                e.b.facing = 0;
            }
            if self.body(parent).act != 2 {
                self.enemy_step(&mut e.b);
                return;
            }
            e.b.act = e.b.act.wrapping_add(2);
            e.b.step = 0;
        }
        if e.b.act == 2 {
            if self.body(parent).act != 4 {
                return;
            }
            e.b.act = e.b.act.wrapping_add(2);
            self.enemy_anim(&mut e.b, 1);
        }
        if e.b.act == 4 {
            if !self.enemy_step(&mut e.b) {
                return;
            }
            e.b.act = e.b.act.wrapping_add(2);
            self.enemy_anim(&mut e.b, 2);
        }
        if e.b.act == 6 {
            if self.body(parent).act != 0xE {
                self.enemy_step(&mut e.b);
                return;
            }
            e.b.act = e.b.act.wrapping_add(2);
            self.enemy_anim(&mut e.b, 3);
        }
        if e.b.act == 8 {
            if self.body(parent).param_b != 0 {
                self.body_mut(parent).param_b = 0;
                e.b.timer = 1;
            }
            if !self.enemy_step(&mut e.b) {
                return;
            }
            e.b.act = e.b.act.wrapping_add(2);
            e.m.sb(0xA4, 1);
            self.enemy_anim(&mut e.b, 4);
        }
        if e.b.act == 0xA {
            if e.b.param_b == 0 {
                self.enemy_step(&mut e.b);
                return;
            }
            e.b.act = e.b.act.wrapping_add(2);
            self.enemy_hopper_sound(e);
            self.enemy_monster_flash_start(e);
        }
        if e.b.act == 0xC {
            if !self.enemy_monster_flash_tick(e) {
                self.enemy_step(&mut e.b);
                return;
            }
            e.b.act = e.b.act.wrapping_add(2);
            self.enemy_anim(&mut e.b, 5);
        }
        if e.b.act == 0xE && self.enemy_step(&mut e.b) {
            e.b.sub = e.b.sub.wrapping_add(2);
        }
    }

    /// 0x4DFEE / 0x4E01E: choose the entrance's initial AI without running it yet.
    fn enemy_hopper_fight_start(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0000);
        self.enemy_hopper_anim_save(e, 0);
        e.b.dir = 0x18;
        e.b.facing = 0;
        e.m.sw(0xAE, e.b.xi() as u16);
        e.m.sw(0x76, self.enemy_pick_target(&e.b));
        let mode = if self.w.stage == 7 || e.b.var26 >> 8 == 0 {
            6
        } else if e.b.var26 >> 8 == 4 {
            0
        } else {
            2
        };
        self.enemy_hopper_reset(e, mode);
    }

    /// 0x4DE28..0x4DEE2: the unused variant 6 falling entrance.
    fn enemy_hopper_drop(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.act == 0 {
            e.b.act = 2;
            self.enemy_anim(&mut e.b, 0x1D);
            e.b.vx = if (self.cam.s2.cam_x as i16)
                .wrapping_add(0xC0)
                .wrapping_sub(e.b.xi())
                >= 0
            {
                -0x180
            } else {
                0x180
            };
            e.b.ax = 0;
            e.b.vy = 0x400;
            e.b.ay = -0x48;
            e.b.vz = -0x100;
            e.b.az = 0;
            e.b.set_yi_keep(0x18);
            e.b.set_zi_keep(0x78);
            e.b.airborne = 1;
            e.m.sb(0x80, 0x14);
        }
        if e.b.act == 2 {
            if e.b.airborne != 0 {
                if e.b.param_b == 0 {
                    e.b.param_b = 0;
                    e.b.motion();
                }
                self.enemy_step(&mut e.b);
                if Self::enemy_hopper_count(e, 0x80) {
                    e.m.sb(0x80, 0);
                    self.terrain_collide_free(&mut e.b);
                }
                self.enemy_draw_check(&mut e.b, h);
                self.enemy_monster_register(h);
                return;
            }
            e.b.act = 4;
            self.enemy_anim(&mut e.b, 0x1E);
        }
        if self.enemy_step(&mut e.b) {
            self.enemy_hopper_fight_start(e);
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_monster_register(h);
    }

    /// 0x4E0C4 / 0x4E14E / 0x4E39C / 0x4E59C / 0x4EA02: AI entries.
    fn enemy_hopper_reset(&self, e: &mut Enemy, mode: u16) {
        e.m.sw(0xA8, mode);
        e.b.act = 0;
        e.b.step = 0;
        match mode {
            0 | 2 | 4 | 6 => {
                e.m.sw(0xAA, 0);
                e.m.sb(0xAC, 0);
                e.m.sb(0xC0, if mode == 0 { 0xFF } else { 0 });
                if mode == 6 {
                    e.m.sw(0xB4, 0);
                    e.m.sb(0xA4, 1);
                }
            }
            8 => {
                e.m.sb(0xB2, 0);
                e.m.sw(0xB4, 0);
                e.m.sb(0xBA, 1);
            }
            _ => {}
        }
    }

    /// 0x4EBEC: become satisfied and growl, preserving the caller's return.
    fn enemy_hopper_satisfied(&mut self, e: &mut Enemy) {
        e.m.sw(0xA8, 0xA);
        e.b.live = 2;
        e.b.act = 0;
        e.b.step = 0;
        self.enemy_hopper_hop_start(e);
        self.enemy_hopper_sound(e);
    }

    /// 0x4ECA2: anger roar before the colour transition.
    fn enemy_hopper_anger(&mut self, e: &mut Enemy) {
        e.m.sw(0xA8, 0xC);
        e.b.act = 0;
        e.b.step = 0;
        e.m.sb(0xA4, 1);
        e.b.live = 2;
        self.enemy_hopper_sound(e);
        self.enemy_anim(&mut e.b, 4);
    }

    /// 0x4ECC6: wait for the roar flag, then flash to hostile colours.
    fn enemy_hopper_colour(&mut self, e: &mut Enemy) {
        if e.b.act == 0 {
            if e.b.param_b == 0 {
                self.enemy_step(&mut e.b);
            } else {
                e.b.act = 2;
                self.enemy_monster_flash_start(e);
            }
        } else if self.enemy_monster_flash_tick(e) {
            e.b.live = 1;
            self.enemy_hopper_reset(e, 6);
        } else {
            self.enemy_step(&mut e.b);
        }
    }

    /// 0x4E0E0: dormant roar timer (the unused variant 4).
    fn enemy_hopper_dormant(&mut self, e: &mut Enemy) {
        match e.b.act {
            0 => {
                e.m.sw(0xAE, e.b.xi() as u16);
                self.enemy_hopper_anim_save(e, 0);
                e.m.sb(0x80, 60);
                e.b.act = 2;
            }
            2 => {
                let t = e.m.b(0x80).wrapping_sub(1);
                e.m.sb(0x80, t);
                if t == 0 {
                    e.m.sb(0x80, 60);
                    if self.rand() & 15 == 0 {
                        e.b.act = 4;
                        self.enemy_anim(&mut e.b, 1);
                        return;
                    }
                }
                self.enemy_step(&mut e.b);
            }
            4 => {
                if self.enemy_step(&mut e.b) {
                    self.enemy_hopper_reset(e, 0);
                }
            }
            _ => {}
        }
    }

    /// 0x4F16E: signed camera leave test; a zero +$5C explicitly disables it.
    fn enemy_hopper_leave_test(&self, e: &Enemy) -> bool {
        e.m.w(0x5C) != 0 && (e.m.w(0x5C) as i16) < self.cam.s2.cam_x as i16
    }

    /// 0x4E168 / 0x4E3B6 / 0x4E5C0: grounded lifetime and underflow-based anger rolls.
    fn enemy_hopper_lifetime(&mut self, e: &mut Enemy) -> bool {
        if e.b.airborne != 0 {
            return false;
        }
        if self.enemy_hopper_leave_test(e) {
            self.enemy_hopper_satisfied(e);
            return true;
        }
        let t = e.m.w(0xAA).wrapping_add(1);
        e.m.sw(0xAA, t);
        let angry = e.m.w(0xA8) == 6;
        if !angry && t >= 1200 {
            self.enemy_hopper_satisfied(e);
            return true;
        }
        if t >= if angry { 900 } else { 600 } && Self::enemy_hopper_count(e, 0xAC) {
            e.m.sb(0xAC, 10);
            let r = self.rand() & 15;
            if angry && r >= 14 {
                self.enemy_hopper_satisfied(e);
                return true;
            }
            if !angry && r == 0 {
                self.enemy_hopper_anger(e);
                return true;
            }
        }
        false
    }

    /// 0x4E114 and other byte timer sites: SUBQ.B with carry on underflow.
    fn enemy_hopper_count(e: &mut Enemy, o: usize) -> bool {
        let t = e.m.b(o);
        e.m.sb(o, t.wrapping_sub(1));
        t == 0
    }

    /// 0x4F36C and callers: remember both frame flags around an animation change.
    fn enemy_hopper_save(e: &mut Enemy) {
        e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
    }

    /// 0x4E252 / 0x4E498 / 0x4E710: animation start with saved flags.
    fn enemy_hopper_anim_save(&self, e: &mut Enemy, anim: u16) {
        self.enemy_anim(&mut e.b, anim);
        Self::enemy_hopper_save(e);
    }

    /// 0x4E29A: edge turn uses signed compares and has asymmetric screen limits.
    fn enemy_hopper_edge(&self, e: &mut Enemy) -> bool {
        let x =
            e.b.xi()
                .wrapping_sub(self.cam.s2.cam_x as i16)
                .wrapping_add(0x40);
        if x <= 0x80 {
            e.b.dir = 8;
            e.b.facing = 1;
            true
        } else if x.wrapping_add(0x180) >= 0x300 {
            e.b.dir = 0x18;
            e.b.facing = 0;
            true
        } else {
            false
        }
    }

    /// 0x4E2E0: leap speeds from the ROM, one attack serial even without a hitbox.
    fn enemy_hopper_leap(&mut self, e: &mut Enemy) {
        let t = &self.assets.enemy.t;
        let mut vx = t.i16(0x4FAF0, 0);
        let mut ax = t.i16(0x4FAF0, 2);
        if e.b.facing == 0 {
            vx = vx.wrapping_neg();
            ax = ax.wrapping_neg();
        }
        e.b.vx = vx;
        e.b.ax = ax;
        e.b.vy = t.i16(0x4FAF0, 4);
        e.b.ay = t.i16(0x4FAF0, 6);
        e.b.vz = 0;
        e.b.az = 0;
        e.b.airborne = 1;
        self.enemy_begin_attack(&mut e.b, 0x22);
    }

    /// 0x4E32C / 0x4E52C / 0x4E7D6: leap and landing wait shared by the three AIs.
    fn enemy_hopper_air(&mut self, e: &mut Enemy) {
        if Self::enemy_hopper_count(e, 0x80) {
            e.m.sb(0x80, 0);
            e.b.motion();
        }
        if e.b.airborne == 0 {
            e.m.sw(0xAE, e.b.xi() as u16);
            e.m.sb(0x80, 15);
            e.b.act = e.b.act.wrapping_add(2);
            self.enemy_hopper_anim_save(e, 13);
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x4F204 / 0x4F220: the last live player stays selected even outside the window.
    fn enemy_hopper_players(&self, e: &mut Enemy) -> bool {
        for p in 0..3 {
            let b = &self.players[p].b;
            if b.live == 0 {
                continue;
            }
            e.m.sw(0x76, Self::player_handle(p));
            if (b.xi().wrapping_sub(e.b.xi()).wrapping_add(0x50) as u16) <= 0xA0
                && (b.zi().wrapping_sub(e.b.zi()).wrapping_add(0x10) as u16) <= 0x20
            {
                return true;
            }
        }
        false
    }

    /// 0x4F260: nearest eligible previous-frame enemy, last one wins equal distances. The list is the shared storage of 0x4BEA
    /// (`combat_hurt_list_seen`: this frame's pushes have replaced its high entries; MAME ep8_3p 748).
    fn enemy_hopper_enemies(&self, e: &mut Enemy, own: Handle) {
        let list = self.combat_hurt_list_seen();
        if list.is_empty() {
            e.m.sw(0x76, own);
            return;
        }
        let mut best = 0xFFFF;
        for h in list {
            let b = self.body(h);
            if let Some(crate::game::Slot::Enemy(i)) = crate::game::slot_of(h)
                && (self.enemies.slots[i].m.b(0x7D) as i8) < 0 {
                    continue;
                }
            let dx = b.xi().wrapping_sub(e.b.xi()).wrapping_abs() as u16;
            let dz = b.zi().wrapping_sub(e.b.zi()).wrapping_abs() as u16;
            let d = dx.wrapping_add(dz);
            if d <= best {
                best = d;
                e.m.sw(0x76, h);
            }
        }
    }

    /// 0x4F300: shared in-front attack window.
    fn enemy_hopper_in_front(&self, e: &Enemy, h: Handle) -> bool {
        let b = self.body(h);
        b.live != 0
            && (b.zi().wrapping_sub(e.b.zi()).wrapping_add(11) as u16) <= 22
            && (e
                .b
                .xi()
                .wrapping_add(if e.b.facing == 0 { -64 } else { 64 })
                .wrapping_sub(b.xi())
                .wrapping_add(16) as u16)
                <= 32
    }

    /// 0x4F2B4: screen-gated attack search, players first then the previous enemy list.
    fn enemy_hopper_attack_window(&self, e: &mut Enemy) -> bool {
        let x = e.b.xi() as u16;
        let left = x.wrapping_sub(0x30);
        if left < self.cam.s2.cam_x || left.wrapping_sub(self.cam.s2.cam_x) > 0x120 {
            return false;
        }
        for h in (0..3)
            .map(Self::player_handle)
            .chain(self.combat_hurt_list_seen())
        {
            if self.enemy_hopper_in_front(e, h) {
                e.m.sw(0x76, h);
                return true;
            }
        }
        false
    }

    /// 0x4E4B2 / 0x4E75C / 0x4EAF0: aim, save the destination and clamp to forward hops.
    fn enemy_hopper_aim(&self, e: &mut Enemy, hostile: bool, own: Handle) {
        let b = if e.m.w(0x76) == own {
            &e.b
        } else {
            self.body(e.m.w(0x76))
        };
        let (x, z) = (b.xi() as u16, b.zi() as u16);
        e.m.sw(0x88, x);
        e.m.sw(0x8A, z);
        let mut d = Self::enemy_angle_to(&e.b, x, z);
        let (lo, hi) = if hostile { (13, 19) } else { (12, 20) };
        if e.b.facing == 0 {
            d = ((d.wrapping_sub(8) & 31).clamp(lo, hi)).wrapping_add(8) & 31;
        } else {
            d = ((d.wrapping_add(8) & 31).clamp(lo, hi)).wrapping_sub(8) & 31;
        }
        e.b.dir = d;
    }

    /// 0x4E168..0x4E59A: KIMAMA and MATOWARI, including hop decision RNG order.
    fn enemy_hopper_neutral(&mut self, e: &mut Enemy) {
        if self.enemy_hopper_lifetime(e) {
            return;
        }
        let follow = e.m.w(0xA8) == 4;
        match e.b.act {
            0 => {
                e.b.act = 2;
                e.m.sb(0xAD, u8::from(follow) + u8::from(self.rank.aggression >= 8));
                self.enemy_hopper_hop_start(e);
            }
            2 => {
                if self.w.scripted != 0 {
                    self.enemy_hopper_satisfied(e);
                    return;
                }
                if e.b.airborne == 0 {
                    if !follow && self.enemy_hopper_players(e) {
                        self.enemy_hopper_reset(e, 4);
                        return;
                    }
                    Self::enemy_hopper_save(e);
                    if !self.enemy_step(&mut e.b) {
                        self.enemy_hopper_hop_step(e);
                        return;
                    }
                    if !self.enemy_hopper_edge(e) {
                        if self.rand() & 15 >= 12 {
                            self.enemy_hopper_jump_start(e, 13);
                            return;
                        }
                        if self.rand() & 15 == 15 {
                            e.m.sw(0xAE, e.b.xi() as u16);
                            self.enemy_hopper_anim_save(e, 8);
                            e.b.act = 8;
                            return;
                        }
                    }
                    if follow {
                        self.enemy_hopper_players(e);
                        self.enemy_hopper_aim(e, false, 0);
                    } else {
                        let r = self.rand() & 15;
                        let d = self.assets.enemy.t.u8(0x4E28A, r as u32);
                        e.b.dir = d.wrapping_add(if e.b.facing == 0 { 16 } else { 0 });
                    }
                    self.enemy_hopper_hop_step(e);
                } else {
                    self.enemy_hopper_jump_start(e, 13);
                }
            }
            4 => self.enemy_hopper_air(e),
            6 => {
                if Self::enemy_hopper_count(e, 0x80) {
                    self.enemy_hopper_anim_save(e, 8);
                    e.b.act = 0;
                }
            }
            8 => {
                Self::enemy_hopper_save(e);
                if self.enemy_step(&mut e.b) {
                    e.b.act = 0;
                }
            }
            _ => {}
        }
    }

    /// 0x4E222 / 0x4E468 / 0x4E6DA: takeoff entry does not tick the new animation.
    fn enemy_hopper_jump_start(&mut self, e: &mut Enemy, anim: u16) {
        e.b.act = e.b.act.wrapping_add(2);
        self.enemy_anim(&mut e.b, anim);
        e.m.sb(0x80, e.b.timer);
        self.enemy_hopper_leap(e);
    }

    /// 0x4E5C0..0x4EBE8: OKORI and KYOURAN.
    fn enemy_hopper_hostile(&mut self, e: &mut Enemy, h: Handle) {
        let rage = e.m.w(0xA8) == 8;
        if rage {
            if self.enemy_hopper_leave_test(e) {
                self.enemy_hopper_satisfied(e);
                return;
            }
        } else if self.enemy_hopper_lifetime(e) {
            return;
        }
        match e.b.act {
            0 => {
                e.m.sw(0xAE, e.b.xi() as u16);
                if rage {
                    self.enemy_hopper_anim_save(e, 8);
                    e.m.sw(0x76, self.enemy_pick_target(&e.b));
                    e.b.act = 2;
                } else {
                    e.m.sw(0x76, self.enemy_pick_target(&e.b));
                    e.m.sw(0xB0, 0);
                    e.b.act = 4;
                    e.m.sb(0xAD, 2 + u8::from(self.rank.aggression >= 8));
                    self.enemy_hopper_hop_start(e);
                }
            }
            2 => {
                Self::enemy_hopper_save(e);
                if self.enemy_step(&mut e.b) {
                    if rage {
                        e.b.act = 4;
                        e.m.sb(0xAD, 3);
                        self.enemy_hopper_hop_start(e);
                    } else {
                        e.b.act = 0;
                    }
                }
            }
            4 => {
                if self.w.scripted != 0 {
                    self.enemy_hopper_satisfied(e);
                    return;
                }
                if rage || e.b.airborne == 0 {
                    let t = e.m.w(0xB4);
                    e.m.sw(0xB4, t.wrapping_sub(1));
                    if t == 0 {
                        e.m.sw(0xB4, 0);
                        if self.spawn.g.b4d5 == 0 && self.enemy_hopper_attack_window(e) {
                            let b = self.body(e.m.w(0x76));
                            if b.mode == 2 && b.sub == 0 {
                                if rage {
                                    let n = e.m.b(0xB2).wrapping_add(1);
                                    e.m.sb(0xB2, n);
                                    if n >= 3 {
                                        self.enemy_hopper_reset(e, 6);
                                        return;
                                    }
                                }
                                e.b.act = if rage { 6 } else { 10 };
                                let r = self.rand() & 15;
                                e.b.step = self.assets.enemy.t.u8(0x4FAF8, r as u32);
                                return;
                            }
                        }
                    }
                    Self::enemy_hopper_save(e);
                    if !self.enemy_step(&mut e.b) {
                        self.enemy_hopper_hop_step(e);
                        return;
                    }
                    let edge = self.enemy_hopper_edge(e);
                    if !rage && !edge {
                        if self.rand() & 15 >= 13 {
                            self.enemy_hopper_jump_start(e, 25);
                            return;
                        }
                        if self.rand() & 15 == 15 {
                            e.b.act = 2;
                            e.m.sw(0xAE, e.b.xi() as u16);
                            self.enemy_hopper_anim_save(e, 8);
                            return;
                        }
                    }
                    if rage {
                        self.enemy_hopper_players(e);
                    } else {
                        let b = if e.m.w(0x76) == h {
                            &e.b
                        } else {
                            self.body(e.m.w(0x76))
                        };
                        let t = e.m.w(0xB0);
                        let normal = b.mode == 2 && b.sub == 0;
                                        if normal {
                            e.m.sw(0xB0, t.wrapping_sub(1));
                        }
                        if !normal || t == 0 {
                            e.m.sw(0xB0, 300);
                            let found = self.enemy_hopper_players(e);
                            if !found || self.rand() & 15 >= 9 {
                                self.enemy_hopper_enemies(e, h);
                            }
                        }
                    }
                    self.enemy_hopper_aim(e, true, h);
                    self.enemy_hopper_hop_step(e);
                } else {
                    self.enemy_hopper_jump_start(e, 25);
                }
            }
            6 if !rage => self.enemy_hopper_air(e),
            8 if !rage => {
                if Self::enemy_hopper_count(e, 0x80) {
                    self.enemy_hopper_anim_save(e, 8);
                    e.b.act = 0;
                }
            }
            6 | 10 => self.enemy_hopper_attack(e),
            8 | 12 => self.enemy_hopper_recovery(e, rage),
            _ => {}
        }
    }

    /// 0x4E8DE: claw, bite, and pounce decision states.
    fn enemy_hopper_attack(&mut self, e: &mut Enemy) {
        match e.b.step {
            0 | 4 => {
                let bite = e.b.step == 4;
                self.enemy_hopper_anim_save(e, if bite { 15 } else { 14 });
                self.enemy_begin_attack(&mut e.b, if bite { 0x21 } else { 0x20 });
                e.b.step += 2;
            }
            2 | 6 => {
                Self::enemy_hopper_save(e);
                if self.enemy_step(&mut e.b) {
                    e.b.act += 2;
                    e.b.step = 0;
                }
            }
            8 => {
                let b = self.body(e.m.w(0x76));
                let mut ok = b.live == 1 && b.mode == 2 && b.sub == 0 && b.airborne == 0;
                if let Some(crate::game::Slot::Player(i)) = crate::game::slot_of(e.m.w(0x76)) {
                    ok &= self.players[i].action != 0x11;
                }
                if ok {
                    e.m.sw(0xB6, e.m.w(0xA8));
                    e.b.sub = 4;
                    e.b.act = 0;
                    e.b.step = 0;
                    self.snd_request(0x5F, self.snd_pan_x(e.b.xi()) as u32, 0);
                } else {
                    e.b.step = 0;
                }
            }
            _ => {}
        }
    }

    /// 0x4E834 / 0x4EB6A: pause while the victim is dizzy, then recover with one roll.
    fn enemy_hopper_recovery(&mut self, e: &mut Enemy, rage: bool) {
        let dizzy = if let Some(crate::game::Slot::Player(i)) = crate::game::slot_of(e.m.w(0x76)) {
            self.players[i].action == 0x23
        } else {
            false
        };
        if e.b.step == 0 && dizzy {
            self.enemy_hopper_anim_save(e, 16);
            e.m.sb(0x80, if rage { 60 } else { 120 });
            e.b.step = 2;
            self.snd_request(0x5C, self.snd_pan_x(e.b.xi()) as u32, 0);
        } else if e.b.step == 0 {
            self.enemy_hopper_recovery_end(e);
            return;
        }
        Self::enemy_hopper_save(e);
        self.enemy_step(&mut e.b);
        if Self::enemy_hopper_count(e, 0x80) {
            self.enemy_hopper_recovery_end(e);
        } else if !rage && e.m.b(0x80) & 0x3F == 0 {
            self.snd_request(0x5C, self.snd_pan_x(e.b.xi()) as u32, 0);
        }
    }

    /// 0x4E898 / 0x4EBBC: attack cooldown and the idle versus approach decision.
    fn enemy_hopper_recovery_end(&mut self, e: &mut Enemy) {
        e.m.sw(0xB4, 300);
        if self.rand() & 15 >= 12 {
            e.b.act = 0;
            self.enemy_anim(&mut e.b, 8);
        } else {
            e.b.act = 4;
            e.b.step = 0;
            self.enemy_hopper_hop_start(e);
        }
    }

    /// 0x4ED16: the guard recoil preserves the old AI in +$B6.
    fn enemy_hopper_block(&mut self, e: &mut Enemy) {
        if e.b.act == 0 {
            e.m.sw(0xAE, e.b.xi() as u16);
            self.enemy_hopper_anim_save(e, 20);
            e.m.sb(0x80, 30);
            e.b.vx = 0x400;
            e.b.ax = -0x24;
            if e.b.facing != 0 {
                e.b.vx = -e.b.vx;
                e.b.ax = -e.b.ax;
            }
            e.b.vy = 0;
            e.b.ay = 0;
            e.b.vz = 0;
            e.b.az = 0;
            e.b.act = 2;
        } else if Self::enemy_hopper_count(e, 0x80) {
            e.b.act = 0;
            e.b.step = 0;
            e.m.sw(0xA8, e.m.w(0xB6));
            e.b.live = 1;
        } else {
            e.b.vx = e.b.vx.wrapping_add(e.b.ax);
            e.m.sl(
                0xAE,
                e.m.l(0xAE).wrapping_add(((e.b.vx as i32) << 8) as u32),
            );
        }
    }

    /// 0x4EDA8..0x4EF3E: flinch, knockdown and burn; five hostile hits trigger KYOURAN once.
    fn enemy_hopper_hit(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.act == 0 && e.b.step == 0 && e.b.hp != 0 && e.b.stun != 0 && e.b.airborne == 0 {
            self.enemy_anim(&mut e.b, 17);
            self.enemy_hopper_hud(e, h);
            e.b.step = 2;
            e.b.hitstop = e.b.hitstop.wrapping_sub(1);
            if e.b.hitstop == 0 {
                self.enemy_hopper_hit_end(e);
            }
            return;
        }
        if e.b.act == 0 && e.b.step != 0 {
            e.b.hitstop = e.b.hitstop.wrapping_sub(1);
            if e.b.hitstop == 0 {
                self.enemy_hopper_hit_end(e);
            }
            return;
        }
        match e.b.step {
            0 => {
                if e.b.act == 4 {
                    self.enemy_burn_flames(h, e);
                    e.b.palette_override = 0x84;
                }
                e.b.act = 2;
                self.enemy_anim(&mut e.b, 18);
                let t = &self.assets.enemy.t;
                let sign = if e.b.facing == 0 { 1i16 } else { -1 };
                e.b.vx = t.i16(0x4FB08, 0).wrapping_mul(sign);
                e.b.ax = t.i16(0x4FB08, 2).wrapping_mul(sign);
                e.b.vy = t.i16(0x4FB08, 4);
                e.b.ay = t.i16(0x4FB08, 6);
                e.b.vz = 0;
                e.b.az = 0;
                e.b.airborne = 1;
                self.enemy_hopper_hud(e, h);
                e.b.step = 2;
                self.enemy_hopper_hit_fly(e, h);
            }
            2 => self.enemy_hopper_hit_fly(e, h),
            4 => {
                if Self::enemy_hopper_count(e, 0x80) {
                    self.enemy_anim(&mut e.b, if e.m.b(0xA4) == 0 { 26 } else { 27 });
                    e.b.step = 6;
                }
                self.enemy_step(&mut e.b);
            }
            6 => {
                if self.enemy_step(&mut e.b) {
                    self.enemy_hopper_hit_end(e);
                }
            }
            _ => {}
        }
        // 0x4EE0C and 0x4EDB6 both append the body. Keep the duplicate
        // draw in the heavy/burn path, as the original sprite list does.
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x16B36 at 0x4EDEC / 0x4EE74: request the current object's HP bar.
    fn enemy_hopper_hud(&mut self, e: &Enemy, h: Handle) {
        if e.b.who > 2 {
            return;
        }
        let i = e.b.who as usize;
        let w = self.hud.ring_w[i];
        let m = &mut self.hud.rings[i][(w >> 4) as usize];
        if (m[0] as i16) >= 0 {
            return;
        }
        *m = [
            e.b.id as u16,
            e.b.hp_bar,
            e.b.hp_prev,
            e.b.hp,
            0xFFFF,
            h,
            m[6],
            m[7],
        ];
        self.hud.ring_w[i] = w.wrapping_add(16);
    }

    /// 0x4EE7E: register the flying hurt body, advance motion, then land.
    fn enemy_hopper_hit_fly(&mut self, e: &mut Enemy, h: Handle) {
        self.combat.push(0x6AC2, h);
        self.enemy_step(&mut e.b);
        e.b.motion();
        self.terrain_collide_free(&mut e.b);
        if e.b.airborne == 0 {
            e.b.palette_override = e.m.b(0xBB);
            self.enemy_anim(&mut e.b, 19);
            e.m.sb(0x80, 60);
            e.b.step = 4;
        }
    }

    /// 0x4EEE8: restore colour/poise and re-enter the appropriate AI.
    fn enemy_hopper_hit_end(&mut self, e: &mut Enemy) {
        e.b.dir = if e.b.facing == 0 { 24 } else { 8 };
        e.b.stun = 10;
        e.m.sw(0xAE, e.b.xi() as u16);
        e.b.palette_override = e.m.b(0xBB);
        e.b.sub = 0;
        e.b.act = 0;
        e.b.step = 0;
        if e.b.hp == 0 {
            self.enemy_hopper_satisfied(e);
            return;
        }
        e.b.live = 1;
        if e.m.b(0xA4) == 0 {
            self.enemy_hopper_anger(e);
            return;
        }
        if e.m.b(0xBA) == 0 {
            let hits = e.m.b(0xB9).wrapping_add(1);
            e.m.sb(0xB9, hits);
            if hits >= 5 {
                self.enemy_hopper_reset(e, 8);
            }
        }
    }

    /// 0x4EF44..0x4F164: pounce attachment, held bite and release.
    fn enemy_hopper_hold(&mut self, e: &mut Enemy, h: Handle) {
        let target = e.m.w(0x76);
        if e.b.act == 0 {
            let b = self.body(target);
            if b.live != 1 || b.mode != 2 || b.sub != 0 || b.airborne != 0 {
                self.enemy_hopper_hold_end(e);
                return;
            }
            if let Some(crate::game::Slot::Enemy(i)) = crate::game::slot_of(target)
                && (self.enemies.slots[i].m.b(0x7D) as i8) < 0 {
                    self.enemy_hopper_hold_end(e);
                    return;
                }
            e.b.partner = target;
            let b = self.body_mut(target);
            b.partner = h;
            b.set_state(0x0206_0000);
            b.kind = 14;
            if let Some(crate::game::Slot::Player(i)) = crate::game::slot_of(target) {
                let p = &mut self.players[i];
                p.action = 0x40;
                p.bc7 = 0;
                p.b.hp_prev = p.b.hp;
                e.b.act = 2;
            } else {
                let i = self.enemy_index(target);
                self.enemies.slots[i].b.who = e.b.who;
                // 0x4EFF8 `move.w $6C(a3), $6E(a3)`: the grab code does not set A3, the census (0x3242E) left it (MAME's watch of the write,
                // Episode 1 with Jack, Hannah and Mustapha, a thug held, frame 11654: A2 = A3 = 0xFFB3F4): the previous HP of that hero becomes his HP.
                let a3 = self.enemy_census_a3();
                let p3 = &mut self.players[a3].b;
                p3.hp_prev = p3.hp;
                e.b.act = 4;
            }
            self.enemy_anim(&mut e.b, 28);
            return;
        }
        if self.enemy_step(&mut e.b) {
            self.enemy_hopper_hold_end(e);
            return;
        }
        if e.b.act == 4 && e.b.kind == 12 {
            self.enemy_hopper_place_enemy(e);
        }
        if e.b.param_a != 0 {
            return;
        }
        e.b.param_a = 1;
        let b = self.body(target);
        if b.live != 1 || b.mode != 2 || b.sub != 6 || b.kind != 14 {
            self.enemy_hopper_hold_end(e);
            return;
        }
        self.enemy_begin_attack(&mut e.b, 0x29);
        if e.b.act == 2 {
            // 0x4F082 `jsr 0x125C0`: A2 is not set by the Hopper's code, the census left it (MAME's watch of the HP write, 2 player Episode 1,
            // frame 12148: A2 = A3 = 0xFFB3F4 with Hannah held and player 2 the last counted target): the damage scale 0x6422 reads its +$84 and
            // the sound pan its X.
            let (w84, ax) = self.enemy_census_a2();
            self.enemy_hit_held_player_scale(e, h, w84, ax);
            let Some(crate::game::Slot::Player(i)) = crate::game::slot_of(target) else {
                return;
            };
            if self.players[i].action != 0x40 {
                self.enemy_hopper_hold_end(e);
                return;
            }
            let b = &mut self.players[i].b;
            b.set_state(0x0202_0800);
            b.set_xi_keep(
                e.b.xi()
                    .wrapping_add(if e.b.facing == 0 { 24 } else { -24 }),
            );
            b.set_yi_keep(e.b.yi().wrapping_add(64));
            b.facing = u8::from(e.b.facing != 0);
        } else {
            self.enemy_hopper_bite_enemy(e, h, target);
            if self.body(target).kind != 14 {
                self.enemy_hopper_hold_end(e);
                return;
            }
            let b = self.body_mut(target);
            b.set_state(0x0202_0600);
            b.set_xi_keep(
                e.b.xi()
                    .wrapping_add(if e.b.facing == 0 { 24 } else { -24 }),
            );
            b.set_yi_keep(e.b.yi().wrapping_add(64));
            b.facing = u8::from(e.b.facing != 0);
        }
    }

    /// 0x3267A, d0 = 0: place an enemy in the Hopper's mouth and load its held frame.
    fn enemy_hopper_place_enemy(&mut self, e: &Enemy) {
        let i = self.enemy_index(e.b.partner);
        let w84 = self.enemies.slots[i].m.w(0x84);
        let t = &self.assets.enemy.t;
        let base = t.u32(0x2E4E0, 0);
        let row = base
            .wrapping_add(t.i16(base, (w84 >> 1) as u32) as i32 as u32)
            .wrapping_add((e.b.param_b as u32) << 3);
        let (mut dx, dy, anim, mut face) =
            (t.i16(row, 0), t.i16(row, 2), t.u16(row, 4), t.u8(row, 6));
        if e.b.facing != 0 {
            dx = dx.wrapping_neg();
            face ^= 1;
        }
        let table = t.u32(0x2E724, w84 as u32);
        let frame = self.assets.sprites.anim(table, anim);
        let b = &mut self.enemies.slots[i].b;
        b.set_xi_keep(e.b.xi().wrapping_add(dx));
        b.set_yi_keep(e.b.yi().wrapping_add(dy));
        b.set_zi_keep(e.b.zi());
        b.facing = face;
        b.load_frame(&self.assets.sprites, frame);
    }

    /// 0x12686 / 0x2B904: the held enemy bite, including the attacker's raw +$C6 power.
    fn enemy_hopper_bite_enemy(&mut self, e: &Enemy, h: Handle, target: Handle) {
        let i = self.enemy_index(target);
        let rec = self.assets.player.enemy_attacks[e.b.atk as usize];
        {
            let v = &mut self.enemies.slots[i];
            v.b.attacker = h;
            v.b.serial = e.b.serial;
            v.b.live = 2;
            v.b.who = 0xFF;
            v.b.set_state(0x0202_0000);
            v.m.sb(0x7A, 0);
            v.b.hp_prev = v.b.hp;
        }
        let v = &self.enemies.slots[i];
        let t = &self.assets.enemy.t;
        let w84 = v.m.w(0x84);
        let mut d = rec.damage as u8 as u32;
        // The attached victim has frame class 0xE, so the counter branch cannot run.
        for o in [0, 2] {
            if t.u8(0xA32FA, w84 as u32 + o).wrapping_mul(2) == e.b.atk {
                let p = (d as u16 as u32) * t.u8(0xA32FA, w84 as u32 + o + 1) as u32;
                d = (p & 0xFFFF0000) | ((p as u16 >> 5) as u32);
                break;
            }
        }
        d = (d as u16 as u32) * e.m.b(0xC6) as u32;
        let index = w84.wrapping_shl(3).wrapping_add(self.w.level);
        d = ((d as u16 as u32) * t.u8(0x2BECE, index as u32) as u32) >> 10;
        if d == 0 {
            d = 1;
        }
        let mut hp = v.b.hp;
        let mut row = 0u16;
        while hp > 100 {
            hp = hp.wrapping_sub(100);
            row = row.wrapping_add(64);
        }
        let r = self.rand() & 0x3E;
        d = (d as u16 as u32) * self.assets.enemy.t.u16(0x2BC4E, row.wrapping_add(r) as u32) as u32;
        let row: u16 = if hp >= 75 {
            0
        } else if hp >= 50 {
            64
        } else if hp >= 25 {
            128
        } else {
            192
        };
        let r = self.rand() & 0x3E;
        d = (d as u16 as u32) * self.assets.enemy.t.u16(0x2BB4E, row.wrapping_add(r) as u32) as u32;
        let damage = ((d as u16) >> 10).max(1);
        let v = &mut self.enemies.slots[i];
        let hp = (v.b.hp as i16).wrapping_sub(damage as i16);
        v.b.hp = if hp > 0 {
            hp as u16
        } else {
            u16::from((v.m.b(0x7D) as i8) < 0)
        };
        v.b.stun = v.b.stun.saturating_sub(rec.stun);
        v.b.reaction = rec.reaction;
        v.b.act = rec.reaction;
        v.b.step = 0;
        v.b.b9a = rec.b7;
        // 0x1270C consumes the score byte before the sound test.
        if rec.score != 0 {
            self.snd_hit(rec.score, e.b.xi(), e.b.facing);
        }
    }

    /// 0x4F038: release cooldown restores the saved hostility mode.
    fn enemy_hopper_hold_end(&self, e: &mut Enemy) {
        e.b.mode = 2;
        e.b.sub = 0;
        e.b.act = 0;
        e.b.step = 0;
        e.m.sw(0xA8, e.m.w(0xB6));
        e.m.sw(0xB4, 180);
    }

    /// 0xA09E: growl with the original world-X pan, without an RNG call.
    fn enemy_hopper_sound(&mut self, e: &Enemy) {
        self.snd_request(0x60, self.snd_pan_x(e.b.xi()) as u32, 0);
    }

    /// 0x4F350: select the hop animation from aggression and gait.
    fn enemy_hopper_hop_start(&self, e: &mut Enemy) {
        let id =
            e.m.b(0xAD)
                .wrapping_add(9)
                .wrapping_add(if e.m.b(0xA4) != 0 { 12 } else { 0 });
        self.enemy_anim(&mut e.b, id as u16);
        e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
    }

    /// 0x4F374: a changed frame flag advances the ground anchor and depth.
    fn enemy_hopper_hop_step(&self, e: &mut Enemy) {
        if e.b.param_b == e.b.w34 as u8 {
            return;
        }
        let off = 4 * e.b.dir as u32;
        let dx = self.assets.enemy.t.i16(0x4FA30, off);
        let dz = self.assets.enemy.t.i16(0x4FA30, off + 2);
        e.b.vx = dx;
        e.b.vz = dz;
        e.m.sl(0xAE, e.m.l(0xAE).wrapping_add(((dx as i32) << 8) as u32));
        e.b.z = e.b.z.wrapping_add((dz as i32) << 8);
    }

    /// 0x4F3B4: per-frame visual hop offset from the fractional ground anchor.
    fn enemy_hopper_position(&self, e: &mut Enemy) {
        if e.b.airborne != 0 {
            return;
        }
        let mut dx = self.assets.enemy.t.i16(0x4F3DA, e.b.param_a as u32);
        if e.b.facing != 0 {
            dx = dx.wrapping_neg();
        }
        e.b.set_xi_keep((e.m.w(0xAE) as i16).wrapping_add(dx));
    }

    /// 0x4E064..0x4E0AE: guard entry, stun timers, AI, terrain and monster list.
    fn enemy_hopper_alive(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.m.b(0xB8) != 0 {
            e.b.act = 0;
            e.b.step = 0;
            e.m.sw(0xB6, e.m.w(0xA8));
            e.m.sw(0xA8, 0xE);
            e.b.live = 2;
        }
        // 0x4E070: $79 counts down; at zero the stun is reset to 10 (move.b #$A,$78).
        let t = e.m.b(0x79);
        if t != 0 {
            e.m.sb(0x79, t.wrapping_sub(1));
            if t == 1 {
                e.b.stun = 10;
            }
        }
        // 0x4E082: $7A counts down and skips the dispatch, the position and the floor walk.
        if e.m.b(0x7A) != 0 {
            e.m.sb(0x7A, e.m.b(0x7A).wrapping_sub(1));
        } else {
            match e.m.w(0xA8) {
                0 => self.enemy_hopper_dormant(e),
                2 | 4 => self.enemy_hopper_neutral(e),
                6 | 8 => self.enemy_hopper_hostile(e, h),
                0xA => self.enemy_hopper_leave(e, i),
                0xC => self.enemy_hopper_colour(e),
                0xE => self.enemy_hopper_block(e),
                _ => {}
            }
            self.enemy_hopper_position(e); // 0x4F3B4
            self.terrain_collide_free(&mut e.b); // 0x12E20
        }
        self.enemy_draw_check(&mut e.b, h); // 0x49CA
        self.enemy_monster_register(h); // 0x4C0A (jmp)
        e.m.sb(0xB8, 0); // 0x4E04E: clr.b $b8 after the sub handler
    }

    /// 0x4EC06..0x4EC9E: satisfied roar, restore colour, hop off screen.
    fn enemy_hopper_leave(&mut self, e: &mut Enemy, i: usize) {
        self.enemy_despawn_if_far(e, i);
        match e.b.act {
            0 => {
                if e.m.b(0xA4) == 0 {
                    e.b.act = 6;
                    self.enemy_hopper_hop_start(e);
                } else {
                    e.m.sb(0xA4, 0);
                    self.enemy_anim(&mut e.b, 4);
                    e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
                    e.b.act = e.b.act.wrapping_add(2);
                }
            }
            2 => {
                if e.b.param_b == 0 {
                    self.enemy_step(&mut e.b);
                } else {
                    e.b.act = e.b.act.wrapping_add(2);
                    self.enemy_monster_flash_start(e);
                }
            }
            4 => {
                if self.enemy_monster_flash_tick(e) {
                    e.b.act = 6;
                    self.enemy_hopper_hop_start(e);
                } else {
                    self.enemy_step(&mut e.b);
                }
            }
            6 => {
                e.m.sw(0xAE, e.b.xi() as u16);
                e.b.dir = 0x18;
                e.b.facing = 0;
                e.m.sb(0xAD, 2);
                e.b.act = e.b.act.wrapping_add(2);
                self.enemy_hopper_hop_start(e);
            }
            8 => {
                e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
                self.enemy_step(&mut e.b);
                self.enemy_hopper_hop_step(e);
            }
            _ => {}
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
    fn hopper_intro_matches_mame_in_step() {
        let rom = Path::new("../../../dino.zip");
        let directory = Path::new("../../../re/out/traces");
        if !rom.exists() || !directory.exists() {
            return;
        }
        let assets = Box::leak(Box::new(dino_extract::extract(rom).unwrap()));
        let mut frames = 0;
        for name in [
            "m600_bs_v1_v1_nat",
            "boss_vice_2p_r0",
            "boss_vice_2p_r24",
            "boss_vice_3p_r0",
            "boss_vice_3p_r24",
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
                        if e.b.live == 0
                            || !(e.b.k20 == 29 * 4
                                && e.b.var26 >> 8 == 0
                                && ((e.b.mode == 0 && e.b.sub <= 4)
                                    || (e.b.mode == 2 && e.b.sub == 0 && e.m.w(0xA8) == 0xA)))
                        {
                            continue;
                        }
                        let mut e = std::mem::take(&mut g.enemies.slots[i]);
                        g.enemy_class29(&mut e, h, i);
                        let mut image = Collect::default();
                        e.trace_visit(&mut F::new(&mut image, 0xFF0000 | h as u32, assets));
                        for offset in 0..0xE0 {
                            if image.mask[a + offset] == 0 {
                                continue;
                            }
                            assert_eq!(
                                image.ram[a + offset],
                                ram[a + offset],
                                "{name} frame {frame} Hopper +{offset:02X}"
                            );
                        }
                        g.enemies.slots[i] = e;
                        checked += 1;
                    }
                }
                previous = Some((ram, gfx));
            }
            assert!(checked > 0, "{name}: no Hopper intro invocations");
            eprintln!("{name}: {checked} Hopper intro invocations equal MAME");
            frames += checked;
        }
        assert!(frames > 0, "no Hopper reference recordings found");
    }

    #[test]
    fn hopper_all_states_match_mame_in_step() {
        let rom = Path::new("../../../dino.zip");
        if !rom.exists() {
            return;
        }
        let assets = Box::leak(Box::new(dino_extract::extract(rom).unwrap()));
        let mut total = 0;
        for name in [
            "m700_a12_h3_neutral_replay",
            "m700_a13_h3_hit_replay",
            "m700_a14_h3_hostile_l31_replay",
            "m712_hostile_l0",
            "m712_fire_hopper3",
            "m712_fifth_hit",
            "m712_dormant",
            "m712_drop",
            "m712_enemy_hold",
            "m712_guard_rage4",
        ] {
            let prefix = format!("../../../re/out/traces/{name}");
            if !Path::new(&format!("{prefix}.mame.trc")).exists() {
                continue;
            }
            let mut reader = TraceReader::open(&format!("{prefix}.mame.trc")).unwrap();
            let script = crate::input::Script::load(&format!("{prefix}.input.txt")).unwrap();
            let mut pokes = crate::harness::PokeSet::default();
            pokes
                .add_file(
                    &format!("{prefix}.pokes.txt"),
                    crate::harness::PokeClock::Game,
                )
                .unwrap();
            let mut old: Option<(Vec<u8>, Vec<u8>)> = None;
            let mut g = Game::new(assets);
            let mut checked = 0;
            while let Some(frame) = reader.next() {
                let ram = reader.region("ram").unwrap().data.clone();
                let gfx = reader.region("gfx").unwrap().data.clone();
                if let Some((prev, video)) = &old {
                    g.trace_load(prev, video);
                    pokes.apply(&mut g, frame, false).unwrap();
                    let input = script.at(frame as u32);
                    g.w.input = input.players;
                    g.w.system = input.system | 8;
                    g.w.coins = input.coins;
                    g.tick();
                    pokes.apply(&mut g, frame, true).unwrap();
                    for i in 0..g.enemies.slots.len() {
                        let h = g.enemies.handle(i);
                        let a = h as usize;
                        if u16::from_be_bytes([ram[a + 0x20], ram[a + 0x21]]) != 29 * 4 {
                            continue;
                        }
                        let mut e = g.enemies.slots[i].clone();
                        let mut image = Collect::default();
                        e.trace_visit(&mut F::new(&mut image, 0xFF0000 | h as u32, assets));
                        for off in 0..0xE0 {
                            if image.mask[a + off] != 0 {
                                assert_eq!(
                                    image.ram[a + off],
                                    ram[a + off],
                                    "{name} frame {frame} Hopper +{off:02X}"
                                );
                            }
                        }
                        checked += 1;
                    }
                }
                old = Some((ram, gfx));
            }
            assert!(checked > 0, "{name}: no Hopper frames");
            eprintln!("{name}: {checked} Hopper object-frames equal MAME");
            total += checked;
        }
        assert!(total > 0, "no Hopper AI recordings found");
    }

    #[test]
    fn hopper_countdown_underflows_instead_of_testing_zero_after_subtraction() {
        let mut e = Enemy::default();
        e.m.sb(0x80, 1);
        assert!(!Game::enemy_hopper_count(&mut e, 0x80));
        assert_eq!(e.m.b(0x80), 0);
        assert!(Game::enemy_hopper_count(&mut e, 0x80));
        assert_eq!(e.m.b(0x80), 255);
    }
}
