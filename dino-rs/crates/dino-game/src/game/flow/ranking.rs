//! The ranking screen "BRAVES": the routine 0x1516C with its drawing code 0x15594, 0x15748, 0x15888, 0x1591C
//! (M1-07). Owner: flow / HUD port. Spec: `D4d-front-end-and-story.md` s3.5, s7, s13 A6.
//!
//! The attract task calls it (0x983D8, which first ends the blink task) in `st::RANK_START`; it runs inside the
//! task like the profile cards (`profiles.rs`): the script position of `front.task` is one of the `rst`
//! values, `task.count` is the wait counter G.$7018. The routine is a loop with a state byte G.$7012 (0 set-up,
//! 2 draw, 4 wait, 6 end; the set-up and the draw are one pass) and a pass is: the handler of the state, the
//! scroll2 camera + 8 and the scroll3 camera + 4 (the desert goes by), pool A 0x23420, pool B 0x23604, pool C
//! 0x237B2, the sprite list 0x13FA6, sleep 1; in state 6 the pass ends the routine (stop code 0xFF00) before the
//! objects.
//!
//! * Set-up (0x151C4): the pools are reset (0x49A8, 0x4A46, 0x4A18, 0x4C76, 0x4C48, 0x4CA4), the cloud manager
//!   (pool B class 89, 0x8C96C) is created, palette set 4 at full brightness, the stop code, scroll1 cleared.
//! * Draw (0x15210 .. 0x152F0): the label block 0x15508 (BRAVES, 1ST to 10TH), scroll2 page 0x16 four times,
//!   scroll2 Y 0x40, scroll3 page 0x7C eight times, the ten rows of the table G.$703E (0x15594), G.$7018 = 0x1A4,
//!   the Cadillac (pool B class 86, 0x152F6).
//! * Wait (0x15336): G.$7018 counts down; below 0 the state is 6.
//!
//! Row (0x15594): the line at the rows 6, 10, 12, 14, 16, 19, 21, 23, 25 and 27 (the steps of 0x153CA) holds, in
//! the font of 0x15748 / 0x15888 (letters and digits of 2 cells high, 2 cells wide; blank glyphs 2 x 2 blanks),
//! the three letters of the name from column 10, the score in 8 digits from column 20 with the leading zeros
//! blank, the face icon of the hero (3 x 2 cells, tables 0x156A8 and 0x156E8) at column 40, the stage digit at
//! column 43 and the two small marks (codes 0x2D and 0x52) at the columns 45 and 46 one row down.
//!
//! # CPU time
//!
//! The drawing is 2000 cell writes and the original needs more than the rest of a frame for it, so it is cut
//! by the video frame boundary at a place that depends on the CPU time before it. The port builds the writes in
//! the original's order (`ranking_ops`) and does the first `ranking_cut` words of them in the frame after the call
//! and the rest, the end of the draw and the first pass in the next one (the measured cuts are in the table
//! `CUTS`). The first run is one frame ahead of the later ones (its second pass comes in the frame of the first
//! one: 422 frames against 423), like the profile cards.

use super::attract::Stub;
use super::boot::st;
use crate::game::{Game, POOL_A, POOL_B, POOL_C, Pool};

/// Script positions of the slot 0x10 task inside the routine.
pub(super) mod rst {
    /// The first part of the draw (the frame after the call).
    pub const DRAW1: u16 = 80;
    /// The rest of the draw, the end of the handler and the first pass.
    pub const DRAW2: u16 = 81;
    /// 0x15336 and the rest of a pass.
    pub const PASS: u16 = 82;

    pub fn owns(state: u16) -> bool {
        (DRAW1..=PASS).contains(&state)
    }
}

/// The tables: row steps (0x153CA), the label block (0x15508), face pointers and palettes (0x156A8, 0x156E8),
/// the font (glyph codes by character 0x15768, widths 0x15828) and the cloud lists are in `FRONT_TABLES`.
const ROW_STEPS: u32 = 0x153CA;
const LABELS: u32 = 0x15508;
const FACE_POINTERS: u32 = 0x156A8;
const FACE_PALETTES: u32 = 0x156E8;
const FONT_CODES: u32 = 0x15768;
const FONT_WIDTHS: u32 = 0x15828;

