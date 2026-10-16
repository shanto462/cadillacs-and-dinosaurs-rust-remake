//! `--autoplay` (M3-05): a bot for review runs from power on. It puts in a coin, presses start and attacks to pick Jack on the
//! select screen, then holds right and attacks in pulses; kill all runs every `KILL_EVERY` frames so the waves end. While the
//! game is not running (attract, game over) the coin, start and pick presses repeat every `CYCLE` frames.
//! It prints one line per change of the game mode to stderr: the frame, the episode, the area and the mode.

use crate::debug::Debug;
use crate::game::Game;
use crate::input::{Frame, ATTACK, JUMP, RIGHT};

/// Frames between kill-all presses in a game.
const KILL_EVERY: u32 = 600;
/// Frames of the attract cycle (coin at 10, start at 100, attack at 200).
const CYCLE: u32 = 300;

#[derive(Default)]
pub struct Bot {
    /// The last (episode, area, mode) reported.
    last: Option<(u8, u8, u8)>,
}

impl Bot {
    /// The input of this tick. Kill all goes through `debug`.
    pub fn frame(&mut self, g: &Game, tick: u32, debug: &mut Debug) -> Frame {
        let mut f = Frame::default();
        if g.flow.in_game == 0 {
            let t = tick % CYCLE;
            match t {
                10..=13 => f.coins = 1,
                100..=103 => f.system = 1,
                200..=203 => f.players[0] = ATTACK,
                _ => {}
            }
            return f;
        }
        // Jump pulses get Jack over the walls and the pits of the stages.
        f.players[0] = RIGHT | if tick % 12 < 4 { ATTACK } else { 0 } | if tick % 180 < 8 { JUMP } else { 0 };
        if tick % KILL_EVERY == 0 {
            debug.kill_all();
        }
        f
    }

    /// After a tick: one line when the game mode, the episode or the area changes.
    pub fn report(&mut self, g: &Game, tick: u32) {
        let now = (g.w.stage + 1, g.cam.s2.area, g.flow.mode);
        if self.last != Some(now) {
            self.last = Some(now);
            eprintln!("autoplay: frame {tick} episode {} area {} mode {:#04x} sub {}", now.0, now.1, now.2, g.flow.sub);
        }
    }
}
