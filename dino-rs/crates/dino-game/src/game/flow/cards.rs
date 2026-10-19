//! The map panel of the link scenes (M2-25): the framed picture of the map with the name plate of the place, drawn into scroll1 and slid in from the
//! right, and the blinking marker of the next place. The same routines are used by the stage clear tally (`tally.rs`, states 4 and 6) and by the intro card
//! of the next episode (`flow.rs`, M2-26). The panel is a picture of tile cells in the first page of scroll1 (0x908010) that the slide copies, column by
//! column, to the columns behind the right edge while scroll1 moves left; the colours are the palettes 1 to 13 of scroll1 (0x914400 + 0x20 * n), loaded by
//! stage from the ROM palettes 0xBC252 + 0x20 * n; the marker is a change of some of these palettes.
//!
//! All tables are by the stage G.$4D9 (the stage that has just been cleared): `FRONT_TABLES`.
//!
//! The stage intro card (M2-26, M2-27): the intro task 0xB9BE of the stages 1 to 7 slides the panel in from the right (`intro_slide_in`), types the title of
//! the episode and blinks the marker (`intro_title`, `intro_blink`), slides the panel out (`intro_slide_out`) and hands over to the heroes' script
//! (G.$4E2, G.$4E3; the walk in and the HUD slide in are in `flow.rs`).
//!
//! The second half of the file is the story panel of the game mode 8 (M2-28, 0xD898): a black screen with a small picture, two or three lines of text
//! and the music of the story, between the clear tally of an episode and the start of the next (`flow_story_mode`).

use crate::game::flow::IntroAfter;
use crate::game::vram::PaletteRam;
use crate::game::Game;

/// The ROM tables.
mod rom {
    /// The 32 palettes of the map (16 words each).
    pub const PALETTES: u32 = 0xBC252;
    /// 8 pointers to the marker lists: (palette slot, palette, palette) words, ended by a negative word.
    pub const MARKERS: u32 = 0xCACE;
    /// 8 pointers to the 13 palette numbers of the stage.
    pub const PALETTE_LISTS: u32 = 0xCD6C;
    /// The tile codes (380 words) and the attributes (380 words) of the frame and the picture.
    pub const CODES: u32 = 0xCEC4;
    pub const ATTRS: u32 = 0xD1BC;
    /// The 19 columns of 6 cells (codes, then attributes) of the stages 6 and 7.
    pub const EXTRA_CODES: u32 = 0xD4DC;
    pub const EXTRA_ATTRS: u32 = 0xD5C0;
    /// The 12 columns of 3 cells of the stage 7.
    pub const LAST_CODES: u32 = 0xD6CE;
    pub const LAST_ATTRS: u32 = 0xD716;
    /// 8 pointers to the 12 tile codes of the name plate.
    pub const PLATES: u32 = 0xD780;
}

impl Game {
    /// 0xCD1A: the palettes 1 to 13 of scroll1 from the ROM palettes the stage's list names; the whole bank then gets the full brightness (mask 2, 0x8932),
    /// and the mask stays 2.
    pub(crate) fn flow_card_palettes(&mut self) {
        let stage = self.w.stage as u32;
        let t = &self.assets.frontend.text;
        let list = t.u32(rom::PALETTE_LISTS, 4 * stage);
        for n in 1..=13u32 {
            let index = t.u16(list, 2 * (n - 1)) as u32;
            let words: Vec<u16> = (0..16).map(|k| t.u16(rom::PALETTES + 32 * index, 2 * k)).collect();
            self.pal.copy(PaletteRam::SCROLL1 + 16 * n as usize, &words);
        }
        self.fade.mask = 2;
        self.brightness_full_current(); // jsr 0x8932
    }

    /// 0xCB7A: the palette `slot` of scroll1 is the ROM palette `index` with the brightness F.
    fn flow_card_palette(&mut self, slot: u16, index: u16) {
        let t = &self.assets.frontend.text;
        let words: Vec<u16> = (0..16).map(|k| t.u16(rom::PALETTES + 32 * index as u32, 2 * k) | 0xF000).collect();
        self.pal.copy(PaletteRam::SCROLL1 + 16 * slot as usize, &words);
    }

    /// 0xCAA8: the marker of the next place: every (slot, palette, palette) of the stage's list gets the first palette (`blink` 0) or the second (1).
    pub(crate) fn flow_card_marker(&mut self, blink: bool) {
        let stage = self.w.stage as u32;
        let t = &self.assets.frontend.text;
        let mut a = t.u32(rom::MARKERS, 4 * stage);
        let mut set = Vec::new();
        loop {
            let d0 = t.u16(a, 0);
            if d0 & 0x8000 != 0 {
                break;
            }
            let d1 = if blink { t.u16(a, 4) } else { t.u16(a, 2) };
            set.push((d0, d1));
            a += 6;
        }
        for (slot, index) in set {
            self.flow_card_palette(slot, index);
        }
    }

    /// One tile cell of a panel: code word, attribute word at `a`, from the two tables at the offsets `k` words.
    fn flow_card_cell(&mut self, a: u32, codes: u32, attrs: u32, k: u32) {
        let t = &self.assets.frontend.text;
        let (code, attr) = (t.u16(codes, 2 * k), t.u16(attrs, 2 * k));
        self.hud_put(a, code, attr);
    }

