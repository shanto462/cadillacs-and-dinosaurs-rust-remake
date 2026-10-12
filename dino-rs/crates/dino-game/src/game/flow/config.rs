//! The settings of the machine, as a small config file instead of the original's EEPROM and its
//! test menu (D4d section 4.3, 10).
//!
//! The original keeps 19 bytes of options in an EEPROM (factory image at 0xACBF4) and "cooks"
//! them at boot (0xAC794) into three bytes: G.$777A (coins and chutes), G.$777B (difficulty,
//! timer level, extra life) and G.$777C (lives, continue, demo sound, free play). The rest of the
//! game reads only those bytes ([`Settings`]). Here the options have names and live in a JSON file
//! (`dino-config.json`); [`Config::cook`] builds the three bytes the way 0xAC794 does. An absent
//! file, or an absent option, means the factory value (the values a fresh MAME run shows).
//!
//! Lookup: `--config PATH`, then `$DINO_CONFIG`, then `./dino-config.json`, then
//! `../dino-config.json`. `dino-game --print-config` prints the factory file.

// The services are used by the scenes of the next tasks (T1.2 to T1.8); until they are, parts of
// them are unused.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The cooked settings G.$777A / $777B / $777C. These are the factory values (0xACBF4 cooked by
/// 0xAC794): 1 coin 1 credit, 3-player layout with three chutes, QSound, difficulty 3, timer level
/// 1, extra life at 500000 and 1000000, 1 extra life, continue on, demo sound off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// G.$777A: bit 7 QSound, bit 6 coin mode 8, bits 3-4 chute mode, bits 0-2 coin mode 0-7.
    pub dip_a: u8,
    /// G.$777B: bits 5-6 extra life option, bits 3-4 timer level, bits 0-2 difficulty.
    pub dip_b: u8,
    /// G.$777C: bit 6 continue, bit 5 demo sound, bit 4 (unused), bit 2 free play, bits 0-1 lives.
    pub dip_c: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Config::default().cook()
    }
}

/// What the three cooked bytes mean (the original reads them with `btst` and masks at the places
/// that name them).
impl Settings {
    /// G.$777A bit 7: QSound (1) or monaural (0): the sound output flag the boot task sets.
    pub fn qsound(&self) -> bool {
        self.dip_a & 0x80 != 0
    }
    /// Coin mode 0..8 (0x9CFC6 reads bits 0-2 and bit 6).
    pub fn coin_mode(&self) -> u8 {
        if self.dip_a & 0x40 != 0 { 8 } else { self.dip_a & 7 }
    }
    /// Chute mode 0 = three players with three chutes, 1 = three players one chute, 2 = two players
    /// one chute (G.$777A bits 3-4).
    pub fn chute_mode(&self) -> u8 {
        (self.dip_a >> 3) & 3
    }
    /// "RANK" 0 (extra easy) to 7 (hardest): index of the rank tables (G.$777B bits 0-2).
    pub fn difficulty(&self) -> u8 {
        self.dip_b & 7
    }
    /// "Timer level" 0..3: the rank speed is this plus 1 (G.$777B bits 3-4).
    pub fn timer_level(&self) -> u8 {
        (self.dip_b >> 3) & 3
    }
    /// Extra life option 0..3 (G.$777B bits 5-6): 0 none, 1 at 300000 and 700000, 2 at 500000 and
    /// 1000000, 3 at 1000000 only (table 0x6BFE).
    pub fn extra_life(&self) -> u8 {
        (self.dip_b >> 5) & 3
    }
    /// Continue allowed (G.$777C bit 6; clear sets G.$4D7 at game start, 0xA804).
    pub fn continue_allowed(&self) -> bool {
        self.dip_c & 0x40 != 0
    }
    /// Demo sound: the attract and demo play their sounds (G.$777C bit 5, the sound flush 0x610).
    pub fn demo_sound(&self) -> bool {
        self.dip_c & 0x20 != 0
    }
    /// Free play (G.$777C bit 2).
    pub fn free_play(&self) -> bool {
        self.dip_c & 4 != 0
    }
    /// Extra lives at the start (G.$777C bits 0-1, 0x4DD6).
    pub fn lives(&self) -> u8 {
        self.dip_c & 3
    }
}

