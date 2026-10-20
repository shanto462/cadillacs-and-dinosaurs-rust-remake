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
    /// +$10 camera Z (16.16, G.$74C; only the stage 2 chase integrates it, spawners read it).
    pub cam_z: u16,
    pub cam_z_frac: u16,
    /// +$18 velocity Z, +$1A..+$1E accelerations (mode 5).
    pub w18: u16,
    pub a1a: i16,
    pub a1c: i16,
    pub a1e: i16,
    /// Area whose map header is loaded (+$40..+$42 come from it).
    pub map_area: u8,
    /// Stage 2: the map is the controller's own header 0x22AFA (its init 0x22404) and not the table's (0x207CA, the demo's area setup):
    /// the screen array pointer +$42 is 0x22B04 instead of 0x2173E. Only the word 77 of the arrays differs (the top rows of the last column).
    pub own_map: bool,
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
    /// ROM address of the event list `next_event` indexes (0x2099A sets it). 0 = not set yet.
    pub event_base: u32,
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
    /// +$78 = G.$7B4, the lock state: 0 free, 1 locked (wave / area end), 2 stage end or boss fight. The one owner of the byte: the
    /// aggression table (0x182BA), the area music (0xA55A), the time over check (0x1042E), the hint (0x7BBF4) and the stage start
    /// (0xACAC) read and clear it here. The boss inits that write 2 (0x4FDCA Hogg, 0x6187A class 84, 0x687A0 class 91, 0x819BE the Slisaur
    /// starter, every frame) belong to the real class ports; the placeholders do not write it.
    pub lock78: u8,
    /// +$79 events trigger on Y (also freezes scroll3 parallax), +$7B mode after a mode-4 lock.
    pub by_y79: u8,
    pub next_mode7b: u8,
}

/// The row scroll task of stage 2 (struct at G.$7780, 0x2335A): +$04 state (0 init, 2 run), +$08 a word that is
/// cleared at the init and never written again (the formula subtracts it from the camera X).
#[derive(Clone, Debug, Default)]
pub struct RowScroll {
    pub state: u8,
    pub base8: u16,
}

impl Scroll2 {
    /// 0x189C with a6 = the controller: velocity += acceleration (the 8.8 words +$14 / +$16 / +$18 and +$1A / +$1C / +$1E),
    /// then the 16.16 position (X +$08, Y +$0C, Z +$10) moves by velocity * 256.
    fn integrate(&mut self) {
        fn axis(v: &mut u16, a: i16, hi: &mut u16, lo: &mut u16) {
            *v = (*v as i16).wrapping_add(a) as u16;
            let p = (((*hi as u32) << 16) | *lo as u32).wrapping_add(((*v as i16 as i32) << 8) as u32);
            *hi = (p >> 16) as u16;
            *lo = p as u16;
        }
        axis(&mut self.w14, self.a1a, &mut self.cam_x, &mut self.cam_x_frac);
        axis(&mut self.w16, self.a1c, &mut self.cam_y, &mut self.cam_y_frac);
        axis(&mut self.w18, self.a1e, &mut self.cam_z, &mut self.cam_z_frac);
    }
}

/// Scroll3 controller (struct at G.$7DC).
#[derive(Clone, Debug, Default)]
pub struct Scroll3 {
    pub state: u8,
    pub b5: u8,
    pub cam_x: u16,
    /// G.$7E6: the fraction of the scroll3 X (the long G.$7E4 is moved by the desert of the ending, `subi.l #$1000`).
    pub cam_x_frac: u16,
    pub cam_y: u16,
    pub map_area: u8,

    pub sx46: u16,
    pub sy48: u16,
    pub x5a: u16,
    pub y5c: u16,
    pub shift5f: u8,
}

/// The camera shadows the video hardware scrolls by: scroll1 G.$6C4 / $6C8, scroll2 G.$744 / $748,
/// scroll3 G.$7E4 / $7E8, as (X, Y) pairs.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScrollShadows {
    pub cam1: (u16, u16),
    pub s2: (u16, u16),
    pub s3: (u16, u16),
}

/// How many frames late the video hardware shows what the game computes: the scroll registers (CPS-A
/// 0x0C .. 0x16) of a recording's record k are the camera shadows of record k - 2, and a MAME picture
/// taken at the same moment shows the sprite list of that frame (the one the shadows of k - 2 positioned),
/// so the tiles and the sprites stay in step with each other. (The tile maps and the palettes are written
/// at once.) `Camera::shown_scroll` and `SpriteList::latch` give what the screen shows.
pub const VIDEO_LATENCY: usize = 2;

/// The last shadows, oldest first, for [`Camera::shown_scroll`].
#[derive(Clone, Debug, Default)]
pub struct ScrollLatch {
    hist: [ScrollShadows; VIDEO_LATENCY + 1],
    filled: bool,
}

#[derive(Clone, Debug)]
pub struct Camera {
    pub s2: Scroll2,
    pub s3: Scroll3,
    /// Stage 2 row scroll task (G.$7780).
    pub row: RowScroll,
    /// G.$51A: the handle of the car object (pool B class 88, made by the radio event, M6). Mode 8 of the stage 2
    /// controller reads its X, and its velocity and acceleration when the time is over (G.$518 = 2).
    pub car: super::Handle,
    /// Scroll1 camera (G.$6C4 / $6C8). In game it is (0, 0): 0xAE0 clears it at stage start and
    /// the HUD lives at scroll1 rows 34+ (0x90A014). 0xAC4E sets Y = 0x100 for a non-game screen.
    pub cam1_x: u16,
    pub cam1_y: u16,
    /// G.$6CA: the fraction of the scroll1 Y (G.$6C8 is the long Y.fraction when the staff roll of the ending moves it, 0xAE0 clears only the word).
    pub cam1_y_frac: u16,
    /// G.$755C: shake strength requested this frame (consumed by 0x222D6, see `cam_shake`).
    pub shake_req: u8,
    /// G.$755A, $755D..$7561: state, remaining strength, timer, sign and amplitude.
    pub shake_state: u8,
    pub shake_strength: u8,
    pub shake_timer: u8,
    pub shake_sign: i8,
    pub shake_amp: i16,
    /// Display-only offsets, included in the video latch, never in object coordinates.
    pub shake_dy1: i16,
    pub shake_dy2: i16,
    /// G.$868 (display_y2): the layer 2 scroll Y the hardware reads at the start of a pass (0x714). 0x71C copies the camera Y there at
    /// the start of each pass; the shake routine (0x22340..0x22368) writes camera Y + its offset on the frames it runs.
    pub display_y2: u16,
    /// G.$860 (display_y1): the scroll1 Y the hardware reads, copied from camera Y 1 at the start of a pass (0x708); the shake routine writes
    /// camera Y 1 + its offset on the scroll1 path (0x22356, when G.$4D9 = 6 and G.$786 = 1).
    pub display_y1: u16,
    /// G.$7B8: scroll1 Y shadow copied when 0x22388 stops the shake.
    pub shake_scroll1_y: u16,
    /// The shadows of the last frames (see [`VIDEO_LATENCY`]).
    pub latch: ScrollLatch,
}