    /// A block of `columns` columns of `rows` cells at `a0` (column by column, a cell is a code and an attribute from the two tables, `k` words in);
    /// returns the next word of the tables and the address of the next column.
    fn flow_card_block(&mut self, mut a0: u32, tables: (u32, u32), mut k: u32, columns: u32, rows: u32) -> (u32, u32) {
        for _ in 0..columns {
            let mut a = a0;
            for _ in 0..rows {
                self.flow_card_cell(a, tables.0, tables.1, k);
                k += 1;
                a = a.wrapping_add(4);
            }
            a0 = a0.wrapping_add(0x80);
        }
        (k, a0)
    }

    /// 0xCE5C: the panel at `a0` (the frame and the picture in three blocks, 10 x 13, 8 x 15 and 10 x 13 cells: the second starts two cells above the
    /// first block's next column, the third where the second ended), and for the stages 6 and 7 the extra blocks (0xD4B4, 0xD6A4).
    pub(crate) fn flow_card_panel(&mut self, a0: u32) {
        let t = (rom::CODES, rom::ATTRS);
        let (k, a) = self.flow_card_block(a0, t, 0, 10, 13);
        let (k, a) = self.flow_card_block(a.wrapping_sub(8), t, k, 8, 15);
        self.flow_card_block(a.wrapping_add(8), t, k, 10, 13);
        let stage = self.w.stage;
        if stage >= 6 {
            self.flow_card_block(a0.wrapping_add(0x498), (rom::EXTRA_CODES, rom::EXTRA_ATTRS), 0, 19, 6); // 0xD4B4
        }
        if stage == 7 {
            self.flow_card_block(a0.wrapping_add(0x4A4), (rom::LAST_CODES, rom::LAST_ATTRS), 0, 12, 3); // 0xD6A4
        }
    }

    /// 0xD75E: the name plate of the place, 12 tile codes in a row (the codes only) at the panel + 0x42C.
    pub(crate) fn flow_card_plate(&mut self, panel: u32) {
        let t = &self.assets.frontend.text;
        let list = t.u32(rom::PLATES, 4 * self.w.stage as u32);
        let mut a = panel.wrapping_add(0x42C);
        for k in 0..12 {
            let code = self.assets.frontend.text.u16(list, 2 * k);
            self.hud_word(a, code);
            a = a.wrapping_add(0x80);
        }
    }
}

// ------------------------------------------------------------------------------------------------------------------ the intro card

impl Game {
    /// 0xC526 after its first sleep (state 0): state 2, and for the stages 1 to 7 the card is drawn: the marker time 0x78 and period 0x14, the slide
    /// from 0x90A028 to 0x909808, the palettes of the stage, the panel at 0x90A030 and its name plate. (Stage 0 has no card: the camera Y goes to 0.)
    pub(super) fn intro_state0(&mut self) {
        let it = &mut self.flow.intro;
        it.state = it.state.wrapping_add(2);
        if self.w.stage == 0 {
            self.cam.cam1_y = 0;
            return;
        }
        it.w4 = 0x78;
        it.b2 = 0x14;
        it.src = 0x90A028;
        it.dst = 0x909808;
        it.w6 = 0;
        it.w8 = 0xC;
        self.flow_card_palettes(); // bsr 0xCD1A
        self.flow_card_panel(0x90A030); // bsr 0xCE5C
        self.flow_card_plate(0x90A030); // bra 0xD75E
    }

    /// 0xC642 (state 2, sub 0): the columns of the card (23 cells each, moved: the source gets blank cells) go to the page at the right as far as `+8`
    /// goes, then `+8` and scroll1 X grow by 12 until X is 0x130; the title comes after one more sleep.
    pub(super) fn intro_slide_in(&mut self) {
        loop {
            let it = &mut self.flow.intro;
            if (it.w8 as i16) < (it.w6 as i16) {
                break; // blt
            }
            let (src, dst) = (it.src, it.dst);
            for c in 0..23u32 {
                let (s, d) = (Self::flow_cell(src.wrapping_add(4 * c)), Self::flow_cell(dst.wrapping_add(4 * c)));
                self.vram.scroll1[d] = self.vram.scroll1[s];
                self.vram.scroll1[s].code = 0x20; // move.l d0,(a0)+ with d0 = 0x00200000
                self.vram.scroll1[s].attr = 0;
            }
            let it = &mut self.flow.intro;
            it.w6 = it.w6.wrapping_add(8);
            it.src = src.wrapping_add(0x80);
            it.dst = (dst.wrapping_add(0x80) & 0x909FFF) | 0x908000;
        }
        let it = &mut self.flow.intro;
        it.w8 = it.w8.wrapping_add(0xC);
        self.cam.cam1_x = self.cam.cam1_x.wrapping_add(0xC);
        if self.cam.cam1_x >= 0x130 {
            // 0xC6A8: sub 2, a sleep (jsr 0xA76), then the title
            self.flow.intro.sub = self.flow.intro.sub.wrapping_add(2);
            self.flow.intro.after = IntroAfter::Title;
        }
    }

    /// 0xC6AC after the sleep: the typewriter 0x7EC20 gets the title of the episode (the pointer table 0xC6D2: 8 for the Japan set, 8 for the other sets).
    pub(super) fn intro_title(&mut self) {
        let k = self.w.stage as u32 * 4 + if self.flow.region.0 != 0 { 0x20 } else { 0 };
        let text = {
            let t = &self.assets.frontend.text;
            t.slice_from(t.u32(0xC6D2, k)).to_vec()
        };
        self.flow_typer_spawn_pool_b(&text, 1);
    }