/// The named options. Every field has the factory value as its default, so a file may hold only
/// the options it changes. Unknown names are an error (a typo should not pass silently).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Coin mode 0..8: 0 = 1 coin 1 credit, 1 = 1 coin 2 credits, 2 = 1 coin 3 credits, 3 = 1 coin
    /// 4 credits, 4 = 1 coin 6 credits, 5 = 2 coins 1 credit, 6 = 3 coins 1 credit, 7 = 4 coins
    /// 1 credit, 8 = 2 coins to start and 1 coin to continue.
    pub coin_mode: u8,
    /// Chute mode 0..2: 0 = three players, three chutes (credits per player), 1 = three players,
    /// one chute (common credits), 2 = two players, one chute (the 2 player layout).
    pub chute_mode: u8,
    /// Free play: no coins are needed (the screens show FREE PLAY).
    pub free_play: bool,
    /// Continue after the last life.
    pub continue_allowed: bool,
    /// Play the attract and demo sounds (title tune, demo music).
    pub demo_sound: bool,
    /// Sound output: true QSound (stereo), false monaural.
    pub qsound: bool,
    /// "RANK" 0..7, 0 = extra easy, 3 = factory, 7 = hardest.
    pub difficulty: u8,
    /// "Timer level" 0..3 (easy, normal, hard, very hard): rank speed = level + 1.
    pub timer_level: u8,
    /// Extra life option 0..3: 0 none, 1 = 300000 and 700000, 2 = 500000 and 1000000, 3 = 1000000.
    pub extra_life: u8,
    /// Extra lives at the start of a game 0..3 (factory 1, so 2 lives in all).
    pub lives: u8,
}

impl Default for Config {
    /// The factory image 0xACBF4: coin mode 0, QSound, chute mode 0, difficulty 3, 1 extra life,
    /// extra life option 2, free play off, timer level 1, continue on, demo sound off.
    fn default() -> Self {
        Config {
            coin_mode: 0,
            chute_mode: 0,
            free_play: false,
            continue_allowed: true,
            demo_sound: false,
            qsound: true,
            difficulty: 3,
            timer_level: 1,
            extra_life: 2,
            lives: 1,
        }
    }
}

impl Config {
    /// 0xAC794 / 0xAC7BC: the three cooked bytes. Values out of range are cut the way the
    /// original cuts the EEPROM bytes: a coin mode above 8 is coin mode 0, a chute mode above 2 is
    /// 1, the fields are masked to their bits.
    pub fn cook(&self) -> Settings {
        let coin_mode = if self.coin_mode > 8 { 0 } else { self.coin_mode };
        let chute = if self.chute_mode > 2 { 1 } else { self.chute_mode };
        // coin mode 8 sets bit 6 and leaves bits 0-2 clear (the byte is cut to 0 above 7)
        let mut a = if coin_mode > 7 { 0 } else { coin_mode };
        a |= chute << 3;
        a |= ((coin_mode == 8) as u8) << 6;
        a |= (self.qsound as u8) << 7;
        let b = (self.difficulty & 7) | ((self.timer_level & 3) << 3) | ((self.extra_life & 3) << 5);
        let c = (self.lives & 3)
            | ((self.free_play as u8) << 2)
            | ((self.demo_sound as u8) << 5)
            | ((self.continue_allowed as u8) << 6);
        Settings { dip_a: a, dip_b: b, dip_c: c }
    }

