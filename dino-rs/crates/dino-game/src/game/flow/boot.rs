//! Power on to the title: the boot task 0xA69A, the boot text screens 0x9DCF6 / 0x9DD80, the head of
//! the attract task 0x97C20 / 0x97D84, the title screen 0x9A3CE with its INSERT COIN blink task
//! 0x98874, the press start screen 0x983EE / 0x978EE, the green background colour 0x980B0 and the
//! Capcom sign 0x110C. Owner: flow / HUD port (M1-03). Spec: `D4d-front-end-and-story.md` s2, s3.3,
//! s3.9, s13 A1, A2, A4.
//!
//! The slot 0x10 task of the original (boot task, then the attract task that never returns) is one
//! [`Scene`] whose state is the position in the script (`st`); the title (600 passes of its frame
//! loop) is a part of the same task. Slot 0x20 holds the blink task or the press start task (two
//! scenes, one of them alive). Everything runs from `Game::flow_attract_task`, in slot order.
//!
//! # Timing
//!
//! A sleep of n frames runs again n frames later (`scene.rs`). The original's long actions (a clear
//! of all layers 0xAE0 takes about 1.3 video frames of CPU time, the EEPROM read about 26) are written
//! as sleeps of the measured length, so every visible change falls on the video frame of the
//! MAME recording (`re/out/traces/attract.mame.trc`, record k = video frame k + 2, the port's tick k).
//! Two places run past the end of a video frame and are written to match what the recording shows:
//! the first title pass follows the setup in the same frame, and the fourth title block is half
//! drawn at the end of the setup frame (see `title_setup`).

use crate::game::flow::scene::Scene;
use crate::game::vram::PaletteRam;
use crate::game::Game;

/// Script positions of the slot 0x10 task (the attract states of `attract.rs` are here too).
pub(super) mod st {
    /// Reset code 0x41A / 0xAA944: 41 video frames.
    pub const RESET: u16 = 0;
    /// The reset code's text palette (frame 2).
    pub const RESET_PALETTE: u16 = 14;
    /// Sound code 0xFF0B of the reset code (frame 41).
    pub const SOUND: u16 = 1;
    /// 0xA69A: hi-score table, EEPROM read.
    pub const ENTRY: u16 = 2;
    /// 0xA69A after the EEPROM read: sound reset, clear, DIP bytes, tasks (frame 68).
    pub const CLEAR: u16 = 3;
    /// The rest of the clear 0xAE0 (frame 69).
    pub const CLEAR_END: u16 = 13;
    /// 0xA69A: the three player inits, then 0x9DCF6 text 1 (frame 70).
    pub const TEXT1: u16 = 4;
    /// 0x9DD80 after text 1, then text 2.
    pub const TEXT1_WAIT: u16 = 5;
    /// 0x9DD80 after text 2, the end of the boot screens (frame 312).
    pub const TEXT2_WAIT: u16 = 6;
    /// 0x97C20: after the first two frames, palettes and cameras.
    pub const ATTRACT: u16 = 7;
    /// 0x97C46: the pass of the attract task (layer control 0x18D6), the state routine.
    pub const PASS: u16 = 8;
    /// 0x97D84 after the clear: brightness, then the title setup 0x9A3CE.
    pub const TITLE_START: u16 = 9;
    /// 0x9A3CE after its clear: palettes, blink task, layer control 0x1396, the blocks.
    pub const TITLE_SETUP: u16 = 10;
    /// 0x9A3CE after the blocks: the logo objects and the first pass of the frame loop.
    pub const TITLE_LOGO: u16 = 11;
    /// 0x9A514: the frame loop (one pass per frame).
    pub const TITLE_LOOP: u16 = 12;
    /// The title returned: the rest of the routine of the attract state.
    pub const AFTER_TITLE: u16 = 15;
    /// The profile cards, the demo, the slides, the ranking and the movie returned (`attract.rs`).
    pub const PROFILES_END: u16 = 16;
    pub const DEMO_END: u16 = 17;
    pub const SLIDES_END: u16 = 18;
    pub const RANK_START: u16 = 19;
    pub const RANK_END: u16 = 20;
    pub const MOVIE_END: u16 = 21;
    /// The ranking shown after a game over (0x97C94), after its clear.
    pub const GAME_OVER_RANKING: u16 = 22;
    /// A wait of more than 255 frames in chained sleeps.
    pub const LONG_WAIT: u16 = 23;
}

