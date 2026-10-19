//! Object drawing, ported from the sprite list builder (0x13FA6) and the per-object draw
//! routines: 0x141F8 (draw list entry, grab pair order), 0x14136 (split body), 0x14230/0x14234
//! (frame), 0x14180 (compound frames), 0x142AA/0x142BE (normal compositions), 0x14494 (alt
//! compositions), 0x1439C/0x1463E (wading clip), 0x14BC6 (water splash), 0x147FE (raw sprite
//! lists), 0x14EC0 (dynamic shadow pass). Formats: `re/specs/anim/SPEC.md` 2 and 3.
//!
//! The original writes hardware sprite entries (x, y, code, attr) where the first entry has the
//! highest priority. We keep exactly that list, then hand it to the GPU renderer back to front.
//!
//! Note: drawing is not read-only in the original: the wading clip sets bit 7 of obj+$53, which
//! the shadow pass (and the player code) read later. `draw_object` therefore takes the body
//! mutably, like the 68000 code.

use super::body::Body;
use super::camera::VIDEO_LATENCY;
use super::enemy::Ram;
use super::{Game, Handle, Slot, collide, slot_of};
use crate::render::{DrawTarget, Layer};
use dino_assets::effect::{EffectData, RawSprite};
use dino_assets::{Composition, Frame, FrameDraw, FrameId, SpriteData};
use macroquad::prelude::{Color, GREEN, ORANGE, RED};

#[derive(Clone, Copy, Debug)]
pub struct ObjEntry {
    pub x: i16,
    pub y: i16,
    pub code: u16,
    pub attr: u16,
    /// Not in the original entry: white hit flash amount for the renderer.
    pub flash: u8,
    /// Not in the original entry: handle of the object that drew it (harness: which entries come
    /// from classes the port does not run). 0 = none.
    pub src: Handle,
}

/// Tag of a frame id (+$28) that names a pool D slot: the object's draw record and its raw sprite list are the bytes of
/// that slot (the stage clock, 0x81096..0x810A0), the low byte is the slot index. The original's +$28 is the slot's RAM
/// address; `trace.rs` converts. It is neither an effect frame nor a raw frame of the ROM.
pub const SLOT_FRAME: dino_assets::FrameId = 0x2000_0000;

/// The frame id of pool D slot `i`.
pub fn slot_frame(i: usize) -> dino_assets::FrameId {
    SLOT_FRAME | i as dino_assets::FrameId
}

pub fn is_slot_frame(id: dino_assets::FrameId) -> bool {
    id & 0xE000_0000 == SLOT_FRAME
}

/// Camera base used by the draw code: $69B6 = camera X - 0x40, $69B8 = camera Y.
#[derive(Clone, Copy, Debug, Default)]
pub struct DrawCamera {
    pub x: i16,
    pub y: i16,
}

impl DrawCamera {
    pub fn new(camera_x: i16, camera_y: i16) -> DrawCamera {
        DrawCamera { x: camera_x.wrapping_sub(0x40), y: camera_y }
    }
}

/// Debug hit box outline (screen pixels), drawn on top of everything by `submit`.
#[derive(Clone, Copy, Debug)]
pub struct DebugBox {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// 0 = body (+$48), 1 = attack (+$49, frame type 4), 2 = probe (+$49, other frame types).
    pub kind: u8,
}

#[derive(Clone)]
pub struct SpriteList {
    pub entries: Vec<ObjEntry>,
    /// Debug hit box overlay (the original's "BODY HIT / ATACK HIT" viewer draws box corners
    /// from 0x149B0 when the debug DIPs are set). Toggle from the input code (F1).
    pub debug_boxes: bool,
    pub debug_rects: Vec<DebugBox>,
    /// -$7126(a5): frames until the splash phase advances (reset to 1 by 0x49A8).
    pub splash_timer: u16,
    /// -$7124(a5): splash animation phase 0, 4, 8, 12.
    pub splash_phase: u16,
    /// Handle of the object being drawn (stored into `ObjEntry::src`).
    pub cur_src: Handle,
    /// The raw list of the pool D slot of the object being drawn (`Game::draw_plain` fills it for a slot frame).
    slot_sprites: Vec<RawSprite>,
    /// What the screen shows of the lists: see `latch`. `None` until the first frame ends.
    shown: Option<Shown>,
    /// 0xAE0 zeroed the buffers and no list has been built since (the story panels of mode 8 build none): the trace shows 256 zero entries and no end marker.
    pub blank: bool,
}

