//! Pool A class 22 (handler 0x47C42..0x4825E), the brontosaurus foot of Episode 6 area 1 (D3b section 6), and class 23 (0x48266..0x48376), its leg.
//! The foot cannot be hit (`+$00` stays 2, `+$7D` 0xFF); it rises, hovers over the first hero in its home column, waits, slams (attack 0x4C, 30 base damage with
//! a knock down, the stomp sound 0xA288 and the screen shake 4) and rests. The leg follows the foot (X, Z + 1, Y 0x1E), is drawn on the frames the mask
//! `+$A8` allows and casts the shadow oval. Owner: enemy port (M8-01).
//!
//! The foot's own bytes (the original offsets): `+$A0` word the wait timer of the act, `+$A2` / `+$A6` longs the velocity of the hover in X and Z (the wander
//! writes both speeds to `+$A2`, as the original does), `+$AA` / `+$AC` words the home X and Z, `+$AE` word the wander timer, `+$B0` word the turn timer,
//! `+$B4` byte the act saved by the hurt state. The leg's: `+$A0` word the animation (0), `+$A2` long the foot, `+$A6` word a frame counter, `+$A8` word the
//! mask of the frames it is drawn on.
//! Tables (read from the ROM through dino-extract): 0x47EAC and 0x47EBC the wander speed choices (16 bytes each); the animation tables 0x718EE (foot) and 0x4837C
//! (leg); the box table 0x103C00.

use super::{off::*, Enemy};
use crate::game::{Game, Handle};

/// The foot's own bytes.
mod o {
    pub const TIMER: usize = 0xA0;
    pub const VX: usize = 0xA2;
    pub const VZ: usize = 0xA6;
    pub const HOME_X: usize = 0xAA;
    pub const HOME_Z: usize = 0xAC;
    pub const WANDER_T: usize = 0xAE;
    pub const TURN_T: usize = 0xB0;
    pub const SAVED_ACT: usize = 0xB4;
}

/// The wander speed choice tables (indexed by a random number & 15) and the box table.
const WANDER_X: u32 = 0x47EAC;
const WANDER_Z: u32 = 0x47EBC;
const BOX_TABLE: u32 = 0x103C00;

