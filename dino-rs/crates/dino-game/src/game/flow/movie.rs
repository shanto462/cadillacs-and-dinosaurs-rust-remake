//! The opening movie of the attract loop, part 1: the routine 0x9BF50 with its timeline engine, the two helper
//! routines 0x9C7C4 and 0x9C7F8 and the handlers of the counters 10, 200 and 500 (0x9C04A, 0x9C088, 0x9C21C)
//! (M1-09). Owner: flow / HUD port. Spec: `D4d-front-end-and-story.md` s3.8, s13 A9.
//!
//! The attract task calls it after the ranking of state 6 (`flow_attract_ranking_end`, 0x97D34); it runs inside
//! the task like the profile cards (`profiles.rs`): the script position of `front.task` is one of the `mst`
//! values, the counter G.-$7004 is `task.count` and the index into the two tables G.-$7002 is `task.sub` (a byte
//! offset, steps of 2). One pass of the frame loop (0x9BFA8): sleep 1, the helper 0x9C7C4 (scrolls the jungle
//! layers while the flag G.-$7000 is set), the helper 0x9C7F8 (shakes scroll2 Y with a random number while the
//! flag G.-$6FFE is set), the shake task 0x222D6, pool B 0x23604, pool A 0x23420, the sprite list 0x13FA6, the
//! counter + 1; at 0x5C8 the routine returns; when the counter has reached the entry of the table 0x9C002 the
//! handler of the same index in the table 0x9C026 runs (only one per pass):
//!
//! | counter | handler | what |
//! | --- | --- | --- |
//! | 10 | 0x9C04A | cameras 0, fade out, clear, the first caption 0x9CBB2 (tall font), fade in |
//! | 200 | 0x9C088 | fade out, clear, the crawl 0x9CDEC (tall font), fade in |
//! | 500 | 0x9C21C | fade out, clear, scroll2 X = -12, two blocks of tiles (0x9C280, 0x9C28C), the Capcom sign 0x110C, fade in |
//! | 600 .. 1450 | 0x9C29C .. 0x9C788 | the jungle and the logo (M1-10, M1-11): **a stub** |
//!
//! The counter does not run while a handler waits (the fades are loops inside the handler), which is why the
//! 1480 counts last 1743 frames. The movie draws no object before the counter 600 and no random number before
//! the jungle (the first one is 12413 in `attract3.mame.trc`).
//!
//! # The stub
//!
//! The handlers of the counters 600 to 1450 are not ported yet: at the pass of the counter 600 (698 frames after the
//! call of the routine) the screen is cleared with a label (`flow_attract_stub`) and the rest of the movie
//! is a wait of the measured length, so the routine returns 1745 frames after its call as the original does
//! (`MOVIE_FRAMES` in `attract.rs`, both runs of `attract3.mame.trc`). M1-10 and M1-11 replace it handler by
//! handler (`Game::flow_movie_handler`).
//!
//! # CPU time
//!
//! Like the other scenes: a clear of all layers needs 1.3 frames (`flow_attract_busy`, the code after it runs two
//! frames later), the fades are 16 steps with one frame between them, all inside the task. The recording shows
//! the same frames in both runs (`attract3.mame.trc`: the call is at the records 11664 and 24769).

use super::attract::Stub;
use super::boot::st;
use crate::game::Game;

/// Script positions of the slot 0x10 task inside the routine (the others are `boot::st`).
pub(super) mod mst {
    /// 0x9BF84 after the clear: palette set 0 and full brightness, the counters.
    pub const PALETTES: u16 = 160;
    /// 0x9BFA8: the pass of a frame (after its sleep).
    pub const PASS: u16 = 161;
    /// 0x87D8 as called by 0x87BA / 0x879A: the fade loop, one step per frame.
    pub const FADE_OUT: u16 = 162;
    pub const FADE_IN: u16 = 163;
    /// A handler after its fade out: the clear of all layers.
    pub const CLEAR: u16 = 164;
    /// A handler after the clear: what it draws, then its fade in.
    pub const DRAW: u16 = 165;
    /// The frame after a clear (see `Game::flow_movie_clear`).
    pub const LATE: u16 = 166;

    /// The script positions of this routine.
    pub fn owns(state: u16) -> bool {
        (PALETTES..=LATE).contains(&state)
    }
}

