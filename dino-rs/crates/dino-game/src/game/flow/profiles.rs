//! The four character profile cards of the attract loop: the timeline routine 0x9A7E6 with its
//! handlers 0x9A92E .. 0x9AB32 (M1-05). Owner: flow / HUD port. Spec: `D4d-front-end-and-story.md`
//! s3.4, s13 A5.
//!
//! The routine runs inside the attract task (slot 0x10): the script position of `front.task` is one
//! of the `pst` values while it runs, the counter G.-$7018 is `task.count` and the index into the two
//! tables G.-$7016 is `task.sub` (a byte offset, steps of 2). One pass of the frame loop (0x9A844):
//! sleep 1, the two helpers 0x9AC86 and 0x9ACBA (they only act when G.-$7014 or G.-$7012 is not 0,
//! and nothing in the ROM ever sets them, so the checker board does not move by them), the shake task
//! 0x222D6, pool B 0x23604, pool A 0x23420, the sprite list 0x13FA6, the counter + 1; at 0x4B0 the
//! routine returns; when the counter has reached the entry of the table 0x9A8E4 the handler of the
//! same index in the table 0x9A908 runs (only one per pass):
//!
//! | counter | handler | what |
//! | --- | --- | --- |
//! | 9, 11 | 0x9A94E | nothing |
//! | 10 | 0x9A950 | card 1 (Jack): the music 0x13, label block, text, objects, labels |
//! | 308, 608, 908, 1190 | 0x9A92E | fade out (0x87BA: 16 steps, one per frame) |
//! | 309, 609, 909 | 0x9ABDC | clear all layers (0xAE0), sleep 30 |
//! | 310, 610, 910 | 0x9A9FE, 0x9AA98, 0x9AB32 | cards 2 to 4 (Hannah, Mustapha, Mess) |
//! | 311, 611, 911 | 0x9A93E | fade in (0x879A) |
//!
//! The counter does not run while a handler waits (the fade loops and the sleeps are inside the
//! handlers), which is why 1200 counts last 1398 frames.
//!
//! # CPU time
//!
//! The recording shows the effects of the original's CPU time, which the port writes as measured
//! waits (like `boot.rs`): a card's objects are created at the end of its handler; the next pass
//! starts one frame later and the first update of the checker board object (pool B class 93, it
//! clears the scroll3 map twice) runs over the video frame boundary, so the palette is written in
//! the first frame and the rest of the pass (board, pool A, sprite list, counter) in the second
//! (`pst::CUT` and `pst::CUT_END`). Two runs are measured, the first one in `attract.mame.trc` and
//! the later ones in `attract3.mame.trc` (`front.profile_runs`): the first run is one frame ahead
//! of the later ones at 8 places (it ends 8 frames earlier, 1399 frames against 1407): the first
//! pass after the set-up, the pass of the counter 12 (it comes in the frame of the pass of 11) and, for
//! each of the cards 2 to 4, the pass after the sleep of the clear and the second step of the fade in
//! (in the frame of the first). The checks are in `mame_check.rs`.

use super::attract::Stub;
use super::boot::st;
use crate::game::Game;

/// Script positions of the slot 0x10 task inside the routine (the others are `boot::st`).
pub(super) mod pst {
    /// 0x9A844: the pass of a frame.
    pub const PASS: u16 = 64;
    /// The first half of the pass after a card start (the frame boundary falls in it).
    pub const CUT: u16 = 65;
    /// The second half of that pass: the rest of the pool B update, pool A, the sprite list, the counter.
    pub const CUT_END: u16 = 66;
    /// 0x87D8 as called by 0x9A92E / 0x9A93E: the fade loop, one step per frame.
    pub const FADE_OUT: u16 = 67;
    pub const FADE_IN: u16 = 68;
    /// 0x9ABDC after its clear: sleep 30, then the handler returns.
    pub const CLEAR_SLEEP: u16 = 69;
    pub const CLEAR_DONE: u16 = 70;
    /// 0x9A820 after the clear of the set-up: the palettes.
    pub const PALETTES: u16 = 71;

    /// The script positions of this routine.
    pub fn owns(state: u16) -> bool {
        (PASS..=PALETTES).contains(&state)
    }
}

/// The tables of the routine: the counter values (words, 0x9A8E4) and the handler offsets from 0x9A908
/// (words, one more entry at the front). Both are in `FRONT_TABLES`.
const TIMELINE: u32 = 0x9A8E4;
const HANDLERS: u32 = 0x9A908;
/// The counter at which the routine returns (0x9A86E).
const LAST_COUNT: i16 = 0x4B0;
/// The card texts (tall font), one per card in the order of the screen: Jack, Hannah, Mustapha, Mess.
const CARD_TEXT: [u32; 4] = [0x9B0F8, 0x9B192, 0x9B224, 0x9B2C2];
/// The label block 0x9AC16 (the frame of the card), "CM / KG" 0x9ABF6 and "HEIGHT / WEIGHT" 0x9AC02.
const LABEL_BLOCK: u32 = 0x9AC16;
const LABEL_UNITS: u32 = 0x9ABF6;
const LABEL_NAMES: u32 = 0x9AC02;
/// Model field of the pool B class 93 object: its first update is split by the frame boundary
/// (the object writes it to the state that waits for the second half, see `prop/class93.rs`).
pub(crate) const CUT_FIELD: usize = 0xBF;

