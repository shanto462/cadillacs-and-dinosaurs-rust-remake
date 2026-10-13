//! The attract demo: the rows table, the RAM clear and the setup of the original (0x99F10), the 30 recorded input
//! streams and their playback (0x9A146, 0x9A0CE), and the frame loop and the end (0x99E88). Owner: flow / HUD port
//! (M1-12 and M1-13). Spec: `D4d-front-end-and-story.md` s3.6, s13 A3, A7.
//!
//! The demo is a routine of the attract task (slot 0x10): the states 2 and 6 of `attract.rs` call it after the title
//! (0x97CC0, 0x97D02) with the demo counter G.$757A already stepped (0x97D6C, 0x97D78). It plays whatever level the
//! row names with three heroes that "press" the recorded buttons, with the random number generator reset
//! (G.$4 to G.$E = 0, G.$14 = 0x1C3), so every demo of a row is the same. It runs 1201 game frames (G.$757C counts
//! 0x4B0 down to -1), keeps the last picture for 100 frames, and fades out in 16 steps.
//!
//! # The setup 0x99F10
//!
//! The setup clears most of the work RAM (0xFF0E66 to 0xFF8000, G.$0 to $15, G.$34 to $449, G.$4AC to $6EDD,
//! G.$6F1E to $700D, G.$735E to $7579; it keeps the attract state byte, the inputs, the video registers, the fade state,
//! the system task (G.$46A to G.$4AB), the high score table G.$703E, the demo counter and the settings), and then does
//! the same as a game start: the DIP decode (0xA804), the rank (0x17F8A), the three player objects (0x4D46 ...), the
//! stage setup (0xAC6A with 0xA9BE), the start of the scroll controllers (0x2228A) and the HUD (0x162AE). The port has
//! no memory image: [`Game::flow_demo_clear_ram`] builds a fresh [`Game`] and moves what the clear keeps into it. The
//! original needs about 10 frames of CPU time for all of this; the visible steps are written as waits of the measured
//! length (`dst`). A is the video frame (record) in which the title returned and the demo counter G.$757A stepped
//! (records of `flow_m113-long.mame.trc`: 3343, 9914, 16448, 23019, 29553; record = video frame - 2):
//!
//! | frame | what the recording shows |
//! | --- | --- |
//! | A | the sweeps of the clear begin: the attract state byte and the title counter are 0 |
//! | A + 2 | G.$0, G.$14 (0), the mode, the stage, the objects, G.$4CD ... are cleared |
//! | A + 3 | the sweeps are over, the state byte is back, 0xAE0 begins: the sprite buffers are blank |
//! | A + 4 | scroll1 blank |
//! | A + 5 | scroll2 blank, G.$757C and the playback state cleared; the rank and the player objects (four draws of the generator) |
//! | A + 6 | the heroes, G.$14 = 0x1C3, the row (G.$4D9, G.$786, G.$757C), the palettes |
//! | A + 7 | 0xA9BE: mode 4, G.$28 = 0x12D6 |
//! | A + 8 | mode 0xC, the scroll2 camera Y of the area, the sound codes 0xFF00 and the music |
//! | A + 9 | the map (0x2228A) |
//! | A + 10 | the HUD, the music again, and the first game frame (G.$0 = 1) |
//!
//! That is the timeline of the runs after the first (the demos 2 to 5 of the recording). The first demo after power on
//! is one frame ahead in the middle (the rank, the players, the row and the first part of the palettes in A + 5, the
//! brightness in A + 6, mode 0xC in A + 7 with the camera, the map and the HUD in A + 9) and then sleeps one frame before the first pass
//! (A + 10); Episode 1 (row 0, the sixth demo) does not sleep: its first pass starts in A + 9. In all of them the frame ends inside
//! the draw of the map (later runs: its last 44 cells come in A + 10) or of the HUD (Episode 3: its first 40 cells are
//! there in A + 9); the port draws both whole (the records A + 9 and A + 10 differ in those cells, see
//! `demos_set_up_and_run_in_step_with_mame`).
//!
//! The frame loop 0x99EA0 then runs the six routines of the level frame without the rank (0x186CC) and the time limit
//! (0x103FE, 0x1037E): the objects 0xE90E (the playback of the input streams is part of the player update 0x18834),
//! the collisions 0x10546, the HUD 0x16BEA, the sprite list 0x13FA6, the lists 0x105E8 and the spawn script 0xFBF4.
//! A frame never takes more than one video frame in the demos of the recordings (G.$0 counts every video frame, there is
//! no slowdown). The last pass is in A + 1210; the picture stays for 100 frames, the fade out has 16 steps (A + 1310 to A + 1325),
//! and the routine returns in A + 1325 (the old stub of M1-04 had this length: 1325 frames).
//!
//! # The streams
//!
//! A record is two bytes: the buttons (bit 0 right, 1 left, 2 down, 3 up, 4 attack, 5 jump) and the number of frames.
//! Each hero has a pointer to its record and the frames left (G.$758A + 0x10 * who). The routine writes the pad byte
//! G.$77A4 + who, which the player update reads instead of the real input (G.$16, which stays what it is). The port
//! keeps the pad bytes in [`Demo::pad`] and puts them into `w.input` for the object update only (nothing else reads the
//! real input in a level frame yet: `lib_target_input`, 0x32B94, is the only reader of G.$16 and none of its callers is
//! ported). The routine also writes four hex digits of the hero number (the debug display of 0x18F8) at the top left of
//! scroll1, where the last hero's value stays: 0xFFFF when a record changed, else the hero number times 4.

