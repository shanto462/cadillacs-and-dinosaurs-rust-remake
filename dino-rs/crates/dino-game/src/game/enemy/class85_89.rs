//! The five parts of Fessenden form 2 (class 84, `class84.rs`), made with it from the sorted slots (0x6175A); each keeps the body in `+$B0`.
//! * 85 (0x63778): the body layer: it copies the body's place, palette and frame and draws it (the body takes itself off the draw list).
//! * 86 (0x63806, table 0x647D8): the arm or tail; the body's request `+$A2` 2 / 4 / 6 starts the strikes 0x5F / 0x60 / 0x61 (animations 2,
//!   3, 4), 8 the grab pose (animation 5).
//! * 87 (0x63BC4, table 0x648AA): the second limb; it only follows the grab pose (`+$A3` 8, animation 1).
//! * 88 / 89 (0x63E30, table 0x646A4, `+$5C` 0 left / -1 right): the heads; the body's `+$A4` / `+$A5` 2 the bite 0x5E (repeated while it hits,
//!   up to `+$AE` times), 4 the fire (a class 95 ball at the frames with `+$2C`, its variant by the distance), 6 the sway.
//! The arm, limb and heads take their place and animation from the body's frame byte `+$2D` (rows of 6 bytes: X, Y, Z offsets and the
//! animation, 0x642DE / 0x64328), copy its live and visible bytes, hit id, HP and palette every frame (0x638CE), are on the hurtable list and a
//! hit on one is a hit on the body (0x62F12). D3d 6.3, BT16, BT18. Owner: enemy port.
//!
//! Not ported: the test-mode HUD of the parts (0x63B36, 0x63DA2, 0x64366, with G.$4AA and the service switch).

use super::{off::*, Enemy};
use crate::game::{slot_of, Game, Handle, Slot};

/// The rows by the body's frame byte (6 bytes: X word, Y word, Z byte, the animation byte).
const ARM_ROWS: u32 = 0x648EA;
const LIMB_ROWS: u32 = 0x6496E;
const HEAD_L_ROWS: u32 = 0x649F2;
const HEAD_R_ROWS: u32 = 0x64A76;
/// 0x64F5C (words by the X distance / 32) and 0x64F82 (signed bytes by `rand & 31`): the fireball's variant.
const FIRE_DIST: u32 = 0x64F5C;
const FIRE_JITTER: u32 = 0x64F82;

/// Own bytes: `+$A8` the action (part), `+$AA` the body's frame byte, `+$AB` the animation of the row (negative: none, the part is not drawn).
const ACTION: usize = 0xA8;
const ROW_FRAME: usize = 0xAA;
const ROW_ANIM: usize = 0xAB;

impl Game {
    /// The body (`+$B0`) as a pool A index.
    fn part_body(e: &Enemy) -> Option<usize> {
        match slot_of(e.m.w(0xB0)) {
            Some(Slot::Enemy(k)) => Some(k),
            _ => None,
        }
    }

    /// A byte of the body.
    fn part_body_b(&self, e: &Enemy, field: usize) -> u8 {
        Self::part_body(e).map(|k| self.enemies.slots[k].m.b(field)).unwrap_or(0)
    }

