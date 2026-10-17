//! Pool B class 59 (0x861E8): foundry far-layer palette cycler (Episode 4 area 2).
//! Spec: D3e part 2 section 6, A213, W216.

use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x861E8: pool B class 59 update.
    pub fn prop_b59(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x861FC: init
                let var = p.b.var26 as u32;
                let reload = self.assets.enemy.t.u8(0x862B6, var); // 0x86200
                p.m.sb(0xA5, reload); // 0x86206
                let entry_addr = 0x8622A + var * 8; // 0x8620E
                let list_ptr = self.assets.enemy.t.u32(entry_addr, 0); // 0x86212
                let pal_dest = self.assets.enemy.t.u32(entry_addr, 4); // 0x8621C
                p.m.sl(0xA0, list_ptr); // 0x86212
                p.m.sl(0xA8, list_ptr); // 0x86216
                p.m.sl(0xAC, pal_dest); // 0x8621C
                p.m.sb(0xA4, 0); // 0x86220: clr.b $a4(a6)
                p.b.mode = 2; // 0x86224: addq.b #2, $4(a6)
                // 0x86228: rts
            }
            2 => {
                // 0x86242: cmpi.b #3, $4d9(a5) - stage guard (Episode 4 is stage 3)
                if self.w.stage != 3 {
                    p.b.mode += 2; // 0x8624E: addq.b #2, $4(a6)
                    return;
                }
                // 0x86254: subq.b #1, $a4(a6)
                let timer = p.m.b(0xA4).wrapping_sub(1);
                p.m.sb(0xA4, timer);
                if timer != 0xFF {
                    return; // 0x8625A: rts
                }
                // 0x8625C: reload timer
                p.m.sb(0xA4, p.m.b(0xA5));
                let mut cur_ptr = p.m.l(0xA0);
                let mut frame_ptr = self.assets.enemy.t.u32(cur_ptr, 0); // 0x86266
                cur_ptr += 4;
                if (frame_ptr as i32) < 0 {
                    cur_ptr = p.m.l(0xA8); // 0x8626A: loop back
                    frame_ptr = self.assets.enemy.t.u32(cur_ptr, 0);
                    cur_ptr += 4;
                }
                p.m.sl(0xA0, cur_ptr); // 0x86270

                // 0x86276: read subrange offset and count from 0x862A4
                let var = p.b.var26 as u32;
                let sub_addr = 0x862A4 + var * 4;
                let word_offset = self.assets.enemy.t.u16(sub_addr, 0) as u32; // 0x86280
                let word_count = self.assets.enemy.t.u16(sub_addr, 2); // 0x8628C
                let dest_base = p.m.l(0xAC);
                let dest_addr = dest_base + word_offset * 2; // 0x86284, 0x86288

                // 0x86292..0x862A2: copy palette words while preserving brightness
                self.prop_palette_copy(dest_addr, frame_ptr, word_count);
            }
            _ => self.prop_free(p, i), // 0x862B0: jmp 0x4938
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use std::path::Path;

    fn rom_game() -> Option<Game> {
        let path = Path::new("../../../dino.zip");
        let alt = Path::new("../dino.zip");
        let p = if path.exists() { path } else if alt.exists() { alt } else { return None };
        let assets: &'static AssetPack = Box::leak(Box::new(dino_extract::extract(p).expect("the ROM set loads")));
        Some(Game::new(assets))
    }

    #[test]
    fn b59_init_and_cycling() {
        let Some(mut g) = rom_game() else { return };
        g.w.stage = 3; // Episode 4

        for var in 0..3 {
            let h = g.spawn_prop(59, var, None).expect("spawn b59");
            let i = (0..24).find(|&k| g.props.handle(k) == h).unwrap();
            let mut p = std::mem::take(&mut g.props.slots[i]);

            g.prop_b59(&mut p, i);
            assert_eq!(p.b.mode, 2);
            assert_eq!(p.m.b(0xA5), 8); // reload
            assert_eq!(p.m.b(0xA4), 0); // timer
            assert_ne!(p.m.l(0xA0), 0);
            assert_ne!(p.m.l(0xAC), 0);

            // First step in mode 2 underflows timer from 0 to 0xFF and triggers reload
            g.prop_b59(&mut p, i);
            assert_eq!(p.m.b(0xA4), 8);

            // Next 8 calls count down without reload
            for expected in (1..=8).rev() {
                g.prop_b59(&mut p, i);
                assert_eq!(p.m.b(0xA4), expected - 1);
            }
            // 9th call triggers reload again
            g.prop_b59(&mut p, i);
            assert_eq!(p.m.b(0xA4), 8);
        }

        // Test stage guard: stage != 3 advances mode to 4 then frees
        let h = g.spawn_prop(59, 0, None).expect("spawn b59");
        let i = (0..24).find(|&k| g.props.handle(k) == h).unwrap();
        let mut p = std::mem::take(&mut g.props.slots[i]);
        p.b.mode = 2;
        g.w.stage = 0; // Stage 0 (not Episode 4)
        g.prop_b59(&mut p, i);
        assert_eq!(p.b.mode, 4);
        g.prop_b59(&mut p, i);
        assert_eq!(p.b.live, 0);
    }
}
