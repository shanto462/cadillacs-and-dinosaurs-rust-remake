//! Pool B: controllers and props (0x23604): the wave controller (class 15, 0x7B336) and the boss
//! reinforcement controller (class 22, 0x7BEF6). Owner: enemy port. See SPEC section 8.2.

use super::body::Body;
use super::enemy::Ram;
use super::spawn::clear_header;
use super::{Game, Handle};
use dino_assets::enemy::Wave;

// Pool B classes of the front end (M1-06): the profile card board (93) and its scroll (94).
#[path = "prop/class93.rs"]
mod class93;
#[path = "prop/class94.rs"]
mod class94;
// Pool B classes of the ranking screen (M1-07): the Cadillac (86) and the cloud manager (89).
#[path = "prop/class86.rs"]
mod class86;
#[path = "prop/class89.rs"]
mod class89;
// Pool B classes of the opening movie (M1-10, M1-11): the black bars (32).
#[path = "prop/class32.rs"]
mod class32;
#[path = "prop/class66.rs"]
mod class66;
#[path = "prop/class72.rs"]
mod class72;
// Pool B classes of the character select (M2-04): the frame (17), the pictures over the group picture (67) and the player tags (68).
#[path = "prop/class17.rs"]
mod class17;
#[path = "prop/class19.rs"]
mod class19;
#[path = "prop/class67.rs"]
mod class67;
#[path = "prop/class68.rs"]
mod class68;

#[derive(Clone, Debug, Default)]
pub struct Prop {
    pub b: Body,
    pub m: Ram,
}

/// Wave controller fields (pointer +$A0 kept as group / record indexes).
mod off {
    pub const TIMER: usize = 0x80; // word; the end phase uses its high byte
    pub const GROUP: usize = 0xA0; // group index (+$A0 pointer in the original)
    pub const REC: usize = 0xA2; // record index inside the group
    pub const SPAWNED: usize = 0xA6; // +$A6 records spawned
}
use off::*;

impl Game {
    /// 0x23604 pool B update: every live slot in slot order, dispatch on +$20.
    pub fn pool_b_update(&mut self) {
        for i in 0..self.props.slots.len() {
            if self.props.slots[i].b.live == 0 {
                continue;
            }
            let h = self.props.handle(i);
            let mut p = std::mem::take(&mut self.props.slots[i]);
            match p.b.k20 / 4 {
                15 => self.prop_wave(&mut p, h, i),
                17 => self.prop_class17(&mut p, i),
                19 => self.prop_class19(&mut p, i),
                32 => self.prop_class32(&mut p, i),
                66 => self.prop_class66(&mut p, i),
                67 => self.prop_class67(&mut p, i),
                68 => self.prop_class68(&mut p, i),
                72 => self.prop_class72(&mut p, i),
                73 => self.prop_bird_spawner(&mut p, i),
                86 => self.prop_class86(&mut p, i),
                89 => self.prop_class89(&mut p, i),
                93 => self.prop_class93(&mut p, i),
                94 => self.prop_class94(&mut p, i),
                _ => self.prop_unported(&mut p, h, i),
            }
            self.props.slots[i] = p;
        }
        self.prop_area_walk_fallback();
    }

    /// Classes not ported yet (props, scenery, event objects): freed at once.
    fn prop_unported(&mut self, p: &mut Prop, h: Handle, i: usize) {
        if self.trace_hold(h, &p.b) {
            return; // harness: kept as the snapshot had it
        }
        self.prop_free(p, i);
    }

    /// 0x4938 pool B free: clears +$00..+$27 and +$63.
    pub fn prop_free(&mut self, p: &mut Prop, i: usize) {
        clear_header(&mut p.b);
        p.b.b63 = 0;
        self.props.release(i);
    }