    /// 0xCA20 (state 2, sub 2): the marker of the next place blinks (the same code as the tally's 0xC5F6); when the time is over (`+4` negative at the end
    /// of a period) the marker is lit, the slide out is prepared (source: the page at the current X, 0x100 above; `+6` = X), state 4, a sleep of 60 frames.
    pub(super) fn intro_blink(&mut self) {
        let it = &mut self.flow.intro;
        it.w4 = it.w4.wrapping_sub(1);
        if it.b2 == 0x14 {
            it.b3 = 1;
            self.flow_card_marker(true); // bsr 0xCAA8
        }
        let it = &mut self.flow.intro;
        if it.b2 == 8 {
            it.b3 = 0;
            self.flow_card_marker(false);
        }
        let it = &mut self.flow.intro;
        it.b2 = it.b2.wrapping_sub(1);
        if it.b2 != 0 {
            return;
        }
        it.b2 = 0x14;
        if (it.w4 as i16) >= 0 {
            return;
        }
        it.b3 = 1;
        self.flow_card_marker(true);
        let x = self.cam.cam1_x;
        let it = &mut self.flow.intro;
        it.src = 0x908000u32.wrapping_add(((x << 4) & 0x1F80) as u32).wrapping_sub(0x100);
        it.w6 = x;
        // 0xCA86
        it.state = it.state.wrapping_add(2);
        it.sub = 0;
        it.after = IntroAfter::Wait60; // jsr 0xA82
        self.flow.intro.sleep = 60;
    }

    /// 0xCA94 after the 60 frame sleep: scroll1 X + 12, G.$4D5 = 1 (the heroes are held).
    pub(super) fn intro_after_wait(&mut self) {
        self.cam.cam1_x = self.cam.cam1_x.wrapping_add(0xC);
        self.spawn.g.b4d5 = 1;
    }

    /// 0xCBBE (state 4, sub 0): the card slides out to the left: the columns it has left behind are blanked (32 cells each) as far as the screen edge,
    /// X + 12 a frame; at 0x260 the scroll1 palettes of the stage are loaded again (mask 2, full brightness), scroll1 is cleared, the places of the HUD
    /// slide in are set (`+6`, `+8`, `+$A`, `+$E`) and G.$4E2 goes to 0.
    pub(super) fn intro_slide_out(&mut self) {
        loop {
            let it = &mut self.flow.intro;
            let d0 = it.w6.wrapping_add(8);
            if (d0 as i16) > (self.cam.cam1_x as i16) {
                break; // bgt
            }
            let a = it.src;
            for c in 0..32u32 {
                let cell = &mut self.vram.scroll1[Self::flow_cell(a.wrapping_add(4 * c))];
                cell.code = 0x20;
                cell.attr = 0;
            }
            let it = &mut self.flow.intro;
            it.w6 = d0;
            it.src = a.wrapping_add(0x80) & 0x909FFF;
        }
        self.cam.cam1_x = self.cam.cam1_x.wrapping_add(0xC);
        if self.cam.cam1_x < 0x260 {
            return; // bcs
        }
        self.cam.cam1_x = self.cam.cam1_x.wrapping_add(0xC);
        let d0 = self.cam.cam1_x.wrapping_add(0x180);
        let it = &mut self.flow.intro;
        it.src = 0x90A000;
        it.dst = 0x908000u32.wrapping_add(((d0 << 4) & 0x1F80) as u32);
        it.w6 = d0.wrapping_add(8);
        it.w8 = d0.wrapping_add(8).wrapping_add(0xC);
        let words = self.assets.stages.scroll1_palettes.clone();
        self.pal.copy(PaletteRam::SCROLL1, &words); // jsr 0x8A8E
        self.fade.mask = 2;
        self.brightness_full_current(); // jsr 0x8932
        self.flow_clear_scroll1(); // jsr 0xB52
        // The last two instructions (sub + 2, G.$4E2 = 0) come after the clear: in the recordings of Episodes 4 and 5 (ten items on the floor,
        // pool E) they are in the next record, in the others (0 to 4 items, 4 enemies at most) in the record of the step (`link-ep3` .. `ep8`).
        // The frame is estimated busy from the items alive, as the tally's panel is (`tally.rs`).
        if self.items.slots.iter().filter(|i| i.b.live != 0).count() >= SLIDE_OUT_BUSY_ITEMS {
            self.flow.intro.after = IntroAfter::SlideOutEnd;
            return;
        }
        let it = &mut self.flow.intro;
        it.sub = it.sub.wrapping_add(2);
        self.flow.b4e2 = 0;
    }
}

/// Items alive (pool E) from which the frame of the card's last slide out step is estimated to run over the vertical blank.
pub(crate) const SLIDE_OUT_BUSY_ITEMS: usize = 8;

// ------------------------------------------------------------------------------------------------------------------ the story panels

