//! The five story slides of the attract loop: the routine 0x988C0 (M1-08). Owner: flow / HUD port. Spec:
//! `D4d-front-end-and-story.md` s3.7, s13 A8.
//!
//! The attract task calls it after the demo of state 2 (`flow_attract_demo_end`, 0x97CD6); it runs inside the
//! task like the profile cards (`profiles.rs`): the script position of `front.task` is one of the `sst` values,
//! `task.count` is the counter of the two loops and `task.sub` the position a fade goes on in. The routine is a
//! straight script of sleeps (0xA76 = 1, 0xA7E = 30, 0xA82 = 60, 0xA8C = 180 frames). It has no frame loop and
//! no object: the pools are not updated, no sprite list is built (0xAE0 blanks the sprites), no random number is
//! drawn. The frames below count from `S`, the frame the routine is called (the demo's fade ends in it).
//!
//! | slide | frames | what |
//! | --- | --- | --- |
//! | 1 | S to S + 332 | clear, palette set 0 and full brightness, music 0x24, scroll2 pages 0x0D and 0x0E, scroll3 pages 0x6A and 0x6B, scroll3 Y 0x10; the two captions at S + 32 and S + 92 |
//! | 2 | S + 332 to S + 649 | hard cut: clear, an 8 x 6 scroll2 block at 0x90F1D4, caption (the second is empty), fade out 16 |
//! | 3 | S + 649 to S + 951 | clear, scroll2 pages 0x10 and 0x11, scroll3 pages 0x64 and 0x71, fade in 16, captions at S + 696 (and an empty one), fade out |
//! | 4 | S + 951 to S + 1407 | clear, scroll3 X 0x180, layer control 0x1396, the 8 x 6 block again, fade in, a 4 x 3 scroll3 block at 0x913828, 49 frames of scroll3 X - 8 and scroll2 X - 2, caption at S + 1077 |
//! | 5 | S + 1407 to S + 1777 | music 0x25, layer control 0x12D6, clear, caption, 120 frames later scroll1 cleared, scroll3 pages 0x6D and 0x6E, two captions, 128 frames, 120 more |
//!
//! The routine ends with a clear of all layers (S + 1777) and returns two frames later (S + 1779, the length of
//! the stub it replaces); `attract.rs` goes on with the stop code, the clear and the ranking.
//!
//! # CPU time
//!
//! Each 0xAE0 needs 1.3 frames, so the code after it runs two frames later (`flow_attract_busy`, scroll1 is blank
//! at once, scroll2 a frame later and scroll3 one more: `flow_attract_clear`). Everything between two sleeps
//! (a clear, the four page loads, the captions) falls inside one video frame in the recording: no cut by the
//! frame boundary is visible. The recording shows the same frames in both runs (`attract3.mame.trc`: S = 4668 and
//! 17773), and the scroll registers of the screen are the shadows two frames late, as everywhere (`present.rs`).

use super::attract::Stub;
use super::boot::st;
use crate::game::Game;

/// Script positions of the slot 0x10 task inside the routine (the others are `boot::st`).
pub(super) mod sst {
    /// 0x988C6 after the clear: palettes, music, the four pictures of slide 1.
    pub const PICS1: u16 = 96;
    /// 0x9892C: the first caption of slide 1.
    pub const TEXT1A: u16 = 97;
    /// 0x9893E: the second caption of slide 1.
    pub const TEXT1B: u16 = 98;
    /// 0x98950: A8C.
    pub const WAIT1: u16 = 99;
    /// 0x98956: the clear that cuts to slide 2.
    pub const CLEAR2: u16 = 100;
    /// 0x9895C: the block and the caption of slide 2.
    pub const PICS2: u16 = 101;
    pub const TEXT2B: u16 = 102;
    pub const WAIT2: u16 = 103;
    /// 0x989A6: the fade out to slide 3.
    pub const FADE_OUT2: u16 = 104;
    /// 0x989B4: the clear and the pictures of slide 3.
    pub const CLEAR3: u16 = 105;
    pub const PICS3: u16 = 106;
    /// 0x98A06 after the fade in: A7E.
    pub const IN3: u16 = 107;
    pub const TEXT3A: u16 = 108;
    pub const TEXT3B: u16 = 109;
    pub const FADE_OUT3: u16 = 110;
    /// 0x98A3E: the clear, the registers and the block of slide 4.
    pub const CLEAR4: u16 = 111;
    pub const PICS4: u16 = 112;
    pub const IN4: u16 = 113;
    /// 0x98A7E: the 4 x 3 block, then the loop of 49 steps of the cameras.
    pub const BLOCK4: u16 = 114;
    pub const PAN: u16 = 115;
    pub const TEXT4A: u16 = 116;
    pub const TEXT4B: u16 = 117;
    pub const WAIT4: u16 = 118;
    pub const WAIT4B: u16 = 119;
    /// 0x98AEE: music 0x25, the blank, the clear of slide 5.
    pub const BLANK5: u16 = 120;
    pub const TEXT5A: u16 = 121;
    pub const TEXT5B: u16 = 122;
    /// 0x98B20: scroll1 cleared, the pictures and the two captions of slide 5.
    pub const PICS5: u16 = 123;
    /// 0x98B5C: the loop of 128 frames.
    pub const LOOP5: u16 = 124;
    pub const END1: u16 = 125;
    /// 0x98B72: the last clear.
    pub const FINAL: u16 = 126;
    /// 0x87D8 as called by 0x87BA / 0x879A: the fade loop, one step per frame.
    pub const FADE_OUT: u16 = 127;
    pub const FADE_IN: u16 = 128;

