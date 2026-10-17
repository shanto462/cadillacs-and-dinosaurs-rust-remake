//! Pool B classes 61 (0x864DA) and 63 (0x869E6): Episode 7 lab light palette cycles.
//! Spec: D3e part 3 sections 2.1, 4.4, A18, W313.
//!
//! - B61 (0x864DA): 4 scroll2 palettes (84, 85, 86, 80) cycle 3 steps every 17 frames;
//!   queues sound 0x6D (d1 = 0x10, d2 = 0) every 121 frames; active only in stage 6.
//! - B63 (0x869E6): 10 scroll2 palettes (79, 67, 69, 89, 66, 91, 83, 87, 88, 82)
//!   cycle 2 steps every 17 frames; active only in stage 6.

use super::drivers::{PaletteCycler, PaletteLane};
use super::Prop;
use crate::game::Game;

/// 0x86596: B61 palette lanes (4 scroll2 palettes).
const LANES_B61: &[PaletteLane] = &[
    PaletteLane { dest: 0x914A80, count: 16, list: 0x865BA }, // palette 84
    PaletteLane { dest: 0x914AA0, count: 16, list: 0x865C8 }, // palette 85
    PaletteLane { dest: 0x914AC0, count: 16, list: 0x865D6 }, // palette 86
    PaletteLane { dest: 0x914A00, count: 16, list: 0x865E4 }, // palette 80
];

/// B61 palette cycler: reload 16 (every 17 frames), 3 steps.
const CYCLER_B61: PaletteCycler<'static> = PaletteCycler {
    reload: 16,
    steps: 3,
    lanes: LANES_B61,
    counter: PaletteCycler::COUNTER,
    index: PaletteCycler::INDEX,
};

/// 0x86A84: B63 palette lanes (10 scroll2 palettes).
const LANES_B63: &[PaletteLane] = &[
    PaletteLane { dest: 0x9149E0, count: 16, list: 0x86AD8 }, // palette 79
    PaletteLane { dest: 0x914860, count: 16, list: 0x86AE2 }, // palette 67
    PaletteLane { dest: 0x9148A0, count: 16, list: 0x86AEC }, // palette 69
    PaletteLane { dest: 0x914B20, count: 16, list: 0x86AF6 }, // palette 89
    PaletteLane { dest: 0x914840, count: 16, list: 0x86B00 }, // palette 66
    PaletteLane { dest: 0x914B60, count: 16, list: 0x86B0A }, // palette 91
    PaletteLane { dest: 0x914A60, count: 16, list: 0x86B14 }, // palette 83
    PaletteLane { dest: 0x914AE0, count: 16, list: 0x86B1E }, // palette 87
    PaletteLane { dest: 0x914B00, count: 16, list: 0x86B28 }, // palette 88
    PaletteLane { dest: 0x914A40, count: 16, list: 0x86B32 }, // palette 82
];

/// B63 palette cycler: reload 16 (every 17 frames), 2 steps.
const CYCLER_B63: PaletteCycler<'static> = PaletteCycler {
    reload: 16,
    steps: 2,
    lanes: LANES_B63,
    counter: PaletteCycler::COUNTER,
    index: PaletteCycler::INDEX,
};