/// What differs between the runs of the title (the original's CPU time falls differently on each
/// pass; measured in `attract3.mame.trc`): the frames from the setup (layer control 0x1396) to the
/// first pass of the frame loop (0 to 2), how many of the 512 cells of the four blocks are written
/// when the setup frame ends (the rest follows in the next frame) and the frame the INSERT COIN
/// blink task first runs, after the setup (0 or 1). The runs after the table use the values of the
/// last pass of the same kind of state routine.
const TITLE_RUNS: [(u8, u16, u8); 10] =
    [(1, 485, 1), (0, 512, 0), (2, 269, 1), (2, 268, 1), (2, 221, 1), (1, 512, 0), (2, 268, 1), (2, 260, 1), (2, 221, 1), (1, 512, 0)];

/// Frames the title shows (the counter G.-$7022 runs to 600, 0x9A536).
const TITLE_FRAMES: u16 = 0x258;

/// The words of scroll3's palette bank the brightness loop 0x8982 has done when the frame of the press start call ends (M1-15).
const PRESS_CALL_CUT: usize = 154;
/// The same, and the cells of scroll3 the clear has done in the frame it clears scroll2 in, when the call comes from the demo.
const PRESS_CALL_CUT_DEMO: usize = 139;
const PRESS_CALL_CLEAR3_DEMO: usize = 3584;

impl Game {
    // ------------------------------------------------------------------ power on

    /// Power on: the state the reset code (0x41A, 0xAA944) leaves in the video memory by video
    /// frame 27, the boot task created in slot 0x10 and the other tasks not yet created.
    /// (Frames 0 to 24 of the original are the reset code clearing and testing the video RAM: the
    /// first records differ from the port in a few cells until then.)
    pub fn flow_boot_start(&mut self) {
        self.flow.front = Default::default();
        // the reset code: RAM cleared (the splash timer and phase are 0 until 0x49A8), G.$26 = 0xE
        self.sprites.splash_timer = 0;
        self.sprites.splash_phase = 0;
        self.w.video_ctrl = 0x0E; // 0x460
        // the settings are cooked at frame 68 (0xAC794); before that the bytes are 0
        self.flow.dip = super::Settings { dip_a: 0, dip_b: 0, dip_c: 0 };
        // scroll1 blank with attribute 0, scroll2 blank except cell 0 (a 0x20 / 0 left by the test),
        // scroll3 blank.
        self.vram.scroll1.iter_mut().for_each(|c| (c.code, c.attr) = (0x20, 0));
        self.vram.clear2();
        self.vram.scroll2[0] = dino_assets::stage::Cell { code: 0x20, attr: 0 };
        self.vram.clear3();
        self.flow.front.task = Scene::start(st::RESET);
    }

