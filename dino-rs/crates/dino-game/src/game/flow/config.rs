//! The settings of the machine, as a small config file instead of the original's EEPROM and its
//! test menu (D4d section 4.3, 10).
//!
//! The original keeps 19 bytes of options in an EEPROM (factory image at 0xACBF4) and "cooks"
//! them at boot (0xAC794) into three bytes: G.$777A (coins and chutes), G.$777B (difficulty,
//! timer level, extra life) and G.$777C (lives, continue, demo sound, free play). The rest of the
//! game reads only those bytes ([`Settings`]). Here the options have names and live in a plain text
//! file `dino.cfg` next to the ROM (M2-01): one `key = value` per line, `#` starts a comment.
//! [`Config::cook`] builds the three bytes the way 0xAC794 does. An absent file, or an absent
//! option, means the factory value (the values a fresh MAME run shows). The EEPROM test menu that
//! edits these bytes in the original is not ported: the file is the only way to change them.
//!
//! The options: `coin_mode`, `chute_mode`, `free_play`, `continue_allowed`, `demo_sound`, `stereo`,
//! `difficulty`, `timer_level`, `bonus_life`, `start_lives` (`dino-game --print-config` prints the
//! factory file with the meaning of each). A flag is `on` / `off` (also `true` / `false`, `yes` /
//! `no`, `1` / `0`).
//!
//! Lookup: `--config PATH`, then `$DINO_CONFIG`, then `dino.cfg` next to the ROM (`--rom`,
//! `$DINO_ROM`), then `./dino.cfg`, then `../dino.cfg`.
//!
//! Where the cooked bytes go: the boot task cooks them at frame 68 (`boot.rs`, 0xAC794) into
//! `Flow::dip`; the game start copies G.$777B into `rank.dip` (`flow_mode_init`, 0x17F8A reads the
//! difficulty and the timer level), the extra life table 0x6BFE is selected by G.$777B bits 5-6
//! (`Flow::extend_sel`, 0x6B88), the lives come from G.$777C (0x4DD6), the sound filter reads the
//! demo sound bit (`snd.rs`, 0x610) and the system task reads the coin, chute and free play bits
//! (`system.rs`, 0x9CFC6).

// Some readers of `Settings` are used only by tasks that come later (select, join, continue).
#![allow(dead_code)]

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// `coin_mode` 0..8: 0 = 1 coin 1 credit, 1 = 1 coin 2 credits, 2 = 1 coin 3 credits, 3 = 1 coin
    /// 4 credits, 4 = 1 coin 6 credits, 5 = 2 coins 1 credit, 6 = 3 coins 1 credit, 7 = 4 coins
    /// 1 credit, 8 = 2 coins to start and 1 coin to continue.
    pub coin_mode: u8,
    /// `chute_mode` 0..2: 0 = three players, three chutes (credits per player), 1 = three players,
    /// one chute (common credits), 2 = two players, one chute (the 2 player layout).
    pub chute_mode: u8,
    /// `free_play`: no coins are needed (the screens show FREE PLAY).
    pub free_play: bool,
    /// `continue_allowed`: continue after the last life.
    pub continue_allowed: bool,
    /// `demo_sound`: play the attract and demo sounds (title tune, demo music).
    pub demo_sound: bool,
    /// `stereo`: sound output QSound (stereo) or monaural.
    pub qsound: bool,
    /// `difficulty` 0..7, 0 = extra easy, 3 = factory, 7 = hardest.
    pub difficulty: u8,
    /// `timer_level` 0..3 (easy, normal, hard, very hard): rank speed = level + 1.
    pub timer_level: u8,
    /// `bonus_life` 0..3: 0 none, 1 = 300000 and 700000, 2 = 500000 and 1000000, 3 = 1000000.
    pub extra_life: u8,
    /// `start_lives` 0..3: extra lives at the start of a game (factory 1, so 2 lives in all).
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

