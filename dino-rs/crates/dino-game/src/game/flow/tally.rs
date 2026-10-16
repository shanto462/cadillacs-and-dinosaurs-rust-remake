//! The stage clear tally (M2-24): the task 0xB97E (slot 0x30) that follows the death of a stage boss. It shows, for each hero, the panels
//! VITALITY, ENEMIES, FINAL BLOW and M.V.P. with their points, then the TOTAL, which counts down into the score at 100 a frame, and then
//! the map panel and the end (M2-25, states 4 to 0xA below). Spec: `plan/game/design/D4d-front-end-and-story.md` section 9.1.
//!
//! # The task
//!
//! The struct G.$735E (shared with the stage intro task 0xB9BE; the tally task starts with `clr.l` of its first four bytes and clears the
//! three player blocks only) is kept as a byte image, `Tally::raw`, with the original offsets: `+0` state, `+1` sub step, `+4` counter,
//! `+$12` / `+$13` the counting flag and the sound counter, and from `+$20` three blocks of 32 bytes (G.$737E, $739E, $73BE), one for each
//! hero (G.$4E4 bit k, the hero is present when his mode is not 0):
//!
//! | offset | meaning |
//! | --- | --- |
//! | +0 (long) | the scroll1 address of the panel, by the number of heroes (0xBA76, table 0xBAAE) |
//! | +4, +5, +7, +$10, +$17 | the panel of this hero is the best one: VITALITY, ENEMIES, FINAL BLOW (the hero who has the final blow flag), M.V.P., TOTAL |
//! | +6 | present |
//! | +8, +$A (words) | the position of the number sprite, by step and number of heroes (0xBAD2, table 0xBB24) |
//! | +$C (long, BCD) | the total |
//! | +$14 (word) | the Y the numbers move to at the end, 0x50 |
//! | +$16 | the total is still counting down |
//! | +$18 (long, BCD) | 100: the amount a frame |
//! | +$1C, +$1D | the blink counter and phase of the portrait of the best total |
//!
//! The state routines (table 0xB9A4): 0 (0xBA28) the tally starts, the layers dim (a fade of 4 steps, mask 0xD, delay 2, `fade.task`), 2
//! (0xBBFC, after the fade) the nine sub steps below, 4 (0xC4CE) the map panel is prepared (M2-25), 6 (0xC576) it slides in and its marker blinks,
//! 8 (0xCBA4), 0xA (0xCCFE) the end: G.$4E2 = 0 and the task ends. A routine that sleeps inside (`jsr 0xA7E`, 30 frames) goes on after it
//! in the same routine (`After`); every routine ends with the loop's `jsr 0xA76` (one frame).
//!
//! | sub | routine | what |
//! | --- | --- | --- |
//! | 0 | 0xBC28 | sound 0x49, the flash of the number palette (pool C class 22), the best vitality, the VITALITY panel of each hero (portrait, label, number object), 30 frames |
//! | 2 | 0xBD7E | the ENEMIES panel (the best kills) |
//! | 4 | 0xBEAA | the FINAL BLOW panel of the hero with the flag |
//! | 6 | 0xBFA6 | the M.V.P. panels (the best damage, ties share) |
//! | 8 | 0xC0EE | the four text rows of each panel are cleared, counter `+4` = 14 |
//! | 0xA | 0xC14C | after 14 frames: the best total, the TOTAL panels (and WINNER! with the pool C class 34 sparkle when several heroes play), 30 frames |
//! | 0xC | 0xC322 | every frame each total counts down 100 into the score (0x101E6), sound 0x49 every 11 frames, the portrait of the best total blinks; 30 frames when all are done |
//! | 0xE | 0xC444 | the panel cells are cleared, 1 frame, the layers brighten again (fade of 4 steps up) |
//! | 0x10 | 0xC4BE | after the fade the state goes to 4 and the pose lock G.$503 is released |
//!
//! The points: VITALITY (HP - 1) * 100 (20000 at HP 100), ENEMIES kills * 100 (+$10B), FINAL BLOW 20000, M.V.P. 10000 (the damage dealt
//! +$10C is the highest of the present heroes); the numbers are BCD. The values are added to the total by the number objects (pool B class 25,
//! `prop/b25.rs`). The anti tamper roll 0xD860 draws a random number for the first hero when his HP is even (it never finds a patch).

use crate::game::enemy::Ram;
use crate::game::hud::{bcd_add32, bcd_sub32};
use crate::game::{Game, Handle};

/// Offsets of the task struct G.$735E.
mod o {
    pub const STATE: usize = 0;
    pub const SUB: usize = 1;
    /// The map panel: blink phase counter (0x14 .. 1) and the marker on / off byte.
    pub const B2: usize = 2;
    pub const B3: usize = 3;
    /// The sub step 8 counter (14 frames), the map panel's marker time (0x78).
    pub const W4: usize = 4;
    pub const W6: usize = 6;
    pub const W8: usize = 8;
    pub const LA: usize = 0xA;
    pub const LE: usize = 0xE;
    /// G.$7370: the counting started (read by the number objects), G.$7371: the frames to the next count sound.
    pub const COUNTING: usize = 0x12;
    pub const SOUND: usize = 0x13;
}