impl Game {
    /// 0x47C42 (mode table 0x47C4E): 0 the init 0x47C56 (falls into 2), 2 the run 0x47D32, 4 and 6 the free 0x48260 (jmp 0x48FC).
    pub(super) fn enemy_class22(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.c22_init(e, h);
                self.c22_run(e, h, i);
            }
            2 => self.c22_run(e, h, i),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x12CB4 of an object of this pool with the obstacle stored at +$94.
    fn c22_terrain(&mut self, e: &mut Enemy, h: Handle) {
        if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
            e.m.sw(OBSTACLE, ob);
        }
    }

    /// The attack serial of a new swing: `+$5E` takes G.$67BE and G.$67BE counts up.
    fn c22_new_serial(&mut self, e: &mut Enemy) {
        e.b.serial = self.w.attack_serial;
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
    }

    /// 0x47C56: the box table, attack 0x63, the foot is not counted and cannot be hit, the table 0x718EE, no shadow, the terrain, the home column, the leg (class 23)
    /// at the foot's place, HP 100.
    fn c22_init(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sb(B87, 0);
        e.b.hitstop = 0;
        e.b.box_table = BOX_TABLE;
        e.b.multi_box = false;
        e.b.atk = 0x63;
        e.b.knock = 0;
        e.b.hitstop = 0;
        self.c22_new_serial(e);
        e.b.live = 2;
        e.m.sb(ATTACKING, 0xFF);
        e.b.b63 = 0xFF;
        e.b.anim_table = 0x718EE;
        e.b.draw_flags = 0x10;
        self.c22_terrain(e, h);
        e.b.mode = e.b.mode.wrapping_add(2);
        e.b.airborne = 1;
        e.b.set_yi_keep(0);
        e.m.sw(o::HOME_X, e.b.xi() as u16);
        e.m.sw(o::HOME_Z, e.b.zi() as u16);
        e.b.vx = 0;
        e.b.ax = 0;
        e.b.vz = 0;
        e.b.az = 0;
        self.enemy_anim(&mut e.b, 0);
        // 0x483C: the leg, at the foot's place (the foot's direction, the foot as its owner)
        if let Some(j) = self.enemies.alloc() {
            let (x, y, z, dir) = (e.b.xi(), e.b.yi(), e.b.zi(), e.b.dir);
            let c = &mut self.enemies.slots[j];
            c.b.live = 1;
            c.b.k20 = 23 * 4;
            c.b.set_xi_keep(x);
            c.b.set_yi_keep(y);
            c.b.set_zi_keep(z);
            c.b.var26 = 0;
            c.b.dir = dir;
            c.m.sl(o::VX, h as i16 as i32 as u32); // a6 is sign extended (0xFFFFDB54)
        }
        e.b.hp_bar = 0x64;
        e.b.hp = 0x64;
        e.b.hp_prev = 0x64;
    }

    /// 0x47D32: the sub state (table 0x47D40: 0 the acts at 0x47D50, 2 to 6 the hurt at 0x48206), then the clamp to the home area, the picture by height, the terrain, the
    /// far test, the animation step, the draw check and the hostile list (0x4C0A).
    fn c22_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.sub != 0 {
            return self.c22_hurt(e);
        }
        match e.b.act {
            0 => self.c22_start_rise(e), // 0x47DB2
            2 => self.c22_rise(e),
            4 => self.c22_lower(e),
            6 => self.c22_hover(e),
            8 => {
                // 0x47ECC: wait, then the wind up
                let t = e.m.w(o::TIMER).wrapping_sub(1);
                e.m.sw(o::TIMER, t);
                if t == 0 {
                    self.c22_start_windup(e);
                }
            }
            0xA => self.c22_windup(e),
            0xC => self.c22_fall(e),
            0xE => {
                // 0x47F08: rest, then the next rise
                let t = e.m.w(o::TIMER).wrapping_sub(1);
                e.m.sw(o::TIMER, t);
                if t == 0 {
                    self.c22_start_rise(e);
                }
            }
            0x10 => self.c22_pushed(e),
            _ => {}
        }
        self.c22_clamp(e);
        self.c22_picture(e);
        self.c22_terrain(e, h);
        self.enemy_despawn_if_far(e, i); // 0x173C
        self.enemy_step(&mut e.b); // 0x121E
        self.enemy_draw_check(&mut e.b, h); // 0x49CA
        self.enemy_monster_register(h); // 0x4C0A
    }

    /// 0x4809A: act 2, the rise: airborne, vy 0, ay 0x50, no speed in X or Z.
    fn c22_start_rise(&mut self, e: &mut Enemy) {
        e.b.act = 2;
        e.b.airborne = 1;
        e.b.vy = 0;
        e.b.ay = 0x50;
        e.b.vx = 0;
        e.b.ax = 0;
        e.b.vz = 0;
        e.b.az = 0;
    }

    /// 0x47DB6: act 2, the rise: the move (0x189C), vy at most 0x800, act 4 from Y 0x100.
    fn c22_rise(&mut self, e: &mut Enemy) {
        e.b.motion();
        if e.b.vy > 0x800 {
            e.b.vy = 0x800;
        }
        if e.b.yi() >= 0x100 {
            e.b.act = 4; // 0x480CC
        }
    }

    /// 0x47DD8: act 4, the lowering: 2 pixels per frame down to Y 0x90, then the hover (0x480D4: act 6, speed 7, 60 frames of wander timer, turn timer 6, 180 frames).
    fn c22_lower(&mut self, e: &mut Enemy) {
        e.b.hitstop = 0;
        e.b.set_yi_keep(e.b.yi().wrapping_sub(2));
        if e.b.yi() <= 0x90 {
            e.b.act = 6;
            self.enemy_set_speed(&mut e.b, 7); // 0x17A4
            e.b.vy = 0;
            e.b.ay = 0;
            e.m.sw(o::WANDER_T, 0x3C);
            e.m.sw(o::TURN_T, 6);
            e.m.sw(o::TIMER, 0xB4);
        }
    }

    /// 0x47DF0: act 6, the hover: the timer (0x48102 at 0: act 8, 60 frames), the wander timer, the move over the first hero (0x47F36).
    fn c22_hover(&mut self, e: &mut Enemy) {
        let t = e.m.w(o::TIMER).wrapping_sub(1);
        e.m.sw(o::TIMER, t);
        if t == 0 {
            e.b.act = 8;
            e.m.sw(o::TIMER, 0x3C);
            return;
        }
        e.m.sw(o::WANDER_T, e.m.w(o::WANDER_T).wrapping_sub(1));
        self.c22_track(e);
    }

    /// 0x48110: act 0xA, the wind up: rise with ay 0x80.
    fn c22_start_windup(&mut self, e: &mut Enemy) {
        e.b.airborne = 1;
        e.b.act = 0xA;
        e.b.vy = 0;
        e.b.ay = 0x80;
        e.b.vx = 0;
        e.b.ax = 0;
        e.b.vz = 0;
        e.b.az = 0;
    }

    /// 0x47ED6: act 0xA: the move; from Y 0x100 the slam starts (0x48142: attack 0x4C, a new serial, act 0xC, vy -0x400, ay -0x80).
    fn c22_windup(&mut self, e: &mut Enemy) {
        e.b.motion();
        if e.b.yi() >= 0x100 {
            e.b.atk = 0x4C;
            e.b.knock = 0;
            e.b.hitstop = 0;
            self.c22_new_serial(e);
            e.b.act = 0xC;
            e.b.vy = -0x400;
            e.b.ay = -0x80;
            e.b.vx = 0;
            e.b.ax = 0;
            e.b.vz = 0;
            e.b.az = 0;
        }
    }

    /// 0x47EE8: act 0xC, the fall: the move; on the floor the stomp (0xA288: sound 0x76 with the pan of the foot), the screen shake 4 and the rest (0x48186).
    fn c22_fall(&mut self, e: &mut Enemy) {
        e.b.motion();
        if e.b.airborne != 0 {
            return;
        }
        let pan = self.snd_pan_x(e.b.xi()) as u32;
        self.snd_request(0x76, pan, 0); // 0xA288
        self.cam_shake(4); // G.$755C
        e.b.act = 0xE;
        e.b.hitstop = 0;
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.m.sb(ATTACKING, 0xFF);
        e.b.timer = 1;
        e.b.hp_bar = 0x64;
        e.b.hp = 0x64;
        e.b.hp_prev = 0x64;
        e.b.atk = 0x63;
        e.b.knock = 0;
        self.c22_new_serial(e);
        self.rand(); // 0x481D4: the result is not used
        e.m.sw(o::TIMER, 0x1E);
    }

    /// 0x47F12: act 0x10 (after the hurt state): the timer or the floor gives the act back (0x47F2A: the saved act, `+$7A` 0).
    fn c22_pushed(&mut self, e: &mut Enemy) {
        let t = e.m.w(o::TIMER).wrapping_sub(1);
        e.m.sw(o::TIMER, t);
        if t != 0 {
            e.b.motion();
            if e.b.airborne != 0 {
                return;
            }
        }
        e.b.act = e.m.b(o::SAVED_ACT);
        e.m.sb(WAIT, 0);
    }

    /// 0x48206: a sub state other than 0 (a hit reaction asked by another routine): the act is saved, act 0x10 for 16 frames with a small hop, the sound 0x36. The frame
    /// ends here (no clamp, picture, terrain or draw).
    fn c22_hurt(&mut self, e: &mut Enemy) {
        e.m.sb(o::SAVED_ACT, e.b.act);
        e.b.hitstop = 0;
        e.b.sub = 0;
        e.b.act = 0x10;
        e.m.sw(o::TIMER, 0x10);
        e.b.vy = -0x100;
        e.b.ay = 0x20;
        e.b.vx = 0;
        e.b.ax = 0;
        e.b.vz = 0;
        e.b.az = 0;
        e.b.airborne = 1;
        self.snd_request(0x36, 0x0007_0010, 0x0200_0220);
    }

    /// 0x47F36: the hover moves over the first hero that is in the game (G.$3274, G.$33F4, G.$3574); with none, or one outside the home column (|X - home| above 0x50), the
    /// wander (0x47E02).
    fn c22_track(&mut self, e: &mut Enemy) {
        let Some(k) = (0..3).find(|&k| self.players[k].b.live != 0) else {
            return self.c22_wander(e);
        };
        let (hx, hz) = (self.players[k].b.xi(), self.players[k].b.zi());
        let home = e.m.w(o::HOME_X) as i16;
        if (home.wrapping_add(0x50).wrapping_sub(hx) as u16) > 0xA0 {
            return self.c22_wander(e);
        }
        // X: 0 within 8 .. 0x18 left of the hero, else 2 pixels per frame toward him
        if (e.b.xi().wrapping_add(8).wrapping_sub(hx) as u16) <= 0x10 {
            e.m.sl(o::VX, 0);
        } else if hx >= e.b.xi() {
            e.m.sl(o::VX, 0x20000);
        } else {
            e.m.sl(o::VX, 0xFFFE_0000);
        }
        // Z: the same, 1.5 pixels per frame
        if (e.b.zi().wrapping_add(8).wrapping_sub(hz) as u16) <= 0x10 {
            e.m.sl(o::VZ, 0);
        } else if hz >= e.b.zi() {
            e.m.sl(o::VZ, 0x18000);
        } else {
            e.m.sl(o::VZ, 0xFFFE_8000);
        }
        self.c22_move(e);
    }

    /// 0x47E02: no hero in the column: every second frame the direction turns (`+$22` + 1 & 0x1F) and the foot moves along it (0x1862); every 60 frames two random numbers
    /// choose a new speed (both go to `+$A2`, the second wins: the Z speed `+$A6` stays as it was).
    fn c22_wander(&mut self, e: &mut Enemy) {
        let t = e.m.w(o::TURN_T).wrapping_sub(1);
        e.m.sw(o::TURN_T, t);
        if t == 0 {
            e.m.sw(o::TURN_T, 2);
            e.b.dir = e.b.dir.wrapping_add(1) & 0x1F;
            self.enemy_move_dir(&mut e.b); // 0x1862
        }
        if e.m.w(o::WANDER_T) == 0 {
            e.m.sw(o::WANDER_T, 0x3C);
            let d = (self.rand() & 0xF) as u32;
            let first = self.assets.enemy.t.u8(WANDER_X, d) as i8;
            e.m.sl(o::VX, if first == 0 { 0 } else if first < 0 { 0xFFFE_0000 } else { 0x20000 });
            let d = (self.rand() & 0xF) as u32;
            let second = self.assets.enemy.t.u8(WANDER_Z, d) as i8;
            e.m.sl(o::VX, if second == 0 { 0 } else if second < 0 { 0xFFFE_8000 } else { 0x18000 });
        }
        self.c22_move(e);
    }

    /// 0x47E9A: X += `+$A2`, Z += `+$A6` (16.16).
    fn c22_move(&mut self, e: &mut Enemy) {
        e.b.x = e.b.x.wrapping_add(e.m.l(o::VX) as i32);
        e.b.z = e.b.z.wrapping_add(e.m.l(o::VZ) as i32);
    }

    /// 0x47FF6: the foot stays within 0x50 pixels of its home in X and 0x20 in Z (words, the fraction stays).
    fn c22_clamp(&mut self, e: &mut Enemy) {
        let (hx, hz) = (e.m.w(o::HOME_X) as i16, e.m.w(o::HOME_Z) as i16);
        if hx.wrapping_add(0x50) < e.b.xi() {
            e.b.set_xi_keep(hx.wrapping_add(0x50));
        }
        if hx.wrapping_sub(0x50) > e.b.xi() {
            e.b.set_xi_keep(hx.wrapping_sub(0x50));
        }
        if hz.wrapping_add(0x20) < e.b.zi() {
            e.b.set_zi_keep(hz.wrapping_add(0x20));
        }
        if hz.wrapping_sub(0x20) > e.b.zi() {
            e.b.set_zi_keep(hz.wrapping_sub(0x20));
        }
    }

    /// 0x48048: the picture by the height: act 2 and 0xE the flat foot (0), up to Y 0x30 the same, up to 0x60 the raised foot (1), above the high one (2); then 0x120E.
    fn c22_picture(&mut self, e: &mut Enemy) {
        let id = if e.b.act == 2 || e.b.act == 0xE {
            0
        } else if e.b.yi() <= 0x30 {
            0
        } else if e.b.yi() <= 0x60 {
            1
        } else {
            2
        };
        self.enemy_anim(&mut e.b, id);
    }

    /// 0x48266 (mode table 0x48272): 0 the init 0x4827A (falls into 2), 2 the run 0x482B0, 4 and 6 the free 0x48376 (jmp 0x48FC).
    pub(super) fn enemy_class23(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                // 0x4827A: not counted, cannot be hit, the table 0x4837C, the draw mask 3, the terrain, animation 0
                e.b.live = 2;
                e.b.anim_table = 0x4837C;
                e.m.sb(ATTACKING, 0xFF);
                e.b.b63 = 0xFF;
                e.m.sw(0xA0, 0);
                e.m.sw(0xA8, 3);
                self.c22_terrain(e, h);
                self.enemy_anim(&mut e.b, 0);
                e.b.mode = e.b.mode.wrapping_add(2);
                self.c23_run(e, h, i);
            }
            2 => self.c23_run(e, h, i),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x482B0: the viewer input (0x48340), the place under the foot (0x482DC), the far test (0x173C), the mask by the foot's height (0x4830C), the frame counter `+$A6`;
    /// the draw check (0x49CA) only on the frames where the counter and the mask have no bit in common. The leg is not in the hostile list.
    fn c23_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        self.c23_input(e);
        self.c23_follow(e);
        self.enemy_despawn_if_far(e, i);
        self.c23_mask(e);
        let n = e.m.w(0xA6).wrapping_add(1);
        e.m.sw(0xA6, n);
        if n & e.m.w(0xA8) == 0 {
            self.enemy_draw_check(&mut e.b, h);
        }
    }

    /// 0x48340: a leftover viewer: a changed input of player 2 that is exactly jump (0x20) starts animation `+$A0` again, exactly 0x40 sets `+$A0` to 0 and starts
    /// animation 0 (the table has one animation, so nothing shows).
    fn c23_input(&mut self, e: &mut Enemy) {
        let (cur, prev) = (self.w.input[1], self.flow.front.prev_input[1]);
        if cur == prev {
            return;
        }
        if cur == 0x20 {
            let a = e.m.w(0xA0);
            self.enemy_anim(&mut e.b, a);
        } else if cur == 0x40 {
            e.m.sw(0xA0, 0);
            self.enemy_anim(&mut e.b, 0);
        }
    }

    /// 0x482DC: under the foot (`+$A2`) while it is live 2: its X, its Z + 1, Y 0x1E; else the free (mode + 2).
    fn c23_follow(&mut self, e: &mut Enemy) {
        let foot = e.m.l(0xA2) as Handle;
        let p = self.body(foot);
        if p.live != 2 {
            e.b.mode = e.b.mode.wrapping_add(2);
            return;
        }
        let (z, x) = (p.zi().wrapping_add(1), p.xi());
        e.b.set_zi_keep(z);
        e.b.set_xi_keep(x);
        e.b.set_yi_keep(0x1E);
    }

    /// 0x4830C: the mask `+$A8` is 1 while the foot is above Y 0x68 (the leg is drawn every second frame), else 0.
    fn c23_mask(&mut self, e: &mut Enemy) {
        let foot = e.m.l(0xA2) as Handle;
        let y = self.body(foot).yi();
        e.m.sw(0xA8, if y > 0x68 { 1 } else { 0 });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn rom() -> Option<&'static AssetPack> {
        static A: std::sync::OnceLock<Option<&'static AssetPack>> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let path = std::path::Path::new("../../dino.zip");
            let path = if path.exists() { path } else { std::path::Path::new("../../../dino.zip") };
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// Episode 6 area 1 with the hero in control and a foot (variant 0) 0x80 right of the camera (the probe `spawn A 22 var=0 x=+0x80`), Z 0x100.
    fn game_with_foot(a: &'static AssetPack) -> (Game, usize) {
        let mut g = Game::new(a);
        g.flow_apply_config(crate::game::flow::Config::default());
        g.start_stage_at(5, 1, 0, &[0]);
        for _ in 0..400 {
            g.w.input = [0x10, 0, 0];
            g.tick();
            if g.flow.mode == 0xC && g.w.scripted == 0 {
                break;
            }
        }
        let x = g.cam.s2.cam_x as i16 + 0x80;
        let i = g.enemies.alloc().expect("a free slot");
        let e = &mut g.enemies.slots[i];
        e.b.live = 1;
        e.b.k20 = 22 * 4;
        e.b.set_xi(x);
        e.b.set_zi(0x100);
        (g, i)
    }

    /// The cycle of D3b section 6 (A20, bronto-0): act 2 (the rise), 4 (the lowering), 6 (the hover, 180 frames), 8 (60 frames), 0xA (the wind up), 0xC (the slam,
    /// attack 0x4C), 0xE (30 frames) and 2 again. The lengths of the rise, the lowering and the fall depend on the height of the floor; the recordings of the
    /// harness (m800_d3b_a20_foot) give them for the floor of the area.
    #[test]
    fn the_foot_runs_the_cycle_of_the_original() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let (mut g, i) = game_with_foot(a);
        let mut first: Vec<(u8, usize)> = Vec::new();
        let mut atk_in_slam = None;
        for t in 0..430 {
            g.w.input = [0, 0, 0];
            g.tick();
            let b = &g.enemies.slots[i].b;
            assert_eq!(b.live, 2, "the foot cannot be hit (frame {t})");
            if first.last().map(|f| f.0) != Some(b.act) {
                first.push((b.act, t));
            }
            if b.act == 0xC {
                atk_in_slam = Some(b.atk);
            }
        }
        let acts: Vec<u8> = first.iter().map(|f| f.0).collect();
        assert_eq!(acts, vec![2, 4, 6, 8, 0xA, 0xC, 0xE, 2]);
        let at = |act: u8| first.iter().find(|f| f.0 == act).unwrap().1;
        assert_eq!(at(8) - at(6), 180, "the hover");
        assert_eq!(at(0xA) - at(8), 60, "the wait");
        assert_eq!(first[7].1 - at(0xE), 30, "the rest");
        assert_eq!(atk_in_slam, Some(0x4C));
        assert_eq!(g.enemies.slots[i].b.atk, 0x63, "after the slam the foot has the touch attack again");
    }

    /// The leg (class 23) is made by the foot, follows it (X, Z + 1, Y 0x1E) and goes when the foot goes.
    #[test]
    fn the_leg_follows_the_foot_and_goes_with_it() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let (mut g, i) = game_with_foot(a);
        for _ in 0..60 {
            g.w.input = [0, 0, 0];
            g.tick();
        }
        let leg = g.enemies.slots.iter().position(|e| e.b.live != 0 && e.b.k20 == 23 * 4).expect("the foot made a leg");
        let (foot, l) = (&g.enemies.slots[i].b, &g.enemies.slots[leg].b);
        assert_eq!((l.xi(), l.zi(), l.yi(), l.live), (foot.xi(), foot.zi() + 1, 0x1E, 2));
        assert_eq!(g.enemies.slots[leg].m.l(0xA2), 0xFFFF_0000 | g.enemies.handle(i) as u32, "the owner is a sign extended address");
        // the foot is freed: the leg is gone two frames later
        g.enemies.slots[i].b.live = 0;
        for _ in 0..3 {
            g.tick();
        }
        assert_eq!(g.enemies.slots[leg].b.live, 0, "the leg goes with its foot");
    }
}
