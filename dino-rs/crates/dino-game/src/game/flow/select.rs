//! Character select (game mode 4, routine 0xAD46) and the part of the join task 0x6C4A that runs before and during it.
//! Owner: flow / HUD port. Spec: `D4d-front-end-and-story.md` s5 and s6 (M2-03, M2-04).
//!
//! The mode routine has three sub modes (G.$4D1): 0 the set-up (0xAD5E), 2 the main pass (0xAF7C, once per frame: the 10 second
//! timer, the three player steps, the pool B update and the sprite list), 4 the end (0xB810). The state is the struct G.$62C
//! (`Select`: 12 byte records of the three players at +0xC, +0x18 and +0x24, the shared timer at +9 / +$A, the end variables at
//! +$30 ..). A record is the cursor of one player: +0 player, +1 state (0 new, 2 choosing, 4 picked), +2 cursor position 0 to 3
//! (Jack, Hannah, Mustapha, Mess on the group picture; the hero ids are 0, 2, 1, 3, table 0xB24A), +4 the step of the picked
//! state (the palette flash), +5 the lock counter (10 frames after a player joined, no pick yet), +6 .. +8 the flash.
//!
//! * set-up (0xAD5E): layer control 0x06D6, scene palette set 1, two frames of sleep (the game task sleeps inside the routine:
//!   `Flow::resume` keeps the loop counters), the fade in, the scroll1 pages 5 and 6 (3 player layout, 3 and 4 for the 2 player one),
//!   scroll2 pages 1 and 2, scroll3 pages 6 and 9, the select music 0, the records, the timer (10 seconds), the pool B objects (two
//!   class 67 pictures, class 17, the player tags class 68 are made by the first step of each player), TIME, and the first main pass.
//! * main pass (0xAF7C): the timer counts 60 frames a second (the number is printed when the 60 frame counter runs out; while a
//!   player's lock counter is running the timer is held at 10); at 0 the three input bytes get an attack press (every player who has not
//!   chosen takes the hero under his cursor). Each started player (G.$4CD) steps its record (0xB06A): state 0 makes the tag object and
//!   takes the first free cursor position, state 2 reads the cursor (0xB122: a press of left or right moves to the next free position,
//!   sound 0x32; attack, jump or start picks, sound 0x33, after the lock) and draws the panel (0xB25E: the face, the name, POWER SPEED
//!   SKILL with the diamonds, the rating lines), state 4 flashes the hero's palette for 60 frames. The sounds carry a pan (0, 0x0B,
//!   0x15, 0x20 by position, table 0xB24E) that `Game::sound` does not record.
//! * end (0xB810): when nobody is busy and a record is in the picked state's last step, 60 frames of hold, 16 palette steps every 2
//!   frames that darken the heroes that were not chosen and brighten the chosen ones, then the fade out and the stage start 0xAC4E.
//!
//! The join task (slot 0x10, `select_join_task`) is the real routine 0x6C4A for the states a player has before the level starts: 0 absent
//! without a credit (the prompt blinks), 2 absent with a credit (waits for start or attack: takes the credit and joins), 4 joined and
//! choosing a hero (waits for the pick), 6 playing. The prompts of the in-level HUD (G.$4D5 = 0) and the states 8 to 0xC are M2-05.

use crate::game::Game;


/// ROM addresses of the select code (inside the blobs `FRONT_TABLES` lists).
mod rom {
    /// 0xAEFC: the records of the two class 67 pictures (class word, variant, Y, Z).
    pub const OBJECTS: u32 = 0xAEFC;
    /// 0xAF50: the timer digits, 4 bytes per number (tens glyph, ones glyph).
    pub const DIGITS: u32 = 0xAF50;
    /// 0xB07E: the player tag objects (class word, variant, Y, Z), 8 bytes each; 0xB096 the class 19 pictures of the 2 player layout.
    pub const TAGS: u32 = 0xB07E;
    pub const TAGS_2P: u32 = 0xB096;
    /// 0xB24A: cursor position to hero id; 0xB24E the pan of a position; 0xB28C the panel origins
    /// (3 and 2 player layout); 0xB2D2 the face lists; 0xB362 their attributes. (The tables 0xB10A, the input byte addresses of the players,
    /// and 0xB252, their objects, are the arrays `Game::w.input` and `Game::players`.)
    pub const POS_TO_ID: u32 = 0xB24A;
    #[allow(dead_code)] // read by `select_pan`
    pub const PAN: u32 = 0xB24E;
    pub const PANEL: u32 = 0xB28C;
    pub const FACE_LISTS: u32 = 0xB2D2;
    pub const FACE_ATTR: u32 = 0xB362;
    /// 0xB396: the name strings (offsets by position); 0xB3E6 POWER, SPEED and SKILL; 0xB444 the diamonds, 3 bytes per position.
    pub const NAMES: u32 = 0xB396;
    pub const LABELS: u32 = 0xB3E6;
    pub const DIAMONDS: u32 = 0xB444;
    /// 0xB47C: the rating lines of Japan, 0xB57C of the other regions (4 pointers each).
    pub const RATING_JP: u32 = 0xB47C;
    pub const RATING: u32 = 0xB57C;
    /// 0xB70E: pick palette (target RAM address, source) by position; 0xB78E the flash lists (target, list of 6 palettes).
    pub const PICK_PALETTES: u32 = 0xB70E;
    pub const FLASH: u32 = 0xB78E;
    /// 0xB8DE: the end sequence: 5 palette addresses, then the sources for the heroes not chosen (0xB8F2) and chosen (0xB906).
    pub const END_TARGET: u32 = 0xB8DE;
    pub const END_DARK: u32 = 0xB8F2;
    pub const END_BRIGHT: u32 = 0xB906;
    /// The join prompts (glyph word strings, 0xC9A) and where they go for the 3 player layout in the select (0x824E).
    pub const PROMPT_BLANK: u32 = 0x843A;
    pub const PROMPT_A: u32 = 0x84D6;
    pub const PROMPT_COIN: u32 = 0x852E;
    pub const PROMPT_B: u32 = 0x8586;
    pub const PROMPT_C: u32 = 0x85C4;
    pub const PROMPT_D: u32 = 0x8602;
    pub const PROMPT_FREE: u32 = 0x8640;
    pub const PROMPT_JOIN: u32 = 0x867E;
}

