//! Input: keyboard, gamepads and scripted input, cooked into the original's active-high bytes.
//!
//! * Players (G.$16 / $17 / $18): 1 right, 2 left, 4 down, 8 up, 0x10 attack, 0x20 jump.
//! * System (G.$1A, cooked by the vblank input code 0x738): bit0..2 start 1..3.
//! * Coins (G.$20): bit0..2 coin slot 1..3.
//!
//! Keyboard (MAME-style defaults):
//! * Player 1: arrows, attack Z or Left Ctrl, jump X or Left Alt.
//! * Player 2: R F D G (up down left right), attack A, jump S.
//! * Player 3: I K J L, attack N, jump M.
//! * Start 1/2/3: keys 1/2/3 (Enter also starts player 1). Coin 1/2/3: keys 5/6/7.
//! * F1: hit box overlay (debug).
//!
//! Gamepads: the first three pads drive players 1-3. D-pad or left stick moves, South (A / Cross)
//! attacks, East (B / Circle) jumps, Start = start, Select / Back = coin.
//!
//! Script (`--input`): `start-end:KEYS,...` per frame for player 1, KEYS from U D L R A J,
//! plus S (start 1) and C (coin 1). Example: `--input 0-40:R,41-45:A,46-50:,51-55:A`.

use gilrs::{Axis, Button, Gilrs};
use macroquad::prelude::*;

pub const RIGHT: u8 = 1;
pub const LEFT: u8 = 2;
pub const DOWN: u8 = 4;
pub const UP: u8 = 8;
pub const ATTACK: u8 = 0x10;
pub const JUMP: u8 = 0x20;

/// One frame of cooked input.
#[derive(Clone, Copy, Debug, Default)]
pub struct Frame {
    pub players: [u8; 3],
    pub system: u8,
    pub coins: u8,
}

pub struct Script {
    segments: Vec<(u32, u32, Frame)>,
}

impl Script {
    pub fn parse(text: &str) -> Script {
        let mut segments = Vec::new();
        for part in text.split(',').filter(|p| !p.is_empty()) {
            let (range, keys) = part.split_once(':').unwrap_or((part, ""));
            let (a, b) = range.split_once('-').unwrap_or((range, range));
            let (Ok(a), Ok(b)) = (a.trim().parse(), b.trim().parse()) else { continue };
            let mut f = Frame::default();
            for c in keys.chars() {
                match c.to_ascii_uppercase() {
                    'R' => f.players[0] |= RIGHT,
                    'L' => f.players[0] |= LEFT,
                    'D' => f.players[0] |= DOWN,
                    'U' => f.players[0] |= UP,
                    'A' => f.players[0] |= ATTACK,
                    'J' => f.players[0] |= JUMP,
                    'S' => f.system |= 1,
                    'C' => f.coins |= 1,
                    _ => {}
                }
            }
            segments.push((a, b, f));
        }
        Script { segments }
    }

    pub fn at(&self, frame: u32) -> Frame {
        let mut out = Frame::default();
        for (a, b, f) in &self.segments {
            if frame >= *a && frame <= *b {
                out.players[0] |= f.players[0];
                out.system |= f.system;
                out.coins |= f.coins;
            }
        }
        out
    }
}

pub struct Inputs {
    gilrs: Option<Gilrs>,
}

fn keys(list: &[(KeyCode, u8)]) -> u8 {
    list.iter().filter(|(k, _)| is_key_down(*k)).fold(0, |acc, (_, bit)| acc | bit)
}

impl Inputs {
    pub fn new() -> Inputs {
        // No gamepad support is not an error: keyboard still works.
        Inputs { gilrs: Gilrs::new().ok() }
    }

    /// Reads keyboard and gamepads for this frame.
    pub fn poll(&mut self) -> Frame {
        use KeyCode::*;
        let mut f = Frame {
            players: [
                keys(&[(Right, RIGHT), (Left, LEFT), (Down, DOWN), (Up, UP), (Z, ATTACK), (LeftControl, ATTACK), (X, JUMP), (LeftAlt, JUMP)]),
                keys(&[(G, RIGHT), (D, LEFT), (F, DOWN), (R, UP), (A, ATTACK), (S, JUMP)]),
                keys(&[(L, RIGHT), (J, LEFT), (K, DOWN), (I, UP), (N, ATTACK), (M, JUMP)]),
            ],
            system: keys(&[(Key1, 1), (Enter, 1), (Key2, 2), (Key3, 4)]),
            coins: keys(&[(Key5, 1), (Key6, 2), (Key7, 4)]),
        };
        if let Some(g) = &mut self.gilrs {
            while g.next_event().is_some() {}
            for (slot, (_, pad)) in g.gamepads().take(3).enumerate() {
                let mut p = 0u8;
                let (x, y) = (pad.value(Axis::LeftStickX), pad.value(Axis::LeftStickY));
                if pad.is_pressed(Button::DPadRight) || x > 0.5 {
                    p |= RIGHT;
                }
                if pad.is_pressed(Button::DPadLeft) || x < -0.5 {
                    p |= LEFT;
                }
                if pad.is_pressed(Button::DPadDown) || y < -0.5 {
                    p |= DOWN;
                }
                if pad.is_pressed(Button::DPadUp) || y > 0.5 {
                    p |= UP;
                }
                if pad.is_pressed(Button::South) {
                    p |= ATTACK;
                }
                if pad.is_pressed(Button::East) {
                    p |= JUMP;
                }
                f.players[slot] |= p;
                if pad.is_pressed(Button::Start) {
                    f.system |= 1 << slot;
                }
                if pad.is_pressed(Button::Select) {
                    f.coins |= 1 << slot;
                }
            }
        }
        // The cabinet cannot press left+right or up+down at once; the game never sees it.
        for p in &mut f.players {
            if *p & (LEFT | RIGHT) == LEFT | RIGHT {
                *p &= !(LEFT | RIGHT);
            }
            if *p & (UP | DOWN) == UP | DOWN {
                *p &= !(UP | DOWN);
            }
        }
        f
    }
}
