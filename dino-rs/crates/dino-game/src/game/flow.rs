//! Game flow: attract, title, character select, stage start, continue, game over.
//! Not ported yet: the game starts straight in a stage (see `start_stage`).

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
}
