//! Pool B 76: the short backdrop flash shared by boss introductions.
use super::Prop;
use crate::game::Game;
use dino_assets::stage::Cell;

impl Game {
    /// 0x89EDE / 0x89EF2 / 0x89F10: save the far-layer scroll, flash, restore, free.
    pub fn prop_b76(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode = 2;
                p.b.live = 2;
                p.m.sw(0x82, self.cam.s3.cam_y);
                p.m.sw(0x84, self.cam.s3.cam_x);
                p.m.sb(0x88, 0);
            }
            2 => {}
            _ => return self.prop_free(p, i), // 0x8A062
        }
        match p.b.sub {
            0 => {
                // 0x89F24: falls through to the countdown in the same update.
                p.b.sub = 2;
                p.m.sw(0x86, 10);
                self.w.scroll3_base = 0x9200;
                self.prop_b76_fill(p.b.var26);
                self.w.scroll3_base = 0x9100;
                self.prop_b76_countdown(p);
            }
            2 => self.prop_b76_countdown(p),
            _ => {
                // 0x89FBE: fixed normal values, not the values present at spawn.
                self.w.scroll3_base = 0x9100;
                self.w.layer_ctrl = 0x12D6;
                self.cam.s3.cam_y = p.m.w(0x82);
                self.cam.s3.cam_x = p.m.w(0x84);
                p.b.mode = p.b.mode.wrapping_add(2);
            }
        }
    }

    /// 0x89F58: on multiples of four, alternate the flash map and the normal backdrop.
    fn prop_b76_countdown(&mut self, p: &mut Prop) {
        let left = p.m.w(0x86).wrapping_sub(1);
        p.m.sw(0x86, left);
        if left == 0 {
            p.b.sub = p.b.sub.wrapping_add(2);
        }
        if left & 3 != 0 {
            return;
        }
        if left & 4 == 0 {
            self.w.layer_ctrl = 0x1396;
            self.w.scroll3_base = 0x9200;
            self.cam.s3.cam_x = 0;
            self.cam.s3.cam_y = 0;
            p.m.sb(0x88, 1);
        } else {
            self.w.layer_ctrl = 0x12D6;
            self.w.scroll3_base = 0x9100;
            self.cam.s3.cam_y = p.m.w(0x82);
            self.cam.s3.cam_x = p.m.w(0x84);
            p.m.sb(0x88, 0);
        }
    }

    /// 0x8A03C, called for 0x923800 then 0x922800. DBRA writes 0x401 cells each.
    fn prop_b76_fill(&mut self, variant: u16) {
        let offset = variant.wrapping_mul(2) as u32;
        let cell = Cell {
            code: self.assets.frontend.text.u16(0x8A020, offset),
            attr: self.assets.frontend.text.u16(0x8A02E, offset),
        };
        self.vram.intro3.resize(0x1201, Cell::default());
        for base in [0x3800, 0x2800] {
            self.vram.intro3[base / 4..base / 4 + 0x401].fill(cell);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    #[test]
    fn b76_falls_through_counts_ten_ticks_and_restores_before_freeing() {
        let mut pack = AssetPack::default();
        pack.frontend.text.blobs.push((0x8A020, vec![0; 28]));
        let mut g = Game::new(Box::leak(Box::new(pack)));
        g.cam.s3.cam_x = 0x1234;
        g.cam.s3.cam_y = 0x5678;
        let i = g.props.alloc().unwrap();
        let mut p = Prop::default();
        p.b.live = 1;
        for frame in 0..10 {
            g.prop_b76(&mut p, i);
            assert_eq!(p.m.w(0x86), 9 - frame);
            let flash = matches!(frame, 1..=4 | 9);
            assert_eq!(g.w.scroll3_base, if flash { 0x9200 } else { 0x9100 });
            assert_eq!(p.m.b(0x88), flash as u8);
        }
        assert_eq!(p.b.sub, 4);
        g.prop_b76(&mut p, i);
        assert_eq!(
            (p.b.mode, g.w.layer_ctrl, g.w.scroll3_base),
            (4, 0x12D6, 0x9100)
        );
        assert_eq!((g.cam.s3.cam_x, g.cam.s3.cam_y), (0x1234, 0x5678));
        assert_eq!(
            p.m.b(0x88),
            1,
            "the restore deliberately leaves the flash byte set"
        );
        g.prop_b76(&mut p, i);
        assert_eq!(p.b.live, 0);
        assert_eq!(g.props.free_count(), 24);
    }
}