    /// 0x63778: the body layer (mode table 0x63784): 0 the init 0x6378C (`+$7D` and `+$63` 0xFF, in the air, visible), 2 the copy 0x637AA,
    /// 4 and 6 the free.
    pub(super) fn enemy_class85(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                e.m.sb(ATTACKING, 0xFF);
                e.b.b63 = 0xFF;
                e.b.airborne = 1;
                e.b.visible = 1;
                e.b.mode = e.b.mode.wrapping_add(2);
            }
            2 => {
                // 0x637AA: nothing once the body is hidden (`+$A6`); the place (words), palette, frame, `+$2C..+$2F`, facing and draw flags,
                // `+$4C..+$4F`; drawn (0x49C0) unless the body is in its grab with `+$06` set.
                let Some(k) = Self::part_body(e) else { return };
                let b = &self.enemies.slots[k];
                if b.m.b(0xA6) != 0 {
                    return;
                }
                let put = |v: &mut i32, w: i32| *v = (*v & 0xFFFF) | (w & !0xFFFF);
                put(&mut e.b.x, b.b.x);
                put(&mut e.b.y, b.b.y);
                put(&mut e.b.z, b.b.z);
                e.b.palette_override = b.b.palette_override;
                e.b.set_frame(b.b.frame);
                e.b.param_a = b.b.param_a;
                e.b.param_b = b.b.param_b;
                e.b.kind = b.b.kind;
                e.b.timer = b.b.timer;
                e.b.facing = b.b.facing;
                e.b.draw_flags = b.b.draw_flags;
                e.b.extra = b.b.extra;
                if b.b.mode == 2 && b.b.sub == 4 && b.b.act != 0 {
                    return;
                }
                self.enemy_draw_visible(&e.b, h);
            }
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// The common init of the arm, limb and heads (0x6381A / 0x63BD8 / 0x63E44): name 0x22, `+$84` 0x88, the table, the box table 0x102400,
    /// poise 0x32, guard 2, `+$7D` and `+$63` 0xFF, `+$72` 0xC, draw flags 0x10, in the air, `+$A8` 0, animation 0, `+$35` 0xFF, the target.
    fn part_init(&mut self, e: &mut Enemy, table: u32) {
        e.b.id = 0x22;
        e.m.sw(W84, 0x88);
        e.b.anim_table = table;
        e.b.box_table = 0x102400;
        e.b.multi_box = false;
        e.b.stun = 0x32;
        e.m.sb(0xBD, 2);
        e.m.sb(POISE_T, 0);
        e.m.sb(WAIT, 0);
        e.m.sb(B87, 0);
        e.b.hitstop = 0;
        e.m.sb(ATTACKING, 0xFF);
        e.b.b72 = 0xC;
        e.b.draw_flags = 0x10;
        e.b.b63 = 0xFF;
        e.b.airborne = 1;
        e.m.sb(ACTION, 0);
        self.enemy_anim(&mut e.b, 0);
        e.m.sb(0x35, 0xFF);
        let t = self.enemy_pick_target(&e.b);
        e.m.sw(TARGET, t);
        e.b.mode = e.b.mode.wrapping_add(2);
    }

    /// 0x642DE: the place by the body's frame byte: X + the row's word (mirrored by the body's facing), Y + word, Z + byte; the body's facing.
    fn part_place(&self, e: &mut Enemy, rows: u32) {
        let Some(k) = Self::part_body(e) else { return };
        let b = &self.enemies.slots[k].b;
        let off = (b.param_b as u32) * 6;
        let tb = &self.assets.enemy.t;
        let mut dx = tb.i16(rows, off);
        if b.facing != 0 {
            dx = dx.wrapping_neg();
        }
        let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
        put(&mut e.b.x, b.xi().wrapping_add(dx));
        put(&mut e.b.y, b.yi().wrapping_add(tb.i16(rows, off + 2)));
        put(&mut e.b.z, b.zi().wrapping_add(tb.i8(rows, off + 4) as i16));
        e.b.facing = b.facing;
    }

    /// 0x64328: the row's animation: a new one (not negative) starts; true (carry) when it did.
    fn part_sync(&mut self, e: &mut Enemy, rows: u32) -> bool {
        let Some(k) = Self::part_body(e) else { return false };
        let f = self.enemies.slots[k].b.param_b;
        e.m.sb(ROW_FRAME, f);
        let a = self.assets.enemy.t.u8(rows, f as u32 * 6 + 5);
        if a == e.m.b(ROW_ANIM) {
            return false;
        }
        e.m.sb(ROW_ANIM, a);
        if (a as i8) < 0 {
            return false;
        }
        self.enemy_anim(&mut e.b, a as u16);
        true
    }

