//! Players (0x18834 and the state machine). Owner: player port. See `re/specs/player/SPEC.md`.

use super::Game;
use super::body::Body;

/// Player object (0x180 bytes in the original). `b` holds the shared header; player-only fields follow.
#[derive(Clone, Debug, Default)]
pub struct Player {
    pub b: Body,
}

impl Game {
    /// Player `i` joins with character `chr` (0 Jack, 1 Mustapha, 2 Hannah, 3 Mess):
    /// init 0x4D46 / 0x4D5E / 0x4D76 plus what the select screen sets.
    pub fn player_join(&mut self, _i: usize, _chr: u8) {}

    /// 0x18834: all players (P1, P2, P3).
    pub fn players_update(&mut self) {}
    /// 0x188DA: players currently held by an enemy.
    pub fn held_players_update(&mut self) {}
}