    /// The script positions of this routine.
    pub fn owns(state: u16) -> bool {
        (PICS1..=FADE_IN).contains(&state)
    }
}

/// Scroll2 and scroll3 page load addresses (the first and the second picture of a slide).
const S2_A: u32 = 0x90F000;
const S2_B: u32 = 0x90F400;
const S3_A: u32 = 0x913800;
const S3_B: u32 = 0x913900;
/// The block of slides 2 and 4 (scroll2, 8 x 6 cells, lists 0x98BD6 and 0x98C4E) and the one of slide 4
/// (scroll3, 4 x 3 cells, lists 0x98C36 and 0x98CAE).
const BLOCK2_AT: u32 = 0x90F1D4;
const BLOCK3_AT: u32 = 0x913828;
const BLOCK2_CODES: u32 = 0x98BD6;
const BLOCK2_ATTRS: u32 = 0x98C4E;
const BLOCK3_CODES: u32 = 0x98C36;
const BLOCK3_ATTRS: u32 = 0x98CAE;
/// The captions (tall font strings, 0x84F72), two per slide; the second of slides 2 to 4 only holds its
/// position and the end mark.
const TEXT_1A: u32 = 0x98CC6;
const TEXT_1B: u32 = 0x98D14;
const TEXT_2A: u32 = 0x98D7E;
const TEXT_2B: u32 = 0x98DE2;
const TEXT_3A: u32 = 0x98DE8;
const TEXT_3B: u32 = 0x98E4A;
const TEXT_4A: u32 = 0x98E50;
const TEXT_4B: u32 = 0x98EDA;
const TEXT_5A: u32 = 0x98EE0;
const TEXT_5B: u32 = 0x98F2A;
const TEXT_5C: u32 = 0x98F8A;
/// The steps of the pan of slide 4: d1 = 0x30 (49 passes of the loop at 0x98AA8).
const PAN_STEPS: u16 = 0x30;
/// The loop of slide 5: d5 = 0x7F (128 sleeps of one frame, 0x98B5A).
const LOOP5_STEPS: u16 = 0x7F;

impl Game {
    /// The caption at `addr` (0x84F72).
    fn flow_slides_text(&mut self, addr: u32) {
        let s = self.assets.frontend.text.slice_from(addr);
        self.text_tall(s);
    }

    /// 0x98B86 with the lists of the 8 x 6 block, at 0x90F1D4.
    fn flow_slides_block2(&mut self) {
        let t = &self.assets.frontend.text;
        let (codes, attrs) = (t.slice_from(BLOCK2_CODES), t.slice_from(BLOCK2_ATTRS));
        self.flow_block2(BLOCK2_AT, codes, attrs, 8, 6); // d1 = 7, d2 = 5
    }

    /// 0x988C0: the first instruction (0xAE0). The call takes two frames of CPU time; the palettes and the
    /// first pictures follow in `sst::PICS1`.
    pub(super) fn flow_slides_start(&mut self) {
        self.flow.front.stub = Stub::Slides;
        self.flow_attract_clear(); // 0xAE0
        self.flow_attract_busy(2, 1, sst::PICS1);
    }