    /// The boot task 0xA69A and the attract task 0x97C20 (slot 0x10).
    pub fn flow_boot_task(&mut self) {
        if !self.flow.front.task.tick() {
            return;
        }
        loop {
            match self.flow.front.task.state {
                st::RESET => {
                    // 0x41A .. 0x4FE: RAM clear, video registers (G.$26 = 0xE, G.$28 = 0x12D6),
                    // 0xAE0, 0xAA944, then the boot task is created.
                    self.flow.front.task.wait(2, st::RESET_PALETTE);
                    return;
                }
                st::RESET_PALETTE => {
                    // scroll1 palette 0 is the white to grey text palette at full brightness
                    let t = &self.assets.frontend.text;
                    let mut words = [0xF000u16; 16];
                    for (i, w) in words.iter_mut().take(7).enumerate() {
                        *w = t.u16(0x9DDB8, 2 * i as u32); // the first palette of the boot text table
                    }
                    self.pal.copy(PaletteRam::SCROLL1, &words);
                    self.flow.front.task.wait(39, st::SOUND);
                    return;
                }
                st::SOUND => {
                    self.sound(0xFF0B); // the reset code resets the sound CPU
                    // 0xA69A starts in this frame: the hi-score table (0x1504E) is the first thing it does
                    self.hud_hiscore_init();
                    self.flow.front.task.wait(1, st::ENTRY);
                    return;
                }
                st::ENTRY => {
                    // 0xA69A: the row scroll fill 0xA7B8, the attract state byte and the game over
                    // flag (0xA7FA), the EEPROM read 0xAC794 (about 26 frames; the settings are
                    // cooked after it, see `st::CLEAR`).
                    self.flow.front.attract_state = 0; // clr.w -$7030(a5)
                    self.flow.front.game_over_flag = 0; // clr.b -$702E(a5)
                    self.flow.front.task.wait(26, st::CLEAR);
                    return;
                }
                st::CLEAR => {
                    // 0xA6BC..0xA73C: debug flag (not ported), sound output flag, sound codes
                    // 0xFF0B and 0xFF00 (0xA5D2), the clear of all layers 0xAE0 (1.3 frames),
                    // region word G.$32 = 2, DIP decode 0xA804.
                    self.flow.dip = self.flow.config.cook(); // 0xAC794 (G.$777A, G.$777B, G.$777C)
                    self.sound(0xFF0B);
                    self.sound(0xFF00);
                    // 0xAE0 takes 1.3 frames: scroll1 is blank (code 0x20, attribute 6) up to cell
                    // 3884 when the frame ends, the rest of the layers is cleared in the next one
                    for c in self.vram.scroll1.iter_mut().take(3884) {
                        (c.code, c.attr) = (0x20, 6);
                    }
                    self.flow.front.task.wait(1, st::CLEAR_END);
                    return;
                }
                st::CLEAR_END => {
                    self.flow_clear_screen();
                    self.flow.region = super::Region(2);
                    self.flow.in_game = 0;
                    self.flow.started = 0;
                    self.flow.b4cf = (self.flow.dip.dip_a & 0x10 != 0) as u8;
                    if self.flow.dip.dip_c & 0x40 == 0 {
                        self.flow.b4d7 = 1;
                    }
                    // 0x9CFC6 (coin and chute modes) and the creation of the system task (slot 0);
                    // the skip wait task 0x9E38C (slot 0xF0) is not ported. The rest runs in
                    // frame 70 (the clear ends frame 69). The original's pass runs across the sample of record 69: G.$32 and the
                    // coin modes G.$470 / G.$471 show in record 70 there, together with the system task's first pass (G.$21), which
                    // the port runs in tick 70 (allow/m1.txt section 3).
                    self.flow_sys_init();
                    self.flow.front.task.wait(1, st::TEXT1);
                    return;
                }
                st::TEXT1 => {
                    // 0x4D46 / 0x4D5E / 0x4D76: the three player objects (HP 100, facing 1, power
                    // factor with one rand() each, extra lives from the settings).
                    for i in 0..3 {
                        self.flow_boot_player(i);
                    }
                    // 0x9DCF6: fade counter = 15, brightness 0 (0x8938), the sleep 0xA76 of the
                    // original falls inside the video frame here (the recording shows the palette
                    // and the text in the same frame), the boot palette, text 1.
                    self.fade.mask = 0x0F;
                    self.brightness_low();
                    self.boot_palette();
                    self.boot_text(0x9DDB2); // text 1: name, date code, region
                    self.flow.front.task.state = st::TEXT1_WAIT;
                    self.flow.front.task.sub = 0;
                }
                st::TEXT1_WAIT => {
                    if !self.boot_wait() {
                        return;
                    }
                    self.flow_clear_scroll1(); // 0xB52
                    self.boot_text(0x9DDAC); // text 2: the legal warning
                    self.flow.front.task.state = st::TEXT2_WAIT;
                }
                st::TEXT2_WAIT => {
                    if !self.boot_wait() {
                        return;
                    }
                    self.flow_clear_scroll1(); // jmp 0xB52, then the boot task goes on:
                    self.flow_scene_palettes(0); // 0xA780: palette set 0 (0x89E6)
                    self.brightness_full(); // 0x892C
                    // 0xA78E: DIP C bit 7 clear would start the test mode (not ported), else
                    // jmp 0x97C20: sleep 2 (0xA7A)
                    self.flow.front.task.wait(2, st::ATTRACT);
                    return;
                }
                st::ATTRACT => {
                    // 0x97C26: G.$4CC = 0, palette set 0 and full brightness, the scroll1 and
                    // scroll2 cameras to 0 (G.$6C4, G.$744), sleep 1 (0xA76)
                    self.flow.in_game = 0;
                    self.flow_scene_palettes(0);
                    self.brightness_full();
                    self.cam.cam1_x = 0;
                    self.cam.s2.cam_x = 0;
                    self.flow.front.task.wait(1, st::PASS);
                    return;
                }
                st::PASS => {
                    self.flow_attract_pass();
                    if self.flow.front.task.sleeping() > 0 {
                        return;
                    }
                }
                st::TITLE_START => {
                    self.brightness_full(); // 0x97D98: 0x892C
                    // 0x9A3CE: the pools and lists are reset, the title counter and the cameras
                    // to 0, a full clear (1.3 frames)
                    self.title_reset();
                    self.flow_attract_busy(2, 2, st::TITLE_SETUP);
                    return;
                }
                st::TITLE_SETUP => {
                    self.title_setup();
                    if self.flow.front.task.sleeping() > 0 {
                        return;
                    }
                }
                st::TITLE_LOGO => {
                    self.title_logo();
                    if self.flow.front.task.sleeping() > 0 {
                        return;
                    }
                }
                st::AFTER_TITLE => {
                    self.flow_attract_after_title();
                    if self.flow.front.task.sleeping() > 0 {
                        return;
                    }
                }
                s if super::profiles::pst::owns(s) => {
                    self.flow_profile_script(); // the profile cards (`profiles.rs`)
                    if self.flow.front.task.sleeping() > 0 {
                        return;
                    }
                }
                s if super::movie::mst::owns(s) => {
                    self.flow_movie_script(); // the opening movie (`movie.rs`)
                    if self.flow.front.task.sleeping() > 0 {
                        return;
                    }
                }
                s if super::demo::dst::owns(s) => {
                    self.flow_demo_script(); // the demo (`demo.rs`)
                    if self.flow.front.task.sleeping() > 0 {
                        return;
                    }
                }
                s if super::slides::sst::owns(s) => {
                    self.flow_slides_script(); // the story slides (`slides.rs`)
                    if self.flow.front.task.sleeping() > 0 {
                        return;
                    }
                }
                s if super::ranking::rst::owns(s) => {
                    self.flow_ranking_script(); // the ranking screen (`ranking.rs`)
                    if self.flow.front.task.sleeping() > 0 {
                        return;
                    }
                }
                st::PROFILES_END => {
                    self.flow_attract_profiles_end();
                    return;
                }
                st::DEMO_END => {
                    self.flow_attract_demo_end();
                    return;
                }
                st::SLIDES_END => {
                    self.flow_attract_slides_end();
                    return;
                }
                st::GAME_OVER_RANKING => {
                    self.flow_attract_game_over_ranking();
                    return;
                }
                st::RANK_START => {
                    self.flow_attract_ranking_start();
                    return;
                }
                st::RANK_END => {
                    self.flow_attract_ranking_end();
                    if self.flow.front.task.sleeping() > 0 {
                        return;
                    }
                }
                st::MOVIE_END => self.flow_attract_movie_end(), // goes on with the next pass
                st::LONG_WAIT => {
                    self.flow_attract_long_wait();
                    return;
                }
                _ => {
                    if self.title_pass() {
                        return;
                    }
                }
            }
        }
    }

