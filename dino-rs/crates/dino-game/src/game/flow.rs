//! Game flow: the top-level tasks of the original (system task, attract / game task, join and
//! continue task, stage intro and stage clear tasks), and the stage routines the camera calls.
//! Owner: flow / HUD port. Spec: `re/specs/flow/SPEC.md`.
//!
//! The original runs these as cooperative tasks, once per frame in slot order:
//! slot 0 system task 0x9D048 (coins, credits, start), slot 0x10 attract 0x97C20 or (in game)
//! the join / continue task 0x6C4A, slot 0x20 the game task 0xA870 (mode byte G.$4D0), slot 0x30
//! the stage intro 0xB9BE or stage clear 0xB97E task, slot 0x40 the fade task (`palette.rs`).
//! `flow_frame` runs them in that order; each task keeps its own "sleep" counter.
//!
//! Sub-modules: `system` (the system task: coins, credits, SERVICE, the credit text, the start of a game), `front` (the
//! state of the front end tasks), `select` (the character select and the join task before the level), `join` (join / continue task and the all-players-out continue
//! mode), and the services of the front end scenes: `scene` (the task runner with sleep counters), `text` (string printers
//! and the typewriter), `picture` (page loaders, block printers, scene palettes) and `config` (the settings file
//! `dino.cfg`). `HANDOFF.md` in `re/specs/flow/` shows how a scene script uses them.

#[path = "flow/attract.rs"]
pub mod attract;
#[path = "flow/boot.rs"]
pub mod boot;
#[path = "flow/cards.rs"]
pub mod cards;
#[path = "flow/config.rs"]
pub mod config;
#[path = "flow/demo.rs"]
pub mod demo;
#[path = "flow/front.rs"]
pub mod front;
#[path = "flow/join.rs"]
pub mod join;
pub mod name_entry;
pub mod cont;
#[cfg(test)]
#[path = "flow/mame_check.rs"]
mod mame_check;
#[path = "flow/movie.rs"]
pub mod movie;
#[path = "flow/picture.rs"]
pub mod picture;
#[path = "flow/profiles.rs"]
pub mod profiles;
#[path = "flow/ranking.rs"]
pub mod ranking;
#[path = "flow/scene.rs"]
pub mod scene;
#[path = "flow/select.rs"]
pub mod select;
#[path = "flow/slides.rs"]
pub mod slides;
#[path = "flow/system.rs"]
pub mod system;
#[path = "flow/tally.rs"]
pub mod tally;
#[path = "flow/text.rs"]
pub mod text;

pub use config::{Config, Settings};

use super::Game;
use super::Handle;

/// The region word G.$32 (0 Japan, 1 USA, 2 ETC). The boot task writes 2 at 0xA736 whatever the
/// ROM set is, so the default is 2 (the World set is the only target).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region(pub u16);

impl Default for Region {
    fn default() -> Self {
        Region(2)
    }
}

/// Where the stage intro task goes on after a sleep inside one of its routines (`jsr 0xA76` in the middle of a state).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IntroAfter {
    /// The loop: the state routine of the next pass.
    #[default]
    Loop,
    /// 0xC526: the code after its first sleep (the card is set up).
    State0,
    /// 0xC6AC: the code after the sleep that ends the slide in (the title of the episode is typed).
    Title,
    /// 0xCA94: the code after the 60 frame sleep (0xA82) that ends the marker blink.
    Wait60,
    /// 0xCC7C: the HUD redraw 0x162AE after the sleep inside 0xCC58.
    Redraw,
}

/// Stage intro task 0xB9BE (struct G.$735E).
#[derive(Clone, Copy, Debug, Default)]
pub struct Intro {
    pub live: bool,
    /// +0 state, +1 sub-state, +2 and +3 the marker period counter and its light flag, +4 the marker time (the map card of the stages 1 to 7).
    pub state: u8,
    pub sub: u8,
    pub b2: u8,
    pub b3: u8,
    pub w4: u16,
    /// +6 columns copied (x8), +8 copy limit, +$A source and +$E target scroll1 address.
    pub w6: u16,
    pub w8: u16,
    pub src: u32,
    pub dst: u32,
    /// Frames to sleep before the next run.
    pub sleep: u16,
    /// The task has been created but has not run yet: its first run is 0xB9BE with the init 0xB9F6 (the struct above is still 0 until
    /// then) and a sleep of one frame.
    pub init: bool,
    /// Where the next run goes on (a sleep inside a state routine).
    pub after: IntroAfter,
}