/// The parts of the mode 8 routine (the places where it sleeps: `jsr 0xA76` and the longer sleeps of the original).
mod part {
    /// The first pass: the palettes of the stage and the clear 0xAE0 start.
    pub const START: u8 = 0;
    /// Two frames later (0xAE0 has the CPU through them): 0xD8B4 and the entry of the stage up to its first sleep.
    pub const SETUP: u8 = 1;
    /// The wait loop 0xDFCC: a sleep, the attack check of the heroes, the counter G.-$719A.
    pub const WAIT: u8 = 2;
    /// Episode 3: 241 sleeps with scroll2 X + 0x10 each.
    pub const SCROLL: u8 = 3;
    /// Episode 5, the second picture: the loop of the two pictures that alternate (0xDC54).
    pub const PICTURES: u8 = 4;
    /// Episode 5, after the loop: one sleep between clearing scroll3 and scroll2.
    pub const CLEAR2: u8 = 5;
    /// Episode 8: 61 steps of the palette play (0xE0E4: 5 palettes, a sleep each).
    pub const PALETTES: u8 = 6;
    /// After the clear 0xAE0 at the end of the entry (its code goes on 2 frames later), then the stage start 0xAC4E.
    pub const CLEARED: u8 = 7;
    /// A sleep, then the clear 0xAE0 at the end of the entry (Episode 8).
    pub const CLEAR: u8 = 8;
}

/// The mode 8 routine's own variables (the ones the original keeps in registers and in G.-$719A).
#[derive(Clone, Copy, Debug, Default)]
pub struct Story {
    pub part: u8,
    /// G.-$719A (0xFF0E66): the frames of the wait loop; also the pass counter of the loops.
    pub counter: u16,
    /// The step of the Episode 8 palette play (d1 / d2 of 0xDF60 / 0xE0F2).
    pub step: u16,
    pub sub: u8,
    /// The frame of the stage start after Episode 5's panel (`story_exit_ep5`).
    pub exit_step: u8,
}

impl Game {
    /// 0xD898: the story panel of the game mode 8, called by the game task. The routine sleeps inside itself (the game task goes on in it
    /// through `Flow::wait` and `Flow::resume`).
    pub(super) fn flow_story_mode(&mut self) {
        if self.flow.select_exit != 0 {
            if self.w.stage == 4 {
                return self.story_exit_ep5();
            }
            return self.select_exit(); // the phases of 0xAC4E after the panel
        }
        match self.flow.story.part {
            part::START => self.story_start(),
            part::SETUP => self.story_setup(),
            part::WAIT => self.story_wait(),
            part::SCROLL => self.story_scroll(),
            part::PICTURES => self.story_pictures(),
            part::CLEAR2 => self.story_clear2(),
            part::PALETTES => self.story_palettes(),
            part::CLEAR => self.story_clear_end(),
            _ => self.story_cleared(),
        }
    }

    /// The routine sleeps `frames` frames and goes on in `part`.
    fn story_sleep(&mut self, frames: u8, part: u8) {
        self.flow.story.part = part;
        self.flow.wait = frames;
        self.flow.resume = true;
    }

    /// 0xD898: the palettes of the stage (0x8A44, 0x8932), then 0xAE0 (the CPU is busy with it for the next two frames).
    fn story_start(&mut self) {
        self.flow.story = Story::default();
        self.load_all_palettes(); // 0x8A44
        self.brightness_full_current(); // 0x8932
        self.flow_attract_clear(); // jsr 0xAE0
        self.story_sleep(2, part::SETUP);
    }

    /// 0xD8B4: the palette set 2, full brightness, then the entry of the stage (table 0xD8D8; stage 0 has none).
    fn story_setup(&mut self) {
        self.flow_scene_palettes(2); // moveq #2,d0; jsr 0x89E6
        self.brightness_full(); // jsr 0x892C
        match self.w.stage {
            1 => self.story_ep2(),
            2 => self.story_ep3(),
            3 => self.story_ep4(),
            4 => self.story_ep5(),
            5 => self.story_ep6(),
            6 => self.story_ep7(),
            7 => self.story_ep8(),
            _ => self.story_end(),
        }
    }

    /// 0xE0AA: the id of the playing hero: the first present player (P1, else P2, else P3), 0 when none.
    fn story_playing_hero(&self) -> u16 {
        (0..3).find(|&p| self.players[p].b.live == 1).map_or(0, |p| self.players[p].b.id as u16)
    }

    /// 0xE034: the portrait of the "caller": the lowest hero id that no present player has, with 1 and 2 swapped for the portrait table.
    fn story_caller(&self) -> u16 {
        let mut d0 = 0u16;
        while (0..3).any(|p| self.players[p].b.live == 1 && self.players[p].b.id as u16 == d0) {
            d0 += 1;
        }
        match d0 & 3 {
            1 => 2,
            2 => 1,
            d => d,
        }
    }

    /// 0x84736 as the story panels call it: the portrait palette word it keeps in +$A2 of the caller's object lands in G.$6A32 - 4 * (heroes - 1) here (the
    /// register a6 of the game task is G.$6990 - 4 * G.$4EF: the slot 19 + 2 * G.$4EF of the obstacle list, which the story panel does not use).
    fn story_portrait(&mut self, a0: u32, portrait: u16, text: i16) {
        self.text_portrait(a0, portrait, text);
        if portrait != 0x63 {
            let k = 19 + 2 * self.w.players_minus_one as usize;
            if let Some(slot) = self.terrain.buf.get_mut(k) {
                *slot = self.assets.frontend.text.u16(0x8461A, 2 * portrait as u32);
            }
        }
    }

    /// The tall text at ROM address `a` (0x84F72).
    fn story_text(&mut self, a: u32) {
        let t = &self.assets.frontend.text;
        let s = t.slice_from(a);
        self.text_tall(s);
    }