/// Offsets inside a block (G.$737E + 0x20 * k).
pub mod blk {
    pub const PTR: usize = 0;
    pub const BEST_VIT: usize = 4;
    pub const BEST_ENEMIES: usize = 5;
    pub const PRESENT: usize = 6;
    pub const BEST_FB: usize = 7;
    pub const X: usize = 8;
    pub const Y: usize = 0xA;
    pub const TOTAL: usize = 0xC;
    pub const BEST_MVP: usize = 0x10;
    pub const TARGET: usize = 0x14;
    pub const COUNTING: usize = 0x16;
    pub const BEST_TOTAL: usize = 0x17;
    pub const STEP: usize = 0x18;
    pub const BLINK: usize = 0x1C;
    pub const PHASE: usize = 0x1D;
}

/// The offset of block `k` in the struct.
pub const fn block(k: usize) -> usize {
    0x20 + 0x20 * k
}

/// Bytes of the struct the tally keeps (G.$735E .. G.$73DD).
pub const SIZE: usize = 0x80;

/// The ROM tables (`FRONT_TABLES`).
mod rom {
    pub const BLOCK_PTRS: u32 = 0xBAAE;
    pub const POSITIONS: u32 = 0xBB24;
    pub const VITALITY: u32 = 0xBD72;
    pub const ENEMIES: u32 = 0xBEA0;
    pub const FINAL_BLOW: u32 = 0xBF98;
    pub const MVP: u32 = 0xC0E6;
    pub const WINNER: u32 = 0xC2C2;
    pub const TOTAL: u32 = 0xC2FA;
    pub const PORTRAITS: u32 = 0x84612;
    pub const GLYPHS: u32 = 0xF60;
}

/// Where a routine that slept inside goes on (the original keeps the return address of the task).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum After {
    /// The loop of the task: the next run dispatches the state.
    #[default]
    Loop,
    /// The task was just made: its first run clears the struct (0xB97E).
    Start,
    /// A sub step of state 2 slept 30 frames (0xA7E): `addq.b #2,1(a6)`.
    Advance,
    /// The same for the sub step 0xA, which also clears the sound counter (0xC1B4).
    AdvanceTotal,
    /// Sub step 0xE slept one frame inside 0xC46A: `addq.b #2,1(a6)`, then the fade of 0xC444.
    Cleanup,
    /// The routine slept (state 6, 60 frames) and only returns.
    Resume,
}

/// The state of the clear task.
#[derive(Clone, Debug)]
pub struct Tally {
    /// The task runs (slot 0x30).
    pub live: bool,
    /// Frames to sleep before the next run (1 = the next frame).
    pub sleep: u16,
    pub after: After,
    /// G.$735E holds this struct (it is shared with the stage intro task, whose struct the trace shows otherwise).
    pub owner: bool,
    /// G.$735E to G.$73DD.
    pub raw: Ram,
}

impl Default for Tally {
    fn default() -> Self {
        Tally { live: false, sleep: 0, after: After::Loop, owner: false, raw: Ram::default() }
    }
}

impl Game {
    // ------------------------------------------------------------------------------------------------ the task

    /// 0xAB84 (the stage clear switch): the task 0xB97E is made in slot 0x30. It runs for the first time in the same frame (the scheduler goes on
    /// to the slot 0x30 after the game task of slot 0x20).
    pub(super) fn flow_tally_start(&mut self) {
        // the struct keeps what the stage intro task left in it until the first run
        let it = self.flow.intro;
        let t = &mut self.flow.tally;
        if !t.owner {
            t.raw = Ram::default();
            t.raw.sb(o::STATE, it.state);
            t.raw.sb(o::SUB, it.sub);
            t.raw.sw(o::W6, it.w6);
            t.raw.sw(o::W8, it.w8);
            t.raw.sl(o::LA, it.src);
            t.raw.sl(o::LE, it.dst);
        }
        t.live = true;
        t.sleep = 1;
        t.after = After::Start;
    }

    /// One frame of the task: the kernel's sleep counter, the continuation of a routine that slept inside, or the state dispatch.
    pub fn flow_tally_task(&mut self) {
        if !self.flow.tally.live {
            return;
        }
        if self.flow.tally.sleep > 1 {
            self.flow.tally.sleep -= 1;
            return;
        }
        self.flow.tally.sleep = 1; // jsr 0xA76 at the end of every loop pass
        match std::mem::take(&mut self.flow.tally.after) {
            After::Loop => {}
            After::Start => {
                // 0xB97E: clr.l 0(a6), the blocks cleared (0xB9B0), and the first pass of the loop
                let t = &mut self.flow.tally;
                t.owner = true;
                t.raw.sl(0, 0);
                for k in 0x20..SIZE {
                    t.raw.sb(k, 0);
                }
            }
            After::Advance => return self.tally_sub_advance(false),
            After::AdvanceTotal => return self.tally_sub_advance(true),
            After::Cleanup => return self.tally_cleanup_fade(),
            After::Resume => return,
        }
        // 0xB98E: the state routine (table 0xB9A4)
        match self.flow.tally.raw.b(o::STATE) {
            0 => self.tally_state0(),
            2 => self.tally_state2(),
            4 => self.tally_state4(),
            6 => self.tally_state6(),
            8 => self.flow.tally.raw.sb(o::STATE, 0xA), // 0xCBA4: addq.b #2,0(a6)
            _ => self.tally_end(),                        // 0xCCFE
        }
    }