#[derive(Default)]
pub struct Flow {
    /// The cooked settings G.$777A..C and the named options they come from (`config.rs`).
    pub dip: Settings,
    pub config: Config,
    /// G.$32 region word.
    pub region: Region,
    /// The typewriter objects (pool B class 0x68 in the original), see `text.rs`.
    pub typers: Vec<text::Typer>,
    /// G.$4CC in a game; G.$4CD players that pressed start (bits); G.$4E4 players that chose a
    /// character (bits).
    pub in_game: u8,
    pub started: u8,
    pub chosen: u8,
    /// G.$4CF 2-player layout (from DIP A bit 4 at game start).
    pub b4cf: u8,
    /// A game was started by the system task and the game task 0xA870 is not created yet (the
    /// fade out and the clear take 18 frames): `flow_frame` does not run the game task.
    pub game_pending: bool,
    /// G.$4D0 game mode, G.$4D1 sub-mode, G.$4D2 sub-sub word.
    pub mode: u8,
    pub sub: u8,
    pub sub2: u16,
    /// G.$4D6 game task period in frames (1 normal, 4 = slow motion after a boss), and the
    /// frames left before the game task runs again.
    pub speed: u8,
    pub wait: u8,
    /// G.$4D7 no continue (DIP, or the last stage was cleared), G.$4DB, G.$507 ending flags.
    pub b4d7: u8,
    pub b4db: u8,
    pub b507: u8,
    /// G.$5E4 name entry running (1 while a player enters, 2 at the game over wait).
    pub b5e4: u8,
    /// The name entry blocks and shared blink state (`name_entry.rs`).
    pub name: name_entry::NameEntry,
    /// G.$4DA game frame counter (prompt blink timing).
    pub frame: u8,
    /// G.$4E2 stage clear sequence running, G.$4E3 player script reached its mark (set by the
    /// player scripts 0x1DA10; see `flow_stage_start`).
    pub b4e2: u8,
    pub b4e3: u8,
    /// G.$4E6 extra lives at game start (for the rank code), G.$4E5 extra-life table selector.
    pub b4e6: u8,
    pub b4e5: u8,
    /// G.$508 stage clear timer.
    pub timer508: u16,
    /// G.$510 game mode saved while the continue screen (mode 0x10) runs.
    pub saved_mode: [u8; 4],
    pub intro: Intro,
    /// The stage clear task 0xB97E (`flow/tally.rs`).
    pub tally: tally::Tally,
    /// The story panel of mode 8 (0xD898, `flow/cards.rs`).
    pub story: cards::Story,
    pub front: front::Front,
    /// The opening movie's own state (`movie.rs`).
    pub movie: movie::Movie,
    pub join: [join::Join; 3],
    /// G.$5CC: continue screen state and the separate scene tilemap banks.
    pub cont: cont::Continue,
    /// The join task 0x6C4A (slot 0x10, `select.rs`): created by mode 0.
    pub join_task: scene::Scene,
    /// The mode routine of the game task sleeps inside itself (`jsr 0xA76` and the other sleeps): its next run goes on in the routine
    /// and does not pass the top of the loop 0xA914 (no counting of G.$0000 and G.$4DA).
    pub resume: bool,
    /// Video frames in which no task runs (the interrupt code does): a game frame that takes the CPU for more than one frame. The first
    /// level frame of an episode (the area setup, the map and the script start) takes two (`select.rs`).
    pub lag: u8,
    /// The phase of 0xAC4E that the character select is in after its fade out (`select.rs`, `flow_stage_start_staged`); 0 = not in it.
    pub select_exit: u8,
    /// The globals of the scripted control (`script.rs`): G.$4D8, G.$4DF, G.$4EC.., G.$4FD, G.$503, G.$514, G.$516.
    pub script: super::player::script::ScriptGlobals,
}

impl Flow {
    /// G.$4CF.
    pub fn two_player_mode(&self) -> bool {
        self.b4cf != 0
    }
    pub fn b4cf(&self) -> u8 {
        self.b4cf
    }
    /// G.$4E5 as the word offset into the extra-life table 0x6BFE (0x6B88).
    pub fn extend_sel(&self) -> u8 {
        (self.dip_b_bits(0x60) >> 4) & 0xE
    }
    fn dip_b_bits(&self, m: u8) -> u8 {
        self.dip.dip_b & m
    }
}

impl Game {
    // ---------------------------------------------------------------- entry points

    /// Takes the settings file (`config.rs`): the named options and the cooked bytes G.$777A..C
    /// that the rest of the game reads (the game start 0xA804, the rank code, the score code and
    /// the sound filter). Call it before `flow_boot` or `start_stage`; without it the factory
    /// values are in place.
    pub fn flow_apply_config(&mut self, config: Config) {
        self.flow.dip = config.cook();
        self.flow.config = config;
    }

    /// Power on: the reset state, then the boot task 0xA69A (`boot.rs`): the two boot text screens
    /// and the attract task with the title. Use this to run the whole game; `start_stage` starts
    /// in a stage.
    pub fn flow_boot(&mut self) {
        self.flow_boot_start();
    }

    /// Direct start in a stage (testing; `main` passes --stage and --chars): the game start
    /// 0xA870 / mode 0 with the given players and characters, without the select screen, then
    /// the stage start 0xAC4E.
    pub fn start_stage(&mut self, stage: u8, chars: &[u8]) {
        self.start_stage_at(stage, 0, 0, chars);
    }

