//! Player-side collision passes (owner: player port). See `re/specs/player/SPEC.md` section 6.

use super::Game;

impl Game {
    /// 0x10664 player vs player.
    pub fn pass_10664(&mut self) {}
    /// 0x107F8 player attacks vs enemies, grab test (0x108AE), combo-throw probe (0x108FA).
    pub fn pass_107f8(&mut self) {}
}