    /// The sleep inside a routine (`jsr 0xA7E` = 30 frames): the continuation `after` runs in the frame it ends.
    fn tally_sleep(&mut self, frames: u16, after: After) {
        self.flow.tally.sleep = frames;
        self.flow.tally.after = after;
    }

    /// The code after the sleep of a sub step: `addq.b #2,1(a6)`; the sub step 0xA also clears G.$7371 (0xC1B4).
    fn tally_sub_advance(&mut self, clear_sound: bool) {
        let t = &mut self.flow.tally;
        let s = t.raw.b(o::SUB);
        t.raw.sb(o::SUB, s.wrapping_add(2));
        if clear_sound {
            t.raw.sb(o::SOUND, 0);
        }
    }

    /// 0xBA28: state 0.
    fn tally_state0(&mut self) {
        let t = &mut self.flow.tally;
        let s = t.raw.b(o::STATE);
        t.raw.sb(o::STATE, s.wrapping_add(2));
        self.tally_present(); // bsr 0xBBC0
        for k in 0..3 {
            let t = &mut self.flow.tally;
            t.raw.sl(block(k) + blk::TOTAL, 0);
            t.raw.sl(block(k) + blk::STEP, 0x100); // 100 a frame (BCD)
        }
        // the layers dim: mask 0xD (the sprites, scroll2, scroll3), delay 2, 4 steps down (0x8854 = d1 -0x1000)
        self.tally_fade(false);
    }

    /// 0x8854 / 0x8838 with the settings of 0xBA54 / 0xC44E (delay 2, mask 0xD, steps 4): the fade task in slot 0x40.
    fn tally_fade(&mut self, fade_in: bool) {
        self.fade.delay = 2;
        self.fade.mask = 0x0D;
        self.fade.steps = 4;
        self.fade.task = Some(crate::game::palette::FadeTask { fade_in, sleep: 0, started: false });
    }

    /// 0xBBC0: the hero is present when his bit is in G.$4E4 and his mode byte (+$04) is not 0.
    fn tally_present(&mut self) {
        let chosen = self.flow.chosen;
        for k in 0..3 {
            if chosen & (1 << k) != 0 && self.players[k].b.mode != 0 {
                self.flow.tally.raw.sb(block(k) + blk::PRESENT, 1);
            }
        }
    }

    fn tally_is_present(&self, k: usize) -> bool {
        self.flow.tally.raw.b(block(k) + blk::PRESENT) != 0
    }

    /// 0xBBFC: state 2 waits for the fade (G.$44A), then runs the sub step.
    fn tally_state2(&mut self) {
        if self.fade.mask != 0 {
            return;
        }
        self.tally_block_pointers(); // bsr 0xBA76
        match self.flow.tally.raw.b(o::SUB) {
            0 => self.tally_sub_panel(0),
            2 => self.tally_sub_panel(2),
            4 => self.tally_sub_panel(4),
            6 => self.tally_sub_panel(6),
            8 => self.tally_sub_clear(),
            0xA => self.tally_sub_total(),
            0xC => self.tally_sub_count(),
            0xE => self.tally_sub_cleanup(),
            _ => self.tally_sub_wait_fade(),
        }
    }

    /// 0xBA76: the panel origin of each chosen hero: the table 0xBAAE, 12 bytes (three longs) by the number of heroes G.$4EF, in the order of the
    /// chosen heroes.
    fn tally_block_pointers(&mut self) {
        let row = 12 * self.w.players_minus_one as u32;
        let mut k = 0;
        for who in 0..3 {
            if self.flow.chosen & (1 << who) != 0 {
                let a = self.assets.frontend.text.u32(rom::BLOCK_PTRS, row + 4 * k);
                k += 1;
                self.flow.tally.raw.sl(block(who) + blk::PTR, a);
            }
        }
    }

    /// 0xBAD2: the position of each chosen hero's number for this step: the word at 0xBB24 + the sub step points at a table of 12 bytes rows by
    /// the number of heroes (X, Y per hero). The sub step 0xA reads the entry after the four.
    fn tally_positions(&mut self) {
        let t = &self.assets.frontend.text;
        let sub = self.flow.tally.raw.b(o::SUB) as u32;
        let base = rom::POSITIONS + t.u16(rom::POSITIONS, sub) as u32 + 12 * self.w.players_minus_one as u32;
        let mut k = 0;
        for who in 0..3 {
            if self.flow.chosen & (1 << who) != 0 {
                let (x, y) = (t.u16(base, 4 * k), t.u16(base, 4 * k + 2));
                k += 1;
                self.flow.tally.raw.sw(block(who) + blk::X, x);
                self.flow.tally.raw.sw(block(who) + blk::Y, y);
            }
        }
    }

