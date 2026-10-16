//! The attract task 0x97C20: its state byte (G.-$7030) and the routines of the states, which call the screens.
//! Owner: flow / HUD port (M1-04, M1-13). Spec: `D4d-front-end-and-story.md` s3.1,
//! s3.2, s13 A3.
//!
//! The task is the slot 0x10 scene of `boot.rs` (`front.task`); this file holds its script from the
//! pass 0x97C46 on. Each pass runs the routine of the state byte, which steps by 2 and is **not**
//! reset when a game starts and ends (F25), so the loop goes on where it was:
//!
//! | state | routine | parts in order |
//! | --- | --- | --- |
//! | 0, 4 | 0x97D84 | stop sound, palettes, clear, title, profiles, (0x98330), clear, ranking |
//! | 2 | 0x97CC0 | stop sound, title, 0x98330, demo counter + 1, demo, slides, stop sound, clear, ranking |
//! | 6 | 0x97D02 | stop sound, palettes, clear, title, 0x98330, demo counter + 1, demo, clear, ranking, movie, state = 0 |
//! | 8 | 0x97D42 | ranking only (not reachable) |
//! | 0xA | 0x97DBE | state = 0 (not reachable) |
//!
//! Every screen of the loop is real: the title (`boot.rs`), the profile cards (0x9A7E6, `profiles.rs`, M1-05), the ranking
//! (0x1516C through 0x983D8, `ranking.rs`, M1-07), the story slides (0x988C0, `slides.rs`, M1-08), the opening movie (0x9BF50,
//! `movie.rs`, M1-09 to M1-11) and the demo (0x99E88, `demo.rs`, M1-12 and M1-13). The state byte changes at the frames of the
//! MAME recording `flow_m113-long.mame.trc` (video frames 2744, 6874, 9311, 13411, 15848, 19979, 22416, 26516, 28953; the
//! demo takes the byte out of RAM for 3 frames in between and puts it back). The lengths of the clears 0xAE0 are measured, not
//! computed: the original needs 1.3 frames of CPU time for one, which the port writes as waits (`flow_attract_busy`).

use super::boot::st;
use crate::game::Game;
use crate::game::flow::scene::Scene;

/// The screen that is up and not a plain title (the profile cards, the ranking, the demo, the slides, the movie).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Stub {
    #[default]
    None,
    Profiles,
    Ranking,
    Demo,
    Slides,
    Movie,
}

/// The cells of scroll3 the clear 0xAE0 has done when the frame it clears scroll2 in ends (the
/// original clears the maps one after the other, 4096 cells at about 32 cycles each). Any value from
/// 1072 to 3576 gives what the recording shows (56 of the 216 board cells left, see
/// `flow_attract_clear_step`); the middle of the range that the cycle count allows is used.
const CLEAR3_FIRST: usize = 3000;

impl Game {
    /// A stretch of CPU time of the original that the port writes as a sleep: `frames` until the
    /// script goes on in `next`. The attract task does not yield during it, so the system task (slot
    /// 0, which runs before it) misses its pass on the first `lost` of these frames.
    pub(super) fn flow_attract_busy(&mut self, frames: u16, lost: u8, next: u16) {
        self.flow.front.task.wait(frames, next);
        self.flow.front.sys_skip = lost;
    }

    /// 0xAE0 in the attract task: the original needs 1.3 frames of CPU time for it, so the recording
    /// shows the sprite buffers and scroll1 blank after the frame it starts in, scroll2 one frame
    /// later and scroll3 one more frame later. The script waits 2 frames after it
    /// (`flow_attract_busy`); the rest is done by `flow_attract_clear_step` at the start of each frame.
    pub(super) fn flow_attract_clear(&mut self) {
        let c = &mut self.cam;
        (c.cam1_x, c.cam1_y, c.s2.cam_x, c.s2.cam_y, c.s3.cam_x, c.s3.cam_y) = (0, 0, 0, 0, 0, 0);
        self.sprites.wipe();
        self.flow_clear_scroll1();
        self.flow.front.clear_stage = 1;
    }

