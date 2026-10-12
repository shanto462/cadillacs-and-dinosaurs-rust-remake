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
//! Sub-modules: `front` (system task, title, press start, character select) and `join`
//! (join / continue task and the all-players-out continue mode).

#[path = "flow/front.rs"]
pub mod front;
#[path = "flow/join.rs"]
pub mod join;

use super::Game;
use super::Handle;

/// Cooked settings G.$777A / $777B / $777C. The original builds them from the EEPROM settings;
/// these are the factory defaults (0xACBF4, decoded by 0xAC7BC): 1 coin 1 credit, 3-player
/// layout, difficulty 3, initial rank speed 1, extra life at 500000, 1 extra life, continue on.
#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub dip_a: u8,
    pub dip_b: u8,
    pub dip_c: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { dip_a: 0x80, dip_b: 0x4B, dip_c: 0x41 }
    }
}

/// Stage intro task 0xB9BE (struct G.$735E).
#[derive(Clone, Copy, Debug, Default)]
pub struct Intro {
    pub live: bool,
    /// +0 state, +1 sub-state.
    pub state: u8,
    pub sub: u8,
    /// +6 columns copied (x8), +8 copy limit, +$A source and +$E target scroll1 address.
    pub w6: u16,
    pub w8: u16,
    pub src: u32,
    pub dst: u32,
    /// Frames to sleep before the next run.
    pub sleep: u16,
}

/// Stage clear task 0xB97E (simplified: see `flow_clear_task`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Clear {
    pub live: bool,
    pub state: u8,
    pub timer: u16,
}

#[derive(Default)]
pub struct Flow {
    pub dip: Settings,
    /// G.$4CC in a game; G.$4CD players that pressed start (bits); G.$4E4 players that chose a
    /// character (bits).
    pub in_game: u8,
    pub started: u8,
    pub chosen: u8,
    /// G.$4CF 2-player layout (from DIP A bit 4 at game start).
    pub b4cf: u8,
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
    pub clear: Clear,
    pub front: front::Front,
    pub join: [join::Join; 3],
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

    /// Power on: what the boot task 0xA69A does before the attract loop (high score table),
    /// then the attract mode. Use this to run the whole game; `start_stage` starts in a stage.
    pub fn flow_boot(&mut self) {
        self.hud_hiscore_init();
        self.flow.front = front::Front::default();
        self.flow_attract_start();
    }

    /// Direct start in a stage (testing; `main` passes --stage and --chars): the game start
    /// 0xA870 / mode 0 with the given players and characters, without the select screen, then
    /// the stage start 0xAC4E.
    pub fn start_stage(&mut self, stage: u8, chars: &[u8]) {
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
        self.cam.s2.area = 0;
        self.flow_stage_start();
    }

    /// One frame of all flow tasks, in task slot order.
    pub fn flow_frame(&mut self) {
        self.flow_system_task(); // slot 0
        if self.flow.in_game != 0 {
            self.flow_join_task(); // slot 0x10
            self.flow_game_task(); // slot 0x20
        } else {
            self.flow_attract_task(); // slots 0x10 / 0x20
        }
        self.flow_intro_task(); // slot 0x30
        self.flow_clear_task(); // slot 0x30
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
        if self.flow.in_game == 0 {
            return;
        }
        if self.flow.wait > 1 {
            self.flow.wait -= 1;
            return;
        }
        self.flow.wait = self.flow.speed & 0xF;
        // 0xA914..: the frame counters G.$0000 and G.$4DA are incremented before the mode runs,
        // so everything that reads G.$0000 in a game frame (parity tests, spark jitter, ...) sees
        // the new value.
        self.w.frame = self.w.frame.wrapping_add(1);
        self.flow.frame = self.flow.frame.wrapping_add(1);
        match self.flow.mode {
            0x00 => self.flow_mode_init(),
            0x04 => self.flow_select_mode(),
            0x08 => self.flow_stage_start(), // 0xD898: intermission cut-scenes not ported
            0x0C => {
                self.game_frame();
                self.flow_ingame_switch();
            }
            0x10 => self.flow_continue_mode(),
            0x14 | 0x18 => self.flow_game_over(),
            _ => self.flow_game_over(), // 0x1C ending 0xA69B8: not ported
        }
    }

    /// Mode 0 (0xA978): game start setup, then the character select.
    fn flow_mode_init(&mut self) {
        self.flow_join_start(); // task 0x6C4A in slot 0x10
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
        self.load_all_palettes(); // 0x8A44
        self.brightness_full(); // 0x892C
        self.flow_scroll_reset(); // 0x20402
        self.flow.front.select = Default::default(); // 0xAD36
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
        self.flow_area_reset(); // 0xAA0E
    }