    /// Review aid (`--episode`, `--area`, `--camx`, M3-05): `start_stage` in camera area `area`, with the camera at X `camx`
    /// (0 = the area's start). The players move with the camera, and the spawn records the camera has passed are skipped
    /// (like the probe jump, see `re/harness/probe/probe.lua`). Not exact: no camera events ran to reach X.
    pub fn start_stage_at(&mut self, stage: u8, area: u8, camx: u16, chars: &[u8]) {
        self.flow_sys_init(); // Direct starts bypass boot's 0x9CFC6 credit/task setup.
        self.hud_hiscore_init();
        let mut bits = 0u8;
        for i in 0..chars.len().min(3) {
            bits |= 1 << i;
        }
        self.flow.started = bits;
        self.flow_game_task_start();
        for (i, &c) in chars.iter().take(3).enumerate() {
            self.player_join(i, c);
            self.players[i].b.live = 1;
        }
        self.flow.chosen = bits;
        self.flow_mode_init();
        self.w.stage = stage;
        self.cam.s2.area = area;
        self.flow_stage_start();
        // Direct starts bypass the select task's 0x71DA transition to playing.
        for j in self.flow.join.iter_mut().take(chars.len().min(3)) {
            j.state = 6;
        }
        if camx != 0 {
            self.flow_jump_camera(camx);
        }
        // `area` above 0 only sets the area byte before the start: the camera init (0x20A9C) resets it, so `main.rs`
        // reaches later areas in play with the game's own area change.
    }

    /// Moves the camera to X (review aid, see `start_stage_at`): the present players move by the same distance, and the spawn
    /// lists go on after the records the camera has already passed (`spawn.rs` trigger test: X unsigned, `bhi`/`bls`).
    pub fn flow_jump_camera(&mut self, x: u16) {
        let d = x.wrapping_sub(self.cam.s2.cam_x) as i16;
        self.cam.s2.cam_x = x;
        for p in self.players.iter_mut().filter(|p| p.b.live != 0) {
            let nx = p.b.xi().wrapping_add(d);
            p.b.set_xi(nx);
        }
        let area = self.cam.s2.area;
        let assets = self.assets;
        let sc = &assets.enemy.stages[self.w.stage as usize].areas[area as usize & 3];
        let passed = |t: u16| t <= x;
        self.spawn.init = 2; // the list driver keeps its position (it resets only when init is 0)
        self.spawn.area = area;
        while sc.list1.get(self.spawn.l1).is_some_and(|r| passed(r.trig)) {
            self.spawn.l1 += 1;
        }
        while sc.list2.get(self.spawn.l2).is_some_and(|r| passed(r.trig)) {
            self.spawn.l2 += 1;
        }
    }

    /// One frame of all flow tasks, in task slot order.
    pub fn flow_frame(&mut self) {
        self.flow_coin_isr(); // the interrupt: 0x9D4BE reads the coin inputs before the tasks
        if self.flow.lag > 0 {
            // a task keeps the CPU through this video frame: no task runs (the next vertical blank only runs the interrupts)
            self.flow.lag -= 1;
            self.flow.front.prev_input = self.w.input;
            self.flow.front.prev_system = self.w.system;
            return;
        }
        self.flow_system_task(); // slot 0
        if self.flow.in_game != 0 {
            self.flow_join_task(); // slot 0x10
            self.select_join_task(); // slot 0x10: the states before the level (modes 0 and 4)
            self.flow_game_task(); // slot 0x20
        } else {
            self.flow_attract_task(); // slots 0x10 / 0x20
        }
        self.flow_intro_task(); // slot 0x30
        self.flow_tally_task(); // slot 0x30
        // The vertical blank of the next frame copies G.$16..18 to G.$1B..1D and G.$1A to G.$1F
        // (0x738) before it reads the new inputs, so during a task the "previous" bytes are the
        // inputs of the frame before.
        self.flow.front.prev_input = self.w.input;
        self.flow.front.prev_system = self.w.system;
        self.flow.front.ticks = self.flow.front.ticks.wrapping_add(1);
    }

    // ---------------------------------------------------------------- game task 0xA870

    /// 0xA870 entry: game start. Players that pressed start get +$00 = 1 (their init is the
    /// player port's `player_join`, called by the character select).
    pub fn flow_game_task_start(&mut self) {
        // 0xA804: DIP decode
        self.flow.b4cf = (self.flow.dip.dip_a & 0x10 != 0) as u8;
        if self.flow.dip.dip_c & 0x40 == 0 {
            self.flow.b4d7 = 1;
        }
        self.flow.in_game = 1;
        self.spawn.g.b4d5 = 1;
        self.flow.speed = 1;
        self.flow.wait = 0;
        // 0x6B88 extra-life selector
        self.flow.b4e5 = self.flow.extend_sel();
        for i in 0..3 {
            self.flow_player_init(i);
        }
        self.flow.chosen = self.flow.started;
        for i in 0..3 {
            if self.flow.started & (1 << i) != 0 {
                self.players[i].b.live = 1;
            }
        }
        self.flow.mode = 0;
        self.flow.sub = 0;
        self.flow.sub2 = 0;
    }

