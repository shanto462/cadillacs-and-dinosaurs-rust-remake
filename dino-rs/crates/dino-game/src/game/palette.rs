//! Palette loading, brightness and fades (0x8824..0x8E4C). See `re/specs/stage/SPEC.md` 2.3-2.4.

use super::Game;
use super::vram::PaletteRam;

/// Fade control block at G.$44A.
#[derive(Default)]
pub struct Fade {
    /// +0 bank mask: bit0 OBJ, bit1 scroll1, bit2 scroll2, bit3 scroll3. 0 = idle (callers poll it).
    pub mask: u8,
    /// +1 steps remaining.
    pub steps: u8,
    /// +2 frames to sleep between steps.
    pub delay: u16,
    /// The asynchronous fade task (created in task slot 0x40), if running.
    pub task: Option<FadeTask>,
}

#[derive(Clone, Copy)]
pub struct FadeTask {
    pub fade_in: bool,
    /// Frames left to sleep before the next step (0 = run this frame).
    pub sleep: u16,
    pub started: bool,
}

const BANKS: [usize; 4] = [PaletteRam::OBJ, PaletteRam::SCROLL1, PaletteRam::SCROLL2, PaletteRam::SCROLL3];

impl Game {
    /// 0x8E4C: OBJ palette slot i = ROM palette `list[i]` for the current stage and area.
    pub fn load_obj_palettes(&mut self) {
        let key = (self.w.stage, self.cam.s2.area);
        if let Some(set) = self.assets.obj_palette_sets.get(&key) {
            for (i, &n) in set.iter().enumerate() {
                let words = self.assets.obj_palette_bank[n as usize];
                self.pal.copy(PaletteRam::OBJ + i * 16, &words);
            }
        }
    }

    /// 0x8A44: OBJ, scroll1, scroll2, scroll3 palettes and the priority masks for this area.
    pub fn load_all_palettes(&mut self) {
        self.load_obj_palettes();
        let st = &self.assets.stages;
        self.pal.copy(PaletteRam::SCROLL1, &st.scroll1_palettes);
        let area = st.area(self.w.stage, self.cam.s2.area);
        self.pal.copy(PaletteRam::SCROLL2, &area.scroll2_palettes);
        self.pal.copy(PaletteRam::SCROLL3, &area.scroll3_palettes);
        self.w.priority_masks = area.priority_masks;
    }

    /// 0x892C: select all banks and set full brightness.
    pub fn brightness_full(&mut self) {
        self.fade.mask = 0x0F;
        self.brightness_full_current();
    }

    /// 0x8932: OR 0xF000 into every word of the banks in the current mask.
    pub fn brightness_full_current(&mut self) {
        for (i, &b) in BANKS.iter().enumerate() {
            if self.fade.mask & (1 << i) != 0 {
                self.pal.bank_mut(b / 512).iter_mut().for_each(|w| *w |= 0xF000);
            }
        }
    }

    /// 0x8938: clear the brightness nibble of the banks in the current mask.
    pub fn brightness_low(&mut self) {
        for (i, &b) in BANKS.iter().enumerate() {
            if self.fade.mask & (1 << i) != 0 {
                self.pal.bank_mut(b / 512).iter_mut().for_each(|w| *w &= 0x0FFF);
            }
        }
    }

    /// 0x88F4: one fade step on one bank. Only colour 0 is tested; every word changes (keep).
    fn fade_bank_step(&mut self, bank: usize, fade_in: bool) {
        let words = self.pal.bank_mut(bank);
        let probe = if fade_in { words[0].wrapping_add(0x1000) } else { words[0] };
        if probe & 0xF000 == 0 {
            return;
        }
        for w in words.iter_mut() {
            *w = if fade_in { w.wrapping_add(0x1000) } else { w.wrapping_sub(0x1000) };
        }
    }

    /// 0x88B0: one step on every bank in the mask, in order OBJ, scroll1, scroll2, scroll3.
    pub(crate) fn fade_step(&mut self, fade_in: bool) {
        for i in 0..4 {
            if self.fade.mask & (1 << i) != 0 {
                self.fade_bank_step(i, fade_in);
            }
        }
    }

    /// 0x8824 (fade in) / 0x8840 (fade out): start the fade task. The entry points themselves
    /// write steps = 16 and delay = 1, so a delay set by the caller is overwritten.
    pub fn start_fade_task(&mut self, fade_in: bool) {
        self.fade.steps = 16;
        self.fade.delay = 1;
        self.fade.task = Some(FadeTask { fade_in, sleep: 0, started: false });
    }

    /// Body of the fade task (0x885A), run once per frame by the task phase of the tick.
    pub fn run_fade_task(&mut self) {
        let Some(mut t) = self.fade.task else { return };
        if !t.started {
            t.started = true;
            if !(self.fade.mask != 0 && self.fade.delay != 0 && self.fade.steps != 0) {
                self.fade.mask = 0;
                self.fade.task = None;
                return;
            }
        }
        if t.sleep > 0 {
            t.sleep -= 1;
            if t.sleep > 0 {
                self.fade.task = Some(t);
                return;
            }
        }
        self.fade_step(t.fade_in);
        self.fade.steps -= 1;
        if self.fade.steps == 0 {
            self.fade.mask = 0;
            self.fade.task = None;
            return;
        }
        t.sleep = self.fade.delay;
        self.fade.task = Some(t);
    }
}
