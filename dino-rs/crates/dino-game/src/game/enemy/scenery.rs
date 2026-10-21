//! Pool A scenery and opening scene classes (phase 1 task 4a):
//! * class 49 (`+$20 = 0xC4`, 0x56A70): background bird controller, three of them at the start of
//!   stage 1 (they sit at the far Y = Z of the sky layer and swing left and right);
//! * class 50 (0x56C80): birds and similar sprites that fly in at run time, one `rand()` each
//!   at the start (variant = `+$26`, 0..0xC);
//! * class 54 (0x576D2): a bystander of the opening scene that follows the state of a master
//!   object (pool B, handle in `+$A4`);
//! * class 55 (0x5785C): the scripted thugs of the opening scene (Ferris or Gneiss, `+$84` 0 / 4):
//!   they play the master's act list, and fall back to the shared hit reaction 0x2A606 and the
//!   shared grab 0x2B35A when a player hits them.
//!
//! * class 56 (0x57C9C, M2-11): the flying cast member of the opening scene (X 0x4E0, height 0x60, depth 0x131). It is
//!   invisible while the master (pool B class 74, `prop/b74.rs`) is in its first two steps; in the third (the master's `+$05`
//!   is 6 when it looks) it makes a pool A class 57 object (`enemy/class57.rs`: the animation of the table 0x57E0E with the
//!   sound 0xE7) at (0x4E5, 0x6C, 0x131), is drawn for that one frame and then only waits to be freed (0x57D56).
//!
//! None of these touches the census (`+$7D = 0xFF`) except class 54. Offsets used here keep
//! the original meaning of the class (they differ from class 2's); the trace names of the raw
//! bytes are those of class 2, a byte is a byte (see `trace.rs`).

use super::{Enemy, off::*};
use crate::game::{Game, Handle};

/// Class 49 / 50 raw offsets.
mod o {
    /// Class 49: scroll2 camera X at the last frame, scroll3 camera X at the last frame.
    pub const CAM2_X: usize = 0x82;
    pub const CAM3_X: usize = 0x84;
    /// Class 49: X velocity / acceleration (longs).
    pub const BIRD_VX: usize = 0x8A;
    pub const BIRD_AX: usize = 0x86;
    /// Class 50: X acceleration, X velocity, Y acceleration, Y velocity (longs).
    pub const FLY_AX: usize = 0xA6;
    pub const FLY_VX: usize = 0xAA;
    pub const FLY_AY: usize = 0xAE;
    pub const FLY_VY: usize = 0xB2;
    /// Class 54 / 55.
    pub const MASTER: usize = 0xA4; // long: pointer to the master object
    pub const MASTER_SUB: usize = 0xA2; // the master's +$05 seen last
    pub const ONCE: usize = 0xA3; // "animation started" flag, also a toggle in class 55
    pub const TIMER: usize = 0xA8;
    pub const SPEED_X: usize = 0xAA;
    pub const SPEED_Z: usize = 0xAC;
    pub const GRAVITY: usize = 0xB0; // long
    pub const SPEED_Y: usize = 0xB4; // long
    /// Class 56: the child (class 57) has been made.
    pub const CHILD_MADE: usize = 0xA8;
    pub const SOUND_DONE: usize = 0xB8;
    pub const SOUND_T: usize = 0xBA;
}

/// Tables of class 55 (ROM addresses, `dino-extract` blobs).
mod t {
    pub const SPEED_X: u32 = 0x5794A;
    pub const SPEED_Z: u32 = 0x57956;
}

impl Game {
    // ------------------------------------------------------------------ class 49

