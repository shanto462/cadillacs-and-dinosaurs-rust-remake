//! Keyboard and scripted input, cooked into the original's active-high input bytes:
//! 1 right, 2 left, 4 down, 8 up, 0x10 attack, 0x20 jump (G.$16 / $17 / $18).
//!
//! Keys: player 1 = arrows, Z attack, X jump; player 2 = I J K L, A attack, S jump.
//! Script (`--input`): `start-end:KEYS,...` per frame, KEYS from U D L R A J, for player 1,
//! e.g. `--input 0-40:R,41-45:A,46-50:,51-55:A`. Frames count from the first game tick.

use macroquad::prelude::*;

pub const RIGHT: u8 = 1;
pub const LEFT: u8 = 2;
pub const DOWN: u8 = 4;
pub const UP: u8 = 8;
pub const ATTACK: u8 = 0x10;
pub const JUMP: u8 = 0x20;

pub struct Script {
    segments: Vec<(u32, u32, u8)>,
}

impl Script {
    pub fn parse(text: &str) -> Script {
        let mut segments = Vec::new();
        for part in text.split(',').filter(|p| !p.is_empty()) {
            let (range, keys) = part.split_once(':').unwrap_or((part, ""));
            let (a, b) = range.split_once('-').unwrap_or((range, range));
            let (Ok(a), Ok(b)) = (a.trim().parse(), b.trim().parse()) else { continue };
            let bits = keys.chars().fold(0u8, |acc, c| {
                acc | match c.to_ascii_uppercase() {
                    'R' => RIGHT,
                    'L' => LEFT,
                    'D' => DOWN,
                    'U' => UP,
                    'A' => ATTACK,
                    'J' => JUMP,
                    _ => 0,
                }
            });
            segments.push((a, b, bits));
        }
        Script { segments }
    }

    pub fn at(&self, frame: u32) -> u8 {
        self.segments.iter().filter(|(a, b, _)| frame >= *a && frame <= *b).fold(0, |acc, s| acc | s.2)
    }
}

fn keys(r: KeyCode, l: KeyCode, d: KeyCode, u: KeyCode, a: KeyCode, j: KeyCode) -> u8 {
    let mut v = 0;
    for (k, bit) in [(r, RIGHT), (l, LEFT), (d, DOWN), (u, UP), (a, ATTACK), (j, JUMP)] {
        if is_key_down(k) {
            v |= bit;
        }
    }
    v
}

/// Reads the keyboard for players 1 and 2.
pub fn keyboard() -> [u8; 3] {
    [
        keys(KeyCode::Right, KeyCode::Left, KeyCode::Down, KeyCode::Up, KeyCode::Z, KeyCode::X),
        keys(KeyCode::L, KeyCode::J, KeyCode::K, KeyCode::I, KeyCode::A, KeyCode::S),
        0,
    ]
}