/// One record of the select struct (12 bytes at G.$62C + 0xC * (player + 1)).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rec {
    /// +0 player index.
    pub who: u8,
    /// +1 state: 0 new, 2 choosing, 4 picked.
    pub state: u8,
    /// +2 cursor position (0xFF = none yet).
    pub pos: u8,
    /// +3 (unused).
    pub b3: u8,
    /// +4 step of the picked state: 0 start, 2 the flash runs, 4 done.
    pub step: u8,
    /// +5 lock counter: no pick while it is not 0 (it counts down in the frames of a started player).
    pub pending: u8,
    /// +6 flash frame, +7 frames to the next flash frame, +8 flash rounds left.
    pub flash: u8,
    pub wait: u8,
    pub rounds: u8,
}

/// The select struct G.$62C and the end variables (+$30 ..).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Select {
    /// +0 some player is busy this pass (cleared at its start, set by the steps).
    pub busy: u8,
    /// +3 the cursor positions that are held (bits 0 to 3).
    pub taken: u8,
    /// +9 seconds on the timer, +$A frames to the next second.
    pub seconds: u8,
    pub frames: u8,
    pub rec: [Rec; 3],
    /// +$30 frames to the next palette step of the end (counted down from 2), +$31 steps left, +$32.. one flag per hero position
    /// (0 .. 3, and 4 a copy of 2: the second palette of position 2).
    pub end_wait: u8,
    pub end_steps: u8,
    pub end_flags: [u8; 5],
    /// Not in the original: the set-up sleeps inside its routine, 0 = the first part, 1 = after the sleep.
    pub part: u8,
}

impl Select {
    /// 0xAD36: clears the first 0x30 bytes (the end variables from +$30 stay).
    pub fn clear(&mut self) {
        *self = Select { end_wait: self.end_wait, end_steps: self.end_steps, end_flags: self.end_flags, ..Default::default() };
    }

    /// The struct as the 0x38 bytes of RAM at G.$62C (for the comparison with a recording).
    #[cfg(test)]
    pub fn bytes(&self) -> [u8; 0x38] {
        let mut m = [0u8; 0x38];
        m[0] = self.busy;
        m[3] = self.taken;
        m[9] = self.seconds;
        m[0xA] = self.frames;
        for (i, r) in self.rec.iter().enumerate() {
            let o = 0xC + 0xC * i;
            m[o..o + 9].copy_from_slice(&[r.who, r.state, r.pos, r.b3, r.step, r.pending, r.flash, r.wait, r.rounds]);
        }
        m[0x30] = self.end_wait;
        m[0x31] = self.end_steps;
        m[0x32..0x37].copy_from_slice(&self.end_flags);
        m
    }
}

impl Game {
    /// 0xAD46: the character select, called by the game task in mode 4.
    pub fn flow_select_mode(&mut self) {
        if self.flow.select_exit != 0 {
            return self.select_exit();
        }
        match self.flow.sub {
            0 => self.select_setup(),
            2 => self.select_main(),
            _ => self.select_end(),
        }
    }

    // ------------------------------------------------------------------------------------------------------ set-up

    /// 0xAD5E: sub mode 0. The routine sleeps two frames (`jsr 0xA76` twice) after the palette set: the game task runs it again
    /// two frames later (`Flow::resume`) and it goes on.
    fn select_setup(&mut self) {
        if self.flow.front.select.part == 0 {
            self.w.layer_ctrl = 0x06D6; // 0xAD5E
            self.flow_scene_palettes(1); // 0xAD64: moveq #1,d0; jsr 0x89E6
            self.flow.front.select.part = 1;
            self.flow.wait = 2; // 0xAD70, 0xAD76: two sleeps of one frame
            self.flow.resume = true;
            return;
        }
        self.flow.front.select.part = 0;
        // 0xAD7C: the fade in of all four palette banks, 16 steps
        self.fade.delay = 1;
        self.fade.mask = 0x0F;
        self.start_fade_task(true); // 0x8824
        // 0xAD98: the scroll1 pages (the panels), 0xADD8 the group picture, 0xADF4 the scroll3 pages
        let (p1, p2) = if self.flow.b4cf == 0 { (5, 6) } else { (3, 4) };
        self.flow_page1(0x90A000, p1);
        self.flow_page1(0x90B000, p2);
        self.flow_page2(0x90F000, 1);
        self.flow_page2(0x90F400, 2);
        self.flow_page3(0x913800, 6);
        self.flow_page3(0x913900, 9);
        self.sound(0); // 0xAE10: the select music (d1 and d2 are what the registers hold)
        let s = &mut self.flow.front.select;
        s.rec[0].who = 0; // 0xAE18
        s.rec[1].who = 1;
        s.rec[2].who = 2;
        self.flow.sub += 2; // 0xAE2A: G.$4D1 = 2
        let s = &mut self.flow.front.select;
        s.taken = 0; // 0xAE2E
        // 0xAE32: the draw list (0x49A8) and the pool B (0x4C76) start again
        self.draw_list.clear();
        self.sprites.splash_timer = 1;
        self.props = crate::game::pool_b_init();
        // 0xAE3E: the timer shows 10 for the first frame, every record is locked for 10 frames and has no cursor yet
        let s = &mut self.flow.front.select;
        s.seconds = 10;
        s.frames = 1;
        for r in s.rec.iter_mut() {
            r.pending = 10;
            r.pos = 0xFF;
        }
        // 0xAE6E: the two class 67 pictures (the golden man and Hannah's, variants 4 and 0xC)
        let t = &self.assets.frontend.text;
        for k in 0..2u32 {
            let rec = |o: u32| t.u16(rom::OBJECTS, 8 * k + o);
            let (class, var, y, z) = (rec(0), rec(2), rec(4), rec(6));
            if let Some(i) = self.props.alloc() {
                let b = &mut self.props.slots[i].b;
                b.live = 1;
                b.k20 = class;
                b.var26 = var;
                b.set_yi(y as i16);
                b.set_zi(z as i16);
            }
        }
        // 0xAE9C: class 17, the frame of the picture
        if let Some(i) = self.props.alloc() {
            let b = &mut self.props.slots[i].b;
            b.live = 1;
            b.k20 = 0x44;
        }
        // 0xAEAE: the label TIME, four 2 x 1 columns of big letters at 0x90AA8C
        let mut a = 0x90AA8Cu32;
        for (top, bottom) in [(0x194u16, 0x1A4u16), (0x168, 0x178), (0x16C, 0x17C), (0x164, 0x174)] {
            self.hud_put(a, top, 1);
            self.hud_put(a + 4, bottom, 1);
            a += 0x80;
        }
        self.select_main(); // 0xAEF8: bra 0xAF7C
    }