/// The two tables of the routine (words, in `FRONT_TABLES`): the counter values (0x9C002) and the handler offsets
/// from 0x9C026 (one more entry at the front).
const TIMELINE: u32 = 0x9C002;
const HANDLERS: u32 = 0x9C026;
/// The counter at which the routine returns (0x9BFD2).
const LAST_COUNT: i16 = 0x5C8;
/// The captions, tall font strings: the first (0x9C076) and the crawl (0x9C09C).
const TEXT_CAPTION: u32 = 0x9CBB2;
const TEXT_CRAWL: u32 = 0x9CDEC;
/// The two blocks of the Capcom part (0x9C236, 0x9C252): the code lists 0x9C280 and 0x9C28C, the attribute lists
/// 0x9C28E and 0x9C29A.
const BLOCK_A: (u32, u32, u32) = (0x90F224, 0x9C280, 0x9C28E);
const BLOCK_B: (u32, u32, u32) = (0x90F228, 0x9C28C, 0x9C29A);
/// The handlers of the table 0x9C026 that are ported (by address).
const H_CAPTION: u32 = 0x9C04A;
const H_CRAWL: u32 = 0x9C088;
const H_CAPCOM: u32 = 0x9C21C;

/// The state of the movie task that the other tasks do not have: the running handler and the two flags of the
/// helpers (M1-10 and M1-11 set them from the handlers of the counters 850 to 1150).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Movie {
    /// The handler that is running (its address), for the positions it shares with the others.
    pub handler: u32,
    /// G.-$7000: 1 and 2 scroll the jungle layers (0x9C7C4).
    pub scroll: u16,
    /// G.-$6FFE: the random shake of scroll2 Y (0x9C7F8).
    pub shake: u16,
    /// G.-$6FFC: the Y that the shake goes round.
    pub base_y: u16,
    /// The tick the routine was called in (`front.ticks`): the stub waits for the rest of the measured length.
    pub start: u32,
    /// The position the script goes on in after the clear that is under way (`mst::LATE`).
    pub after: u16,
}

impl Game {
    /// 0x9BF50 up to the clear of all layers (0xAE0): the pools reset (0x49A8, 0x4A18, 0x4A46, 0x4C48, 0x4C76,
    /// 0x12BE4, 0xFBE8), G.-$6FFE = 0, the clear (two frames, the system task loses a pass); `mst::PALETTES` goes on.
    pub(super) fn flow_movie_start(&mut self) {
        self.flow.front.stub = Stub::Movie;
        self.flow.movie.start = self.flow.front.ticks;
        self.title_pools_reset();
        self.flow.movie.shake = 0; // clr.w -$6FFE(a5)
        self.flow_movie_clear(mst::PALETTES); // 0xAE0
    }

    /// 0xAE0 in the movie task: the clear needs 1.3 frames, so the code after it runs two frames later (as everywhere,
    /// `flow_attract_busy`), but here the system task misses its pass in the second of the two frames, not in the
    /// first (the free play wait counter of `m109_free` shows it at all four clears of the part that is ported): the
    /// movie's pass (the pool updates and the sprite list) has run before the clear in the same frame.
    fn flow_movie_clear(&mut self, next: u16) {
        self.flow_attract_clear();
        self.flow.movie.after = next;
        self.flow.front.task.wait(1, mst::LATE);
    }

    /// The script of the routine, one position per call (see `boot.rs`).
    pub(super) fn flow_movie_script(&mut self) {
        match self.flow.front.task.state {
            mst::PALETTES => {
                // 0x9BF84: palette set 0 (0x89E6) at full brightness (0x892C), the counter G.-$7004, the index
                // G.-$7002 and the flag G.-$7000 to 0, then the sleep at the top of the loop
                self.flow_scene_palettes(0);
                self.brightness_full();
                let t = &mut self.flow.front.task;
                (t.count, t.sub) = (0, 0);
                self.flow.movie.scroll = 0;
                self.flow_movie_loop_top();
            }
            mst::PASS => {
                self.flow_movie_helpers(); // 0x9BFAE, 0x9BFB2
                self.screen_shake(); // 0x222D6
                self.pool_b_update(); // 0x23604
                self.enemies_update(); // 0x23420
                self.build_sprite_list(); // 0x13FA6
                self.flow_movie_count();
            }
            mst::FADE_OUT => self.flow_movie_fade_step(false),
            mst::FADE_IN => self.flow_movie_fade_step(true),
            mst::CLEAR => {
                // the fade out is over: 0xAE0 (the call takes two frames of CPU time, the system task loses a pass)
                self.flow_movie_clear(mst::DRAW);
            }
            mst::LATE => {
                self.flow.front.sys_skip = 1;
                self.flow.front.task.wait(1, self.flow.movie.after);
            }
            _ => self.flow_movie_draw(), // mst::DRAW
        }
    }