/// The lists built in the last frames, newest last, and what the screen shows now.
#[derive(Clone, Default)]
struct Shown {
    /// (frame, entries, debug boxes) of the last build of each frame that built one.
    builds: Vec<(u32, Vec<ObjEntry>, Vec<DebugBox>)>,
    /// A build started in this frame.
    built: bool,
    /// 0xAE0 zeroed the sprite buffers in this frame.
    wiped: bool,
    frame: u32,
    now: (Vec<ObjEntry>, Vec<DebugBox>),
}

impl Default for SpriteList {
    fn default() -> SpriteList {
        // 0x49A8 draw list reset: -$7126 = 1.
        SpriteList {
            entries: Vec::new(),
            debug_boxes: false,
            debug_rects: Vec::new(),
            splash_timer: 1,
            splash_phase: 0,
            cur_src: 0,
            slot_sprites: Vec::new(),
            shown: None,
            blank: false,
        }
    }
}

/// Where a frame id lives: player / enemy frames in `SpriteData`, effect frames (tagged ids)
/// and raw-list frames in `EffectData`.
#[derive(Clone, Copy)]
pub struct Frames<'a> {
    pub s: &'a SpriteData,
    pub fx: Option<&'a EffectData>,
}

impl<'a> Frames<'a> {
    pub fn frame(&self, id: FrameId) -> &'a Frame {
        if EffectData::is_effect_frame(id) {
            self.fx.expect("effect frame without effect data").frame(id)
        } else {
            self.s.frame(id)
        }
    }
}

impl SpriteList {
    /// Start of a frame's list. Also does the splash phase step of 0x13FE6, which the original
    /// runs at the start of every build_sprite_list.
    pub fn clear(&mut self) {
        self.blank = false;
        self.entries.clear();
        self.debug_rects.clear();
        if let Some(s) = &mut self.shown {
            s.built = true;
        }
        // 0x13FE6: subq.w #1,-$7126 ; bne ; -$7126 = 10 ; -$7124 = (-$7124 + 4) & 0xC
        self.splash_timer = self.splash_timer.wrapping_sub(1);
        if self.splash_timer == 0 {
            self.splash_timer = 10;
            self.splash_phase = self.splash_phase.wrapping_add(4) & 0xC;
        }
    }

    /// Draws one body's current frame (0x14230) without effect data (used by the viewer).
    pub fn draw_body(&mut self, s: &SpriteData, b: &Body, cam: DrawCamera, flash: u8) {
        let mut b = b.clone();
        let fr = Frames { s, fx: None };
        let frame = b.frame;
        self.draw_frame(fr, &mut b, frame, cam, flash);
    }

    /// 0x14230 / 0x14234: draw frame `frame` of body `b` (a3 = frame record).
    pub fn draw_frame(&mut self, fr: Frames, b: &mut Body, frame: FrameId, cam: DrawCamera, flash: u8) {
        // 0x14234: raw sprite list path. Quirk: it reads +$28 again, not a3 (matters only if a
        // split body also had bit 6 set, which no object does).
        if b.draw_flags & 0x40 != 0 {
            self.draw_raw(fr, b, cam, flash);
            return;
        }
        let (d4, d5) = base(b, cam);
        let f = fr.frame(frame);
        match &f.draw {
            FrameDraw::Comp { main, overlay } => {
                // Main composition first (on top) with the clip flag -$7128 = word 1, then the
                // overlay with -$7128 = 0xFFFF (so only the last one drawn can be clipped).
                let d5s = screen_y(d5);
                self.draw_comp(fr, fr.s.comp(*main), b, d4, d5s, f.w1, flash);
                if let Some(o) = overlay {
                    self.draw_comp(fr, fr.s.comp(*o), b, d4, d5s, 0xFFFF, flash);
                }
            }
            FrameDraw::Compound(parts) => {
                // 0x14180: each part moves X by +-dx and the height by dy (restored after each).
                for p in parts {
                    let x = if b.facing != 0 { d4.wrapping_sub(p.dx) } else { d4.wrapping_add(p.dx) };
                    let y = d5.wrapping_add(p.dy);
                    self.draw_comp(fr, fr.s.comp(p.comp), b, x, screen_y(y), p.clip_flag, flash);
                }
            }
        }
    }

