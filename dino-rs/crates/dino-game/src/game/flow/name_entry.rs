//! Name entry for a qualifying score (join state 0xA, the high score table).
//!
//! Original routines: 0x154EE qualification, 0x15AE2 setup, 0x15BE4 per frame, helpers
//! 0x15C7A (same frame flag), 0x15CB2 (inputs), 0x15D1A (rank blink), 0x15D3C / 0x15D74 (rank
//! text), 0x15DC2 / 0x15DE4 (blank rows), 0x15E3A (keys), 0x15EE8 (enter a glyph), 0x15F86
//! (direction repeat), 0x15FB4 (letters), 0x160FC (cursor), 0x1616C (profanity filter),
//! 0x161DA (insert), 0x16230 (store), 0x16260 (sort), and 0x74A4 (state 0xA).
//!
//! Block layout (32 bytes per player at G.-$711C + 0x20 * player), with the offsets of the
//! original in the field docs. Shared globals: G.$7013 (same frame flag), G.$701A (frame
//! stamp), G.$7017 (rank blink counter); G.$5E4 is `Flow::b5e4`.

use crate::game::Game;

/// 0x1A4: idle frames before the entry ends (420).
const IDLE_FRAMES: u16 = 0x1A4;
/// 0x14: seconds for the whole entry (20).
const SECONDS: u16 = 0x14;
/// 0x3C: frames per second in the entry (60).
const FRAMES_PER_SECOND: u16 = 0x3C;
/// Glyph index of RUB (28) and END (29).
const GLYPH_RUB: u8 = 0x1C;
const GLYPH_END: u8 = 0x1D;

/// One player's entry block (G.-$711C + 0x20 * player).
#[derive(Clone, Copy, Debug, Default)]
pub struct Block {
    /// +0 seconds left (word, 20 at the start).
    pub secs: u16,
    /// +2 idle frames left (word, 420).
    pub idle: u16,
    /// +4 rank (1 to 10), copied from the player's rank byte +$10A.
    pub rank: u8,
    /// +5 letters entered.
    pub count: u8,
    /// +6 to +9 the name (3 letters and a space, ASCII).
    pub name: [u8; 4],
    /// +$A frames per second (word, 60).
    pub fps: u16,
    /// +$C current and +$D previous input (G.$16+p and G.$1B+p).
    pub cur: u8,
    pub prev: u8,
    /// +$E key repeat counter.
    pub rep: u8,
    /// +$F glyph under the cursor (0 to 29).
    pub glyph: u8,
    /// +$10 to +$13 glyph index of each letter (the picture of the name).
    pub shown: [u8; 4],
    /// +$14 blink timer (word): cursor blink, then the finish blink.
    pub blink: u16,
    /// +$16 key lock after an enter (frames).
    pub lock: u8,
    /// +$17 done flag: the entry is over (time, idle, END).
    pub done: u8,
    /// +$18 finish stage (0 before the table write, 2 after).
    pub finish: u8,
    /// +$19 finish blink count (4).
    pub flashes: u8,
}

impl Block {
    /// The byte at the offset `o` of the block (the fields have their original offsets; past +$19 the bytes are not modeled).
    fn peek(&self, o: u16) -> u8 {
        match o {
            0 => (self.secs >> 8) as u8,
            1 => self.secs as u8,
            2 => (self.idle >> 8) as u8,
            3 => self.idle as u8,
            4 => self.rank,
            5 => self.count,
            6..=9 => self.name[o as usize - 6],
            0xA => (self.fps >> 8) as u8,
            0xB => self.fps as u8,
            0xC => self.cur,
            0xD => self.prev,
            0xE => self.rep,
            0xF => self.glyph,
            0x10..=0x13 => self.shown[o as usize - 0x10],
            0x14 => (self.blink >> 8) as u8,
            0x15 => self.blink as u8,
            0x16 => self.lock,
            0x17 => self.done,
            0x18 => self.finish,
            0x19 => self.flashes,
            _ => 0,
        }
    }