    // ------------------------------------------------------------------------------------------------------ main pass

    /// 0xAF0E: the number of seconds in the two big digits at 0x90AA94 (2 x 2 tiles each, the glyph table 0xAF50).
    fn select_draw_timer(&mut self) {
        let n = self.flow.front.select.seconds as u32;
        let t = &self.assets.frontend.text;
        let glyphs = [t.u16(rom::DIGITS, 4 * n), t.u16(rom::DIGITS, 4 * n + 2)];
        let mut a = 0x90AA94u32;
        for g in glyphs {
            self.hud_put(a, g, 1);
            self.hud_put(a + 4, g.wrapping_add(0x10), 1);
            self.hud_put(a + 0x80, g.wrapping_add(1), 1);
            self.hud_put(a + 0x84, g.wrapping_add(0x11), 1);
            a += 0x100;
        }
    }

    /// 0xAF7C: sub mode 2, once per frame.
    fn select_main(&mut self) {
        let s = &mut self.flow.front.select;
        s.frames = s.frames.wrapping_sub(1);
        if s.frames == 0 {
            self.select_draw_timer(); // 0xAF82
            let s = &mut self.flow.front.select;
            s.frames = 0x3C;
            s.seconds = s.seconds.wrapping_sub(1);
            if (s.seconds as i8) < 0 {
                // 0xAF90: time is up, every player presses attack
                s.seconds = 0;
                self.w.input = [0x10; 3];
            }
        }
        self.flow.front.select.busy = 0; // 0xAFA6
        for i in 0..3 {
            if self.flow.started >> i & 1 == 0 {
                continue;
            }
            // 0xAFAA / 0xAFD4 / 0xAFFE: a started player's lock counter runs (and holds the timer at 10), then its step
            let s = &mut self.flow.front.select;
            if s.rec[i].pending != 0 {
                s.rec[i].pending -= 1;
                s.seconds = 10;
                s.frames = 1;
            }
            self.select_step(i);
        }
        // 0xB028: nobody busy and a record in the last step of the picked state: the end
        let s = &self.flow.front.select;
        if s.busy == 0 && s.rec.iter().fold(0, |a, r| a | r.step) == 4 {
            self.flow.sub += 2;
        }
        self.pool_b_update(); // 0xB044: jsr 0x23604
        self.build_sprite_list(); // jmp 0x13FA6
    }

    /// The pan of the sounds of a cursor position (table 0xB24E: 0x00, 0x0B, 0x15, 0x20), the d1 of the sound request (`Game::sound` records
    /// only the code: the sound port, M4, takes the pan from here).
    #[allow(dead_code)]
    pub fn select_pan(&self, pos: u8) -> u8 {
        self.assets.frontend.text.u8(rom::PAN, pos as u32)
    }

    /// 0xB06A: the step of the record of player `i`.
    fn select_step(&mut self, i: usize) {
        match self.flow.front.select.rec[i].state {
            0 => {
                // 0xB0A6: the tag object of the player (class 68), and for the 2 player layout the picture of the hero (class 19)
                let t = &self.assets.frontend.text;
                let rec = |base: u32, o: u32| t.u16(base, 8 * i as u32 + o);
                if let Some(k) = self.props.alloc() {
                    let (class, var, y, z) = (rec(rom::TAGS, 0), rec(rom::TAGS, 2), rec(rom::TAGS, 4), rec(rom::TAGS, 6));
                    let b = &mut self.props.slots[k].b;
                    b.live = 1;
                    b.k20 = class;
                    b.var26 = var;
                    b.set_yi(y as i16);
                    b.set_zi(z as i16);
                }
                if self.flow.b4cf != 0 {
                    if let Some(k) = self.props.alloc() {
                        let (class, var, x, y) = (rec(rom::TAGS_2P, 0), rec(rom::TAGS_2P, 2), rec(rom::TAGS_2P, 4), rec(rom::TAGS_2P, 6));
                        let b = &mut self.props.slots[k].b;
                        b.live = 1;
                        b.k20 = class;
                        b.var26 = var;
                        b.set_xi(x as i16);
                        b.set_yi(y as i16);
                    }
                }
                let s = &mut self.flow.front.select;
                s.rec[i].state = 2; // 0xB0F0
                // 0xB050: the first free cursor position
                let mut p = 0u8;
                while s.taken >> p & 1 != 0 {
                    p += 1;
                    if p == 3 {
                        break;
                    }
                }
                s.rec[i].pos = p;
                s.taken |= 1 << p;
                self.select_cursor(i); // 0xB0FA
                self.select_panel(i);
                self.flow.front.select.busy = 1;
            }
            2 => {
                self.select_cursor(i);
                self.select_panel(i);
                self.flow.front.select.busy = 1;
            }
            _ => self.select_picked(i),
        }
    }