    /// The part of the player init the boot task does (0x4D46 / 0x4D5E / 0x4D76 with 0x4D8E and
    /// 0x4DD6): a cleared object with HP 100, facing 1 and +$10A = 0x33, the power factor
    /// (0x11738: one `rand()`), the index and the extra lives of the settings.
    pub(super) fn flow_boot_player(&mut self, i: usize) {
        let mut p = crate::game::player::Player::default();
        p.b.hp = 100;
        p.b.hp_prev = 100;
        p.b.hp_bar = 100;
        p.b.facing = 1;
        p.b10a = 0x33;
        p.b.who = i as u8;
        p.b.serial = (i as u8) << 4;
        self.players[i] = p;
        self.player_power_factor(i);
        self.flow_lives_init(i);
    }

    // ------------------------------------------------------------------ boot screens

    /// 0x9DD0E: the scroll1 palettes 0 to 2 get 8 colours each from the 48 bytes at
    /// 0x9DDB8 + G.$32 * 0x20 (blue text for the World set).
    fn boot_palette(&mut self) {
        let t = &self.assets.frontend.text;
        let base = 0x9DDB8 + 0x20 * self.flow.region.0 as u32;
        for k in 0..3u32 {
            let words: Vec<u16> = (0..8).map(|i| t.u16(base, k * 16 + 2 * i)).collect();
            self.pal.copy(PaletteRam::SCROLL1 + 16 * k as usize, &words);
        }
    }