    /// The flow-owned part of the player init 0x4D46 / 0x4D5E / 0x4D76 (0x4D8E clears the
    /// object, HP 100, rank 0x33) and 0x4DD6 (extra lives from the settings).
    pub fn flow_player_init(&mut self, i: usize) {
        self.hud.stats[i] = super::hud::Stats::default();
        self.players[i].b10a = 0x33;
        let b = &mut self.players[i].b;
        b.hp = 100;
        b.hp_prev = 100;
        b.hp_bar = 100;
        b.who = i as u8;
        self.flow_lives_init(i);
    }

    /// 0x4DD6: extra lives = settings C bits 0-1; also G.$4E6.
    pub fn flow_lives_init(&mut self, i: usize) {
        let n = self.flow.dip.dip_c & 3;
        self.players[i].lives = n;
        self.flow.b4e6 = n;
    }

    /// The game task loop body (0xA914): sleep (G.$4D6 & 15) frames, count, run the mode.
    fn flow_game_task(&mut self) {
        if self.flow.in_game == 0 || self.flow.game_pending {
            return;
        }
        if self.flow.wait > 1 {
            self.flow.wait -= 1;
            return;
        }
        self.flow.wait = self.flow.speed & 0xF;
        // 0xA914..: the frame counters G.$0000 and G.$4DA are incremented before the mode runs,
        // so everything that reads G.$0000 in a game frame (parity tests, spark jitter, ...) sees
        // the new value. A mode routine that slept inside itself goes on without passing here.
        if !std::mem::take(&mut self.flow.resume) {
            self.w.frame = self.w.frame.wrapping_add(1);
            self.flow.frame = self.flow.frame.wrapping_add(1);
        }
        match self.flow.mode {
            0x00 => self.flow_mode_start(),
            0x04 => self.flow_select_mode(),
            0x08 => self.flow_story_mode(), // 0xD898
            0x0C => {
                self.game_frame();
                self.flow_ingame_switch();
            }
            0x10 => self.flow_continue_mode(),
            0x14 | 0x18 => self.flow_game_over(),
            _ => self.flow_game_over(), // 0x1C ending 0xA69B8: not ported
        }
    }

    /// Mode 0 as the game task runs it after a start from the attract loop (0xA978): the routine
    /// needs 4 frames of CPU time. The recording shows the rank start (0x17F8A) and the palettes of the
    /// stage at full brightness (0x8A44, 0x892C) after the first 2, the rest of `flow_mode_init`
    /// (HUD, mode 4) 2 frames later.
    fn flow_mode_start(&mut self) {
        if self.flow.sub2 != 0 {
            self.flow_stage_setup(); // 0xA9BE (loads the palettes again), sets mode 4
            // the recording shows the first part of the select (layer control 0x06D6, the palette set 1) in the same frame as the mode
            self.flow_select_mode();
            return;
        }
        self.flow_join_start(); // task 0x6C4A in slot 0x10
        self.flow.join_task = scene::Scene::start(0);
        self.flow.join_task.sleep(2); // the recording: its first pass is two frames after the first pass of mode 0 (mode 0 keeps the CPU)
        self.vram.clear2();
        self.vram.clear3();
        self.sprites.clear();
        self.rank.dip = self.flow.dip.dip_b;
        self.rank_game_start(); // 0x17F8A
        self.rank.started = true;
        self.load_all_palettes(); // 0x8A44
        self.brightness_full(); // 0x892C
        self.flow.sub2 = 1;
        self.flow.wait = 2;
    }

    /// Mode 0 (0xA978): game start setup, then the character select (in one go: the direct start
    /// `start_stage` uses it).
    fn flow_mode_init(&mut self) {
        self.flow_join_start(); // task 0x6C4A in slot 0x10
        self.flow.join_task = scene::Scene::start(0);
        self.flow.join_task.sleep(2); // the recording: its first pass is two frames after the first pass of mode 0 (mode 0 keeps the CPU)
        self.vram.clear2();
        self.vram.clear3();
        self.sprites.clear();
        self.rank.dip = self.flow.dip.dip_b;
        self.rank_game_start(); // 0x17F8A
        self.rank.started = true;
        self.flow_stage_setup(); // 0xA9BE, sets mode 4
    }

    /// 0xA9BE: setup shared by game start and every stage start.
    fn flow_stage_setup(&mut self) {
        self.flow_stage_setup_palettes();
        self.flow_stage_setup_rest();
    }

    /// The first part of 0xA9BE: the palettes of the stage and the full brightness.
    fn flow_stage_setup_palettes(&mut self) {
        self.load_all_palettes(); // 0x8A44
        self.brightness_full(); // 0x892C
    }

    /// The rest of 0xA9BE: the scroll controllers, the select struct, the rank, the layer control, the HUD, the area start and the mode
    /// byte (4, sub 0).
    fn flow_stage_setup_rest(&mut self) {
        self.flow_stage_setup_rest_a();
        self.flow_area_reset_b();
    }