    /// A byte store at the offset `o` of the block (see `peek`).
    fn poke(&mut self, o: u16, v: u8) {
        match o {
            0 => self.secs = (self.secs & 0xFF) | (v as u16) << 8,
            1 => self.secs = (self.secs & 0xFF00) | v as u16,
            2 => self.idle = (self.idle & 0xFF) | (v as u16) << 8,
            3 => self.idle = (self.idle & 0xFF00) | v as u16,
            4 => self.rank = v,
            5 => self.count = v,
            6..=9 => self.name[o as usize - 6] = v,
            0xA => self.fps = (self.fps & 0xFF) | (v as u16) << 8,
            0xB => self.fps = (self.fps & 0xFF00) | v as u16,
            0xC => self.cur = v,
            0xD => self.prev = v,
            0xE => self.rep = v,
            0xF => self.glyph = v,
            0x10..=0x13 => self.shown[o as usize - 0x10] = v,
            0x14 => self.blink = (self.blink & 0xFF) | (v as u16) << 8,
            0x15 => self.blink = (self.blink & 0xFF00) | v as u16,
            0x16 => self.lock = v,
            0x17 => self.done = v,
            0x18 => self.finish = v,
            0x19 => self.flashes = v,
            _ => {}
        }
    }
}

/// The name entry globals of the flow (all blocks and the shared blink state).
#[derive(Clone, Copy, Debug, Default)]
pub struct NameEntry {
    pub blocks: [Block; 3],
    /// G.$701A: the frame stamp of the last call (`name_same_frame`).
    pub seen_frame: u16,
    /// G.$7013: this frame was already run for another player.
    pub same_frame: bool,
    /// G.$7017: rank text blink counter (20 frames, blank for the last 5).
    pub blink: u8,
}

impl Game {
    /// 0x154EE and 0x7396: qualify a finished player. Rank 10 or better and a score above the
    /// tenth entry (unsigned compare, as the original's `cmp.l`) start the entry (state 0xA,
    /// G.$5E4 = 1, setup 0x15AE2). Otherwise the state is 0xC and G.$5E4 = 0.
    pub(super) fn name_qualify(&mut self, i: usize) {
        let p = &self.players[i];
        let tenth = self.hud.hiscores[9].score;
        if p.b10a <= 10 && tenth < p.score {
            self.flow.join[i].state = 0xA;
            self.flow.b5e4 = 1;
            self.name_setup(i);
        } else {
            self.flow.join[i].state = 0xC;
            self.flow.b5e4 = 0;
        }
    }

    /// 0x15AE2: set up the entry block of player `i` (the rank comes from the player).
    /// The original does not clear the letter count or the key bytes, so neither does this.
    pub fn name_setup(&mut self, i: usize) {
        let rank = self.players[i].b10a;
        let frame = self.w.frame;
        let b = &mut self.flow.name.blocks[i];
        b.idle = IDLE_FRAMES;
        b.secs = SECONDS;
        b.fps = FRAMES_PER_SECOND;
        b.rank = rank;
        b.name = *b"    ";
        b.shown = [0x20; 4];
        b.glyph = 0;
        b.done = 0;
        b.finish = 0;
        b.blink = frame;
    }

    /// 0x74A4: the join task's state 0xA. Parked while the ending runs (G.$507).
    pub(super) fn name_state(&mut self, i: usize) {
        if self.flow.mode == 0x1C && self.flow.b507 != 0 {
            return;
        }
        if self.name_frame(i) {
            self.flow.join[i].state = 0xC; // 0x74C2
            self.flow.b5e4 = 0;
            self.flow.started &= !(1 << i); // bclr.b d0,$4CD(a5)
            if self.flow.mode != 0x10 {
                self.flow_join_clear(i, 0x7FB6, 0x7FC2); // bra 0x7F86 (0x7FCA in the mode 0x10: not ported)
            }
        }
    }

    /// 0x15C7A: G.$7013 = 1 when the frame stamp did not change since the last call.
    fn name_same_frame(&mut self) {
        self.flow.name.same_frame = false;
        let frame = self.w.frame;
        if frame == self.flow.name.seen_frame {
            self.flow.name.same_frame = true;
        } else {
            self.flow.name.seen_frame = frame;
        }
    }