    /// 0x9DD2C / 0x9DD56: the text of the region (offset table entry G.$32 * 2, entry 0 when the
    /// region word is above 2) printed by 0xBB6.
    fn boot_text(&mut self, table: u32) {
        let t = &self.assets.frontend.text;
        let r = if self.flow.region.0 > 2 { 0 } else { self.flow.region.0 as u32 };
        let a = (table as i32 + t.i16(table, 2 * r) as i32) as u32;
        let s = t.slice_from(a);
        self.text_bytes(s, true);
    }

    /// 0x9DD80: sleep 30 frames (not skippable), then up to 91 more frames in steps of 1; an
    /// attack or jump (mask 0x30) of player 1 or 2 ends it at once. True when the wait is over.
    fn boot_wait(&mut self) -> bool {
        loop {
            match self.flow.front.task.sub {
                0 => {
                    self.flow.front.task.sleep(30);
                    self.flow.front.task.sub = 1;
                    return false;
                }
                1 => {
                    self.flow.front.task.count = 0x5A; // d5
                    self.flow.front.task.sub = 2;
                }
                2 => {
                    if (self.w.input[0] | self.w.input[1]) & 0x30 != 0 {
                        self.flow.front.task.sub = 0;
                        return true;
                    }
                    self.flow.front.task.sleep(1);
                    self.flow.front.task.sub = 3;
                    return false;
                }
                _ => {
                    // dbra d5: the loop ends when d5 passes 0
                    let b = &mut self.flow.front.task;
                    b.count = b.count.wrapping_sub(1);
                    if b.count == 0xFFFF {
                        b.sub = 0;
                        return true;
                    }
                    b.sub = 2;
                }
            }
        }
    }

    // ------------------------------------------------------------------ title 0x9A3CE

    /// The calls every picture task makes first: 0x49A8 (draw list), 0x4A18 / 0x4A46 (the lists
    /// the combat code fills each frame: empty here), 0x4C48 / 0x4C76 (pool A and B), 0x12BE4 (floor
    /// table of the area), 0xFBE8 (spawn lists).
    pub(super) fn title_pools_reset(&mut self) {
        self.draw_list.clear();
        self.sprites.splash_timer = 1; // 0x49B4 (the phase -$7124 is not reset)
        self.enemies = crate::game::pool_a_init();
        self.props = crate::game::pool_b_init();
        self.hook_12be4();
        self.spawn_reset();
    }

    /// 0x9A3CE up to the first clear: pool resets, G.-$701C = 0, G.$7E4 and G.$744 = 0, 0xAE0 (the title
    /// counter G.-$7022 is cleared later, in `title_setup`).
    pub(super) fn title_reset(&mut self) {
        // what differs in this run (measured, see `TITLE_RUNS`)
        let k = self.flow.front.title_runs as usize;
        self.flow.front.title_runs = self.flow.front.title_runs.saturating_add(1);
        self.flow.front.title_run = match TITLE_RUNS.get(k) {
            Some(&r) => r,
            None if self.flow.front.attract_state & 0x3E == 2 => (1, 512, 0),
            None => (2, 268, 1),
        };
        self.cam.s3.cam_x = 0; // G.$7E4
        self.cam.s2.cam_x = 0; // G.$744
        self.flow_attract_clear(); // 0xAE0 (about 1.3 frames)
        self.title_pools_reset();
    }