    /// The part of 0xA9BE up to the pool A init of the area start (0x4C48): the part of a stage start that comes in the frame before the last one
    /// (`flow_stage_start_staged`, phase 3).
    fn flow_stage_setup_rest_a(&mut self) {
        self.flow_scroll_reset(); // 0x20402
        self.flow.front.select.clear(); // 0xAD36
        self.rank_stage_start(); // 0x1800E
        self.w.layer_ctrl = 0x12D6;
        self.w.stage_clear = 0;
        self.hud_init(); // 0x16894
        self.hud_rings_init(); // 0x4CEE
        self.flow.mode = 4;
        self.flow.sub = 0;
        self.flow.sub2 = 0;
        // 0x4A00 list, 0x4C1A pool E init
        self.items = super::Pool::new(super::POOL_E);
        self.flow_area_reset_a(); // 0xAA0E
    }

    /// 0x20402: reset the scroll2 / scroll3 controllers and clear both layers.
    fn flow_scroll_reset(&mut self) {
        self.flow_scroll_state_reset();
        self.vram.clear2();
        self.vram.clear3();
    }

    /// The first half of 0x20402: the controller fields (the frame of the recordings that start it shows them zero before the rest of 0xA9BE).
    fn flow_scroll_state_reset(&mut self) {
        let s = &mut self.cam.s2;
        s.state = 0;
        s.mode = 0;
        s.sub = 0;
        s.cam_x = 0;
        s.cam_x_frac = 0;
        s.cam_y = 0;
        s.cam_y_frac = 0;
        s.x5a = 0;
        s.y5c = 0;
        let t = &mut self.cam.s3;
        t.state = 0;
        t.b5 = 0;
        t.cam_x = 0;
        t.cam_y = 0;
        t.x5a = 0;
        t.y5c = 0;
    }

    /// 0xAE0: all cameras to 0, clear the sprite list and the three layers.
    pub fn flow_clear_screen(&mut self) {
        self.cam.cam1_x = 0;
        self.cam.cam1_y = 0;
        self.cam.s2.cam_x = 0;
        self.cam.s2.cam_y = 0;
        self.cam.s3.cam_x = 0;
        self.cam.s3.cam_y = 0;
        // 0xB10: the two sprite buffers. (Not `SpriteList::clear`: that is the start of the sprite
        // list builder 0x13FA6 and steps the water splash timer, which 0xAE0 does not.)
        self.sprites.wipe();
        self.flow_clear_scroll1();
        self.vram.clear2();
        self.vram.clear3();
    }

    /// 0xB52: scroll1 filled with spaces (code 0x20, attribute 6).
    pub fn flow_clear_scroll1(&mut self) {
        for c in self.vram.scroll1.iter_mut() {
            c.code = 0x20;
            c.attr = 6;
        }
    }

    /// 0xAA0E..0xAAF4: area start (also the camera hook 0xACD0).
    fn flow_area_reset(&mut self) {
        self.flow_area_reset_a();
        self.flow_area_reset_b();
    }

    /// 0xAA0E up to the pool A init (0x4C48): the heroes' start, the start script, the lists.
    fn flow_area_reset_a(&mut self) {
        for i in 0..3 {
            self.flow_player_area_start(i); // 0x4E0A
        }
        self.script_area_start(); // 0x1DA10: the start script of the area, the chosen heroes under script
        // 0x49A8.. per-frame lists: player port / main.
        self.sprites.splash_timer = 1; // 0x49B4: the water splash timer (the phase -$7124 is not reset)
        self.terrain_obstacles_reset(); // 0x4A76
        self.enemies = super::Pool::new(super::POOL_A); // 0x4C48
    }

    /// 0xAA0E from the pool B init (0x4C76) on. In a stage start the 68000 is busy with it in the next video frame (`flow_stage_start_staged`).
    fn flow_area_reset_b(&mut self) {
        self.effects = super::Pool::new(super::POOL_C); // 0x4CA4
        self.draw_special_init(); // 0x4CCC: the free stack only, the slots keep their bytes
        self.spawn_area_reset_objects(); // pools A, B; pool E loop; G.$4F1.. cleared
        self.rank_area_start(); // 0x18048
        self.hook_aae2(); // 0x1031A, 0x12BE4, 0xFBE8
    }

    /// 0x4E0A: a present player's start position for this stage and area (table 0x4E56),
    /// state 0.
    pub fn flow_player_area_start(&mut self, i: usize) {
        if self.players[i].b.live == 0 {
            return;
        }
        let t = &self.assets.text;
        let st = self.w.stage as u32;
        let tbl = 0x4E56 + t.u16(0x4E56, 2 * st) as u32;
        let off = self.cam.s2.area as u32 * 18 + i as u32 * 6;
        let (x, y, z) = (t.i16(tbl, off), t.i16(tbl, off + 2), t.i16(tbl, off + 4));
        let b = &mut self.players[i].b;
        b.set_state(0);
        b.set_xi(x);
        b.set_yi(y);
        b.set_zi(z);
    }

    /// 0x655E: G.$4EF = (players present and not in state 0) - 1, at least 0.
    pub fn flow_count_players(&mut self) {
        let n = self.players.iter().filter(|p| p.b.live != 0 && p.b.mode != 0).count() as u8;
        self.w.players_minus_one = n.saturating_sub(1);
    }

    /// 0xAC4E: stage start (after the select screen and after each stage clear).
    pub fn flow_stage_start(&mut self) {
        self.flow_clear_screen(); // 0xAE0
        self.cam.cam1_y = 0x100;
        self.flow_intro_start(); // task 0xB9BE in slot 0x30
        self.flow_stage_setup(); // 0xA9BE
        self.flow_stage_start_tail();
    }