    /// 0x56A70 class 49: background bird controller. Mode 0 initialises and runs the first
    /// frame at once, mode 2 is the frame, mode 4 / 6 free the slot (0x56C32).
    pub fn enemy_class49(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                self.enemy_bird_init(e);
                self.enemy_bird_frame(e, h);
            }
            2 => self.enemy_bird_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i);
            }
        }
    }

    /// 0x56A84.
    fn enemy_bird_init(&mut self, e: &mut Enemy) {
        e.b.live = 2;
        e.b.anim_table = 0x56C38;
        e.b.palette_override = 0x9D;
        e.m.sb(ATTACKING, 0xFF);
        e.b.y = e.b.z; // move.l $10(a6),$c(a6)
        e.m.sw(T80, 0);
        e.b.draw_flags = 0x10;
        e.m.sl(o::BIRD_VX, 0x1000);
        e.m.sl(o::BIRD_AX, 0x100);
        let r = self.rand() as u32 & 0x3F00; // andi.l #$3f00,d0
        e.m.sl(o::BIRD_VX, e.m.l(o::BIRD_VX).wrapping_add(r));
        e.m.sw(o::CAM2_X, self.cam.s2.cam_x);
        e.m.sw(o::CAM3_X, self.cam.s3.cam_x);
        self.enemy_floor_collision(e);
        self.enemy_anim(&mut e.b, 2);
        e.b.mode += 2;
    }

    /// 0x56AEC: the swing of the variant (0x56B10 / 0x56B5E), animation, despawn test, draw.
    fn enemy_bird_frame(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.var26 {
            0 => self.enemy_bird_swing(e, 0xA000, 0xFFFF_FF00, 0x100, 1),
            2 => self.enemy_bird_swing(e, 0x8000, 0xFFFF_FF40, 0xC0, 4),
            v => panic!("class 49 variant {v:#x}"),
        }
        self.enemy_step(&mut e.b);
        let i = self.enemy_index(h);
        self.enemy_despawn_if_far(e, i);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x56B10 (`limit` 0xA000, acceleration -0x100 / +0x100, scroll3 factor 1) and 0x56B5E
    /// (0x8000, -0xC0 / +0xC0, factor 4: `asl.w #2,d1` at 0x56C24).
    fn enemy_bird_swing(&mut self, e: &mut Enemy, limit: i32, to_left: u32, to_right: u32, factor: i16) {
        let vx = e.m.l(o::BIRD_VX).wrapping_add(e.m.l(o::BIRD_AX));
        e.m.sl(o::BIRD_VX, vx);
        e.b.x = e.b.x.wrapping_add(vx as i32);
        let v = vx as i32;
        // cmpi.l #limit,$8a(a6) ; blt: at or above the limit the bird turns left.
        if v >= limit {
            e.m.sl(o::BIRD_AX, to_left);
            self.enemy_anim(&mut e.b, 0);
        }
        // cmpi.l #-limit,$8a(a6) ; bgt: at or below minus the limit it turns right.
        if v <= -limit {
            e.m.sl(o::BIRD_AX, to_right);
            self.enemy_anim(&mut e.b, 1);
        }
        self.enemy_bird_parallax(e, factor);
    }

    /// 0x56BAC / 0x56BEE: the bird follows scroll2 (with it) and counters scroll3 (against it,
    /// `factor` times). Only the integer part of X moves.
    fn enemy_bird_parallax(&mut self, e: &mut Enemy, factor: i16) {
        let cx = self.cam.s2.cam_x;
        let last = e.m.w(o::CAM2_X);
        if last != cx {
            let d = cx.wrapping_sub(last);
            e.b.set_xi_keep(e.b.xi().wrapping_add(d as i16));
            e.m.sw(o::CAM2_X, cx);
        }
        let c3 = self.cam.s3.cam_x;
        let last = e.m.w(o::CAM3_X);
        if last != c3 {
            let d = (c3.wrapping_sub(last) as i16).wrapping_mul(factor);
            e.b.set_xi_keep(e.b.xi().wrapping_sub(d));
            e.m.sw(o::CAM3_X, c3);
        }
    }

    // ------------------------------------------------------------------ class 50

    /// 0x56C80 class 50: flying scenery. Mode 0 builds the variant and returns (the first
    /// frame runs on the next call), mode 2 moves it, mode 4 / 6 free the slot (0x57086).
    pub fn enemy_class50(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => self.enemy_fly_init(e),
            2 => self.enemy_fly_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i);
            }
        }
    }

    /// 0x56C94 with the variant inits 0x56CE0 .. 0x56F0C.
    fn enemy_fly_init(&mut self, e: &mut Enemy) {
        e.b.live = 2;
        e.b.anim_table = 0x5708C;
        e.b.palette_override = 0x9D;
        e.m.sb(ATTACKING, 0xFF);
        e.m.sw(WA0, 0);
        e.b.draw_flags = 0x10;
        let cx = self.cam.s2.cam_x as i16;
        let cy = self.cam.s2.cam_y;
        match e.b.var26 {
            0 => {
                // 0x56CE0
                e.b.set_xi_keep(cx.wrapping_sub(0x30));
                if cy != 0x100 {
                    e.b.mode += 2;
                }
                self.enemy_fly_pos(e, 0x110 - 0xB0, 0x180);
                e.m.sl(o::FLY_VY, 0xFFFF_E800);
                e.m.sl(o::FLY_AY, 0x80);
                let r = self.rand() as u32 & 0x30;
                e.m.sl(o::FLY_AY, e.m.l(o::FLY_AY).wrapping_add(r));
                e.m.sl(o::FLY_VX, 0x10000);
                e.m.sl(o::FLY_AX, 0);
                self.enemy_anim(&mut e.b, 1);
            }
            2 => {
                // 0x56D48
                e.b.set_xi_keep(cx.wrapping_add(0x190));
                if cy != 0x100 {
                    e.b.mode += 2;
                }
                self.enemy_fly_pos(e, 0x110 - 0xB0, 0x180);
                e.m.sl(o::FLY_VY, 0xFFFF_F000);
                e.m.sl(o::FLY_AY, 0x80);
                let r = self.rand() as u32 & 0x30;
                e.m.sl(o::FLY_AY, e.m.l(o::FLY_AY).wrapping_add(r));
                e.m.sl(o::FLY_VX, 0xFFFF_0000);
                e.m.sl(o::FLY_AX, 0);
                self.enemy_anim(&mut e.b, 0);
            }
            4 => {
                // 0x56DB0: the random X offset is drawn first.
                let r = self.rand() & 0x30;
                e.b.set_xi_keep(cx.wrapping_add(0x190).wrapping_sub(r as i16));
                e.b.set_yi_keep((cy as i16).wrapping_sub(0x80));
                e.b.set_zi_keep(0x180);
                e.m.sl(o::FLY_VY, 0xFFFC_0000);
                e.m.sl(o::FLY_AY, 0x3000);
                e.m.sl(o::FLY_VX, 0xFFFB_0000);
                // d0.l = 0 above the rand() word here (moveq #0 at 0x56CB8): the mask keeps bits 12..14.
                let r = self.rand() as u32 & 0x17000;
                e.m.sl(o::FLY_VX, e.m.l(o::FLY_VX).wrapping_sub(r));
                e.m.sl(o::FLY_AX, 0x100);
                e.m.sw(WA2, 0);
                self.enemy_anim(&mut e.b, 3);
            }
            6 => {
                // 0x56E1A
                e.b.set_xi_keep(cx.wrapping_sub(0x30));
                self.enemy_fly_pos(e, 0x110 - 0xB0, 0x180);
                e.m.sl(o::FLY_VY, 0xFFFF_E000);
                e.m.sl(o::FLY_AY, 0x100);
                let r = self.rand() as u32 & 0x30;
                e.m.sl(o::FLY_AY, e.m.l(o::FLY_AY).wrapping_add(r));
                e.m.sl(o::FLY_VX, 0x80000);
                e.m.sl(o::FLY_AX, 0);
                e.b.facing = 1;
                self.enemy_anim(&mut e.b, 4);
            }
            8 => {
                // 0x56E7C
                e.b.set_xi_keep(cx.wrapping_add(0x190));
                self.enemy_fly_pos(e, 0x120 - 0xB0, 0x180);
                e.m.sl(o::FLY_VY, 0xFFFF_E000);
                e.m.sl(o::FLY_AY, 0x100);
                let r = self.rand() as u32 & 0x30;
                e.m.sl(o::FLY_AY, e.m.l(o::FLY_AY).wrapping_add(r));
                e.m.sl(o::FLY_VX, 0xFFF8_0000);
                e.m.sl(o::FLY_AX, 0);
                e.b.facing = 0;
                self.enemy_anim(&mut e.b, 4);
            }
            0xA => {
                // 0x56EDE, then the tail 0x56F36
                e.b.set_xi_keep(cx.wrapping_sub(0x18));
                self.enemy_fly_pos(e, 0x110 - 0xE0, 0x150);
                e.m.sl(o::FLY_VX, 0x80000);
                self.enemy_fly_tail(e);
            }
            0xC => {
                // 0x56F0C, then the tail 0x56F36
                e.b.set_xi_keep(cx.wrapping_sub(0x30));
                self.enemy_fly_pos(e, 0x110 - 0x100, 0x130);
                e.m.sl(o::FLY_VX, 0xC0000);
                self.enemy_fly_tail(e);
            }
            v => panic!("class 50 variant {v:#x}"),
        }
        self.enemy_floor_collision(e);
        e.b.mode += 2;
    }

    /// `move.w d0,$c(a6)` / `move.w #z,$10(a6)`: the integer parts of Y and Z.
    fn enemy_fly_pos(&mut self, e: &mut Enemy, y: i16, z: i16) {
        e.b.set_yi_keep(y);
        e.b.set_zi_keep(z);
    }

    /// 0x56F36: variants 0xA and 0xC share the rest of the init.
    fn enemy_fly_tail(&mut self, e: &mut Enemy) {
        e.m.sl(o::FLY_VY, 0);
        e.m.sl(o::FLY_AY, 0x2000);
        let r = self.rand() as u32 & 0x700;
        e.m.sl(o::FLY_AY, e.m.l(o::FLY_AY).wrapping_add(r));
        e.m.sl(o::FLY_AX, 0);
        e.b.facing = 1;
        self.enemy_anim(&mut e.b, 4);
    }

    /// 0x56F6E: the move of the variant (table 0x56F8E), animation, despawn test, draw.
    fn enemy_fly_frame(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.var26 {
            0 | 2 | 6 | 8 => self.enemy_fly_move(e),
            4 => self.enemy_fly_flap(e),
            0xA => self.enemy_fly_glide(e, 0x2B0),
            0xC => self.enemy_fly_glide(e, 0x100),
            v => panic!("class 50 variant {v:#x}"),
        }
        self.enemy_step(&mut e.b);
        let i = self.enemy_index(h);
        self.enemy_despawn_if_far(e, i);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// vy += ay, Y += vy, vx += ax, X += vx (all longs, 16.16).
    fn enemy_fly_move(&mut self, e: &mut Enemy) {
        self.enemy_fly_vertical(e);
        self.enemy_fly_horizontal(e);
    }

    fn enemy_fly_vertical(&mut self, e: &mut Enemy) {
        let vy = e.m.l(o::FLY_VY).wrapping_add(e.m.l(o::FLY_AY));
        e.m.sl(o::FLY_VY, vy);
        e.b.y = e.b.y.wrapping_add(vy as i32);
    }

    /// Returns the new X velocity (the glide variants keep it in d0).
    fn enemy_fly_horizontal(&mut self, e: &mut Enemy) -> u32 {
        let vx = e.m.l(o::FLY_VX).wrapping_add(e.m.l(o::FLY_AX));
        e.m.sl(o::FLY_VX, vx);
        e.b.x = e.b.x.wrapping_add(vx as i32);
        vx
    }

    /// 0x56FBE (variant 4): like the plain move, and once the bird stops rising (vy >= 0 while
    /// `+$A2` is still clear) it flaps: vy = 0, ay = 0x2000, animation 2. `tst.l $a2(a6)` reads
    /// +$A2..+$A5, but the init clears only the word at +$A2.
    fn enemy_fly_flap(&mut self, e: &mut Enemy) {
        self.enemy_fly_vertical(e);
        if e.m.l(WA2) == 0 && (e.m.l(o::FLY_VY) as i32) >= 0 {
            e.m.sl(o::FLY_VY, 0);
            e.m.sl(o::FLY_AY, 0x2000);
            e.m.sw(WA2, 1);
            self.enemy_anim(&mut e.b, 2);
        }
        self.enemy_fly_horizontal(e);
    }

    /// 0x5700E (X >= 0x2B0) / 0x5704A (X >= 0x100), variants 0xA and 0xC: the vertical move only
    /// once the bird is far enough right, then X plus a random jitter. The mask of the jitter
    /// is applied to d0 = the new X velocity with its low word replaced by `rand()`, so bits 16 and
    /// 17 come from the high word of the velocity.
    fn enemy_fly_glide(&mut self, e: &mut Enemy, from_x: i16) {
        if e.b.xi() >= from_x {
            self.enemy_fly_vertical(e);
        }
        let vx = self.enemy_fly_horizontal(e);
        let d0 = (vx & 0xFFFF_0000) | self.rand() as u32;
        e.b.x = e.b.x.wrapping_add((d0 & 0x3F000) as i32);
    }

    // ------------------------------------------------------------------ class 54

    /// 0x576D2 class 54: bystander of the opening scene.
    pub fn enemy_class54(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                // 0x576E6
                e.b.live = 2;
                e.b.anim_table = 0x577D2;
                e.b.palette_override = 0x99;
                e.m.sw(WA0, 0);
                e.m.sb(o::ONCE, 0);
                self.enemy_floor_collision(e);
                self.enemy_anim(&mut e.b, 0);
                e.b.mode += 2;
                self.enemy_scene_watch_frame(e, h);
            }
            2 => self.enemy_scene_watch_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i);
            }
        }
    }

    /// The master's `+$05` (0x577AA / 0x57B52 read `$5(a3)` of the long at +$A4). An address that
    /// is no object reads 0.
    fn enemy_scene_master_sub(&self, e: &Enemy) -> u8 {
        let h = e.m.w(o::MASTER + 2);
        if crate::game::slot_of(h).is_some() { self.body(h).sub } else { 0 }
    }

    /// 0x57714: follow the master (0x577AA), the act routine (table 0x57732 by `+$05`), animation,
    /// draw.
    fn enemy_scene_watch_frame(&mut self, e: &mut Enemy, h: Handle) {
        let m = self.enemy_scene_master_sub(e);
        if m != e.m.b(o::MASTER_SUB) {
            e.m.sb(o::ONCE, 0);
            e.b.sub = e.b.sub.wrapping_add(2);
            e.m.sb(o::MASTER_SUB, m);
        }
        match e.b.sub {
            4 => {
                // 0x57746
                if e.m.b(o::ONCE) == 0 {
                    self.enemy_anim(&mut e.b, 1);
                    e.m.sb(o::ONCE, 1);
                }
            }
            8 => {
                // 0x5775E
                if e.m.b(o::ONCE) == 0 {
                    self.enemy_anim(&mut e.b, 2);
                    e.m.sb(o::ONCE, 1);
                }
            }
            0xC => {
                // 0x57776
                e.b.sub = e.b.sub.wrapping_add(2);
                e.m.sw(o::TIMER, 0x3C);
            }
            0xE => {
                // 0x57782
                let t = e.m.w(o::TIMER).wrapping_sub(1);
                e.m.sw(o::TIMER, t);
                if t == 0 {
                    self.enemy_anim(&mut e.b, 3);
                    e.b.facing = 1;
                    e.b.sub = e.b.sub.wrapping_add(2);
                }
            }
            0x10 => {
                // 0x5779E: walk right, then the despawn test.
                e.b.set_xi_keep(e.b.xi().wrapping_add(4));
                let i = self.enemy_index(h);
                self.enemy_despawn_if_far(e, i);
            }
            _ => {} // 0, 2, 6, 0xA: rts (0x57744)
        }
        self.enemy_step(&mut e.b);
        self.enemy_draw_check(&mut e.b, h);
    }

    // ------------------------------------------------------------------ class 56

    /// 0x57C9C class 56: the flying cast member of the opening scene (jump table 0x57CA8: mode 0 -> 0x57CB0, 2 -> 0x57CDC,
    /// 4 and 6 -> 0x57D7E, which is the free routine 0x48FC).
    pub fn enemy_class56(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                // 0x57CB0: the animation table 0x57D84, the flags +$A0 .. +$A1, +$A3 and +$A8 clear, the terrain (0x12CB4), animation 0
                e.b.live = 2;
                e.b.anim_table = 0x57D84;
                e.m.sw(WA0, 0);
                e.m.sb(o::ONCE, 0);
                e.m.sb(o::CHILD_MADE, 0);
                self.enemy_floor_collision(e);
                self.enemy_anim(&mut e.b, 0);
                e.b.mode += 2;
                self.enemy_c56_frame(e, h); // falls into 0x57CDC
            }
            2 => self.enemy_c56_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i);
            }
        }
    }

    /// 0x57CDC: follow the master (0x57D5C, the same routine as class 54's 0x577AA), then the routine of `+$05` (table 0x57CEE).
    fn enemy_c56_frame(&mut self, e: &mut Enemy, h: Handle) {
        let m = self.enemy_scene_master_sub(e);
        if m != e.m.b(o::MASTER_SUB) {
            e.m.sb(o::ONCE, 0);
            e.b.sub = e.b.sub.wrapping_add(2);
            e.m.sb(o::MASTER_SUB, m);
        }
        match e.b.sub {
            0 | 2 => {} // 0x57CFE: rts (no animation step and no picture)
            4 => {
                // 0x57D00: once, the class 57 object at (0x4E5, 0x6C, 0x131) that remembers this object in `+$A4`; the animation
                // step and the draw list
                if e.m.b(o::CHILD_MADE) == 0 {
                    if let Some(c) = self.spawn_alloc(0) {
                        let b = self.body_mut(c);
                        b.live = 1;
                        b.k20 = 0xE4;
                        b.var26 = 0;
                        b.set_xi_keep(0x4E5);
                        b.set_yi_keep(0x6C);
                        b.set_zi_keep(0x131);
                        let i = self.enemy_index(c);
                        self.enemies.slots[i].m.sl(o::MASTER, 0xFFFF_0000 | h as u32); // move.l a6,$a4(a0)
                    }
                    e.m.sb(o::CHILD_MADE, 1);
                }
                self.enemy_step(&mut e.b);
                self.enemy_draw_check(&mut e.b, h);
            }
            6 | 8 | 0xA | 0xC => {
                // 0x57D50: jmp 0x173C
                let i = self.enemy_index(h);
                self.enemy_despawn_if_far(e, i);
            }
            0xE => e.b.mode += 2, // 0x57D56: freed in the next frame
            _ => {} // the table has 8 entries (sub 0 to 0xE); the master never goes further
        }
    }

    // ------------------------------------------------------------------ class 55

    /// 0x5785C class 55: a scripted thug of the opening scene.
    pub fn enemy_class55(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => self.enemy_scene_thug_init(e),
            2 => self.enemy_scene_thug_alive(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i);
            }
        }
    }

    /// 0x57870: one `rand()` picks Ferris (animation table 0x57BA4) or Gneiss (0x57C20); HP 1.
    fn enemy_scene_thug_init(&mut self, e: &mut Enemy) {
        e.b.live = 1;
        if self.rand() & 1 == 0 {
            e.b.anim_table = 0x57BA4;
            e.b.id = 0;
            e.m.sw(W84, 0);
        } else {
            e.b.anim_table = 0x57C20;
            e.b.id = 1;
            e.m.sw(W84, 4);
        }
        self.enemy_anim(&mut e.b, 0);
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        e.b.hp_bar = 0x14;
        e.b.hp = 1;
        e.b.hp_prev = 1;
        e.b.stun = 0xA;
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.b.reaction = 0;
        e.m.sb(ATTACKING, 0);
        e.m.sb(B87, 0);
        e.b.b63 = 0;
        e.b.b72 = 0xC8;
        e.m.sw(WA0, 0);
        e.m.sw(o::SOUND_DONE, 0);
        let v = e.b.var26;
        e.m.sw(o::SOUND_T, 1u16.wrapping_add(v.wrapping_add(v)));
        let k = v.wrapping_add(v) as u32;
        e.m.sw(o::SPEED_X, self.assets.enemy.t.u16(t::SPEED_X, k));
        e.m.sw(o::SPEED_Z, self.assets.enemy.t.u16(t::SPEED_Z, k));
        self.enemy_floor_collision(e);
        if (v as i16) >= 3 {
            self.enemy_anim(&mut e.b, 1);
        }
        e.m.sb(ATTACKING, 0xFF);
        e.b.mode += 2;
    }

    /// 0x57962: sub 0 = the act list below, sub 2 = the shared hit reaction 0x2A606, sub 6 = the
    /// shared grab 0x2B35A (the table at 0x57970 holds 0 for sub 4).
    fn enemy_scene_thug_alive(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => self.enemy_scene_thug_act(e, h),
            2 => self.enemy_hit_reaction(e, h),
            6 => self.enemy_grabbed(e, h),
            s => panic!("class 55 sub {s:#x}"),
        }
    }

    /// 0x57980: take the master's act (0x57B52), run it (table 0x5799E), register as hurtable
    /// (0x4BEA), step the animation (0x121E).
    fn enemy_scene_thug_act(&mut self, e: &mut Enemy, h: Handle) {
        let m = self.enemy_scene_master_sub(e);
        if m != e.m.b(o::MASTER_SUB) {
            e.b.act = m;
            e.m.sb(o::MASTER_SUB, m);
        }
        match e.b.act {
            0 | 2 | 4 => self.enemy_scene_thug_walk_right(e, h),
            6 | 0xA => {
                // 0x57A74 / 0x57AA6: the standing animation, restarted every frame.
                self.enemy_anim(&mut e.b, 1);
                self.enemy_draw_check(&mut e.b, h);
            }
            8 => self.enemy_scene_thug_walk_to_spot(e, h),
            0xC => {
                // 0x57A82: run at +$AA px per frame, +$AC in Z every second frame.
                e.b.set_xi_keep(e.b.xi().wrapping_add(e.m.w(o::SPEED_X) as i16));
                let t = e.m.b(o::ONCE).wrapping_add(1) & 1;
                e.m.sb(o::ONCE, t);
                if t == 0 {
                    e.b.set_zi_keep(e.b.zi().wrapping_add(e.m.w(o::SPEED_Z) as i16));
                }
                self.enemy_draw_check(&mut e.b, h);
            }
            0xE => {
                // 0x57AB4: knocked into the air: gravity -0x4000, speed 4.0, a sound request.
                self.enemy_anim(&mut e.b, 2);
                e.m.sl(o::GRAVITY, 0xFFFF_C000);
                e.m.sl(o::SPEED_Y, 0x40000);
                e.b.set_yi_keep(1);
                self.sound(0x3B); // 0xA63C with d0 = 0x3B
                self.enemy_draw_check(&mut e.b, h);
            }
            0x10 => self.enemy_scene_thug_fly(e, h),
            0x12 => {
                // 0x57B2A: blink out, then die (mode 4 frees the slot).
                let t = e.m.w(o::TIMER).wrapping_sub(1);
                e.m.sw(o::TIMER, t);
                if t == 0 {
                    e.b.mode += 2;
                }
                let d0 = e.m.w(o::TIMER) as i16;
                if d0 > 0x30 || d0 & 1 == 0 {
                    self.enemy_draw_check(&mut e.b, h);
                }
            }
            a => panic!("class 55 act {a:#x}"),
        }
        self.enemy_push_hurtable(h);
        self.enemy_step(&mut e.b);
    }

    /// 0x579B2: walk right to 0x3C0 (variant 3) or 0x3D0 (4, 5); variants 0..2 stay.
    fn enemy_scene_thug_walk_right(&mut self, e: &mut Enemy, h: Handle) {
        let v = e.b.var26 as i16;
        if v >= 3 {
            let limit = if v == 3 { 0x3C0 } else { 0x3D0 };
            if e.b.xi() >= limit {
                self.enemy_anim(&mut e.b, 0);
            } else {
                e.b.set_xi_keep(e.b.xi().wrapping_add(2));
            }
        }
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x579F6: walk to the spot: right to 0x3C0 / 0x3D0 (variants 3..5) or left to 0x480 / 0x470
    /// (0..2), animation 0 on arrival.
    fn enemy_scene_thug_walk_to_spot(&mut self, e: &mut Enemy, h: Handle) {
        let v = e.b.var26 as i16;
        if v >= 3 {
            let limit = if v == 3 { 0x3C0 } else { 0x3D0 };
            if e.b.xi() >= limit {
                self.enemy_anim(&mut e.b, 0);
            } else {
                e.b.set_xi_keep(e.b.xi().wrapping_add(2));
            }
        } else {
            let limit = if v == 0 { 0x480 } else { 0x470 };
            if e.b.xi() <= limit {
                self.enemy_anim(&mut e.b, 0);
            } else {
                e.b.set_xi_keep(e.b.xi().wrapping_sub(2));
            }
        }
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x57AEA: the flight backwards. A sound request after `+$BA` frames, then 0x57B6E (the
    /// vertical speed changes by the gravity, X goes back by +$AA, Z by +$AC every second frame).
    fn enemy_scene_thug_fly(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.w(o::SOUND_DONE) == 0 {
            let t = e.m.w(o::SOUND_T).wrapping_sub(1);
            e.m.sw(o::SOUND_T, t);
            if t == 0 {
                e.m.sw(o::SOUND_DONE, 1);
                self.snd_enemy_death(0, e.b.xi(), e.b.facing); // 0xA354
            }
        }
        // 0x57B6E
        let vy = e.m.l(o::SPEED_Y).wrapping_add(e.m.l(o::GRAVITY));
        e.m.sl(o::SPEED_Y, vy);
        e.b.y = e.b.y.wrapping_add(vy as i32);
        e.b.set_xi_keep(e.b.xi().wrapping_sub(e.m.w(o::SPEED_X) as i16));
        let t = e.m.b(o::ONCE).wrapping_add(1) & 1;
        e.m.sb(o::ONCE, t);
        if t == 0 {
            e.b.set_zi_keep(e.b.zi().wrapping_sub(e.m.w(o::SPEED_Z) as i16));
        }
        if e.b.yi() < 0 {
            self.enemy_anim(&mut e.b, 2);
            e.m.sw(o::TIMER, 0x60);
            e.b.act = e.b.act.wrapping_add(2);
        }
        self.enemy_draw_check(&mut e.b, h);
    }
}