/// Frames the wait state lasts (G.$7018 starts here and counts down to below 0, 0x152F0).
const WAIT_COUNT: u16 = 0x1A4;

/// One write of the draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Op {
    /// A cell of scroll1, scroll2 or scroll3 (layer 1, 2, 3): index, code, attribute.
    Cell(u8, u16, u16, u16),
    /// G.$748, the scroll2 camera Y.
    CamY(u16),
}

/// How many writes the first frame after the call does, counted in words (a cell is a code word and an attribute
/// word: the frame can end between them), by run of the ranking. Measured in `attract3.mame.trc`
/// (`ranking_cut_report` in `mame_check.rs`); the runs come after the profile cards, the story slides, the
/// profile cards, the demo and so on (`attract.rs`). Later runs, and the one after a game over, take the value
/// of the screen before: `CUT_AFTER_PROFILES`, `CUT_AFTER_SLIDES`, `CUT_AFTER_DEMO`.
const CUTS: [usize; 9] = [3958, 3054, 3786, 3958, 3786, 3051, 3786, 3957, 3786];
const CUT_AFTER_PROFILES: usize = 3786;
const CUT_AFTER_SLIDES: usize = 3051;
const CUT_AFTER_DEMO: usize = 3957;

impl Game {
    /// The ranking runs for the first time since power on.
    fn ranking_first_run(&self) -> bool {
        self.flow.front.rank_runs <= 1
    }

    /// The number of words (2 a cell) the frame after the call writes (see the module comment). The state byte of
    /// the attract task says which screen came before: 0 and 4 the profile cards, 2 the story slides, 6 the demo.
    fn ranking_cut(&self) -> usize {
        let run = self.flow.front.rank_runs as usize; // 1 for the first run
        match CUTS.get(run.wrapping_sub(1)) {
            Some(&k) => k,
            None => match self.flow.front.attract_state & 0x3E {
                2 => CUT_AFTER_SLIDES,
                6 => CUT_AFTER_DEMO,
                _ => CUT_AFTER_PROFILES,
            },
        }
    }

    /// 0x1516C up to the first sleep: the set-up (state 0) and the start of the draw (state 2): the pools, the
    /// cloud manager, the palettes, the stop code and the clear of scroll1; the writes follow in the next frame.
    pub(super) fn flow_ranking_start(&mut self) {
        self.flow.front.stub = Stub::Ranking;
        self.w.layer_ctrl = 0x12D6; // 0x1516C
        // 0x151C4: 0x49A8, 0x4A46, 0x4A18 (the lists), 0x4C76, 0x4C48, 0x4CA4 (pools B, A, C)
        self.draw_list.clear();
        self.sprites.splash_timer = 1;
        self.props = Pool::new(POOL_B);
        self.enemies = Pool::new(POOL_A);
        self.effects = Pool::new(POOL_C);
        if let Some(i) = self.props.alloc() {
            // 0x8C96C: jsr 0x4860, +$00 = 1, +$20 = 0x164
            let p = &mut self.props.slots[i];
            p.b.live = 1;
            p.b.k20 = 0x164;
        }
        // 0x151F2: palette set 4 (0x89E6), full brightness (0x892C), stop (0xA5D2), scroll1 cleared (0xB52)
        self.flow_scene_palettes(4);
        self.brightness_full();
        self.sound(0xFF00);
        self.flow_clear_scroll1();
        self.flow.front.task.wait(1, rst::DRAW1);
    }