    /// 0x89ABA pool B class 73: the bird spawner of stage 1. Every 0x100..0x2FF frames it takes a
    /// pool A slot for a class 50 bird (`+$20 = 0xC8`) of variant 0, 2, 4, 6, 8, 4, 6 or 8 (table
    /// 0x89B32, one `rand()`), then draws the next delay (one `rand()`). A full pool still draws
    /// the delay. (Phase 1 task 4a: the birds need their spawner for the `rand()` order.)
    fn prop_bird_spawner(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x89ACE, continues at 0x89AE0 in the same frame.
                p.b.mode = 2;
                p.b.live = 2;
                p.m.sw(TIMER, 0x80);
                self.prop_bird_spawner_tick(p);
            }
            2 => self.prop_bird_spawner_tick(p),
            _ => self.prop_free(p, i),
        }
    }

    /// 0x89AE0.
    fn prop_bird_spawner_tick(&mut self, p: &mut Prop) {
        let t = p.m.w(TIMER).wrapping_sub(1);
        p.m.sw(TIMER, t);
        if t != 0 {
            return;
        }
        if let Some(h) = self.spawn_alloc(0) {
            let b = self.body_mut(h);
            b.live = 1;
            b.k20 = 0xC8;
            b.var26 = 0;
            b.dir = 0;
            let k = ((self.rand() & 7) * 2) as u32;
            let v = self.assets.enemy.t.u16(0x89B32, k);
            self.body_mut(h).var26 = v;
        }
        let d = (self.rand() & 0x1FF).wrapping_add(0x100);
        p.m.sw(TIMER, d);
    }

    fn prop_wave_script(&self, p: &Prop) -> &'static Wave {
        &self.assets.enemy.stages[self.w.stage as usize].waves[p.b.var26 as usize]
    }

    /// 0x7B336 wave controller (pool B class 15).
    fn prop_wave(&mut self, p: &mut Prop, h: Handle, i: usize) {
        let _ = h;
        match p.b.mode {
            0 => {
                // 0x7B34A
                p.m.sw(GROUP, 0);
                p.m.sw(REC, 0);
                self.spawn.g.wave_timer = 0x1C20;
                p.m.sw(SPAWNED, 0);
                p.m.sw(TIMER, 0);
                p.b.mode += 2;
            }
            2 => {
                if self.spawn.g.car_mode != 0 {
                    return self.prop_wave_end_go(p);
                }
                match p.b.sub {
                    0 => self.prop_wave_wait(p),
                    2 => self.prop_wave_spawn(p),
                    _ => self.prop_wave_end(p),
                }
            }
            _ => self.prop_free(p, i),
        }
    }

    /// 0x7B3B6: wait for the threshold (or the 7200-frame budget), then start the next group.
    fn prop_wave_wait(&mut self, p: &mut Prop) {
        let t = p.m.w(TIMER);
        if t != 0 {
            p.m.sw(TIMER, t - 1);
            return;
        }
        let w = self.prop_wave_script(p);
        let g = p.m.w(GROUP) as usize;
        let thr = if g < w.groups.len() { w.groups[g].thr } else { w.final_thr };
        let lim = thr[self.w.players_minus_one as usize % 3];
        let alive = self.spawn.g.alive as u16;
        let bt = self.spawn.g.wave_timer;
        self.spawn.g.wave_timer = bt.wrapping_sub(1);
        if bt != 0 && alive > lim {
            return;
        }
        if g >= w.groups.len() {
            // End word (negative): +$80 high byte = 0x1E, sub 4.
            p.m.sb(TIMER, 0x1E);
            p.b.sub = 4;
            if (self.spawn.g.wave_timer as i16) < 0 {
                self.spawn.g.kill_all = 2;
            }
            return;
        }
        p.m.sw(TIMER, w.groups[g].spawns[0].trig);
        p.m.sw(REC, 0);
        p.b.sub = 2;
    }

    /// 0x7B41E: spawn the group's records, each after its delay.
    fn prop_wave_spawn(&mut self, p: &mut Prop) {
        let w = self.prop_wave_script(p);
        let g = p.m.w(GROUP) as usize;
        loop {
            let t = p.m.w(TIMER);
            p.m.sw(TIMER, t.wrapping_sub(1));
            if t != 0 {
                return;
            }
            let s = p.m.w(REC) as usize;
            let r = w.groups[g].spawns[s];
            if self.spawn_record(&r, false) {
                p.m.sw(SPAWNED, p.m.w(SPAWNED).wrapping_add(1));
            }
            let s = s + 1;
            p.m.sw(REC, s as u16);
            if s >= w.groups[g].spawns.len() {
                // terminator: skip it, back to the threshold wait
                p.m.sw(GROUP, (g + 1) as u16);
                p.m.sw(REC, 0);
                p.m.sw(TIMER, 3);
                self.spawn.g.wave_timer = 0x1C20;
                p.b.mode = 2;
                p.b.sub = 0;
                return;
            }
            p.m.sw(TIMER, w.groups[g].spawns[s].trig);
            if (w.groups[g].spawns[s].trig as i16) < 0 {
                return;
            }
        }
    }

    /// 0x7B464: end of the wave once every present player is in normal control.
    fn prop_wave_end(&mut self, p: &mut Prop) {
        let t = p.m.b(TIMER);
        if t != 0 {
            p.m.sb(TIMER, t - 1);
            if t - 1 == 0x19 {
                self.spawn.g.kill_all = 0;
            }
            return;
        }
        for i in 0..3 {
            let b = &self.players[i].b;
            if b.live != 0 && !(b.mode == 2 && b.sub == 0) {
                return;
            }
        }
        let end = self.prop_wave_script(p).end;
        match end & 0xE {
            0 => self.prop_wave_end_go(p),
            2 => {
                self.prop_set_leader();
                self.prop_area_walk(2);
                p.b.mode = 6;
                p.b.sub = 0;
            }
            _ => {
                self.prop_set_leader();
                self.prop_area_walk(4);
                p.b.mode = 6;
                p.b.sub = 0;
            }
        }
    }

    /// 0x7B4DE: GO (camera go flag G.$7A9) and done.
    fn prop_wave_end_go(&mut self, p: &mut Prop) {
        self.camera_go();
        self.prop_set_leader();
        p.b.mode = 6;
        p.b.sub = 0;
    }

    /// 0x1E5F6: the right-most present player leads the walk-off (player +$C0).
    pub fn prop_set_leader(&mut self) {
        for i in 0..3 {
            self.players[i].bc0 = 0;
        }
        let mut best = 0usize;
        let mut x: i16 = 0;
        for i in 0..3 {
            let b = &self.players[i].b;
            if b.live != 0 && x <= b.xi() {
                x = b.xi();
                best = i;
            }
        }
        self.players[best].bc0 = 1;
    }

    /// 0x1D95E (kind 2: walk to the next area) / 0x1DAC4 (kind 4: stage exit) belong to the
    /// player / flow port (scripted walk). TEMP: remember it and give the camera go flag while
    /// the camera waits (see `prop_area_walk_fallback`).
    fn prop_area_walk(&mut self, kind: u8) {
        if kind == 4 {
            self.spawn.g.b4fc = 1;
        }
        self.spawn.g.area_walk = true;
    }

    /// TEMP until the scripted walk (0x1D95E) is ported: release wave / area-end locks.
    fn prop_area_walk_fallback(&mut self) {
        if self.spawn.g.area_walk && self.cam.s2.lock78 == 1 {
            self.camera_go();
        }
    }
}
