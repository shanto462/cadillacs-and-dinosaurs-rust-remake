//! Tyrog's host, pool A class 72 (handler 0x5DD58..0x5E4F4, table 0x702C4): the man whose body the creature takes between two forms. Class 4's
//! defeat of forms 2 and 1 rewrites its slot into this class with the variant one lower (0x38D9E). The host has a 30 frame intro (not hittable,
//! a palette flicker, then a puff, class 75 variant 5 or 0x12), punches (attack 0x3F), jumps to the edge of the screen (attack 0x34, a landing
//! mark class 75 variant 0x10 / 0x11), lands with the caption 0x5A and sound 0x79, and transforms (animation 0x22 or 0x23) back into class 4 with
//! the same variant. HP index 0 (450 at level 10), poise 1; killed it ends the stage (0xACEE) and scores 100,000. D3d 3.6, 3.7, BT11, BT12.
//! Owner: enemy port.

use super::{off::*, Enemy};
use crate::game::{slot_of, Game, Handle, Slot};

/// Own bytes (the original offsets): `+$A0` the phase flag (1), `+$A1` busy, `+$A4` (a stop: the AI returns at once), `+$A5` the X class,
/// `+$AC` word the state timer, `+$B0` / `+$B4` longs the jump target, `+$BE` byte the knock downs, `+$C4` long the landing mark, `+$C8` byte the
/// intro runs, `+$80` word the blink time of the death.
mod o {
    pub const BUSY: usize = 0xA1;
    pub const STOP: usize = 0xA4;
    pub const XCLASS: usize = 0xA5;
    pub const TIMER: usize = 0xAC;
    pub const TX: usize = 0xB0;
    pub const TZ: usize = 0xB4;
    pub const DOWNS: usize = 0xBE;
    pub const MARK: usize = 0xC4;
    pub const INTRO: usize = 0xC8;
    pub const BLINK: usize = 0x80;
}

/// G.-$704A, the palette of the flicker: only class 72 variant 2 writes it (0x5DDA4), which never exists, so it holds the 0 of the RAM clear.
const SAVED_PALETTE: u8 = 0;

impl Game {
    /// 0x5DD58 (mode table 0x5DD66): 0 the init 0x5DD6E, 2 the run 0x5DE4E, 4 the death 0x5E470, 6 the body 0x5E4BC (drawn only).
    pub(super) fn enemy_class72(&mut self, e: &mut Enemy, h: Handle, _i: usize) {
        match e.b.mode {
            0 => self.c72_init(e, h),
            2 => self.c72_run(e, h),
            4 => self.c72_dying(e, h),
            _ => self.enemy_draw_check(&mut e.b, h), // 0x5E4BC
        }
    }

