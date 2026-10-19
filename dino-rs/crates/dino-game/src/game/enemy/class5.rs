//! The Butcher in his first form, class 5 (handler 0x38DDE, 1,172 bytes): he chops the dead dinosaur with two
//! machetes, makes the talk box and the camera mover, and after 475 frames rewrites his own class word to 61
//! (`class61.rs`). D3c section 3.1, BS08.
//!
//! The handler is a two level jump table: the mode byte `+$04` (0 = the story below, 2 = a plain
//! animation frame that nothing sets, 4 and 6 = free, 0x39052 / 0x39064) and, in mode 0, the step byte
//! `+$05` (0x38E00: 0 init, 2 / 6 / 0xA / 0xE waits, 4 talk, 8 turn, 0xC camera, 0x10 the change). The object makes
//! four helpers in its init (the carcass A51, the machete helper A53 and two knife holders A58, `class51_58.rs`),
//! keeps their handles in `+$A4`, `+$A8`, `+$B0` and `+$B2`, and is never hit (it is never put in a hit list).
//! Owner: enemy port (M6-09).

use super::lib32::put_hi;
use super::Enemy;
use crate::game::{slot_of, Game, Handle, Slot};

/// Own bytes of the starter (the original offsets).
mod o {
    /// `+$A2` word: the wait of the current step.
    pub const WAIT: usize = 0xA2;
    /// `+$A4` long: the carcass A51 (a sign extended RAM address, so 0xFFFF0000 + handle).
    pub const CARCASS: usize = 0xA4;
    /// `+$A8` long: the machete helper A53.
    pub const HELPER: usize = 0xA8;
    /// `+$AC` byte: 1 while the knife holders show (the chop animation of the first and the talk).
    pub const CHOP: usize = 0xAC;
    /// `+$B0` word and `+$B2` word: the two knife holders A58 (variants 0 and 0x100).
    pub const HOLDER_A: usize = 0xB0;
    pub const HOLDER_B: usize = 0xB2;
}

/// A pointer field the way `move.l a6,..` / `move.l a0,..` stores it: the address registers hold a sign extended word.
pub(super) fn ptr(h: Handle) -> u32 {
    0xFFFF_0000 | h as u32
}

