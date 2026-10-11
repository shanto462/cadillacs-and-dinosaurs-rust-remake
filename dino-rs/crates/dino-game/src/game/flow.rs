//! Game flow: attract, title, character select, stage start, stage clear, continue, game over.
//! Owner: flow / HUD port. Not ported yet: the game starts straight in a stage (`start_stage`).

use super::Game;

#[derive(Default)]
pub struct Flow {}

impl Game {
    /// Temporary entry used by `main`: what the original does between "start pressed" and the
    /// first in-game frame, reduced to what the ported parts need.
    pub fn start_stage(&mut self, stage: u8, chars: &[u8]) {
        self.w.stage = stage;
        // 0x983EE ends with 0x892C: all banks selected, full brightness.
        self.fade.mask = 0x0F;
        self.w.players_minus_one = chars.len().clamp(1, 3) as u8 - 1;
        for (i, &c) in chars.iter().take(3).enumerate() {
            self.player_join(i, c);
        }
    }

    /// Runs the current top-level mode for one frame.
    pub fn flow_frame(&mut self) {
        self.game_frame();
    }

    // Stage-level routines the camera calls during area changes (`camera.rs`). Not ported yet.

    /// 0xA5EA, called when an area-end lock is released (before the fade out).
    pub fn hook_a5ea(&mut self) {}
    /// 0xACD0, called after the fade out, before the next area is loaded.
    pub fn hook_acd0(&mut self) {}
    /// 0x12BE4, called after the next area's map is set up.
    pub fn hook_12be4(&mut self) {}
    /// 0x162AE, called before the fade in of the next area.
    pub fn hook_162ae(&mut self) {}
    /// 0xAAE2, 0x1031A, 0xACE0, 0x9236C: called by camera mode 8 (sub-area switch).
    pub fn hook_aae2(&mut self) {}
    pub fn hook_1031a(&mut self) {}
    pub fn hook_ace0(&mut self) {}
    pub fn hook_9236c(&mut self) {}
}
