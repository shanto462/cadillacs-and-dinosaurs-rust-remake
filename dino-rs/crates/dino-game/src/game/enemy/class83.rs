//! Pool A 83 flame pieces, 0x6152C..0x61628.
use super::Enemy;
use crate::game::{body::Body, Game, Handle};

impl Game {
    /// 0x6152C: allocate a flame, copying only integer position words and parameters.
    pub(crate) fn enemy_c83_spawn(&mut self, b: &Body, variant: u16, life: u16) {
        let Some(i) = self.enemies.alloc() else {
            return;
        };
        let e = &mut self.enemies.slots[i];
        e.b.live = 1;
        e.b.k20 = 83 * 4;
        e.b.set_xi_keep(b.xi());
        e.b.set_yi_keep(b.yi());
        e.b.set_zi_keep(b.zi());
        e.b.var26 = variant;
        e.m.sw(0xA8, life);
    }

    /// 0x6155C: initialize then fall through into the moving hostile flame.
    pub(super) fn enemy_class83(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.mode == 0 {
            e.b.anim_table = 0x6162E;
            e.b.draw_flags = 0;
            e.m.sb(0x7D, 0xFF);
            let t = &self.assets.enemy.t;
            e.b.set_xi_keep(e.b.xi().wrapping_add(t.i16(0x61730, 0)));
            e.b.set_yi_keep(e.b.yi().wrapping_add(t.i16(0x61730, 2)));
            e.b.set_zi_keep(e.b.zi().wrapping_add(t.i16(0x61730, 4)));
            e.m.sl(0xA0, t.u32(0x61730, 6));
            e.m.sl(0xA4, t.u32(0x61730, 10));
            e.b.box_table = 0x100000;
            e.b.multi_box = false;
            e.b.hitstop = 0;
            e.b.atk = 0x5D;
            e.b.knock = 0;
            e.b.serial = self.w.attack_serial;
            self.w.attack_serial = self.w.attack_serial.wrapping_add(1);
            e.m.sw(0xAA, 5);
            self.enemy_floor_collision(e);
            let r = self.rand() & 3;
            self.enemy_anim(&mut e.b, r);
            e.b.mode = 2;
        } else if e.b.mode != 2 {
            return self.enemy_free_pool_a(e, i);
        }
        let life = e.m.w(0xA8).wrapping_sub(1);
        e.m.sw(0xA8, life);
        if (life as i16) < 0 {
            return self.enemy_free_pool_a(e, i);
        }
        e.b.x = e.b.x.wrapping_add(e.m.l(0xA0) as i32);
        e.b.z = e.b.z.wrapping_add(e.m.l(0xA4) as i32);
        let n = e.m.w(0xAA).wrapping_sub(1);
        e.m.sw(0xAA, n);
        if (n as i16) < 0 {
            e.m.sw(0xAA, 5);
            self.enemy_floor_collision(e);
        }
        self.enemy_monster_register(h);
        self.enemy_step(&mut e.b);
        self.enemy_draw_check(&mut e.b, h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 0x615D2, 0x615E8: one RNG call on init, signed word life and long movement.
    #[test]
    fn flame_initialization_and_lifetime() {
        let path = std::path::Path::new("../../../dino.zip");
        if !path.exists() {
            return;
        }
        let a = Box::leak(Box::new(dino_extract::extract(path).unwrap()));
        let mut g = Game::new(a);
        let mut control = Game::new(a);
        control.rand();
        let mut source = Body::default();
        source.set_xi(672);
        source.set_yi(48);
        source.set_zi(848);
        g.enemy_c83_spawn(&source, 0, 2);
        let i = g.enemies.slots.iter().position(|e| e.b.live != 0).unwrap();
        let h = g.enemies.handle(i);
        let mut e = std::mem::take(&mut g.enemies.slots[i]);
        g.enemy_class83(&mut e, h, i);
        assert_eq!(e.b.atk, 0x5D);
        assert_eq!(e.m.w(0xA8), 1);
        assert_eq!(g.rand(), control.rand());
        let x = e.b.x;
        let z = e.b.z;
        g.enemy_class83(&mut e, h, i);
        assert_eq!(e.b.x, x.wrapping_add(e.m.l(0xA0) as i32));
        assert_eq!(e.b.z, z.wrapping_add(e.m.l(0xA4) as i32));
        assert_eq!(e.m.w(0xA8), 0);
        g.enemy_class83(&mut e, h, i);
        assert_eq!(e.b.live, 0);
    }
}