    /// 0x15BE4: one frame of the entry of player `i`. Returns true when the entry is over
    /// (the finish blink ended, state 0xC).
    fn name_frame(&mut self, i: usize) -> bool {
        self.name_same_frame(); // 0x15C7A
        // 0x15CB2: the current and previous input bytes (G.$16+p and G.$1B+p).
        let cur = self.w.input[i];
        let prev = self.flow.front.prev_input[i];
        {
            let b = &mut self.flow.name.blocks[i];
            b.cur = cur;
            b.prev = prev;
        }
        if self.flow.name.blocks[i].done != 0 {
            return self.name_finish(i); // 0x15C32
        }
        // Seconds: 0x15BF8..0x15C0A.
        let b = &mut self.flow.name.blocks[i];
        b.fps = b.fps.wrapping_sub(1);
        if b.fps == 0 {
            b.fps = FRAMES_PER_SECOND;
            b.secs = b.secs.wrapping_sub(1);
            if b.secs == 0 {
                b.done = 1;
                return false;
            }
        }
        self.name_blink(i); // 0x15D1A
        self.name_keys(i); // 0x15E3A
        self.name_draw_letters(i); // 0x15FB4
        self.name_draw_cursor(i); // 0x160FC
        // 0x15C22: idle timer.
        let b = &mut self.flow.name.blocks[i];
        b.idle = b.idle.wrapping_sub(1);
        if (b.idle as i16) < 0 {
            b.done = 1;
        }
        false
    }

    /// 0x15C32: the finish. Filter the name, insert it, sort, then blink the rank line
    /// 4 times for 30 frames each. Returns true at the end (0x15C76).
    fn name_finish(&mut self, i: usize) -> bool {
        if self.flow.name.blocks[i].finish == 0 {
            self.flow.name.blocks[i].finish += 2; // addq.b #2, $18
            self.name_profanity(i); // 0x1616C
            self.name_insert(i); // 0x161DA
            let b = &mut self.flow.name.blocks[i];
            b.blink = 0x1E; // move.w #$1E, $14
            b.flashes = 4; // move.b #4, $19
        }
        let b = &mut self.flow.name.blocks[i];
        let old = b.blink;
        b.blink = old.wrapping_sub(1); // subq.w #1, $14
        if old == 0 {
            // Borrow: the flash count goes down (0x15C56).
            let (f, borrow) = b.flashes.overflowing_sub(1);
            b.flashes = f;
            if borrow {
                return true; // 0x15C76: moveq #1
            }
            b.blink = 0x1E;
        }
        if (self.flow.name.blocks[i].blink as i16) < 5 {
            self.name_blank_rows(i, 16); // 0x15DE4
        } else {
            self.name_rank_text(i); // 0x15D3C
            self.name_draw_letters(i); // 0x15FB4
        }
        false
    }

    /// 0x15D1A: the rank text blink. Shown while the counter is above 5, blank otherwise.
    fn name_blink(&mut self, i: usize) {
        if !self.flow.name.same_frame {
            let (v, borrow) = self.flow.name.blink.overflowing_sub(1);
            self.flow.name.blink = v;
            if borrow {
                self.flow.name.blink = 0x14;
                return;
            }
        }
        if (self.flow.name.blink as i8) > 5 {
            self.name_rank_text(i); // 0x15D3C
        } else {
            self.name_blank_rows(i, 6); // 0x15DC2
        }
    }