    /// 0x638CE: the live and visible bytes of the body, its hit id (while this part is not hit), HP, HP before, the bar and the palette;
    /// with a row animation the hurtable list and the draw (0x49C0).
    fn part_mirror(&mut self, e: &mut Enemy, h: Handle) {
        if let Some(k) = Self::part_body(e) {
            let b = &self.enemies.slots[k].b;
            e.b.live = b.live;
            e.b.visible = b.visible;
            if e.b.live & 1 == 0 {
                e.b.serial = b.serial;
            }
            e.b.hp = b.hp;
            e.b.hp_prev = b.hp_prev;
            e.b.hp_bar = b.hp_bar;
            e.b.palette_override = b.palette_override;
        }
        if (e.m.b(ROW_ANIM) as i8) >= 0 {
            self.enemy_push_hurtable(h);
            self.enemy_draw_visible(&e.b, h);
        }
    }

    /// The hit of a part (0x63B9C / 0x63E08 / 0x6441E): the row's animation, live 1, state 0x02000000, no hit stop (the body took the hit).
    fn part_hit(&mut self, e: &mut Enemy, rows: u32) {
        self.part_sync(e, rows);
        e.b.live = 1;
        e.b.set_state(0x0200_0000);
        e.b.hitstop = 0;
    }

    // ------------------------------------------------------------------ 86, the arm

    /// 0x63806 (mode table 0x63812): 0 the init, 2 the frame 0x6389E (by `+$05`, table 0x6391C: 0 the action, 2 the hit, 4 and 6 the free),
    /// 4 and 6 the free.
    pub(super) fn enemy_class86(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.part_init(e, 0x647D8);
                e.m.sb(0xA2, 0);
            }
            2 => {
                self.part_place(e, ARM_ROWS);
                match e.b.sub {
                    0 => self.arm_action(e),
                    2 => {
                        self.part_hit(e, ARM_ROWS);
                        Self::arm_reset(e);
                    }
                    _ => return self.enemy_free_pool_a(e, i),
                }
                self.part_mirror(e, h);
            }
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x63944: no action.
    fn arm_reset(e: &mut Enemy) {
        e.m.sb(ACTION, 0);
        e.m.sb(0xA2, 0);
        e.b.act = 0;
        e.b.step = 0;
    }

    /// 0x6392C by `+$A8` (table 0x6393A): 0 the idle (0x63956: animation 0, then the body's request starts an action, 0x63B12), 2 / 4 / 6 the
    /// strikes (attack 0x5F / 0x60 / 0x61, animation 2 / 3 / 4, then the idle), 8 the grab pose (animation 5 while the body's `+$A2` is 8).
    fn arm_action(&mut self, e: &mut Enemy) {
        self.part_sync(e, ARM_ROWS);
        let a = e.m.b(ACTION);
        match a {
            0 => {
                if e.b.act == 0 {
                    e.b.act = 2;
                    return self.enemy_anim(&mut e.b, 0);
                }
                let req = self.part_body_b(e, 0xA2);
                if req == 0 {
                    self.enemy_step(&mut e.b);
                } else if matches!(req, 2 | 4 | 6 | 8) {
                    e.m.sb(ACTION, req);
                    e.m.sb(0xA2, req);
                    e.b.act = 0;
                    e.b.step = 0;
                }
            }
            2 | 4 | 6 => {
                if e.b.act == 0 {
                    e.b.act = 2;
                    let k = (a / 2) as u16;
                    self.enemy_begin_attack(&mut e.b, 0x5E + k as u8);
                    return self.enemy_anim(&mut e.b, 1 + k);
                }
                if self.enemy_step(&mut e.b) {
                    Self::arm_reset(e);
                }
            }
            _ => {
                if e.b.act == 0 {
                    e.b.act = 2;
                    return self.enemy_anim(&mut e.b, 5);
                }
                self.enemy_step(&mut e.b);
                if self.part_body_b(e, 0xA2) != 8 {
                    Self::arm_reset(e);
                }
            }
        }
    }

    // ------------------------------------------------------------------ 87, the second limb

