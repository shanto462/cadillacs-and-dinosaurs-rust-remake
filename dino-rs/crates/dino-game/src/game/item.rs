//! Pool E: items, food, weapons, containers, projectiles (0x233C6). Owner: enemy port.

use super::Game;
use super::body::Body;
use super::enemy::Ram;

#[derive(Clone, Debug, Default)]
pub struct Item {
    pub b: Body,
    pub m: Ram,
}

impl Game {
    /// 0x24B36: item drop of a dying enemy (+$96 row of table 0xB6D12). Not ported yet.
    pub fn item_drop_from_enemy(&mut self, e: &mut super::enemy::Enemy) {
        e.m.sb(super::enemy::off::DROP, 0);
    }

    /// 0x233C6 pool E update.
    pub fn pool_e_update(&mut self) {}
}