use super::attract::Stub;
use super::boot::st;
use crate::game::Game;

/// Script positions of the slot 0x10 task inside the routine (the others are `boot::st`).
pub(super) mod dst {
    /// A + 2: the sweeps of the RAM clear are done (G.$14 = 0, the objects, G.$4D0 ... are gone).
    pub const CLEAR_RNG: u16 = 192;
    /// A + 3: the state byte is restored, 0xAE0 begins (the cameras, the sprite buffers).
    pub const CLEAR_END: u16 = 193;
    /// A + 4: 0xAE0 has cleared scroll1.
    pub const CLEAR_S1: u16 = 194;
    /// A + 5: scroll2 and scroll3 are cleared; the rank and the players (the first run: the rest of the setup, the row, the
    /// palettes too).
    pub const CLEAR_S23: u16 = 195;
    /// A + 6 in the runs after the first: the other two players, the generator, the row, the streams, the palettes.
    pub const ROW: u16 = 196;
    /// A + 6 in the first run: 0x892C, the brightness of the palettes.
    pub const BRIGHT: u16 = 197;
    /// A + 7: 0xAC6A (0xA9BE and the rest), in the runs after the first the part up to the mode byte.
    pub const STAGE: u16 = 198;
    /// A + 8 in the runs after the first: the rest of 0xAC6A (mode 0xC, the sound codes).
    pub const TAIL: u16 = 199;
    /// A + 9: 0x2228A, the map; the first run goes on with the HUD and the music, the others do that in the next frame.
    pub const MAP: u16 = 200;
    /// A + 10 in the runs after the first: the HUD (0x162AE), the area music, and the first pass of the loop in the same frame.
    pub const HUD: u16 = 201;
    /// 0x99EA0: one pass of the frame loop per frame.
    pub const LOOP: u16 = 202;
    /// 0x99EF6 after its sleep of 100 frames: the fade out.
    pub const FADE_START: u16 = 203;
    pub const FADE: u16 = 204;

    /// The script positions of this routine.
    pub fn owns(state: u16) -> bool {
        (CLEAR_RNG..=FADE).contains(&state)
    }
}

/// The rows of the 3 player layout (0x9A1E2) and of the 2 player layout (0x9A1EE): word offsets from the table start.
const ROWS_3P: u32 = 0x9A1E2;
const ROWS_2P: u32 = 0x9A1EE;
/// The stream pointer tables (longs): 18 for the 3 player layout (0x9A2DE), 18 for the 2 player layout (0x9A326).
const STREAMS_3P: u32 = 0x9A2DE;
const STREAMS_2P: u32 = 0x9A326;
/// The start event of the scroll controller by stage and area, 4 bytes per stage (0x222B6, read by 0x2228A).
const START_EVENTS: u32 = 0x222B6;
/// The sleep before the fade out (0x99EF6: the last picture stays).
const FROZEN_FRAMES: u16 = 0x64;
/// The palette words (of the 2048 of the four banks) the loader 0x8A44 has written when the frame that starts the load ends: the
/// recording shows the OBJ and scroll1 banks and the first 8 palettes of scroll2 (words 0 to 1151) then.
const PALETTE_CUT: usize = 1152;

