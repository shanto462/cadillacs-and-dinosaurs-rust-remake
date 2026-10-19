//! Pool C class 2 (+$20 = 0x08), handler 0x92808: falling cave dust and rain drops with floor splash.
//! Spec: D3e part 3 section 2.3, D3f class 2.
//!
//! Spawned by Pool B class 56 (rain drops controller at 0x85916) in Episode 6 Area 1.
//!
//! - Mode 0 (0x9282A): loads raw sprite list by variant (var 0: 0x928C0, var 4: 0x928DA, var 8: 0x928F4);
//!   sets draw_flags = 0x50; dx, dy from 0x9281E stored in +$58 (owner58) and +$5A (owner_class);
//!   advances mode to 2 and falls through to mode 2 in the same frame.
//! - Mode 2 (0x92856): visible toggles with (G.$0000 + (47 - i)) & 1; moves by dx, dy;
//!   if Y < 0x1E (30), hits the floor: advances mode to 4, sets visible = 1, starts splash
//!   raw list at 0x9290E, and ticks it; otherwise ticks the falling raw list and adds to draw list.
//! - Mode 4 (0x9289C): ticks the splash raw list; when finished, frees with fx_free(i).

use super::*;

impl Game {
    /// 0x92808: pool C class 2 update (falling cave dust and rain drops).
    pub(super) fn fx_cave_drop(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => {
                // 0x9282A: mode 0 init
                let var = self.effects.slots[i].b.var26;
                let raw_addr = match var {
                    4 => 0x928DA,
                    8 => 0x928F4,
                    _ => 0x928C0,
                };
                let fx = self.fx_data();
                raw_load(&mut self.effects.slots[i].b, fx, fx.raw_at(raw_addr)); // 0x16C6
                self.effects.slots[i].b.draw_flags = 0x50; // 0x9283C: move.b #$50, $25(a6)
                // 0x92846: dx, dy from 0x9281E
                let (dx, dy): (i16, i16) = match var {
                    4 => (-6, -12),
                    8 => (-10, -10),
                    _ => (0, -14),
                };
                self.effects.slots[i].owner58 = dx as u16;
                self.effects.slots[i].owner_class = dy as u16;
                self.effects.slots[i].b.mode = 2; // 0x92852: addq.b #2, $4(a6)
                // 0x92852 falls through into mode 2
                self.fx_cave_drop_mode2(i);
            }
            2 => {
                self.fx_cave_drop_mode2(i);
            }
            4 => {
                // 0x9289C: mode 4 floor splash
                let fx = self.fx_data();
                if raw_tick(&mut self.effects.slots[i].b, fx) {
                    return self.fx_free(i); // 0x928AA: jmp 0x4964
                }
                self.draw_list_add_if_visible(self.effects.handle(i)); // 0x928A4: jmp 0x49C0
            }
            _ => {
                // 0x928AA: jmp 0x4964
                self.fx_free(i);
            }
        }
    }

    /// 0x92856: pool C class 2 mode 2 update (falling motion and floor hit test).
    fn fx_cave_drop_mode2(&mut self, i: usize) {
        // 0x92856: visible flag toggles every frame
        // In effects loop 0x237B2, d7 counts 47 down to 0, so d7 = 47 - i as u16.
        let visible = (self.w.frame.wrapping_add(47 - i as u16) & 1) as u8;
        self.effects.slots[i].b.visible = visible;

        // 0x92864: x += dx, y += dy
        let dx = self.effects.slots[i].owner58 as i16;
        let dy = self.effects.slots[i].owner_class as i16;
        let xi = self.effects.slots[i].b.xi().wrapping_add(dx);
        let yi = self.effects.slots[i].b.yi().wrapping_add(dy);
        self.effects.slots[i].b.set_xi(xi);
        self.effects.slots[i].b.set_yi(yi);

        // 0x92874: cmpi.w #$1E, $c(a6) ; bcs 0x92888 (unsigned lower)
        if (yi as u16) < 0x1E {
            // 0x92888: hit floor
            self.effects.slots[i].b.mode = 4;
            self.effects.slots[i].b.visible = 1; // 0x9288C: move.b #1, $1(a6)
            let fx = self.fx_data();
            raw_load(&mut self.effects.slots[i].b, fx, fx.raw_at(0x9290E)); // 0x92892, 0x16C6
            // 0x92896 falls through into 0x9289C (tick)
            if raw_tick(&mut self.effects.slots[i].b, fx) {
                return self.fx_free(i);
            }
            self.draw_list_add_if_visible(self.effects.handle(i));
            return;
        }

        // 0x9287C: tick falling raw list
        let fx = self.fx_data();
        raw_tick(&mut self.effects.slots[i].b, fx);
        self.draw_list_add_if_visible(self.effects.handle(i));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::effect::{RawFrame, RAW_FRAME};
    use dino_assets::AssetPack;

    fn test_game() -> Game {
        let mut a = AssetPack::default();
        // Register raw frames: 0x928C0 (falling) and 0x9290E (splash chain: dur 3, dur 2, dur 1)
        let frames = [
            (0x928C0u32, 2u8, false, 0),
            (0x9290Eu32, 3u8, false, 2),
            (0x92916u32, 2u8, false, 3),
            (0x9291Eu32, 1u8, true, 3),
        ];
        for &(addr, dur, last, next_idx) in &frames {
            let id = a.effects.raw_frames.len() as u32;
            a.effects.raw_frames.push(RawFrame {
                rom_addr: addr,
                list_addr: 0,
                sprites: Vec::new(),
                param_a: 0,
                param_b: 0,
                kind: 0,
                duration: dur,
                last,
                next: RAW_FRAME | next_idx,
            });
            a.effects.raw_by_addr.insert(addr, RAW_FRAME | id);
        }
        Game::new(Box::leak(Box::new(a)))
    }

    #[test]
    fn fx02_falls_and_splashes() {
        let mut g = test_game();
        let idx = g.effects.alloc().unwrap();
        {
            let e = &mut g.effects.slots[idx];
            e.b.live = 1;
            e.b.k20 = FX_CAVE_DROP;
            e.b.set_xi(100);
            e.b.set_yi(50); // above floor 30
            e.b.var26 = 0;
        }

        // Update 1: mode 0 inits, moves by dy = -14 -> Y = 36
        g.pool_c_update();
        assert_eq!(g.effects.slots[idx].b.mode, 2);
        assert_eq!(g.effects.slots[idx].b.yi(), 36);

        // Update 2: moves by dy = -14 -> Y = 22 (< 30), hits floor, switches to mode 4
        g.pool_c_update();
        assert_eq!(g.effects.slots[idx].b.mode, 4);
        assert_eq!(g.effects.slots[idx].b.visible, 1);

        // Updates 3..7: mode 4 ticks splash until finished (total 3+2+1=6 frames, 1 consumed on entry)
        for _ in 0..5 {
            g.pool_c_update();
        }
        assert_eq!(g.effects.slots[idx].b.live, 0);
    }
}