    /// 0x63BC4 (mode table 0x63BD0): 0 the init, 2 the frame 0x63C5C (by `+$05`, table 0x63CDA: 0 the action, 2 the hit, 4 and 6 the free).
    pub(super) fn enemy_class87(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                self.part_init(e, 0x648AA);
                e.m.sb(0xA2, 0);
            }
            2 => {
                self.part_place(e, LIMB_ROWS);
                match e.b.sub {
                    0 => self.limb_action(e),
                    2 => {
                        self.part_hit(e, LIMB_ROWS);
                        Self::limb_reset(e);
                    }
                    _ => return self.enemy_free_pool_a(e, i),
                }
                self.part_mirror(e, h);
            }
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// 0x63D02: no action.
    fn limb_reset(e: &mut Enemy) {
        e.m.sb(ACTION, 0);
        e.b.act = 0;
        e.b.step = 0;
    }

    /// 0x63CEA by `+$A8` (table 0x63CF8): 0 .. 6 the idle (0x63D0E: animation 0; with the body's `+$A3` set and its `+$A2` 8 the pose), 8 the
    /// pose (0x63D58: animation 1 while the body's `+$A3` is 8).
    fn limb_action(&mut self, e: &mut Enemy) {
        self.part_sync(e, LIMB_ROWS);
        if e.m.b(ACTION) != 8 {
            if e.b.act == 0 {
                e.b.act = 2;
                return self.enemy_anim(&mut e.b, 0);
            }
            if self.part_body_b(e, 0xA3) != 0 {
                if self.part_body_b(e, 0xA2) == 8 {
                    e.m.sb(ACTION, 8);
                    e.b.act = 0;
                    e.b.step = 0;
                }
                return;
            }
            self.enemy_step(&mut e.b);
            return;
        }
        if e.b.act == 0 {
            e.b.act = 2;
            return self.enemy_anim(&mut e.b, 1);
        }
        if self.part_body_b(e, 0xA3) != 8 {
            return Self::limb_reset(e);
        }
        self.enemy_step(&mut e.b);
    }

    // ------------------------------------------------------------------ 88 and 89, the heads

