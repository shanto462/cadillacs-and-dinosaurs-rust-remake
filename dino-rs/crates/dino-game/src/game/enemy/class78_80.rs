//! The ground critters of Episode 2 area 0, classes 78 (handler 0x5F198) and 80 (handler 0x5FF2E). D3b section 7. Both are harmless:
//! neither registers in a hit list (no 0x4BEA, 0x4BFA or 0x4C0A) and neither can be hit, they have no kill score and are not counted
//! (+$7D = 0xFF, name 0, credit 0xFF). Each frame they end with the far test (0x173C) and the draw check (0x49CA).
//!
//! Class 78 wanders: it keeps a direction (+$22) and turns by a random step every 11 frames, stops now and then to look around, and
//! stops and plays a reaction (state 4) when a hero or an enemy stands on top of it. After 1200 frames (+$A2) it walks off to the left.
//! Class 80 creeps left at 0.46 px a frame (the long +$A0 is subtracted from X) and stops and looks (animation 1) while a hero is
//! within 0x50 px in front of it.
//!
//! Owner: enemy port (M6-03).

use super::{off::*, Enemy};
use crate::game::{Game, Handle};

/// Tables read from the ROM at run time (`dino-extract` TABLES): 0x5F5A0 the direction steps of the wander, 0x5F5B0 the directions after
/// a rest, 0x5F5C0 the stop roll (0 = go on).
mod t {
    pub const STEP: u32 = 0x5F5A0;
    pub const REST_DIR: u32 = 0x5F5B0;
    pub const STOP_ROLL: u32 = 0x5F5C0;
}

const A78_ANIM: u32 = 0x5F4F8;
const A80_ANIM: u32 = 0x600E0;

/// Class 78 own bytes: +$A0 word wander timer, +$A2 word life, +$A4 byte direction timer.
const WANDER_T: usize = 0xA0;
const LIFE: usize = 0xA2;
const TURN_T: usize = 0xA4;

impl Game {
    /// 0x5F198: class 78. Mode 0 init (then the same frame mode 2), mode 2 the states, 4 and 6 free (0x5F4F2).
    pub(super) fn enemy_class78(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.c78_init(e);
                self.c78_run(e, h);
            }
            2 => self.c78_run(e, h),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x5F1AC: palette 0x9E; a record on the right of the camera X + 0xC0 walks left (dir 0x18), else right (dir 8); speed table 2.
    fn c78_init(&mut self, e: &mut Enemy) {
        e.b.palette_override = 0x9E;
        let cx = self.cam.s2.cam_x.wrapping_add(0xC0);
        let (dir, facing) = if cx >= e.b.xi() as u16 { (8, 1) } else { (0x18, 0) };
        e.b.dir = dir;
        e.b.facing = facing;
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sb(TURN_T, 0xA);
        e.m.sw(WANDER_T, 0xF0);
        e.m.sw(LIFE, 0x4B0);
        self.enemy_set_speed(&mut e.b, 2); // 0x17A4
        e.b.hitstop = 0;
        e.b.who = 0xFF;
        e.b.id = 0;
        e.m.sb(ATTACKING, 0xFF);
        e.m.sb(B87, 0);
        e.b.b63 = 0xFF;
        e.b.anim_table = A78_ANIM;
        self.enemy_anim(&mut e.b, 0);
        e.b.mode = e.b.mode.wrapping_add(2);
    }

    /// 0x5F22A: the life counter, the state (+$06, table 0x5F246), the far test and the draw check.
    fn c78_run(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sw(LIFE, e.m.w(LIFE).wrapping_sub(1));
        match e.b.act {
            0 => self.c78_wander(e),
            2 => self.c78_rest(e),
            4 => self.c78_react(e),
            6 => self.c78_leave(e),
            _ => {}
        }
        self.enemy_despawn_if_far(e, 0); // 0x173C
        self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
    }

    /// 0x5F24E: state 0, the wander. The wander timer +$A0 runs out into state 2; every 11 frames a random step turns the direction.
    fn c78_wander(&mut self, e: &mut Enemy) {
        let w = e.m.w(WANDER_T).wrapping_sub(1);
        e.m.sw(WANDER_T, w);
        if (w as i16) < 0 {
            e.b.act = 2;
        } else {
            let t = e.m.b(TURN_T).wrapping_sub(1);
            e.m.sb(TURN_T, t);
            if (t as i8) < 0 {
                let r = (self.rand() & 0xF) as u32;
                let step = self.assets.enemy.t.u8(t::STEP, r);
                e.b.dir = e.b.dir.wrapping_add(step) & 0x1F;
                e.b.facing = if e.b.dir & 0x10 != 0 { 0 } else { 1 };
                self.c78_near(e); // jsr 0x5F324
                e.m.sb(TURN_T, 0xA);
            }
        }
        self.enemy_move_dir(&mut e.b); // 0x1862
        self.enemy_floor_collision(e); // 0x12CB4
        self.enemy_step(&mut e.b); // jmp 0x121E
    }

