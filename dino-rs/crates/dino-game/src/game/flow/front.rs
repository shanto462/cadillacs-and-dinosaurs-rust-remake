//! Front end state: the character select (game mode 4, 0xAD46) and the variables of the front end tasks (the attract
//! task, the blink and press start tasks, the demo). The system task 0x9D048 with the coins, credits and the credit text
//! is in `system.rs` (M2-02). Owner: flow / HUD port. Spec: `D4d-front-end-and-story.md` s3.10, s4, s5.

use crate::game::Game;
use crate::game::flow::attract::Stub;
use crate::game::flow::demo::Demo;
use crate::game::flow::scene::Scene;
use crate::game::flow::system::Sys;

/// Character select struct G.$62C (cleared by 0xAD36).
#[derive(Clone, Copy, Debug, Default)]
pub struct Select {
    /// +3 taken cursor bits, +9 seconds left, +$A frame counter.
    pub taken: u8,
    pub seconds: u8,
    pub frames: u8,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Front {
    pub select: Select,
    /// G.-$702E set by the game over (0xAC20).
    pub game_over_flag: u8,
    /// G.$1A of the previous frame (G.$1F), for edge detection.
    pub prev_system: u8,
    /// G.$16..18 of the previous frame (G.$1B..1D), kept by `flow_frame`.
    pub prev_input: [u8; 3],
    /// Slot 0x10: the boot task 0xA69A, then the attract task 0x97C20 (`boot.rs`, `attract.rs`).
    pub task: Scene,
    /// Slot 0x20: the INSERT COIN blink task 0x98874 or the press start screen task 0x978EE.
    pub blink: Scene,
    pub press: Scene,
    /// G.-$7022 the title frame counter (0 to 0x258).
    pub title: u16,
    /// G.-$7030 the attract state byte (0, 2, 4, 6).
    pub attract_state: u8,
    /// G.$757A the demo counter (the demo row, M1-12).
    pub demo_counter: u16,
    /// The demo's own variables: G.$757C, the input playback, the pad bytes (`demo.rs`).
    pub demo: Demo,
    /// Slot 0: the system task 0x9D048.
    pub sys_task: Scene,
    pub sys: Sys,
    /// Frames counted since power on (the original has no such counter; the stubs need it).
    pub ticks: u32,
    /// Passes of the system task to skip (the attract task is busy and the scan of the task slots
    /// does not come round, see `Game::flow_attract_busy`).
    pub sys_skip: u8,
    /// The screen that is up: the profile cards or a stub.
    pub stub: Stub,
    /// How many times the title, the profile cards and the ranking ran (the measured lengths of
    /// some frames differ by pass).
    pub title_runs: u16,
    pub profile_runs: u16,
    pub rank_runs: u16,
    /// What is different in this run of the title (`boot.rs`, `TITLE_RUNS`): the frames from the setup to
    /// its first pass, the cells of the blocks drawn in the setup frame, the blink task's first frame.
    pub title_run: (u8, u16, u8),
    /// The part of the clear 0xAE0 still to do in the next frames of the attract task (0 none, 1
    /// scroll2 and scroll3, 2 scroll3).
    pub clear_stage: u8,
    /// The cells of scroll3 the clear has done when the frame it clears scroll2 in ends (0: the usual `CLEAR3_FIRST`).
    pub clear3_cut: usize,
    /// The game over ranking of 0x97C94 is up: the pass goes on after it.
    pub after_ranking_pass: bool,
    /// The press start call 0x983EE is under way (it takes the system task 3 frames): 2 = the next frame clears scroll2 (and the
    /// first part of scroll3), 1 = the next frame clears the rest of scroll3 and does the rest of the call (`flow_press_start_rest`).
    pub press_call: u8,
    /// The words of scroll3's palette bank the brightness loop has done when the frame that ends the press start call ends.
    pub press_cut: usize,
}

impl Game {
    /// Game mode 4 (0xAD46) character select. TODO(M2-03): screen, cursors, timer (0xAF7C, 0xB06A,
    /// 0xB122, 0xB25E). For now the players that pressed start get the heroes in order (Jack,
    /// Mustapha, Hannah) and the stage starts at once.
    pub fn flow_select_mode(&mut self) {
        for i in 0..3 {
            if self.flow.started >> i & 1 != 0 {
                self.player_join(i, i as u8);
            }
        }
        self.flow.chosen = self.flow.started;
        self.flow_stage_start();
    }
}