    /// The script of the routine, one position per call (see `boot.rs`).
    pub(super) fn flow_slides_script(&mut self) {
        match self.flow.front.task.state {
            sst::PICS1 => {
                // 0x988C6: palette set 0 (0x89E6), full brightness (0x892C), scroll2 and scroll3 X to 0, the
                // music 0x24 (0xA63C), scroll3 Y 0x10, the four pictures, then A7E
                self.flow_scene_palettes(0);
                self.brightness_full();
                self.cam.s2.cam_x = 0;
                self.cam.s3.cam_x = 0;
                self.sound(0x24);
                self.cam.s3.cam_y = 0x10;
                self.flow_page2(S2_A, 0x0D); // 0x204CA
                self.flow_page2(S2_B, 0x0E);
                self.flow_page3(S3_A, 0x6A); // 0x20592
                self.flow_page3(S3_B, 0x6B);
                self.flow.front.task.wait(30, sst::TEXT1A); // 0xA7E
            }
            sst::TEXT1A => {
                self.flow_slides_text(TEXT_1A); // 0x9892C
                self.flow.front.task.wait(60, sst::TEXT1B); // 0xA82
            }
            sst::TEXT1B => {
                self.flow_slides_text(TEXT_1B); // 0x9893E
                self.flow.front.task.wait(60, sst::WAIT1);
            }
            sst::WAIT1 => self.flow.front.task.wait(180, sst::CLEAR2), // 0xA8C
            sst::CLEAR2 => {
                // 0x98956: the hard cut. The registers: scroll3 Y 0 (0xAE0 clears it too)
                self.flow_attract_clear();
                self.flow_attract_busy(2, 1, sst::PICS2);
            }
            sst::PICS2 => {
                self.cam.s3.cam_y = 0; // 0x9895C
                self.flow_slides_block2(); // 0x98B86
                self.flow_slides_text(TEXT_2A); // 0x9897C
                self.flow.front.task.wait(60, sst::TEXT2B); // 0x98988: A82
            }
            sst::TEXT2B => {
                self.flow_slides_text(TEXT_2B); // 0x9898E (prints nothing)
                self.flow.front.task.wait(60, sst::WAIT2);
            }
            sst::WAIT2 => self.flow.front.task.wait(180, sst::FADE_OUT2), // 0x989A0: A8C
            sst::FADE_OUT2 => self.flow_slides_fade_start(false, sst::CLEAR3), // 0x989A6: 0x87BA
            sst::CLEAR3 => {
                self.flow_attract_clear(); // 0x989B4
                self.flow_attract_busy(2, 1, sst::PICS3);
            }
            sst::PICS3 => {
                self.cam.s3.cam_y = 0x10; // 0x989BA
                self.flow_page2(S2_A, 0x10);
                self.flow_page2(S2_B, 0x11);
                self.flow_page3(S3_A, 0x64);
                self.flow_page3(S3_B, 0x71);
                self.flow_slides_fade_start(true, sst::IN3); // 0x989F8: 0x879A
            }
            sst::IN3 => self.flow.front.task.wait(30, sst::TEXT3A), // 0x98A06: A7E
            sst::TEXT3A => {
                self.flow_slides_text(TEXT_3A); // 0x98A0C
                self.flow.front.task.wait(60, sst::TEXT3B);
            }
            sst::TEXT3B => {
                self.flow_slides_text(TEXT_3B); // 0x98A1E (prints nothing)
                self.flow.front.task.wait(180, sst::FADE_OUT3); // 0x98A2A: A8C
            }
            sst::FADE_OUT3 => self.flow_slides_fade_start(false, sst::CLEAR4), // 0x98A30
            sst::CLEAR4 => {
                self.flow_attract_clear(); // 0x98A3E
                self.flow_attract_busy(2, 1, sst::PICS4);
            }
            sst::PICS4 => {
                self.cam.s3.cam_x = 0x180; // 0x98A44
                self.cam.s3.cam_y = 0x10;
                self.w.layer_ctrl = 0x1396;
                self.flow_slides_block2();
                self.flow_slides_fade_start(true, sst::IN4); // 0x98A70
            }
            sst::IN4 => self.flow.front.task.wait(30, sst::BLOCK4), // 0x98A7E
            sst::BLOCK4 => {
                let t = &self.assets.frontend.text;
                let (codes, attrs) = (t.slice_from(BLOCK3_CODES), t.slice_from(BLOCK3_ATTRS));
                self.flow_block3(BLOCK3_AT, codes, attrs, 4, 3); // 0x98BAE, d1 = 3, d2 = 2
                self.flow.front.task.count = PAN_STEPS;
                self.flow.front.task.wait(30, sst::PAN); // 0x98A9E: A7E
            }
            sst::PAN => {
                // 0x98AA4 (d1 = 0x30), 0x98AA8: scroll3 X - 8, scroll2 X - 2, sleep 1, dbra d1
                self.cam.s3.cam_x = self.cam.s3.cam_x.wrapping_sub(8);
                self.cam.s2.cam_x = self.cam.s2.cam_x.wrapping_sub(2);
                let t = &mut self.flow.front.task;
                if t.count == 0 {
                    t.wait(1, sst::TEXT4A);
                } else {
                    t.count -= 1;
                    t.wait(1, sst::PAN);
                }
            }
            sst::TEXT4A => {
                self.flow_slides_text(TEXT_4A); // 0x98ABE
                self.flow.front.task.wait(60, sst::TEXT4B);
            }
            sst::TEXT4B => {
                self.flow_slides_text(TEXT_4B); // 0x98AD0 (prints nothing)
                self.flow.front.task.wait(60, sst::WAIT4);
            }
            sst::WAIT4 => self.flow.front.task.wait(180, sst::WAIT4B), // 0x98AE2: A8C
            sst::WAIT4B => self.flow.front.task.wait(30, sst::BLANK5), // 0x98AE8: A7E
            sst::BLANK5 => {
                // 0x98AEE: music 0x25, layer control 0x12D6, scroll3 Y 0, the clear
                self.sound(0x25);
                self.w.layer_ctrl = 0x12D6;
                self.cam.s3.cam_y = 0;
                self.flow_attract_clear();
                self.flow_attract_busy(2, 1, sst::TEXT5A);
            }
            sst::TEXT5A => {
                self.flow_slides_text(TEXT_5A); // 0x98B08
                self.flow.front.task.wait(60, sst::TEXT5B);
            }
            sst::TEXT5B => self.flow.front.task.wait(60, sst::PICS5), // 0x98B1A
            sst::PICS5 => {
                // 0x98B20: scroll1 cleared (0xB52), the two pictures, the two captions, then 128 sleeps
                self.flow_clear_scroll1();
                self.flow_page3(S3_A, 0x6D); // 0x20592
                self.flow_page3(S3_B, 0x6E);
                self.flow_slides_text(TEXT_5B); // 0x98B42
                self.flow_slides_text(TEXT_5C); // 0x98B4E
                self.flow.front.task.count = LOOP5_STEPS;
                self.flow.front.task.wait(1, sst::LOOP5);
            }
            sst::LOOP5 => {
                // 0x98B5C: dbra d5 after each sleep; the end goes on with A82
                let t = &mut self.flow.front.task;
                t.count = t.count.wrapping_sub(1);
                if t.count == 0xFFFF {
                    t.wait(60, sst::END1);
                } else {
                    t.wait(1, sst::LOOP5);
                }
            }
            sst::END1 => self.flow.front.task.wait(60, sst::FINAL), // 0x98B66 / 0x98B6C: A82
            sst::FINAL => {
                // 0x98B72: the last clear, the cameras to 0 (G.$744, G.$7E4), the routine returns. The state
                // routine that follows (`flow_attract_slides_end`) starts its own clear at once, so the CPU is
                // busy for more than two frames and the system task misses two passes here (the free play wait
                // counter of `m108_free` shows it), one at each of the other clears.
                self.flow_attract_clear();
                self.cam.s2.cam_x = 0;
                self.cam.s3.cam_x = 0;
                self.flow_attract_busy(2, 2, st::SLIDES_END);
            }
            sst::FADE_OUT => self.flow_slides_fade_step(false),
            _ => self.flow_slides_fade_step(true), // sst::FADE_IN
        }
    }