impl Game {
    /// 0x38DDE: class 5.
    pub(super) fn enemy_class5(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => self.c5_story(e, h),
            2 => {
                // 0x39052: the animation, the far test, the draw check (in this order).
                self.enemy_step(&mut e.b);
                self.enemy_despawn_if_far(e, 0);
                self.enemy_draw_check(&mut e.b, h);
            }
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x39064
            }
        }
    }

    /// `move.b #n,$4(aN)` on another object (a helper the object made). A stale handle that is no pool A object is ignored.
    fn c5_set_mode(&mut self, handle: Handle, mode: u8) {
        if let Some(Slot::Enemy(i)) = slot_of(handle) {
            self.enemies.slots[i].b.mode = mode;
        }
    }

    /// 0x38DF2: the step table at 0x38E00.
    fn c5_story(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => self.c5_init(e, h),
            2 => self.c5_chop(e, h),
            4 => self.c5_talk(e, h),
            8 => self.c5_turn(e, h),
            0xC => self.c5_camera(e, h),
            0x10 => self.c5_change(e, h),
            // 6, 0xA, 0xE (0x38FE2): the waits.
            _ => self.c5_wait(e, h),
        }
    }

    /// 0x38E12: the init. Sound 0x21, the palette 0x9E, the chop animation, then the four helpers. A full pool ends the
    /// list of helpers early (`bcs` to 0x38F26), the step and the wait are set anyway. No draw check in this frame.
    fn c5_init(&mut self, e: &mut Enemy, h: Handle) {
        self.sound(0x21);
        e.b.live = 2;
        e.b.anim_table = 0x3906A;
        e.m.sw(0xA0, 0);
        e.m.sb(o::CHOP, 0);
        e.b.palette_override = 0x9E;
        self.enemy_floor_collision(e); // 0x12CB4
        e.b.facing = 1;
        self.enemy_anim(&mut e.b, 0xB);
        self.c5_make_helpers(e, h);
        e.m.sw(o::WAIT, 0xB4); // 0x38F26
        e.b.sub = e.b.sub.wrapping_add(2);
    }

    /// 0x38E4A .. 0x38F22: the carcass A51 at (X + 0x50, Y + 0x10, Z + 1) that knows its owner (`+$A4`), the helper A53 at
    /// (X + 0x54, Y + 0x10, Z) with variant 0x100, the facing copy and the draw flags 0x10, and the two knife holders A58
    /// (variants 0 and 0x100, owner in `+$A0`). Each allocation is the pool A allocator 0x483C.
    fn c5_make_helpers(&mut self, e: &mut Enemy, h: Handle) {
        let (x, y, z) = (e.b.xi(), e.b.yi(), e.b.zi());
        let Some(a) = self.enemies.alloc() else { return };
        {
            let c = &mut self.enemies.slots[a];
            c.b.live = 1;
            c.b.k20 = 0xCC;
            put_hi(&mut c.b.x, x.wrapping_add(0x50));
            put_hi(&mut c.b.y, y.wrapping_add(0x10));
            put_hi(&mut c.b.z, z.wrapping_add(1));
            c.b.var26 = 0;
            c.m.sl(0xA4, ptr(h)); // move.l a6,$A4(a0)
        }
        e.m.sl(o::CARCASS, ptr(self.enemies.handle(a)));

        let Some(a) = self.enemies.alloc() else { return };
        {
            let c = &mut self.enemies.slots[a];
            c.b.live = 1;
            c.b.k20 = 0xD4;
            put_hi(&mut c.b.x, x.wrapping_add(0x54));
            put_hi(&mut c.b.y, y.wrapping_add(0x10));
            put_hi(&mut c.b.z, z);
            c.b.var26 = 0x100;
            c.b.dir = e.b.dir;
            c.b.draw_flags = 0x10;
        }
        e.m.sl(o::HELPER, ptr(self.enemies.handle(a)));

        for (var, field) in [(0, o::HOLDER_A), (0x100, o::HOLDER_B)] {
            let Some(a) = self.enemies.alloc() else { return };
            let c = &mut self.enemies.slots[a];
            c.b.live = 1;
            c.b.k20 = 0xE8;
            c.b.var26 = var;
            c.m.sl(0xA0, ptr(h));
            e.m.sw(field, self.enemies.handle(a)); // move.w a0,$B0(a6)
        }
    }

    /// 0x38FFC: the chop wait of 180 frames (`+$A2` counts to 0, then the next step), the sound 0x5F every 32 frames
    /// (the test is on the new count, so also in the frame the wait ends), the animation, the draw check.
    fn c5_chop(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(o::WAIT).wrapping_sub(1);
        e.m.sw(o::WAIT, t);
        if t == 0 {
            e.b.sub = e.b.sub.wrapping_add(2);
        }
        if t & 0x1F == 0 {
            self.sound(0x5F);
        }
        self.enemy_step(&mut e.b);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x38F32: the talk. The sound 0x59 with the pan of his X (0xA2EE), the chop flag, the animation 0xC, the helper
    /// A53 gets mode 6 (it frees itself), the talk box B50 variant 1, a 180 frame wait.
    fn c5_talk(&mut self, e: &mut Enemy, h: Handle) {
        let pan = self.snd_pan_x(e.b.xi()) as u32;
        self.snd_request(0x59, pan, 0); // 0xA2EE
        e.m.sb(o::CHOP, 1);
        self.enemy_anim(&mut e.b, 0xC);
        self.c5_set_mode(e.m.w(o::HELPER + 2), 6);
        self.spawn_prop(50, 1, None); // 0x4860, class 0xC8
        e.m.sw(o::WAIT, 0xB4);
        e.b.sub = e.b.sub.wrapping_add(2);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x38F82: he turns to the hero (animation 0xD), the carcass starts to slide away (`+$A2` of A51), a 60 frame wait.
    fn c5_turn(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_anim(&mut e.b, 0xD);
        if let Some(Slot::Enemy(i)) = slot_of(e.m.w(o::CARCASS + 2)) {
            self.enemies.slots[i].m.sb(0xA2, 1);
        }
        e.m.sw(o::WAIT, 0x3C);
        e.b.sub = e.b.sub.wrapping_add(2);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x38FA4: the chop flag off, the animation 0xE and the camera mover B76 (variant 0), a 60 frame wait.
    fn c5_camera(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sb(o::CHOP, 0);
        self.enemy_anim(&mut e.b, 0xE);
        self.spawn_prop(76, 0, None); // 0x4860, class 0x130
        e.m.sw(o::WAIT, 0x3C);
        e.b.sub = e.b.sub.wrapping_add(2);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x38FE2: a wait (the steps 6, 0xA and 0xE): `+$A2` counts down, at 0 the next step; the animation and the draw
    /// check run in every frame.
    fn c5_wait(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.w(o::WAIT).wrapping_sub(1);
        e.m.sw(o::WAIT, t);
        if t == 0 {
            e.b.sub = e.b.sub.wrapping_add(2);
        }
        self.enemy_step(&mut e.b);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x3902E: the end. Both knife holders get mode 4 (they free themselves in their next update), the class word becomes
    /// 0xF4 (class 61) and the whole state long is cleared: the class 61 init runs in the next frame.
    fn c5_change(&mut self, e: &mut Enemy, h: Handle) {
        for field in [o::HOLDER_A, o::HOLDER_B] {
            self.c5_set_mode(e.m.w(field), 4);
        }
        e.b.k20 = 0xF4;
        e.b.set_state(0);
        self.enemy_draw_check(&mut e.b, h);
    }
}