    /// 0x98BAE with the lists at `codes` and `attrs`: a block of `width` x `height` cells at `a0` in scroll3.
    fn story_block3(&mut self, a0: u32, codes: u32, attrs: u32, width: u16, height: u16) {
        let t = &self.assets.frontend.text;
        let (c, a) = (t.slice_from(codes), t.slice_from(attrs));
        self.flow_block3(a0, c, a, width, height);
    }

    /// 0x99992: a block of `width` x `height` cells of scroll2 from a list of codes (0x5800 is added), with the attribute 0xE.
    fn story_block2(&mut self, a0: u32, codes: u32, width: u32, height: u32) {
        let list: Vec<u16> = {
            let t = &self.assets.frontend.text;
            (0..width * height).map(|k| t.u16(codes, 2 * k)).collect()
        };
        for row in 0..height {
            for col in 0..width {
                let at = a0 + col * 0x40 + row * 4;
                let cell = &mut self.vram.scroll2[((at.wrapping_sub(0x90C000) / 4) & 0xFFF) as usize];
                cell.code = list[(row * width + col) as usize].wrapping_add(0x5800);
                cell.attr = 0xE;
            }
        }
    }

    /// The attack check of the wait loops (0xDFD2 .. 0xE02C): a present hero (G.$3274 = 1 and so on) whose input is exactly the attack button and has
    /// changed since the frame before sets the counter to 1 (jump does nothing).
    fn story_skip_check(&mut self) {
        for p in 0..3 {
            let (now, before) = (self.w.input[p], self.flow.front.prev_input[p]);
            if self.players[p].b.live == 1 && now != before && now == 0x10 {
                self.flow.story.counter = 1;
            }
        }
    }

    /// The end of the entry's wait: sleeps `frames`, then the wait loop 0xDFCC (the sleep comes first) with the counter G.-$719A = `count`.
    fn story_begin_wait(&mut self, count: u16) {
        self.flow.story.counter = count;
        self.story_sleep(1, part::WAIT);
    }

    /// 0xDFCC after its sleep: the attack check, the counter; at 0 the entry goes on after its wait.
    fn story_wait(&mut self) {
        self.story_skip_check();
        self.flow.story.counter = self.flow.story.counter.wrapping_sub(1);
        if self.flow.story.counter != 0 {
            return self.story_sleep(1, part::WAIT);
        }
        match self.w.stage {
            1 => {
                // 0xD92E: the anti tamper roll: 1 in 16 checks the ROM (the sum is right, nothing happens)
                let _ = self.rand();
                self.story_end()
            }
            3 => {
                // 0xDB56: 1 in 8 (0x5DD30, the same check)
                let _ = self.rand();
                self.story_clear_end()
            }
            4 => self.story_ep5_second(),
            _ => self.story_clear_end(),
        }
    }

    /// The end of an entry that clears the screen (0xAE0 inside it): the code after it runs 2 frames later (the CPU is busy), then 0xAC4E.
    fn story_clear_end(&mut self) {
        self.flow_attract_clear(); // jsr 0xAE0
        self.story_sleep(2, part::CLEARED);
    }

    /// The code after the 0xAE0 at the end of an entry: the camera values the entry changed (0xAC4E clears the cameras again at once, so only
    /// the layer control stays), then the end.
    fn story_cleared(&mut self) {
        match self.w.stage {
            2 => {
                self.cam.s2.cam_x = 0; // 0xDA32
                self.cam.s3.cam_y = self.cam.s3.cam_y.wrapping_add(0x10);
                self.cam.s3.cam_x = self.cam.s3.cam_x.wrapping_sub(0x10);
                self.w.layer_ctrl = 0x12D6;
            }
            3 => {
                self.cam.s3.cam_y = self.cam.s3.cam_y.wrapping_sub(0x10); // 0xDB84
                self.cam.s3.cam_x = self.cam.s3.cam_x.wrapping_sub(0x10);
            }
            6 => self.cam.s3.cam_y = 0, // 0xDEDA
            _ => {}
        }
        self.story_end();
    }

    /// The end of the routine: `jmp 0xAC4E`, the stage start (the phases 1 to 4 come in the next frames, `select_exit`).
    fn story_end(&mut self) {
        self.flow.story.part = part::START;
        self.flow.story.exit_step = 0;
        self.flow.select_exit = 1;
        self.flow_stage_start_staged(0);
        self.flow.wait = 1;
        self.flow.resume = true;
    }

    /// The stage start 0xAC4E after Episode 5's panel: its second clear (0xAE0) is cut differently by the video frame boundaries than after the other panels:
    /// the phases 1 and 2 (the camera, the palettes) come in the frame after the phase 0, the scroll controllers are reset one frame later, and the phases 3 and 4 come in one frame.
    fn story_exit_ep5(&mut self) {
        let k = self.flow.story.exit_step;
        self.flow.story.exit_step += 1;
        match k {
            0 => {
                self.flow_stage_start_staged(1);
                self.flow_stage_start_staged(20);
                self.flow.wait = 1;
                self.flow.resume = true;
            }
            1 => {
                self.flow_stage_start_staged(21);
                self.flow.wait = 1;
                self.flow.resume = true;
            }
            _ => {
                self.flow_stage_start_staged(3);
                self.flow.select_exit = 4;
                self.flow.story.exit_step = 0;
                self.select_exit(); // phase 4 and the first level frame
            }
        }
    }