/// What the original keeps in RAM between the frames of the demo: the loop counter G.$757C, the input playback of the
/// three heroes (G.$758A + 0x10 * who: the ROM address of the current record, G.$758E the frames left), the pad bytes
/// G.$77A4..6 and what the RAM clear took out of the attract task.
#[derive(Clone, Copy, Debug, Default)]
pub struct Demo {
    /// G.$757C game frames left (0x4B0 at the start; the loop ends when it passes 0).
    pub left: u16,
    /// G.$758A + 0x10 * who: ROM address of the current record and the frames left of it.
    pub rec: [(u32, u8); 3],
    /// G.$77A4..6: the buttons the heroes press this frame.
    pub pad: [u8; 3],
    /// The attract state word, saved by 0x99F10 (d5) while the clear takes it out of RAM.
    pub saved_state: u8,
    /// How many demos ran since power on (the setup of the first one is timed differently, see the module comment).
    pub runs: u16,
}

/// One row of the table: the level, how many frames, and for each hero the stream, the hero and the class word.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DemoRow {
    pub stage: u8,
    pub area: u8,
    pub frames: u16,
    /// Per hero: (index of the stream pointer in bytes, the word at +2 of the object: who and hero id, class = id * 4).
    pub heroes: [(u16, u16, u16); 3],
}

impl Game {
    /// 0x9A03C..0x9A05E: the row of the counter G.$757A (the table of the 2 player layout when G.$4CF is 1). A counter of 6
    /// or more is taken as 0 and G.$757A is set to 0 (so the demo after the fifth is Episode 1).
    pub fn flow_demo_row(&mut self) -> DemoRow {
        let t = &self.assets.frontend.text;
        let two = self.flow.b4cf != 0;
        let base = if two { ROWS_2P } else { ROWS_3P };
        let mut c = self.flow.front.demo_counter;
        if c >= 6 {
            c = 0;
            self.flow.front.demo_counter = 0;
        }
        let a = base + t.u16(base, 2 * (c & 7) as u32) as u32;
        let mut row = DemoRow { stage: t.u8(a, 0), area: t.u8(a, 1), frames: t.u16(a, 2), ..Default::default() };
        let n = if two { 2 } else { 3 };
        for (k, h) in row.heroes.iter_mut().take(n).enumerate() {
            let p = 4 + 6 * k as u32;
            *h = (t.u16(a, p), t.u16(a, p + 2), t.u16(a, p + 4));
        }
        row
    }

    /// 0x9A0CE for one hero: the stream pointer and its first count (G.$758A + 0x10 * who, +4), the who / id word (+2) and
    /// the class (+$20) of the object. The position fields are cleared (the stage setup sets them).
    fn flow_demo_hero(&mut self, who: usize, h: (u16, u16, u16)) {
        let table = if self.flow.b4cf != 0 { STREAMS_2P } else { STREAMS_3P };
        let t = &self.assets.frontend.text;
        let ptr = t.u32(table, h.0 as u32);
        let count = t.u8(ptr, 1);
        self.flow.front.demo.rec[who] = (ptr, count);
        let b = &mut self.players[who].b;
        b.who = (h.1 >> 8) as u8;
        b.id = h.1 as u8;
        b.k20 = h.2;
    }

    /// 0x9A146, called by the player update 0x18834 for each hero (the third one only in the 3 player layout) when G.$4CC
    /// is 0: the frames left of the record count down; when they reach 0 the next record starts. The byte of the current
    /// record goes to the pad G.$77A4 + who. Then the debug display 0x18F8 writes four hex digits at 0x90A000 (columns 0 to
    /// 3 of the first row): d0 = 0xFFFF (the delay loop leaves it) when a record changed, else who * 4.
    fn flow_demo_playback(&mut self) {
        let n = if self.flow.b4cf != 0 { 2 } else { 3 };
        for who in 0..n {
            let (ptr, left) = self.flow.front.demo.rec[who];
            let t = &self.assets.frontend.text;
            let left = left.wrapping_sub(1); // subq.b #1, 4(a0)
            let (digits, rec) = if left == 0 {
                // 0x9A1B2: the next record: the new count, the byte of the new record
                let next = ptr + 2;
                self.flow.front.demo.pad[who] = t.u8(next, 0);
                (0xFFFFu16, (next, t.u8(next, 1)))
            } else {
                self.flow.front.demo.pad[who] = t.u8(ptr, 0); // 0x9A1D4
                (4 * who as u16, (ptr, left))
            };
            self.flow.front.demo.rec[who] = rec;
            // 0x18F8: the four nibbles, high first, one cell per column, attribute 0
            for k in 0..4u32 {
                let nib = (digits >> (12 - 4 * k)) & 0xF;
                self.hud_put(0x90A000 + 0x80 * k, nib, 0);
            }
        }
    }

