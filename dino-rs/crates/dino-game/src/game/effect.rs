//! Pool C (effects, sparks, shots; 0x237B2) and pool D (special).

use super::Game;
use super::body::Body;

#[derive(Clone, Debug, Default)]
pub struct Effect {
    pub b: Body,
}

#[derive(Clone, Debug, Default)]
pub struct Special {
    pub b: Body,
}

impl Game {
    /// 0x237B2 pool C update.
    pub fn pool_c_update(&mut self) {}
}