    /// 0x15E3A: the keys of player `i`: move the cursor, enter a glyph on attack.
    fn name_keys(&mut self, i: usize) {
        {
            let b = &mut self.flow.name.blocks[i];
            b.lock = b.lock.saturating_sub(1); // subq.b #1, $16; clr if it borrowed
        }
        let mut d4 = self.flow.name.blocks[i].glyph;
        let dirs = self.name_dirs(i); // 0x15F86
        if dirs != 0 {
            let count = self.flow.name.blocks[i].count;
            if dirs & 2 != 0 {
                // Left (0x15E56).
                self.flow.name.blocks[i].idle = IDLE_FRAMES;
                d4 = d4.wrapping_sub(1);
                if count == 0 && d4 == GLYPH_RUB {
                    d4 = d4.wrapping_sub(1);
                }
                if count >= 3 && (d4 as i8) < GLYPH_RUB as i8 {
                    d4 = 0xFF;
                }
                if (d4 as i8) < 0 {
                    d4 = GLYPH_END;
                }
            } else if dirs & 1 != 0 {
                // Right (0x15E8E).
                self.flow.name.blocks[i].idle = IDLE_FRAMES;
                d4 = d4.wrapping_add(1);
                if count == 0 && d4 == GLYPH_RUB {
                    d4 = d4.wrapping_add(1);
                }
                if d4 > GLYPH_END {
                    d4 = if count < 3 { 0 } else { GLYPH_RUB };
                }
            }
        }
        self.flow.name.blocks[i].glyph = d4;
        // 0x15EBE: attack edge (bit 4) with the key lock free.
        let b = &self.flow.name.blocks[i];
        if b.lock != 0 {
            return;
        }
        if (b.cur & !b.prev) & 0x10 != 0 {
            let b = &mut self.flow.name.blocks[i];
            b.lock = 4;
            b.idle = IDLE_FRAMES;
            self.name_enter(i); // 0x15EE8
        }
    }

    /// 0x15F86: the direction bits to act on this frame (0 when none). Bit 1 is left, bit 0
    /// right. A new direction acts at once and then waits 24 frames; a held one repeats every 4.
    fn name_dirs(&mut self, i: usize) -> u8 {
        let b = &mut self.flow.name.blocks[i];
        let cur = b.cur;
        if cur & 0xF == 0 || (cur & 0xF) ^ b.prev != 0 {
            b.rep = 0x18;
            return cur;
        }
        b.rep = b.rep.wrapping_sub(1);
        if b.rep != 0 {
            return 0;
        }
        b.rep = 4;
        cur
    }

    /// 0x15EE8: the attack key enters the glyph under the cursor.
    fn name_enter(&mut self, i: usize) {
        let b = &mut self.flow.name.blocks[i];
        let d2 = b.glyph;
        if d2 == GLYPH_END {
            b.done = 1; // 0x15F52
            return;
        }
        if d2 == GLYPH_RUB {
            // 0x15F5A: erase the last letter, the cursor goes to its glyph.
            if b.count != 0 {
                let c = b.count as u16;
                b.poke(0x10 + c, 0x20);
                b.poke(6 + c, 0x20);
                b.count -= 1;
                b.glyph = b.peek(0x10 + c - 1);
            }
            return;
        }
        let letter = self.assets.text.u8(0x15F34, d2 as u32);
        let b = &mut self.flow.name.blocks[i];
        let c = b.count;
        // 0x15F10 / 0x15F18: the letter at +6 + count and its glyph at +$10 + count. A fourth letter and more (the cursor can leave END and
        // the attack key still enters) write on into the block: +$A is the high byte of the frame count of a second, +$14 the blink timer.
        b.poke(6u16 + c as u16, letter);
        b.poke(0x10 + c as u16, d2);
        b.count = c.wrapping_add(1);
        b.glyph = if b.count >= 3 { GLYPH_END } else { 0 };
    }

    /// 0x15FB4: the four letter pictures (2 by 2 tiles each) from scroll1 0x400 below the block.
    fn name_draw_letters(&mut self, i: usize) {
        let mut a = self.name_base(i) + 0x400;
        for k in 0..4 {
            let g = self.flow.name.blocks[i].shown[k];
            self.name_glyph(a, g);
            a += 0x100;
        }
    }

    /// 0x160FC: the cursor, one letter slot after the last letter; it blinks every 6 frames.
    fn name_draw_cursor(&mut self, i: usize) {
        let base = self.name_base(i) + 0x400 + ((self.flow.name.blocks[i].count as u32) << 8);
        let frame = self.w.frame;
        let b = &mut self.flow.name.blocks[i];
        if (frame.wrapping_sub(b.blink) as i16) < 6 {
            let g = b.glyph;
            self.name_glyph(base, g);
        } else {
            b.blink = frame;
            self.name_blank(base);
        }
    }

    /// One glyph index as a 2 by 2 picture at scroll1 address `a` (0x15FB4 / 0x160FC).
    /// Words of the table: top left, bottom left, top right, bottom right.
    fn name_glyph(&mut self, a: u32, g: u8) {
        let t = &self.assets.text;
        let at = 8 * g as u32;
        let w = [t.u16(0x15FF4, at), t.u16(0x15FF4, at + 2), t.u16(0x15FF4, at + 4), t.u16(0x15FF4, at + 6)];
        self.name_cells2x2(a, w[0], w[1], w[2], w[3]);
    }

