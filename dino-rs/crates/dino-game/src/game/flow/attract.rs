//! The attract task 0x97C20: its state byte (G.-$7030), the routines of the states and the screens
//! that are not ported yet. Owner: flow / HUD port (M1-04). Spec: `D4d-front-end-and-story.md` s3.1,
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
//! The title is real (`boot.rs`). The profiles (0x9A7E6, M1-05), the ranking (0x1516C through 0x983D8,
//! M1-07), the demo (0x99E88, M1-12), the story slides (0x988C0, M1-08) and the opening movie
//! (0x9BF50, M1-09) are stubs: a blank screen with a label that lasts exactly as long as the real
//! screen did in the MAME recording `attract3.mame.trc`, so the state byte changes at the same
//! frames (video frames 2744, 6874, 9311, 13411, 15848, 19979, 22416, 26516, 28953). The lengths are
//! measured, not computed: the original takes a few frames more or less on some passes (one clear of
//! all layers needs 1.3 frames of CPU time, the demo and the profile cards drop frames), which the
//! port does not model. Each stub says where its numbers come from; the real screens replace them.

use super::boot::st;
use crate::game::Game;
use crate::game::flow::scene::Scene;

/// The stub screen that is up (for the label and the lost system task passes).
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

/// Frames of the stubs, measured in `attract3.mame.trc` (records = video frames - 2): from the
/// frame the original calls the routine to the frame it returns.
/// The profile cards: 1399 the first time, 1407 afterwards (the first run is 8 frames shorter).
const PROFILE_FRAMES: [u16; 2] = [1399, 1407];
/// The ranking: 422 the first time, 423 afterwards.
const RANKING_FRAMES: [u16; 2] = [422, 423];
/// The demo with its last picture and the fade (1200 game frames, 5 frames of setup, 100 frozen,
/// 16 of fade), the story slides and the opening movie (to the end of the routine).
const DEMO_FRAMES: u16 = 1325;
const SLIDES_FRAMES: u16 = 1779;
const MOVIE_FRAMES: u16 = 1745;

/// Frames after the start of the profile stub in which the system task loses its pass: the first
/// frame (the screen is set up with clears) and the three card changes (the counter G.$4F6 of
/// the free play recording `m104_free.mame.trc` misses a count there).
const PROFILE_LOST: [u32; 4] = [1, 326, 687, 1048];

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
        self.sprites.entries.clear();
        self.sprites.debug_rects.clear();
        self.flow_clear_scroll1();
        self.flow.front.clear_stage = 1;
    }

    /// The next part of a clear that is under way (see `flow_attract_clear`), at the start of a frame.
    pub(super) fn flow_attract_clear_step(&mut self) {
        match self.flow.front.clear_stage {
            1 => {
                self.vram.clear2();
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
    /// such waits as loops, here they are chained sleeps) before the script goes on in `next`.
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
                // 0x9A7E6: the profile cards
                let k = self.flow.front.profile_runs as usize;
                self.flow.front.profile_runs = self.flow.front.profile_runs.saturating_add(1);
                self.flow_attract_stub(Stub::Profiles, "CHARACTER PROFILES");
                self.flow_attract_wait(PROFILE_FRAMES[k.min(1)], st::PROFILES_END);
            }
            _ => {
                // 2 and 6: 0x98330 (the World set re-creates the INSERT COIN blink task), the demo
                // counter G.$757A + 1 (0x97D6C / 0x97D78), then the demo 0x99E88
                self.flow_warning_picture();
                self.flow.front.demo_counter = self.flow.front.demo_counter.wrapping_add(1);
                self.flow_attract_stub(Stub::Demo, "DEMO");
                self.flow_attract_wait(DEMO_FRAMES, st::DEMO_END);
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
            // 0x97CD6: kill the blink task (slot 0x20), then the story slides 0x988C0
            self.flow.front.blink.kill();
            self.flow_attract_stub(Stub::Slides, "STORY SLIDES");
            self.flow_attract_wait(SLIDES_FRAMES, st::SLIDES_END);
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

    /// 0x983D8: kill the blink task (slot 0x20), then the ranking 0x1516C (stub).
    pub(super) fn flow_attract_ranking_start(&mut self) {
        self.brightness_full(); // 0x892C (the states that clear first do it here)
        self.flow.front.blink.kill();
        self.sound(0xFF00); // 0x1516C stops the sound first
        let k = self.flow.front.rank_runs as usize;
        self.flow.front.rank_runs = self.flow.front.rank_runs.saturating_add(1);
        self.flow_attract_stub(Stub::Ranking, "RANKING");
        // the ranking set-up is busy for one more frame
        self.flow.front.sys_skip = 1;
        self.flow_attract_wait(RANKING_FRAMES[k.min(1)], st::RANK_END);
    }

    /// The ranking returned: 0x983E8 (scroll1 Y to 0), then the end of the state routine.
    pub(super) fn flow_attract_ranking_end(&mut self) {
        self.flow.front.stub = Stub::None;
        self.sound(0xFF00); // the end of 0x1516C
        self.cam.cam1_y = 0; // 0x983E8: G.$6C8
        if self.flow.front.after_ranking_pass {
            // the game over ranking of 0x97C94: the flag is cleared, the pass goes on with its routine
            self.flow.front.after_ranking_pass = false;
            self.flow.front.game_over_flag = 0; // clr.b -$702E(a5)
            self.flow_attract_state_routine();
            return;
        }
        let s = self.flow.front.attract_state & 0x3E;
        if s == 6 {
            // 0x97D34: the opening movie 0x9BF50, then the state byte is 0
            self.flow_attract_stub(Stub::Movie, "OPENING MOVIE");
            self.flow_attract_wait(MOVIE_FRAMES, st::MOVIE_END);
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

    /// Starts a stub screen: clears the layers and writes the label.
    fn flow_attract_stub(&mut self, stub: Stub, label: &str) {
        self.flow.front.stub = stub;
        self.flow.front.stub_start = self.flow.front.ticks;
        self.flow_clear_screen();
        // centred in row 46 of the scroll1 map (the rows 32 to 63 are the ones the screen shows)
        let text = format!("{label} (NOT PORTED)");
        let col = 24 - text.len().min(46) / 2;
        let at = 0x90A000 + (col as u32) * 0x80 + 14 * 4;
        for (i, c) in text.bytes().enumerate() {
            self.hud_put(at + 0x80 * i as u32, c as u16, 0x180);
        }
    }

    /// True when the stub window loses the pass of the system task at this frame (see
    /// [`PROFILE_LOST`]).
    pub(super) fn flow_attract_pass_lost(&self) -> bool {
        let f = &self.flow.front;
        f.stub == Stub::Profiles && PROFILE_LOST.contains(&f.ticks.wrapping_sub(f.stub_start))
    }
}