    /// 0xAC4E as it runs after the character select (`select.rs`), in the frames of the recording (`flow_sel-1p`, the fade out is over in
    /// 1487): phase 0 (1488) the clear 0xAE0 starts (the sprites and scroll1 now, scroll2 and scroll3 in the next two frames, `flow_attract_clear`),
    /// phase 2 (1490) the camera, the intro task and the palettes of the stage with full brightness, phase 3 (1491) the rest of 0xA9BE
    /// (the mode byte is 4 again), phase 4 (1492) the mode 0xC, the sound stop and the music.
    pub(super) fn flow_stage_start_staged(&mut self, phase: u8) {
        match phase {
            0 => self.flow_attract_clear(),
            2 => {
                self.flow_stage_start_staged(20);
                self.flow_stage_start_staged(21);
            }
            20 => {
                self.cam.cam1_y = 0x100;
                self.flow_intro_start();
                self.flow.intro.sleep = 3; // the task first runs in the frame that finishes the stage start (phase 4), after the game task
            }
            // the palettes (0x8A44, 0x892C) and 0x20402 starts (the controllers are zero in the record of this frame; after the story panel they held the
            // level's; after the panel of Episode 5 the video frame boundary comes before them, `story_exit_ep5`)
            21 => {
                self.flow_stage_setup_palettes();
                self.flow_scroll_state_reset();
            }
            3 => {
                // The video frame boundary falls inside 0xAA0E, between the pool A init (0x4C48) and the pool B init (0x4C76, 0xAA80): in
                // this frame the objects of the screen before (the props of the select screen, the pools C and D, the stage time, the floor)
                // are still there (the recordings: they are gone one frame later).
                self.flow_stage_setup_rest_a();
            }
            4 => {
                self.flow_area_reset_b(); // 0x4C76 on
                self.flow_stage_start_tail();
            }
            _ => {}
        }
    }

    /// 0xAC6E to the end of 0xAC4E: the game mode 0xC, the players of the level, the sound stop and the music of the stage.
    fn flow_stage_start_tail(&mut self) {
        self.flow.mode = 0xC;
        self.flow_count_players();
        for (i, p) in self.players.iter_mut().enumerate() {
            self.spawn.g.kills[i] = 0; // clr.b $10B(a6)
            p.b.hp = 100; // 0x4DC2
            p.b.hp_prev = 100;
            p.b.hp_bar = 100;
        }
        self.rank.b7b4 = 0;
        self.sound(0xFF00);
        if self.w.stage == 0 {
            self.sound(1);
        } else {
            self.flow_area_music();
        }
    }

    /// 0xA55A: music of the stage and area (table 0xA582; the G.$7B4 == 2 variant 0xA5C8).
    fn flow_area_music(&mut self) {
        let t = &self.assets.text;
        let id = if self.rank.b7b4 == 2 {
            let mut k = self.w.stage as u32;
            if k == 7 && self.hud.boss != 0 {
                // stage 7: a boss with +$7D == 0xFE selects entry 8 (not tracked here) [U]
                k = 7;
            }
            t.u8(0xA5C8, k)
        } else {
            t.u8(0xA582, self.w.stage as u32 * 4 + self.cam.s2.area as u32)
        };
        self.sound(id as u16);
    }

    /// 0xAB2C: after the game frame, the stage clear sequence on G.$4D1.
    fn flow_ingame_switch(&mut self) {
        match self.flow.sub {
            2 => {
                // 0xAB44: wait until the boss is in its death state, then slow motion ends
                let boss = self.spawn.g.killer;
                if super::slot_of(boss).is_some() && self.body(boss).mode >= 4 {
                    self.flow.speed = 1;
                    self.flow.timer508 = 0x708;
                    self.flow.sub = 4;
                }
            }
            4 => {
                // 0xAB66: walk-off (or 1800 frames), then the clear task
                self.flow.timer508 = self.flow.timer508.wrapping_sub(1);
                if (self.flow.timer508 as i16) < 0 || self.flow.b4e3 != 0 {
                    self.flow_tally_start(); // task 0xB97E in slot 0x30
                    self.flow.b4e2 = 1;
                    self.flow.timer508 = if self.w.stage == 7 { 0xE10 } else { 0x708 };
                    self.flow.sub = 6;
                }
            }
            6 => {
                // 0xABA8: when the clear task is done, the next stage (or the ending)
                self.flow.timer508 = self.flow.timer508.wrapping_sub(1);
                if (self.flow.timer508 as i16) < 0 || self.flow.b4e2 == 0 {
                    if self.w.stage == 7 {
                        self.flow.mode = 0x1C;
                        self.cam.s2.area = 0;
                        self.flow.sub = 0;
                        self.flow.sub2 = 0;
                        self.flow.b4d7 = 1;
                        self.flow.b4db = 1;
                        self.flow.b507 = 1;
                    } else {
                        self.w.stage = (self.w.stage + 1) & 7;
                        self.cam.s2.area = 0;
                        self.flow.mode = 8;
                        self.flow.sub = 0;
                        self.flow.sub2 = 0;
                    }
                }
            }
            _ => {}
        }
    }