    /// The next part of a clear that is under way (see `flow_attract_clear`), at the start of a frame.
    /// Scroll3 is cleared in two frames: when scroll2 is done the clear of scroll3 has not got far
    /// (`CLEAR3_FIRST` cells). The recording shows it when scroll3 holds the checker board of a
    /// profile card: the last 56 cells it holds, those of the fourth block, are still there.
    pub(super) fn flow_attract_clear_step(&mut self) {
        match self.flow.front.clear_stage {
            1 => {
                self.vram.clear2();
                let cut = std::mem::take(&mut self.flow.front.clear3_cut);
                for c in self.vram.scroll3.iter_mut().take(if cut != 0 { cut } else { CLEAR3_FIRST }) {
                    (c.code, c.attr) = (0x1C00, 0);
                }
                self.flow.front.clear_stage = 2;
            }
            2 => {
                self.vram.clear3();
                self.flow.front.clear_stage = 0;
            }
            _ => {}
        }
    }

    /// A wait of `frames` frames (more than the 255 the kernel sleep byte holds: the original writes
    /// such waits as loops, here they are chained sleeps) before the script goes on in `next`. (The demo stub
    /// of M1-04 used it; the screens of the loop are all real now, it is kept for the next long wait.)
    #[allow(dead_code)]
    fn flow_attract_wait(&mut self, frames: u16, next: u16) {
        let t = &mut self.flow.front.task;
        if frames <= 255 {
            t.wait(frames, next);
        } else {
            t.count = frames - 255;
            t.sub = next;
            t.wait(255, st::LONG_WAIT);
        }
    }

    /// The chained sleep of [`Game::flow_attract_wait`]: the rest of the frames, then `next`.
    pub(super) fn flow_attract_long_wait(&mut self) {
        let t = &mut self.flow.front.task;
        let rest = t.count;
        if rest > 255 {
            t.count = rest - 255;
            t.sleep(255);
        } else {
            let next = t.sub;
            t.wait(rest, next);
        }
    }

    /// 0x97C46: the pass of the attract task: layer control 0x18D6, the game over ranking (0x97C94),
    /// then the routine of the state byte (table 0x97C88).
    pub(super) fn flow_attract_pass(&mut self) {
        self.w.layer_ctrl = 0x18D6;
        // 0x97C94: after a game over the ranking is shown once before anything else (0x983D8,
        // the ranking itself is a stub, M1-07), then the flag is cleared.
        if self.flow.front.game_over_flag != 0 {
            self.flow_scene_palettes(0);
            self.sound(0xFF00);
            self.flow_attract_clear();
            self.flow_attract_busy(2, 2, st::GAME_OVER_RANKING);
            return;
        }
        self.flow_attract_state_routine();
    }

    /// The routine of the state byte: 0x97D84 (0, 4), 0x97CC0 (2), 0x97D02 (6), 0x97D42 (8), 0x97DBE (0xA).
    fn flow_attract_state_routine(&mut self) {
        match self.flow.front.attract_state & 0x3E {
            0 | 4 | 6 => {
                // 0x97D84 / 0x97D02: stop sound, palette set 0, clear all layers, then the title
                // (brightness first, `st::TITLE_START`)
                self.sound(0xFF00);
                self.flow_scene_palettes(0);
                self.flow_attract_clear();
                self.flow_attract_busy(2, 2, st::TITLE_START);
            }
            2 => {
                // 0x97CC0: stop sound, the title at once
                self.sound(0xFF00);
                self.title_reset();
                self.flow_attract_busy(2, 1, st::TITLE_SETUP);
            }
            8 => {
                // 0x97D42: stop sound, clear, brightness, the ranking; the state steps on
                self.sound(0xFF00);
                self.flow_attract_clear();
                self.flow_attract_busy(2, 2, st::RANK_START);
            }
            _ => {
                // 0xA (0x97DBE) and anything else the table does not have: state = 0, next pass
                self.flow.front.attract_state = 0;
                self.flow.front.task.state = st::PASS;
            }
        }
    }

    /// The game over ranking of 0x97C94 after its clear: brightness, the ranking stub; when it
    /// returns the flag is cleared and the pass goes on with its state routine.
    pub(super) fn flow_attract_game_over_ranking(&mut self) {
        self.brightness_full(); // 0x892C
        self.flow.front.after_ranking_pass = true;
        self.flow_attract_ranking_start();
    }

