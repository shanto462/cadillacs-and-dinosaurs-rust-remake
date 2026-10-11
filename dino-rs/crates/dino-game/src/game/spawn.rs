//! Stage spawn interpreter (0xFBF4) and the enemy-side globals. Owner: enemy port.
//! See `re/specs/enemy/SPEC.md` section 8.
//!
//! Interface for the flow port (area / stage changes):
//! * `area_reset_objects()` = the pool part of 0xAA0E plus 0xAA92..0xAAD8 (call from `hook_acd0`).
//! * `rank_area_start()` (0x18048), `time_area_start()` (0x1031A) and `spawn_reset()` (0xFBE8)
//!   are the other calls of 0xAADC..0xAAEE.
//!
//! Until the flow port calls `spawn_reset`, `spawn_script_tick` notices an area change by itself.

use super::{Game, Handle, POOL_A, POOL_B, Pool};
use dino_assets::enemy::SpawnRec;

/// Enemy-side globals of the original (G.$xxxx), kept here because this port owns them.
#[derive(Default)]
pub struct Globals {
    /// G.$4F3 alive enemies counted by the previous census, G.$4FB the running count.
    pub alive: u8,
    pub alive_run: u8,
    /// G.$50E kill-all: 1 = credit `killer`, 2 = no credit (wave timeout).
    pub kill_all: u8,
    /// G.$4E0 object credited by a kill-all.
    pub killer: Handle,
    /// G.$50C wave group time budget (7200 frames).
    pub wave_timer: u16,
    /// G.$518 special mode (1 = Cadillac drive section). [I]
    pub car_mode: u8,
    /// G.$502 boss threshold list enable (set by the boss summon).
    pub b502: u8,
    /// G.$4D5 stage clear in progress (set by 0xACEE). [I]
    pub b4d5: u8,
    /// G.$4F1, G.$4FC, G.$504, G.$505: cleared at area start (0xAAC2..). G.$4FC is cleared by
    /// the Vice T. intro; the others are item / event counters. [U]
    pub b4f1: u8,
    pub b4fc: u8,
    pub b504: u8,
    pub b505: u8,
    /// G.$4F9 / G.$4F8: dizzy enemies last frame / this frame.
    pub dizzy_prev: u8,
    pub dizzy: u8,
    /// Census counters per player (player +$E0..+$E3 last frame, +$E4..+$E7 this frame):
    /// [targeting me, of those attacking, enemies left of me, enemies right of me].
    pub census_prev: [[u8; 4]; 3],
    pub census: [[u8; 4]; 3],
    /// TEMP: a wave ended with "walk to the next area" (0x1D95E, player/flow port). Until the
    /// automatic walk is ported, the camera's go flag is given while the camera waits.
    pub area_walk: bool,
    /// Player kill counts (player +$10B; not in the player port's struct yet).
    pub kills: [u8; 3],
    /// G.$4DE area transition kind (set by 0x1D95E, player / flow port).
    pub b4de: u8,
    /// G.$7B2 a burning enemy exists (2).
    pub b7b2: u8,
}

/// State of the interpreter (struct at G.$66C).
#[derive(Default)]
pub struct Spawn {
    /// +$04 init done.
    pub init: u8,
    /// +$08 / +$0C current record of list 1 / list 2 (index in the area lists).
    pub l1: usize,
    pub l2: usize,
    /// Area whose lists are loaded (to notice an area change, see the module doc).
    pub area: u8,
    pub g: Globals,
}

/// Clears what the pool free routines clear: +$00..+$27.
pub fn clear_header(b: &mut super::body::Body) {
    b.live = 0;
    b.visible = 0;
    b.who = 0;
    b.id = 0;
    b.set_state(0);
    b.x = 0;
    b.y = 0;
    b.z = 0;
    b.vx = 0;
    b.vy = 0;
    b.vz = 0;
    b.ax = 0;
    b.ay = 0;
    b.az = 0;
    b.k20 = 0;
    b.dir = 0;
    b.palette_override = 0;
    b.facing = 0;
    b.draw_flags = 0;
    b.var26 = 0;
}

impl Game {
    /// 0xFBE8: restart the spawn lists (area start).
    pub fn spawn_reset(&mut self) {
        self.spawn.l1 = 0;
        self.spawn.l2 = 0;
        self.spawn.init = 0;
    }

    /// Pool part of 0xAA0E (pool A / B init 0x4C48 / 0x4C76) and 0xAA92..0xAAD8: pool E objects
    /// except held weapons are told to free themselves, area globals cleared.
    pub fn spawn_area_reset_objects(&mut self) {
        self.enemies = Pool::new(POOL_A);
        self.props = Pool::new(POOL_B);
        for it in self.items.slots.iter_mut() {
            if it.b.live != 0 && !(it.b.k20 == 0 && it.b.mode == 2 && it.b.sub == 8) {
                it.b.mode = 6;
                it.b.sub = 0;
            }
        }
        let g = &mut self.spawn.g;
        g.b4f1 = 0;
        g.b4fc = 0;
        g.b504 = 0;
        g.b505 = 0;
        g.kill_all = 0;
    }

