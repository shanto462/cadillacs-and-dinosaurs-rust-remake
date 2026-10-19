//! Collision phase (0x10546) and the per-frame object lists it reads.
//!
//! The pass routines are ported in `player_combat` (player-owned passes) and `enemy_combat`
//! (enemy-owned passes); this file keeps the original call order and the list bookkeeping.

use super::{Game, Handle};

/// Per-frame object lists, keyed by the RAM address of the original count word
/// (e.g. 0x6718 = collision list of players, pointer $6720, storage 0xE72E).
///
/// The original's lists are stacks that grow downwards (`move.w a6,-(a4)`, e.g. 0x4BEA) and every
/// pass walks them from the top pointer upwards (`movea.w $6726(a5),a1 ... (a1)+`), so the object
/// pushed last is visited first. `get` returns the list in that visiting order.
#[derive(Default)]
pub struct Lists {
    lists: Vec<(u16, Vec<Handle>)>,
}

impl Lists {
    /// The list in the order the original's passes visit it: newest push first.
    pub fn get(&self, id: u16) -> &[Handle] {
        self.lists.iter().find(|(k, _)| *k == id).map(|(_, v)| v.as_slice()).unwrap_or(&[])
    }

    pub fn push(&mut self, id: u16, h: Handle) {
        match self.lists.iter_mut().find(|(k, _)| *k == id) {
            Some((_, v)) => v.insert(0, h),
            None => self.lists.push((id, vec![h])),
        }
    }

    pub fn clear(&mut self, id: u16) {
        if let Some((_, v)) = self.lists.iter_mut().find(|(k, _)| *k == id) {
            v.clear();
        }
    }

    /// Copies list `from` into list `to` (the original keeps last frame's list 4 in $69BA).
    pub fn copy(&mut self, from: u16, to: u16) {
        let v = self.get(from).to_vec();
        self.clear(to);
        if let Some((_, l)) = self.lists.iter_mut().find(|(k, _)| *k == to) {
            *l = v;
        } else {
            self.lists.push((to, v));
        }
    }

    /// Part of 0xE90E: item list ($69EE / $69F0) reset before pool E runs.
    pub fn items_reset(&mut self) {
        self.clear(0x69EE);
    }
}

impl Game {
    /// 0x10546: all hit tests of the frame, in the original order.
    pub fn collision(&mut self) {
        self.pass_10664(); // player vs player
        if self.w.scripted == 0 {
            self.pass_10dcc();
        }
        self.pass_10f96();
        self.pass_1132a();
        self.pass_107f8(); // player attacks and grabs vs enemies
        self.pass_10cd0();
        self.pass_11962();
        self.pass_11a9e();
        self.pass_11f86();
        self.pass_1201c();
        self.pass_120d2();
        self.pass_11e56();
        // The car passes (car.rs, M6-20): only in the drive (G.$518) with the car in place (G.$77A0); the monsters hit the car only
        // while the stage end lock G.$7B4 is set (after Hogg exists).
        if self.spawn.g.car_mode != 0 && self.spawn.g.car_on != 0 {
            self.pass_121d8();
            self.pass_122d8();
            self.pass_12342();
            if self.cam.s2.lock78 != 0 {
                self.pass_12454();
            }
        }
        if self.w.scripted == 0 {
            self.pass_115b0(); // enemy attacks vs players
            self.pass_11cec();
        }
        self.pass_1185a();
    }

    /// 0x105E8: reset the per-frame lists (list 0x671E is kept one frame in 0x69BA).
    pub fn combat_lists_reset(&mut self) {
        self.combat.clear(0x6718);
        self.combat.clear(0x671A);
        self.combat.copy(0x671E, 0x69BA);
        self.combat.clear(0x671E);
        self.combat.clear(0x671C);
        for id in [0x6B64, 0x6AC2, 0x6AFC, 0x6B30] {
            self.combat.clear(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The passes visit the newest push first (the original's lists are downward stacks).
    #[test]
    fn lists_are_visited_newest_first() {
        let mut l = Lists::default();
        for h in [0xC8F4, 0xC9D4, 0xCAB4] {
            l.push(0x671E, h);
        }
        assert_eq!(l.get(0x671E), &[0xCAB4, 0xC9D4, 0xC8F4]);
        l.copy(0x671E, 0x69BA);
        assert_eq!(l.get(0x69BA), l.get(0x671E));
        l.clear(0x671E);
        assert!(l.get(0x671E).is_empty());
    }
}
