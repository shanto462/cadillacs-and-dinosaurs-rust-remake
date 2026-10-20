//! Continue countdown and the frozen level, 0x732E and mode 0x10 (0xE9E6).
use crate::game::enemy::Ram;
use crate::game::{Game, POOL_C, Pool};
use dino_assets::stage::Cell;

#[derive(Clone, Default)]
pub struct Continue {
    /// G.$5CC..$5E9, with the original offsets. +8 number, +9 frames,
    /// +$A return timer, +$C..$F face animation, +$11 skip lock,
    /// +$12..$13 red flash, +$16 mouth index, +$1B..$1D credit/taunt state.
    pub raw: Ram,
    /// The scene uses different CPS tilemap banks, preserving the level's maps.
    pub level2: Vec<Cell>,
    pub level3: Vec<Cell>,
    /// 0xEE82 crosses a vblank while loading the return picture.
    pub return_pending: bool,
    /// Sleeps inside 0xEE3A, 0xAC06 and 0xAC26, including video clear time.
    pub exit_wait: u16,
}

impl Game {
    /// 0x732E..0x7474: dead player's countdown and one-credit continue request.
    pub(super) fn flow_continue_player(&mut self, i: usize) {
        if self.flow.mode == 0x10 {
            if self.flow.sub == 4 {
                self.name_qualify(i); // 0x7396: the ranking qualification after the countdown
                return;
            }
        } else {
            self.flow_continue_hud_tick(i); // 0x7346
            let p = &mut self.players[i]; // 0x71FE
            p.prev = p.held;
            p.held = self.w.input[i];
            p.pressed = p.held & !p.prev;
            if self.flow.join[i].seconds <= 18 && p.pressed & 0x30 != 0 {
                if self.flow.join[i].seconds != 0 {
                    self.flow.join[i].frames = 60;
                    self.flow.join[i].seconds = self.flow.join[i].seconds.wrapping_sub(1);
                }
                self.flow_continue_draw_hud(i);
            }
            if (self.flow.join[i].seconds as i8) < 0 {
                self.flow_join_clear(i, 0x7FB6, 0x7FC2);
                self.name_qualify(i); // 0x7392 and 0x7396: qualification (name entry or 0xC)
                return;
            }
        }
        let s = self.flow.front.sys;
        let credits = if s.common {
            s.credits
        } else {
            s.slots[i].credits
        };
        if (!s.free_play && credits == 0) || self.flow.b4d7 != 0 {
            return;
        }
        let starts = self.w.system & !self.flow.front.prev_system;
        // 0x73F2: even the separate-chute attack shortcut requires some start edge.
        if starts == 0 {
            return;
        }
        let attack = self.flow.region.0 != 0
            && !s.common
            && self.w.input[i] & !self.flow.front.prev_input[i] & 0x10 != 0;
        if starts & (1 << i) == 0 && !attack {
            return;
        }
        // 0x7426: subtraction is also performed in free play, saturating at zero.
        let c = if s.common {
            &mut self.flow.front.sys.credits
        } else {
            &mut self.flow.front.sys.slots[i].credits
        };
        let next = c.wrapping_sub(1);
        *c = if (next as i8) < 0 { 0 } else { next };
        self.flow.started |= 1 << i;
        let b = &mut self.players[i].b; // 0x4DE8 preserves hero, score and RNG.
        b.set_state(0x0002_0000);
        b.hp = 100;
        b.hp_prev = 100;
        b.hp_bar = 100;
        b.facing = 1;
        self.flow_lives_init(i); // 0x4DD6
        self.flow_continue_time(); // 0x10490
        self.flow_join_scores(); // 0x7476
        self.rank_speed_add(-2); // 0x7468: MOVEQ #$FE sign extends to -2.
        // 0x70B4, without the ordinary join's object reset or RNG call.
        self.flow.join[i] = super::join::Join {
            state: 4,
            seconds: if self.flow.b4cf != 0 { 6 } else { 0x11 },
            frames: 1,
        };
        self.players[i].w80 = 0;
        self.flow_join_cursor(i, -1);
        self.flow_join_cursor(i, 1);
        self.flow_join_clear(i, 0x7FB6, 0x7FC2);
    }

    /// 0x10490: 600 wave frames and 4:00 BCD time, capped by 0x104A8.
    fn flow_continue_time(&mut self) {
        self.spawn.g.wave_timer = self.spawn.g.wave_timer.wrapping_add(600);
        self.rank_add_time(0x400);
    }

    /// 0x7E70: HUD countdown pauses during scripted control and HUD hold.
    fn flow_continue_hud_tick(&mut self, i: usize) {
        if self.w.scripted != 0 || self.spawn.g.b4d5 != 0 || self.flow.mode > 0xC {
            return;
        }
        let t = &self.assets.frontend.text;
        self.text_words(t.slice_from(0x86BC), self.flow_join_prompt_at(i));
        if self.flow.front.sys.hold != 0 {
            self.flow.join[i].seconds = 21;
            self.flow.join[i].frames = 1;
        }
        let j = &mut self.flow.join[i];
        j.frames = j.frames.wrapping_sub(1);
        if j.frames != 0 {
            return;
        }
        j.frames = 60;
        if (j.seconds as i8) < 0 {
            return;
        }
        j.seconds = j.seconds.wrapping_sub(1);
        if (j.seconds as i8) >= 0 {
            self.flow_continue_draw_hud(i);
        }
    }