    fn spawn_area_script(&self) -> &'static dino_assets::enemy::AreaScript {
        &self.assets.enemy.stages[self.w.stage as usize].areas[self.cam.s2.area as usize & 3]
    }

    /// 0xFBF4: spawn lists driven by camera X. Runs last in the frame.
    pub fn spawn_script_tick(&mut self) {
        if self.spawn.init != 0 && self.spawn.area != self.cam.s2.area {
            // TEMP (see module doc): area changed without spawn_reset.
            self.spawn_area_reset_objects();
            self.rank_area_start();
            self.rank_time_area_start();
            self.spawn_reset();
            self.spawn.g.area_walk = false;
        }
        if self.spawn.init == 0 {
            self.spawn.l1 = 0;
            self.spawn.l2 = 0;
            self.spawn.area = self.cam.s2.area;
            self.spawn.init = 2;
            return;
        }
        let sc = self.spawn_area_script();
        // List 1: 14-byte records.
        while let Some(&r) = sc.list1.get(self.spawn.l1) {
            if !self.spawn_trigger_reached(r.trig) {
                break;
            }
            if r.flags & 0x80 != 0 {
                self.spawn_command(&r);
                self.spawn.l1 += 1;
                continue;
            }
            match self.spawn_alloc(r.flags) {
                None => {
                    self.spawn.l1 += 1; // the record is dropped when the pool is full
                    break;
                }
                Some(h) => {
                    let b = self.body_mut(h);
                    b.live = 1;
                    b.k20 = r.class;
                    b.set_xi(r.x as i16);
                    b.set_yi(r.wc as i16);
                    b.set_zi(r.z as i16);
                    b.var26 = r.var;
                    self.spawn.l1 += 1;
                }
            }
        }
        // List 2: 16-byte records.
        while let Some(&r) = sc.list2.get(self.spawn.l2) {
            if !self.spawn_trigger_reached(r.trig) {
                break;
            }
            let ok = self.spawn_record(&r, false);
            self.spawn.l2 += 1;
            if !ok {
                break;
            }
        }
    }

    /// Trigger test of 0xFC7E / 0xFD06: camera X unsigned (`bhi`), or camera Y signed (`blt`)
    /// in a vertical scene.
    fn spawn_trigger_reached(&self, t: u16) -> bool {
        if self.w.vertical_scene != 0 {
            (t as i16) >= (self.cam.s2.cam_y as i16)
        } else {
            t <= self.cam.s2.cam_x
        }
    }

    /// Allocator by pool selector (table 0xFD46): 0 A, 1 B, 3 E. Returns the object handle.
    pub fn spawn_alloc(&mut self, flags: u16) -> Option<Handle> {
        match flags & 7 {
            0 => self.enemies.alloc().map(|i| self.enemies.handle(i)),
            1 => self.props.alloc().map(|i| self.props.handle(i)),
            3 => self.items.alloc().map(|i| self.items.handle(i)),
            _ => panic!("spawn record with pool selector {:#x} (null allocator in 0xFD46)", flags & 7),
        }
    }

    /// 0xFDAE (`screen_x` = false) / 0xFE6A (`screen_x` = true: X += camera X). Returns false
    /// when the pool was full (carry set). Records skipped by their conditions return true.
    pub fn spawn_record(&mut self, r: &SpawnRec, screen_x: bool) -> bool {
        let f = r.flags;
        if f & 0x80 != 0 {
            self.spawn_command(r);
            return true;
        }
        match f & 0x60 {
            0x40 if self.w.players_minus_one == 0 => return true,
            0x60 if self.w.players_minus_one != 2 => return true,
            _ => {}
        }
        if f & 0x18 != 0 {
            let hi = f >> 8;
            if f & 0x10 != 0 {
                if hi < self.w.level {
                    return true;
                }
            } else if hi > self.w.level {
                return true;
            }
        }
        let Some(h) = self.spawn_alloc(f) else { return false };
        let x = if screen_x { r.x.wrapping_add(self.cam.s2.cam_x) } else { r.x };
        let b = self.body_mut(h);
        b.live = 1;
        b.k20 = r.class;
        b.set_xi(x as i16);
        b.set_zi(r.z as i16);
        b.var26 = r.var;
        self.enemy_set_spawn_extras(h, r.wc, r.b5b, r.b96);
        true
    }

    /// Commands (table 0xFD60).
    fn spawn_command(&mut self, r: &SpawnRec) {
        match r.class {
            0x00 => self.spawn.l1 += 1,                 // 0xFD74: skips one list-1 record
            0x04 => self.rank_reroll_aggression(),           // 0xFD88 -> 0x182A2
            0x08 => {
                // 0xFD8E: force aggression while camera X < arg6
                self.rank.force_x = r.x;
                self.rank.force_val = r.wc;
            }
            0x0C => self.rank_set_time(r.x),                 // 0xFD9C
            0x10 => self.rank_add_time(r.x),                 // 0xFDA4 -> 0x104A8
            c => panic!("unknown spawn command {c:#x}"),
        }
    }
}