impl Default for Camera {
    fn default() -> Self {
        Camera { s2: Scroll2::default(), s3: Scroll3::default(), row: RowScroll::default(), car: 0, cam1_x: 0, cam1_y: 0, cam1_y_frac: 0, shake_req: 0, shake_state: 0, shake_strength: 0, shake_timer: 0, shake_sign: 0, shake_amp: 0,
            shake_dy1: 0, shake_dy2: 0, display_y1: 0, display_y2: 0, shake_scroll1_y: 0, latch: ScrollLatch::default() }
    }
}

impl Scroll3 {
    /// `subi.l #v,$7E4(a5)`: the long X (integer and fraction) minus `v`.
    pub fn cam_x_long_sub(&mut self, v: u32) {
        let x = (((self.cam_x as u32) << 16) | self.cam_x_frac as u32).wrapping_sub(v);
        (self.cam_x, self.cam_x_frac) = ((x >> 16) as u16, x as u16);
    }
}

impl Camera {
    /// 0x222D6 / 0x222EE / 0x22316: advance the shake and write this frame's Y offset.
    fn step_shake(&mut self, scroll1: bool) {
        self.shake_dy1 = 0;
        self.shake_dy2 = 0;
        if self.shake_state == 0 {
            self.shake_strength = self.shake_req;
            if self.shake_strength == 0 {
                self.shake_scroll1_y = self.cam1_y;
                return;
            }
            self.shake_state = 2;
            self.shake_req = 0;
            self.shake_timer = 0;
            self.shake_amp = self.shake_strength as i16;
            self.shake_sign = -1;
        }
        if self.shake_req != 0 && (self.shake_req as i8) > (self.shake_strength as i8) {
            self.shake_strength = self.shake_req;
            self.shake_amp = self.shake_req as i16;
            self.shake_sign = -1;
        }
        self.shake_req = 0;
        let off = if self.shake_sign < 0 { self.shake_amp.wrapping_neg() } else { self.shake_amp };
        if scroll1 {
            self.shake_dy1 = off;
            // 0x22356: move.w d0,$860(a5) with d0 = the offset + G.$6C8 (camera Y 1 of this frame)
            self.display_y1 = self.cam1_y.wrapping_add(off as u16);
        } else {
            self.shake_dy2 = off;
            // 0x22364: move.w d0,$868(a5) with d0 = the offset + G.$748 (the camera Y of this frame)
            self.display_y2 = self.s2.cam_y.wrapping_add(off as u16);
        }
        let (timer, borrow) = self.shake_timer.overflowing_sub(1);
        self.shake_timer = timer;
        if borrow {
            self.shake_timer = 2;
            self.shake_sign = self.shake_sign.wrapping_neg();
            if self.shake_sign < 0 {
                self.shake_amp = self.shake_amp.wrapping_sub(1);
                self.shake_strength = self.shake_strength.wrapping_sub(1);
                if self.shake_strength == 0 {
                    // 0x22388 does not clear this frame's final positive offset.
                    self.shake_scroll1_y = self.cam1_y.wrapping_add(self.shake_dy1 as u16);
                    self.shake_state = 0;
                }
            }
        }
    }

    /// The shadows now.
    fn scroll_now(&self) -> ScrollShadows {
        ScrollShadows { cam1: (self.cam1_x, self.display_y1), s2: (self.s2.cam_x, self.display_y2), s3: (self.s3.cam_x, self.s3.cam_y) }
    }

    /// The end of a frame: remember the shadows. The first call fills the whole history, so a game that
    /// starts (or loads a state) in a scene shows its scroll at once.
    pub fn latch_scroll(&mut self) {
        let now = self.scroll_now();
        let l = &mut self.latch;
        if !l.filled {
            l.hist = [now; VIDEO_LATENCY + 1];
            l.filled = true;
        } else {
            l.hist.rotate_left(1);
            l.hist[VIDEO_LATENCY] = now;
        }
    }

    /// The scroll the hardware shows after the frames latched so far (the live shadows before the first).
    pub fn shown_scroll(&self) -> ScrollShadows {
        if self.latch.filled { self.latch.hist[0] } else { self.scroll_now() }
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
        let st = &self.assets.stages.stages[self.w.stage as usize];
        match &st.controller_map {
            Some(own) if self.cam.s2.own_map => own,
            _ => &self.assets.stages.area(self.w.stage, self.cam.s2.map_area).scroll2,
        }
    }

