//! Stage spawn interpreter (0xFBF4). Owner: enemy port.

use super::Game;

#[derive(Default)]
pub struct Spawn {}

impl Game {
    /// 0xFBF4: spawn lists driven by camera X. Runs last in the frame.
    pub fn spawn_script_tick(&mut self) {}
}
