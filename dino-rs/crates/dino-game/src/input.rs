//! Input: keyboard, gamepads and scripted input, cooked into the original's active-high bytes.
//!
//! * Players (G.$16 / $17 / $18): 1 right, 2 left, 4 down, 8 up, 0x10 attack, 0x20 jump.
//! * System (G.$1A, cooked by the vblank input code 0x738): bit0..2 start 1..3, bit 7 SERVICE (adds a credit for every
//!   player, the system task 0x9D326). Bit 6 is the TEST button of the original (it leaves for the test menu): not mapped.
//! * Coins (G.$20): bit0..2 coin slot 1..3.
//!
//! Keyboard (player 1 only; Ctrl and Alt are not used, they clash with system shortcuts):
//! * Move: arrows.
//! * Attack: J or Z. Jump: K or X.
//! * Special move keys (`Moves`): A command move, S dive kick, D jump attack, F special (mega crush), G dash attack. Each key plays the
//!   original's own input sequence for the move (D2a sections 4, 6, 7 and 8), so the game logic sees only the buttons a player would press.
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

/// The special moves that one key can play (player 1, keyboard).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    /// D2a s8, the "sure kill" (0x6AC8 checks it in the walk state): tap down alone, tap up alone within 10 frames, then attack while
    /// holding up within 10 frames.
    Command,
    /// D2a s4.1, the dive: a neutral jump (a forward jump does not reach the split height), then attack and the direction at the top of
    /// the jump. Measured (M: d2a_basic jattack_*): attack first pressed 20 frames after the jump starts the dive for all four heroes.
    Dive,
    /// D2a s4.1, the low jump attack: attack held while the jump is 16 px or higher (M: held from +3 .. +16 gives the low attack).
    JumpAttack,
    /// D2a s6, the special (0x63AA / 0x63E6): attack and jump together.
    Special,
    /// D2a s3 and s7: a run (the second press of the direction at most 12 frames after the first), then attack while running.
    Dash,
}

impl Move {
    /// The player 1 buttons of each frame of the move, from its first frame. `dir` is RIGHT or LEFT (the way to go).
    pub fn frames(self, dir: u8) -> Vec<u8> {
        let mut v = Vec::new();
        let mut add = |n: usize, bits: u8| v.extend(std::iter::repeat_n(bits, n));
        match self {
            Move::Command => {
                add(2, DOWN);
                add(2, UP);
                add(3, UP | ATTACK);
            }
            Move::Dive => {
                add(2, JUMP);
                add(18, 0);
                add(4, dir | ATTACK);
            }
            Move::JumpAttack => {
                add(2, JUMP);
                add(2, 0);
                add(14, ATTACK);
            }
            Move::Special => add(2, ATTACK | JUMP),
            Move::Dash => {
                add(2, dir);
                add(2, 0);
                add(6, dir);
                add(3, dir | ATTACK);
            }
        }
        v
    }
}

/// The keys of the special moves.
const MOVE_KEYS: [(KeyCode, Move); 5] =
    [(KeyCode::A, Move::Command), (KeyCode::S, Move::Dive), (KeyCode::D, Move::JumpAttack), (KeyCode::F, Move::Special), (KeyCode::G, Move::Dash)];

pub struct Inputs {
    gilrs: Option<Gilrs>,
    /// The frames left of a special move key: while it runs they replace player 1's keyboard buttons.
    moves: std::collections::VecDeque<u8>,
    /// The last horizontal direction player 1 held (the way a dive or a dash goes when no arrow is held).
    last_dir: u8,
}

fn keys(list: &[(KeyCode, u8)]) -> u8 {
    list.iter().filter(|(k, _)| is_key_down(*k)).fold(0, |acc, (_, bit)| acc | bit)
}

impl Inputs {
    pub fn new() -> Inputs {
        // No gamepad support is not an error: keyboard still works.
        Inputs { gilrs: Gilrs::new().ok(), moves: Default::default(), last_dir: RIGHT }
    }

    /// Player 1's keyboard buttons of this frame, with the special move keys: a move key starts its sequence (when none runs), and while
    /// a sequence runs it replaces the keyboard buttons of player 1.
    fn player1_keys(&mut self, live: u8) -> u8 {
        if live & (LEFT | RIGHT) == RIGHT || live & (LEFT | RIGHT) == LEFT {
            self.last_dir = live & (LEFT | RIGHT);
        }
        if self.moves.is_empty() {
            if let Some(&(_, m)) = MOVE_KEYS.iter().find(|(k, _)| is_key_pressed(*k)) {
                self.moves.extend(m.frames(self.last_dir));
            }
        }
        self.moves.pop_front().unwrap_or(live)
    }

    /// Reads keyboard and gamepads for this frame.
    pub fn poll(&mut self) -> Frame {
        use KeyCode::*;
        let live = keys(&[(Right, RIGHT), (Left, LEFT), (Down, DOWN), (Up, UP), (J, ATTACK), (Z, ATTACK), (K, JUMP), (X, JUMP)]);
        let mut f = Frame {
            players: [self.player1_keys(live), 0, 0],
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

    /// The move keys keep the windows of D2a: the command move's taps and attack within 10 frames, the dive's first attack at +20,
    /// the jump attack's held attack from +4, the special's two buttons together, the dash's second press within 12 frames.
    #[test]
    fn move_sequences_fit_the_original_windows() {
        let first = |v: &[u8], bit: u8| v.iter().position(|b| b & bit != 0).unwrap();
        let c = Move::Command.frames(RIGHT);
        assert!(first(&c, UP) - first(&c, DOWN) <= 10 && first(&c, ATTACK) - first(&c, UP) <= 10);
        assert!(c.iter().all(|&b| b & (LEFT | RIGHT) == 0), "the taps are alone");
        assert_eq!(c[first(&c, ATTACK)] & UP, UP, "attack while up is held");
        let d = Move::Dive.frames(LEFT);
        assert_eq!((first(&d, JUMP), first(&d, ATTACK), first(&d, LEFT)), (0, 20, 20), "a neutral jump, then attack and the direction at +20");
        let j = Move::JumpAttack.frames(RIGHT);
        assert_eq!(first(&j, ATTACK), 4);
        assert!(j.iter().all(|&b| b & (LEFT | RIGHT) == 0));
        assert_eq!(Move::Special.frames(RIGHT)[0], ATTACK | JUMP);
        let s = Move::Dash.frames(RIGHT);
        let second = s.iter().enumerate().skip(1).find(|&(i, &b)| b & RIGHT != 0 && s[i - 1] & RIGHT == 0).unwrap().0;
        assert!(second <= 12 && first(&s, ATTACK) > second);
    }
}