    /// One game frame of the loop 0x99EC4: the counters G.$0 and G.$4DA, the objects (the player update with the playback),
    /// the collisions, the HUD, the sprite list, the lists and the spawn script. No rank (0x186CC), no time limit.
    fn flow_demo_frame(&mut self) {
        self.w.frame = self.w.frame.wrapping_add(1); // addq.w #1, 0(a5)
        self.flow.frame = self.flow.frame.wrapping_add(1); // addq.b #1, $4DA(a5)
        // 0xE90E: the player update (0x18834) copies G.$16..18 to G.$77A4..6, plays the stream over them and reads them
        self.flow_demo_playback();
        let real = self.w.input;
        self.w.input = self.flow.front.demo.pad;
        self.objects_update(); // 0xE90E
        self.w.input = real;
        self.collision(); // 0x10546
        self.hud_update(); // 0x16BEA
        self.build_sprite_list(); // 0x13FA6
        self.combat_lists_reset(); // 0x105E8
        self.spawn_script_tick(); // 0xFBF4
    }

    /// 0x99F10: what the RAM clear keeps (see the module comment) is moved into a fresh game, the rest is zero like the
    /// original's RAM. The state of the video and the palettes are not RAM.
    fn flow_demo_clear_ram(&mut self) {
        let mut fresh = Game::new(self.assets);
        // G.$16 to G.$33: the inputs, the system and coin bytes, the video registers
        fresh.w.input = self.w.input;
        fresh.w.system = self.w.system;
        fresh.w.coins = self.w.coins;
        fresh.w.video_ctrl = self.w.video_ctrl;
        fresh.w.layer_ctrl = self.w.layer_ctrl;
        fresh.w.priority_masks = self.w.priority_masks;
        fresh.w.sounds = std::mem::take(&mut self.w.sounds);
        // video memory, the palettes, the sprite buffers and the fade state are not cleared by the sweeps
        fresh.vram = std::mem::take(&mut self.vram);
        fresh.pal = std::mem::take(&mut self.pal);
        fresh.fade = std::mem::take(&mut self.fade);
        fresh.sprites = std::mem::take(&mut self.sprites);
        fresh.cam.latch = std::mem::take(&mut self.cam.latch);
        // the high score table, the settings, the region, the system task and the attract task
        fresh.hud.hiscores = std::mem::take(&mut self.hud.hiscores);
        let flow = std::mem::take(&mut self.flow);
        fresh.flow.dip = flow.dip;
        fresh.flow.config = flow.config;
        fresh.flow.region = flow.region;
        fresh.flow.front = flow.front;
        fresh.flow.front.select = Default::default(); // G.$62C is in the cleared range
        fresh.hold_unported = self.hold_unported;
        fresh.trace_unmapped = std::mem::take(&mut self.trace_unmapped);
        *self = fresh;
    }

    /// The call of the routine 0x99E88 by the state routines of the attract task (0x97CC0, 0x97D02), in the frame A the title
    /// returned and the counter was stepped: the first sweeps of the RAM clear take the attract state word and the title
    /// counter out of RAM.
    pub(super) fn flow_demo_start(&mut self) {
        let front = &mut self.flow.front;
        front.stub = Stub::Demo;
        front.demo.saved_state = front.attract_state; // move.w -$7030(a5), d5
        front.attract_state = 0;
        front.title = 0; // G.-$7022
        // the attract task does not sleep until the first pass of the loop, 9 frames from now: the system task (slot 0,
        // which runs before it) loses 9 passes, and the blink task (slot 0x20, which runs after it) first runs in the
        // frame after the task first sleeps (it was created by 0x98330 a moment ago)
        front.sys_skip = 9;
        front.blink.sleep(11);
        front.demo.runs = front.demo.runs.saturating_add(1);
        front.task.wait(2, dst::CLEAR_RNG);
    }