    /// 0xB122: the cursor of player `i`: a press of left or right moves it to the next free position (sound 0x32), a press of
    /// attack, jump or start picks the hero (sound 0x33) unless the record is still locked.
    fn select_cursor(&mut self, i: usize) {
        let who = self.flow.front.select.rec[i].who as usize;
        let start = self.w.system & !self.flow.front.prev_system; // 0xB128: G.$1A and not G.$1F
        let mut pick = start >> who & 1 != 0;
        let mut dir: i8 = 0;
        if !pick {
            let (cur, before) = (self.w.input[who], self.flow.front.prev_input[who]); // the bytes of the table 0xB10A (G.$16 + who, G.$1B + who)
            let edge = cur & (cur ^ before); // 0xB14C: the buttons that went down
            if edge == 0 {
                return;
            }
            if edge & 2 != 0 {
                dir = -1; // left
            } else if edge & 1 != 0 {
                dir = 1; // right
            } else if edge & 0x30 != 0 {
                pick = true; // attack or jump
            } else {
                return;
            }
        }
        let pos = self.flow.front.select.rec[i].pos;
        if pick {
            // 0xB17E
            if self.flow.front.select.rec[i].pending != 0 {
                return;
            }
            let pan = self.select_pan(pos) as u32;
            self.sound_ex(0x33, pan, 0); // 0xB17E: d1 = the pan of the position, d2 = 0
            let t = &self.assets.frontend.text;
            let id = t.u8(rom::POS_TO_ID, pos as u32);
            self.flow.front.select.taken |= 1 << pos; // 0xB1BE
            let p = &mut self.players[who];
            p.b.id = id; // 0xB1CA: +3 = the hero, +$20 = hero * 4, +0 = 1
            p.b.k20 = id as u16 * 4;
            p.b.live = 1;
            if id & 1 == 0 && self.flow.front.select.seconds == 3 {
                // 0xB1E4: a random number 3 seconds before the end for Jack and Hannah, and with probability 1 in 32 the check
                // of the program checksum 0x5A454 (it passes on the original program: nothing happens)
                let _ = self.rand() & 0x1F;
            }
            self.flow.front.select.rec[i].state = 4; // 0xB204
        } else {
            // 0xB20C
            let pan = self.select_pan(pos) as u32;
            self.sound_ex(0x32, pan, 0); // 0xB20C: d1 = the pan of the old position, d2 = 0
            let s = &mut self.flow.front.select;
            s.taken &= !(1 << pos);
            let mut p = pos;
            loop {
                p = (p as i8 + dir) as u8 & 3;
                if s.taken >> p & 1 == 0 {
                    break;
                }
            }
            s.taken |= 1 << p;
            s.rec[i].pos = p;
        }
    }

    /// 0xB25E: the panel of the record: the face, the name, the labels with the diamonds and the rating lines.
    fn select_panel(&mut self, i: usize) {
        let pos = self.flow.front.select.rec[i].pos as u32;
        let t = &self.assets.frontend.text;
        let layout = if self.flow.b4cf != 0 { 3 } else { 0 };
        let base = t.u32(rom::PANEL, 4 * (i as u32 + layout));
        // 0xB2A0: the face, 4 x 4 cells (the list is column by column)
        let attr = t.u16(rom::FACE_ATTR, 2 * pos);
        let list = t.u32(rom::FACE_LISTS, 4 * pos);
        for k in 0..16u32 {
            self.hud_put(base + (k / 4) * 0x80 + (k % 4) * 4, t.u16(list, 2 * k), attr);
        }
        // 0xB36A: the name, one cell per character along the row, from 4 columns right of the face
        let name = rom::NAMES + t.u16(rom::NAMES, 2 * pos) as u32;
        let mut a = base + 0x200;
        for c in self.flow_sel_str(name) {
            self.hud_put(a, c as u16, 0);
            a += 0x80;
        }
        // 0xB3CA: POWER, SPEED and SKILL on the next three rows, 0xB3F8 the diamonds after them
        let labels = self.flow_sel_str(rom::LABELS);
        let a3 = base + 0x204;
        let mut at = a3;
        let mut n = 0;
        for c in labels {
            if c == b'@' {
                n += 1;
                at = a3 + 4 * n;
                continue;
            }
            self.hud_put(at, c as u16, 0);
            at += 0x80;
        }
        let t = &self.assets.frontend.text;
        for row in 0..3u32 {
            let count = t.u8(rom::DIAMONDS, 3 * pos + row) as i32;
            for k in 0..5u32 {
                let (code, attr) = if (k as i32) < count { (0x693, 6) } else { (0x20, 0) };
                self.hud_put(a3 + 0x280 + 4 * row + 0x80 * k, code, attr);
            }
        }
        // 0xB450: the rating, two lines of words ending in 0x40 each (the attribute is 0 in Japan, 1 elsewhere)
        let t = &self.assets.frontend.text;
        let region_attr = (self.flow.region.0 != 0) as u16;
        let table = if self.flow.region.0 == 0 { rom::RATING_JP } else { rom::RATING };
        let mut list = t.u32(table, 4 * pos);
        for line in 0..2u32 {
            let mut a = base + 0x10 + 4 * line;
            loop {
                let w = self.assets.frontend.text.u16(list, 0);
                list += 2;
                if w == 0x40 {
                    break;
                }
                self.hud_put(a, w, region_attr);
                a += 0x80;
            }
        }
    }

    /// The bytes of a '@' ended string of the select tables.
    fn flow_sel_str(&self, a: u32) -> Vec<u8> {
        let t = &self.assets.frontend.text;
        let mut v = Vec::new();
        let mut k = 0;
        loop {
            let c = t.u8(a, k);
            k += 1;
            // 0xB37C ends a string at 0x40; the label list holds three strings, each ends with '@' (0x40) and the list ends after the third
            if c == 0x40 {
                if a == rom::LABELS && v.iter().filter(|&&x| x == b'@').count() < 2 {
                    v.push(c);
                    continue;
                }
                return v;
            }
            v.push(c);
        }
    }

    // ------------------------------------------------------------------------------------------------------ picked

    /// 0xB6A2: the state of a picked record. Step 0 sets the palette of the hero (0xB6BE: the colours of its picture jump to the
    /// picked set; position 2 sets a second palette), step 2 flashes the palette from the 6 frame lists (every 2 frames, 6 frames
    /// a round, 5 rounds: 60 frames) and keeps the select busy, step 4 waits for the others.
    fn select_picked(&mut self, i: usize) {
        match self.flow.front.select.rec[i].step {
            0 => {
                let s = &mut self.flow.front.select;
                s.rec[i].step += 2; // 0xB6B6
                let pos = s.rec[i].pos as u32;
                let t = &self.assets.frontend.text;
                let (target, source) = (t.u32(rom::PICK_PALETTES, 8 * pos), t.u32(rom::PICK_PALETTES, 8 * pos + 4));
                self.select_palette_or(target, source);
                if pos == 2 {
                    // 0xB6EA: Mustapha has a second palette (the same source, 4 more times)
                    for _ in 0..4 {
                        self.select_palette_or(0x914860, 0xBFFB2);
                    }
                }
                let r = &mut self.flow.front.select.rec[i];
                r.wait = 1; // 0xB6FC
                r.rounds = 4;
                r.flash = 0;
            }
            2 => {
                let r = &mut self.flow.front.select.rec[i];
                r.wait = r.wait.wrapping_sub(1);
                if r.wait == 0 {
                    let pos = r.pos as u32;
                    let flash = r.flash as u32;
                    let t = &self.assets.frontend.text;
                    let target = t.u32(rom::FLASH, 8 * pos);
                    let list = t.u32(rom::FLASH, 8 * pos + 4);
                    let source = t.u32(list, 4 * flash);
                    self.select_palette_or(target, source); // 0xB752
                    let r = &mut self.flow.front.select.rec[i];
                    r.wait = 2;
                    r.flash += 1;
                    if r.flash > 5 {
                        r.flash = 0;
                        r.rounds = r.rounds.wrapping_sub(1);
                        if (r.rounds as i8) < 0 {
                            r.step = 4; // 0xB780
                        }
                    }
                }
                self.flow.front.select.busy = 1; // 0xB786
            }
            _ => {}
        }
    }