    /// 0x142BE (normal format) / 0x14494 (alt format): place every entry of a composition.
    /// `clip_flag` is -$7128(a5): its high byte negative allows the wading clip.
    #[allow(clippy::too_many_arguments)]
    fn draw_comp(&mut self, fr: Frames, c: &Composition, b: &mut Body, d4: i16, d5: i16, clip_flag: u16, flash: u8) {
        let mut n = c.entries.len();
        // 0x142BE: tst.b $53 ; tst.b -$7128 (bpl) ; cmpi.w #$18,$C (blt => clip)
        if b.wading != 0 && clip_flag & 0x8000 != 0 && b.yi() < 0x18 {
            // 0x1439C
            b.wading |= 0x80;
            if b.draw_flags & 0x20 == 0 {
                self.draw_splash(fr, b, d4, d5);
            }
            let lvl = ((b.yi().max(0).wrapping_add(8) as u16 >> 4) & 1) as usize;
            // sub.b $4C(a0,d0.w),d6 with d6 = n-1; borrow => nothing is drawn.
            let skip = b.extra[lvl] as usize;
            if skip > (n - 1) & 0xFF {
                return;
            }
            n -= skip;
        }
        for e in &c.entries[..n] {
            let mut attr = e.attr;
            if b.palette_override & 0x80 != 0 {
                attr = (attr & 0xFFE0) | (b.palette_override as u16 & 0x1F);
            }
            let x = if b.facing == 0 {
                d4.wrapping_add(e.dx)
            } else {
                attr ^= 0x20;
                // Normal format always mirrors with 16 (its entries are 1x1); alt uses the width.
                let w = if c.alt { width(attr) } else { 16 };
                d4.wrapping_sub(e.dx).wrapping_sub(w)
            };
            self.push(x, d5.wrapping_add(e.dy), e.code, attr, flash);
        }
    }

    /// Culling of every draw path (0x14306..0x1432C): it only skips output.
    fn push(&mut self, x: i16, y: i16, code: u16, attr: u16, flash: u8) {
        if x > 0x1C0 || (x <= 0x40 && (width(attr) as u16) < (0x40i16.wrapping_sub(x)) as u16) {
            return;
        }
        self.entries.push(ObjEntry { x, y, code, attr, flash, src: self.cur_src });
    }

    /// 0x147FE: frames that are raw sprite lists (obj+$25 bit 6, 8-byte records).
    fn draw_raw(&mut self, fr: Frames, b: &Body, cam: DrawCamera, flash: u8) {
        let Some(fx) = fr.fx else { return };
        // the list: a ROM record's, or the one the object keeps in its pool D slot (`movea.l (a1),a1`, 0x147FE)
        let own;
        let list: &[RawSprite] = if is_slot_frame(b.frame) {
            own = std::mem::take(&mut self.slot_sprites);
            &own
        } else if EffectData::is_raw_frame(b.frame) {
            &fx.raw(b.frame).sprites
        } else {
            return;
        };
        let (d4, d5) = base(b, cam);
        let d5 = screen_y(d5);
        for e in list {
            let mut attr = e.attr;
            if b.palette_override & 0x80 != 0 {
                attr = (attr & 0xFFE0) | (b.palette_override as u16 & 0x1F);
            }
            let x = if b.facing == 0 {
                d4.wrapping_add(e.dx)
            } else {
                attr ^= 0x20;
                d4.wrapping_sub(e.dx).wrapping_sub(width(attr))
            };
            self.push(x, d5.wrapping_add(e.dy), e.code, attr, flash);
        }
    }

    /// 0x14BC6: water splash around the feet of a wading object, drawn before (so above) the
    /// clipped composition. No palette override. Y is moved back down by the height (+$0C).
    fn draw_splash(&mut self, fr: Frames, b: &Body, d4: i16, d5: i16) {
        let Some(fx) = fr.fx else { return };
        let t = b.extra[2];
        if t & 0x80 != 0 {
            return;
        }
        // (t & 0xF0) + -$7124 is a byte offset into the long table at 0x14C9C.
        let list: &[RawSprite] = &fx.splashes[(t >> 4) as usize][(self.splash_phase >> 2) as usize & 3];
        for e in list {
            let mut attr = e.attr;
            let x = if b.facing == 0 {
                d4.wrapping_add(e.dx)
            } else {
                attr ^= 0x20;
                d4.wrapping_sub(e.dx).wrapping_sub(width(attr))
            };
            let y = d5.wrapping_add(b.yi()).wrapping_add(e.dy);
            self.push(x, y, e.code, attr, 0);
        }
    }

    /// 0xAE0 (0xB10): both sprite buffers are zeroed at once. The screen goes blank in the next frame.
    pub fn wipe(&mut self) {
        self.blank = true;
        self.entries.clear();
        self.debug_rects.clear();
        if let Some(s) = &mut self.shown {
            s.wiped = true;
        }
    }

