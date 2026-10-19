//! Pool B 45, flame vent, 0x83E0C..0x8411C.
use super::{tile_block::TilePos, Prop};
use crate::game::{Game, Handle};

impl Game {
    /// 0x83E0C: init, cycle, hit continuation, destruction and free.
    pub(super) fn prop_b45(&mut self, p: &mut Prop, h: Handle, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode = 2;
                p.b.visible = 1;
                p.hit_init(0x84122, 48);
                p.b.hp_bar = 48;
                p.m.sw(0xAC, self.assets.enemy.t.u16(0x84182, 0));
                p.m.sw(0xAE, self.assets.enemy.t.u16(0x84182, 2));
                p.m.sb(0xAB, self.assets.enemy.t.u8(0x84182, 4));
                self.prop_b45_cycle(p, h, i);
            }
            2 => self.prop_b45_cycle(p, h, i),
            4 => {
                if p.b.hp != 0 {
                    p.b.mode = 2;
                    return;
                }
                self.prop_b45_destroy(p);
            }
            _ => self.prop_free(p, i),
        }
    }

    /// 0x83E60..0x8400E: five cycle states. Signed countdowns include zero.
    fn prop_b45_cycle(&mut self, p: &mut Prop, h: Handle, i: usize) {
        if self.cam.s2.cam_x.wrapping_sub(0x20) >= p.b.xi() as u16 {
            return self.prop_free(p, i);
        }
        self.prop_register_hit(h);
        if p.b.act == 0 {
            match p.b.sub {
                0 | 6 => {
                    let r = (self.rand() & 0x1E) as u32;
                    let table = if p.b.sub == 0 { 0x8413A } else { 0x8415A };
                    p.m.sw(0xA8, self.assets.enemy.t.u16(table, r));
                }
                2 => {
                    p.m.sb(0xAA, 5);
                    p.m.sw(0xA8, 45);
                }
                4 => {
                    p.m.sb(0xAA, p.m.b(0xAB));
                    p.m.sw(0xA8, 0);
                }
                8 => p.m.sw(0xA8, p.m.w(0xAE)),
                _ => return,
            }
            p.b.act = 2;
        }
        match p.b.sub {
            0 | 2 => {
                let n = p.m.w(0xA8).wrapping_sub(1);
                p.m.sw(0xA8, n);
                if (n as i16) < 0 {
                    p.b.sub += 2;
                    p.b.act = 0;
                    return;
                }
                if p.b.sub == 2 && Self::prop_b45_pulse(p, 5) {
                    p.b.dir = (self.rand() & 7) as u8;
                    self.fx_prop_muzzle((p.b.xi(), p.b.yi(), p.b.zi()), 0, 0xA002E, p.b.dir);
                }
            }
            4 | 8 => {
                if !Self::prop_b45_pulse(p, p.m.b(0xAB)) {
                    return;
                }
                let length = if p.b.sub == 4 {
                    p.m.w(0xA8).wrapping_add(p.m.w(0xAC))
                } else {
                    p.m.w(0xA8).wrapping_sub(p.m.w(0xAC))
                };
                p.m.sw(0xA8, length);
                if p.b.sub == 8 && (length as i16) < 0 {
                    p.b.sub = 0;
                    p.b.act = 0;
                    return;
                }
                self.enemy_c83_spawn(&p.b, 0, length);
                if p.b.sub == 4 && length > p.m.w(0xAE) {
                    p.b.sub = 6;
                    p.b.act = 0;
                }
            }
            6 => {
                if Self::prop_b45_pulse(p, p.m.b(0xAB)) {
                    self.enemy_c83_spawn(&p.b, 0, p.m.w(0xAE));
                }
                let n = p.m.w(0xA8).wrapping_sub(1);
                p.m.sw(0xA8, n);
                if (n as i16) < 0 {
                    p.b.sub = 8;
                    p.b.act = 0;
                }
            }
            _ => {}
        }
    }

    /// 0x83F3C, 0x83F9A, 0x8406A: signed byte pulse countdown.
    fn prop_b45_pulse(p: &mut Prop, reload: u8) -> bool {
        let n = p.m.b(0xAA).wrapping_sub(1);
        p.m.sb(0xAA, n);
        if (n as i8) >= 0 {
            return false;
        }
        p.m.sb(0xAA, reload);
        true
    }

    /// 0x84020..0x840E6: broken socket and 120-frame small-flame tail.
    fn prop_b45_destroy(&mut self, p: &mut Prop) {
        if p.b.step == 0 {
            p.m.sb(0xAA, 5);
            p.m.sw(0xA8, 120);
            let dest = self.assets.enemy.t.u32(0x840E8, p.b.var26 as u32);
            for k in 0..8 {
                let addr = dest + (k / 4) * 0x40 + (k % 4) * 4;
                self.prop_tile_code(TilePos::at(addr), self.assets.enemy.t.u16(0x84100, k * 2));
            }
            p.b.dir = (self.rand() & 7) as u8;
            p.m.sb(0xA7, 1);
            p.b.step = 2;
        }
        let n = p.m.w(0xA8).wrapping_sub(1);
        p.m.sw(0xA8, n);
        if (n as i16) < 0 {
            p.b.mode = p.b.mode.wrapping_add(2);
        }
        if Self::prop_b45_pulse(p, 5) {
            let r = (self.rand() & 7) as u32;
            if self.assets.enemy.t.u8(0x8417A, r) != 0 {
                p.m.sb(0xA7, p.m.b(0xA7).wrapping_neg());
            }
            p.b.dir = p.b.dir.wrapping_add(p.m.b(0xA7)) & 7;
            self.fx_prop_muzzle((p.b.xi(), p.b.yi(), p.b.zi()), 0x104, 0xA002E, p.b.dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 0x84010: nonfatal hits return to mode 2 without advancing a cycle timer.
    #[test]
    fn hit_continues_and_pulse_uses_signed_byte() {
        let a = Box::leak(Box::new(dino_assets::AssetPack::default()));
        let mut g = Game::new(a);
        let mut p = Prop::default();
        p.b.mode = 4;
        p.b.hp = 38;
        p.m.sw(0xA8, 23);
        g.prop_b45(&mut p, 0, 0);
        assert_eq!(p.b.mode, 2);
        assert_eq!(p.m.w(0xA8), 23);
        p.m.sb(0xAA, 0);
        assert!(Game::prop_b45_pulse(&mut p, 4));
        for _ in 0..4 {
            assert!(!Game::prop_b45_pulse(&mut p, 4));
        }
        assert!(Game::prop_b45_pulse(&mut p, 4));
    }
}
