//! Rock Hopper: Vice's partner intro. Hostile behaviour remains M7-12.
use super::Enemy;
use crate::game::{Game, Handle};

impl Game {
    /// 0x4DCF6: init, variant 0 intro, and the satisfied leave path.
    pub(super) fn enemy_class29(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if e.b.mode == 0 && e.b.var26 >> 8 == 0 && e.b.sub <= 4 {
            match e.b.sub {
                0 => self.enemy_hopper_init(e, h),
                2 => {
                    e.b.sub = e.b.sub.wrapping_add(2);
                    self.enemy_hopper_intro(e);
                    self.enemy_draw_check(&mut e.b, h);
                }
                4 => {
                    self.enemy_hopper_intro(e);
                    self.enemy_draw_check(&mut e.b, h);
                }
                _ => unreachable!(),
            }
        } else if e.b.mode == 2 && e.b.sub == 0 && e.m.w(0xA8) == 0xA {
            self.enemy_hopper_tail_leave(e, h, i);
        } else {
            // M7 owns other entrances, reactions and hostile AI. Keep the registry
            // until those modes are ported. Snapshot mode holds them as before.
            if !self.hold_unported && e.b.mode == 0 && e.b.sub == 6 {
                e.b.sub = 0;
            }
            self.enemy_class_unported(e, h, i);
        }
    }

    /// 0x4DD1E..0x4DE04: shared init; +$5C deliberately remains stale.
    fn enemy_hopper_init(&mut self, e: &mut Enemy, h: Handle) {
        e.b.id = 4;
        e.m.sw(0x84, 0x10);
        e.b.anim_table = 0x4F4E4;
        self.enemy_anim(&mut e.b, 0);
        e.b.box_table = 0x106000;
        e.b.multi_box = true;
        e.b.stun = 10;
        e.b.reaction = 0;
        for o in [0x79, 0x7A, 0x7B, 0x7D, 0xA4, 0xB8, 0xB9, 0xBA, 0xC1, 0xAC, 0xAD, 0xBB] {
            e.m.sb(o, 0);
        }
        e.b.b72 = 0xC8;
        e.m.sb(0x87, 4);
        e.b.speed_table = 0x4FA30;
        e.m.sw(0x76, self.enemy_pick_target(&e.b));
        e.m.sw(0xA8, 0);
        e.m.sw(0xAA, 0);
        self.enemy_set_hp(&mut e.b);
        self.enemy_snap_to_floor(e);
        e.m.sb(0x7D, 0xFF);
        if self.w.stage == 4 || self.w.stage == 7 {
            let palette = if self.w.stage == 4 { 0x9E } else { 0x9F };
            e.b.palette_override = palette;
            e.m.sb(0xBB, palette);
        }
        if self.w.stage == 7 {
            e.m.sw(0xA8, 6);
            e.m.sb(0xA4, 1);
        }
        e.b.sub = e.b.sub.wrapping_add(2);
        // 0x4DDFE jmp 0x94522: the sweat beads (pool C class 20) over this hopper.
        self.fx_spawn_sweat(h);
    }

    /// 0x4DEE4..0x4DFEC: partner states, including same-frame fallthroughs.
    fn enemy_hopper_intro(&mut self, e: &mut Enemy) {
        let parent = e.m.l(0xA0) as u16;
        if e.b.act == 0 {
            if e.b.step == 0 {
                e.b.step = e.b.step.wrapping_add(2);
                self.enemy_anim(&mut e.b, 0);
                e.b.facing = 0;
            }
            if self.body(parent).act != 2 {
                self.enemy_step(&mut e.b);
                return;
            }
            e.b.act = e.b.act.wrapping_add(2);
            e.b.step = 0;
        }
        if e.b.act == 2 {
            if self.body(parent).act != 4 { return; }
            e.b.act = e.b.act.wrapping_add(2);
            self.enemy_anim(&mut e.b, 1);
        }
        if e.b.act == 4 {
            if !self.enemy_step(&mut e.b) { return; }
            e.b.act = e.b.act.wrapping_add(2);
            self.enemy_anim(&mut e.b, 2);
        }
        if e.b.act == 6 {
            if self.body(parent).act != 0xE {
                self.enemy_step(&mut e.b);
                return;
            }
            e.b.act = e.b.act.wrapping_add(2);
            self.enemy_anim(&mut e.b, 3);
        }
        if e.b.act == 8 {
            if self.body(parent).param_b != 0 {
                self.body_mut(parent).param_b = 0;
                e.b.timer = 1;
            }
            if !self.enemy_step(&mut e.b) { return; }
            e.b.act = e.b.act.wrapping_add(2);
            e.m.sb(0xA4, 1);
            self.enemy_anim(&mut e.b, 4);
        }
        if e.b.act == 0xA {
            if e.b.param_b == 0 {
                self.enemy_step(&mut e.b);
                return;
            }
            e.b.act = e.b.act.wrapping_add(2);
            self.enemy_hopper_sound(e);
            self.enemy_monster_flash_start(e);
        }
        if e.b.act == 0xC {
            if !self.enemy_monster_flash_tick(e) {
                self.enemy_step(&mut e.b);
                return;
            }
            e.b.act = e.b.act.wrapping_add(2);
            self.enemy_anim(&mut e.b, 5);
        }
        if e.b.act == 0xE && self.enemy_step(&mut e.b) {
            e.b.sub = e.b.sub.wrapping_add(2);
        }
    }

    /// 0xA09E: growl with the original world-X pan, without an RNG call.
    fn enemy_hopper_sound(&mut self, e: &Enemy) {
        self.snd_request(0x60, self.snd_pan_x(e.b.xi()) as u32, 0);
    }