    /// 0xB6D0: 16 palette words at the palette RAM address `target` become the words at the ROM address `source`, each keeping the
    /// brightness nibble (0xF000) it has.
    fn select_palette_or(&mut self, target: u32, source: u32) {
        let first = ((target - 0x914000) / 2) as usize;
        for k in 0..16 {
            let s = self.assets.frontend.text.u16(source, 2 * k as u32);
            let w = &mut self.pal.words[first + k];
            *w = (*w & 0xF000) | s;
        }
        self.pal.dirty = true;
    }

    // ------------------------------------------------------------------------------------------------------ end

    /// 0xB810: sub mode 4 (the byte G.$4D2 is its step: 0 the flags, 2 the palette steps, 4 after them). When the 16 palette steps
    /// are over the routine sleeps 60 frames (0xA82), starts the fade out of the screen (0x8840) and polls the fade mask once a frame
    /// until it is 0, then jumps to the stage start 0xAC4E.
    fn select_end(&mut self) {
        match (self.flow.sub2 >> 8) as u8 {
            0 => {
                // 0xB824: the flags of the positions that were chosen
                self.flow.sub2 += 0x200;
                let s = &mut self.flow.front.select;
                s.end_wait = 2;
                s.end_steps = 0x10;
                s.end_flags = [0; 5];
                for r in s.rec {
                    if r.pos != 0xFF {
                        s.end_flags[r.pos as usize & 3] = 1; // 0xB84C
                    }
                }
                s.end_flags[4] = s.end_flags[2]; // 0xB85A
                self.select_end_steps();
            }
            2 => self.select_end_steps(),
            _ => {
                let s = &mut self.flow.front.select;
                match s.part {
                    0 => {
                        // 0xB8AA: the 60 frame sleep is over: the fade out of the screen
                        s.part = 1;
                        self.fade.delay = 1;
                        self.fade.mask = 0x0F;
                        self.start_fade_task(false); // 0x8840
                    }
                    _ => {
                        if self.fade.mask == 0 {
                            // 0xB8D8: the fade is over (G.$44A is 0): jmp 0xAC4E, phase 0 (see `flow_stage_start_staged`)
                            s.part = 0;
                            self.flow.select_exit = 1;
                            self.flow_stage_start_staged(0);
                        }
                        // else 0xB8CC: jsr 0xA76, tst.b G.$44A, bne: the routine polls the fade mask once a frame
                    }
                }
                self.flow.wait = 1; // the routine goes on in the next frame
                self.flow.resume = true;
            }
        }
    }

    /// The phases 1 to 4 of 0xAC4E after the select (one frame each), then the game task runs the level.
    pub(super) fn select_exit(&mut self) {
        let phase = self.flow.select_exit;
        self.flow.select_exit += 1;
        self.flow_stage_start_staged(phase);
        if phase == 4 {
            self.flow.select_exit = 0; // the mode is 0xC
            // The loop of the game task goes on at once (the flag G.-$7EAE of the resumed routine: 0xA914 does not sleep, 0x9B6): the
            // frame counters count and the first level frame runs in this frame. It takes the CPU through the next video frame too (the
            // area setup, the map, the start of the script): no task runs in that one (the recording: G.$0000 counts in the frame
            // of the mode 0xC and again 2 frames later).
            self.w.frame = self.w.frame.wrapping_add(1);
            self.flow.frame = self.flow.frame.wrapping_add(1);
            self.flow.wait = self.flow.speed & 0xF;
            self.game_frame();
            self.flow_ingame_switch();
            // Episodes 3 and 5 (the stages 2 and 4) take 3 video frames: the area start reads more map for them (after their story panels, the records)
            self.flow.lag = if matches!(self.w.stage, 2 | 4) { 2 } else { 1 };
            return;
        }
        self.flow.wait = 1;
        self.flow.resume = true;
    }

    /// 0xB860: one frame of the end steps: every 2 frames one step of 16.
    fn select_end_steps(&mut self) {
        let s = &mut self.flow.front.select;
        s.end_wait = s.end_wait.wrapping_sub(1);
        if s.end_wait != 0 {
            return;
        }
        s.end_steps = s.end_steps.wrapping_sub(1);
        if (s.end_steps as i8) < 0 {
            // 0xB8A6: the steps are over: G.$4D2 = 4, sleep 60 frames (0xA82), the routine goes on at 0xB8AA
            self.flow.sub2 += 0x200;
            self.flow.wait = 60;
            self.flow.resume = true;
            return;
        }
        s.end_wait = 1; // 0xB86E
        let flags = s.end_flags;
        let t = &self.assets.frontend.text;
        for d in 0..5u32 {
            let target = t.u32(rom::END_TARGET, 4 * d);
            let (chosen, source) = (flags[d as usize] != 0, if flags[d as usize] != 0 { rom::END_BRIGHT } else { rom::END_DARK });
            let source = t.u32(source, 4 * d);
            self.select_palette_step(target, source, chosen);
        }
    }

    /// 0xB91A / 0xB94C: every colour channel of the 16 words moves one step towards the target palette: down when the hero was
    /// not chosen, up when it was.
    fn select_palette_step(&mut self, target: u32, source: u32, up: bool) {
        let first = ((target - 0x914000) / 2) as usize;
        for k in 0..16 {
            let goal = self.assets.frontend.text.u16(source, 2 * k as u32);
            let w = &mut self.pal.words[first + k];
            let mut cur = *w;
            // the original rotates the word by 4 bits three times and treats the nibbles 0, 1 and 2 (the three colours; the brightness
            // nibble is the top one and stays)
            for n in 0..3 {
                let shift = 4 * n;
                let (c, g) = ((cur >> shift) & 0xF, (goal >> shift) & 0xF);
                let c = if up {
                    if c < g { c + 1 } else { c }
                } else if c > g {
                    c - 1
                } else {
                    c
                };
                cur = (cur & !(0xF << shift)) | (c << shift);
            }
            *w = cur;
        }
        self.pal.dirty = true;
    }
}

