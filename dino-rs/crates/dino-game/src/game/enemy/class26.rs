//! Pool A class 26 (handler 0x4D48A..0x4D900, table 0x71408): the thrown fire of the bosses. Variants (`+$26`): 0 to 2 Morgan's grenades (they
//! fly to a spot near the target and explode, kind 3 of 0x27DD8, or drop a weapon item), 3 Tyrog's fireball (falls while it slides 8 px per
//! frame back or 6.5 forward, then burns on the ground), 4 Tyrog's lava rock (palette 0x81, thrown along its direction `+$22`, burns where it
//! lands). The spot is the target's position plus one of a cross of offsets (0x4D49E / 0x4D51E by `rand() & 31`). D3d 2.4, 3.5, D3f class 26.
//! Owner: enemy port.

use super::{off::*, Enemy};
use crate::game::item::{class as item_class, Item};
use crate::game::{slot_of, Game, Handle, Slot};

/// 0x4D49E / 0x4D51E: the X and Z offsets of the landing spot (32 longs each, 16.16).
const SPOT_X: u32 = 0x4D49E;
const SPOT_Z: u32 = 0x4D51E;

impl Game {
    /// 0x4D48A (mode table 0x4D496): 0 the init 0x4D59E, 2 the run 0x4D79A, 4 and 6 the free 0x4D8FA.
    pub(super) fn enemy_class26(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => self.c26_init(e, h),
            2 => self.c26_run(e, h, i),
            _ => self.enemy_free_pool_a(e, i),
        }
    }

    /// The thrower (`+$A0`, a long; its low word is the handle).
    fn c26_owner(e: &Enemy) -> Option<usize> {
        match slot_of(e.m.l(0xA0) as u16) {
            Some(Slot::Enemy(k)) => Some(k),
            _ => None,
        }
    }

    /// 0x4D59E: attack 0x44, `+$00` 2, not counted, nobody credited, `+$63` 0xFF, the table 0x71408; the thrower's target; the spot (`+$B0`,
    /// `+$B4`) = the target plus the offsets of `rand() & 31`; the terrain; in the air. Variants 0 to 2: the speeds = the distances / 0x3800
    /// (counted), vy 0x700, gravity -0x40, animation 0 (variant 0) or 1. Variant 4: palette 0x81, speed id 9 along `+$22`, vy 0x400, gravity
    /// -0x40, attack 0x4E, animation 0xA. Variant 3: vx 0x680 (-0x800 when the thrower faces left), no Y speed, gravity -0x38, attack 0x67,
    /// animation 0xC. The X and Z speeds turn round when the spot (d2, d3) is left of / nearer than the object; variant 4 never sets d2 and d3,
    /// the terrain routine leaves small values there, so both turn round.
    fn c26_init(&mut self, e: &mut Enemy, h: Handle) {
        e.m.sb(B87, 0);
        e.m.sb(0xAF, 0);
        e.b.hitstop = 0;
        e.b.box_table = 0x103C00;
        e.b.multi_box = false;
        self.enemy_begin_attack(&mut e.b, 0x44);
        e.b.live = 2;
        e.m.sb(ATTACKING, 0xFF);
        e.b.who = 0xFF;
        e.b.b63 = 0xFF;
        e.b.anim_table = 0x71408;
        e.m.sb(0xA5, 0);
        e.m.sw(0xBA, 0);
        let target = Self::c26_owner(e).map(|k| self.enemies.slots[k].m.w(TARGET)).unwrap_or(0);
        e.m.sw(TARGET, target);
        let (tx, tz) = if slot_of(target).is_some() {
            let b = self.body(target);
            (b.x, b.z)
        } else {
            (0, 0)
        };
        let r = (self.rand() & 0x1F) as u32 * 4;
        let sx = tx.wrapping_add(self.assets.enemy.t.u32(SPOT_X, r) as i32);
        let sz = tz.wrapping_add(self.assets.enemy.t.u32(SPOT_Z, r) as i32);
        e.m.sl(0xB0, sx as u32);
        e.m.sl(0xB4, sz as u32);
        if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
            e.m.sw(OBSTACLE, ob);
        }
        e.b.mode = e.b.mode.wrapping_add(2);
        e.b.act = 0;
        e.b.airborne = 1;
        let var = e.b.var26;
        let (d2, d3) = if var == 4 {
            e.b.palette_override = 0x81;
            self.enemy_set_speed(&mut e.b, 9);
            e.b.vy = 0x400;
            e.b.ax = 0;
            e.b.ay = -0x40;
            e.b.az = 0;
            self.enemy_begin_attack(&mut e.b, 0x4E);
            (i32::MIN, i32::MIN) // below any X and Z: both speeds turn round
        } else {
            e.b.vx = Self::c26_steps(sx.wrapping_sub(e.b.x));
            e.b.vz = Self::c26_steps(sz.wrapping_sub(e.b.z));
            e.b.vy = 0x700;
            e.b.ax = 0;
            e.b.ay = -0x40;
            e.b.az = 0;
            (sx, sz)
        };
        if d2 < e.b.x {
            e.b.vx = e.b.vx.wrapping_neg();
        }
        if d3 < e.b.z {
            e.b.vz = e.b.vz.wrapping_neg();
        }
        let anim = match var {
            4 => 0xA,
            3 => {
                e.b.vx = 0x680;
                e.b.vy = 0;
                e.b.vz = 0;
                e.b.ax = 0;
                e.b.ay = -0x38;
                e.b.az = 0;
                self.enemy_begin_attack(&mut e.b, 0x67);
                let left = Self::c26_owner(e).map(|k| self.enemies.slots[k].b.facing == 0).unwrap_or(true);
                if left {
                    e.b.vx = -0x800;
                }
                0xC
            }
            0 => 0,
            _ => 1,
        };
        self.enemy_anim(&mut e.b, anim);
        self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
    }

    /// The counted division by 0x3800 (n + 1 steps of |d|).
    fn c26_steps(d: i32) -> i16 {
        let mut d0 = if d < 0 { d.wrapping_neg() } else { d };
        let mut n: u16 = 0;
        loop {
            n = n.wrapping_add(1);
            d0 = d0.wrapping_sub(0x3800);
            if d0 < 0 {
                return n as i16;
            }
        }
    }

    /// 0x4D79A by `+$05` (the long table 0x4D7A8): 0 the flight / burn by `+$06` (0x4D7C6: 0 0x4D7CE, 2 0x4D8E4), 2 .. 6 a hit (0x4D850).
    fn c26_run(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.sub != 0 {
            return self.c26_hit(e, h);
        }
        if e.b.act != 0 {
            return self.c26_burn(e, h, i);
        }
        // 0x4D7CE: grenades in a hit stop take the hit path.
        if (e.b.var26 as i16) < 3 && e.b.hitstop != 0 {
            return self.c26_hit(e, h);
        }
        e.b.motion();
        if let Some(ob) = self.terrain_collide(&mut e.b, h, false) {
            e.m.sw(OBSTACLE, ob);
        }
        if e.b.yi() >= 0 && e.b.airborne != 0 {
            // 0x4D8C2: in flight.
            self.enemy_step(&mut e.b);
            e.m.sw(0xBA, e.m.w(0xBA).wrapping_add(1));
            self.enemy_push_hurtable(h);
            return self.enemy_draw_check(&mut e.b, h);
        }
        // 0x4D804: on the ground.
        match e.b.var26 {
            4 | 3 => {
                if e.b.var26 == 4 {
                    self.enemy_anim(&mut e.b, 0xB);
                } else {
                    self.snd_rumble(e.b.xi(), e.b.facing); // 0x9F12
                    self.enemy_anim(&mut e.b, 0xD);
                }
                e.b.box_table = 0x103000;
                e.b.multi_box = false;
                e.b.act = 2;
                self.c26_burn(e, h, i);
            }
            0 => self.c26_hit(e, h),
            _ => {
                // 0x4D872: a weapon item (pool E class 0, variant 0x18, `+$07` 2) where it lands; mode 6.
                self.enemy_anim(&mut e.b, 0);
                let pos = (e.b.xi(), e.b.yi(), e.b.zi());
                if let Some(j) = self.item_spawn(item_class::WEAPON * 4, pos, Some(0x18)) {
                    let it = &mut self.items.slots[j];
                    it.b.act = 0; // built by hand: no act 2
                    it.b.step = 2;
                }
                e.b.mode = 6;
            }
        }
    }

    /// 0x4D850: fire (variants 3 and 4) ignores it (`+$05` 0); a grenade explodes (0x27DD8 with kind 3) and goes (mode 6).
    fn c26_hit(&mut self, e: &mut Enemy, h: Handle) {
        if (e.b.var26 as i16) >= 3 {
            e.b.sub = 0;
            return;
        }
        let src = Item { b: e.b.clone(), ..Default::default() };
        self.item_explosion(&src, h, 3);
        e.b.mode = 6;
    }

    /// 0x4D8E4: the burn: the animation, at its end the free; the hurtable list and the draw check.
    fn c26_burn(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if self.enemy_step(&mut e.b) {
            return self.enemy_free_pool_a(e, i);
        }
        self.enemy_push_hurtable(h);
        self.enemy_draw_check(&mut e.b, h);
    }
}
