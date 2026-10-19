//! The test menu (`--test`, or F1 with the debug keys on): jump points, the hero and player count, and god mode. The game is paused
//! while it is open. A jump restarts the game in place with the same start as the command line (`main.rs`, `enter_target`).
//! Only the window draws it with macroquad text (`draw_window`); the CPU screenshots draw the same lines (`headless.rs`).

use crate::jumps::{self, Jump};
use macroquad::prelude::*;

/// Hero names by character id (0 Jack, 1 Mustapha, 2 Hannah, 3 Mess).
const HEROES: [&str; 4] = ["Jack", "Mustapha", "Hannah", "Mess"];

/// The footer, as the window shows it on one line (the CPU screenshot draws the two halves on two lines).
pub const FOOTER: [&str; 2] = ["F1 menu  F2 kill all  F3 skip wave", "F4 time 5:00  F5 god  F6 next area"];

/// One row of the menu. Headings are not selectable.
#[derive(Clone, Copy, PartialEq)]
enum Row {
    Heading(u8),
    Jump(usize),
    Options,
    Hero,
    Players,
    God,
}

/// What a line looks like on screen.
#[derive(Clone, Copy, PartialEq)]
pub enum Style {
    Heading,
    Normal,
    Selected,
}

pub struct Line {
    pub text: String,
    pub style: Style,
}

/// The result of Enter on the selected row.
pub enum Enter {
    Nothing,
    /// Start at this point (the menu closes).
    Jump(&'static Jump),
    /// Toggle god mode (the caller calls `Debug::toggle_god`).
    God,
}

pub struct Menu {
    pub open: bool,
    rows: Vec<Row>,
    cursor: usize,
    /// The first row on screen (set by `lines`).
    top: usize,
    /// Hero (character id 0..3) and number of players (1..3) for the next jump.
    pub hero: u8,
    pub players: u8,
}

impl Menu {
    /// `hero` is the first character of the command line (`--chars`), `players` its count. `open` at start (`--test`).
    pub fn new(hero: u8, players: u8, open: bool) -> Menu {
        let mut rows = Vec::new();
        let mut episode = 0;
        for (i, j) in jumps::JUMPS.iter().enumerate() {
            if j.episode != episode {
                episode = j.episode;
                rows.push(Row::Heading(episode));
            }
            rows.push(Row::Jump(i));
        }
        rows.extend([Row::Options, Row::Hero, Row::Players, Row::God]);
        let cursor = rows.iter().position(|r| matches!(r, Row::Jump(_))).unwrap_or(0);
        Menu { open, rows, cursor, top: 0, hero: hero.min(3), players: players.clamp(1, 3) }
    }