    /// 0x9BFA8: the sleep at the top of the loop.
    fn flow_movie_loop_top(&mut self) {
        self.flow.front.task.wait(1, mst::PASS);
    }

    /// 0x9BFCE..0x9BFFA: the counter + 1, the end test and the handler of the table.
    fn flow_movie_count(&mut self) {
        let t = &mut self.flow.front.task;
        t.count = t.count.wrapping_add(1); // addq.w #1,-$7004(a5)
        let counter = t.count as i16;
        if counter >= LAST_COUNT {
            // 0x9BFFC: the routine returns (to the state routine 0x97D02, `attract.rs`)
            self.flow.front.task.state = st::MOVIE_END;
            return;
        }
        let text = &self.assets.frontend.text;
        let entry = text.i16(TIMELINE, t.sub as u32);
        if entry > counter {
            self.flow_movie_loop_top(); // not yet: the next pass
            return;
        }
        t.sub += 2; // addq.w #2,-$7002(a5)
        let handler = HANDLERS + text.u16(HANDLERS, t.sub as u32) as u32;
        self.flow_movie_handler(handler);
    }

    /// The handler `handler` of the table 0x9C026 starts. The ones of M1-10 and M1-11 are the stub.
    fn flow_movie_handler(&mut self, handler: u32) {
        self.flow.movie.handler = handler;
        match handler {
            H_CAPTION => {
                // 0x9C04A: scroll2 Y, scroll3 X and Y to 0 (G.$748, G.$7E4, G.$7E8), then the fade out
                self.cam.s2.cam_y = 0;
                self.cam.s3.cam_x = 0;
                self.cam.s3.cam_y = 0;
                self.flow_movie_fade_start(false);
            }
            H_CRAWL | H_CAPCOM => self.flow_movie_fade_start(false), // 0x9C088, 0x9C21C
            _ => {
                // 0x9C29C .. 0x9C788 (M1-10, M1-11): the screen is cleared with a label and the routine returns
                // as late as the original does
                self.flow_attract_stub(Stub::Movie, "OPENING MOVIE");
                let elapsed = self.flow.front.ticks.wrapping_sub(self.flow.movie.start) as u16;
                self.flow_attract_wait(super::attract::MOVIE_FRAMES.saturating_sub(elapsed), st::MOVIE_END);
            }
        }
    }

    /// What the handler draws after its clear, then its fade in (0x9C076 .. 0x9C082, 0x9C09C .. 0x9C0A8,
    /// 0x9C22A .. 0x9C27A).
    fn flow_movie_draw(&mut self) {
        match self.flow.movie.handler {
            H_CAPTION => self.flow_movie_text(TEXT_CAPTION), // 0x9C076: 0x84F72
            H_CRAWL => self.flow_movie_text(TEXT_CRAWL),     // 0x9C09C: 0x84F72
            _ => {
                // H_CAPCOM: scroll2 X = -12 (0x9C22A, after the clear), the blocks, the Capcom sign
                self.cam.s2.cam_x = 0xFFF4;
                self.flow_movie_block(BLOCK_A, 6, 1); // d1 = 5, d2 = 0 (0x98B86)
                self.flow_movie_block(BLOCK_B, 1, 1); // d1 = 0, d2 = 0
                self.flow_capcom_sign(0x10, 0x0E); // 0x110C with d0 = 0x10, d1 = 0xE
            }
        }
        self.flow_movie_fade_start(true); // moveq #$10,d0 ; jmp 0x879A
    }

    /// A tall font text (0x84F72).
    fn flow_movie_text(&mut self, addr: u32) {
        let s = self.assets.frontend.text.slice_from(addr);
        self.text_tall(s);
    }

    /// A scroll2 block (0x98B86): the address, the code list and the attribute list of the original.
    fn flow_movie_block(&mut self, (at, codes, attrs): (u32, u32, u32), width: u16, height: u16) {
        let t = &self.assets.frontend.text;
        let (codes, attrs) = (t.slice_from(codes), t.slice_from(attrs));
        self.flow_block2(at, codes, attrs, width, height);
    }