    /// The end of a frame. The screen shows the list of the last build that is `VIDEO_LATENCY` frames old or
    /// older (the original builds into one of two buffers and the video hardware copies the one its register
    /// points to, which the vertical blank code updates once a frame; a game that builds no list for a while,
    /// like the profile cards during their fades, keeps showing the last one). The first call starts the
    /// history, so a game that starts in a scene shows its sprites at once.
    pub fn latch(&mut self) {
        let (entries, rects) = (std::mem::take(&mut self.entries), std::mem::take(&mut self.debug_rects));
        let s = self.shown.get_or_insert_with(|| Shown { built: true, ..Default::default() });
        if s.built {
            if s.builds.last().is_some_and(|b| b.0 == s.frame) {
                s.builds.pop();
            }
            s.builds.push((s.frame, entries.clone(), rects.clone()));
        }
        self.entries = entries;
        self.debug_rects = rects;
        // the newest build that is old enough is shown; the builds before it are not needed again
        let due = s.frame.checked_sub(VIDEO_LATENCY as u32);
        let at = due.and_then(|d| s.builds.iter().rposition(|b| b.0 <= d));
        s.now = at.map_or_else(Default::default, |k| (s.builds[k].1.clone(), s.builds[k].2.clone()));
        if let Some(k) = at {
            s.builds.drain(..k);
        }
        if s.wiped {
            s.builds.clear();
        }
        s.built = false;
        s.wiped = false;
        s.frame = s.frame.wrapping_add(1);
    }

    /// Hands the list the screen shows to the GPU, last entry first, so the first entry ends on top.
    pub fn submit(&self, gpu: &mut impl DrawTarget) {
        let (entries, rects) = match &self.shown {
            Some(s) => (&s.now.0, &s.now.1),
            None => (&self.entries, &self.debug_rects),
        };
        for e in entries.iter().rev() {
            let sy = (e.y as i32) & 0x1FF;
            let sy = if sy >= 0x1F0 { sy - 0x200 } else { sy };
            gpu.sprite(Layer::Sprites, e.x as i32 - 0x40, sy - 16, e.code, e.attr, e.flash);
        }
        for r in rects {
            let c: Color = match r.kind {
                0 => GREEN,
                1 => RED,
                _ => ORANGE,
            };
            gpu.debug_rect(r.x as f32, r.y as f32, r.w as f32, r.h as f32, c);
        }
    }

    /// Debug overlay: body and attack / probe boxes of one object, in screen pixels.
    fn debug_body(&mut self, s: &SpriteData, b: &Body, cam: DrawCamera) {
        if b.box_table == 0 {
            return;
        }
        let (d4, d5) = base(b, cam);
        let feet = screen_y(d5) as i32 - 16;
        let kind_atk = if b.kind == 4 { 1 } else { 2 };
        for (index, kind) in [(b.body_box, 0u8), (b.attack_box, kind_atk)] {
            if index == 0 {
                continue;
            }
            for h in collide::boxes(s, b, index) {
                let left = if b.facing != 0 { h.xoff.wrapping_neg().wrapping_sub(h.xsize) } else { h.xoff };
                let x = d4 as i32 + left as i32 - 0x40;
                let y = feet - h.yoff as i32 - h.ysize as i32;
                self.debug_rects.push(DebugBox { x, y, w: h.xsize as i32, h: h.ysize as i32, kind });
            }
        }
    }
}

/// d4 / d5 of the draw code: X and height + depth, camera relative unless +$25 bit 7.
fn base(b: &Body, cam: DrawCamera) -> (i16, i16) {
    let mut d4 = b.xi();
    let mut d5 = b.yi().wrapping_add(b.zi());
    if b.draw_flags & 0x80 == 0 {
        d4 = d4.wrapping_sub(cam.x);
        d5 = d5.wrapping_sub(cam.y);
    }
    (d4, d5)
}

/// Block width in pixels from an attr word: ((attr >> 4) & 0xF0) + 16.
fn width(attr: u16) -> i16 {
    ((attr >> 4) & 0xF0) as i16 + 16
}

/// `d5 = ((d5 & 0xFF00) | (!d5 & 0x00FF)) + 1`, i.e. (0x100 - v) mod 0x200 for the usual range:
/// turns "height above the bottom" into a hardware sprite Y.
pub fn screen_y(d5: i16) -> i16 {
    (((d5 as u16 & 0xFF00) | (!d5 as u16 & 0x00FF)) as i16).wrapping_add(1)
}

