//! Camera, background streaming and parallax: 0x2042E and the scroll2 / scroll3 controllers.
//! Source: `re/specs/stage/SPEC.md` sections 5-8, checked against the listings at 0x206D4,
//! 0x2099A, 0x20A22, 0x20BF4..0x21266, 0x2126E..0x2152C and 0x22CAA..0x2311C.

use super::Game;
use dino_assets::stage::{AreaMap, Cell};

/// Scroll2 controller (struct at G.$73C). Field names carry the original offset.
#[derive(Clone, Debug, Default)]
pub struct Scroll2 {
    /// +$04 task state (0 init, 2 run), +$05 camera mode, +$06 mode sub-state.
    pub state: u8,
    pub mode: u8,
    pub sub: u8,
    /// +$08 / +$0C camera X / Y (16.16; high word here, low word in `*_frac`). G.$744 / G.$748.
    pub cam_x: u16,
    pub cam_x_frac: u16,
    pub cam_y: u16,
    pub cam_y_frac: u16,
    /// +$14 / +$16: camera X / Y before this frame's follow; velocity X / Y in mode 5.
    pub w14: u16,
    pub w16: u16,
    /// +$18 velocity Z, +$1A..+$1E accelerations (mode 5).
    pub w18: u16,
    pub a1a: i16,
    pub a1c: i16,
    pub a1e: i16,
    /// Area whose map header is loaded (+$40..+$42 come from it).
    pub map_area: u8,
    /// +$46 / +$48 last streamed X / Y (multiples of 32).
    pub sx46: u16,
    pub sy48: u16,
    /// +$4A area (G.$786), +$4B vertical follow enabled.
    pub area: u8,
    pub vfollow: u8,
    /// +$4C / +$4E event X / Y, +$50 push threshold, +$52 / +$54 vertical thresholds.
    pub x4c: u16,
    pub y4e: u16,
    pub push50: u16,
    pub up52: u16,
    pub down54: u16,
    /// +$56 pointer to the next event record (here an index into the stage's event list).
    pub next_event: usize,
    /// +$5A camera X when the event loaded (= event X), +$5C camera Y when it loaded.
    pub x5a: u16,
    pub y5c: u16,
    /// +$60..+$66 player extents.
    pub max_x60: u16,
    pub max_y62: u16,
    pub min_x64: u16,
    pub min_y66: u16,
    /// +$6C event index, +$6D go flag (set by stage flow code, cleared by the camera).
    pub event_idx: u8,
    pub go: u8,
    /// +$70 event overrides the push threshold.
    pub flag70: u8,
    /// +$71..+$75 tile animation timers and phase.
    pub t71: u8,
    pub phase72: u8,
    pub prev73: u8,
    pub t75: u8,
    /// +$78 lock state: 0 free, 1 locked (wave / area end), 2 stage end.
    pub lock78: u8,
    /// +$79 events trigger on Y (also freezes scroll3 parallax), +$7B mode after a mode-4 lock.
    pub by_y79: u8,
    pub next_mode7b: u8,
}

/// Scroll3 controller (struct at G.$7DC).
#[derive(Clone, Debug, Default)]
pub struct Scroll3 {
    pub state: u8,
    pub b5: u8,
    pub cam_x: u16,
    pub cam_y: u16,
    pub map_area: u8,
    pub sx46: u16,
    pub sy48: u16,
    pub x5a: u16,
    pub y5c: u16,
    pub shift5f: u8,
}

#[derive(Clone, Debug)]
pub struct Camera {
    pub s2: Scroll2,
    pub s3: Scroll3,
    /// Scroll1 camera (G.$6C4 / $6C8). In game it is (0, 0): 0xAE0 clears it at stage start and
    /// the HUD lives at scroll1 rows 34+ (0x90A014). 0xAC4E sets Y = 0x100 for a non-game screen.
    pub cam1_x: u16,
    pub cam1_y: u16,
    /// G.$755C: shake strength requested this frame (consumed by 0x222D6, see `cam_shake`).
    pub shake_req: u8,
}

impl Default for Camera {
    fn default() -> Self {
        Camera { s2: Scroll2::default(), s3: Scroll3::default(), cam1_x: 0, cam1_y: 0, shake_req: 0 }
    }
}

/// Push thresholds by number of players (0x20B62).
const PUSH_BY_PLAYERS: [u16; 3] = [0xC0, 0xE8, 0xE8];

fn ge32(d: u16) -> bool {
    (d.wrapping_sub(0x20) as i16) >= 0
}

fn abs16(d: u16) -> u16 {
    if d & 0x8000 != 0 { d.wrapping_neg() } else { d }
}

fn round32(v: u16) -> u16 {
    (if v & 0x10 != 0 { v.wrapping_add(0x10) } else { v }) & 0xFFE0
}

/// Cell index of the scroll2 block at world (x, y) (0x214C0, byte offset / 4).
fn vram2(x: u16, y: u16) -> usize {
    let col = ((x & 0x3E0) << 2) as usize;
    let d2 = 0x3E0 - (y & 0x3E0);
    (col + (((d2 & 0x300) << 4) + ((d2 & 0xE0) >> 2)) as usize) / 4
}