    /// 0x7EE8 / 0x75AC / 0x8156: large HUD digit and beep, including redraws.
    pub(super) fn flow_continue_draw_hud(&mut self, i: usize) {
        if self.spawn.g.b4d5 != 0 {
            return;
        }
        self.snd_request(0x35, 0x10, 0);
        let t = &self.assets.frontend.text;
        let at = t.u32(
            if self.flow.b4cf == 0 { 0x7F48 } else { 0x7F54 },
            i as u32 * 4,
        );
        let code = t.u16(0x7F5C, 2 * self.flow.join[i].seconds as u32);
        for (off, add) in [(0, 0), (4, 0x10), (0x80, 1), (0x84, 0x11)] {
            self.hud_put(at + off, code.wrapping_add(add), 0x182);
        }
    }

    /// 0xE9E6: mode 0x10 dispatch. Sub 4 is the game-over scene (0xEBA6..0xEE6C).
    pub fn flow_continue_mode(&mut self) {
        match self.flow.sub {
            0 => self.flow_continue_setup(),
            2 => self.flow_continue_countdown(),
            4 => self.flow_continue_game_over(),
            6 => self.flow_continue_return(),
            _ => {}
        }
    }

    /// 0xEA00..0xEA3C: the setup starts before its first vblank boundary.
    pub(super) fn flow_continue_begin(&mut self) {
        self.sound(0xFF00);
        self.flow.speed = 1;
        self.flow_scene_palettes(1);
        self.brightness_full();
        self.flow_clear_scroll1();
        self.sprites.wipe();
    }

    /// 0xEA42..0xEB60: switch tilemap banks, load villain and start counting.
    fn flow_continue_setup(&mut self) {
        self.flow.cont.level2 = self.vram.scroll2.clone();
        self.flow.cont.level3 = self.vram.scroll3.clone();
        let coords = [
            self.cam.s2.cam_x,
            self.cam.s2.cam_y,
            self.cam.s3.cam_x,
            self.cam.s3.cam_y,
        ];
        for (k, v) in coords.into_iter().enumerate() {
            self.flow.cont.raw.sw(2 * k, v);
        }
        self.cam.s2.cam_x = 0;
        self.cam.s2.cam_y = 0;
        self.cam.s3.cam_x = 0;
        self.cam.s3.cam_y = 0;
        self.flow_page2(0x91F000, 0x18);
        self.flow_page2(0x91F400, 0x19);
        self.flow_page3(0x923800, 0x75);
        self.flow_page3(0x923900, 0x76);
        self.flow_continue_reset_effects();
        let r = &mut self.flow.cont.raw;
        for (off, v) in [(8, 20), (9, 100), (0x1D, 21), (0x12, 100), (0x13, 20)] {
            r.sb(off, v);
        }
        r.sw(0x16, 0);
        for j in &mut self.flow.join {
            j.seconds = 20;
            j.frames = 100;
        }
        if self.flow.b4d7 == 0 {
            self.flow_continue_digits();
            self.flow_continue_bubble();
            self.flow_continue_taunt();
        } else {
            self.flow.cont.raw.sw(8, 0xFFFF);
        }
        // 0xEB1E moveq #$1B,d0; jsr 0xA63C: d1 and d2 are what the digit, bubble and taunt text routines (0xF0A8, 0xF13E, 0x84F72) left in
        // them, the same in all 111 continues of the whole-episode recordings (1 to 3 heroes, all episodes): d1 = 0x46, d2 = 0x40. With
        // G.$4D7 set those routines are skipped and the registers are not known (the music then keeps 0 and 0).
        if self.flow.b4d7 == 0 {
            self.snd_request(0x1B, 0x46, 0x40);
        } else {
            self.sound(0x1B);
        }
        let r = &mut self.flow.cont.raw;
        for (off, v) in [(0xC, 1), (0xE, 1), (0xD, 0), (0xF, 0)] {
            r.sb(off, v);
        }
        // 0xEC92 / 0xECC4: solid backgrounds and their palette words.
        for i in 0..16 {
            self.pal.words[0x620 + i] = 0xFF00;
            self.pal.words[0x630 + i] = 0xFFFF;
        }
        for (base, n, attr) in [(0x923000, 96, 2), (0x922800, 256, 3)] {
            for k in 0..n {
                self.vram.scroll3[((base + k * 4 - 0x910000) / 4 & 0xFFF) as usize] =
                    Cell { code: 0x1C01, attr };
            }
        }
        self.flow.sub = 2;
        // 0xEB46 / 0xEB4C: scroll 2 base 0x91C0 (not modelled), scroll 3 base G.$2E = 0x9200 for the continue screen.
        self.w.scroll3_base = 0x9200;
        let hold = self.flow.front.sys.hold;
        self.flow.cont.raw.sb(0x1B, hold);
        self.flow.cont.raw.sb(0x1C, hold);
        self.flow_continue_countdown();
        // The setup crosses the preceding vblank; the first completed record also
        // contains the next countdown pass (A15, first sub-2 record).
        self.flow_continue_countdown();
    }

    /// 0x49A8 / 0x4CA4: reset the draw list and pool C, leaving the frozen level's actors.
    fn flow_continue_reset_effects(&mut self) {
        self.draw_list.clear();
        self.sprites.splash_timer = 1;
        self.effects = Pool::new(POOL_C);
    }

