//! Enemy bullets, all variants of class 59 (0x57FB4).
use super::Enemy;
use crate::game::{Game, Handle};

impl Game {
    /// 0x57F62: allocate a bullet, preserving fractional coordinates and stale +$84.
    pub(super) fn enemy_spawn_bullet(&mut self, owner: &Enemy, variant: u16, dx: i16, dy: i16) {
        let Some(i) = self.enemies.alloc() else { return };
        let e = &mut self.enemies.slots[i];
        e.b.live = 1;
        e.b.k20 = 59 * 4;
        e.b.var26 = variant;
        let dx = if owner.b.facing == 0 { dx.wrapping_neg() } else { dx };
        // 0x57F90 is ADD.W, including the reused slot's integer X.
        e.b.set_xi_keep(e.b.xi().wrapping_add(owner.b.xi().wrapping_add(dx)));
        e.b.set_yi_keep(owner.b.yi().wrapping_add(dy));
        e.b.set_zi_keep(owner.b.zi());
        e.b.anim_table = 0x6FB1A;
        e.b.facing = owner.b.facing;
    }

    /// 0x57FB4 / 0x57FC8 / 0x58082: init and straight flight.
    pub(super) fn enemy_class59(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => {
                e.b.box_table = 0x103C00;
                e.b.multi_box = false;
                self.enemy_begin_attack(&mut e.b, 0xB);
                e.b.live = 1;
                e.b.anim_table = 0x6FB1A;
                e.b.draw_flags = 0x10;
                e.m.sb(0x7D, 0xFF);
                e.b.b63 = 0xFF;
                // 0x5800C's read of address zero is overwritten at 0x58001C.
                // Both serial increments are observable even though the first
                // serial is replaced before the bullet can collide.
                self.enemy_begin_attack(&mut e.b, 0xB);
                e.m.sw(0xA0, 7);
                self.enemy_anim(&mut e.b, 0);
                if e.b.var26 != 0 {
                    e.m.sw(0xA0, 7);
                    self.enemy_anim(&mut e.b, 0);
                }
                if e.b.var26 == 5 {
                    e.m.sw(0xA2, 30);
                    e.m.sw(0xA0, 8);
                    self.enemy_anim(&mut e.b, 1);
                }
                if e.b.facing == 0 { e.m.sw(0xA0, e.m.w(0xA0).wrapping_neg()); }
                self.terrain_collide(&mut e.b, h, false);
                e.b.mode = e.b.mode.wrapping_add(2);
            }
            2 => {}
            _ => { self.enemy_free_pool_a(e, i); return; }
        }
        // 0x173C marks mode 6 but still finishes this update.
        self.enemy_despawn_if_far(e, i);
        let mut move_x = true;
        if e.b.var26 == 5 {
            let life = e.m.w(0xA2).wrapping_sub(1);
            e.m.sw(0xA2, life);
            if life == 0 { self.enemy_free_pool_a(e, i); return; }
            move_x = (life as i16) > 12;
        }
        if move_x { e.b.set_xi_keep(e.b.xi().wrapping_add(e.m.w(0xA0) as i16)); }
        self.enemy_step(&mut e.b);
        self.enemy_push_hurtable(h);
        self.enemy_draw_check(&mut e.b, h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::trace::{Collect, F};
    use crate::harness::TraceReader;
    use std::path::Path;

    /// 0x5802A..0x580AA: every variant, signed motion, double serial and timed stop.
    #[test]
    fn bullet_variants_preserve_word_motion_and_serials() {
        let rom = Path::new("../../../dino.zip");
        if !rom.exists() { return; }
        let assets = Box::leak(Box::new(dino_extract::extract(rom).unwrap()));
        for variant in 0..=5 {
            for facing in 0..=1 {
                let mut g = Game::new(assets);
                let i = g.enemies.alloc().unwrap();
                let h = g.enemies.handle(i);
                let mut e = std::mem::take(&mut g.enemies.slots[i]);
                e.b.live = 1;
                e.b.k20 = 59*4;
                e.b.var26 = variant;
                e.b.facing = facing;
                e.b.x = (100 << 16) | 0xABCD;
                e.b.y = 128 << 16;
                e.b.z = 50 << 16;
                e.m.sw(0x84, 0x1234);
                g.w.attack_serial = 0xFF;
                let speed = if variant == 5 { 8 } else { 7 } * if facing == 0 { -1 } else { 1 };
                g.enemy_class59(&mut e, h, i);
                assert_eq!((e.b.serial, g.w.attack_serial), (0, 1));
                assert_eq!(e.b.x, ((100 + speed) << 16) | 0xABCD);
                assert_eq!((e.m.w(0x84), e.b.atk), (0x1234, 0xB));
                if variant == 5 {
                    for _ in 1..18 { g.enemy_class59(&mut e, h, i); }
                    let stop_x = e.b.x;
                    // Countdown 12 through 1 still draws and registers the bullet.
                    for _ in 18..29 { g.enemy_class59(&mut e, h, i); assert_eq!(e.b.x, stop_x); }
                    assert_ne!(e.b.live, 0);
                    g.enemy_class59(&mut e, h, i);
                    assert_eq!(e.b.live, 0);
                }
            }
        }
    }

    /// 0x57FB4: compare the entire bullet against the H5 MAME reference updates.
    #[test]
    fn bullet_h5_matches_mame() {
        let rom = Path::new("../../../dino.zip");
        let path = "../../../re/out/traces/m600_h5_bsp_ctrl.mame.trc";
        if !rom.exists() || !Path::new(path).exists() { return; }
        let assets = Box::leak(Box::new(dino_extract::extract(rom).unwrap()));
        let mut reader = TraceReader::open(path).unwrap();
        let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
        let mut g = Game::new(assets);
        let mut checked = 0;
        let script = crate::input::Script::parse(&std::fs::read_to_string("../../../re/out/traces/m600_h5_bsp_ctrl.input.txt").unwrap());
        let mut pokes = crate::harness::PokeSet::default();
        pokes.add_file("../../../re/out/traces/m600_h5_bsp_ctrl.pokes.txt", crate::harness::PokeClock::Power).unwrap();
        while let Some(frame) = reader.next() {
            let ram = reader.region("ram").unwrap().data.clone();
            let gfx = reader.region("gfx").unwrap().data.clone();
            if let Some((old, video)) = &previous {
                g.trace_load(old, video);
                g.hold_unported = true;
                pokes.apply(&mut g, frame, false).unwrap();
                g.w.input = script.at(frame as u32).players;
                let bullets: Vec<_> = (0..g.enemies.slots.len()).filter(|&i|
                    g.enemies.slots[i].b.live != 0 && g.enemies.slots[i].b.k20 == 59*4).collect();
                g.tick();
                pokes.apply(&mut g, frame, true).unwrap();
                for i in 0..g.enemies.slots.len() {
                    let h = g.enemies.handle(i);
                    let a = h as usize;
                    if !bullets.contains(&i) { continue; }
                    let mut e = std::mem::take(&mut g.enemies.slots[i]);
                    let mut image = Collect::default();
                    e.trace_visit(&mut F::new(&mut image, 0xFF0000 | h as u32, assets));
                    for offset in 0..0xE0 {
                        if image.mask[a+offset] != 0 {
                            assert_eq!(image.ram[a+offset], ram[a+offset], "H5 frame {frame} bullet +{offset:02X}");
                        }
                    }
                    g.enemies.slots[i] = e;
                    checked += 1;
                }
            }
            previous = Some((ram, gfx));
        }
        assert!(checked > 0);
    }
}
