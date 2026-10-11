//! Join / continue task 0x6C4A (player +$A2 states 0..0xC) and the all-players-out continue
//! mode 0x10 (0xE9E6). Owner: flow / HUD port. Spec: `re/specs/flow/SPEC.md` section 7.
//! Not ported yet (stubs).

use crate::game::Game;

/// Per-player join / continue state (player +$A2, +$B0 seconds, +$B1 frames).
#[derive(Clone, Copy, Debug, Default)]
pub struct Join {
    pub state: u8,
    pub seconds: u8,
    pub frames: u8,
}

impl Game {
    /// Mode 0 creates the task 0x6C4A in slot 0x10.
    pub fn flow_join_start(&mut self) {
        self.flow.join = Default::default();
    }

    /// TODO(port): task 0x6C4A body (INSERT COIN / PRESS START / select timer / CONTINUE
    /// countdown / GAME OVER, mode switch to 0x10 when nobody plays).
    pub fn flow_join_task(&mut self) {}

    /// TODO(port): 0x7562 redraw of the join prompts of absent players.
    pub fn flow_join_prompts(&mut self) {}

    /// Game mode 0x10 (0xE9E6). TODO(port): continue screen; for now game over (mode 0x14).
    pub fn flow_continue_mode(&mut self) {
        self.flow.mode = 0x14;
    }
}