    /// 0xACEE: a stage boss was defeated (called by the boss code at HP 0).
    pub fn flow_boss_defeated(&mut self, boss: Handle) {
        self.w.stage_clear = 1;
        self.flow.speed = 4;
        self.spawn.g.b4d5 = 1;
        self.spawn.g.killer = boss;
        self.spawn.g.kill_all = 1;
        self.flow.sub = 2;
        self.rank_speed_add(1);
        self.hud_clear_idle_blocks(); // 0x165FC
        if self.w.stage == 7 {
            self.flow.b4d7 = 1;
            self.flow.b4db = 1;
        }
    }

    /// Modes 0x14 / 0x18 (0xAC06 / 0xAC26): game over, back to the attract mode.
    fn flow_game_over(&mut self) {
        if self.flow.cont.exit_wait > 1 {
            self.flow.cont.exit_wait -= 1;
            if self.flow.mode == 0x18 && self.flow.cont.exit_wait == 2 {
                self.sound(0xFF00); // 0xAC2C (a5d2), after the clear
            }
            return;
        }
        if self.flow.mode == 0x14 {
            self.flow.intro.live = false;
            self.flow.tally.live = false;
            self.flow.join_task.kill(); // 0xAC06: slot 0x10.
            self.flow.mode = 0x18;
            self.flow.front.game_over_flag = 1;
            self.flow_clear_screen();
            self.flow.cont.exit_wait = 4; // clear 0xAE0 crosses two vblanks.
            return;
        }
        self.flow.cont.exit_wait = 0;
        self.flow.front.game_over_flag = 1;
        if self.flow.mode != 0x18 {
            self.sound(0xFF00);
        }
        self.flow.in_game = 0;
        self.flow.game_pending = false;
        self.flow.front.sys.press_up = 0;
        self.flow_attract_start();
        if self.flow.mode == 0x18 {
            // The new task slept two frames while the game task was clearing.
            self.flow.front.task.sleep(1);
            self.flow_attract_task(); // 0x97C26, the first awake pass.
        }
    }

    // ---------------------------------------------------------------- stage intro 0xB9BE

    /// 0xAC5E: the task 0xB9BE is created (slot 0x30). Its first run is not in the frame of the creation: the original's frame is taken by
    /// the stage start (0xAC4E keeps the CPU through the palettes, the rest of 0xA9BE and the area start), so the first run (the struct
    /// is still 0 until it) comes in the frame that finishes the stage start and runs the first level frame, after the game task.
    /// After the character select the first run is 2 frames after the creation (`flow_stage_start_staged`: the recording of the Episode 1 start).
    fn flow_intro_start(&mut self) {
        // the struct G.$735E is not cleared by the creation: it keeps the bytes of the last task (the clear tally's, or the intro task's of the level before)
        self.flow.intro = Intro { live: true, sleep: 1, init: true, after: IntroAfter::Loop, ..self.flow.intro };
    }

    /// 0xB9BE, the first run: `clr.l 0(a6)` (state, sub, +2, +3), then for the stage 0 the entry 0xB9F6 (state 4 sub 2: no map card, the walk in
    /// of the heroes comes first; `jsr 0xA76`), for the other stages the state 0 (0xC526: the map card, `jsr 0xA76` first).
    fn flow_intro_init(&mut self) {
        // the struct G.$735E is the intro task's again (the tally image is not shown any more), with the bytes the tally left in it
        if self.flow.tally.owner {
            let r = &self.flow.tally.raw;
            self.flow.intro =
                Intro { state: r.b(0), sub: r.b(1), b2: r.b(2), b3: r.b(3), w4: r.w(4), w6: r.w(6), w8: r.w(8), src: r.l(0xA), dst: r.l(0xE), ..self.flow.intro };
        }
        self.flow.tally.owner = false;
        let it = &mut self.flow.intro;
        it.state = 0;
        it.sub = 0;
        it.b2 = 0;
        it.b3 = 0;
        it.init = false;
        if self.w.stage == 0 {
            it.state = 4;
            it.sub = 2;
            it.w6 = 0;
            it.w8 = 0xC;
            it.src = 0x90A000;
            it.dst = 0x909800;
        } else {
            it.after = IntroAfter::State0;
        }
    }