    /// The script of the routine, one step per frame it is awake in (`front.task`).
    pub(super) fn flow_demo_script(&mut self) {
        match self.flow.front.task.state {
            dst::CLEAR_RNG => {
                // the sweeps reach G.$0 .. G.$15 (the generator), G.$34 .. and G.$4AC ..: everything they take is gone
                self.flow_demo_clear_ram();
                self.flow.front.task.wait(1, dst::CLEAR_END);
            }
            dst::CLEAR_END => {
                // the sweeps are over, the state word is back (move.w d5, -$7030(a5)), 0xAE0 begins with the cameras and
                // the sprite buffers
                self.flow.front.attract_state = self.flow.front.demo.saved_state;
                let c = &mut self.cam;
                (c.cam1_x, c.cam1_y, c.s2.cam_x, c.s2.cam_y, c.s3.cam_x, c.s3.cam_y) = (0, 0, 0, 0, 0, 0);
                self.sprites.wipe();
                self.flow.front.task.wait(1, dst::CLEAR_S1);
            }
            dst::CLEAR_S1 => {
                self.flow_clear_scroll1(); // 0xB52
                self.flow.front.task.wait(1, dst::CLEAR_S23);
            }
            dst::CLEAR_S23 => {
                self.vram.clear2();
                self.vram.clear3();
                // 0x99F76: 31 words from G.$757C (the loop counter and the playback state) are cleared; the pad bytes are not
                let d = &mut self.flow.front.demo;
                (d.left, d.rec) = (0, Default::default());
                if self.flow.front.demo.runs <= 1 {
                    // the first run: the whole setup in this frame, then 0xA9BE begins with 0x8A44, which loads the four banks in
                    // order; this frame ends in the scroll2 bank, so the words of PALETTE_CUT on are still the title's (measured)
                    self.flow_demo_setup(0);
                    self.flow_demo_setup(1);
                    let old = self.pal.words[PALETTE_CUT..].to_vec();
                    self.load_all_palettes();
                    self.pal.words[PALETTE_CUT..].copy_from_slice(&old);
                    self.flow.front.task.wait(1, dst::BRIGHT);
                } else {
                    // the later runs: the frame ends after the rank and the three player objects (four draws of the generator)
                    self.flow_demo_setup(0);
                    self.flow.front.task.wait(1, dst::ROW);
                }
            }
            dst::ROW => {
                self.flow_demo_setup(1);
                self.load_all_palettes(); // 0x8A44 and 0x892C end in this frame
                self.brightness_full();
                self.flow.front.task.wait(1, dst::STAGE);
            }
            dst::BRIGHT => {
                self.load_all_palettes(); // the rest of 0x8A44 ...
                self.brightness_full(); // ... and 0x892C
                self.flow.front.task.wait(1, dst::STAGE);
            }
            dst::STAGE => {
                if self.flow.front.demo.runs <= 1 {
                    self.flow_demo_stage(true);
                    self.flow_demo_stage_tail();
                    self.flow.front.task.wait(2, dst::MAP);
                } else {
                    self.flow_demo_stage(false); // 0xA9BE: mode 4, G.$28 = 0x12D6
                    self.flow.front.task.wait(1, dst::TAIL);
                }
            }
            dst::TAIL => {
                self.flow_demo_stage_tail();
                self.flow_demo_camera_y();
                self.flow.front.task.wait(1, dst::MAP);
            }
            dst::MAP => {
                self.flow_demo_map();
                if self.flow.front.demo.runs <= 1 {
                    self.flow_demo_hud();
                    // the first pass of Episode 1 (row 0) starts in this frame (its G.$0 is 1 in the record of the map),
                    // the other rows sleep 1 frame first (0x99EB6)
                    let early = self.flow.front.demo_counter == 0;
                    self.flow_demo_enter_loop(early);
                } else {
                    self.flow.front.task.wait(1, dst::HUD);
                }
            }
            dst::HUD => {
                self.flow_demo_hud();
                self.flow_demo_enter_loop(true);
            }
            dst::LOOP => {
                self.flow_demo_frame(); // 0x99EC4 .. 0x99EEA
                let d = &mut self.flow.front.demo;
                let borrow = d.left == 0; // subq.w #1, $757C(a5); bcc.b
                d.left = d.left.wrapping_sub(1);
                if borrow {
                    // 0x99EF6: sleep 100 frames, the last picture stays, then the fade out
                    self.flow.front.task.wait(FROZEN_FRAMES, dst::FADE_START);
                } else {
                    self.flow.front.task.wait((self.flow.speed & 0xF) as u16, dst::LOOP);
                }
            }
            dst::FADE_START => {
                // 0x87BA: all four banks, 16 steps with one frame between them (G.$44B = 0x10, G.$44C = 1)
                self.fade.delay = 1;
                self.fade.mask = 0x0F;
                self.fade.steps = 0x10;
                self.flow_demo_fade_step();
            }
            _ => self.flow_demo_fade_step(), // dst::FADE
        }
    }

    /// 0x8808: one step of the fade out (0x88B0), then the step counter; when it is 0 the fade is over (0x881E) and the
    /// routine ends: G.$26 bit 0 is cleared (0x99F08) and the attract task goes on in `st::DEMO_END`.
    fn flow_demo_fade_step(&mut self) {
        self.fade_step(false);
        self.fade.steps = self.fade.steps.wrapping_sub(1);
        if self.fade.steps == 0 {
            self.fade.mask = 0;
            self.w.video_ctrl &= !1; // andi.w #$fffe, $26(a5)
            self.flow.front.task.state = st::DEMO_END;
        } else {
            self.flow.front.task.wait(self.fade.delay, dst::FADE);
        }
    }