    /// 0x5F2B6: state 2, the rest. Step 0 looks around for 120 frames (animation 1); then a new direction from the table 0x5F5B0 and the
    /// wander (180 frames), or the leave (state 6) once the life counter is below zero.
    fn c78_rest(&mut self, e: &mut Enemy) {
        if e.b.step == 0 {
            e.b.step = 2;
            e.m.sw(WANDER_T, 0x78);
            return self.enemy_anim(&mut e.b, 1);
        }
        let w = e.m.w(WANDER_T).wrapping_sub(1);
        e.m.sw(WANDER_T, w);
        if (w as i16) >= 0 {
            self.enemy_step(&mut e.b); // jmp 0x121E
            return;
        }
        if (e.m.w(LIFE) as i16) < 0 {
            e.b.act = 6; // 0x5F2F0
        } else {
            e.b.act = 0;
            let r = (self.rand() & 0xF) as u32;
            e.b.dir = self.assets.enemy.t.u8(t::REST_DIR, r);
            e.m.sw(WANDER_T, 0xB4);
        }
        e.b.step = 0; // 0x5F318
        self.enemy_anim(&mut e.b, 0);
    }

    /// 0x5F45A: state 4, the reaction to a hero or an enemy on top of it: animation 2 for 120 frames, animation 3 to its end, then the
    /// rest again (state 2, animation 1, 120 frames).
    fn c78_react(&mut self, e: &mut Enemy) {
        match e.b.step {
            0 => {
                e.m.sw(WANDER_T, 0x78);
                e.b.step = 2;
                self.enemy_anim(&mut e.b, 2);
            }
            2 => {
                let w = e.m.w(WANDER_T).wrapping_sub(1);
                e.m.sw(WANDER_T, w);
                if (w as i16) >= 0 {
                    self.enemy_step(&mut e.b); // jmp 0x121E
                } else {
                    e.b.step = 4;
                    self.enemy_anim(&mut e.b, 3);
                }
            }
            4 => {
                if self.enemy_step(&mut e.b) {
                    e.m.sw(WANDER_T, 0x78);
                    e.b.act = 2;
                    e.b.step = 0;
                    self.enemy_anim(&mut e.b, 1);
                }
            }
            _ => {}
        }
    }

    /// 0x5F4C0: state 6, the leave: walks left (dir 0x18) until the far test frees it.
    fn c78_leave(&mut self, e: &mut Enemy) {
        if e.b.step == 0 {
            e.b.step = 2;
            e.b.dir = 0x18;
            e.b.facing = 0;
        } else {
            self.enemy_move_dir(&mut e.b); // 0x1862
        }
        self.enemy_step(&mut e.b); // jmp 0x121E
    }

    /// 0x5F324: is a hero (not in the air) within 16 px in X and 8 px in depth, or an enemy of last frame's list 0x69BA (on the ground,
    /// not in a hit reaction act 2, not with the reaction 0x30)? Then 0x5F43C: a roll of 4 bits against the table 0x5F5C0 may start state 4.
    fn c78_near(&mut self, e: &mut Enemy) {
        let (x, z) = (e.b.xi(), e.b.zi());
        let mut hit = false;
        for p in 0..3 {
            let pb = &self.players[p].b;
            if pb.live == 0 || pb.airborne != 0 {
                continue;
            }
            let dx = pb.xi().wrapping_sub(x).wrapping_add(0x10);
            if dx < 0 || dx as u16 >= 0x20 {
                continue;
            }
            if (pb.zi().wrapping_sub(z).wrapping_add(8) as u16) < 0x10 {
                hit = true;
                break;
            }
        }
        if !hit {
            for a1 in self.combat.get(0x69BA).to_vec() {
                let b = self.body(a1);
                if b.live == 0 || b.airborne != 0 || b.act == 2 || b.reaction == 0x30 {
                    continue;
                }
                let dx = b.xi().wrapping_sub(x).wrapping_add(0x10);
                if dx < 0 || dx as u16 >= 0x20 {
                    continue;
                }
                let dz = z.wrapping_sub(b.zi()).wrapping_add(8);
                if dz >= 0 && (dz as u16) < 0x10 {
                    hit = true;
                    break;
                }
            }
        }
        if hit {
            let r = (self.rand() & 0xF) as u32;
            if self.assets.enemy.t.u8(t::STOP_ROLL, r) != 0 {
                e.b.act = 4;
            }
        }
    }

    // ------------------------------------------------------------------ class 80