    /// The best heroes of a panel: `value(k)` is the figure of present hero k and `flag` the block offset of the best mark. A hero is marked when
    /// no one before him is better (ties share); a better one takes the mark of the earlier ones away. The compare is signed (`bgt`) unless
    /// `unsigned` (the ENEMIES panel compares bytes with `bhi`). The listing of 0xBC96, 0xBDD4, 0xBFFC and 0xC1BA, which keeps the marks in d1
    /// and the index of the last leader in d2.
    fn tally_best(&mut self, flag: usize, value: impl Fn(&Game, usize) -> i64) {
        let (mut d0, mut d1, mut d2) = (0i64, 0u8, 0u8);
        // the first hero
        if self.tally_is_present(0) && !(d0 > value(self, 0)) {
            d0 = value(self, 0);
            d1 |= 1;
        }
        if self.tally_is_present(1) && !(d0 > value(self, 1)) {
            if d0 != value(self, 1) {
                d1 &= !(1 << d2);
            }
            d0 = value(self, 1);
            d1 |= 2;
            d2 = 1;
        }
        if self.tally_is_present(2) && !(d0 > value(self, 2)) {
            if d0 != value(self, 2) {
                d1 &= !(1 << d2);
            }
            d0 = value(self, 2);
            d1 |= 4;
            d2 = 0;
            if d1 & (1 << d2) != 0 && d0 < value(self, 0) {
                d1 &= !(1 << d2);
            }
        }
        for k in 0..3 {
            if d1 & (1 << k) != 0 {
                self.flow.tally.raw.sb(block(k) + flag, 1);
            }
        }
    }

    /// The panel steps 0 to 6 (0xBC28, 0xBD7E, 0xBEAA, 0xBFA6): the sound, (the flash of the first), the positions, the best heroes, the panel
    /// of each present hero, 30 frames.
    fn tally_sub_panel(&mut self, sub: u8) {
        self.snd_request(0x49, 0x10, 0);
        if sub == 0 {
            self.fx_spawn_tally_flash(); // jsr 0x9582A
        }
        self.tally_positions();
        match sub {
            0 => {
                // 0xBC96: the best vitality; the HP of the first hero is looked at with the anti tamper roll 0xD860, which draws a number when it is even
                self.tally_best(blk::BEST_VIT, |g, k| {
                    let hp = g.players[k].b.hp;
                    hp as i16 as i64
                });
                if self.tally_is_present(0) && self.players[0].b.hp as i16 >= 0 && self.players[0].b.hp & 1 == 0 {
                    let _ = self.rand(); // 0xD860 (btst #0,d0 / bne; jsr 0x119C)
                }
            }
            2 => self.tally_best(blk::BEST_ENEMIES, |g, k| g.spawn.g.kills[k] as i64), // bdd4: bytes, bhi
            4 => self.tally_final_blow(),
            _ => self.tally_best(blk::BEST_MVP, |g, k| g.players[k].w10c as i16 as i64), // bffc: words
        }
        for k in 0..3 {
            if sub == 0 {
                self.flow.tally.raw.sw(block(k) + blk::TARGET, 0x50);
            }
            if self.tally_is_present(k) {
                match sub {
                    0 => self.tally_panel_vitality(k),
                    2 => self.tally_panel_enemies(k),
                    4 => self.tally_panel_final_blow(k),
                    _ => self.tally_panel_mvp(k),
                }
            }
        }
        self.tally_sleep(30, After::Advance); // jsr 0xA7E
    }

    /// 0xBF00: the final blow goes to the last present hero whose flag +$C0 is set.
    fn tally_final_blow(&mut self) {
        let mut hit = None;
        for k in 0..3 {
            if self.tally_is_present(k) && self.players[k].bc0 != 0 {
                hit = Some(k);
            }
        }
        if let Some(k) = hit {
            self.flow.tally.raw.sb(block(k) + blk::BEST_FB, 1);
        }
    }

    /// 0x84604: the portrait of a hero at the panel (`flag`: the winner picture), 9 x 9 cells at +0x80.
    fn tally_portrait(&mut self, k: usize, flag: bool) {
        let a0 = self.flow.tally.raw.l(block(k) + blk::PTR).wrapping_add(0x80);
        let id = self.players[k].b.id as u32;
        let d = id + 4 * flag as u32;
        let portrait = self.assets.frontend.text.u8(rom::PORTRAITS, d) as u16;
        self.text_portrait(a0, portrait, -1);
    }

    /// 0xF1C: a string of codes `attr word, characters, 0x40` printed down the columns from `a0` (+0x80 a character): letters are looked up in the table 0xF60,
    /// the other codes are tile codes. (The code 0x2F, a second line, is not in the strings of the tally.)
    fn tally_print(&mut self, mut a0: u32, rom_string: u32) {
        let t = &self.assets.frontend.text;
        let attr = t.u16(rom_string, 0);
        let mut i = 2;
        loop {
            let c = t.u8(rom_string, i) as u16;
            i += 1;
            if c == 0x40 {
                break;
            }
            let code = if c > 0x40 { t.u16(rom::GLYPHS, 2 * (c - 0x41) as u32) } else { c };
            self.hud_put(a0, code, attr);
            a0 = a0.wrapping_add(0x80);
        }
    }

    /// 0x7E72C: the number object of this step for hero `k` (the colour 0xB when `best`, else 0xC).
    fn tally_number(&mut self, k: usize, best: bool) {
        self.prop_tally_number(k, if best { 0xB } else { 0xC }, None);
    }

    /// 0xBD26: the VITALITY panel.
    fn tally_panel_vitality(&mut self, k: usize) {
        let best = self.flow.tally.raw.b(block(k) + blk::BEST_VIT) != 0;
        self.tally_portrait(k, best);
        let a = self.flow.tally.raw.l(block(k) + blk::PTR);
        self.tally_print(a.wrapping_add(0x128), rom::VITALITY);
        self.tally_number(k, best);
    }