    /// 0x99F10 after the clear and 0xAE0 up to 0xA9BE, in two parts (in the runs after the first the video frame ends between
    /// them, after the fourth draw of the generator). Part 0: the DIP decode (0xA804), the rank (0x17F8A), the extra life
    /// selector (0x6B88), no continue (G.$4D7 = 1) and the three player objects (0x4D46 ...); part 1: the heroes that play, the
    /// random number generator, the row and the streams (0x9A0CE).
    fn flow_demo_setup(&mut self, part: u8) {
        if part == 0 {
            self.flow.b4cf = (self.flow.dip.dip_a & 0x10 != 0) as u8; // 0xA804
            if self.flow.dip.dip_c & 0x40 == 0 {
                self.flow.b4d7 = 1;
            }
            // 0xFF2E, 0xA836: the HUD score flags, G.$755A .., the scroll controller structs are zero in a fresh game
            self.rank.dip = self.flow.dip.dip_b; // 0x17F8A reads G.$777B
            self.rank_game_start(); // 0x17F8A (one rand())
            self.rank.started = true;
            self.flow.b4e5 = self.flow.extend_sel(); // 0x6B88
            self.flow.b4d7 = 1; // 0x99F9E: no continue
            for i in 0..3 {
                self.flow_boot_player(i); // 0x4D46, 0x4D5E, 0x4D76 (with who 0..2 and no hero yet: the row sets who, id and class later)
            }
            return;
        }
        // hero 1 and 2 (and 3 in the 3 player layout) are present (+0 = 1, G.$4CD bits); 0x18C88 is what the first update
        // of an object in mode 0 does (`player_spawn_stage_start`)
        let n = if self.flow.b4cf != 0 { 2 } else { 3 };
        for i in 0..n {
            self.players[i].b.live = 1;
            self.flow.started |= 1 << i;
        }
        self.flow.chosen = self.flow.started; // G.$4E4 = G.$4CD
        self.flow.speed = 1; // G.$4D6
        self.w.rng = 0x1C3; // G.$14 (G.$4 .. G.$E are 0)
        let row = self.flow_demo_row();
        self.w.stage = row.stage; // G.$4D9
        self.cam.s2.area = row.area; // G.$786
        self.flow.front.demo.left = row.frames; // G.$757C
        for (i, h) in row.heroes.iter().enumerate().take(n) {
            self.flow_demo_hero(i, *h);
        }
        self.w.frame = 0; // G.$0, G.$4DA
        self.flow.frame = 0;
    }

    /// 0xAC6A, the call of 0xA9BE (the stage setup: palettes, pools, mode 4, G.$28 = 0x12D6). `camera`: the scroll2 camera is
    /// put at the start of the area now (in the first run the recording shows it in the frame of the mode byte).
    fn flow_demo_stage(&mut self, camera: bool) {
        self.flow_stage_setup(); // 0xA9BE (0x1DA10, the player scripts, is not ported)
        self.sprites.splash_timer = 1; // 0x49A8 (in 0xAA0E): the water splash timer
        self.sprites.splash_phase = 0;
        if camera {
            self.flow_demo_camera_y();
        }
    }

    /// The camera of the start of 0x2228A (the map is drawn later).
    fn flow_demo_camera_y(&mut self) {
        let map = &self.assets.stages.area(self.w.stage, self.cam.s2.area).scroll2;
        let (x, y) = (map.init_x, map.init_y);
        self.cam.s2.cam_x = x;
        self.cam.s2.cam_y = y;
    }

    /// 0xAC6E ..: mode 0xC, the player count, 100 HP for everybody, the sound codes and the music.
    fn flow_demo_stage_tail(&mut self) {
        self.flow.mode = 0xC; // move.b #$C, $4D0(a5)
        self.flow_count_players(); // 0x655E
        for p in self.players.iter_mut() {
            // 0x4DC2 for all three
            p.b.hp = 100;
            p.b.hp_prev = 100;
            p.b.hp_bar = 100;
        }
        self.rank.b7b4 = 0; // clr.b $7B4(a5)
        self.sound(0xFF00); // 0xA5D2
        if self.w.stage == 0 {
            self.sound(1); // 0xA63C with d0 = 1
        } else {
            self.flow_area_music(); // 0xA55A
        }
    }