    /// 0x9A40E..0x9A4D8: palette set 0 (black), brightness, the 1 in 128 roll, the blink task, the
    /// layer control 0x1396 and the four green blocks of the background.
    ///
    /// The recording shows the frame ending while the blocks are being written: only the first
    /// `n` of their 512 cells (written block after block, row after row) are there, `title_logo`
    /// writes the rest in the next frame (`n` is in `TITLE_RUNS`).
    pub(super) fn title_setup(&mut self) {
        self.flow_scene_palettes(0); // 0x9A410
        self.brightness_full(); // 0x9A416
        // 0x9A41C: one chance in 128 calls 0x6A73A, which scans the vblank code for a patched word
        // and never finds one on this ROM (carry clear: nothing happens). Only the rand() matters.
        let _roll = self.rand();
        // 0x9A448: the INSERT COIN blink task in slot 0x20. It runs in the frame of the setup (state
        // 2) or the next one, as the recording shows; the attract task is still busy past the end
        // of the setup frame, the scan of the task slots starts over.
        let (delay, mut cells, blink_at) = self.flow.front.title_run;
        if self.flow.front.sys.free_play && cells < 512 {
            // the system task of free play does more each frame (the start detection): the frame
            // ends 10 cells earlier in the blocks (measured in the first title run)
            cells -= 10;
        }
        let mut blink = Scene::start(0);
        if blink_at > 0 {
            blink.sleep(blink_at as u16 + 1); // the scene is ticked in this frame too
        }
        self.flow.front.blink = blink;
        self.flow.front.title = 0; // G.-$7022, -$7020, -$701E
        self.w.layer_ctrl = 0x1396; // 0x9A468
        for at in [0x90F000, 0x90F020, 0x90F400, 0x90F420] {
            self.title_block(at, 0x9A5E6, 0x9A6E6); // 0x98B86: 16 x 8 cells
        }
        // the frame ends inside the blocks: the cells after the first `cells` are still blank
        for c in cells as u32..512 {
            let at = [0x90F000u32, 0x90F020, 0x90F400, 0x90F420][c as usize / 128];
            let (row, col) = ((c % 128) / 16, c % 16);
            let cell = &mut self.vram.scroll2[Self::title_cell2(at + col * 0x40 + row * 4)];
            cell.code = 0x5800;
            cell.attr = 0;
        }
        if cells >= 512 {
            // all blocks are there: the rest of the setup is in this frame, the first pass after `delay`
            self.title_logo_step();
            if delay == 0 {
                self.flow.front.task.state = st::TITLE_LOOP;
            } else {
                self.flow.front.task.wait(delay as u16, st::TITLE_LOOP);
            }
        } else {
            // the rest of the setup follows in the next frame
            self.flow.front.task.wait(1, st::TITLE_LOGO);
        }
    }

    /// One 16 x 8 block of tiles of the title / press start background (0x98B86).
    fn title_block(&mut self, at: u32, codes: u32, attrs: u32) {
        let t = &self.assets.frontend.text;
        let (c, a) = (t.slice_from(codes), t.slice_from(attrs));
        self.flow_block2(at, c, a, 16, 8);
    }

    fn title_cell2(a: u32) -> usize {
        ((a.wrapping_sub(0x90C000) / 4) & 0xFFF) as usize
    }

    /// 0x980B0: scroll2 palette 0 (all 16 colours) becomes the green of the background, the
    /// brightness nibble stays.
    fn title_green(&mut self) {
        for w in self.pal.bank_mut(2).iter_mut().take(16) {
            *w = (*w & 0xF000) | 0x392;
        }
    }

    /// 0x9A4DE..0x9A514: the rest of the blocks, the green colour, the two logo objects (variants 0
    /// and 1); the frame loop starts in the same frame or, in some runs, one frame later.
    fn title_logo(&mut self) {
        self.title_logo_step();
        if self.flow.front.title_run.0 >= 2 {
            self.flow.front.task.wait(self.flow.front.title_run.0 as u16 - 1, st::TITLE_LOOP);
        } else {
            self.flow.front.task.state = st::TITLE_LOOP;
        }
    }

    fn title_logo_step(&mut self) {
        for at in [0x90F000, 0x90F020, 0x90F400, 0x90F420] {
            self.title_block(at, 0x9A5E6, 0x9A6E6); // the cells the frame boundary cut off
        }
        self.title_green(); // 0x9A4DE
        self.enemy_spawn_logo(0); // 0x9A4E4
        self.enemy_spawn_logo(1); // 0x9A4FC
    }