// ----------------------------------------------------------------------------------------------------------------- join task

impl Game {
    /// 0x6C4A: the join task (slot 0x10) while the game is before its level: modes 0 and 4. Each pass sets the player numbers of the
    /// player objects (0x6C4E: +2 = 0, 1, 2), runs the state routine of each player (0x6D60) and counts the prompt hold G.$477 down,
    /// then sleeps one frame. TODO(M2-05): the switch to the continue mode (0x6C8A .. 0x6D4A) and the states 6 to 0xC of the level.
    pub fn select_join_task(&mut self) {
        if !matches!(self.flow.mode, 0 | 4) || !self.flow.join_task.tick() {
            return;
        }
        for i in 0..3 {
            self.players[i].b.who = i as u8; // 0x6C4E, 0x6C52, 0x6C58
        }
        for i in 0..3 {
            if i == 2 && self.flow.b4cf != 0 {
                break; // 0x6C76: the 2 player layout has no third player
            }
            match self.flow.join[i].state {
                0 => self.select_join_absent(i),
                2 => self.select_join_credit(i),
                4 => self.select_join_choosing(i),
                _ => {} // 6: playing (0x72EE), TODO(M2-05)
            }
        }
        let s = &mut self.flow.front.sys; // 0x6D4A
        if s.hold != 0 {
            s.hold -= 1;
        }
        self.flow.join_task.wait(1, 0);
    }

    /// 0x6D7C: a player that is not there and has no credit: the prompt blinks (INSERT COIN); a credit, free play or a live player object
    /// (the starter of the game) goes to state 2.
    fn select_join_absent(&mut self, i: usize) {
        if self.flow.b4db != 0 {
            return;
        }
        let s = self.flow.front.sys;
        let credit = if self.players[i].b.live != 0 || s.free_play {
            true
        } else if s.common {
            s.credits != 0 || s.common_coins != 0
        } else {
            s.slots[i].coins != 0 || s.slots[i].credits != 0
        };
        if credit {
            self.flow.join[i].state = 2; // 0x6DCC
            self.flow.front.sys.hold = 2;
            // 0x6DD8: one random number; one time in 32 the program is scanned for patches (0x86B3C), which finds none
            let _ = self.rand() & 0x1F;
            return;
        }
        // 0x6DFC: the prompt of a player without a credit, by the frame counter G.$4DA
        match self.flow.frame & 0x7F {
            0 => self.select_join_prompt(i, rom::PROMPT_A), // 0x8050
            0x30 | 0x70 => self.select_join_prompt(i, rom::PROMPT_BLANK), // 0x80A2
            0x3F => self.select_join_prompt(i, rom::PROMPT_COIN), // 0x8104
            _ => {}
        }
    }

    /// 0x6E2A: a player that is not there and has a credit (or coins waiting, or free play): the prompt, then the start check; a start
    /// takes the credit and joins (0x7072).
    fn select_join_credit(&mut self, i: usize) {
        if self.flow.b4db != 0 {
            return;
        }
        if self.players[i].b.live != 0 {
            self.flow.front.sys.hold = 3; // 0x6EA4
            self.select_join_init(i);
            return;
        }
        let s = self.flow.front.sys;
        if s.free_play {
            // 0x6F5A: FREE PLAY style prompt
            match self.flow.frame & 0x3F {
                0 => self.select_join_prompt(i, rom::PROMPT_FREE),
                0x30 => self.select_join_prompt(i, rom::PROMPT_BLANK),
                _ => {}
            }
        } else {
            let (credits, coins) = if s.common { (s.credits, s.common_coins) } else { (s.slots[i].credits, s.slots[i].coins) };
            if credits == 0 && coins == 0 {
                self.flow.join[i].state = 0; // 0x6E62
                return;
            }
            self.select_join_credit_prompt(i, credits, coins); // 0x6EB0 / 0x6EC8
        }
        // 0x6E8A: a start?
        if !self.select_join_start_check(i) {
            return;
        }
        // 0x6E92: jsr 0xA6B74 (the play statistics, not ported), 0x18410 (the rank speed), 0x1049E (the clock)
        let mode = self.flow.mode;
        self.rank_event_18410(mode);
        self.select_join_clock();
        self.flow.front.sys.hold = 3;
        self.select_join_init(i);
    }

    /// 0x6EDC: the prompt of a player with credits or coins by the frame counter: PUSH START style with a credit (0x867E), the coin
    /// count prompts without one (0x8586, 0x8602 with the coins per credit, 0x85C4).
    fn select_join_credit_prompt(&mut self, i: usize, credits: u8, coins: u8) {
        let s = self.flow.front.sys;
        let frame = self.flow.frame;
        if s.two_coin_start && credits < 2 {
            // 0x6F34
            match frame & 0x7F {
                0 => self.select_join_prompt(i, rom::PROMPT_B),
                0x30 | 0x70 => self.select_join_prompt(i, rom::PROMPT_BLANK),
                0x3F => self.select_join_prompt(i, rom::PROMPT_C),
                _ => {}
            }
        } else if credits != 0 {
            // 0x6EEC
            match frame & 0x3F {
                0 => self.select_join_prompt(i, rom::PROMPT_JOIN), // 0x81D6
                0x30 => self.select_join_prompt(i, rom::PROMPT_BLANK),
                _ => {}
            }
        } else if coins != 0 {
            // 0x6F08
            match frame & 0x7F {
                0 => self.select_join_prompt(i, rom::PROMPT_B),
                0x30 | 0x70 => self.select_join_prompt(i, rom::PROMPT_BLANK),
                0x3F => {
                    // 0x81E6: the string and under it the coins per credit
                    self.select_join_prompt(i, rom::PROMPT_D);
                    if let Some(a1) = self.select_join_prompt_at(i) {
                        self.hud_put(a1 + 4, s.coins_per_credit as u16, 0);
                    }
                }
                _ => {}
            }
        }
    }