    /// 0x99F10 from 0x2228A on: the scroll controllers start in the event the table gives for the stage and area, the
    /// rank is not counted (G.$5EC, G.$5EE = 0, G.$602 = 1: frozen).
    fn flow_demo_map(&mut self) {
        let t = &self.assets.frontend.text;
        let first = t.u8(START_EVENTS, 4 * self.w.stage as u32 + self.cam.s2.area as u32);
        self.flow_demo_camera_init(first); // 0x2228A
        self.rank.aggression = 0; // G.$5EC
        self.w.level = 0; // G.$5EE
        self.rank.freeze = 0x100; // G.$602 (a byte write: move.b #1, $602(a5), the high byte of the word)
    }

    /// The HUD (0x162AE), the area music (0xA55A, a second time) and the end of the setup: Episode 6 (stage 5) kills the INSERT
    /// COIN blink task (slot 0x20, jsr 0x92E, 0x99E8C). The task then sleeps before each pass (0x99EB6: G.$4D6 and 15 frames);
    /// `now` runs the first pass in this frame.
    fn flow_demo_hud(&mut self) {
        self.hook_162ae(); // 0x162AE
        self.flow_area_music(); // jmp 0xA55A
        if self.w.stage == 5 {
            self.flow.front.blink.kill();
        }
    }

