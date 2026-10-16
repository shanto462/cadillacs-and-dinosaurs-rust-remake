//! Pool C classes of the stage clear tally (`flow/tally.rs`, M2-24):
//!
//! * class 22 (`+$20 = 0x58`, handler 0x95840, made by 0x9582A at the first panel): the colours of the number sprites. Mode 0 (0x95854) copies the ROM
//!   palette 0xB8612 (with the brightness nibble forced to F) into the sprite palettes 0xB and 0xC (palette RAM 0x914160 and 0x914180), mode 2
//!   (0x95898) then changes the palette 0xB every 4 frames between the two palettes 0xB8612 and 0xB8632, so the numbers of the best hero (palette override
//!   0x8B) blink and the others (0x8C) do not. It never ends by itself: the effect pool is cleared at the next area start.
//! * class 34 (`+$20 = 0x88`, handler 0x96DC4, made by 0x96DAA with the address of the label in `+$5A`): the sparkle of the WINNER! label. Every 5 frames
//!   `+$58` goes through 0, 2, 0, 2 and the palette of the 8 columns of the label (the low five bits of the attribute words of 16 cells) is set to it.

use super::{FX_TALLY_FLASH, FX_TALLY_WINNER};
use crate::game::Game;

/// The two palettes the flash alternates between.
const PALETTES: [u32; 2] = [0xB8612, 0xB8632];

impl Game {
    /// 0x9582A: the flash of the number palettes.
    pub fn fx_spawn_tally_flash(&mut self) {
        let Some(i) = self.fx_alloc() else { return };
        let b = &mut self.effects.slots[i].b;
        b.live = 1;
        b.k20 = FX_TALLY_FLASH;
    }

    /// 0x96DAA: the sparkle of the label at the scroll1 address `label` (the attribute word of its first cell).
    pub fn fx_spawn_tally_winner(&mut self, label: u32) {
        let Some(i) = self.fx_alloc() else { return };
        let e = &mut self.effects.slots[i];
        e.b.live = 1;
        e.b.k20 = FX_TALLY_WINNER;
        e.owner_class = (label >> 16) as u16;
        e.w5c = label as u16;
    }

    /// The 16 words of the ROM palette `rom` with the brightness F, into the sprite palette `n` (0xB or 0xC).
    fn fx_tally_palette(&mut self, rom: u32, n: usize) {
        let t = &self.assets.frontend.text;
        let words: Vec<u16> = (0..16).map(|k| t.u16(rom, 2 * k) | 0xF000).collect();
        self.pal.copy(16 * n, &words);
    }

    /// 0x95840 pool C class 0x58 (jump table 0x9584C: mode 0 -> 0x95854, 2 -> 0x95898, 4 and 6 -> 0x958DE).
    pub(super) fn fx_tally_flash(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => {
                let e = &mut self.effects.slots[i];
                e.b.mode += 2;
                e.owner58 = 0; // +$58 and +$59
                self.fx_tally_palette(PALETTES[0], 0xB);
                self.fx_tally_palette(PALETTES[0], 0xC);
            }
            2 => {
                let e = &mut self.effects.slots[i];
                let (toggle, count) = ((e.owner58 >> 8) as u8, e.owner58 as u8);
                let borrow = count == 0;
                e.owner58 = (toggle as u16) << 8 | count.wrapping_sub(1) as u16;
                if !borrow {
                    return; // subq.b #1,$59(a6) / bcc
                }
                let toggle = (toggle + 1) & 1;
                e.owner58 = (toggle as u16) << 8 | 3;
                self.fx_tally_palette(PALETTES[toggle as usize], 0xB);
            }
            _ => self.fx_free(i),
        }
    }

    /// 0x96DC4 pool C class 0x88 (jump table 0x96DD0: mode 0 -> 0x96DD8, 2 -> 0x96DE4, 4 and 6 -> 0x96E2C).
    pub(super) fn fx_tally_winner(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => {
                let e = &mut self.effects.slots[i];
                e.b.mode += 2;
                e.owner58 = 0;
                self.fx_tally_winner_step(i); // no rts: 0x96DE4 follows
            }
            2 => self.fx_tally_winner_step(i),
            _ => self.fx_free(i),
        }
    }

    /// 0x96DE4: every 5 frames the phase goes 0, 2, 0, 2 and the label takes it as its palette.
    fn fx_tally_winner_step(&mut self, i: usize) {
        let e = &mut self.effects.slots[i];
        let (phase, count) = ((e.owner58 >> 8) as u8, e.owner58 as u8);
        let borrow = count == 0;
        e.owner58 = (phase as u16) << 8 | count.wrapping_sub(1) as u16;
        if !borrow {
            return;
        }
        let phase = (phase + 2) & 3;
        e.owner58 = (phase as u16) << 8 | 4;
        let mut a0 = (e.owner_class as u32) << 16 | e.w5c as u32;
        // 0x96E00: the palette of 8 columns of two cells
        for _ in 0..8 {
            for _ in 0..2 {
                let attr = self.hud_read(a0);
                self.hud_word(a0, (attr & 0xFFE0) | phase as u16);
                a0 = a0.wrapping_add(4);
            }
            a0 = a0.wrapping_add(0x78);
        }
    }
}
