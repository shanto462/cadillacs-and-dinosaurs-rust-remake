//! Vice T., class 18 variant 0: init and intro. Fight and Walther remain placeholders.
use super::{in_win, Enemy};
use crate::game::{Game, Handle};

impl Game {
    /// 0x42590 / 0x425BC: the native intro portion of the still-partial class.
    pub(crate) fn enemy_vice_intro_active(b: &crate::game::body::Body) -> bool {
        b.k20 == 18 * 4 && b.var26 >> 8 == 0 && b.mode == 0
    }

    /// 0x42590 / 0x425A4: Vice mode 0 is init (sub 0), then intro (sub 2).
    pub(super) fn enemy_class18(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.var26 >> 8 != 0 || e.b.mode != 0 {
            // M5-11..13 own the fight, reactions, death and Walther. Snapshot runs
            // hold those modes, as before; normal play keeps the placeholder fight.
            self.enemy_class_unported(e, h, i);
            return;
        }
        if e.b.sub == 0 {
            self.enemy_vice_init(e, h);
        }
        self.enemy_vice_intro(e, h);
        self.enemy_draw_check(&mut e.b, h); // 0x426A6 -> 0x49CA
    }

    /// 0x425B6..0x4268C: Vice's init. 0x939B6 is the optional debug marker.
    fn enemy_vice_init(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_vice_hopper(e, h); // 0x4DCC6, before selecting a target
        e.b.id = 0x0C;
        e.m.sw(0x84, 0x30);
        e.b.b72 = 60;
        e.b.b63 = 2;
        e.b.anim_table = 0x438F2;
        self.enemy_set_boss_hp(&mut e.b, 0);
        e.m.sw(0xC0, e.b.hp);
        self.enemy_anim(&mut e.b, 0);
        e.b.box_table = 0x103000;
        e.b.multi_box = false;
        e.b.stun = 11;
        e.b.reaction = 0;
        for o in [0x79, 0x7A, 0x7D, 0xB1, 0xBD, 0x87, 0xC3] {
            e.m.sb(o, 0);
        }
        e.m.sb(0xA8, 2);
        e.m.sw(0xAA, 0);
        e.m.sw(0xBE, 900);
        e.m.sb(0xC2, 2);
        e.m.sw(0x76, self.enemy_pick_target(&e.b)); // 0x32032
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sb(0x7D, 0xFF); // 0x44EEC
        self.enemy_vice_initial_target(e); // 0x44298, one RNG call
        e.m.sb(0x7C, 4); // 0x44478, Vice branch has no RNG
        e.b.sub = e.b.sub.wrapping_add(2);
    }

    /// 0x4DCC6: make the chained Rock Hopper. Its intro is M5-10.
    fn enemy_vice_hopper(&mut self, e: &Enemy, h: Handle) {
        let Some(i) = self.enemies.alloc() else { return };
        let hopper = &mut self.enemies.slots[i];
        hopper.b.live = 1;
        hopper.b.k20 = 29 * 4;
        hopper.m.sl(0xA0, 0xFFFF_0000 | h as u32);
        // Word writes preserve the fractional halves of the allocated slot.
        hopper.b.x = (hopper.b.x & 0xFFFF) | (0x2D0 << 16);
        hopper.b.y = (hopper.b.y & 0xFFFF) | ((e.b.yi() as i32) << 16);
        hopper.b.z = (hopper.b.z & 0xFFFF) | (0x148 << 16);
        hopper.b.var26 = 0;
    }

    /// 0x44298 / 0x44416: pack live heroes, then select with the ROM weight row.
    fn enemy_vice_initial_target(&mut self, e: &mut Enemy) {
        for o in [0xA2, 0xA4, 0xA6] {
            e.m.sw(o, 0);
        }
        e.m.sb(0xA0, 0);
        let r = self.rand() & 15;
        let mut count = 0u16;
        for i in 0..3 {
            if self.players[i].b.live != 0 {
                e.m.sw(0xA2 + 2 * count as usize, Game::player_handle(i));
                count += 1;
            }
        }
        if count == 0 {
            e.m.sw(0xA2, Game::player_handle(0));
        }
        let row = self.assets.enemy.t.u16(0x442E0, 2 * count.saturating_sub(1) as u32);
        let mut choice = self.assets.enemy.t.u8(0x442E0, row as u32 + r as u32);
        if choice as i8 > 2 {
            choice = 0;
        }
        e.m.sb(0xA0, choice);
        e.m.sw(0x76, e.m.w(0xA2 + 2 * choice as usize));
    }

