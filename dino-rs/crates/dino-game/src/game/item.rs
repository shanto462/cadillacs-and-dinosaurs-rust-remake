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
    pub fn pool_e_update(&mut self) {
        // Not ported: items stay as they are. Harness: they keep their draw state (`trace_hold`).
        if self.hold_unported {
            for i in 0..self.items.slots.len() {
                if self.items.slots[i].b.live != 0 {
                    let (h, b) = (self.items.handle(i), self.items.slots[i].b.clone());
                    self.trace_hold(h, &b);
                    self.item_hold_obstacle(h);
                }
            }
        }
    }

    /// Harness only (task 2 ports the container class): a class 5 container that is alive and idle
    /// (state 2 / 0, HP left, types 0, 1, 2, 5, 6 = 0x28F3A / 0x28F64) registers itself as an
    /// obstacle every frame (0x28F46 -> 0x4A96), so the terrain test (0x12E9A) of players and enemies
    /// sees it. The real handler 0x28C76 has more paths (hit flash, destruction) that skip this.
    fn item_hold_obstacle(&mut self, h: super::Handle) {
        let Some(i) = self.items.index_of(h) else { return };
        let it = &self.items.slots[i];
        let (b, hp) = (&it.b, it.m.w(0x6C)); // the header copy of +$6C is not loaded for items
        if b.k20 == 5 * 4 && b.mode == 2 && b.sub == 0 && hp != 0 && matches!(b.var26, 0 | 1 | 2 | 5 | 6) {
            self.item_register_obstacle(h);
        }
    }
}