    /// One pass of the title frame loop (0x9A51A..0x9A5B4), after its sleep. True when the task
    /// sleeps (the caller returns), false when the title is over and the script goes on.
    fn title_pass(&mut self) -> bool {
        self.screen_shake(); // 0x222D6
        self.pool_b_update(); // 0x23604
        self.enemies_update(); // 0x23420
        self.build_sprite_list(); // 0x13FA6
        let n = self.flow.front.title.wrapping_add(1);
        self.flow.front.title = n;
        if n as i16 >= TITLE_FRAMES as i16 {
            // 0x9A5B8: back to the routine of the attract state (`attract.rs`)
            self.flow.front.task.state = st::AFTER_TITLE;
            return false;
        }
        if n == 30 {
            // the two small lines, then the Capcom sign (0x110C at column 0x10, row 0x1A)
            let s = self.assets.frontend.text.slice_from(0x97F64);
            self.text_bytes(s, true);
            self.flow_capcom_sign(0x10, 0x1A);
        }
        if n == 0x168 {
            // the blink task ends, the licence block replaces the screen text
            self.flow.front.blink.kill();
            self.flow_clear_scroll1(); // 0xB52
            let s = self.assets.frontend.text.slice_from(0x97F9E);
            self.text_bytes(s, true);
            // two cells at 0x90A6E8 (0x9A582); for the regions other than ETC a third and a
            // copyright line
            self.hud_put(0x90A6E8, 0x40, 0);
            self.hud_put(0x90A6EC, 0x40, 0);
            if self.flow.region.0 != 2 {
                self.hud_put(0x90A6F0, 0x40, 0);
                let s = self.assets.frontend.text.slice_from(0x98094);
                self.text_bytes(s, true);
            }
        }
        self.flow.front.task.wait(1, st::TITLE_LOOP); // 0x9A514
        true
    }

    /// 0x110C: the Capcom sign, 16 columns of 3 cells (tile codes from 0x113C, attribute 0x0A) with
    /// the top left cell at column `col`, row `row` (the address of 0xA7218: base 0x90A000 plus
    /// `col * 0x80 + row * 4`, wrapped into 0x4000).
    pub fn flow_capcom_sign(&mut self, col: u16, row: u16) {
        let a0 = 0x90A000 + ((((col & 0xFF) << 7) + ((row & 0xFF) << 2)) & 0x3FFF) as u32;
        for k in 0..16u32 {
            for j in 0..3u32 {
                let code = self.assets.frontend.text.u16(0x113C, 2 * (3 * k + j));
                self.hud_put(a0 + 0x80 * k + 4 * j, code, 0x0A);
            }
        }
    }

    // ------------------------------------------------------------------ slot 0x20 tasks

    /// 0x98874: the INSERT COIN blink task: print for 30 frames, erase for 30 frames, forever
    /// (FREE PLAY when G.$46B is set, which the coin mode setup 0x9CFC6 takes from the settings).
    pub fn flow_blink_task(&mut self) {
        if !self.flow.front.blink.tick() {
            return;
        }
        let a = if self.flow.front.sys.free_play { 0x988B3 } else { 0x988A4 };
        let s = self.assets.frontend.text.slice_from(a);
        if self.flow.front.blink.state == 0 {
            self.text_bytes(s, true); // 0xBB6
            self.flow.front.blink.wait(30, 1); // 0xA7E
        } else {
            self.text_bytes(s, false); // 0xBAE
            self.flow.front.blink.wait(30, 0);
        }
    }

    // ------------------------------------------------------------------ press start

    /// 0x983EE: the press start screen, called by the system task when a credit is there (M1-04
    /// ends the attract task before it): clear all, palette set 0, the screen task 0x978EE in slot
    /// 0x20, layer control 0x12D6, clear scroll1, the scroll2 / scroll3 base shadows (not modelled),
    /// layer control 0x1396, brightness.
    ///
    /// The call takes the system task 3 frames (M1-15; before it was written in one frame): the clear 0xAE0 needs 1.3 frames
    /// of CPU time, so the recording shows the sprite buffers and scroll1 blank after the frame of the call, scroll2 (and
    /// scroll3 up to a cell) in the next frame, and in the third frame the rest of scroll3 and everything after the clear (the
    /// palette set 0, the new task, the layer control, the second clear of scroll1, the brightness). The staging is the one of
    /// `flow_attract_clear` (`front.press_call` counts the frames, the system task runs them at the start of its pass).
    /// Records 458 to 460 of `flow_c-title`, 4008 to 4010 of `flow_m113-coin`.
    #[allow(dead_code)] // the system task calls it (M1-04); the press start test of `mame_check.rs` does now
    pub fn flow_press_start_screen(&mut self) {
        // The cuts by the video frame boundary depend on how late in the frame the system task runs: in the demo (the interrupt
        // has built a game frame) later than in the title (`flow_c-title`: scroll3 is empty there, the palette cut is 154 words;
        // `flow_m113-coin`: the first part of the clear of scroll3 ends at cell 3584 at the latest and the palette cut is 139).
        let from_demo = self.flow.front.stub == super::attract::Stub::Demo;
        self.flow_attract_clear(); // 0xAE0 (the first frame of it)
        if from_demo {
            self.flow.front.clear3_cut = PRESS_CALL_CLEAR3_DEMO;
        }
        self.flow.front.press_cut = if from_demo { PRESS_CALL_CUT_DEMO } else { PRESS_CALL_CUT };
        self.flow.front.blink.kill(); // the new task replaces the one of slot 0x20
        self.flow.front.press_call = 2;
    }

