//! Bouncing and rolling barrel, pool A class 65 (handler 0x5B8B2..0x5BB89). HP 8, attack 0x32. A hero hit (the monster
//! pass sets sub 2) breaks it on the next frame: wood pieces and a container-table drop, or a metal puff. Owner: enemy port.
//! D3a section 10, H11.

use super::{off::*, Enemy};
use crate::game::{Game, Handle};

/// Words read at run time (`dino-extract` TABLES).
mod t {
    /// 0x5BA70: vx, vy, ay of a hop.
    pub const HOP: u32 = 0x5BB80;
    /// 0x5B912: vy, ay of the cave fall (variant high byte nonzero).
    pub const FALL: u32 = 0x5BB86;
    /// Animation table stored at +$40 (0x5B96A).
    pub const ANIM: u32 = 0x5BAE0;
}

impl Game {
    /// 0x5B8B2 (mode table 0x5B8BE): 0 the init (0x5B8C6, falls into the run), 2 the run (0x5B99C), 4 and 6 free (0x5BADA).
    pub(super) fn enemy_class65(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.enemy_c65_init(e);
                self.enemy_c65_run(e, h, i);
            }
            2 => self.enemy_c65_run(e, h, i),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x5B8C6. The high byte of the variant (+$26) picks the floor hop or the cave fall. Both share 0x5B930.
    fn enemy_c65_init(&mut self, e: &mut Enemy) {
        enemy_c65_face(self, e);
        if e.b.var26 >> 8 == 0 {
            self.enemy_c65_hop_vel(e); // 0x5BA70
            self.enemy_snap_to_floor(e); // 0x3140C
        } else {
            self.enemy_snap_to_floor(e);
            let tb = &self.assets.enemy.t;
            e.b.vy = tb.i16(t::FALL, 0);
            e.b.ay = tb.i16(t::FALL, 2);
            e.b.set_yi_keep(e.b.yi().wrapping_add(0x100)); // addi.w #$100, +$0C
            e.m.sb(T80, 0x1E);
            e.b.act = 2;
        }
        e.b.box_table = 0xC010_0000;
        e.b.multi_box = false;
        e.b.hitstop = 0;
        e.b.who = 0xFF;
        e.b.id = 0;
        e.m.sb(ATTACKING, 0xFF); // not counted
        e.m.sb(B87, 0);
        e.b.hp = 8;
        e.b.hp_prev = 8;
        e.b.hp_bar = 8;
        e.b.airborne = 1;
        e.b.anim_table = t::ANIM;
        let anim = e.b.var26 & 0xFF; // +$27
        self.enemy_anim(&mut e.b, anim); // 0x120E
        e.b.atk = 0x32;
        e.b.knock = 0;
        e.b.hitstop = 0;
        e.b.serial = self.w.attack_serial; // G.$67BE
        self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
        e.b.mode = e.b.mode.wrapping_add(2);
    }

    /// 0x5B99C (longs at 0x5B9A8): sub 0 is alive (0x5B9B8), sub 2, 4 and 6 break (0x5BA8E).
    fn enemy_c65_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.sub == 0 {
            self.enemy_c65_alive(e, h);
        } else {
            self.enemy_c65_break(e, i);
        }
    }

    /// 0x5B9B8: the act (word table 0x5B9DC), then the animation (0x121E), the hostile list (0x4C0A), the near
    /// despawn (0x1760) and the draw (0x49CA).
    fn enemy_c65_alive(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => self.enemy_c65_hop(e),
            2 => self.enemy_c65_fall(e),
            _ => {}
        }
        self.enemy_step(&mut e.b);
        self.enemy_monster_register(h);
        self.enemy_despawn_near(e);
        self.enemy_draw_check(&mut e.b, h);
    }

    /// 0x5B9E0: hop. On the ground, the landing sound, a fresh hop, and 1 in 16 hops a little higher.
    fn enemy_c65_hop(&mut self, e: &mut Enemy) {
        e.b.motion(); // 0x189C
        self.enemy_floor_collision(e); // 0x12CB4
        if e.b.airborne != 0 {
            return;
        }
        if e.b.var26 & 0xFF == 0 {
            self.item_snd_thud(e.b.xi()); // 0x9FB6
        } else {
            self.enemy_c65_snd_alt(e); // 0xA3C6
        }
        self.enemy_c65_hop_vel(e);
        e.b.airborne = 1;
        if self.rand() & 0xF == 0 {
            e.b.vy = e.b.vy.wrapping_add(0x100);
        }
    }

    /// 0x5BA2C: the cave barrel waits in +$80, then falls. The landing doubles the hop's vy and joins the hop.
    fn enemy_c65_fall(&mut self, e: &mut Enemy) {
        let timer = e.m.b(T80);
        if timer != 0 {
            let n = timer.wrapping_sub(1);
            e.m.sb(T80, n);
            if n != 0 {
                return;
            }
        }
        e.b.motion();
        self.enemy_floor_collision(e);
        if e.b.airborne != 0 {
            return;
        }
        self.item_snd_thud(e.b.xi());
        self.enemy_c65_hop_vel(e);
        e.b.vy = e.b.vy.wrapping_add(e.b.vy); // add.w d0, d0
        e.b.airborne = 1;
        e.b.act = 0;
    }

    /// 0x5BA70: vx, vy and ay from 0x5BB80. Facing 0 negates vx.
    fn enemy_c65_hop_vel(&mut self, e: &mut Enemy) {
        let tb = &self.assets.enemy.t;
        let mut vx = tb.i16(t::HOP, 0);
        if e.b.facing == 0 {
            vx = vx.wrapping_neg();
        }
        e.b.vx = vx;
        e.b.vy = tb.i16(t::HOP, 2);
        e.b.ay = tb.i16(t::HOP, 4);
    }

    /// 0xA3C6: the metal landing. Bit 6 of G.$43B picks 0x8B or 0x104. d1 is the pan byte, d2 is 0.
    fn enemy_c65_snd_alt(&mut self, e: &Enemy) {
        let pan = self.snd_pan_x(e.b.xi()) as u32;
        let old = self.snd_toggle_b(6);
        self.snd_request(if old { 0x8B } else { 0x104 }, pan, 0);
    }

    /// 0x5BA8E: a hero hit. Low byte 0 of the variant throws two wood pieces (0x292A6); any other low byte is the
    /// puff 0x2943C (variant 0x1A; the debris pointer in a1 is ignored). Both drop from the container table (0x24B48)
    /// and free the slot (0x48FC).
    fn enemy_c65_break(&mut self, e: &mut Enemy, i: usize) {
        if e.b.var26 & 0xFF == 0 {
            self.item_snd_thud(e.b.xi());
            let mut vx = 0x380i16;
            if e.b.facing == 0 {
                vx = vx.wrapping_neg();
            }
            e.b.vx = vx;
            self.item_two_pieces(&e.b);
        } else {
            self.item_snd_break(e.b.xi()); // 0x9E60
            self.fx_spawn_class21(&e.b, 0x1A, e.b.facing); // 0x2943C -> 0x94640
        }
        self.item_drop_container_enemy(e);
        self.enemy_free_pool_a(e, i);
    }
}

/// 0x5B8CE / 0x5B8F4: facing 1 when `(camera X + 0xC0) >= X` unsigned, else 0.
fn enemy_c65_face(g: &Game, e: &mut Enemy) {
    let cam = (g.cam.s2.cam_x as i16).wrapping_add(0xC0);
    e.b.facing = u8::from((cam as u16) >= (e.b.xi() as u16));
}
