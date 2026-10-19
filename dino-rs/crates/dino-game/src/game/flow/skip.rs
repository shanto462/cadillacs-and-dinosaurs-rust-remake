//! The skip wait task 0x9E38C: holding attack or jump fast forwards a scripted scene.
//!
//! The original task kernel (`0x83C`) runs the tasks in slot order, round after round, and the vertical blank interrupt (`0x542`) lets a sleeping task's
//! counter count down once a frame (`0x5E2`: state 1, counter - 1, at 0 state 4 = ready). The skip wait task is the last slot (0xF0, `0x9E38C`) and yields every
//! pass. While **G.$4CC** (in a game) is set, **G.$4D8 bit 0** is set (the scripted control: the script action 0x28 at 0x1DEA0 sets it, the end of the script
//! 0x1E2BE clears it) and a present hero's input byte (G.$16 / $17 / $18, the cooked pad: `+$00` of the hero not 0) has attack or jump (mask 0x30), it does what the
//! interrupt does to the sleeping tasks in **every round**: every `sleep(n)` of every task ends after about n rounds instead of n vblanks. The game task
//! (`0xA914`, `sleep(G.$4D6 & 15)`) runs a game frame every round, the fade task steps once a round, the system task and the others too. A scripted walk, an
//! area change or an entrance then runs about three times as fast while the button is held.
//!
//! **The CPU time limit.** How many rounds fit in a vblank is not a number in the code: it is the CPU time of one round (a game frame is a third to a half of
//! a vblank, a heavy one with many objects or the area setup of 0x20766 much more). The recordings give 2.2 rounds a vblank for the fade of an area change in
//! `m517_ep1_jack` (the fade-out steps 0xA to 0 in video frames 4232 to 4236), 3.1 for its whole lag record (43 game frames in 14 vblanks, the area setup
//! included), 4 for `ep6_jack` (139 in 35) and 1.7 for `ep7_3p` (15 in 9, a heavier scene). The port uses a **fixed number: [`SKIP_ROUNDS`] = 3** rounds a tick, the
//! middle of the measured 2.2 .. 4: a scene skips about three times as fast, a fade of 16 steps is over in 6 ticks, but the frames are not record exact (the real number
//! changes with the load of the frame).
//!
//! **What a round is.** One run of the flow tasks (system, join, game, intro, tally: [`Game::flow_tasks`]) and of the fade task ([`Game::run_fade_task`]): the same
//! counters as a normal frame, so every sleep that counts per frame counts per round. Not repeated: the interrupt work (the coin interrupt, the input cooking and the
//! `prev` bytes, the display latches of `Game::tick`), which happens once per vblank.
//!
//! **The game clock records.** The recorder of the harness (`record.lua`) writes a record at an interrupt only when no task of the slots 0 to 14 is ready or running
//! (`lagging()`): while the skip keeps the CPU busy the records are cut when the tasks happen to be asleep. The port cannot know that moment, so it joins the ticks of a
//! skip into one record **while the area change is in progress** ([`Game::skip_join_more`]: the fade task runs, or the camera is in the states of the area end 0x20C6A /
//! 0x20CC0 / 0x20CE6), which is what the three recordings show (`m517_ep1_jack` record 2368: the rest of the fade-out, the area load and the fade-in in one
//! record of 14 vblanks). Any other skip is one record a tick.

use super::*;

/// Rounds of the flow tasks and the fade task in a tick while the skip wait task is active, the first one included (see the module doc for the number).
pub const SKIP_ROUNDS: u8 = 3;

impl Game {
    /// 0x9E38C: the condition of the task for this pass: in a game, scripted control (G.$4D8 bit 0) and a present hero holding attack or jump. (The debug
    /// branch with G.$4AA is not ported.)
    pub fn skip_wait_active(&self) -> bool {
        if self.flow.in_game == 0 || self.flow.script.b4d8 & 1 == 0 {
            return false;
        }
        (0..3).any(|i| self.players[i].b.live != 0 && self.w.input[i] & 0x30 != 0)
    }