    /// 0x87BA (fade out) and 0x879A (fade in) with d0 = 0x10: all four banks, 16 steps with one frame between
    /// them, in the task itself (the step counter is G.$44B, the mask G.$44A). Step 1 runs at once; `next` is
    /// the position the script goes on in when the fade is over.
    fn flow_slides_fade_start(&mut self, fade_in: bool, next: u16) {
        self.fade.delay = 1; // move.w #1,$44C(a5)
        self.fade.mask = 0x0F; // move.b #$f,$44A(a5)
        self.fade.steps = 0x10;
        self.flow.front.task.sub = next;
        self.flow_slides_fade_step(fade_in);
    }

    /// 0x8808: one step of all banks (0x88B0), then the step counter; at 0 the fade is over (the mask byte is
    /// cleared, 0x881E) and the script goes on in the same frame, else the task sleeps one frame (0x95A).
    fn flow_slides_fade_step(&mut self, fade_in: bool) {
        self.fade_step(fade_in); // 0x88B0
        self.fade.steps = self.fade.steps.wrapping_sub(1);
        if self.fade.steps == 0 {
            self.fade.mask = 0;
            self.flow.front.task.state = self.flow.front.task.sub;
        } else {
            let next = if fade_in { sst::FADE_IN } else { sst::FADE_OUT };
            self.flow.front.task.wait(self.fade.delay, next);
        }
    }
}