    /// 0x426C0..0x42880: intro jump table (r6 = 0, 2, ..., 0x12).
    fn enemy_vice_intro(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.act {
            0 => { // 0x426C0
                if (self.cam.s2.cam_x as i16) >= 0x1D0 {
                    e.b.act = e.b.act.wrapping_add(2);
                    if let Some(talk) = self.spawn_prop(50, 0, None) {
                        e.m.sw(0xAC, talk);
                        self.sound(0x20);
                    }
                }
            }
            2 => { // 0x426F4
                if self.body(e.m.w(0xAC)).live == 0 {
                    e.m.sb(0x80, 40);
                    e.b.act = e.b.act.wrapping_add(2);
                }
            }
            4 => { // 0x4270A, branches on zero, not borrow
                let t = e.m.b(0x80).wrapping_sub(1);
                e.m.sb(0x80, t);
                if t == 0 {
                    self.spawn_prop(76, 0, None);
                    self.enemy_anim(&mut e.b, 0x14);
                    e.b.act = e.b.act.wrapping_add(2);
                    self.enemy_vice_pull(e);
                }
            }
            6 => self.enemy_vice_pull(e),
            8 => { // 0x4274A, advances on borrow
                let t = e.m.b(0x80);
                e.m.sb(0x80, t.wrapping_sub(1));
                if t == 0 {
                    e.b.act = e.b.act.wrapping_add(2);
                    e.m.sb(0x80, 0);
                    self.enemy_anim(&mut e.b, 1);
                    self.enemy_set_speed(&mut e.b, 7);
                    e.m.sw(0x88, 0x317);
                    e.m.sw(0x8A, 0x136);
                    e.b.dir = Game::enemy_angle_to(&e.b, 0x317, 0x136);
                    self.enemy_vice_walk(e);
                }
            }
            0x0A => self.enemy_vice_walk(e),
            0x0C => self.enemy_vice_captions(e), // 0x427B2 (fallthrough from walk)
            0x0E => self.enemy_vice_gun(e, h),
            0x10 => { // 0x4284C
                let t = e.m.b(0x80);
                e.m.sb(0x80, t.wrapping_sub(1));
                if t == 0 {
                    e.b.act = e.b.act.wrapping_add(2);
                    self.spawn.g.b4fc = 0;
                    self.sound(0x0E);
                }
            }
            0x12 => { // 0x42864..0x4287C; alive AI entry is M5-11
                if let Some(controller) = self.boss_spawn_controller(h) {
                    // 0x7BEDC copies sign-extended a6, not a 24-bit bus address.
                    if let Some(crate::game::Slot::Prop(i)) = crate::game::slot_of(controller) {
                        self.props.slots[i].m.sl(0xAA, 0xFFFF_0000 | h as u32);
                    }
                }
                self.hud.boss = h;
                e.b.mode = e.b.mode.wrapping_add(2);
                e.b.sub = 0;
                e.b.act = 0;
                e.b.step = 0;
            }
            _ => {}
        }
    }

    /// 0x42736: finish the chain-pull animation, then wait 60 (borrow) frames.
    fn enemy_vice_pull(&self, e: &mut Enemy) {
        if self.enemy_step(&mut e.b) {
            e.b.act = e.b.act.wrapping_add(2);
            e.m.sb(0x80, 60);
        }
    }

    /// 0x42782 / 0x44AA2: unsigned arrival windows before motion and animation.
    fn enemy_vice_walk(&mut self, e: &mut Enemy) {
        let (x, z) = (e.m.w(0x88), e.m.w(0x8A));
        if in_win(e.b.xi().wrapping_sub(x as i16), 8) && in_win(e.b.zi().wrapping_sub(z as i16), 4) {
            e.b.act = e.b.act.wrapping_add(2);
            e.m.sb(0x80, 20);
            self.enemy_vice_captions(e);
        } else {
            e.b.dir = Game::enemy_angle_to(&e.b, x, z);
            self.enemy_move_dir(&mut e.b);
            self.enemy_step(&mut e.b);
        }
    }

    /// 0x427B2..0x427F4: both captions spawn immediately on arrival.
    fn enemy_vice_captions(&mut self, e: &mut Enemy) {
        if self.spawn_prop(66, 0x0B, None).is_some() && self.flow.region.0 != 0 {
            self.spawn_prop(66, 0x55, None);
        }
        self.enemy_anim(&mut e.b, 0x15);
        e.b.act = e.b.act.wrapping_add(2);
        // The 0x427F8 countdown also runs in this frame.
        e.m.sb(0x80, e.m.b(0x80).wrapping_sub(1));
    }