    /// 0xBE54: the ENEMIES panel.
    fn tally_panel_enemies(&mut self, k: usize) {
        let best = self.flow.tally.raw.b(block(k) + blk::BEST_ENEMIES) != 0;
        self.tally_portrait(k, best);
        let a = self.flow.tally.raw.l(block(k) + blk::PTR);
        self.tally_print(a.wrapping_add(0x1B4), rom::ENEMIES);
        self.tally_number(k, best);
    }

    /// 0xBF44: the FINAL BLOW panel (a label and a number only for the hero with the flag).
    fn tally_panel_final_blow(&mut self, k: usize) {
        let flag = self.flow.tally.raw.b(block(k) + blk::BEST_FB) != 0;
        self.tally_portrait(k, flag);
        if !flag {
            return;
        }
        let a = self.flow.tally.raw.l(block(k) + blk::PTR);
        self.tally_print(a.wrapping_add(0x40), rom::FINAL_BLOW);
        self.tally_number(k, true);
    }

    /// 0xC07C: the M.V.P. panel (a label and a number only for the best hero; the label is 0xC cells higher without a FINAL BLOW panel).
    fn tally_panel_mvp(&mut self, k: usize) {
        let flag = self.flow.tally.raw.b(block(k) + blk::BEST_MVP) != 0;
        self.tally_portrait(k, flag);
        if !flag {
            return;
        }
        let fb = self.flow.tally.raw.b(block(k) + blk::BEST_FB) != 0;
        let mut a = self.flow.tally.raw.l(block(k) + blk::PTR).wrapping_add(0x2CC);
        if !fb {
            a = a.wrapping_sub(0xC);
        }
        self.tally_print(a, rom::MVP);
        if !fb {
            let y = self.flow.tally.raw.w(block(k) + blk::Y).wrapping_add(0x18);
            self.flow.tally.raw.sw(block(k) + blk::Y, y);
        }
        self.tally_number(k, true);
    }

    /// 0xC0EE: sub step 8: the four text rows of every panel are cleared (code 0x20 only); `+4` = 14, G.$7370 = 0.
    fn tally_sub_clear(&mut self) {
        for k in 0..3 {
            if !self.tally_is_present(k) {
                continue;
            }
            let mut a1 = self.flow.tally.raw.l(block(k) + blk::PTR).wrapping_add(0x28); // 0xC128
            for _ in 0..4 {
                let mut a0 = a1;
                for _ in 0..10 {
                    self.hud_word(a0, 0x20);
                    a0 = a0.wrapping_add(0x80);
                }
                a1 = a1.wrapping_add(0xC);
            }
        }
        let t = &mut self.flow.tally;
        let s = t.raw.b(o::SUB);
        t.raw.sb(o::SUB, s.wrapping_add(2));
        t.raw.sw(o::W4, 0xE);
        t.raw.sb(o::COUNTING, 0);
    }

    /// 0xC14C: sub step 0xA: after 14 frames the TOTAL panels.
    fn tally_sub_total(&mut self) {
        let n = self.flow.tally.raw.w(o::W4).wrapping_sub(1);
        self.flow.tally.raw.sw(o::W4, n);
        if n != 0 {
            return;
        }
        self.tally_positions(); // bsr 0xBAD2
        self.tally_best_total(); // bsr 0xC1BA
        self.flow.tally.raw.sb(o::COUNTING, 1);
        for k in 0..3 {
            if self.tally_is_present(k) {
                self.flow.tally.raw.sb(block(k) + blk::COUNTING, 1);
                self.tally_panel_total(k);
            }
        }
        self.tally_sleep(30, After::AdvanceTotal);
    }

    /// 0xC1BA: the best total (long compares, signed).
    fn tally_best_total(&mut self) {
        self.tally_best(blk::BEST_TOTAL, |g, k| g.flow.tally.raw.l(block(k) + blk::TOTAL) as i32 as i64);
    }

    /// 0xC23A: the TOTAL panel of hero `k`: portrait (the winner picture for the best total), the label TOTAL, and with several heroes the label WINNER!
    /// of the best one and its sparkle (pool C class 34), then the total number (pool B class 25, variant 8).
    fn tally_panel_total(&mut self, k: usize) {
        let best = self.flow.tally.raw.b(block(k) + blk::BEST_TOTAL) != 0;
        self.tally_portrait(k, best);
        let a = self.flow.tally.raw.l(block(k) + blk::PTR);
        self.tally_label(a.wrapping_add(0x2C), rom::TOTAL, 10); // 0xC2DE
        if best && self.w.players_minus_one != 0 {
            self.tally_label(a.wrapping_add(0x144), rom::WINNER, 7); // 0xC2A6
            self.fx_spawn_tally_winner(a.wrapping_add(0x146)); // jsr 0x96DAA
        }
        self.prop_tally_number(k, if best { 0xB } else { 0xC }, Some(8)); // jmp 0x7E772
    }

    /// 0xC2A6 / 0xC2DE: a label of `columns` columns of two tiles (codes from the table, attribute 0), the tall letters.
    pub(crate) fn tally_label(&mut self, mut a0: u32, table: u32, columns: u32) {
        for c in 0..columns {
            let t = &self.assets.frontend.text;
            let (upper, lower) = (t.u16(table, 4 * c), t.u16(table, 4 * c + 2));
            self.hud_put(a0, upper, 0);
            self.hud_put(a0.wrapping_add(4), lower, 0);
            a0 = a0.wrapping_add(0x80);
        }
    }