    // ------------------------------------------------------------------------------------------------ the seven entries

    /// 0xD8E8 (Episode 2): the Butcher in the northern woods.
    fn story_ep2(&mut self) {
        self.sound(0x13); // moveq #0x13,d0; jsr 0xA63C
        self.cam.s3.cam_y = 0;
        self.cam.s3.cam_x = 0;
        self.story_block3(0x913884, 0xD94E, 0xD966, 4, 3);
        self.story_text(0xE42A);
        self.story_begin_wait(0x12C);
    }

    /// 0xD97E (Episode 3): a friend says the heroes look busy; scroll2 page 0x1B three times, scrolling.
    fn story_ep3(&mut self) {
        self.sound(0x14);
        self.w.layer_ctrl = 0x1396;
        self.cam.s3.cam_y = self.cam.s3.cam_y.wrapping_sub(0x10);
        self.cam.s3.cam_x = self.cam.s3.cam_x.wrapping_add(0x10);
        self.story_block3(0x91380C, 0xDA4C, 0xDA80, 13, 2);
        self.flow_scene_palettes(2); // 0x89E6, 0x892C again
        self.brightness_full();
        for a in [0x90F000, 0x90F400, 0x90F800] {
            self.flow_page2(a, 0x1B); // 0x204CA
        }
        self.story_text(0xE500);
        let base = PaletteRam::SCROLL3 + 0x1F0; // 0x914FE0: sixteen black words
        for k in 0..16 {
            self.pal.words[base + k] = 0xF000;
        }
        self.flow.story.counter = 0xF1; // d1 = 0xF0: dbra
        self.story_sleep(2, part::SCROLL);
    }

    /// 0xDA0C: sleep, scroll2 X + 0x10 (back to 0 from 0x180), 241 times.
    fn story_scroll(&mut self) {
        let x = self.cam.s2.cam_x.wrapping_add(0x10);
        self.cam.s2.cam_x = if (x as i16) < 0x180 { x } else { 0 };
        self.flow.story.counter -= 1;
        if self.flow.story.counter != 0 {
            return self.story_sleep(1, part::SCROLL);
        }
        self.story_clear_end();
    }

    /// 0xDAB4 (Episode 4): Mustapha (or another hero, by the party) says somebody will pay for the garage.
    fn story_ep4(&mut self) {
        self.sound(0x15);
        self.cam.s3.cam_y = 0;
        self.cam.s3.cam_x = 0;
        self.cam.s3.cam_y = self.cam.s3.cam_y.wrapping_sub(0x30);
        self.cam.s3.cam_x = self.cam.s3.cam_x.wrapping_add(0x10);
        self.story_block3(0x9138B0, 0xDB92, 0xDBA2, 4, 2);
        self.story_text(0xE5AE);
        let caller = self.story_caller();
        let id = if caller == 1 { 0x29 } else { 0x28 };
        self.story_portrait(0x90A108, caller, id);
        let (caller, hero) = (self.story_caller(), self.story_playing_hero());
        self.story_portrait(0x90A108, caller, (hero + 0x2A) as i16);
        self.story_begin_wait(0x12C);
    }

    /// 0xDBB2 (Episode 5): a radio call from a village under attack ("HELLO THIS IS <hero>"), then Jack says something happened to him.
    fn story_ep5(&mut self) {
        self.sound(0x16);
        self.cam.s3.cam_y = 0;
        self.cam.s3.cam_x = 0;
        self.story_block3(0x913824, 0xDD8C, 0xDDAC, 4, 2);
        self.story_block3(0x9138F0, 0xDD9C, 0xDDBC, 4, 2);
        self.story_text(0xE606);
        let hero = self.story_playing_hero();
        self.story_portrait(0x90A150, 0x63, (hero + 0x2F) as i16);
        self.story_begin_wait(0x12C);
    }

    /// 0xDC2E: the second half of Episode 5: scroll1 cleared, the second text, 64 rounds of two pictures.
    fn story_ep5_second(&mut self) {
        self.flow_clear_scroll1(); // jsr 0xB52
        self.story_text(0xE8B0);
        self.flow_scene_palettes(2);
        self.brightness_full();
        self.flow.story.counter = 0x40;
        self.flow.story.sub = 0;
        self.story_pictures();
        self.flow.wait = 2; // the clear of scroll1, the text and the first picture keep the CPU through the next frame (the first sleep wakes 2 frames later)
    }

    /// 0xDC54: the two pictures of scroll2 in turn, a sleep after each; after the second the attack check and the counter.
    fn story_pictures(&mut self) {
        if self.flow.story.sub == 0 {
            self.story_block2(0x90F3A0, 0xDD0C, 8, 4);
            self.flow.story.sub = 1;
            return self.story_sleep(1, part::PICTURES);
        }
        if self.flow.story.sub == 1 {
            self.story_block2(0x90F3A0, 0xDD4C, 8, 4);
            self.flow.story.sub = 2;
            return self.story_sleep(1, part::PICTURES);
        }
        self.flow.story.sub = 0;
        self.story_skip_check();
        self.flow.story.counter = self.flow.story.counter.wrapping_sub(1);
        if self.flow.story.counter != 0 {
            return self.story_pictures();
        }
        // 0xDCF2: scroll3 cleared (0xB88), a sleep, scroll2 cleared (0xB6C), then 0xAE0
        self.vram.clear3();
        self.story_sleep(1, part::CLEAR2);
    }

