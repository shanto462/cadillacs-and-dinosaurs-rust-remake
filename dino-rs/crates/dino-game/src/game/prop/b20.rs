//! Pool B class 20, GO burst and idle hint (0x7BB68).
//!
use super::Prop;
use super::tile_block::TilePos;
use crate::game::Game;

impl Game {
    /// 0x7BB68: variant 0 idle hint; nonzero variant enters 0x7BD40.
    pub fn prop_go_hint(&mut self, p: &mut Prop, i: usize) {
        if p.b.var26 != 0 {
            match p.b.mode {
                0 => {
                    // 0x7BD56 falls through into the first blink this frame.
                    p.b.mode = p.b.mode.wrapping_add(2);
                    p.m.sb(0x81, 2);
                    p.m.sb(0x80, 35);
                    self.prop_go_palette();
                }
                2 => {}
                _ => return self.prop_free(p, i), // 0x7BD3A -> 0x4938
            }
            if self.prop_go_blink(p) {
                // 0x7BDA8: free on the next update, after all 105 frames.
                p.b.mode = p.b.mode.wrapping_add(2);
            }
            return;
        }

        match p.b.mode {
            0 => {
                // 0x7BB84 falls through, so the first countdown is 599.
                self.spawn.g.go_camera_x = self.cam.s2.cam_x;
                p.m.sw(0xA0, self.cam.s2.cam_x);
                self.spawn.g.go_timer = 600;
                p.b.mode = p.b.mode.wrapping_add(2);
            }
            2 => {}
            _ => return self.prop_free(p, i),
        }
        if self.spawn.g.go_timer != 0 {
            self.spawn.g.go_timer = self.spawn.g.go_timer.wrapping_sub(1);
            if self.spawn.g.go_timer == 0 {
                // 0x7BBB0..0x7BBBC: signed BGE, including negative deltas.
                let delta = self.cam.s2.cam_x.wrapping_sub(p.m.w(0xA0)) as i16;
                p.m.sw(0xA0, self.cam.s2.cam_x);
                if delta < 0x80 {
                    p.m.sb(0x81, 2);
                    p.m.sb(0x80, 35);
                    p.b.sub = 2;
                    self.prop_go_palette();
                } else {
                    self.spawn.g.go_timer = 600;
                }
            }
        }
        let blocked = self.cam.s2.lock78 != 0 || self.w.scripted != 0 || self.spawn.g.b4d5 != 0;
        if p.b.sub == 0 {
            if blocked {
                // 0x7BC0E: this runs after decrement and trigger testing.
                self.spawn.g.go_timer = 180;
                p.m.sw(0xA0, self.cam.s2.cam_x);
            }
        } else if blocked {
            // 0x7BC36: cancellation uses 600, not the idle hold of 180.
            self.prop_go_picture(false);
            self.spawn.g.go_timer = 600;
            p.b.sub = 0;
        } else if self.prop_go_blink(p) {
            self.spawn.g.go_timer = 600;
            p.b.sub = 0;
        }
    }

    /// 0x7BC46 / 0x7BD6A: one frame of the three 35-frame blink periods.
    fn prop_go_blink(&mut self, p: &mut Prop) -> bool {
        match p.m.b(0x80) {
            35 => {
                self.prop_go_picture(true);
                self.sound(0x3E);
            }
            5 => self.prop_go_picture(false),
            _ => {}
        }
        let ticks = p.m.b(0x80).wrapping_sub(1);
        p.m.sb(0x80, ticks);
        if ticks != 0 {
            return false;
        }
        p.m.sb(0x80, 35);
        // SUBQ.B followed by BCC tests borrow, not the sign of the result.
        let (left, borrow) = p.m.b(0x81).overflowing_sub(1);
        p.m.sb(0x81, left);
        borrow
    }

    /// 0x7BC90: replace seven scroll1 palette 10 colours, keeping brightness.
    fn prop_go_palette(&mut self) {
        for n in 0..7 {
            let at = (0x914540 - 0x914000) / 2 + n;
            let colour = self.assets.enemy.t.u16(0x7BCAC, (n * 2) as u32);
            self.pal.words[at] = (self.pal.words[at] & 0xF000) | colour;
        }
        self.pal.dirty = true;
    }

