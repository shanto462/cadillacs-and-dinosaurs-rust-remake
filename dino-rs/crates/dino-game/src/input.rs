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
//! * Q: the combo toggle (`Combo`): while it is on, the press that would be the third hit of a punch chain (two hits landed on an enemy)
//!   plays the command move (the A key).
//! * P: pause and continue (main.rs).
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

/// What the combo toggle reads of player 1's hero each frame (`main.rs` fills it from the game, before the frame's tick).
#[derive(Clone, Copy, Default)]
pub struct Hero {
    /// Two hits of a punch chain landed on an enemy and the chain is still open: the next attack press would be the chain's third hit.
    pub third_next: bool,
    /// The hero can start the command move now (standing or walking, free to walk, no press buffered).
    pub ready: bool,
}

/// The combo toggle (Q, player 1, keyboard). While it is on, the attack press that would be the third hit of a punch chain (two hits of
/// the chain landed on an enemy, `Hero::third_next`) plays the command move (the A key) in place of that hit. A chain that misses is reset
/// by the game itself (its hit count goes back to 0), so presses in the air or a miss never count. The command move starts only in the walk
/// state (0x6AC8), so it waits until the hero is ready again (a few frames after the hit), and the press itself is not passed on: attack
/// stays off until the key is let go. A wait ends when the chain is no longer at its third hit (the hero is hurt, grabbed, jumps or picks
/// up a weapon, the chain runs out, the enemy dies), when Q turns it off, when a special move key takes over, or after `WAIT` frames.
#[derive(Default)]
pub struct Combo {
    pub on: bool,
    /// Frames left to wait for the hero to be ready (0: no command move waits).
    waiting: u32,
    /// Attack is held since a press the toggle took: it is not passed on until the key is let go.
    muted: bool,
    attack_before: bool,
}

impl Combo {
    /// How long a waiting command move may wait for the hero (then it is dropped).
    const WAIT: u32 = 60;

    /// Switches the toggle (a waiting command move is dropped).
    pub fn toggle(&mut self) {
        self.on = !self.on;
        self.waiting = 0;
    }

    /// A frame the toggle does not decide (a special move sequence plays): only the attack key is followed, and a waiting move is dropped.
    pub fn skip(&mut self, live: u8) {
        self.attack_before = live & ATTACK != 0;
        self.waiting = 0;
    }

    /// One frame: `live` is player 1's keyboard buttons. Returns the buttons to pass on, and whether the command move starts this frame.
    pub fn step(&mut self, live: u8, hero: Hero) -> (u8, bool) {
        let attack = live & ATTACK != 0;
        let press = attack && !self.attack_before;
        self.attack_before = attack;
        if !attack {
            self.muted = false;
        }
        let mut start = false;
        if self.waiting > 0 {
            if press {
                self.muted = true;
            }
            if !self.on || !hero.third_next {
                self.waiting = 0;
            } else if hero.ready {
                self.waiting = 0;
                start = true;
            } else {
                self.waiting -= 1;
            }
        } else if self.on && press && hero.third_next {
            self.muted = true;
            if hero.ready {
                start = true;
            } else {
                self.waiting = Self::WAIT;
            }
        }
        (if self.muted || self.waiting > 0 { live & !ATTACK } else { live }, start)
    }
}

/// Player 1's keyboard buttons with the special move keys and the combo toggle (no key reading here, so tests drive it frame by frame).
pub struct Player1Keys {
    /// The frames left of a special move key: while it runs they replace player 1's keyboard buttons.
    moves: std::collections::VecDeque<u8>,
    /// The last horizontal direction player 1 held (the way a dive or a dash goes when no arrow is held).
    last_dir: u8,
    /// The Q toggle.
    pub combo: Combo,
}

impl Default for Player1Keys {
    fn default() -> Self {
        Player1Keys { moves: Default::default(), last_dir: RIGHT, combo: Combo::default() }
    }
}

impl Player1Keys {
    /// One frame: `live` is the held keys, `q` says Q was pressed, `key` is the special move key pressed (if any), `hero` what the combo
    /// toggle reads of the hero. A move key starts its sequence (when none runs), and while a sequence runs it replaces the keyboard buttons
    /// of player 1.
    pub fn frame(&mut self, live: u8, q: bool, key: Option<Move>, hero: Hero) -> u8 {
        if live & (LEFT | RIGHT) == RIGHT || live & (LEFT | RIGHT) == LEFT {
            self.last_dir = live & (LEFT | RIGHT);
        }
        if q {
            self.combo.toggle();
        }
        if self.moves.is_empty() {
            if let Some(m) = key {
                self.combo.skip(live);
                self.moves.extend(m.frames(self.last_dir));
            } else {
                let (passed, start) = self.combo.step(live, hero);
                if !start {
                    return passed;
                }
                self.moves.extend(Move::Command.frames(self.last_dir));
            }
        } else {
            self.combo.skip(live);
        }
        self.moves.pop_front().unwrap_or(live)
    }
}