    fn story_clear2(&mut self) {
        self.vram.clear2(); // 0xB6C keeps the CPU through the rest of the frame: 0xAE0 starts in the next one
        self.story_sleep(1, part::CLEAR);
    }

    /// 0xDDCC (Episode 6): a hero says the doctor is mad (no portrait when Hannah plays alone: the original draws none).
    fn story_ep6(&mut self) {
        self.sound(0x13);
        self.cam.s3.cam_y = 0;
        self.cam.s3.cam_x = 0;
        self.story_block3(0x913884, 0xDE44, 0xDE58, 5, 2);
        let hero = self.story_playing_hero();
        if hero != 2 {
            let portrait = if hero == 1 { 2 } else { hero };
            self.story_portrait(0x90A150, portrait, 0x2E);
        }
        self.story_text(0xE6B0);
        self.story_begin_wait(0x168);
    }

    /// 0xDE6C (Episode 7): a hero recognises Dr. Fessenden's equipment; the portrait of the caller.
    fn story_ep7(&mut self) {
        self.sound(0x14);
        self.cam.s3.cam_y = 0;
        self.cam.s3.cam_x = 0;
        self.story_block3(0x91388C, 0xDEE2, 0xDF00, 5, 3);
        self.story_text(0xE74E);
        let caller = self.story_caller();
        self.story_portrait(0x90A108, caller, if caller == 1 { 0x34 } else { 0x33 });
        self.story_begin_wait(0x168);
    }

    /// 0xDF1E (Episode 8): the villain invites the heroes; a palette play of 61 x 5 frames.
    fn story_ep8(&mut self) {
        self.sound(0x15);
        self.cam.s3.cam_y = 0;
        self.cam.s3.cam_x = 0;
        self.story_block3(0x913864, 0xDF78, 0xDFA2, 7, 3);
        self.story_text(0xE7DC);
        self.brightness_full_current(); // jsr 0x8932
        self.flow.story.counter = 0x3C + 1; // d1 = 0x3C: dbra
        self.flow.story.step = 0;
        self.story_palettes();
    }