    /// The heroes of a jump: the hero, then the next heroes in order (2 players: hero and the next one).
    pub fn chars(&self) -> Vec<u8> {
        (0..self.players).map(|k| (self.hero + k) % 4).collect()
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    pub fn up(&mut self) {
        if let Some(i) = (0..self.cursor).rev().find(|&i| self.selectable(i)) {
            self.cursor = i;
        }
    }

    pub fn down(&mut self) {
        if let Some(i) = (self.cursor + 1..self.rows.len()).find(|&i| self.selectable(i)) {
            self.cursor = i;
        }
    }

    /// Left and Right: change the hero or the player count (the rows that have a value).
    pub fn side(&mut self, dir: i8) {
        match self.rows[self.cursor] {
            Row::Hero => self.hero = (self.hero as i8 + dir).rem_euclid(4) as u8,
            Row::Players => self.players = ((self.players as i8 - 1 + dir).rem_euclid(3) + 1) as u8,
            _ => {}
        }
    }

    /// Enter: a jump point jumps, Hero and Players change their value, God mode toggles.
    pub fn enter(&mut self) -> Enter {
        match self.rows[self.cursor] {
            Row::Jump(i) => Enter::Jump(&jumps::JUMPS[i]),
            Row::Hero | Row::Players => {
                self.side(1);
                Enter::Nothing
            }
            Row::God => Enter::God,
            _ => Enter::Nothing,
        }
    }

    fn selectable(&self, i: usize) -> bool {
        !matches!(self.rows[i], Row::Heading(_) | Row::Options)
    }

    fn text(&self, row: Row, god: bool) -> String {
        match row {
            Row::Heading(e) => format!("Episode {e}"),
            Row::Jump(i) => {
                let j = &jumps::JUMPS[i];
                format!("  {:<10} {}", j.name, j.title)
            }
            Row::Options => "Options".to_string(),
            Row::Hero => format!("  Hero: {}", HEROES[self.hero as usize]),
            Row::Players => format!("  Players: {}", self.players),
            Row::God => format!("  God mode: {}", if god { "on" } else { "off" }),
        }
    }

    /// The rows that fit in `fit` lines, scrolled so the cursor is on screen. Returns the title-less list of lines.
    pub fn lines(&mut self, fit: usize, god: bool) -> Vec<Line> {
        let fit = fit.max(1);
        if self.cursor < self.top {
            self.top = self.cursor;
        }
        if self.cursor >= self.top + fit {
            self.top = self.cursor + 1 - fit;
        }
        self.rows
            .iter()
            .enumerate()
            .skip(self.top)
            .take(fit)
            .map(|(i, &r)| Line {
                text: self.text(r, god),
                style: match r {
                    Row::Heading(_) | Row::Options => Style::Heading,
                    _ if i == self.cursor => Style::Selected,
                    _ => Style::Normal,
                },
            })
            .collect()
    }
}

/// The window's menu: a dark box over the game with macroquad text. `fit` is computed from the window height.
pub fn draw_window(menu: &mut Menu, god: bool) {
    let (w, h) = (screen_width(), screen_height());
    draw_rectangle(30.0, 30.0, w - 60.0, h - 60.0, Color::new(0.0, 0.0, 0.0, 0.88));
    draw_text("Test menu", 52.0, 66.0, 30.0, WHITE);
    let top = 96.0;
    let line = 22.0;
    let fit = ((h - 60.0 - 110.0) / line) as usize;
    for (k, l) in menu.lines(fit, god).iter().enumerate() {
        let color = match l.style {
            Style::Heading => Color::new(0.55, 0.8, 1.0, 1.0),
            Style::Selected => Color::new(1.0, 0.85, 0.2, 1.0),
            Style::Normal => Color::new(0.92, 0.92, 0.92, 1.0),
        };
        draw_text(&l.text, 52.0, top + line * k as f32, 20.0, color);
    }
    draw_text(&format!("{}  {}", FOOTER[0], FOOTER[1]), 52.0, h - 50.0, 20.0, Color::new(0.7, 0.7, 0.7, 1.0));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The heroes of a jump: the hero first, then the next heroes in order (Mess wraps to Jack).
    #[test]
    fn heroes_of_the_players_count() {
        assert_eq!(Menu::new(0, 1, false).chars(), vec![0]);
        assert_eq!(Menu::new(2, 2, false).chars(), vec![2, 3]);
        assert_eq!(Menu::new(3, 3, false).chars(), vec![3, 0, 1]);
    }

    /// Up and Down skip the headings and the Options line; Left and Right change the hero (0 to 3) and the players (1 to 3).
    #[test]
    fn cursor_skips_headings_and_settings_change_values() {
        let mut m = Menu::new(0, 1, true);
        assert!(matches!(m.enter(), Enter::Jump(j) if j.name == "ep1"));
        m.up();
        assert!(matches!(m.enter(), Enter::Jump(j) if j.name == "ep1"), "Up stops at the first point");
        // go to the settings: the last rows are Options, Hero, Players, God mode
        while !matches!(m.rows[m.cursor], Row::Hero) {
            m.down();
        }
        m.side(-1);
        assert_eq!(m.hero, 3, "Left from Jack wraps to Mess");
        m.side(1);
        assert_eq!(m.hero, 0);
        m.down();
        m.side(1);
        m.side(1);
        assert_eq!(m.players, 3);
        m.side(1);
        assert_eq!(m.players, 1, "Right from 3 wraps to 1");
        m.down();
        assert!(matches!(m.enter(), Enter::God));
    }

    /// The window lines scroll with the cursor: the selected line is always on screen.
    #[test]
    fn the_selected_line_stays_on_screen() {
        let mut m = Menu::new(0, 1, true);
        for _ in 0..40 {
            m.down();
        }
        let lines = m.lines(10, false);
        assert_eq!(lines.len(), 10);
        assert!(lines.iter().any(|l| l.style == Style::Selected), "the cursor line is visible");
    }
}