    /// The extra rounds of a tick (called by `Game::tick` after the first round): while the condition holds, up to `SKIP_ROUNDS - 1` more rounds. A task that took
    /// the CPU for more than a frame (`flow.lag`) ends them: the rounds do not fit.
    pub(in crate::game) fn skip_wait_rounds(&mut self) {
        self.flow.skip_tick = self.skip_wait_active();
        for _ in 1..SKIP_ROUNDS {
            if !self.skip_wait_active() || self.flow.lag > 0 {
                break;
            }
            self.flow_round_start();
            self.flow_tasks();
            self.run_fade_task();
        }
    }

    /// Does the record of the game clock go on with another tick? True after a skip tick while the area change that the skip is fast forwarding is in progress:
    /// the fade task runs or the camera is in one of the area end sub states (0x20C6A, 0x20CC0, 0x20CE6).
    pub fn skip_join_more(&self) -> bool {
        self.flow.skip_tick && (self.fade.task.is_some() || (self.cam.s2.mode == 1 && matches!(self.cam.s2.sub, 4 | 6 | 8)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> Game {
        Game::new(Box::leak(Box::new(dino_assets::AssetPack::default())))
    }

    fn rom() -> Option<&'static dino_assets::AssetPack> {
        static A: std::sync::OnceLock<Option<&'static dino_assets::AssetPack>> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let path = std::path::Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("ROM loads"))))
        })
    }

    #[test]
    fn the_condition_needs_a_game_the_script_flag_and_a_held_button_of_a_present_hero() {
        let mut g = game();
        g.players[0].b.live = 1;
        g.w.input[0] = 0x10;
        assert!(!g.skip_wait_active(), "not in a game");
        g.flow.in_game = 1;
        assert!(!g.skip_wait_active(), "G.$4D8 is 0");
        g.flow.script.b4d8 = 1;
        assert!(g.skip_wait_active());
        g.w.input[0] = 0x20;
        assert!(g.skip_wait_active(), "jump counts too");
        g.w.input[0] = 0x0F;
        assert!(!g.skip_wait_active(), "a direction is not enough");
        g.w.input[0] = 0x10;
        g.players[0].b.live = 0;
        assert!(!g.skip_wait_active(), "the hero is absent");
        g.players[2].b.live = 1;
        g.w.input[2] = 0x10;
        assert!(g.skip_wait_active(), "hero 3");
    }

    #[test]
    fn the_fade_of_a_skipped_scene_runs_three_steps_a_tick() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        g.flow.in_game = 1;
        g.flow.game_pending = true; // the game task has not started: only the fade task has work
        g.flow.script.b4d8 = 1;
        g.players[0].b.live = 1;
        g.w.input[0] = 0x10;
        g.fade.mask = 0xF;
        g.start_fade_task(false);
        g.tick();
        let after_one = g.fade.steps;
        g.tick();
        assert_eq!(after_one, 16 - 3, "three steps in the first tick, like three frames");
        assert_eq!(g.fade.steps, 16 - 6);
        assert!(g.flow.skip_tick);
        g.w.input[0] = 0;
        g.tick();
        assert_eq!(g.fade.steps, 16 - 7, "one step a tick again after the release");
        assert!(!g.flow.skip_tick);
    }

    #[test]
    fn the_game_clock_record_goes_on_while_the_area_change_of_a_skip_is_in_progress() {
        let mut g = game();
        assert!(!g.skip_join_more());
        g.flow.skip_tick = true;
        assert!(!g.skip_join_more(), "no fade, no area change");
        g.cam.s2.mode = 1;
        g.cam.s2.sub = 4;
        assert!(g.skip_join_more(), "the camera waits for the fade at 0x20C6A");
        g.cam.s2.sub = 2;
        assert!(!g.skip_join_more());
        g.fade.task = Some(crate::game::palette::FadeTask { fade_in: false, sleep: 0, started: true });
        assert!(g.skip_join_more(), "the fade task runs");
        g.flow.skip_tick = false;
        assert!(!g.skip_join_more(), "a tick without a skip is one record");
    }
}

