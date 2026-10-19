//! Bouncing rock, pool A class 66 (handler 0x5BB8A..0x5BE63). HP 8, attack 0x46. Two bounces, then a roll toward the
//! camera (act 4, the only state on the hostile list). A hero hit, or the landing of that roll, breaks it into the
//! boulder debris (0x294AA, list 0x29650, effect class 11) and a container-table drop. The stone cap is the debris
//! routine's own count (W225): a bonus is rolled while fewer than 3 have been chosen, and the stone item counts
//! itself later. Owner: enemy port. D3a section 10, H12.

use super::{off::*, Enemy};
use crate::game::{Game, Handle};

/// Words and bytes read at run time (`dino-extract` TABLES). The debris list is the blob at 0x29552.
mod t {
    /// 0x5BBC0: vx, vy, ay.
    pub const SPEED: u32 = 0x5BDFC;
    /// 0x5BCAC: 16 hold times, `rand() & 15`, written to +$A3.
    pub const HOLD: u32 = 0x5BE02;
    /// 0x5BCF8: 16 facings, `rand() & 15`.
    pub const FACE: u32 = 0x5BE13;
    /// 0x5BD3A: 32 vz words, index `rand() & 0x3E`.
    pub const VZ: u32 = 0x5BE24;
    /// Animation table stored at +$40 (0x5BC2E).
    pub const ANIM: u32 = 0x5BDA6;
    /// Boulder debris list (0x5BD7E).
    pub const DEBRIS: u32 = 0x29650;
}

impl Game {
    /// 0x5BB8A (mode table 0x5BB96): 0 the init (0x5BB9E, falls into the run), 2 the run (0x5BC62),
    /// 4 the break (0x5BD6C), 6 free (0x5BDA0).
    pub(super) fn enemy_class66(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.enemy_c66_init(e);
                self.enemy_c66_run(e, h, i);
            }
            2 => self.enemy_c66_run(e, h, i),
            4 => self.enemy_c66_break(e, h, i),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x5BB9E. Faces the camera, stands, then jumps from the speed table with no shadow.
    fn enemy_c66_init(&mut self, e: &mut Enemy) {
        let cam = (self.cam.s2.cam_x as i16).wrapping_add(0xC0);
        let facing = u8::from((cam as u16) >= (e.b.xi() as u16));
        e.b.facing = facing;
        self.enemy_snap_to_floor(e); // 0x3140C
        let tb = &self.assets.enemy.t;
        let spd = tb.i16(t::SPEED, 0);
        e.m.sw(WA0, spd as u16);
        e.b.vx = if facing == 0 { spd.wrapping_neg() } else { spd };
        e.b.vy = tb.i16(t::SPEED, 2);
        e.b.ay = tb.i16(t::SPEED, 4);
        e.b.set_yi_keep(e.b.yi().wrapping_add(0x90)); // addi.w #$90, +$0C
        e.m.sb(WA2, 2); // bounces left, +$A2. +$A3 is left as it was.
        e.b.draw_flags = 0x10; // no shadow. The handler writes this byte and no other of +$25.
        e.b.box_table = 0xC010_0000;
        e.b.multi_box = false;
        e.b.hitstop = 0;
        e.b.who = 0xFF;
        e.b.id = 0;
        e.m.sb(ATTACKING, 0xFF); // not counted
        e.m.sb(B87, 0);
        e.b.b63 = 0xFF;
        e.b.hp = 8;
        e.b.hp_prev = 8;
        e.b.hp_bar = 8;
        e.b.airborne = 1;
        e.b.anim_table = t::ANIM;
        self.enemy_anim(&mut e.b, 0); // 0x120E
        e.b.atk = 0x46;
        e.b.knock = 0;
        e.b.hitstop = 0;
        e.b.serial = self.w.attack_serial; // G.$67BE
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
        e.b.mode = e.b.mode.wrapping_add(2);
        e.b.act = 0;
    }