    /// The title returned (counter 600): what the routine of the state does next.
    pub(super) fn flow_attract_after_title(&mut self) {
        match self.flow.front.attract_state & 0x3E {
            0 | 4 => {
                // 0x9A7E6: the profile cards (`profiles.rs`); it returns in `st::PROFILES_END`
                self.flow.front.profile_runs = self.flow.front.profile_runs.saturating_add(1);
                self.flow_profile_start();
            }
            _ => {
                // 2 and 6: 0x98330 (the World set re-creates the INSERT COIN blink task), the demo counter G.$757A + 1
                // (0x97D6C / 0x97D78), then the demo 0x99E88 (`demo.rs`); it returns in `st::DEMO_END`
                self.flow_warning_picture();
                self.flow.front.demo_counter = self.flow.front.demo_counter.wrapping_add(1);
                self.flow_demo_start();
            }
        }
    }

    /// 0x98330 for the World set (G.$32 = 2): only the blink task is created again and the cameras
    /// are set to 0 (0x983BC); the warning picture of the other regions is not ported.
    fn flow_warning_picture(&mut self) {
        let mut blink = Scene::start(0);
        blink.sleep(2);
        self.flow.front.blink = blink;
        self.cam.s3.cam_x = 0;
        self.cam.s2.cam_x = 0;
    }

    /// The profile cards returned (states 0 and 4): 0x98330 (its blink task is killed again by
    /// 0x983D8 before it ever runs), a clear of all layers, then the ranking.
    pub(super) fn flow_attract_profiles_end(&mut self) {
        self.flow.front.stub = Stub::None;
        self.flow_attract_clear();
        self.flow_attract_busy(2, 2, st::RANK_START);
    }

    /// The demo returned (states 2 and 6).
    pub(super) fn flow_attract_demo_end(&mut self) {
        self.flow.front.stub = Stub::None;
        if self.flow.front.attract_state & 0x3E == 2 {
            // 0x97CD6: kill the blink task (slot 0x20), then the story slides 0x988C0 (`slides.rs`);
            // they return in `st::SLIDES_END`
            self.flow.front.blink.kill();
            self.flow_slides_start();
        } else {
            // 0x97D2A: clear all layers, the ranking
            self.flow_attract_clear();
            self.flow_attract_busy(2, 2, st::RANK_START);
        }
    }

    /// The slides returned (state 2): stop sound, clear, brightness, then the ranking.
    pub(super) fn flow_attract_slides_end(&mut self) {
        self.flow.front.stub = Stub::None;
        self.sound(0xFF00);
        self.flow_attract_clear();
        self.flow_attract_busy(2, 2, st::RANK_START);
    }

    /// 0x983D8: kill the blink task (slot 0x20), then the ranking 0x1516C (`ranking.rs`), which returns in
    /// `st::RANK_END`.
    pub(super) fn flow_attract_ranking_start(&mut self) {
        self.flow.front.blink.kill();
        self.flow.front.rank_runs = self.flow.front.rank_runs.saturating_add(1);
        // the ranking set-up is busy for one more frame (the system task loses a pass)
        self.flow.front.sys_skip = 1;
        self.flow_ranking_start();
    }

    /// The ranking returned: 0x983E8 (scroll1 Y to 0), then the end of the state routine.
    pub(super) fn flow_attract_ranking_end(&mut self) {
        self.flow.front.stub = Stub::None;
        self.sound(0xFF00); // the end of 0x1516C
        self.cam.cam1_y = 0; // 0x983E8: G.$6C8
        if self.flow.front.after_ranking_pass {
            // the game over ranking of 0x97C94: the flag is cleared, the pass goes on with its routine
            self.flow.front.after_ranking_pass = false;
            self.flow.ending.ranking_follows = false;
            self.flow.front.game_over_flag = 0; // clr.b -$702E(a5)
            self.flow_attract_state_routine();
            return;
        }
        let s = self.flow.front.attract_state & 0x3E;
        if s == 6 {
            // 0x97D34: the opening movie 0x9BF50 (`movie.rs`, it returns in `st::MOVIE_END`), then the state byte is 0
            self.flow_movie_start();
        } else {
            // 0x97D00 / 0x97CFC / 0x97DB8: the state steps by 2
            self.flow.front.attract_state = self.flow.front.attract_state.wrapping_add(2);
            self.flow.front.task.state = st::PASS;
        }
    }

    /// The opening movie returned: 0x97D3A: state = 0, next pass.
    pub(super) fn flow_attract_movie_end(&mut self) {
        self.flow.front.stub = Stub::None;
        self.flow.front.attract_state = 0;
        self.flow.front.task.state = st::PASS;
    }
}
