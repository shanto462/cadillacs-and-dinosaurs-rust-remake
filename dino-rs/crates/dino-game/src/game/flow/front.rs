//! Front end: system task 0x9D048 (coins, credits, start), attract / title 0x97C20, press start
//! screen 0x983EE and the character select (game mode 4, 0xAD46). Owner: flow / HUD port.
//! Spec: `re/specs/flow/SPEC.md` section 6 and `HANDOFF.md`. Not ported yet (stubs).

use crate::game::Game;
use crate::game::flow::scene::Scene;

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
    /// Slot 0x10: the boot task 0xA69A, then the attract task 0x97C20 (`boot.rs`).
    pub task: Scene,
    /// Slot 0x20: the INSERT COIN blink task 0x98874 or the press start screen task 0x978EE.
    pub blink: Scene,
    pub press: Scene,
    /// G.-$7022 the title frame counter (0 to 0x258).
    pub title: u16,
    /// G.-$7030 the attract state byte (0, 2, 4, 6).
    pub attract_state: u8,
}

impl Game {
    /// Slot 0 system task 0x9D048. TODO(port): coin slots 0x9D4BE, credits 0x9D386, start
    /// 0x9D196 (edge on G.$1A bits 0-2), press-start screen (`flow_press_start_screen`).
    /// (`flow_frame` keeps the previous inputs G.$1F and G.$1B..1D.)
    pub fn flow_system_task(&mut self) {}

    /// Game mode 4 (0xAD46) character select. TODO(port): screen, cursors, timer (0xAF7C,
    /// 0xB06A, 0xB122, 0xB25E). For now the players keep their characters and the stage starts.
    pub fn flow_select_mode(&mut self) {
        self.flow_stage_start();
    }
}