    /// The script of the routine, one position per call (see `boot.rs`).
    pub(super) fn flow_ranking_script(&mut self) {
        match self.flow.front.task.state {
            rst::DRAW1 => {
                let ops = self.ranking_ops();
                let cut = self.ranking_cut().min(2 * ops.len());
                self.ranking_apply(&ops[..cut / 2]);
                if cut % 2 == 1 {
                    // the frame ends between the code word and the attribute word of a cell
                    if let Op::Cell(layer, i, code, _) = ops[cut / 2] {
                        self.ranking_cell(layer, i).code = code;
                    }
                }
                self.flow.front.task.wait(1, rst::DRAW2);
            }
            rst::DRAW2 => {
                let ops = self.ranking_ops();
                let cut = self.ranking_cut().min(2 * ops.len());
                self.ranking_apply(&ops[cut / 2..]);
                // 0x152F0: the wait counter, 0x152F6: the Cadillac
                self.flow.front.task.count = WAIT_COUNT;
                if let Some(i) = self.props.alloc() {
                    let p = &mut self.props.slots[i];
                    p.b.live = 1;
                    p.b.k20 = 0x158;
                    p.b.set_xi((self.cam.s2.cam_x as i16).wrapping_add(-0x80)); // addi.w #$FF80,$8(a0)
                    p.b.set_yi((self.cam.s2.cam_y as i16).wrapping_add(0x20));
                    p.b.set_zi(0); // G.$74C, the scroll2 camera depth (0 outside the stages)
                    p.b.dir = 0;
                    p.b.facing = 0;
                }
                self.flow_ranking_pass_end();
            }
            _ => {
                // 0x15336: state 4
                let (n, borrow) = self.flow.front.task.count.overflowing_sub(1);
                self.flow.front.task.count = n;
                if borrow {
                    // state 6: the camera steps, the stop code, the routine returns (`st::RANK_END`)
                    self.cam.s2.cam_x = self.cam.s2.cam_x.wrapping_add(8);
                    self.cam.s3.cam_x = self.cam.s3.cam_x.wrapping_add(4);
                    self.flow.front.task.state = st::RANK_END;
                } else {
                    self.flow_ranking_pass_end();
                }
            }
        }
    }

    /// 0x15184 .. 0x151BA: the camera steps, the object updates, the sprite list, the sleep. The first run
    /// does the second pass in the frame of the first one.
    fn flow_ranking_pass_end(&mut self) {
        self.cam.s2.cam_x = self.cam.s2.cam_x.wrapping_add(8); // addq.w #8,$744(a5)
        self.cam.s3.cam_x = self.cam.s3.cam_x.wrapping_add(4); // addq.w #4,$7E4(a5)
        self.enemies_update(); // 0x23420
        self.pool_b_update(); // 0x23604
        self.pool_c_update(); // 0x237B2
        self.build_sprite_list(); // 0x13FA6
        if self.ranking_first_run() && self.flow.front.task.state == rst::DRAW2 {
            self.flow.front.task.state = rst::PASS;
        } else {
            self.flow.front.task.wait(1, rst::PASS);
        }
    }

    /// A cell of one of the three maps.
    fn ranking_cell(&mut self, layer: u8, i: u16) -> &mut dino_assets::stage::Cell {
        let cells = match layer {
            1 => &mut self.vram.scroll1,
            2 => &mut self.vram.scroll2,
            _ => &mut self.vram.scroll3,
        };
        &mut cells[i as usize]
    }

    /// Does the writes of the draw.
    fn ranking_apply(&mut self, ops: &[Op]) {
        for op in ops {
            match *op {
                Op::Cell(layer, i, code, attr) => {
                    let c = self.ranking_cell(layer, i);
                    (c.code, c.attr) = (code, attr);
                }
                Op::CamY(y) => self.cam.s2.cam_y = y,
            }
        }
    }

    // ------------------------------------------------------------------ the writes of the draw

    /// All writes of the draw in the original's order (0x15210 .. 0x152EE).
    pub(super) fn ranking_ops(&self) -> Vec<Op> {
        let mut ops = Vec::with_capacity(2600);
        self.rk_label_block(&mut ops); // 0x15210
        for a0 in [0x90F000, 0x90F400, 0x90F800, 0x90FC00] {
            self.rk_page2(&mut ops, a0, 0x16); // 0x204CA
        }
        ops.push(Op::CamY(0x40)); // move.w #$40,$748(a5)
        for k in 0..8 {
            self.rk_page3(&mut ops, 0x913800 + 0x100 * k, 0x7C); // 0x20592
        }
        let text = &self.assets.frontend.text;
        let mut row = 6u16; // move.w #6,$7010(a5)
        for d3 in 0..10 {
            self.rk_row(&mut ops, d3, row);
            row += text.u8(ROW_STEPS, d3 as u32) as u16; // 0x152DC
        }
        ops
    }
}