/// Cell index of the scroll3 tile at world (x, y) (0x23090).
fn vram3(x: u16, y: u16) -> usize {
    let d2 = 0x7E0 - (y & 0x7E0);
    ((x & 0x7E0) as usize + (((d2 & 0x700) << 3) + ((d2 & 0xE0) >> 3)) as usize) / 4
}

/// Screen id lookup shared by both layers (0x214F0 / 0x230BE), with the 68000 word math.
fn screen_lookup(map: &AreaMap, xs: u16, y: u16) -> u16 {
    let col = ((xs as i16) >> 8) as u16;
    let row = ((y as i16) >> 8) as u16;
    let idx = ((col as u32 * map.h as u32) as u16).wrapping_add(row);
    let byte = idx.wrapping_mul(2) as i16 as i32;
    if byte < 0 {
        return 0;
    }
    map.screens.get((byte / 2) as usize).copied().unwrap_or(0)
}

impl Game {
    fn s2_map(&self) -> &'static AreaMap {
        &self.assets.stages.area(self.w.stage, self.cam.s2.map_area).scroll2
    }

    fn s3_map(&self) -> &'static AreaMap {
        &self.assets.stages.area(self.w.stage, self.cam.s3.map_area).scroll3
    }

    fn screen_id2(&self, x: u16, y: u16) -> u16 {
        let mut xs = x & 0xFFE0;
        if self.w.stage == 2 {
            while xs >= 0x4D00 {
                xs -= 0xF00;
            }
        }
        screen_lookup(self.s2_map(), xs, y & 0xFFE0)
    }

    fn screen_id3(&self, x: u16, y: u16) -> u16 {
        let mut xs = x;
        if self.w.stage == 2 {
            while xs >= 0xA00 {
                xs -= 0x100;
            }
        }
        screen_lookup(self.s3_map(), xs, y) & 0x7F
    }

    /// 0x2152C: word index into the stage's scroll2 screen data.
    fn block_ptr2(id: u16, x: u16, y: u16) -> usize {
        id as usize * 64 + ((x & 0xE0) >> 2) as usize + (7 - ((y & 0xE0) >> 5)) as usize
    }

    /// 0x2142A: copy one 2x2 block into scroll2 (cells i, i+1 and i+16, i+17).
    fn write_block2(&mut self, i: usize, p: usize) {
        let st = &self.assets.stages.stages[self.w.stage as usize];
        let blk = st.scroll2_screen_words.get(p).copied().unwrap_or(0);
        let cells = st.scroll2_blocks.get(((blk << 4) as i16 / 16) as usize).copied().unwrap_or([Cell::default(); 4]);
        let v = &mut self.vram.scroll2;
        v[i % 4096] = cells[0];
        v[(i + 1) % 4096] = cells[1];
        v[(i + 16) % 4096] = cells[2];
        v[(i + 17) % 4096] = cells[3];
    }

    /// 0x213F8: up to n+1 blocks downward from (x, y).
    fn draw_column2(&mut self, x: u16, mut y: u16, n: u16) {
        let mut i = vram2(x, y);
        let mut p = Self::block_ptr2(self.screen_id2(x, y), x, y);
        let mut cnt = n;
        loop {
            self.write_block2(i, p);
            p += 1;
            i += 2;
            let old_y = y;
            if y == 0 {
                return;
            }
            y = y.wrapping_sub(0x20);
            if old_y & 0xE0 == 0 {
                i = vram2(x, y);
                p = Self::block_ptr2(self.screen_id2(x, y), x, y);
            }
            if cnt == 0 {
                return;
            }
            cnt -= 1;
        }
    }

    /// 0x213B8: n+1 blocks to the right from (x, y).
    fn draw_row2(&mut self, mut x: u16, y: u16, n: u16) {
        let mut i = vram2(x, y);
        let mut p = Self::block_ptr2(self.screen_id2(x, y), x, y);
        let mut cnt = n;
        loop {
            self.write_block2(i, p);
            p += 8;
            x = x.wrapping_add(0x20);
            if x & 0x3E0 != 0 {
                i = vram2(x, y);
            }
            if x & 0xE0 == 0 {
                i = vram2(x, y);
                p = Self::block_ptr2(self.screen_id2(x, y), x, y);
            }
            if cnt == 0 {
                return;
            }
            cnt -= 1;
        }
    }

    /// 0x2126E: keep the scroll2 window filled around the camera.
    fn stream2(&mut self) {
        let s = &self.cam.s2;
        let (cx, cy, sx, sy) = (s.cam_x, s.cam_y, s.sx46, s.sy48);
        if sx < cx {
            if ge32(cx.wrapping_sub(sx)) {
                self.draw_column2(sx.wrapping_add(0x200), sy.wrapping_add(0x180), 0x11);
            }
        } else if ge32(sx.wrapping_sub(cx)) && sx != 0 {
            self.draw_column2(sx.wrapping_sub(0x40), sy.wrapping_add(0x180), 0x11);
        }
        let x0 = {
            let v = sx.wrapping_sub(0x120);
            if (v as i16) < 0 { 0 } else { v }
        };
        if sy < cy {
            if ge32(cy.wrapping_sub(sy)) {
                self.draw_row2(x0, sy.wrapping_add(0x160), 0x19);
            }
        } else if ge32(sy.wrapping_sub(cy)) {
            let y = sy.wrapping_sub(0x80);
            if (y as i16) >= 0 {
                self.draw_row2(x0, y, 0x19);
            }
        }
        let s = &mut self.cam.s2;
        if ge32(abs16(s.cam_x.wrapping_sub(s.sx46))) {
            s.sx46 = round32(s.cam_x);
        }
        if ge32(abs16(s.cam_y.wrapping_sub(s.sy48))) {
            s.sy48 = round32(s.cam_y);
        }
    }

    fn block_ptr3(id: u16, x: u16, y: u16) -> usize {
        id as usize * 64 + ((x & 0xE0) >> 5) as usize * 8 + (7 - ((y & 0xE0) >> 5)) as usize
    }

    fn write_cell3(&mut self, i: usize, p: usize) {
        if self.s3_map().w == 0 {
            return; // no far layer in this area (the original reads garbage hidden behind scroll2)
        }
        let c = self.assets.stages.scroll3_screen_cells.get(p).copied().unwrap_or_default();
        self.vram.scroll3[i % 4096] = c;
    }

    /// 0x23066: up to n+1 tiles downward from (x, y).
    fn draw_column3(&mut self, x: u16, mut y: u16, n: u16) {
        let mut i = vram3(x, y);
        let mut p = Self::block_ptr3(self.screen_id3(x, y), x, y);
        let mut cnt = n;
        loop {
            self.write_cell3(i, p);
            p += 1;
            i += 1;
            let old_y = y;
            if y == 0 {
                return;
            }
            y = y.wrapping_sub(0x20);
            if old_y & 0xE0 == 0 {
                i = vram3(x, y);
                p = Self::block_ptr3(self.screen_id3(x, y), x, y);
            }
            if cnt == 0 {
                return;
            }
            cnt -= 1;
        }
    }

    /// 0x23028: n+1 tiles to the right from (x, y).
    fn draw_row3(&mut self, mut x: u16, y: u16, n: u16) {
        let mut i = vram3(x, y);
        let mut p = Self::block_ptr3(self.screen_id3(x, y), x, y);
        let mut cnt = n;
        loop {
            self.write_cell3(i, p);
            p += 8;
            x = x.wrapping_add(0x20);
            if x & 0x7E0 != 0 {
                i = vram3(x, y);
            }
            if x & 0xE0 == 0 {
                i = vram3(x, y);
                p = Self::block_ptr3(self.screen_id3(x, y), x, y);
            }
            if cnt == 0 {
                return;
            }
            cnt -= 1;
        }
    }

    /// 0x22ED2: keep the scroll3 window filled.
    fn stream3(&mut self) {
        let s = &self.cam.s3;
        let (cx, cy, sx, sy) = (s.cam_x, s.cam_y, s.sx46, s.sy48);
        if sx < cx {
            let y = sy.wrapping_add(0x180);
            if ge32(cx.wrapping_sub(sx)) && (y as i16) >= 0 {
                self.draw_column3(sx.wrapping_add(0x200), y, 0x11);
            }
        } else {
            let y = sy.wrapping_add(0x180);
            if ge32(sx.wrapping_sub(cx)) && sx != 0 && (y as i16) >= 0 {
                self.draw_column3(sx.wrapping_sub(0x40), y, 0x11);
            }
        }
        let x0 = {
            let v = sx.wrapping_sub(0x120);
            if (v as i16) < 0 { 0 } else { v }
        };
        if sy < cy {
            let y = sy.wrapping_add(0x160);
            if ge32(cy.wrapping_sub(sy)) && (y as i16) >= 0 {
                self.draw_row3(x0, y, 0x19);
            }
        } else {
            let y = sy.wrapping_sub(0x80);
            if ge32(sy.wrapping_sub(cy)) && (y as i16) >= 0 {
                self.draw_row3(x0, y, 0x19);
            }
        }
        let s = &mut self.cam.s3;
        if ge32(abs16(s.cam_x.wrapping_sub(s.sx46))) {
            s.sx46 = round32(s.cam_x);
        }
        if ge32(abs16(s.cam_y.wrapping_sub(s.sy48))) {
            s.sy48 = round32(s.cam_y);
        }
    }

    /// 0x2042E: background controllers, called at the end of the object update.
    pub fn background_update(&mut self) {
        // Stage 2 (desert drive) runs its own scroll2 task (0x22398) and rowscroll (0x2335A); not ported yet.
        self.scroll2_task();
        self.scroll3_task();
    }

    /// 0x206D4.
    fn scroll2_task(&mut self) {
        if self.cam.s2.state == 0 {
            self.scroll2_init();
        } else {
            self.scroll2_run();
        }
    }

    /// 0x206EA: scroll2 init (state 0).
    fn scroll2_init(&mut self) {
        let s = &mut self.cam.s2;
        s.state = 2;
        s.t71 = 1;
        s.t75 = 1;
        s.phase72 = 0;
        s.prev73 = 0xFF;
        self.load_all_palettes();
        self.brightness_full_current();
        if self.w.stage == 2 {
            self.w.video_ctrl |= 1;
        } else {
            self.w.video_ctrl &= !1;
        }
        let s = &mut self.cam.s2;
        if (s.mode as i8) < 0x0E {
            s.area = 0;
        }
        s.go = 0;
        s.lock78 = 0;
        s.event_idx = 0;
        if s.mode != 0x0E && s.mode != 0x0F {
            self.load_event_index();
            self.cam.s2.next_event += 1;
            self.area_setup();
        } else {
            self.reinit_map();
        }
    }

    /// 0x2099A: load the event at the current event index.
    fn load_event_index(&mut self) {
        self.cam.s2.next_event = self.cam.s2.event_idx as usize;
        self.load_event();
    }

    /// 0x209C0: copy the event record at `next_event` into the controller.
    fn load_event(&mut self) {
        let events = &self.assets.stages.stages[self.w.stage as usize].camera_events;
        let s = &mut self.cam.s2;
        let ev = events.get(s.next_event).copied().unwrap_or_default();
        s.x4c = ev.x;
        s.y4e = ev.y;
        s.push50 = ev.push;
        s.up52 = ev.up;
        s.down54 = ev.down;
        s.area = ev.area;
        s.mode = ev.mode;
        s.sub = 0;
        s.vfollow = ev.vfollow;
        s.flag70 = ev.flag70;
        s.by_y79 = ev.by_y;
        s.x5a = ev.x;
        s.next_mode7b = ev.next_mode;
        s.y5c = s.cam_y;
        s.event_idx = s.event_idx.wrapping_add(1);
    }

    /// 0x20766: area setup (also used by modes 1 and 0x12).
    fn area_setup(&mut self) {
        self.w.vertical_scene = 0;
        self.w.layer_ctrl = 0x12D6;
        if self.w.stage == 6 && self.cam.s2.area == 1 {
            self.w.vertical_scene = 1;
            self.w.layer_ctrl = 0x06D6;
            // 0x205C6 fills the scroll1 elevator background: not ported yet.
        }
        if self.w.stage == 6 && self.cam.s2.area == 2 {
            // 0xB52 clear scroll1, 0x162AE: not ported yet.
        }
        self.reinit_map();
    }

    /// 0x207CA: load the area's scroll2 header, reset the camera and draw the first screen.
    fn reinit_map(&mut self) {
        let area = self.cam.s2.area;
        let map = &self.assets.stages.area(self.w.stage, area).scroll2;
        let s = &mut self.cam.s2;
        s.map_area = area;
        s.sx46 = map.init_x;
        s.sy48 = map.init_y;
        s.cam_x = map.init_x;
        s.cam_y = map.init_y;
        self.vram.clear2();
        self.vram.clear3();
        let (mut x, y) = (self.cam.s2.sx46, self.cam.s2.sy48);
        // When the area starts at X = 0 the original also pre-draws one screen at a fixed VRAM
        // offset (+0xBC8); its purpose is unknown and it is not ported.
        let ncols = if x != 0 {
            x = x.wrapping_sub(0x20);
            19
        } else {
            17
        };
        for _ in 0..ncols {
            self.draw_column2(x, y.wrapping_add(0x140), 0xD);
            x = x.wrapping_add(0x20);
        }
    }

    /// 0x20898: scroll2 run (state 2).
    fn scroll2_run(&mut self) {
        self.tile_anim_tick();
        let mode = self.cam.s2.mode;
        if ![0x0E, 0x0F, 0x01, 0x12, 0x09].contains(&mode) {
            let events = &self.assets.stages.stages[self.w.stage as usize].camera_events;
            let s = &self.cam.s2;
            let ev = events.get(s.next_event).copied().unwrap_or_default();
            let by_y = s.by_y79 != 0 || (self.w.stage == 6 && s.area == 1);
            let hit = if by_y { ev.y >= s.cam_y } else { ev.x <= s.cam_x };
            if hit {
                self.load_event();
                self.cam.s2.next_event += 1;
            }
        }
        self.player_extents();
        match self.cam.s2.mode {
            0x00 => {
                self.follow_x();
                self.follow_y();
                self.stream2();
            }
            0x01 => self.mode_area_end(),
            0x02 => self.mode_diagonal(false),
            0x03 => self.mode_diagonal(true),
            0x04 => self.mode_lock(),
            0x05 => self.mode_elevator(),
            0x06 => self.mode_stage_end(),
            0x07 => self.mode_descend(),
            0x08 => self.mode_sub_area(),
            0x09 => self.mode_slow_fade(),
            0x0A => {
                self.follow_x();
                let s = &mut self.cam.s2;
                s.cam_y = (s.cam_x.wrapping_sub(s.x5a) >> 2).wrapping_add(s.y5c);
                self.stream2();
            }
            0x0B => {
                self.follow_x();
                let sh = self.assets.stages.area(self.w.stage, self.cam.s2.area).slope_shift;
                let s = &mut self.cam.s2;
                s.cam_y = s.y5c.wrapping_sub(s.cam_x.wrapping_sub(s.x5a) >> sh);
                if (s.cam_y as i16) < (s.y4e as i16) {
                    s.cam_y = s.y4e;
                }
                self.stream2();
            }
            0x0C => {
                let s = &mut self.cam.s2;
                if (s.max_x60 as i16) >= 0x750 {
                    s.cam_y = s.cam_y.wrapping_sub(1);
                    if (s.cam_y as i16) < 0 {
                        s.cam_y = 0;
                    }
                }
                self.stream2();
            }
            0x0D => {
                self.follow_x();
                let s = &mut self.cam.s2;
                let mut y = (s.cam_x.wrapping_sub(s.x5a) >> 1).wrapping_add(s.y5c);
                if (y as i16) > (s.y4e as i16) {
                    y = s.y4e;
                }
                s.cam_y = y;
                self.stream2();
            }
            0x0E | 0x0F => self.stream2(),
            0x10 => {
                let s = &mut self.cam.s2;
                if s.sub == 0 {
                    s.sub = 2;
                    s.go = 0;
                }
                s.cam_x = s.x4c;
                if s.go != 0 {
                    self.w.stage_clear = 1;
                    self.cam.s2.mode = 0;
                }
                self.cam.s2.go = 0;
            }
            0x11 => {
                self.cam.s2.cam_x = self.cam.s2.cam_x.wrapping_add(3);
                self.stream2();
            }
            0x12 => self.mode_area_change(),
            0x13 => self.mode_stage_end_fade(),
            _ => {}
        }
    }

    /// 0x20956 / 0x2097C: tile animation phase (stages 4 and 7 only).
    fn tile_anim_tick(&mut self) {
        let stage = self.w.stage;
        let s = &mut self.cam.s2;
        match stage {
            4 => {
                s.t71 = s.t71.wrapping_sub(1);
                if s.t71 == 0 {
                    s.t71 = 8;
                    s.phase72 = (s.phase72 + 4) & 0xF;
                    return;
                }
            }
            7 => {
                s.t75 = s.t75.wrapping_sub(1);
                if s.t75 == 0 {
                    s.t75 = 0x18;
                    s.phase72 = (s.phase72 + 4) & 0xF;
                    return;
                }
            }
            _ => return,
        }
        s.prev73 = s.phase72;
    }

    /// 0x20A8C: extents of the live players.
    fn player_extents(&mut self) {
        let (mut max_x, mut min_x, mut max_y, mut min_y) = (0u16, 0xFFFFu16, 0u16, 0xFFFFu16);
        let mut any = 0u8;
        for p in &self.players {
            any |= p.b.live;
            if p.b.live == 0 || p.b.mode == 0 {
                continue;
            }
            let yt = (p.b.floor as u16).wrapping_add(p.b.zi() as u16);
            max_y = max_y.max(yt);
            min_y = min_y.min(yt);
            let x = p.b.xi() as u16;
            max_x = max_x.max(x);
            min_x = min_x.min(x);
        }
        let s = &mut self.cam.s2;
        if (min_x as i16) < (s.cam_x as i16) {
            min_x = s.cam_x.wrapping_add(0x10);
        }
        if any != 0 {
            s.max_x60 = max_x;
            s.max_y62 = max_y;
            s.min_y66 = min_y;
            s.min_x64 = if (min_x as i16) < 0 { 0x10 } else { min_x };
        }
    }

    /// 0x20A22: horizontal push scrolling.
    fn follow_x(&mut self) {
        let players = self.w.players_minus_one as usize;
        let s = &mut self.cam.s2;
        if s.flag70 == 0 {
            s.push50 = PUSH_BY_PLAYERS[players.min(2)];
        }
        s.w14 = s.cam_x;
        let lead = s.max_x60.wrapping_sub(s.cam_x);
        if (lead as i16) <= 0x40 || lead < s.push50 {
            return;
        }
        let spread = s.max_x60.wrapping_sub(s.min_x64);
        if spread >= s.push50.wrapping_sub(0x10) {
            let mut d = s.min_x64.wrapping_sub(0x10).wrapping_sub(s.cam_x) as i16;
            d = d.clamp(0, 6);
            s.cam_x = s.cam_x.wrapping_add(d as u16);
        } else {
            let d = (lead.wrapping_sub(s.push50) as i16).min(6);
            s.cam_x = s.cam_x.wrapping_add(d as u16);
        }
    }

    /// 0x20B68: vertical follow (only when +$4B is set). Keeps the original's overshoot quirk.
    fn follow_y(&mut self) {
        let s = &mut self.cam.s2;
        if s.vfollow == 0 {
            return;
        }
        s.w16 = s.cam_y;
        let low = s.min_y66.wrapping_sub(s.cam_y);
        let high = s.max_y62.wrapping_sub(s.cam_y);
        let step: i16 = if low > s.down54 {
            if (high as i16) < (s.up52 as i16) || (low as i16) <= 0x10 {
                return;
            }
            let d = high.wrapping_sub(s.up52) as i16;
            if d.unsigned_abs() > 6 { -6 } else { d }
        } else {
            if (high as i16) >= 0xA0 {
                return;
            }
            let d = low.wrapping_sub(s.down54) as i16;
            if d.unsigned_abs() > 6 { -6 } else { d }
        };
        s.cam_y = s.cam_y.wrapping_add(step as u16);
        if (s.cam_y as i16) < 0 {
            s.cam_y = 0;
        }
    }

    /// Mode 4 (0x21086): lock for a fight until the go flag.
    fn mode_lock(&mut self) {
        let s = &mut self.cam.s2;
        if s.sub == 0 {
            s.sub = 2;
            s.lock78 = 1;
            s.go = 0;
            if self.w.vertical_scene != 0 {
                self.sound(0x87);
            }
        }
        let s = &mut self.cam.s2;
        if s.by_y79 != 0 {
            s.cam_y = s.y4e;
        } else {
            s.cam_x = s.x4c;
        }
        if s.go != 0 {
            s.mode = s.next_mode7b;
            s.sub = 0;
            s.lock78 = 0;
        }
        s.go = 0;
    }

    /// Mode 1 (0x20BF4): end of an area with a fade. Sub-states 0/2/4/6/8 ->
    /// 0x20C0C (falls into 0x20C32), 0x20C32, 0x20C6A, 0x20CC0, 0x20CE6.
    fn mode_area_end(&mut self) {
        match self.cam.s2.sub {
            0 => {
                let s = &mut self.cam.s2;
                s.sub = 2;
                s.lock78 = 1;
                s.go = 0;
                if self.w.vertical_scene != 0 {
                    self.sound(0x87);
                }
                self.area_end_wait_go();
            }
            2 => self.area_end_wait_go(),
            4 => self.area_end_next_area(true),
            6 => self.area_end_fade_in(),
            8 => self.area_end_load_next(),
            _ => {}
        }
    }

    /// 0x20C32: hold the camera at the event X until the go flag, then fade out.
    fn area_end_wait_go(&mut self) {
        self.follow_y();
        let s = &mut self.cam.s2;
        s.cam_x = s.x4c;
        if s.go != 0 {
            s.sub += 2;
            self.hook_a5ea();
            self.fade.delay = 2;
            self.fade.mask = 0x0F;
            self.start_fade_task(false);
            return;
        }
        s.go = 0;
    }

    /// 0x20C6A (with the fade wait) / 0x20D3A (without): skip to the next event with X == 0,
    /// then reload the palettes and the map of its area.
    fn area_end_next_area(&mut self, wait_fade: bool) {
        if wait_fade && self.fade.mask != 0 {
            self.cam.s2.go = 0;
            return;
        }
        let events = &self.assets.stages.stages[self.w.stage as usize].camera_events;
        self.cam.s2.lock78 = 0;
        loop {
            let ev = events.get(self.cam.s2.next_event).copied().unwrap_or_default();
            if ev.x == 0 {
                self.cam.s2.area = ev.area;
                break;
            }
            self.cam.s2.next_event += 1;
            self.cam.s2.event_idx = self.cam.s2.event_idx.wrapping_add(1);
        }
        self.hook_acd0();
        self.load_all_palettes();
        self.cam.s3.state = 0;
        self.area_setup();
        self.cam.s2.sub += 2;
        self.hook_12be4();
        self.cam.s2.go = 0;
    }

    /// 0x20CC0: fade back in.
    fn area_end_fade_in(&mut self) {
        self.cam.s2.sub += 2;
        self.hook_162ae();
        self.fade.delay = 2;
        self.fade.mask = 0x0F;
        self.start_fade_task(true);
    }

    /// 0x20CE6: once the go flag is clear, load the next event (its mode takes over).
    fn area_end_load_next(&mut self) {
        if self.cam.s2.go == 0 {
            self.load_event_index();
            self.cam.s2.next_event += 1;
            self.cam.s2.sub = 0;
        }
    }

    /// Mode 0x12 (0x20CFE): area change without a fade. Sub-states 0/2/4/6 -> 0x20D14 (falls
    /// into 0x20D22), 0x20D22, 0x20D3A, 0x20D88.
    fn mode_area_change(&mut self) {
        match self.cam.s2.sub {
            0 | 2 => {
                if self.cam.s2.sub == 0 {
                    let s = &mut self.cam.s2;
                    s.sub = 2;
                    s.lock78 = 1;
                    s.go = 0;
                }
                self.follow_y();
                let s = &mut self.cam.s2;
                s.cam_x = s.x4c;
                if s.go != 0 {
                    s.sub += 2;
                } else {
                    s.go = 0;
                }
            }
            4 => self.area_end_next_area(false),
            6 => {
                if self.cam.s2.go == 0 {
                    self.load_event_index();
                    self.cam.s2.next_event += 1;
                    self.cam.s2.sub = 0;
                    self.brightness_full();
                }
            }
            _ => {}
        }
    }

    /// Mode 6 (0x20DA6): stage end.
    fn mode_stage_end(&mut self) {
        let s = &mut self.cam.s2;
        if s.sub == 0 {
            s.sub = 2;
            s.lock78 = 2;
            s.go = 0;
        }
        if s.sub == 2 {
            s.cam_x = s.x4c;
            if s.go != 0 {
                self.w.stage_clear = 1;
            }
        }
    }

    /// Mode 0x13 (0x20E20): stage end with a fade that keeps the HUD (mask 0x0D).
    fn mode_stage_end_fade(&mut self) {
        let s = &mut self.cam.s2;
        if s.sub == 0 {
            s.sub = 2;
            s.lock78 = 2;
            s.go = 0;
        }
        if s.sub == 2 {
            s.cam_x = s.x4c;
            if s.go != 0 {
                self.w.stage_clear = 1;
                self.cam.s2.sub = 4;
                self.fade.delay = 2;
                self.fade.mask = 0x0D;
                self.start_fade_task(false);
            }
        }
    }

    /// Mode 8 (0x20E78): switch to a sub-area without reloading the map.
    fn mode_sub_area(&mut self) {
        let s = &mut self.cam.s2;
        if s.sub == 0 {
            s.sub = 2;
            s.go = 0;
            s.lock78 = 1;
            s.cam_x = s.x4c;
        }
        if self.cam.s2.go != 0 {
            let s = &mut self.cam.s2;
            s.lock78 = 0;
            s.area = s.area.wrapping_add(1);
            s.mode = 0;
            self.load_obj_palettes();
            self.fade.mask = 1;
            self.brightness_full_current();
            self.hook_aae2();
            self.hook_1031a();
            self.hook_ace0();
            self.hook_9236c();
        }
        self.cam.s2.go = 0;
    }

    /// Mode 9 (0x2120E): slow fade transition. Sub-states 0/2/4/6 -> 0x21240, then mode 1's
    /// 0x20C6A, 0x20CC0, 0x20CE6. Always streams afterwards.
    fn mode_slow_fade(&mut self) {
        if self.cam.s2.sub < 4 && self.fade.mask != 0 {
            let s = &mut self.cam.s2;
            s.cam_y = s.cam_y.wrapping_sub(4);
            if (s.cam_y as i16) < 0 {
                s.cam_y = 0;
            }
        }
        match self.cam.s2.sub {
            0 => {
                let s = &mut self.cam.s2;
                s.cam_x = s.x4c;
                s.sub += 2;
                self.fade.delay = 12;
                self.fade.mask = 0x0F;
                self.start_fade_task(false);
            }
            2 => self.area_end_next_area(true),
            4 => self.area_end_fade_in(),
            6 => self.area_end_load_next(),
            _ => {}
        }
        self.stream2();
    }

    /// Mode 7 (0x211AE): descend with the players.
    fn mode_descend(&mut self) {
        let mut m = 0xFFFFu16;
        for p in &self.players {
            if p.b.live != 0 {
                m = m.min(p.b.yi() as u16);
            }
        }
        let s = &mut self.cam.s2;
        let t = m.wrapping_add(0x100).wrapping_sub(s.cam_y);
        if t <= s.down54 {
            s.cam_y = s.cam_y.wrapping_add(t.wrapping_sub(s.down54));
        }
        if s.cam_y <= 0x100 {
            s.cam_y = 0x100;
        }
        self.stream2();
    }

    /// Modes 2 (0x20F92, descent) and 3 (0x2100C, ascent) to the right.
    fn mode_diagonal(&mut self, ascent: bool) {
        self.follow_x();
        self.follow_y();
        let s = &mut self.cam.s2;
        if !ascent {
            let lim = s.max_y62.wrapping_sub(0xA0);
            if (lim as i16) >= (s.cam_y as i16) {
                s.cam_y = lim;
                s.cam_x = s.w14;
            }
            let mut d = s.cam_x.wrapping_sub(s.x5a) as i16;
            let d1 = s.y5c.wrapping_sub(s.cam_y) as i16;
            if d < d1 {
                d = d1;
            }
            s.cam_x = s.x5a.wrapping_add(d as u16);
            let mut y = s.y5c.wrapping_sub(d as u16);
            if (lim as i16) >= (y as i16) {
                y = lim;
                s.cam_x = s.y5c.wrapping_sub(y).wrapping_add(s.x5a);
            }
            if (y as i16) < (s.y4e as i16) {
                y = s.y4e;
                s.mode = 0;
            }
            s.cam_y = y;
        } else {
            let lim = s.min_y66.wrapping_sub(0x10);
            if (lim as i16) <= (s.cam_y as i16) {
                s.cam_y = lim;
                s.cam_x = s.w14;
            }
            let mut d = s.cam_x.wrapping_sub(s.x5a) as i16;
            let d1 = s.cam_y.wrapping_sub(s.y5c) as i16;
            if d < d1 {
                d = d1;
            }
            s.cam_x = s.x5a.wrapping_add(d as u16);
            let mut y = s.y5c.wrapping_add(d as u16);
            if (lim as i16) < (y as i16) {
                y = lim;
                s.cam_x = y.wrapping_sub(s.y5c).wrapping_add(s.x5a);
            }
            if (y as i16) >= (s.y4e as i16) {
                y = s.y4e;
                s.mode = 0;
            }
            s.cam_y = y;
        }
        self.stream2();
    }

    /// Mode 5 (0x210E0): stage 6 elevator ride.
    fn mode_elevator(&mut self) {
        // Velocities live in +$14/+$16 (as i16), integrated on the 16.16 camera by 0x189C.
        fn integrate(s: &mut Scroll2) {
            let vx = (s.w14 as i16).wrapping_add(s.a1a);
            s.w14 = vx as u16;
            let x = (((s.cam_x as u32) << 16) | s.cam_x_frac as u32).wrapping_add(((vx as i32) << 8) as u32);
            s.cam_x = (x >> 16) as u16;
            s.cam_x_frac = x as u16;
            let vy = (s.w16 as i16).wrapping_add(s.a1c);
            s.w16 = vy as u16;
            let y = (((s.cam_y as u32) << 16) | s.cam_y_frac as u32).wrapping_add(((vy as i32) << 8) as u32);
            s.cam_y = (y >> 16) as u16;
            s.cam_y_frac = y as u16;
        }
        if self.cam.s2.sub == 0 {
            self.sound(0x86);
            let s = &mut self.cam.s2;
            s.w14 = 0;
            s.a1a = 0;
            s.w16 = (-0x30i16) as u16;
            s.a1c = -6;
            s.w18 = 0;
            s.a1e = 0;
            s.sub = 2;
            self.cam_shake(2); // 0x21128: move.b #2,$755C(a5)
        }
        if self.cam.s2.sub == 2 {
            if (self.cam.s2.w16 as i16) > -0x600 {
                integrate(&mut self.cam.s2);
                self.stream2();
                return;
            }
            self.cam.s2.sub = 4;
        }
        if self.cam.s2.sub == 4 {
            let events = &self.assets.stages.stages[self.w.stage as usize].camera_events;
            let ev = events.get(self.cam.s2.next_event).copied().unwrap_or_default();
            let s = &mut self.cam.s2;
            s.cam_y = s.cam_y.wrapping_sub(6);
            if (s.cam_y as i16) <= (ev.y.wrapping_add(0x300) as i16) {
                s.a1c = s.a1c.wrapping_neg();
                s.sub = 6;
            }
            self.stream2();
            return;
        }
        if self.cam.s2.sub == 6 {
            let s = &mut self.cam.s2;
            if (s.w16 as i16) >= -0x30 {
                s.a1c = 0;
            }
            if s.cam_y <= 0xFFFC {
                integrate(s);
            } else {
                s.cam_y = 0;
                s.mode = 1;
                s.sub = 0;
                self.sound(0x87);
                return;
            }
        }
        self.stream2();
    }

    /// 0x22CAA: scroll3 controller.
    fn scroll3_task(&mut self) {
        if self.cam.s3.state == 0 {
            self.scroll3_init();
        } else {
            self.scroll3_run();
        }
    }

    /// 0x22CC0.
    fn scroll3_init(&mut self) {
        let area = self.cam.s2.area;
        let a = self.assets.stages.area(self.w.stage, area);
        let t = &mut self.cam.s3;
        t.state = 2;
        t.map_area = area;
        t.sx46 = a.scroll3.init_x;
        t.sy48 = a.scroll3.init_y;
        t.x5a = t.sx46;
        t.y5c = t.sy48;
        t.shift5f = a.scroll3_shift;
        if (t.b5 as i8) < 0x0E {
            t.cam_x = t.sx46;
            t.cam_y = t.sy48;
        } else {
            t.sx46 = t.cam_x;
            t.sy48 = t.cam_y;
        }
        let (mut x, y) = (self.cam.s3.sx46, self.cam.s3.sy48);
        let ncols = if x != 0 {
            x = x.wrapping_sub(0x20);
            0x13
        } else {
            0x11
        };
        for _ in 0..ncols {
            self.draw_column3(x, y.wrapping_add(0x140), 0xD);
            x = x.wrapping_add(0x20);
        }
    }

    /// 0x22D70.
    fn scroll3_run(&mut self) {
        let t = &mut self.cam.s3;
        if (t.cam_x as i16) < 0 {
            t.cam_x = 0;
        }
        if (t.cam_y as i16) < 0 {
            t.cam_y = 0;
        }
        let s2 = &self.cam.s2;
        match s2.mode {
            0x05 => {
                if (s2.cam_y as i16) <= 0x2000 {
                    let y = (s2.cam_y >> 4) as i16 - 0x60;
                    self.cam.s3.cam_y = if y < 0 { 0 } else { y as u16 };
                }
            }
            0x09 => {
                if (s2.sub as i8) <= 2 {
                    self.cam.s3.cam_y = 0x200u16.wrapping_sub(s2.y5c.wrapping_sub(s2.cam_y) >> 1);
                } else {
                    self.cam.s3.b5 = 0;
                }
            }
            0x0E | 0x0F => {}
            _ => {
                if s2.by_y79 == 0 {
                    let shift = self.cam.s3.shift5f;
                    self.cam.s3.cam_x = s2.cam_x >> shift;
                    let a = self.assets.stages.area(self.w.stage, s2.area);
                    if a.scroll3_yfollow {
                        let init_y = self.assets.stages.area(self.w.stage, s2.map_area).scroll2.init_y;
                        let d = s2.cam_y.wrapping_sub(init_y) as i16;
                        self.cam.s3.cam_y = ((d >> shift) as u16).wrapping_add(self.cam.s3.y5c);
                    }
                }
            }
        }
        self.stream3();
    }

    /// 0x222D6 screen shake (struct G.$755A).
    // TODO(port): 0x222D6. Exact logic is in re/specs/stage/HANDOFF.md ("Screen shake"); it offsets
    // only the displayed scroll2 Y (scroll1 Y in stage 6 area 1), never the camera.
    pub fn screen_shake(&mut self) {}

    /// Request a screen shake (the original writes the strength byte to G.$755C). Other ports
    /// call this where the original does `move.b #n,$755C(a5)`; a larger strength replaces a
    /// running shake, a smaller one is ignored (0x22316).
    pub fn cam_shake(&mut self, strength: u8) {
        self.cam.shake_req = strength;
    }

    /// Sets the camera's go flag (G.$7A9), used by stage flow code to release a lock.
    pub fn camera_go(&mut self) {
        self.cam.s2.go = 1;
    }
}

/// Tilemap pixel origin of each layer for a camera position, as the hardware sees it:
/// screen pixel (sx, sy) shows tilemap pixel ((ox + sx) & mask, (oy + sy) & mask).
pub fn layer_origin(cam_x: u16, cam_y: u16, base_y: u16) -> (u16, u16) {
    // Register X = camX - 0x40, visible area starts at hardware X 0x40 -> camX.
    // Register Y = base - camY, visible area starts at line 16.
    (cam_x, base_y.wrapping_sub(cam_y).wrapping_add(16))
}