impl Game {
    /// The profile routine runs for the first time since power on (the measured differences).
    fn profile_first_run(&self) -> bool {
        self.flow.front.profile_runs <= 1
    }

    /// 0x9A7E6 up to the clear of all layers (0xAE0): layer control 0x12D6, the pools reset, the clear
    /// (two frames, the system task loses a pass); `pst::PALETTES` goes on.
    pub(super) fn flow_profile_start(&mut self) {
        self.flow.front.stub = Stub::Profiles;
        self.w.layer_ctrl = 0x12D6;
        self.title_pools_reset(); // 0x49A8, 0x4A18, 0x4A46, 0x4C48, 0x4C76, 0x12BE4, 0xFBE8
        self.flow_attract_clear(); // 0xAE0
        // the counter G.-$7018 and the index G.-$7016 are 0 when the routine is called (0x9A832 and
        // 0x9A838 set them after the palettes; the RAM the demo clears and the first boot leave 0)
        self.flow.front.task.count = 0;
        self.flow.front.task.sub = 0;
        self.flow_attract_busy(2, 1, pst::PALETTES);
    }

    /// The script of the routine, one position per call (see `boot.rs`).
    pub(super) fn flow_profile_script(&mut self) {
        match self.flow.front.task.state {
            pst::PALETTES => {
                // 0x9A820: palette set 0 at full brightness (the counters are 0 already, see
                // `flow_profile_start`), then the sleep at the top of the loop (the first run is a
                // frame ahead)
                self.flow_scene_palettes(0); // 0x89E6
                self.brightness_full(); // 0x892C
                self.flow_profile_loop_top(self.profile_first_run());
            }
            pst::PASS => {
                // 0x9A84A / 0x9A84E: the helpers do nothing (G.-$7014 and G.-$7012 are 0)
                self.screen_shake(); // 0x222D6
                self.pool_b_update(); // 0x23604
                self.enemies_update(); // 0x23420
                self.build_sprite_list(); // 0x13FA6
                self.flow_profile_count();
            }
            pst::CUT => {
                // the first half of the pass after a card start: pool B up to the frame boundary
                self.screen_shake();
                self.pool_b_update();
                self.flow.front.task.wait(1, pst::CUT_END);
            }
            pst::CUT_END => {
                self.prop_card_board_finish(); // the rest of the board object's first update
                self.enemies_update();
                self.build_sprite_list();
                self.flow_profile_count();
            }
            pst::FADE_OUT => self.flow_profile_fade_step(false),
            pst::FADE_IN => self.flow_profile_fade_step(true),
            pst::CLEAR_SLEEP => self.flow.front.task.wait(30, pst::CLEAR_DONE), // 0xA7E
            _ => {
                // pst::CLEAR_DONE: 0x9ABDC returns, then the sleep at the top of the loop. The first
                // run is a frame ahead here.
                self.flow_profile_loop_top(self.profile_first_run());
            }
        }
    }

    /// 0x9A844: the sleep at the top of the loop. `ahead`: the frame the first run does without.
    fn flow_profile_loop_top(&mut self, ahead: bool) {
        self.w.layer_ctrl = 0x12D6; // 0x9A896
        if ahead {
            self.flow.front.task.state = pst::PASS;
        } else {
            self.flow.front.task.wait(1, pst::PASS);
        }
    }

    /// 0x9A86A..0x9A892: the counter + 1, the end test and the handler of the table.
    fn flow_profile_count(&mut self) {
        let t = &mut self.flow.front.task;
        t.count = t.count.wrapping_add(1); // addq.w #1,-$7018
        let counter = t.count as i16;
        if counter >= LAST_COUNT {
            // 0x9A89E: the routine returns (to the state routine 0x97D84, `attract.rs`)
            self.w.layer_ctrl = 0x12D6;
            self.flow.front.task.state = st::PROFILES_END;
            return;
        }
        let text = &self.assets.frontend.text;
        let entry = text.i16(TIMELINE, t.sub as u32);
        if entry > counter {
            self.flow_profile_loop_top(false); // not yet: the next pass
            return;
        }
        t.sub += 2; // addq.w #2,-$7016
        let handler = HANDLERS + text.u16(HANDLERS, t.sub as u32) as u32;
        match handler {
            0x9A950 => self.flow_profile_card(0),
            0x9A9FE => self.flow_profile_card(1),
            0x9AA98 => self.flow_profile_card(2),
            0x9AB32 => self.flow_profile_card(3),
            0x9A92E => self.flow_profile_fade_start(false),
            0x9A93E => self.flow_profile_fade_start(true),
            0x9ABDC => {
                // clear all layers; the call takes two frames of CPU time, the system task loses one pass
                self.flow_attract_clear(); // 0xAE0
                self.flow_attract_busy(2, 1, pst::CLEAR_SLEEP);
            }
            // 0x9A94E: rts. (0x9AC4A, a fade and a clear for the table entries 2200 and 10000, is
            // never reached: the routine returns at 1200.) For the first run the counter 11 is
            // the pass that follows the card start and runs a frame ahead, see the module comment.
            _ => self.flow_profile_loop_top(counter == 11 && self.profile_first_run()),
        }
    }