impl Game {
    /// The scroll1 cell of an address (base 0x908000).
    fn rk_cell1(a: u32, code: u16, attr: u16) -> Op {
        Op::Cell(1, ((a.wrapping_sub(0x908000) / 4) & 0xFFF) as u16, code, attr)
    }

    /// 0x1591C: the address of the cell at column `col`, row `row` (rows 0 to 31 are the page 0x90A000, from 32 on
    /// the page 0x908000).
    fn rk_address(col: u16, row: u16) -> u32 {
        let (col, row) = ((col & 0x3F) as u32, (row & 0x3F) as u32);
        let r = if row & 0x20 != 0 { (row - 0x20) << 2 } else { (row << 2) + 0x2000 };
        0x908000 + (col << 7) + r
    }

    /// 0x15748: the glyph code of a character and the width of the glyph minus 1 (the d0 and d5 it returns).
    fn rk_char(&self, c: u16) -> (u16, u16) {
        let text = &self.assets.frontend.text;
        let idx = if c != 0 { c.wrapping_sub(0x20) } else { 0 } as u32;
        (text.u16(FONT_CODES, 2 * idx), text.u8(FONT_WIDTHS, idx) as u16)
    }

    /// 0x158E6: a blank glyph, 2 columns of 2 cells, attribute 0.
    fn rk_blank(ops: &mut Vec<Op>, a1: &mut u32) {
        for _ in 0..2 {
            ops.push(Self::rk_cell1(*a1, 0x20, 0));
            ops.push(Self::rk_cell1(*a1 + 4, 0x20, 0));
            *a1 += 0x80;
        }
    }

    /// 0x15888: draws a glyph at `a1` (which moves on): `d5 + 1` columns of 2 cells (the code and the code + 0x10
    /// below it, the next column the next code); a negative code is a blank glyph; 0x10B and 0x10C (the two
    /// marks that stand on the line) are one blank column, one with the code in the lower cell.
    fn rk_glyph(ops: &mut Vec<Op>, a1: &mut u32, mut code: u16, attr: u16, d5: u16) {
        for _ in 0..=d5 {
            if (code as i16) < 0 {
                return Self::rk_blank(ops, a1);
            }
            if code == 0x10B || code == 0x10C {
                ops.push(Self::rk_cell1(*a1, 0x20, attr)); // 0x158BA
                ops.push(Self::rk_cell1(*a1 + 4, code, attr));
                *a1 += 0x80;
                ops.push(Self::rk_cell1(*a1, 0x20, attr));
                ops.push(Self::rk_cell1(*a1 + 4, 0x20, attr));
                *a1 += 0x80;
                return;
            }
            ops.push(Self::rk_cell1(*a1, code, attr));
            ops.push(Self::rk_cell1(*a1 + 4, code.wrapping_add(0x10), attr));
            code = code.wrapping_add(1);
            *a1 += 0x80;
        }
    }

    /// 0x1555A: the label block: position blocks (column, row, attribute, characters), 0x2F starts the next
    /// block, 0x40 ends. Positions are relative to 0x90A000.
    fn rk_label_block(&self, ops: &mut Vec<Op>) {
        let block = self.assets.frontend.text.slice_from(LABELS);
        let mut i = 0;
        loop {
            let (col, row) = (block[i] as u32, block[i + 1] as u32);
            let mut a1 = 0x90A000 + ((col << 7) + (row << 2));
            let attr = block[i + 2] as u16;
            i += 3;
            loop {
                let c = block[i] as u16;
                i += 1;
                match c {
                    0x40 => return,
                    0x2F => break,
                    _ => {
                        let (code, d5) = self.rk_char(c);
                        Self::rk_glyph(ops, &mut a1, code, attr, d5);
                    }
                }
            }
        }
    }