    /// 0xEB60 / 0xF41E / 0xF4BA / 0xF512: tick, input skip and red flash.
    fn flow_continue_countdown(&mut self) {
        if self.flow.b4d7 == 0 {
            let r = &mut self.flow.cont.raw;
            r.sb(0x1B, self.flow.front.sys.hold);
            let flash = r.b(0x12).wrapping_sub(1);
            r.sb(0x12, flash);
            if flash == 0 {
                r.sb(0x12, self.assets.frontend.text.u8(0xF544, r.b(8) as u32));
                r.sb(0x13, 1);
                self.cam.s3.cam_y = 0x100;
            } else {
                let v = r.b(0x13).wrapping_sub(1);
                r.sb(0x13, v);
                if (v as i8) < 0 {
                    self.cam.s3.cam_y = 0;
                }
            }
            if r.b(0x1B) != r.b(0x1C) {
                r.sb(8, 21);
                r.sb(0x1D, if (r.b(0x1D) as i8) <= 15 { 20 } else { 19 });
                r.sb(9, 1);
            }
            let lock = r.b(0x11).wrapping_sub(1);
            r.sb(0x11, if (lock as i8) < 0 { 0 } else { lock });
            if (lock as i8) < 0
                && (r.b(8) as i8) <= 17
                && (0..3).any(|i| self.w.input[i] & !self.flow.front.prev_input[i] & 0x30 != 0)
            {
                r.sb(0x11, 0);
                r.sb(9, 1);
            }
            let frames = r.b(9).wrapping_sub(1);
            r.sb(9, frames);
            if frames == 0 {
                self.snd_request(0x35, 0x10, 0);
                let r = &mut self.flow.cont.raw;
                r.sb(9, 60);
                if (r.b(8) as i8) >= 0 {
                    let n = r.b(8).wrapping_sub(1);
                    r.sb(8, n);
                    if n == 9 {
                        // 0xF47A: `moveq #$1C,d0; jsr 0xA63C` right after the tick 0x35 (`moveq #$10,d1; moveq #$0,d2` at 0xF45C), so
                        // d1 and d2 are still the tick's 0x10 and 0 (M4-15)
                        self.snd_request(0x1C, 0x10, 0);
                    }
                    if (n as i8) >= 0 {
                        self.flow_continue_digits();
                    }
                }
            }
            let hold = self.flow.cont.raw.b(0x1B);
            self.flow.cont.raw.sb(0x1C, hold);
        }
        self.flow_continue_face(); // 0xEFE4, even on the expiry/accept frame.
        if self.flow.started != 0 {
            self.flow.sub = 6;
            self.flow.sub2 = 0;
        } else if (self.flow.cont.raw.w(8) as i16) < 0 {
            self.flow.b4db = 1;
            self.flow.b4d7 = 1;
            self.flow.sub = 4;
        }
    }

    /// 0xEBA6..0xEE6C: red flash, countdown digit spawns, palette drain and GAME OVER.
    fn flow_continue_game_over(&mut self) {
        match self.flow.sub2 >> 8 {
            0 => {
                self.flow.sub2 = 0x200;
                self.flow.cont.raw.sb(0xA, 100);
                self.flow.cont.raw.sb(0x10, 0);
                return;
            }
            2 => match self.flow.sub2 as u8 {
                0 => {
                    self.flow.sub2 += 2;
                    // 0xF118: erase the speech rectangle on scroll2.
                    for col in 0..8 {
                        for row in 0..5 {
                            let k = ((0x91F00C + col * 0x40 + row * 4 - 0x90C000) / 4 & 0xFFF) as usize;
                            self.vram.scroll2[k] = Cell { code: 0x5800, attr: 0 };
                        }
                    }
                    self.flow_clear_scroll1();
                    self.flow.cont.raw.sb(9, 0);
                    self.flow.cont.raw.sb(0x10, 5);
                }
                2 => {
                    if self.flow.chosen != 0 {
                        self.flow.sub = 6;
                        self.flow.sub2 = 0;
                    } else {
                        let n = self.flow.cont.raw.b(9);
                        if n == 0 {
                            self.flow_continue_red_flash();
                        } else if n == 2 {
                            self.cam.s3.cam_y = 0;
                        } else if n == 5 && self.effects.free_count() > 2 {
                            if let Some(i) = self.effects.alloc() {
                                let index = self.flow.cont.raw.b(0x10) as u32 * 4;
                                let t = &self.assets.frontend.text;
                                let b = &mut self.effects.slots[i].b;
                                b.live = 1;
                                b.k20 = 0x44;
                                b.var26 = 0;
                                b.x = (t.u16(0xED40, index) as i32) << 16;
                                b.y = (t.u16(0xED40, index + 2) as i32) << 16;
                                b.z = 0;
                            }
                        }
                        let n = n.wrapping_add(1);
                        self.flow.cont.raw.sb(9, n);
                        if (n as i8) >= 18 {
                            self.flow.cont.raw.sb(9, 0);
                            let n = self.flow.cont.raw.b(0x10).wrapping_sub(1);
                            self.flow.cont.raw.sb(0x10, n);
                            if (n as i8) < 0 {
                                self.flow.sub2 += 2;
                                self.flow.cont.raw.sb(0x19, 16);
                                self.flow.cont.raw.sb(0x1A, 0);
                                self.flow_continue_red_flash();
                            }
                        }
                    }
                }
                4 => {
                    if self.flow.chosen != 0 {
                        self.flow.sub = 6;
                        self.flow.sub2 = 0;
                    } else {
                        self.cam.s3.cam_y = 0;
                        let n = self.flow.cont.raw.b(0x1A);
                        self.flow.cont.raw.sb(0x1A, n.wrapping_sub(1));
                        if n == 0 {
                            self.flow.cont.raw.sb(0x1A, 4);
                            let n = self.flow.cont.raw.b(0x19).wrapping_sub(1);
                            self.flow.cont.raw.sb(0x19, n);
                            if (n as i8) < 0 {
                                self.flow.sub2 += 2;
                                // The wait step with a name entry plays music 0x1F, one frame later (arm 6).
                                if self.flow.join.iter().any(|j| j.state == 0xA) {
                                    self.flow.cont.raw.sb(0x18, 1);
                                }
                            } else {
                                self.flow_continue_palette();
                            }
                        }
                    }
                }
                6 => {
                    // 0xEFC2: name entry owns state 0xA. M2-22 resumes here when it ends.
                    if !self.flow.join.iter().any(|j| j.state == 0xA) {
                        self.flow.sub2 += 2;
                        self.flow.cont.raw.sw(0x14, 200);
                        self.sound(0x1D);

                        for var in [0, 4] {
                            if let Some(i) = self.effects.alloc() {
                                let b = &mut self.effects.slots[i].b;
                                b.live = 1;
                                b.k20 = 0x40;
                                b.var26 = var;
                            }
                        }
                    }
                    if self.flow.cont.raw.b(0x18) == 1 {
                        self.flow.cont.raw.sb(0x18, 2);
                        self.flow.b5e4 = 2; // G.$5E4 = 2 with the name entry music (D4d A17: 1694)
                        self.sound(0x1F);
                    }
                }
                8 => {
                    // Sub 8: the 0xC labels of the finished entries go to 0 (M2-22, D4d A17: 2146 and 2147).
                    for j in self.flow.join.iter_mut() {
                        if j.state == 0xC {
                            j.state = 0;
                        }
                    }
                    if self.flow.join.iter().any(|j| j.state == 4) {
                        self.flow.sub = 6;
                        self.flow.sub2 = 0x400;
                    } else {
                        let n = self.flow.cont.raw.w(0x14).wrapping_sub(1);
                        self.flow.cont.raw.sw(0x14, n);
                        if n == 0 {
                            self.flow.sub2 += 0x200;
                        }
                    }
                }
                _ => {}
            },
            4 => {
                if self.flow.cont.exit_wait == 0 {
                    self.flow.cont.exit_wait = 90; // 0xA82 (60) followed by 0xA7E (30).
                } else {
                    self.flow.cont.exit_wait -= 1;
                    if self.flow.cont.exit_wait == 0 {
                        self.flow.mode = 0x14;
                        self.flow.sub = 0;
                        self.flow.sub2 = 0;
                        self.flow_clear_screen();
                        self.flow.cont.exit_wait = 4; // 0xAE0 clear and 0xA76.
                    }
                }
                return;
            }
            _ => return,
        }
        self.pool_c_update();
        self.build_sprite_list();
    }