/// The options in file order: (key, highest value, what it means). A flag has the highest value 1.
const OPTIONS: &[(&str, u8, &str)] = &[
    ("coin_mode", 8, "0 = 1 coin 1 credit, 1 = 1 coin 2 credits, 2 = 1 coin 3 credits, 3 = 1 coin 4 credits, 4 = 1 coin 6 credits,\n#   5 = 2 coins 1 credit, 6 = 3 coins 1 credit, 7 = 4 coins 1 credit, 8 = 2 coins to start, 1 coin to continue"),
    ("chute_mode", 2, "0 = three players, three chutes (credits per player), 1 = three players, one chute, 2 = two players, one chute"),
    ("free_play", 1, "on = no coins needed"),
    ("continue_allowed", 1, "on = a player can continue after the last life"),
    ("demo_sound", 1, "on = the attract mode and the demos play their sounds"),
    ("stereo", 1, "on = QSound (stereo), off = mono"),
    ("difficulty", 7, "0 = extra easy ... 3 = normal ... 7 = hardest"),
    ("timer_level", 3, "0 = easy, 1 = normal, 2 = hard, 3 = very hard (how fast the rank grows)"),
    ("bonus_life", 3, "0 = none, 1 = 300000 and 700000, 2 = 500000 and 1000000, 3 = 1000000 only"),
    ("start_lives", 3, "extra lives at the start of a game (1 = 2 lives in all)"),
];

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

    /// The value of an option as a number (flags 0 / 1), in the order of [`OPTIONS`].
    fn values(&self) -> [u8; 10] {
        [
            self.coin_mode,
            self.chute_mode,
            self.free_play as u8,
            self.continue_allowed as u8,
            self.demo_sound as u8,
            self.qsound as u8,
            self.difficulty,
            self.timer_level,
            self.extra_life,
            self.lives,
        ]
    }

    fn set(&mut self, key: &str, v: u8) {
        match key {
            "coin_mode" => self.coin_mode = v,
            "chute_mode" => self.chute_mode = v,
            "free_play" => self.free_play = v != 0,
            "continue_allowed" => self.continue_allowed = v != 0,
            "demo_sound" => self.demo_sound = v != 0,
            "stereo" => self.qsound = v != 0,
            "difficulty" => self.difficulty = v,
            "timer_level" => self.timer_level = v,
            "bonus_life" => self.extra_life = v,
            "start_lives" => self.lives = v,
            _ => unreachable!("{key} is in OPTIONS"),
        }
    }

    /// The config as the text of a `dino.cfg` file, with a comment for each option (the factory
    /// file when `self` is the default).
    pub fn to_text(&self) -> String {
        let mut out = String::from(
            "# dino.cfg: the settings of the machine. One `key = value` per line, `#` starts a comment.\n\
             # A missing file or a missing option means the factory value (shown here).\n\
             # Flags are on / off.\n",
        );
        for ((key, max, help), v) in OPTIONS.iter().zip(self.values()) {
            let value = if *max == 1 { if v != 0 { "on".to_string() } else { "off".to_string() } } else { v.to_string() };
            out += &format!("\n# {help}\n{key} = {value}\n");
        }
        out
    }

    /// Reads a config file. The error text names the file and the problem.
    pub fn from_file(path: &Path) -> Result<Config, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Config::from_text(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Reads the text of a `dino.cfg`: the options it names change, the others keep the factory
    /// value. A line is `key = value` (the key in any case), an empty line or a comment is skipped.
    /// Unknown keys, repeated keys, values out of range and lines without `=` are errors that name the line.
    pub fn from_text(text: &str) -> Result<Config, String> {
        let mut c = Config::default();
        let mut seen: Vec<String> = Vec::new();
        for (n, raw) in text.lines().enumerate() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let at = |e: String| format!("line {}: {e}", n + 1);
            let (key, value) = line.split_once('=').ok_or_else(|| at(format!("`{line}` is not `key = value`")))?;
            let (key, value) = (key.trim().to_ascii_lowercase(), value.trim());
            let Some(&(name, max, _)) = OPTIONS.iter().find(|o| o.0 == key) else {
                return Err(at(format!("unknown option `{key}`")));
            };
            if seen.iter().any(|k| k == name) {
                return Err(at(format!("`{key}` is set twice")));
            }
            seen.push(name.to_string());
            let v = if max == 1 {
                match value.to_ascii_lowercase().as_str() {
                    "on" | "true" | "yes" | "1" => 1,
                    "off" | "false" | "no" | "0" => 0,
                    _ => return Err(at(format!("{key} = {value}: use on or off"))),
                }
            } else {
                let v: u8 = value.parse().map_err(|_| at(format!("{key} = {value}: not a number")))?;
                if v > max {
                    return Err(at(format!("{key} is {v}, the highest allowed is {max}")));
                }
                v
            };
            c.set(name, v);
        }
        Ok(c)
    }

    /// Finds and reads the config: `explicit` (from `--config`), `$DINO_CONFIG`, `dino.cfg` next to
    /// the ROM, `./dino.cfg`, `../dino.cfg`. No file is not an error (the factory values); a file
    /// that exists but is wrong is.
    pub fn locate(explicit: Option<&str>) -> Result<Config, String> {
        if let Some(p) = explicit {
            return Config::from_file(Path::new(p));
        }
        let mut places: Vec<PathBuf> = Vec::new();
        if let Some(p) = std::env::var_os("DINO_CONFIG") {
            places.push(PathBuf::from(p));
        }
        if let Ok(rom) = dino_extract::rom::Rom::locate() {
            places.push(rom.with_file_name("dino.cfg"));
        }
        places.push(PathBuf::from("dino.cfg"));
        places.push(PathBuf::from("../dino.cfg"));
        match places.iter().find(|p| p.exists()) {
            Some(p) => Config::from_file(p),
            None => Ok(Config::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;
    use dino_assets::AssetPack;
    use std::sync::OnceLock;

    /// The asset pack of the user's ROM (`../../../dino.zip`), `None` when it is not there (the tests that need it skip).
    fn rom() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// The factory image cooks to the three bytes the port always used (0x80, 0x4B, 0x41).
    #[test]
    fn factory_values_cook_to_the_known_bytes() {
        let s = Config::default().cook();
        assert_eq!((s.dip_a, s.dip_b, s.dip_c), (0x80, 0x4B, 0x41));
        assert_eq!(s, Settings::default());
        assert!(s.qsound() && s.continue_allowed() && !s.demo_sound() && !s.free_play());
        assert_eq!((s.coin_mode(), s.chute_mode(), s.difficulty(), s.timer_level(), s.extra_life(), s.lives()), (0, 0, 3, 1, 2, 1));
    }

    /// The factory file (what `--print-config` prints) and a missing or empty file are the factory settings, and the
    /// text of a default `Config` is a file that reads back to the same options.
    #[test]
    fn the_default_file_is_the_factory_setup() {
        let c = Config::from_text(&Config::default().to_text()).unwrap();
        assert_eq!(c, Config::default());
        assert_eq!(c.cook().dip_b, 0x4B, "difficulty 3, timer level 1, bonus life option 2");
        assert_eq!(Config::from_text("").unwrap(), Config::default());
        assert_eq!(Config::from_text("# only a comment\n\n   \n").unwrap(), Config::default());
        // every option is in the printed file, with the factory value
        let text = Config::default().to_text();
        for (key, value) in [
            ("coin_mode", "0"),
            ("chute_mode", "0"),
            ("free_play", "off"),
            ("continue_allowed", "on"),
            ("demo_sound", "off"),
            ("stereo", "on"),
            ("difficulty", "3"),
            ("timer_level", "1"),
            ("bonus_life", "2"),
            ("start_lives", "1"),
        ] {
            assert!(text.lines().any(|l| l == format!("{key} = {value}")), "{key} = {value} is missing from the factory file");
        }
        // a changed config prints its own values
        let c = Config { difficulty: 6, free_play: true, ..Config::default() };
        assert_eq!(Config::from_text(&c.to_text()).unwrap(), c);
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
    fn partial_files_and_spellings() {
        // only the changed options are needed; comments, blank lines, spaces, case and flag spellings do not matter
        let c = Config::from_text("# my cabinet\n\n  Difficulty=5   # harder\nfree_play = Yes\nstereo = 0\ncontinue_allowed = false\n").unwrap();
        assert_eq!((c.difficulty, c.free_play, c.qsound, c.continue_allowed, c.lives), (5, true, false, false, 1));
        let s = c.cook();
        assert_eq!((s.difficulty(), s.free_play(), s.qsound(), s.continue_allowed()), (5, true, false, false));
        // every flag spelling
        for (word, on) in [("on", true), ("true", true), ("yes", true), ("1", true), ("off", false), ("false", false), ("no", false), ("0", false)] {
            assert_eq!(Config::from_text(&format!("demo_sound = {word}")).unwrap().demo_sound, on, "{word}");
        }
    }

    #[test]
    fn bad_files_are_refused_with_the_line() {
        let err = |t: &str| Config::from_text(t).unwrap_err();
        assert!(err("difficulty = 3\ndificulty = 5").contains("line 2") && err("dificulty = 5").contains("unknown option"));
        assert!(err("difficulty = 8").contains("highest allowed is 7"));
        assert!(err("start_lives = 4").contains("start_lives"));
        assert!(err("coin_mode = 9").contains("coin_mode"));
        assert!(err("coin_mode = two").contains("not a number"));
        assert!(err("coin_mode = -1").contains("not a number"));
        assert!(err("free_play = maybe").contains("on or off"));
        assert!(err("free_play").contains("not `key = value`"));
        assert!(err("difficulty = 1\ndifficulty = 2").contains("set twice"));
    }

    /// No file means the factory values; a file named on the command line must exist.
    #[test]
    fn locate_without_a_file_gives_the_factory_values() {
        assert!(Config::locate(Some("/nonexistent/dino.cfg")).is_err());
        let dir = std::env::temp_dir().join(format!("dino-cfg-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("dino.cfg");
        std::fs::write(&path, "timer_level = 2\n").unwrap();
        assert_eq!(Config::locate(Some(path.to_str().unwrap())).unwrap().timer_level, 2);
        std::fs::write(&path, "timer_level = 9\n").unwrap();
        assert!(Config::locate(Some(path.to_str().unwrap())).unwrap_err().contains("line 1"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The default file selects the entry 2 of the extra life table 0x6BFE (6B88: G.$777B bits 5-6 as a word offset), whose
    /// thresholds are 500000 and 1000000 (D4d s4.3); the file's `bonus_life` picks the other entries.
    #[test]
    fn bonus_life_picks_the_table_0x6bfe_entry() {
        let Some(a) = rom() else { return };
        let thresholds = |text: &str| {
            let mut g = Game::new(a);
            g.flow_apply_config(Config::from_text(text).unwrap());
            let sel = g.flow.extend_sel() as u32;
            let t = &g.assets.text;
            let list = 0x6BFE + t.u16(0x6BFE, sel) as u32;
            (sel, t.u32(list, 0), t.u32(list, 4))
        };
        // the list entries are BCD scores
        assert_eq!(thresholds(""), (4, 0x0050_0000, 0x0100_0000), "factory: option 2, 500000 then 1000000");
        assert_eq!(thresholds("bonus_life = 1"), (2, 0x0030_0000, 0x0070_0000));
        assert_eq!(thresholds("bonus_life = 3").1, 0x0100_0000);
    }

    /// The difficulty of the file reaches the rank code: `flow_apply_config` fills G.$777B, the game start copies it into
    /// `rank.dip` and 0x17F8A picks the start rank from the table row of that difficulty (same random numbers, so the same
    /// table index); the timer level of the file is the rank speed.
    #[test]
    fn the_file_changes_the_start_rank() {
        let Some(a) = rom() else { return };
        let start = |text: &str| {
            let mut g = Game::new(a);
            g.flow_apply_config(Config::from_text(text).unwrap());
            g.start_stage(0, &[0]);
            (g.rank.dip, g.w.rank, g.rank.speed)
        };
        let (d3, normal, speed) = start("");
        assert_eq!((d3, speed), (0x4B, 2), "factory: dip_b 0x4B, timer level 1 = rank speed 2");
        let (_, easy, _) = start("difficulty = 0");
        let (_, hard, _) = start("difficulty = 7");
        assert!(easy < normal && normal < hard, "start ranks: easy {easy}, normal {normal}, hardest {hard}");
        assert_eq!(start("timer_level = 3").2, 4);
    }
}