    /// 0xE0E4: the palettes 0x914EC0 and 0x914EE0 of scroll3 get the next colours of two lists (the brightness nibble stays), a sleep; five times a call.
    fn story_palettes(&mut self) {
        let k = self.flow.story.step as u32;
        let t = &self.assets.frontend.text;
        let (o1, o2) = (t.u16(0xE152, 2 * k), t.u16(0xE15C, 2 * k));
        let words: [Vec<u16>; 2] = [
            (0..16).map(|i| t.u16(0xC2252 + o1 as u32, 2 * i)).collect(),
            (0..16).map(|i| t.u16(0xC2252 + o2 as u32, 2 * i)).collect(),
        ];
        for (n, w) in words.iter().enumerate() {
            let base = PaletteRam::SCROLL3 + 0x160 + 16 * n; // 0x914EC0, 0x914EE0
            for (i, &c) in w.iter().enumerate() {
                let old = self.pal.words[base + i];
                self.pal.words[base + i] = (old & 0xF000) | c;
            }
        }
        self.flow.story.step += 1;
        if self.flow.story.step == 5 {
            self.flow.story.step = 0;
            self.flow.story.counter -= 1;
            if self.flow.story.counter == 0 {
                return self.story_sleep(1, part::CLEAR); // the sleep of the last step, then 0xAE0
            }
        }
        self.story_sleep(1, part::PALETTES);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game() -> Game {
        Game::new(Box::leak(Box::new(AssetPack::default())))
    }

    fn with_heroes(g: &mut Game, ids: &[u8]) {
        for (p, &id) in ids.iter().enumerate() {
            g.players[p].b.live = 1;
            g.players[p].b.id = id;
        }
    }

    /// 0xE0AA: the playing hero is the first present player; 0xE034: the caller is the lowest hero id nobody plays, with 1 and 2 swapped.
    #[test]
    fn the_playing_hero_and_the_caller() {
        let mut g = game();
        with_heroes(&mut g, &[0]);
        assert_eq!((g.story_playing_hero(), g.story_caller()), (0, 2), "Jack alone: Mustapha (id 1) calls, portrait 2");
        let mut g = game();
        with_heroes(&mut g, &[3]);
        assert_eq!((g.story_playing_hero(), g.story_caller()), (3, 0), "Mess alone: Jack calls");
        let mut g = game();
        with_heroes(&mut g, &[1]);
        assert_eq!(g.story_caller(), 0, "Mustapha alone: Jack calls");
        let mut g = game();
        with_heroes(&mut g, &[2]);
        assert_eq!(g.story_caller(), 0, "Hannah alone: Jack calls");
        let mut g = game();
        with_heroes(&mut g, &[0, 2]);
        assert_eq!(g.story_caller(), 2, "Jack and Hannah: Mustapha calls (portrait 2)");
        let mut g = game();
        with_heroes(&mut g, &[0, 1, 3]);
        assert_eq!(g.story_caller(), 1, "Jack, Mustapha and Mess: Hannah calls (portrait 1)");
        // a hero that is not live == 1 does not count (the checks of the original read the byte for 1)
        let mut g = game();
        with_heroes(&mut g, &[0]);
        g.players[0].b.live = 2;
        assert_eq!((g.story_playing_hero(), g.story_caller()), (0, 0));
    }

    /// 0xDFD2: only a present hero whose input is exactly the attack button and has changed since the frame before ends the wait (jump does not).
    #[test]
    fn the_attack_button_skips_the_wait() {
        let mut g = game();
        with_heroes(&mut g, &[0, 1]);
        g.flow.story.counter = 200;
        g.w.input = [0x20, 0, 0]; // jump
        g.story_skip_check();
        assert_eq!(g.flow.story.counter, 200);
        g.w.input = [0x10, 0, 0];
        g.flow.front.prev_input = [0x10, 0, 0]; // held, not new
        g.story_skip_check();
        assert_eq!(g.flow.story.counter, 200);
        g.w.input = [0x11, 0, 0]; // with a direction
        g.flow.front.prev_input = [0, 0, 0];
        g.story_skip_check();
        assert_eq!(g.flow.story.counter, 200);
        g.w.input = [0, 0x10, 0]; // the second hero
        g.story_skip_check();
        assert_eq!(g.flow.story.counter, 1);
        g.flow.story.counter = 200;
        g.w.input = [0, 0, 0x10]; // the third hero is not present
        g.flow.front.prev_input = [0, 0, 0];
        g.story_skip_check();
        assert_eq!(g.flow.story.counter, 200);
    }

    /// 0xC642: the card moves, 23 cells of each column, to the page at the right as far as `+8` goes, 12 pixels a frame; at 0x130 the title is next.
    #[test]
    fn the_card_slides_in() {
        let mut g = game();
        g.flow.intro.src = 0x90A028;
        g.flow.intro.dst = 0x909808;
        g.flow.intro.w6 = 0;
        g.flow.intro.w8 = 0xC;
        for k in 0..23u32 {
            let c = &mut g.vram.scroll1[Game::flow_cell(0x90A028 + 4 * k)];
            c.code = 0x100 + k as u16;
            c.attr = 7;
        }
        g.intro_slide_in();
        // two columns (w6 0 and 8 are <= 0xC): the first moved, the second is the next column of the source
        let moved = g.vram.scroll1[Game::flow_cell(0x909808)];
        assert_eq!((moved.code, moved.attr), (0x100, 7));
        let left = g.vram.scroll1[Game::flow_cell(0x90A028)];
        assert_eq!((left.code, left.attr), (0x20, 0), "the source is blank (0x00200000)");
        assert_eq!((g.flow.intro.w6, g.flow.intro.w8, g.cam.cam1_x), (0x10, 0x18, 0xC));
        assert_eq!(g.flow.intro.src, 0x90A028 + 0x100);
        assert_eq!(g.flow.intro.dst, 0x909808 + 0x100);
        for _ in 0..25 {
            g.intro_slide_in();
        }
        assert_eq!(g.cam.cam1_x, 0x138, "26 frames of 12 pixels: 0x138 is the first value of 0x130 or more");
        assert_eq!((g.flow.intro.sub, g.flow.intro.after), (2, IntroAfter::Title));
    }

    /// 0xCBBE: the card slides out: the columns left behind are blanked; at X 0x260 the HUD slide in is prepared and G.$4E2 is cleared.
    #[test]
    fn the_card_slides_out() {
        let mut a = AssetPack::default();
        a.stages.scroll1_palettes = vec![0x0123; 512];
        let mut g = Game::new(Box::leak(Box::new(a)));
        g.flow.intro.state = 4;
        g.flow.intro.src = 0x90A100;
        g.flow.intro.w6 = 0x138; // the columns before X are blanked already
        g.cam.cam1_x = 0x140;
        g.flow.b4e2 = 1;
        let c = &mut g.vram.scroll1[Game::flow_cell(0x90A100)];
        c.code = 0x55;
        c.attr = 3;
        g.intro_slide_out();
        // w6 + 8 = 0x140 is not greater than X: one column (32 cells) at 0x90A100 is blank, the next goes on
        let c = g.vram.scroll1[Game::flow_cell(0x90A100)];
        assert_eq!((c.code, c.attr), (0x20, 0));
        assert_eq!(g.flow.intro.w6, 0x140);
        assert_eq!(g.cam.cam1_x, 0x14C);
        assert_eq!((g.flow.intro.sub, g.flow.b4e2), (0, 1), "not yet at 0x260");
        g.cam.cam1_x = 0x25C;
        g.flow.intro.w6 = 0x300;
        g.intro_slide_out();
        // X + 12 = 0x268 >= 0x260: X + 12 again, the places of the HUD slide in from X + 0x180, scroll1 cleared, sub 2, G.$4E2 = 0
        assert_eq!(g.cam.cam1_x, 0x274);
        let d0 = 0x274 + 0x180;
        assert_eq!((g.flow.intro.w6, g.flow.intro.w8), (d0 + 8, d0 + 8 + 0xC));
        assert_eq!((g.flow.intro.src, g.flow.intro.dst), (0x90A000, 0x908000 + ((d0 << 4) & 0x1F80) as u32));
        assert_eq!((g.flow.intro.sub, g.flow.b4e2, g.fade.mask), (2, 0, 2));
        assert_eq!(g.pal.words[PaletteRam::SCROLL1], 0xF123, "the scroll1 palettes are loaded again with the full brightness");
    }
}
