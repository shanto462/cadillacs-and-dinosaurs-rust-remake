//! Review aids (M3-05): debug keys (windowed runs, only with `--debug-keys`, off by default) and the state they share with the
//! autoplay bot. Keys: F2 kill all, F3 skip wave, F4 set the stage time to 5:00, F5 god mode, F6 next area. Ctrl, Alt and the
//! TEST input are never used.
//!
//! Kill all is the original's G.$50E flag 2 (no credit, `enemy_kill_all_check`). It is set for one tick and cleared after it,
//! so only the enemies alive in that tick die. God mode is a harness-side guard, not the original: after each tick a player
//! whose HP fell gets its HP back (a death in the same tick can still happen).

use crate::game::Game;
use macroquad::prelude::*;

#[derive(Default)]
pub struct Debug {
    /// `--debug-keys`: the keys are read in windowed runs.
    pub enabled: bool,
    /// Kill all for the tick being run.
    kill: bool,
    /// God mode: the HP of each player when it was turned on.
    god: Option<[u16; 3]>,
}

impl Debug {
    /// `enabled` is `--debug-keys`.
    pub fn new(enabled: bool) -> Debug {
        Debug { enabled, ..Default::default() }
    }

    /// Reads the debug keys of this frame (windowed runs).
    pub fn keys(&mut self, g: &mut Game) {
        if !self.enabled {
            return;
        }
        if is_key_pressed(KeyCode::F2) {
            self.kill_all();
        }
        if is_key_pressed(KeyCode::F3) {
            self.skip_wave(g);
        }
        if is_key_pressed(KeyCode::F4) {
            g.rank_set_time(0x0500);
        }
        if is_key_pressed(KeyCode::F5) {
            self.god = match self.god {
                Some(_) => None,
                None => Some(std::array::from_fn(|i| g.players[i].b.hp)),
            };
        }
        if is_key_pressed(KeyCode::F6) {
            next_area(g);
        }
    }

    /// Kill all on the next tick (G.$50E = 2).
    pub fn kill_all(&mut self) {
        self.kill = true;
    }

    /// Skip wave: kill the enemies and end the wave timer, so the controller moves to its next group.
    pub fn skip_wave(&mut self, g: &mut Game) {
        self.kill_all();
        g.spawn.g.wave_timer = 0;
    }

    /// Before a tick: the kill-all flag and the god mode HP.
    pub fn before_tick(&mut self, g: &mut Game) {
        if self.kill {
            g.spawn.g.kill_all = 2;
        }
    }

    /// After a tick: the kill-all flag goes back to 0, god mode gives the HP back.
    pub fn after_tick(&mut self, g: &mut Game) {
        if self.kill {
            if g.spawn.g.kill_all == 2 {
                g.spawn.g.kill_all = 0;
            }
            self.kill = false;
        }
        if let Some(hp) = self.god {
            for (p, &h) in g.players.iter_mut().zip(hp.iter()) {
                if p.b.live != 0 && p.b.hp < h {
                    p.b.hp = h;
                }
            }
        }
    }
}

/// Next area: the camera's area end with no fade (mode 1, sub 4 = `area_end_next_area`), which takes the next area of the
/// camera events.
pub fn next_area(g: &mut Game) {
    g.cam.s2.mode = 1;
    g.cam.s2.sub = 4;
}