    /// 0x5DD6E: mode 2; name 0x1B, `+$84` 0x6C, the table 0x702C4, `+$63` 2; animation 8 and the flicker (0x5E142); the box table 0x103C00;
    /// speed id 7, direction 0x18; HP index 0 of 0x31E14; the target; poise 1; `+$7D` 0xFE; `+$72` 0x1E; `+$AC` 0x14; `+$A0` 1; the intro
    /// (`+$C8` 1); the HUD bar; G.$4FC 0; the floor (0x3140C); a slot (0x32C5E); animation 0x25 (variant 1) or 0x24.
    fn c72_init(&mut self, e: &mut Enemy, h: Handle) {
        e.b.mode = 2;
        e.b.live = 1;
        e.m.sb(B87, 0);
        e.b.id = 0x1B;
        e.m.sw(W84, 0x6C);
        e.b.anim_table = 0x702C4;
        e.b.b63 = 2;
        self.enemy_anim(&mut e.b, 8);
        self.c72_flicker(e);
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_set_speed(&mut e.b, 7);
        e.b.dir = 0x18;
        self.enemy_set_boss_hp(&mut e.b, 0); // 0x31E14 row 0
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        e.b.stun = 1;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.m.sb(o::STOP, 0);
        e.m.sb(o::DOWNS, 0);
        e.m.sb(ATTACKING, 0xFE);
        e.b.b72 = 0x1E;
        e.m.sw(o::TIMER, 0x14);
        e.m.sb(WAIT, 0);
        e.m.sb(0xA0, 1);
        e.m.sb(o::INTRO, 1);
        self.hud.boss = h; // G.-$70BC
        self.spawn.g.b4fc = 0;
        self.enemy_snap_to_floor(e);
        e.m.sb(ATTACKING, 0xFE);
        self.lib_slot6_new(e); // 0x32C5E
        let anim = if (e.b.var26 >> 8) as u8 != 0 { 0x25 } else { 0x24 };
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x5E142: the palette by variant (0 0x9F, 1 0x9E), the saved palette on frames with `+$2D` set.
    fn c72_flicker(&self, e: &mut Enemy) {
        let form = (e.b.var26 >> 8) as u8;
        let base = match form {
            0 => 0x9F,
            1 => 0x9E,
            _ => return,
        };
        e.b.palette_override = if e.b.param_b != 0 { SAVED_PALETTE } else { base };
    }

    /// 0x5DE4E: the flicker; the intro while `+$C8` is set (0x5DF46); else (an act past 0xE in the death sub is reset) by `+$05` (the long table
    /// 0x5DECA: 0 the AI 0x5DEDA, 2 the hit 0x5E448, 4 the death 0x5E470, 6 held 0x2B35A); outside held X is kept in camera X - 0x20 .. + 0x1B0.
    fn c72_run(&mut self, e: &mut Enemy, h: Handle) {
        self.c72_flicker(e);
        if e.m.b(o::INTRO) != 0 {
            return self.c72_intro(e, h);
        }
        if e.b.sub == 4 && (e.b.act as i8) > 0xE {
            e.b.act = 0;
        }
        match e.b.sub {
            0 => self.c72_ai(e, h),
            2 => self.c72_hit(e, h),
            4 => self.c72_dying(e, h),
            _ => self.enemy_grabbed(e, h),
        }
        if e.b.sub != 6 {
            let cam = self.cam.s2.cam_x as i16;
            let put = |b: &mut crate::game::body::Body, x: i16| b.x = (b.x & 0xFFFF) | ((x as i32) << 16);
            if e.b.xi().wrapping_sub(cam.wrapping_sub(0x20)) < 0 {
                put(&mut e.b, cam.wrapping_sub(0x20));
            }
            if cam.wrapping_add(0x1B0).wrapping_sub(e.b.xi()) < 0 {
                put(&mut e.b, cam.wrapping_add(0x1B0));
            }
        }
    }

    /// 0x5DF46: the intro animation with the flicker of 0x9E / 0x9D (variant 0 / 1) and the draw check (not hittable); at its end 0x5DF94: the
    /// puff (class 75 variant 5, 0x12 for variant 0) and the fight start 0x5DFDE.
    fn c72_intro(&mut self, e: &mut Enemy, h: Handle) {
        if !self.enemy_step(&mut e.b) {
            let form = (e.b.var26 >> 8) as u8;
            if form == 0 {
                e.b.palette_override = if e.b.param_b != 0 { SAVED_PALETTE } else { 0x9E };
            }
            if form == 1 {
                e.b.palette_override = if e.b.param_b != 0 { SAVED_PALETTE } else { 0x9D };
            }
            return self.enemy_draw_check(&mut e.b, h);
        }
        e.m.sb(o::INTRO, 0);
        let var = if (e.b.var26 >> 8) as u8 == 0 { 0x12 } else { 5 };
        if let Some(j) = self.c75_spawn(75 * 4, var, e.b.xi(), e.b.yi(), e.b.zi(), None) {
            self.enemies.slots[j].b.dir = 0;
        }
        self.c72_recover(e);
    }

    /// 0x5DFDE: the fight start, also the recover entry of `+$84` 0x6C (table 0x2B56C): state 0x02000000, animation 8, speed id 5, the target,
    /// face it; a knock down counts in `+$BE`; the flicker; `+$7D` 0xFE; a slot, aim (0x32A70); `+$AC` 0x14; then 0x5E41E: `+$AC` 0x3C, speed id 5,
    /// the X class (0x3293C) and the punch (0x5E36E).
    pub(super) fn c72_recover(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0000);
        self.enemy_anim(&mut e.b, 8);
        self.enemy_set_speed(&mut e.b, 5);
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        self.lib_face_target(e);
        if e.b.reaction == 4 {
            e.m.sb(o::DOWNS, e.m.b(o::DOWNS).wrapping_add(1));
        }
        self.c72_flicker(e);
        e.m.sb(ATTACKING, 0xFE);
        self.lib_slot6_new(e);
        self.lib_aim_at_target(e);
        e.m.sw(o::TIMER, 0x14);
        e.m.sb(o::BUSY, 0);
        // 0x5E41E
        e.m.sb(o::BUSY, 0);
        e.m.sw(o::TIMER, 0x3C);
        self.enemy_set_speed(&mut e.b, 5);
        e.m.sb(o::BUSY, 0);
        let (xc, _) = self.lib_distance_class(e);
        e.m.sb(o::XCLASS, xc as u8);
        self.c72_punch(e);
    }