    /// 0x204CA written cell by cell: scroll2 page `page` at `a0` (8 columns of 8 blocks of 2 x 2 cells).
    fn rk_page2(&self, ops: &mut Vec<Op>, a0: u32, page: usize) {
        let f = &self.assets.frontend;
        let ids = f.scroll2_page(page);
        let (mut a4, mut a) = (a0, a0);
        let mut k = 0;
        for _col in 0..8 {
            for _row in 0..8 {
                let block = &f.scroll2_blocks[ids[k] as usize];
                k += 1;
                for (i, c) in block.iter().enumerate() {
                    let at = a + (i as u32 / 2) * 0x40 + (i as u32 % 2) * 4;
                    ops.push(Op::Cell(2, ((at.wrapping_sub(0x90C000) / 4) & 0xFFF) as u16, c.code, c.attr));
                }
                a += 8;
            }
            a4 += 0x80;
            a = a4;
        }
    }

    /// 0x20592 written cell by cell: scroll3 page `page` at `a0` (8 columns of 8 cells).
    fn rk_page3(&self, ops: &mut Vec<Op>, a0: u32, page: usize) {
        for col in 0..8usize {
            for row in 0..8usize {
                let c = self.assets.stages.scroll3_screen_cells[page * 64 + col * 8 + row];
                let at = a0 + col as u32 * 0x20 + row as u32 * 4;
                ops.push(Op::Cell(3, ((at.wrapping_sub(0x910000) / 4) & 0xFFF) as u16, c.code, c.attr));
            }
        }
    }

    /// 0x15594: the line `d3` of the table at the row `row`: name, score, face, stage and the two marks.
    fn rk_row(&self, ops: &mut Vec<Op>, d3: usize, row: u16) {
        let text = &self.assets.frontend.text;
        let e = self.hud.hiscores[d3].clone();
        let mut col = 0xA_u16; // move.w #$A,$700E(a5)
        // the name: up to 3 letters, a negative byte ends it
        let mut a1 = Self::rk_address(col, row);
        for k in 0..3 {
            let c = e.name[k];
            if (c as i8) < 0 {
                break;
            }
            let (code, d5) = self.rk_char(c as u16);
            Self::rk_glyph(ops, &mut a1, code, 0, d5);
        }
        // the score: 8 digits of the BCD long from the top, the leading zeros blank
        col += 0xA;
        let mut a1 = Self::rk_address(col, row);
        let mut shown = false; // G.$7016
        for d2 in (0..8u32).rev() {
            let digit = ((e.score >> (d2 * 4)) & 0xF) as u16;
            if digit != 0 {
                shown = true;
            }
            if !shown {
                Self::rk_blank(ops, &mut a1); // 0x158E6
            } else {
                let (code, d5) = self.rk_char(digit + 0x30);
                Self::rk_glyph(ops, &mut a1, code, 0, d5);
            }
        }
        // the face of the hero: 3 columns of 2 cells (the tile list of 0x156A8, the palette word of 0x156E8)
        col += 0x11 + 3;
        let mut a0 = Self::rk_address(col, row);
        let id = e.chr as u32;
        let attr = text.u16(FACE_PALETTES, 2 * id);
        let list = text.u32(FACE_POINTERS, 4 * id);
        for k in 0..3u32 {
            for j in 0..2u32 {
                let code = text.u16(list, 2 * (2 * k + j));
                ops.push(Self::rk_cell1(a0 + 4 * j, code, attr));
            }
            a0 += 0x80;
        }
        // the stage digit and the marks "-" and "R" (0x2D and 0x52, attribute 0) one row down, two columns on
        col += 3;
        let start = Self::rk_address(col, row);
        let mut a1 = start;
        let (code, d5) = self.rk_char(e.stage as u16 + 0x30);
        Self::rk_glyph(ops, &mut a1, code, 0, d5);
        ops.push(Self::rk_cell1(start + 0x104, 0x2D, 0));
        ops.push(Self::rk_cell1(start + 0x184, 0x52, 0));
    }
}