    fn s3_map(&self) -> &'static AreaMap {
        &self.assets.stages.area(self.w.stage, self.cam.s3.map_area).scroll3
    }

    pub(crate) fn screen_id2(&self, x: u16, y: u16) -> u16 {
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

    /// One tile of scroll3. An area without a far layer (`w == 0`) is written too (M1-15): the screen id of an empty map is 0
    /// and the cell comes from page 0 of the table, as in the original, which draws these cells hidden behind scroll2 (the
    /// recording of Episode 1 area 1, 187 cells of 0x1C00 / 0x0010; a map the original reads garbage ids from may give other
    /// pages there, always hidden).
    fn write_cell3(&mut self, i: usize, p: usize) {
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

    /// 0x2042E: background controllers, called at the end of the object update. Stage 2 (Episode 3) runs the row scroll
    /// task 0x2335A and its own scroll2 controller 0x22398 (mode table 0x2250A, events 0x22B9E) before scroll3.
    pub fn background_update(&mut self) {
        if self.w.stage == 2 {
            self.rowscroll_task(); // 0x2335A
            self.scroll2_task_stage2(); // 0x22398
        } else {
            self.scroll2_task(); // 0x206D4
        }
        self.scroll3_task(); // 0x22CAA
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

    /// 0x2228A: the demo's start of the scroll2 controller (the demo setup 0x99F10 calls it after
    /// 0xAC6A). The event index `first_event` comes from the table 0x222B6 by stage and area; the
    /// event is loaded (0x2099A) and the pointer to the next event is left on the loaded event (the
    /// normal start 0x206EA adds 0x10 to it, this one does not); the task state is set to 2, so the
    /// init 0x206EA (palettes, timers +$71 / +$75 / +$72 / +$73, the area of mode 0) is skipped; then
    /// the area is set up (0x20766). Added for M1-12 (`flow/demo.rs`).
    pub fn flow_demo_camera_init(&mut self, first_event: u8) {
        self.cam.s2.event_idx = first_event; // move.b (a4), $6c(a6)
        self.load_event_index(); // bsr 0x2099A
        self.cam.s2.state = 2; // move.b #2, 4(a6)
        self.area_setup(); // bra 0x20766
    }

    /// 0x2099A: load the event at the current event index.
    fn load_event_index(&mut self) {
        self.cam.s2.event_base = self.assets.stages.stages.get(self.w.stage as usize).map(|x| x.camera_events_addr).unwrap_or(0);
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
            self.spawn.g.cage_smashed = 0; // 0x20786 clr.b $7B6(a5)
            self.w.layer_ctrl = 0x06D6;
            // The cage: two pages of the scroll1 table, the left half (X 0) and the right half (X 0x100).
            self.lift_background(0, 0x100, 1); // 0x20790 .. 0x2079C
            self.lift_background(0x100, 0x100, 2); // 0x2079C .. 0x207AE
        }
        if self.w.stage == 6 && self.cam.s2.area == 2 {
            // The library takes the cage of the lift off scroll1 (0x207BE: jsr 0xB52, the clear) and draws the HUD again (jsr 0x162AE).
            self.flow_clear_scroll1();
            self.hook_162ae();
        }
        self.reinit_map();
    }

    /// 0x205C6 (with 0x20614, 0x20650, 0x206A8): draws one page of the scroll1 table 0x138C48 (page `page`, the d5 of the
    /// original) as the cage of the lift: 8 block columns of 4 cell columns. `x0` and `y0` are the world X and Y of the first block
    /// (d0, d1: 0 / 0x100 for the left page and 0x100 / 0x100 for the right one). The Y of a block is the world Y of its top edge and
    /// the address counts the rows from the top (0x1E0 - y), so a step of -0x20 is 4 rows down. The page of the table is 8 columns
    /// of 8 words (column by column, top block first). Every block column draws up to 9 blocks of 4 x 4 cells (blocks 0x15CFE8):
    /// the first column of a page starts with the bottom block of the page (y = 0x100 wraps to the word 7) at the scroll1 rows 28 to
    /// 31, then the eight words of the column from the top at the rows 32 to 63 (the second half of the 64 row map, +0x2000).
    /// a0 and a1 are looked up once, before the first column: the first block of every later column is drawn where the end of
    /// the column before left a0 (one cell column to the right of the column before, row 32) with the word the column before left
    /// in a1 (the word 0 of this column), and then, after the lookup of 0x20602, again at its own place. So the cells 4 k + 1 to
    /// 4 k + 3 of the rows 32 to 35 show the left three cell columns of the top block of the next block column: the MAME
    /// cells of `m328_l_e7a0_k16` have it.
    pub(crate) fn lift_background(&mut self, x0: u16, y0: u16, page: usize) {
        // 0x20650: the VRAM address of the block at (d0, d1).
        let addr = |d0: u16, d1: u16| -> u32 {
            let d3 = (d0 & 0x1E0) << 4;
            let d2 = 0x1E0u16.wrapping_sub(d1 & 0x1E0);
            let d4 = ((d2 & 0x100) << 5).wrapping_add((d2 & 0xE0) >> 1);
            0x908000 + d3.wrapping_add(d4) as u32
        };
        // 0x206A8: the word of the page table (a1) of the block at (d0, d1): the column (d0 & 0xE0) / 2 bytes, the word 0xE - (d1 & 0xE0) / 16 bytes.
        let word = |d0: u16, d1: u16| -> usize {
            let bytes = (page as u16) * 128 + ((d0 & 0xE0) >> 1) + 0xE - ((d1 & 0xE0) >> 4);
            (bytes / 2) as usize
        };
        let (mut d0, mut d1) = (x0, y0);
        // a0 and a1 are looked up once (0x205C6); the later columns carry them on from the end of the column before.
        let (mut a0, mut a1) = (addr(d0, d1), word(d0, d1));
        for _ in 0..8 {
            let mut d4 = 8i32;
            loop {
                // 0x20614: one block, 4 columns of 4 cells (the cells go down by +4, the next column is +0x80)
                let id = self.assets.frontend.scroll1_pages[a1 / 64][a1 % 64] as usize;
                let block = self.assets.frontend.scroll1_blocks[id];
                for (i, c) in block.iter().enumerate() {
                    self.hud_put(a0 + (i as u32 / 4) * 0x80 + (i as u32 % 4) * 4, c.code, c.attr);
                }
                a1 += 1; // lea 2(a1), a1
                a0 += 0x10; // lea -0x180(a0), a0: four rows down
                let d6 = d1;
                if d1 == 0 {
                    break; // 0x205F4
                }
                d1 -= 0x20;
                if d6 & 0xE0 == 0 {
                    // 0x20602: the block was at a multiple of 0x100: the address and the table word are looked up again
                    a0 = addr(d0, d1);
                    a1 = word(d0, d1);
                }
                d4 -= 1; // dbra d4
                if d4 < 0 {
                    break;
                }
            }
            d0 = d0.wrapping_add(0x20); // 0x205D6
            d1 = 0x100; // 0x205DA
        }
    }

    /// 0x207CA: load the area's scroll2 header, reset the camera and draw the first screen.
    fn reinit_map(&mut self) {
        let area = self.cam.s2.area;
        let map = &self.assets.stages.area(self.w.stage, area).scroll2;
        let s = &mut self.cam.s2;
        s.map_area = area;
        s.own_map = false;
        s.sx46 = map.init_x;
        s.sy48 = map.init_y;
        s.cam_x = map.init_x;
        s.cam_y = map.init_y;
        self.vram.clear2();
        self.vram.clear3();
        let (mut x, y) = (self.cam.s2.sx46, self.cam.s2.sy48);
        // When the area starts at X = 0 the original also pre-draws one screen at a fixed VRAM
        // offset (+0xBC8); its purpose is unknown.
        let ncols = if x != 0 {
            x = x.wrapping_sub(0x20);
            19
        } else {
            self.predraw_screen2(y);
            17
        };
        for _ in 0..ncols {
            self.draw_column2(x, y.wrapping_add(0x140), 0xD);
            x = x.wrapping_add(0x20);
        }
    }

    /// 0x20856..0x2086E and 0x20520: the area starts at X = 0, so the first screen of the area (id of the screen at
    /// (0, `y`)) is drawn once more at 0xBC8 bytes (754 cells) after the cell of (0, `y`): 8 times the first 8 block words
    /// of the screen (0x20572 loads the screen base again for each of the 8 outer steps, so the first block column
    /// is repeated), 2x2 cells each, 2 cells further down for each block and 32 cells further on for each outer step. The
    /// writes are not wrapped: cells past the end of scroll2 land in scroll3 (0x910000).
    fn predraw_screen2(&mut self, y: u16) {
        let id = self.screen_id2(0, y) as usize;
        let start = vram2(0, y) + 0xBC8 / 4;
        for outer in 0..8 {
            for row in 0..8 {
                let st = &self.assets.stages.stages[self.w.stage as usize];
                let blk = st.scroll2_screen_words.get(id * 64 + row).copied().unwrap_or(0);
                let cells = st.scroll2_blocks.get(((blk << 4) as i16 / 16) as usize).copied().unwrap_or([Cell::default(); 4]);
                let i = start + 32 * outer + 2 * row;
                for (k, d) in [0usize, 1, 16, 17].into_iter().enumerate() {
                    let at = i + d;
                    if at < 4096 {
                        self.vram.scroll2[at] = cells[k];
                    } else if at - 4096 < 4096 {
                        self.vram.scroll3[at - 4096] = cells[k];
                    }
                }
            }
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
                self.follow_x(); // 0x20F78: bsr 0x20A22
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
                self.sound_ex(0x87, 0x10, 0); // d1 = 0x10 (0x20C2C / 0x210AC / 0x2119C)
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
                    self.sound_ex(0x87, 0x10, 0); // d1 = 0x10 (0x20C2C / 0x210AC / 0x2119C)
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
        if self.w.stage == 2 {
            self.stage2_reinit(); // 0x22404 (modes 1 and 6 of the stage 2 controller)
        } else {
            self.area_setup();
        }
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
    /// Hard cut: sub 4 loads the palettes at brightness 0, sub 6 (the next game frame) sets full brightness (0x20D9E
    /// jmp 0x892C). MAME shows 2 to 4 black video frames: the area load of sub 4 (map 0x20766, palettes 0x8A44) runs past
    /// the next vertical blank, so those are lag frames. TODO(M10): model that lag (`flow.lag`) from the CPU time.
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

    /// Mode 8 (0x20E78; stage 2: mode 4, 0x229AE): switch to a sub-area without reloading the map.
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
            if self.w.stage != 2 {
                self.hook_9236c(); // the stage 2 copy of this mode (4, 0x229AE) does not call it
            }
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
        // Velocities live in +$14/+$16 (as i16), integrated on the 16.16 camera by 0x189C (`Scroll2::integrate`).
        if self.cam.s2.sub == 0 {
            self.sound_ex(0x86, 0x10, 0); // d1 = 0x10 (0x21106)
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
                self.cam.s2.integrate();
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
                s.integrate();
            } else {
                s.cam_y = 0;
                s.mode = 1;
                s.sub = 0;
                self.sound_ex(0x87, 0x10, 0); // d1 = 0x10 (0x20C2C / 0x210AC / 0x2119C)
                return;
            }
        }
        self.stream2();
    }

    // ------------------------------------------------------------------------------------------------------------
    // Stage 2 (Episode 3): the row scroll task and the scroll2 controller with its own mode table (M3-08, M3-09, M3-11)
    // ------------------------------------------------------------------------------------------------------------

    /// 0x2335A: the row scroll task (struct G.$7780). State 0 (0x23370) clears the rowscroll RAM 0x915800 .. 0x918000 and the
    /// base word +$08 and falls into state 2 (0x23390) in the same frame. Every frame the 32 words at 0x915920 .. 0x91595E
    /// (screen lines 128 to 159, `other[0x90..0xAF]`) get `(camX - base) / 2` steps of a perspective ramp: the word k (1 =
    /// the last line) is `((d0 - k * d1) + low word add of -d0) >> 5`, taken mod 256 from 0x180 on (the road screens repeat
    /// every 256 pixels). Written in the order of the original (from the last line up); the word arithmetic is the
    /// 68000's: `add.w` changes only the low word, `asr.l` shifts the whole long.
    fn rowscroll_task(&mut self) {
        if self.cam.row.state == 0 {
            self.cam.row.state += 2; // addq.b #2, 4(a6)
            self.vram.rowscroll.fill(0); // 0x23374: 0x915800 .. 0x918000
            self.cam.row.base8 = 0; // move.w #0, 8(a6)
        }
        let start = self.cam.s2.cam_x.wrapping_sub(self.cam.row.base8) as i16 as i32; // moveq #0,d0; move.w $744; sub.w 8(a6); ext.l
        let d3 = (start as u16).wrapping_neg(); // neg.w d3
        let d1 = start >> 1; // asr.l #1, d1
        let mut d0 = start;
        for k in 1..=32usize {
            d0 = d0.wrapping_sub(d1); // sub.l d1, d0
            let d2 = (d0 as u32 & 0xFFFF_0000) | (d0 as u16).wrapping_add(d3) as u32; // add.w d3, d2
            let mut v = ((d2 as i32) >> 5) as u16; // asr.l #5, d2
            if v >= 0x180 {
                v &= 0xFF; // cmpi.w #$180 / bcs / andi.w #$ff
            }
            self.vram.rowscroll[0xB0 - k] = v; // move.w d2, -(a0) from 0x915960
        }
    }

    /// 0x22398: the stage 2 scroll2 controller (state 0 init 0x223AE, state 2 run 0x224A4).
    fn scroll2_task_stage2(&mut self) {
        if self.cam.s2.state == 0 {
            self.scroll2_init_stage2();
        } else {
            self.scroll2_run_stage2();
        }
    }

    /// 0x223AE: the init. No tile animation timers and no area setup (0x20766): the palettes, the rowscroll bit of the
    /// video control, the event of the start, then the map (0x22404).
    fn scroll2_init_stage2(&mut self) {
        self.cam.s2.state += 2; // addq.b #2, 4(a6)
        self.load_all_palettes(); // 0x8A44
        self.brightness_full_current(); // 0x8932
        self.w.video_ctrl |= 1; // ori.w #1, $26(a5): the row scroll is on
        let s = &mut self.cam.s2;
        if (s.mode as i8) < 0x0E {
            s.area = 0;
        }
        s.go = 0;
        s.lock78 = 0;
        s.event_idx = 0;
        if s.mode != 0x0E && s.mode != 0x0F {
            self.load_event_index(); // 0x2251C
            self.cam.s2.next_event += 1; // addi.l #$10, $56(a6)
        }
        self.stage2_reinit();
    }

    /// 0x22404: the map of stage 2: the header at 0x22AFA (the same 60 screens as the table of 0x21580), the camera at the
    /// header's start, both layers cleared and 32 columns of 14 blocks drawn from x - 0x20 (x = 0 starts at 0) at y + 0x140.
    /// No pre-draw at +0xBC8 (that is 0x20520 of the standard controller).
    fn stage2_reinit(&mut self) {
        let st = &self.assets.stages.stages[self.w.stage as usize];
        let map = st.controller_map.as_ref().unwrap_or(&self.assets.stages.area(self.w.stage, self.cam.s2.area).scroll2);
        let s = &mut self.cam.s2;
        s.map_area = s.area;
        s.own_map = st.controller_map.is_some();
        s.sx46 = map.init_x;
        s.sy48 = map.init_y;
        s.cam_x = map.init_x;
        s.cam_y = map.init_y;
        self.vram.clear2();
        self.vram.clear3();
        let (mut x, y) = (self.cam.s2.sx46, self.cam.s2.sy48);
        if x != 0 {
            x = x.wrapping_sub(0x20);
        }
        for _ in 0..32 {
            self.draw_column2(x, y.wrapping_add(0x140), 0xD);
            x = x.wrapping_add(0x20);
        }
    }

    /// 0x224A4: the run state. The car state G.$518 forces mode 8. The event check (X only, unsigned) is skipped in the modes
    /// 1, 3, 6 and 8. Then the player extents (0x22600, the same as 0x20A8C) and the mode table 0x2250A:
    /// 0 follow, 1 area end with a fade, 2 lock, 3 stage end, 4 sub area, 5 auto scroll, 6 area change without a fade,
    /// 7 stage end with a fade, 8 the car drive. The event table (0x22B9E) only has the modes 0 and 2; the car sets 8 and 3.
    fn scroll2_run_stage2(&mut self) {
        if self.spawn.g.car_mode != 0 {
            self.cam.s2.mode = 8;
        }
        if ![1, 3, 6, 8].contains(&self.cam.s2.mode) {
            let events = &self.assets.stages.stages[self.w.stage as usize].camera_events;
            let ev = events.get(self.cam.s2.next_event).copied().unwrap_or_default();
            if ev.x <= self.cam.s2.cam_x {
                self.load_event(); // 0x22532
                self.cam.s2.next_event += 1;
            }
        }
        self.player_extents();
        match self.cam.s2.mode {
            0 => {
                self.follow_x();
                self.follow_y();
                self.stream2();
            }
            1 => self.mode_area_end(),
            2 => self.mode_lock_stage2(),
            3 => self.mode_stage_end_stage2(),
            4 => self.mode_sub_area(),
            5 => {
                self.cam.s2.cam_x = self.cam.s2.cam_x.wrapping_add(3); // addq.w #3, 8(a6)
                self.stream2();
            }
            6 => self.mode_area_change(),
            7 => self.mode_stage_end_fade(),
            8 => self.mode_car_chase(),
            _ => {}
        }
    }

    /// Mode 2 (0x22A04): the lock of a stage 2 event (a fight on foot). Different from the standard lock (mode 4,
    /// 0x21086): the lock state G.$7B4 is set only in a vertical scene (a quirk, never true in stage 2), so the camera stays
    /// "free" for the other code while it holds X. No stream (the camera does not move).
    fn mode_lock_stage2(&mut self) {
        let s = &mut self.cam.s2;
        if s.sub == 0 {
            s.sub += 2;
            s.go = 0;
            if self.w.vertical_scene != 0 {
                self.sound_ex(0x87, 0x10, 0); // d1 = 0x10 (0x20C2C / 0x210AC / 0x2119C)
                self.cam.s2.lock78 = 1;
            }
        }
        let s = &mut self.cam.s2;
        if s.by_y79 == 0 {
            s.cam_x = s.x4c;
        } else {
            s.cam_y = s.y4e;
        }
        if s.go != 0 {
            s.mode = s.next_mode7b;
            s.sub = 0;
            s.lock78 = 0;
        }
        s.go = 0;
    }

    /// Mode 3 (0x2290C): the end of the stage: the lock state is 2 and the camera holds the X it had when the mode began (the
    /// drive ended there). Sub 0 stores the camera X as the event X; sub 2 holds it and, when the go flag is set, raises the
    /// stage clear flag G.$4D4. The go flag is not cleared here. No stream.
    fn mode_stage_end_stage2(&mut self) {
        let s = &mut self.cam.s2;
        if s.sub == 0 {
            s.sub += 2;
            s.lock78 = 2;
            s.go = 0;
            s.x4c = s.cam_x;
        }
        if s.sub == 2 {
            s.cam_x = s.x4c;
            if s.go != 0 {
                self.w.stage_clear = 1;
            }
        }
    }

    /// The X of the car object (word +$08), the velocity (+$14) and the acceleration (+$1A) of the handle G.$51A. A handle that
    /// is no object reads zeros (the original would read the ROM at the same offsets; the car section always has the car).
    fn car_fields(&self) -> (u16, u16, u16) {
        let h = self.cam.car;
        if super::slot_of(h).is_none() {
            return (0, 0, 0);
        }
        let b = self.body(h);
        (b.xi() as u16, b.vx as u16, b.ax as u16)
    }

    /// Mode 8 (0x22A68): the Cadillac drive. Sub 0 steps to 2 and goes on at once. Sub 2: with G.$518 = 1 the camera chases the
    /// car: camX += clamp(car X - (camX + 0xC0), 8, 20) (11 px per frame at the right stick of the drive: the car's own
    /// speed). With G.$518 = 2 (time over) it takes the car's velocity and acceleration (sub 4) and integrates them on its
    /// 16.16 position (0x189C, X, Y and Z); with G.$518 = 0 (the car is gone) the drive ends: both zeroed and mode 3 at the
    /// current X. The stream of the map (0x2126E) follows in every case.
    fn mode_car_chase(&mut self) {
        match self.cam.s2.sub {
            0 => {
                self.cam.s2.sub += 2; // addq.b #2, 6(a6), then falls into sub 2
                self.car_chase_sub2();
            }
            2 => self.car_chase_sub2(),
            4 => self.car_chase_sub4(),
            _ => {}
        }
        self.stream2();
    }

    /// 0x22A86.
    fn car_chase_sub2(&mut self) {
        match self.spawn.g.car_mode {
            2 => {
                let (_, vx, ax) = self.car_fields();
                let s = &mut self.cam.s2;
                s.sub += 2; // 0x22AC4
                s.w14 = vx;
                s.a1a = ax as i16;
                self.car_chase_sub4();
            }
            0 => self.car_chase_end(),
            _ => {
                let (car_x, _, _) = self.car_fields();
                let s = &mut self.cam.s2;
                let d = car_x.wrapping_sub(s.cam_x.wrapping_add(0xC0)) as i16; // sub.w d3, d0
                // 0x22A98..0x22ABE: a signed word; negative takes 8, above 0x14 takes 0x14, below 8 takes 8 (`bmi`, `ble`, `bge`)
                let step: i16 = if d < 0 {
                    8
                } else if d > 0x14 {
                    0x14
                } else if d < 8 {
                    8
                } else {
                    d
                };
                s.cam_x = s.cam_x.wrapping_add(step as u16); // add.w d0, 8(a6)
            }
        }
    }

    /// 0x22AD8: integrate the camera on the velocities of the car; the end of the drive when the car is gone.
    fn car_chase_sub4(&mut self) {
        self.cam.s2.integrate(); // jsr 0x189C
        if self.spawn.g.car_mode == 0 {
            self.car_chase_end();
        }
    }

    /// 0x22AE6: the drive is over: velocity X and acceleration X cleared, mode 3 (stage end) at this X.
    fn car_chase_end(&mut self) {
        let s = &mut self.cam.s2;
        s.w14 = 0;
        s.a1a = 0;
        s.mode = 3;
        s.sub = 0;
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
    /// Only the vblank Y shadow moves. Sprites, collision and camera streaming keep their coordinates.
    pub fn screen_shake(&mut self) {
        self.cam.step_shake(self.w.stage == 6 && self.cam.s2.area == 1);
    }

    /// G.$755C writes at the request sites (for example 0x21128). The last write wins before
    /// 0x22316 compares it, signed, with the running strength. A restart keeps the timer phase.
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


#[cfg(test)]
mod latency_tests {
    use super::*;

    /// The scroll the screen shows is the shadow of two frames ago (the first frames show the first one).
    #[test]
    fn the_scroll_registers_show_the_shadows_two_frames_late() {
        let mut c = Camera::default();
        assert_eq!(c.shown_scroll().s2, (0, 0), "before the first frame the live shadows");
        let mut shown = Vec::new();
        for x in 1..=6u16 {
            c.s2.cam_x = x;
            c.s3.cam_y = 10 * x;
            c.latch_scroll();
            shown.push((c.shown_scroll().s2.0, c.shown_scroll().s3.1));
        }
        assert_eq!(shown, [(1, 10), (1, 10), (1, 10), (2, 20), (3, 30), (4, 40)]);
    }
}

#[cfg(test)]
mod shake_tests {
    use super::*;

    #[test]
    fn shake_duration_decay_and_layer_selection() {
        for strength in 1..=4u8 {
            for scroll1 in [false, true] {
                let mut c = Camera::default();
                c.cam1_y = 0xFFFF;
                c.shake_req = strength;
                let n = 6 * strength as usize - 2;
                for frame in 0..n {
                    c.step_shake(scroll1);
                    let phase = (frame + 2) / 3;
                    let amplitude = strength as i16 - (phase / 2) as i16;
                    let expected = if phase % 2 == 0 { -amplitude } else { amplitude };
                    assert_eq!((c.shake_dy1, c.shake_dy2), if scroll1 { (expected, 0) } else { (0, expected) });
                    assert_eq!((c.cam1_y, c.s2.cam_y), (0xFFFF, 0));
                    if frame + 1 < n { assert_eq!(c.shake_state, 2); }
                }
                assert_eq!((c.shake_state, c.shake_strength), (0, 0));
                assert_eq!(c.shake_scroll1_y, if scroll1 { 0 } else { 0xFFFF });
                c.step_shake(scroll1);
                assert_eq!((c.shake_dy1, c.shake_dy2), (0, 0));
            }
        }
    }

    #[test]
    fn stronger_requests_keep_timer_phase_and_use_signed_byte_compare() {
        let mut c = Camera::default();
        c.shake_req = 2;
        c.step_shake(false);
        assert_eq!(c.shake_timer, 2);
        c.shake_req = 4;
        c.step_shake(false);
        assert_eq!((c.shake_dy2, c.shake_timer, c.shake_sign), (-4, 1, -1));
        c.shake_req = 0xFF; // cmp.b / ble treats this as negative
        c.step_shake(false);
        assert_eq!((c.shake_dy2, c.shake_timer, c.shake_req), (-4, 0, 0));
        c.shake_req = 1;
        c.step_shake(false);
        assert_eq!((c.shake_dy2, c.shake_timer, c.shake_sign), (-4, 2, 1));
        c.latch_scroll();
        c.step_shake(false);
        c.latch_scroll();
        assert_eq!(c.shown_scroll().s2.1, (-4i16) as u16);
        assert_eq!(c.s2.cam_y, 0);
    }

    fn rom() -> Option<&'static dino_assets::AssetPack> {
        static A: std::sync::OnceLock<Option<&'static dino_assets::AssetPack>> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let path = std::path::Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    #[test]
    fn mode_0x12_hard_cut_black_frames() {
        let Some(assets) = rom() else { return };
        let mut g = Game::new(assets);
        g.w.stage = 0;
        g.cam.s2.area = 1;
        g.cam.s2.mode = 0x12;
        g.cam.s2.sub = 2;
        g.cam.s2.go = 1;

        g.cam.s2.state = 2;
        let events = &assets.stages.stages[0].camera_events;
        let (ev_idx, _) = events.iter().enumerate().find(|(_, ev)| ev.mode == 0x12).expect("mode 0x12 event");
        g.cam.s2.next_event = ev_idx;
        g.cam.s2.event_idx = ev_idx as u8;

        // Ensure full brightness initially
        g.brightness_full();
        assert_eq!(g.pal.words[0] & 0xF000, 0xF000);

        // Frame 0: sub 2 sees go != 0 -> sub becomes 4
        g.background_update();
        assert_eq!(g.cam.s2.sub, 4);

        // Frame 1: sub 4 runs area_end_next_area(false): palettes loaded at brightness 0, sub 6
        g.background_update();
        assert_eq!(g.cam.s2.sub, 6);
        assert_eq!(g.pal.words[0] & 0xF000, 0);

        // Frame 2: sub 6 (the go flag is clear): the next event, full brightness (0x20D9E)
        g.background_update();
        assert_eq!(g.cam.s2.sub, 0);
        assert_eq!(g.pal.words[0] & 0xF000, 0xF000);
    }
}

#[cfg(test)]
mod stage2_tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game() -> Game {
        Game::new(Box::leak(Box::new(AssetPack::default())))
    }

    /// A car object (pool B slot) at X `x`, handle in G.$51A and the car state G.$518.
    fn car_at(g: &mut Game, x: i16, mode: u8) -> usize {
        let i = g.props.alloc().unwrap();
        g.props.slots[i].b.live = 1;
        g.props.slots[i].b.set_xi(x);
        g.cam.car = g.props.handle(i);
        g.spawn.g.car_mode = mode;
        i
    }

    /// 0x2335A on the three positions of D4c A13 (MAME peeks of 0x915920 .. 0x91595E): the word of line 159 - j (j = 0 .. 31) is
    /// `-(j + 1) * (camX / 64) mod 256` (the values are above 0x180, so they are taken mod 256) and the first call also clears
    /// the RAM and sets the state.
    #[test]
    fn the_row_scroll_words_of_the_three_camera_positions() {
        for cam_x in [0x300u16, 0x1000, 0x2400] {
            let mut g = game();
            g.vram.rowscroll[0x200] = 0x1234; // the init clears all of 0x915800 .. 0x918000
            g.cam.s2.cam_x = cam_x;
            g.rowscroll_task();
            assert_eq!((g.cam.row.state, g.cam.row.base8, g.vram.rowscroll[0x200]), (2, 0, 0));
            for j in 0..32usize {
                let want = ((j as u16 + 1).wrapping_mul(cam_x / 64)).wrapping_neg() & 0xFF;
                assert_eq!(g.vram.rowscroll[0xAF - j], want, "camX {cam_x:#x}, screen line {}", 159 - j);
            }
            assert_eq!(g.vram.rowscroll[0xB0], 0, "the line after the ramp is not written");
            assert_eq!(g.vram.rowscroll[0x8F], 0, "the line before the ramp is not written");
        }
        // D4c 7.3: at camera X 0x300 the words run from 0x80 (line 128) up by 12 a line to 0xF4 (line 159)
        let mut g = game();
        g.cam.s2.cam_x = 0x300;
        g.rowscroll_task();
        assert_eq!((g.vram.rowscroll[0x90], g.vram.rowscroll[0x91], g.vram.rowscroll[0xAF]), (0x80, 0x8C, 0xF4));
    }

    /// The task keeps its state: the second call does not clear the table again (the words above the ramp stay).
    #[test]
    fn the_row_scroll_init_runs_once() {
        let mut g = game();
        g.rowscroll_task();
        g.vram.rowscroll[0x300] = 7;
        g.cam.s2.cam_x = 0x80;
        g.rowscroll_task();
        assert_eq!(g.vram.rowscroll[0x300], 7);
    }

    /// 0x22A86, G.$518 = 1 (BS-C2: the car drives at 2816 / 256 = 11.0 pixels a frame): the camera takes
    /// clamp(car X - (camX + 0xC0), 8, 20), so once the car is 11 pixels ahead of the chase line the camera moves 11 a frame.
    #[test]
    fn the_chase_camera_follows_a_car_at_eleven_pixels_a_frame() {
        let mut g = game();
        let car = car_at(&mut g, 0x3C0 + 11, 1);
        g.cam.s2.cam_x = 0x300;
        let mut cars = 0x3C0 + 11;
        for frame in 1..=60u16 {
            g.car_chase_sub2();
            assert_eq!(g.cam.s2.cam_x, 0x300 + 11 * frame, "frame {frame}");
            cars += 11;
            g.props.slots[car].b.set_xi(cars);
        }
    }

    /// The clamp of the step: behind or on the line (even a negative distance) 8, ahead 8 to 20 as it is, further ahead 20.
    #[test]
    fn the_chase_step_is_clamped_to_8_to_20() {
        for (gap, step) in [(-0x100i32, 8u16), (-1, 8), (0, 8), (7, 8), (8, 8), (13, 13), (20, 20), (21, 20), (0x300, 20)] {
            let mut g = game();
            g.cam.s2.cam_x = 0x1000;
            car_at(&mut g, (0x1000 + 0xC0 + gap) as i16, 1);
            g.car_chase_sub2();
            assert_eq!(g.cam.s2.cam_x, 0x1000 + step, "gap {gap}");
        }
        // sub.w works on words: a car 0x80B0 behind wraps to a positive word (0x7F50), so the step is the maximum
        let mut g = game();
        g.cam.s2.cam_x = 0x8000;
        car_at(&mut g, 0x10, 1);
        g.car_chase_sub2();
        assert_eq!(g.cam.s2.cam_x, 0x8014);
    }

    /// Sub 0 steps to sub 2 in the same call; G.$518 = 0 ends the drive: velocity and acceleration X cleared, mode 3 (stage
    /// end), sub 0, and the position is not touched (0x22AE6).
    #[test]
    fn the_end_of_the_drive_starts_the_stage_end_mode() {
        let mut g = game();
        g.cam.s2.mode = 8;
        g.cam.s2.cam_x = 0x453A;
        g.cam.s2.w14 = 0x0B00;
        g.cam.s2.a1a = 5;
        g.spawn.g.car_mode = 0;
        g.car_chase_sub2();
        assert_eq!((g.cam.s2.mode, g.cam.s2.sub, g.cam.s2.w14, g.cam.s2.a1a, g.cam.s2.cam_x), (3, 0, 0, 0, 0x453A));
    }

    /// G.$518 = 2 (time over): sub 2 takes the car's velocity (+$14) and acceleration (+$1A), goes to sub 4 and integrates
    /// (0x189C): velocity += acceleration, position += velocity * 256 on the 16.16 camera (X, and Y and Z with their own
    /// words). The next frames stay in sub 4 and keep integrating; the car state 0 ends it after the step.
    #[test]
    fn the_time_over_drive_integrates_the_car_velocity() {
        let mut g = game();
        let car = car_at(&mut g, 0x400, 2);
        g.props.slots[car].b.vx = 0x0B00;
        g.props.slots[car].b.ax = -0x40;
        g.cam.s2.mode = 8;
        g.cam.s2.sub = 2; // sub 0 has stepped to 2 in the frame the mode starts
        g.cam.s2.cam_x = 0x500;
        g.car_chase_sub2();
        assert_eq!((g.cam.s2.sub, g.cam.s2.w14, g.cam.s2.a1a), (4, 0x0B00 - 0x40, -0x40));
        // (0xAC0 << 8) = 0xAC000 on the 16.16 position: 10.75 pixels
        assert_eq!((g.cam.s2.cam_x, g.cam.s2.cam_x_frac), (0x500 + 0x0A, 0xC000));
        g.car_chase_sub4();
        // second step: velocity 0xA80, 0xA8000 on the 16.16 position: 0x50A.C000 + 0xA.8000 = 0x515.4000
        assert_eq!((g.cam.s2.w14, g.cam.s2.cam_x, g.cam.s2.cam_x_frac), (0x0B00 - 0x80, 0x515, 0x4000));
        g.spawn.g.car_mode = 0;
        g.car_chase_sub4();
        assert_eq!((g.cam.s2.mode, g.cam.s2.w14, g.cam.s2.a1a), (3, 0, 0));
    }

    /// Mode 2 (0x22A04): the fight lock holds the event X and ends when the go flag is set; the lock state G.$7B4 stays 0
    /// (it is set only in a vertical scene, a quirk of this copy of the lock); the go flag is cleared in every frame.
    #[test]
    fn the_stage_2_lock_holds_x_and_ends_on_go() {
        let mut g = game();
        g.cam.s2.mode = 2;
        g.cam.s2.x4c = 0x300;
        g.cam.s2.cam_x = 0x305;
        g.cam.s2.go = 1; // sub 0 clears a go from before
        g.mode_lock_stage2();
        assert_eq!((g.cam.s2.sub, g.cam.s2.cam_x, g.cam.s2.lock78, g.cam.s2.go, g.cam.s2.mode), (2, 0x300, 0, 0, 2));
        g.cam.s2.cam_x = 0x303;
        g.mode_lock_stage2();
        assert_eq!((g.cam.s2.cam_x, g.cam.s2.mode), (0x300, 2));
        g.cam.s2.go = 1;
        g.cam.s2.next_mode7b = 0;
        g.mode_lock_stage2();
        assert_eq!((g.cam.s2.mode, g.cam.s2.sub, g.cam.s2.go), (0, 0, 0));
    }

    /// Mode 3 (0x2290C): the stage end holds the X it had, lock state 2, and the go flag raises the stage clear flag G.$4D4.
    #[test]
    fn the_stage_2_stage_end_holds_the_current_x() {
        let mut g = game();
        g.cam.s2.mode = 3;
        g.cam.s2.cam_x = 0x453A;
        g.cam.s2.x4c = 0x2D00;
        g.mode_stage_end_stage2();
        assert_eq!((g.cam.s2.lock78, g.cam.s2.x4c, g.cam.s2.sub, g.w.stage_clear), (2, 0x453A, 2, 0));
        g.cam.s2.cam_x = 0x4540;
        g.mode_stage_end_stage2();
        assert_eq!(g.cam.s2.cam_x, 0x453A);
        g.cam.s2.go = 1;
        g.mode_stage_end_stage2();
        assert_eq!(g.w.stage_clear, 1);
    }
}

#[cfg(test)]
mod lift_tests {
    use super::*;
    use crate::game::vram::Vram;
    use dino_assets::AssetPack;

    /// A pack with pages 0 to 2 (the page `p` lists the ids 64 p + 1 .. 64 p + 64, column by column, top block first) and blocks
    /// whose cells carry their id and their place: code = 100 id + n, attribute = n (n = 0 .. 15, column by column).
    fn game() -> Game {
        let mut pack = AssetPack::default();
        for p in 0..3u16 {
            let mut page = [0u16; 64];
            for (k, w) in page.iter_mut().enumerate() {
                *w = 64 * p + k as u16 + 1;
            }
            pack.frontend.scroll1_pages.push(page);
        }
        for id in 0..=192u16 {
            let mut b = [Cell::default(); 16];
            for (n, c) in b.iter_mut().enumerate() {
                *c = Cell { code: 100 * id + n as u16, attr: n as u16 };
            }
            pack.frontend.scroll1_blocks.push(b);
        }
        Game::new(Box::leak(Box::new(pack)))
    }

    fn block_at(g: &Game, col: usize, row: usize) -> u16 {
        let c = g.vram.scroll1[Vram::index1(col, row)];
        c.code / 100
    }

    /// The cage (0x205C6, called twice by the area setup 0x20766 for the pages 1 and 2): per block column 9 blocks from the bottom
    /// block of the page at the rows 28 to 31 (first column of a page only), then the eight blocks of the column from the top at the
    /// rows 32 to 63. a0 and a1 carry on from the end of the column before, so the first block of a later column is first drawn
    /// one cell column to the right of the end of the column before (cells 4 k + 1 .. 4 k + 4 of the rows 32 to 35) and again at its
    /// place: the cells 4 k + 1 .. 4 k + 3 of the top block of the column k show the left three columns of the top block of the
    /// column k + 1 (MAME: scroll1 of `m328_l_e7a0_k16`).
    #[test]
    fn the_cage_background_fills_the_rows_28_to_63_of_two_pages() {
        let mut g = game();
        g.lift_background(0, 0x100, 1);
        g.lift_background(0x100, 0x100, 2);
        // page 1 (ids 65 .. 128): column c of the page, word r of the column = id 65 + 8 c + r
        for c in 0..8usize {
            for r in 1..8usize {
                let id = 65 + 8 * c + r;
                for (dc, dr) in [(0, 0), (3, 3)] {
                    let cell = g.vram.scroll1[Vram::index1(4 * c + dc, 32 + 4 * r + dr)];
                    assert_eq!(cell.code, 100 * id as u16 + (4 * dc + dr) as u16, "column {c} block {r}");
                }
            }
            // the top block: its own first column, the next three columns come from the stray block of the next column
            let own = 65 + 8 * c;
            assert_eq!(g.vram.scroll1[Vram::index1(4 * c, 33)].code, 100 * own as u16 + 1);
            for j in 0..3usize {
                let (id, n) = if c < 7 { (own + 8, 4 * j + 2) } else { (own, 4 * (j + 1) + 2) };
                assert_eq!(g.vram.scroll1[Vram::index1(4 * c + 1 + j, 34)].code, 100 * id as u16 + n as u16, "column {c} stray {j}");
            }
        }
        // the extra block: the bottom block (word 7) of the first column of each page at the rows 28 to 31, the other columns stay empty
        assert_eq!(block_at(&g, 0, 28), 65 + 7);
        assert_eq!(block_at(&g, 3, 31), 65 + 7);
        assert_eq!(block_at(&g, 4, 28), 0, "the second column has no block above the row 32");
        assert_eq!(block_at(&g, 32, 28), 129 + 7, "the second page: its first column too");
        assert_eq!(block_at(&g, 36, 30), 0);
        // the second page is the columns 32 to 63
        assert_eq!(block_at(&g, 32, 35), 129);
        assert_eq!(block_at(&g, 63, 63), 129 + 63);
        // nothing above the row 28
        assert_eq!(block_at(&g, 0, 27), 0);
    }
}

#[cfg(test)]
mod mode_c_tests {
    use super::*;

    fn rom() -> Option<&'static dino_assets::AssetPack> {
        static A: std::sync::OnceLock<Option<&'static dino_assets::AssetPack>> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let path = std::path::Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// Mode 0xC (0x20F78, the slope of Episode 2 area 1) is the follow of mode 0 (`bsr 0x20A22`) plus a camera Y that falls by 1 while the
    /// heroes are right of X 0x750 (the port had left out the follow: the camera stood still and the bot of `--autoplay` stood at the
    /// edge of the screen for ever).
    #[test]
    fn mode_0c_follows_the_heroes_in_x() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        g.start_stage_at(1, 0, 0, &[0]);
        g.cam.s2.mode = 0x0C;
        g.cam.s2.cam_x = 0x600;
        g.cam.s2.cam_y = 0x20;
        g.cam.s2.event_idx = 0;
        // no event of the stage fires: the next one is the terminator (X 0xFFFF)
        g.cam.s2.next_event = g.assets.stages.stages[1].camera_events.iter().position(|e| e.x == 0xFFFF).expect("the list ends with the terminator");
        g.players[0].b.set_state(0x0200_0000); // in play (the extents skip a hero in mode 0)
        g.players[0].b.set_xi(0x780);
        g.scroll2_run();
        assert_eq!(g.cam.s2.cam_x, 0x606, "the camera follows the hero at 6 pixels a frame");
        assert_eq!(g.cam.s2.cam_y, 0x1F, "and the camera Y falls by 1 with the heroes right of 0x750");
        g.players[0].b.set_xi(0x700);
        g.scroll2_run();
        assert_eq!((g.cam.s2.cam_x, g.cam.s2.cam_y), (0x60C, 0x1F), "the Y stays left of 0x750");
    }

    /// 0x207B6: the area setup of Episode 7 area 2 (the library) clears scroll1 (the cage of the lift is on it) and draws the HUD again; area 1
    /// draws the cage and the other areas of the stage leave scroll1 alone. (The autoplay showed the cage over the computer room.)
    #[test]
    fn the_library_takes_the_cage_off_scroll1() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        g.start_stage_at(6, 0, 0, &[0]);
        g.w.stage = 6;
        g.cam.s2.area = 1;
        g.area_setup();
        let cage = g.vram.scroll1.iter().filter(|c| c.code != 0x20).count();
        assert!(cage > 400, "the lift draws its cage ({cage} cells)");
        g.cam.s2.area = 2;
        g.area_setup();
        let left = g.vram.scroll1.iter().filter(|c| c.code != 0x20 && c.code != 0).count();
        assert!(left < 100, "the library has cleared scroll1 ({left} cells are not blank, the HUD is drawn again)");
    }
}
