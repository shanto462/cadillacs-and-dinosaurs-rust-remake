//! Pool A: enemies, bosses, dinosaurs (0x23420). Owner: enemy port. See `re/specs/enemy/SPEC.md`.

use super::Game;
use super::body::Body;

#[derive(Clone, Debug, Default)]
pub struct Enemy {
    pub b: Body,
}

impl Game {
    /// 0x3242E enemy census.
    pub fn enemy_census(&mut self) {}
    /// 0x23420 pool A update.
    pub fn enemies_update(&mut self) {}
}
