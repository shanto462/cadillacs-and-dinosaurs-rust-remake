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

/// Next area: the camera's area change without a fade (`area_end_next_area(false)`, 0x20D3A at sub 4), which takes the next
/// area of the camera events (the lift of Episode 7 uses the mode 1 path, see below). The standard controller
/// (0x206D4) calls that code mode 0x12; the controller of Episode 3 (0x22398, stage 2) has no mode 0x12, its mode 6 (0x22864)
/// is the same routine.
///
/// The mode 0x12 search (`area_end_next_area`) takes the first record with X 0 after the next event. An area with no such record
/// ahead (Episode 8 area 2: its change is the mode 8 sub-area record, and the list ends with the X 0xFFFF terminator) would run
/// past the list and set area 0. There this does nothing: the camera follows the hero, and the stage's own records make the
/// change. (A camera jump there would skip the wave records of the area, so the lock never ends.) Episode 3 has one area, so
/// it is left as it was.
pub fn next_area(g: &mut Game) {
    let events = g.assets.stages.stages.get(g.w.stage as usize).map(|s| &s.camera_events[..]).unwrap_or(&[]);
    let from = g.cam.s2.next_event;
    let has_start = events.iter().skip(from).any(|e| e.x == 0);
    // Episode 3 (stage 2) has one area and keeps its mode 6 path (the test `the_area_change_key_ends_in_episode_3`).
    if g.w.stage != 2 && !events.is_empty() && !has_start {
        return;
    }
    // The lift (Episode 7 area 0 to 1) is entered through the area change with the fade (mode 1): its sub 6 redraws the HUD over the
    // cage that the area setup has drawn (0x20CC0 -> 0x162AE). The hard cut (mode 0x12) would leave the cage on top of the HUD.
    let lift = g.w.stage == 6 && g.cam.s2.area == 0;
    if lift {
        g.fade.mask = 0; // a direct start (`--episode`) leaves the mask of its palette load: mode 1 sub 4 waits for it to be 0
    }
    g.cam.s2.mode = if g.w.stage == 2 { 6 } else if lift { 1 } else { 0x12 };
    g.cam.s2.sub = 4;
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn rom() -> Option<&'static AssetPack> {
        static A: std::sync::OnceLock<Option<&'static AssetPack>> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let path = std::path::Path::new("../../dino.zip");
            let path = if path.exists() { path } else { std::path::Path::new("../../../dino.zip") };
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// The area change of the debug key: mode 0x12 in the standard controller (0x206D4), mode 6 in the controller of Episode 3
    /// (0x22398, stage 2: it has no mode 0x12, its table 0x2250A stops at 8). Both at sub 4 (0x20D3A / 0x22864 + 0xD6).
    #[test]
    fn the_area_change_key_uses_the_mode_of_the_stage_controller() {
        let mut g = Game::new(Box::leak(Box::new(AssetPack::default())));
        for (stage, mode) in [(0, 0x12), (1, 0x12), (2, 6), (3, 0x12), (6, 1), (7, 0x12)] {
            g.w.stage = stage;
            g.cam.s2.area = 0;
            next_area(&mut g);
            assert_eq!((g.cam.s2.mode, g.cam.s2.sub), (mode, 4), "stage {stage}");
        }
    }

    /// Episode 8 area 2 has no area-start record (X 0) after its lock: the change is the mode 8 sub-area record, and the list ends
    /// with the X 0xFFFF terminator. The key must not start the mode 0x12 search there (it ran past the list and set area 0).
    #[test]
    fn the_area_change_key_does_not_run_past_the_event_list() {
        let Some(assets) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let mut g = Game::new(assets);
        g.start_stage_at(7, 2, 0, &[0]);
        g.cam.s2.next_event = 13;
        let before = (g.cam.s2.mode, g.cam.s2.sub);
        next_area(&mut g);
        assert_eq!((g.cam.s2.mode, g.cam.s2.sub), before);
    }

    /// In Episode 3 the key leads through the area change (modes 6, sub 4 then 6) back to the free camera (mode 0), as in the
    /// other stages. The test starts the stage like `--episode 3`.
    #[test]
    fn the_area_change_key_ends_in_episode_3() {
        let Some(assets) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let mut g = Game::new(assets);
        g.start_stage_at(2, 0, 0, &[0]);
        let mut free = None;
        for t in 0..3000 {
            g.tick();
            if g.flow.mode == 0xC && g.w.scripted == 0 && g.cam.s2.mode == 0 && g.cam.s2.state == 2 {
                free = Some(t);
                break;
            }
        }
        assert!(free.is_some(), "the camera is free after the opening");
        next_area(&mut g);
        let mut seen = vec![(g.cam.s2.mode, g.cam.s2.sub)];
        for _ in 0..40 {
            g.tick();
            let s = (g.cam.s2.mode, g.cam.s2.sub);
            if seen.last() != Some(&s) {
                seen.push(s);
            }
        }
        assert_eq!(seen.first(), Some(&(6, 4)));
        assert!(seen.contains(&(6, 6)), "the change goes through sub 6: {seen:?}");
        assert_eq!(seen.last(), Some(&(0, 0)), "and ends in the free camera: {seen:?}");
    }
}