impl Game {
    /// 0x4CCC pool D init (part of every area start, 0xAA8C): the free stack is rebuilt (slot 0 is pushed first, so the
    /// first request gets slot 15); the bytes of the slots stay as the last user left them.
    pub fn draw_special_init(&mut self) {
        self.specials.reset_free();
    }

    /// 0x48A8 pool D allocator: takes a slot and stores its handle in `+$98` of the caller (`caller` = the caller's bytes
    /// beyond the common header). `None` = no slot left (carry set, `+$98` cleared).
    pub fn draw_special_alloc(&mut self, caller: &mut Ram) -> Option<usize> {
        let slot = self.specials.alloc();
        caller.sw(0x98, slot.map_or(0, |i| self.specials.handle(i)));
        if let Some(i) = slot {
            self.specials.slots[i].b.live = 1; // the mirror of the byte +$00 (the trace tells a used block from a free one by it)
        }
        slot
    }

    /// 0x498C (the check on `+$98`) and 0x4994 pool D free: the slot goes back on the stack and `+$98` is cleared. The slot
    /// is not cleared: its bytes, `+$00` included, stay as they were.
    pub fn draw_special_free(&mut self, caller: &mut Ram) {
        let h = caller.w(0x98);
        if h == 0 {
            return;
        }
        if let Some(Slot::Special(i)) = slot_of(h) {
            self.specials.release(i);
        }
        caller.sw(0x98, 0);
    }