    /// 0x5BC62 (longs at 0x5BC6E): sub 0 is alive (0x5BC7E). Sub 2, 4 and 6 break (0x5BD6C).
    fn enemy_c66_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.sub == 0 {
            self.enemy_c66_alive(e, h);
        } else {
            self.enemy_c66_break(e, h, i);
        }
    }

    /// 0x5BC7E: the act (word table 0x5BC9C), then the animation (0x121E), the near despawn (0x1760)
    /// and the draw (0x49CA). Act 4 joins the hostile list itself (0x5BD66).
    fn enemy_c66_alive(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.act == 0 {
            // 0x5BCA2 falls straight into the bounce (0x5BCBC) the same frame.
            self.enemy_c66_arm(e);
            self.enemy_c66_bounce(e);
        } else if e.b.act == 2 {
            self.enemy_c66_bounce(e);
        } else if e.b.act == 4 {
            self.enemy_c66_roll(e, h);
        }
        self.enemy_step(&mut e.b);
        self.enemy_despawn_near(e);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x5BCA2: one `rand()`, the hold time in +$A3, and act 2.
    fn enemy_c66_arm(&mut self, e: &mut Enemy) {
        let r = (self.rand() & 0xF) as u32;
        e.m.sb(WA2 + 1, self.assets.enemy.t.u8(t::HOLD, r));
        e.b.act = 2;
    }

    /// 0x5BCBC: move, then bounce on the floor or when the hold time runs out while falling.
    /// The second bounce clears the no-shadow bit, picks vz and switches to the roll.
    fn enemy_c66_bounce(&mut self, e: &mut Enemy) {
        e.b.motion(); // 0x189C
        self.enemy_floor_collision(e); // 0x12CB4
        if e.b.airborne != 0 {
            if e.b.vy >= 0 {
                return;
            }
            let n = e.m.b(WA2 + 1).wrapping_sub(1); // subq.b #1, +$A3
            e.m.sb(WA2 + 1, n);
            if n != 0 {
                return;
            }
        }
        e.b.act = 0;
        e.m.sw(WA0, e.m.w(WA0).wrapping_add(0x80));
        let r = (self.rand() & 0xF) as u32;
        let face = self.assets.enemy.t.u8(t::FACE, r);
        let mut spd = e.m.w(WA0) as i16;
        e.b.facing = face;
        if face == 0 {
            spd = spd.wrapping_neg();
        }
        e.b.vx = spd;
        e.b.vy = e.b.vy.wrapping_neg().wrapping_sub(0x180);
        e.b.airborne = 1;
        let left = e.m.b(WA2).wrapping_sub(1); // subq.b #1, +$A2
        e.m.sb(WA2, left);
        if left != 0 {
            return;
        }
        e.b.draw_flags = 0; // clr.b +$25
        let r = (self.rand() & 0x3E) as u32;
        e.b.vz = self.assets.enemy.t.i16(t::VZ, r);
        e.b.act = 4;
    }

    /// 0x5BD4C: the roll. Landing sets mode 4 (the break is the next frame). Always joins the hostile list.
    fn enemy_c66_roll(&mut self, e: &mut Enemy, h: Handle) {
        e.b.motion(); // 0x189C
        self.enemy_floor_collision(e); // 0x12CB4
        if e.b.airborne == 0 {
            e.b.mode = 4;
        }
        self.enemy_monster_register(h); // jmp 0x4C0A
    }

    /// 0x5BD6C: vx ±0x380 by facing, boulder debris, the container drop, an explosion when +$5C is set, then free.
    fn enemy_c66_break(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        let vx = if e.b.facing == 0 { -0x380 } else { 0x380 };
        e.b.vx = vx;
        self.item_debris(&e.b, t::DEBRIS); // 0x294AA, a1 = 0x29650
        self.item_drop_container_enemy(e); // 0x24B48
        if e.m.w(W5C) != 0 {
            self.item_explosion_body(&e.b, h, 0); // 0x27DD8, d0 = 0
        }
        self.enemy_free_pool_a(e, i); // jmp 0x48FC
    }
}
