//! Pool B class 33 (handler 0x818E4): the starter of the Slisaur fight (Episode 7 area 3, script record at trigger 0x3C0). It makes two
//! Slisaurs (pool A class 71) at X 0x580, Z 0x30 and 0x40, a third at Z 0x20 with two or three players (G.$4EF), and keeps their handles
//! (`+$A0`, `+$A2`, `+$A4`, the count `+$AA`); after the scene (G.$4FC) the reinforcement controller, sound 0x11; then every frame the boss
//! flag, the sum of their HP, G.$502 every 300 frames and the HUD bar on the first one alive; when the Slisaurs' hit handler (0x81A7A,
//! `enemy/class71.rs`) has cleared all three handles: the stage end (0xACEE) with the last body, the music fade, the leader flags. D3d 4.2.

use super::Prop;
use crate::game::{slot_of, Game, Handle, Slot};

impl Game {
    /// 0x818E4 (mode table 0x818F0): 0 by `+$05` (0x81906: 0 the creation 0x8190A, 2 the wait 0x8199E), 2 the run 0x819BE, 4 the leader
    /// flags 0x81A5E, 6 the free 0x81A74.
    pub(crate) fn prop_b33(&mut self, p: &mut Prop, h: Handle, i: usize) {
        match p.b.mode {
            0 if p.b.sub == 0 => self.prop_b33_create(p, h),
            0 => {
                // 0x8199E
                if self.spawn.g.b4fc == 0 {
                    self.boss_spawn_controller(h); // 0x7BEDC
                    p.m.sb(0x7D, 0xFF);
                    self.sound(0x11);
                    p.b.mode = p.b.mode.wrapping_add(2);
                }
            }
            2 => self.prop_b33_run(p),
            4 => {
                // 0x81A5E: the leader flags (0x6148) of the last body.
                p.b.mode = p.b.mode.wrapping_add(2);
                if let Some(Slot::Enemy(k)) = slot_of(p.m.w(0xA8)) {
                    let who = self.enemies.slots[k].b.who;
                    self.enemy_boss_leader(who);
                }
            }
            _ => self.prop_free(p, i), // 0x4938
        }
    }

    /// 0x8190A: `+$05` 2; the Slisaurs (a full pool stops the creation where it is).
    fn prop_b33_create(&mut self, p: &mut Prop, h: Handle) {
        p.b.sub = p.b.sub.wrapping_add(2);
        for (off, z) in [(0xA0, 0x30i16), (0xA2, 0x40)] {
            if !self.prop_b33_slisaur(p, h, off, z) {
                return;
            }
        }
        p.m.sb(0xAA, 2);
        if self.w.players_minus_one != 0 {
            if !self.prop_b33_slisaur(p, h, 0xA4, 0x20) {
                return;
            }
            p.m.sb(0xAA, 3);
        }
    }

    /// One Slisaur (0x483C): live, class 71, its handle in `off`, X 0x580, Z `z`, `+$C6` = the starter.
    fn prop_b33_slisaur(&mut self, p: &mut Prop, h: Handle, off: usize, z: i16) -> bool {
        let Some(j) = self.enemies.alloc() else { return false };
        let nh = self.enemies.handle(j);
        let n = &mut self.enemies.slots[j];
        n.b.live = 1;
        n.b.k20 = 71 * 4;
        p.m.sw(off, nh);
        let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
        put(&mut n.b.x, 0x580);
        put(&mut n.b.z, z);
        n.m.sw(0xC6, h);
        true
    }

    /// 0x819BE: G.$7B4 2; the HP sum (0x81A06); `+$A6` down, at its end 300 again and G.$502 1; the HUD bar (0x81AF4); all handles 0: mode
    /// 4, 0xACEE with the last body (`+$A8`), the music fade (0xA5EA).
    fn prop_b33_run(&mut self, p: &mut Prop) {
        self.cam.s2.lock78 = 2;
        let (mut hp, mut hp6e) = (0u16, 0u16);
        for off in [0xA0, 0xA2, 0xA4] {
            if let Some(Slot::Enemy(k)) = slot_of(p.m.w(off)) {
                let b = &self.enemies.slots[k].b;
                hp = hp.wrapping_add(b.hp);
                hp6e = hp6e.wrapping_add(b.hp_prev);
            }
        }
        p.b.hp = hp;
        p.b.hp_prev = hp6e;
        let t = p.m.w(0xA6);
        p.m.sw(0xA6, t.wrapping_sub(1));
        if t == 0 {
            p.m.sw(0xA6, 0x12C);
            self.spawn.g.b502 = 1;
        }
        for off in [0xA0, 0xA2, 0xA4] {
            let a = p.m.w(off);
            if a != 0 {
                self.hud.boss = a; // G.-$70BC
                break;
            }
        }
        if p.m.w(0xA0) | p.m.w(0xA2) | p.m.w(0xA4) == 0 {
            p.b.mode = p.b.mode.wrapping_add(2);
            self.flow_boss_defeated(p.m.w(0xA8)); // 0xACEE, a6 = the last body
            self.hook_a5ea(); // 0xA5EA
        }
    }
}
