//! Pool E: items, food, weapons, containers, projectiles (0x233C6). Owner: enemy port.

use super::Game;
use super::body::Body;

#[derive(Clone, Debug, Default)]
pub struct Item {
    pub b: Body,
}

impl Game {
    /// 0x233C6 pool E update.
    pub fn pool_e_update(&mut self) {}
}
