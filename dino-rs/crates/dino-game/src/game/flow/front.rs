//! Front end: system task 0x9D048 (coins, credits, start), attract / title 0x97C20, press start
//! screen 0x983EE and the character select (game mode 4, 0xAD46). Owner: flow / HUD port.
//! Spec: `re/specs/flow/SPEC.md` section 6 and `HANDOFF.md`. Not ported yet (stubs).

use crate::game::Game;
use crate::game::flow::scene::Scene;
use crate::game::vram::PaletteRam;

/// Character select struct G.$62C (cleared by 0xAD36).
#[derive(Clone, Copy, Debug, Default)]
pub struct Select {
    /// +3 taken cursor bits, +9 seconds left, +$A frame counter.
    pub taken: u8,
    pub seconds: u8,
    pub frames: u8,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Front {
    pub select: Select,
    /// G.-$702E set by the game over (0xAC20).
    pub game_over_flag: u8,
    /// G.$1A of the previous frame (G.$1F), for edge detection.
    pub prev_system: u8,
    /// G.$16..18 of the previous frame (G.$1B..1D), kept by `flow_frame`.
    pub prev_input: [u8; 3],
    /// The boot screens task 0x9DCF6 (the first screen of the T1.1 services, see below).
    pub boot: Scene,
}

/// Frames of black screen before the boot screens start: the boot task 0xA69A and the reset
/// code take this long before 0x9DCF6 runs (D4d section 2: the text is drawn at frame 73 of the
/// video, which is tick `BOOT_BLACK + 1` here). TEMP: T1.2 ports 0xA69A (EEPROM read, sound
/// reset codes 0xFF0B and 0xFF00 at frames 43 and 70) and replaces this wait.
pub const BOOT_BLACK: u16 = 70;

impl Game {
    /// Slot 0 system task 0x9D048. TODO(port): coin slots 0x9D4BE, credits 0x9D386, start
    /// 0x9D196 (edge on G.$1A bits 0-2), press-start screen. (`flow_frame` keeps the previous
    /// inputs G.$1F and G.$1B..1D.)
    pub fn flow_system_task(&mut self) {}

    /// TODO(port): attract task 0x97C20 (title 0x9A3CE, INSERT COIN blink 0x98874).
    pub fn flow_attract_start(&mut self) {
        self.flow.in_game = 0;
    }

    /// TODO(port): attract task body 0x97C20. For now it runs the boot screens only.
    pub fn flow_attract_task(&mut self) {
        self.flow_boot_screens();
    }

    /// Starts the boot screens task (T1.1: the first screen that exercises the text printers).
    pub fn flow_boot_screens_start(&mut self) {
        let mut s = Scene::start(0);
        s.sleep(BOOT_BLACK);
        self.flow.front.boot = s;
    }

    /// 0x9DCF6: the two boot text screens (game name and date code, then the legal warning), both
    /// for the region word G.$32, with the wait 0x9DD80 after each. The scene state is the script
    /// position, the sub state the position inside the wait routine.
    pub fn flow_boot_screens(&mut self) {
        if !self.flow.front.boot.tick() {
            return;
        }
        loop {
            match self.flow.front.boot.state {
                0 => {
                    // 0x9DCF6: fade counter = 15 (the bank mask of the fade control G.$44A),
                    // brightness 0 (0x8938), then sleep 1 (0xA76)
                    self.fade.mask = 0x0F;
                    self.brightness_low();
                    self.flow.front.boot.wait(1, 1);
                    return;
                }
                1 => {
                    self.boot_palette();
                    self.boot_text(0x9DDB2); // text 1: name, date code, region
                    self.flow.front.boot.state = 2;
                    self.flow.front.boot.sub = 0;
                }
                2 => {
                    if !self.boot_wait() {
                        return;
                    }
                    self.flow_clear_scroll1(); // 0xB52
                    self.boot_text(0x9DDAC); // text 2: the legal warning
                    self.flow.front.boot.state = 3;
                }
                _ => {
                    if self.boot_wait() {
                        self.flow_clear_scroll1(); // jmp 0xB52, then the boot task goes on:
                        self.flow_scene_palettes(0); // 0xA77E: palette set 0 (0x89E6)
                        self.brightness_full(); // 0x892C
                        self.flow.front.boot.kill();
                    }
                    return;
                }
            }
        }
    }

    /// 0x9DD0E: the scroll1 palettes 0 to 2 get 8 colours each from the 48 bytes at
    /// 0x9DDB8 + G.$32 * 0x20 (blue text for the World set).
    fn boot_palette(&mut self) {
        let t = &self.assets.frontend.text;
        let base = 0x9DDB8 + 0x20 * self.flow.region.0 as u32;
        for k in 0..3u32 {
            let words: Vec<u16> = (0..8).map(|i| t.u16(base, k * 16 + 2 * i)).collect();
            self.pal.copy(PaletteRam::SCROLL1 + 16 * k as usize, &words);
        }
    }

    /// 0x9DD2C / 0x9DD56: the text of the region (offset table entry G.$32 * 2, entry 0 when the
    /// region word is above 2) printed by 0xBB6.
    fn boot_text(&mut self, table: u32) {
        let t = &self.assets.frontend.text;
        let r = if self.flow.region.0 > 2 { 0 } else { self.flow.region.0 as u32 };
        let a = (table as i32 + t.i16(table, 2 * r) as i32) as u32;
        let s = t.slice_from(a);
        self.text_bytes(s, true);
    }

    /// 0x9DD80: sleep 30 frames (not skippable), then up to 91 more frames in steps of 1; an
    /// attack or jump (mask 0x30) of player 1 or 2 ends it at once. True when the wait is over.
    fn boot_wait(&mut self) -> bool {
        loop {
            match self.flow.front.boot.sub {
                0 => {
                    self.flow.front.boot.sleep(30);
                    self.flow.front.boot.sub = 1;
                    return false;
                }
                1 => {
                    self.flow.front.boot.count = 0x5A; // d5
                    self.flow.front.boot.sub = 2;
                }
                2 => {
                    if (self.w.input[0] | self.w.input[1]) & 0x30 != 0 {
                        self.flow.front.boot.sub = 0;
                        return true;
                    }
                    self.flow.front.boot.sleep(1);
                    self.flow.front.boot.sub = 3;
                    return false;
                }
                _ => {
                    // dbra d5: the loop ends when d5 passes 0
                    let b = &mut self.flow.front.boot;
                    b.count = b.count.wrapping_sub(1);
                    if b.count == 0xFFFF {
                        b.sub = 0;
                        return true;
                    }
                    b.sub = 2;
                }
            }
        }
    }

    /// Game mode 4 (0xAD46) character select. TODO(port): screen, cursors, timer (0xAF7C,
    /// 0xB06A, 0xB122, 0xB25E). For now the players keep their characters and the stage starts.
    pub fn flow_select_mode(&mut self) {
        self.flow_stage_start();
    }
}