    /// 0xB9BE task body (one pass a frame: the state routine, then `jsr 0xA76`): the map card slides in (state 2 sub 0), the title is typed and the
    /// marker blinks (sub 2), the card slides out (state 4 sub 0), the walk in of the heroes (sub 2) and the HUD slides in from the right
    /// (sub 4); state 6 ends. The stage 0 starts at state 4 sub 2.
    fn flow_intro_task(&mut self) {
        if !self.flow.intro.live {
            return;
        }
        if self.flow.intro.sleep > 1 {
            self.flow.intro.sleep -= 1;
            return;
        }
        self.flow.intro.sleep = 1;
        if self.flow.intro.init {
            self.flow_intro_init();
            return;
        }
        match std::mem::take(&mut self.flow.intro.after) {
            IntroAfter::Loop => {}
            IntroAfter::State0 => return self.intro_state0(),
            IntroAfter::Title => return self.intro_title(),
            IntroAfter::Wait60 => return self.intro_after_wait(),
            IntroAfter::Redraw => {
                // 0xCC7C: the code after the sleep inside 0xCC58, then the loop's sleep
                self.hud_redraw(); // 0x162AE
                return;
            }
        }
        let it = self.flow.intro;
        match (it.state, it.sub) {
            (2, 0) => self.intro_slide_in(),
            (2, 2) => self.intro_blink(),
            (4, 0) => self.intro_slide_out(),
            (4, 2) => {
                // 0xCC58
                self.cam.cam1_x = 0;
                self.cam.cam1_y = 0;
                // G.$4E3 is set by the start script of the heroes (0x1DA10, `script.rs`).
                if self.flow.b4e3 != 0 {
                    self.flow.intro.sub = 4;
                    self.cam.cam1_x = it.w6;
                    self.cam.cam1_y = 0x100;
                    self.flow.intro.after = IntroAfter::Redraw; // jsr 0xA76 (sleep 1) inside the step; the HUD is drawn after it
                }
            }
            (4, 4) => {
                // 0xCC84: copy whole columns (32 cells) from the lower half to the upper half
                let mut it = self.flow.intro;
                while (it.w8 as i16) >= (it.w6 as i16) {
                    for k in 0..32u32 {
                        let c = self.vram.scroll1[Self::flow_cell(it.src + 4 * k)];
                        self.vram.scroll1[Self::flow_cell(it.dst + 4 * k)] = c;
                    }
                    it.w6 = it.w6.wrapping_add(8);
                    it.src += 0x80;
                    it.dst = ((it.dst + 0x80) & 0x909FFF) | 0x908000;
                }
                it.w8 = it.w8.wrapping_add(0xC);
                self.cam.cam1_x = self.cam.cam1_x.wrapping_add(0xC);
                let limit = if self.w.stage != 0 { 0x3F0 } else { 0x180 };
                self.flow.intro = it;
                if limit < self.cam.cam1_x {
                    self.cam.cam1_x = 0;
                    self.cam.cam1_y = 0;
                    self.flow.intro.state = 6;
                }
                self.spawn.g.b4d5 = 0;
                self.flow_join_prompts(); // 0x7562
            }
            (6, _) => {
                // 0xCCFE: G.$4E2 = 0 (except stage 7 area 3), task ends
                if !(self.w.stage == 7 && self.cam.s2.area == 3) {
                    self.flow.b4e2 = 0;
                }
                self.flow.intro.live = false;
            }
            _ => self.flow.intro.live = false,
        }
    }

    fn flow_cell(a: u32) -> usize {
        ((a.wrapping_sub(0x908000) / 4) & 0xFFF) as usize
    }

    // ---------------------------------------------------------------- camera hooks

    /// 0xA5EA, when an area-end lock is released: sound command 0xFF07 (music fade out).
    pub fn hook_a5ea(&mut self) {
        self.sound(0xFF07);
    }

    /// 0xACD0, after the fade out, before the next area is loaded: area start 0xAA0E (players
    /// placed, object pools reset, rank and time), area music 0xA55A, HUD redraw 0x162AE,
    /// player count 0x655E, rank speed +2 (0x18566).
    pub fn hook_acd0(&mut self) {
        self.flow_area_reset();
        self.flow_area_music();
        self.hook_162ae();
        self.flow_count_players();
        self.rank_speed_add(2);
    }

    /// 0x12BE4, after the next area's map is set up: selects the floor table of the stage and
    /// area (G.$6AC / $6B0, read by the floor collision 0x12C90 / 0x133E0) and, in the vertical
    /// scene, draws two blocks at 0x91C000 / 0x91C400 (`terrain.rs`).
    pub fn hook_12be4(&mut self) {
        self.terrain_area_start();
    }

    /// 0x162AE, before the fade in of the next area: redraw the HUD of the present players.
    pub fn hook_162ae(&mut self) {
        self.hud_redraw();
    }

    /// 0xAAE2 (tail of the area start): stage time 0x1031A, floor table 0x12BE4, spawn lists
    /// 0xFBE8.
    pub fn hook_aae2(&mut self) {
        self.hook_1031a();
        self.hook_12be4();
        self.spawn_reset();
    }

    /// 0x1031A: stage time for the area (enemy port's `time_area_start`).
    pub fn hook_1031a(&mut self) {
        self.rank_time_area_start();
    }

    /// 0xACE0: player count 0x655E and rank speed +2 (0x18566).
    pub fn hook_ace0(&mut self) {
        self.flow_count_players();
        self.rank_speed_add(2);
    }

    /// 0x9236C: create a pool B object of class 0x60 (+$20 = 0x180) for the sub-area.
    pub fn hook_9236c(&mut self) {
        if let Some(h) = self.spawn_alloc(1) {
            let b = self.body_mut(h);
            b.live = 1;
            b.k20 = 0x180;
        }
    }
}