    /// The blank 2 by 2 picture (tile 0x20 everywhere).
    fn name_blank(&mut self, a: u32) {
        self.name_cells2x2(a, 0x20, 0x20, 0x20, 0x20);
    }

    /// Four cells: (col, row) = tl, (col, row+1) = bl, (col+1, row) = tr, (col+1, row+1) = br.
    /// A row is 4 bytes and a column 0x80 bytes in scroll1.
    fn name_cells2x2(&mut self, a: u32, tl: u16, bl: u16, tr: u16, br: u16) {
        self.hud_put(a, tl, 0);
        self.hud_put(a + 4, bl, 0);
        self.hud_put(a + 0x80, tr, 0);
        self.hud_put(a + 0x84, br, 0);
    }

    /// The scroll1 address of player `i`'s entry block (0x15CEA): table 0x90A010 .. by layout.
    fn name_base(&self, i: usize) -> u32 {
        if self.flow.b4cf != 0 {
            [0x0090_A010, 0x0090_AC10][i.min(1)]
        } else {
            [0x0090_A010, 0x0090_A810, 0x0090_B010][i]
        }
    }

    /// 0x15D3C and 0x15D74: the rank word (1ST .. 10TH) in three 2 by 2 pictures.
    fn name_rank_text(&mut self, i: usize) {
        let mut a = self.name_base(i);
        let rank = self.flow.name.blocks[i].rank;
        let idx = rank.wrapping_sub(1) as u32;
        let t = &self.assets.text;
        let word = t.u16(0x15D9E, 2 * idx.min(9));
        let suffix = idx.min(3);
        // The suffix pairs follow the 10 rank words (offset 0x14 from 0x15D9E).
        let s0 = t.u16(0x15D9E, 0x14 + 4 * suffix);
        let s1 = t.u16(0x15D9E, 0x14 + 4 * suffix + 2);
        self.name_tile2x2(a, word);
        a += 0x100;
        self.name_tile2x2(a, s0);
        a += 0x100;
        self.name_tile2x2(a, s1);
    }

    /// 0x15D74: a picture from one code: tl = c, bl = c + 0x10, tr = c + 1, br = c + 0x11.
    fn name_tile2x2(&mut self, a: u32, c: u16) {
        self.name_cells2x2(a, c, c.wrapping_add(0x10), c.wrapping_add(1), c.wrapping_add(0x11));
    }

    /// 0x15DC2 (6 columns) and 0x15DE4 (16 columns): blank rows of the block.
    fn name_blank_rows(&mut self, i: usize, cols: u32) {
        let mut a = self.name_base(i);
        for _ in 0..cols {
            self.hud_put(a, 0x20, 0);
            self.hud_put(a + 4, 0x20, 0);
            a += 0x80;
        }
    }

    /// 0x1616C: the profanity filter. A name equal to one of the five entries (3 letters) is
    /// replaced by the entry's replacement; the fourth byte becomes a space.
    fn name_profanity(&mut self, i: usize) {
        let t = &self.assets.text;
        let b = &mut self.flow.name.blocks[i];
        for k in 0..5u32 {
            let from: [u8; 3] = [t.u8(0x161B2, 4 * k), t.u8(0x161B2, 4 * k + 1), t.u8(0x161B2, 4 * k + 2)];
            if b.name[..3] == from {
                // The replacements follow the five names (offset 0x14 from 0x161B2).
                for c in 0..3u32 {
                    b.name[c as usize] = t.u8(0x161B2, 0x14 + 4 * k + c);
                }
                b.name[3] = 0x20;
                return;
            }
        }
    }