    /// 0xC322: sub step 0xC: the count down.
    fn tally_sub_count(&mut self) {
        // sound 0x49 every 11 frames (G.$7371: `subq.b #1` and `bcc`: the sound when it borrows, then 10)
        let old = self.flow.tally.raw.b(o::SOUND);
        self.flow.tally.raw.sb(o::SOUND, old.wrapping_sub(1));
        if old == 0 {
            self.flow.tally.raw.sb(o::SOUND, 10);
            self.snd_request(0x49, 0x10, 0);
        }
        for k in 0..3 {
            if self.tally_is_present(k) && self.flow.tally.raw.b(block(k) + blk::COUNTING) != 0 {
                self.tally_count_down(k); // bsr 0xC3EE
                self.tally_blink(k); // bsr 0xC3A8
            }
        }
        let any = (0..3).any(|k| self.flow.tally.raw.b(block(k) + blk::COUNTING) != 0);
        if !any {
            self.tally_sleep(30, After::Advance);
        }
    }

    /// 0xC3EE: one step of the count down: the total goes down by the step (BCD), the score of the hero up; when the total is 0 the flag is cleared.
    fn tally_count_down(&mut self, k: usize) {
        let b = block(k);
        if self.flow.tally.raw.l(b + blk::TOTAL) == 0 {
            self.flow.tally.raw.sb(b + blk::COUNTING, 0);
            return;
        }
        let step = self.flow.tally.raw.l(b + blk::STEP);
        let (total, _) = bcd_sub32(self.flow.tally.raw.l(b + blk::TOTAL), step, 0);
        self.flow.tally.raw.sl(b + blk::TOTAL, total);
        if total == 0 {
            self.flow.tally.raw.sb(b + blk::COUNTING, 0);
        }
        let (score, _) = bcd_add32(self.players[k].score, step); // the carry out of the top byte is lost
        self.players[k].score = score;
        self.hud_draw_all_scores(); // jsr 0x101E6
    }

    /// 0xC3A8: the portrait of the best total changes between its two pictures, 41 and 6 frames.
    fn tally_blink(&mut self, k: usize) {
        let b = block(k);
        if self.flow.tally.raw.b(b + blk::BEST_TOTAL) == 0 {
            return;
        }
        let c = self.flow.tally.raw.b(b + blk::BLINK).wrapping_sub(1);
        self.flow.tally.raw.sb(b + blk::BLINK, c);
        if c != 0xFF {
            return; // bcc: no borrow
        }
        let phase = (self.flow.tally.raw.b(b + blk::PHASE) + 1) & 1;
        self.flow.tally.raw.sb(b + blk::PHASE, phase);
        self.flow.tally.raw.sb(b + blk::BLINK, if phase != 0 { 0x28 } else { 5 });
        let a0 = self.flow.tally.raw.l(b + blk::PTR).wrapping_add(0x80);
        let id = self.players[k].b.id as u32;
        let portrait = self.assets.frontend.text.u8(rom::PORTRAITS, id + 4 * phase as u32) as u16;
        self.text_portrait(a0, portrait, -1);
    }

    /// 0xC444: sub step 0xE: the panel cells are cleared, one frame, then the layers brighten.
    fn tally_sub_cleanup(&mut self) {
        for k in 0..3 {
            if !self.tally_is_present(k) {
                continue;
            }
            let mut a1 = self.flow.tally.raw.l(block(k) + blk::PTR);
            for _ in 0..10 {
                let mut a0 = a1;
                for _ in 0..14 {
                    self.hud_word(a0, 0x20);
                    a0 = a0.wrapping_add(4);
                }
                a1 = a1.wrapping_add(0x80);
            }
        }
        self.tally_sleep(1, After::Cleanup); // jsr 0xA76 inside 0xC46A
    }

    /// 0xC49A: after the sleep inside 0xC46A: `addq.b #2,1(a6)`, then the fade up of 0xC444.
    fn tally_cleanup_fade(&mut self) {
        let t = &mut self.flow.tally;
        let s = t.raw.b(o::SUB);
        t.raw.sb(o::SUB, s.wrapping_add(2));
        self.tally_fade(true); // 0x8838
    }

    /// 0xC4BE: sub step 0x10: with the fade over the state goes on to the map panel and the pose lock is released.
    fn tally_sub_wait_fade(&mut self) {
        if self.fade.mask != 0 {
            return;
        }
        let t = &mut self.flow.tally;
        let s = t.raw.b(o::STATE);
        t.raw.sb(o::STATE, s.wrapping_add(2));
        self.flow.script.b503 = 0;
    }

    // ------------------------------------------------------------------------------------------------ the map panel and the end