    /// 0x87BA (fade out) and 0x879A (fade in) with d0 = 0x10: all four banks, 16 steps with one frame between
    /// them, in the task itself (the step counter is G.$44B, the mask G.$44A). Step 1 runs at once.
    fn flow_movie_fade_start(&mut self, fade_in: bool) {
        self.fade.delay = 1; // move.w #1,$44C(a5)
        self.fade.mask = 0x0F; // move.b #$f,$44A(a5)
        self.fade.steps = 0x10;
        self.flow_movie_fade_step(fade_in);
    }

    /// 0x8808: one step of all banks (0x88B0), then the step counter; at 0 the fade is over (the mask byte is
    /// cleared, 0x881E) and the handler goes on in the same frame (the clear after a fade out, the end of the handler
    /// after a fade in), else the task sleeps one frame (0x95A).
    fn flow_movie_fade_step(&mut self, fade_in: bool) {
        self.fade_step(fade_in); // 0x88B0
        self.fade.steps = self.fade.steps.wrapping_sub(1);
        if self.fade.steps == 0 {
            self.fade.mask = 0;
            if fade_in {
                self.flow_movie_loop_top(); // the handler returns to the loop
            } else {
                self.flow.front.task.state = mst::CLEAR;
            }
        } else {
            let next = if fade_in { mst::FADE_IN } else { mst::FADE_OUT };
            self.flow.front.task.wait(self.fade.delay, next);
        }
    }

    /// 0x9C7C4 and 0x9C7F8, the two helpers of every pass.
    fn flow_movie_helpers(&mut self) {
        // 0x9C7C4: while G.-$7000 is not 0, scroll2 X + 8 (+ 4 when the flag is 2) and scroll3 X + 4, scroll3 X back
        // to 0 above 0x100 (the flags are set by the handlers of the counters 850 to 1150)
        if self.flow.movie.scroll != 0 {
            if self.flow.movie.scroll != 2 {
                self.cam.s2.cam_x = self.cam.s2.cam_x.wrapping_add(4);
            }
            self.cam.s2.cam_x = self.cam.s2.cam_x.wrapping_add(4);
            self.cam.s3.cam_x = self.cam.s3.cam_x.wrapping_add(4);
            if (self.cam.s3.cam_x as i16) > 0x100 {
                self.cam.s3.cam_x = 0;
            }
        }
        // 0x9C7F8: while G.-$6FFE is not 0, scroll2 Y = G.-$6FFC + (rand() & 3) - 1
        if self.flow.movie.shake != 0 {
            let r = self.rand() & 3;
            self.cam.s2.cam_y = r.wrapping_sub(1).wrapping_add(self.flow.movie.base_y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game() -> Game {
        Game::new(Box::leak(Box::new(AssetPack::default())))
    }

    /// 0x9C7C4: flag 1 moves scroll2 X by 8 and scroll3 X by 4 a pass, flag 2 scroll2 X by 4; scroll3 X goes back to 0
    /// above 0x100; flag 0 does nothing.
    #[test]
    fn scroll_helper_0x9c7c4() {
        let mut g = game();
        g.flow_movie_helpers();
        assert_eq!((g.cam.s2.cam_x, g.cam.s3.cam_x), (0, 0));
        g.flow.movie.scroll = 1;
        g.flow_movie_helpers();
        assert_eq!((g.cam.s2.cam_x, g.cam.s3.cam_x), (8, 4));
        g.flow.movie.scroll = 2;
        g.flow_movie_helpers();
        assert_eq!((g.cam.s2.cam_x, g.cam.s3.cam_x), (12, 8));
        // scroll3 X stays at 0x100 (ble 0x9C7F6) and goes back to 0 in the pass after it
        g.cam.s3.cam_x = 0xFC;
        g.flow_movie_helpers();
        assert_eq!(g.cam.s3.cam_x, 0x100);
        g.flow_movie_helpers();
        assert_eq!(g.cam.s3.cam_x, 0);
    }

    /// 0x9C7F8: scroll2 Y = G.-$6FFC + (rand() & 3) - 1, one random number a pass, only while the flag is set.
    #[test]
    fn shake_helper_0x9c7f8() {
        let mut g = game();
        g.flow.movie.base_y = 0x40;
        let rng = g.w.rng;
        g.flow_movie_helpers();
        assert_eq!((g.cam.s2.cam_y, g.w.rng), (0, rng), "flag 0: no random number");
        g.flow.movie.shake = 1;
        let mut probe = game();
        for _ in 0..20 {
            let r = probe.rand() & 3;
            g.flow_movie_helpers();
            assert_eq!(g.cam.s2.cam_y, 0x40 + r - 1);
        }
        assert_eq!(g.w.rng, probe.w.rng, "one rand() a pass");
    }
}
