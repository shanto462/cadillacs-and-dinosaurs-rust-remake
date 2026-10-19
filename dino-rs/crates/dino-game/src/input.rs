//! Input: keyboard, gamepads and scripted input, cooked into the original's active-high bytes.
//!
//! * Players (G.$16 / $17 / $18): 1 right, 2 left, 4 down, 8 up, 0x10 attack, 0x20 jump.
//! * System (G.$1A, cooked by the vblank input code 0x738): bit0..2 start 1..3, bit 7 SERVICE (adds a credit for every
//!   player, the system task 0x9D326). Bit 6 is the TEST button of the original (it leaves for the test menu): not mapped.
//! * Coins (G.$20): bit0..2 coin slot 1..3.
//!
//! Keyboard (player 1 only; Ctrl and Alt are not used, they clash with system shortcuts):
//! * Move: arrows or W A S D.
//! * Attack: J or Z. Jump: K or X.
//! * Start 1/2/3: keys 1/2/3 (Enter also starts player 1). Coin 1/2/3: keys 5/6/7. SERVICE: key 9 (MAME's key for it too).
//! * F1: hit box overlay. With the debug keys on (`--debug-keys`, `--test`) F1 opens the test menu instead (`menu.rs`).
//!
//! Players 2 and 3 use gamepads. Gamepads: the first three pads drive players 1-3. D-pad or left stick moves, South (A / Cross)
//! attacks, East (B / Circle) jumps, Start = start, Select / Back = coin.
//!
//! Script (`--input`): `start-end:KEYS,...` per frame, KEYS from U D L R A J, plus S (start)
//! and C (coin). Keys go to player 1; the digits 1, 2, 3 select the player for the keys that
//! follow (`0-9:R2L3AS` = P1 right, P2 left, P3 attack and start 3); V is SERVICE (port only: MAME's `re/harness` scripts have
//! no such key). Example:
//! `--input 0-40:R,41-45:A,46-50:,51-55:A`. `--input-file PATH` reads the same format from a
//! file, where new lines also separate segments and `#` starts a comment (the harness scripts
//! in `re/harness/scripts/` use it; MAME's `re/harness/record.lua` reads the same format).

use gilrs::{Axis, Button, Gilrs};
use macroquad::prelude::*;

pub const RIGHT: u8 = 1;
pub const LEFT: u8 = 2;
pub const DOWN: u8 = 4;
pub const UP: u8 = 8;
pub const ATTACK: u8 = 0x10;
pub const JUMP: u8 = 0x20;
/// G.$1A bit 7: the SERVICE button.
pub const SERVICE: u8 = 0x80;
/// G.$18 bit 6: the coin slot 3 (the board has no other port for it; the coins G.$20 get it too). The recordings of the 3 player scripts show it
/// in `input_p3` (re/harness/scripts/ep2_3p.txt, `3...C`).
pub const COIN3: u8 = 0x40;
/// G.$18 bit 7: the start button 3 (also bit 2 of G.$1A).
pub const START3: u8 = 0x80;

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
        let text: String = text.lines().map(|l| l.split('#').next().unwrap_or("")).collect::<Vec<_>>().join("\n");
        for part in text.split([',', '\n']).map(str::trim).filter(|p| !p.is_empty()) {
            let (range, keys) = part.split_once(':').unwrap_or((part, ""));
            let (a, b) = range.split_once('-').unwrap_or((range, range));
            let (Ok(a), Ok(b)) = (a.trim().parse(), b.trim().parse()) else { continue };
            let mut f = Frame::default();
            let mut p = 0;
            for c in keys.chars() {
                match c.to_ascii_uppercase() {
                    '1' => p = 0,
                    '2' => p = 1,
                    '3' => p = 2,
                    'R' => f.players[p] |= RIGHT,
                    'L' => f.players[p] |= LEFT,
                    'D' => f.players[p] |= DOWN,
                    'U' => f.players[p] |= UP,
                    'A' => f.players[p] |= ATTACK,
                    'J' => f.players[p] |= JUMP,
                    // Player 3's start and coin are bits 7 and 6 of the P3 port (IN2), so the game also sees them in G.$18 (MAME ep3_3p frame 400).
                    'S' => {
                        f.system |= 1 << p;
                        if p == 2 {
                            f.players[2] |= 0x80;
                        }
                    }
                    'C' => {
                        f.coins |= 1 << p;
                        if p == 2 {
                            f.players[2] |= 0x40;
                        }
                    }
                    'V' => f.system |= SERVICE,
                    _ => {}
                }
            }
            segments.push((a, b, f));
        }
        Script { segments }
    }

    pub fn load(path: &str) -> Result<Script, String> {
        std::fs::read_to_string(path).map(|t| Script::parse(&t)).map_err(|e| format!("{path}: {e}"))
    }

    pub fn at(&self, frame: u32) -> Frame {
        let mut out = Frame::default();
        for (a, b, f) in &self.segments {
            if frame >= *a && frame <= *b {
                for p in 0..3 {
                    out.players[p] |= f.players[p];
                }
                out.system |= f.system;
                out.coins |= f.coins;
            }
        }
        if out.coins & 4 != 0 {
            out.players[2] |= COIN3;
        }
        if out.system & 4 != 0 {
            out.players[2] |= START3;
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
                keys(&[
                    (Right, RIGHT),
                    (Left, LEFT),
                    (Down, DOWN),
                    (Up, UP),
                    (D, RIGHT),
                    (A, LEFT),
                    (S, DOWN),
                    (W, UP),
                    (J, ATTACK),
                    (Z, ATTACK),
                    (K, JUMP),
                    (X, JUMP),
                ]),
                0,
                0,
            ],
            system: keys(&[(Key1, 1), (Enter, 1), (Key2, 2), (Key3, 4), (Key9, SERVICE)]),
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
                if pad.is_pressed(Button::Mode) {
                    f.system |= SERVICE;
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
        if f.coins & 4 != 0 {
            f.players[2] |= COIN3;
        }
        if f.system & 4 != 0 {
            f.players[2] |= START3;
        }
        f
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_players_and_comments() {
        let s = Script::parse("0-9:R2L3AS # comment\n10:C,\n 12-13:J,\n20-21:V");
        let f = s.at(5);
        // player 3's start is also bit 7 of its own byte (IN2)
        assert_eq!(f.players, [RIGHT, LEFT, ATTACK | 0x80]);
        assert_eq!(f.system, 4);
        assert_eq!(Script::parse("0:3C").at(0).players[2], COIN3);
        assert_eq!(s.at(10).coins, 1);
        let c3 = Script::parse("0:3C").at(0);
        assert_eq!((c3.coins, c3.players[2]), (4, 0x40));
        assert_eq!(s.at(20).system, SERVICE);
        assert_eq!(s.at(22).system, 0);
        assert_eq!(s.at(12).players[0], JUMP);
        assert_eq!(s.at(11).players, [0, 0, 0]);
    }
}