    /// 0x427F8..0x42846: delay, gun animation, frame-flag spark and sound.
    fn enemy_vice_gun(&mut self, e: &mut Enemy, h: Handle) {
        let t = e.m.b(0x80);
        e.m.sb(0x80, t.wrapping_sub(1));
        if t == 0 {
            e.m.sb(0x80, 0);
            if self.enemy_step(&mut e.b) {
                e.b.act = e.b.act.wrapping_add(2);
                e.m.sb(0x80, 60);
            }
        }
        if e.b.param_a != 0 {
            e.b.param_a = 0;
            // The pool updater holds e outside the slot. Publish its facing for
            // the shared 0x92422 service, then restore the empty slot's header.
            let i = self.enemy_index(h);
            let facing = self.enemies.slots[i].b.facing;
            self.enemies.slots[i].b.facing = e.b.facing;
            self.fx_spawn_spark_at(h, e.b.xi().wrapping_sub(0x50), e.b.yi().wrapping_add(0x40), e.b.zi(), 0);
            self.enemies.slots[i].b.facing = facing;
            self.snd_request(0x3B, 0x18, 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::trace::{Collect, F};
    use crate::harness::TraceReader;
    use std::path::Path;

    /// The recordings and ROM stay outside source control. Check each invocation
    /// against MAME, including the init RNG roll and all modeled object bytes.
    #[test]
    fn vice_intro_matches_mame_in_step() {
        let rom = Path::new("../../../dino.zip");
        let directory = Path::new("../../../re/out/traces");
        if !rom.exists() || !directory.exists() {
            return;
        }
        let assets = Box::leak(Box::new(dino_extract::extract(rom).unwrap()));
        let mut frames = 0;
        for name in [
            "m600_bs_v1_v1_nat", "boss_vice_2p_r0", "boss_vice_2p_r24",
            "boss_vice_3p_r0", "boss_vice_3p_r24",
        ] {
            let path = directory.join(format!("{name}.mame.trc"));
            if !path.exists() {
                continue;
            }
            let mut reader = TraceReader::open(path.to_str().unwrap()).unwrap();
            let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
            let mut g = Game::new(assets);
            let mut checked = 0;
            while let Some(frame) = reader.next() {
                let ram = reader.region("ram").unwrap().data.clone();
                let gfx = reader.region("gfx").unwrap().data.clone();
                if let Some((old, video)) = &previous {
                    g.trace_load(old, video);
                    for i in 0..g.enemies.slots.len() {
                        let h = g.enemies.handle(i);
                        let a = h as usize;
                        let e = &g.enemies.slots[i];
                        if e.b.live == 0 || !Game::enemy_vice_intro_active(&e.b) || e.b.act == 0x12 {
                            continue;
                        }
                        let mut e = std::mem::take(&mut g.enemies.slots[i]);
                        g.enemy_class18(&mut e, h, i);
                        let mut image = Collect::default();
                        e.trace_visit(&mut F::new(&mut image, 0xFF0000 | h as u32, assets));
                        for offset in 0..0xE0 {
                            if image.mask[a + offset] == 0 {
                                continue;
                            }
                            // 0x4DF80..0x4DF8A, owned by M5-10: the Hopper
                            // consumes Vice's +$2D before Vice's own update.
                            if offset == 0x2D && old[a + offset] != 0 && ram[a + offset] == 0 {
                                assert!(g.enemies.slots.iter().any(|hopper|
                                    hopper.b.live != 0 && hopper.b.k20 == 29 * 4
                                    && hopper.b.act == 8 && hopper.m.l(0xA0) as u16 == h));
                                continue;
                            }
                            assert_eq!(image.ram[a + offset], ram[a + offset],
                                "{name} frame {frame} Vice +{offset:02X}");
                        }
                        if old[a + 5] == 0 {
                            assert_eq!(g.w.rng, u16::from_be_bytes([ram[0x8014], ram[0x8015]]),
                                "{name} frame {frame} init RNG");
                        }
                        g.enemies.slots[i] = e;
                        checked += 1;
                    }
                }
                previous = Some((ram, gfx));
            }
            assert!(checked > 0, "{name}: no Vice intro invocations");
            eprintln!("{name}: {checked} Vice intro invocations equal MAME");
            frames += checked;
        }
        assert!(frames > 0, "no Vice reference recordings found");
    }

    /// 0x4270A tests zero, while 0x4274A and 0x4284C test borrow.
    #[test]
    fn vice_wait_uses_byte_borrow_and_camera_uses_signed_words() {
        let mut g = Game::new(Box::leak(Box::new(dino_assets::AssetPack::default())));
        let mut e = Enemy::default();
        e.b.act = 0x10;
        e.m.sb(0x80, 60);
        g.spawn.g.b4fc = 1;
        for _ in 0..60 {
            g.enemy_vice_intro(&mut e, 0);
            assert_eq!(e.b.act, 0x10);
        }
        g.enemy_vice_intro(&mut e, 0);
        assert_eq!((e.b.act, e.m.b(0x80), g.spawn.g.b4fc), (0x12, 0xFF, 0));
        assert_eq!(g.w.sounds, [0x0E]);
        e.b.act = 0;
        g.cam.s2.cam_x = 0xFFFF;
        g.enemy_vice_intro(&mut e, 0);
        assert_eq!(e.b.act, 0);
        g.cam.s2.cam_x = 0x1D0;
        g.enemy_vice_intro(&mut e, 0);
        assert_eq!(e.b.act, 2);
        assert_eq!(g.w.sounds, [0x0E, 0x20]);
    }
}