    /// 0x161DA and 0x16230: insert the entry at its rank (1 to 10) and write it. The table
    /// moves down by one from the rank (the last entry is lost).
    fn name_insert(&mut self, i: usize) {
        let rank = self.flow.name.blocks[i].rank;
        if rank == 0 || rank > 10 {
            return; // rank 1..10 always; the guard only keeps the index in range
        }
        let idx = rank.wrapping_sub(1) as usize;
        if idx != 49 {
            for k in (idx + 1..=49).rev() {
                let prev = self.hud.hiscores[k - 1];
                let e = &mut self.hud.hiscores[k];
                e.score = prev.score;
                e.name = prev.name;
                e.chr = prev.chr;
                e.stage = prev.stage;
                e.b0a = prev.b0a;
            }
        }
        // 0x16230: score, name, hero id, stage + 1 (+$A keeps its old value).
        let p = &self.players[i];
        let (score, chr) = (p.score, p.b.id);
        let stage = self.w.stage.wrapping_add(1);
        let name = self.flow.name.blocks[i].name;
        let e = &mut self.hud.hiscores[idx];
        e.score = score;
        e.name = name;
        e.chr = chr;
        e.stage = stage;
        self.name_sort();
    }

    /// 0x16260: selection sort of the 50 entries by score, highest first (signed compare;
    /// on a tie the earlier entry stays first).
    fn name_sort(&mut self) {
        let t = &mut self.hud.hiscores;
        for i in 0..49 {
            let mut best = i;
            for k in i + 1..50 {
                if (t[best].score as i32) < (t[k].score as i32) {
                    best = k;
                }
            }
            t.swap(i, best);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::TraceReader;
    use crate::game::hud::HiScore;
    use crate::game::flow::mame_check::listed_args;
    use dino_assets::AssetPack;
    use std::{path::Path, sync::OnceLock};

    fn assets() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let p = Path::new("../../../dino.zip");
            p.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(p).unwrap())))
        })
    }

    /// The block in the original's layout (G.-$711C + 0x20 * i).
    fn block_bytes(b: &Block) -> [u8; 0x20] {
        let mut o = [0u8; 0x20];
        o[0..2].copy_from_slice(&b.secs.to_be_bytes());
        o[2..4].copy_from_slice(&b.idle.to_be_bytes());
        o[4] = b.rank;
        o[5] = b.count;
        o[6..10].copy_from_slice(&b.name);
        o[0xA..0xC].copy_from_slice(&b.fps.to_be_bytes());
        o[0xC] = b.cur;
        o[0xD] = b.prev;
        o[0xE] = b.rep;
        o[0xF] = b.glyph;
        o[0x10..0x14].copy_from_slice(&b.shown);
        o[0x14..0x16].copy_from_slice(&b.blink.to_be_bytes());
        o[0x16] = b.lock;
        o[0x17] = b.done;
        o[0x18] = b.finish;
        o[0x19] = b.flashes;
        o
    }

    /// The high score table as the original lays it out (16 bytes per entry).
    fn table_bytes(h: &[HiScore]) -> Vec<u8> {
        let mut v = Vec::new();
        for e in h {
            v.extend_from_slice(&e.score.to_be_bytes());
            v.extend_from_slice(&e.name);
            v.push(e.chr);
            v.push(e.stage);
            v.push(e.b0a);
            v.extend_from_slice(&[0; 5]);
        }
        v
    }

    /// D4d A17 / M2-22: the name entry of the game over run (`flow_k-name-entry`) against MAME,
    /// frame by frame: join state, the entry block of player 1, the table, and the sound queue.
    /// G.$5E4 is compared on every frame.
    #[test]
    fn name_entry_a17_matches_mame() {
        let Some(a) = assets() else { return };
        let name = "flow_k-name-entry";
        let path = format!("../../../re/out/traces/{name}.mame.trc");
        if !Path::new(&path).exists() {
            return;
        }
        let mut rd = TraceReader::open(&path).unwrap();
        let mut game: Option<Game> = None;
        let mut checked = 0;
        let mut state_a = None;
        let mut state_c = None;
        let mut sounds: Vec<(i32, u16)> = Vec::new();
        let mut sound_pos = 0u16;
        while let Some(f) = rd.next() {
            let ram = &rd.region("ram").unwrap().data;
            let gfx = &rd.region("gfx").unwrap().data;
            let word = |o: usize| u16::from_be_bytes([ram[o], ram[o + 1]]);
            if let Some(g) = &mut game {
                g.w.system = ram[0x801A];
                g.w.coins = ram[0x8020];
                g.w.input.copy_from_slice(&ram[0x8016..0x8019]);
                g.tick();
                checked += 1;
                if g.flow.join[0].state == 0xA && state_a.is_none() {
                    state_a = Some(f);
                }
                if g.flow.join[0].state == 0 && state_a.is_some() && state_c.is_none() {
                    state_c = Some(f);
                }
                assert_eq!(g.flow.join[0].state, ram[0xB316], "{name} frame {f} P1 state");
                assert_eq!(g.flow.b5e4, ram[0x85E4], "{name} frame {f} G.$5E4");
                let mine = block_bytes(&g.flow.name.blocks[0]);
                assert_eq!(&mine[..], &ram[0x0EE4..0x0EE4 + 0x20], "{name} frame {f} P1 entry block");
                let t = table_bytes(&g.hud.hiscores);
                assert!(t == ram[0xF03E..0xF03E + 0x320], "{name} frame {f} table");
                let end = word(0x8034) & 0x3F0;
                let mut frame_sounds = Vec::new();
                let mut frame_args = Vec::new();
                let long = |o: usize| u32::from_be_bytes([ram[o], ram[o + 1], ram[o + 2], ram[o + 3]]);
                while sound_pos != end {
                    let e = 0x803A + sound_pos as usize;
                    frame_sounds.push(word(e));
                    frame_args.push((word(e), long(e + 2), long(e + 6)));
                    sound_pos = (sound_pos + 16) & 0x3F0;
                }
                // The ranking screen after the entry (ambient 0x70) is M1-07 timing: checked only up to 4300.
                if f < 4300 {
                    assert_eq!(g.w.sounds, frame_sounds, "{name} frame {f} sounds");
                    // M4-15: d1 and d2 of the listed codes (name entry music 0x1F) against the ring
                    // the continue music 0x1B (`0xEB1E`) takes d1 = 0x46 and d2 = 0x40 from the registers of the text routines (M4-16: the same in
                    // every continue of the recordings, `flow_continue_start`)
                    let port: Vec<_> = g.w.snd_reqs.iter().map(|r| (r.code, r.d1, r.d2)).collect();
                    assert_eq!(listed_args(&frame_args), listed_args(&port), "{name} frame {f} sound arguments");
                }
                sounds.extend(frame_sounds.iter().map(|&c| (f, c)));
            } else if ram[0x84D0] == 0x10 && ram[0x84D1] == 0 {
                let mut g = Game::new(a);
                g.trace_load(ram, gfx);
                sound_pos = word(0x8034) & 0x3F0;
                game = Some(g);
            }
        }
        assert!(checked > 0);
        eprintln!("{name}: equal for {checked} frames; state 0xA from frame {state_a:?}, back to 0 at {state_c:?}");
        eprintln!("{name}: MAME sounds (frame, code) {sounds:x?}");
    }

    /// The attack key enters the glyph under the cursor even when the cursor has left END after the third letter (the name is only
    /// 4 bytes long): the fourth letter and every one after it are stored on into the block (+$A is the high byte of the frames per
    /// second, +$10 on the glyph pictures), as the original's 0x15F10 / 0x15F18 do, and drawing the pictures never reads outside the ROM
    /// (the bot of `--autoplay` presses attack and right all the time).
    #[test]
    fn letters_past_the_third_are_stored_on_into_the_block() {
        let Some(a) = assets() else { return };
        let mut g = Game::new(a);
        g.flow.name.blocks[0].glyph = 3; // D
        for k in 0..40 {
            g.name_enter(0);
            g.flow.name.blocks[0].glyph = (k % 26) as u8;
            g.name_draw_letters(0);
        }
        let b = &g.flow.name.blocks[0];
        assert_eq!(b.count, 40);
        assert_eq!(b.peek(6), 0x44, "the first letter D");
        let mut c = Block::default();
        c.poke(0x13, 7);
        c.poke(0xA, 0x41);
        assert_eq!((c.shown[3], c.fps), (7, 0x4100));
        assert_eq!((c.peek(0x13), c.peek(0xA), c.peek(0x40)), (7, 0x41, 0), "past the block: not modeled");
    }
}
