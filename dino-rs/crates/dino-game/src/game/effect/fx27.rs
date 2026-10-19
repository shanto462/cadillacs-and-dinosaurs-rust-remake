//! Pool C class 27 (`+$20` = 0x6C, handler 0x963D6): the afterimage of a running Slice (and of the Slisaurs). Made by 0x9639E (`fx_spawn_afterimage`)
//! at the runner's position with his animation frame (`+$28`) and facing. It does not move: six steps of 6 frames, each with the palette byte
//! `+$23` = 0x80 + the step's byte of 0x9644A (a fade), drawn only on frames where the counter `+$5B` is odd, then freed. D3c 6.5, D3f class 27.

use super::*;

/// 0x9644A: the palette bytes of the six steps.
const PALETTES: u32 = 0x9644A;

impl Game {
    /// 0x9639E: an afterimage of `o` (pool C, nothing when it is full). `+$26` gets the caller's d1 (`d1`, unused by the ghost).
    pub fn fx_spawn_afterimage(&mut self, o: &Body, d1: u16) {
        let Some(j) = self.fx_alloc() else { return };
        let e = &mut self.effects.slots[j];
        e.b.live = 1;
        e.b.k20 = FX_AFTERIMAGE;
        set_word(&mut e.b.x, o.xi());
        set_word(&mut e.b.y, o.yi());
        set_word(&mut e.b.z, o.zi());
        e.b.frame = o.frame;
        e.b.facing = o.facing;
        e.b.var26 = d1;
    }

    /// 0x963D6 (mode table 0x963E2: 0 the init 0x963EA falling into the run 0x963FC, 4 and 6 the free 0x96430).
    pub(super) fn fx_afterimage(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => {
                let e = &mut self.effects.slots[i];
                e.b.mode = e.b.mode.wrapping_add(2);
                e.owner58 = 0;
                self.fx_afterimage_palette(i);
                let e = &mut self.effects.slots[i];
                e.owner_class = (e.owner_class & 0xFF) | 0x0600; // +$5A = 6
                self.fx_afterimage_run(i);
            }
            2 => self.fx_afterimage_run(i),
            _ => self.fx_free(i),
        }
    }

    /// 0x963FC: every 6 frames (`+$5A`) the next step (`+$58`); after the sixth the free (0x4964); `+$5B` counts down and the ghost is drawn
    /// (0x49CA) when it is odd.
    fn fx_afterimage_run(&mut self, i: usize) {
        let e = &mut self.effects.slots[i];
        let t = (e.owner_class >> 8) as u8;
        if t == 0 {
            e.owner_class = (e.owner_class & 0xFF) | 0x0600;
            e.owner58 = e.owner58.wrapping_add(1);
            if (e.owner58 as i16) >= 6 {
                return self.fx_free(i);
            }
            self.fx_afterimage_palette(i);
        } else {
            e.owner_class = (e.owner_class & 0xFF) | ((t - 1) as u16) << 8;
        }
        let e = &mut self.effects.slots[i];
        let b = (e.owner_class as u8).wrapping_sub(1);
        e.owner_class = (e.owner_class & 0xFF00) | b as u16;
        if b & 1 != 0 {
            self.draw_list_add_check(self.effects.handle(i));
        }
    }

    /// 0x96436: `+$23` = 0x80 + the palette byte of step `+$58`.
    fn fx_afterimage_palette(&mut self, i: usize) {
        let k = self.effects.slots[i].owner58 as u32;
        let p = self.assets.enemy.t.u8(PALETTES, k).wrapping_add(0x80);
        self.effects.slots[i].b.palette_override = p;
    }
}