    /// The config as a JSON file text (the factory one when `self` is the default).
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("the config is plain data") + "\n"
    }

    /// Reads a config file. The error text names the file and the problem.
    pub fn from_file(path: &Path) -> Result<Config, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Config::from_json(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn from_json(text: &str) -> Result<Config, String> {
        let c: Config = serde_json::from_str(text).map_err(|e| e.to_string())?;
        c.check()?;
        Ok(c)
    }

    /// Range check of the options (a value the original could not store is refused).
    pub fn check(&self) -> Result<(), String> {
        let range = |name: &str, v: u8, max: u8| {
            if v > max { Err(format!("{name} is {v}, the highest allowed is {max}")) } else { Ok(()) }
        };
        range("coin_mode", self.coin_mode, 8)?;
        range("chute_mode", self.chute_mode, 2)?;
        range("difficulty", self.difficulty, 7)?;
        range("timer_level", self.timer_level, 3)?;
        range("extra_life", self.extra_life, 3)?;
        range("lives", self.lives, 3)
    }

    /// Finds and reads the config: `explicit` (from `--config`), `$DINO_CONFIG`, `./dino-config.json`,
    /// `../dino-config.json`. No file is not an error (the factory values); a file that exists
    /// but is wrong is.
    pub fn locate(explicit: Option<&str>) -> Result<Config, String> {
        if let Some(p) = explicit {
            return Config::from_file(Path::new(p));
        }
        let mut places: Vec<PathBuf> = Vec::new();
        if let Some(p) = std::env::var_os("DINO_CONFIG") {
            places.push(PathBuf::from(p));
        }
        places.push(PathBuf::from("dino-config.json"));
        places.push(PathBuf::from("../dino-config.json"));
        match places.iter().find(|p| p.exists()) {
            Some(p) => Config::from_file(p),
            None => Ok(Config::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The factory image cooks to the three bytes the port always used (0x80, 0x4B, 0x41).
    #[test]
    fn factory_values_cook_to_the_known_bytes() {
        let s = Config::default().cook();
        assert_eq!((s.dip_a, s.dip_b, s.dip_c), (0x80, 0x4B, 0x41));
        assert_eq!(s, Settings::default());
        assert!(s.qsound() && s.continue_allowed() && !s.demo_sound() && !s.free_play());
        assert_eq!((s.coin_mode(), s.chute_mode(), s.difficulty(), s.timer_level(), s.extra_life(), s.lives()), (0, 0, 3, 1, 2, 1));
    }

    /// Each option lands in its bit (0xAC7BC to 0xAC868).
    #[test]
    fn options_set_their_bits() {
        let c = Config { coin_mode: 5, chute_mode: 2, qsound: false, difficulty: 7, timer_level: 3, extra_life: 1, lives: 3, free_play: true, demo_sound: true, continue_allowed: false };
        let s = c.cook();
        assert_eq!(s.dip_a, 5 | (2 << 3));
        assert_eq!(s.dip_b, 7 | (3 << 3) | (1 << 5));
        assert_eq!(s.dip_c, 3 | 4 | 0x20);
        assert_eq!((s.coin_mode(), s.chute_mode(), s.demo_sound(), s.free_play(), s.continue_allowed()), (5, 2, true, true, false));
    }

    /// Coin mode 8 ("2 coins start, 1 coin continue") sets bit 6 and clears the low bits.
    #[test]
    fn coin_mode_eight_is_bit_six() {
        let s = Config { coin_mode: 8, ..Config::default() }.cook();
        assert_eq!(s.dip_a, 0x80 | 0x40);
        assert_eq!(s.coin_mode(), 8);
    }

    #[test]
    fn json_round_trip_and_partial_files() {
        let c = Config::default();
        assert_eq!(Config::from_json(&c.to_json()).unwrap(), c);
        // only the changed options are needed
        let c = Config::from_json(r#"{ "difficulty": 5, "free_play": true }"#).unwrap();
        assert_eq!((c.difficulty, c.free_play, c.lives), (5, true, 1));
        assert_eq!(c.cook().difficulty(), 5);
        // no file text at all means the factory values
        assert_eq!(Config::from_json("{}").unwrap(), Config::default());
    }

    #[test]
    fn bad_files_are_refused() {
        assert!(Config::from_json(r#"{ "dificulty": 5 }"#).unwrap_err().contains("unknown field"));
        assert!(Config::from_json(r#"{ "difficulty": 8 }"#).unwrap_err().contains("difficulty"));
        assert!(Config::from_json(r#"{ "lives": 4 }"#).unwrap_err().contains("lives"));
        assert!(Config::from_json("not json").is_err());
    }

    /// No file means the factory values; a file named on the command line must exist.
    #[test]
    fn locate_without_a_file_gives_the_factory_values() {
        assert!(Config::locate(Some("/nonexistent/dino-config.json")).is_err());
    }
}
