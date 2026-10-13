//! The opening movie of the attract loop: the routine 0x9BF50 with its timeline engine, the helpers 0x9C7C4 and 0x9C7F8 and
//! the handlers of the counters 10 to 1450 (0x9C04A .. 0x9C788) (M1-09 to M1-11). Owner: flow / HUD port. Spec:
//! `D4d-front-end-and-story.md` s3.8, s13 A9.
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
//! | 600 | 0x9C29C | the jungle: the bars (pool B 32), the pictures, fade in, the two hunters (pool A 11), the sound 0x61 |
//! | 850, 895 | 0x9C3D4, 0x9C3DC | the flags of the helpers: the scroll, the shake |
//! | 890 | 0x9C5CE | the second line of the hunter (0x9CBE6) |
//! | 900, 930 | 0x9C52A, 0x9C46C | two raptors that run, three that stand (pool A 12) |
//! | 1100 | 0x9C426 | the brown dinosaur (pool A 30: three shakes of the ground, the roar) |
//! | 1120, 1140, 1150 | 0x9C5C6, 0x9C5BE, 0x9C5AC | the shake off, the fast scroll, the third caption (0x9CCD4) and the scroll off |
//! | 1250 | 0x9C3E4 | the two pieces of the big head (pool A 69) and the caption 0x9CC64 |
//! | 1350 | 0x9C5D8 | the caption 0x9CC9C, the two pieces of the logo (pool A 98), the palettes of the logo, the task sleeps 120 frames |
//! | 1450 | 0x9C788 | fade out, clear |
//!
//! The counter does not run while a handler waits (the fades and the sleep are inside the handler), which is why the
//! 1480 counts last 1745 frames. The routine returns after the pass of the counter 1480. The movie draws no random
//! number before the jungle (the first one is at record 12413 in `attract3.mame.trc`).
//!
//! # CPU time
//!
//! Like the other scenes: a clear of all layers needs 1.3 frames (the code after it runs two frames later), the fades are 16
//! steps with one frame between them, all inside the task. In the movie the system task loses its pass in the second of the
//! two frames, not in the first (`flow_movie_clear`). The recording shows the same frames in both runs
//! (`attract3.mame.trc`: the call is at the records 11664 and 24769).
//!
//! The screen shake of the brown dinosaur (G.$755C = 4, `cam_shake`) is requested, but the shake task 0x222D6 is not
//! ported (`screen_shake` is a stub until M3-10): the pictures of the three shakes differ from the original's by the
//! few pixels the displayed scroll2 Y moves; the camera shadows, the tilemaps and the sprites are not affected.

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
    /// The end of a sleep inside a handler (0x9C694: A86): the loop goes on.
    pub const AFTER_SLEEP: u16 = 167;

    /// The script positions of this routine.
    pub fn owns(state: u16) -> bool {
        (PALETTES..=AFTER_SLEEP).contains(&state)
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
/// The jungle (M1-10): the handler of the counter 600, the flags and the caption of the walk (850, 890, 895) and the
/// golden raptors (900, 930).
const H_JUNGLE: u32 = 0x9C29C;
const H_SCROLL_ON: u32 = 0x9C3D4;
const H_TEXT_HUGE: u32 = 0x9C5CE;
const H_SHAKE_ON: u32 = 0x9C3DC;
const H_RAPTORS_RUN: u32 = 0x9C52A;
const H_RAPTORS_STAND: u32 = 0x9C46C;
/// The text of the counter 890 (the hunter's second line).
const TEXT_HUGE: u32 = 0x9CBE6;
/// The brown dinosaur and the flags (1100, 1120, 1140, 1150), the head and the caption (1250), the logo (1350) and the
/// end (1450) (M1-11).
const H_BROWN: u32 = 0x9C426;
const H_SHAKE_OFF: u32 = 0x9C5C6;
const H_SCROLL_FAST: u32 = 0x9C5BE;
const H_TEXT_WILD: u32 = 0x9C5AC;
const H_HEAD: u32 = 0x9C3E4;
const H_LOGO: u32 = 0x9C5D8;
const H_FADE_END: u32 = 0x9C788;
const TEXT_WILD: u32 = 0x9CCD4;
const TEXT_HEAD: u32 = 0x9CC64;
const TEXT_LOGO: u32 = 0x9CC9C;
/// The palettes of the logo (0x9C61A): the offsets in the OBJ... scroll2 bank (0x914800 + `DEST_A`) and in the ROM tables,
/// then the same for scroll3 (0x914C00 + `DEST_B`).
const PAL_DEST_A: u32 = 0x9C69C;
const PAL_SRC_A: u32 = 0x9C6AE;
const PAL_DEST_B: u32 = 0x9C6C0;
const PAL_SRC_B: u32 = 0x9C6C4;
const PAL_ROM_A: u32 = 0xBCA52 + 0x3400;
const PAL_ROM_B: u32 = 0xC2252 + 0x3000;
/// The scroll2 pages of the jungle (0x9C2D0 .. 0x9C302) and the scroll3 page (0x9C308 .. 0x9C322).
const JUNGLE_PAGES2: [(u32, usize); 4] = [(0x90F000, 3), (0x90F400, 4), (0x90F800, 3), (0x90FC00, 3)];
const JUNGLE_PAGE3: usize = 0x72;
const JUNGLE_PAGES3_AT: [u32; 3] = [0x913800, 0x913900, 0x913A00];

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
            mst::AFTER_SLEEP => self.flow_movie_loop_top(),
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
            H_JUNGLE => {
                // 0x9C29C: scroll2 X = 0, the fill of the bars (0x81888), the fade out (no clear: the pictures
                // are loaded over the old ones)
                self.cam.s2.cam_x = 0;
                self.prop_c32_fill();
                self.flow_movie_fade_start(false);
            }
            H_SCROLL_ON => {
                self.flow.movie.scroll = 1; // 0x9C3D4
                self.flow_movie_loop_top();
            }
            H_SHAKE_ON => {
                self.flow.movie.shake = 1; // 0x9C3DC
                self.flow_movie_loop_top();
            }
            H_TEXT_HUGE => {
                self.flow_movie_text(TEXT_HUGE); // 0x9C5CE: jmp 0x84F72
                self.flow_movie_loop_top();
            }
            H_RAPTORS_RUN => {
                // 0x9C52A: two raptors that run (the byte +$26 = 1) 0x180 and 0x280 right of the camera
                self.flow_movie_raptors(&[0x180, 0x280], 0x0100);
                self.flow_movie_loop_top();
            }
            H_RAPTORS_STAND => {
                // 0x9C46C: three raptors that stand about, 0x180, 0x1C0 and 0x230 right of the camera
                self.flow_movie_raptors(&[0x180, 0x1C0, 0x230], 0);
                self.flow_movie_loop_top();
            }
            H_BROWN => {
                self.flow_movie_brown(); // 0x9C426
                self.flow_movie_loop_top();
            }
            H_SHAKE_OFF => {
                self.flow.movie.shake = 0; // 0x9C5C6
                self.flow_movie_loop_top();
            }
            H_SCROLL_FAST => {
                self.flow.movie.scroll = 2; // 0x9C5BE
                self.flow_movie_loop_top();
            }
            H_TEXT_WILD => {
                self.flow_movie_text(TEXT_WILD); // 0x9C5AC
                self.flow.movie.scroll = 0;
                self.flow_movie_loop_top();
            }
            H_HEAD => {
                // 0x9C3E4: two pool A class 69 objects (the variants 0 and 1, no position), the caption
                for var in 0..2u16 {
                    if let Some(i) = self.enemies.alloc() {
                        let b = &mut self.enemies.slots[i].b;
                        b.live = 1;
                        b.k20 = 0x114;
                        b.var26 = var;
                    }
                }
                self.flow_movie_text(TEXT_HEAD);
                self.flow_movie_loop_top();
            }
            H_LOGO => {
                // 0x9C5D8: the caption, the two pieces of the logo (pool A class 98, variants 0x10 and 0x11), the
                // palettes, then A86: the task sleeps 120 frames
                self.flow_movie_text(TEXT_LOGO);
                self.enemy_spawn_logo(0x10);
                self.enemy_spawn_logo(0x11);
                self.flow_movie_logo_palettes();
                self.flow.front.task.wait(120, mst::AFTER_SLEEP);
            }
            H_FADE_END => self.flow_movie_fade_start(false), // 0x9C788: the fade out, then the clear
            _ => {
                // not a handler of the table the movie reaches (the entry 1500 at 0x9C79C comes after the end)
                self.flow_movie_loop_top();
            }
        }
    }

    /// What the handler draws after its clear, then its fade in (0x9C076 .. 0x9C082, 0x9C09C .. 0x9C0A8,
    /// 0x9C22A .. 0x9C27A).
    fn flow_movie_draw(&mut self) {
        match self.flow.movie.handler {
            H_CAPTION => self.flow_movie_text(TEXT_CAPTION), // 0x9C076: 0x84F72
            H_CRAWL => self.flow_movie_text(TEXT_CRAWL),     // 0x9C09C: 0x84F72
            H_JUNGLE => {
                // 0x9C2B6: pool B class 32 (the bars), the pictures: scroll2 pages 3, 4, 3, 3 (0x204CA), scroll3
                // page 0x72 three times (0x20592)
                if let Some(i) = self.props.alloc() {
                    let p = &mut self.props.slots[i];
                    p.b.live = 1;
                    p.b.k20 = 0x80;
                }
                for (at, page) in JUNGLE_PAGES2 {
                    self.flow_page2(at, page);
                }
                for at in JUNGLE_PAGES3_AT {
                    self.flow_page3(at, JUNGLE_PAGE3);
                }
            }
            H_FADE_END => {}
            _ => {
                // H_CAPCOM: scroll2 X = -12 (0x9C22A, after the clear), the blocks, the Capcom sign
                self.cam.s2.cam_x = 0xFFF4;
                self.flow_movie_block(BLOCK_A, 6, 1); // d1 = 5, d2 = 0 (0x98B86)
                self.flow_movie_block(BLOCK_B, 1, 1); // d1 = 0, d2 = 0
                self.flow_capcom_sign(0x10, 0x0E); // 0x110C with d0 = 0x10, d1 = 0xE
            }
        }
        if self.flow.movie.handler == H_FADE_END {
            self.flow_movie_loop_top(); // 0x9C788 ends with the clear (`jmp 0xAE0`): the loop goes on
        } else {
            self.flow_movie_fade_start(true); // moveq #$10,d0 ; jmp 0x879A
        }
    }

    /// What a handler does after its fade in, in the same frame: the jungle creates the two hunters (pool A class 11,
    /// 0x9C340 and 0x9C380), remembers scroll2 Y in G.-$6FFC and queues the sound 0x61 (0x9C3C6).
    fn flow_movie_handler_end(&mut self) {
        if self.flow.movie.handler != H_JUNGLE {
            return;
        }
        let (cx, cy) = (self.cam.s2.cam_x as i16, self.cam.s2.cam_y as i16);
        // `jsr 0x483C` without the carry test, +$00 = 1, +$20 = 0x2C, X = G.$744 + 0x40 / 0x20, Y = G.$748 + 0x28,
        // depth G.$74C (+ 1 for the second), the byte +$26 (the high byte of the variant word) = 0 / 1, +$22 = 0
        for (dx, dz, var) in [(0x40i16, 0i16, 0u16), (0x20, 1, 0x0100)] {
            if let Some(i) = self.enemies.alloc() {
                let b = &mut self.enemies.slots[i].b;
                b.live = 1;
                b.k20 = 0x2C;
                b.set_xi(cx.wrapping_add(dx));
                b.set_yi(cy.wrapping_add(0x28));
                b.set_zi(dz);
                b.var26 = var;
                b.dir = 0;
            }
        }
        self.flow.movie.base_y = self.cam.s2.cam_y; // move.w $748(a5),-$6FFC(a5)
        self.sound(0x61); // moveq #$61,d0 ; jmp 0xA63C
    }

    /// Pool A class 12 objects (`jsr 0x483C`, +$00 = 1, +$20 = 0x30, X = G.$744 + `dx`, Y = G.$748 + 0x20, depth G.$74C, the
    /// byte +$26 = `var`'s high byte, +$22 = 0).
    fn flow_movie_raptors(&mut self, dx: &[i16], var: u16) {
        let (cx, cy) = (self.cam.s2.cam_x as i16, self.cam.s2.cam_y as i16);
        for &d in dx {
            if let Some(i) = self.enemies.alloc() {
                let b = &mut self.enemies.slots[i].b;
                b.live = 1;
                b.k20 = 0x30;
                b.set_xi(cx.wrapping_add(d));
                b.set_yi(cy.wrapping_add(0x20));
                b.set_zi(0);
                b.var26 = var;
                b.dir = 0;
            }
        }
    }

    /// 0x9C426: the brown dinosaur (pool A class 30, `jsr 0x483C`, +$00 = 1, +$20 = 0x78, X = G.$744 + 0x180, Y = G.$748 +
    /// 0x40, depth G.$74C, +$26 = 1, +$22 = 0).
    fn flow_movie_brown(&mut self) {
        if let Some(i) = self.enemies.alloc() {
            let b = &mut self.enemies.slots[i].b;
            b.live = 1;
            b.k20 = 0x78;
            b.set_xi((self.cam.s2.cam_x as i16).wrapping_add(0x180));
            b.set_yi((self.cam.s2.cam_y as i16).wrapping_add(0x40));
            b.set_zi(0);
            b.var26 = 1;
            b.dir = 0;
        }
    }

    /// 0x9C61A..0x9C66E: the palettes of the logo. Nine palettes of the scroll2 bank (0x914800 + the offsets of
    /// 0x9C69C) from the ROM at 0xBCA52 + 0x3400 + the offsets of 0x9C6AE, two of the scroll3 bank (0x914C00) from 0xC2252 +
    /// 0x3000. Every copy is 17 words (0x9C818: the colours under the brightness nibble the bank has), one more than a
    /// palette, so the first colour of the next palette is overwritten first and then written again.
    fn flow_movie_logo_palettes(&mut self) {
        let t = &self.assets.frontend.text;
        let mut jobs: Vec<(usize, Vec<u16>)> = Vec::new();
        for k in 0..9u32 {
            let dst = (t.u16(PAL_DEST_A, 2 * k) / 2) as usize;
            let src = PAL_ROM_A + t.u16(PAL_SRC_A, 2 * k) as u32;
            jobs.push((crate::game::vram::PaletteRam::SCROLL2 + dst, (0..17).map(|j| t.u16(src, 2 * j)).collect()));
        }
        for k in 0..2u32 {
            let dst = (t.u16(PAL_DEST_B, 2 * k) / 2) as usize;
            let src = PAL_ROM_B + t.u16(PAL_SRC_B, 2 * k) as u32;
            jobs.push((crate::game::vram::PaletteRam::SCROLL3 + dst, (0..17).map(|j| t.u16(src, 2 * j)).collect()));
        }
        for (at, words) in jobs {
            for (j, w) in words.into_iter().enumerate() {
                let cell = &mut self.pal.words[at + j];
                *cell = (*cell & 0xF000) | w;
            }
        }
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
                self.flow_movie_handler_end(); // 0x9C340 ..: what the handler does after its fade in
                self.flow_movie_loop_top(); // the handler returns to the loop
            } else if self.flow.movie.handler == H_JUNGLE {
                self.flow.front.task.state = mst::DRAW; // no clear
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
    use std::path::Path;
    use std::sync::OnceLock;

    /// The asset pack of the user's ROM (`../dino.zip`), `None` when it is not there.
    fn rom() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    fn code_at(g: &Game, a: u32) -> u16 {
        g.vram.scroll1[((a - 0x908000) / 4) as usize & 0xFFF].code
    }

    /// The black bars (pool B 32): the fill of 34 rows by 49 cells with the tile 0x18, nothing for 10 frames after the
    /// first update, then every 4th frame two more rows (the pair 0x90A03C / 0x90A040, then one row further out
    /// on each side) of transparent cells (0x1F), 9 steps, the object frees itself after the 10th.
    #[test]
    fn the_bars_open_in_nine_steps() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        let i = g.props.alloc().unwrap();
        g.props.slots[i].b.live = 1;
        g.props.slots[i].b.k20 = 0x80;
        let rows_open = |g: &Game| (0..32u32).filter(|r| code_at(g, 0x90A000 + 4 * r) == 0x1F).count();
        let mut opened = vec![0usize];
        for pass in 1..=50 {
            g.pool_b_update();
            if pass == 1 {
                assert_eq!((code_at(&g, 0x90A004), code_at(&g, 0x90A000 + 4 * 34 + 0x80 * 48)), (0x18, 0x18), "the fill");
                assert_eq!(code_at(&g, 0x90A000), 0, "row 0 is not part of it");
            }
            if opened.last().copied() != Some(rows_open(&g)) {
                opened.push(rows_open(&g));
                assert!(matches!(pass, 12 | 16 | 20 | 24 | 28 | 32 | 36 | 40 | 44), "a step in pass {pass}");
            }
            assert_eq!(g.props.slots[i].b.live != 0, pass < 49, "pass {pass}");
        }
        assert_eq!(opened, [0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
    }

    /// The flock spawner (pool B 72, variant 0x600): seven birds (pool A 46), 8 frames apart from the 8th, at the camera
    /// X + 0xB0 - 0x40 + (a random number & 0x3C), one random number each, then it frees itself.
    #[test]
    fn the_flock_is_seven_birds_eight_frames_apart() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        g.cam.s2.cam_x = 0x100;
        g.prop_flock_spawn();
        let rng = g.w.rng;
        let mut probe = Game::new(a);
        probe.w.rng = rng;
        let mut born = Vec::new();
        let mut seen = 0;
        for pass in 1..=70 {
            g.pool_b_update();
            let n = g.enemies.slots.iter().filter(|e| e.b.live != 0).count();
            if n > seen {
                born.push(pass);
                seen = n;
                let x = 0x100 + 0xB0 - 0x40 + (probe.rand() & 0x3C) as i16;
                let bird = g.enemies.slots.iter().find(|e| e.b.live != 0 && e.b.xi() == x).expect("a bird at the random place");
                assert_eq!((bird.b.k20, bird.b.palette_override, bird.b.var26), (0xB8, 0x98, 0));
            }
        }
        assert_eq!(born, [8, 16, 24, 32, 40, 48, 56]);
        assert_eq!(g.w.rng, probe.w.rng, "one random number a bird");
        assert!(g.props.slots.iter().all(|p| p.b.live == 0), "the spawner is gone");
    }

    /// The brown dinosaur (pool A 30): the ground shakes (request 4, sound 0x75) at its frames 0, 0x40 and 0x80, the roar
    /// 0x36 at 0xC2, and it frees itself when its counter is 0x160 (one frame later, see the module comment of the class).
    #[test]
    fn the_brown_dinosaur_shakes_the_ground_three_times() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        g.flow_movie_brown();
        let mut events = Vec::new();
        let mut gone = None;
        for frame in 0..0x170 {
            g.w.sounds.clear();
            g.cam.shake_req = 0;
            g.enemies_update();
            g.draw_list.clear();
            for &s in &g.w.sounds {
                events.push((frame, s, g.cam.shake_req));
            }
            if gone.is_none() && g.enemies.slots.iter().all(|e| e.b.live == 0) {
                gone = Some(frame);
            }
        }
        assert_eq!(events, [(0, 0x75, 4), (0x40, 0x75, 4), (0x80, 0x75, 4), (0xC2, 0x36, 0)]);
        assert_eq!(gone, Some(0x160), "gone in the frame its counter shows 0x160 + 1");
    }

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