    /// 0x5FF2E: class 80. Mode 0 init (then mode 2 in the same frame), 2 the states, 4 and 6 free (0x600DA).
    pub(super) fn enemy_class80(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.c80_init(e);
                self.c80_run(e, h);
            }
            2 => self.c80_run(e, h),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x5FF42: palette 0x9E, facing left, the creep speed 0x7500 (16.16 pixels a frame, +$A0), a 10 frame start delay (+$A4).
    fn c80_init(&mut self, e: &mut Enemy) {
        e.b.palette_override = 0x9E;
        e.b.facing = 0;
        e.m.sl(0xA0, 0x7500);
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sw(TURN_T, 0xA);
        e.m.sb(0xA6, 0);
        e.b.hitstop = 0;
        e.b.who = 0xFF;
        e.b.id = 0;
        e.m.sb(ATTACKING, 0xFF);
        e.m.sb(B87, 0);
        e.b.b63 = 0xFF;
        e.b.anim_table = A80_ANIM;
        self.enemy_anim(&mut e.b, 0);
        e.b.mode = e.b.mode.wrapping_add(2);
    }

    /// 0x5FF9C: the state (table 0x5FFB4), the far test, the draw check.
    fn c80_run(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => {
                // 0x5FFB8: creep left; after the delay a hero in front stops it.
                let w = e.m.w(TURN_T).wrapping_sub(1);
                e.m.sw(TURN_T, w);
                if (w as i16) < 0 && self.c80_near(e.b.xi(), e.b.zi()) {
                    e.b.act = 2;
                }
                e.b.x = e.b.x.wrapping_sub(e.m.l(0xA0) as i32); // sub.l d0,$8(a6)
                self.enemy_step(&mut e.b); // jmp 0x121E
            }
            2 => {
                if e.b.step == 0 {
                    // 0x60096: look (animation 1) for up to 300 frames.
                    e.b.step = 2;
                    e.m.sw(TURN_T, 0x12C);
                    self.enemy_anim(&mut e.b, 1);
                } else {
                    // 0x600AA
                    let w = e.m.w(TURN_T).wrapping_sub(1);
                    e.m.sw(TURN_T, w);
                    if (w as i16) >= 0 && self.c80_near(e.b.xi(), e.b.zi()) {
                        self.enemy_step(&mut e.b); // jmp 0x121E
                    } else {
                        // 0x600C4: back to the creep, with a delay of 0x7FFF frames before it looks again.
                        e.b.act = 0;
                        e.b.step = 0;
                        e.m.sw(TURN_T, 0x7FFF);
                        self.enemy_anim(&mut e.b, 0);
                    }
                }
            }
            _ => {}
        }
        self.enemy_despawn_if_far(e, 0); // 0x173C
        self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
    }

    /// 0x5FFE0: is a hero 0 .. 0x4F px in front (his X larger than the critter's `x`) and in the depth window -16 .. +23 of `z`?
    /// The original returns d1 = 1; the players are tried in order 1, 2, 3.
    fn c80_near(&self, x: i16, z: i16) -> bool {
        (0..3).any(|p| {
            let pb = &self.players[p].b;
            let dx = pb.xi().wrapping_sub(x);
            let dz = pb.zi().wrapping_sub(z).wrapping_add(0x10);
            pb.live != 0 && dx >= 0 && (dx as u16) < 0x50 && dz >= 0 && (dz as u16) < 0x28
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    /// 0x5FFE0: a hero 0 .. 0x4F px in front of the critter and within -16 .. +23 in depth stops it; the players are tried in order.
    #[test]
    fn the_creeping_critter_sees_a_hero_in_front_only() {
        let mut g = Game::new(Box::leak(Box::new(AssetPack::default())));
        let (x, z) = (0x300, 0x50);
        assert!(!g.c80_near(x, z));
        g.players[1].b.live = 1;
        g.players[1].b.set_xi(0x34F);
        g.players[1].b.set_zi(0x50);
        assert!(g.c80_near(x, z));
        g.players[1].b.set_xi(0x350);
        assert!(!g.c80_near(x, z));
        g.players[1].b.set_xi(0x2FF); // behind
        assert!(!g.c80_near(x, z));
        g.players[1].b.set_xi(0x310);
        g.players[1].b.set_zi(0x50 - 0x11); // 17 behind in depth: outside
        assert!(!g.c80_near(x, z));
        g.players[1].b.set_zi(0x50 - 0x10);
        assert!(g.c80_near(x, z));
        g.players[1].b.set_zi(0x50 + 0x17);
        assert!(g.c80_near(x, z));
        g.players[1].b.set_zi(0x50 + 0x18);
        assert!(!g.c80_near(x, z));
    }
}
