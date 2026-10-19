//! Pool B 71 end-area palette animation, 0x89746..0x8997E.
use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x89746: six-count phase loop, first wait 16 frames, subsequent waits 8.
    pub(super) fn prop_b71(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.live = 2;
                p.m.sw(0x84, 16);
                p.m.sl(0x86, 0x898C8);
                p.m.sw(0x8A, 6);
                p.b.mode = 2;
            }
            2 => {}
            _ => return self.prop_free(p, i),
        }
        let n = p.m.w(0x84).wrapping_sub(1);
        p.m.sw(0x84, n);
        if n != 0 {
            return;
        }
        p.m.sw(0x84, 8);
        let left = p.m.w(0x8A).wrapping_sub(1);
        p.m.sw(0x8A, left);
        // 0x89796 resets before the copy, so the last table column is skipped.
        if left == 0 {
            p.m.sw(0x8A, 6);
            p.m.sl(0x86, 0x898C8);
        }
        self.prop_b71_palettes(p.m.l(0x86));
        p.m.sl(0x86, p.m.l(0x86).wrapping_add(2));
    }

    /// 0x897B4, 0x89964: thirteen palettes, preserving the brightness nibble.
    fn prop_b71_palettes(&mut self, phase: u32) {
        for (k, dest) in [
            0x914820, 0x9148A0, 0x9148C0, 0x9148E0, 0x914900, 0x914940, 0x914980, 0x9149A0,
            0x9149C0, 0x9149E0, 0x914B20, 0x914C20, 0x914E20,
        ]
        .into_iter()
        .enumerate()
        {
            let base: u32 = if k < 11 { 0xBCA52 } else { 0xC2252 };
            let offset = self.assets.enemy.t.i16(phase, k as u32 * 12);
            self.prop_palette_copy(dest, base.wrapping_add(offset as i32 as u32), 16);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 0x8977C: first wait includes init, each later wait is eight updates.
    #[test]
    fn phase_wraps_after_six_steps() {
        let mut a = dino_assets::AssetPack::default();
        // Synthetic offsets and colours, unrelated to ROM contents.
        a.enemy.t.blobs = vec![
            (0x898C8, vec![0; 156]),
            (0xBCA52, vec![0; 32]),
            (0xC2252, vec![0; 32]),
        ];
        let mut g = Game::new(Box::leak(Box::new(a)));
        let mut p = Prop::default();
        for _ in 0..15 {
            g.prop_b71(&mut p, 0);
        }
        assert_eq!(p.m.l(0x86), 0x898C8);
        g.prop_b71(&mut p, 0);
        assert_eq!(p.m.l(0x86), 0x898CA);
        for _ in 0..40 {
            g.prop_b71(&mut p, 0);
        }
        assert_eq!(p.m.l(0x86), 0x898CA);
        assert_eq!(p.m.w(0x8A), 6);
    }
}