    fn draw_frames(&self) -> Frames<'static> {
        Frames { s: &self.assets.sprites, fx: Some(&self.assets.effects) }
    }

    /// 0x141F8: draw one object of the sorted draw list.
    pub fn draw_object(&mut self, h: Handle, cam: DrawCamera) {
        self.sprites.cur_src = h;
        let fr = self.draw_frames();
        let (sub, kind, p, flags) = {
            let b = self.body(h);
            (b.sub, b.kind, b.partner, b.draw_flags)
        };
        // Grab pair: holder in sub-state 4 with frame type 0xC, partner alive with type 0xE.
        if sub == 4 && kind == 0xC && slot_of(p).is_some() {
            let (plive, pkind, p62) = {
                let pb = self.body(p);
                (pb.live, pb.kind, pb.b62)
            };
            if plive != 0 && pkind == 0xE {
                // 0x14114: partner+$62 == 0 => self first (on top), else partner first.
                // Both go through 0x14230 directly (no split-body check).
                let order = if p62 == 0 { [h, p] } else { [p, h] };
                for o in order {
                    self.draw_plain(fr, o, cam);
                }
                return;
            }
        }
        if flags & 1 != 0 {
            self.draw_split(fr, h, cam);
        } else {
            self.draw_plain(fr, h, cam);
        }
    }

    /// 0x14230 for object `h` (its +$28 frame).
    fn draw_plain(&mut self, fr: Frames, h: Handle, cam: DrawCamera) {
        let mut b = std::mem::take(self.body_mut(h));
        let frame = b.frame;
        if b.draw_flags & 0x40 != 0 && is_slot_frame(frame) {
            let i = (frame & 0xFF) as usize;
            self.sprites.slot_sprites = self.specials.slots.get(i).map(|s| s.sprites(self.specials.handle(i))).unwrap_or_default();
        }
        self.sprites.draw_frame(fr, &mut b, frame, cam, 0);
        if self.sprites.debug_boxes {
            self.sprites.debug_body(fr.s, &b, cam);
        }
        *self.body_mut(h) = b;
    }

    /// 0x14136: split body. The upper part (+$28) is drawn first at (+-$3C, +$3E) with +$53
    /// cleared (no wading clip), then the lower part (+$30) at the object position.
    fn draw_split(&mut self, fr: Frames, h: Handle, cam: DrawCamera) {
        let mut b = std::mem::take(self.body_mut(h));
        let (w53, x, y) = (b.wading, b.x, b.y);
        b.wading = 0;
        let dx = if b.facing != 0 { b.dx3c.wrapping_neg() } else { b.dx3c };
        // add.w to the integer words only; the fractions stay (restored anyway).
        b.x = (b.x & 0xFFFF) | ((b.xi().wrapping_add(dx) as i32) << 16);
        b.y = (b.y & 0xFFFF) | ((b.yi().wrapping_add(b.dy3e) as i32) << 16);
        let upper = b.frame;
        self.sprites.draw_frame(fr, &mut b, upper, cam, 0);
        (b.wading, b.x, b.y) = (w53, x, y);
        // 0x14178: a3 = +$30, then 0x14234 (raw check on +$28, then the normal path).
        let lower = b.frame2;
        self.sprites.draw_frame(fr, &mut b, lower, cam, 0);
        if self.sprites.debug_boxes {
            self.sprites.debug_body(fr.s, &b, cam);
        }
        *self.body_mut(h) = b;
    }

    /// 0x14EC0: dynamic shadows for the draw list, after all objects (so below them), in the
    /// same order. Ground poses carry their shadow in the art, so a shadow is only added while
    /// the object is above its floor.
    pub fn shadow_pass(&mut self, list: &[u32], cam: DrawCamera) {
        let fx = &self.assets.effects;
        for &key in list {
            self.sprites.cur_src = key as u16;
            let b = self.body(key as u16).clone();
            if b.draw_flags & 0x10 != 0 {
                continue;
            }
            let mut d1 = b.floor.wrapping_add(b.floor2);
            if b.wading != 0 {
                if b.wading & 0x80 != 0 {
                    continue;
                }
                // cmpi.w #$18,d1 ; bcc (unsigned)
                if (d1 as u16) < 0x18 {
                    d1 = 0x18;
                }
            }
            if d1.wrapping_sub(b.yi()) == 0 {
                continue;
            }
            let mut y = screen_y(d1.wrapping_add(b.zi()).wrapping_sub(cam.y)).wrapping_sub(0x10);
            let d4 = b.xi().wrapping_sub(cam.x);
            let t = b.extra[2];
            if t & 0x80 != 0 {
                // Quirk: the original branches into the inner loop's dbra here (stale d6/a2).
                // No object reaches it in practice (they all set +$25 bit 4 or stay grounded).
                continue;
            }
            if t == 0x70 {
                y = y.wrapping_add(4);
            }
            for e in &fx.shadows[(t >> 4) as usize] {
                self.sprites.push(d4.wrapping_add(e.dx), y, e.code, e.attr, 0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_y_is_0x100_minus_v_mod_0x200() {
        for v in [0i16, 1, 0x10, 0x7F, 0x80, 0xFF, 0x100, 0x180, -1, -0x20] {
            assert_eq!((screen_y(v) as u16) & 0x1FF, 0x100u16.wrapping_sub(v as u16) & 0x1FF, "v={v}");
        }
    }

    fn entry(code: u16) -> ObjEntry {
        ObjEntry { x: 0, y: 0, code, attr: 0, flash: 0, src: 0 }
    }

    /// What the screen shows of the sprite lists: the list of the last build that is 2 frames old or older, which
    /// stays while no list is built; 0xAE0 blanks it from the next frame.
    #[test]
    fn the_screen_shows_the_sprite_list_two_frames_late() {
        let mut s = SpriteList::default();
        let shown = |s: &SpriteList| s.shown.as_ref().map(|x| x.now.0.iter().map(|e| e.code).collect::<Vec<_>>()).unwrap();
        let mut seen = Vec::new();
        for frame in 1..=8u16 {
            if frame <= 5 {
                s.clear(); // a build
                s.entries.push(entry(frame));
            }
            if frame == 7 {
                s.wipe();
            }
            s.latch();
            seen.push(shown(&s));
        }
        let want: Vec<Vec<u16>> = vec![vec![], vec![], vec![1], vec![2], vec![3], vec![4], vec![5], vec![]];
        assert_eq!(seen, want);
    }

    #[test]
    fn splash_phase_steps_every_10_frames() {
        let mut s = SpriteList::default();
        s.clear(); // timer 1 -> 0: first step at once (0x49A8 sets it to 1)
        assert_eq!(s.splash_phase, 4);
        for _ in 0..9 {
            s.clear();
        }
        assert_eq!(s.splash_phase, 4);
        s.clear();
        assert_eq!(s.splash_phase, 8);
    }

    #[test]
    fn cull_matches_0x14306() {
        let mut s = SpriteList::default();
        s.push(0x1C1, 0, 0, 0, 0); // right of 0x1C0: skipped
        s.push(0x30, 0, 0, 0, 0); // 16 >= 0x10: drawn
        s.push(0x2F, 0, 0, 0, 0); // 16 < 0x11: skipped
        s.push(0x2F, 0, 0, 0x0100, 0); // 32 wide: drawn
        assert_eq!(s.entries.len(), 2);
    }
}