    /// 0x9A950 / 0x9A9FE / 0x9AA98 / 0x9AB32: one card. Clear scroll1 (0xB52), the music (card 1
    /// only), the label block, the card text in the tall font (0x84F72), the four objects (pool B
    /// classes 93 and 94, pool A class 100 twice), the units and the names of the two numbers.
    fn flow_profile_card(&mut self, card: usize) {
        self.flow_clear_scroll1(); // jsr 0xB52
        if card == 0 {
            self.sound(0x13); // 0x9A956: moveq #$13,d0 ; 0xA63C with d1 = d2 = 0
        }
        let text = &self.assets.frontend.text;
        let (label, card_text) = (text.slice_from(LABEL_BLOCK), text.slice_from(CARD_TEXT[card]));
        self.text_bytes(label, true); // 0xBB6
        self.text_tall(card_text); // 0x84F72
        // the objects: `jsr 0x4860` (pool B) / `jsr 0x483C` (pool A), then +$00 = 1, +$20 = class * 4,
        // +$26 = the card. The last card creates its class 94 object first.
        let (first, second) = if card == 3 { (94, 93) } else { (93, 94) };
        self.flow_profile_spawn_b(first, card as u16);
        self.flow_profile_spawn_b(second, card as u16);
        self.flow_profile_spawn_a(100, card as u16);
        self.flow_profile_spawn_a(100, card as u16 + 4);
        let text = &self.assets.frontend.text;
        let (units, names) = (text.slice_from(LABEL_UNITS), text.slice_from(LABEL_NAMES));
        self.text_bytes(units, true);
        self.text_bytes(names, true);
        // the handler returns: the next pass is the one the frame boundary falls in
        self.w.layer_ctrl = 0x12D6;
        self.flow.front.task.wait(1, pst::CUT);
    }

    /// `jsr 0x4860`, +$00 = 1, +$20 = class * 4, +$26 = var.
    pub(crate) fn flow_profile_spawn_b(&mut self, class: u16, var: u16) {
        let Some(i) = self.props.alloc() else { return };
        let p = &mut self.props.slots[i];
        p.b.live = 1;
        p.b.k20 = class * 4;
        p.b.var26 = var;
        if class == 93 {
            p.m.sb(CUT_FIELD, 1); // the first update crosses the frame boundary
        }
    }

    /// `jsr 0x483C`, +$00 = 1, +$20 = class * 4, +$26 = var.
    pub(crate) fn flow_profile_spawn_a(&mut self, class: u16, var: u16) {
        let Some(i) = self.enemies.alloc() else { return };
        let e = &mut self.enemies.slots[i];
        e.b.live = 1;
        e.b.k20 = class * 4;
        e.b.var26 = var;
    }

    /// 0x87BA (fade out) and 0x879A (fade in) as the handlers 0x9A92E / 0x9A93E call them: all four
    /// banks, 16 steps with one frame between them, in the task itself. Step 1 runs at once.
    fn flow_profile_fade_start(&mut self, fade_in: bool) {
        self.fade.delay = 1; // move.w #1,$44C(a5)
        self.fade.mask = 0x0F; // move.b #$f,$44A(a5)
        self.fade.steps = 0x10; // d0 = 0x10, 0x87E4
        self.flow_profile_fade_step(fade_in);
        // the first run does the second step of a fade in in the frame of the first one
        if fade_in && self.profile_first_run() && self.flow.front.task.state == pst::FADE_IN {
            self.flow_profile_fade_step(true);
        }
    }

    /// 0x8808: one step of all banks (0x88B0), then the step counter; at 0 the fade is over (the
    /// mask byte is cleared, 0x881E) and the handler returns, else the task sleeps one frame (0x95A).
    fn flow_profile_fade_step(&mut self, fade_in: bool) {
        self.fade_step(fade_in); // 0x88B0
        self.fade.steps = self.fade.steps.wrapping_sub(1);
        if self.fade.steps == 0 {
            self.fade.mask = 0;
            self.flow_profile_loop_top(false); // the handler returns
        } else {
            let next = if fade_in { pst::FADE_IN } else { pst::FADE_OUT };
            self.flow.front.task.wait(self.fade.delay, next);
        }
    }
}