    /// 0xC4CE: state 4: scroll1 is cleared and the map panel is drawn (the frame and the picture of the stage, the palettes, the name plate), 17 cells of each
    /// column from 0x908008 are going to be copied to 0x90B808 and on (`+$A`, `+$E`); the marker blinks for `+4` = 0x78 frames in periods of `+2` = 0x14.
    /// Episode 8 has no panel: the state is 0xA.
    fn tally_state4(&mut self) {
        let t = &mut self.flow.tally;
        let s = t.raw.b(o::STATE);
        t.raw.sb(o::STATE, s.wrapping_add(2));
        t.raw.sb(o::SUB, 0);
        self.flow_clear_scroll1(); // jsr 0xB52
        if self.w.stage == 7 {
            self.flow.tally.raw.sb(o::STATE, 0xA);
            return;
        }
        let t = &mut self.flow.tally;
        t.raw.sw(o::W4, 0x78);
        t.raw.sb(o::B2, 0x14);
        t.raw.sl(o::LA, 0x908008);
        t.raw.sl(o::LE, 0x90B808);
        t.raw.sw(o::W6, 0);
        t.raw.sw(o::W8, 0xC);
        self.flow_card_palettes(); // bsr 0xCD1A
        self.flow_card_panel(0x908010); // bsr 0xCE5C
        self.flow_card_plate(0x908010); // bra 0xD75E
        // The pass of the panel draw takes the CPU through the next vertical blank when many objects are alive (the 68000 draws about 380 cells and
        // runs the object updates of the frame: the recordings with 3 heroes and with 2 heroes and a tie). The interrupt hits in the middle of the
        // draw, so that record shows the last cells (the right column of the panel and the name plate) undrawn (the port draws them at once), and the
        // next pass of the tasks starts late: its game task counts (G.$0000, G.$4DA, the splash timer) in the next record, its tally task runs one
        // record later. The cycles are not counted; the estimate is the objects in use (live byte) in pool D (the sprite lists of the panels) plus
        // twice those in pool A, 17 or more (fitted to the 31 `flow_tally` recordings, all of which it classifies right).
        let a = self.enemies.slots.iter().filter(|e| e.b.live != 0).count();
        let d = self.specials.slots.iter().filter(|e| e.b.live != 0).count();
        if d + 2 * a >= 17 {
            self.flow.tally.sleep = 2;
        }
    }

    /// 0xC576: state 6 (the map panel slides in and its marker blinks). Sub 0 (0xC59A): the columns are copied (17 cells of each, to the page behind the
    /// right edge) as far as `+8` goes, then `+8` and scroll1 X grow by 12 until X is 0x130 (26 frames). Sub 2 (0xC5F6): `+4` counts down every frame,
    /// the marker is lit when the period counter `+2` is 0x14 and dark when it is 8; at the end of a period (every 20 frames) the state goes on when `+4` is
    /// negative: after 140 frames; the marker is dark again, 60 frames of sleep (0xA82), then state 8.
    fn tally_state6(&mut self) {
        match self.flow.tally.raw.b(o::SUB) {
            0 => {
                // 0xC59A
                loop {
                    let t = &mut self.flow.tally;
                    if (t.raw.w(o::W8) as i16) < (t.raw.w(o::W6) as i16) {
                        break; // blt
                    }
                    let (src, dst) = (t.raw.l(o::LA), t.raw.l(o::LE));
                    for c in 0..17u32 {
                        let (s, d) = (src.wrapping_add(4 * c), dst.wrapping_add(4 * c));
                        let cell = self.vram.scroll1[Self::flow_cell(s)];
                        self.vram.scroll1[Self::flow_cell(d)] = cell;
                    }
                    let t = &mut self.flow.tally;
                    t.raw.sw(o::W6, t.raw.w(o::W6).wrapping_add(8));
                    t.raw.sl(o::LA, src.wrapping_add(0x80));
                    t.raw.sl(o::LE, (dst.wrapping_add(0x80) & 0x90BFFF) | 0x90A000);
                }
                let t = &mut self.flow.tally;
                t.raw.sw(o::W8, t.raw.w(o::W8).wrapping_add(0xC));
                self.cam.cam1_x = self.cam.cam1_x.wrapping_add(0xC);
                if self.cam.cam1_x >= 0x130 {
                    let t = &mut self.flow.tally;
                    t.raw.sb(o::SUB, t.raw.b(o::SUB).wrapping_add(2));
                }
            }
            _ => {
                // 0xC5F6
                let w4 = self.flow.tally.raw.w(o::W4).wrapping_sub(1);
                self.flow.tally.raw.sw(o::W4, w4);
                let b2 = self.flow.tally.raw.b(o::B2);
                if b2 == 0x14 {
                    self.flow.tally.raw.sb(o::B3, 1);
                    self.flow_card_marker(true); // bsr 0xCAA8
                }
                if b2 == 8 {
                    self.flow.tally.raw.sb(o::B3, 0);
                    self.flow_card_marker(false);
                }
                let b2 = b2.wrapping_sub(1);
                self.flow.tally.raw.sb(o::B2, b2);
                if b2 != 0 {
                    return;
                }
                self.flow.tally.raw.sb(o::B2, 0x14);
                if (w4 as i16) < 0 {
                    self.flow.tally.raw.sb(o::B3, 0);
                    self.flow_card_marker(false);
                    let t = &mut self.flow.tally;
                    t.raw.sb(o::STATE, t.raw.b(o::STATE).wrapping_add(2));
                    self.tally_sleep(60, After::Resume); // jmp 0xA82
                }
            }
        }
    }

    /// 0xCCFE: state 0xA: G.$4E2 = 0 (except in Episode 8 area 3, which has no map card), the task ends (jmp 0x910).
    fn tally_end(&mut self) {
        if !(self.w.stage == 7 && self.cam.s2.area == 3) {
            self.flow.b4e2 = 0;
        }
        self.flow.tally.live = false;
    }