    /// 0x6F70: does player `i` start (or join)? A press of its start button with a credit (a free play game needs none); without a
    /// start press the attack button joins too, outside Japan and with separate credits (0x6FD4). Takes the credit.
    fn select_join_start_check(&mut self, i: usize) -> bool {
        let s = self.flow.front.sys;
        if !s.free_play {
            let credits = if s.common { s.credits } else { s.slots[i].credits };
            if s.two_coin_start {
                if credits < 2 {
                    return false; // 0x6F90
                }
            } else if credits == 0 {
                return false; // 0x6F9C
            }
        }
        let who = self.players[i].b.who as usize;
        let start = (self.w.system ^ self.flow.front.prev_system) & self.w.system; // 0x6FA2: the press of a start button
        let mut yes = start >> who & 1 != 0;
        if !yes {
            // 0x6FD4: attack alone joins (not in Japan, not with the common pool)
            if self.flow.region.0 == 0 || s.common {
                return false;
            }
            let (cur, before) = (self.w.input[who], self.flow.front.prev_input[who]); // 0x7034
            yes = cur & (cur ^ before) & 0x10 != 0;
        }
        if !yes {
            return false;
        }
        if !s.free_play {
            let n = if s.two_coin_start { 2 } else { 1 }; // 0x6FF2
            let c = if s.common { &mut self.flow.front.sys.credits } else { &mut self.flow.front.sys.slots[i].credits };
            *c = c.saturating_sub(n);
        }
        true
    }

    /// 0x1049E: a joining player adds 3:30 to the stage clock (at most 4:00), 600 frames to the wave time budget G.$50C and unlocks the
    /// clock (G.$4EB).
    fn select_join_clock(&mut self) {
        self.spawn.g.wave_timer = self.spawn.g.wave_timer.wrapping_add(0x258);
        let bcd = |v: u8| (v >> 4) as u32 * 10 + (v & 15) as u32;
        let to_bcd = |v: u32| (((v / 10) << 4) | (v % 10)) as u8;
        let total = (bcd(self.rank.time_min) * 60 + bcd(self.rank.time_sec) + 210).min(240);
        self.rank.time_min = to_bcd(total / 60);
        self.rank.time_sec = to_bcd(total % 60);
        self.rank.time_lock = 0;
    }

    /// 0x7072: the player joins: its object is initialised (0x4D46 / 0x4D5E / 0x4D76: one random number for the power factor), the prompt is
    /// erased, the object gets its state (0x4DE8), the player is marked present (G.$4CD) and goes to state 4. TODO(M2-05): the strip of
    /// heroes of an in-level join (0x723E, 0x7476, 0x7F86).
    fn select_join_init(&mut self, i: usize) {
        self.flow_boot_player(i);
        self.select_join_prompt(i, rom::PROMPT_BLANK); // 0x709A: bsr 0x80A2
        let p = &mut self.players[i];
        p.b.set_state(0x0002_0000); // 0x4DE8
        p.b.hp_bar = 0x64;
        p.b.hp = 0x64;
        p.b.hp_prev = 0x64;
        p.b.facing = 1;
        let j = &mut self.flow.join[i];
        j.seconds = if self.flow.b4cf != 0 { 6 } else { 0x11 }; // 0x70B4
        j.frames = 1;
        // 0x70CE: `clr.w $80(a6)`, then the strip of heroes moves the cursor of the object one step back and one forward (0x70D4, 0x70DA: 0x723E skips the heroes
        // that started players hold), which gives the object the hero (+$03) and the class word (+$20) it shows until the pick: a 3 player game starts with
        // Jack, Hannah and Mustapha on the objects of the three players (ep8_3p 55228)
        self.players[i].w80 = 0;
        self.flow_join_cursor(i, -1);
        self.flow_join_cursor(i, 1);
        self.flow.started |= 1 << self.players[i].b.who; // 0x70DE
        self.flow.join[i].state = 4; // 0x70E6
    }

    /// 0x70F0: a joined player waits for its pick (the select sets its object present): then it is chosen (G.$4E4) and plays (state 6).
    fn select_join_choosing(&mut self, i: usize) {
        if self.players[i].b.live != 0 {
            self.flow.chosen |= 1 << self.players[i].b.who; // 0x71DA
            self.flow.join[i].state = 6;
        }
        // TODO(M2-05): the strip of heroes of an in-level join when the mode is 0xC (0x70F8 ..)
    }

    /// The scroll1 address of the prompt of player `i` in the select (G.$4D5 set, mode 4): the 3 player layout 0x824E, the 2 player layout
    /// 0x80FC. `None` outside the select: the prompts of the level HUD are M2-05.
    fn select_join_prompt_at(&self, i: usize) -> Option<u32> {
        if self.spawn.g.b4d5 == 0 || self.flow.mode != 4 {
            return None;
        }
        Some(if self.flow.b4cf == 0 { [0x90A0E8, 0x90A8E8, 0x90B0E8][i] } else { [0x90A0E0, 0x90ACE0, 0][i] })
    }

