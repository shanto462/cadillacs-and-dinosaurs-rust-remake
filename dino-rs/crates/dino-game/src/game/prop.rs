//! Pool B: controllers and props (0x23604), e.g. the wave controller (class 15). Owner: enemy port.

use super::Game;
use super::body::Body;

#[derive(Clone, Debug, Default)]
pub struct Prop {
    pub b: Body,
}

impl Game {
    /// 0x23604 pool B update.
    pub fn pool_b_update(&mut self) {}
}