    /// 0x63E30 (mode table 0x63E3C): 0 the init, 2 the frame 0x63ECC (the rows 0x649F2 left, 0x64A76 right; by `+$05`, table 0x63F56: 0 the
    /// action, 2 the hit, 4 and 6 the free).
    pub(super) fn enemy_class88(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        let rows = if e.m.w(W5C) == 0 { HEAD_L_ROWS } else { HEAD_R_ROWS };
        match e.b.mode {
            0 => {
                self.part_init(e, 0x646A4);
                e.m.sb(0xA4, 0);
                e.m.sb(0xA5, 0);
            }
            2 => {
                self.part_place(e, rows);
                match e.b.sub {
                    0 => self.head_action(e, h, rows),
                    2 => {
                        self.part_hit(e, rows);
                        Self::head_reset(e);
                    }
                    _ => return self.enemy_free_pool_a(e, i),
                }
                self.part_mirror(e, h);
            }
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// The body's request byte for this head (`+$A4` left, `+$A5` right) and this head's own.
    fn head_req(e: &Enemy) -> usize {
        if e.m.b(W5C) == 0 {
            0xA4
        } else {
            0xA5
        }
    }

    /// 0x63F7C: no action, the head's own request cleared.
    fn head_reset(e: &mut Enemy) {
        e.m.sb(ACTION, 0);
        e.b.act = 0;
        e.b.step = 0;
        e.m.sb(Self::head_req(e), 0);
    }

    /// 0x63F66 by `+$A8` (table 0x63F74): 0 the idle (0x63F9E), 2 the bite (0x64030), 4 the fire (0x640FC), 6 the sway (0x641A0).
    fn head_action(&mut self, e: &mut Enemy, h: Handle, rows: u32) {
        let rq = Self::head_req(e);
        match e.m.b(ACTION) {
            0 => {
                self.part_sync(e, rows);
                if e.b.act == 0 {
                    e.b.act = 2;
                    e.m.sb(ROW_ANIM, 0);
                    return self.enemy_anim(&mut e.b, 0);
                }
                // 0x642B4: the body's request starts an action (the repeats `+$AE` copied for the bite and the fire).
                let req = self.part_body_b(e, rq);
                if req == 0 {
                    self.enemy_step(&mut e.b);
                    return;
                }
                if matches!(req, 2 | 4 | 6) {
                    e.m.sb(ACTION, req);
                    e.b.act = 0;
                    e.b.step = 0;
                    if req != 6 {
                        e.m.sb(0xAE, self.part_body_b(e, 0xAE));
                    }
                    e.m.sb(rq, req);
                }
            }
            2 => {
                if self.part_sync(e, rows) || self.part_body_b(e, rq) != 2 {
                    return Self::head_reset(e);
                }
                if e.b.act == 0 {
                    e.b.act = 2;
                    self.enemy_begin_attack(&mut e.b, 0x5E);
                    return self.enemy_anim(&mut e.b, 1);
                }
                if !self.enemy_step(&mut e.b) {
                    return;
                }
                if e.b.hitstop != 0 {
                    let n = e.m.b(0xAE).wrapping_sub(1);
                    e.m.sb(0xAE, n);
                    if n != 0 {
                        e.b.act = 0;
                        return;
                    }
                }
                e.m.sb(0xAE, 0);
                Self::head_reset(e);
            }
            4 => {
                if self.part_sync(e, rows) || self.part_body_b(e, rq) != 4 {
                    return Self::head_reset(e);
                }
                if e.b.act == 0 {
                    e.b.act = 2;
                    return self.enemy_anim(&mut e.b, 5);
                }
                if self.enemy_step(&mut e.b) {
                    return Self::head_reset(e);
                }
                if e.b.param_a != 0 {
                    e.b.param_a = 0;
                    self.head_fire(e, h);
                }
            }
            _ => {
                self.part_sync(e, rows);
                if self.part_body_b(e, rq) != 6 {
                    return Self::head_reset(e);
                }
                if e.b.act == 0 && e.m.w(W5C) != 0 && self.part_body_b(e, 0xAC) & 2 != 0 {
                    self.enemy_anim(&mut e.b, 0);
                }
            }
        }
    }

    /// 0x64210: the fire: sound 0x69 (0xA1DC), the muzzle flash 0x13 (0x92422) at the mouth (X -62 / +62, Y -25, Z -1), and a class 95 ball
    /// (0x69932) there with the variant T1[body to its target's X distance / 32] + T2[`rand & 31`], kept in 0 .. 8.
    fn head_fire(&mut self, e: &mut Enemy, _h: Handle) {
        let side: u32 = if e.b.facing != 0 { 6 } else { 5 };
        self.sound_ex(0x69, (side << 16) | 0x2000 | self.snd_pan_x(e.b.xi()) as u32, 0x0240_0200);
        let dx: i16 = if e.b.facing != 0 { 0x3E } else { -0x3E };
        let (x, y, z) = (e.b.xi().wrapping_add(dx), e.b.yi().wrapping_sub(0x19), e.b.zi().wrapping_sub(1));
        self.fx_spawn_spark_facing(e.b.facing, x, y, z, 0x13);
        let d = Self::part_body(e).map(|k| {
            let b = &self.enemies.slots[k];
            let tx = self.body(b.m.w(TARGET)).xi();
            let mut d = b.b.xi().wrapping_sub(tx);
            if d < 0 {
                d = d.wrapping_neg();
            }
            ((d as u16) & 0x3E0) >> 4
        });
        let tb = &self.assets.enemy.t;
        let mut v = tb.i16(FIRE_DIST, d.unwrap_or(0) as u32);
        let r = (self.rand() & 0x1F) as u32;
        v = v.wrapping_add(self.assets.enemy.t.i8(FIRE_JITTER, r) as i16);
        if v < 0 {
            v = 0;
        }
        if (v as u16 as u8) > 8 {
            v = 8;
        }
        self.c95_spawn(x, y, z, v as u16, e.b.facing);
    }
}