    /// 0xEC5E: the red flash (sound 0xE6, class 6 variant 4), also used after the last digit loop.
    fn flow_continue_red_flash(&mut self) {
        self.snd_request(0xE6, 0x72020, 0x10000000);
        if let Some(i) = self.effects.alloc() {
            let b = &mut self.effects.slots[i].b;
            b.live = 1;
            b.k20 = 0x18;
            b.var26 = 4;
        }
        self.cam.s3.cam_y = 0x100;
    }

    /// 0xED86 / 0xB91A: lower each RGB nibble toward the six ROM palette targets.
    fn flow_continue_palette(&mut self) {
        let t = &self.assets.frontend.text;
        for pair in 0..6 {
            let at = t.u32(0xFAF8, pair * 8);
            let src = t.u32(0xFAF8, pair * 8 + 4);
            for k in 0..16 {
                let dst = &mut self.pal.words[((at - 0x914000) / 2 + k) as usize];
                let target = t.u16(src, k * 2);
                for shift in [0, 4, 8] {
                    if (*dst >> shift) & 15 > (target >> shift) & 15 {
                        *dst = dst.wrapping_sub(1 << shift);
                    }
                }
            }
        }
    }

    /// 0xF13E: change the speech only on multiples of five at or below the last taunt.
    fn flow_continue_taunt(&mut self) {
        let n = self.flow.cont.raw.b(8);
        if (n as i8) > (self.flow.cont.raw.b(0x1D) as i8) || n % 5 != 0 {
            return;
        }
        self.flow.cont.raw.sb(0x1D, n);
        self.flow_continue_bubble();
        let t = &self.assets.frontend.text;
        let base = if self.flow.region.0 == 0 {
            0xF194
        } else {
            0xF27C
        };
        let addr = base + t.u16(base, 2 * (n / 5) as u32) as u32;
        if self.flow.region.0 == 0 {
            self.text_big(t.slice_from(addr), None, true);
        } else {
            self.text_tall(t.slice_from(addr));
        }
    }

    /// 0xF0A8 / 0xF0DC: 16 rows of speech frame, preserving unrelated attribute bits.
    fn flow_continue_bubble(&mut self) {
        let t = &self.assets.frontend.text;
        for col in 0..16u32 {
            let block = if col == 0 {
                0
            } else if col == 15 {
                2
            } else {
                1
            };
            for row in 0..10u32 {
                let at = 0x90A018 + col * 0x80 + row * 4;
                let idx = ((at - 0x908000) / 4) as usize;
                let attr = self.vram.scroll1[idx].attr & 0xFE7F
                    | ((t.u8(0xFAA6, block * 10 + row) as u16) << 5);
                self.hud_put(at, t.u16(0xFA6A, block * 20 + row * 2), attr);
            }
        }
    }

    /// 0xF488 / 0xF55A: two digits, each six columns by ten rows.
    fn flow_continue_digits(&mut self) {
        self.flow_continue_taunt();
        let t = &self.assets.frontend.text;
        let n = self.flow.cont.raw.b(8) as u32;
        for (k, at) in [(0, 0x90B1A8), (1, 0x90B4A8)] {
            let digit = t.u8(0xF57C, 2 * n + k) as u32;
            let base = 0xF5A6 + t.u16(0xF5A6, 2 * digit) as u32;
            for col in 0..6u32 {
                for row in 0..10u32 {
                    self.hud_put(
                        at + col * 0x80 + row * 4,
                        t.u16(base, col * 20 + row * 2),
                        10,
                    );
                }
            }
        }
    }

