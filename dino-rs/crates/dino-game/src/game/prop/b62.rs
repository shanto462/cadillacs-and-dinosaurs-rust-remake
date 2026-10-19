//! Pool B class 62 (0x865F2): Episode 5 village fire palettes and sound 0x8E.
//! Spec: D3e part 2 s7, A216, W232, P209, open item 2.
//!
//! Six palette phases, reload 8 (a step every 9 frames), sound 0x8E every 201 updates. The word
//! at CPS-B 0x800142 is recorded on `World::cpsb_800142` and is not applied to the picture.

use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x865F2: pool B class 62 update (jump table 0x865FE).
    pub fn prop_b62(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x86606
                p.m.sw(0xB8, 0);
                let reload = self.assets.enemy.t.u8(0x86720, 0); // d0 = 0
                p.m.sb(0xA5, reload);
                p.m.sb(0xA4, 0);
                p.m.sw(0xB0, 0);
                p.m.sw(0xB2, 0);
                p.m.sw(0xB4, 0);
                p.m.sw(0xB6, 0);
                p.b.mode = 2;
            }
            2 => self.prop_b62_run(p),
            _ => self.prop_free(p, i), // 0x8671A: jmp 0x4938
        }
    }

    /// 0x86632: sound, hardware toggle, stage guard, palette step.
    fn prop_b62_run(&mut self, p: &mut Prop) {
        // 0x86632: subq.w #1,+$B8 ; bcc skip sound
        let snd = p.m.w(0xB8).wrapping_sub(1);
        p.m.sw(0xB8, snd);
        if snd == 0xFFFF {
            // 0x86632: subq.w that borrows (was 0), bcc not taken
            p.m.sw(0xB8, 0xC8); // 0x86638: move.w #$C8,+$B8
            self.snd_request(0x8E, 0x10, 0); // 0x8663E: d0=0x8E, d1=0x10, d2=0 ; jsr 0xA63C
        }

        // 0x86650: hardware word at 0x800142 (open item 2: recorded, not shown)
        if p.m.w(0xB2) == 0 {
            let word = p.m.w(0xB0) ^ 0x1FF; // 0x86656: eori.w #$1FF,+$B0
            p.m.sw(0xB0, word);
            self.w.cpsb_800142 = word; // 0x8665C: move.w +$B0,$800142
            let idx = p.m.w(0xB4);
            let wait = self.assets.enemy.t.u16(0x86690, u32::from(idx) * 2); // 0x86664
            p.m.sw(0xB2, wait);
            p.m.sw(0xB4, idx.wrapping_add(1) & 1); // 0x86670 / 0x86674
        }
        p.m.sw(0xB2, p.m.w(0xB2).wrapping_sub(1)); // 0x8667A

        // 0x8667E: cmpi.b #4,$4D9 ; bne free
        if self.w.stage != 4 {
            p.b.mode = 4;
            return;
        }
        self.prop_b62_palettes(p); // 0x86694
    }

    /// 0x86694: one palette step when +$A4 underflows, six phases on +$B6.
    fn prop_b62_palettes(&mut self, p: &mut Prop) {
        let n = p.m.b(0xA4).wrapping_sub(1);
        p.m.sb(0xA4, n);
        if n != 0xFF {
            return;
        }
        p.m.sb(0xA4, p.m.b(0xA5));

        let mut lane = 0x8672Cu32;
        let mut pair = 0x866EEu32;
        loop {
            let list = self.assets.enemy.t.u32(lane, 0); // movea.l (a4)+,a0
            lane += 4;
            if (list as i32) < 0 {
                break;
            }
            let phase = p.m.w(0xB6);
            let set = self.assets.enemy.t.u32(list, u32::from(phase) * 4);
            let word_off = self.assets.enemy.t.u16(pair, 0);
            pair += 2;
            let dest = self.assets.enemy.t.u32(lane, 0).wrapping_add(u32::from(word_off) * 2);
            lane += 4;
            let count = self.assets.enemy.t.u16(pair, 0);
            pair += 2;
            self.prop_palette_copy(dest, set, count);
        }
        let next = p.m.w(0xB6).wrapping_add(1);
        p.m.sw(0xB6, if next >= 6 { 0 } else { next });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{slot_of, Slot};
    use dino_assets::enemy::ByteTables;
    use dino_assets::AssetPack;
    use std::path::Path;

    fn test_game(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs };
        let mut g = Game::new(Box::leak(Box::new(a)));
        g.w.stage = 4;
        g
    }

    fn longs(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|v| v.to_be_bytes()).collect()
    }

    fn words(w: &[u16]) -> Vec<u8> {
        w.iter().flat_map(|v| v.to_be_bytes()).collect()
    }

    fn rom_game() -> Option<Game> {
        let path = Path::new("../../../dino.zip");
        let alt = Path::new("../dino.zip");
        let p = if path.exists() {
            path
        } else if alt.exists() {
            alt
        } else {
            return None;
        };
        let assets: &'static AssetPack = Box::leak(Box::new(dino_extract::extract(p).expect("the ROM set loads")));
        Some(Game::new(assets))
    }

    #[test]
    fn b62_sound_and_six_palette_phases() {
        // one lane: dest 0x914A00, offset 0, count 16, six colour sets then terminator
        let mut lanes = longs(&[0x1000, 0x914000]);
        lanes.extend_from_slice(&0xFFFF_FFFFu32.to_be_bytes());
        let list = longs(&[0x2000, 0x2020, 0x2040, 0x2060, 0x2080, 0x20A0]);
        let mut blobs = vec![
            (0x86720, vec![8]),
            (0x86690, words(&[1, 1])),
            (0x866EE, words(&[0, 16])),
            (0x8672C, lanes),
            (0x1000, list),
        ];
        for (k, addr) in [0x2000u32, 0x2020, 0x2040, 0x2060, 0x2080, 0x20A0].into_iter().enumerate() {
            blobs.push((addr, words(&[0x0100 + k as u16; 16])));
        }
        let mut g = test_game(blobs);
        for w in &mut g.pal.words[0..16] {
            *w = 0xF000;
        }

        let h = g.spawn_prop(62, 0, Some((0xE00, 0x200, 0))).unwrap();
        let i = match slot_of(h).unwrap() {
            Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };

        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 2);
        assert_eq!(g.props.slots[i].m.b(0xA5), 8);
        assert_eq!(g.w.sounds, Vec::<u16>::new());

        // first running frame: sound, hardware toggle, first palette set
        g.pool_b_update();
        assert_eq!(g.w.sounds, vec![0x8E]);
        assert_eq!(g.w.cpsb_800142, 0x1FF);
        assert_eq!(g.pal.words[0], 0xF100);
        assert_eq!(g.props.slots[i].m.w(0xB6), 1);
        assert_eq!(g.props.slots[i].m.w(0xB8), 0xC8);

        // 8 countdown frames, then the next set
        for _ in 0..8 {
            g.w.sounds.clear();
            g.pool_b_update();
            assert_eq!(g.pal.words[0], 0xF100);
            assert!(g.w.sounds.is_empty());
        }
        g.pool_b_update();
        assert_eq!(g.pal.words[0], 0xF101);
        assert_eq!(g.props.slots[i].m.w(0xB6), 2);
        assert_eq!(g.w.priority_masks[3], 0, "hardware toggle is not shown");
    }

    #[test]
    fn b62_frees_outside_episode_5() {
        let Some(mut g) = rom_game() else { return };
        g.w.stage = 0;
        let h = g.spawn_prop(62, 0, None).unwrap();
        let i = match slot_of(h).unwrap() {
            Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 2);
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 4);
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.live, 0);
    }

    #[test]
    fn b62_rom_cycles_six_phases() {
        let Some(mut g) = rom_game() else { return };
        g.w.stage = 4;
        let h = g.spawn_prop(62, 0, None).unwrap();
        let i = match slot_of(h).unwrap() {
            Slot::Prop(pi) => pi,
            _ => unreachable!(),
        };
        g.pool_b_update();
        g.pool_b_update();
        assert_eq!(g.w.sounds, vec![0x8E]);
        let first = g.props.slots[i].m.w(0xB6);
        assert_eq!(first, 1);
        for _ in 0..(8 * 5 + 5) {
            g.pool_b_update();
        }
        assert_eq!(g.props.slots[i].m.w(0xB6), 0);
        assert_eq!(g.props.slots[i].b.live, 1);
    }
}