impl Game {
    /// 0x864DA: pool B class 61 update (lab lights A).
    pub fn prop_b61(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x864EE: init
                CYCLER_B61.start(p);
                p.m.sw(0xB2, 0); // clr.w $b2(a6): sound countdown
                p.b.mode = 2; // addq.b #2, $4(a6)
            }
            2 => {
                // 0x86510: sound countdown (reload 0x78 = 120, period 121 frames)
                let snd_t = p.m.w(0xB2).wrapping_sub(1);
                p.m.sw(0xB2, snd_t);
                if (snd_t as i16) < 0 {
                    p.m.sw(0xB2, 0x78);
                    self.snd_request(0x6D, 0x10, 0);
                }
                // 0x86530: stage guard (cmpi.b #6, $4D9(a5))
                if self.w.stage != 6 {
                    p.b.mode = 4;
                    return;
                }
                // 0x86542: step cycler
                self.prop_palette_cycle(p, &CYCLER_B61);
            }
            _ => {
                // 0x8658C: jmp 0x4938
                self.prop_free(p, i);
            }
        }
    }

    /// 0x869E6: pool B class 63 update (lab lights B).
    pub fn prop_b63(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x869F4: init
                CYCLER_B63.start(p);
                p.b.mode = 2; // addq.b #2, $4(a6)
            }
            2 => {
                // 0x86A18: stage guard (cmpi.b #6, $4D9(a5))
                if self.w.stage != 6 {
                    p.b.mode = 4;
                    return;
                }
                // 0x86A2A: step cycler
                self.prop_palette_cycle(p, &CYCLER_B63);
            }
            _ => {
                // 0x86A74: jmp 0x4938
                self.prop_free(p, i);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use dino_assets::enemy::ByteTables;

    fn test_game(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs };
        Game::new(Box::leak(Box::new(a)))
    }

    #[test]
    fn b61_cycles_palettes_and_queues_sound() {
        // Prepare dummy step pointers and palette color sets for B61
        let mut blobs = Vec::new();
        // 4 lanes: each has 3 step pointers of 4 bytes
        for (i, base) in [0x865BA, 0x865C8, 0x865D6, 0x865E4].iter().enumerate() {
            let mut ptr_bytes = Vec::new();
            for step in 0..3 {
                let color_addr = 0xBF000 + ((i * 3 + step) * 32) as u32;
                ptr_bytes.extend_from_slice(&color_addr.to_be_bytes());
            }
            blobs.push((*base, ptr_bytes));
        }
        // Palette words: 16 words each for all 12 sets
        for i in 0..12 {
            let color_addr = 0xBF000 + (i * 32) as u32;
            let mut words = Vec::new();
            for w in 0..16 {
                words.extend_from_slice(&(0x0111u16 * (w + 1) as u16).to_be_bytes());
            }
            blobs.push((color_addr, words));
        }

        let mut g = test_game(blobs);
        g.w.stage = 6;
        let mut p = Prop::default();
        p.b.live = 1;
        p.b.k20 = 61 * 4;

        // Init frame
        g.prop_b61(&mut p, 0);
        assert_eq!(p.b.mode, 2);

        // Frame 1: first step fires, sound 0x6D queued
        g.prop_b61(&mut p, 0);
        assert_eq!(p.m.w(0xB0), 1);
        assert_eq!(p.m.w(0xB2), 0x78);
        let reqs = g.snd_take_requests();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].code, 0x6D);

        // Frames 2..17: counting down, no sound, no palette step
        for _ in 0..16 {
            g.prop_b61(&mut p, 0);
            assert_eq!(p.m.w(0xB0), 1);
            assert!(g.snd_take_requests().is_empty());
        }

        // Frame 18 (17 frames after frame 1): second step fires
        g.prop_b61(&mut p, 0);
        assert_eq!(p.m.w(0xB0), 2);

        // Advance until stage changes: prop should free
        g.w.stage = 5;
        g.prop_b61(&mut p, 0);
        assert_eq!(p.b.mode, 4);
    }

    #[test]
    fn b63_cycles_10_palettes_in_2_steps() {
        let mut blobs = Vec::new();
        let lane_bases = [
            0x86AD8, 0x86AE2, 0x86AEC, 0x86AF6, 0x86B00,
            0x86B0A, 0x86B14, 0x86B1E, 0x86B28, 0x86B32,
        ];
        for (i, base) in lane_bases.iter().enumerate() {
            let mut ptr_bytes = Vec::new();
            for step in 0..2 {
                let color_addr = 0xBF000 + ((i * 2 + step) * 32) as u32;
                ptr_bytes.extend_from_slice(&color_addr.to_be_bytes());
            }
            blobs.push((*base, ptr_bytes));
        }
        for i in 0..20 {
            let color_addr = 0xBF000 + (i * 32) as u32;
            let mut words = Vec::new();
            for w in 0..16 {
                words.extend_from_slice(&(0x0222u16 * (w + 1) as u16).to_be_bytes());
            }
            blobs.push((color_addr, words));
        }

        let mut g = test_game(blobs);
        g.w.stage = 6;
        let mut p = Prop::default();
        p.b.live = 1;
        p.b.k20 = 63 * 4;

        g.prop_b63(&mut p, 0);
        assert_eq!(p.b.mode, 2);

        // Frame 1: step 0 fires, index advances to 1
        g.prop_b63(&mut p, 0);
        assert_eq!(p.m.w(0xB0), 1);

        // Next 16 frames
        for _ in 0..16 {
            g.prop_b63(&mut p, 0);
            assert_eq!(p.m.w(0xB0), 1);
        }

        // Frame 18: step 1 fires, index wraps to 0
        g.prop_b63(&mut p, 0);
        assert_eq!(p.m.w(0xB0), 0);
    }
}