    /// 0x5E36E: the punch (state 4): face, attack 0x3F, busy, `+$AC` 0x3C, animation 9 (0x20 after a knock down).
    fn c72_punch(&mut self, e: &mut Enemy) {
        self.lib_face_target(e);
        self.enemy_begin_attack(&mut e.b, 0x3F);
        e.m.sb(o::BUSY, 1);
        e.b.act = 4;
        e.m.sw(o::TIMER, 0x3C);
        let anim = if (e.m.b(o::DOWNS) as i8) < 1 { 9 } else { 0x20 };
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x5DEDA: (nothing while `+$A4` is set) the poise refill (1), the terrain; the hit stop (the animation steps when it ends); else a slot point
    /// (0x32C54) and the state (table 0x5DF38: 2 and 4 the punch 0x5E04E, 6 the jump 0x5E03A, 8 the change 0x5E05A, 0xA nothing); the target
    /// check, the draw check, the hurtable list.
    fn c72_ai(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.b(o::STOP) != 0 {
            return;
        }
        let t = e.m.b(POISE_T);
        if t != 0 {
            e.m.sb(POISE_T, t - 1);
            if t == 1 {
                e.b.stun = 1;
            }
        }
        if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
            e.m.sw(OBSTACLE, ob);
        }
        if e.b.hitstop != 0 {
            e.b.hitstop -= 1;
            e.b.timer = 1;
        } else {
            self.lib_slot6_keep(e);
            match e.b.act {
                2 | 4 => {
                    // 0x5E04E
                    if self.enemy_step(&mut e.b) {
                        self.c72_jump(e, h);
                    }
                }
                6 => {
                    // 0x5E03A
                    e.b.motion();
                    if e.b.airborne == 0 {
                        self.c72_land(e);
                    } else {
                        self.enemy_step(&mut e.b);
                    }
                }
                8 => {
                    // 0x5E05A: at the end of the transformation the slot becomes class 4 again (the same variant, facing and place).
                    if self.enemy_step(&mut e.b) {
                        e.b.set_state(0);
                        e.b.live = 1;
                        e.b.k20 = 4 * 4;
                    } else {
                        self.c72_flicker(e);
                    }
                }
                _ => {} // 0xA (0x5DF44, rts); 0 is never set while the AI runs
            }
        }
        self.lib_check_target(e); // 0x32F56
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x5E1C6: the jump (state 6): busy, `+$AC` 0x3C, in the air, attack 0x34, aim; the target point (0x5E3B8: camera X + 0x30 when the host is
    /// left of camera X + 0xC0, else + 0x150, facing that way; camera Z + 0x50) and the landing mark (0x5E0B0); the speeds = the distances / 0x3800
    /// (counted), vy 0x700, gravity -0x40, toward the point; attack 0x34 again, animation 0xA.
    fn c72_jump(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sb(o::BUSY, 1);
        e.m.sw(o::TIMER, 0x3C);
        e.b.act = 6;
        e.b.airborne = 1;
        self.enemy_begin_attack(&mut e.b, 0x34);
        self.lib_aim_at_target(e);
        if (e.m.b(o::XCLASS) as i8) <= 5 {
            e.b.dir &= 0x10; // (the second test, class 2 or less, is inside the first)
        }
        // 0x5E3B8
        let cam_x = ((self.cam.s2.cam_x as u32) << 16 | self.cam.s2.cam_x_frac as u32) as i32;
        let cam_z = ((self.cam.s2.cam_z as u32) << 16 | self.cam.s2.cam_z_frac as u32) as i32;
        let (tx, facing) = if (self.cam.s2.cam_x as i16).wrapping_add(0xC0) > e.b.xi() {
            (cam_x.wrapping_add(0x30_0000), 1)
        } else {
            (cam_x.wrapping_add(0x150_0000), 0)
        };
        e.b.facing = facing;
        let tz = cam_z.wrapping_add(0x50_0000);
        e.m.sl(o::TX, tx as u32);
        e.m.sl(o::TZ, tz as u32);
        self.c72_mark(e, h);
        let steps = |d: i32| -> i16 {
            let mut d0 = if d < 0 { d.wrapping_neg() } else { d };
            let mut n: u16 = 0;
            loop {
                n = n.wrapping_add(1);
                d0 = d0.wrapping_sub(0x3800);
                if d0 < 0 {
                    return n as i16;
                }
            }
        };
        e.b.vx = steps(e.b.x.wrapping_sub(tx));
        e.b.vz = steps(e.b.z.wrapping_sub(tz));
        e.b.vy = 0x700;
        e.b.ax = 0;
        e.b.ay = -0x40;
        e.b.az = 0;
        if e.b.x >= tx {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        if e.b.z >= tz {
            e.b.vz = e.b.vz.wrapping_neg();
        }
        self.enemy_begin_attack(&mut e.b, 0x34);
        self.enemy_anim(&mut e.b, 0xA);
    }

    /// 0x5E0B0: the landing mark (class 75 variant 8, 0x10 for variant 1, 0x11 for variant 0) at camera X - 0x10 (+ 0x1A0 when the host faces left),
    /// Y 0, Z 0x170, direction 8 (0x18 from the right), the host's facing; `+$BC` = the host, the host's `+$C4` = the mark.
    fn c72_mark(&mut self, e: &mut Enemy, h: Handle) {
        let Some(j) = self.enemies.alloc() else { return };
        let host = 0xFFFF_0000 | h as u32;
        let mh = self.enemies.handle(j);
        let form = (e.b.var26 >> 8) as u8;
        let cam = self.cam.s2.cam_x as i16;
        let n = &mut self.enemies.slots[j];
        n.b.live = 1;
        n.b.k20 = 75 * 4;
        n.b.var26 = 8;
        n.m.sl(0xBC, host);
        let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
        let mut x = cam.wrapping_sub(0x10);
        let mut f = e.b.facing.wrapping_add(1) & 1;
        n.b.dir = 8;
        if e.b.facing == 0 {
            x = x.wrapping_add(0x1A0);
            n.b.dir = 0x18;
        }
        put(&mut n.b.x, x);
        put(&mut n.b.y, 0);
        put(&mut n.b.z, 0x170);
        f = f.wrapping_add(1) & 1;
        n.b.facing = f;
        if form == 0 {
            n.b.var26 = 0x11;
        }
        if form == 1 {
            n.b.var26 = 0x10;
        }
        e.m.sl(o::MARK, 0xFFFF_0000 | mh as u32);
    }

    /// 0x5E2F8: the landing: the place of the mark; the caption 0x5A (variants 0 and 1), the sound 0x79; busy, state 8, `+$AC` 0x3C; the
    /// transformation animation 0x23 (variant 0) or 0x22 (variant 1).
    fn c72_land(&mut self, e: &mut Enemy) {
        if let Some(Slot::Enemy(k)) = slot_of(e.m.l(o::MARK) as u16) {
            let (mx, my) = (self.enemies.slots[k].b.xi(), self.enemies.slots[k].b.yi());
            e.b.x = (e.b.x & 0xFFFF) | ((mx as i32) << 16);
            e.b.y = (e.b.y & 0xFFFF) | ((my as i32) << 16);
        }
        let form = (e.b.var26 >> 8) as u8;
        if form <= 1 {
            self.prop_caption_spawn(0x5A); // 0x86D2E
        }
        self.sound(0x79); // 0xA63C, d1 = d2 = 0
        e.m.sb(o::BUSY, 1);
        e.b.act = 8;
        e.m.sw(o::TIMER, 0x3C);
        let anim = match form {
            0 => 0x23,
            1 => 0x22,
            _ => 0x1A,
        };
        self.enemy_anim(&mut e.b, anim);
    }

    /// 0x5E448: the hit: the credit (0x3262A); at HP 0 while `+$6E` is set: `+$7D` 0xFF, the stage end (0xACEE), `+$6E` 0; then 0x2A606.
    fn c72_hit(&mut self, e: &mut Enemy, h: Handle) {
        self.boss_credit(e);
        if e.b.hp == 0 && e.b.hp_prev != 0 {
            e.m.sb(ATTACKING, 0xFF);
            self.flow_boss_defeated(h); // 0xACEE
            e.b.hp_prev = 0;
        }
        self.enemy_hit_reaction(e, h);
    }

    /// 0x5E470: the death (table 0x5E47C): step 0 normal speed (G.$4D6 = 1), 120 frames, the score (0x325E6, 100,000); step 2 the count, drawn
    /// on odd counts; past 0 the leader flags (0x6148) and mode 6.
    fn c72_dying(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.sub == 0 {
            self.flow.speed = 1;
            e.m.sw(o::BLINK, 0x78);
            let b = e.b.clone();
            self.enemy_award_kill_score(&b);
            e.b.sub = e.b.sub.wrapping_add(2);
        }
        let t = e.m.w(o::BLINK);
        if t == 0 {
            self.enemy_boss_leader(e.b.who);
            e.b.mode = 6;
            return;
        }
        e.m.sw(o::BLINK, t - 1);
        if (t - 1) & 1 != 0 {
            self.enemy_draw_check(&mut e.b, h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game() -> Game {
        static A: std::sync::OnceLock<AssetPack> = std::sync::OnceLock::new();
        Game::new(A.get_or_init(|| {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dino.zip");
            dino_extract::extract(&path).expect("Tyrog checks require the user's dino.zip")
        }))
    }

    /// D3d 3.3, BT-A8 / BT-A9: at level 10 the forms have 256 / 288 / 352 HP and the host 450 / 600 / 650 by player count; Tyrog 2 has 300 at
    /// every level and player count.
    #[test]
    fn hp_by_player_count() {
        for (players, form_hp, host_hp) in [(0u8, 256u16, 450u16), (1, 288, 600), (2, 352, 650)] {
            let mut g = game();
            g.w.level = 10;
            g.w.players_minus_one = players;
            for var in [0u16, 0x100] {
                let mut e = Enemy::default();
                e.b.k20 = 4 * 4;
                e.b.var26 = var;
                g.enemy_class4(&mut e, 0, 0);
                assert_eq!(e.b.hp, form_hp, "class 4 var {var:#x}, {} players", players + 1);
                let mut e = Enemy::default();
                e.b.k20 = 72 * 4;
                e.b.var26 = var;
                g.enemy_class72(&mut e, 0, 0);
                assert_eq!(e.b.hp, host_hp, "class 72 var {var:#x}, {} players", players + 1);
            }
            for level in [0u16, 10, 31] {
                g.w.level = level;
                let mut e = Enemy::default();
                e.b.k20 = 91 * 4;
                g.enemy_class91(&mut e, 0, 0);
                assert_eq!(e.b.hp, 300, "class 91 at level {level}");
                assert_eq!(e.m.b(ATTACKING), 0, "Tyrog 2 is counted by the census");
            }
        }
    }

    /// The kill scores of 0x325E6 by name: Tyrog 0x19 60,000, the host 0x1B 100,000, Tyrog 2 0x23 1,000 (D3d 3.7, 3.8).
    #[test]
    fn kill_scores() {
        let g = game();
        let t = &g.assets.enemy.t;
        let score = |id: u32| t.u32(0x10292, t.u8(0x325F8, id) as u32);
        assert_eq!([score(0x19), score(0x1B), score(0x23)], [0x60000, 0x100000, 0x1000]);
    }
}