pub struct Inputs {
    gilrs: Option<Gilrs>,
    pub p1: Player1Keys,
}

/// Enter starts player 1 (bit 0 of the system byte), except while Alt is held: Alt+Enter is the fullscreen toggle (main.rs).
fn enter_start_bit(enter_down: bool, alt_down: bool) -> u8 {
    (enter_down && !alt_down) as u8
}

fn keys(list: &[(KeyCode, u8)]) -> u8 {
    list.iter().filter(|(k, _)| is_key_down(*k)).fold(0, |acc, (_, bit)| acc | bit)
}

impl Inputs {
    pub fn new() -> Inputs {
        // No gamepad support is not an error: keyboard still works.
        Inputs { gilrs: Gilrs::new().ok(), p1: Player1Keys::default() }
    }

    /// Reads keyboard and gamepads for this frame. `p1`: what the combo toggle reads of player 1's hero (`Hero`).
    pub fn poll(&mut self, p1: Hero) -> Frame {
        use KeyCode::*;
        let live = keys(&[(Right, RIGHT), (Left, LEFT), (Down, DOWN), (Up, UP), (J, ATTACK), (Z, ATTACK), (K, JUMP), (X, JUMP)]);
        let mut f = Frame {
            players: [self.p1.frame(live, is_key_pressed(Q), MOVE_KEYS.iter().find(|(k, _)| is_key_pressed(*k)).map(|&(_, m)| m), p1), 0, 0],
            system: keys(&[(Key1, 1), (Key2, 2), (Key3, 4), (Key9, SERVICE)])
                | enter_start_bit(is_key_down(Enter), is_key_down(LeftAlt) || is_key_down(RightAlt)),
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

#[cfg(test)]
mod enter_tests {
    use super::enter_start_bit;

    #[test]
    fn enter_starts_player_one_only_without_alt() {
        assert_eq!(enter_start_bit(true, false), 1);
        assert_eq!(enter_start_bit(true, true), 0, "Alt+Enter toggles fullscreen, it does not start");
        assert_eq!(enter_start_bit(false, false), 0);
        assert_eq!(enter_start_bit(false, true), 0);
    }
}

#[cfg(test)]
mod combo_tests {
    use super::*;
    use crate::game::Game;
    use dino_assets::AssetPack;

    fn rom() -> Option<&'static AssetPack> {
        static A: std::sync::OnceLock<Option<&'static AssetPack>> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let path = std::path::Path::new("../../dino.zip");
            let path = if path.exists() { path } else { std::path::Path::new("../../../dino.zip") };
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    const THIRD: Hero = Hero { third_next: true, ready: true };
    const BUSY: Hero = Hero { third_next: true, ready: false };
    const PLAIN: Hero = Hero { third_next: false, ready: true };

    fn on() -> Combo {
        let mut c = Combo::default();
        c.toggle();
        c
    }

    #[test]
    fn the_combo_is_off_by_default_and_passes_every_press() {
        let mut c = Combo::default();
        assert_eq!(c.step(ATTACK, THIRD), (ATTACK, false));
        assert_eq!(c.step(0, THIRD), (0, false));
        assert_eq!(c.step(ATTACK | RIGHT, THIRD), (ATTACK | RIGHT, false));
    }

    #[test]
    fn the_combo_counts_only_the_games_chain_not_presses() {
        // presses that would not be the third hit of a chain (a miss, presses in the air, the first two hits) pass on, however many
        let mut c = on();
        for _ in 0..10 {
            assert_eq!(c.step(ATTACK, PLAIN), (ATTACK, false));
            assert_eq!(c.step(0, PLAIN), (0, false));
        }
        // the third hit: the move starts at once, the press is held back
        assert_eq!(c.step(ATTACK, THIRD), (0, true));
    }

    #[test]
    fn a_held_attack_key_starts_the_move_once_and_stays_back_until_let_go() {
        let mut c = on();
        assert_eq!(c.step(ATTACK, THIRD), (0, true));
        // held: no second move, and no attack reaches the game (it would be a new press there, a jab)
        for _ in 0..20 {
            assert_eq!(c.step(ATTACK | UP, THIRD), (UP, false));
        }
        assert_eq!(c.step(0, THIRD), (0, false));
        // a new press after letting go is a press again
        assert_eq!(c.step(ATTACK, PLAIN), (ATTACK, false));
    }

    #[test]
    fn a_waiting_move_starts_once_the_hero_is_ready() {
        let mut c = on();
        assert_eq!(c.step(ATTACK, BUSY), (0, false));
        assert_eq!(c.step(0, BUSY), (0, false));
        // more presses while it waits stay back (mashing does not make a second move)
        assert_eq!(c.step(ATTACK, BUSY), (0, false));
        assert_eq!(c.step(0, BUSY), (0, false));
        assert_eq!(c.step(LEFT, THIRD), (LEFT, true));
        // started: nothing waits any more
        assert_eq!(c.step(0, THIRD), (0, false));
        assert_eq!(c.step(0, THIRD), (0, false));
    }

    #[test]
    fn a_waiting_move_is_dropped_when_the_chain_breaks() {
        // the chain moves on or breaks (hurt, grabbed, a jump, a weapon, the enemy died, a press the game had buffered before)
        let mut c = on();
        assert_eq!(c.step(ATTACK, BUSY), (0, false));
        assert_eq!(c.step(0, Hero { third_next: false, ready: false }), (0, false));
        assert_eq!(c.step(0, THIRD), (0, false));
        assert_eq!(c.step(ATTACK, PLAIN), (ATTACK, false));
        // Q off while it waits
        let mut c = on();
        assert_eq!(c.step(ATTACK, BUSY), (0, false));
        c.toggle();
        assert_eq!(c.step(0, THIRD), (0, false));
        assert_eq!(c.step(ATTACK, THIRD), (ATTACK, false));
        // a special move key takes over
        let mut c = on();
        assert_eq!(c.step(ATTACK, BUSY), (0, false));
        c.skip(0);
        assert_eq!(c.step(0, THIRD), (0, false));
        // the wait runs out
        let mut c = on();
        assert_eq!(c.step(ATTACK, BUSY), (0, false));
        for _ in 0..Combo::WAIT {
            assert_eq!(c.step(0, BUSY), (0, false));
        }
        assert_eq!(c.step(0, THIRD), (0, false));
        assert_eq!(c.step(ATTACK, PLAIN), (ATTACK, false));
    }

    #[test]
    fn a_special_move_key_plays_its_sequence_and_the_combo_waits_for_it() {
        let mut k = Player1Keys::default();
        k.frame(0, true, None, PLAIN);
        assert!(k.combo.on);
        let seq = Move::Special.frames(RIGHT);
        assert_eq!(k.frame(0, false, Some(Move::Special), THIRD), seq[0]);
        for &b in &seq[1..] {
            // the sequence runs: no combo move, even on a press that would be the third hit
            assert_eq!(k.frame(ATTACK, false, None, THIRD), b);
        }
        // the key was held through the sequence: not a new press
        assert_eq!(k.frame(ATTACK, false, None, THIRD), ATTACK);
    }

    /// The result of a fight.
    #[derive(Debug, Default)]
    struct Fight {
        /// Hits that landed (`+$B2` going up).
        hits: u32,
        /// Third hits of a chain (`+$B2` going up to 3).
        thirds: u32,
        /// For each command move the toggle started: the chain's hit count (`+$B2`) the frame before the move's state began.
        moves: Vec<u8>,
        /// Command moves the toggle started that did not begin (no command state within 12 frames) while nothing hit the hero.
        lost: u32,
        /// Command moves the toggle started that an enemy's hit stopped (the hero was hurt before the move began).
        interrupted: u32,
        /// Command moves from the bot's own keys (its up / down steering then attack is the real command input).
        native: u32,
    }

    /// Episode 1 with hero `hero` in god mode: the hero walks to the nearest enemy and presses attack (one frame) every `gap` frames,
    /// `frames` frames long, with the combo toggle `on` or off.
    fn fight(hero: u8, on: bool, gap: u32, frames: u32) -> Fight {
        let a = rom().expect("dino.zip");
        let mut g = Game::new(a);
        g.flow_apply_config(crate::game::flow::Config::default());
        g.start_stage_at(0, 0, 0, &[hero]);
        let mut debug = crate::debug::Debug::default();
        debug.god_on();
        let mut watch = crate::HeroWatch::default();
        let mut k = Player1Keys::default();
        if on {
            k.frame(0, true, None, Hero::default());
        }
        // which way up moves Jack in depth (the walk table of the ROM)
        let mut up_sign = 0i32;
        let mut f = Fight::default();
        // frames since the toggle started a command move, and whether the hero was hurt since
        let (mut combo_before, mut state_before, mut started) = (0u8, 0u32, None::<(u32, bool)>);
        let mut z0 = None;
        for t in 0..frames {
            let p = &g.players[0];
            let (px, pz) = (p.b.xi() as i32, p.b.zi() as i32);
            let mut live = 0;
            if g.flow.mode == 0xC && g.w.scripted == 0 && up_sign == 0 {
                // learn the sign: hold up for 8 frames
                let z = *z0.get_or_insert(pz);
                if pz != z {
                    up_sign = (pz - z).signum();
                } else {
                    live = UP;
                }
            } else if up_sign != 0 {
                let target = g
                    .enemies
                    .slots
                    .iter()
                    .filter(|e| e.b.live != 0 && e.b.mode == 2)
                    .map(|e| (e.b.xi() as i32, e.b.zi() as i32))
                    .min_by_key(|&(x, z)| (x - px).abs() + (z - pz).abs());
                match target {
                    None => live = RIGHT,
                    Some((x, z)) => {
                        let (dx, dz) = (x - px, z - pz);
                        if dz.abs() > 4 {
                            live |= if dz.signum() == up_sign { UP } else { DOWN };
                        }
                        if dx.abs() > 0x2C {
                            live |= if dx > 0 { RIGHT } else { LEFT };
                        } else if dz.abs() <= 4 {
                            if (dx > 0) != (p.b.facing != 0) && dx.abs() > 4 && t % gap == 1 {
                                live |= if dx > 0 { RIGHT } else { LEFT };
                            }
                            if t % gap == 0 {
                                live |= ATTACK;
                            }
                        }
                    }
                }
            }
            let view = watch.read(&g);
            let queued = k.moves.len();
            let buttons = k.frame(live, false, None, view);
            let pp = &g.players[0];
            let pp = &g.players[0];
            let me = Game::player_handle(0);
            if queued == 0 && k.moves.len() + 1 == Move::Command.frames(RIGHT).len() {
                started = Some((0, false));
            }
            debug.before_tick(&mut g);
            g.w.input = [buttons, 0, 0];
            g.tick();
            debug.after_tick(&mut g);
            let p = &g.players[0];
            if p.combo > combo_before && p.item_type == 0 {
                f.hits += (p.combo - combo_before) as u32;
                if p.combo == 3 {
                    f.thirds += 1;
                }
            }
            // the command move entry 0x1B6D4 sets the state 0x02000430 (and the chain count to 0)
            if p.b.state() == 0x0200_0430 && state_before != 0x0200_0430 {
                match started.take() {
                    Some(_) => f.moves.push(combo_before),
                    None => f.native += 1,
                }
            }
            started = started.map(|(n, hurt)| (n + 1, hurt || p.b.sub == 2));
            if let Some((n, hurt)) = started
                && n > 12
            {
                started = None;
                if hurt {
                    f.interrupted += 1;
                } else {
                    f.lost += 1;
                }
            }
            combo_before = p.combo;
            state_before = p.b.state();
        }
        f
    }

    /// Q off, the plain chain lands its third hits and the toggle makes no command move; Q on, every command move the toggle makes comes
    /// right after two hits of a chain (so never twice for one chain, never from presses that miss) and each one begins. All four heroes,
    /// three paces.
    #[test]
    fn the_combo_turns_the_third_hit_of_a_chain_into_the_command_move() {
        if rom().is_none() {
            eprintln!("skipped: no dino.zip");
            return;
        }
        for hero in 0..4 {
            // the bot does not reach a chain at every pace, so the counts are summed over the paces
            let (mut thirds_off, mut moves_on) = (0, 0);
            for gap in [6, 9, 14] {
                let off = fight(hero, false, gap, 6000);
                let on = fight(hero, true, gap, 6000);
                let at = format!("hero {hero}, a press every {gap} frames");
                assert!(off.moves.is_empty() && off.lost == 0, "{at}, Q off: the toggle made a command move: {off:?}");
                assert!(on.moves.iter().all(|&c| c == 2), "{at}, Q on: a command move not right after two hits: {on:?}");
                assert_eq!(on.lost, 0, "{at}, Q on: a command move the toggle started did not begin: {on:?}");
                thirds_off += off.thirds;
                moves_on += on.moves.len();
            }
            assert!(thirds_off > 0, "hero {hero}, Q off: no third hit in the fights");
            assert!(moves_on > 0, "hero {hero}, Q on: no command move in the fights");
        }
    }

    /// Q on with nobody to hit (Episode 2 before its first enemies): presses never make a command move.
    #[test]
    fn presses_that_hit_nothing_never_make_the_command_move() {
        let Some(a) = rom() else {
            eprintln!("skipped: no dino.zip");
            return;
        };
        let mut g = Game::new(a);
        g.flow_apply_config(crate::game::flow::Config::default());
        g.start_stage_at(1, 0, 0, &[0]);
        let mut watch = crate::HeroWatch::default();
        let mut k = Player1Keys::default();
        k.frame(0, true, None, Hero::default());
        for t in 0..900u32 {
            let live = if t % 5 == 0 { ATTACK } else { 0 };
            let hero = watch.read(&g);
            g.w.input = [k.frame(live, false, None, hero), 0, 0];
            g.tick();
            assert_ne!(g.players[0].b.state(), 0x0200_0430, "frame {t}: a command move with nothing hit");
        }
    }
}