    /// The pointer a number object keeps to a block or a player (a RAM address, sign extended).
    pub(crate) fn tally_ptr(handle: Handle) -> u32 {
        0xFFFF_0000 | handle as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game(present: [bool; 3]) -> Game {
        let mut g = Game::new(Box::leak(Box::new(AssetPack::default())));
        for (k, &p) in present.iter().enumerate() {
            g.flow.tally.raw.sb(block(k) + blk::PRESENT, p as u8);
        }
        g
    }

    fn marks(g: &Game, flag: usize) -> [bool; 3] {
        [0, 1, 2].map(|k| g.flow.tally.raw.b(block(k) + flag) != 0)
    }

    /// The best of a panel: the top figure, ties share the mark, absent heroes have none (0xBC96 and the three like it).
    #[test]
    fn the_best_heroes_share_ties() {
        let figures = |v: [i64; 3]| move |_: &Game, k: usize| v[k];
        let run = |present: [bool; 3], v: [i64; 3]| {
            let mut g = game(present);
            g.tally_best(blk::BEST_MVP, figures(v));
            marks(&g, blk::BEST_MVP)
        };
        assert_eq!(run([true, false, false], [5, 9, 9]), [true, false, false], "alone");
        assert_eq!(run([true, true, false], [5, 9, 0]), [false, true, false]);
        assert_eq!(run([true, true, false], [9, 5, 0]), [true, false, false]);
        assert_eq!(run([true, true, false], [7, 7, 0]), [true, true, false], "a tie");
        assert_eq!(run([true, true, true], [7, 3, 7]), [true, false, true], "the first and the third");
        assert_eq!(run([true, true, true], [3, 7, 7]), [false, true, true]);
        assert_eq!(run([true, true, true], [7, 7, 7]), [true, true, true]);
        // the listing's quirk (0xC03C): a hero who beats a tie only takes the mark of the last leader (the second), so the first keeps his
        assert_eq!(run([true, true, true], [3, 3, 9]), [true, false, true]);
        assert_eq!(run([true, true, true], [9, 3, 3]), [true, false, false]);
        assert_eq!(run([true, false, true], [4, 9, 4]), [true, false, true], "an absent hero does not count");
        assert_eq!(run([true, true, true], [-3, -9, -3]), [false, false, false], "signed: the best starts at 0");
        assert_eq!(run([true, true, false], [0, 0, 0]), [true, true, false], "zero ties too");
    }

    /// Episode 8 has no map panel (0xC4DC): the state 4 goes to 0xA at once, and the task ends (0xCCFE) leaving G.$4E2 set in the last area (3), so that
    /// the stage clear sequence waits for its timer (0xE10) and then goes to the ending, mode 0x1C.
    #[test]
    fn episode_eight_ends_without_the_map() {
        let mut g = game([true, false, false]);
        g.w.stage = 7;
        g.flow.tally.raw.sb(o::STATE, 4);
        g.tally_state4();
        assert_eq!(g.flow.tally.raw.b(o::STATE), 0xA);
        g.cam.s2.area = 3;
        g.flow.b4e2 = 1;
        g.flow.tally.live = true;
        g.tally_end();
        assert_eq!((g.flow.b4e2, g.flow.tally.live), (1, false));
        g.cam.s2.area = 0;
        g.flow.tally.live = true;
        g.tally_end();
        assert_eq!(g.flow.b4e2, 0);
    }

    /// The panel draw of a busy frame (3 heroes) makes the task late by one frame (`tally_state4`); a quiet one does not. With the ROM data (skipped without
    /// `dino.zip`).
    #[test]
    fn a_busy_panel_frame_delays_the_task() {
        let path = std::path::Path::new("../../../dino.zip");
        if !path.exists() {
            return;
        }
        let assets: &'static AssetPack = Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads")));
        let mut g = Game::new(assets);
        g.w.stage = 0;
        g.flow.tally.sleep = 1;
        g.flow.tally.raw.sb(o::STATE, 4);
        g.tally_state4();
        assert_eq!(g.flow.tally.sleep, 1, "no objects");
        for k in 0..13 {
            g.specials.slots[k].b.live = 0xFF;
        }
        for k in 0..3 {
            g.enemies.slots[k].b.live = 1;
        }
        g.tally_state4();
        assert_eq!(g.flow.tally.sleep, 2, "13 + 2 * 3 is 19");
        g.enemies.slots[0].b.live = 0;
        g.enemies.slots[1].b.live = 0;
        g.flow.tally.sleep = 1;
        g.tally_state4();
        assert_eq!(g.flow.tally.sleep, 1, "13 + 2 is 15");
        // the panel is drawn: the frame cells of the picture are in scroll1 (the first cell of the panel is at 0x908010)
        assert_ne!(g.vram.scroll1[Game::flow_cell(0x908010)], g.vram.scroll1[Game::flow_cell(0x908000)]);
    }

    /// The panel positions (block X / Y), the sizes of the struct and the blocks: the offsets the number objects and the trace use.
    #[test]
    fn the_struct_layout() {
        assert_eq!((block(0), block(1), block(2)), (0x20, 0x40, 0x60));
        assert_eq!(SIZE, 0x80);
        assert_eq!(Game::tally_ptr(0xB6F4), 0xFFFF_B6F4);
    }
}