    /// 0x4F350: select the hop animation from aggression and gait.
    fn enemy_hopper_hop_start(&self, e: &mut Enemy) {
        let id = e.m.b(0xAD).wrapping_add(9).wrapping_add(if e.m.b(0xA4) != 0 { 12 } else { 0 });
        self.enemy_anim(&mut e.b, id as u16);
        e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
    }

    /// 0x4F374: a changed frame flag advances the ground anchor and depth.
    fn enemy_hopper_hop_step(&self, e: &mut Enemy) {
        if e.b.param_b == e.b.w34 as u8 { return; }
        let off = 4 * e.b.dir as u32;
        let dx = self.assets.enemy.t.i16(0x4FA30, off);
        let dz = self.assets.enemy.t.i16(0x4FA30, off + 2);
        e.b.vx = dx;
        e.b.vz = dz;
        e.m.sl(0xAE, e.m.l(0xAE).wrapping_add(((dx as i32) << 8) as u32));
        e.b.z = e.b.z.wrapping_add((dz as i32) << 8);
    }

    /// 0x4F3B4: per-frame visual hop offset from the fractional ground anchor.
    fn enemy_hopper_position(&self, e: &mut Enemy) {
        if e.b.airborne != 0 { return; }
        let mut dx = self.assets.enemy.t.i16(0x4F3DA, e.b.param_a as u32);
        if e.b.facing != 0 { dx = dx.wrapping_neg(); }
        e.b.set_xi_keep((e.m.w(0xAE) as i16).wrapping_add(dx));
    }

    /// 0x4E064..0x4E0AE: the mode 2 sub 0 tail for $A8 = 0xA (the leave). The hit flag $B8
    /// (0x4ECFC, M7 hostile code) is not ported; a leaving Hopper is not hit by this code.
    fn enemy_hopper_tail_leave(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        // 0x4E070: $79 counts down; at zero the stun is reset to 10 (move.b #$A,$78).
        let t = e.m.b(0x79);
        if t != 0 {
            e.m.sb(0x79, t.wrapping_sub(1));
            if t == 1 {
                e.b.stun = 10;
            }
        }
        // 0x4E082: $7A counts down and skips the dispatch, the position and the floor walk.
        if e.m.b(0x7A) != 0 {
            e.m.sb(0x7A, e.m.b(0x7A).wrapping_sub(1));
        } else {
            self.enemy_hopper_leave(e, i); // 0x4EC06 through the $A8 dispatch
            self.enemy_hopper_position(e); // 0x4F3B4
            self.terrain_collide_free(&mut e.b); // 0x12E20
        }
        self.enemy_draw_check(&mut e.b, h); // 0x49CA
        self.enemy_monster_register(h); // 0x4C0A (jmp)
        e.m.sb(0xB8, 0); // 0x4E04E: clr.b $b8 after the sub handler
    }

    /// 0x4EC06..0x4EC9E: satisfied roar, restore colour, hop off screen.
    fn enemy_hopper_leave(&mut self, e: &mut Enemy, i: usize) {
        self.enemy_despawn_if_far(e, i);
        match e.b.act {
            0 => {
                if e.m.b(0xA4) == 0 {
                    e.b.act = 6;
                    self.enemy_hopper_hop_start(e);
                } else {
                    e.m.sb(0xA4, 0);
                    self.enemy_anim(&mut e.b, 4);
                    e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
                    e.b.act = e.b.act.wrapping_add(2);
                }
            }
            2 => {
                if e.b.param_b == 0 { self.enemy_step(&mut e.b); }
                else {
                    e.b.act = e.b.act.wrapping_add(2);
                    self.enemy_monster_flash_start(e);
                }
            }
            4 => {
                if self.enemy_monster_flash_tick(e) {
                    e.b.act = 6;
                    self.enemy_hopper_hop_start(e);
                } else { self.enemy_step(&mut e.b); }
            }
            6 => {
                e.m.sw(0xAE, e.b.xi() as u16);
                e.b.dir = 0x18;
                e.b.facing = 0;
                e.m.sb(0xAD, 2);
                e.b.act = e.b.act.wrapping_add(2);
                self.enemy_hopper_hop_start(e);
            }
            8 => {
                e.b.w34 = u16::from_be_bytes([e.b.param_a, e.b.param_b]);
                self.enemy_step(&mut e.b);
                self.enemy_hopper_hop_step(e);
            }
            _ => {}
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
    fn hopper_intro_matches_mame_in_step() {
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
                        if e.b.live == 0 || !(e.b.k20 == 29 * 4 && e.b.var26 >> 8 == 0 && ((e.b.mode == 0 && e.b.sub <= 4) || (e.b.mode == 2 && e.b.sub == 0 && e.m.w(0xA8) == 0xA))) {
                            continue;
                        }
                        let mut e = std::mem::take(&mut g.enemies.slots[i]);
                        g.enemy_class29(&mut e, h, i);
                        let mut image = Collect::default();
                        e.trace_visit(&mut F::new(&mut image, 0xFF0000 | h as u32, assets));
                        for offset in 0..0xE0 {
                            if image.mask[a + offset] == 0 {
                                continue;
                            }
                            assert_eq!(image.ram[a + offset], ram[a + offset],
                                "{name} frame {frame} Hopper +{offset:02X}");
                        }
                        g.enemies.slots[i] = e;
                        checked += 1;
                    }
                }
                previous = Some((ram, gfx));
            }
            assert!(checked > 0, "{name}: no Hopper intro invocations");
            eprintln!("{name}: {checked} Hopper intro invocations equal MAME");
            frames += checked;
        }
        assert!(frames > 0, "no Hopper reference recordings found");
    }

}