    fn flow_demo_enter_loop(&mut self, now: bool) {
        if now {
            self.flow.front.task.state = dst::LOOP;
        } else {
            self.flow.front.task.wait((self.flow.speed & 0xF) as u16, dst::LOOP);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    /// A game whose front end tables hold made up bytes (the real ones are read from the ROM at run time).
    fn game_with(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut a = AssetPack::default();
        a.frontend.text.blobs = blobs;
        Game::new(Box::leak(Box::new(a)))
    }

    fn digits(g: &Game) -> [u16; 4] {
        [0, 1, 2, 3].map(|k| g.vram.scroll1[Game::flow_cell(0x90A000 + 0x80 * k)].code)
    }

    /// 0x9A146: a record plays for its count in frames (the first one for one frame less: its count is loaded by the setup, before
    /// the first call), the next record is taken in the frame the count reaches 0, and that frame writes the hex digits FFFF; the other
    /// frames write the hero number times 4.
    #[test]
    fn a_stream_plays_its_records() {
        // records (buttons, frames): (1, 3) (0x12, 2) (0, 1) (8, 5)
        let mut g = game_with(vec![(0xC869A, vec![1, 3, 0x12, 2, 0, 1, 8, 5])]);
        for who in 0..3 {
            g.flow.front.demo.rec[who] = (0xC869A, 3); // as flow_demo_hero: the pointer and the first count
        }
        let mut pads = Vec::new();
        let mut shown = Vec::new();
        for _ in 0..7 {
            g.flow_demo_playback();
            pads.push(g.flow.front.demo.pad[0]);
            shown.push(digits(&g));
        }
        assert_eq!(pads, [1, 1, 0x12, 0x12, 0, 8, 8]);
        // all three heroes play the same stream; the last one (hero 2) writes the digits last: FFFF in the frames a record
        // changed (the 3rd, 5th and 6th call), else its number times 4 (8)
        let f = [0xF; 4];
        assert_eq!(shown, [[0, 0, 0, 8], [0, 0, 0, 8], f, [0, 0, 0, 8], f, f, [0, 0, 0, 8]]);
    }

    /// The display after a record change shows FFFF (the delay loop of 0x9A1B2 leaves d0 = 0xFFFF) when the last hero changed.
    #[test]
    fn a_new_record_shows_four_f() {
        let mut g = game_with(vec![(0xC869A, vec![1, 1, 2, 9])]);
        g.flow.b4cf = 1; // two heroes: hero 1 writes last (its number is 1, times 4)
        g.flow.front.demo.rec[0] = (0xC869A, 5);
        g.flow.front.demo.rec[1] = (0xC869A, 1); // changes at once
        g.flow_demo_playback();
        assert_eq!(g.flow.front.demo.pad, [1, 2, 0]);
        assert_eq!(digits(&g), [0xF, 0xF, 0xF, 0xF]);
        g.flow_demo_playback();
        assert_eq!(digits(&g), [0, 0, 0, 4], "the hero number times 4 in the 2 player layout: hero 1 is the last");
    }

    /// 0x9A03C: the row is the counter (G.$757A) as a table index; a counter of 6 or more is row 0 and resets the counter. The 2
    /// player layout reads the other table and has two heroes. (The tables of the ROM are one block: 6 offsets of the 3 player rows, 6
    /// of the 2 player rows, then the rows.)
    #[test]
    fn the_counter_chooses_the_row() {
        let mut b = vec![0u8; 24];
        for (heroes, offsets_at) in [(3usize, 0usize), (2, 12)] {
            for r in 0..6 {
                let row_len = 4 + 6 * heroes;
                // the offset is from the start of its table (ROWS_3P or ROWS_2P)
                let at = b.len() - offsets_at;
                b[offsets_at + 2 * r..offsets_at + 2 * r + 2].copy_from_slice(&(at as u16).to_be_bytes());
                let mut row = vec![0u8; row_len];
                row[0] = r as u8; // stage
                row[1] = 10 + r as u8; // area
                row[2..4].copy_from_slice(&0x4B0u16.to_be_bytes());
                for h in 0..heroes {
                    let p = 4 + 6 * h;
                    row[p..p + 2].copy_from_slice(&(4 * (r + h) as u16).to_be_bytes()); // stream index
                    row[p + 2..p + 4].copy_from_slice(&(((h as u16) << 8) | (h as u16 + 1)).to_be_bytes()); // who, id
                    row[p + 4..p + 6].copy_from_slice(&(4 * (h as u16 + 1)).to_be_bytes()); // class
                }
                b.extend(row);
            }
        }
        let mut g = game_with(vec![(ROWS_3P, b)]);
        for c in 0..6u16 {
            g.flow.front.demo_counter = c;
            let row = g.flow_demo_row();
            assert_eq!((row.stage, row.area, row.frames), (c as u8, 10 + c as u8, 0x4B0), "counter {c}");
            assert_eq!(row.heroes[2], (4 * (c + 2), 0x0203, 12), "the third hero of the 3 player layout");
        }
        g.flow.front.demo_counter = 6;
        assert_eq!(g.flow_demo_row().stage, 0, "the demo after the fifth is row 0 (Episode 1)");
        assert_eq!(g.flow.front.demo_counter, 0, "and the counter goes back to 0");
        g.flow.b4cf = 1;
        g.flow.front.demo_counter = 3;
        let row = g.flow_demo_row();
        assert_eq!((row.stage, row.heroes[1].1, row.heroes[2]), (3, 0x0102, (0, 0, 0)), "the 2 player table has two heroes");
    }

    /// 0x99F10: the RAM clear takes the game state and keeps the inputs, the video memory, the palettes, the fade state, the
    /// attract and system tasks, the settings and the high score table.
    #[test]
    fn the_ram_clear_keeps_what_the_original_keeps() {
        let mut g = game_with(Vec::new());
        // kept
        g.w.input = [1, 2, 3];
        g.w.system = 9;
        g.w.coins = 4;
        g.w.layer_ctrl = 0x1396;
        g.vram.scroll1[7].code = 0x41;
        g.pal.words[5] = 0xF123;
        g.fade.mask = 3;
        g.hud.hiscores[0].score = 777;
        g.flow.dip.dip_b = 0x55;
        g.flow.front.attract_state = 6;
        g.flow.front.sys.credits = 2;
        g.flow.front.demo_counter = 4;
        // taken
        g.w.rng = 0x1234;
        g.w.frame = 99;
        g.w.stage = 5;
        g.flow.mode = 0xC;
        g.flow.b4d7 = 1;
        g.flow.front.select.taken = 3;
        g.players[0].b.live = 1;
        g.players[0].lives = 3;
        g.rank.w4f6 = 40;
        g.cam.s2.cam_x = 300;
        g.enemies.alloc();
        g.flow_demo_clear_ram();
        assert_eq!(g.w.input, [1, 2, 3]);
        assert_eq!((g.w.system, g.w.coins, g.w.layer_ctrl), (9, 4, 0x1396));
        assert_eq!(g.vram.scroll1[7].code, 0x41);
        assert_eq!(g.pal.words[5], 0xF123);
        assert_eq!(g.fade.mask, 3);
        assert_eq!(g.hud.hiscores[0].score, 777);
        assert_eq!(g.flow.dip.dip_b, 0x55);
        assert_eq!((g.flow.front.attract_state, g.flow.front.sys.credits, g.flow.front.demo_counter), (6, 2, 4));
        assert_eq!((g.w.rng, g.w.frame, g.w.stage, g.flow.mode, g.flow.b4d7), (0, 0, 0, 0, 0));
        assert_eq!(g.flow.front.select.taken, 0);
        assert_eq!((g.players[0].b.live, g.players[0].lives, g.rank.w4f6, g.cam.s2.cam_x), (0, 0, 0, 0));
        assert_eq!(g.enemies.free_count(), 24, "the pools are empty again");
    }
}