    /// 0xEFE4 / 0xF048 / 0xF07E: mouth and eye tile-code animation.
    fn flow_continue_face(&mut self) {
        let t = &self.assets.frontend.text;
        for (timer, phase) in [(0xC, 0xD), (0xE, 0xF)] {
            let r = &mut self.flow.cont.raw;
            let next = r.b(timer).wrapping_sub(1);
            r.sb(timer, next);
            if next != 0 {
                continue;
            }
            if timer == 0xC {
                let mut pos = r.w(0x16) as u32;
                if (t.u8(0xF026, pos) as i8) < 0 {
                    pos = 0;
                }
                r.sb(phase, t.u8(0xF026, pos));
                r.sb(timer, t.u8(0xF026, pos + 1));
                r.sw(0x16, pos.wrapping_add(2) as u16);
            } else {
                r.sb(timer, 180);
                r.sb(phase, (r.b(phase) + 4) & 4);
            }
            let (ptr, at, offsets): (u32, u32, &[u32]) = if timer == 0xC {
                (0xF072, 0x91F360, &[0, 4, 8, 0x40, 0x44, 0x48])
            } else {
                (0xF0A0, 0x91F318, &[0, 4, 0x40, 0x44])
            };
            let base = t.u32(ptr, r.b(phase) as u32);
            for (k, off) in offsets.iter().enumerate() {
                self.vram.scroll2[((at + off - 0x90C000) / 4 & 0xFFF) as usize].code =
                    t.u16(base, 2 * k as u32);
            }
        }
    }