    /// 0x20402: reset the scroll2 / scroll3 controllers and clear both layers.
    fn flow_scroll_reset(&mut self) {
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
        self.vram.clear2();
        self.vram.clear3();
    }

    /// 0xAE0: all cameras to 0, clear the sprite list and the three layers.
    pub fn flow_clear_screen(&mut self) {
        self.cam.cam1_x = 0;
        self.cam.cam1_y = 0;
        self.cam.s2.cam_x = 0;
        self.cam.s2.cam_y = 0;
        self.cam.s3.cam_x = 0;
        self.cam.s3.cam_y = 0;
        self.sprites.clear();
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
        for i in 0..3 {
            self.flow_player_area_start(i); // 0x4E0A
        }
        // 0x1DA10 (player scripts for the area), 0x49A8.. per-frame lists: player port / main.
        self.terrain_obstacles_reset(); // 0x4A76
        self.effects = super::Pool::new(super::POOL_C); // 0x4CA4
        self.specials = super::Pool::new(super::POOL_D); // 0x4CCC
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
        self.flow.mode = 0xC;
        self.flow_count_players();
        for p in self.players.iter_mut() {
            // 0x4DC2; +$10B (kills) = 0 is a player field
            p.b.hp = 100;
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
                    self.flow_clear_start(); // task 0xB97E in slot 0x30
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
        self.flow.intro.live = false;
        self.flow.clear.live = false;
        self.flow.front.game_over_flag = 1; // G.-$702E
        self.flow_clear_screen();
        self.sound(0xFF00);
        self.flow.in_game = 0;
        for p in self.players.iter_mut() {
            p.b.live = 0;
        }
        self.flow_attract_start();
    }

    // ---------------------------------------------------------------- stage intro 0xB9BE

    fn flow_intro_start(&mut self) {
        // 0xB9F6 (stage 0 entry; other stages first show the map panel and the episode title,
        // states 0..2, not ported: they use the same HUD slide-in below).
        self.flow.intro = Intro {
            live: true,
            state: 4,
            sub: 2,
            w6: 0,
            w8: 0xC,
            src: 0x90A000,
            dst: 0x909800,
            sleep: 1,
        };
    }

    /// 0xB9BE task body: state 4 sub 2 waits for the players' walk-in, then the HUD slides in
    /// from the right (sub 4), state 6 ends.
    fn flow_intro_task(&mut self) {
        if !self.flow.intro.live {
            return;
        }
        if self.flow.intro.sleep > 1 {
            self.flow.intro.sleep -= 1;
            return;
        }
        self.flow.intro.sleep = 1;
        let it = self.flow.intro;
        match (it.state, it.sub) {
            (4, 2) => {
                // 0xCC58
                self.cam.cam1_x = 0;
                self.cam.cam1_y = 0;
                // TEMP: G.$4E3 is set by the players' walk-in script (0x1DA10, player port).
                // Until that is ported the HUD appears at once.
                if self.flow.b4e3 != 0 || !self.flow_player_scripts_ported() {
                    self.flow.intro.sub = 4;
                    self.cam.cam1_x = it.w6;
                    self.cam.cam1_y = 0x100;
                    self.flow.intro.sleep = 2; // jsr 0xA76 (sleep 1) inside the step
                    self.hud_redraw(); // 0x162AE
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

    /// Whether the player scripts (0x1DA10, which set G.$4E3) exist yet. Remove with the TEMP.
    fn flow_player_scripts_ported(&self) -> bool {
        false
    }

    fn flow_cell(a: u32) -> usize {
        ((a.wrapping_sub(0x908000) / 4) & 0xFFF) as usize
    }

    // ---------------------------------------------------------------- stage clear 0xB97E

    fn flow_clear_start(&mut self) {
        self.flow.clear = Clear { live: true, state: 0, timer: 0 };
    }

    /// Stage clear task 0xB97E, simplified: the tally panels (vitality, enemies, final blow,
    /// M.V.P., bonus count-up) are not ported yet; the task waits 120 frames and ends the stage
    /// (G.$4E2 = 0).
    fn flow_clear_task(&mut self) {
        if !self.flow.clear.live {
            return;
        }
        self.flow.clear.timer += 1;
        if self.flow.clear.timer >= 120 {
            self.flow.b4e2 = 0;
            self.flow.clear.live = false;
        }
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