    /// 0x80A2 .. 0x81FE: print the glyph word string at the ROM address `string` as the prompt of player `i` (0xC9A).
    fn select_join_prompt(&mut self, i: usize, string: u32) {
        if let Some(a1) = self.select_join_prompt_at(i) {
            let s = self.assets.frontend.text.slice_from(string);
            self.text_words(s, a1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use std::path::Path;
    use std::sync::OnceLock;

    /// The asset pack of the user's ROM (`../../../dino.zip`), `None` when it is not there (the tests skip).
    fn rom() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// A game in the main pass of the select with `n` players that have joined and chose nothing yet.
    fn choosing(a: &'static AssetPack, n: usize) -> Game {
        let mut g = Game::new(a);
        g.flow.mode = 4;
        g.flow.sub = 2;
        g.flow.started = (1 << n) - 1;
        let s = &mut g.flow.front.select;
        s.seconds = 10;
        s.frames = 60;
        for i in 0..n {
            s.rec[i] = Rec { who: i as u8, state: 2, pos: i as u8, ..Default::default() };
            s.taken |= 1 << i;
        }
        g
    }

    /// One frame of the select: the buttons that go down this frame.
    fn press(g: &mut Game, input: [u8; 3]) {
        g.flow.front.prev_input = g.w.input;
        g.w.input = input;
        g.w.sounds.clear();
        g.select_main();
    }

    /// The pans of the cursor sounds (table 0xB24E) and the hero ids of the positions (table 0xB24A): Jack, Hannah, Mustapha, Mess.
    #[test]
    fn the_positions_have_pans_and_hero_ids() {
        let Some(a) = rom() else { return };
        let g = Game::new(a);
        assert_eq!([0, 1, 2, 3].map(|p| g.select_pan(p)), [0x00, 0x0B, 0x15, 0x20]);
        let t = &a.frontend.text;
        assert_eq!([0, 1, 2, 3].map(|p| t.u8(rom::POS_TO_ID, p)), [0, 2, 1, 3], "positions to hero ids: Jack 0, Hannah 2, Mustapha 1, Mess 3");
    }

    /// The cursors never meet: with three players pressing left in the same frame every cursor goes to the next free position (the
    /// positions rotate, as in `flow_sel-3p-all-left`), and a long run of presses never puts two cursors on one position.
    #[test]
    fn two_cursors_never_hold_the_same_hero() {
        let Some(a) = rom() else { return };
        let mut g = choosing(a, 3);
        press(&mut g, [2, 2, 2]); // all three press left
        let pos = g.flow.front.select.rec.map(|r| r.pos);
        assert_eq!(pos, [3, 0, 1], "the first goes to the free Mess, the others take the places that were left");
        assert_eq!(g.w.sounds.iter().filter(|&&c| c == 0x32).count(), 3);
        // a run of presses (a press needs a frame without the button before it)
        let mut seed = 12345u32;
        for _ in 0..400 {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let pad = |k: u32| match (seed >> (4 * k)) % 5 {
                0 => 1u8,
                1 => 2,
                _ => 0,
            };
            press(&mut g, [pad(0), pad(1), pad(2)]);
            let s = &g.flow.front.select;
            let mut seen = [false; 4];
            for r in s.rec {
                assert!(!seen[r.pos as usize], "two cursors on {}: {:?}", r.pos, s.rec.map(|r| r.pos));
                seen[r.pos as usize] = true;
            }
            let bits: u8 = s.rec.iter().fold(0, |m, r| m | 1 << r.pos);
            assert_eq!(s.taken, bits, "the taken mask is the set of positions");
        }
    }

    /// A pick (attack, jump or start) is ignored while the lock counter of the record runs (10 frames after the player joined), then it
    /// takes the hero of the position: sound 0x33, the hero id and class word, the object present, the state 4; moves are not locked.
    #[test]
    fn a_pick_waits_for_the_lock() {
        let Some(a) = rom() else { return };
        let mut g = choosing(a, 1);
        g.flow.front.select.rec[0].pos = 1; // Hannah
        g.flow.front.select.taken = 2;
        g.flow.front.select.rec[0].pending = 3;
        press(&mut g, [0x10, 0, 0]);
        assert_eq!(g.flow.front.select.rec[0].state, 2, "locked");
        assert!(!g.w.sounds.contains(&0x33));
        press(&mut g, [0, 0, 0]);
        g.flow.front.select.rec[0].pending = 0;
        press(&mut g, [0x20, 0, 0]); // jump picks too
        let r = g.flow.front.select.rec[0];
        assert_eq!(r.state, 4);
        assert_eq!(g.w.sounds, [0x33]);
        let b = &g.players[0].b;
        assert_eq!((b.id, b.k20, b.live), (2, 8, 1), "Hannah is hero 2");
        // a start press picks too
        let mut g = choosing(a, 1);
        g.w.system = 8 | 1;
        g.flow.front.prev_system = 8;
        g.select_main();
        assert_eq!(g.flow.front.select.rec[0].state, 4);
    }

    /// The timer: the number is printed when the 60 frame counter runs out and goes down by one; at 0 the three input bytes get an attack
    /// press (so every player who has not chosen picks) and the timer stays at 0.
    #[test]
    fn the_timer_runs_out_with_an_attack_press() {
        let Some(a) = rom() else { return };
        let mut g = choosing(a, 2);
        g.flow.front.select.seconds = 1;
        g.flow.front.select.frames = 2;
        press(&mut g, [0; 3]);
        assert_eq!((g.flow.front.select.seconds, g.flow.front.select.frames), (1, 1));
        press(&mut g, [0; 3]); // the counter runs out: prints "01", seconds 0
        assert_eq!((g.flow.front.select.seconds, g.flow.front.select.frames), (0, 60));
        g.flow.front.select.frames = 1;
        g.w.input = [0; 3];
        g.flow.front.prev_input = [0; 3];
        g.w.sounds.clear();
        g.select_main(); // prints "00", seconds would be -1: inputs 0x10 for the players' steps in this very frame
        assert_eq!(g.flow.front.select.seconds, 0);
        assert_eq!(g.w.input, [0x10; 3]);
        assert_eq!(g.flow.front.select.rec[0].state, 4, "player 1 picked");
        assert_eq!(g.flow.front.select.rec[1].state, 4, "player 2 picked");
        assert_eq!(g.w.sounds.iter().filter(|&&c| c == 0x33).count(), 2);
    }

    /// The 2 player layout (never measured, from the code): the first step of a record also makes the class 19 picture of the hero, which
    /// plays the animation 0 of the table of the position and the animation 1 when the hero is picked.
    #[test]
    fn the_2_player_layout_has_hero_pictures() {
        let Some(a) = rom() else { return };
        let mut g = choosing(a, 2);
        g.flow.b4cf = 1;
        g.flow.front.select.rec[0].state = 0;
        g.flow.front.select.rec[1].state = 0;
        g.flow.front.select.taken = 0;
        g.select_main();
        let classes: Vec<u16> = g.props.slots.iter().filter(|p| p.b.live != 0).map(|p| p.b.k20 / 4).collect();
        assert_eq!(classes.iter().filter(|&&c| c == 19).count(), 2, "two pictures: {classes:?}");
        assert_eq!(classes.iter().filter(|&&c| c == 68).count(), 2, "and two tags");
        g.select_main();
        let pic = g.props.slots.iter().find(|p| p.b.k20 / 4 == 19 && p.b.var26 == 4).unwrap();
        assert_eq!((pic.b.xi(), pic.b.yi()), (0x160, 0x10), "the second picture is on the right");
        assert_eq!(pic.b.anim_table, a.frontend.text.u32(0x7B85E, 4), "the table of the position 1");
        g.flow.front.select.rec[1].state = 4;
        g.select_main();
        let pic = g.props.slots.iter().find(|p| p.b.k20 / 4 == 19 && p.b.var26 == 4).unwrap();
        assert_eq!(pic.b.mode, 4, "picked");
    }
}