    /// 0xEE6E / 0xEE82 / 0xEEEA / 0xEF02: forty sliding frames, then restore the level.
    fn flow_continue_return(&mut self) {
        if self.flow.sub2 == 0 {
            if !self.flow.cont.return_pending {
                self.flow_clear_scroll1();
                self.sprites.wipe();
                self.flow.cont.return_pending = true;
                return;
            }
            self.flow.cont.return_pending = false;
            self.flow_page2(0x91F000, 0x1A);
            self.flow_page2(0x91F400, 0x13);
            for k in 0..8 {
                self.flow_page3(0x923800 + 0x100 * k, 0x77);
            }
            self.cam.s3.cam_y = 0;
            if let Some(i) = self.effects.alloc() {
                // 0xEEC2 / 0x4884
                let b = &mut self.effects.slots[i].b;
                b.live = 1;
                b.k20 = 0x18;
                b.var26 = 0;
            }
            self.flow.cont.raw.sw(0xA, 40);
            self.sound(0xFF00);
            self.flow.sub2 = 0x200;
            // The first completed picture record contains two passes of 0xEEEA.
            self.flow_continue_return();
        }
        let n = self.flow.cont.raw.w(0xA).wrapping_sub(1);
        self.flow.cont.raw.sw(0xA, n);
        if n != 0xFFFF {
            self.cam.s3.cam_x = self.cam.s3.cam_x.wrapping_add(11);
            self.pool_c_update();
            self.build_sprite_list();
            return;
        }
        self.flow.sub2 = self.flow.sub2.wrapping_add(0x200);
        self.load_all_palettes();
        self.brightness_full();
        self.cam.s2.cam_x = self.flow.cont.raw.w(0);
        self.cam.s2.cam_y = self.flow.cont.raw.w(2);
        self.cam.s3.cam_x = self.flow.cont.raw.w(4);
        self.cam.s3.cam_y = self.flow.cont.raw.w(6);
        if !self.flow.cont.level2.is_empty() {
            std::mem::swap(&mut self.vram.scroll2, &mut self.flow.cont.level2);
            std::mem::swap(&mut self.vram.scroll3, &mut self.flow.cont.level3);
            self.flow.cont.level2.clear();
            self.flow.cont.level3.clear();
        }
        self.sprites.wipe();
        // 0xEF44 / 0xEF4A: scroll 2 base back to 0x90C0 (not modelled) and G.$2E to 0x9100, then the mode 0xC.
        self.w.scroll3_base = 0x9100;
        let saved = self.flow.saved_mode;
        self.flow.mode = 0xC;
        self.flow.sub = saved[1];
        self.flow.sub2 = u16::from_be_bytes([saved[2], saved[3]]);
        self.spawn.g.b4d5 = 0;
        self.flow_area_music();
        for i in 0..3 {
            if self.flow.join[i].state == 8 {
                self.flow_continue_draw_hud(i);
            }
        }
        self.flow_continue_reset_effects();
    }
    /// 0x92C9A / 0x92CC4 / 0x92D5E: class 6 variant 0, the return flash.
    /// Variant 4 is the game-over red flash (`effect.rs`, 0x92CF6).
    pub(crate) fn flow_continue_flash(&mut self, i: usize) {
        let e = &mut self.effects.slots[i];
        if e.b.mode == 0 {
            e.b.visible = 1;
            e.b.x = 0x80 << 16;
            e.b.y = 0;
            let fx = &self.assets.effects;
            crate::game::effect::fx_load(&mut e.b, fx, fx.frame_at(0x92E12));
            e.b.facing = 0;
            e.b.draw_flags = 0x10;
            e.b.mode = 2;
            return;
        }
        if e.b.sub == 0 {
            if (e.b.yi() as u16) < 0x80 {
                e.b.y = e.b.y.wrapping_add(8 << 16);
                self.draw_list_add_if_visible(self.effects.handle(i));
                return;
            }
            e.b.sub = 2;
            self.cam.s3.cam_y = 0x200;
            e.owner58 = (e.owner58 & 0xFF) | 0x500; // +$58 is a byte counter.
            self.snd_request(0xAB, 0x72020, 0x1C000000);
            self.snd_request(0x79, 0x72020, 0x1C000000);
            self.cam_shake(2); // 0x92DBC
        }
        let e = &mut self.effects.slots[i];
        let left = ((e.owner58 >> 8) as u8).wrapping_sub(1);
        e.owner58 = (e.owner58 & 0xFF) | ((left as u16) << 8);
        if left == 0 {
            self.cam.s3.cam_y = 0;
        }
        self.draw_list_add_if_visible(self.effects.handle(i));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::TraceReader;
    use dino_assets::AssetPack;
    use std::{path::Path, sync::OnceLock};
    use crate::game::flow::mame_check::listed_args;

    fn assets() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let p = Path::new("../../../dino.zip");
            p.exists()
                .then(|| &*Box::leak(Box::new(dino_extract::extract(p).unwrap())))
        })
    }

    /// D4d A16: uninterrupted game-over scene through 0xAC06 / 0xAC26.
    #[test]
    fn continue_a16_game_over_matches_mame() {
        let Some(a) = assets() else { return };
        for name in ["flow_k-1p", "flow_k-mash", "flow_k-nocoin-start"] {
            let path = format!("../../../re/out/traces/{name}.mame.trc");
            if !Path::new(&path).exists() { continue; }
            let mut rd = TraceReader::open(&path).unwrap();
            let mut game: Option<Game> = None;
            let mut checked = 0;
            let mut sound_pos = 0u16;
            while let Some(f) = rd.next() {
                let ram = &rd.region("ram").unwrap().data;
                let gfx = &rd.region("gfx").unwrap().data;
                if let Some(g) = &mut game {
                    g.w.system = ram[0x801A];
                    g.w.coins = ram[0x8020];
                    g.w.input.copy_from_slice(&ram[0x8016..0x8019]);
                    g.tick();
                    assert_eq!((g.flow.mode, g.flow.sub, g.flow.sub2, g.flow.in_game),
                        (ram[0x84D0], ram[0x84D1], u16::from_be_bytes([ram[0x84D2], ram[0x84D3]]), ram[0x84CC]),
                        "{name} frame {f} mode");
                    if g.flow.mode == 0x10 {
                        assert_eq!(&g.flow.cont.raw.0[..0x1E], &ram[0x85CC..0x85EA], "{name} frame {f} scene state");
                        let actual = g.trace_ram();
                        for i in 0..POOL_C.2 {
                            let at = POOL_C.0 as usize + POOL_C.1 as usize * i;
                            assert_eq!(actual.ram[at], ram[at], "{name} frame {f} effect {i} presence");
                            if ram[at] == 0 { continue; }
                            for k in 0..0x68 {
                                if actual.mask[at + k] != 0 {
                                    assert_eq!(actual.ram[at + k], ram[at + k], "{name} frame {f} effect {i} +{k:X}");
                                }
                            }
                        }
                        assert_eq!(g.w.rng, u16::from_be_bytes([ram[0x8014], ram[0x8015]]), "{name} frame {f} RNG");
                        for (k, word) in g.pal.words.iter().enumerate() {
                            let off = 0x14000 + 2 * k;
                            assert_eq!(*word, u16::from_be_bytes([gfx[off], gfx[off + 1]]), "{name} frame {f} palette {k:X}");
                        }
                    }
                    let word = |bytes: &[u8], o: usize| u16::from_be_bytes([bytes[o], bytes[o + 1]]);
                    let end = word(ram, 0x8034) & 0x3F0;
                    let mut sounds = Vec::new();
                    let mut args = Vec::new();
                    let long = |o: usize| u32::from_be_bytes([ram[o], ram[o + 1], ram[o + 2], ram[o + 3]]);
                    while sound_pos != end {
                        let e = 0x803A + sound_pos as usize;
                        sounds.push(word(ram, e));
                        args.push((word(ram, e), long(e + 2), long(e + 6)));
                        sound_pos = (sound_pos + 16) & 0x3F0;
                    }
                    assert_eq!(g.w.sounds, sounds, "{name} frame {f} sounds");
                    // M4-15: d1 and d2 of the listed codes (game over 0xE6 / 0x1D / 0x1F) against the ring
                    let port: Vec<_> = g.w.snd_reqs.iter().map(|r| (r.code, r.d1, r.d2)).collect();
                    assert_eq!(listed_args(&args), listed_args(&port), "{name} frame {f} sound arguments");
                    if g.flow.mode == 0x10 && g.flow.sub2 >> 8 == 2 {
                        let base = if word(ram, 0xE9B4) != 0 { 0x4000 } else { 0 };
                        let mut expected = Vec::new();
                        for i in 0..256 {
                            let off = base + 8 * i;
                            if word(gfx, off + 6) == 0xFF00 { break; }
                            expected.push((word(gfx, off), word(gfx, off + 2), word(gfx, off + 4), word(gfx, off + 6)));
                        }
                        if expected.len() == 256 { expected.clear(); }
                        let actual: Vec<_> = g.sprites.entries.iter().map(|e| (e.x as u16, e.y as u16, e.code, e.attr)).collect();
                        assert_eq!(actual, expected, "{name} frame {f} sprite list");
                    }
                    checked += 1;
                    if g.flow.in_game == 0 { break; }
                } else if ram[0x84D0] == 0x10 && ram[0x84D1] == 4 && ram[0x84D2] == 0 {
                    let mut g = Game::new(a);
                    g.trace_load(ram, gfx);
                    sound_pos = u16::from_be_bytes([ram[0x8034], ram[0x8035]]) & 0x3F0;
                    game = Some(g);
                }
            }
            assert!(checked > 0);
            eprintln!("{name}: A16 equal for {checked} frames");
        }
    }

    #[test]
    fn continue_credit_edges_preserve_hero_score_and_rng() {
        let Some(a) = assets() else { return };
        for (common, free, credits, start, attack, expected) in [
            (false, false, 1, 0, 0x10, false),
            (false, false, 0, 1, 0, false),
            (false, false, 1, 1, 0, true),
            (false, false, 1, 2, 0x10, true), // other start plus own attack
            (true, false, 1, 2, 0x10, false),
            (true, false, 1, 1, 0, true),
            (false, true, 0, 1, 0, true),
        ] {
            let mut g = Game::new(a);
            g.flow.mode = 0x10;
            g.flow.sub = 2;
            g.flow.join[0].state = 8;
            g.flow.front.sys.common = common;
            g.flow.front.sys.free_play = free;
            g.flow.front.sys.credits = credits;
            g.flow.front.sys.slots[0].credits = credits;
            g.flow.front.sys.two_coin_start = true; // continues still cost one
            g.players[0].b.id = 2;
            g.players[0].score = 0x123450;
            g.w.rng = 0x3210;
            g.w.system = start;
            g.w.input[0] = attack;
            g.rank_set_time(0x100);
            g.spawn.g.wave_timer = 0xFF00;
            g.flow_continue_player(0);
            assert_eq!(g.flow.join[0].state == 4, expected);
            assert_eq!(g.w.rng, 0x3210);
            assert_eq!(g.players[0].b.id, 2);
            if expected {
                assert_eq!(g.players[0].score, 0x123451);
                assert_eq!((g.rank.time_min, g.rank.time_sec), (4, 0));
                assert_eq!(g.spawn.g.wave_timer, 0x158);
                assert_eq!(g.players[0].b.hp, 100);
            }
        }
        for seconds in [20, 19, 18, 0] {
            let mut g = Game::new(a);
            g.flow.mode = 0xC;
            g.flow.join[0] = super::super::join::Join {
                state: 8,
                seconds,
                frames: 10,
            };
            g.w.input[0] = 0x20;
            g.flow_continue_player(0);
            assert_eq!(
                g.flow.join[0].seconds,
                if seconds == 18 { 17 } else { seconds }
            );
        }
    }

    /// D4d A15: keep state between ticks, from mode entry through expiry or restore.
    #[test]
    fn continue_a15_runs_without_snapshot_resets() {
        let Some(a) = assets() else { return };
        for name in ["flow_k-1p", "flow_k-mash", "flow_k-coin-late"] {
            let path = format!("../../../re/out/traces/{name}.mame.trc");
            if !Path::new(&path).exists() {
                continue;
            }
            let mut rd = TraceReader::open(&path).unwrap();
            let mut game: Option<Game> = None;
            let mut count = 0;
            while let Some(f) = rd.next() {
                let ram = &rd.region("ram").unwrap().data;
                let gfx = &rd.region("gfx").unwrap().data;
                if let Some(g) = &mut game {
                    g.w.system = ram[0x801A];
                    g.w.coins = ram[0x8020];
                    g.w.input.copy_from_slice(&ram[0x8016..0x8019]);
                    g.tick();
                    assert_eq!(
                        (g.flow.mode, g.flow.sub),
                        (ram[0x84D0], ram[0x84D1]),
                        "{name} frame {f} continuous mode"
                    );
                    assert_eq!(
                        g.flow.cont.raw.w(8),
                        u16::from_be_bytes([ram[0x85D4], ram[0x85D5]]),
                        "{name} frame {f} continuous countdown"
                    );
                    for i in 0..3 {
                        assert_eq!(
                            g.flow.join[i].state,
                            ram[0xB316 + 0x180 * i],
                            "{name} frame {f} continuous P{i}"
                        );
                    }
                    count += 1;
                    if g.flow.mode == 0xC || g.flow.sub == 4 {
                        break;
                    }
                } else if ram[0x84D0] == 0x10 && ram[0x84D1] == 0 {
                    let mut g = Game::new(a);
                    g.trace_load(ram, gfx);
                    game = Some(g);
                }
            }
            assert!(count > 0);
            eprintln!("{name}: continuous countdown/accept/restore equal for {count} frames");
        }
    }

    /// D4d A15: one task pass from each previous MAME snapshot, as in A14.
    #[test]
    fn continue_a15_matches_mame() {
        let Some(a) = assets() else { return };
        for name in [
            "flow_k-1p",
            "flow_k-mash",
            "flow_k-coin-late",
            "flow_k-coin2",
            "flow_k-nocoin-start",
            "flow_k2-timeout",
            "flow_k2-coin",
            "flow_k2-both",
        ] {
            let path = format!("../../../re/out/traces/{name}.mame.trc");
            if !Path::new(&path).exists() {
                continue;
            }
            let mut rd = TraceReader::open(&path).unwrap();
            let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
            let mut checked = 0;
            while let Some(f) = rd.next() {
                let ram = &rd.region("ram").unwrap().data;
                let gfx = &rd.region("gfx").unwrap().data;
                if let Some((pr, pg)) = &previous {
                    let scene = pr[0x84D0] == 0x10 && matches!(pr[0x84D1], 0 | 2 | 6);
                    let hud = pr[0x84D0] == 0xC && (0..3).any(|i| pr[0xB316 + 0x180 * i] == 8);
                    if scene || hud {
                        let mut g = Game::new(a);
                        g.trace_load(pr, pg);
                        g.w.system = ram[0x801A];
                        g.w.coins = ram[0x8020];
                        g.w.input.copy_from_slice(&ram[0x8016..0x8019]);
                        g.flow_coin_isr();
                        g.flow_system_task();
                        g.flow_join_task();
                        if g.flow.mode == 0x10 && g.flow.wait <= 1 {
                            g.flow_continue_mode();
                        }
                        assert_eq!(
                            (g.flow.mode, g.flow.sub, g.flow.sub2),
                            (
                                ram[0x84D0],
                                ram[0x84D1],
                                u16::from_be_bytes([ram[0x84D2], ram[0x84D3]])
                            ),
                            "{name} frame {f} mode"
                        );
                        for i in 0..3 {
                            if pr[0xB316 + 0x180 * i] != 8 {
                                continue;
                            }
                            let at = 0xB274 + 0x180 * i;
                            assert_eq!(
                                (
                                    g.flow.join[i].state,
                                    g.flow.join[i].seconds,
                                    g.flow.join[i].frames
                                ),
                                (ram[at + 0xA2], ram[at + 0xB0], ram[at + 0xB1]),
                                "{name} frame {f} P{i} state/countdown"
                            );
                            assert_eq!(
                                g.players[i].score,
                                u32::from_be_bytes(ram[at + 0x100..at + 0x104].try_into().unwrap()),
                                "{name} frame {f} score"
                            );
                        }
                        if scene && g.flow.mode == 0x10 && matches!(g.flow.sub, 2 | 6) {
                            assert_eq!(
                                &g.flow.cont.raw.0[..0x1E],
                                &ram[0x85CC..0x85EA],
                                "{name} frame {f} continue state"
                            );
                        }
                        // HUD and speech/digits are written only by these tasks. In a running
                        // level skip the HUD's health/score rows, updated later by game_frame.
                        for col in 0..48 {
                            if !scene && pr[0xB316 + 0x180 * (col / 16)] != 8 {
                                continue;
                            }
                            for row in if scene { 32..64 } else { 34..38 } {
                                let idx = crate::game::vram::Vram::index1(col, row);
                                let off = 0x8000 + idx * 4;
                                let c = g.vram.scroll1[idx];
                                assert_eq!(
                                    (c.code, c.attr),
                                    (
                                        u16::from_be_bytes([gfx[off], gfx[off + 1]]),
                                        u16::from_be_bytes([gfx[off + 2], gfx[off + 3]])
                                    ),
                                    "{name} frame {f} HUD {col},{row}"
                                );
                            }
                        }
                        if scene {
                            for (coord, off) in [
                                (g.cam.s2.cam_x, 0x8744),
                                (g.cam.s2.cam_y, 0x8748),
                                (g.cam.s3.cam_x, 0x87E4),
                                (g.cam.s3.cam_y, 0x87E8),
                            ] {
                                assert_eq!(
                                    coord,
                                    u16::from_be_bytes([ram[off], ram[off + 1]]),
                                    "{name} frame {f} scroll {off:X}"
                                );
                            }
                            for (k, p) in g.pal.words.iter().enumerate() {
                                let off = 0x14000 + 2 * k;
                                assert_eq!(
                                    *p,
                                    u16::from_be_bytes([gfx[off], gfx[off + 1]]),
                                    "{name} frame {f} palette {k:X}"
                                );
                            }
                            if pr[0x84D1] == 0 {
                                for (cells, base, ranges) in [
                                    (&g.vram.scroll2, 0x1C000, vec![0x3000..0x3800]),
                                    (
                                        &g.vram.scroll3,
                                        0x20000,
                                        vec![0x2800..0x2C00, 0x3000..0x3180, 0x3800..0x3A00],
                                    ),
                                ] {
                                    for range in ranges {
                                        for k in (range.start / 4)..(range.end / 4) {
                                            let c = cells[k];
                                            let off = base + 4 * k;
                                            assert_eq!(
                                                (c.code, c.attr),
                                                (
                                                    u16::from_be_bytes([gfx[off], gfx[off + 1]]),
                                                    u16::from_be_bytes([
                                                        gfx[off + 2],
                                                        gfx[off + 3]
                                                    ])
                                                ),
                                                "{name} frame {f} setup cell {off:X}"
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        if scene && pr[0x84D1] != 0 {
                            let word = |a: &[u8], o: usize| u16::from_be_bytes([a[o], a[o + 1]]);
                            let mut pos = word(pr, 0x8034) & 0x3F0;
                            let end = word(ram, 0x8034) & 0x3F0;
                            let mut sounds = Vec::new();
                            while pos != end {
                                sounds.push(word(ram, 0x803A + pos as usize));
                                pos = (pos + 16) & 0x3F0;
                            }
                            assert_eq!(g.w.sounds, sounds, "{name} frame {f} sounds");
                        }
                        if scene && pr[0x84D1] == 2 && g.flow.sub == 2 {
                            for (cells, base) in
                                [(&g.vram.scroll2, 0x1C000), (&g.vram.scroll3, 0x20000)]
                            {
                                for (k, c) in cells.iter().enumerate() {
                                    let off = base + 4 * k;
                                    assert_eq!(
                                        (c.code, c.attr),
                                        (
                                            u16::from_be_bytes([gfx[off], gfx[off + 1]]),
                                            u16::from_be_bytes([gfx[off + 2], gfx[off + 3]])
                                        ),
                                        "{name} frame {f} scene cell {off:X}"
                                    );
                                }
                            }
                        }
                        checked += 1;
                    }
                }
                previous = Some((ram.clone(), gfx.clone()));
            }
            assert!(checked > 0, "{name}: no countdown records");
            eprintln!("{name}: continue state, timers and HUD equal for {checked} frames");
        }
    }
}
