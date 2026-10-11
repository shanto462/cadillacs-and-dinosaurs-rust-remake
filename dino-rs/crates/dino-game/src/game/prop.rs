//! Pool B: controllers and props (0x23604): the wave controller (class 15, 0x7B336) and the boss
//! reinforcement controller (class 22, 0x7BEF6). Owner: enemy port. See SPEC section 8.2.

use super::body::Body;
use super::enemy::Ram;
use super::spawn::clear_header;
use super::{Game, Handle};
use dino_assets::enemy::Wave;

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
                _ => self.prop_unported(&mut p, i),
            }
            self.props.slots[i] = p;
        }
        self.prop_area_walk_fallback();
    }

    /// Classes not ported yet (props, scenery, event objects): freed at once.
    fn prop_unported(&mut self, p: &mut Prop, i: usize) {
        self.prop_free(p, i);
    }

    /// 0x4938 pool B free: clears +$00..+$27 and +$63.
    pub fn prop_free(&mut self, p: &mut Prop, i: usize) {
        clear_header(&mut p.b);
        p.b.b63 = 0;
        self.props.release(i);
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