    /// 0x7BCBA / 0x7BCC2 / 0x7BCC6: five columns of four scroll1 cells.
    fn prop_go_picture(&mut self, visible: bool) {
        let table = if visible { 0x7BCEA } else { 0x7BD12 };
        let at = TilePos::at(0x90B238);
        for col in 0..5 {
            for row in 0..4 {
                let code = self.assets.enemy.t.u16(table, ((col * 4 + row) * 2) as u32);
                self.prop_tile_put(at.right(col).down(row), code, 10);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game(variant: u16) -> (Game, usize) {
        let mut a = AssetPack::default();
        // Synthetic cells and colours, deliberately unrelated to the ROM artwork.
        a.enemy.t.blobs = vec![
            (0x7BCAC, vec![0; 14]),
            (0x7BCEA, vec![1; 40]),
            (0x7BD12, vec![0; 40]),
        ];
        let mut g = Game::new(Box::leak(Box::new(a)));
        let i = g.props.alloc().unwrap();
        let p = &mut g.props.slots[i];
        p.b.live = 1;
        p.b.k20 = 20 * 4;
        p.b.var26 = variant;
        (g, i)
    }

    #[test]
    fn idle_hold_countdown_burst_and_cancel() {
        let (mut g, i) = game(0);
        g.cam.s2.cam_x = 400;
        g.cam.s2.lock78 = 1;
        for _ in 0..450 {
            g.pool_b_update();
            assert_eq!(g.spawn.g.go_timer, 180);
        }
        g.cam.s2.lock78 = 0;
        for left in (1..180).rev() {
            g.pool_b_update();
            assert_eq!(g.spawn.g.go_timer, left);
            assert_eq!(g.props.slots[i].b.sub, 0);
        }
        for frame in 0..105 {
            g.pool_b_update();
            assert_eq!(g.spawn.g.go_timer, if frame == 104 { 600 } else { 0 });
            assert_eq!(g.props.slots[i].b.sub, if frame == 104 { 0 } else { 2 });
        }
        g.spawn.g.go_timer = 1;
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.sub, 2);
        g.w.scripted = 1;
        g.pool_b_update();
        assert_eq!((g.spawn.g.go_timer, g.props.slots[i].b.sub), (600, 0));
        g.pool_b_update();
        assert_eq!(g.spawn.g.go_timer, 180);
    }

    #[test]
    fn moving_camera_reloads_and_forced_burst_ignores_lock() {
        let (mut g, i) = game(0);
        g.pool_b_update();
        g.spawn.g.go_timer = 1;
        g.cam.s2.cam_x = 128;
        g.pool_b_update();
        assert_eq!((g.spawn.g.go_timer, g.props.slots[i].b.sub), (600, 0));
        // Signed subtraction: moving backwards triggers the hint too.
        g.spawn.g.go_timer = 1;
        g.cam.s2.cam_x = 127;
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.sub, 2);
        let (mut g, i) = game(2);
        g.cam.s2.lock78 = 1;
        g.w.scripted = 1;
        for _ in 0..105 {
            g.pool_b_update();
            assert_ne!(g.props.slots[i].b.live, 0);
        }
        assert_eq!(g.props.slots[i].b.mode, 4);
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.live, 0);
    }

    #[test]
    fn snapshot_restores_hint_globals_and_mid_blink_state() {
        let (mut g, i) = game(0);
        g.pool_b_update();
        g.spawn.g.go_timer = 1;
        g.pool_b_update();
        for _ in 0..38 {
            g.pool_b_update();
        }
        let before = g.trace_ram();
        let mut loaded = Game::new(g.assets);
        loaded.trace_load(&before.ram, &vec![0; 0x30000]);
        assert_eq!(loaded.spawn.g.go_camera_x, g.spawn.g.go_camera_x);
        assert_eq!(loaded.spawn.g.go_timer, g.spawn.g.go_timer);
        assert_eq!(loaded.props.slots[i].m.w(0x80), g.props.slots[i].m.w(0x80));
        assert_eq!(loaded.props.slots[i].m.w(0xA0), g.props.slots[i].m.w(0xA0));
        for _ in 0..70 {
            g.pool_b_update();
            loaded.pool_b_update();
            assert_eq!(loaded.spawn.g.go_timer, g.spawn.g.go_timer);
            assert_eq!(loaded.props.slots[i].b.sub, g.props.slots[i].b.sub);
            assert_eq!(loaded.props.slots[i].m.w(0x80), g.props.slots[i].m.w(0x80));
        }
    }
}