    /// The rest of 0x983EE, in the third frame of the call: after the clear 0xAE0 the palette set 0 (0x89E6), the screen task
    /// 0x978EE in slot 0x20 (it first runs the next frame; its pass is ticked in this frame too), the layer control 0x12D6,
    /// scroll1 cleared again (0xB52), the base shadows, layer control 0x1396 and the brightness (0x892C).
    pub(super) fn flow_press_start_rest(&mut self) {
        self.flow_scene_palettes(0);
        let mut press = Scene::start(0);
        press.sleep(2);
        self.flow.front.press = press;
        self.w.layer_ctrl = 0x12D6;
        self.flow_clear_scroll1(); // 0xB52
        self.w.layer_ctrl = 0x1396;
        self.brightness_full(); // 0x892C
        // The video frame boundary falls in the middle of the brightness loop 0x8982 (an OR of 0xF000 into every word, bank by bank):
        // the words of scroll3's bank from the 154th (title) or 139th (demo) on get theirs at the start of the next frame (records
        // 460 and 461 of `flow_c-title`, 4010 and 4011 of `flow_m113-coin`; the task's first run sets the full brightness again).
        let cut = self.flow.front.press_cut;
        self.pal.bank_mut(3).iter_mut().skip(cut).for_each(|w| *w &= 0x0FFF);
    }

    /// 0x978EE: the press start screen task: the same green background as the title (lists 0x97A20
    /// and 0x97B20), the two fixed logo pieces (variants 2 and 3), then the object loop forever.
    /// The prompts are printed by the system task.
    pub fn flow_press_start_task(&mut self) {
        if !self.flow.front.press.tick() {
            return;
        }
        if self.flow.front.press.state == 0 {
            self.title_pools_reset();
            self.cam.s3.cam_x = 0;
            self.cam.s2.cam_x = 0;
            self.cam.s2.cam_y = 0;
            self.flow_scene_palettes(0);
            self.brightness_full();
            self.w.layer_ctrl = 0x12D6;
            for at in [0x90F000, 0x90F020, 0x90F400, 0x90F420] {
                self.title_block(at, 0x97A20, 0x97B20);
            }
            self.title_green();
            self.enemy_spawn_logo(2); // 0x979CA
            self.enemy_spawn_logo(3); // 0x979E2
            self.brightness_full(); // 0x979FA
            // 0x97A00: the setup takes most of a frame: the first pass of the object loop comes 2
            // frames after the first run in the recording
            self.flow.front.press.wait(2, 1);
            return;
        }
        self.screen_shake(); // 0x222D6
        self.pool_b_update(); // 0x23604
        self.enemies_update(); // 0x23420
        self.build_sprite_list(); // 0x13FA6
        self.flow.front.press.wait(1, 1);
    }

    // ------------------------------------------------------------------ the task slots

    /// Slot 0x10 (boot task, attract task) and slot 0x20 (blink task or press start task).
    pub fn flow_attract_task(&mut self) {
        if self.flow.front.press_call == 0 {
            self.flow_attract_clear_step(); // (during the press start call the system task runs the clear)
        }
        self.flow_boot_task();
        self.flow_blink_task();
        self.flow_press_start_task();
    }

    /// The attract task starts again at 0x97C20 (after a game over): sleep 2, then the pass.
    pub fn flow_attract_start(&mut self) {
        self.flow.in_game = 0;
        self.flow.front.stub = super::attract::Stub::None;
        self.flow.front.after_ranking_pass = false;
        self.flow.front.task = Scene::start(st::ATTRACT);
        self.flow.front.task.sleep(2);
    }
}
